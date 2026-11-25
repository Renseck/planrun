//! What preflight, a simulation or the dry run found: a stable code, a severity, the plan span
//! and the path. The CLI renders findings like parse diagnostics; tests pin codes.

use planrun_model::{RepoPath, Span};

use crate::workspace::WorkspaceError;

/// How bad a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// The plan cannot run as it stands.
    Error,
    /// It can; something needs a look.
    Warning,
}

/// One finding. `code` is stable and kebab-case (`blob-mismatch`, `create-exists`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Severity.
    pub severity: Severity,
    /// Stable code.
    pub code: &'static str,
    /// The plan text it is about, when it is about an action or an entry.
    pub span: Option<Span>,
    /// The repository path it is about.
    pub path: Option<RepoPath>,
    /// What, for a human.
    pub message: String,
}

impl Finding {
    /// An error.
    pub fn error(
        code: &'static str,
        span: Option<Span>,
        path: Option<&RepoPath>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            code,
            span,
            path: path.cloned(),
            message: message.into(),
        }
    }

    /// A warning.
    pub fn warning(
        code: &'static str,
        span: Option<Span>,
        path: Option<&RepoPath>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            ..Self::error(code, span, path, message)
        }
    }

    /// An error made from a workspace or git failure.
    pub fn workspace(e: &WorkspaceError, span: Option<Span>, path: Option<&RepoPath>) -> Self {
        Self::error(e.code(), span, path, e.to_string())
    }

    /// True for an error.
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}
