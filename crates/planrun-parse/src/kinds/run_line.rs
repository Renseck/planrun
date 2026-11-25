//! `run-line`: a command in a paragraph (`` Run: `cargo test` ``) → `RunScript` in the spec's
//! shell, with the `Expected: …` of the same paragraph as its expectation (decision D39).

use std::sync::Arc;

use planrun_model::{Action, ExitExpect, Expectation, RecognizerId, Script, ScriptIntent, Shell};
use regex::Regex;

use super::params::{ExitRow, Params, exit_rows, first_exit};
use super::script::{IntentRow, intent_of, intent_rows, script_action};
use super::{capture_ref, paragraph};
use crate::diagnostic::Diagnostic;
use crate::recognizer::{Cursor, Recognized, Recognizer, RecognizerFactory, StepContext};

/// Builds `run-line` recognizers.
pub struct RunLineFactory;

impl RecognizerFactory for RunLineFactory {
    fn kind(&self) -> &'static str {
        "run-line"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let lead = p.regex("lead", &["cmd"])?;
        let shell = match p.string("shell")? {
            "pwsh" => Shell::Pwsh,
            "bash" => Shell::Bash,
            other => return Err(format!("shell `{other}` is not pwsh or bash")),
        };
        let exit = exit_rows(&mut p, "text")?;
        let intents = intent_rows(&mut p, false)?;
        p.finish()?;
        Ok(Arc::new(RunLine {
            id,
            lead,
            shell,
            exit,
            intents,
        }))
    }
}

struct RunLine {
    id: RecognizerId,
    lead: Regex,
    shell: Shell,
    exit: Vec<ExitRow>,
    intents: Vec<IntentRow>,
}

impl Recognizer for RunLine {
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
        let Some(body) = capture_ref(&caps, "cmd", base).filter(|_| !caps["cmd"].is_empty()) else {
            return Recognized::Malformed(Diagnostic::error(
                "missing-block",
                Some(lead.span),
                "this line announces a command, and names none",
            ));
        };
        let expect = caps.name("expected").map(|said| Expectation {
            exit: first_exit(&self.exit, said.as_str()).unwrap_or(ExitExpect::Unknown),
            lines: None,
            prose: capture_ref(&caps, "expected", base),
            comments: None,
        });
        let script = Script {
            shell: self.shell,
            body,
            expect,
            intent: ScriptIntent::Verify,
        };
        let kind = script_action(script, intent_of(&self.intents, &caps["cmd"], None));
        cursor.advance(1);
        Recognized::Yes(vec![Action {
            kind,
            span: lead.span,
            origin: self.id.clone(),
            path_origin: Default::default(),
        }])
    }
}
