//! Persistent pwsh and bash sessions (docs/architecture.md §6.3; decisions D24, D60–D65).
//!
//! One [`Session`] is one shell process. Each block is written to a file in a run directory
//! under the OS temp directory and run by one ASCII line on the session's stdin: dot-sourced in
//! pwsh, `source`d at the top level in bash, with stdin taken away from the block. A sentinel
//! with a per-run secret and the block's number follows it on stdout and on stderr; the block
//! ends when both arrive.

mod bash;
mod kill;
mod pwsh;
mod session;

use std::io;
use std::path::PathBuf;

pub use session::{Session, SessionConfig, locate};

/// Installs planrun's interrupt handling once, from the main thread, before the first session
/// starts (D73). On Unix SIGINT, SIGTERM and SIGHUP kill every session's process group and then
/// end planrun as the signal would; on Windows Ctrl-C ends planrun and the job objects kill the
/// sessions.
pub fn install_interrupt_handler() {
    kill::install_interrupts();
}

/// Ends planrun now with exit code `code`, as Ctrl-C would (D73, D91): every session's process
/// tree is killed, no destructor runs and no `run-end` is written, so `resume` goes on from the
/// journal. For a UI that reads Ctrl-C as a key (the TUI's raw mode); restore the terminal
/// first.
pub fn end_now(code: i32) -> ! {
    kill::end_now(code)
}

/// While the guard lives, Ctrl-C does not end planrun: the `[o]` shell runs in the foreground
/// and gets it instead (D73).
pub struct InterruptsIgnored(());

impl InterruptsIgnored {
    /// Starts ignoring Ctrl-C until the guard is dropped.
    pub fn new() -> InterruptsIgnored {
        kill::ignore_interrupts(true);
        InterruptsIgnored(())
    }
}

impl Default for InterruptsIgnored {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for InterruptsIgnored {
    fn drop(&mut self) {
        kill::ignore_interrupts(false);
    }
}

/// Which stream a line came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// One line of a block's output. Order within a stream is exact; order across the two streams
/// is the order of arrival, best effort.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutLine {
    /// The stream.
    pub stream: Stream,
    /// The text without its LF (a CR before it stays), UTF-8 decoded lossily.
    pub text: String,
    /// False for the block's last output on this stream when it did not end with a newline.
    pub newline: bool,
}

/// How a block ended (decision D61).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunStatus {
    /// It ran to its end. pwsh: `$LASTEXITCODE`, reset to 0 before the block (`exit n` ends only
    /// the block); bash: `$?` of the `source`.
    Exited(i32),
    /// pwsh only: a terminating error ended the block.
    Threw,
    /// The session ended during the block: bash `exit n` (with its code), or the shell went away
    /// without its sentinel (`None`). The session is dead.
    SessionEnded(Option<i32>),
    /// The block ran past the session's timeout; its process tree was killed.
    TimedOut,
    /// The protocol broke (a sentinel for another block, unexpected fields). The session is
    /// dead.
    Broken(Box<str>),
}

/// The result of running one shell block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRun {
    /// How it ended.
    pub status: RunStatus,
    /// pwsh: `$Error[0]` is another object after the block than before it (a non-terminating
    /// error was recorded, decision D61). Always false in bash. Never changes `status`.
    pub errored: bool,
    /// Its output, both streams.
    pub lines: Vec<OutLine>,
    /// The session's working directory after the block, when the sentinel reported it.
    pub cwd: Option<PathBuf>,
}

/// A long-lived shell: variables, the working directory and functions persist across blocks.
pub trait ShellSession: Send {
    /// Runs `script`, streaming each output line to `on_line` as it arrives. An error means the
    /// block was not started (its file could not be written, or the session is dead).
    fn run(&mut self, script: &str, on_line: &mut dyn FnMut(&OutLine)) -> io::Result<ScriptRun>;
}

/// Why a session could not be had.
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    /// The shell is not installed, or not the one planrun accepts (decision D63).
    #[error("{shell} is not available: {reason}")]
    Unavailable {
        /// `pwsh` or `bash`.
        shell: &'static str,
        /// What the probe found.
        reason: String,
    },
    /// The shell started but its prologue did not answer as expected.
    #[error("the {shell} session did not start: {reason}")]
    Start {
        /// `pwsh` or `bash`.
        shell: &'static str,
        /// What went wrong.
        reason: String,
    },
    /// Spawning, the run directory, or the process tree failed.
    #[error(transparent)]
    Io(#[from] io::Error),
}
