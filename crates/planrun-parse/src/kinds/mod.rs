//! The built-in recognizer kinds (docs/extending.md §3).

mod before_after;
mod ceremony;
mod delete;
mod expected;
mod file_block;
mod generated;
mod manual_prose;
mod params;
mod run_line;
mod script;

use std::sync::Arc;

use planrun_model::{
    Action, ActionKind, ChangeKind, ManifestEntry, PathOrigin, RepoPath, Span, TextRef,
};
use regex::Captures;

use crate::diagnostic::Diagnostic;
use crate::document::{Block, Located};
use crate::recognizer::{RecognizerFactory, StepContext};

pub use before_after::BeforeAfterFactory;
pub use ceremony::CeremonyFactory;
pub use delete::DeleteFactory;
pub use expected::{ExpectedBlockFactory, ExpectedProseFactory};
pub use file_block::FileBlockFactory;
pub use generated::GeneratedFactory;
pub use manual_prose::ManualProseFactory;
pub use run_line::RunLineFactory;
pub use script::ScriptFactory;

/// Every built-in factory, for [`crate::Registry::with_builtins`].
pub fn builtin_factories() -> Vec<Arc<dyn RecognizerFactory>> {
    vec![
        Arc::new(FileBlockFactory),
        Arc::new(CeremonyFactory),
        Arc::new(DeleteFactory),
        Arc::new(GeneratedFactory),
        Arc::new(ScriptFactory),
        Arc::new(ExpectedBlockFactory),
        Arc::new(ExpectedProseFactory),
        Arc::new(RunLineFactory),
        Arc::new(BeforeAfterFactory),
        Arc::new(ManualProseFactory),
    ]
}

/// The text of a paragraph block, if it is one.
fn paragraph<'a>(cx: &'a StepContext<'_>, block: &'a Located) -> Option<(&'a str, u32)> {
    let Block::Paragraph { text } = &block.block else {
        return None;
    };
    Some((cx.doc.text(text), text.span().start))
}

/// The span of capture `name` in text that starts at `base`.
fn capture_span(caps: &Captures<'_>, name: &str, base: u32) -> Option<Span> {
    let m = caps.name(name)?;
    Some(Span::new(base + offset(m.start()), base + offset(m.end())))
}

/// The capture `name` as a span reference.
fn capture_ref(caps: &Captures<'_>, name: &str, base: u32) -> Option<TextRef> {
    capture_span(caps, name, base).map(TextRef::Span)
}

/// The `path` capture as a repository path, or a `bad-path` error at the capture.
fn lead_path(caps: &Captures<'_>, base: u32, lead: Span) -> Result<RepoPath, Diagnostic> {
    let span = capture_span(caps, "path", base).unwrap_or(lead);
    let raw = caps.name("path").map_or("", |m| m.as_str());
    RepoPath::new(raw).map_err(|e| Diagnostic::error("bad-path", Some(span), e.to_string()))
}

/// The `path` capture as a repository path, resolved against the task's manifest when
/// `by_file_name` (decision D32): a path without `/` that is no entry itself becomes the one
/// entry with that file name, when there is exactly one and it was written in full.
fn resolved_lead_path(
    cx: &StepContext<'_>,
    caps: &Captures<'_>,
    base: u32,
    lead: Span,
    by_file_name: bool,
) -> Result<(RepoPath, PathOrigin), Diagnostic> {
    let path = lead_path(caps, base, lead)?;
    let entries = &cx.manifest.entries;
    let unresolved =
        !by_file_name || path.as_str().contains('/') || entries.iter().any(|e| e.path == path);
    if unresolved {
        return Ok((path, PathOrigin::Written));
    }
    let same: Vec<&ManifestEntry> = entries
        .iter()
        .filter(|e| e.path.file_name() == path.as_str())
        .collect();
    let first = same.first().map(|e| &e.path);
    let one = first.filter(|p| same.iter().all(|e| &e.path == *p && e.origin.is_written()));
    match one {
        Some(entry) => Ok((entry.clone(), PathOrigin::FileName)),
        None => Ok((path, PathOrigin::Written)),
    }
}

/// The task's one manifest entry an edit with no path can mean (decision D32): not Test,
/// Generated or Delete. `None` with none or several.
fn only_entry(cx: &StepContext<'_>) -> Option<RepoPath> {
    let mut editable = cx.manifest.entries.iter().filter(|e| {
        !matches!(
            e.change,
            ChangeKind::Test | ChangeKind::Generated | ChangeKind::Delete
        )
    });
    let first = editable.next()?;
    editable
        .all(|e| e.path == first.path)
        .then(|| first.path.clone())
}

/// The script of the action an expectation amends: the last action, when it is a script.
fn amendable(cx: &StepContext<'_>, lead: Span) -> Result<Action, Diagnostic> {
    let script = cx
        .previous
        .last()
        .filter(|a| matches!(a.kind, ActionKind::RunScript(_) | ActionKind::Commit(_)));
    let Some(action) = script else {
        return Err(Diagnostic::error(
            "expected-without-script",
            Some(lead),
            "an expectation with no script directly before it",
        ));
    };
    let already = match &action.kind {
        ActionKind::RunScript(s) | ActionKind::Commit(s) => s.expect.is_some(),
        _ => false,
    };
    if already {
        return Err(Diagnostic::error(
            "expected-twice",
            Some(lead),
            "the script before this already has an expectation",
        )
        .with_related(action.span, "the script"));
    }
    Ok(action.clone())
}

fn offset(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
