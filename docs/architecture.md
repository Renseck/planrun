# planrun — Architecture

planrun executes structured Markdown implementation plans against a local git repository, task by
task and step by step, the way Rens executes them by hand. It is **not an agent**: it never
decides what to write. Every byte it writes and every command it runs is in the plan. Where the
plan's instruction is not mechanical, planrun shows it and waits.

Decisions are numbered `D<n>` and recorded in [decisions.md](decisions.md). The plan formats are
specified in [plan-grammar.md](plan-grammar.md); how to add one is in [extending.md](extending.md);
the build order is in [roadmap.md](roadmap.md).

## 1. Priorities

In this order, and they decide every trade-off (D25):

1. **Correctness.** Never misread an instruction, never change a file the plan doesn't name,
   never apply an approximate patch, never mark something done before its effect is verified.
2. **Maintainability and extensibility.** New plan styles keep appearing (§5); adding one must not
   touch execution.
3. **Execution speed.** No wasted process starts, no repeated reads. (The tool's own overhead is
   milliseconds; `cargo`/`dotnet`/`npm` dominate wall time. Don't optimize I/O speculatively.)
4. **Low memory.** The plan is held once; text is referenced by span, not copied. Only the current
   task's files are cached.
5. **Predictability.** The same plan and the same starting tree give the same behaviour.

## 2. Shape

```
                       ┌───────────────────────────────────────────────┐
                       │ planrun-cli   check · show · run · resume     │
                       │               Ui (prompts), policy approval   │
                       └───────────────┬───────────────────┬───────────┘
                                       │                   │
              ┌────────────────────────▼──────┐   ┌────────▼──────────────────────────────┐
              │ planrun-parse                 │   │ planrun-exec                          │
              │ source → document → format →  │   │ Runner · Workspace · Git · Shells ·   │
              │ outline → recognizers → check │   │ Expect · Policy · Journal · Handlers  │
              │ Registry (formats, factories) │   │ HandlerSet (handlers per action kind) │
              │ never touches the repo        │   │ never reads Markdown                  │
              └────────────────────────┬──────┘   └────────┬──────────────────────────────┘
                                       │                   │
                               ┌───────▼───────────────────▼───────┐
                               │ planrun-model  the typed IR       │
                               │ Plan · Task · Step · Action ·     │
                               │ Span/TextRef · RepoPath · BlobId  │
                               └───────────────────────────────────┘
```

Four crates. The dependency arrows are the architecture (D2):

- `planrun-model` depends on nothing of ours. It is the contract.
- `planrun-parse` depends on the model only. It cannot execute: it has no process, git or write
  code, and the compiler enforces that.
- `planrun-exec` depends on the model only. It cannot be tempted to re-parse Markdown.
- `planrun-cli` composes them.

Rust has no inheritance. "Inheritance and composition" here means: **traits** define each
extension point (with default methods where a shared behaviour exists), **composition** builds
formats from recognizer instances and the runner from handlers, and **registries** are the
composition roots. Nothing is a deep type hierarchy.

## 3. The IR (planrun-model)

```
Plan { source: PlanSource, format: FormatId, meta: PlanMeta, tasks: Vec<Task> }
Task { id: TaskId { raw, ordinal }, title, span, manifest: Manifest, steps: Vec<Step> }
Manifest { entries: Vec<ManifestEntry>, span }
ManifestEntry { path: ManifestPath (File(RepoPath) | Pattern(PathPattern)), change, lines, span, origin: PathOrigin }
Step { number: Option<u32>, title: Option<TextRef>, span, actions: Vec<Action> }
Action { kind: ActionKind, span, origin: RecognizerId, path_origin: PathOrigin }
PathOrigin = Written | Sibling | FileName | OnlyEntry | Bare | PreviousEdit

ActionKind =
  | WriteFile    { path, body, mode: Create | Overwrite | FromManifest }
  | ApplyPatch   { path, patch, pre: Option<AbbrevBlobId>, post: Option<AbbrevBlobId> }
  | ReplaceExact { path, before, after }
  | Delete       { path, via_git }
  | Generated    { path }
  | RunScript(Script)
  | Commit(Script)
  | Note         { text }
  | Manual       { reason, text }

Script { shell: Pwsh | Bash, body, expect: Option<Expectation>, intent: Verify|Gate|Preview|Apply|Other }
Expectation { exit: Success | Failure | Code(n) | Unknown, lines, prose, comments: Option<TextRef> }
```

- **Text is never copied.** `TextRef::Span` points into the normalized plan text owned by
  `PlanSource`. `TextRef::Owned` exists only for text that isn't contiguous in the source (a fence
  inside a list item whose indentation CommonMark strips).
- **Normalization happens once** (D6): BOM stripped, CRLF/CR → LF, at load. Every span, fence body
  and line number refers to the normalized text. A fence body is exact: what a full-contents write
  writes is byte-identical to the block.
- **Phases are metadata** (`PlanMeta::phase`), not a tree level (D5): one plan file is one phase.
- **The action set is closed** (D4). Plan styles vary; what a plan can make planrun *do* doesn't.
  A new style maps its wording onto these actions. Adding a variant is a deliberate, cross-cutting
  change (extending.md §4).
- **`Note` vs `Manual`** (D21): prose no recognizer consumed becomes `Note` (shown in place, never
  blocks). A fenced block no recognizer consumed, prose matching the format's `manual_markers`, and
  prose edits ("add these members to the class") become `Manual` (always blocks until
  acknowledged). Nothing is dropped.
- **`Generated`** is checked to exist after the task's last non-commit action, before the
  commit. A path a task names as generated more than once (an intro notice and a step notice)
  is one check.
- **`RepoPath`** (D7) is the only path type in actions: relative, forward slashes, no `..`, no
  `.`, no empty segments, no drive letters, no wildcards (D42), no spelling Windows reserves or
  git refuses (`:`, `.git` in any form, device names, D51). Symlink escapes are refused by the
  workspace at write time (D52). `Manifest::scopes` and `Manifest::is_bare_target` decide D50's
  scope for parse and exec alike.

## 4. Parsing (planrun-parse)

Pipeline: **source → document → format selection → outline → recognition → checks.** Parsing is
total before anything executes (D3): a plan with any error is never returned.

### 4.1 Document

pulldown-cmark's offset iterator builds a flat `Vec<Located>` of blocks: `Heading`, `Paragraph`,
`ListItem` (with `task_box` for `- [ ]`), `Fence { info, body, fence_len }`, `Table`, `Quote`
(a blockquote, kept whole like a table), `Rule`, `Html`. Each block has its span and list depth.
Recognizers never see pulldown-cmark types.

Paragraph and list-item text is the **raw source** (inline markup kept): patterns like
``^File `(?P<path>[^`]+)` \(full contents\):$`` match what is written.

pulldown-cmark implements CommonMark fence rules correctly (a ```` ```` ```` fence contains ```` ``` ````
fences). One CommonMark behaviour is refused explicitly: an **unclosed fence** runs to the end of
its container in CommonMark; planrun reports `unclosed-fence`, because it would silently swallow the
rest of the plan. The usual form of that bug looks closed: the next block's opener
(```` ```powershell ````) becomes body text, and that block's closing fence closes the first one.
So inside an open fence, a line indented at most 3 spaces that opens a fence of the same
character, at least as long as the opener, with a non-empty info string, is `unclosed-fence` too,
reported at the opener with that line as a related span. A ```` ````markdown ```` fence holding
```` ```rust ```` blocks stays legal.

### 4.2 Format selection

Each registered `Format` scores the document 0–100 (`detect`): each of its patterns that matches
a line outside fenced blocks adds its weight once. The highest score at or above the registry
threshold wins; no score at the threshold is `no-format`, a tie for the highest is
`ambiguous-format`. `--format <id>` overrides (`unknown-format` for an id no format has). Never a
guess.

### 4.3 Outline

The format finds: plan metadata, tasks (heading pattern, `ord` capture for ordering), each task's
manifest (marker + item pattern), and step boundaries. The manifest's blocks are consumed by the
outline: they fall in no step, and coverage counts them as "manifest"; its items end at the first
step boundary (D37). A format with a manifest pattern requires one per task (`manifest-missing`)
unless it sets `required = false` (D27); an item that does not match is `manifest-item`. Paths in
items may be completed from the item itself (`:lines` tokens, and sibling file names where the
format opts in, D32; bare file names otherwise, D45) and give one path its own change (D44).
A manifest may also be written inline in its marker paragraph (D43); what such a paragraph
declares nothing with is shown as a Note at the start of the intro step. Ranges and globs are
pattern entries (D41). Blocks between a task heading and its first step form an unnumbered **intro
step** (it can hold a `Generated` notice or `Manual` text). A heading at the task level inside the
tasks that is not a task is `bad-task-heading`. A task with no step boundaries is one implicit
step. A step title can carry a **mark** (`[[step_marks]]`, D30): expected failure or human check.
Text after the tasks (a heading that is no task, at or above `tasks_end_level`) is not executed:
`show` lists it; `check` reports `Manual` markers in it as notes. A task heading after that point
is `task-after-end` (D37).

### 4.4 Recognition

For each step, a cursor walks the step's blocks. At each position the format's recognizers are
tried **in order**; the first that answers anything but `No` wins:

- `Yes(actions)`: append them (possibly none: the absorbed patch ceremony), cursor advanced.
- `Amend(action)`: replace the previous action (an `Expected …` block attaching to its script).
- `Malformed(diagnostic)`: the idiom matched but is broken (a lead sentence with no fence, a patch
  whose paths disagree with its lead). This is a **parse error, never a fallback** to Manual.

If no recognizer matches, the built-in fallback consumes one block: a paragraph or list item
becomes `Note` (or `Manual(Marked)` on a manual marker, `Manual(HumanCheck)` in a human-check
step); a fence becomes `Manual(Unrecognized)`, or
`Manual(ForeignLanguage)` for `sql` and other non-shell languages, and a `text` fence becomes
`Note`; tables, blockquotes (or `Manual(Marked)` on a marker), headings and rules become `Note`;
HTML comments are dropped (they are corpus metadata).

Recognizers are pure (no I/O, no state between calls) and see a `StepContext`: the document, the
task id, the manifest, the format settings, the step's previous actions, whether the step starts
at a written boundary, and its mark. The cursor also shows the block before the current one, so
`script` can require a run lead (D28). After recognition, an expected-failure step's verify
scripts expect failure (`expected-conflict` when one says success).

### 4.5 Checks

`check::Rule` implementations run over the recognized plan and report stable codes
(`duplicate-task-id`, `manifest-undeclared`, `blob-chain`, …; the list is in
[check.rs](../crates/planrun-parse/src/check.rs) and pinned by `tests/corpus/invalid`). Pure
checks only. A check that needs the repository (the first preimage against the file on disk) is a
preflight in planrun-exec.

Rules run only when the outline and recognition had no errors, so a broken block is reported
once, not again as a missing manifest touch. A format's `[checks]` table may make
`manifest-untouched` and `manifest-undeclared` warnings or switch them off, and nothing else
(D31).

The **coverage report** is part of `check`, not a rule: per task, how many blocks became actions,
were absorbed (the patch ceremony, listed with their positions), were the manifest, or became
`Note`/`Manual`. It is how you see where a plan stops being automatic.

## 5. Extensibility

Three levels, cheapest first. A new plan style should use the lowest level that works.

| Level | You write | Rebuild? | Example |
|---|---|---|---|
| 1. Format spec | a TOML file: detection, outline patterns, an ordered list of configured built-in recognizers | no | a plan that writes `**Stap 1 — …**` and `Bestand \`x\` (volledig):` |
| 2. Recognizer kind | a `RecognizerFactory` + `Recognizer` in Rust, registered by kind name | yes | a new idiom: "insert after line N", a YAML front-matter step list |
| 3. Format | a `Format` implementation in Rust | yes | a plan whose tasks are a table, or JSON |

Built-in formats are **level-1 specs** too (`formats/*.toml`, embedded with `include_str!`): the
path a user format takes is the path every built-in format takes (D8). User formats load from
`.planrun/formats/*.toml` in the target repository, then the user config directory; a duplicate id
is an error.

On the execution side, the extension points are traits with registries too: `ActionHandler` per
action kind (`HandlerSet`), `ShellSession` per shell, `CommandClassifier` for policy, `Ui` for the
front end. They change rarely; formats change often. Details: [extending.md](extending.md).

## 6. Execution (planrun-exec)

### 6.1 Runner

`runner::Runner`: a state machine over Task → Step → Action, talking to a `Ui` (§7). Per action:
`Pending → Running → Done | Failed | Skipped`; per task:

1. Show the task.
2. **Preflight** (`preflight::preflight`): no touched path has uncommitted changes
   (`path-dirty`; other dirt is a warning, D55); manifest Create entries absent, Modify, Rewrite
   and Delete entries present (patterns and Bare entries are skipped: they name no single known
   file, D41, D45); then the task's file actions up to its first script or Manual, simulated on
   a fresh virtual tree: every preimage blob, every patch, every `before`. A Bare-targeted
   Overwrite or Delete is `path-ambiguous`; a git delete of an untracked file is
   `delete-untracked` (D78). Any error stops the run. `task-start` is journaled.
3. The steps in order. Consecutive file actions are one transaction (§6.2): an unscoped one asks
   first (D50), `step-prepared` is the write-ahead record, `step-written` follows. Scripts run
   one at a time in their shell's session (§6.3) per their decision (§6.5): `action-start`, the
   block streamed to the `Ui`, the matcher's verdict (§6.4) and what it asks, `action-end`. A
   Manual waits; a Note is shown.
4. Before the Commit: the task's `Generated` paths are checked once each, a Gate from a session
   that has ended runs again, dirt outside the task's paths is asked about (D80), and the commit
   gate (D75) must be open or opened with a typed `yes`. The plan's own commit script then runs
   as written; it is made only when HEAD moved by one commit holding the task's paths (D79).
   After it, paths the commit changed outside the task are reported.
5. `task-end`. Every way a run ends is journaled as `run-end`, except a kill (§6.6).

### 6.2 Workspace and files

- **The virtual tree** (D52): a temporary index and object directory under the OS temp
  directory, with the repository's objects as alternates, passed to git per call
  (`GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`). Paths are seeded
  from the working tree on first use. Each `ActionHandler` simulates its action on it: create
  targets absent, overwrite targets present, every patch with `git apply --cached` after
  `--check`, every `before` exactly once. With `core.ignorecase`, two spellings of one path are
  `path-collision`.
- **Step transactions** (D23, D53): `transaction::prepare` simulates the file actions of a step,
  up to the next non-file action, on a virtual tree seeded from the disk, stores the pre-images,
  and builds every post-image there: a patched file's bytes are `git cat-file --filters`, the
  conversion `git apply` would make; the others are D54 bytes. `transaction::commit` hands the
  `PreparedStep` to a write-ahead recorder (P5: the journal), then writes it. One failure in
  prepare writes nothing; one in commit is rolled back from the pre-images, and what could not
  be restored is reported. A `Delete` with `via_git` updates the index last, in one call.
- **Pre-images** of every file about to change are stored in git's object database
  (`git hash-object -w --no-filters`: the exact bytes) and their ids journaled: rollback is
  `git cat-file`, untracked files included, no extra storage. They are unreachable objects;
  `git gc` may prune them after two weeks; only resume's `[r]` needs them, and it checks first
  (D57, D77).
- **One writer** (D52): the path is resolved first (no link, junction, short name or case
  mismatch in its existing part); directories are created one segment at a time, each checked;
  the contents go to `.planrun-<random>.tmp` in the target's directory (`create_new`); the parent
  is checked again; the rename replaces the target. A parent swapped between that check and the
  rename is not caught.
- **Line endings** (D10, D54): created files are written as in the plan (LF). For Overwrites and
  `ReplaceExact`, an `eol` attribute decides, `-text` keeps bytes; otherwise the file's own
  convention; a file without line endings takes git's checkout convention; a mixed file is
  overwritten by the checkout convention and refused for an exact edit (`eol-mixed`). `before` is
  matched in LF space. Patches go through git.
- **Patches** go through `git apply --whitespace=nowarn -` with the patch on stdin (D20): git's
  exact semantics, no temp file, no fuzz. The `index` line's blob ids are checked before (preimage)
  and after (postimage). Blob ids are computed by `git hash-object --path=<p>`, so clean filters
  and `autocrlf` match git's view; planrun-exec computes none in-process (D56).
- **The working tree stays clean of planrun**: nothing is written there except what the plan
  writes, and a writer's temp file until its rename. State and scratch go under `.git/planrun/`,
  pre-images into the object database, the virtual tree into the OS temp directory. Status calls
  run with `--no-optional-locks`, so they do not refresh `.git/index`.
- The absorbed **patch ceremony** (`save the block as $env:TEMP\…`, `git apply`, `Remove-Item`)
  is never run (D17).

### 6.3 Shell sessions

One persistent session per shell kind per run (D24): `pwsh` for `powershell`/`pwsh`/`ps1`
fences, `bash` for `bash`/`sh`/`shell` (the spec defaults, extending.md §2). Variables,
functions and the working directory persist across blocks, as in Rens' terminal (`$gate` set in
the Gate block is read in the commit block; DevReady plans set `$P2` once). `shell::Session`
implements `ShellSession`; P5 keeps one per shell.

- **Which shells** (D63): `pwsh` from `PATH` when it is PowerShell 7 or later, never Windows
  PowerShell 5.1. On Windows, bash is Git for Windows' `<root>\usr\bin\bash.exe`, `<root>` found
  from `git --exec-path` and checked with `uname -s` (MINGW or MSYS), never `bash` from `PATH`
  (WSL); its `PATH` gets `/usr/bin` in front. Elsewhere `bash` from `PATH`.
- **Protocol** (D60): sessions start at the repository root with `-NoProfile`
  (`--noprofile --norc`), detached from planrun's terminal. Each block is written to a file in a
  fresh run directory under the OS temp directory (never the working tree, D7), whose path comes
  through the environment. Blocks get no stdin: pwsh has none at all and its `-Command` script
  is a loop that dot-sources each block file as it appears; bash gets one ASCII line per block,
  `source "<dir>/b<n>.sh" </dev/null`, at the top level. After each block a sentinel with a
  per-session secret and the block's number arrives on stdout (status, `errored`, the working
  directory) and on stderr; the block ends when both have. Output is streamed line by line with
  its stream and kept for the matcher; order across the two streams is arrival order.
- **How a block ends** (D61): `Exited(code)` (pwsh `$LASTEXITCODE`, reset before the block;
  bash `$?`), `Threw` (pwsh), `SessionEnded` (bash `exit`, or the shell gone), `TimedOut`,
  `Broken` (a sentinel that does not fit). pwsh also reports `errored`: `$Error[0]` changed.
- **Environment** (D62): planrun's own, without `BASH_ENV`, `ENV` and the git wrapper's
  variables, with `GIT_TERMINAL_PROMPT=0`.
- **The process tree** (D64, D71, D73): a job object on Windows, `setsid` and `PR_SET_PDEATHSIG`
  on Linux; a timeout (none by default, D65), a drop, or a shell that exited while a descendant
  holds its pipes kills every descendant. On Unix SIGINT, SIGTERM and SIGHUP kill the sessions'
  groups before planrun ends; while the `[o]` shell runs, Ctrl-C goes to it. `shell::end_now`
  ends planrun the same way for a UI that reads Ctrl-C as a key (D91). `shell/kill.rs` holds
  planrun's only `unsafe` code.
- **In a run** (D81): one session per shell, started at the first block; a dead one is a
  mismatch whose retry starts a new one, announced by a notice that must be acknowledged (also
  on resume and with `--from`); a working directory outside the repository asks before the
  next block.
- **Stated differences from the terminal:** `$LASTEXITCODE` is reset to 0 before each block; a
  terminating error ends the block; `$PSScriptRoot` and `$MyInvocation` name the block's temp
  file; pwsh `exit n` ends only the block, bash `exit n` the session; `GIT_TERMINAL_PROMPT=0`.

Windows with PowerShell 7 is the primary platform (D19); Linux with pwsh and bash is supported.

### 6.4 Expectations

`expect::check` turns a run and its `Expectation` into `Pass`, `Mismatch` or `Confirm` (D66).

- **Exit**: `Success` = 0; `Failure` = non-zero or a throw; `Code(n)`; `Unknown` = not checked.
  A session that ended, timed out or broke is a mismatch whatever was expected.
- **Summary lines** (fenced `Expected …:` blocks): every non-blank expected line must match an
  output line, in order (a subsequence); `…` matches any run of characters, the empty one
  included, and only `…`; the first literal segment is anchored at the start, the last at the
  end. Output is split on LF, trailing whitespace is stripped on both sides (a CR with it), ANSI
  CSI and OSC sequences are stripped from the output; leading whitespace counts.
- **Prose** (`Expected: all pass`) and **comments** (`# main` after a command, D35, D48): the
  exit status the format's phrase table derived is checked, and then the text is **shown for
  confirmation**, the two labelled apart. Prose never passes a script on its own (D11).
- **Asking without prose:** an exit the plan does not state that is not 0 (or a throw), and a
  pwsh `errored` run where nothing but an exit status was expected, ask too. Every
  confirmation shows `errored`.
- **Expected failures** (`Expected: build error`, "run the test to verify it fails"): a Verify
  script expecting failure is **skipped by default** (D12, D67), decided by the policy before
  the run; `--run-expected-failures` runs it. Gates, previews and applies always run.
- A mismatch stops and offers accept (journaled as `accepted`; it closes the commit gate), retry,
  open a shell, abort; a dead session offers only retry, shell and abort (D75).
- `expect::align` gives each non-blank expected line the output line it matched, or none, as
  `check` matches them; the `mismatch` question carries it (D86).

### 6.5 Policy

Every runnable thing has a class (D13): `Read` < `Build` < `Modify` < `Destructive`. File actions
are `Modify`. A script's class is the highest class of any command in it, by the
`DefaultClassifier` (D68): a built-in allowlist that no spec or flag changes, over a splitter
per shell that recurses into subexpressions, script blocks, command substitutions and
expanding here-strings and heredocs. What it cannot take apart with confidence, and every
command its tables do not know, is **Destructive**.

| Class | Examples |
|---|---|
| Read | `git status`, `diff`, `log`, `show`, `branch` (listing), `rev-parse`; `rg`; `Get-Content`, `Get-ChildItem`, `Select-String`, `Test-Path`, `Sort-Object`, `ConvertFrom-Json`, `Set-Location`; bash `cat`, `ls`, `head`, `grep`, `echo`, `test`; `replace … -DryRun`; DevReady functions whose agent mode is `safe` |
| Build | `cargo build`/`test`/`check`/`clippy`/`fmt --check`/`doc`; `dotnet build`/`test`/`restore`; `npm ci`/`test`/`run`; `go build`/`test`/`vet`; DevReady `test` |
| Modify | `git add`, `commit`, `apply`, `rm`, `mv`, `switch`, `checkout -b`, `stash`, `reset` (soft, mixed); `cargo fmt`, `fix`, `add`; `npm install`; `dotnet format`; `go mod tidy`; `New-Item`, `Set-Content`, `Add-Content`, `Copy-Item`, `Move-Item`, `Remove-Item` (no `-Recurse`); `mkdir`, `cp`, `mv`, `rm` (no `-r`), `sed -i`; `replace … -All`; a redirection to a relative path |
| Destructive | `Remove-Item -Recurse`, `rm -r`; `git push`, `fetch`, `pull`, `reset --hard`, `clean`, `restore`, `checkout <name>` or `--`, `switch -f`, `branch -D`, `stash drop`; `git -c` with any key but `color.*`, `advice.*`, `core.quotepath`, `user.name`, `user.email`; `cargo run`/`install`, `dotnet run`, `npx`; `iex`, `Start-Process`, `Invoke-WebRequest`, `curl`; `pwsh -File`, `bash -c`, `cmd /c`, `source`, `eval`; a static or instance .NET call outside a short pure list; a redirection outside the repository; anything unknown |

Defaults (D13, D69): file actions within the task's scope run automatically; every script asks,
Read ones included; Destructive scripts show the script and ask for a typed `yes`; `Manual`
always waits; a file action outside the scope (D50) and an Apply script always ask. `--auto
read,build,modify` (any of them) lets those classes run without asking; `destructive` is
refused. `Policy::decide` computes the decision; the runner prompts for it (D74) and never turns one
decision into another.

### 6.6 Journal and resume

JSONL at `<git-dir>/planrun/<plan file stem>.jsonl` (D18, D76), one record per transition, each
with the schema version and the time: `run` (the plan's path and fingerprint, HEAD, the policy
and range), `task-start`, `step-prepared` (the write-ahead record of a step: per path the
pre-image id, the post-image's ids and mode, the directories it creates, the writer's temp
name), `step-written`, `action-start` (with the decision and HEAD; for a Commit the task's paths
as on disk), `action-end` (outcome, status, the last output lines, HEAD), the user's answers,
`gate-override`, `task-end`, `run-end`. One write per record; `step-prepared`, `action-start`
and `run-end` are synced. A torn last line is dropped, and cut off before the next append; any
other bad line is an error. One planrun runs per repository at a time: `<git-dir>/planrun/lock`
(D77). A run also holds `lock.live`, with a blocking lock once `lock` is held, so `lock::live`
sees a run in progress with a try on `lock.live` alone, never on `lock` (D90).

Resume never trusts the journal alone (D14, D77). It reads the last chain (a `run` and the
`resume`s after it) and stops when the plan's fingerprint differs, when `.git/index.lock`
exists, or when HEAD is not what the journal last recorded, except for the one action that was
running (a Commit found made is done, D79; for anything else the user decides). The interrupted
task runs again from its start in a new session. Each prepared step is compared with the disk
(`done::step_state`): every path as planned → `Done`, not written again; every path as before →
`NotDone`, written again; anything else → `Unknown`: restore it from its pre-images (when `git
gc` has left them, D57) or accept it with a typed `yes`. Scripts run again under their
decisions; a Modify or Destructive one that ran, and any that was running, asks first. Dirt in
a path that no verified step wrote and that a remaining file action writes over may be the
user's: it is asked about once (`resume-dirt`). A resume's own `run` record never sets the
recorded HEAD; where resume accepts a moved HEAD it journals the running action's end (or
`head-seen` for a retry) at once, so a stopped or crashed resume leaves the next one the same
check.

`runner::preview::preview` reads what resume would find without the lock and without writing,
for the TUI's resume preview (D85): whether a run is live, the fingerprint, `index.lock`,
resume's verdict on HEAD (`resume::head_verdict`, the function resume uses), each prepared step
on disk with whether it can be restored and its temp files, the scripts that will ask "run
again or keep" (the rule `replayed` uses), and the `resume-dirt` paths (`preflight_resumed`
itself). Resume checks all of it again when it runs. A journal takes any number of observers (`Journal::observe`), called after each
append and each write of a step's commit in the order they were added: the `e2e` fault seam
(D82) and the TUI's live view (D90).

## 7. CLI

```
planrun check  <plan> [--format <id>]          parse + checks + coverage; writes nothing
planrun show   <plan> [P1-T2[/1]]              the IR, a task, or a step
planrun run    <plan> [--from P1-T2] [--to P1-T3] [--auto read,build] [--dry-run]
               [--run-expected-failures] [--restart]
planrun resume <plan> [--auto …] [--from …] [--to …] [--run-expected-failures]
planrun status <plan>                          what the journal says; changes nothing
planrun update [--check]                       install the latest release over this executable
```

`update` (D83) is the one command that is not about a plan: it compares the latest GitHub release
with the build's version, and when it is newer downloads the build's target's archive with curl,
unpacks it with tar under the OS temp directory, checks that the new executable prints the
release's version, and renames it over the running one (on Windows the running one goes aside as
`planrun.old.exe`, which the next start removes). `--check` only reports. Exit 0 when up to
date, reported or updated, 1 when the update failed; the installed executable is then
unchanged.

`run` executes the plan in the repository of the current directory, under the lock (D77). It
refuses a plan whose journal holds an unfinished run; `--restart` keeps that journal under
another name and starts again. `resume` goes on with the unfinished run, with its policy and
range unless the flags say otherwise. Exit 0 when the run completed, 1 when it stopped (a
failure, a check, a question answered with quit, or no terminal to answer), 2 for usage, git,
the lock or the journal.

`--dry-run` (D55): parse, check, check real dirt once for the paths of every task, then preflight
every task against **one virtual tree** carried across the tasks (D52), so a task sees what the
tasks before it wrote, and simulate every file action. Commits are assumed. Runs no script and
writes nothing: not the working tree, not `.git`. Once a script, commit or Manual has gone by
without running, a failing file action is a warning ("depends on a step that did not run") and
its path is unknown from then on; every ok after that point assumes those steps change no files.
Each script and commit shows its class and what the default policy would do with it ("would ask
(Build)", "typed yes (Destructive)", "skipped: expected failure", D69). The report is typed
(`preflight::DryRun`): a file action that would run carries its `handler::Effect`, whose text is
what the report prints (D84). Exit 0 when nothing fails, 1 when something does, 2 when git or the repository cannot be used.

The prompts are lines on stdout, answered by a line on stdin (D15, D74); only a terminal
answers: a stdin that is not one stops the run at its first question. A sketch of the screen:

```
planrun · greeter · plan assay-style.md (assay) · repo C:\src\greeter

  [✓] P1-T1  the greet module
  [>] P1-T2  re-export greeting, remove the legacy module      step 2/3
  [ ] P1-T3  …

Step 2 — verify
  ▸ RunScript (pwsh, Build)  New-Item … docs/modules.txt
    expect: exit 0; lines: greet.rs · lib.rs

[enter] run   [s] skip   [v] view action   [d] diff so far   [o] open shell   [q] quit
```

## 8. Error reporting

Diagnostics carry a stable code, a span and related spans, rendered with `annotate-snippets`
against the normalized text (file:line:col, the source lines, a caret). Execution failures show the
action, its source line in the plan, the command output tail, and what is on disk now.
