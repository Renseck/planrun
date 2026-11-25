//! Plan, task, step and the task's file manifest.

use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::ids::{FormatId, TaskId};
use crate::path::{PathOrigin, RepoPath};
use crate::pattern::ManifestPath;
use crate::source::PlanSource;
use crate::span::{Span, TextRef};

/// A parsed, checked plan. Owns its source; everything else refers into it.
#[derive(Debug, Clone)]
pub struct Plan {
    /// The normalized plan text.
    pub source: PlanSource,
    /// The format that parsed it.
    pub format: FormatId,
    /// Plan-level facts.
    pub meta: PlanMeta,
    /// Tasks in document order.
    pub tasks: Vec<Task>,
}

impl Plan {
    /// The text of `text`.
    pub fn text<'a>(&'a self, text: &'a TextRef) -> &'a str {
        self.source.resolve(text)
    }
}

/// Plan-level facts. Phases are metadata, not a level of the tree (decision D5).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanMeta {
    /// The first level-1 heading.
    pub title: Option<TextRef>,
    /// The phase id when the format captures one (`P1`, `P3c-1`).
    pub phase: Option<Box<str>>,
    /// The base commit the plan says to start from, when stated.
    pub base_commit: Option<Box<str>>,
    /// Paragraphs before the first task that match the format's `manual_markers`: text the user
    /// must read before task 1.
    pub preamble: Vec<TextRef>,
    /// The text after the tasks (`## Part 1 green state`, …): shown, never run.
    pub after_tasks: Option<Span>,
}

/// One task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// Its id.
    pub id: TaskId,
    /// Its title.
    pub title: TextRef,
    /// Heading through the last block before the next task.
    pub span: Span,
    /// The declared files. Empty when the format has none for this task.
    pub manifest: Manifest,
    /// Steps in order. A format without step headings yields one implicit step.
    pub steps: Vec<Step>,
}

/// One step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    /// Its number, when written; must count up from 1 within the task.
    pub number: Option<u32>,
    /// Its title, when written.
    pub title: Option<TextRef>,
    /// Heading through the last block of the step.
    pub span: Span,
    /// Actions in order.
    pub actions: Vec<Action>,
}

/// The files a task declares: its write set (check rule `manifest`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Entries in order.
    pub entries: Vec<ManifestEntry>,
    /// The manifest block, when present.
    pub span: Option<Span>,
}

impl Manifest {
    /// Whether a file action at `path` is in the task's declared scope (decision D50): a file
    /// entry equal to it whose origin is not `Bare`, or a pattern entry that matches it. Always
    /// false for a task without a manifest.
    pub fn scopes(&self, path: &RepoPath) -> bool {
        self.entries.iter().any(|e| match &e.path {
            ManifestPath::File(file) => file == path && e.origin != PathOrigin::Bare,
            ManifestPath::Pattern(pattern) => pattern.matches(path),
        })
    }

    /// Whether some `Bare` file entry names `path` (decisions D45, D50): the plan wrote a file
    /// name whose directory it never said, so `path` may not be the file it means.
    pub fn is_bare_target(&self, path: &RepoPath) -> bool {
        self.entries
            .iter()
            .any(|e| e.origin == PathOrigin::Bare && e.path.file() == Some(path))
    }
}

/// One declared file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// The file, or the files a pattern names (decision D41).
    pub path: ManifestPath,
    /// What the task does to it.
    pub change: ChangeKind,
    /// A line range written after the path (`CleanupStep.cs:3-10`); informational.
    pub lines: Option<LineRange>,
    /// The entry's text.
    pub span: Span,
    /// `Sibling` when the item named only a file name after a path with a directory.
    pub origin: PathOrigin,
}

/// What a task declares it does to a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeKind {
    /// New file.
    Create,
    /// Existing file, changed.
    Modify,
    /// Existing file, replaced whole.
    Rewrite,
    /// Removed.
    Delete,
    /// A test file (created or modified; the actions decide).
    Test,
    /// Produced by a command the task runs; no file action needed (decision D27).
    Generated,
}

/// `start-end`, 1-based inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineRange {
    /// First line.
    pub start: u32,
    /// Last line.
    pub end: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::PathPattern;

    fn path(p: &str) -> RepoPath {
        RepoPath::new(p).unwrap_or_else(|e| panic!("{e}"))
    }

    fn entry(path: ManifestPath, origin: PathOrigin) -> ManifestEntry {
        ManifestEntry {
            path,
            change: ChangeKind::Modify,
            lines: None,
            span: Span::new(0, 0),
            origin,
        }
    }

    fn manifest(entries: Vec<ManifestEntry>) -> Manifest {
        Manifest {
            entries,
            span: Some(Span::new(0, 0)),
        }
    }

    #[test]
    fn written_and_sibling_entries_and_patterns_scope_a_path() {
        let glob = PathPattern::glob("db/*.sql").unwrap_or_else(|e| panic!("{e}"));
        let m = manifest(vec![
            entry(ManifestPath::File(path("src/a.rs")), PathOrigin::Written),
            entry(ManifestPath::File(path("src/b.rs")), PathOrigin::Sibling),
            entry(ManifestPath::Pattern(glob), PathOrigin::Written),
        ]);
        for scoped in ["src/a.rs", "src/b.rs", "db/1.sql"] {
            assert!(m.scopes(&path(scoped)), "{scoped}");
        }
        for unscoped in ["src/c.rs", "db/x/1.sql", "a.rs"] {
            assert!(!m.scopes(&path(unscoped)), "{unscoped}");
        }
        assert!(!m.is_bare_target(&path("src/a.rs")));
    }

    #[test]
    fn a_bare_entry_targets_but_never_scopes() {
        let m = manifest(vec![
            entry(ManifestPath::File(path("src/a.rs")), PathOrigin::Written),
            entry(ManifestPath::File(path("b.ps1")), PathOrigin::Bare),
        ]);
        assert!(!m.scopes(&path("b.ps1")));
        assert!(m.is_bare_target(&path("b.ps1")));
        assert!(!m.is_bare_target(&path("src/b.ps1")));
    }

    #[test]
    fn a_path_with_a_written_and_a_bare_entry_is_both() {
        let m = manifest(vec![
            entry(ManifestPath::File(path("b.ps1")), PathOrigin::Written),
            entry(ManifestPath::File(path("b.ps1")), PathOrigin::Bare),
        ]);
        assert!(m.scopes(&path("b.ps1")));
        assert!(m.is_bare_target(&path("b.ps1")));
    }

    #[test]
    fn no_manifest_scopes_nothing() {
        let m = Manifest::default();
        assert!(!m.scopes(&path("a.rs")));
        assert!(!m.is_bare_target(&path("a.rs")));
    }
}
