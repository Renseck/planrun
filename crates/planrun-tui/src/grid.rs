//! The cell grid as text, for snapshots (decision D93): each row's characters, then its style
//! runs by token name (`3-9 act/bg2 b`), so a snapshot can be read against the design's
//! artboards cell for cell. Cells in the app's base style (fg on bg0, no modifier) are left out
//! of the runs.

use std::fmt::Write;

use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};

use crate::theme::Token;

/// The grid of `buffer`: one text line per row (trailing spaces cut), followed by the row's
/// style runs when it has any.
pub fn grid(buffer: &Buffer) -> String {
    let area = buffer.area;
    let mut out = String::new();
    for y in area.top()..area.bottom() {
        let mut text = String::new();
        let mut runs: Vec<(u16, u16, String)> = Vec::new();
        let mut x = area.left();
        while x < area.right() {
            let cell = &buffer[(x, y)];
            text.push_str(cell.symbol());
            let style = describe(cell.fg, cell.bg, cell.modifier);
            match runs.last_mut() {
                Some((_, end, s)) if *s == style && *end + 1 == x => *end = x,
                _ => runs.push((x, x, style)),
            }
            x += 1;
        }
        let _ = writeln!(out, "{y:>2} {}", text.trim_end());
        let shown: Vec<String> = runs
            .into_iter()
            .filter(|(_, _, s)| s != "fg/bg0")
            .map(|(a, b, s)| format!("{a}-{b} {s}"))
            .collect();
        if !shown.is_empty() {
            let _ = writeln!(out, "   {}", shown.join(", "));
        }
    }
    out
}

fn describe(fg: Color, bg: Color, modifier: Modifier) -> String {
    let mut s = format!("{}/{}", color(fg), color(bg));
    if modifier.contains(Modifier::BOLD) {
        s.push_str(" b");
    }
    if modifier.contains(Modifier::REVERSED) {
        s.push_str(" r");
    }
    s
}

fn color(c: Color) -> String {
    if let Some(token) = Token::of(c) {
        return token.name().to_owned();
    }
    match c {
        Color::Reset => "-".to_owned(),
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;
    use ratatui::style::Style;

    use super::*;
    use crate::theme::{base, bold, on};

    #[test]
    fn a_row_shows_its_text_and_its_styled_runs() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 12, 2));
        buffer.set_style(buffer.area, base());
        buffer.set_string(0, 0, " PLANRUN", on(Token::Fg, Token::Bg1));
        buffer.set_string(1, 0, "PLANRUN", bold(Token::Fg).bg(Token::Bg1.color()));
        buffer.set_string(9, 1, "ok", Style::new().fg(Token::Ok.color()));
        let text = grid(&buffer);
        assert_eq!(
            text,
            " 0  PLANRUN\n   0-0 fg/bg1, 1-7 fg/bg1 b\n 1          ok\n   9-10 ok/bg0\n"
        );
    }
}
