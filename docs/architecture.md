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
Step { number: Option<u32>, title: Option<TextRef>, span, actions: Vec<Action> }
Action { kind: ActionKind, span, origin: RecognizerId }

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
Expectation { exit: Success | Failure | Code(n) | Unknown, lines: Option<TextRef>, prose: Option<TextRef> }
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
  `.`, no empty segments, no drive letters. Symlink escapes are refused by the workspace at write
  time.

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
format opts in, D32). Blocks between a task heading and its first step form an unnumbered **intro
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

A state machine over Task → Step → Action. Per action: `Pending → Previewed → Running →
Done | Failed | Skipped`. Per task, the lifecycle of your §6.1 proposal:

1. Show the task, its manifest and its actions (with `Note`/`Manual` in place).
2. **Preflight**: manifest Create targets absent, Modify targets present, the first patch's
   preimage blob matches the file on disk, `git status --porcelain` empty when the plan says so.
3. Execute steps in order; each step's file actions as one transaction (§6.2), scripts in sessions
   (§6.3), stopping per policy (§6.5).
4. Commit only through the plan's own commit script, and only when the task's gate/verify passed
   (D22).
5. Journal every transition (§6.6).

### 6.2 Workspace and files

- **Step transactions** (D23): all file actions of a step, up to the next non-file action, are
  validated against an in-memory copy first (create targets absent, overwrite targets present,
  every patch applies with `git apply --check`, every `before` occurs exactly once). Only then are
  they written. One failure writes nothing.
- **Pre-images** of every file about to change are stored in git's object database
  (`git hash-object -w`) and their ids journaled: rollback is `git cat-file`, untracked files
  included, no extra storage.
- **Writes are atomic**: write a temp file in the same directory, then rename.
- **Line endings** (D10): created files are written as in the plan (LF). Overwrites and
  `ReplaceExact` keep the target's existing convention unless `.gitattributes` says `eol=lf`;
  `before` is matched in LF space. Patches go through `git apply` unchanged.
- **Patches** go through `git apply --whitespace=nowarn -` with the patch on stdin (D20): git's
  exact semantics, no temp file, no fuzz. The `index` line's blob ids are checked before (preimage)
  and after (postimage). Blob ids are computed with `git hash-object --path=<p>` so clean filters
  and `autocrlf` match git's view; an in-process SHA-1 is only a fast path when no filter applies.
- **The working tree stays clean of planrun**: nothing is written there except what the plan
  writes. State and scratch go under `.git/planrun/` and the OS temp directory.
- The absorbed **patch ceremony** (`save the block as $env:TEMP\…`, `git apply`, `Remove-Item`)
  is never run (D17).

### 6.3 Shell sessions

One persistent session per shell kind per run (D24): `pwsh` for `powershell`/`pwsh`/`ps1` fences,
`bash` for `bash`/`sh`. Variables, functions and the working directory persist across blocks, as
in Rens' terminal (`$gate` set in the Gate block is read in the commit block; DevReady plans set
`$P2` once).

Protocol: each block is written to a temp file outside the repository and dot-sourced (`. 'file'`
/ `source file`), wrapped so that a per-run random sentinel line carrying the exit status follows
it, even when the block throws. Output is streamed to the UI line by line and captured for the
matcher. Sessions start at the repository root with `-NoProfile`.

Windows with PowerShell 7 is the primary platform (D19); Linux with pwsh and bash is supported.

### 6.4 Expectations

- **Exit**: `Success` = 0; `Failure` = non-zero; `Code(n)`; `Unknown` = not checked.
- **Summary lines** (fenced `Expected …:` blocks): every expected line must match an output line,
  in order (a subsequence); `…` matches any run of characters; trailing whitespace ignored.
- **Prose** (`Expected: all pass`): mapped to an exit expectation by the format's phrase table,
  checked, and then **shown for confirmation**. Prose never passes a script on its own (D11).
- **Expected failures** (`Expected: build error`, "run the test to verify it fails"): recorded in
  the IR, **skipped by default** (D12). `--run-expected-failures` runs them.
- A mismatch stops and offers: accept (journaled as accepted-with-mismatch), retry, open a shell,
  abort.

### 6.5 Policy

Every runnable thing has a class (D13): `Read` < `Build` < `Modify` < `Destructive`. File actions
are `Modify`. A script's class is the highest class of any command in it, by a
`CommandClassifier` (known heads: `git status|diff|log|show`, `rg`, `Get-Content` → Read;
`cargo|dotnet|npm|npx test|build|check|ci` → Build; `git add|commit|apply|rm|switch|checkout`,
`replace … -All`, `Set-Content` → Modify; `Remove-Item -Recurse`, `rm -rf`, `git reset --hard`,
`git push`, `git clean` → Destructive). **Unknown commands are Destructive.**

Defaults: file actions run automatically; Build and Modify scripts ask; Destructive scripts show
the script and ask for a typed `yes`; `Manual` always waits. `--auto <classes>` widens this.

### 6.6 Journal and resume

JSONL at `.git/planrun/<plan-fingerprint>.jsonl` (D18), one record per transition: run start
(plan fingerprint, HEAD, repo root), action start (`Running`), action end (outcome, output digest,
pre-image ids, post-state file hashes), task end, user decisions.

Resume never trusts the journal alone (D14). For each action recorded `Running` or `Done`, the
handler reports `DoneState` from the disk: postimages present → `Done`; preimages present →
`NotDone`; neither or mixed → `Unknown`, and the user decides. A changed HEAD or a plan whose
fingerprint differs from the journal's stops the resume with an explanation.

## 7. CLI

```
planrun check  <plan> [--format <id>]          parse + checks + coverage; writes nothing
planrun show   <plan> [P1-T2[/1]]              the IR, a task, or a step
planrun run    <plan> [--from P1-T2] [--to P1-T3] [--auto read,build] [--dry-run]
               [--run-expected-failures]
planrun resume <plan>
planrun status <plan>
```

`--dry-run`: parse, check, preflight against the real repository, then run every file action
against a **virtual tree** in memory (patches with `git apply --check` in sequence). Runs no script,
writes nothing.

The interactive screen (line prompts first, D15):

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
