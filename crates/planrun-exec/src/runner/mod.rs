//! The runner (docs/architecture.md §6.1; decisions D14, D22, D69, D73–D81): Task → Step →
//! Action over the disk, the journal, the shell sessions and a [`Ui`].
//!
//! Per task: preflight, `task-start`, the steps in order, the `Generated` checks before the
//! commit (or at the end), `task-end`. Per step: consecutive file actions are one transaction
//! whose write-ahead record is the journal's `step-prepared`; scripts run one at a time in their
//! shell's session and are judged by the matcher; a Commit runs only behind the commit gate.
//! P4's [`Policy`] decides; the runner prompts (D69) and never turns one decision into another.

mod gate;
pub mod resume;
mod script;
mod sessions;
mod task;

use std::io;
use std::path::PathBuf;

use planrun_model::{Action, ActionKind, Plan, Task};

use crate::git::{GitError, Head};
use crate::handler::{HandlerSet, file_path};
use crate::journal::{At, End, Journal, Record};
use crate::policy::{DefaultClassifier, Policy, PolicyClass};
use crate::shell::{SessionConfig, ShellError};
use crate::ui::{Choice, Position, Question, QuestionKind, Reply, Ui};
use crate::workspace::{Workspace, WorkspaceError};

use sessions::Sessions;

/// What a run is given besides the plan.
#[derive(Debug, Clone)]
pub struct RunConfig {
    /// The plan's path as the journal records it.
    pub plan_path: String,
    /// The policy (`--auto`, `--run-expected-failures`).
    pub policy: Policy,
    /// The first task to run, by position (`--from`).
    pub from: Option<usize>,
    /// The last task to run, by position (`--to`).
    pub to: Option<usize>,
    /// How sessions start (root, environment, timeout, temp directory).
    pub session: SessionConfig,
}

/// How a run ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finished {
    /// Completed or stopped.
    pub end: End,
    /// Why it stopped.
    pub reason: Option<String>,
}

/// What stops a run that is not a finding: the journal, git, the file system, a shell.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// The journal could not be written.
    #[error("the journal: {0}")]
    Journal(#[source] io::Error),
    /// A git call failed.
    #[error(transparent)]
    Git(#[from] GitError),
    /// The repository or a path could not be used.
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    /// A shell could not be started.
    #[error(transparent)]
    Shell(#[from] ShellError),
}

pub(crate) type Result<T> = std::result::Result<T, RunError>;

/// Whether the run goes on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Flow {
    Go,
    Stop(String),
}

/// A plan being run.
pub struct Runner<'r> {
    plan: &'r Plan,
    ws: &'r Workspace,
    journal: &'r mut Journal,
    ui: &'r mut dyn Ui,
    config: RunConfig,
    classifier: DefaultClassifier,
    handlers: HandlerSet,
    sessions: Sessions,
    /// A notice to acknowledge before the next block: the session is new (D81).
    notice: Option<Notice>,
    /// Tasks whose scripts ran before, for the notice.
    earlier: Vec<String>,
    /// What resume replays of the interrupted task (D77).
    replay: Option<resume::Replay>,
}

impl<'r> Runner<'r> {
    /// A runner over `plan` in `ws`, journaling to `journal` and asking `ui`.
    pub fn new(
        plan: &'r Plan,
        ws: &'r Workspace,
        journal: &'r mut Journal,
        ui: &'r mut dyn Ui,
        config: RunConfig,
    ) -> Runner<'r> {
        let sessions = Sessions::new(config.session.clone());
        Runner {
            plan,
            ws,
            journal,
            ui,
            config,
            classifier: DefaultClassifier,
            handlers: HandlerSet::files(),
            sessions,
            notice: None,
            earlier: Vec::new(),
            replay: None,
        }
    }

    /// Runs the plan's tasks from `--from` to `--to`. Every way it ends is journaled
    /// (`run-end`), except a kill.
    pub fn run(&mut self) -> Result<Finished> {
        let result = self.start();
        let (end, reason) = match &result {
            Ok(Flow::Go) => (End::Completed, None),
            Ok(Flow::Stop(reason)) => (End::Stopped, Some(reason.clone())),
            Err(e) => (End::Stopped, Some(e.to_string())),
        };
        let ended = self.append(Record::RunEnd {
            end,
            reason: reason.clone(),
        });
        result?;
        ended?;
        Ok(Finished { end, reason })
    }

    fn start(&mut self) -> Result<Flow> {
        self.start_record(false)?;
        let from = self.config.from.unwrap_or(0);
        if from > 0 {
            self.earlier = self.plan.tasks[..from]
                .iter()
                .map(|t| t.id.raw.to_string())
                .collect();
            self.notice = Some(self.notice("the run starts at a later task (--from)"));
        }
        self.tasks_from(from)
    }

    /// The `run` record: a run, or a resume (D76).
    fn start_record(&mut self, resume: bool) -> Result<()> {
        let policy = &self.config.policy;
        let auto = policy.auto().iter().map(|c| class_name(*c)).collect();
        let record = Record::Run {
            plan: self.config.plan_path.clone(),
            fingerprint: self.plan.source.fingerprint.to_string(),
            format: self.plan.format.0.to_string(),
            root: self.ws.root().display().to_string(),
            head: self.ws.git().head()?,
            auto,
            run_expected_failures: policy.runs_expected_failures(),
            from: self
                .config
                .from
                .map(|i| self.plan.tasks[i].id.raw.to_string()),
            to: self
                .config
                .to
                .map(|i| self.plan.tasks[i].id.raw.to_string()),
            resume,
            planrun: env!("CARGO_PKG_VERSION").to_owned(),
        };
        self.append(record)
    }

    /// Runs the tasks from position `from` to `--to`.
    fn tasks_from(&mut self, from: usize) -> Result<Flow> {
        let to = self
            .config
            .to
            .unwrap_or(self.plan.tasks.len().saturating_sub(1));
        for index in from..=to.min(self.plan.tasks.len().saturating_sub(1)) {
            if let Flow::Stop(reason) = self.task(index)? {
                return Ok(Flow::Stop(reason));
            }
            self.earlier.push(self.plan.tasks[index].id.raw.to_string());
        }
        Ok(Flow::Go)
    }

    fn append(&mut self, record: Record) -> Result<()> {
        self.journal.append(record).map_err(RunError::Journal)
    }

    fn head(&self) -> Result<Head> {
        Ok(self.ws.git().head()?)
    }

    fn at(&self, pos: Position) -> At {
        At {
            task: self.plan.tasks[pos.task].id.raw.to_string(),
            step: pos.step,
            action: pos.action,
        }
    }

    fn line(&self, action: &Action) -> u32 {
        self.plan.source.line_col(action.span.start).line
    }

    /// The notice that a session is new (D81), with the tasks whose scripts ran before.
    fn notice(&self, why: &str) -> Notice {
        Notice {
            why: why.to_owned(),
            earlier: self.earlier.clone(),
        }
    }

    /// Asks `question` until it gets a choice or a typed line: `[v]`, `[d]` and `[o]` are
    /// handled here, and HEAD is read again after `[o]` (D80). Every answer is journaled.
    fn ask(&mut self, question: Question) -> Result<Answer> {
        loop {
            match self.ui.ask(&question) {
                Reply::Choice(Choice::View) => match question.at {
                    Some(at) => self.ui.view(at),
                    None => self.ui.status("there is no action to view here"),
                },
                Reply::Choice(Choice::Diff) => self.ui.diff(),
                Reply::Choice(Choice::Shell) => {
                    self.ui.shell();
                    let head = self.head()?;
                    self.append(Record::HeadSeen { head })?;
                }
                Reply::Choice(choice) if !question.choices.contains(&choice) => {
                    self.ui.status("that is not one of the answers offered");
                }
                Reply::Choice(choice) => {
                    self.answered(&question, choice.label())?;
                    return Ok(Answer::Choice(choice));
                }
                Reply::Typed(line) => {
                    self.answered(&question, &line)?;
                    return Ok(Answer::Typed(line));
                }
                Reply::Stop(why) => return Ok(Answer::Stop(why)),
            }
        }
    }

    fn answered(&mut self, question: &Question, answer: &str) -> Result<()> {
        let at = question.at.map(|p| self.at(p));
        self.append(Record::Answer {
            at,
            question: question.kind.name().to_owned(),
            answer: answer.to_owned(),
        })
    }

    /// Asks for a typed `yes` (D74): exactly `yes` agrees, `quit` stops, `skip` skips when the
    /// question's words offer it; anything else does not run and asks again.
    fn typed_yes(&mut self, question: Question) -> Result<Typed> {
        let skippable = question.typed.is_some_and(|words| words.contains(&"skip"));
        loop {
            match self.ask(question.clone())? {
                Answer::Typed(line) if line == "yes" => return Ok(Typed::Yes),
                Answer::Typed(line) if line == "quit" => return Ok(Typed::Quit),
                Answer::Typed(line) if line == "skip" && skippable => return Ok(Typed::Skip),
                Answer::Stop(why) => return Ok(Typed::Stop(why)),
                _ => self.ui.status(if skippable {
                    "type `yes` to run it, `skip` to skip it, `quit` to stop"
                } else {
                    "type `yes` to go on, or `quit` to stop"
                }),
            }
        }
    }

    /// Asks the pending session notice, if any; it is acknowledged even with `--auto` (D81).
    fn acknowledge_notice(&mut self, at: Option<Position>) -> Result<Flow> {
        let Some(notice) = self.notice.take() else {
            return Ok(Flow::Go);
        };
        let question = Question {
            text: notice.text(),
            kind: QuestionKind::Session {
                why: notice.why,
                earlier: notice.earlier,
            },
            at,
            choices: vec![Choice::Continue, Choice::Shell, Choice::Quit],
            typed: None,
        };
        Ok(match self.ask(question)? {
            Answer::Choice(Choice::Continue) => Flow::Go,
            Answer::Stop(why) => Flow::Stop(why),
            _ => Flow::Stop("stopped at the session notice".to_owned()),
        })
    }
}

/// A pending session notice (D81): why the session is new, and the tasks whose scripts ran
/// before.
pub(crate) struct Notice {
    why: String,
    earlier: Vec<String>,
}

impl Notice {
    /// What is lost, and which tasks' scripts ran before.
    fn text(&self) -> Vec<String> {
        let mut text = vec![
            format!("This is a new shell session: {}.", self.why),
            "Variables, functions and the working directory set by earlier blocks are gone."
                .to_owned(),
        ];
        if !self.earlier.is_empty() {
            text.push(format!(
                "Scripts of these earlier tasks ran in another session: {}.",
                self.earlier.join(", ")
            ));
        }
        text
    }
}

/// An answer the runner acts on.
pub(crate) enum Answer {
    Choice(Choice),
    Typed(String),
    Stop(String),
}

/// What a typed question came to.
pub(crate) enum Typed {
    Yes,
    Skip,
    Quit,
    Stop(String),
}

fn class_name(class: PolicyClass) -> String {
    format!("{class:?}").to_lowercase()
}

/// A short description of an action: `write src/a.rs`, `script (pwsh)`.
pub(crate) fn describe(action: &Action) -> String {
    let path = file_path(&action.kind).map(|p| p.to_string());
    let what = match &action.kind {
        ActionKind::WriteFile { .. } => "write",
        ActionKind::ApplyPatch { .. } => "patch",
        ActionKind::ReplaceExact { .. } => "replace",
        ActionKind::Delete { .. } => "delete",
        ActionKind::Generated { path } => return format!("generated {path}"),
        ActionKind::RunScript(s) => return format!("script ({})", sessions::name(s.shell)),
        ActionKind::Commit(s) => return format!("commit ({})", sessions::name(s.shell)),
        ActionKind::Note { .. } => "note",
        ActionKind::Manual { .. } => "manual",
    };
    match path {
        Some(path) => format!("{what} {path}"),
        None => what.to_owned(),
    }
}

/// True when `task` is a file action.
pub(crate) fn is_file_action(action: &Action) -> bool {
    file_path(&action.kind).is_some()
}

/// The canonical form of `dir`; `None` when it is no file-system path planrun can resolve
/// (a pwsh provider path such as `HKLM:\`).
pub(crate) fn canonical(dir: &std::path::Path) -> Option<PathBuf> {
    std::fs::canonicalize(dir).ok()
}

/// The paths a task names: its file actions, its `Generated` paths, and its manifest's file
/// entries (patterns are matched separately).
pub(crate) fn task_paths(task: &Task) -> Vec<planrun_model::RepoPath> {
    let mut paths: Vec<planrun_model::RepoPath> = Vec::new();
    let actions = task.steps.iter().flat_map(|s| s.actions.iter());
    let named = actions.filter_map(|a| match &a.kind {
        ActionKind::Generated { path } => Some(path),
        kind => file_path(kind),
    });
    let entries = task.manifest.entries.iter().filter_map(|e| e.path.file());
    for path in named.chain(entries) {
        if !paths.contains(path) {
            paths.push(path.clone());
        }
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_names_are_the_auto_spellings() {
        assert_eq!(class_name(PolicyClass::Build), "build");
    }
}
