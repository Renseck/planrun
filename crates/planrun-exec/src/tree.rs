//! The virtual tree (decision D52): a temporary index and object directory under the OS temp
//! directory, with the repository's objects as alternates. File actions are simulated on it;
//! patches go through `git apply --cached`. Nothing in the repository is written.

use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::ffi::OsString;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::path::{Path, PathBuf};

use planrun_model::{RepoPath, Span};

use crate::eol::Eol;
use crate::finding::Finding;
use crate::git::{Attributes, BlobId, GitError};
use crate::workspace::{Workspace, WorkspaceError};

/// What the tree knows about a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Not on disk when it was first seen.
    Absent,
    /// Deleted by an action of this run.
    Deleted,
    /// Present, with its blob (git's normalized view), its mode, and its working-tree bytes
    /// when known (not after a patch, until asked for).
    Present {
        /// The blob id, filtered.
        blob: BlobId,
        /// The index mode (`100644`).
        mode: String,
        /// The working-tree bytes.
        bytes: Option<Vec<u8>>,
    },
    /// A script or a manual step that did not run may have changed it (dry run, D55).
    Unknown,
}

impl State {
    /// True when the path exists in the tree.
    pub fn exists(&self) -> bool {
        matches!(self, State::Present { .. })
    }
}

struct Slot {
    spelled: RepoPath,
    state: State,
}

/// The virtual tree of one dry run or one step transaction.
pub struct VirtualTree<'w> {
    ws: &'w Workspace,
    dir: PathBuf,
    env: Vec<(OsString, OsString)>,
    slots: HashMap<String, Slot>,
    attributes: HashMap<String, Attributes>,
    checkout: HashMap<String, Eol>,
    one_line: Option<BlobId>,
}

impl<'w> VirtualTree<'w> {
    /// A fresh, empty tree over `ws`; paths are seeded from the disk on first use.
    pub fn new(ws: &'w Workspace) -> Result<Self, WorkspaceError> {
        let objects = ws.git().object_dir()?;
        let dir = std::env::temp_dir().join(format!(
            "planrun-tree-{}-{:016x}",
            std::process::id(),
            RandomState::new().build_hasher().finish()
        ));
        fs::create_dir_all(dir.join("objects")).map_err(|source| WorkspaceError::Io {
            path: dir.display().to_string(),
            source,
        })?;
        let env = vec![
            var("GIT_INDEX_FILE", &dir.join("index")),
            var("GIT_OBJECT_DIRECTORY", &dir.join("objects")),
            var("GIT_ALTERNATE_OBJECT_DIRECTORIES", &objects),
        ];
        Ok(Self {
            ws,
            dir,
            env,
            slots: HashMap::new(),
            attributes: HashMap::new(),
            checkout: HashMap::new(),
            one_line: None,
        })
    }

    /// The workspace this tree simulates.
    pub fn workspace(&self) -> &'w Workspace {
        self.ws
    }

    /// The key of `path`: case-folded with `core.ignorecase` (D52).
    fn key(&self, path: &RepoPath) -> String {
        if self.ws.ignorecase() {
            path.as_str().to_lowercase()
        } else {
            path.as_str().to_owned()
        }
    }

    /// The state of `path`, seeding it from the disk on first use. Another spelling of a path
    /// already in the tree is `path-collision`.
    pub fn state(&mut self, path: &RepoPath, span: Option<Span>) -> Result<&State, Finding> {
        let key = self.key(path);
        if let Some(slot) = self.slots.get(&key) {
            if slot.spelled != *path {
                return Err(collision(path, &slot.spelled, span));
            }
        } else {
            let state = self
                .seed(path)
                .map_err(|e| Finding::workspace(&e, span, Some(path)))?;
            let spelled = path.clone();
            self.slots.insert(key.clone(), Slot { spelled, state });
        }
        self.slots
            .get(&key)
            .map(|s| &s.state)
            .ok_or_else(|| Finding::error("internal", span, Some(path), "a seeded path vanished"))
    }

    fn seed(&self, path: &RepoPath) -> Result<State, WorkspaceError> {
        let Some(bytes) = self.ws.read(path)? else {
            return Ok(State::Absent);
        };
        let git = self.ws.git();
        let mode = git
            .index_entry(path, &[])?
            .map_or_else(|| "100644".to_owned(), |(mode, _)| mode);
        let blob = git.hash_object(path, &bytes, true, true, &self.env)?;
        git.index_add(path, &mode, &blob, &self.env)?;
        Ok(State::Present {
            blob,
            mode,
            bytes: Some(bytes),
        })
    }

    fn slot_mut(&mut self, path: &RepoPath) -> Option<&mut Slot> {
        let key = self.key(path);
        self.slots.get_mut(&key)
    }

    /// The working-tree bytes of a present path: as seeded or written, or for a patched path
    /// what a checkout would write (`cat-file --filters`, D53).
    pub fn bytes(&mut self, path: &RepoPath, span: Option<Span>) -> Result<Vec<u8>, Finding> {
        let state = self.state(path, span)?.clone();
        let State::Present { blob, mode, bytes } = state else {
            return Err(Finding::error(
                "target-missing",
                span,
                Some(path),
                format!("`{path}` does not exist"),
            ));
        };
        if let Some(bytes) = bytes {
            return Ok(bytes);
        }
        let git = self.ws.git();
        let out = git
            .cat_filtered(&blob, path, &self.env)
            .map_err(|e| git_finding(&e, span, path))?;
        if let Some(slot) = self.slot_mut(path) {
            slot.state = State::Present {
                blob,
                mode,
                bytes: Some(out.clone()),
            };
        }
        Ok(out)
    }

    /// The blob of a present path.
    pub fn blob(&mut self, path: &RepoPath, span: Option<Span>) -> Result<Option<BlobId>, Finding> {
        Ok(match self.state(path, span)? {
            State::Present { blob, .. } => Some(*blob),
            _ => None,
        })
    }

    /// Puts `bytes` at `path` (filtered blob id, `hash-object -w --path` into the tree's objects).
    pub fn put(
        &mut self,
        path: &RepoPath,
        bytes: Vec<u8>,
        span: Option<Span>,
    ) -> Result<(), Finding> {
        let mode = match self.state(path, span)? {
            State::Present { mode, .. } => mode.clone(),
            _ => "100644".to_owned(),
        };
        let git = self.ws.git();
        let blob = git
            .hash_object(path, &bytes, true, true, &self.env)
            .and_then(|blob| git.index_add(path, &mode, &blob, &self.env).map(|()| blob))
            .map_err(|e| git_finding(&e, span, path))?;
        self.set(
            path,
            State::Present {
                blob,
                mode,
                bytes: Some(bytes),
            },
        );
        Ok(())
    }

    /// Applies `patch` to the tree's index (`git apply --cached`, after `--check`). The error is
    /// git's message.
    pub fn apply(
        &mut self,
        path: &RepoPath,
        patch: &[u8],
        span: Option<Span>,
    ) -> Result<(), String> {
        self.state(path, span).map_err(|f| f.message)?;
        let git = self.ws.git();
        git.apply(patch, true, true, &self.env)
            .and_then(|()| git.apply(patch, true, false, &self.env))
            .map_err(|e| git_message(&e))?;
        let entry = git
            .index_entry(path, &self.env)
            .map_err(|e| git_message(&e))?;
        let state = match entry {
            Some((mode, blob)) => State::Present {
                blob,
                mode,
                bytes: None,
            },
            None => State::Deleted,
        };
        self.set(path, state);
        Ok(())
    }

    /// Deletes `path` from the tree.
    pub fn delete(&mut self, path: &RepoPath, span: Option<Span>) -> Result<(), Finding> {
        self.state(path, span)?;
        self.ws
            .git()
            .index_remove(&[path], &self.env)
            .map_err(|e| git_finding(&e, span, path))?;
        self.set(path, State::Deleted);
        Ok(())
    }

    /// Marks `path` as changed by something that did not run (D55).
    pub fn mark_unknown(&mut self, path: &RepoPath) {
        let key = self.key(path);
        let spelled = self
            .slots
            .get(&key)
            .map_or_else(|| path.clone(), |s| s.spelled.clone());
        self.slots.insert(
            key,
            Slot {
                spelled,
                state: State::Unknown,
            },
        );
    }

    /// True when `path` was marked unknown.
    pub fn is_unknown(&self, path: &RepoPath) -> bool {
        self.slots
            .get(&self.key(path))
            .is_some_and(|s| s.state == State::Unknown)
    }

    fn set(&mut self, path: &RepoPath, state: State) {
        if let Some(slot) = self.slot_mut(path) {
            slot.state = state;
        }
    }

    /// The `eol` and `text` attributes of `path`, read once per tree (D54: a step's attributes
    /// are those at its start).
    pub fn attributes(
        &mut self,
        path: &RepoPath,
        span: Option<Span>,
    ) -> Result<Attributes, Finding> {
        let key = self.key(path);
        if let Some(found) = self.attributes.get(&key) {
            return Ok(found.clone());
        }
        let found = self
            .ws
            .git()
            .attributes(&[path])
            .map_err(|e| git_finding(&e, span, path))?
            .pop()
            .unwrap_or_default();
        self.attributes.insert(key, found.clone());
        Ok(found)
    }

    /// git's checkout convention for `path` (D54): a one-line blob converted as a checkout of
    /// that path would convert it.
    pub fn checkout_eol(&mut self, path: &RepoPath, span: Option<Span>) -> Result<Eol, Finding> {
        let key = self.key(path);
        if let Some(eol) = self.checkout.get(&key) {
            return Ok(*eol);
        }
        let git = self.ws.git();
        let blob = match self.one_line {
            Some(blob) => blob,
            None => git
                .hash_object(path, b"a\n", false, true, &self.env)
                .map_err(|e| git_finding(&e, span, path))?,
        };
        self.one_line = Some(blob);
        let out = git
            .cat_filtered(&blob, path, &self.env)
            .map_err(|e| git_finding(&e, span, path))?;
        let eol = if out.ends_with(b"\r\n") {
            Eol::Crlf
        } else {
            Eol::Lf
        };
        self.checkout.insert(key, eol);
        Ok(eol)
    }
}

impl Drop for VirtualTree<'_> {
    fn drop(&mut self) {
        clear_read_only(&self.dir);
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn var(name: &str, path: &Path) -> (OsString, OsString) {
    (OsString::from(name), path.as_os_str().to_os_string())
}

fn collision(path: &RepoPath, other: &RepoPath, span: Option<Span>) -> Finding {
    Finding::error(
        "path-collision",
        span,
        Some(path),
        format!("`{path}` and `{other}` are one file here (core.ignorecase)"),
    )
}

fn git_message(e: &GitError) -> String {
    match e {
        GitError::Failed { stderr, .. } => stderr.clone(),
        other => other.to_string(),
    }
}

fn git_finding(e: &GitError, span: Option<Span>, path: &RepoPath) -> Finding {
    Finding::error("git", span, Some(path), e.to_string())
}

/// git writes object files read-only; Windows refuses to delete those.
fn clear_read_only(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            clear_read_only(&path);
            continue;
        }
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        let mut permissions = meta.permissions();
        if permissions.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = fs::set_permissions(&path, permissions);
        }
    }
}
