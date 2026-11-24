//! Static checks over a recognized plan (docs/architecture.md §5.4). Pure: plan text only.
//!
//! Each rule is a [`Rule`]; [`default_rules`] is the set `planrun check` runs. A rule reports
//! with stable codes; the invalid corpus pins them.

use planrun_model::Plan;

use crate::diagnostic::Diagnostic;

/// One static check.
pub trait Rule: Send + Sync {
    /// Its id (`manifest`, `blob-chain`).
    fn id(&self) -> &'static str;

    /// Appends its findings for `plan`.
    fn check(&self, plan: &Plan, out: &mut Vec<Diagnostic>);
}

/// The rules of `planrun check`, in reporting order. Phase 1 fills this list:
///
/// - `task-order`: task ids unique, ordinals strictly increasing.
/// - `step-order`: written step numbers count up from 1 in each task.
/// - `manifest`: every file action's path is declared (except in formats without manifests);
///   every declared Create/Modify/Rewrite/Delete is touched by an action or a `Generated`;
///   a `WriteFile { FromManifest }` resolves to Create or Overwrite.
/// - `patch-shape`: a patch names exactly its target path, has at least one hunk, its hunks'
///   line counts match their bodies, and its `index` line (when present) parses.
/// - `blob-chain`: per path, each patch's preimage equals the postimage of the last earlier
///   patch on that path (when both are known); a Create's later patch starts from the created
///   content's blob id; a Delete ends the chain.
/// - `commit`: at most one Commit per task, and it is the task's last runnable action.
/// - `coverage` (warning): the share of a task's blocks that became Note or Manual(Unrecognized).
pub fn default_rules() -> Vec<Box<dyn Rule>> {
    Vec::new()
}
