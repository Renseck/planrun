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