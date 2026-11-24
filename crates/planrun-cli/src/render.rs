//! Diagnostics with source context (annotate-snippets, plain: no colour, stable for snapshots)
//! and the coverage report.

use annotate_snippets::{AnnotationKind, Group, Level, Renderer, Snippet};
use planrun_model::{ActionKind, Plan};
use planrun_parse::{Diagnostic, Severity, TaskCoverage};

/// One diagnostic against `text` (the normalized plan), with `name` as its path:
/// `error[code]: message`, then `--> name:line:col`, the source line and a caret.
pub fn diagnostic(name: &str, text: &str, d: &Diagnostic) -> String {
    let level = match d.severity {
        Severity::Error => Level::ERROR,
        Severity::Warning => Level::WARNING,
        Severity::Note => Level::NOTE,
    };
    let title = level.primary_title(d.message.as_str()).id(d.code);
    let renderer = Renderer::plain();
    let Some(span) = d.span else {
        return renderer.render(&[Group::with_title(title)]);
    };
    let mut snippet = Snippet::source(text)
        .path(name)
        .annotation(AnnotationKind::Primary.span(span.start as usize..span.end as usize));
    for (related, label) in &d.related {
        snippet = snippet.annotation(
            AnnotationKind::Context
                .span(related.start as usize..related.end as usize)
                .label(label.as_str()),
        );
    }
    renderer.render(&[title.element(snippet)])
}

/// Every diagnostic, one after another, each followed by a blank line.
pub fn diagnostics(name: &str, text: &str, all: &[Diagnostic], out: &mut String) {
    for d in all {
        out.push_str(&diagnostic(name, text, d));
        out.push_str("\n\n");
    }
}

/// The coverage table, the absorbed blocks and the Manual actions of `plan`.
pub fn coverage(plan: &Plan, coverage: &[TaskCoverage], out: &mut String) {
    out.push_str("coverage\n");
    out.push_str("  task          blocks  actions  absorbed  manifest  notes  manual\n");
    for c in coverage {
        out.push_str(&format!(
            "  {:<12}  {:>6}  {:>7}  {:>8}  {:>8}  {:>5}  {:>6}\n",
            c.task.raw,
            c.blocks(),
            c.actions,
            c.absorbed.len(),
            c.manifest,
            c.notes,
            c.manual
        ));
    }
    let line = |at: u32| plan.source.line_col(at).line;
    let absorbed: Vec<String> = coverage
        .iter()
        .flat_map(|c| c.absorbed.iter().map(move |a| (c, a)))
        .map(|(c, a)| {
            format!(
                "  {:<12}  L{:<5} {}\n",
                c.task.raw,
                line(a.span.start),
                a.recognizer.0
            )
        })
        .collect();
    if !absorbed.is_empty() {
        out.push_str("absorbed (never run)\n");
        absorbed.iter().for_each(|l| out.push_str(l));
    }
    let manual: Vec<String> = plan
        .tasks
        .iter()
        .flat_map(|t| {
            t.steps
                .iter()
                .flat_map(move |s| s.actions.iter().map(move |a| (t, a)))
        })
        .filter_map(|(t, a)| match &a.kind {
            ActionKind::Manual { reason, text } => Some(format!(
                "  {:<12}  L{:<5} {reason:?}: {}\n",
                t.id.raw,
                line(a.span.start),
                first_line(plan.text(text))
            )),
            _ => None,
        })
        .collect();
    if !manual.is_empty() {
        out.push_str("manual (planrun waits for you)\n");
        manual.iter().for_each(|l| out.push_str(l));
    }
}

fn first_line(text: &str) -> String {
    let line = text.lines().next().unwrap_or("");
    let short: String = line.chars().take(60).collect();
    if short.len() < line.len() {
        format!("{short}…")
    } else {
        short
    }
}
