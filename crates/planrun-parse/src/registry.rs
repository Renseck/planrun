//! The registry: recognizer factories by kind, and the formats built from specs with them.
//!
//! Composition root of parsing. Built-ins register in [`Registry::with_builtins`]; user formats
//! are added from TOML (`.planrun/formats/*.toml` in the repository, then the user config
//! directory) with [`Registry::add_spec`]. A spec that names an unknown recognizer kind, has an
//! invalid regex or duplicates a format id fails here, before any plan is read.

use std::collections::HashMap;
use std::sync::Arc;

use crate::diagnostic::Diagnostic;
use crate::document::Document;
use crate::format::Format;
use crate::kinds;
use crate::recognizer::RecognizerFactory;
use crate::spec::{FormatSpec, SpecFormat};

/// Recognizer factories and formats.
pub struct Registry {
    factories: HashMap<&'static str, Arc<dyn RecognizerFactory>>,
    formats: Vec<Arc<dyn Format>>,
    /// Detection scores below this never select a format.
    pub min_score: u8,
}

/// Why a factory, spec or format was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegistryError {
    /// Two factories with one kind.
    #[error("recognizer kind `{0}` is registered twice")]
    DuplicateKind(String),
    /// Two formats with one id.
    #[error("format `{0}` is registered twice")]
    DuplicateFormat(String),
    /// A spec names a kind no factory provides.
    #[error("format `{format}`: unknown recognizer kind `{kind}`")]
    UnknownKind {
        /// The format.
        format: String,
        /// The kind.
        kind: String,
    },
    /// A spec does not parse or a factory refused its parameters.
    #[error("format `{format}`: {message}")]
    InvalidSpec {
        /// The format (or the file name when the id is unreadable).
        format: String,
        /// What is wrong.
        message: String,
    },
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// An empty registry with the default threshold (50).
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            formats: Vec::new(),
            min_score: 50,
        }
    }

    /// The built-in recognizer kinds and formats (Phase 1: the `assay` format and the kinds it
    /// uses; Phase 2: the hand-written ones).
    pub fn with_builtins() -> Result<Self, RegistryError> {
        let mut registry = Self::new();
        for factory in kinds::builtin_factories() {
            registry.add_factory(factory)?;
        }
        registry.add_spec("assay", include_str!("../../../formats/assay.toml"))?;
        registry.add_spec(
            "superpowers",
            include_str!("../../../formats/superpowers.toml"),
        )?;
        registry.add_spec(
            "before-after",
            include_str!("../../../formats/before-after.toml"),
        )?;
        registry.add_spec("devready", include_str!("../../../formats/devready.toml"))?;
        Ok(registry)
    }

    /// Registers a recognizer factory.
    pub fn add_factory(
        &mut self,
        factory: Arc<dyn RecognizerFactory>,
    ) -> Result<(), RegistryError> {
        let kind = factory.kind();
        if self.factories.insert(kind, factory).is_some() {
            return Err(RegistryError::DuplicateKind(kind.to_owned()));
        }
        Ok(())
    }

    /// Registers a hand-written format.
    pub fn add_format(&mut self, format: Arc<dyn Format>) -> Result<(), RegistryError> {
        if self.formats.iter().any(|f| f.id() == format.id()) {
            return Err(RegistryError::DuplicateFormat(format.id().0.to_string()));
        }
        self.formats.push(format);
        Ok(())
    }

    /// Builds a format from TOML spec text and registers it. `origin` is the spec's file stem,
    /// which must equal its `id` (docs/extending.md §2).
    pub fn add_spec(&mut self, origin: &str, toml_text: &str) -> Result<(), RegistryError> {
        let invalid = |message: String| RegistryError::InvalidSpec {
            format: origin.to_owned(),
            message,
        };
        let spec: FormatSpec = toml::from_str(toml_text).map_err(|e| invalid(e.to_string()))?;
        if spec.id != origin {
            return Err(invalid(format!(
                "the id `{}` is not the file name `{origin}`",
                spec.id
            )));
        }
        let unknown = spec
            .recognizers
            .iter()
            .filter_map(|t| t.get("kind").and_then(|k| k.as_str()))
            .find(|kind| !self.factories.contains_key(kind));
        if let Some(kind) = unknown {
            return Err(RegistryError::UnknownKind {
                format: origin.to_owned(),
                kind: kind.to_owned(),
            });
        }
        let format =
            SpecFormat::compile(spec, |kind| self.factories.get(kind).cloned()).map_err(invalid)?;
        self.add_format(Arc::new(format))
    }

    /// The factory for `kind`.
    pub fn factory(&self, kind: &str) -> Option<&Arc<dyn RecognizerFactory>> {
        self.factories.get(kind)
    }

    /// All formats, in registration order.
    pub fn formats(&self) -> &[Arc<dyn Format>] {
        &self.formats
    }

    /// The format for `doc`: the one named by `forced` (`unknown-format` when none has that
    /// id), else the single highest detection score at or above [`Registry::min_score`]
    /// (`no-format` when none reaches it, `ambiguous-format` on a tie). Never a guess.
    pub fn select(
        &self,
        doc: &Document,
        forced: Option<&str>,
    ) -> Result<Arc<dyn Format>, Diagnostic> {
        if let Some(id) = forced {
            return self
                .formats
                .iter()
                .find(|f| &*f.id().0 == id)
                .cloned()
                .ok_or_else(|| {
                    Diagnostic::error("unknown-format", None, format!("no format has id `{id}`"))
                });
        }
        let mut scored: Vec<(u8, &Arc<dyn Format>)> = self
            .formats
            .iter()
            .map(|f| (f.detect(doc), f))
            .filter(|(score, _)| *score >= self.min_score)
            .collect();
        scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
        match scored.as_slice() {
            [] => Err(Diagnostic::error(
                "no-format",
                None,
                format!(
                    "no format recognizes this plan (no detection score reaches {}); name one \
                     with --format",
                    self.min_score
                ),
            )),
            [(a, first), (b, second), ..] if a == b => Err(Diagnostic::error(
                "ambiguous-format",
                None,
                format!(
                    "formats `{}` and `{}` both score {a}; name one with --format",
                    first.id().0,
                    second.id().0
                ),
            )),
            [(_, first), ..] => Ok(Arc::clone(first)),
        }
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
