//! The block stream: the plan as a flat list of top-level and list-nested blocks, each with its
//! span (docs/architecture.md §4.1). Built once from pulldown-cmark's offset iterator; formats and
//! recognizers never see pulldown-cmark types.

use std::ops::Range;

use planrun_model::{PlanSource, Span, TextRef};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::diagnostic::{Diagnostic, Diagnostics};

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
        /// The item's first paragraph, raw (after the `[ ]` of a task item).
        text: TextRef,
        /// `Some(checked)` for `- [ ]` / `- [x]`.
        task_box: Option<bool>,
    },
    /// A fenced code block. `body` is exact: a full-contents write takes it byte for byte. An
    /// indented code block is a `Fence` with an empty info string and `fence_len` 0.
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
    /// A blockquote, kept whole like a table.
    Quote {
        /// The blockquote's source text, `>` markers included.
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

impl Block {
    /// The language of a fence: the first word of its info string.
    pub fn fence_language<'a>(&'a self, doc: &'a Document) -> Option<&'a str> {
        let Block::Fence { info, .. } = self else {
            return None;
        };
        Some(doc.text(info).split_whitespace().next().unwrap_or(""))
    }
}

/// A block with its position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located {
    /// The block.
    pub block: Block,
    /// Its full source span (for a fence: opening through closing fence; for a list item: its
    /// marker through its own text).
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
    /// Builds the block stream of `source`. An unclosed fence is an error (`unclosed-fence`),
    /// and so is a fence closed by accident (docs/architecture.md §4.1).
    pub fn build(source: PlanSource) -> Result<Self, Diagnostics> {
        let (blocks, errors) = Builder::new(source.text()).run();
        if !errors.is_empty() {
            return Err(Diagnostics(errors));
        }
        Ok(Self { source, blocks })
    }

    /// The text of `text`.
    pub fn text<'a>(&'a self, text: &'a TextRef) -> &'a str {
        self.source.resolve(text)
    }

    /// The lines outside fenced blocks, without their newline (format detection reads only
    /// these).
    pub fn lines_outside_fences(&self) -> Vec<&str> {
        let mut fences = self
            .blocks
            .iter()
            .filter(|b| matches!(b.block, Block::Fence { .. }))
            .map(|b| b.span)
            .peekable();
        let mut lines = Vec::new();
        let mut at = 0usize;
        for line in self.source.text().split_inclusive('\n') {
            let start = at;
            at += line.len();
            while fences.next_if(|f| f.end as usize <= start).is_some() {}
            let inside = fences.peek().is_some_and(|f| f.start as usize <= start);
            if !inside {
                lines.push(line.trim_end_matches('\n'));
            }
        }
        lines
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

impl BlockRange {
    /// True when `index` is in the range.
    pub fn contains(&self, index: usize) -> bool {
        (self.start..self.end).contains(&index)
    }

    /// The number of blocks.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// True for an empty range.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A list item being read: its block index, and the inline text seen before its first nested
/// block.
struct Item {
    block: usize,
    start: usize,
    text: Option<Range<usize>>,
    done: bool,
}

/// A fenced block being read.
struct OpenFence {
    range: Range<usize>,
    pieces: Vec<Range<usize>>,
    owned: String,
    contiguous: bool,
}

/// What an inline event belongs to.
enum Inline {
    Heading {
        level: u8,
        text: Option<Range<usize>>,
    },
    Paragraph,
}

struct Builder<'s> {
    text: &'s str,
    blocks: Vec<Located>,
    errors: Vec<Diagnostic>,
    items: Vec<Item>,
    inline: Option<Inline>,
    fence: Option<OpenFence>,
    /// Events inside this range belong to a block kept whole (table, quote, HTML).
    skip: Option<Range<usize>>,
}

impl<'s> Builder<'s> {
    fn new(text: &'s str) -> Self {
        Self {
            text,
            blocks: Vec::new(),
            errors: Vec::new(),
            items: Vec::new(),
            inline: None,
            fence: None,
            skip: None,
        }
    }

    fn run(mut self) -> (Vec<Located>, Vec<Diagnostic>) {
        let options = Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS;
        let text = self.text;
        for (event, range) in Parser::new_ext(text, options).into_offset_iter() {
            if self.skipped(&range) {
                continue;
            }
            self.event(event, range);
        }
        (self.blocks, self.errors)
    }

    fn skipped(&mut self, range: &Range<usize>) -> bool {
        let Some(skip) = &self.skip else {
            return false;
        };
        if range.start >= skip.start && range.end <= skip.end {
            return true;
        }
        self.skip = None;
        false
    }

    fn event(&mut self, event: Event<'_>, range: Range<usize>) {
        match event {
            Event::Start(tag) => self.start(tag, range),
            Event::End(tag) => self.end(tag, range),
            Event::Text(piece) if self.fence.is_some() => self.fence_text(&piece, range),
            Event::TaskListMarker(checked) => self.task_box(checked),
            Event::Rule => self.push(Block::Rule, range),
            _ => self.inline_event(range),
        }
    }

    fn start(&mut self, tag: Tag<'_>, range: Range<usize>) {
        match tag {
            Tag::Paragraph => self.inline = Some(Inline::Paragraph),
            Tag::Heading { level, .. } => {
                self.close_item_text();
                self.inline = Some(Inline::Heading {
                    level: level as u8,
                    text: None,
                });
            }
            Tag::Item => self.start_item(range),
            Tag::CodeBlock(_) => self.start_fence(range),
            Tag::Table(_) => self.keep_whole(range, |text| Block::Table { text }),
            Tag::BlockQuote(_) => self.keep_whole(range, |text| Block::Quote { text }),
            Tag::HtmlBlock => self.keep_whole(range, |text| Block::Html { text }),
            Tag::List(_) => self.close_item_text(),
            _ => self.inline_event(range),
        }
    }

    fn end(&mut self, tag: TagEnd, range: Range<usize>) {
        match tag {
            TagEnd::Paragraph => self.end_paragraph(range),
            TagEnd::Heading(_) => self.end_heading(range),
            TagEnd::Item => self.end_item(),
            TagEnd::CodeBlock => self.end_fence(),
            _ => {}
        }
    }

    fn keep_whole(&mut self, range: Range<usize>, block: fn(TextRef) -> Block) {
        self.close_item_text();
        let span = self.trimmed(&range);
        self.push(block(TextRef::Span(span)), range.clone());
        self.skip = Some(range);
    }

    fn inline_event(&mut self, range: Range<usize>) {
        if let Some(Inline::Heading { text, .. }) = &mut self.inline {
            *text = Some(widen(text.take(), range));
            return;
        }
        if self.inline.is_some() {
            return;
        }
        let Some(item) = self.items.last_mut() else {
            return;
        };
        if !item.done {
            item.text = Some(widen(item.text.take(), range));
        }
    }

    fn task_box(&mut self, checked: bool) {
        let Some(item) = self.items.last() else {
            return;
        };
        if let Block::ListItem { task_box, .. } = &mut self.blocks[item.block].block {
            *task_box = Some(checked);
        }
    }

    fn end_paragraph(&mut self, range: Range<usize>) {
        self.inline = None;
        let span = self.trimmed(&range);
        if let Some(item) = self.items.last_mut() {
            let first_in_item = !item.done && item.block + 1 == self.blocks.len();
            if first_in_item {
                item.text = Some(span.start as usize..span.end as usize);
                self.close_item_text();
                return;
            }
        }
        self.close_item_text();
        self.push(
            Block::Paragraph {
                text: TextRef::Span(span),
            },
            range,
        );
    }

    fn end_heading(&mut self, range: Range<usize>) {
        let Some(Inline::Heading { level, text }) = self.inline.take() else {
            return;
        };
        let span = self.trimmed(&range);
        let text = text.map_or(Span::new(span.end, span.end), |r| self.trimmed(&r));
        self.push(
            Block::Heading {
                level,
                text: TextRef::Span(text),
            },
            range,
        );
    }

    fn start_item(&mut self, range: Range<usize>) {
        self.close_item_text();
        let at = offset(range.start);
        self.push(
            Block::ListItem {
                text: TextRef::Span(Span::new(at, at)),
                task_box: None,
            },
            range.start..range.start,
        );
        self.items.push(Item {
            block: self.blocks.len() - 1,
            start: range.start,
            text: None,
            done: false,
        });
    }

    fn end_item(&mut self) {
        self.close_item_text();
        self.items.pop();
    }

    /// Fixes the innermost open item's text: called when its first nested block starts or the
    /// item ends.
    fn close_item_text(&mut self) {
        let Some(item) = self.items.last_mut() else {
            return;
        };
        if item.done {
            return;
        }
        item.done = true;
        let text = item.text.clone().unwrap_or(item.start..item.start);
        let span = trim_end(self.text, text);
        let start = offset(item.start);
        let located = &mut self.blocks[item.block];
        located.span = Span::new(start, span.end.max(start));
        if let Block::ListItem { text, .. } = &mut located.block {
            *text = TextRef::Span(span);
        }
    }

    fn start_fence(&mut self, range: Range<usize>) {
        self.close_item_text();
        self.fence = Some(OpenFence {
            range,
            pieces: Vec::new(),
            owned: String::new(),
            contiguous: true,
        });
    }

    fn fence_text(&mut self, piece: &str, range: Range<usize>) {
        let Some(fence) = &mut self.fence else {
            return;
        };
        let borrowed = self.text.get(range.clone()) == Some(piece);
        let follows = fence
            .pieces
            .last()
            .is_none_or(|last| last.end == range.start);
        fence.contiguous &= borrowed && follows;
        fence.owned.push_str(piece);
        fence.pieces.push(range);
    }

    fn end_fence(&mut self) {
        let Some(fence) = self.fence.take() else {
            return;
        };
        let opener = line_at(self.text, fence.range.start);
        let (fence_char, fence_len, info) = fence_opener(self.text, opener.clone());
        let body = match (fence.pieces.first(), fence.pieces.last()) {
            (Some(first), Some(last)) if fence.contiguous => {
                TextRef::Span(Span::new(offset(first.start), offset(last.end)))
            }
            (Some(_), Some(_)) => TextRef::Owned {
                text: fence.owned.as_str().into(),
                from: self.trimmed(&fence.range),
            },
            _ => {
                let at = offset((opener.end + 1).min(fence.range.end));
                TextRef::Span(Span::new(at, at))
            }
        };
        if fence_len > 0 {
            self.check_closed(&fence, opener.clone(), fence_char, fence_len);
        }
        self.push(
            Block::Fence {
                info: TextRef::Span(info),
                body,
                fence_len: u8::try_from(fence_len).unwrap_or(u8::MAX),
            },
            fence.range,
        );
    }

    /// `unclosed-fence`: the fence runs to the end of its container, or a body line opens a
    /// fence of the same character, at least as long, with an info string.
    fn check_closed(
        &mut self,
        fence: &OpenFence,
        opener: Range<usize>,
        fence_char: u8,
        fence_len: usize,
    ) {
        let opener_span = Span::new(offset(opener.start), offset(opener.end));
        let last = last_line(self.text, &fence.range);
        // The closing line keeps its container's indentation in the source: trim all of it.
        let closed = last.start > opener.start
            && closing_fence(self.text[last.clone()].trim_start(), fence_char, fence_len);
        if !closed {
            self.errors.push(Diagnostic::error(
                "unclosed-fence",
                Some(opener_span),
                "this fenced block is never closed: it would swallow the rest of the plan",
            ));
            return;
        }
        let Some(line) = self.suspicious_line(fence, fence_char, fence_len) else {
            return;
        };
        self.errors.push(
            Diagnostic::error(
                "unclosed-fence",
                Some(opener_span),
                "this fenced block is not closed where it looks closed: a line inside it opens \
                 another block",
            )
            .with_related(line, "this line opens a fenced block inside the open one"),
        );
    }

    fn suspicious_line(&self, fence: &OpenFence, fence_char: u8, fence_len: usize) -> Option<Span> {
        for piece in &fence.pieces {
            let piece_text = &self.text[piece.clone()];
            let mut at = piece.start;
            for line in piece_text.split_inclusive('\n') {
                if opens_fence(line, fence_char, fence_len) {
                    let line = line.trim_end_matches('\n');
                    return Some(Span::new(offset(at), offset(at + line.len())));
                }
                at += line.len();
            }
        }
        None
    }

    fn push(&mut self, block: Block, range: Range<usize>) {
        let span = if range.is_empty() {
            Span::new(offset(range.start), offset(range.end))
        } else {
            self.trimmed(&range)
        };
        // A list item is pushed before it is opened, so its depth is its list's.
        let depth = u8::try_from(self.items.len()).unwrap_or(u8::MAX);
        self.blocks.push(Located { block, span, depth });
    }

    fn trimmed(&self, range: &Range<usize>) -> Span {
        trim_end(self.text, range.clone())
    }
}

fn widen(current: Option<Range<usize>>, range: Range<usize>) -> Range<usize> {
    match current {
        Some(r) => r.start.min(range.start)..r.end.max(range.end),
        None => range,
    }
}

fn trim_end(text: &str, range: Range<usize>) -> Span {
    let trimmed = text[range.clone()].trim_end_matches([' ', '\t', '\n']);
    Span::new(offset(range.start), offset(range.start + trimmed.len()))
}

/// The line containing `at`, without its `\n`.
fn line_at(text: &str, at: usize) -> Range<usize> {
    let start = text[..at].rfind('\n').map_or(0, |i| i + 1);
    let end = text[at..].find('\n').map_or(text.len(), |i| at + i);
    start..end
}

/// The last non-empty line of `range`, without its `\n`.
fn last_line(text: &str, range: &Range<usize>) -> Range<usize> {
    let trimmed = text[range.clone()].trim_end_matches('\n');
    let end = range.start + trimmed.len();
    let start = trimmed
        .rfind('\n')
        .map_or(range.start, |i| range.start + i + 1);
    start..end
}

/// The fence character, the fence length and the info string's span of an opening line; a
/// length of 0 for an indented code block.
fn fence_opener(text: &str, line: Range<usize>) -> (u8, usize, Span) {
    let raw = &text[line.clone()];
    let indent = raw.len() - raw.trim_start_matches(' ').len();
    let rest = &raw[indent..];
    let Some(fence_char) = rest.bytes().next().filter(|&c| c == b'`' || c == b'~') else {
        let end = offset(line.end);
        return (0, 0, Span::new(end, end));
    };
    let fence_len = rest.bytes().take_while(|&c| c == fence_char).count();
    let after = &rest[fence_len..];
    let info_start = line.start + indent + fence_len + (after.len() - after.trim_start().len());
    let info_end = line.start + indent + fence_len + after.trim_end().len();
    let info = Span::new(offset(info_start), offset(info_end.max(info_start)));
    (fence_char, fence_len, info)
}

fn fence_run(line: &str, fence_char: u8) -> Option<(usize, &str)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let run = rest.bytes().take_while(|&c| c == fence_char).count();
    Some((run, &rest[run..]))
}

fn closing_fence(line: &str, fence_char: u8, fence_len: usize) -> bool {
    fence_run(line, fence_char)
        .is_some_and(|(run, rest)| run >= fence_len && rest.trim().is_empty())
}

fn opens_fence(line: &str, fence_char: u8, fence_len: usize) -> bool {
    let Some((run, rest)) = fence_run(line, fence_char) else {
        return false;
    };
    let info = rest.trim();
    run >= fence_len && !info.is_empty() && !(fence_char == b'`' && info.contains('`'))
}

fn offset(value: usize) -> u32 {
    // The source module refuses plans over 64 MiB, so every offset fits.
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
#[path = "document_tests.rs"]
mod tests;
