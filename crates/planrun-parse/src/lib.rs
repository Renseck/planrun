//! Markdown plan → typed [`planrun_model::Plan`] (docs/architecture.md §4–§5).
//!
//! Pipeline: read and normalize ([`source`]) → block stream ([`document`]) → pick a format
//! ([`registry`]) → outline: tasks, manifests, steps ([`format`], [`outline`]) → recognizers turn
//! each step's blocks into actions ([`recognize`], [`recognizer`], the kinds in `kinds`) →
//! static checks ([`check`]).
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
mod kinds;
pub mod outline;
pub mod patch;
pub mod recognize;
pub mod recognizer;
pub mod registry;
pub mod source;
pub mod spec;

pub use diagnostic::{Diagnostic, Diagnostics, Severity};
pub use recognize::TaskCoverage;
pub use registry::Registry;

use planrun_model::Plan;

use crate::document::Document;
use crate::format::{FormatSettings, Outline};

/// A plan that parsed and checked without errors, with its warnings and notes and its coverage
/// report.
#[derive(Debug, Clone)]
pub struct Parsed {
    /// The plan.
    pub plan: Plan,
    /// Warnings and notes, in source order.
    pub diagnostics: Diagnostics,
    /// Per task, where its blocks went.
    pub coverage: Vec<TaskCoverage>,
}

/// Parses and checks `bytes` as a plan named `name`, with the formats in `registry`.
///
/// `format` forces a format by id; otherwise the one with the highest detection score wins, and
/// a tie or a score below the registry's threshold is an error (never a guess).
///
/// A plan is returned only when there is no error (`Never execute a partially parsed plan`,
/// decision D3); otherwise every diagnostic comes back, in source order. Check rules run only
/// when the outline and recognition had no errors.
pub fn parse_plan(
    registry: &Registry,
    name: &str,
    bytes: &[u8],
    format: Option<&str>,
) -> Result<Parsed, Diagnostics> {
    let source = source::load(name, bytes).map_err(Diagnostics::single)?;
    let doc = Document::build(source)?;
    let format = registry.select(&doc, format).map_err(Diagnostics::single)?;
    let outline = format.outline(&doc)?;
    let (tasks, coverage) = recognize::recognize(format.as_ref(), &doc, &outline)?;
    let mut found = outside_notes(&doc, format.settings(), &outline);
    let plan = Plan {
        source: doc.source,
        format: format.id().clone(),
        meta: outline.meta,
        tasks,
    };
    for rule in check::default_rules() {
        rule.check(&plan, &mut found);
    }
    let mut diagnostics = Diagnostics(found);
    diagnostics.sort();
    if diagnostics.has_errors() {
        return Err(diagnostics);
    }
    Ok(Parsed {
        plan,
        diagnostics,
        coverage,
    })
}

/// Manual markers outside the tasks: before task 1 (read them first) and after the tasks (not
/// run).
fn outside_notes(doc: &Document, settings: &FormatSettings, outline: &Outline) -> Vec<Diagnostic> {
    let before = outline.meta.preamble.iter().map(|text| {
        Diagnostic::note(
            "manual-outside-tasks",
            Some(text.span()),
            "read this before task 1: it is an instruction for you",
        )
    });
    let after = outline
        .after_tasks
        .map(|range| outline::marked(doc, settings, range))
        .unwrap_or_default();
    let after = after.into_iter().map(|text| {
        Diagnostic::note(
            "manual-outside-tasks",
            Some(text.span()),
            "after the tasks: planrun does not run this",
        )
    });
    before.chain(after).collect()
}
