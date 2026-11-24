//! Static checks over a recognized plan (docs/architecture.md §4.5). Pure: plan text only.
//!
//! Each rule is a [`Rule`]; [`default_rules`] is the set `planrun check` runs. A rule reports
//! with stable codes; the invalid corpus pins them.

use std::collections::HashMap;

use planrun_model::{
    AbbrevBlobId, Action, ActionKind, ChangeKind, Plan, RepoPath, Span, Task, WriteMode,
};

use crate::diagnostic::Diagnostic;
use crate::patch;

/// One static check.
pub trait Rule: Send + Sync {
    /// Its id (`manifest`, `blob-chain`).
    fn id(&self) -> &'static str;

    /// Appends its findings for `plan`.
    fn check(&self, plan: &Plan, out: &mut Vec<Diagnostic>);
}

/// The rules of `planrun check`, in reporting order:
///
/// - `task-order`: a repeated task id is `duplicate-task-id`; among the other tasks, ordinals
///   strictly increase (`task-order`).
/// - `step-order`: written step numbers count up from 1 in each task.
/// - `manifest`: every file action's path is declared (`manifest-undeclared`); every declared
///   Create/Modify/Rewrite/Delete is touched by an action or a `Generated`
///   (`manifest-untouched`); a `WriteFile { FromManifest }` on a declared path resolves to Create
///   or Overwrite (`manifest-write-mode`). A task without a manifest block is skipped.
/// - `patch-shape`: a patch has at least one hunk and its hunks' line counts match their bodies.
///   (Its paths and `index` line are checked when it is recognized: `patch-path`, `patch-shape`.)
/// - `blob-chain`: per path, each patch's preimage equals the postimage of the last earlier
///   patch on that path (when both are known); a Create's later patch starts from the created
///   content's blob id; a Delete ends the chain.
/// - `commit`: at most one Commit per task (`commit-twice`), and it is the task's last runnable
///   action (`commit-not-last`).
///
/// Rules run only on a plan whose outline and recognition had no errors. The coverage report is
/// not a rule: `parse_plan` returns it beside the plan.
pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(TaskOrder),
        Box::new(StepOrder),
        Box::new(ManifestRule),
        Box::new(PatchShape),
        Box::new(BlobChain),
        Box::new(CommitRule),
    ]
}

fn actions(task: &Task) -> impl Iterator<Item = &Action> {
    task.steps.iter().flat_map(|s| s.actions.iter())
}

/// The task's heading line.
fn heading(task: &Task) -> Span {
    Span::new(task.span.start, task.title.span().end.max(task.span.start))
}

/// The path a file action changes.
fn file_path(kind: &ActionKind) -> Option<&RepoPath> {
    match kind {
        ActionKind::WriteFile { path, .. }
        | ActionKind::ApplyPatch { path, .. }
        | ActionKind::ReplaceExact { path, .. }
        | ActionKind::Delete { path, .. }
        | ActionKind::Generated { path } => Some(path),
        _ => None,
    }
}

/// `task-order`.
pub struct TaskOrder;

impl Rule for TaskOrder {
    fn id(&self) -> &'static str {
        "task-order"
    }

    fn check(&self, plan: &Plan, out: &mut Vec<Diagnostic>) {
        let mut seen: HashMap<&str, Span> = HashMap::new();
        let mut last: Option<&Task> = None;
        for task in &plan.tasks {
            if let Some(first) = seen.get(&*task.id.raw) {
                out.push(
                    Diagnostic::error(
                        "duplicate-task-id",
                        Some(heading(task)),
                        format!("task `{}` is defined twice", task.id),
                    )
                    .with_related(*first, "first defined here"),
                );
                continue;
            }
            seen.insert(&task.id.raw, heading(task));
            if let Some(prev) = last.filter(|p| task.id.ordinal <= p.id.ordinal) {
                out.push(
                    Diagnostic::error(
                        "task-order",
                        Some(heading(task)),
                        format!("task `{}` comes after task `{}`", task.id, prev.id),
                    )
                    .with_related(heading(prev), "the earlier task"),
                );
            }
            last = Some(task);
        }
    }
}

/// `step-order`.
pub struct StepOrder;

impl Rule for StepOrder {
    fn id(&self) -> &'static str {
        "step-order"
    }

    fn check(&self, plan: &Plan, out: &mut Vec<Diagnostic>) {
        for task in &plan.tasks {
            let numbered = task.steps.iter().filter_map(|s| s.number.map(|n| (n, s)));
            let wrong = numbered
                .zip(1u32..)
                .find(|((n, _), expected)| n != expected);
            let Some(((n, step), expected)) = wrong else {
                continue;
            };
            let at = Span::new(
                step.span.start,
                step.title.as_ref().map_or(step.span.end, |t| t.span().end),
            );
            out.push(Diagnostic::error(
                "step-order",
                Some(at),
                format!(
                    "step {n} where step {expected} comes next in task `{}`",
                    task.id
                ),
            ));
        }
    }
}

/// `manifest`.
pub struct ManifestRule;

impl Rule for ManifestRule {
    fn id(&self) -> &'static str {
        "manifest"
    }

    fn check(&self, plan: &Plan, out: &mut Vec<Diagnostic>) {
        for task in plan.tasks.iter().filter(|t| t.manifest.span.is_some()) {
            check_manifest(task, out);
        }
    }
}

fn check_manifest(task: &Task, out: &mut Vec<Diagnostic>) {
    let manifest = task.manifest.span.unwrap_or(task.span);
    let declared = |path: &RepoPath| task.manifest.entries.iter().find(|e| &e.path == path);
    let mut touched: Vec<&RepoPath> = Vec::new();
    for action in actions(task) {
        let Some(path) = file_path(&action.kind) else {
            continue;
        };
        if touched.contains(&path) {
            continue;
        }
        touched.push(path);
        let Some(entry) = declared(path) else {
            out.push(
                Diagnostic::error(
                    "manifest-undeclared",
                    Some(action.span),
                    format!("`{path}` is changed here but not declared in the task's files"),
                )
                .with_related(manifest, "the task's files"),
            );
            continue;
        };
        let unresolved = matches!(
            action.kind,
            ActionKind::WriteFile {
                mode: WriteMode::FromManifest,
                ..
            }
        );
        if unresolved {
            out.push(
                Diagnostic::error(
                    "manifest-write-mode",
                    Some(action.span),
                    format!(
                        "full contents for `{path}`, declared {:?}: neither create nor overwrite",
                        entry.change
                    ),
                )
                .with_related(entry.span, "declared here"),
            );
        }
    }
    let must_touch = |c: ChangeKind| !matches!(c, ChangeKind::Test);
    for entry in &task.manifest.entries {
        if must_touch(entry.change) && !touched.contains(&&entry.path) {
            out.push(Diagnostic::error(
                "manifest-untouched",
                Some(entry.span),
                format!(
                    "`{}` is declared, and no action of the task changes it",
                    entry.path
                ),
            ));
        }
    }
}

/// `patch-shape`.
pub struct PatchShape;

impl Rule for PatchShape {
    fn id(&self) -> &'static str {
        "patch-shape"
    }

    fn check(&self, plan: &Plan, out: &mut Vec<Diagnostic>) {
        for action in plan.tasks.iter().flat_map(actions) {
            let ActionKind::ApplyPatch { patch, .. } = &action.kind else {
                continue;
            };
            if let Err(e) = patch::check_hunks(plan.text(patch)) {
                out.push(Diagnostic::error("patch-shape", Some(action.span), e));
            }
        }
    }
}

/// What is known about a file's content at a point of the plan.
#[derive(Debug, Clone, Copy)]
enum Known {
    /// The blob id of content the plan wrote in full.
    Full([u8; 20]),
    /// The postimage of a patch.
    Abbrev(AbbrevBlobId),
}

impl Known {
    fn agrees(&self, pre: &AbbrevBlobId) -> bool {
        match self {
            Known::Full(id) => pre.matches(id),
            Known::Abbrev(post) => {
                let (a, b) = (post.to_string(), pre.to_string());
                a.starts_with(&b) || b.starts_with(&a)
            }
        }
    }

    fn hex(&self) -> String {
        match self {
            Known::Full(id) => id.iter().map(|b| format!("{b:02x}")).collect(),
            Known::Abbrev(id) => id.to_string(),
        }
    }
}

/// The git blob id of `content`.
pub fn blob_id(content: &str) -> [u8; 20] {
    let mut sha = sha1_smol::Sha1::new();
    sha.update(format!("blob {}\0", content.len()).as_bytes());
    sha.update(content.as_bytes());
    sha.digest().bytes()
}

/// `blob-chain`.
pub struct BlobChain;

impl Rule for BlobChain {
    fn id(&self) -> &'static str {
        "blob-chain"
    }

    fn check(&self, plan: &Plan, out: &mut Vec<Diagnostic>) {
        let mut known: HashMap<&RepoPath, (Known, Span)> = HashMap::new();
        for action in plan.tasks.iter().flat_map(actions) {
            match &action.kind {
                ActionKind::WriteFile { path, body, .. } => {
                    known.insert(path, (Known::Full(blob_id(plan.text(body))), action.span));
                }
                ActionKind::ApplyPatch {
                    path, pre, post, ..
                } => {
                    let broken = pre
                        .zip(known.get(path))
                        .filter(|(pre, (s, _))| !s.agrees(pre));
                    if let Some((pre, (state, before))) = broken {
                        out.push(chain_error(path, &pre, state, action.span, *before));
                    }
                    match post {
                        Some(post) => known.insert(path, (Known::Abbrev(*post), action.span)),
                        None => known.remove(path),
                    };
                }
                ActionKind::ReplaceExact { path, .. }
                | ActionKind::Delete { path, .. }
                | ActionKind::Generated { path } => {
                    known.remove(path);
                }
                _ => {}
            }
        }
    }
}

fn chain_error(
    path: &RepoPath,
    pre: &AbbrevBlobId,
    state: &Known,
    at: Span,
    before: Span,
) -> Diagnostic {
    Diagnostic::error(
        "blob-chain",
        Some(at),
        format!(
            "this patch of `{path}` starts from {pre}, but the plan's last change to it leaves {}",
            state.hex()
        ),
    )
    .with_related(before, "the last change")
}

/// `commit`.
pub struct CommitRule;

impl Rule for CommitRule {
    fn id(&self) -> &'static str {
        "commit"
    }

    fn check(&self, plan: &Plan, out: &mut Vec<Diagnostic>) {
        for task in &plan.tasks {
            check_commit(task, out);
        }
    }
}

fn check_commit(task: &Task, out: &mut Vec<Diagnostic>) {
    let mut commit: Option<Span> = None;
    for action in actions(task) {
        let is_commit = matches!(action.kind, ActionKind::Commit(_));
        let runnable = !matches!(
            action.kind,
            ActionKind::Note { .. } | ActionKind::Manual { .. }
        );
        match commit {
            Some(first) if is_commit => out.push(
                Diagnostic::error(
                    "commit-twice",
                    Some(action.span),
                    format!("a second commit in task `{}`", task.id),
                )
                .with_related(first, "the first commit"),
            ),
            Some(first) if runnable => out.push(
                Diagnostic::error(
                    "commit-not-last",
                    Some(action.span),
                    format!("this runs after the commit of task `{}`", task.id),
                )
                .with_related(first, "the commit"),
            ),
            None if is_commit => commit = Some(action.span),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_ids_are_gits() {
        let hex: String = blob_id("").iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
        let hello: String = blob_id("hello\n")
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(hello, "ce013625030ba8dba906f756967f9e9ca394464a");
    }

    #[test]
    fn abbreviations_agree_by_prefix() {
        let id = |s: &str| AbbrevBlobId::parse(s).unwrap_or_else(|e| panic!("{e}"));
        assert!(Known::Abbrev(id("97bd92a")).agrees(&id("97bd92a0")));
        assert!(!Known::Abbrev(id("97bd92a")).agrees(&id("97bd92b")));
        assert!(Known::Full(blob_id("hello\n")).agrees(&id("ce01362")));
    }
}
