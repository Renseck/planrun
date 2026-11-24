//! Markdown plan → typed [`planrun_model::Plan`] (docs/architecture.md §4–§5).
//!
//! Pipeline: read and normalize ([`source`]) → block stream ([`document`]) → pick a format
//! ([`registry`]) → outline: tasks, manifests, steps ([`format`]) → recognizers turn each step's
//! blocks into actions ([`recognizer`]) → static checks ([`check`]).
//!
//! This crate never touches the repository and never runs anything (decision D2). Checks that
//! need the repository (preimage blob ids against the files on disk) live in planrun-exec.
//!
//! Extension points, in the order a new plan style should try them (docs/extending.md):
//! 1. a format spec in TOML that reuses the built-in recognizer kinds ([`spec`]);
//! 2. a new [`recognizer::RecognizerFactory`] for a new idiom;
//! 3. a hand-written [`format::Format`] when the outline is not heading-shaped.

pub mod check;
pub mod diagnostic;
pub mod document;
pub mod format;
pub mod outline;
pub mod recognizer;
pub mod registry;
pub mod source;
pub mod spec;

pub use diagnostic::{Diagnostic, Diagnostics, Severity};
pub use registry::Registry;

use planrun_model::Plan;

/// Parses and checks `bytes` as a plan named `name`, with the formats in `registry`.
///
/// `format` forces a format by id; otherwise the one with the highest detection score wins, and
/// a tie or a score below the registry's threshold is an error (never a guess).
///
/// Errors and warnings come back together; a plan is returned only when there is no error
/// (`Never execute a partially parsed plan`, decision D3).
pub fn parse_plan(
    registry: &Registry,
    name: &str,
    bytes: &[u8],
    format: Option<&str>,
) -> Result<(Plan, Diagnostics), Diagnostics> {
    let _ = (registry, name, bytes, format);
    Err(Diagnostics::single(Diagnostic::error(
        "not-implemented",
        None,
        "parse_plan is implemented in Phase 1 (docs/roadmap.md)",
    )))
}
