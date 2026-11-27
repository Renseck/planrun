//! The command palette (docs/tui-design.md §6.G): an overlay on a dimmed view, grouped results,
//! the matched characters in act, the shortcuts on the right. Commands come from a registry the
//! app builds from what is on screen; `ctrl+p` is the same overlay limited to the current plan's
//! files and actions. A command does what its key does: the palette decides nothing.

use std::path::PathBuf;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::draw::{Pen, fill, hrule};
use crate::glyph::{Glyph, Glyphs};
use crate::history::RunEntry;
use crate::keys::Key;
use crate::text;
use crate::theme::{Token, bold, fg, on};
use crate::tree::Node;

/// What a command does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cmd {
    /// Open a plan in Inspect (`enter` in Plans).
    Inspect(PathBuf),
    /// Run the inspected plan with its assumed policy, from a task (`R`, `shift+R`).
    Run(Option<usize>),
    /// Open the assumed policy (`a`).
    Policy,
    /// The resume preview of a run (`R` in History).
    Preview(RunEntry),
    /// A key, as if pressed (F2, F4, a console filter, `c`, `q`).
    Key(Key),
    /// Select a node of the plan's tree (`ctrl+p`).
    Select(Node),
}

/// What a key did in the palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pick {
    /// Still open.
    Open,
    /// A command was chosen; the palette closes.
    Chosen(Cmd),
    /// Closed without a command.
    Closed,
}

/// One command of the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// Its group (`PLAN`, `HISTORY`, `VIEW`).
    pub group: &'static str,
    /// What it says.
    pub title: String,
    /// Its shortcut, shown on the right.
    pub key: String,
    /// What it does.
    pub cmd: Cmd,
}

/// The open palette.
#[derive(Debug, Clone)]
pub struct Palette {
    /// `ctrl+p`: the plan's files and actions only.
    pub go_to: bool,
    /// The registry, in group order.
    pub commands: Vec<Command>,
    /// What was typed.
    pub query: String,
    /// The selected result.
    pub selected: usize,
}

/// The positions of `query`'s characters in `title`, matched in order ignoring case; `None`
/// when it does not match. An empty query matches everything.
pub fn matches(query: &str, title: &str) -> Option<Vec<usize>> {
    let mut at = Vec::new();
    let mut chars = title
        .char_indices()
        .map(|(i, c)| (i, c.to_ascii_lowercase()));
    for q in query.chars().map(|c| c.to_ascii_lowercase()) {
        loop {
            let (i, c) = chars.next()?;
            if c == q {
                at.push(i);
                break;
            }
        }
    }
    Some(at)
}

impl Palette {
    /// The palette over `commands`.
    pub fn new(go_to: bool, commands: Vec<Command>) -> Palette {
        Palette {
            go_to,
            commands,
            query: String::new(),
            selected: 0,
        }
    }

    /// The results: each command that matches, with its matched positions.
    pub fn results(&self) -> Vec<(&Command, Vec<usize>)> {
        self.commands
            .iter()
            .filter_map(|c| matches(&self.query, &c.title).map(|m| (c, m)))
            .collect()
    }

    /// A key: the command chosen on enter, or the palette closed.
    pub fn key(&mut self, key: Key) -> Pick {
        let n = self.results().len();
        match key {
            Key::Esc | Key::Ctrl('k') | Key::Ctrl('p') => return Pick::Closed,
            Key::Char(c) => {
                self.query.push(c);
                self.selected = 0;
            }
            Key::Backspace => {
                self.query.pop();
                self.selected = 0;
            }
            Key::Up => self.selected = self.selected.saturating_sub(1),
            Key::Down => self.selected = (self.selected + 1).min(n.saturating_sub(1)),
            Key::Enter => {
                return match self.results().get(self.selected) {
                    Some((c, _)) => Pick::Chosen(c.cmd.clone()),
                    None => Pick::Closed,
                };
            }
            _ => {}
        }
        Pick::Open
    }

    /// Draws the palette over `area`, dimming what is under it.
    pub fn draw(&self, buf: &mut Buffer, area: Rect, g: Glyphs) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_fg(Token::Fg3.color());
                    cell.modifier = ratatui::style::Modifier::empty();
                }
            }
        }
        let width = area.width.saturating_sub(30).clamp(60, 92);
        let results = self.results();
        let rows = u16::try_from(results.len() + 3 * 3).unwrap_or(u16::MAX);
        let height = (rows + 4).min(area.height.saturating_sub(4));
        let x = area.x + (area.width - width) / 2;
        let y = area.y + 3;
        let frame = Rect::new(x, y, width, height);
        fill(buf, frame, on(Token::Fg, Token::Bg2));
        let inner = Rect::new(x + 2, y, width - 4, height);
        let mut pen = Pen::new(buf, inner.x, y + 1, inner.right(), g);
        pen.text(g.get(Glyph::Crumb), bold(Token::Act).bg(Token::Bg2.color()))
            .gap(1);
        pen.text(&self.query, bold(Token::Fg).bg(Token::Bg2.color()));
        pen.glyph(Glyph::Caret, on(Token::Act, Token::Bg2));
        let count = format!("{} of {}", results.len(), self.commands.len());
        Pen::new(buf, inner.x, y + 1, inner.right(), g).right(&count, on(Token::Fg3, Token::Bg2));
        hrule(buf, x, x + width, y + 2, g);
        let mut row = y + 3;
        let last = y + height - 2;
        let mut group = "";
        for (i, (cmd, at)) in results.iter().enumerate() {
            if row >= last {
                break;
            }
            if cmd.group != group {
                group = cmd.group;
                Pen::new(buf, inner.x, row, inner.right(), g)
                    .text(group, bold(Token::Fg3).bg(Token::Bg2.color()));
                row += 1;
                if row >= last {
                    break;
                }
            }
            let selected = i == self.selected;
            let bg = if selected { Token::Bg1 } else { Token::Bg2 };
            fill(buf, Rect::new(x, row, width, 1), on(Token::Fg, bg));
            if selected {
                Pen::new(buf, x, row, x + 1, g).glyph(Glyph::Select, on(Token::Act, bg));
            }
            let mut pen = Pen::new(buf, inner.x + 2, row, inner.right().saturating_sub(10), g);
            for (j, c) in cmd.title.char_indices() {
                let style = if at.contains(&j) {
                    bold(Token::Act)
                } else {
                    fg(Token::Fg)
                };
                pen.text(&c.to_string(), style.bg(bg.color()));
            }
            Pen::new(buf, inner.x, row, inner.right(), g).right(&cmd.key, on(Token::Fg3, bg));
            row += 1;
        }
        let note = if self.go_to {
            "the plan's files and actions"
        } else {
            "commands come from a registry; each view adds its own"
        };
        Pen::new(buf, inner.x, y + height - 1, inner.right(), g).text(
            &text::fit(note, usize::from(inner.width), "…"),
            on(Token::Fg3, Token::Bg2),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_matches_its_characters_in_order() {
        assert_eq!(matches("run", "Run assay-style.md"), Some(vec![0, 1, 2]));
        assert_eq!(
            matches("rasy", "Run assay-style.md"),
            Some(vec![0, 4, 5, 8])
        );
        assert_eq!(matches("zz", "Run"), None);
        assert_eq!(matches("", "Run"), Some(vec![]));
    }

    #[test]
    fn enter_chooses_the_selected_result_and_esc_closes() {
        let cmd = |title: &str, key: Key| Command {
            group: "VIEW",
            title: title.to_owned(),
            key: String::new(),
            cmd: Cmd::Key(key),
        };
        let mut p = Palette::new(
            false,
            vec![cmd("Plans", Key::F(2)), cmd("History", Key::F(4))],
        );
        for c in "his".chars() {
            assert_eq!(p.key(Key::Char(c)), Pick::Open);
        }
        assert_eq!(p.key(Key::Enter), Pick::Chosen(Cmd::Key(Key::F(4))));
        assert_eq!(p.key(Key::Esc), Pick::Closed);
    }
}
