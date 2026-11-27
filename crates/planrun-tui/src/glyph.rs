//! The glyphs (docs/tui-design.md §5, decision D89): every one is a single cell wide, has an
//! ASCII fallback, and `PLANRUN_ASCII=1` switches the whole set to the fallbacks. No Nerd Font,
//! no wide or emoji glyph. A unit test pins the widths; the crate README holds the manual check
//! in Windows Terminal with Cascadia Mono.

/// One glyph the TUI draws itself (plan text is shown as written).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Glyph {
    /// `○` pending.
    Pending,
    /// `◉` running.
    Running,
    /// `✓` done, verified.
    Done,
    /// `✕` failed.
    Failed,
    /// `⊘` skipped.
    Skipped,
    /// `◆` waiting for you.
    Waiting,
    /// `!` mismatch, warning.
    Warning,
    /// `·` dry run: not simulated; also the separator dot.
    Dot,
    /// `◌` background work.
    Background,
    /// `◐◓◑◒` the console's live line, frame by frame.
    Spin(u8),
    /// `+` WriteFile.
    Write,
    /// `±` ApplyPatch.
    Patch,
    /// `≈` ReplaceExact.
    Replace,
    /// `−` Delete.
    Delete,
    /// `∗` Generated.
    Generated,
    /// `$` RunScript.
    Script,
    /// `⊙` Commit.
    Commit,
    /// `¶` Note.
    Note,
    /// `⚑` Manual.
    Manual,
    /// `›` between crumbs.
    Crumb,
    /// `│` a vertical hairline.
    VRule,
    /// `─` a horizontal hairline.
    HRule,
    /// `▎` the focused panel's mark.
    Focus,
    /// `▌` the selected row's mark.
    Select,
    /// `▐` the dock's band.
    Band,
    /// `▾` unfolded.
    Open,
    /// `▸` folded.
    Closed,
    /// `⇣` the console follows.
    Follow,
    /// `←` points back.
    Left,
    /// `◂` an aligned output line.
    Aligned,
    /// `→` leads to.
    Right,
    /// `…` text cut short.
    Ellipsis,
    /// `↑` an update.
    Up,
    /// `█` coverage: action.
    CovAction,
    /// `▪` coverage: absorbed.
    CovAbsorbed,
    /// `▒` coverage: manifest.
    CovManifest,
    /// `░` coverage: note.
    CovNote,
    /// `⏎` enter after a typed word.
    Enter,
    /// `▏` the input caret.
    Caret,
}

impl Glyph {
    /// Every glyph, for the width test.
    pub const ALL: [Glyph; 42] = [
        Glyph::Pending,
        Glyph::Running,
        Glyph::Done,
        Glyph::Failed,
        Glyph::Skipped,
        Glyph::Waiting,
        Glyph::Warning,
        Glyph::Dot,
        Glyph::Background,
        Glyph::Spin(0),
        Glyph::Spin(1),
        Glyph::Spin(2),
        Glyph::Spin(3),
        Glyph::Write,
        Glyph::Patch,
        Glyph::Replace,
        Glyph::Delete,
        Glyph::Generated,
        Glyph::Script,
        Glyph::Commit,
        Glyph::Note,
        Glyph::Manual,
        Glyph::Crumb,
        Glyph::VRule,
        Glyph::HRule,
        Glyph::Focus,
        Glyph::Select,
        Glyph::Band,
        Glyph::Open,
        Glyph::Closed,
        Glyph::Follow,
        Glyph::Left,
        Glyph::Aligned,
        Glyph::Right,
        Glyph::Ellipsis,
        Glyph::Up,
        Glyph::CovAction,
        Glyph::CovAbsorbed,
        Glyph::CovManifest,
        Glyph::CovNote,
        Glyph::Enter,
        Glyph::Caret,
    ];

    /// The glyph.
    pub fn unicode(self) -> &'static str {
        match self {
            Glyph::Pending => "○",
            Glyph::Running => "◉",
            Glyph::Done => "✓",
            Glyph::Failed => "✕",
            Glyph::Skipped => "⊘",
            Glyph::Waiting => "◆",
            Glyph::Warning => "!",
            Glyph::Dot => "·",
            Glyph::Background => "◌",
            Glyph::Spin(n) => ["◐", "◓", "◑", "◒"][usize::from(n % 4)],
            Glyph::Write => "+",
            Glyph::Patch => "±",
            Glyph::Replace => "≈",
            Glyph::Delete => "−",
            Glyph::Generated => "∗",
            Glyph::Script => "$",
            Glyph::Commit => "⊙",
            Glyph::Note => "¶",
            Glyph::Manual => "⚑",
            Glyph::Crumb => "›",
            Glyph::VRule => "│",
            Glyph::HRule => "─",
            Glyph::Focus => "▎",
            Glyph::Select => "▌",
            Glyph::Band => "▐",
            Glyph::Open => "▾",
            Glyph::Closed => "▸",
            Glyph::Follow => "⇣",
            Glyph::Left => "←",
            Glyph::Aligned => "◂",
            Glyph::Right => "→",
            Glyph::Ellipsis => "…",
            Glyph::Up => "↑",
            Glyph::CovAction => "█",
            Glyph::CovAbsorbed => "▪",
            Glyph::CovManifest => "▒",
            Glyph::CovNote => "░",
            Glyph::Enter => "⏎",
            Glyph::Caret => "▏",
        }
    }

    /// Its ASCII fallback.
    pub fn ascii(self) -> &'static str {
        match self {
            Glyph::Pending => "o",
            Glyph::Running => "*",
            Glyph::Done => "v",
            Glyph::Failed => "x",
            Glyph::Skipped | Glyph::Delete | Glyph::HRule => "-",
            Glyph::Waiting => "?",
            Glyph::Warning => "!",
            Glyph::Dot | Glyph::Note | Glyph::CovNote => ".",
            Glyph::Background | Glyph::Patch => "~",
            Glyph::Spin(n) => ["-", "\\", "|", "/"][usize::from(n % 4)],
            Glyph::Write => "+",
            Glyph::Replace | Glyph::CovAbsorbed => "=",
            Glyph::Generated => "*",
            Glyph::Script => "$",
            Glyph::Commit => "@",
            Glyph::Manual => "M",
            Glyph::Crumb | Glyph::Right | Glyph::Select | Glyph::Closed => ">",
            Glyph::VRule | Glyph::Focus | Glyph::Band | Glyph::Caret => "|",
            Glyph::Open | Glyph::Follow => "v",
            Glyph::Left | Glyph::Aligned | Glyph::Enter => "<",
            Glyph::Ellipsis => "~",
            Glyph::Up => "^",
            Glyph::CovAction => "#",
            Glyph::CovManifest => ":",
        }
    }
}

/// The glyph set in use: Unicode, or every fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Glyphs {
    ascii: bool,
}

impl Glyphs {
    /// The Unicode set.
    pub fn unicode() -> Glyphs {
        Glyphs { ascii: false }
    }

    /// The ASCII fallbacks.
    pub fn ascii() -> Glyphs {
        Glyphs { ascii: true }
    }

    /// `PLANRUN_ASCII=1` selects the fallbacks.
    pub fn from_env() -> Glyphs {
        Glyphs {
            ascii: std::env::var_os("PLANRUN_ASCII").is_some_and(|v| v == "1"),
        }
    }

    /// The text of `glyph`.
    pub fn get(self, glyph: Glyph) -> &'static str {
        if self.ascii {
            glyph.ascii()
        } else {
            glyph.unicode()
        }
    }
}

#[cfg(test)]
mod tests {
    use unicode_width::UnicodeWidthStr;

    use super::*;

    #[test]
    fn every_glyph_and_fallback_is_one_cell_wide() {
        for glyph in Glyph::ALL {
            assert_eq!(glyph.unicode().width(), 1, "{glyph:?} {}", glyph.unicode());
            assert_eq!(glyph.unicode().chars().count(), 1, "{glyph:?}");
            assert_eq!(glyph.ascii().len(), 1, "{glyph:?}");
            assert!(glyph.ascii().is_ascii(), "{glyph:?}");
        }
    }

    #[test]
    fn the_switch_selects_the_fallbacks() {
        assert_eq!(Glyphs::unicode().get(Glyph::Manual), "⚑");
        assert_eq!(Glyphs::ascii().get(Glyph::Manual), "M");
        assert_eq!(Glyphs::ascii().get(Glyph::Spin(5)), "\\");
    }
}
