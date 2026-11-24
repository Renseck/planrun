//! `expected-block` (`Expected (…):` + a `text` fence of summary lines) and `expected-prose`
//! (`Expected: …`): both amend the script directly before them.

use std::sync::Arc;

use planrun_model::{Action, ActionKind, ExitExpect, Expectation, RecognizerId, Span, TextRef};
use regex::Regex;

use super::params::{ExitRow, Params, exit_rows, first_exit};
use super::{amendable, capture_ref, paragraph};
use crate::diagnostic::Diagnostic;
use crate::document::Block;
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
        let lead = p.regex("lead", &["text"])?;
        let exit = exit_rows(&mut p, "text")?;
        p.finish()?;
        Ok(Arc::new(ExpectedProse { id, lead, exit }))
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
        let script = match amendable(cx, lead.span) {
            Ok(script) => script,
            Err(d) => return Recognized::Malformed(d),
        };
        let said = caps.name("text").map_or("", |m| m.as_str());
        let expect = Expectation {
            exit: first_exit(&self.exit, said).unwrap_or(ExitExpect::Unknown),
            lines: None,
            prose: capture_ref(&caps, "text", base).or(Some(TextRef::Span(lead.span))),
        };
        cursor.advance(1);
        Recognized::Amend(amended(script, expect, lead.span))
    }
}

/// `script` with `expect` attached, its span grown over the expectation.
fn amended(mut script: Action, expect: Expectation, over: Span) -> Action {
    if let ActionKind::RunScript(s) | ActionKind::Commit(s) = &mut script.kind {
        s.expect = Some(expect);
    }
    script.span = script.span.join(over);
    script
}
