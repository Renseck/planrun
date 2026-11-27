//! The decision dock (docs/tui-design.md §6.D, decisions D85, D86): a question opens over the
//! console's region and leaves the navigator and CHANGES visible. One layout per
//! [`QuestionKind`], matched exhaustively, so the compiler finds a question the TUI does not
//! show. The answers are `Question.choices` and `Question.typed`, every one the runner offers and
//! no other; a typed question takes the whole word and enter, never a single key (D74).

use std::sync::mpsc::Sender;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use planrun_exec::done::Seen;
use planrun_exec::expect::Mismatch;
use planrun_exec::journal::At;
use planrun_exec::shell::RunStatus;
use planrun_exec::ui::{Choice, Question, QuestionKind, Reply};
use planrun_model::{Action, ActionKind, Plan, Shell};

use crate::draw::{Pen, fill};
use crate::glyph::{Glyph, Glyphs};
use crate::inspect::class_name;
use crate::keys::Key;
use crate::text;
use crate::theme::{Token, bold, fg, on};

/// One line of the dock's body: styled pieces.
type Spans = Vec<(String, Style)>;

/// An open question.
pub struct Dock {
    /// The question.
    pub question: Question,
    reply: Option<Sender<Reply>>,
    /// The typed answer so far.
    pub input: String,
    /// A typed word that is not one of the answers was entered.
    pub refused: Option<String>,
    /// The first body line shown.
    pub top: usize,
    /// The question is about a Commit: `q` needs pressing twice (the P5→P6 handover's item 8:
    /// quitting after the commit was made loses it for resume).
    pub guard_quit: bool,
    /// `q` was pressed once.
    pub armed: bool,
}

impl Dock {
    /// The dock for `question`, answered on `reply`.
    pub fn new(question: Question, reply: Sender<Reply>) -> Dock {
        Dock {
            question,
            reply: Some(reply),
            input: String::new(),
            refused: None,
            top: 0,
            guard_quit: false,
            armed: false,
        }
    }

    /// Sends `reply` to the runner; the dock is done.
    pub fn answer(&mut self, reply: Reply) {
        if let Some(tx) = self.reply.take() {
            let _ = tx.send(reply);
        }
    }

    /// Whether it was answered.
    pub fn answered(&self) -> bool {
        self.reply.is_none()
    }

    /// A key: the reply when it answers the question. A typed question gets the word on enter;
    /// a word it does not accept is shown as refused and nothing is sent (D74: it would ask
    /// again).
    pub fn key(&mut self, key: Key) -> Option<Reply> {
        if let Some(words) = self.question.typed {
            match key {
                Key::Char(c) => {
                    self.input.push(c);
                    self.refused = None;
                }
                Key::Backspace => {
                    self.input.pop();
                }
                Key::Enter => {
                    let word = std::mem::take(&mut self.input);
                    if words.contains(&word.as_str()) {
                        return Some(Reply::Typed(word));
                    }
                    self.refused = Some(word);
                }
                Key::Down => self.top += 1,
                Key::Up => self.top = self.top.saturating_sub(1),
                _ => {}
            }
            return None;
        }
        let pressed = match key {
            Key::Enter => "",
            Key::Char('q') if self.guard_quit && !self.armed && self.offers(Choice::Quit) => {
                self.armed = true;
                return None;
            }
            Key::Char(c) => return self.choice(&c.to_string()),
            Key::Down => {
                self.top += 1;
                return None;
            }
            Key::Up => {
                self.top = self.top.saturating_sub(1);
                return None;
            }
            _ => return None,
        };
        self.choice(pressed)
    }

    fn offers(&self, choice: Choice) -> bool {
        self.question.choices.contains(&choice)
    }

    fn choice(&self, key: &str) -> Option<Reply> {
        self.question
            .choices
            .iter()
            .find(|c| c.key() == key)
            .map(|c| Reply::Choice(*c))
    }

    /// The keys the footer shows: the answers, then `tab`.
    pub fn footer(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = match self.question.typed {
            Some(words) => words
                .iter()
                .map(|w| {
                    (
                        format!("{w}{}", Glyphs::unicode().get(Glyph::Enter)),
                        word_meaning(&self.question.kind, w).to_owned(),
                    )
                })
                .collect(),
            None => self
                .question
                .choices
                .iter()
                .map(|c| (key_name(*c).to_owned(), c.label().to_owned()))
                .collect(),
        };
        out.push(("tab".into(), "look around".into()));
        out
    }

    /// The band's title: glyph, words, colour.
    fn title(&self) -> (Glyph, &'static str, &'static str) {
        match &self.question.kind {
            QuestionKind::Run { .. } => (Glyph::Waiting, "DECISION", "script"),
            QuestionKind::TypedYes { .. } => (
                Glyph::Waiting,
                "DECISION",
                "Destructive script · typed answer",
            ),
            QuestionKind::Mismatch { dead: true, .. } => (
                Glyph::Warning,
                "SESSION ENDED",
                "the session is gone; no accept",
            ),
            QuestionKind::Mismatch { .. } => {
                (Glyph::Warning, "MISMATCH", "expected output did not match")
            }
            QuestionKind::Confirm { .. } => (
                Glyph::Waiting,
                "CONFIRM",
                "the plan says it in words; you judge",
            ),
            QuestionKind::Session { .. } => (
                Glyph::Waiting,
                "NEW SESSION",
                "acknowledged even with --auto",
            ),
            QuestionKind::Manual => (Glyph::Manual, "MANUAL", "the plan asks you to do this"),
            QuestionKind::Unscoped => (
                Glyph::Waiting,
                "OUTSIDE SCOPE",
                "a file action outside the task's files",
            ),
            QuestionKind::Dirt { .. } => (
                Glyph::Waiting,
                "OTHER CHANGES",
                "outside the task's paths, before the commit",
            ),
            QuestionKind::Gate { .. } => (Glyph::Waiting, "COMMIT GATE", "closed · typed answer"),
            QuestionKind::Outside { .. } => (
                Glyph::Waiting,
                "WORKING DIRECTORY",
                "outside the repository",
            ),
            QuestionKind::Changed { .. } => (
                Glyph::Waiting,
                "CHANGED SINCE PREPARE",
                "something else writes the tree",
            ),
            QuestionKind::Again { .. } => (
                Glyph::Waiting,
                "RAN BEFORE THE INTERRUPTION",
                "run again or keep",
            ),
            QuestionKind::HeadMoved { .. } => {
                (Glyph::Warning, "HEAD MOVED", "while an action was running")
            }
            QuestionKind::Unknown { .. } => (
                Glyph::Waiting,
                "UNKNOWN STEP STATE",
                "neither as before nor as planned",
            ),
            QuestionKind::AcceptDone { .. } => (
                Glyph::Waiting,
                "ACCEPT AS DONE",
                "overrides the disk check · typed answer",
            ),
            QuestionKind::ResumeDirt { .. } => (
                Glyph::Waiting,
                "YOUR CHANGES",
                "a remaining file action writes over them",
            ),
        }
    }

    /// The body: what the run found (the payload), then the plan's text where it matters, then
    /// the runner's own words.
    fn body(&self, plan: &Plan, g: Glyphs) -> Vec<Spans> {
        let q = &self.question;
        let action = q.at.and_then(|at| {
            plan.tasks
                .get(at.task)?
                .steps
                .get(at.step)?
                .actions
                .get(at.action)
        });
        let mut out: Vec<Spans> = Vec::new();
        let row = |label: &str, text: String, color: Token| -> Spans {
            vec![
                (text::pad(label, 9, "…"), fg(Token::Fg3)),
                (text, fg(color)),
            ]
        };
        match &q.kind {
            QuestionKind::Run { decision } => {
                let why = match decision {
                    planrun_exec::policy::Decision::Ask(c) => {
                        format!("{} is not in --auto", class_name(*c))
                    }
                    other => format!("{other:?}"),
                };
                out.push(row("why", why, Token::Fg2));
                out.push(Vec::new());
                out.extend(block(plan, action));
            }
            QuestionKind::TypedYes { reasons } => {
                out.push(row(
                    "why",
                    format!("Destructive: {}", reasons.join("; ")),
                    Token::Err,
                ));
                out.push(Vec::new());
                out.extend(block(plan, action));
                out.push(Vec::new());
                let shell = action.and_then(script_shell).map_or("pwsh", shell_name);
                out.push(row(
                    "if yes",
                    format!("runs in the {shell} session; journaled as your approval"),
                    Token::Fg2,
                ));
                out.push(row(
                    "if skip",
                    "the block is journaled as skipped by you".to_owned(),
                    Token::Fg2,
                ));
                out.push(row(
                    "if quit",
                    "the run stops before this block; resume asks again".to_owned(),
                    Token::Fg2,
                ));
            }
            QuestionKind::Mismatch {
                status,
                mismatches,
                output,
                aligned,
                commit,
                dead,
            } => {
                out.push(row(
                    "ended",
                    status_text(status),
                    if *dead { Token::Err } else { Token::Fg2 },
                ));
                for m in mismatches {
                    if let Mismatch::Exit { expected, got } = m {
                        out.push(row(
                            "exit",
                            format!("expected {expected:?}, got {}", status_text(got)),
                            Token::Err,
                        ));
                    }
                }
                if !aligned.is_empty() {
                    out.push(Vec::new());
                    out.push(vec![
                        (
                            text::pad("EXPECTED lines, in order", 42, "…"),
                            bold(Token::Fg3),
                        ),
                        ("GOT".into(), bold(Token::Fg3)),
                    ]);
                    let expected = action
                        .and_then(|a| expected_lines(plan, a))
                        .unwrap_or_default();
                    for al in aligned {
                        let want = expected.get(al.number - 1).cloned().unwrap_or_default();
                        let (glyph, color) = if al.output.is_some() {
                            (Glyph::Done, Token::Ok)
                        } else {
                            (Glyph::Failed, Token::Err)
                        };
                        let got = al
                            .output
                            .and_then(|i| output.get(i))
                            .map_or(String::new(), |l| l.text.trim_end_matches('\r').to_owned());
                        let mut line: Spans = vec![
                            (format!("{} ", g.get(glyph)), bold(color)),
                            (text::pad(&want, 40, g.get(Glyph::Ellipsis)), fg(Token::Fg)),
                        ];
                        match al.output {
                            Some(i) => {
                                line.push((
                                    text::pad(&got, 28, g.get(Glyph::Ellipsis)),
                                    fg(Token::Fg2),
                                ));
                                line.push((
                                    format!("{} {}", g.get(Glyph::Aligned), i + 1),
                                    fg(Token::Fg3),
                                ));
                            }
                            None => line.push((
                                format!("{} not found", g.get(Glyph::Left)),
                                fg(Token::Err),
                            )),
                        }
                        out.push(line);
                    }
                }
                if let Some(problem) = commit {
                    out.push(row("commit", problem.clone(), Token::Err));
                }
                if *dead {
                    out.push(row("dead", "accepting would send the commit into a session without its variables (D75)".to_owned(), Token::Fg2));
                }
            }
            QuestionKind::Confirm { reasons, errored } => {
                let words: Vec<String> = reasons
                    .iter()
                    .map(|r| format!("{r:?}").to_lowercase())
                    .collect();
                out.push(row(
                    "checked",
                    "the exit status and any summary lines held".to_owned(),
                    Token::Ok,
                ));
                out.push(row("asks", words.join(", "), Token::Fg2));
                if *errored {
                    out.push(row(
                        "errored",
                        "pwsh recorded an error".to_owned(),
                        Token::Warn,
                    ));
                }
                if let Some(prose) = action.and_then(|a| expectation_prose(plan, a)) {
                    out.push(Vec::new());
                    out.push(vec![("the plan says".into(), bold(Token::Fg3))]);
                    out.extend(quote(&prose));
                }
            }
            QuestionKind::Session { why, earlier } => {
                out.push(row("why", why.clone(), Token::Fg2));
                out.push(row(
                    "gone",
                    "variables, functions and the directory of earlier blocks".to_owned(),
                    Token::Fg2,
                ));
                if !earlier.is_empty() {
                    out.push(row("earlier", earlier.join(", "), Token::Meta));
                }
            }
            QuestionKind::Manual => {
                let text = action.map(|a| match &a.kind {
                    ActionKind::Manual { text, .. } => plan.text(text).to_owned(),
                    _ => String::new(),
                });
                out.extend(quote(&text.unwrap_or_default()));
            }
            QuestionKind::Unscoped => {
                if let Some(a) = action
                    && let Some(path) = planrun_exec::handler::file_path(&a.kind)
                {
                    out.push(row("path", path.to_string(), Token::Fg));
                    out.push(row(
                        "origin",
                        format!("{:?}", a.path_origin).to_lowercase(),
                        Token::Fg2,
                    ));
                    let line = plan.source.line_col(a.span.start).line;
                    out.push(row("plan", format!("line {line}"), Token::Meta));
                }
            }
            QuestionKind::Dirt { paths } => {
                for p in paths {
                    out.push(vec![
                        (format!("{} ", p.code), fg(Token::Warn)),
                        (p.path.clone(), fg(Token::Fg)),
                    ]);
                }
            }
            QuestionKind::Outside { shell, cwd } => {
                out.push(row("shell", shell_name(*shell).to_owned(), Token::Fg2));
                out.push(row("cwd", cwd.display().to_string(), Token::Warn));
            }
            QuestionKind::Gate { reasons } => {
                for r in reasons {
                    out.push(vec![
                        (format!("{} ", g.get(Glyph::Failed)), bold(Token::Err)),
                        (r.clone(), fg(Token::Fg)),
                    ]);
                }
            }
            QuestionKind::Changed { paths } | QuestionKind::ResumeDirt { paths } => {
                for p in paths {
                    out.push(vec![
                        (format!("{} ", g.get(Glyph::Warning)), bold(Token::Warn)),
                        (p.to_string(), fg(Token::Fg)),
                    ]);
                }
            }
            QuestionKind::Again { running } => {
                let what = if *running {
                    "was running when planrun was interrupted"
                } else {
                    "ran before the interruption"
                };
                out.push(row("it", what.to_owned(), Token::Fg2));
            }
            QuestionKind::Unknown {
                step,
                paths,
                restorable,
            } => {
                out.push(row("step", place(step), Token::Meta));
                for (path, seen) in paths {
                    let (word, color) = match seen {
                        Seen::Post => ("as planned", Token::Ok),
                        Seen::Pre => ("as before", Token::Fg2),
                        Seen::Neither => ("neither", Token::Warn),
                    };
                    out.push(vec![
                        (text::pad(&path.to_string(), 40, "…"), fg(Token::Fg)),
                        (word.into(), fg(color)),
                    ]);
                }
                let r = if *restorable {
                    "every pre-image is there: restore is offered"
                } else {
                    "a pre-image is gone: no restore (D57)"
                };
                out.push(row("restore", r.to_owned(), Token::Fg2));
            }
            QuestionKind::AcceptDone { step } => {
                out.push(row("step", place(step), Token::Meta));
                out.push(row(
                    "means",
                    "its paths count as written, without the disk check".to_owned(),
                    Token::Fg2,
                ));
            }
            QuestionKind::HeadMoved {
                action,
                recorded,
                now,
            } => {
                out.push(row("action", place(action), Token::Meta));
                out.push(row("recorded", head_text(recorded), Token::Meta));
                out.push(row("now", head_text(now), Token::Meta));
            }
        }
        if !q.text.is_empty() {
            out.push(Vec::new());
            for line in &q.text {
                out.push(vec![(line.clone(), fg(Token::Fg2))]);
            }
        }
        out
    }

    /// Draws the dock over `area`.
    pub fn draw(&self, buf: &mut Buffer, area: Rect, plan: &Plan, focused: bool, g: Glyphs) {
        fill(
            buf,
            Rect::new(area.x, area.y, area.width, 1),
            on(Token::Fg, Token::Bg1),
        );
        let (glyph, title, detail) = self.title();
        let mut pen = Pen::new(buf, area.x, area.y, area.right(), g);
        pen.glyph(Glyph::Band, on(Token::Warn, Token::Bg1)).gap(1);
        let title_style = if focused {
            bold(Token::Warn)
        } else {
            fg(Token::Warn)
        };
        pen.glyph(glyph, title_style.bg(Token::Bg1.color())).gap(1);
        pen.text(title, title_style.bg(Token::Bg1.color())).gap(2);
        pen.text(detail, on(Token::Fg2, Token::Bg1));
        if let Some(at) = self.question.at
            && let Some(task) = plan.tasks.get(at.task)
        {
            let step = task
                .steps
                .get(at.step)
                .and_then(|s| s.number)
                .map_or_else(String::new, |n| format!(" {} step {n}", g.get(Glyph::Dot)));
            Pen::new(buf, area.x, area.y, area.right().saturating_sub(1), g).right(
                &format!("{}{step}", task.id.raw),
                on(Token::Fg3, Token::Bg1),
            );
        }
        let answers_h = 3;
        let body_area = Rect::new(
            area.x + 2,
            area.y + 2,
            area.width.saturating_sub(3),
            area.height.saturating_sub(2 + answers_h),
        );
        let body = self.body(plan, g);
        for (i, line) in body
            .iter()
            .skip(self.top)
            .take(usize::from(body_area.height))
            .enumerate()
        {
            let mut pen = Pen::new(
                buf,
                body_area.x,
                body_area.y + u16::try_from(i).unwrap_or(0),
                body_area.right(),
                g,
            );
            for (s, style) in line {
                pen.text(s, *style);
            }
        }
        let y = area.bottom().saturating_sub(answers_h);
        self.draw_answers(
            buf,
            Rect::new(area.x + 2, y, area.width.saturating_sub(3), answers_h),
            g,
        );
    }

    fn draw_answers(&self, buf: &mut Buffer, area: Rect, g: Glyphs) {
        let mut pen = Pen::new(buf, area.x, area.y, area.right(), g);
        match self.question.typed {
            Some(words) => {
                pen.text("answer  ", fg(Token::Fg3));
                let field = format!("{}{}", self.input, g.get(Glyph::Caret));
                pen.cell(&field, 12, bold(Token::Fg).bg(Token::Bg2.color()))
                    .gap(2);
                for w in words {
                    pen.text(w, bold(Token::Fg)).gap(1);
                    pen.text(word_meaning(&self.question.kind, w), fg(Token::Fg2))
                        .gap(3);
                }
                let note = match &self.refused {
                    Some(w) => (
                        format!("`{w}` is not one of the answers: type the whole word, then enter"),
                        fg(Token::Warn),
                    ),
                    None => (
                        "typed: the whole word, then enter; a single key never answers".to_owned(),
                        fg(Token::Fg3),
                    ),
                };
                Pen::new(buf, area.x, area.y + 1, area.right(), g).text(&note.0, note.1);
            }
            None => {
                for (i, c) in self.question.choices.iter().enumerate() {
                    if i > 0 {
                        pen.gap(3);
                    }
                    pen.chip(key_name(*c), c.label(), Token::Bg0);
                }
                if self.armed {
                    Pen::new(buf, area.x, area.y + 1, area.right(), g).text(
                        "q again quits; if the commit was made, quitting loses it for resume: answer y",
                        fg(Token::Warn),
                    );
                }
            }
        }
    }
}

/// The key a choice is pressed with, as the dock names it.
pub fn key_name(c: Choice) -> &'static str {
    match c.key() {
        "" => "enter",
        k => k,
    }
}

/// What a typed word does, for the answer row.
fn word_meaning(kind: &QuestionKind, word: &str) -> &'static str {
    match (kind, word) {
        (QuestionKind::Gate { .. }, "yes") => "open the gate",
        (QuestionKind::AcceptDone { .. }, "yes") => "accept as done",
        (_, "yes") => "run it",
        (_, "skip") => "skip this script",
        (_, "quit") => "stop the run",
        _ => "",
    }
}

/// The action's script block, numbered.
fn block(plan: &Plan, action: Option<&Action>) -> Vec<Spans> {
    let Some(script) = action.and_then(|a| match &a.kind {
        ActionKind::RunScript(s) | ActionKind::Commit(s) => Some(s),
        _ => None,
    }) else {
        return Vec::new();
    };
    plan.text(&script.body)
        .lines()
        .enumerate()
        .map(|(i, l)| {
            vec![
                (format!("{:>3}  ", i + 1), fg(Token::Fg3)),
                (l.to_owned(), fg(Token::Fg)),
            ]
        })
        .collect()
}

fn quote(text: &str) -> Vec<Spans> {
    text.lines()
        .map(|l| {
            vec![
                ("│ ".to_owned(), fg(Token::Warn)),
                (l.to_owned(), fg(Token::Fg)),
            ]
        })
        .collect()
}

fn script_shell(action: &Action) -> Option<Shell> {
    match &action.kind {
        ActionKind::RunScript(s) | ActionKind::Commit(s) => Some(s.shell),
        _ => None,
    }
}

fn shell_name(shell: Shell) -> &'static str {
    match shell {
        Shell::Pwsh => "pwsh",
        Shell::Bash => "bash",
    }
}

fn expected_lines(plan: &Plan, action: &Action) -> Option<Vec<String>> {
    let script = match &action.kind {
        ActionKind::RunScript(s) | ActionKind::Commit(s) => s,
        _ => return None,
    };
    let lines = script.expect.as_ref()?.lines.as_ref()?;
    Some(plan.text(lines).lines().map(str::to_owned).collect())
}

fn expectation_prose(plan: &Plan, action: &Action) -> Option<String> {
    let script = match &action.kind {
        ActionKind::RunScript(s) | ActionKind::Commit(s) => s,
        _ => return None,
    };
    let prose = script.expect.as_ref()?.prose.as_ref()?;
    Some(plan.text(prose).to_owned())
}

fn status_text(status: &RunStatus) -> String {
    match status {
        RunStatus::Exited(code) => format!("exit {code}"),
        RunStatus::Threw => "a terminating error".to_owned(),
        RunStatus::SessionEnded(Some(code)) => format!("the session ended (exit {code})"),
        RunStatus::SessionEnded(None) => "the session ended".to_owned(),
        RunStatus::TimedOut => "timed out".to_owned(),
        RunStatus::Broken(why) => format!("the session broke: {why}"),
    }
}

fn place(at: &At) -> String {
    format!("{} step {} action {}", at.task, at.step + 1, at.action + 1)
}

fn head_text(head: &planrun_exec::git::Head) -> String {
    let commit = head
        .commit
        .as_deref()
        .map_or("(none)", |c| &c[..c.len().min(7)]);
    match &head.reference {
        Some(r) => format!("{commit} on {r}"),
        None => commit.to_owned(),
    }
}
