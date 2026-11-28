//! The `planrun` binary (docs/architecture.md §7).

#![forbid(unsafe_code)]

mod dry_run;
#[cfg(feature = "e2e")]
mod e2e;
mod list;
mod render;
mod run;
mod ui;
mod update;
mod view;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use planrun_exec::discover::{self, Discovery};
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
        /// The plan file, or a name the discovered plans' file names contain (D97).
        plan: PathBuf,
        /// Force a format id instead of detecting one.
        #[arg(long)]
        format: Option<String>,
    },
    /// Print the typed plan, or one task (`P1-T2`) or step (`P1-T2/1`, `P1-T2/intro`).
    Show {
        /// The plan file, or a name the discovered plans' file names contain (D97).
        plan: PathBuf,
        /// A task or task/step.
        target: Option<String>,
        /// Force a format id.
        #[arg(long)]
        format: Option<String>,
    },
    /// Execute a plan in the git repository of the current directory, task by task.
    Run {
        /// The plan file, or a name the discovered plans' file names contain (D97).
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
        /// The plan file, or a name the discovered plans' file names contain (D97).
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
        /// The plan file, or a name the discovered plans' file names contain (D97).
        plan: PathBuf,
        /// Force a format id.
        #[arg(long)]
        format: Option<String>,
    },
    /// The plan files of the current directory's repository, with their formats and journals
    /// (D96, D97). Changes nothing.
    List,
    /// The TUI over the repository of the current directory (docs/tui-design.md, D84–D94).
    Tui {
        /// The directory of plan files; `docs/plans` under the repository's root when not
        /// given.
        plans_dir: Option<PathBuf>,
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
            with_plan(plan, format.as_deref(), &mut out, |file, p, out| {
                run::run(file, p, &opts, out)
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
            with_plan(plan, format.as_deref(), &mut out, |file, p, out| {
                run::resume(file, p, &opts, out)
            })
        }
        Command::Status { plan, format } => {
            with_plan(plan, format.as_deref(), &mut out, |file, p, out| {
                run::status(file, p, out)
            })
        }
        Command::List => match setup(&mut out) {
            Ok(setup) => list::list(&setup, &mut out),
            Err(code) => code,
        },
        Command::Update { check } => update::run(*check, &mut out),
        Command::Tui { plans_dir } => tui(plans_dir.clone(), &mut out),
    };
    if code == OK {
        print!("{out}");
    } else {
        eprint!("{out}");
    }
    ExitCode::from(code)
}

/// `planrun tui`: exit 0 when it quits, 2 when it cannot start (no terminal, no repository).
fn tui(plans_dir: Option<PathBuf>, out: &mut String) -> u8 {
    let dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            out.push_str(&format!("planrun: the current directory: {e}\n"));
            return USAGE;
        }
    };
    let host = planrun_tui::Host {
        dir,
        plans: plans_dir,
        version: env!("CARGO_PKG_VERSION").to_owned(),
        update: Some(Box::new(update::state)),
    };
    match planrun_tui::run(host) {
        Ok(()) => OK,
        Err(e) => {
            out.push_str(&format!("planrun: {e}\n"));
            USAGE
        }
    }
}

/// Loads the plan and hands it to `f`; a plan that does not parse runs nothing (D3).
fn with_plan(
    path: &Path,
    format: Option<&str>,
    out: &mut String,
    f: impl FnOnce(&Path, &planrun_model::Plan, &mut String) -> u8,
) -> u8 {
    match load(path, format, out) {
        Loaded::Parsed(parsed, _, file) => f(&file, &parsed.plan, out),
        Loaded::Failed(code) => code,
    }
}

/// A plan read and parsed, with its format's description (D95) and the file it was read from
/// (D97), or the exit code when it was not.
enum Loaded {
    Parsed(Box<Parsed>, String, PathBuf),
    Failed(u8),
}

/// What discovery found from the current directory (D96), and the registry built from it.
pub(crate) struct Setup {
    /// What discovery found.
    pub(crate) discovery: Discovery,
    /// The built-in formats with the discovered ones on top (D95).
    pub(crate) registry: Registry,
}

/// Discovers from the current directory and layers the formats found over the built-ins; a
/// problem is a usage problem (exit code 2).
pub(crate) fn setup(out: &mut String) -> Result<Setup, u8> {
    let here = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            out.push_str(&format!("planrun: the current directory: {e}\n"));
            return Err(USAGE);
        }
    };
    let discovery = match Discovery::open(&here, discover::user_dir().as_deref()) {
        Ok(discovery) => discovery,
        Err(e) => {
            out.push_str(&format!("planrun: error[{}]: {e}\n", e.code()));
            return Err(USAGE);
        }
    };
    let layers = discovery
        .formats()
        .iter()
        .map(|s| (s.id.as_str(), s.text.as_str(), &s.source));
    match Registry::with_layers(layers) {
        Ok(registry) => Ok(Setup {
            discovery,
            registry,
        }),
        Err(e) => {
            out.push_str(&format!("planrun: the formats: {e}\n"));
            Err(USAGE)
        }
    }
}

fn load(path: &Path, format: Option<&str>, out: &mut String) -> Loaded {
    let setup = match setup(out) {
        Ok(setup) => setup,
        Err(code) => return Loaded::Failed(code),
    };
    let resolved = match setup.discovery.resolve(path, &[]) {
        Ok(resolved) => resolved,
        Err(e) => {
            out.push_str(&format!("planrun: error[{}]: {e}\n", e.code()));
            return Loaded::Failed(USAGE);
        }
    };
    if resolved.by_name {
        eprintln!("planrun: `{}` is {}", path.display(), resolved.shown);
    }
    let name = resolved.shown.clone();
    let bytes = match std::fs::read(&resolved.path) {
        Ok(bytes) => bytes,
        Err(e) => {
            out.push_str(&format!("planrun: {name}: {e}\n"));
            return Loaded::Failed(USAGE);
        }
    };
    match parse_plan(&setup.registry, &name, &bytes, format) {
        Ok(parsed) => {
            let id = parsed.plan.format.0.to_string();
            let label = setup
                .registry
                .origin(&id)
                .map_or_else(|| id.clone(), |o| o.describe(&id, setup.discovery.root()));
            Loaded::Parsed(Box::new(parsed), label, resolved.path)
        }
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
    let (parsed, label) = match load(path, format, out) {
        Loaded::Parsed(parsed, label, _) => (parsed, label),
        Loaded::Failed(code) => return code,
    };
    let plan = &parsed.plan;
    let name = plan.source.name.to_string();
    render::diagnostics(&name, plan.source.text(), &parsed.diagnostics.0, out);
    render::coverage(plan, &parsed.coverage, out);
    out.push_str(&format!(
        "planrun: {name}: ok ({label}, {} tasks)\n",
        plan.tasks.len()
    ));
    OK
}

fn show(path: &Path, target: Option<&str>, format: Option<&str>, out: &mut String) -> u8 {
    let (parsed, label) = match load(path, format, out) {
        Loaded::Parsed(parsed, label, _) => (parsed, label),
        Loaded::Failed(code) => return code,
    };
    let viewer = Viewer::new(&parsed.plan);
    let Some(target) = target else {
        let mut whole = viewer.plan();
        whole.format = label;
        view::plan_text(&whole, out);
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
        Loaded::Parsed(parsed, ..) => parsed,
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
