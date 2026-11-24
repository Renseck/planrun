//! `script`: a fence in a shell language → `RunScript`, or `Commit` when its body says so.

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
enum Intent {
    /// The `Commit` action kind (its script carries `ScriptIntent::Other`).
    Commit,
    /// A `RunScript` with this intent.
    Run(ScriptIntent),
}

impl RecognizerFactory for ScriptFactory {
    fn kind(&self) -> &'static str {
        "script"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let intents = p
            .rows("intents", &["intent", "body"])?
            .into_iter()
            .map(|row| {
                let intent = intent_named(row_string(row, "intent")?)?;
                Ok((
                    compile_regex("intents.body", row_string(row, "body")?, &[])?,
                    intent,
                ))
            })
            .collect::<Result<_, String>>()?;
        p.finish()?;
        Ok(Arc::new(ScriptRecognizer { id, intents }))
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
    intents: Vec<(Regex, Intent)>,
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
        let text = cx.doc.text(body);
        let intent = self
            .intents
            .iter()
            .find(|(regex, _)| regex.is_match(text))
            .map_or(Intent::Run(ScriptIntent::Verify), |(_, intent)| *intent);
        let script = |intent| Script {
            shell,
            body: body.clone(),
            expect: None,
            intent,
        };
        let kind = match intent {
            Intent::Commit => ActionKind::Commit(script(ScriptIntent::Other)),
            Intent::Run(intent) => ActionKind::RunScript(script(intent)),
        };
        cursor.advance(1);
        Recognized::Yes(vec![Action {
            kind,
            span: located.span,
            origin: self.id.clone(),
        }])
    }
}
