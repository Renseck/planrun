//! Recognition (docs/architecture.md §4.4): for each step, a cursor walks its blocks and the
//! format's recognizers are tried in order; the first that answers anything but `No` wins. A
//! block no recognizer takes goes to the built-in fallback, so every block is accounted for
//! (decision D21).

use planrun_model::{
    Action, ActionKind, ExitExpect, Expectation, ManualReason, RecognizerId, ScriptIntent, Span,
    Step, Task, TaskId, TextRef,
};

use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::document::{Block, Document};
use crate::format::{Format, Outline, OutlineStep, OutlineTask, StepMark};
use crate::recognizer::{Cursor, Recognized, StepContext};

/// Where one task's blocks went (the coverage report, docs/architecture.md §4.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskCoverage {
    /// The task.
    pub task: TaskId,
    /// Blocks that became actions other than `Manual`, expectations included.
    pub actions: usize,
    /// Blocks absorbed without an action (the patch ceremony), with where they are.
    pub absorbed: Vec<Absorbed>,
    /// The manifest's blocks.
    pub manifest: usize,
    /// Blocks that became `Note`.
    pub notes: usize,
    /// Blocks that became `Manual`, through the fallback or a recognizer (`manual-prose`).
    pub manual: usize,
    /// HTML comments, dropped.
    pub dropped: usize,
}

impl TaskCoverage {
    /// Every block of the task's steps and manifest.
    pub fn blocks(&self) -> usize {
        self.actions + self.absorbed.len() + self.manifest + self.notes + self.manual + self.dropped
    }
}

/// One absorbed block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Absorbed {
    /// The recognizer that absorbed it.
    pub recognizer: RecognizerId,
    /// The block.
    pub span: Span,
}

/// The recognizer id of the built-in fallback.
pub const FALLBACK: &str = "fallback";

/// The tasks of `outline` with their actions, and the coverage of each. Every `Malformed` is
/// collected; any means no tasks.
pub fn recognize(
    format: &dyn Format,
    doc: &Document,
    outline: &Outline,
) -> Result<(Vec<Task>, Vec<TaskCoverage>), Diagnostics> {
    let mut errors = Vec::new();
    let mut tasks = Vec::with_capacity(outline.tasks.len());
    let mut coverage = Vec::with_capacity(outline.tasks.len());
    for task in &outline.tasks {
        let mut cov = TaskCoverage {
            task: task.id.clone(),
            actions: 0,
            absorbed: Vec::new(),
            manifest: task.manifest_blocks.map_or(0, |m| m.len()),
            notes: 0,
            manual: 0,
            dropped: 0,
        };
        let steps = task
            .steps
            .iter()
            .map(|s| step(format, doc, task, s, &mut cov, &mut errors))
            .collect();
        tasks.push(Task {
            id: task.id.clone(),
            title: task.title.clone(),
            span: task.span,
            manifest: task.manifest.clone(),
            steps,
        });
        coverage.push(cov);
    }
    if !errors.is_empty() {
        return Err(Diagnostics(errors));
    }
    Ok((tasks, coverage))
}

fn step(
    format: &dyn Format,
    doc: &Document,
    task: &OutlineTask,
    outline: &OutlineStep,
    cov: &mut TaskCoverage,
    errors: &mut Vec<Diagnostic>,
) -> Step {
    let mut cursor = Cursor::with_skip(doc, outline.blocks, task.manifest_blocks);
    let mut actions: Vec<Action> = Vec::new();
    while let Some(at) = cursor.peek(0).map(|b| b.span) {
        let before = cursor.consumed();
        let cx = StepContext {
            doc,
            task: &task.id,
            manifest: &task.manifest,
            settings: format.settings(),
            previous: &actions,
            written_step: outline.written,
        };
        let (origin, outcome) = first_match(format, &cx, &mut cursor);
        let consumed = cursor.consumed() - before;
        match outcome {
            Recognized::No => {
                if let Some(action) = fallback(&cx, &mut cursor, cov) {
                    actions.push(action);
                }
            }
            Recognized::Malformed(d) => {
                errors.push(d);
                cursor.advance(1);
            }
            _ if consumed == 0 => {
                errors.push(Diagnostic::error(
                    "recognizer-stalled",
                    Some(at),
                    format!("recognizer `{origin}` matched without consuming a block"),
                ));
                cursor.advance(1);
            }
            Recognized::Yes(new) if new.is_empty() => cov.absorbed.push(Absorbed {
                recognizer: RecognizerId(origin.into()),
                span: at,
            }),
            Recognized::Yes(new) => {
                let manual = new
                    .iter()
                    .all(|a| matches!(a.kind, ActionKind::Manual { .. }));
                match manual {
                    true => cov.manual += consumed,
                    false => cov.actions += consumed,
                }
                actions.extend(new);
            }
            Recognized::Amend(action) => {
                match action.kind {
                    ActionKind::Manual { .. } => cov.manual += consumed,
                    _ => cov.actions += consumed,
                }
                actions.pop();
                actions.push(action);
            }
        }
    }
    apply_mark(outline.mark, &mut actions, cov, errors);
    Step {
        number: outline.number,
        title: outline.title.clone(),
        span: outline.span,
        actions,
    }
}

/// Applies the mark of a step's title to its actions (decision D30).
fn apply_mark(
    mark: Option<StepMark>,
    actions: &mut [Action],
    cov: &mut TaskCoverage,
    errors: &mut Vec<Diagnostic>,
) {
    match mark {
        None => {}
        Some(StepMark::ExpectedFailure) => {
            errors.extend(actions.iter_mut().filter_map(expect_failure));
        }
        Some(StepMark::HumanCheck) => {
            for action in actions.iter_mut() {
                let ActionKind::Note { text } = &action.kind else {
                    continue;
                };
                action.kind = ActionKind::Manual {
                    reason: ManualReason::HumanCheck,
                    text: text.clone(),
                };
                cov.notes = cov.notes.saturating_sub(1);
                cov.manual += 1;
            }
        }
    }
}

/// An `expected-failure` step's `Verify` script expects failure: none or `any` becomes
/// `Failure`; an expected success is `expected-conflict`. Commits and gates are left alone.
fn expect_failure(action: &mut Action) -> Option<Diagnostic> {
    let span = action.span;
    let ActionKind::RunScript(script) = &mut action.kind else {
        return None;
    };
    if script.intent != ScriptIntent::Verify {
        return None;
    }
    let Some(expect) = &mut script.expect else {
        script.expect = Some(Expectation {
            exit: ExitExpect::Failure,
            lines: None,
            prose: None,
        });
        return None;
    };
    match expect.exit {
        ExitExpect::Unknown => {
            expect.exit = ExitExpect::Failure;
            None
        }
        ExitExpect::Success => Some(Diagnostic::error(
            "expected-conflict",
            Some(span),
            "the step's title says this fails, and its expectation says it succeeds",
        )),
        ExitExpect::Failure | ExitExpect::Code(_) => None,
    }
}

/// The first recognizer that answers anything but `No`, with its id.
fn first_match<'f>(
    format: &'f dyn Format,
    cx: &StepContext<'_>,
    cursor: &mut Cursor<'_>,
) -> (&'f str, Recognized) {
    for recognizer in format.recognizers() {
        let outcome = recognizer.recognize(cx, cursor);
        if !matches!(outcome, Recognized::No) {
            return (recognizer.id(), outcome);
        }
    }
    (FALLBACK, Recognized::No)
}

/// The built-in fallback: consumes one block. Prose becomes `Note` (or `Manual(Marked)`), a
/// fence `Manual`, a `text` fence `Note`; an HTML comment is dropped.
fn fallback(
    cx: &StepContext<'_>,
    cursor: &mut Cursor<'_>,
    cov: &mut TaskCoverage,
) -> Option<Action> {
    let located = cursor.peek(0)?;
    cursor.advance(1);
    let whole = TextRef::Span(located.span);
    let prose = |text: &TextRef| {
        if cx.settings.is_marked(cx.doc.text(text)) {
            ActionKind::Manual {
                reason: ManualReason::Marked,
                text: text.clone(),
            }
        } else {
            ActionKind::Note { text: text.clone() }
        }
    };
    let kind = match &located.block {
        Block::Paragraph { text }
        | Block::ListItem { text, .. }
        | Block::Quote { text }
        | Block::Heading { text, .. } => prose(text),
        Block::Table { text } => ActionKind::Note { text: text.clone() },
        Block::Rule => ActionKind::Note { text: whole },
        Block::Html { text } if cx.doc.text(text).trim_start().starts_with("<!--") => {
            cov.dropped += 1;
            return None;
        }
        Block::Html { text } => ActionKind::Note { text: text.clone() },
        Block::Fence { .. } => fence(cx, &located.block, whole),
    };
    match kind {
        ActionKind::Manual { .. } => cov.manual += 1,
        _ => cov.notes += 1,
    }
    Some(Action {
        kind,
        span: located.span,
        origin: RecognizerId(FALLBACK.into()),
        path_origin: Default::default(),
    })
}

fn fence(cx: &StepContext<'_>, block: &Block, whole: TextRef) -> ActionKind {
    let language = block.fence_language(cx.doc).unwrap_or("");
    let reason = match language {
        "text" => return ActionKind::Note { text: whole },
        "" => ManualReason::Unrecognized,
        shell if cx.settings.shell_for(shell).is_some() => ManualReason::Unrecognized,
        other => ManualReason::ForeignLanguage(other.into()),
    };
    ActionKind::Manual {
        reason,
        text: whole,
    }
}
