//! A step's file actions as one transaction (decisions D23, D53, D78): **prepare** validates
//! them on a virtual tree seeded from the disk, stores the pre-images and builds every
//! post-image; **commit** checks that the disk is still what prepare saw, hands the step to a
//! write-ahead recorder, and then writes it with the one writer, rolling back on any error.

use std::path::PathBuf;

use planrun_model::{Action, ActionKind, PlanSource, RepoPath};
use serde::{Deserialize, Serialize};

use crate::finding::Finding;
use crate::git::BlobId;
use crate::handler::{HandlerSet, file_path};
use crate::tree::{State, VirtualTree};
use crate::workspace::{Workspace, WorkspaceError, remove_dirs, temp_name};

/// What a path was before the step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreImage {
    /// It did not exist.
    Absent,
    /// Its exact bytes, stored with `hash-object -w --no-filters` (D52).
    Stored(BlobId),
}

/// What a path becomes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostImage {
    /// These working-tree bytes.
    Bytes(Vec<u8>),
    /// Removed.
    Delete,
}

/// One path the step changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedWrite {
    /// The path, and everything resume needs about it.
    pub record: WriteRecord,
    /// After: the bytes, or a delete.
    pub post: PostImage,
}

/// A planned write without its bytes: what the journal keeps (D76) and resume checks the disk
/// against (D14, D77).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteRecord {
    /// The path.
    pub path: RepoPath,
    /// Before.
    pub pre: PreImage,
    /// The index mode before (`100644`), when the path was present.
    pub pre_mode: Option<String>,
    /// The exact-bytes id of the post-image (`--no-filters`, not stored); `None` for a delete.
    pub post: Option<BlobId>,
    /// The post-image's blob id as git will see it (filtered); `None` for a delete.
    pub post_blob: Option<BlobId>,
    /// The index mode after (a patch's mode lines included); `None` for a delete.
    pub mode: Option<String>,
    /// The directories the write will create, from the root, outermost first.
    pub creates_dirs: Vec<String>,
    /// The writer's temp file name in the target's directory, chosen here so that resume can
    /// remove exactly it (D77).
    pub temp: String,
}

/// A validated step, ready to commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedStep {
    /// The paths, in the order their first action names them.
    pub writes: Vec<PlannedWrite>,
    /// The paths a `Delete` with `via_git` removes from the index after every write.
    pub index_removals: Vec<RepoPath>,
}

/// A prepared step without its bytes: the write-ahead record (D53, D76).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepRecord {
    /// The planned writes.
    pub writes: Vec<WriteRecord>,
    /// The index removals.
    pub index_removals: Vec<RepoPath>,
}

impl PreparedStep {
    /// What the journal records before the step is written.
    pub fn record(&self) -> StepRecord {
        StepRecord {
            writes: self.writes.iter().map(|w| w.record.clone()).collect(),
            index_removals: self.index_removals.clone(),
        }
    }
}

/// Records a prepared step before it is written. P5's recorder is the journal; a real run
/// without that record is not allowed (D53).
pub trait WriteAhead {
    /// Stores `step` durably; an error stops the commit before anything is written.
    fn record(&mut self, step: &PreparedStep) -> std::io::Result<()>;

    /// Called after each path is written, with how many are written so far. The default does
    /// nothing; planrun-cli's fault seam aborts here in its e2e tests (D82).
    fn after_write(&mut self, _written: usize) {}
}

/// Why a commit failed, and what rollback could not undo.
#[derive(Debug, thiserror::Error)]
#[error("{cause}{}", unrestored_note(.unrestored))]
pub struct CommitError {
    /// The first failure.
    pub cause: CommitCause,
    /// Paths rollback could not restore; empty when it restored everything.
    pub unrestored: Vec<RepoPath>,
}

/// The first failure of a commit.
#[derive(Debug, thiserror::Error)]
pub enum CommitCause {
    /// Between prepare and commit something changed these paths (or created a directory the
    /// step would create); nothing was recorded or written (D78).
    #[error("changed since the step was prepared: {}", list(.0))]
    Changed(Vec<RepoPath>),
    /// The write-ahead record failed; nothing was written.
    #[error("the write-ahead record failed: {0}")]
    Record(#[source] std::io::Error),
    /// A write, a delete or the index update failed.
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
}

fn list(paths: &[RepoPath]) -> String {
    let names: Vec<&str> = paths.iter().map(RepoPath::as_str).collect();
    names.join(", ")
}

fn unrestored_note(paths: &[RepoPath]) -> String {
    if paths.is_empty() {
        return String::new();
    }
    format!("; rollback could not restore {}", list(paths))
}

/// Validates `actions` (a step's file actions) on a fresh virtual tree, stores the pre-image of
/// every path they change in the repository's object database, and builds the post-images.
/// The first finding stops it; then nothing but pre-image objects was written.
pub fn prepare(
    ws: &Workspace,
    source: &PlanSource,
    handlers: &HandlerSet,
    actions: &[&Action],
) -> Result<PreparedStep, Finding> {
    let mut tree = VirtualTree::new(ws).map_err(|e| Finding::workspace(&e, None, None))?;
    let mut touched: Vec<RepoPath> = Vec::new();
    let mut index_removals = Vec::new();
    for action in actions {
        let span = Some(action.span);
        let path = file_path(&action.kind)
            .ok_or_else(|| Finding::error("internal", span, None, "not a file action"))?;
        let handler = handlers
            .find(action)
            .ok_or_else(|| Finding::error("internal", span, Some(path), "no handler"))?;
        handler.simulate(action, source, &mut tree)?;
        if !touched.contains(path) {
            touched.push(path.clone());
        }
        if matches!(action.kind, ActionKind::Delete { via_git: true, .. }) {
            index_removals.push(path.clone());
        }
    }
    let mut writes = Vec::new();
    for path in touched {
        if let Some(write) = planned(ws, &mut tree, path)? {
            writes.push(write);
        }
    }
    Ok(PreparedStep {
        writes,
        index_removals,
    })
}

fn git_finding(path: &RepoPath, e: impl std::fmt::Display) -> Finding {
    Finding::error("git", None, Some(path), e.to_string())
}

/// The planned write of one touched path; `None` when it ends as it began (absent).
fn planned(
    ws: &Workspace,
    tree: &mut VirtualTree<'_>,
    path: RepoPath,
) -> Result<Option<PlannedWrite>, Finding> {
    let on_disk = ws
        .read(&path)
        .map_err(|e| Finding::workspace(&e, None, Some(&path)))?;
    let (post, post_blob, mode) = match tree.state(&path, None)?.clone() {
        State::Present { blob, mode, .. } => (
            PostImage::Bytes(tree.bytes(&path, None)?),
            Some(blob),
            Some(mode),
        ),
        _ if on_disk.is_none() => return Ok(None),
        _ => (PostImage::Delete, None, None),
    };
    let git = ws.git();
    let (pre, pre_mode) = match &on_disk {
        None => (PreImage::Absent, None),
        Some(bytes) => {
            let id = git
                .hash_object(&path, bytes, false, true, &[])
                .map_err(|e| git_finding(&path, e))?;
            let mode = git
                .index_entry(&path, &[])
                .map_err(|e| git_finding(&path, e))?
                .map_or_else(|| "100644".to_owned(), |(mode, _)| mode);
            (PreImage::Stored(id), Some(mode))
        }
    };
    let post_raw = match &post {
        PostImage::Bytes(bytes) => Some(
            git.hash_object(&path, bytes, false, false, &[])
                .map_err(|e| git_finding(&path, e))?,
        ),
        PostImage::Delete => None,
    };
    let resolved = ws
        .resolve(&path)
        .map_err(|e| Finding::workspace(&e, None, Some(&path)))?;
    let segments: Vec<&str> = path.as_str().split('/').collect();
    let creates_dirs = (resolved.existing.min(segments.len() - 1)..segments.len() - 1)
        .map(|i| segments[..=i].join("/"))
        .collect();
    Ok(Some(PlannedWrite {
        record: WriteRecord {
            path,
            pre,
            pre_mode,
            post: post_raw,
            post_blob,
            mode,
            creates_dirs,
            temp: temp_name(),
        },
        post,
    }))
}

/// Writes `step`. First the disk is checked against what prepare saw (D78): a difference
/// writes and records nothing. Then the write-ahead record, then every path in order, then the
/// index removals in one call. On an error every path already written is restored from its
/// pre-image, created files and directories are removed, and the error names what could not be
/// restored.
pub fn commit(
    ws: &Workspace,
    step: &PreparedStep,
    ahead: &mut dyn WriteAhead,
) -> Result<(), CommitError> {
    let fail = |cause| CommitError {
        cause,
        unrestored: Vec::new(),
    };
    let changed = changed_since_prepare(ws, step).map_err(|e| fail(e.into()))?;
    if !changed.is_empty() {
        return Err(fail(CommitCause::Changed(changed)));
    }
    ahead
        .record(step)
        .map_err(|e| fail(CommitCause::Record(e)))?;
    let mut done: Vec<(&PlannedWrite, Vec<PathBuf>)> = Vec::new();
    for write in &step.writes {
        let record = &write.record;
        let result = match &write.post {
            PostImage::Bytes(bytes) => {
                let executable = record.mode.as_deref().map(|m| m == "100755");
                ws.write_as(&record.path, bytes, &record.temp, executable)
            }
            PostImage::Delete => ws.remove(&record.path).map(|_| Vec::new()),
        };
        match result {
            Ok(dirs) => done.push((write, dirs)),
            Err(e) => return Err(rollback(ws, &done, e)),
        }
        ahead.after_write(done.len());
    }
    let removals: Vec<&RepoPath> = step.index_removals.iter().collect();
    if let Err(e) = ws.git().index_remove(&removals, &[]) {
        return Err(rollback(ws, &done, e.into()));
    }
    Ok(())
}

/// The paths whose disk state is no longer the pre-image prepare stored, and those whose
/// directories to create appeared: stat first (absent stays absent, present must be a file),
/// then one `hash-object --stdin-paths` over the present ones.
fn changed_since_prepare(
    ws: &Workspace,
    step: &PreparedStep,
) -> Result<Vec<RepoPath>, WorkspaceError> {
    let mut changed = Vec::new();
    let mut to_hash: Vec<(&RepoPath, &BlobId)> = Vec::new();
    for write in &step.writes {
        let record = &write.record;
        let resolved = ws.resolve(&record.path)?;
        let present = resolved.exists() && !resolved.is_dir;
        let dir_appeared = record
            .creates_dirs
            .iter()
            .any(|d| ws.root().join(d).exists());
        match &record.pre {
            PreImage::Absent if present || resolved.is_dir || dir_appeared => {
                changed.push(record.path.clone());
            }
            PreImage::Absent => {}
            PreImage::Stored(_) if !present => changed.push(record.path.clone()),
            PreImage::Stored(id) => to_hash.push((&record.path, id)),
        }
    }
    let paths: Vec<&RepoPath> = to_hash.iter().map(|(p, _)| *p).collect();
    let ids = ws.git().hash_paths(&paths, false)?;
    for ((path, pre), now) in to_hash.iter().zip(ids) {
        if **pre != now {
            changed.push((*path).clone());
        }
    }
    Ok(changed)
}

/// Restores `done` in reverse order and reports what it could not restore.
fn rollback(
    ws: &Workspace,
    done: &[(&PlannedWrite, Vec<PathBuf>)],
    cause: WorkspaceError,
) -> CommitError {
    let mut unrestored = Vec::new();
    for (write, dirs) in done.iter().rev() {
        if restore(ws, &write.record).is_err() {
            unrestored.push(write.record.path.clone());
        }
        remove_dirs(dirs);
    }
    CommitError {
        cause: CommitCause::Workspace(cause),
        unrestored,
    }
}

/// Puts one path back as it was before the step: the pre-image's exact bytes and mode, or no
/// file at all.
pub(crate) fn restore(ws: &Workspace, record: &WriteRecord) -> Result<(), WorkspaceError> {
    match &record.pre {
        PreImage::Absent => ws.remove(&record.path).map(|_| ()),
        PreImage::Stored(id) => {
            let bytes = ws.git().cat_blob(id, &[])?;
            let executable = record.pre_mode.as_deref().map(|m| m == "100755");
            ws.write_as(&record.path, &bytes, &temp_name(), executable)
                .map(|_| ())
        }
    }
}

#[cfg(test)]
#[path = "transaction_tests.rs"]
mod tests;
