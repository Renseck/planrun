//! A plan format: how to find tasks, manifests and steps, and which recognizers turn a step's
//! blocks into actions (docs/architecture.md §5).

use std::sync::Arc;

use planrun_model::{FormatId, Manifest, PlanMeta, Shell, Span, TaskId, TextRef};

use crate::diagnostic::{Diagnostic, Diagnostics, Severity};
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
    /// The blocks after the tasks (from a heading at or above `tasks_end_level`): never run.
    pub after_tasks: Option<BlockRange>,
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
    /// The manifest's blocks (marker and items): consumed by the outline, in no step. Recognition
    /// skips them where they fall inside a step's range.
    pub manifest_blocks: Option<BlockRange>,
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
    /// True when the step starts at a written boundary; false for the implicit first step.
    pub written: bool,
    /// The mark its title carries (`[[step_marks]]`, decision D30).
    pub mark: Option<StepMark>,
}

/// What a step's title says about the whole step (decision D30).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMark {
    /// "…verify it fails": the step's `Verify` scripts are expected to fail.
    ExpectedFailure,
    /// "Check by eye", "Manual verification": the step's Notes are `Manual(HumanCheck)`.
    HumanCheck,
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
    /// Severity overrides for the codes in [`DOWNGRADABLE`] (decision D31).
    pub checks: Vec<(&'static str, CheckLevel)>,
}

/// The only check codes a format may downgrade: manifest coverage, never a safety check
/// (decision D31).
pub const DOWNGRADABLE: [&str; 2] = ["manifest-untouched", "manifest-undeclared"];

/// The severity a format gives a downgradable check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckLevel {
    /// As built in.
    Error,
    /// Reported, does not stop the plan.
    Warning,
    /// Not reported.
    Off,
}

impl FormatSettings {
    /// The shell that runs a fence in `language`, if any.
    pub fn shell_for(&self, language: &str) -> Option<Shell> {
        if self.pwsh_languages.iter().any(|l| l == language) {
            return Some(Shell::Pwsh);
        }
        if self.bash_languages.iter().any(|l| l == language) {
            return Some(Shell::Bash);
        }
        None
    }

    /// `diagnostics` with this format's check levels applied.
    pub fn apply_checks(&self, diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
        let level = |code: &str| {
            self.checks
                .iter()
                .find(|(c, _)| *c == code)
                .map(|(_, level)| *level)
        };
        diagnostics
            .into_iter()
            .filter_map(|mut d| match level(d.code) {
                Some(CheckLevel::Off) => None,
                Some(CheckLevel::Warning) => {
                    d.severity = Severity::Warning;
                    Some(d)
                }
                _ => Some(d),
            })
            .collect()
    }

    /// True when `text` matches one of the manual markers.
    pub fn is_marked(&self, text: &str) -> bool {
        self.manual_markers.iter().any(|m| m.is_match(text))
    }
}
