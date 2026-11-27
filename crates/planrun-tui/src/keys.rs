//! Keys as the TUI reads them (docs/tui-design.md §7, decision D94). crossterm reports a press
//! and a release on Windows: only presses count, and a held key's repeats only for movement.
//! Ctrl-C arrives as a key in raw mode; the dock confirms it (D91).

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// One key, normalised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// A character; shift is in the character itself (`R`).
    Char(char),
    /// Ctrl and a letter, lower case.
    Ctrl(char),
    /// Enter.
    Enter,
    /// Esc.
    Esc,
    /// Tab.
    Tab,
    /// Shift-Tab.
    BackTab,
    /// Backspace.
    Backspace,
    /// Up.
    Up,
    /// Down.
    Down,
    /// Left.
    Left,
    /// Right.
    Right,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Home.
    Home,
    /// End.
    End,
    /// A function key.
    F(u8),
}

impl Key {
    /// Whether a held key's repeats count: movement only.
    fn repeats(self) -> bool {
        matches!(
            self,
            Key::Up | Key::Down | Key::Left | Key::Right | Key::PageUp | Key::PageDown
        )
    }
}

/// The key `event` stands for; `None` for a release, a repeat of a key that does not move, and
/// keys the TUI does not use.
pub fn key(event: &KeyEvent) -> Option<Key> {
    let key = match event.code {
        KeyCode::Char(c) if event.modifiers.contains(KeyModifiers::CONTROL) => {
            Key::Ctrl(c.to_ascii_lowercase())
        }
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Esc,
        KeyCode::Tab if event.modifiers.contains(KeyModifiers::SHIFT) => Key::BackTab,
        KeyCode::Tab => Key::Tab,
        KeyCode::BackTab => Key::BackTab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::F(n) => Key::F(n),
        _ => return None,
    };
    match event.kind {
        KeyEventKind::Press => Some(key),
        KeyEventKind::Repeat if key.repeats() => Some(key),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::KeyEventState;

    use super::*;

    fn event(code: KeyCode, modifiers: KeyModifiers, kind: KeyEventKind) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind,
            state: KeyEventState::NONE,
        }
    }

    #[rustfmt::skip]
    #[test]
    fn presses_count_releases_never_and_repeats_only_for_movement() {
        let none = KeyModifiers::NONE;
        let (press, repeat, release) = (KeyEventKind::Press, KeyEventKind::Repeat, KeyEventKind::Release);
        assert_eq!(key(&event(KeyCode::Char('R'), KeyModifiers::SHIFT, press)), Some(Key::Char('R')));
        assert_eq!(key(&event(KeyCode::Char('C'), KeyModifiers::CONTROL, press)), Some(Key::Ctrl('c')));
        assert_eq!(key(&event(KeyCode::Char('y'), none, release)), None);
        assert_eq!(key(&event(KeyCode::Char('y'), none, repeat)), None);
        assert_eq!(key(&event(KeyCode::Down, none, repeat)), Some(Key::Down));
        assert_eq!(key(&event(KeyCode::Tab, KeyModifiers::SHIFT, press)), Some(Key::BackTab));
        assert_eq!(key(&event(KeyCode::F(2), none, press)), Some(Key::F(2)));
        assert_eq!(key(&event(KeyCode::Insert, none, press)), None);
    }
}
