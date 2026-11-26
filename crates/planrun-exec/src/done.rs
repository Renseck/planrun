//! A journaled step's state on the disk, for resume (decisions D14, D77, D78): whether its
//! writes are all there (Done), none of them are (NotDone), or anything else (Unknown, the user
//! decides); whether its pre-images survive to restore it; and the writer's temp files it may
//! have left behind.

use std::fs;

use planrun_model::RepoPath;

use crate::DoneState;
use crate::git::BlobId;
use crate::transaction::{PreImage, StepRecord, WriteRecord, restore};
use crate::workspace::{Workspace, WorkspaceError};

/// What one path of a step looks like now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seen {
    /// As the step leaves it (bytes, mode under `core.filemode`, and for a `via_git` delete
    /// the index entry gone).
    Post,
    /// As it was before the step.
    Pre,
    /// Neither.
    Neither,
}

/// A step's paths as they are now, and what that makes the step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepOnDisk {
    /// Each planned path, in the step's order.
    pub paths: Vec<(RepoPath, Seen)>,
    /// Done when every path is as the step leaves it, NotDone when every path is as it was,
    /// Unknown otherwise. A path whose pre- and post-image are the same counts for both.
    pub state: DoneState,
}

/// One path's disk state.
struct Now {
    present: bool,
    raw: Option<BlobId>,
    executable: bool,
    in_index: Option<bool>,
}

/// Compares the disk with `step`: one stat per path, one `hash-object --stdin-paths` over the
/// present ones, one `ls-files` per index removal.
pub fn step_state(ws: &Workspace, step: &StepRecord) -> Result<StepOnDisk, WorkspaceError> {
    let mut nows = Vec::new();
    for record in &step.writes {
        let resolved = ws.resolve(&record.path)?;
        let present = resolved.exists() && !resolved.is_dir;
        let in_index = if step.index_removals.contains(&record.path) {
            Some(ws.git().index_entry(&record.path, &[])?.is_some())
        } else {
            None
        };
        nows.push(Now {
            present,
            raw: None,
            executable: present && Workspace::is_executable(&resolved.full),
            in_index,
        });
    }
    let present: Vec<&RepoPath> = step
        .writes
        .iter()
        .zip(&nows)
        .filter(|(_, now)| now.present)
        .map(|(r, _)| &r.path)
        .collect();
    let mut ids = ws.git().hash_paths(&present, false)?.into_iter();
    for now in nows.iter_mut().filter(|n| n.present) {
        now.raw = ids.next();
    }
    let mut paths = Vec::new();
    let (mut all_post, mut all_pre) = (true, true);
    for (record, now) in step.writes.iter().zip(&nows) {
        let post = is_post(ws, record, now);
        let pre = is_pre(ws, record, now);
        all_post &= post;
        all_pre &= pre;
        let seen = match (post, pre) {
            (true, _) => Seen::Post,
            (false, true) => Seen::Pre,
            (false, false) => Seen::Neither,
        };
        paths.push((record.path.clone(), seen));
    }
    let state = match (all_post, all_pre) {
        (true, _) => DoneState::Done,
        (false, true) => DoneState::NotDone,
        (false, false) => DoneState::Unknown,
    };
    Ok(StepOnDisk { paths, state })
}

fn mode_holds(ws: &Workspace, mode: Option<&str>, now: &Now) -> bool {
    !ws.filemode() || mode.is_none_or(|m| (m == "100755") == now.executable)
}

fn is_post(ws: &Workspace, record: &WriteRecord, now: &Now) -> bool {
    match record.post {
        Some(id) => now.raw == Some(id) && mode_holds(ws, record.mode.as_deref(), now),
        None => !now.present && now.in_index != Some(true),
    }
}

fn is_pre(ws: &Workspace, record: &WriteRecord, now: &Now) -> bool {
    match &record.pre {
        PreImage::Absent => !now.present,
        PreImage::Stored(id) => {
            now.raw == Some(*id)
                && mode_holds(ws, record.pre_mode.as_deref(), now)
                && now.in_index != Some(false)
        }
    }
}

/// The pre-image objects of `step` that the object database no longer has (D57).
pub fn missing_pre_images(
    ws: &Workspace,
    step: &StepRecord,
) -> Result<Vec<BlobId>, WorkspaceError> {
    let ids: Vec<BlobId> = step
        .writes
        .iter()
        .filter_map(|w| match w.pre {
            PreImage::Stored(id) => Some(id),
            PreImage::Absent => None,
        })
        .collect();
    Ok(ws.git().missing_objects(&ids)?)
}

/// Whether [`restore_step`] can put `step` back as it was: every pre-image object is there,
/// and no index removal has happened yet (its entry would be lost).
pub fn restorable(ws: &Workspace, step: &StepRecord) -> Result<bool, WorkspaceError> {
    if !missing_pre_images(ws, step)?.is_empty() {
        return Ok(false);
    }
    for path in &step.index_removals {
        if ws.git().index_entry(path, &[])?.is_none() {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Puts every path of `step` back as it was before it (resume's `[r]`, D77): pre-image bytes
/// and mode, created files removed, created directories removed while empty. Returns the paths
/// it could not restore.
pub fn restore_step(ws: &Workspace, step: &StepRecord) -> Vec<RepoPath> {
    let mut unrestored = Vec::new();
    for record in step.writes.iter().rev() {
        if restore(ws, record).is_err() {
            unrestored.push(record.path.clone());
        }
        for dir in record.creates_dirs.iter().rev() {
            let _ = fs::remove_dir(ws.root().join(dir));
        }
    }
    unrestored
}

/// Removes the writer's temp files `step` names, where they exist (a crash between the temp
/// write and the rename, D77). Nothing is searched for by pattern. Returns what it removed, as
/// repository-relative paths.
pub fn remove_temps(ws: &Workspace, step: &StepRecord) -> Vec<String> {
    let mut removed = Vec::new();
    for record in &step.writes {
        let rel = match record.path.dir() {
            Some(dir) => format!("{dir}/{}", record.temp),
            None => record.temp.clone(),
        };
        let Ok(path) = RepoPath::new(&rel) else {
            continue;
        };
        let Ok(resolved) = ws.resolve(&path) else {
            continue;
        };
        let is_file = fs::symlink_metadata(&resolved.full).is_ok_and(|m| m.is_file());
        if is_file && fs::remove_file(&resolved.full).is_ok() {
            removed.push(rel);
        }
    }
    removed
}

#[cfg(test)]
#[path = "done_tests.rs"]
mod tests;
