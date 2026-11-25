# planrun P3 — Workspace, git and dry run

**Goal:** P3 of docs/roadmap.md: the execution side's first half. The P2b review items go first (handover open items 2–4 and `write-mode-unknown`), then `RepoPath` hardening, the `git` wrapper, the workspace with its one writer, line endings, the virtual tree, step transactions as prepare and commit, task preflight with D50's scope, and `planrun run --dry-run`. Decisions D50–D59. After P3 the dry run of `tests/corpus/synthetic/assay-style.md` passes on a fixture copy and writes nothing; a changed `src/lib.rs` fails preflight with the blob mismatch; a Create onto an existing file fails before anything is written.

- **P3-T1** (`planrun-parse`, `formats`, docs): an edit series' leading word names its file (D59, handover item 2); `(run from …)` anywhere in its lead (item 3); negated failures map to `any` (item 4); `write-mode-unknown` (D58); decisions D50–D59 with their amended-by lines.
- **P3-T2** (`planrun-model`, `planrun-parse`, corpus): `RepoPath` refuses Windows-reserved spellings and git directories (D51); `Manifest::scopes` and `Manifest::is_bare_target` (D50), which parse's Bare test now calls.
- **P3-T3** (`planrun-exec`, tests): the shared fixture helper `tests/support/fixture.rs`; the `git` wrapper and `BlobId`; `sha1_smol` out of planrun-exec (D56).
- **P3-T4** (`planrun-exec`): the workspace: path resolution with the link and case check, the one writer, the test seam (D52); line endings (D54).
- **P3-T5** (`planrun-exec`): findings, the virtual tree (D52), and `ActionHandler::simulate` for WriteFile, ApplyPatch, ReplaceExact and Delete (D53).
- **P3-T6** (`planrun-exec`): step transactions: prepare, commit with a write-ahead recorder, rollback, the `via_git` index update (D53).
- **P3-T7** (`planrun-exec`): task preflight and the dry run: manifest checks, scope and `path-ambiguous` (D50), real dirt and steps that did not run (D55).
- **P3-T8** (`planrun-cli`): `planrun run --dry-run`, its report, and the P3 acceptance tests on parsed plans.
- **P3-T9** (docs, `CLAUDE.md`): architecture.md §3, §6.1, §6.2 and §7, extending.md §4 and §7, the roadmap, the CLAUDE.md layout and test conventions.

**Base:** `eecab1e` (`Handover and P2 done`), the child of `67caf36` (P2b-T5) that adds only `docs/handovers/2026-11-25-p2-to-p3.md` and the roadmap's "done" marks; execute from the commit of this plan. Gate per task: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Shell and environment:** PowerShell 7 on Windows; Rust 1.95.0 (stable); git on `PATH` (2.29 or later: `rev-parse --show-object-format`). No new crate. `Cargo.lock` changes once, in P3-T3: `sha1_smol` leaves planrun-exec's dependency list.

**Before P3-T1:** this plan committed, and a clean `git status --porcelain`. The real plans stay in `tests/corpus/local/` (git-ignored, D16); this plan names them by file at most and quotes none of them. Execution tests run on temporary copies of `tests/fixtures/greeter` under the OS temp directory, never on this repository.

**How this plan was verified:** every task was built in a scratch clone of `eecab1e` outside this repository (`core.autocrlf=false`), one commit per task. At each commit `cargo fmt --check`, clippy and `cargo test` pass, and so does the task's targeted command; the summary lines below come from those runs. After every task, `planrun show` and `planrun check` on the six local plans printed exactly what they printed at `eecab1e`, and the local-corpus test passed (`6 parsed, 0 skipped`). The plan was generated from that history (full contents from `git show`, exact diffs from `git diff`, blob ids from `git rev-parse`), `planrun check` passes on it, and it was replayed task by task on a fresh clone with `core.autocrlf=true`: every commit step committed, and `git status --porcelain` was empty after each.

## Review rulings (P3 questions, 2026-11-25)

Each accepted answer is a decision in docs/decisions.md (all written in P3-T1, stated decision 3); this list maps them.

1. **Q0** base: `eecab1e`, checked (stated decision 1).
2. **Q1** D50 with three changes: Bare is unscoped; a Bare-targeted Overwrite or Delete is `path-ambiguous`, a Bare-targeted ApplyPatch or ReplaceExact is unscoped and confirmed, `target-missing` with the D45 hint; nothing switches D50 off. The rule is keyed on the manifest, not on the action's origin (stated decision 2), and the predicates live in planrun-model (P3-T2).
3. **Q2** dirt: a dirty touched path is `path-dirty`, other dirt a warning; the dry run checks once for every task's paths; every status call runs `git --no-optional-locks status --porcelain -z` (D52, D55). The P5 items (dirt from planrun's own writes after a Manual commit, a commit script that stages everything) go into the P3→P4 handover.
4. **Q3** the virtual tree in OS temp with the repository's objects as alternates, env per `Command` only; pre-images with `hash-object -w --no-filters` (D52); path identity under `core.ignorecase` (`path-collision`); "not seeded", "absent", "deleted", "unknown" told apart; the real run builds every post-image on the virtual tree and one writer writes them (no `git apply` on the working tree); `ActionHandler` gets `simulate` only; prepare → commit with a write-ahead recorder; rollback and the `via_git` index update last (D53); pruning recorded for P5 (D57). The "writes nothing" test hashes every file under the copy, `.git` included, with `core.autocrlf=true` (P3-T8).
5. **Q4** after a step that did not run, a failing file action is a warning and its path unknown; exit 0 with only warnings; the header says what ok means (D55, stated decision 14).
6. **Q5** `RepoPath` refuses `:`, `<`, `>`, `"`, `|`, control characters, `.git` in any spelling and `git~1`, device names with their superscript forms, segments ending in a dot or a space (D51); one invalid-corpus file for a `.git/hooks/...` write (stated decision 7).
7. **Q6** line endings: attribute, then the file, then git's checkout convention; a mixed file is overwritten by the checkout convention and refused for an exact edit (`eol-mixed`); created files LF; attributes read at the start of a step (D54). Tests with `core.autocrlf` true and false (P3-T5).
8. **Q7** no `tempfile`: one helper, `tests/support/fixture.rs`, included with `#[path]` by the exec and cli tests, as specified (stated decision 8).
9. **Q8** no SHA-1 fast path; `sha1_smol` leaves planrun-exec, its D9 line says planrun-parse only (D56).
10. **Q9** defaults as amended: simulate only; P3-T1 is items 2–4 plus `write-mode-unknown`, tested with a spec of its own; exec refuses `FromManifest` as an error that never prompts (D58).
11. **Additions:** the writer's second check, `create_new` temp files named `.planrun-<random>.tmp`, directories one segment at a time, the remaining race stated (D52; tests in P3-T4); architecture §3, §6.1, §7 (P3-T9); the amended-by lines and the D46 heading (P3-T1); the P4/P5 items for the next handover ("After execution").
12. **Second round:** Bare-targeting keyed on the manifest, the predicates in the model (P3-T2); exec tests build IR by hand, tests that need a parsed plan live in planrun-cli, and the two roadmap cases call preflight and prepare directly (P3-T8). The exec refusal of `write-mode-unknown` is tested in P3-T5 (`simulate.rs`, `writes_check_existence_and_from_manifest_is_refused`), the case collision in P3-T5 (`two_spellings_of_one_path_collide_with_ignorecase`, and `spellings_stay_apart_without_ignorecase` as its control).

## Stated decisions made while building

1. **Base.** `eecab1e`'s parent is `67caf36`, it changes only `docs/roadmap.md` and `docs/handovers/2026-11-25-p2-to-p3.md`, and `git status --porcelain` was empty, as checked before the build.
2. **Actions never carry `PathOrigin::Bare`** (Q1(b) as first ruled). `crates/planrun-parse/src/manifest.rs:414` sets `Bare` on manifest entries only, and a lead that names a bare file name resolves to `Written` (`crates/planrun-parse/src/kinds/mod.rs:104-106`). So "Bare-targeted" means: some Bare file entry of the task equals the action's path. `Manifest::scopes` and `Manifest::is_bare_target` (planrun-model, P3-T2) are the one implementation; parse's `is_bare` calls the second, and exec calls both. A path with both a Written and a Bare entry is scoped and Bare-targeted at once (conservative, loud), as ruled.
3. **All of D50–D59 are written in P3-T1**, with the amended-by lines in D7, D9, D10, D23, D42 and D49 and the D46 heading fixed, instead of in the docs task. The code cites the decisions from P3-T1 on, and docs/decisions.md keeps its numbers in file order. P3-T9 does the other docs.
4. **D59 is a decision of its own.** Handover item 2 changes D49's behaviour, and a changed decision gets a number (CLAUDE.md, Docs stay in step). It covers the inferred path too, not only the continuation: the word names the file either way, and with the continuation alone `NOTES edit 1` in a task whose one editable entry is `README.md` would still edit `README.md`. The stem is compared exactly; a mismatch only ever falls back to Manual.
5. **Negated failures map to `any`**, not to `success`: exit not checked, the sentence confirmed (D11). The row `(?:\b(?:no|not|nothing|none|never|without)\b|n't\b)[^.;\n]*\b(?:fail|build error)` leads the devready (two tables), superpowers (two) and before-after (one) tables. assay's table is unchanged: planbuild writes no negated failures, and it has its own `no output` row. "FAIL with "cannot find …"" stays a failure (`cannot` is no `\bnot\b`; tested).
6. **`write-mode-unknown` is reported once.** In assay, an undeclared `write` is already `manifest-undeclared` (an error), and `tests/corpus/invalid/manifest-undeclared.md` must keep exactly one code; `check::drop_reported_write_modes` drops a `write-mode-unknown` whose action has another error, after the format's `[checks]` levels are applied, so where `manifest-undeclared` is a warning it stays. No real plan and no invalid-corpus file changes.
7. **`tests/corpus/invalid/bad-path.git-hooks.md`.** The file-name test required `<code>.md`; it now accepts `<code>.<case>.md` for a second case of one code, and the corpus README says so. `cli__check_invalid_corpus.snap` gains that one section; it is the only changed snapshot in P3.
8. **The fixture helper** (Q7): the copy is `<temp>/planrun-test-<pid>-<nanos>-<n>/repo`, the empty global config is the `gitconfig` file next to it, and `GIT_CEILING_DIRECTORIES` is their parent; `outside()` gives a directory next to the copy for link targets and for the plan copy the CLI tests run. It commits once with an author and committer set in its env. With `core.autocrlf=true` it deletes and checks out the tracked files again, so the working tree really has CRLF (`blob_ids_are_git_s_view_with_and_without_autocrlf` asserts it). After every gate run no `planrun-test-*` or `planrun-tree-*` directory was left in the temp directory.
9. **The git wrapper** sets `GIT_LITERAL_PATHSPECS=1` on every call (a path with `[` would otherwise be a glob pathspec), runs status with `--untracked-files=all`, and `Workspace::open` refuses SHA-256 repositories (`object-format`): plans carry SHA-1 abbreviations.
10. **The test seam is `#[cfg(test)]` only** (`Faults` in workspace.rs): exec has no public test API. The unit tests that use it (workspace, transaction) include the fixture and IR helpers with `#[path]` from `lib.rs`. The file-symlink test skips on Windows without the privilege (it skipped here); the junction tests ran.
11. **The link and junction tests are in P3-T4**, not P3-T8 as the task list said: they test the writer, which P3-T4 builds.
12. **Checkout convention** (D54 case 3): a one-line blob `a\n` is stored in the virtual tree's objects and converted with `git cat-file --filters --path=<p>`; CRLF out means CRLF. That honours attributes, `core.eol` and `core.autocrlf` without re-implementing git's rules.
13. **The roadmap's "per-task file cache" is the virtual tree**: a path is read and hashed once, on first use. Seeding costs three git calls per path (index mode, `hash-object -w`, `update-index`); acceptable for now (D25).
14. **D55 covers Manual steps and state failures only.** A Manual prose edit changes files as a script does (`a_manual_counts_as_a_step_that_did_not_run`), so a Manual sets "did not run" too, and the label is "depends on a step that did not run". Only failures of the tree's state are downgraded; a bad path, `path-ambiguous` and `write-mode-unknown` stay errors. D55's text says so.
15. **One `git status` for the dirt check**, partitioned into touched paths and the rest: one process instead of two, and no pathspec list that could pass the Windows command-line limit on a plan with many paths.
16. **A `Delete` with `via_git` is not checked for being tracked** in a simulation: a file an earlier task created is tracked only after a commit the dry run assumes. `update-index --force-remove` on an untracked path does nothing.
17. **An empty `before`** is `before-not-unique` (0 occurrences), never a match everywhere.
18. **The CLI.** `run` takes `--format`; `run` without `--dry-run` exits 2 with a P5 message; the repository is the one of the current directory (no `--repo`). Findings are rendered like parse diagnostics (annotate-snippets), after the action table and the summary. The report lists both `Generated` notices of `assay-style.md` P1-T2 (the intro's and the step's); the one-check rule for them is P5's.
19. **Snapshots.** New: `dry_run__dry_run_assay_style.snap` and `dry_run__dry_run_create_exists.snap` (P3-T8). Changed: `cli__check_invalid_corpus.snap` (P3-T2, stated decision 7). The two new files carry no `assertion_line`, like the existing ones.
20. **Blank context lines in test patches** are written as `\x20` in the Rust string literals (`tests/git.rs`, `tests/support/ir.rs`), so no editor strips them; git would accept an empty line as blank context, but the patches stay byte-equal to the plan's.
21. **Local corpus.** None of P3-T1's three spec changes hits a real plan: `planrun show` and `planrun check` on all six local plans print exactly what they printed at `eecab1e`, after every task.
22. **The fixture is copied as git stores it.** The first replay on a `core.autocrlf=true` clone failed P3-T3: there `tests/fixtures/greeter` is checked out with CRLF, the helper copied those bytes, and the copy's `src/lib.rs` was no longer blob `aa80cf1`. The helper now turns CRLF back into LF while copying (the fixture is text, stored with LF) and skips a `target/` from building it. The scratch clone (`core.autocrlf=false`) could not show this; the replay did.
23. **Diffs only for modified files over 300 lines**, as in P1, P2a and P2b; every other file is full contents, new files included. Every fence is one backtick longer than any backtick run at 0–3 spaces of indentation in its body.

## Tasks

### P3-T1: edit-series stems (D59), run-from anywhere in its lead, negated failures, write-mode-unknown (D58), decisions D50-D59

**Goal:** Handover open items 2–4 and the D58 rule, all in planrun-parse and the specs. A pathless `before-after` lead may capture the word before "edit" as `stem`; the path it continues or infers must have that stem, or the edit is Manual (D59), and the devready series lead captures it. `manual-prose#run-from` loses its end anchor, so `(run from `x`) (Bash tool):` is Manual and not a claimed script. A negation row leads the exit tables of devready, superpowers and before-after. The new rule `write-mode` reports a `FromManifest` write that no entry declares as `write-mode-unknown`, once. docs/decisions.md gets D50–D59, the amended-by lines and the fixed D46 heading.

**Files**

- Modify `crates/planrun-parse/src/check.rs`
- Modify `crates/planrun-parse/src/kinds/before_after.rs`
- Modify `crates/planrun-parse/src/lib.rs`
- Create `crates/planrun-parse/tests/spec_touchups.rs`
- Modify `docs/decisions.md`
- Modify `docs/extending.md`
- Modify `docs/plan-grammar.md`
- Modify `formats/before-after.toml`
- Modify `formats/devready.toml`
- Modify `formats/superpowers.toml`

Over 300 lines, so they come as exact diffs: `crates/planrun-parse/src/check.rs`, `docs/decisions.md`.

**Step 1 — write the files**

File `crates/planrun-parse/src/check.rs` (exact diff; save the block as `$env:TEMP\p3-t1-check-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/check.rs b/crates/planrun-parse/src/check.rs
index 5a7a3e3..0a16a13 100644
--- a/crates/planrun-parse/src/check.rs
+++ b/crates/planrun-parse/src/check.rs
@@ -9,7 +9,7 @@ use planrun_model::{
     AbbrevBlobId, Action, ActionKind, ChangeKind, Plan, RepoPath, Span, Task, WriteMode,
 };
 
-use crate::diagnostic::Diagnostic;
+use crate::diagnostic::{Diagnostic, Severity};
 use crate::patch;
 
 /// One static check.
@@ -32,6 +32,10 @@ pub trait Rule: Send + Sync {
 ///   matches (`manifest-untouched`); a `WriteFile` on a declared path has a mode that agrees with the
 ///   entry: `FromManifest` resolves, Create is not on an existing file, Overwrite not on a new one
 ///   (`manifest-write-mode`, decision D26). A task without a manifest block is skipped.
+/// - `write-mode`: a `WriteFile` whose mode is still `FromManifest` and that no manifest entry
+///   declares (a task without a manifest, or an undeclared path where `manifest-undeclared` is
+///   only a warning) is `write-mode-unknown`, in every format (decision D58). No `[checks]`
+///   entry can switch it off.
 /// - `patch-shape`: a patch has at least one hunk and its hunks' line counts match their bodies.
 ///   (Its paths and `index` line are checked when it is recognized: `patch-path`, `patch-shape`.)
 /// - `blob-chain`: per path, each patch's preimage equals the postimage of the last earlier
@@ -47,6 +51,7 @@ pub fn default_rules() -> Vec<Box<dyn Rule>> {
         Box::new(TaskOrder),
         Box::new(StepOrder),
         Box::new(ManifestRule),
+        Box::new(WriteModeRule),
         Box::new(PatchShape),
         Box::new(BlobChain),
         Box::new(CommitRule),
@@ -146,6 +151,54 @@ impl Rule for StepOrder {
     }
 }
 
+/// `write-mode`.
+pub struct WriteModeRule;
+
+impl Rule for WriteModeRule {
+    fn id(&self) -> &'static str {
+        "write-mode"
+    }
+
+    fn check(&self, plan: &Plan, out: &mut Vec<Diagnostic>) {
+        for task in &plan.tasks {
+            for action in actions(task) {
+                let ActionKind::WriteFile {
+                    path,
+                    mode: WriteMode::FromManifest,
+                    ..
+                } = &action.kind
+                else {
+                    continue;
+                };
+                if task.manifest.entries.iter().any(|e| e.path.covers(path)) {
+                    continue;
+                }
+                out.push(Diagnostic::error(
+                    "write-mode-unknown",
+                    Some(action.span),
+                    format!(
+                        "full contents for `{path}`: the plan says neither create nor \
+                         overwrite, and no manifest entry declares the file"
+                    ),
+                ));
+            }
+        }
+    }
+}
+
+/// Drops each `write-mode-unknown` whose action already has another error (`manifest-undeclared`
+/// where the format keeps it an error), so a broken write is reported once. Runs after a
+/// format's `[checks]` levels are applied: where `manifest-undeclared` is a warning or off,
+/// `write-mode-unknown` stays (D58).
+pub fn drop_reported_write_modes(diagnostics: &mut Vec<Diagnostic>) {
+    let reported: Vec<Option<Span>> = diagnostics
+        .iter()
+        .filter(|d| d.severity == Severity::Error && d.code != "write-mode-unknown")
+        .map(|d| d.span)
+        .collect();
+    diagnostics.retain(|d| d.code != "write-mode-unknown" || !reported.contains(&d.span));
+}
+
 /// `manifest`.
 pub struct ManifestRule;
 
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p3-t1-check-rs.patch"
Remove-Item "$env:TEMP\p3-t1-check-rs.patch"
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
//!
//! A pathless lead may capture the word before "edit" as `stem` (`README edit 2`). Then the
//! path it continues or infers must have that file-name stem (`README` of `README.md`), or the
//! edit is Manual: the word names the file, and a different file is never edited.

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
        let lead_stem = lead.capture_names().flatten().any(|n| n == "stem");
        if has_path && lead_stem {
            return Err("`stem` is only for a lead without a `path` capture".to_owned());
        }
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

    /// The path a lead without one edits: the series it continues, else (with `infer_path`)
    /// the task's one editable entry. Either must agree with the lead's `stem`, when captured.
    fn pathless(
        &self,
        cx: &StepContext<'_>,
        caps: &regex::Captures<'_>,
    ) -> Option<(RepoPath, PathOrigin)> {
        let agrees = |path: &RepoPath| caps.name("stem").is_none_or(|s| stem(path) == s.as_str());
        if let Some(path) = self.continued(cx, caps).filter(|p| agrees(p)) {
            return Some((path, PathOrigin::PreviousEdit));
        }
        if !self.infer_path {
            return None;
        }
        only_entry(cx)
            .filter(|p| agrees(p))
            .map(|path| (path, PathOrigin::OnlyEntry))
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
        let target = if self.infer_path || self.series.is_some() {
            self.pathless(cx, &caps)
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

/// The file name without its last extension (`README` of `docs/README.md`).
fn stem(path: &RepoPath) -> &str {
    let name = path.file_name();
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}
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
    let mut checked = format.settings().apply_checks(checked);
    check::drop_reported_write_modes(&mut checked);
    found.extend(checked);
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

File `crates/planrun-parse/tests/spec_touchups.rs` (full contents):

```rust
//! The P2b review items fixed at the start of P3: an edit series' leading word names its file
//! (handover item 2), `(run from …)` claims its fence wherever it stands in the lead (item 3), a
//! negated failure is no expected failure (item 4), and a full-contents write that nothing
//! declares is `write-mode-unknown` (decision D58).

use planrun_model::{ActionKind, ExitExpect};
use planrun_parse::{Registry, parse_plan};

fn parse(format: &str, text: &str) -> Result<planrun_parse::Parsed, Vec<&'static str>> {
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, "t.md", text.as_bytes(), Some(format)).map_err(|d| d.error_codes())
}

fn ok(format: &str, text: &str) -> planrun_parse::Parsed {
    parse(format, text).unwrap_or_else(|codes| panic!("{codes:?}"))
}

/// One label per action of the first task, in order.
fn labels(parsed: &planrun_parse::Parsed) -> Vec<String> {
    parsed.plan.tasks[0]
        .steps
        .iter()
        .flat_map(|s| s.actions.iter())
        .map(|a| match &a.kind {
            ActionKind::Note { .. } => "note".to_owned(),
            ActionKind::Manual { reason, .. } => format!("manual {reason:?}"),
            ActionKind::ReplaceExact { path, .. } => format!("replace {path} {:?}", a.path_origin),
            ActionKind::RunScript(s) => {
                let exit = s.expect.as_ref().map_or(ExitExpect::Unknown, |e| e.exit);
                format!("run {exit:?}")
            }
            other => format!("{other:?}"),
        })
        .collect()
}

fn fence(lang: &str, body: &str) -> String {
    format!("```{lang}\n{body}\n```\n\n")
}

fn edit(lead: &str) -> String {
    format!(
        "{lead}\n\n{}{}",
        fence("powershell", "old"),
        fence("powershell", "new")
    )
}

fn devready(files: &str, body: &str) -> String {
    format!("### Task 1: a\n\n**Files:** {files}\n\n{body}")
}

#[test]
fn a_series_word_must_name_the_continued_file() {
    let continues = devready(
        "modify `a/x.ps1`; modify `README.md`",
        &format!(
            "{}{}",
            edit("**1a.** `a/x.ps1`, edit 1 (old → new):"),
            edit("x edit 2 (old → new):")
        ),
    );
    assert_eq!(
        labels(&ok("devready", &continues)),
        ["replace a/x.ps1 Written", "replace a/x.ps1 PreviousEdit"]
    );
    let other_file = continues.replace("x edit 2", "README edit 2");
    assert_eq!(
        labels(&ok("devready", &other_file)),
        ["replace a/x.ps1 Written", "manual ProseEdit"]
    );
}

#[test]
fn a_series_word_must_name_the_inferred_file_too() {
    let named = devready("modify `README.md`", &edit("README edit 1 (old → new):"));
    assert_eq!(
        labels(&ok("devready", &named)),
        ["replace README.md OnlyEntry"]
    );
    let other = devready("modify `README.md`", &edit("NOTES edit 1 (old → new):"));
    assert_eq!(labels(&ok("devready", &other)), ["manual ProseEdit"]);
}

#[test]
fn a_stem_capture_needs_a_lead_without_a_path() {
    let spec = r#"
id = "t"
name = "stem test"
[detect]
patterns = [{ regex = '^### Task', weight = 60 }]
[outline]
task = { level = 3, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
[[recognizer]]
kind = "before-after"
id = "before-after#t"
lead = '^(?P<stem>\w+) `(?P<path>[^`]+)` edit:$'
"#;
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let err = registry
        .add_spec("t", spec)
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();
    assert!(err.contains("`stem` is only for a lead without"), "{err}");
}

#[test]
fn run_from_claims_its_fence_before_a_bash_tool_lead() {
    let text = devready(
        "create `a/x.ps1`",
        &format!(
            "**1a.** Preview (run from `$GR`) (Bash tool):\n\n{}",
            fence("bash", "ls")
        ),
    );
    assert_eq!(labels(&ok("devready", &text)), ["manual HumanCheck"]);
}

#[test]
fn a_negated_failure_is_no_expected_failure_in_devready() {
    let verify = |expected: &str| {
        devready(
            "create `a/x.ps1`",
            &format!(
                "**Verify:**\n\n{}{expected}\n",
                fence("powershell", "cargo test")
            ),
        )
    };
    let cases = [
        ("Expected: nothing fails.", "run Unknown"),
        ("Expected: it doesn't fail.", "run Unknown"),
        ("Expected: the build fails.", "run Failure"),
        ("Expected:\n\n- no test fails", "run Unknown"),
        ("Expected:\n\n- the build fails", "run Failure"),
    ];
    for (expected, label) in cases {
        assert_eq!(
            labels(&ok("devready", &verify(expected))),
            ["note", label],
            "{expected}"
        );
    }
}

#[test]
fn a_negated_failure_is_no_expected_failure_in_superpowers() {
    let run = |expected: &str| {
        format!(
            "### Task 1: a\n\n- [ ] **Step 1: Run the tests**\n\nRun: `cargo test`\n{expected}\n"
        )
    };
    let cases = [
        ("Expected: no test fails", "run Unknown"),
        (
            "Expected: FAIL with \"cannot find function\"",
            "run Failure",
        ),
    ];
    for (expected, label) in cases {
        assert_eq!(
            labels(&ok("superpowers", &run(expected))),
            [label],
            "{expected}"
        );
    }
}

#[test]
fn a_negated_failure_is_no_expected_failure_in_before_after() {
    let check = |expected: &str| {
        format!(
            "## Task 1: a\n\n- [ ] **Step 1: Check**\n\n{}{expected}\n",
            fence("powershell", "cargo test")
        )
    };
    let cases = [
        ("Expected: nothing fails.", "run Unknown"),
        ("Expected: build error.", "run Failure"),
    ];
    for (expected, label) in cases {
        assert_eq!(
            labels(&ok("before-after", &check(expected))),
            [label],
            "{expected}"
        );
    }
}

const WRITE_SPEC: &str = r#"
id = "w"
name = "write-mode test"
[detect]
patterns = [{ regex = '^### Task', weight = 60 }]
[outline]
task = { level = 3, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
[outline.manifest]
marker = '^\*\*Files\*\*$'
item = '^(?P<change>Create|Modify|Test) `(?P<paths>[^`]+)`$'
changes = { Create = "create", Modify = "modify", Test = "test" }
required = false
[checks]
manifest-untouched = "off"
manifest-undeclared = "warning"
[[recognizer]]
kind = "file-block"
id = "file-block#w"
lead = '^File `(?P<path>[^`]+)` \((?P<op>full contents)\):$'
ops = { "full contents" = "write" }
"#;

/// The error codes of parsing `text` with [`WRITE_SPEC`]; empty when it parses.
fn write_codes(text: &str) -> Vec<&'static str> {
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    registry
        .add_spec("w", WRITE_SPEC)
        .unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, "w.md", text.as_bytes(), Some("w"))
        .err()
        .map(|d| d.error_codes())
        .unwrap_or_default()
}

fn write_task(files: &str) -> String {
    format!(
        "### Task 1: a\n\n{files}File `a.txt` (full contents):\n\n{}",
        fence("text", "A")
    )
}

#[test]
fn an_undeclared_write_without_a_mode_is_an_error_in_every_format() {
    // No built-in format can produce it (assay requires a manifest and makes
    // `manifest-undeclared` an error; the others never use the `write` op), hence this spec.
    assert_eq!(write_codes(&write_task("")), ["write-mode-unknown"]);
    let undeclared = write_task("**Files**\n\n- Create `b.txt`\n\n");
    assert_eq!(write_codes(&undeclared), ["write-mode-unknown"]);
    let declared = write_task("**Files**\n\n- Create `a.txt`\n\n");
    assert!(write_codes(&declared).is_empty());
    let test_entry = write_task("**Files**\n\n- Test `a.txt`\n\n");
    assert_eq!(write_codes(&test_entry), ["manifest-write-mode"]);
}
```

File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p3-t1-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index 2f248a1..dc2bc03 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -43,6 +43,9 @@ Relative, forward slashes, no `..`/`.`/empty segments, no drive letter, no backs
 Symlink escapes are refused by the workspace at write time. Nothing outside the repository is
 written except under `.git/planrun/` and the OS temp directory.
 
+Amended by D42 (wildcards), D51 (Windows-reserved spellings) and D52 (pre-images in the object
+database, the writer's temp file).
+
 ## D8 Formats are data first; built-ins are specs too
 
 Three extension levels: TOML spec (no rebuild) → recognizer kind (Rust) → Format (Rust). The
@@ -59,7 +62,7 @@ Each dependency is justified here; add none without a line.
 | `serde`, `toml` | format specs |
 | `serde_json` | the journal (JSONL) |
 | `thiserror` | error types |
-| `sha1_smol` | git blob ids in-process: the `blob-chain` check from a created file's contents (planrun-parse); the fast path in planrun-exec, where `git hash-object` is the authority |
+| `sha1_smol` | git blob ids in-process: the `blob-chain` check from a created file's contents (planrun-parse only; planrun-exec asks `git hash-object`, D56) |
 | `similar` | showing diffs of overwrites and replacements. **Not** for applying patches (D20) |
 | `clap` | arguments |
 | `dialoguer` | line prompts (D15) |
@@ -76,6 +79,8 @@ Created files: as in the plan (LF). Overwrite and `ReplaceExact`: keep the targe
 convention unless `.gitattributes` says `eol=lf`; `before` is matched in LF space. Patches: through
 `git apply` unchanged.
 
+Amended by D54 (attributes, files without or with mixed line endings).
+
 ## D11 Prose never passes a script on its own
 
 `Expected: all pass` is mapped to an exit expectation, checked, and then shown for confirmation.
@@ -143,6 +148,8 @@ when every verification of the task passed or was skipped by policy (D12).
 All file actions of a step, up to the next non-file action, are validated in memory before any is
 written; pre-images are stored in the git object database first; writes are temp-file + rename.
 
+Amended by D52 (the virtual tree, the writer) and D53 (prepare, then commit).
+
 ## D24 One persistent session per shell kind per run
 
 Variables and the working directory persist across blocks (`$gate`, `$P2`), as in Rens' terminal.
@@ -321,6 +328,8 @@ was chosen instead, so "no write targets a wildcard" is a property of the types.
 Amends D7: a `RepoPath` also refuses `*` and `?`. Both are invalid in Windows file names
 (D19), and a path with one is a pattern (D41). A lead that names such a path is `bad-path`.
 
+Amended by D51.
+
 ## D43 Inline Files clauses
 
 A manifest may be written inside its marker paragraph: ``**Files:** create `a`; modify `b`.``
@@ -366,7 +375,7 @@ is unknown.
 - `show` marks Bare entries. `AmbiguousPath` is produced only by this rule; it is not a
   `manual-prose` reason.
 
-  ## D46 Step labels that stay in their step
+## D46 Step labels that stay in their step
 
 `[[outline.steps]] keep = true`: the boundary block is the first block of its step instead of
 being consumed by the outline. Hand-written plans put the step's lead in its label
@@ -409,4 +418,179 @@ number `n - 1`: it edits that path, with `PathOrigin::PreviousEdit`.
 - The spec load refuses `series` on a lead that does not capture `n`.
 - The previous edit's number is read from its lead paragraph by `series`, because a
   recognizer cannot see another recognizer's captures (recognizers are pure and independent,
-  extending.md §5); the edit with the path is usually matched by another recognizer instance.
\ No newline at end of file
+  extending.md §5); the edit with the path is usually matched by another recognizer instance.
+
+Amended by D59 (a leading word names the file).
+
+Decisions D50 onwards were agreed on 2026-11-25 at the start of P3, from the review of the
+execution design (workspace, git, dry run) against the IR that P2 left.
+
+## D50 Unscoped file actions are confirmed
+
+A file action (WriteFile, ApplyPatch, ReplaceExact, Delete) is scoped when its task has a
+manifest and either a file entry equal to its path whose origin is not Bare, or a pattern entry
+that matches it (D41). Any other file action is unscoped: in a task without a manifest
+(`required = false`), at a path no entry covers (`manifest-undeclared` downgraded, D31), or at
+a path only a Bare entry names (D45).
+
+- An unscoped file action never runs automatically: the runner shows its op, path, PathOrigin
+  and plan line and asks, whatever `--auto` says. Preflight checks it like any other action;
+  the dry run reports it as "would ask". P3 computes scope; P5 adds the prompt.
+- An action is Bare-targeted when some Bare file entry of its task equals its path. A
+  Bare-targeted WriteFile Overwrite or Delete is `path-ambiguous`, a preflight error in every
+  format: nothing in the action confirms the target, where an ApplyPatch has `pre` and a
+  ReplaceExact has "`before` exactly once". A Bare-targeted ApplyPatch or ReplaceExact stays an
+  action; it is unscoped, and a missing target is `target-missing` with the D45 hint. A path
+  with both a Written and a Bare entry is scoped and Bare-targeted at once: conservative, and
+  loud.
+- Paths resolved by D32 and D49 (FileName, OnlyEntry, PreviousEdit) are scoped: each resolves
+  only to an entry written in full.
+- Scope and Bare-targeting are `Manifest` methods in planrun-model, used by parse (D45's
+  `Manual(AmbiguousPath)`) and exec alike. Scope is computed from the IR, never read from a
+  spec: no format spec, `[checks]` entry or `--auto` value switches D50 off (as D31).
+
+Actions never carry `PathOrigin::Bare`: the manifest sets it on entries only, and a lead that
+names a bare file name resolves to `Written`. So the rule is keyed on the manifest, not on the
+action's origin.
+
+## D51 RepoPath refuses Windows-reserved spellings
+
+Amends D7 and D42. Besides absolute paths, `..`, `.`, empty segments, backslashes and
+wildcards, `RepoPath` refuses, as `bad-path` in `check`:
+
+- `:` anywhere (an NTFS alternate data stream: `a.rs:x`), `<`, `>`, `"`, `|`, and the control
+  characters 0x00–0x1F;
+- a segment that equals `.git` after ASCII case-folding and stripping trailing dots and spaces
+  (`.GIT`, `.git.`, `.git `), and its 8.3 alias `git~1` (CVE-2014-9390): a plan never writes
+  into a git directory, hooks above all;
+- a Windows device name as a segment, with or without an extension, in any case: `con`, `prn`,
+  `aux`, `nul`, `com0`–`com9`, `lpt0`–`lpt9`, the superscript forms `com¹`–`com³` and
+  `lpt¹`–`lpt³`, `conin$`, `conout$`;
+- a segment that ends in a dot or a space.
+
+`.gitattributes`, `.gitignore` and `.gitmodules` stay writable.
+
+## D52 The virtual tree, pre-images and the writer
+
+Amends D7 and D23; corrects architecture.md §6.2 and §7.
+
+- **Virtual tree.** A temporary index file and object directory in a fresh directory under the
+  OS temp directory, with the repository's objects as alternates. Its git calls get
+  `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY` and `GIT_ALTERNATE_OBJECT_DIRECTORIES` on their own
+  `Command`; no other git call inherits them, and nothing in the repository is written. A path
+  is seeded from the working tree on first use (`hash-object -w --path`). The tree tells apart
+  a path not seeded yet, one absent on disk, one deleted in this run, and one whose state a
+  script that did not run may have changed (dry run, D55). Patches are applied to it with
+  `git apply --cached` (D20 holds: git applies them), so preimage and postimage ids are
+  compared in git's normalized space, with every filter and `autocrlf` applied by git.
+- **Path identity.** With `core.ignorecase=true`, paths are compared case-insensitively in the
+  virtual tree and within a step: two spellings of one path in a run are `path-collision`. A
+  plan path whose existing segments differ in case from the disk is `path-case`.
+- **Pre-images** of every path a step changes are stored in the repository's object database
+  with `git hash-object -w --no-filters` (the exact bytes, CRLF and untracked files included)
+  before anything is written. This id differs from the filtered blob id the checks use.
+- **The writer.** Every write goes through one writer, which resolves the path first: the
+  deepest existing ancestor, canonicalized, must equal the repository root plus the same
+  segments, compared exactly. That refuses symlinks, junctions, 8.3 short names and case
+  mismatches. Directories a write needs are created one segment at a time, each checked. The
+  contents go to `.planrun-<random>.tmp`, created with `create_new` in the target's directory;
+  the parent is checked again directly before the rename, and the rename replaces the target
+  (a symlink placed there is replaced, never written through). The temp file is the one thing
+  besides the plan's own writes that appears in the working tree, and only until the rename.
+- **What is not closed:** a parent directory swapped for a link between the second check and
+  the rename. planrun is no defence against a process racing it in the same tree.
+- Status calls run as `git --no-optional-locks status --porcelain -z`, so even they leave
+  `.git/index` alone.
+
+## D53 A step transaction is prepare, then commit
+
+Amends D23.
+
+- **prepare** validates a step's file actions on a virtual tree seeded from the disk, stores
+  the pre-images, and builds every post-image there: patches with `git apply --cached`, the
+  other actions as D54 bytes (`hash-object -w --path`). It returns a `PreparedStep`: per path
+  the pre-image id (or "absent"), the post-image bytes (or "delete"), and the directories it
+  will create. The bytes of a patched file are `git cat-file --filters --path=<p> <blob>`, the
+  same conversion to the working tree that `git apply` uses.
+- **commit** writes the prepared post-images with the one writer (D52), in order. It first
+  hands the `PreparedStep` to a write-ahead recorder; it cannot write without one. P5's recorder
+  is the journal; a real run without that record is not allowed. P3's CLI has only `--dry-run`,
+  so nothing in P3 commits outside the tests.
+- **Rollback:** an error during commit restores every path already written from its pre-image
+  (exact bytes), removes files it created and the directories it created (deepest first, when
+  empty), and reports any path it could not restore.
+- **The index:** a `Delete` with `via_git` also removes the path from the index, as one
+  `git update-index --force-remove` call after every working-tree write succeeded. It is the
+  last effect of the step and writes the index once, under git's lock: it either happens whole
+  or not at all, and when it fails the working-tree writes are rolled back.
+- `ActionHandler` gets `simulate` (the dry-run behaviour of extending.md §4) and no per-kind
+  write: writing is the transaction's job.
+
+## D54 Line endings of written files, refined
+
+Amends D10. For an Overwrite and a ReplaceExact target, in this order:
+
+1. An `eol` attribute (`lf` or `crlf`) decides. `-text` (or `binary`) means the bytes are used
+   unchanged: an Overwrite writes the plan's bytes, a ReplaceExact matches raw bytes.
+2. Otherwise the file decides: CRLF when it has at least one CRLF and no bare LF, LF when it
+   has at least one LF and no CRLF.
+3. A file with no line ending at all (empty, or one line): git's checkout convention for that
+   path (`core.eol`, `core.autocrlf`, attributes), found by converting a one-line blob with
+   `git cat-file --filters --path=<p>`. That is what a fresh checkout would produce.
+4. Mixed: an Overwrite uses the checkout convention (the whole file is replaced); a ReplaceExact
+   is `eol-mixed`, a preflight error, because a one-line edit must not rewrite every line
+   ending in the file.
+
+`before` is matched in LF space (D10). Created files stay LF. Attributes are read once, at the
+start of a step; a step that writes `.gitattributes` changes the attributes of later steps
+only, and the report says so.
+
+## D55 The dry run
+
+- It parses, checks, and preflights against one virtual tree carried across every task of the
+  run, so a task sees the files the tasks before it wrote. Commits are not run; the dry run
+  assumes each task's commit happens.
+- Real dirt is checked once, at the start, for the paths every task of the run touches: a dirty
+  or untracked touched path is `path-dirty` (an error); other dirt is a warning.
+- Scripts are not run, and neither are Manual steps (a prose edit changes a file by hand). Any
+  of them can touch any path, so the rule is plain: once a script, commit or Manual that did not
+  run comes earlier in the run, a failing file action is a warning, "depends on a step that did
+  not run", not a failure, and its path is unknown in the virtual tree from then on; later
+  actions on it get the same label. Only failures of the tree's state are downgraded (a missing
+  or existing target, a blob or patch mismatch, `before` not unique, `eol-mixed`); a bad path,
+  `path-ambiguous` or `write-mode-unknown` stays an error. Every ok after that point assumes
+  those steps change no files (`cargo fmt` does). The report says so in its header, and its
+  summary counts these warnings. A failure before any of them is a failure. Exit 0 when only
+  warnings remain.
+- `Generated` is "not checked" (its script did not run), a `Manual` "waits", an unscoped file
+  action "would ask" (D50).
+
+## D56 No in-process blob ids in planrun-exec
+
+Amends D9; corrects architecture.md §6.2. planrun-exec has no SHA-1 fast path: knowing that no
+filter applies costs about as much as one `git hash-object`, and D25 rules out speculative speed
+work. `sha1_smol` stays for planrun-parse's `blob-chain` check only.
+
+## D57 Pre-image objects can be pruned
+
+The pre-images D52 stores are unreachable objects: `git gc` may prune them once they are older
+than `gc.pruneExpire` (two weeks by default). P5 decides whether a resume past that window is
+detected (the object is gone: the user decides, D14) or whether the objects are pinned (for
+example by a ref under `refs/planrun/`). Recorded now, not solved in P3.
+
+## D58 A write without a mode is an error
+
+A `WriteFile` whose mode is still `FromManifest` at a path no manifest entry declares (a task
+without a manifest, or an undeclared path where `manifest-undeclared` is only a warning) is
+`write-mode-unknown`, a check error in every format. No built-in format produces one: assay
+requires a manifest and keeps `manifest-undeclared` an error, and the other formats never use
+the `write` op. Where `manifest-undeclared` is already an error on the action, it is reported
+once. planrun-exec refuses a `FromManifest` write too, as an error that never prompts.
+
+## D59 An edit series' leading word names its file
+
+Amends D49. A `before-after` lead without a path may capture the word before "edit" as `stem`
+(`README edit 2 (old → new):`). The path the lead continues (D49) or infers (D32) must then have
+that file-name stem, the file name without its last extension (`README` of `README.md`),
+compared exactly; otherwise the series does not continue, and with no other agreeing path the
+edit is `Manual(ProseEdit)`. The spec load refuses `stem` on a lead that captures `path`.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p3-t1-decisions-md.patch"
Remove-Item "$env:TEMP\p3-t1-decisions-md.patch"
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
| `before-after` | lead (naming a path), fence, *separator paragraph*, fence | `ReplaceExact`, or `Manual(ProseEdit)` when `infer_path` finds no single entry | `lead` (captures `path`, or none with `infer_path = true`), *`separator`* (e.g. `^After:$`; absent = two fences back to back), *`file_name_from_manifest`*, *`series`* (a regex capturing `n`; the lead then captures `n` too, D49); a pathless lead may capture *`stem`* (D59) |
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
  before it in the step (`PathOrigin::PreviousEdit`), before `infer_path` is tried. A
  pathless lead's `stem` capture (`README edit 2`) must equal the file-name stem of the path
  it continues or infers, or the edit is `Manual(ProseEdit)` (D59).
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
| Edit series | `Edit 2 (old → new):` directly after the ReplaceExact of edit 1 edits the same path (D49); otherwise the task's one editable entry, else `Manual(ProseEdit)`. A word before "edit" (`README edit 2`) names the file: the path must have that stem (D59) |
| Edit, Old/New | ``**2c.** `p`. Old:``, ``README.md. Old:``, ``**2d.** `p`: insert … Old (end of file):`` + fence + `New:` + fence → `ReplaceExact`; `Old:`, `(2) Old:`, ``In `Member`. Old:`` take the task's one editable entry, else `Manual(ProseEdit)` |
| Prose edits | inline `` Old: `a` `` / `` New: `b` ``, `` Edit: `a` → `b` `` (D34), `New:` + fence after an inline Old, "append …:", "Insert directly before …:" + fence → `Manual(ProseEdit)` |
| Delete | `` `git rm p` ``, ``**4g.** `git rm p` `` → `Delete { via_git: true }` |
| Scripts | a shell fence runs only behind an allowlisted lead: a Verify label, `**Run + verify:**`, `**Commit:**` / `Commit:`, `**4l. Generate and verify.**`, a trailing `(Bash tool):`. Never behind a bare sub-label (D28, D46); any other shell fence is `Manual(Unrecognized)`. `-DryRun` / `-All` bodies are Preview / Apply |
| Not run | ``… (run from `$REPO`) …:`` + fence (wherever the parenthesis stands in the lead, so a later `(Bash tool):` does not claim the fence) and "Then the same command with `-All` instead of `-DryRun`. Verify:" + fence → `Manual(HumanCheck)`: a script has no working directory of its own, and the second fence uses the first one's variables |
| Negated failures | `Expected: nothing fails`, `… doesn't fail`: a negation before "fail" or "build error" maps to `any`, never to an expected failure (D12 would skip the script); the same row leads the superpowers and before-after tables |
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
  # A negated failure ("nothing fails", "doesn't fail") is no expected failure: checked by you.
  { text = '''(?i)(?:\b(?:no|not|nothing|none|never|without)\b|n't\b)[^.;\n]*\b(?:fail|build error)''', exit = "any" },
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
# `README edit 2 (old → new):` edits only a path whose file-name stem is `README`.
lead = '^(?:(?P<stem>\w+) )?[Ee]dit (?P<n>\d+) \(old → new(?:; [^)]*)?\):$'
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
# Not end-anchored: `(run from `x`) (Bash tool):` must not reach the script lead.
lead = '\(run from `[^`]+`\)'
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
  # A negated failure ("nothing fails", "doesn't fail") is no expected failure: checked by you.
  { text = '''(?i)(?:\b(?:no|not|nothing|none|never|without)\b|n't\b)[^.;\n]*\b(?:fail|build error)''', exit = "any" },
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
  # A negated failure ("nothing fails", "doesn't fail") is no expected failure: checked by you.
  { text = '''(?i)(?:\b(?:no|not|nothing|none|never|without)\b|n't\b)[^.;\n]*\b(?:fail|build error)''', exit = "any" },
  { text = '(?i)\b(?:fails?|build errors?)\b', exit = "failure" },
]
```

File `formats/superpowers.toml` (full contents):

```toml
# The superpowers format: plans written by the writing-plans skill (docs/plan-grammar.md §3).
# Hand-written, partly automatable: full-file writes, exact before/after edits, run lines and
# commit fences run; edits described in prose are Manual. Every regex is matched against raw
# block text (inline markup kept), anchored per block.

id = "superpowers"
name = "Superpowers plan (writing-plans skill)"

[detect]
patterns = [
  { regex = '^### Task \d+: ', weight = 40 },
  { regex = '^- \[ \] \*\*Step \d+: ', weight = 40 },
  { regex = '^Run: `', weight = 20 },
]

[outline]
task = { level = 3, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
# `## Self-Review` and the like end the tasks.
tasks_end_level = 2

[outline.manifest]
marker = '^\*\*Files:\*\*$'
# - Create: `p`   - Modify: `p:3-4` (the `X` record)   - Test: `p`
item = '^(?P<change>Create|Modify|Test): `(?P<paths>[^`]+)`(?: \(.+\))?$'
changes = { Create = "create", Modify = "modify", Test = "test" }
required = false

[[outline.steps]]
block = "list-item"
regex = '^\*\*Step (?P<n>\d+): (?P<title>.+)\*\*$'

# A step title that says the step fails or is checked by eye marks the whole step (D30).
[[step_marks]]
title = '(?i)\bverify (?:it|they) fails?\b'
mark = "expected-failure"

[[step_marks]]
title = '(?i)\b(?:manual verification|check by eye|by eye)\b'
mark = "human-check"

[settings]
manual_markers = ['(?i)\bby hand\b', '(?i)\bRens runs\b', '(?i)\bmanual step\b']

# Prose edits change declared files, so "declared, untouched" is not a finding here (D31).
[checks]
manifest-untouched = "off"
manifest-undeclared = "warning"

# --- Recognizers, tried in this order at each position of a step -------------------------------

[[recognizer]]
kind = "file-block"
id = "file-block#superpowers"
# Create `p` with only the test module:   Replace the entire contents of `p` with:
lead = '^(?P<op>Create|Replace the entire contents of) `(?P<path>[^`]+)`(?: with\b[^:]*)?:$'
ops = { "Create" = "create", "Replace the entire contents of" = "overwrite" }

[[recognizer]]
kind = "before-after"
id = "before-after#change-to"
# In `p`, line 134, change:  + fence + to: + fence
lead = '^In `(?P<path>[^`]+)`, lines? [\d~-]+, change:$'
separator = '^to:$'

[[recognizer]]
kind = "run-line"
id = "run-line#superpowers"
# Run: `cmd`, with `Expected: …` on the next line of the same paragraph (D39: pwsh).
lead = '^Run: `(?P<cmd>[^`]+)`(?:\nExpected: (?P<expected>.+))?$'
shell = "pwsh"
exit = [
  # A negated failure ("nothing fails", "doesn't fail") is no expected failure: checked by you.
  { text = '''(?i)(?:\b(?:no|not|nothing|none|never|without)\b|n't\b)[^.;\n]*\b(?:fail|build error)''', exit = "any" },
  { text = '(?i)\b(fail|build error|does not exist)', exit = "failure" },
  { text = '(?i)\b(pass(es)?|succeeds?|succeeded)\b', exit = "success" },
]

[[recognizer]]
kind = "manual-prose"
id = "manual-prose#edit"
# Append to `Class`:   In `p`, replace … with:   Above the test module in `p`, add:
lead = '^(?:Append|Above|Below|In|Add|At the top of|Replace|And change|Change)\b.*:$'
reason = "prose-edit"
takes_fence = true

[[recognizer]]
kind = "script"
id = "script#superpowers"
# Only the commit fences run: the first block of a written step, or behind `Run:` (D28).
leads = ['^Run:$']
first_block = true
intents = [{ intent = "commit", body = '(?m)\bgit commit\b' }]

[[recognizer]]
kind = "expected-prose"
id = "expected-prose#superpowers"
lead = '^Expected: (?P<text>.+)$'
exit = [
  # A negated failure ("nothing fails", "doesn't fail") is no expected failure: checked by you.
  { text = '''(?i)(?:\b(?:no|not|nothing|none|never|without)\b|n't\b)[^.;\n]*\b(?:fail|build error)''', exit = "any" },
  { text = '(?i)\b(fail|build error|does not exist)', exit = "failure" },
  { text = '(?i)\b(pass(es)?|succeeds?|succeeded)\b', exit = "success" },
]
```

**Step 2 — verify**

Targeted: the touch-up tests:

```powershell
cargo test -p planrun-parse --test spec_touchups
```

Expected (exit code 0; summary lines):

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-parse/src/check.rs crates/planrun-parse/src/kinds/before_after.rs crates/planrun-parse/src/lib.rs crates/planrun-parse/tests/spec_touchups.rs docs/decisions.md docs/extending.md docs/plan-grammar.md formats/before-after.toml formats/devready.toml formats/superpowers.toml
  $verified = [ordered]@{
      'crates/planrun-parse/src/check.rs' = '0a16a13886f4f395d7321cba3dc928f8c78816fa'
      'crates/planrun-parse/src/kinds/before_after.rs' = '4c4de18f554a5ce20313dddd4001852f9eb091cb'
      'crates/planrun-parse/src/lib.rs' = '6db17587d05357cbb7ae11241967bee584544a4d'
      'crates/planrun-parse/tests/spec_touchups.rs' = '32a43b80d66766afbf5bb045b70e2407596162ac'
      'docs/decisions.md' = 'dc2bc03858ac2d8e52861b3aaebf1fedf1ba842a'
      'docs/extending.md' = '37c4256570657d61346c37f65d9119686a895d36'
      'docs/plan-grammar.md' = '7f9f612e902a1d4503c0ae63d9d30cf2a167413c'
      'formats/before-after.toml' = 'acaaabbf9205e3a869ff56840913b61e44c2c548'
      'formats/devready.toml' = '5dfdb2046bc2e959ac122d85ae69721599cafd0c'
      'formats/superpowers.toml' = '4b9186a2b67f029548169bef6585b056d9c0cc51'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P3-T1: edit-series stems (D59), run-from anywhere in its lead, negated failures, write-mode-unknown (D58), decisions D50-D59" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P3-T2: RepoPath refuses Windows-reserved spellings (D51), Manifest scope and Bare-target predicates (D50)

**Goal:** D51 and D50's model half. `RepoPath::new` refuses `:` and the other Windows-reserved characters, a `.git` segment in any spelling and `git~1`, device names, and segments ending in a dot or a space, each with its own error variant and unit tests. `Manifest::scopes` and `Manifest::is_bare_target` decide scope and Bare-targeting; parse's `is_bare` calls the second. A new invalid-corpus file writes `.git/hooks/pre-commit`.

**Files**

- Modify `crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap`
- Modify `crates/planrun-model/src/path.rs`
- Modify `crates/planrun-model/src/plan.rs`
- Modify `crates/planrun-parse/src/kinds/mod.rs`
- Modify `crates/planrun-parse/tests/invalid_corpus.rs`
- Modify `docs/plan-grammar.md`
- Modify `tests/corpus/invalid/README.md`
- Create `tests/corpus/invalid/bad-path.git-hooks.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap`, `crates/planrun-model/src/path.rs`.

**Step 1 — write the files**

File `crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap` (exact diff; save the block as `$env:TEMP\p3-t2-cli-check-invalid-corpus-snap.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap b/crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap
index 087affc..90faf6c 100644
--- a/crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap
+++ b/crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap
@@ -2,6 +2,15 @@
 source: crates/planrun-cli/tests/cli.rs
 expression: all
 ---
+==== bad-path.git-hooks.md
+error[bad-path]: `.git/hooks/pre-commit` names a git directory; a plan never writes into one
+  --> tests/corpus/invalid/bad-path.git-hooks.md:14:11
+   |
+14 | - Create `.git/hooks/pre-commit`
+   |           ^^^^^^^^^^^^^^^^^^^^^
+
+planrun: tests/corpus/invalid/bad-path.git-hooks.md: 1 error
+
 ==== bad-path.md
 error[bad-path]: `../outside.rs` has a `.`, `..` or empty segment
   --> tests/corpus/invalid/bad-path.md:14:11
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p3-t2-cli-check-invalid-corpus-snap.patch"
Remove-Item "$env:TEMP\p3-t2-cli-check-invalid-corpus-snap.patch"
```

File `crates/planrun-model/src/path.rs` (exact diff; save the block as `$env:TEMP\p3-t2-path-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-model/src/path.rs b/crates/planrun-model/src/path.rs
index ff4716d..b49c334 100644
--- a/crates/planrun-model/src/path.rs
+++ b/crates/planrun-model/src/path.rs
@@ -6,7 +6,11 @@ use serde::{Deserialize, Serialize};
 
 /// A path relative to the repository root, with forward slashes, that cannot leave the
 /// repository by its spelling: no absolute path, no drive letter, no `..`, no `.` segment, no
-/// empty segment, no backslash, no NUL, no wildcard (`*`, `?`: decision D42).
+/// empty segment, no backslash, no NUL, no wildcard (`*`, `?`: decision D42). Nor a spelling
+/// Windows reserves or git refuses (decision D51): no `:`, `<`, `>`, `"`, `|` or control
+/// character, no segment that is `.git` (in any case, with trailing dots or spaces) or its 8.3
+/// alias `git~1`, no device name (`con`, `nul.txt`, `com1`, …), no segment ending in a dot or a
+/// space.
 ///
 /// This checks the spelling only. Symlinks that point out of the repository are refused by the
 /// workspace when it resolves the path on disk (planrun-exec).
@@ -35,6 +39,19 @@ pub enum RepoPathError {
     /// A `*` or `?`: a pattern, never a file (decisions D41, D42).
     #[error("`{0}` contains a wildcard; a pattern is not a file path")]
     Wildcard(String),
+    /// `:`, `<`, `>`, `"`, `|` or a control character (decision D51).
+    #[error("`{0}` contains a character Windows reserves in file names")]
+    Reserved(String),
+    /// A segment that is a git directory: `.git` in any case, with trailing dots or spaces, or
+    /// `git~1` (decision D51).
+    #[error("`{0}` names a git directory; a plan never writes into one")]
+    GitDir(String),
+    /// A segment that is a Windows device name, with or without an extension (decision D51).
+    #[error("`{0}` has a segment Windows reserves as a device name")]
+    DeviceName(String),
+    /// A segment that ends in a dot or a space, which Windows drops (decision D51).
+    #[error("`{0}` has a segment that ends in a dot or a space")]
+    TrailingDotOrSpace(String),
 }
 
 impl RepoPath {
@@ -56,11 +73,11 @@ impl RepoPath {
         if path.starts_with('/') || (bytes.len() >= 2 && bytes[1] == b':') {
             return Err(RepoPathError::Absolute(path.to_owned()));
         }
-        if path
-            .split('/')
-            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
-        {
-            return Err(RepoPathError::BadSegment(path.to_owned()));
+        if path.contains(|c: char| matches!(c, ':' | '<' | '>' | '"' | '|') || c < ' ') {
+            return Err(RepoPathError::Reserved(path.to_owned()));
+        }
+        for segment in path.split('/') {
+            check_segment(path, segment)?;
         }
         Ok(Self(path.into()))
     }
@@ -81,6 +98,50 @@ impl RepoPath {
     }
 }
 
+/// Refuses a segment that is empty, `.` or `..`, a git directory, a device name, or ends in a
+/// dot or a space.
+fn check_segment(path: &str, segment: &str) -> Result<(), RepoPathError> {
+    if segment.is_empty() || segment == "." || segment == ".." {
+        return Err(RepoPathError::BadSegment(path.to_owned()));
+    }
+    let trimmed = segment.trim_end_matches(['.', ' ']).to_ascii_lowercase();
+    if trimmed == ".git" || trimmed == "git~1" {
+        return Err(RepoPathError::GitDir(path.to_owned()));
+    }
+    if is_device_name(segment) {
+        return Err(RepoPathError::DeviceName(path.to_owned()));
+    }
+    if segment.ends_with(['.', ' ']) {
+        return Err(RepoPathError::TrailingDotOrSpace(path.to_owned()));
+    }
+    Ok(())
+}
+
+/// True for `con`, `prn`, `aux`, `nul`, `conin$`, `conout$`, `com0`–`com9`, `lpt0`–`lpt9` and
+/// the superscript forms `com¹`–`com³`, `lpt¹`–`lpt³`, in any case, alone or before an extension
+/// (`nul.txt`, `con .rs`): Windows opens the device instead of the file.
+fn is_device_name(segment: &str) -> bool {
+    let base = segment.split('.').next().unwrap_or(segment);
+    let base = base.trim_end_matches(' ').to_lowercase();
+    if matches!(
+        base.as_str(),
+        "con" | "prn" | "aux" | "nul" | "conin$" | "conout$"
+    ) {
+        return true;
+    }
+    let Some(number) = base
+        .strip_prefix("com")
+        .or_else(|| base.strip_prefix("lpt"))
+    else {
+        return false;
+    };
+    let mut chars = number.chars();
+    let digit = chars
+        .next()
+        .is_some_and(|c| c.is_ascii_digit() || matches!(c, '¹' | '²' | '³'));
+    digit && chars.next().is_none()
+}
+
 /// Where a path in the IR came from, when the plan did not write it in full (decision D32).
 #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
 pub enum PathOrigin {
@@ -163,4 +224,106 @@ mod tests {
             assert!(RepoPath::new(bad).is_err(), "{bad:?}");
         }
     }
+
+    fn refused(path: &str) -> Option<RepoPathError> {
+        RepoPath::new(path).err()
+    }
+
+    #[test]
+    fn reserved_characters_are_refused() {
+        for bad in [
+            "a.rs:x",
+            "src/a:b.rs",
+            "a<b",
+            "a>b",
+            "a\"b",
+            "a|b",
+            "a\tb",
+            "a\u{1f}b",
+        ] {
+            assert_eq!(
+                refused(bad),
+                Some(RepoPathError::Reserved(bad.to_owned())),
+                "{bad:?}"
+            );
+        }
+    }
+
+    #[test]
+    fn git_directories_are_refused_in_any_spelling() {
+        for bad in [
+            ".git/hooks/pre-commit",
+            ".GIT/config",
+            "a/.Git/x",
+            ".git./x",
+            ".git /x",
+            ".git. ./x",
+            "git~1/config",
+            "a/GIT~1/x",
+        ] {
+            assert_eq!(
+                refused(bad),
+                Some(RepoPathError::GitDir(bad.to_owned())),
+                "{bad:?}"
+            );
+        }
+        for ok in [
+            ".gitattributes",
+            ".gitignore",
+            ".gitmodules",
+            "a/.github/x",
+            "git/x",
+        ] {
+            assert!(RepoPath::new(ok).is_ok(), "{ok:?}");
+        }
+    }
+
+    #[test]
+    fn device_names_are_refused_with_or_without_an_extension() {
+        for bad in [
+            "con",
+            "CON",
+            "a/prn.txt",
+            "aux.rs",
+            "Nul",
+            "nul.tar.gz",
+            "con .rs",
+            "com0",
+            "COM9",
+            "lpt1.log",
+            "com¹",
+            "LPT³.txt",
+            "conin$",
+            "CONOUT$.x",
+        ] {
+            assert_eq!(
+                refused(bad),
+                Some(RepoPathError::DeviceName(bad.to_owned())),
+                "{bad:?}"
+            );
+        }
+        for ok in [
+            "cons",
+            "console.rs",
+            "a/nullable.rs",
+            "com10",
+            "lpt",
+            "com",
+            "auxiliary",
+        ] {
+            assert!(RepoPath::new(ok).is_ok(), "{ok:?}");
+        }
+    }
+
+    #[test]
+    fn segments_ending_in_a_dot_or_a_space_are_refused() {
+        for bad in ["a.", "a /b", "dir./x", "x/y "] {
+            assert_eq!(
+                refused(bad),
+                Some(RepoPathError::TrailingDotOrSpace(bad.to_owned())),
+                "{bad:?}"
+            );
+        }
+        assert!(RepoPath::new("a.b/c").is_ok());
+    }
 }
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p3-t2-path-rs.patch"
Remove-Item "$env:TEMP\p3-t2-path-rs.patch"
```

File `crates/planrun-model/src/plan.rs` (full contents):

```rust
//! Plan, task, step and the task's file manifest.

use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::ids::{FormatId, TaskId};
use crate::path::{PathOrigin, RepoPath};
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

impl Manifest {
    /// Whether a file action at `path` is in the task's declared scope (decision D50): a file
    /// entry equal to it whose origin is not `Bare`, or a pattern entry that matches it. Always
    /// false for a task without a manifest.
    pub fn scopes(&self, path: &RepoPath) -> bool {
        self.entries.iter().any(|e| match &e.path {
            ManifestPath::File(file) => file == path && e.origin != PathOrigin::Bare,
            ManifestPath::Pattern(pattern) => pattern.matches(path),
        })
    }

    /// Whether some `Bare` file entry names `path` (decisions D45, D50): the plan wrote a file
    /// name whose directory it never said, so `path` may not be the file it means.
    pub fn is_bare_target(&self, path: &RepoPath) -> bool {
        self.entries
            .iter()
            .any(|e| e.origin == PathOrigin::Bare && e.path.file() == Some(path))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::PathPattern;

    fn path(p: &str) -> RepoPath {
        RepoPath::new(p).unwrap_or_else(|e| panic!("{e}"))
    }

    fn entry(path: ManifestPath, origin: PathOrigin) -> ManifestEntry {
        ManifestEntry {
            path,
            change: ChangeKind::Modify,
            lines: None,
            span: Span::new(0, 0),
            origin,
        }
    }

    fn manifest(entries: Vec<ManifestEntry>) -> Manifest {
        Manifest {
            entries,
            span: Some(Span::new(0, 0)),
        }
    }

    #[test]
    fn written_and_sibling_entries_and_patterns_scope_a_path() {
        let glob = PathPattern::glob("db/*.sql").unwrap_or_else(|e| panic!("{e}"));
        let m = manifest(vec![
            entry(ManifestPath::File(path("src/a.rs")), PathOrigin::Written),
            entry(ManifestPath::File(path("src/b.rs")), PathOrigin::Sibling),
            entry(ManifestPath::Pattern(glob), PathOrigin::Written),
        ]);
        for scoped in ["src/a.rs", "src/b.rs", "db/1.sql"] {
            assert!(m.scopes(&path(scoped)), "{scoped}");
        }
        for unscoped in ["src/c.rs", "db/x/1.sql", "a.rs"] {
            assert!(!m.scopes(&path(unscoped)), "{unscoped}");
        }
        assert!(!m.is_bare_target(&path("src/a.rs")));
    }

    #[test]
    fn a_bare_entry_targets_but_never_scopes() {
        let m = manifest(vec![
            entry(ManifestPath::File(path("src/a.rs")), PathOrigin::Written),
            entry(ManifestPath::File(path("b.ps1")), PathOrigin::Bare),
        ]);
        assert!(!m.scopes(&path("b.ps1")));
        assert!(m.is_bare_target(&path("b.ps1")));
        assert!(!m.is_bare_target(&path("src/b.ps1")));
    }

    #[test]
    fn a_path_with_a_written_and_a_bare_entry_is_both() {
        let m = manifest(vec![
            entry(ManifestPath::File(path("b.ps1")), PathOrigin::Written),
            entry(ManifestPath::File(path("b.ps1")), PathOrigin::Bare),
        ]);
        assert!(m.scopes(&path("b.ps1")));
        assert!(m.is_bare_target(&path("b.ps1")));
    }

    #[test]
    fn no_manifest_scopes_nothing() {
        let m = Manifest::default();
        assert!(!m.scopes(&path("a.rs")));
        assert!(!m.is_bare_target(&path("a.rs")));
    }
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

/// True when a `Bare` entry names `path` (decision D45; the model's test, shared with exec, D50).
fn is_bare(cx: &StepContext<'_>, path: &RepoPath) -> bool {
    cx.manifest.is_bare_target(path)
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

File `crates/planrun-parse/tests/invalid_corpus.rs` (full contents):

```rust
//! Every plan in `tests/corpus/invalid` fails with exactly the error code on its first line
//! (`<!-- expect: <code> -->`, tests/corpus/invalid/README.md). Codes are pinned, messages are
//! not.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use planrun_parse::{Registry, parse_plan};

fn invalid_plans() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/invalid");
    let mut plans: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap_or_else(|e| panic!("{e}")).path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .filter(|p| !p.ends_with("README.md"))
        .collect();
    plans.sort();
    plans
}

fn expected_code(text: &str) -> Option<&str> {
    text.lines()
        .next()?
        .strip_prefix("<!-- expect: ")?
        .strip_suffix(" -->")
}

#[test]
fn every_invalid_plan_yields_exactly_its_code() {
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let plans = invalid_plans();
    assert!(plans.len() >= 20, "only {} invalid plans", plans.len());
    let mut wrong = Vec::new();
    for path in &plans {
        let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let text = String::from_utf8_lossy(&bytes);
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let Some(expected) = expected_code(&text) else {
            wrong.push(format!("{name}: no `<!-- expect: … -->` first line"));
            continue;
        };
        let codes: BTreeSet<&str> = match parse_plan(&registry, name, &bytes, None) {
            Ok(_) => BTreeSet::new(),
            Err(d) => d.error_codes().into_iter().collect(),
        };
        if codes != BTreeSet::from([expected]) {
            wrong.push(format!("{name}: expected {{{expected}}}, got {codes:?}"));
        }
    }
    assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
}

/// `<code>.md`, or `<code>.<case>.md` for a second case of one code (`bad-path.git-hooks.md`).
#[test]
fn the_file_name_is_the_code() {
    for path in invalid_plans() {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
        let stem = path.file_stem().and_then(|s| s.to_str());
        let code = stem.map(|s| s.split_once('.').map_or(s, |(code, _)| code));
        assert_eq!(expected_code(&text), code, "{}", path.display());
    }
}
```

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
`bad-path`, and so is a spelling Windows reserves or git refuses (D51): `:`, `<`, `>`, `"`, `|`
or a control character; a `.git` segment in any spelling (`.GIT`, `.git.`, `git~1`); a device
name (`con`, `nul.txt`, `com1`); a segment ending in a dot or a space.

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
| Edit series | `Edit 2 (old → new):` directly after the ReplaceExact of edit 1 edits the same path (D49); otherwise the task's one editable entry, else `Manual(ProseEdit)`. A word before "edit" (`README edit 2`) names the file: the path must have that stem (D59) |
| Edit, Old/New | ``**2c.** `p`. Old:``, ``README.md. Old:``, ``**2d.** `p`: insert … Old (end of file):`` + fence + `New:` + fence → `ReplaceExact`; `Old:`, `(2) Old:`, ``In `Member`. Old:`` take the task's one editable entry, else `Manual(ProseEdit)` |
| Prose edits | inline `` Old: `a` `` / `` New: `b` ``, `` Edit: `a` → `b` `` (D34), `New:` + fence after an inline Old, "append …:", "Insert directly before …:" + fence → `Manual(ProseEdit)` |
| Delete | `` `git rm p` ``, ``**4g.** `git rm p` `` → `Delete { via_git: true }` |
| Scripts | a shell fence runs only behind an allowlisted lead: a Verify label, `**Run + verify:**`, `**Commit:**` / `Commit:`, `**4l. Generate and verify.**`, a trailing `(Bash tool):`. Never behind a bare sub-label (D28, D46); any other shell fence is `Manual(Unrecognized)`. `-DryRun` / `-All` bodies are Preview / Apply |
| Not run | ``… (run from `$REPO`) …:`` + fence (wherever the parenthesis stands in the lead, so a later `(Bash tool):` does not claim the fence) and "Then the same command with `-All` instead of `-DryRun`. Verify:" + fence → `Manual(HumanCheck)`: a script has no working directory of its own, and the second fence uses the first one's variables |
| Negated failures | `Expected: nothing fails`, `… doesn't fail`: a negation before "fail" or "build error" maps to `any`, never to an expected failure (D12 would skip the script); the same row leads the superpowers and before-after tables |
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

File `tests/corpus/invalid/README.md` (full contents):

````markdown
# Invalid corpus

Each `*.md` here must fail `planrun check` with **exactly one error**, whose code is the one on
the file's first line:

```
<!-- expect: <code> -->
<!-- why: <what is wrong, for the reader> -->
```

The test (`crates/planrun-parse/tests/invalid_corpus.rs`, Phase 1) walks this folder, parses each
file with the built-in registry, and asserts the set of error codes equals `{<code>}`. Codes are
pinned, messages are not (they are snapshot-tested separately, so a wording change is reviewed
but does not break this test).

Adding a check rule means adding a file here. Adding a file here never needs a code change.
A file is named after its code (`bad-path.md`); a second case of the same code adds a word
after a dot (`bad-path.git-hooks.md`).
````

File `tests/corpus/invalid/bad-path.git-hooks.md` (full contents):

````markdown
<!-- expect: bad-path -->
<!-- why: a write into the git directory: a hook would run on the next commit (D51) -->

# Invalid — bad-path

## Tasks

### P1-T1: a

**Goal:** a.

**Files**

- Create `.git/hooks/pre-commit`

**Step 1 — write the files**

File `.git/hooks/pre-commit` (full contents):

```sh
exit 0
```

````

**Step 2 — verify**

Targeted: the model's tests:

```powershell
cargo test -p planrun-model
```

Expected (exit code 0; summary lines):

```text
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap crates/planrun-model/src/path.rs crates/planrun-model/src/plan.rs crates/planrun-parse/src/kinds/mod.rs crates/planrun-parse/tests/invalid_corpus.rs docs/plan-grammar.md tests/corpus/invalid/README.md tests/corpus/invalid/bad-path.git-hooks.md
  $verified = [ordered]@{
      'crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap' = '90faf6c9d6c6a3f1dba00890f640efa5a3688fed'
      'crates/planrun-model/src/path.rs' = 'b49c334dc007822c9ef9812921cd3e019d76d785'
      'crates/planrun-model/src/plan.rs' = '1e5eecf7d73b974a969738be582af86146aa237c'
      'crates/planrun-parse/src/kinds/mod.rs' = 'cc0ea83a9b6bb6e8839eb215101197841f190657'
      'crates/planrun-parse/tests/invalid_corpus.rs' = '8a59a1a10646570ec43012e181a945da2dd31ac1'
      'docs/plan-grammar.md' = 'e6969468b56b69320f49dc9100d1dc0f354bc5f5'
      'tests/corpus/invalid/README.md' = 'e29be83abe20ea69e2ec7d119c13de9d5e6ff992'
      'tests/corpus/invalid/bad-path.git-hooks.md' = '9da2674dc527faf0b20f5b36e1ce3310f56ac7c5'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P3-T2: RepoPath refuses Windows-reserved spellings (D51), Manifest scope and Bare-target predicates (D50)" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P3-T3: the git wrapper and BlobId, the shared test fixture, sha1_smol out of planrun-exec (D56)

**Goal:** The test helper and the git wrapper. `tests/support/fixture.rs` makes a temporary git copy of the greeter fixture with an environment that keeps the user's git configuration out. `planrun-exec::git` wraps every git command P3 needs, one process each, with `GIT_LITERAL_PATHSPECS=1`; `BlobId` is a full SHA-1. The tests prove the blob ids are git's view with and without `core.autocrlf`, and that status and `apply --check` write nothing. `sha1_smol` leaves planrun-exec (D56).

**Files**

- Modify `Cargo.lock`
- Modify `crates/planrun-exec/Cargo.toml`
- Create `crates/planrun-exec/src/git.rs`
- Modify `crates/planrun-exec/src/lib.rs`
- Create `crates/planrun-exec/tests/git.rs`
- Create `tests/support/fixture.rs`

Over 300 lines, so they come as exact diffs: `Cargo.lock`.

**Step 1 — write the files**

File `Cargo.lock` (exact diff; save the block as `$env:TEMP\p3-t3-cargo-lock.patch`, outside the repository, then apply it):

```diff
diff --git a/Cargo.lock b/Cargo.lock
index 9f67bc6..cc9a88f 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -352,7 +352,6 @@ dependencies = [
  "planrun-model",
  "serde",
  "serde_json",
- "sha1_smol",
  "similar 3.2.1",
  "thiserror",
 ]
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p3-t3-cargo-lock.patch"
Remove-Item "$env:TEMP\p3-t3-cargo-lock.patch"
```

File `crates/planrun-exec/Cargo.toml` (full contents):

```toml
[package]
name = "planrun-exec"
description = "Executes a typed plan against a git repository: workspace, git, shells, expectations, policy, journal, runner"
version.workspace = true
edition.workspace = true
license.workspace = true
publish.workspace = true

# Deliberately no dependency on planrun-parse: execution sees only the IR (D2).
[dependencies]
planrun-model.workspace = true
serde.workspace = true
serde_json.workspace = true
similar.workspace = true
thiserror.workspace = true

[lints]
workspace = true
```

File `crates/planrun-exec/src/git.rs` (full contents):

```rust
//! A thin wrapper over the `git` binary (docs/architecture.md §6.2). Every call is one process,
//! run at the repository root with `GIT_LITERAL_PATHSPECS=1` (a path is never a pathspec
//! pattern) and the wrapper's own environment. The virtual tree (D52) passes its index and
//! object directory per call; no other call sees them.

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use planrun_model::{AbbrevBlobId, RepoPath};

/// A full SHA-1 object id (planrun-exec refuses SHA-256 repositories: plans carry SHA-1
/// abbreviations).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlobId([u8; 20]);

impl BlobId {
    /// Parses 40 hex digits (surrounding whitespace ignored).
    pub fn parse(hex: &str) -> Option<Self> {
        let hex = hex.trim();
        if hex.len() != 40 || !hex.is_ascii() {
            return None;
        }
        let mut bytes = [0u8; 20];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()?;
        }
        Some(Self(bytes))
    }

    /// True when the abbreviation from a plan's `index` line names this object.
    pub fn matches(&self, abbrev: &AbbrevBlobId) -> bool {
        abbrev.matches(&self.0)
    }
}

impl fmt::Display for BlobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|b| write!(f, "{b:02x}"))
    }
}

/// Why a git call failed.
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    /// git could not be started (not installed, not on `PATH`).
    #[error("could not run git: {0}")]
    Spawn(#[source] std::io::Error),
    /// git ran and exited non-zero.
    #[error("`git {args}` failed (exit code {code:?}): {stderr}")]
    Failed {
        /// The arguments, joined.
        args: String,
        /// The exit code; `None` when git was ended by a signal.
        code: Option<i32>,
        /// What git wrote to stderr, trimmed.
        stderr: String,
    },
    /// git printed something this wrapper cannot read.
    #[error("unexpected output from `git {args}`: {output}")]
    Output {
        /// The arguments, joined.
        args: String,
        /// The output.
        output: String,
    },
}

/// Extra environment for one call: `(name, value)` pairs.
pub type Env<'a> = &'a [(OsString, OsString)];

/// One line of `git status --porcelain -z`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    /// The two status letters (`??`, ` M`, `A `, …).
    pub code: String,
    /// The path, with forward slashes.
    pub path: String,
}

/// The `eol` and `text` attributes of one path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attributes {
    /// `lf`, `crlf`, or `None` when unspecified.
    pub eol: Option<String>,
    /// `false` when `text` is unset (`-text`, `binary`); `true` otherwise.
    pub text: bool,
}

/// The git binary, bound to one repository.
#[derive(Debug, Clone)]
pub struct Git {
    root: PathBuf,
    env: Vec<(OsString, OsString)>,
}

impl Git {
    /// git at `root` (the work-tree root), with `env` added to every call.
    pub fn new(root: &Path, env: Vec<(OsString, OsString)>) -> Self {
        Self {
            root: root.to_path_buf(),
            env,
        }
    }

    /// The work-tree root that contains `dir` (`rev-parse --show-toplevel`).
    pub fn toplevel(dir: &Path, env: Env<'_>) -> Result<PathBuf, GitError> {
        let probe = Self::new(dir, env.to_vec());
        let out = probe.run(&["rev-parse", "--show-toplevel"], None, &[])?;
        Ok(PathBuf::from(text(&out).trim()))
    }

    /// The repository root this wrapper runs in.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Runs `git <args>` at the root; `stdin` is written to its standard input. Returns stdout.
    pub fn run(
        &self,
        args: &[&str],
        stdin: Option<&[u8]>,
        extra: Env<'_>,
    ) -> Result<Vec<u8>, GitError> {
        let mut command = Command::new("git");
        command
            .args(args)
            .current_dir(&self.root)
            .env("GIT_LITERAL_PATHSPECS", "1")
            .envs(self.env.iter().map(|(k, v)| (k, v)))
            .envs(extra.iter().map(|(k, v)| (k, v)))
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().map_err(GitError::Spawn)?;
        if let (Some(bytes), Some(mut pipe)) = (stdin, child.stdin.take()) {
            pipe.write_all(bytes).map_err(GitError::Spawn)?;
        }
        let out = child.wait_with_output().map_err(GitError::Spawn)?;
        if !out.status.success() {
            return Err(GitError::Failed {
                args: args.join(" "),
                code: out.status.code(),
                stderr: text(&out.stderr).trim().to_owned(),
            });
        }
        Ok(out.stdout)
    }

    /// `rev-parse <args>` as one trimmed line.
    pub fn rev_parse(&self, args: &[&str]) -> Result<String, GitError> {
        let mut all = vec!["rev-parse"];
        all.extend_from_slice(args);
        Ok(text(&self.run(&all, None, &[])?).trim().to_owned())
    }

    /// The absolute path of the repository's object directory.
    pub fn object_dir(&self) -> Result<PathBuf, GitError> {
        let objects = PathBuf::from(self.rev_parse(&["--git-path", "objects"])?);
        Ok(if objects.is_absolute() {
            objects
        } else {
            self.root.join(objects)
        })
    }

    /// A boolean config value; `None` when unset.
    pub fn config_bool(&self, key: &str) -> Result<Option<bool>, GitError> {
        match self.run(&["config", "--bool", "--get", key], None, &[]) {
            Ok(out) => Ok(Some(text(&out).trim() == "true")),
            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// `git --no-optional-locks status --porcelain -z --untracked-files=all [-- paths]`: never
    /// refreshes the index, so it writes nothing (D52).
    pub fn status(&self, paths: Option<&[&RepoPath]>) -> Result<Vec<StatusEntry>, GitError> {
        let mut args = vec![
            "--no-optional-locks",
            "status",
            "--porcelain",
            "-z",
            "--untracked-files=all",
        ];
        if let Some(paths) = paths {
            if paths.is_empty() {
                return Ok(Vec::new());
            }
            args.push("--");
            args.extend(paths.iter().map(|p| p.as_str()));
        }
        let out = self.run(&args, None, &[])?;
        Ok(parse_status(&text(&out)))
    }

    /// The blob id git gives `bytes` at `path`. `filters` applies the clean filters and
    /// `autocrlf` of that path (`--path`); without it the bytes are hashed as they are
    /// (`--no-filters`). `write` stores the object (`-w`) in the object database `extra` selects.
    pub fn hash_object(
        &self,
        path: &RepoPath,
        bytes: &[u8],
        filters: bool,
        write: bool,
        extra: Env<'_>,
    ) -> Result<BlobId, GitError> {
        let path_arg = format!("--path={path}");
        let mut args = vec!["hash-object"];
        if write {
            args.push("-w");
        }
        args.push(if filters { &path_arg } else { "--no-filters" });
        args.push("--stdin");
        let out = self.run(&args, Some(bytes), extra)?;
        blob_id(&args, &out)
    }

    /// The bytes of blob `id`, unconverted.
    pub fn cat_blob(&self, id: &BlobId, extra: Env<'_>) -> Result<Vec<u8>, GitError> {
        self.run(&["cat-file", "blob", &id.to_string()], None, extra)
    }

    /// The bytes of blob `id` as a checkout of `path` would write them (`cat-file --filters`:
    /// smudge filters and line-ending conversion, as `git apply` does).
    pub fn cat_filtered(
        &self,
        id: &BlobId,
        path: &RepoPath,
        extra: Env<'_>,
    ) -> Result<Vec<u8>, GitError> {
        let path_arg = format!("--path={path}");
        self.run(
            &["cat-file", "--filters", &path_arg, &id.to_string()],
            None,
            extra,
        )
    }

    /// `git apply --whitespace=nowarn [--cached] [--check] -` with the patch on stdin (D20).
    pub fn apply(
        &self,
        patch: &[u8],
        cached: bool,
        check: bool,
        extra: Env<'_>,
    ) -> Result<(), GitError> {
        let mut args = vec!["apply", "--whitespace=nowarn"];
        if cached {
            args.push("--cached");
        }
        if check {
            args.push("--check");
        }
        args.push("-");
        self.run(&args, Some(patch), extra).map(|_| ())
    }

    /// The `eol` and `text` attributes of each path, in order.
    pub fn attributes(&self, paths: &[&RepoPath]) -> Result<Vec<Attributes>, GitError> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }
        let mut args = vec!["check-attr", "-z", "eol", "text", "--"];
        args.extend(paths.iter().map(|p| p.as_str()));
        let out = text(&self.run(&args, None, &[])?).into_owned();
        let fields: Vec<&str> = out.split('\0').collect();
        let mut found = vec![Attributes::default(); paths.len()];
        for triple in fields.chunks_exact(3) {
            let [path, attr, value] = triple else {
                continue;
            };
            let Some(i) = paths.iter().position(|p| p.as_str() == *path) else {
                continue;
            };
            apply_attribute(&mut found[i], attr, value);
        }
        Ok(found)
    }

    /// The mode and blob id of `path` in the index `extra` selects; `None` when not in it.
    pub fn index_entry(
        &self,
        path: &RepoPath,
        extra: Env<'_>,
    ) -> Result<Option<(String, BlobId)>, GitError> {
        let args = ["ls-files", "-s", "-z", "--", path.as_str()];
        let out = text(&self.run(&args, None, extra)?).into_owned();
        let Some(line) = out.split('\0').find(|l| !l.is_empty()) else {
            return Ok(None);
        };
        let mut parts = line.split_whitespace();
        let (Some(mode), Some(id)) = (parts.next(), parts.next()) else {
            return Err(output_error(&args, line));
        };
        let id = BlobId::parse(id).ok_or_else(|| output_error(&args, line))?;
        Ok(Some((mode.to_owned(), id)))
    }

    /// Puts `id` at `path` in the index `extra` selects.
    pub fn index_add(
        &self,
        path: &RepoPath,
        mode: &str,
        id: &BlobId,
        extra: Env<'_>,
    ) -> Result<(), GitError> {
        let info = format!("{mode},{id},{path}");
        self.run(
            &["update-index", "--add", "--cacheinfo", &info],
            None,
            extra,
        )
        .map(|_| ())
    }

    /// Removes `paths` from the index `extra` selects, in one call (one index write).
    pub fn index_remove(&self, paths: &[&RepoPath], extra: Env<'_>) -> Result<(), GitError> {
        if paths.is_empty() {
            return Ok(());
        }
        let mut args = vec!["update-index", "--force-remove", "--"];
        args.extend(paths.iter().map(|p| p.as_str()));
        self.run(&args, None, extra).map(|_| ())
    }
}

fn apply_attribute(found: &mut Attributes, attr: &str, value: &str) {
    match (attr, value) {
        ("eol", "lf" | "crlf") => found.eol = Some(value.to_owned()),
        ("text", "unset") => found.text = false,
        ("text", _) => found.text = true,
        _ => {}
    }
}

fn text(bytes: &[u8]) -> std::borrow::Cow<'_, str> {
    String::from_utf8_lossy(bytes)
}

fn output_error(args: &[&str], output: &str) -> GitError {
    GitError::Output {
        args: args.join(" "),
        output: output.to_owned(),
    }
}

fn blob_id(args: &[&str], out: &[u8]) -> Result<BlobId, GitError> {
    let line = text(out);
    BlobId::parse(&line).ok_or_else(|| output_error(args, &line))
}

/// Parses `status --porcelain -z`: `XY path\0`, and for renames and copies `XY new\0old\0`.
fn parse_status(out: &str) -> Vec<StatusEntry> {
    let mut entries = Vec::new();
    let mut fields = out.split('\0').filter(|f| !f.is_empty());
    while let Some(field) = fields.next() {
        let (code, path) = field.split_at(field.len().min(3));
        let code = code.get(..2).unwrap_or(code).to_owned();
        if code.starts_with(['R', 'C']) {
            fields.next();
        }
        entries.push(StatusEntry {
            code,
            path: path.to_owned(),
        });
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blob_id_round_trips_and_matches_its_abbreviation() {
        let hex = "97bd92a0123456789abcdef0123456789abcdef0";
        let id = BlobId::parse(hex).unwrap_or_else(|| panic!("{hex}"));
        assert_eq!(id.to_string(), hex);
        let abbrev = AbbrevBlobId::parse("97bd92a").unwrap_or_else(|e| panic!("{e}"));
        assert!(id.matches(&abbrev));
        assert!(BlobId::parse("97bd92a").is_none());
        assert!(BlobId::parse(&"g".repeat(40)).is_none());
    }

    #[test]
    fn status_reads_renames_as_one_entry() {
        let out = " M src/a.rs\0R  new.rs\0old.rs\0?? x y.txt\0";
        let entries = parse_status(out);
        let paths: Vec<_> = entries
            .iter()
            .map(|e| (e.code.as_str(), e.path.as_str()))
            .collect();
        assert_eq!(
            paths,
            [(" M", "src/a.rs"), ("R ", "new.rs"), ("??", "x y.txt")]
        );
    }
}
```

File `crates/planrun-exec/src/lib.rs` (full contents):

```rust
//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Planned modules (docs/roadmap.md):
//!
//! - `workspace`: repository root, [`RepoPath`](planrun_model::RepoPath) resolution with the
//!   symlink check, per-task file cache, step transactions (validate every file action of a step
//!   in memory, then write), atomic writes, line-ending policy (D10), pre-images stored with
//!   `git hash-object -w`.
//! - [`git`]: a thin wrapper over the `git` binary: `apply` with the patch on stdin,
//!   `hash-object`, `status --porcelain` without optional locks, `rev-parse`, `cat-file`,
//!   `check-attr`, `update-index`, `ls-files`; [`BlobId`](git::BlobId).
//! - `handler`: one [`ActionHandler`] per action kind; the [`HandlerSet`] is the execution-side
//!   plugin point.
//! - `shell`: persistent pwsh and bash sessions behind [`ShellSession`].
//! - `expect`: exit-code and summary-line matching.
//! - `policy`: command classes and the [`CommandClassifier`].
//! - `journal`: JSONL under `.git/planrun/`.
//! - `runner`: the state machine Task → Step → Action, talking to a [`Ui`].

use planrun_model::Action;

pub mod git;

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;

/// The command classes of decision D13, least to most dangerous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PolicyClass {
    /// Reads only (`git status`, `rg`).
    Read,
    /// Builds or tests (`cargo test`, `dotnet build`, `npm ci`).
    Build,
    /// Changes the repository (`git add`, `git commit`, `replace … -All`, file actions).
    Modify,
    /// Destroys or reaches outside (`Remove-Item -Recurse`, `git reset --hard`, `git push`,
    /// anything unclassified).
    Destructive,
}

/// Classifies a script: the most dangerous class of any command in it.
pub trait CommandClassifier: Send + Sync {
    /// The class of `script` run by `shell`.
    fn classify(&self, shell: planrun_model::Shell, script: &str) -> PolicyClass;
}

/// The result of running one shell block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRun {
    /// `$LASTEXITCODE` / `$?` as the session reported it; `None` when the block threw.
    pub exit: Option<i32>,
    /// Merged stdout and stderr, as UTF-8 (lossy).
    pub output: String,
}

/// A long-lived shell: variables, the working directory and functions persist across blocks.
pub trait ShellSession: Send {
    /// Runs `script`, streaming each output line to `on_line` as it arrives.
    fn run(&mut self, script: &str, on_line: &mut dyn FnMut(&str)) -> std::io::Result<ScriptRun>;
}

/// The state of an action's effects on disk, for resuming (decision D14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoneState {
    /// Every effect is present (postimages match).
    Done,
    /// No effect is present (preimages match).
    NotDone,
    /// Some are, some are not, or neither matches: the user decides.
    Unknown,
}

/// Executes one kind of action. Registered per kind in the [`HandlerSet`].
pub trait ActionHandler: Send + Sync {
    /// True for the action kinds this handler executes.
    fn handles(&self, action: &Action) -> bool;
}

/// The handlers, tried in order; the first that handles an action executes it.
#[derive(Default)]
pub struct HandlerSet {
    handlers: Vec<Box<dyn ActionHandler>>,
}

impl HandlerSet {
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

/// The interactive surface the runner talks to (decision D15: line prompts first).
pub trait Ui {
    /// Shows a line of progress.
    fn status(&mut self, line: &str);
}
```

File `crates/planrun-exec/tests/git.rs` (full contents):

```rust
//! The git wrapper on a temporary copy of the greeter fixture, with `core.autocrlf` off and on.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;

use fixture::{Fixture, canonical};
use planrun_exec::git::{Attributes, BlobId, Git};
use planrun_model::{AbbrevBlobId, RepoPath};

fn path(p: &str) -> RepoPath {
    RepoPath::new(p).unwrap_or_else(|e| panic!("{e}"))
}

fn git(f: &Fixture) -> Git {
    Git::new(f.path(), f.env())
}

fn abbrev(hex: &str) -> AbbrevBlobId {
    AbbrevBlobId::parse(hex).unwrap_or_else(|e| panic!("{e}"))
}

/// The P1-T1 patch of `tests/corpus/synthetic/assay-style.md`.
const PATCH: &str = "diff --git a/src/lib.rs b/src/lib.rs
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

#[test]
fn the_copy_is_a_repository_of_its_own() {
    let f = Fixture::greeter(false);
    let top = Git::toplevel(f.path(), &f.env()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(canonical(&top), canonical(f.path()));
    assert_eq!(
        git(&f).config_bool("core.autocrlf").ok().flatten(),
        Some(false)
    );
}

#[test]
fn blob_ids_are_git_s_view_with_and_without_autocrlf() {
    for autocrlf in [false, true] {
        let f = Fixture::greeter(autocrlf);
        let bytes = f.read("src/lib.rs").unwrap_or_default();
        assert_eq!(bytes.windows(2).any(|w| w == b"\r\n"), autocrlf);
        let g = git(&f);
        let filtered = g
            .hash_object(&path("src/lib.rs"), &bytes, true, false, &[])
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(filtered.matches(&abbrev("aa80cf1")), "{filtered}");
        let raw = g
            .hash_object(&path("src/lib.rs"), &bytes, false, false, &[])
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(raw == filtered, !autocrlf);
    }
}

#[test]
fn a_patch_checks_against_the_working_tree_with_and_without_autocrlf() {
    for autocrlf in [false, true] {
        let f = Fixture::greeter(autocrlf);
        let before = f.snapshot();
        git(&f)
            .apply(PATCH.as_bytes(), false, true, &[])
            .unwrap_or_else(|e| panic!("autocrlf={autocrlf}: {e}"));
        assert_eq!(f.snapshot(), before, "--check wrote something");
    }
}

#[test]
fn status_reports_dirt_and_writes_nothing() {
    let f = Fixture::greeter(true);
    let g = git(&f);
    let before = f.snapshot();
    assert!(g.status(None).unwrap_or_else(|e| panic!("{e}")).is_empty());
    assert_eq!(f.snapshot(), before);
    f.write("src/lib.rs", b"changed\n");
    f.write("docs/new file.txt", b"x\n");
    let all = g.status(None).unwrap_or_else(|e| panic!("{e}"));
    let codes: Vec<_> = all
        .iter()
        .map(|e| (e.code.as_str(), e.path.as_str()))
        .collect();
    assert_eq!(codes, [(" M", "src/lib.rs"), ("??", "docs/new file.txt")]);
    let one = g
        .status(Some(&[&path("src/old.rs")]))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(one.is_empty());
}

#[test]
fn objects_round_trip_through_cat_file() {
    let f = Fixture::greeter(true);
    let g = git(&f);
    let crlf = b"a\r\nb\r\n";
    let raw = g
        .hash_object(&path("x.txt"), crlf, false, true, &[])
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(g.cat_blob(&raw, &[]).ok().as_deref(), Some(&crlf[..]));
    let lf = g
        .hash_object(&path("x.txt"), b"a\nb\n", false, true, &[])
        .unwrap_or_else(|e| panic!("{e}"));
    let checkout = g
        .cat_filtered(&lf, &path("x.txt"), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(checkout, crlf, "autocrlf=true checks text out with CRLF");
}

#[test]
fn attributes_report_eol_and_text() {
    let f = Fixture::greeter(false);
    f.write(
        ".gitattributes",
        b"*.crlf eol=crlf\n*.lf eol=lf\n*.bin binary\n",
    );
    let paths = [path("a.crlf"), path("a.lf"), path("a.bin"), path("a.rs")];
    let refs: Vec<&RepoPath> = paths.iter().collect();
    let found = git(&f).attributes(&refs).unwrap_or_else(|e| panic!("{e}"));
    let attr = |eol: Option<&str>, text: bool| Attributes {
        eol: eol.map(str::to_owned),
        text,
    };
    assert_eq!(
        found,
        [
            attr(Some("crlf"), true),
            attr(Some("lf"), true),
            attr(None, false),
            attr(None, true)
        ]
    );
}

#[test]
fn the_index_entry_of_a_tracked_file() {
    let f = Fixture::greeter(false);
    let entry = git(&f)
        .index_entry(&path("src/lib.rs"), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    let (mode, id) = entry.unwrap_or_else(|| panic!("src/lib.rs is tracked"));
    assert_eq!(mode, "100644");
    assert!(id.matches(&abbrev("aa80cf1")));
    let none = git(&f)
        .index_entry(&path("nothing.rs"), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(none, None::<(String, BlobId)>);
}
```

File `tests/support/fixture.rs` (full contents):

```rust
//! A temporary copy of `tests/fixtures/greeter` as a git repository of its own, for the tests of
//! planrun-exec and planrun-cli (included with `#[path]`; D2 keeps the workspace at four
//! crates). Never inside this repository: the copy lives under the OS temp directory, and every
//! git call gets an environment that keeps the user's configuration out (no system config, an
//! empty global config, a ceiling above the copy).

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A fixture copy: `<temp>/planrun-test-<pid>-<nanos>-<n>/repo`, with the empty global config
/// next to it. Removed on drop.
pub struct Fixture {
    base: PathBuf,
    repo: PathBuf,
}

impl Fixture {
    /// The greeter fixture, committed once, with `core.autocrlf` set as given. With `true`, the
    /// working tree is checked out again so that its text files have CRLF line endings.
    pub fn greeter(autocrlf: bool) -> Fixture {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let base =
            std::env::temp_dir().join(format!("planrun-test-{}-{nanos}-{n}", std::process::id()));
        let repo = base.join("repo");
        std::fs::create_dir_all(&repo).unwrap_or_else(|e| panic!("{}: {e}", repo.display()));
        std::fs::write(base.join("gitconfig"), "").unwrap_or_else(|e| panic!("{e}"));
        let fixture = Fixture { base, repo };
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/greeter");
        copy_dir(&source, &fixture.repo);
        fixture.git(&["init", "-q", "-b", "main"]);
        let top = fixture.git(&["rev-parse", "--show-toplevel"]);
        assert_eq!(
            canonical(Path::new(top.trim())),
            canonical(&fixture.repo),
            "git init did not make the copy a repository of its own"
        );
        fixture.git(&[
            "config",
            "core.autocrlf",
            if autocrlf { "true" } else { "false" },
        ]);
        fixture.git(&["add", "-A"]);
        fixture.git(&["commit", "-q", "-m", "fixture"]);
        if autocrlf {
            fixture.checkout_again();
        }
        fixture
    }

    /// The repository root.
    pub fn path(&self) -> &Path {
        &self.repo
    }

    /// The environment every git call of a test gets.
    pub fn env(&self) -> Vec<(OsString, OsString)> {
        let var = |k: &str, v: &str| (OsString::from(k), OsString::from(v));
        vec![
            var("GIT_CONFIG_NOSYSTEM", "1"),
            (
                OsString::from("GIT_CONFIG_GLOBAL"),
                self.base.join("gitconfig").into_os_string(),
            ),
            (
                OsString::from("GIT_CEILING_DIRECTORIES"),
                self.base.clone().into_os_string(),
            ),
            var("GIT_AUTHOR_NAME", "planrun test"),
            var("GIT_AUTHOR_EMAIL", "test@planrun.invalid"),
            var("GIT_COMMITTER_NAME", "planrun test"),
            var("GIT_COMMITTER_EMAIL", "test@planrun.invalid"),
        ]
    }

    /// Runs git in the copy with [`Fixture::env`]; panics when it fails. Returns stdout.
    pub fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.repo)
            .envs(self.env())
            .output()
            .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// Writes `bytes` to `rel` (forward slashes) in the copy, creating directories.
    pub fn write(&self, rel: &str, bytes: &[u8]) {
        let path = self.repo.join(rel);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("{e}"));
        }
        std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }

    /// The bytes of `rel`, or `None` when it does not exist.
    pub fn read(&self, rel: &str) -> Option<Vec<u8>> {
        std::fs::read(self.repo.join(rel)).ok()
    }

    /// Every file under the copy, `.git` included: relative path → contents.
    pub fn snapshot(&self) -> BTreeMap<String, Vec<u8>> {
        let mut files = BTreeMap::new();
        collect(&self.repo, &self.repo, &mut files);
        files
    }

    /// `git status --porcelain` without refreshing the index.
    pub fn status(&self) -> String {
        self.git(&["--no-optional-locks", "status", "--porcelain"])
    }

    /// Deletes the tracked files and checks them out again, so that `core.autocrlf` applies.
    fn checkout_again(&self) {
        let listed = self.git(&["ls-files", "-z"]);
        for file in listed.split('\0').filter(|f| !f.is_empty()) {
            let path = self.repo.join(file);
            std::fs::remove_file(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        }
        self.git(&["checkout", "--", "."]);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        clear_read_only(&self.base);
        if let Err(e) = std::fs::remove_dir_all(&self.base) {
            eprintln!(
                "could not remove the fixture copy {}: {e}",
                self.base.display()
            );
        }
    }
}

/// The path canonicalized, or as given when that fails.
pub fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Copies the fixture as git stores it: its files are text with LF line endings in this
/// repository, so CRLF from a checkout with `core.autocrlf=true` is turned back into LF, and the
/// copy's blob ids are the ones the plans name (`aa80cf1`). A `target/` from building the
/// fixture is skipped.
fn copy_dir(from: &Path, to: &Path) {
    let entries = std::fs::read_dir(from).unwrap_or_else(|e| panic!("{}: {e}", from.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("{e}"));
        let target = to.join(entry.file_name());
        let kind = entry.file_type().unwrap_or_else(|e| panic!("{e}"));
        if kind.is_dir() && entry.file_name() == "target" {
            continue;
        }
        if kind.is_dir() {
            std::fs::create_dir_all(&target).unwrap_or_else(|e| panic!("{e}"));
            copy_dir(&entry.path(), &target);
            continue;
        }
        let bytes = std::fs::read(entry.path()).unwrap_or_else(|e| panic!("{e}"));
        let text = String::from_utf8(bytes).unwrap_or_else(|e| panic!("the fixture is text: {e}"));
        std::fs::write(&target, text.replace("\r\n", "\n")).unwrap_or_else(|e| panic!("{e}"));
    }
}

fn collect(root: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, files);
            continue;
        }
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let key = rel.to_string_lossy().replace('\\', "/");
        files.insert(key, std::fs::read(&path).unwrap_or_default());
    }
}

/// git writes its object files read-only; Windows refuses to delete those.
fn clear_read_only(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            clear_read_only(&path);
            continue;
        }
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        let mut permissions = meta.permissions();
        if permissions.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = std::fs::set_permissions(&path, permissions);
        }
    }
}
```

**Step 2 — verify**

Targeted: the git wrapper on a fixture copy:

```powershell
cargo test -p planrun-exec --test git
```

Expected (exit code 0; summary lines):

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- Cargo.lock crates/planrun-exec/Cargo.toml crates/planrun-exec/src/git.rs crates/planrun-exec/src/lib.rs crates/planrun-exec/tests/git.rs tests/support/fixture.rs
  $verified = [ordered]@{
      'Cargo.lock' = 'cc9a88f2854a42e94b5ef3d544813754e7326295'
      'crates/planrun-exec/Cargo.toml' = '8a98699ab6b31158720ebcd0ad2c9deebac36fcc'
      'crates/planrun-exec/src/git.rs' = '222bc1cdece21aa68466a5c7fc487a53f734de21'
      'crates/planrun-exec/src/lib.rs' = '5342dfeaf8e1d43497859d86fe222763ef9ba790'
      'crates/planrun-exec/tests/git.rs' = '805430768cf0fe671f8cd430bb4faf5e4cfbeb04'
      'tests/support/fixture.rs' = '01610053203774e7095fb58c4b25f516e73d7481'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P3-T3: the git wrapper and BlobId, the shared test fixture, sha1_smol out of planrun-exec (D56)" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P3-T4: the workspace (resolution, the writer, D52) and line endings (D54)

**Goal:** The workspace and line endings. `Workspace::resolve` refuses links, junctions, short names and case mismatches by comparing the canonical existing part with the root plus the same segments; `write` creates directories one segment at a time, writes a `create_new` temp file, checks the parent again and renames (D52); a `#[cfg(test)]` seam fails a chosen write or runs a hook before the rename. `eol` is pure: the conventions of D54 and the conversions.

**Files**

- Create `crates/planrun-exec/src/eol.rs`
- Modify `crates/planrun-exec/src/lib.rs`
- Create `crates/planrun-exec/src/workspace.rs`
- Create `crates/planrun-exec/src/workspace_tests.rs`
- Modify `tests/support/fixture.rs`

**Step 1 — write the files**

File `crates/planrun-exec/src/eol.rs` (full contents):

```rust
//! Line endings of written files (decisions D10, D54). Pure: the bytes, the path's attributes and
//! git's checkout convention for the path go in, the bytes to write come out.

use crate::git::Attributes;

/// A line ending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eol {
    /// `\n`.
    Lf,
    /// `\r\n`.
    Crlf,
}

/// The line endings a file has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endings {
    /// No line ending at all (empty, or one line).
    None,
    /// Only LF.
    Lf,
    /// Only CRLF.
    Crlf,
    /// Both.
    Mixed,
}

/// How a target's new contents are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Convention {
    /// `-text` or `binary`: the bytes as they are, matched as they are.
    Bytes,
    /// Text, with this line ending.
    Text(Eol),
}

/// A ReplaceExact on a file with both LF and CRLF (`eol-mixed`, D54).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mixed;

/// The line endings of `bytes`.
pub fn endings(bytes: &[u8]) -> Endings {
    let mut lf = false;
    let mut crlf = false;
    for (i, &b) in bytes.iter().enumerate() {
        if b != b'\n' {
            continue;
        }
        if i > 0 && bytes[i - 1] == b'\r' {
            crlf = true;
        } else {
            lf = true;
        }
    }
    match (lf, crlf) {
        (false, false) => Endings::None,
        (true, false) => Endings::Lf,
        (false, true) => Endings::Crlf,
        (true, true) => Endings::Mixed,
    }
}

/// `bytes` with every CRLF turned into LF (a lone CR stays).
pub fn to_lf(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for (i, &b) in bytes.iter().enumerate() {
        let crlf = b == b'\r' && bytes.get(i + 1) == Some(&b'\n');
        if !crlf {
            out.push(b);
        }
    }
    out
}

/// LF text `lf` with the line ending `eol`.
pub fn with_eol(lf: &[u8], eol: Eol) -> Vec<u8> {
    if eol == Eol::Lf {
        return lf.to_vec();
    }
    let mut out = Vec::with_capacity(lf.len() + lf.len() / 32);
    for &b in lf {
        if b == b'\n' {
            out.push(b'\r');
        }
        out.push(b);
    }
    out
}

/// The attribute's decision, when it makes one: `-text` → bytes, `eol` → that ending.
fn by_attributes(attrs: &Attributes) -> Option<Convention> {
    if !attrs.text {
        return Some(Convention::Bytes);
    }
    match attrs.eol.as_deref() {
        Some("crlf") => Some(Convention::Text(Eol::Crlf)),
        Some("lf") => Some(Convention::Text(Eol::Lf)),
        _ => None,
    }
}

/// The convention of an Overwrite of a file whose current contents are `existing`.
pub fn for_overwrite(attrs: &Attributes, existing: &[u8], checkout: Eol) -> Convention {
    if let Some(decided) = by_attributes(attrs) {
        return decided;
    }
    match endings(existing) {
        Endings::Lf => Convention::Text(Eol::Lf),
        Endings::Crlf => Convention::Text(Eol::Crlf),
        Endings::None | Endings::Mixed => Convention::Text(checkout),
    }
}

/// The convention of a ReplaceExact in `existing`; a mixed file is refused.
pub fn for_replace(
    attrs: &Attributes,
    existing: &[u8],
    checkout: Eol,
) -> Result<Convention, Mixed> {
    if let Some(decided) = by_attributes(attrs) {
        return Ok(decided);
    }
    match endings(existing) {
        Endings::Mixed => Err(Mixed),
        Endings::Lf => Ok(Convention::Text(Eol::Lf)),
        Endings::Crlf => Ok(Convention::Text(Eol::Crlf)),
        Endings::None => Ok(Convention::Text(checkout)),
    }
}

/// The plan's LF text `body` as written under `convention`.
pub fn render(body: &[u8], convention: Convention) -> Vec<u8> {
    match convention {
        Convention::Bytes => body.to_vec(),
        Convention::Text(eol) => with_eol(body, eol),
    }
}

/// The text a ReplaceExact searches in: LF space for text (D10), the bytes for `-text`.
pub fn search_space(existing: &[u8], convention: Convention) -> Vec<u8> {
    match convention {
        Convention::Bytes => existing.to_vec(),
        Convention::Text(_) => to_lf(existing),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(eol: Option<&str>, text: bool) -> Attributes {
        Attributes {
            eol: eol.map(str::to_owned),
            text,
        }
    }

    #[test]
    fn endings_are_told_apart() {
        assert_eq!(endings(b""), Endings::None);
        assert_eq!(endings(b"one line"), Endings::None);
        assert_eq!(endings(b"a\nb\n"), Endings::Lf);
        assert_eq!(endings(b"a\r\nb\r\n"), Endings::Crlf);
        assert_eq!(endings(b"a\r\nb\n"), Endings::Mixed);
        assert_eq!(endings(b"\n"), Endings::Lf);
    }

    #[test]
    fn conversions_round_trip() {
        assert_eq!(to_lf(b"a\r\nb\rc\r\n"), b"a\nb\rc\n");
        assert_eq!(with_eol(b"a\nb\n", Eol::Crlf), b"a\r\nb\r\n");
        assert_eq!(with_eol(b"a\nb\n", Eol::Lf), b"a\nb\n");
    }

    #[test]
    fn attributes_win_and_binary_keeps_bytes() {
        let crlf_file = b"a\r\nb\r\n";
        assert_eq!(
            for_overwrite(&attrs(Some("lf"), true), crlf_file, Eol::Crlf),
            Convention::Text(Eol::Lf)
        );
        assert_eq!(
            for_overwrite(&attrs(Some("crlf"), true), b"a\n", Eol::Lf),
            Convention::Text(Eol::Crlf)
        );
        assert_eq!(
            for_replace(&attrs(Some("crlf"), false), b"a\r\nb\n", Eol::Lf),
            Ok(Convention::Bytes)
        );
    }

    #[test]
    fn the_file_decides_without_attributes() {
        let plain = attrs(None, true);
        assert_eq!(
            for_overwrite(&plain, b"a\r\n", Eol::Lf),
            Convention::Text(Eol::Crlf)
        );
        assert_eq!(
            for_overwrite(&plain, b"a\n", Eol::Crlf),
            Convention::Text(Eol::Lf)
        );
        assert_eq!(
            for_replace(&plain, b"a\r\n", Eol::Lf),
            Ok(Convention::Text(Eol::Crlf))
        );
    }

    #[test]
    fn no_line_endings_take_the_checkout_convention() {
        let plain = attrs(None, true);
        for checkout in [Eol::Lf, Eol::Crlf] {
            assert_eq!(
                for_overwrite(&plain, b"", checkout),
                Convention::Text(checkout)
            );
            assert_eq!(
                for_replace(&plain, b"one", checkout),
                Ok(Convention::Text(checkout))
            );
        }
    }

    #[test]
    fn a_mixed_file_is_overwritten_by_checkout_and_never_edited() {
        let plain = attrs(None, true);
        let mixed = b"a\r\nb\n";
        assert_eq!(
            for_overwrite(&plain, mixed, Eol::Crlf),
            Convention::Text(Eol::Crlf)
        );
        assert_eq!(for_replace(&plain, mixed, Eol::Lf), Err(Mixed));
    }

    #[test]
    fn search_space_is_lf_for_text_and_raw_for_bytes() {
        let file = b"a\r\nb\r\n";
        assert_eq!(search_space(file, Convention::Text(Eol::Crlf)), b"a\nb\n");
        assert_eq!(search_space(file, Convention::Bytes), file);
        assert_eq!(render(b"x\n", Convention::Text(Eol::Crlf)), b"x\r\n");
        assert_eq!(render(b"x\n", Convention::Bytes), b"x\n");
    }
}
```

File `crates/planrun-exec/src/lib.rs` (full contents):

```rust
//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Planned modules (docs/roadmap.md):
//!
//! - [`workspace`]: the repository root, [`RepoPath`](planrun_model::RepoPath) resolution with
//!   the link and case check, and the one writer (temp file, second check, rename; D52).
//! - [`eol`]: the line endings of written files (D10, D54).
//! - [`git`]: a thin wrapper over the `git` binary: `apply` with the patch on stdin,
//!   `hash-object`, `status --porcelain` without optional locks, `rev-parse`, `cat-file`,
//!   `check-attr`, `update-index`, `ls-files`; [`BlobId`](git::BlobId).
//! - `handler`: one [`ActionHandler`] per action kind; the [`HandlerSet`] is the execution-side
//!   plugin point.
//! - `shell`: persistent pwsh and bash sessions behind [`ShellSession`].
//! - `expect`: exit-code and summary-line matching.
//! - `policy`: command classes and the [`CommandClassifier`].
//! - `journal`: JSONL under `.git/planrun/`.
//! - `runner`: the state machine Task → Step → Action, talking to a [`Ui`].

use planrun_model::Action;

pub mod eol;
pub mod git;
pub mod workspace;

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;

/// The command classes of decision D13, least to most dangerous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PolicyClass {
    /// Reads only (`git status`, `rg`).
    Read,
    /// Builds or tests (`cargo test`, `dotnet build`, `npm ci`).
    Build,
    /// Changes the repository (`git add`, `git commit`, `replace … -All`, file actions).
    Modify,
    /// Destroys or reaches outside (`Remove-Item -Recurse`, `git reset --hard`, `git push`,
    /// anything unclassified).
    Destructive,
}

/// Classifies a script: the most dangerous class of any command in it.
pub trait CommandClassifier: Send + Sync {
    /// The class of `script` run by `shell`.
    fn classify(&self, shell: planrun_model::Shell, script: &str) -> PolicyClass;
}

/// The result of running one shell block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRun {
    /// `$LASTEXITCODE` / `$?` as the session reported it; `None` when the block threw.
    pub exit: Option<i32>,
    /// Merged stdout and stderr, as UTF-8 (lossy).
    pub output: String,
}

/// A long-lived shell: variables, the working directory and functions persist across blocks.
pub trait ShellSession: Send {
    /// Runs `script`, streaming each output line to `on_line` as it arrives.
    fn run(&mut self, script: &str, on_line: &mut dyn FnMut(&str)) -> std::io::Result<ScriptRun>;
}

/// The state of an action's effects on disk, for resuming (decision D14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoneState {
    /// Every effect is present (postimages match).
    Done,
    /// No effect is present (preimages match).
    NotDone,
    /// Some are, some are not, or neither matches: the user decides.
    Unknown,
}

/// Executes one kind of action. Registered per kind in the [`HandlerSet`].
pub trait ActionHandler: Send + Sync {
    /// True for the action kinds this handler executes.
    fn handles(&self, action: &Action) -> bool;
}

/// The handlers, tried in order; the first that handles an action executes it.
#[derive(Default)]
pub struct HandlerSet {
    handlers: Vec<Box<dyn ActionHandler>>,
}

impl HandlerSet {
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

/// The interactive surface the runner talks to (decision D15: line prompts first).
pub trait Ui {
    /// Shows a line of progress.
    fn status(&mut self, line: &str);
}
```

File `crates/planrun-exec/src/workspace.rs` (full contents):

```rust
//! The repository on disk: path resolution with the link and case check, and the one writer
//! (decision D52). Nothing here decides what to write; the transaction does (D53).

use std::collections::hash_map::RandomState;
use std::ffi::OsString;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use planrun_model::RepoPath;

use crate::git::{Git, GitError};

/// Why the workspace refused or failed.
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    /// The path leaves the repository through a link, or is not what its spelling says.
    #[error("`{path}`: {reason}")]
    Unsafe {
        /// The plan's path.
        path: String,
        /// What is wrong.
        reason: String,
    },
    /// The path's existing segments differ in case from the disk (D52).
    #[error("`{path}` is spelled `{on_disk}` on disk")]
    Case {
        /// The plan's path.
        path: String,
        /// The spelling on disk, from the root.
        on_disk: String,
    },
    /// A file operation failed.
    #[error("`{path}`: {source}")]
    Io {
        /// The plan's path.
        path: String,
        /// The error.
        #[source]
        source: std::io::Error,
    },
    /// A git call failed.
    #[error(transparent)]
    Git(#[from] GitError),
    /// The repository uses an object format other than SHA-1.
    #[error("the repository uses `{0}` object ids; planrun supports SHA-1 repositories only")]
    ObjectFormat(String),
}

impl WorkspaceError {
    /// The stable code of a finding made from this error.
    pub fn code(&self) -> &'static str {
        match self {
            WorkspaceError::Unsafe { .. } => "path-unsafe",
            WorkspaceError::Case { .. } => "path-case",
            WorkspaceError::Io { .. } => "io",
            WorkspaceError::Git(_) => "git",
            WorkspaceError::ObjectFormat(_) => "object-format",
        }
    }
}

/// A path resolved on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// The full path.
    pub full: PathBuf,
    /// How many leading segments exist on disk.
    pub existing: usize,
    /// The number of segments.
    pub segments: usize,
    /// True when the whole path exists and is a directory.
    pub is_dir: bool,
}

impl Resolved {
    /// True when the whole path exists.
    pub fn exists(&self) -> bool {
        self.existing == self.segments
    }
}

/// The repository root, its git, and the writer.
pub struct Workspace {
    root: PathBuf,
    git: Git,
    ignorecase: bool,
    #[cfg(test)]
    pub(crate) faults: Faults,
}

impl Workspace {
    /// The repository that contains `dir`, with `env` added to every git call.
    pub fn open(dir: &Path, env: Vec<(OsString, OsString)>) -> Result<Self, WorkspaceError> {
        let top = Git::toplevel(dir, &env)?;
        let root = fs::canonicalize(&top).map_err(|source| WorkspaceError::Io {
            path: top.display().to_string(),
            source,
        })?;
        let git = Git::new(&top, env);
        let format = git.rev_parse(&["--show-object-format"])?;
        if format != "sha1" {
            return Err(WorkspaceError::ObjectFormat(format));
        }
        let ignorecase = git.config_bool("core.ignorecase")?.unwrap_or(false);
        Ok(Self {
            root,
            git,
            ignorecase,
            #[cfg(test)]
            faults: Faults::default(),
        })
    }

    /// The canonical repository root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// git, run at the root.
    pub fn git(&self) -> &Git {
        &self.git
    }

    /// `core.ignorecase`: path identity is case-insensitive (D52).
    pub fn ignorecase(&self) -> bool {
        self.ignorecase
    }

    /// Resolves `path`: every existing segment must be a real directory (the last one may be a
    /// file), never a link, and the existing part must canonicalize to the root plus the same
    /// segments, compared exactly. That refuses symlinks, junctions, 8.3 short names and case
    /// mismatches (D52).
    pub fn resolve(&self, path: &RepoPath) -> Result<Resolved, WorkspaceError> {
        let segments: Vec<&str> = path.as_str().split('/').collect();
        let mut current = self.root.clone();
        let mut existing = 0;
        let mut is_dir = false;
        for (i, segment) in segments.iter().enumerate() {
            let next = current.join(segment);
            let meta = match fs::symlink_metadata(&next) {
                Ok(meta) => meta,
                Err(e) if e.kind() == ErrorKind::NotFound => break,
                Err(source) => return Err(io_error(path, source)),
            };
            if meta.file_type().is_symlink() {
                return Err(unsafe_path(
                    path,
                    "a segment is a symbolic link or junction",
                ));
            }
            if i + 1 < segments.len() && !meta.is_dir() {
                return Err(unsafe_path(
                    path,
                    "a segment before the last is not a directory",
                ));
            }
            is_dir = meta.is_dir();
            existing = i + 1;
            current = next;
        }
        self.check_spelling(path, &segments[..existing], &current)?;
        let mut full = current;
        for segment in &segments[existing..] {
            full.push(segment);
        }
        Ok(Resolved {
            full,
            existing,
            segments: segments.len(),
            is_dir: is_dir && existing == segments.len(),
        })
    }

    /// The existing part of a path, canonicalized, must be the root plus exactly these segments.
    fn check_spelling(
        &self,
        path: &RepoPath,
        segments: &[&str],
        deepest: &Path,
    ) -> Result<(), WorkspaceError> {
        if segments.is_empty() {
            return Ok(());
        }
        let actual = fs::canonicalize(deepest).map_err(|e| io_error(path, e))?;
        let expected = segments.iter().fold(self.root.clone(), |p, s| p.join(s));
        if actual == expected {
            return Ok(());
        }
        let shown = |p: &Path| p.to_string_lossy().to_lowercase();
        if shown(&actual) == shown(&expected) {
            let on_disk = actual.strip_prefix(&self.root).unwrap_or(&actual);
            return Err(WorkspaceError::Case {
                path: path.to_string(),
                on_disk: on_disk.to_string_lossy().replace('\\', "/"),
            });
        }
        Err(unsafe_path(
            path,
            "it resolves to another place on disk (a link or a short name)",
        ))
    }

    /// The bytes at `path`; `None` when it does not exist.
    pub fn read(&self, path: &RepoPath) -> Result<Option<Vec<u8>>, WorkspaceError> {
        let resolved = self.resolve(path)?;
        if !resolved.exists() {
            return Ok(None);
        }
        if resolved.is_dir {
            return Err(unsafe_path(path, "it is a directory"));
        }
        fs::read(&resolved.full)
            .map(Some)
            .map_err(|e| io_error(path, e))
    }

    /// Writes `bytes` to `path` through a temp file and a rename, creating the directories it
    /// needs one segment at a time, each checked. Returns the directories it created, deepest
    /// last. On an error nothing of this write remains: no temp file, no new directory.
    pub fn write(&self, path: &RepoPath, bytes: &[u8]) -> Result<Vec<PathBuf>, WorkspaceError> {
        let resolved = self.resolve(path)?;
        if resolved.is_dir {
            return Err(unsafe_path(path, "it is a directory"));
        }
        let created = self.make_dirs(path, &resolved)?;
        if let Err(e) = self.replace_file(path, &resolved.full, bytes) {
            remove_dirs(&created);
            return Err(e);
        }
        Ok(created)
    }

    /// Removes the file at `path`; true when there was one.
    pub fn remove(&self, path: &RepoPath) -> Result<bool, WorkspaceError> {
        let resolved = self.resolve(path)?;
        if !resolved.exists() {
            return Ok(false);
        }
        if resolved.is_dir {
            return Err(unsafe_path(path, "it is a directory"));
        }
        fs::remove_file(&resolved.full).map_err(|e| io_error(path, e))?;
        Ok(true)
    }

    /// The directories of `path` that do not exist yet, created one at a time, each checked
    /// right after it was made.
    fn make_dirs(
        &self,
        path: &RepoPath,
        resolved: &Resolved,
    ) -> Result<Vec<PathBuf>, WorkspaceError> {
        let segments: Vec<&str> = path.as_str().split('/').collect();
        let dirs = segments.len() - 1;
        let first_missing = resolved.existing.min(dirs);
        let mut current = segments[..first_missing]
            .iter()
            .fold(self.root.clone(), |p, s| p.join(s));
        let mut created = Vec::new();
        for segment in &segments[first_missing..dirs] {
            current.push(segment);
            if let Err(e) = fs::create_dir(&current) {
                remove_dirs(&created);
                return Err(io_error(path, e));
            }
            created.push(current.clone());
            let real = fs::symlink_metadata(&current)
                .is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink());
            if !real {
                remove_dirs(&created);
                return Err(unsafe_path(path, "a directory it created is not one"));
            }
        }
        Ok(created)
    }

    /// The temp file, the second check of the parent, and the rename.
    fn replace_file(
        &self,
        path: &RepoPath,
        full: &Path,
        bytes: &[u8],
    ) -> Result<(), WorkspaceError> {
        let parent = full.parent().unwrap_or(&self.root);
        let temp = parent.join(format!(".planrun-{:016x}.tmp", random()));
        if let Err(e) = write_new(&temp, bytes) {
            let _ = fs::remove_file(&temp);
            return Err(io_error(path, e));
        }
        let renamed = self
            .before_rename(path, full)
            .and_then(|()| self.recheck_parent(path))
            .and_then(|()| fs::rename(&temp, full).map_err(|e| io_error(path, e)));
        if renamed.is_err() {
            let _ = fs::remove_file(&temp);
        }
        renamed
    }

    /// The parent directory again, directly before the rename (D52). A parent swapped after
    /// this check is the race D52 states.
    fn recheck_parent(&self, path: &RepoPath) -> Result<(), WorkspaceError> {
        let Some(dir) = path.dir() else {
            return Ok(());
        };
        let parent = RepoPath::new(dir).map_err(|e| unsafe_path(path, &e.to_string()))?;
        let resolved = self.resolve(&parent)?;
        if resolved.exists() && resolved.is_dir {
            return Ok(());
        }
        Err(unsafe_path(path, "its directory changed before the rename"))
    }

    #[cfg(not(test))]
    fn before_rename(&self, _path: &RepoPath, _full: &Path) -> Result<(), WorkspaceError> {
        Ok(())
    }

    #[cfg(test)]
    fn before_rename(&self, path: &RepoPath, full: &Path) -> Result<(), WorkspaceError> {
        self.faults.before_rename(path, full)
    }
}

/// Test-only faults: fail the n-th write (1-based), or run a hook right before a rename.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct Faults {
    /// The write that fails, counted from 1.
    pub fail_write: std::cell::Cell<Option<usize>>,
    writes: std::cell::Cell<usize>,
    /// Runs before the parent check and the rename, with the target's full path.
    #[allow(clippy::type_complexity)]
    pub hook: std::cell::RefCell<Option<Box<dyn Fn(&Path)>>>,
}

#[cfg(test)]
impl Faults {
    fn before_rename(&self, path: &RepoPath, full: &Path) -> Result<(), WorkspaceError> {
        let n = self.writes.get() + 1;
        self.writes.set(n);
        if let Some(hook) = self.hook.borrow().as_ref() {
            hook(full);
        }
        if self.fail_write.get() == Some(n) {
            return Err(io_error(path, std::io::Error::other("injected fault")));
        }
        Ok(())
    }
}

fn write_new(temp: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// Removes directories this write created, deepest first, while they are empty.
pub(crate) fn remove_dirs(created: &[PathBuf]) {
    for dir in created.iter().rev() {
        let _ = fs::remove_dir(dir);
    }
}

fn random() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut hasher = RandomState::new().build_hasher();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    hasher.write_u128(nanos);
    hasher.write_u64(COUNTER.fetch_add(1, Ordering::Relaxed));
    hasher.write_u32(std::process::id());
    hasher.finish()
}

fn io_error(path: &RepoPath, source: std::io::Error) -> WorkspaceError {
    WorkspaceError::Io {
        path: path.to_string(),
        source,
    }
}

fn unsafe_path(path: &RepoPath, reason: &str) -> WorkspaceError {
    WorkspaceError::Unsafe {
        path: path.to_string(),
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
#[path = "workspace_tests.rs"]
mod tests;
```

File `crates/planrun-exec/src/workspace_tests.rs` (full contents):

```rust
//! The workspace on a fixture copy: resolution, the writer, links and the test seam.

use std::path::Path;

use planrun_model::RepoPath;

use super::*;
use crate::fixture::Fixture;

fn path(p: &str) -> RepoPath {
    RepoPath::new(p).unwrap_or_else(|e| panic!("{e}"))
}

fn open(f: &Fixture) -> Workspace {
    Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
}

fn temp_files(f: &Fixture) -> Vec<String> {
    f.snapshot()
        .into_keys()
        .filter(|k| k.contains(".planrun-"))
        .collect()
}

/// A directory link: a symlink on Unix, a junction on Windows (no privilege needed).
fn link_dir(link: &Path, target: &Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
            .is_ok_and(|o| o.status.success())
    }
}

/// A file symlink; on Windows it needs a privilege the test may not have.
fn link_file(link: &Path, target: &Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link).is_ok()
    }
}

#[test]
fn a_path_resolves_and_reads() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let lib = ws
        .read(&path("src/lib.rs"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(lib, f.read("src/lib.rs"));
    assert_eq!(ws.read(&path("src/none.rs")).ok(), Some(None));
    let missing = ws
        .resolve(&path("a/b/c.rs"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!((missing.existing, missing.segments), (0, 3));
    assert!(ws.read(&path("src")).is_err(), "a directory is no file");
}

#[test]
fn a_write_creates_its_directories_and_leaves_no_temp_file() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let created = ws
        .write(&path("a/b/c.txt"), b"x\r\n")
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(created.len(), 2);
    assert_eq!(f.read("a/b/c.txt").as_deref(), Some(&b"x\r\n"[..]));
    ws.write(&path("src/lib.rs"), b"replaced\n")
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.read("src/lib.rs").as_deref(), Some(&b"replaced\n"[..]));
    assert!(temp_files(&f).is_empty());
    assert_eq!(ws.remove(&path("a/b/c.txt")).ok(), Some(true));
    assert_eq!(ws.remove(&path("a/b/c.txt")).ok(), Some(false));
}

#[test]
fn a_case_mismatch_is_refused_where_the_disk_ignores_case() {
    let f = Fixture::greeter(false);
    if !f.path().join("SRC/LIB.RS").exists() {
        eprintln!("skipped: this file system is case-sensitive");
        return;
    }
    let ws = open(&f);
    let err = ws.resolve(&path("SRC/lib.rs")).err();
    assert_eq!(err.as_ref().map(WorkspaceError::code), Some("path-case"));
    assert!(ws.write(&path("src/LIB.rs"), b"x").is_err());
    assert_eq!(f.status(), "");
}

#[test]
fn a_linked_directory_is_refused() {
    let f = Fixture::greeter(false);
    let outside = f.outside();
    if !link_dir(&f.path().join("linked"), &outside) {
        eprintln!("skipped: could not create a directory link");
        return;
    }
    let ws = open(&f);
    let err = ws.write(&path("linked/x.txt"), b"x").err();
    assert_eq!(err.as_ref().map(WorkspaceError::code), Some("path-unsafe"));
    assert!(std::fs::read_dir(&outside).map_or(0, |d| d.count()) == 0);
}

#[test]
fn a_target_turned_into_a_link_before_the_rename_is_replaced_not_followed() {
    let f = Fixture::greeter(false);
    let outside = f.outside().join("secret.txt");
    std::fs::write(&outside, b"secret\n").unwrap_or_else(|e| panic!("{e}"));
    let ws = open(&f);
    let target = outside.clone();
    let made = std::rc::Rc::new(std::cell::Cell::new(false));
    let flag = made.clone();
    *ws.faults.hook.borrow_mut() = Some(Box::new(move |full: &Path| {
        let _ = std::fs::remove_file(full);
        flag.set(link_file(full, &target));
    }));
    let written = ws.write(&path("src/lib.rs"), b"new\n");
    if !made.get() {
        eprintln!("skipped: could not create a file symlink");
        return;
    }
    written.unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read(&outside).ok().as_deref(),
        Some(&b"secret\n"[..])
    );
    let meta = std::fs::symlink_metadata(f.path().join("src/lib.rs"));
    assert!(
        meta.is_ok_and(|m| m.is_file()),
        "the link itself was replaced"
    );
    assert_eq!(f.read("src/lib.rs").as_deref(), Some(&b"new\n"[..]));
}

#[test]
fn a_parent_swapped_for_a_link_before_the_rename_is_refused() {
    let f = Fixture::greeter(false);
    let outside = f.outside();
    let ws = open(&f);
    let made = std::rc::Rc::new(std::cell::Cell::new(false));
    let flag = made.clone();
    let target = outside.clone();
    *ws.faults.hook.borrow_mut() = Some(Box::new(move |full: &Path| {
        let Some(parent) = full.parent() else {
            return;
        };
        let moved = parent.with_extension("old");
        if std::fs::rename(parent, &moved).is_ok() {
            flag.set(link_dir(parent, &target));
        }
    }));
    let written = ws.write(&path("src/lib.rs"), b"new\n");
    if !made.get() {
        eprintln!("skipped: could not create a directory link");
        return;
    }
    let err = written.err();
    assert_eq!(err.as_ref().map(WorkspaceError::code), Some("path-unsafe"));
    assert!(std::fs::read_dir(&outside).map_or(0, |d| d.count()) == 0);
}

#[test]
fn an_injected_fault_leaves_nothing_behind() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let before = f.snapshot();
    ws.faults.fail_write.set(Some(1));
    let err = ws.write(&path("new/dir/x.txt"), b"x").err();
    assert_eq!(err.as_ref().map(WorkspaceError::code), Some("io"));
    assert_eq!(f.snapshot(), before);
    assert!(!f.path().join("new").exists());
}
```

File `tests/support/fixture.rs` (full contents):

```rust
//! A temporary copy of `tests/fixtures/greeter` as a git repository of its own, for the tests of
//! planrun-exec and planrun-cli (included with `#[path]`; D2 keeps the workspace at four
//! crates). Never inside this repository: the copy lives under the OS temp directory, and every
//! git call gets an environment that keeps the user's configuration out (no system config, an
//! empty global config, a ceiling above the copy).

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A fixture copy: `<temp>/planrun-test-<pid>-<nanos>-<n>/repo`, with the empty global config
/// next to it. Removed on drop.
pub struct Fixture {
    base: PathBuf,
    repo: PathBuf,
}

impl Fixture {
    /// The greeter fixture, committed once, with `core.autocrlf` set as given. With `true`, the
    /// working tree is checked out again so that its text files have CRLF line endings.
    pub fn greeter(autocrlf: bool) -> Fixture {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let base =
            std::env::temp_dir().join(format!("planrun-test-{}-{nanos}-{n}", std::process::id()));
        let repo = base.join("repo");
        std::fs::create_dir_all(&repo).unwrap_or_else(|e| panic!("{}: {e}", repo.display()));
        std::fs::write(base.join("gitconfig"), "").unwrap_or_else(|e| panic!("{e}"));
        let fixture = Fixture { base, repo };
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/greeter");
        copy_dir(&source, &fixture.repo);
        fixture.git(&["init", "-q", "-b", "main"]);
        let top = fixture.git(&["rev-parse", "--show-toplevel"]);
        assert_eq!(
            canonical(Path::new(top.trim())),
            canonical(&fixture.repo),
            "git init did not make the copy a repository of its own"
        );
        fixture.git(&[
            "config",
            "core.autocrlf",
            if autocrlf { "true" } else { "false" },
        ]);
        fixture.git(&["add", "-A"]);
        fixture.git(&["commit", "-q", "-m", "fixture"]);
        if autocrlf {
            fixture.checkout_again();
        }
        fixture
    }

    /// The repository root.
    pub fn path(&self) -> &Path {
        &self.repo
    }

    /// A directory next to the copy, outside the repository (removed with it): where a test's
    /// links point.
    pub fn outside(&self) -> PathBuf {
        let dir = self.base.join("outside");
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
        dir
    }

    /// The environment every git call of a test gets.
    pub fn env(&self) -> Vec<(OsString, OsString)> {
        let var = |k: &str, v: &str| (OsString::from(k), OsString::from(v));
        vec![
            var("GIT_CONFIG_NOSYSTEM", "1"),
            (
                OsString::from("GIT_CONFIG_GLOBAL"),
                self.base.join("gitconfig").into_os_string(),
            ),
            (
                OsString::from("GIT_CEILING_DIRECTORIES"),
                self.base.clone().into_os_string(),
            ),
            var("GIT_AUTHOR_NAME", "planrun test"),
            var("GIT_AUTHOR_EMAIL", "test@planrun.invalid"),
            var("GIT_COMMITTER_NAME", "planrun test"),
            var("GIT_COMMITTER_EMAIL", "test@planrun.invalid"),
        ]
    }

    /// Runs git in the copy with [`Fixture::env`]; panics when it fails. Returns stdout.
    pub fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.repo)
            .envs(self.env())
            .output()
            .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// Writes `bytes` to `rel` (forward slashes) in the copy, creating directories.
    pub fn write(&self, rel: &str, bytes: &[u8]) {
        let path = self.repo.join(rel);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("{e}"));
        }
        std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }

    /// The bytes of `rel`, or `None` when it does not exist.
    pub fn read(&self, rel: &str) -> Option<Vec<u8>> {
        std::fs::read(self.repo.join(rel)).ok()
    }

    /// Every file under the copy, `.git` included: relative path → contents.
    pub fn snapshot(&self) -> BTreeMap<String, Vec<u8>> {
        let mut files = BTreeMap::new();
        collect(&self.repo, &self.repo, &mut files);
        files
    }

    /// `git status --porcelain` without refreshing the index.
    pub fn status(&self) -> String {
        self.git(&["--no-optional-locks", "status", "--porcelain"])
    }

    /// Deletes the tracked files and checks them out again, so that `core.autocrlf` applies.
    fn checkout_again(&self) {
        let listed = self.git(&["ls-files", "-z"]);
        for file in listed.split('\0').filter(|f| !f.is_empty()) {
            let path = self.repo.join(file);
            std::fs::remove_file(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        }
        self.git(&["checkout", "--", "."]);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        clear_read_only(&self.base);
        if let Err(e) = std::fs::remove_dir_all(&self.base) {
            eprintln!(
                "could not remove the fixture copy {}: {e}",
                self.base.display()
            );
        }
    }
}

/// The path canonicalized, or as given when that fails.
pub fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Copies the fixture as git stores it: its files are text with LF line endings in this
/// repository, so CRLF from a checkout with `core.autocrlf=true` is turned back into LF, and the
/// copy's blob ids are the ones the plans name (`aa80cf1`). A `target/` from building the
/// fixture is skipped.
fn copy_dir(from: &Path, to: &Path) {
    let entries = std::fs::read_dir(from).unwrap_or_else(|e| panic!("{}: {e}", from.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("{e}"));
        let target = to.join(entry.file_name());
        let kind = entry.file_type().unwrap_or_else(|e| panic!("{e}"));
        if kind.is_dir() && entry.file_name() == "target" {
            continue;
        }
        if kind.is_dir() {
            std::fs::create_dir_all(&target).unwrap_or_else(|e| panic!("{e}"));
            copy_dir(&entry.path(), &target);
            continue;
        }
        let bytes = std::fs::read(entry.path()).unwrap_or_else(|e| panic!("{e}"));
        let text = String::from_utf8(bytes).unwrap_or_else(|e| panic!("the fixture is text: {e}"));
        std::fs::write(&target, text.replace("\r\n", "\n")).unwrap_or_else(|e| panic!("{e}"));
    }
}

fn collect(root: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, files);
            continue;
        }
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let key = rel.to_string_lossy().replace('\\', "/");
        files.insert(key, std::fs::read(&path).unwrap_or_default());
    }
}

/// git writes its object files read-only; Windows refuses to delete those.
fn clear_read_only(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            clear_read_only(&path);
            continue;
        }
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        let mut permissions = meta.permissions();
        if permissions.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = std::fs::set_permissions(&path, permissions);
        }
    }
}
```

**Step 2 — verify**

Targeted: the workspace and line-ending unit tests:

```powershell
cargo test -p planrun-exec --lib
```

Expected (exit code 0; summary lines):

```text
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-exec/src/eol.rs crates/planrun-exec/src/lib.rs crates/planrun-exec/src/workspace.rs crates/planrun-exec/src/workspace_tests.rs tests/support/fixture.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/eol.rs' = 'e6be0adf176ba30d6cb138425f22f651af5a1820'
      'crates/planrun-exec/src/lib.rs' = '0d7797d1905435d36c113141133b101adbeb2688'
      'crates/planrun-exec/src/workspace.rs' = '0e137cb9fcb298e60ca9c5e7ba50dcc18a07e8dd'
      'crates/planrun-exec/src/workspace_tests.rs' = 'b15003c6bc2af4e1a1c31f127ec5c435c8152089'
      'tests/support/fixture.rs' = '5ad2e25d484826ad35bf7ca4d2b1397d8d4ff050'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P3-T4: the workspace (resolution, the writer, D52) and line endings (D54)" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P3-T5: the virtual tree (D52) and ActionHandler::simulate for the file actions (D53)

**Goal:** The virtual tree and the handlers. `VirtualTree` holds a temporary index and object directory, seeds paths from the disk on first use, applies patches with `git apply --cached`, and tells absent, deleted, present and unknown apart, with `path-collision` under `core.ignorecase`. `ActionHandler` moves to `handler.rs` and gets `simulate`; `HandlerSet::files()` registers the four file handlers. `Finding` is exec's diagnostic. The tests run the assay patches with `core.autocrlf` true and false and check that the repository is unchanged.

**Files**

- Create `crates/planrun-exec/src/finding.rs`
- Create `crates/planrun-exec/src/handler.rs`
- Modify `crates/planrun-exec/src/lib.rs`
- Create `crates/planrun-exec/src/tree.rs`
- Create `crates/planrun-exec/tests/simulate.rs`
- Create `crates/planrun-exec/tests/support/ir.rs`

**Step 1 — write the files**

File `crates/planrun-exec/src/finding.rs` (full contents):

```rust
//! What preflight, a simulation or the dry run found: a stable code, a severity, the plan span
//! and the path. The CLI renders findings like parse diagnostics; tests pin codes.

use planrun_model::{RepoPath, Span};

use crate::workspace::WorkspaceError;

/// How bad a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// The plan cannot run as it stands.
    Error,
    /// It can; something needs a look.
    Warning,
}

/// One finding. `code` is stable and kebab-case (`blob-mismatch`, `create-exists`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Severity.
    pub severity: Severity,
    /// Stable code.
    pub code: &'static str,
    /// The plan text it is about, when it is about an action or an entry.
    pub span: Option<Span>,
    /// The repository path it is about.
    pub path: Option<RepoPath>,
    /// What, for a human.
    pub message: String,
}

impl Finding {
    /// An error.
    pub fn error(
        code: &'static str,
        span: Option<Span>,
        path: Option<&RepoPath>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            code,
            span,
            path: path.cloned(),
            message: message.into(),
        }
    }

    /// A warning.
    pub fn warning(
        code: &'static str,
        span: Option<Span>,
        path: Option<&RepoPath>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            ..Self::error(code, span, path, message)
        }
    }

    /// An error made from a workspace or git failure.
    pub fn workspace(e: &WorkspaceError, span: Option<Span>, path: Option<&RepoPath>) -> Self {
        Self::error(e.code(), span, path, e.to_string())
    }

    /// True for an error.
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}
```

File `crates/planrun-exec/src/handler.rs` (full contents):

```rust
//! Action handlers: one per action kind, registered in a [`HandlerSet`] (the execution-side
//! plugin point, docs/extending.md §7). A handler simulates its action on a virtual tree (the
//! dry-run behaviour of extending.md §4); writing is the transaction's job (D53).

use planrun_model::{AbbrevBlobId, Action, ActionKind, PlanSource, RepoPath, Span, WriteMode};

use crate::eol;
use crate::finding::Finding;
use crate::tree::{State, VirtualTree};

/// What a simulation did, for the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Simulated {
    /// The path it changed.
    pub path: RepoPath,
    /// A short description (`create`, `aa80cf1 → 97bd92a`).
    pub detail: String,
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
        let (bytes, detail) = match mode {
            WriteMode::Create if present => return Err(exists(path, span)),
            WriteMode::Create => (body.to_vec(), "create"),
            _ if !present => return Err(missing(path, span, "; the plan overwrites it")),
            _ => (overwrite_bytes(tree, path, body, span)?, "overwrite"),
        };
        tree.put(path, bytes, span)?;
        Ok(Simulated {
            path: path.clone(),
            detail: detail.to_owned(),
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
        let id = |a: Option<&AbbrevBlobId>| a.map_or_else(|| "?".to_owned(), |a| a.to_string());
        Ok(Simulated {
            path: path.clone(),
            detail: format!("{} → {}", id(pre.as_ref()), id(post.as_ref())),
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
            detail: "replace".to_owned(),
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
            detail: if *via_git {
                "delete (git rm)"
            } else {
                "delete"
            }
            .to_owned(),
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
```

File `crates/planrun-exec/src/lib.rs` (full contents):

```rust
//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Modules (docs/roadmap.md; P3 built the first six):
//!
//! - [`workspace`]: the repository root, [`RepoPath`](planrun_model::RepoPath) resolution with
//!   the link and case check, and the one writer (temp file, second check, rename; D52).
//! - [`eol`]: the line endings of written files (D10, D54).
//! - [`git`]: a thin wrapper over the `git` binary: `apply` with the patch on stdin,
//!   `hash-object`, `status --porcelain` without optional locks, `rev-parse`, `cat-file`,
//!   `check-attr`, `update-index`, `ls-files`; [`BlobId`](git::BlobId).
//! - [`tree`]: the virtual tree, a temporary index and object directory (D52).
//! - [`handler`]: one [`ActionHandler`] per action kind, which simulates it on a virtual tree;
//!   the [`HandlerSet`] is the execution-side plugin point.
//! - [`finding`]: what preflight, a simulation and the dry run report.
//! - `shell`: persistent pwsh and bash sessions behind [`ShellSession`].
//! - `expect`: exit-code and summary-line matching.
//! - `policy`: command classes and the [`CommandClassifier`].
//! - `journal`: JSONL under `.git/planrun/`.
//! - `runner`: the state machine Task → Step → Action, talking to a [`Ui`].

pub mod eol;
pub mod finding;
pub mod git;
pub mod handler;
pub mod tree;
pub mod workspace;

pub use handler::{ActionHandler, HandlerSet};

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;

/// The command classes of decision D13, least to most dangerous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PolicyClass {
    /// Reads only (`git status`, `rg`).
    Read,
    /// Builds or tests (`cargo test`, `dotnet build`, `npm ci`).
    Build,
    /// Changes the repository (`git add`, `git commit`, `replace … -All`, file actions).
    Modify,
    /// Destroys or reaches outside (`Remove-Item -Recurse`, `git reset --hard`, `git push`,
    /// anything unclassified).
    Destructive,
}

/// Classifies a script: the most dangerous class of any command in it.
pub trait CommandClassifier: Send + Sync {
    /// The class of `script` run by `shell`.
    fn classify(&self, shell: planrun_model::Shell, script: &str) -> PolicyClass;
}

/// The result of running one shell block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRun {
    /// `$LASTEXITCODE` / `$?` as the session reported it; `None` when the block threw.
    pub exit: Option<i32>,
    /// Merged stdout and stderr, as UTF-8 (lossy).
    pub output: String,
}

/// A long-lived shell: variables, the working directory and functions persist across blocks.
pub trait ShellSession: Send {
    /// Runs `script`, streaming each output line to `on_line` as it arrives.
    fn run(&mut self, script: &str, on_line: &mut dyn FnMut(&str)) -> std::io::Result<ScriptRun>;
}

/// The state of an action's effects on disk, for resuming (decision D14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoneState {
    /// Every effect is present (postimages match).
    Done,
    /// No effect is present (preimages match).
    NotDone,
    /// Some are, some are not, or neither matches: the user decides.
    Unknown,
}

/// The interactive surface the runner talks to (decision D15: line prompts first).
pub trait Ui {
    /// Shows a line of progress.
    fn status(&mut self, line: &str);
}
```

File `crates/planrun-exec/src/tree.rs` (full contents):

```rust
//! The virtual tree (decision D52): a temporary index and object directory under the OS temp
//! directory, with the repository's objects as alternates. File actions are simulated on it;
//! patches go through `git apply --cached`. Nothing in the repository is written.

use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::ffi::OsString;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::path::{Path, PathBuf};

use planrun_model::{RepoPath, Span};

use crate::eol::Eol;
use crate::finding::Finding;
use crate::git::{Attributes, BlobId, GitError};
use crate::workspace::{Workspace, WorkspaceError};

/// What the tree knows about a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Not on disk when it was first seen.
    Absent,
    /// Deleted by an action of this run.
    Deleted,
    /// Present, with its blob (git's normalized view), its mode, and its working-tree bytes
    /// when known (not after a patch, until asked for).
    Present {
        /// The blob id, filtered.
        blob: BlobId,
        /// The index mode (`100644`).
        mode: String,
        /// The working-tree bytes.
        bytes: Option<Vec<u8>>,
    },
    /// A script or a manual step that did not run may have changed it (dry run, D55).
    Unknown,
}

impl State {
    /// True when the path exists in the tree.
    pub fn exists(&self) -> bool {
        matches!(self, State::Present { .. })
    }
}

struct Slot {
    spelled: RepoPath,
    state: State,
}

/// The virtual tree of one dry run or one step transaction.
pub struct VirtualTree<'w> {
    ws: &'w Workspace,
    dir: PathBuf,
    env: Vec<(OsString, OsString)>,
    slots: HashMap<String, Slot>,
    attributes: HashMap<String, Attributes>,
    checkout: HashMap<String, Eol>,
    one_line: Option<BlobId>,
}

impl<'w> VirtualTree<'w> {
    /// A fresh, empty tree over `ws`; paths are seeded from the disk on first use.
    pub fn new(ws: &'w Workspace) -> Result<Self, WorkspaceError> {
        let objects = ws.git().object_dir()?;
        let dir = std::env::temp_dir().join(format!(
            "planrun-tree-{}-{:016x}",
            std::process::id(),
            RandomState::new().build_hasher().finish()
        ));
        fs::create_dir_all(dir.join("objects")).map_err(|source| WorkspaceError::Io {
            path: dir.display().to_string(),
            source,
        })?;
        let env = vec![
            var("GIT_INDEX_FILE", &dir.join("index")),
            var("GIT_OBJECT_DIRECTORY", &dir.join("objects")),
            var("GIT_ALTERNATE_OBJECT_DIRECTORIES", &objects),
        ];
        Ok(Self {
            ws,
            dir,
            env,
            slots: HashMap::new(),
            attributes: HashMap::new(),
            checkout: HashMap::new(),
            one_line: None,
        })
    }

    /// The workspace this tree simulates.
    pub fn workspace(&self) -> &'w Workspace {
        self.ws
    }

    /// The key of `path`: case-folded with `core.ignorecase` (D52).
    fn key(&self, path: &RepoPath) -> String {
        if self.ws.ignorecase() {
            path.as_str().to_lowercase()
        } else {
            path.as_str().to_owned()
        }
    }

    /// The state of `path`, seeding it from the disk on first use. Another spelling of a path
    /// already in the tree is `path-collision`.
    pub fn state(&mut self, path: &RepoPath, span: Option<Span>) -> Result<&State, Finding> {
        let key = self.key(path);
        if let Some(slot) = self.slots.get(&key) {
            if slot.spelled != *path {
                return Err(collision(path, &slot.spelled, span));
            }
        } else {
            let state = self
                .seed(path)
                .map_err(|e| Finding::workspace(&e, span, Some(path)))?;
            let spelled = path.clone();
            self.slots.insert(key.clone(), Slot { spelled, state });
        }
        self.slots
            .get(&key)
            .map(|s| &s.state)
            .ok_or_else(|| Finding::error("internal", span, Some(path), "a seeded path vanished"))
    }

    fn seed(&self, path: &RepoPath) -> Result<State, WorkspaceError> {
        let Some(bytes) = self.ws.read(path)? else {
            return Ok(State::Absent);
        };
        let git = self.ws.git();
        let mode = git
            .index_entry(path, &[])?
            .map_or_else(|| "100644".to_owned(), |(mode, _)| mode);
        let blob = git.hash_object(path, &bytes, true, true, &self.env)?;
        git.index_add(path, &mode, &blob, &self.env)?;
        Ok(State::Present {
            blob,
            mode,
            bytes: Some(bytes),
        })
    }

    fn slot_mut(&mut self, path: &RepoPath) -> Option<&mut Slot> {
        let key = self.key(path);
        self.slots.get_mut(&key)
    }

    /// The working-tree bytes of a present path: as seeded or written, or for a patched path
    /// what a checkout would write (`cat-file --filters`, D53).
    pub fn bytes(&mut self, path: &RepoPath, span: Option<Span>) -> Result<Vec<u8>, Finding> {
        let state = self.state(path, span)?.clone();
        let State::Present { blob, mode, bytes } = state else {
            return Err(Finding::error(
                "target-missing",
                span,
                Some(path),
                format!("`{path}` does not exist"),
            ));
        };
        if let Some(bytes) = bytes {
            return Ok(bytes);
        }
        let git = self.ws.git();
        let out = git
            .cat_filtered(&blob, path, &self.env)
            .map_err(|e| git_finding(&e, span, path))?;
        if let Some(slot) = self.slot_mut(path) {
            slot.state = State::Present {
                blob,
                mode,
                bytes: Some(out.clone()),
            };
        }
        Ok(out)
    }

    /// The blob of a present path.
    pub fn blob(&mut self, path: &RepoPath, span: Option<Span>) -> Result<Option<BlobId>, Finding> {
        Ok(match self.state(path, span)? {
            State::Present { blob, .. } => Some(*blob),
            _ => None,
        })
    }

    /// Puts `bytes` at `path` (filtered blob id, `hash-object -w --path` into the tree's objects).
    pub fn put(
        &mut self,
        path: &RepoPath,
        bytes: Vec<u8>,
        span: Option<Span>,
    ) -> Result<(), Finding> {
        let mode = match self.state(path, span)? {
            State::Present { mode, .. } => mode.clone(),
            _ => "100644".to_owned(),
        };
        let git = self.ws.git();
        let blob = git
            .hash_object(path, &bytes, true, true, &self.env)
            .and_then(|blob| git.index_add(path, &mode, &blob, &self.env).map(|()| blob))
            .map_err(|e| git_finding(&e, span, path))?;
        self.set(
            path,
            State::Present {
                blob,
                mode,
                bytes: Some(bytes),
            },
        );
        Ok(())
    }

    /// Applies `patch` to the tree's index (`git apply --cached`, after `--check`). The error is
    /// git's message.
    pub fn apply(
        &mut self,
        path: &RepoPath,
        patch: &[u8],
        span: Option<Span>,
    ) -> Result<(), String> {
        self.state(path, span).map_err(|f| f.message)?;
        let git = self.ws.git();
        git.apply(patch, true, true, &self.env)
            .and_then(|()| git.apply(patch, true, false, &self.env))
            .map_err(|e| git_message(&e))?;
        let entry = git
            .index_entry(path, &self.env)
            .map_err(|e| git_message(&e))?;
        let state = match entry {
            Some((mode, blob)) => State::Present {
                blob,
                mode,
                bytes: None,
            },
            None => State::Deleted,
        };
        self.set(path, state);
        Ok(())
    }

    /// Deletes `path` from the tree.
    pub fn delete(&mut self, path: &RepoPath, span: Option<Span>) -> Result<(), Finding> {
        self.state(path, span)?;
        self.ws
            .git()
            .index_remove(&[path], &self.env)
            .map_err(|e| git_finding(&e, span, path))?;
        self.set(path, State::Deleted);
        Ok(())
    }

    /// Marks `path` as changed by something that did not run (D55).
    pub fn mark_unknown(&mut self, path: &RepoPath) {
        let key = self.key(path);
        let spelled = self
            .slots
            .get(&key)
            .map_or_else(|| path.clone(), |s| s.spelled.clone());
        self.slots.insert(
            key,
            Slot {
                spelled,
                state: State::Unknown,
            },
        );
    }

    /// True when `path` was marked unknown.
    pub fn is_unknown(&self, path: &RepoPath) -> bool {
        self.slots
            .get(&self.key(path))
            .is_some_and(|s| s.state == State::Unknown)
    }

    fn set(&mut self, path: &RepoPath, state: State) {
        if let Some(slot) = self.slot_mut(path) {
            slot.state = state;
        }
    }

    /// The `eol` and `text` attributes of `path`, read once per tree (D54: a step's attributes
    /// are those at its start).
    pub fn attributes(
        &mut self,
        path: &RepoPath,
        span: Option<Span>,
    ) -> Result<Attributes, Finding> {
        let key = self.key(path);
        if let Some(found) = self.attributes.get(&key) {
            return Ok(found.clone());
        }
        let found = self
            .ws
            .git()
            .attributes(&[path])
            .map_err(|e| git_finding(&e, span, path))?
            .pop()
            .unwrap_or_default();
        self.attributes.insert(key, found.clone());
        Ok(found)
    }

    /// git's checkout convention for `path` (D54): a one-line blob converted as a checkout of
    /// that path would convert it.
    pub fn checkout_eol(&mut self, path: &RepoPath, span: Option<Span>) -> Result<Eol, Finding> {
        let key = self.key(path);
        if let Some(eol) = self.checkout.get(&key) {
            return Ok(*eol);
        }
        let git = self.ws.git();
        let blob = match self.one_line {
            Some(blob) => blob,
            None => git
                .hash_object(path, b"a\n", false, true, &self.env)
                .map_err(|e| git_finding(&e, span, path))?,
        };
        self.one_line = Some(blob);
        let out = git
            .cat_filtered(&blob, path, &self.env)
            .map_err(|e| git_finding(&e, span, path))?;
        let eol = if out.ends_with(b"\r\n") {
            Eol::Crlf
        } else {
            Eol::Lf
        };
        self.checkout.insert(key, eol);
        Ok(eol)
    }
}

impl Drop for VirtualTree<'_> {
    fn drop(&mut self) {
        clear_read_only(&self.dir);
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn var(name: &str, path: &Path) -> (OsString, OsString) {
    (OsString::from(name), path.as_os_str().to_os_string())
}

fn collision(path: &RepoPath, other: &RepoPath, span: Option<Span>) -> Finding {
    Finding::error(
        "path-collision",
        span,
        Some(path),
        format!("`{path}` and `{other}` are one file here (core.ignorecase)"),
    )
}

fn git_message(e: &GitError) -> String {
    match e {
        GitError::Failed { stderr, .. } => stderr.clone(),
        other => other.to_string(),
    }
}

fn git_finding(e: &GitError, span: Option<Span>, path: &RepoPath) -> Finding {
    Finding::error("git", span, Some(path), e.to_string())
}

/// git writes object files read-only; Windows refuses to delete those.
fn clear_read_only(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            clear_read_only(&path);
            continue;
        }
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        let mut permissions = meta.permissions();
        if permissions.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = fs::set_permissions(&path, permissions);
        }
    }
}
```

File `crates/planrun-exec/tests/simulate.rs` (full contents):

```rust
//! File actions simulated on the virtual tree (D52, D53, D54) over a fixture copy: what each
//! handler accepts and refuses, and that the repository stays unchanged.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[path = "support/ir.rs"]
mod ir;

use fixture::Fixture;
use ir::{Ir, PATCH_T1, PATCH_T2, path};
use planrun_exec::HandlerSet;
use planrun_exec::finding::Finding;
use planrun_exec::handler::Simulated;
use planrun_exec::tree::VirtualTree;
use planrun_exec::workspace::Workspace;
use planrun_model::{Action, WriteMode};

fn open(f: &Fixture) -> Workspace {
    Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
}

/// Simulates `actions` in order on one tree; the outcome of each, as `ok <detail>` or the code.
fn run(ws: &Workspace, ir: &Ir, actions: &[Action]) -> Vec<String> {
    let handlers = HandlerSet::files();
    let source = ir.source();
    let mut tree = VirtualTree::new(ws).unwrap_or_else(|e| panic!("{e}"));
    actions
        .iter()
        .map(|a| {
            let handler = handlers.find(a).unwrap_or_else(|| panic!("no handler"));
            let outcome: Result<Simulated, Finding> = handler.simulate(a, &source, &mut tree);
            match outcome {
                Ok(s) => format!("ok {}", s.detail),
                Err(f) => f.code.to_owned(),
            }
        })
        .collect()
}

/// The bytes the tree holds for `p` after `actions`.
fn bytes_after(ws: &Workspace, ir: &Ir, actions: &[Action], p: &str) -> Vec<u8> {
    let handlers = HandlerSet::files();
    let source = ir.source();
    let mut tree = VirtualTree::new(ws).unwrap_or_else(|e| panic!("{e}"));
    for a in actions {
        let handler = handlers.find(a).unwrap_or_else(|| panic!("no handler"));
        handler
            .simulate(a, &source, &mut tree)
            .unwrap_or_else(|f| panic!("{}: {}", f.code, f.message));
    }
    tree.bytes(&path(p), None)
        .unwrap_or_else(|f| panic!("{}", f.message))
}

#[test]
fn the_assay_patches_chain_on_the_tree_and_the_repository_stays_unchanged() {
    for autocrlf in [false, true] {
        let f = Fixture::greeter(autocrlf);
        let before = f.snapshot();
        let ws = open(&f);
        let mut ir = Ir::default();
        let actions = [
            ir.write("src/greet.rs", "pub fn greeting() {}\n", WriteMode::Create),
            ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a"),
            ir.patch("src/lib.rs", PATCH_T2, "97bd92a", "6b269fc"),
            ir.delete("src/old.rs", true),
        ];
        assert_eq!(
            run(&ws, &ir, &actions),
            [
                "ok create",
                "ok aa80cf1 → 97bd92a",
                "ok 97bd92a → 6b269fc",
                "ok delete (git rm)"
            ],
            "autocrlf={autocrlf}"
        );
        assert_eq!(
            f.snapshot(),
            before,
            "autocrlf={autocrlf}: the tree wrote to the repository"
        );
    }
}

#[test]
fn a_patched_file_reads_as_a_checkout_would_write_it() {
    for (autocrlf, crlf) in [(false, false), (true, true)] {
        let f = Fixture::greeter(autocrlf);
        let ws = open(&f);
        let mut ir = Ir::default();
        let actions = [ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a")];
        let bytes = bytes_after(&ws, &ir, &actions, "src/lib.rs");
        assert_eq!(bytes.windows(2).any(|w| w == b"\r\n"), crlf);
        assert!(String::from_utf8_lossy(&bytes).contains("pub mod greet;"));
    }
}

#[test]
fn a_changed_file_fails_the_preimage_check() {
    let f = Fixture::greeter(false);
    f.write("src/lib.rs", b"//! changed\n");
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a")];
    assert_eq!(run(&ws, &ir, &actions), ["blob-mismatch"]);
}

#[test]
fn a_patch_without_its_preimage_does_not_apply() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [ir.patch("src/lib.rs", PATCH_T2, "aa80cf1", "6b269fc")];
    assert_eq!(run(&ws, &ir, &actions), ["patch-does-not-apply"]);
}

#[test]
fn writes_check_existence_and_from_manifest_is_refused() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.write("src/lib.rs", "x\n", WriteMode::Create),
        ir.write("src/none.rs", "x\n", WriteMode::Overwrite),
        ir.write("src/x.rs", "x\n", WriteMode::FromManifest),
        ir.delete("src/none.rs", false),
        ir.replace("src/none.rs", "a", "b"),
    ];
    assert_eq!(
        run(&ws, &ir, &actions),
        [
            "create-exists",
            "target-missing",
            "write-mode-unknown",
            "target-missing",
            "target-missing"
        ]
    );
}

#[test]
fn a_deleted_path_can_be_created_again() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.delete("src/old.rs", false),
        ir.write("src/old.rs", "new\n", WriteMode::Create),
        ir.write("src/old.rs", "newer\n", WriteMode::Overwrite),
    ];
    assert_eq!(
        run(&ws, &ir, &actions),
        ["ok delete", "ok create", "ok overwrite"]
    );
}

#[test]
fn a_replace_keeps_crlf_and_needs_one_occurrence() {
    let f = Fixture::greeter(true);
    let ws = open(&f);
    let mut ir = Ir::default();
    let edit = [ir.replace(
        "src/old.rs",
        "format!(\"Hello {name}\")",
        "format!(\"Hi {name}\")",
    )];
    let bytes = bytes_after(&ws, &ir, &edit, "src/old.rs");
    assert!(String::from_utf8_lossy(&bytes).contains("Hi {name}"));
    assert!(!bytes.windows(2).any(|w| w[1] == b'\n' && w[0] != b'\r'));
    let mut ir = Ir::default();
    let actions = [
        ir.replace("src/old.rs", "name", "x"),
        ir.replace("src/old.rs", "absent text", "x"),
    ];
    assert_eq!(
        run(&ws, &ir, &actions),
        ["before-not-unique", "before-not-unique"]
    );
}

#[test]
fn a_mixed_file_refuses_an_exact_edit_and_is_overwritten_by_checkout() {
    let f = Fixture::greeter(true);
    f.write("mixed.txt", b"a\r\nb\n");
    let ws = open(&f);
    let mut ir = Ir::default();
    let edit = [ir.replace("mixed.txt", "a", "c")];
    assert_eq!(run(&ws, &ir, &edit), ["eol-mixed"]);
    let mut ir = Ir::default();
    let overwrite = [ir.write("mixed.txt", "x\ny\n", WriteMode::Overwrite)];
    assert_eq!(
        bytes_after(&ws, &ir, &overwrite, "mixed.txt"),
        b"x\r\ny\r\n"
    );
}

#[test]
fn a_file_without_line_endings_takes_the_checkout_convention() {
    for (autocrlf, expected) in [(false, &b"x\ny\n"[..]), (true, &b"x\r\ny\r\n"[..])] {
        let f = Fixture::greeter(autocrlf);
        f.write("one.txt", b"one line");
        let ws = open(&f);
        let mut ir = Ir::default();
        let actions = [ir.write("one.txt", "x\ny\n", WriteMode::Overwrite)];
        assert_eq!(
            bytes_after(&ws, &ir, &actions, "one.txt"),
            expected,
            "autocrlf={autocrlf}"
        );
    }
}

#[test]
fn an_eol_attribute_wins_and_created_files_stay_lf() {
    let f = Fixture::greeter(true);
    f.write(".gitattributes", b"*.lf eol=lf\n");
    f.write("a.lf", b"a\r\nb\r\n");
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.write("a.lf", "x\ny\n", WriteMode::Overwrite),
        ir.write("new.txt", "x\ny\n", WriteMode::Create),
    ];
    assert_eq!(bytes_after(&ws, &ir, &actions, "a.lf"), b"x\ny\n");
    assert_eq!(bytes_after(&ws, &ir, &actions, "new.txt"), b"x\ny\n");
}

#[test]
fn two_spellings_of_one_path_collide_with_ignorecase() {
    let f = Fixture::greeter(false);
    f.git(&["config", "core.ignorecase", "true"]);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.write("src/new.rs", "a\n", WriteMode::Create),
        ir.write("src/NEW.rs", "b\n", WriteMode::Create),
    ];
    assert_eq!(run(&ws, &ir, &actions), ["ok create", "path-collision"]);
}

#[test]
fn spellings_stay_apart_without_ignorecase() {
    let f = Fixture::greeter(false);
    f.git(&["config", "core.ignorecase", "false"]);
    if f.path().join("SRC").exists() {
        eprintln!("skipped: this file system ignores case, so the paths are one file on disk");
        return;
    }
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.write("src/new.rs", "a\n", WriteMode::Create),
        ir.write("src/NEW.rs", "b\n", WriteMode::Create),
    ];
    assert_eq!(run(&ws, &ir, &actions), ["ok create", "ok create"]);
}
```

File `crates/planrun-exec/tests/support/ir.rs` (full contents):

```rust
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
```

**Step 2 — verify**

Targeted: the simulations:

```powershell
cargo test -p planrun-exec --test simulate
```

Expected (exit code 0; summary lines):

```text
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-exec/src/finding.rs crates/planrun-exec/src/handler.rs crates/planrun-exec/src/lib.rs crates/planrun-exec/src/tree.rs crates/planrun-exec/tests/simulate.rs crates/planrun-exec/tests/support/ir.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/finding.rs' = 'f6ecec75dc70602a03973a9ac483c68bffa0564c'
      'crates/planrun-exec/src/handler.rs' = '27b534929bc7afb81ae20720902730bd0d27af19'
      'crates/planrun-exec/src/lib.rs' = '5e509a5ee2303e9ffb6086be953eef7291b5cc34'
      'crates/planrun-exec/src/tree.rs' = 'c9704b83f96a543b1f6a780fe25427bd204aa9a6'
      'crates/planrun-exec/tests/simulate.rs' = 'd291ad25e4ce52a0d30c3f3adc4dbcf6f29be5f4'
      'crates/planrun-exec/tests/support/ir.rs' = 'c6da4fd18511c2af30813095cff6ce3b48124417'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P3-T5: the virtual tree (D52) and ActionHandler::simulate for the file actions (D53)" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P3-T6: step transactions: prepare, then commit, with rollback (D53)

**Goal:** Step transactions (D53). `prepare` simulates a step's file actions on a fresh tree, stores the pre-images with `--no-filters`, and returns a `PreparedStep` with every post-image's bytes and blob id and the directories it creates. `commit` hands it to a `WriteAhead` recorder first, writes with the one writer, removes `via_git` deletes from the index last, and on any error restores what it wrote. The fault test fails the 2nd and the 3rd of three writes (a CRLF file, an untracked file, a new directory) and checks the working tree byte for byte.

**Files**

- Modify `crates/planrun-exec/src/lib.rs`
- Create `crates/planrun-exec/src/transaction.rs`
- Create `crates/planrun-exec/src/transaction_tests.rs`

**Step 1 — write the files**

File `crates/planrun-exec/src/lib.rs` (full contents):

```rust
//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Modules (docs/roadmap.md; P3 built the first six):
//!
//! - [`workspace`]: the repository root, [`RepoPath`](planrun_model::RepoPath) resolution with
//!   the link and case check, and the one writer (temp file, second check, rename; D52).
//! - [`eol`]: the line endings of written files (D10, D54).
//! - [`git`]: a thin wrapper over the `git` binary: `apply` with the patch on stdin,
//!   `hash-object`, `status --porcelain` without optional locks, `rev-parse`, `cat-file`,
//!   `check-attr`, `update-index`, `ls-files`; [`BlobId`](git::BlobId).
//! - [`tree`]: the virtual tree, a temporary index and object directory (D52).
//! - [`handler`]: one [`ActionHandler`] per action kind, which simulates it on a virtual tree;
//!   the [`HandlerSet`] is the execution-side plugin point.
//! - [`transaction`]: a step's file actions as prepare, then commit, with rollback (D53).
//! - [`finding`]: what preflight, a simulation and the dry run report.
//! - `shell`: persistent pwsh and bash sessions behind [`ShellSession`].
//! - `expect`: exit-code and summary-line matching.
//! - `policy`: command classes and the [`CommandClassifier`].
//! - `journal`: JSONL under `.git/planrun/`.
//! - `runner`: the state machine Task → Step → Action, talking to a [`Ui`].

pub mod eol;
pub mod finding;
pub mod git;
pub mod handler;
pub mod transaction;
pub mod tree;
pub mod workspace;

pub use handler::{ActionHandler, HandlerSet};

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "../tests/support/ir.rs"]
mod ir;

/// The command classes of decision D13, least to most dangerous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PolicyClass {
    /// Reads only (`git status`, `rg`).
    Read,
    /// Builds or tests (`cargo test`, `dotnet build`, `npm ci`).
    Build,
    /// Changes the repository (`git add`, `git commit`, `replace … -All`, file actions).
    Modify,
    /// Destroys or reaches outside (`Remove-Item -Recurse`, `git reset --hard`, `git push`,
    /// anything unclassified).
    Destructive,
}

/// Classifies a script: the most dangerous class of any command in it.
pub trait CommandClassifier: Send + Sync {
    /// The class of `script` run by `shell`.
    fn classify(&self, shell: planrun_model::Shell, script: &str) -> PolicyClass;
}

/// The result of running one shell block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRun {
    /// `$LASTEXITCODE` / `$?` as the session reported it; `None` when the block threw.
    pub exit: Option<i32>,
    /// Merged stdout and stderr, as UTF-8 (lossy).
    pub output: String,
}

/// A long-lived shell: variables, the working directory and functions persist across blocks.
pub trait ShellSession: Send {
    /// Runs `script`, streaming each output line to `on_line` as it arrives.
    fn run(&mut self, script: &str, on_line: &mut dyn FnMut(&str)) -> std::io::Result<ScriptRun>;
}

/// The state of an action's effects on disk, for resuming (decision D14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoneState {
    /// Every effect is present (postimages match).
    Done,
    /// No effect is present (preimages match).
    NotDone,
    /// Some are, some are not, or neither matches: the user decides.
    Unknown,
}

/// The interactive surface the runner talks to (decision D15: line prompts first).
pub trait Ui {
    /// Shows a line of progress.
    fn status(&mut self, line: &str);
}
```

File `crates/planrun-exec/src/transaction.rs` (full contents):

```rust
//! A step's file actions as one transaction (decisions D23, D53): **prepare** validates them on
//! a virtual tree seeded from the disk, stores the pre-images and builds every post-image;
//! **commit** hands the prepared step to a write-ahead recorder and then writes it with the one
//! writer, rolling back on any error.

use std::path::PathBuf;

use planrun_model::{Action, ActionKind, PlanSource, RepoPath};

use crate::finding::Finding;
use crate::git::BlobId;
use crate::handler::{HandlerSet, file_path};
use crate::tree::{State, VirtualTree};
use crate::workspace::{Workspace, WorkspaceError, remove_dirs};

/// What a path was before the step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreImage {
    /// It did not exist.
    Absent,
    /// Its exact bytes, stored with `hash-object -w --no-filters` (D52).
    Stored(BlobId),
}

/// What a path becomes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostImage {
    /// These working-tree bytes.
    Bytes(Vec<u8>),
    /// Removed.
    Delete,
}

/// One path the step changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedWrite {
    /// The path.
    pub path: RepoPath,
    /// Before.
    pub pre: PreImage,
    /// After.
    pub post: PostImage,
    /// The post-image's blob id as git will see it; `None` for a delete.
    pub post_blob: Option<BlobId>,
    /// The directories the write will create, from the root, outermost first.
    pub creates_dirs: Vec<String>,
}

/// A validated step, ready to commit: what P5 journals before writing (write-ahead, D53).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedStep {
    /// The paths, in the order their first action names them.
    pub writes: Vec<PlannedWrite>,
    /// The paths a `Delete` with `via_git` removes from the index after every write.
    pub index_removals: Vec<RepoPath>,
}

/// Records a prepared step before it is written. P5's recorder is the journal; a real run
/// without that record is not allowed (D53).
pub trait WriteAhead {
    /// Stores `step` durably; an error stops the commit before anything is written.
    fn record(&mut self, step: &PreparedStep) -> std::io::Result<()>;
}

/// Why a commit failed, and what rollback could not undo.
#[derive(Debug, thiserror::Error)]
#[error("{cause}{}", unrestored_note(.unrestored))]
pub struct CommitError {
    /// The first failure.
    pub cause: CommitCause,
    /// Paths rollback could not restore; empty when it restored everything.
    pub unrestored: Vec<RepoPath>,
}

/// The first failure of a commit.
#[derive(Debug, thiserror::Error)]
pub enum CommitCause {
    /// The write-ahead record failed; nothing was written.
    #[error("the write-ahead record failed: {0}")]
    Record(#[source] std::io::Error),
    /// A write, a delete or the index update failed.
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
}

fn unrestored_note(paths: &[RepoPath]) -> String {
    if paths.is_empty() {
        return String::new();
    }
    let list: Vec<&str> = paths.iter().map(RepoPath::as_str).collect();
    format!("; rollback could not restore {}", list.join(", "))
}

/// Validates `actions` (a step's file actions) on a fresh virtual tree, stores the pre-image of
/// every path they change in the repository's object database, and builds the post-images.
/// The first finding stops it; then nothing but pre-image objects was written.
pub fn prepare(
    ws: &Workspace,
    source: &PlanSource,
    handlers: &HandlerSet,
    actions: &[&Action],
) -> Result<PreparedStep, Finding> {
    let mut tree = VirtualTree::new(ws).map_err(|e| Finding::workspace(&e, None, None))?;
    let mut touched: Vec<RepoPath> = Vec::new();
    let mut index_removals = Vec::new();
    for action in actions {
        let span = Some(action.span);
        let path = file_path(&action.kind)
            .ok_or_else(|| Finding::error("internal", span, None, "not a file action"))?;
        let handler = handlers
            .find(action)
            .ok_or_else(|| Finding::error("internal", span, Some(path), "no handler"))?;
        handler.simulate(action, source, &mut tree)?;
        if !touched.contains(path) {
            touched.push(path.clone());
        }
        if matches!(action.kind, ActionKind::Delete { via_git: true, .. }) {
            index_removals.push(path.clone());
        }
    }
    let mut writes = Vec::new();
    for path in touched {
        if let Some(write) = planned(ws, &mut tree, path)? {
            writes.push(write);
        }
    }
    Ok(PreparedStep {
        writes,
        index_removals,
    })
}

/// The planned write of one touched path; `None` when it ends as it began (absent).
fn planned(
    ws: &Workspace,
    tree: &mut VirtualTree<'_>,
    path: RepoPath,
) -> Result<Option<PlannedWrite>, Finding> {
    let on_disk = ws
        .read(&path)
        .map_err(|e| Finding::workspace(&e, None, Some(&path)))?;
    let (post, post_blob) = match tree.state(&path, None)?.clone() {
        State::Present { blob, .. } => (PostImage::Bytes(tree.bytes(&path, None)?), Some(blob)),
        _ if on_disk.is_none() => return Ok(None),
        _ => (PostImage::Delete, None),
    };
    let pre = match &on_disk {
        None => PreImage::Absent,
        Some(bytes) => PreImage::Stored(
            ws.git()
                .hash_object(&path, bytes, false, true, &[])
                .map_err(|e| Finding::error("git", None, Some(&path), e.to_string()))?,
        ),
    };
    let resolved = ws
        .resolve(&path)
        .map_err(|e| Finding::workspace(&e, None, Some(&path)))?;
    let segments: Vec<&str> = path.as_str().split('/').collect();
    let creates_dirs = (resolved.existing.min(segments.len() - 1)..segments.len() - 1)
        .map(|i| segments[..=i].join("/"))
        .collect();
    Ok(Some(PlannedWrite {
        path,
        pre,
        post,
        post_blob,
        creates_dirs,
    }))
}

/// Writes `step`: first the write-ahead record, then every path in order, then the index
/// removals in one call. On an error every path already written is restored from its
/// pre-image, created files and directories are removed, and the error names what could not be
/// restored.
pub fn commit(
    ws: &Workspace,
    step: &PreparedStep,
    ahead: &mut dyn WriteAhead,
) -> Result<(), CommitError> {
    ahead.record(step).map_err(|e| CommitError {
        cause: CommitCause::Record(e),
        unrestored: Vec::new(),
    })?;
    let mut done: Vec<(&PlannedWrite, Vec<PathBuf>)> = Vec::new();
    for write in &step.writes {
        let result = match &write.post {
            PostImage::Bytes(bytes) => ws.write(&write.path, bytes),
            PostImage::Delete => ws.remove(&write.path).map(|_| Vec::new()),
        };
        match result {
            Ok(dirs) => done.push((write, dirs)),
            Err(e) => return Err(rollback(ws, &done, e)),
        }
    }
    let removals: Vec<&RepoPath> = step.index_removals.iter().collect();
    if let Err(e) = ws.git().index_remove(&removals, &[]) {
        return Err(rollback(ws, &done, e.into()));
    }
    Ok(())
}

/// Restores `done` in reverse order and reports what it could not restore.
fn rollback(
    ws: &Workspace,
    done: &[(&PlannedWrite, Vec<PathBuf>)],
    cause: WorkspaceError,
) -> CommitError {
    let mut unrestored = Vec::new();
    for (write, dirs) in done.iter().rev() {
        if restore(ws, write).is_err() {
            unrestored.push(write.path.clone());
        }
        remove_dirs(dirs);
    }
    CommitError {
        cause: CommitCause::Workspace(cause),
        unrestored,
    }
}

fn restore(ws: &Workspace, write: &PlannedWrite) -> Result<(), WorkspaceError> {
    match &write.pre {
        PreImage::Absent => ws.remove(&write.path).map(|_| ()),
        PreImage::Stored(id) => {
            let bytes = ws.git().cat_blob(id, &[])?;
            ws.write(&write.path, &bytes).map(|_| ())
        }
    }
}

#[cfg(test)]
#[path = "transaction_tests.rs"]
mod tests;
```

File `crates/planrun-exec/src/transaction_tests.rs` (full contents):

```rust
//! Step transactions on a fixture copy: prepare refuses before writing, commit writes what was
//! prepared, and an injected fault is rolled back byte for byte (D53).

use std::collections::BTreeMap;

use planrun_model::{Action, WriteMode};

use super::*;
use crate::fixture::Fixture;
use crate::ir::{Ir, PATCH_T1, abbrev, path};

#[derive(Default)]
struct Recorder {
    steps: Vec<PreparedStep>,
    fail: bool,
}

impl WriteAhead for Recorder {
    fn record(&mut self, step: &PreparedStep) -> std::io::Result<()> {
        if self.fail {
            return Err(std::io::Error::other("the journal is full"));
        }
        self.steps.push(step.clone());
        Ok(())
    }
}

fn open(f: &Fixture) -> Workspace {
    Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
}

fn prepared(ws: &Workspace, ir: &Ir, actions: &[Action]) -> Result<PreparedStep, Finding> {
    let refs: Vec<&Action> = actions.iter().collect();
    prepare(ws, &ir.source(), &HandlerSet::files(), &refs)
}

/// The working tree without `.git` (prepare stores pre-images there by design).
fn work_tree(f: &Fixture) -> BTreeMap<String, Vec<u8>> {
    f.snapshot()
        .into_iter()
        .filter(|(k, _)| !k.starts_with(".git/"))
        .collect()
}

#[test]
fn a_create_onto_an_existing_file_fails_before_anything_is_written() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let before = f.snapshot();
    let mut ir = Ir::default();
    let actions = [
        ir.write("src/greet.rs", "a\n", WriteMode::Create),
        ir.write("src/lib.rs", "b\n", WriteMode::Create),
    ];
    let err = prepared(&ws, &ir, &actions).err();
    assert_eq!(err.map(|f| f.code), Some("create-exists"));
    assert_eq!(
        f.snapshot(),
        before,
        "nothing, not even a pre-image, was written"
    );
}

#[test]
fn commit_writes_the_prepared_step_after_recording_it() {
    let f = Fixture::greeter(true);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.write("src/greet.rs", "pub fn greeting() {}\n", WriteMode::Create),
        ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a"),
        ir.delete("src/old.rs", true),
    ];
    let step = prepared(&ws, &ir, &actions).unwrap_or_else(|f| panic!("{}", f.message));
    assert_eq!(step.writes.len(), 3);
    assert_eq!(step.index_removals, [path("src/old.rs")]);
    let mut recorder = Recorder::default();
    commit(&ws, &step, &mut recorder).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(recorder.steps, [step]);
    let lib = f.read("src/lib.rs").unwrap_or_default();
    assert!(
        lib.windows(2).any(|w| w == b"\r\n"),
        "checked out as git apply would"
    );
    let id = ws
        .git()
        .hash_object(&path("src/lib.rs"), &lib, true, false, &[])
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(id.matches(&abbrev("97bd92a")));
    assert_eq!(
        f.read("src/greet.rs").as_deref(),
        Some(&b"pub fn greeting() {}\n"[..])
    );
    let mut status: Vec<String> = f.status().lines().map(str::to_owned).collect();
    status.sort();
    assert_eq!(
        status,
        [" M src/lib.rs", "?? src/greet.rs", "D  src/old.rs"]
    );
}

#[test]
fn a_failed_record_writes_nothing() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [ir.write("src/new.rs", "a\n", WriteMode::Create)];
    let step = prepared(&ws, &ir, &actions).unwrap_or_else(|f| panic!("{}", f.message));
    let before = work_tree(&f);
    let mut recorder = Recorder {
        fail: true,
        ..Recorder::default()
    };
    let err = commit(&ws, &step, &mut recorder).err();
    assert!(matches!(err.map(|e| e.cause), Some(CommitCause::Record(_))));
    assert_eq!(work_tree(&f), before);
}

#[test]
fn a_fault_in_a_step_is_rolled_back_byte_for_byte() {
    for failing in [2, 3] {
        let f = Fixture::greeter(true);
        f.write("notes.txt", b"untracked\r\nnotes\n");
        let ws = open(&f);
        let before = work_tree(&f);
        let mut ir = Ir::default();
        let actions = [
            ir.write("src/lib.rs", "//! replaced\n", WriteMode::Overwrite),
            ir.write("notes.txt", "replaced\n", WriteMode::Overwrite),
            ir.write("new/dir/x.txt", "x\n", WriteMode::Create),
        ];
        let step = prepared(&ws, &ir, &actions).unwrap_or_else(|f| panic!("{}", f.message));
        ws.faults.fail_write.set(Some(failing));
        let err = commit(&ws, &step, &mut Recorder::default())
            .err()
            .unwrap_or_else(|| panic!("write {failing} should fail"));
        assert!(err.unrestored.is_empty(), "{err}");
        assert_eq!(work_tree(&f), before, "write {failing}: not restored");
        assert!(
            !f.path().join("new").exists(),
            "write {failing}: directory left"
        );
        let temps = f
            .snapshot()
            .into_keys()
            .filter(|k| k.contains(".planrun-"))
            .count();
        assert_eq!(temps, 0, "write {failing}: temp files left");
    }
}

#[test]
fn a_path_that_ends_as_it_began_is_not_written() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.write("tmp.txt", "a\n", WriteMode::Create),
        ir.delete("tmp.txt", false),
    ];
    let step = prepared(&ws, &ir, &actions).unwrap_or_else(|f| panic!("{}", f.message));
    assert!(step.writes.is_empty());
}

#[test]
fn prepare_lists_the_directories_a_write_creates() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [ir.write("docs/a/b.txt", "x\n", WriteMode::Create)];
    let step = prepared(&ws, &ir, &actions).unwrap_or_else(|f| panic!("{}", f.message));
    assert_eq!(step.writes[0].creates_dirs, ["docs", "docs/a"]);
    assert_eq!(step.writes[0].pre, PreImage::Absent);
}
```

**Step 2 — verify**

Targeted: the transaction tests:

```powershell
cargo test -p planrun-exec --lib transaction
```

Expected (exit code 0; summary lines):

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in …s
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-exec/src/lib.rs crates/planrun-exec/src/transaction.rs crates/planrun-exec/src/transaction_tests.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/lib.rs' = 'e13175fe38ea8d896a2252772d60055453fdf474'
      'crates/planrun-exec/src/transaction.rs' = '3f192ccb4654c1ce3de3885f11e98fb871d04560'
      'crates/planrun-exec/src/transaction_tests.rs' = '38c72af06e8f392ee72d9a66bdf1d75e9a3b3e2b'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P3-T6: step transactions: prepare, then commit, with rollback (D53)" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P3-T7: task preflight and the dry run: scope (D50), real dirt and steps that did not run (D55)

**Goal:** Preflight and the dry run. `dry_run` checks real dirt once, then walks every task over one virtual tree: the manifest checks, then each action, with D50's scope (`would ask`) and `path-ambiguous`, and D55's rule for steps that did not run. `preflight` does the same for one task on a fresh tree and stops at its first script or Manual. Tests on hand-built IR.

**Files**

- Modify `crates/planrun-exec/src/lib.rs`
- Create `crates/planrun-exec/src/preflight.rs`
- Create `crates/planrun-exec/tests/preflight.rs`

**Step 1 — write the files**

File `crates/planrun-exec/src/lib.rs` (full contents):

```rust
//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Modules (docs/roadmap.md; P3 built the first eight):
//!
//! - [`workspace`]: the repository root, [`RepoPath`](planrun_model::RepoPath) resolution with
//!   the link and case check, and the one writer (temp file, second check, rename; D52).
//! - [`eol`]: the line endings of written files (D10, D54).
//! - [`git`]: a thin wrapper over the `git` binary: `apply` with the patch on stdin,
//!   `hash-object`, `status --porcelain` without optional locks, `rev-parse`, `cat-file`,
//!   `check-attr`, `update-index`, `ls-files`; [`BlobId`](git::BlobId).
//! - [`tree`]: the virtual tree, a temporary index and object directory (D52).
//! - [`handler`]: one [`ActionHandler`] per action kind, which simulates it on a virtual tree;
//!   the [`HandlerSet`] is the execution-side plugin point.
//! - [`transaction`]: a step's file actions as prepare, then commit, with rollback (D53).
//! - [`preflight`]: task preflight and the dry run over one virtual tree (D50, D55).
//! - [`finding`]: what preflight, a simulation and the dry run report.
//! - `shell`: persistent pwsh and bash sessions behind [`ShellSession`].
//! - `expect`: exit-code and summary-line matching.
//! - `policy`: command classes and the [`CommandClassifier`].
//! - `journal`: JSONL under `.git/planrun/`.
//! - `runner`: the state machine Task → Step → Action, talking to a [`Ui`].

pub mod eol;
pub mod finding;
pub mod git;
pub mod handler;
pub mod preflight;
pub mod transaction;
pub mod tree;
pub mod workspace;

pub use handler::{ActionHandler, HandlerSet};

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "../tests/support/ir.rs"]
mod ir;

/// The command classes of decision D13, least to most dangerous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PolicyClass {
    /// Reads only (`git status`, `rg`).
    Read,
    /// Builds or tests (`cargo test`, `dotnet build`, `npm ci`).
    Build,
    /// Changes the repository (`git add`, `git commit`, `replace … -All`, file actions).
    Modify,
    /// Destroys or reaches outside (`Remove-Item -Recurse`, `git reset --hard`, `git push`,
    /// anything unclassified).
    Destructive,
}

/// Classifies a script: the most dangerous class of any command in it.
pub trait CommandClassifier: Send + Sync {
    /// The class of `script` run by `shell`.
    fn classify(&self, shell: planrun_model::Shell, script: &str) -> PolicyClass;
}

/// The result of running one shell block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRun {
    /// `$LASTEXITCODE` / `$?` as the session reported it; `None` when the block threw.
    pub exit: Option<i32>,
    /// Merged stdout and stderr, as UTF-8 (lossy).
    pub output: String,
}

/// A long-lived shell: variables, the working directory and functions persist across blocks.
pub trait ShellSession: Send {
    /// Runs `script`, streaming each output line to `on_line` as it arrives.
    fn run(&mut self, script: &str, on_line: &mut dyn FnMut(&str)) -> std::io::Result<ScriptRun>;
}

/// The state of an action's effects on disk, for resuming (decision D14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoneState {
    /// Every effect is present (postimages match).
    Done,
    /// No effect is present (preimages match).
    NotDone,
    /// Some are, some are not, or neither matches: the user decides.
    Unknown,
}

/// The interactive surface the runner talks to (decision D15: line prompts first).
pub trait Ui {
    /// Shows a line of progress.
    fn status(&mut self, line: &str);
}
```

File `crates/planrun-exec/src/preflight.rs` (full contents):

```rust
//! Task preflight and the dry run (docs/architecture.md §6.1, §7; decisions D50, D55).
//!
//! Both walk a task over a virtual tree: the manifest checks first (file entries, not Bare, not
//! patterns), then the actions in order. The dry run carries one tree across every task and
//! simulates every file action; a real run's preflight (P5) uses a fresh tree and stops at the
//! task's first script or Manual, because later file actions may depend on it.

use std::collections::HashSet;

use planrun_model::{
    Action, ActionKind, ChangeKind, ManifestEntry, ManifestPath, PathOrigin, Plan, RepoPath, Task,
    WriteMode,
};

use crate::finding::{Finding, Severity};
use crate::handler::{HandlerSet, file_path};
use crate::tree::VirtualTree;
use crate::workspace::{Workspace, WorkspaceError};

/// What became of one action in a dry run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// A file action that would run on its own.
    Ok(String),
    /// A file action that would run after you confirm it: unscoped (D50).
    WouldAsk(String),
    /// A file action that would fail.
    Failed(Finding),
    /// A file action whose result depends on a script or Manual step that did not run (D55).
    Depends(Finding),
    /// A script or a commit: never run in a dry run.
    NotRun,
    /// A `Generated` check: its script did not run.
    NotChecked,
    /// A Manual: planrun would wait for you.
    Waits,
    /// A Note: shown in place.
    Shown,
}

/// One action of the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionReport {
    /// The step's position in the task, from 0.
    pub step: usize,
    /// The action.
    pub action: Action,
    /// What became of it.
    pub outcome: Outcome,
}

/// One task of the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskReport {
    /// The task's position in the plan, from 0.
    pub task: usize,
    /// What the manifest checks found.
    pub manifest: Vec<Finding>,
    /// The actions in order.
    pub actions: Vec<ActionReport>,
}

/// The dry run of a plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRun {
    /// What the start found: dirt in the paths the plan touches (errors) or elsewhere (a
    /// warning).
    pub start: Vec<Finding>,
    /// Per task.
    pub tasks: Vec<TaskReport>,
    /// Paths a file action writes whose attributes the dry run reads from the disk as they are
    /// (a `.gitattributes` the plan writes, D54).
    pub attribute_writes: Vec<RepoPath>,
}

impl DryRun {
    /// Every finding: the start, the manifests, the actions.
    pub fn findings(&self) -> impl Iterator<Item = &Finding> {
        let actions = self.tasks.iter().flat_map(|t| {
            t.manifest
                .iter()
                .chain(t.actions.iter().filter_map(|a| match &a.outcome {
                    Outcome::Failed(f) | Outcome::Depends(f) => Some(f),
                    _ => None,
                }))
        });
        self.start.iter().chain(actions)
    }

    /// True when anything would fail.
    pub fn has_errors(&self) -> bool {
        self.findings().any(Finding::is_error)
    }
}

/// Walks tasks over one tree: whether a script or Manual has gone by without running.
struct Walk<'t, 'w> {
    tree: &'t mut VirtualTree<'w>,
    handlers: HandlerSet,
    unrun: bool,
}

/// The dry run of `plan` (D55): parse and checks are done; this preflights every task against
/// one virtual tree, checks real dirt once for the paths of every task, and simulates every file
/// action. Writes nothing. An error here is a git or file-system failure, not a finding.
pub fn dry_run(ws: &Workspace, plan: &Plan) -> Result<DryRun, WorkspaceError> {
    let start = dirt(ws, plan.tasks.iter())?;
    let mut tree = VirtualTree::new(ws)?;
    let mut walk = Walk {
        tree: &mut tree,
        handlers: HandlerSet::files(),
        unrun: false,
    };
    let tasks = plan
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| walk.task(plan, i, task, false))
        .collect();
    let attribute_writes = plan
        .tasks
        .iter()
        .flat_map(|t| t.steps.iter().flat_map(|s| s.actions.iter()))
        .filter_map(|a| file_path(&a.kind))
        .filter(|p| p.file_name() == ".gitattributes")
        .cloned()
        .collect();
    Ok(DryRun {
        start,
        tasks,
        attribute_writes,
    })
}

/// A real run's preflight of `task` on the disk as it is now (P5 runs it before each task):
/// dirt in the task's paths, the manifest checks, and the file actions up to the first script
/// or Manual. Every error finding stops the task.
pub fn preflight(ws: &Workspace, plan: &Plan, task: &Task) -> Result<Vec<Finding>, WorkspaceError> {
    let mut found = dirt(ws, std::iter::once(task))?;
    let mut tree = VirtualTree::new(ws)?;
    let mut walk = Walk {
        tree: &mut tree,
        handlers: HandlerSet::files(),
        unrun: false,
    };
    let report = walk.task(plan, 0, task, true);
    found.extend(report.manifest);
    found.extend(report.actions.into_iter().filter_map(|a| match a.outcome {
        Outcome::Failed(f) | Outcome::Depends(f) => Some(f),
        _ => None,
    }));
    Ok(found)
}

impl Walk<'_, '_> {
    fn task(&mut self, plan: &Plan, index: usize, task: &Task, stop_at_unrun: bool) -> TaskReport {
        let manifest = task
            .manifest
            .entries
            .iter()
            .filter_map(|e| self.entry(e))
            .collect();
        let mut actions = Vec::new();
        for (step, s) in task.steps.iter().enumerate() {
            for action in &s.actions {
                if stop_at_unrun && self.unrun {
                    return TaskReport {
                        task: index,
                        manifest,
                        actions,
                    };
                }
                let outcome = self.action(plan, task, action);
                actions.push(ActionReport {
                    step,
                    action: action.clone(),
                    outcome,
                });
            }
        }
        TaskReport {
            task: index,
            manifest,
            actions,
        }
    }

    /// The manifest check of one entry: Create absent; Modify, Rewrite, Delete present. Test
    /// and Generated entries, patterns (D41) and Bare entries (D45) are not checked.
    fn entry(&mut self, entry: &ManifestEntry) -> Option<Finding> {
        let ManifestPath::File(path) = &entry.path else {
            return None;
        };
        if entry.origin == PathOrigin::Bare {
            return None;
        }
        let must_exist = match entry.change {
            ChangeKind::Create => false,
            ChangeKind::Modify | ChangeKind::Rewrite | ChangeKind::Delete => true,
            ChangeKind::Test | ChangeKind::Generated => return None,
        };
        let span = Some(entry.span);
        if self.tree.is_unknown(path) {
            return None;
        }
        let exists = match self.tree.state(path, span) {
            Ok(state) => state.exists(),
            Err(f) => return Some(f),
        };
        let finding = match (must_exist, exists) {
            (false, true) => Finding::error(
                "create-exists",
                span,
                Some(path),
                format!("`{path}` already exists; the task's files say it is created"),
            ),
            (true, false) => Finding::error(
                "target-missing",
                span,
                Some(path),
                format!("`{path}` does not exist; the task's files say it is changed"),
            ),
            _ => return None,
        };
        Some(self.after_unrun(finding))
    }

    fn action(&mut self, plan: &Plan, task: &Task, action: &Action) -> Outcome {
        match &action.kind {
            ActionKind::RunScript(_) | ActionKind::Commit(_) => {
                self.unrun = true;
                Outcome::NotRun
            }
            ActionKind::Manual { .. } => {
                self.unrun = true;
                Outcome::Waits
            }
            ActionKind::Note { .. } => Outcome::Shown,
            ActionKind::Generated { .. } => Outcome::NotChecked,
            _ => self.file_action(plan, task, action),
        }
    }

    fn file_action(&mut self, plan: &Plan, task: &Task, action: &Action) -> Outcome {
        let Some(path) = file_path(&action.kind) else {
            return Outcome::Shown;
        };
        let span = Some(action.span);
        if let Some(f) = ambiguous(task, action, path) {
            return Outcome::Failed(f);
        }
        if self.tree.is_unknown(path) {
            return Outcome::Depends(depends(path, action));
        }
        let Some(handler) = self.handlers.find(action) else {
            return Outcome::Failed(Finding::error("internal", span, Some(path), "no handler"));
        };
        match handler.simulate(action, &plan.source, self.tree) {
            Ok(done) if task.manifest.scopes(path) => Outcome::Ok(done.detail),
            Ok(done) => Outcome::WouldAsk(done.detail),
            Err(f) if f.severity == Severity::Warning => Outcome::Depends(f),
            Err(f) if self.unrun && depends_on_state(f.code) => {
                self.tree.mark_unknown(path);
                Outcome::Depends(self.after_unrun(f))
            }
            Err(f) => Outcome::Failed(with_bare_hint(task, path, f)),
        }
    }

    /// After a script or Manual that did not run, a failure of the tree's state is a warning.
    fn after_unrun(&self, f: Finding) -> Finding {
        if !self.unrun || !depends_on_state(f.code) {
            return f;
        }
        Finding {
            severity: Severity::Warning,
            message: format!(
                "{} (depends on a script or Manual step that did not run)",
                f.message
            ),
            ..f
        }
    }
}

/// D50: a Bare-targeted Overwrite or Delete has nothing that confirms its target.
fn ambiguous(task: &Task, action: &Action, path: &RepoPath) -> Option<Finding> {
    let unconfirmed = matches!(
        action.kind,
        ActionKind::WriteFile {
            mode: WriteMode::Overwrite,
            ..
        } | ActionKind::Delete { .. }
    );
    if !unconfirmed || !task.manifest.is_bare_target(path) {
        return None;
    }
    Some(Finding::error(
        "path-ambiguous",
        Some(action.span),
        Some(path),
        format!(
            "`{path}` is a bare file name in the task's files: its directory is unknown, and \
             nothing in this action confirms the target (D45, D50)"
        ),
    ))
}

fn with_bare_hint(task: &Task, path: &RepoPath, f: Finding) -> Finding {
    if f.code != "target-missing" || !task.manifest.is_bare_target(path) {
        return f;
    }
    Finding {
        message: format!(
            "{}; it is a bare file name in the task's files, so its directory is unknown (D45)",
            f.message
        ),
        ..f
    }
}

fn depends(path: &RepoPath, action: &Action) -> Finding {
    Finding::warning(
        "unverified",
        Some(action.span),
        Some(path),
        format!("`{path}` depends on a script or Manual step that did not run"),
    )
}

/// The codes that report the tree's state, which an earlier script or Manual may have changed.
fn depends_on_state(code: &str) -> bool {
    matches!(
        code,
        "create-exists"
            | "target-missing"
            | "blob-mismatch"
            | "post-mismatch"
            | "patch-does-not-apply"
            | "before-not-unique"
            | "eol-mixed"
    )
}

/// The paths a task touches: its file actions, `Generated` paths and manifest file entries.
fn touched(task: &Task) -> impl Iterator<Item = &RepoPath> {
    let actions = task
        .steps
        .iter()
        .flat_map(|s| s.actions.iter())
        .filter_map(|a| match &a.kind {
            ActionKind::Generated { path } => Some(path),
            kind => file_path(kind),
        });
    actions.chain(task.manifest.entries.iter().filter_map(|e| e.path.file()))
}

/// Real dirt (D55), from one `git status`: a touched path with uncommitted changes is
/// `path-dirty`; any other dirt is one `tree-dirty` warning.
fn dirt<'p>(
    ws: &Workspace,
    tasks: impl Iterator<Item = &'p Task>,
) -> Result<Vec<Finding>, WorkspaceError> {
    let fold = |p: &str| {
        if ws.ignorecase() {
            p.to_lowercase()
        } else {
            p.to_owned()
        }
    };
    let mine: HashSet<String> = tasks.flat_map(touched).map(|p| fold(p.as_str())).collect();
    let (touched_dirt, other_dirt): (Vec<_>, Vec<_>) = ws
        .git()
        .status(None)?
        .into_iter()
        .partition(|e| mine.contains(&fold(&e.path)));
    let mut found: Vec<Finding> = touched_dirt
        .into_iter()
        .map(|e| {
            let path = RepoPath::new(&e.path).ok();
            Finding::error(
                "path-dirty",
                None,
                path.as_ref(),
                format!(
                    "`{}` has uncommitted changes (`{}`); the plan changes it",
                    e.path, e.code
                ),
            )
        })
        .collect();
    let other = other_dirt.len();
    if other > 0 {
        found.push(Finding::warning(
            "tree-dirty",
            None,
            None,
            format!("{other} other path(s) have uncommitted changes; the plan does not touch them"),
        ));
    }
    Ok(found)
}
```

File `crates/planrun-exec/tests/preflight.rs` (full contents):

```rust
//! Preflight and the dry run on hand-built IR (D2) over a fixture copy: scope (D50), Bare
//! targets, results that depend on a step that did not run (D55), and real dirt.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[path = "support/ir.rs"]
mod ir;

use fixture::Fixture;
use ir::{Ir, PATCH_T1, PATCH_T2};
use planrun_exec::preflight::{DryRun, Outcome, dry_run, preflight};
use planrun_exec::workspace::Workspace;
use planrun_model::{ChangeKind, Manifest, PathOrigin, Plan, WriteMode};

fn open(f: &Fixture) -> Workspace {
    Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
}

fn run(f: &Fixture, plan: &Plan) -> DryRun {
    dry_run(&open(f), plan).unwrap_or_else(|e| panic!("{e}"))
}

/// One label per action of task `t`: `ok`, `ask`, the code of a failure, `depends <code>`, …
fn labels(report: &DryRun, t: usize) -> Vec<String> {
    report.tasks[t]
        .actions
        .iter()
        .map(|a| match &a.outcome {
            Outcome::Ok(_) => "ok".to_owned(),
            Outcome::WouldAsk(_) => "ask".to_owned(),
            Outcome::Failed(f) => f.code.to_owned(),
            Outcome::Depends(f) => format!("depends {}", f.code),
            Outcome::NotRun => "not run".to_owned(),
            Outcome::NotChecked => "not checked".to_owned(),
            Outcome::Waits => "waits".to_owned(),
            Outcome::Shown => "note".to_owned(),
        })
        .collect()
}

const W: PathOrigin = PathOrigin::Written;

/// The two tasks of `assay-style.md`, by hand.
fn assay(ir: &mut Ir) -> Plan {
    let m1 = ir.manifest(&[
        ("src/greet.rs", ChangeKind::Create, W),
        ("src/lib.rs", ChangeKind::Modify, W),
    ]);
    let t1 = vec![
        vec![
            ir.write("src/greet.rs", "pub fn greeting() {}\n", WriteMode::Create),
            ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a"),
        ],
        vec![ir.script("cargo test")],
        vec![ir.commit("git commit")],
    ];
    let m2 = ir.manifest(&[
        ("src/lib.rs", ChangeKind::Modify, W),
        ("src/old.rs", ChangeKind::Delete, W),
        ("docs/modules.txt", ChangeKind::Create, W),
    ]);
    let t2 = vec![
        vec![
            ir.patch("src/lib.rs", PATCH_T2, "97bd92a", "6b269fc"),
            ir.delete("src/old.rs", false),
            ir.generated("docs/modules.txt"),
        ],
        vec![ir.script("cargo test")],
        vec![ir.commit("git commit")],
    ];
    let tasks = vec![ir.task("P1-T1", m1, t1), ir.task("P1-T2", m2, t2)];
    ir.plan(tasks)
}

#[test]
fn the_assay_plan_passes_and_writes_nothing() {
    for autocrlf in [false, true] {
        let f = Fixture::greeter(autocrlf);
        let before = f.snapshot();
        let plan = assay(&mut Ir::default());
        let report = run(&f, &plan);
        assert!(
            !report.has_errors(),
            "{:?}",
            report.findings().collect::<Vec<_>>()
        );
        assert_eq!(labels(&report, 0), ["ok", "ok", "not run", "not run"]);
        assert_eq!(
            labels(&report, 1),
            ["ok", "ok", "not checked", "not run", "not run"]
        );
        assert_eq!(f.snapshot(), before, "autocrlf={autocrlf}");
    }
}

#[test]
fn preflight_finds_the_blob_mismatch_of_a_changed_file() {
    let f = Fixture::greeter(false);
    f.write("src/lib.rs", b"//! changed\n");
    f.git(&["commit", "-q", "-am", "changed"]);
    let plan = assay(&mut Ir::default());
    let found = preflight(&open(&f), &plan, &plan.tasks[0]).unwrap_or_else(|e| panic!("{e}"));
    let codes: Vec<_> = found.iter().map(|f| f.code).collect();
    assert_eq!(codes, ["blob-mismatch"]);
}

#[test]
fn a_dirty_touched_path_is_an_error_and_other_dirt_a_warning() {
    let f = Fixture::greeter(false);
    f.write("src/lib.rs", b"//! changed\n");
    f.write("unrelated.txt", b"x\n");
    let plan = assay(&mut Ir::default());
    let report = run(&f, &plan);
    let start: Vec<_> = report
        .start
        .iter()
        .map(|f| (f.code, f.is_error()))
        .collect();
    assert_eq!(start, [("path-dirty", true), ("tree-dirty", false)]);
}

#[test]
fn an_unscoped_write_would_ask() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let declared = ir.manifest(&[("a.txt", ChangeKind::Create, W)]);
    let steps = vec![vec![
        ir.write("a.txt", "a\n", WriteMode::Create),
        ir.write("b.txt", "b\n", WriteMode::Create),
    ]];
    let without = vec![vec![ir.write("c.txt", "c\n", WriteMode::Create)]];
    let tasks = vec![
        ir.task("T1", declared, steps),
        ir.task("T2", Manifest::default(), without),
    ];
    let report = run(&f, &ir.plan(tasks));
    assert_eq!(labels(&report, 0), ["ok", "ask"]);
    assert_eq!(labels(&report, 1), ["ask"]);
    assert!(!report.has_errors());
}

#[test]
fn a_bare_target_is_ambiguous_for_an_overwrite_or_a_delete() {
    let f = Fixture::greeter(false);
    f.write("lib.rs", b"a root file that happens to share the name\n");
    f.git(&["add", "lib.rs"]);
    f.git(&["commit", "-q", "-m", "root lib.rs"]);
    let mut ir = Ir::default();
    let m = ir.manifest(&[
        ("src/greet.rs", ChangeKind::Modify, W),
        ("lib.rs", ChangeKind::Modify, PathOrigin::Bare),
        ("missing.rs", ChangeKind::Modify, PathOrigin::Bare),
    ]);
    let steps = vec![vec![
        ir.write("lib.rs", "x\n", WriteMode::Overwrite),
        ir.delete("lib.rs", false),
        ir.replace("lib.rs", "a root file", "a file"),
        ir.replace("missing.rs", "x", "y"),
    ]];
    let tasks = vec![ir.task("T1", m, steps)];
    let report = run(&f, &ir.plan(tasks));
    assert_eq!(
        labels(&report, 0),
        ["path-ambiguous", "path-ambiguous", "ask", "target-missing"]
    );
    let Outcome::Failed(missing) = &report.tasks[0].actions[3].outcome else {
        panic!("target-missing expected");
    };
    assert!(missing.message.contains("D45"), "{}", missing.message);
}

#[test]
fn after_a_step_that_did_not_run_a_failure_is_a_warning_and_its_path_unknown() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let steps = vec![
        vec![ir.replace("src/old.rs", "absent text", "x")],
        vec![ir.script("cargo fmt")],
        vec![
            ir.replace("src/lib.rs", "absent text", "x"),
            ir.replace("src/lib.rs", "pub mod old;", "pub mod older;"),
            ir.write("src/old.rs", "x\n", WriteMode::Overwrite),
        ],
    ];
    let tasks = vec![ir.task("T1", Manifest::default(), steps)];
    let report = run(&f, &ir.plan(tasks));
    assert_eq!(
        labels(&report, 0),
        [
            "before-not-unique",
            "not run",
            "depends before-not-unique",
            "depends unverified",
            "ask"
        ]
    );
    let findings: Vec<_> = report.findings().map(|f| (f.code, f.is_error())).collect();
    assert_eq!(
        findings,
        [
            ("before-not-unique", true),
            ("before-not-unique", false),
            ("unverified", false)
        ]
    );
}

#[test]
fn a_manual_counts_as_a_step_that_did_not_run() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let steps = vec![vec![
        ir.manual("add a function by hand"),
        ir.replace("src/lib.rs", "a function added by hand", "x"),
    ]];
    let tasks = vec![ir.task("T1", Manifest::default(), steps)];
    let report = run(&f, &ir.plan(tasks));
    assert_eq!(labels(&report, 0), ["waits", "depends before-not-unique"]);
    assert!(!report.has_errors());
}

#[test]
fn preflight_stops_at_the_first_step_that_does_not_run() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let steps = vec![
        vec![ir.write("a.txt", "a\n", WriteMode::Create)],
        vec![ir.script("cargo build")],
        vec![ir.write("src/lib.rs", "x\n", WriteMode::Create)],
    ];
    let tasks = vec![ir.task("T1", Manifest::default(), steps)];
    let plan = ir.plan(tasks);
    let found = preflight(&open(&f), &plan, &plan.tasks[0]).unwrap_or_else(|e| panic!("{e}"));
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn manifest_entries_are_checked_at_the_start_of_their_task() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let m = ir.manifest(&[
        ("src/lib.rs", ChangeKind::Create, W),
        ("src/none.rs", ChangeKind::Modify, W),
        ("src/old.rs", ChangeKind::Test, W),
        ("nowhere.rs", ChangeKind::Modify, PathOrigin::Bare),
    ]);
    let tasks = vec![ir.task("T1", m, vec![vec![]])];
    let report = run(&f, &ir.plan(tasks));
    let codes: Vec<_> = report.tasks[0].manifest.iter().map(|f| f.code).collect();
    assert_eq!(codes, ["create-exists", "target-missing"]);
}
```

**Step 2 — verify**

Targeted: preflight and the dry run:

```powershell
cargo test -p planrun-exec --test preflight
```

Expected (exit code 0; summary lines):

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-exec/src/lib.rs crates/planrun-exec/src/preflight.rs crates/planrun-exec/tests/preflight.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/lib.rs' = '65def4e8d17483d13036e3a5f4ce1c6c60d3086f'
      'crates/planrun-exec/src/preflight.rs' = 'a09fd75f0a1a9132140a670bc85a838626ca0fed'
      'crates/planrun-exec/tests/preflight.rs' = '317d03a3c75b7a9a31d019daa129cf96fcf5da0a'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P3-T7: task preflight and the dry run: scope (D50), real dirt and steps that did not run (D55)" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P3-T8: planrun run --dry-run, its report and the P3 acceptance tests

**Goal:** The CLI. `planrun run --dry-run` parses, checks and runs the dry run in the current directory's repository, prints one line per action and a summary, then the findings with their source lines; exit 0, 1 or 2. The tests run the binary on a fixture copy (a snapshot of the passing run, one of a failing run, and the "writes nothing" test with `core.autocrlf=true`), call preflight and prepare on the parsed `assay-style.md` for the two roadmap cases, and check `path-ambiguous` on a devready plan.

**Files**

- Create `crates/planrun-cli/src/dry_run.rs`
- Modify `crates/planrun-cli/src/main.rs`
- Create `crates/planrun-cli/tests/dry_run.rs`
- Create `crates/planrun-cli/tests/snapshots/dry_run__dry_run_assay_style.snap`
- Create `crates/planrun-cli/tests/snapshots/dry_run__dry_run_create_exists.snap`

**Step 1 — write the files**

File `crates/planrun-cli/src/dry_run.rs` (full contents):

```rust
//! `planrun run --dry-run` (docs/architecture.md §7, decision D55): one line per action with its
//! outcome, a summary, then every finding with its source context.

use planrun_exec::finding::{Finding, Severity as ExecSeverity};
use planrun_exec::preflight::{DryRun, Outcome};
use planrun_model::{ActionKind, Plan};
use planrun_parse::{Diagnostic, Severity};

use crate::render;

/// Counts for the summary line.
#[derive(Default)]
struct Counts {
    ok: usize,
    ask: usize,
    failed: usize,
    depends: usize,
    not_run: usize,
    not_checked: usize,
    waits: usize,
}

/// The report of `run`, rendered into `out`.
pub fn report(plan: &Plan, run: &DryRun, out: &mut String) {
    let name = plan.source.name.to_string();
    out.push_str(&format!(
        "dry run of {name} ({}, {} tasks): nothing is written, no script runs\n",
        plan.format.0,
        plan.tasks.len()
    ));
    out.push_str(
        "after the first script, commit or Manual step, each result assumes it changed no files\n",
    );
    for path in &run.attribute_writes {
        out.push_str(&format!(
            "note: the plan writes `{path}`; the dry run reads attributes from the disk as they are\n"
        ));
    }
    let mut counts = Counts::default();
    for task in &run.tasks {
        let t = &plan.tasks[task.task];
        out.push_str(&format!("\n{}  {}\n", t.id, plan.text(&t.title)));
        if !task.manifest.is_empty() {
            out.push_str(&format!("  files: {} problem(s)\n", task.manifest.len()));
        }
        for a in &task.actions {
            if let Some(line) = action_line(plan, a, &mut counts) {
                out.push_str(&line);
            }
        }
    }
    out.push_str(&format!(
        "\nsummary: {} ok, {} would ask, {} failed, {} depend on a step that did not run; \
         {} scripts and commits not run, {} generated files not checked, {} manual\n",
        counts.ok,
        counts.ask,
        counts.failed,
        counts.depends,
        counts.not_run,
        counts.not_checked,
        counts.waits
    ));
    let findings: Vec<Diagnostic> = run.findings().map(diagnostic).collect();
    if !findings.is_empty() {
        out.push('\n');
        render::diagnostics(&name, plan.source.text(), &findings, out);
    }
}

/// One table line; `None` for a Note.
fn action_line(
    plan: &Plan,
    a: &planrun_exec::preflight::ActionReport,
    counts: &mut Counts,
) -> Option<String> {
    let (what, path) = describe(&a.action.kind);
    let outcome = match &a.outcome {
        Outcome::Shown => return None,
        Outcome::Ok(detail) => {
            counts.ok += 1;
            format!("{detail}: ok")
        }
        Outcome::WouldAsk(detail) => {
            counts.ask += 1;
            format!("{detail}: would ask (not in the task's files, D50)")
        }
        Outcome::Failed(f) => {
            counts.failed += 1;
            format!("FAILED {}", f.code)
        }
        Outcome::Depends(f) => {
            counts.depends += 1;
            format!("unverified {} (depends on a step that did not run)", f.code)
        }
        Outcome::NotRun => {
            counts.not_run += 1;
            "not run".to_owned()
        }
        Outcome::NotChecked => {
            counts.not_checked += 1;
            "not checked (its script did not run)".to_owned()
        }
        Outcome::Waits => {
            counts.waits += 1;
            "waits for you".to_owned()
        }
    };
    let line = plan.source.line_col(a.action.span.start).line;
    Some(format!(
        "  L{line:<5} {what:<9} {path:<24} {outcome}\n",
        path = path.unwrap_or_default()
    ))
}

fn describe(kind: &ActionKind) -> (&'static str, Option<String>) {
    match kind {
        ActionKind::WriteFile { path, .. } => ("write", Some(path.to_string())),
        ActionKind::ApplyPatch { path, .. } => ("patch", Some(path.to_string())),
        ActionKind::ReplaceExact { path, .. } => ("replace", Some(path.to_string())),
        ActionKind::Delete { path, .. } => ("delete", Some(path.to_string())),
        ActionKind::Generated { path } => ("generated", Some(path.to_string())),
        ActionKind::RunScript(_) => ("script", None),
        ActionKind::Commit(_) => ("commit", None),
        ActionKind::Manual { .. } => ("manual", None),
        ActionKind::Note { .. } => ("note", None),
    }
}

/// A finding as a diagnostic, so it renders like the parser's.
fn diagnostic(f: &Finding) -> Diagnostic {
    Diagnostic {
        severity: match f.severity {
            ExecSeverity::Error => Severity::Error,
            ExecSeverity::Warning => Severity::Warning,
        },
        code: f.code,
        span: f.span,
        message: f.message.clone(),
        related: Vec::new(),
    }
}
```

File `crates/planrun-cli/src/main.rs` (full contents):

```rust
//! The `planrun` binary (docs/architecture.md §7).

mod dry_run;
mod render;
mod view;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use planrun_exec::preflight;
use planrun_exec::workspace::Workspace;
use planrun_parse::{Diagnostics, Parsed, Registry, parse_plan, source};

use crate::view::{Detail, Viewer};

/// Executes structured implementation plans against a git repository, task by task.
#[derive(Debug, Parser)]
#[command(name = "planrun", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse and check a plan; print diagnostics and the coverage report. Writes nothing.
    Check {
        /// The plan file.
        plan: PathBuf,
        /// Force a format id instead of detecting one.
        #[arg(long)]
        format: Option<String>,
    },
    /// Print the typed plan, or one task (`P1-T2`) or step (`P1-T2/1`, `P1-T2/intro`).
    Show {
        /// The plan file.
        plan: PathBuf,
        /// A task or task/step.
        target: Option<String>,
        /// Force a format id.
        #[arg(long)]
        format: Option<String>,
    },
    /// Execute a plan in the git repository of the current directory. Only `--dry-run` exists
    /// so far (Phase 3); the run itself is Phase 5.
    Run {
        /// The plan file.
        plan: PathBuf,
        /// Parse, check and validate against a virtual tree; change nothing, run nothing.
        #[arg(long)]
        dry_run: bool,
        /// Force a format id.
        #[arg(long)]
        format: Option<String>,
    },
}

/// Exit codes: 0 fine, 1 the plan has errors, 2 a usage or I/O problem.
const OK: u8 = 0;
const PLAN_ERRORS: u8 = 1;
const USAGE: u8 = 2;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut out = String::new();
    let code = match &cli.command {
        Command::Check { plan, format } => check(plan, format.as_deref(), &mut out),
        Command::Show {
            plan,
            target,
            format,
        } => show(plan, target.as_deref(), format.as_deref(), &mut out),
        Command::Run {
            plan,
            dry_run: true,
            format,
        } => run_dry(plan, format.as_deref(), &mut out),
        Command::Run { .. } => {
            out.push_str(
                "planrun: run without --dry-run is not implemented yet (docs/roadmap.md, P5)\n",
            );
            USAGE
        }
    };
    if code == OK {
        print!("{out}");
    } else {
        eprint!("{out}");
    }
    ExitCode::from(code)
}

/// A plan read and parsed, or the exit code when it was not.
enum Loaded {
    Parsed(Box<Parsed>),
    Failed(u8),
}

fn load(path: &Path, format: Option<&str>, out: &mut String) -> Loaded {
    let name = path.display().to_string();
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            out.push_str(&format!("planrun: {name}: {e}\n"));
            return Loaded::Failed(USAGE);
        }
    };
    let registry = match Registry::with_builtins() {
        Ok(registry) => registry,
        Err(e) => {
            out.push_str(&format!("planrun: the built-in formats: {e}\n"));
            return Loaded::Failed(USAGE);
        }
    };
    match parse_plan(&registry, &name, &bytes, format) {
        Ok(parsed) => Loaded::Parsed(Box::new(parsed)),
        Err(diagnostics) => {
            failed(&name, &bytes, &diagnostics, out);
            Loaded::Failed(PLAN_ERRORS)
        }
    }
}

/// Renders the diagnostics of a plan that did not parse, against its normalized text.
fn failed(name: &str, bytes: &[u8], diagnostics: &Diagnostics, out: &mut String) {
    let text = source::load(name, bytes)
        .map(|s| s.text().to_owned())
        .unwrap_or_default();
    render::diagnostics(name, &text, &diagnostics.0, out);
    let errors = diagnostics.error_codes().len();
    let plural = if errors == 1 { "" } else { "s" };
    out.push_str(&format!("planrun: {name}: {errors} error{plural}\n"));
}

fn check(path: &Path, format: Option<&str>, out: &mut String) -> u8 {
    let parsed = match load(path, format, out) {
        Loaded::Parsed(parsed) => parsed,
        Loaded::Failed(code) => return code,
    };
    let plan = &parsed.plan;
    let name = plan.source.name.to_string();
    render::diagnostics(&name, plan.source.text(), &parsed.diagnostics.0, out);
    render::coverage(plan, &parsed.coverage, out);
    out.push_str(&format!(
        "planrun: {name}: ok ({}, {} tasks)\n",
        plan.format.0,
        plan.tasks.len()
    ));
    OK
}

fn show(path: &Path, target: Option<&str>, format: Option<&str>, out: &mut String) -> u8 {
    let parsed = match load(path, format, out) {
        Loaded::Parsed(parsed) => parsed,
        Loaded::Failed(code) => return code,
    };
    let viewer = Viewer::new(&parsed.plan);
    let Some(target) = target else {
        view::plan_text(&viewer.plan(), out);
        return OK;
    };
    let (task_id, step) = match target.split_once('/') {
        Some((task, step)) => (task, Some(step)),
        None => (target, None),
    };
    let Some(task) = parsed.plan.tasks.iter().find(|t| &*t.id.raw == task_id) else {
        out.push_str(&format!("planrun: the plan has no task `{task_id}`\n"));
        return USAGE;
    };
    let Some(step) = step else {
        view::task_text(&viewer.task(task), Detail::Summary, out);
        return OK;
    };
    let found = task.steps.iter().find(|s| match s.number {
        Some(n) => n.to_string() == step,
        None => step == "intro",
    });
    let Some(found) = found else {
        out.push_str(&format!("planrun: task `{task_id}` has no step `{step}`\n"));
        return USAGE;
    };
    view::step_text(&viewer.step(found), Detail::Full, out);
    OK
}

/// `run --dry-run`: parse and check, then the dry run in the repository of the current
/// directory. Exit 0 when nothing would fail, 1 when something would, 2 when git or the
/// repository cannot be used.
fn run_dry(path: &Path, format: Option<&str>, out: &mut String) -> u8 {
    let parsed = match load(path, format, out) {
        Loaded::Parsed(parsed) => parsed,
        Loaded::Failed(code) => return code,
    };
    let plan = &parsed.plan;
    let name = plan.source.name.to_string();
    render::diagnostics(&name, plan.source.text(), &parsed.diagnostics.0, out);
    let here = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            out.push_str(&format!("planrun: the current directory: {e}\n"));
            return USAGE;
        }
    };
    let report = Workspace::open(&here, Vec::new()).and_then(|ws| preflight::dry_run(&ws, plan));
    let report = match report {
        Ok(report) => report,
        Err(e) => {
            out.push_str(&format!("planrun: {e}\n"));
            return USAGE;
        }
    };
    dry_run::report(plan, &report, out);
    if report.has_errors() {
        let errors = report.findings().filter(|f| f.is_error()).count();
        out.push_str(&format!(
            "planrun: {name}: the dry run found {errors} error(s)\n"
        ));
        return PLAN_ERRORS;
    }
    out.push_str(&format!("planrun: {name}: dry run ok\n"));
    OK
}
```

File `crates/planrun-cli/tests/dry_run.rs` (full contents):

````rust
//! `planrun run --dry-run` on a temporary copy of the greeter fixture, and the roadmap's P3
//! acceptance cases on parsed plans: the dry run of `assay-style.md` passes and writes nothing;
//! a changed `src/lib.rs` fails preflight with the blob mismatch; a Create onto an existing file
//! fails before anything is written. The CLI has only `--dry-run` in P3, so the last two call
//! planrun-exec's preflight and prepare directly.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;

use std::path::Path;
use std::process::Command;

use fixture::Fixture;
use planrun_exec::HandlerSet;
use planrun_exec::handler::file_path;
use planrun_exec::preflight::{Outcome, dry_run, preflight};
use planrun_exec::transaction::prepare;
use planrun_exec::workspace::Workspace;
use planrun_model::{Action, Plan};
use planrun_parse::{Registry, parse_plan};

const PLAN: &str = "assay-style.md";

fn plan_text() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/corpus/synthetic")
        .join(PLAN);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn parse(name: &str, text: &str, format: Option<&str>) -> Plan {
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, name, text.as_bytes(), format)
        .unwrap_or_else(|d| panic!("{:?}", d.error_codes()))
        .plan
}

fn open(f: &Fixture) -> Workspace {
    Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
}

/// Runs `planrun run --dry-run` in the copy on a copy of the plan next to it (outside the
/// repository), so the plan's name in the output is the same on every machine.
fn planrun_dry_run(f: &Fixture) -> (i32, String) {
    let plan = f.outside().join(PLAN);
    std::fs::write(&plan, plan_text()).unwrap_or_else(|e| panic!("{e}"));
    let output = Command::new(env!("CARGO_BIN_EXE_planrun"))
        .args(["run", "--dry-run", "../outside/assay-style.md"])
        .current_dir(f.path())
        .envs(f.env())
        .output()
        .unwrap_or_else(|e| panic!("{e}"));
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), text)
}

#[test]
fn the_dry_run_of_the_assay_plan_passes() {
    let f = Fixture::greeter(false);
    let (code, out) = planrun_dry_run(&f);
    assert_eq!(code, 0, "{out}");
    insta::assert_snapshot!("dry_run_assay_style", out);
}

#[test]
fn the_dry_run_writes_nothing_with_autocrlf() {
    let f = Fixture::greeter(true);
    f.outside();
    let before = f.snapshot();
    let (code, out) = planrun_dry_run(&f);
    assert_eq!(code, 0, "{out}");
    let after = f.snapshot();
    let changed: Vec<&String> = before
        .keys()
        .chain(after.keys())
        .filter(|k| before.get(*k) != after.get(*k))
        .collect();
    assert!(changed.is_empty(), "the dry run changed {changed:?}");
}

#[test]
fn a_dry_run_that_fails_renders_the_finding_and_exits_1() {
    let f = Fixture::greeter(false);
    f.write("src/greet.rs", b"//! already here\n");
    f.git(&["add", "src/greet.rs"]);
    f.git(&["commit", "-q", "-m", "greet exists"]);
    let (code, out) = planrun_dry_run(&f);
    assert_eq!(code, 1, "{out}");
    insta::assert_snapshot!("dry_run_create_exists", out);
}

#[test]
fn a_changed_lib_rs_fails_preflight_with_the_blob_mismatch() {
    let f = Fixture::greeter(false);
    f.write("src/lib.rs", b"//! changed\n");
    f.git(&["commit", "-q", "-am", "changed"]);
    let plan = parse(PLAN, &plan_text(), None);
    let found = preflight(&open(&f), &plan, &plan.tasks[0]).unwrap_or_else(|e| panic!("{e}"));
    let codes: Vec<_> = found.iter().map(|f| f.code).collect();
    assert_eq!(codes, ["blob-mismatch"]);
}

#[test]
fn a_create_onto_an_existing_file_fails_before_anything_is_written() {
    let f = Fixture::greeter(false);
    f.write("src/greet.rs", b"//! already here\n");
    let before = f.snapshot();
    let plan = parse(PLAN, &plan_text(), None);
    let step: Vec<&Action> = plan.tasks[0]
        .steps
        .iter()
        .map(|s| {
            s.actions
                .iter()
                .filter(|a| file_path(&a.kind).is_some())
                .collect::<Vec<_>>()
        })
        .find(|actions| !actions.is_empty())
        .unwrap_or_default();
    assert_eq!(
        step.len(),
        2,
        "step 1 writes src/greet.rs and patches src/lib.rs"
    );
    let ws = open(&f);
    let err = prepare(&ws, &plan.source, &HandlerSet::files(), &step).err();
    assert_eq!(err.map(|f| f.code), Some("create-exists"));
    assert_eq!(f.snapshot(), before, "the tree or .git changed");
}

const DEVREADY: &str = "### Task 1: tools

**Files:** modify `tools/a.ps1` and `b.ps1`

**1a.** `b.ps1` (replaces the whole file):

```powershell
'b'
```
";

#[test]
fn an_overwrite_of_a_bare_entry_is_ambiguous_in_the_devready_format() {
    let f = Fixture::greeter(false);
    f.write("b.ps1", b"'old'\n");
    f.git(&["add", "b.ps1"]);
    f.git(&["commit", "-q", "-m", "a root b.ps1"]);
    let plan = parse("devready.md", DEVREADY, Some("devready"));
    let report = dry_run(&open(&f), &plan).unwrap_or_else(|e| panic!("{e}"));
    let outcomes: Vec<_> = report.tasks[0]
        .actions
        .iter()
        .filter_map(|a| match &a.outcome {
            Outcome::Failed(f) => Some(f.code),
            Outcome::Shown => None,
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(outcomes, ["path-ambiguous"]);
}

#[test]
fn run_without_dry_run_is_not_there_yet() {
    let f = Fixture::greeter(false);
    let output = Command::new(env!("CARGO_BIN_EXE_planrun"))
        .args(["run", "../outside/none.md"])
        .current_dir(f.path())
        .envs(f.env())
        .output()
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("P5"));
}
````

File `crates/planrun-cli/tests/snapshots/dry_run__dry_run_assay_style.snap` (full contents):

```text
---
source: crates/planrun-cli/tests/dry_run.rs
expression: out
---
dry run of ../outside/assay-style.md (assay, 2 tasks): nothing is written, no script runs
after the first script, commit or Manual step, each result assumes it changed no files

P1-T1  the greet module
  L35    write     src/greet.rs             create: ok
  L54    patch     src/lib.rs               aa80cf1 → 97bd92a: ok
  L83    script                             not run
  L95    script                             not run
  L110   commit                             not run

P1-T2  re-export greeting, remove the legacy module
  L130   generated docs/modules.txt         not checked (its script did not run)
  L134   patch     src/lib.rs               97bd92a → 6b269fc: ok
  L159   delete    src/old.rs               delete: ok
  L161   generated docs/modules.txt         not checked (its script did not run)
  L167   script                             not run
  L181   script                             not run
  L196   commit                             not run

summary: 4 ok, 0 would ask, 0 failed, 0 depend on a step that did not run; 6 scripts and commits not run, 2 generated files not checked, 0 manual
planrun: ../outside/assay-style.md: dry run ok
```

File `crates/planrun-cli/tests/snapshots/dry_run__dry_run_create_exists.snap` (full contents):

```text
---
source: crates/planrun-cli/tests/dry_run.rs
expression: out
---
dry run of ../outside/assay-style.md (assay, 2 tasks): nothing is written, no script runs
after the first script, commit or Manual step, each result assumes it changed no files

P1-T1  the greet module
  files: 1 problem(s)
  L35    write     src/greet.rs             FAILED create-exists
  L54    patch     src/lib.rs               aa80cf1 → 97bd92a: ok
  L83    script                             not run
  L95    script                             not run
  L110   commit                             not run

P1-T2  re-export greeting, remove the legacy module
  L130   generated docs/modules.txt         not checked (its script did not run)
  L134   patch     src/lib.rs               97bd92a → 6b269fc: ok
  L159   delete    src/old.rs               delete: ok
  L161   generated docs/modules.txt         not checked (its script did not run)
  L167   script                             not run
  L181   script                             not run
  L196   commit                             not run

summary: 3 ok, 0 would ask, 1 failed, 0 depend on a step that did not run; 6 scripts and commits not run, 2 generated files not checked, 0 manual

error[create-exists]: `src/greet.rs` already exists; the task's files say it is created
  --> ../outside/assay-style.md:30:1
   |
30 | - Create `src/greet.rs`
   | ^^^^^^^^^^^^^^^^^^^^^^^

error[create-exists]: `src/greet.rs` already exists; the plan creates it
  --> ../outside/assay-style.md:35:1
   |
35 | / File `src/greet.rs` (full contents):
36 | |
37 | | ```rust
38 | | //! Greetings (P1-T1).
...  |
52 | | ```
   | |___^

planrun: ../outside/assay-style.md: the dry run found 2 error(s)
```

**Step 2 — verify**

Targeted: the dry run of the CLI:

```powershell
cargo test -p planrun-cli --test dry_run
```

Expected (exit code 0; summary lines):

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-cli/src/dry_run.rs crates/planrun-cli/src/main.rs crates/planrun-cli/tests/dry_run.rs crates/planrun-cli/tests/snapshots/dry_run__dry_run_assay_style.snap crates/planrun-cli/tests/snapshots/dry_run__dry_run_create_exists.snap
  $verified = [ordered]@{
      'crates/planrun-cli/src/dry_run.rs' = '34ad072d3f3be4b2123a88de042a9e9ae774150d'
      'crates/planrun-cli/src/main.rs' = '654edc5cc2949b16eb6deabcff49e68453eda670'
      'crates/planrun-cli/tests/dry_run.rs' = 'fb82127d2ce2fd27e808d379bbea859f564315d6'
      'crates/planrun-cli/tests/snapshots/dry_run__dry_run_assay_style.snap' = '08bae4f015cab67e435c801156347c17cd9674cf'
      'crates/planrun-cli/tests/snapshots/dry_run__dry_run_create_exists.snap' = '1ce46ff470c555bef87fb7e16ddd74701d4d5e81'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P3-T8: planrun run --dry-run, its report and the P3 acceptance tests" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P3-T9: architecture, extending, roadmap and CLAUDE.md for the workspace and the dry run

**Goal:** The docs that describe what P3 built: architecture.md §3 (the IR as it is now), §6.1 (preflight), §6.2 (the virtual tree, transactions, the writer, line endings) and §7 (the dry run); extending.md §4 and §7 (`simulate`); the roadmap's P3 paragraph; CLAUDE.md's layout and test conventions.

**Files**

- Modify `CLAUDE.md`
- Modify `docs/architecture.md`
- Modify `docs/extending.md`
- Modify `docs/roadmap.md`

Over 300 lines, so they come as exact diffs: `docs/architecture.md`.

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
crates/planrun-exec    IR → effects: git, workspace, eol, tree, handler, transaction,
                       preflight, finding (P3); shells, expect, policy, journal, runner
                       (P4–P5)
crates/planrun-cli     the `planrun` binary: check, show (view, render), run --dry-run
formats/*.toml         built-in plan formats (embedded): assay, superpowers, before-after; the
                       file name is the format id
docs/                  architecture, plan-grammar, extending, decisions, roadmap, plans/,
                       handovers/ (one per finished phase)
tests/corpus/          synthetic/ (one plan per format), invalid/ (one error each), local/ (private)
tests/fixtures/greeter the repo the synthetic assay plan applies to
tests/support          fixture.rs: a temporary git copy of the greeter fixture, included
                       with #[path] by the exec and cli tests
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
  never on this repository: `tests/support/fixture.rs` makes one under the OS temp directory,
  with an environment that keeps the user's git configuration out. Exec tests build the IR by
  hand (`crates/planrun-exec/tests/support/ir.rs`, D2); tests that need a parsed plan live in
  planrun-cli.

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

File `docs/architecture.md` (exact diff; save the block as `$env:TEMP\p3-t9-architecture-md.patch`, outside the repository, then apply it):

````diff
diff --git a/docs/architecture.md b/docs/architecture.md
index b68869b..dc09ca3 100644
--- a/docs/architecture.md
+++ b/docs/architecture.md
@@ -64,8 +64,11 @@ composition roots. Nothing is a deep type hierarchy.
 ```
 Plan { source: PlanSource, format: FormatId, meta: PlanMeta, tasks: Vec<Task> }
 Task { id: TaskId { raw, ordinal }, title, span, manifest: Manifest, steps: Vec<Step> }
+Manifest { entries: Vec<ManifestEntry>, span }
+ManifestEntry { path: ManifestPath (File(RepoPath) | Pattern(PathPattern)), change, lines, span, origin: PathOrigin }
 Step { number: Option<u32>, title: Option<TextRef>, span, actions: Vec<Action> }
-Action { kind: ActionKind, span, origin: RecognizerId }
+Action { kind: ActionKind, span, origin: RecognizerId, path_origin: PathOrigin }
+PathOrigin = Written | Sibling | FileName | OnlyEntry | Bare | PreviousEdit
 
 ActionKind =
   | WriteFile    { path, body, mode: Create | Overwrite | FromManifest }
@@ -79,7 +82,7 @@ ActionKind =
   | Manual       { reason, text }
 
 Script { shell: Pwsh | Bash, body, expect: Option<Expectation>, intent: Verify|Gate|Preview|Apply|Other }
-Expectation { exit: Success | Failure | Code(n) | Unknown, lines: Option<TextRef>, prose: Option<TextRef> }
+Expectation { exit: Success | Failure | Code(n) | Unknown, lines, prose, comments: Option<TextRef> }
 ```
 
 - **Text is never copied.** `TextRef::Span` points into the normalized plan text owned by
@@ -100,8 +103,10 @@ Expectation { exit: Success | Failure | Code(n) | Unknown, lines: Option<TextRef
   commit. A path a task names as generated more than once (an intro notice and a step notice)
   is one check.
 - **`RepoPath`** (D7) is the only path type in actions: relative, forward slashes, no `..`, no
-  `.`, no empty segments, no drive letters. Symlink escapes are refused by the workspace at write
-  time.
+  `.`, no empty segments, no drive letters, no wildcards (D42), no spelling Windows reserves or
+  git refuses (`:`, `.git` in any form, device names, D51). Symlink escapes are refused by the
+  workspace at write time (D52). `Manifest::scopes` and `Manifest::is_bare_target` decide D50's
+  scope for parse and exec alike.
 
 ## 4. Parsing (planrun-parse)
 
@@ -222,9 +227,12 @@ A state machine over Task → Step → Action. Per action: `Pending → Previewe
 Done | Failed | Skipped`. Per task, the lifecycle of your §6.1 proposal:
 
 1. Show the task, its manifest and its actions (with `Note`/`Manual` in place).
-2. **Preflight**: manifest Create targets absent, Modify targets present (pattern entries are
-   skipped: they name no single file, D41), the first patch's preimage blob matches the file on
-   disk, `git status --porcelain` empty when the plan says so.
+2. **Preflight** (`preflight::preflight`): no touched path has uncommitted changes
+   (`path-dirty`; other dirt is a warning, D55); manifest Create entries absent, Modify, Rewrite
+   and Delete entries present (patterns and Bare entries are skipped: they name no single known
+   file, D41, D45); then the task's file actions up to its first script or Manual, simulated on
+   a fresh virtual tree: every preimage blob, every patch, every `before`. A Bare-targeted
+   Overwrite or Delete is `path-ambiguous`; an unscoped file action is confirmed (D50).
 3. Execute steps in order; each step's file actions as one transaction (§6.2), scripts in sessions
    (§6.3), stopping per policy (§6.5).
 4. Commit only through the plan's own commit script, and only when the task's gate/verify passed
@@ -233,23 +241,42 @@ Done | Failed | Skipped`. Per task, the lifecycle of your §6.1 proposal:
 
 ### 6.2 Workspace and files
 
-- **Step transactions** (D23): all file actions of a step, up to the next non-file action, are
-  validated against an in-memory copy first (create targets absent, overwrite targets present,
-  every patch applies with `git apply --check`, every `before` occurs exactly once). Only then are
-  they written. One failure writes nothing.
+- **The virtual tree** (D52): a temporary index and object directory under the OS temp
+  directory, with the repository's objects as alternates, passed to git per call
+  (`GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`). Paths are seeded
+  from the working tree on first use. Each `ActionHandler` simulates its action on it: create
+  targets absent, overwrite targets present, every patch with `git apply --cached` after
+  `--check`, every `before` exactly once. With `core.ignorecase`, two spellings of one path are
+  `path-collision`.
+- **Step transactions** (D23, D53): `transaction::prepare` simulates the file actions of a step,
+  up to the next non-file action, on a virtual tree seeded from the disk, stores the pre-images,
+  and builds every post-image there: a patched file's bytes are `git cat-file --filters`, the
+  conversion `git apply` would make; the others are D54 bytes. `transaction::commit` hands the
+  `PreparedStep` to a write-ahead recorder (P5: the journal), then writes it. One failure in
+  prepare writes nothing; one in commit is rolled back from the pre-images, and what could not
+  be restored is reported. A `Delete` with `via_git` updates the index last, in one call.
 - **Pre-images** of every file about to change are stored in git's object database
-  (`git hash-object -w`) and their ids journaled: rollback is `git cat-file`, untracked files
-  included, no extra storage.
-- **Writes are atomic**: write a temp file in the same directory, then rename.
-- **Line endings** (D10): created files are written as in the plan (LF). Overwrites and
-  `ReplaceExact` keep the target's existing convention unless `.gitattributes` says `eol=lf`;
-  `before` is matched in LF space. Patches go through `git apply` unchanged.
+  (`git hash-object -w --no-filters`: the exact bytes) and their ids journaled: rollback is
+  `git cat-file`, untracked files included, no extra storage. They are unreachable objects;
+  pruning is a P5 decision (D57).
+- **One writer** (D52): the path is resolved first (no link, junction, short name or case
+  mismatch in its existing part); directories are created one segment at a time, each checked;
+  the contents go to `.planrun-<random>.tmp` in the target's directory (`create_new`); the parent
+  is checked again; the rename replaces the target. A parent swapped between that check and the
+  rename is not caught.
+- **Line endings** (D10, D54): created files are written as in the plan (LF). For Overwrites and
+  `ReplaceExact`, an `eol` attribute decides, `-text` keeps bytes; otherwise the file's own
+  convention; a file without line endings takes git's checkout convention; a mixed file is
+  overwritten by the checkout convention and refused for an exact edit (`eol-mixed`). `before` is
+  matched in LF space. Patches go through git.
 - **Patches** go through `git apply --whitespace=nowarn -` with the patch on stdin (D20): git's
   exact semantics, no temp file, no fuzz. The `index` line's blob ids are checked before (preimage)
-  and after (postimage). Blob ids are computed with `git hash-object --path=<p>` so clean filters
-  and `autocrlf` match git's view; an in-process SHA-1 is only a fast path when no filter applies.
+  and after (postimage). Blob ids are computed by `git hash-object --path=<p>`, so clean filters
+  and `autocrlf` match git's view; planrun-exec computes none in-process (D56).
 - **The working tree stays clean of planrun**: nothing is written there except what the plan
-  writes. State and scratch go under `.git/planrun/` and the OS temp directory.
+  writes, and a writer's temp file until its rename. State and scratch go under `.git/planrun/`,
+  pre-images into the object database, the virtual tree into the OS temp directory. Status calls
+  run with `--no-optional-locks`, so they do not refresh `.git/index`.
 - The absorbed **patch ceremony** (`save the block as $env:TEMP\…`, `git apply`, `Remove-Item`)
   is never run (D17).
 
@@ -313,9 +340,13 @@ planrun resume <plan>
 planrun status <plan>
 ```
 
-`--dry-run`: parse, check, preflight against the real repository, then run every file action
-against a **virtual tree** in memory (patches with `git apply --check` in sequence). Runs no script,
-writes nothing.
+`--dry-run` (D55): parse, check, check real dirt once for the paths of every task, then preflight
+every task against **one virtual tree** carried across the tasks (D52), so a task sees what the
+tasks before it wrote, and simulate every file action. Commits are assumed. Runs no script and
+writes nothing: not the working tree, not `.git`. Once a script, commit or Manual has gone by
+without running, a failing file action is a warning ("depends on a step that did not run") and
+its path is unknown from then on; every ok after that point assumes those steps change no files.
+Exit 0 when nothing fails, 1 when something does, 2 when git or the repository cannot be used.
 
 The interactive screen (line prompts first, D15):
 
````

```powershell
git apply --whitespace=nowarn "$env:TEMP\p3-t9-architecture-md.patch"
Remove-Item "$env:TEMP\p3-t9-architecture-md.patch"
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
| `before-after` | lead (naming a path), fence, *separator paragraph*, fence | `ReplaceExact`, or `Manual(ProseEdit)` when `infer_path` finds no single entry | `lead` (captures `path`, or none with `infer_path = true`), *`separator`* (e.g. `^After:$`; absent = two fences back to back), *`file_name_from_manifest`*, *`series`* (a regex capturing `n`; the lead then captures `n` too, D49); a pathless lead may capture *`stem`* (D59) |
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
  before it in the step (`PathOrigin::PreviousEdit`), before `infer_path` is tried. A
  pathless lead's `stem` capture (`README edit 2`) must equal the file-name stem of the path
  it continues or infers, or the edit is `Manual(ProseEdit)` (D59).
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
2. Its handler in planrun-exec: `simulate` on the virtual tree (its dry-run behaviour and
   its part of a step transaction, D53), and its `DoneState` logic for resume (P5).
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
| `ActionHandler` | `HandlerSet` | how an action kind is simulated on the virtual tree (D52, D53); the transaction writes the result. `DoneState` follows in P5 |
| `ShellSession` | one per `Shell` | another shell (`cmd`, `zsh`): a new `Shell` variant + session |
| `CommandClassifier` | the runner's | different policy classification |
| `Ui` | the CLI | a TUI (ratatui) later, a non-interactive batch UI |
````

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

## P2 — The other three formats (done)

The decisions are D26–D40. Split in two plans on one schema (D40):

- **P2a (done).** The P1 outline fixes and `task-after-end` (D37); the spec schema (`required`,
  a marker that captures `change`, `siblings`, `ChangeKind::Generated`, `[checks]`,
  `[[step_marks]]`, `script` `leads`/`first_block`/intent `lead`); the kinds `run-line`,
  `before-after`, `manual-prose`; path resolution from the manifest (D32);
  `formats/superpowers.toml` and `before-after.toml` with IR snapshots of their synthetic plans.
  The superpowers and before-after local plans parse with zero errors. Plan:
  `docs/plans/2026-11-24-p2a-hand-written-formats.md`.
- **P2b (done).** First, pattern manifest entries: a range or glob item is one entry whose
  target is a `PathPattern` (D41, D42). Then the schema for hand-written DevReady plans: inline
  Files clauses (D43), a change per path (D44), Bare file names (D45), kept step labels (D46),
  `Expected:` with a list (D47), comment expectations that a sentence extends (D35, D48),
  numbered edit series (D49). Then `formats/devready.toml` with the `**Commit:**` Manual (D33),
  detection that outscores superpowers on both devready plans, the IR snapshot of its synthetic
  plan, and the Apply invariant in plan-grammar §5. All six local plans parse with zero errors.
  Plan: `docs/plans/2026-11-25-p2b-devready-format.md`.

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

The decisions are D50–D59. Plan: `docs/plans/2026-11-25-p3-workspace-dry-run.md`. First the P2b
review items (series stems D59, `run-from`, negated failures, `write-mode-unknown` D58) and the
`RepoPath` hardening (D51); then the git wrapper, the workspace and its one writer, line endings
(D54), the virtual tree (D52), step transactions as prepare and commit (D53), preflight with
scope (D50) and the dry run (D55). The per-task file cache is the virtual tree: a task's paths are
read once, on first use.

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

Targeted: the synthetic assay plan:

```powershell
cargo run -q -p planrun-cli -- check tests/corpus/synthetic/assay-style.md
```

Expected (exit code 0; summary lines):

```text
planrun: tests/corpus/synthetic/assay-style.md: ok (assay, 2 tasks)
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- CLAUDE.md docs/architecture.md docs/extending.md docs/roadmap.md
  $verified = [ordered]@{
      'CLAUDE.md' = 'e72de052f553a1f5e2d7e0d025bc07ec9aed12c6'
      'docs/architecture.md' = 'dc09ca3634c46e8798fca419d576c9bd0c5a5c3e'
      'docs/extending.md' = '46fc4f386a6b62bced84cdb50ff586f9cb1e7dfd'
      'docs/roadmap.md' = 'ea6798f1bbe63695278897480586df1b7116e6d6'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P3-T9: architecture, extending, roadmap and CLAUDE.md for the workspace and the dry run" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

## P3 green state

Run from the repository root after P3-T9:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-parse --test local_corpus -- --ignored local_corpus }
```

Expected: every `test result:` line ok; the last one `1 passed`, with `6 parsed, 0 skipped` in the local-corpus output.

User-visible change: `planrun run --dry-run <plan>` in a repository validates every file action of the plan against a virtual tree and prints what would run, ask, fail or wait; `planrun check` reports `write-mode-unknown` and the new `bad-path` spellings.

## After execution

- `git log --oneline -10` lists the commit of this plan, then `P3-T1` to `P3-T9`.
- `cargo run -q -p planrun-cli -- check docs/plans/2026-11-25-p3-workspace-dry-run.md` exits 0: this plan is an assay plan.
- Write the handover `docs/handovers/<date>-p3-to-p4.md` in a separate session or commit after review. Items for it from the P3 review: the write-ahead record P5 must persist before `commit` (D53); `git gc` pruning of pre-images (D57); dirt from planrun's own writes after a task whose commit is Manual (D33); a commit script that stages everything on a tree with unrelated dirt; `.planrun-*.tmp` debris after a crash; D50's prompt; the one-check rule for repeated `Generated` notices; handover P2→P3 items 5 (the Apply invariant to plan-grammar §1) and 6 (a script working directory, P4).
- Also for that handover, from the plan review. Each is required before P5 runs `commit` outside the tests; none is reachable in P3:
  1. **File mode.** `Workspace::write` creates its temp file with default permissions, so an Overwrite of a 100755 file drops the exec bit on Linux (the rename replaces the inode). A patch with `old mode`/`new mode` or `new file mode 100755` gives `PlannedWrite` bytes but no mode. Needed: the post-image's mode, from the virtual index or the target's existing permissions, applied to the temp file before the rename, under `core.filemode`; and a D-entry. Windows with `core.filemode=false` is unaffected.
  2. **The gap between prepare and commit.** `commit` does not check that each path on disk still matches its pre-image, and P5 puts D50's prompt between the two. Needed: right before the write-ahead record, `commit` re-hashes each touched path (`hash-object --no-filters` without `-w`, or a byte comparison); on any difference, or a path that appeared or vanished, it aborts with nothing written, and the step is prepared again.
  3. **A `via_git` Delete in a real preflight.** Stated decision 16 holds for the dry run. In `preflight()` on the real disk, a `via_git` target that is not tracked is a plan error (a manual `git rm` would fail), not a silent delete of an untracked file.
  4. **Report wording.** In a failing report, `files: N problem(s)` next to the trailer's `M error(s)` reads as inconsistent; align the wording when P5 reworks the report.
