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
| `sha1_smol` | git blob ids in-process: the `blob-chain` check from a created file's contents (planrun-parse); the fast path in planrun-exec, where `git hash-object` is the authority |
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
