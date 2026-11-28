//! The registry: recognizer factories by kind, and the formats built from specs with them.
//!
//! Composition root of parsing. Built-ins register in [`Registry::with_builtins`]; the formats
//! discovery found (planrun-exec's `discover`, D96) are layered on top with
//! [`Registry::with_layers`]: a user spec replaces a built-in format with its id, a repository
//! spec replaces either (D95). A spec that names an unknown recognizer kind or has an invalid
//! regex fails here, before any plan is read.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use planrun_model::FormatSource;

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
    /// Where each format of `formats` came from, at the same index.
    origins: Vec<Origin>,
    /// Detection scores below this never select a format.
    pub min_score: u8,
}

/// Where a registered format came from, and what it replaced (D95).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    /// Its spec's place.
    pub source: FormatSource,
    /// The format with its id that it replaced, if any.
    pub overrides: Option<FormatSource>,
}

impl Origin {
    /// `id` when it is built in and replaces nothing; else `id (path, overrides …)`, with paths
    /// relative to `root` when they are under it.
    pub fn describe(&self, id: &str, root: Option<&Path>) -> String {
        let mut parts = Vec::new();
        if let Some(path) = place(&self.source, root) {
            parts.push(path);
        }
        if let Some(old) = &self.overrides {
            let what = place(old, root).unwrap_or_else(|| "built-in".to_owned());
            parts.push(format!("overrides {what}"));
        }
        if parts.is_empty() {
            return id.to_owned();
        }
        format!("{id} ({})", parts.join(", "))
    }
}

/// A spec file's path for display, or `None` for a built-in format.
fn place(source: &FormatSource, root: Option<&Path>) -> Option<String> {
    let path = match source {
        FormatSource::BuiltIn => return None,
        FormatSource::User(path) | FormatSource::Repository(path) => path,
    };
    let relative = root.and_then(|r| path.strip_prefix(r).ok());
    Some(match relative {
        Some(rel) => rel.to_string_lossy().replace('\\', "/"),
        None => path.display().to_string(),
    })
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
    /// A discovered spec file was refused.
    #[error("{path}: {error}")]
    InFile {
        /// The spec file.
        path: PathBuf,
        /// Why.
        error: Box<RegistryError>,
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
            origins: Vec::new(),
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

    /// The built-ins with discovered specs layered on top, in order (D95): each replaces the
    /// format with its id that is already registered, whole. Discovery gives the user layer
    /// before the repository layer, so the repository wins. A refused spec is an error that
    /// names its file.
    pub fn with_layers<'a>(
        specs: impl IntoIterator<Item = (&'a str, &'a str, &'a FormatSource)>,
    ) -> Result<Self, RegistryError> {
        let mut registry = Self::with_builtins()?;
        for (origin, text, source) in specs {
            registry
                .add_spec_from(origin, text, source.clone())
                .map_err(|e| match source {
                    FormatSource::BuiltIn => e,
                    FormatSource::User(path) | FormatSource::Repository(path) => {
                        RegistryError::InFile {
                            path: path.clone(),
                            error: Box::new(e),
                        }
                    }
                })?;
        }
        Ok(registry)
    }

    /// Builds a format from a spec found at `source` and registers it; a registered format with
    /// its id is replaced in place, and its source becomes the new one's `overrides`.
    pub fn add_spec_from(
        &mut self,
        origin: &str,
        toml_text: &str,
        source: FormatSource,
    ) -> Result<(), RegistryError> {
        let format: Arc<dyn Format> = Arc::new(self.compile_spec(origin, toml_text)?);
        let at = self.formats.iter().position(|f| f.id() == format.id());
        let Some(at) = at else {
            self.formats.push(format);
            self.origins.push(Origin {
                source,
                overrides: None,
            });
            return Ok(());
        };
        let old = std::mem::replace(&mut self.origins[at].source, source);
        self.origins[at].overrides = Some(old);
        self.formats[at] = format;
        Ok(())
    }

    /// Where the format `id` came from.
    pub fn origin(&self, id: &str) -> Option<&Origin> {
        let at = self.formats.iter().position(|f| &*f.id().0 == id)?;
        self.origins.get(at)
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

    /// Registers a hand-written format, as a built-in one.
    pub fn add_format(&mut self, format: Arc<dyn Format>) -> Result<(), RegistryError> {
        if self.formats.iter().any(|f| f.id() == format.id()) {
            return Err(RegistryError::DuplicateFormat(format.id().0.to_string()));
        }
        self.formats.push(format);
        self.origins.push(Origin {
            source: FormatSource::BuiltIn,
            overrides: None,
        });
        Ok(())
    }

    /// Builds a built-in format from TOML spec text and registers it. `origin` is the spec's
    /// file stem, which must equal its `id` (docs/extending.md §2); a second format with one id
    /// is an error.
    pub fn add_spec(&mut self, origin: &str, toml_text: &str) -> Result<(), RegistryError> {
        let format = self.compile_spec(origin, toml_text)?;
        self.add_format(Arc::new(format))
    }

    /// Compiles a spec with the registered recognizer kinds.
    fn compile_spec(&self, origin: &str, toml_text: &str) -> Result<SpecFormat, RegistryError> {
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
        SpecFormat::compile(spec, |kind| self.factories.get(kind).cloned()).map_err(invalid)
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
