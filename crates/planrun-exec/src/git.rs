//! A thin wrapper over the `git` binary (docs/architecture.md §6.2). Every call is one process,
//! run at the repository root with `GIT_LITERAL_PATHSPECS=1` (a path is never a pathspec
//! pattern) and the wrapper's own environment. The virtual tree (D52) passes its index and
//! object directory per call; no other call sees them.

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use planrun_model::{AbbrevBlobId, RepoPath};

/// A full SHA-1 object id (planrun-exec refuses SHA-256 repositories: plans carry SHA-1
/// abbreviations).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlobId([u8; 20]);

impl BlobId {
    /// Parses 40 hex digits (surrounding whitespace ignored).
    pub fn parse(hex: &str) -> Option<Self> {
        let hex = hex.trim();
        if hex.len() != 40 || !hex.is_ascii() {
            return None;
        }
        let mut bytes = [0u8; 20];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()?;
        }
        Some(Self(bytes))
    }

    /// True when the abbreviation from a plan's `index` line names this object.
    pub fn matches(&self, abbrev: &AbbrevBlobId) -> bool {
        abbrev.matches(&self.0)
    }
}

impl fmt::Display for BlobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|b| write!(f, "{b:02x}"))
    }
}

/// 40 lowercase hex digits in the journal (D76).
impl serde::Serialize for BlobId {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for BlobId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let hex = String::deserialize(d)?;
        BlobId::parse(&hex).ok_or_else(|| serde::de::Error::custom(format!("not a blob id: {hex}")))
    }
}

/// HEAD as git reports it: the commit (none in an empty repository) and the ref it points to
/// (none when detached).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Head {
    /// The commit id.
    pub commit: Option<String>,
    /// `refs/heads/main`.
    pub reference: Option<String>,
}

/// Why a git call failed.
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    /// git could not be started (not installed, not on `PATH`).
    #[error("could not run git: {0}")]
    Spawn(#[source] std::io::Error),
    /// git ran and exited non-zero.
    #[error("`git {args}` failed (exit code {code:?}): {stderr}")]
    Failed {
        /// The arguments, joined.
        args: String,
        /// The exit code; `None` when git was ended by a signal.
        code: Option<i32>,
        /// What git wrote to stderr, trimmed.
        stderr: String,
    },
    /// git printed something this wrapper cannot read.
    #[error("unexpected output from `git {args}`: {output}")]
    Output {
        /// The arguments, joined.
        args: String,
        /// The output.
        output: String,
    },
}

/// Extra environment for one call: `(name, value)` pairs.
pub type Env<'a> = &'a [(OsString, OsString)];

/// One line of `git status --porcelain -z`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    /// The two status letters (`??`, ` M`, `A `, …).
    pub code: String,
    /// The path, with forward slashes.
    pub path: String,
}

/// The `eol` and `text` attributes of one path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attributes {
    /// `lf`, `crlf`, or `None` when unspecified.
    pub eol: Option<String>,
    /// `false` when `text` is unset (`-text`, `binary`); `true` otherwise.
    pub text: bool,
}

/// The git binary, bound to one repository.
#[derive(Debug, Clone)]
pub struct Git {
    root: PathBuf,
    env: Vec<(OsString, OsString)>,
}

impl Git {
    /// git at `root` (the work-tree root), with `env` added to every call.
    pub fn new(root: &Path, env: Vec<(OsString, OsString)>) -> Self {
        Self {
            root: root.to_path_buf(),
            env,
        }
    }

    /// The work-tree root that contains `dir` (`rev-parse --show-toplevel`).
    pub fn toplevel(dir: &Path, env: Env<'_>) -> Result<PathBuf, GitError> {
        let probe = Self::new(dir, env.to_vec());
        let out = probe.run(&["rev-parse", "--show-toplevel"], None, &[])?;
        Ok(PathBuf::from(text(&out).trim()))
    }

    /// The repository root this wrapper runs in.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Runs `git <args>` at the root; `stdin` is written to its standard input. Returns stdout.
    pub fn run(
        &self,
        args: &[&str],
        stdin: Option<&[u8]>,
        extra: Env<'_>,
    ) -> Result<Vec<u8>, GitError> {
        let mut command = Command::new("git");
        command
            .args(args)
            .current_dir(&self.root)
            .env("GIT_LITERAL_PATHSPECS", "1")
            .envs(self.env.iter().map(|(k, v)| (k, v)))
            .envs(extra.iter().map(|(k, v)| (k, v)))
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().map_err(GitError::Spawn)?;
        if let (Some(bytes), Some(mut pipe)) = (stdin, child.stdin.take()) {
            pipe.write_all(bytes).map_err(GitError::Spawn)?;
        }
        let out = child.wait_with_output().map_err(GitError::Spawn)?;
        if !out.status.success() {
            return Err(GitError::Failed {
                args: args.join(" "),
                code: out.status.code(),
                stderr: text(&out.stderr).trim().to_owned(),
            });
        }
        Ok(out.stdout)
    }

    /// `rev-parse <args>` as one trimmed line.
    pub fn rev_parse(&self, args: &[&str]) -> Result<String, GitError> {
        let mut all = vec!["rev-parse"];
        all.extend_from_slice(args);
        Ok(text(&self.run(&all, None, &[])?).trim().to_owned())
    }

    /// The absolute path of this work tree's git directory (`--absolute-git-dir`; a linked
    /// worktree has its own).
    pub fn git_dir(&self) -> Result<PathBuf, GitError> {
        Ok(PathBuf::from(self.rev_parse(&["--absolute-git-dir"])?))
    }

    /// HEAD: `rev-parse -q --verify HEAD` (none when unborn) and `symbolic-ref -q HEAD` (none
    /// when detached).
    pub fn head(&self) -> Result<Head, GitError> {
        let optional = |args: &[&str]| match self.run(args, None, &[]) {
            Ok(out) => Ok(Some(text(&out).trim().to_owned())),
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(e) => Err(e),
        };
        Ok(Head {
            commit: optional(&["rev-parse", "-q", "--verify", "HEAD"])?,
            reference: optional(&["symbolic-ref", "-q", "HEAD"])?,
        })
    }

    /// The first parent of `commit`; `None` for a root commit.
    pub fn parent(&self, commit: &str) -> Result<Option<String>, GitError> {
        let spec = format!("{commit}^");
        match self.run(&["rev-parse", "-q", "--verify", &spec], None, &[]) {
            Ok(out) => Ok(Some(text(&out).trim().to_owned())),
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// The blob id of each of `paths` in `commit`'s tree, in order; `None` where the tree has
    /// no file there.
    pub fn tree_blobs(
        &self,
        commit: &str,
        paths: &[&RepoPath],
    ) -> Result<Vec<Option<BlobId>>, GitError> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }
        let mut args = vec!["ls-tree", "-r", "-z", commit, "--"];
        args.extend(paths.iter().map(|p| p.as_str()));
        let out = text(&self.run(&args, None, &[])?).into_owned();
        let mut found = vec![None; paths.len()];
        for entry in out.split('\0').filter(|e| !e.is_empty()) {
            let Some((meta, path)) = entry.split_once('\t') else {
                return Err(output_error(&args, entry));
            };
            let id = meta.split_whitespace().nth(2).and_then(BlobId::parse);
            if let Some(i) = paths.iter().position(|p| p.as_str() == path) {
                found[i] = id;
            }
        }
        Ok(found)
    }

    /// The paths that differ between the trees of `from` and `to`
    /// (`diff-tree -r --name-only -z`).
    pub fn changed_paths(&self, from: &str, to: &str) -> Result<Vec<String>, GitError> {
        let out = self.run(
            &["diff-tree", "-r", "--name-only", "-z", from, to],
            None,
            &[],
        )?;
        Ok(text(&out)
            .split('\0')
            .filter(|p| !p.is_empty())
            .map(str::to_owned)
            .collect())
    }

    /// The absolute path of the repository's object directory.
    pub fn object_dir(&self) -> Result<PathBuf, GitError> {
        let objects = PathBuf::from(self.rev_parse(&["--git-path", "objects"])?);
        Ok(if objects.is_absolute() {
            objects
        } else {
            self.root.join(objects)
        })
    }

    /// A boolean config value; `None` when unset.
    pub fn config_bool(&self, key: &str) -> Result<Option<bool>, GitError> {
        match self.run(&["config", "--bool", "--get", key], None, &[]) {
            Ok(out) => Ok(Some(text(&out).trim() == "true")),
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// `git --no-optional-locks status --porcelain -z --untracked-files=all [-- paths]`: never
    /// refreshes the index, so it writes nothing (D52).
    pub fn status(&self, paths: Option<&[&RepoPath]>) -> Result<Vec<StatusEntry>, GitError> {
        let mut args = vec![
            "--no-optional-locks",
            "status",
            "--porcelain",
            "-z",
            "--untracked-files=all",
        ];
        if let Some(paths) = paths {
            if paths.is_empty() {
                return Ok(Vec::new());
            }
            args.push("--");
            args.extend(paths.iter().map(|p| p.as_str()));
        }
        let out = self.run(&args, None, &[])?;
        Ok(parse_status(&text(&out)))
    }

    /// The blob id git gives `bytes` at `path`. `filters` applies the clean filters and
    /// `autocrlf` of that path (`--path`); without it the bytes are hashed as they are
    /// (`--no-filters`). `write` stores the object (`-w`) in the object database `extra` selects.
    pub fn hash_object(
        &self,
        path: &RepoPath,
        bytes: &[u8],
        filters: bool,
        write: bool,
        extra: Env<'_>,
    ) -> Result<BlobId, GitError> {
        let path_arg = format!("--path={path}");
        let mut args = vec!["hash-object"];
        if write {
            args.push("-w");
        }
        args.push(if filters { &path_arg } else { "--no-filters" });
        args.push("--stdin");
        let out = self.run(&args, Some(bytes), extra)?;
        blob_id(&args, &out)
    }

    /// The blob ids of the files at `paths`, in one `hash-object --stdin-paths` call, nothing
    /// stored: git's view with `filters` (what a commit would hold), the exact bytes
    /// (`--no-filters`) without. Every path must exist: git fails the whole batch on a missing
    /// one, so callers stat first.
    pub fn hash_paths(&self, paths: &[&RepoPath], filters: bool) -> Result<Vec<BlobId>, GitError> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }
        let args: &[&str] = if filters {
            &["hash-object", "--stdin-paths"]
        } else {
            &["hash-object", "--no-filters", "--stdin-paths"]
        };
        let input: String = paths.iter().map(|p| format!("{p}\n")).collect();
        let out = self.run(args, Some(input.as_bytes()), &[])?;
        let ids: Vec<BlobId> = text(&out)
            .lines()
            .map(|line| BlobId::parse(line).ok_or_else(|| output_error(args, line)))
            .collect::<Result<_, _>>()?;
        if ids.len() != paths.len() {
            return Err(output_error(args, &text(&out)));
        }
        Ok(ids)
    }

    /// The ids among `ids` that the object database does not have (pruned by `git gc`, D57),
    /// from one `cat-file --batch-check` call.
    pub fn missing_objects(&self, ids: &[BlobId]) -> Result<Vec<BlobId>, GitError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let input: String = ids.iter().map(|id| format!("{id}\n")).collect();
        let out = self.run(
            &["cat-file", "--batch-check=%(objectname)"],
            Some(input.as_bytes()),
            &[],
        )?;
        let out = text(&out);
        Ok(ids
            .iter()
            .zip(out.lines())
            .filter(|(_, line)| line.ends_with(" missing"))
            .map(|(id, _)| *id)
            .collect())
    }

    /// The bytes of blob `id`, unconverted.
    pub fn cat_blob(&self, id: &BlobId, extra: Env<'_>) -> Result<Vec<u8>, GitError> {
        self.run(&["cat-file", "blob", &id.to_string()], None, extra)
    }

    /// The bytes of blob `id` as a checkout of `path` would write them (`cat-file --filters`:
    /// smudge filters and line-ending conversion, as `git apply` does).
    pub fn cat_filtered(
        &self,
        id: &BlobId,
        path: &RepoPath,
        extra: Env<'_>,
    ) -> Result<Vec<u8>, GitError> {
        let path_arg = format!("--path={path}");
        self.run(
            &["cat-file", "--filters", &path_arg, &id.to_string()],
            None,
            extra,
        )
    }

    /// `git apply --whitespace=nowarn [--cached] [--check] -` with the patch on stdin (D20).
    pub fn apply(
        &self,
        patch: &[u8],
        cached: bool,
        check: bool,
        extra: Env<'_>,
    ) -> Result<(), GitError> {
        let mut args = vec!["apply", "--whitespace=nowarn"];
        if cached {
            args.push("--cached");
        }
        if check {
            args.push("--check");
        }
        args.push("-");
        self.run(&args, Some(patch), extra).map(|_| ())
    }

    /// The `eol` and `text` attributes of each path, in order.
    pub fn attributes(&self, paths: &[&RepoPath]) -> Result<Vec<Attributes>, GitError> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }
        let mut args = vec!["check-attr", "-z", "eol", "text", "--"];
        args.extend(paths.iter().map(|p| p.as_str()));
        let out = text(&self.run(&args, None, &[])?).into_owned();
        let fields: Vec<&str> = out.split('\0').collect();
        let mut found = vec![Attributes::default(); paths.len()];
        for triple in fields.chunks_exact(3) {
            let [path, attr, value] = triple else {
                continue;
            };
            let Some(i) = paths.iter().position(|p| p.as_str() == *path) else {
                continue;
            };
            apply_attribute(&mut found[i], attr, value);
        }
        Ok(found)
    }

    /// The mode and blob id of `path` in the index `extra` selects; `None` when not in it.
    pub fn index_entry(
        &self,
        path: &RepoPath,
        extra: Env<'_>,
    ) -> Result<Option<(String, BlobId)>, GitError> {
        let args = ["ls-files", "-s", "-z", "--", path.as_str()];
        let out = text(&self.run(&args, None, extra)?).into_owned();
        let Some(line) = out.split('\0').find(|l| !l.is_empty()) else {
            return Ok(None);
        };
        let mut parts = line.split_whitespace();
        let (Some(mode), Some(id)) = (parts.next(), parts.next()) else {
            return Err(output_error(&args, line));
        };
        let id = BlobId::parse(id).ok_or_else(|| output_error(&args, line))?;
        Ok(Some((mode.to_owned(), id)))
    }

    /// Puts `id` at `path` in the index `extra` selects.
    pub fn index_add(
        &self,
        path: &RepoPath,
        mode: &str,
        id: &BlobId,
        extra: Env<'_>,
    ) -> Result<(), GitError> {
        let info = format!("{mode},{id},{path}");
        self.run(
            &["update-index", "--add", "--cacheinfo", &info],
            None,
            extra,
        )
        .map(|_| ())
    }

    /// Removes `paths` from the index `extra` selects, in one call (one index write).
    pub fn index_remove(&self, paths: &[&RepoPath], extra: Env<'_>) -> Result<(), GitError> {
        if paths.is_empty() {
            return Ok(());
        }
        let mut args = vec!["update-index", "--force-remove", "--"];
        args.extend(paths.iter().map(|p| p.as_str()));
        self.run(&args, None, extra).map(|_| ())
    }
}

fn apply_attribute(found: &mut Attributes, attr: &str, value: &str) {
    match (attr, value) {
        ("eol", "lf" | "crlf") => found.eol = Some(value.to_owned()),
        ("text", "unset") => found.text = false,
        ("text", _) => found.text = true,
        _ => {}
    }
}

fn text(bytes: &[u8]) -> std::borrow::Cow<'_, str> {
    String::from_utf8_lossy(bytes)
}

fn output_error(args: &[&str], output: &str) -> GitError {
    GitError::Output {
        args: args.join(" "),
        output: output.to_owned(),
    }
}

fn blob_id(args: &[&str], out: &[u8]) -> Result<BlobId, GitError> {
    let line = text(out);
    BlobId::parse(&line).ok_or_else(|| output_error(args, &line))
}

/// Parses `status --porcelain -z`: `XY path\0`, and for renames and copies `XY new\0old\0`.
fn parse_status(out: &str) -> Vec<StatusEntry> {
    let mut entries = Vec::new();
    let mut fields = out.split('\0').filter(|f| !f.is_empty());
    while let Some(field) = fields.next() {
        let (code, path) = field.split_at(field.len().min(3));
        let code = code.get(..2).unwrap_or(code).to_owned();
        if code.starts_with(['R', 'C']) {
            fields.next();
        }
        entries.push(StatusEntry {
            code,
            path: path.to_owned(),
        });
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blob_id_round_trips_and_matches_its_abbreviation() {
        let hex = "97bd92a0123456789abcdef0123456789abcdef0";
        let id = BlobId::parse(hex).unwrap_or_else(|| panic!("{hex}"));
        assert_eq!(id.to_string(), hex);
        let abbrev = AbbrevBlobId::parse("97bd92a").unwrap_or_else(|e| panic!("{e}"));
        assert!(id.matches(&abbrev));
        assert!(BlobId::parse("97bd92a").is_none());
        assert!(BlobId::parse(&"g".repeat(40)).is_none());
    }

    #[test]
    fn status_reads_renames_as_one_entry() {
        let out = " M src/a.rs\0R  new.rs\0old.rs\0?? x y.txt\0";
        let entries = parse_status(out);
        let paths: Vec<_> = entries
            .iter()
            .map(|e| (e.code.as_str(), e.path.as_str()))
            .collect();
        assert_eq!(
            paths,
            [(" M", "src/a.rs"), ("R ", "new.rs"), ("??", "x y.txt")]
        );
    }
}
