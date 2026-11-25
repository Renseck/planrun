//! A temporary copy of `tests/fixtures/greeter` as a git repository of its own, for the tests of
//! planrun-exec and planrun-cli (included with `#[path]`; D2 keeps the workspace at four
//! crates). Never inside this repository: the copy lives under the OS temp directory, and every
//! git call gets an environment that keeps the user's configuration out (no system config, an
//! empty global config, a ceiling above the copy).

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A fixture copy: `<temp>/planrun-test-<pid>-<nanos>-<n>/repo`, with the empty global config
/// next to it. Removed on drop.
pub struct Fixture {
    base: PathBuf,
    repo: PathBuf,
}

impl Fixture {
    /// The greeter fixture, committed once, with `core.autocrlf` set as given. With `true`, the
    /// working tree is checked out again so that its text files have CRLF line endings.
    pub fn greeter(autocrlf: bool) -> Fixture {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let base =
            std::env::temp_dir().join(format!("planrun-test-{}-{nanos}-{n}", std::process::id()));
        let repo = base.join("repo");
        std::fs::create_dir_all(&repo).unwrap_or_else(|e| panic!("{}: {e}", repo.display()));
        std::fs::write(base.join("gitconfig"), "").unwrap_or_else(|e| panic!("{e}"));
        let fixture = Fixture { base, repo };
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/greeter");
        copy_dir(&source, &fixture.repo);
        fixture.git(&["init", "-q", "-b", "main"]);
        let top = fixture.git(&["rev-parse", "--show-toplevel"]);
        assert_eq!(
            canonical(Path::new(top.trim())),
            canonical(&fixture.repo),
            "git init did not make the copy a repository of its own"
        );
        fixture.git(&[
            "config",
            "core.autocrlf",
            if autocrlf { "true" } else { "false" },
        ]);
        fixture.git(&["add", "-A"]);
        fixture.git(&["commit", "-q", "-m", "fixture"]);
        if autocrlf {
            fixture.checkout_again();
        }
        fixture
    }

    /// The repository root.
    pub fn path(&self) -> &Path {
        &self.repo
    }

    /// The environment every git call of a test gets.
    pub fn env(&self) -> Vec<(OsString, OsString)> {
        let var = |k: &str, v: &str| (OsString::from(k), OsString::from(v));
        vec![
            var("GIT_CONFIG_NOSYSTEM", "1"),
            (
                OsString::from("GIT_CONFIG_GLOBAL"),
                self.base.join("gitconfig").into_os_string(),
            ),
            (
                OsString::from("GIT_CEILING_DIRECTORIES"),
                self.base.clone().into_os_string(),
            ),
            var("GIT_AUTHOR_NAME", "planrun test"),
            var("GIT_AUTHOR_EMAIL", "test@planrun.invalid"),
            var("GIT_COMMITTER_NAME", "planrun test"),
            var("GIT_COMMITTER_EMAIL", "test@planrun.invalid"),
        ]
    }

    /// Runs git in the copy with [`Fixture::env`]; panics when it fails. Returns stdout.
    pub fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.repo)
            .envs(self.env())
            .output()
            .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// Writes `bytes` to `rel` (forward slashes) in the copy, creating directories.
    pub fn write(&self, rel: &str, bytes: &[u8]) {
        let path = self.repo.join(rel);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("{e}"));
        }
        std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }

    /// The bytes of `rel`, or `None` when it does not exist.
    pub fn read(&self, rel: &str) -> Option<Vec<u8>> {
        std::fs::read(self.repo.join(rel)).ok()
    }

    /// Every file under the copy, `.git` included: relative path → contents.
    pub fn snapshot(&self) -> BTreeMap<String, Vec<u8>> {
        let mut files = BTreeMap::new();
        collect(&self.repo, &self.repo, &mut files);
        files
    }

    /// `git status --porcelain` without refreshing the index.
    pub fn status(&self) -> String {
        self.git(&["--no-optional-locks", "status", "--porcelain"])
    }

    /// Deletes the tracked files and checks them out again, so that `core.autocrlf` applies.
    fn checkout_again(&self) {
        let listed = self.git(&["ls-files", "-z"]);
        for file in listed.split('\0').filter(|f| !f.is_empty()) {
            let path = self.repo.join(file);
            std::fs::remove_file(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        }
        self.git(&["checkout", "--", "."]);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        clear_read_only(&self.base);
        if let Err(e) = std::fs::remove_dir_all(&self.base) {
            eprintln!(
                "could not remove the fixture copy {}: {e}",
                self.base.display()
            );
        }
    }
}

/// The path canonicalized, or as given when that fails.
pub fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Copies the fixture as git stores it: its files are text with LF line endings in this
/// repository, so CRLF from a checkout with `core.autocrlf=true` is turned back into LF, and the
/// copy's blob ids are the ones the plans name (`aa80cf1`). A `target/` from building the
/// fixture is skipped.
fn copy_dir(from: &Path, to: &Path) {
    let entries = std::fs::read_dir(from).unwrap_or_else(|e| panic!("{}: {e}", from.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("{e}"));
        let target = to.join(entry.file_name());
        let kind = entry.file_type().unwrap_or_else(|e| panic!("{e}"));
        if kind.is_dir() && entry.file_name() == "target" {
            continue;
        }
        if kind.is_dir() {
            std::fs::create_dir_all(&target).unwrap_or_else(|e| panic!("{e}"));
            copy_dir(&entry.path(), &target);
            continue;
        }
        let bytes = std::fs::read(entry.path()).unwrap_or_else(|e| panic!("{e}"));
        let text = String::from_utf8(bytes).unwrap_or_else(|e| panic!("the fixture is text: {e}"));
        std::fs::write(&target, text.replace("\r\n", "\n")).unwrap_or_else(|e| panic!("{e}"));
    }
}

fn collect(root: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, files);
            continue;
        }
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let key = rel.to_string_lossy().replace('\\', "/");
        files.insert(key, std::fs::read(&path).unwrap_or_default());
    }
}

/// git writes its object files read-only; Windows refuses to delete those.
fn clear_read_only(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            clear_read_only(&path);
            continue;
        }
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        let mut permissions = meta.permissions();
        if permissions.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = std::fs::set_permissions(&path, permissions);
        }
    }
}
