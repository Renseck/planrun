//! The `planrun` binary (docs/architecture.md §7).

#![forbid(unsafe_code)]

mod dry_run;
#[cfg(feature = "e2e")]
mod e2e;
mod render;
mod run;
mod ui;
mod update;
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
    /// Execute a plan in the git repository of the current directory, task by task.
    Run {
        /// The plan file.
        plan: PathBuf,
        /// Parse, check and validate against a virtual tree; change nothing, run nothing.
        #[arg(long)]
        dry_run: bool,
        /// Force a format id.
        #[arg(long)]
        format: Option<String>,
        /// Script classes that run without asking: read, build, modify (D69).
        #[arg(long)]
        auto: Option<String>,
        /// Run the verifications that expect to fail (D67).
        #[arg(long)]
        run_expected_failures: bool,
        /// The first task to run.
        #[arg(long)]
        from: Option<String>,
        /// The last task to run.
        #[arg(long)]
        to: Option<String>,
        /// Give up the plan's unfinished run; its journal is kept under another name.
        #[arg(long)]
        restart: bool,
    },
    /// Go on with the plan's interrupted or stopped run (D14).
    Resume {
        /// The plan file.
        plan: PathBuf,
        /// Force a format id.
        #[arg(long)]
        format: Option<String>,
        /// Script classes that run without asking; the journaled run's when not given.
        #[arg(long)]
        auto: Option<String>,
        /// Run the verifications that expect to fail.
        #[arg(long)]
        run_expected_failures: bool,
        /// The first task (the journaled run's when not given).
        #[arg(long)]
        from: Option<String>,
        /// The last task (the journaled run's when not given).
        #[arg(long)]
        to: Option<String>,
    },
    /// What the plan's journal says: the tasks done, the one interrupted. Changes nothing.
    Status {
        /// The plan file.
        plan: PathBuf,
        /// Force a format id.
        #[arg(long)]
        format: Option<String>,
    },
    /// Install the latest GitHub release over this executable when it is newer (D83).
    Update {
        /// Report whether a newer release exists; change nothing.
        #[arg(long)]
        check: bool,
    },
}

/// Exit codes: 0 fine, 1 the plan has errors or the run stopped, 2 a usage or I/O problem.
const OK: u8 = 0;
const PLAN_ERRORS: u8 = 1;
const USAGE: u8 = 2;

fn main() -> ExitCode {
    update::cleanup_beside_current_exe();
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
            ..
        } => run_dry(plan, format.as_deref(), &mut out),
        Command::Run {
            plan,
            format,
            auto,
            run_expected_failures,
            from,
            to,
            restart,
            ..
        } => {
            let opts = run::Options {
                auto: auto.clone(),
                run_expected_failures: *run_expected_failures,
                from: from.clone(),
                to: to.clone(),
                restart: *restart,
            };
            with_plan(plan, format.as_deref(), &mut out, |p, out| {
                run::run(plan, p, &opts, out)
            })
        }
        Command::Resume {
            plan,
            format,
            auto,
            run_expected_failures,
            from,
            to,
        } => {
            let opts = run::Options {
                auto: auto.clone(),
                run_expected_failures: *run_expected_failures,
                from: from.clone(),
                to: to.clone(),
                restart: false,
            };
            with_plan(plan, format.as_deref(), &mut out, |p, out| {
                run::resume(plan, p, &opts, out)
            })
        }
        Command::Status { plan, format } => {
            with_plan(plan, format.as_deref(), &mut out, |p, out| {
                run::status(plan, p, out)
            })
        }
        Command::Update { check } => update::run(*check, &mut out),
    };
    if code == OK {
        print!("{out}");
    } else {
        eprint!("{out}");
    }
    ExitCode::from(code)
}

/// Loads the plan and hands it to `f`; a plan that does not parse runs nothing (D3).
fn with_plan(
    path: &Path,
    format: Option<&str>,
    out: &mut String,
    f: impl FnOnce(&planrun_model::Plan, &mut String) -> u8,
) -> u8 {
    match load(path, format, out) {
        Loaded::Parsed(parsed) => f(&parsed.plan, out),
        Loaded::Failed(code) => code,
    }
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
