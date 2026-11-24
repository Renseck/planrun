//! Format specs: a plan format described in TOML (docs/extending.md §2), and the generic
//! [`Format`](crate::format::Format) built from one.
//!
//! Built-in formats are specs too (`formats/*.toml`, embedded with `include_str!`), so the path a
//! user format takes is the path every built-in format takes (decision D8).

use serde::Deserialize;

/// A format spec as read from TOML. The schema is documented in docs/extending.md §2; keep the
/// two in step.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatSpec {
    /// Unique id.
    pub id: String,
    /// Human name.
    pub name: String,
    /// Detection patterns.
    pub detect: DetectSpec,
    /// Structure.
    pub outline: OutlineSpec,
    /// Shells and manual markers.
    #[serde(default)]
    pub settings: SettingsSpec,
    /// Recognizer instances, in order; each table has `kind` plus the kind's parameters.
    #[serde(default, rename = "recognizer")]
    pub recognizers: Vec<toml::Table>,
}

/// Detection: each matching pattern adds its weight; the score is capped at 100.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetectSpec {
    /// Weighted patterns.
    pub patterns: Vec<WeightedPattern>,
}

/// A pattern and its weight.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeightedPattern {
    /// Regex over one line of the plan (multi-line mode).
    pub regex: String,
    /// Score added when it matches anywhere.
    pub weight: u8,
}

/// Where tasks, manifests and steps are.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutlineSpec {
    /// Tasks are searched only after a heading matching this (`^Tasks$`), when set.
    pub tasks_after: Option<HeadingPattern>,
    /// A task heading; captures `id`, `ord` (number), `title`.
    pub task: HeadingPattern,
    /// The plan's phase: a regex over the first task id, capturing `phase`.
    pub phase_from_task_id: Option<String>,
    /// A heading at or above this level after a task ends the tasks.
    pub tasks_end_level: Option<u8>,
    /// The manifest of a task.
    pub manifest: Option<ManifestSpec>,
    /// Step boundaries; a task without one is a single implicit step.
    #[serde(default)]
    pub steps: Vec<StepPattern>,
}

/// A heading of a given level whose text matches.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeadingPattern {
    /// Heading level.
    pub level: u8,
    /// Regex over the heading text.
    pub regex: String,
}

/// The manifest: a marker block followed by list items.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestSpec {
    /// Regex over the marker paragraph (`^\*\*Files:?\*\*$`).
    pub marker: String,
    /// Regex over each list item; captures `change` and one or more `path`s (see `path_list`).
    pub item: String,
    /// Maps the captured `change` word to Create / Modify / Rewrite / Delete / Test.
    pub changes: std::collections::BTreeMap<String, String>,
    /// When set, the `paths` capture is split on this regex into several paths.
    pub path_list: Option<String>,
}

/// A block that starts a step.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepPattern {
    /// `paragraph`, `list-item` or `heading`.
    pub block: String,
    /// Regex over the block text; may capture `n` and `title`.
    pub regex: String,
}

/// Shells and manual markers.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsSpec {
    /// Paragraph regexes that make text `Manual(Marked)`.
    #[serde(default)]
    pub manual_markers: Vec<String>,
    /// Fence info strings run by pwsh; default `powershell`, `pwsh`, `ps1`.
    pub pwsh: Option<Vec<String>>,
    /// Fence info strings run by bash; default `bash`, `sh`, `shell`.
    pub bash: Option<Vec<String>>,
}
