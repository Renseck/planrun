//! The app: all of the TUI's state, the keys and messages that change it, and the effects it
//! asks the driver for. It never blocks: git, shells and the network are background jobs or the
//! runner's worker thread (D90). Drawing reads the state and changes nothing.

use std::sync::Arc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::chrome::{self, MIN, Nav};
use crate::draw::fill;
use crate::glyph::{Glyph, Glyphs};
use crate::inspect::{Did, Inspect};
use crate::jobs::{Bg, Context, Done, Job, UpdateCheck, UpdateState};
use crate::keys::Key;
use crate::plans::{self, Labels, Plans};
use crate::theme::{Token, base, on};

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
    /// F3: the plan view.
    Plan,
}

/// What the app asks the driver to do.
pub enum Effect {
    /// Run a job on a thread of its own.
    Job(Job),
    /// Stop.
    Quit,
}

/// What reaches the app from its threads.
pub enum Msg {
    /// A background job's result.
    Done(Done),
}

/// The TUI's state.
pub struct App {
    pub(crate) ctx: Arc<Context>,
    pub(crate) g: Glyphs,
    pub(crate) clock: Clock,
    pub(crate) view: View,
    pub(crate) plans: Plans,
    pub(crate) inspect: Option<Inspect>,
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

    /// The jobs to start with.
    pub fn start(&mut self) -> Vec<Effect> {
        self.next_poll = (self.clock)() + POLL_MS;
        let mut effects = vec![Effect::Job(Job::Scan), Effect::Job(Job::Live)];
        if let Some(check) = self.update_check.take() {
            effects.push(Effect::Job(Job::Update(check)));
        }
        effects
    }

    /// Time passes: the spinner turns, and every two seconds the plans directory is scanned and
    /// `lock.live` probed.
    pub fn tick(&mut self) -> Vec<Effect> {
        let now = (self.clock)();
        self.frame = u8::try_from((now / 250) % 4).unwrap_or(0);
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
        }
    }

    /// A key.
    pub fn key(&mut self, key: Key) -> Vec<Effect> {
        match key {
            Key::F(2) => {
                self.view = View::Plans;
                return Vec::new();
            }
            Key::F(3) if self.inspect.is_some() => {
                self.view = View::Plan;
                return Vec::new();
            }
            Key::Ctrl('c') if self.view == View::Plans => return vec![Effect::Quit],
            _ => {}
        }
        match self.view {
            View::Plans => self.plans_key(key),
            View::Plan => self.plan_key(key),
        }
    }

    fn plan_key(&mut self, key: Key) -> Vec<Effect> {
        let Some(inspect) = self.inspect.as_mut() else {
            self.view = View::Plans;
            return Vec::new();
        };
        match inspect.key(key) {
            Did::Back => self.view = View::Plans,
            Did::Unused if key == Key::Char('q') => self.view = View::Plans,
            _ => {}
        }
        Vec::new()
    }

    /// Opens `path` in Inspect and starts its dry run.
    fn open(&mut self, path: std::path::PathBuf) -> Vec<Effect> {
        let Some(parsed) = self.plans.parsed(&path) else {
            return Vec::new();
        };
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
            plans::Outcome::Unused if key == Key::Char('q') => vec![Effect::Quit],
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
            chrome::too_small(buf, area, self.g);
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
        let (keys, state) = match (self.view, &self.inspect) {
            (View::Plan, Some(inspect)) => {
                crumbs.extend(inspect.crumbs(self.g));
                (inspect.footer(), inspect.state(self.g))
            }
            _ => {
                crumbs.push("plans".to_owned());
                (self.plans.footer(), self.state())
            }
        };
        chrome::header(buf, area, &crumbs, &state, self.g);
        match (self.view, &self.inspect) {
            (View::Plan, Some(inspect)) => {
                let rel = self.labels().relative(&inspect.path);
                inspect.draw(buf, body, &rel, self.g);
            }
            _ => self.plans.draw(buf, body, &self.labels(), self.g),
        }
        if footer_rows == 2 {
            chrome::keys(buf, area, area.bottom() - 2, &keys, self.g);
        }
        chrome::nav(
            buf,
            area,
            area.bottom() - 1,
            self.nav(),
            &self.ctx,
            &self.update,
            self.g,
        );
    }

    fn nav(&self) -> Nav {
        match self.view {
            View::Plans => Nav::Plans,
            View::Plan => Nav::Plan,
        }
    }

    /// The header's right side: what planrun is doing, and other runs.
    fn state(&self) -> Vec<(String, Style)> {
        let bar = |t: Token| on(t, Token::Bg1);
        let mut out = vec![
            (format!("{} ", self.g.get(Glyph::Pending)), bar(Token::Fg3)),
            ("idle".to_owned(), bar(Token::Fg2)),
        ];
        let dot = format!("  {}  ", self.g.get(Glyph::Dot));
        match self.live {
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
