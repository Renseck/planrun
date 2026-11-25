//! Repository-relative paths: the only path type an action may carry (decision D7).

use std::fmt;

use serde::{Deserialize, Serialize};

/// A path relative to the repository root, with forward slashes, that cannot leave the
/// repository by its spelling: no absolute path, no drive letter, no `..`, no `.` segment, no
/// empty segment, no backslash, no NUL, no wildcard (`*`, `?`: decision D42). Nor a spelling
/// Windows reserves or git refuses (decision D51): no `:`, `<`, `>`, `"`, `|` or control
/// character, no segment that is `.git` (in any case, with trailing dots or spaces) or its 8.3
/// alias `git~1`, no device name (`con`, `nul.txt`, `com1`, …), no segment ending in a dot or a
/// space.
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
    /// A `*` or `?`: a pattern, never a file (decisions D41, D42).
    #[error("`{0}` contains a wildcard; a pattern is not a file path")]
    Wildcard(String),
    /// `:`, `<`, `>`, `"`, `|` or a control character (decision D51).
    #[error("`{0}` contains a character Windows reserves in file names")]
    Reserved(String),
    /// A segment that is a git directory: `.git` in any case, with trailing dots or spaces, or
    /// `git~1` (decision D51).
    #[error("`{0}` names a git directory; a plan never writes into one")]
    GitDir(String),
    /// A segment that is a Windows device name, with or without an extension (decision D51).
    #[error("`{0}` has a segment Windows reserves as a device name")]
    DeviceName(String),
    /// A segment that ends in a dot or a space, which Windows drops (decision D51).
    #[error("`{0}` has a segment that ends in a dot or a space")]
    TrailingDotOrSpace(String),
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
        if path.contains(['*', '?']) {
            return Err(RepoPathError::Wildcard(path.to_owned()));
        }
        let bytes = path.as_bytes();
        if path.starts_with('/') || (bytes.len() >= 2 && bytes[1] == b':') {
            return Err(RepoPathError::Absolute(path.to_owned()));
        }
        if path.contains(|c: char| matches!(c, ':' | '<' | '>' | '"' | '|') || c < ' ') {
            return Err(RepoPathError::Reserved(path.to_owned()));
        }
        for segment in path.split('/') {
            check_segment(path, segment)?;
        }
        Ok(Self(path.into()))
    }

    /// The path with forward slashes.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The last segment (`lib.rs` of `src/lib.rs`).
    pub fn file_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }

    /// The directory, without a trailing slash; `None` at the repository root.
    pub fn dir(&self) -> Option<&str> {
        self.0.rsplit_once('/').map(|(dir, _)| dir)
    }
}

/// Refuses a segment that is empty, `.` or `..`, a git directory, a device name, or ends in a
/// dot or a space.
fn check_segment(path: &str, segment: &str) -> Result<(), RepoPathError> {
    if segment.is_empty() || segment == "." || segment == ".." {
        return Err(RepoPathError::BadSegment(path.to_owned()));
    }
    let trimmed = segment.trim_end_matches(['.', ' ']).to_ascii_lowercase();
    if trimmed == ".git" || trimmed == "git~1" {
        return Err(RepoPathError::GitDir(path.to_owned()));
    }
    if is_device_name(segment) {
        return Err(RepoPathError::DeviceName(path.to_owned()));
    }
    if segment.ends_with(['.', ' ']) {
        return Err(RepoPathError::TrailingDotOrSpace(path.to_owned()));
    }
    Ok(())
}

/// True for `con`, `prn`, `aux`, `nul`, `conin$`, `conout$`, `com0`–`com9`, `lpt0`–`lpt9` and
/// the superscript forms `com¹`–`com³`, `lpt¹`–`lpt³`, in any case, alone or before an extension
/// (`nul.txt`, `con .rs`): Windows opens the device instead of the file.
fn is_device_name(segment: &str) -> bool {
    let base = segment.split('.').next().unwrap_or(segment);
    let base = base.trim_end_matches(' ').to_lowercase();
    if matches!(
        base.as_str(),
        "con" | "prn" | "aux" | "nul" | "conin$" | "conout$"
    ) {
        return true;
    }
    let Some(number) = base
        .strip_prefix("com")
        .or_else(|| base.strip_prefix("lpt"))
    else {
        return false;
    };
    let mut chars = number.chars();
    let digit = chars
        .next()
        .is_some_and(|c| c.is_ascii_digit() || matches!(c, '¹' | '²' | '³'));
    digit && chars.next().is_none()
}

/// Where a path in the IR came from, when the plan did not write it in full (decision D32).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathOrigin {
    /// Written in full where it is used.
    #[default]
    Written,
    /// A manifest token without a directory, placed in the directory of the token before it.
    Sibling,
    /// A lead that names only a file name: the one manifest entry with that file name.
    FileName,
    /// A lead that names no path: the task's one editable manifest entry.
    OnlyEntry,
    /// A manifest token without a directory after a token with one, in a format without
    /// siblings (decision D45): spelled as written, its location unknown. Never resolved to,
    /// and a Create onto it is `Manual(AmbiguousPath)`.
    Bare,
    /// An edit with no path that continues a numbered edit series: the path of the edit
    /// directly before it (decision D49).
    PreviousEdit,
}

impl PathOrigin {
    /// True when the plan wrote the path in full.
    pub fn is_written(&self) -> bool {
        *self == PathOrigin::Written
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
            "a/*.rs",
            "a/b?.rs",
        ] {
            assert!(RepoPath::new(bad).is_err(), "{bad:?}");
        }
    }

    fn refused(path: &str) -> Option<RepoPathError> {
        RepoPath::new(path).err()
    }

    #[test]
    fn reserved_characters_are_refused() {
        for bad in [
            "a.rs:x",
            "src/a:b.rs",
            "a<b",
            "a>b",
            "a\"b",
            "a|b",
            "a\tb",
            "a\u{1f}b",
        ] {
            assert_eq!(
                refused(bad),
                Some(RepoPathError::Reserved(bad.to_owned())),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn git_directories_are_refused_in_any_spelling() {
        for bad in [
            ".git/hooks/pre-commit",
            ".GIT/config",
            "a/.Git/x",
            ".git./x",
            ".git /x",
            ".git. ./x",
            "git~1/config",
            "a/GIT~1/x",
        ] {
            assert_eq!(
                refused(bad),
                Some(RepoPathError::GitDir(bad.to_owned())),
                "{bad:?}"
            );
        }
        for ok in [
            ".gitattributes",
            ".gitignore",
            ".gitmodules",
            "a/.github/x",
            "git/x",
        ] {
            assert!(RepoPath::new(ok).is_ok(), "{ok:?}");
        }
    }

    #[test]
    fn device_names_are_refused_with_or_without_an_extension() {
        for bad in [
            "con",
            "CON",
            "a/prn.txt",
            "aux.rs",
            "Nul",
            "nul.tar.gz",
            "con .rs",
            "com0",
            "COM9",
            "lpt1.log",
            "com¹",
            "LPT³.txt",
            "conin$",
            "CONOUT$.x",
        ] {
            assert_eq!(
                refused(bad),
                Some(RepoPathError::DeviceName(bad.to_owned())),
                "{bad:?}"
            );
        }
        for ok in [
            "cons",
            "console.rs",
            "a/nullable.rs",
            "com10",
            "lpt",
            "com",
            "auxiliary",
        ] {
            assert!(RepoPath::new(ok).is_ok(), "{ok:?}");
        }
    }

    #[test]
    fn segments_ending_in_a_dot_or_a_space_are_refused() {
        for bad in ["a.", "a /b", "dir./x", "x/y "] {
            assert_eq!(
                refused(bad),
                Some(RepoPathError::TrailingDotOrSpace(bad.to_owned())),
                "{bad:?}"
            );
        }
        assert!(RepoPath::new("a.b/c").is_ok());
    }
}
