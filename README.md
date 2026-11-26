# planrun

Executes structured Markdown implementation plans against a git repository, task by task and step
by step, the way you would by hand: write the files the plan gives, apply its exact diffs, run its
commands, check their expected output, commit when the gate passes. Anything it cannot do
mechanically it shows and waits for. It is not an agent.

```
planrun check  plan.md         parse, check, coverage report; changes nothing
planrun show   plan.md P1-T2   the typed form of a task
planrun run    plan.md         execute interactively (--dry-run: change nothing)
planrun resume plan.md         continue after an interruption, re-checking the disk
planrun status plan.md         what the plan's journal says
planrun update                 install the latest release (--check: only report)
```

Status: P0–P5 are done; P6 is planrun running its own plans. See
[docs/roadmap.md](docs/roadmap.md).

## Install

Windows (PowerShell 7 or Windows PowerShell 5.1), the latest release into
`%LOCALAPPDATA%\Programs\planrun\bin`, added to the user `PATH`:

```powershell
irm https://raw.githubusercontent.com/Renseck/planrun/main/scripts/install.ps1 | iex
```

Linux x86_64, the latest release into `~/.local/bin`:

```sh
curl -fsSL https://raw.githubusercontent.com/Renseck/planrun/main/scripts/install.sh | sh
```

From a clone, built with cargo into the same directory:

```powershell
.\scripts\install.ps1 -FromSource      # Windows
sh scripts/install.sh --from-source    # Linux
```

`-InstallDir` / `--install-dir` choose another directory. Then keep it current:

```
planrun update --check   is a newer release out?
planrun update           install it over this executable
```

`planrun update` uses curl and tar (Windows 10 1803 and later ship both) and replaces only its
own executable (decision D83).

## Documents

| | |
|---|---|
| [CLAUDE.md](CLAUDE.md) | rules for working on this code |
| [docs/architecture.md](docs/architecture.md) | how it works |
| [docs/plan-grammar.md](docs/plan-grammar.md) | the plan formats it reads |
| [docs/extending.md](docs/extending.md) | adding a plan format |
| [docs/decisions.md](docs/decisions.md) | decisions D1–D83 |
| [docs/roadmap.md](docs/roadmap.md) | phases P0–P6 |
| [docs/plans/](docs/plans/) | phase plans (assay format) |
| [docs/handovers/](docs/handovers/) | the state at the end of each phase |

## Build

```powershell
cargo build
cargo test
```

Rust 2024 edition, stable toolchain. Windows with PowerShell 7 is the primary platform; Linux
works. CI runs the full gate on both (`.github/workflows/ci.yml`); a pushed `v*.*.*` tag
releases (`.github/workflows/release.yml`).

## License

MIT; see [LICENSE](LICENSE).
