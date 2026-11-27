//! `planrun tui` on a real terminal (decision D94): stdin and stdout must be terminals; the
//! repository is the one of the current directory (D88); the plans directory defaults to
//! `docs/plans` under its root. One loop polls crossterm's events and drains the app's channel
//! between polls; there is no input thread, which would read the keys meant for the `[o]` shell.
//! Interrupts are installed once, before the terminal is set up; a panic restores the terminal
//! only when it is the render thread's.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::thread::ThreadId;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event};

use planrun_exec::shell::install_interrupt_handler;
use planrun_exec::workspace::Workspace;

use crate::app::{App, system_clock};
use crate::driver::Core;
use crate::glyph::Glyphs;
use crate::jobs::{Context, UpdateCheck};
use crate::keys;

/// How long one poll for a key waits before the app's channel is drained again.
const POLL: Duration = Duration::from_millis(50);

/// What planrun-cli hands the TUI.
pub struct Host {
    /// The current directory: its repository is the target.
    pub dir: PathBuf,
    /// The plans directory as given (relative to `dir`), or `None` for `docs/plans` under the
    /// repository's root.
    pub plans: Option<PathBuf>,
    /// planrun's version.
    pub version: String,
    /// The update check.
    pub update: Option<UpdateCheck>,
}

/// The default plans directory, under the repository's root (D94).
pub const DEFAULT_PLANS: &str = "docs/plans";

/// Builds the context for the repository of `dir`.
pub fn context(
    dir: &Path,
    plans: Option<&Path>,
    version: &str,
    env: Vec<(std::ffi::OsString, std::ffi::OsString)>,
) -> Result<Context, String> {
    let ws = Workspace::open(dir, env.clone()).map_err(|e| e.to_string())?;
    let git_dir = ws.git().git_dir().map_err(|e| e.to_string())?;
    let root = ws.root().to_path_buf();
    let plans_dir = match plans {
        Some(p) if p.is_absolute() => p.to_path_buf(),
        Some(p) => dir.join(p),
        None => root.join(DEFAULT_PLANS),
    };
    let canonical = std::fs::canonicalize(&plans_dir).unwrap_or_else(|_| plans_dir.clone());
    let plans_label = match canonical.strip_prefix(&root) {
        Ok(rel) if rel.as_os_str().is_empty() => ".".to_owned(),
        Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
        Err(_) => plans_dir.display().to_string(),
    };
    let repo = root.file_name().map_or_else(
        || root.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    Ok(Context {
        target: root.display().to_string(),
        repo,
        root,
        git_dir,
        plans_dir,
        plans_label,
        version: version.to_owned(),
        env,
    })
}

/// Runs the TUI until it quits. `Err` is the message for a usage problem: not a terminal, not a
/// repository, or a terminal that could not be set up.
pub fn run(host: Host) -> Result<(), String> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(
            "planrun tui needs a terminal on stdin and stdout; `planrun run` is the line version"
                .to_owned(),
        );
    }
    let ctx = context(&host.dir, host.plans.as_deref(), &host.version, Vec::new())?;
    let app = App::new(ctx, Glyphs::from_env(), system_clock(), host.update);
    install_interrupt_handler();
    let report = std::panic::take_hook();
    let terminal = ratatui::init();
    install_panic_hook(report);
    let result = drive(Core::new(app, terminal));
    ratatui::restore();
    result
}

/// What a panic does to the terminal (D94).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnPanic {
    /// The render thread's: restore the terminal, then report as Rust does.
    Restore,
    /// Another thread's (the runner's, a job's): leave the terminal and print nothing; the
    /// thread catches the panic and the app shows it as that work's failure.
    Leave,
}

/// What a panic on `current` does, when `render` owns the terminal.
pub fn on_panic(render: ThreadId, current: ThreadId) -> OnPanic {
    if render == current {
        OnPanic::Restore
    } else {
        OnPanic::Leave
    }
}

/// The hook a panic report goes through.
type Report = Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Sync + Send>;

/// Replaces ratatui's hook, which would restore the terminal on every thread's panic, caught or
/// not: `report` (the hook before `ratatui::init`) runs only for the render thread's.
fn install_panic_hook(report: Report) {
    let render = std::thread::current().id();
    std::panic::set_hook(Box::new(move |info| {
        if on_panic(render, std::thread::current().id()) == OnPanic::Restore {
            ratatui::restore();
            report(info);
        }
    }));
}

fn drive<B: ratatui::backend::Backend>(mut core: Core<B>) -> Result<(), String> {
    let term = |e: B::Error| format!("the terminal: {e}");
    let io = |e: std::io::Error| format!("the terminal: {e}");
    core.start();
    core.draw().map_err(term)?;
    while !core.done() {
        core.pump();
        if event::poll(POLL).map_err(io)? {
            match event::read().map_err(io)? {
                Event::Key(ev) => {
                    if let Some(key) = keys::key(&ev) {
                        core.key(key);
                    }
                }
                Event::Resize(..) => core.redraw().map_err(term)?,
                _ => {}
            }
        }
        core.tick();
        core.pump();
        core.draw().map_err(term)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_render_thread_s_panic_restores_the_terminal() {
        let render = std::thread::current().id();
        let other = std::thread::spawn(|| std::thread::current().id())
            .join()
            .unwrap_or_else(|_| panic!("the thread panicked"));
        assert_eq!(on_panic(render, render), OnPanic::Restore);
        assert_eq!(on_panic(render, other), OnPanic::Leave);
    }
}
