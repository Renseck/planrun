//! `planrun run --dry-run` (docs/architecture.md §7, decision D55): one line per action with its
//! outcome, a summary, then every finding with its source context. A script or commit shows its
//! class and what the default policy would do with it (D68, D69).

use planrun_exec::finding::{Finding, Severity as ExecSeverity};
use planrun_exec::policy::{Decision, DefaultClassifier, Policy, Skip};
use planrun_exec::preflight::{DryRun, Outcome};
use planrun_model::{ActionKind, Plan, Task};
use planrun_parse::{Diagnostic, Severity};

use crate::render;

/// Counts for the summary line.
#[derive(Default)]
struct Counts {
    ok: usize,
    ask: usize,
    failed: usize,
    depends: usize,
    not_run: usize,
    not_checked: usize,
    waits: usize,
}

/// The report of `run`, rendered into `out`.
pub fn report(plan: &Plan, run: &DryRun, out: &mut String) {
    let name = plan.source.name.to_string();
    out.push_str(&format!(
        "dry run of {name} ({}, {} tasks): nothing is written, no script runs\n",
        plan.format.0,
        plan.tasks.len()
    ));
    out.push_str(
        "after the first script, commit or Manual step, each result assumes it changed no files\n",
    );
    for path in &run.attribute_writes {
        out.push_str(&format!(
            "note: the plan writes `{path}`; the dry run reads attributes from the disk as they are\n"
        ));
    }
    let mut counts = Counts::default();
    for task in &run.tasks {
        let t = &plan.tasks[task.task];
        out.push_str(&format!("\n{}  {}\n", t.id, plan.text(&t.title)));
        if !task.manifest.is_empty() {
            let errors = task.manifest.iter().filter(|f| f.is_error()).count();
            let warnings = task.manifest.len() - errors;
            out.push_str(&format!(
                "  the task's files: {errors} error(s), {warnings} warning(s)\n"
            ));
        }
        for a in &task.actions {
            if let Some(line) = action_line(plan, t, a, &mut counts) {
                out.push_str(&line);
            }
        }
    }
    out.push_str(&format!(
        "\nsummary: {} ok, {} would ask, {} failed, {} depend on a step that did not run; \
         {} scripts and commits not run, {} generated files not checked, {} manual\n",
        counts.ok,
        counts.ask,
        counts.failed,
        counts.depends,
        counts.not_run,
        counts.not_checked,
        counts.waits
    ));
    let findings: Vec<Diagnostic> = run.findings().map(diagnostic).collect();
    if !findings.is_empty() {
        out.push('\n');
        render::diagnostics(&name, plan.source.text(), &findings, out);
    }
}

/// One table line; `None` for a Note.
fn action_line(
    plan: &Plan,
    task: &Task,
    a: &planrun_exec::preflight::ActionReport,
    counts: &mut Counts,
) -> Option<String> {
    let (what, path) = describe(&a.action.kind);
    let outcome = match &a.outcome {
        Outcome::Shown => return None,
        Outcome::Ok(detail) => {
            counts.ok += 1;
            format!("{detail}: ok")
        }
        Outcome::WouldAsk(detail) => {
            counts.ask += 1;
            format!("{detail}: would ask (not in the task's files, D50)")
        }
        Outcome::Failed(f) => {
            counts.failed += 1;
            format!("FAILED {}", f.code)
        }
        Outcome::Depends(f) => {
            counts.depends += 1;
            format!("unverified {} (depends on a step that did not run)", f.code)
        }
        Outcome::NotRun => {
            counts.not_run += 1;
            let decision =
                Policy::default().decide(task, &a.action, &plan.source, &DefaultClassifier);
            format!("not run; {}", decided(decision))
        }
        Outcome::NotChecked => {
            counts.not_checked += 1;
            "not checked (its script did not run)".to_owned()
        }
        Outcome::Waits => {
            counts.waits += 1;
            "waits for you".to_owned()
        }
    };
    let line = plan.source.line_col(a.action.span.start).line;
    Some(format!(
        "  L{line:<5} {what:<9} {path:<24} {outcome}\n",
        path = path.unwrap_or_default()
    ))
}

/// What the default policy (no `--auto`) would do with a script or commit.
fn decided(decision: Decision) -> String {
    match decision {
        Decision::Ask(class) => format!("would ask ({class:?})"),
        Decision::TypedYes => "typed yes (Destructive)".to_owned(),
        Decision::Skip(Skip::ExpectedFailure) => "skipped: expected failure".to_owned(),
        other => format!("{other:?}"),
    }
}

fn describe(kind: &ActionKind) -> (&'static str, Option<String>) {
    match kind {
        ActionKind::WriteFile { path, .. } => ("write", Some(path.to_string())),
        ActionKind::ApplyPatch { path, .. } => ("patch", Some(path.to_string())),
        ActionKind::ReplaceExact { path, .. } => ("replace", Some(path.to_string())),
        ActionKind::Delete { path, .. } => ("delete", Some(path.to_string())),
        ActionKind::Generated { path } => ("generated", Some(path.to_string())),
        ActionKind::RunScript(_) => ("script", None),
        ActionKind::Commit(_) => ("commit", None),
        ActionKind::Manual { .. } => ("manual", None),
        ActionKind::Note { .. } => ("note", None),
    }
}

/// A finding as a diagnostic, so it renders like the parser's.
fn diagnostic(f: &Finding) -> Diagnostic {
    Diagnostic {
        severity: match f.severity {
            ExecSeverity::Error => Severity::Error,
            ExecSeverity::Warning => Severity::Warning,
        },
        code: f.code,
        span: f.span,
        message: f.message.clone(),
        related: Vec::new(),
    }
}
