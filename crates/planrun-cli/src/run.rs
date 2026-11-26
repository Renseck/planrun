//! `planrun run`, `resume` and `status` (docs/architecture.md §7; decisions D74, D76, D77):
//! the lock, the journal of this plan, the policy, and the runner with the line prompts.

use std::path::{Path, PathBuf};

use planrun_exec::journal::{self, End, Journal, Line, Record};
use planrun_exec::lock::{self, LockError};
use planrun_exec::policy::{Policy, PolicyClass};
use planrun_exec::runner::resume::{Analysis, analyse};
use planrun_exec::runner::{Finished, RunConfig, Runner};
use planrun_exec::shell::{SessionConfig, install_interrupt_handler};
use planrun_exec::workspace::Workspace;
use planrun_model::Plan;

use crate::ui::LineUi;
use crate::{OK, PLAN_ERRORS, USAGE};

/// The flags of `run` and `resume`.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// `--auto`.
    pub auto: Option<String>,
    /// `--run-expected-failures`.
    pub run_expected_failures: bool,
    /// `--from`.
    pub from: Option<String>,
    /// `--to`.
    pub to: Option<String>,
    /// `--restart` (run only): give up an unfinished journal, keeping it under another name.
    pub restart: bool,
}

/// The repository, the plan's journal and its recorded path.
struct Context {
    ws: Workspace,
    git_dir: PathBuf,
    journal: PathBuf,
    plan_path: String,
}

fn context(plan_file: &Path, out: &mut String) -> Result<Context, u8> {
    let here = std::env::current_dir().map_err(|e| {
        out.push_str(&format!("planrun: the current directory: {e}\n"));
        USAGE
    })?;
    let ws = Workspace::open(&here, Vec::new()).map_err(|e| {
        out.push_str(&format!("planrun: {e}\n"));
        USAGE
    })?;
    let git_dir = ws.git().git_dir().map_err(|e| {
        out.push_str(&format!("planrun: {e}\n"));
        USAGE
    })?;
    let stem = plan_file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "plan".to_owned());
    let canonical = std::fs::canonicalize(plan_file).unwrap_or_else(|_| plan_file.to_path_buf());
    let plan_path = match canonical.strip_prefix(ws.root()) {
        Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
        Err(_) => canonical.display().to_string(),
    };
    Ok(Context {
        journal: journal::path_for(&git_dir, &stem),
        ws,
        git_dir,
        plan_path,
    })
}

/// The journal's lines, after checking that it belongs to this plan (D76): a journal of the
/// same stem that another plan path wrote is a hard stop.
fn lines(ctx: &Context, out: &mut String) -> Result<Vec<Line>, u8> {
    let lines = journal::read(&ctx.journal).map_err(|e| {
        out.push_str(&format!("planrun: {e}\n"));
        USAGE
    })?;
    let other = lines.iter().find_map(|l| match &l.record {
        Record::Run { plan, .. } if *plan != ctx.plan_path => Some(plan.clone()),
        _ => None,
    });
    if let Some(other) = other {
        out.push_str(&format!(
            "planrun: the journal {} belongs to the plan {other}, not to {}: two plans share \
             a file name; rename one of them\n",
            ctx.journal.display(),
            ctx.plan_path
        ));
        return Err(USAGE);
    }
    Ok(lines)
}

fn take_lock(ctx: &Context, out: &mut String) -> Result<lock::RunLock, u8> {
    lock::acquire(&journal::dir(&ctx.git_dir)).map_err(|e| {
        match e {
            LockError::Held { .. } => out.push_str(&format!("planrun: {e}\n")),
            LockError::Io(_) => out.push_str(&format!("planrun: {e}\n")),
        }
        USAGE
    })
}

fn task_index(
    plan: &Plan,
    id: Option<&str>,
    flag: &str,
    out: &mut String,
) -> Result<Option<usize>, u8> {
    let Some(id) = id else {
        return Ok(None);
    };
    match plan.tasks.iter().position(|t| &*t.id.raw == id) {
        Some(i) => Ok(Some(i)),
        None => {
            out.push_str(&format!(
                "planrun: {flag} {id}: the plan has no such task\n"
            ));
            Err(USAGE)
        }
    }
}

fn policy(auto: Option<&str>, run_expected_failures: bool, out: &mut String) -> Result<Policy, u8> {
    let classes: Vec<PolicyClass> = match auto {
        Some(text) if !text.is_empty() => Policy::parse_auto(text).map_err(|e| {
            out.push_str(&format!("planrun: {e}\n"));
            USAGE
        })?,
        _ => Vec::new(),
    };
    Policy::new(&classes, run_expected_failures).map_err(|e| {
        out.push_str(&format!("planrun: {e}\n"));
        USAGE
    })
}

fn session(ctx: &Context) -> SessionConfig {
    SessionConfig {
        root: ctx.ws.root().to_path_buf(),
        env: Vec::new(),
        timeout: None,
        temp: std::env::temp_dir(),
    }
}

fn open_journal(ctx: &Context, out: &mut String) -> Result<Journal, u8> {
    let mut journal = Journal::open(&ctx.journal).map_err(|e| {
        out.push_str(&format!("planrun: {}: {e}\n", ctx.journal.display()));
        USAGE
    })?;
    #[cfg(feature = "e2e")]
    if let Some(observer) = crate::e2e::observer() {
        journal.observe(observer);
    }
    #[cfg(not(feature = "e2e"))]
    let _ = &mut journal;
    Ok(journal)
}

fn finished(
    name: &str,
    result: Result<Finished, planrun_exec::runner::RunError>,
    out: &mut String,
) -> u8 {
    match result {
        Ok(Finished {
            end: End::Completed,
            ..
        }) => {
            out.push_str(&format!("planrun: {name}: done\n"));
            OK
        }
        Ok(Finished { reason, .. }) => {
            out.push_str(&format!(
                "planrun: {name}: stopped: {}; `planrun resume {name}` goes on from here\n",
                reason.unwrap_or_default()
            ));
            PLAN_ERRORS
        }
        Err(e) => {
            out.push_str(&format!("planrun: {name}: {e}\n"));
            USAGE
        }
    }
}

/// `planrun run`: refuses a plan with an unfinished journal unless `--restart` keeps it aside.
pub fn run(plan_file: &Path, plan: &Plan, opts: &Options, out: &mut String) -> u8 {
    let name = plan_file.display().to_string();
    let ctx = match context(plan_file, out) {
        Ok(ctx) => ctx,
        Err(code) => return code,
    };
    let _lock = match take_lock(&ctx, out) {
        Ok(lock) => lock,
        Err(code) => return code,
    };
    let lines = match lines(&ctx, out) {
        Ok(lines) => lines,
        Err(code) => return code,
    };
    let unfinished = analyse(&lines).is_some_and(|a| a.resumable());
    if unfinished && !opts.restart {
        out.push_str(&format!(
            "planrun: {name} has an unfinished run: `planrun resume {name}` goes on with it, \
             `planrun run --restart {name}` gives it up (its journal is kept)\n"
        ));
        return USAGE;
    }
    if unfinished {
        let kept = ctx.journal.with_extension(format!("{}.jsonl", now_ms()));
        if let Err(e) = std::fs::rename(&ctx.journal, &kept) {
            out.push_str(&format!("planrun: {}: {e}\n", ctx.journal.display()));
            return USAGE;
        }
        println!(
            "planrun: the unfinished journal is kept as {}",
            kept.display()
        );
    }
    let (from, to) = match (
        task_index(plan, opts.from.as_deref(), "--from", out),
        task_index(plan, opts.to.as_deref(), "--to", out),
    ) {
        (Ok(from), Ok(to)) => (from, to),
        (Err(code), _) | (_, Err(code)) => return code,
    };
    let policy = match policy(opts.auto.as_deref(), opts.run_expected_failures, out) {
        Ok(policy) => policy,
        Err(code) => return code,
    };
    let mut journal = match open_journal(&ctx, out) {
        Ok(journal) => journal,
        Err(code) => return code,
    };
    install_interrupt_handler();
    let mut ui = LineUi::new(plan, ctx.ws.root().to_path_buf());
    let config = RunConfig {
        plan_path: ctx.plan_path.clone(),
        policy,
        from,
        to,
        session: session(&ctx),
    };
    let result = Runner::new(plan, &ctx.ws, &mut journal, &mut ui, config).run();
    finished(&name, result, out)
}

/// `planrun resume`: the journal's last chain, with its policy and range unless the flags say
/// otherwise.
pub fn resume(plan_file: &Path, plan: &Plan, opts: &Options, out: &mut String) -> u8 {
    let name = plan_file.display().to_string();
    let ctx = match context(plan_file, out) {
        Ok(ctx) => ctx,
        Err(code) => return code,
    };
    let _lock = match take_lock(&ctx, out) {
        Ok(lock) => lock,
        Err(code) => return code,
    };
    let lines = match lines(&ctx, out) {
        Ok(lines) => lines,
        Err(code) => return code,
    };
    let Some(analysis) = analyse(&lines).filter(Analysis::resumable) else {
        out.push_str(&format!(
            "planrun: {name}: there is no unfinished run to resume\n"
        ));
        return USAGE;
    };
    let journaled = analysis.run.auto.join(",");
    let auto = opts.auto.clone().unwrap_or(journaled);
    let expected = opts.run_expected_failures || analysis.run.run_expected_failures;
    let policy = match policy(Some(&auto), expected, out) {
        Ok(policy) => policy,
        Err(code) => return code,
    };
    let from = opts.from.clone().or_else(|| analysis.run.from.clone());
    let to = opts.to.clone().or_else(|| analysis.run.to.clone());
    let (from, to) = match (
        task_index(plan, from.as_deref(), "--from", out),
        task_index(plan, to.as_deref(), "--to", out),
    ) {
        (Ok(from), Ok(to)) => (from, to),
        (Err(code), _) | (_, Err(code)) => return code,
    };
    let mut journal = match open_journal(&ctx, out) {
        Ok(journal) => journal,
        Err(code) => return code,
    };
    install_interrupt_handler();
    let mut ui = LineUi::new(plan, ctx.ws.root().to_path_buf());
    let config = RunConfig {
        plan_path: ctx.plan_path.clone(),
        policy,
        from,
        to,
        session: session(&ctx),
    };
    let result = Runner::new(plan, &ctx.ws, &mut journal, &mut ui, config).resume(&analysis);
    finished(&name, result, out)
}

/// `planrun status`: what the journal says, read-only; takes no lock.
pub fn status(plan_file: &Path, plan: &Plan, out: &mut String) -> u8 {
    let name = plan_file.display().to_string();
    let ctx = match context(plan_file, out) {
        Ok(ctx) => ctx,
        Err(code) => return code,
    };
    let lines = match lines(&ctx, out) {
        Ok(lines) => lines,
        Err(code) => return code,
    };
    out.push_str(&format!(
        "planrun · {name} · journal {}\n",
        ctx.journal.display()
    ));
    let Some(a) = analyse(&lines) else {
        out.push_str("no run yet\n");
        return OK;
    };
    let state = match a.end {
        Some(End::Completed) => "completed",
        Some(End::Stopped) => "stopped (resume goes on)",
        None => "interrupted (resume goes on)",
    };
    out.push_str(&format!("last run: {state}\n"));
    if a.run.fingerprint != *plan.source.fingerprint {
        out.push_str("the plan changed since that run: resume will refuse it\n");
    }
    let head = ctx.ws.git().head().ok();
    if head.as_ref() != Some(&a.head) {
        out.push_str("HEAD is not where the journal left it\n");
    }
    for task in &plan.tasks {
        let id = task.id.raw.to_string();
        let mark = if a.done.contains(&id) {
            "[✓]"
        } else if a.interrupted.as_deref() == Some(id.as_str()) {
            "[>]"
        } else {
            "[ ]"
        };
        out.push_str(&format!("  {mark} {id}  {}\n", plan.text(&task.title)));
    }
    if let Some((at, _)) = a.running() {
        out.push_str(&format!(
            "was running: step {}, action {} of {}\n",
            at.step + 1,
            at.action + 1,
            at.task
        ));
    }
    OK
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
}
