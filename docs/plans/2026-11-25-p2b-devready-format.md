# planrun P2b — The devready format

**Goal:** pattern manifest entries first (handover open item 1), then the schema and the recognizer parameters that hand-written DevReady plans need, then `formats/devready.toml` on them (docs/roadmap.md P2b, decisions D41–D49). After P2b, all six local plans parse with zero errors; the two devready plans have no actions behind prose they do not state, and every fence planrun would run sits behind an allowlisted run lead.

- **P2b-T1** (`planrun-model`, `planrun-parse`, `planrun-cli`, `formats`, docs): pattern manifest entries (D41); `RepoPath` refuses wildcards (D42); preflight skips patterns (architecture §6.1).
- **P2b-T2** (`planrun-model`, `planrun-parse`, `planrun-cli`, docs, `CLAUDE.md`): the manifest module; inline Files clauses (D43); a change per path (D44); Bare file names and `Manual(AmbiguousPath)` (D45); review item 6.
- **P2b-T3** (`planrun-model`, `planrun-parse`, `planrun-cli`, `formats`, docs): kept step labels (D46); `Expected:` with a list (D47, open item 5); comment expectations that a sentence extends (D48); numbered edit series (D49).
- **P2b-T4** (`formats`, `planrun-parse`, `planrun-cli`, corpus): the devready format, its synthetic plan and IR snapshot.
- **P2b-T5** (docs, `CLAUDE.md`): plan-grammar §4 and the Apply invariant in §5 (open item 4), the roadmap, the review-seat rule.

**Base:** `9b92ffa` (`docs: P2a done, handover P2a to P2b`), the child of the handover's `e8e0fdc` that adds only the handover and the roadmap's P2b paragraph; execute from the commit of this plan. Gate per task: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Shell and environment:** PowerShell 7 on Windows; Rust 1.95.0 (stable). No new crate, and `Cargo.lock` does not change.

**Before P2b-T1:** this plan committed, and a clean `git status --porcelain`. The real plans stay in `tests/corpus/local/` (git-ignored, D16); this plan describes them by file, task and step and quotes none of them.

**How this plan was verified:** every task was built in a scratch clone of `9b92ffa` outside this repository (`core.autocrlf=false`), one commit per task. At each commit `cargo fmt --check`, clippy and `cargo test` pass, and so does the task's targeted command; the summary lines below come from those runs. The plan was generated from that history (full contents from `git show`, exact diffs from `git diff`, blob ids from `git rev-parse`), `planrun check` passes on it, and it was replayed task by task on a fresh clone with `core.autocrlf=true`: every commit step committed, and `git status --porcelain` was empty after each.

## Review rulings (P2b questions, 2026-11-25)

Each accepted answer is a decision in docs/decisions.md; this list maps them.

1. **Q0** base and P3c-1: see stated decision 1.
2. **Q1** inline Files clauses with `items`, `clause_list` and the same `item` regex; leftover text is a Note at the start of the intro step, counted once as manifest; a list after an inline marker is `manifest-item` (D43).
3. **Q2** `path_change` per path, values through `changes`, precedence path > item > marker (D44). The handover's item-level mapping was dropped, and D44 says why.
4. **Q3** `PathOrigin::Bare` instead of root entries; never resolved to; counts toward `infer_path` ambiguity (Q3'); a Create onto it is `Manual(AmbiguousPath)`, produced only by that rule (Q3''); D33 treats it as non-literal (D45).
5. **Q4** no `infer_path` change; a numbered edit series continues the path of the edit directly before it (D49).
6. **Q5** a `PathPattern` target type; `RepoPath` refuses `*` and `?` (D42); ranges detected before `path_list`, in every format, and the en dash removed from `before-after.toml`'s `path_list`; edge cases defined and tested; preflight in architecture §6.1 (D41).
7. **Q6** `expected-prose` `list = true`; nested paragraphs, list items and `text` fences join, any other fence ends it (D47).
8. **Q7** a bare `` `p`: `` lead writes with op `create`.
9. **Q8** an `expected-prose` after a fence whose only expectation is its comment lines extends it, the two parts labelled apart; `expected-block` there and a second `Expected:` stay `expected-twice` (D48, D35 superseded in part).
10. **Defaults:** `keep` per step row, no `n` capture (D46); the run-lead allowlist; Verify with several commands is `Manual(HumanCheck)`, with one a pwsh run line; `Commit: `…`` a pwsh run line with the commit intent; `[checks]` untouched off, undeclared warning; detection as proposed; plan-grammar §4 rewritten.
11. **`(run from …):` leads** are Manual, and so is the fence after the apply described in prose (stated decision 6). The P4 candidate is in the roadmap.

## Stated decisions made while building

1. **Q0.** `9b92ffa` is the direct child of `e8e0fdc` and changes only `docs/handovers/2026-11-24-p2a-to-p2b.md` and `docs/roadmap.md`. P3c-1's `manual-outside-tasks` is a note, not a warning, and was already there in P1: `outside_notes` (`lib.rs`) and `formats/assay.toml` are unchanged since `12361bd` (P1-T5); the local-corpus test only started printing diagnostics in P2a-T7.
2. **Ranges are found on the item's paths before `path_list` splits them** (Q5(b)): the en dash left `before-after.toml`'s `path_list` and stays in its `item` regex. The only en dash between code spans in the before-after corpus (the cleanup plan and `before-after-style.md`) is the range item: `planrun show` on the cleanup plan differs from `9b92ffa` in that one entry only (`Modify <dir>/0001–0007_*.sql (a pattern: never a write target)` instead of two entries), and its one `manifest-undeclared` warning is gone. No existing snapshot changes in P2b-T1 to P2b-T3.
3. **A pattern ends the sibling context.** A token after a pattern entry in the same item is not its sibling and gets no `:lines` from it (`` `db/a.cs`, `db/m/0001`–`0003.sql` and `b.cs` `` gives `b.cs` at the root); for D45 a pattern with a directory counts as "a token with a directory".
4. **The manifest code moved to `crates/planrun-parse/src/manifest.rs`** (P2b-T2): `outline.rs` was over 600 lines and the inline clauses, `path_change` and Bare would have added 200. The move is unchanged code first; CLAUDE.md's Layout names the module.
5. **D49 reads the previous edit's number with a `series` regex, not with its lead's `n` capture** (review wording of Q4). Evidence: the edit with the path (``\`p\`, edit 1 (old → new):``) is matched by one `before-after` instance and the continuation (`Edit 2 (old → new):`) by another; a recognizer gets only its own captures and the actions, and must stay pure (extending.md §5), so it cannot see the other instance's `n`. The continuation lead must still capture `n` itself, and the spec load refuses `series` without it, as ruled. `series` reads the number from the previous action's lead paragraph.
6. **`(run from …):` (rules-logging Task 2, step 2b).** The fence body sets no location (no `Set-Location`/`Push-Location`). The review's premise that the stated directory differs from where sessions start does not hold for this plan: the directory variable is the plan's name for the repository root (its Repo/branch line, and the convention that every block starts from that root). It stays Manual anyway, for two other reasons: a spec cannot know that a variable names the root (the synthetic plan uses another variable), and the verify fence after the apply described in prose uses a variable that only the preview fence defines, so running it without the preview would search the whole repository. Both are `Manual(HumanCheck)` (`manual-prose#run-from`, `manual-prose#apply-in-prose`); the stated directory is shown with the fence.
7. **The two `(Bash tool):` claims in plugin-core Task 1** are kept as claims: step 1a is a read-only probe (the CLI's version and help texts); step 1c writes an empty helper script into the OS temp directory and runs the module's test suites, which write nowhere else. Step 1b installs and removes a throwaway plugin in the user's global configuration and is not claimed (its label has no `(Bash tool):` at the end), nor are rules-logging Task 1's two `(Bash tool)` labels without a colon.
8. **Manual markers.** "record" counts only as an instruction (at a sentence or clause start, or after a bold label): rules-logging Task 2 has "the record's message", which is no instruction and stays a Note. "don't" is matched in any case: rules-logging Task 5 has a lowercase "don't edit the fixture".
9. **Detection hits** (the devready weights are 30, 20, 30, 20, 20; the score is capped at 100; the threshold is 50):

   | Plan | devready hits | devready | superpowers | before-after | assay |
   |---|---|---|---|---|---|
   | plugin-core | 30+20+30+20+20 = 120 | 100 | 40 | 0 | 0 |
   | rules-logging | 30+20+30+20+20 = 120 | 100 | 40 | 0 | 0 |
   | `devready-style.md` | 30+20+30+20+20 = 120 | 100 | 40 | 0 | 0 |
   | heatmap, `superpowers-style.md` | 30 | 30 | 100 | 40 | 0 |
   | cleanup, `before-after-style.md` | 0 | 0 | 40 | 100 | 0 |
   | the assay plans | 0 | 0 | 0 | 0 | 100 |

   Each devready plan reaches 50 without the cap, and without the task pattern too (90).
10. **Real plans after P2b-T4:** all six parse with zero errors. Diagnostics: plugin-core 9 `manual-outside-tasks` notes (VERIFY, "record" and "don't" lines in its preamble and in the sections after the tasks); rules-logging 4 such notes and 2 `manifest-undeclared` warnings (stated decision 11); P3c-1 its one note; heatmap, routes-seam and cleanup none.

    | plugin-core task | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 |
    |---|---|---|---|---|---|---|---|---|---|---|---|---|
    | actions | 1 | 2 | 9 | 1 | 13 | 6 | 19 | 4 | 7 | 6 | 14 | 1 |
    | notes | 3 | 13 | 4 | 4 | 2 | 2 | 3 | 2 | 3 | 3 | 2 | 2 |
    | manual | 1 | 10 | 1 | 7 | 2 | 1 | 1 | 2 | 2 | 1 | 10 | 4 |

    | rules-logging task | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
    |---|---|---|---|---|---|---|---|---|---|---|
    | actions | 0 | 0 | 16 | 10 | 31 | 14 | 10 | 10 | 6 | 0 |
    | notes | 2 | 19 | 8 | 1 | 8 | 3 | 5 | 2 | 11 | 11 |
    | manual | 2 | 11 | 6 | 0 | 30 | 2 | 32 | 12 | 4 | 1 |

11. **Bare entries (Q3).** plugin-core Task 10: `CLAUDE.md` after the utils README path; its three edits name `CLAUDE.md` and become ReplaceExact (Modify stays an action); the task's three README edits have no path and stay `Manual(ProseEdit)` (README and the Bare entry are two candidates). rules-logging Task 5: the two test files named by bare file name; their create leads name the full paths, so they are Creates of undeclared paths (two `manifest-undeclared` warnings), and the Bare entries are untouched (the check is off). rules-logging Task 7: `CLAUDE.md`; its first edit names it and is a ReplaceExact, the others have no path and are Manual. `devready-style.md` Task 2: `README.md`; its edit names it and is a ReplaceExact. No real plan creates onto a Bare entry; `manifest_inline.rs` tests the `Manual(AmbiguousPath)` rule.
12. **Edit series (Q4).** plugin-core Task 6: edit 1 names its file by file name (`FileName`), edits 2–4 continue it (`PreviousEdit`). Task 8: edit 1 has no path and takes the task's only entry (`OnlyEntry`), edit 2 continues it. Task 10: README edits 2 and 3 follow a Manual, so they do not continue anything and stay Manual.
13. **`Expected:` with a list (Q6).** Five occurrences in the devready tasks (plugin-core Tasks 4, 6, 7, 10; rules-logging Task 8) and cleanup Task 6 step 2: each follows a claimed script (a Verify label, a `(Bash tool):` lead, a `Preview:` lead). The sections after the tasks hold three more (one in plugin-core, two in rules-logging); they are never parsed. In cleanup Task 6 step 2 the paragraph and its three list items are now the preview script's prose expectation instead of four Notes (the only change in that plan from P2b-T3).
14. **D48 count.** One claimed fence has comment expectations and an `Expected:` after it: rules-logging Task 8's full-suite fence (a `(Bash tool):` lead), followed by an `Expected:` list. It is extended, not `expected-twice`. `tests/corpus/invalid/expected-twice.md` (two expectations after one assay script) still yields `expected-twice`; the code's trigger is narrower, not wider, so no invalid-corpus file is added.
15. **Fences the devready `script` claims, by lead form:**
    - plugin-core: a Verify label in Tasks 0, 2, 4, 5, 6, 7, 8, 9, 10, 11; `**Run + verify:**` in Task 3; `(Bash tool):` in Task 1 steps 1a and 1c (stated decision 7).
    - rules-logging: a Verify label in Tasks 2, 5, 6, 7; `**4l. Generate and verify.**` in Task 4; `Commit:` + fence in Tasks 2, 4, 5, 6, 8; `(Bash tool):` in Task 8. The run lines: Task 3's one-command Verify label, and the `Commit: `…`` lines of Tasks 3 and 7.
    - No fence is claimed behind a bare sub-label: the other sub-label fences are file writes and edits, prose edits (Manual), or `Manual(Unrecognized)`.
16. **rules-logging Task 1** (D40) has no action: its probe fence and its baseline fence are `Manual(Unrecognized)` (labels without a run lead), its VERIFY, record and "stop and report" paragraphs and list items are `Manual(Marked)`, and its labels, its expected outcomes and two branch items are Notes (listed below).
17. **Comment expectations** (D35, D48) are confirmed, never matched: the body runs unchanged, so a remark after a `#` or a `#` inside a string only adds a line to what `show` and the confirmation print. `show` prints "N comment expectation(s) (confirmed by you)" after the sentence; the IR carries them apart from the sentence (`comments`).
18. **Snapshots.** The only snapshot change in P2b is the new `devready_style_ir.snap` (P2b-T4). `ManifestPath`'s `Display` and `Debug` print what the plan wrote, and `ExpectView.comments` is skipped when empty, so the assay, superpowers and before-after IR snapshots, the `show` snapshot and the CLI check snapshots are unchanged.
19. **Diffs only for modified files over 300 lines**, as in P1 and P2a; every other file is full contents, new files included. Every fence is one backtick longer than any backtick run at 0–3 spaces of indentation in its body.

## Imperative Notes left in the real plans (D36)

Checked with `planrun show` on the P2b-T4 build. Text is not quoted (D16). Kept step labels without a lead of their own (`**Verify:**`, `**4j.** …`) and `---` rules are Notes and are not listed.

- `2026-11-09-p2-plugin-core.md`:
  - Task 1, step 1a: the last item of the record list (whether a CLI option exists); the record paragraph before it is Manual.
  - Task 1, step 1b: the `Expected and recorded:` paragraph; the item that asks to paste command output; the item that asks to adjust Task 6 when a file shape differs; the cleanup item. The VERIFY and record items in that list are Manual.
  - Task 3, step Run + verify: the two items of the check-by-eye list (the paragraph is Manual).
  - Task 6, intro: the paragraph that asks to add an option and adjust a function when Task 1 found so.
  - Leftovers of inline manifests (D43): Task 3 and Task 6 explain what the files generate or export; Task 11's is an instruction (to regenerate files), and the generator fence it refers to is part of the Manual inline edit before it.
- `2026-11-09-p4-rules-logging.md`:
  - Task 1: the paragraph that asks to read the probe's table and reply; the expected-outcome items under 1b and 1d; branch item B2 (a conditional edit of a later task); the cleanup line with its remove command.
  - Task 2, step Verify: the paragraph that asks to run one more command without an explicit import.
  - Task 4, step 4k: the paragraph that describes an edit in three calls; its fence (a replace command) is `Manual(Unrecognized)`.
  - Task 8: the paragraph that announces the timing script (its fence is Manual); the budget list items (expectations).
  - Task 9: the report list (what to reply with), D40.
- `2026-11-19-cleanup-batched-runner.md`: as in P2a, except Task 6 step 2, which is now an expectation (stated decision 13).
- `2026-09-19-heatmap-mode-toggle.md`: none.

## Tasks

### P2b-T1: pattern manifest entries (D41), RepoPath refuses wildcards (D42)

**Goal:** Handover open item 1, first: a range or a glob in a Files item is one manifest entry whose target is a `PathPattern`, never a write target. `ManifestEntry.path` becomes a `ManifestPath` (a file or a pattern) whose `Display` and `Debug` print what the plan wrote, so no existing snapshot changes. Ranges are found before `path_list` splits an item, in every format, and the en dash leaves `before-after.toml`'s `path_list`. `manifest-undeclared` and `manifest-untouched` match paths against pattern entries, which clears the cleanup plan's one warning. `RepoPath` refuses `*` and `?`. Preflight skips pattern entries (architecture §6.1).

**Files**

- Modify `crates/planrun-cli/src/view.rs`
- Modify `crates/planrun-model/src/lib.rs`
- Modify `crates/planrun-model/src/path.rs`
- Create `crates/planrun-model/src/pattern.rs`
- Modify `crates/planrun-model/src/plan.rs`
- Modify `crates/planrun-parse/src/check.rs`
- Modify `crates/planrun-parse/src/kinds/file_block.rs`
- Modify `crates/planrun-parse/src/kinds/mod.rs`
- Modify `crates/planrun-parse/src/outline.rs`
- Create `crates/planrun-parse/tests/manifest_patterns.rs`
- Modify `docs/architecture.md`
- Modify `docs/decisions.md`
- Modify `docs/extending.md`
- Modify `docs/plan-grammar.md`
- Modify `formats/before-after.toml`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/src/view.rs`, `crates/planrun-parse/src/check.rs`, `crates/planrun-parse/src/outline.rs`, `docs/architecture.md`, `docs/decisions.md`.

**Step 1 — write the files**

File `crates/planrun-cli/src/view.rs` (exact diff; save the block as `$env:TEMP\p2b-t1-view-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/view.rs b/crates/planrun-cli/src/view.rs
index f77a9f0..f688973 100644
--- a/crates/planrun-cli/src/view.rs
+++ b/crates/planrun-cli/src/view.rs
@@ -2,8 +2,8 @@
 //! a line (`L37`). `planrun show` prints it; the IR snapshot is its YAML.
 
 use planrun_model::{
-    Action, ActionKind, ExitExpect, Expectation, ManifestEntry, PathOrigin, Plan, Script, Span,
-    Step, Task, TextRef,
+    Action, ActionKind, ExitExpect, Expectation, ManifestEntry, ManifestPath, PathOrigin, Plan,
+    Script, Span, Step, Task, TextRef,
 };
 use serde::Serialize;
 
@@ -310,12 +310,12 @@ fn entry(entry: &ManifestEntry) -> String {
         .lines
         .map(|l| format!(":{}-{}", l.start, l.end))
         .unwrap_or_default();
-    let sibling = if entry.origin.is_written() {
-        ""
-    } else {
-        " (in the directory of the path before)"
+    let note = match (&entry.path, entry.origin) {
+        (ManifestPath::Pattern(_), _) => " (a pattern: never a write target)",
+        (_, PathOrigin::Written) => "",
+        _ => " (in the directory of the path before)",
     };
-    format!("{:?} {}{lines}{sibling}", entry.change, entry.path)
+    format!("{:?} {}{lines}{note}", entry.change, entry.path)
 }
 
 /// How much of each action `show` prints.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t1-view-rs.patch"
Remove-Item "$env:TEMP\p2b-t1-view-rs.patch"
```

File `crates/planrun-model/src/lib.rs` (full contents):

```rust
//! The typed plan IR (docs/architecture.md §3).
//!
//! Parsing produces a [`Plan`]; execution consumes it. Neither side depends on the other: this
//! crate is the contract between them. Text is never copied out of the plan: every piece of text
//! is a [`TextRef`] into the normalized plan source held by [`PlanSource`].
//!
//! The action set ([`ActionKind`]) is closed on purpose (decision D4): plan formats vary,
//! actions don't. A new format maps its own wording onto these actions.

mod action;
mod expect;
mod ids;
mod path;
mod pattern;
mod plan;
mod source;
mod span;

pub use action::{Action, ActionKind, ManualReason, Script, ScriptIntent, Shell, WriteMode};
pub use expect::{ExitExpect, Expectation};
pub use ids::{AbbrevBlobId, BlobIdError, FormatId, RecognizerId, TaskId};
pub use path::{PathOrigin, RepoPath, RepoPathError};
pub use pattern::{ManifestPath, PathPattern, PatternError};
pub use plan::{ChangeKind, LineRange, Manifest, ManifestEntry, Plan, PlanMeta, Step, Task};
pub use source::PlanSource;
pub use span::{LineCol, Span, TextRef};
```

File `crates/planrun-model/src/path.rs` (full contents):

```rust
//! Repository-relative paths: the only path type an action may carry (decision D7).

use std::fmt;

use serde::{Deserialize, Serialize};

/// A path relative to the repository root, with forward slashes, that cannot leave the
/// repository by its spelling: no absolute path, no drive letter, no `..`, no `.` segment, no
/// empty segment, no backslash, no NUL, no wildcard (`*`, `?`: decision D42).
///
/// This checks the spelling only. Symlinks that point out of the repository are refused by the
/// workspace when it resolves the path on disk (planrun-exec).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RepoPath(Box<str>);

/// Why a path was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RepoPathError {
    /// Empty input.
    #[error("the path is empty")]
    Empty,
    /// `/x`, `C:\x`, `C:x` or `\\server\x`.
    #[error("`{0}` is absolute; plan paths are relative to the repository root")]
    Absolute(String),
    /// A `\` in the path.
    #[error("`{0}` contains a backslash; plan paths use forward slashes")]
    Backslash(String),
    /// A `..`, `.` or empty segment.
    #[error("`{0}` has a `.`, `..` or empty segment")]
    BadSegment(String),
    /// A NUL byte.
    #[error("the path contains a NUL byte")]
    Nul,
    /// A `*` or `?`: a pattern, never a file (decisions D41, D42).
    #[error("`{0}` contains a wildcard; a pattern is not a file path")]
    Wildcard(String),
}

impl RepoPath {
    /// Validates `path` as written in a plan.
    pub fn new(path: &str) -> Result<Self, RepoPathError> {
        if path.is_empty() {
            return Err(RepoPathError::Empty);
        }
        if path.contains('\0') {
            return Err(RepoPathError::Nul);
        }
        if path.contains('\\') {
            return Err(RepoPathError::Backslash(path.to_owned()));
        }
        if path.contains(['*', '?']) {
            return Err(RepoPathError::Wildcard(path.to_owned()));
        }
        let bytes = path.as_bytes();
        if path.starts_with('/') || (bytes.len() >= 2 && bytes[1] == b':') {
            return Err(RepoPathError::Absolute(path.to_owned()));
        }
        if path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        {
            return Err(RepoPathError::BadSegment(path.to_owned()));
        }
        Ok(Self(path.into()))
    }

    /// The path with forward slashes.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The last segment (`lib.rs` of `src/lib.rs`).
    pub fn file_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }

    /// The directory, without a trailing slash; `None` at the repository root.
    pub fn dir(&self) -> Option<&str> {
        self.0.rsplit_once('/').map(|(dir, _)| dir)
    }
}

/// Where a path in the IR came from, when the plan did not write it in full (decision D32).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathOrigin {
    /// Written in full where it is used.
    #[default]
    Written,
    /// A manifest token without a directory, placed in the directory of the token before it.
    Sibling,
    /// A lead that names only a file name: the one manifest entry with that file name.
    FileName,
    /// A lead that names no path: the task's one editable manifest entry.
    OnlyEntry,
}

impl PathOrigin {
    /// True when the plan wrote the path in full.
    pub fn is_written(&self) -> bool {
        *self == PathOrigin::Written
    }
}

impl TryFrom<String> for RepoPath {
    type Error = RepoPathError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<RepoPath> for String {
    fn from(value: RepoPath) -> Self {
        value.0.into()
    }
}

impl fmt::Display for RepoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_are_accepted() {
        for ok in [
            "a.rs",
            "crates/x/src/lib.rs",
            ".claude-plugin/plugin.json",
            "a/.x",
        ] {
            assert_eq!(RepoPath::new(ok).map(|p| p.to_string()), Ok(ok.to_owned()));
        }
    }

    #[test]
    fn escaping_or_ambiguous_paths_are_refused() {
        for bad in [
            "",
            "/etc/passwd",
            "C:\\x",
            "C:x",
            "a\\b",
            "../x",
            "a/../b",
            "./a",
            "a//b",
            "a/",
            "a/*.rs",
            "a/b?.rs",
        ] {
            assert!(RepoPath::new(bad).is_err(), "{bad:?}");
        }
    }
}
```

File `crates/planrun-model/src/pattern.rs` (full contents):

```rust
//! Manifest patterns (decision D41): a glob or a range a plan writes in a Files list instead of
//! one file. A pattern is never a write target: actions carry a [`RepoPath`], which refuses
//! wildcards (D42), and a manifest entry is either a file or a pattern ([`ManifestPath`]).

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::path::RepoPath;

/// A glob (`dir/*.sql`, `functions/{a,b}.ps1`) or a range (`dir/0001`–`0007_*.sql`), relative to
/// the repository root as written and never resolved to files (D41).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PathPattern {
    text: Box<str>,
    kind: PatternKind,
}

/// The two kinds of pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum PatternKind {
    /// The alternatives of the glob after brace expansion.
    Glob(Vec<Box<str>>),
    /// Every file in `dir` whose name lies between `start` and `end`.
    Range {
        dir: Box<str>,
        start: Box<str>,
        end: Box<str>,
    },
}

/// Why a pattern was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PatternError {
    /// The spelling is no repository path (absolute, `..`, backslash, …).
    #[error("`{0}` is not a repository-relative pattern")]
    Spelling(String),
    /// `**`, which would cross directories.
    #[error("`{0}` uses `**`; a pattern matches within one directory level")]
    DoubleStar(String),
    /// `{` inside `{…}`, or an unbalanced brace.
    #[error("`{0}` has nested or unbalanced braces")]
    Braces(String),
    /// A range whose ends are in different directories, or whose start has a wildcard.
    #[error("the range `{0}`–`{1}` must name two file names in one directory")]
    Range(String, String),
}

impl PathPattern {
    /// True when a manifest token is a glob: it contains `*`, `?` or `{`.
    pub fn is_glob(token: &str) -> bool {
        token.contains(['*', '?', '{'])
    }

    /// A glob. `*` matches any run of characters except `/`, `?` one character except `/`, and
    /// `{a,b}` either alternative (no nesting).
    pub fn glob(text: &str) -> Result<Self, PatternError> {
        if text.contains("**") {
            return Err(PatternError::DoubleStar(text.to_owned()));
        }
        let alternatives = expand_braces(text)?;
        for alternative in &alternatives {
            check_spelling(alternative)?;
        }
        Ok(Self {
            text: text.into(),
            kind: PatternKind::Glob(alternatives.into_iter().map(Into::into).collect()),
        })
    }

    /// A range from `start` (with a directory) to `end` (a file name in the same directory, or
    /// a path with that same directory). `start` has no wildcard; `end` may have one.
    pub fn range(start: &str, end: &str) -> Result<Self, PatternError> {
        let refused = || PatternError::Range(start.to_owned(), end.to_owned());
        let start_path = RepoPath::new(start).map_err(|_| refused())?;
        let dir = start_path.dir().ok_or_else(refused)?;
        let end_name = match end.rsplit_once('/') {
            Some((end_dir, name)) if end_dir == dir => name,
            Some(_) => return Err(refused()),
            None => end,
        };
        if end_name.is_empty() || end_name.contains("**") || end_name.contains(['{', '}']) {
            return Err(refused());
        }
        check_spelling(&format!("{dir}/{end_name}"))?;
        Ok(Self {
            text: format!("{start}–{end_name}").into(),
            kind: PatternKind::Range {
                dir: dir.into(),
                start: start_path.file_name().into(),
                end: end_name.into(),
            },
        })
    }

    /// The pattern as written (a range as `start–end`).
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Whether `path` is one of the files the pattern names.
    ///
    /// A range `dir/A`–`B` matches a path in `dir` whose file name `f` has `A <= f`. When `B` has
    /// no wildcard, also `f <= B`. Otherwise, with `B0` the part of `B` before its first
    /// wildcard: `f` has at least `|B0|` characters, its first `|B0|` characters are `<= B0`, and
    /// the rest of `f` matches the rest of `B` as a glob. Comparisons are by character.
    pub fn matches(&self, path: &RepoPath) -> bool {
        match &self.kind {
            PatternKind::Glob(alternatives) => {
                alternatives.iter().any(|a| glob_match(a, path.as_str()))
            }
            PatternKind::Range { dir, start, end } => {
                path.dir() == Some(&**dir) && in_range(start, end, path.file_name())
            }
        }
    }
}

impl fmt::Display for PathPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// What a manifest entry declares: one file, or the files a pattern names.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManifestPath {
    /// One file.
    File(RepoPath),
    /// A glob or a range (D41).
    Pattern(PathPattern),
}

impl ManifestPath {
    /// The file, when the entry names one.
    pub fn file(&self) -> Option<&RepoPath> {
        match self {
            ManifestPath::File(path) => Some(path),
            ManifestPath::Pattern(_) => None,
        }
    }

    /// Whether `path` is declared by this entry: equal to its file, or matched by its pattern.
    pub fn covers(&self, path: &RepoPath) -> bool {
        match self {
            ManifestPath::File(file) => file == path,
            ManifestPath::Pattern(pattern) => pattern.matches(path),
        }
    }
}

impl fmt::Display for ManifestPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestPath::File(path) => path.fmt(f),
            ManifestPath::Pattern(pattern) => pattern.fmt(f),
        }
    }
}

impl fmt::Debug for ManifestPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestPath::File(path) => path.fmt(f),
            ManifestPath::Pattern(pattern) => pattern.fmt(f),
        }
    }
}

/// The spelling rules of [`RepoPath`] with the wildcards replaced by a plain character.
fn check_spelling(text: &str) -> Result<(), PatternError> {
    let plain = text.replace(['*', '?'], "x");
    RepoPath::new(&plain)
        .map(|_| ())
        .map_err(|_| PatternError::Spelling(text.to_owned()))
}

/// `a{b,c}d` → `abd`, `acd`; at most one brace group per level, never nested.
fn expand_braces(text: &str) -> Result<Vec<String>, PatternError> {
    let bad = || PatternError::Braces(text.to_owned());
    let Some(open) = text.find('{') else {
        if text.contains('}') {
            return Err(bad());
        }
        return Ok(vec![text.to_owned()]);
    };
    let close = text[open..].find('}').map(|i| open + i).ok_or_else(bad)?;
    let inner = &text[open + 1..close];
    if inner.contains('{') {
        return Err(bad());
    }
    let (head, tail) = (&text[..open], &text[close + 1..]);
    let tails = expand_braces(tail)?;
    Ok(inner
        .split(',')
        .flat_map(|alt| tails.iter().map(move |t| format!("{head}{alt}{t}")))
        .collect())
}

/// `*` and `?` never match `/`.
fn glob_match(pattern: &str, text: &str) -> bool {
    let (p, t): (Vec<char>, Vec<char>) = (pattern.chars().collect(), text.chars().collect());
    glob_chars(&p, &t)
}

fn glob_chars(p: &[char], t: &[char]) -> bool {
    let Some((&first, rest)) = p.split_first() else {
        return t.is_empty();
    };
    match first {
        '*' => (0..=t.len())
            .take_while(|&k| k == 0 || t[k - 1] != '/')
            .any(|k| glob_chars(rest, &t[k..])),
        '?' => t.first().is_some_and(|&c| c != '/') && glob_chars(rest, &t[1..]),
        c => t.first() == Some(&c) && glob_chars(rest, &t[1..]),
    }
}

fn in_range(start: &str, end: &str, name: &str) -> bool {
    if name < start {
        return false;
    }
    let Some(wild) = end.find(['*', '?']) else {
        return name <= end;
    };
    let (literal, rest) = end.split_at(wild);
    let count = literal.chars().count();
    let chars: Vec<char> = name.chars().collect();
    if chars.len() < count {
        return false;
    }
    let head: String = chars[..count].iter().collect();
    let tail: String = chars[count..].iter().collect();
    head.as_str() <= literal && glob_match(rest, &tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(p: &str) -> RepoPath {
        RepoPath::new(p).unwrap_or_else(|e| panic!("{e}"))
    }

    fn glob(p: &str) -> PathPattern {
        PathPattern::glob(p).unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn globs_match_within_one_segment() {
        let g = glob("dir/*.sql");
        assert!(g.matches(&path("dir/a.sql")));
        assert!(!g.matches(&path("dir/sub/a.sql")));
        assert!(!g.matches(&path("other/a.sql")));
        assert!(glob("dir/a?.rs").matches(&path("dir/ab.rs")));
        assert!(!glob("dir/a?.rs").matches(&path("dir/a.rs")));
    }

    #[test]
    fn braces_expand_to_alternatives() {
        let g = glob("f/{text/replace,git/git*}.ps1");
        assert!(g.matches(&path("f/text/replace.ps1")));
        assert!(g.matches(&path("f/git/gitinfo.ps1")));
        assert!(!g.matches(&path("f/text/other.ps1")));
    }

    #[test]
    fn double_stars_and_nested_braces_are_refused() {
        assert!(matches!(
            PathPattern::glob("a/**/b.rs"),
            Err(PatternError::DoubleStar(_))
        ));
        assert!(matches!(
            PathPattern::glob("a/{b,{c,d}}.rs"),
            Err(PatternError::Braces(_))
        ));
        assert!(matches!(
            PathPattern::glob("a/{b.rs"),
            Err(PatternError::Braces(_))
        ));
        assert!(PathPattern::glob("../*.rs").is_err());
    }

    #[test]
    fn a_range_bounds_the_file_name() {
        let r = PathPattern::range("s/m/0001", "0007_*.sql").unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(r.as_str(), "s/m/0001–0007_*.sql");
        assert!(r.matches(&path("s/m/0002_initial.sql")));
        assert!(r.matches(&path("s/m/0007_last.sql")));
        assert!(!r.matches(&path("s/m/0008_after.sql")));
        assert!(!r.matches(&path("s/m/0000_before.sql")));
        assert!(!r.matches(&path("s/m/0003_x.txt")));
        assert!(!r.matches(&path("s/Other/0002_initial.sql")));
    }

    #[test]
    fn a_name_shorter_than_the_literal_end_is_outside() {
        let r = PathPattern::range("d/0001", "0007_*.sql").unwrap_or_else(|e| panic!("{e}"));
        assert!(!r.matches(&path("d/0002")));
    }

    #[test]
    fn an_end_without_wildcard_is_inclusive() {
        let r = PathPattern::range("d/a.rs", "c.rs").unwrap_or_else(|e| panic!("{e}"));
        assert!(r.matches(&path("d/b.rs")));
        assert!(r.matches(&path("d/c.rs")));
        assert!(!r.matches(&path("d/c.rsx")));
    }

    #[test]
    fn range_ends_share_a_directory() {
        assert!(PathPattern::range("d/0001", "d/0007").is_ok());
        assert!(PathPattern::range("d/0001", "e/0007").is_err());
        assert!(PathPattern::range("0001", "0007").is_err());
        assert!(PathPattern::range("d/0*", "0007").is_err());
    }

    #[test]
    fn manifest_paths_print_as_written() {
        let file = ManifestPath::File(path("a/b.rs"));
        assert_eq!(file.to_string(), "a/b.rs");
        assert_eq!(format!("{file:?}"), "RepoPath(\"a/b.rs\")");
        assert!(file.covers(&path("a/b.rs")));
        let pattern = ManifestPath::Pattern(glob("a/*.rs"));
        assert!(pattern.covers(&path("a/c.rs")));
        assert_eq!(pattern.file(), None);
    }
}
```

File `crates/planrun-model/src/plan.rs` (full contents):

```rust
//! Plan, task, step and the task's file manifest.

use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::ids::{FormatId, TaskId};
use crate::path::PathOrigin;
use crate::pattern::ManifestPath;
use crate::source::PlanSource;
use crate::span::{Span, TextRef};

/// A parsed, checked plan. Owns its source; everything else refers into it.
#[derive(Debug, Clone)]
pub struct Plan {
    /// The normalized plan text.
    pub source: PlanSource,
    /// The format that parsed it.
    pub format: FormatId,
    /// Plan-level facts.
    pub meta: PlanMeta,
    /// Tasks in document order.
    pub tasks: Vec<Task>,
}

impl Plan {
    /// The text of `text`.
    pub fn text<'a>(&'a self, text: &'a TextRef) -> &'a str {
        self.source.resolve(text)
    }
}

/// Plan-level facts. Phases are metadata, not a level of the tree (decision D5).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanMeta {
    /// The first level-1 heading.
    pub title: Option<TextRef>,
    /// The phase id when the format captures one (`P1`, `P3c-1`).
    pub phase: Option<Box<str>>,
    /// The base commit the plan says to start from, when stated.
    pub base_commit: Option<Box<str>>,
    /// Paragraphs before the first task that match the format's `manual_markers`: text the user
    /// must read before task 1.
    pub preamble: Vec<TextRef>,
    /// The text after the tasks (`## Part 1 green state`, …): shown, never run.
    pub after_tasks: Option<Span>,
}

/// One task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// Its id.
    pub id: TaskId,
    /// Its title.
    pub title: TextRef,
    /// Heading through the last block before the next task.
    pub span: Span,
    /// The declared files. Empty when the format has none for this task.
    pub manifest: Manifest,
    /// Steps in order. A format without step headings yields one implicit step.
    pub steps: Vec<Step>,
}

/// One step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    /// Its number, when written; must count up from 1 within the task.
    pub number: Option<u32>,
    /// Its title, when written.
    pub title: Option<TextRef>,
    /// Heading through the last block of the step.
    pub span: Span,
    /// Actions in order.
    pub actions: Vec<Action>,
}

/// The files a task declares: its write set (check rule `manifest`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Entries in order.
    pub entries: Vec<ManifestEntry>,
    /// The manifest block, when present.
    pub span: Option<Span>,
}

/// One declared file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// The file, or the files a pattern names (decision D41).
    pub path: ManifestPath,
    /// What the task does to it.
    pub change: ChangeKind,
    /// A line range written after the path (`CleanupStep.cs:3-10`); informational.
    pub lines: Option<LineRange>,
    /// The entry's text.
    pub span: Span,
    /// `Sibling` when the item named only a file name after a path with a directory.
    pub origin: PathOrigin,
}

/// What a task declares it does to a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeKind {
    /// New file.
    Create,
    /// Existing file, changed.
    Modify,
    /// Existing file, replaced whole.
    Rewrite,
    /// Removed.
    Delete,
    /// A test file (created or modified; the actions decide).
    Test,
    /// Produced by a command the task runs; no file action needed (decision D27).
    Generated,
}

/// `start-end`, 1-based inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineRange {
    /// First line.
    pub start: u32,
    /// Last line.
    pub end: u32,
}
```

File `crates/planrun-parse/src/check.rs` (exact diff; save the block as `$env:TEMP\p2b-t1-check-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/check.rs b/crates/planrun-parse/src/check.rs
index 2375dba..5a7a3e3 100644
--- a/crates/planrun-parse/src/check.rs
+++ b/crates/planrun-parse/src/check.rs
@@ -26,9 +26,10 @@ pub trait Rule: Send + Sync {
 /// - `task-order`: a repeated task id is `duplicate-task-id`; among the other tasks, ordinals
 ///   strictly increase (`task-order`).
 /// - `step-order`: written step numbers count up from 1 in each task.
-/// - `manifest`: every file action's path is declared (`manifest-undeclared`); every declared
-///   Create/Modify/Rewrite/Delete is touched by an action or a `Generated`
-///   (`manifest-untouched`); a `WriteFile` on a declared path has a mode that agrees with the
+/// - `manifest`: every file action's path is declared (`manifest-undeclared`), by an entry for
+///   that file or by a pattern entry that matches it (D41); every declared Create/Modify/
+///   Rewrite/Delete is touched by an action or a `Generated`, a pattern entry by any path it
+///   matches (`manifest-untouched`); a `WriteFile` on a declared path has a mode that agrees with the
 ///   entry: `FromManifest` resolves, Create is not on an existing file, Overwrite not on a new one
 ///   (`manifest-write-mode`, decision D26). A task without a manifest block is skipped.
 /// - `patch-shape`: a patch has at least one hunk and its hunks' line counts match their bodies.
@@ -162,7 +163,7 @@ impl Rule for ManifestRule {
 
 fn check_manifest(task: &Task, out: &mut Vec<Diagnostic>) {
     let manifest = task.manifest.span.unwrap_or(task.span);
-    let declared = |path: &RepoPath| task.manifest.entries.iter().find(|e| &e.path == path);
+    let declared = |path: &RepoPath| task.manifest.entries.iter().find(|e| e.path.covers(path));
     let mut touched: Vec<&RepoPath> = Vec::new();
     for action in actions(task) {
         let Some(path) = file_path(&action.kind) else {
@@ -199,7 +200,7 @@ fn check_manifest(task: &Task, out: &mut Vec<Diagnostic>) {
     }
     let must_touch = |c: ChangeKind| !matches!(c, ChangeKind::Test | ChangeKind::Generated);
     for entry in &task.manifest.entries {
-        if must_touch(entry.change) && !touched.contains(&&entry.path) {
+        if must_touch(entry.change) && !touched.iter().any(|p| entry.path.covers(p)) {
             out.push(Diagnostic::error(
                 "manifest-untouched",
                 Some(entry.span),
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t1-check-rs.patch"
Remove-Item "$env:TEMP\p2b-t1-check-rs.patch"
```

File `crates/planrun-parse/src/kinds/file_block.rs` (full contents):

```rust
//! `file-block`: a lead paragraph naming a path and an operation, then one fence →
//! `WriteFile` or `ApplyPatch`.

use std::sync::Arc;

use planrun_model::{Action, ActionKind, ChangeKind, Manifest, RecognizerId, RepoPath, WriteMode};
use regex::Regex;

use super::params::Params;
use super::{paragraph, resolved_lead_path};
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
        let by_file_name = p.opt_bool("file_name_from_manifest")?.unwrap_or(false);
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
            by_file_name,
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
    /// Resolve a bare file name against the manifest (decision D32).
    by_file_name: bool,
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
        let (path, path_origin) =
            match resolved_lead_path(cx, &caps, base, lead.span, self.by_file_name) {
                Ok(resolved) => resolved,
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
            path_origin,
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
        .find(|e| e.path.file() == Some(path))
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
```

File `crates/planrun-parse/src/kinds/mod.rs` (full contents):

```rust
//! The built-in recognizer kinds (docs/extending.md §3).

mod before_after;
mod ceremony;
mod delete;
mod expected;
mod file_block;
mod generated;
mod manual_prose;
mod params;
mod run_line;
mod script;

use std::sync::Arc;

use planrun_model::{
    Action, ActionKind, ChangeKind, ManifestEntry, PathOrigin, RepoPath, Span, TextRef,
};
use regex::Captures;

use crate::diagnostic::Diagnostic;
use crate::document::{Block, Located};
use crate::recognizer::{RecognizerFactory, StepContext};

pub use before_after::BeforeAfterFactory;
pub use ceremony::CeremonyFactory;
pub use delete::DeleteFactory;
pub use expected::{ExpectedBlockFactory, ExpectedProseFactory};
pub use file_block::FileBlockFactory;
pub use generated::GeneratedFactory;
pub use manual_prose::ManualProseFactory;
pub use run_line::RunLineFactory;
pub use script::ScriptFactory;

/// Every built-in factory, for [`crate::Registry::with_builtins`].
pub fn builtin_factories() -> Vec<Arc<dyn RecognizerFactory>> {
    vec![
        Arc::new(FileBlockFactory),
        Arc::new(CeremonyFactory),
        Arc::new(DeleteFactory),
        Arc::new(GeneratedFactory),
        Arc::new(ScriptFactory),
        Arc::new(ExpectedBlockFactory),
        Arc::new(ExpectedProseFactory),
        Arc::new(RunLineFactory),
        Arc::new(BeforeAfterFactory),
        Arc::new(ManualProseFactory),
    ]
}

/// The text of a paragraph block, if it is one.
fn paragraph<'a>(cx: &'a StepContext<'_>, block: &'a Located) -> Option<(&'a str, u32)> {
    let Block::Paragraph { text } = &block.block else {
        return None;
    };
    Some((cx.doc.text(text), text.span().start))
}

/// The span of capture `name` in text that starts at `base`.
fn capture_span(caps: &Captures<'_>, name: &str, base: u32) -> Option<Span> {
    let m = caps.name(name)?;
    Some(Span::new(base + offset(m.start()), base + offset(m.end())))
}

/// The capture `name` as a span reference.
fn capture_ref(caps: &Captures<'_>, name: &str, base: u32) -> Option<TextRef> {
    capture_span(caps, name, base).map(TextRef::Span)
}

/// The `path` capture as a repository path, or a `bad-path` error at the capture.
fn lead_path(caps: &Captures<'_>, base: u32, lead: Span) -> Result<RepoPath, Diagnostic> {
    let span = capture_span(caps, "path", base).unwrap_or(lead);
    let raw = caps.name("path").map_or("", |m| m.as_str());
    RepoPath::new(raw).map_err(|e| Diagnostic::error("bad-path", Some(span), e.to_string()))
}

/// The `path` capture as a repository path, resolved against the task's manifest when
/// `by_file_name` (decision D32): a path without `/` that is no entry itself becomes the one
/// entry with that file name, when there is exactly one and it was written in full.
fn resolved_lead_path(
    cx: &StepContext<'_>,
    caps: &Captures<'_>,
    base: u32,
    lead: Span,
    by_file_name: bool,
) -> Result<(RepoPath, PathOrigin), Diagnostic> {
    let path = lead_path(caps, base, lead)?;
    let entries = &cx.manifest.entries;
    let is_entry = entries.iter().any(|e| e.path.file() == Some(&path));
    if !by_file_name || path.as_str().contains('/') || is_entry {
        return Ok((path, PathOrigin::Written));
    }
    let same: Vec<(&RepoPath, &ManifestEntry)> = entries
        .iter()
        .filter_map(|e| e.path.file().map(|file| (file, e)))
        .filter(|(file, _)| file.file_name() == path.as_str())
        .collect();
    let first = same.first().map(|(file, _)| *file);
    let one = first.filter(|p| {
        same.iter()
            .all(|(file, e)| file == p && e.origin.is_written())
    });
    match one {
        Some(entry) => Ok((entry.clone(), PathOrigin::FileName)),
        None => Ok((path, PathOrigin::Written)),
    }
}

/// The task's one manifest entry an edit with no path can mean (decision D32): not Test,
/// Generated or Delete, and never a pattern (D41). `None` with none or several.
fn only_entry(cx: &StepContext<'_>) -> Option<RepoPath> {
    let mut editable = cx
        .manifest
        .entries
        .iter()
        .filter(|e| {
            !matches!(
                e.change,
                ChangeKind::Test | ChangeKind::Generated | ChangeKind::Delete
            )
        })
        .filter_map(|e| e.path.file());
    let first = editable.next()?;
    editable.all(|p| p == first).then(|| first.clone())
}

/// The script of the action an expectation amends: the last action, when it is a script.
fn amendable(cx: &StepContext<'_>, lead: Span) -> Result<Action, Diagnostic> {
    let script = cx
        .previous
        .last()
        .filter(|a| matches!(a.kind, ActionKind::RunScript(_) | ActionKind::Commit(_)));
    let Some(action) = script else {
        return Err(Diagnostic::error(
            "expected-without-script",
            Some(lead),
            "an expectation with no script directly before it",
        ));
    };
    let already = match &action.kind {
        ActionKind::RunScript(s) | ActionKind::Commit(s) => s.expect.is_some(),
        _ => false,
    };
    if already {
        return Err(Diagnostic::error(
            "expected-twice",
            Some(lead),
            "the script before this already has an expectation",
        )
        .with_related(action.span, "the script"));
    }
    Ok(action.clone())
}

fn offset(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
```

File `crates/planrun-parse/src/outline.rs` (exact diff; save the block as `$env:TEMP\p2b-t1-outline-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/outline.rs b/crates/planrun-parse/src/outline.rs
index e62f668..18742dc 100644
--- a/crates/planrun-parse/src/outline.rs
+++ b/crates/planrun-parse/src/outline.rs
@@ -3,8 +3,8 @@
 //! format may too.
 
 use planrun_model::{
-    ChangeKind, LineRange, Manifest, ManifestEntry, PathOrigin, PlanMeta, RepoPath, Span, TaskId,
-    TextRef,
+    ChangeKind, LineRange, Manifest, ManifestEntry, ManifestPath, PathOrigin, PathPattern,
+    PlanMeta, RepoPath, Span, TaskId, TextRef,
 };
 use regex::{Captures, Regex};
 
@@ -403,66 +403,121 @@ fn manifest_item(
         ));
         return;
     };
-    let pieces: Vec<&str> = match &spec.path_list {
-        Some(split) => split.split(paths.as_str()).collect(),
-        None => vec![paths.as_str()],
-    };
     let paths_span = Span::new(base + offset(paths.start()), base + offset(paths.end()));
     let mut previous: Option<(RepoPath, PathOrigin)> = None;
-    for piece in pieces.iter().map(|p| p.trim()) {
-        let Some((path, origin, lines)) = token(spec, piece, previous.as_ref()) else {
-            errors.push(Diagnostic::error(
-                "bad-path",
-                Some(paths_span),
-                format!("`{piece}` is not a path"),
-            ));
-            continue;
-        };
-        match path {
-            Ok(path) => {
-                previous = Some((path.clone(), origin));
-                entries.push(ManifestEntry {
-                    path,
-                    change,
-                    lines,
-                    span: located.span,
-                    origin,
-                });
+    for piece in pieces(spec, paths.as_str()) {
+        let (path, origin, lines) = match token(spec, piece, previous.as_ref()) {
+            Ok(token) => token,
+            Err((code, message)) => {
+                errors.push(Diagnostic::error(code, Some(paths_span), message));
+                continue;
             }
-            Err(e) => errors.push(Diagnostic::error(
-                "bad-path",
-                Some(paths_span),
-                e.to_string(),
-            )),
-        }
+        };
+        previous = path.file().map(|file| (file.clone(), origin));
+        entries.push(ManifestEntry {
+            path,
+            change,
+            lines,
+            span: located.span,
+            origin,
+        });
+    }
+}
+
+/// A piece of an item's paths: one token, or a range.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+enum Piece<'a> {
+    /// A path, a pattern or a `:lines` token.
+    Token(&'a str),
+    /// `` `a`–`b` ``: an en dash between two code spans (decision D41).
+    Range(&'a str, &'a str),
+}
+
+/// The pieces of `paths`. Ranges are found first, in every format, so a spec's `path_list` never
+/// sees their en dash; the text around them is split by `path_list` (D41).
+fn pieces<'a>(spec: &ManifestRules, paths: &'a str) -> Vec<Piece<'a>> {
+    const DASH: &str = "`–`";
+    let mut out = Vec::new();
+    let mut rest = paths;
+    let mut after_range = false;
+    while let Some(at) = rest.find(DASH) {
+        let (before, after) = (&rest[..at], &rest[at + DASH.len()..]);
+        let start_at = before.rfind('`').map_or(0, |i| i + 1);
+        let end_len = after.find('`').unwrap_or(after.len());
+        out.extend(split(spec, &before[..start_at], after_range, true));
+        out.push(Piece::Range(&before[start_at..], &after[..end_len]));
+        rest = &after[end_len..];
+        after_range = true;
     }
+    out.extend(split(spec, rest, after_range, false));
+    out
 }
 
-/// A path as spelled.
-type Spelled = Result<RepoPath, planrun_model::RepoPathError>;
+/// `text` split by `path_list`. Next to a range, the empty piece on its side is what is left of
+/// the separator, not an empty path: `after_range` drops a leading one, `before_range` a
+/// trailing one.
+fn split<'a>(
+    spec: &ManifestRules,
+    text: &'a str,
+    after_range: bool,
+    before_range: bool,
+) -> Vec<Piece<'a>> {
+    let next_to_range = after_range || before_range;
+    if next_to_range && text.trim().is_empty() {
+        return Vec::new();
+    }
+    let mut parts: Vec<&str> = match &spec.path_list {
+        Some(split) => split.split(text).collect(),
+        None => vec![text],
+    };
+    if before_range && parts.last().is_some_and(|p| p.trim().is_empty()) {
+        parts.pop();
+    }
+    if after_range && parts.first().is_some_and(|p| p.trim().is_empty()) {
+        parts.remove(0);
+    }
+    parts.into_iter().map(|p| Piece::Token(p.trim())).collect()
+}
 
-/// One token of a manifest item, after the token `previous` (decision D32): `:3-10` is more lines
-/// of the previous path; with `siblings`, a bare file name after a path with a directory is in
-/// that directory. `None` for a `:lines` token with no path before it.
+/// A diagnostic code and message.
+type Refusal = (&'static str, String);
+
+/// One piece of a manifest item, after the file `previous` (decisions D32, D41): `:3-10` is more
+/// lines of the previous path; with `siblings`, a bare file name after a path with a directory
+/// is in that directory; a token with `*`, `?` or `{`, and a range, are patterns.
 fn token(
     spec: &ManifestRules,
-    piece: &str,
+    piece: Piece<'_>,
     previous: Option<&(RepoPath, PathOrigin)>,
-) -> Option<(Spelled, PathOrigin, Option<LineRange>)> {
+) -> Result<(ManifestPath, PathOrigin, Option<LineRange>), Refusal> {
+    let piece = match piece {
+        Piece::Range(start, end) => {
+            let pattern =
+                PathPattern::range(start, end).map_err(|e| ("manifest-item", e.to_string()))?;
+            return Ok((ManifestPath::Pattern(pattern), PathOrigin::Written, None));
+        }
+        Piece::Token(piece) => piece,
+    };
     if let Some(range) = piece.strip_prefix(':') {
-        let (path, origin) = previous?;
-        return Some((Ok(path.clone()), *origin, line_range(range)));
+        let Some((path, origin)) = previous else {
+            return Err(("bad-path", format!("`{piece}` is not a path")));
+        };
+        return Ok((ManifestPath::File(path.clone()), *origin, line_range(range)));
+    }
+    if PathPattern::is_glob(piece) {
+        let pattern = PathPattern::glob(piece).map_err(|e| ("manifest-item", e.to_string()))?;
+        return Ok((ManifestPath::Pattern(pattern), PathOrigin::Written, None));
     }
     let (path, lines) = split_lines(piece);
     let dir = previous.and_then(|(p, _)| p.dir());
-    match dir {
-        Some(dir) if spec.siblings && !path.contains('/') => Some((
-            RepoPath::new(&format!("{dir}/{path}")),
-            PathOrigin::Sibling,
-            lines,
-        )),
-        _ => Some((RepoPath::new(path), PathOrigin::Written, lines)),
-    }
+    let (spelled, origin) = match dir {
+        Some(dir) if spec.siblings && !path.contains('/') => {
+            (RepoPath::new(&format!("{dir}/{path}")), PathOrigin::Sibling)
+        }
+        _ => (RepoPath::new(path), PathOrigin::Written),
+    };
+    let path = spelled.map_err(|e| ("bad-path", e.to_string()))?;
+    Ok((ManifestPath::File(path), origin, lines))
 }
 
 /// `src/a.rs:3-10` → (`src/a.rs`, 3–10); `src/a.rs:7` → (`src/a.rs`, 7–7).
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t1-outline-rs.patch"
Remove-Item "$env:TEMP\p2b-t1-outline-rs.patch"
```

File `crates/planrun-parse/tests/manifest_patterns.rs` (full contents):

```rust
//! Pattern manifest entries (decisions D41, D42): a range or a glob item is one entry that is
//! never a write target; `manifest-undeclared` and `manifest-untouched` match paths against it.

use planrun_model::{ChangeKind, ManifestPath};
use planrun_parse::{Registry, parse_plan};

const SPEC: &str = r#"
id = "t"
name = "manifest pattern test"
[detect]
patterns = [{ regex = '^### Task', weight = 60 }]
[outline]
task = { level = 3, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
[outline.manifest]
marker = '^\*\*Files:\*\*$'
item = '^(?P<change>Create|Modify): `(?P<paths>[^`]+(?:`(?:, | and |–)`[^`]+)*)`$'
changes = { Create = "create", Modify = "modify" }
path_list = '`(?:, | and )`'
siblings = true
[checks]
manifest-untouched = "warning"
manifest-undeclared = "warning"
[[recognizer]]
kind = "file-block"
id = "file-block#t"
lead = '^Replace `(?P<path>[^`]+)`:$'
ops = {}
default_op = "overwrite"
"#;

fn parse(text: &str) -> Result<planrun_parse::Parsed, Vec<&'static str>> {
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    registry
        .add_spec("t", SPEC)
        .unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, "t.md", text.as_bytes(), Some("t")).map_err(|d| d.error_codes())
}

fn task(files: &str, body: &str) -> String {
    format!("### Task 1: a\n\n**Files:**\n{files}\n\n{body}")
}

fn write(path: &str) -> String {
    format!("Replace `{path}`:\n\n```sql\nx\n```\n\n")
}

/// (path as written, is a pattern, change) per entry of the first task.
fn entries(parsed: &planrun_parse::Parsed) -> Vec<(String, bool, ChangeKind)> {
    parsed.plan.tasks[0]
        .manifest
        .entries
        .iter()
        .map(|e| {
            let pattern = matches!(e.path, ManifestPath::Pattern(_));
            (e.path.to_string(), pattern, e.change)
        })
        .collect()
}

fn warnings(parsed: &planrun_parse::Parsed) -> Vec<&str> {
    parsed.diagnostics.0.iter().map(|d| d.code).collect()
}

#[test]
fn a_range_item_is_one_pattern_entry_that_declares_its_files() {
    let text = task(
        "- Modify: `db/m/0001`–`0007_*.sql`",
        &write("db/m/0002_init.sql"),
    );
    let parsed = parse(&text).unwrap_or_else(|codes| panic!("{codes:?}"));
    let expected = [("db/m/0001–0007_*.sql".to_owned(), true, ChangeKind::Modify)];
    assert_eq!(entries(&parsed), expected);
    assert!(warnings(&parsed).is_empty(), "{:?}", warnings(&parsed));
}

#[test]
fn a_path_outside_the_range_is_undeclared_and_the_range_untouched() {
    let text = task(
        "- Modify: `db/m/0001`–`0007_*.sql`",
        &write("db/m/0008_late.sql"),
    );
    let parsed = parse(&text).unwrap_or_else(|codes| panic!("{codes:?}"));
    let mut found = warnings(&parsed);
    found.sort_unstable();
    assert_eq!(found, ["manifest-undeclared", "manifest-untouched"]);
}

#[test]
fn ranges_are_found_before_the_path_list_splits() {
    let text = task("- Modify: `db/a.cs`, `db/m/0001`–`0003.sql` and `b.cs`", "");
    let parsed = parse(&text).unwrap_or_else(|codes| panic!("{codes:?}"));
    let found: Vec<(String, bool)> = entries(&parsed)
        .into_iter()
        .map(|(p, pattern, _)| (p, pattern))
        .collect();
    let expected = [
        ("db/a.cs".to_owned(), false),
        ("db/m/0001–0003.sql".to_owned(), true),
        ("b.cs".to_owned(), false),
    ];
    assert_eq!(found, expected);
}

#[test]
fn a_glob_item_is_a_pattern_entry() {
    let text = task("- Modify: `src/{a,b}/*.rs`", &write("src/b/x.rs"));
    let parsed = parse(&text).unwrap_or_else(|codes| panic!("{codes:?}"));
    assert!(entries(&parsed)[0].1);
    assert!(warnings(&parsed).is_empty(), "{:?}", warnings(&parsed));
}

#[test]
fn broken_patterns_are_manifest_item_errors() {
    for files in [
        "- Modify: `db/0001`–`other/0007.sql`",
        "- Modify: `src/**/*.rs`",
        "- Modify: `src/{a,{b,c}}.rs`",
    ] {
        let codes = parse(&task(files, "")).err();
        assert_eq!(codes, Some(vec!["manifest-item"]), "{files}");
    }
}

#[test]
fn a_write_never_targets_a_wildcard() {
    let text = task("- Modify: `src/*.rs`", &write("src/*.rs"));
    assert_eq!(parse(&text).err(), Some(vec!["bad-path"]));
}
```

File `docs/architecture.md` (exact diff; save the block as `$env:TEMP\p2b-t1-architecture-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/architecture.md b/docs/architecture.md
index 034b4f5..bb8cf1e 100644
--- a/docs/architecture.md
+++ b/docs/architecture.md
@@ -219,8 +219,9 @@ A state machine over Task → Step → Action. Per action: `Pending → Previewe
 Done | Failed | Skipped`. Per task, the lifecycle of your §6.1 proposal:
 
 1. Show the task, its manifest and its actions (with `Note`/`Manual` in place).
-2. **Preflight**: manifest Create targets absent, Modify targets present, the first patch's
-   preimage blob matches the file on disk, `git status --porcelain` empty when the plan says so.
+2. **Preflight**: manifest Create targets absent, Modify targets present (pattern entries are
+   skipped: they name no single file, D41), the first patch's preimage blob matches the file on
+   disk, `git status --porcelain` empty when the plan says so.
 3. Execute steps in order; each step's file actions as one transaction (§6.2), scripts in sessions
    (§6.3), stopping per policy (§6.5).
 4. Commit only through the plan's own commit script, and only when the task's gate/verify passed
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t1-architecture-md.patch"
Remove-Item "$env:TEMP\p2b-t1-architecture-md.patch"
```

File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p2b-t1-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index 2477239..ca56565 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -276,3 +276,40 @@ superpowers format: its run lines share no variables with its bash commit fences
 They are written for an agent (probes, conditional branches, "record"). The target is zero errors
 and honest Manual coverage, not automation. P2 is split: P2a builds the schema, the kinds and the
 superpowers and before-after formats; P2b the devready format on the same schema.
+
+Decisions D41 onwards were agreed on 2026-11-25 at the start of P2b, from the review of the
+devready format against its two real plans.
+
+## D41 Pattern manifest entries
+
+A Files item may name a set of files instead of one: a glob (a token with `*`, `?` or `{`) or
+a range (an en dash between two code spans, `` `dir/0001`–`0007_*.sql` ``). Each is one manifest
+entry whose target is a `PathPattern`, not a `RepoPath`: `ManifestEntry.path` is a
+`ManifestPath`, either a file or a pattern.
+
+- Ranges are found in the item text before `path_list` splits it, in every format, like the
+  `:lines` tokens. A spec's `path_list` therefore never lists the en dash.
+- Globs: `*` matches any run of characters except `/`, `?` one character except `/`,
+  `{a,b}` either alternative. `**`, nested or unbalanced braces are `manifest-item`.
+- A range `dir/A`–`B` names the files of `dir` whose name `f` has `A <= f` and, when `B` has
+  no wildcard, `f <= B`. When it has one, with `B0` the part of `B` before its first wildcard:
+  `f` has at least `|B0|` characters, its first `|B0|` characters are `<= B0`, and the rest of
+  `f` matches the rest of `B` as a glob. Comparisons are by character. `B` may repeat `dir`;
+  a `B` in another directory, or a wildcard in `A`, is `manifest-item`.
+- A pattern is repository-relative as written and never resolved: a pattern relative to a
+  subdirectory, or one with prose in it, matches nothing (at worst a `manifest-undeclared`
+  warning).
+- `manifest-undeclared` accepts a path a pattern entry matches; `manifest-untouched` counts a
+  pattern entry as touched when any action path matches it. The write-mode check uses the
+  entry's change as for a file.
+- A pattern is never a candidate for `file_name_from_manifest` or `infer_path`, and never a
+  write target (D42). D33 counts it as non-literal.
+- P3 preflight skips the existence checks for pattern entries (architecture.md §6.1).
+
+Handover P2a→P2b open item 1 proposed a `PathOrigin` for patterns; a target type of its own
+was chosen instead, so "no write targets a wildcard" is a property of the types.
+
+## D42 RepoPath refuses wildcards
+
+Amends D7: a `RepoPath` also refuses `*` and `?`. Both are invalid in Windows file names
+(D19), and a path with one is a pattern (D41). A lead that names such a path is `bad-path`.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t1-decisions-md.patch"
Remove-Item "$env:TEMP\p2b-t1-decisions-md.patch"
```

File `docs/extending.md` (full contents):

````markdown
# Extending planrun

New plan styles will keep appearing. They are added without touching execution, in three levels
(architecture.md §5). Use the lowest level that works.

## 1. Decide the level

1. Describe the style in a table like the ones in [plan-grammar.md](plan-grammar.md), from at
   least two real plans.
2. For each row, find the built-in recognizer kind that covers it (§3). If every row is covered,
   write a **format spec** (§2). Done, no rebuild.
3. If a row is a new idiom, write a **recognizer kind** (§5) and use it from a spec.
4. Only if the plan's *structure* is not headings-with-steps (tasks in a table, a JSON plan),
   write a **Format** (§6).
5. A new **action kind** (§4) is almost never the answer: map the wording onto an existing action.

## 2. Format specs (TOML)

Location: built-ins in `formats/<id>.toml` (embedded); per repository in
`<repo>/.planrun/formats/<id>.toml`; per user in the config directory. The file name is the id.

```toml
id = "assay"                      # unique; the file name
name = "Assay plan (planbuild.py)"

[detect]                          # a pattern that matches a line outside fenced blocks adds
patterns = [ { regex = '^### P\d+-T\d+: ', weight = 60 } ]   # its weight once; capped at 100

[outline]
tasks_after = { level = 2, regex = '^Tasks$' }          # optional
task = { level = 3, regex = '^(?P<id>(?P<phase>P\d+)-T(?P<ord>\d+)): (?P<title>.+)$' }
tasks_end_level = 2                                      # optional
# captures: id, ord (a number), title (all required); phase (optional, from the first task)

[outline.manifest]                                       # optional
# The marker and its items are consumed by the outline: they fall in no step. The items end at
# the first block that is no list item at the marker's depth, or that is a step boundary (D37).
marker = '^\*\*Files\*\*$'                                # may capture `change` (`**Files:** create`)
item = '^(?P<change>Create|Modify|Delete) `(?P<paths>[^`]+)`$'   # `change` unless the marker has it
changes = { Create = "create", Modify = "modify", Delete = "delete" }
# → create | modify | rewrite | delete | test | generated (D27)
path_list = ', '                                         # optional: split `paths` into several;
# a `:3-10` piece adds lines to the path before it (D32). Built in, before the split, in every
# format: `a`–`b` (an en dash between code spans) is one range entry, so `path_list` must not
# list the en dash; a piece with `*`, `?` or `{` is a glob entry (D41). Pattern entries declare
# the files they match and are never a write target.
required = true                                          # optional, default true: false lets a
                                                         # task have no manifest (D27)
siblings = false                                         # optional, default false: a bare file
                                                         # name after `a/b.rs` is `a/<name>` (D32)

[[outline.steps]]                                        # zero or more; none = one implicit step
block = "paragraph"                                      # paragraph | list-item | heading
regex = '^\*\*Step (?P<n>\d+) — (?P<title>[^*]+)\*\*$'   # n and title optional

[settings]
manual_markers = ['(?i)\bby hand\b']                     # paragraph → Manual(Marked)
pwsh = ["powershell", "pwsh", "ps1"]                     # optional; these are the defaults
bash = ["bash", "sh", "shell"]

[[step_marks]]                                           # zero or more (D30), first match wins
title = '(?i)\bverify (?:it|they) fails?\b'              # regex over the step's title
mark = "expected-failure"                                # expected-failure | human-check
# expected-failure: the step's Verify scripts expect failure (none or `any` becomes Failure; an
# expected success is the error expected-conflict). human-check: its prose Notes become
# Manual(HumanCheck).

[checks]                                                 # optional (D31): only these two codes,
manifest-untouched = "off"                               # each error | warning | off; any
manifest-undeclared = "warning"                          # other key fails the spec load

[[recognizer]]                                           # ordered; first match wins
kind = "file-block"                                      # a registered kind (§3)
id = "file-block#assay"                                  # shown by `show` and coverage
# … the kind's own parameters
```

Rules:

- Unknown keys are errors (`deny_unknown_fields`). A spec is fully validated when the registry
  loads it (unknown kind, bad regex, bad parameter), before any plan is read.
- Regexes use the `regex` crate syntax; `(?P<name>…)` captures. A regex that lacks a capture
  its reader requires (`path`, `id`, …) is refused when the spec loads.
- `crates/planrun-parse/tests/spec_files.rs` checks every built-in spec against the schema.

## 3. Built-in recognizer kinds

Parameters in *italics* are optional.

| Kind | Matches | Produces | Parameters |
|---|---|---|---|
| `file-block` | a lead paragraph naming a path (and op), then one fence | `WriteFile` / `ApplyPatch` | `lead` (captures `path`, *`op`*), `ops` (op prefix → `write` \| `create` \| `overwrite` \| `patch`), *`default_op`*, *`file_name_from_manifest`* |
| `before-after` | lead (naming a path), fence, *separator paragraph*, fence | `ReplaceExact`, or `Manual(ProseEdit)` when `infer_path` finds no single entry | `lead` (captures `path`, or none with `infer_path = true`), *`separator`* (e.g. `^After:$`; absent = two fences back to back), *`file_name_from_manifest`* |
| `delete` | a sentence or inline `git rm` | `Delete` | `lead` (captures `path`), *`via_git`*, *`file_name_from_manifest`* |
| `generated` | a "generated, not by hand" sentence | `Generated` | `lead` (captures `path`) |
| `ceremony` | a fence directly after a given action whose body matches | nothing (absorbed) | `after` (`patch`), `body` |
| `script` | a fence in a shell language | `RunScript` / `Commit` | *`intents`* (`[{ intent, body?, lead? }]`: `commit` \| `gate` \| `preview` \| `apply` \| `verify` \| `other`), *`leads`* (regexes), *`first_block`* |
| `run-line` | `` Run: `cmd` `` in a paragraph, with an optional expectation in the same paragraph | `RunScript` / `Commit` (one line) | `lead` (captures `cmd`, *`expected`*), `shell` (`pwsh` \| `bash`), *`exit`* table over `expected` (field `text`), *`intents`* (`[{ intent, body }]`) |
| `expected-block` | `Expected (…):` paragraph + `text` fence after a script | amends the script | `lead` (captures `spec`), `exit` table, `lines = "summary"` |
| `expected-prose` | `Expected: …` sentence after a script | amends the script | `lead` (captures `text`), `exit` table (`success` \| `failure` \| `any` \| `code:N`) |
| `manual-prose` | a paragraph (+ the fence after it, with `takes_fence`) that describes what planrun cannot do exactly | `Manual(reason)` | `lead`, `reason` (`prose-edit` \| `output-dependent` \| `human-check`), *`takes_fence`* |

Notes on the kinds:

- `file-block`: the longest `ops` prefix of the captured `op` wins; `write` takes Create or
  Overwrite from the manifest. Once the lead matched, an op no prefix covers is
  `Malformed(unknown-file-op)`, a lead without its fence `Malformed(missing-block)`, and for
  `patch` a patch whose paths differ from the lead's `Malformed(patch-path)`.
- Paths from the manifest (D32): `file_name_from_manifest = true` resolves a lead path without
  `/` that is not an entry itself to the one entry with that file name, when that entry was
  written in full (not a sibling); otherwise the path stays as written. `before-after` with
  `infer_path = true` takes the task's one entry that is not Test, Generated or Delete; with none
  or several, the lead and both fences become one `Manual(ProseEdit)`. `infer_path` is never
  available for writes. Each resolved action carries its `PathOrigin`; `show` marks it.
- `run-line`: the command is the `cmd` capture, run as one line in `shell` (D39). The `expected`
  capture, when the lead has one and it matched, becomes a prose expectation through the `exit`
  table (no row matches: `any`). An empty command is `Malformed(missing-block)`.
- `before-after`: once the lead matched, a missing fence or separator is
  `Malformed(missing-block)`.
- `script`: `intent = "commit"` selects the `Commit` action kind, not a `ScriptIntent`; a commit
  script carries `ScriptIntent::Other`. The other intents set the `ScriptIntent`; no match is
  `verify`. A row matches the fence body (`body`), the paragraph directly before the fence
  (`lead`), or both; at least one is required (D29). Preview/apply pairs are rows like these.
- `script` guard (D28): with `leads` set, the kind claims a fence only when the paragraph
  directly before it matches one of `leads`, or, with `first_block = true`, when the fence is the
  first block of a written step (never the implicit first step, so never right after the
  manifest). A body intent never qualifies a fence on its own. Any other shell fence falls to the
  fallback, `Manual(Unrecognized)`. Without `leads` (the assay spec) every shell fence is a script.
- `expected-block` and `expected-prose` amend the script directly before them. Anything else there
  is `Malformed(expected-without-script)`; a second expectation on one script is
  `Malformed(expected-twice)`. `expected-block` needs a `text` fence (`missing-block`). An
  `expected-prose` sentence directly after a `Manual` joins that Manual's text instead (D38).
- Coverage counts the blocks of a recognizer's `Manual` (and of an expectation joined to a
  Manual) as manual, not as actions.
- **Exit tables are first match**: rows are tried in order and the first whose regex matches
  decides, so their order matters. Values: `success`, `failure`, `any` (`ExitExpect::Unknown`: not
  checked; prose is still confirmed, D11), `code:N`, and `code` (the row's regex captures `code`;
  0 is `success`). An `expected-block` spec no row matches is `Malformed(expected-spec)`; an
  `expected-prose` text no row matches is `any`.

The **fallback** (not configurable) runs when nothing matched: paragraph, list item, blockquote →
`Note` or `Manual(Marked)`; fence → `Manual(Unrecognized)` / `Manual(ForeignLanguage)`, a `text`
fence → `Note`; table, heading, rule → `Note`; HTML comment → dropped.

## 4. Adding an action kind

Rare (D4). The checklist, all in one change:

1. The variant in `planrun-model::ActionKind`, with docs.
2. Its handler in planrun-exec, its `DoneState` logic for resume, its dry-run behaviour.
3. Its policy class.
4. Its journal record.
5. Its `show` rendering and coverage label.
6. A corpus plan that produces it, and an invalid-corpus file for each new error.

## 5. Adding a recognizer kind

```rust
pub struct InsertAfterFactory;

impl RecognizerFactory for InsertAfterFactory {
    fn kind(&self) -> &'static str { "insert-after" }

    fn build(&self, params: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        // Read and validate every parameter here: a bad regex fails when the spec loads.
        Ok(Arc::new(InsertAfter { lead: regex(params, "lead")?, id: id(params)? }))
    }
}

struct InsertAfter { lead: Regex, id: String }

impl Recognizer for InsertAfter {
    fn id(&self) -> &str { &self.id }

    fn recognize(&self, cx: &StepContext<'_>, cursor: &mut Cursor<'_>) -> Recognized {
        // 1. Peek; return Recognized::No without advancing unless the lead matches.
        // 2. Once the lead matched, a missing or wrong block is Recognized::Malformed, not No.
        // 3. Advance past exactly what was consumed; return the actions.
        Recognized::No
    }
}
```

Register it in `Registry::with_builtins`. Rules:

- **Pure.** No I/O, no clock, no state between calls. Same input, same output.
- **`No` leaves the cursor where it was.** Advance only on `Yes`/`Amend`.
- **Commit to the idiom early.** Once the distinctive lead matched, failures are `Malformed`
  errors. Falling through to `Manual` would hide a broken plan.
- **Tests**: unit tests on hand-built blocks, plus a corpus plan in `tests/corpus/synthetic` and
  an invalid-corpus file per `Malformed` code.

## 6. Adding a Format by hand

Implement `Format` (`detect`, `outline`, `recognizers`, `settings`) and register it with
`Registry::add_format`. Reuse recognizer instances from the factories for the step contents: only
the outline should be new. Every block of every task must fall in exactly one step range.

## 7. Execution-side extension points

| Trait | Registry | For |
|---|---|---|
| `ActionHandler` | `HandlerSet` | how an action kind executes, previews, and reports `DoneState` |
| `ShellSession` | one per `Shell` | another shell (`cmd`, `zsh`): a new `Shell` variant + session |
| `CommandClassifier` | the runner's | different policy classification |
| `Ui` | the CLI | a TUI (ratatui) later, a non-interactive batch UI |
````

File `docs/plan-grammar.md` (full contents):

```markdown
# Plan grammar

What planrun reads: the rules common to every format (§1), then the four known formats (§2–§5).
Every format is described by its spec in `formats/<id>.toml`; this document explains them for
people. The synthetic examples are in `tests/corpus/synthetic/`.

## 1. Common rules

### 1.1 Text

- UTF-8. A leading BOM is removed; CRLF and lone CR become LF (D6). Nothing else is changed.
- Patterns match the **raw** text of a block (backticks, `**`, `—` included), anchored to the
  block.

### 1.2 Fenced blocks

- CommonMark fences: a fence opened with N backticks closes at the first line of ≥ N backticks.
  This is what lets a ```` ````markdown ```` block contain ```` ``` ```` blocks.
- An **unclosed fence is an error** (`unclosed-fence`), not "runs to the end of the document".
  So is a fence closed by accident: inside an open fence, a line indented at most 3 spaces that
  opens a fence of the same character, at least as long, with an info string (architecture.md
  §4.1).
- The body is exact: everything between the fence lines, each line ending in `\n`.
- The info string's first word is the language. `powershell`, `pwsh`, `ps1` run in pwsh; `bash`,
  `sh`, `shell` in bash (a format may change these lists). `text` is data (an expectation, or a
  `Note`). Any other language not consumed by a recognizer is `Manual(ForeignLanguage)`.

### 1.3 Fences inside list items

CommonMark strips the item's indentation from a nested fence, so its body is not contiguous in the
source. The document builder rebuilds it (`TextRef::Owned`) and keeps the original span for
messages.

### 1.4 Paths

Paths in plans are repository-relative with forward slashes. `` `src/a.rs:3-10` `` in a Files list
is a path with an informational line range. Anything absolute, with `..`, or with a backslash is
`bad-path`.

### 1.5 What is never executed

- Text before the first task (the preamble) and after the tasks: shown by `show`, never run.
  Manual markers in the preamble are listed by `check` and shown before task 1.
- HTML comments.
- `Note`s: shown in place.

## 2. `assay` — plans generated by planbuild.py

The strictest format; fully automatable. Example: `tests/corpus/synthetic/assay-style.md`.

| Element | Form |
|---|---|
| Tasks start | `## Tasks` (level 2) |
| Task | `### P1-T1: title`, `### P3c-1-T1: title` (id `<phase>-T<ord>`; phase `P1`, `P3c-1`) |
| Tasks end | the next level-2 heading (`## Part 1 green state`) |
| Goal | `**Goal:** …` (a Note) |
| Manifest | `**Files**` + list items ``Create `p` ``, ``Modify `p` ``, ``Delete `p` ``, each optionally followed by a parenthesised note; consumed by the outline |
| Step | `**Step 1 — title**` (em dash) |
| Full contents | ``File `p` (full contents):`` + fence. Create or overwrite per the manifest |
| Patch | ``File `p` (exact diff; save the block as `…`, outside the repository, then apply it):`` + `diff` fence |
| Patch ceremony | the `powershell` fence right after a patch with `git apply … $env:TEMP\….patch` / `Remove-Item`: **absorbed**, never run (D17) |
| Delete | ``Delete `p`.`` |
| Other ops | ``File `p` (…):`` with any other parenthesis: `unknown-file-op` |
| Generated | ``` `p` is generated by …, not written by hand ``` (or "never"); ``` `p` changes: it is regenerated by … ```; ``Restore `p` from … (never written by hand; …):`` before the script that restores it |
| Script | `powershell` fence. Intent: `commit` when it runs `git commit`; `gate` when it sets `$gate = $LASTEXITCODE`; else verify |
| Expected | `Expected (exit code N; summary lines):` + `text` fence → exit code N (0 is Success) + summary lines |
| Prose expected | `Expected: …` sentence → phrase table → exit expectation + prose |
| Commit | the commit script, as written; it is gated by `$gate` in the plan itself, and planrun gates it on the task's verification too |

Notes:

- A task may name a generated path twice (an intro notice and a step notice): one check, after
  the task's last non-commit action.
- The patch's `index aaaaaaa..bbbbbbb` line carries the pre- and postimage blob ids; consecutive
  patches on a path chain (`blob-chain`).
- The commit script's `git diff --cached --exit-code <scratch-sha>` check refers to scratch commits
  that exist only on the plan author's machine; it is run as written and its failure is reported
  like any failure.

## 3. `superpowers` — the writing-plans skill

Hand-written by Claude; partly automatable. Spec: `formats/superpowers.toml`. Example:
`superpowers-style.md`.

| Element | Form |
|---|---|
| Task | `### Task 1: title` (id `Task 1`, ordinal from the number) |
| Tasks end | the next level-2 heading (`## Self-Review`) |
| Manifest | `**Files:**` + `- Create: \`p\``, `- Modify: \`p:3-10\``, `- Test: \`p\``, each optionally followed by a parenthesised note. Optional: a task may have none (D27) |
| Step | list item `- [ ] **Step 1: title**`; the manifest's items end at the first step (D37) |
| Run | `` Run: `cmd` `` in a paragraph, often with `Expected: …` on the next line **of the same paragraph** → `RunScript` in pwsh with that expectation (`run-line`, D39) |
| Create | ``Create `p` with …:`` + fence → `WriteFile(Create)`; ``Replace the entire contents of `p` with:`` + fence → `WriteFile(Overwrite)`. The op decides; a manifest entry that disagrees is `manifest-write-mode` (D26) |
| Exact edit | ``In `p`, line 134, change:`` + fence + `to:` + fence → `ReplaceExact` |
| Prose edits | `Append to …:`, ``In `p`, replace … with:``, `Above …, add:` + fence → **`Manual(ProseEdit)`** (`manual-prose`) |
| Commit | `bash` fence with `git commit`, as the first block of its step |
| Other shell fences | only behind `Run:`; otherwise `Manual(Unrecognized)` (D28) |

Step marks (D30): a title like "…verify it fails" marks the step's verify scripts as **expected
failures**, skipped by default (D12); "Manual verification", "Check by eye" turn the step's Notes
into `Manual(HumanCheck)`.

Checks (D31): `manifest-untouched` is off (prose edits change declared files),
`manifest-undeclared` is a warning.

## 4. `devready` — DevReady plans

Hand-written; partly automatable. Example: `devready-style.md`.

| Element | Form |
|---|---|
| Task | `### Task 0: title` (Task 0 is allowed; ordinals strictly increase from it) |
| Manifest | `**Files:** create` / `modify` / `delete` + paths in prose or a list |
| Steps | none written; sub-labels `**4a.**`, `**Verify:**`, `**Commit:**` split the task into steps |
| Create | `` `p`: `` + fence |
| Edit | "edit 1 (old → new):" / "Edit (old → new):" + two fences → `ReplaceExact` |
| Delete | `` `git rm p` `` → `Delete { via_git: true }` |
| Verify | `**Verify:**` + fences; trailing `# expected` comments on command lines become prose expectations (never auto-pass) |
| Commit | `**Commit:** \`message\`` → `Manual` that shows a suggested `git add -- <manifest paths>; git commit -m "<message>"` (the plan does not say which files; open question for Phase 2) |
| Manual markers | "Don't …", "manual step", "Rens runs" |

## 5. `before-after` — "before:/After:" plans

Hand-written; mostly automatable. Spec: `formats/before-after.toml`. Example:
`before-after-style.md`.

| Element | Form |
|---|---|
| Task | `## Task 1: title` (**level 2**); a level-2 heading that is no task ends the tasks (D37) |
| File map | a table before the tasks: shown, never parsed into actions |
| Manifest | `**Files:**` + `Create:`, `Modify:` (with `:lines`), `Rewrite:`, `Test:`; several paths per item, separated by `, ` or ` and `; `` `dir/0001`–`0007_*.sql` `` is one range entry (D41); a bare file name after a path is in that path's directory and a `:lines` token adds lines to the path before it (D32); a trailing `(note)`. Optional: a task may have none |
| Step | list item `- [ ] **Step 1: title**` |
| Create | `` `p`: `` / ``Create `p` (…):`` + fence → `WriteFile(Create)` |
| Overwrite | `` `p` — replace the whole file: `` / ``Replace `p`:`` + fence → `WriteFile(Overwrite)` |
| Edit | `` `p`, before: `` + fence + `After:` + fence → `ReplaceExact` (`before` must occur exactly once) |
| Edit without a path | `Before:` / ``In `Member`, before:`` + fence + `After:` + fence → `ReplaceExact` on the task's one editable entry, else `Manual(ProseEdit)` (D32) |
| Partial edits | ``"`p`, replace the class (keep …):"``, ``"`p`, add these members to the class:"``, `Add … :` → `Manual(ProseEdit)` |
| Scripts | `powershell` fences that are the first block of a step or follow a run lead (`Run:`, `Preview…:`, `Apply:`, `Verify…:`, `… Then:`); any other shell fence is `Manual(Unrecognized)` (D28) |
| Expected | prose `Expected: …`, including `build error` (expected failure) and `no matches` (`rg` exits 1); after a Manual (an `sql` block) it joins the Manual (D38) |
| Preview/apply | a `Preview…:` / `Apply:` lead, or a `-DryRun` / `-All` body: intent Preview then Apply (D29); planrun always stops between them and shows the count next to the plan's expected count |
| Output-dependent | "Replace … with the `Actual` values" → `Manual(OutputDependent)` |
| Other languages | `sql` fences → `Manual(ForeignLanguage("sql"))` |
| Prose checks | `Verify: <sentence>` → `Manual(HumanCheck)` |

Step marks (D30): "…verify they fail" marks expected failures; a title ending in "on … TST" or
starting with "Hand off" turns the step's prose into `Manual(HumanCheck)`.

Checks (D31): `manifest-untouched` is off (scripts and prose edits change declared files),
`manifest-undeclared` is a warning.

## 6. Recognizing a new format

Write the table above for it first, from at least two real plans. Then follow extending.md §2.
```

File `formats/before-after.toml` (full contents):

```toml
# The before-after format: "`p`, before: / After:" plans (docs/plan-grammar.md §5). Hand-written,
# mostly automatable: full-file writes, exact before/after edits, preview/apply pairs and the
# scripts behind run leads run; partial edits described in prose, output-dependent edits and
# foreign-language blocks are Manual. Every regex is matched against raw block text.

id = "before-after"
name = "Before/After plan"

[detect]
patterns = [
  { regex = '^## Task \d+: ', weight = 40 },
  { regex = '^- \[ \] \*\*Step \d+: ', weight = 40 },
  { regex = '^`[^`]+`, before:$', weight = 20 },
]

[outline]
# Tasks and the heading that ends them are both level 2 (D37).
task = { level = 2, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
tasks_end_level = 2

[outline.manifest]
marker = '^\*\*Files:\*\*$'
# - Modify: `a/b.cs:3-10`   - Rewrite: `a/x.razor`, `a/x.razor.cs`   - Modify: `a/p.cs:84-85` and `:241-243`
# - Modify: `a/r.resx`, `r.en.resx` (lines 227-230)   - Modify: `a/0001`–`0007_*.sql` (a range: one pattern entry, D41)
item = '^(?P<change>Create|Modify|Rewrite|Test): `(?P<paths>[^`]+(?:`(?:, | and |–)`[^`]+)*)`(?: \(.+\))?$'
changes = { Create = "create", Modify = "modify", Rewrite = "rewrite", Test = "test" }
path_list = '`(?:, | and )`'
required = false
siblings = true

[[outline.steps]]
block = "list-item"
regex = '^\*\*Step (?P<n>\d+): (?P<title>.+)\*\*$'

# A step title that says the step fails, or that the step happens elsewhere, marks the step (D30).
[[step_marks]]
title = '(?i)\bverify (?:it|they) fails?\b'
mark = "expected-failure"

[[step_marks]]
title = '(?i)(?:\bon (?:\S+ )?TST$|^Hand off\b|\bby eye\b)'
mark = "human-check"

[settings]
manual_markers = ['(?i)\bby hand\b', '(?i)\bRens runs\b', '(?i)\bmanual step\b']

# Scripts (`replace`) and prose edits change declared files, and items name ranges (D31).
[checks]
manifest-untouched = "off"
manifest-undeclared = "warning"

# --- Recognizers, tried in this order at each position of a step -------------------------------

[[recognizer]]
kind = "file-block"
id = "file-block#create"
# `p`:   Create `p` (only `*.sql` is embedded, …):
lead = '^(?:Create )?`(?P<path>[^`]+)`(?: \([^)]*\))?:$'
ops = {}
default_op = "create"

[[recognizer]]
kind = "file-block"
id = "file-block#replace-whole"
# `p` — replace the whole file:
lead = '^`(?P<path>[^`]+)` — replace the whole file:$'
ops = {}
default_op = "overwrite"

[[recognizer]]
kind = "file-block"
id = "file-block#replace"
# Replace `p`:
lead = '^Replace `(?P<path>[^`]+)`:$'
ops = {}
default_op = "overwrite"

[[recognizer]]
kind = "before-after"
id = "before-after#path"
# `p`, before: + fence + After: + fence
lead = '^`(?P<path>[^`]+)`, before:$'
separator = '^After:$'

[[recognizer]]
kind = "before-after"
id = "before-after#no-path"
# Before: / In `Member`, before: — the task's one editable entry, else Manual (D32).
lead = '^(?:In `[^`]+`, before|Before):$'
separator = '^After:$'
infer_path = true

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#output-dependent"
# … Replace the hashes … with the `Actual` values.
lead = '\bwith the `Actual` values?\b'
reason = "output-dependent"

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#edit"
# `p`, replace the class (keep …):   `p`, add these members to the class:   Add to the usings …:
lead = '^(?:`[^`]+`, (?:replace|add|insert|remove|change)\b.*|(?:Add|Append|Insert|Remove)\b.*):$'
reason = "prose-edit"
takes_fence = true

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#edit-then-verify"
# In the `X` row of the table near line 452, add … . Verify:   (the script after it runs)
lead = '^In\b.*\b(?:add|change|replace|remove)\b.*\. Verify:$'
reason = "prose-edit"

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#check"
# Verify: rerunning the preview shows 0 everywhere.
lead = '^Verify: \S'
reason = "human-check"

[[recognizer]]
kind = "script"
id = "script#before-after"
# Run only as the first block of a written step or behind a run lead (D28).
leads = [
  '^(?:Run|Preview|Apply|Verify)\b[^\n]*:$',
  '(?:^|\. )(?:Then|Verify):$',
  '^Expected: .*\bAlso confirm\b.*:$',
]
first_block = true
intents = [
  { intent = "commit", body = '(?m)\bgit commit\b' },
  { intent = "preview", lead = '^Preview\b' },
  { intent = "apply", lead = '^Apply:$' },
  { intent = "preview", body = '\s-DryRun\b' },
  { intent = "apply", body = '\s-All\b' },
]

[[recognizer]]
kind = "expected-prose"
id = "expected-prose#before-after"
lead = '^Expected: (?P<text>.+)$'
# First match wins: "no matches" is rg's exit 1.
exit = [
  { text = '(?i)\b(fails?|build errors?|does not exist)\b', exit = "failure" },
  { text = '(?i)\bno matches\b', exit = "code:1" },
  { text = '(?i)\b(pass(es)?|succeeds?)\b', exit = "success" },
]
```

**Step 2 — verify**

Targeted: the pattern tests:

```powershell
cargo test -p planrun-parse --test manifest_patterns
```

Expected (exit code 0; summary lines):

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-cli/src/view.rs crates/planrun-model/src/lib.rs crates/planrun-model/src/path.rs crates/planrun-model/src/pattern.rs crates/planrun-model/src/plan.rs crates/planrun-parse/src/check.rs crates/planrun-parse/src/kinds/file_block.rs crates/planrun-parse/src/kinds/mod.rs crates/planrun-parse/src/outline.rs crates/planrun-parse/tests/manifest_patterns.rs docs/architecture.md docs/decisions.md docs/extending.md docs/plan-grammar.md formats/before-after.toml
  $verified = [ordered]@{
      'crates/planrun-cli/src/view.rs' = 'f68897327be84bb8ba325b870d14b51c29301b30'
      'crates/planrun-model/src/lib.rs' = '459bda21c2599469631c5bf391a85e35274e4c3b'
      'crates/planrun-model/src/path.rs' = '775c81b7dc8a87d821b05976e0623beba3accc3b'
      'crates/planrun-model/src/pattern.rs' = '84dc806634c512c2989935bb05ab2834c528da84'
      'crates/planrun-model/src/plan.rs' = '73e336bc1e5e08d4d4d6cbd55195bd4be6bbf5b3'
      'crates/planrun-parse/src/check.rs' = '5a7a3e36cc6df7703bb915bb60eccc4889c96b3f'
      'crates/planrun-parse/src/kinds/file_block.rs' = '0a4e21584021cb88ca4cb45ebfb3370a201057a9'
      'crates/planrun-parse/src/kinds/mod.rs' = 'c8d5854f0f3221bc488a6d6aaf1f9f64869be87d'
      'crates/planrun-parse/src/outline.rs' = '18742dc6b0c71e0640837eec1ef9c5de77296fb6'
      'crates/planrun-parse/tests/manifest_patterns.rs' = '4ef7f0e8a282ac1ba67c6bfd91243c3b01f254ae'
      'docs/architecture.md' = 'bb8cf1ea68f91137e3b5d39804fdace7a067977c'
      'docs/decisions.md' = 'ca565653d6110ba39d335f7066563408b8400fe4'
      'docs/extending.md' = 'f454a55d9da607b0060762335929aa31aef79e3c'
      'docs/plan-grammar.md' = '3476ee89c5738772727366e0286907bb5b183ced'
      'formats/before-after.toml' = '5862d53ef3dac47cce5c8c2e481c72aea58905e7'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P2b-T1: pattern manifest entries (D41), RepoPath refuses wildcards (D42)" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P2b-T2: inline Files clauses (D43), a change per path (D44), bare names (D45)

**Goal:** The manifest schema the devready plans need. The manifest code moves out of `outline.rs` into a new module, `manifest.rs`, unchanged, and grows there: a marker that captures `items` holds the manifest inline, split by `clause_list`, and the text it declares nothing with is shown as a Note (or `Manual(Marked)`) at the start of the intro step; `path_change` gives one path its own change; without `siblings`, a bare file name after a path with a directory is `PathOrigin::Bare`, never resolved to, and a Create onto it is `Manual(AmbiguousPath)`. Also review item 6: the `match` on a bool in `recognize.rs` becomes an `if`.

**Files**

- Modify `CLAUDE.md`
- Modify `crates/planrun-cli/src/view.rs`
- Modify `crates/planrun-model/src/action.rs`
- Modify `crates/planrun-model/src/path.rs`
- Modify `crates/planrun-parse/src/format.rs`
- Modify `crates/planrun-parse/src/kinds/file_block.rs`
- Modify `crates/planrun-parse/src/kinds/mod.rs`
- Modify `crates/planrun-parse/src/lib.rs`
- Create `crates/planrun-parse/src/manifest.rs`
- Modify `crates/planrun-parse/src/outline.rs`
- Modify `crates/planrun-parse/src/outline_tests.rs`
- Modify `crates/planrun-parse/src/recognize.rs`
- Modify `crates/planrun-parse/src/spec.rs`
- Create `crates/planrun-parse/tests/manifest_inline.rs`
- Modify `crates/planrun-parse/tests/manifest_schema.rs`
- Modify `docs/architecture.md`
- Modify `docs/decisions.md`
- Modify `docs/extending.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/src/view.rs`, `crates/planrun-parse/src/outline.rs`, `crates/planrun-parse/src/recognize.rs`, `crates/planrun-parse/src/spec.rs`, `docs/architecture.md`, `docs/decisions.md`.

`outline.rs` loses its manifest code to `manifest.rs` (a move, then the new code); the `split_lines` test moves with it from `outline_tests.rs`.

**Step 1 — write the files**

File `CLAUDE.md` (full contents):

````markdown
# planrun

A Rust CLI that executes structured Markdown implementation plans against a git repository, task by
task, step by step: exactly what the plan says, nothing more. It is not an agent. Read
`docs/architecture.md` before changing code; `docs/decisions.md` holds the rules below with their
reasons (cited as `D<n>`).

## Priorities (D1, D25)

Correctness > maintainability/extensibility > execution speed > low memory > predictability.
When two of these conflict, the earlier one wins. Speed means no wasted process starts or repeated
reads; don't optimize I/O speculatively (cargo/npm dominate wall time).

## Layout

```
crates/planrun-model   the IR: Plan, Task, Step, Action, Span/TextRef, RepoPath, PathOrigin,
                       ManifestPath/PathPattern,
                       AbbrevBlobId
crates/planrun-parse   Markdown → IR: source, document, format, spec, registry, outline,
                       manifest, recognizer, recognize, kinds/ (one file per recognizer kind),
                       patch, check, diagnostic
crates/planrun-exec    IR → effects: workspace, git, shells, expect, policy, journal, runner
crates/planrun-cli     the `planrun` binary: check, show (view, render)
formats/*.toml         built-in plan formats (embedded): assay, superpowers, before-after; the
                       file name is the format id
docs/                  architecture, plan-grammar, extending, decisions, roadmap, plans/,
                       handovers/ (one per finished phase)
tests/corpus/          synthetic/ (one plan per format), invalid/ (one error each), local/ (private)
tests/fixtures/greeter the repo the synthetic assay plan applies to
```

## Hard rules

1. **planrun-parse never executes and never touches the repository. planrun-exec never depends on
   planrun-parse** (D2). Don't add a dependency that crosses these lines.
2. **No partial plans** (D3). Any error means no `Plan`, nothing runs.
3. **Nothing is dropped.** Every block of a task becomes an action, `Note` or `Manual` (D21). Once
   a recognizer's distinctive lead matched, a broken idiom is a `Malformed` error, never a
   fallback.
4. **The action set is closed** (D4). New plan wording maps onto existing actions. Adding an
   `ActionKind` follows docs/extending.md §4, all in one change, and only with Rens' agreement.
5. **New plan styles are data first** (D8): a TOML spec, then a recognizer kind, then a hand-written
   `Format`, in that order of preference (docs/extending.md).
6. **Paths in actions are `RepoPath`** (D7). Nothing outside the repository is written except
   `.git/planrun/` and the OS temp directory. The working tree gets only what the plan writes.
7. **Patches go through `git apply`** with exact context (D20). No fuzz, no `similar`-based
   patching.
8. **Prose never passes a script on its own** (D11); expected failures are skipped by default (D12).
9. **No new dependency without a line in docs/decisions.md D9.**
10. **Real plans stay in `tests/corpus/local/`** (git-ignored, D16). Never copy their content into
    committed files, tests, snapshots, docs or commit messages. Synthetic plans only.

## Commands

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test -- --ignored local_corpus      # parses the private real plans, when present
cargo run -p planrun-cli -- check tests/corpus/synthetic/assay-style.md
```

The gate for every task is all three of fmt, clippy (warnings are errors) and test.

## Code conventions

- Library code: no `unwrap`/`expect` (clippy warns; the gate fails). Tests may use
  `unwrap_or_else(|e| panic!("{e}"))`.
- Every public item has a doc comment (`missing_docs` warns).
- Diagnostics have a stable kebab-case `code`; tests pin codes, snapshots pin messages.
- Text from the plan is a `TextRef`, never a copied `String`, except `TextRef::Owned` where the
  source isn't contiguous.
- Prefer enums over strings, traits for extension points, composition over deep generic layering.
- Keep functions flat: guard clauses and early returns, at most three levels of nesting.
- Recognizers are pure; parameters are validated when a spec loads, not when a plan is parsed.

## Tests

- Unit tests sit next to the code (`mod tests`), or in `<module>_tests.rs` once a file passes
  ~400 lines.
- `insta` snapshots for IR and CLI output. A changed snapshot is a review item: explain it in the
  commit, never re-pin by reflex.
- Adding a check rule adds a file to `tests/corpus/invalid/`. Adding a format adds a plan to
  `tests/corpus/synthetic/` with its snapshot.
- Execution tests run on a temporary copy of `tests/fixtures/greeter` (`git init` in the copy),
  never on this repository.

## Docs stay in step

- A changed decision: a new `D<n>` in docs/decisions.md that supersedes the old one.
- A changed spec schema: `spec.rs`, docs/extending.md §2 and `formats/*.toml` together (the
  `spec_files` test checks the files).
- A new recognizer kind: docs/extending.md §3.
- A new format: docs/plan-grammar.md.

## Working with Rens

- Workflow: brainstorm → design spec → implementation plan (`docs/plans/`, assay format, see
  `docs/plans/README.md`) → execution. Propose before making non-trivial edits; stop after writing
  a plan.
- Direct, technically precise; no flattery. Put review text and anything meant to be pasted in
  code blocks.
- Rens works on Windows with PowerShell 7. Commands in plans are PowerShell 7.
- One commit per task, message `P<N>-T<n>: <title>`. Don't push.
- When a phase is done, write a handover for the next session:
  `docs/handovers/YYYY-MM-DD-p<N>-to-p<N+1>.md` (state, what the phase built, open items, notes
  for the next phase, how plans are verified). A new session starts from the latest one.
````

File `crates/planrun-cli/src/view.rs` (exact diff; save the block as `$env:TEMP\p2b-t2-view-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/view.rs b/crates/planrun-cli/src/view.rs
index f688973..e2571fa 100644
--- a/crates/planrun-cli/src/view.rs
+++ b/crates/planrun-cli/src/view.rs
@@ -301,6 +301,7 @@ fn path_from(origin: PathOrigin) -> Option<String> {
         PathOrigin::Sibling => "the directory of the manifest path before it",
         PathOrigin::FileName => "the manifest entry with this file name",
         PathOrigin::OnlyEntry => "the task's only editable manifest entry",
+        PathOrigin::Bare => "a bare file name in the manifest, its directory unknown",
     };
     Some(format!("resolved from manifest: {how}"))
 }
@@ -313,6 +314,7 @@ fn entry(entry: &ManifestEntry) -> String {
     let note = match (&entry.path, entry.origin) {
         (ManifestPath::Pattern(_), _) => " (a pattern: never a write target)",
         (_, PathOrigin::Written) => "",
+        (_, PathOrigin::Bare) => " (a bare file name: its directory is unknown)",
         _ => " (in the directory of the path before)",
     };
     format!("{:?} {}{lines}{note}", entry.change, entry.path)
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t2-view-rs.patch"
Remove-Item "$env:TEMP\p2b-t2-view-rs.patch"
```

File `crates/planrun-model/src/action.rs` (full contents):

```rust
//! Actions: the closed set of things a plan can make planrun do (decision D4).

use serde::{Deserialize, Serialize};

use crate::expect::Expectation;
use crate::ids::{AbbrevBlobId, RecognizerId};
use crate::path::{PathOrigin, RepoPath};
use crate::span::{Span, TextRef};

/// One action of a step, with where it came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    /// What to do.
    pub kind: ActionKind,
    /// The plan text this action was recognized from (lead sentence through the last block).
    pub span: Span,
    /// The recognizer instance that produced it.
    pub origin: RecognizerId,
    /// Where the action's path came from, when the plan did not write it in full (decision
    /// D32). `Written` for actions without a path.
    #[serde(default)]
    pub path_origin: PathOrigin,
}

/// What to do. Adding a variant is a cross-cutting change: a handler, a policy class, a journal
/// record, dry-run support and a coverage label (docs/extending.md §4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    /// Write a whole file.
    WriteFile {
        /// Target.
        path: RepoPath,
        /// The exact contents (fence body).
        body: TextRef,
        /// Create (must not exist) or overwrite (must exist).
        mode: WriteMode,
    },
    /// Apply a git-style patch with `git apply` semantics: exact context, no fuzz.
    ApplyPatch {
        /// Target, as named by the plan; must equal the paths inside the patch.
        path: RepoPath,
        /// The patch text.
        patch: TextRef,
        /// Preimage blob id from the `index` line, when present.
        pre: Option<AbbrevBlobId>,
        /// Postimage blob id from the `index` line, when present.
        post: Option<AbbrevBlobId>,
    },
    /// Replace text that occurs exactly once (`before:`/`After:`, `old → new`).
    ReplaceExact {
        /// Target.
        path: RepoPath,
        /// The text to find; must occur exactly once (line endings compared as LF).
        before: TextRef,
        /// Its replacement.
        after: TextRef,
    },
    /// Delete a file.
    Delete {
        /// Target.
        path: RepoPath,
        /// `git rm` (index too) rather than a plain delete.
        via_git: bool,
    },
    /// A file the plan says a command generates. Checked to exist after the task's last
    /// non-commit action, before the commit; a path the task names as generated more than once
    /// (an intro notice and a step notice) is one check.
    Generated {
        /// The generated file.
        path: RepoPath,
    },
    /// Run a script in a persistent shell session.
    RunScript(Script),
    /// Run a commit script, as written, only when the task's verification passed.
    Commit(Script),
    /// Plan text with no recognized instruction that is not an instruction either: shown in
    /// place, never blocks.
    Note {
        /// The text.
        text: TextRef,
    },
    /// Something the user must do or confirm by hand. Always blocks until acknowledged.
    Manual {
        /// Why planrun does not do it itself.
        reason: ManualReason,
        /// The plan text to show.
        text: TextRef,
    },
}

/// Create or overwrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteMode {
    /// The file must not exist.
    Create,
    /// The file must exist.
    Overwrite,
    /// The plan does not say (`(full contents)` with no Files entry): the manifest decides;
    /// a check error when it cannot.
    FromManifest,
}

/// Why an instruction is manual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManualReason {
    /// A fenced block no recognizer consumed.
    Unrecognized,
    /// An edit described in prose with no exact anchor ("add these members to the class").
    ProseEdit,
    /// A fenced block in a language planrun does not run (`sql`, `text` without a script).
    ForeignLanguage(Box<str>),
    /// An edit that depends on earlier output ("replace the hashes with the Actual values").
    OutputDependent,
    /// A check by eye, in a browser, on another machine.
    HumanCheck,
    /// Text that matched the format's `manual_markers` ("Rens runs it by hand").
    Marked,
    /// A write the plan creates at a manifest path whose directory it never wrote (a `Bare`
    /// entry, decision D45): the location is ambiguous.
    AmbiguousPath,
}

/// A script and what is expected of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Script {
    /// The shell that runs it, from the fence's info string.
    pub shell: Shell,
    /// The script, as written.
    pub body: TextRef,
    /// What the plan expects, when it says.
    pub expect: Option<Expectation>,
    /// Why the plan runs it.
    pub intent: ScriptIntent,
}

/// The shells planrun runs. Any other fence language becomes `Manual(ForeignLanguage)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Shell {
    /// PowerShell 7 (`powershell`, `pwsh`, `ps1`).
    Pwsh,
    /// bash (`bash`, `sh`, `shell`).
    Bash,
}

/// Why a script runs; drives policy defaults and the commit gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptIntent {
    /// A test, build or check whose result matters.
    Verify,
    /// The gate of a task (its result decides the commit).
    Gate,
    /// The preview half of a preview/apply pair (`-DryRun`, a count).
    Preview,
    /// The apply half: always stops for confirmation after its preview.
    Apply,
    /// Anything else (setup, generation).
    Other,
}
```

File `crates/planrun-model/src/path.rs` (full contents):

```rust
//! Repository-relative paths: the only path type an action may carry (decision D7).

use std::fmt;

use serde::{Deserialize, Serialize};

/// A path relative to the repository root, with forward slashes, that cannot leave the
/// repository by its spelling: no absolute path, no drive letter, no `..`, no `.` segment, no
/// empty segment, no backslash, no NUL, no wildcard (`*`, `?`: decision D42).
///
/// This checks the spelling only. Symlinks that point out of the repository are refused by the
/// workspace when it resolves the path on disk (planrun-exec).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RepoPath(Box<str>);

/// Why a path was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RepoPathError {
    /// Empty input.
    #[error("the path is empty")]
    Empty,
    /// `/x`, `C:\x`, `C:x` or `\\server\x`.
    #[error("`{0}` is absolute; plan paths are relative to the repository root")]
    Absolute(String),
    /// A `\` in the path.
    #[error("`{0}` contains a backslash; plan paths use forward slashes")]
    Backslash(String),
    /// A `..`, `.` or empty segment.
    #[error("`{0}` has a `.`, `..` or empty segment")]
    BadSegment(String),
    /// A NUL byte.
    #[error("the path contains a NUL byte")]
    Nul,
    /// A `*` or `?`: a pattern, never a file (decisions D41, D42).
    #[error("`{0}` contains a wildcard; a pattern is not a file path")]
    Wildcard(String),
}

impl RepoPath {
    /// Validates `path` as written in a plan.
    pub fn new(path: &str) -> Result<Self, RepoPathError> {
        if path.is_empty() {
            return Err(RepoPathError::Empty);
        }
        if path.contains('\0') {
            return Err(RepoPathError::Nul);
        }
        if path.contains('\\') {
            return Err(RepoPathError::Backslash(path.to_owned()));
        }
        if path.contains(['*', '?']) {
            return Err(RepoPathError::Wildcard(path.to_owned()));
        }
        let bytes = path.as_bytes();
        if path.starts_with('/') || (bytes.len() >= 2 && bytes[1] == b':') {
            return Err(RepoPathError::Absolute(path.to_owned()));
        }
        if path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        {
            return Err(RepoPathError::BadSegment(path.to_owned()));
        }
        Ok(Self(path.into()))
    }

    /// The path with forward slashes.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The last segment (`lib.rs` of `src/lib.rs`).
    pub fn file_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }

    /// The directory, without a trailing slash; `None` at the repository root.
    pub fn dir(&self) -> Option<&str> {
        self.0.rsplit_once('/').map(|(dir, _)| dir)
    }
}

/// Where a path in the IR came from, when the plan did not write it in full (decision D32).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathOrigin {
    /// Written in full where it is used.
    #[default]
    Written,
    /// A manifest token without a directory, placed in the directory of the token before it.
    Sibling,
    /// A lead that names only a file name: the one manifest entry with that file name.
    FileName,
    /// A lead that names no path: the task's one editable manifest entry.
    OnlyEntry,
    /// A manifest token without a directory after a token with one, in a format without
    /// siblings (decision D45): spelled as written, its location unknown. Never resolved to,
    /// and a Create onto it is `Manual(AmbiguousPath)`.
    Bare,
}

impl PathOrigin {
    /// True when the plan wrote the path in full.
    pub fn is_written(&self) -> bool {
        *self == PathOrigin::Written
    }
}

impl TryFrom<String> for RepoPath {
    type Error = RepoPathError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<RepoPath> for String {
    fn from(value: RepoPath) -> Self {
        value.0.into()
    }
}

impl fmt::Display for RepoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_are_accepted() {
        for ok in [
            "a.rs",
            "crates/x/src/lib.rs",
            ".claude-plugin/plugin.json",
            "a/.x",
        ] {
            assert_eq!(RepoPath::new(ok).map(|p| p.to_string()), Ok(ok.to_owned()));
        }
    }

    #[test]
    fn escaping_or_ambiguous_paths_are_refused() {
        for bad in [
            "",
            "/etc/passwd",
            "C:\\x",
            "C:x",
            "a\\b",
            "../x",
            "a/../b",
            "./a",
            "a//b",
            "a/",
            "a/*.rs",
            "a/b?.rs",
        ] {
            assert!(RepoPath::new(bad).is_err(), "{bad:?}");
        }
    }
}
```

File `crates/planrun-parse/src/format.rs` (full contents):

```rust
//! A plan format: how to find tasks, manifests and steps, and which recognizers turn a step's
//! blocks into actions (docs/architecture.md §5).

use std::sync::Arc;

use planrun_model::{FormatId, Manifest, PlanMeta, Shell, Span, TaskId, TextRef};

use crate::diagnostic::{Diagnostic, Diagnostics, Severity};
use crate::document::{BlockRange, Document};
use crate::recognizer::Recognizer;

/// A plan format. Most formats are a [`crate::spec::SpecFormat`] built from a TOML spec; a
/// format whose structure is not heading-shaped implements this trait by hand.
pub trait Format: Send + Sync {
    /// Its id (`assay`).
    fn id(&self) -> &FormatId;

    /// How well `doc` matches, 0–100. Cheap: patterns over the lines outside fenced blocks.
    fn detect(&self, doc: &Document) -> u8;

    /// The plan's structure: metadata, tasks, their manifests and step ranges. Every block of
    /// every task must fall in exactly one step range (the recognizers then account for each).
    fn outline(&self, doc: &Document) -> Result<Outline, Diagnostics>;

    /// The recognizers, in the order they are tried at each position of a step.
    fn recognizers(&self) -> &[Arc<dyn Recognizer>];

    /// Format-level settings the recognizers and checks read.
    fn settings(&self) -> &FormatSettings;
}

/// The structure of a plan before recognition.
#[derive(Debug, Clone)]
pub struct Outline {
    /// Plan metadata.
    pub meta: PlanMeta,
    /// Tasks in order.
    pub tasks: Vec<OutlineTask>,
    /// The blocks after the tasks (from a heading at or above `tasks_end_level`): never run.
    pub after_tasks: Option<BlockRange>,
}

/// A task before recognition.
#[derive(Debug, Clone)]
pub struct OutlineTask {
    /// Its id.
    pub id: TaskId,
    /// Its title.
    pub title: TextRef,
    /// Its span.
    pub span: Span,
    /// The parsed manifest.
    pub manifest: Manifest,
    /// The manifest's blocks (marker and items): consumed by the outline, in no step. Recognition
    /// skips them where they fall inside a step's range.
    pub manifest_blocks: Option<BlockRange>,
    /// Text of an inline manifest that declares nothing (decision D43): shown at the start of
    /// the intro step as a Note, or `Manual(Marked)` when it matches a manual marker.
    pub manifest_notes: Vec<TextRef>,
    /// Steps: number, title, blocks.
    pub steps: Vec<OutlineStep>,
}

/// A step before recognition.
#[derive(Debug, Clone)]
pub struct OutlineStep {
    /// Its number, when written.
    pub number: Option<u32>,
    /// Its title, when written.
    pub title: Option<TextRef>,
    /// Its span.
    pub span: Span,
    /// The blocks the recognizers consume.
    pub blocks: BlockRange,
    /// True when the step starts at a written boundary; false for the implicit first step.
    pub written: bool,
    /// The mark its title carries (`[[step_marks]]`, decision D30).
    pub mark: Option<StepMark>,
}

/// What a step's title says about the whole step (decision D30).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMark {
    /// "…verify it fails": the step's `Verify` scripts are expected to fail.
    ExpectedFailure,
    /// "Check by eye", "Manual verification": the step's Notes are `Manual(HumanCheck)`.
    HumanCheck,
}

/// Settings shared by a format's recognizers and the checks.
#[derive(Debug, Clone, Default)]
pub struct FormatSettings {
    /// Paragraphs matching any of these become `Manual(Marked)` instead of `Note`.
    pub manual_markers: Vec<regex::Regex>,
    /// Fence info strings that run in pwsh / bash. Anything else is `Manual(ForeignLanguage)`.
    pub pwsh_languages: Vec<String>,
    /// See `pwsh_languages`.
    pub bash_languages: Vec<String>,
    /// Severity overrides for the codes in [`DOWNGRADABLE`] (decision D31).
    pub checks: Vec<(&'static str, CheckLevel)>,
}

/// The only check codes a format may downgrade: manifest coverage, never a safety check
/// (decision D31).
pub const DOWNGRADABLE: [&str; 2] = ["manifest-untouched", "manifest-undeclared"];

/// The severity a format gives a downgradable check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckLevel {
    /// As built in.
    Error,
    /// Reported, does not stop the plan.
    Warning,
    /// Not reported.
    Off,
}

impl FormatSettings {
    /// The shell that runs a fence in `language`, if any.
    pub fn shell_for(&self, language: &str) -> Option<Shell> {
        if self.pwsh_languages.iter().any(|l| l == language) {
            return Some(Shell::Pwsh);
        }
        if self.bash_languages.iter().any(|l| l == language) {
            return Some(Shell::Bash);
        }
        None
    }

    /// `diagnostics` with this format's check levels applied.
    pub fn apply_checks(&self, diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
        let level = |code: &str| {
            self.checks
                .iter()
                .find(|(c, _)| *c == code)
                .map(|(_, level)| *level)
        };
        diagnostics
            .into_iter()
            .filter_map(|mut d| match level(d.code) {
                Some(CheckLevel::Off) => None,
                Some(CheckLevel::Warning) => {
                    d.severity = Severity::Warning;
                    Some(d)
                }
                _ => Some(d),
            })
            .collect()
    }

    /// True when `text` matches one of the manual markers.
    pub fn is_marked(&self, text: &str) -> bool {
        self.manual_markers.iter().any(|m| m.is_match(text))
    }
}
```

File `crates/planrun-parse/src/kinds/file_block.rs` (full contents):

```rust
//! `file-block`: a lead paragraph naming a path and an operation, then one fence →
//! `WriteFile` or `ApplyPatch`.

use std::sync::Arc;

use planrun_model::{
    Action, ActionKind, ChangeKind, Manifest, ManualReason, RecognizerId, RepoPath, Span, TextRef,
    WriteMode,
};
use regex::Regex;

use super::params::Params;
use super::{is_bare, paragraph, resolved_lead_path};
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
        let by_file_name = p.opt_bool("file_name_from_manifest")?.unwrap_or(false);
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
            by_file_name,
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
    /// Resolve a bare file name against the manifest (decision D32).
    by_file_name: bool,
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
        let (path, path_origin) =
            match resolved_lead_path(cx, &caps, base, lead.span, self.by_file_name) {
                Ok(resolved) => resolved,
                Err(d) => return Recognized::Malformed(d),
            };
        let Block::Fence { body, .. } = &fence.block else {
            return Recognized::No;
        };
        let span = lead.span.join(fence.span);
        let kind = match op {
            Op::Patch => match patch_action(cx, path, body, lead.span, fence.span) {
                Ok(kind) => kind,
                Err(d) => return Recognized::Malformed(d),
            },
            Op::Write => write(path.clone(), body, mode_from(cx.manifest, &path)),
            Op::Create => write(path, body, WriteMode::Create),
            Op::Overwrite => write(path, body, WriteMode::Overwrite),
        };
        let kind = ambiguous_create(cx, kind, span);
        cursor.advance(2);
        Recognized::Yes(vec![Action {
            kind,
            span,
            origin: self.id.clone(),
            path_origin,
        }])
    }
}

/// A Create whose path is declared only as a `Bare` entry: its directory is unknown, so it is
/// `Manual(AmbiguousPath)` instead of a write (decision D45). Modify and Rewrite stay actions:
/// preflight catches a wrong target.
fn ambiguous_create(cx: &StepContext<'_>, kind: ActionKind, span: Span) -> ActionKind {
    let create = matches!(
        &kind,
        ActionKind::WriteFile { path, mode: WriteMode::Create, .. } if is_bare(cx, path)
    );
    if !create {
        return kind;
    }
    ActionKind::Manual {
        reason: ManualReason::AmbiguousPath,
        text: TextRef::Span(span),
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
        .find(|e| e.path.file() == Some(path))
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
```

File `crates/planrun-parse/src/kinds/mod.rs` (full contents):

```rust
//! The built-in recognizer kinds (docs/extending.md §3).

mod before_after;
mod ceremony;
mod delete;
mod expected;
mod file_block;
mod generated;
mod manual_prose;
mod params;
mod run_line;
mod script;

use std::sync::Arc;

use planrun_model::{
    Action, ActionKind, ChangeKind, ManifestEntry, PathOrigin, RepoPath, Span, TextRef,
};
use regex::Captures;

use crate::diagnostic::Diagnostic;
use crate::document::{Block, Located};
use crate::recognizer::{RecognizerFactory, StepContext};

pub use before_after::BeforeAfterFactory;
pub use ceremony::CeremonyFactory;
pub use delete::DeleteFactory;
pub use expected::{ExpectedBlockFactory, ExpectedProseFactory};
pub use file_block::FileBlockFactory;
pub use generated::GeneratedFactory;
pub use manual_prose::ManualProseFactory;
pub use run_line::RunLineFactory;
pub use script::ScriptFactory;

/// Every built-in factory, for [`crate::Registry::with_builtins`].
pub fn builtin_factories() -> Vec<Arc<dyn RecognizerFactory>> {
    vec![
        Arc::new(FileBlockFactory),
        Arc::new(CeremonyFactory),
        Arc::new(DeleteFactory),
        Arc::new(GeneratedFactory),
        Arc::new(ScriptFactory),
        Arc::new(ExpectedBlockFactory),
        Arc::new(ExpectedProseFactory),
        Arc::new(RunLineFactory),
        Arc::new(BeforeAfterFactory),
        Arc::new(ManualProseFactory),
    ]
}

/// The text of a paragraph block, if it is one.
fn paragraph<'a>(cx: &'a StepContext<'_>, block: &'a Located) -> Option<(&'a str, u32)> {
    let Block::Paragraph { text } = &block.block else {
        return None;
    };
    Some((cx.doc.text(text), text.span().start))
}

/// The span of capture `name` in text that starts at `base`.
fn capture_span(caps: &Captures<'_>, name: &str, base: u32) -> Option<Span> {
    let m = caps.name(name)?;
    Some(Span::new(base + offset(m.start()), base + offset(m.end())))
}

/// The capture `name` as a span reference.
fn capture_ref(caps: &Captures<'_>, name: &str, base: u32) -> Option<TextRef> {
    capture_span(caps, name, base).map(TextRef::Span)
}

/// The `path` capture as a repository path, or a `bad-path` error at the capture.
fn lead_path(caps: &Captures<'_>, base: u32, lead: Span) -> Result<RepoPath, Diagnostic> {
    let span = capture_span(caps, "path", base).unwrap_or(lead);
    let raw = caps.name("path").map_or("", |m| m.as_str());
    RepoPath::new(raw).map_err(|e| Diagnostic::error("bad-path", Some(span), e.to_string()))
}

/// The `path` capture as a repository path, resolved against the task's manifest when
/// `by_file_name` (decision D32): a path without `/` that is no entry itself becomes the one
/// entry with that file name, when there is exactly one and it was written in full.
fn resolved_lead_path(
    cx: &StepContext<'_>,
    caps: &Captures<'_>,
    base: u32,
    lead: Span,
    by_file_name: bool,
) -> Result<(RepoPath, PathOrigin), Diagnostic> {
    let path = lead_path(caps, base, lead)?;
    let entries = &cx.manifest.entries;
    let is_entry = entries.iter().any(|e| e.path.file() == Some(&path));
    if !by_file_name || path.as_str().contains('/') || is_entry {
        return Ok((path, PathOrigin::Written));
    }
    let same: Vec<(&RepoPath, &ManifestEntry)> = entries
        .iter()
        .filter_map(|e| e.path.file().map(|file| (file, e)))
        .filter(|(file, _)| file.file_name() == path.as_str())
        .collect();
    let first = same.first().map(|(file, _)| *file);
    let one = first.filter(|p| {
        same.iter()
            .all(|(file, e)| file == p && e.origin.is_written())
    });
    match one {
        Some(entry) => Ok((entry.clone(), PathOrigin::FileName)),
        None => Ok((path, PathOrigin::Written)),
    }
}

/// The task's one manifest entry an edit with no path can mean (decision D32): not Test,
/// Generated or Delete, and never a pattern (D41). `None` with none or several. A `Bare` entry
/// counts as one of several but is never the answer (D45).
fn only_entry(cx: &StepContext<'_>) -> Option<RepoPath> {
    let mut editable = cx.manifest.entries.iter().filter(|e| {
        let editable = !matches!(
            e.change,
            ChangeKind::Test | ChangeKind::Generated | ChangeKind::Delete
        );
        editable && e.path.file().is_some()
    });
    let first = editable.next()?;
    let path = first.path.file()?;
    let one = editable.all(|e| e.path.file() == Some(path) && e.origin != PathOrigin::Bare);
    (one && first.origin != PathOrigin::Bare).then(|| path.clone())
}

/// True when `path` is declared only as a `Bare` entry (decision D45).
fn is_bare(cx: &StepContext<'_>, path: &RepoPath) -> bool {
    cx.manifest
        .entries
        .iter()
        .any(|e| e.path.file() == Some(path) && e.origin == PathOrigin::Bare)
}

/// The script of the action an expectation amends: the last action, when it is a script.
fn amendable(cx: &StepContext<'_>, lead: Span) -> Result<Action, Diagnostic> {
    let script = cx
        .previous
        .last()
        .filter(|a| matches!(a.kind, ActionKind::RunScript(_) | ActionKind::Commit(_)));
    let Some(action) = script else {
        return Err(Diagnostic::error(
            "expected-without-script",
            Some(lead),
            "an expectation with no script directly before it",
        ));
    };
    let already = match &action.kind {
        ActionKind::RunScript(s) | ActionKind::Commit(s) => s.expect.is_some(),
        _ => false,
    };
    if already {
        return Err(Diagnostic::error(
            "expected-twice",
            Some(lead),
            "the script before this already has an expectation",
        )
        .with_related(action.span, "the script"));
    }
    Ok(action.clone())
}

fn offset(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
```

File `crates/planrun-parse/src/lib.rs` (full contents):

```rust
//! Markdown plan → typed [`planrun_model::Plan`] (docs/architecture.md §4–§5).
//!
//! Pipeline: read and normalize ([`source`]) → block stream ([`document`]) → pick a format
//! ([`registry`]) → outline: tasks, manifests, steps ([`format`], [`outline`]) → recognizers turn
//! each step's blocks into actions ([`recognize`], [`recognizer`], the kinds in `kinds`) →
//! static checks ([`check`]).
//!
//! This crate never touches the repository and never runs anything (decision D2). Checks that
//! need the repository (preimage blob ids against the files on disk) live in planrun-exec.
//!
//! Extension points, in the order a new plan style should try them (docs/extending.md):
//! 1. a format spec in TOML that reuses the built-in recognizer kinds ([`spec`]);
//! 2. a new [`recognizer::RecognizerFactory`] for a new idiom;
//! 3. a hand-written [`format::Format`] when the outline is not heading-shaped.

pub mod check;
pub mod diagnostic;
pub mod document;
pub mod format;
mod kinds;
pub mod manifest;
pub mod outline;
pub mod patch;
pub mod recognize;
pub mod recognizer;
pub mod registry;
pub mod source;
pub mod spec;

pub use diagnostic::{Diagnostic, Diagnostics, Severity};
pub use recognize::TaskCoverage;
pub use registry::Registry;

use planrun_model::Plan;

use crate::document::Document;
use crate::format::{FormatSettings, Outline};

/// A plan that parsed and checked without errors, with its warnings and notes and its coverage
/// report.
#[derive(Debug, Clone)]
pub struct Parsed {
    /// The plan.
    pub plan: Plan,
    /// Warnings and notes, in source order.
    pub diagnostics: Diagnostics,
    /// Per task, where its blocks went.
    pub coverage: Vec<TaskCoverage>,
}

/// Parses and checks `bytes` as a plan named `name`, with the formats in `registry`.
///
/// `format` forces a format by id; otherwise the one with the highest detection score wins, and
/// a tie or a score below the registry's threshold is an error (never a guess).
///
/// A plan is returned only when there is no error (`Never execute a partially parsed plan`,
/// decision D3); otherwise every diagnostic comes back, in source order. Check rules run only
/// when the outline and recognition had no errors.
pub fn parse_plan(
    registry: &Registry,
    name: &str,
    bytes: &[u8],
    format: Option<&str>,
) -> Result<Parsed, Diagnostics> {
    let source = source::load(name, bytes).map_err(Diagnostics::single)?;
    let doc = Document::build(source)?;
    let format = registry.select(&doc, format).map_err(Diagnostics::single)?;
    let outline = format.outline(&doc)?;
    let (tasks, coverage) = recognize::recognize(format.as_ref(), &doc, &outline)?;
    let mut found = outside_notes(&doc, format.settings(), &outline);
    let plan = Plan {
        source: doc.source,
        format: format.id().clone(),
        meta: outline.meta,
        tasks,
    };
    let mut checked = Vec::new();
    for rule in check::default_rules() {
        rule.check(&plan, &mut checked);
    }
    found.extend(format.settings().apply_checks(checked));
    let mut diagnostics = Diagnostics(found);
    diagnostics.sort();
    if diagnostics.has_errors() {
        return Err(diagnostics);
    }
    Ok(Parsed {
        plan,
        diagnostics,
        coverage,
    })
}

/// Manual markers outside the tasks: before task 1 (read them first) and after the tasks (not
/// run).
fn outside_notes(doc: &Document, settings: &FormatSettings, outline: &Outline) -> Vec<Diagnostic> {
    let before = outline.meta.preamble.iter().map(|text| {
        Diagnostic::note(
            "manual-outside-tasks",
            Some(text.span()),
            "read this before task 1: it is an instruction for you",
        )
    });
    let after = outline
        .after_tasks
        .map(|range| outline::marked(doc, settings, range))
        .unwrap_or_default();
    let after = after.into_iter().map(|text| {
        Diagnostic::note(
            "manual-outside-tasks",
            Some(text.span()),
            "after the tasks: planrun does not run this",
        )
    });
    before.chain(after).collect()
}
```

File `crates/planrun-parse/src/manifest.rs` (full contents):

```rust
//! The manifest of a task (docs/architecture.md §4.3): the Files marker, its list items or its
//! inline clauses, and the entries they declare (decisions D27, D32, D41, D43–D45).

use planrun_model::{
    ChangeKind, LineRange, Manifest, ManifestEntry, ManifestPath, PathOrigin, PathPattern,
    RepoPath, Span, TextRef,
};
use regex::Regex;

use crate::diagnostic::Diagnostic;
use crate::document::{Block, BlockRange, Document};

/// A manifest: a marker paragraph and the list items after it, or the clauses inside it.
pub struct ManifestRules {
    /// Over the marker paragraph; may capture `change` for items that are bare paths, and
    /// `items` for a manifest written inline (D43).
    pub marker: Regex,
    /// Over each item or clause; captures `paths`, and `change` unless the marker does.
    pub item: Regex,
    /// The captured change word and what it means.
    pub changes: Vec<(String, ChangeKind)>,
    /// Splits `paths` into several, when set.
    pub path_list: Option<Regex>,
    /// Splits the marker's `items` capture into clauses, when set (D43).
    pub clause_list: Option<Regex>,
    /// Over the text right after a path token; its `change` capture changes that one path
    /// (D44).
    pub path_change: Option<Regex>,
    /// When false, a task may have no manifest (decision D27).
    pub required: bool,
    /// A bare file name after a path with a directory is in that directory (decision D32).
    pub siblings: bool,
}

/// What the outline found for a task's manifest.
#[derive(Default)]
pub struct Found {
    /// The entries.
    pub manifest: Manifest,
    /// The marker and its list items: consumed, in no step.
    pub blocks: Option<BlockRange>,
    /// Text of an inline marker that declares nothing: shown in place as a Note (D43).
    pub leftovers: Vec<TextRef>,
}

/// The manifest: the first block before any step boundary that matches the marker, and the list
/// items right after it, up to the first step boundary (a list-item step right after the items
/// is in the same CommonMark list, decision D37). A marker whose `items` capture took part holds
/// the manifest itself (D43).
pub fn manifest(
    spec: &ManifestRules,
    doc: &Document,
    head: usize,
    body: BlockRange,
    boundary: impl Fn(usize) -> bool,
    errors: &mut Vec<Diagnostic>,
) -> Found {
    let marker = (body.start..body.end)
        .take_while(|&i| !boundary(i))
        .find(|&i| matches!(&doc.blocks[i].block, Block::Paragraph { text } if spec.marker.is_match(doc.text(text))));
    let Some(marker) = marker else {
        if spec.required {
            errors.push(Diagnostic::error(
                "manifest-missing",
                Some(doc.blocks[head].span),
                format!("the task has no manifest (`{}`)", spec.marker.as_str()),
            ));
        }
        return Found::default();
    };
    let Block::Paragraph { text } = &doc.blocks[marker].block else {
        return Found::default();
    };
    let marker_text = doc.text(text);
    let base = text.span().start;
    let caps = spec.marker.captures(marker_text);
    let marker_change = caps
        .as_ref()
        .and_then(|c| c.name("change"))
        .map(|m| m.as_str());
    let depth = doc.blocks[marker].depth;
    let items_end = (marker + 1..body.end)
        .find(|&i| {
            let b = &doc.blocks[i];
            !matches!(b.block, Block::ListItem { .. }) || b.depth != depth || boundary(i)
        })
        .unwrap_or(body.end);
    let mut entries = Vec::new();
    let mut leftovers = Vec::new();
    let inline = caps.as_ref().and_then(|c| c.name("items"));
    let end = match inline {
        Some(items) => {
            if items_end > marker + 1 {
                errors.push(Diagnostic::error(
                    "manifest-item",
                    Some(doc.blocks[marker + 1].span),
                    "a list after a Files line that already names its files",
                ));
            }
            let at = Inline {
                text: marker_text,
                base,
                items,
                marker_change,
            };
            inline_items(spec, &at, &mut entries, &mut leftovers, errors);
            marker + 1
        }
        None => {
            if items_end == marker + 1 {
                errors.push(Diagnostic::error(
                    "manifest-item",
                    Some(doc.blocks[marker].span),
                    "the manifest has no items",
                ));
            }
            for i in marker + 1..items_end {
                list_item(spec, doc, i, marker_change, &mut entries, errors);
            }
            items_end
        }
    };
    let range = BlockRange { start: marker, end };
    let last = doc.blocks[end - 1].span.end;
    Found {
        manifest: Manifest {
            entries,
            span: Some(Span::new(doc.blocks[marker].span.start, last)),
        },
        blocks: Some(range),
        leftovers,
    }
}

/// The marker paragraph of an inline manifest and its `items` capture.
struct Inline<'a> {
    text: &'a str,
    base: u32,
    items: regex::Match<'a>,
    marker_change: Option<&'a str>,
}

/// The clauses of an inline manifest (D43). A clause with a code span is an item; one without
/// declares nothing and is a leftover, as is any text after the `items` capture.
fn inline_items(
    spec: &ManifestRules,
    at: &Inline<'_>,
    entries: &mut Vec<ManifestEntry>,
    leftovers: &mut Vec<TextRef>,
    errors: &mut Vec<Diagnostic>,
) {
    let items = at.items.as_str();
    let mut clauses: Vec<(usize, &str)> = Vec::new();
    let mut from = 0;
    if let Some(split) = &spec.clause_list {
        for sep in split.find_iter(items) {
            clauses.push((from, &items[from..sep.start()]));
            from = sep.end();
        }
    }
    clauses.push((from, &items[from..]));
    let start = at.items.start();
    for (offset_in_items, clause) in clauses {
        let trimmed = clause.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lead = clause.len() - clause.trim_start().len();
        let at_start = at.base + offset(start + offset_in_items + lead);
        let span = Span::new(at_start, at_start + offset(trimmed.len()));
        if !trimmed.contains('`') {
            leftovers.push(TextRef::Span(span));
            continue;
        }
        let item = Item {
            text: trimmed,
            base: at_start,
            span,
            marker_change: at.marker_change,
        };
        item_entries(spec, &item, entries, errors);
    }
    let tail = &at.text[at.items.end()..];
    let rest = tail.trim_start_matches(|c: char| c == '.' || c.is_whitespace());
    if !rest.trim().is_empty() {
        let rest_at = at.base + offset(at.text.len() - rest.len());
        let rest = rest.trim_end();
        leftovers.push(TextRef::Span(Span::new(
            rest_at,
            rest_at + offset(rest.len()),
        )));
    }
}

/// One list item of a manifest.
fn list_item(
    spec: &ManifestRules,
    doc: &Document,
    index: usize,
    marker_change: Option<&str>,
    entries: &mut Vec<ManifestEntry>,
    errors: &mut Vec<Diagnostic>,
) {
    let located = &doc.blocks[index];
    let Block::ListItem { text, .. } = &located.block else {
        return;
    };
    let item = Item {
        text: doc.text(text),
        base: text.span().start,
        span: located.span,
        marker_change,
    };
    item_entries(spec, &item, entries, errors);
}

/// The text of one item or clause.
struct Item<'a> {
    text: &'a str,
    base: u32,
    span: Span,
    marker_change: Option<&'a str>,
}

/// The entries of one item or clause: its change (captured by the item, or else by the marker)
/// and its paths.
fn item_entries(
    spec: &ManifestRules,
    item: &Item<'_>,
    entries: &mut Vec<ManifestEntry>,
    errors: &mut Vec<Diagnostic>,
) {
    let parsed = spec.item.captures(item.text).and_then(|caps| {
        let word = caps
            .name("change")
            .map(|m| m.as_str())
            .or(item.marker_change)?;
        Some((change_named(spec, word)?, caps.name("paths")?))
    });
    let Some((change, paths)) = parsed else {
        errors.push(Diagnostic::error(
            "manifest-item",
            Some(item.span),
            format!(
                "a manifest item that does not match `{}`",
                spec.item.as_str()
            ),
        ));
        return;
    };
    let paths_span = Span::new(
        item.base + offset(paths.start()),
        item.base + offset(paths.end()),
    );
    let tail = &item.text[paths.end()..];
    let mut previous: Option<(RepoPath, PathOrigin)> = None;
    let mut after_dir = false;
    for piece in pieces(spec, paths.as_str(), tail) {
        let context = Context {
            previous: previous.as_ref(),
            after_dir,
        };
        let (path, origin, lines, own) = match token(spec, piece, &context) {
            Ok(token) => token,
            Err((code, message)) => {
                errors.push(Diagnostic::error(code, Some(paths_span), message));
                continue;
            }
        };
        after_dir |= path.to_string().contains('/');
        previous = path.file().map(|file| (file.clone(), origin));
        entries.push(ManifestEntry {
            path,
            change: own.unwrap_or(change),
            lines,
            span: item.span,
            origin,
        });
    }
}

fn change_named(spec: &ManifestRules, word: &str) -> Option<ChangeKind> {
    spec.changes
        .iter()
        .find(|(w, _)| w == word)
        .map(|(_, k)| *k)
}

/// A piece of an item's paths: one token, or a range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Piece<'a> {
    /// A path, a pattern or a `:lines` token, and the text right after it up to the next token
    /// (a separator, or the rest of the item).
    Token(&'a str, &'a str),
    /// `` `a`–`b` ``: an en dash between two code spans (decision D41).
    Range(&'a str, &'a str),
}

/// The pieces of `paths` (followed in the item by `tail`). Ranges are found first, in every
/// format, so a spec's `path_list` never sees their en dash; the text around them is split by
/// `path_list` (D41).
fn pieces<'a>(spec: &ManifestRules, paths: &'a str, tail: &'a str) -> Vec<Piece<'a>> {
    const DASH: &str = "`–`";
    let mut out = Vec::new();
    let mut rest = paths;
    let mut after_range = false;
    while let Some(at) = rest.find(DASH) {
        let (before, after) = (&rest[..at], &rest[at + DASH.len()..]);
        let start_at = before.rfind('`').map_or(0, |i| i + 1);
        let end_len = after.find('`').unwrap_or(after.len());
        out.extend(split(spec, &before[..start_at], "", (after_range, true)));
        out.push(Piece::Range(&before[start_at..], &after[..end_len]));
        rest = &after[end_len..];
        after_range = true;
    }
    out.extend(split(spec, rest, tail, (after_range, false)));
    out
}

/// `text` split by `path_list`, each token with the separator after it (the last with `tail`).
/// Next to a range, the empty piece on its side is what is left of the separator, not an empty
/// path: `after_range` drops a leading one, `before_range` a trailing one.
fn split<'a>(
    spec: &ManifestRules,
    text: &'a str,
    tail: &'a str,
    (after_range, before_range): (bool, bool),
) -> Vec<Piece<'a>> {
    if (after_range || before_range) && text.trim().is_empty() {
        return Vec::new();
    }
    let mut parts: Vec<(&str, &str)> = Vec::new();
    let mut from = 0;
    if let Some(split) = &spec.path_list {
        for sep in split.find_iter(text) {
            parts.push((&text[from..sep.start()], sep.as_str()));
            from = sep.end();
        }
    }
    parts.push((&text[from..], tail));
    if before_range && parts.last().is_some_and(|(p, _)| p.trim().is_empty()) {
        parts.pop();
    }
    if after_range && parts.first().is_some_and(|(p, _)| p.trim().is_empty()) {
        parts.remove(0);
    }
    parts
        .into_iter()
        .map(|(p, after)| Piece::Token(p.trim(), after))
        .collect()
}

/// A diagnostic code and message.
type Refusal = (&'static str, String);

/// What a token knows of the tokens before it in its item.
struct Context<'a> {
    /// The file before it.
    previous: Option<&'a (RepoPath, PathOrigin)>,
    /// Some earlier file token had a directory.
    after_dir: bool,
}

/// One entry: the path, its origin, its lines, and its own change (D44).
type Token = (
    ManifestPath,
    PathOrigin,
    Option<LineRange>,
    Option<ChangeKind>,
);

/// One piece of a manifest item (decisions D32, D41, D44, D45): `:3-10` is more lines of the
/// previous path; with `siblings`, a bare file name after a path with a directory is in that
/// directory, without them it is `Bare`; a token with `*`, `?` or `{`, and a range, are
/// patterns; `path_change` over the text after a token gives that path its own change.
fn token(spec: &ManifestRules, piece: Piece<'_>, cx: &Context<'_>) -> Result<Token, Refusal> {
    let (piece, after) = match piece {
        Piece::Range(start, end) => {
            let pattern =
                PathPattern::range(start, end).map_err(|e| ("manifest-item", e.to_string()))?;
            return Ok((
                ManifestPath::Pattern(pattern),
                PathOrigin::Written,
                None,
                None,
            ));
        }
        Piece::Token(piece, after) => (piece, after),
    };
    let own = own_change(spec, after)?;
    if let Some(range) = piece.strip_prefix(':') {
        let Some((path, origin)) = cx.previous else {
            return Err(("bad-path", format!("`{piece}` is not a path")));
        };
        let lines = line_range(range);
        return Ok((ManifestPath::File(path.clone()), *origin, lines, own));
    }
    if PathPattern::is_glob(piece) {
        let pattern = PathPattern::glob(piece).map_err(|e| ("manifest-item", e.to_string()))?;
        return Ok((
            ManifestPath::Pattern(pattern),
            PathOrigin::Written,
            None,
            own,
        ));
    }
    let (path, lines) = split_lines(piece);
    let dir = cx.previous.and_then(|(p, _)| p.dir());
    let bare = !path.contains('/');
    let (spelled, origin) = match dir {
        Some(dir) if spec.siblings && bare => {
            (RepoPath::new(&format!("{dir}/{path}")), PathOrigin::Sibling)
        }
        _ if !spec.siblings && bare && cx.after_dir => (RepoPath::new(path), PathOrigin::Bare),
        _ => (RepoPath::new(path), PathOrigin::Written),
    };
    let path = spelled.map_err(|e| ("bad-path", e.to_string()))?;
    Ok((ManifestPath::File(path), origin, lines, own))
}

/// The change `path_change` gives the token followed by `after`, mapped through `changes`.
fn own_change(spec: &ManifestRules, after: &str) -> Result<Option<ChangeKind>, Refusal> {
    let Some(word) = spec
        .path_change
        .as_ref()
        .and_then(|r| r.captures(after))
        .and_then(|c| c.name("change"))
    else {
        return Ok(None);
    };
    match change_named(spec, word.as_str()) {
        Some(kind) => Ok(Some(kind)),
        None => Err((
            "manifest-item",
            format!("`{}` is no change this format knows", word.as_str()),
        )),
    }
}

/// `src/a.rs:3-10` → (`src/a.rs`, 3–10); `src/a.rs:7` → (`src/a.rs`, 7–7).
fn split_lines(path: &str) -> (&str, Option<LineRange>) {
    let Some((file, range)) = path.rsplit_once(':') else {
        return (path, None);
    };
    match line_range(range) {
        Some(lines) => (file, Some(lines)),
        None => (path, None),
    }
}

/// `3-10` or `7`.
fn line_range(range: &str) -> Option<LineRange> {
    let (start, end) = range.split_once('-').unwrap_or((range, range));
    Some(LineRange {
        start: start.parse().ok()?,
        end: end.parse().ok()?,
    })
}

fn offset(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_ranges_are_split_from_paths() {
        assert_eq!(
            split_lines("src/a.rs:3-10"),
            ("src/a.rs", Some(LineRange { start: 3, end: 10 }))
        );
        assert_eq!(
            split_lines("src/a.rs:7"),
            ("src/a.rs", Some(LineRange { start: 7, end: 7 }))
        );
        assert_eq!(split_lines("src/a.rs"), ("src/a.rs", None));
        assert_eq!(split_lines("a:b"), ("a:b", None));
    }
}
```

File `crates/planrun-parse/src/outline.rs` (exact diff; save the block as `$env:TEMP\p2b-t2-outline-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/outline.rs b/crates/planrun-parse/src/outline.rs
index 18742dc..e1a2b23 100644
--- a/crates/planrun-parse/src/outline.rs
+++ b/crates/planrun-parse/src/outline.rs
@@ -2,15 +2,13 @@
 //! tasks, each task's manifest and its steps. [`crate::spec::SpecFormat`] uses it; a hand-written
 //! format may too.
 
-use planrun_model::{
-    ChangeKind, LineRange, Manifest, ManifestEntry, ManifestPath, PathOrigin, PathPattern,
-    PlanMeta, RepoPath, Span, TaskId, TextRef,
-};
+use planrun_model::{PlanMeta, Span, TaskId, TextRef};
 use regex::{Captures, Regex};
 
 use crate::diagnostic::{Diagnostic, Diagnostics};
 use crate::document::{Block, BlockRange, Document};
 use crate::format::{FormatSettings, Outline, OutlineStep, OutlineTask, StepMark};
+use crate::manifest::{self, Found, ManifestRules};
 
 /// The compiled `[outline]` of a spec.
 pub struct OutlineRules {
@@ -36,22 +34,6 @@ pub struct HeadingRule {
     pub regex: Regex,
 }
 
-/// A manifest: a marker paragraph and the list items after it.
-pub struct ManifestRules {
-    /// Over the marker paragraph; may capture `change` for items that are bare paths.
-    pub marker: Regex,
-    /// Over each item; captures `paths`, and `change` unless the marker does.
-    pub item: Regex,
-    /// The captured change word and what it means.
-    pub changes: Vec<(String, ChangeKind)>,
-    /// Splits `paths` into several, when set.
-    pub path_list: Option<Regex>,
-    /// When false, a task may have no manifest (decision D27).
-    pub required: bool,
-    /// A bare file name after a path with a directory is in that directory (decision D32).
-    pub siblings: bool,
-}
-
 /// A block that starts a step.
 pub struct StepRule {
     /// Which kind of block.
@@ -275,8 +257,15 @@ fn task(
         start: head + 1,
         end: next,
     };
-    let (manifest, manifest_blocks) = manifest(rules, doc, head, body, errors);
-    let steps = steps(rules, doc, body, manifest_blocks, errors);
+    let found = match &rules.manifest {
+        Some(spec) => {
+            let boundary = |i| boundary(rules, doc, i).is_some();
+            manifest::manifest(spec, doc, head, body, boundary, errors)
+        }
+        None => Found::default(),
+    };
+    let has_notes = !found.leftovers.is_empty();
+    let steps = steps(rules, doc, body, (found.blocks, has_notes), errors);
     let span = Span::new(
         located.span.start,
         range_end(doc, body).unwrap_or(located.span.end),
@@ -288,8 +277,9 @@ fn task(
         },
         title: capture("title").unwrap_or(heading),
         span,
-        manifest,
-        manifest_blocks,
+        manifest: found.manifest,
+        manifest_blocks: found.blocks,
+        manifest_notes: found.leftovers,
         steps,
     };
     (task, phase)
@@ -303,243 +293,6 @@ fn capture_ref(caps: Option<&Captures<'_>>, name: &str, base: u32) -> Option<Tex
     )))
 }
 
-/// The manifest: the first block before any step boundary that matches the marker, and the list
-/// items right after it, up to the first step boundary (a list-item step right after the items
-/// is in the same CommonMark list, decision D37).
-fn manifest(
-    rules: &OutlineRules,
-    doc: &Document,
-    head: usize,
-    body: BlockRange,
-    errors: &mut Vec<Diagnostic>,
-) -> (Manifest, Option<BlockRange>) {
-    let Some(spec) = &rules.manifest else {
-        return (Manifest::default(), None);
-    };
-    let marker = (body.start..body.end)
-        .take_while(|&i| boundary(rules, doc, i).is_none())
-        .find(|&i| matches!(&doc.blocks[i].block, Block::Paragraph { text } if spec.marker.is_match(doc.text(text))));
-    let Some(marker) = marker else {
-        if spec.required {
-            errors.push(Diagnostic::error(
-                "manifest-missing",
-                Some(doc.blocks[head].span),
-                format!("the task has no manifest (`{}`)", spec.marker.as_str()),
-            ));
-        }
-        return (Manifest::default(), None);
-    };
-    let marker_change = match &doc.blocks[marker].block {
-        Block::Paragraph { text } => spec
-            .marker
-            .captures(doc.text(text))
-            .and_then(|c| c.name("change"))
-            .map(|m| m.as_str()),
-        _ => None,
-    };
-    let depth = doc.blocks[marker].depth;
-    let items_end = (marker + 1..body.end)
-        .find(|&i| {
-            let b = &doc.blocks[i];
-            !matches!(b.block, Block::ListItem { .. })
-                || b.depth != depth
-                || boundary(rules, doc, i).is_some()
-        })
-        .unwrap_or(body.end);
-    if items_end == marker + 1 {
-        errors.push(Diagnostic::error(
-            "manifest-item",
-            Some(doc.blocks[marker].span),
-            "the manifest has no items",
-        ));
-    }
-    let mut entries = Vec::new();
-    for i in marker + 1..items_end {
-        manifest_item(spec, doc, i, marker_change, &mut entries, errors);
-    }
-    let range = BlockRange {
-        start: marker,
-        end: items_end,
-    };
-    let span = Span::new(
-        doc.blocks[marker].span.start,
-        range_end(doc, range).unwrap_or(doc.blocks[marker].span.end),
-    );
-    let manifest = Manifest {
-        entries,
-        span: Some(span),
-    };
-    (manifest, Some(range))
-}
-
-/// One manifest item: its change (captured by the item, or else by the marker) and its paths.
-fn manifest_item(
-    spec: &ManifestRules,
-    doc: &Document,
-    index: usize,
-    marker_change: Option<&str>,
-    entries: &mut Vec<ManifestEntry>,
-    errors: &mut Vec<Diagnostic>,
-) {
-    let located = &doc.blocks[index];
-    let Block::ListItem { text, .. } = &located.block else {
-        return;
-    };
-    let base = text.span().start;
-    let item = doc.text(text);
-    let parsed = spec.item.captures(item).and_then(|caps| {
-        let change = caps.name("change").map(|m| m.as_str()).or(marker_change)?;
-        let kind = spec.changes.iter().find(|(w, _)| w == change)?.1;
-        Some((kind, caps.name("paths")?))
-    });
-    let Some((change, paths)) = parsed else {
-        errors.push(Diagnostic::error(
-            "manifest-item",
-            Some(located.span),
-            format!(
-                "a manifest item that does not match `{}`",
-                spec.item.as_str()
-            ),
-        ));
-        return;
-    };
-    let paths_span = Span::new(base + offset(paths.start()), base + offset(paths.end()));
-    let mut previous: Option<(RepoPath, PathOrigin)> = None;
-    for piece in pieces(spec, paths.as_str()) {
-        let (path, origin, lines) = match token(spec, piece, previous.as_ref()) {
-            Ok(token) => token,
-            Err((code, message)) => {
-                errors.push(Diagnostic::error(code, Some(paths_span), message));
-                continue;
-            }
-        };
-        previous = path.file().map(|file| (file.clone(), origin));
-        entries.push(ManifestEntry {
-            path,
-            change,
-            lines,
-            span: located.span,
-            origin,
-        });
-    }
-}
-
-/// A piece of an item's paths: one token, or a range.
-#[derive(Debug, Clone, Copy, PartialEq, Eq)]
-enum Piece<'a> {
-    /// A path, a pattern or a `:lines` token.
-    Token(&'a str),
-    /// `` `a`–`b` ``: an en dash between two code spans (decision D41).
-    Range(&'a str, &'a str),
-}
-
-/// The pieces of `paths`. Ranges are found first, in every format, so a spec's `path_list` never
-/// sees their en dash; the text around them is split by `path_list` (D41).
-fn pieces<'a>(spec: &ManifestRules, paths: &'a str) -> Vec<Piece<'a>> {
-    const DASH: &str = "`–`";
-    let mut out = Vec::new();
-    let mut rest = paths;
-    let mut after_range = false;
-    while let Some(at) = rest.find(DASH) {
-        let (before, after) = (&rest[..at], &rest[at + DASH.len()..]);
-        let start_at = before.rfind('`').map_or(0, |i| i + 1);
-        let end_len = after.find('`').unwrap_or(after.len());
-        out.extend(split(spec, &before[..start_at], after_range, true));
-        out.push(Piece::Range(&before[start_at..], &after[..end_len]));
-        rest = &after[end_len..];
-        after_range = true;
-    }
-    out.extend(split(spec, rest, after_range, false));
-    out
-}
-
-/// `text` split by `path_list`. Next to a range, the empty piece on its side is what is left of
-/// the separator, not an empty path: `after_range` drops a leading one, `before_range` a
-/// trailing one.
-fn split<'a>(
-    spec: &ManifestRules,
-    text: &'a str,
-    after_range: bool,
-    before_range: bool,
-) -> Vec<Piece<'a>> {
-    let next_to_range = after_range || before_range;
-    if next_to_range && text.trim().is_empty() {
-        return Vec::new();
-    }
-    let mut parts: Vec<&str> = match &spec.path_list {
-        Some(split) => split.split(text).collect(),
-        None => vec![text],
-    };
-    if before_range && parts.last().is_some_and(|p| p.trim().is_empty()) {
-        parts.pop();
-    }
-    if after_range && parts.first().is_some_and(|p| p.trim().is_empty()) {
-        parts.remove(0);
-    }
-    parts.into_iter().map(|p| Piece::Token(p.trim())).collect()
-}
-
-/// A diagnostic code and message.
-type Refusal = (&'static str, String);
-
-/// One piece of a manifest item, after the file `previous` (decisions D32, D41): `:3-10` is more
-/// lines of the previous path; with `siblings`, a bare file name after a path with a directory
-/// is in that directory; a token with `*`, `?` or `{`, and a range, are patterns.
-fn token(
-    spec: &ManifestRules,
-    piece: Piece<'_>,
-    previous: Option<&(RepoPath, PathOrigin)>,
-) -> Result<(ManifestPath, PathOrigin, Option<LineRange>), Refusal> {
-    let piece = match piece {
-        Piece::Range(start, end) => {
-            let pattern =
-                PathPattern::range(start, end).map_err(|e| ("manifest-item", e.to_string()))?;
-            return Ok((ManifestPath::Pattern(pattern), PathOrigin::Written, None));
-        }
-        Piece::Token(piece) => piece,
-    };
-    if let Some(range) = piece.strip_prefix(':') {
-        let Some((path, origin)) = previous else {
-            return Err(("bad-path", format!("`{piece}` is not a path")));
-        };
-        return Ok((ManifestPath::File(path.clone()), *origin, line_range(range)));
-    }
-    if PathPattern::is_glob(piece) {
-        let pattern = PathPattern::glob(piece).map_err(|e| ("manifest-item", e.to_string()))?;
-        return Ok((ManifestPath::Pattern(pattern), PathOrigin::Written, None));
-    }
-    let (path, lines) = split_lines(piece);
-    let dir = previous.and_then(|(p, _)| p.dir());
-    let (spelled, origin) = match dir {
-        Some(dir) if spec.siblings && !path.contains('/') => {
-            (RepoPath::new(&format!("{dir}/{path}")), PathOrigin::Sibling)
-        }
-        _ => (RepoPath::new(path), PathOrigin::Written),
-    };
-    let path = spelled.map_err(|e| ("bad-path", e.to_string()))?;
-    Ok((ManifestPath::File(path), origin, lines))
-}
-
-/// `src/a.rs:3-10` → (`src/a.rs`, 3–10); `src/a.rs:7` → (`src/a.rs`, 7–7).
-fn split_lines(path: &str) -> (&str, Option<LineRange>) {
-    let Some((file, range)) = path.rsplit_once(':') else {
-        return (path, None);
-    };
-    match line_range(range) {
-        Some(lines) => (file, Some(lines)),
-        None => (path, None),
-    }
-}
-
-/// `3-10` or `7`.
-fn line_range(range: &str) -> Option<LineRange> {
-    let (start, end) = range.split_once('-').unwrap_or((range, range));
-    Some(LineRange {
-        start: start.parse().ok()?,
-        end: end.parse().ok()?,
-    })
-}
-
 /// A step boundary's `n` capture.
 enum StepNumber {
     /// No `n` capture.
@@ -580,7 +333,7 @@ fn steps(
     rules: &OutlineRules,
     doc: &Document,
     body: BlockRange,
-    manifest: Option<BlockRange>,
+    (manifest, has_notes): (Option<BlockRange>, bool),
     errors: &mut Vec<Diagnostic>,
 ) -> Vec<OutlineStep> {
     let outside = |i: &usize| !manifest.is_some_and(|m| m.contains(*i));
@@ -594,11 +347,14 @@ fn steps(
         start: body.start,
         end: intro_end,
     };
-    if (intro.start..intro.end).any(|i| outside(&i)) {
+    if has_notes || (intro.start..intro.end).any(|i| outside(&i)) {
+        let marker = manifest.map(|m| doc.blocks[m.start].span);
         steps.push(OutlineStep {
             number: None,
             title: None,
-            span: range_span(doc, intro, manifest).unwrap_or(Span::new(0, 0)),
+            span: range_span(doc, intro, manifest)
+                .or(marker)
+                .unwrap_or(Span::new(0, 0)),
             blocks: intro,
             written: false,
             mark: None,
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t2-outline-rs.patch"
Remove-Item "$env:TEMP\p2b-t2-outline-rs.patch"
```

File `crates/planrun-parse/src/outline_tests.rs` (full contents):

```rust
use planrun_model::ChangeKind;

use super::*;
use crate::format::Format;
use crate::source;
use crate::spec::{FormatSpec, SpecFormat};

const SPEC: &str = r#"
id = "t"
name = "test"
[detect]
patterns = [{ regex = '^### P\d+', weight = 60 }]
[outline]
tasks_after = { level = 2, regex = '^Tasks$' }
task = { level = 3, regex = '^(?P<id>(?P<phase>P\d+[a-z]?(?:-\d+)?)-T(?P<ord>\d+)): (?P<title>.+)$' }
tasks_end_level = 2
[outline.manifest]
marker = '^\*\*Files\*\*$'
item = '^(?P<change>Create|Modify|Delete) `(?P<paths>[^`]+)`(?: \(.+\))?$'
changes = { Create = "create", Modify = "modify", Delete = "delete" }
[[outline.steps]]
block = "paragraph"
regex = '^\*\*Step (?P<n>\d+) — (?P<title>.+)\*\*$'
[settings]
manual_markers = ['(?i)\bby hand\b']
"#;

fn format() -> SpecFormat {
    let spec: FormatSpec = toml::from_str(SPEC).unwrap_or_else(|e| panic!("{e}"));
    SpecFormat::compile(spec, |_| None).unwrap_or_else(|e| panic!("{e}"))
}

fn doc(text: &str) -> Document {
    let source = source::load("t.md", text.as_bytes()).unwrap_or_else(|e| panic!("{e:?}"));
    Document::build(source).unwrap_or_else(|e| panic!("{e:?}"))
}

fn outline_of(text: &str) -> (Document, Result<Outline, Diagnostics>) {
    let d = doc(text);
    let outline = format().outline(&d);
    (d, outline)
}

fn codes(result: Result<Outline, Diagnostics>) -> Vec<&'static str> {
    match result {
        Ok(_) => Vec::new(),
        Err(d) => d.0.iter().map(|d| d.code).collect(),
    }
}

const PLAN: &str = "# Plan title

Check this by hand before starting.

## Tasks

### P3c-1-T1: first

**Goal:** one.

**Files**

- Create `src/a.rs`
- Modify `web/lock.json` (restored in step 1)

`src/gen.rs` is generated by step 2, not written by hand.

**Step 1 — write *the* files**

Text.

**Step 2 — verify**

More.

### P3c-1-T4: second

**Files**

- Delete `src/old.rs`

**Step 1 — delete**

Delete it.

## Green state

Run by hand.
";

#[test]
fn tasks_manifests_and_steps() {
    let (d, outline) = outline_of(PLAN);
    let outline = outline.unwrap_or_else(|e| panic!("{e:?}"));
    let text = |t: &TextRef| d.text(t).to_owned();
    assert_eq!(
        outline.meta.title.as_ref().map(text).as_deref(),
        Some("Plan title")
    );
    assert_eq!(outline.meta.phase.as_deref(), Some("P3c-1"));
    assert_eq!(
        outline.meta.preamble.iter().map(text).collect::<Vec<_>>(),
        ["Check this by hand before starting."]
    );
    let after = outline.after_tasks.map(|r| r.len());
    assert_eq!(after, Some(2));
    let after_span = outline
        .meta
        .after_tasks
        .map(|s| d.source.slice(s).to_owned());
    assert_eq!(
        after_span.as_deref(),
        Some("## Green state\n\nRun by hand.\n")
    );

    let ids: Vec<(String, u32, String)> = outline
        .tasks
        .iter()
        .map(|t| (t.id.raw.to_string(), t.id.ordinal, text(&t.title)))
        .collect();
    assert_eq!(
        ids,
        [
            ("P3c-1-T1".to_owned(), 1, "first".to_owned()),
            ("P3c-1-T4".to_owned(), 4, "second".to_owned())
        ]
    );

    let first = &outline.tasks[0];
    let entries: Vec<(String, ChangeKind)> = first
        .manifest
        .entries
        .iter()
        .map(|e| (e.path.to_string(), e.change))
        .collect();
    assert_eq!(
        entries,
        [
            ("src/a.rs".to_owned(), ChangeKind::Create),
            ("web/lock.json".to_owned(), ChangeKind::Modify)
        ]
    );
    let steps: Vec<(Option<u32>, Option<String>, Vec<String>)> = first
        .steps
        .iter()
        .map(|s| {
            let blocks = (s.blocks.start..s.blocks.end)
                .filter(|i| !first.manifest_blocks.is_some_and(|m| m.contains(*i)))
                .map(|i| d.source.slice(d.blocks[i].span).to_owned())
                .collect();
            (s.number, s.title.as_ref().map(text), blocks)
        })
        .collect();
    assert_eq!(
        steps,
        [
            (
                None,
                None,
                vec![
                    "**Goal:** one.".to_owned(),
                    "`src/gen.rs` is generated by step 2, not written by hand.".to_owned()
                ]
            ),
            (
                Some(1),
                Some("write *the* files".to_owned()),
                vec!["Text.".to_owned()]
            ),
            (Some(2), Some("verify".to_owned()), vec!["More.".to_owned()]),
        ]
    );
    assert_eq!(first.manifest_blocks.map(|m| m.len()), Some(3));
}

#[test]
fn a_task_level_heading_that_is_no_task_is_an_error() {
    let (_, outline) =
        outline_of("## Tasks\n\n### P1-T1: a\n\n**Files**\n\n- Create `a`\n\n### P1-T2 b\n\nx\n");
    assert_eq!(codes(outline), ["bad-task-heading"]);
}

#[test]
fn manifest_errors() {
    let missing = outline_of("## Tasks\n\n### P1-T1: a\n\n**Step 1 — x**\n\ny\n").1;
    assert_eq!(codes(missing), ["manifest-missing"]);
    let item = outline_of("## Tasks\n\n### P1-T1: a\n\n**Files**\n\n- Rename `a`\n").1;
    assert_eq!(codes(item), ["manifest-item"]);
    let empty = outline_of("## Tasks\n\n### P1-T1: a\n\n**Files**\n\nNothing.\n").1;
    assert_eq!(codes(empty), ["manifest-item"]);
    let path = outline_of("## Tasks\n\n### P1-T1: a\n\n**Files**\n\n- Create `../x`\n").1;
    assert_eq!(codes(path), ["bad-path"]);
}

#[test]
fn no_tasks_heading_or_no_task_is_an_error() {
    assert_eq!(codes(outline_of("# T\n\n### P1-T1: a\n").1), ["no-tasks"]);
    assert_eq!(
        codes(outline_of("# T\n\n## Tasks\n\nNone.\n").1),
        ["no-tasks"]
    );
}

#[test]
fn a_task_without_step_boundaries_is_one_implicit_step() {
    let (_, outline) =
        outline_of("## Tasks\n\n### P1-T1: a\n\n**Files**\n\n- Create `a`\n\nx\n\ny\n");
    let outline = outline.unwrap_or_else(|e| panic!("{e:?}"));
    let steps = &outline.tasks[0].steps;
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].number, None);
}

#[test]
fn a_task_heading_after_the_end_of_the_tasks_is_an_error() {
    let text = "## Tasks\n\n### P1-T1: a\n\n**Files**\n\n- Create `a`\n\n## Green\n\nx\n\n\
                ### P1-T2: b\n\ny\n";
    assert_eq!(codes(outline_of(text).1), ["task-after-end"]);
}

/// Tasks and the end heading both at level 2, list-item steps (the before-after shape).
const LIST_SPEC: &str = r#"
id = "l"
name = "list steps"
[detect]
patterns = [{ regex = '^## Task', weight = 60 }]
[outline]
task = { level = 2, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
tasks_end_level = 2
[outline.manifest]
marker = '^\*\*Files:\*\*$'
item = '^(?P<change>Create|Test): `(?P<paths>[^`]+)`$'
changes = { Create = "create", Test = "test" }
[[outline.steps]]
block = "list-item"
regex = '^\*\*Step (?P<n>\d+): (?P<title>.+)\*\*$'
"#;

fn list_outline(text: &str) -> Outline {
    let spec: FormatSpec = toml::from_str(LIST_SPEC).unwrap_or_else(|e| panic!("{e}"));
    let format = SpecFormat::compile(spec, |_| None).unwrap_or_else(|e| panic!("{e}"));
    format
        .outline(&doc(text))
        .unwrap_or_else(|e| panic!("{e:?}"))
}

#[test]
fn the_manifest_ends_at_a_list_item_step() {
    let outline = list_outline(
        "## Task 1: a\n\n**Files:**\n- Create: `a`\n- Test: `a`\n\n- [ ] **Step 1: write**\n\nx\n\n\
         - [ ] **Step 2: check**\n\ny\n\n## Task 2: b\n\n**Files:**\n- Create: `b`\n\nz\n\n## Self-Review\n\nw\n",
    );
    let first = &outline.tasks[0];
    assert_eq!(first.manifest.entries.len(), 2);
    assert_eq!(first.manifest_blocks.map(|m| m.len()), Some(3));
    let numbers: Vec<Option<u32>> = first.steps.iter().map(|s| s.number).collect();
    assert_eq!(numbers, [Some(1), Some(2)]);
    assert_eq!(outline.tasks.len(), 2);
    assert_eq!(outline.after_tasks.map(|r| r.len()), Some(2));
}
```

File `crates/planrun-parse/src/recognize.rs` (exact diff; save the block as `$env:TEMP\p2b-t2-recognize-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/recognize.rs b/crates/planrun-parse/src/recognize.rs
index e76f6a1..812c67a 100644
--- a/crates/planrun-parse/src/recognize.rs
+++ b/crates/planrun-parse/src/recognize.rs
@@ -101,6 +101,13 @@ fn step(
 ) -> Step {
     let mut cursor = Cursor::with_skip(doc, outline.blocks, task.manifest_blocks);
     let mut actions: Vec<Action> = Vec::new();
+    if !outline.written {
+        actions.extend(
+            task.manifest_notes
+                .iter()
+                .map(|text| manifest_note(format, doc, text)),
+        );
+    }
     while let Some(at) = cursor.peek(0).map(|b| b.span) {
         let before = cursor.consumed();
         let cx = StepContext {
@@ -140,9 +147,10 @@ fn step(
                 let manual = new
                     .iter()
                     .all(|a| matches!(a.kind, ActionKind::Manual { .. }));
-                match manual {
-                    true => cov.manual += consumed,
-                    false => cov.actions += consumed,
+                if manual {
+                    cov.manual += consumed;
+                } else {
+                    cov.actions += consumed;
                 }
                 actions.extend(new);
             }
@@ -199,6 +207,28 @@ fn expect_failure(action: &mut Action) -> Option<Diagnostic> {
     }
 }
 
+/// The recognizer id of text an inline manifest declares nothing with (decision D43).
+pub const MANIFEST: &str = "manifest";
+
+/// A leftover of an inline manifest: a Note, or `Manual(Marked)` when it matches a manual
+/// marker. Its block is the manifest's, so coverage counts it there, once.
+fn manifest_note(format: &dyn Format, doc: &Document, text: &TextRef) -> Action {
+    let kind = if format.settings().is_marked(doc.text(text)) {
+        ActionKind::Manual {
+            reason: ManualReason::Marked,
+            text: text.clone(),
+        }
+    } else {
+        ActionKind::Note { text: text.clone() }
+    };
+    Action {
+        kind,
+        span: text.span(),
+        origin: RecognizerId(MANIFEST.into()),
+        path_origin: Default::default(),
+    }
+}
+
 /// The first recognizer that answers anything but `No`, with its id.
 fn first_match<'f>(
     format: &'f dyn Format,
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t2-recognize-rs.patch"
Remove-Item "$env:TEMP\p2b-t2-recognize-rs.patch"
```

File `crates/planrun-parse/src/spec.rs` (exact diff; save the block as `$env:TEMP\p2b-t2-spec-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/spec.rs b/crates/planrun-parse/src/spec.rs
index 3b80b92..e4e2d3d 100644
--- a/crates/planrun-parse/src/spec.rs
+++ b/crates/planrun-parse/src/spec.rs
@@ -13,7 +13,8 @@ use serde::Deserialize;
 use crate::diagnostic::Diagnostics;
 use crate::document::Document;
 use crate::format::{CheckLevel, DOWNGRADABLE, Format, FormatSettings, Outline, StepMark};
-use crate::outline::{self, HeadingRule, ManifestRules, OutlineRules, StepBlock, StepRule};
+use crate::manifest::ManifestRules;
+use crate::outline::{self, HeadingRule, OutlineRules, StepBlock, StepRule};
 use crate::recognizer::{Recognizer, RecognizerFactory};
 
 /// A format spec as read from TOML. The schema is documented in docs/extending.md §2; keep the
@@ -110,6 +111,12 @@ pub struct ManifestSpec {
     /// A bare file name after a path with a directory is in that directory; default false
     /// (decision D32).
     pub siblings: Option<bool>,
+    /// Splits the marker's `items` capture into clauses (decision D43); needs a marker that
+    /// captures `items`.
+    pub clause_list: Option<String>,
+    /// Over the text right after each path token; its `change` capture, mapped through
+    /// `changes`, overrides the change of that one path (decision D44).
+    pub path_change: Option<String>,
 }
 
 /// A `[[step_marks]]` row.
@@ -293,6 +300,10 @@ fn compile_manifest(spec: &ManifestSpec) -> Result<ManifestRules, String> {
         .collect::<Result<_, String>>()?;
     let marker = compile_regex("outline.manifest.marker", &spec.marker, &[])?;
     let marker_change = marker.capture_names().flatten().any(|n| n == "change");
+    let marker_items = marker.capture_names().flatten().any(|n| n == "items");
+    if spec.clause_list.is_some() && !marker_items {
+        return Err("outline.manifest.clause_list: the marker has no `items` capture".to_owned());
+    }
     let item_needs: &[&str] = if marker_change {
         &["paths"]
     } else {
@@ -307,6 +318,16 @@ fn compile_manifest(spec: &ManifestSpec) -> Result<ManifestRules, String> {
             .as_deref()
             .map(|p| compile_regex("outline.manifest.path_list", p, &[]))
             .transpose()?,
+        clause_list: spec
+            .clause_list
+            .as_deref()
+            .map(|p| compile_regex("outline.manifest.clause_list", p, &[]))
+            .transpose()?,
+        path_change: spec
+            .path_change
+            .as_deref()
+            .map(|p| compile_regex("outline.manifest.path_change", p, &["change"]))
+            .transpose()?,
         required: spec.required.unwrap_or(true),
         siblings: spec.siblings.unwrap_or(false),
     })
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t2-spec-rs.patch"
Remove-Item "$env:TEMP\p2b-t2-spec-rs.patch"
```

File `crates/planrun-parse/tests/manifest_inline.rs` (full contents):

```rust
//! Inline Files clauses (decision D43), a change per path (D44) and bare file names without
//! siblings (D45).

use planrun_model::{ActionKind, ChangeKind, PathOrigin};
use planrun_parse::{Registry, parse_plan};

const SPEC: &str = r#"
id = "t"
name = "inline manifest test"
[detect]
patterns = [{ regex = '^### Task', weight = 60 }]
[outline]
task = { level = 3, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
[outline.manifest]
marker = '^\*\*Files:\*\*(?:$| (?P<items>(?:[^`.\n]|`[^`]*`)+))'
item = '^(?P<change>Create|Modify|create|modify)(?: \([^)]*\))?:? `(?P<paths>[^`]+(?:`(?: \([^)]*\))?(?:, | and )`[^`]+)*)`(?: \(.+\))?$'
changes = { Create = "create", Modify = "modify", create = "create", modify = "modify", "full replacement" = "rewrite" }
path_list = '`(?: \([^)]*\))?(?:, | and )`'
clause_list = '; '
path_change = '^` \((?P<change>full replacement)\)'
required = false
[settings]
manual_markers = ['(?i)\bby hand\b']
[checks]
manifest-untouched = "off"
manifest-undeclared = "warning"
[[recognizer]]
kind = "file-block"
id = "file-block#t"
lead = '^`(?P<path>[^`]+)`(?: \((?P<op>replaces the whole file)\))?:$'
ops = { "replaces the whole file" = "overwrite" }
default_op = "create"
[[recognizer]]
kind = "before-after"
id = "before-after#t"
lead = '^`(?P<path>[^`]+)` edit:$'
[[recognizer]]
kind = "before-after"
id = "before-after#no-path"
lead = '^Edit:$'
infer_path = true
"#;

fn parse(text: &str) -> Result<planrun_parse::Parsed, Vec<&'static str>> {
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    registry
        .add_spec("t", SPEC)
        .unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, "t.md", text.as_bytes(), Some("t")).map_err(|d| d.error_codes())
}

fn ok(text: &str) -> planrun_parse::Parsed {
    parse(text).unwrap_or_else(|codes| panic!("{codes:?}"))
}

/// (path, change, origin) per entry of the first task.
fn entries(parsed: &planrun_parse::Parsed) -> Vec<(String, ChangeKind, PathOrigin)> {
    parsed.plan.tasks[0]
        .manifest
        .entries
        .iter()
        .map(|e| (e.path.to_string(), e.change, e.origin))
        .collect()
}

/// The actions of the first task, as the text of each Note and the kind of everything else.
fn actions(parsed: &planrun_parse::Parsed) -> Vec<String> {
    let plan = &parsed.plan;
    plan.tasks[0]
        .steps
        .iter()
        .flat_map(|s| s.actions.iter())
        .map(|a| match &a.kind {
            ActionKind::Note { text } => format!("note {}", plan.text(text)),
            ActionKind::Manual { reason, text } => format!("manual {reason:?} {}", plan.text(text)),
            ActionKind::WriteFile { path, mode, .. } => format!("write {path} {mode:?}"),
            ActionKind::ReplaceExact { path, .. } => format!("replace {path}"),
            other => format!("{other:?}"),
        })
        .collect()
}

fn fence(body: &str) -> String {
    format!("```text\n{body}\n```\n\n")
}

#[test]
fn clauses_split_on_the_clause_list() {
    let parsed = ok("### Task 1: a\n\n**Files:** create `a/x.ps1`; modify `a/y.ps1`.\n");
    let expected = [
        (
            "a/x.ps1".to_owned(),
            ChangeKind::Create,
            PathOrigin::Written,
        ),
        (
            "a/y.ps1".to_owned(),
            ChangeKind::Modify,
            PathOrigin::Written,
        ),
    ];
    assert_eq!(entries(&parsed), expected);
    assert!(actions(&parsed).is_empty(), "{:?}", actions(&parsed));
}

#[test]
fn text_that_declares_nothing_is_a_note_at_the_start_of_the_task() {
    let text = "### Task 1: a\n\n**Files:** modify `a/x.ps1`; regenerate the README examples.\n\
                Then run it by hand.\n\nMore prose.\n";
    let parsed = ok(text);
    assert_eq!(entries(&parsed).len(), 1);
    let expected = [
        "note regenerate the README examples",
        "manual Marked Then run it by hand.",
        "note More prose.",
    ];
    assert_eq!(actions(&parsed), expected);
    let coverage = &parsed.coverage[0];
    assert_eq!((coverage.manifest, coverage.notes), (1, 1));
}

#[test]
fn a_task_with_only_an_inline_manifest_still_shows_its_leftover() {
    let parsed = ok("### Task 1: a\n\n**Files:** create `a/x.ps1`. It generates `a/y.json`.\n");
    assert_eq!(actions(&parsed), ["note It generates `a/y.json`."]);
}

#[test]
fn an_inline_manifest_takes_no_list() {
    let text = "### Task 1: a\n\n**Files:** create `a/x.ps1`\n- `a/y.ps1`\n";
    assert_eq!(parse(text).err(), Some(vec!["manifest-item"]));
}

#[test]
fn a_clause_with_a_code_span_must_be_an_item() {
    let text = "### Task 1: a\n\n**Files:** create `a/x.ps1`; rename `a/y.ps1`.\n";
    assert_eq!(parse(text).err(), Some(vec!["manifest-item"]));
}

#[test]
fn a_note_after_one_path_changes_only_that_path() {
    let text = format!(
        "### Task 1: a\n\n**Files:**\n\
         - Create (Write tool): `lib/base.ps1` (full replacement), `lib/one.ps1`, `lib/two.ps1`\n\n\
         `lib/base.ps1` (replaces the whole file):\n\n{}`lib/one.ps1`:\n\n{}",
        fence("a"),
        fence("b")
    );
    let parsed = ok(&text);
    let changes: Vec<ChangeKind> = entries(&parsed).into_iter().map(|e| e.1).collect();
    assert_eq!(
        changes,
        [ChangeKind::Rewrite, ChangeKind::Create, ChangeKind::Create]
    );
    let writes = ["write lib/base.ps1 Overwrite", "write lib/one.ps1 Create"];
    assert_eq!(actions(&parsed), writes);
    assert!(parsed.diagnostics.0.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn without_siblings_a_bare_name_after_a_directory_is_bare() {
    let parsed = ok("### Task 1: a\n\n**Files:** modify `src/x/a.rs` and `README.md`\n");
    let origins: Vec<PathOrigin> = entries(&parsed).into_iter().map(|e| e.2).collect();
    assert_eq!(origins, [PathOrigin::Written, PathOrigin::Bare]);
    let first = ok("### Task 1: a\n\n**Files:** modify `README.md` and `src/x/a.rs`\n");
    assert_eq!(entries(&first)[0].2, PathOrigin::Written);
}

#[test]
fn a_create_on_a_bare_entry_is_manual() {
    let text = format!(
        "### Task 1: a\n\n**Files:** create `src/x/a.rs`, `notes.md`\n\n`notes.md`:\n\n{}",
        fence("n")
    );
    let found = actions(&ok(&text));
    assert_eq!(found.len(), 1);
    assert!(found[0].starts_with("manual AmbiguousPath"), "{found:?}");
}

#[test]
fn an_edit_of_a_bare_entry_stays_an_action() {
    let text = format!(
        "### Task 1: a\n\n**Files:** modify `src/x/a.rs` and `README.md`\n\n`README.md` edit:\n\n{}{}",
        fence("old"),
        fence("new")
    );
    assert_eq!(actions(&ok(&text)), ["replace README.md"]);
}

#[test]
fn a_bare_entry_makes_an_edit_without_a_path_ambiguous() {
    let text = format!(
        "### Task 1: a\n\n**Files:** modify `src/x/a.rs` and `README.md`\n\nEdit:\n\n{}{}",
        fence("old"),
        fence("new")
    );
    let found = actions(&ok(&text));
    assert!(found[0].starts_with("manual ProseEdit"), "{found:?}");
    let only_bare = format!(
        "### Task 1: a\n\n**Files:** create `src/x/a.rs`; modify `x/b.rs`, `c.rs`\n\nEdit:\n\n{}{}",
        fence("old"),
        fence("new")
    );
    let found = actions(&ok(&only_bare));
    assert!(found[0].starts_with("manual ProseEdit"), "{found:?}");
}

#[test]
fn the_mode_check_still_sees_the_path_change() {
    let text = format!(
        "### Task 1: a\n\n**Files:**\n- Create: `h/a.ps1`, `h/b.ps1` (full replacement)\n\n\
         `h/b.ps1`:\n\n{}",
        fence("b")
    );
    assert_eq!(parse(&text).err(), Some(vec!["manifest-write-mode"]));
}
```

File `crates/planrun-parse/tests/manifest_schema.rs` (full contents):

```rust
//! The manifest keys of a format spec (docs/extending.md §2, decisions D26, D27, D32): an
//! optional manifest, a marker that captures the change, sibling file names, `:lines` tokens,
//! `Generated` entries, and a write whose op contradicts the manifest.

use planrun_model::{ChangeKind, PathOrigin};
use planrun_parse::{Registry, parse_plan};

const SPEC: &str = r#"
id = "t"
name = "manifest schema test"
[detect]
patterns = [{ regex = '^### Task', weight = 60 }]
[outline]
task = { level = 3, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
[outline.manifest]
marker = '^\*\*Files:\*\*(?: (?P<change>create|modify))?$'
item = '^(?:(?P<change>Create|Modify|Generated): )?`(?P<paths>[^`]+(?:`(?:, | and )`[^`]+)*)`$'
changes = { Create = "create", Modify = "modify", Generated = "generated", create = "create", modify = "modify" }
path_list = '`(?:, | and )`'
required = false
siblings = true
[checks]
manifest-untouched = "off"
manifest-undeclared = "warning"
[[recognizer]]
kind = "file-block"
id = "file-block#t"
lead = '^Create `(?P<path>[^`]+)`:$'
ops = {}
default_op = "create"
"#;

fn registry(spec: &str) -> Registry {
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    registry
        .add_spec("t", spec)
        .unwrap_or_else(|e| panic!("{e}"));
    registry
}

fn parse(text: &str) -> Result<planrun_parse::Parsed, Vec<&'static str>> {
    parse_plan(&registry(SPEC), "t.md", text.as_bytes(), Some("t")).map_err(|d| d.error_codes())
}

/// One manifest entry: path, change, lines, origin.
type Entry = (String, ChangeKind, Option<(u32, u32)>, PathOrigin);

/// The first task's entries.
fn entries(text: &str) -> Vec<Entry> {
    let parsed = parse(text).unwrap_or_else(|codes| panic!("{codes:?}"));
    parsed.plan.tasks[0]
        .manifest
        .entries
        .iter()
        .map(|e| {
            let lines = e.lines.map(|l| (l.start, l.end));
            (e.path.to_string(), e.change, lines, e.origin)
        })
        .collect()
}

#[test]
fn a_task_may_have_no_manifest() {
    let parsed = parse("### Task 0: baseline\n\nNothing to write.\n")
        .unwrap_or_else(|codes| panic!("{codes:?}"));
    assert!(parsed.plan.tasks[0].manifest.span.is_none());
}

#[test]
fn the_marker_can_carry_the_change() {
    let text = "### Task 1: a\n\n**Files:** create\n- `src/a.rs`\n- `src/b.rs`\n\n\
                Create `src/a.rs`:\n\n```rust\na\n```\n\nCreate `src/b.rs`:\n\n```rust\nb\n```\n";
    let changes: Vec<ChangeKind> = entries(text).into_iter().map(|e| e.1).collect();
    assert_eq!(changes, [ChangeKind::Create, ChangeKind::Create]);
}

#[test]
fn siblings_and_line_tokens_follow_the_path_before_them() {
    let text = "### Task 1: a\n\n**Files:**\n\
                - Modify: `src/x/a.rs:3-4`, `:9` and `b.rs`\n\
                - Generated: `docs/out.txt`\n";
    assert_eq!(
        entries(text),
        [
            (
                "src/x/a.rs".to_owned(),
                ChangeKind::Modify,
                Some((3, 4)),
                PathOrigin::Written
            ),
            (
                "src/x/a.rs".to_owned(),
                ChangeKind::Modify,
                Some((9, 9)),
                PathOrigin::Written
            ),
            (
                "src/x/b.rs".to_owned(),
                ChangeKind::Modify,
                None,
                PathOrigin::Sibling
            ),
            (
                "docs/out.txt".to_owned(),
                ChangeKind::Generated,
                None,
                PathOrigin::Written
            ),
        ]
    );
}

#[test]
fn without_siblings_a_bare_name_is_bare() {
    let spec = SPEC.replace("siblings = true", "siblings = false");
    let text = "### Task 1: a\n\n**Files:**\n- Modify: `src/x/a.rs`, `Cargo.toml`\n";
    let parsed = parse_plan(&registry(&spec), "t.md", text.as_bytes(), Some("t"))
        .unwrap_or_else(|d| panic!("{:?}", d.error_codes()));
    let paths: Vec<String> = parsed.plan.tasks[0]
        .manifest
        .entries
        .iter()
        .map(|e| format!("{} {:?}", e.path, e.origin))
        .collect();
    assert_eq!(paths, ["src/x/a.rs Written", "Cargo.toml Bare"]);
}

#[test]
fn a_create_on_a_declared_modify_is_an_error() {
    let text = "### Task 1: a\n\n**Files:** modify\n- `src/a.rs`\n\n\
                Create `src/a.rs`:\n\n```rust\na\n```\n";
    assert_eq!(parse(text).err(), Some(vec!["manifest-write-mode"]));
}

#[test]
fn an_item_needs_a_change_from_itself_or_the_marker() {
    let text = "### Task 1: a\n\n**Files:**\n- `src/a.rs`\n";
    assert_eq!(parse(text).err(), Some(vec!["manifest-item"]));
    let no_change = SPEC.replace("(?: (?P<change>create|modify))?", "");
    let no_change = no_change.replace("(?:(?P<change>Create|Modify|Generated): )?", "");
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let refused = registry.add_spec("t", &no_change);
    assert!(
        refused.is_err_and(|e| e.to_string().contains("no `change` capture")),
        "a spec with no change capture loads"
    );
}

#[test]
fn a_spec_downgrades_only_the_manifest_coverage_checks() {
    let text = "### Task 1: a\n\n**Files:** create\n- `src/a.rs`\n\n\
                Create `src/b.rs`:\n\n```rust\nb\n```\n";
    let parsed = parse(text).unwrap_or_else(|codes| panic!("{codes:?}"));
    let warnings: Vec<&str> = parsed.diagnostics.0.iter().map(|d| d.code).collect();
    assert_eq!(warnings, ["manifest-undeclared"]);

    for bad in [
        "manifest-write-mode = \"off\"",
        "blob-chain = \"warning\"",
        "manifest-untouched = \"ignore\"",
    ] {
        let spec = SPEC.replace("manifest-untouched = \"off\"", bad);
        let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
        assert!(registry.add_spec("t", &spec).is_err(), "{bad} loads");
    }
}
```

File `docs/architecture.md` (exact diff; save the block as `$env:TEMP\p2b-t2-architecture-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/architecture.md b/docs/architecture.md
index bb8cf1e..b68869b 100644
--- a/docs/architecture.md
+++ b/docs/architecture.md
@@ -144,7 +144,10 @@ outline: they fall in no step, and coverage counts them as "manifest"; its items
 step boundary (D37). A format with a manifest pattern requires one per task (`manifest-missing`)
 unless it sets `required = false` (D27); an item that does not match is `manifest-item`. Paths in
 items may be completed from the item itself (`:lines` tokens, and sibling file names where the
-format opts in, D32). Blocks between a task heading and its first step form an unnumbered **intro
+format opts in, D32; bare file names otherwise, D45) and give one path its own change (D44).
+A manifest may also be written inline in its marker paragraph (D43); what such a paragraph
+declares nothing with is shown as a Note at the start of the intro step. Ranges and globs are
+pattern entries (D41). Blocks between a task heading and its first step form an unnumbered **intro
 step** (it can hold a `Generated` notice or `Manual` text). A heading at the task level inside the
 tasks that is not a task is `bad-task-heading`. A task with no step boundaries is one implicit
 step. A step title can carry a **mark** (`[[step_marks]]`, D30): expected failure or human check.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t2-architecture-md.patch"
Remove-Item "$env:TEMP\p2b-t2-architecture-md.patch"
```

File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p2b-t2-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index ca56565..85e8f86 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -313,3 +313,48 @@ was chosen instead, so "no write targets a wildcard" is a property of the types.
 
 Amends D7: a `RepoPath` also refuses `*` and `?`. Both are invalid in Windows file names
 (D19), and a path with one is a pattern (D41). A lead that names such a path is `bad-path`.
+
+## D43 Inline Files clauses
+
+A manifest may be written inside its marker paragraph: ``**Files:** create `a`; modify `b`.``
+
+- The marker regex captures `items`. When that capture takes part, the paragraph holds the
+  manifest and no list items are taken after it; a list at the marker's depth right after it is
+  `manifest-item`. When the marker matches without `items`, list items are taken as before.
+- `clause_list` splits `items` into clauses; each clause is matched by the same `item` regex as
+  a list item. A clause with a code span that does not match is `manifest-item`.
+- Text that declares nothing (a clause without a code span, such as "regenerate the README
+  examples", and the text after the `items` capture) is not dropped: it is shown in place at the
+  start of the task's intro step, as a `Note`, or as `Manual(Marked)` when it matches a manual
+  marker. Coverage counts the marker paragraph once, as manifest. Each phase plan lists these
+  leftovers with its imperative Notes (D36).
+
+## D44 A change per path
+
+`[outline.manifest] path_change` is a regex over the text right after a path token (the
+separator up to the next token, or the rest of the item after the last one). Its `change`
+capture, mapped through `changes`, overrides the change of that one path. The values are
+therefore `ChangeKind`s and D26 stays strict: a wrong mapping is a loud `manifest-write-mode`,
+or a preflight error in P3. Precedence: path > item > marker. A note binds to the path directly
+before it; in the real plans no multi-path item has a note meant for the whole item.
+
+Handover P2a→P2b open item 2 proposed mapping the "(full replacement)" note at item level. In
+the real plan that note follows the first of six paths in one item; at item level the five new
+files would become Rewrite entries and their create leads `manifest-write-mode` errors. The
+item-level mapping was dropped for this rule.
+
+## D45 Bare file names without siblings
+
+Amends D32. With `siblings` off, a token without `/` that follows a token with a directory in
+the same item (or clause) gets `PathOrigin::Bare`: its path stays as written, and its location
+is unknown.
+
+- `file_name_from_manifest` and `infer_path` never resolve to a Bare entry. For `infer_path`, a
+  Bare entry still counts as one of the task's editable entries: dropping it would turn an
+  ambiguous task into a confident guess.
+- A Create-mode write (op `create`, or `write` resolved to Create) whose path equals a Bare
+  entry is `Manual(AmbiguousPath)`, not a write. A Modify or Rewrite on a Bare entry stays an
+  action: P3 preflight catches a wrong target.
+- D33: a Bare entry is not literal, so the commit suggestion shows the message only.
+- `show` marks Bare entries. `AmbiguousPath` is produced only by this rule; it is not a
+  `manual-prose` reason.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t2-decisions-md.patch"
Remove-Item "$env:TEMP\p2b-t2-decisions-md.patch"
```

File `docs/extending.md` (full contents):

````markdown
# Extending planrun

New plan styles will keep appearing. They are added without touching execution, in three levels
(architecture.md §5). Use the lowest level that works.

## 1. Decide the level

1. Describe the style in a table like the ones in [plan-grammar.md](plan-grammar.md), from at
   least two real plans.
2. For each row, find the built-in recognizer kind that covers it (§3). If every row is covered,
   write a **format spec** (§2). Done, no rebuild.
3. If a row is a new idiom, write a **recognizer kind** (§5) and use it from a spec.
4. Only if the plan's *structure* is not headings-with-steps (tasks in a table, a JSON plan),
   write a **Format** (§6).
5. A new **action kind** (§4) is almost never the answer: map the wording onto an existing action.

## 2. Format specs (TOML)

Location: built-ins in `formats/<id>.toml` (embedded); per repository in
`<repo>/.planrun/formats/<id>.toml`; per user in the config directory. The file name is the id.

```toml
id = "assay"                      # unique; the file name
name = "Assay plan (planbuild.py)"

[detect]                          # a pattern that matches a line outside fenced blocks adds
patterns = [ { regex = '^### P\d+-T\d+: ', weight = 60 } ]   # its weight once; capped at 100

[outline]
tasks_after = { level = 2, regex = '^Tasks$' }          # optional
task = { level = 3, regex = '^(?P<id>(?P<phase>P\d+)-T(?P<ord>\d+)): (?P<title>.+)$' }
tasks_end_level = 2                                      # optional
# captures: id, ord (a number), title (all required); phase (optional, from the first task)

[outline.manifest]                                       # optional
# The marker and its items are consumed by the outline: they fall in no step. The items end at
# the first block that is no list item at the marker's depth, or that is a step boundary (D37).
marker = '^\*\*Files\*\*$'                                # may capture `change` (`**Files:** create`)
# and `items`: then the marker paragraph holds the manifest, split by `clause_list`, each
# clause matched by `item`; text that declares nothing is shown as a Note (D43)
item = '^(?P<change>Create|Modify|Delete) `(?P<paths>[^`]+)`$'   # `change` unless the marker has it
changes = { Create = "create", Modify = "modify", Delete = "delete" }
# → create | modify | rewrite | delete | test | generated (D27)
path_list = ', '                                         # optional: split `paths` into several;
# a `:3-10` piece adds lines to the path before it (D32). Built in, before the split, in every
# format: `a`–`b` (an en dash between code spans) is one range entry, so `path_list` must not
# list the en dash; a piece with `*`, `?` or `{` is a glob entry (D41). Pattern entries declare
# the files they match and are never a write target.
required = true                                          # optional, default true: false lets a
                                                         # task have no manifest (D27)
siblings = false                                         # optional, default false: a bare file
                                                         # name after `a/b.rs` is `a/<name>` (D32);
                                                         # without siblings it is Bare (D45)
clause_list = '; '                                       # optional: needs an `items` capture (D43)
path_change = '^` \((?P<change>full replacement)\)'     # optional (D44): over the text right
# after a path token; `change` goes through `changes` and wins over the item's change, which
# wins over the marker's

[[outline.steps]]                                        # zero or more; none = one implicit step
block = "paragraph"                                      # paragraph | list-item | heading
regex = '^\*\*Step (?P<n>\d+) — (?P<title>[^*]+)\*\*$'   # n and title optional

[settings]
manual_markers = ['(?i)\bby hand\b']                     # paragraph → Manual(Marked)
pwsh = ["powershell", "pwsh", "ps1"]                     # optional; these are the defaults
bash = ["bash", "sh", "shell"]

[[step_marks]]                                           # zero or more (D30), first match wins
title = '(?i)\bverify (?:it|they) fails?\b'              # regex over the step's title
mark = "expected-failure"                                # expected-failure | human-check
# expected-failure: the step's Verify scripts expect failure (none or `any` becomes Failure; an
# expected success is the error expected-conflict). human-check: its prose Notes become
# Manual(HumanCheck).

[checks]                                                 # optional (D31): only these two codes,
manifest-untouched = "off"                               # each error | warning | off; any
manifest-undeclared = "warning"                          # other key fails the spec load

[[recognizer]]                                           # ordered; first match wins
kind = "file-block"                                      # a registered kind (§3)
id = "file-block#assay"                                  # shown by `show` and coverage
# … the kind's own parameters
```

Rules:

- Unknown keys are errors (`deny_unknown_fields`). A spec is fully validated when the registry
  loads it (unknown kind, bad regex, bad parameter), before any plan is read.
- Regexes use the `regex` crate syntax; `(?P<name>…)` captures. A regex that lacks a capture
  its reader requires (`path`, `id`, …) is refused when the spec loads.
- `crates/planrun-parse/tests/spec_files.rs` checks every built-in spec against the schema.

## 3. Built-in recognizer kinds

Parameters in *italics* are optional.

| Kind | Matches | Produces | Parameters |
|---|---|---|---|
| `file-block` | a lead paragraph naming a path (and op), then one fence | `WriteFile` / `ApplyPatch` | `lead` (captures `path`, *`op`*), `ops` (op prefix → `write` \| `create` \| `overwrite` \| `patch`), *`default_op`*, *`file_name_from_manifest`* |
| `before-after` | lead (naming a path), fence, *separator paragraph*, fence | `ReplaceExact`, or `Manual(ProseEdit)` when `infer_path` finds no single entry | `lead` (captures `path`, or none with `infer_path = true`), *`separator`* (e.g. `^After:$`; absent = two fences back to back), *`file_name_from_manifest`* |
| `delete` | a sentence or inline `git rm` | `Delete` | `lead` (captures `path`), *`via_git`*, *`file_name_from_manifest`* |
| `generated` | a "generated, not by hand" sentence | `Generated` | `lead` (captures `path`) |
| `ceremony` | a fence directly after a given action whose body matches | nothing (absorbed) | `after` (`patch`), `body` |
| `script` | a fence in a shell language | `RunScript` / `Commit` | *`intents`* (`[{ intent, body?, lead? }]`: `commit` \| `gate` \| `preview` \| `apply` \| `verify` \| `other`), *`leads`* (regexes), *`first_block`* |
| `run-line` | `` Run: `cmd` `` in a paragraph, with an optional expectation in the same paragraph | `RunScript` / `Commit` (one line) | `lead` (captures `cmd`, *`expected`*), `shell` (`pwsh` \| `bash`), *`exit`* table over `expected` (field `text`), *`intents`* (`[{ intent, body }]`) |
| `expected-block` | `Expected (…):` paragraph + `text` fence after a script | amends the script | `lead` (captures `spec`), `exit` table, `lines = "summary"` |
| `expected-prose` | `Expected: …` sentence after a script | amends the script | `lead` (captures `text`), `exit` table (`success` \| `failure` \| `any` \| `code:N`) |
| `manual-prose` | a paragraph (+ the fence after it, with `takes_fence`) that describes what planrun cannot do exactly | `Manual(reason)` | `lead`, `reason` (`prose-edit` \| `output-dependent` \| `human-check`), *`takes_fence`* |

Notes on the kinds:

- `file-block`: the longest `ops` prefix of the captured `op` wins; `write` takes Create or
  Overwrite from the manifest. Once the lead matched, an op no prefix covers is
  `Malformed(unknown-file-op)`, a lead without its fence `Malformed(missing-block)`, and for
  `patch` a patch whose paths differ from the lead's `Malformed(patch-path)`.
- Paths from the manifest (D32): `file_name_from_manifest = true` resolves a lead path without
  `/` that is not an entry itself to the one entry with that file name, when that entry was
  written in full (not a sibling, not Bare); otherwise the path stays as written. `before-after`
  with `infer_path = true` takes the task's one entry that is not Test, Generated or Delete; with
  none or several (a Bare entry counts, but is never taken, D45), the lead and both fences
  become one `Manual(ProseEdit)`. Pattern entries are never candidates (D41). `infer_path` is never
  available for writes. Each resolved action carries its `PathOrigin`; `show` marks it.
- `run-line`: the command is the `cmd` capture, run as one line in `shell` (D39). The `expected`
  capture, when the lead has one and it matched, becomes a prose expectation through the `exit`
  table (no row matches: `any`). An empty command is `Malformed(missing-block)`.
- `before-after`: once the lead matched, a missing fence or separator is
  `Malformed(missing-block)`.
- `script`: `intent = "commit"` selects the `Commit` action kind, not a `ScriptIntent`; a commit
  script carries `ScriptIntent::Other`. The other intents set the `ScriptIntent`; no match is
  `verify`. A row matches the fence body (`body`), the paragraph directly before the fence
  (`lead`), or both; at least one is required (D29). Preview/apply pairs are rows like these.
- `script` guard (D28): with `leads` set, the kind claims a fence only when the paragraph
  directly before it matches one of `leads`, or, with `first_block = true`, when the fence is the
  first block of a written step (never the implicit first step, so never right after the
  manifest). A body intent never qualifies a fence on its own. Any other shell fence falls to the
  fallback, `Manual(Unrecognized)`. Without `leads` (the assay spec) every shell fence is a script.
- `expected-block` and `expected-prose` amend the script directly before them. Anything else there
  is `Malformed(expected-without-script)`; a second expectation on one script is
  `Malformed(expected-twice)`. `expected-block` needs a `text` fence (`missing-block`). An
  `expected-prose` sentence directly after a `Manual` joins that Manual's text instead (D38).
- Coverage counts the blocks of a recognizer's `Manual` (and of an expectation joined to a
  Manual) as manual, not as actions.
- **Exit tables are first match**: rows are tried in order and the first whose regex matches
  decides, so their order matters. Values: `success`, `failure`, `any` (`ExitExpect::Unknown`: not
  checked; prose is still confirmed, D11), `code:N`, and `code` (the row's regex captures `code`;
  0 is `success`). An `expected-block` spec no row matches is `Malformed(expected-spec)`; an
  `expected-prose` text no row matches is `any`.

The **fallback** (not configurable) runs when nothing matched: paragraph, list item, blockquote →
`Note` or `Manual(Marked)`; fence → `Manual(Unrecognized)` / `Manual(ForeignLanguage)`, a `text`
fence → `Note`; table, heading, rule → `Note`; HTML comment → dropped.

## 4. Adding an action kind

Rare (D4). The checklist, all in one change:

1. The variant in `planrun-model::ActionKind`, with docs.
2. Its handler in planrun-exec, its `DoneState` logic for resume, its dry-run behaviour.
3. Its policy class.
4. Its journal record.
5. Its `show` rendering and coverage label.
6. A corpus plan that produces it, and an invalid-corpus file for each new error.

## 5. Adding a recognizer kind

```rust
pub struct InsertAfterFactory;

impl RecognizerFactory for InsertAfterFactory {
    fn kind(&self) -> &'static str { "insert-after" }

    fn build(&self, params: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        // Read and validate every parameter here: a bad regex fails when the spec loads.
        Ok(Arc::new(InsertAfter { lead: regex(params, "lead")?, id: id(params)? }))
    }
}

struct InsertAfter { lead: Regex, id: String }

impl Recognizer for InsertAfter {
    fn id(&self) -> &str { &self.id }

    fn recognize(&self, cx: &StepContext<'_>, cursor: &mut Cursor<'_>) -> Recognized {
        // 1. Peek; return Recognized::No without advancing unless the lead matches.
        // 2. Once the lead matched, a missing or wrong block is Recognized::Malformed, not No.
        // 3. Advance past exactly what was consumed; return the actions.
        Recognized::No
    }
}
```

Register it in `Registry::with_builtins`. Rules:

- **Pure.** No I/O, no clock, no state between calls. Same input, same output.
- **`No` leaves the cursor where it was.** Advance only on `Yes`/`Amend`.
- **Commit to the idiom early.** Once the distinctive lead matched, failures are `Malformed`
  errors. Falling through to `Manual` would hide a broken plan.
- **Tests**: unit tests on hand-built blocks, plus a corpus plan in `tests/corpus/synthetic` and
  an invalid-corpus file per `Malformed` code.

## 6. Adding a Format by hand

Implement `Format` (`detect`, `outline`, `recognizers`, `settings`) and register it with
`Registry::add_format`. Reuse recognizer instances from the factories for the step contents: only
the outline should be new. Every block of every task must fall in exactly one step range.

## 7. Execution-side extension points

| Trait | Registry | For |
|---|---|---|
| `ActionHandler` | `HandlerSet` | how an action kind executes, previews, and reports `DoneState` |
| `ShellSession` | one per `Shell` | another shell (`cmd`, `zsh`): a new `Shell` variant + session |
| `CommandClassifier` | the runner's | different policy classification |
| `Ui` | the CLI | a TUI (ratatui) later, a non-interactive batch UI |
````

**Step 2 — verify**

Targeted: the inline-manifest tests:

```powershell
cargo test -p planrun-parse --test manifest_inline
```

Expected (exit code 0; summary lines):

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- CLAUDE.md crates/planrun-cli/src/view.rs crates/planrun-model/src/action.rs crates/planrun-model/src/path.rs crates/planrun-parse/src/format.rs crates/planrun-parse/src/kinds/file_block.rs crates/planrun-parse/src/kinds/mod.rs crates/planrun-parse/src/lib.rs crates/planrun-parse/src/manifest.rs crates/planrun-parse/src/outline.rs crates/planrun-parse/src/outline_tests.rs crates/planrun-parse/src/recognize.rs crates/planrun-parse/src/spec.rs crates/planrun-parse/tests/manifest_inline.rs crates/planrun-parse/tests/manifest_schema.rs docs/architecture.md docs/decisions.md docs/extending.md
  $verified = [ordered]@{
      'CLAUDE.md' = '4b5b2ef3ceb2fc50b4e93be2c490508536019e2b'
      'crates/planrun-cli/src/view.rs' = 'e2571fab21b51abd1deca18dab967d0ef3da5efd'
      'crates/planrun-model/src/action.rs' = '57610abfd042f25b1e023fe2a20f450ad7aaa2a6'
      'crates/planrun-model/src/path.rs' = 'f057d8af344440c571ce203916c7115d1b134f16'
      'crates/planrun-parse/src/format.rs' = 'f9cb474501543e14c3e4a0f8673445b7255970cc'
      'crates/planrun-parse/src/kinds/file_block.rs' = '2b13b68c1dbbbfd9056c8d5b9c6ce0ed8df9eccb'
      'crates/planrun-parse/src/kinds/mod.rs' = 'e776ba4b2d5a2a5f691020d13778e58723758d4a'
      'crates/planrun-parse/src/lib.rs' = '6fe53134875bbad8aaf81be5c89556efb19a8c7f'
      'crates/planrun-parse/src/manifest.rs' = '86ae960d81799dd61313fc369eb1abf4b97fb6a8'
      'crates/planrun-parse/src/outline.rs' = 'e1a2b232241a4d070c5fb60ad969c392245af9eb'
      'crates/planrun-parse/src/outline_tests.rs' = 'f4755665cb541412a7afe082416b866ae5cd1886'
      'crates/planrun-parse/src/recognize.rs' = '812c67a82101cedfe4612db8c9bc77abb58c0130'
      'crates/planrun-parse/src/spec.rs' = 'e4e2d3db208b4c6fee719bfb9c04a7b03213cb73'
      'crates/planrun-parse/tests/manifest_inline.rs' = 'ef79be960b470e5bf2cbf7be07f36301ff237579'
      'crates/planrun-parse/tests/manifest_schema.rs' = 'fb189a1c98dac1f50caf19f2c755f7d4d52ffb54'
      'docs/architecture.md' = 'b68869b3cef6013c7351bc075779cf1b5acb2b6e'
      'docs/decisions.md' = '85e8f8656df91b2704501e458b15ad04576c8490'
      'docs/extending.md' = 'b39325a9b4181d33c8a0bdddf680ecf277a0c758'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P2b-T2: inline Files clauses (D43), a change per path (D44), bare names (D45)" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P2b-T3: kept step labels (D46), Expected lists (D47), comment expectations (D48), edit series (D49)

**Goal:** The recognition pieces of the devready format, each behind a spec key that no other format sets, except one: `[[outline.steps]] keep`; `script` `line_expect` and an `Expected:` after it that extends the comment expectation instead of being `expected-twice` (`Expectation.comments`, shown apart from the sentence); `expected-prose` `list`; `before-after` `series` with `PathOrigin::PreviousEdit`. Open item 5: `before-after.toml` gains the `Expected (n):` + list row, which changes only the cleanup plan's Task 6 step 2 (its paragraph and three list items become the preview script's prose expectation).

**Files**

- Modify `crates/planrun-cli/src/view.rs`
- Modify `crates/planrun-model/src/expect.rs`
- Modify `crates/planrun-model/src/path.rs`
- Modify `crates/planrun-parse/src/kinds/before_after.rs`
- Modify `crates/planrun-parse/src/kinds/expected.rs`
- Modify `crates/planrun-parse/src/kinds/run_line.rs`
- Modify `crates/planrun-parse/src/kinds/script.rs`
- Modify `crates/planrun-parse/src/outline.rs`
- Modify `crates/planrun-parse/src/recognize.rs`
- Modify `crates/planrun-parse/src/spec.rs`
- Create `crates/planrun-parse/tests/labels_and_expectations.rs`
- Modify `docs/decisions.md`
- Modify `docs/extending.md`
- Modify `formats/before-after.toml`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/src/view.rs`, `crates/planrun-parse/src/outline.rs`, `crates/planrun-parse/src/recognize.rs`, `crates/planrun-parse/src/spec.rs`, `docs/decisions.md`.

**Step 1 — write the files**

File `crates/planrun-cli/src/view.rs` (exact diff; save the block as `$env:TEMP\p2b-t3-view-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/view.rs b/crates/planrun-cli/src/view.rs
index e2571fa..3880f27 100644
--- a/crates/planrun-cli/src/view.rs
+++ b/crates/planrun-cli/src/view.rs
@@ -152,6 +152,9 @@ pub struct ExpectView {
     pub lines: Option<String>,
     /// The sentence.
     pub prose: Option<String>,
+    /// The command lines with a trailing comment expectation (decision D48).
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub comments: Option<String>,
 }
 
 /// Builds the views of one plan.
@@ -290,6 +293,7 @@ impl<'a> Viewer<'a> {
             exit,
             lines: expect.lines.as_ref().map(|l| self.text(l)),
             prose: expect.prose.as_ref().map(|p| self.text(p)),
+            comments: expect.comments.as_ref().map(|c| self.text(c)),
         }
     }
 }
@@ -302,6 +306,7 @@ fn path_from(origin: PathOrigin) -> Option<String> {
         PathOrigin::FileName => "the manifest entry with this file name",
         PathOrigin::OnlyEntry => "the task's only editable manifest entry",
         PathOrigin::Bare => "a bare file name in the manifest, its directory unknown",
+        PathOrigin::PreviousEdit => "the edit before it in the same numbered series",
     };
     Some(format!("resolved from manifest: {how}"))
 }
@@ -371,10 +376,15 @@ fn action_text(action: &ActionView, detail: Detail, out: &mut String) {
 
 fn expect_summary(e: &ExpectView) -> String {
     let lines = e.lines.as_ref().map(|l| l.lines().count());
-    match (lines, &e.prose) {
+    let comments = e.comments.as_ref().map(|c| c.lines().count());
+    let said = match (lines, &e.prose) {
         (Some(n), _) => format!("{}; {n} summary line(s)", e.exit),
         (None, Some(p)) => format!("{}; \"{p}\" (confirmed by you)", e.exit),
         (None, None) => e.exit.clone(),
+    };
+    match comments {
+        Some(n) => format!("{said}; {n} comment expectation(s) (confirmed by you)"),
+        None => said,
     }
 }
 
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t3-view-rs.patch"
Remove-Item "$env:TEMP\p2b-t3-view-rs.patch"
```

File `crates/planrun-model/src/expect.rs` (full contents):

```rust
//! What a script is expected to do.

use serde::{Deserialize, Serialize};

use crate::span::TextRef;

/// The expectation attached to a script (docs/architecture.md §6.4).
///
/// `lines` come from a fenced `Expected …:` block and are matched mechanically. `prose` comes
/// from a sentence (`Expected: all pass`), `comments` from trailing comments on the script's
/// command lines (`git branch --show-current   # main`, decisions D35, D48). Neither ever
/// passes a script on its own: the matcher derives `exit` from the sentence and the user
/// confirms both, labelled apart (decision D11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expectation {
    /// The expected exit status.
    pub exit: ExitExpect,
    /// Summary lines: an ordered subsequence of the output; `…` matches any run of characters.
    pub lines: Option<TextRef>,
    /// The expectation as a sentence.
    pub prose: Option<TextRef>,
    /// The script's lines that carry a trailing comment expectation, one per line.
    pub comments: Option<TextRef>,
}

/// The expected exit status of a script.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitExpect {
    /// Exit code 0.
    Success,
    /// Any non-zero exit code: a test that must fail before its implementation exists. Such a
    /// verification is skipped by default (decision D12).
    Failure,
    /// This exact code (`rg` with no match exits 1).
    Code(i32),
    /// The plan does not say.
    Unknown,
}
```

File `crates/planrun-model/src/path.rs` (full contents):

```rust
//! Repository-relative paths: the only path type an action may carry (decision D7).

use std::fmt;

use serde::{Deserialize, Serialize};

/// A path relative to the repository root, with forward slashes, that cannot leave the
/// repository by its spelling: no absolute path, no drive letter, no `..`, no `.` segment, no
/// empty segment, no backslash, no NUL, no wildcard (`*`, `?`: decision D42).
///
/// This checks the spelling only. Symlinks that point out of the repository are refused by the
/// workspace when it resolves the path on disk (planrun-exec).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RepoPath(Box<str>);

/// Why a path was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RepoPathError {
    /// Empty input.
    #[error("the path is empty")]
    Empty,
    /// `/x`, `C:\x`, `C:x` or `\\server\x`.
    #[error("`{0}` is absolute; plan paths are relative to the repository root")]
    Absolute(String),
    /// A `\` in the path.
    #[error("`{0}` contains a backslash; plan paths use forward slashes")]
    Backslash(String),
    /// A `..`, `.` or empty segment.
    #[error("`{0}` has a `.`, `..` or empty segment")]
    BadSegment(String),
    /// A NUL byte.
    #[error("the path contains a NUL byte")]
    Nul,
    /// A `*` or `?`: a pattern, never a file (decisions D41, D42).
    #[error("`{0}` contains a wildcard; a pattern is not a file path")]
    Wildcard(String),
}

impl RepoPath {
    /// Validates `path` as written in a plan.
    pub fn new(path: &str) -> Result<Self, RepoPathError> {
        if path.is_empty() {
            return Err(RepoPathError::Empty);
        }
        if path.contains('\0') {
            return Err(RepoPathError::Nul);
        }
        if path.contains('\\') {
            return Err(RepoPathError::Backslash(path.to_owned()));
        }
        if path.contains(['*', '?']) {
            return Err(RepoPathError::Wildcard(path.to_owned()));
        }
        let bytes = path.as_bytes();
        if path.starts_with('/') || (bytes.len() >= 2 && bytes[1] == b':') {
            return Err(RepoPathError::Absolute(path.to_owned()));
        }
        if path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        {
            return Err(RepoPathError::BadSegment(path.to_owned()));
        }
        Ok(Self(path.into()))
    }

    /// The path with forward slashes.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The last segment (`lib.rs` of `src/lib.rs`).
    pub fn file_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }

    /// The directory, without a trailing slash; `None` at the repository root.
    pub fn dir(&self) -> Option<&str> {
        self.0.rsplit_once('/').map(|(dir, _)| dir)
    }
}

/// Where a path in the IR came from, when the plan did not write it in full (decision D32).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathOrigin {
    /// Written in full where it is used.
    #[default]
    Written,
    /// A manifest token without a directory, placed in the directory of the token before it.
    Sibling,
    /// A lead that names only a file name: the one manifest entry with that file name.
    FileName,
    /// A lead that names no path: the task's one editable manifest entry.
    OnlyEntry,
    /// A manifest token without a directory after a token with one, in a format without
    /// siblings (decision D45): spelled as written, its location unknown. Never resolved to,
    /// and a Create onto it is `Manual(AmbiguousPath)`.
    Bare,
    /// An edit with no path that continues a numbered edit series: the path of the edit
    /// directly before it (decision D49).
    PreviousEdit,
}

impl PathOrigin {
    /// True when the plan wrote the path in full.
    pub fn is_written(&self) -> bool {
        *self == PathOrigin::Written
    }
}

impl TryFrom<String> for RepoPath {
    type Error = RepoPathError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<RepoPath> for String {
    fn from(value: RepoPath) -> Self {
        value.0.into()
    }
}

impl fmt::Display for RepoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_are_accepted() {
        for ok in [
            "a.rs",
            "crates/x/src/lib.rs",
            ".claude-plugin/plugin.json",
            "a/.x",
        ] {
            assert_eq!(RepoPath::new(ok).map(|p| p.to_string()), Ok(ok.to_owned()));
        }
    }

    #[test]
    fn escaping_or_ambiguous_paths_are_refused() {
        for bad in [
            "",
            "/etc/passwd",
            "C:\\x",
            "C:x",
            "a\\b",
            "../x",
            "a/../b",
            "./a",
            "a//b",
            "a/",
            "a/*.rs",
            "a/b?.rs",
        ] {
            assert!(RepoPath::new(bad).is_err(), "{bad:?}");
        }
    }
}
```

File `crates/planrun-parse/src/kinds/before_after.rs` (full contents):

```rust
//! `before-after`: a lead naming a file, the text to find in a fence, an optional separator
//! paragraph (`After:`), and its replacement in a second fence → `ReplaceExact`.
//!
//! A lead without a path first tries to continue a numbered edit series (`series`, decision
//! D49): "Edit 2" directly after the ReplaceExact of "`p`, edit 1" edits `p`. Otherwise it takes
//! the task's one editable manifest entry when `infer_path` is set; with none or several, the
//! lead and both fences are `Manual(ProseEdit)` (decision D32).

use std::sync::Arc;

use planrun_model::{
    Action, ActionKind, ManualReason, PathOrigin, RecognizerId, RepoPath, Span, TextRef,
};
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
        let series = p
            .opt_string("series")?
            .map(|s| compile_regex("series", s, &["n"]))
            .transpose()?;
        p.finish()?;
        let has_path = lead.capture_names().flatten().any(|n| n == "path");
        let pathless = infer_path || series.is_some();
        if has_path == pathless {
            return Err(
                "`lead` must capture `path`, or have no `path` capture and `infer_path = true` \
                 or a `series`"
                    .to_owned(),
            );
        }
        let lead_n = lead.capture_names().flatten().any(|n| n == "n");
        if series.is_some() && !lead_n {
            return Err("`series` needs a lead that captures the edit number `n`".to_owned());
        }
        Ok(Arc::new(BeforeAfter {
            id,
            lead,
            separator,
            infer_path,
            by_file_name,
            series,
        }))
    }
}

struct BeforeAfter {
    id: RecognizerId,
    lead: Regex,
    separator: Option<Regex>,
    infer_path: bool,
    by_file_name: bool,
    /// Reads the edit number from a lead; set, a lead without a path continues the series of
    /// the edit directly before it (D49).
    series: Option<Regex>,
}

impl BeforeAfter {
    /// The path of the edit this lead continues (D49): this lead's `n` is at least 2, and the
    /// action directly before it in the step is a ReplaceExact whose lead paragraph has number
    /// `n - 1` as `series` reads it.
    fn continued(&self, cx: &StepContext<'_>, caps: &regex::Captures<'_>) -> Option<RepoPath> {
        let series = self.series.as_ref()?;
        let n: u32 = caps.name("n")?.as_str().parse().ok().filter(|n| *n >= 2)?;
        let number = |text: &str| -> Option<u32> { series.captures(text)?["n"].parse().ok() };
        let before = cx.previous.last()?;
        let ActionKind::ReplaceExact { path, .. } = &before.kind else {
            return None;
        };
        let its_lead = cx
            .doc
            .blocks
            .iter()
            .find(|b| b.span.start == before.span.start)?;
        let (text, _) = paragraph(cx, its_lead)?;
        (number(text)? + 1 == n).then(|| path.clone())
    }

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
        let continued = self.continued(cx, &caps);
        let target = if let Some(path) = continued {
            Some((path, PathOrigin::PreviousEdit))
        } else if self.infer_path {
            only_entry(cx).map(|path| (path, PathOrigin::OnlyEntry))
        } else if self.series.is_some() {
            None
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
```

File `crates/planrun-parse/src/kinds/expected.rs` (full contents):

```rust
//! `expected-block` (`Expected (…):` + a `text` fence of summary lines) and `expected-prose`
//! (`Expected: …`, or with `list = true` an `Expected:` paragraph and the list after it,
//! decision D47): both amend the script directly before them. An `expected-prose` after a
//! script whose only expectation is its comment lines extends it (D48).

use std::sync::Arc;

use planrun_model::{Action, ActionKind, ExitExpect, Expectation, RecognizerId, Span, TextRef};
use regex::Regex;

use super::params::{ExitRow, Params, exit_rows, first_exit};
use super::{amendable, capture_ref, paragraph};
use crate::diagnostic::Diagnostic;
use crate::document::{Block, Located};
use crate::recognizer::{Cursor, Recognized, Recognizer, RecognizerFactory, StepContext};

/// Builds `expected-block` recognizers.
pub struct ExpectedBlockFactory;

/// Builds `expected-prose` recognizers.
pub struct ExpectedProseFactory;

impl RecognizerFactory for ExpectedBlockFactory {
    fn kind(&self) -> &'static str {
        "expected-block"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let lead = p.regex("lead", &["spec"])?;
        let exit = exit_rows(&mut p, "spec")?;
        let lines = p.opt_string("lines")?;
        if lines.is_some_and(|l| l != "summary") {
            return Err("`lines` must be \"summary\"".to_owned());
        }
        p.finish()?;
        Ok(Arc::new(ExpectedBlock { id, lead, exit }))
    }
}

impl RecognizerFactory for ExpectedProseFactory {
    fn kind(&self) -> &'static str {
        "expected-prose"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let list = p.opt_bool("list")?.unwrap_or(false);
        let needs: &[&str] = if list { &[] } else { &["text"] };
        let lead = p.regex("lead", needs)?;
        let exit = exit_rows(&mut p, "text")?;
        p.finish()?;
        Ok(Arc::new(ExpectedProse {
            id,
            lead,
            exit,
            list,
        }))
    }
}

struct ExpectedBlock {
    id: RecognizerId,
    lead: Regex,
    exit: Vec<ExitRow>,
}

struct ExpectedProse {
    id: RecognizerId,
    lead: Regex,
    exit: Vec<ExitRow>,
    /// The lead is followed by a list that belongs to it (D47).
    list: bool,
}

impl ExpectedProse {
    /// How many blocks after the lead belong to it: with `list`, the list items at the lead's
    /// depth and, nested in them, paragraphs, list items and `text` fences. Any other block ends
    /// it. `None` when `list` is set and no list follows.
    fn taken(&self, cx: &StepContext<'_>, cursor: &Cursor<'_>, lead: &Located) -> Option<usize> {
        if !self.list {
            return Some(0);
        }
        let belongs = |b: &Located| match &b.block {
            Block::ListItem { .. } if b.depth == lead.depth => true,
            Block::Paragraph { .. } | Block::ListItem { .. } => b.depth > lead.depth,
            Block::Fence { .. } => {
                b.depth > lead.depth && b.block.fence_language(cx.doc) == Some("text")
            }
            _ => false,
        };
        let first = cursor.peek(1)?;
        if !matches!(first.block, Block::ListItem { .. }) || first.depth != lead.depth {
            return None;
        }
        Some(
            (1..)
                .take_while(|&k| cursor.peek(k).is_some_and(belongs))
                .count(),
        )
    }
}

impl Recognizer for ExpectedBlock {
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
        let Some(caps) = self.lead.captures(text) else {
            return Recognized::No;
        };
        let script = match amendable(cx, lead.span) {
            Ok(script) => script,
            Err(d) => return Recognized::Malformed(d),
        };
        let spec = caps.name("spec").map_or("", |m| m.as_str());
        let Some(exit) = first_exit(&self.exit, spec) else {
            return Recognized::Malformed(Diagnostic::error(
                "expected-spec",
                Some(lead.span),
                format!("`{spec}` names no exit status this format knows"),
            ));
        };
        let fence = cursor.peek(1).filter(|b| {
            matches!(b.block, Block::Fence { .. }) && b.block.fence_language(cx.doc) == Some("text")
        });
        let Some(fence) = fence else {
            return Recognized::Malformed(Diagnostic::error(
                "missing-block",
                Some(lead.span),
                "this expectation announces a ```text block of summary lines, and none follows it",
            ));
        };
        let Block::Fence { body, .. } = &fence.block else {
            return Recognized::No;
        };
        let expect = Expectation {
            exit,
            lines: Some(body.clone()),
            prose: None,
            comments: None,
        };
        cursor.advance(2);
        Recognized::Amend(amended(script, expect, lead.span.join(fence.span)))
    }
}

impl Recognizer for ExpectedProse {
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
        let Some(taken) = self.taken(cx, cursor, lead) else {
            return Recognized::Malformed(Diagnostic::error(
                "missing-block",
                Some(lead.span),
                "this expectation announces a list, and none follows it",
            ));
        };
        let span = cursor
            .peek(taken)
            .map_or(lead.span, |last| lead.span.join(last.span));
        if let Some(manual) = joined_manual(cx, span) {
            cursor.advance(1 + taken);
            return Recognized::Amend(manual);
        }
        let (script, comments) = match extendable(cx, lead.span) {
            Ok(found) => found,
            Err(d) => return Recognized::Malformed(d),
        };
        let (said, prose) = if self.list {
            (cx.doc.source.slice(span), TextRef::Span(span))
        } else {
            let said = caps.name("text").map_or("", |m| m.as_str());
            let prose = capture_ref(&caps, "text", base).unwrap_or(TextRef::Span(lead.span));
            (said, prose)
        };
        let expect = Expectation {
            exit: first_exit(&self.exit, said).unwrap_or(ExitExpect::Unknown),
            lines: None,
            prose: Some(prose),
            comments,
        };
        cursor.advance(1 + taken);
        Recognized::Amend(amended(script, expect, span))
    }
}

/// The script an `expected-prose` amends, and the comment expectations it extends: a script
/// whose only expectation is its comment lines takes the sentence too (D48); any other
/// expectation already there is `expected-twice`.
fn extendable(cx: &StepContext<'_>, lead: Span) -> Result<(Action, Option<TextRef>), Diagnostic> {
    let comments_only = cx.previous.last().and_then(|a| match &a.kind {
        ActionKind::RunScript(s) | ActionKind::Commit(s) => s
            .expect
            .as_ref()
            .filter(|e| e.lines.is_none() && e.prose.is_none())
            .and_then(|e| e.comments.clone())
            .map(|c| (a.clone(), c)),
        _ => None,
    });
    if let Some((script, comments)) = comments_only {
        return Ok((script, Some(comments)));
    }
    amendable(cx, lead).map(|script| (script, None))
}

/// The Manual directly before an expectation, grown over it: what to check is part of what the
/// user does by hand (decision D38).
fn joined_manual(cx: &StepContext<'_>, lead: Span) -> Option<Action> {
    let mut manual = cx
        .previous
        .last()
        .filter(|a| matches!(a.kind, ActionKind::Manual { .. }))?
        .clone();
    manual.span = manual.span.join(lead);
    if let ActionKind::Manual { text, .. } = &mut manual.kind {
        *text = TextRef::Span(manual.span);
    }
    Some(manual)
}

/// `script` with `expect` attached, its span grown over the expectation.
fn amended(mut script: Action, expect: Expectation, over: Span) -> Action {
    if let ActionKind::RunScript(s) | ActionKind::Commit(s) = &mut script.kind {
        s.expect = Some(expect);
    }
    script.span = script.span.join(over);
    script
}
```

File `crates/planrun-parse/src/kinds/run_line.rs` (full contents):

```rust
//! `run-line`: a command in a paragraph (`` Run: `cargo test` ``) → `RunScript` in the spec's
//! shell, with the `Expected: …` of the same paragraph as its expectation (decision D39).

use std::sync::Arc;

use planrun_model::{Action, ExitExpect, Expectation, RecognizerId, Script, ScriptIntent, Shell};
use regex::Regex;

use super::params::{ExitRow, Params, exit_rows, first_exit};
use super::script::{IntentRow, intent_of, intent_rows, script_action};
use super::{capture_ref, paragraph};
use crate::diagnostic::Diagnostic;
use crate::recognizer::{Cursor, Recognized, Recognizer, RecognizerFactory, StepContext};

/// Builds `run-line` recognizers.
pub struct RunLineFactory;

impl RecognizerFactory for RunLineFactory {
    fn kind(&self) -> &'static str {
        "run-line"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let lead = p.regex("lead", &["cmd"])?;
        let shell = match p.string("shell")? {
            "pwsh" => Shell::Pwsh,
            "bash" => Shell::Bash,
            other => return Err(format!("shell `{other}` is not pwsh or bash")),
        };
        let exit = exit_rows(&mut p, "text")?;
        let intents = intent_rows(&mut p, false)?;
        p.finish()?;
        Ok(Arc::new(RunLine {
            id,
            lead,
            shell,
            exit,
            intents,
        }))
    }
}

struct RunLine {
    id: RecognizerId,
    lead: Regex,
    shell: Shell,
    exit: Vec<ExitRow>,
    intents: Vec<IntentRow>,
}

impl Recognizer for RunLine {
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
        let Some(body) = capture_ref(&caps, "cmd", base).filter(|_| !caps["cmd"].is_empty()) else {
            return Recognized::Malformed(Diagnostic::error(
                "missing-block",
                Some(lead.span),
                "this line announces a command, and names none",
            ));
        };
        let expect = caps.name("expected").map(|said| Expectation {
            exit: first_exit(&self.exit, said.as_str()).unwrap_or(ExitExpect::Unknown),
            lines: None,
            prose: capture_ref(&caps, "expected", base),
            comments: None,
        });
        let script = Script {
            shell: self.shell,
            body,
            expect,
            intent: ScriptIntent::Verify,
        };
        let kind = script_action(script, intent_of(&self.intents, &caps["cmd"], None));
        cursor.advance(1);
        Recognized::Yes(vec![Action {
            kind,
            span: lead.span,
            origin: self.id.clone(),
            path_origin: Default::default(),
        }])
    }
}
```

File `crates/planrun-parse/src/kinds/script.rs` (full contents):

```rust
//! `script`: a fence in a shell language → `RunScript`, or `Commit` when its intent says so.
//!
//! With `leads` set, the kind claims a fence only behind a run lead or, with `first_block`, as
//! the first block of a written step; any other shell fence falls to the fallback, Manual
//! (decision D28). An `intents` row matches the body, the lead paragraph, or both (D29). With
//! `line_expect`, the command lines with a trailing comment expectation become the script's
//! comment expectation, always confirmed (D35, D48).

use std::sync::Arc;

use planrun_model::{
    Action, ActionKind, ExitExpect, Expectation, RecognizerId, Script, ScriptIntent, TextRef,
};
use regex::Regex;

use super::params::{Params, row_string};
use crate::document::Block;
use crate::recognizer::{Cursor, Recognized, Recognizer, RecognizerFactory, StepContext};
use crate::spec::compile_regex;

/// Builds `script` recognizers.
pub struct ScriptFactory;

/// What an `intents` row selects.
#[derive(Debug, Clone, Copy)]
pub(super) enum Intent {
    /// The `Commit` action kind (its script carries `ScriptIntent::Other`).
    Commit,
    /// A `RunScript` with this intent.
    Run(ScriptIntent),
}

/// One `intents` row: what it matches, and the intent it gives.
pub(super) struct IntentRow {
    body: Option<Regex>,
    lead: Option<Regex>,
    intent: Intent,
}

impl IntentRow {
    fn matches(&self, body: &str, lead: Option<&str>) -> bool {
        let body_ok = self.body.as_ref().is_none_or(|r| r.is_match(body));
        let lead_ok = self
            .lead
            .as_ref()
            .is_none_or(|r| lead.is_some_and(|l| r.is_match(l)));
        body_ok && lead_ok
    }
}

/// The `intents` rows of a kind; `lead` is allowed when `with_lead`.
pub(super) fn intent_rows(p: &mut Params<'_>, with_lead: bool) -> Result<Vec<IntentRow>, String> {
    let keys: &[&str] = if with_lead {
        &["intent", "body", "lead"]
    } else {
        &["intent", "body"]
    };
    p.rows("intents", keys)?
        .into_iter()
        .map(|row| {
            let intent = intent_named(row_string(row, "intent")?)?;
            let pattern = |key: &str| {
                row.get(key)
                    .map(|v| {
                        let text = v
                            .as_str()
                            .ok_or_else(|| format!("intents.{key} must be a string"))?;
                        compile_regex(&format!("intents.{key}"), text, &[])
                    })
                    .transpose()
            };
            let (body, lead) = (pattern("body")?, pattern("lead")?);
            if body.is_none() && lead.is_none() {
                return Err("an intents row needs `body` or `lead`".to_owned());
            }
            Ok(IntentRow { body, lead, intent })
        })
        .collect()
}

/// The intent of the first row that matches; `verify` when none does.
pub(super) fn intent_of(rows: &[IntentRow], body: &str, lead: Option<&str>) -> Intent {
    rows.iter()
        .find(|row| row.matches(body, lead))
        .map_or(Intent::Run(ScriptIntent::Verify), |row| row.intent)
}

/// `script` as the action of `intent`.
pub(super) fn script_action(script: Script, intent: Intent) -> ActionKind {
    match intent {
        Intent::Commit => ActionKind::Commit(Script {
            intent: ScriptIntent::Other,
            ..script
        }),
        Intent::Run(intent) => ActionKind::RunScript(Script { intent, ..script }),
    }
}

impl RecognizerFactory for ScriptFactory {
    fn kind(&self) -> &'static str {
        "script"
    }

    fn build(&self, table: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        let mut p = Params::new(table);
        let id = p.id()?;
        let intents = intent_rows(&mut p, true)?;
        let leads = p
            .opt_strings("leads")?
            .map(|list| {
                list.iter()
                    .map(|l| compile_regex("leads", l, &[]))
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;
        let first_block = p.opt_bool("first_block")?;
        let line_expect = p
            .opt_string("line_expect")?
            .map(|l| compile_regex("line_expect", l, &["expect"]))
            .transpose()?;
        p.finish()?;
        if first_block.is_some() && leads.is_none() {
            return Err("`first_block` needs `leads`".to_owned());
        }
        Ok(Arc::new(ScriptRecognizer {
            id,
            intents,
            leads,
            first_block: first_block.unwrap_or(false),
            line_expect,
        }))
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
    intents: Vec<IntentRow>,
    /// When set, only fences behind one of these leads (or a first block) are claimed.
    leads: Option<Vec<Regex>>,
    first_block: bool,
    /// Over each command line; a match with an `expect` capture is a comment expectation.
    line_expect: Option<Regex>,
}

impl ScriptRecognizer {
    /// The comment expectation of `body` (D35): every line that is not a whole-line comment and
    /// matches `line_expect`, one per line, exit not checked. `None` without a match.
    fn comments(&self, body: &TextRef, text: &str) -> Option<Expectation> {
        let regex = self.line_expect.as_ref()?;
        let lines: Vec<&str> = text
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .filter(|l| {
                regex
                    .captures(l)
                    .is_some_and(|c| c.name("expect").is_some())
            })
            .map(str::trim)
            .collect();
        if lines.is_empty() {
            return None;
        }
        Some(Expectation {
            exit: ExitExpect::Unknown,
            lines: None,
            prose: None,
            comments: Some(TextRef::Owned {
                text: lines.join("\n").into(),
                from: body.span(),
            }),
        })
    }

    /// Whether the guard (decision D28) lets the kind claim the fence at the cursor. A body
    /// intent never qualifies a fence on its own.
    fn claims(&self, cx: &StepContext<'_>, cursor: &Cursor<'_>, lead: Option<&str>) -> bool {
        let Some(leads) = &self.leads else {
            return true;
        };
        if self.first_block && cx.written_step && cursor.at_start() {
            return true;
        }
        lead.is_some_and(|text| leads.iter().any(|l| l.is_match(text)))
    }
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
        let lead = cursor.previous().and_then(|b| match &b.block {
            Block::Paragraph { text } => Some(cx.doc.text(text)),
            _ => None,
        });
        if !self.claims(cx, cursor, lead) {
            return Recognized::No;
        }
        let text = cx.doc.text(body);
        let script = Script {
            shell,
            body: body.clone(),
            expect: self.comments(body, text),
            intent: ScriptIntent::Verify,
        };
        let kind = script_action(script, intent_of(&self.intents, text, lead));
        cursor.advance(1);
        Recognized::Yes(vec![Action {
            kind,
            span: located.span,
            origin: self.id.clone(),
            path_origin: Default::default(),
        }])
    }
}
```

File `crates/planrun-parse/src/outline.rs` (exact diff; save the block as `$env:TEMP\p2b-t3-outline-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/outline.rs b/crates/planrun-parse/src/outline.rs
index e1a2b23..1bd76e6 100644
--- a/crates/planrun-parse/src/outline.rs
+++ b/crates/planrun-parse/src/outline.rs
@@ -40,6 +40,8 @@ pub struct StepRule {
     pub block: StepBlock,
     /// Over its text; may capture `n` and `title`.
     pub regex: Regex,
+    /// The boundary block is the step's first block (decision D46).
+    pub keep: bool,
 }
 
 /// The kinds of block a step can start with.
@@ -303,6 +305,22 @@ enum StepNumber {
     Bad,
 }
 
+/// True when the boundary at `index` stays in its step (decision D46).
+fn kept(rules: &OutlineRules, doc: &Document, index: usize) -> bool {
+    let (kind, text) = match &doc.blocks[index].block {
+        Block::Paragraph { text } => (StepBlock::Paragraph, text),
+        Block::ListItem { text, .. } => (StepBlock::ListItem, text),
+        Block::Heading { text, .. } => (StepBlock::Heading, text),
+        _ => return false,
+    };
+    rules
+        .steps
+        .iter()
+        .filter(|r| r.block == kind)
+        .find(|r| r.regex.is_match(doc.text(text)))
+        .is_some_and(|r| r.keep)
+}
+
 /// The step that block `index` starts, as (number, title), when it is a step boundary.
 fn boundary(
     rules: &OutlineRules,
@@ -362,7 +380,7 @@ fn steps(
     }
     for (k, &b) in boundaries.iter().enumerate() {
         let blocks = BlockRange {
-            start: b + 1,
+            start: if kept(rules, doc, b) { b } else { b + 1 },
             end: boundaries.get(k + 1).copied().unwrap_or(body.end),
         };
         let Some((number, title)) = boundary(rules, doc, b) else {
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t3-outline-rs.patch"
Remove-Item "$env:TEMP\p2b-t3-outline-rs.patch"
```

File `crates/planrun-parse/src/recognize.rs` (exact diff; save the block as `$env:TEMP\p2b-t3-recognize-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/recognize.rs b/crates/planrun-parse/src/recognize.rs
index 812c67a..2fb278b 100644
--- a/crates/planrun-parse/src/recognize.rs
+++ b/crates/planrun-parse/src/recognize.rs
@@ -190,6 +190,7 @@ fn expect_failure(action: &mut Action) -> Option<Diagnostic> {
             exit: ExitExpect::Failure,
             lines: None,
             prose: None,
+            comments: None,
         });
         return None;
     };
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t3-recognize-rs.patch"
Remove-Item "$env:TEMP\p2b-t3-recognize-rs.patch"
```

File `crates/planrun-parse/src/spec.rs` (exact diff; save the block as `$env:TEMP\p2b-t3-spec-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/spec.rs b/crates/planrun-parse/src/spec.rs
index e4e2d3d..91fd1f8 100644
--- a/crates/planrun-parse/src/spec.rs
+++ b/crates/planrun-parse/src/spec.rs
@@ -137,6 +137,9 @@ pub struct StepPattern {
     pub block: String,
     /// Regex over the block text; may capture `n` and `title`.
     pub regex: String,
+    /// The boundary block stays in its step, so recognizers see it (a label such as
+    /// ``**2a.** `p`, edit 1 (old → new):``); default false (decision D46).
+    pub keep: Option<bool>,
 }
 
 /// Shells and manual markers.
@@ -269,6 +272,7 @@ fn compile_outline(spec: &OutlineSpec, marks: &[StepMarkSpec]) -> Result<Outline
             Ok(StepRule {
                 block: step_block(&s.block)?,
                 regex: compile_regex("outline.steps", &s.regex, &[])?,
+                keep: s.keep.unwrap_or(false),
             })
         })
         .collect::<Result<_, String>>()?;
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t3-spec-rs.patch"
Remove-Item "$env:TEMP\p2b-t3-spec-rs.patch"
```

File `crates/planrun-parse/tests/labels_and_expectations.rs` (full contents):

```rust
//! Step labels that stay in their step (decision D46), `Expected:` with a list (D47), comment
//! expectations and what extends them (D35, D48), and numbered edit series (D49).

use planrun_model::{ActionKind, ExitExpect};
use planrun_parse::{Registry, parse_plan};

const SPEC: &str = r#"
id = "t"
name = "labels and expectations test"
[detect]
patterns = [{ regex = '^### Task', weight = 60 }]
[outline]
task = { level = 3, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
[outline.manifest]
marker = '^\*\*Files:\*\* (?P<change>modify)$'
item = '^`(?P<paths>[^`]+)`$'
changes = { modify = "modify" }
required = false
[[outline.steps]]
block = "paragraph"
regex = '^\*\*(?P<title>\d+[a-z]\.[^*]*|Verify:)\*\*'
keep = true
[checks]
manifest-untouched = "off"
manifest-undeclared = "warning"
[[recognizer]]
kind = "before-after"
id = "before-after#path"
lead = '^(?:\*\*\d+[a-z]\.\*\* )?`(?P<path>[^`]+)`, edit (?P<n>\d+) \(old → new\):$'
[[recognizer]]
kind = "before-after"
id = "before-after#series"
lead = '^Edit (?P<n>\d+) \(old → new\):$'
series = '(?i)\bedit (?P<n>\d+) \(old → new\)'
infer_path = true
[[recognizer]]
kind = "script"
id = "script#t"
leads = ['^\*\*Verify:\*\*$']
first_block = true
line_expect = '\s#\s(?P<expect>\S.*)$'
[[recognizer]]
kind = "expected-block"
id = "expected-block#t"
lead = '^Expected \((?P<spec>[^)]+)\):$'
exit = [{ spec = '^exit code (?P<code>\d+)$', exit = "code" }]
[[recognizer]]
kind = "expected-prose"
id = "expected-prose#t"
lead = '^Expected: (?P<text>.+)$'
exit = [{ text = '(?i)\ball pass\b', exit = "success" }]
[[recognizer]]
kind = "expected-prose"
id = "expected-prose#list"
lead = '^Expected:$'
list = true
exit = [{ text = '(?i)\b0 failed\b', exit = "success" }]
"#;

fn registry(spec: &str) -> Result<Registry, String> {
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    registry.add_spec("t", spec).map_err(|e| e.to_string())?;
    Ok(registry)
}

fn parse(text: &str) -> Result<planrun_parse::Parsed, Vec<&'static str>> {
    let registry = registry(SPEC).unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, "t.md", text.as_bytes(), Some("t")).map_err(|d| d.error_codes())
}

fn ok(text: &str) -> planrun_parse::Parsed {
    parse(text).unwrap_or_else(|codes| panic!("{codes:?}"))
}

/// One label per action of the first task, by step.
fn labels(parsed: &planrun_parse::Parsed) -> Vec<Vec<String>> {
    let plan = &parsed.plan;
    plan.tasks[0]
        .steps
        .iter()
        .map(|s| {
            s.actions
                .iter()
                .map(|a| match &a.kind {
                    ActionKind::Note { .. } => "note".to_owned(),
                    ActionKind::Manual { reason, .. } => format!("manual {reason:?}"),
                    ActionKind::ReplaceExact { path, .. } => {
                        format!("replace {path} {:?}", a.path_origin)
                    }
                    ActionKind::RunScript(s) => {
                        let e = s.expect.as_ref();
                        let exit = e.map_or(ExitExpect::Unknown, |e| e.exit);
                        let comments = e.and_then(|e| e.comments.as_ref()).map(|c| plan.text(c));
                        let prose = e.and_then(|e| e.prose.as_ref()).map(|p| plan.text(p));
                        format!("run {exit:?} comments={comments:?} prose={prose:?}")
                    }
                    other => format!("{other:?}"),
                })
                .collect()
        })
        .collect()
}

fn fence(lang: &str, body: &str) -> String {
    format!("```{lang}\n{body}\n```\n\n")
}

fn task(body: &str) -> String {
    format!("### Task 1: a\n\n**Files:** modify\n- `a/x.ps1`\n- `a/y.ps1`\n\n{body}")
}

#[test]
fn a_kept_label_is_the_first_block_of_its_step() {
    let text = task(&format!(
        "**2a.** `a/x.ps1`, edit 1 (old → new):\n\n{}{}",
        fence("powershell", "old"),
        fence("powershell", "new")
    ));
    assert_eq!(
        labels(&ok(&text)),
        [vec!["replace a/x.ps1 Written".to_owned()]]
    );
}

#[test]
fn first_block_never_claims_the_fence_after_a_kept_label() {
    let text = task(&format!(
        "**2b.** Something else\n\n{}",
        fence("powershell", "rm x")
    ));
    assert_eq!(
        labels(&ok(&text)),
        [vec!["note".to_owned(), "manual Unrecognized".to_owned()]]
    );
}

#[test]
fn trailing_comments_become_a_comment_expectation() {
    let body = "# a whole-line comment\ngit status --short   # empty\ncargo build\n";
    let text = task(&format!("**Verify:**\n\n{}", fence("powershell", body)));
    let expected = "run Unknown comments=Some(\"git status --short   # empty\") prose=None";
    assert_eq!(labels(&ok(&text))[0], ["note", expected]);
}

#[test]
fn a_script_without_comment_lines_has_no_expectation() {
    let text = task(&format!(
        "**Verify:**\n\n{}",
        fence("powershell", "cargo build")
    ));
    assert_eq!(
        labels(&ok(&text))[0][1],
        "run Unknown comments=None prose=None"
    );
}

#[test]
fn an_expected_sentence_extends_the_comment_expectation() {
    let text = task(&format!(
        "**Verify:**\n\n{}Expected: all pass\n",
        fence("powershell", "cargo test   # 3 passed")
    ));
    let expected =
        "run Success comments=Some(\"cargo test   # 3 passed\") prose=Some(\"all pass\")";
    assert_eq!(labels(&ok(&text))[0][1], expected);
}

#[test]
fn an_expected_list_extends_it_too() {
    let text = task(&format!(
        "**Verify:**\n\n{}Expected:\n- `3 passed, 0 failed`\n- no warnings\n",
        fence("bash", "cargo test   # 3 passed")
    ));
    let found = &labels(&ok(&text))[0][1];
    assert!(
        found.starts_with("run Success comments=Some(\"cargo test   # 3 passed\") prose=Some(\"Expected:\\n- `3 passed"),
        "{found}"
    );
}

#[test]
fn a_second_expectation_is_still_expected_twice() {
    let two = task(&format!(
        "**Verify:**\n\n{}Expected: all pass\n\nExpected: all pass\n",
        fence("powershell", "cargo test   # 3 passed")
    ));
    assert_eq!(parse(&two).err(), Some(vec!["expected-twice"]));
    let block = task(&format!(
        "**Verify:**\n\n{}Expected (exit code 0):\n\n{}",
        fence("powershell", "cargo test   # 3 passed"),
        fence("text", "ok")
    ));
    assert_eq!(parse(&block).err(), Some(vec!["expected-twice"]));
}

#[test]
fn an_expected_list_takes_nested_prose_and_stops_at_a_shell_fence() {
    let text = task(&format!(
        "**Verify:**\n\n{}Expected:\n- one\n  - nested\n\n  more text\n\n- two\n\n  {}\nAfter.\n",
        fence("powershell", "cargo test"),
        "```powershell\n  cargo build\n  ```\n"
    ));
    let found = labels(&ok(&text));
    assert!(found[0][1].contains("- two"), "{found:?}");
    assert_eq!(found[0].len(), 4, "{found:?}");
    assert_eq!(found[0][3], "note");
    assert!(found[0][2].starts_with("manual Unrecognized"), "{found:?}");
}

#[test]
fn an_expected_list_lead_without_a_list_is_malformed() {
    let text = task(&format!(
        "**Verify:**\n\n{}Expected:\n\nAfter.\n",
        fence("powershell", "cargo test")
    ));
    assert_eq!(parse(&text).err(), Some(vec!["missing-block"]));
}

#[test]
fn an_expected_list_after_a_manual_joins_it() {
    let text = task(&format!(
        "Some prose:\n\n{}Expected:\n- it works\n",
        fence("powershell", "cargo test")
    ));
    assert_eq!(
        labels(&ok(&text)),
        [vec!["note".to_owned(), "manual Unrecognized".to_owned()]]
    );
}

fn edit(lead: &str) -> String {
    format!(
        "{lead}\n\n{}{}",
        fence("powershell", "old"),
        fence("powershell", "new")
    )
}

#[test]
fn a_numbered_series_continues_the_path_of_the_edit_before_it() {
    let text = task(&format!(
        "{}{}{}",
        edit("`a/y.ps1`, edit 1 (old → new):"),
        edit("Edit 2 (old → new):"),
        edit("Edit 3 (old → new):")
    ));
    let expected = [
        "replace a/y.ps1 Written",
        "replace a/y.ps1 PreviousEdit",
        "replace a/y.ps1 PreviousEdit",
    ];
    assert_eq!(labels(&ok(&text))[0], expected);
}

#[test]
fn anything_between_or_a_gap_in_the_numbers_breaks_the_series() {
    let between = task(&format!(
        "{}A note.\n\n{}",
        edit("`a/y.ps1`, edit 1 (old → new):"),
        edit("Edit 2 (old → new):")
    ));
    assert_eq!(
        labels(&ok(&between))[0],
        ["replace a/y.ps1 Written", "note", "manual ProseEdit"]
    );
    let gap = task(&format!(
        "{}{}",
        edit("`a/y.ps1`, edit 1 (old → new):"),
        edit("Edit 3 (old → new):")
    ));
    assert_eq!(
        labels(&ok(&gap))[0],
        ["replace a/y.ps1 Written", "manual ProseEdit"]
    );
}

#[test]
fn a_series_needs_a_lead_that_captures_its_number() {
    let spec = SPEC.replace(
        "lead = '^Edit (?P<n>\\d+) \\(old → new\\):$'",
        "lead = '^Edit \\d+ \\(old → new\\):$'",
    );
    assert_ne!(spec, SPEC);
    let err = registry(&spec).err().unwrap_or_default();
    assert!(err.contains("`series` needs a lead"), "{err}");
}
```

File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p2b-t3-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index 85e8f86..2459eb3 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -185,6 +185,10 @@ can create a script file whose contents contain git commands. Reason: hand-writt
 file contents and code to insert in shell fences behind prose that no recognizer can reliably
 name; unguarded, `script` would execute them. The assay spec sets no `leads` and is unchanged.
 
+In a step whose boundary is kept (D46), the boundary label is the step's first block, so
+`first_block` never matches there: a fence directly after a kept label runs only when the
+label itself matches one of `leads`.
+
 ## D29 Script intents by lead; no `preview-apply` kind
 
 An `intents` row of `script` matches the body, the lead paragraph, or both (`body`, `lead`).
@@ -246,6 +250,9 @@ A `script` parameter `line_expect` (a regex over each line) turns trailing comme
 `# feature/p2-tooling` into one prose expectation per fence: exit `Unknown`, confirmed (D11).
 DevReady only. A fence that also has an `Expected:` after it is `expected-twice`.
 
+Superseded in part by D48: an `Expected:` sentence or list after such a fence extends its
+expectation instead of being `expected-twice`.
+
 ## D36 Imperative prose
 
 Per-format `manual_markers` and `human-check` step marks catch the instructions that would
@@ -358,3 +365,48 @@ is unknown.
 - D33: a Bare entry is not literal, so the commit suggestion shows the message only.
 - `show` marks Bare entries. `AmbiguousPath` is produced only by this rule; it is not a
   `manual-prose` reason.
+
+## D46 Step labels that stay in their step
+
+`[[outline.steps]] keep = true`: the boundary block is the first block of its step instead of
+being consumed by the outline. Hand-written plans put the step's lead in its label
+(``**2a.** `p`, edit 1 (old → new):``, ``**Verify:** `cmd` → …``, ``**Commit:** `msg` ``), and
+recognizers see it only when it stays. A kept label no recognizer takes is a `Note`. With
+`keep`, `first_block` cannot match in that step (see D28): only `leads` decide which fence after
+a label runs.
+
+## D47 `Expected:` with a list
+
+`expected-prose` with `list = true`: a lead paragraph (``^Expected(?: \([^)]*\))?:$``) and the
+list right after it are one prose expectation (D11: always confirmed). The list is the list
+items at the lead's depth and, nested in them, paragraphs, list items and `text` fences; any
+other block (a shell fence above all) ends it and goes through recognition as usual. A step
+boundary ends it too. The exit table reads the whole text. A lead with no list after it is
+`missing-block`. After a Manual, the lead and its list join the Manual (D38). The lead
+`Expected and recorded:` is not a list lead: its items are instructions, not expectations.
+
+## D48 A sentence after comment expectations extends them
+
+Supersedes D35 in part. An `expected-prose` (plain or `list = true`) directly after a script
+whose only expectation is its comment lines (`line_expect`) extends that expectation instead of
+being `expected-twice`. The exit status comes from the sentence's exit table; the comment lines
+and the sentence stay two labelled parts (`comments`, `prose`), so the confirmation (D11) shows
+which is which. An `expected-block` there is still `expected-twice`, and so is a second
+`Expected:` after one script. The body always runs unchanged: a comment that is a remark, or a
+`#` inside a string, only adds text to what is shown for confirmation.
+
+## D49 Numbered edit series
+
+`before-after` parameter `series`: a regex with an `n` capture that reads the edit number of a
+lead paragraph. A lead without a path whose own `n` capture is at least 2 continues the series
+when the action directly before it in the same step is a ReplaceExact whose lead paragraph has
+number `n - 1`: it edits that path, with `PathOrigin::PreviousEdit`.
+
+- Any action in between (a Note included) breaks the series. A chain is fine: edit 3 continues
+  the edit 2 that got `PreviousEdit`.
+- A continuation is tried before `infer_path`, and only on leads without a path. D32's
+  `infer_path` rule is unchanged.
+- The spec load refuses `series` on a lead that does not capture `n`.
+- The previous edit's number is read from its lead paragraph by `series`, because a
+  recognizer cannot see another recognizer's captures (recognizers are pure and independent,
+  extending.md §5); the edit with the path is usually matched by another recognizer instance.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t3-decisions-md.patch"
Remove-Item "$env:TEMP\p2b-t3-decisions-md.patch"
```

File `docs/extending.md` (full contents):

````markdown
# Extending planrun

New plan styles will keep appearing. They are added without touching execution, in three levels
(architecture.md §5). Use the lowest level that works.

## 1. Decide the level

1. Describe the style in a table like the ones in [plan-grammar.md](plan-grammar.md), from at
   least two real plans.
2. For each row, find the built-in recognizer kind that covers it (§3). If every row is covered,
   write a **format spec** (§2). Done, no rebuild.
3. If a row is a new idiom, write a **recognizer kind** (§5) and use it from a spec.
4. Only if the plan's *structure* is not headings-with-steps (tasks in a table, a JSON plan),
   write a **Format** (§6).
5. A new **action kind** (§4) is almost never the answer: map the wording onto an existing action.

## 2. Format specs (TOML)

Location: built-ins in `formats/<id>.toml` (embedded); per repository in
`<repo>/.planrun/formats/<id>.toml`; per user in the config directory. The file name is the id.

```toml
id = "assay"                      # unique; the file name
name = "Assay plan (planbuild.py)"

[detect]                          # a pattern that matches a line outside fenced blocks adds
patterns = [ { regex = '^### P\d+-T\d+: ', weight = 60 } ]   # its weight once; capped at 100

[outline]
tasks_after = { level = 2, regex = '^Tasks$' }          # optional
task = { level = 3, regex = '^(?P<id>(?P<phase>P\d+)-T(?P<ord>\d+)): (?P<title>.+)$' }
tasks_end_level = 2                                      # optional
# captures: id, ord (a number), title (all required); phase (optional, from the first task)

[outline.manifest]                                       # optional
# The marker and its items are consumed by the outline: they fall in no step. The items end at
# the first block that is no list item at the marker's depth, or that is a step boundary (D37).
marker = '^\*\*Files\*\*$'                                # may capture `change` (`**Files:** create`)
# and `items`: then the marker paragraph holds the manifest, split by `clause_list`, each
# clause matched by `item`; text that declares nothing is shown as a Note (D43)
item = '^(?P<change>Create|Modify|Delete) `(?P<paths>[^`]+)`$'   # `change` unless the marker has it
changes = { Create = "create", Modify = "modify", Delete = "delete" }
# → create | modify | rewrite | delete | test | generated (D27)
path_list = ', '                                         # optional: split `paths` into several;
# a `:3-10` piece adds lines to the path before it (D32). Built in, before the split, in every
# format: `a`–`b` (an en dash between code spans) is one range entry, so `path_list` must not
# list the en dash; a piece with `*`, `?` or `{` is a glob entry (D41). Pattern entries declare
# the files they match and are never a write target.
required = true                                          # optional, default true: false lets a
                                                         # task have no manifest (D27)
siblings = false                                         # optional, default false: a bare file
                                                         # name after `a/b.rs` is `a/<name>` (D32);
                                                         # without siblings it is Bare (D45)
clause_list = '; '                                       # optional: needs an `items` capture (D43)
path_change = '^` \((?P<change>full replacement)\)'     # optional (D44): over the text right
# after a path token; `change` goes through `changes` and wins over the item's change, which
# wins over the marker's

[[outline.steps]]                                        # zero or more; none = one implicit step
block = "paragraph"                                      # paragraph | list-item | heading
regex = '^\*\*Step (?P<n>\d+) — (?P<title>[^*]+)\*\*$'   # n and title optional
keep = false                                             # optional: true keeps the boundary
                                                         # block in its step (D46)

[settings]
manual_markers = ['(?i)\bby hand\b']                     # paragraph → Manual(Marked)
pwsh = ["powershell", "pwsh", "ps1"]                     # optional; these are the defaults
bash = ["bash", "sh", "shell"]

[[step_marks]]                                           # zero or more (D30), first match wins
title = '(?i)\bverify (?:it|they) fails?\b'              # regex over the step's title
mark = "expected-failure"                                # expected-failure | human-check
# expected-failure: the step's Verify scripts expect failure (none or `any` becomes Failure; an
# expected success is the error expected-conflict). human-check: its prose Notes become
# Manual(HumanCheck).

[checks]                                                 # optional (D31): only these two codes,
manifest-untouched = "off"                               # each error | warning | off; any
manifest-undeclared = "warning"                          # other key fails the spec load

[[recognizer]]                                           # ordered; first match wins
kind = "file-block"                                      # a registered kind (§3)
id = "file-block#assay"                                  # shown by `show` and coverage
# … the kind's own parameters
```

Rules:

- Unknown keys are errors (`deny_unknown_fields`). A spec is fully validated when the registry
  loads it (unknown kind, bad regex, bad parameter), before any plan is read.
- Regexes use the `regex` crate syntax; `(?P<name>…)` captures. A regex that lacks a capture
  its reader requires (`path`, `id`, …) is refused when the spec loads.
- `crates/planrun-parse/tests/spec_files.rs` checks every built-in spec against the schema.

## 3. Built-in recognizer kinds

Parameters in *italics* are optional.

| Kind | Matches | Produces | Parameters |
|---|---|---|---|
| `file-block` | a lead paragraph naming a path (and op), then one fence | `WriteFile` / `ApplyPatch` | `lead` (captures `path`, *`op`*), `ops` (op prefix → `write` \| `create` \| `overwrite` \| `patch`), *`default_op`*, *`file_name_from_manifest`* |
| `before-after` | lead (naming a path), fence, *separator paragraph*, fence | `ReplaceExact`, or `Manual(ProseEdit)` when `infer_path` finds no single entry | `lead` (captures `path`, or none with `infer_path = true`), *`separator`* (e.g. `^After:$`; absent = two fences back to back), *`file_name_from_manifest`*, *`series`* (a regex capturing `n`; the lead then captures `n` too, D49) |
| `delete` | a sentence or inline `git rm` | `Delete` | `lead` (captures `path`), *`via_git`*, *`file_name_from_manifest`* |
| `generated` | a "generated, not by hand" sentence | `Generated` | `lead` (captures `path`) |
| `ceremony` | a fence directly after a given action whose body matches | nothing (absorbed) | `after` (`patch`), `body` |
| `script` | a fence in a shell language | `RunScript` / `Commit` | *`intents`* (`[{ intent, body?, lead? }]`: `commit` \| `gate` \| `preview` \| `apply` \| `verify` \| `other`), *`leads`* (regexes), *`first_block`*, *`line_expect`* (a regex capturing `expect`, D35, D48) |
| `run-line` | `` Run: `cmd` `` in a paragraph, with an optional expectation in the same paragraph | `RunScript` / `Commit` (one line) | `lead` (captures `cmd`, *`expected`*), `shell` (`pwsh` \| `bash`), *`exit`* table over `expected` (field `text`), *`intents`* (`[{ intent, body }]`) |
| `expected-block` | `Expected (…):` paragraph + `text` fence after a script | amends the script | `lead` (captures `spec`), `exit` table, `lines = "summary"` |
| `expected-prose` | `Expected: …` sentence after a script; with `list`, an `Expected:` paragraph and its list | amends the script | `lead` (captures `text`, unless `list`), `exit` table (`success` \| `failure` \| `any` \| `code:N`), *`list`* (D47) |
| `manual-prose` | a paragraph (+ the fence after it, with `takes_fence`) that describes what planrun cannot do exactly | `Manual(reason)` | `lead`, `reason` (`prose-edit` \| `output-dependent` \| `human-check`), *`takes_fence`* |

Notes on the kinds:

- `file-block`: the longest `ops` prefix of the captured `op` wins; `write` takes Create or
  Overwrite from the manifest. Once the lead matched, an op no prefix covers is
  `Malformed(unknown-file-op)`, a lead without its fence `Malformed(missing-block)`, and for
  `patch` a patch whose paths differ from the lead's `Malformed(patch-path)`.
- Paths from the manifest (D32): `file_name_from_manifest = true` resolves a lead path without
  `/` that is not an entry itself to the one entry with that file name, when that entry was
  written in full (not a sibling, not Bare); otherwise the path stays as written. `before-after`
  with `infer_path = true` takes the task's one entry that is not Test, Generated or Delete; with
  none or several (a Bare entry counts, but is never taken, D45), the lead and both fences
  become one `Manual(ProseEdit)`. Pattern entries are never candidates (D41). `infer_path` is never
  available for writes. Each resolved action carries its `PathOrigin`; `show` marks it.
- `run-line`: the command is the `cmd` capture, run as one line in `shell` (D39). The `expected`
  capture, when the lead has one and it matched, becomes a prose expectation through the `exit`
  table (no row matches: `any`). An empty command is `Malformed(missing-block)`.
- `before-after`: once the lead matched, a missing fence or separator is
  `Malformed(missing-block)`.
- `script`: `intent = "commit"` selects the `Commit` action kind, not a `ScriptIntent`; a commit
  script carries `ScriptIntent::Other`. The other intents set the `ScriptIntent`; no match is
  `verify`. A row matches the fence body (`body`), the paragraph directly before the fence
  (`lead`), or both; at least one is required (D29). Preview/apply pairs are rows like these.
- `script` `line_expect` (D35, D48): every line of a claimed fence that is not a whole-line
  comment and matches gives the script a comment expectation, exit not checked; an
  `expected-prose` right after it extends it instead of being `expected-twice`.
- `before-after` `series` (D49): a lead without a path continues the numbered edit directly
  before it in the step (`PathOrigin::PreviousEdit`), before `infer_path` is tried.
- `expected-prose` `list` (D47): the list items at the lead's depth and the paragraphs, list
  items and `text` fences nested in them belong to the expectation; no list is
  `missing-block`.
- `script` guard (D28): with `leads` set, the kind claims a fence only when the paragraph
  directly before it matches one of `leads`, or, with `first_block = true`, when the fence is the
  first block of a written step (never the implicit first step, so never right after the
  manifest, and never in a step with a kept label, D46). A body intent never qualifies a fence
  on its own. Any other shell fence falls to the fallback, `Manual(Unrecognized)`. Without
  `leads` (the assay spec) every shell fence is a script.
- `expected-block` and `expected-prose` amend the script directly before them. Anything else there
  is `Malformed(expected-without-script)`; a second expectation on one script is
  `Malformed(expected-twice)`. `expected-block` needs a `text` fence (`missing-block`). An
  `expected-prose` sentence directly after a `Manual` joins that Manual's text instead (D38).
- Coverage counts the blocks of a recognizer's `Manual` (and of an expectation joined to a
  Manual) as manual, not as actions.
- **Exit tables are first match**: rows are tried in order and the first whose regex matches
  decides, so their order matters. Values: `success`, `failure`, `any` (`ExitExpect::Unknown`: not
  checked; prose is still confirmed, D11), `code:N`, and `code` (the row's regex captures `code`;
  0 is `success`). An `expected-block` spec no row matches is `Malformed(expected-spec)`; an
  `expected-prose` text no row matches is `any`.

The **fallback** (not configurable) runs when nothing matched: paragraph, list item, blockquote →
`Note` or `Manual(Marked)`; fence → `Manual(Unrecognized)` / `Manual(ForeignLanguage)`, a `text`
fence → `Note`; table, heading, rule → `Note`; HTML comment → dropped.

## 4. Adding an action kind

Rare (D4). The checklist, all in one change:

1. The variant in `planrun-model::ActionKind`, with docs.
2. Its handler in planrun-exec, its `DoneState` logic for resume, its dry-run behaviour.
3. Its policy class.
4. Its journal record.
5. Its `show` rendering and coverage label.
6. A corpus plan that produces it, and an invalid-corpus file for each new error.

## 5. Adding a recognizer kind

```rust
pub struct InsertAfterFactory;

impl RecognizerFactory for InsertAfterFactory {
    fn kind(&self) -> &'static str { "insert-after" }

    fn build(&self, params: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        // Read and validate every parameter here: a bad regex fails when the spec loads.
        Ok(Arc::new(InsertAfter { lead: regex(params, "lead")?, id: id(params)? }))
    }
}

struct InsertAfter { lead: Regex, id: String }

impl Recognizer for InsertAfter {
    fn id(&self) -> &str { &self.id }

    fn recognize(&self, cx: &StepContext<'_>, cursor: &mut Cursor<'_>) -> Recognized {
        // 1. Peek; return Recognized::No without advancing unless the lead matches.
        // 2. Once the lead matched, a missing or wrong block is Recognized::Malformed, not No.
        // 3. Advance past exactly what was consumed; return the actions.
        Recognized::No
    }
}
```

Register it in `Registry::with_builtins`. Rules:

- **Pure.** No I/O, no clock, no state between calls. Same input, same output.
- **`No` leaves the cursor where it was.** Advance only on `Yes`/`Amend`.
- **Commit to the idiom early.** Once the distinctive lead matched, failures are `Malformed`
  errors. Falling through to `Manual` would hide a broken plan.
- **Tests**: unit tests on hand-built blocks, plus a corpus plan in `tests/corpus/synthetic` and
  an invalid-corpus file per `Malformed` code.

## 6. Adding a Format by hand

Implement `Format` (`detect`, `outline`, `recognizers`, `settings`) and register it with
`Registry::add_format`. Reuse recognizer instances from the factories for the step contents: only
the outline should be new. Every block of every task must fall in exactly one step range.

## 7. Execution-side extension points

| Trait | Registry | For |
|---|---|---|
| `ActionHandler` | `HandlerSet` | how an action kind executes, previews, and reports `DoneState` |
| `ShellSession` | one per `Shell` | another shell (`cmd`, `zsh`): a new `Shell` variant + session |
| `CommandClassifier` | the runner's | different policy classification |
| `Ui` | the CLI | a TUI (ratatui) later, a non-interactive batch UI |
````

File `formats/before-after.toml` (full contents):

```toml
# The before-after format: "`p`, before: / After:" plans (docs/plan-grammar.md §5). Hand-written,
# mostly automatable: full-file writes, exact before/after edits, preview/apply pairs and the
# scripts behind run leads run; partial edits described in prose, output-dependent edits and
# foreign-language blocks are Manual. Every regex is matched against raw block text.

id = "before-after"
name = "Before/After plan"

[detect]
patterns = [
  { regex = '^## Task \d+: ', weight = 40 },
  { regex = '^- \[ \] \*\*Step \d+: ', weight = 40 },
  { regex = '^`[^`]+`, before:$', weight = 20 },
]

[outline]
# Tasks and the heading that ends them are both level 2 (D37).
task = { level = 2, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
tasks_end_level = 2

[outline.manifest]
marker = '^\*\*Files:\*\*$'
# - Modify: `a/b.cs:3-10`   - Rewrite: `a/x.razor`, `a/x.razor.cs`   - Modify: `a/p.cs:84-85` and `:241-243`
# - Modify: `a/r.resx`, `r.en.resx` (lines 227-230)   - Modify: `a/0001`–`0007_*.sql` (a range: one pattern entry, D41)
item = '^(?P<change>Create|Modify|Rewrite|Test): `(?P<paths>[^`]+(?:`(?:, | and |–)`[^`]+)*)`(?: \(.+\))?$'
changes = { Create = "create", Modify = "modify", Rewrite = "rewrite", Test = "test" }
path_list = '`(?:, | and )`'
required = false
siblings = true

[[outline.steps]]
block = "list-item"
regex = '^\*\*Step (?P<n>\d+): (?P<title>.+)\*\*$'

# A step title that says the step fails, or that the step happens elsewhere, marks the step (D30).
[[step_marks]]
title = '(?i)\bverify (?:it|they) fails?\b'
mark = "expected-failure"

[[step_marks]]
title = '(?i)(?:\bon (?:\S+ )?TST$|^Hand off\b|\bby eye\b)'
mark = "human-check"

[settings]
manual_markers = ['(?i)\bby hand\b', '(?i)\bRens runs\b', '(?i)\bmanual step\b']

# Scripts (`replace`) and prose edits change declared files, and items name ranges (D31).
[checks]
manifest-untouched = "off"
manifest-undeclared = "warning"

# --- Recognizers, tried in this order at each position of a step -------------------------------

[[recognizer]]
kind = "file-block"
id = "file-block#create"
# `p`:   Create `p` (only `*.sql` is embedded, …):
lead = '^(?:Create )?`(?P<path>[^`]+)`(?: \([^)]*\))?:$'
ops = {}
default_op = "create"

[[recognizer]]
kind = "file-block"
id = "file-block#replace-whole"
# `p` — replace the whole file:
lead = '^`(?P<path>[^`]+)` — replace the whole file:$'
ops = {}
default_op = "overwrite"

[[recognizer]]
kind = "file-block"
id = "file-block#replace"
# Replace `p`:
lead = '^Replace `(?P<path>[^`]+)`:$'
ops = {}
default_op = "overwrite"

[[recognizer]]
kind = "before-after"
id = "before-after#path"
# `p`, before: + fence + After: + fence
lead = '^`(?P<path>[^`]+)`, before:$'
separator = '^After:$'

[[recognizer]]
kind = "before-after"
id = "before-after#no-path"
# Before: / In `Member`, before: — the task's one editable entry, else Manual (D32).
lead = '^(?:In `[^`]+`, before|Before):$'
separator = '^After:$'
infer_path = true

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#output-dependent"
# … Replace the hashes … with the `Actual` values.
lead = '\bwith the `Actual` values?\b'
reason = "output-dependent"

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#edit"
# `p`, replace the class (keep …):   `p`, add these members to the class:   Add to the usings …:
lead = '^(?:`[^`]+`, (?:replace|add|insert|remove|change)\b.*|(?:Add|Append|Insert|Remove)\b.*):$'
reason = "prose-edit"
takes_fence = true

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#edit-then-verify"
# In the `X` row of the table near line 452, add … . Verify:   (the script after it runs)
lead = '^In\b.*\b(?:add|change|replace|remove)\b.*\. Verify:$'
reason = "prose-edit"

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#check"
# Verify: rerunning the preview shows 0 everywhere.
lead = '^Verify: \S'
reason = "human-check"

[[recognizer]]
kind = "script"
id = "script#before-after"
# Run only as the first block of a written step or behind a run lead (D28).
leads = [
  '^(?:Run|Preview|Apply|Verify)\b[^\n]*:$',
  '(?:^|\. )(?:Then|Verify):$',
  '^Expected: .*\bAlso confirm\b.*:$',
]
first_block = true
intents = [
  { intent = "commit", body = '(?m)\bgit commit\b' },
  { intent = "preview", lead = '^Preview\b' },
  { intent = "apply", lead = '^Apply:$' },
  { intent = "preview", body = '\s-DryRun\b' },
  { intent = "apply", body = '\s-All\b' },
]

[[recognizer]]
kind = "expected-prose"
id = "expected-prose#before-after"
lead = '^Expected: (?P<text>.+)$'
# First match wins: "no matches" is rg's exit 1.
exit = [
  { text = '(?i)\b(fails?|build errors?|does not exist)\b', exit = "failure" },
  { text = '(?i)\bno matches\b', exit = "code:1" },
  { text = '(?i)\b(pass(es)?|succeeds?)\b', exit = "success" },
]

[[recognizer]]
kind = "expected-prose"
id = "expected-prose#list"
# Expected (16):  + a list: one prose expectation over the paragraph and its list (D47).
lead = '^Expected(?: \([^)]*\))?:$'
list = true
```

**Step 2 — verify**

Targeted: the label and expectation tests:

```powershell
cargo test -p planrun-parse --test labels_and_expectations
```

Expected (exit code 0; summary lines):

```text
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-cli/src/view.rs crates/planrun-model/src/expect.rs crates/planrun-model/src/path.rs crates/planrun-parse/src/kinds/before_after.rs crates/planrun-parse/src/kinds/expected.rs crates/planrun-parse/src/kinds/run_line.rs crates/planrun-parse/src/kinds/script.rs crates/planrun-parse/src/outline.rs crates/planrun-parse/src/recognize.rs crates/planrun-parse/src/spec.rs crates/planrun-parse/tests/labels_and_expectations.rs docs/decisions.md docs/extending.md formats/before-after.toml
  $verified = [ordered]@{
      'crates/planrun-cli/src/view.rs' = '3880f27764fc95a57442707fa7991152257760d1'
      'crates/planrun-model/src/expect.rs' = 'd76f11a7955aae7530403755dcf1caca6a8b7cc4'
      'crates/planrun-model/src/path.rs' = 'ff4716d8d5a720723941577a1b10bf7ead72939f'
      'crates/planrun-parse/src/kinds/before_after.rs' = '9ebaa53885e026779e76e21b34df4a1ed046dfe1'
      'crates/planrun-parse/src/kinds/expected.rs' = '69d148f4dcb72b3791515a5990e84b70b78c55dd'
      'crates/planrun-parse/src/kinds/run_line.rs' = 'a65010100f2f3c8a847b0a12d35f33a2129d846c'
      'crates/planrun-parse/src/kinds/script.rs' = '696da67e28ec3bc6c4acd58a70a353179b8e006a'
      'crates/planrun-parse/src/outline.rs' = '1bd76e66806bada859946b899c9073f992825e72'
      'crates/planrun-parse/src/recognize.rs' = '2fb278b736780932b0fcc014389d1c65872cbc93'
      'crates/planrun-parse/src/spec.rs' = '91fd1f83dea74c2f1e6902285a97b27614d5fa9e'
      'crates/planrun-parse/tests/labels_and_expectations.rs' = '32937fb10e0097569c0dc49c93871629d5e29c94'
      'docs/decisions.md' = '2459eb32b8886308fd2d1604e7016b5ad5f0137b'
      'docs/extending.md' = 'cd71e1c1f9d04a3b1e03234c0acc2e9fe66d649d'
      'formats/before-after.toml' = '624c5d25747cc87425c0295d177bc615bf474255'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P2b-T3: kept step labels (D46), Expected lists (D47), comment expectations (D48), edit series (D49)" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P2b-T4: the devready format, its synthetic plan and IR snapshot

**Goal:** `formats/devready.toml`, registered as a built-in. The synthetic plan gains Task 3 (a Files list with the note after the first of several paths, `(replaces the whole file)`, `Old:`/`New:`, an `Expected:` list with nested prose and a nested shell fence that ends it, `Commit:` as a run line) and Task 4 (an inline manifest with a leftover, an edit series, a one-command `**Verify:**` line, `Commit:` + fence). Its IR snapshot is new.

**Files**

- Create `crates/planrun-cli/src/snapshots/planrun__view__tests__devready_style_ir.snap`
- Modify `crates/planrun-cli/src/view.rs`
- Modify `crates/planrun-parse/src/registry.rs`
- Create `formats/devready.toml`
- Modify `tests/corpus/synthetic/README.md`
- Modify `tests/corpus/synthetic/devready-style.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/src/view.rs`.

The new snapshot is the review item of this task: every devready rule shows in it (comment expectations, the Bare `README.md`, the per-path Rewrite, the leftover Note with origin `manifest`, `PreviousEdit`).

**Step 1 — write the files**

File `crates/planrun-cli/src/snapshots/planrun__view__tests__devready_style_ir.snap` (full contents):

```text
---
source: crates/planrun-cli/src/view.rs
expression: "Viewer::new(&plan).plan()"
---
plan: tests/corpus/synthetic/devready-style.md
format: devready
title: P2 — Greeter tooling Implementation Plan
phase: ~
preamble:
  - "**Don't push from this session.** Pushing is a manual step (M2)."
tasks:
  - id: Task 0
    title: Branch
    at: L25
    files: []
    steps:
      - step: intro
        at: L27
        actions:
          - at: L27
            by: fallback
            manual:
              reason: Unrecognized
              text: "```powershell\ngit switch main\ngit status --short            # empty\ngit switch -c feature/p2-tooling\n```"
      - step: Verify
        at: L33
        actions:
          - at: L33
            by: fallback
            note:
              text: "**Verify:**"
          - at: L34
            by: "script#devready"
            run:
              shell: pwsh
              intent: verify
              body: "git branch --show-current     # feature/p2-tooling\n"
              expect:
                exit: not checked
                lines: ~
                prose: ~
                comments: "git branch --show-current     # feature/p2-tooling"
          - at: L38
            by: fallback
            note:
              text: "---"
  - id: Task 1
    title: The helper and its config
    at: L40
    files:
      - Create tools/greet.ps1
      - Create tools/greet.json
    steps:
      - step: intro
        at: L46
        actions:
          - at: L46
            by: "file-block#devready"
            write-file:
              path: tools/greet.json
              mode: Create
              body: "{\n  \"punctuation\": \"!\",\n  \"names\": [\"Rens\"]\n}\n"
          - at: L54
            by: "file-block#devready"
            write-file:
              path: tools/greet.ps1
              mode: Create
              body: "# Greets every name in greet.json (P2 D1: a script, not a module).\n$config = Get-Content (Join-Path $PSScriptRoot 'greet.json') -Raw | ConvertFrom-Json\nforeach ($name in $config.names) { \"Hello, $name$($config.punctuation)\" }\n"
      - step: Verify
        at: L61
        actions:
          - at: L61
            by: fallback
            note:
              text: "**Verify:**"
          - at: L62
            by: "script#devready"
            run:
              shell: pwsh
              intent: verify
              body: "pwsh -NoProfile -File tools/greet.ps1   # Hello, Rens!\nGet-Content tools/greet.json -Raw | ConvertFrom-Json | Out-Null; 'json ok'\n"
              expect:
                exit: not checked
                lines: ~
                prose: ~
                comments: "pwsh -NoProfile -File tools/greet.ps1   # Hello, Rens!"
      - step: Commit
        at: L67
        actions:
          - at: L67
            by: "manual-prose#commit-message"
            manual:
              reason: HumanCheck
              text: "**Commit:** `Greet helper and config`"
          - at: L69
            by: fallback
            note:
              text: "---"
  - id: Task 2
    title: "Punctuation from the crate, drop the duplicate"
    at: L71
    files:
      - Modify tools/greet.ps1
      - "Modify README.md (a bare file name: its directory is unknown)"
      - Delete tools/greet.json
    steps:
      - step: 2a.
        at: L75
        actions:
          - at: L75
            by: "before-after#old-new"
            replace-exact:
              path: tools/greet.ps1
              before: "$config = Get-Content (Join-Path $PSScriptRoot 'greet.json') -Raw | ConvertFrom-Json\nforeach ($name in $config.names) { \"Hello, $name$($config.punctuation)\" }\n"
              after: "param([string[]]$Names = @('Rens'))\nforeach ($name in $Names) { \"Hello, $name!\" }\n"
      - step: 2b.
        at: L85
        actions:
          - at: L85
            by: "delete#git-rm"
            delete:
              path: tools/greet.json
              via_git: true
      - step: 2c.
        at: L87
        actions:
          - at: L87
            by: "before-after#old-new"
            replace-exact:
              path: README.md
              before: "cargo test\n```\n"
              after: "cargo test\npwsh -NoProfile -File tools/greet.ps1 -Names Ada,Linus\n```\n"
      - step: Verify
        at: L98
        actions:
          - at: L98
            by: fallback
            note:
              text: "**Verify:**"
          - at: L99
            by: "script#devready"
            run:
              shell: bash
              intent: verify
              body: "pwsh -NoProfile -File tools/greet.ps1 -Names Ada | tail -1   # Hello, Ada!\ntest ! -e tools/greet.json && echo gone\n"
              expect:
                exit: not checked
                lines: ~
                prose: ~
                comments: "pwsh -NoProfile -File tools/greet.ps1 -Names Ada | tail -1   # Hello, Ada!"
          - at: L104
            by: fallback
            manual:
              reason: Marked
              text: "Check by eye: the README example renders as one code block."
      - step: Commit
        at: L106
        actions:
          - at: L106
            by: "manual-prose#commit-message"
            manual:
              reason: HumanCheck
              text: "**Commit:** `Greet helper: names as parameters`"
          - at: L108
            by: fallback
            note:
              text: "---"
  - id: Task 3
    title: Names from a library
    at: L110
    files:
      - Rewrite tools/greet.ps1
      - Create tools/lib/names.ps1
      - Modify tools/lib/format.ps1
    steps:
      - step: 3a.
        at: L116
        actions:
          - at: L116
            by: "file-block#devready"
            write-file:
              path: tools/greet.ps1
              mode: Overwrite
              body: ". (Join-Path $PSScriptRoot 'lib/names.ps1')\nforeach ($name in (Get-GreetNames)) { \"Hello, $name!\" }\n"
      - step: 3b.
        at: L122
        actions:
          - at: L122
            by: "file-block#devready"
            write-file:
              path: tools/lib/names.ps1
              mode: Create
              body: "function Get-GreetNames { @('Ada', 'Linus') }\n"
      - step: 3c.
        at: L127
        actions:
          - at: L127
            by: "before-after#old-path"
            replace-exact:
              path: tools/lib/format.ps1
              before: "'{0}!'\n"
              after: "'{0}.'\n"
      - step: Verify
        at: L136
        actions:
          - at: L136
            by: fallback
            note:
              text: "**Verify** (Bash tool, the shell CI uses):"
          - at: L137
            by: "script#devready"
            run:
              shell: bash
              intent: verify
              body: "pwsh -NoProfile -File tools/greet.ps1 | head -1   # Hello, Ada!\n"
              expect:
                exit: not checked
                lines: ~
                prose: "Expected:\n- two greetings, one per name;\n\n  the order follows `Get-GreetNames`.\n- nothing on stderr:"
                comments: "pwsh -NoProfile -File tools/greet.ps1 | head -1   # Hello, Ada!"
          - at: L146
            by: fallback
            manual:
              reason: Unrecognized
              text: "```bash\n  pwsh -NoProfile -File tools/greet.ps1 2>&1 >/dev/null | wc -l\n  ```"
      - step: Commit
        at: L150
        actions:
          - at: L150
            by: "run-line#commit"
            commit:
              shell: pwsh
              intent: other
              body: "git add tools/greet.ps1 tools/lib/names.ps1 tools/lib/format.ps1; git commit -m \"Greet: names from a library\""
              expect: ~
          - at: L152
            by: fallback
            note:
              text: "---"
  - id: Task 4
    title: Punctuation as a parameter
    at: L154
    files:
      - Modify tools/greet.ps1
    steps:
      - step: intro
        at: L158
        actions:
          - at: L156
            by: manifest
            manual:
              reason: Marked
              text: regenerate the README examples by hand
          - at: L158
            by: "before-after#old-new"
            path_from: "resolved from manifest: the manifest entry with this file name"
            replace-exact:
              path: tools/greet.ps1
              before: ". (Join-Path $PSScriptRoot 'lib/names.ps1')\n"
              after: "param([string]$Punctuation = '!')\n. (Join-Path $PSScriptRoot 'lib/names.ps1')\n"
          - at: L167
            by: "before-after#series"
            path_from: "resolved from manifest: the edit before it in the same numbered series"
            replace-exact:
              path: tools/greet.ps1
              before: "\"Hello, $name!\"\n"
              after: "\"Hello, $name$Punctuation\"\n"
      - step: Verify
        at: L175
        actions:
          - at: L175
            by: "run-line#verify"
            run:
              shell: pwsh
              intent: verify
              body: pwsh -NoProfile -File tools/greet.ps1 -Punctuation ?
              expect:
                exit: exit 0
                lines: ~
                prose: all pass (one line per name).
      - step: Commit
        at: L177
        actions:
          - at: L177
            by: fallback
            note:
              text: "Commit:"
          - at: L178
            by: "script#devready"
            commit:
              shell: pwsh
              intent: other
              body: "git add tools/greet.ps1\ngit commit -m \"Greet: punctuation as a parameter\"\n"
              expect: ~
          - at: L183
            by: fallback
            note:
              text: "---"
after_tasks: L185
```

File `crates/planrun-cli/src/view.rs` (exact diff; save the block as `$env:TEMP\p2b-t4-view-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/view.rs b/crates/planrun-cli/src/view.rs
index 3880f27..fa8a528 100644
--- a/crates/planrun-cli/src/view.rs
+++ b/crates/planrun-cli/src/view.rs
@@ -505,6 +505,13 @@ mod tests {
         insta::assert_yaml_snapshot!("before_after_style_ir", Viewer::new(&plan).plan());
     }
 
+    #[test]
+    fn the_ir_of_the_synthetic_devready_plan() {
+        let plan = synthetic_plan("devready-style.md");
+        assert_eq!(&*plan.format.0, "devready");
+        insta::assert_yaml_snapshot!("devready_style_ir", Viewer::new(&plan).plan());
+    }
+
     #[test]
     fn show_prints_one_line_per_action() {
         let plan = synthetic();
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2b-t4-view-rs.patch"
Remove-Item "$env:TEMP\p2b-t4-view-rs.patch"
```

File `crates/planrun-parse/src/registry.rs` (full contents):

```rust
//! The registry: recognizer factories by kind, and the formats built from specs with them.
//!
//! Composition root of parsing. Built-ins register in [`Registry::with_builtins`]; user formats
//! are added from TOML (`.planrun/formats/*.toml` in the repository, then the user config
//! directory) with [`Registry::add_spec`]. A spec that names an unknown recognizer kind, has an
//! invalid regex or duplicates a format id fails here, before any plan is read.

use std::collections::HashMap;
use std::sync::Arc;

use crate::diagnostic::Diagnostic;
use crate::document::Document;
use crate::format::Format;
use crate::kinds;
use crate::recognizer::RecognizerFactory;
use crate::spec::{FormatSpec, SpecFormat};

/// Recognizer factories and formats.
pub struct Registry {
    factories: HashMap<&'static str, Arc<dyn RecognizerFactory>>,
    formats: Vec<Arc<dyn Format>>,
    /// Detection scores below this never select a format.
    pub min_score: u8,
}

/// Why a factory, spec or format was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegistryError {
    /// Two factories with one kind.
    #[error("recognizer kind `{0}` is registered twice")]
    DuplicateKind(String),
    /// Two formats with one id.
    #[error("format `{0}` is registered twice")]
    DuplicateFormat(String),
    /// A spec names a kind no factory provides.
    #[error("format `{format}`: unknown recognizer kind `{kind}`")]
    UnknownKind {
        /// The format.
        format: String,
        /// The kind.
        kind: String,
    },
    /// A spec does not parse or a factory refused its parameters.
    #[error("format `{format}`: {message}")]
    InvalidSpec {
        /// The format (or the file name when the id is unreadable).
        format: String,
        /// What is wrong.
        message: String,
    },
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// An empty registry with the default threshold (50).
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            formats: Vec::new(),
            min_score: 50,
        }
    }

    /// The built-in recognizer kinds and formats (Phase 1: the `assay` format and the kinds it
    /// uses; Phase 2: the hand-written ones).
    pub fn with_builtins() -> Result<Self, RegistryError> {
        let mut registry = Self::new();
        for factory in kinds::builtin_factories() {
            registry.add_factory(factory)?;
        }
        registry.add_spec("assay", include_str!("../../../formats/assay.toml"))?;
        registry.add_spec(
            "superpowers",
            include_str!("../../../formats/superpowers.toml"),
        )?;
        registry.add_spec(
            "before-after",
            include_str!("../../../formats/before-after.toml"),
        )?;
        registry.add_spec("devready", include_str!("../../../formats/devready.toml"))?;
        Ok(registry)
    }

    /// Registers a recognizer factory.
    pub fn add_factory(
        &mut self,
        factory: Arc<dyn RecognizerFactory>,
    ) -> Result<(), RegistryError> {
        let kind = factory.kind();
        if self.factories.insert(kind, factory).is_some() {
            return Err(RegistryError::DuplicateKind(kind.to_owned()));
        }
        Ok(())
    }

    /// Registers a hand-written format.
    pub fn add_format(&mut self, format: Arc<dyn Format>) -> Result<(), RegistryError> {
        if self.formats.iter().any(|f| f.id() == format.id()) {
            return Err(RegistryError::DuplicateFormat(format.id().0.to_string()));
        }
        self.formats.push(format);
        Ok(())
    }

    /// Builds a format from TOML spec text and registers it. `origin` is the spec's file stem,
    /// which must equal its `id` (docs/extending.md §2).
    pub fn add_spec(&mut self, origin: &str, toml_text: &str) -> Result<(), RegistryError> {
        let invalid = |message: String| RegistryError::InvalidSpec {
            format: origin.to_owned(),
            message,
        };
        let spec: FormatSpec = toml::from_str(toml_text).map_err(|e| invalid(e.to_string()))?;
        if spec.id != origin {
            return Err(invalid(format!(
                "the id `{}` is not the file name `{origin}`",
                spec.id
            )));
        }
        let unknown = spec
            .recognizers
            .iter()
            .filter_map(|t| t.get("kind").and_then(|k| k.as_str()))
            .find(|kind| !self.factories.contains_key(kind));
        if let Some(kind) = unknown {
            return Err(RegistryError::UnknownKind {
                format: origin.to_owned(),
                kind: kind.to_owned(),
            });
        }
        let format =
            SpecFormat::compile(spec, |kind| self.factories.get(kind).cloned()).map_err(invalid)?;
        self.add_format(Arc::new(format))
    }

    /// The factory for `kind`.
    pub fn factory(&self, kind: &str) -> Option<&Arc<dyn RecognizerFactory>> {
        self.factories.get(kind)
    }

    /// All formats, in registration order.
    pub fn formats(&self) -> &[Arc<dyn Format>] {
        &self.formats
    }

    /// The format for `doc`: the one named by `forced` (`unknown-format` when none has that
    /// id), else the single highest detection score at or above [`Registry::min_score`]
    /// (`no-format` when none reaches it, `ambiguous-format` on a tie). Never a guess.
    pub fn select(
        &self,
        doc: &Document,
        forced: Option<&str>,
    ) -> Result<Arc<dyn Format>, Diagnostic> {
        if let Some(id) = forced {
            return self
                .formats
                .iter()
                .find(|f| &*f.id().0 == id)
                .cloned()
                .ok_or_else(|| {
                    Diagnostic::error("unknown-format", None, format!("no format has id `{id}`"))
                });
        }
        let mut scored: Vec<(u8, &Arc<dyn Format>)> = self
            .formats
            .iter()
            .map(|f| (f.detect(doc), f))
            .filter(|(score, _)| *score >= self.min_score)
            .collect();
        scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
        match scored.as_slice() {
            [] => Err(Diagnostic::error(
                "no-format",
                None,
                format!(
                    "no format recognizes this plan (no detection score reaches {}); name one \
                     with --format",
                    self.min_score
                ),
            )),
            [(a, first), (b, second), ..] if a == b => Err(Diagnostic::error(
                "ambiguous-format",
                None,
                format!(
                    "formats `{}` and `{}` both score {a}; name one with --format",
                    first.id().0,
                    second.id().0
                ),
            )),
            [(_, first), ..] => Ok(Arc::clone(first)),
        }
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
```

File `formats/devready.toml` (full contents):

```toml
# The devready format: DevReady plans written for an agent (docs/plan-grammar.md §4). Hand-written,
# partly automatable (D40): file writes, exact old/new edits, deletes, and the scripts behind an
# allowlisted run lead run; probes, conditional branches, prose edits and checks by eye are Manual
# or Notes. Every regex is matched against raw block text (inline markup kept), anchored per block.

id = "devready"
name = "DevReady plan"

[detect]
patterns = [
  { regex = '^### Task \d+: ', weight = 30 },
  { regex = '^\*\*Repo / branch:\*\* ', weight = 20 },
  { regex = '^\*\*\d+[a-z]\.', weight = 30 },
  { regex = '^\*\*Verify:\*\*', weight = 20 },
  { regex = '^## Manual steps\b', weight = 20 },
]

[outline]
task = { level = 3, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
# `## Acceptance …` and `## Manual steps …` end the tasks.
tasks_end_level = 2

[outline.manifest]
# **Files:** create + a list of bare paths; **Files:** + a list of `Change: paths` items; or the
# manifest inline: **Files:** create `a`; modify `b`. It generates … (D43)
marker = '^\*\*Files:\*\*(?: (?P<change>create)$| (?P<items>(?:[^`.\n]|`[^`]*`)+)|$)'
# - `p`   - Modify (note): `a`, `b`   - Create (Write tool): `a` (full replacement), `b`   - Delete: `p` (`git rm`)
# inline clauses: create `a`, `b`   modify `a` and `b`   Modify `a` (note) and `b`
item = '^(?:(?P<change>[Cc]reate|[Mm]odify|[Dd]elete|Generated)(?: \([^)]*\))?:? )?`(?P<paths>[^`]+(?:`(?: \([^)]*\))?(?:, | and )`[^`]+)*)`(?: \(.+\))?$'
changes = { create = "create", Create = "create", modify = "modify", Modify = "modify", delete = "delete", Delete = "delete", Generated = "generated", "full replacement" = "rewrite" }
path_list = '`(?: \([^)]*\))?(?:, | and )`'
clause_list = '; '
# `a` (full replacement), `b`: only `a` exists (D44).
path_change = '^` \((?P<change>full replacement)\)'
required = false
# `README.md` after `src/x/a.rs` is a root file in these plans as often as a sibling: Bare (D45).
siblings = false

# Sub-labels split a task into steps and stay in them, so their leads reach the recognizers (D46).
[[outline.steps]]
block = "paragraph"
# **2a.**   **1a. title** (Bash tool):   **1a–1d. title**   **4l. Generate and verify.**
regex = '^\*\*(?P<title>\d+[a-z](?:–\d+[a-z])?\.[^*]*)\*\*'
keep = true

[[outline.steps]]
block = "paragraph"
# **Verify:**   **Verify** (…):   **Run + verify:**   **Commit:** `msg`
regex = '^\*\*(?P<title>Verify|Run \+ verify|Commit)(?::\*\*|\*\*)'
keep = true

[[outline.steps]]
block = "paragraph"
# Commit:   Commit: `git add …; git commit -m "…"`
regex = '^(?P<title>Commit):(?: |$)'
keep = true

[settings]
manual_markers = [
  '(?i)\bby hand\b',
  '(?i)\bRens runs\b',
  '(?i)\bmanual step\b',
  '\bVERIFY\b',
  # Record: …   Record the timings   …, record which form works   (not "the record's message")
  '(?i)(?:^|[.;:,] |\*\* )record\b',
  '(?i)\bstop and report\b',
  '(?i)^check by eye\b',
  '(?i)^negative check\b',
  "(?i)\\bdon't\\b",
]

# Prose edits and scripts change declared files (D31).
[checks]
manifest-untouched = "off"
manifest-undeclared = "warning"

# --- Recognizers, tried in this order at each position of a step -------------------------------

[[recognizer]]
kind = "file-block"
id = "file-block#devready"
# `p`:   **4b.** `p` (note):   **2f.** Create `p` (note):   **4a.** `p` (replaces the whole file):
lead = '^(?:\*\*\d+[a-z]\.\*\* )?(?:Create )?`(?P<path>[^`]+)`(?: \((?P<op>replaces the whole file)\)| \([^)]*\))?:$'
ops = { "replaces the whole file" = "overwrite" }
default_op = "create"

[[recognizer]]
kind = "before-after"
id = "before-after#old-new"
# `p`, edit 1 (old → new):   NOTES.md edit 1 (old → new):   **2c.** `p` edit (old → new; note):
lead = '^(?:\*\*\d+[a-z]\.\*\* )?`?(?P<path>[\w./-]*\w\.\w+)`?,? edit(?: (?P<n>\d+))? \(old → new(?:; [^)]*)?\):$'
file_name_from_manifest = true

[[recognizer]]
kind = "before-after"
id = "before-after#series"
# Edit 2 (old → new): after `p`, edit 1 (D49); else the task's one editable entry (D32).
lead = '^(?:\w+ )?[Ee]dit (?P<n>\d+) \(old → new(?:; [^)]*)?\):$'
series = '(?i)\bedit (?P<n>\d+) \(old → new'
infer_path = true

[[recognizer]]
kind = "before-after"
id = "before-after#old-path"
# **2c.** `p`. Old:   `p`, three edits. Old:   README.md. Old:   **2d.** `p`: insert … Old (end of file):  + fence + New: + fence
lead = '^(?:\*\*\d+[a-z]\.\*\* )?`?(?P<path>[\w./-]*\w\.\w+)`?(?:[ .:,][^\n]*)?\bOld(?: \([^)]*\))?:$'
separator = '^New:$'
file_name_from_manifest = true

[[recognizer]]
kind = "before-after"
id = "before-after#old"
# Old:   (2) Old:   In `Member`. Old:   (3) In the `$x` hashtable. Old:  — the task's one editable entry
lead = '^(?:[^\n]*[.)] )?Old(?: \([^)]*\))?:$'
separator = '^New:$'
infer_path = true

[[recognizer]]
kind = "delete"
id = "delete#git-rm"
# `git rm p`   **4g.** `git rm p`
lead = '^(?:\*\*\d+[a-z]\.\*\* )?`git rm (?P<path>[^`\s]+)`$'
via_git = true

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#apply-in-prose"
# Then the same command with `-All` instead of `-DryRun`. Verify:  + its fence: the apply is
# described, not written, and the fence relies on the preview's variables.
lead = '^Then the same command with\b[^\n]*:$'
reason = "human-check"
takes_fence = true

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#run-from"
# … (run from `$REPO`):  + its fence: a script has no working directory of its own (D24).
lead = '\(run from `[^`]+`\):$'
reason = "human-check"
takes_fence = true

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#inline-edit"
# Old: `a`  /  New: `b`   Edit: `a` → `b`   Old (end of …): `x` … New: insert after `x`:  (D34)
lead = '(?m)(?:\bOld(?: \([^)]*\))?: `|^Edit: `[^`]+` → `)'
reason = "prose-edit"
takes_fence = true

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#new"
# New:  + fence, after an inline Old: `…`
lead = '^New:$'
reason = "prose-edit"
takes_fence = true

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#insert"
# append after the closing `}` of `X`:   Insert directly before `function X {`:   And insert before …:
lead = '(?i)\b(?:insert|append)\b[^\n]*:$'
reason = "prose-edit"
takes_fence = true

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#verify-several"
# **Verify:** `a` → x; `b` → y  (two commands: checked by you)
lead = '^\*\*Verify:\*\* `[^`]+` →[^\n]*`[^`]+` →'
reason = "human-check"

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#commit-message"
# **Commit:** `message`  (D33: planrun synthesizes no commit)
lead = '^\*\*Commit:\*\* `[^`]+`$'
reason = "human-check"

[[recognizer]]
kind = "run-line"
id = "run-line#verify"
# **Verify:** `cmd` → all pass (…)
lead = '^\*\*Verify:\*\* `(?P<cmd>[^`]+)` → (?P<expected>[^→]+)$'
shell = "pwsh"
exit = [
  { text = '(?i)\b(?:all pass|0 failed|exit 0)\b', exit = "success" },
]

[[recognizer]]
kind = "run-line"
id = "run-line#commit"
# Commit: `git add a; git commit -m "…"`  (D22 gates it)
lead = '^Commit: `(?P<cmd>[^`]+)`$'
shell = "pwsh"
intents = [{ intent = "commit", body = '\bgit commit\b' }]

[[recognizer]]
kind = "script"
id = "script#devready"
# Only behind an allowlisted run lead (D28, D46): never behind a bare sub-label.
leads = [
  '^\*\*Verify(?::\*\*|\*\* \([^)]*\):)$',
  '^\*\*Run \+ verify:\*\*$',
  '^(?:\*\*Commit:\*\*|Commit:)$',
  '^\*\*\d+[a-z]\. Generate and verify\.\*\*$',
  '\(Bash tool\):$',
]
# `git branch --show-current   # feature/x`: a comment expectation, always confirmed (D35, D48).
line_expect = '\s#\s(?P<expect>\S.*)$'
intents = [
  { intent = "commit", body = '(?m)\bgit commit\b' },
  { intent = "preview", body = '\s-DryRun\b' },
  { intent = "apply", body = '\s-All\b' },
]

[[recognizer]]
kind = "expected-prose"
id = "expected-prose#devready"
lead = '^Expected: (?P<text>.+)$'
exit = [
  { text = '(?i)\b(?:fails?|build errors?)\b', exit = "failure" },
  { text = '(?i)\b(?:all pass(?:es)?|0 failed|exit 0)\b', exit = "success" },
]

[[recognizer]]
kind = "expected-prose"
id = "expected-prose#devready-list"
# Expected:  + a list (D47). `Expected and recorded:` is no list lead.
lead = '^Expected(?: \([^)]*\))?:$'
list = true
exit = [
  { text = '(?i)\b(?:fails?|build errors?)\b', exit = "failure" },
]
```

File `tests/corpus/synthetic/README.md` (full contents):

```markdown
# Synthetic corpus

One plan per known format, small, written to exercise each format's idioms. They describe a tiny
crate (`tests/fixtures/greeter`) so the examples read naturally, but only `assay-style.md` is
built to execute: its diffs were generated by git from the fixture, so the `index` lines carry
real blob ids and the P1-T1 → P1-T2 blob chain (`aa80cf1 → 97bd92a → 6b269fc`) holds.

| File | Format | Exercises |
|---|---|---|
| `assay-style.md` | `assay` | `## Tasks`, `### P1-T1:`, `**Files**`, `**Step n — …**`, full contents, exact diff + absorbed ceremony, `Delete`, generated file, `Expected (…):` blocks, gate + gated commit, text after the tasks |
| `superpowers-style.md` | `superpowers` | `### Task n:`, `**Files:**` with `Create:`/`Modify: …:3-4`/`Test:`, `- [ ] **Step n: …**`, expected failure, `Run:`/`Expected:` prose, prose edits (Manual), check by eye, bash commit |
| `devready-style.md` | `devready` | `Task 0`, inline Files clauses (a leftover, a bare name), a Files list with a per-path `(full replacement)`, `` `path`: `` + block creates, `(replaces the whole file)`, `**Verify:**` with `# expected` comments, `Expected:` + a list (nested prose, a nested shell fence that ends it), `**Commit:** \`msg\``, `Commit:` + a run line or a fence, kept `**2a.**` labels, `edit 1 (old → new)` and its series, `Old:`/`New:`, four-backtick fences, `git rm`, bash + powershell in one task, conventions with a manual marker |
| `before-after-style.md` | `before-after` | `## Task n:`, file map table, `Rewrite:`, `` `path`: `` create, `— replace the whole file:`, `before:`/`After:`, expected failure (prose), prose insertion (Manual), `no matches`, preview/apply pair, output-dependent edit, `sql` block |

Golden snapshots of the parsed IR live next to the tests that produce them (`insta`, in
`crates/planrun-cli/src/snapshots/`): P1 for `assay-style.md`, P2a for `superpowers-style.md`
and `before-after-style.md`, P2b for `devready-style.md`. A snapshot change is a review item,
never a re-pin by reflex.
```

File `tests/corpus/synthetic/devready-style.md` (full contents):

`````markdown
# P2 — Greeter tooling Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Phase:** P2 · **Spec:** docs/design/greeter.md v0.2 (§3, §4)

**Goal:**
1. A `greet` PowerShell helper next to the crate.
2. A config file the helper reads.

**Repo / branch:** greeter (`$GR`) → `feature/p2-tooling`, from `main`.

**Decisions taken in this plan (Rens: review):**
- **D1 The helper is a script, not a module.** One file, dot-sourced.
- **D2 Config is JSON.** ConvertFrom-Json is built in.

## Conventions for this plan
- Blocks marked `powershell` are PowerShell 7. Blocks marked `bash` run with the Bash tool as-is.
- Every block starts from the `$GR` root.
- **Don't push from this session.** Pushing is a manual step (M2).
- Commit at the end of each task that says so.

---

### Task 0: Branch

```powershell
git switch main
git status --short            # empty
git switch -c feature/p2-tooling
```

**Verify:**
```powershell
git branch --show-current     # feature/p2-tooling
```

---

### Task 1: The helper and its config

**Files:** create
- `tools/greet.ps1`
- `tools/greet.json`

`tools/greet.json`:
```json
{
  "punctuation": "!",
  "names": ["Rens"]
}
```

`tools/greet.ps1`:
```powershell
# Greets every name in greet.json (P2 D1: a script, not a module).
$config = Get-Content (Join-Path $PSScriptRoot 'greet.json') -Raw | ConvertFrom-Json
foreach ($name in $config.names) { "Hello, $name$($config.punctuation)" }
```

**Verify:**
```powershell
pwsh -NoProfile -File tools/greet.ps1   # Hello, Rens!
Get-Content tools/greet.json -Raw | ConvertFrom-Json | Out-Null; 'json ok'
```

**Commit:** `Greet helper and config`

---

### Task 2: Punctuation from the crate, drop the duplicate

**Files:** modify `tools/greet.ps1`, `README.md`; delete `tools/greet.json`.

**2a.** `tools/greet.ps1`, edit 1 (old → new):
```powershell
$config = Get-Content (Join-Path $PSScriptRoot 'greet.json') -Raw | ConvertFrom-Json
foreach ($name in $config.names) { "Hello, $name$($config.punctuation)" }
```
```powershell
param([string[]]$Names = @('Rens'))
foreach ($name in $Names) { "Hello, $name!" }
```

**2b.** `git rm tools/greet.json`

**2c.** `README.md` edit (old → new; the old text includes the closing fence of the Examples block):
````markdown
cargo test
```
````
````markdown
cargo test
pwsh -NoProfile -File tools/greet.ps1 -Names Ada,Linus
```
````

**Verify:**
```bash
pwsh -NoProfile -File tools/greet.ps1 -Names Ada | tail -1   # Hello, Ada!
test ! -e tools/greet.json && echo gone
```

Check by eye: the README example renders as one code block.

**Commit:** `Greet helper: names as parameters`

---

### Task 3: Names from a library

**Files:**
- Create (Write tool): `tools/greet.ps1` (full replacement), `tools/lib/names.ps1`
- Modify: `tools/lib/format.ps1`

**3a.** `tools/greet.ps1` (replaces the whole file):
```powershell
. (Join-Path $PSScriptRoot 'lib/names.ps1')
foreach ($name in (Get-GreetNames)) { "Hello, $name!" }
```

**3b.** `tools/lib/names.ps1`:
```powershell
function Get-GreetNames { @('Ada', 'Linus') }
```

**3c.** `tools/lib/format.ps1`. Old:
```powershell
'{0}!'
```
New:
```powershell
'{0}.'
```

**Verify** (Bash tool, the shell CI uses):
```bash
pwsh -NoProfile -File tools/greet.ps1 | head -1   # Hello, Ada!
```
Expected:
- two greetings, one per name;

  the order follows `Get-GreetNames`.
- nothing on stderr:

  ```bash
  pwsh -NoProfile -File tools/greet.ps1 2>&1 >/dev/null | wc -l
  ```

Commit: `git add tools/greet.ps1 tools/lib/names.ps1 tools/lib/format.ps1; git commit -m "Greet: names from a library"`

---

### Task 4: Punctuation as a parameter

**Files:** modify `tools/greet.ps1`; regenerate the README examples by hand.

`greet.ps1`, edit 1 (old → new):
```powershell
. (Join-Path $PSScriptRoot 'lib/names.ps1')
```
```powershell
param([string]$Punctuation = '!')
. (Join-Path $PSScriptRoot 'lib/names.ps1')
```

Edit 2 (old → new):
```powershell
"Hello, $name!"
```
```powershell
"Hello, $name$Punctuation"
```

**Verify:** `pwsh -NoProfile -File tools/greet.ps1 -Punctuation ?` → all pass (one line per name).

Commit:
```powershell
git add tools/greet.ps1
git commit -m "Greet: punctuation as a parameter"
```

---

## Manual steps for Rens

### M1 Before execution
- Review D1–D2.

### M2 After Claude Code finishes
- Push and squash-merge.
`````

**Step 2 — verify**

Targeted: the CLI crate (the new IR snapshot):

```powershell
cargo test -p planrun-cli
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-cli/src/snapshots/planrun__view__tests__devready_style_ir.snap crates/planrun-cli/src/view.rs crates/planrun-parse/src/registry.rs formats/devready.toml tests/corpus/synthetic/README.md tests/corpus/synthetic/devready-style.md
  $verified = [ordered]@{
      'crates/planrun-cli/src/snapshots/planrun__view__tests__devready_style_ir.snap' = '77c8b8ffaa9104f59b4bbfee14beb553c93fb00f'
      'crates/planrun-cli/src/view.rs' = 'fa8a5287d4540b1e2bf81ab43d41de08b8931b77'
      'crates/planrun-parse/src/registry.rs' = 'bb5ef88e4cca57cc84fbea3f02889f5976788f7b'
      'formats/devready.toml' = '03a4ff0fd7f1ab1a833805b9dae6156d76c10aac'
      'tests/corpus/synthetic/README.md' = '83c7ba11166cc44b9f5cc3f2cdb2ed19d39e2869'
      'tests/corpus/synthetic/devready-style.md' = '337080d6ba6a0ee8c5cc8a7569643ede87aacfe3'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P2b-T4: the devready format, its synthetic plan and IR snapshot" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P2b-T5: plan-grammar devready section, the Apply invariant, roadmap and CLAUDE.md

**Goal:** Docs: plan-grammar §4 rewritten from the format (the D33 commit row, the new markers, steps from kept labels); §5 states the Apply invariant (open item 4); the roadmap's P2b paragraph and the P4 candidate "script working directory from a run lead"; CLAUDE.md's rule for rulings relayed from a review seat.

**Files**

- Modify `CLAUDE.md`
- Modify `docs/plan-grammar.md`
- Modify `docs/roadmap.md`

**Step 1 — write the files**

File `CLAUDE.md` (full contents):

````markdown
# planrun

A Rust CLI that executes structured Markdown implementation plans against a git repository, task by
task, step by step: exactly what the plan says, nothing more. It is not an agent. Read
`docs/architecture.md` before changing code; `docs/decisions.md` holds the rules below with their
reasons (cited as `D<n>`).

## Priorities (D1, D25)

Correctness > maintainability/extensibility > execution speed > low memory > predictability.
When two of these conflict, the earlier one wins. Speed means no wasted process starts or repeated
reads; don't optimize I/O speculatively (cargo/npm dominate wall time).

## Layout

```
crates/planrun-model   the IR: Plan, Task, Step, Action, Span/TextRef, RepoPath, PathOrigin,
                       ManifestPath/PathPattern,
                       AbbrevBlobId
crates/planrun-parse   Markdown → IR: source, document, format, spec, registry, outline,
                       manifest, recognizer, recognize, kinds/ (one file per recognizer kind),
                       patch, check, diagnostic
crates/planrun-exec    IR → effects: workspace, git, shells, expect, policy, journal, runner
crates/planrun-cli     the `planrun` binary: check, show (view, render)
formats/*.toml         built-in plan formats (embedded): assay, superpowers, before-after; the
                       file name is the format id
docs/                  architecture, plan-grammar, extending, decisions, roadmap, plans/,
                       handovers/ (one per finished phase)
tests/corpus/          synthetic/ (one plan per format), invalid/ (one error each), local/ (private)
tests/fixtures/greeter the repo the synthetic assay plan applies to
```

## Hard rules

1. **planrun-parse never executes and never touches the repository. planrun-exec never depends on
   planrun-parse** (D2). Don't add a dependency that crosses these lines.
2. **No partial plans** (D3). Any error means no `Plan`, nothing runs.
3. **Nothing is dropped.** Every block of a task becomes an action, `Note` or `Manual` (D21). Once
   a recognizer's distinctive lead matched, a broken idiom is a `Malformed` error, never a
   fallback.
4. **The action set is closed** (D4). New plan wording maps onto existing actions. Adding an
   `ActionKind` follows docs/extending.md §4, all in one change, and only with Rens' agreement.
5. **New plan styles are data first** (D8): a TOML spec, then a recognizer kind, then a hand-written
   `Format`, in that order of preference (docs/extending.md).
6. **Paths in actions are `RepoPath`** (D7). Nothing outside the repository is written except
   `.git/planrun/` and the OS temp directory. The working tree gets only what the plan writes.
7. **Patches go through `git apply`** with exact context (D20). No fuzz, no `similar`-based
   patching.
8. **Prose never passes a script on its own** (D11); expected failures are skipped by default (D12).
9. **No new dependency without a line in docs/decisions.md D9.**
10. **Real plans stay in `tests/corpus/local/`** (git-ignored, D16). Never copy their content into
    committed files, tests, snapshots, docs or commit messages. Synthetic plans only.

## Commands

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test -- --ignored local_corpus      # parses the private real plans, when present
cargo run -p planrun-cli -- check tests/corpus/synthetic/assay-style.md
```

The gate for every task is all three of fmt, clippy (warnings are errors) and test.

## Code conventions

- Library code: no `unwrap`/`expect` (clippy warns; the gate fails). Tests may use
  `unwrap_or_else(|e| panic!("{e}"))`.
- Every public item has a doc comment (`missing_docs` warns).
- Diagnostics have a stable kebab-case `code`; tests pin codes, snapshots pin messages.
- Text from the plan is a `TextRef`, never a copied `String`, except `TextRef::Owned` where the
  source isn't contiguous.
- Prefer enums over strings, traits for extension points, composition over deep generic layering.
- Keep functions flat: guard clauses and early returns, at most three levels of nesting.
- Recognizers are pure; parameters are validated when a spec loads, not when a plan is parsed.

## Tests

- Unit tests sit next to the code (`mod tests`), or in `<module>_tests.rs` once a file passes
  ~400 lines.
- `insta` snapshots for IR and CLI output. A changed snapshot is a review item: explain it in the
  commit, never re-pin by reflex.
- Adding a check rule adds a file to `tests/corpus/invalid/`. Adding a format adds a plan to
  `tests/corpus/synthetic/` with its snapshot.
- Execution tests run on a temporary copy of `tests/fixtures/greeter` (`git init` in the copy),
  never on this repository.

## Docs stay in step

- A changed decision: a new `D<n>` in docs/decisions.md that supersedes the old one.
- A changed spec schema: `spec.rs`, docs/extending.md §2 and `formats/*.toml` together (the
  `spec_files` test checks the files).
- A new recognizer kind: docs/extending.md §3.
- A new format: docs/plan-grammar.md.

## Working with Rens

- Workflow: brainstorm → design spec → implementation plan (`docs/plans/`, assay format, see
  `docs/plans/README.md`) → execution. Propose before making non-trivial edits; stop after writing
  a plan.
- Direct, technically precise; no flattery. Put review text and anything meant to be pasted in
  code blocks.
- Rens works on Windows with PowerShell 7. Commands in plans are PowerShell 7.
- Rulings relayed from a review seat (another Claude session with less context) are checked
  against the code and the plans before acting. Where one rests on a wrong premise, keep the
  correct position and state the evidence (file, task, what the plan says) in the plan's
  stated decisions or the reply, so a further review can settle it.
- One commit per task, message `P<N>-T<n>: <title>`. Don't push.
- When a phase is done, write a handover for the next session:
  `docs/handovers/YYYY-MM-DD-p<N>-to-p<N+1>.md` (state, what the phase built, open items, notes
  for the next phase, how plans are verified). A new session starts from the latest one.
````

File `docs/plan-grammar.md` (full contents):

```markdown
# Plan grammar

What planrun reads: the rules common to every format (§1), then the four known formats (§2–§5).
Every format is described by its spec in `formats/<id>.toml`; this document explains them for
people. The synthetic examples are in `tests/corpus/synthetic/`.

## 1. Common rules

### 1.1 Text

- UTF-8. A leading BOM is removed; CRLF and lone CR become LF (D6). Nothing else is changed.
- Patterns match the **raw** text of a block (backticks, `**`, `—` included), anchored to the
  block.

### 1.2 Fenced blocks

- CommonMark fences: a fence opened with N backticks closes at the first line of ≥ N backticks.
  This is what lets a ```` ````markdown ```` block contain ```` ``` ```` blocks.
- An **unclosed fence is an error** (`unclosed-fence`), not "runs to the end of the document".
  So is a fence closed by accident: inside an open fence, a line indented at most 3 spaces that
  opens a fence of the same character, at least as long, with an info string (architecture.md
  §4.1).
- The body is exact: everything between the fence lines, each line ending in `\n`.
- The info string's first word is the language. `powershell`, `pwsh`, `ps1` run in pwsh; `bash`,
  `sh`, `shell` in bash (a format may change these lists). `text` is data (an expectation, or a
  `Note`). Any other language not consumed by a recognizer is `Manual(ForeignLanguage)`.

### 1.3 Fences inside list items

CommonMark strips the item's indentation from a nested fence, so its body is not contiguous in the
source. The document builder rebuilds it (`TextRef::Owned`) and keeps the original span for
messages.

### 1.4 Paths

Paths in plans are repository-relative with forward slashes. `` `src/a.rs:3-10` `` in a Files list
is a path with an informational line range. Anything absolute, with `..`, or with a backslash is
`bad-path`.

### 1.5 What is never executed

- Text before the first task (the preamble) and after the tasks: shown by `show`, never run.
  Manual markers in the preamble are listed by `check` and shown before task 1.
- HTML comments.
- `Note`s: shown in place.

## 2. `assay` — plans generated by planbuild.py

The strictest format; fully automatable. Example: `tests/corpus/synthetic/assay-style.md`.

| Element | Form |
|---|---|
| Tasks start | `## Tasks` (level 2) |
| Task | `### P1-T1: title`, `### P3c-1-T1: title` (id `<phase>-T<ord>`; phase `P1`, `P3c-1`) |
| Tasks end | the next level-2 heading (`## Part 1 green state`) |
| Goal | `**Goal:** …` (a Note) |
| Manifest | `**Files**` + list items ``Create `p` ``, ``Modify `p` ``, ``Delete `p` ``, each optionally followed by a parenthesised note; consumed by the outline |
| Step | `**Step 1 — title**` (em dash) |
| Full contents | ``File `p` (full contents):`` + fence. Create or overwrite per the manifest |
| Patch | ``File `p` (exact diff; save the block as `…`, outside the repository, then apply it):`` + `diff` fence |
| Patch ceremony | the `powershell` fence right after a patch with `git apply … $env:TEMP\….patch` / `Remove-Item`: **absorbed**, never run (D17) |
| Delete | ``Delete `p`.`` |
| Other ops | ``File `p` (…):`` with any other parenthesis: `unknown-file-op` |
| Generated | ``` `p` is generated by …, not written by hand ``` (or "never"); ``` `p` changes: it is regenerated by … ```; ``Restore `p` from … (never written by hand; …):`` before the script that restores it |
| Script | `powershell` fence. Intent: `commit` when it runs `git commit`; `gate` when it sets `$gate = $LASTEXITCODE`; else verify |
| Expected | `Expected (exit code N; summary lines):` + `text` fence → exit code N (0 is Success) + summary lines |
| Prose expected | `Expected: …` sentence → phrase table → exit expectation + prose |
| Commit | the commit script, as written; it is gated by `$gate` in the plan itself, and planrun gates it on the task's verification too |

Notes:

- A task may name a generated path twice (an intro notice and a step notice): one check, after
  the task's last non-commit action.
- The patch's `index aaaaaaa..bbbbbbb` line carries the pre- and postimage blob ids; consecutive
  patches on a path chain (`blob-chain`).
- The commit script's `git diff --cached --exit-code <scratch-sha>` check refers to scratch commits
  that exist only on the plan author's machine; it is run as written and its failure is reported
  like any failure.

## 3. `superpowers` — the writing-plans skill

Hand-written by Claude; partly automatable. Spec: `formats/superpowers.toml`. Example:
`superpowers-style.md`.

| Element | Form |
|---|---|
| Task | `### Task 1: title` (id `Task 1`, ordinal from the number) |
| Tasks end | the next level-2 heading (`## Self-Review`) |
| Manifest | `**Files:**` + `- Create: \`p\``, `- Modify: \`p:3-10\``, `- Test: \`p\``, each optionally followed by a parenthesised note. Optional: a task may have none (D27) |
| Step | list item `- [ ] **Step 1: title**`; the manifest's items end at the first step (D37) |
| Run | `` Run: `cmd` `` in a paragraph, often with `Expected: …` on the next line **of the same paragraph** → `RunScript` in pwsh with that expectation (`run-line`, D39) |
| Create | ``Create `p` with …:`` + fence → `WriteFile(Create)`; ``Replace the entire contents of `p` with:`` + fence → `WriteFile(Overwrite)`. The op decides; a manifest entry that disagrees is `manifest-write-mode` (D26) |
| Exact edit | ``In `p`, line 134, change:`` + fence + `to:` + fence → `ReplaceExact` |
| Prose edits | `Append to …:`, ``In `p`, replace … with:``, `Above …, add:` + fence → **`Manual(ProseEdit)`** (`manual-prose`) |
| Commit | `bash` fence with `git commit`, as the first block of its step |
| Other shell fences | only behind `Run:`; otherwise `Manual(Unrecognized)` (D28) |

Step marks (D30): a title like "…verify it fails" marks the step's verify scripts as **expected
failures**, skipped by default (D12); "Manual verification", "Check by eye" turn the step's Notes
into `Manual(HumanCheck)`.

Checks (D31): `manifest-untouched` is off (prose edits change declared files),
`manifest-undeclared` is a warning.

## 4. `devready` — DevReady plans

Hand-written for an agent; partly automatable (D40): the target is zero errors and honest
Manual coverage, not automation. Spec: `formats/devready.toml`. Example: `devready-style.md`.

| Element | Form |
|---|---|
| Task | `### Task 0: title` (Task 0 is allowed; ordinals strictly increase from it); `## Acceptance …` / `## Manual steps …` end the tasks |
| Manifest, list | `**Files:** create` + bare items `` - `p` `` (the change is in the marker); `**Files:**` + `- Modify: \`p\``, `- Modify (note): \`a\`, \`b\``, `- Delete: \`p\` (\`git rm\`)`, `- Generated: …`. Optional: Tasks 0 and 1 have none (D27) |
| Manifest, inline | ``**Files:** create `a`; modify `b` and `c`. It generates …`` (D43): clauses split on `; `, the manifest ends at the first period outside a code span. Text that declares nothing ("regenerate the README examples", the sentence after the period) is a Note at the start of the task |
| Paths | `` `a` (full replacement), `b` ``: only `a` is Rewrite (D44). No siblings: `` `src/x/a.rs` and `README.md` `` declares a Bare `README.md` whose directory is unknown (D45). `` `functions/{a,b*}.ps1` `` is a pattern entry (D41) |
| Steps | the sub-labels `**2a.**`, `**1a. title** (…)`, `**1a–1d. title**`, `**Verify:**`, `**Verify** (…):`, `**Run + verify:**`, `**Commit:**` and a plain `Commit:` split a task into steps and stay in them (D46); the label's `1a` is no step number |
| Create | `` `p`: ``, ``**4b.** `p` (note):``, ``**2f.** Create `p` (note):`` + fence → `WriteFile(Create)`; ``(replaces the whole file):`` → `WriteFile(Overwrite)`. A Create onto a Bare entry is `Manual(AmbiguousPath)` (D45). File contents with no lead (right after an inline `**Files:** create`) stay `Manual(Unrecognized)` |
| Edit, two fences | ``\`p\`, edit 1 (old → new):``, ``NOTES.md edit 1 (old → new):`` + two fences → `ReplaceExact`; a basename resolves through the manifest (D32) |
| Edit series | `Edit 2 (old → new):` directly after the ReplaceExact of edit 1 edits the same path (D49); otherwise the task's one editable entry, else `Manual(ProseEdit)` |
| Edit, Old/New | ``**2c.** `p`. Old:``, ``README.md. Old:``, ``**2d.** `p`: insert … Old (end of file):`` + fence + `New:` + fence → `ReplaceExact`; `Old:`, `(2) Old:`, ``In `Member`. Old:`` take the task's one editable entry, else `Manual(ProseEdit)` |
| Prose edits | inline `` Old: `a` `` / `` New: `b` ``, `` Edit: `a` → `b` `` (D34), `New:` + fence after an inline Old, "append …:", "Insert directly before …:" + fence → `Manual(ProseEdit)` |
| Delete | `` `git rm p` ``, ``**4g.** `git rm p` `` → `Delete { via_git: true }` |
| Scripts | a shell fence runs only behind an allowlisted lead: a Verify label, `**Run + verify:**`, `**Commit:**` / `Commit:`, `**4l. Generate and verify.**`, a trailing `(Bash tool):`. Never behind a bare sub-label (D28, D46); any other shell fence is `Manual(Unrecognized)`. `-DryRun` / `-All` bodies are Preview / Apply |
| Not run | ``… (run from `$REPO`):`` + fence and "Then the same command with `-All` instead of `-DryRun`. Verify:" + fence → `Manual(HumanCheck)`: a script has no working directory of its own, and the second fence uses the first one's variables |
| Comment expectations | trailing `# expected` comments on a claimed fence's command lines → a comment expectation, exit not checked, always confirmed (D35); an `Expected:` sentence or list after it extends it (D48) |
| Verify line | ``**Verify:** `cmd` → result`` → `RunScript` in pwsh with that expectation; with two commands (`` `a` → x; `b` → y ``) → `Manual(HumanCheck)` |
| Expected | `Expected: …` → prose expectation; `Expected:` + a list → one prose expectation over the list and what is nested in it (D47); after a Manual it joins the Manual (D38). `Expected and recorded:` is no expectation |
| Commit | `**Commit:** \`msg\`` → `Manual(HumanCheck)` carrying the message; P5 suggests `git add -- <paths>; git commit -m "<msg>"` only when every manifest path of the task is literal (no pattern, sibling, Bare or unresolved name, D33). ``Commit: `git add …; git commit -m "…"` `` → a pwsh run line with the commit intent; `Commit:` + fence → the commit script |
| Manual markers | `VERIFY`, "record" as an instruction, "stop and report", "Check by eye", "Negative check", "don't …", "by hand", "manual step", "Rens runs" |

Checks (D31): `manifest-untouched` is off (prose edits and scripts change declared files),
`manifest-undeclared` is a warning.

## 5. `before-after` — "before:/After:" plans

Hand-written; mostly automatable. Spec: `formats/before-after.toml`. Example:
`before-after-style.md`.

| Element | Form |
|---|---|
| Task | `## Task 1: title` (**level 2**); a level-2 heading that is no task ends the tasks (D37) |
| File map | a table before the tasks: shown, never parsed into actions |
| Manifest | `**Files:**` + `Create:`, `Modify:` (with `:lines`), `Rewrite:`, `Test:`; several paths per item, separated by `, ` or ` and `; `` `dir/0001`–`0007_*.sql` `` is one range entry (D41); a bare file name after a path is in that path's directory and a `:lines` token adds lines to the path before it (D32); a trailing `(note)`. Optional: a task may have none |
| Step | list item `- [ ] **Step 1: title**` |
| Create | `` `p`: `` / ``Create `p` (…):`` + fence → `WriteFile(Create)` |
| Overwrite | `` `p` — replace the whole file: `` / ``Replace `p`:`` + fence → `WriteFile(Overwrite)` |
| Edit | `` `p`, before: `` + fence + `After:` + fence → `ReplaceExact` (`before` must occur exactly once) |
| Edit without a path | `Before:` / ``In `Member`, before:`` + fence + `After:` + fence → `ReplaceExact` on the task's one editable entry, else `Manual(ProseEdit)` (D32) |
| Partial edits | ``"`p`, replace the class (keep …):"``, ``"`p`, add these members to the class:"``, `Add … :` → `Manual(ProseEdit)` |
| Scripts | `powershell` fences that are the first block of a step or follow a run lead (`Run:`, `Preview…:`, `Apply:`, `Verify…:`, `… Then:`); any other shell fence is `Manual(Unrecognized)` (D28) |
| Expected | prose `Expected: …`, including `build error` (expected failure) and `no matches` (`rg` exits 1); after a Manual (an `sql` block) it joins the Manual (D38) |
| Preview/apply | a `Preview…:` / `Apply:` lead, or a `-DryRun` / `-All` body: intent Preview then Apply (D29); planrun always stops between them and shows the count next to the plan's expected count. **The Apply invariant:** planrun always stops before an Apply-intent script, whether or not a Preview ran before it; the stop never depends on a Preview action directly before it, because the preview may be Manual (D28). P5 enforces it |
| Output-dependent | "Replace … with the `Actual` values" → `Manual(OutputDependent)` |
| Other languages | `sql` fences → `Manual(ForeignLanguage("sql"))` |
| Prose checks | `Verify: <sentence>` → `Manual(HumanCheck)` |

Step marks (D30): "…verify they fail" marks expected failures; a title ending in "on … TST" or
starting with "Hand off" turns the step's prose into `Manual(HumanCheck)`.

Checks (D31): `manifest-untouched` is off (scripts and prose edits change declared files),
`manifest-undeclared` is a warning.

## 6. Recognizing a new format

Write the table above for it first, from at least two real plans. Then follow extending.md §2.
```

File `docs/roadmap.md` (full contents):

```markdown
# Roadmap

Each phase gets its own plan in `docs/plans/` (conventions: [plans/README.md](plans/README.md)),
reviewed by Rens before it is executed. A phase is done when its acceptance list holds and
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` are green.

The order puts the riskiest assumption first: that real plans can be parsed strictly, with full
accounting of every block.

## P0 — Starter package (done)

Workspace of four crates; the IR in `planrun-model` with its tests; the parse and exec extension
points as traits; the CLI argument surface; `formats/assay.toml`; the synthetic and invalid
corpus; the greeter fixture; these docs.

## P1 — Parse the assay format strictly (done)

The deliverable: `planrun check` and `planrun show` on assay plans, with precise diagnostics.

Suggested tasks (the plan may split or merge them):

1. **Source and document.** `source::load` (UTF-8, ≤ 64 MiB, BOM, CRLF/CR → LF, SHA-256 of the
   original bytes; adds `sha2`, D9). The document builder over pulldown-cmark's offset iterator:
   every `Block` variant, spans, list depth, `TextRef::Owned` for fences inside list items, and
   the `unclosed-fence` error. Unit tests on small strings, including a ```` ```` ```` fence that
   contains ```` ``` ```` fences, a fence inside a list item, CRLF input, and an empty fence body.
2. **Specs, registry, format selection, outline.** `Registry::add_spec` (deserialize, compile
   every regex, build recognizers through factories, reject unknown kinds and duplicate ids),
   `with_builtins` embedding `formats/assay.toml`, the generic `SpecFormat` (`detect`, `outline`:
   tasks after `tasks_after`, task ids and ordinals, phase, intro step, manifest, step ranges,
   `tasks_end_level`), selection with threshold and ties (`no-format`, `ambiguous-format`).
3. **Recognition.** The cursor loop with `No`/`Yes`/`Amend`/`Malformed` and the fallback; the
   kinds the assay spec uses: `file-block`, `ceremony`, `delete`, `generated`, `script`,
   `expected-block`, `expected-prose`. Patch parsing for `ApplyPatch` (target path from
   `diff --git`/`---`/`+++` agrees with the lead: `patch-path`; the `index` line → pre/post).
4. **Checks.** The rules `task-order` (with `duplicate-task-id`), `step-order`, `manifest`
   (`manifest-undeclared`, `manifest-untouched`, `manifest-write-mode`), `patch-shape`,
   `blob-chain`, `commit` (`commit-twice`, `commit-not-last`); the coverage report. The
   invalid-corpus test (`tests/corpus/invalid/README.md`), which also pins the codes of the
   outline and the recognizers (`bad-path`, `unknown-file-op`, `missing-block`, `patch-path`, …).
5. **CLI.** `planrun check` (diagnostics via `annotate-snippets`, coverage table, exit 0 / 1 on
   errors / 2 on usage) and `planrun show` (plan, task, task/step). An `insta` snapshot of the IR
   of `assay-style.md` and of `planrun check` output. The `#[ignore]`d local-corpus test.

Acceptance:

- `planrun check tests/corpus/synthetic/assay-style.md` exits 0; every block of every task is an
  action except the `**Goal:**` lines and prose (Notes); the absorbed ceremonies are listed as
  absorbed.
- Every `tests/corpus/invalid/*.md` yields exactly its pinned error code.
- The two local assay plans (`2026-11-05-s5-1-p1-routes-seam.md`,
  `2026-11-11-s5-1-p3c1-builder-core.md`) parse with zero errors. Expect to widen the `generated`
  pattern for "`Cargo.lock` changes: it is regenerated by cargo in the next step", and to see
  the P3c-1 plan's prose-heavy preamble as Notes.
- Every diagnostic shows file:line:col and the source line.

## P2 — The other three formats

The decisions are D26–D40. Split in two plans on one schema (D40):

- **P2a (done).** The P1 outline fixes and `task-after-end` (D37); the spec schema (`required`,
  a marker that captures `change`, `siblings`, `ChangeKind::Generated`, `[checks]`,
  `[[step_marks]]`, `script` `leads`/`first_block`/intent `lead`); the kinds `run-line`,
  `before-after`, `manual-prose`; path resolution from the manifest (D32);
  `formats/superpowers.toml` and `before-after.toml` with IR snapshots of their synthetic plans.
  The superpowers and before-after local plans parse with zero errors. Plan:
  `docs/plans/2026-11-24-p2a-hand-written-formats.md`.
- **P2b.** First, pattern manifest entries: a range or glob item is one entry whose target is a
  `PathPattern` (D41, D42). Then the schema for hand-written DevReady plans: inline Files clauses
  (D43), a change per path (D44), Bare file names (D45), kept step labels (D46), `Expected:` with
  a list (D47), comment expectations that a sentence extends (D35, D48), numbered edit series
  (D49). Then `formats/devready.toml` with the `**Commit:**` Manual (D33), detection that
  outscores superpowers on both devready plans, the IR snapshot of its synthetic plan, and the
  Apply invariant in plan-grammar §5. All six local plans parse with zero errors. Plan:
  `docs/plans/2026-11-25-p2b-devready-format.md`.

Each plan lists the imperative Notes that remain in the real plans (D36). `expected-failure-step`
became step marks (D30); `preview-apply` became script intents by lead (D29).

## P3 — Workspace, git and dry-run

`RepoPath` resolution with the symlink check; the per-task file cache; step transactions (D23);
line-ending policy (D10); pre-images in the git object database; the `git` wrapper (`apply
--check`/`apply` on stdin, `hash-object --path`, `status --porcelain`, `cat-file`); blob-id checks;
task preflight; `planrun run --dry-run` against a virtual tree. Integration tests on a temporary
copy of `tests/fixtures/greeter`: the dry run of `assay-style.md` passes; a fixture with
`src/lib.rs` edited fails preflight with the blob mismatch; a Create onto an existing file fails
before anything is written.

## P4 — Shells, expectations, policy

Persistent pwsh and bash sessions (D24) with the sentinel protocol: variables persist across
blocks, a `throw` ends the block and not the session, output streams. The expectation matcher
(summary-line subsequence with `…`, exit expectations, prose → confirmation, D11). The command
classifier (D13). Expected failures skipped by default (D12). Tests for each, with pwsh and bash
when present (skipped with a message when not).

Candidate (from P2b): a script working directory taken from a run lead (``(run from `$REPO`):``).
`Script` has no cwd today, so such fences are Manual in the devready format.

## P5 — Runner, journal, resume, interactive CLI

The runner state machine; the commit gate (D22); the JSONL journal (D18); resume with `DoneState`
(D14); `run`, `resume`, `status`; the line-prompt UI (D15) with view/diff/shell/skip/quit. End to
end: `planrun run tests/corpus/synthetic/assay-style.md --auto read,build,modify` on a fixture copy
produces the same two commits (same trees) as the fixture's history; killing the process mid-task
and resuming completes it without repeating a write.

## P6 — Dogfood and polish

planrun runs its own later phase plans. Candidates, each its own decision: a ratatui UI, a
sidecar IR from planbuild.py, `.planrun/formats/` discovery polish, a non-interactive batch mode
for CI.
```

**Step 2 — verify**

Targeted: the synthetic devready plan:

```powershell
cargo run -q -p planrun-cli -- check tests/corpus/synthetic/devready-style.md
```

Expected (exit code 0; summary lines):

```text
planrun: tests/corpus/synthetic/devready-style.md: ok (devready, 5 tasks)
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- CLAUDE.md docs/plan-grammar.md docs/roadmap.md
  $verified = [ordered]@{
      'CLAUDE.md' = '7f802b70b41562a1675438fc17de0a6e9576b5ba'
      'docs/plan-grammar.md' = '2171610d00936e56dc16a454ba32d365e750a1e5'
      'docs/roadmap.md' = '7aa6115a8b9ca54f0e5700bb43cf7c5cdc74ba54'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P2b-T5: plan-grammar devready section, the Apply invariant, roadmap and CLAUDE.md" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

## P2b green state

Run from the repository root after P2b-T5:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-parse --test local_corpus -- --ignored local_corpus }
```

Expected: every `test result:` line ok; the last one `1 passed`, with `6 parsed, 0 skipped` in the local-corpus output.

User-visible change: `planrun check` and `planrun show` work on devready plans; manifests show pattern and bare entries; scripts show their comment expectations.

## After execution

- `git log --oneline -7` lists the commit of this plan, then `P2b-T1` to `P2b-T5`.
- `cargo run -q -p planrun-cli -- check docs/plans/2026-11-25-p2b-devready-format.md` exits 0: this plan is an assay plan.
- Write the handover `docs/handovers/2026-11-25-p2-to-p3.md` in a separate session or commit after review.
