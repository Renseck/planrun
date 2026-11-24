//! `file-block`: a lead paragraph naming a path and an operation, then one fence →
//! `WriteFile` or `ApplyPatch`.

use std::sync::Arc;

use planrun_model::{Action, ActionKind, ChangeKind, Manifest, RecognizerId, RepoPath, WriteMode};
use regex::Regex;

use super::params::Params;
use super::{lead_path, paragraph};
use crate::diagnostic::Diagnostic;
use crate::document::Block;
use crate::patch;
use crate::recognizer::{Cursor, Recognized, Recognizer, RecognizerFactory, StepContext};

/// Builds `file-block` recognizers.
pub struct FileBlockFactory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Write,
    Create,
    Overwrite,
    Patch,
}

impl RecognizerFactory for FileBlockFactory {
    fn kind(&self) -> &'static str {
        "file-block"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let lead = p.regex("lead", &["path"])?;
        let ops = p
            .table("ops")?
            .iter()
            .map(|(prefix, op)| {
                let op = op.as_str().ok_or("`ops` values must be strings")?;
                Ok((prefix.clone(), op_named(op)?))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let default_op = p.opt_string("default_op")?.map(op_named).transpose()?;
        p.finish()?;
        let has_op = lead.capture_names().flatten().any(|n| n == "op");
        if !has_op && default_op.is_none() {
            return Err("`lead` has no `op` capture and there is no `default_op`".to_owned());
        }
        Ok(Arc::new(FileBlock {
            id,
            lead,
            ops,
            default_op,
        }))
    }
}

fn op_named(name: &str) -> Result<Op, String> {
    match name {
        "write" => Ok(Op::Write),
        "create" => Ok(Op::Create),
        "overwrite" => Ok(Op::Overwrite),
        "patch" => Ok(Op::Patch),
        other => Err(format!(
            "op `{other}` is not write, create, overwrite or patch"
        )),
    }
}

struct FileBlock {
    id: RecognizerId,
    lead: Regex,
    ops: Vec<(String, Op)>,
    default_op: Option<Op>,
}

impl FileBlock {
    /// The op of the longest prefix of `written`.
    fn op(&self, written: &str) -> Option<Op> {
        self.ops
            .iter()
            .filter(|(prefix, _)| written.starts_with(prefix.as_str()))
            .max_by_key(|(prefix, _)| prefix.len())
            .map(|(_, op)| *op)
    }

    fn known_ops(&self) -> String {
        let names: Vec<String> = self.ops.iter().map(|(p, _)| format!("`{p}`")).collect();
        names.join(", ")
    }
}

impl Recognizer for FileBlock {
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
        let written = caps.name("op").map(|m| m.as_str());
        let op = written.map_or(self.default_op, |w| self.op(w));
        let Some(op) = op else {
            return Recognized::Malformed(Diagnostic::error(
                "unknown-file-op",
                Some(lead.span),
                format!(
                    "`{}` is not a file operation this format knows ({})",
                    written.unwrap_or(""),
                    self.known_ops()
                ),
            ));
        };
        let fence = cursor
            .peek(1)
            .filter(|b| matches!(b.block, Block::Fence { .. }));
        let Some(fence) = fence else {
            return Recognized::Malformed(Diagnostic::error(
                "missing-block",
                Some(lead.span),
                "this sentence announces a fenced block, and none follows it",
            ));
        };
        let path = match lead_path(&caps, base, lead.span) {
            Ok(path) => path,
            Err(d) => return Recognized::Malformed(d),
        };
        let Block::Fence { body, .. } = &fence.block else {
            return Recognized::No;
        };
        let kind = match op {
            Op::Patch => match patch_action(cx, path, body, lead.span, fence.span) {
                Ok(kind) => kind,
                Err(d) => return Recognized::Malformed(d),
            },
            Op::Write => write(path.clone(), body, mode_from(cx.manifest, &path)),
            Op::Create => write(path, body, WriteMode::Create),
            Op::Overwrite => write(path, body, WriteMode::Overwrite),
        };
        cursor.advance(2);
        Recognized::Yes(vec![Action {
            kind,
            span: lead.span.join(fence.span),
            origin: self.id.clone(),
        }])
    }
}

fn write(path: RepoPath, body: &planrun_model::TextRef, mode: WriteMode) -> ActionKind {
    ActionKind::WriteFile {
        path,
        body: body.clone(),
        mode,
    }
}

/// Create or Overwrite from the manifest; `FromManifest` when it does not say (the `manifest`
/// check reports it).
fn mode_from(manifest: &Manifest, path: &RepoPath) -> WriteMode {
    let change = manifest
        .entries
        .iter()
        .find(|e| &e.path == path)
        .map(|e| e.change);
    match change {
        Some(ChangeKind::Create) => WriteMode::Create,
        Some(ChangeKind::Modify | ChangeKind::Rewrite) => WriteMode::Overwrite,
        _ => WriteMode::FromManifest,
    }
}

fn patch_action(
    cx: &StepContext<'_>,
    path: RepoPath,
    body: &planrun_model::TextRef,
    lead: planrun_model::Span,
    fence: planrun_model::Span,
) -> Result<ActionKind, Diagnostic> {
    let header = patch::header(cx.doc.text(body))
        .map_err(|e| Diagnostic::error("patch-shape", Some(fence), e))?;
    if let Some(other) = header.paths.iter().find(|p| p.as_str() != path.as_str()) {
        return Err(Diagnostic::error(
            "patch-path",
            Some(fence),
            format!("the patch changes `{other}`; the sentence before it names `{path}`"),
        )
        .with_related(lead, "the path named here"));
    }
    Ok(ActionKind::ApplyPatch {
        path,
        patch: body.clone(),
        pre: header.pre,
        post: header.post,
    })
}
