//! `expected-block` (`Expected (…):` + a `text` fence of summary lines) and `expected-prose`
//! (`Expected: …`, or with `list = true` an `Expected:` paragraph and the list after it,
//! decision D47): both amend the script directly before them. An `expected-prose` after a
//! script whose only expectation is its comment lines extends it (D48).

use std::sync::Arc;

use planrun_model::{Action, ActionKind, ExitExpect, Expectation, RecognizerId, Span, TextRef};
use regex::Regex;

use super::params::{ExitRow, Params, exit_rows, first_exit};
use super::{amendable, capture_ref, paragraph};
use crate::diagnostic::Diagnostic;
use crate::document::{Block, Located};
use crate::recognizer::{Cursor, Recognized, Recognizer, RecognizerFactory, StepContext};

/// Builds `expected-block` recognizers.
pub struct ExpectedBlockFactory;

/// Builds `expected-prose` recognizers.
pub struct ExpectedProseFactory;

impl RecognizerFactory for ExpectedBlockFactory {
    fn kind(&self) -> &'static str {
        "expected-block"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let lead = p.regex("lead", &["spec"])?;
        let exit = exit_rows(&mut p, "spec")?;
        let lines = p.opt_string("lines")?;
        if lines.is_some_and(|l| l != "summary") {
            return Err("`lines` must be \"summary\"".to_owned());
        }
        p.finish()?;
        Ok(Arc::new(ExpectedBlock { id, lead, exit }))
    }
}

impl RecognizerFactory for ExpectedProseFactory {
    fn kind(&self) -> &'static str {
        "expected-prose"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let list = p.opt_bool("list")?.unwrap_or(false);
        let needs: &[&str] = if list { &[] } else { &["text"] };
        let lead = p.regex("lead", needs)?;
        let exit = exit_rows(&mut p, "text")?;
        p.finish()?;
        Ok(Arc::new(ExpectedProse {
            id,
            lead,
            exit,
            list,
        }))
    }
}

struct ExpectedBlock {
    id: RecognizerId,
    lead: Regex,
    exit: Vec<ExitRow>,
}

struct ExpectedProse {
    id: RecognizerId,
    lead: Regex,
    exit: Vec<ExitRow>,
    /// The lead is followed by a list that belongs to it (D47).
    list: bool,
}

impl ExpectedProse {
    /// How many blocks after the lead belong to it: with `list`, the list items at the lead's
    /// depth and, nested in them, paragraphs, list items and `text` fences. Any other block ends
    /// it. `None` when `list` is set and no list follows.
    fn taken(&self, cx: &StepContext<'_>, cursor: &Cursor<'_>, lead: &Located) -> Option<usize> {
        if !self.list {
            return Some(0);
        }
        let belongs = |b: &Located| match &b.block {
            Block::ListItem { .. } if b.depth == lead.depth => true,
            Block::Paragraph { .. } | Block::ListItem { .. } => b.depth > lead.depth,
            Block::Fence { .. } => {
                b.depth > lead.depth && b.block.fence_language(cx.doc) == Some("text")
            }
            _ => false,
        };
        let first = cursor.peek(1)?;
        if !matches!(first.block, Block::ListItem { .. }) || first.depth != lead.depth {
            return None;
        }
        Some(
            (1..)
                .take_while(|&k| cursor.peek(k).is_some_and(belongs))
                .count(),
        )
    }
}

impl Recognizer for ExpectedBlock {
    fn id(&self) -> &str {
        &self.id.0
    }

    fn recognize(&self, cx: &StepContext<'_>, cursor: &mut Cursor<'_>) -> Recognized {
        let Some(lead) = cursor.peek(0) else {
            return Recognized::No;
        };
        let Some((text, _)) = paragraph(cx, lead) else {
            return Recognized::No;
        };
        let Some(caps) = self.lead.captures(text) else {
            return Recognized::No;
        };
        let script = match amendable(cx, lead.span) {
            Ok(script) => script,
            Err(d) => return Recognized::Malformed(d),
        };
        let spec = caps.name("spec").map_or("", |m| m.as_str());
        let Some(exit) = first_exit(&self.exit, spec) else {
            return Recognized::Malformed(Diagnostic::error(
                "expected-spec",
                Some(lead.span),
                format!("`{spec}` names no exit status this format knows"),
            ));
        };
        let fence = cursor.peek(1).filter(|b| {
            matches!(b.block, Block::Fence { .. }) && b.block.fence_language(cx.doc) == Some("text")
        });
        let Some(fence) = fence else {
            return Recognized::Malformed(Diagnostic::error(
                "missing-block",
                Some(lead.span),
                "this expectation announces a ```text block of summary lines, and none follows it",
            ));
        };
        let Block::Fence { body, .. } = &fence.block else {
            return Recognized::No;
        };
        let expect = Expectation {
            exit,
            lines: Some(body.clone()),
            prose: None,
            comments: None,
        };
        cursor.advance(2);
        Recognized::Amend(amended(script, expect, lead.span.join(fence.span)))
    }
}

impl Recognizer for ExpectedProse {
    fn id(&self) -> &str {
        &self.id.0
    }

    fn recognize(&self, cx: &StepContext<'_>, cursor: &mut Cursor<'_>) -> Recognized {
        let Some(lead) = cursor.peek(0) else {
            return Recognized::No;
        };
        let Some((text, base)) = paragraph(cx, lead) else {
            return Recognized::No;
        };
        let Some(caps) = self.lead.captures(text) else {
            return Recognized::No;
        };
        let Some(taken) = self.taken(cx, cursor, lead) else {
            return Recognized::Malformed(Diagnostic::error(
                "missing-block",
                Some(lead.span),
                "this expectation announces a list, and none follows it",
            ));
        };
        let span = cursor
            .peek(taken)
            .map_or(lead.span, |last| lead.span.join(last.span));
        if let Some(manual) = joined_manual(cx, span) {
            cursor.advance(1 + taken);
            return Recognized::Amend(manual);
        }
        let (script, comments) = match extendable(cx, lead.span) {
            Ok(found) => found,
            Err(d) => return Recognized::Malformed(d),
        };
        let (said, prose) = if self.list {
            (cx.doc.source.slice(span), TextRef::Span(span))
        } else {
            let said = caps.name("text").map_or("", |m| m.as_str());
            let prose = capture_ref(&caps, "text", base).unwrap_or(TextRef::Span(lead.span));
            (said, prose)
        };
        let expect = Expectation {
            exit: first_exit(&self.exit, said).unwrap_or(ExitExpect::Unknown),
            lines: None,
            prose: Some(prose),
            comments,
        };
        cursor.advance(1 + taken);
        Recognized::Amend(amended(script, expect, span))
    }
}

/// The script an `expected-prose` amends, and the comment expectations it extends: a script
/// whose only expectation is its comment lines takes the sentence too (D48); any other
/// expectation already there is `expected-twice`.
fn extendable(cx: &StepContext<'_>, lead: Span) -> Result<(Action, Option<TextRef>), Diagnostic> {
    let comments_only = cx.previous.last().and_then(|a| match &a.kind {
        ActionKind::RunScript(s) | ActionKind::Commit(s) => s
            .expect
            .as_ref()
            .filter(|e| e.lines.is_none() && e.prose.is_none())
            .and_then(|e| e.comments.clone())
            .map(|c| (a.clone(), c)),
        _ => None,
    });
    if let Some((script, comments)) = comments_only {
        return Ok((script, Some(comments)));
    }
    amendable(cx, lead).map(|script| (script, None))
}

/// The Manual directly before an expectation, grown over it: what to check is part of what the
/// user does by hand (decision D38).
fn joined_manual(cx: &StepContext<'_>, lead: Span) -> Option<Action> {
    let mut manual = cx
        .previous
        .last()
        .filter(|a| matches!(a.kind, ActionKind::Manual { .. }))?
        .clone();
    manual.span = manual.span.join(lead);
    if let ActionKind::Manual { text, .. } = &mut manual.kind {
        *text = TextRef::Span(manual.span);
    }
    Some(manual)
}

/// `script` with `expect` attached, its span grown over the expectation.
fn amended(mut script: Action, expect: Expectation, over: Span) -> Action {
    if let ActionKind::RunScript(s) | ActionKind::Commit(s) = &mut script.kind {
        s.expect = Some(expect);
    }
    script.span = script.span.join(over);
    script
}
