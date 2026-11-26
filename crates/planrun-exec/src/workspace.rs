//! The repository on disk: path resolution with the link and case check, and the one writer
//! (decision D52). Nothing here decides what to write; the transaction does (D53).

use std::collections::hash_map::RandomState;
use std::ffi::OsString;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use planrun_model::RepoPath;

use crate::git::{Git, GitError};

/// Why the workspace refused or failed.
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    /// The path leaves the repository through a link, or is not what its spelling says.
    #[error("`{path}`: {reason}")]
    Unsafe {
        /// The plan's path.
        path: String,
        /// What is wrong.
        reason: String,
    },
    /// The path's existing segments differ in case from the disk (D52).
    #[error("`{path}` is spelled `{on_disk}` on disk")]
    Case {
        /// The plan's path.
        path: String,
        /// The spelling on disk, from the root.
        on_disk: String,
    },
    /// A file operation failed.
    #[error("`{path}`: {source}")]
    Io {
        /// The plan's path.
        path: String,
        /// The error.
        #[source]
        source: std::io::Error,
    },
    /// A git call failed.
    #[error(transparent)]
    Git(#[from] GitError),
    /// The repository uses an object format other than SHA-1.
    #[error("the repository uses `{0}` object ids; planrun supports SHA-1 repositories only")]
    ObjectFormat(String),
}

impl WorkspaceError {
    /// The stable code of a finding made from this error.
    pub fn code(&self) -> &'static str {
        match self {
            WorkspaceError::Unsafe { .. } => "path-unsafe",
            WorkspaceError::Case { .. } => "path-case",
            WorkspaceError::Io { .. } => "io",
            WorkspaceError::Git(_) => "git",
            WorkspaceError::ObjectFormat(_) => "object-format",
        }
    }
}

/// A path resolved on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// The full path.
    pub full: PathBuf,
    /// How many leading segments exist on disk.
    pub existing: usize,
    /// The number of segments.
    pub segments: usize,
    /// True when the whole path exists and is a directory.
    pub is_dir: bool,
}

impl Resolved {
    /// True when the whole path exists.
    pub fn exists(&self) -> bool {
        self.existing == self.segments
    }
}

/// The repository root, its git, and the writer.
pub struct Workspace {
    root: PathBuf,
    git: Git,
    ignorecase: bool,
    filemode: bool,
    #[cfg(test)]
    pub(crate) faults: Faults,
}

impl Workspace {
    /// The repository that contains `dir`, with `env` added to every git call.
    pub fn open(dir: &Path, env: Vec<(OsString, OsString)>) -> Result<Self, WorkspaceError> {
        let top = Git::toplevel(dir, &env)?;
        let root = fs::canonicalize(&top).map_err(|source| WorkspaceError::Io {
            path: top.display().to_string(),
            source,
        })?;
        let git = Git::new(&top, env);
        let format = git.rev_parse(&["--show-object-format"])?;
        if format != "sha1" {
            return Err(WorkspaceError::ObjectFormat(format));
        }
        let ignorecase = git.config_bool("core.ignorecase")?.unwrap_or(false);
        let filemode = git.config_bool("core.filemode")?.unwrap_or(cfg!(unix));
        Ok(Self {
            root,
            git,
            ignorecase,
            filemode,
            #[cfg(test)]
            faults: Faults::default(),
        })
    }

    /// The canonical repository root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// git, run at the root.
    pub fn git(&self) -> &Git {
        &self.git
    }

    /// `core.ignorecase`: path identity is case-insensitive (D52).
    pub fn ignorecase(&self) -> bool {
        self.ignorecase
    }

    /// `core.filemode` (unset: true on Unix, false elsewhere): the executable bit counts (D78).
    /// Always false on Windows, which has no executable bit.
    pub fn filemode(&self) -> bool {
        self.filemode && cfg!(unix)
    }

    /// Whether the file at `full` has its executable bit set; false on Windows.
    pub fn is_executable(full: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::metadata(full).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
        }
        #[cfg(not(unix))]
        {
            let _ = full;
            false
        }
    }

    /// Resolves `path`: every existing segment must be a real directory (the last one may be a
    /// file), never a link, and the existing part must canonicalize to the root plus the same
    /// segments, compared exactly. That refuses symlinks, junctions, 8.3 short names and case
    /// mismatches (D52).
    pub fn resolve(&self, path: &RepoPath) -> Result<Resolved, WorkspaceError> {
        let segments: Vec<&str> = path.as_str().split('/').collect();
        let mut current = self.root.clone();
        let mut existing = 0;
        let mut is_dir = false;
        for (i, segment) in segments.iter().enumerate() {
            let next = current.join(segment);
            let meta = match fs::symlink_metadata(&next) {
                Ok(meta) => meta,
                Err(e) if e.kind() == ErrorKind::NotFound => break,
                Err(source) => return Err(io_error(path, source)),
            };
            if meta.file_type().is_symlink() {
                return Err(unsafe_path(
                    path,
                    "a segment is a symbolic link or junction",
                ));
            }
            if i + 1 < segments.len() && !meta.is_dir() {
                return Err(unsafe_path(
                    path,
                    "a segment before the last is not a directory",
                ));
            }
            is_dir = meta.is_dir();
            existing = i + 1;
            current = next;
        }
        self.check_spelling(path, &segments[..existing], &current)?;
        let mut full = current;
        for segment in &segments[existing..] {
            full.push(segment);
        }
        Ok(Resolved {
            full,
            existing,
            segments: segments.len(),
            is_dir: is_dir && existing == segments.len(),
        })
    }

    /// The existing part of a path, canonicalized, must be the root plus exactly these segments.
    fn check_spelling(
        &self,
        path: &RepoPath,
        segments: &[&str],
        deepest: &Path,
    ) -> Result<(), WorkspaceError> {
        if segments.is_empty() {
            return Ok(());
        }
        let actual = fs::canonicalize(deepest).map_err(|e| io_error(path, e))?;
        let expected = segments.iter().fold(self.root.clone(), |p, s| p.join(s));
        if actual == expected {
            return Ok(());
        }
        let shown = |p: &Path| p.to_string_lossy().to_lowercase();
        if shown(&actual) == shown(&expected) {
            let on_disk = actual.strip_prefix(&self.root).unwrap_or(&actual);
            return Err(WorkspaceError::Case {
                path: path.to_string(),
                on_disk: on_disk.to_string_lossy().replace('\\', "/"),
            });
        }
        Err(unsafe_path(
            path,
            "it resolves to another place on disk (a link or a short name)",
        ))
    }

    /// The bytes at `path`; `None` when it does not exist.
    pub fn read(&self, path: &RepoPath) -> Result<Option<Vec<u8>>, WorkspaceError> {
        let resolved = self.resolve(path)?;
        if !resolved.exists() {
            return Ok(None);
        }
        if resolved.is_dir {
            return Err(unsafe_path(path, "it is a directory"));
        }
        fs::read(&resolved.full)
            .map(Some)
            .map_err(|e| io_error(path, e))
    }

    /// Writes `bytes` to `path` through a temp file and a rename, creating the directories it
    /// needs one segment at a time, each checked. Returns the directories it created, deepest
    /// last. On an error nothing of this write remains: no temp file, no new directory.
    pub fn write(&self, path: &RepoPath, bytes: &[u8]) -> Result<Vec<PathBuf>, WorkspaceError> {
        self.write_as(path, bytes, &temp_name(), None)
    }

    /// [`Workspace::write`] with the temp file's name chosen beforehand (a step transaction
    /// journals it, D77) and, under `core.filemode`, the executable bit set or cleared on the
    /// temp file before the rename (`Some(true)` for mode 100755, D78).
    pub fn write_as(
        &self,
        path: &RepoPath,
        bytes: &[u8],
        temp: &str,
        executable: Option<bool>,
    ) -> Result<Vec<PathBuf>, WorkspaceError> {
        let resolved = self.resolve(path)?;
        if resolved.is_dir {
            return Err(unsafe_path(path, "it is a directory"));
        }
        let created = self.make_dirs(path, &resolved)?;
        let executable = executable.filter(|_| self.filemode());
        if let Err(e) = self.replace_file(path, &resolved.full, bytes, temp, executable) {
            remove_dirs(&created);
            return Err(e);
        }
        Ok(created)
    }

    /// Removes the file at `path`; true when there was one.
    pub fn remove(&self, path: &RepoPath) -> Result<bool, WorkspaceError> {
        let resolved = self.resolve(path)?;
        if !resolved.exists() {
            return Ok(false);
        }
        if resolved.is_dir {
            return Err(unsafe_path(path, "it is a directory"));
        }
        fs::remove_file(&resolved.full).map_err(|e| io_error(path, e))?;
        Ok(true)
    }

    /// The directories of `path` that do not exist yet, created one at a time, each checked
    /// right after it was made.
    fn make_dirs(
        &self,
        path: &RepoPath,
        resolved: &Resolved,
    ) -> Result<Vec<PathBuf>, WorkspaceError> {
        let segments: Vec<&str> = path.as_str().split('/').collect();
        let dirs = segments.len() - 1;
        let first_missing = resolved.existing.min(dirs);
        let mut current = segments[..first_missing]
            .iter()
            .fold(self.root.clone(), |p, s| p.join(s));
        let mut created = Vec::new();
        for segment in &segments[first_missing..dirs] {
            current.push(segment);
            if let Err(e) = fs::create_dir(&current) {
                remove_dirs(&created);
                return Err(io_error(path, e));
            }
            created.push(current.clone());
            let real = fs::symlink_metadata(&current)
                .is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink());
            if !real {
                remove_dirs(&created);
                return Err(unsafe_path(path, "a directory it created is not one"));
            }
        }
        Ok(created)
    }

    /// The temp file, the second check of the parent, and the rename.
    fn replace_file(
        &self,
        path: &RepoPath,
        full: &Path,
        bytes: &[u8],
        temp: &str,
        executable: Option<bool>,
    ) -> Result<(), WorkspaceError> {
        let parent = full.parent().unwrap_or(&self.root);
        let temp = parent.join(temp);
        if let Err(e) = write_new(&temp, bytes).and_then(|()| set_executable(&temp, executable)) {
            let _ = fs::remove_file(&temp);
            return Err(io_error(path, e));
        }
        let renamed = self
            .before_rename(path, full)
            .and_then(|()| self.recheck_parent(path))
            .and_then(|()| fs::rename(&temp, full).map_err(|e| io_error(path, e)));
        if renamed.is_err() {
            let _ = fs::remove_file(&temp);
        }
        renamed
    }

    /// The parent directory again, directly before the rename (D52). A parent swapped after
    /// this check is the race D52 states.
    fn recheck_parent(&self, path: &RepoPath) -> Result<(), WorkspaceError> {
        let Some(dir) = path.dir() else {
            return Ok(());
        };
        let parent = RepoPath::new(dir).map_err(|e| unsafe_path(path, &e.to_string()))?;
        let resolved = self.resolve(&parent)?;
        if resolved.exists() && resolved.is_dir {
            return Ok(());
        }
        Err(unsafe_path(path, "its directory changed before the rename"))
    }

    #[cfg(not(test))]
    fn before_rename(&self, _path: &RepoPath, _full: &Path) -> Result<(), WorkspaceError> {
        Ok(())
    }

    #[cfg(test)]
    fn before_rename(&self, path: &RepoPath, full: &Path) -> Result<(), WorkspaceError> {
        self.faults.before_rename(path, full)
    }
}

/// Test-only faults: fail the n-th write (1-based), or run a hook right before a rename.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct Faults {
    /// The write that fails, counted from 1.
    pub fail_write: std::cell::Cell<Option<usize>>,
    writes: std::cell::Cell<usize>,
    /// Runs before the parent check and the rename, with the target's full path.
    #[allow(clippy::type_complexity)]
    pub hook: std::cell::RefCell<Option<Box<dyn Fn(&Path)>>>,
}

#[cfg(test)]
impl Faults {
    fn before_rename(&self, path: &RepoPath, full: &Path) -> Result<(), WorkspaceError> {
        let n = self.writes.get() + 1;
        self.writes.set(n);
        if let Some(hook) = self.hook.borrow().as_ref() {
            hook(full);
        }
        if self.fail_write.get() == Some(n) {
            return Err(io_error(path, std::io::Error::other("injected fault")));
        }
        Ok(())
    }
}

/// A fresh name for a writer's temp file: `.planrun-<16 hex>.tmp`.
pub fn temp_name() -> String {
    format!(".planrun-{:016x}.tmp", random())
}

/// True for a name [`temp_name`] makes.
pub fn is_temp_name(name: &str) -> bool {
    name.strip_prefix(".planrun-")
        .and_then(|r| r.strip_suffix(".tmp"))
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// Sets or clears the executable bits (for every class that may read the file); nothing when
/// `executable` is `None` or on Windows.
fn set_executable(file: &Path, executable: Option<bool>) -> std::io::Result<()> {
    let Some(executable) = executable else {
        return Ok(());
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(file)?.permissions();
        let mode = permissions.mode();
        let mode = if executable {
            mode | ((mode & 0o444) >> 2)
        } else {
            mode & !0o111
        };
        permissions.set_mode(mode);
        fs::set_permissions(file, permissions)?;
    }
    #[cfg(not(unix))]
    let _ = (file, executable);
    Ok(())
}

fn write_new(temp: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// Removes directories this write created, deepest first, while they are empty.
pub(crate) fn remove_dirs(created: &[PathBuf]) {
    for dir in created.iter().rev() {
        let _ = fs::remove_dir(dir);
    }
}

fn random() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut hasher = RandomState::new().build_hasher();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    hasher.write_u128(nanos);
    hasher.write_u64(COUNTER.fetch_add(1, Ordering::Relaxed));
    hasher.write_u32(std::process::id());
    hasher.finish()
}

fn io_error(path: &RepoPath, source: std::io::Error) -> WorkspaceError {
    WorkspaceError::Io {
        path: path.to_string(),
        source,
    }
}

fn unsafe_path(path: &RepoPath, reason: &str) -> WorkspaceError {
    WorkspaceError::Unsafe {
        path: path.to_string(),
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
#[path = "workspace_tests.rs"]
mod tests;
