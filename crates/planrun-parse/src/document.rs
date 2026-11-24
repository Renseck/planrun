//! The block stream: the plan as a flat list of top-level and list-nested blocks, each with its
//! span (docs/architecture.md §4.1). Built once from pulldown-cmark's offset iterator; formats and
//! recognizers never see pulldown-cmark types.

use planrun_model::{PlanSource, Span, TextRef};

/// One block of the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// `#`…`######`.
    Heading {
        /// 1–6.
        level: u8,
        /// The heading text, without the `#`s.
        text: TextRef,
    },
    /// A paragraph, as raw source text (inline markup kept: patterns match the raw text).
    Paragraph {
        /// The paragraph text.
        text: TextRef,
    },
    /// A list item's own text (nested blocks follow as separate blocks with `depth + 1`).
    ListItem {
        /// The item's first paragraph, raw.
        text: TextRef,
        /// `Some(checked)` for `- [ ]` / `- [x]`.
        task_box: Option<bool>,
    },
    /// A fenced code block. `body` is exact: a full-contents write takes it byte for byte.
    Fence {
        /// The info string (`rust`, `powershell`, `diff`, ``) as written.
        info: TextRef,
        /// The content between the fences, ending in `\n` unless empty.
        body: TextRef,
        /// The opening fence's length (3 for ```` ``` ````, 4 for ```` ```` ````).
        fence_len: u8,
    },
    /// A table, kept whole (the "File map" table is shown, never parsed into actions).
    Table {
        /// The table's source text.
        text: TextRef,
    },
    /// `---`.
    Rule,
    /// Raw HTML, including `<!-- comments -->`.
    Html {
        /// The HTML source.
        text: TextRef,
    },
}

/// A block with its position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located {
    /// The block.
    pub block: Block,
    /// Its full source span (for a fence: opening through closing fence).
    pub span: Span,
    /// List nesting depth: 0 at top level.
    pub depth: u8,
}

/// The plan source with its block stream.
#[derive(Debug, Clone)]
pub struct Document {
    /// The normalized plan.
    pub source: PlanSource,
    /// All blocks in source order.
    pub blocks: Vec<Located>,
}

impl Document {
    /// The text of `text`.
    pub fn text<'a>(&'a self, text: &'a TextRef) -> &'a str {
        self.source.resolve(text)
    }
}

/// An index range `[start, end)` into [`Document::blocks`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRange {
    /// First block.
    pub start: usize,
    /// One past the last block.
    pub end: usize,
}
