//! Builds IR by hand for planrun-exec's tests: exec never parses Markdown (D2). The text of
//! every action is appended to one source, and each action's span is its own text.

#![allow(dead_code)]

use planrun_model::{
    AbbrevBlobId, Action, ActionKind, ChangeKind, FormatId, Manifest, ManifestEntry, ManifestPath,
    ManualReason, PathOrigin, Plan, PlanMeta, PlanSource, RecognizerId, RepoPath, Script,
    ScriptIntent, Shell, Span, Step, Task, TaskId, TextRef, WriteMode,
};

/// A repository path; panics on a bad spelling.
pub fn path(p: &str) -> RepoPath {
    RepoPath::new(p).unwrap_or_else(|e| panic!("{e}"))
}

/// A blob id abbreviation; panics on bad hex.
pub fn abbrev(hex: &str) -> AbbrevBlobId {
    AbbrevBlobId::parse(hex).unwrap_or_else(|e| panic!("{e}"))
}

/// The plan text being built.
#[derive(Default)]
pub struct Ir {
    text: String,
}

impl Ir {
    /// Appends `s` and a newline; returns `s`'s span.
    pub fn text(&mut self, s: &str) -> Span {
        let start = u32::try_from(self.text.len()).unwrap_or(u32::MAX);
        self.text.push_str(s);
        let end = u32::try_from(self.text.len()).unwrap_or(u32::MAX);
        self.text.push('\n');
        Span::new(start, end)
    }

    fn action(&mut self, kind: impl FnOnce(&mut Self) -> ActionKind, origin: PathOrigin) -> Action {
        let start = u32::try_from(self.text.len()).unwrap_or(u32::MAX);
        let kind = kind(self);
        let end = u32::try_from(self.text.len()).unwrap_or(u32::MAX);
        Action {
            kind,
            span: Span::new(start, end),
            origin: RecognizerId("test".into()),
            path_origin: origin,
        }
    }

    /// `WriteFile`.
    pub fn write(&mut self, p: &str, body: &str, mode: WriteMode) -> Action {
        self.action(
            |ir| ActionKind::WriteFile {
                path: path(p),
                body: TextRef::Span(ir.text(body)),
                mode,
            },
            PathOrigin::Written,
        )
    }

    /// `ApplyPatch` with the ids of its `index` line.
    pub fn patch(&mut self, p: &str, patch: &str, pre: &str, post: &str) -> Action {
        self.action(
            |ir| ActionKind::ApplyPatch {
                path: path(p),
                patch: TextRef::Span(ir.text(patch)),
                pre: Some(abbrev(pre)),
                post: Some(abbrev(post)),
            },
            PathOrigin::Written,
        )
    }

    /// `ReplaceExact`.
    pub fn replace(&mut self, p: &str, before: &str, after: &str) -> Action {
        self.action(
            |ir| ActionKind::ReplaceExact {
                path: path(p),
                before: TextRef::Span(ir.text(before)),
                after: TextRef::Span(ir.text(after)),
            },
            PathOrigin::Written,
        )
    }

    /// `Delete`.
    pub fn delete(&mut self, p: &str, via_git: bool) -> Action {
        self.action(
            |_| ActionKind::Delete {
                path: path(p),
                via_git,
            },
            PathOrigin::Written,
        )
    }

    /// `Generated`.
    pub fn generated(&mut self, p: &str) -> Action {
        self.action(
            |_| ActionKind::Generated { path: path(p) },
            PathOrigin::Written,
        )
    }

    /// A pwsh `RunScript`.
    pub fn script(&mut self, body: &str) -> Action {
        self.action(
            |ir| {
                ActionKind::RunScript(Script {
                    shell: Shell::Pwsh,
                    body: TextRef::Span(ir.text(body)),
                    expect: None,
                    intent: ScriptIntent::Verify,
                })
            },
            PathOrigin::Written,
        )
    }

    /// A pwsh `Commit`.
    pub fn commit(&mut self, body: &str) -> Action {
        self.action(
            |ir| {
                ActionKind::Commit(Script {
                    shell: Shell::Pwsh,
                    body: TextRef::Span(ir.text(body)),
                    expect: None,
                    intent: ScriptIntent::Other,
                })
            },
            PathOrigin::Written,
        )
    }

    /// A `Note`.
    pub fn note(&mut self, text: &str) -> Action {
        self.action(
            |ir| ActionKind::Note {
                text: TextRef::Span(ir.text(text)),
            },
            PathOrigin::Written,
        )
    }

    /// A `Manual(ProseEdit)`.
    pub fn manual(&mut self, text: &str) -> Action {
        self.action(
            |ir| ActionKind::Manual {
                reason: ManualReason::ProseEdit,
                text: TextRef::Span(ir.text(text)),
            },
            PathOrigin::Written,
        )
    }

    /// The source of everything built so far.
    pub fn source(&self) -> PlanSource {
        PlanSource::from_normalized("test.md", self.text.clone(), "0".to_owned())
    }

    /// A manifest entry: `(path, change, origin)`.
    pub fn manifest(&mut self, entries: &[(&str, ChangeKind, PathOrigin)]) -> Manifest {
        let entries = entries
            .iter()
            .map(|(p, change, origin)| ManifestEntry {
                path: ManifestPath::File(path(p)),
                change: *change,
                lines: None,
                span: self.text(p),
                origin: *origin,
            })
            .collect();
        Manifest {
            entries,
            span: Some(Span::new(0, 0)),
        }
    }

    /// A task with one step per action list.
    pub fn task(&mut self, id: &str, manifest: Manifest, steps: Vec<Vec<Action>>) -> Task {
        let title = self.text(id);
        let steps = steps
            .into_iter()
            .zip(1u32..)
            .map(|(actions, n)| Step {
                number: Some(n),
                title: None,
                span: title,
                actions,
            })
            .collect();
        Task {
            id: TaskId {
                raw: id.into(),
                ordinal: 1,
            },
            title: TextRef::Span(title),
            span: title,
            manifest,
            steps,
        }
    }

    /// The plan of `tasks`, over everything built so far.
    pub fn plan(&self, tasks: Vec<Task>) -> Plan {
        Plan {
            source: self.source(),
            format: FormatId("test".into()),
            meta: PlanMeta::default(),
            tasks,
        }
    }
}

/// The P1-T1 patch of `tests/corpus/synthetic/assay-style.md` (aa80cf1 → 97bd92a).
pub const PATCH_T1: &str = "diff --git a/src/lib.rs b/src/lib.rs
index aa80cf1..97bd92a 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,8 +1,9 @@
 //! A tiny fixture crate for the planrun corpus.
\x20
+pub mod greet;
 pub mod old;
\x20
 /// The greeting for `name`.
 pub fn greeting(name: &str) -> String {
-    old::legacy_greeting(name)
+    greet::greeting(name)
 }
";

/// The P1-T2 patch of `tests/corpus/synthetic/assay-style.md` (97bd92a → 6b269fc).
pub const PATCH_T2: &str = "diff --git a/src/lib.rs b/src/lib.rs
index 97bd92a..6b269fc 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,9 +1,5 @@
 //! A tiny fixture crate for the planrun corpus.
\x20
 pub mod greet;
-pub mod old;
\x20
-/// The greeting for `name`.
-pub fn greeting(name: &str) -> String {
-    greet::greeting(name)
-}
+pub use greet::greeting;
";
