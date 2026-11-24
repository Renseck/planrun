//! Plan, task, step and the task's file manifest.

use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::ids::{FormatId, TaskId};
use crate::path::RepoPath;
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

/// One declared file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// The file.
    pub path: RepoPath,
    /// What the task does to it.
    pub change: ChangeKind,
    /// A line range written after the path (`CleanupStep.cs:3-10`); informational.
    pub lines: Option<LineRange>,
    /// The entry's text.
    pub span: Span,
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
}

/// `start-end`, 1-based inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineRange {
    /// First line.
    pub start: u32,
    /// Last line.
    pub end: u32,
}
