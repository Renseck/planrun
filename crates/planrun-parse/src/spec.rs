//! Format specs: a plan format described in TOML (docs/extending.md §2), and the generic
//! [`Format`](crate::format::Format) built from one.
//!
//! Built-in formats are specs too (`formats/*.toml`, embedded with `include_str!`), so the path a
//! user format takes is the path every built-in format takes (decision D8).

use std::sync::Arc;

use planrun_model::{ChangeKind, FormatId};
use regex::Regex;
use serde::Deserialize;

use crate::diagnostic::Diagnostics;
use crate::document::Document;
use crate::format::{Format, FormatSettings, Outline};
use crate::outline::{self, HeadingRule, ManifestRules, OutlineRules, StepBlock, StepRule};
use crate::recognizer::{Recognizer, RecognizerFactory};

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
    /// Regex over each line of the plan outside fenced blocks.
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
    /// A task heading; captures `id`, `ord` (number), `title`, and optionally `phase` (the
    /// plan's phase, taken from the first task).
    pub task: HeadingPattern,
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
    /// Regex over each list item; captures `change` and `paths` (split by `path_list` when set).
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

/// Compiles `pattern`, refusing it when it lacks one of the `required` captures. `what` names it
/// in the message (`outline.task`, `lead`).
pub fn compile_regex(what: &str, pattern: &str, required: &[&str]) -> Result<Regex, String> {
    let regex = Regex::new(pattern).map_err(|e| format!("{what}: {e}"))?;
    let names: Vec<&str> = regex.capture_names().flatten().collect();
    if let Some(missing) = required.iter().find(|r| !names.contains(r)) {
        return Err(format!("{what}: the regex has no `{missing}` capture"));
    }
    Ok(regex)
}

/// The generic format: a compiled [`FormatSpec`].
pub struct SpecFormat {
    id: FormatId,
    name: String,
    detect: Vec<(Regex, u8)>,
    outline: OutlineRules,
    settings: FormatSettings,
    recognizers: Vec<Arc<dyn Recognizer>>,
}

impl SpecFormat {
    /// Compiles `spec`: every regex, every parameter, every recognizer through `factory` (the
    /// registry's lookup by kind). Errors name the offending key.
    pub fn compile(
        spec: FormatSpec,
        factory: impl Fn(&str) -> Option<Arc<dyn RecognizerFactory>>,
    ) -> Result<Self, String> {
        let detect = spec
            .detect
            .patterns
            .iter()
            .map(|p| Ok((compile_regex("detect", &p.regex, &[])?, p.weight)))
            .collect::<Result<_, String>>()?;
        let outline = compile_outline(&spec.outline)?;
        let settings = compile_settings(&spec.settings)?;
        let recognizers = spec
            .recognizers
            .iter()
            .map(|table| build_recognizer(table, &factory))
            .collect::<Result<_, String>>()?;
        Ok(Self {
            id: FormatId(spec.id.into()),
            name: spec.name,
            detect,
            outline,
            settings,
            recognizers,
        })
    }

    /// The human name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Format for SpecFormat {
    fn id(&self) -> &FormatId {
        &self.id
    }

    fn detect(&self, doc: &Document) -> u8 {
        let lines = doc.lines_outside_fences();
        let score: u32 = self
            .detect
            .iter()
            .filter(|(regex, _)| lines.iter().any(|line| regex.is_match(line)))
            .map(|(_, weight)| u32::from(*weight))
            .sum();
        u8::try_from(score.min(100)).unwrap_or(100)
    }

    fn outline(&self, doc: &Document) -> Result<Outline, Diagnostics> {
        outline::outline(&self.outline, &self.settings, doc)
    }

    fn recognizers(&self) -> &[Arc<dyn Recognizer>] {
        &self.recognizers
    }

    fn settings(&self) -> &FormatSettings {
        &self.settings
    }
}

fn compile_outline(spec: &OutlineSpec) -> Result<OutlineRules, String> {
    let tasks_after = spec
        .tasks_after
        .as_ref()
        .map(|h| heading_rule("outline.tasks_after", h, &[]))
        .transpose()?;
    let task = heading_rule("outline.task", &spec.task, &["id", "ord", "title"])?;
    let manifest = spec.manifest.as_ref().map(compile_manifest).transpose()?;
    let steps = spec
        .steps
        .iter()
        .map(|s| {
            Ok(StepRule {
                block: step_block(&s.block)?,
                regex: compile_regex("outline.steps", &s.regex, &[])?,
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(OutlineRules {
        tasks_after,
        task,
        tasks_end_level: spec.tasks_end_level,
        manifest,
        steps,
    })
}

fn heading_rule(what: &str, h: &HeadingPattern, required: &[&str]) -> Result<HeadingRule, String> {
    if !(1..=6).contains(&h.level) {
        return Err(format!("{what}: level {} is not 1–6", h.level));
    }
    Ok(HeadingRule {
        level: h.level,
        regex: compile_regex(what, &h.regex, required)?,
    })
}

fn compile_manifest(spec: &ManifestSpec) -> Result<ManifestRules, String> {
    let changes = spec
        .changes
        .iter()
        .map(|(word, kind)| Ok((word.clone(), change_kind(kind)?)))
        .collect::<Result<_, String>>()?;
    Ok(ManifestRules {
        marker: compile_regex("outline.manifest.marker", &spec.marker, &[])?,
        item: compile_regex("outline.manifest.item", &spec.item, &["change", "paths"])?,
        changes,
        path_list: spec
            .path_list
            .as_deref()
            .map(|p| compile_regex("outline.manifest.path_list", p, &[]))
            .transpose()?,
    })
}

fn change_kind(name: &str) -> Result<ChangeKind, String> {
    match name {
        "create" => Ok(ChangeKind::Create),
        "modify" => Ok(ChangeKind::Modify),
        "rewrite" => Ok(ChangeKind::Rewrite),
        "delete" => Ok(ChangeKind::Delete),
        "test" => Ok(ChangeKind::Test),
        other => Err(format!(
            "outline.manifest.changes: `{other}` is not create, modify, rewrite, delete or test"
        )),
    }
}

fn step_block(name: &str) -> Result<StepBlock, String> {
    match name {
        "paragraph" => Ok(StepBlock::Paragraph),
        "list-item" => Ok(StepBlock::ListItem),
        "heading" => Ok(StepBlock::Heading),
        other => Err(format!(
            "outline.steps: block `{other}` is not paragraph, list-item or heading"
        )),
    }
}

fn compile_settings(spec: &SettingsSpec) -> Result<FormatSettings, String> {
    let strings = |list: &[&str]| list.iter().map(|s| (*s).to_owned()).collect();
    Ok(FormatSettings {
        manual_markers: spec
            .manual_markers
            .iter()
            .map(|m| compile_regex("settings.manual_markers", m, &[]))
            .collect::<Result<_, String>>()?,
        pwsh_languages: spec
            .pwsh
            .clone()
            .unwrap_or_else(|| strings(&["powershell", "pwsh", "ps1"])),
        bash_languages: spec
            .bash
            .clone()
            .unwrap_or_else(|| strings(&["bash", "sh", "shell"])),
    })
}

fn build_recognizer(
    table: &toml::Table,
    factory: &impl Fn(&str) -> Option<Arc<dyn RecognizerFactory>>,
) -> Result<Arc<dyn Recognizer>, String> {
    let Some(kind) = table.get("kind").and_then(|k| k.as_str()) else {
        return Err("a [[recognizer]] without a string `kind`".to_owned());
    };
    let Some(factory) = factory(kind) else {
        return Err(format!("unknown recognizer kind `{kind}`"));
    };
    let mut params = table.clone();
    params.remove("kind");
    factory
        .build(&params)
        .map_err(|e| format!("recognizer `{kind}`: {e}"))
}
