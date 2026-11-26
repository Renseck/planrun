//! Resume (docs/architecture.md §6.6; decisions D14, D57, D77): what the journal says, checked
//! against the disk and HEAD, never trusted alone.
//!
//! The last chain of the journal (a `run`, and every `resume` after it) is read into an
//! [`Analysis`]. Resume stops when the plan's fingerprint differs, when `.git/index.lock`
//! exists, and when HEAD is not what the journal last recorded (except for the one action that
//! was running: a Commit whose new HEAD sits on the recorded one and holds the task's files is
//! done; any other action that moved HEAD is the user's call). A resume's own `run` record
//! never sets the recorded HEAD: only the records written after HEAD was checked or accepted
//! do, so a resume that stopped leaves the check as it found it. The interrupted task runs again
//! from its start in a new session: file steps done on disk are verified, not written again;
//! scripts run again under their decisions, Modify and Destructive ones asking first; a script
//! that was running asks whatever its class.

use std::collections::HashMap;

use planrun_model::RepoPath;

use super::{Answer, Flow, Result, Runner, Typed, task_paths};
use crate::DoneState;
use crate::done::{Seen, remove_temps, restorable, restore_step, step_state};
use crate::git::Head;
use crate::journal::{At, End, Line, Outcome, PathBlob, Record};
use crate::preflight::Resumed;
use crate::transaction::StepRecord;
use crate::ui::{Choice, Question};

/// What the run record of the chain says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunInfo {
    /// The plan's path as recorded.
    pub plan: String,
    /// The plan's fingerprint.
    pub fingerprint: String,
    /// `--auto`.
    pub auto: Vec<String>,
    /// `--run-expected-failures`.
    pub run_expected_failures: bool,
    /// `--from`.
    pub from: Option<String>,
    /// `--to`.
    pub to: Option<String>,
}

/// What became of an action, as journaled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Past {
    /// `action-start` without its end: it was running when planrun stopped.
    Running {
        /// `script`, `commit`, `manual`, `generated`.
        kind: String,
        /// HEAD before it.
        head: Head,
        /// For a Commit, the task's paths it was to hold.
        expect: Vec<PathBlob>,
    },
    /// It ended so.
    Ended(Outcome),
}

/// A step the journal prepared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    /// Where.
    pub at: At,
    /// What it writes.
    pub step: StepRecord,
    /// `step-written`, `step-verified` or `step-accepted` followed it.
    pub written: bool,
}

/// The last chain of a journal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    /// The chain's first `run` record.
    pub run: RunInfo,
    /// The tasks with a `task-end`, in order.
    pub done: Vec<String>,
    /// The task with a `task-start` and no `task-end`, if any.
    pub interrupted: Option<String>,
    /// HEAD as last recorded: by the first `run`, `task-start`, `action-end`, `head-seen` or
    /// `task-end`; never by a resume's `run` record, which is written before HEAD is checked.
    pub head: Head,
    /// How the chain's last process ended; `None` when it was killed.
    pub end: Option<End>,
    /// The interrupted task's prepared steps, the latest per step.
    pub prepared: Vec<Prepared>,
    /// The interrupted task's actions, the latest record per action.
    pub past: Vec<(At, Past)>,
}

impl Analysis {
    /// Whether there is anything to resume: the chain's last process did not complete it.
    pub fn resumable(&self) -> bool {
        self.end != Some(End::Completed)
    }

    /// The action that was running when planrun stopped, if any.
    pub fn running(&self) -> Option<(&At, &Past)> {
        self.past
            .iter()
            .find(|(_, p)| matches!(p, Past::Running { .. }))
            .map(|(at, p)| (at, p))
    }
}

/// Reads the journal's last chain. `None` when the journal has no run.
pub fn analyse(lines: &[Line]) -> Option<Analysis> {
    let start = lines
        .iter()
        .rposition(|l| matches!(l.record, Record::Run { resume: false, .. }))?;
    let mut a: Option<Analysis> = None;
    let mut prepared: Vec<Prepared> = Vec::new();
    let mut past: Vec<(At, Past)> = Vec::new();
    for line in &lines[start..] {
        let record = &line.record;
        if let Record::Run {
            plan,
            fingerprint,
            head,
            auto,
            run_expected_failures,
            from,
            to,
            resume,
            ..
        } = record
            && !resume
        {
            a = Some(Analysis {
                run: RunInfo {
                    plan: plan.clone(),
                    fingerprint: fingerprint.clone(),
                    auto: auto.clone(),
                    run_expected_failures: *run_expected_failures,
                    from: from.clone(),
                    to: to.clone(),
                },
                done: Vec::new(),
                interrupted: None,
                head: head.clone(),
                end: None,
                prepared: Vec::new(),
                past: Vec::new(),
            });
        }
        let Some(a) = a.as_mut() else {
            continue;
        };
        apply(a, record, &mut prepared, &mut past);
    }
    let mut a = a?;
    let task = a.interrupted.clone();
    a.prepared = prepared
        .into_iter()
        .filter(|p| Some(&p.at.task) == task.as_ref())
        .collect();
    a.past = past
        .into_iter()
        .filter(|(at, _)| Some(&at.task) == task.as_ref())
        .collect();
    Some(a)
}

fn upsert<T>(list: &mut Vec<(At, T)>, at: &At, value: T) {
    list.retain(|(k, _)| k != at);
    list.push((at.clone(), value));
}

fn apply(
    a: &mut Analysis,
    record: &Record,
    prepared: &mut Vec<Prepared>,
    past: &mut Vec<(At, Past)>,
) {
    let mark = |prepared: &mut Vec<Prepared>, at: &At, written: bool| {
        if let Some(p) = prepared.iter_mut().find(|p| p.at == *at) {
            p.written = written;
        }
    };
    match record {
        Record::Run { head, resume, .. } => {
            if !resume {
                a.head = head.clone();
            }
            a.end = None;
        }
        Record::HeadSeen { head } => a.head = head.clone(),
        Record::TaskStart { task, head } => {
            a.interrupted = Some(task.clone());
            a.head = head.clone();
        }
        Record::TaskEnd { task, head } => {
            if !a.done.contains(task) {
                a.done.push(task.clone());
            }
            a.interrupted = None;
            a.head = head.clone();
        }
        Record::StepPrepared { at, step } => {
            prepared.retain(|p| p.at != *at);
            prepared.push(Prepared {
                at: at.clone(),
                step: step.clone(),
                written: false,
            });
        }
        Record::StepWritten { at } | Record::StepVerified { at } | Record::StepAccepted { at } => {
            mark(prepared, at, true);
        }
        Record::StepRestored { at } => mark(prepared, at, false),
        Record::ActionStart {
            at,
            kind,
            head,
            expect,
            ..
        } => {
            let running = Past::Running {
                kind: kind.clone(),
                head: head.clone(),
                expect: expect.clone(),
            };
            upsert(past, at, running);
        }
        Record::ActionEnd {
            at, outcome, head, ..
        } => {
            upsert(past, at, Past::Ended(*outcome));
            a.head = head.clone();
        }
        Record::RunEnd { end, .. } => a.end = Some(*end),
        _ => {}
    }
}

/// What the runner replays for the interrupted task.
#[derive(Debug, Default)]
pub(crate) struct Replay {
    pub task: usize,
    pub prepared: Vec<Prepared>,
    pub past: HashMap<(usize, usize), Past>,
    /// File groups found done on disk (or accepted), by their first action.
    pub done_groups: Vec<(usize, usize)>,
    /// A Commit that was running and is found made.
    pub commit_done: Option<(usize, usize)>,
}

fn key(at: &At) -> (usize, usize) {
    (at.step, at.action)
}

impl Runner<'_> {
    /// Resumes the chain `analysis` describes (D14, D77).
    pub fn resume(&mut self, analysis: &Analysis) -> Result<super::Finished> {
        let result = self.start_resume(analysis);
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
        Ok(super::Finished { end, reason })
    }

    fn start_resume(&mut self, analysis: &Analysis) -> Result<Flow> {
        self.start_record(true)?;
        let plan = self.plan;
        if analysis.run.fingerprint != *plan.source.fingerprint {
            return Ok(Flow::Stop(format!(
                "the plan changed since the run: its fingerprint is {}, the journal's {}",
                plan.source.fingerprint, analysis.run.fingerprint
            )));
        }
        let lock = self.ws.git().git_dir()?.join("index.lock");
        if lock.exists() {
            return Ok(Flow::Stop(format!(
                "{} exists: another git command is running, or one crashed; planrun never \
                 removes it",
                lock.display()
            )));
        }
        let index_of = |id: &str| plan.tasks.iter().position(|t| &*t.id.raw == id);
        let mut replay = Replay::default();
        if let Some(task) = analysis.interrupted.as_deref().and_then(index_of) {
            replay.task = task;
            replay.prepared = analysis.prepared.clone();
            replay.past = analysis
                .past
                .iter()
                .map(|(at, p)| (key(at), p.clone()))
                .collect();
        }
        if let Flow::Stop(why) = self.check_head(analysis, &mut replay)? {
            return Ok(Flow::Stop(why));
        }
        let after_done = analysis
            .done
            .iter()
            .filter_map(|id| index_of(id))
            .max()
            .map(|i| i + 1);
        let start = match analysis.interrupted.as_deref().and_then(index_of) {
            Some(i) => i,
            None => after_done
                .or_else(|| analysis.run.from.as_deref().and_then(index_of))
                .unwrap_or(0),
        };
        self.earlier = plan.tasks[..start]
            .iter()
            .map(|t| t.id.raw.to_string())
            .collect();
        self.notice = Some(self.notice_text("planrun resumes an interrupted run"));
        if analysis.interrupted.is_some() {
            self.replay = Some(replay);
        }
        self.tasks_from(start)
    }

    /// HEAD against the journal (D14, D80): equal, or moved by the one action that was
    /// running. Where a moved HEAD is accepted, the acceptance is journaled at once (an
    /// `action-end`, or `head-seen` for a retry), so a crash after it resumes from it.
    fn check_head(&mut self, analysis: &Analysis, replay: &mut Replay) -> Result<Flow> {
        let now = self.head()?;
        if now == analysis.head {
            if let Some((at, Past::Running { kind, .. })) = analysis.running()
                && kind == "commit"
            {
                replay.past.remove(&key(at));
            }
            return Ok(Flow::Go);
        }
        let stop = Flow::Stop(format!(
            "HEAD is {} ({}), but the journal last recorded {} ({}): something else changed \
             the repository",
            now.commit.as_deref().unwrap_or("none"),
            now.reference.as_deref().unwrap_or("detached"),
            analysis.head.commit.as_deref().unwrap_or("none"),
            analysis.head.reference.as_deref().unwrap_or("detached"),
        ));
        let Some((at, Past::Running { kind, head, expect })) = analysis.running() else {
            return Ok(stop);
        };
        let Some(commit) = now.commit.as_deref() else {
            return Ok(stop);
        };
        let on_top = self.ws.git().parent(commit)?.as_deref() == head.commit.as_deref();
        if !on_top || now.reference != head.reference {
            return Ok(stop);
        }
        if kind == "commit"
            && self
                .commit_problem(head.commit.as_deref(), expect)?
                .is_none()
        {
            self.ui.status(&format!(
                "  the commit of {} was made before the interruption",
                at.task
            ));
            self.accepted(at, Outcome::Passed, &now)?;
            replay.commit_done = Some(key(at));
            return Ok(Flow::Go);
        }
        let question = Question {
            kind: "head-moved",
            at: None,
            text: vec![
                format!(
                    "HEAD moved by one commit while the {kind} at step {}, action {} of {} was \
                     running",
                    at.step + 1,
                    at.action + 1,
                    at.task
                ),
                "keep treats it as done; retry runs it again".to_owned(),
            ],
            // No `[o]`: it journals HEAD as seen (D80), which would pass this check unanswered.
            choices: vec![Choice::Keep, Choice::Retry, Choice::Diff, Choice::Quit],
            typed: false,
        };
        Ok(match self.ask(question)? {
            Answer::Choice(Choice::Keep) => {
                self.accepted(at, Outcome::Kept, &now)?;
                replay.past.insert(key(at), Past::Ended(Outcome::Kept));
                Flow::Go
            }
            Answer::Choice(Choice::Retry) => {
                self.append(Record::HeadSeen { head: now })?;
                replay.past.remove(&key(at));
                Flow::Go
            }
            Answer::Stop(why) => Flow::Stop(why),
            _ => Flow::Stop("quit".to_owned()),
        })
    }

    /// The end of the action that was running, journaled when resume accepts the HEAD it
    /// moved: a later resume finds the action ended and HEAD recorded.
    fn accepted(&mut self, at: &At, outcome: Outcome, head: &Head) -> Result<()> {
        self.append(Record::ActionEnd {
            at: at.clone(),
            outcome,
            status: None,
            errored: false,
            cwd: None,
            tail: vec!["accepted by resume: HEAD moved while it was running".to_owned()],
            head: head.clone(),
        })
    }

    /// The interrupted task's prepared steps against the disk (D14, D77): temp files removed,
    /// Done steps not written again, an Unknown step restored or accepted by the user. Returns
    /// what preflight must leave out.
    pub(crate) fn replay_steps(
        &mut self,
        index: usize,
    ) -> Result<std::result::Result<Resumed, String>> {
        let Some(replay) = self.replay.as_ref().filter(|r| r.task == index) else {
            return Ok(Ok(Resumed::default()));
        };
        let prepared = replay.prepared.clone();
        let task = &self.plan.tasks[index];
        // Any path of the task may have been changed by the interrupted run, by a step or by a
        // script: its dirt is a warning, and its manifest checks are left out. Dirt outside the
        // verified steps that a remaining file action writes over is asked about (D77).
        let mut resumed = Resumed {
            done: Vec::new(),
            own: task_paths(task),
            verified: Vec::new(),
        };
        for p in &prepared {
            for removed in remove_temps(self.ws, &p.step) {
                self.ui.status(&format!(
                    "  removed a temp file of the interrupted run: {removed}"
                ));
            }
            for w in &p.step.writes {
                if !resumed.own.contains(&w.path) {
                    resumed.own.push(w.path.clone());
                }
            }
            let found = step_state(self.ws, &p.step)?;
            let done = match found.state {
                DoneState::Done => true,
                DoneState::NotDone => false,
                DoneState::Unknown => match self.unknown_step(&p.at, &p.step, &found.paths)? {
                    Ok(done) => done,
                    Err(why) => return Ok(Err(why)),
                },
            };
            if !done {
                continue;
            }
            for w in &p.step.writes {
                if !resumed.verified.contains(&w.path) {
                    resumed.verified.push(w.path.clone());
                }
            }
            let step = &task.steps[p.at.step];
            let group = step.actions[p.at.action..]
                .iter()
                .take_while(|a| super::is_file_action(a))
                .count();
            resumed
                .done
                .extend((p.at.action..p.at.action + group).map(|a| (p.at.step, a)));
            if let Some(r) = self.replay.as_mut() {
                r.done_groups.push(key(&p.at));
            }
        }
        Ok(Ok(resumed))
    }

    /// A step that is neither done nor undone on disk (D14): restore it from its pre-images
    /// when they are all there (D57), accept it as done with a typed `yes`, or stop.
    fn unknown_step(
        &mut self,
        at: &At,
        step: &StepRecord,
        paths: &[(RepoPath, Seen)],
    ) -> Result<std::result::Result<bool, String>> {
        let can_restore = restorable(self.ws, step)?;
        let mut text = vec![format!(
            "step {} of {} is partly written (the run was interrupted while writing it):",
            at.step + 1,
            at.task
        )];
        text.extend(paths.iter().map(|(p, s)| format!("  {p}: {}", seen(*s))));
        let mut choices = Vec::new();
        if can_restore {
            choices.push(Choice::Restore);
        } else {
            text.push("its pre-images are gone (pruned by git gc, D57), or a git delete already left the index: it cannot be restored".to_owned());
        }
        choices.extend([Choice::Accept, Choice::Diff, Choice::Shell, Choice::Quit]);
        let question = Question {
            kind: "unknown",
            at: None,
            text,
            choices,
            typed: false,
        };
        match self.ask(question)? {
            Answer::Choice(Choice::Restore) => {
                let unrestored = restore_step(self.ws, step);
                if !unrestored.is_empty() {
                    let names: Vec<&str> = unrestored.iter().map(RepoPath::as_str).collect();
                    return Ok(Err(format!("could not restore {}", names.join(", "))));
                }
                self.append(Record::StepRestored { at: at.clone() })?;
                self.ui
                    .status("  restored from the pre-images; the step is written again");
                Ok(Ok(false))
            }
            Answer::Choice(Choice::Accept) => {
                let question = Question {
                    kind: "accept-done",
                    at: None,
                    text: vec![
                        "accepting the step as done overrides the disk check".to_owned(),
                        "type `yes` to accept it, or `quit`".to_owned(),
                    ],
                    choices: Vec::new(),
                    typed: true,
                };
                match self.typed_yes(question, false)? {
                    Typed::Yes => {
                        self.append(Record::StepAccepted { at: at.clone() })?;
                        Ok(Ok(true))
                    }
                    Typed::Stop(why) => Ok(Err(why)),
                    _ => Ok(Err("quit".to_owned())),
                }
            }
            Answer::Stop(why) => Ok(Err(why)),
            _ => Ok(Err("quit".to_owned())),
        }
    }

    /// The replay entry of the action at (step, action) in task `index`, taken once.
    pub(crate) fn take_past(&mut self, index: usize, at: (usize, usize)) -> Option<Past> {
        let replay = self.replay.as_mut().filter(|r| r.task == index)?;
        replay.past.remove(&at)
    }

    /// Whether the file group at (step, action) of task `index` was found done on disk.
    pub(crate) fn group_done(&self, index: usize, at: (usize, usize)) -> bool {
        self.replay
            .as_ref()
            .is_some_and(|r| r.task == index && r.done_groups.contains(&at))
    }

    /// Whether the Commit at (step, action) of task `index` was made before the interruption.
    pub(crate) fn commit_made(&self, index: usize, at: (usize, usize)) -> bool {
        self.replay
            .as_ref()
            .is_some_and(|r| r.task == index && r.commit_done == Some(at))
    }
}

fn seen(s: Seen) -> &'static str {
    match s {
        Seen::Post => "written",
        Seen::Pre => "not written",
        Seen::Neither => "neither as before nor as planned",
    }
}
