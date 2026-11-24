# planrun

A Rust CLI that executes structured Markdown implementation plans against a git repository, task by
task, step by step: exactly what the plan says, nothing more. It is not an agent. Read
`docs/architecture.md` before changing code; `docs/decisions.md` holds the rules below with their
reasons (cited as `D<n>`).

## Priorities (D1, D25)

Correctness > maintainability/extensibility > execution speed > low memory > predictability.
When two of these conflict, the earlier one wins. Speed means no wasted process starts or repeated
reads; don't optimize I/O speculatively (cargo/npm dominate wall time).

## Layout

```
crates/planrun-model   the IR: Plan, Task, Step, Action, Span/TextRef, RepoPath, AbbrevBlobId
crates/planrun-parse   Markdown → IR: source, document, format, recognizer, registry, spec, check
crates/planrun-exec    IR → effects: workspace, git, shells, expect, policy, journal, runner
crates/planrun-cli     the `planrun` binary
formats/*.toml         built-in plan formats (embedded); the file name is the format id
docs/                  architecture, plan-grammar, extending, decisions, roadmap, plans/
tests/corpus/          synthetic/ (one plan per format), invalid/ (one error each), local/ (private)
tests/fixtures/greeter the repo the synthetic assay plan applies to
```

## Hard rules

1. **planrun-parse never executes and never touches the repository. planrun-exec never depends on
   planrun-parse** (D2). Don't add a dependency that crosses these lines.
2. **No partial plans** (D3). Any error means no `Plan`, nothing runs.
3. **Nothing is dropped.** Every block of a task becomes an action, `Note` or `Manual` (D21). Once
   a recognizer's distinctive lead matched, a broken idiom is a `Malformed` error, never a
   fallback.
4. **The action set is closed** (D4). New plan wording maps onto existing actions. Adding an
   `ActionKind` follows docs/extending.md §4, all in one change, and only with Rens' agreement.
5. **New plan styles are data first** (D8): a TOML spec, then a recognizer kind, then a hand-written
   `Format`, in that order of preference (docs/extending.md).
6. **Paths in actions are `RepoPath`** (D7). Nothing outside the repository is written except
   `.git/planrun/` and the OS temp directory. The working tree gets only what the plan writes.
7. **Patches go through `git apply`** with exact context (D20). No fuzz, no `similar`-based
   patching.
8. **Prose never passes a script on its own** (D11); expected failures are skipped by default (D12).
9. **No new dependency without a line in docs/decisions.md D9.**
10. **Real plans stay in `tests/corpus/local/`** (git-ignored, D16). Never copy their content into
    committed files, tests, snapshots, docs or commit messages. Synthetic plans only.

## Commands

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test -- --ignored local_corpus      # parses the private real plans, when present
cargo run -p planrun-cli -- check tests/corpus/synthetic/assay-style.md
```

The gate for every task is all three of fmt, clippy (warnings are errors) and test.

## Code conventions

- Library code: no `unwrap`/`expect` (clippy warns; the gate fails). Tests may use
  `unwrap_or_else(|e| panic!("{e}"))`.
- Every public item has a doc comment (`missing_docs` warns).
- Diagnostics have a stable kebab-case `code`; tests pin codes, snapshots pin messages.
- Text from the plan is a `TextRef`, never a copied `String`, except `TextRef::Owned` where the
  source isn't contiguous.
- Prefer enums over strings, traits for extension points, composition over deep generic layering.
- Keep functions flat: guard clauses and early returns, at most three levels of nesting.
- Recognizers are pure; parameters are validated when a spec loads, not when a plan is parsed.

## Tests

- Unit tests sit next to the code (`mod tests`), or in `<module>_tests.rs` once a file passes
  ~400 lines.
- `insta` snapshots for IR and CLI output. A changed snapshot is a review item: explain it in the
  commit, never re-pin by reflex.
- Adding a check rule adds a file to `tests/corpus/invalid/`. Adding a format adds a plan to
  `tests/corpus/synthetic/` with its snapshot.
- Execution tests run on a temporary copy of `tests/fixtures/greeter` (`git init` in the copy),
  never on this repository.

## Docs stay in step

- A changed decision: a new `D<n>` in docs/decisions.md that supersedes the old one.
- A changed spec schema: `spec.rs`, docs/extending.md §2 and `formats/*.toml` together (the
  `spec_files` test checks the files).
- A new recognizer kind: docs/extending.md §3.
- A new format: docs/plan-grammar.md.

## Working with Rens

- Workflow: brainstorm → design spec → implementation plan (`docs/plans/`, assay format, see
  `docs/plans/README.md`) → execution. Propose before making non-trivial edits; stop after writing
  a plan.
- Direct, technically precise; no flattery. Put review text and anything meant to be pasted in
  code blocks.
- Rens works on Windows with PowerShell 7. Commands in plans are PowerShell 7.
- One commit per task, message `P<N>-T<n>: <title>`. Don't push.
