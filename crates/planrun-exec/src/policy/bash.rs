//! The bash splitter (decision D68). It walks the script once and recurses into `( … )`
//! subshells, `$( … )` and backticks (also inside double quotes and unquoted heredocs), and
//! `<( … )`/`>( … )`. Single quotes and quoted heredocs are data. `case`, `${…}` with a command
//! inside, an unterminated quote or heredoc, and an unmatched `)` are errors, which make the
//! whole script Destructive.

use super::{Acc, PolicyClass, Word, heads, redirect_target};

/// Reserved words that only introduce or close a compound command: what follows them is a
/// command of its own.
#[rustfmt::skip]
const LEADING: &[&str] = &[
    "!", "do", "done", "elif", "else", "esac", "fi", "if", "then", "time", "until", "while",
    "{", "}",
];

type Parsed = Result<(), String>;

/// Classifies a bash script into `acc`.
pub(super) fn classify(script: &str, acc: &mut Acc) -> Parsed {
    let chars: Vec<char> = script.chars().collect();
    let mut parser = Parser {
        c: &chars,
        i: 0,
        acc,
        heredocs: Vec::new(),
    };
    parser.statements(None)?;
    if !parser.heredocs.is_empty() {
        return Err("a heredoc without its body".to_owned());
    }
    Ok(())
}

/// A heredoc whose body starts after the current line.
struct Heredoc {
    delimiter: String,
    strip_tabs: bool,
    expand: bool,
}

struct Parser<'a> {
    c: &'a [char],
    i: usize,
    acc: &'a mut Acc,
    heredocs: Vec<Heredoc>,
}

fn is_name(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl Parser<'_> {
    fn peek(&self, k: usize) -> Option<char> {
        self.c.get(self.i + k).copied()
    }

    fn statements(&mut self, close: Option<char>) -> Parsed {
        let mut words: Vec<Word> = Vec::new();
        loop {
            self.blank();
            let Some(c) = self.peek(0) else {
                self.command(std::mem::take(&mut words));
                return match close {
                    Some(close) => Err(format!("`{close}` is missing")),
                    None => Ok(()),
                };
            };
            if Some(c) == close {
                self.command(std::mem::take(&mut words));
                self.i += 1;
                return Ok(());
            }
            let next = self.peek(1);
            match c {
                ')' => return Err("an unmatched `)`".to_owned()),
                '\n' => {
                    self.command(std::mem::take(&mut words));
                    self.i += 1;
                    self.heredoc_bodies()?;
                }
                ';' if next == Some(';') => return Err("`;;` (`case`) is not supported".to_owned()),
                '&' if next == Some('>') => self.redirect()?,
                ';' | '&' | '|' => {
                    self.command(std::mem::take(&mut words));
                    self.i += if matches!(next, Some('&' | '|')) && c != ';' {
                        2
                    } else {
                        1
                    };
                }
                '#' => {
                    while self.peek(0).is_some_and(|c| c != '\n') {
                        self.i += 1;
                    }
                }
                '\\' if next == Some('\n') => self.i += 2,
                '(' if next == Some('(') && words.is_empty() => self.arithmetic()?,
                '(' if words.is_empty() => {
                    self.i += 1;
                    self.statements(Some(')'))?;
                }
                '(' if next == Some(')') => {
                    // `name() { … }`: a function definition; its body follows as commands.
                    words.clear();
                    self.i += 2;
                }
                '(' => return Err("an unexpected `(`".to_owned()),
                '<' | '>' => self.redirect()?,
                _ if self.fd_redirect() => self.redirect()?,
                _ => {
                    let word = self.word()?;
                    words.push(word);
                }
            }
        }
    }

    fn blank(&mut self) {
        while matches!(self.peek(0), Some(' ' | '\t' | '\r')) {
            self.i += 1;
        }
    }

    /// `2>`, `10<`: digits right before a redirection operator.
    fn fd_redirect(&self) -> bool {
        let mut k = 0;
        while self.peek(k).is_some_and(|c| c.is_ascii_digit()) {
            k += 1;
        }
        k > 0 && matches!(self.peek(k), Some('<' | '>'))
    }

    /// `(( … ))` as a command: arithmetic, Read.
    fn arithmetic(&mut self) -> Parsed {
        self.i += 2;
        let mut depth = 0;
        while let Some(c) = self.peek(0) {
            self.i += 1;
            match c {
                '(' => depth += 1,
                ')' if depth > 0 => depth -= 1,
                ')' if self.peek(0) == Some(')') => {
                    self.i += 1;
                    return Ok(());
                }
                _ => {}
            }
        }
        Err("an unterminated `((`".to_owned())
    }

    fn redirect(&mut self) -> Parsed {
        while self.peek(0).is_some_and(|c| c.is_ascii_digit() || c == '&') {
            self.i += 1;
        }
        let op = self.peek(0);
        let next = self.peek(1);
        self.i += 1;
        if next == Some('(') {
            self.i += 1;
            return self.statements(Some(')'));
        }
        if op == Some('<') {
            return self.input(next);
        }
        let mut dup = false;
        match next {
            Some('>' | '|') => self.i += 1,
            Some('&') => {
                self.i += 1;
                dup = true;
            }
            _ => {}
        }
        self.blank();
        let target = self.word()?;
        if target.text.is_empty() {
            return Err("a redirection without a target".to_owned());
        }
        let fd = target.literal && target.text.chars().all(|c| c.is_ascii_digit() || c == '-');
        if dup && fd {
            return Ok(());
        }
        redirect_target(&target, self.acc);
        Ok(())
    }

    /// `<`, `<>`, `<&n`, `<<<`, and heredocs (`<<`, `<<-`).
    fn input(&mut self, next: Option<char>) -> Parsed {
        let heredoc = next == Some('<') && self.peek(1) != Some('<');
        match next {
            Some('<') if !heredoc => self.i += 2,
            Some('<' | '&') => self.i += 1,
            Some('>') => {
                self.i += 1;
                self.blank();
                let target = self.word()?;
                redirect_target(&target, self.acc);
                return Ok(());
            }
            _ => {}
        }
        let strip_tabs = heredoc && self.peek(0) == Some('-');
        if strip_tabs {
            self.i += 1;
        }
        self.blank();
        let word = self.word()?;
        if word.text.is_empty() {
            return Err("a redirection without a target".to_owned());
        }
        if heredoc {
            self.heredocs.push(Heredoc {
                delimiter: word.text,
                strip_tabs,
                expand: !word.quoted,
            });
        }
        Ok(())
    }

    /// The bodies of the heredocs opened on the line that just ended.
    fn heredoc_bodies(&mut self) -> Parsed {
        for doc in std::mem::take(&mut self.heredocs) {
            let mut closed = false;
            while self.i < self.c.len() {
                let end = self.c[self.i..]
                    .iter()
                    .position(|&c| c == '\n')
                    .map_or(self.c.len(), |k| self.i + k);
                let line: String = self.c[self.i..end].iter().collect();
                let start = self.i;
                self.i = (end + 1).min(self.c.len());
                let text = if doc.strip_tabs {
                    line.trim_start_matches('\t')
                } else {
                    &line
                };
                if text.trim_end_matches('\r') == doc.delimiter {
                    closed = true;
                    break;
                }
                if doc.expand {
                    self.expansions(start, end)?;
                }
            }
            if !closed {
                return Err(format!("an unterminated heredoc `{}`", doc.delimiter));
            }
        }
        Ok(())
    }

    /// The `$( … )` and backticks in the characters `start..end` of an expanding heredoc.
    fn expansions(&mut self, start: usize, end: usize) -> Parsed {
        let resume = self.i;
        self.i = start;
        let mut scratch = Word::default();
        while self.i < end {
            match (self.peek(0), self.peek(1)) {
                (Some('\\'), _) => self.i += 2,
                (Some('$'), _) => self.dollar(&mut scratch)?,
                (Some('`'), _) => self.backtick(&mut scratch)?,
                _ => self.i += 1,
            }
        }
        self.i = resume;
        Ok(())
    }

    fn word(&mut self) -> Result<Word, String> {
        let mut w = Word {
            literal: true,
            ..Word::default()
        };
        while let Some(c) = self.peek(0) {
            match c {
                ' ' | '\t' | '\r' | '\n' | ';' | '&' | '|' | ')' | '<' | '>' => break,
                '(' if w.text.ends_with('=') => self.array(&mut w)?,
                '(' => break,
                '\'' => {
                    self.i += 1;
                    self.single(&mut w)?;
                }
                '"' => {
                    self.i += 1;
                    self.double(&mut w)?;
                }
                '$' if self.peek(1) == Some('\'') => {
                    self.i += 2;
                    self.ansi_c(&mut w)?;
                }
                '$' if self.peek(1) == Some('"') => {
                    self.i += 2;
                    self.double(&mut w)?;
                }
                '$' => self.dollar(&mut w)?,
                '`' => self.backtick(&mut w)?,
                '\\' => {
                    self.i += 1;
                    match self.peek(0) {
                        Some('\n') => self.i += 1,
                        Some(c) => {
                            w.text.push(c);
                            w.quoted = true;
                            self.i += 1;
                        }
                        None => {}
                    }
                }
                _ => {
                    w.text.push(c);
                    self.i += 1;
                }
            }
        }
        Ok(w)
    }

    /// `name=( … )`: the words are data, their expansions are not.
    fn array(&mut self, w: &mut Word) -> Parsed {
        self.i += 1;
        loop {
            while matches!(self.peek(0), Some(' ' | '\t' | '\r' | '\n')) {
                self.i += 1;
            }
            match self.peek(0) {
                None => return Err("an unterminated array".to_owned()),
                Some(')') => {
                    self.i += 1;
                    w.literal = false;
                    w.text.push_str("(…)");
                    return Ok(());
                }
                Some(_) => {
                    let element = self.word()?;
                    if element.text.is_empty() {
                        return Err("an unexpected character in an array".to_owned());
                    }
                }
            }
        }
    }

    fn single(&mut self, w: &mut Word) -> Parsed {
        w.quoted = true;
        while let Some(c) = self.peek(0) {
            self.i += 1;
            if c == '\'' {
                return Ok(());
            }
            w.text.push(c);
        }
        Err("an unterminated single-quoted string".to_owned())
    }

    fn ansi_c(&mut self, w: &mut Word) -> Parsed {
        w.quoted = true;
        while let Some(c) = self.peek(0) {
            self.i += 1;
            match c {
                '\\' => self.i += 1,
                '\'' => return Ok(()),
                _ => w.text.push(c),
            }
        }
        Err("an unterminated $'…' string".to_owned())
    }

    fn double(&mut self, w: &mut Word) -> Parsed {
        w.quoted = true;
        while let Some(c) = self.peek(0) {
            match c {
                '"' => {
                    self.i += 1;
                    return Ok(());
                }
                '\\' => {
                    w.text.extend(self.peek(1));
                    self.i += 2;
                }
                '$' => self.dollar(w)?,
                '`' => self.backtick(w)?,
                _ => {
                    w.text.push(c);
                    self.i += 1;
                }
            }
        }
        Err("an unterminated double-quoted string".to_owned())
    }

    /// `$( … )`, `$(( … ))`, `${…}`, `$name`, `$1`, `$?`, or a lone `$`.
    fn dollar(&mut self, w: &mut Word) -> Parsed {
        match self.peek(1) {
            Some('(') if self.peek(2) == Some('(') => {
                self.i += 1;
                self.arithmetic()?;
                dynamic(w, "$((…))");
            }
            Some('(') => {
                self.i += 2;
                self.statements(Some(')'))?;
                dynamic(w, "$(…)");
            }
            Some('{') => {
                self.i += 2;
                let mut depth = 1;
                while let Some(c) = self.peek(0) {
                    self.i += 1;
                    match c {
                        '{' => depth += 1,
                        '}' if depth == 1 => {
                            dynamic(w, "${…}");
                            return Ok(());
                        }
                        '}' => depth -= 1,
                        '$' if self.peek(0) == Some('(') => {
                            return Err("a command inside `${…}`".to_owned());
                        }
                        '`' => return Err("a command inside `${…}`".to_owned()),
                        _ => {}
                    }
                }
                return Err("an unterminated `${`".to_owned());
            }
            Some('?' | '#' | '@' | '*' | '!' | '$' | '-') => {
                self.i += 2;
                dynamic(w, "$?");
            }
            next if is_name(next) => {
                self.i += 1;
                dynamic(w, "$");
                while is_name(self.peek(0)) {
                    w.text.extend(self.peek(0));
                    self.i += 1;
                }
            }
            _ => {
                w.text.push('$');
                self.i += 1;
            }
        }
        Ok(())
    }

    /// `` `…` ``: its text is a script of its own.
    fn backtick(&mut self, w: &mut Word) -> Parsed {
        self.i += 1;
        let mut inner = String::new();
        while let Some(c) = self.peek(0) {
            self.i += 1;
            match c {
                '`' => {
                    classify(&inner, self.acc)?;
                    dynamic(w, "`…`");
                    return Ok(());
                }
                '\\' if matches!(self.peek(0), Some('`' | '\\' | '$')) => {
                    inner.extend(self.peek(0));
                    self.i += 1;
                }
                _ => inner.push(c),
            }
        }
        Err("an unterminated backtick".to_owned())
    }

    fn command(&mut self, words: Vec<Word>) {
        let lead = words
            .iter()
            .take_while(|w| w.bare().is_some_and(|t| LEADING.contains(&t)))
            .count();
        let words = &words[lead..];
        let Some(head) = words.first() else {
            return;
        };
        match head.bare() {
            Some("for" | "select" | "[[" | "in") => return,
            Some("function") => return self.command(words.iter().skip(2).cloned().collect()),
            Some("case") => {
                let reason = "`case` is not supported".to_owned();
                return self.acc.add(PolicyClass::Destructive, Some(reason));
            }
            _ => {}
        }
        let assignments = words.iter().take_while(|w| assignment(w)).count();
        let Some(head) = words.get(assignments) else {
            return;
        };
        let reason = if assignments > 0 {
            Some("an environment prefix on a command".to_owned())
        } else {
            match head.bare() {
                None => Some("a command whose name is not literal".to_owned()),
                Some(name) if name.contains('/') => {
                    Some(format!("a path-qualified command `{name}`"))
                }
                Some(_) => None,
            }
        };
        if let Some(reason) = reason {
            return self.acc.add(PolicyClass::Destructive, Some(reason));
        }
        let (class, reason) = heads::bash(&head.text, &words[assignments + 1..]);
        self.acc.add(class, reason);
    }
}

/// `NAME=…` or `NAME+=…` with an unquoted name.
fn assignment(w: &Word) -> bool {
    let Some(eq) = w.text.find('=') else {
        return false;
    };
    let name = w.text[..eq].strip_suffix('+').unwrap_or(&w.text[..eq]);
    !name.is_empty()
        && !name.starts_with(|c: char| c.is_ascii_digit())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn dynamic(w: &mut Word, what: &str) {
    w.literal = false;
    w.text.push_str(what);
}
