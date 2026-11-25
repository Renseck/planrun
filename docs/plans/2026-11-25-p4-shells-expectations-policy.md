# planrun P4 — Shells, expectations, policy

**Goal:** P4 of docs/roadmap.md: persistent pwsh and bash sessions with the sentinel protocol and a process tree that a timeout or a drop kills whole; the expectation matcher; the command classifier, a built-in allowlist over a splitter per shell; the policy decisions P5's runner will act on; and the class of every script in `planrun run --dry-run`. Decisions D60–D70. After P4 a block runs in a session at the repository root with no stdin and its output streamed, `expect::check` says whether it passed, asks or mismatched, and `Policy::decide` says whether it runs, asks, needs a typed `yes`, waits or is skipped.

- **P4-T1** (docs, workspace lints): decisions D60–D70 with their amended-by lines in D9, D11, D12, D13 and D24; `unsafe_code` denied in the workspace and forbidden at the roots of planrun-model, planrun-parse and planrun-cli; `undocumented_unsafe_blocks` denied.
- **P4-T2** (`planrun-exec`): `shell/`: the session protocol (D60), how a block ends (D61), the environment (D62), which shells (D63), the process tree in `kill.rs`, planrun's only `unsafe` code (D64); the shell tests on fixture copies.
- **P4-T3** (`planrun-exec`): `expect`: the matcher (D66), with table tests.
- **P4-T4** (`planrun-exec`): `policy/`: the pwsh and bash splitters and the tables (D68), with table tests.
- **P4-T5** (`planrun-exec`): `policy::Policy`: `--auto`, typed yes, unscoped file actions, expected failures skipped (D67, D69).
- **P4-T6** (`planrun-cli`): the dry run shows each script's class and decision (two snapshots change); the ignored `local_classes` test.
- **P4-T7** (docs, `CLAUDE.md`): architecture.md §6.3–§6.5 and §7, extending.md §7, the roadmap, the CLAUDE.md layout, commands and test conventions.

**Base:** `dbfbd36` (`File repair`), the commit that applies the P3→P4 handover's open item 0; at it the nine repaired files have the verified blob ids (stated decision 1). Execute from the commit of this plan. Gate per task: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, with `PLANRUN_REQUIRE_SHELLS=1`, so a missing pwsh or bash fails the shell tests instead of skipping them.

**Shell and environment:** PowerShell 7 on Windows; Rust 1.95.0 (stable); git on `PATH`; pwsh 7 on `PATH` and Git for Windows (its `usr\bin\bash.exe` is found from `git --exec-path`, D63). No new crate; `Cargo.lock` does not change.

**Before P4-T1:** this plan committed, and a clean `git status --porcelain`. The real plans stay in `tests/corpus/local/` (git-ignored, D16); this plan quotes none of them, and names two by file only where it carries docs/roadmap.md in full (its P1 acceptance list, unchanged). Execution tests run on temporary copies of `tests/fixtures/greeter` under the OS temp directory, never on this repository; shell sessions start in such a copy, with its git isolation.

**How this plan was verified:** every task was built in a scratch clone of `dbfbd36` outside this repository (`core.autocrlf=false`), one commit per task. At each commit `cargo fmt --check`, clippy and `cargo test` (with `PLANRUN_REQUIRE_SHELLS=1`) pass, and so does the task's targeted command; the summary lines below come from those runs. After every task, `planrun show` and `planrun check` on the six local plans printed exactly what they printed at `dbfbd36`, and the local-corpus test passed. The plan was generated from that history (full contents from `git show`, exact diffs from `git diff`, blob ids from `git rev-parse`), `planrun check` passes on it, and it was replayed task by task on a fresh clone with `core.autocrlf=true`: every commit step committed, `git status --porcelain` was empty after each, and the replay's final tree is the scratch tree.

## Review rulings (P4 questions, 2026-11-25)

Each ruling is a decision in docs/decisions.md (all written in P4-T1); this list maps them.

1. **Q0** base: Rens applied the handover's repair as `dbfbd36`; checked (stated decision 1).
2. **Q1** pwsh status: `Threw` or `$LASTEXITCODE` reset to 0 before the block; the prologue pins `$ErrorActionPreference = 'Continue'` and `$PSNativeCommandUseErrorActionPreference = $false`; `errored` from `$Error[0]` by reference, never a count; it makes a run with no expectation or an exit-only Success ask (D61, D66). The measured table is in D61.
3. **Q2** bash `exit` is `SessionEnded(code)`, EOF without a sentinel `SessionEnded(None)`, a misfit sentinel `Broken`; nothing restarts a session silently (D61).
4. **Q3** an unstated exit that is not 0 asks (D66).
5. **Q4** leading whitespace counts; only `…`; anchored first and last segments; the normalization rules (D66).
6. **Q5** the tables are Rust constants; the allowlist is built in, one classifier per runner (D68).
7. **Q6** nested shells are Destructive, their strings never parsed (D68).
8. **Q7** DevReady by agent mode, pwsh only, from 1.19.1 (D68).
9. **Q8** Git for Windows' bash from `git --exec-path`, probed, never `PATH`; `/usr/bin` prepended, not `/mingw64/bin`; pwsh 7 or later (D63).
10. **Q9** no default timeout; P5 decides (D65).
11. **Q10** the run-from candidate is deferred, with the ruled reason and what it would take (D70).
12. **Q11** the dry run shows each script's class and decision; the ignored `local_classes` test (D69, P4-T6).
13. **K** `unsafe_code = "deny"`, `#![forbid(unsafe_code)]` in the other three crates, `#![allow(unsafe_code)]` only in `shell/kill.rs`, `undocumented_unsafe_blocks` denied, the layout size and the read-back flag tested, no fallback without the tree, `kill(-pgid)` only before the wait (D64).
14. **Rulings on the wrapper, environment, process tree, matcher, policy and classifier** (second review): D60, D62, D64, D66–D69, with the tests each asked for (P4-T2 to P4-T5). Where the build found a premise that does not hold, the stated decisions below say so (2, 3).

## Stated decisions made while building

1. **Base.** `dbfbd36`'s parent is `249e8da`; it changes exactly the nine files of open item 0, and at it each has the verified blob id (`git rev-parse dbfbd36:<path>`). fmt, clippy, `cargo test` (23 `test result: ok` lines) and the local-corpus test passed there.
2. **pwsh blocks do not come over stdin** (a ruling's premise that does not hold). A native command started by pwsh inherits pwsh's own stdin: with blocks on stdin, `git hash-object --stdin` in a block read the protocol and hung until the timeout (first build of `pwsh_and_bash_blocks_get_no_stdin`). So the pwsh session gets a null stdin, and its `-Command` script is the prologue plus a top-level loop that waits for `b<n>.ps1` with a `FileSystemWatcher` (re-checked every 250 ms) and dot-sources it inside `do { try { … } catch { … } } while ($false)`. The ruling's reasons all still hold: no path travels over a pipe (pwsh reads nothing; the bash line names only `$__planrun_dir/b<n>.sh`), the run directory comes through `PLANRUN_SESSION_DIR`, which the prologue removes, and a non-ASCII run directory is tested (`a_run_directory_with_non_ascii_spaces_and_quotes_works_and_is_removed`). Block files are written as `b<n>.tmp` with `create_new` and renamed into place, so the loop never reads half a block. The `do` keeps a block's `break` and `continue` inside it (`pwsh_break_trap_and_a_parse_error_still_reach_the_sentinel`).
3. **bash reports its directory with `pwd -W` on Windows.** There `$PWD` is an MSYS path, and the OS temp directory is the `/tmp` mount, so `/c/…` conversion is not enough; `$(builtin pwd -W)` costs a fork per block, on Windows only. Elsewhere `$PWD`.
4. **`-ExecutionPolicy Bypass`** for the pwsh session process on Windows: a dot-sourced block file is a script to the execution policy, where the same text typed in the terminal is not.
5. **A terminating error ends the block** (`Threw`), also a statement-terminating one, after which a block pasted into the terminal would go on with the next statement. The sentinel has to follow the block, and such a block is a failure either way. Listed with the other differences from the terminal in D60 and architecture.md §6.3.
6. **`SessionConfig::temp`**: where the run directory is made. P5 passes the OS temp directory; the tests pass a directory next to the fixture copy, which is how the non-ASCII case is tested without changing the process environment (`set_var` is `unsafe`).
7. **The tree owns the child.** `Tree::kill` does nothing once `Tree::wait` reaped the process, so `kill(-pgid)` after the wait cannot happen by construction. A dead session refuses the next block with an error, and `Session::is_alive` says so.
8. **Moved types.** `ScriptRun` and `ShellSession` live in `shell`, `PolicyClass` and `CommandClassifier` in `policy`; lib.rs re-exports all four under their old paths. `ShellSession::run` streams `&OutLine` (stream, text, final-newline flag) instead of `&str`, and `ScriptRun` carries `status`, `errored`, `lines` and `cwd` (D60, D61). Nothing used the P0 signatures.
9. **The prologue is block 0.** `Session::start` waits for its sentinel and refuses a prologue that printed anything or ended otherwise, so a broken shell fails at the start, not at the first block.
10. **The wrapper uses `builtin printf`** in bash, so a plan's own `printf` function cannot break the sentinel. In pwsh 7.0 and 7.1 `$PSStyle` does not exist: the prologue sets it only when present, and the cwd is hex-encoded with `[BitConverter]`, which those versions have.
11. **The matcher reports every mismatch at once** (the exit and each unmatched line). An exit-only `Unknown` expectation is treated like none for `errored`.
12. **Classifier choices beyond the rulings.** `& { … }` is Destructive (only a literal bare name is classified as itself); `ForEach-Object` with a member name (`% Delete`) is Destructive; `New-Item -ItemType SymbolicLink|Junction|HardLink` is Destructive; an environment prefix on a bash command is Destructive; `find -delete/-exec/…` Destructive, `sed -i` and `sort -o` Modify, `rm -r` Destructive; DevReady `test` and `testfailed` are Build and `killport` Destructive, although their agent mode is `safe`; `replace` with neither `-DryRun` nor `-All` is Destructive; a cargo alias such as `cargo xtask` is an unknown subcommand, so Destructive.
13. **Policy choices beyond the rulings.** Without `--auto`, Read scripts ask too (D13 lists only Build and Modify; the roadmap's `--auto read,build` implies it); an Apply script always asks, as `ScriptIntent::Apply` says; `Generated` is `Auto` (a check).
14. **Table tests stay compact.** The classifier, matcher and decision rows are one line each under `#[rustfmt::skip]`, and the classifier's tables are `#[rustfmt::skip]` constants, so that rustfmt does not put every element on a line of its own.
15. **Snapshots.** `dry_run__dry_run_assay_style.snap` and `dry_run__dry_run_create_exists.snap` change in P4-T6: each of their six scripts and commits gains `; would ask (Build)` or `; would ask (Modify)` after `not run`. No other snapshot changes; neither carries an `assertion_line`.
16. **Local corpus.** `planrun show` and `planrun check` on all six local plans print exactly what they printed at `dbfbd36` after every task: P4 changes nothing in planrun-parse or the specs. The ignored `local_classes` test reports, in total, 12 Read, 35 Build, 31 Modify and 33 Destructive scripts and commits; the reasons for Destructive are, by count, a nested `pwsh` (35), `cargo xtask` (14), static .NET calls (10), 24 other unknown commands (other shells among them), and `git restore`, a call operator on a variable and a redirection through a variable (1 each).
17. **Shell test timing.** The 15 shell tests start one session each (pwsh about 0.5 s, Git Bash less) and take about 14 s; the timeout test waits 5 s per shell.

## Tasks

### P4-T1: decisions D60-D70, unsafe denied in the workspace and forbidden outside planrun-exec

**Goal:** The decisions first, so the code can cite them: D60–D70 at the end of docs/decisions.md, and amended-by lines in D9 (no crate for the process tree, and the existing D56), D11, D12, D13 and D24. The workspace lint `unsafe_code` becomes `deny`, and planrun-model, planrun-parse and planrun-cli forbid `unsafe` at their roots, so that only planrun-exec could allow it, and P4-T2 allows it in one file. `clippy::undocumented_unsafe_blocks` is denied workspace-wide.

**Files**

- Modify `Cargo.toml`
- Modify `crates/planrun-cli/src/main.rs`
- Modify `crates/planrun-model/src/lib.rs`
- Modify `crates/planrun-parse/src/lib.rs`
- Modify `docs/decisions.md`

Over 300 lines, so they come as exact diffs: `docs/decisions.md`.

**Step 1 — write the files**

File `Cargo.toml` (full contents):

```toml
[workspace]
resolver = "3"
members = ["crates/*"]
# The fixture repos are inputs of the tests, not workspace members.
exclude = ["tests/fixtures"]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
publish = false

[workspace.dependencies]
planrun-model = { path = "crates/planrun-model" }
planrun-parse = { path = "crates/planrun-parse" }
planrun-exec = { path = "crates/planrun-exec" }

# Each dependency is justified in docs/decisions.md (D9). Add none without a line there.
annotate-snippets = "0.12"
clap = { version = "4.6", features = ["derive"] }
dialoguer = { version = "0.12", default-features = false }
insta = { version = "1.49", features = ["yaml"] }
pulldown-cmark = { version = "0.13", default-features = false }
regex = "1.13"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha1_smol = "1.0"
sha2 = "0.11"
similar = "3.2"
thiserror = "2.0"
toml = "1.2"

[workspace.lints.rust]
unsafe_code = "deny"
missing_docs = "warn"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
unwrap_used = "warn"
expect_used = "warn"
undocumented_unsafe_blocks = "deny"
```


File `crates/planrun-cli/src/main.rs` (full contents):

```rust
//! The `planrun` binary (docs/architecture.md §7).

#![forbid(unsafe_code)]

mod dry_run;
mod render;
mod view;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use planrun_exec::preflight;
use planrun_exec::workspace::Workspace;
use planrun_parse::{Diagnostics, Parsed, Registry, parse_plan, source};

use crate::view::{Detail, Viewer};

/// Executes structured implementation plans against a git repository, task by task.
#[derive(Debug, Parser)]
#[command(name = "planrun", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse and check a plan; print diagnostics and the coverage report. Writes nothing.
    Check {
        /// The plan file.
        plan: PathBuf,
        /// Force a format id instead of detecting one.
        #[arg(long)]
        format: Option<String>,
    },
    /// Print the typed plan, or one task (`P1-T2`) or step (`P1-T2/1`, `P1-T2/intro`).
    Show {
        /// The plan file.
        plan: PathBuf,
        /// A task or task/step.
        target: Option<String>,
        /// Force a format id.
        #[arg(long)]
        format: Option<String>,
    },
    /// Execute a plan in the git repository of the current directory. Only `--dry-run` exists
    /// so far (Phase 3); the run itself is Phase 5.
    Run {
        /// The plan file.
        plan: PathBuf,
        /// Parse, check and validate against a virtual tree; change nothing, run nothing.
        #[arg(long)]
        dry_run: bool,
        /// Force a format id.
        #[arg(long)]
        format: Option<String>,
    },
}

/// Exit codes: 0 fine, 1 the plan has errors, 2 a usage or I/O problem.
const OK: u8 = 0;
const PLAN_ERRORS: u8 = 1;
const USAGE: u8 = 2;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut out = String::new();
    let code = match &cli.command {
        Command::Check { plan, format } => check(plan, format.as_deref(), &mut out),
        Command::Show {
            plan,
            target,
            format,
        } => show(plan, target.as_deref(), format.as_deref(), &mut out),
        Command::Run {
            plan,
            dry_run: true,
            format,
        } => run_dry(plan, format.as_deref(), &mut out),
        Command::Run { .. } => {
            out.push_str(
                "planrun: run without --dry-run is not implemented yet (docs/roadmap.md, P5)\n",
            );
            USAGE
        }
    };
    if code == OK {
        print!("{out}");
    } else {
        eprint!("{out}");
    }
    ExitCode::from(code)
}

/// A plan read and parsed, or the exit code when it was not.
enum Loaded {
    Parsed(Box<Parsed>),
    Failed(u8),
}

fn load(path: &Path, format: Option<&str>, out: &mut String) -> Loaded {
    let name = path.display().to_string();
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            out.push_str(&format!("planrun: {name}: {e}\n"));
            return Loaded::Failed(USAGE);
        }
    };
    let registry = match Registry::with_builtins() {
        Ok(registry) => registry,
        Err(e) => {
            out.push_str(&format!("planrun: the built-in formats: {e}\n"));
            return Loaded::Failed(USAGE);
        }
    };
    match parse_plan(&registry, &name, &bytes, format) {
        Ok(parsed) => Loaded::Parsed(Box::new(parsed)),
        Err(diagnostics) => {
            failed(&name, &bytes, &diagnostics, out);
            Loaded::Failed(PLAN_ERRORS)
        }
    }
}

/// Renders the diagnostics of a plan that did not parse, against its normalized text.
fn failed(name: &str, bytes: &[u8], diagnostics: &Diagnostics, out: &mut String) {
    let text = source::load(name, bytes)
        .map(|s| s.text().to_owned())
        .unwrap_or_default();
    render::diagnostics(name, &text, &diagnostics.0, out);
    let errors = diagnostics.error_codes().len();
    let plural = if errors == 1 { "" } else { "s" };
    out.push_str(&format!("planrun: {name}: {errors} error{plural}\n"));
}

fn check(path: &Path, format: Option<&str>, out: &mut String) -> u8 {
    let parsed = match load(path, format, out) {
        Loaded::Parsed(parsed) => parsed,
        Loaded::Failed(code) => return code,
    };
    let plan = &parsed.plan;
    let name = plan.source.name.to_string();
    render::diagnostics(&name, plan.source.text(), &parsed.diagnostics.0, out);
    render::coverage(plan, &parsed.coverage, out);
    out.push_str(&format!(
        "planrun: {name}: ok ({}, {} tasks)\n",
        plan.format.0,
        plan.tasks.len()
    ));
    OK
}

fn show(path: &Path, target: Option<&str>, format: Option<&str>, out: &mut String) -> u8 {
    let parsed = match load(path, format, out) {
        Loaded::Parsed(parsed) => parsed,
        Loaded::Failed(code) => return code,
    };
    let viewer = Viewer::new(&parsed.plan);
    let Some(target) = target else {
        view::plan_text(&viewer.plan(), out);
        return OK;
    };
    let (task_id, step) = match target.split_once('/') {
        Some((task, step)) => (task, Some(step)),
        None => (target, None),
    };
    let Some(task) = parsed.plan.tasks.iter().find(|t| &*t.id.raw == task_id) else {
        out.push_str(&format!("planrun: the plan has no task `{task_id}`\n"));
        return USAGE;
    };
    let Some(step) = step else {
        view::task_text(&viewer.task(task), Detail::Summary, out);
        return OK;
    };
    let found = task.steps.iter().find(|s| match s.number {
        Some(n) => n.to_string() == step,
        None => step == "intro",
    });
    let Some(found) = found else {
        out.push_str(&format!("planrun: task `{task_id}` has no step `{step}`\n"));
        return USAGE;
    };
    view::step_text(&viewer.step(found), Detail::Full, out);
    OK
}

/// `run --dry-run`: parse and check, then the dry run in the repository of the current
/// directory. Exit 0 when nothing would fail, 1 when something would, 2 when git or the
/// repository cannot be used.
fn run_dry(path: &Path, format: Option<&str>, out: &mut String) -> u8 {
    let parsed = match load(path, format, out) {
        Loaded::Parsed(parsed) => parsed,
        Loaded::Failed(code) => return code,
    };
    let plan = &parsed.plan;
    let name = plan.source.name.to_string();
    render::diagnostics(&name, plan.source.text(), &parsed.diagnostics.0, out);
    let here = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            out.push_str(&format!("planrun: the current directory: {e}\n"));
            return USAGE;
        }
    };
    let report = Workspace::open(&here, Vec::new()).and_then(|ws| preflight::dry_run(&ws, plan));
    let report = match report {
        Ok(report) => report,
        Err(e) => {
            out.push_str(&format!("planrun: {e}\n"));
            return USAGE;
        }
    };
    dry_run::report(plan, &report, out);
    if report.has_errors() {
        let errors = report.findings().filter(|f| f.is_error()).count();
        out.push_str(&format!(
            "planrun: {name}: the dry run found {errors} error(s)\n"
        ));
        return PLAN_ERRORS;
    }
    out.push_str(&format!("planrun: {name}: dry run ok\n"));
    OK
}
```


File `crates/planrun-model/src/lib.rs` (full contents):

```rust
//! The typed plan IR (docs/architecture.md §3).
//!
//! Parsing produces a [`Plan`]; execution consumes it. Neither side depends on the other: this
//! crate is the contract between them. Text is never copied out of the plan: every piece of text
//! is a [`TextRef`] into the normalized plan source held by [`PlanSource`].
//!
//! The action set ([`ActionKind`]) is closed on purpose (decision D4): plan formats vary,
//! actions don't. A new format maps its own wording onto these actions.

#![forbid(unsafe_code)]

mod action;
mod expect;
mod ids;
mod path;
mod pattern;
mod plan;
mod source;
mod span;

pub use action::{Action, ActionKind, ManualReason, Script, ScriptIntent, Shell, WriteMode};
pub use expect::{ExitExpect, Expectation};
pub use ids::{AbbrevBlobId, BlobIdError, FormatId, RecognizerId, TaskId};
pub use path::{PathOrigin, RepoPath, RepoPathError};
pub use pattern::{ManifestPath, PathPattern, PatternError};
pub use plan::{ChangeKind, LineRange, Manifest, ManifestEntry, Plan, PlanMeta, Step, Task};
pub use source::PlanSource;
pub use span::{LineCol, Span, TextRef};
```


File `crates/planrun-parse/src/lib.rs` (full contents):

```rust
//! Markdown plan → typed [`planrun_model::Plan`] (docs/architecture.md §4–§5).
//!
//! Pipeline: read and normalize ([`source`]) → block stream ([`document`]) → pick a format
//! ([`registry`]) → outline: tasks, manifests, steps ([`format`], [`outline`]) → recognizers turn
//! each step's blocks into actions ([`recognize`], [`recognizer`], the kinds in `kinds`) →
//! static checks ([`check`]).
//!
//! This crate never touches the repository and never runs anything (decision D2). Checks that
//! need the repository (preimage blob ids against the files on disk) live in planrun-exec.
//!
//! Extension points, in the order a new plan style should try them (docs/extending.md):
//! 1. a format spec in TOML that reuses the built-in recognizer kinds ([`spec`]);
//! 2. a new [`recognizer::RecognizerFactory`] for a new idiom;
//! 3. a hand-written [`format::Format`] when the outline is not heading-shaped.

#![forbid(unsafe_code)]

pub mod check;
pub mod diagnostic;
pub mod document;
pub mod format;
mod kinds;
pub mod manifest;
pub mod outline;
pub mod patch;
pub mod recognize;
pub mod recognizer;
pub mod registry;
pub mod source;
pub mod spec;

pub use diagnostic::{Diagnostic, Diagnostics, Severity};
pub use recognize::TaskCoverage;
pub use registry::Registry;

use planrun_model::Plan;

use crate::document::Document;
use crate::format::{FormatSettings, Outline};

/// A plan that parsed and checked without errors, with its warnings and notes and its coverage
/// report.
#[derive(Debug, Clone)]
pub struct Parsed {
    /// The plan.
    pub plan: Plan,
    /// Warnings and notes, in source order.
    pub diagnostics: Diagnostics,
    /// Per task, where its blocks went.
    pub coverage: Vec<TaskCoverage>,
}

/// Parses and checks `bytes` as a plan named `name`, with the formats in `registry`.
///
/// `format` forces a format by id; otherwise the one with the highest detection score wins, and
/// a tie or a score below the registry's threshold is an error (never a guess).
///
/// A plan is returned only when there is no error (`Never execute a partially parsed plan`,
/// decision D3); otherwise every diagnostic comes back, in source order. Check rules run only
/// when the outline and recognition had no errors.
pub fn parse_plan(
    registry: &Registry,
    name: &str,
    bytes: &[u8],
    format: Option<&str>,
) -> Result<Parsed, Diagnostics> {
    let source = source::load(name, bytes).map_err(Diagnostics::single)?;
    let doc = Document::build(source)?;
    let format = registry.select(&doc, format).map_err(Diagnostics::single)?;
    let outline = format.outline(&doc)?;
    let (tasks, coverage) = recognize::recognize(format.as_ref(), &doc, &outline)?;
    let mut found = outside_notes(&doc, format.settings(), &outline);
    let plan = Plan {
        source: doc.source,
        format: format.id().clone(),
        meta: outline.meta,
        tasks,
    };
    let mut checked = Vec::new();
    for rule in check::default_rules() {
        rule.check(&plan, &mut checked);
    }
    let mut checked = format.settings().apply_checks(checked);
    check::drop_reported_write_modes(&mut checked);
    found.extend(checked);
    let mut diagnostics = Diagnostics(found);
    diagnostics.sort();
    if diagnostics.has_errors() {
        return Err(diagnostics);
    }
    Ok(Parsed {
        plan,
        diagnostics,
        coverage,
    })
}

/// Manual markers outside the tasks: before task 1 (read them first) and after the tasks (not
/// run).
fn outside_notes(doc: &Document, settings: &FormatSettings, outline: &Outline) -> Vec<Diagnostic> {
    let before = outline.meta.preamble.iter().map(|text| {
        Diagnostic::note(
            "manual-outside-tasks",
            Some(text.span()),
            "read this before task 1: it is an instruction for you",
        )
    });
    let after = outline
        .after_tasks
        .map(|range| outline::marked(doc, settings, range))
        .unwrap_or_default();
    let after = after.into_iter().map(|text| {
        Diagnostic::note(
            "manual-outside-tasks",
            Some(text.span()),
            "after the tasks: planrun does not run this",
        )
    });
    before.chain(after).collect()
}
```


File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p4-t1-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index dc2bc03..023f31b 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -71,7 +71,9 @@ Each dependency is justified here; add none without a line.
 | `sha2` (Phase 1) | the plan fingerprint (SHA-256) |
 
 Rejected: `rusqlite` (D18), `ratatui` for now (D15), `tracing` for now (the journal is the record),
-`similar` as a patch engine (D20).
+`similar` as a patch engine (D20), `windows-sys` and `libc` for the process tree (D64).
+
+Amended by D56 (no `sha1_smol` in planrun-exec) and D64 (the hand-declared process-tree calls).
 
 ## D10 Line endings of written files
 
@@ -86,17 +88,23 @@ Amended by D54 (attributes, files without or with mixed line endings).
 `Expected: all pass` is mapped to an exit expectation, checked, and then shown for confirmation.
 Only fenced summary lines and exit codes pass automatically.
 
+Amended by D66 (the matching rules, what else asks).
+
 ## D12 Expected failures are skipped by default
 
 A verification expected to fail (TDD "verify it fails", `Expected: build error`) is recorded in
 the IR and skipped; `--run-expected-failures` runs it. Rens sees these failures in the IDE.
 
+Amended by D67 (Verify scripts only).
+
 ## D13 Policy classes
 
 Read < Build < Modify < Destructive; a script takes the highest class of its commands; unknown
 commands are Destructive. Defaults: file actions automatic; Build/Modify ask; Destructive needs a
 typed `yes`; Manual always waits.
 
+Amended by D68 (the classifier, a built-in allowlist) and D69 (`--auto`, Read asks too).
+
 ## D14 Resume checks the disk, never the journal alone
 
 An action recorded `Running` or `Done` is re-examined: postimages present → done; preimages
@@ -154,6 +162,8 @@ Amended by D52 (the virtual tree, the writer) and D53 (prepare, then commit).
 
 Variables and the working directory persist across blocks (`$gate`, `$P2`), as in Rens' terminal.
 
+Amended by D60 (the protocol, and how a session differs from the terminal).
+
 ## D25 Maintainability/extensibility ranks second
 
 Supersedes D1's order; the rest of D1 (not an agent, Rust) stands.
@@ -594,3 +604,267 @@ Amends D49. A `before-after` lead without a path may capture the word before "ed
 that file-name stem, the file name without its last extension (`README` of `README.md`),
 compared exactly; otherwise the series does not continue, and with no other agreeing path the
 edit is `Manual(ProseEdit)`. The spec load refuses `stem` on a lead that captures `path`.
+
+Decisions D60 onwards were agreed on 2026-11-25 at the start of P4, from the review of the
+shell, expectation and policy design against the IR and the six local plans.
+
+## D60 Shell sessions and their protocol
+
+Amends D24; corrects architecture.md §6.3.
+
+- **One process per session**, started at the repository root with `-NoProfile`
+  (`--noprofile --norc`), detached from planrun's terminal (D64). Each block is written to
+  `b<n>.tmp` with `create_new` in a run directory and renamed to `b<n>.ps1` or `b<n>.sh`, so
+  nothing reads half a block. The run directory is `planrun-shell-<random>` in the OS temp
+  directory, created with create-dir semantics (it must not exist), mode 0700 on Unix, and
+  removed when the session is dropped. Its path reaches the session in the environment
+  variable `PLANRUN_SESSION_DIR`, which the prologue copies and removes: no path ever travels
+  over a pipe, so a non-ASCII temp directory works. Block bodies are written unchanged (D48).
+- **Blocks get no stdin.** A native command started by pwsh inherits pwsh's own stdin: with
+  the protocol on stdin, `git hash-object --stdin` in a block read the protocol and hung. So
+  pwsh gets a null stdin, and its `-Command` script is the prologue and a loop at the top level
+  that waits for the next block file (a `FileSystemWatcher`, re-checked every 250 ms) and
+  dot-sources it inside `do { try { … } catch { … } } while ($false)`; the `do` keeps a block's
+  `break` or `continue` inside it. bash keeps its blocks on stdin, one top-level line each:
+  `source "<dir>/b<n>.sh" </dev/null`, so a `cat` in a block reads nothing. No function wraps
+  `source`: inside a function a block's `declare` would make locals, `$1` would be the temp
+  path, and `local` would succeed where the terminal refuses it.
+- **The sentinel.** A 128-bit secret per session plus the block's number:
+  `__planrun_<secret>_<n>__`, printed on stdout with four fields (`ok`|`threw`|`exit`, the
+  status, `errored` as 0 or 1, the working directory as hex of its UTF-8 bytes) and alone on
+  stderr. A block ends when both have arrived, so late stderr never leaks into the next block.
+  Text before the sentinel on its line is the block's last output line, marked as having no
+  final newline. A sentinel for another block, or anything after the defined fields, is
+  `Broken`: the session is dead. Block 0 is the prologue's own sentinel; a prologue that prints
+  anything or ends otherwise fails the start.
+- **Wrapper names** are `$global:__planrun_*` in pwsh, reset before every block, and
+  `__planrun_*` in bash; a plan's `$t`, `$s` or `n` is never touched.
+- **Output** is kept per line with its stream and a final-newline flag. Order within a stream
+  is exact; order across the two streams is arrival order, best effort.
+- **The working directory** after each block comes with the sentinel (pwsh
+  `$PWD.ProviderPath`; bash `$PWD`, on Windows `pwd -W`, because there `$PWD` is an MSYS path
+  and `/tmp` is a mount). P5 can warn when it leaves the repository; the deferred candidate
+  (D70) needs it too.
+- **pwsh prologue:** `$PSStyle.OutputRendering = 'PlainText'`, `$ProgressPreference =
+  'SilentlyContinue'`, `$ErrorActionPreference = 'Continue'`,
+  `$PSNativeCommandUseErrorActionPreference = $false`, UTF-8 without BOM for
+  `[Console]::OutputEncoding` and `$OutputEncoding`; on Windows `-ExecutionPolicy Bypass` for
+  the session process, because a dot-sourced file is a script to the execution policy where a
+  line typed in the terminal is not. A block that changes any of these changes later blocks,
+  as in the terminal.
+
+Stated differences from Rens' terminal: `$LASTEXITCODE` is reset to 0 before each block (in
+the terminal it carries over between commands); a terminating error ends the block (`try`); in
+pwsh `$PSScriptRoot` and `$MyInvocation` name the block's temp file; pwsh `exit n` ends only the
+block (D61); `GIT_TERMINAL_PROMPT=0` (D62).
+
+## D61 How a block ends
+
+`RunStatus` is `Exited(code)`, `Threw` (pwsh), `SessionEnded(Option<code>)`, `TimedOut` or
+`Broken(reason)`.
+
+- **pwsh:** `Threw` when a terminating error reached the wrapper's `catch` (a parse error of the
+  block included); otherwise `$LASTEXITCODE`, which is 0 when no native command ran. `exit n`
+  in a dot-sourced file ends only the block: `Exited(n)`, and the session's variables survive.
+- **bash:** `$?` of the `source`. A syntax error in a block ends the block with `Exited(2)`; the
+  session survives. `exit n` ends the session: the EXIT trap reports `SessionEnded(Some(n))`, and
+  so does `set -e` followed by a failing command. EOF on both streams without a sentinel (a
+  block replaced the trap, or `exec`ed) is `SessionEnded(None)`. Nothing restarts a session
+  silently; a dead session refuses the next block, and P5 stops.
+- **`errored`** (pwsh only; always false in bash): `$Error[0]` is not the same object before
+  and after the block (`[object]::ReferenceEquals`). A `$Error.Count` delta stays at 256 once
+  `$Error` is full. It never changes the status. Measured in pwsh 7.6:
+
+  | Construct | `errored` |
+  |---|---|
+  | `-ErrorAction SilentlyContinue` | true |
+  | `Write-Error`, a cmdlet error | true |
+  | `-ErrorAction Ignore` | false |
+  | native `2>$null`, native `2>&1` | false |
+  | native stderr, not redirected | false |
+
+  In the six local plans at most 6 of 222 pwsh fences use a silencing probe, all
+  `SilentlyContinue`: accepted as noise.
+
+## D62 The session environment
+
+A session gets planrun's own process environment minus `BASH_ENV` and `ENV` (bash reads
+`BASH_ENV` even with `--norc`) and minus `GIT_LITERAL_PATHSPECS`, `GIT_INDEX_FILE`,
+`GIT_OBJECT_DIRECTORY` and `GIT_ALTERNATE_OBJECT_DIRECTORIES`: the git wrapper's and the virtual
+tree's variables never reach a plan's commands (a literal pathspec would change what `git add
+*.rs` means). It gets `GIT_TERMINAL_PROMPT=0`, so git fails instead of prompting: a stated
+difference from the terminal. The tests add the fixture's git isolation (`Fixture::env()`), and
+a block's `git config --show-origin --list` shows only the copy's own `.git/config`.
+
+## D63 Which shells run
+
+- **pwsh** is `pwsh` from `PATH`, accepted when a probe reports
+  `$PSVersionTable.PSVersion.Major` of 7 or more. Windows PowerShell 5.1 (`powershell.exe`) is
+  never used.
+- **bash on Windows** is Git for Windows' own: `<root>\usr\bin\bash.exe`, where `<root>` is the
+  first of `git --exec-path` and its parents, at most four levels up, that has one (no layout
+  below the root is assumed: `mingw64`, `clangarm64`). A probe runs `PATH=/usr/bin:$PATH
+  uname -s`, which must report `MINGW…` or `MSYS…`. bash is never taken from `PATH`, where
+  WSL's `System32\bash.exe` can come first, and a failed probe never falls back to it.
+  `bin\bash.exe` (the launcher) is not used: it costs a process and sets its own environment.
+- **`/usr/bin` is prepended** to the session's `PATH` on Windows, as `/etc/profile` would: a
+  `--noprofile` Git Bash without Git's `usr\bin` on the Windows `PATH` (the default install)
+  finds no `uname`, `cat`, `ls` or `grep`. Only `/usr/bin`, not `/mingw64/bin`: that directory
+  depends on the install's layout, and without it `git` in a bash block resolves through the
+  Windows `PATH`, the same git planrun itself uses.
+- **bash elsewhere** is `bash` from `PATH`, checked with `$BASH_VERSION`.
+- **Tests** need both shells when `PLANRUN_REQUIRE_SHELLS` is set; otherwise a missing shell
+  skips its tests with a message on stderr (a skipped test counts as passed).
+
+## D64 The process tree, and planrun's only unsafe code
+
+Amends D9 (no crate).
+
+- **Windows:** a job object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, read back with
+  `QueryInformationJobObject` and refused when the flag did not stick. The session is assigned
+  right after its spawn and before planrun writes anything to it; a shell starts no child before
+  it has a block, so no descendant escapes between the two. When assignment fails (planrun runs
+  inside a job that refuses nesting), the session fails to start: there is no session without
+  the tree kill. `TerminateJobObject` kills the tree on timeout, and closing the job on drop
+  kills what is left, also when planrun crashes. The session is spawned with
+  `CREATE_NO_WINDOW`, so it has a console of its own and its `OutputEncoding` cannot change
+  the code page of Rens' terminal (tested with `chcp`). Not `taskkill`: it misses descendants
+  whose parent has exited, does nothing when planrun crashes, and is found through `PATH`.
+- **Unix:** `setsid` in `pre_exec`, so a block that reads `/dev/tty` fails instead of stopping
+  a background process group forever; `kill(-pgid, SIGKILL)`, and only while the session
+  process has not been waited on, so the group id cannot have been reused.
+- **Drop** kills the tree and never waits for the reader threads: a build server (MSBuild
+  nodes, VBCSCompiler) inherits the pipes and keeps them open.
+- **`unsafe`.** The workspace lint is `unsafe_code = "deny"`; planrun-model, planrun-parse and
+  planrun-cli say `#![forbid(unsafe_code)]` at their roots, so the relaxation reaches
+  planrun-exec only, and there only `crates/planrun-exec/src/shell/kill.rs` says
+  `#![allow(unsafe_code)]`. That module declares the five kernel32 functions and `setsid`/`kill`
+  by hand, owns every `unsafe` call, and exposes safe operations (`Tree::spawn`, `Tree::kill`,
+  `Tree::wait`, `detach`). `clippy::undocumented_unsafe_blocks` is denied: every unsafe block
+  has a SAFETY comment. The hand-written `JOBOBJECT_EXTENDED_LIMIT_INFORMATION` is tested to be
+  144 bytes on 64-bit targets, and the read-back flag is tested, so a wrong layout fails a test.
+  `windows-sys` would bring hundreds of generated declarations for five functions.
+
+## D65 No default timeout yet
+
+A block may run as long as it runs: cargo, dotnet and npm runs are long, and Rens' terminal has
+no timeout. `SessionConfig::timeout` exists; the tests use 60 seconds (5 for the timeout test,
+which checks that a grandchild dies with the tree, on both platforms). Whether the runner has a
+default is a P5 decision.
+
+## D66 Matching a run against its expectation
+
+Amends D11; corrects architecture.md §6.4.
+
+- **Exit:** `Success` is `Exited(0)`; `Failure` is a non-zero `Exited` or `Threw`; `Code(n)` is
+  exactly `Exited(n)`; `Unknown` is not checked. `SessionEnded`, `TimedOut` and `Broken` are a
+  mismatch whatever was expected, also for a script with no expectation.
+- **Summary lines:** each non-blank expected line must match an output line (either stream)
+  after the one the line before it matched; the earliest match is taken, which finds a
+  subsequence whenever one exists. Only `…` is a wildcard (ASCII `...` is three dots; none of
+  the 26 expected text fences of the local plans has one): it matches any run of characters,
+  the empty one included; the first literal segment is anchored at the start of the line and
+  the last at its end. Normalization, and nothing else: output is split on LF; trailing
+  whitespace is stripped from expected and output lines (which removes a CR); ANSI CSI and OSC
+  sequences are stripped from output lines. Leading whitespace counts.
+- **What asks:** prose (D11) and comment expectations (D35, D48) always ask, labelled apart; an
+  exit the plan does not state (no expectation, or `Unknown`) asks when it is not 0 or the block
+  threw; `errored` asks when nothing but an exit `Success` or `Unknown` was expected (a
+  `Copy-Item` that failed without terminating exits 0). Every confirmation shows `errored`.
+  Otherwise the run passes.
+- The verdict is `Pass`, `Mismatch` (the exit, and each unmatched line with its line number) or
+  `Confirm` (the reasons and `errored`). Skipping is not a verdict: it is decided before the run
+  (D67).
+
+## D67 Only Verify scripts are skipped as expected failures
+
+Amends D12. A `RunScript` whose intent is Verify and whose exit expectation is `Failure` is
+skipped unless `--run-expected-failures` is given. A Gate, Preview or Apply that expects failure
+runs (a skipped Gate would leave `$gate` unset for the commit); a Commit is never skipped;
+`Code(n)` is never an expected failure. A skip counts as "skipped by policy" for D22.
+
+## D68 The command classifier is a built-in allowlist
+
+Amends D13; corrects architecture.md §6.5. In the spirit of D31: the classes are built in, no
+format spec, `[checks]` entry or flag changes a class, and a runner has one classifier, not one
+per format. The tables are Rust constants in `crates/planrun-exec/src/policy/heads.rs`; no
+`toml` in planrun-exec.
+
+- **The splitter** (one per shell) walks a script once: statements (newline, `;`, `&&`, `||`),
+  pipeline elements (`|`), comments (`#`, pwsh `<# #>`), line continuations; it recurses into
+  `( … )`, `$( … )`, `@( … )`, `@{ … }` (hashtable values), script blocks, `$( … )` inside
+  double quotes and `@" … "@`, bash backticks, `<( … )`/`>( … )` and unquoted heredoc bodies.
+  Single quotes, `@' … '@` and quoted heredoc bodies are data; the heredoc's own command is
+  still classified. Keywords are not commands (pwsh `if`, `foreach`, `function`, … at the start
+  of a statement; bash `if`, `then`, `do`, `!`, `{`, … before a command); an expression
+  statement (`"…"`, `$x`, a number) is Read; an assignment classifies its right side; a function
+  body is classified where it is defined, and calling the function by name is unknown.
+- **What it cannot take apart with confidence makes the whole script Destructive:** an
+  unterminated quote, here-string or heredoc, an unmatched bracket, pwsh `<`, bash `case` and
+  `;;`, a command inside `${…}`.
+- **pwsh lexing:** `–`, `—` and `―` are parameter dashes; `‘ ’ ‚ ‛` single and `“ ” „` double
+  quotes; names, aliases and parameters are case-insensitive, aliases (`rm`, `ri`, `del`, `cp`,
+  `mv`, `cat`, `ls`, `cd`, `iex`, `saps`, `iwr`, `irm`, `%`, `?`, …) map to their cmdlet first;
+  a `.exe` or `.cmd` suffix is dropped; a path- or module-qualified command is Destructive.
+  Parameters match by prefix as pwsh does (`-r`, `-Rec`, `-Recurse:$true`), and splatting
+  (`@args`) on a command with a dangerous switch is Destructive. `& name` is classified as
+  `name` when `name` is a literal bare command name; `&` with anything else, and dot-sourcing a
+  file, are Destructive. A static .NET call is Destructive except on `[IO.Path]`, `[Math]`,
+  `[string]` and `[regex]`; an instance call except `Trim`, `Split`, `Replace`, `Substring`,
+  `ToString`, `Contains`, `StartsWith`, `EndsWith`, `ToLower`, `ToUpper`. `Add-Type` and
+  `New-Object` are unknown.
+- **Redirections:** `$null`, `/dev/null` and `NUL` are nothing; a literal relative path without
+  `..` is Modify; anything else (absolute, `~`, a variable, a subexpression) is Destructive.
+  Operators: pwsh `> >> 2> *> n>` (and the merges `n>&1`), bash `> >> >| &> <> >&file`;
+  `Tee-Object -FilePath` is Modify.
+- **Nested shells:** `pwsh -File`, `pwsh -c`, `bash <file>`, `bash -c`, `cmd /c`, `source`,
+  `eval`, `exec` are unknown, so Destructive; a nested shell's string is never parsed. An
+  environment prefix on a bash command (`FOO=1 git status`) is Destructive.
+- **git:** options before the subcommand are parsed: `-C <dir>` keeps the subcommand's class;
+  `--git-dir`, `--work-tree`, `--exec-path` and any other unknown option are Destructive; `-c
+  k=v` is allowed only for `color.*`, `core.quotepath`, `user.name`, `user.email` and
+  `advice.*` (aliases, `core.pager`, `core.fsmonitor`, `diff.external`, `core.sshCommand` run
+  programs). Unknown subcommands, aliases included, are Destructive. `checkout -b/-B/--orphan`
+  and `switch` without `-f`/`--discard-changes` are Modify; `checkout <name>` without `-b` is
+  Destructive (the name can be a path), and so are `checkout --`, `-f`, `switch -f`,
+  `--discard-changes`, `restore`, `reset --hard/--merge/--keep`, `clean`, `push`, `fetch`,
+  `pull`, `stash drop/clear`, `branch -D/-f`, `config --global/--system/--file`. `diff`, `log`
+  and `show` with `--output` are Modify, with `--ext-diff` Destructive.
+- **cargo, npm, dotnet, go:** options before the subcommand are parsed (`cargo +nightly`,
+  `--manifest-path x`); `cargo --config` is Destructive. Build may fetch dependencies into
+  caches and ignored directories, so `npm ci` and `dotnet restore` are Build; Modify changes
+  tracked files, manifests or lockfiles (`cargo fmt` without `--check`, `fix`, `clippy --fix`,
+  `add`, `update`, `remove`, `npm install`/`uninstall`/`update`, `dotnet add`, `dotnet format`
+  without `--verify-no-changes`, `go mod tidy`, `go get`); `cargo install`/`publish`/`run`, `go
+  install`/`run`, `dotnet run` and `npx` are Destructive. A cargo alias (`cargo xtask`) is an
+  unknown subcommand.
+- **DevReady** (pwsh table only; the list is from DevReady 1.19.1, `DEVREADY_VERSION`):
+  functions whose agent mode is `safe` are Read; `brief { … }` is classified by its block;
+  `replace -DryRun` is Read, `replace -All` Modify, `replace` with neither Destructive (it stops
+  to ask); `test` and `testfailed` are Build (they run the tests); `killport` is Destructive
+  although DevReady calls it safe (it kills a process); `dry-run-only` and `interactive`
+  functions are Destructive.
+- The local plans' scripts are classified by an ignored test in planrun-cli
+  (`local_classes`), whose output stays in the terminal (D16).
+
+## D69 `--auto`, and what P4 decides for P5
+
+Amends D13. `--auto` takes `read`, `build` and `modify`; `destructive` is a usage error. Without
+`--auto` every script asks, Read ones included. A Destructive script always needs a typed `yes`,
+a Manual always waits, a file action outside the manifest's scope always asks (D50), and an
+Apply script always asks (it follows its preview), whatever `--auto` says; file actions within
+scope run automatically. P4 computes the decision (`Policy::decide`: `Auto`, `Ask(class)`,
+`AskUnscoped`, `TypedYes`, `Wait`, `Show`, `Skip`); P5's runner prompts. The dry run shows each
+script's class and decision under the default policy ("would ask (Build)", "typed yes
+(Destructive)", "skipped: expected failure").
+
+## D70 A script working directory from a run lead is deferred
+
+The P2b candidate (``(run from `$REPO`):`` gives a script its directory) is not built in P4.
+The name in the lead is a session value that planrun cannot resolve statically; evaluating it
+would run prose as code; and the directory can be outside the repository. (A
+`Push-Location`/`Pop-Location` around the block would keep D24 intact, so D24 is no reason.)
+Such fences stay `Manual(HumanCheck)` (`manual-prose#run-from`). What it would take later:
+`Script.workdir` with `skip_serializing_if`, so no IR snapshot changes; the run-from lead
+becoming a script lead; and the directory resolved from the session (D60 reports the working
+directory after every block) and checked against the repository root before the block runs.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p4-t1-decisions-md.patch"
Remove-Item "$env:TEMP\p4-t1-decisions-md.patch"
```


**Step 2 — verify**

Targeted: the model's tests, under the new lints:

```powershell
cargo test -p planrun-model
```


Expected (exit code 0; summary lines):

```text
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```


Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```


Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```


**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- Cargo.toml crates/planrun-cli/src/main.rs crates/planrun-model/src/lib.rs crates/planrun-parse/src/lib.rs docs/decisions.md
  $verified = [ordered]@{
      'Cargo.toml' = '875e66bca26f0db16de0252206ab32ac4d078797'
      'crates/planrun-cli/src/main.rs' = 'a8b84cb32e958b0556d4ff0f1ea7c1284abea5b1'
      'crates/planrun-model/src/lib.rs' = '876d5bd983b97d5f0784f6c9d5e46f835ce18c4b'
      'crates/planrun-parse/src/lib.rs' = '84b5bf541d6000d703369e51764c4bd51616d2b4'
      'docs/decisions.md' = '023f31b2fc92647391d60e038cbcda58d6793a65'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P4-T1: decisions D60-D70, unsafe denied in the workspace and forbidden outside planrun-exec" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```


Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P4-T2: persistent pwsh and bash sessions: the sentinel protocol, the process tree, the session environment

**Goal:** Persistent sessions. `Tree` (in `shell/kill.rs`, the only module that allows `unsafe`) spawns a shell detached from the terminal, in a job object on Windows or a session of its own on Unix, and kills the whole tree. `Session` runs blocks from files in a run directory under the OS temp directory, with no stdin for the block, and reads the two-stream sentinel with the per-block number; pwsh runs a loop over block files, bash one top-level `source` line per block. The tests run both shells on fixture copies: state across blocks, the wrapper's names, exit and throw, `errored`, output order, stdin, the environment, a forged sentinel, the timeout's tree kill, the code page, a non-ASCII run directory.

**Files**

- Modify `crates/planrun-exec/src/lib.rs`
- Create `crates/planrun-exec/src/shell/bash.rs`
- Create `crates/planrun-exec/src/shell/kill.rs`
- Create `crates/planrun-exec/src/shell/mod.rs`
- Create `crates/planrun-exec/src/shell/pwsh.rs`
- Create `crates/planrun-exec/src/shell/session.rs`
- Create `crates/planrun-exec/tests/shell.rs`

**Step 1 — write the files**

File `crates/planrun-exec/src/lib.rs` (full contents):

```rust
//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Modules (docs/roadmap.md; P3 built the first eight, P4 the next three):
//!
//! - [`workspace`]: the repository root, [`RepoPath`](planrun_model::RepoPath) resolution with
//!   the link and case check, and the one writer (temp file, second check, rename; D52).
//! - [`eol`]: the line endings of written files (D10, D54).
//! - [`git`]: a thin wrapper over the `git` binary: `apply` with the patch on stdin,
//!   `hash-object`, `status --porcelain` without optional locks, `rev-parse`, `cat-file`,
//!   `check-attr`, `update-index`, `ls-files`; [`BlobId`](git::BlobId).
//! - [`tree`]: the virtual tree, a temporary index and object directory (D52).
//! - [`handler`]: one [`ActionHandler`] per action kind, which simulates it on a virtual tree;
//!   the [`HandlerSet`] is the execution-side plugin point.
//! - [`transaction`]: a step's file actions as prepare, then commit, with rollback (D53).
//! - [`preflight`]: task preflight and the dry run over one virtual tree (D50, D55).
//! - [`finding`]: what preflight, a simulation and the dry run report.
//! - [`shell`]: persistent pwsh and bash sessions behind [`ShellSession`] (D60–D65).
//! - `expect`: exit-code and summary-line matching.
//! - `policy`: command classes and the [`CommandClassifier`].
//! - `journal`: JSONL under `.git/planrun/`.
//! - `runner`: the state machine Task → Step → Action, talking to a [`Ui`].

pub mod eol;
pub mod finding;
pub mod git;
pub mod handler;
pub mod preflight;
pub mod shell;
pub mod transaction;
pub mod tree;
pub mod workspace;

pub use handler::{ActionHandler, HandlerSet};
pub use shell::{ScriptRun, ShellSession};

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "../tests/support/ir.rs"]
mod ir;

/// The command classes of decision D13, least to most dangerous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PolicyClass {
    /// Reads only (`git status`, `rg`).
    Read,
    /// Builds or tests (`cargo test`, `dotnet build`, `npm ci`).
    Build,
    /// Changes the repository (`git add`, `git commit`, `replace … -All`, file actions).
    Modify,
    /// Destroys or reaches outside (`Remove-Item -Recurse`, `git reset --hard`, `git push`,
    /// anything unclassified).
    Destructive,
}

/// Classifies a script: the most dangerous class of any command in it.
pub trait CommandClassifier: Send + Sync {
    /// The class of `script` run by `shell`.
    fn classify(&self, shell: planrun_model::Shell, script: &str) -> PolicyClass;
}

/// The state of an action's effects on disk, for resuming (decision D14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoneState {
    /// Every effect is present (postimages match).
    Done,
    /// No effect is present (preimages match).
    NotDone,
    /// Some are, some are not, or neither matches: the user decides.
    Unknown,
}

/// The interactive surface the runner talks to (decision D15: line prompts first).
pub trait Ui {
    /// Shows a line of progress.
    fn status(&mut self, line: &str);
}
```


File `crates/planrun-exec/src/shell/bash.rs` (full contents):

```rust
//! bash (decisions D60, D61, D63): `bash --noprofile --norc`, one top-level line per block on
//! stdin, the block `source`d from its file with stdin from `/dev/null`. On Windows, Git for
//! Windows' bash only, found from git itself, never from `PATH`.

use std::path::PathBuf;
use std::process::Command;

use super::ShellError;
use super::kill::detach;
use super::session::{DIR_VAR, Dialect};

/// The bash dialect.
pub(super) struct Bash;

fn unavailable(reason: String) -> ShellError {
    ShellError::Unavailable {
        shell: "bash",
        reason,
    }
}

/// Windows: `<root>\usr\bin\bash.exe`, where `<root>` is the first of `git --exec-path` and its
/// parents (at most four levels up) that has one; checked with `uname -s`, which must report
/// MINGW or MSYS. Elsewhere: `bash` from `PATH`, checked with `$BASH_VERSION`.
pub(super) fn locate() -> Result<PathBuf, ShellError> {
    let program = if cfg!(windows) {
        git_bash()?
    } else {
        PathBuf::from("bash")
    };
    let check = if cfg!(windows) {
        "PATH=/usr/bin:$PATH uname -s"
    } else {
        "echo \"$BASH_VERSION\""
    };
    let mut probe = Command::new(&program);
    probe.args(["--noprofile", "--norc", "-c", check]);
    detach(&mut probe);
    let out = probe
        .output()
        .map_err(|e| unavailable(format!("{} did not start: {e}", program.display())))?;
    let said = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let accepted = if cfg!(windows) {
        said.starts_with("MINGW") || said.starts_with("MSYS")
    } else {
        !said.is_empty()
    };
    if !out.status.success() || !accepted {
        return Err(unavailable(format!(
            "{} answered {said:?} to `{check}`",
            program.display()
        )));
    }
    Ok(program)
}

fn git_bash() -> Result<PathBuf, ShellError> {
    let out = Command::new("git")
        .arg("--exec-path")
        .output()
        .map_err(|e| unavailable(format!("`git --exec-path` did not run: {e}")))?;
    let exec_path = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    exec_path
        .ancestors()
        .take(5)
        .map(|dir| dir.join("usr").join("bin").join("bash.exe"))
        .find(|bash| bash.is_file())
        .ok_or_else(|| {
            unavailable(format!(
                "no usr\\bin\\bash.exe at or above {}",
                exec_path.display()
            ))
        })
}

impl Dialect for Bash {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn args(&self, _marker: &str) -> Vec<String> {
        vec!["--noprofile".to_owned(), "--norc".to_owned()]
    }

    fn extension(&self) -> &'static str {
        "sh"
    }

    /// On Windows `/usr/bin` is prepended to `PATH`, as `/etc/profile` would (only that
    /// directory: `git` keeps resolving through the Windows `PATH`, as planrun's own does), and
    /// the working directory is `pwd -W`: `$PWD` is an MSYS path, and `/tmp` is a mount.
    fn prologue(&self, marker: &str) -> Option<String> {
        let (path, pwd) = if cfg!(windows) {
            ("PATH=/usr/bin:$PATH; ", "$(builtin pwd -W)")
        } else {
            ("", "$PWD")
        };
        Some(format!(
            "__planrun_dir=${DIR_VAR}; unset {DIR_VAR}; {path}\
             __planrun_hex() {{ local LC_ALL=C __planrun_i __planrun_c; __planrun_h=; \
             for ((__planrun_i = 0; __planrun_i < ${{#1}}; __planrun_i++)); do \
             builtin printf -v __planrun_c '%02x' \"'${{1:__planrun_i:1}}\"; \
             __planrun_h+=$__planrun_c; done; }}; \
             __planrun_end() {{ __planrun_hex \"{pwd}\"; \
             builtin printf '%s\\n' \"{marker}${{__planrun_n}}__ $1 $__planrun_s 0 $__planrun_h\"; \
             builtin printf '%s\\n' \"{marker}${{__planrun_n}}__\" >&2; }}; \
             trap '__planrun_s=$?; __planrun_end exit' EXIT; \
             __planrun_n=0; __planrun_s=0; __planrun_end ok"
        ))
    }

    fn block_line(&self, n: u64) -> Option<String> {
        Some(format!(
            "__planrun_n={n}; source \"$__planrun_dir/b{n}.sh\" </dev/null; __planrun_s=$?; \
             __planrun_end ok"
        ))
    }

    /// On Windows `pwd -W` reports `C:/x`: its slashes become backslashes.
    fn cwd(&self, reported: String) -> PathBuf {
        if cfg!(windows) {
            return PathBuf::from(reported.replace('/', "\\"));
        }
        PathBuf::from(reported)
    }
}
```


File `crates/planrun-exec/src/shell/kill.rs` (full contents):

```rust
//! The process tree of a shell session (decision D64). Every `unsafe` call of planrun lives in
//! this module: the job object on Windows, `setsid` and `kill` on Unix, each declared by hand.
//! The rest of the crate sees three safe operations: [`Tree::spawn`], [`Tree::kill`] and
//! [`Tree::wait`].

#![allow(unsafe_code)]

use std::io;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};

/// The three pipes of a spawned session.
pub(crate) struct Pipes {
    /// The protocol channel (bash: one line per block); `None` when the session got none.
    pub stdin: Option<ChildStdin>,
    /// The session's stdout.
    pub stdout: ChildStdout,
    /// The session's stderr.
    pub stderr: ChildStderr,
}

/// A spawned process and every process it starts. Killing the tree kills them all; dropping it
/// kills them too, and never waits for anything but the process itself.
pub(crate) struct Tree {
    child: Child,
    reaped: Option<ExitStatus>,
    #[cfg(windows)]
    job: windows::Job,
}

impl Tree {
    /// Spawns `command` with piped stdout and stderr, stdin piped or null, detached from planrun's terminal
    /// (Windows: a console of its own; Unix: a session of its own), and puts it in a tree. When
    /// the tree cannot be made (Windows: a job that refuses nesting), the process is killed and
    /// the spawn fails: there is no session without the tree kill.
    pub(crate) fn spawn(command: &mut Command, stdin: bool) -> io::Result<(Tree, Pipes)> {
        let input = if stdin { Stdio::piped() } else { Stdio::null() };
        command
            .stdin(input)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        detach(command);
        #[cfg(windows)]
        let job = windows::Job::new()?;
        let mut child = command.spawn()?;
        #[cfg(windows)]
        if let Err(e) = job.assign(&child) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
        let input = child.stdin.take();
        let pipes = match (input, child.stdout.take(), child.stderr.take()) {
            (input, Some(stdout), Some(stderr)) if input.is_some() == stdin => Pipes {
                stdin: input,
                stdout,
                stderr,
            },
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::Error::other("the session's pipes were not created"));
            }
        };
        let tree = Tree {
            child,
            reaped: None,
            #[cfg(windows)]
            job,
        };
        Ok((tree, pipes))
    }

    /// Kills the process and all its descendants. Does nothing once the process was waited on:
    /// on Unix its process group id could then belong to someone else.
    pub(crate) fn kill(&mut self) -> io::Result<()> {
        if self.reaped.is_some() {
            return Ok(());
        }
        #[cfg(windows)]
        let killed = self.job.terminate();
        #[cfg(unix)]
        let killed = unix::kill_group(self.child.id());
        if killed.is_err() {
            let _ = self.child.kill();
        }
        killed
    }

    /// Waits for the process itself (not its descendants) and remembers its status.
    pub(crate) fn wait(&mut self) -> io::Result<ExitStatus> {
        if let Some(status) = self.reaped {
            return Ok(status);
        }
        let status = self.child.wait()?;
        self.reaped = Some(status);
        Ok(status)
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = self.kill();
        let _ = self.wait();
    }
}

/// Detaches a command from planrun's terminal: on Windows `CREATE_NO_WINDOW`, so the session
/// gets a console of its own and its code page changes stay there; on Unix `setsid`, so a read
/// from `/dev/tty` fails instead of stopping the process group.
pub(crate) fn detach(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(unix)]
    unix::setsid_on_exec(command);
}

#[cfg(unix)]
mod unix {
    use std::io;
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    const SIGKILL: i32 = 9;
    const ESRCH: i32 = 3;

    unsafe extern "C" {
        fn setsid() -> i32;
        fn kill(pid: i32, signal: i32) -> i32;
    }

    pub(super) fn setsid_on_exec(command: &mut Command) {
        // SAFETY: the closure runs in the forked child before exec. It only calls `setsid`,
        // which is async-signal-safe, and builds an `io::Error` from errno, which allocates
        // nothing; it touches no lock or memory shared with the parent.
        unsafe {
            command.pre_exec(|| {
                // SAFETY: `setsid` takes no arguments and only changes the calling process.
                if setsid() == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    /// Kills the process group led by `pid`. The caller guarantees `pid` was not waited on yet,
    /// so the group id is still this tree's (a zombie leader keeps it reserved).
    pub(super) fn kill_group(pid: u32) -> io::Result<()> {
        let Ok(pid) = i32::try_from(pid) else {
            return Err(io::Error::other("process id out of range"));
        };
        // SAFETY: `kill` takes plain integers; a negative pid names the process group, which
        // `setsid` made equal to the session's pid.
        if unsafe { kill(-pid, SIGKILL) } == 0 {
            return Ok(());
        }
        let e = io::Error::last_os_error();
        if e.raw_os_error() == Some(ESRCH) {
            return Ok(());
        }
        Err(e)
    }
}

#[cfg(windows)]
mod windows {
    use std::ffi::c_void;
    use std::io;
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;

    type Handle = *mut c_void;

    const EXTENDED_LIMIT_INFORMATION: i32 = 9;
    pub(super) const KILL_ON_JOB_CLOSE: u32 = 0x2000;

    #[repr(C)]
    #[derive(Default)]
    struct BasicLimitInformation {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: u32,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct IoCounters {
        read_operation_count: u64,
        write_operation_count: u64,
        other_operation_count: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }

    /// `JOBOBJECT_EXTENDED_LIMIT_INFORMATION`, laid out by hand; the size is tested.
    #[repr(C)]
    #[derive(Default)]
    pub(super) struct ExtendedLimitInformation {
        basic: BasicLimitInformation,
        io: IoCounters,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> Handle;
        fn SetInformationJobObject(job: Handle, class: i32, info: *const c_void, len: u32) -> i32;
        fn QueryInformationJobObject(
            job: Handle,
            class: i32,
            info: *mut c_void,
            len: u32,
            returned: *mut u32,
        ) -> i32;
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn TerminateJobObject(job: Handle, exit_code: u32) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
    }

    /// A job object that kills its processes when its last handle closes.
    pub(super) struct Job(Handle);

    // SAFETY: a job object handle is a kernel handle; any thread may use or close it.
    unsafe impl Send for Job {}

    impl Job {
        /// A new anonymous job with `KILL_ON_JOB_CLOSE`, read back to check it is set.
        pub(super) fn new() -> io::Result<Job> {
            // SAFETY: both arguments may be null (default security, no name).
            let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
            let job = Job(handle);
            let mut info = ExtendedLimitInformation::default();
            info.basic.limit_flags = KILL_ON_JOB_CLOSE;
            let len = size_of::<ExtendedLimitInformation>() as u32;
            // SAFETY: `info` is a live, correctly sized `JOBOBJECT_EXTENDED_LIMIT_INFORMATION`
            // for the class passed, and `job.0` is the handle created above.
            let set = unsafe {
                SetInformationJobObject(
                    job.0,
                    EXTENDED_LIMIT_INFORMATION,
                    (&raw const info).cast(),
                    len,
                )
            };
            if set == 0 {
                return Err(io::Error::last_os_error());
            }
            if job.limit_flags()? & KILL_ON_JOB_CLOSE == 0 {
                return Err(io::Error::other(
                    "the job object did not keep KILL_ON_JOB_CLOSE",
                ));
            }
            Ok(job)
        }

        /// The job's limit flags, as Windows reports them.
        pub(super) fn limit_flags(&self) -> io::Result<u32> {
            let mut info = ExtendedLimitInformation::default();
            let mut returned = 0u32;
            let len = size_of::<ExtendedLimitInformation>() as u32;
            // SAFETY: `info` is writable and `len` bytes long; `returned` is a valid out
            // pointer; `self.0` is a live job handle.
            let ok = unsafe {
                QueryInformationJobObject(
                    self.0,
                    EXTENDED_LIMIT_INFORMATION,
                    (&raw mut info).cast(),
                    len,
                    &raw mut returned,
                )
            };
            if ok == 0 {
                return Err(io::Error::last_os_error());
            }
            if returned != len {
                return Err(io::Error::other(
                    "the job's limit information has another size",
                ));
            }
            Ok(info.basic.limit_flags)
        }

        /// Puts `child` in the job. The session is assigned right after its spawn and before
        /// planrun writes its first line; a shell starts no child before it reads a block, so
        /// no descendant can escape between the two.
        pub(super) fn assign(&self, child: &Child) -> io::Result<()> {
            let process = child.as_raw_handle();
            // SAFETY: `self.0` is a live job handle and `process` the live handle `Child`
            // owns; the call keeps neither.
            if unsafe { AssignProcessToJobObject(self.0, process.cast()) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }

        /// Kills every process in the job.
        pub(super) fn terminate(&self) -> io::Result<()> {
            // SAFETY: `self.0` is a live job handle.
            if unsafe { TerminateJobObject(self.0, 1) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            // SAFETY: the handle was created by `new` and is closed only here, once.
            unsafe { CloseHandle(self.0) };
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[cfg(target_pointer_width = "64")]
        #[test]
        fn the_limit_information_has_windows_size() {
            assert_eq!(size_of::<ExtendedLimitInformation>(), 144);
        }

        #[test]
        fn a_new_job_kills_on_close() {
            let job = Job::new().unwrap_or_else(|e| panic!("{e}"));
            let flags = job.limit_flags().unwrap_or_else(|e| panic!("{e}"));
            assert_ne!(flags & KILL_ON_JOB_CLOSE, 0, "limit flags {flags:#x}");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::time::{Duration, Instant};

    use super::Tree;

    #[test]
    fn a_killed_tree_ends_and_kill_after_wait_does_nothing() {
        // `git cat-file --batch` waits on its stdin, which the tree keeps open.
        let mut command = Command::new("git");
        command.args(["cat-file", "--batch"]);
        let (mut tree, pipes) = Tree::spawn(&mut command, true).unwrap_or_else(|e| panic!("{e}"));
        let started = Instant::now();
        tree.kill().unwrap_or_else(|e| panic!("{e}"));
        let status = tree.wait().unwrap_or_else(|e| panic!("{e}"));
        assert!(!status.success(), "{status}");
        assert!(started.elapsed() < Duration::from_secs(10));
        tree.kill().unwrap_or_else(|e| panic!("{e}"));
        drop(pipes);
    }
}
```


File `crates/planrun-exec/src/shell/mod.rs` (full contents):

```rust
//! Persistent pwsh and bash sessions (docs/architecture.md §6.3; decisions D24, D60–D65).
//!
//! One [`Session`] is one shell process. Each block is written to a file in a run directory
//! under the OS temp directory and run by one ASCII line on the session's stdin: dot-sourced in
//! pwsh, `source`d at the top level in bash, with stdin taken away from the block. A sentinel
//! with a per-run secret and the block's number follows it on stdout and on stderr; the block
//! ends when both arrive.

mod bash;
mod kill;
mod pwsh;
mod session;

use std::io;
use std::path::PathBuf;

pub use session::{Session, SessionConfig, locate};

/// Which stream a line came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// One line of a block's output. Order within a stream is exact; order across the two streams
/// is the order of arrival, best effort.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutLine {
    /// The stream.
    pub stream: Stream,
    /// The text without its LF (a CR before it stays), UTF-8 decoded lossily.
    pub text: String,
    /// False for the block's last output on this stream when it did not end with a newline.
    pub newline: bool,
}

/// How a block ended (decision D61).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunStatus {
    /// It ran to its end. pwsh: `$LASTEXITCODE`, reset to 0 before the block (`exit n` ends only
    /// the block); bash: `$?` of the `source`.
    Exited(i32),
    /// pwsh only: a terminating error ended the block.
    Threw,
    /// The session ended during the block: bash `exit n` (with its code), or the shell went away
    /// without its sentinel (`None`). The session is dead.
    SessionEnded(Option<i32>),
    /// The block ran past the session's timeout; its process tree was killed.
    TimedOut,
    /// The protocol broke (a sentinel for another block, unexpected fields). The session is
    /// dead.
    Broken(Box<str>),
}

/// The result of running one shell block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRun {
    /// How it ended.
    pub status: RunStatus,
    /// pwsh: `$Error[0]` is another object after the block than before it (a non-terminating
    /// error was recorded, decision D61). Always false in bash. Never changes `status`.
    pub errored: bool,
    /// Its output, both streams.
    pub lines: Vec<OutLine>,
    /// The session's working directory after the block, when the sentinel reported it.
    pub cwd: Option<PathBuf>,
}

/// A long-lived shell: variables, the working directory and functions persist across blocks.
pub trait ShellSession: Send {
    /// Runs `script`, streaming each output line to `on_line` as it arrives. An error means the
    /// block was not started (its file could not be written, or the session is dead).
    fn run(&mut self, script: &str, on_line: &mut dyn FnMut(&OutLine)) -> io::Result<ScriptRun>;
}

/// Why a session could not be had.
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    /// The shell is not installed, or not the one planrun accepts (decision D63).
    #[error("{shell} is not available: {reason}")]
    Unavailable {
        /// `pwsh` or `bash`.
        shell: &'static str,
        /// What the probe found.
        reason: String,
    },
    /// The shell started but its prologue did not answer as expected.
    #[error("the {shell} session did not start: {reason}")]
    Start {
        /// `pwsh` or `bash`.
        shell: &'static str,
        /// What went wrong.
        reason: String,
    },
    /// Spawning, the run directory, or the process tree failed.
    #[error(transparent)]
    Io(#[from] io::Error),
}
```


File `crates/planrun-exec/src/shell/pwsh.rs` (full contents):

```rust
//! PowerShell 7 (decisions D60, D61, D63). A native command started by pwsh inherits pwsh's
//! own stdin, so the session gets none (`git hash-object --stdin` would read the protocol):
//! its `-Command` script is the prologue and a loop at the top level that waits for the next
//! block file in the run directory and dot-sources it inside `do`/`try`/`catch`.

use std::path::PathBuf;
use std::process::Command;

use super::ShellError;
use super::kill::detach;
use super::session::{DIR_VAR, Dialect};

/// The pwsh dialect.
pub(super) struct Pwsh;

/// `pwsh` from `PATH`, when it is PowerShell 7 or later; Windows PowerShell 5.1 is
/// `powershell.exe` and never used.
pub(super) fn locate() -> Result<PathBuf, ShellError> {
    let unavailable = |reason: String| ShellError::Unavailable {
        shell: "pwsh",
        reason,
    };
    let mut probe = Command::new("pwsh");
    probe.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$PSVersionTable.PSVersion.Major",
    ]);
    detach(&mut probe);
    let out = probe
        .output()
        .map_err(|e| unavailable(format!("`pwsh` did not start: {e}")))?;
    let major = String::from_utf8_lossy(&out.stdout).trim().parse::<u32>();
    match major {
        Ok(major) if out.status.success() && major >= 7 => Ok(PathBuf::from("pwsh")),
        Ok(major) => Err(unavailable(format!("PowerShell {major} is older than 7"))),
        Err(_) => Err(unavailable("`pwsh` did not report its version".to_owned())),
    }
}

impl Dialect for Pwsh {
    fn name(&self) -> &'static str {
        "pwsh"
    }

    fn args(&self, marker: &str) -> Vec<String> {
        let mut args = vec!["-NoProfile", "-NonInteractive", "-NoLogo"];
        if cfg!(windows) {
            args.extend(["-ExecutionPolicy", "Bypass"]);
        }
        args.push("-Command");
        let mut args: Vec<String> = args.into_iter().map(str::to_owned).collect();
        args.push(script(marker));
        args
    }

    fn extension(&self) -> &'static str {
        "ps1"
    }

    fn prologue(&self, _marker: &str) -> Option<String> {
        None
    }

    fn block_line(&self, _n: u64) -> Option<String> {
        None
    }

    fn cwd(&self, reported: String) -> PathBuf {
        PathBuf::from(reported)
    }
}

/// The session's whole `-Command` script: the prologue, the sentinel of block 0, then the loop.
/// It has no double quotes, so the command line needs no escaping beyond Windows' own.
fn script(marker: &str) -> String {
    [
        format!("$global:__planrun_dir = $env:{DIR_VAR}"),
        format!("Remove-Item Env:{DIR_VAR}"),
        "if ($PSStyle) { $PSStyle.OutputRendering = 'PlainText' }".to_owned(),
        "$global:ProgressPreference = 'SilentlyContinue'".to_owned(),
        "$global:ErrorActionPreference = 'Continue'".to_owned(),
        "$global:PSNativeCommandUseErrorActionPreference = $false".to_owned(),
        "[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)".to_owned(),
        "$global:OutputEncoding = [Text.UTF8Encoding]::new($false)".to_owned(),
        end_function(marker),
        "$global:__planrun_n = 0".to_owned(),
        "$global:__planrun_w = [IO.FileSystemWatcher]::new($global:__planrun_dir, 'b*.ps1')"
            .to_owned(),
        RESET.to_owned(),
        "__planrun_end 0".to_owned(),
        run_loop(),
    ]
    .join("; ")
}

/// Resets every wrapper variable and `$LASTEXITCODE` before a block (D61).
const RESET: &str =
    "$global:__planrun_t = 0; $global:__planrun_e = $global:Error[0]; $global:LASTEXITCODE = 0";

/// Waits for `b<n>.ps1` (renamed into place whole), runs it in the global scope, and prints its
/// sentinel. `do … while ($false)` keeps a `break` or `continue` of the block inside it.
fn run_loop() -> String {
    format!(
        "while ($true) {{ $global:__planrun_n++; \
         $global:__planrun_f = [IO.Path]::Combine($global:__planrun_dir, 'b' + $global:__planrun_n + '.ps1'); \
         while (-not [IO.File]::Exists($global:__planrun_f)) {{ \
         [void]$global:__planrun_w.WaitForChanged([IO.WatcherChangeTypes]::Renamed, 250) }}; \
         {RESET}; \
         do {{ try {{ . $global:__planrun_f }} catch {{ $global:__planrun_t = 1; \
         [Console]::Error.WriteLine(($_ | Out-String).TrimEnd()) }} }} while ($false); \
         __planrun_end $global:__planrun_n }}"
    )
}

/// `__planrun_end <n>`: the sentinel on stdout (kind, status, errored, cwd as hex of UTF-8) and
/// on stderr, each written with LF and flushed.
fn end_function(marker: &str) -> String {
    format!(
        "function global:__planrun_end([long]$n) {{ \
         $s = $global:LASTEXITCODE; if ($null -eq $s) {{ $s = 0 }}; \
         $k = if ($global:__planrun_t) {{ 'threw' }} else {{ 'ok' }}; \
         $e = if ([object]::ReferenceEquals($global:__planrun_e, $global:Error[0])) {{ 0 }} else {{ 1 }}; \
         $c = [BitConverter]::ToString([Text.Encoding]::UTF8.GetBytes($PWD.ProviderPath)).Replace('-', ''); \
         $m = '{marker}' + $n + '__'; \
         [Console]::Out.Write($m + ' ' + $k + ' ' + $s + ' ' + $e + ' ' + $c + [char]10); [Console]::Out.Flush(); \
         [Console]::Error.Write($m + [char]10); [Console]::Error.Flush() }}"
    )
}
```


File `crates/planrun-exec/src/shell/session.rs` (full contents):

```rust
//! The session protocol (decision D60): the run directory, the spawn with its environment
//! (D62), the reader threads, and the two-stream sentinel with the per-block number.

use std::collections::hash_map::RandomState;
use std::ffi::OsString;
use std::fs::{DirBuilder, OpenOptions};
use std::hash::{BuildHasher, Hasher};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{ChildStdin, Command};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use planrun_model::Shell;

use super::kill::Tree;
use super::{OutLine, RunStatus, ScriptRun, ShellError, ShellSession, Stream, bash, pwsh};

/// The environment variable that hands the run directory to the prologue, which removes it.
pub(super) const DIR_VAR: &str = "PLANRUN_SESSION_DIR";

/// Variables a session never inherits (D62): bash reads `BASH_ENV` (and `ENV`) even with
/// `--norc`, and the git variables would change what a plan's own git commands mean.
const REMOVED: [&str; 6] = [
    "BASH_ENV",
    "ENV",
    "GIT_LITERAL_PATHSPECS",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

/// What a session needs to start.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// The working directory the session starts in: the repository root.
    pub root: PathBuf,
    /// Variables added to planrun's own environment (tests: the fixture's git isolation).
    pub env: Vec<(OsString, OsString)>,
    /// How long one block may run; `None` waits forever (D65).
    pub timeout: Option<Duration>,
    /// Where the run directory is made: the OS temp directory outside the tests.
    pub temp: PathBuf,
}

/// How a shell is started and spoken to; one implementation per shell.
pub(super) trait Dialect: Send {
    /// `pwsh` or `bash`, for messages.
    fn name(&self) -> &'static str;
    /// The arguments of the shell process.
    fn args(&self, marker: &str) -> Vec<String>;
    /// The block file's extension.
    fn extension(&self) -> &'static str;
    /// The prologue as one ASCII stdin line that ends with the sentinel of block 0, or `None`
    /// when the session gets no stdin and its arguments carry the prologue.
    fn prologue(&self, marker: &str) -> Option<String>;
    /// The stdin line that runs block `n` from its file and prints its sentinel, or `None` when
    /// the shell picks the file up itself.
    fn block_line(&self, n: u64) -> Option<String>;
    /// The working directory as the shell reported it, as a path of this platform.
    fn cwd(&self, reported: String) -> PathBuf;
}

/// Finds the shell planrun runs for `shell` (D63) and checks it with a probe.
pub fn locate(shell: Shell) -> Result<PathBuf, ShellError> {
    match shell {
        Shell::Pwsh => pwsh::locate(),
        Shell::Bash => bash::locate(),
    }
}

/// One running shell.
pub struct Session {
    dialect: Box<dyn Dialect>,
    tree: Tree,
    stdin: Option<ChildStdin>,
    events: Receiver<Event>,
    dir: RunDir,
    marker: String,
    block: u64,
    timeout: Option<Duration>,
    alive: bool,
}

enum Event {
    Line {
        stream: Stream,
        bytes: Vec<u8>,
        newline: bool,
    },
    Eof(Stream),
}

/// What a sentinel line on stdout says.
struct Frame {
    kind: Kind,
    status: i32,
    errored: bool,
    cwd: String,
}

#[derive(PartialEq, Eq)]
enum Kind {
    Ok,
    Threw,
    Exit,
}

/// What reading one block's events found.
struct Collected {
    lines: Vec<OutLine>,
    frame: Option<Frame>,
    ended: bool,
}

impl Session {
    /// Starts `shell` at `config.root` and runs its prologue.
    pub fn start(shell: Shell, config: &SessionConfig) -> Result<Session, ShellError> {
        let program = locate(shell)?;
        let dialect: Box<dyn Dialect> = match shell {
            Shell::Pwsh => Box::new(pwsh::Pwsh),
            Shell::Bash => Box::new(bash::Bash),
        };
        let dir = RunDir::create(&config.temp)?;
        let marker = format!("__planrun_{}_", secret());
        let mut command = Command::new(&program);
        command
            .args(dialect.args(&marker))
            .current_dir(&config.root);
        for name in REMOVED {
            command.env_remove(name);
        }
        command
            .env("GIT_TERMINAL_PROMPT", "0")
            .envs(config.env.iter().map(|(k, v)| (k, v)))
            .env(DIR_VAR, dir.path());
        let prologue = dialect.prologue(&marker);
        let (tree, pipes) = Tree::spawn(&mut command, prologue.is_some())?;
        let (tx, events) = channel();
        read(Stream::Stdout, pipes.stdout, tx.clone());
        read(Stream::Stderr, pipes.stderr, tx);
        let mut session = Session {
            dialect,
            tree,
            stdin: pipes.stdin,
            events,
            dir,
            marker,
            block: 0,
            timeout: config.timeout,
            alive: true,
        };
        session.handshake(prologue)?;
        Ok(session)
    }

    /// Whether the session can run another block.
    pub fn is_alive(&self) -> bool {
        self.alive
    }

    fn handshake(&mut self, prologue: Option<String>) -> Result<(), ShellError> {
        let name = self.dialect.name();
        let run = self.send(prologue, &mut |_| {})?;
        let output: Vec<&str> = run.lines.iter().map(|l| l.text.as_str()).collect();
        match run.status {
            RunStatus::Exited(0) if output.is_empty() => Ok(()),
            status => Err(ShellError::Start {
                shell: name,
                reason: format!("the prologue gave {status:?} with output {output:?}"),
            }),
        }
    }

    /// Writes `line`, if any, to the session's stdin and collects the block's events.
    fn send(
        &mut self,
        line: Option<String>,
        on_line: &mut dyn FnMut(&OutLine),
    ) -> io::Result<ScriptRun> {
        if let (Some(line), Some(stdin)) = (line, self.stdin.as_mut()) {
            let written = stdin
                .write_all(line.as_bytes())
                .and_then(|()| stdin.write_all(b"\n"))
                .and_then(|()| stdin.flush());
            if written.is_err() {
                return Ok(self.end(Vec::new(), RunStatus::SessionEnded(None)));
            }
        }
        let deadline = self.timeout.map(|t| Instant::now() + t);
        let collected = match self.collect(deadline, on_line) {
            Ok(collected) => collected,
            Err((lines, status)) => return Ok(self.end(lines, status)),
        };
        Ok(self.finish(collected))
    }

    /// Reads events until both sentinels of the current block arrived, or a stream ended.
    fn collect(
        &mut self,
        deadline: Option<Instant>,
        on_line: &mut dyn FnMut(&OutLine),
    ) -> Result<Collected, (Vec<OutLine>, RunStatus)> {
        let mut c = Collected {
            lines: Vec::new(),
            frame: None,
            ended: false,
        };
        let (mut out_done, mut err_done) = (false, false);
        while !(out_done && err_done) {
            let event = match self.next(deadline) {
                Ok(event) => event,
                Err(RecvTimeoutError::Timeout) => return Err((c.lines, RunStatus::TimedOut)),
                Err(RecvTimeoutError::Disconnected) => {
                    c.ended = true;
                    break;
                }
            };
            let (stream, bytes, newline) = match event {
                Event::Eof(Stream::Stdout) => {
                    (out_done, c.ended) = (true, true);
                    continue;
                }
                Event::Eof(Stream::Stderr) => {
                    (err_done, c.ended) = (true, true);
                    continue;
                }
                Event::Line {
                    stream,
                    bytes,
                    newline,
                } => (stream, bytes, newline),
            };
            let text = String::from_utf8_lossy(&bytes).into_owned();
            let Some(at) = text.find(&self.marker) else {
                emit(&mut c.lines, on_line, stream, text, newline);
                continue;
            };
            if at > 0 {
                emit(&mut c.lines, on_line, stream, text[..at].to_owned(), false);
            }
            let rest = &text[at + self.marker.len()..];
            match (stream, self.parse(rest, stream)) {
                (_, Err(reason)) => return Err((c.lines, RunStatus::Broken(reason.into()))),
                (Stream::Stdout, Ok(frame)) => {
                    c.frame = frame;
                    out_done = true;
                }
                (Stream::Stderr, Ok(_)) => err_done = true,
            }
        }
        Ok(c)
    }

    fn next(&self, deadline: Option<Instant>) -> Result<Event, RecvTimeoutError> {
        let Some(deadline) = deadline else {
            return self
                .events
                .recv()
                .map_err(|_| RecvTimeoutError::Disconnected);
        };
        let left = deadline.saturating_duration_since(Instant::now());
        self.events.recv_timeout(left)
    }

    /// Parses what follows the marker: `<n>__`, then on stdout ` <kind> <status> <errored>
    /// <cwd hex>` and nothing else, on stderr nothing at all.
    fn parse(&self, rest: &str, stream: Stream) -> Result<Option<Frame>, String> {
        let rest = rest.strip_suffix('\r').unwrap_or(rest);
        let Some((number, fields)) = rest.split_once("__") else {
            return Err(format!("a sentinel without its block number: {rest:?}"));
        };
        if number.parse::<u64>().ok() != Some(self.block) {
            return Err(format!(
                "a sentinel for block {number:?} during block {}",
                self.block
            ));
        }
        if stream == Stream::Stderr {
            if !fields.is_empty() {
                return Err(format!(
                    "unexpected text after the stderr sentinel: {fields:?}"
                ));
            }
            return Ok(None);
        }
        let parts: Vec<&str> = fields.split(' ').collect();
        let [empty, kind, status, errored, cwd] = parts.as_slice() else {
            return Err(format!("unexpected sentinel fields: {fields:?}"));
        };
        let kind = match *kind {
            "ok" => Kind::Ok,
            "threw" => Kind::Threw,
            "exit" => Kind::Exit,
            other => return Err(format!("an unknown sentinel kind {other:?}")),
        };
        let (Ok(status), true, Some(cwd)) = (
            status.parse::<i32>(),
            empty.is_empty() && matches!(*errored, "0" | "1"),
            unhex(cwd),
        ) else {
            return Err(format!("unexpected sentinel fields: {fields:?}"));
        };
        Ok(Some(Frame {
            kind,
            status,
            errored: *errored == "1",
            cwd,
        }))
    }

    /// The result of a block whose events were all read.
    fn finish(&mut self, c: Collected) -> ScriptRun {
        let Some(frame) = c.frame else {
            return self.end(c.lines, RunStatus::SessionEnded(None));
        };
        let cwd = Some(self.dialect.cwd(frame.cwd));
        let status = match frame.kind {
            Kind::Exit => RunStatus::SessionEnded(Some(frame.status)),
            _ if c.ended => RunStatus::SessionEnded(None),
            Kind::Threw => RunStatus::Threw,
            Kind::Ok => RunStatus::Exited(frame.status),
        };
        let mut run = match status {
            RunStatus::SessionEnded(_) => self.end(c.lines, status),
            status => ScriptRun {
                status,
                errored: false,
                lines: c.lines,
                cwd: None,
            },
        };
        run.errored = frame.errored;
        run.cwd = cwd;
        run
    }

    /// Ends the session: kills its tree and marks it dead.
    fn end(&mut self, lines: Vec<OutLine>, status: RunStatus) -> ScriptRun {
        self.alive = false;
        let _ = self.tree.kill();
        let _ = self.tree.wait();
        ScriptRun {
            status,
            errored: false,
            lines,
            cwd: None,
        }
    }
}

impl ShellSession for Session {
    fn run(&mut self, script: &str, on_line: &mut dyn FnMut(&OutLine)) -> io::Result<ScriptRun> {
        if !self.alive {
            return Err(io::Error::other(format!(
                "the {} session has ended",
                self.dialect.name()
            )));
        }
        self.block += 1;
        let staged = self.dir.path().join(format!("b{}.tmp", self.block));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)?;
        file.write_all(script.as_bytes())?;
        drop(file);
        let name = format!("b{}.{}", self.block, self.dialect.extension());
        std::fs::rename(&staged, self.dir.path().join(name))?;
        let line = self.dialect.block_line(self.block);
        self.send(line, on_line)
    }
}

fn emit(
    lines: &mut Vec<OutLine>,
    on_line: &mut dyn FnMut(&OutLine),
    stream: Stream,
    text: String,
    newline: bool,
) {
    let line = OutLine {
        stream,
        text,
        newline,
    };
    on_line(&line);
    lines.push(line);
}

/// One reader thread per stream, detached: a build server that inherited the pipe may keep it
/// open long after the session is gone, and nothing waits for the thread.
fn read(stream: Stream, pipe: impl Read + Send + 'static, tx: Sender<Event>) {
    std::thread::spawn(move || {
        let mut reader = BufReader::new(pipe);
        loop {
            let mut bytes = Vec::new();
            match reader.read_until(b'\n', &mut bytes) {
                Ok(0) | Err(_) => {
                    let _ = tx.send(Event::Eof(stream));
                    return;
                }
                Ok(_) => {}
            }
            let newline = bytes.last() == Some(&b'\n');
            if newline {
                bytes.pop();
            }
            let line = Event::Line {
                stream,
                bytes,
                newline,
            };
            if tx.send(line).is_err() {
                return;
            }
        }
    });
}

/// Lowercase or uppercase hex of UTF-8 bytes back to text; `None` when it is not hex.
fn unhex(hex: &str) -> Option<String> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    let bytes: Option<Vec<u8>> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok())
        .collect();
    Some(String::from_utf8_lossy(&bytes?).into_owned())
}

/// 128 random bits as 32 hex digits.
fn secret() -> String {
    let half = |salt: u64| {
        let mut hasher = RandomState::new().build_hasher();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        hasher.write_u128(nanos);
        hasher.write_u32(std::process::id());
        hasher.write_u64(salt);
        hasher.finish()
    };
    format!("{:016x}{:016x}", half(1), half(2))
}

/// The run directory: created fresh (it must not exist), private on Unix, removed on drop.
struct RunDir(PathBuf);

impl RunDir {
    fn create(temp: &Path) -> io::Result<RunDir> {
        let path = temp.join(format!("planrun-shell-{}", &secret()[..16]));
        let builder = DirBuilder::new();
        #[cfg(unix)]
        let builder = {
            use std::os::unix::fs::DirBuilderExt;
            let mut builder = builder;
            builder.mode(0o700);
            builder
        };
        builder.create(&path)?;
        Ok(RunDir(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for RunDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_utf8_and_refuses_odd_lengths() {
        assert_eq!(unhex("52656ec3a9").as_deref(), Some("René"));
        assert_eq!(unhex("").as_deref(), Some(""));
        assert_eq!(unhex("5"), None);
        assert_eq!(unhex("zz"), None);
    }

    #[test]
    fn secrets_differ() {
        let (a, b) = (secret(), secret());
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
    }
}
```


File `crates/planrun-exec/tests/shell.rs` (full contents):

```rust
//! pwsh and bash sessions on a fixture copy (D60–D65): state across blocks, the wrapper's own
//! names, exit and throw, stdin isolation, the environment, the timeout's tree kill. A missing
//! shell skips its tests with a message, unless `PLANRUN_REQUIRE_SHELLS` is set.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use fixture::Fixture;
use planrun_exec::ShellSession;
use planrun_exec::shell::{
    OutLine, RunStatus, ScriptRun, Session, SessionConfig, ShellError, Stream,
};
use planrun_model::Shell;

const EMPTY_BLOB: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";

/// A session in the fixture copy with its git isolation, or `None` (and a message) when the
/// shell is missing and shells are not required.
fn start(shell: Shell, f: &Fixture, timeout: Duration) -> Option<Session> {
    start_in(shell, f, timeout, f.outside())
}

fn start_in(shell: Shell, f: &Fixture, timeout: Duration, temp: PathBuf) -> Option<Session> {
    let config = SessionConfig {
        root: f.path().to_path_buf(),
        env: f.env(),
        timeout: Some(timeout),
        temp,
    };
    match Session::start(shell, &config) {
        Ok(session) => Some(session),
        Err(e @ ShellError::Unavailable { .. })
            if std::env::var_os("PLANRUN_REQUIRE_SHELLS").is_none() =>
        {
            eprintln!("skipped: {e}");
            None
        }
        Err(e) => panic!("{e}"),
    }
}

fn session(shell: Shell, f: &Fixture) -> Option<Session> {
    start(shell, f, Duration::from_secs(60))
}

fn run(s: &mut Session, script: &str) -> ScriptRun {
    s.run(script, &mut |_| {}).unwrap_or_else(|e| panic!("{e}"))
}

/// The run's stdout lines, without a trailing CR.
fn stdout(r: &ScriptRun) -> Vec<&str> {
    r.lines
        .iter()
        .filter(|l| l.stream == Stream::Stdout)
        .map(|l| l.text.trim_end_matches('\r'))
        .collect()
}

fn same_dir(a: &Path, b: &Path) -> bool {
    let canon =
        |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    canon(a) == canon(b)
}

#[test]
fn pwsh_keeps_variables_functions_and_the_working_directory() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    let first = run(
        &mut s,
        "$x = 41\nfunction F { 'f-called' }\nNew-Item -ItemType Directory sub | Out-Null\nSet-Location sub\n",
    );
    assert_eq!(first.status, RunStatus::Exited(0));
    let second = run(&mut s, "$x + 1\nF\n");
    assert_eq!(stdout(&second), ["42", "f-called"]);
    let cwd = second.cwd.unwrap_or_else(|| panic!("no cwd"));
    assert!(same_dir(&cwd, &f.path().join("sub")), "{}", cwd.display());
}

#[test]
fn pwsh_wrapper_names_do_not_touch_the_plan_s_and_a_throw_is_reset() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    let threw = run(
        &mut s,
        "$t = 'mine'; $s = 'own'; $e = 'kept'\nthrow 'boom'\n",
    );
    assert_eq!(threw.status, RunStatus::Threw);
    assert!(threw.errored);
    assert!(
        threw
            .lines
            .iter()
            .any(|l| l.stream == Stream::Stderr && l.text.contains("boom"))
    );
    let after = run(&mut s, "\"$t $s $e\"\n");
    assert_eq!(after.status, RunStatus::Exited(0));
    assert!(!after.errored);
    assert_eq!(stdout(&after), ["mine own kept"]);
}

#[test]
fn pwsh_exit_ends_the_block_only_and_lastexitcode_is_reset() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    assert_eq!(
        run(&mut s, "$v = 'kept'\nexit 7\n'not reached'\n").status,
        RunStatus::Exited(7)
    );
    let next = run(&mut s, "$v\n");
    assert_eq!(
        (next.status.clone(), stdout(&next)),
        (RunStatus::Exited(0), vec!["kept"])
    );
    let native = run(&mut s, "git rev-parse --verify --quiet refs/heads/nosuch\n");
    assert_eq!(native.status, RunStatus::Exited(1));
    assert_eq!(
        run(&mut s, "'no native command'\n").status,
        RunStatus::Exited(0)
    );
}

#[test]
fn pwsh_break_trap_and_a_parse_error_still_reach_the_sentinel() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    let broke = run(&mut s, "'before'\nbreak\n'after'\n");
    assert_eq!(stdout(&broke), ["before"]);
    let trapped = run(&mut s, "trap { 'trapped' }\nthrow 'x'\n'after the throw'\n");
    assert_eq!(stdout(&trapped), ["trapped", "after the throw"]);
    let parse = run(&mut s, "'start'\nif (\n");
    assert_eq!(parse.status, RunStatus::Threw);
    assert!(stdout(&parse).is_empty(), "a parse error runs nothing");
    assert_eq!(run(&mut s, "'alive'\n").status, RunStatus::Exited(0));
}

#[test]
fn pwsh_errored_follows_error_zero() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    let cases = [
        ("Get-Item nosuch -ErrorAction SilentlyContinue\n", true),
        ("Get-Item nosuch -ErrorAction Ignore\n", false),
        ("git rev-parse --verify nosuch 2>$null\n", false),
        ("$x = git rev-parse --verify nosuch 2>&1\n", false),
        ("Write-Error 'non-terminating'\n", true),
        ("'clean'\n", false),
    ];
    for (script, errored) in cases {
        let r = run(&mut s, script);
        assert_eq!(r.errored, errored, "{script:?}");
        assert!(!matches!(r.status, RunStatus::Threw), "{script:?}");
    }
}

#[test]
fn pwsh_output_order_partial_lines_and_unicode() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    let order = run(&mut s, "Write-Output a; git --version; Write-Output b\n");
    let out = stdout(&order);
    assert_eq!(out.len(), 3, "{out:?}");
    assert_eq!((out[0], out[2]), ("a", "b"));
    assert!(out[1].starts_with("git version"), "{out:?}");
    let mut seen = Vec::new();
    let partial = s
        .run(
            "'René ✓'\n[Console]::Out.Write('tail')\n",
            &mut |l: &OutLine| seen.push(l.clone()),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(seen, partial.lines, "every line is streamed");
    assert_eq!(stdout(&partial), ["René ✓", "tail"]);
    assert!(!partial.lines[1].newline);
}

#[test]
fn pwsh_and_bash_blocks_get_no_stdin() {
    let f = Fixture::greeter(false);
    let script = "git hash-object --stdin\n";
    for shell in [Shell::Pwsh, Shell::Bash] {
        let Some(mut s) = session(shell, &f) else {
            continue;
        };
        let r = run(&mut s, script);
        assert_eq!(stdout(&r), [EMPTY_BLOB], "{shell:?}");
        let cat = if shell == Shell::Bash {
            "cat; echo next\n"
        } else {
            "'next'\n"
        };
        assert_eq!(stdout(&run(&mut s, cat)), ["next"], "{shell:?}");
    }
}

#[test]
fn sessions_get_planrun_s_environment_without_the_git_wrapper_s() {
    let f = Fixture::greeter(false);
    let pwsh = "foreach ($n in 'GIT_LITERAL_PATHSPECS', 'GIT_INDEX_FILE', 'GIT_OBJECT_DIRECTORY', \
                'GIT_ALTERNATE_OBJECT_DIRECTORIES', 'BASH_ENV', 'ENV', 'PLANRUN_SESSION_DIR') \
                { if (Test-Path \"Env:$n\") { \"set: $n\" } }\n\"prompt=$env:GIT_TERMINAL_PROMPT\"\n\
                git config --show-origin --list\n";
    let bash = "for n in GIT_LITERAL_PATHSPECS GIT_INDEX_FILE GIT_OBJECT_DIRECTORY \
                GIT_ALTERNATE_OBJECT_DIRECTORIES BASH_ENV ENV PLANRUN_SESSION_DIR; do \
                [ -n \"${!n+x}\" ] && echo \"set: $n\"; done; echo \"prompt=$GIT_TERMINAL_PROMPT\"\n\
                git config --show-origin --list\n";
    for (shell, script) in [(Shell::Pwsh, pwsh), (Shell::Bash, bash)] {
        let Some(mut s) = session(shell, &f) else {
            continue;
        };
        let r = run(&mut s, script);
        let out = stdout(&r);
        assert_eq!(out.first().copied(), Some("prompt=0"), "{shell:?}: {out:?}");
        assert!(out.len() > 1, "{shell:?}: no configuration listed");
        for line in &out[1..] {
            assert!(line.starts_with("file:.git/config\t"), "{shell:?}: {line}");
        }
    }
}

#[test]
fn bash_keeps_declared_variables_and_functions_and_has_no_arguments() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Bash, &f) else {
        return;
    };
    let first = run(
        &mut s,
        "declare -A m=([k]=v)\ny=5\ng() { echo g-called; }\nmkdir -p sub && cd sub\n",
    );
    assert_eq!(first.status, RunStatus::Exited(0));
    let second = run(&mut s, "echo \"${m[k]} $y\"\ng\necho \"[$1][$#]\"\n");
    assert_eq!(stdout(&second), ["v 5", "g-called", "[][0]"]);
    let cwd = second.cwd.unwrap_or_else(|| panic!("no cwd"));
    assert!(same_dir(&cwd, &f.path().join("sub")), "{}", cwd.display());
}

#[test]
fn bash_wrapper_names_do_not_touch_the_plan_s_and_status_is_the_last_command_s() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Bash, &f) else {
        return;
    };
    assert_eq!(
        run(&mut s, "s=mine; t=own; n=kept\nfalse\n").status,
        RunStatus::Exited(1)
    );
    let after = run(&mut s, "echo \"$s $t $n\"\n");
    assert_eq!(after.status, RunStatus::Exited(0));
    assert_eq!(stdout(&after), ["mine own kept"]);
    let syntax = run(&mut s, "echo before\nif then fi\n");
    assert_eq!(
        syntax.status,
        RunStatus::Exited(2),
        "a syntax error ends the block only"
    );
    assert_eq!(stdout(&run(&mut s, "echo alive\n")), ["alive"]);
}

#[test]
fn bash_exit_set_e_a_replaced_trap_and_exec_end_the_session() {
    let f = Fixture::greeter(false);
    let cases: [(&str, RunStatus); 4] = [
        ("echo bye\nexit 3\n", RunStatus::SessionEnded(Some(3))),
        ("set -e\n", RunStatus::SessionEnded(Some(1))),
        ("trap - EXIT\nexit 4\n", RunStatus::SessionEnded(None)),
        ("exec true\n", RunStatus::SessionEnded(None)),
    ];
    for (script, status) in cases {
        let Some(mut s) = session(Shell::Bash, &f) else {
            return;
        };
        let mut r = run(&mut s, script);
        if script.starts_with("set -e") {
            assert_eq!(r.status, RunStatus::Exited(0));
            r = run(&mut s, "false\n");
        }
        assert_eq!(r.status, status, "{script:?}");
        assert!(!s.is_alive(), "{script:?}");
        assert!(s.run("echo x\n", &mut |_| {}).is_err(), "{script:?}");
    }
}

#[test]
fn a_forged_sentinel_breaks_the_session() {
    let f = Fixture::greeter(false);
    let pwsh =
        "$m = ([regex]'__planrun_[0-9a-f]+_').Match(${function:__planrun_end}.ToString()).Value\n";
    let cases = [
        (Shell::Pwsh, format!("{pwsh}[Console]::Out.WriteLine($m + '99__ ok 0 0 ')\n")),
        (Shell::Pwsh, format!("{pwsh}[Console]::Out.WriteLine($m + '1__ ok 0 0  extra')\n")),
        (
            Shell::Bash,
            "m=$(declare -f __planrun_end | grep -o '__planrun_[0-9a-f]*_' | head -1)\necho \"${m}1__ ok 0 0 zz\"\n"
                .to_owned(),
        ),
    ];
    for (shell, script) in cases {
        let Some(mut s) = session(shell, &f) else {
            continue;
        };
        let r = run(&mut s, &script);
        assert!(
            matches!(r.status, RunStatus::Broken(_)),
            "{shell:?}: {:?}",
            r.status
        );
        assert!(!s.is_alive());
    }
}

/// Whether the process `pid` still runs.
fn alive(pid: &str) -> bool {
    let out = if cfg!(windows) {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
    } else {
        Command::new("kill").args(["-0", pid]).output()
    }
    .unwrap_or_else(|e| panic!("{e}"));
    if cfg!(windows) {
        return String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .any(|w| w == pid);
    }
    out.status.success()
}

#[test]
fn a_timeout_kills_the_whole_tree() {
    let f = Fixture::greeter(false);
    let pid_file = f.outside().join("pid");
    let file = pid_file.display().to_string();
    let pwsh = format!(
        "$p = Start-Process pwsh -ArgumentList '-NoProfile', '-Command', 'Start-Sleep 300' -PassThru\n\
         Set-Content -LiteralPath '{file}' -Value $p.Id\nStart-Sleep 300\n"
    );
    let bash = format!(
        "sleep 300 &\nif [ -r /proc/$!/winpid ]; then cat /proc/$!/winpid; else echo $!; fi > '{}'\nsleep 300\n",
        file.replace('\\', "/")
    );
    for (shell, script) in [(Shell::Pwsh, pwsh), (Shell::Bash, bash)] {
        let Some(mut s) = start(shell, &f, Duration::from_secs(5)) else {
            continue;
        };
        let r = run(&mut s, &script);
        assert_eq!(r.status, RunStatus::TimedOut, "{shell:?}");
        assert!(!s.is_alive());
        let pid = std::fs::read_to_string(&pid_file).unwrap_or_else(|e| panic!("{shell:?}: {e}"));
        let pid = pid.trim().to_owned();
        let until = Instant::now() + Duration::from_secs(10);
        while alive(&pid) && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(!alive(&pid), "{shell:?}: the grandchild {pid} survived");
        let _ = std::fs::remove_file(&pid_file);
    }
}

#[cfg(windows)]
#[test]
fn a_session_leaves_the_terminal_s_code_page_alone() {
    let chcp = || {
        let out = Command::new("cmd").args(["/c", "chcp"]).output();
        String::from_utf8_lossy(&out.unwrap_or_else(|e| panic!("{e}")).stdout).into_owned()
    };
    let before = chcp();
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    assert_eq!(run(&mut s, "'x'\n").status, RunStatus::Exited(0));
    drop(s);
    assert_eq!(chcp(), before);
}

#[test]
fn a_run_directory_with_non_ascii_spaces_and_quotes_works_and_is_removed() {
    let f = Fixture::greeter(false);
    let temp = f.outside().join("René 'x' ✓");
    std::fs::create_dir_all(&temp).unwrap_or_else(|e| panic!("{e}"));
    for shell in [Shell::Pwsh, Shell::Bash] {
        let Some(mut s) = start_in(shell, &f, Duration::from_secs(60), temp.clone()) else {
            continue;
        };
        let r = run(
            &mut s, "echo ok
",
        );
        assert_eq!(
            (r.status.clone(), stdout(&r)),
            (RunStatus::Exited(0), vec!["ok"]),
            "{shell:?}"
        );
        drop(s);
        let left = std::fs::read_dir(&temp)
            .unwrap_or_else(|e| panic!("{e}"))
            .count();
        assert_eq!(left, 0, "{shell:?}: the run directory stayed");
    }
}
```


**Step 2 — verify**

Targeted: the shell tests, with both shells required:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-exec --test shell
```


Expected (exit code 0; summary lines):

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```


Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```


Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```


**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-exec/src/lib.rs crates/planrun-exec/src/shell/bash.rs crates/planrun-exec/src/shell/kill.rs crates/planrun-exec/src/shell/mod.rs crates/planrun-exec/src/shell/pwsh.rs crates/planrun-exec/src/shell/session.rs crates/planrun-exec/tests/shell.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/lib.rs' = '4d74e157d6ccf8552e893b407875a3e73a16869d'
      'crates/planrun-exec/src/shell/bash.rs' = '84fe470e770a26a60646f17f7915b152ddf41faf'
      'crates/planrun-exec/src/shell/kill.rs' = '7d230d8eba2aad143af945874578f47d86306818'
      'crates/planrun-exec/src/shell/mod.rs' = '1569051d9ff31fc5d58b3f2b9493b6d7ba7009f7'
      'crates/planrun-exec/src/shell/pwsh.rs' = '917c94affc2c81083c2b0f0c911dab9bf5d7e357'
      'crates/planrun-exec/src/shell/session.rs' = '08001692e77b5342c7f46361a86921816ff91fee'
      'crates/planrun-exec/tests/shell.rs' = '6a58c066845b69e924c9d3e6a181b9b83e9e9465'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P4-T2: persistent pwsh and bash sessions: the sentinel protocol, the process tree, the session environment" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```


Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P4-T3: the expectation matcher: exit codes, summary lines with …, what asks for confirmation

**Goal:** `expect::check`: a run against its expectation, `Pass`, `Mismatch` or `Confirm` (D66). Summary lines as a subsequence with `…`, the normalization rules, the exit rules, and what asks: prose, comments, an unstated non-zero exit, `errored`.

**Files**

- Create `crates/planrun-exec/src/expect.rs`
- Modify `crates/planrun-exec/src/lib.rs`

**Step 1 — write the files**

File `crates/planrun-exec/src/expect.rs` (full contents):

```rust
//! Matches a block's run against its expectation (docs/architecture.md §6.4; decisions D11,
//! D61, D66). Only exit codes and fenced summary lines pass a script on their own; prose and
//! comment expectations, an exit the plan did not state that is not 0, and a recorded pwsh error
//! all ask for confirmation. Skipping an expected failure is decided before the run (D67), never
//! here.

use std::borrow::Cow;

use planrun_model::{ExitExpect, Expectation, PlanSource};

use crate::shell::{RunStatus, ScriptRun};

/// What the matcher concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Everything stated was checked and holds; nothing asks.
    Pass,
    /// Something checked does not hold.
    Mismatch(Vec<Mismatch>),
    /// The checked parts hold, and the user confirms the rest.
    Confirm(Confirm),
}

/// Why a run asks for confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Confirm {
    /// The reasons, in the order prose, comments, unchecked exit, recorded error.
    pub reasons: Vec<Reason>,
    /// The run's `errored` flag, shown with every confirmation.
    pub errored: bool,
}

/// One reason to confirm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// The plan says what to expect in a sentence (D11).
    Prose,
    /// The script's trailing comments say it (D35, D48).
    Comments,
    /// The plan states no exit status, and the block did not exit 0 (it failed or threw).
    UncheckedExit,
    /// pwsh recorded an error, and nothing but an exit status (or nothing) was expected.
    Errored,
}

/// One thing that does not hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mismatch {
    /// The exit status. `expected` is `None` when the plan states none: a session that ended,
    /// timed out or broke is a mismatch whatever was expected.
    Exit {
        /// What the plan expects.
        expected: Option<ExitExpect>,
        /// What happened.
        got: RunStatus,
    },
    /// A summary line no output line matches, in order after the ones before it.
    Line {
        /// Its 1-based line number within the expected block.
        number: usize,
        /// The expected line, as written.
        line: String,
    },
}

/// Checks `run` against `expect` (`None`: the plan expects nothing in particular).
pub fn check(expect: Option<&Expectation>, run: &ScriptRun, source: &PlanSource) -> Verdict {
    let mut mismatches = Vec::new();
    let ended = matches!(
        run.status,
        RunStatus::SessionEnded(_) | RunStatus::TimedOut | RunStatus::Broken(_)
    );
    let exit = expect.map_or(ExitExpect::Unknown, |e| e.exit);
    let exit_holds = match exit {
        ExitExpect::Success => run.status == RunStatus::Exited(0),
        ExitExpect::Failure => {
            matches!(run.status, RunStatus::Exited(n) if n != 0) || run.status == RunStatus::Threw
        }
        ExitExpect::Code(code) => run.status == RunStatus::Exited(code),
        ExitExpect::Unknown => !ended,
    };
    if !exit_holds {
        mismatches.push(Mismatch::Exit {
            expected: expect.map(|e| e.exit),
            got: run.status.clone(),
        });
    }
    if let Some(lines) = expect.and_then(|e| e.lines.as_ref()) {
        mismatches.extend(unmatched(source.resolve(lines), run));
    }
    if !mismatches.is_empty() {
        return Verdict::Mismatch(mismatches);
    }
    let reasons = reasons(expect, exit, run);
    if reasons.is_empty() {
        return Verdict::Pass;
    }
    Verdict::Confirm(Confirm {
        reasons,
        errored: run.errored,
    })
}

fn reasons(expect: Option<&Expectation>, exit: ExitExpect, run: &ScriptRun) -> Vec<Reason> {
    let mut reasons = Vec::new();
    let (lines, prose, comments) = expect.map_or((false, false, false), |e| {
        (e.lines.is_some(), e.prose.is_some(), e.comments.is_some())
    });
    if prose {
        reasons.push(Reason::Prose);
    }
    if comments {
        reasons.push(Reason::Comments);
    }
    if exit == ExitExpect::Unknown && run.status != RunStatus::Exited(0) {
        reasons.push(Reason::UncheckedExit);
    }
    let exit_only = !lines && !prose && !comments;
    let plain = matches!(exit, ExitExpect::Success | ExitExpect::Unknown);
    if run.errored && exit_only && plain {
        reasons.push(Reason::Errored);
    }
    reasons
}

/// The expected lines that do not match, as a subsequence of the output: each non-blank line
/// must match an output line after the one the line before it matched. The earliest match is
/// taken for each, which finds a subsequence whenever one exists.
fn unmatched(expected: &str, run: &ScriptRun) -> Vec<Mismatch> {
    let output: Vec<Cow<'_, str>> = run.lines.iter().map(|l| strip_ansi(&l.text)).collect();
    let mut next = 0;
    let mut missing = Vec::new();
    for (i, line) in expected.split('\n').enumerate() {
        let pattern = line.trim_end();
        if pattern.is_empty() {
            continue;
        }
        let found = output[next..]
            .iter()
            .position(|out| line_matches(pattern, out.trim_end()));
        match found {
            Some(at) => next += at + 1,
            None => missing.push(Mismatch::Line {
                number: i + 1,
                line: line.to_owned(),
            }),
        }
    }
    missing
}

/// Whether `line` matches `pattern`: equal, except that each `…` matches any run of characters,
/// the empty one included. The first literal segment is anchored at the start, the last at the
/// end; the ones between are found in order, each at its earliest place.
pub fn line_matches(pattern: &str, line: &str) -> bool {
    let segments: Vec<&str> = pattern.split('…').collect();
    let [first, middle @ .., last] = segments.as_slice() else {
        return pattern == line;
    };
    if first.len() + last.len() > line.len() || !line.starts_with(first) || !line.ends_with(last) {
        return false;
    }
    let mut rest = &line[first.len()..line.len() - last.len()];
    for segment in middle {
        let Some(at) = rest.find(segment) else {
            return false;
        };
        rest = &rest[at + segment.len()..];
    }
    true
}

/// `text` without ANSI CSI (`ESC [ … final`) and OSC (`ESC ] … BEL` or `ESC ] … ESC \`)
/// sequences. An unterminated sequence is dropped to the end of the text.
pub fn strip_ansi(text: &str) -> Cow<'_, str> {
    if !text.contains('\u{1b}') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            Some('[') => {
                chars.next();
                for c in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&c) {
                        break;
                    }
                }
            }
            Some(']') => {
                chars.next();
                while let Some(c) = chars.next() {
                    if c == '\u{7}' || (c == '\u{1b}' && chars.next_if_eq(&'\\').is_some()) {
                        break;
                    }
                }
            }
            _ => out.push(c),
        }
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use planrun_model::{Span, TextRef};

    use super::*;
    use crate::shell::{OutLine, Stream};

    #[test]
    fn line_patterns() {
        #[rustfmt::skip]
        let rows = [
            ("abc", "abc", true),
            ("abc", "abcd", false),
            ("abc", " abc", false),
            ("a…b", "ab", true),
            ("a…b", "a-b", true),
            ("a…b", "a", false),
            ("a…b", "b", false),
            ("a…a", "a", false),
            ("a…a", "aa", true),
            ("…ab…ab", "abab", true),
            ("…ab…ab", "ab", false),
            ("…ab…ab", "xabyab", true),
            ("…", "", true),
            ("…", "anything", true),
            ("a……b", "ab", true),
            ("test result: ok. … passed; 0 failed; …", "test result: ok. 3 passed; 0 failed; 0 ignored", true),
            ("test result: ok. … passed; 0 failed; …", "test result: FAILED. 3 passed; 1 failed; 0 ignored", false),
            ("a...b", "a-b", false),
            ("a...b", "a...b", true),
            ("René…✓", "René sees ✓", true),
        ];
        for (pattern, line, expected) in rows {
            assert_eq!(
                line_matches(pattern, line),
                expected,
                "{pattern:?} against {line:?}"
            );
        }
    }

    #[test]
    fn ansi_sequences() {
        #[rustfmt::skip]
        let rows = [
            ("plain", "plain"),
            ("\u{1b}[32mok\u{1b}[0m", "ok"),
            ("\u{1b}[1;31merror\u{1b}[0m: x", "error: x"),
            ("\u{1b}]0;title\u{7}text", "text"),
            ("\u{1b}]8;;http://x\u{1b}\\link\u{1b}]8;;\u{1b}\\", "link"),
            ("a\u{1b}[", "a"),
            ("a\u{1b}x", "a\u{1b}x"),
        ];
        for (text, expected) in rows {
            assert_eq!(strip_ansi(text), expected, "{text:?}");
        }
    }

    /// The shape of an expectation; its texts are made up by `verdict`.
    struct Case {
        exit: ExitExpect,
        lines: Option<&'static str>,
        prose: bool,
        comments: bool,
    }

    /// One row: the expectation, the run's status, `errored`, its output, the verdict.
    type Row = (
        Option<Case>,
        RunStatus,
        bool,
        &'static [&'static str],
        &'static str,
    );

    fn verdict(case: Option<Case>, status: RunStatus, errored: bool, output: &[&str]) -> String {
        let mut text = String::new();
        let mut piece = |s: &str| {
            let start = text.len() as u32;
            text.push_str(s);
            TextRef::Span(Span::new(start, text.len() as u32))
        };
        let expect = case.map(|c| Expectation {
            exit: c.exit,
            lines: c.lines.map(&mut piece),
            prose: c.prose.then(|| piece("Expected: all pass.")),
            comments: c
                .comments
                .then(|| piece("git branch --show-current   # main")),
        });
        let source = PlanSource::from_normalized("t.md", text, String::new());
        let line = |l: &&str| OutLine {
            stream: Stream::Stdout,
            text: (*l).to_owned(),
            newline: true,
        };
        let run = ScriptRun {
            status,
            errored,
            lines: output.iter().map(line).collect(),
            cwd: None,
        };
        match check(expect.as_ref(), &run, &source) {
            Verdict::Pass => "pass".to_owned(),
            Verdict::Mismatch(m) => format!("mismatch {m:?}"),
            Verdict::Confirm(c) => format!("confirm {:?} errored={}", c.reasons, c.errored),
        }
    }

    #[test]
    fn verdicts() {
        use ExitExpect::{Code, Failure, Success, Unknown};
        use RunStatus::{Broken, Exited, SessionEnded, Threw, TimedOut};
        let exit = |exit| {
            Some(Case {
                exit,
                lines: None,
                prose: false,
                comments: false,
            })
        };
        let lines = |exit, lines| {
            Some(Case {
                exit,
                lines: Some(lines),
                prose: false,
                comments: false,
            })
        };
        let prose = |exit| {
            Some(Case {
                exit,
                lines: None,
                prose: true,
                comments: false,
            })
        };
        let comments = |prose| {
            Some(Case {
                exit: Unknown,
                lines: None,
                prose,
                comments: true,
            })
        };
        let summary = "ok: …\n\n   \ndone\r\n";
        #[rustfmt::skip]
        let rows: Vec<Row> = vec![
            (None, Exited(0), false, &[], "pass"),
            (None, Exited(1), false, &[], "confirm [UncheckedExit] errored=false"),
            (None, Threw, true, &[], "confirm [UncheckedExit, Errored] errored=true"),
            (None, Exited(0), true, &[], "confirm [Errored] errored=true"),
            (None, TimedOut, false, &[], "mismatch [Exit { expected: None, got: TimedOut }]"),
            (exit(Success), Exited(0), false, &[], "pass"),
            (exit(Success), Exited(0), true, &[], "confirm [Errored] errored=true"),
            (exit(Success), Exited(2), false, &[], "mismatch [Exit { expected: Some(Success), got: Exited(2) }]"),
            (exit(Success), Threw, false, &[], "mismatch [Exit { expected: Some(Success), got: Threw }]"),
            (exit(Failure), Exited(101), false, &[], "pass"),
            (exit(Failure), Threw, true, &[], "pass"),
            (exit(Failure), Exited(0), false, &[], "mismatch [Exit { expected: Some(Failure), got: Exited(0) }]"),
            (exit(Failure), SessionEnded(Some(1)), false, &[], "mismatch [Exit { expected: Some(Failure), got: SessionEnded(Some(1)) }]"),
            (exit(Code(1)), Exited(1), false, &[], "pass"),
            (exit(Code(1)), Exited(0), false, &[], "mismatch [Exit { expected: Some(Code(1)), got: Exited(0) }]"),
            (lines(Success, summary), Exited(0), true, &["x", "ok: 3", "noise", "done   "], "pass"),
            (lines(Success, summary), Exited(0), false, &["\u{1b}[32mok: 3\u{1b}[0m", "done"], "pass"),
            (lines(Success, summary), Exited(0), false, &["done", "ok: 3"], "mismatch [Line { number: 4, line: \"done\\r\" }]"),
            (lines(Code(2), summary), Exited(1), false, &["ok: 3", "done"], "mismatch [Exit { expected: Some(Code(2)), got: Exited(1) }]"),
            (lines(Unknown, summary), Exited(1), false, &["ok: 3", "done"], "confirm [UncheckedExit] errored=false"),
            (prose(Success), Exited(0), true, &[], "confirm [Prose] errored=true"),
            (prose(Success), Exited(1), false, &[], "mismatch [Exit { expected: Some(Success), got: Exited(1) }]"),
            (prose(Unknown), Exited(1), false, &[], "confirm [Prose, UncheckedExit] errored=false"),
            (comments(false), Exited(0), false, &[], "confirm [Comments] errored=false"),
            (comments(true), Exited(0), false, &[], "confirm [Prose, Comments] errored=false"),
            (comments(false), Broken("x".into()), false, &[], "mismatch [Exit { expected: Some(Unknown), got: Broken(\"x\") }]"),
        ];
        for (case, status, errored, output, expected) in rows {
            let shown = format!("{status:?} errored={errored} {output:?}");
            assert_eq!(verdict(case, status, errored, output), expected, "{shown}");
        }
    }
}
```


File `crates/planrun-exec/src/lib.rs` (full contents):

```rust
//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Modules (docs/roadmap.md; P3 built the first eight, P4 the next three):
//!
//! - [`workspace`]: the repository root, [`RepoPath`](planrun_model::RepoPath) resolution with
//!   the link and case check, and the one writer (temp file, second check, rename; D52).
//! - [`eol`]: the line endings of written files (D10, D54).
//! - [`git`]: a thin wrapper over the `git` binary: `apply` with the patch on stdin,
//!   `hash-object`, `status --porcelain` without optional locks, `rev-parse`, `cat-file`,
//!   `check-attr`, `update-index`, `ls-files`; [`BlobId`](git::BlobId).
//! - [`tree`]: the virtual tree, a temporary index and object directory (D52).
//! - [`handler`]: one [`ActionHandler`] per action kind, which simulates it on a virtual tree;
//!   the [`HandlerSet`] is the execution-side plugin point.
//! - [`transaction`]: a step's file actions as prepare, then commit, with rollback (D53).
//! - [`preflight`]: task preflight and the dry run over one virtual tree (D50, D55).
//! - [`finding`]: what preflight, a simulation and the dry run report.
//! - [`shell`]: persistent pwsh and bash sessions behind [`ShellSession`] (D60–D65).
//! - [`expect`]: a run against its expectation: exit codes, summary lines with `…`, what asks
//!   for confirmation (D11, D66).
//! - `policy`: command classes and the [`CommandClassifier`].
//! - `journal`: JSONL under `.git/planrun/`.
//! - `runner`: the state machine Task → Step → Action, talking to a [`Ui`].

pub mod eol;
pub mod expect;
pub mod finding;
pub mod git;
pub mod handler;
pub mod preflight;
pub mod shell;
pub mod transaction;
pub mod tree;
pub mod workspace;

pub use handler::{ActionHandler, HandlerSet};
pub use shell::{ScriptRun, ShellSession};

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "../tests/support/ir.rs"]
mod ir;

/// The command classes of decision D13, least to most dangerous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PolicyClass {
    /// Reads only (`git status`, `rg`).
    Read,
    /// Builds or tests (`cargo test`, `dotnet build`, `npm ci`).
    Build,
    /// Changes the repository (`git add`, `git commit`, `replace … -All`, file actions).
    Modify,
    /// Destroys or reaches outside (`Remove-Item -Recurse`, `git reset --hard`, `git push`,
    /// anything unclassified).
    Destructive,
}

/// Classifies a script: the most dangerous class of any command in it.
pub trait CommandClassifier: Send + Sync {
    /// The class of `script` run by `shell`.
    fn classify(&self, shell: planrun_model::Shell, script: &str) -> PolicyClass;
}

/// The state of an action's effects on disk, for resuming (decision D14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoneState {
    /// Every effect is present (postimages match).
    Done,
    /// No effect is present (preimages match).
    NotDone,
    /// Some are, some are not, or neither matches: the user decides.
    Unknown,
}

/// The interactive surface the runner talks to (decision D15: line prompts first).
pub trait Ui {
    /// Shows a line of progress.
    fn status(&mut self, line: &str);
}
```


**Step 2 — verify**

Targeted: the matcher's tables:

```powershell
cargo test -p planrun-exec --lib expect
```


Expected (exit code 0; summary lines):

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in …s
```


Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```


Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```


**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-exec/src/expect.rs crates/planrun-exec/src/lib.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/expect.rs' = '4e09c23328ef4ffe444c2151d24a476cea077490'
      'crates/planrun-exec/src/lib.rs' = '9ae3ff9748adf64ae700a407739c50787c92b184'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P4-T3: the expectation matcher: exit codes, summary lines with …, what asks for confirmation" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```


Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P4-T4: the command classifier: a splitter per shell and the built-in tables

**Goal:** `DefaultClassifier`: a script's class from a splitter per shell and the tables in `policy/heads.rs` (D68). pwsh: dashes, smart quotes, aliases, parameter prefixes, splatting, the call operator, .NET calls, here-strings, hashtables. bash: quoting, substitutions, heredocs, arrays, keywords, environment prefixes. git, cargo, npm, dotnet and go with their options before the subcommand. What the splitter cannot take apart is Destructive. `explain` gives the reasons, for the dry run's tests and the local-corpus test.

**Files**

- Modify `crates/planrun-exec/src/lib.rs`
- Create `crates/planrun-exec/src/policy/bash.rs`
- Create `crates/planrun-exec/src/policy/heads.rs`
- Create `crates/planrun-exec/src/policy/mod.rs`
- Create `crates/planrun-exec/src/policy/pwsh.rs`
- Create `crates/planrun-exec/src/policy/tests.rs`

**Step 1 — write the files**

File `crates/planrun-exec/src/lib.rs` (full contents):

```rust
//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Modules (docs/roadmap.md; P3 built the first eight, P4 the next three):
//!
//! - [`workspace`]: the repository root, [`RepoPath`](planrun_model::RepoPath) resolution with
//!   the link and case check, and the one writer (temp file, second check, rename; D52).
//! - [`eol`]: the line endings of written files (D10, D54).
//! - [`git`]: a thin wrapper over the `git` binary: `apply` with the patch on stdin,
//!   `hash-object`, `status --porcelain` without optional locks, `rev-parse`, `cat-file`,
//!   `check-attr`, `update-index`, `ls-files`; [`BlobId`](git::BlobId).
//! - [`tree`]: the virtual tree, a temporary index and object directory (D52).
//! - [`handler`]: one [`ActionHandler`] per action kind, which simulates it on a virtual tree;
//!   the [`HandlerSet`] is the execution-side plugin point.
//! - [`transaction`]: a step's file actions as prepare, then commit, with rollback (D53).
//! - [`preflight`]: task preflight and the dry run over one virtual tree (D50, D55).
//! - [`finding`]: what preflight, a simulation and the dry run report.
//! - [`shell`]: persistent pwsh and bash sessions behind [`ShellSession`] (D60–D65).
//! - [`expect`]: a run against its expectation: exit codes, summary lines with `…`, what asks
//!   for confirmation (D11, D66).
//! - [`policy`]: command classes and the [`CommandClassifier`]: a built-in allowlist over a
//!   splitter per shell (D13, D68).
//! - `journal`: JSONL under `.git/planrun/`.
//! - `runner`: the state machine Task → Step → Action, talking to a [`Ui`].

pub mod eol;
pub mod expect;
pub mod finding;
pub mod git;
pub mod handler;
pub mod policy;
pub mod preflight;
pub mod shell;
pub mod transaction;
pub mod tree;
pub mod workspace;

pub use handler::{ActionHandler, HandlerSet};
pub use policy::{CommandClassifier, PolicyClass};
pub use shell::{ScriptRun, ShellSession};

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "../tests/support/ir.rs"]
mod ir;

/// The state of an action's effects on disk, for resuming (decision D14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoneState {
    /// Every effect is present (postimages match).
    Done,
    /// No effect is present (preimages match).
    NotDone,
    /// Some are, some are not, or neither matches: the user decides.
    Unknown,
}

/// The interactive surface the runner talks to (decision D15: line prompts first).
pub trait Ui {
    /// Shows a line of progress.
    fn status(&mut self, line: &str);
}
```


File `crates/planrun-exec/src/policy/bash.rs` (full contents):

```rust
//! The bash splitter (decision D68). It walks the script once and recurses into `( … )`
//! subshells, `$( … )` and backticks (also inside double quotes and unquoted heredocs), and
//! `<( … )`/`>( … )`. Single quotes and quoted heredocs are data. `case`, `${…}` with a command
//! inside, an unterminated quote or heredoc, and an unmatched `)` are errors, which make the
//! whole script Destructive.

use super::{Acc, PolicyClass, Word, heads, redirect_target};

/// Reserved words that only introduce or close a compound command: what follows them is a
/// command of its own.
#[rustfmt::skip]
const LEADING: &[&str] = &[
    "!", "do", "done", "elif", "else", "esac", "fi", "if", "then", "time", "until", "while",
    "{", "}",
];

type Parsed = Result<(), String>;

/// Classifies a bash script into `acc`.
pub(super) fn classify(script: &str, acc: &mut Acc) -> Parsed {
    let chars: Vec<char> = script.chars().collect();
    let mut parser = Parser {
        c: &chars,
        i: 0,
        acc,
        heredocs: Vec::new(),
    };
    parser.statements(None)?;
    if !parser.heredocs.is_empty() {
        return Err("a heredoc without its body".to_owned());
    }
    Ok(())
}

/// A heredoc whose body starts after the current line.
struct Heredoc {
    delimiter: String,
    strip_tabs: bool,
    expand: bool,
}

struct Parser<'a> {
    c: &'a [char],
    i: usize,
    acc: &'a mut Acc,
    heredocs: Vec<Heredoc>,
}

fn is_name(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl Parser<'_> {
    fn peek(&self, k: usize) -> Option<char> {
        self.c.get(self.i + k).copied()
    }

    fn statements(&mut self, close: Option<char>) -> Parsed {
        let mut words: Vec<Word> = Vec::new();
        loop {
            self.blank();
            let Some(c) = self.peek(0) else {
                self.command(std::mem::take(&mut words));
                return match close {
                    Some(close) => Err(format!("`{close}` is missing")),
                    None => Ok(()),
                };
            };
            if Some(c) == close {
                self.command(std::mem::take(&mut words));
                self.i += 1;
                return Ok(());
            }
            let next = self.peek(1);
            match c {
                ')' => return Err("an unmatched `)`".to_owned()),
                '\n' => {
                    self.command(std::mem::take(&mut words));
                    self.i += 1;
                    self.heredoc_bodies()?;
                }
                ';' if next == Some(';') => return Err("`;;` (`case`) is not supported".to_owned()),
                '&' if next == Some('>') => self.redirect()?,
                ';' | '&' | '|' => {
                    self.command(std::mem::take(&mut words));
                    self.i += if matches!(next, Some('&' | '|')) && c != ';' {
                        2
                    } else {
                        1
                    };
                }
                '#' => {
                    while self.peek(0).is_some_and(|c| c != '\n') {
                        self.i += 1;
                    }
                }
                '\\' if next == Some('\n') => self.i += 2,
                '(' if next == Some('(') && words.is_empty() => self.arithmetic()?,
                '(' if words.is_empty() => {
                    self.i += 1;
                    self.statements(Some(')'))?;
                }
                '(' if next == Some(')') => {
                    // `name() { … }`: a function definition; its body follows as commands.
                    words.clear();
                    self.i += 2;
                }
                '(' => return Err("an unexpected `(`".to_owned()),
                '<' | '>' => self.redirect()?,
                _ if self.fd_redirect() => self.redirect()?,
                _ => {
                    let word = self.word()?;
                    words.push(word);
                }
            }
        }
    }

    fn blank(&mut self) {
        while matches!(self.peek(0), Some(' ' | '\t' | '\r')) {
            self.i += 1;
        }
    }

    /// `2>`, `10<`: digits right before a redirection operator.
    fn fd_redirect(&self) -> bool {
        let mut k = 0;
        while self.peek(k).is_some_and(|c| c.is_ascii_digit()) {
            k += 1;
        }
        k > 0 && matches!(self.peek(k), Some('<' | '>'))
    }

    /// `(( … ))` as a command: arithmetic, Read.
    fn arithmetic(&mut self) -> Parsed {
        self.i += 2;
        let mut depth = 0;
        while let Some(c) = self.peek(0) {
            self.i += 1;
            match c {
                '(' => depth += 1,
                ')' if depth > 0 => depth -= 1,
                ')' if self.peek(0) == Some(')') => {
                    self.i += 1;
                    return Ok(());
                }
                _ => {}
            }
        }
        Err("an unterminated `((`".to_owned())
    }

    fn redirect(&mut self) -> Parsed {
        while self.peek(0).is_some_and(|c| c.is_ascii_digit() || c == '&') {
            self.i += 1;
        }
        let op = self.peek(0);
        let next = self.peek(1);
        self.i += 1;
        if next == Some('(') {
            self.i += 1;
            return self.statements(Some(')'));
        }
        if op == Some('<') {
            return self.input(next);
        }
        let mut dup = false;
        match next {
            Some('>' | '|') => self.i += 1,
            Some('&') => {
                self.i += 1;
                dup = true;
            }
            _ => {}
        }
        self.blank();
        let target = self.word()?;
        if target.text.is_empty() {
            return Err("a redirection without a target".to_owned());
        }
        let fd = target.literal && target.text.chars().all(|c| c.is_ascii_digit() || c == '-');
        if dup && fd {
            return Ok(());
        }
        redirect_target(&target, self.acc);
        Ok(())
    }

    /// `<`, `<>`, `<&n`, `<<<`, and heredocs (`<<`, `<<-`).
    fn input(&mut self, next: Option<char>) -> Parsed {
        let heredoc = next == Some('<') && self.peek(1) != Some('<');
        match next {
            Some('<') if !heredoc => self.i += 2,
            Some('<' | '&') => self.i += 1,
            Some('>') => {
                self.i += 1;
                self.blank();
                let target = self.word()?;
                redirect_target(&target, self.acc);
                return Ok(());
            }
            _ => {}
        }
        let strip_tabs = heredoc && self.peek(0) == Some('-');
        if strip_tabs {
            self.i += 1;
        }
        self.blank();
        let word = self.word()?;
        if word.text.is_empty() {
            return Err("a redirection without a target".to_owned());
        }
        if heredoc {
            self.heredocs.push(Heredoc {
                delimiter: word.text,
                strip_tabs,
                expand: !word.quoted,
            });
        }
        Ok(())
    }

    /// The bodies of the heredocs opened on the line that just ended.
    fn heredoc_bodies(&mut self) -> Parsed {
        for doc in std::mem::take(&mut self.heredocs) {
            let mut closed = false;
            while self.i < self.c.len() {
                let end = self.c[self.i..]
                    .iter()
                    .position(|&c| c == '\n')
                    .map_or(self.c.len(), |k| self.i + k);
                let line: String = self.c[self.i..end].iter().collect();
                let start = self.i;
                self.i = (end + 1).min(self.c.len());
                let text = if doc.strip_tabs {
                    line.trim_start_matches('\t')
                } else {
                    &line
                };
                if text.trim_end_matches('\r') == doc.delimiter {
                    closed = true;
                    break;
                }
                if doc.expand {
                    self.expansions(start, end)?;
                }
            }
            if !closed {
                return Err(format!("an unterminated heredoc `{}`", doc.delimiter));
            }
        }
        Ok(())
    }

    /// The `$( … )` and backticks in the characters `start..end` of an expanding heredoc.
    fn expansions(&mut self, start: usize, end: usize) -> Parsed {
        let resume = self.i;
        self.i = start;
        let mut scratch = Word::default();
        while self.i < end {
            match (self.peek(0), self.peek(1)) {
                (Some('\\'), _) => self.i += 2,
                (Some('$'), _) => self.dollar(&mut scratch)?,
                (Some('`'), _) => self.backtick(&mut scratch)?,
                _ => self.i += 1,
            }
        }
        self.i = resume;
        Ok(())
    }

    fn word(&mut self) -> Result<Word, String> {
        let mut w = Word {
            literal: true,
            ..Word::default()
        };
        while let Some(c) = self.peek(0) {
            match c {
                ' ' | '\t' | '\r' | '\n' | ';' | '&' | '|' | ')' | '<' | '>' => break,
                '(' if w.text.ends_with('=') => self.array(&mut w)?,
                '(' => break,
                '\'' => {
                    self.i += 1;
                    self.single(&mut w)?;
                }
                '"' => {
                    self.i += 1;
                    self.double(&mut w)?;
                }
                '$' if self.peek(1) == Some('\'') => {
                    self.i += 2;
                    self.ansi_c(&mut w)?;
                }
                '$' if self.peek(1) == Some('"') => {
                    self.i += 2;
                    self.double(&mut w)?;
                }
                '$' => self.dollar(&mut w)?,
                '`' => self.backtick(&mut w)?,
                '\\' => {
                    self.i += 1;
                    match self.peek(0) {
                        Some('\n') => self.i += 1,
                        Some(c) => {
                            w.text.push(c);
                            w.quoted = true;
                            self.i += 1;
                        }
                        None => {}
                    }
                }
                _ => {
                    w.text.push(c);
                    self.i += 1;
                }
            }
        }
        Ok(w)
    }

    /// `name=( … )`: the words are data, their expansions are not.
    fn array(&mut self, w: &mut Word) -> Parsed {
        self.i += 1;
        loop {
            while matches!(self.peek(0), Some(' ' | '\t' | '\r' | '\n')) {
                self.i += 1;
            }
            match self.peek(0) {
                None => return Err("an unterminated array".to_owned()),
                Some(')') => {
                    self.i += 1;
                    w.literal = false;
                    w.text.push_str("(…)");
                    return Ok(());
                }
                Some(_) => {
                    let element = self.word()?;
                    if element.text.is_empty() {
                        return Err("an unexpected character in an array".to_owned());
                    }
                }
            }
        }
    }

    fn single(&mut self, w: &mut Word) -> Parsed {
        w.quoted = true;
        while let Some(c) = self.peek(0) {
            self.i += 1;
            if c == '\'' {
                return Ok(());
            }
            w.text.push(c);
        }
        Err("an unterminated single-quoted string".to_owned())
    }

    fn ansi_c(&mut self, w: &mut Word) -> Parsed {
        w.quoted = true;
        while let Some(c) = self.peek(0) {
            self.i += 1;
            match c {
                '\\' => self.i += 1,
                '\'' => return Ok(()),
                _ => w.text.push(c),
            }
        }
        Err("an unterminated $'…' string".to_owned())
    }

    fn double(&mut self, w: &mut Word) -> Parsed {
        w.quoted = true;
        while let Some(c) = self.peek(0) {
            match c {
                '"' => {
                    self.i += 1;
                    return Ok(());
                }
                '\\' => {
                    w.text.extend(self.peek(1));
                    self.i += 2;
                }
                '$' => self.dollar(w)?,
                '`' => self.backtick(w)?,
                _ => {
                    w.text.push(c);
                    self.i += 1;
                }
            }
        }
        Err("an unterminated double-quoted string".to_owned())
    }

    /// `$( … )`, `$(( … ))`, `${…}`, `$name`, `$1`, `$?`, or a lone `$`.
    fn dollar(&mut self, w: &mut Word) -> Parsed {
        match self.peek(1) {
            Some('(') if self.peek(2) == Some('(') => {
                self.i += 1;
                self.arithmetic()?;
                dynamic(w, "$((…))");
            }
            Some('(') => {
                self.i += 2;
                self.statements(Some(')'))?;
                dynamic(w, "$(…)");
            }
            Some('{') => {
                self.i += 2;
                let mut depth = 1;
                while let Some(c) = self.peek(0) {
                    self.i += 1;
                    match c {
                        '{' => depth += 1,
                        '}' if depth == 1 => {
                            dynamic(w, "${…}");
                            return Ok(());
                        }
                        '}' => depth -= 1,
                        '$' if self.peek(0) == Some('(') => {
                            return Err("a command inside `${…}`".to_owned());
                        }
                        '`' => return Err("a command inside `${…}`".to_owned()),
                        _ => {}
                    }
                }
                return Err("an unterminated `${`".to_owned());
            }
            Some('?' | '#' | '@' | '*' | '!' | '$' | '-') => {
                self.i += 2;
                dynamic(w, "$?");
            }
            next if is_name(next) => {
                self.i += 1;
                dynamic(w, "$");
                while is_name(self.peek(0)) {
                    w.text.extend(self.peek(0));
                    self.i += 1;
                }
            }
            _ => {
                w.text.push('$');
                self.i += 1;
            }
        }
        Ok(())
    }

    /// `` `…` ``: its text is a script of its own.
    fn backtick(&mut self, w: &mut Word) -> Parsed {
        self.i += 1;
        let mut inner = String::new();
        while let Some(c) = self.peek(0) {
            self.i += 1;
            match c {
                '`' => {
                    classify(&inner, self.acc)?;
                    dynamic(w, "`…`");
                    return Ok(());
                }
                '\\' if matches!(self.peek(0), Some('`' | '\\' | '$')) => {
                    inner.extend(self.peek(0));
                    self.i += 1;
                }
                _ => inner.push(c),
            }
        }
        Err("an unterminated backtick".to_owned())
    }

    fn command(&mut self, words: Vec<Word>) {
        let lead = words
            .iter()
            .take_while(|w| w.bare().is_some_and(|t| LEADING.contains(&t)))
            .count();
        let words = &words[lead..];
        let Some(head) = words.first() else {
            return;
        };
        match head.bare() {
            Some("for" | "select" | "[[" | "in") => return,
            Some("function") => return self.command(words.iter().skip(2).cloned().collect()),
            Some("case") => {
                let reason = "`case` is not supported".to_owned();
                return self.acc.add(PolicyClass::Destructive, Some(reason));
            }
            _ => {}
        }
        let assignments = words.iter().take_while(|w| assignment(w)).count();
        let Some(head) = words.get(assignments) else {
            return;
        };
        let reason = if assignments > 0 {
            Some("an environment prefix on a command".to_owned())
        } else {
            match head.bare() {
                None => Some("a command whose name is not literal".to_owned()),
                Some(name) if name.contains('/') => {
                    Some(format!("a path-qualified command `{name}`"))
                }
                Some(_) => None,
            }
        };
        if let Some(reason) = reason {
            return self.acc.add(PolicyClass::Destructive, Some(reason));
        }
        let (class, reason) = heads::bash(&head.text, &words[assignments + 1..]);
        self.acc.add(class, reason);
    }
}

/// `NAME=…` or `NAME+=…` with an unquoted name.
fn assignment(w: &Word) -> bool {
    let Some(eq) = w.text.find('=') else {
        return false;
    };
    let name = w.text[..eq].strip_suffix('+').unwrap_or(&w.text[..eq]);
    !name.is_empty()
        && !name.starts_with(|c: char| c.is_ascii_digit())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn dynamic(w: &mut Word, what: &str) {
    w.literal = false;
    w.text.push_str(what);
}
```


File `crates/planrun-exec/src/policy/heads.rs` (full contents):

```rust
//! The command tables (decision D68): which command, with which arguments, is which class.
//! A command that is not here is Destructive. Build may fetch dependencies into caches and
//! ignored directories; Modify changes tracked files, manifests or lockfiles.

use super::PolicyClass::{self, Build, Destructive, Modify, Read};
use super::Word;

/// A class, with the reason when it is Destructive.
pub(super) type Verdict = (PolicyClass, Option<String>);

fn ok(class: PolicyClass) -> Verdict {
    (class, None)
}

fn no(reason: impl Into<String>) -> Verdict {
    (Destructive, Some(reason.into()))
}

/// The DevReady version the pwsh list below was taken from (its agent modes).
pub const DEVREADY_VERSION: &str = "1.19.1";

/// DevReady functions whose agent mode is `safe`: Read.
#[rustfmt::skip]
const DEVREADY_SAFE: &[&str] = &[
    "brief", "decode", "deps", "depsaudit", "depscheck", "depsoutdated", "depswhy", "dimages",
    "dps", "dstats", "encode", "functions", "gitinfo", "json2yaml", "jsonvalidate", "lines",
    "logerrors", "logfilter", "logsearch", "logstats", "markers", "netdns", "nettest", "ports",
    "test-devreadyplan", "tfchanges", "tfdrift", "tferrors", "tfparse", "tfrisk", "tfshow",
    "tfsummary", "todos", "unixtime", "urlinfo", "what", "yaml2json", "yamlvalidate",
];

/// DevReady functions whose mode is `dry-run-only` or `interactive`, and `killport` (safe for
/// DevReady, but it kills a process): Destructive.
#[rustfmt::skip]
const DEVREADY_DESTRUCTIVE: &[&str] = &[
    "dbconnect", "depsclean", "dprune", "gbackup", "gprune", "gtag", "killport", "mergeback",
    "newgo", "newpython", "newrust", "note", "notify", "purge", "rewind", "secrets",
];

/// pwsh aliases, resolved before the table.
#[rustfmt::skip]
const PWSH_ALIASES: &[(&str, &str)] = &[
    ("%", "foreach-object"), ("?", "where-object"), ("ac", "add-content"),
    ("cat", "get-content"), ("cd", "set-location"), ("chdir", "set-location"),
    ("clc", "clear-content"), ("clear", "clear-host"), ("cls", "clear-host"),
    ("compare", "compare-object"), ("copy", "copy-item"), ("cp", "copy-item"),
    ("cpi", "copy-item"), ("del", "remove-item"), ("dir", "get-childitem"),
    ("echo", "write-output"), ("erase", "remove-item"), ("fl", "format-list"),
    ("foreach", "foreach-object"), ("ft", "format-table"), ("fw", "format-wide"),
    ("gc", "get-content"), ("gci", "get-childitem"), ("gcm", "get-command"),
    ("gi", "get-item"), ("gl", "get-location"), ("gm", "get-member"),
    ("gp", "get-itemproperty"), ("group", "group-object"), ("gv", "get-variable"),
    ("iex", "invoke-expression"), ("irm", "invoke-restmethod"), ("iwr", "invoke-webrequest"),
    ("ls", "get-childitem"), ("md", "mkdir"), ("measure", "measure-object"),
    ("mi", "move-item"), ("move", "move-item"), ("mv", "move-item"), ("ni", "new-item"),
    ("popd", "pop-location"), ("pushd", "push-location"), ("pwd", "get-location"),
    ("rd", "remove-item"), ("ren", "rename-item"), ("ri", "remove-item"),
    ("rm", "remove-item"), ("rmdir", "remove-item"), ("rni", "rename-item"),
    ("rv", "remove-variable"), ("saps", "start-process"), ("select", "select-object"),
    ("set", "set-variable"), ("sl", "set-location"), ("sleep", "start-sleep"),
    ("sort", "sort-object"), ("start", "start-process"), ("sv", "set-variable"),
    ("tee", "tee-object"), ("type", "get-content"), ("where", "where-object"),
    ("write", "write-output"),
];

/// pwsh cmdlets and functions that only read, print or change the session's own state.
#[rustfmt::skip]
const PWSH_READ: &[&str] = &[
    "add-member", "clear-host", "clear-variable", "compare-object", "convertfrom-csv",
    "convertfrom-json", "convertfrom-stringdata", "convertto-csv", "convertto-json",
    "format-custom", "format-list", "format-table", "format-wide", "get-alias",
    "get-childitem", "get-command", "get-content", "get-date", "get-filehash", "get-help",
    "get-item", "get-itemproperty", "get-location", "get-member", "get-module", "get-process",
    "get-random", "get-unique", "get-variable", "group-object", "join-path", "join-string",
    "measure-object", "new-guid", "new-timespan", "new-variable", "out-default", "out-host",
    "out-null", "out-string", "pop-location", "push-location", "remove-variable",
    "resolve-path", "select-object", "select-string", "select-xml", "set-location",
    "set-variable", "sort-object", "split-path", "start-sleep", "test-json", "test-path",
    "where-object", "write-debug", "write-error", "write-host", "write-information",
    "write-output", "write-progress", "write-verbose", "write-warning",
];

/// pwsh cmdlets that write files in place.
#[rustfmt::skip]
const PWSH_MODIFY: &[&str] = &[
    "add-content", "clear-content", "copy-item", "mkdir", "move-item", "out-file",
    "rename-item", "set-content",
];

/// bash commands that only read or print, or change the session's own state.
#[rustfmt::skip]
const BASH_READ: &[&str] = &[
    "[", "basename", "cat", "cd", "cmp", "column", "comm", "cut", "date", "declare", "df",
    "diff", "dirname", "dirs", "du", "echo", "egrep", "exit", "export", "false", "fgrep",
    "file", "grep", "head", "hostname", "id", "jq", "local", "ls", "md5sum", "nl", "od",
    "paste", "popd", "printf", "pushd", "pwd", "read", "readlink", "readonly", "realpath",
    "return", "rev", "seq", "set", "sha1sum", "sha256sum", "shift", "shopt", "sleep", "stat",
    "tac", "tail", "test", "tr", "true", "type", "typeset", "uname", "uniq", "unset", "wait",
    "wc", "which", "whoami", "xxd",
];

/// bash commands that write files in place.
const BASH_MODIFY: &[&str] = &["chmod", "cp", "mkdir", "mv", "rmdir", "tee", "touch"];

/// A pwsh command by its name (lowercase, `.exe`/`.cmd` already dropped).
pub(super) fn pwsh(name: &str, args: &[Word]) -> Verdict {
    let name = PWSH_ALIASES
        .iter()
        .find(|(alias, _)| *alias == name)
        .map_or(name, |(_, cmdlet)| cmdlet);
    if let Some(verdict) = tool(name, args) {
        return verdict;
    }
    if PWSH_READ.contains(&name) || DEVREADY_SAFE.contains(&name) {
        return ok(Read);
    }
    if PWSH_MODIFY.contains(&name) {
        return ok(Modify);
    }
    match name {
        "remove-item" => switched(args, Modify, &["recurse"], "Remove-Item -Recurse"),
        "new-item" => new_item(args),
        "tee-object" => tee_object(args),
        "foreach-object" => foreach_object(args),
        "replace" => replace(args),
        "test" | "testfailed" => ok(Build),
        _ if DEVREADY_DESTRUCTIVE.contains(&name) => no(format!("`{name}`")),
        _ => no(format!("unknown command `{name}`")),
    }
}

/// A bash command by its name.
pub(super) fn bash(name: &str, args: &[Word]) -> Verdict {
    if let Some(verdict) = tool(name, args) {
        return verdict;
    }
    if BASH_READ.contains(&name) {
        return ok(Read);
    }
    if BASH_MODIFY.contains(&name) {
        return ok(Modify);
    }
    match name {
        "rm" if args
            .iter()
            .any(|a| short(a, &['r', 'R']) || long(a, "--recursive")) =>
        {
            no("rm -r")
        }
        "rm" => ok(Modify),
        "sed"
            if args
                .iter()
                .any(|a| short(a, &['i']) || long(a, "--in-place")) =>
        {
            ok(Modify)
        }
        "sed" => ok(Read),
        "sort" if args.iter().any(|a| short(a, &['o']) || long(a, "--output")) => ok(Modify),
        "sort" => ok(Read),
        "find" => find(args),
        "command"
            if args
                .first()
                .and_then(Word::bare)
                .is_some_and(|a| a == "-v" || a == "-V") =>
        {
            ok(Read)
        }
        "env" if args.is_empty() => ok(Read),
        _ => no(format!("unknown command `{name}`")),
    }
}

/// The tools both shells share.
fn tool(name: &str, args: &[Word]) -> Option<Verdict> {
    let verdict = match name {
        "git" => git(args),
        "cargo" => cargo(args),
        "npm" => npm(args),
        "dotnet" => dotnet(args),
        "go" => go(args),
        "rg" if args.iter().any(|a| long(a, "--pre")) => no("rg --pre"),
        "rg" | "jq" => ok(Read),
        _ => return None,
    };
    Some(verdict)
}

/// A bash short-option cluster (`-rf`) that holds one of `letters`.
fn short(arg: &Word, letters: &[char]) -> bool {
    arg.bare().is_some_and(|a| {
        a.starts_with('-') && !a.starts_with("--") && a[1..].chars().any(|c| letters.contains(&c))
    })
}

/// `--name` or `--name=…`.
fn long(arg: &Word, name: &str) -> bool {
    arg.bare()
        .is_some_and(|a| a == name || a.strip_prefix(name).is_some_and(|r| r.starts_with('=')))
}

/// A pwsh parameter matches `name` as pwsh matches it: any prefix, any case, with or without
/// `:value` (`-r`, `-Rec`, `-Recurse:$true`).
fn param(arg: &Word, name: &str) -> bool {
    if arg.quoted {
        return false;
    }
    let Some(given) = arg.text.strip_prefix('-') else {
        return false;
    };
    let (given, value) = given.split_once(':').unwrap_or((given, ""));
    let given = given.to_ascii_lowercase();
    let literal = arg.literal || !value.is_empty();
    literal && !given.is_empty() && name.starts_with(&given)
}

/// A splatted argument (`@args`): its parameters are unknown.
fn splat(arg: &Word) -> bool {
    !arg.literal && !arg.quoted && arg.text.starts_with('@')
}

/// `base`, unless one of `switches` (lowercase parameter names) is given, or splatted.
fn switched(args: &[Word], base: PolicyClass, switches: &[&str], what: &str) -> Verdict {
    let given = args
        .iter()
        .any(|a| splat(a) || switches.iter().any(|s| param(a, s)));
    if given {
        return no(what);
    }
    ok(base)
}

fn new_item(args: &[Word]) -> Verdict {
    let link = args.windows(2).any(|w| {
        param(&w[0], "itemtype")
            && ["symboliclink", "junction", "hardlink"]
                .iter()
                .any(|t| w[1].text.eq_ignore_ascii_case(t))
    });
    if link || args.iter().any(splat) {
        return no("New-Item of a link");
    }
    ok(Modify)
}

fn tee_object(args: &[Word]) -> Verdict {
    let file = args.iter().any(|a| {
        ["filepath", "path", "literalpath"]
            .iter()
            .any(|p| param(a, p))
    });
    ok(if file { Modify } else { Read })
}

/// `ForEach-Object` with script blocks reads; with a member name it calls a method by name.
fn foreach_object(args: &[Word]) -> Verdict {
    let by_name = args.iter().any(|a| {
        param(a, "membername")
            || splat(a)
            || a.quoted
            || a.bare().is_some_and(|t| !t.starts_with('-'))
    });
    if by_name {
        return no("ForEach-Object with a member name");
    }
    ok(Read)
}

/// DevReady `replace`: `-DryRun` previews, `-All` writes, neither stops to ask.
fn replace(args: &[Word]) -> Verdict {
    if args.iter().any(|a| param(a, "all")) {
        return ok(Modify);
    }
    if args.iter().any(|a| param(a, "dryrun")) {
        return ok(Read);
    }
    no("replace without -DryRun or -All")
}

fn find(args: &[Word]) -> Verdict {
    let acting = [
        "-delete", "-exec", "-execdir", "-ok", "-okdir", "-fprint", "-fprint0", "-fprintf", "-fls",
    ];
    match args
        .iter()
        .find_map(|a| a.bare().filter(|t| acting.contains(t)))
    {
        Some(action) => no(format!("find {action}")),
        None => ok(Read),
    }
}

/// Literal option words before the subcommand; `takes` lists the options that take a value.
/// Returns the subcommand's index, or the Destructive verdict for an option not in `skip`.
fn subcommand(tool: &str, args: &[Word], skip: &[&str], takes: &[&str]) -> Result<usize, Verdict> {
    let mut i = 0;
    while let Some(arg) = args.get(i) {
        let Some(text) = arg.bare() else {
            return Err(no(format!("{tool} with a dynamic argument")));
        };
        if !text.starts_with(['-', '+']) {
            return Ok(i);
        }
        let name = text.split('=').next().unwrap_or_default();
        if takes.contains(&name) {
            i += if text.contains('=') { 1 } else { 2 };
            continue;
        }
        if !skip.contains(&text) && !text.starts_with('+') {
            return Err(no(format!("{tool} {text}")));
        }
        i += 1;
    }
    Ok(i)
}

fn has(rest: &[Word], flags: &[&str]) -> bool {
    rest.iter().any(|a| {
        a.bare().is_some_and(|t| {
            flags
                .iter()
                .any(|f| t == *f || t.strip_prefix(f).is_some_and(|r| r.starts_with('=')))
        })
    })
}

const GIT_CONFIG_ALLOWED: &[&str] = &["core.quotepath", "user.name", "user.email"];

fn git(args: &[Word]) -> Verdict {
    let mut i = 0;
    while let Some(text) = args.get(i).map(|a| a.bare()) {
        let Some(text) = text else {
            return no("git with a dynamic argument");
        };
        match text {
            "-C" => i += 2,
            "-c" => {
                let kv = args.get(i + 1).and_then(Word::bare).unwrap_or_default();
                let key = kv
                    .split('=')
                    .next()
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                let allowed = key.starts_with("color.")
                    || key.starts_with("advice.")
                    || GIT_CONFIG_ALLOWED.contains(&key.as_str());
                if !allowed {
                    return no(format!("git -c {key}"));
                }
                i += 2;
            }
            "--no-pager" | "--no-optional-locks" | "--literal-pathspecs" | "-P" => i += 1,
            "--version" | "-v" | "--help" | "-h" => return ok(Read),
            t if t.starts_with('-') => return no(format!("git {t}")),
            _ => break,
        }
    }
    let Some(sub) = args.get(i).and_then(Word::bare) else {
        return no("git without a literal subcommand");
    };
    let rest = &args[i + 1..];
    git_subcommand(sub, rest)
}

fn git_subcommand(sub: &str, rest: &[Word]) -> Verdict {
    let first = rest.first().and_then(Word::bare);
    let positional = rest
        .iter()
        .any(|a| a.bare().is_none_or(|t| !t.starts_with('-')));
    match sub {
        "status" | "rev-parse" | "ls-files" | "ls-tree" | "blame" | "describe" | "cat-file"
        | "shortlog" | "rev-list" | "name-rev" | "merge-base" | "show-ref" | "for-each-ref"
        | "version" | "help" | "var" | "check-ignore" | "check-attr" | "count-objects"
        | "show-branch" | "whatchanged" => ok(Read),
        "grep" if has(rest, &["-O", "--open-files-in-pager"]) => no("git grep -O"),
        "grep" => ok(Read),
        "diff" | "log" | "show" if has(rest, &["--ext-diff"]) => {
            no(format!("git {sub} --ext-diff"))
        }
        "diff" | "log" | "show" if has(rest, &["--output"]) => ok(Modify),
        "diff" | "log" | "show" => ok(Read),
        "hash-object" if has(rest, &["-w"]) => ok(Modify),
        "hash-object" => ok(Read),
        "config" if has(rest, &["--global", "--system", "--file", "-f"]) => {
            no("git config outside the repository")
        }
        "config"
            if has(
                rest,
                &["--get", "--get-all", "--get-regexp", "--list", "-l"],
            ) =>
        {
            ok(Read)
        }
        "config" => ok(Modify),
        "branch" => git_branch(rest, positional),
        "tag" if rest.is_empty() || has(rest, &["-l", "--list", "-n"]) => ok(Read),
        "tag" => ok(Modify),
        "remote" => match first {
            None | Some("-v" | "--verbose" | "show" | "get-url") => ok(Read),
            Some("add" | "rename" | "set-url" | "set-head" | "set-branches") => ok(Modify),
            Some(other) => no(format!("git remote {other}")),
        },
        "stash" => match first {
            None | Some("push" | "save" | "pop" | "apply" | "create" | "store" | "branch") => {
                ok(Modify)
            }
            Some("list" | "show") => ok(Read),
            Some(other) => no(format!("git stash {other}")),
        },
        "worktree" => match first {
            Some("list") => ok(Read),
            Some("add" | "lock" | "unlock" | "move" | "repair") => ok(Modify),
            _ => no("git worktree"),
        },
        "reflog" => match first {
            None | Some("show") => ok(Read),
            Some(other) => no(format!("git reflog {other}")),
        },
        "add" | "commit" | "apply" | "rm" | "mv" | "merge" | "cherry-pick" | "revert" | "notes"
        | "update-index" | "am" | "init" => ok(Modify),
        "switch" if has(rest, &["-f", "--force", "--discard-changes"]) => no("git switch --force"),
        "switch" => ok(Modify),
        "checkout" if has(rest, &["-f", "--force", "--", "-p", "--patch"]) => {
            no("git checkout of paths or with --force")
        }
        "checkout" if has(rest, &["-b", "-B", "--orphan"]) => ok(Modify),
        "checkout" => no("git checkout <name> (the name can be a path)"),
        "reset" if has(rest, &["--hard", "--merge", "--keep"]) => no("git reset --hard"),
        "reset" => ok(Modify),
        _ => no(format!("git {sub}")),
    }
}

fn git_branch(rest: &[Word], positional: bool) -> Verdict {
    if has(rest, &["-D", "-f", "--force", "-M", "-C"]) {
        return no("git branch --force");
    }
    let changes = [
        "-d",
        "--delete",
        "-m",
        "--move",
        "-c",
        "--copy",
        "-u",
        "--set-upstream-to",
        "--unset-upstream",
        "--edit-description",
    ];
    if has(rest, &changes) {
        return ok(Modify);
    }
    if has(rest, &["-l", "--list"]) || !positional {
        return ok(Read);
    }
    ok(Modify)
}

fn cargo(args: &[Word]) -> Verdict {
    if has(args, &["--config"]) {
        return no("cargo --config");
    }
    let skip = [
        "-q",
        "--quiet",
        "-v",
        "-vv",
        "--verbose",
        "--locked",
        "--offline",
        "--frozen",
        "--version",
        "-V",
        "--list",
        "-h",
        "--help",
    ];
    let takes = ["--manifest-path", "--color", "-Z", "-C"];
    let i = match subcommand("cargo", args, &skip, &takes) {
        Ok(i) => i,
        Err(verdict) => return verdict,
    };
    let Some(sub) = args.get(i).and_then(Word::bare) else {
        return ok(Read);
    };
    let rest = &args[i + 1..];
    match sub {
        "build" | "b" | "test" | "t" | "check" | "c" | "doc" | "d" | "bench" | "tree"
        | "metadata" | "verify-project" | "version" | "pkgid" | "locate-project" | "search"
        | "fetch" => ok(Build),
        "clippy" if has(rest, &["--fix"]) => ok(Modify),
        "clippy" => ok(Build),
        "fmt" if has(rest, &["--check"]) => ok(Build),
        "fmt" | "fix" | "add" | "update" | "remove" | "rm" | "generate-lockfile" | "new"
        | "init" | "vendor" => ok(Modify),
        _ => no(format!("cargo {sub}")),
    }
}

fn npm(args: &[Word]) -> Verdict {
    let skip = ["-s", "--silent", "-q", "--quiet", "--no-fund", "--no-audit"];
    let takes = ["--prefix", "-w", "--workspace", "--loglevel"];
    let i = match subcommand("npm", args, &skip, &takes) {
        Ok(i) => i,
        Err(verdict) => return verdict,
    };
    let Some(sub) = args.get(i).and_then(Word::bare) else {
        return ok(Read);
    };
    let rest = &args[i + 1..];
    match sub {
        "audit" if rest.first().and_then(Word::bare) == Some("fix") => ok(Modify),
        "ci" | "test" | "t" | "tst" | "run" | "run-script" | "ls" | "list" | "outdated"
        | "audit" | "view" | "info" | "explain" | "why" => ok(Build),
        "install" | "i" | "add" | "uninstall" | "un" | "remove" | "rm" | "r" | "update" | "up"
        | "upgrade" | "dedupe" => ok(Modify),
        _ => no(format!("npm {sub}")),
    }
}

fn dotnet(args: &[Word]) -> Verdict {
    let skip = ["--info", "--version", "--list-sdks", "--list-runtimes"];
    let i = match subcommand("dotnet", args, &skip, &[]) {
        Ok(i) => i,
        Err(verdict) => return verdict,
    };
    let Some(sub) = args.get(i).and_then(Word::bare) else {
        return ok(Read);
    };
    let rest = &args[i + 1..];
    match sub {
        "build" | "test" | "restore" | "msbuild" | "pack" | "clean" | "vstest" | "list" => {
            ok(Build)
        }
        "format" if has(rest, &["--verify-no-changes"]) => ok(Build),
        "format" | "add" | "remove" | "new" | "sln" => ok(Modify),
        _ => no(format!("dotnet {sub}")),
    }
}

fn go(args: &[Word]) -> Verdict {
    let Some(sub) = args.first().and_then(Word::bare) else {
        return no("go without a literal subcommand");
    };
    let rest = &args[1..];
    let first = rest.first().and_then(Word::bare);
    match sub {
        "env" if has(rest, &["-w", "-u"]) => ok(Modify),
        "build" | "test" | "vet" | "list" | "version" | "env" | "doc" => ok(Build),
        "fmt" | "get" => ok(Modify),
        "mod" => match first {
            Some("tidy" | "edit" | "vendor" | "init") => ok(Modify),
            Some("download" | "graph" | "verify" | "why") => ok(Build),
            _ => no("go mod"),
        },
        _ => no(format!("go {sub}")),
    }
}
```


File `crates/planrun-exec/src/policy/mod.rs` (full contents):

```rust
//! Policy classes and the command classifier (docs/architecture.md §6.5; decisions D13, D68).
//!
//! The classifier is a built-in allowlist: a script's class is the highest class of any command
//! in it, a command the tables do not know is Destructive, and so is a script the splitter
//! cannot take apart with confidence. No format spec, `[checks]` entry or flag changes a class,
//! and a runner has one classifier, not one per format.

mod bash;
mod heads;
mod pwsh;

pub use heads::DEVREADY_VERSION;

use planrun_model::Shell;

/// The command classes of decision D13, least to most dangerous.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolicyClass {
    /// Reads only (`git status`, `rg`).
    #[default]
    Read,
    /// Builds or tests (`cargo test`, `dotnet build`, `npm ci`).
    Build,
    /// Changes the repository (`git add`, `git commit`, `replace … -All`, file actions).
    Modify,
    /// Destroys or reaches outside (`Remove-Item -Recurse`, `git reset --hard`, `git push`,
    /// anything unclassified).
    Destructive,
}

/// Classifies a script: the most dangerous class of any command in it.
pub trait CommandClassifier: Send + Sync {
    /// The class of `script` run by `shell`.
    fn classify(&self, shell: Shell, script: &str) -> PolicyClass;
}

/// A script's class with the reasons that made it Destructive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classification {
    /// The highest class of any command.
    pub class: PolicyClass,
    /// What made it Destructive (`git push`, `unknown command \`foo\``, `unparsed: …`), in
    /// script order; empty below Destructive.
    pub reasons: Vec<String>,
}

/// The built-in classifier (D68), with the tables of `heads.rs`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultClassifier;

impl DefaultClassifier {
    /// The class of `script` and why.
    pub fn explain(&self, shell: Shell, script: &str) -> Classification {
        let mut acc = Acc::default();
        let parsed = match shell {
            Shell::Pwsh => pwsh::classify(script, &mut acc),
            Shell::Bash => bash::classify(script, &mut acc),
        };
        if let Err(why) = parsed {
            acc.add(PolicyClass::Destructive, Some(format!("unparsed: {why}")));
        }
        Classification {
            class: acc.class,
            reasons: acc.reasons,
        }
    }
}

impl CommandClassifier for DefaultClassifier {
    fn classify(&self, shell: Shell, script: &str) -> PolicyClass {
        self.explain(shell, script).class
    }
}

/// One word of a command as the splitter saw it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Word {
    /// Its text: the value without quotes when `literal`, the raw spelling otherwise.
    pub text: String,
    /// No variable, subexpression or other expansion: `text` is exactly the value.
    pub literal: bool,
    /// Some part was quoted (a quoted word is never a parameter or a keyword).
    pub quoted: bool,
}

impl Word {
    /// A literal word that was not quoted.
    pub fn bare(&self) -> Option<&str> {
        (self.literal && !self.quoted).then_some(self.text.as_str())
    }
}

/// What the splitter found so far: the highest class, and the reasons for Destructive.
#[derive(Debug, Default)]
pub(crate) struct Acc {
    pub class: PolicyClass,
    pub reasons: Vec<String>,
}

impl Acc {
    /// Raises the class to `class`; a reason is kept only for Destructive.
    pub fn add(&mut self, class: PolicyClass, reason: Option<String>) {
        self.class = self.class.max(class);
        if class == PolicyClass::Destructive {
            self.reasons
                .push(reason.unwrap_or_else(|| "destructive".to_owned()));
        }
    }
}

/// A redirection target's class (D68): nothing for the null device, Modify for a literal
/// relative path without `..`, Destructive for anything else.
pub(crate) fn redirect_target(word: &Word, acc: &mut Acc) {
    let text = word.text.as_str();
    let null = ["$null", "/dev/null"].contains(&text) || text.eq_ignore_ascii_case("nul");
    if null && (word.literal || text == "$null") {
        return;
    }
    let relative = word.literal
        && !text.is_empty()
        && !text.starts_with(['/', '\\', '~'])
        && !text.contains(':')
        && !text.split(['/', '\\']).any(|s| s == "..");
    if relative {
        acc.add(PolicyClass::Modify, None);
        return;
    }
    acc.add(
        PolicyClass::Destructive,
        Some(format!("a redirection to `{text}`")),
    );
}

#[cfg(test)]
mod tests;
```


File `crates/planrun-exec/src/policy/pwsh.rs` (full contents):

```rust
//! The pwsh splitter (decision D68). It walks the script once, statement by statement and
//! pipeline element by pipeline element, and recurses into every place code can hide: `( … )`,
//! `$( … )`, `@( … )`, `@{ … }`, script blocks, and `$( … )` inside double-quoted strings and
//! `@" … "@`. Single-quoted strings and `@' … '@` are data. Anything it cannot take apart with
//! confidence (an unterminated quote or here-string, an unmatched bracket) is an error, which
//! makes the whole script Destructive.

use super::{Acc, PolicyClass, Word, heads, redirect_target};

#[rustfmt::skip]
const KEYWORDS: &[&str] = &[
    "begin", "break", "catch", "class", "clean", "continue", "do", "dynamicparam", "else",
    "elseif", "end", "enum", "exit", "filter", "finally", "for", "foreach", "function", "if",
    "param", "process", "return", "switch", "throw", "trap", "try", "until", "while",
];

/// Types whose static methods are pure (lowercase, as written in `[ ]`).
#[rustfmt::skip]
const PURE_TYPES: &[&str] = &[
    "io.path", "system.io.path", "math", "system.math", "string", "system.string", "regex",
    "system.text.regularexpressions.regex",
];

/// Instance methods that are pure string methods (lowercase).
#[rustfmt::skip]
const PURE_METHODS: &[&str] = &[
    "contains", "endswith", "replace", "split", "startswith", "substring", "tolower",
    "tostring", "toupper", "trim",
];

type Parsed = Result<(), String>;

/// Classifies a pwsh script into `acc`.
pub(super) fn classify(script: &str, acc: &mut Acc) -> Parsed {
    let chars: Vec<char> = script.chars().collect();
    let mut parser = Parser {
        c: &chars,
        i: 0,
        acc,
    };
    parser.statements(None, false)
}

fn is_single(c: Option<char>) -> bool {
    matches!(c, Some('\'' | '‘' | '’' | '‚' | '‛'))
}

fn is_double(c: Option<char>) -> bool {
    matches!(c, Some('"' | '“' | '”' | '„'))
}

fn is_dash(c: char) -> bool {
    matches!(c, '-' | '–' | '—' | '―')
}

fn is_ident(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_alphanumeric() || c == '_')
}

fn is_assign(text: &str) -> bool {
    matches!(text, "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "??=")
}

struct Parser<'a> {
    c: &'a [char],
    i: usize,
    acc: &'a mut Acc,
}

impl Parser<'_> {
    fn peek(&self, k: usize) -> Option<char> {
        self.c.get(self.i + k).copied()
    }

    /// Statements up to `close` (consumed) or the end of the script. In a hashtable, each
    /// statement is `key = value` and only the value is classified.
    fn statements(&mut self, close: Option<char>, hashtable: bool) -> Parsed {
        let mut words: Vec<Word> = Vec::new();
        let mut first = true;
        loop {
            self.blank();
            let Some(c) = self.peek(0) else {
                self.command(std::mem::take(&mut words), first, hashtable);
                return match close {
                    Some(close) => Err(format!("`{close}` is missing")),
                    None => Ok(()),
                };
            };
            if Some(c) == close {
                self.command(std::mem::take(&mut words), first, hashtable);
                self.i += 1;
                return Ok(());
            }
            let next = self.peek(1);
            match c {
                ')' | '}' | ']' => return Err(format!("an unmatched `{c}`")),
                '\n' | ';' => self.end(&mut words, &mut first, true, 1, hashtable),
                '|' if next == Some('|') => self.end(&mut words, &mut first, true, 2, hashtable),
                '&' if next == Some('&') => self.end(&mut words, &mut first, true, 2, hashtable),
                '|' => self.end(&mut words, &mut first, false, 1, hashtable),
                '#' => self.line_comment(),
                '<' if next == Some('#') => self.block_comment()?,
                '<' => return Err("`<` is not a pwsh operator".to_owned()),
                '&' => {
                    words.push(Word {
                        text: "&".to_owned(),
                        literal: true,
                        quoted: false,
                    });
                    self.i += 1;
                }
                ',' => self.i += 1,
                '`' if next == Some('\n') => self.i += 2,
                '`' if next == Some('\r') && self.peek(2) == Some('\n') => self.i += 3,
                _ if self.redirect_start() => self.redirect()?,
                _ => {
                    let word = self.word()?;
                    words.push(word);
                }
            }
        }
    }

    /// Ends the current command; `statement` starts a new statement, otherwise the next
    /// pipeline element.
    fn end(
        &mut self,
        words: &mut Vec<Word>,
        first: &mut bool,
        statement: bool,
        skip: usize,
        hashtable: bool,
    ) {
        self.command(std::mem::take(words), *first, hashtable);
        *first = statement;
        self.i += skip;
    }

    fn blank(&mut self) {
        while matches!(self.peek(0), Some(' ' | '\t' | '\r' | '\u{c}')) {
            self.i += 1;
        }
    }

    fn line_comment(&mut self) {
        while self.peek(0).is_some_and(|c| c != '\n') {
            self.i += 1;
        }
    }

    fn block_comment(&mut self) -> Parsed {
        self.i += 2;
        while let Some(c) = self.peek(0) {
            self.i += 1;
            if c == '#' && self.peek(0) == Some('>') {
                self.i += 1;
                return Ok(());
            }
        }
        Err("an unterminated `<#` comment".to_owned())
    }

    fn redirect_start(&self) -> bool {
        let c = self.peek(0);
        c == Some('>')
            || (c.is_some_and(|c| c.is_ascii_digit() || c == '*') && self.peek(1) == Some('>'))
    }

    /// `>`, `>>`, `n>`, `*>`, and the merges `n>&1`, which have no file.
    fn redirect(&mut self) -> Parsed {
        if self.peek(0) != Some('>') {
            self.i += 1;
        }
        self.i += 1;
        if self.peek(0) == Some('>') {
            self.i += 1;
        }
        if self.peek(0) == Some('&') && self.peek(1).is_some_and(|c| c.is_ascii_digit()) {
            self.i += 2;
            return Ok(());
        }
        self.blank();
        let target = self.word()?;
        if target.text.is_empty() {
            return Err("a redirection without a target".to_owned());
        }
        redirect_target(&target, self.acc);
        Ok(())
    }

    /// One word: everything up to whitespace or an operator, its parts concatenated.
    fn word(&mut self) -> Result<Word, String> {
        let mut w = Word {
            literal: true,
            ..Word::default()
        };
        let start = self.i;
        let mut value = false;
        let mut expression = false;
        while let Some(c) = self.peek(0) {
            let at_start = self.i == start;
            let next = self.peek(1);
            match c {
                ' ' | '\t' | '\r' | '\n' | ';' | '|' | '&' | ')' | '}' | ']' | ',' => break,
                '=' if expression && !at_start => break,
                '+' | '-' | '*' | '/' | '%' if expression && next == Some('=') => break,
                _ if is_single(Some(c)) => {
                    self.single(&mut w)?;
                    value = true;
                }
                _ if is_double(Some(c)) => {
                    self.double(&mut w)?;
                    value = true;
                }
                '@' if is_single(next) || is_double(next) => {
                    self.here(&mut w)?;
                    value = true;
                }
                '@' if next == Some('(') => {
                    self.i += 2;
                    self.statements(Some(')'), false)?;
                    dynamic(&mut w, "@(…)");
                    value = true;
                }
                '@' if next == Some('{') => {
                    self.i += 2;
                    self.statements(Some('}'), true)?;
                    dynamic(&mut w, "@{…}");
                    value = true;
                }
                '@' if at_start && is_ident(next) => {
                    self.i += 1;
                    dynamic(&mut w, "@");
                    self.ident(&mut w);
                }
                '$' => {
                    self.dollar(&mut w)?;
                    value = true;
                    expression |= at_start;
                }
                '(' => {
                    self.i += 1;
                    self.statements(Some(')'), false)?;
                    dynamic(&mut w, "(…)");
                    value = true;
                    expression |= at_start;
                }
                '{' => {
                    self.i += 1;
                    self.statements(Some('}'), false)?;
                    dynamic(&mut w, "{…}");
                    value = false;
                }
                '[' if at_start => {
                    self.type_literal(&mut w)?;
                    value = true;
                    expression = true;
                }
                '[' if value => {
                    self.i += 1;
                    self.statements(Some(']'), false)?;
                    dynamic(&mut w, "[…]");
                }
                '.' if value && is_ident(next) => self.member(&mut w)?,
                '`' => {
                    self.i += 1;
                    if let Some(c) = self.peek(0).filter(|c| *c != '\n') {
                        w.text.push(c);
                        self.i += 1;
                    }
                    value = false;
                }
                _ => {
                    w.text.push(if at_start && is_dash(c) { '-' } else { c });
                    self.i += 1;
                    value = false;
                }
            }
        }
        Ok(w)
    }

    fn ident(&mut self, w: &mut Word) {
        while is_ident(self.peek(0)) || self.peek(0) == Some(':') {
            if let Some(c) = self.peek(0) {
                w.text.push(c);
            }
            self.i += 1;
        }
    }

    /// `'…'` with `''` for a quote; data.
    fn single(&mut self, w: &mut Word) -> Parsed {
        w.quoted = true;
        self.i += 1;
        while let Some(c) = self.peek(0) {
            self.i += 1;
            if !is_single(Some(c)) {
                w.text.push(c);
                continue;
            }
            if !is_single(self.peek(0)) {
                return Ok(());
            }
            w.text.push(c);
            self.i += 1;
        }
        Err("an unterminated single-quoted string".to_owned())
    }

    /// `"…"`: `` ` `` escapes, `""` for a quote, variables and `$( … )` inside.
    fn double(&mut self, w: &mut Word) -> Parsed {
        w.quoted = true;
        self.i += 1;
        while let Some(c) = self.peek(0) {
            match c {
                '`' => {
                    if let Some(next) = self.peek(1) {
                        w.text.push(next);
                    }
                    self.i += 2;
                }
                '$' if matches!(self.peek(1), Some('(' | '{')) || is_ident(self.peek(1)) => {
                    self.dollar(w)?;
                }
                _ if is_double(Some(c)) => {
                    self.i += 1;
                    if !is_double(self.peek(0)) {
                        return Ok(());
                    }
                    w.text.push(c);
                    self.i += 1;
                }
                _ => {
                    w.text.push(c);
                    self.i += 1;
                }
            }
        }
        Err("an unterminated double-quoted string".to_owned())
    }

    /// `@'…'@` (data) or `@"…"@` (with `$( … )`): the opener ends its line, the closer starts
    /// one.
    fn here(&mut self, w: &mut Word) -> Parsed {
        w.quoted = true;
        let expandable = is_double(self.peek(1));
        self.i += 2;
        self.blank();
        if self.peek(0) != Some('\n') {
            return Err("a here-string whose opener does not end its line".to_owned());
        }
        while let Some(c) = self.peek(0) {
            let closes = if expandable { is_double } else { is_single };
            if c == '\n' && closes(self.peek(1)) && self.peek(2) == Some('@') {
                self.i += 3;
                return Ok(());
            }
            if expandable && c == '$' && self.peek(1) == Some('(') {
                self.dollar(w)?;
                continue;
            }
            if expandable && c == '`' {
                self.i += 1;
            }
            self.i += 1;
        }
        Err("an unterminated here-string".to_owned())
    }

    /// `$( … )`, `${…}`, `$name`, `$scope:name`, or a lone `$`.
    fn dollar(&mut self, w: &mut Word) -> Parsed {
        match self.peek(1) {
            Some('(') => {
                self.i += 2;
                self.statements(Some(')'), false)?;
                dynamic(w, "$(…)");
            }
            Some('{') => {
                self.i += 2;
                while self.peek(0).is_some_and(|c| c != '}') {
                    self.i += 1;
                }
                if self.peek(0).is_none() {
                    return Err("an unterminated `${`".to_owned());
                }
                self.i += 1;
                dynamic(w, "${…}");
            }
            Some('?' | '^' | '$') => {
                self.i += 2;
                dynamic(w, "$?");
            }
            next if is_ident(next) => {
                self.i += 1;
                dynamic(w, "$");
                self.ident(w);
            }
            _ => {
                w.text.push('$');
                self.i += 1;
            }
        }
        Ok(())
    }

    /// `[Type]`, then `::Member` or `::Method( … )`: a static call is Destructive unless the
    /// type is pure.
    fn type_literal(&mut self, w: &mut Word) -> Parsed {
        self.i += 1;
        let mut depth = 1;
        let mut name = String::new();
        while let Some(c) = self.peek(0) {
            self.i += 1;
            match c {
                '[' => depth += 1,
                ']' if depth == 1 => {
                    depth = 0;
                    break;
                }
                ']' => depth -= 1,
                '\n' => break,
                _ => {}
            }
            name.push(c);
        }
        if depth != 0 {
            return Err("a `[` without its `]`".to_owned());
        }
        dynamic(w, "[…]");
        if !(self.peek(0) == Some(':') && self.peek(1) == Some(':')) {
            return Ok(());
        }
        self.i += 2;
        let mut member = String::new();
        while is_ident(self.peek(0)) {
            member.extend(self.peek(0));
            self.i += 1;
        }
        if self.peek(0) != Some('(') {
            return Ok(());
        }
        let name = name.trim().to_ascii_lowercase();
        if !PURE_TYPES.contains(&name.as_str()) {
            let reason = format!("a static call [{name}]::{member}(");
            self.acc.add(PolicyClass::Destructive, Some(reason));
        }
        self.i += 1;
        self.statements(Some(')'), false)
    }

    /// `.Member` or `.Method( … )` after a value: a call is Destructive unless it is a pure
    /// string method.
    fn member(&mut self, w: &mut Word) -> Parsed {
        self.i += 1;
        let mut name = String::new();
        while is_ident(self.peek(0)) {
            name.extend(self.peek(0));
            self.i += 1;
        }
        dynamic(w, ".");
        w.text.push_str(&name);
        if self.peek(0) != Some('(') {
            return Ok(());
        }
        if !PURE_METHODS.contains(&name.to_ascii_lowercase().as_str()) {
            let reason = format!("a method call .{name}(");
            self.acc.add(PolicyClass::Destructive, Some(reason));
        }
        self.i += 1;
        self.statements(Some(')'), false)
    }

    /// Classifies one command (or expression) of a pipeline; `first` when it starts a
    /// statement, where keywords count.
    fn command(&mut self, mut words: Vec<Word>, first: bool, hashtable: bool) {
        if hashtable && words.get(1).and_then(Word::bare) == Some("=") {
            words.drain(..2);
            return self.command(words, true, false);
        }
        let key = words.first().and_then(Word::bare).and_then(|t| t.find('='));
        if let (true, Some(eq)) = (hashtable, key) {
            let value = words[0].text.split_off(eq + 1);
            words[0].text = value;
            if words[0].text.is_empty() {
                words.remove(0);
            }
            return self.command(words, true, false);
        }
        let assigned = words
            .iter()
            .position(|w| w.bare().is_some_and(is_assign))
            .filter(|&k| k > 0 && words[..k].iter().all(|w| w.text.starts_with(['$', '['])));
        if let Some(k) = assigned {
            words.drain(..=k);
            return self.command(words, true, false);
        }
        let Some(head) = words.first() else {
            return;
        };
        let Some(name) = head.bare() else {
            return;
        };
        match name {
            "&" => self.call(&words[1..]),
            "." => {
                let reason = "dot-sourcing a file".to_owned();
                self.acc.add(PolicyClass::Destructive, Some(reason));
            }
            _ if name.starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '!') => {}
            _ => {
                let lower = name.to_lowercase();
                if !(first && KEYWORDS.contains(&lower.as_str())) {
                    return self.named(&lower, &words[1..]);
                }
                if matches!(lower.as_str(), "return" | "throw" | "exit") && words.len() > 1 {
                    words.remove(0);
                    self.command(words, true, false);
                }
            }
        }
    }

    /// `& name …`: classified as `name`, when it is a literal bare name.
    fn call(&mut self, words: &[Word]) {
        match words.first().and_then(Word::bare) {
            Some(name) if !name.is_empty() => self.named(&name.to_lowercase(), &words[1..]),
            _ => {
                let reason = "`&` with something other than a command name".to_owned();
                self.acc.add(PolicyClass::Destructive, Some(reason));
            }
        }
    }

    fn named(&mut self, name: &str, args: &[Word]) {
        if name.contains(['\\', '/']) {
            let reason = format!("a path-qualified command `{name}`");
            self.acc.add(PolicyClass::Destructive, Some(reason));
            return;
        }
        let name = name
            .strip_suffix(".exe")
            .or_else(|| name.strip_suffix(".cmd"))
            .unwrap_or(name);
        let (class, reason) = heads::pwsh(name, args);
        self.acc.add(class, reason);
    }
}

/// Marks `w` as not literal and appends `what` to its spelling.
fn dynamic(w: &mut Word, what: &str) {
    w.literal = false;
    w.text.push_str(what);
}
```


File `crates/planrun-exec/src/policy/tests.rs` (full contents):

```rust
//! The classifier's tables, row by row: one test per shell, and one for what the splitter
//! refuses.

use planrun_model::Shell;

use super::PolicyClass::{Build, Destructive, Modify, Read};
use super::{DefaultClassifier, PolicyClass};

fn check(shell: Shell, rows: &[(&str, PolicyClass)]) {
    for (script, class) in rows {
        let found = DefaultClassifier.explain(shell, script);
        assert_eq!(
            found.class, *class,
            "{shell:?} {script:?}: {:?}",
            found.reasons
        );
        assert_eq!(
            found.reasons.is_empty(),
            *class != Destructive,
            "{script:?}"
        );
    }
}

#[test]
fn pwsh_commands() {
    #[rustfmt::skip]
    let rows = [
        ("", Read),
        ("git status --short            # empty", Read),
        ("Get-Content docs/modules.txt", Read),
        ("rg -n 'format!\\(\"Hello' src --glob '!src/lang.rs'", Read),
        ("$x = 41\nfunction F { 'f-called' }", Read),
        ("\"Hello, $name$($config.punctuation)\"", Read),
        ("$config = Get-Content (Join-Path $PSScriptRoot 'greet.json') -Raw | ConvertFrom-Json", Read),
        ("foreach ($name in $config.names) { \"Hello, $name\" }", Read),
        ("if ($gate -eq 0) { 'ok' }\nelse { 'no' }", Read),
        ("replace 'a' 'b' -Path src -Extension .rs -DryRun", Read),
        ("gitinfo; markers; Test-DevReadyPlan x.md", Read),
        ("[IO.Path]::Combine('a', 'b'); [Console]::Out; $s.Trim().Split(',').Count", Read),
        ("@{ a = 1; 'b' = 'x' }; @{a=1}", Read),
        ("git -C sub status; git -c color.ui=always --no-pager log --oneline", Read),
        ("git branch --show-current; git branch; git branch --list 'p*'", Read),
        ("Get-ChildItem -Recurse src | Sort-Object | Select-Object -First 3", Read),
        ("@'\nRemove-Item -Recurse x\n'@", Read),
        ("<# rm -rf #> 'x'; exit $gate", Read),
        ("& git status; Get-Item x | % { $_.Name } | ? { $_ }", Read),
        ("Get-Item x -ErrorAction SilentlyContinue; $a[0]; Write-Output a > $null 2>&1", Read),
        ("cargo --version; git --version; Set-Location sub; cd ..", Read),
        ("cargo test", Build),
        ("cargo test; $gate = $LASTEXITCODE; \"gate exit code: $gate\"", Build),
        ("cargo +nightly --manifest-path x/Cargo.toml build -q", Build),
        ("cargo fmt --check; cargo clippy --all-targets -- -D warnings", Build),
        ("dotnet build; dotnet restore; npm ci; npm run build; go test ./...", Build),
        ("brief { cargo test -p x }; test", Build),
        ("git add -- src/greet.rs src/lib.rs", Modify),
        ("if ($gate -eq 0) {\n  git add -- a\n  git commit -m \"m\"\n} else { 'no' }\ngit status --porcelain", Modify),
        ("git switch -c feature/x; git checkout -b y; git branch new", Modify),
        ("replace 'greeting(' 'greeting_en(' -Path src -Extension .rs -All", Modify),
        ("New-Item -ItemType Directory -Force docs | Out-Null; (Get-ChildItem src -Name | Sort-Object) -join \"`n\" | Set-Content -NoNewline docs/modules.txt", Modify),
        ("Remove-Item x.txt; Copy-Item a b; ni x; Tee-Object -FilePath x", Modify),
        ("'x' > out.txt; 'y' >> logs/out.txt", Modify),
        ("cargo fmt; cargo clippy --fix; npm install; dotnet format; go mod tidy", Modify),
        ("git diff --output=x.diff", Modify),
        ("Remove-Item -Recurse x", Destructive),
        ("Remove-Item –Recurse x", Destructive),
        ("rm -r x", Destructive),
        ("ri -rec x", Destructive),
        ("Remove-Item -Recurse:$true x", Destructive),
        ("Remove-Item @rest", Destructive),
        ("git push", Destructive),
        ("git reset --hard", Destructive),
        ("git -C x reset --hard", Destructive),
        ("git -c core.pager=less log", Destructive),
        ("git -c alias.x='!rm -rf .' x", Destructive),
        ("git --git-dir=x status", Destructive),
        ("git checkout main", Destructive),
        ("git checkout -- src/lib.rs", Destructive),
        ("git switch -f main", Destructive),
        ("git clean -fdx", Destructive),
        ("git stash drop", Destructive),
        ("git branch -D x", Destructive),
        ("git diff --ext-diff", Destructive),
        ("git config --global user.name x", Destructive),
        ("GIT.EXE PUSH", Destructive),
        ("iex 'x'", Destructive),
        ("Start-Process x", Destructive),
        ("irm http://x", Destructive),
        ("pwsh -NoProfile -File tools/greet.ps1   # Hello, Rens!", Destructive),
        ("bash -c 'ls'", Destructive),
        ("cmd /c dir", Destructive),
        ("& $cmd", Destructive),
        ("& 'git' status", Destructive),
        ("& .\\x.ps1", Destructive),
        (". ./x.ps1", Destructive),
        ("./script.ps1", Destructive),
        ("C:\\tools\\x.exe", Destructive),
        ("Microsoft.PowerShell.Management\\Remove-Item x", Destructive),
        ("frobnicate", Destructive),
        ("[IO.File]::Delete('x')", Destructive),
        ("$p.Kill()", Destructive),
        ("Get-ChildItem | % Delete", Destructive),
        ("'x' > C:\\out.txt", Destructive),
        ("'x' > ../out.txt", Destructive),
        ("'x' > $path", Destructive),
        ("cargo run; cargo install x", Destructive),
        ("npx x", Destructive),
        ("dotnet run", Destructive),
        ("go run .", Destructive),
        ("Import-Module ./module/x.psd1 -Force", Destructive),
        ("Add-Type -TypeDefinition $s", Destructive),
        ("New-Object Net.WebClient", Destructive),
        ("New-Item -ItemType SymbolicLink -Path l -Target x", Destructive),
        ("killport 8080", Destructive),
        ("replace 'a' 'b'", Destructive),
        ("\"$(Remove-Item -Recurse x)\"", Destructive),
        ("@\"\n$(git push)\n\"@", Destructive),
        ("@(git push)", Destructive),
        ("Write-Output ([IO.File]::ReadAllText('x'))", Destructive),
    ];
    check(Shell::Pwsh, &rows);
}

#[test]
fn bash_commands() {
    #[rustfmt::skip]
    let rows = [
        ("git branch --show-current     # feature/x", Read),
        ("cat x | head -1; wc -l < x", Read),
        ("test ! -e x && echo gone", Read),
        ("echo \"$(git status --short)\"; x=$(git rev-parse HEAD)", Read),
        ("for f in a b; do echo \"$f\"; done", Read),
        ("if [ -f x ]; then echo y; else echo n; fi", Read),
        ("arr=(a \"b c\" $(git status)); declare -A m=([k]=v)", Read),
        ("[[ -n $x ]]; (( i++ ))", Read),
        ("find . -name '*.rs'; sed 's/a/b/' x; command -v git", Read),
        ("cat <<'EOF'\nrm -rf /\nEOF\necho after", Read),
        ("echo x > /dev/null 2>&1; echo y >&2; echo z 2>/dev/null", Read),
        ("(cd sub && ls)", Read),
        ("cargo test 2>&1 | tail -3", Build),
        ("git add src && git commit -m 'x'", Modify),
        ("mkdir -p sub && cd sub", Modify),
        ("echo x > out.txt; echo y >> out.txt", Modify),
        ("sed -i 's/a/b/' x; rm x; touch y; sort -o x y; cp a b", Modify),
        ("cat <<EOF > out.txt\n$(git status)\nEOF", Modify),
        ("g() { echo; }", Read),
        ("rm -rf x", Destructive),
        ("rm --recursive x", Destructive),
        ("find . -delete", Destructive),
        ("find . -exec rm {} \\;", Destructive),
        ("git push origin main", Destructive),
        ("curl http://x", Destructive),
        ("bash x.sh", Destructive),
        ("./x.sh", Destructive),
        ("source x.sh", Destructive),
        (". x.sh", Destructive),
        ("eval \"$x\"", Destructive),
        ("$cmd", Destructive),
        ("FOO=1 git status", Destructive),
        ("pwsh -NoProfile -File tools/greet.ps1 -Names Ada | tail -1   # Hello, Ada!", Destructive),
        ("echo `rm -rf x`", Destructive),
        ("echo \"$(rm -rf x)\"", Destructive),
        ("cat <(curl x)", Destructive),
        ("cat <<EOF\n$(rm -rf x)\nEOF", Destructive),
        ("echo x > /etc/x", Destructive),
        ("echo x > ~/x", Destructive),
        ("echo x > \"$f\"", Destructive),
        ("g() { rm -rf x; }", Destructive),
        ("function g { git push; }", Destructive),
        ("xargs rm", Destructive),
        ("exec true", Destructive),
        ("\\rm x", Destructive),
    ];
    check(Shell::Bash, &rows);
}

#[test]
fn what_the_splitter_cannot_take_apart_is_destructive() {
    #[rustfmt::skip]
    let rows = [
        (Shell::Pwsh, "'unterminated"),
        (Shell::Pwsh, "\"unterminated"),
        (Shell::Pwsh, "@'\nnever closed"),
        (Shell::Pwsh, "@'not on its own line'@"),
        (Shell::Pwsh, "Write-Output ("),
        (Shell::Pwsh, "Write-Output )"),
        (Shell::Pwsh, "<# open"),
        (Shell::Pwsh, "$(Get-Item x"),
        (Shell::Pwsh, "[IO.Path"),
        (Shell::Pwsh, "Get-Content < x"),
        (Shell::Pwsh, "'x' >"),
        (Shell::Bash, "echo 'x"),
        (Shell::Bash, "echo \"x"),
        (Shell::Bash, "case $x in a) echo;; esac"),
        (Shell::Bash, "cat <<EOF\nnever"),
        (Shell::Bash, "echo ${x:-$(rm -rf /)}"),
        (Shell::Bash, "echo )"),
        (Shell::Bash, "echo $("),
        (Shell::Bash, "echo `x"),
        (Shell::Bash, "f ("),
    ];
    for (shell, script) in rows {
        let found = DefaultClassifier.explain(shell, script);
        assert_eq!(found.class, Destructive, "{shell:?} {script:?}");
        let unparsed = found.reasons.iter().any(|r| r.starts_with("unparsed: "));
        assert!(unparsed, "{shell:?} {script:?}: {:?}", found.reasons);
    }
}
```


**Step 2 — verify**

Targeted: the classifier's tables:

```powershell
cargo test -p planrun-exec --lib policy
```


Expected (exit code 0; summary lines):

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 31 filtered out; finished in …s
```


Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```


Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```


**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-exec/src/lib.rs crates/planrun-exec/src/policy/bash.rs crates/planrun-exec/src/policy/heads.rs crates/planrun-exec/src/policy/mod.rs crates/planrun-exec/src/policy/pwsh.rs crates/planrun-exec/src/policy/tests.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/lib.rs' = 'd93ba075f622580f97da6b288676a9147f9f57a7'
      'crates/planrun-exec/src/policy/bash.rs' = 'a43e330bfe5f3dd936a96e4a79ec4aa17d8a5322'
      'crates/planrun-exec/src/policy/heads.rs' = '7ab15b7476d4cf416f57829d533e3b4d3bbcdd4e'
      'crates/planrun-exec/src/policy/mod.rs' = '165b254413c15e2e709568c748efb72db96d11bd'
      'crates/planrun-exec/src/policy/pwsh.rs' = 'fd551b412152cfad8d2d2ebff733874bc82f7199'
      'crates/planrun-exec/src/policy/tests.rs' = '70ec3d602bf6e0eed31cdf9f4aef17191ae3bd42'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P4-T4: the command classifier: a splitter per shell and the built-in tables" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```


Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P4-T5: policy decisions: --auto, typed yes, unscoped file actions, expected failures skipped

**Goal:** `Policy::decide`: what the runner does with an action before running it (D67, D69). `--auto` takes read, build and modify; Destructive needs a typed `yes`; Manual waits; an unscoped file action asks; an Apply asks; a Verify script expecting failure is skipped unless expected failures run.

**Files**

- Create `crates/planrun-exec/src/policy/decide.rs`
- Modify `crates/planrun-exec/src/policy/mod.rs`

**Step 1 — write the files**

File `crates/planrun-exec/src/policy/decide.rs` (full contents):

```rust
//! What happens to an action before it runs (decisions D13, D50, D67, D69). P4 computes the
//! decision; P5's runner acts on it and prompts.

use planrun_model::{Action, ActionKind, ExitExpect, PlanSource, Script, ScriptIntent, Task};

use super::{CommandClassifier, PolicyClass};

/// What the runner does with an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// It runs without asking.
    Auto,
    /// It asks first; the class that made it ask.
    Ask(PolicyClass),
    /// A file action outside the task's declared files (D50): it always asks.
    AskUnscoped,
    /// A Destructive script: shown whole, it needs a typed `yes`.
    TypedYes,
    /// A Manual: it waits until acknowledged.
    Wait,
    /// A Note: shown in place.
    Show,
    /// It does not run (D67).
    Skip(Skip),
}

/// Why an action does not run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    /// A verification expected to fail, without `--run-expected-failures` (D12, D67).
    ExpectedFailure,
}

/// A refused `--auto` value.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    /// Not a class name.
    #[error("`--auto` takes read, build and modify; `{0}` is not one of them")]
    Unknown(String),
    /// `destructive`: a Destructive script always needs a typed `yes` (D69).
    #[error("`--auto destructive` is refused: a Destructive script always needs a typed `yes`")]
    Destructive,
}

/// The run's policy: which classes run without asking, and whether expected failures run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Policy {
    auto: Vec<PolicyClass>,
    run_expected_failures: bool,
}

impl Policy {
    /// A policy; `auto` may hold Read, Build and Modify, never Destructive.
    pub fn new(auto: &[PolicyClass], run_expected_failures: bool) -> Result<Policy, PolicyError> {
        if auto.contains(&PolicyClass::Destructive) {
            return Err(PolicyError::Destructive);
        }
        Ok(Policy {
            auto: auto.to_vec(),
            run_expected_failures,
        })
    }

    /// Parses `--auto`'s value: `read,build,modify`, any of them, in any case.
    pub fn parse_auto(text: &str) -> Result<Vec<PolicyClass>, PolicyError> {
        text.split(',')
            .map(|part| match part.trim().to_ascii_lowercase().as_str() {
                "read" => Ok(PolicyClass::Read),
                "build" => Ok(PolicyClass::Build),
                "modify" => Ok(PolicyClass::Modify),
                "destructive" => Err(PolicyError::Destructive),
                _ => Err(PolicyError::Unknown(part.trim().to_owned())),
            })
            .collect()
    }

    /// The decision for `action` of `task`; scripts are classified by `classifier`.
    pub fn decide(
        &self,
        task: &Task,
        action: &Action,
        source: &PlanSource,
        classifier: &dyn CommandClassifier,
    ) -> Decision {
        match &action.kind {
            ActionKind::Note { .. } => Decision::Show,
            ActionKind::Manual { .. } => Decision::Wait,
            ActionKind::Generated { .. } => Decision::Auto,
            ActionKind::WriteFile { path, .. }
            | ActionKind::ApplyPatch { path, .. }
            | ActionKind::ReplaceExact { path, .. }
            | ActionKind::Delete { path, .. } => {
                if task.manifest.scopes(path) {
                    Decision::Auto
                } else {
                    Decision::AskUnscoped
                }
            }
            ActionKind::RunScript(script) if self.skips(script) => {
                Decision::Skip(Skip::ExpectedFailure)
            }
            ActionKind::RunScript(script) | ActionKind::Commit(script) => {
                let class = classifier.classify(script.shell, source.resolve(&script.body));
                self.script(script, class)
            }
        }
    }

    /// D67: a Verify script that expects failure, unless expected failures run. A Gate, a
    /// Preview or an Apply runs (a skipped Gate leaves `$gate` unset for the commit), and a
    /// `Code(n)` is never an expected failure.
    fn skips(&self, script: &Script) -> bool {
        let failure = script
            .expect
            .as_ref()
            .is_some_and(|e| e.exit == ExitExpect::Failure);
        failure && script.intent == ScriptIntent::Verify && !self.run_expected_failures
    }

    fn script(&self, script: &Script, class: PolicyClass) -> Decision {
        if class == PolicyClass::Destructive {
            return Decision::TypedYes;
        }
        if script.intent == ScriptIntent::Apply || !self.auto.contains(&class) {
            return Decision::Ask(class);
        }
        Decision::Auto
    }
}

#[cfg(test)]
mod tests {
    use planrun_model::{ActionKind, ChangeKind, Expectation, PathOrigin, WriteMode};

    use super::*;
    use crate::ir::Ir;
    use crate::policy::DefaultClassifier;

    fn expecting(mut action: Action, exit: ExitExpect, intent: ScriptIntent) -> Action {
        if let ActionKind::RunScript(script) | ActionKind::Commit(script) = &mut action.kind {
            script.intent = intent;
            script.expect = Some(Expectation {
                exit,
                lines: None,
                prose: None,
                comments: None,
            });
        }
        action
    }

    #[test]
    fn decisions() {
        use Decision::{Ask, AskUnscoped, Auto, Show, TypedYes, Wait};
        use PolicyClass::{Build, Modify, Read};
        use ScriptIntent::{Apply, Gate, Preview, Verify};
        let mut ir = Ir::default();
        let manifest = ir.manifest(&[
            ("src/lib.rs", ChangeKind::Modify, PathOrigin::Written),
            ("README.md", ChangeKind::Modify, PathOrigin::Bare),
        ]);
        let none = Policy::default();
        let build = Policy::new(&[Read, Build], false).unwrap_or_else(|e| panic!("{e}"));
        let all = Policy::new(&[Read, Build, Modify], true).unwrap_or_else(|e| panic!("{e}"));
        let fails =
            |ir: &mut Ir, intent| expecting(ir.script("cargo test"), ExitExpect::Failure, intent);
        #[rustfmt::skip]
        let rows: Vec<(Action, &Policy, Decision)> = vec![
            (ir.note("a note"), &none, Show),
            (ir.manual("by hand"), &all, Wait),
            (ir.generated("Cargo.lock"), &none, Auto),
            (ir.write("src/lib.rs", "x", WriteMode::Overwrite), &none, Auto),
            (ir.write("src/new.rs", "x", WriteMode::Create), &all, AskUnscoped),
            (ir.replace("README.md", "a", "b"), &all, AskUnscoped),
            (ir.delete("src/lib.rs", true), &none, Auto),
            (ir.script("git status"), &none, Ask(Read)),
            (ir.script("git status"), &build, Auto),
            (ir.script("cargo test"), &build, Auto),
            (ir.script("git add -A"), &build, Ask(Modify)),
            (ir.script("git add -A"), &all, Auto),
            (ir.script("git push"), &all, TypedYes),
            (ir.commit("git add -A; git commit -m x"), &all, Auto),
            (expecting(ir.script("replace a b -All"), ExitExpect::Success, Apply), &all, Ask(Modify)),
            (fails(&mut ir, Verify), &none, Decision::Skip(Skip::ExpectedFailure)),
            (fails(&mut ir, Verify), &build, Decision::Skip(Skip::ExpectedFailure)),
            (fails(&mut ir, Verify), &all, Auto),
            (fails(&mut ir, Gate), &build, Auto),
            (fails(&mut ir, Preview), &none, Ask(Build)),
            (expecting(ir.script("rg x"), ExitExpect::Code(1), Verify), &none, Ask(Read)),
            (expecting(ir.commit("git commit -m x"), ExitExpect::Failure, Verify), &all, Auto),
        ];
        let task = ir.task("P1-T1", manifest, vec![]);
        let source = ir.source();
        for (action, policy, expected) in rows {
            let got = policy.decide(&task, &action, &source, &DefaultClassifier);
            assert_eq!(got, expected, "{:?}", source.slice(action.span));
        }
    }

    #[test]
    fn auto_values() {
        use PolicyClass::{Build, Modify, Read};
        assert_eq!(
            Policy::parse_auto("read,Build, MODIFY"),
            Ok(vec![Read, Build, Modify])
        );
        assert_eq!(
            Policy::parse_auto("read,destructive"),
            Err(PolicyError::Destructive)
        );
        let unknown = PolicyError::Unknown("all".to_owned());
        assert_eq!(Policy::parse_auto("all"), Err(unknown));
        let refused = Policy::new(&[PolicyClass::Destructive], false);
        assert_eq!(refused, Err(PolicyError::Destructive));
    }
}
```


File `crates/planrun-exec/src/policy/mod.rs` (full contents):

```rust
//! Policy classes and the command classifier (docs/architecture.md §6.5; decisions D13, D68).
//!
//! The classifier is a built-in allowlist: a script's class is the highest class of any command
//! in it, a command the tables do not know is Destructive, and so is a script the splitter
//! cannot take apart with confidence. No format spec, `[checks]` entry or flag changes a class,
//! and a runner has one classifier, not one per format.

mod bash;
mod decide;
mod heads;
mod pwsh;

pub use decide::{Decision, Policy, PolicyError, Skip};
pub use heads::DEVREADY_VERSION;

use planrun_model::Shell;

/// The command classes of decision D13, least to most dangerous.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolicyClass {
    /// Reads only (`git status`, `rg`).
    #[default]
    Read,
    /// Builds or tests (`cargo test`, `dotnet build`, `npm ci`).
    Build,
    /// Changes the repository (`git add`, `git commit`, `replace … -All`, file actions).
    Modify,
    /// Destroys or reaches outside (`Remove-Item -Recurse`, `git reset --hard`, `git push`,
    /// anything unclassified).
    Destructive,
}

/// Classifies a script: the most dangerous class of any command in it.
pub trait CommandClassifier: Send + Sync {
    /// The class of `script` run by `shell`.
    fn classify(&self, shell: Shell, script: &str) -> PolicyClass;
}

/// A script's class with the reasons that made it Destructive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classification {
    /// The highest class of any command.
    pub class: PolicyClass,
    /// What made it Destructive (`git push`, `unknown command \`foo\``, `unparsed: …`), in
    /// script order; empty below Destructive.
    pub reasons: Vec<String>,
}

/// The built-in classifier (D68), with the tables of `heads.rs`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultClassifier;

impl DefaultClassifier {
    /// The class of `script` and why.
    pub fn explain(&self, shell: Shell, script: &str) -> Classification {
        let mut acc = Acc::default();
        let parsed = match shell {
            Shell::Pwsh => pwsh::classify(script, &mut acc),
            Shell::Bash => bash::classify(script, &mut acc),
        };
        if let Err(why) = parsed {
            acc.add(PolicyClass::Destructive, Some(format!("unparsed: {why}")));
        }
        Classification {
            class: acc.class,
            reasons: acc.reasons,
        }
    }
}

impl CommandClassifier for DefaultClassifier {
    fn classify(&self, shell: Shell, script: &str) -> PolicyClass {
        self.explain(shell, script).class
    }
}

/// One word of a command as the splitter saw it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Word {
    /// Its text: the value without quotes when `literal`, the raw spelling otherwise.
    pub text: String,
    /// No variable, subexpression or other expansion: `text` is exactly the value.
    pub literal: bool,
    /// Some part was quoted (a quoted word is never a parameter or a keyword).
    pub quoted: bool,
}

impl Word {
    /// A literal word that was not quoted.
    pub fn bare(&self) -> Option<&str> {
        (self.literal && !self.quoted).then_some(self.text.as_str())
    }
}

/// What the splitter found so far: the highest class, and the reasons for Destructive.
#[derive(Debug, Default)]
pub(crate) struct Acc {
    pub class: PolicyClass,
    pub reasons: Vec<String>,
}

impl Acc {
    /// Raises the class to `class`; a reason is kept only for Destructive.
    pub fn add(&mut self, class: PolicyClass, reason: Option<String>) {
        self.class = self.class.max(class);
        if class == PolicyClass::Destructive {
            self.reasons
                .push(reason.unwrap_or_else(|| "destructive".to_owned()));
        }
    }
}

/// A redirection target's class (D68): nothing for the null device, Modify for a literal
/// relative path without `..`, Destructive for anything else.
pub(crate) fn redirect_target(word: &Word, acc: &mut Acc) {
    let text = word.text.as_str();
    let null = ["$null", "/dev/null"].contains(&text) || text.eq_ignore_ascii_case("nul");
    if null && (word.literal || text == "$null") {
        return;
    }
    let relative = word.literal
        && !text.is_empty()
        && !text.starts_with(['/', '\\', '~'])
        && !text.contains(':')
        && !text.split(['/', '\\']).any(|s| s == "..");
    if relative {
        acc.add(PolicyClass::Modify, None);
        return;
    }
    acc.add(
        PolicyClass::Destructive,
        Some(format!("a redirection to `{text}`")),
    );
}

#[cfg(test)]
mod tests;
```


**Step 2 — verify**

Targeted: the classifier's and the decisions' tables:

```powershell
cargo test -p planrun-exec --lib policy
```


Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 31 filtered out; finished in …s
```


Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```


Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```


**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-exec/src/policy/decide.rs crates/planrun-exec/src/policy/mod.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/policy/decide.rs' = 'a639b65d5abd83f88ad1b1dae076442f86c2b7d1'
      'crates/planrun-exec/src/policy/mod.rs' = 'a465fd93441cd891aba2e85b75e79bce202ee804'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P4-T5: policy decisions: --auto, typed yes, unscoped file actions, expected failures skipped" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```


Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P4-T6: the dry run shows each script's class and decision; the classifier over the local plans

**Goal:** The dry run shows each script's and commit's class and the default policy's decision. `local_classes` (ignored) classifies every script of the local plans and prints counts and the reasons for Destructive, in the terminal only (D16).

**Files**

- Modify `crates/planrun-cli/src/dry_run.rs`
- Create `crates/planrun-cli/tests/local_classes.rs`
- Modify `crates/planrun-cli/tests/snapshots/dry_run__dry_run_assay_style.snap`
- Modify `crates/planrun-cli/tests/snapshots/dry_run__dry_run_create_exists.snap`

**Step 1 — write the files**

File `crates/planrun-cli/src/dry_run.rs` (full contents):

```rust
//! `planrun run --dry-run` (docs/architecture.md §7, decision D55): one line per action with its
//! outcome, a summary, then every finding with its source context. A script or commit shows its
//! class and what the default policy would do with it (D68, D69).

use planrun_exec::finding::{Finding, Severity as ExecSeverity};
use planrun_exec::policy::{Decision, DefaultClassifier, Policy, Skip};
use planrun_exec::preflight::{DryRun, Outcome};
use planrun_model::{ActionKind, Plan, Task};
use planrun_parse::{Diagnostic, Severity};

use crate::render;

/// Counts for the summary line.
#[derive(Default)]
struct Counts {
    ok: usize,
    ask: usize,
    failed: usize,
    depends: usize,
    not_run: usize,
    not_checked: usize,
    waits: usize,
}

/// The report of `run`, rendered into `out`.
pub fn report(plan: &Plan, run: &DryRun, out: &mut String) {
    let name = plan.source.name.to_string();
    out.push_str(&format!(
        "dry run of {name} ({}, {} tasks): nothing is written, no script runs\n",
        plan.format.0,
        plan.tasks.len()
    ));
    out.push_str(
        "after the first script, commit or Manual step, each result assumes it changed no files\n",
    );
    for path in &run.attribute_writes {
        out.push_str(&format!(
            "note: the plan writes `{path}`; the dry run reads attributes from the disk as they are\n"
        ));
    }
    let mut counts = Counts::default();
    for task in &run.tasks {
        let t = &plan.tasks[task.task];
        out.push_str(&format!("\n{}  {}\n", t.id, plan.text(&t.title)));
        if !task.manifest.is_empty() {
            out.push_str(&format!("  files: {} problem(s)\n", task.manifest.len()));
        }
        for a in &task.actions {
            if let Some(line) = action_line(plan, t, a, &mut counts) {
                out.push_str(&line);
            }
        }
    }
    out.push_str(&format!(
        "\nsummary: {} ok, {} would ask, {} failed, {} depend on a step that did not run; \
         {} scripts and commits not run, {} generated files not checked, {} manual\n",
        counts.ok,
        counts.ask,
        counts.failed,
        counts.depends,
        counts.not_run,
        counts.not_checked,
        counts.waits
    ));
    let findings: Vec<Diagnostic> = run.findings().map(diagnostic).collect();
    if !findings.is_empty() {
        out.push('\n');
        render::diagnostics(&name, plan.source.text(), &findings, out);
    }
}

/// One table line; `None` for a Note.
fn action_line(
    plan: &Plan,
    task: &Task,
    a: &planrun_exec::preflight::ActionReport,
    counts: &mut Counts,
) -> Option<String> {
    let (what, path) = describe(&a.action.kind);
    let outcome = match &a.outcome {
        Outcome::Shown => return None,
        Outcome::Ok(detail) => {
            counts.ok += 1;
            format!("{detail}: ok")
        }
        Outcome::WouldAsk(detail) => {
            counts.ask += 1;
            format!("{detail}: would ask (not in the task's files, D50)")
        }
        Outcome::Failed(f) => {
            counts.failed += 1;
            format!("FAILED {}", f.code)
        }
        Outcome::Depends(f) => {
            counts.depends += 1;
            format!("unverified {} (depends on a step that did not run)", f.code)
        }
        Outcome::NotRun => {
            counts.not_run += 1;
            let decision =
                Policy::default().decide(task, &a.action, &plan.source, &DefaultClassifier);
            format!("not run; {}", decided(decision))
        }
        Outcome::NotChecked => {
            counts.not_checked += 1;
            "not checked (its script did not run)".to_owned()
        }
        Outcome::Waits => {
            counts.waits += 1;
            "waits for you".to_owned()
        }
    };
    let line = plan.source.line_col(a.action.span.start).line;
    Some(format!(
        "  L{line:<5} {what:<9} {path:<24} {outcome}\n",
        path = path.unwrap_or_default()
    ))
}

/// What the default policy (no `--auto`) would do with a script or commit.
fn decided(decision: Decision) -> String {
    match decision {
        Decision::Ask(class) => format!("would ask ({class:?})"),
        Decision::TypedYes => "typed yes (Destructive)".to_owned(),
        Decision::Skip(Skip::ExpectedFailure) => "skipped: expected failure".to_owned(),
        other => format!("{other:?}"),
    }
}

fn describe(kind: &ActionKind) -> (&'static str, Option<String>) {
    match kind {
        ActionKind::WriteFile { path, .. } => ("write", Some(path.to_string())),
        ActionKind::ApplyPatch { path, .. } => ("patch", Some(path.to_string())),
        ActionKind::ReplaceExact { path, .. } => ("replace", Some(path.to_string())),
        ActionKind::Delete { path, .. } => ("delete", Some(path.to_string())),
        ActionKind::Generated { path } => ("generated", Some(path.to_string())),
        ActionKind::RunScript(_) => ("script", None),
        ActionKind::Commit(_) => ("commit", None),
        ActionKind::Manual { .. } => ("manual", None),
        ActionKind::Note { .. } => ("note", None),
    }
}

/// A finding as a diagnostic, so it renders like the parser's.
fn diagnostic(f: &Finding) -> Diagnostic {
    Diagnostic {
        severity: match f.severity {
            ExecSeverity::Error => Severity::Error,
            ExecSeverity::Warning => Severity::Warning,
        },
        code: f.code,
        span: f.span,
        message: f.message.clone(),
        related: Vec::new(),
    }
}
```


File `crates/planrun-cli/tests/local_classes.rs` (full contents):

```rust
//! The classifier over the scripts of the real plans in `tests/corpus/local` (git-ignored,
//! decision D16). Ignored by default: `cargo test -p planrun-cli --test local_classes --
//! --ignored --nocapture`. It prints, per plan and in total, how many scripts and commits fall
//! in each class, and which reasons made scripts Destructive. Nothing it prints goes into a
//! committed file or snapshot.

use std::collections::BTreeMap;
use std::path::Path;

use planrun_exec::policy::{DefaultClassifier, PolicyClass};
use planrun_model::ActionKind;
use planrun_parse::{Registry, parse_plan};

#[test]
#[ignore = "needs the private plans in tests/corpus/local"]
fn local_classes() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/local");
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap_or_else(|e| panic!("{e}")).path())
        .filter(|p| p.extension().is_some_and(|x| x == "md") && !p.ends_with("README.md"))
        .collect();
    paths.sort();
    let mut total: BTreeMap<PolicyClass, usize> = BTreeMap::new();
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    for path in &paths {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let parsed = parse_plan(&registry, name, &bytes, None)
            .unwrap_or_else(|d| panic!("{name}: {:?}", d.error_codes()));
        let plan = parsed.plan;
        let mut counts: BTreeMap<PolicyClass, usize> = BTreeMap::new();
        let scripts = plan
            .tasks
            .iter()
            .flat_map(|t| &t.steps)
            .flat_map(|s| &s.actions)
            .filter_map(|a| match &a.kind {
                ActionKind::RunScript(s) | ActionKind::Commit(s) => Some(s),
                _ => None,
            });
        for script in scripts {
            let found = DefaultClassifier.explain(script.shell, plan.text(&script.body));
            *counts.entry(found.class).or_default() += 1;
            *total.entry(found.class).or_default() += 1;
            for reason in found.reasons {
                *reasons.entry(reason).or_default() += 1;
            }
        }
        println!("{name}: {counts:?}");
    }
    println!("total: {total:?}");
    println!("what made scripts Destructive:");
    for (reason, n) in &reasons {
        println!("  {n:>4}  {reason}");
    }
}
```


File `crates/planrun-cli/tests/snapshots/dry_run__dry_run_assay_style.snap` (full contents):

```text
---
source: crates/planrun-cli/tests/dry_run.rs
expression: out
---
dry run of ../outside/assay-style.md (assay, 2 tasks): nothing is written, no script runs
after the first script, commit or Manual step, each result assumes it changed no files

P1-T1  the greet module
  L35    write     src/greet.rs             create: ok
  L54    patch     src/lib.rs               aa80cf1 → 97bd92a: ok
  L83    script                             not run; would ask (Build)
  L95    script                             not run; would ask (Build)
  L110   commit                             not run; would ask (Modify)

P1-T2  re-export greeting, remove the legacy module
  L130   generated docs/modules.txt         not checked (its script did not run)
  L134   patch     src/lib.rs               97bd92a → 6b269fc: ok
  L159   delete    src/old.rs               delete: ok
  L161   generated docs/modules.txt         not checked (its script did not run)
  L167   script                             not run; would ask (Modify)
  L181   script                             not run; would ask (Build)
  L196   commit                             not run; would ask (Modify)

summary: 4 ok, 0 would ask, 0 failed, 0 depend on a step that did not run; 6 scripts and commits not run, 2 generated files not checked, 0 manual
planrun: ../outside/assay-style.md: dry run ok
```


File `crates/planrun-cli/tests/snapshots/dry_run__dry_run_create_exists.snap` (full contents):

```text
---
source: crates/planrun-cli/tests/dry_run.rs
expression: out
---
dry run of ../outside/assay-style.md (assay, 2 tasks): nothing is written, no script runs
after the first script, commit or Manual step, each result assumes it changed no files

P1-T1  the greet module
  files: 1 problem(s)
  L35    write     src/greet.rs             FAILED create-exists
  L54    patch     src/lib.rs               aa80cf1 → 97bd92a: ok
  L83    script                             not run; would ask (Build)
  L95    script                             not run; would ask (Build)
  L110   commit                             not run; would ask (Modify)

P1-T2  re-export greeting, remove the legacy module
  L130   generated docs/modules.txt         not checked (its script did not run)
  L134   patch     src/lib.rs               97bd92a → 6b269fc: ok
  L159   delete    src/old.rs               delete: ok
  L161   generated docs/modules.txt         not checked (its script did not run)
  L167   script                             not run; would ask (Modify)
  L181   script                             not run; would ask (Build)
  L196   commit                             not run; would ask (Modify)

summary: 3 ok, 0 would ask, 1 failed, 0 depend on a step that did not run; 6 scripts and commits not run, 2 generated files not checked, 0 manual

error[create-exists]: `src/greet.rs` already exists; the task's files say it is created
  --> ../outside/assay-style.md:30:1
   |
30 | - Create `src/greet.rs`
   | ^^^^^^^^^^^^^^^^^^^^^^^

error[create-exists]: `src/greet.rs` already exists; the plan creates it
  --> ../outside/assay-style.md:35:1
   |
35 | / File `src/greet.rs` (full contents):
36 | |
37 | | ```rust
38 | | //! Greetings (P1-T1).
...  |
52 | | ```
   | |___^

planrun: ../outside/assay-style.md: the dry run found 2 error(s)
```


**Step 2 — verify**

Targeted: the dry run of the CLI:

```powershell
cargo test -p planrun-cli --test dry_run
```


Expected (exit code 0; summary lines):

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```


Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```


Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```


**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-cli/src/dry_run.rs crates/planrun-cli/tests/local_classes.rs crates/planrun-cli/tests/snapshots/dry_run__dry_run_assay_style.snap crates/planrun-cli/tests/snapshots/dry_run__dry_run_create_exists.snap
  $verified = [ordered]@{
      'crates/planrun-cli/src/dry_run.rs' = '3742eaa0cd2d5844339843677b8210d9c104eb51'
      'crates/planrun-cli/tests/local_classes.rs' = '815472e9b51ad32ce0f03c1c708ee6377efd2be3'
      'crates/planrun-cli/tests/snapshots/dry_run__dry_run_assay_style.snap' = '336606aa643ef17d8e85069f398631a36786fb80'
      'crates/planrun-cli/tests/snapshots/dry_run__dry_run_create_exists.snap' = 'c5677ab0ba176e977aa7434d479810da33c1d288'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P4-T6: the dry run shows each script's class and decision; the classifier over the local plans" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```


Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P4-T7: architecture, extending, roadmap and CLAUDE.md for shells, expectations and policy

**Goal:** The docs in step: architecture.md §6.3 (sessions), §6.4 (expectations), §6.5 (policy, the corrected table, `--auto`) and the dry run's classes in §7; extending.md §7; the roadmap's P4 entry; the CLAUDE.md layout, the `local_classes` command and the shell-test and table-test conventions.

**Files**

- Modify `CLAUDE.md`
- Modify `docs/architecture.md`
- Modify `docs/extending.md`
- Modify `docs/roadmap.md`

Over 300 lines, so they come as exact diffs: `docs/architecture.md`.

**Step 1 — write the files**

File `CLAUDE.md` (full contents):

````markdown
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
crates/planrun-model   the IR: Plan, Task, Step, Action, Span/TextRef, RepoPath, PathOrigin,
                       ManifestPath/PathPattern,
                       AbbrevBlobId
crates/planrun-parse   Markdown → IR: source, document, format, spec, registry, outline,
                       manifest, recognizer, recognize, kinds/ (one file per recognizer kind),
                       patch, check, diagnostic
crates/planrun-exec    IR → effects: git, workspace, eol, tree, handler, transaction,
                       preflight, finding (P3); shell/ (sessions, kill.rs: the only unsafe
                       code), expect, policy/ (classifier, decisions) (P4); journal, runner
                       (P5)
crates/planrun-cli     the `planrun` binary: check, show (view, render), run --dry-run
formats/*.toml         built-in plan formats (embedded): assay, superpowers, before-after; the
                       file name is the format id
docs/                  architecture, plan-grammar, extending, decisions, roadmap, plans/,
                       handovers/ (one per finished phase)
tests/corpus/          synthetic/ (one plan per format), invalid/ (one error each), local/ (private)
tests/fixtures/greeter the repo the synthetic assay plan applies to
tests/support          fixture.rs: a temporary git copy of the greeter fixture, included
                       with #[path] by the exec and cli tests
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
cargo test -p planrun-cli --test local_classes -- --ignored --nocapture   # their script classes
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
  never on this repository: `tests/support/fixture.rs` makes one under the OS temp directory,
  with an environment that keeps the user's git configuration out. Exec tests build the IR by
  hand (`crates/planrun-exec/tests/support/ir.rs`, D2); tests that need a parsed plan live in
  planrun-cli.
- Shell tests (`crates/planrun-exec/tests/shell.rs`) skip a missing shell with a message;
  `PLANRUN_REQUIRE_SHELLS=1` makes it a failure, and every plan's verification sets it (D63).
- Table tests (classifier, matcher) keep their rows on one line each under `#[rustfmt::skip]`.

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
- Rulings relayed from a review seat (another Claude session with less context) are checked
  against the code and the plans before acting. Where one rests on a wrong premise, keep the
  correct position and state the evidence (file, task, what the plan says) in the plan's
  stated decisions or the reply, so a further review can settle it.
- One commit per task, message `P<N>-T<n>: <title>`. Don't push.
- When a phase is done, write a handover for the next session:
  `docs/handovers/YYYY-MM-DD-p<N>-to-p<N+1>.md` (state, what the phase built, open items, notes
  for the next phase, how plans are verified). A new session starts from the latest one.
````


File `docs/architecture.md` (exact diff; save the block as `$env:TEMP\p4-t7-architecture-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/architecture.md b/docs/architecture.md
index dc09ca3..a05fd7b 100644
--- a/docs/architecture.md
+++ b/docs/architecture.md
@@ -282,41 +282,83 @@ Done | Failed | Skipped`. Per task, the lifecycle of your §6.1 proposal:
 
 ### 6.3 Shell sessions
 
-One persistent session per shell kind per run (D24): `pwsh` for `powershell`/`pwsh`/`ps1` fences,
-`bash` for `bash`/`sh`. Variables, functions and the working directory persist across blocks, as
-in Rens' terminal (`$gate` set in the Gate block is read in the commit block; DevReady plans set
-`$P2` once).
-
-Protocol: each block is written to a temp file outside the repository and dot-sourced (`. 'file'`
-/ `source file`), wrapped so that a per-run random sentinel line carrying the exit status follows
-it, even when the block throws. Output is streamed to the UI line by line and captured for the
-matcher. Sessions start at the repository root with `-NoProfile`.
+One persistent session per shell kind per run (D24): `pwsh` for `powershell`/`pwsh`/`ps1`
+fences, `bash` for `bash`/`sh`/`shell` (the spec defaults, extending.md §2). Variables,
+functions and the working directory persist across blocks, as in Rens' terminal (`$gate` set in
+the Gate block is read in the commit block; DevReady plans set `$P2` once). `shell::Session`
+implements `ShellSession`; P5 keeps one per shell.
+
+- **Which shells** (D63): `pwsh` from `PATH` when it is PowerShell 7 or later, never Windows
+  PowerShell 5.1. On Windows, bash is Git for Windows' `<root>\usr\bin\bash.exe`, `<root>` found
+  from `git --exec-path` and checked with `uname -s` (MINGW or MSYS), never `bash` from `PATH`
+  (WSL); its `PATH` gets `/usr/bin` in front. Elsewhere `bash` from `PATH`.
+- **Protocol** (D60): sessions start at the repository root with `-NoProfile`
+  (`--noprofile --norc`), detached from planrun's terminal. Each block is written to a file in a
+  fresh run directory under the OS temp directory (never the working tree, D7), whose path comes
+  through the environment. Blocks get no stdin: pwsh has none at all and its `-Command` script
+  is a loop that dot-sources each block file as it appears; bash gets one ASCII line per block,
+  `source "<dir>/b<n>.sh" </dev/null`, at the top level. After each block a sentinel with a
+  per-session secret and the block's number arrives on stdout (status, `errored`, the working
+  directory) and on stderr; the block ends when both have. Output is streamed line by line with
+  its stream and kept for the matcher; order across the two streams is arrival order.
+- **How a block ends** (D61): `Exited(code)` (pwsh `$LASTEXITCODE`, reset before the block;
+  bash `$?`), `Threw` (pwsh), `SessionEnded` (bash `exit`, or the shell gone), `TimedOut`,
+  `Broken` (a sentinel that does not fit). pwsh also reports `errored`: `$Error[0]` changed.
+- **Environment** (D62): planrun's own, without `BASH_ENV`, `ENV` and the git wrapper's
+  variables, with `GIT_TERMINAL_PROMPT=0`.
+- **The process tree** (D64): a job object on Windows, `setsid` on Unix; a timeout (none by
+  default, D65) or a drop kills every descendant. `shell/kill.rs` holds planrun's only
+  `unsafe` code.
+- **Stated differences from the terminal:** `$LASTEXITCODE` is reset to 0 before each block; a
+  terminating error ends the block; `$PSScriptRoot` and `$MyInvocation` name the block's temp
+  file; pwsh `exit n` ends only the block, bash `exit n` the session; `GIT_TERMINAL_PROMPT=0`.
 
 Windows with PowerShell 7 is the primary platform (D19); Linux with pwsh and bash is supported.
 
 ### 6.4 Expectations
 
-- **Exit**: `Success` = 0; `Failure` = non-zero; `Code(n)`; `Unknown` = not checked.
-- **Summary lines** (fenced `Expected …:` blocks): every expected line must match an output line,
-  in order (a subsequence); `…` matches any run of characters; trailing whitespace ignored.
-- **Prose** (`Expected: all pass`): mapped to an exit expectation by the format's phrase table,
-  checked, and then **shown for confirmation**. Prose never passes a script on its own (D11).
-- **Expected failures** (`Expected: build error`, "run the test to verify it fails"): recorded in
-  the IR, **skipped by default** (D12). `--run-expected-failures` runs them.
-- A mismatch stops and offers: accept (journaled as accepted-with-mismatch), retry, open a shell,
-  abort.
+`expect::check` turns a run and its `Expectation` into `Pass`, `Mismatch` or `Confirm` (D66).
+
+- **Exit**: `Success` = 0; `Failure` = non-zero or a throw; `Code(n)`; `Unknown` = not checked.
+  A session that ended, timed out or broke is a mismatch whatever was expected.
+- **Summary lines** (fenced `Expected …:` blocks): every non-blank expected line must match an
+  output line, in order (a subsequence); `…` matches any run of characters, the empty one
+  included, and only `…`; the first literal segment is anchored at the start, the last at the
+  end. Output is split on LF, trailing whitespace is stripped on both sides (a CR with it), ANSI
+  CSI and OSC sequences are stripped from the output; leading whitespace counts.
+- **Prose** (`Expected: all pass`) and **comments** (`# main` after a command, D35, D48): the
+  exit status the format's phrase table derived is checked, and then the text is **shown for
+  confirmation**, the two labelled apart. Prose never passes a script on its own (D11).
+- **Asking without prose:** an exit the plan does not state that is not 0 (or a throw), and a
+  pwsh `errored` run where nothing but an exit status was expected, ask too. Every
+  confirmation shows `errored`.
+- **Expected failures** (`Expected: build error`, "run the test to verify it fails"): a Verify
+  script expecting failure is **skipped by default** (D12, D67), decided by the policy before
+  the run; `--run-expected-failures` runs it. Gates, previews and applies always run.
+- A mismatch stops and offers (P5): accept (journaled as accepted-with-mismatch), retry, open a
+  shell, abort.
 
 ### 6.5 Policy
 
 Every runnable thing has a class (D13): `Read` < `Build` < `Modify` < `Destructive`. File actions
-are `Modify`. A script's class is the highest class of any command in it, by a
-`CommandClassifier` (known heads: `git status|diff|log|show`, `rg`, `Get-Content` → Read;
-`cargo|dotnet|npm|npx test|build|check|ci` → Build; `git add|commit|apply|rm|switch|checkout`,
-`replace … -All`, `Set-Content` → Modify; `Remove-Item -Recurse`, `rm -rf`, `git reset --hard`,
-`git push`, `git clean` → Destructive). **Unknown commands are Destructive.**
-
-Defaults: file actions run automatically; Build and Modify scripts ask; Destructive scripts show
-the script and ask for a typed `yes`; `Manual` always waits. `--auto <classes>` widens this.
+are `Modify`. A script's class is the highest class of any command in it, by the
+`DefaultClassifier` (D68): a built-in allowlist that no spec or flag changes, over a splitter
+per shell that recurses into subexpressions, script blocks, command substitutions and
+expanding here-strings and heredocs. What it cannot take apart with confidence, and every
+command its tables do not know, is **Destructive**.
+
+| Class | Examples |
+|---|---|
+| Read | `git status`, `diff`, `log`, `show`, `branch` (listing), `rev-parse`; `rg`; `Get-Content`, `Get-ChildItem`, `Select-String`, `Test-Path`, `Sort-Object`, `ConvertFrom-Json`, `Set-Location`; bash `cat`, `ls`, `head`, `grep`, `echo`, `test`; `replace … -DryRun`; DevReady functions whose agent mode is `safe` |
+| Build | `cargo build`/`test`/`check`/`clippy`/`fmt --check`/`doc`; `dotnet build`/`test`/`restore`; `npm ci`/`test`/`run`; `go build`/`test`/`vet`; DevReady `test` |
+| Modify | `git add`, `commit`, `apply`, `rm`, `mv`, `switch`, `checkout -b`, `stash`, `reset` (soft, mixed); `cargo fmt`, `fix`, `add`; `npm install`; `dotnet format`; `go mod tidy`; `New-Item`, `Set-Content`, `Add-Content`, `Copy-Item`, `Move-Item`, `Remove-Item` (no `-Recurse`); `mkdir`, `cp`, `mv`, `rm` (no `-r`), `sed -i`; `replace … -All`; a redirection to a relative path |
+| Destructive | `Remove-Item -Recurse`, `rm -r`; `git push`, `fetch`, `pull`, `reset --hard`, `clean`, `restore`, `checkout <name>` or `--`, `switch -f`, `branch -D`, `stash drop`; `git -c` with any key but `color.*`, `advice.*`, `core.quotepath`, `user.name`, `user.email`; `cargo run`/`install`, `dotnet run`, `npx`; `iex`, `Start-Process`, `Invoke-WebRequest`, `curl`; `pwsh -File`, `bash -c`, `cmd /c`, `source`, `eval`; a static or instance .NET call outside a short pure list; a redirection outside the repository; anything unknown |
+
+Defaults (D13, D69): file actions within the task's scope run automatically; every script asks,
+Read ones included; Destructive scripts show the script and ask for a typed `yes`; `Manual`
+always waits; a file action outside the scope (D50) and an Apply script always ask. `--auto
+read,build,modify` (any of them) lets those classes run without asking; `destructive` is
+refused. `Policy::decide` computes the decision in P4; the runner prompts in P5.
 
 ### 6.6 Journal and resume
 
@@ -346,7 +388,8 @@ tasks before it wrote, and simulate every file action. Commits are assumed. Runs
 writes nothing: not the working tree, not `.git`. Once a script, commit or Manual has gone by
 without running, a failing file action is a warning ("depends on a step that did not run") and
 its path is unknown from then on; every ok after that point assumes those steps change no files.
-Exit 0 when nothing fails, 1 when something does, 2 when git or the repository cannot be used.
+Each script and commit shows its class and what the default policy would do with it ("would ask
+(Build)", "typed yes (Destructive)", "skipped: expected failure", D69). Exit 0 when nothing fails, 1 when something does, 2 when git or the repository cannot be used.
 
 The interactive screen (line prompts first, D15):
 
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p4-t7-architecture-md.patch"
Remove-Item "$env:TEMP\p4-t7-architecture-md.patch"
```


File `docs/extending.md` (full contents):

````markdown
# Extending planrun

New plan styles will keep appearing. They are added without touching execution, in three levels
(architecture.md §5). Use the lowest level that works.

## 1. Decide the level

1. Describe the style in a table like the ones in [plan-grammar.md](plan-grammar.md), from at
   least two real plans.
2. For each row, find the built-in recognizer kind that covers it (§3). If every row is covered,
   write a **format spec** (§2). Done, no rebuild.
3. If a row is a new idiom, write a **recognizer kind** (§5) and use it from a spec.
4. Only if the plan's *structure* is not headings-with-steps (tasks in a table, a JSON plan),
   write a **Format** (§6).
5. A new **action kind** (§4) is almost never the answer: map the wording onto an existing action.

## 2. Format specs (TOML)

Location: built-ins in `formats/<id>.toml` (embedded); per repository in
`<repo>/.planrun/formats/<id>.toml`; per user in the config directory. The file name is the id.

```toml
id = "assay"                      # unique; the file name
name = "Assay plan (planbuild.py)"

[detect]                          # a pattern that matches a line outside fenced blocks adds
patterns = [ { regex = '^### P\d+-T\d+: ', weight = 60 } ]   # its weight once; capped at 100

[outline]
tasks_after = { level = 2, regex = '^Tasks$' }          # optional
task = { level = 3, regex = '^(?P<id>(?P<phase>P\d+)-T(?P<ord>\d+)): (?P<title>.+)$' }
tasks_end_level = 2                                      # optional
# captures: id, ord (a number), title (all required); phase (optional, from the first task)

[outline.manifest]                                       # optional
# The marker and its items are consumed by the outline: they fall in no step. The items end at
# the first block that is no list item at the marker's depth, or that is a step boundary (D37).
marker = '^\*\*Files\*\*$'                                # may capture `change` (`**Files:** create`)
# and `items`: then the marker paragraph holds the manifest, split by `clause_list`, each
# clause matched by `item`; text that declares nothing is shown as a Note (D43)
item = '^(?P<change>Create|Modify|Delete) `(?P<paths>[^`]+)`$'   # `change` unless the marker has it
changes = { Create = "create", Modify = "modify", Delete = "delete" }
# → create | modify | rewrite | delete | test | generated (D27)
path_list = ', '                                         # optional: split `paths` into several;
# a `:3-10` piece adds lines to the path before it (D32). Built in, before the split, in every
# format: `a`–`b` (an en dash between code spans) is one range entry, so `path_list` must not
# list the en dash; a piece with `*`, `?` or `{` is a glob entry (D41). Pattern entries declare
# the files they match and are never a write target.
required = true                                          # optional, default true: false lets a
                                                         # task have no manifest (D27)
siblings = false                                         # optional, default false: a bare file
                                                         # name after `a/b.rs` is `a/<name>` (D32);
                                                         # without siblings it is Bare (D45)
clause_list = '; '                                       # optional: needs an `items` capture (D43)
path_change = '^` \((?P<change>full replacement)\)'     # optional (D44): over the text right
# after a path token; `change` goes through `changes` and wins over the item's change, which
# wins over the marker's

[[outline.steps]]                                        # zero or more; none = one implicit step
block = "paragraph"                                      # paragraph | list-item | heading
regex = '^\*\*Step (?P<n>\d+) — (?P<title>[^*]+)\*\*$'   # n and title optional
keep = false                                             # optional: true keeps the boundary
                                                         # block in its step (D46)

[settings]
manual_markers = ['(?i)\bby hand\b']                     # paragraph → Manual(Marked)
pwsh = ["powershell", "pwsh", "ps1"]                     # optional; these are the defaults
bash = ["bash", "sh", "shell"]

[[step_marks]]                                           # zero or more (D30), first match wins
title = '(?i)\bverify (?:it|they) fails?\b'              # regex over the step's title
mark = "expected-failure"                                # expected-failure | human-check
# expected-failure: the step's Verify scripts expect failure (none or `any` becomes Failure; an
# expected success is the error expected-conflict). human-check: its prose Notes become
# Manual(HumanCheck).

[checks]                                                 # optional (D31): only these two codes,
manifest-untouched = "off"                               # each error | warning | off; any
manifest-undeclared = "warning"                          # other key fails the spec load

[[recognizer]]                                           # ordered; first match wins
kind = "file-block"                                      # a registered kind (§3)
id = "file-block#assay"                                  # shown by `show` and coverage
# … the kind's own parameters
```

Rules:

- Unknown keys are errors (`deny_unknown_fields`). A spec is fully validated when the registry
  loads it (unknown kind, bad regex, bad parameter), before any plan is read.
- Regexes use the `regex` crate syntax; `(?P<name>…)` captures. A regex that lacks a capture
  its reader requires (`path`, `id`, …) is refused when the spec loads.
- `crates/planrun-parse/tests/spec_files.rs` checks every built-in spec against the schema.

## 3. Built-in recognizer kinds

Parameters in *italics* are optional.

| Kind | Matches | Produces | Parameters |
|---|---|---|---|
| `file-block` | a lead paragraph naming a path (and op), then one fence | `WriteFile` / `ApplyPatch` | `lead` (captures `path`, *`op`*), `ops` (op prefix → `write` \| `create` \| `overwrite` \| `patch`), *`default_op`*, *`file_name_from_manifest`* |
| `before-after` | lead (naming a path), fence, *separator paragraph*, fence | `ReplaceExact`, or `Manual(ProseEdit)` when `infer_path` finds no single entry | `lead` (captures `path`, or none with `infer_path = true`), *`separator`* (e.g. `^After:$`; absent = two fences back to back), *`file_name_from_manifest`*, *`series`* (a regex capturing `n`; the lead then captures `n` too, D49); a pathless lead may capture *`stem`* (D59) |
| `delete` | a sentence or inline `git rm` | `Delete` | `lead` (captures `path`), *`via_git`*, *`file_name_from_manifest`* |
| `generated` | a "generated, not by hand" sentence | `Generated` | `lead` (captures `path`) |
| `ceremony` | a fence directly after a given action whose body matches | nothing (absorbed) | `after` (`patch`), `body` |
| `script` | a fence in a shell language | `RunScript` / `Commit` | *`intents`* (`[{ intent, body?, lead? }]`: `commit` \| `gate` \| `preview` \| `apply` \| `verify` \| `other`), *`leads`* (regexes), *`first_block`*, *`line_expect`* (a regex capturing `expect`, D35, D48) |
| `run-line` | `` Run: `cmd` `` in a paragraph, with an optional expectation in the same paragraph | `RunScript` / `Commit` (one line) | `lead` (captures `cmd`, *`expected`*), `shell` (`pwsh` \| `bash`), *`exit`* table over `expected` (field `text`), *`intents`* (`[{ intent, body }]`) |
| `expected-block` | `Expected (…):` paragraph + `text` fence after a script | amends the script | `lead` (captures `spec`), `exit` table, `lines = "summary"` |
| `expected-prose` | `Expected: …` sentence after a script; with `list`, an `Expected:` paragraph and its list | amends the script | `lead` (captures `text`, unless `list`), `exit` table (`success` \| `failure` \| `any` \| `code:N`), *`list`* (D47) |
| `manual-prose` | a paragraph (+ the fence after it, with `takes_fence`) that describes what planrun cannot do exactly | `Manual(reason)` | `lead`, `reason` (`prose-edit` \| `output-dependent` \| `human-check`), *`takes_fence`* |

Notes on the kinds:

- `file-block`: the longest `ops` prefix of the captured `op` wins; `write` takes Create or
  Overwrite from the manifest. Once the lead matched, an op no prefix covers is
  `Malformed(unknown-file-op)`, a lead without its fence `Malformed(missing-block)`, and for
  `patch` a patch whose paths differ from the lead's `Malformed(patch-path)`.
- Paths from the manifest (D32): `file_name_from_manifest = true` resolves a lead path without
  `/` that is not an entry itself to the one entry with that file name, when that entry was
  written in full (not a sibling, not Bare); otherwise the path stays as written. `before-after`
  with `infer_path = true` takes the task's one entry that is not Test, Generated or Delete; with
  none or several (a Bare entry counts, but is never taken, D45), the lead and both fences
  become one `Manual(ProseEdit)`. Pattern entries are never candidates (D41). `infer_path` is never
  available for writes. Each resolved action carries its `PathOrigin`; `show` marks it.
- `run-line`: the command is the `cmd` capture, run as one line in `shell` (D39). The `expected`
  capture, when the lead has one and it matched, becomes a prose expectation through the `exit`
  table (no row matches: `any`). An empty command is `Malformed(missing-block)`.
- `before-after`: once the lead matched, a missing fence or separator is
  `Malformed(missing-block)`.
- `script`: `intent = "commit"` selects the `Commit` action kind, not a `ScriptIntent`; a commit
  script carries `ScriptIntent::Other`. The other intents set the `ScriptIntent`; no match is
  `verify`. A row matches the fence body (`body`), the paragraph directly before the fence
  (`lead`), or both; at least one is required (D29). Preview/apply pairs are rows like these.
- `script` `line_expect` (D35, D48): every line of a claimed fence that is not a whole-line
  comment and matches gives the script a comment expectation, exit not checked; an
  `expected-prose` right after it extends it instead of being `expected-twice`.
- `before-after` `series` (D49): a lead without a path continues the numbered edit directly
  before it in the step (`PathOrigin::PreviousEdit`), before `infer_path` is tried. A
  pathless lead's `stem` capture (`README edit 2`) must equal the file-name stem of the path
  it continues or infers, or the edit is `Manual(ProseEdit)` (D59).
- `expected-prose` `list` (D47): the list items at the lead's depth and the paragraphs, list
  items and `text` fences nested in them belong to the expectation; no list is
  `missing-block`.
- `script` guard (D28): with `leads` set, the kind claims a fence only when the paragraph
  directly before it matches one of `leads`, or, with `first_block = true`, when the fence is the
  first block of a written step (never the implicit first step, so never right after the
  manifest, and never in a step with a kept label, D46). A body intent never qualifies a fence
  on its own. Any other shell fence falls to the fallback, `Manual(Unrecognized)`. Without
  `leads` (the assay spec) every shell fence is a script.
- `expected-block` and `expected-prose` amend the script directly before them. Anything else there
  is `Malformed(expected-without-script)`; a second expectation on one script is
  `Malformed(expected-twice)`. `expected-block` needs a `text` fence (`missing-block`). An
  `expected-prose` sentence directly after a `Manual` joins that Manual's text instead (D38).
- Coverage counts the blocks of a recognizer's `Manual` (and of an expectation joined to a
  Manual) as manual, not as actions.
- **Exit tables are first match**: rows are tried in order and the first whose regex matches
  decides, so their order matters. Values: `success`, `failure`, `any` (`ExitExpect::Unknown`: not
  checked; prose is still confirmed, D11), `code:N`, and `code` (the row's regex captures `code`;
  0 is `success`). An `expected-block` spec no row matches is `Malformed(expected-spec)`; an
  `expected-prose` text no row matches is `any`.

The **fallback** (not configurable) runs when nothing matched: paragraph, list item, blockquote →
`Note` or `Manual(Marked)`; fence → `Manual(Unrecognized)` / `Manual(ForeignLanguage)`, a `text`
fence → `Note`; table, heading, rule → `Note`; HTML comment → dropped.

## 4. Adding an action kind

Rare (D4). The checklist, all in one change:

1. The variant in `planrun-model::ActionKind`, with docs.
2. Its handler in planrun-exec: `simulate` on the virtual tree (its dry-run behaviour and
   its part of a step transaction, D53), and its `DoneState` logic for resume (P5).
3. Its policy class.
4. Its journal record.
5. Its `show` rendering and coverage label.
6. A corpus plan that produces it, and an invalid-corpus file for each new error.

## 5. Adding a recognizer kind

```rust
pub struct InsertAfterFactory;

impl RecognizerFactory for InsertAfterFactory {
    fn kind(&self) -> &'static str { "insert-after" }

    fn build(&self, params: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        // Read and validate every parameter here: a bad regex fails when the spec loads.
        Ok(Arc::new(InsertAfter { lead: regex(params, "lead")?, id: id(params)? }))
    }
}

struct InsertAfter { lead: Regex, id: String }

impl Recognizer for InsertAfter {
    fn id(&self) -> &str { &self.id }

    fn recognize(&self, cx: &StepContext<'_>, cursor: &mut Cursor<'_>) -> Recognized {
        // 1. Peek; return Recognized::No without advancing unless the lead matches.
        // 2. Once the lead matched, a missing or wrong block is Recognized::Malformed, not No.
        // 3. Advance past exactly what was consumed; return the actions.
        Recognized::No
    }
}
```

Register it in `Registry::with_builtins`. Rules:

- **Pure.** No I/O, no clock, no state between calls. Same input, same output.
- **`No` leaves the cursor where it was.** Advance only on `Yes`/`Amend`.
- **Commit to the idiom early.** Once the distinctive lead matched, failures are `Malformed`
  errors. Falling through to `Manual` would hide a broken plan.
- **Tests**: unit tests on hand-built blocks, plus a corpus plan in `tests/corpus/synthetic` and
  an invalid-corpus file per `Malformed` code.

## 6. Adding a Format by hand

Implement `Format` (`detect`, `outline`, `recognizers`, `settings`) and register it with
`Registry::add_format`. Reuse recognizer instances from the factories for the step contents: only
the outline should be new. Every block of every task must fall in exactly one step range.

## 7. Execution-side extension points

| Trait | Registry | For |
|---|---|---|
| `ActionHandler` | `HandlerSet` | how an action kind is simulated on the virtual tree (D52, D53); the transaction writes the result. `DoneState` follows in P5 |
| `ShellSession` | one per `Shell` | another shell (`cmd`, `zsh`): a new `Shell` variant, a `Dialect` in `shell/` (launch arguments, prologue, block line, sentinel; D60), its discovery and probe (D63), and its splitter in `policy/` |
| `CommandClassifier` | the runner's | the tests' own classifier; `DefaultClassifier` is the built-in allowlist (D68), extended by adding rows to `policy/heads.rs` and their test rows, never from a format spec |
| `Ui` | the CLI | a TUI (ratatui) later, a non-interactive batch UI |
````


File `docs/roadmap.md` (full contents):

```markdown
# Roadmap

Each phase gets its own plan in `docs/plans/` (conventions: [plans/README.md](plans/README.md)),
reviewed by Rens before it is executed. A phase is done when its acceptance list holds and
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` are green.

The order puts the riskiest assumption first: that real plans can be parsed strictly, with full
accounting of every block.

## P0 — Starter package (done)

Workspace of four crates; the IR in `planrun-model` with its tests; the parse and exec extension
points as traits; the CLI argument surface; `formats/assay.toml`; the synthetic and invalid
corpus; the greeter fixture; these docs.

## P1 — Parse the assay format strictly (done)

The deliverable: `planrun check` and `planrun show` on assay plans, with precise diagnostics.

Suggested tasks (the plan may split or merge them):

1. **Source and document.** `source::load` (UTF-8, ≤ 64 MiB, BOM, CRLF/CR → LF, SHA-256 of the
   original bytes; adds `sha2`, D9). The document builder over pulldown-cmark's offset iterator:
   every `Block` variant, spans, list depth, `TextRef::Owned` for fences inside list items, and
   the `unclosed-fence` error. Unit tests on small strings, including a ```` ```` ```` fence that
   contains ```` ``` ```` fences, a fence inside a list item, CRLF input, and an empty fence body.
2. **Specs, registry, format selection, outline.** `Registry::add_spec` (deserialize, compile
   every regex, build recognizers through factories, reject unknown kinds and duplicate ids),
   `with_builtins` embedding `formats/assay.toml`, the generic `SpecFormat` (`detect`, `outline`:
   tasks after `tasks_after`, task ids and ordinals, phase, intro step, manifest, step ranges,
   `tasks_end_level`), selection with threshold and ties (`no-format`, `ambiguous-format`).
3. **Recognition.** The cursor loop with `No`/`Yes`/`Amend`/`Malformed` and the fallback; the
   kinds the assay spec uses: `file-block`, `ceremony`, `delete`, `generated`, `script`,
   `expected-block`, `expected-prose`. Patch parsing for `ApplyPatch` (target path from
   `diff --git`/`---`/`+++` agrees with the lead: `patch-path`; the `index` line → pre/post).
4. **Checks.** The rules `task-order` (with `duplicate-task-id`), `step-order`, `manifest`
   (`manifest-undeclared`, `manifest-untouched`, `manifest-write-mode`), `patch-shape`,
   `blob-chain`, `commit` (`commit-twice`, `commit-not-last`); the coverage report. The
   invalid-corpus test (`tests/corpus/invalid/README.md`), which also pins the codes of the
   outline and the recognizers (`bad-path`, `unknown-file-op`, `missing-block`, `patch-path`, …).
5. **CLI.** `planrun check` (diagnostics via `annotate-snippets`, coverage table, exit 0 / 1 on
   errors / 2 on usage) and `planrun show` (plan, task, task/step). An `insta` snapshot of the IR
   of `assay-style.md` and of `planrun check` output. The `#[ignore]`d local-corpus test.

Acceptance:

- `planrun check tests/corpus/synthetic/assay-style.md` exits 0; every block of every task is an
  action except the `**Goal:**` lines and prose (Notes); the absorbed ceremonies are listed as
  absorbed.
- Every `tests/corpus/invalid/*.md` yields exactly its pinned error code.
- The two local assay plans (`2026-11-05-s5-1-p1-routes-seam.md`,
  `2026-11-11-s5-1-p3c1-builder-core.md`) parse with zero errors. Expect to widen the `generated`
  pattern for "`Cargo.lock` changes: it is regenerated by cargo in the next step", and to see
  the P3c-1 plan's prose-heavy preamble as Notes.
- Every diagnostic shows file:line:col and the source line.

## P2 — The other three formats (done)

The decisions are D26–D40. Split in two plans on one schema (D40):

- **P2a (done).** The P1 outline fixes and `task-after-end` (D37); the spec schema (`required`,
  a marker that captures `change`, `siblings`, `ChangeKind::Generated`, `[checks]`,
  `[[step_marks]]`, `script` `leads`/`first_block`/intent `lead`); the kinds `run-line`,
  `before-after`, `manual-prose`; path resolution from the manifest (D32);
  `formats/superpowers.toml` and `before-after.toml` with IR snapshots of their synthetic plans.
  The superpowers and before-after local plans parse with zero errors. Plan:
  `docs/plans/2026-11-24-p2a-hand-written-formats.md`.
- **P2b (done).** First, pattern manifest entries: a range or glob item is one entry whose
  target is a `PathPattern` (D41, D42). Then the schema for hand-written DevReady plans: inline
  Files clauses (D43), a change per path (D44), Bare file names (D45), kept step labels (D46),
  `Expected:` with a list (D47), comment expectations that a sentence extends (D35, D48),
  numbered edit series (D49). Then `formats/devready.toml` with the `**Commit:**` Manual (D33),
  detection that outscores superpowers on both devready plans, the IR snapshot of its synthetic
  plan, and the Apply invariant in plan-grammar §5. All six local plans parse with zero errors.
  Plan: `docs/plans/2026-11-25-p2b-devready-format.md`.

Each plan lists the imperative Notes that remain in the real plans (D36). `expected-failure-step`
became step marks (D30); `preview-apply` became script intents by lead (D29).

## P3 — Workspace, git and dry-run (done)

`RepoPath` resolution with the symlink check; the per-task file cache; step transactions (D23);
line-ending policy (D10); pre-images in the git object database; the `git` wrapper (`apply
--check`/`apply` on stdin, `hash-object --path`, `status --porcelain`, `cat-file`); blob-id checks;
task preflight; `planrun run --dry-run` against a virtual tree. Integration tests on a temporary
copy of `tests/fixtures/greeter`: the dry run of `assay-style.md` passes; a fixture with
`src/lib.rs` edited fails preflight with the blob mismatch; a Create onto an existing file fails
before anything is written.

The decisions are D50–D59. Plan: `docs/plans/2026-11-25-p3-workspace-dry-run.md`. First the P2b
review items (series stems D59, `run-from`, negated failures, `write-mode-unknown` D58) and the
`RepoPath` hardening (D51); then the git wrapper, the workspace and its one writer, line endings
(D54), the virtual tree (D52), step transactions as prepare and commit (D53), preflight with
scope (D50) and the dry run (D55). The per-task file cache is the virtual tree: a task's paths are
read once, on first use.

## P4 — Shells, expectations, policy

Persistent pwsh and bash sessions (D24) with the sentinel protocol: variables persist across
blocks, a `throw` ends the block and not the session, output streams. The expectation matcher
(summary-line subsequence with `…`, exit expectations, prose → confirmation, D11). The command
classifier (D13). Expected failures skipped by default (D12). Tests for each, with pwsh and bash
when present (skipped with a message when not).

The decisions are D60–D70. Plan: `docs/plans/2026-11-25-p4-shells-expectations-policy.md`. First
the decisions and the `unsafe` lints (D64); then the sessions (the protocol D60, how a block
ends D61, the environment D62, which shells D63, the process tree D64); the matcher (D66); the
classifier (D68) and the policy decisions (D67, D69); the classes in the dry run. The candidate
(a script working directory from a run lead) is deferred (D70).

## P5 — Runner, journal, resume, interactive CLI

The runner state machine; the commit gate (D22); the JSONL journal (D18); resume with `DoneState`
(D14); `run`, `resume`, `status`; the line-prompt UI (D15) with view/diff/shell/skip/quit. End to
end: `planrun run tests/corpus/synthetic/assay-style.md --auto read,build,modify` on a fixture copy
produces the same two commits (same trees) as the fixture's history; killing the process mid-task
and resuming completes it without repeating a write.

## P6 — Dogfood and polish

planrun runs its own later phase plans. Candidates, each its own decision: a ratatui UI, a
sidecar IR from planbuild.py, `.planrun/formats/` discovery polish, a non-interactive batch mode
for CI.
```


**Step 2 — verify**

Targeted: the spec files against extending.md:

```powershell
cargo test -p planrun-parse --test spec_files
```


Expected (exit code 0; summary lines):

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```


Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```


Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```


**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- CLAUDE.md docs/architecture.md docs/extending.md docs/roadmap.md
  $verified = [ordered]@{
      'CLAUDE.md' = '261e3ac1a90a4835e835376726dd9887b1c3f9ee'
      'docs/architecture.md' = 'a05fd7bc1c4dd36212550c84fccdcc7aa0f75728'
      'docs/extending.md' = 'e59cf07e4dd9d108e9ddabd75f2c528a925aea19'
      'docs/roadmap.md' = '09d4595b11330d8a188934a54195a41b44fecccd'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P4-T7: architecture, extending, roadmap and CLAUDE.md for shells, expectations and policy" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```


Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

## P4 green state

Run from the repository root after P4-T7:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-parse --test local_corpus -- --ignored local_corpus }
```

Expected: every `test result:` line ok; the last one `1 passed`, with `6 parsed, 0 skipped` in the local-corpus output.

User-visible change: `planrun run --dry-run <plan>` shows each script's and commit's class and what the default policy would do with it.

## After execution

- `git log --oneline -9` lists the commit of this plan, then `P4-T1` to `P4-T7`.
- `cargo run -q -p planrun-cli -- check docs/plans/2026-11-25-p4-shells-expectations-policy.md` exits 0: this plan is an assay plan.
- Compare the executed tree with the verified one: `git ls-tree -r HEAD` against the scratch tree the plan was generated from (the handover's open item 0 passed every test).
- Write the handover `docs/handovers/<date>-p4-to-p5.md` in a separate session or commit after review. Items for it: the default timeout (D65); restarting a dead session, and what the user is told (D61); warning when a block leaves the repository (`ScriptRun::cwd`, D60); the prompt for every `Decision` (D69); the commit gate over `Verdict` and `Skip` (D22, D66, D67); whether `cargo xtask` and a nested `pwsh -File <repo script>` should get a class of their own (33 of the local plans' 111 scripts are Destructive, 49 of those reasons from these two); the deferred run-from candidate (D70); and the P3→P4 handover's open items 1–11, still open.
