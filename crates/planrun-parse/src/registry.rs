//! The registry: recognizer factories by kind, and the formats built from specs with them.
//!
//! Composition root of parsing. Built-ins register in [`Registry::with_builtins`]; user formats
//! are added from TOML (`.planrun/formats/*.toml` in the repository, then the user config
//! directory) with [`Registry::add_spec`]. A spec that names an unknown recognizer kind, has an
//! invalid regex or duplicates a format id fails here, before any plan is read.

use std::collections::HashMap;
use std::sync::Arc;

use crate::format::Format;
use crate::recognizer::RecognizerFactory;

/// Recognizer factories and formats.
#[derive(Default)]
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

impl Registry {
    /// An empty registry.
    pub fn new() -> Self {
        Self {
            min_score: 50,
            ..Self::default()
        }
    }

    /// The built-in recognizer kinds and formats (Phase 1: the `assay` format and the kinds it
    /// uses; Phase 2: the other three).
    pub fn with_builtins() -> Result<Self, RegistryError> {
        Ok(Self::new())
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

    /// Builds a format from TOML spec text and registers it (Phase 1).
    pub fn add_spec(&mut self, origin: &str, toml_text: &str) -> Result<(), RegistryError> {
        let _ = toml_text;
        Err(RegistryError::InvalidSpec {
            format: origin.to_owned(),
            message: "spec loading is implemented in Phase 1".to_owned(),
        })
    }

    /// The factory for `kind`.
    pub fn factory(&self, kind: &str) -> Option<&Arc<dyn RecognizerFactory>> {
        self.factories.get(kind)
    }

    /// All formats, in registration order.
    pub fn formats(&self) -> &[Arc<dyn Format>] {
        &self.formats
    }
}
