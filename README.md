# planrun

Executes structured Markdown implementation plans against a git repository, task by task and step
by step, the way you would by hand: write the files the plan gives, apply its exact diffs, run its
commands, check their expected output, commit when the gate passes. Anything it cannot do
mechanically it shows and waits for. It is not an agent.

```
planrun check plan.md          parse, check, coverage report; changes nothing
planrun show  plan.md P1-T2    the typed form of a task
planrun run   plan.md          execute interactively (--dry-run: change nothing)
planrun resume plan.md         continue after an interruption, re-checking the disk
```

Status: starter package (P0). See [docs/roadmap.md](docs/roadmap.md).

## Documents

| | |
|---|---|
| [CLAUDE.md](CLAUDE.md) | rules for working on this code |
| [docs/architecture.md](docs/architecture.md) | how it works |
| [docs/plan-grammar.md](docs/plan-grammar.md) | the plan formats it reads |
| [docs/extending.md](docs/extending.md) | adding a plan format |
| [docs/decisions.md](docs/decisions.md) | decisions D1–D24 |
| [docs/roadmap.md](docs/roadmap.md) | phases P1–P6 |
| [docs/plans/](docs/plans/) | phase plans (assay format) |
| [STARTER_PROMPT.md](STARTER_PROMPT.md) | the prompt that hands this package to Claude Code |

## Build

```powershell
cargo build
cargo test
```

Rust 2024 edition, stable toolchain. Windows with PowerShell 7 is the primary platform; Linux works.
