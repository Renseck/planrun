//! The resolved view of a plan: the IR with every `TextRef` turned into text and every span into
//! a line (`L37`). `planrun show` prints it; the IR snapshot is its YAML.

use planrun_model::{
    Action, ActionKind, ExitExpect, Expectation, ManifestEntry, PathOrigin, Plan, Script, Span,
    Step, Task, TextRef,
};
use serde::Serialize;

/// A plan.
#[derive(Debug, Serialize)]
pub struct PlanView {
    /// The plan's name as given.
    pub plan: String,
    /// The format id.
    pub format: String,
    /// The first level-1 heading.
    pub title: Option<String>,
    /// The phase.
    pub phase: Option<String>,
    /// Manual-marked text before task 1.
    pub preamble: Vec<String>,
    /// The tasks.
    pub tasks: Vec<TaskView>,
    /// Where the text after the tasks starts (never run).
    pub after_tasks: Option<String>,
}

/// A task.
#[derive(Debug, Serialize)]
pub struct TaskView {
    /// `P1-T1`.
    pub id: String,
    /// Its title.
    pub title: String,
    /// Its heading's line.
    pub at: String,
    /// The manifest, one `Change path` per entry.
    pub files: Vec<String>,
    /// The steps.
    pub steps: Vec<StepView>,
}

/// A step.
#[derive(Debug, Serialize)]
pub struct StepView {
    /// `1 — write the files`, or `intro` for the blocks before the first step.
    pub step: String,
    /// Its first line.
    pub at: String,
    /// The actions.
    pub actions: Vec<ActionView>,
}

/// An action.
#[derive(Debug, Serialize)]
pub struct ActionView {
    /// Its first line.
    pub at: String,
    /// The recognizer that produced it.
    pub by: String,
    /// How the path was found when the plan did not write it in full (decision D32).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_from: Option<String>,
    /// What it does.
    #[serde(flatten)]
    pub what: What,
}

/// What an action does, resolved.
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum What {
    /// `WriteFile`.
    WriteFile {
        /// Target.
        path: String,
        /// Create / Overwrite / FromManifest.
        mode: String,
        /// The contents.
        body: String,
    },
    /// `ApplyPatch`.
    ApplyPatch {
        /// Target.
        path: String,
        /// Preimage blob id.
        pre: Option<String>,
        /// Postimage blob id.
        post: Option<String>,
        /// The patch.
        patch: String,
    },
    /// `ReplaceExact`.
    ReplaceExact {
        /// Target.
        path: String,
        /// The text to find.
        before: String,
        /// Its replacement.
        after: String,
    },
    /// `Delete`.
    Delete {
        /// Target.
        path: String,
        /// Through `git rm`.
        via_git: bool,
    },
    /// `Generated`.
    Generated {
        /// The generated file.
        path: String,
    },
    /// `RunScript`.
    Run(ScriptView),
    /// `Commit`.
    Commit(ScriptView),
    /// `Note`.
    Note {
        /// The text.
        text: String,
    },
    /// `Manual`.
    Manual {
        /// Why.
        reason: String,
        /// The text.
        text: String,
    },
}

/// A script.
#[derive(Debug, Serialize)]
pub struct ScriptView {
    /// pwsh / bash.
    pub shell: String,
    /// Verify / Gate / …
    pub intent: String,
    /// The script.
    pub body: String,
    /// What the plan expects.
    pub expect: Option<ExpectView>,
}

/// An expectation.
#[derive(Debug, Serialize)]
pub struct ExpectView {
    /// `exit 0`, `exit 3`, `fails`, `not checked`.
    pub exit: String,
    /// Summary lines.
    pub lines: Option<String>,
    /// The sentence.
    pub prose: Option<String>,
}

/// Builds the views of one plan.
pub struct Viewer<'a> {
    plan: &'a Plan,
}

impl<'a> Viewer<'a> {
    /// A viewer of `plan`.
    pub fn new(plan: &'a Plan) -> Self {
        Self { plan }
    }

    fn text(&self, text: &TextRef) -> String {
        self.plan.text(text).to_owned()
    }

    fn at(&self, span: Span) -> String {
        format!("L{}", self.plan.source.line_col(span.start).line)
    }

    /// The whole plan.
    pub fn plan(&self) -> PlanView {
        let meta = &self.plan.meta;
        PlanView {
            plan: self.plan.source.name.to_string(),
            format: self.plan.format.0.to_string(),
            title: meta.title.as_ref().map(|t| self.text(t)),
            phase: meta.phase.as_ref().map(|p| p.to_string()),
            preamble: meta.preamble.iter().map(|t| self.text(t)).collect(),
            tasks: self.plan.tasks.iter().map(|t| self.task(t)).collect(),
            after_tasks: meta.after_tasks.map(|s| self.at(s)),
        }
    }

    /// One task.
    pub fn task(&self, task: &Task) -> TaskView {
        TaskView {
            id: task.id.raw.to_string(),
            title: self.text(&task.title),
            at: self.at(task.span),
            files: task.manifest.entries.iter().map(entry).collect(),
            steps: task.steps.iter().map(|s| self.step(s)).collect(),
        }
    }

    /// One step.
    pub fn step(&self, step: &Step) -> StepView {
        let title = step.title.as_ref().map(|t| self.text(t));
        let name = match (step.number, title) {
            (Some(n), Some(t)) => format!("{n} — {t}"),
            (Some(n), None) => n.to_string(),
            (None, Some(t)) => t,
            (None, None) => "intro".to_owned(),
        };
        StepView {
            step: name,
            at: self.at(step.span),
            actions: step.actions.iter().map(|a| self.action(a)).collect(),
        }
    }

    fn action(&self, action: &Action) -> ActionView {
        ActionView {
            at: self.at(action.span),
            by: action.origin.0.to_string(),
            path_from: path_from(action.path_origin),
            what: self.what(&action.kind),
        }
    }

    fn what(&self, kind: &ActionKind) -> What {
        let id = |i: &Option<planrun_model::AbbrevBlobId>| i.map(|i| i.to_string());
        match kind {
            ActionKind::WriteFile { path, body, mode } => What::WriteFile {
                path: path.to_string(),
                mode: format!("{mode:?}"),
                body: self.text(body),
            },
            ActionKind::ApplyPatch {
                path,
                patch,
                pre,
                post,
            } => What::ApplyPatch {
                path: path.to_string(),
                pre: id(pre),
                post: id(post),
                patch: self.text(patch),
            },
            ActionKind::ReplaceExact {
                path,
                before,
                after,
            } => What::ReplaceExact {
                path: path.to_string(),
                before: self.text(before),
                after: self.text(after),
            },
            ActionKind::Delete { path, via_git } => What::Delete {
                path: path.to_string(),
                via_git: *via_git,
            },
            ActionKind::Generated { path } => What::Generated {
                path: path.to_string(),
            },
            ActionKind::RunScript(s) => What::Run(self.script(s)),
            ActionKind::Commit(s) => What::Commit(self.script(s)),
            ActionKind::Note { text } => What::Note {
                text: self.text(text),
            },
            ActionKind::Manual { reason, text } => What::Manual {
                reason: format!("{reason:?}"),
                text: self.text(text),
            },
        }
    }

    fn script(&self, script: &Script) -> ScriptView {
        ScriptView {
            shell: format!("{:?}", script.shell).to_lowercase(),
            intent: format!("{:?}", script.intent).to_lowercase(),
            body: self.text(&script.body),
            expect: script.expect.as_ref().map(|e| self.expect(e)),
        }
    }

    fn expect(&self, expect: &Expectation) -> ExpectView {
        let exit = match expect.exit {
            ExitExpect::Success => "exit 0".to_owned(),
            ExitExpect::Code(n) => format!("exit {n}"),
            ExitExpect::Failure => "fails".to_owned(),
            ExitExpect::Unknown => "not checked".to_owned(),
        };
        ExpectView {
            exit,
            lines: expect.lines.as_ref().map(|l| self.text(l)),
            prose: expect.prose.as_ref().map(|p| self.text(p)),
        }
    }
}

/// How a resolved path was found; `None` when the plan wrote it.
fn path_from(origin: PathOrigin) -> Option<String> {
    let how = match origin {
        PathOrigin::Written => return None,
        PathOrigin::Sibling => "the directory of the manifest path before it",
        PathOrigin::FileName => "the manifest entry with this file name",
        PathOrigin::OnlyEntry => "the task's only editable manifest entry",
    };
    Some(format!("resolved from manifest: {how}"))
}

fn entry(entry: &ManifestEntry) -> String {
    let lines = entry
        .lines
        .map(|l| format!(":{}-{}", l.start, l.end))
        .unwrap_or_default();
    let sibling = if entry.origin.is_written() {
        ""
    } else {
        " (in the directory of the path before)"
    };
    format!("{:?} {}{lines}{sibling}", entry.change, entry.path)
}

/// How much of each action `show` prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detail {
    /// One line per action.
    Summary,
    /// Every body in full.
    Full,
}

/// The text `planrun show` prints for a task.
pub fn task_text(task: &TaskView, detail: Detail, out: &mut String) {
    out.push_str(&format!("{}  {}  {}\n", task.id, task.title, task.at));
    if !task.files.is_empty() {
        out.push_str(&format!("  files  {}\n", task.files.join(", ")));
    }
    for step in &task.steps {
        step_text(step, detail, out);
    }
}

/// The text `planrun show` prints for a step.
pub fn step_text(step: &StepView, detail: Detail, out: &mut String) {
    out.push_str(&format!("  step {}  {}\n", step.step, step.at));
    for action in &step.actions {
        action_text(action, detail, out);
    }
}

fn action_text(action: &ActionView, detail: Detail, out: &mut String) {
    let (label, summary, body) = describe(&action.what);
    out.push_str(&format!(
        "    {:<6} {label:<11} {summary}  ({})\n",
        action.at, action.by
    ));
    if let Some(from) = &action.path_from {
        out.push_str(&format!("           path {from}\n"));
    }
    if let What::Run(s) | What::Commit(s) = &action.what
        && let Some(e) = &s.expect
    {
        out.push_str(&format!("           expect: {}\n", expect_summary(e)));
    }
    if detail == Detail::Full {
        for line in body.lines() {
            out.push_str(&format!("           | {line}\n"));
        }
    }
}

fn expect_summary(e: &ExpectView) -> String {
    let lines = e.lines.as_ref().map(|l| l.lines().count());
    match (lines, &e.prose) {
        (Some(n), _) => format!("{}; {n} summary line(s)", e.exit),
        (None, Some(p)) => format!("{}; \"{p}\" (confirmed by you)", e.exit),
        (None, None) => e.exit.clone(),
    }
}

/// A label, a one-line summary and the full body of an action.
fn describe(what: &What) -> (&'static str, String, &str) {
    match what {
        What::WriteFile { path, mode, body } => ("write-file", format!("{path} ({mode})"), body),
        What::ApplyPatch {
            path,
            pre,
            post,
            patch,
        } => {
            let ids = format!(
                "{}..{}",
                pre.as_deref().unwrap_or("?"),
                post.as_deref().unwrap_or("?")
            );
            ("apply-patch", format!("{path} {ids}"), patch)
        }
        What::ReplaceExact { path, before, .. } => ("replace", path.clone(), before),
        What::Delete { path, via_git } => {
            let how = if *via_git { " (git rm)" } else { "" };
            ("delete", format!("{path}{how}"), "")
        }
        What::Generated { path } => ("generated", path.clone(), ""),
        What::Run(s) => (
            "run",
            format!("{} {}  {}", s.shell, s.intent, first_line(&s.body)),
            &s.body,
        ),
        What::Commit(s) => (
            "commit",
            format!("{}  {}", s.shell, first_line(&s.body)),
            &s.body,
        ),
        What::Note { text } => ("note", first_line(text), text),
        What::Manual { reason, text } => {
            ("manual", format!("{reason}: {}", first_line(text)), text)
        }
    }
}

/// The first line of `text`, at most 72 characters.
fn first_line(text: &str) -> String {
    let line = text.lines().next().unwrap_or("");
    let short: String = line.chars().take(72).collect();
    if short.len() < line.len() {
        format!("{short}…")
    } else {
        short
    }
}

/// The text `planrun show` prints for a whole plan.
pub fn plan_text(plan: &PlanView, out: &mut String) {
    out.push_str(&format!("plan    {}\nformat  {}\n", plan.plan, plan.format));
    if let Some(title) = &plan.title {
        out.push_str(&format!("title   {title}\n"));
    }
    if let Some(phase) = &plan.phase {
        out.push_str(&format!("phase   {phase}\n"));
    }
    for text in &plan.preamble {
        out.push_str(&format!("read first: {}\n", first_line(text)));
    }
    for task in &plan.tasks {
        out.push('\n');
        task_text(task, Detail::Summary, out);
    }
    if let Some(at) = &plan.after_tasks {
        out.push_str(&format!("\nafter the tasks (never run): from {at}\n"));
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use planrun_parse::{Registry, parse_plan};

    use super::*;

    /// The synthetic plan `name` (`assay-style.md`), parsed with the built-in formats.
    fn synthetic_plan(name: &str) -> Plan {
        let relative = format!("tests/corpus/synthetic/{name}");
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(&relative);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
        parse_plan(&registry, &relative, &bytes, None)
            .unwrap_or_else(|d| panic!("{name}: {:?}", d.error_codes()))
            .plan
    }

    fn synthetic() -> Plan {
        synthetic_plan("assay-style.md")
    }

    #[test]
    fn the_ir_of_the_synthetic_assay_plan() {
        let plan = synthetic();
        insta::assert_yaml_snapshot!("assay_style_ir", Viewer::new(&plan).plan());
    }

    #[test]
    fn the_ir_of_the_synthetic_superpowers_plan() {
        let plan = synthetic_plan("superpowers-style.md");
        assert_eq!(&*plan.format.0, "superpowers");
        insta::assert_yaml_snapshot!("superpowers_style_ir", Viewer::new(&plan).plan());
    }

    #[test]
    fn show_prints_one_line_per_action() {
        let plan = synthetic();
        let mut out = String::new();
        plan_text(&Viewer::new(&plan).plan(), &mut out);
        insta::assert_snapshot!("assay_style_show", out);
    }
}
