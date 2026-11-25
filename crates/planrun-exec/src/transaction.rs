//! A step's file actions as one transaction (decisions D23, D53): **prepare** validates them on
//! a virtual tree seeded from the disk, stores the pre-images and builds every post-image;
//! **commit** hands the prepared step to a write-ahead recorder and then writes it with the one
//! writer, rolling back on any error.

use std::path::PathBuf;

use planrun_model::{Action, ActionKind, PlanSource, RepoPath};

use crate::finding::Finding;
use crate::git::BlobId;
use crate::handler::{HandlerSet, file_path};
use crate::tree::{State, VirtualTree};
use crate::workspace::{Workspace, WorkspaceError, remove_dirs};

/// What a path was before the step.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// The path.
    pub path: RepoPath,
    /// Before.
    pub pre: PreImage,
    /// After.
    pub post: PostImage,
    /// The post-image's blob id as git will see it; `None` for a delete.
    pub post_blob: Option<BlobId>,
    /// The directories the write will create, from the root, outermost first.
    pub creates_dirs: Vec<String>,
}

/// A validated step, ready to commit: what P5 journals before writing (write-ahead, D53).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedStep {
    /// The paths, in the order their first action names them.
    pub writes: Vec<PlannedWrite>,
    /// The paths a `Delete` with `via_git` removes from the index after every write.
    pub index_removals: Vec<RepoPath>,
}

/// Records a prepared step before it is written. P5's recorder is the journal; a real run
/// without that record is not allowed (D53).
pub trait WriteAhead {
    /// Stores `step` durably; an error stops the commit before anything is written.
    fn record(&mut self, step: &PreparedStep) -> std::io::Result<()>;
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
    /// The write-ahead record failed; nothing was written.
    #[error("the write-ahead record failed: {0}")]
    Record(#[source] std::io::Error),
    /// A write, a delete or the index update failed.
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
}

fn unrestored_note(paths: &[RepoPath]) -> String {
    if paths.is_empty() {
        return String::new();
    }
    let list: Vec<&str> = paths.iter().map(RepoPath::as_str).collect();
    format!("; rollback could not restore {}", list.join(", "))
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

/// The planned write of one touched path; `None` when it ends as it began (absent).
fn planned(
    ws: &Workspace,
    tree: &mut VirtualTree<'_>,
    path: RepoPath,
) -> Result<Option<PlannedWrite>, Finding> {
    let on_disk = ws
        .read(&path)
        .map_err(|e| Finding::workspace(&e, None, Some(&path)))?;
    let (post, post_blob) = match tree.state(&path, None)?.clone() {
        State::Present { blob, .. } => (PostImage::Bytes(tree.bytes(&path, None)?), Some(blob)),
        _ if on_disk.is_none() => return Ok(None),
        _ => (PostImage::Delete, None),
    };
    let pre = match &on_disk {
        None => PreImage::Absent,
        Some(bytes) => PreImage::Stored(
            ws.git()
                .hash_object(&path, bytes, false, true, &[])
                .map_err(|e| Finding::error("git", None, Some(&path), e.to_string()))?,
        ),
    };
    let resolved = ws
        .resolve(&path)
        .map_err(|e| Finding::workspace(&e, None, Some(&path)))?;
    let segments: Vec<&str> = path.as_str().split('/').collect();
    let creates_dirs = (resolved.existing.min(segments.len() - 1)..segments.len() - 1)
        .map(|i| segments[..=i].join("/"))
        .collect();
    Ok(Some(PlannedWrite {
        path,
        pre,
        post,
        post_blob,
        creates_dirs,
    }))
}

/// Writes `step`: first the write-ahead record, then every path in order, then the index
/// removals in one call. On an error every path already written is restored from its
/// pre-image, created files and directories are removed, and the error names what could not be
/// restored.
pub fn commit(
    ws: &Workspace,
    step: &PreparedStep,
    ahead: &mut dyn WriteAhead,
) -> Result<(), CommitError> {
    ahead.record(step).map_err(|e| CommitError {
        cause: CommitCause::Record(e),
        unrestored: Vec::new(),
    })?;
    let mut done: Vec<(&PlannedWrite, Vec<PathBuf>)> = Vec::new();
    for write in &step.writes {
        let result = match &write.post {
            PostImage::Bytes(bytes) => ws.write(&write.path, bytes),
            PostImage::Delete => ws.remove(&write.path).map(|_| Vec::new()),
        };
        match result {
            Ok(dirs) => done.push((write, dirs)),
            Err(e) => return Err(rollback(ws, &done, e)),
        }
    }
    let removals: Vec<&RepoPath> = step.index_removals.iter().collect();
    if let Err(e) = ws.git().index_remove(&removals, &[]) {
        return Err(rollback(ws, &done, e.into()));
    }
    Ok(())
}

/// Restores `done` in reverse order and reports what it could not restore.
fn rollback(
    ws: &Workspace,
    done: &[(&PlannedWrite, Vec<PathBuf>)],
    cause: WorkspaceError,
) -> CommitError {
    let mut unrestored = Vec::new();
    for (write, dirs) in done.iter().rev() {
        if restore(ws, write).is_err() {
            unrestored.push(write.path.clone());
        }
        remove_dirs(dirs);
    }
    CommitError {
        cause: CommitCause::Workspace(cause),
        unrestored,
    }
}

fn restore(ws: &Workspace, write: &PlannedWrite) -> Result<(), WorkspaceError> {
    match &write.pre {
        PreImage::Absent => ws.remove(&write.path).map(|_| ()),
        PreImage::Stored(id) => {
            let bytes = ws.git().cat_blob(id, &[])?;
            ws.write(&write.path, &bytes).map(|_| ())
        }
    }
}

#[cfg(test)]
#[path = "transaction_tests.rs"]
mod tests;
