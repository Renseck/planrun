//! Unified diffs as rows (docs/tui-design.md §6.B/C): the planned diff of a file action in
//! Inspect, and `git diff HEAD` of a task's files in Run. Split view and syntax highlighting are
//! not in v1.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use planrun_model::{ActionKind, Plan, WriteMode};

use crate::draw::{Pen, fill};
use crate::glyph::{Glyph, Glyphs};
use crate::text;
use crate::theme::{Token, bold, fg, on};

/// What a row of a diff is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// An unchanged line.
    Context,
    /// An added line.
    Add,
    /// A deleted line.
    Del,
    /// A note about the diff (`@@`, `deleted`, `overwrites the file on disk`).
    Note,
}

/// One row: the old and new line numbers, what it is, its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The line in the old file.
    pub old: Option<u32>,
    /// The line in the new file.
    pub new: Option<u32>,
    /// What it is.
    pub kind: Kind,
    /// The text without its sign.
    pub text: String,
}

/// A file's diff.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileDiff {
    /// The path.
    pub path: String,
    /// Its rows.
    pub rows: Vec<Row>,
}

impl FileDiff {
    /// Added and deleted lines.
    pub fn counts(&self) -> (usize, usize) {
        let add = self.rows.iter().filter(|r| r.kind == Kind::Add).count();
        let del = self.rows.iter().filter(|r| r.kind == Kind::Del).count();
        (add, del)
    }
}

fn row(old: Option<u32>, new: Option<u32>, kind: Kind, text: &str) -> Row {
    Row {
        old,
        new,
        kind,
        text: text.to_owned(),
    }
}

/// Parses unified diff text (a plan's patch, or `git diff` output) into one diff per file.
pub fn parse(patch: &str) -> Vec<FileDiff> {
    let mut files: Vec<FileDiff> = Vec::new();
    let (mut old, mut new) = (0u32, 0u32);
    for line in patch.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            let path = rest.rsplit(" b/").next().unwrap_or(rest).to_owned();
            files.push(FileDiff {
                path,
                rows: Vec::new(),
            });
            continue;
        }
        if line.starts_with("+++ ") || line.starts_with("--- ") || line.starts_with("index ") {
            continue;
        }
        let Some(file) = files.last_mut() else {
            continue;
        };
        if let Some(hunk) = line.strip_prefix("@@ ") {
            (old, new) = hunk_starts(hunk);
            file.rows.push(row(None, None, Kind::Note, line));
            continue;
        }
        if let Some(text) = line.strip_prefix('+') {
            file.rows.push(row(None, Some(new), Kind::Add, text));
            new += 1;
        } else if let Some(text) = line.strip_prefix('-') {
            file.rows.push(row(Some(old), None, Kind::Del, text));
            old += 1;
        } else if let Some(text) = line.strip_prefix(' ').or(line.is_empty().then_some("")) {
            file.rows
                .push(row(Some(old), Some(new), Kind::Context, text));
            old += 1;
            new += 1;
        } else {
            file.rows.push(row(None, None, Kind::Note, line));
        }
    }
    files
}

/// `-a,b +c,d @@` → (a, c).
fn hunk_starts(hunk: &str) -> (u32, u32) {
    let mut parts = hunk.split_whitespace();
    let start = |p: Option<&str>| {
        p.and_then(|p| p[1..].split(',').next().and_then(|n| n.parse().ok()))
            .unwrap_or(1)
    };
    let old = start(parts.next());
    let new = start(parts.next());
    (old, new)
}

/// The planned diff of a file action, from the plan alone.
pub fn planned(plan: &Plan, kind: &ActionKind) -> Option<FileDiff> {
    let file = |path: &dyn ToString, rows| FileDiff {
        path: path.to_string(),
        rows,
    };
    match kind {
        ActionKind::WriteFile { path, body, mode } => {
            let mut rows: Vec<Row> = plan
                .text(body)
                .lines()
                .enumerate()
                .map(|(i, l)| row(None, u32::try_from(i + 1).ok(), Kind::Add, l))
                .collect();
            if *mode == WriteMode::Overwrite {
                rows.insert(
                    0,
                    row(None, None, Kind::Note, "overwrites the whole file on disk"),
                );
            }
            Some(file(path, rows))
        }
        ActionKind::ApplyPatch { patch, path, .. } => {
            let mut parsed = parse(plan.text(patch));
            let rows = if parsed.is_empty() {
                Vec::new()
            } else {
                parsed.remove(0).rows
            };
            Some(file(path, rows))
        }
        ActionKind::ReplaceExact {
            path,
            before,
            after,
        } => {
            let mut rows: Vec<Row> = plan
                .text(before)
                .lines()
                .map(|l| row(None, None, Kind::Del, l))
                .collect();
            rows.extend(
                plan.text(after)
                    .lines()
                    .map(|l| row(None, None, Kind::Add, l)),
            );
            Some(file(path, rows))
        }
        ActionKind::Delete { path, via_git } => {
            let what = if *via_git {
                "deleted (git rm)"
            } else {
                "deleted"
            };
            Some(file(path, vec![row(None, None, Kind::Note, what)]))
        }
        _ => None,
    }
}

/// The diff's band: the path in bold, `+n −m`, the change-state chip, and `unified` on the
/// right (the split view, `s` at 160 columns, is not in v1).
pub fn header(
    buf: &mut Buffer,
    area: Rect,
    diff: &FileDiff,
    chip: (&str, Token),
    detail: &str,
    g: Glyphs,
) {
    let rowr = Rect::new(area.x, area.y, area.width, 1);
    fill(buf, rowr, on(Token::Fg, Token::Bg1));
    let (add, del) = diff.counts();
    let mut pen = Pen::new(buf, area.x + 1, area.y, area.right(), g);
    pen.text(&diff.path, bold(Token::Fg).bg(Token::Bg1.color()))
        .gap(2);
    pen.text(
        &format!("+{add} {}{del}", g.get(Glyph::Delete)),
        on(Token::Fg2, Token::Bg1),
    )
    .gap(4);
    pen.text(
        &format!(" {} ", chip.0),
        bold(chip.1).bg(Token::Bg2.color()),
    );
    if !detail.is_empty() {
        pen.gap(2).text(detail, on(Token::Fg3, Token::Bg1));
    }
    let x = pen.x;
    let mut pen = Pen::new(buf, x, area.y, area.right().saturating_sub(2), g);
    pen.right("unified", on(Token::Fg3, Token::Bg1));
}

/// Draws `rows` from `skip` in `area`: old and new line numbers, the sign, the text; added and
/// deleted rows tinted.
pub fn draw_rows(buf: &mut Buffer, area: Rect, rows: &[Row], skip: usize, g: Glyphs) {
    for (i, r) in rows
        .iter()
        .skip(skip)
        .take(usize::from(area.height))
        .enumerate()
    {
        let y = area.y + u16::try_from(i).unwrap_or(0);
        let (bg, sign, sign_style, text_color) = match r.kind {
            Kind::Context => (Token::Bg0, " ", fg(Token::Fg3), Token::Fg2),
            Kind::Add => (Token::AddBg, "+", bold(Token::Ok), Token::Fg),
            Kind::Del => (Token::DelBg, "-", bold(Token::Err), Token::Fg),
            Kind::Note => (Token::Bg0, " ", fg(Token::Fg3), Token::Fg3),
        };
        if bg != Token::Bg0 {
            fill(buf, Rect::new(area.x, y, area.width, 1), on(Token::Fg, bg));
        }
        let num = |n: Option<u32>| {
            n.map_or_else(|| "    ".to_owned(), |n| text::right(&n.to_string(), 4))
        };
        let mut pen = Pen::new(buf, area.x + 1, y, area.right(), g);
        let numbers = on(Token::Fg3, bg);
        if r.kind == Kind::Note {
            pen.gap(10).text(&r.text, numbers);
            continue;
        }
        pen.text(&num(r.old), numbers)
            .gap(1)
            .text(&num(r.new), numbers)
            .gap(1);
        pen.text(sign, sign_style.bg(bg.color())).gap(1);
        pen.text(&r.text, on(text_color, bg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_patch_becomes_numbered_rows() {
        let patch = "diff --git a/src/lib.rs b/src/lib.rs\nindex aa80cf1..97bd92a 100644\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,3 +1,3 @@\n a\n-b\n+c\n d\n";
        let files = parse(patch);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "src/lib.rs");
        let rows: Vec<(Option<u32>, Option<u32>, Kind)> = files[0]
            .rows
            .iter()
            .map(|r| (r.old, r.new, r.kind))
            .collect();
        assert_eq!(
            rows,
            [
                (None, None, Kind::Note),
                (Some(1), Some(1), Kind::Context),
                (Some(2), None, Kind::Del),
                (None, Some(2), Kind::Add),
                (Some(3), Some(3), Kind::Context),
            ]
        );
        assert_eq!(files[0].counts(), (1, 1));
    }
}
