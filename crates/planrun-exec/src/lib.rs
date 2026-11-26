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
//! - [`transaction`]: a step's file actions as prepare, then commit, with rollback (D53); the
//!   disk is checked again between the two (D78).
//! - [`done`]: a journaled step's state on the disk, for resume (D14, D77).
//! - [`preflight`]: task preflight and the dry run over one virtual tree (D50, D55).
//! - [`finding`]: what preflight, a simulation and the dry run report.
//! - [`shell`]: persistent pwsh and bash sessions behind [`ShellSession`] (D60–D65).
//! - [`expect`]: a run against its expectation: exit codes, summary lines with `…`, what asks
//!   for confirmation (D11, D66).
//! - [`policy`]: command classes and the [`CommandClassifier`]: a built-in allowlist over a
//!   splitter per shell (D13, D68).
//! - [`journal`]: one JSONL file per plan under `<git-dir>/planrun/`, the write-ahead recorder
//!   (D18, D76).
//! - [`lock`]: one planrun per repository (D77).
//! - [`runner`]: the state machine Task → Step → Action over the journal and the sessions,
//!   talking to a [`Ui`] (D73–D81).
//! - [`ui`]: the [`Ui`] trait and its questions (D15, D74).

pub mod done;
pub mod eol;
pub mod expect;
pub mod finding;
pub mod git;
pub mod handler;
pub mod journal;
pub mod lock;
pub mod policy;
pub mod preflight;
pub mod runner;
pub mod shell;
pub mod transaction;
pub mod tree;
pub mod ui;
pub mod workspace;

pub use handler::{ActionHandler, HandlerSet};
pub use policy::{CommandClassifier, PolicyClass};
pub use shell::{ScriptRun, ShellSession};
pub use ui::Ui;

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "../tests/support/ir.rs"]
mod ir;

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
