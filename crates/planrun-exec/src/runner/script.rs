//! One script or commit: its decision and prompt (D69, D74), the questions before its block
//! (the session notice, a working directory outside the repository; D81), the run, the
//! matcher's verdict and what each verdict asks (D66, D75), and the commit's own checks.

use planrun_model::{Action, Script, ScriptIntent};

use super::gate::Attempt;
use super::resume::Past;
use super::sessions::{NotRun, name};
use super::task::TaskRun;
use super::{Answer, Flow, Result, RunError, Runner, Typed, canonical, describe};
use crate::expect::{self, Mismatch, Reason, Verdict};
use crate::journal::{Outcome, PathBlob, Record};
use crate::policy::{Decision, PolicyClass, Skip};
use crate::shell::{RunStatus, ScriptRun};
use crate::ui::{Choice, Position, Question};

/// What the matcher, and for a commit HEAD, concluded.
enum Judged {
    Pass,
    Confirm(Vec<String>),
    /// What does not hold; `dead` when the session ended, timed out or broke.
    Mismatch {
        lines: Vec<String>,
        dead: bool,
    },
}

/// How many output lines the journal keeps.
const TAIL: usize = 20;

impl Runner<'_> {
    /// Runs the script (or commit) at `pos` per its decision.
    pub(crate) fn script(
        &mut self,
        run: &mut TaskRun,
        pos: Position,
        action: &Action,
        script: &Script,
        commit: bool,
    ) -> Result<Flow> {
        let plan = self.plan;
        let task = &plan.tasks[pos.task];
        let decision = self
            .config
            .policy
            .decide(task, action, &plan.source, &self.classifier);
        let line = self.line(action);
        match self.replayed(pos, action, script, commit)? {
            Replayed::Again => {}
            Replayed::Keep => {
                return self.skipped(run, pos, action, script, decision, Outcome::Kept);
            }
            Replayed::Done => return Ok(Flow::Go),
            Replayed::Stop(why) => return Ok(Flow::Stop(why)),
        }
        if let Decision::Skip(Skip::ExpectedFailure) = decision {
            self.ui.status(&format!(
                "  L{line:<5} {}: skipped: expected failure (D67)",
                describe(action)
            ));
            return self.skipped(run, pos, action, script, decision, Outcome::SkippedPolicy);
        }
        match self.permission(pos, action, script, decision)? {
            Permit::Run => {}
            Permit::Skip => {
                return self.skipped(run, pos, action, script, decision, Outcome::SkippedUser);
            }
            Permit::Stop(why) => return Ok(Flow::Stop(why)),
        }
        loop {
            if let Flow::Stop(why) = self.before_block(script, pos)? {
                return Ok(Flow::Stop(why));
            }
            let head = self.head()?;
            let expect = if commit {
                self.commit_expect(task)?
            } else {
                Vec::new()
            };
            self.append(Record::ActionStart {
                at: self.at(pos),
                kind: if commit { "commit" } else { "script" }.to_owned(),
                decision: format!("{decision:?}"),
                head: head.clone(),
                expect: expect.clone(),
            })?;
            self.ui
                .status(&format!("  L{line:<5} {}: running", describe(action)));
            let ran = self.block(script)?;
            let before = head.commit.as_deref();
            let judged = self.judge(script, &ran, commit, before, &expect)?;
            let outcome = match self.verdict(pos, script, &ran, judged)? {
                Resolved::Outcome(outcome) => outcome,
                Resolved::Retry => {
                    self.ended(pos, Outcome::Failed, &ran)?;
                    continue;
                }
                Resolved::Stop(why) => {
                    self.ended(pos, Outcome::Failed, &ran)?;
                    return Ok(Flow::Stop(why));
                }
            };
            self.ended(pos, outcome, &ran)?;
            let attempt = Attempt {
                line,
                outcome,
                gate: script.intent == ScriptIntent::Gate,
                shell: script.shell,
                generation: self.sessions.generation(script.shell),
            };
            run.gate.record((pos.step, pos.action), attempt);
            if commit {
                self.commit_outside(task, before)?;
            }
            return Ok(Flow::Go);
        }
    }

    /// What resume does with a script that ran, or was running, before the interruption
    /// (D77): a made Commit is done; one that was running asks whatever its class; one that
    /// ended runs again, Modify and Destructive ones asking first.
    fn replayed(
        &mut self,
        pos: Position,
        action: &Action,
        script: &Script,
        commit: bool,
    ) -> Result<Replayed> {
        let line = self.line(action);
        if commit && self.commit_made(pos.task, (pos.step, pos.action)) {
            self.ui.status(&format!(
                "  L{line:<5} {}: made before the interruption",
                describe(action)
            ));
            return Ok(Replayed::Done);
        }
        let Some(past) = self.take_past(pos.task, (pos.step, pos.action)) else {
            return Ok(Replayed::Again);
        };
        let ended_ok = matches!(
            past,
            Past::Ended(Outcome::Passed | Outcome::Confirmed | Outcome::Accepted)
        );
        if commit && ended_ok {
            self.ui.status(&format!(
                "  L{line:<5} {}: made before the interruption",
                describe(action)
            ));
            return Ok(Replayed::Done);
        }
        let why = match past {
            Past::Ended(Outcome::Kept) => return Ok(Replayed::Keep),
            Past::Ended(Outcome::SkippedPolicy) => return Ok(Replayed::Again),
            Past::Running { .. } => {
                "was running when planrun was interrupted; it may have done part of its work"
            }
            Past::Ended(_) => {
                let body = self.plan.source.resolve(&script.body);
                let class = self.classifier.explain(script.shell, body).class;
                if class < PolicyClass::Modify {
                    self.ui.status(&format!(
                        "  L{line:<5} {}: ran before the interruption; it runs again",
                        describe(action)
                    ));
                    return Ok(Replayed::Again);
                }
                "ran before the interruption"
            }
        };
        let question = Question {
            kind: "again",
            at: Some(pos),
            text: vec![
                format!("L{line} {} {why}", describe(action)),
                "retry runs it again (its decision applies); keep leaves its result as it is"
                    .to_owned(),
            ],
            choices: vec![
                Choice::Retry,
                Choice::Keep,
                Choice::View,
                Choice::Diff,
                Choice::Shell,
                Choice::Quit,
            ],
            typed: false,
        };
        Ok(match self.ask(question)? {
            Answer::Choice(Choice::Retry) => Replayed::Again,
            Answer::Choice(Choice::Keep) => Replayed::Keep,
            Answer::Stop(why) => Replayed::Stop(why),
            _ => Replayed::Stop("quit".to_owned()),
        })
    }

    /// A skipped script, journaled as started and ended, and recorded for the gate.
    fn skipped(
        &mut self,
        run: &mut TaskRun,
        pos: Position,
        action: &Action,
        script: &Script,
        decision: Decision,
        outcome: Outcome,
    ) -> Result<Flow> {
        let head = self.head()?;
        self.append(Record::ActionStart {
            at: self.at(pos),
            kind: "script".to_owned(),
            decision: format!("{decision:?}"),
            head: head.clone(),
            expect: Vec::new(),
        })?;
        self.append(Record::ActionEnd {
            at: self.at(pos),
            outcome,
            status: None,
            errored: false,
            cwd: None,
            tail: Vec::new(),
            head,
        })?;
        let attempt = Attempt {
            line: self.line(action),
            outcome,
            gate: script.intent == ScriptIntent::Gate,
            shell: script.shell,
            generation: if outcome == Outcome::Kept {
                u64::MAX
            } else {
                self.sessions.generation(script.shell)
            },
        };
        run.gate.record((pos.step, pos.action), attempt);
        Ok(Flow::Go)
    }

    /// The prompt the decision calls for (D69): none for Auto, a choice for Ask, the whole
    /// script and a typed `yes` for Destructive.
    fn permission(
        &mut self,
        pos: Position,
        action: &Action,
        script: &Script,
        decision: Decision,
    ) -> Result<Permit> {
        let plan = self.plan;
        let body = plan.source.resolve(&script.body);
        let head = format!("L{} {}", self.line(action), describe(action));
        match decision {
            Decision::Auto => {
                let class = self.classifier.explain(script.shell, body).class;
                self.ui
                    .status(&format!("  {head}: runs ({class:?}, --auto)"));
                Ok(Permit::Run)
            }
            Decision::TypedYes => {
                let found = self.classifier.explain(script.shell, body);
                let mut text = vec![format!("{head}: Destructive, the whole script:")];
                text.extend(body.lines().map(|l| format!("  | {l}")));
                text.extend(found.reasons.iter().map(|r| format!("  because: {r}")));
                text.push("type `yes` to run it, `skip` to skip it, `quit` to stop".to_owned());
                let question = Question {
                    kind: "typed-yes",
                    at: Some(pos),
                    text,
                    choices: Vec::new(),
                    typed: true,
                };
                Ok(match self.typed_yes(question, true)? {
                    Typed::Yes => Permit::Run,
                    Typed::Skip => Permit::Skip,
                    Typed::Quit => Permit::Stop("quit".to_owned()),
                    Typed::Stop(why) => Permit::Stop(why),
                })
            }
            _ => {
                let class = match decision {
                    Decision::Ask(class) => format!("{class:?}"),
                    other => format!("{other:?}"),
                };
                let mut text = vec![format!("{head} ({class})")];
                text.extend(body.lines().take(6).map(|l| format!("  | {l}")));
                let question = Question {
                    kind: "run",
                    at: Some(pos),
                    text,
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
                Ok(match self.ask(question)? {
                    Answer::Choice(Choice::Run) => Permit::Run,
                    Answer::Choice(Choice::Skip) => Permit::Skip,
                    Answer::Stop(why) => Permit::Stop(why),
                    _ => Permit::Stop("quit".to_owned()),
                })
            }
        }
    }

    /// The questions before a block (D81): the session notice, and while the session's working
    /// directory is outside the repository, whether to run there. Neither changes the
    /// block's decision.
    fn before_block(&mut self, script: &Script, pos: Position) -> Result<Flow> {
        if let Some(restart) = self.sessions.starts(script.shell) {
            if restart && self.notice.is_none() {
                let why = format!("the {} session ended", name(script.shell));
                self.notice = Some(self.notice_text(&why));
            }
            let reason = if restart { "restarted" } else { "started" };
            if let Flow::Stop(why) = self.acknowledge_notice(Some(pos))? {
                return Ok(Flow::Stop(why));
            }
            self.append(Record::Session {
                shell: name(script.shell).to_owned(),
                reason: reason.to_owned(),
            })?;
            return Ok(Flow::Go);
        }
        let Some(cwd) = self.sessions.cwd(script.shell).map(|p| p.to_path_buf()) else {
            return Ok(Flow::Go);
        };
        let root = self.ws.root();
        if canonical(&cwd).is_some_and(|c| c.starts_with(root)) {
            return Ok(Flow::Go);
        }
        let question = Question {
            kind: "outside",
            at: Some(pos),
            text: vec![format!(
                "the {} session's working directory is {}, outside the repository; the next \
                 block runs there",
                name(script.shell),
                cwd.display()
            )],
            choices: vec![Choice::Continue, Choice::View, Choice::Shell, Choice::Quit],
            typed: false,
        };
        Ok(match self.ask(question)? {
            Answer::Choice(Choice::Continue) => Flow::Go,
            Answer::Stop(why) => Flow::Stop(why),
            _ => Flow::Stop("quit".to_owned()),
        })
    }

    /// Runs the block, streaming its output. A block the session refused is a run whose
    /// session ended.
    fn block(&mut self, script: &Script) -> Result<ScriptRun> {
        let plan = self.plan;
        let body = plan.source.resolve(&script.body);
        let ui = &mut *self.ui;
        let result = self
            .sessions
            .run(script.shell, body, &mut |line| ui.output(line));
        match result {
            Ok(run) => Ok(run),
            Err(NotRun::Start(e)) => Err(RunError::Shell(e)),
            Err(NotRun::Refused(e)) => {
                self.ui.status(&format!("  the block did not run: {e}"));
                Ok(ScriptRun {
                    status: RunStatus::SessionEnded(None),
                    errored: false,
                    lines: Vec::new(),
                    cwd: None,
                })
            }
        }
    }

    /// The matcher's verdict (D66), and for a commit whether HEAD moved as it should (D79).
    fn judge(
        &self,
        script: &Script,
        ran: &ScriptRun,
        commit: bool,
        before: Option<&str>,
        expect: &[PathBlob],
    ) -> Result<Judged> {
        let dead = matches!(
            ran.status,
            RunStatus::SessionEnded(_) | RunStatus::TimedOut | RunStatus::Broken(_)
        );
        let verdict = expect::check(script.expect.as_ref(), ran, &self.plan.source);
        let problem = if commit && !dead {
            self.commit_problem(before, expect)?
        } else {
            None
        };
        Ok(match (verdict, problem) {
            (Verdict::Mismatch(ms), problem) => Judged::Mismatch {
                lines: ms.iter().map(mismatch_line).chain(problem).collect(),
                dead,
            },
            (_, Some(problem)) => Judged::Mismatch {
                lines: vec![problem],
                dead,
            },
            (Verdict::Pass, None) => Judged::Pass,
            (Verdict::Confirm(c), None) => {
                let mut lines: Vec<String> = c
                    .reasons
                    .iter()
                    .map(|r| self.reason_line(script, *r))
                    .collect();
                lines.push(format!("errored: {}", c.errored));
                Judged::Confirm(lines)
            }
        })
    }

    fn reason_line(&self, script: &Script, reason: Reason) -> String {
        let expect = script.expect.as_ref();
        let text = |t: Option<&planrun_model::TextRef>| {
            t.map(|t| self.plan.source.resolve(t).trim().to_owned())
                .unwrap_or_default()
        };
        match reason {
            Reason::Prose => format!(
                "the plan expects (prose): {}",
                text(expect.and_then(|e| e.prose.as_ref()))
            ),
            Reason::Comments => format!(
                "the script's comments expect: {}",
                text(expect.and_then(|e| e.comments.as_ref()))
            ),
            Reason::UncheckedExit => {
                "the plan states no exit status, and the block did not exit 0".to_owned()
            }
            Reason::Errored => "pwsh recorded an error ($Error[0] changed)".to_owned(),
        }
    }

    /// What a verdict asks (D66, D75): a confirmation for prose; for a mismatch accept, retry,
    /// shell or abort, and only retry, shell or abort when the session is dead.
    fn verdict(
        &mut self,
        pos: Position,
        script: &Script,
        ran: &ScriptRun,
        judged: Judged,
    ) -> Result<Resolved> {
        match judged {
            Judged::Pass => {
                self.ui.status("    passed");
                Ok(Resolved::Outcome(Outcome::Passed))
            }
            Judged::Confirm(mut text) => {
                text.insert(0, "confirm the result:".to_owned());
                let question = Question {
                    kind: "confirm",
                    at: Some(pos),
                    text,
                    choices: vec![
                        Choice::Yes,
                        Choice::No,
                        Choice::View,
                        Choice::Diff,
                        Choice::Shell,
                        Choice::Quit,
                    ],
                    typed: false,
                };
                Ok(match self.ask(question)? {
                    Answer::Choice(Choice::Yes) => Resolved::Outcome(Outcome::Confirmed),
                    Answer::Choice(Choice::No) => Resolved::Outcome(Outcome::Refused),
                    Answer::Stop(why) => Resolved::Stop(why),
                    _ => Resolved::Stop("quit".to_owned()),
                })
            }
            Judged::Mismatch { lines, dead } => {
                let mut text = vec![format!("MISMATCH ({:?}):", ran.status)];
                text.extend(lines.into_iter().map(|l| format!("  {l}")));
                let mut choices = if dead {
                    text.push(format!(
                        "the {} session is gone; retry starts a new one",
                        name(script.shell)
                    ));
                    vec![Choice::Retry]
                } else {
                    vec![Choice::Accept, Choice::Retry]
                };
                choices.extend([Choice::View, Choice::Diff, Choice::Shell, Choice::Quit]);
                let question = Question {
                    kind: "mismatch",
                    at: Some(pos),
                    text,
                    choices,
                    typed: false,
                };
                match self.ask(question)? {
                    Answer::Choice(Choice::Accept) => Ok(Resolved::Outcome(Outcome::Accepted)),
                    Answer::Choice(Choice::Retry) => {
                        if dead || !self.sessions.alive(script.shell) {
                            self.sessions.restart(script.shell);
                        }
                        Ok(Resolved::Retry)
                    }
                    Answer::Stop(why) => Ok(Resolved::Stop(why)),
                    _ => Ok(Resolved::Stop("quit".to_owned())),
                }
            }
        }
    }

    /// `action-end` for the attempt at `pos`, with HEAD after it (D80).
    fn ended(&mut self, pos: Position, outcome: Outcome, ran: &ScriptRun) -> Result<()> {
        let head = self.head()?;
        let start = ran.lines.len().saturating_sub(TAIL);
        self.append(Record::ActionEnd {
            at: self.at(pos),
            outcome,
            status: Some(format!("{:?}", ran.status)),
            errored: ran.errored,
            cwd: ran.cwd.as_ref().map(|p| p.display().to_string()),
            tail: ran.lines[start..].iter().map(|l| l.text.clone()).collect(),
            head,
        })
    }
}

/// What resume does with a script.
enum Replayed {
    Again,
    Keep,
    Done,
    Stop(String),
}

/// What the run prompt came to.
enum Permit {
    Run,
    Skip,
    Stop(String),
}

/// What a verdict came to.
enum Resolved {
    Outcome(Outcome),
    Retry,
    Stop(String),
}

fn mismatch_line(m: &Mismatch) -> String {
    match m {
        Mismatch::Exit { expected, got } => match expected {
            Some(e) => format!("exit: expected {e:?}, got {got:?}"),
            None => format!("the block ended as {got:?}"),
        },
        Mismatch::Line { number, line } => format!("expected line {number} not found: {line}"),
    }
}
