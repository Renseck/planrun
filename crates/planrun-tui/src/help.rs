//! Help (F1, `?`): the keys of docs/tui-design.md §7 over the view, and the view's own keys
//! from its footer.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::draw::{Pen, fill, hrule};
use crate::glyph::Glyphs;
use crate::theme::{Token, bold, on};

/// The keys, as §7 lists them.
pub const KEYS: [(&str, &str); 17] = [
    ("F1 / ?", "help"),
    ("F2 F3 F4", "Plans · Plan · History"),
    ("↑↓ j k", "move"),
    ("←→ h l", "fold and unfold"),
    ("enter", "open / inspect"),
    ("esc", "back (in a dock: nothing; answers are explicit)"),
    ("tab shift+tab", "focus the next or previous panel"),
    (
        "R / shift+R",
        "run / run from this task, with Inspect's policy",
    ),
    ("a", "Inspect: the assumed policy"),
    ("c", "check the plans again"),
    ("o", "open a shell, when a question offers it"),
    ("ctrl+c", "end planrun after a confirm in the dock (D91)"),
    ("f · 1–5", "console: follow · filter"),
    ("ctrl+k", "commands"),
    ("ctrl+p", "go to a file or an action of the plan"),
    ("q", "back; quits only from Plans with no run active"),
    (
        "typed answers",
        "the whole word, then enter; a single key never answers",
    ),
];

/// Draws help over `area`; `view` is the current view's footer keys.
pub fn draw(buf: &mut Buffer, area: Rect, view: &[(String, String)], g: Glyphs) {
    let width = 84.min(area.width.saturating_sub(4));
    let rows = u16::try_from(KEYS.len() + view.len() + 6).unwrap_or(u16::MAX);
    let height = rows.min(area.height.saturating_sub(2));
    let x = area.x + (area.width - width) / 2;
    let y = area.y + 1;
    let frame = Rect::new(x, y, width, height);
    fill(buf, frame, on(Token::Fg, Token::Bg2));
    let mut row = y + 1;
    Pen::new(buf, x + 2, row, x + width - 2, g)
        .text("KEYS", bold(Token::Act).bg(Token::Bg2.color()));
    Pen::new(buf, x + 2, row, x + width - 2, g).right("esc closes", on(Token::Fg3, Token::Bg2));
    row += 1;
    for (key, what) in KEYS {
        if row >= y + height - 1 {
            return;
        }
        let mut pen = Pen::new(buf, x + 2, row, x + width - 2, g);
        pen.cell(key, 16, bold(Token::Fg).bg(Token::Bg2.color()));
        pen.text(what, on(Token::Fg2, Token::Bg2));
        row += 1;
    }
    if view.is_empty() || row + 2 >= y + height {
        return;
    }
    hrule(buf, x, x + width, row, g);
    row += 1;
    Pen::new(buf, x + 2, row, x + width - 2, g)
        .text("THIS VIEW", bold(Token::Fg3).bg(Token::Bg2.color()));
    row += 1;
    for (key, what) in view {
        if row >= y + height - 1 {
            return;
        }
        let mut pen = Pen::new(buf, x + 2, row, x + width - 2, g);
        pen.cell(key, 16, bold(Token::Fg).bg(Token::Bg2.color()));
        pen.text(what, on(Token::Fg2, Token::Bg2));
        row += 1;
    }
}
