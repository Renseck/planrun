# Decisions

Numbered, never renumbered. A changed decision gets a new number that supersedes the old one;
the old entry stays with a "superseded by" line. Code and docs cite them as `D<n>`.

Agreed between Rens and Claude on 2026-11-24, from two proposals (Rens' "Deterministic Plan
Execution CLI" and Claude's pattern study of six real plans).

## D1 Priorities, and what planrun is

Correctness > execution speed > low memory > predictability > maintainability/extensibility.
planrun is not an agent: it does exactly what the plan says, shows what it cannot do
mechanically, and decides nothing. Rust.

Order superseded by D25.

## D2 Four crates; parsing never executes, execution never parses

`planrun-model` (IR), `planrun-parse` (depends on model), `planrun-exec` (depends on model, **not**
on parse), `planrun-cli`. The crate graph enforces the separation.

## D3 A plan runs only when it parsed and checked without errors

Parse the whole document first; any error means no `Plan`. No partial execution.

## D4 The action set is closed

`WriteFile`, `ApplyPatch`, `ReplaceExact`, `Delete`, `Generated`, `RunScript`, `Commit`, `Note`,
`Manual`. Formats vary, actions don't. Adding one is a cross-cutting change (extending.md §4).

## D5 Phases are metadata

One plan file is one phase in every real plan. `PlanMeta::phase`, not a tree level.

## D6 Normalize the plan once

At load: strip a BOM, CRLF/CR → LF. Every span, fence body and line number refers to the
normalized text.

## D7 RepoPath is the only path in an action

Relative, forward slashes, no `..`/`.`/empty segments, no drive letter, no backslash, no NUL.
Symlink escapes are refused by the workspace at write time. Nothing outside the repository is
written except under `.git/planrun/` and the OS temp directory.

Amended by D42 (wildcards), D51 (Windows-reserved spellings), D52 (pre-images in the object
database, the writer's temp file) and D83 (`planrun update` writes its own install directory).

## D8 Formats are data first; built-ins are specs too

Three extension levels: TOML spec (no rebuild) → recognizer kind (Rust) → Format (Rust). The
built-in formats are TOML specs embedded in the binary, so user formats take the same path.

## D9 Dependencies

Each dependency is justified here; add none without a line.

| Crate | Why |
|---|---|
| `pulldown-cmark` | CommonMark with byte offsets; correct nested-fence handling |
| `regex` | spec patterns; linear time, no backtracking blow-ups |
| `serde`, `toml` | format specs; `.planrun/config.toml` (planrun-exec, D96) |
| `serde_json` | the journal (JSONL); the release description `planrun update` reads (D83) |
| `thiserror` | error types |
| `sha1_smol` | git blob ids in-process: the `blob-chain` check from a created file's contents (planrun-parse only; planrun-exec asks `git hash-object`, D56) |
| `similar` | showing diffs of overwrites and replacements. **Not** for applying patches (D20) |
| `clap` | arguments |
| `annotate-snippets` | diagnostics with source context |
| `insta` (dev) | golden snapshots of the IR |
| `sha2` (Phase 1) | the plan fingerprint (SHA-256) |
| `ratatui` | the TUI's layout and widgets (planrun-tui only, D92) |
| `crossterm` | the TUI's terminal: raw mode, the alternate screen, key events (planrun-tui only, D92) |
| `unicode-width` | the TUI's cell widths, and the test that pins its glyphs to one cell (planrun-tui only, D89) |

Rejected: `rusqlite` (D18), `tracing` for now (the journal is the record),
`similar` as a patch engine (D20), `windows-sys` and `libc` for the process tree (D64), `dialoguer`
(D74: it refuses a stdin that is not a terminal), `ctrlc` (D73: it brings `nix` and
`windows-sys`), `ureq`, `reqwest`, `self_update`, `self-replace`, `zip` and `flate2` for
`planrun update` (D83: curl and tar do it, like git, with no TLS stack in the build).

Amended by D56 (no `sha1_smol` in planrun-exec), D64 (the hand-declared process-tree calls), D73
(`ctrlc` rejected), D74 (`dialoguer` dropped), D83 (`serde_json` in planrun-cli; the update
crates rejected), D92 (the TUI's crates, in planrun-tui only) and D96 (`toml` in planrun-exec).

## D10 Line endings of written files

Created files: as in the plan (LF). Overwrite and `ReplaceExact`: keep the target's existing
convention unless `.gitattributes` says `eol=lf`; `before` is matched in LF space. Patches: through
`git apply` unchanged.

Amended by D54 (attributes, files without or with mixed line endings).

## D11 Prose never passes a script on its own

`Expected: all pass` is mapped to an exit expectation, checked, and then shown for confirmation.
Only fenced summary lines and exit codes pass automatically.

Amended by D66 (the matching rules, what else asks).

## D12 Expected failures are skipped by default

A verification expected to fail (TDD "verify it fails", `Expected: build error`) is recorded in
the IR and skipped; `--run-expected-failures` runs it. Rens sees these failures in the IDE.

Amended by D67 (Verify scripts only).

## D13 Policy classes

Read < Build < Modify < Destructive; a script takes the highest class of its commands; unknown
commands are Destructive. Defaults: file actions automatic; Build/Modify ask; Destructive needs a
typed `yes`; Manual always waits.

Amended by D68 (the classifier, a built-in allowlist) and D69 (`--auto`, Read asks too).

## D14 Resume checks the disk, never the journal alone

An action recorded `Running` or `Done` is re-examined: postimages present → done; preimages
present → not done; otherwise the user decides.

Amended by D77 (resume, per action kind).

## D15 Line prompts first

`dialoguer` and plain output keep cargo's output in the terminal's scrollback. A ratatui TUI can
come later behind the `Ui` trait.

Amended by D74 (line prompts on std, without `dialoguer`) and D84–D92 (the TUI,
docs/tui-design.md).

## D16 Real plans stay local

The six real plans contain customer and internal details. They live in `tests/corpus/local/`
(git-ignored) and are parsed by an `#[ignore]`d test. Committed tests use synthetic plans.

## D17 The patch ceremony is absorbed

The `git apply "$env:TEMP\….patch"` / `Remove-Item` block after an exact diff is never run:
planrun applies the patch itself. There is no `--verbatim` mode.

## D18 The journal is JSONL in `.git/planrun/`

One writer, sequential, a few hundred records per plan: SQLite adds a C dependency and a binary
file nobody can read when it matters. JSONL is appendable, crash-tolerant (a torn last line is
dropped), and readable. Inside `.git/` so `git status --porcelain` stays empty.

Amended by D76 (one file per plan, the records, when they are synced).

## D19 Windows first

Every real plan is PowerShell 7 on Windows. Linux (pwsh and bash) is supported and is where CI
can run.

Amended by D83 (CI runs the gate on Windows and on Linux).

## D20 Patches are applied by `git apply`

With the patch on stdin, `--whitespace=nowarn`, after `--check`. Same semantics as Rens' manual
runs; exact context; no fuzz. Blob ids from `index` lines are verified before and after.

## D21 Note vs Manual

Prose no recognizer consumed is a `Note` (shown, never blocks). An unconsumed fence, prose that
matches a format's manual markers, and edits described in prose are `Manual` (always blocks).

## D22 Commits are gated twice

The plan's commit script runs as written (it may gate itself on `$gate`), and planrun runs it only
when every verification of the task passed or was skipped by policy (D12).

Amended by D75 (the commit gate, a question of its own).

## D23 File actions are transactional per step

All file actions of a step, up to the next non-file action, are validated in memory before any is
written; pre-images are stored in the git object database first; writes are temp-file + rename.

Amended by D52 (the virtual tree, the writer) and D53 (prepare, then commit).

## D24 One persistent session per shell kind per run

Variables and the working directory persist across blocks (`$gate`, `$P2`), as in Rens' terminal.

Amended by D60 (the protocol, and how a session differs from the terminal).

## D25 Maintainability/extensibility ranks second

Supersedes D1's order; the rest of D1 (not an agent, Rust) stands.
Correctness > maintainability/extensibility > execution speed > low memory > predictability.
Reason: new plan styles keep arriving, so planrun is only valuable if adding one stays cheap
and safe; speed and memory rarely decide anything, because cargo, npm and dotnet dominate
wall time.

Decisions D26–D40 were agreed on 2026-11-24 at the start of P2, from the review of the three
hand-written formats against their real plans.

## D26 The file operation decides Create or Overwrite

A lead that says create writes with `Create`, one that says replace writes with `Overwrite`,
whatever the manifest says: Create refuses an existing target at preflight (P3), so the op
deciding is safe. Only the `write` op takes the mode from the manifest. Where a task has a
manifest and its entry disagrees with the op (Create on a Modify/Rewrite/Delete entry, Overwrite
on a Create/Delete entry), `manifest-write-mode` is an error, in every format.

## D27 Optional manifests; `Generated` entries

`[outline.manifest] required` (default true) lets a format have tasks without a Files list; the
manifest checks skip such a task. The marker may capture `change` for formats whose items are
bare paths. `ChangeKind::Generated` is a manifest change of its own (a model variant, not an
action kind, so D4 does not apply): a generated entry needs no action, and `show` reports it
as generated.

## D28 A shell fence runs only behind a run lead in hand-written formats

`script` takes `leads` (regexes over the paragraph directly before the fence) and `first_block`
(the fence is the first block of a written step). With `leads` set, the kind claims a fence only
when one of the two holds; any other shell fence falls to the fallback, `Manual(Unrecognized)`.
`first_block` never applies to the implicit first step, so never to the block right after a
manifest. A body intent (`git commit` in the body) never qualifies a fence on its own: a plan
can create a script file whose contents contain git commands. Reason: hand-written plans put
file contents and code to insert in shell fences behind prose that no recognizer can reliably
name; unguarded, `script` would execute them. The assay spec sets no `leads` and is unchanged.

In a step whose boundary is kept (D46), the boundary label is the step's first block, so
`first_block` never matches there: a fence directly after a kept label runs only when the
label itself matches one of `leads`.

## D29 Script intents by lead; no `preview-apply` kind

An `intents` row of `script` matches the body, the lead paragraph, or both (`body`, `lead`).
`leads` decides whether the kind claims a fence; the per-intent `lead` decides which intent a
claimed fence gets. Preview/apply pairs need nothing more, so the `preview-apply` kind is
dropped.

## D30 Step marks replace the `expected-failure-step` kind

A recognizer cannot see a step's title, so the title is matched by the outline:
`[[step_marks]] title = '…', mark = "expected-failure" | "human-check"`.

- `expected-failure`: every `Verify` script of the step (never a commit or a gate) expects
  failure. A script with no expectation or with `any` becomes `Failure`; one that expects success
  is an error, `expected-conflict`.
- `human-check`: every prose `Note` of the step (a paragraph, list item or quote the fallback
  takes) becomes `Manual(HumanCheck)`; rules and tables stay Notes.

## D31 A spec may downgrade only the manifest coverage checks

`[checks]` maps a check code to `error`, `warning` or `off`, and only `manifest-untouched` and
`manifest-undeclared` may appear in it. Any other key is a spec load error, so a user spec can
never switch off a safety check: `manifest-write-mode` and every outline and recognizer error
stay errors in every format. Reason: in hand-written plans, prose edits (Manual) and scripts
(`replace`) change declared files, and manifests name ranges and globs.

## D32 Paths resolved from the manifest

Deterministic and only against the task's own manifest:

- In a manifest item, a `:lines` token adds lines to the path before it (every format).
- `[outline.manifest] siblings = true`: a token without `/` after a token with a directory is a
  file in that directory. Opt-in, because `crates/x/src/a.rs`, `Cargo.toml` in one item would
  otherwise mean `crates/x/src/Cargo.toml`.
- A recognizer parameter `file_name_from_manifest`: a lead path without `/` that is not itself
  an entry resolves to the one entry with that file name, when that entry was written in full
  (not a sibling). Otherwise the path stays as written.
- `before-after` parameter `infer_path`: a lead with no path takes the task's one entry that is
  not Test, Generated or Delete. Never for writes. With none or several, the lead and its fences
  are `Manual(ProseEdit)`.

Each resolved path carries its `PathOrigin`, and `show` marks it.

## D33 The DevReady `**Commit:** \`msg\`` line is Manual

`Manual(HumanCheck)` carrying the message. planrun synthesizes no commit and never runs
`git add -A`. P5 shows `git add -- <paths>; git commit -m "<msg>"` as a suggestion only when every
manifest path of the task is literal (no glob, range, sibling or unresolved file name);
otherwise it shows the message alone.

## D34 Inline edits are Manual for now

`Edit: \`a\` → \`b\``, `Old: \`a\`` / `New: \`b\`` and similar one-line edits become
`Manual(ProseEdit)`: code spans lose whitespace. Revisit after dogfooding.

## D35 Comment expectations in DevReady scripts

A `script` parameter `line_expect` (a regex over each line) turns trailing comments such as
`# feature/p2-tooling` into one prose expectation per fence: exit `Unknown`, confirmed (D11).
DevReady only. A fence that also has an `Expected:` after it is `expected-twice`.

Superseded in part by D48: an `Expected:` sentence or list after such a fence extends its
expectation instead of being `expected-twice`.

## D36 Imperative prose

Per-format `manual_markers` and `human-check` step marks catch the instructions that would
otherwise be Notes (D21). Each phase plan lists, by file, task and step, the imperative Notes
that remain in the real plans, so every slip is a recorded choice.

## D37 Outline fixes

- A manifest's items end at the first step boundary: a list-item step right after the manifest
  is in the same CommonMark list, and is a step, not an item.
- A heading that matches the task rule is a task; only a heading that is no task, at or above
  `tasks_end_level`, ends the tasks. A task heading after that end is an error, `task-after-end`:
  it would otherwise be text that never runs.

## D38 An expectation after a Manual joins it

An `Expected: …` sentence directly after a `Manual` (an `sql` fence, a fence no lead claimed)
becomes part of that Manual's text, instead of `expected-without-script`. After a `Note` it stays
an error.

## D39 `run-line` names its shell

Each `run-line` recognizer has a `shell` parameter; there is no format default. pwsh for the
superpowers format: its run lines share no variables with its bash commit fences (D24).

## D40 Target for DevReady plans

They are written for an agent (probes, conditional branches, "record"). The target is zero errors
and honest Manual coverage, not automation. P2 is split: P2a builds the schema, the kinds and the
superpowers and before-after formats; P2b the devready format on the same schema.

Decisions D41 onwards were agreed on 2026-11-25 at the start of P2b, from the review of the
devready format against its two real plans.

## D41 Pattern manifest entries

A Files item may name a set of files instead of one: a glob (a token with `*`, `?` or `{`) or
a range (an en dash between two code spans, `` `dir/0001`–`0007_*.sql` ``). Each is one manifest
entry whose target is a `PathPattern`, not a `RepoPath`: `ManifestEntry.path` is a
`ManifestPath`, either a file or a pattern.

- Ranges are found in the item text before `path_list` splits it, in every format, like the
  `:lines` tokens. A spec's `path_list` therefore never lists the en dash.
- Globs: `*` matches any run of characters except `/`, `?` one character except `/`,
  `{a,b}` either alternative. `**`, nested or unbalanced braces are `manifest-item`.
- A range `dir/A`–`B` names the files of `dir` whose name `f` has `A <= f` and, when `B` has
  no wildcard, `f <= B`. When it has one, with `B0` the part of `B` before its first wildcard:
  `f` has at least `|B0|` characters, its first `|B0|` characters are `<= B0`, and the rest of
  `f` matches the rest of `B` as a glob. Comparisons are by character. `B` may repeat `dir`;
  a `B` in another directory, or a wildcard in `A`, is `manifest-item`.
- A pattern is repository-relative as written and never resolved: a pattern relative to a
  subdirectory, or one with prose in it, matches nothing (at worst a `manifest-undeclared`
  warning).
- `manifest-undeclared` accepts a path a pattern entry matches; `manifest-untouched` counts a
  pattern entry as touched when any action path matches it. The write-mode check uses the
  entry's change as for a file.
- A pattern is never a candidate for `file_name_from_manifest` or `infer_path`, and never a
  write target (D42). D33 counts it as non-literal.
- P3 preflight skips the existence checks for pattern entries (architecture.md §6.1).

Handover P2a→P2b open item 1 proposed a `PathOrigin` for patterns; a target type of its own
was chosen instead, so "no write targets a wildcard" is a property of the types.

Amended by D96 (discovery globs, which allow `**`; manifest globs still refuse it).

## D42 RepoPath refuses wildcards

Amends D7: a `RepoPath` also refuses `*` and `?`. Both are invalid in Windows file names
(D19), and a path with one is a pattern (D41). A lead that names such a path is `bad-path`.

Amended by D51.

## D43 Inline Files clauses

A manifest may be written inside its marker paragraph: ``**Files:** create `a`; modify `b`.``

- The marker regex captures `items`. When that capture takes part, the paragraph holds the
  manifest and no list items are taken after it; a list at the marker's depth right after it is
  `manifest-item`. When the marker matches without `items`, list items are taken as before.
- `clause_list` splits `items` into clauses; each clause is matched by the same `item` regex as
  a list item. A clause with a code span that does not match is `manifest-item`.
- Text that declares nothing (a clause without a code span, such as "regenerate the README
  examples", and the text after the `items` capture) is not dropped: it is shown in place at the
  start of the task's intro step, as a `Note`, or as `Manual(Marked)` when it matches a manual
  marker. Coverage counts the marker paragraph once, as manifest. Each phase plan lists these
  leftovers with its imperative Notes (D36).

## D44 A change per path

`[outline.manifest] path_change` is a regex over the text right after a path token (the
separator up to the next token, or the rest of the item after the last one). Its `change`
capture, mapped through `changes`, overrides the change of that one path. The values are
therefore `ChangeKind`s and D26 stays strict: a wrong mapping is a loud `manifest-write-mode`,
or a preflight error in P3. Precedence: path > item > marker. A note binds to the path directly
before it; in the real plans no multi-path item has a note meant for the whole item.

Handover P2a→P2b open item 2 proposed mapping the "(full replacement)" note at item level. In
the real plan that note follows the first of six paths in one item; at item level the five new
files would become Rewrite entries and their create leads `manifest-write-mode` errors. The
item-level mapping was dropped for this rule.

## D45 Bare file names without siblings

Amends D32. With `siblings` off, a token without `/` that follows a token with a directory in
the same item (or clause) gets `PathOrigin::Bare`: its path stays as written, and its location
is unknown.

- `file_name_from_manifest` and `infer_path` never resolve to a Bare entry. For `infer_path`, a
  Bare entry still counts as one of the task's editable entries: dropping it would turn an
  ambiguous task into a confident guess.
- A Create-mode write (op `create`, or `write` resolved to Create) whose path equals a Bare
  entry is `Manual(AmbiguousPath)`, not a write. A Modify or Rewrite on a Bare entry stays an
  action: P3 preflight catches a wrong target.
- D33: a Bare entry is not literal, so the commit suggestion shows the message only.
- `show` marks Bare entries. `AmbiguousPath` is produced only by this rule; it is not a
  `manual-prose` reason.

## D46 Step labels that stay in their step

`[[outline.steps]] keep = true`: the boundary block is the first block of its step instead of
being consumed by the outline. Hand-written plans put the step's lead in its label
(``**2a.** `p`, edit 1 (old → new):``, ``**Verify:** `cmd` → …``, ``**Commit:** `msg` ``), and
recognizers see it only when it stays. A kept label no recognizer takes is a `Note`. With
`keep`, `first_block` cannot match in that step (see D28): only `leads` decide which fence after
a label runs.

## D47 `Expected:` with a list

`expected-prose` with `list = true`: a lead paragraph (``^Expected(?: \([^)]*\))?:$``) and the
list right after it are one prose expectation (D11: always confirmed). The list is the list
items at the lead's depth and, nested in them, paragraphs, list items and `text` fences; any
other block (a shell fence above all) ends it and goes through recognition as usual. A step
boundary ends it too. The exit table reads the whole text. A lead with no list after it is
`missing-block`. After a Manual, the lead and its list join the Manual (D38). The lead
`Expected and recorded:` is not a list lead: its items are instructions, not expectations.

## D48 A sentence after comment expectations extends them

Supersedes D35 in part. An `expected-prose` (plain or `list = true`) directly after a script
whose only expectation is its comment lines (`line_expect`) extends that expectation instead of
being `expected-twice`. The exit status comes from the sentence's exit table; the comment lines
and the sentence stay two labelled parts (`comments`, `prose`), so the confirmation (D11) shows
which is which. An `expected-block` there is still `expected-twice`, and so is a second
`Expected:` after one script. The body always runs unchanged: a comment that is a remark, or a
`#` inside a string, only adds text to what is shown for confirmation.

## D49 Numbered edit series

`before-after` parameter `series`: a regex with an `n` capture that reads the edit number of a
lead paragraph. A lead without a path whose own `n` capture is at least 2 continues the series
when the action directly before it in the same step is a ReplaceExact whose lead paragraph has
number `n - 1`: it edits that path, with `PathOrigin::PreviousEdit`.

- Any action in between (a Note included) breaks the series. A chain is fine: edit 3 continues
  the edit 2 that got `PreviousEdit`.
- A continuation is tried before `infer_path`, and only on leads without a path. D32's
  `infer_path` rule is unchanged.
- The spec load refuses `series` on a lead that does not capture `n`.
- The previous edit's number is read from its lead paragraph by `series`, because a
  recognizer cannot see another recognizer's captures (recognizers are pure and independent,
  extending.md §5); the edit with the path is usually matched by another recognizer instance.

Amended by D59 (a leading word names the file).

Decisions D50 onwards were agreed on 2026-11-25 at the start of P3, from the review of the
execution design (workspace, git, dry run) against the IR that P2 left.

## D50 Unscoped file actions are confirmed

A file action (WriteFile, ApplyPatch, ReplaceExact, Delete) is scoped when its task has a
manifest and either a file entry equal to its path whose origin is not Bare, or a pattern entry
that matches it (D41). Any other file action is unscoped: in a task without a manifest
(`required = false`), at a path no entry covers (`manifest-undeclared` downgraded, D31), or at
a path only a Bare entry names (D45).

- An unscoped file action never runs automatically: the runner shows its op, path, PathOrigin
  and plan line and asks, whatever `--auto` says. Preflight checks it like any other action;
  the dry run reports it as "would ask". P3 computes scope; P5 adds the prompt.
- An action is Bare-targeted when some Bare file entry of its task equals its path. A
  Bare-targeted WriteFile Overwrite or Delete is `path-ambiguous`, a preflight error in every
  format: nothing in the action confirms the target, where an ApplyPatch has `pre` and a
  ReplaceExact has "`before` exactly once". A Bare-targeted ApplyPatch or ReplaceExact stays an
  action; it is unscoped, and a missing target is `target-missing` with the D45 hint. A path
  with both a Written and a Bare entry is scoped and Bare-targeted at once: conservative, and
  loud.
- Paths resolved by D32 and D49 (FileName, OnlyEntry, PreviousEdit) are scoped: each resolves
  only to an entry written in full.
- Scope and Bare-targeting are `Manifest` methods in planrun-model, used by parse (D45's
  `Manual(AmbiguousPath)`) and exec alike. Scope is computed from the IR, never read from a
  spec: no format spec, `[checks]` entry or `--auto` value switches D50 off (as D31).

Actions never carry `PathOrigin::Bare`: the manifest sets it on entries only, and a lead that
names a bare file name resolves to `Written`. So the rule is keyed on the manifest, not on the
action's origin.

## D51 RepoPath refuses Windows-reserved spellings

Amends D7 and D42. Besides absolute paths, `..`, `.`, empty segments, backslashes and
wildcards, `RepoPath` refuses, as `bad-path` in `check`:

- `:` anywhere (an NTFS alternate data stream: `a.rs:x`), `<`, `>`, `"`, `|`, and the control
  characters 0x00–0x1F;
- a segment that equals `.git` after ASCII case-folding and stripping trailing dots and spaces
  (`.GIT`, `.git.`, `.git `), and its 8.3 alias `git~1` (CVE-2014-9390): a plan never writes
  into a git directory, hooks above all;
- a Windows device name as a segment, with or without an extension, in any case: `con`, `prn`,
  `aux`, `nul`, `com0`–`com9`, `lpt0`–`lpt9`, the superscript forms `com¹`–`com³` and
  `lpt¹`–`lpt³`, `conin$`, `conout$`;
- a segment that ends in a dot or a space.

`.gitattributes`, `.gitignore` and `.gitmodules` stay writable.

## D52 The virtual tree, pre-images and the writer

Amends D7 and D23; corrects architecture.md §6.2 and §7.

- **Virtual tree.** A temporary index file and object directory in a fresh directory under the
  OS temp directory, with the repository's objects as alternates. Its git calls get
  `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY` and `GIT_ALTERNATE_OBJECT_DIRECTORIES` on their own
  `Command`; no other git call inherits them, and nothing in the repository is written. A path
  is seeded from the working tree on first use (`hash-object -w --path`). The tree tells apart
  a path not seeded yet, one absent on disk, one deleted in this run, and one whose state a
  script that did not run may have changed (dry run, D55). Patches are applied to it with
  `git apply --cached` (D20 holds: git applies them), so preimage and postimage ids are
  compared in git's normalized space, with every filter and `autocrlf` applied by git.
- **Path identity.** With `core.ignorecase=true`, paths are compared case-insensitively in the
  virtual tree and within a step: two spellings of one path in a run are `path-collision`. A
  plan path whose existing segments differ in case from the disk is `path-case`.
- **Pre-images** of every path a step changes are stored in the repository's object database
  with `git hash-object -w --no-filters` (the exact bytes, CRLF and untracked files included)
  before anything is written. This id differs from the filtered blob id the checks use.
- **The writer.** Every write goes through one writer, which resolves the path first: the
  deepest existing ancestor, canonicalized, must equal the repository root plus the same
  segments, compared exactly. That refuses symlinks, junctions, 8.3 short names and case
  mismatches. Directories a write needs are created one segment at a time, each checked. The
  contents go to `.planrun-<random>.tmp`, created with `create_new` in the target's directory;
  the parent is checked again directly before the rename, and the rename replaces the target
  (a symlink placed there is replaced, never written through). The temp file is the one thing
  besides the plan's own writes that appears in the working tree, and only until the rename.
- **What is not closed:** a parent directory swapped for a link between the second check and
  the rename. planrun is no defence against a process racing it in the same tree.
- Status calls run as `git --no-optional-locks status --porcelain -z`, so even they leave
  `.git/index` alone.

## D53 A step transaction is prepare, then commit

Amends D23.

- **prepare** validates a step's file actions on a virtual tree seeded from the disk, stores
  the pre-images, and builds every post-image there: patches with `git apply --cached`, the
  other actions as D54 bytes (`hash-object -w --path`). It returns a `PreparedStep`: per path
  the pre-image id (or "absent"), the post-image bytes (or "delete"), and the directories it
  will create. The bytes of a patched file are `git cat-file --filters --path=<p> <blob>`, the
  same conversion to the working tree that `git apply` uses.
- **commit** writes the prepared post-images with the one writer (D52), in order. It first
  hands the `PreparedStep` to a write-ahead recorder; it cannot write without one. P5's recorder
  is the journal; a real run without that record is not allowed. P3's CLI has only `--dry-run`,
  so nothing in P3 commits outside the tests.
- **Rollback:** an error during commit restores every path already written from its pre-image
  (exact bytes), removes files it created and the directories it created (deepest first, when
  empty), and reports any path it could not restore.
- **The index:** a `Delete` with `via_git` also removes the path from the index, as one
  `git update-index --force-remove` call after every working-tree write succeeded. It is the
  last effect of the step and writes the index once, under git's lock: it either happens whole
  or not at all, and when it fails the working-tree writes are rolled back.
- `ActionHandler` gets `simulate` (the dry-run behaviour of extending.md §4) and no per-kind
  write: writing is the transaction's job.

## D54 Line endings of written files, refined

Amends D10. For an Overwrite and a ReplaceExact target, in this order:

1. An `eol` attribute (`lf` or `crlf`) decides. `-text` (or `binary`) means the bytes are used
   unchanged: an Overwrite writes the plan's bytes, a ReplaceExact matches raw bytes.
2. Otherwise the file decides: CRLF when it has at least one CRLF and no bare LF, LF when it
   has at least one LF and no CRLF.
3. A file with no line ending at all (empty, or one line): git's checkout convention for that
   path (`core.eol`, `core.autocrlf`, attributes), found by converting a one-line blob with
   `git cat-file --filters --path=<p>`. That is what a fresh checkout would produce.
4. Mixed: an Overwrite uses the checkout convention (the whole file is replaced); a ReplaceExact
   is `eol-mixed`, a preflight error, because a one-line edit must not rewrite every line
   ending in the file.

`before` is matched in LF space (D10). Created files stay LF. Attributes are read once, at the
start of a step; a step that writes `.gitattributes` changes the attributes of later steps
only, and the report says so.

## D55 The dry run

- It parses, checks, and preflights against one virtual tree carried across every task of the
  run, so a task sees the files the tasks before it wrote. Commits are not run; the dry run
  assumes each task's commit happens.
- Real dirt is checked once, at the start, for the paths every task of the run touches: a dirty
  or untracked touched path is `path-dirty` (an error); other dirt is a warning.
- Scripts are not run, and neither are Manual steps (a prose edit changes a file by hand). Any
  of them can touch any path, so the rule is plain: once a script, commit or Manual that did not
  run comes earlier in the run, a failing file action is a warning, "depends on a step that did
  not run", not a failure, and its path is unknown in the virtual tree from then on; later
  actions on it get the same label. Only failures of the tree's state are downgraded (a missing
  or existing target, a blob or patch mismatch, `before` not unique, `eol-mixed`); a bad path,
  `path-ambiguous` or `write-mode-unknown` stays an error. Every ok after that point assumes
  those steps change no files (`cargo fmt` does). The report says so in its header, and its
  summary counts these warnings. A failure before any of them is a failure. Exit 0 when only
  warnings remain.
- `Generated` is "not checked" (its script did not run), a `Manual` "waits", an unscoped file
  action "would ask" (D50).

## D56 No in-process blob ids in planrun-exec

Amends D9; corrects architecture.md §6.2. planrun-exec has no SHA-1 fast path: knowing that no
filter applies costs about as much as one `git hash-object`, and D25 rules out speculative speed
work. `sha1_smol` stays for planrun-parse's `blob-chain` check only.

## D57 Pre-image objects can be pruned

The pre-images D52 stores are unreachable objects: `git gc` may prune them once they are older
than `gc.pruneExpire` (two weeks by default). P5 decides whether a resume past that window is
detected (the object is gone: the user decides, D14) or whether the objects are pinned (for
example by a ref under `refs/planrun/`). Recorded now, not solved in P3.

Settled by D77: detection, no pin.

## D58 A write without a mode is an error

A `WriteFile` whose mode is still `FromManifest` at a path no manifest entry declares (a task
without a manifest, or an undeclared path where `manifest-undeclared` is only a warning) is
`write-mode-unknown`, a check error in every format. No built-in format produces one: assay
requires a manifest and keeps `manifest-undeclared` an error, and the other formats never use
the `write` op. Where `manifest-undeclared` is already an error on the action, it is reported
once. planrun-exec refuses a `FromManifest` write too, as an error that never prompts.

## D59 An edit series' leading word names its file

Amends D49. A `before-after` lead without a path may capture the word before "edit" as `stem`
(`README edit 2 (old → new):`). The path the lead continues (D49) or infers (D32) must then have
that file-name stem, the file name without its last extension (`README` of `README.md`),
compared exactly; otherwise the series does not continue, and with no other agreeing path the
edit is `Manual(ProseEdit)`. The spec load refuses `stem` on a lead that captures `path`.

Decisions D60 onwards were agreed on 2026-11-25 at the start of P4, from the review of the
shell, expectation and policy design against the IR and the six local plans.

## D60 Shell sessions and their protocol

Amends D24; corrects architecture.md §6.3.

- **One process per session**, started at the repository root with `-NoProfile`
  (`--noprofile --norc`), detached from planrun's terminal (D64). Each block is written to
  `b<n>.tmp` with `create_new` in a run directory and renamed to `b<n>.ps1` or `b<n>.sh`, so
  nothing reads half a block. The run directory is `planrun-shell-<random>` in the OS temp
  directory, created with create-dir semantics (it must not exist), mode 0700 on Unix, and
  removed when the session is dropped. Its path reaches the session in the environment
  variable `PLANRUN_SESSION_DIR`, which the prologue copies and removes: no path ever travels
  over a pipe, so a non-ASCII temp directory works. Block bodies are written unchanged (D48).
- **Blocks get no stdin.** A native command started by pwsh inherits pwsh's own stdin: with
  the protocol on stdin, `git hash-object --stdin` in a block read the protocol and hung. So
  pwsh gets a null stdin, and its `-Command` script is the prologue and a loop at the top level
  that waits for the next block file (a `FileSystemWatcher`, re-checked every 250 ms) and
  dot-sources it inside `do { try { … } catch { … } } while ($false)`; the `do` keeps a block's
  `break` or `continue` inside it. bash keeps its blocks on stdin, one top-level line each:
  `source "<dir>/b<n>.sh" </dev/null`, so a `cat` in a block reads nothing. No function wraps
  `source`: inside a function a block's `declare` would make locals, `$1` would be the temp
  path, and `local` would succeed where the terminal refuses it.
- **The sentinel.** A 128-bit secret per session plus the block's number:
  `__planrun_<secret>_<n>__`, printed on stdout with four fields (`ok`|`threw`|`exit`, the
  status, `errored` as 0 or 1, the working directory as hex of its UTF-8 bytes) and alone on
  stderr. A block ends when both have arrived, so late stderr never leaks into the next block.
  Text before the sentinel on its line is the block's last output line, marked as having no
  final newline. A sentinel for another block, or anything after the defined fields, is
  `Broken`: the session is dead. Block 0 is the prologue's own sentinel; a prologue that prints
  anything or ends otherwise fails the start.
- **Wrapper names** are `$global:__planrun_*` in pwsh, reset before every block, and
  `__planrun_*` in bash; a plan's `$t`, `$s` or `n` is never touched.
- **Output** is kept per line with its stream and a final-newline flag. Order within a stream
  is exact; order across the two streams is arrival order, best effort.
- **The working directory** after each block comes with the sentinel (pwsh
  `$PWD.ProviderPath`; bash `$PWD`, on Windows `pwd -W`, because there `$PWD` is an MSYS path
  and `/tmp` is a mount). P5 can warn when it leaves the repository; the deferred candidate
  (D70) needs it too.
- **pwsh prologue:** `$PSStyle.OutputRendering = 'PlainText'`, `$ProgressPreference =
  'SilentlyContinue'`, `$ErrorActionPreference = 'Continue'`,
  `$PSNativeCommandUseErrorActionPreference = $false`, UTF-8 without BOM for
  `[Console]::OutputEncoding` and `$OutputEncoding`; on Windows `-ExecutionPolicy Bypass` for
  the session process, because a dot-sourced file is a script to the execution policy where a
  line typed in the terminal is not. A block that changes any of these changes later blocks,
  as in the terminal.

Stated differences from Rens' terminal: `$LASTEXITCODE` is reset to 0 before each block (in
the terminal it carries over between commands); a terminating error ends the block (`try`); in
pwsh `$PSScriptRoot` and `$MyInvocation` name the block's temp file; pwsh `exit n` ends only the
block (D61); `GIT_TERMINAL_PROMPT=0` (D62).

Amended by D71 (`builtin` for the wrapper's commands, RemoteSigned).

## D61 How a block ends

`RunStatus` is `Exited(code)`, `Threw` (pwsh), `SessionEnded(Option<code>)`, `TimedOut` or
`Broken(reason)`.

- **pwsh:** `Threw` when a terminating error reached the wrapper's `catch` (a parse error of the
  block included); otherwise `$LASTEXITCODE`, which is 0 when no native command ran. `exit n`
  in a dot-sourced file ends only the block: `Exited(n)`, and the session's variables survive.
- **bash:** `$?` of the `source`. A syntax error in a block ends the block with `Exited(2)`; the
  session survives. `exit n` ends the session: the EXIT trap reports `SessionEnded(Some(n))`, and
  so does `set -e` followed by a failing command. EOF on both streams without a sentinel (a
  block replaced the trap, or `exec`ed) is `SessionEnded(None)`. Nothing restarts a session
  silently; a dead session refuses the next block, and P5 stops.
- **`errored`** (pwsh only; always false in bash): `$Error[0]` is not the same object before
  and after the block (`[object]::ReferenceEquals`). A `$Error.Count` delta stays at 256 once
  `$Error` is full. It never changes the status. Measured in pwsh 7.6:

  | Construct | `errored` |
  |---|---|
  | `-ErrorAction SilentlyContinue` | true |
  | `Write-Error`, a cmdlet error | true |
  | `-ErrorAction Ignore` | false |
  | native `2>$null`, native `2>&1` | false |
  | native stderr, not redirected | false |

  In the six local plans at most 6 of 222 pwsh fences use a silencing probe, all
  `SilentlyContinue`: accepted as noise.

Amended by D71 (the shell gone while its pipes are held) and D81 (a dead session in a run).

## D62 The session environment

A session gets planrun's own process environment minus `BASH_ENV` and `ENV` (bash reads
`BASH_ENV` even with `--norc`) and minus `GIT_LITERAL_PATHSPECS`, `GIT_INDEX_FILE`,
`GIT_OBJECT_DIRECTORY` and `GIT_ALTERNATE_OBJECT_DIRECTORIES`: the git wrapper's and the virtual
tree's variables never reach a plan's commands (a literal pathspec would change what `git add
*.rs` means). It gets `GIT_TERMINAL_PROMPT=0`, so git fails instead of prompting: a stated
difference from the terminal. The tests add the fixture's git isolation (`Fixture::env()`), and
a block's `git config --show-origin --list` shows only the copy's own `.git/config`.

## D63 Which shells run

- **pwsh** is `pwsh` from `PATH`, accepted when a probe reports
  `$PSVersionTable.PSVersion.Major` of 7 or more. Windows PowerShell 5.1 (`powershell.exe`) is
  never used.
- **bash on Windows** is Git for Windows' own: `<root>\usr\bin\bash.exe`, where `<root>` is the
  first of `git --exec-path` and its parents, at most four levels up, that has one (no layout
  below the root is assumed: `mingw64`, `clangarm64`). A probe runs `PATH=/usr/bin:$PATH
  uname -s`, which must report `MINGW…` or `MSYS…`. bash is never taken from `PATH`, where
  WSL's `System32\bash.exe` can come first, and a failed probe never falls back to it.
  `bin\bash.exe` (the launcher) is not used: it costs a process and sets its own environment.
- **`/usr/bin` is prepended** to the session's `PATH` on Windows, as `/etc/profile` would: a
  `--noprofile` Git Bash without Git's `usr\bin` on the Windows `PATH` (the default install)
  finds no `uname`, `cat`, `ls` or `grep`. Only `/usr/bin`, not `/mingw64/bin`: that directory
  depends on the install's layout, and without it `git` in a bash block resolves through the
  Windows `PATH`, the same git planrun itself uses.
- **bash elsewhere** is `bash` from `PATH`, checked with `$BASH_VERSION`.
- **Tests** need both shells when `PLANRUN_REQUIRE_SHELLS` is set; otherwise a missing shell
  skips its tests with a message on stderr (a skipped test counts as passed).

## D64 The process tree, and planrun's only unsafe code

Amends D9 (no crate).

- **Windows:** a job object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, read back with
  `QueryInformationJobObject` and refused when the flag did not stick. The session is assigned
  right after its spawn and before planrun writes anything to it; a shell starts no child before
  it has a block, so no descendant escapes between the two. When assignment fails (planrun runs
  inside a job that refuses nesting), the session fails to start: there is no session without
  the tree kill. `TerminateJobObject` kills the tree on timeout, and closing the job on drop
  kills what is left, also when planrun crashes. The session is spawned with
  `CREATE_NO_WINDOW`, so it has a console of its own and its `OutputEncoding` cannot change
  the code page of Rens' terminal (tested with `chcp`). Not `taskkill`: it misses descendants
  whose parent has exited, does nothing when planrun crashes, and is found through `PATH`.
- **Unix:** `setsid` in `pre_exec`, so a block that reads `/dev/tty` fails instead of stopping
  a background process group forever; `kill(-pgid, SIGKILL)`, and only while the session
  process has not been waited on, so the group id cannot have been reused.
- **Drop** kills the tree and never waits for the reader threads: a build server (MSBuild
  nodes, VBCSCompiler) inherits the pipes and keeps them open.
- **`unsafe`.** The workspace lint is `unsafe_code = "deny"`; planrun-model, planrun-parse and
  planrun-cli say `#![forbid(unsafe_code)]` at their roots, so the relaxation reaches
  planrun-exec only, and there only `crates/planrun-exec/src/shell/kill.rs` says
  `#![allow(unsafe_code)]`. That module declares the five kernel32 functions and `setsid`/`kill`
  by hand, owns every `unsafe` call, and exposes safe operations (`Tree::spawn`, `Tree::kill`,
  `Tree::wait`, `detach`). `clippy::undocumented_unsafe_blocks` is denied: every unsafe block
  has a SAFETY comment. The hand-written `JOBOBJECT_EXTENDED_LIMIT_INFORMATION` is tested to be
  144 bytes on 64-bit targets, and the read-back flag is tested, so a wrong layout fails a test.
  `windows-sys` would bring hundreds of generated declarations for five functions.

Amended by D71 (PDEATHSIG, `waitid`) and D73 (the interrupt handlers).

## D65 No default timeout yet

A block may run as long as it runs: cargo, dotnet and npm runs are long, and Rens' terminal has
no timeout. `SessionConfig::timeout` exists; the tests use 60 seconds (5 for the timeout test,
which checks that a grandchild dies with the tree, on both platforms). Whether the runner has a
default is a P5 decision.

Amended by D73: still no default timeout.

## D66 Matching a run against its expectation

Amends D11; corrects architecture.md §6.4.

- **Exit:** `Success` is `Exited(0)`; `Failure` is a non-zero `Exited` or `Threw`; `Code(n)` is
  exactly `Exited(n)`; `Unknown` is not checked. `SessionEnded`, `TimedOut` and `Broken` are a
  mismatch whatever was expected, also for a script with no expectation.
- **Summary lines:** each non-blank expected line must match an output line (either stream)
  after the one the line before it matched; the earliest match is taken, which finds a
  subsequence whenever one exists. Only `…` is a wildcard (ASCII `...` is three dots; none of
  the 26 expected text fences of the local plans has one): it matches any run of characters,
  the empty one included; the first literal segment is anchored at the start of the line and
  the last at its end. Normalization, and nothing else: output is split on LF; trailing
  whitespace is stripped from expected and output lines (which removes a CR); ANSI CSI and OSC
  sequences are stripped from output lines. Leading whitespace counts.
- **What asks:** prose (D11) and comment expectations (D35, D48) always ask, labelled apart; an
  exit the plan does not state (no expectation, or `Unknown`) asks when it is not 0 or the block
  threw; `errored` asks when nothing but an exit `Success` or `Unknown` was expected (a
  `Copy-Item` that failed without terminating exits 0). Every confirmation shows `errored`.
  Otherwise the run passes.
- The verdict is `Pass`, `Mismatch` (the exit, and each unmatched line with its line number) or
  `Confirm` (the reasons and `errored`). Skipping is not a verdict: it is decided before the run
  (D67).

## D67 Only Verify scripts are skipped as expected failures

Amends D12. A `RunScript` whose intent is Verify and whose exit expectation is `Failure` is
skipped unless `--run-expected-failures` is given. A Gate, Preview or Apply that expects failure
runs (a skipped Gate would leave `$gate` unset for the commit); a Commit is never skipped;
`Code(n)` is never an expected failure. A skip counts as "skipped by policy" for D22.

## D68 The command classifier is a built-in allowlist

Amends D13; corrects architecture.md §6.5. In the spirit of D31: the classes are built in, no
format spec, `[checks]` entry or flag changes a class, and a runner has one classifier, not one
per format. The tables are Rust constants in `crates/planrun-exec/src/policy/heads.rs`; no
`toml` in planrun-exec.

- **The splitter** (one per shell) walks a script once: statements (newline, `;`, `&&`, `||`),
  pipeline elements (`|`), comments (`#`, pwsh `<# #>`), line continuations; it recurses into
  `( … )`, `$( … )`, `@( … )`, `@{ … }` (hashtable values), script blocks, `$( … )` inside
  double quotes and `@" … "@`, bash backticks, `<( … )`/`>( … )` and unquoted heredoc bodies.
  Single quotes, `@' … '@` and quoted heredoc bodies are data; the heredoc's own command is
  still classified. Keywords are not commands (pwsh `if`, `foreach`, `function`, … at the start
  of a statement; bash `if`, `then`, `do`, `!`, `{`, … before a command); an expression
  statement (`"…"`, `$x`, a number) is Read; an assignment classifies its right side; a function
  body is classified where it is defined, and calling the function by name is unknown.
- **What it cannot take apart with confidence makes the whole script Destructive:** an
  unterminated quote, here-string or heredoc, an unmatched bracket, pwsh `<`, bash `case` and
  `;;`, a command inside `${…}`.
- **pwsh lexing:** `–`, `—` and `―` are parameter dashes; `‘ ’ ‚ ‛` single and `“ ” „` double
  quotes; names, aliases and parameters are case-insensitive, aliases (`rm`, `ri`, `del`, `cp`,
  `mv`, `cat`, `ls`, `cd`, `iex`, `saps`, `iwr`, `irm`, `%`, `?`, …) map to their cmdlet first;
  a `.exe` or `.cmd` suffix is dropped; a path- or module-qualified command is Destructive.
  Parameters match by prefix as pwsh does (`-r`, `-Rec`, `-Recurse:$true`), and splatting
  (`@args`) on a command with a dangerous switch is Destructive. `& name` is classified as
  `name` when `name` is a literal bare command name; `&` with anything else, and dot-sourcing a
  file, are Destructive. A static .NET call is Destructive except on `[IO.Path]`, `[Math]`,
  `[string]` and `[regex]`; an instance call except `Trim`, `Split`, `Replace`, `Substring`,
  `ToString`, `Contains`, `StartsWith`, `EndsWith`, `ToLower`, `ToUpper`. `Add-Type` and
  `New-Object` are unknown.
- **Redirections:** `$null`, `/dev/null` and `NUL` are nothing; a literal relative path without
  `..` is Modify; anything else (absolute, `~`, a variable, a subexpression) is Destructive.
  Operators: pwsh `> >> 2> *> n>` (and the merges `n>&1`), bash `> >> >| &> <> >&file`;
  `Tee-Object -FilePath` is Modify.
- **Nested shells:** `pwsh -File`, `pwsh -c`, `bash <file>`, `bash -c`, `cmd /c`, `source`,
  `eval`, `exec` are unknown, so Destructive; a nested shell's string is never parsed. An
  environment prefix on a bash command (`FOO=1 git status`) is Destructive.
- **git:** options before the subcommand are parsed: `-C <dir>` keeps the subcommand's class;
  `--git-dir`, `--work-tree`, `--exec-path` and any other unknown option are Destructive; `-c
  k=v` is allowed only for `color.*`, `core.quotepath`, `user.name`, `user.email` and
  `advice.*` (aliases, `core.pager`, `core.fsmonitor`, `diff.external`, `core.sshCommand` run
  programs). Unknown subcommands, aliases included, are Destructive. `checkout -b/-B/--orphan`
  and `switch` without `-f`/`--discard-changes` are Modify; `checkout <name>` without `-b` is
  Destructive (the name can be a path), and so are `checkout --`, `-f`, `switch -f`,
  `--discard-changes`, `restore`, `reset --hard/--merge/--keep`, `clean`, `push`, `fetch`,
  `pull`, `stash drop/clear`, `branch -D/-f`, `config --global/--system/--file`. `diff`, `log`
  and `show` with `--output` are Modify, with `--ext-diff` Destructive.
- **cargo, npm, dotnet, go:** options before the subcommand are parsed (`cargo +nightly`,
  `--manifest-path x`); `cargo --config` is Destructive. Build may fetch dependencies into
  caches and ignored directories, so `npm ci` and `dotnet restore` are Build; Modify changes
  tracked files, manifests or lockfiles (`cargo fmt` without `--check`, `fix`, `clippy --fix`,
  `add`, `update`, `remove`, `npm install`/`uninstall`/`update`, `dotnet add`, `dotnet format`
  without `--verify-no-changes`, `go mod tidy`, `go get`); `cargo install`/`publish`/`run`, `go
  install`/`run`, `dotnet run` and `npx` are Destructive. A cargo alias (`cargo xtask`) is an
  unknown subcommand.
- **DevReady** (pwsh table only; the list is from DevReady 1.19.1, `DEVREADY_VERSION`):
  functions whose agent mode is `safe` are Read; `brief { … }` is classified by its block;
  `replace -DryRun` is Read, `replace -All` Modify, `replace` with neither Destructive (it stops
  to ask); `test` and `testfailed` are Build (they run the tests); `killport` is Destructive
  although DevReady calls it safe (it kills a process); `dry-run-only` and `interactive`
  functions are Destructive.
- The local plans' scripts are classified by an ignored test in planrun-cli
  (`local_classes`), whose output stays in the terminal (D16).

Amended by D72 (sed; xtask and repository scripts stay Destructive).

## D69 `--auto`, and what P4 decides for P5

Amends D13. `--auto` takes `read`, `build` and `modify`; `destructive` is a usage error. Without
`--auto` every script asks, Read ones included. A Destructive script always needs a typed `yes`,
a Manual always waits, a file action outside the manifest's scope always asks (D50), and an
Apply script always asks (it follows its preview), whatever `--auto` says; file actions within
scope run automatically. P4 computes the decision (`Policy::decide`: `Auto`, `Ask(class)`,
`AskUnscoped`, `TypedYes`, `Wait`, `Show`, `Skip`); P5's runner prompts. The dry run shows each
script's class and decision under the default policy ("would ask (Build)", "typed yes
(Destructive)", "skipped: expected failure").

## D70 A script working directory from a run lead is deferred

The P2b candidate (``(run from `$REPO`):`` gives a script its directory) is not built in P4.
The name in the lead is a session value that planrun cannot resolve statically; evaluating it
would run prose as code; and the directory can be outside the repository. (A
`Push-Location`/`Pop-Location` around the block would keep D24 intact, so D24 is no reason.)
Such fences stay `Manual(HumanCheck)` (`manual-prose#run-from`). What it would take later:
`Script.workdir` with `skip_serializing_if`, so no IR snapshot changes; the run-from lead
becoming a script lead; and the directory resolved from the session (D60 reports the working
directory after every block) and checked against the repository root before the block runs.

## D71 Sessions that cannot hang or drift

Amends D60, D61 and D64.

- **The shell gone, its pipes held.** `Session::collect` waits for events in slices of 100 ms.
  Between slices it asks whether the shell itself has exited, without reaping it: on Linux
  `waitid(P_PID, pid, WEXITED | WNOHANG | WNOWAIT)` (std's `try_wait` would reap the shell, and
  D64 allows `kill(-pgid)` only before the reap, when the group id cannot have been reused); on
  Windows a zero-timeout `WaitForSingleObject` on the process handle the tree keeps open. When
  it has, the tree is killed so that a descendant holding stdout or stderr (a build server, a
  background job) lets go, and the block ends as `SessionEnded(None)`. Other Unix systems answer
  "not exited" and keep the P4 behaviour (D19).
- **A block that cannot be written ends the session.** Any I/O error while `ShellSession::run`
  writes or renames `b<n>` kills the session: the shell would wait for that file forever, and
  the next block would be numbered past it.
- **Every command the bash wrapper runs goes through `builtin`:** `source`, `local`, `printf`,
  `pwd`. A plan function of any of those names changes nothing. The prologue's own `unset` and
  `trap` run before any plan code. Not defended: a plan function named `builtin`, or one named
  like the wrapper's `__planrun_*` functions.
- **The shell dies with planrun on Linux.** The `pre_exec` of every session sets
  `PR_SET_PDEATHSIG` to SIGKILL after `setsid`, then compares `getppid()` with planrun's process
  id read before the fork and refuses to start when they differ (the parent died in between).
  The signal follows the thread that spawned the child, so sessions are spawned from planrun's
  main thread. It reaches the shell only, not its descendants (D73). Other Unix systems are out
  of scope (D19). `prctl`, `getppid` and `waitid` are declared by hand in `shell/kill.rs`, with
  SAFETY comments; `siginfo_t` is read only for its first field and its size is tested.
- **`-ExecutionPolicy RemoteSigned`** replaces `Bypass` for the pwsh session on Windows: the
  block files are local and run; a downloaded script a block calls keeps the Mark-of-the-Web
  check of Rens' terminal. A `-ExecutionPolicy` argument sets the Process scope, which outranks
  the user's CurrentUser and LocalMachine settings: a terminal set to Bypass or Unrestricted is
  less strict than the session. A group policy (MachinePolicy, UserPolicy) still wins over both.

## D72 sed is read; xtask and repository scripts stay Destructive

Amends D68.

- **sed** without `-i` is Read only when its script neither writes nor runs anything. The
  script (`-e`, `--expression`, or the first operand) is read command by command with GNU's
  syntax: addresses, `!`, `{ }`, `s` with any delimiter and its flags, `y`, the text of `a`,
  `i`, `c` (continued lines included), labels, `r`/`R`. `w`, `W` and the `s///w` flag are
  Modify; `e` and the `s///e` flag are Destructive; so is `-f`/`--file` (a script the
  classifier never sees), a script with an expansion, an unknown option, and anything the
  reader does not recognise with confidence. `-i` stays Modify.
- **`cargo xtask …` and `pwsh -File <relative path>`** stay Destructive. `cargo test` is Build
  because a test runner is what it is; `xtask` is a general task runner, and a repository script
  can do anything, deploys and releases included. Under `--auto modify` either would run
  unasked. D13 holds: a command whose effect the tables do not know is Destructive. In the six
  local plans these two account for 49 of the Destructive reasons; the question goes to the P6
  handover with the dogfooding data.

## D73 Interrupts and timeouts

Amends D64 and D65; D9 rejects `ctrlc`.

- **No default timeout** (D65 holds). A hung block is ended by the user: Ctrl-C ends planrun,
  and `resume` goes on from the journal.
- **Windows:** no handler on the normal path. Ctrl-C ends planrun and the job objects
  (`KILL_ON_JOB_CLOSE`) kill every session's tree.
- **Unix:** `setsid` keeps SIGINT and SIGHUP from the sessions, and PDEATHSIG (D71) kills only
  the shell, so a running build would keep writing after planrun died. planrun installs a
  handler for SIGINT, SIGTERM and SIGHUP that does only async-signal-safe things: `kill(-pgid,
  SIGKILL)` for every registered session group (a static array of atomics; a slot is cleared
  before its leader is reaped, as D64 requires), then restores the default action and raises
  the signal again. It is installed with `signal`, not `sigaction`: `struct sigaction` differs
  between architectures and C libraries, and a hand-declared layout would be the riskiest
  declaration in `kill.rs`; glibc's `signal` keeps the handler installed and restarts
  interrupted calls.
- **While the `[o]` shell runs** planrun ignores Ctrl-C, so it reaches the shell and not
  planrun. Windows: a console control handler routine that returns TRUE for Ctrl-C and
  Ctrl-Break while a flag is set. Not `SetConsoleCtrlHandler(NULL, TRUE)`: that attribute is
  inherited by child processes, so the shell and every command started in it would ignore
  Ctrl-C too. Unix: the same flag makes the handler ignore SIGINT.
- **What an interrupt leaves.** It can land anywhere, inside a step's commit too. Resume
  recognises every state it can leave (D77): a step as before, as planned, or mixed (Unknown);
  a journal whose last line is torn (dropped, D76); a writer's temp file (removed by name,
  D77). A SIGKILL of planrun on Unix leaves a session's descendants running: stated, not
  solved.
- `ctrlc` is rejected (D9): it brings `nix` and `windows-sys`, which D64 already rejected.

Amended by D91 (`shell::end_now`, for a UI that reads Ctrl-C as a key).

## D74 Line prompts on std

Amends D15; D9 drops `dialoguer`.

- **std only.** `dialoguer` refuses a stdin that is not a terminal, and its prompts redraw lines.
  The prompts are plain lines on stdout and a line read from stdin, so cargo's output stays in
  the scrollback. planrun-cli implements `Ui`; planrun-exec's runner decides what is asked.
- **Only a terminal answers.** When stdin is not a terminal (`IsTerminal`), the first question
  stops the run with an explanation, journaled as `run-end` with the reason; there is never a
  default answer. planrun-cli's `e2e` feature lifts this for its own tests (D82).
- **A typed `yes`** is the line with only its trailing `\n` or `\r\n` removed, equal to `yes`
  exactly: case-sensitive, nothing trimmed. `skip` skips a Destructive script and `quit`
  stops; anything else does not run and asks again.
- **The prompt per decision** (D69): `Auto` shows one line and runs; `Ask` shows the script's
  head and offers `[enter] run [s] skip [v] view [d] diff [o] shell [q] quit`; `AskUnscoped`
  shows op, path, PathOrigin and plan line with the same keys; `TypedYes` shows the whole
  script and the classifier's reasons; `Wait` (a Manual) shows its text and waits for
  `[enter]`; `Show` prints a Note; `Skip` prints why. `[v]` shows the step in full, `[d]` runs
  `git --no-optional-locks diff HEAD` and lists untracked files, `[o]` starts an interactive
  shell at the root that is not the plan's session (it says so). An answer the question did
  not offer asks again. Every answer is journaled.
- **Questions of their own**, asked whatever `--auto` says: the session notice (D81), a working
  directory outside the repository (D81), dirt before a commit (D80), a closed commit gate
  (D75), a step changed between prepare and commit (D78), and resume's Unknown step,
  interrupted script and dirt a remaining file action writes over (D77). Each answer is journaled with the question's name.

Amended by D86 (`QuestionKind`).

## D75 The commit gate

Amends D22. The commit script still runs as written, and may gate itself on `$gate`.

- **A question of its own**, before the Commit: when the gate is closed, the user sees every
  reason and opens it only with a typed `yes`, whatever `--auto` says, journaled as
  `gate-override` with the reasons. The Commit's own decision then applies unchanged: P4
  decides, P5 prompts, and P5 never turns a decision into another one.
- **Its inputs** are the last attempt of every script of the task (a passing retry replaces
  the mismatch), not only Verify and Gate, the file actions the user skipped, and the
  `Generated` checks. It is closed by an unresolved mismatch, a confirmation answered no, a user
  skip of any script or file action, an accepted mismatch, a missing `Generated` path, and a
  Gate that ran only in a session that has ended. A skip by policy (D67) keeps it open.
- **A Gate from an ended session runs again** before the commit, under its decision: its
  `$gate` is gone with the session, so its verdict no longer counts.
- **Accept** is offered only for an `Exited` or `Threw` mismatch. For `SessionEnded`, `TimedOut`
  and `Broken` the answers are retry (a new session, D81), shell and abort: accepting a dead
  session would send the commit block into a session without its variables.
- A Gate's confirmation always asks (D11); "no" counts as failed.

## D76 The journal

Amends D18; corrects architecture.md §6.6.

- **One file per plan:** `<git-dir>/planrun/<plan file stem>.jsonl` (`rev-parse
  --absolute-git-dir`, so a linked worktree has its own). Every `run` record stores the plan's
  path (repository-relative, or canonical when the plan is outside the repository) and its
  fingerprint. A journal of that stem whose records name another path is a hard stop that names
  both: two plans never share a file, and planrun never picks another name on its own. The
  fingerprint is not the name: a changed plan must find its journal to be told it changed.
- **Runs are appended.** A `run` record starts a chain; each `resume` appends its own `run`
  record (`resume: true`) to the chain. `run` refuses a plan whose last chain did not complete
  and points to `resume`; `run --restart` gives the unfinished run up by renaming its journal to
  `<stem>.<milliseconds>.jsonl`. Nothing deletes a journal.
- **Records**, each with the schema version and the time: `run`, `task-start`,
  `step-prepared` (the write-ahead record: per path the pre-image id or absent, the post-image's
  exact and filtered ids, the modes, the directories it creates and the writer's temp name),
  `step-written`, `step-failed`, `step-verified`, `step-restored`, `step-accepted`,
  `action-start` (with the decision, `Auto` included, HEAD before it, and for a Commit the
  task's paths as they are on disk), `action-end` (outcome, status, `errored`, the working
  directory, the last 20 output lines, HEAD after it), `answer`, `gate-override`, `head-seen`,
  `session`, `task-end`, `run-end`. No output digest: nothing reads one.
- **Writes:** one `write_all` per record (the JSON and its `\n`), no buffer across records.
  `step-prepared`, every `action-start` and `run-end` are followed by `sync_data`; the rest
  reach the OS only. A killed process loses nothing the OS has; `sync_data` matters only after a
  power cut, and those three are the records whose loss would change what resume does. The
  plan's own file writes are not synced: after a power cut a step is neither as before nor as
  planned, and resume asks (Unknown, D77).
- **Reads:** a line counts only when it ends in `\n`. A last line without one is dropped even
  when it parses, and before the first append the file is cut back to its last `\n`, so the next
  record never joins a fragment. A bad line that is not the last one is an error, never
  skipped. `status` reads the same way and takes no lock.
- **An observer** sees every appended record and every path a step's commit wrote; planrun-cli
  uses it for its fault seam (D82).

Amended by D90 (a journal takes any number of observers).

## D77 Resume, and one planrun per repository

Amends D14 and D57.

- **What is read:** the journal's last chain. It stops, with an explanation, when the plan's
  fingerprint differs from the chain's (both are shown), when `.git/index.lock` exists (planrun
  never removes it), and when HEAD or the ref it points to is not what the journal last recorded
  (HEAD is recorded after every script, Manual, `[o]` and Commit, D80). The one exception is
  the action that was running: when HEAD moved by one commit on the recorded one, a Commit
  whose new tree holds what it was to hold is done (D79); for any other action the user
  decides (keep it as done, or run it again; the question offers no `[o]`, whose `head-seen`
  would pass the check unanswered).
- **The recorded HEAD** is set by the chain's first `run`, `task-start`, `action-end`,
  `head-seen` and `task-end`, never by a resume's own `run` record: that one is written before
  the checks, so a resume that stops leaves the next one the same check. Where resume accepts a
  moved HEAD it journals that at once: an `action-end` for the running action (`passed` for a
  Commit found made, `kept` for the user's keep), or `head-seen` for a retry. A crash after it
  resumes from there; `head-seen` alone would leave a made Commit "running" at an equal HEAD,
  and run it again.
- **The interrupted task runs again from its start**, in a new session, under the journaled
  policy and range unless the flags say otherwise. Before its first script a notice is shown
  that must be acknowledged even with `--auto` (D81).
- **File steps** the journal prepared are checked against the disk, never trusted from it: every
  path as planned (bytes, mode under `core.filemode`, and for a git delete the index entry
  gone) is Done, and the step is not written again (`step-verified`); every path as before is
  NotDone, and the step is prepared and written again; anything else is Unknown: `[r]` restores
  it from its pre-images and writes it again, offered only when every pre-image object is still
  there (`cat-file --batch-check`; `git gc` may have pruned them, D57) and no git delete of the
  step has left the index yet; `[a]` accepts it as done, which overrides the disk check and so
  needs a typed `yes`. The writer's temp files the step named are removed by name, never found
  by pattern.
- **D57 is settled by detection.** Done and NotDone compare hashes and need no object; only
  `[r]` needs the pre-images. No ref pins them. Rollback inside a run is unaffected: its objects
  are minutes old.
- **Scripts:** one that ended runs again under its decision; a Read or Build one is only
  announced (without `--auto` it asks as always), a Modify or Destructive one first asks "ran
  before the interruption: run again or keep". One journaled as running asks the same, whatever
  its class and `--auto`, shown as "was running when interrupted". A kept script keeps its
  result; a kept Gate still runs again before the commit (D75). A Manual acknowledged before is
  not asked again. A Commit that was running when HEAD did not move runs again under its
  decision.
- **Preflight of the resumed task** leaves out the file steps found done, and every path of the
  task may have been changed by the interrupted run (by a step or by a script): dirt in those
  paths is a warning, and their manifest checks are left out. The remaining file actions are
  simulated exactly. Dirt in a path that no step found done or accepted wrote, and that a
  remaining file action writes over, may be the user's (an edit between the crash and the
  resume): it is `resume-dirt`, listed and asked about once before the task's first action,
  whatever `--auto` says. A path that only a script writes is not asked about; a script that
  ran before the interruption asks on its own (above).
- **One planrun per repository:** `<git-dir>/planrun/lock`, held with `File::try_lock` (stable
  since Rust 1.89; the workspace sets no `rust-version`) for the whole of `run` and `resume`,
  prompts included, for every plan of the repository. The operating system releases it when the
  process ends, however it ends: there is no stale lock. The holder's process id goes into
  `lock.pid` for the message only (on Windows another process cannot read a locked file).
  `status` and `--dry-run` take no lock. Its limit: on Unix, after a SIGKILL of planrun, a
  session's descendants can outlive the lock (D73).

Amended by D90 (`lock.live`: a live run is seen without trying the lock).

## D78 Step transactions in a real run

Amends D53. Settles the P3 open items on the gap between prepare and commit, file modes, a git
delete of an untracked file, and the writer's temp files.

- **The order inside commit:** check the disk again, then the write-ahead record (synced,
  D76), then the writes. Each path is stat'ed first (`hash-object --stdin-paths` fails the whole
  batch on a missing file), then the present ones are hashed in one call and compared with the
  pre-images; the directories the step will create must still be absent. On any difference
  nothing is recorded or written, the paths are shown, and the user chooses `[p]` prepare again
  or `[q]` quit. planrun never prepares again silently: something else is writing the tree, and
  an Overwrite prepared again would clobber it. Nothing that prompts lies between the check and
  the writes.
- **Modes:** the post-image's mode is the virtual tree's index entry after prepare (a patch's
  mode lines included), journaled in `step-prepared`, and set on the temp file before the
  rename when `core.filemode` is true (unset: true on Unix). Windows has no executable bit.
  Rollback and restore put the pre-image's mode back; Done compares the mode too.
- **Ids for resume:** `step-prepared` holds each post-image's exact-bytes id (`--no-filters`,
  not stored) beside its filtered blob id, so Done and NotDone compare like with like.
- **Temp names** are chosen at prepare and journaled per path; resume removes exactly those
  (D77).
- **A git delete of an untracked file** (`Delete` with `via_git`) is `delete-untracked`, an
  error of a real run's preflight: the plan's own `git rm` would refuse it. The dry run keeps
  simulating it (P3's stated decision 16).

## D79 When a commit counts as made

A Commit is made only when HEAD moved by exactly one commit on the HEAD before it and that
commit's tree holds the task's paths as they were on disk when the Commit started: its file
actions, `Generated` paths and manifest file entries, journaled in the Commit's `action-start`
as git's blob ids (absent paths absent). This is not the journaled post-images of the task's
steps: a script between a step and the commit may change a written file (`cargo fmt`, D55),
and a `Generated` path has no post-image at all. A commit script that exits 0 without making
that commit (a gated script with `$gate` unset) is a mismatch, not done; resume applies the
same test to a Commit that was running (D77).

## D80 The repository around a commit

- **HEAD and the ref it points to** are recorded after every script, every Manual, every `[o]`
  and every Commit, not only at a task's end: any of them can move HEAD (a commit by hand, D33,
  or a Modify script with `git commit`).
- **Before every Commit**, one `status` over the tree: dirt outside the task's paths (its file
  actions, manifest entries, patterns included, and `Generated` paths) is listed and asked about,
  whatever `--auto` says. A commit script that stages everything would take it.
- **After the Commit**, `git diff-tree -r --name-only` between the HEAD before and after: a
  path outside the task's paths is reported loudly.
- **After a Manual** that moved HEAD (a commit by hand), the task's paths that still have
  changes are reported.

## D81 Sessions within a run

Amends D61.

- **A dead session** (`SessionEnded`, `TimedOut`, `Broken`) is a mismatch whose answers are
  retry, shell and abort (D75). Retry starts a new session; nothing restarts one silently.
- **The session notice:** before the first block of a new session that is not the run's first
  (a restart), of a resume, and of a run with `--from`, planrun says that the session is new and
  that variables, functions and the working directory of earlier blocks are gone, and lists the
  earlier tasks whose scripts ran in another session. It must be acknowledged even with
  `--auto`.
- **A working directory outside the repository** (compared canonically; a pwsh provider path
  that is no file-system path counts as outside) asks before each next block of that session,
  whatever `--auto` says: D68 classes a relative redirection as Modify on the assumption that
  the directory is inside the repository. The block's decision is unchanged.

## D82 The end-to-end tests drive the binary

planrun-cli has a test-only feature, `e2e`. Its end-to-end test target (`tests/run.rs`) has
`required-features = ["e2e"]`, and every plan's gate runs `cargo clippy -p planrun-cli
--all-targets --features e2e -- -D warnings` and `cargo test -p planrun-cli --features e2e
--test run` (with `PLANRUN_REQUIRE_SHELLS=1`). No release build compiles it. Under the feature
only, prompts read answers from a stdin that is not a terminal, and `PLANRUN_E2E_ABORT=<seam>@<task>`
ends the process right after a seam: `step-prepared` (no write yet), `write<k>` (after a step's
k-th write), `commit-start` (after a Commit's `action-start`). It ends with `exit(134)`, not
`abort()`: no destructor runs, as in a kill, and on Windows no Windows Error Reporting is raised
for the test machine to answer. A kill of `git commit` itself (HEAD moved, `action-end` missing)
is tested in planrun-exec, where the observer can make the commit before it crashes.

Amended by D93 (the TUI is tested in-process).

## D83 Releases, install scripts and `planrun update`

planrun is installed and kept current from GitHub releases; `Renseck/planrun` is public, so no
step needs a token.

- **CI.** `.github/workflows/ci.yml` runs the gate of every task (fmt on Linux; clippy, test, and
  both with the `e2e` feature, D82) on `windows-latest` and `ubuntu-latest`, with
  `PLANRUN_REQUIRE_SHELLS=1`, on pushes to `main`, pull requests, and when the release calls it.
  The private local corpus (D16) is not there, so its ignored tests do not run.
- **Releases.** A pushed tag `v<major>.<minor>.<patch>` whose version equals
  `[workspace.package]`'s runs `.github/workflows/release.yml`: the CI gate, the tag check, a
  `--release --locked` build per target, and a GitHub release with generated notes and one
  archive per target: `planrun-<tag>-x86_64-pc-windows-msvc.zip` and
  `planrun-<tag>-x86_64-unknown-linux-gnu.tar.gz`, the executable at the archive's root. The
  Linux build runs on the oldest supported Ubuntu runner, so the binary needs an older glibc.
  Other targets are built from source. No checksum file: TLS carries the download, and the
  `--version` check below catches a wrong or broken binary.
- **Install scripts.** `scripts/install.ps1` and `scripts/install.sh` download the latest
  release's archive into `%LOCALAPPDATA%\Programs\planrun\bin` or `~/.local/bin`, or build the
  clone they are in with `cargo install --locked --root` into the same directory
  (`-FromSource`, `--from-source`). The PowerShell script adds the directory to the user `PATH`
  (`-NoPath`: not); the shell script only warns.
- **`planrun update [--check]`** (planrun-cli, `update.rs`). It asks
  `api.github.com/repos/Renseck/planrun/releases/latest` with curl, compares the tag with the
  build's `CARGO_PKG_VERSION` (`major.minor.patch`, numerically; a tag that is no version is an
  error), and when the release is newer downloads the build's own target's archive into
  `planrun-update-<pid>` under the OS temp directory, unpacks it with tar, runs the unpacked
  `planrun --version`, which must print the release's version, and only then swaps it in.
  `--check` reports and changes nothing. Exit 0 up to date, reported or updated; 1 otherwise,
  with a code: `update-tools`, `update-fetch`, `update-unsupported`, `update-no-asset`,
  `update-download`, `update-verify`, `update-swap`.
- **curl and tar are external tools**, like git: no HTTP, TLS or archive crate (D9). On Windows
  they are `%SystemRoot%\System32\curl.exe` and `tar.exe` by absolute path (Windows 10 1803 and
  later), so Git for Windows' curl or GNU tar earlier on `PATH` never runs; GNU tar does not read
  zip.
- **What it writes.** D7's rule is about plan execution. `update` is not one: it writes its
  directory under the OS temp directory, and in the directory of `current_exe()` only
  `planrun.new[.exe]`, `planrun.old[.exe]` and the executable itself. The new binary is copied
  in as `.new` (mode 755 on Unix) and renamed over the executable: on Unix one atomic rename; on
  Windows, where a running executable can be renamed but not replaced or deleted, the executable
  goes aside as `.old` first, and comes back when the second rename fails.
- **Cleanup.** Every planrun start removes `planrun.old[.exe]` and `planrun.new[.exe]` beside its
  executable, best effort and silently: one or two file removals, no process. On Windows the
  `.old` of the update just made stays until the next start, because it is still running.

## D84 The TUI: one plan view, two modes

Amends D15. The design is docs/tui-design.md; D84–D92 are its decisions.

- **`planrun tui`** is a second `Ui` over the same crates, in a crate of its own, `planrun-tui`
  (ratatui and crossterm), composed by planrun-cli. It shows the domain model and adds no model
  of its own. Resume, policy and preflight stay authoritative: the TUI explains and asks; it never
  decides and never turns one decision into another (D74). The line prompts stay (D74).
- **One plan view, two modes.** Inspect is filled from a dry run (D55); Run from the live runner.
  Same layout, same tree. Inspect states the policy it assumes (`--auto`,
  `--run-expected-failures`, what `Policy::new` takes), and `R` runs with exactly that policy.
  The dry run is advisory: the real preflight decides at run time, so a stale Inspect never
  authorises anything.
- **What Inspect reads.** The dry run's report is typed: `Outcome::Ok` and `Outcome::WouldAsk`
  carry an `Effect` (create, overwrite, a patch with its blob ids, replace, delete), whose
  `Display` is the text `planrun run --dry-run` prints, so the CLI's output does not change. The
  class and policy column come from `DefaultClassifier::explain` and `Policy::decide`, both pure,
  called by the TUI.

## D85 Questions open in a dock; the resume preview only shows

Amends D15.

- A question opens in a dock over the console's region and leaves the plan tree and the change
  inspector visible. There are no centred modals.
- **The resume preview** is a read-only full view, built from `analyse` and
  `runner::preview::preview`: whether a run is live (D90), the plan's fingerprint against the
  journal's, `.git/index.lock`, resume's own verdict on HEAD (unchanged, moved by the action that
  was running, or moved so that resume stops; computed by the function resume uses), each
  prepared step of the interrupted task as it is on disk (Done, NotDone or Unknown, whether it
  could be restored, the writer's temp files), the scripts that will ask "run again or keep",
  and the paths resume will ask about as `resume-dirt` (never an Unknown step's: restoring puts
  them back, accepting verifies them).
  It takes no lock and writes nothing, and it shows when it was read. Every question in it is labelled "asked when resume reaches it":
  `Runner::resume` takes the lock, checks everything again against the disk as it is then, and
  asks in the dock, each answer journaled when asked (D74, D77). Answering in the preview would
  be a decision of its own, with the runner checking that the state still matches: not now.

## D86 `QuestionKind`

Amends D74.

- `Question.kind` is an enum, `QuestionKind`, one variant per question the runner asks, with what
  the run knows about it that the plan does not say as its payload: the decision (`run`), the
  classifier's reasons (`typed-yes`), the matcher's reasons and `errored` (`confirm`), the status,
  the mismatches, the block's output lines, the expected lines aligned to them, the commit
  problem and whether the session is dead (`mismatch`), the dirty paths (`dirt`,
  `resume-dirt`), the shell and directory (`outside`), why a session is new and the earlier
  tasks (`session`), the gate's reasons (`gate`), the changed paths (`changed`), whether the
  script was running (`again`), the step's journal place with each path's state and whether it
  can be restored (`unknown`, and the place for `accept-done`), and the running action's place
  with both HEADs (`head-moved`). What the plan says is reached through `Question.at`; resume's
  questions name their step or action by its journal place, since they have no `at`.
- **Typed answers are explicit.** `Question.typed` is the list of words a typed question
  accepts (`yes`, `skip`, `quit` for a Destructive script; `yes`, `quit` for the commit gate and
  for accepting a step as done), and `None` for a question of choices. The runner reads it back
  to decide what a word does, so the list and the behaviour cannot drift.
- `QuestionKind::name()` is the journal name (`run`, `typed-yes`, …): the journal format does not
  change.
- A UI matches on it exhaustively, so the compiler finds a question it does not show. Its keys
  and typed input come from `Question.choices` and `Question.typed`; it shows every answer the
  runner offers and no other.
- **The matcher's alignment:** `expect::align` returns each non-blank expected line with the
  output line it matched (or none), the way `expect::check` matches them (D66); `check` is
  unchanged.

## D87 Phases are not a navigator level

Amends D15. One plan file is one phase (D5): the TUI's Plans view is a table of plan files, and
the plan view's tree is Task → Step → Action.

## D88 No workspaces yet

Amends D15. `planrun tui [<plans-dir>]` runs from the target repository, like `run`. A list of
repositories is a later decision.

## D89 Colour and glyphs

Amends D15.

- 24-bit colour, chosen when `COLORTERM` is `truecolor` or `24bit` or `WT_SESSION` is set. The
  256-colour fallback and `NO_COLOR` (bold for focus, reverse for selection) come after the first
  version; the colour tokens keep their 256-colour indices for it.
- Green only after verification, amber only when planrun waits for you (or for a warning), red
  for failure and the Destructive class, blue for focus and the running action, purple for
  identifiers. Bold and reverse only.
- The glyphs are single-width in `unicode-width` (a unit test) and checked by hand in Windows
  Terminal with Cascadia Mono; each has an ASCII fallback, and `PLANRUN_ASCII=1` switches to them
  all. No Nerd Font, no wide or emoji glyphs.

Amended by D94 (v1 renders 24-bit colour without detecting it).

## D90 The runner stays synchronous, on a worker thread

Amends D15, D76 and D77.

- The runner runs on a worker thread; the render thread owns the terminal and never waits on git,
  a shell or the network. The runner reaches the terminal only through blocking round trips:
  `ask`, `shell`, `view` and `diff`. For `shell` the render thread leaves the alternate screen,
  turns raw mode off, sets D73's "shell running" flag, runs the shell, restores the terminal and
  replies; `view` and `diff` are answered in the TUI with the dock left open.
- **The journal drives the live view.** The TUI's tree, its outcome tags, its change states and
  its history come from the journal's records (D76), which a journal observer forwards over a
  channel; there is no event stream beside it. A journal takes any number of observers, called
  in the order they were added, so the TUI's and the `e2e` seam's (D82) coexist. `Ui::status` is
  shown as prose and never parsed; `Ui::output` carries each output line with its stream.
- **Background work** (plan checks, `update --check`, `git status --porcelain
  --no-optional-locks`, journal scans, the liveness probe) runs on threads; each result carries
  the fingerprint of its input. Background checks never replace preflight.
- **A live run is seen through a second lock, `<git-dir>/planrun/lock.live`.** A run takes
  `lock` as before (a try: held means "another planrun is running", D77), then `lock.live` with a
  blocking lock, and holds both until it ends; the operating system releases them however it
  ends. `lock::live` probes with a try on `lock.live` only and lets go at once: a run starting
  while a probe holds it waits a moment, never fails. A successful try means no run is live;
  `lock.pid` is then only shown, never trusted (it stays when its run ends). The one gap: a probe
  in the instant between a starting run's two locks sees no live run. Never probe `lock` itself:
  a try there would make a starting run fail.

## D91 Ctrl-C in the TUI ends planrun, as it does on the line prompts

Amends D73.

- Raw mode makes Ctrl-C a key. The TUI confirms in the dock, restores the terminal, and calls
  `shell::end_now(code)`, which ends planrun the way an interrupt does: every session's process
  tree is killed, no destructor runs and no `run-end` is written, so `resume` goes on from the
  journal (D73, D77). On Windows it is `process::exit`: exiting closes every job handle, and
  `KILL_ON_JOB_CLOSE` kills each tree (D64). On Unix it first kills every registered process
  group with the loop the signal handler uses, now one function that both call; no `unsafe` is
  added.
- Cancelling the run while planrun stays up would kill a running block from inside (D64): not
  offered. Nor is a pause: asking under `--auto` would turn Auto into Ask, a decision change the
  UI may not make. If it is wanted, it is a decision of its own with runner support.

## D92 The TUI's dependencies

Amends D9. `ratatui`, `crossterm` and `unicode-width`, in planrun-tui only. crossterm
brings `windows-sys`, `mio` and `signal-hook`, which D64 and D73 keep out of planrun-exec; they
stay out of it: planrun-exec's dependencies and its `unsafe` surface (`shell/kill.rs`) do not
change.

## D93 The TUI is tested in-process

Amends D82 for the TUI.

- The TUI cannot be driven through pipes: crossterm reads keys from the console (on Windows
  `ReadConsoleInput`), so driving the binary would need a pseudo-console, which means a new
  dependency or `unsafe` code outside `shell/kill.rs`. Instead, planrun-tui's loop is one core
  (`driver::Core`) over any ratatui backend, fed keys, ticks and the messages of its threads.
  The binary gives it crossterm's terminal and events; the tests a `TestBackend` and scripted
  keys.
- **Its acceptance** runs the real runner on a copy of the greeter fixture
  (`tests/support/fixture.rs`) through that core: the synthetic assay plan is inspected, run,
  answered in the dock and committed, and the journal and the tree are checked as P5's binary
  test checks them. A crash is a panic from a journal observer, as in planrun-exec's resume
  tests, and the resume preview and resume are driven from the journal it leaves.
- **Snapshots are cell grids.** `grid::grid` prints each row's characters and then its style runs
  by token name (`act/bg2 b`), so a snapshot reads against the design's artboards cell for
  cell. Times come from a fixed clock, and the footer's target and plans labels are fixed.
- What only a real terminal shows (the glyphs in Cascadia Mono, the keys, the `[o]` shell, Ctrl-C,
  a resize, the terminal restored after a panic) is a manual check list in the crate's README.

## D94 How `planrun tui` starts and reads keys

Amends D89.

- **A terminal on stdin and stdout**, or it stops before raw mode with exit code 2 and says
  that `planrun run` is the line version.
- **The repository is the current directory's** (D88). The plans directory is the argument,
  relative to the current directory, or `docs/plans` under the repository's root; a missing
  directory is shown in the Plans view, not refused. Its `*.md` files are listed, not
  recursively; a file no format detects is listed as "not a plan".
- **Keys:** only presses count (Windows reports releases too), and a held key's repeats only for
  movement. One loop polls crossterm's events and drains the app's channel between polls: no
  input thread, which would read the keys meant for the `[o]` shell. Ctrl-C is a key in raw mode
  (D91). VS Code's terminal keeps `ctrl+k` and `ctrl+p` for itself unless
  `terminal.integrated.sendKeybindingsToShell` is set; Windows Terminal passes them.
- **Colour:** v1 renders 24-bit colour without looking at `COLORTERM` or `WT_SESSION`; the
  detection comes with the 256-colour fallback.
- **Times are UTC**, marked so where a date is shown: std has no time zone, and a time-zone crate
  for one label is not worth a D9 line. The console shows the time since the run started.
- **The plan files are checked in the background** on start, every two seconds when a file's
  length or modification time changed, and on `c`.
- **Interrupts** (D73) are installed once, before the terminal is set up, not at each run: the
  `[o]` shell's `InterruptsIgnored` relies on them.
- **A panic** on the render thread restores the terminal, then reports as Rust does. On any
  other thread (the runner's, a background job's) it leaves the terminal alone and prints
  nothing: the thread catches it, and its message comes back to the app as that work's failure
  (a run that ended with an error, a job in its failed state).

## D96 Discovery: format specs and plan files

Amends D41 and D9. Decided in the P7 brainstorm (docs/superpowers/specs/2026-11-28-discovery-design.md).

- **One `Discovery` value in planrun-exec** (`discover`): it reads the user's and the
  repository's format specs and the repository's `.planrun/config.toml` when it opens, starts no
  process, and returns spec texts, never a `Format` (D2). Callers hand the texts to the registry.
- **The root** is the nearest directory upwards from the current directory with a `.git` (a
  directory, or a file for a worktree or submodule), as for `run` and the TUI (D88, D94).
  Outside a repository there is no repository layer and no config file; that is not an error.
- **Format directories:** `<root>/.planrun/formats/` and `<user>/formats/`, where `<user>` is
  `PLANRUN_CONFIG_DIR` when set, else `%APPDATA%\planrun` on Windows and
  `$XDG_CONFIG_HOME/planrun` or `$HOME/.config/planrun` elsewhere; no `dirs` crate. A missing
  directory is empty. Only `*.toml` files are read, sorted by name; a stem that is not letters,
  digits, `-` and `_` is `format-file-name`, a file that is not UTF-8 is `format-io`. A spec that
  cannot be used stops the command (exit code 2) and names its file: never skipped, also not a
  user-level one that every repository sees (D3).
- **`.planrun/config.toml`** v1 has one key, `[plans] globs`, which replaces the default globs
  `docs/plans/*.md`, `**/superpowers/**/plans/*.md` and `**/plans/*.md`. Unknown keys are
  `config-parse`, a refused glob `config-glob`, `globs = []` `config-globs-empty`. A default
  format for `--format` was left out: detection picks it, and `--format` covers the rest.
- **Discovery globs** are D41 globs plus `**`, a whole path segment that matches zero or more
  segments (`PathPattern::discovery_glob`). `**` inside a segment is refused. Manifest globs keep
  refusing `**`, so no plan changes meaning.
- **`toml` in planrun-exec**, for the config file: a crate the workspace has already (D9).
