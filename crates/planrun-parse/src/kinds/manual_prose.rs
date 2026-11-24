//! `manual-prose`: a paragraph that describes something planrun cannot do exactly (an edit with
//! no anchor, an edit that depends on output, a check by eye), with the fence after it when
//! `takes_fence` → one `Manual` (decision D21).

use std::sync::Arc;

use planrun_model::{Action, ActionKind, ManualReason, RecognizerId, TextRef};
use regex::Regex;

use super::paragraph;
use super::params::Params;
use crate::document::Block;
use crate::recognizer::{Cursor, Recognized, Recognizer, RecognizerFactory, StepContext};

/// Builds `manual-prose` recognizers.
pub struct ManualProseFactory;

impl RecognizerFactory for ManualProseFactory {
    fn kind(&self) -> &'static str {
        "manual-prose"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let lead = p.regex("lead", &[])?;
        let reason = match p.string("reason")? {
            "prose-edit" => ManualReason::ProseEdit,
            "output-dependent" => ManualReason::OutputDependent,
            "human-check" => ManualReason::HumanCheck,
            other => {
                return Err(format!(
                    "reason `{other}` is not prose-edit, output-dependent or human-check"
                ));
            }
        };
        let takes_fence = p.opt_bool("takes_fence")?.unwrap_or(false);
        p.finish()?;
        Ok(Arc::new(ManualProse {
            id,
            lead,
            reason,
            takes_fence,
        }))
    }
}

struct ManualProse {
    id: RecognizerId,
    lead: Regex,
    reason: ManualReason,
    takes_fence: bool,
}

impl Recognizer for ManualProse {
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
        if !self.lead.is_match(text) {
            return Recognized::No;
        }
        let fence = cursor
            .peek(1)
            .filter(|b| self.takes_fence && matches!(b.block, Block::Fence { .. }));
        let span = fence.map_or(lead.span, |f| lead.span.join(f.span));
        cursor.advance(if fence.is_some() { 2 } else { 1 });
        Recognized::Yes(vec![Action {
            kind: ActionKind::Manual {
                reason: self.reason.clone(),
                text: TextRef::Span(span),
            },
            span,
            origin: self.id.clone(),
            path_origin: Default::default(),
        }])
    }
}
