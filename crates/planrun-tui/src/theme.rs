//! The colour tokens (docs/tui-design.md §4, decision D89). v1 renders 24-bit colour only; each
//! token keeps its 256-colour index for the fallback that comes later.
//!
//! The rules: green only after verification, amber only when planrun waits for you (or for a
//! warning), red for failure and the Destructive class, blue for focus and the running action,
//! purple for identifiers. Bold and reverse are the only modifiers.

use ratatui::style::{Color, Modifier, Style};

/// One colour token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    /// App background.
    Bg0,
    /// Header, footer, panel titles, the dock's band.
    Bg1,
    /// Selection, key chips, inputs, overlays.
    Bg2,
    /// Hairlines between panels.
    Rule,
    /// The overlay frame.
    Border,
    /// Primary text.
    Fg,
    /// Secondary text.
    Fg2,
    /// Tertiary text: line numbers, timestamps.
    Fg3,
    /// Verified, done.
    Ok,
    /// planrun waits for you; warnings.
    Warn,
    /// Failure, error, the Destructive class.
    Err,
    /// Focus, the running action.
    Act,
    /// Hashes, ids, versions, absorbed blocks.
    Meta,
    /// A diff's added row.
    AddBg,
    /// A diff's deleted row.
    DelBg,
}

impl Token {
    /// Every token, in the design's order.
    pub const ALL: [Token; 15] = [
        Token::Bg0,
        Token::Bg1,
        Token::Bg2,
        Token::Rule,
        Token::Border,
        Token::Fg,
        Token::Fg2,
        Token::Fg3,
        Token::Ok,
        Token::Warn,
        Token::Err,
        Token::Act,
        Token::Meta,
        Token::AddBg,
        Token::DelBg,
    ];

    /// Its 24-bit value.
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            Token::Bg0 => (0x11, 0x18, 0x20),
            Token::Bg1 => (0x19, 0x23, 0x2E),
            Token::Bg2 => (0x26, 0x34, 0x42),
            Token::Rule => (0x2E, 0x3D, 0x4C),
            Token::Border => (0x52, 0x65, 0x79),
            Token::Fg => (0xD8, 0xE1, 0xE9),
            Token::Fg2 => (0x91, 0xA3, 0xB5),
            Token::Fg3 => (0x6C, 0x80, 0x94),
            Token::Ok => (0x69, 0xB9, 0x95),
            Token::Warn => (0xD9, 0xA8, 0x5F),
            Token::Err => (0xD8, 0x75, 0x75),
            Token::Act => (0x75, 0xA9, 0xD5),
            Token::Meta => (0xB6, 0x9B, 0xD9),
            Token::AddBg => (0x1B, 0x32, 0x29),
            Token::DelBg => (0x3A, 0x22, 0x28),
        }
    }

    /// Its index in the 256-colour palette, for the fallback mode (not used in v1).
    pub fn index256(self) -> u8 {
        match self {
            Token::Bg0 => 234,
            Token::Bg1 | Token::AddBg => 235,
            Token::Bg2 | Token::DelBg => 236,
            Token::Rule => 237,
            Token::Border => 60,
            Token::Fg => 254,
            Token::Fg2 => 109,
            Token::Fg3 => 66,
            Token::Ok => 72,
            Token::Warn => 179,
            Token::Err => 174,
            Token::Act => 110,
            Token::Meta => 140,
        }
    }

    /// Its name in the design (`bg0`, `fg2`, …).
    pub fn name(self) -> &'static str {
        match self {
            Token::Bg0 => "bg0",
            Token::Bg1 => "bg1",
            Token::Bg2 => "bg2",
            Token::Rule => "rule",
            Token::Border => "border",
            Token::Fg => "fg",
            Token::Fg2 => "fg2",
            Token::Fg3 => "fg3",
            Token::Ok => "ok",
            Token::Warn => "warn",
            Token::Err => "err",
            Token::Act => "act",
            Token::Meta => "meta",
            Token::AddBg => "addbg",
            Token::DelBg => "delbg",
        }
    }

    /// The ratatui colour.
    pub fn color(self) -> Color {
        let (r, g, b) = self.rgb();
        Color::Rgb(r, g, b)
    }

    /// The token whose colour is `color`, if any.
    pub fn of(color: Color) -> Option<Token> {
        Token::ALL.into_iter().find(|t| t.color() == color)
    }
}

/// Text in `fg` on the background of whatever is under it.
pub fn fg(fg: Token) -> Style {
    Style::new().fg(fg.color())
}

/// Text in `fg` on `bg`.
pub fn on(fg: Token, bg: Token) -> Style {
    Style::new().fg(fg.color()).bg(bg.color())
}

/// Bold text in `fg`.
pub fn bold(fg: Token) -> Style {
    Style::new().fg(fg.color()).add_modifier(Modifier::BOLD)
}

/// The app's background with primary text.
pub fn base() -> Style {
    on(Token::Fg, Token::Bg0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_token_has_its_own_colour_and_name() {
        for (i, a) in Token::ALL.iter().enumerate() {
            for b in &Token::ALL[i + 1..] {
                assert_ne!(a.rgb(), b.rgb(), "{} and {}", a.name(), b.name());
                assert_ne!(a.name(), b.name());
            }
            assert_eq!(Token::of(a.color()), Some(*a));
        }
    }

    #[test]
    fn the_256_indices_are_the_design_s() {
        let indices: Vec<u8> = Token::ALL.iter().map(|t| t.index256()).collect();
        assert_eq!(
            indices,
            [
                234, 235, 236, 237, 60, 254, 109, 66, 72, 179, 174, 110, 140, 235, 236
            ]
        );
    }
}
