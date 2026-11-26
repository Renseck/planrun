//! The session protocol (decision D60): the run directory, the spawn with its environment
//! (D62), the reader threads, and the two-stream sentinel with the per-block number.

use std::collections::hash_map::RandomState;
use std::ffi::OsString;
use std::fs::{DirBuilder, OpenOptions};
use std::hash::{BuildHasher, Hasher};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{ChildStdin, Command};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use planrun_model::Shell;

use super::kill::Tree;
use super::{OutLine, RunStatus, ScriptRun, ShellError, ShellSession, Stream, bash, pwsh};

/// The environment variable that hands the run directory to the prologue, which removes it.
pub(super) const DIR_VAR: &str = "PLANRUN_SESSION_DIR";

/// Variables a session never inherits (D62): bash reads `BASH_ENV` (and `ENV`) even with
/// `--norc`, and the git variables would change what a plan's own git commands mean.
const REMOVED: [&str; 6] = [
    "BASH_ENV",
    "ENV",
    "GIT_LITERAL_PATHSPECS",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

/// What a session needs to start.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// The working directory the session starts in: the repository root.
    pub root: PathBuf,
    /// Variables added to planrun's own environment (tests: the fixture's git isolation).
    pub env: Vec<(OsString, OsString)>,
    /// How long one block may run; `None` waits forever (D65).
    pub timeout: Option<Duration>,
    /// Where the run directory is made: the OS temp directory outside the tests.
    pub temp: PathBuf,
}

/// How a shell is started and spoken to; one implementation per shell.
pub(super) trait Dialect: Send {
    /// `pwsh` or `bash`, for messages.
    fn name(&self) -> &'static str;
    /// The arguments of the shell process.
    fn args(&self, marker: &str) -> Vec<String>;
    /// The block file's extension.
    fn extension(&self) -> &'static str;
    /// The prologue as one ASCII stdin line that ends with the sentinel of block 0, or `None`
    /// when the session gets no stdin and its arguments carry the prologue.
    fn prologue(&self, marker: &str) -> Option<String>;
    /// The stdin line that runs block `n` from its file and prints its sentinel, or `None` when
    /// the shell picks the file up itself.
    fn block_line(&self, n: u64) -> Option<String>;
    /// The working directory as the shell reported it, as a path of this platform.
    fn cwd(&self, reported: String) -> PathBuf;
}

/// Finds the shell planrun runs for `shell` (D63) and checks it with a probe.
pub fn locate(shell: Shell) -> Result<PathBuf, ShellError> {
    match shell {
        Shell::Pwsh => pwsh::locate(),
        Shell::Bash => bash::locate(),
    }
}

/// One running shell.
pub struct Session {
    dialect: Box<dyn Dialect>,
    tree: Tree,
    stdin: Option<ChildStdin>,
    events: Receiver<Event>,
    dir: RunDir,
    marker: String,
    block: u64,
    timeout: Option<Duration>,
    alive: bool,
    /// The shell itself has exited and its tree was killed; the pipes close soon (D71).
    gone: bool,
}

/// How often `collect` checks whether the shell itself is still there (D71).
const SLICE: Duration = Duration::from_millis(100);

enum Event {
    Line {
        stream: Stream,
        bytes: Vec<u8>,
        newline: bool,
    },
    Eof(Stream),
}

/// What a sentinel line on stdout says.
struct Frame {
    kind: Kind,
    status: i32,
    errored: bool,
    cwd: String,
}

#[derive(PartialEq, Eq)]
enum Kind {
    Ok,
    Threw,
    Exit,
}

/// What reading one block's events found.
struct Collected {
    lines: Vec<OutLine>,
    frame: Option<Frame>,
    ended: bool,
}

impl Session {
    /// Starts `shell` at `config.root` and runs its prologue.
    pub fn start(shell: Shell, config: &SessionConfig) -> Result<Session, ShellError> {
        let program = locate(shell)?;
        let dialect: Box<dyn Dialect> = match shell {
            Shell::Pwsh => Box::new(pwsh::Pwsh),
            Shell::Bash => Box::new(bash::Bash),
        };
        let dir = RunDir::create(&config.temp)?;
        let marker = format!("__planrun_{}_", secret());
        let mut command = Command::new(&program);
        command
            .args(dialect.args(&marker))
            .current_dir(&config.root);
        for name in REMOVED {
            command.env_remove(name);
        }
        command
            .env("GIT_TERMINAL_PROMPT", "0")
            .envs(config.env.iter().map(|(k, v)| (k, v)))
            .env(DIR_VAR, dir.path());
        let prologue = dialect.prologue(&marker);
        let (tree, pipes) = Tree::spawn(&mut command, prologue.is_some())?;
        let (tx, events) = channel();
        read(Stream::Stdout, pipes.stdout, tx.clone());
        read(Stream::Stderr, pipes.stderr, tx);
        let mut session = Session {
            dialect,
            tree,
            stdin: pipes.stdin,
            events,
            dir,
            marker,
            block: 0,
            timeout: config.timeout,
            alive: true,
            gone: false,
        };
        session.handshake(prologue)?;
        Ok(session)
    }

    /// Whether the session can run another block.
    pub fn is_alive(&self) -> bool {
        self.alive
    }

    fn handshake(&mut self, prologue: Option<String>) -> Result<(), ShellError> {
        let name = self.dialect.name();
        let run = self.send(prologue, &mut |_| {})?;
        let output: Vec<&str> = run.lines.iter().map(|l| l.text.as_str()).collect();
        match run.status {
            RunStatus::Exited(0) if output.is_empty() => Ok(()),
            status => Err(ShellError::Start {
                shell: name,
                reason: format!("the prologue gave {status:?} with output {output:?}"),
            }),
        }
    }

    /// Writes `line`, if any, to the session's stdin and collects the block's events.
    fn send(
        &mut self,
        line: Option<String>,
        on_line: &mut dyn FnMut(&OutLine),
    ) -> io::Result<ScriptRun> {
        if let (Some(line), Some(stdin)) = (line, self.stdin.as_mut()) {
            let written = stdin
                .write_all(line.as_bytes())
                .and_then(|()| stdin.write_all(b"\n"))
                .and_then(|()| stdin.flush());
            if written.is_err() {
                return Ok(self.end(Vec::new(), RunStatus::SessionEnded(None)));
            }
        }
        let deadline = self.timeout.map(|t| Instant::now() + t);
        let collected = match self.collect(deadline, on_line) {
            Ok(collected) => collected,
            Err((lines, status)) => return Ok(self.end(lines, status)),
        };
        Ok(self.finish(collected))
    }

    /// Reads events until both sentinels of the current block arrived, or a stream ended.
    fn collect(
        &mut self,
        deadline: Option<Instant>,
        on_line: &mut dyn FnMut(&OutLine),
    ) -> Result<Collected, (Vec<OutLine>, RunStatus)> {
        let mut c = Collected {
            lines: Vec::new(),
            frame: None,
            ended: false,
        };
        let (mut out_done, mut err_done) = (false, false);
        while !(out_done && err_done) {
            let event = match self.next(deadline) {
                Ok(event) => event,
                Err(RecvTimeoutError::Timeout) => return Err((c.lines, RunStatus::TimedOut)),
                Err(RecvTimeoutError::Disconnected) => {
                    c.ended = true;
                    break;
                }
            };
            let (stream, bytes, newline) = match event {
                Event::Eof(Stream::Stdout) => {
                    (out_done, c.ended) = (true, true);
                    continue;
                }
                Event::Eof(Stream::Stderr) => {
                    (err_done, c.ended) = (true, true);
                    continue;
                }
                Event::Line {
                    stream,
                    bytes,
                    newline,
                } => (stream, bytes, newline),
            };
            let text = String::from_utf8_lossy(&bytes).into_owned();
            let Some(at) = text.find(&self.marker) else {
                emit(&mut c.lines, on_line, stream, text, newline);
                continue;
            };
            if at > 0 {
                emit(&mut c.lines, on_line, stream, text[..at].to_owned(), false);
            }
            let rest = &text[at + self.marker.len()..];
            match (stream, self.parse(rest, stream)) {
                (_, Err(reason)) => return Err((c.lines, RunStatus::Broken(reason.into()))),
                (Stream::Stdout, Ok(frame)) => {
                    c.frame = frame;
                    out_done = true;
                }
                (Stream::Stderr, Ok(_)) => err_done = true,
            }
        }
        Ok(c)
    }

    /// The next event, in slices: when the shell itself has exited without its sentinel while
    /// a descendant still holds its pipes, the tree is killed so that the pipes close, and the
    /// block ends as `SessionEnded(None)` (D71).
    fn next(&mut self, deadline: Option<Instant>) -> Result<Event, RecvTimeoutError> {
        loop {
            let wait = deadline.map_or(SLICE, |d| {
                d.saturating_duration_since(Instant::now()).min(SLICE)
            });
            match self.events.recv_timeout(wait) {
                Err(RecvTimeoutError::Timeout) => {}
                other => return other,
            }
            if deadline.is_some_and(|d| Instant::now() >= d) {
                return Err(RecvTimeoutError::Timeout);
            }
            if !self.gone && self.tree.exited().unwrap_or(false) {
                self.gone = true;
                let _ = self.tree.kill();
            }
        }
    }

    /// Parses what follows the marker: `<n>__`, then on stdout ` <kind> <status> <errored>
    /// <cwd hex>` and nothing else, on stderr nothing at all.
    fn parse(&self, rest: &str, stream: Stream) -> Result<Option<Frame>, String> {
        let rest = rest.strip_suffix('\r').unwrap_or(rest);
        let Some((number, fields)) = rest.split_once("__") else {
            return Err(format!("a sentinel without its block number: {rest:?}"));
        };
        if number.parse::<u64>().ok() != Some(self.block) {
            return Err(format!(
                "a sentinel for block {number:?} during block {}",
                self.block
            ));
        }
        if stream == Stream::Stderr {
            if !fields.is_empty() {
                return Err(format!(
                    "unexpected text after the stderr sentinel: {fields:?}"
                ));
            }
            return Ok(None);
        }
        let parts: Vec<&str> = fields.split(' ').collect();
        let [empty, kind, status, errored, cwd] = parts.as_slice() else {
            return Err(format!("unexpected sentinel fields: {fields:?}"));
        };
        let kind = match *kind {
            "ok" => Kind::Ok,
            "threw" => Kind::Threw,
            "exit" => Kind::Exit,
            other => return Err(format!("an unknown sentinel kind {other:?}")),
        };
        let (Ok(status), true, Some(cwd)) = (
            status.parse::<i32>(),
            empty.is_empty() && matches!(*errored, "0" | "1"),
            unhex(cwd),
        ) else {
            return Err(format!("unexpected sentinel fields: {fields:?}"));
        };
        Ok(Some(Frame {
            kind,
            status,
            errored: *errored == "1",
            cwd,
        }))
    }

    /// The result of a block whose events were all read.
    fn finish(&mut self, c: Collected) -> ScriptRun {
        let Some(frame) = c.frame else {
            return self.end(c.lines, RunStatus::SessionEnded(None));
        };
        let cwd = Some(self.dialect.cwd(frame.cwd));
        let status = match frame.kind {
            Kind::Exit => RunStatus::SessionEnded(Some(frame.status)),
            _ if c.ended => RunStatus::SessionEnded(None),
            Kind::Threw => RunStatus::Threw,
            Kind::Ok => RunStatus::Exited(frame.status),
        };
        let mut run = match status {
            RunStatus::SessionEnded(_) => self.end(c.lines, status),
            status => ScriptRun {
                status,
                errored: false,
                lines: c.lines,
                cwd: None,
            },
        };
        run.errored = frame.errored;
        run.cwd = cwd;
        run
    }

    /// Ends the session: kills its tree and marks it dead.
    fn end(&mut self, lines: Vec<OutLine>, status: RunStatus) -> ScriptRun {
        self.alive = false;
        let _ = self.tree.kill();
        let _ = self.tree.wait();
        ScriptRun {
            status,
            errored: false,
            lines,
            cwd: None,
        }
    }
}

impl ShellSession for Session {
    fn run(&mut self, script: &str, on_line: &mut dyn FnMut(&OutLine)) -> io::Result<ScriptRun> {
        if !self.alive {
            return Err(io::Error::other(format!(
                "the {} session has ended",
                self.dialect.name()
            )));
        }
        self.block += 1;
        if let Err(e) = self.stage(script) {
            self.end(Vec::new(), RunStatus::SessionEnded(None));
            let block = self.block;
            return Err(io::Error::new(
                e.kind(),
                format!("block {block} could not be written, so the session was ended: {e}"),
            ));
        }
        let line = self.dialect.block_line(self.block);
        self.send(line, on_line)
    }
}

impl Session {
    /// Writes the current block's file: `b<n>.tmp`, then renamed whole. An error leaves the
    /// shell waiting for a block that never comes, so the caller ends the session (D71).
    fn stage(&self, script: &str) -> io::Result<()> {
        let staged = self.dir.path().join(format!("b{}.tmp", self.block));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)?;
        file.write_all(script.as_bytes())?;
        drop(file);
        let name = format!("b{}.{}", self.block, self.dialect.extension());
        std::fs::rename(&staged, self.dir.path().join(name))
    }
}

fn emit(
    lines: &mut Vec<OutLine>,
    on_line: &mut dyn FnMut(&OutLine),
    stream: Stream,
    text: String,
    newline: bool,
) {
    let line = OutLine {
        stream,
        text,
        newline,
    };
    on_line(&line);
    lines.push(line);
}

/// One reader thread per stream, detached: a build server that inherited the pipe may keep it
/// open long after the session is gone, and nothing waits for the thread.
fn read(stream: Stream, pipe: impl Read + Send + 'static, tx: Sender<Event>) {
    std::thread::spawn(move || {
        let mut reader = BufReader::new(pipe);
        loop {
            let mut bytes = Vec::new();
            match reader.read_until(b'\n', &mut bytes) {
                Ok(0) | Err(_) => {
                    let _ = tx.send(Event::Eof(stream));
                    return;
                }
                Ok(_) => {}
            }
            let newline = bytes.last() == Some(&b'\n');
            if newline {
                bytes.pop();
            }
            let line = Event::Line {
                stream,
                bytes,
                newline,
            };
            if tx.send(line).is_err() {
                return;
            }
        }
    });
}

/// Lowercase or uppercase hex of UTF-8 bytes back to text; `None` when it is not hex.
fn unhex(hex: &str) -> Option<String> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    let bytes: Option<Vec<u8>> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok())
        .collect();
    Some(String::from_utf8_lossy(&bytes?).into_owned())
}

/// 128 random bits as 32 hex digits.
fn secret() -> String {
    let half = |salt: u64| {
        let mut hasher = RandomState::new().build_hasher();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        hasher.write_u128(nanos);
        hasher.write_u32(std::process::id());
        hasher.write_u64(salt);
        hasher.finish()
    };
    format!("{:016x}{:016x}", half(1), half(2))
}

/// The run directory: created fresh (it must not exist), private on Unix, removed on drop.
struct RunDir(PathBuf);

impl RunDir {
    fn create(temp: &Path) -> io::Result<RunDir> {
        let path = temp.join(format!("planrun-shell-{}", &secret()[..16]));
        let builder = DirBuilder::new();
        #[cfg(unix)]
        let builder = {
            use std::os::unix::fs::DirBuilderExt;
            let mut builder = builder;
            builder.mode(0o700);
            builder
        };
        builder.create(&path)?;
        Ok(RunDir(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for RunDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_utf8_and_refuses_odd_lengths() {
        assert_eq!(unhex("52656ec3a9").as_deref(), Some("René"));
        assert_eq!(unhex("").as_deref(), Some(""));
        assert_eq!(unhex("5"), None);
        assert_eq!(unhex("zz"), None);
    }

    #[test]
    fn secrets_differ() {
        let (a, b) = (secret(), secret());
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
    }
}
