//! The resume preview (decisions D77, D85): what [`Runner::resume`](super::Runner::resume)
//! will find, read without the run lock and without writing anything, for a UI to show before
//! the user resumes. It decides nothing: resume takes the lock, checks everything again against
//! the disk as it is then, and asks its own questions. Where resume judges, the preview calls
//! the same function: [`head_verdict`], `again`, `resumed_for` and
//! [`preflight_resumed`](crate::preflight::preflight_resumed).

use planrun_model::{ActionKind, Plan, RepoPath};

use super::resume::{Again, Analysis, HeadVerdict, Past, again, head_verdict, resumed_for};
use crate::DoneState;
use crate::done::{StepOnDisk, restorable, step_state, temps};
use crate::git::Head;
use crate::journal::{self, At};
use crate::lock;
use crate::policy::DefaultClassifier;
use crate::preflight::preflight_resumed;
use crate::workspace::{Workspace, WorkspaceError};

/// What resume would find now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    /// A run holds the repository's lock now (D90): resume would be refused.
    pub live: bool,
    /// The plan's fingerprint is not the journal's: resume would stop.
    pub plan_changed: bool,
    /// `.git/index.lock` exists: resume would stop.
    pub index_lock: bool,
    /// HEAD now.
    pub now: Head,
    /// What resume makes of it (D77).
    pub head: HeadVerdict,
    /// The interrupted task's prepared steps, in the journal's order, as they are on disk.
    pub steps: Vec<StepPreview>,
    /// The scripts and commits that will ask "run again or keep" (D77).
    pub again: Vec<AgainPreview>,
    /// The paths resume will ask about as `resume-dirt` (D77). An Unknown step counts as done
    /// here: its paths are never asked about, since `[r]` puts them back and `[a]` verifies
    /// them.
    pub resume_dirt: Vec<RepoPath>,
}

/// One prepared step as it is now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepPreview {
    /// Where.
    pub at: At,
    /// Each path, and what that makes the step: Done is not written again, NotDone is, and
    /// Unknown is asked about.
    pub disk: StepOnDisk,
    /// For an Unknown step, whether `[r]` could restore it (D57); false otherwise.
    pub restorable: bool,
    /// The writer's temp files of the step that exist; resume removes them.
    pub temps: Vec<String>,
}

/// A script or commit that will ask "run again or keep".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgainPreview {
    /// Where.
    pub at: At,
    /// It was running when planrun was interrupted, not ended.
    pub running: bool,
}

/// Reads what resuming `analysis` on `plan` in `ws` would find: git and the disk are read,
/// nothing is written, and no lock is taken.
pub fn preview(
    ws: &Workspace,
    plan: &Plan,
    analysis: &Analysis,
) -> Result<Preview, WorkspaceError> {
    let git = ws.git();
    let git_dir = git.git_dir()?;
    let now = git.head()?;
    let head = head_verdict(ws, analysis, &now)?;
    let mut steps = Vec::new();
    for prepared in &analysis.prepared {
        let disk = step_state(ws, &prepared.step)?;
        let restorable = disk.state == DoneState::Unknown && restorable(ws, &prepared.step)?;
        steps.push(StepPreview {
            at: prepared.at.clone(),
            disk,
            restorable,
            temps: temps(ws, &prepared.step),
        });
    }
    let task = analysis
        .interrupted
        .as_deref()
        .and_then(|id| plan.tasks.iter().find(|t| &*t.id.raw == id));
    let mut again_list = Vec::new();
    let mut resume_dirt = Vec::new();
    if let Some(task) = task {
        again_list = asks_again(plan, task, analysis, &head);
        let done: Vec<bool> = steps
            .iter()
            .map(|s| s.disk.state != DoneState::NotDone)
            .collect();
        let resumed = resumed_for(task, &analysis.prepared, &done);
        resume_dirt = preflight_resumed(ws, plan, task, &resumed)?
            .into_iter()
            .filter(|f| f.code == "resume-dirt")
            .filter_map(|f| f.path)
            .collect();
    }
    let lock_dir = journal::dir(&git_dir);
    let live = lock::live(&lock_dir).map_err(|source| WorkspaceError::Io {
        path: lock_dir.join("lock.live").display().to_string(),
        source,
    })?;
    Ok(Preview {
        live,
        plan_changed: analysis.run.fingerprint != *plan.source.fingerprint,
        index_lock: git_dir.join("index.lock").exists(),
        now,
        head,
        steps,
        again: again_list,
        resume_dirt,
    })
}

/// The scripts and commits of the interrupted task that will ask "run again or keep". An action
/// that was running is left to the HEAD verdict when HEAD moved while it ran, and so is a
/// Commit that was running (resume runs it again or finds it made).
fn asks_again(
    plan: &Plan,
    task: &planrun_model::Task,
    analysis: &Analysis,
    head: &HeadVerdict,
) -> Vec<AgainPreview> {
    let mut found = Vec::new();
    for (at, past) in &analysis.past {
        let Some(action) = task
            .steps
            .get(at.step)
            .and_then(|s| s.actions.get(at.action))
        else {
            continue;
        };
        let (script, commit) = match &action.kind {
            ActionKind::RunScript(s) => (s, false),
            ActionKind::Commit(s) => (s, true),
            _ => continue,
        };
        let moved_it = matches!(head, HeadVerdict::MovedByRunning { at: a, .. } if a == at);
        if matches!(past, Past::Running { .. }) && (commit || moved_it) {
            continue;
        }
        let body = plan.source.resolve(&script.body);
        let class = DefaultClassifier.explain(script.shell, body).class;
        if let Again::Asks { running } = again(past, commit, class) {
            found.push(AgainPreview {
                at: at.clone(),
                running,
            });
        }
    }
    found
}
