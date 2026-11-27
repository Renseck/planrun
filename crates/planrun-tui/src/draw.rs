//! Painting on the cell grid: a pen that writes styled text left to right on one row, key chips,
//! bands and hairlines. Every view draws through these, so the spacing rules live in one place.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::glyph::{Glyph, Glyphs};
use crate::text;
use crate::theme::{Token, bold, fg, on};

/// Writes styled text along one row, from `x` up to `right` (exclusive); text past `right` is
/// cut with an ellipsis.
pub struct Pen<'b> {
    buf: &'b mut Buffer,
    /// The next column.
    pub x: u16,
    y: u16,
    right: u16,
    glyphs: Glyphs,
}

impl<'b> Pen<'b> {
    /// A pen at (`x`, `y`) that stops before `right`.
    pub fn new(buf: &'b mut Buffer, x: u16, y: u16, right: u16, glyphs: Glyphs) -> Pen<'b> {
        Pen {
            buf,
            x,
            y,
            right,
            glyphs,
        }
    }

    /// The cells left before `right`.
    pub fn room(&self) -> usize {
        usize::from(self.right.saturating_sub(self.x))
    }

    /// Writes `s` in `style`.
    pub fn text(&mut self, s: &str, style: Style) -> &mut Self {
        let room = self.room();
        if room == 0 {
            return self;
        }
        let shown = text::fit(s, room, self.glyphs.get(Glyph::Ellipsis));
        let w = text::width(&shown);
        self.buf.set_stringn(self.x, self.y, &shown, room, style);
        self.x += u16::try_from(w).unwrap_or(u16::MAX);
        self
    }

    /// Writes `glyph` in `style`.
    pub fn glyph(&mut self, glyph: Glyph, style: Style) -> &mut Self {
        let s = self.glyphs.get(glyph);
        self.text(s, style)
    }

    /// Skips `n` cells.
    pub fn gap(&mut self, n: u16) -> &mut Self {
        self.x = (self.x + n).min(self.right);
        self
    }

    /// Moves to column `x` (never backwards).
    pub fn to(&mut self, x: u16) -> &mut Self {
        self.x = self.x.max(x).min(self.right);
        self
    }

    /// Writes `s` so that it ends at `right` (or right after what is already written).
    pub fn right(&mut self, s: &str, style: Style) -> &mut Self {
        let w = u16::try_from(text::width(s)).unwrap_or(u16::MAX);
        let at = self.right.saturating_sub(w).max(self.x);
        self.x = at;
        self.text(s, style)
    }

    /// Writes `s` padded or cut to exactly `cells` cells.
    pub fn cell(&mut self, s: &str, cells: u16, style: Style) -> &mut Self {
        let start = self.x;
        self.text(
            &text::fit(s, usize::from(cells), self.glyphs.get(Glyph::Ellipsis)),
            style,
        );
        self.x = (start + cells).min(self.right);
        self
    }

    /// A key chip: ` key ` in bold on bg2, then `label` in fg2, on the band's background.
    pub fn chip(&mut self, key: &str, label: &str, band: Token) -> &mut Self {
        self.text(&format!(" {key} "), bold(Token::Fg).bg(Token::Bg2.color()));
        self.text(&format!(" {label}"), on(Token::Fg2, band))
    }
}

/// Fills `area` with `style` (spaces).
pub fn fill(buf: &mut Buffer, area: Rect, style: Style) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.reset();
                cell.set_style(style);
            }
        }
    }
}

/// A vertical hairline at column `x` from `top` to `bottom` (exclusive).
pub fn vrule(buf: &mut Buffer, x: u16, top: u16, bottom: u16, glyphs: Glyphs) {
    for y in top..bottom {
        buf.set_string(x, y, glyphs.get(Glyph::VRule), on(Token::Rule, Token::Bg0));
    }
}

/// A horizontal hairline on row `y` from `left` to `right` (exclusive).
pub fn hrule(buf: &mut Buffer, left: u16, right: u16, y: u16, glyphs: Glyphs) {
    let line = glyphs
        .get(Glyph::HRule)
        .repeat(usize::from(right.saturating_sub(left)));
    buf.set_string(left, y, line, fg(Token::Rule));
}

/// A section title in a panel: `TITLE ───── right`, the title bold fg3, the rule to `right`.
pub fn section(
    buf: &mut Buffer,
    area: Rect,
    y: u16,
    title: &str,
    tail: &[(String, Style)],
    glyphs: Glyphs,
) {
    let tail_w: usize = tail.iter().map(|(s, _)| text::width(s)).sum();
    let mut pen = Pen::new(buf, area.x, y, area.right(), glyphs);
    pen.text(title, bold(Token::Fg3)).gap(1);
    let rule_end = usize::from(area.right()).saturating_sub(tail_w + usize::from(!tail.is_empty()));
    let n = rule_end.saturating_sub(usize::from(pen.x));
    pen.text(&glyphs.get(Glyph::HRule).repeat(n), fg(Token::Rule));
    if !tail.is_empty() {
        pen.gap(1);
        for (s, style) in tail {
            pen.text(s, *style);
        }
    }
}

/// A panel's band: bg1, with `▎` and the title in act bold when the panel has focus, the title
/// in fg2 bold otherwise, then `detail` in fg3.
pub fn band(
    buf: &mut Buffer,
    area: Rect,
    focused: bool,
    title: &str,
    detail: &str,
    glyphs: Glyphs,
) -> u16 {
    let row = Rect::new(area.x, area.y, area.width, 1);
    fill(buf, row, on(Token::Fg, Token::Bg1));
    let mut pen = Pen::new(buf, area.x, area.y, area.right(), glyphs);
    if focused {
        pen.glyph(Glyph::Focus, on(Token::Act, Token::Bg1)).gap(1);
        pen.text(title, bold(Token::Act).bg(Token::Bg1.color()));
    } else {
        pen.gap(2);
        pen.text(title, bold(Token::Fg2).bg(Token::Bg1.color()));
    }
    pen.gap(2).text(detail, on(Token::Fg3, Token::Bg1));
    pen.x
}

/// The selected row's style: bg2 under the whole row, `▌` in act at its first cell.
pub fn select_row(buf: &mut Buffer, area: Rect, y: u16, glyphs: Glyphs) {
    for x in area.left()..area.right() {
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_bg(Token::Bg2.color());
        }
    }
    buf.set_string(
        area.x,
        y,
        glyphs.get(Glyph::Select),
        on(Token::Act, Token::Bg2),
    );
}

/// `style` made bold.
pub fn strong(style: Style) -> Style {
    style.add_modifier(Modifier::BOLD)
}
