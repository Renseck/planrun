//! Manifest patterns (decision D41): a glob or a range a plan writes in a Files list instead of
//! one file. A pattern is never a write target: actions carry a [`RepoPath`], which refuses
//! wildcards (D42), and a manifest entry is either a file or a pattern ([`ManifestPath`]).

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::path::RepoPath;

/// A glob (`dir/*.sql`, `functions/{a,b}.ps1`) or a range (`dir/0001`–`0007_*.sql`), relative to
/// the repository root as written and never resolved to files (D41).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PathPattern {
    text: Box<str>,
    kind: PatternKind,
}

/// The two kinds of pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum PatternKind {
    /// The alternatives of the glob after brace expansion.
    Glob(Vec<Box<str>>),
    /// Every file in `dir` whose name lies between `start` and `end`.
    Range {
        dir: Box<str>,
        start: Box<str>,
        end: Box<str>,
    },
}

/// Why a pattern was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PatternError {
    /// The spelling is no repository path (absolute, `..`, backslash, …).
    #[error("`{0}` is not a repository-relative pattern")]
    Spelling(String),
    /// `**`, which would cross directories.
    #[error("`{0}` uses `**`; a pattern matches within one directory level")]
    DoubleStar(String),
    /// `{` inside `{…}`, or an unbalanced brace.
    #[error("`{0}` has nested or unbalanced braces")]
    Braces(String),
    /// A range whose ends are in different directories, or whose start has a wildcard.
    #[error("the range `{0}`–`{1}` must name two file names in one directory")]
    Range(String, String),
}

impl PathPattern {
    /// True when a manifest token is a glob: it contains `*`, `?` or `{`.
    pub fn is_glob(token: &str) -> bool {
        token.contains(['*', '?', '{'])
    }

    /// A glob. `*` matches any run of characters except `/`, `?` one character except `/`, and
    /// `{a,b}` either alternative (no nesting).
    pub fn glob(text: &str) -> Result<Self, PatternError> {
        if text.contains("**") {
            return Err(PatternError::DoubleStar(text.to_owned()));
        }
        let alternatives = expand_braces(text)?;
        for alternative in &alternatives {
            check_spelling(alternative)?;
        }
        Ok(Self {
            text: text.into(),
            kind: PatternKind::Glob(alternatives.into_iter().map(Into::into).collect()),
        })
    }

    /// A range from `start` (with a directory) to `end` (a file name in the same directory, or
    /// a path with that same directory). `start` has no wildcard; `end` may have one.
    pub fn range(start: &str, end: &str) -> Result<Self, PatternError> {
        let refused = || PatternError::Range(start.to_owned(), end.to_owned());
        let start_path = RepoPath::new(start).map_err(|_| refused())?;
        let dir = start_path.dir().ok_or_else(refused)?;
        let end_name = match end.rsplit_once('/') {
            Some((end_dir, name)) if end_dir == dir => name,
            Some(_) => return Err(refused()),
            None => end,
        };
        if end_name.is_empty() || end_name.contains("**") || end_name.contains(['{', '}']) {
            return Err(refused());
        }
        check_spelling(&format!("{dir}/{end_name}"))?;
        Ok(Self {
            text: format!("{start}–{end_name}").into(),
            kind: PatternKind::Range {
                dir: dir.into(),
                start: start_path.file_name().into(),
                end: end_name.into(),
            },
        })
    }

    /// The pattern as written (a range as `start–end`).
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Whether `path` is one of the files the pattern names.
    ///
    /// A range `dir/A`–`B` matches a path in `dir` whose file name `f` has `A <= f`. When `B` has
    /// no wildcard, also `f <= B`. Otherwise, with `B0` the part of `B` before its first
    /// wildcard: `f` has at least `|B0|` characters, its first `|B0|` characters are `<= B0`, and
    /// the rest of `f` matches the rest of `B` as a glob. Comparisons are by character.
    pub fn matches(&self, path: &RepoPath) -> bool {
        match &self.kind {
            PatternKind::Glob(alternatives) => {
                alternatives.iter().any(|a| glob_match(a, path.as_str()))
            }
            PatternKind::Range { dir, start, end } => {
                path.dir() == Some(&**dir) && in_range(start, end, path.file_name())
            }
        }
    }
}

impl fmt::Display for PathPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// What a manifest entry declares: one file, or the files a pattern names.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManifestPath {
    /// One file.
    File(RepoPath),
    /// A glob or a range (D41).
    Pattern(PathPattern),
}

impl ManifestPath {
    /// The file, when the entry names one.
    pub fn file(&self) -> Option<&RepoPath> {
        match self {
            ManifestPath::File(path) => Some(path),
            ManifestPath::Pattern(_) => None,
        }
    }

    /// Whether `path` is declared by this entry: equal to its file, or matched by its pattern.
    pub fn covers(&self, path: &RepoPath) -> bool {
        match self {
            ManifestPath::File(file) => file == path,
            ManifestPath::Pattern(pattern) => pattern.matches(path),
        }
    }
}

impl fmt::Display for ManifestPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestPath::File(path) => path.fmt(f),
            ManifestPath::Pattern(pattern) => pattern.fmt(f),
        }
    }
}

impl fmt::Debug for ManifestPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestPath::File(path) => path.fmt(f),
            ManifestPath::Pattern(pattern) => pattern.fmt(f),
        }
    }
}

/// The spelling rules of [`RepoPath`] with the wildcards replaced by a plain character.
fn check_spelling(text: &str) -> Result<(), PatternError> {
    let plain = text.replace(['*', '?'], "x");
    RepoPath::new(&plain)
        .map(|_| ())
        .map_err(|_| PatternError::Spelling(text.to_owned()))
}

/// `a{b,c}d` → `abd`, `acd`; at most one brace group per level, never nested.
fn expand_braces(text: &str) -> Result<Vec<String>, PatternError> {
    let bad = || PatternError::Braces(text.to_owned());
    let Some(open) = text.find('{') else {
        if text.contains('}') {
            return Err(bad());
        }
        return Ok(vec![text.to_owned()]);
    };
    let close = text[open..].find('}').map(|i| open + i).ok_or_else(bad)?;
    let inner = &text[open + 1..close];
    if inner.contains('{') {
        return Err(bad());
    }
    let (head, tail) = (&text[..open], &text[close + 1..]);
    let tails = expand_braces(tail)?;
    Ok(inner
        .split(',')
        .flat_map(|alt| tails.iter().map(move |t| format!("{head}{alt}{t}")))
        .collect())
}

/// `*` and `?` never match `/`.
fn glob_match(pattern: &str, text: &str) -> bool {
    let (p, t): (Vec<char>, Vec<char>) = (pattern.chars().collect(), text.chars().collect());
    glob_chars(&p, &t)
}

fn glob_chars(p: &[char], t: &[char]) -> bool {
    let Some((&first, rest)) = p.split_first() else {
        return t.is_empty();
    };
    match first {
        '*' => (0..=t.len())
            .take_while(|&k| k == 0 || t[k - 1] != '/')
            .any(|k| glob_chars(rest, &t[k..])),
        '?' => t.first().is_some_and(|&c| c != '/') && glob_chars(rest, &t[1..]),
        c => t.first() == Some(&c) && glob_chars(rest, &t[1..]),
    }
}

fn in_range(start: &str, end: &str, name: &str) -> bool {
    if name < start {
        return false;
    }
    let Some(wild) = end.find(['*', '?']) else {
        return name <= end;
    };
    let (literal, rest) = end.split_at(wild);
    let count = literal.chars().count();
    let chars: Vec<char> = name.chars().collect();
    if chars.len() < count {
        return false;
    }
    let head: String = chars[..count].iter().collect();
    let tail: String = chars[count..].iter().collect();
    head.as_str() <= literal && glob_match(rest, &tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(p: &str) -> RepoPath {
        RepoPath::new(p).unwrap_or_else(|e| panic!("{e}"))
    }

    fn glob(p: &str) -> PathPattern {
        PathPattern::glob(p).unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn globs_match_within_one_segment() {
        let g = glob("dir/*.sql");
        assert!(g.matches(&path("dir/a.sql")));
        assert!(!g.matches(&path("dir/sub/a.sql")));
        assert!(!g.matches(&path("other/a.sql")));
        assert!(glob("dir/a?.rs").matches(&path("dir/ab.rs")));
        assert!(!glob("dir/a?.rs").matches(&path("dir/a.rs")));
    }

    #[test]
    fn braces_expand_to_alternatives() {
        let g = glob("f/{text/replace,git/git*}.ps1");
        assert!(g.matches(&path("f/text/replace.ps1")));
        assert!(g.matches(&path("f/git/gitinfo.ps1")));
        assert!(!g.matches(&path("f/text/other.ps1")));
    }

    #[test]
    fn double_stars_and_nested_braces_are_refused() {
        assert!(matches!(
            PathPattern::glob("a/**/b.rs"),
            Err(PatternError::DoubleStar(_))
        ));
        assert!(matches!(
            PathPattern::glob("a/{b,{c,d}}.rs"),
            Err(PatternError::Braces(_))
        ));
        assert!(matches!(
            PathPattern::glob("a/{b.rs"),
            Err(PatternError::Braces(_))
        ));
        assert!(PathPattern::glob("../*.rs").is_err());
    }

    #[test]
    fn a_range_bounds_the_file_name() {
        let r = PathPattern::range("s/m/0001", "0007_*.sql").unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(r.as_str(), "s/m/0001–0007_*.sql");
        assert!(r.matches(&path("s/m/0002_initial.sql")));
        assert!(r.matches(&path("s/m/0007_last.sql")));
        assert!(!r.matches(&path("s/m/0008_after.sql")));
        assert!(!r.matches(&path("s/m/0000_before.sql")));
        assert!(!r.matches(&path("s/m/0003_x.txt")));
        assert!(!r.matches(&path("s/Other/0002_initial.sql")));
    }

    #[test]
    fn a_name_shorter_than_the_literal_end_is_outside() {
        let r = PathPattern::range("d/0001", "0007_*.sql").unwrap_or_else(|e| panic!("{e}"));
        assert!(!r.matches(&path("d/0002")));
    }

    #[test]
    fn an_end_without_wildcard_is_inclusive() {
        let r = PathPattern::range("d/a.rs", "c.rs").unwrap_or_else(|e| panic!("{e}"));
        assert!(r.matches(&path("d/b.rs")));
        assert!(r.matches(&path("d/c.rs")));
        assert!(!r.matches(&path("d/c.rsx")));
    }

    #[test]
    fn range_ends_share_a_directory() {
        assert!(PathPattern::range("d/0001", "d/0007").is_ok());
        assert!(PathPattern::range("d/0001", "e/0007").is_err());
        assert!(PathPattern::range("0001", "0007").is_err());
        assert!(PathPattern::range("d/0*", "0007").is_err());
    }

    #[test]
    fn manifest_paths_print_as_written() {
        let file = ManifestPath::File(path("a/b.rs"));
        assert_eq!(file.to_string(), "a/b.rs");
        assert_eq!(format!("{file:?}"), "RepoPath(\"a/b.rs\")");
        assert!(file.covers(&path("a/b.rs")));
        let pattern = ManifestPath::Pattern(glob("a/*.rs"));
        assert!(pattern.covers(&path("a/c.rs")));
        assert_eq!(pattern.file(), None);
    }
}
