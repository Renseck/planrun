//! The shell every view shares (docs/tui-design.md §3): the header band with the crumbs and the
//! state on the right, the footer's contextual keys (from 30 rows up) and its navigation line
//! with the target, the plans directory and the update indicator pinned bottom-right.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::draw::{Pen, fill};
use crate::glyph::{Glyph, Glyphs};
use crate::jobs::{Context, UpdateState};
use crate::text;
use crate::theme::{Token, bold, on};

/// The smallest terminal v1 lays out (§11): below it the TUI says so.
pub const MIN: (u16, u16) = (100, 30);

/// The views F2–F4 switch between, and F1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nav {
    /// F1.
    Help,
    /// F2.
    Plans,
    /// F3.
    Plan,
    /// F4.
    History,
}

/// The header: ` PLANRUN  crumb › crumb › current`, and `right` ending one cell before the edge.
pub fn header(
    buf: &mut Buffer,
    area: Rect,
    crumbs: &[String],
    right: &[(String, Style)],
    g: Glyphs,
) {
    let row = Rect::new(area.x, area.y, area.width, 1);
    fill(buf, row, on(Token::Fg, Token::Bg1));
    let right_w: usize = right.iter().map(|(s, _)| text::width(s)).sum();
    let crumbs_end = area
        .right()
        .saturating_sub(u16::try_from(right_w).unwrap_or(u16::MAX) + 2);
    let mut pen = Pen::new(buf, area.x + 1, area.y, crumbs_end, g);
    pen.text("PLANRUN", bold(Token::Fg).bg(Token::Bg1.color()))
        .gap(2);
    for (i, crumb) in crumbs.iter().enumerate() {
        if i > 0 {
            pen.text(
                &format!(" {} ", g.get(Glyph::Crumb)),
                on(Token::Fg3, Token::Bg1),
            );
        }
        let style = if i + 1 == crumbs.len() {
            bold(Token::Fg).bg(Token::Bg1.color())
        } else {
            on(Token::Fg2, Token::Bg1)
        };
        pen.text(crumb, style);
    }
    let start = area
        .right()
        .saturating_sub(u16::try_from(right_w).unwrap_or(0) + 1);
    let mut pen = Pen::new(buf, start, area.y, area.right().saturating_sub(1), g);
    for (s, style) in right {
        pen.text(s, *style);
    }
}

/// The contextual keys: chips on bg1, three cells apart.
pub fn keys(buf: &mut Buffer, area: Rect, y: u16, keys: &[(String, String)], g: Glyphs) {
    let row = Rect::new(area.x, y, area.width, 1);
    fill(buf, row, on(Token::Fg, Token::Bg1));
    let mut pen = Pen::new(buf, area.x + 1, y, area.right(), g);
    for (i, (key, label)) in keys.iter().enumerate() {
        if i > 0 {
            pen.gap(3);
        }
        pen.chip(key, label, Token::Bg1);
    }
}

/// The navigation line: F1–F4 (the active one in act on bg2), the target and the plans
/// directory, and the update indicator, never green or red.
pub fn nav(
    buf: &mut Buffer,
    area: Rect,
    y: u16,
    active: Nav,
    ctx: &Context,
    update: &UpdateState,
    g: Glyphs,
) {
    let row = Rect::new(area.x, y, area.width, 1);
    fill(buf, row, on(Token::Fg, Token::Bg1));
    let (indicator, style) = indicator(update, &ctx.version, g);
    let end = area
        .right()
        .saturating_sub(u16::try_from(text::width(&indicator)).unwrap_or(0) + 2);
    let mut pen = Pen::new(buf, area.x + 1, y, end, g);
    let tabs = [
        (Nav::Help, "F1 Help"),
        (Nav::Plans, "F2 Plans"),
        (Nav::Plan, "F3 Plan"),
        (Nav::History, "F4 History"),
    ];
    for (i, (nav, label)) in tabs.iter().enumerate() {
        if i > 0 {
            pen.gap(1);
        }
        let style = if *nav == active {
            bold(Token::Act).bg(Token::Bg2.color())
        } else {
            on(Token::Fg3, Token::Bg1)
        };
        pen.text(&format!(" {label} "), style);
    }
    pen.gap(4);
    pen.text("target ", on(Token::Fg3, Token::Bg1))
        .text(&ctx.target, on(Token::Fg2, Token::Bg1))
        .text(
            &format!("  {}  plans ", g.get(Glyph::Dot)),
            on(Token::Fg3, Token::Bg1),
        )
        .text(&ctx.plans_label, on(Token::Fg2, Token::Bg1));
    let mut pen = Pen::new(buf, end, y, area.right().saturating_sub(1), g);
    pen.right(&indicator, style);
}

fn indicator(update: &UpdateState, version: &str, g: Glyphs) -> (String, Style) {
    match update {
        UpdateState::Unchecked | UpdateState::Current => {
            (format!("planrun {version}"), on(Token::Fg3, Token::Bg1))
        }
        UpdateState::Available(v) => (
            format!("{} {v} available", g.get(Glyph::Up)),
            on(Token::Meta, Token::Bg1),
        ),
        UpdateState::Failed => ("update check failed".to_owned(), on(Token::Fg3, Token::Bg1)),
    }
}

/// The notice below 100 × 30 (§11): the run, if any, goes on underneath.
pub fn too_small(buf: &mut Buffer, area: Rect, g: Glyphs) {
    let lines = [
        "terminal too small".to_owned(),
        format!("needs {} {} {}", MIN.0, "×", MIN.1),
        format!("now {} {} {}", area.width, "×", area.height),
    ];
    let top = area.y + area.height / 2 - 1;
    for (i, line) in lines.iter().enumerate() {
        let w = u16::try_from(text::width(line)).unwrap_or(0);
        let x = area.x + area.width.saturating_sub(w) / 2;
        let style = if i == 0 {
            bold(Token::Fg)
        } else {
            on(Token::Fg3, Token::Bg0)
        };
        let mut pen = Pen::new(buf, x, top + u16::try_from(i).unwrap_or(0), area.right(), g);
        pen.text(line, style);
    }
}
