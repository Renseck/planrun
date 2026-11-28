//! Background work (docs/tui-design.md §2, decision D90): each job runs on a thread of its own
//! and sends one result back; the render thread never waits on git, a shell or the network.
//! Each result carries the fingerprint of its input, so a result for an input that has changed
//! since is shown as stale. Background checks never replace preflight.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use planrun_exec::discover::Discovery;
use planrun_exec::journal::{self, End, Line, Record};
use planrun_exec::lock;
use planrun_exec::preflight::{self, DryRun};
use planrun_exec::runner::resume::analyse;
use planrun_exec::workspace::Workspace;
use planrun_parse::{Parsed, Registry, Severity, parse_plan, source};

/// What the TUI knows about the repository and itself, shared with every job.
#[derive(Debug, Clone)]
pub struct Context {
    /// The repository root (canonical).
    pub root: PathBuf,
    /// Its git directory (`rev-parse --absolute-git-dir`).
    pub git_dir: PathBuf,
    /// The repository's name, the first crumb.
    pub repo: String,
    /// The target as the footer shows it.
    pub target: String,
    /// The plans directory (absolute).
    pub plans_dir: PathBuf,
    /// The plans directory as the footer shows it.
    pub plans_label: String,
    /// planrun's version.
    pub version: String,
    /// Added to every git call and every session (the tests keep the user's configuration out).
    pub env: Vec<(OsString, OsString)>,
    /// What discovery found (D96): the format specs every check layers over the built-ins.
    pub discovery: Discovery,
}

impl Context {
    /// The journal directory, `<git-dir>/planrun`.
    pub fn journal_dir(&self) -> PathBuf {
        journal::dir(&self.git_dir)
    }

    /// The built-in formats with the discovered ones layered on top (D95).
    pub fn registry(&self) -> Result<Registry, String> {
        let layers = self
            .discovery
            .formats()
            .iter()
            .map(|s| (s.id.as_str(), s.text.as_str(), &s.source));
        Registry::with_layers(layers).map_err(|e| format!("the formats: {e}"))
    }

    /// The plan's path as a journal records it: repository-relative with forward slashes, or
    /// canonical when it is outside the repository (D76).
    pub fn plan_path(&self, file: &Path) -> String {
        let canonical = std::fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf());
        match canonical.strip_prefix(&self.root) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => canonical.display().to_string(),
        }
    }
}

/// A file's fingerprint for change detection: its length and modification time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stamp {
    /// Bytes.
    pub len: u64,
    /// Milliseconds since the Unix epoch.
    pub modified: u64,
}

impl Stamp {
    /// The stamp of `path`, or `None` when it cannot be read.
    pub fn of(path: &Path) -> Option<Stamp> {
        let meta = std::fs::metadata(path).ok()?;
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));
        Some(Stamp {
            len: meta.len(),
            modified,
        })
    }
}

/// The five states of background work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bg<T> {
    /// Started; no result yet.
    Running,
    /// The result for the input as it is.
    Current(T),
    /// A result for an input that has changed since; a new one is on its way.
    Stale(T),
    /// The input is not there (no journal directory, no update check).
    Unavailable(String),
    /// The work failed.
    Failed(String),
}

impl<T> Bg<T> {
    /// The result, current or stale.
    pub fn value(&self) -> Option<&T> {
        match self {
            Bg::Current(v) | Bg::Stale(v) => Some(v),
            _ => None,
        }
    }

    /// The state with its result marked stale.
    pub fn stale(self) -> Bg<T> {
        match self {
            Bg::Current(v) | Bg::Stale(v) => Bg::Stale(v),
            _ => Bg::Running,
        }
    }
}

/// What `planrun update --check` found (D83); planrun-cli runs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateState {
    /// Not checked (yet, or not at all).
    Unchecked,
    /// Up to date.
    Current,
    /// A newer release.
    Available(String),
    /// The check failed.
    Failed,
}

/// The update check planrun-cli hands in.
pub type UpdateCheck = Box<dyn FnOnce() -> UpdateState + Send>;

/// One diagnostic of a plan check, with its line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diag {
    /// An error (the plan does not parse), or a warning or note.
    pub error: bool,
    /// Its stable code.
    pub code: &'static str,
    /// Its message.
    pub message: String,
    /// Its 1-based line in the plan, when it has a span.
    pub line: Option<u32>,
}

/// The journal of a plan, as the Plans view shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunState {
    /// No journal, or an empty one.
    NotRun,
    /// The last chain completed; when.
    Done(u64),
    /// The last chain stopped; resume goes on.
    Stopped,
    /// The last chain has no `run-end`: killed, or running now.
    Interrupted,
    /// A journal of this stem written for another plan path (D76).
    Foreign(String),
    /// The journal could not be read.
    Unreadable(String),
}

/// A plan file, checked.
#[derive(Debug, Clone)]
pub struct PlanFile {
    /// Its lines.
    pub lines: usize,
    /// The parsed plan, or `None` when it has errors.
    pub parsed: Option<Arc<Parsed>>,
    /// Every diagnostic, errors included, in source order.
    pub diags: Vec<Diag>,
    /// No format detects it: not a plan.
    pub not_a_plan: bool,
    /// Its journal.
    pub run: RunState,
}

impl PlanFile {
    /// Errors, warnings.
    pub fn counts(&self) -> (usize, usize) {
        let errors = self.diags.iter().filter(|d| d.error).count();
        let warnings = self.diags.iter().filter(|d| !d.error).count();
        (errors, warnings)
    }
}

/// One piece of background work.
pub enum Job {
    /// Lists the plans directory's `*.md` files.
    Scan,
    /// Parses and checks a plan, and reads its journal.
    Check {
        /// The file.
        path: PathBuf,
        /// Its stamp when the check started.
        stamp: Stamp,
    },
    /// Probes `lock.live` (D90).
    Live,
    /// `planrun update --check`.
    Update(UpdateCheck),
    /// `git diff HEAD` of a running task's paths, for CHANGES.
    Changes {
        /// The task, by position.
        task: usize,
        /// Its paths.
        paths: Vec<String>,
    },
    /// `git diff HEAD` and the untracked files, for `[d]`.
    WorkTree,
    /// Every run in the journal directory, for History.
    Journals,
    /// What resume would find for the journal's last chain (D85), at the clock's time.
    Preview {
        /// The journal.
        journal: PathBuf,
        /// The plan's path as recorded.
        plan: String,
        /// The time it is read at.
        now: u64,
    },
    /// The dry run of a plan (D55), for Inspect.
    DryRun {
        /// The plan file.
        path: PathBuf,
        /// Its parsed plan.
        parsed: Arc<Parsed>,
    },
}

/// What a job found.
#[derive(Debug, Clone)]
pub enum Done {
    /// The plans directory's files with their stamps, sorted by name.
    Scanned(Result<Vec<(PathBuf, Stamp)>, String>),
    /// A plan's check.
    Checked {
        /// The file.
        path: PathBuf,
        /// The stamp it was checked at.
        stamp: Stamp,
        /// What it found, or why it could not read the file.
        file: Result<PlanFile, String>,
    },
    /// A run holds the lock now (with the pid in `lock.pid`), or not.
    Live(Result<Option<Option<u32>>, String>),
    /// The update check.
    Update(UpdateState),
    /// A task's diff against HEAD, one per file that has one.
    Changes {
        /// The task, by position.
        task: usize,
        /// The diffs.
        files: Vec<crate::diff::FileDiff>,
    },
    /// The working tree against HEAD, and the untracked files.
    WorkTree(Result<(Vec<crate::diff::FileDiff>, Vec<String>), String>),
    /// Every run, newest first.
    Journals(Result<Vec<crate::history::RunEntry>, String>),
    /// The resume preview of a journal.
    Preview {
        /// The journal.
        journal: PathBuf,
        /// What it found.
        result: Result<Box<crate::preview::PreviewData>, String>,
    },
    /// A plan's dry run.
    DryRun {
        /// The plan file.
        path: PathBuf,
        /// The fingerprint of the plan it ran on.
        fingerprint: String,
        /// The report, or why it could not run.
        result: Result<DryRun, String>,
    },
}

/// What a job sends when it panicked: its own failed state, with the panic's message (§2).
pub type OnFailure = Box<dyn FnOnce(String) -> Done + Send>;

/// A panic's message, from its payload.
pub fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        return (*s).to_owned();
    }
    if let Some(s) = payload.downcast_ref::<String>() {
        return s.clone();
    }
    "(no message)".to_owned()
}

impl Job {
    /// The result to send if the job panics: the failed state of what it works on.
    pub fn on_failure(&self) -> OnFailure {
        match self {
            Job::Scan => Box::new(|e| Done::Scanned(Err(e))),
            Job::Check { path, stamp } => {
                let (path, stamp) = (path.clone(), *stamp);
                Box::new(move |e| Done::Checked {
                    path,
                    stamp,
                    file: Err(e),
                })
            }
            Job::Live => Box::new(|e| Done::Live(Err(e))),
            Job::Update(_) => Box::new(|_| Done::Update(UpdateState::Failed)),
            Job::WorkTree => Box::new(|e| Done::WorkTree(Err(e))),
            Job::Journals => Box::new(|e| Done::Journals(Err(e))),
            Job::Preview { journal, .. } => {
                let journal = journal.clone();
                Box::new(move |e| Done::Preview {
                    journal,
                    result: Err(e),
                })
            }
            Job::Changes { task, .. } => {
                // CHANGES falls back to the planned diff.
                let task = *task;
                Box::new(move |_| Done::Changes {
                    task,
                    files: Vec::new(),
                })
            }
            Job::DryRun { path, parsed } => {
                let path = path.clone();
                let fingerprint = parsed.plan.source.fingerprint.to_string();
                Box::new(move |e| Done::DryRun {
                    path,
                    fingerprint,
                    result: Err(e),
                })
            }
        }
    }

    /// Does the work.
    pub fn run(self, ctx: &Context) -> Done {
        match self {
            Job::Scan => Done::Scanned(scan(&ctx.plans_dir)),
            Job::Check { path, stamp } => {
                let file = check(ctx, &path);
                Done::Checked { path, stamp, file }
            }
            Job::Live => Done::Live(live(ctx)),
            Job::Update(check) => Done::Update(check()),
            Job::Changes { task, paths } => Done::Changes {
                task,
                files: changes(ctx, &paths),
            },
            Job::WorkTree => {
                let untracked = git_out(ctx, &["ls-files", "--others", "--exclude-standard"])
                    .lines()
                    .map(str::to_owned)
                    .collect();
                Done::WorkTree(Ok((changes(ctx, &[".".to_owned()]), untracked)))
            }
            Job::Journals => Done::Journals(crate::history::scan(&ctx.journal_dir())),
            Job::Preview { journal, plan, now } => {
                let result = preview(ctx, &journal, &plan, now).map(Box::new);
                Done::Preview { journal, result }
            }
            Job::DryRun { path, parsed } => {
                let plan = &parsed.plan;
                let result = Workspace::open(&ctx.root, ctx.env.clone())
                    .and_then(|ws| preflight::dry_run(&ws, plan))
                    .map_err(|e| e.to_string());
                Done::DryRun {
                    path,
                    fingerprint: plan.source.fingerprint.to_string(),
                    result,
                }
            }
        }
    }
}

/// Reads what resume would find: the plan parsed now, the journal's last chain, the disk.
fn preview(
    ctx: &Context,
    journal_path: &Path,
    plan: &str,
    now: u64,
) -> Result<crate::preview::PreviewData, String> {
    let path = if Path::new(plan).is_absolute() {
        PathBuf::from(plan)
    } else {
        ctx.root.join(plan)
    };
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let registry = ctx.registry()?;
    let name = path.display().to_string();
    let parsed = parse_plan(&registry, &name, &bytes, None)
        .map_err(|_| format!("{plan} does not parse any more: resume would refuse it"))?;
    let lines = journal::read(journal_path).map_err(|e| e.to_string())?;
    let analysis = analyse(&lines).ok_or_else(|| "the journal has no run".to_owned())?;
    let ws = Workspace::open(&ctx.root, ctx.env.clone()).map_err(|e| e.to_string())?;
    let preview = planrun_exec::runner::preview::preview(&ws, &parsed.plan, &analysis)
        .map_err(|e| e.to_string())?;
    let untracked = git_out(ctx, &["ls-files", "--others", "--exclude-standard"])
        .lines()
        .map(str::to_owned)
        .collect();
    Ok(crate::preview::PreviewData {
        taken: now,
        path,
        parsed: Arc::new(parsed),
        analysis,
        preview,
        untracked,
    })
}

/// The output of `git --no-optional-locks args…`; empty when git fails.
fn git_out(ctx: &Context, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .arg("--no-optional-locks")
        .args(args)
        .current_dir(&ctx.root)
        .envs(ctx.env.iter().map(|(k, v)| (k, v)))
        .output();
    match out {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        _ => String::new(),
    }
}

/// `git --no-optional-locks diff HEAD -- paths`, read-only; empty when git fails.
fn changes(ctx: &Context, paths: &[String]) -> Vec<crate::diff::FileDiff> {
    if paths.is_empty() {
        return Vec::new();
    }
    let out = std::process::Command::new("git")
        .args(["--no-optional-locks", "diff", "--no-color", "HEAD", "--"])
        .args(paths)
        .current_dir(&ctx.root)
        .envs(ctx.env.iter().map(|(k, v)| (k, v)))
        .output();
    match out {
        Ok(out) if out.status.success() => {
            crate::diff::parse(&String::from_utf8_lossy(&out.stdout))
        }
        _ => Vec::new(),
    }
}

fn scan(dir: &Path) -> Result<Vec<(PathBuf, Stamp)>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut files: Vec<(PathBuf, Stamp)> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "md"))
        .filter_map(|p| Stamp::of(&p).map(|s| (p, s)))
        .collect();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

fn check(ctx: &Context, path: &Path) -> Result<PlanFile, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let registry = ctx.registry()?;
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    let lines = bytes.iter().filter(|&&b| b == b'\n').count();
    let run = journal_state(ctx, path);
    let file = match parse_plan(&registry, &name, &bytes, None) {
        Ok(parsed) => {
            let diags = diags(&parsed.plan.source, &parsed.diagnostics.0);
            PlanFile {
                lines,
                parsed: Some(Arc::new(parsed)),
                diags,
                not_a_plan: false,
                run,
            }
        }
        Err(diagnostics) => {
            let not_a_plan = diagnostics.0.iter().any(|d| d.code == "no-format");
            let source = source::load(&name, &bytes).ok();
            let diags = match &source {
                Some(source) => diags(source, &diagnostics.0),
                None => diags_without_lines(&diagnostics.0),
            };
            PlanFile {
                lines,
                parsed: None,
                diags,
                not_a_plan,
                run,
            }
        }
    };
    Ok(file)
}

fn diags(source: &planrun_model::PlanSource, all: &[planrun_parse::Diagnostic]) -> Vec<Diag> {
    all.iter()
        .map(|d| Diag {
            error: d.severity == Severity::Error,
            code: d.code,
            message: d.message.clone(),
            line: d.span.map(|s| source.line_col(s.start).line),
        })
        .collect()
}

fn diags_without_lines(all: &[planrun_parse::Diagnostic]) -> Vec<Diag> {
    all.iter()
        .map(|d| Diag {
            error: d.severity == Severity::Error,
            code: d.code,
            message: d.message.clone(),
            line: None,
        })
        .collect()
}

/// The plan's journal: its stem's file in `<git-dir>/planrun` (D76).
fn journal_state(ctx: &Context, path: &Path) -> RunState {
    let stem = path
        .file_stem()
        .map_or_else(|| "plan".to_owned(), |s| s.to_string_lossy().into_owned());
    let lines = match journal::read(&journal::path_for(&ctx.git_dir, &stem)) {
        Ok(lines) => lines,
        Err(e) => return RunState::Unreadable(e.to_string()),
    };
    run_state(&lines, &ctx.plan_path(path))
}

/// What the journal's last chain says about the plan at `plan_path`.
pub fn run_state(lines: &[Line], plan_path: &str) -> RunState {
    let foreign = lines.iter().find_map(|l| match &l.record {
        Record::Run { plan, .. } if plan != plan_path => Some(plan.clone()),
        _ => None,
    });
    if let Some(other) = foreign {
        return RunState::Foreign(other);
    }
    let Some(analysis) = analyse(lines) else {
        return RunState::NotRun;
    };
    match analysis.end {
        Some(End::Completed) => RunState::Done(lines.last().map_or(0, |l| l.ms)),
        Some(End::Stopped) => RunState::Stopped,
        None => RunState::Interrupted,
    }
}

fn live(ctx: &Context) -> Result<Option<Option<u32>>, String> {
    let dir = ctx.journal_dir();
    match lock::live(&dir) {
        Ok(true) => Ok(Some(lock::holder(&dir))),
        Ok(false) => Ok(None),
        Err(e) => Err(format!("{}: {e}", dir.join("lock.live").display())),
    }
}
