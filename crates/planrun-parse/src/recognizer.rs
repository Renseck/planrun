//! Recognizers: small, composable matchers that turn a run of blocks into actions
//! (docs/architecture.md §4.4). A format is an ordered list of recognizer instances.

use std::sync::Arc;

use planrun_model::{Action, Manifest, TaskId};

use crate::diagnostic::Diagnostic;
use crate::document::{BlockRange, Document, Located};
use crate::format::FormatSettings;

/// What a recognizer may read besides the blocks.
pub struct StepContext<'a> {
    /// The document.
    pub doc: &'a Document,
    /// The task being recognized.
    pub task: &'a TaskId,
    /// Its manifest (a full-contents write consults it for create vs overwrite).
    pub manifest: &'a Manifest,
    /// Format settings.
    pub settings: &'a FormatSettings,
    /// Actions already produced in this step: an expectation attaches to the script before it;
    /// the patch ceremony is absorbed only right after a patch.
    pub previous: &'a [Action],
}

/// A position in a step's blocks. A recognizer that matches advances it past what it consumed.
/// Blocks in the `skip` range (the task's manifest) are invisible to it.
pub struct Cursor<'a> {
    blocks: &'a [Located],
    range: BlockRange,
    skip: Option<BlockRange>,
    position: usize,
    consumed: usize,
}

impl<'a> Cursor<'a> {
    /// A cursor at the start of `range`.
    pub fn new(doc: &'a Document, range: BlockRange) -> Self {
        Self::with_skip(doc, range, None)
    }

    /// A cursor at the start of `range` that never sees the blocks of `skip`.
    pub fn with_skip(doc: &'a Document, range: BlockRange, skip: Option<BlockRange>) -> Self {
        let mut cursor = Self {
            blocks: &doc.blocks,
            range,
            skip,
            position: range.start,
            consumed: 0,
        };
        cursor.position = cursor.settle(range.start);
        cursor
    }

    /// `index`, or the first index after the skipped range when it falls inside it.
    fn settle(&self, index: usize) -> usize {
        match self.skip {
            Some(skip) if skip.contains(index) => skip.end,
            _ => index,
        }
    }

    /// The block `offset` positions ahead, if it is still in the step.
    pub fn peek(&self, offset: usize) -> Option<&'a Located> {
        let mut index = self.position;
        for _ in 0..offset {
            index = self.settle(index + 1);
        }
        (index < self.range.end).then(|| &self.blocks[index])
    }

    /// Consumes `count` blocks.
    pub fn advance(&mut self, count: usize) {
        for _ in 0..count {
            if self.is_done() {
                return;
            }
            self.position = self.settle(self.position + 1);
            self.consumed += 1;
        }
    }

    /// True when every block of the step is consumed.
    pub fn is_done(&self) -> bool {
        self.position >= self.range.end
    }

    /// The current position (for coverage and diagnostics).
    pub fn position(&self) -> usize {
        self.position
    }

    /// How many blocks have been consumed so far.
    pub fn consumed(&self) -> usize {
        self.consumed
    }
}

/// The outcome of one recognizer at one position.
pub enum Recognized {
    /// Not this recognizer's idiom; the cursor must be unchanged.
    No,
    /// Matched: the cursor is advanced; these actions are appended (may be empty, e.g. the
    /// absorbed patch ceremony).
    Yes(Vec<Action>),
    /// The idiom matched but is malformed (lead sentence without its fence, a patch whose
    /// paths disagree with the lead): a parse error, never a fallback to `Manual`.
    Malformed(Diagnostic),
    /// Matched an earlier action instead of producing one (an `Expected` block): replace the
    /// last action with this one.
    Amend(Action),
}

/// One configured matcher. Must be pure: no I/O, no state between calls.
pub trait Recognizer: Send + Sync {
    /// Instance id for coverage and `planrun show` (`file-block#full-contents`).
    fn id(&self) -> &str;

    /// Tries to match at the cursor.
    fn recognize(&self, cx: &StepContext<'_>, cursor: &mut Cursor<'_>) -> Recognized;
}

/// Builds recognizer instances of one kind from the `[[recognizer]]` tables of a format spec.
/// Registered by kind name in the [`crate::Registry`]; this is the code-level plugin point.
pub trait RecognizerFactory: Send + Sync {
    /// The kind name used in specs (`file-block`).
    fn kind(&self) -> &'static str;

    /// One instance from its spec table (`params` excludes `kind`). Invalid params are an
    /// error when the registry loads the format, never at parse time.
    fn build(&self, params: &toml::Table) -> Result<Arc<dyn Recognizer>, String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source;

    #[test]
    fn the_cursor_never_sees_the_skipped_blocks() {
        let text = "a\n\nb\n\nc\n\nd\n";
        let source = source::load("t", text.as_bytes()).unwrap_or_else(|e| panic!("{e:?}"));
        let doc = Document::build(source).unwrap_or_else(|e| panic!("{e:?}"));
        let all = BlockRange { start: 0, end: 4 };
        let skip = Some(BlockRange { start: 1, end: 3 });
        let mut cursor = Cursor::with_skip(&doc, all, skip);
        let at = |c: &Cursor<'_>, k| c.peek(k).map(|b| doc.source.slice(b.span).to_owned());
        assert_eq!(at(&cursor, 0).as_deref(), Some("a"));
        assert_eq!(at(&cursor, 1).as_deref(), Some("d"));
        assert_eq!(at(&cursor, 2), None);
        cursor.advance(1);
        assert_eq!(cursor.position(), 3);
        cursor.advance(5);
        assert!(cursor.is_done());
        assert_eq!(cursor.consumed(), 2);

        let from_skip = Cursor::with_skip(&doc, BlockRange { start: 1, end: 4 }, skip);
        assert_eq!(from_skip.position(), 3);
    }
}
