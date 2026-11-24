# Starter prompt for Claude Code

Paste the block below as the first message of a Claude Code session started in the repository
root, after these setup steps (by hand):

1. Unzip the package, `cd planrun`, `git init`, `git add -A`, `git commit -m "P0: starter package"`.
2. Copy the six real plans into `tests/corpus/local/` (listed in its README). They are
   git-ignored.
3. `cargo test` once: 8 `test result: ok` lines.

```text
You are taking over planrun, a Rust CLI that executes structured Markdown implementation plans
against a git repository, task by task. The design is settled; this repository is the starter
package (phase P0).

Read, in this order, before anything else:
1. CLAUDE.md — the hard rules and conventions. They are binding.
2. docs/architecture.md — how it works.
3. docs/decisions.md — D1–D24, the reasons behind the rules.
4. docs/plan-grammar.md and docs/extending.md — the plan formats and the extension model.
5. docs/roadmap.md — phases; you start with P1.
6. docs/plans/README.md — how planrun's own plans are written (the assay format).
7. The code: crates/planrun-model (complete IR), crates/planrun-parse (traits and spec schema,
   implementations pending), crates/planrun-exec and crates/planrun-cli (stubs).
8. tests/corpus/synthetic/assay-style.md, tests/corpus/invalid/*, formats/assay.toml.
9. The real plans in tests/corpus/local/, if present: read the two assay-format plans
   (2026-11-05-s5-1-p1-routes-seam.md, 2026-11-11-s5-1-p3c1-builder-core.md) fully, and skim the
   other four for their structure. Never copy their content into a committed file (D16).

Then:
A. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`, and
   report the result.
B. Report, briefly and concretely: anything in the docs that contradicts itself or the code;
   anything in formats/assay.toml or the synthetic corpus that the two real assay plans would
   break (patterns that don't match real headings, notices, lead sentences or Expected blocks);
   and the decisions P1 still needs. Put the questions for Rens in one code block.
C. Stop and wait for Rens' answers.
D. After the answers: write the P1 plan as docs/plans/<today>-p1-parse-assay.md, following
   docs/plans/README.md and the P1 scope and acceptance in docs/roadmap.md. Full contents for every
   file. Stop after writing it; Rens reviews it before anything is executed.

Do not implement P1 before the plan is reviewed. Do not push. Do not add dependencies without a
line in docs/decisions.md D9.
```
