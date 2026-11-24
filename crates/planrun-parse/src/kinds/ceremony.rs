//! `ceremony`: the fence directly after a patch that saves and applies it by hand. planrun
//! applies the patch itself, so the fence is absorbed: consumed, never run (decision D17).

use std::sync::Arc;

use planrun_model::{ActionKind, RecognizerId};
use regex::Regex;

use super::params::Params;
use crate::document::Block;
use crate::recognizer::{Cursor, Recognized, Recognizer, RecognizerFactory, StepContext};

/// Builds `ceremony` recognizers.
pub struct CeremonyFactory;

impl RecognizerFactory for CeremonyFactory {
    fn kind(&self) -> &'static str {
        "ceremony"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let after = p.string("after")?;
        if after != "patch" {
            return Err(format!("`after = \"{after}\"`: only `patch` is supported"));
        }
        let body = p.regex("body", &[])?;
        p.finish()?;
        Ok(Arc::new(Ceremony { id, body }))
    }
}

struct Ceremony {
    id: RecognizerId,
    body: Regex,
}

impl Recognizer for Ceremony {
    fn id(&self) -> &str {
        &self.id.0
    }

    fn recognize(&self, cx: &StepContext<'_>, cursor: &mut Cursor<'_>) -> Recognized {
        let after_patch = cx
            .previous
            .last()
            .is_some_and(|a| matches!(a.kind, ActionKind::ApplyPatch { .. }));
        if !after_patch {
            return Recognized::No;
        }
        let Some(Block::Fence { body, .. }) = cursor.peek(0).map(|b| &b.block) else {
            return Recognized::No;
        };
        if !self.body.is_match(cx.doc.text(body)) {
            return Recognized::No;
        }
        cursor.advance(1);
        Recognized::Yes(Vec::new())
    }
}
