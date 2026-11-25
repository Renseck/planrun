//! Matches a block's run against its expectation (docs/architecture.md §6.4; decisions D11,
//! D61, D66). Only exit codes and fenced summary lines pass a script on their own; prose and
//! comment expectations, an exit the plan did not state that is not 0, and a recorded pwsh error
//! all ask for confirmation. Skipping an expected failure is decided before the run (D67), never
//! here.

use std::borrow::Cow;

use planrun_model::{ExitExpect, Expectation, PlanSource};

use crate::shell::{RunStatus, ScriptRun};

/// What the matcher concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Everything stated was checked and holds; nothing asks.
    Pass,
    /// Something checked does not hold.
    Mismatch(Vec<Mismatch>),
    /// The checked parts hold, and the user confirms the rest.
    Confirm(Confirm),
}

/// Why a run asks for confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Confirm {
    /// The reasons, in the order prose, comments, unchecked exit, recorded error.
    pub reasons: Vec<Reason>,
    /// The run's `errored` flag, shown with every confirmation.
    pub errored: bool,
}

/// One reason to confirm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// The plan says what to expect in a sentence (D11).
    Prose,
    /// The script's trailing comments say it (D35, D48).
    Comments,
    /// The plan states no exit status, and the block did not exit 0 (it failed or threw).
    UncheckedExit,
    /// pwsh recorded an error, and nothing but an exit status (or nothing) was expected.
    Errored,
}

/// One thing that does not hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mismatch {
    /// The exit status. `expected` is `None` when the plan states none: a session that ended,
    /// timed out or broke is a mismatch whatever was expected.
    Exit {
        /// What the plan expects.
        expected: Option<ExitExpect>,
        /// What happened.
        got: RunStatus,
    },
    /// A summary line no output line matches, in order after the ones before it.
    Line {
        /// Its 1-based line number within the expected block.
        number: usize,
        /// The expected line, as written.
        line: String,
    },
}

/// Checks `run` against `expect` (`None`: the plan expects nothing in particular).
pub fn check(expect: Option<&Expectation>, run: &ScriptRun, source: &PlanSource) -> Verdict {
    let mut mismatches = Vec::new();
    let ended = matches!(
        run.status,
        RunStatus::SessionEnded(_) | RunStatus::TimedOut | RunStatus::Broken(_)
    );
    let exit = expect.map_or(ExitExpect::Unknown, |e| e.exit);
    let exit_holds = match exit {
        ExitExpect::Success => run.status == RunStatus::Exited(0),
        ExitExpect::Failure => {
            matches!(run.status, RunStatus::Exited(n) if n != 0) || run.status == RunStatus::Threw
        }
        ExitExpect::Code(code) => run.status == RunStatus::Exited(code),
        ExitExpect::Unknown => !ended,
    };
    if !exit_holds {
        mismatches.push(Mismatch::Exit {
            expected: expect.map(|e| e.exit),
            got: run.status.clone(),
        });
    }
    if let Some(lines) = expect.and_then(|e| e.lines.as_ref()) {
        mismatches.extend(unmatched(source.resolve(lines), run));
    }
    if !mismatches.is_empty() {
        return Verdict::Mismatch(mismatches);
    }
    let reasons = reasons(expect, exit, run);
    if reasons.is_empty() {
        return Verdict::Pass;
    }
    Verdict::Confirm(Confirm {
        reasons,
        errored: run.errored,
    })
}

fn reasons(expect: Option<&Expectation>, exit: ExitExpect, run: &ScriptRun) -> Vec<Reason> {
    let mut reasons = Vec::new();
    let (lines, prose, comments) = expect.map_or((false, false, false), |e| {
        (e.lines.is_some(), e.prose.is_some(), e.comments.is_some())
    });
    if prose {
        reasons.push(Reason::Prose);
    }
    if comments {
        reasons.push(Reason::Comments);
    }
    if exit == ExitExpect::Unknown && run.status != RunStatus::Exited(0) {
        reasons.push(Reason::UncheckedExit);
    }
    let exit_only = !lines && !prose && !comments;
    let plain = matches!(exit, ExitExpect::Success | ExitExpect::Unknown);
    if run.errored && exit_only && plain {
        reasons.push(Reason::Errored);
    }
    reasons
}

/// The expected lines that do not match, as a subsequence of the output: each non-blank line
/// must match an output line after the one the line before it matched. The earliest match is
/// taken for each, which finds a subsequence whenever one exists.
fn unmatched(expected: &str, run: &ScriptRun) -> Vec<Mismatch> {
    let output: Vec<Cow<'_, str>> = run.lines.iter().map(|l| strip_ansi(&l.text)).collect();
    let mut next = 0;
    let mut missing = Vec::new();
    for (i, line) in expected.split('\n').enumerate() {
        let pattern = line.trim_end();
        if pattern.is_empty() {
            continue;
        }
        let found = output[next..]
            .iter()
            .position(|out| line_matches(pattern, out.trim_end()));
        match found {
            Some(at) => next += at + 1,
            None => missing.push(Mismatch::Line {
                number: i + 1,
                line: line.to_owned(),
            }),
        }
    }
    missing
}

/// Whether `line` matches `pattern`: equal, except that each `…` matches any run of characters,
/// the empty one included. The first literal segment is anchored at the start, the last at the
/// end; the ones between are found in order, each at its earliest place.
pub fn line_matches(pattern: &str, line: &str) -> bool {
    let segments: Vec<&str> = pattern.split('…').collect();
    let [first, middle @ .., last] = segments.as_slice() else {
        return pattern == line;
    };
    if first.len() + last.len() > line.len() || !line.starts_with(first) || !line.ends_with(last) {
        return false;
    }
    let mut rest = &line[first.len()..line.len() - last.len()];
    for segment in middle {
        let Some(at) = rest.find(segment) else {
            return false;
        };
        rest = &rest[at + segment.len()..];
    }
    true
}

/// `text` without ANSI CSI (`ESC [ … final`) and OSC (`ESC ] … BEL` or `ESC ] … ESC \`)
/// sequences. An unterminated sequence is dropped to the end of the text.
pub fn strip_ansi(text: &str) -> Cow<'_, str> {
    if !text.contains('\u{1b}') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            Some('[') => {
                chars.next();
                for c in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&c) {
                        break;
                    }
                }
            }
            Some(']') => {
                chars.next();
                while let Some(c) = chars.next() {
                    if c == '\u{7}' || (c == '\u{1b}' && chars.next_if_eq(&'\\').is_some()) {
                        break;
                    }
                }
            }
            _ => out.push(c),
        }
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use planrun_model::{Span, TextRef};

    use super::*;
    use crate::shell::{OutLine, Stream};

    #[test]
    fn line_patterns() {
        #[rustfmt::skip]
        let rows = [
            ("abc", "abc", true),
            ("abc", "abcd", false),
            ("abc", " abc", false),
            ("a…b", "ab", true),
            ("a…b", "a-b", true),
            ("a…b", "a", false),
            ("a…b", "b", false),
            ("a…a", "a", false),
            ("a…a", "aa", true),
            ("…ab…ab", "abab", true),
            ("…ab…ab", "ab", false),
            ("…ab…ab", "xabyab", true),
            ("…", "", true),
            ("…", "anything", true),
            ("a……b", "ab", true),
            ("test result: ok. … passed; 0 failed; …", "test result: ok. 3 passed; 0 failed; 0 ignored", true),
            ("test result: ok. … passed; 0 failed; …", "test result: FAILED. 3 passed; 1 failed; 0 ignored", false),
            ("a...b", "a-b", false),
            ("a...b", "a...b", true),
            ("René…✓", "René sees ✓", true),
        ];
        for (pattern, line, expected) in rows {
            assert_eq!(
                line_matches(pattern, line),
                expected,
                "{pattern:?} against {line:?}"
            );
        }
    }

    #[test]
    fn ansi_sequences() {
        #[rustfmt::skip]
        let rows = [
            ("plain", "plain"),
            ("\u{1b}[32mok\u{1b}[0m", "ok"),
            ("\u{1b}[1;31merror\u{1b}[0m: x", "error: x"),
            ("\u{1b}]0;title\u{7}text", "text"),
            ("\u{1b}]8;;http://x\u{1b}\\link\u{1b}]8;;\u{1b}\\", "link"),
            ("a\u{1b}[", "a"),
            ("a\u{1b}x", "a\u{1b}x"),
        ];
        for (text, expected) in rows {
            assert_eq!(strip_ansi(text), expected, "{text:?}");
        }
    }

    /// The shape of an expectation; its texts are made up by `verdict`.
    struct Case {
        exit: ExitExpect,
        lines: Option<&'static str>,
        prose: bool,
        comments: bool,
    }

    /// One row: the expectation, the run's status, `errored`, its output, the verdict.
    type Row = (
        Option<Case>,
        RunStatus,
        bool,
        &'static [&'static str],
        &'static str,
    );

    fn verdict(case: Option<Case>, status: RunStatus, errored: bool, output: &[&str]) -> String {
        let mut text = String::new();
        let mut piece = |s: &str| {
            let start = text.len() as u32;
            text.push_str(s);
            TextRef::Span(Span::new(start, text.len() as u32))
        };
        let expect = case.map(|c| Expectation {
            exit: c.exit,
            lines: c.lines.map(&mut piece),
            prose: c.prose.then(|| piece("Expected: all pass.")),
            comments: c
                .comments
                .then(|| piece("git branch --show-current   # main")),
        });
        let source = PlanSource::from_normalized("t.md", text, String::new());
        let line = |l: &&str| OutLine {
            stream: Stream::Stdout,
            text: (*l).to_owned(),
            newline: true,
        };
        let run = ScriptRun {
            status,
            errored,
            lines: output.iter().map(line).collect(),
            cwd: None,
        };
        match check(expect.as_ref(), &run, &source) {
            Verdict::Pass => "pass".to_owned(),
            Verdict::Mismatch(m) => format!("mismatch {m:?}"),
            Verdict::Confirm(c) => format!("confirm {:?} errored={}", c.reasons, c.errored),
        }
    }

    #[test]
    fn verdicts() {
        use ExitExpect::{Code, Failure, Success, Unknown};
        use RunStatus::{Broken, Exited, SessionEnded, Threw, TimedOut};
        let exit = |exit| {
            Some(Case {
                exit,
                lines: None,
                prose: false,
                comments: false,
            })
        };
        let lines = |exit, lines| {
            Some(Case {
                exit,
                lines: Some(lines),
                prose: false,
                comments: false,
            })
        };
        let prose = |exit| {
            Some(Case {
                exit,
                lines: None,
                prose: true,
                comments: false,
            })
        };
        let comments = |prose| {
            Some(Case {
                exit: Unknown,
                lines: None,
                prose,
                comments: true,
            })
        };
        let summary = "ok: …\n\n   \ndone\r\n";
        #[rustfmt::skip]
        let rows: Vec<Row> = vec![
            (None, Exited(0), false, &[], "pass"),
            (None, Exited(1), false, &[], "confirm [UncheckedExit] errored=false"),
            (None, Threw, true, &[], "confirm [UncheckedExit, Errored] errored=true"),
            (None, Exited(0), true, &[], "confirm [Errored] errored=true"),
            (None, TimedOut, false, &[], "mismatch [Exit { expected: None, got: TimedOut }]"),
            (exit(Success), Exited(0), false, &[], "pass"),
            (exit(Success), Exited(0), true, &[], "confirm [Errored] errored=true"),
            (exit(Success), Exited(2), false, &[], "mismatch [Exit { expected: Some(Success), got: Exited(2) }]"),
            (exit(Success), Threw, false, &[], "mismatch [Exit { expected: Some(Success), got: Threw }]"),
            (exit(Failure), Exited(101), false, &[], "pass"),
            (exit(Failure), Threw, true, &[], "pass"),
            (exit(Failure), Exited(0), false, &[], "mismatch [Exit { expected: Some(Failure), got: Exited(0) }]"),
            (exit(Failure), SessionEnded(Some(1)), false, &[], "mismatch [Exit { expected: Some(Failure), got: SessionEnded(Some(1)) }]"),
            (exit(Code(1)), Exited(1), false, &[], "pass"),
            (exit(Code(1)), Exited(0), false, &[], "mismatch [Exit { expected: Some(Code(1)), got: Exited(0) }]"),
            (lines(Success, summary), Exited(0), true, &["x", "ok: 3", "noise", "done   "], "pass"),
            (lines(Success, summary), Exited(0), false, &["\u{1b}[32mok: 3\u{1b}[0m", "done"], "pass"),
            (lines(Success, summary), Exited(0), false, &["done", "ok: 3"], "mismatch [Line { number: 4, line: \"done\\r\" }]"),
            (lines(Code(2), summary), Exited(1), false, &["ok: 3", "done"], "mismatch [Exit { expected: Some(Code(2)), got: Exited(1) }]"),
            (lines(Unknown, summary), Exited(1), false, &["ok: 3", "done"], "confirm [UncheckedExit] errored=false"),
            (prose(Success), Exited(0), true, &[], "confirm [Prose] errored=true"),
            (prose(Success), Exited(1), false, &[], "mismatch [Exit { expected: Some(Success), got: Exited(1) }]"),
            (prose(Unknown), Exited(1), false, &[], "confirm [Prose, UncheckedExit] errored=false"),
            (comments(false), Exited(0), false, &[], "confirm [Comments] errored=false"),
            (comments(true), Exited(0), false, &[], "confirm [Prose, Comments] errored=false"),
            (comments(false), Broken("x".into()), false, &[], "mismatch [Exit { expected: Some(Unknown), got: Broken(\"x\") }]"),
        ];
        for (case, status, errored, output, expected) in rows {
            let shown = format!("{status:?} errored={errored} {output:?}");
            assert_eq!(verdict(case, status, errored, output), expected, "{shown}");
        }
    }
}
