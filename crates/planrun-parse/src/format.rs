//! A plan format: how to find tasks, manifests and steps, and which recognizers turn a step's
//! blocks into actions (docs/architecture.md §5).

use std::sync::Arc;

use planrun_model::{FormatId, Manifest, PlanMeta, Span, TaskId, TextRef};

use crate::diagnostic::Diagnostics;
use crate::document::{BlockRange, Document};
use crate::recognizer::Recognizer;

/// A plan format. Most formats are a [`crate::spec::SpecFormat`] built from a TOML spec; a
/// format whose structure is not heading-shaped implements this trait by hand.
pub trait Format: Send + Sync {
    /// Its id (`assay`).
    fn id(&self) -> &FormatId;

    /// How well `doc` matches, 0–100. Cheap: patterns over the lines outside fenced blocks.
    fn detect(&self, doc: &Document) -> u8;

    /// The plan's structure: metadata, tasks, their manifests and step ranges. Every block of
    /// every task must fall in exactly one step range (the recognizers then account for each).
    fn outline(&self, doc: &Document) -> Result<Outline, Diagnostics>;

    /// The recognizers, in the order they are tried at each position of a step.
    fn recognizers(&self) -> &[Arc<dyn Recognizer>];

    /// Format-level settings the recognizers and checks read.
    fn settings(&self) -> &FormatSettings;
}

/// The structure of a plan before recognition.
#[derive(Debug, Clone)]
pub struct Outline {
    /// Plan metadata.
    pub meta: PlanMeta,
    /// Tasks in order.
    pub tasks: Vec<OutlineTask>,
}

/// A task before recognition.
#[derive(Debug, Clone)]
pub struct OutlineTask {
    /// Its id.
    pub id: TaskId,
    /// Its title.
    pub title: TextRef,
    /// Its span.
    pub span: Span,
    /// The parsed manifest.
    pub manifest: Manifest,
    /// Steps: number, title, blocks.
    pub steps: Vec<OutlineStep>,
}

/// A step before recognition.
#[derive(Debug, Clone)]
pub struct OutlineStep {
    /// Its number, when written.
    pub number: Option<u32>,
    /// Its title, when written.
    pub title: Option<TextRef>,
    /// Its span.
    pub span: Span,
    /// The blocks the recognizers consume.
    pub blocks: BlockRange,
}

/// Settings shared by a format's recognizers and the checks.
#[derive(Debug, Clone, Default)]
pub struct FormatSettings {
    /// Paragraphs matching any of these become `Manual(Marked)` instead of `Note`.
    pub manual_markers: Vec<regex::Regex>,
    /// Fence info strings that run in pwsh / bash. Anything else is `Manual(ForeignLanguage)`.
    pub pwsh_languages: Vec<String>,
    /// See `pwsh_languages`.
    pub bash_languages: Vec<String>,
}
