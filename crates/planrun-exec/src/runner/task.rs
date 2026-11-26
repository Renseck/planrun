//! One task: preflight, its steps, file transactions, Manuals, `Generated` checks, and what is
//! checked around its commit (D75, D78, D80).

use planrun_model::{Action, ActionKind, RepoPath, Task};

use super::gate::Gate;
use super::{Answer, Flow, Result, RunError, Runner, Typed, describe, is_file_action, task_paths};
use crate::finding::{Finding, Severity};
use crate::journal::{Outcome, PathBlob, Record};
use crate::policy::Decision;
use crate::preflight;
use crate::transaction::{self, CommitCause};
use crate::ui::{Choice, Position, Question};

/// The state of the task being run.
pub(crate) struct TaskRun {
    pub index: usize,
    pub gate: Gate,
    /// The task's `Generated` paths, once each (architecture §3), with the first action naming them.
    pub generated: Vec<(RepoPath, Position)>,
    pub generated_checked: bool,
}

impl Runner<'_> {
    /// Runs task `index`.
    pub(crate) fn task(&mut self, index: usize) -> Result<Flow> {
        let plan = self.plan;
        let task = &plan.tasks[index];
        self.ui
            .status(&format!("{}  {}", task.id, plan.text(&task.title)));
        let found = preflight::preflight(self.ws, plan, task)?;
        if let Flow::Stop(why) = self.findings(task, &found) {
            return Ok(Flow::Stop(why));
        }
        let head = self.head()?;
        self.append(Record::TaskStart {
            task: task.id.raw.to_string(),
            head,
        })?;
        let mut run = TaskRun {
            index,
            gate: Gate::default(),
            generated: generated(task, index),
            generated_checked: false,
        };
        for (si, step) in task.steps.iter().enumerate() {
            let mut ai = 0;
            while ai < step.actions.len() {
                let pos = Position {
                    task: index,
                    step: si,
                    action: ai,
                };
                let group: Vec<&Action> = step.actions[ai..]
                    .iter()
                    .take_while(|a| is_file_action(a))
                    .collect();
                let flow = if group.is_empty() {
                    ai += 1;
                    self.action(&mut run, pos, &step.actions[pos.action])?
                } else {
                    ai += group.len();
                    self.files(&mut run, pos, &group)?
                };
                if let Flow::Stop(why) = flow {
                    return Ok(Flow::Stop(why));
                }
            }
        }
        if let Flow::Stop(why) = self.check_generated(&mut run)? {
            return Ok(Flow::Stop(why));
        }
        let head = self.head()?;
        self.append(Record::TaskEnd {
            task: task.id.raw.to_string(),
            head,
        })?;
        Ok(Flow::Go)
    }

    /// Shows preflight's findings; any error stops the run.
    fn findings(&mut self, task: &Task, found: &[Finding]) -> Flow {
        for f in found {
            let line = f
                .span
                .map(|s| format!("L{} ", self.plan.source.line_col(s.start).line))
                .unwrap_or_default();
            let severity = match f.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };
            self.ui
                .status(&format!("  {severity}: {line}{}: {}", f.code, f.message));
        }
        if found.iter().any(Finding::is_error) {
            return Flow::Stop(format!("{}: preflight found errors", task.id));
        }
        Flow::Go
    }

    /// One action that is not a file action.
    fn action(&mut self, run: &mut TaskRun, pos: Position, action: &Action) -> Result<Flow> {
        let task = &self.plan.tasks[pos.task];
        match &action.kind {
            ActionKind::Note { text } => {
                let text = self.plan.text(text);
                self.ui.status(&format!("  note: {}", first_line(text)));
                Ok(Flow::Go)
            }
            ActionKind::Generated { path } => {
                self.ui
                    .status(&format!("  generated {path}: checked before the commit"));
                Ok(Flow::Go)
            }
            ActionKind::Manual { .. } => self.manual(task, pos, action),
            ActionKind::RunScript(script) => self.script(run, pos, action, script, false),
            ActionKind::Commit(script) => {
                if let Flow::Stop(why) = self.before_commit(run, pos)? {
                    return Ok(Flow::Stop(why));
                }
                self.script(run, pos, action, script, true)
            }
            _ => Ok(Flow::Go),
        }
    }

    /// A step's consecutive file actions as one transaction (D53, D78). An unscoped one asks
    /// first (D50); a skipped one is left out and closes the gate.
    fn files(&mut self, run: &mut TaskRun, pos: Position, group: &[&Action]) -> Result<Flow> {
        let plan = self.plan;
        let task = &plan.tasks[pos.task];
        let mut chosen: Vec<&Action> = Vec::new();
        for (k, action) in group.iter().enumerate() {
            let here = Position {
                action: pos.action + k,
                ..pos
            };
            let decision = self
                .config
                .policy
                .decide(task, action, &plan.source, &self.classifier);
            if decision != Decision::AskUnscoped {
                chosen.push(action);
                continue;
            }
            let question = Question {
                kind: "unscoped",
                at: Some(here),
                text: vec![format!(
                    "L{} {} ({:?}): not in the task's files (D50)",
                    self.line(action),
                    describe(action),
                    action.path_origin
                )],
                choices: vec![
                    Choice::Run,
                    Choice::Skip,
                    Choice::View,
                    Choice::Diff,
                    Choice::Shell,
                    Choice::Quit,
                ],
                typed: false,
            };
            match self.ask(question)? {
                Answer::Choice(Choice::Run) => chosen.push(action),
                Answer::Choice(Choice::Skip) => run.gate.close(format!(
                    "L{}: {} was skipped by you",
                    self.line(action),
                    describe(action)
                )),
                Answer::Stop(why) => return Ok(Flow::Stop(why)),
                _ => return Ok(Flow::Stop("quit".to_owned())),
            }
        }
        if chosen.is_empty() {
            return Ok(Flow::Go);
        }
        self.write_group(pos, &chosen)
    }

    /// Prepares and commits `actions`; asks when the disk changed since prepare (D78).
    pub(crate) fn write_group(&mut self, pos: Position, actions: &[&Action]) -> Result<Flow> {
        let at = self.at(pos);
        loop {
            let step =
                match transaction::prepare(self.ws, &self.plan.source, &self.handlers, actions) {
                    Ok(step) => step,
                    Err(f) => {
                        let line = f
                            .span
                            .map(|s| self.plan.source.line_col(s.start).line)
                            .unwrap_or_default();
                        self.ui
                            .status(&format!("  FAILED L{line} {}: {}", f.code, f.message));
                        return Ok(Flow::Stop(format!("{}: {}", f.code, f.message)));
                    }
                };
            let result = transaction::commit(self.ws, &step, &mut self.journal.ahead(&at));
            let err = match result {
                Ok(()) => {
                    self.append(Record::StepWritten { at })?;
                    for action in actions {
                        let line = self.line(action);
                        self.ui
                            .status(&format!("  L{line:<5} {}: done", describe(action)));
                    }
                    return Ok(Flow::Go);
                }
                Err(err) => err,
            };
            match err.cause {
                CommitCause::Changed(paths) => {
                    let names: Vec<&str> = paths.iter().map(RepoPath::as_str).collect();
                    let question = Question {
                        kind: "changed",
                        at: Some(pos),
                        text: vec![
                            format!("changed since the step was prepared: {}", names.join(", ")),
                            "something else is writing the tree; nothing was written".to_owned(),
                        ],
                        choices: vec![
                            Choice::PrepareAgain,
                            Choice::Diff,
                            Choice::Shell,
                            Choice::Quit,
                        ],
                        typed: false,
                    };
                    match self.ask(question)? {
                        Answer::Choice(Choice::PrepareAgain) => continue,
                        Answer::Stop(why) => return Ok(Flow::Stop(why)),
                        _ => return Ok(Flow::Stop("quit".to_owned())),
                    }
                }
                CommitCause::Record(e) => return Err(RunError::Journal(e)),
                CommitCause::Workspace(e) => {
                    let error = e.to_string();
                    self.append(Record::StepFailed {
                        at,
                        error: error.clone(),
                        unrestored: err.unrestored.clone(),
                    })?;
                    self.ui.status(&format!("  FAILED: {error}"));
                    for path in &err.unrestored {
                        self.ui.status(&format!("  not restored: {path}"));
                    }
                    return Ok(Flow::Stop(error));
                }
            }
        }
    }

    /// A Manual waits until acknowledged (D21), whatever `--auto` says. When HEAD moved (a
    /// commit by hand, D33), the task's paths that are still dirty are reported (D80).
    pub(crate) fn manual(&mut self, task: &Task, pos: Position, action: &Action) -> Result<Flow> {
        let ActionKind::Manual { reason, text } = &action.kind else {
            return Ok(Flow::Go);
        };
        let head = self.head()?;
        self.append(Record::ActionStart {
            at: self.at(pos),
            kind: "manual".to_owned(),
            decision: format!("{:?}", Decision::Wait),
            head: head.clone(),
            expect: Vec::new(),
        })?;
        let mut lines = vec![format!(
            "L{} manual ({reason:?}): do this by hand, then continue",
            self.line(action)
        )];
        lines.extend(self.plan.text(text).lines().map(|l| format!("  {l}")));
        let question = Question {
            kind: "manual",
            at: Some(pos),
            text: lines,
            choices: vec![
                Choice::Continue,
                Choice::View,
                Choice::Diff,
                Choice::Shell,
                Choice::Quit,
            ],
            typed: false,
        };
        match self.ask(question)? {
            Answer::Choice(Choice::Continue) => {}
            Answer::Stop(why) => return Ok(Flow::Stop(why)),
            _ => return Ok(Flow::Stop("quit".to_owned())),
        }
        let after = self.head()?;
        self.append(Record::ActionEnd {
            at: self.at(pos),
            outcome: Outcome::Acknowledged,
            status: None,
            errored: false,
            cwd: None,
            tail: Vec::new(),
            head: after.clone(),
        })?;
        if after.commit != head.commit {
            self.still_dirty(task)?;
        }
        Ok(Flow::Go)
    }

    /// After a commit by hand: the task's paths that still have changes.
    fn still_dirty(&mut self, task: &Task) -> Result<()> {
        let paths = task_paths(task);
        let refs: Vec<&RepoPath> = paths.iter().collect();
        let dirty = self.ws.git().status(Some(&refs))?;
        for entry in dirty {
            self.ui.status(&format!(
                "  warning: HEAD moved, but `{}` still has changes ({})",
                entry.path, entry.code
            ));
        }
        Ok(())
    }

    /// The `Generated` paths, checked once each after the task's last non-commit action (D27,
    /// architecture §3). A missing one closes the gate.
    pub(crate) fn check_generated(&mut self, run: &mut TaskRun) -> Result<Flow> {
        if run.generated_checked {
            return Ok(Flow::Go);
        }
        run.generated_checked = true;
        for (path, pos) in run.generated.clone() {
            let head = self.head()?;
            self.append(Record::ActionStart {
                at: self.at(pos),
                kind: "generated".to_owned(),
                decision: format!("{:?}", Decision::Auto),
                head: head.clone(),
                expect: Vec::new(),
            })?;
            let present = self.ws.read(&path)?.is_some();
            let outcome = if present {
                Outcome::Present
            } else {
                Outcome::Missing
            };
            self.append(Record::ActionEnd {
                at: self.at(pos),
                outcome,
                status: None,
                errored: false,
                cwd: None,
                tail: Vec::new(),
                head,
            })?;
            if present {
                self.ui.status(&format!("  generated {path}: present"));
                continue;
            }
            self.ui.status(&format!(
                "  generated {path}: MISSING (the plan says a step makes it)"
            ));
            run.gate.close(format!(
                "`{path}` should have been generated, and is missing"
            ));
        }
        Ok(Flow::Go)
    }

    /// What comes before a Commit (D75, D80): the `Generated` checks, stale Gates run again,
    /// dirt outside the task's paths asked about, and a closed gate opened only by a typed
    /// `yes` naming every reason.
    fn before_commit(&mut self, run: &mut TaskRun, pos: Position) -> Result<Flow> {
        if let Flow::Stop(why) = self.check_generated(run)? {
            return Ok(Flow::Stop(why));
        }
        if let Flow::Stop(why) = self.rerun_stale_gates(run)? {
            return Ok(Flow::Stop(why));
        }
        if let Flow::Stop(why) = self.outside_dirt(pos)? {
            return Ok(Flow::Stop(why));
        }
        let reasons = run.gate.reasons(&self.sessions);
        if reasons.is_empty() {
            return Ok(Flow::Go);
        }
        let task = &self.plan.tasks[pos.task];
        let mut text = vec![format!("the commit gate of {} is closed:", task.id)];
        text.extend(reasons.iter().map(|r| format!("  {r}")));
        text.push("type `yes` to commit anyway, or `quit`".to_owned());
        let question = Question {
            kind: "gate",
            at: Some(pos),
            text,
            choices: Vec::new(),
            typed: true,
        };
        match self.typed_yes(question, false)? {
            Typed::Yes => {
                self.append(Record::GateOverride {
                    task: task.id.raw.to_string(),
                    reasons,
                })?;
                Ok(Flow::Go)
            }
            Typed::Stop(why) => Ok(Flow::Stop(why)),
            Typed::Skip | Typed::Quit => Ok(Flow::Stop("the commit gate is closed".to_owned())),
        }
    }

    /// Each Gate whose verdict comes from a session that has ended runs again (D75).
    fn rerun_stale_gates(&mut self, run: &mut TaskRun) -> Result<Flow> {
        let task = &self.plan.tasks[run.index];
        for (si, ai) in run.gate.stale_gates(&self.sessions) {
            let action = &task.steps[si].actions[ai];
            let ActionKind::RunScript(script) = &action.kind else {
                continue;
            };
            self.ui.status(&format!(
                "  L{}: the gate ran in a session that has ended; it runs again",
                self.line(action)
            ));
            let pos = Position {
                task: run.index,
                step: si,
                action: ai,
            };
            if let Flow::Stop(why) = self.script(run, pos, action, script, false)? {
                return Ok(Flow::Stop(why));
            }
        }
        Ok(Flow::Go)
    }

    /// Dirt outside the task's paths before a commit (D80): listed and asked about.
    fn outside_dirt(&mut self, pos: Position) -> Result<Flow> {
        let task = &self.plan.tasks[pos.task];
        let outside: Vec<String> = self
            .ws
            .git()
            .status(None)?
            .into_iter()
            .filter(|e| !self.in_task(task, &e.path))
            .map(|e| format!("  {} {}", e.code, e.path))
            .collect();
        if outside.is_empty() {
            return Ok(Flow::Go);
        }
        let mut text = vec![format!(
            "{} path(s) outside the task's files have changes; a commit that stages everything \
             would take them:",
            outside.len()
        )];
        text.extend(outside);
        let question = Question {
            kind: "dirt",
            at: Some(pos),
            text,
            choices: vec![Choice::Continue, Choice::Diff, Choice::Shell, Choice::Quit],
            typed: false,
        };
        Ok(match self.ask(question)? {
            Answer::Choice(Choice::Continue) => Flow::Go,
            Answer::Stop(why) => Flow::Stop(why),
            _ => Flow::Stop("quit".to_owned()),
        })
    }

    /// Whether `path` (as git prints it) is one of the task's paths or matches a pattern entry.
    pub(crate) fn in_task(&self, task: &Task, path: &str) -> bool {
        let fold = |p: &str| {
            if self.ws.ignorecase() {
                p.to_lowercase()
            } else {
                p.to_owned()
            }
        };
        let wanted = fold(path);
        if task_paths(task).iter().any(|p| fold(p.as_str()) == wanted) {
            return true;
        }
        let Ok(path) = RepoPath::new(path) else {
            return false;
        };
        task.manifest.entries.iter().any(|e| e.path.covers(&path))
    }

    /// The task's paths as they are on disk now (git's view), which the commit must hold.
    pub(crate) fn commit_expect(&self, task: &Task) -> Result<Vec<PathBlob>> {
        let paths = task_paths(task);
        let mut present = Vec::new();
        let mut expect = Vec::new();
        for path in &paths {
            let resolved = self.ws.resolve(path)?;
            if resolved.exists() && !resolved.is_dir {
                present.push(path);
            }
        }
        let mut ids = self.ws.git().hash_paths(&present, true)?.into_iter();
        for path in &paths {
            let blob = if present.contains(&path) {
                ids.next()
            } else {
                None
            };
            expect.push(PathBlob {
                path: path.clone(),
                blob,
            });
        }
        Ok(expect)
    }

    /// Why a commit script did not commit what it should (D79): HEAD did not move, moved by
    /// other than one commit on `before`, or its tree does not hold `expect`.
    pub(crate) fn commit_problem(
        &self,
        before: Option<&str>,
        expect: &[PathBlob],
    ) -> Result<Option<String>> {
        let git = self.ws.git();
        let head = git.head()?;
        let Some(now) = head.commit.as_deref() else {
            return Ok(Some("there is no commit at HEAD".to_owned()));
        };
        if Some(now) == before {
            return Ok(Some(
                "the commit script ran, but HEAD did not move".to_owned(),
            ));
        }
        if git.parent(now)?.as_deref() != before {
            return Ok(Some(
                "HEAD moved, but not by one commit on top of the one before".to_owned(),
            ));
        }
        let paths: Vec<&RepoPath> = expect.iter().map(|e| &e.path).collect();
        let held = git.tree_blobs(now, &paths)?;
        let wrong: Vec<&str> = expect
            .iter()
            .zip(&held)
            .filter(|(e, h)| e.blob != **h)
            .map(|(e, _)| e.path.as_str())
            .collect();
        if wrong.is_empty() {
            return Ok(None);
        }
        Ok(Some(format!(
            "the commit does not hold the task's files as they were: {}",
            wrong.join(", ")
        )))
    }

    /// After a commit: paths it changed outside the task's, reported loudly (D80).
    pub(crate) fn commit_outside(&mut self, task: &Task, before: Option<&str>) -> Result<()> {
        let head = self.head()?;
        let (Some(before), Some(now)) = (before, head.commit.as_deref()) else {
            return Ok(());
        };
        let changed = self.ws.git().changed_paths(before, now)?;
        let outside: Vec<String> = changed
            .into_iter()
            .filter(|p| !self.in_task(task, p))
            .collect();
        if !outside.is_empty() {
            self.ui.status(&format!(
                "  WARNING: the commit also changed paths outside the task's files: {}",
                outside.join(", ")
            ));
        }
        Ok(())
    }
}

/// The task's `Generated` paths, once each, with the first action naming each.
fn generated(task: &Task, index: usize) -> Vec<(RepoPath, Position)> {
    let mut found: Vec<(RepoPath, Position)> = Vec::new();
    for (si, step) in task.steps.iter().enumerate() {
        for (ai, action) in step.actions.iter().enumerate() {
            let ActionKind::Generated { path } = &action.kind else {
                continue;
            };
            if found.iter().any(|(p, _)| p == path) {
                continue;
            }
            let pos = Position {
                task: index,
                step: si,
                action: ai,
            };
            found.push((path.clone(), pos));
        }
    }
    found
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}
