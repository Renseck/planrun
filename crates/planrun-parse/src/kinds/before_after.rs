//! `before-after`: a lead naming a file, the text to find in a fence, an optional separator
//! paragraph (`After:`), and its replacement in a second fence → `ReplaceExact`.
//!
//! A lead without a path takes the task's one editable manifest entry when `infer_path` is set;
//! with none or several, the lead and both fences are `Manual(ProseEdit)` (decision D32).

use std::sync::Arc;

use planrun_model::{Action, ActionKind, ManualReason, PathOrigin, RecognizerId, Span, TextRef};
use regex::Regex;

use super::params::Params;
use super::{only_entry, paragraph, resolved_lead_path};
use crate::diagnostic::Diagnostic;
use crate::document::{Block, Located};
use crate::recognizer::{Cursor, Recognized, Recognizer, RecognizerFactory, StepContext};
use crate::spec::compile_regex;

/// Builds `before-after` recognizers.
pub struct BeforeAfterFactory;

impl RecognizerFactory for BeforeAfterFactory {
    fn kind(&self) -> &'static str {
        "before-after"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let lead = p.regex("lead", &[])?;
        let separator = p
            .opt_string("separator")?
            .map(|s| compile_regex("separator", s, &[]))
            .transpose()?;
        let infer_path = p.opt_bool("infer_path")?.unwrap_or(false);
        let by_file_name = p.opt_bool("file_name_from_manifest")?.unwrap_or(false);
        p.finish()?;
        let has_path = lead.capture_names().flatten().any(|n| n == "path");
        if has_path == infer_path {
            return Err(
                "`lead` must capture `path`, or have no `path` capture and `infer_path = true`"
                    .to_owned(),
            );
        }
        Ok(Arc::new(BeforeAfter {
            id,
            lead,
            separator,
            infer_path,
            by_file_name,
        }))
    }
}

struct BeforeAfter {
    id: RecognizerId,
    lead: Regex,
    separator: Option<Regex>,
    infer_path: bool,
    by_file_name: bool,
}

impl BeforeAfter {
    /// The two fences after the lead, and how many blocks lead through the second fence take.
    fn fences<'a>(
        &self,
        cx: &StepContext<'_>,
        cursor: &Cursor<'a>,
        lead: Span,
    ) -> Result<(&'a Located, &'a Located, usize), Diagnostic> {
        let missing = |what: &str| {
            Diagnostic::error(
                "missing-block",
                Some(lead),
                format!("this edit announces {what}, and none follows"),
            )
        };
        let fence = |offset| {
            cursor
                .peek(offset)
                .filter(|b| matches!(b.block, Block::Fence { .. }))
        };
        let before = fence(1).ok_or_else(|| missing("the text to replace in a fenced block"))?;
        let Some(separator) = &self.separator else {
            let after = fence(2).ok_or_else(|| missing("a second fenced block"))?;
            return Ok((before, after, 3));
        };
        let between = cursor
            .peek(2)
            .and_then(|b| paragraph(cx, b))
            .is_some_and(|(text, _)| separator.is_match(text));
        if !between {
            return Err(missing(&format!(
                "a `{}` paragraph after its first block",
                separator.as_str()
            )));
        }
        let after = fence(3).ok_or_else(|| missing("the replacement in a fenced block"))?;
        Ok((before, after, 4))
    }
}

impl Recognizer for BeforeAfter {
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
        let (before, after, count) = match self.fences(cx, cursor, lead.span) {
            Ok(found) => found,
            Err(d) => return Recognized::Malformed(d),
        };
        let span = lead.span.join(after.span);
        let target = if self.infer_path {
            only_entry(cx).map(|path| (path, PathOrigin::OnlyEntry))
        } else {
            match resolved_lead_path(cx, &caps, base, lead.span, self.by_file_name) {
                Ok(resolved) => Some(resolved),
                Err(d) => return Recognized::Malformed(d),
            }
        };
        let (Block::Fence { body: old, .. }, Block::Fence { body: new, .. }) =
            (&before.block, &after.block)
        else {
            return Recognized::No;
        };
        let (kind, path_origin) = match target {
            Some((path, origin)) => (
                ActionKind::ReplaceExact {
                    path,
                    before: old.clone(),
                    after: new.clone(),
                },
                origin,
            ),
            None => (
                ActionKind::Manual {
                    reason: ManualReason::ProseEdit,
                    text: TextRef::Span(span),
                },
                PathOrigin::Written,
            ),
        };
        cursor.advance(count);
        Recognized::Yes(vec![Action {
            kind,
            span,
            origin: self.id.clone(),
            path_origin,
        }])
    }
}
