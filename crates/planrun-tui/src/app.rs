//! The app: all of the TUI's state, the keys and messages that change it, and the effects it
//! asks the driver for. It never blocks: git, shells and the network are background jobs or the
//! runner's worker thread (D90). Drawing reads the state and changes nothing.

use std::path::PathBuf;
use std::sync::Arc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::chrome::{self, MIN, Nav};
use crate::draw::fill;
use crate::glyph::{Glyph, Glyphs};
use crate::history::{self, History};
use crate::inspect::{Assumed, Did, Inspect};
use crate::jobs::{Bg, Context, Done, Job, UpdateCheck, UpdateState};
use crate::keys::Key;
use crate::palette::{Cmd, Command, Palette, Pick};
use crate::plans::{self, Labels, Plans};
use crate::preview::ResumeView;
use crate::run::{Follow, Inspector, Run};
use crate::theme::{Token, base, on};
use crate::worker::{ObserverFactory, RunMsg, RunRequest};

/// Milliseconds since the Unix epoch; the tests fix it.
pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

/// The system clock.
pub fn system_clock() -> Clock {
    Arc::new(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
    })
}

/// How often the plans directory is scanned and `lock.live` probed.
const POLL_MS: u64 = 2000;

/// Which view the body shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// F2.
    Plans,
    /// F3: the plan view, in Run when there is a run, in Inspect otherwise.
    Plan,
    /// F4.
    History,
    /// The resume preview of a run in History.
    Resume,
}

/// What lies over the view.
pub enum Overlay {
    /// F1, `?`.
    Help,
    /// `ctrl+k`, `ctrl+p`.
    Palette(Palette),
}

/// What the app asks the driver to do.
pub enum Effect {
    /// Run a job on a thread of its own.
    Job(Job),
    /// Start the runner on its worker thread.
    Run(RunRequest),
    /// The `[o]` round trip: run the shell, then reply (D90).
    Shell(std::sync::mpsc::Sender<()>),
    /// Ctrl-C, confirmed: end planrun as an interrupt does (D91).
    EndNow,
    /// Stop.
    Quit,
}

/// What reaches the app from its threads.
pub enum Msg {
    /// A background job's result.
    Done(Done),
    /// The runner's worker.
    Run(RunMsg),
}

/// The TUI's state.
pub struct App {
    pub(crate) ctx: Arc<Context>,
    pub(crate) g: Glyphs,
    pub(crate) clock: Clock,
    pub(crate) view: View,
    pub(crate) plans: Plans,
    pub(crate) inspect: Option<Inspect>,
    pub(crate) run: Option<Run>,
    pub(crate) history: History,
    pub(crate) resume: Option<ResumeView>,
    pub(crate) overlay: Option<Overlay>,
    pub(crate) observers: Vec<ObserverFactory>,
    pub(crate) update: UpdateState,
    pub(crate) update_check: Option<UpdateCheck>,
    /// A run of another planrun holds the lock, with `lock.pid`'s pid (D90).
    pub(crate) live: Option<Option<u32>>,
    pub(crate) next_poll: u64,
    pub(crate) frame: u8,
    pub(crate) quit: bool,
}

impl App {
    /// The app for the repository `ctx` describes; `update` is the update check, if any.
    pub fn new(ctx: Context, g: Glyphs, clock: Clock, update: Option<UpdateCheck>) -> App {
        App {
            ctx: Arc::new(ctx),
            g,
            clock,
            view: View::Plans,
            plans: Plans::default(),
            inspect: None,
            run: None,
            history: History::default(),
            resume: None,
            overlay: None,
            observers: Vec::new(),
            update: UpdateState::Unchecked,
            update_check: update,
            live: None,
            next_poll: 0,
            frame: 0,
            quit: false,
        }
    }

    /// The context jobs run in.
    pub fn context(&self) -> Arc<Context> {
        self.ctx.clone()
    }

    /// Whether the app is done.
    pub fn done(&self) -> bool {
        self.quit
    }

    /// The Plans view's state.
    pub fn plans(&self) -> &Plans {
        &self.plans
    }

    /// What the update check found.
    pub fn update(&self) -> &UpdateState {
        &self.update
    }

    /// The plan view in Inspect, when a plan is open.
    pub fn inspect(&self) -> Option<&Inspect> {
        self.inspect.as_ref()
    }

    /// History's state.
    pub fn history(&self) -> &History {
        &self.history
    }

    /// The resume preview, when it is open.
    pub fn resume_view(&self) -> Option<&ResumeView> {
        self.resume.as_ref()
    }

    /// The open palette, if any.
    pub fn palette(&self) -> Option<&Palette> {
        match &self.overlay {
            Some(Overlay::Palette(p)) => Some(p),
            _ => None,
        }
    }

    /// The run, when there is one.
    pub fn run(&self) -> Option<&Run> {
        self.run.as_ref()
    }

    /// Adds a journal observer to the next run (the tests' crash seam, D93).
    pub fn observe_next_run(&mut self, factory: ObserverFactory) {
        self.observers.push(factory);
    }

    fn now(&self) -> u64 {
        (self.clock)()
    }

    /// The jobs to start with.
    pub fn start(&mut self) -> Vec<Effect> {
        self.next_poll = self.now() + POLL_MS;
        let mut effects = vec![Effect::Job(Job::Scan), Effect::Job(Job::Live)];
        if let Some(check) = self.update_check.take() {
            effects.push(Effect::Job(Job::Update(check)));
        }
        effects
    }

    /// Time passes: the spinner turns, and every two seconds the plans directory is scanned and
    /// `lock.live` probed.
    pub fn tick(&mut self) -> Vec<Effect> {
        let now = self.now();
        self.frame = u8::try_from((now / 250) % 4).unwrap_or(0);
        if let Some(run) = self.run.as_mut() {
            run.frame = self.frame;
        }
        if now < self.next_poll {
            return Vec::new();
        }
        self.next_poll = now + POLL_MS;
        vec![Effect::Job(Job::Scan), Effect::Job(Job::Live)]
    }

    /// A message from a thread.
    pub fn msg(&mut self, msg: Msg) -> Vec<Effect> {
        match msg {
            Msg::Done(done) => self.done_job(done),
            Msg::Run(run) => self.run_msg(run),
        }
    }

    fn done_job(&mut self, done: Done) -> Vec<Effect> {
        match done {
            Done::Scanned(files) => self
                .plans
                .scanned(files)
                .into_iter()
                .map(Effect::Job)
                .collect(),
            Done::Checked { path, stamp, file } => {
                self.plans.checked(&path, stamp, file);
                Vec::new()
            }
            Done::Live(live) => {
                self.live = live.unwrap_or(None);
                Vec::new()
            }
            Done::Update(state) => {
                self.update = state;
                Vec::new()
            }
            Done::DryRun {
                path,
                fingerprint,
                result,
            } => {
                let Some(inspect) = self.inspect.as_mut() else {
                    return Vec::new();
                };
                if inspect.path == path && *inspect.plan().source.fingerprint == *fingerprint {
                    inspect.dry = match result {
                        Ok(dry) => Bg::Current(dry),
                        Err(e) => Bg::Failed(e),
                    };
                }
                Vec::new()
            }
            Done::Journals(runs) => {
                let n = runs.as_ref().map_or(0, Vec::len);
                self.history.runs = Some(runs);
                self.history.selected = self.history.selected.min(n.saturating_sub(1));
                Vec::new()
            }
            Done::Preview { journal, result } => {
                if let Some(view) = self.resume.as_mut().filter(|v| v.entry.journal == journal) {
                    view.data = match result {
                        Ok(data) => Bg::Current(data),
                        Err(e) => Bg::Failed(e),
                    };
                }
                Vec::new()
            }
            Done::WorkTree(result) => {
                let now = self.now();
                let Some(run) = self.run.as_mut() else {
                    return Vec::new();
                };
                match result {
                    Ok((files, untracked)) => run.inspector = Inspector::Tree(files, untracked),
                    Err(e) => run.status(now, &format!("[d] could not read the diff: {e}")),
                }
                Vec::new()
            }
            Done::Changes { task, files } => {
                if let Some(run) = self.run.as_mut() {
                    run.diffed(task, files);
                }
                Vec::new()
            }
        }
    }

    fn run_msg(&mut self, msg: RunMsg) -> Vec<Effect> {
        let now = self.now();
        let Some(run) = self.run.as_mut() else {
            return Vec::new();
        };
        match msg {
            RunMsg::Started => run.started(),
            RunMsg::Record(record) => {
                if let Follow::Changes(task, paths) = run.record(now, &record) {
                    return vec![Effect::Job(Job::Changes { task, paths })];
                }
            }
            RunMsg::Wrote(_) => {}
            RunMsg::Status(line) => run.status(now, &line),
            RunMsg::Output(line) => run.output(now, &line),
            RunMsg::Ask(question, reply) => run.ask(question, reply),
            RunMsg::View(at, reply) => {
                run.inspector = Inspector::Action((at.task, at.step, at.action));
                let _ = reply.send(());
            }
            RunMsg::Diff(reply) => {
                let _ = reply.send(());
                return vec![Effect::Job(Job::WorkTree)];
            }
            RunMsg::Shell(reply) => return vec![Effect::Shell(reply)],
            RunMsg::Ended(result) => {
                run.ended(now, result);
                return vec![Effect::Job(Job::Scan), Effect::Job(Job::Journals)];
            }
        }
        Vec::new()
    }

    /// A key.
    pub fn key(&mut self, key: Key) -> Vec<Effect> {
        if let Some(overlay) = self.overlay.as_mut() {
            return match overlay {
                Overlay::Help => {
                    if matches!(key, Key::Esc | Key::F(1) | Key::Char('?') | Key::Char('q')) {
                        self.overlay = None;
                    }
                    Vec::new()
                }
                Overlay::Palette(p) => match p.key(key) {
                    Pick::Open => Vec::new(),
                    Pick::Closed => {
                        self.overlay = None;
                        Vec::new()
                    }
                    Pick::Chosen(cmd) => {
                        self.overlay = None;
                        self.command(cmd)
                    }
                },
            };
        }
        let typing = self.typing();
        match key {
            Key::F(1) => {
                self.overlay = Some(Overlay::Help);
                return Vec::new();
            }
            Key::Char('?') if !typing => {
                self.overlay = Some(Overlay::Help);
                return Vec::new();
            }
            Key::Ctrl('k') => {
                self.overlay = Some(Overlay::Palette(Palette::new(false, self.commands())));
                return Vec::new();
            }
            Key::Ctrl('p') => {
                let targets = self.targets();
                if !targets.is_empty() {
                    self.overlay = Some(Overlay::Palette(Palette::new(true, targets)));
                }
                return Vec::new();
            }
            _ => {}
        }
        match key {
            Key::F(2) => {
                self.view = View::Plans;
                return Vec::new();
            }
            Key::F(3) if self.inspect.is_some() || self.run.is_some() => {
                self.view = View::Plan;
                return Vec::new();
            }
            Key::F(4) => {
                self.view = View::History;
                return vec![Effect::Job(Job::Journals)];
            }
            Key::Ctrl('c') if !self.running() => return vec![Effect::Quit],
            Key::Ctrl('c') => {
                if let Some(run) = self.run.as_mut() {
                    if run.ending {
                        return vec![Effect::EndNow];
                    }
                    run.ending = true;
                    self.view = View::Plan;
                }
                return Vec::new();
            }
            _ => {}
        }
        match self.view {
            View::Plans => self.plans_key(key),
            View::Plan if self.run.is_some() => self.run_key(key),
            View::Plan => self.plan_key(key),
            View::History => self.history_key(key),
            View::Resume => self.resume_key(key),
        }
    }

    fn history_key(&mut self, key: Key) -> Vec<Effect> {
        match self.history.key(key) {
            history::Did::Preview(entry) => self.history_key_preview(entry),
            history::Did::Back => {
                self.view = View::Plans;
                Vec::new()
            }
            history::Did::None => Vec::new(),
        }
    }

    /// Opens the resume preview of `entry`.
    fn history_key_preview(&mut self, entry: history::RunEntry) -> Vec<Effect> {
        let job = Job::Preview {
            journal: entry.journal.clone(),
            plan: entry.plan.clone(),
            now: self.now(),
        };
        self.resume = Some(ResumeView {
            entry,
            data: Bg::Running,
        });
        self.view = View::Resume;
        vec![Effect::Job(job)]
    }

    fn resume_key(&mut self, key: Key) -> Vec<Effect> {
        let Some(view) = self.resume.as_mut() else {
            self.view = View::History;
            return Vec::new();
        };
        match key {
            Key::Char('g') => {
                view.data = std::mem::replace(&mut view.data, Bg::Running).stale();
                vec![Effect::Job(Job::Preview {
                    journal: view.entry.journal.clone(),
                    plan: view.entry.plan.clone(),
                    now: (self.clock)(),
                })]
            }
            Key::Enter => self.start_resume(),
            Key::Esc | Key::Char('q') => {
                self.view = View::History;
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    /// Starts `Runner::resume` from the preview: with the journaled policy and range, as
    /// `planrun resume` without flags (D77); it checks everything again itself.
    fn start_resume(&mut self) -> Vec<Effect> {
        if self.running() {
            return Vec::new();
        }
        let Some(data) = self.resume.as_ref().and_then(|v| v.data.value()) else {
            return Vec::new();
        };
        let run = &data.analysis.run;
        let assumed = assumed_from(&run.auto, run.run_expected_failures);
        let plan = &data.parsed.plan;
        let from = run
            .from
            .as_deref()
            .and_then(|id| plan.tasks.iter().position(|t| &*t.id.raw == id));
        let (path, parsed) = (data.path.clone(), data.parsed.clone());
        self.run = Some(Run::new(
            path.clone(),
            parsed.clone(),
            assumed.clone(),
            true,
            self.now(),
        ));
        self.view = View::Plan;
        vec![Effect::Run(RunRequest {
            path,
            parsed,
            auto: assumed.auto,
            expected: assumed.expected,
            from,
            resume: true,
            observers: std::mem::take(&mut self.observers),
        })]
    }

    /// Whether keys go to a text field: a typed answer, or the Plans filter.
    fn typing(&self) -> bool {
        let typed = self.view == View::Plan
            && self.run.as_ref().is_some_and(|r| {
                r.focus == 2 && r.dock.as_ref().is_some_and(|d| d.question.typed.is_some())
            });
        typed || (self.view == View::Plans && self.plans.editing)
    }

    /// The registry: what can be done from here, as the keys do it.
    fn commands(&self) -> Vec<Command> {
        let mut out = Vec::new();
        let command = |group, title: String, key: &str, cmd| Command {
            group,
            title,
            key: key.to_owned(),
            cmd,
        };
        if let Some(inspect) = self.inspect.as_ref().filter(|_| !self.running()) {
            let name = inspect
                .path
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
            let policy = inspect.assumed.describe();
            out.push(command(
                "PLAN",
                format!("Run {name} with policy {policy}"),
                "R",
                Cmd::Run(None),
            ));
            for (t, task) in inspect.plan().tasks.iter().enumerate().skip(1) {
                out.push(command(
                    "PLAN",
                    format!(
                        "Run from task {} {}",
                        task.id.raw,
                        inspect.plan().text(&task.title)
                    ),
                    "shift+R",
                    Cmd::Run(Some(t)),
                ));
            }
            out.push(command(
                "PLAN",
                "Assume a policy…".to_owned(),
                "a",
                Cmd::Policy,
            ));
        }
        for entry in &self.plans.entries {
            if entry.check.value().is_some_and(|f| f.parsed.is_some()) {
                out.push(command(
                    "PLANS",
                    format!("Inspect {}", entry.name),
                    "enter",
                    Cmd::Inspect(entry.path.clone()),
                ));
            }
        }
        if let Some(Ok(runs)) = &self.history.runs {
            for run in runs.iter().filter(|r| r.resumable()) {
                out.push(command(
                    "HISTORY",
                    format!("Resume {}: preview", run.name()),
                    "R",
                    Cmd::Preview(run.clone()),
                ));
            }
        }
        out.push(command(
            "VIEW",
            "Plans".to_owned(),
            "F2",
            Cmd::Key(Key::F(2)),
        ));
        out.push(command(
            "VIEW",
            "Plan".to_owned(),
            "F3",
            Cmd::Key(Key::F(3)),
        ));
        out.push(command(
            "VIEW",
            "History".to_owned(),
            "F4",
            Cmd::Key(Key::F(4)),
        ));
        out.push(command(
            "VIEW",
            "Help".to_owned(),
            "F1",
            Cmd::Key(Key::F(1)),
        ));
        if self.run.is_some() {
            for (n, what) in [
                (1, "everything"),
                (2, "events only"),
                (3, "run output only"),
                (4, "verdicts only"),
                (5, "warnings only"),
            ] {
                let key = char::from(b'0' + n);
                out.push(command(
                    "VIEW",
                    format!("Console: show {what}"),
                    &key.to_string(),
                    Cmd::Key(Key::Char(key)),
                ));
            }
        }
        out.push(command(
            "VIEW",
            "Check the plans again".to_owned(),
            "c",
            Cmd::Key(Key::Char('c')),
        ));
        if !self.running() {
            out.push(command(
                "VIEW",
                "Quit".to_owned(),
                "q",
                Cmd::Key(Key::Ctrl('c')),
            ));
        }
        out
    }

    /// `ctrl+p`: the plan's files and actions.
    fn targets(&self) -> Vec<Command> {
        let plan = match (&self.run, &self.inspect) {
            (Some(run), _) => run.plan(),
            (None, Some(inspect)) => inspect.plan(),
            _ => return Vec::new(),
        };
        let mut out = Vec::new();
        for (t, task) in plan.tasks.iter().enumerate() {
            for (s, step) in task.steps.iter().enumerate() {
                for (a, action) in step.actions.iter().enumerate() {
                    if matches!(action.kind, planrun_model::ActionKind::Note { .. }) {
                        continue;
                    }
                    let group = if planrun_exec::handler::file_path(&action.kind).is_some() {
                        "FILES"
                    } else {
                        "ACTIONS"
                    };
                    let number = step
                        .number
                        .map_or_else(|| "·".to_owned(), |n| n.to_string());
                    let title = format!(
                        "{} {}",
                        self.g.get(crate::tree::kind_glyph(action)),
                        crate::tree::label(plan, action)
                    );
                    out.push(Command {
                        group,
                        title,
                        key: format!("{} step {number}", task.id.raw),
                        cmd: Cmd::Select(crate::tree::Node::Action(t, s, a)),
                    });
                }
            }
        }
        out.sort_by_key(|c| c.group != "FILES");
        out
    }

    /// Does what a palette command says, as its key does it.
    fn command(&mut self, cmd: Cmd) -> Vec<Effect> {
        match cmd {
            Cmd::Inspect(path) => self.open(path),
            Cmd::Run(from) => self.start_run(from),
            Cmd::Policy => {
                if let Some(inspect) = self.inspect.as_mut() {
                    self.view = View::Plan;
                    inspect.picking = true;
                }
                Vec::new()
            }
            Cmd::Preview(entry) => {
                self.view = View::History;
                self.history_key_preview(entry)
            }
            Cmd::Key(Key::Ctrl('c')) => vec![Effect::Quit],
            Cmd::Key(key) => self.key(key),
            Cmd::Select(node) => {
                self.view = View::Plan;
                match (&mut self.run, &mut self.inspect) {
                    (Some(run), _) => {
                        run.tree.select(node);
                        run.focus = 0;
                    }
                    (None, Some(inspect)) => {
                        inspect.tree.select(node);
                        inspect.focus = crate::inspect::Focus::Tree;
                    }
                    _ => {}
                }
                Vec::new()
            }
        }
    }

    /// Whether the runner is running.
    pub fn running(&self) -> bool {
        self.run.as_ref().is_some_and(Run::active)
    }

    fn run_key(&mut self, key: Key) -> Vec<Effect> {
        let Some(run) = self.run.as_mut() else {
            return Vec::new();
        };
        if run.ending {
            match key {
                Key::Char('y') => return vec![Effect::EndNow],
                Key::Char('n') | Key::Esc => run.ending = false,
                _ => {}
            }
            return Vec::new();
        }
        if let Some(dock) = run.dock.as_mut()
            && run.focus == 2
            && !matches!(key, Key::Tab | Key::BackTab)
        {
            if let Some(reply) = dock.key(key) {
                dock.answer(reply);
                run.dock = None;
                run.inspector = Inspector::Changes;
            }
            return Vec::new();
        }
        if run.key(key) {
            return Vec::new();
        }
        if matches!(key, Key::Esc | Key::Char('q')) {
            self.view = View::Plans;
        }
        Vec::new()
    }

    fn plan_key(&mut self, key: Key) -> Vec<Effect> {
        let Some(inspect) = self.inspect.as_mut() else {
            self.view = View::Plans;
            return Vec::new();
        };
        match inspect.key(key) {
            Did::Back => self.view = View::Plans,
            Did::Unused if key == Key::Char('q') => self.view = View::Plans,
            Did::Run(from) => return self.start_run(from),
            _ => {}
        }
        Vec::new()
    }

    /// Runs the plan in Inspect with the policy Inspect assumes (D84).
    fn start_run(&mut self, from: Option<usize>) -> Vec<Effect> {
        let Some(inspect) = self.inspect.as_ref() else {
            return Vec::new();
        };
        if self.running() {
            return Vec::new();
        }
        let path = inspect.path.clone();
        let parsed = inspect.parsed.clone();
        let assumed = inspect.assumed.clone();
        self.run = Some(Run::new(
            path.clone(),
            parsed.clone(),
            assumed.clone(),
            false,
            self.now(),
        ));
        self.view = View::Plan;
        vec![Effect::Run(RunRequest {
            path,
            parsed,
            auto: assumed.auto,
            expected: assumed.expected,
            from,
            resume: false,
            observers: std::mem::take(&mut self.observers),
        })]
    }

    /// Opens `path` in Inspect and starts its dry run; an ended run is put away.
    fn open(&mut self, path: PathBuf) -> Vec<Effect> {
        let Some(parsed) = self.plans.parsed(&path) else {
            return Vec::new();
        };
        if self.running() {
            self.view = View::Plan;
            return Vec::new();
        }
        self.run = None;
        self.view = View::Plan;
        let same = self
            .inspect
            .as_ref()
            .is_some_and(|i| i.path == path && Arc::ptr_eq(&i.parsed, &parsed));
        if same {
            return Vec::new();
        }
        self.inspect = Some(Inspect::new(path.clone(), parsed.clone()));
        vec![Effect::Job(Job::DryRun { path, parsed })]
    }

    fn plans_key(&mut self, key: Key) -> Vec<Effect> {
        match self.plans.key(key) {
            plans::Outcome::Check(jobs) => jobs.into_iter().map(Effect::Job).collect(),
            plans::Outcome::Inspect(path) | plans::Outcome::Run(path) => self.open(path),
            plans::Outcome::Unused if key == Key::Char('q') && !self.running() => {
                vec![Effect::Quit]
            }
            _ => Vec::new(),
        }
    }

    fn labels(&self) -> Labels {
        Labels {
            plans: self.ctx.plans_label.clone(),
            root: self.ctx.root.clone(),
        }
    }

    /// Draws the whole screen.
    pub fn draw(&self, buf: &mut Buffer) {
        let area = buf.area;
        fill(buf, area, base());
        if area.width < MIN.0 || area.height < MIN.1 {
            // A dock that opens while the terminal is too small is still answerable: the
            // notice turns into the dock alone (§11).
            match self
                .run
                .as_ref()
                .and_then(|r| r.dock.as_ref().map(|d| (r, d)))
            {
                Some((run, dock)) => dock.draw(buf, area, run.plan(), true, self.g),
                None => chrome::too_small(buf, area, self.g),
            }
            return;
        }
        let footer_rows = if area.height >= 30 { 2 } else { 1 };
        let body = Rect::new(
            area.x,
            area.y + 1,
            area.width,
            area.height - 1 - footer_rows,
        );
        let mut crumbs = vec![self.ctx.repo.clone()];
        let (keys, state) = match (self.view, &self.run, &self.inspect) {
            (View::Plan, Some(run), _) => {
                crumbs.extend(run.crumbs(self.g));
                (run.footer(), run.state(self.g))
            }
            (View::Plan, None, Some(inspect)) => {
                crumbs.extend(inspect.crumbs(self.g));
                (inspect.footer(), inspect.state(self.g))
            }
            (View::History, _, _) => {
                crumbs.push("history".to_owned());
                let dir = self.labels().relative(&self.ctx.journal_dir());
                (self.history.footer(), self.history.state(&dir))
            }
            (View::Resume, _, _) if self.resume.is_some() => {
                crumbs.push("history".to_owned());
                if let Some(view) = &self.resume {
                    crumbs.push(view.entry.name());
                }
                crumbs.push("resume preview".to_owned());
                let keys = self
                    .resume
                    .as_ref()
                    .map(ResumeView::footer)
                    .unwrap_or_default();
                (keys, ResumeView::state())
            }
            _ => {
                crumbs.push("plans".to_owned());
                (self.plans.footer(), self.state())
            }
        };
        chrome::header(buf, area, &crumbs, &state, self.g);
        match (self.view, &self.run, &self.inspect) {
            (View::Plan, Some(run), _) => run.draw(buf, body, self.g),
            (View::Plan, None, Some(inspect)) => {
                let rel = self.labels().relative(&inspect.path);
                inspect.draw(buf, body, &rel, self.g);
            }
            (View::History, _, _) => self.history.draw(buf, body, self.live, self.g),
            (View::Resume, _, _) if self.resume.is_some() => {
                if let Some(view) = &self.resume {
                    view.draw(buf, body, self.g);
                }
            }
            _ => self.plans.draw(buf, body, &self.labels(), self.g),
        }
        if footer_rows == 2 {
            chrome::keys(buf, area, area.bottom() - 2, &keys, self.g);
        }
        let active = if matches!(self.overlay, Some(Overlay::Help)) {
            Nav::Help
        } else {
            self.nav()
        };
        chrome::nav(
            buf,
            area,
            area.bottom() - 1,
            active,
            &self.ctx,
            &self.update,
            self.g,
        );
        match &self.overlay {
            Some(Overlay::Help) => crate::help::draw(buf, body, &keys, self.g),
            Some(Overlay::Palette(p)) => {
                p.draw(buf, body, self.g);
                let keys = [
                    ("↑↓".to_owned(), "select".to_owned()),
                    ("enter".to_owned(), "run command".to_owned()),
                    ("esc".to_owned(), "close".to_owned()),
                ];
                if footer_rows == 2 {
                    chrome::keys(buf, area, area.bottom() - 2, &keys, self.g);
                }
            }
            None => {}
        }
    }

    fn nav(&self) -> Nav {
        match self.view {
            View::Plans => Nav::Plans,
            View::Plan => Nav::Plan,
            View::History | View::Resume => Nav::History,
        }
    }

    /// The header's right side: what planrun is doing, and other runs.
    fn state(&self) -> Vec<(String, Style)> {
        let bar = |t: Token| on(t, Token::Bg1);
        let mut out = if self.running() {
            vec![
                (format!("{} ", self.g.get(Glyph::Running)), bar(Token::Act)),
                ("running".to_owned(), bar(Token::Fg)),
            ]
        } else {
            vec![
                (format!("{} ", self.g.get(Glyph::Pending)), bar(Token::Fg3)),
                ("idle".to_owned(), bar(Token::Fg2)),
            ]
        };
        let dot = format!("  {}  ", self.g.get(Glyph::Dot));
        match self.live {
            Some(_) if self.running() => {}
            Some(pid) => {
                let pid =
                    pid.map_or_else(|| "running".to_owned(), |p| format!("running in pid {p}"));
                out.push((dot, bar(Token::Fg3)));
                out.push((pid, bar(Token::Fg2)));
            }
            None => {
                let n = self.plans.interrupted();
                if n > 0 {
                    let s = if n == 1 { "" } else { "s" };
                    out.push((dot, bar(Token::Fg3)));
                    out.push((format!("{n} interrupted run{s}"), bar(Token::Warn)));
                }
            }
        }
        out
    }
}

/// The assumed policy of a resumed run, from its journal's `--auto` words.
pub fn assumed_from(auto: &[String], expected: bool) -> Assumed {
    let auto = planrun_exec::policy::Policy::parse_auto(&auto.join(",")).unwrap_or_default();
    Assumed { auto, expected }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
