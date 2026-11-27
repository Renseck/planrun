//! The plan view in Run mode (docs/tui-design.md §6.C, decisions D84, D90): the same layout as
//! Inspect, filled from the live runner. The navigator's states, durations, outcome tags and
//! change-state chips come from the journal's records; the console from `Ui::output` and
//! `Ui::status`, shown as prose and never parsed. A question opens the dock over the console.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use planrun_exec::journal::{At, Outcome, Record};
use planrun_exec::runner::Finished;
use planrun_exec::shell::{OutLine, Stream};
use planrun_exec::ui::{Question, Reply};
use planrun_model::{ActionKind, ManifestPath, Plan};
use planrun_parse::Parsed;

use crate::diff::{self, FileDiff};
use crate::dock::Dock;
use crate::draw::{Pen, band, hrule};
use crate::glyph::{Glyph, Glyphs};
use crate::inspect::{Assumed, tag};
use crate::keys::Key;
use crate::text;
use crate::theme::{Token, bold, fg, on};
use crate::tree::{self, Mark, Node, Tree};

/// A console line's label (§6.B/C).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Label {
    /// A task starts.
    Task,
    /// Preflight.
    Pre,
    /// A step's files.
    File,
    /// A script starts.
    Run,
    /// Output on stdout.
    Out,
    /// Output on stderr.
    Err,
    /// A script's verdict.
    Verify,
    /// A commit.
    Commit,
    /// Status prose, answers.
    Event,
}

impl Label {
    fn word(self) -> &'static str {
        match self {
            Label::Task => "TASK",
            Label::Pre => "PRE",
            Label::File => "FILE",
            Label::Run => "RUN",
            Label::Out => "out",
            Label::Err => "err",
            Label::Verify => "VERIFY",
            Label::Commit => "COMMIT",
            Label::Event => "EVENT",
        }
    }

    fn style(self) -> Style {
        match self {
            Label::Task => bold(Token::Fg),
            Label::Run => bold(Token::Act),
            Label::Out => fg(Token::Fg3),
            Label::Err => fg(Token::Warn),
            Label::Verify | Label::Commit => bold(Token::Fg2),
            Label::Pre | Label::File | Label::Event => fg(Token::Fg2),
        }
    }

    /// Whether filter `n` (1 all, 2 events, 3 output, 4 verify, 5 warnings) shows it.
    fn shown(self, filter: u8, warning: bool) -> bool {
        match filter {
            2 => !matches!(self, Label::Out | Label::Err),
            3 => matches!(self, Label::Out | Label::Err),
            4 => matches!(self, Label::Verify | Label::Commit),
            5 => self == Label::Err || warning,
            _ => true,
        }
    }
}

/// One console line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// Milliseconds since the run started.
    pub at: u64,
    /// Its label.
    pub label: Label,
    /// Its text.
    pub text: String,
    /// A warning (a mismatch, a failure).
    pub warning: bool,
}

/// How far the run is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    /// Taking the lock, opening the journal.
    Starting,
    /// Running.
    Executing,
    /// It ended.
    Ended(Result<Finished, String>),
}

/// One action's journaled state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// `action-start` without its end.
    Running(u64),
    /// `action-end`, with its duration and status.
    Ended {
        /// How.
        outcome: Outcome,
        /// Milliseconds it took.
        took: u64,
    },
}

/// The live run.
pub struct Run {
    /// The plan file.
    pub path: PathBuf,
    /// The plan.
    pub parsed: Arc<Parsed>,
    /// The policy it runs with.
    pub assumed: Assumed,
    /// A resume.
    pub resume: bool,
    /// When it started.
    pub started: u64,
    /// How far it is.
    pub phase: Phase,
    /// Tasks started, by position.
    pub tasks_started: BTreeSet<usize>,
    /// Tasks ended, by position, with their HEAD.
    pub tasks_done: BTreeMap<usize, Option<String>>,
    /// The current task.
    pub current: Option<usize>,
    /// Actions by (task, step, action).
    pub acts: BTreeMap<(usize, usize, usize), Act>,
    /// Written file groups (their first action).
    pub written: BTreeSet<(usize, usize, usize)>,
    /// The console.
    pub console: Vec<Line>,
    /// The console filter, 1–5.
    pub filter: u8,
    /// Following the console's end.
    pub follow: bool,
    /// The console's first line when not following.
    pub scroll: usize,
    /// The navigator.
    pub tree: Tree,
    /// The open question.
    pub dock: Option<Dock>,
    /// The CHANGES files of the current task, and the one shown.
    pub changes: Vec<FileDiff>,
    /// The file shown in CHANGES.
    pub file: usize,
    /// The diff `git diff HEAD` gave for the current task's paths.
    pub on_disk: Vec<FileDiff>,
    /// Focus: 0 the navigator, 1 CHANGES, 2 the console or the dock.
    pub focus: u8,
    /// What the upper right panel shows.
    pub inspector: Inspector,
    /// Ctrl-C was pressed: the dock asks whether to end planrun (D91).
    pub ending: bool,
}

/// What the upper right panel shows while a run is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inspector {
    /// CHANGES: the current task's files.
    Changes,
    /// `[v]`: the action in full.
    Action((usize, usize, usize)),
    /// `[d]`: `git diff HEAD` and the untracked files.
    Tree(Vec<FileDiff>, Vec<String>),
}

/// What the app should do after a run message.
pub enum Follow {
    /// Nothing.
    None,
    /// Read the current task's diff again.
    Changes(usize, Vec<String>),
}

impl Run {
    /// A run of `parsed` with `assumed`, starting at `now`.
    pub fn new(
        path: PathBuf,
        parsed: Arc<Parsed>,
        assumed: Assumed,
        resume: bool,
        now: u64,
    ) -> Run {
        Run {
            path,
            parsed,
            assumed,
            resume,
            started: now,
            phase: Phase::Starting,
            tasks_started: BTreeSet::new(),
            tasks_done: BTreeMap::new(),
            current: None,
            acts: BTreeMap::new(),
            written: BTreeSet::new(),
            console: Vec::new(),
            filter: 1,
            follow: true,
            scroll: 0,
            tree: Tree::default(),
            dock: None,
            changes: Vec::new(),
            file: 0,
            on_disk: Vec::new(),
            focus: 0,
            inspector: Inspector::Changes,
            ending: false,
        }
    }

    /// The plan.
    pub fn plan(&self) -> &Plan {
        &self.parsed.plan
    }

    /// Whether the runner still runs.
    pub fn active(&self) -> bool {
        !matches!(self.phase, Phase::Ended(_))
    }

    fn task_of(&self, id: &str) -> Option<usize> {
        self.plan().tasks.iter().position(|t| &*t.id.raw == id)
    }

    fn place(&self, at: &At) -> Option<(usize, usize, usize)> {
        Some((self.task_of(&at.task)?, at.step, at.action))
    }

    fn log(&mut self, now: u64, label: Label, text: String, warning: bool) {
        self.console.push(Line {
            at: now.saturating_sub(self.started),
            label,
            text,
            warning,
        });
    }

    /// The run started.
    pub fn started(&mut self) {
        self.phase = Phase::Executing;
    }

    /// The run ended.
    pub fn ended(&mut self, now: u64, result: Result<Finished, String>) {
        let (text, warning) = match &result {
            Ok(f) if f.reason.is_none() => ("the run completed".to_owned(), false),
            Ok(f) => (
                format!("stopped: {}", f.reason.clone().unwrap_or_default()),
                true,
            ),
            Err(e) => (e.clone(), true),
        };
        self.log(now, Label::Event, text, warning);
        self.phase = Phase::Ended(result);
        self.dock = None;
    }

    /// `Ui::status`.
    pub fn status(&mut self, now: u64, line: &str) {
        let label = if line.starts_with("preflight") {
            Label::Pre
        } else {
            Label::Event
        };
        for l in line.lines() {
            self.log(now, label, l.to_owned(), false);
        }
    }

    /// `Ui::output`.
    pub fn output(&mut self, now: u64, line: &OutLine) {
        let label = match line.stream {
            Stream::Stdout => Label::Out,
            Stream::Stderr => Label::Err,
        };
        self.log(
            now,
            label,
            line.text.trim_end_matches('\r').to_owned(),
            false,
        );
    }

    /// `Ui::ask`.
    pub fn ask(&mut self, question: Question, reply: Sender<Reply>) {
        if let Some(at) = question.at {
            self.tree.select(Node::Action(at.task, at.step, at.action));
        }
        let commit = question.at.is_some_and(|at| {
            matches!(
                self.action((at.task, at.step, at.action)).map(|a| &a.kind),
                Some(ActionKind::Commit(_))
            )
        });
        let mut dock = Dock::new(question, reply);
        dock.guard_quit = commit;
        self.dock = Some(dock);
        self.focus = 2;
    }

    /// A journal record.
    pub fn record(&mut self, now: u64, record: &Record) -> Follow {
        match record {
            Record::TaskStart { task, .. } => {
                let Some(t) = self.task_of(task) else {
                    return Follow::None;
                };
                self.tasks_started.insert(t);
                self.current = Some(t);
                let title = self.plan().text(&self.plan().tasks[t].title).to_owned();
                self.log(now, Label::Task, format!("{task} {title}"), false);
                self.file = 0;
                self.on_disk.clear();
                self.changes = self.files(t);
                return self.refresh();
            }
            Record::StepPrepared { at, .. } => {
                self.log(
                    now,
                    Label::File,
                    format!("step {} prepared", at.step),
                    false,
                );
            }
            Record::StepWritten { at }
            | Record::StepVerified { at }
            | Record::StepAccepted { at } => {
                if let Some(p) = self.place(at) {
                    self.written.insert(p);
                }
                let what = if matches!(record, Record::StepWritten { .. }) {
                    "written"
                } else {
                    "found done"
                };
                let paths = self.group_paths(at);
                self.log(
                    now,
                    Label::File,
                    format!("step {what} {} {paths}", Glyphs::unicode().get(Glyph::Dot)),
                    false,
                );
                return self.refresh();
            }
            Record::StepFailed { error, .. } => {
                self.log(
                    now,
                    Label::File,
                    format!("step failed, rolled back: {error}"),
                    true,
                );
            }
            Record::ActionStart { at, kind, .. } => {
                if let Some(p) = self.place(at) {
                    self.acts.insert(p, Act::Running(now));
                    self.tree.select(Node::Action(p.0, p.1, p.2));
                    if kind == "script" {
                        let label = self.action_label(p);
                        self.log(now, Label::Run, format!("$ {label}"), false);
                    }
                }
            }
            Record::ActionEnd {
                at,
                outcome,
                status,
                head,
                ..
            } => {
                let Some(p) = self.place(at) else {
                    return Follow::None;
                };
                let started = match self.acts.get(&p) {
                    Some(Act::Running(s)) => *s,
                    _ => now,
                };
                self.acts.insert(
                    p,
                    Act::Ended {
                        outcome: *outcome,
                        took: now.saturating_sub(started),
                    },
                );
                let commit = matches!(self.action(p).map(|a| &a.kind), Some(ActionKind::Commit(_)));
                let warning = matches!(
                    outcome,
                    Outcome::Failed | Outcome::Refused | Outcome::Accepted | Outcome::Missing
                );
                let status = status.clone().unwrap_or_default();
                if commit {
                    let hash = head
                        .commit
                        .as_deref()
                        .map_or("", |c| &c[..c.len().min(7)])
                        .to_owned();
                    self.log(
                        now,
                        Label::Commit,
                        format!("{hash} {}", outcome_word(*outcome)),
                        warning,
                    );
                } else {
                    let glyph = if warning { Glyph::Failed } else { Glyph::Done };
                    self.log(
                        now,
                        Label::Verify,
                        format!(
                            "{} {} {status}",
                            Glyphs::unicode().get(glyph),
                            outcome_word(*outcome)
                        ),
                        warning,
                    );
                }
                return self.refresh();
            }
            Record::Answer {
                question, answer, ..
            } => {
                self.log(
                    now,
                    Label::Event,
                    format!("answered {question}: {answer}"),
                    false,
                );
            }
            Record::GateOverride { .. } => {
                self.log(
                    now,
                    Label::Event,
                    "the commit gate was opened with a typed yes".to_owned(),
                    true,
                );
            }
            Record::TaskEnd { task, head } => {
                if let Some(t) = self.task_of(task) {
                    self.tasks_done.insert(t, head.commit.clone());
                    self.tree.folded.insert(t);
                    return self.refresh();
                }
            }
            _ => {}
        }
        Follow::None
    }

    /// The paths of the file group at `at`.
    fn group_paths(&self, at: &At) -> String {
        let Some((t, s, a)) = self.place(at) else {
            return String::new();
        };
        let step = &self.plan().tasks[t].steps[s];
        let g = Glyphs::unicode();
        step.actions[a..]
            .iter()
            .take_while(|x| {
                planrun_exec::handler::file_path(&x.kind).is_some()
                    && !matches!(x.kind, ActionKind::Generated { .. })
            })
            .map(|x| {
                format!(
                    "{} {}",
                    g.get(tree::kind_glyph(x)),
                    tree::label(self.plan(), x)
                )
            })
            .collect::<Vec<_>>()
            .join("  ")
    }

    fn action(&self, (t, s, a): (usize, usize, usize)) -> Option<&planrun_model::Action> {
        self.plan().tasks.get(t)?.steps.get(s)?.actions.get(a)
    }

    fn action_label(&self, p: (usize, usize, usize)) -> String {
        self.action(p)
            .map_or_else(String::new, |a| tree::label(self.plan(), a))
    }

    /// The current task's files: its file actions' paths and its manifest's file entries.
    fn files(&self, t: usize) -> Vec<FileDiff> {
        let plan = self.plan();
        let task = &plan.tasks[t];
        let mut out: Vec<FileDiff> = Vec::new();
        for action in task.steps.iter().flat_map(|s| s.actions.iter()) {
            if matches!(action.kind, ActionKind::Generated { .. }) {
                continue;
            }
            let Some(planned) = diff::planned(plan, &action.kind) else {
                continue;
            };
            match out.iter_mut().find(|f| f.path == planned.path) {
                Some(f) => f.rows.extend(planned.rows),
                None => out.push(planned),
            }
        }
        for entry in &task.manifest.entries {
            if let ManifestPath::File(p) = &entry.path
                && !out.iter().any(|f| f.path == p.to_string())
            {
                out.push(FileDiff {
                    path: p.to_string(),
                    rows: Vec::new(),
                });
            }
        }
        out
    }

    fn refresh(&self) -> Follow {
        match self.current {
            Some(t) => Follow::Changes(t, self.changes.iter().map(|f| f.path.clone()).collect()),
            None => Follow::None,
        }
    }

    /// `git diff HEAD` (or the commit's own diff) of task `t`'s paths.
    pub fn diffed(&mut self, t: usize, files: Vec<FileDiff>) {
        if self.current == Some(t) {
            self.on_disk = files;
        }
    }

    /// Whether the file group containing (`t`, `s`, `a`) was written.
    fn applied(&self, t: usize, s: usize, a: usize) -> bool {
        let step = &self.plan().tasks[t].steps[s];
        (0..=a)
            .rev()
            .take_while(|&i| planrun_exec::handler::file_path(&step.actions[i].kind).is_some())
            .any(|i| self.written.contains(&(t, s, i)))
    }

    /// The change state of `path` in the current task: committed, applied or planned.
    fn chip(&self, path: &str) -> (&'static str, Token) {
        let Some(t) = self.current else {
            return ("PLANNED", Token::Fg2);
        };
        if self.tasks_done.contains_key(&t) {
            return ("COMMITTED", Token::Ok);
        }
        let task = &self.plan().tasks[t];
        for (s, step) in task.steps.iter().enumerate() {
            for (a, action) in step.actions.iter().enumerate() {
                let same = planrun_exec::handler::file_path(&action.kind)
                    .is_some_and(|p| p.to_string() == path);
                if same && self.applied(t, s, a) {
                    return ("APPLIED", Token::Act);
                }
            }
        }
        ("PLANNED", Token::Fg2)
    }

    /// The navigator's mark of `node`.
    pub fn mark(&self, node: Node) -> Mark {
        match node {
            Node::Task(t) => {
                if let Some(head) = self.tasks_done.get(&t) {
                    let hash = head
                        .as_deref()
                        .map_or(String::new(), |c| c[..c.len().min(7)].to_owned());
                    return Mark {
                        glyph: Glyph::Done,
                        color: Token::Ok,
                        tag: hash,
                        tag_color: Token::Meta,
                    };
                }
                if self.current == Some(t) && self.active() {
                    let glyph = if self.dock.is_some() {
                        Glyph::Waiting
                    } else {
                        Glyph::Running
                    };
                    let color = if self.dock.is_some() {
                        Token::Warn
                    } else {
                        Token::Act
                    };
                    return Mark::plain(glyph, color);
                }
                if self.current == Some(t) {
                    return Mark::plain(Glyph::Failed, Token::Err);
                }
                let manual = self.plan().tasks[t]
                    .steps
                    .iter()
                    .flat_map(|s| s.actions.iter())
                    .filter(|a| matches!(a.kind, ActionKind::Manual { .. }))
                    .count();
                let tag = if manual > 0 {
                    format!("{} {manual}", Glyphs::unicode().get(Glyph::Manual))
                } else {
                    String::new()
                };
                Mark {
                    glyph: Glyph::Pending,
                    color: Token::Fg3,
                    tag,
                    tag_color: Token::Warn,
                }
            }
            Node::Step(t, s) => {
                let n = self.plan().tasks[t].steps[s].actions.len();
                let marks: Vec<Mark> = (0..n).map(|a| self.mark(Node::Action(t, s, a))).collect();
                let pick = [
                    Glyph::Waiting,
                    Glyph::Running,
                    Glyph::Failed,
                    Glyph::Warning,
                ]
                .into_iter()
                .find_map(|g| marks.iter().find(|m| m.glyph == g));
                if let Some(m) = pick {
                    return Mark::plain(m.glyph, m.color);
                }
                if !marks.is_empty()
                    && marks.iter().all(|m| {
                        m.glyph == Glyph::Done || m.glyph == Glyph::Skipped || m.glyph == Glyph::Dot
                    })
                    && marks.iter().any(|m| m.glyph == Glyph::Done)
                {
                    return Mark::plain(Glyph::Done, Token::Ok);
                }
                if !marks.is_empty() && marks.iter().all(|m| m.glyph == Glyph::Skipped) {
                    return Mark::plain(Glyph::Skipped, Token::Fg3);
                }
                Mark::plain(Glyph::Pending, Token::Fg3)
            }
            Node::Action(t, s, a) => self.action_mark(t, s, a),
        }
    }

    fn action_mark(&self, t: usize, s: usize, a: usize) -> Mark {
        let plan = self.plan();
        let task = &plan.tasks[t];
        let action = &task.steps[s].actions[a];
        let waiting = self
            .dock
            .as_ref()
            .and_then(|d| d.question.at)
            .is_some_and(|p| (p.task, p.step, p.action) == (t, s, a));
        if waiting {
            let tag = if self
                .dock
                .as_ref()
                .is_some_and(|d| d.question.typed.is_some())
            {
                "yes?"
            } else {
                "waiting"
            };
            return Mark {
                glyph: Glyph::Waiting,
                color: Token::Warn,
                tag: tag.into(),
                tag_color: Token::Warn,
            };
        }
        if matches!(action.kind, ActionKind::Note { .. }) {
            return Mark::plain(Glyph::Dot, Token::Fg3);
        }
        let file = planrun_exec::handler::file_path(&action.kind).is_some()
            && !matches!(action.kind, ActionKind::Generated { .. });
        if file && self.applied(t, s, a) {
            return Mark {
                glyph: Glyph::Done,
                color: Token::Ok,
                tag: "applied".into(),
                tag_color: Token::Fg3,
            };
        }
        match self.acts.get(&(t, s, a)) {
            Some(Act::Running(_)) => Mark {
                glyph: Glyph::Running,
                color: Token::Act,
                tag: "running".into(),
                tag_color: Token::Act,
            },
            Some(Act::Ended { outcome, took }) => {
                let (glyph, color) = match outcome {
                    Outcome::Passed
                    | Outcome::Confirmed
                    | Outcome::Acknowledged
                    | Outcome::Present
                    | Outcome::Kept => (Glyph::Done, Token::Ok),
                    Outcome::Accepted => (Glyph::Warning, Token::Warn),
                    Outcome::SkippedPolicy | Outcome::SkippedUser => (Glyph::Skipped, Token::Fg3),
                    Outcome::Failed | Outcome::Refused | Outcome::Missing => {
                        (Glyph::Failed, Token::Err)
                    }
                };
                let tag = match outcome {
                    Outcome::Passed | Outcome::Confirmed => text::duration(*took),
                    other => outcome_word(*other).to_owned(),
                };
                Mark {
                    glyph,
                    color,
                    tag,
                    tag_color: Token::Fg3,
                }
            }
            None => {
                let decision = self.assumed.policy().decide(
                    task,
                    action,
                    &plan.source,
                    &planrun_exec::policy::DefaultClassifier,
                );
                let (tag, tag_color) = tag(action, decision);
                Mark {
                    glyph: Glyph::Pending,
                    color: Token::Fg3,
                    tag,
                    tag_color,
                }
            }
        }
    }

    /// Handles `key` outside the dock; false when it is not the run view's.
    pub fn key(&mut self, key: Key) -> bool {
        match key {
            Key::Tab => self.focus = (self.focus + 1) % 3,
            Key::BackTab => self.focus = (self.focus + 2) % 3,
            Key::Char('f') => {
                self.follow = true;
            }
            Key::Char(c @ '1'..='5') => self.filter = c as u8 - b'0',
            _ => return self.panel_key(key),
        }
        true
    }

    fn panel_key(&mut self, key: Key) -> bool {
        match self.focus {
            0 => {
                let plan = self.parsed.plan.clone();
                self.tree.key(key, &plan)
            }
            1 => {
                let n = self.changes.len().max(1);
                match key {
                    Key::Right | Key::Char('l') => self.file = (self.file + 1) % n,
                    Key::Left | Key::Char('h') => self.file = (self.file + n - 1) % n,
                    _ => return false,
                }
                true
            }
            _ => {
                let shown = self.shown().len();
                let at = if self.follow {
                    shown.saturating_sub(1)
                } else {
                    self.scroll
                };
                match key {
                    Key::Up | Key::Char('k') => {
                        self.follow = false;
                        self.scroll = at.saturating_sub(1);
                    }
                    Key::Down | Key::Char('j') => {
                        self.scroll = (at + 1).min(shown.saturating_sub(1));
                        self.follow = self.scroll + 1 >= shown;
                    }
                    Key::PageUp => {
                        self.follow = false;
                        self.scroll = at.saturating_sub(10);
                    }
                    Key::End => self.follow = true,
                    _ => return false,
                }
                true
            }
        }
    }

    fn shown(&self) -> Vec<&Line> {
        self.console
            .iter()
            .filter(|l| l.label.shown(self.filter, l.warning))
            .collect()
    }

    /// The header's right side.
    pub fn state(&self, g: Glyphs) -> Vec<(String, Style)> {
        let bar = |t: Token| on(t, Token::Bg1);
        let mut out = vec![
            (" RUN ".to_owned(), bold(Token::Act).bg(Token::Bg1.color())),
            ("  ".to_owned(), bar(Token::Fg)),
        ];
        match (&self.phase, &self.dock) {
            (Phase::Ended(Ok(f)), _) if f.reason.is_none() => {
                out.push((format!("{} done", g.get(Glyph::Done)), bar(Token::Ok)));
            }
            (Phase::Ended(_), _) => {
                out.push((format!("{} stopped", g.get(Glyph::Failed)), bar(Token::Err)))
            }
            (_, Some(_)) => out.push((
                format!("{} waiting for you", g.get(Glyph::Waiting)),
                bold(Token::Warn).bg(Token::Bg1.color()),
            )),
            (Phase::Starting, None) => out.push((
                format!("{} starting", g.get(Glyph::Background)),
                bar(Token::Fg2),
            )),
            (Phase::Executing, None) => {
                out.push((format!("{} ", g.get(Glyph::Running)), bar(Token::Act)));
                out.push(("executing".to_owned(), bar(Token::Fg)));
            }
        }
        if self.dock.is_none() || !self.active() {
            out.push((
                format!("  {}  {}", g.get(Glyph::Dot), self.assumed.describe()),
                bar(Token::Fg3),
            ));
        }
        out
    }

    /// The crumbs after the repository.
    pub fn crumbs(&self, g: Glyphs) -> Vec<String> {
        let name = self
            .path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let mut out = vec![name];
        let plan = self.plan();
        let Some(node) = self.tree.current(plan) else {
            return out;
        };
        out.push(plan.tasks[node.task()].id.raw.to_string());
        if let Node::Action(t, s, a) = node {
            let step = &plan.tasks[t].steps[s];
            if let Some(n) = step.number {
                out.push(format!("step {n}"));
            }
            let action = &step.actions[a];
            let label = tree::label(plan, action);
            let short = match action.kind {
                ActionKind::RunScript(_) => {
                    label.split_whitespace().next().unwrap_or("").to_owned()
                }
                _ => text::fit(&label, 24, g.get(Glyph::Ellipsis)),
            };
            out.push(format!("{} {short}", g.get(tree::kind_glyph(action))));
        }
        out
    }

    /// The keys the footer shows.
    pub fn footer(&self) -> Vec<(String, String)> {
        if self.ending {
            return vec![
                ("y".into(), "end planrun".into()),
                ("n".into(), "keep running".into()),
            ];
        }
        if let Some(dock) = &self.dock {
            return dock.footer();
        }
        if !self.active() {
            return vec![
                ("esc".into(), "plans".into()),
                ("F4".into(), "history".into()),
                ("tab".into(), "next panel".into()),
            ];
        }
        vec![
            ("f".into(), "follow".into()),
            ("1–5".into(), "filter".into()),
            ("tab".into(), "next panel".into()),
            ("ctrl+c".into(), "interrupt…".into()),
        ]
    }

    /// Draws the run view in `area`.
    pub fn draw(&self, buf: &mut Buffer, area: Rect, g: Glyphs) {
        let plan = self.plan();
        let nav = Rect::new(area.x, area.y, tree::WIDTH, area.height);
        let n = plan.tasks.len();
        let at = self.current.map_or(0, |t| t + 1);
        let title = format!("task {at} of {n}");
        self.tree.draw(
            buf,
            nav,
            plan,
            &title,
            self.focus == 0,
            &|node| self.mark(node),
            g,
        );
        let right = Rect::new(
            area.x + tree::WIDTH,
            area.y,
            area.width - tree::WIDTH,
            area.height,
        );
        let upper_h = right.height * 55 / 100;
        let upper = Rect::new(right.x, right.y, right.width, upper_h);
        let lower = Rect::new(
            right.x,
            right.y + upper_h,
            right.width,
            right.height - upper_h,
        );
        match &self.inspector {
            Inspector::Changes => self.draw_changes(buf, upper, g),
            Inspector::Action(p) => self.draw_viewed(buf, upper, *p, g),
            Inspector::Tree(files, untracked) => {
                self.draw_tree_diff(buf, upper, files, untracked, g)
            }
        }
        match &self.dock {
            _ if self.ending => draw_end(buf, lower, g),
            Some(dock) => dock.draw(buf, lower, plan, self.focus == 2, g),
            None => self.draw_console(buf, lower, g),
        }
    }

    /// `[v]`: the action in full, its planned diff or its block; the dock stays open (D90).
    fn draw_viewed(&self, buf: &mut Buffer, area: Rect, p: (usize, usize, usize), g: Glyphs) {
        let plan = self.plan();
        let Some(action) = self.action(p) else {
            return;
        };
        let line = plan.source.line_col(action.span.start).line;
        band(
            buf,
            area,
            self.focus == 1,
            "ACTION",
            &format!("line {line} of the plan"),
            g,
        );
        let mut pen = Pen::new(buf, area.x + 2, area.y + 1, area.right(), g);
        pen.glyph(tree::kind_glyph(action), bold(tree::kind_color(action)))
            .gap(1);
        pen.text(&tree::label(plan, action), bold(Token::Fg));
        let rest = Rect::new(
            area.x + 1,
            area.y + 2,
            area.width - 1,
            area.height.saturating_sub(2),
        );
        if let Some(diff) = diff::planned(plan, &action.kind) {
            diff::header(buf, rest, &diff, ("PLANNED", Token::Fg2), "", g);
            let rows = Rect::new(
                rest.x,
                rest.y + 1,
                rest.width,
                rest.height.saturating_sub(1),
            );
            diff::draw_rows(buf, rows, &diff.rows, 0, g);
            return;
        }
        let body = match &action.kind {
            ActionKind::RunScript(s) | ActionKind::Commit(s) => plan.text(&s.body),
            ActionKind::Manual { text, .. } | ActionKind::Note { text } => plan.text(text),
            _ => "",
        };
        for (i, l) in body.lines().take(usize::from(rest.height)).enumerate() {
            let mut pen = Pen::new(
                buf,
                rest.x + 1,
                rest.y + u16::try_from(i).unwrap_or(0),
                rest.right(),
                g,
            );
            pen.text(&text::right(&(i + 1).to_string(), 4), fg(Token::Fg3))
                .gap(2)
                .text(l, fg(Token::Fg));
        }
    }

    /// `[d]`: the working tree against HEAD, and the untracked files.
    fn draw_tree_diff(
        &self,
        buf: &mut Buffer,
        area: Rect,
        files: &[FileDiff],
        untracked: &[String],
        g: Glyphs,
    ) {
        band(
            buf,
            area,
            self.focus == 1,
            "DIFF",
            "git diff HEAD, and the untracked files",
            g,
        );
        let mut y = area.y + 1;
        for file in files {
            if y + 1 >= area.bottom() {
                return;
            }
            let header = Rect::new(area.x + 1, y, area.width - 1, 1);
            diff::header(buf, header, file, ("WORKING TREE", Token::Act), "", g);
            let n = u16::try_from(file.rows.len())
                .unwrap_or(u16::MAX)
                .min(area.bottom() - y - 1);
            diff::draw_rows(
                buf,
                Rect::new(area.x + 1, y + 1, area.width - 1, n),
                &file.rows,
                0,
                g,
            );
            y += n + 1;
        }
        for path in untracked {
            if y >= area.bottom() {
                return;
            }
            Pen::new(buf, area.x + 2, y, area.right(), g)
                .text("?? ", fg(Token::Warn))
                .text(path, fg(Token::Fg));
            y += 1;
        }
        if files.is_empty() && untracked.is_empty() {
            Pen::new(buf, area.x + 2, y, area.right(), g)
                .text("no changes against HEAD", fg(Token::Fg2));
        }
    }

    fn draw_changes(&self, buf: &mut Buffer, area: Rect, g: Glyphs) {
        let task = self
            .current
            .map(|t| self.plan().tasks[t].id.raw.to_string());
        let n = self.changes.len();
        let detail = match &task {
            Some(id) => format!(
                "task {id} {} {n} file{}",
                g.get(Glyph::Dot),
                if n == 1 { "" } else { "s" }
            ),
            None => String::new(),
        };
        band(buf, area, self.focus == 1, "CHANGES", &detail, g);
        if n == 0 {
            return;
        }
        let shown: Vec<FileDiff> = self
            .changes
            .iter()
            .map(|planned| {
                self.on_disk
                    .iter()
                    .find(|d| d.path == planned.path)
                    .cloned()
                    .unwrap_or_else(|| planned.clone())
            })
            .collect();
        let mut pen = Pen::new(buf, area.x + 2, area.y + 1, area.right(), g);
        for (i, f) in shown.iter().enumerate() {
            let (add, del) = f.counts();
            let style = if i == self.file {
                bold(Token::Fg)
            } else {
                fg(Token::Fg2)
            };
            pen.text(&f.path, style).gap(1);
            if add > 0 {
                pen.text(&format!("+{add}"), fg(Token::Fg2)).gap(1);
            }
            if del > 0 {
                pen.text(&format!("{}{del}", g.get(Glyph::Delete)), fg(Token::Fg2))
                    .gap(1);
            }
            pen.gap(2);
        }
        hrule(buf, area.x + 1, area.right(), area.y + 2, g);
        let file = &shown[self.file.min(n - 1)];
        let header = Rect::new(
            area.x + 1,
            area.y + 3,
            area.width - 1,
            area.height.saturating_sub(3),
        );
        let chip = self.chip(&file.path);
        let detail = match chip.0 {
            "APPLIED" => "on disk, not committed",
            "COMMITTED" => "in HEAD",
            _ => "not written yet",
        };
        diff::header(buf, header, file, chip, detail, g);
        let rows = Rect::new(
            header.x,
            header.y + 1,
            header.width,
            header.height.saturating_sub(1),
        );
        diff::draw_rows(buf, rows, &file.rows, 0, g);
    }

    fn draw_console(&self, buf: &mut Buffer, area: Rect, g: Glyphs) {
        band(buf, area, self.focus == 2, "CONSOLE", "", g);
        let mut pen = Pen::new(buf, area.x + 13, area.y, area.right(), g);
        for (i, name) in ["all", "events", "output", "verify", "warnings"]
            .iter()
            .enumerate()
        {
            let on_ = usize::from(self.filter) == i + 1;
            let style = if on_ {
                bold(Token::Act).bg(Token::Bg1.color())
            } else {
                on(Token::Fg3, Token::Bg1)
            };
            pen.text(name, style).gap(3);
        }
        let follow = if self.follow {
            format!("{} following", g.get(Glyph::Follow))
        } else {
            "f follows".to_owned()
        };
        Pen::new(buf, area.x, area.y, area.right().saturating_sub(1), g)
            .right(&follow, on(Token::Fg3, Token::Bg1));
        let lines = self.shown();
        let live = self.active() && self.dock.is_none();
        let height = usize::from(area.height.saturating_sub(1 + u16::from(live)));
        let end = if self.follow {
            lines.len()
        } else {
            (self.scroll + 1).min(lines.len())
        };
        let start = end.saturating_sub(height);
        for (i, line) in lines[start..end].iter().enumerate() {
            let y = area.y + 1 + u16::try_from(i).unwrap_or(0);
            let mut pen = Pen::new(buf, area.x + 2, y, area.right(), g);
            pen.text(&clock(line.at), fg(Token::Fg3));
            if line.label == Label::Err {
                pen.glyph(Glyph::Select, fg(Token::Warn));
            } else {
                pen.gap(1);
            }
            pen.cell(line.label.word(), 8, line.label.style());
            let color = if line.warning {
                Token::Warn
            } else if line.label == Label::Run {
                Token::Act
            } else {
                Token::Fg
            };
            pen.text(&line.text, fg(color));
        }
        if live {
            let y = area.y + 1 + u16::try_from(end - start).unwrap_or(0);
            let running = self
                .acts
                .values()
                .filter_map(|a| match a {
                    Act::Running(s) => Some(*s),
                    _ => None,
                })
                .max();
            let mut pen = Pen::new(buf, area.x + 2, y, area.right(), g);
            pen.glyph(Glyph::Spin(0), bold(Token::Act)).gap(1);
            let what = match running {
                Some(_) => "running".to_owned(),
                None => "working".to_owned(),
            };
            pen.text(
                &format!("{what} {} {}", g.get(Glyph::Dot), self.assumed.describe()),
                fg(Token::Fg2),
            );
        }
    }
}

/// The Ctrl-C confirm (D91): planrun ends as an interrupt ends it; resume goes on.
fn draw_end(buf: &mut Buffer, area: Rect, g: Glyphs) {
    crate::draw::fill(
        buf,
        Rect::new(area.x, area.y, area.width, 1),
        on(Token::Fg, Token::Bg1),
    );
    let mut pen = Pen::new(buf, area.x, area.y, area.right(), g);
    pen.glyph(Glyph::Band, on(Token::Warn, Token::Bg1)).gap(1);
    pen.glyph(Glyph::Waiting, bold(Token::Warn).bg(Token::Bg1.color()))
        .gap(1);
    pen.text("END PLANRUN", bold(Token::Warn).bg(Token::Bg1.color()))
        .gap(2);
    pen.text("ctrl+c", on(Token::Fg2, Token::Bg1));
    let lines = [
        "planrun ends now, as an interrupt ends it: every session's process tree is killed,",
        "the running block stops mid-way, and no run-end is written.",
        "`planrun resume` (or History, F4) goes on from the journal.",
    ];
    for (i, l) in lines.iter().enumerate() {
        Pen::new(
            buf,
            area.x + 2,
            area.y + 2 + u16::try_from(i).unwrap_or(0),
            area.right(),
            g,
        )
        .text(l, fg(Token::Fg2));
    }
    let y = area.bottom().saturating_sub(3);
    let mut pen = Pen::new(buf, area.x + 2, y, area.right(), g);
    pen.chip("y", "end planrun now", Token::Bg0)
        .gap(3)
        .chip("n", "keep running", Token::Bg0);
}

fn outcome_word(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Passed => "passed",
        Outcome::Confirmed => "confirmed",
        Outcome::Refused => "refused",
        Outcome::Accepted => "accepted",
        Outcome::Failed => "failed",
        Outcome::SkippedPolicy => "skipped",
        Outcome::SkippedUser => "skipped by you",
        Outcome::Acknowledged => "done",
        Outcome::Present => "present",
        Outcome::Missing => "missing",
        Outcome::Kept => "kept",
    }
}

/// `mm:ss` since the run started (D94).
pub fn clock(ms: u64) -> String {
    let s = ms / 1000;
    format!("{:02}:{:02}", s / 60, s % 60)
}
