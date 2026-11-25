//! The generic outline of a heading-shaped plan (docs/architecture.md §4.3): plan metadata, the
//! tasks, each task's manifest and its steps. [`crate::spec::SpecFormat`] uses it; a hand-written
//! format may too.

use planrun_model::{
    ChangeKind, LineRange, Manifest, ManifestEntry, ManifestPath, PathOrigin, PathPattern,
    PlanMeta, RepoPath, Span, TaskId, TextRef,
};
use regex::{Captures, Regex};

use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::document::{Block, BlockRange, Document};
use crate::format::{FormatSettings, Outline, OutlineStep, OutlineTask, StepMark};

/// The compiled `[outline]` of a spec.
pub struct OutlineRules {
    /// Tasks start after this heading, when set.
    pub tasks_after: Option<HeadingRule>,
    /// A task heading; captures `id`, `ord`, `title`, optionally `phase`.
    pub task: HeadingRule,
    /// A heading at or above this level after a task ends the tasks.
    pub tasks_end_level: Option<u8>,
    /// The manifest of a task, when the format has one.
    pub manifest: Option<ManifestRules>,
    /// Step boundaries.
    pub steps: Vec<StepRule>,
    /// Step titles that mark the whole step, first match wins (decision D30).
    pub step_marks: Vec<(Regex, StepMark)>,
}

/// A heading of one level whose text matches.
pub struct HeadingRule {
    /// 1–6.
    pub level: u8,
    /// Over the heading text.
    pub regex: Regex,
}

/// A manifest: a marker paragraph and the list items after it.
pub struct ManifestRules {
    /// Over the marker paragraph; may capture `change` for items that are bare paths.
    pub marker: Regex,
    /// Over each item; captures `paths`, and `change` unless the marker does.
    pub item: Regex,
    /// The captured change word and what it means.
    pub changes: Vec<(String, ChangeKind)>,
    /// Splits `paths` into several, when set.
    pub path_list: Option<Regex>,
    /// When false, a task may have no manifest (decision D27).
    pub required: bool,
    /// A bare file name after a path with a directory is in that directory (decision D32).
    pub siblings: bool,
}

/// A block that starts a step.
pub struct StepRule {
    /// Which kind of block.
    pub block: StepBlock,
    /// Over its text; may capture `n` and `title`.
    pub regex: Regex,
}

/// The kinds of block a step can start with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepBlock {
    /// A paragraph (`**Step 1 — …**`).
    Paragraph,
    /// A list item (`- [ ] **Step 1: …**`).
    ListItem,
    /// A heading.
    Heading,
}

/// The outline of `doc` under `rules`. Every error is collected; any error means no outline.
pub fn outline(
    rules: &OutlineRules,
    settings: &FormatSettings,
    doc: &Document,
) -> Result<Outline, Diagnostics> {
    let mut errors = Vec::new();
    let start = tasks_start(rules, doc).map_err(Diagnostics::single)?;
    let (heads, end) = task_heads(rules, doc, start, &mut errors);
    let Some(&first) = heads.first() else {
        errors.push(Diagnostic::error(
            "no-tasks",
            None,
            "no heading matches the format's task pattern",
        ));
        return Err(Diagnostics(errors));
    };
    let mut tasks = Vec::with_capacity(heads.len());
    let mut phase = None;
    for (k, &head) in heads.iter().enumerate() {
        let next = heads.get(k + 1).copied().unwrap_or(end);
        let (task, task_phase) = task(rules, doc, head, next, &mut errors);
        phase = phase.or(task_phase);
        tasks.push(task);
    }
    if !errors.is_empty() {
        return Err(Diagnostics(errors));
    }
    let after_tasks = (end < doc.blocks.len()).then_some(BlockRange {
        start: end,
        end: doc.blocks.len(),
    });
    let meta = PlanMeta {
        title: title(doc, first),
        phase,
        base_commit: None,
        preamble: marked(
            doc,
            settings,
            BlockRange {
                start: 0,
                end: first,
            },
        ),
        after_tasks: after_tasks.map(|r| {
            Span::new(
                doc.blocks[r.start].span.start,
                offset(doc.source.text().len()),
            )
        }),
    };
    Ok(Outline {
        meta,
        tasks,
        after_tasks,
    })
}

/// The paragraphs, list items and quotes of `range` that match a manual marker.
pub fn marked(doc: &Document, settings: &FormatSettings, range: BlockRange) -> Vec<TextRef> {
    doc.blocks[range.start..range.end]
        .iter()
        .filter_map(|b| match &b.block {
            Block::Paragraph { text } | Block::ListItem { text, .. } | Block::Quote { text } => {
                Some(text)
            }
            _ => None,
        })
        .filter(|text| settings.is_marked(doc.text(text)))
        .cloned()
        .collect()
}

fn tasks_start(rules: &OutlineRules, doc: &Document) -> Result<usize, Diagnostic> {
    let Some(rule) = &rules.tasks_after else {
        return Ok(0);
    };
    doc.blocks
        .iter()
        .position(|b| heading_matches(doc, &b.block, rule))
        .map(|i| i + 1)
        .ok_or_else(|| {
            Diagnostic::error(
                "no-tasks",
                None,
                format!(
                    "no level-{} heading matches `{}`, so there are no tasks",
                    rule.level,
                    rule.regex.as_str()
                ),
            )
        })
}

/// The task headings from `start`, and the index where the tasks end. A heading that matches the
/// task rule is a task; only a heading that is no task, at or above `tasks_end_level`, ends the
/// tasks (decision D37).
fn task_heads(
    rules: &OutlineRules,
    doc: &Document,
    start: usize,
    errors: &mut Vec<Diagnostic>,
) -> (Vec<usize>, usize) {
    let mut heads = Vec::new();
    for (i, located) in doc.blocks.iter().enumerate().skip(start) {
        let Block::Heading { level, .. } = &located.block else {
            continue;
        };
        if heading_matches(doc, &located.block, &rules.task) {
            heads.push(i);
            continue;
        }
        let in_tasks = !heads.is_empty() || rules.tasks_after.is_some();
        if !heads.is_empty() && rules.tasks_end_level.is_some_and(|l| *level <= l) {
            late_tasks(rules, doc, i, errors);
            return (heads, i);
        }
        if in_tasks && *level == rules.task.level {
            errors.push(Diagnostic::error(
                "bad-task-heading",
                Some(located.span),
                format!(
                    "a level-{level} heading among the tasks that is not a task: it does not match `{}`",
                    rules.task.regex.as_str()
                ),
            ));
        }
    }
    (heads, doc.blocks.len())
}

/// Task headings after the heading at `end`, which ended the tasks: they would be text that never
/// runs, so each is an error (`task-after-end`).
fn late_tasks(rules: &OutlineRules, doc: &Document, end: usize, errors: &mut Vec<Diagnostic>) {
    let ended = doc.blocks[end].span;
    let late = doc.blocks[end + 1..]
        .iter()
        .filter(|b| heading_matches(doc, &b.block, &rules.task));
    for located in late {
        errors.push(
            Diagnostic::error(
                "task-after-end",
                Some(located.span),
                "a task heading after the tasks ended: planrun would never run it",
            )
            .with_related(ended, "the tasks end here"),
        );
    }
}

fn heading_matches(doc: &Document, block: &Block, rule: &HeadingRule) -> bool {
    let Block::Heading { level, text } = block else {
        return false;
    };
    *level == rule.level && rule.regex.is_match(doc.text(text))
}

fn title(doc: &Document, before: usize) -> Option<TextRef> {
    doc.blocks[..before].iter().find_map(|b| match &b.block {
        Block::Heading { level: 1, text } => Some(text.clone()),
        _ => None,
    })
}

/// One task: the heading at `head`, its blocks up to `next`. Returns its phase capture.
fn task(
    rules: &OutlineRules,
    doc: &Document,
    head: usize,
    next: usize,
    errors: &mut Vec<Diagnostic>,
) -> (OutlineTask, Option<Box<str>>) {
    let located = &doc.blocks[head];
    let heading = match &located.block {
        Block::Heading { text, .. } => text.clone(),
        _ => TextRef::Span(located.span),
    };
    let text = doc.text(&heading);
    let base = heading.span().start;
    let caps = rules.task.regex.captures(text);
    let capture = |name: &str| capture_ref(caps.as_ref(), name, base);
    let raw = caps
        .as_ref()
        .and_then(|c| c.name("id"))
        .map_or(text, |m| m.as_str());
    let ordinal = caps
        .as_ref()
        .and_then(|c| c.name("ord"))
        .and_then(|m| m.as_str().parse::<u32>().ok());
    if ordinal.is_none() {
        errors.push(Diagnostic::error(
            "bad-task-id",
            Some(located.span),
            "the task's `ord` capture is not a number",
        ));
    }
    let phase = caps
        .as_ref()
        .and_then(|c| c.name("phase"))
        .map(|m| m.as_str().into());
    let body = BlockRange {
        start: head + 1,
        end: next,
    };
    let (manifest, manifest_blocks) = manifest(rules, doc, head, body, errors);
    let steps = steps(rules, doc, body, manifest_blocks, errors);
    let span = Span::new(
        located.span.start,
        range_end(doc, body).unwrap_or(located.span.end),
    );
    let task = OutlineTask {
        id: TaskId {
            raw: raw.into(),
            ordinal: ordinal.unwrap_or(0),
        },
        title: capture("title").unwrap_or(heading),
        span,
        manifest,
        manifest_blocks,
        steps,
    };
    (task, phase)
}

fn capture_ref(caps: Option<&Captures<'_>>, name: &str, base: u32) -> Option<TextRef> {
    let m = caps?.name(name)?;
    Some(TextRef::Span(Span::new(
        base + offset(m.start()),
        base + offset(m.end()),
    )))
}

/// The manifest: the first block before any step boundary that matches the marker, and the list
/// items right after it, up to the first step boundary (a list-item step right after the items
/// is in the same CommonMark list, decision D37).
fn manifest(
    rules: &OutlineRules,
    doc: &Document,
    head: usize,
    body: BlockRange,
    errors: &mut Vec<Diagnostic>,
) -> (Manifest, Option<BlockRange>) {
    let Some(spec) = &rules.manifest else {
        return (Manifest::default(), None);
    };
    let marker = (body.start..body.end)
        .take_while(|&i| boundary(rules, doc, i).is_none())
        .find(|&i| matches!(&doc.blocks[i].block, Block::Paragraph { text } if spec.marker.is_match(doc.text(text))));
    let Some(marker) = marker else {
        if spec.required {
            errors.push(Diagnostic::error(
                "manifest-missing",
                Some(doc.blocks[head].span),
                format!("the task has no manifest (`{}`)", spec.marker.as_str()),
            ));
        }
        return (Manifest::default(), None);
    };
    let marker_change = match &doc.blocks[marker].block {
        Block::Paragraph { text } => spec
            .marker
            .captures(doc.text(text))
            .and_then(|c| c.name("change"))
            .map(|m| m.as_str()),
        _ => None,
    };
    let depth = doc.blocks[marker].depth;
    let items_end = (marker + 1..body.end)
        .find(|&i| {
            let b = &doc.blocks[i];
            !matches!(b.block, Block::ListItem { .. })
                || b.depth != depth
                || boundary(rules, doc, i).is_some()
        })
        .unwrap_or(body.end);
    if items_end == marker + 1 {
        errors.push(Diagnostic::error(
            "manifest-item",
            Some(doc.blocks[marker].span),
            "the manifest has no items",
        ));
    }
    let mut entries = Vec::new();
    for i in marker + 1..items_end {
        manifest_item(spec, doc, i, marker_change, &mut entries, errors);
    }
    let range = BlockRange {
        start: marker,
        end: items_end,
    };
    let span = Span::new(
        doc.blocks[marker].span.start,
        range_end(doc, range).unwrap_or(doc.blocks[marker].span.end),
    );
    let manifest = Manifest {
        entries,
        span: Some(span),
    };
    (manifest, Some(range))
}

/// One manifest item: its change (captured by the item, or else by the marker) and its paths.
fn manifest_item(
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
    let base = text.span().start;
    let item = doc.text(text);
    let parsed = spec.item.captures(item).and_then(|caps| {
        let change = caps.name("change").map(|m| m.as_str()).or(marker_change)?;
        let kind = spec.changes.iter().find(|(w, _)| w == change)?.1;
        Some((kind, caps.name("paths")?))
    });
    let Some((change, paths)) = parsed else {
        errors.push(Diagnostic::error(
            "manifest-item",
            Some(located.span),
            format!(
                "a manifest item that does not match `{}`",
                spec.item.as_str()
            ),
        ));
        return;
    };
    let paths_span = Span::new(base + offset(paths.start()), base + offset(paths.end()));
    let mut previous: Option<(RepoPath, PathOrigin)> = None;
    for piece in pieces(spec, paths.as_str()) {
        let (path, origin, lines) = match token(spec, piece, previous.as_ref()) {
            Ok(token) => token,
            Err((code, message)) => {
                errors.push(Diagnostic::error(code, Some(paths_span), message));
                continue;
            }
        };
        previous = path.file().map(|file| (file.clone(), origin));
        entries.push(ManifestEntry {
            path,
            change,
            lines,
            span: located.span,
            origin,
        });
    }
}

/// A piece of an item's paths: one token, or a range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Piece<'a> {
    /// A path, a pattern or a `:lines` token.
    Token(&'a str),
    /// `` `a`–`b` ``: an en dash between two code spans (decision D41).
    Range(&'a str, &'a str),
}

/// The pieces of `paths`. Ranges are found first, in every format, so a spec's `path_list` never
/// sees their en dash; the text around them is split by `path_list` (D41).
fn pieces<'a>(spec: &ManifestRules, paths: &'a str) -> Vec<Piece<'a>> {
    const DASH: &str = "`–`";
    let mut out = Vec::new();
    let mut rest = paths;
    let mut after_range = false;
    while let Some(at) = rest.find(DASH) {
        let (before, after) = (&rest[..at], &rest[at + DASH.len()..]);
        let start_at = before.rfind('`').map_or(0, |i| i + 1);
        let end_len = after.find('`').unwrap_or(after.len());
        out.extend(split(spec, &before[..start_at], after_range, true));
        out.push(Piece::Range(&before[start_at..], &after[..end_len]));
        rest = &after[end_len..];
        after_range = true;
    }
    out.extend(split(spec, rest, after_range, false));
    out
}

/// `text` split by `path_list`. Next to a range, the empty piece on its side is what is left of
/// the separator, not an empty path: `after_range` drops a leading one, `before_range` a
/// trailing one.
fn split<'a>(
    spec: &ManifestRules,
    text: &'a str,
    after_range: bool,
    before_range: bool,
) -> Vec<Piece<'a>> {
    let next_to_range = after_range || before_range;
    if next_to_range && text.trim().is_empty() {
        return Vec::new();
    }
    let mut parts: Vec<&str> = match &spec.path_list {
        Some(split) => split.split(text).collect(),
        None => vec![text],
    };
    if before_range && parts.last().is_some_and(|p| p.trim().is_empty()) {
        parts.pop();
    }
    if after_range && parts.first().is_some_and(|p| p.trim().is_empty()) {
        parts.remove(0);
    }
    parts.into_iter().map(|p| Piece::Token(p.trim())).collect()
}

/// A diagnostic code and message.
type Refusal = (&'static str, String);

/// One piece of a manifest item, after the file `previous` (decisions D32, D41): `:3-10` is more
/// lines of the previous path; with `siblings`, a bare file name after a path with a directory
/// is in that directory; a token with `*`, `?` or `{`, and a range, are patterns.
fn token(
    spec: &ManifestRules,
    piece: Piece<'_>,
    previous: Option<&(RepoPath, PathOrigin)>,
) -> Result<(ManifestPath, PathOrigin, Option<LineRange>), Refusal> {
    let piece = match piece {
        Piece::Range(start, end) => {
            let pattern =
                PathPattern::range(start, end).map_err(|e| ("manifest-item", e.to_string()))?;
            return Ok((ManifestPath::Pattern(pattern), PathOrigin::Written, None));
        }
        Piece::Token(piece) => piece,
    };
    if let Some(range) = piece.strip_prefix(':') {
        let Some((path, origin)) = previous else {
            return Err(("bad-path", format!("`{piece}` is not a path")));
        };
        return Ok((ManifestPath::File(path.clone()), *origin, line_range(range)));
    }
    if PathPattern::is_glob(piece) {
        let pattern = PathPattern::glob(piece).map_err(|e| ("manifest-item", e.to_string()))?;
        return Ok((ManifestPath::Pattern(pattern), PathOrigin::Written, None));
    }
    let (path, lines) = split_lines(piece);
    let dir = previous.and_then(|(p, _)| p.dir());
    let (spelled, origin) = match dir {
        Some(dir) if spec.siblings && !path.contains('/') => {
            (RepoPath::new(&format!("{dir}/{path}")), PathOrigin::Sibling)
        }
        _ => (RepoPath::new(path), PathOrigin::Written),
    };
    let path = spelled.map_err(|e| ("bad-path", e.to_string()))?;
    Ok((ManifestPath::File(path), origin, lines))
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

/// A step boundary's `n` capture.
enum StepNumber {
    /// No `n` capture.
    Unwritten,
    /// A number.
    Written(u32),
    /// Too large for `u32`.
    Bad,
}

/// The step that block `index` starts, as (number, title), when it is a step boundary.
fn boundary(
    rules: &OutlineRules,
    doc: &Document,
    index: usize,
) -> Option<(StepNumber, Option<TextRef>)> {
    let (kind, text) = match &doc.blocks[index].block {
        Block::Paragraph { text } => (StepBlock::Paragraph, text),
        Block::ListItem { text, .. } => (StepBlock::ListItem, text),
        Block::Heading { text, .. } => (StepBlock::Heading, text),
        _ => return None,
    };
    let base = text.span().start;
    let caps = rules
        .steps
        .iter()
        .filter(|r| r.block == kind)
        .find_map(|r| r.regex.captures(doc.text(text)))?;
    let number = match caps.name("n").map(|m| m.as_str().parse::<u32>()) {
        None => StepNumber::Unwritten,
        Some(Ok(n)) => StepNumber::Written(n),
        Some(Err(_)) => StepNumber::Bad,
    };
    Some((number, capture_ref(Some(&caps), "title", base)))
}

fn steps(
    rules: &OutlineRules,
    doc: &Document,
    body: BlockRange,
    manifest: Option<BlockRange>,
    errors: &mut Vec<Diagnostic>,
) -> Vec<OutlineStep> {
    let outside = |i: &usize| !manifest.is_some_and(|m| m.contains(*i));
    let boundaries: Vec<usize> = (body.start..body.end)
        .filter(outside)
        .filter(|&i| boundary(rules, doc, i).is_some())
        .collect();
    let mut steps = Vec::with_capacity(boundaries.len() + 1);
    let intro_end = boundaries.first().copied().unwrap_or(body.end);
    let intro = BlockRange {
        start: body.start,
        end: intro_end,
    };
    if (intro.start..intro.end).any(|i| outside(&i)) {
        steps.push(OutlineStep {
            number: None,
            title: None,
            span: range_span(doc, intro, manifest).unwrap_or(Span::new(0, 0)),
            blocks: intro,
            written: false,
            mark: None,
        });
    }
    for (k, &b) in boundaries.iter().enumerate() {
        let blocks = BlockRange {
            start: b + 1,
            end: boundaries.get(k + 1).copied().unwrap_or(body.end),
        };
        let Some((number, title)) = boundary(rules, doc, b) else {
            continue;
        };
        let start = doc.blocks[b].span;
        let number = match number {
            StepNumber::Written(n) => Some(n),
            StepNumber::Bad => {
                errors.push(Diagnostic::error(
                    "step-order",
                    Some(start),
                    "the step number is too large",
                ));
                None
            }
            StepNumber::Unwritten => None,
        };
        let mark = step_mark(rules, doc, b, title.as_ref());
        steps.push(OutlineStep {
            number,
            title,
            span: Span::new(start.start, range_end(doc, blocks).unwrap_or(start.end)),
            blocks,
            written: true,
            mark,
        });
    }
    steps
}

/// The mark of the step that block `index` starts: the first `[[step_marks]]` title regex that
/// matches its title (or, without a title capture, the whole boundary block).
fn step_mark(
    rules: &OutlineRules,
    doc: &Document,
    index: usize,
    title: Option<&TextRef>,
) -> Option<StepMark> {
    let text = match title {
        Some(title) => doc.text(title),
        None => doc.source.slice(doc.blocks[index].span),
    };
    rules
        .step_marks
        .iter()
        .find(|(regex, _)| regex.is_match(text))
        .map(|(_, mark)| *mark)
}

/// The span from the first to the last block of `range` outside `skip`.
fn range_span(doc: &Document, range: BlockRange, skip: Option<BlockRange>) -> Option<Span> {
    let mut inside = (range.start..range.end).filter(|i| !skip.is_some_and(|s| s.contains(*i)));
    let first = inside.next()?;
    let last = inside.next_back().unwrap_or(first);
    Some(Span::new(
        doc.blocks[first].span.start,
        doc.blocks[last].span.end,
    ))
}

fn range_end(doc: &Document, range: BlockRange) -> Option<u32> {
    (!range.is_empty()).then(|| doc.blocks[range.end - 1].span.end)
}

fn offset(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
#[path = "outline_tests.rs"]
mod tests;
