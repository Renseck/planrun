//! `delete`: a sentence naming a file to delete → `Delete`.

use std::sync::Arc;

use planrun_model::{Action, ActionKind, RecognizerId};
use regex::Regex;

use super::params::Params;
use super::{lead_path, paragraph};
use crate::recognizer::{Cursor, Recognized, Recognizer, RecognizerFactory, StepContext};

/// Builds `delete` recognizers.
pub struct DeleteFactory;

impl RecognizerFactory for DeleteFactory {
    fn kind(&self) -> &'static str {
        "delete"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let lead = p.regex("lead", &["path"])?;
        let via_git = p.opt_bool("via_git")?.unwrap_or(false);
        p.finish()?;
        Ok(Arc::new(Delete { id, lead, via_git }))
    }
}

struct Delete {
    id: RecognizerId,
    lead: Regex,
    via_git: bool,
}

impl Recognizer for Delete {
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
        let path = match lead_path(&caps, base, lead.span) {
            Ok(path) => path,
            Err(d) => return Recognized::Malformed(d),
        };
        cursor.advance(1);
        Recognized::Yes(vec![Action {
            kind: ActionKind::Delete {
                path,
                via_git: self.via_git,
            },
            span: lead.span,
            origin: self.id.clone(),
        }])
    }
}
