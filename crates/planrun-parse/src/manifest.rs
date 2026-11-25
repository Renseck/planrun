//! The manifest of a task (docs/architecture.md §4.3): the Files marker, its list items or its
//! inline clauses, and the entries they declare (decisions D27, D32, D41, D43–D45).

use planrun_model::{
    ChangeKind, LineRange, Manifest, ManifestEntry, ManifestPath, PathOrigin, PathPattern,
    RepoPath, Span, TextRef,
};
use regex::Regex;

use crate::diagnostic::Diagnostic;
use crate::document::{Block, BlockRange, Document};

/// A manifest: a marker paragraph and the list items after it, or the clauses inside it.
pub struct ManifestRules {
    /// Over the marker paragraph; may capture `change` for items that are bare paths, and
    /// `items` for a manifest written inline (D43).
    pub marker: Regex,
    /// Over each item or clause; captures `paths`, and `change` unless the marker does.
    pub item: Regex,
    /// The captured change word and what it means.
    pub changes: Vec<(String, ChangeKind)>,
    /// Splits `paths` into several, when set.
    pub path_list: Option<Regex>,
    /// Splits the marker's `items` capture into clauses, when set (D43).
    pub clause_list: Option<Regex>,
    /// Over the text right after a path token; its `change` capture changes that one path
    /// (D44).
    pub path_change: Option<Regex>,
    /// When false, a task may have no manifest (decision D27).
    pub required: bool,
    /// A bare file name after a path with a directory is in that directory (decision D32).
    pub siblings: bool,
}

/// What the outline found for a task's manifest.
#[derive(Default)]
pub struct Found {
    /// The entries.
    pub manifest: Manifest,
    /// The marker and its list items: consumed, in no step.
    pub blocks: Option<BlockRange>,
    /// Text of an inline marker that declares nothing: shown in place as a Note (D43).
    pub leftovers: Vec<TextRef>,
}

/// The manifest: the first block before any step boundary that matches the marker, and the list
/// items right after it, up to the first step boundary (a list-item step right after the items
/// is in the same CommonMark list, decision D37). A marker whose `items` capture took part holds
/// the manifest itself (D43).
pub fn manifest(
    spec: &ManifestRules,
    doc: &Document,
    head: usize,
    body: BlockRange,
    boundary: impl Fn(usize) -> bool,
    errors: &mut Vec<Diagnostic>,
) -> Found {
    let marker = (body.start..body.end)
        .take_while(|&i| !boundary(i))
        .find(|&i| matches!(&doc.blocks[i].block, Block::Paragraph { text } if spec.marker.is_match(doc.text(text))));
    let Some(marker) = marker else {
        if spec.required {
            errors.push(Diagnostic::error(
                "manifest-missing",
                Some(doc.blocks[head].span),
                format!("the task has no manifest (`{}`)", spec.marker.as_str()),
            ));
        }
        return Found::default();
    };
    let Block::Paragraph { text } = &doc.blocks[marker].block else {
        return Found::default();
    };
    let marker_text = doc.text(text);
    let base = text.span().start;
    let caps = spec.marker.captures(marker_text);
    let marker_change = caps
        .as_ref()
        .and_then(|c| c.name("change"))
        .map(|m| m.as_str());
    let depth = doc.blocks[marker].depth;
    let items_end = (marker + 1..body.end)
        .find(|&i| {
            let b = &doc.blocks[i];
            !matches!(b.block, Block::ListItem { .. }) || b.depth != depth || boundary(i)
        })
        .unwrap_or(body.end);
    let mut entries = Vec::new();
    let mut leftovers = Vec::new();
    let inline = caps.as_ref().and_then(|c| c.name("items"));
    let end = match inline {
        Some(items) => {
            if items_end > marker + 1 {
                errors.push(Diagnostic::error(
                    "manifest-item",
                    Some(doc.blocks[marker + 1].span),
                    "a list after a Files line that already names its files",
                ));
            }
            let at = Inline {
                text: marker_text,
                base,
                items,
                marker_change,
            };
            inline_items(spec, &at, &mut entries, &mut leftovers, errors);
            marker + 1
        }
        None => {
            if items_end == marker + 1 {
                errors.push(Diagnostic::error(
                    "manifest-item",
                    Some(doc.blocks[marker].span),
                    "the manifest has no items",
                ));
            }
            for i in marker + 1..items_end {
                list_item(spec, doc, i, marker_change, &mut entries, errors);
            }
            items_end
        }
    };
    let range = BlockRange { start: marker, end };
    let last = doc.blocks[end - 1].span.end;
    Found {
        manifest: Manifest {
            entries,
            span: Some(Span::new(doc.blocks[marker].span.start, last)),
        },
        blocks: Some(range),
        leftovers,
    }
}

/// The marker paragraph of an inline manifest and its `items` capture.
struct Inline<'a> {
    text: &'a str,
    base: u32,
    items: regex::Match<'a>,
    marker_change: Option<&'a str>,
}

/// The clauses of an inline manifest (D43). A clause with a code span is an item; one without
/// declares nothing and is a leftover, as is any text after the `items` capture.
fn inline_items(
    spec: &ManifestRules,
    at: &Inline<'_>,
    entries: &mut Vec<ManifestEntry>,
    leftovers: &mut Vec<TextRef>,
    errors: &mut Vec<Diagnostic>,
) {
    let items = at.items.as_str();
    let mut clauses: Vec<(usize, &str)> = Vec::new();
    let mut from = 0;
    if let Some(split) = &spec.clause_list {
        for sep in split.find_iter(items) {
            clauses.push((from, &items[from..sep.start()]));
            from = sep.end();
        }
    }
    clauses.push((from, &items[from..]));
    let start = at.items.start();
    for (offset_in_items, clause) in clauses {
        let trimmed = clause.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lead = clause.len() - clause.trim_start().len();
        let at_start = at.base + offset(start + offset_in_items + lead);
        let span = Span::new(at_start, at_start + offset(trimmed.len()));
        if !trimmed.contains('`') {
            leftovers.push(TextRef::Span(span));
            continue;
        }
        let item = Item {
            text: trimmed,
            base: at_start,
            span,
            marker_change: at.marker_change,
        };
        item_entries(spec, &item, entries, errors);
    }
    let tail = &at.text[at.items.end()..];
    let rest = tail.trim_start_matches(|c: char| c == '.' || c.is_whitespace());
    if !rest.trim().is_empty() {
        let rest_at = at.base + offset(at.text.len() - rest.len());
        let rest = rest.trim_end();
        leftovers.push(TextRef::Span(Span::new(
            rest_at,
            rest_at + offset(rest.len()),
        )));
    }
}

/// One list item of a manifest.
fn list_item(
    spec: &ManifestRules,
    doc: &Document,
    index: usize,
    marker_change: Option<&str>,
    entries: &mut Vec<ManifestEntry>,
    errors: &mut Vec<Diagnostic>,
) {
    let located = &doc.blocks[index];
    let Block::ListItem { text, .. } = &located.block else {
        return;
    };
    let item = Item {
        text: doc.text(text),
        base: text.span().start,
        span: located.span,
        marker_change,
    };
    item_entries(spec, &item, entries, errors);
}

/// The text of one item or clause.
struct Item<'a> {
    text: &'a str,
    base: u32,
    span: Span,
    marker_change: Option<&'a str>,
}

/// The entries of one item or clause: its change (captured by the item, or else by the marker)
/// and its paths.
fn item_entries(
    spec: &ManifestRules,
    item: &Item<'_>,
    entries: &mut Vec<ManifestEntry>,
    errors: &mut Vec<Diagnostic>,
) {
    let parsed = spec.item.captures(item.text).and_then(|caps| {
        let word = caps
            .name("change")
            .map(|m| m.as_str())
            .or(item.marker_change)?;
        Some((change_named(spec, word)?, caps.name("paths")?))
    });
    let Some((change, paths)) = parsed else {
        errors.push(Diagnostic::error(
            "manifest-item",
            Some(item.span),
            format!(
                "a manifest item that does not match `{}`",
                spec.item.as_str()
            ),
        ));
        return;
    };
    let paths_span = Span::new(
        item.base + offset(paths.start()),
        item.base + offset(paths.end()),
    );
    let tail = &item.text[paths.end()..];
    let mut previous: Option<(RepoPath, PathOrigin)> = None;
    let mut after_dir = false;
    for piece in pieces(spec, paths.as_str(), tail) {
        let context = Context {
            previous: previous.as_ref(),
            after_dir,
        };
        let (path, origin, lines, own) = match token(spec, piece, &context) {
            Ok(token) => token,
            Err((code, message)) => {
                errors.push(Diagnostic::error(code, Some(paths_span), message));
                continue;
            }
        };
        after_dir |= path.to_string().contains('/');
        previous = path.file().map(|file| (file.clone(), origin));
        entries.push(ManifestEntry {
            path,
            change: own.unwrap_or(change),
            lines,
            span: item.span,
            origin,
        });
    }
}

fn change_named(spec: &ManifestRules, word: &str) -> Option<ChangeKind> {
    spec.changes
        .iter()
        .find(|(w, _)| w == word)
        .map(|(_, k)| *k)
}

/// A piece of an item's paths: one token, or a range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Piece<'a> {
    /// A path, a pattern or a `:lines` token, and the text right after it up to the next token
    /// (a separator, or the rest of the item).
    Token(&'a str, &'a str),
    /// `` `a`–`b` ``: an en dash between two code spans (decision D41).
    Range(&'a str, &'a str),
}

/// The pieces of `paths` (followed in the item by `tail`). Ranges are found first, in every
/// format, so a spec's `path_list` never sees their en dash; the text around them is split by
/// `path_list` (D41).
fn pieces<'a>(spec: &ManifestRules, paths: &'a str, tail: &'a str) -> Vec<Piece<'a>> {
    const DASH: &str = "`–`";
    let mut out = Vec::new();
    let mut rest = paths;
    let mut after_range = false;
    while let Some(at) = rest.find(DASH) {
        let (before, after) = (&rest[..at], &rest[at + DASH.len()..]);
        let start_at = before.rfind('`').map_or(0, |i| i + 1);
        let end_len = after.find('`').unwrap_or(after.len());
        out.extend(split(spec, &before[..start_at], "", (after_range, true)));
        out.push(Piece::Range(&before[start_at..], &after[..end_len]));
        rest = &after[end_len..];
        after_range = true;
    }
    out.extend(split(spec, rest, tail, (after_range, false)));
    out
}

/// `text` split by `path_list`, each token with the separator after it (the last with `tail`).
/// Next to a range, the empty piece on its side is what is left of the separator, not an empty
/// path: `after_range` drops a leading one, `before_range` a trailing one.
fn split<'a>(
    spec: &ManifestRules,
    text: &'a str,
    tail: &'a str,
    (after_range, before_range): (bool, bool),
) -> Vec<Piece<'a>> {
    if (after_range || before_range) && text.trim().is_empty() {
        return Vec::new();
    }
    let mut parts: Vec<(&str, &str)> = Vec::new();
    let mut from = 0;
    if let Some(split) = &spec.path_list {
        for sep in split.find_iter(text) {
            parts.push((&text[from..sep.start()], sep.as_str()));
            from = sep.end();
        }
    }
    parts.push((&text[from..], tail));
    if before_range && parts.last().is_some_and(|(p, _)| p.trim().is_empty()) {
        parts.pop();
    }
    if after_range && parts.first().is_some_and(|(p, _)| p.trim().is_empty()) {
        parts.remove(0);
    }
    parts
        .into_iter()
        .map(|(p, after)| Piece::Token(p.trim(), after))
        .collect()
}

/// A diagnostic code and message.
type Refusal = (&'static str, String);

/// What a token knows of the tokens before it in its item.
struct Context<'a> {
    /// The file before it.
    previous: Option<&'a (RepoPath, PathOrigin)>,
    /// Some earlier file token had a directory.
    after_dir: bool,
}

/// One entry: the path, its origin, its lines, and its own change (D44).
type Token = (
    ManifestPath,
    PathOrigin,
    Option<LineRange>,
    Option<ChangeKind>,
);

/// One piece of a manifest item (decisions D32, D41, D44, D45): `:3-10` is more lines of the
/// previous path; with `siblings`, a bare file name after a path with a directory is in that
/// directory, without them it is `Bare`; a token with `*`, `?` or `{`, and a range, are
/// patterns; `path_change` over the text after a token gives that path its own change.
fn token(spec: &ManifestRules, piece: Piece<'_>, cx: &Context<'_>) -> Result<Token, Refusal> {
    let (piece, after) = match piece {
        Piece::Range(start, end) => {
            let pattern =
                PathPattern::range(start, end).map_err(|e| ("manifest-item", e.to_string()))?;
            return Ok((
                ManifestPath::Pattern(pattern),
                PathOrigin::Written,
                None,
                None,
            ));
        }
        Piece::Token(piece, after) => (piece, after),
    };
    let own = own_change(spec, after)?;
    if let Some(range) = piece.strip_prefix(':') {
        let Some((path, origin)) = cx.previous else {
            return Err(("bad-path", format!("`{piece}` is not a path")));
        };
        let lines = line_range(range);
        return Ok((ManifestPath::File(path.clone()), *origin, lines, own));
    }
    if PathPattern::is_glob(piece) {
        let pattern = PathPattern::glob(piece).map_err(|e| ("manifest-item", e.to_string()))?;
        return Ok((
            ManifestPath::Pattern(pattern),
            PathOrigin::Written,
            None,
            own,
        ));
    }
    let (path, lines) = split_lines(piece);
    let dir = cx.previous.and_then(|(p, _)| p.dir());
    let bare = !path.contains('/');
    let (spelled, origin) = match dir {
        Some(dir) if spec.siblings && bare => {
            (RepoPath::new(&format!("{dir}/{path}")), PathOrigin::Sibling)
        }
        _ if !spec.siblings && bare && cx.after_dir => (RepoPath::new(path), PathOrigin::Bare),
        _ => (RepoPath::new(path), PathOrigin::Written),
    };
    let path = spelled.map_err(|e| ("bad-path", e.to_string()))?;
    Ok((ManifestPath::File(path), origin, lines, own))
}

/// The change `path_change` gives the token followed by `after`, mapped through `changes`.
fn own_change(spec: &ManifestRules, after: &str) -> Result<Option<ChangeKind>, Refusal> {
    let Some(word) = spec
        .path_change
        .as_ref()
        .and_then(|r| r.captures(after))
        .and_then(|c| c.name("change"))
    else {
        return Ok(None);
    };
    match change_named(spec, word.as_str()) {
        Some(kind) => Ok(Some(kind)),
        None => Err((
            "manifest-item",
            format!("`{}` is no change this format knows", word.as_str()),
        )),
    }
}

/// `src/a.rs:3-10` → (`src/a.rs`, 3–10); `src/a.rs:7` → (`src/a.rs`, 7–7).
fn split_lines(path: &str) -> (&str, Option<LineRange>) {
    let Some((file, range)) = path.rsplit_once(':') else {
        return (path, None);
    };
    match line_range(range) {
        Some(lines) => (file, Some(lines)),
        None => (path, None),
    }
}

/// `3-10` or `7`.
fn line_range(range: &str) -> Option<LineRange> {
    let (start, end) = range.split_once('-').unwrap_or((range, range));
    Some(LineRange {
        start: start.parse().ok()?,
        end: end.parse().ok()?,
    })
}

fn offset(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_ranges_are_split_from_paths() {
        assert_eq!(
            split_lines("src/a.rs:3-10"),
            ("src/a.rs", Some(LineRange { start: 3, end: 10 }))
        );
        assert_eq!(
            split_lines("src/a.rs:7"),
            ("src/a.rs", Some(LineRange { start: 7, end: 7 }))
        );
        assert_eq!(split_lines("src/a.rs"), ("src/a.rs", None));
        assert_eq!(split_lines("a:b"), ("a:b", None));
    }
}
