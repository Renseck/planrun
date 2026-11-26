//! GNU sed's class (decision D72): a sed script can write files (`w`, `W`, the `s///w` flag) and
//! run commands (`e`, the `s///e` flag), and `-f` reads a script the classifier never sees. So
//! the script is read command by command: Read when it does neither, Modify when it writes (or
//! with `-i`), Destructive when it runs commands, comes from a file, or cannot be read with
//! confidence.

use super::PolicyClass::{Destructive, Modify, Read};
use super::Word;
use super::heads::Verdict;

/// What a script does besides printing.
#[derive(Debug, Default, PartialEq, Eq)]
struct Effects {
    writes: bool,
    runs: bool,
}

fn no(reason: impl Into<String>) -> Verdict {
    (Destructive, Some(reason.into()))
}

/// The class of `sed` with `args`.
pub(super) fn sed(args: &[Word]) -> Verdict {
    let mut scripts: Vec<&str> = Vec::new();
    let mut operands: Vec<&Word> = Vec::new();
    let mut in_place = false;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        i += 1;
        let Some(text) = arg.bare() else {
            operands.push(arg);
            continue;
        };
        if text == "--" {
            operands.extend(&args[i..]);
            break;
        }
        let option = if let Some(long) = text.strip_prefix("--") {
            long_option(long)
        } else if text.len() > 1 && text.starts_with('-') {
            short_cluster(&text[1..])
        } else {
            operands.push(arg);
            continue;
        };
        match option {
            Opt::Flag => {}
            Opt::InPlace => in_place = true,
            Opt::Script(Some(script)) => scripts.push(script),
            Opt::Script(None) | Opt::Number => {
                let Some(next) = args.get(i) else {
                    return no(format!("sed option `{text}` without its value"));
                };
                i += 1;
                if matches!(option, Opt::Script(None)) {
                    let Some(script) = next.literal.then_some(next.text.as_str()) else {
                        return no("a sed script with an expansion");
                    };
                    scripts.push(script);
                }
            }
            Opt::File => return no("sed -f"),
            Opt::Unknown => return no(format!("sed option `{text}`")),
        }
    }
    if scripts.is_empty() {
        let Some(first) = operands.first() else {
            return no("sed without a script");
        };
        let Some(script) = first.literal.then_some(first.text.as_str()) else {
            return no("a sed script with an expansion");
        };
        scripts.push(script);
    }
    let mut effects = Effects::default();
    for script in scripts {
        match read_script(script) {
            Ok(found) => {
                effects.writes |= found.writes;
                effects.runs |= found.runs;
            }
            Err(why) => return no(format!("a sed script it cannot read: {why}")),
        }
    }
    if effects.runs {
        return no("sed runs a command (`e`)");
    }
    if effects.writes || in_place {
        return (Modify, None);
    }
    (Read, None)
}

/// What one option is.
enum Opt<'a> {
    Flag,
    InPlace,
    /// `-e`/`--expression`: the script glued to it, or `None` for the next argument.
    Script(Option<&'a str>),
    /// `-l N`: the next argument is a number.
    Number,
    File,
    Unknown,
}

fn long_option(name: &str) -> Opt<'_> {
    let (name, value) = match name.split_once('=') {
        Some((n, v)) => (n, Some(v)),
        None => (name, None),
    };
    match (name, value) {
        ("in-place", _) => Opt::InPlace,
        ("expression", value) => Opt::Script(value),
        ("file", _) => Opt::File,
        ("line-length", Some(_)) => Opt::Flag,
        ("line-length", None) => Opt::Number,
        (
            "quiet" | "silent" | "regexp-extended" | "separate" | "unbuffered" | "null-data"
            | "posix" | "debug" | "sandbox" | "follow-symlinks",
            None,
        ) => Opt::Flag,
        _ => Opt::Unknown,
    }
}

/// A short-option cluster without its dash: `n`, `nE`, `i.bak`, `ne`, `es/a/b/`, `l80`.
fn short_cluster(cluster: &str) -> Opt<'_> {
    for (at, c) in cluster.char_indices() {
        let rest = &cluster[at + 1..];
        match c {
            'n' | 'E' | 'r' | 's' | 'u' | 'z' => {}
            // `-i` takes the rest of the cluster as its backup suffix.
            'i' => return Opt::InPlace,
            'e' if rest.is_empty() => return Opt::Script(None),
            'e' => return Opt::Script(Some(rest)),
            'l' if rest.is_empty() => return Opt::Number,
            'l' => return Opt::Flag,
            'f' => return Opt::File,
            _ => return Opt::Unknown,
        }
    }
    Opt::Flag
}

/// Reads a sed script command by command (GNU syntax). An error is anything it does not
/// recognise with confidence.
fn read_script(script: &str) -> Result<Effects, String> {
    let chars: Vec<char> = script.chars().collect();
    let mut p = Parser {
        chars: &chars,
        at: 0,
        effects: Effects::default(),
    };
    p.commands()?;
    Ok(p.effects)
}

struct Parser<'a> {
    chars: &'a [char],
    at: usize,
    effects: Effects,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        self.at += usize::from(c.is_some());
        c
    }

    fn skip_blanks(&mut self) {
        while self.peek().is_some_and(|c| c == ' ' || c == '\t') {
            self.at += 1;
        }
    }

    fn commands(&mut self) -> Result<(), String> {
        loop {
            while self
                .peek()
                .is_some_and(|c| c.is_whitespace() || c == ';' || c == '}')
            {
                self.at += 1;
            }
            if self.peek().is_none() {
                return Ok(());
            }
            self.address()?;
            self.skip_blanks();
            if self.peek() == Some(',') {
                self.at += 1;
                self.skip_blanks();
                self.address()?;
            }
            self.skip_blanks();
            while self.peek() == Some('!') {
                self.at += 1;
                self.skip_blanks();
            }
            self.command()?;
        }
    }

    /// One address: a line number (`3`, `0`, `1~2`, `+3`, `~4`), `$`, `/re/` or `\cREc`, each
    /// with optional `I` and `M` flags after a regex. No address is fine.
    fn address(&mut self) -> Result<(), String> {
        match self.peek() {
            Some(c) if c.is_ascii_digit() || c == '+' || c == '~' => {
                self.at += 1;
                while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '~') {
                    self.at += 1;
                }
            }
            Some('$') => self.at += 1,
            Some('/') => {
                self.at += 1;
                self.delimited('/')?;
                self.regex_flags();
            }
            Some('\\') => {
                self.at += 1;
                let delim = self.bump().ok_or("an address ends after `\\`")?;
                self.delimited(delim)?;
                self.regex_flags();
            }
            _ => {}
        }
        Ok(())
    }

    fn regex_flags(&mut self) {
        while self.peek().is_some_and(|c| c == 'I' || c == 'M') {
            self.at += 1;
        }
    }

    /// Up to the next `delim` that no backslash escapes; the delimiter is consumed. A newline
    /// inside is refused: GNU sed accepts some, and nothing here needs them.
    fn delimited(&mut self, delim: char) -> Result<(), String> {
        loop {
            match self.bump() {
                None | Some('\n') => return Err(format!("an unterminated `{delim}` part")),
                Some('\\') => {
                    self.bump().ok_or("a part ends after `\\`")?;
                }
                Some(c) if c == delim => return Ok(()),
                Some(_) => {}
            }
        }
    }

    /// The rest of the line, for a text, file name or label argument.
    fn rest_of_line(&mut self) {
        while self.peek().is_some_and(|c| c != '\n') {
            self.at += 1;
        }
    }

    /// The text of `a`, `i` and `c`: to the end of the line, and on through every line that
    /// ends in an unescaped backslash.
    fn text(&mut self) {
        loop {
            let mut escaped = false;
            while let Some(c) = self.peek() {
                if c == '\n' {
                    break;
                }
                escaped = c == '\\' && !escaped;
                self.at += 1;
            }
            if !escaped || self.bump().is_none() {
                return;
            }
        }
    }

    /// A label: up to `;` or the end of the line.
    fn label(&mut self) {
        while self.peek().is_some_and(|c| c != '\n' && c != ';') {
            self.at += 1;
        }
    }

    fn command(&mut self) -> Result<(), String> {
        let Some(c) = self.bump() else {
            return Err("an address without a command".to_owned());
        };
        match c {
            '{' => {}
            '#' | 'r' | 'R' => self.rest_of_line(),
            'a' | 'i' | 'c' => self.text(),
            'w' | 'W' => {
                self.effects.writes = true;
                self.rest_of_line();
            }
            'e' => {
                self.effects.runs = true;
                self.rest_of_line();
            }
            ':' | 'b' | 't' | 'T' | 'v' => self.label(),
            '=' | 'd' | 'D' | 'g' | 'G' | 'h' | 'H' | 'n' | 'N' | 'p' | 'P' | 'x' | 'z' | 'F' => {}
            'l' | 'L' | 'q' | 'Q' => {
                self.skip_blanks();
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.at += 1;
                }
            }
            's' => self.substitute()?,
            'y' => {
                let delim = self.delimiter()?;
                self.delimited(delim)?;
                self.delimited(delim)?;
            }
            other => return Err(format!("the command `{other}`")),
        }
        Ok(())
    }

    fn delimiter(&mut self) -> Result<char, String> {
        match self.bump() {
            Some(c) if c != '\n' && c != '\\' => Ok(c),
            _ => Err("a missing delimiter".to_owned()),
        }
    }

    /// `s/re/replacement/flags`: `w file` writes, `e` runs the pattern space as a command.
    fn substitute(&mut self) -> Result<(), String> {
        let delim = self.delimiter()?;
        self.delimited(delim)?;
        self.delimited(delim)?;
        while let Some(c) = self.peek() {
            match c {
                'g' | 'p' | 'i' | 'I' | 'm' | 'M' => self.at += 1,
                c if c.is_ascii_digit() => self.at += 1,
                'e' => {
                    self.effects.runs = true;
                    self.at += 1;
                }
                'w' => {
                    self.effects.writes = true;
                    self.rest_of_line();
                }
                _ => break,
            }
        }
        match self.peek() {
            None | Some(';' | '\n' | '}' | ' ' | '\t' | '#') => Ok(()),
            Some(other) => Err(format!("the `s` flag `{other}`")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn scripts() {
        let rows: &[(&str, Result<Effects, ()>)] = &[
            ("s/a/b/", Ok(Effects::default())),
            ("s/a/b/g; 1,5p; $d", Ok(Effects::default())),
            ("/start/,/end/{ s|x|y|2; p }", Ok(Effects::default())),
            ("s/test/\\/e/g", Ok(Effects::default())),
            (r"\,a,d", Ok(Effects::default())),
            ("y/abc/xyz/", Ok(Effects::default())),
            ("1i\\\nheader", Ok(Effects::default())),
            ("r other.txt", Ok(Effects::default())),
            ("s/a/b/w out.txt", Ok(Effects { writes: true, runs: false })),
            ("/x/w out.txt", Ok(Effects { writes: true, runs: false })),
            ("W out.txt", Ok(Effects { writes: true, runs: false })),
            ("s/.*/date/e", Ok(Effects { writes: false, runs: true })),
            ("1e date", Ok(Effects { writes: false, runs: true })),
            ("s/a/b", Err(())),
            ("s/a/b/k", Err(())),
            ("1,2", Err(())),
            ("K", Err(())),
        ];
        for (script, expected) in rows {
            let got = read_script(script).map_err(|_| ());
            assert_eq!(&got, expected, "{script:?}");
        }
    }
}
