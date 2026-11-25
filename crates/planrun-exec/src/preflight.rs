//! Task preflight and the dry run (docs/architecture.md §6.1, §7; decisions D50, D55).
//!
//! Both walk a task over a virtual tree: the manifest checks first (file entries, not Bare, not
//! patterns), then the actions in order. The dry run carries one tree across every task and
//! simulates every file action; a real run's preflight (P5) uses a fresh tree and stops at the
//! task's first script or Manual, because later file actions may depend on it.

use std::collections::HashSet;

use planrun_model::{
    Action, ActionKind, ChangeKind, ManifestEntry, ManifestPath, PathOrigin, Plan, RepoPath, Task,
    WriteMode,
};

use crate::finding::{Finding, Severity};
use crate::handler::{HandlerSet, file_path};
use crate::tree::VirtualTree;
use crate::workspace::{Workspace, WorkspaceError};

/// What became of one action in a dry run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// A file action that would run on its own.
    Ok(String),
    /// A file action that would run after you confirm it: unscoped (D50).
    WouldAsk(String),
    /// A file action that would fail.
    Failed(Finding),
    /// A file action whose result depends on a script or Manual step that did not run (D55).
    Depends(Finding),
    /// A script or a commit: never run in a dry run.
    NotRun,
    /// A `Generated` check: its script did not run.
    NotChecked,
    /// A Manual: planrun would wait for you.
    Waits,
    /// A Note: shown in place.
    Shown,
}

/// One action of the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionReport {
    /// The step's position in the task, from 0.
    pub step: usize,
    /// The action.
    pub action: Action,
    /// What became of it.
    pub outcome: Outcome,
}

/// One task of the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskReport {
    /// The task's position in the plan, from 0.
    pub task: usize,
    /// What the manifest checks found.
    pub manifest: Vec<Finding>,
    /// The actions in order.
    pub actions: Vec<ActionReport>,
}

/// The dry run of a plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRun {
    /// What the start found: dirt in the paths the plan touches (errors) or elsewhere (a
    /// warning).
    pub start: Vec<Finding>,
    /// Per task.
    pub tasks: Vec<TaskReport>,
    /// Paths a file action writes whose attributes the dry run reads from the disk as they are
    /// (a `.gitattributes` the plan writes, D54).
    pub attribute_writes: Vec<RepoPath>,
}

impl DryRun {
    /// Every finding: the start, the manifests, the actions.
    pub fn findings(&self) -> impl Iterator<Item = &Finding> {
        let actions = self.tasks.iter().flat_map(|t| {
            t.manifest
                .iter()
                .chain(t.actions.iter().filter_map(|a| match &a.outcome {
                    Outcome::Failed(f) | Outcome::Depends(f) => Some(f),
                    _ => None,
                }))
        });
        self.start.iter().chain(actions)
    }

    /// True when anything would fail.
    pub fn has_errors(&self) -> bool {
        self.findings().any(Finding::is_error)
    }
}

/// Walks tasks over one tree: whether a script or Manual has gone by without running.
struct Walk<'t, 'w> {
    tree: &'t mut VirtualTree<'w>,
    handlers: HandlerSet,
    unrun: bool,
}

/// The dry run of `plan` (D55): parse and checks are done; this preflights every task against
/// one virtual tree, checks real dirt once for the paths of every task, and simulates every file
/// action. Writes nothing. An error here is a git or file-system failure, not a finding.
pub fn dry_run(ws: &Workspace, plan: &Plan) -> Result<DryRun, WorkspaceError> {
    let start = dirt(ws, plan.tasks.iter())?;
    let mut tree = VirtualTree::new(ws)?;
    let mut walk = Walk {
        tree: &mut tree,
        handlers: HandlerSet::files(),
        unrun: false,
    };
    let tasks = plan
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| walk.task(plan, i, task, false))
        .collect();
    let attribute_writes = plan
        .tasks
        .iter()
        .flat_map(|t| t.steps.iter().flat_map(|s| s.actions.iter()))
        .filter_map(|a| file_path(&a.kind))
        .filter(|p| p.file_name() == ".gitattributes")
        .cloned()
        .collect();
    Ok(DryRun {
        start,
        tasks,
        attribute_writes,
    })
}

/// A real run's preflight of `task` on the disk as it is now (P5 runs it before each task):
/// dirt in the task's paths, the manifest checks, and the file actions up to the first script
/// or Manual. Every error finding stops the task.
pub fn preflight(ws: &Workspace, plan: &Plan, task: &Task) -> Result<Vec<Finding>, WorkspaceError> {
    let mut found = dirt(ws, std::iter::once(task))?;
    let mut tree = VirtualTree::new(ws)?;
    let mut walk = Walk {
        tree: &mut tree,
        handlers: HandlerSet::files(),
        unrun: false,
    };
    let report = walk.task(plan, 0, task, true);
    found.extend(report.manifest);
    found.extend(report.actions.into_iter().filter_map(|a| match a.outcome {
        Outcome::Failed(f) | Outcome::Depends(f) => Some(f),
        _ => None,
    }));
    Ok(found)
}

impl Walk<'_, '_> {
    fn task(&mut self, plan: &Plan, index: usize, task: &Task, stop_at_unrun: bool) -> TaskReport {
        let manifest = task
            .manifest
            .entries
            .iter()
            .filter_map(|e| self.entry(e))
            .collect();
        let mut actions = Vec::new();
        for (step, s) in task.steps.iter().enumerate() {
            for action in &s.actions {
                if stop_at_unrun && self.unrun {
                    return TaskReport {
                        task: index,
                        manifest,
                        actions,
                    };
                }
                let outcome = self.action(plan, task, action);
                actions.push(ActionReport {
                    step,
                    action: action.clone(),
                    outcome,
                });
            }
        }
        TaskReport {
            task: index,
            manifest,
            actions,
        }
    }

    /// The manifest check of one entry: Create absent; Modify, Rewrite, Delete present. Test
    /// and Generated entries, patterns (D41) and Bare entries (D45) are not checked.
    fn entry(&mut self, entry: &ManifestEntry) -> Option<Finding> {
        let ManifestPath::File(path) = &entry.path else {
            return None;
        };
        if entry.origin == PathOrigin::Bare {
            return None;
        }
        let must_exist = match entry.change {
            ChangeKind::Create => false,
            ChangeKind::Modify | ChangeKind::Rewrite | ChangeKind::Delete => true,
            ChangeKind::Test | ChangeKind::Generated => return None,
        };
        let span = Some(entry.span);
        if self.tree.is_unknown(path) {
            return None;
        }
        let exists = match self.tree.state(path, span) {
            Ok(state) => state.exists(),
            Err(f) => return Some(f),
        };
        let finding = match (must_exist, exists) {
            (false, true) => Finding::error(
                "create-exists",
                span,
                Some(path),
                format!("`{path}` already exists; the task's files say it is created"),
            ),
            (true, false) => Finding::error(
                "target-missing",
                span,
                Some(path),
                format!("`{path}` does not exist; the task's files say it is changed"),
            ),
            _ => return None,
        };
        Some(self.after_unrun(finding))
    }

    fn action(&mut self, plan: &Plan, task: &Task, action: &Action) -> Outcome {
        match &action.kind {
            ActionKind::RunScript(_) | ActionKind::Commit(_) => {
                self.unrun = true;
                Outcome::NotRun
            }
            ActionKind::Manual { .. } => {
                self.unrun = true;
                Outcome::Waits
            }
            ActionKind::Note { .. } => Outcome::Shown,
            ActionKind::Generated { .. } => Outcome::NotChecked,
            _ => self.file_action(plan, task, action),
        }
    }

    fn file_action(&mut self, plan: &Plan, task: &Task, action: &Action) -> Outcome {
        let Some(path) = file_path(&action.kind) else {
            return Outcome::Shown;
        };
        let span = Some(action.span);
        if let Some(f) = ambiguous(task, action, path) {
            return Outcome::Failed(f);
        }
        if self.tree.is_unknown(path) {
            return Outcome::Depends(depends(path, action));
        }
        let Some(handler) = self.handlers.find(action) else {
            return Outcome::Failed(Finding::error("internal", span, Some(path), "no handler"));
        };
        match handler.simulate(action, &plan.source, self.tree) {
            Ok(done) if task.manifest.scopes(path) => Outcome::Ok(done.detail),
            Ok(done) => Outcome::WouldAsk(done.detail),
            Err(f) if f.severity == Severity::Warning => Outcome::Depends(f),
            Err(f) if self.unrun && depends_on_state(f.code) => {
                self.tree.mark_unknown(path);
                Outcome::Depends(self.after_unrun(f))
            }
            Err(f) => Outcome::Failed(with_bare_hint(task, path, f)),
        }
    }

    /// After a script or Manual that did not run, a failure of the tree's state is a warning.
    fn after_unrun(&self, f: Finding) -> Finding {
        if !self.unrun || !depends_on_state(f.code) {
            return f;
        }
        Finding {
            severity: Severity::Warning,
            message: format!(
                "{} (depends on a script or Manual step that did not run)",
                f.message
            ),
            ..f
        }
    }
}

/// D50: a Bare-targeted Overwrite or Delete has nothing that confirms its target.
fn ambiguous(task: &Task, action: &Action, path: &RepoPath) -> Option<Finding> {
    let unconfirmed = matches!(
        action.kind,
        ActionKind::WriteFile {
            mode: WriteMode::Overwrite,
            ..
        } | ActionKind::Delete { .. }
    );
    if !unconfirmed || !task.manifest.is_bare_target(path) {
        return None;
    }
    Some(Finding::error(
        "path-ambiguous",
        Some(action.span),
        Some(path),
        format!(
            "`{path}` is a bare file name in the task's files: its directory is unknown, and \
             nothing in this action confirms the target (D45, D50)"
        ),
    ))
}

fn with_bare_hint(task: &Task, path: &RepoPath, f: Finding) -> Finding {
    if f.code != "target-missing" || !task.manifest.is_bare_target(path) {
        return f;
    }
    Finding {
        message: format!(
            "{}; it is a bare file name in the task's files, so its directory is unknown (D45)",
            f.message
        ),
        ..f
    }
}

fn depends(path: &RepoPath, action: &Action) -> Finding {
    Finding::warning(
        "unverified",
        Some(action.span),
        Some(path),
        format!("`{path}` depends on a script or Manual step that did not run"),
    )
}

/// The codes that report the tree's state, which an earlier script or Manual may have changed.
fn depends_on_state(code: &str) -> bool {
    matches!(
        code,
        "create-exists"
            | "target-missing"
            | "blob-mismatch"
            | "post-mismatch"
            | "patch-does-not-apply"
            | "before-not-unique"
            | "eol-mixed"
    )
}

/// The paths a task touches: its file actions, `Generated` paths and manifest file entries.
fn touched(task: &Task) -> impl Iterator<Item = &RepoPath> {
    let actions = task
        .steps
        .iter()
        .flat_map(|s| s.actions.iter())
        .filter_map(|a| match &a.kind {
            ActionKind::Generated { path } => Some(path),
            kind => file_path(kind),
        });
    actions.chain(task.manifest.entries.iter().filter_map(|e| e.path.file()))
}

/// Real dirt (D55), from one `git status`: a touched path with uncommitted changes is
/// `path-dirty`; any other dirt is one `tree-dirty` warning.
fn dirt<'p>(
    ws: &Workspace,
    tasks: impl Iterator<Item = &'p Task>,
) -> Result<Vec<Finding>, WorkspaceError> {
    let fold = |p: &str| {
        if ws.ignorecase() {
            p.to_lowercase()
        } else {
            p.to_owned()
        }
    };
    let mine: HashSet<String> = tasks.flat_map(touched).map(|p| fold(p.as_str())).collect();
    let (touched_dirt, other_dirt): (Vec<_>, Vec<_>) = ws
        .git()
        .status(None)?
        .into_iter()
        .partition(|e| mine.contains(&fold(&e.path)));
    let mut found: Vec<Finding> = touched_dirt
        .into_iter()
        .map(|e| {
            let path = RepoPath::new(&e.path).ok();
            Finding::error(
                "path-dirty",
                None,
                path.as_ref(),
                format!(
                    "`{}` has uncommitted changes (`{}`); the plan changes it",
                    e.path, e.code
                ),
            )
        })
        .collect();
    let other = other_dirt.len();
    if other > 0 {
        found.push(Finding::warning(
            "tree-dirty",
            None,
            None,
            format!("{other} other path(s) have uncommitted changes; the plan does not touch them"),
        ));
    }
    Ok(found)
}
