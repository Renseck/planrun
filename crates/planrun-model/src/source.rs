//! The plan source: read once, normalized once, never copied.

use std::sync::Arc;

use crate::span::{LineCol, Span, TextRef};

/// The plan text after normalization (CRLF and lone CR to LF; a leading BOM removed), with a
/// line index and the SHA-256 of the original bytes.
///
/// Normalization happens exactly once, here (decision D6): every span, every fenced block and
/// every line number refers to the normalized text. Line numbers are unchanged by it.
#[derive(Debug, Clone)]
pub struct PlanSource {
    /// Display name (usually the path as given on the command line).
    pub name: Arc<str>,
    text: Arc<str>,
    /// Byte offset of the start of each line.
    line_starts: Arc<[u32]>,
    /// Hex SHA-256 of the bytes as read; identifies the plan in the journal.
    pub fingerprint: Arc<str>,
}

impl PlanSource {
    /// Builds a source from already-normalized text. The parser crate owns reading and
    /// normalizing (it knows the encoding rules); this constructor only indexes.
    pub fn from_normalized(name: &str, text: String, fingerprint: String) -> Self {
        let mut line_starts = vec![0u32];
        for (i, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(offset(i + 1));
            }
        }
        Self {
            name: name.into(),
            text: text.into(),
            line_starts: line_starts.into(),
            fingerprint: fingerprint.into(),
        }
    }

    /// The whole normalized text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The text of `span`. Panics only on a span that did not come from this source.
    pub fn slice(&self, span: Span) -> &str {
        &self.text[span.start as usize..span.end as usize]
    }

    /// The text of `text`, borrowed from the source or from the rebuilt string.
    pub fn resolve<'a>(&'a self, text: &'a TextRef) -> &'a str {
        match text {
            TextRef::Span(span) => self.slice(*span),
            TextRef::Owned { text, .. } => text,
        }
    }

    /// The line and column of a byte offset.
    pub fn line_col(&self, at: u32) -> LineCol {
        let line = self.line_starts.partition_point(|&start| start <= at);
        let start = self.line_starts[line - 1] as usize;
        let col = self.text[start..at as usize].chars().count() + 1;
        LineCol {
            line: offset(line),
            col: offset(col),
        }
    }
}

fn offset(value: usize) -> u32 {
    // Plans are far below 4 GiB; the parser rejects larger input before indexing.
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_col_counts_lines_and_chars() {
        let source = PlanSource::from_normalized("t", "ab\nçd\nef".into(), String::new());
        assert_eq!(source.line_col(0), LineCol { line: 1, col: 1 });
        assert_eq!(source.line_col(3), LineCol { line: 2, col: 1 });
        assert_eq!(source.line_col(5), LineCol { line: 2, col: 2 });
        assert_eq!(source.line_col(7), LineCol { line: 3, col: 1 });
    }
}
