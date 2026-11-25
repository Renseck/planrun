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

Amended by D42 (wildcards), D51 (Windows-reserved spellings) and D52 (pre-images in the object
database, the writer's temp file).

## D8 Formats are data first; built-ins are specs too

Three extension levels: TOML spec (no rebuild) → recognizer kind (Rust) → Format (Rust). The
built-in formats are TOML specs embedded in the binary, so user formats take the same path.

## D9 Dependencies

Each dependency is justified here; add none without a line.

| Crate | Why |
|---|---|
| `pulldown-cmark` | CommonMark with byte offsets; correct nested-fence handling |
| `regex` | spec patterns; linear time, no backtracking blow-ups |
| `serde`, `toml` | format specs |
| `serde_json` | the journal (JSONL) |
| `thiserror` | error types |
| `sha1_smol` | git blob ids in-process: the `blob-chain` check from a created file's contents (planrun-parse only; planrun-exec asks `git hash-object`, D56) |
| `similar` | showing diffs of overwrites and replacements. **Not** for applying patches (D20) |
| `clap` | arguments |
| `dialoguer` | line prompts (D15) |
| `annotate-snippets` | diagnostics with source context |
| `insta` (dev) | golden snapshots of the IR |
| `sha2` (Phase 1) | the plan fingerprint (SHA-256) |

Rejected: `rusqlite` (D18), `ratatui` for now (D15), `tracing` for now (the journal is the record),
`similar` as a patch engine (D20).

## D10 Line endings of written files

Created files: as in the plan (LF). Overwrite and `ReplaceExact`: keep the target's existing
convention unless `.gitattributes` says `eol=lf`; `before` is matched in LF space. Patches: through
`git apply` unchanged.

Amended by D54 (attributes, files without or with mixed line endings).

## D11 Prose never passes a script on its own

`Expected: all pass` is mapped to an exit expectation, checked, and then shown for confirmation.
Only fenced summary lines and exit codes pass automatically.

## D12 Expected failures are skipped by default

A verification expected to fail (TDD "verify it fails", `Expected: build error`) is recorded in
the IR and skipped; `--run-expected-failures` runs it. Rens sees these failures in the IDE.

## D13 Policy classes

Read < Build < Modify < Destructive; a script takes the highest class of its commands; unknown
commands are Destructive. Defaults: file actions automatic; Build/Modify ask; Destructive needs a
typed `yes`; Manual always waits.

## D14 Resume checks the disk, never the journal alone

An action recorded `Running` or `Done` is re-examined: postimages present → done; preimages
present → not done; otherwise the user decides.

## D15 Line prompts first

`dialoguer` and plain output keep cargo's output in the terminal's scrollback. A ratatui TUI can
come later behind the `Ui` trait.

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

## D19 Windows first

Every real plan is PowerShell 7 on Windows. Linux (pwsh and bash) is supported and is where CI
can run.

## D20 Patches are applied by `git apply`

With the patch on stdin, `--whitespace=nowarn`, after `--check`. Same semantics as Rens' manual
runs; exact context; no fuzz. Blob ids from `index` lines are verified before and after.

## D21 Note vs Manual

Prose no recognizer consumed is a `Note` (shown, never blocks). An unconsumed fence, prose that
matches a format's manual markers, and edits described in prose are `Manual` (always blocks).

## D22 Commits are gated twice

The plan's commit script runs as written (it may gate itself on `$gate`), and planrun runs it only
when every verification of the task passed or was skipped by policy (D12).

## D23 File actions are transactional per step

All file actions of a step, up to the next non-file action, are validated in memory before any is
written; pre-images are stored in the git object database first; writes are temp-file + rename.

Amended by D52 (the virtual tree, the writer) and D53 (prepare, then commit).

## D24 One persistent session per shell kind per run

Variables and the working directory persist across blocks (`$gate`, `$P2`), as in Rens' terminal.

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