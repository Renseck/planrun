//! Parse and check diagnostics: a stable code, a severity, a span, a message.

use planrun_model::Span;

/// How bad a diagnostic is. Any `Error` stops the plan from being returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// The plan cannot run.
    Error,
    /// The plan can run; something looks wrong (e.g. low coverage).
    Warning,
    /// Information (coverage summary lines).
    Note,
}

/// One finding. `code` is stable and kebab-case (`duplicate-task-id`); the invalid corpus pins
/// codes, never messages (tests/corpus/invalid/README.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Severity.
    pub severity: Severity,
    /// Stable code.
    pub code: &'static str,
    /// Where, when known.
    pub span: Option<Span>,
    /// What, for a human.
    pub message: String,
    /// Further spans with labels ("first defined here").
    pub related: Vec<(Span, String)>,
}

impl Diagnostic {
    /// An error.
    pub fn error(code: &'static str, span: Option<Span>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code,
            span,
            message: message.into(),
            related: Vec::new(),
        }
    }

    /// A warning.
    pub fn warning(code: &'static str, span: Option<Span>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            ..Self::error(code, span, message)
        }
    }

    /// A note (information, never blocks).
    pub fn note(code: &'static str, span: Option<Span>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Note,
            ..Self::error(code, span, message)
        }
    }

    /// Adds a related span.
    pub fn with_related(mut self, span: Span, label: impl Into<String>) -> Self {
        self.related.push((span, label.into()));
        self
    }
}

/// All diagnostics of one parse, in source order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnostics(pub Vec<Diagnostic>);

impl Diagnostics {
    /// One diagnostic.
    pub fn single(diagnostic: Diagnostic) -> Self {
        Self(vec![diagnostic])
    }

    /// True when any diagnostic is an error.
    pub fn has_errors(&self) -> bool {
        self.0.iter().any(|d| d.severity == Severity::Error)
    }

    /// The codes of the errors, in order.
    pub fn error_codes(&self) -> Vec<&'static str> {
        self.0
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| d.code)
            .collect()
    }

    /// Sorts into source order (diagnostics without a span first); stable.
    pub fn sort(&mut self) {
        self.0.sort_by_key(|d| d.span.map(|s| s.start));
    }
}
