//! The typed plan IR (docs/architecture.md §3).
//!
//! Parsing produces a [`Plan`]; execution consumes it. Neither side depends on the other: this
//! crate is the contract between them. Text is never copied out of the plan: every piece of text
//! is a [`TextRef`] into the normalized plan source held by [`PlanSource`].
//!
//! The action set ([`ActionKind`]) is closed on purpose (decision D4): plan formats vary,
//! actions don't. A new format maps its own wording onto these actions.

#![forbid(unsafe_code)]

mod action;
mod expect;
mod ids;
mod path;
mod pattern;
mod plan;
mod source;
mod span;

pub use action::{Action, ActionKind, ManualReason, Script, ScriptIntent, Shell, WriteMode};
pub use expect::{ExitExpect, Expectation};
pub use ids::{AbbrevBlobId, BlobIdError, FormatId, RecognizerId, TaskId};
pub use path::{PathOrigin, RepoPath, RepoPathError};
pub use pattern::{ManifestPath, PathPattern, PatternError};
pub use plan::{ChangeKind, LineRange, Manifest, ManifestEntry, Plan, PlanMeta, Step, Task};
pub use source::PlanSource;
pub use span::{LineCol, Span, TextRef};
