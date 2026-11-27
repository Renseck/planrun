//! The journal (docs/architecture.md §6.6; decisions D18, D76): one JSONL file per plan at
//! `<git-dir>/planrun/<plan file stem>.jsonl`, one record per line, appended by one writer.
//!
//! - Every line carries the schema version and the time. One `write_all` per record (the JSON
//!   and its `\n`); no buffer spans two records.
//! - `step-prepared`, every `action-start` and `run-end` are followed by `sync_data`: they are
//!   the records whose loss after a power cut would change what resume does. The rest reach the
//!   OS only, which a killed process cannot lose.
//! - A line counts only when it ends in `\n`: a last line without one is dropped, even if it
//!   parses, and before the first append the file is cut back to its last `\n`. A bad line that
//!   is not the last one is an error, never skipped.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use planrun_model::RepoPath;
use serde::{Deserialize, Serialize};

use crate::git::BlobId;
pub use crate::git::Head;
use crate::transaction::{PreparedStep, StepRecord, WriteAhead};

/// The schema version every line carries.
pub const SCHEMA: u32 = 1;

/// Where an action is: the task's id, the step's and the action's positions from 0.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct At {
    /// The task id (`P1-T2`).
    pub task: String,
    /// The step's position in the task.
    pub step: usize,
    /// The action's position in the step (for a file group: its first action).
    pub action: usize,
}

/// A path and the blob id git would commit for it (`None`: absent).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathBlob {
    /// The path.
    pub path: RepoPath,
    /// Its filtered blob id on disk, or `None` when it is absent.
    pub blob: Option<BlobId>,
}

/// How an action ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    /// Everything stated was checked and holds.
    Passed,
    /// The user confirmed what prose or comments say (or an unchecked exit).
    Confirmed,
    /// The user answered no to a confirmation.
    Refused,
    /// The user accepted a mismatch.
    Accepted,
    /// It failed and the user stopped, or retried (a later attempt follows).
    Failed,
    /// Skipped by policy (D67).
    SkippedPolicy,
    /// Skipped by the user.
    SkippedUser,
    /// A Manual, acknowledged.
    Acknowledged,
    /// A `Generated` path exists.
    Present,
    /// A `Generated` path is missing.
    Missing,
    /// A script that ran before an interruption, kept without running it again (resume, D77).
    Kept,
}

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum End {
    /// Every task of the run is done.
    Completed,
    /// It stopped: the user quit, a step failed, a check stopped it.
    Stopped,
}

/// One journal record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "kebab-case")]
pub enum Record {
    /// A run or a resume starts.
    Run {
        /// The plan's path: repository-relative, or canonical when outside the repository.
        plan: String,
        /// The SHA-256 of the plan's bytes.
        fingerprint: String,
        /// The format id.
        format: String,
        /// The repository root.
        root: String,
        /// HEAD at the start.
        head: Head,
        /// The `--auto` classes.
        auto: Vec<String>,
        /// `--run-expected-failures`.
        run_expected_failures: bool,
        /// `--from`.
        from: Option<String>,
        /// `--to`.
        to: Option<String>,
        /// True for `resume`.
        resume: bool,
        /// planrun's version.
        planrun: String,
    },
    /// A task starts.
    TaskStart {
        /// Its id.
        task: String,
        /// HEAD.
        head: Head,
    },
    /// The write-ahead record of a step's file actions (D53), before any is written.
    StepPrepared {
        /// The group's first action.
        at: At,
        /// What will be written.
        step: StepRecord,
    },
    /// Every write of the step is done.
    StepWritten {
        /// The group.
        at: At,
    },
    /// The step's commit failed and was rolled back; `unrestored` remain changed.
    StepFailed {
        /// The group.
        at: At,
        /// The error.
        error: String,
        /// Paths rollback could not restore.
        unrestored: Vec<RepoPath>,
    },
    /// Resume found the step done on disk; nothing is written again (D77).
    StepVerified {
        /// The group.
        at: At,
    },
    /// Resume put the step back from its pre-images (`[r]`).
    StepRestored {
        /// The group.
        at: At,
    },
    /// The user accepted an Unknown step as done (`[a]`, a typed `yes`).
    StepAccepted {
        /// The group.
        at: At,
    },
    /// A script, commit, Manual or Generated check starts.
    ActionStart {
        /// Where.
        at: At,
        /// `script`, `commit`, `manual`, `generated`.
        kind: String,
        /// The policy's decision (`Auto` included).
        decision: String,
        /// HEAD before it.
        head: Head,
        /// For a Commit: the task's paths as they are on disk now, which the commit must hold.
        expect: Vec<PathBlob>,
    },
    /// It ended.
    ActionEnd {
        /// Where.
        at: At,
        /// How.
        outcome: Outcome,
        /// The block's status (`Exited(0)`), for a script or commit.
        status: Option<String>,
        /// pwsh's `errored`.
        errored: bool,
        /// The session's working directory after it.
        cwd: Option<String>,
        /// The last lines of its output.
        tail: Vec<String>,
        /// HEAD after it.
        head: Head,
    },
    /// The user answered a question.
    Answer {
        /// The action it was about, if any.
        at: Option<At>,
        /// Which question (`confirm`, `mismatch`, `dirt`, `outside`, `session`, `gate`, …).
        question: String,
        /// The answer.
        answer: String,
    },
    /// The user opened the commit gate with a typed `yes` (D75).
    GateOverride {
        /// The task.
        task: String,
        /// Every reason the gate was closed.
        reasons: Vec<String>,
    },
    /// HEAD as read after something that may move it (`[o]`, D80).
    HeadSeen {
        /// HEAD.
        head: Head,
    },
    /// A shell session started.
    Session {
        /// `pwsh` or `bash`.
        shell: String,
        /// `started` or `restarted`.
        reason: String,
    },
    /// A task is done.
    TaskEnd {
        /// Its id.
        task: String,
        /// HEAD.
        head: Head,
    },
    /// The run ends.
    RunEnd {
        /// How.
        end: End,
        /// Why it stopped.
        reason: Option<String>,
    },
}

impl Record {
    /// Whether the record is followed by `sync_data` (D76).
    fn durable(&self) -> bool {
        matches!(
            self,
            Record::StepPrepared { .. } | Record::ActionStart { .. } | Record::RunEnd { .. }
        )
    }
}

/// One line as stored: the schema version, the time, the record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    /// The schema version.
    pub v: u32,
    /// Milliseconds since the Unix epoch.
    pub ms: u64,
    /// The record.
    #[serde(flatten)]
    pub record: Record,
}

/// Why a journal could not be read.
#[derive(Debug, thiserror::Error)]
pub enum JournalError {
    /// Reading the file failed.
    #[error("{path}: {source}")]
    Io {
        /// The journal.
        path: String,
        /// The error.
        #[source]
        source: io::Error,
    },
    /// A line that is not the last one is not a record of this schema.
    #[error("{path}:{line}: not a journal record: {reason}")]
    Corrupt {
        /// The journal.
        path: String,
        /// Its 1-based line number.
        line: usize,
        /// What is wrong.
        reason: String,
    },
}

/// The journal's directory: `<git-dir>/planrun`.
pub fn dir(git_dir: &Path) -> PathBuf {
    git_dir.join("planrun")
}

/// The journal of the plan whose file stem is `stem`.
pub fn path_for(git_dir: &Path, stem: &str) -> PathBuf {
    dir(git_dir).join(format!("{stem}.jsonl"))
}

/// Reads every complete line of the journal at `path`; an absent file is empty. A last line
/// without its `\n` is dropped (a torn write); any other bad line is an error.
pub fn read(path: &Path) -> Result<Vec<Line>, JournalError> {
    let name = path.display().to_string();
    let io = |source| JournalError::Io {
        path: name.clone(),
        source,
    };
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(io(e)),
    };
    let complete = match bytes.iter().rposition(|&b| b == b'\n') {
        Some(end) => &bytes[..=end],
        None => &[][..],
    };
    let mut lines = Vec::new();
    for (i, raw) in complete.split_inclusive(|&b| b == b'\n').enumerate() {
        let raw = &raw[..raw.len() - 1];
        let corrupt = |reason: String| JournalError::Corrupt {
            path: name.clone(),
            line: i + 1,
            reason,
        };
        let line: Line = serde_json::from_slice(raw).map_err(|e| corrupt(e.to_string()))?;
        if line.v != SCHEMA {
            return Err(corrupt(format!(
                "schema {} (this planrun reads {SCHEMA})",
                line.v
            )));
        }
        lines.push(line);
    }
    Ok(lines)
}

/// What a journal observer sees: a record that is now with the OS (and synced, when durable),
/// or the number of paths a step's commit has written so far.
#[derive(Debug, Clone, Copy)]
pub enum Seam<'a> {
    /// A record was appended.
    Appended(&'a Record),
    /// A step's commit wrote its n-th path.
    Wrote(usize),
}

/// A callback that sees every [`Seam`]: planrun-cli's fault seam for its e2e tests (D82), and
/// the TUI's view of a live run (D90).
pub type Observer = Box<dyn FnMut(Seam<'_>)>;

/// The appending writer.
pub struct Journal {
    file: File,
    path: PathBuf,
    observers: Vec<Observer>,
}

impl Journal {
    /// Opens the journal at `path` for appending, creating it and its directory. A torn last
    /// line is cut off first, so the next record starts on a line of its own.
    pub fn open(path: &Path) -> io::Result<Journal> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let keep = bytes.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
        if keep != bytes.len() {
            file.set_len(keep as u64)?;
            file.sync_data()?;
        }
        file.seek(SeekFrom::End(0))?;
        Ok(Journal {
            file,
            path: path.to_path_buf(),
            observers: Vec::new(),
        })
    }

    /// Adds `observer`: it is called after every append and every write of a step's commit,
    /// after the observers added before it.
    pub fn observe(&mut self, observer: Observer) {
        self.observers.push(observer);
    }

    fn seam(&mut self, seam: Seam<'_>) {
        for observer in &mut self.observers {
            observer(seam);
        }
    }

    /// The journal's path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends `record` as one line in one write; durable records are synced (D76).
    pub fn append(&mut self, record: Record) -> io::Result<()> {
        let durable = record.durable();
        let line = Line {
            v: SCHEMA,
            ms: now_ms(),
            record,
        };
        let mut bytes = serde_json::to_vec(&line).map_err(io::Error::other)?;
        bytes.push(b'\n');
        self.file.write_all(&bytes)?;
        if durable {
            self.file.sync_data()?;
        }
        self.seam(Seam::Appended(&line.record));
        Ok(())
    }

    /// The write-ahead recorder for the step at `at` (D53): `record` appends `step-prepared`.
    pub fn ahead<'j>(&'j mut self, at: &'j At) -> StepAhead<'j> {
        StepAhead { journal: self, at }
    }
}

/// [`WriteAhead`] over the journal, for one step.
pub struct StepAhead<'j> {
    journal: &'j mut Journal,
    at: &'j At,
}

impl WriteAhead for StepAhead<'_> {
    fn record(&mut self, step: &PreparedStep) -> io::Result<()> {
        self.journal.append(Record::StepPrepared {
            at: self.at.clone(),
            step: step.record(),
        })
    }

    fn after_write(&mut self, written: usize) {
        self.journal.seam(Seam::Wrote(written));
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

#[cfg(test)]
#[path = "journal_tests.rs"]
mod tests;
