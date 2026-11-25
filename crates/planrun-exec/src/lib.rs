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
