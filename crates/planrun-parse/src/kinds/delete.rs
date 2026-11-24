//! `delete`: a sentence naming a file to delete → `Delete`.

use std::sync::Arc;

use planrun_model::{Action, ActionKind, RecognizerId};
use regex::Regex;

use super::params::Params;
use super::{paragraph, resolved_lead_path};
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
        let by_file_name = p.opt_bool("file_name_from_manifest")?.unwrap_or(false);
        p.finish()?;
        Ok(Arc::new(Delete {
            id,
            lead,
            via_git,
            by_file_name,
        }))
    }
}

struct Delete {
    id: RecognizerId,
    lead: Regex,
    via_git: bool,
    /// Resolve a bare file name against the manifest (decision D32).
    by_file_name: bool,
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
        let (path, path_origin) =
            match resolved_lead_path(cx, &caps, base, lead.span, self.by_file_name) {
                Ok(resolved) => resolved,
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
            path_origin,
        }])
    }
}
