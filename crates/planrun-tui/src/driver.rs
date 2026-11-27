//! The loop's core, shared by the terminal and the tests (decision D93): it feeds the app keys,
//! ticks and messages, carries out the effects the app asks for, and draws. The terminal adds
//! crossterm's events; the tests a `TestBackend` and scripted keys.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use ratatui::Terminal;
use ratatui::backend::Backend;

use crate::app::{App, Effect, Msg};
use crate::jobs::panic_message;
use crate::keys::Key;

/// The app, its terminal, and the channel its threads answer on.
pub struct Core<B: Backend> {
    app: App,
    terminal: Terminal<B>,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
}

impl<B: Backend> Core<B> {
    /// The core over `terminal`.
    pub fn new(app: App, terminal: Terminal<B>) -> Core<B> {
        let (tx, rx) = channel();
        Core {
            app,
            terminal,
            tx,
            rx,
        }
    }

    /// The app.
    pub fn app(&self) -> &App {
        &self.app
    }

    /// The terminal.
    pub fn terminal(&self) -> &Terminal<B> {
        &self.terminal
    }

    /// Whether the app is done.
    pub fn done(&self) -> bool {
        self.app.done()
    }

    /// Starts the app's first jobs.
    pub fn start(&mut self) {
        let effects = self.app.start();
        self.apply(effects);
    }

    /// Feeds a key.
    pub fn key(&mut self, key: Key) {
        let effects = self.app.key(key);
        self.apply(effects);
    }

    /// Lets time pass.
    pub fn tick(&mut self) {
        let effects = self.app.tick();
        self.apply(effects);
    }

    /// Hands the app every message that has arrived; true when there was one.
    pub fn pump(&mut self) -> bool {
        let mut any = false;
        while let Ok(msg) = self.rx.try_recv() {
            any = true;
            let effects = self.app.msg(msg);
            self.apply(effects);
        }
        any
    }

    /// Waits up to `timeout` for `until` to hold, handing the app messages as they arrive.
    pub fn wait(&mut self, timeout: Duration, until: impl Fn(&App) -> bool) -> bool {
        let end = Instant::now() + timeout;
        loop {
            self.pump();
            if until(&self.app) {
                return true;
            }
            let left = end.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            if let Ok(msg) = self.rx.recv_timeout(left.min(Duration::from_millis(20))) {
                let effects = self.app.msg(msg);
                self.apply(effects);
            }
        }
    }

    /// Draws the app.
    pub fn draw(&mut self) -> Result<(), B::Error> {
        let app = &self.app;
        self.terminal.draw(|frame| app.draw(frame.buffer_mut()))?;
        Ok(())
    }

    /// Forces a full redraw after the screen was left.
    pub fn redraw(&mut self) -> Result<(), B::Error> {
        self.terminal.clear()?;
        self.draw()
    }

    fn apply(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::Job(job) => {
                    let tx = self.tx.clone();
                    let ctx = self.app.context();
                    let failed = job.on_failure();
                    std::thread::spawn(move || {
                        let done = match catch_unwind(AssertUnwindSafe(|| job.run(&ctx))) {
                            Ok(done) => done,
                            Err(payload) => failed(format!(
                                "the job panicked: {}",
                                panic_message(payload.as_ref())
                            )),
                        };
                        let _ = tx.send(Msg::Done(done));
                    });
                }
                Effect::Quit => self.app.quit = true,
            }
        }
    }
}
