//! The pwsh splitter (decision D68). It walks the script once, statement by statement and
//! pipeline element by pipeline element, and recurses into every place code can hide: `( … )`,
//! `$( … )`, `@( … )`, `@{ … }`, script blocks, and `$( … )` inside double-quoted strings and
//! `@" … "@`. Single-quoted strings and `@' … '@` are data. Anything it cannot take apart with
//! confidence (an unterminated quote or here-string, an unmatched bracket) is an error, which
//! makes the whole script Destructive.

use super::{Acc, PolicyClass, Word, heads, redirect_target};

#[rustfmt::skip]
const KEYWORDS: &[&str] = &[
    "begin", "break", "catch", "class", "clean", "continue", "do", "dynamicparam", "else",
    "elseif", "end", "enum", "exit", "filter", "finally", "for", "foreach", "function", "if",
    "param", "process", "return", "switch", "throw", "trap", "try", "until", "while",
];

/// Types whose static methods are pure (lowercase, as written in `[ ]`).
#[rustfmt::skip]
const PURE_TYPES: &[&str] = &[
    "io.path", "system.io.path", "math", "system.math", "string", "system.string", "regex",
    "system.text.regularexpressions.regex",
];

/// Instance methods that are pure string methods (lowercase).
#[rustfmt::skip]
const PURE_METHODS: &[&str] = &[
    "contains", "endswith", "replace", "split", "startswith", "substring", "tolower",
    "tostring", "toupper", "trim",
];

type Parsed = Result<(), String>;

/// Classifies a pwsh script into `acc`.
pub(super) fn classify(script: &str, acc: &mut Acc) -> Parsed {
    let chars: Vec<char> = script.chars().collect();
    let mut parser = Parser {
        c: &chars,
        i: 0,
        acc,
    };
    parser.statements(None, false)
}

fn is_single(c: Option<char>) -> bool {
    matches!(c, Some('\'' | '‘' | '’' | '‚' | '‛'))
}

fn is_double(c: Option<char>) -> bool {
    matches!(c, Some('"' | '“' | '”' | '„'))
}

fn is_dash(c: char) -> bool {
    matches!(c, '-' | '–' | '—' | '―')
}

fn is_ident(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_alphanumeric() || c == '_')
}

fn is_assign(text: &str) -> bool {
    matches!(text, "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "??=")
}

struct Parser<'a> {
    c: &'a [char],
    i: usize,
    acc: &'a mut Acc,
}

impl Parser<'_> {
    fn peek(&self, k: usize) -> Option<char> {
        self.c.get(self.i + k).copied()
    }

    /// Statements up to `close` (consumed) or the end of the script. In a hashtable, each
    /// statement is `key = value` and only the value is classified.
    fn statements(&mut self, close: Option<char>, hashtable: bool) -> Parsed {
        let mut words: Vec<Word> = Vec::new();
        let mut first = true;
        loop {
            self.blank();
            let Some(c) = self.peek(0) else {
                self.command(std::mem::take(&mut words), first, hashtable);
                return match close {
                    Some(close) => Err(format!("`{close}` is missing")),
                    None => Ok(()),
                };
            };
            if Some(c) == close {
                self.command(std::mem::take(&mut words), first, hashtable);
                self.i += 1;
                return Ok(());
            }
            let next = self.peek(1);
            match c {
                ')' | '}' | ']' => return Err(format!("an unmatched `{c}`")),
                '\n' | ';' => self.end(&mut words, &mut first, true, 1, hashtable),
                '|' if next == Some('|') => self.end(&mut words, &mut first, true, 2, hashtable),
                '&' if next == Some('&') => self.end(&mut words, &mut first, true, 2, hashtable),
                '|' => self.end(&mut words, &mut first, false, 1, hashtable),
                '#' => self.line_comment(),
                '<' if next == Some('#') => self.block_comment()?,
                '<' => return Err("`<` is not a pwsh operator".to_owned()),
                '&' => {
                    words.push(Word {
                        text: "&".to_owned(),
                        literal: true,
                        quoted: false,
                    });
                    self.i += 1;
                }
                ',' => self.i += 1,
                '`' if next == Some('\n') => self.i += 2,
                '`' if next == Some('\r') && self.peek(2) == Some('\n') => self.i += 3,
                _ if self.redirect_start() => self.redirect()?,
                _ => {
                    let word = self.word()?;
                    words.push(word);
                }
            }
        }
    }

    /// Ends the current command; `statement` starts a new statement, otherwise the next
    /// pipeline element.
    fn end(
        &mut self,
        words: &mut Vec<Word>,
        first: &mut bool,
        statement: bool,
        skip: usize,
        hashtable: bool,
    ) {
        self.command(std::mem::take(words), *first, hashtable);
        *first = statement;
        self.i += skip;
    }

    fn blank(&mut self) {
        while matches!(self.peek(0), Some(' ' | '\t' | '\r' | '\u{c}')) {
            self.i += 1;
        }
    }

    fn line_comment(&mut self) {
        while self.peek(0).is_some_and(|c| c != '\n') {
            self.i += 1;
        }
    }

    fn block_comment(&mut self) -> Parsed {
        self.i += 2;
        while let Some(c) = self.peek(0) {
            self.i += 1;
            if c == '#' && self.peek(0) == Some('>') {
                self.i += 1;
                return Ok(());
            }
        }
        Err("an unterminated `<#` comment".to_owned())
    }

    fn redirect_start(&self) -> bool {
        let c = self.peek(0);
        c == Some('>')
            || (c.is_some_and(|c| c.is_ascii_digit() || c == '*') && self.peek(1) == Some('>'))
    }

    /// `>`, `>>`, `n>`, `*>`, and the merges `n>&1`, which have no file.
    fn redirect(&mut self) -> Parsed {
        if self.peek(0) != Some('>') {
            self.i += 1;
        }
        self.i += 1;
        if self.peek(0) == Some('>') {
            self.i += 1;
        }
        if self.peek(0) == Some('&') && self.peek(1).is_some_and(|c| c.is_ascii_digit()) {
            self.i += 2;
            return Ok(());
        }
        self.blank();
        let target = self.word()?;
        if target.text.is_empty() {
            return Err("a redirection without a target".to_owned());
        }
        redirect_target(&target, self.acc);
        Ok(())
    }

    /// One word: everything up to whitespace or an operator, its parts concatenated.
    fn word(&mut self) -> Result<Word, String> {
        let mut w = Word {
            literal: true,
            ..Word::default()
        };
        let start = self.i;
        let mut value = false;
        let mut expression = false;
        while let Some(c) = self.peek(0) {
            let at_start = self.i == start;
            let next = self.peek(1);
            match c {
                ' ' | '\t' | '\r' | '\n' | ';' | '|' | '&' | ')' | '}' | ']' | ',' => break,
                '=' if expression && !at_start => break,
                '+' | '-' | '*' | '/' | '%' if expression && next == Some('=') => break,
                _ if is_single(Some(c)) => {
                    self.single(&mut w)?;
                    value = true;
                }
                _ if is_double(Some(c)) => {
                    self.double(&mut w)?;
                    value = true;
                }
                '@' if is_single(next) || is_double(next) => {
                    self.here(&mut w)?;
                    value = true;
                }
                '@' if next == Some('(') => {
                    self.i += 2;
                    self.statements(Some(')'), false)?;
                    dynamic(&mut w, "@(…)");
                    value = true;
                }
                '@' if next == Some('{') => {
                    self.i += 2;
                    self.statements(Some('}'), true)?;
                    dynamic(&mut w, "@{…}");
                    value = true;
                }
                '@' if at_start && is_ident(next) => {
                    self.i += 1;
                    dynamic(&mut w, "@");
                    self.ident(&mut w);
                }
                '$' => {
                    self.dollar(&mut w)?;
                    value = true;
                    expression |= at_start;
                }
                '(' => {
                    self.i += 1;
                    self.statements(Some(')'), false)?;
                    dynamic(&mut w, "(…)");
                    value = true;
                    expression |= at_start;
                }
                '{' => {
                    self.i += 1;
                    self.statements(Some('}'), false)?;
                    dynamic(&mut w, "{…}");
                    value = false;
                }
                '[' if at_start => {
                    self.type_literal(&mut w)?;
                    value = true;
                    expression = true;
                }
                '[' if value => {
                    self.i += 1;
                    self.statements(Some(']'), false)?;
                    dynamic(&mut w, "[…]");
                }
                '.' if value && is_ident(next) => self.member(&mut w)?,
                '`' => {
                    self.i += 1;
                    if let Some(c) = self.peek(0).filter(|c| *c != '\n') {
                        w.text.push(c);
                        self.i += 1;
                    }
                    value = false;
                }
                _ => {
                    w.text.push(if at_start && is_dash(c) { '-' } else { c });
                    self.i += 1;
                    value = false;
                }
            }
        }
        Ok(w)
    }

    fn ident(&mut self, w: &mut Word) {
        while is_ident(self.peek(0)) || self.peek(0) == Some(':') {
            if let Some(c) = self.peek(0) {
                w.text.push(c);
            }
            self.i += 1;
        }
    }

    /// `'…'` with `''` for a quote; data.
    fn single(&mut self, w: &mut Word) -> Parsed {
        w.quoted = true;
        self.i += 1;
        while let Some(c) = self.peek(0) {
            self.i += 1;
            if !is_single(Some(c)) {
                w.text.push(c);
                continue;
            }
            if !is_single(self.peek(0)) {
                return Ok(());
            }
            w.text.push(c);
            self.i += 1;
        }
        Err("an unterminated single-quoted string".to_owned())
    }

    /// `"…"`: `` ` `` escapes, `""` for a quote, variables and `$( … )` inside.
    fn double(&mut self, w: &mut Word) -> Parsed {
        w.quoted = true;
        self.i += 1;
        while let Some(c) = self.peek(0) {
            match c {
                '`' => {
                    if let Some(next) = self.peek(1) {
                        w.text.push(next);
                    }
                    self.i += 2;
                }
                '$' if matches!(self.peek(1), Some('(' | '{')) || is_ident(self.peek(1)) => {
                    self.dollar(w)?;
                }
                _ if is_double(Some(c)) => {
                    self.i += 1;
                    if !is_double(self.peek(0)) {
                        return Ok(());
                    }
                    w.text.push(c);
                    self.i += 1;
                }
                _ => {
                    w.text.push(c);
                    self.i += 1;
                }
            }
        }
        Err("an unterminated double-quoted string".to_owned())
    }

    /// `@'…'@` (data) or `@"…"@` (with `$( … )`): the opener ends its line, the closer starts
    /// one.
    fn here(&mut self, w: &mut Word) -> Parsed {
        w.quoted = true;
        let expandable = is_double(self.peek(1));
        self.i += 2;
        self.blank();
        if self.peek(0) != Some('\n') {
            return Err("a here-string whose opener does not end its line".to_owned());
        }
        while let Some(c) = self.peek(0) {
            let closes = if expandable { is_double } else { is_single };
            if c == '\n' && closes(self.peek(1)) && self.peek(2) == Some('@') {
                self.i += 3;
                return Ok(());
            }
            if expandable && c == '$' && self.peek(1) == Some('(') {
                self.dollar(w)?;
                continue;
            }
            if expandable && c == '`' {
                self.i += 1;
            }
            self.i += 1;
        }
        Err("an unterminated here-string".to_owned())
    }

    /// `$( … )`, `${…}`, `$name`, `$scope:name`, or a lone `$`.
    fn dollar(&mut self, w: &mut Word) -> Parsed {
        match self.peek(1) {
            Some('(') => {
                self.i += 2;
                self.statements(Some(')'), false)?;
                dynamic(w, "$(…)");
            }
            Some('{') => {
                self.i += 2;
                while self.peek(0).is_some_and(|c| c != '}') {
                    self.i += 1;
                }
                if self.peek(0).is_none() {
                    return Err("an unterminated `${`".to_owned());
                }
                self.i += 1;
                dynamic(w, "${…}");
            }
            Some('?' | '^' | '$') => {
                self.i += 2;
                dynamic(w, "$?");
            }
            next if is_ident(next) => {
                self.i += 1;
                dynamic(w, "$");
                self.ident(w);
            }
            _ => {
                w.text.push('$');
                self.i += 1;
            }
        }
        Ok(())
    }

    /// `[Type]`, then `::Member` or `::Method( … )`: a static call is Destructive unless the
    /// type is pure.
    fn type_literal(&mut self, w: &mut Word) -> Parsed {
        self.i += 1;
        let mut depth = 1;
        let mut name = String::new();
        while let Some(c) = self.peek(0) {
            self.i += 1;
            match c {
                '[' => depth += 1,
                ']' if depth == 1 => {
                    depth = 0;
                    break;
                }
                ']' => depth -= 1,
                '\n' => break,
                _ => {}
            }
            name.push(c);
        }
        if depth != 0 {
            return Err("a `[` without its `]`".to_owned());
        }
        dynamic(w, "[…]");
        if !(self.peek(0) == Some(':') && self.peek(1) == Some(':')) {
            return Ok(());
        }
        self.i += 2;
        let mut member = String::new();
        while is_ident(self.peek(0)) {
            member.extend(self.peek(0));
            self.i += 1;
        }
        if self.peek(0) != Some('(') {
            return Ok(());
        }
        let name = name.trim().to_ascii_lowercase();
        if !PURE_TYPES.contains(&name.as_str()) {
            let reason = format!("a static call [{name}]::{member}(");
            self.acc.add(PolicyClass::Destructive, Some(reason));
        }
        self.i += 1;
        self.statements(Some(')'), false)
    }

    /// `.Member` or `.Method( … )` after a value: a call is Destructive unless it is a pure
    /// string method.
    fn member(&mut self, w: &mut Word) -> Parsed {
        self.i += 1;
        let mut name = String::new();
        while is_ident(self.peek(0)) {
            name.extend(self.peek(0));
            self.i += 1;
        }
        dynamic(w, ".");
        w.text.push_str(&name);
        if self.peek(0) != Some('(') {
            return Ok(());
        }
        if !PURE_METHODS.contains(&name.to_ascii_lowercase().as_str()) {
            let reason = format!("a method call .{name}(");
            self.acc.add(PolicyClass::Destructive, Some(reason));
        }
        self.i += 1;
        self.statements(Some(')'), false)
    }

    /// Classifies one command (or expression) of a pipeline; `first` when it starts a
    /// statement, where keywords count.
    fn command(&mut self, mut words: Vec<Word>, first: bool, hashtable: bool) {
        if hashtable && words.get(1).and_then(Word::bare) == Some("=") {
            words.drain(..2);
            return self.command(words, true, false);
        }
        let key = words.first().and_then(Word::bare).and_then(|t| t.find('='));
        if let (true, Some(eq)) = (hashtable, key) {
            let value = words[0].text.split_off(eq + 1);
            words[0].text = value;
            if words[0].text.is_empty() {
                words.remove(0);
            }
            return self.command(words, true, false);
        }
        let assigned = words
            .iter()
            .position(|w| w.bare().is_some_and(is_assign))
            .filter(|&k| k > 0 && words[..k].iter().all(|w| w.text.starts_with(['$', '['])));
        if let Some(k) = assigned {
            words.drain(..=k);
            return self.command(words, true, false);
        }
        let Some(head) = words.first() else {
            return;
        };
        let Some(name) = head.bare() else {
            return;
        };
        match name {
            "&" => self.call(&words[1..]),
            "." => {
                let reason = "dot-sourcing a file".to_owned();
                self.acc.add(PolicyClass::Destructive, Some(reason));
            }
            _ if name.starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '!') => {}
            _ => {
                let lower = name.to_lowercase();
                if !(first && KEYWORDS.contains(&lower.as_str())) {
                    return self.named(&lower, &words[1..]);
                }
                if matches!(lower.as_str(), "return" | "throw" | "exit") && words.len() > 1 {
                    words.remove(0);
                    self.command(words, true, false);
                }
            }
        }
    }

    /// `& name …`: classified as `name`, when it is a literal bare name.
    fn call(&mut self, words: &[Word]) {
        match words.first().and_then(Word::bare) {
            Some(name) if !name.is_empty() => self.named(&name.to_lowercase(), &words[1..]),
            _ => {
                let reason = "`&` with something other than a command name".to_owned();
                self.acc.add(PolicyClass::Destructive, Some(reason));
            }
        }
    }

    fn named(&mut self, name: &str, args: &[Word]) {
        if name.contains(['\\', '/']) {
            let reason = format!("a path-qualified command `{name}`");
            self.acc.add(PolicyClass::Destructive, Some(reason));
            return;
        }
        let name = name
            .strip_suffix(".exe")
            .or_else(|| name.strip_suffix(".cmd"))
            .unwrap_or(name);
        let (class, reason) = heads::pwsh(name, args);
        self.acc.add(class, reason);
    }
}

/// Marks `w` as not literal and appends `what` to its spelling.
fn dynamic(w: &mut Word, what: &str) {
    w.literal = false;
    w.text.push_str(what);
}
