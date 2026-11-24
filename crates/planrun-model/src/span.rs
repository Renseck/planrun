//! Byte spans into the normalized plan text, and text that refers to them.

use serde::{Deserialize, Serialize};

/// A half-open byte range `[start, end)` into [`crate::PlanSource::text`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Inclusive start offset.
    pub start: u32,
    /// Exclusive end offset.
    pub end: u32,
}

impl Span {
    /// A span from `start` to `end`.
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// The length in bytes.
    pub const fn len(self) -> u32 {
        self.end - self.start
    }

    /// True for an empty span.
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// The smallest span covering both.
    pub fn join(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }
}

/// A 1-based line and column (columns count Unicode scalar values), for messages only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LineCol {
    /// 1-based line.
    pub line: u32,
    /// 1-based column.
    pub col: u32,
}

/// Text of the plan.
///
/// Almost always a [`Span`] into the source, so a 4,000-line plan is held once. `Owned` exists
/// for the rare text that is not contiguous in the source, e.g. a fenced block inside a list item
/// whose indentation CommonMark strips (docs/plan-grammar.md §1.3).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextRef {
    /// Contiguous text of the source.
    Span(Span),
    /// Text rebuilt by the parser, with the span it came from.
    Owned {
        /// The rebuilt text.
        text: Box<str>,
        /// Where it came from, for messages.
        from: Span,
    },
}

impl TextRef {
    /// The span this text came from.
    pub fn span(&self) -> Span {
        match self {
            TextRef::Span(span) => *span,
            TextRef::Owned { from, .. } => *from,
        }
    }
}
