//! Repository-relative paths: the only path type an action may carry (decision D7).

use std::fmt;

use serde::{Deserialize, Serialize};

/// A path relative to the repository root, with forward slashes, that cannot leave the
/// repository by its spelling: no absolute path, no drive letter, no `..`, no `.` segment, no
/// empty segment, no backslash, no NUL.
///
/// This checks the spelling only. Symlinks that point out of the repository are refused by the
/// workspace when it resolves the path on disk (planrun-exec).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RepoPath(Box<str>);

/// Why a path was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RepoPathError {
    /// Empty input.
    #[error("the path is empty")]
    Empty,
    /// `/x`, `C:\x`, `C:x` or `\\server\x`.
    #[error("`{0}` is absolute; plan paths are relative to the repository root")]
    Absolute(String),
    /// A `\` in the path.
    #[error("`{0}` contains a backslash; plan paths use forward slashes")]
    Backslash(String),
    /// A `..`, `.` or empty segment.
    #[error("`{0}` has a `.`, `..` or empty segment")]
    BadSegment(String),
    /// A NUL byte.
    #[error("the path contains a NUL byte")]
    Nul,
}

impl RepoPath {
    /// Validates `path` as written in a plan.
    pub fn new(path: &str) -> Result<Self, RepoPathError> {
        if path.is_empty() {
            return Err(RepoPathError::Empty);
        }
        if path.contains('\0') {
            return Err(RepoPathError::Nul);
        }
        if path.contains('\\') {
            return Err(RepoPathError::Backslash(path.to_owned()));
        }
        let bytes = path.as_bytes();
        if path.starts_with('/') || (bytes.len() >= 2 && bytes[1] == b':') {
            return Err(RepoPathError::Absolute(path.to_owned()));
        }
        if path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        {
            return Err(RepoPathError::BadSegment(path.to_owned()));
        }
        Ok(Self(path.into()))
    }

    /// The path with forward slashes.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RepoPath {
    type Error = RepoPathError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<RepoPath> for String {
    fn from(value: RepoPath) -> Self {
        value.0.into()
    }
}

impl fmt::Display for RepoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_are_accepted() {
        for ok in [
            "a.rs",
            "crates/x/src/lib.rs",
            ".claude-plugin/plugin.json",
            "a/.x",
        ] {
            assert_eq!(RepoPath::new(ok).map(|p| p.to_string()), Ok(ok.to_owned()));
        }
    }

    #[test]
    fn escaping_or_ambiguous_paths_are_refused() {
        for bad in [
            "",
            "/etc/passwd",
            "C:\\x",
            "C:x",
            "a\\b",
            "../x",
            "a/../b",
            "./a",
            "a//b",
            "a/",
        ] {
            assert!(RepoPath::new(bad).is_err(), "{bad:?}");
        }
    }
}
