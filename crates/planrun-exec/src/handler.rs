//! Action handlers: one per action kind, registered in a [`HandlerSet`] (the execution-side
//! plugin point, docs/extending.md §7). A handler simulates its action on a virtual tree (the
//! dry-run behaviour of extending.md §4); writing is the transaction's job (D53).

use std::fmt;

use planrun_model::{AbbrevBlobId, Action, ActionKind, PlanSource, RepoPath, Span, WriteMode};

use crate::eol;
use crate::finding::Finding;
use crate::tree::{State, VirtualTree};

/// What a simulation did, for the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Simulated {
    /// The path it changed.
    pub path: RepoPath,
    /// What it did to the path.
    pub effect: Effect,
}

/// What a file action does to its path. Its `Display` is the dry run's short description
/// (`create`, `aa80cf1 → 97bd92a`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Full contents for a path that does not exist.
    Create,
    /// Full contents for a path that exists.
    Overwrite,
    /// A patch, with the blob ids the plan states before and after it.
    Patch {
        /// The `index` line's pre-image id, when the patch has one.
        pre: Option<AbbrevBlobId>,
        /// Its post-image id.
        post: Option<AbbrevBlobId>,
    },
    /// An exact replacement.
    Replace,
    /// A delete; `via_git` for `git rm`.
    Delete {
        /// Through `git rm`.
        via_git: bool,
    },
}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id =
            |a: &Option<AbbrevBlobId>| a.as_ref().map_or_else(|| "?".to_owned(), |a| a.to_string());
        match self {
            Effect::Create => f.write_str("create"),
            Effect::Overwrite => f.write_str("overwrite"),
            Effect::Patch { pre, post } => write!(f, "{} → {}", id(pre), id(post)),
            Effect::Replace => f.write_str("replace"),
            Effect::Delete { via_git: true } => f.write_str("delete (git rm)"),
            Effect::Delete { via_git: false } => f.write_str("delete"),
        }
    }
}

/// Executes one kind of action. Registered per kind in the [`HandlerSet`].
pub trait ActionHandler: Send + Sync {
    /// True for the action kinds this handler executes.
    fn handles(&self, action: &Action) -> bool;

    /// Applies `action` to `tree`, checking everything the disk would refuse. `source` resolves
    /// the action's text.
    fn simulate(
        &self,
        action: &Action,
        source: &PlanSource,
        tree: &mut VirtualTree<'_>,
    ) -> Result<Simulated, Finding>;
}

/// The handlers, tried in order; the first that handles an action executes it.
#[derive(Default)]
pub struct HandlerSet {
    handlers: Vec<Box<dyn ActionHandler>>,
}

impl HandlerSet {
    /// The handlers of the file actions: WriteFile, ApplyPatch, ReplaceExact, Delete.
    pub fn files() -> Self {
        let mut set = Self::default();
        set.add(Box::new(WriteHandler));
        set.add(Box::new(PatchHandler));
        set.add(Box::new(ReplaceHandler));
        set.add(Box::new(DeleteHandler));
        set
    }

    /// Adds a handler.
    pub fn add(&mut self, handler: Box<dyn ActionHandler>) {
        self.handlers.push(handler);
    }

    /// The handler for `action`.
    pub fn find(&self, action: &Action) -> Option<&dyn ActionHandler> {
        self.handlers
            .iter()
            .find(|h| h.handles(action))
            .map(|h| h.as_ref())
    }
}

/// The path a file action changes (`Generated` is no file action: nothing writes it).
pub fn file_path(kind: &ActionKind) -> Option<&RepoPath> {
    match kind {
        ActionKind::WriteFile { path, .. }
        | ActionKind::ApplyPatch { path, .. }
        | ActionKind::ReplaceExact { path, .. }
        | ActionKind::Delete { path, .. } => Some(path),
        _ => None,
    }
}

/// The state of `path` as present/absent; an unknown path cannot be simulated.
fn presence(
    tree: &mut VirtualTree<'_>,
    path: &RepoPath,
    span: Option<Span>,
) -> Result<bool, Finding> {
    match tree.state(path, span)? {
        State::Unknown => Err(Finding::warning(
            "unverified",
            span,
            Some(path),
            format!("`{path}` depends on a script or Manual step that did not run"),
        )),
        state => Ok(state.exists()),
    }
}

fn missing(path: &RepoPath, span: Option<Span>, hint: &str) -> Finding {
    Finding::error(
        "target-missing",
        span,
        Some(path),
        format!("`{path}` does not exist{hint}"),
    )
}

fn exists(path: &RepoPath, span: Option<Span>) -> Finding {
    Finding::error(
        "create-exists",
        span,
        Some(path),
        format!("`{path}` already exists; the plan creates it"),
    )
}

fn wrong_kind(action: &Action) -> Finding {
    Finding::error(
        "internal",
        Some(action.span),
        None,
        "a handler got an action of another kind",
    )
}

/// `WriteFile`: Create onto an absent path, Overwrite onto a present one, with the D54 bytes.
/// `FromManifest` is refused (`write-mode-unknown`, D58): exec never guesses from the disk.
pub struct WriteHandler;

impl ActionHandler for WriteHandler {
    fn handles(&self, action: &Action) -> bool {
        matches!(action.kind, ActionKind::WriteFile { .. })
    }

    fn simulate(
        &self,
        action: &Action,
        source: &PlanSource,
        tree: &mut VirtualTree<'_>,
    ) -> Result<Simulated, Finding> {
        let ActionKind::WriteFile { path, body, mode } = &action.kind else {
            return Err(wrong_kind(action));
        };
        let span = Some(action.span);
        if *mode == WriteMode::FromManifest {
            return Err(Finding::error(
                "write-mode-unknown",
                span,
                Some(path),
                format!("full contents for `{path}`: the plan says neither create nor overwrite"),
            ));
        }
        let present = presence(tree, path, span)?;
        let body = source.resolve(body).as_bytes();
        let (bytes, effect) = match mode {
            WriteMode::Create if present => return Err(exists(path, span)),
            WriteMode::Create => (body.to_vec(), Effect::Create),
            _ if !present => return Err(missing(path, span, "; the plan overwrites it")),
            _ => (overwrite_bytes(tree, path, body, span)?, Effect::Overwrite),
        };
        tree.put(path, bytes, span)?;
        Ok(Simulated {
            path: path.clone(),
            effect,
        })
    }
}

fn overwrite_bytes(
    tree: &mut VirtualTree<'_>,
    path: &RepoPath,
    body: &[u8],
    span: Option<Span>,
) -> Result<Vec<u8>, Finding> {
    let existing = tree.bytes(path, span)?;
    let attrs = tree.attributes(path, span)?;
    let checkout = tree.checkout_eol(path, span)?;
    Ok(eol::render(
        body,
        eol::for_overwrite(&attrs, &existing, checkout),
    ))
}

/// `ApplyPatch`: the `index` line's preimage matches, `git apply --cached` applies it, and the
/// postimage matches (D20).
pub struct PatchHandler;

impl ActionHandler for PatchHandler {
    fn handles(&self, action: &Action) -> bool {
        matches!(action.kind, ActionKind::ApplyPatch { .. })
    }

    fn simulate(
        &self,
        action: &Action,
        source: &PlanSource,
        tree: &mut VirtualTree<'_>,
    ) -> Result<Simulated, Finding> {
        let ActionKind::ApplyPatch {
            path,
            patch,
            pre,
            post,
        } = &action.kind
        else {
            return Err(wrong_kind(action));
        };
        let span = Some(action.span);
        check_pre(tree, path, pre.as_ref(), span)?;
        let text = source.resolve(patch).as_bytes();
        tree.apply(path, text, span).map_err(|message| {
            Finding::error(
                "patch-does-not-apply",
                span,
                Some(path),
                format!("`{path}`: {message}"),
            )
        })?;
        check_post(tree, path, post.as_ref(), span)?;
        Ok(Simulated {
            path: path.clone(),
            effect: Effect::Patch {
                pre: *pre,
                post: *post,
            },
        })
    }
}

fn check_pre(
    tree: &mut VirtualTree<'_>,
    path: &RepoPath,
    pre: Option<&AbbrevBlobId>,
    span: Option<Span>,
) -> Result<(), Finding> {
    let present = presence(tree, path, span)?;
    let Some(pre) = pre else {
        return Ok(());
    };
    if pre.is_absent() {
        return if present {
            Err(exists(path, span))
        } else {
            Ok(())
        };
    }
    let Some(blob) = tree.blob(path, span)? else {
        return Err(missing(path, span, "; the patch changes it"));
    };
    if blob.matches(pre) {
        return Ok(());
    }
    Err(Finding::error(
        "blob-mismatch",
        span,
        Some(path),
        format!("`{path}` is blob {blob} here; the patch starts from {pre}"),
    ))
}

fn check_post(
    tree: &mut VirtualTree<'_>,
    path: &RepoPath,
    post: Option<&AbbrevBlobId>,
    span: Option<Span>,
) -> Result<(), Finding> {
    let Some(post) = post else {
        return Ok(());
    };
    let blob = tree.blob(path, span)?;
    let fits = match blob {
        None => post.is_absent(),
        Some(blob) => blob.matches(post),
    };
    if fits {
        return Ok(());
    }
    let found = blob.map_or_else(|| "absent".to_owned(), |b| b.to_string());
    Err(Finding::error(
        "post-mismatch",
        span,
        Some(path),
        format!("`{path}` became {found}; the patch says {post}"),
    ))
}

/// `ReplaceExact`: `before` occurs exactly once (in LF space for text, D10), and the result
/// keeps the file's convention (D54); a mixed file is `eol-mixed`.
pub struct ReplaceHandler;

impl ActionHandler for ReplaceHandler {
    fn handles(&self, action: &Action) -> bool {
        matches!(action.kind, ActionKind::ReplaceExact { .. })
    }

    fn simulate(
        &self,
        action: &Action,
        source: &PlanSource,
        tree: &mut VirtualTree<'_>,
    ) -> Result<Simulated, Finding> {
        let ActionKind::ReplaceExact {
            path,
            before,
            after,
        } = &action.kind
        else {
            return Err(wrong_kind(action));
        };
        let span = Some(action.span);
        if !presence(tree, path, span)? {
            return Err(missing(path, span, "; the plan edits it"));
        }
        let existing = tree.bytes(path, span)?;
        let attrs = tree.attributes(path, span)?;
        let checkout = tree.checkout_eol(path, span)?;
        let convention = eol::for_replace(&attrs, &existing, checkout).map_err(|_| {
            Finding::error(
                "eol-mixed",
                span,
                Some(path),
                format!(
                    "`{path}` has both LF and CRLF line endings; an exact edit would rewrite them"
                ),
            )
        })?;
        let hay = eol::search_space(&existing, convention);
        let before = source.resolve(before).as_bytes();
        let after = source.resolve(after).as_bytes();
        let replaced = replace_once(&hay, before, after).map_err(|count| {
            Finding::error(
                "before-not-unique",
                span,
                Some(path),
                format!("the text to replace occurs {count} times in `{path}`; it must occur once"),
            )
        })?;
        tree.put(path, eol::render(&replaced, convention), span)?;
        Ok(Simulated {
            path: path.clone(),
            effect: Effect::Replace,
        })
    }
}

/// `hay` with its one occurrence of `before` replaced; the number of occurrences otherwise.
fn replace_once(hay: &[u8], before: &[u8], after: &[u8]) -> Result<Vec<u8>, usize> {
    if before.is_empty() {
        return Err(0);
    }
    let found: Vec<usize> = hay
        .windows(before.len())
        .enumerate()
        .filter(|(_, w)| *w == before)
        .map(|(i, _)| i)
        .take(2)
        .collect();
    let [at] = found[..] else {
        return Err(found.len());
    };
    let mut out = Vec::with_capacity(hay.len() + after.len());
    out.extend_from_slice(&hay[..at]);
    out.extend_from_slice(after);
    out.extend_from_slice(&hay[at + before.len()..]);
    Ok(out)
}

/// `Delete`: the target is present.
pub struct DeleteHandler;

impl ActionHandler for DeleteHandler {
    fn handles(&self, action: &Action) -> bool {
        matches!(action.kind, ActionKind::Delete { .. })
    }

    fn simulate(
        &self,
        action: &Action,
        _source: &PlanSource,
        tree: &mut VirtualTree<'_>,
    ) -> Result<Simulated, Finding> {
        let ActionKind::Delete { path, via_git } = &action.kind else {
            return Err(wrong_kind(action));
        };
        let span = Some(action.span);
        if !presence(tree, path, span)? {
            return Err(missing(path, span, "; the plan deletes it"));
        }
        tree.delete(path, span)?;
        Ok(Simulated {
            path: path.clone(),
            effect: Effect::Delete { via_git: *via_git },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::replace_once;

    #[test]
    fn replace_needs_exactly_one_occurrence() {
        assert_eq!(replace_once(b"a b a", b"b", b"c"), Ok(b"a c a".to_vec()));
        assert_eq!(replace_once(b"a b a", b"a", b"c"), Err(2));
        assert_eq!(replace_once(b"a b a", b"x", b"c"), Err(0));
        assert_eq!(replace_once(b"a", b"", b"c"), Err(0));
    }
}
