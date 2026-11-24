//! Static checks over a recognized plan (docs/architecture.md §4.5). Pure: plan text only.
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
    Vec::new()
}
