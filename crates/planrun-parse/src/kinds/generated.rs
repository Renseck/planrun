//! `generated`: a sentence saying a command produces a file, not the plan → `Generated`.

use std::sync::Arc;

use planrun_model::{Action, ActionKind, RecognizerId};
use regex::Regex;

use super::params::Params;
use super::{lead_path, paragraph};
use crate::recognizer::{Cursor, Recognized, Recognizer, RecognizerFactory, StepContext};

/// Builds `generated` recognizers.
pub struct GeneratedFactory;

impl RecognizerFactory for GeneratedFactory {
    fn kind(&self) -> &'static str {
        "generated"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let lead = p.regex("lead", &["path"])?;
        p.finish()?;
        Ok(Arc::new(Generated { id, lead }))
    }
}

struct Generated {
    id: RecognizerId,
    lead: Regex,
}

impl Recognizer for Generated {
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
            kind: ActionKind::Generated { path },
            span: lead.span,
            origin: self.id.clone(),
            path_origin: Default::default(),
        }])
    }
}
