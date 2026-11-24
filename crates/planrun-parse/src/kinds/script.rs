//! `script`: a fence in a shell language → `RunScript`, or `Commit` when its intent says so.
//!
//! With `leads` set, the kind claims a fence only behind a run lead or, with `first_block`, as
//! the first block of a written step; any other shell fence falls to the fallback, Manual
//! (decision D28). An `intents` row matches the body, the lead paragraph, or both (D29).

use std::sync::Arc;

use planrun_model::{Action, ActionKind, RecognizerId, Script, ScriptIntent};
use regex::Regex;

use super::params::{Params, row_string};
use crate::document::Block;
use crate::recognizer::{Cursor, Recognized, Recognizer, RecognizerFactory, StepContext};
use crate::spec::compile_regex;

/// Builds `script` recognizers.
pub struct ScriptFactory;

/// What an `intents` row selects.
#[derive(Debug, Clone, Copy)]
pub(super) enum Intent {
    /// The `Commit` action kind (its script carries `ScriptIntent::Other`).
    Commit,
    /// A `RunScript` with this intent.
    Run(ScriptIntent),
}

/// One `intents` row: what it matches, and the intent it gives.
pub(super) struct IntentRow {
    body: Option<Regex>,
    lead: Option<Regex>,
    intent: Intent,
}

impl IntentRow {
    fn matches(&self, body: &str, lead: Option<&str>) -> bool {
        let body_ok = self.body.as_ref().is_none_or(|r| r.is_match(body));
        let lead_ok = self
            .lead
            .as_ref()
            .is_none_or(|r| lead.is_some_and(|l| r.is_match(l)));
        body_ok && lead_ok
    }
}

/// The `intents` rows of a kind; `lead` is allowed when `with_lead`.
pub(super) fn intent_rows(p: &mut Params<'_>, with_lead: bool) -> Result<Vec<IntentRow>, String> {
    let keys: &[&str] = if with_lead {
        &["intent", "body", "lead"]
    } else {
        &["intent", "body"]
    };
    p.rows("intents", keys)?
        .into_iter()
        .map(|row| {
            let intent = intent_named(row_string(row, "intent")?)?;
            let pattern = |key: &str| {
                row.get(key)
                    .map(|v| {
                        let text = v
                            .as_str()
                            .ok_or_else(|| format!("intents.{key} must be a string"))?;
                        compile_regex(&format!("intents.{key}"), text, &[])
                    })
                    .transpose()
            };
            let (body, lead) = (pattern("body")?, pattern("lead")?);
            if body.is_none() && lead.is_none() {
                return Err("an intents row needs `body` or `lead`".to_owned());
            }
            Ok(IntentRow { body, lead, intent })
        })
        .collect()
}

/// The intent of the first row that matches; `verify` when none does.
pub(super) fn intent_of(rows: &[IntentRow], body: &str, lead: Option<&str>) -> Intent {
    rows.iter()
        .find(|row| row.matches(body, lead))
        .map_or(Intent::Run(ScriptIntent::Verify), |row| row.intent)
}

/// `script` as the action of `intent`.
pub(super) fn script_action(script: Script, intent: Intent) -> ActionKind {
    match intent {
        Intent::Commit => ActionKind::Commit(Script {
            intent: ScriptIntent::Other,
            ..script
        }),
        Intent::Run(intent) => ActionKind::RunScript(Script { intent, ..script }),
    }
}

impl RecognizerFactory for ScriptFactory {
    fn kind(&self) -> &'static str {
        "script"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let intents = intent_rows(&mut p, true)?;
        let leads = p
            .opt_strings("leads")?
            .map(|list| {
                list.iter()
                    .map(|l| compile_regex("leads", l, &[]))
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;
        let first_block = p.opt_bool("first_block")?;
        p.finish()?;
        if first_block.is_some() && leads.is_none() {
            return Err("`first_block` needs `leads`".to_owned());
        }
        Ok(Arc::new(ScriptRecognizer {
            id,
            intents,
            leads,
            first_block: first_block.unwrap_or(false),
        }))
    }
}

fn intent_named(name: &str) -> Result<Intent, String> {
    match name {
        "commit" => Ok(Intent::Commit),
        "gate" => Ok(Intent::Run(ScriptIntent::Gate)),
        "preview" => Ok(Intent::Run(ScriptIntent::Preview)),
        "apply" => Ok(Intent::Run(ScriptIntent::Apply)),
        "verify" => Ok(Intent::Run(ScriptIntent::Verify)),
        "other" => Ok(Intent::Run(ScriptIntent::Other)),
        other => Err(format!(
            "intent `{other}` is not commit, gate, preview, apply, verify or other"
        )),
    }
}

struct ScriptRecognizer {
    id: RecognizerId,
    intents: Vec<IntentRow>,
    /// When set, only fences behind one of these leads (or a first block) are claimed.
    leads: Option<Vec<Regex>>,
    first_block: bool,
}

impl ScriptRecognizer {
    /// Whether the guard (decision D28) lets the kind claim the fence at the cursor. A body
    /// intent never qualifies a fence on its own.
    fn claims(&self, cx: &StepContext<'_>, cursor: &Cursor<'_>, lead: Option<&str>) -> bool {
        let Some(leads) = &self.leads else {
            return true;
        };
        if self.first_block && cx.written_step && cursor.at_start() {
            return true;
        }
        lead.is_some_and(|text| leads.iter().any(|l| l.is_match(text)))
    }
}

impl Recognizer for ScriptRecognizer {
    fn id(&self) -> &str {
        &self.id.0
    }

    fn recognize(&self, cx: &StepContext<'_>, cursor: &mut Cursor<'_>) -> Recognized {
        let Some(located) = cursor.peek(0) else {
            return Recognized::No;
        };
        let Block::Fence { body, .. } = &located.block else {
            return Recognized::No;
        };
        let language = located.block.fence_language(cx.doc).unwrap_or("");
        let Some(shell) = cx.settings.shell_for(language) else {
            return Recognized::No;
        };
        let lead = cursor.previous().and_then(|b| match &b.block {
            Block::Paragraph { text } => Some(cx.doc.text(text)),
            _ => None,
        });
        if !self.claims(cx, cursor, lead) {
            return Recognized::No;
        }
        let text = cx.doc.text(body);
        let script = Script {
            shell,
            body: body.clone(),
            expect: None,
            intent: ScriptIntent::Verify,
        };
        let kind = script_action(script, intent_of(&self.intents, text, lead));
        cursor.advance(1);
        Recognized::Yes(vec![Action {
            kind,
            span: located.span,
            origin: self.id.clone(),
            path_origin: Default::default(),
        }])
    }
}
