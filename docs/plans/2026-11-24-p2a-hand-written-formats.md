# planrun P2a — Hand-written formats: schema, kinds, superpowers and before-after

**Goal:** the spec schema and recognizer kinds that all three hand-written formats need, and the `superpowers` and `before-after` formats on them (docs/roadmap.md P2a, decisions D26–D40). The superpowers and before-after real plans parse with zero errors; the devready format follows in P2b on the same schema.

- **P2a-T1** (`planrun-parse`, docs, `tests/corpus/invalid`): the two P1 outline fixes and `task-after-end` (D37), before any new format; D26–D40; the roadmap's P2a/P2b split.
- **P2a-T2** (`planrun-model`, `planrun-parse`, `planrun-cli`, docs): manifest `required`, a marker that captures `change`, `siblings`, `:lines` tokens, `ChangeKind::Generated`, `PathOrigin`, the op-manifest check (D26), `[checks]` (D31).
- **P2a-T3** (`planrun-parse`, docs): the `script` guard and intents by lead (D28, D29), step marks (D30), an expectation that joins a Manual (D38).
- **P2a-T4** (`planrun-model`, `planrun-parse`, `planrun-cli`, docs): the kinds `run-line`, `before-after`, `manual-prose`; paths resolved from the manifest (D32).
- **P2a-T5** (`formats`, `planrun-parse`, `planrun-cli`, docs, corpus): the superpowers format.
- **P2a-T6** (`formats`, `planrun-parse`, `planrun-cli`, docs, corpus): the before-after format.
- **P2a-T7** (docs, `CLAUDE.md`, `planrun-parse`): architecture §4, the CLAUDE.md Layout, the local-corpus report.

**Base:** `3910b9b` (`docs: P1 done, handover P1 to P2`); execute from the commit of this plan. Gate per task: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Shell and environment:** PowerShell 7 on Windows; Rust 1.95.0 (stable). No new crate, and `Cargo.lock` does not change.

**Before P2a-T1:** this plan committed, and a clean `git status --porcelain`. The real plans stay in `tests/corpus/local/` (git-ignored, D16); P2a-T7 reads them, and the plan quotes none of them.

**How this plan was verified:** every task was built in a scratch clone of `3910b9b` outside this repository, one commit per task. At each commit `cargo fmt --check`, clippy and `cargo test` pass, and every count below comes from those runs. Full contents are exported from those commits; files over 300 lines come as exact diffs made by `git diff`, so their `index` lines are real blob ids. Each commit step compares the blob id of every staged file with the verified one before it commits. The P2a-T7 build's `planrun check` passes this plan, and the plan was replayed on a fresh clone with `core.autocrlf=true`.

## Review rulings (P2 questions, 2026-11-24)

Each accepted answer is a decision in docs/decisions.md (P2a-T1); this list maps them.

1. **Contradictions:** `Expected:` is in the same paragraph as `Run:` and the manifest is optional (docs fixed in P2a-T5 and P2a-T2); `expected-failure-step` and `preview-apply` are replaced (D30, D29); both latent P1 bugs are fixed in P2a-T1 (D37); the plan-grammar sentence "Create only when the manifest says Create" is dropped, the op decides and a disagreeing manifest entry is `manifest-write-mode` in every format (D26).
2. **task_heads:** a heading that matches the task rule is a task; only a non-task heading at or above `tasks_end_level` ends the tasks; a task heading after that is `task-after-end` (D37).
3. **Defaults** accepted; `run-line` names its shell per spec (D39); an `Expected:` after a Manual joins it (D38). No real plan has an `Expected:` after a Note inside a human-check step, so that stays `expected-without-script`.
4. **C1** step marks (D30): expected-failure applies to verify-intent scripts only; `any` becomes Failure; success is `expected-conflict`.
5. **C2** the DevReady `**Commit:**` line is `Manual(HumanCheck)` with the message; the P5 suggestion only for literal manifest paths (D33). Built in P2b.
6. **C3** the script guard (D28): `leads` and a per-format `first_block`, never for the implicit first step; a body intent never qualifies a fence on its own.
7. **C4** `[checks]` downgrades only `manifest-untouched` and `manifest-undeclared` (D31).
8. **C5** path resolution (D32): `siblings` opt-in, `:lines` everywhere, `file_name_from_manifest` only to one literal entry, `infer_path` only for edits; `show` marks resolved paths.
9. **C6** `ChangeKind::Generated` (D27); `required` defaults to true; `keep = true` and inline Files clauses are P2b.
10. **C7** no `preview-apply` kind; `leads` and per-intent `lead` are separate `script` parameters (D29).
11. **C8** inline edits are `Manual(ProseEdit)` for now (D34). **C9** `line_expect`, devready only (D35), P2b.
12. **C10–C11** markers and human-check marks; the imperative Notes that remain are listed below (D36); DevReady targets zero errors and honest Manual coverage (D40).
13. **C12** P2a / P2b on one schema (D40). **C13** the CLAUDE.md Layout is updated in P2a-T7.

## Stated decisions made while building

1. **Coverage counts a recognizer's Manual as manual** (P2a-T4), like the fallback's, and an expectation joined to a Manual counts as manual too. Before, a `Yes` from any recognizer counted as actions.
2. **A human-check step converts only its prose** (P2a-T6): the fallback knows the block kind, so paragraphs, list items and quotes become `Manual(HumanCheck)` and a `---` rule or a table stays a Note. D30 says so. The expected-failure mark still runs after recognition, on the step's verify scripts.
3. **`file_name_from_manifest` is a parameter that no P2a spec sets.** Neither real P2a plan needs it, and in the assay format a root-level lead such as `Cargo.toml` must stay as written. P2b's devready spec turns it on.
4. **A `:lines` token is a second entry for the same path** (`Program.cs:84-85` and `:241-243` → two Modify entries with their line ranges). The manifest checks look entries up by path, so nothing counts twice.
5. **Range items stay as written** (review ruling 8): `` `Steps/0001`–`0007_*.sql` `` gives the entries `Steps/0001` (written) and `Steps/0007_*.sql` (a sibling). The before-after real plan's edit of one of those files is the one `manifest-undeclared` warning it has. P3's preflight must not treat these entries as files that exist.
6. **Generated entries need no action** (D27): `manifest-untouched` skips them like Test entries.
7. **The task rule keeps its level.** "A task at any level" in the review answer is read as: the task test comes before the end test, so a level-2 task under `tasks_end_level = 2` is a task (before-after). A task heading at another level than the rule's is no task.
8. **New codes:** `task-after-end` (P2a-T1) and `expected-conflict` (P2a-T5), each with an invalid-corpus file; spec load errors for `[checks]` keys outside the allowlist, `first_block` without `leads`, an intents row with neither `body` nor `lead`, and `before-after` leads whose `path` capture disagrees with `infer_path`.
9. **Detection:** superpowers scores `### Task n:` 40, `- [ ] **Step n:` 40, `` Run: ` `` 20; before-after scores `## Task n:` 40, the same step item 40, `` `p`, before: `` 20. Each reaches the threshold (50) only on its own plans; neither devready plan reaches it, so both stay skipped until P2b.
10. **Real plans:** the superpowers plan parses with 4 tasks and no diagnostics; the before-after plan with 11 tasks and one warning (item 5). Both assay plans are unchanged.
11. **Diffs only for files over 300 lines**, as in P1; every other file is full contents. Every fence is one backtick longer than any fence line inside it.
12. **Final newlines:** `docs/decisions.md`, `docs/roadmap.md`, `docs/extending.md` and `docs/plan-grammar.md` had none at `3910b9b`. A full-contents block always ends in a newline, so the first task that writes each of them adds it; the replay caught the blob mismatch this would otherwise cause.

## Imperative Notes left in the real plans (D36)

Checked with `planrun show` on the P2a-T7 build. Text is not quoted (D16).

- `2026-09-19-heatmap-mode-toggle.md`: none. Its only Notes are the `---` rules between tasks and one sentence in Task 3's intro that says why the task has no unit test.
- `2026-11-19-cleanup-batched-runner.md`: no Note is an instruction. The Notes are the run-lead labels (`Preview:`, `Apply:`, `Verify:`, `Run:`), which stay Notes beside the script they lead, and descriptions of what a step's scripts do. Three related slips, each a choice:
  - Task 6, step 2: an `Expected (n):` paragraph and its list are Notes, not an expectation of the preview script before them (the lead is `Expected: …`). They are shown in place.
  - Task 6, step 5: the first preview script follows an explanatory paragraph that is no run lead, so the guard makes it `Manual(Unrecognized)` (D28): planrun waits instead of running it.
  - Task 3, step 7 and Task 10, step 3: an `Expected:` sentence carries a further instruction (what to fix if a count is off; open the app and look at a page). It is part of the prose expectation that you confirm (D11), not a separate Manual.
- The two devready plans are listed in P2b's plan.

## Tasks

### P2a-T1: outline fixes, task-after-end, decisions D26–D40

**Goal:** The two latent P1 outline bugs fixed before any new format, so the assay gate proves the fixes change nothing for assay: a manifest's items end at the first step boundary, and a task heading after the end of the tasks is the error `task-after-end` (D37). Every accepted P2 answer recorded as D26–D40; the roadmap splits P2 into P2a and P2b.

**Files**

- Modify `crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap`
- Modify `crates/planrun-parse/src/outline.rs`
- Modify `crates/planrun-parse/src/outline_tests.rs`
- Modify `docs/decisions.md`
- Modify `docs/roadmap.md`
- Create `tests/corpus/invalid/task-after-end.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-parse/src/outline.rs`.

The invalid-corpus snapshot gains the `task-after-end` entry; nothing else in it changes.

**Step 1 — write the files**

File `crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap` (full contents):

```text
---
source: crates/planrun-cli/tests/cli.rs
expression: all
---
==== bad-path.md
error[bad-path]: `../outside.rs` has a `.`, `..` or empty segment
  --> tests/corpus/invalid/bad-path.md:14:11
   |
14 | - Create `../outside.rs`
   |           ^^^^^^^^^^^^^

planrun: tests/corpus/invalid/bad-path.md: 1 error

==== bad-task-heading.md
error[bad-task-heading]: a level-3 heading among the tasks that is not a task: it does not match `^(?P<id>(?P<phase>P\d+[a-z]?(?:-\d+)?)-T(?P<ord>\d+)): (?P<title>.+)$`
  --> tests/corpus/invalid/bad-task-heading.md:24:1
   |
24 | ### P1-T2 b
   | ^^^^^^^^^^^

planrun: tests/corpus/invalid/bad-task-heading.md: 1 error

==== blob-chain.md
error[blob-chain]: this patch of `src/a.rs` starts from 3333333, but the plan's last change to it leaves 2222222
  --> tests/corpus/invalid/blob-chain.md:40:1
   |
18 | / File `src/a.rs` (exact diff; save the block as `$env:TEMP\p.patch`, outside the repository, then apply it):
19 | |
20 | | ```diff
21 | | diff --git a/src/a.rs b/src/a.rs
...  |
27 | | +b
28 | | ```
   | |___- the last change
...
40 | / File `src/a.rs` (exact diff; save the block as `$env:TEMP\p2.patch`, outside the repository, then apply it):
41 | |
42 | | ```diff
43 | | diff --git a/src/a.rs b/src/a.rs
...  |
49 | | +c
50 | | ```
   | |___^

planrun: tests/corpus/invalid/blob-chain.md: 1 error

==== commit-not-last.md
error[commit-not-last]: this runs after the commit of task `P1-T1`
  --> tests/corpus/invalid/commit-not-last.md:33:1
   |
26 | / ```powershell
27 | | git add -- src/a.rs
28 | | git commit -m "P1-T1: a"
29 | | ```
   | |___- the commit
...
33 | / ```powershell
34 | | cargo test
35 | | ```
   | |___^

planrun: tests/corpus/invalid/commit-not-last.md: 1 error

==== commit-twice.md
error[commit-twice]: a second commit in task `P1-T1`
  --> tests/corpus/invalid/commit-twice.md:31:1
   |
26 | / ```powershell
27 | | git add -- src/a.rs
28 | | git commit -m "P1-T1: a"
29 | | ```
   | |___- the first commit
30 |
31 | / ```powershell
32 | | git commit --amend --no-edit
33 | | ```
   | |___^

planrun: tests/corpus/invalid/commit-twice.md: 1 error

==== duplicate-task-id.md
error[duplicate-task-id]: task `P1-T1` is defined twice
  --> tests/corpus/invalid/duplicate-task-id.md:30:1
   |
 8 | ### P1-T1: a
   | ------------ first defined here
...
30 | ### P1-T1: b
   | ^^^^^^^^^^^^

planrun: tests/corpus/invalid/duplicate-task-id.md: 1 error

==== expected-spec.md
error[expected-spec]: `summary lines` names no exit status this format knows
  --> tests/corpus/invalid/expected-spec.md:30:1
   |
30 | Expected (summary lines):
   | ^^^^^^^^^^^^^^^^^^^^^^^^^

planrun: tests/corpus/invalid/expected-spec.md: 1 error

==== expected-twice.md
error[expected-twice]: the script before this already has an expectation
  --> tests/corpus/invalid/expected-twice.md:36:1
   |
26 | / ```powershell
27 | | cargo test
28 | | ```
...  |
33 | | test result: ok. …
34 | | ```
   | |___- the script
35 |
36 |   Expected: all tests pass.
   |   ^^^^^^^^^^^^^^^^^^^^^^^^^

planrun: tests/corpus/invalid/expected-twice.md: 1 error

==== expected-without-script.md
error[expected-without-script]: an expectation with no script directly before it
  --> tests/corpus/invalid/expected-without-script.md:24:1
   |
24 | Expected (exit code 0; summary lines):
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

planrun: tests/corpus/invalid/expected-without-script.md: 1 error

==== manifest-item.md
error[manifest-item]: a manifest item that does not match `^(?P<change>Create|Modify|Delete) `(?P<paths>[^`]+)`(?: \(.+\))?$`
  --> tests/corpus/invalid/manifest-item.md:14:1
   |
14 | - Rename `src/a.rs`
   | ^^^^^^^^^^^^^^^^^^^

planrun: tests/corpus/invalid/manifest-item.md: 1 error

==== manifest-missing.md
error[manifest-missing]: the task has no manifest (`^\*\*Files\*\*$`)
 --> tests/corpus/invalid/manifest-missing.md:8:1
  |
8 | ### P1-T1: a
  | ^^^^^^^^^^^^

planrun: tests/corpus/invalid/manifest-missing.md: 1 error

==== manifest-undeclared.md
error[manifest-undeclared]: `src/b.rs` is changed here but not declared in the task's files
  --> tests/corpus/invalid/manifest-undeclared.md:24:1
   |
12 | / **Files**
13 | |
14 | | - Create `src/a.rs`
   | |___________________- the task's files
...
24 | / File `src/b.rs` (full contents):
25 | |
26 | | ```rust
27 | | // x
28 | | ```
   | |___^

planrun: tests/corpus/invalid/manifest-undeclared.md: 1 error

==== manifest-untouched.md
error[manifest-untouched]: `src/b.rs` is declared, and no action of the task changes it
  --> tests/corpus/invalid/manifest-untouched.md:15:1
   |
15 | - Modify `src/b.rs`
   | ^^^^^^^^^^^^^^^^^^^

planrun: tests/corpus/invalid/manifest-untouched.md: 1 error

==== manifest-write-mode.md
error[manifest-write-mode]: full contents for `src/a.rs`, declared Delete: neither create nor overwrite
  --> tests/corpus/invalid/manifest-write-mode.md:18:1
   |
14 |   - Delete `src/a.rs`
   |   ------------------- declared here
...
18 | / File `src/a.rs` (full contents):
19 | |
20 | | ```rust
21 | | // x
22 | | ```
   | |___^

planrun: tests/corpus/invalid/manifest-write-mode.md: 1 error

==== missing-block.md
error[missing-block]: this sentence announces a fenced block, and none follows it
  --> tests/corpus/invalid/missing-block.md:18:1
   |
18 | File `src/a.rs` (full contents):
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

planrun: tests/corpus/invalid/missing-block.md: 1 error

==== no-format.md
error[no-format]: no format recognizes this plan (no detection score reaches 50); name one with --format

planrun: tests/corpus/invalid/no-format.md: 1 error

==== no-tasks.md
error[no-tasks]: no level-2 heading matches `^Tasks$`, so there are no tasks

planrun: tests/corpus/invalid/no-tasks.md: 1 error

==== patch-path.md
error[patch-path]: the patch changes `src/b.rs`; the sentence before it names `src/a.rs`
  --> tests/corpus/invalid/patch-path.md:20:1
   |
18 |   File `src/a.rs` (exact diff; save the block as `$env:TEMP\p.patch`, outside the repository, then apply it):
   |   ----------------------------------------------------------------------------------------------------------- the path named here
19 |
20 | / ```diff
21 | | diff --git a/src/b.rs b/src/b.rs
22 | | index 1111111..2222222 100644
23 | | --- a/src/b.rs
...  |
27 | | +b
28 | | ```
   | |___^

planrun: tests/corpus/invalid/patch-path.md: 1 error

==== patch-shape.md
error[patch-shape]: the hunk at patch line 5 ends early: 1 old and 0 new lines missing
  --> tests/corpus/invalid/patch-shape.md:18:1
   |
18 | / File `src/a.rs` (exact diff; save the block as `$env:TEMP\p.patch`, outside the repository, then apply it):
19 | |
20 | | ```diff
21 | | diff --git a/src/a.rs b/src/a.rs
...  |
27 | | +b
28 | | ```
   | |___^

planrun: tests/corpus/invalid/patch-shape.md: 1 error

==== step-order.md
error[step-order]: step 3 where step 2 comes next in task `P1-T1`
  --> tests/corpus/invalid/step-order.md:24:1
   |
24 | **Step 3 — verify**
   | ^^^^^^^^^^^^^^^^^

planrun: tests/corpus/invalid/step-order.md: 1 error

==== task-after-end.md
error[task-after-end]: a task heading after the tasks ended: planrun would never run it
  --> tests/corpus/invalid/task-after-end.md:26:1
   |
24 | ## Green state
   | -------------- the tasks end here
25 |
26 | ### P1-T2: b
   | ^^^^^^^^^^^^

planrun: tests/corpus/invalid/task-after-end.md: 1 error

==== task-order.md
error[task-order]: task `P1-T2` comes after task `P1-T3`
  --> tests/corpus/invalid/task-order.md:46:1
   |
30 | ### P1-T3: c
   | ------------ the earlier task
...
46 | ### P1-T2: b
   | ^^^^^^^^^^^^

planrun: tests/corpus/invalid/task-order.md: 1 error

==== unclosed-fence.md
error[unclosed-fence]: this fenced block is not closed where it looks closed: a line inside it opens another block
  --> tests/corpus/invalid/unclosed-fence.md:20:1
   |
20 | ```rust
   | ^^^^^^^
...
26 | ```powershell
   | ------------- this line opens a fenced block inside the open one

planrun: tests/corpus/invalid/unclosed-fence.md: 1 error

==== unknown-file-op.md
error[unknown-file-op]: `partial contents` is not a file operation this format knows (`exact diff`, `full contents`)
  --> tests/corpus/invalid/unknown-file-op.md:18:1
   |
18 | File `src/a.rs` (partial contents):
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

planrun: tests/corpus/invalid/unknown-file-op.md: 1 error
```

File `crates/planrun-parse/src/outline.rs` (exact diff; save the block as `$env:TEMP\p2a-t1-outline-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/outline.rs b/crates/planrun-parse/src/outline.rs
index 6ffd544..daa38f2 100644
--- a/crates/planrun-parse/src/outline.rs
+++ b/crates/planrun-parse/src/outline.rs
@@ -158,7 +158,9 @@ fn tasks_start(rules: &OutlineRules, doc: &Document) -> Result<usize, Diagnostic
         })
 }
 
-/// The task headings from `start`, and the index where the tasks end.
+/// The task headings from `start`, and the index where the tasks end. A heading that matches the
+/// task rule is a task; only a heading that is no task, at or above `tasks_end_level`, ends the
+/// tasks (decision D37).
 fn task_heads(
     rules: &OutlineRules,
     doc: &Document,
@@ -176,6 +178,7 @@ fn task_heads(
         }
         let in_tasks = !heads.is_empty() || rules.tasks_after.is_some();
         if !heads.is_empty() && rules.tasks_end_level.is_some_and(|l| *level <= l) {
+            late_tasks(rules, doc, i, errors);
             return (heads, i);
         }
         if in_tasks && *level == rules.task.level {
@@ -192,6 +195,25 @@ fn task_heads(
     (heads, doc.blocks.len())
 }
 
+/// Task headings after the heading at `end`, which ended the tasks: they would be text that never
+/// runs, so each is an error (`task-after-end`).
+fn late_tasks(rules: &OutlineRules, doc: &Document, end: usize, errors: &mut Vec<Diagnostic>) {
+    let ended = doc.blocks[end].span;
+    let late = doc.blocks[end + 1..]
+        .iter()
+        .filter(|b| heading_matches(doc, &b.block, &rules.task));
+    for located in late {
+        errors.push(
+            Diagnostic::error(
+                "task-after-end",
+                Some(located.span),
+                "a task heading after the tasks ended: planrun would never run it",
+            )
+            .with_related(ended, "the tasks end here"),
+        );
+    }
+}
+
 fn heading_matches(doc: &Document, block: &Block, rule: &HeadingRule) -> bool {
     let Block::Heading { level, text } = block else {
         return false;
@@ -275,7 +297,8 @@ fn capture_ref(caps: Option<&Captures<'_>>, name: &str, base: u32) -> Option<Tex
 }
 
 /// The manifest: the first block before any step boundary that matches the marker, and the list
-/// items right after it.
+/// items right after it, up to the first step boundary (a list-item step right after the items
+/// is in the same CommonMark list, decision D37).
 fn manifest(
     rules: &OutlineRules,
     doc: &Document,
@@ -301,7 +324,9 @@ fn manifest(
     let items_end = (marker + 1..body.end)
         .find(|&i| {
             let b = &doc.blocks[i];
-            !matches!(b.block, Block::ListItem { .. }) || b.depth != depth
+            !matches!(b.block, Block::ListItem { .. })
+                || b.depth != depth
+                || boundary(rules, doc, i).is_some()
         })
         .unwrap_or(body.end);
     if items_end == marker + 1 {
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2a-t1-outline-rs.patch"
Remove-Item "$env:TEMP\p2a-t1-outline-rs.patch"
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

File `docs/decisions.md` (full contents):

```markdown
# Decisions

Numbered, never renumbered. A changed decision gets a new number that supersedes the old one;
the old entry stays with a "superseded by" line. Code and docs cite them as `D<n>`.

Agreed between Rens and Claude on 2026-11-24, from two proposals (Rens' "Deterministic Plan
Execution CLI" and Claude's pattern study of six real plans).

## D1 Priorities, and what planrun is

Correctness > execution speed > low memory > predictability > maintainability/extensibility.
planrun is not an agent: it does exactly what the plan says, shows what it cannot do
mechanically, and decides nothing. Rust.

Order superseded by D25.

## D2 Four crates; parsing never executes, execution never parses

`planrun-model` (IR), `planrun-parse` (depends on model), `planrun-exec` (depends on model, **not**
on parse), `planrun-cli`. The crate graph enforces the separation.

## D3 A plan runs only when it parsed and checked without errors

Parse the whole document first; any error means no `Plan`. No partial execution.

## D4 The action set is closed

`WriteFile`, `ApplyPatch`, `ReplaceExact`, `Delete`, `Generated`, `RunScript`, `Commit`, `Note`,
`Manual`. Formats vary, actions don't. Adding one is a cross-cutting change (extending.md §4).

## D5 Phases are metadata

One plan file is one phase in every real plan. `PlanMeta::phase`, not a tree level.

## D6 Normalize the plan once

At load: strip a BOM, CRLF/CR → LF. Every span, fence body and line number refers to the
normalized text.

## D7 RepoPath is the only path in an action

Relative, forward slashes, no `..`/`.`/empty segments, no drive letter, no backslash, no NUL.
Symlink escapes are refused by the workspace at write time. Nothing outside the repository is
written except under `.git/planrun/` and the OS temp directory.

## D8 Formats are data first; built-ins are specs too

Three extension levels: TOML spec (no rebuild) → recognizer kind (Rust) → Format (Rust). The
built-in formats are TOML specs embedded in the binary, so user formats take the same path.

## D9 Dependencies

Each dependency is justified here; add none without a line.

| Crate | Why |
|---|---|
| `pulldown-cmark` | CommonMark with byte offsets; correct nested-fence handling |
| `regex` | spec patterns; linear time, no backtracking blow-ups |
| `serde`, `toml` | format specs |
| `serde_json` | the journal (JSONL) |
| `thiserror` | error types |
| `sha1_smol` | git blob ids in-process: the `blob-chain` check from a created file's contents (planrun-parse); the fast path in planrun-exec, where `git hash-object` is the authority |
| `similar` | showing diffs of overwrites and replacements. **Not** for applying patches (D20) |
| `clap` | arguments |
| `dialoguer` | line prompts (D15) |
| `annotate-snippets` | diagnostics with source context |
| `insta` (dev) | golden snapshots of the IR |
| `sha2` (Phase 1) | the plan fingerprint (SHA-256) |

Rejected: `rusqlite` (D18), `ratatui` for now (D15), `tracing` for now (the journal is the record),
`similar` as a patch engine (D20).

## D10 Line endings of written files

Created files: as in the plan (LF). Overwrite and `ReplaceExact`: keep the target's existing
convention unless `.gitattributes` says `eol=lf`; `before` is matched in LF space. Patches: through
`git apply` unchanged.

## D11 Prose never passes a script on its own

`Expected: all pass` is mapped to an exit expectation, checked, and then shown for confirmation.
Only fenced summary lines and exit codes pass automatically.

## D12 Expected failures are skipped by default

A verification expected to fail (TDD "verify it fails", `Expected: build error`) is recorded in
the IR and skipped; `--run-expected-failures` runs it. Rens sees these failures in the IDE.

## D13 Policy classes

Read < Build < Modify < Destructive; a script takes the highest class of its commands; unknown
commands are Destructive. Defaults: file actions automatic; Build/Modify ask; Destructive needs a
typed `yes`; Manual always waits.

## D14 Resume checks the disk, never the journal alone

An action recorded `Running` or `Done` is re-examined: postimages present → done; preimages
present → not done; otherwise the user decides.

## D15 Line prompts first

`dialoguer` and plain output keep cargo's output in the terminal's scrollback. A ratatui TUI can
come later behind the `Ui` trait.

## D16 Real plans stay local

The six real plans contain customer and internal details. They live in `tests/corpus/local/`
(git-ignored) and are parsed by an `#[ignore]`d test. Committed tests use synthetic plans.

## D17 The patch ceremony is absorbed

The `git apply "$env:TEMP\….patch"` / `Remove-Item` block after an exact diff is never run:
planrun applies the patch itself. There is no `--verbatim` mode.

## D18 The journal is JSONL in `.git/planrun/`

One writer, sequential, a few hundred records per plan: SQLite adds a C dependency and a binary
file nobody can read when it matters. JSONL is appendable, crash-tolerant (a torn last line is
dropped), and readable. Inside `.git/` so `git status --porcelain` stays empty.

## D19 Windows first

Every real plan is PowerShell 7 on Windows. Linux (pwsh and bash) is supported and is where CI
can run.

## D20 Patches are applied by `git apply`

With the patch on stdin, `--whitespace=nowarn`, after `--check`. Same semantics as Rens' manual
runs; exact context; no fuzz. Blob ids from `index` lines are verified before and after.

## D21 Note vs Manual

Prose no recognizer consumed is a `Note` (shown, never blocks). An unconsumed fence, prose that
matches a format's manual markers, and edits described in prose are `Manual` (always blocks).

## D22 Commits are gated twice

The plan's commit script runs as written (it may gate itself on `$gate`), and planrun runs it only
when every verification of the task passed or was skipped by policy (D12).

## D23 File actions are transactional per step

All file actions of a step, up to the next non-file action, are validated in memory before any is
written; pre-images are stored in the git object database first; writes are temp-file + rename.

## D24 One persistent session per shell kind per run

Variables and the working directory persist across blocks (`$gate`, `$P2`), as in Rens' terminal.

## D25 Maintainability/extensibility ranks second

Supersedes D1's order; the rest of D1 (not an agent, Rust) stands.
Correctness > maintainability/extensibility > execution speed > low memory > predictability.
Reason: new plan styles keep arriving, so planrun is only valuable if adding one stays cheap
and safe; speed and memory rarely decide anything, because cargo, npm and dotnet dominate
wall time.

Decisions D26–D40 were agreed on 2026-11-24 at the start of P2, from the review of the three
hand-written formats against their real plans.

## D26 The file operation decides Create or Overwrite

A lead that says create writes with `Create`, one that says replace writes with `Overwrite`,
whatever the manifest says: Create refuses an existing target at preflight (P3), so the op
deciding is safe. Only the `write` op takes the mode from the manifest. Where a task has a
manifest and its entry disagrees with the op (Create on a Modify/Rewrite/Delete entry, Overwrite
on a Create/Delete entry), `manifest-write-mode` is an error, in every format.

## D27 Optional manifests; `Generated` entries

`[outline.manifest] required` (default true) lets a format have tasks without a Files list; the
manifest checks skip such a task. The marker may capture `change` for formats whose items are
bare paths. `ChangeKind::Generated` is a manifest change of its own (a model variant, not an
action kind, so D4 does not apply): a generated entry needs no action, and `show` reports it
as generated.

## D28 A shell fence runs only behind a run lead in hand-written formats

`script` takes `leads` (regexes over the paragraph directly before the fence) and `first_block`
(the fence is the first block of a written step). With `leads` set, the kind claims a fence only
when one of the two holds; any other shell fence falls to the fallback, `Manual(Unrecognized)`.
`first_block` never applies to the implicit first step, so never to the block right after a
manifest. A body intent (`git commit` in the body) never qualifies a fence on its own: a plan
can create a script file whose contents contain git commands. Reason: hand-written plans put
file contents and code to insert in shell fences behind prose that no recognizer can reliably
name; unguarded, `script` would execute them. The assay spec sets no `leads` and is unchanged.

## D29 Script intents by lead; no `preview-apply` kind

An `intents` row of `script` matches the body, the lead paragraph, or both (`body`, `lead`).
`leads` decides whether the kind claims a fence; the per-intent `lead` decides which intent a
claimed fence gets. Preview/apply pairs need nothing more, so the `preview-apply` kind is
dropped.

## D30 Step marks replace the `expected-failure-step` kind

A recognizer cannot see a step's title, so the title is matched by the outline:
`[[step_marks]] title = '…', mark = "expected-failure" | "human-check"`.

- `expected-failure`: every `Verify` script of the step (never a commit or a gate) expects
  failure. A script with no expectation or with `any` becomes `Failure`; one that expects success
  is an error, `expected-conflict`.
- `human-check`: every `Note` of the step becomes `Manual(HumanCheck)`.

## D31 A spec may downgrade only the manifest coverage checks

`[checks]` maps a check code to `error`, `warning` or `off`, and only `manifest-untouched` and
`manifest-undeclared` may appear in it. Any other key is a spec load error, so a user spec can
never switch off a safety check: `manifest-write-mode` and every outline and recognizer error
stay errors in every format. Reason: in hand-written plans, prose edits (Manual) and scripts
(`replace`) change declared files, and manifests name ranges and globs.

## D32 Paths resolved from the manifest

Deterministic and only against the task's own manifest:

- In a manifest item, a `:lines` token adds lines to the path before it (every format).
- `[outline.manifest] siblings = true`: a token without `/` after a token with a directory is a
  file in that directory. Opt-in, because `crates/x/src/a.rs`, `Cargo.toml` in one item would
  otherwise mean `crates/x/src/Cargo.toml`.
- A recognizer parameter `file_name_from_manifest`: a lead path without `/` that is not itself
  an entry resolves to the one entry with that file name, when that entry was written in full
  (not a sibling). Otherwise the path stays as written.
- `before-after` parameter `infer_path`: a lead with no path takes the task's one entry that is
  not Test, Generated or Delete. Never for writes. With none or several, the lead and its fences
  are `Manual(ProseEdit)`.

Each resolved path carries its `PathOrigin`, and `show` marks it.

## D33 The DevReady `**Commit:** \`msg\`` line is Manual

`Manual(HumanCheck)` carrying the message. planrun synthesizes no commit and never runs
`git add -A`. P5 shows `git add -- <paths>; git commit -m "<msg>"` as a suggestion only when every
manifest path of the task is literal (no glob, range, sibling or unresolved file name);
otherwise it shows the message alone.

## D34 Inline edits are Manual for now

`Edit: \`a\` → \`b\``, `Old: \`a\`` / `New: \`b\`` and similar one-line edits become
`Manual(ProseEdit)`: code spans lose whitespace. Revisit after dogfooding.

## D35 Comment expectations in DevReady scripts

A `script` parameter `line_expect` (a regex over each line) turns trailing comments such as
`# feature/p2-tooling` into one prose expectation per fence: exit `Unknown`, confirmed (D11).
DevReady only. A fence that also has an `Expected:` after it is `expected-twice`.

## D36 Imperative prose

Per-format `manual_markers` and `human-check` step marks catch the instructions that would
otherwise be Notes (D21). Each phase plan lists, by file, task and step, the imperative Notes
that remain in the real plans, so every slip is a recorded choice.

## D37 Outline fixes

- A manifest's items end at the first step boundary: a list-item step right after the manifest
  is in the same CommonMark list, and is a step, not an item.
- A heading that matches the task rule is a task; only a heading that is no task, at or above
  `tasks_end_level`, ends the tasks. A task heading after that end is an error, `task-after-end`:
  it would otherwise be text that never runs.

## D38 An expectation after a Manual joins it

An `Expected: …` sentence directly after a `Manual` (an `sql` fence, a fence no lead claimed)
becomes part of that Manual's text, instead of `expected-without-script`. After a `Note` it stays
an error.

## D39 `run-line` names its shell

Each `run-line` recognizer has a `shell` parameter; there is no format default. pwsh for the
superpowers format: its run lines share no variables with its bash commit fences (D24).

## D40 Target for DevReady plans

They are written for an agent (probes, conditional branches, "record"). The target is zero errors
and honest Manual coverage, not automation. P2 is split: P2a builds the schema, the kinds and the
superpowers and before-after formats; P2b the devready format on the same schema.
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

- **P2a.** The P1 outline fixes and `task-after-end` (D37); the spec schema (`required`, a marker
  that captures `change`, `siblings`, `ChangeKind::Generated`, `[checks]`, `[[step_marks]]`,
  `script` `leads`/`first_block`/intent `lead`); the kinds `run-line`, `before-after`,
  `manual-prose`; path resolution from the manifest (D32); `formats/superpowers.toml` and
  `before-after.toml` with IR snapshots of their synthetic plans. The superpowers and
  before-after local plans parse with zero errors.
- **P2b.** `formats/devready.toml`: steps whose boundary block stays in the step
  (`keep = true`), inline Files clauses, `line_expect` (D35), the `**Commit:**` Manual (D33),
  the IR snapshot of its synthetic plan. All six local plans parse with zero errors.

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

File `tests/corpus/invalid/task-after-end.md` (full contents):

````markdown
<!-- expect: task-after-end -->
<!-- why: a level-2 heading ended the tasks, so P1-T2 after it would never run -->

# Invalid — task-after-end

## Tasks

### P1-T1: a

**Goal:** a.

**Files**

- Create `src/a.rs`

**Step 1 — write the files**

File `src/a.rs` (full contents):

```rust
// x
```

## Green state

### P1-T2: b

**Goal:** b.
````

**Step 2 — verify**

Targeted: the outline tests:

```powershell
cargo test -p planrun-parse --lib outline
```

Expected (exit code 0; summary lines):

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 36 filtered out; finished in …s
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap crates/planrun-parse/src/outline.rs crates/planrun-parse/src/outline_tests.rs docs/decisions.md docs/roadmap.md tests/corpus/invalid/task-after-end.md
  $verified = [ordered]@{
    'crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap' = '44e270ecdec75846e1ae79c3baf71b6e501d5adf'
    'crates/planrun-parse/src/outline.rs' = 'daa38f2b1bf9615af9549eae0b233e32dbf39d34'
    'crates/planrun-parse/src/outline_tests.rs' = '72d187e2513f0bd1afa8e768e0743054cb1486d5'
    'docs/decisions.md' = '1c043081e7d847e638888d469a99de430067a7bb'
    'docs/roadmap.md' = 'd060d74a88a43ae93768623863276ef00c69874f'
    'tests/corpus/invalid/task-after-end.md' = 'f1dd935bb73ed228427c011def3d500fbdc7b532'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P2a-T1: outline fixes, task-after-end, decisions D26-D40" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P2a-T2: manifest schema, Generated entries, op-manifest agreement, [checks]

**Goal:** The manifest keys of the spec schema: `required` (default true, D27), a marker that captures `change`, `siblings` and `:lines` tokens (D32); `ChangeKind::Generated` (D27) and `PathOrigin` in the model; `manifest-write-mode` also when a write's op contradicts its entry (D26); the `[checks]` allowlist (D31). docs/extending.md §2 moves with spec.rs.

**Files**

- Modify `crates/planrun-cli/src/view.rs`
- Modify `crates/planrun-model/src/lib.rs`
- Modify `crates/planrun-model/src/path.rs`
- Modify `crates/planrun-model/src/plan.rs`
- Modify `crates/planrun-parse/src/check.rs`
- Modify `crates/planrun-parse/src/format.rs`
- Modify `crates/planrun-parse/src/lib.rs`
- Modify `crates/planrun-parse/src/outline.rs`
- Modify `crates/planrun-parse/src/spec.rs`
- Create `crates/planrun-parse/tests/manifest_schema.rs`
- Modify `docs/extending.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/src/view.rs`, `crates/planrun-parse/src/check.rs`, `crates/planrun-parse/src/outline.rs`, `crates/planrun-parse/src/spec.rs`.

**Step 1 — write the files**

File `crates/planrun-cli/src/view.rs` (exact diff; save the block as `$env:TEMP\p2a-t2-view-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/view.rs b/crates/planrun-cli/src/view.rs
index cae388c..556ecb8 100644
--- a/crates/planrun-cli/src/view.rs
+++ b/crates/planrun-cli/src/view.rs
@@ -295,7 +295,12 @@ fn entry(entry: &ManifestEntry) -> String {
         .lines
         .map(|l| format!(":{}-{}", l.start, l.end))
         .unwrap_or_default();
-    format!("{:?} {}{lines}", entry.change, entry.path)
+    let sibling = if entry.origin.is_written() {
+        ""
+    } else {
+        " (in the directory of the path before)"
+    };
+    format!("{:?} {}{lines}{sibling}", entry.change, entry.path)
 }
 
 /// How much of each action `show` prints.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2a-t2-view-rs.patch"
Remove-Item "$env:TEMP\p2a-t2-view-rs.patch"
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
mod plan;
mod source;
mod span;

pub use action::{Action, ActionKind, ManualReason, Script, ScriptIntent, Shell, WriteMode};
pub use expect::{ExitExpect, Expectation};
pub use ids::{AbbrevBlobId, BlobIdError, FormatId, RecognizerId, TaskId};
pub use path::{PathOrigin, RepoPath, RepoPathError};
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
/// empty segment, no backslash, no NUL.
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
        ] {
            assert!(RepoPath::new(bad).is_err(), "{bad:?}");
        }
    }
}
```

File `crates/planrun-model/src/plan.rs` (full contents):

```rust
//! Plan, task, step and the task's file manifest.

use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::ids::{FormatId, TaskId};
use crate::path::{PathOrigin, RepoPath};
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
    /// The file.
    pub path: RepoPath,
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

File `crates/planrun-parse/src/check.rs` (exact diff; save the block as `$env:TEMP\p2a-t2-check-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/check.rs b/crates/planrun-parse/src/check.rs
index 3dc4596..2375dba 100644
--- a/crates/planrun-parse/src/check.rs
+++ b/crates/planrun-parse/src/check.rs
@@ -28,8 +28,9 @@ pub trait Rule: Send + Sync {
 /// - `step-order`: written step numbers count up from 1 in each task.
 /// - `manifest`: every file action's path is declared (`manifest-undeclared`); every declared
 ///   Create/Modify/Rewrite/Delete is touched by an action or a `Generated`
-///   (`manifest-untouched`); a `WriteFile { FromManifest }` on a declared path resolves to Create
-///   or Overwrite (`manifest-write-mode`). A task without a manifest block is skipped.
+///   (`manifest-untouched`); a `WriteFile` on a declared path has a mode that agrees with the
+///   entry: `FromManifest` resolves, Create is not on an existing file, Overwrite not on a new one
+///   (`manifest-write-mode`, decision D26). A task without a manifest block is skipped.
 /// - `patch-shape`: a patch has at least one hunk and its hunks' line counts match their bodies.
 ///   (Its paths and `index` line are checked when it is recognized: `patch-path`, `patch-shape`.)
 /// - `blob-chain`: per path, each patch's preimage equals the postimage of the last earlier
@@ -182,20 +183,13 @@ fn check_manifest(task: &Task, out: &mut Vec<Diagnostic>) {
             );
             continue;
         };
-        let unresolved = matches!(
-            action.kind,
-            ActionKind::WriteFile {
-                mode: WriteMode::FromManifest,
-                ..
-            }
-        );
-        if unresolved {
+        if let Some(problem) = write_mode_problem(&action.kind, entry.change) {
             out.push(
                 Diagnostic::error(
                     "manifest-write-mode",
                     Some(action.span),
                     format!(
-                        "full contents for `{path}`, declared {:?}: neither create nor overwrite",
+                        "full contents for `{path}`, declared {:?}: {problem}",
                         entry.change
                     ),
                 )
@@ -203,7 +197,7 @@ fn check_manifest(task: &Task, out: &mut Vec<Diagnostic>) {
             );
         }
     }
-    let must_touch = |c: ChangeKind| !matches!(c, ChangeKind::Test);
+    let must_touch = |c: ChangeKind| !matches!(c, ChangeKind::Test | ChangeKind::Generated);
     for entry in &task.manifest.entries {
         if must_touch(entry.change) && !touched.contains(&&entry.path) {
             out.push(Diagnostic::error(
@@ -218,6 +212,24 @@ fn check_manifest(task: &Task, out: &mut Vec<Diagnostic>) {
     }
 }
 
+/// What is wrong with a full-contents write to a path the manifest declares `change`: a mode the
+/// manifest cannot resolve, or an op that contradicts it (decision D26).
+fn write_mode_problem(kind: &ActionKind, change: ChangeKind) -> Option<&'static str> {
+    let ActionKind::WriteFile { mode, .. } = kind else {
+        return None;
+    };
+    match (mode, change) {
+        (WriteMode::FromManifest, _) => Some("neither create nor overwrite"),
+        (WriteMode::Create, ChangeKind::Modify | ChangeKind::Rewrite | ChangeKind::Delete) => {
+            Some("the plan creates a file the manifest says exists")
+        }
+        (WriteMode::Overwrite, ChangeKind::Create | ChangeKind::Delete) => {
+            Some("the plan overwrites a file the manifest says is new or deleted")
+        }
+        _ => None,
+    }
+}
+
 /// `patch-shape`.
 pub struct PatchShape;
 
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2a-t2-check-rs.patch"
Remove-Item "$env:TEMP\p2a-t2-check-rs.patch"
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

File `crates/planrun-parse/src/outline.rs` (exact diff; save the block as `$env:TEMP\p2a-t2-outline-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/outline.rs b/crates/planrun-parse/src/outline.rs
index daa38f2..8c5fdde 100644
--- a/crates/planrun-parse/src/outline.rs
+++ b/crates/planrun-parse/src/outline.rs
@@ -3,7 +3,8 @@
 //! format may too.
 
 use planrun_model::{
-    ChangeKind, LineRange, Manifest, ManifestEntry, PlanMeta, RepoPath, Span, TaskId, TextRef,
+    ChangeKind, LineRange, Manifest, ManifestEntry, PathOrigin, PlanMeta, RepoPath, Span, TaskId,
+    TextRef,
 };
 use regex::{Captures, Regex};
 
@@ -35,14 +36,18 @@ pub struct HeadingRule {
 
 /// A manifest: a marker paragraph and the list items after it.
 pub struct ManifestRules {
-    /// Over the marker paragraph.
+    /// Over the marker paragraph; may capture `change` for items that are bare paths.
     pub marker: Regex,
-    /// Over each item; captures `change` and `paths`.
+    /// Over each item; captures `paths`, and `change` unless the marker does.
     pub item: Regex,
     /// The captured change word and what it means.
     pub changes: Vec<(String, ChangeKind)>,
     /// Splits `paths` into several, when set.
     pub path_list: Option<Regex>,
+    /// When false, a task may have no manifest (decision D27).
+    pub required: bool,
+    /// A bare file name after a path with a directory is in that directory (decision D32).
+    pub siblings: bool,
 }
 
 /// A block that starts a step.
@@ -313,13 +318,23 @@ fn manifest(
         .take_while(|&i| boundary(rules, doc, i).is_none())
         .find(|&i| matches!(&doc.blocks[i].block, Block::Paragraph { text } if spec.marker.is_match(doc.text(text))));
     let Some(marker) = marker else {
-        errors.push(Diagnostic::error(
-            "manifest-missing",
-            Some(doc.blocks[head].span),
-            format!("the task has no manifest (`{}`)", spec.marker.as_str()),
-        ));
+        if spec.required {
+            errors.push(Diagnostic::error(
+                "manifest-missing",
+                Some(doc.blocks[head].span),
+                format!("the task has no manifest (`{}`)", spec.marker.as_str()),
+            ));
+        }
         return (Manifest::default(), None);
     };
+    let marker_change = match &doc.blocks[marker].block {
+        Block::Paragraph { text } => spec
+            .marker
+            .captures(doc.text(text))
+            .and_then(|c| c.name("change"))
+            .map(|m| m.as_str()),
+        _ => None,
+    };
     let depth = doc.blocks[marker].depth;
     let items_end = (marker + 1..body.end)
         .find(|&i| {
@@ -338,7 +353,7 @@ fn manifest(
     }
     let mut entries = Vec::new();
     for i in marker + 1..items_end {
-        manifest_item(spec, doc, i, &mut entries, errors);
+        manifest_item(spec, doc, i, marker_change, &mut entries, errors);
     }
     let range = BlockRange {
         start: marker,
@@ -355,10 +370,12 @@ fn manifest(
     (manifest, Some(range))
 }
 
+/// One manifest item: its change (captured by the item, or else by the marker) and its paths.
 fn manifest_item(
     spec: &ManifestRules,
     doc: &Document,
     index: usize,
+    marker_change: Option<&str>,
     entries: &mut Vec<ManifestEntry>,
     errors: &mut Vec<Diagnostic>,
 ) {
@@ -369,7 +386,7 @@ fn manifest_item(
     let base = text.span().start;
     let item = doc.text(text);
     let parsed = spec.item.captures(item).and_then(|caps| {
-        let change = caps.name("change")?.as_str();
+        let change = caps.name("change").map(|m| m.as_str()).or(marker_change)?;
         let kind = spec.changes.iter().find(|(w, _)| w == change)?.1;
         Some((kind, caps.name("paths")?))
     });
@@ -389,15 +406,27 @@ fn manifest_item(
         None => vec![paths.as_str()],
     };
     let paths_span = Span::new(base + offset(paths.start()), base + offset(paths.end()));
+    let mut previous: Option<(RepoPath, PathOrigin)> = None;
     for piece in pieces.iter().map(|p| p.trim()) {
-        let (path, lines) = split_lines(piece);
-        match RepoPath::new(path) {
-            Ok(path) => entries.push(ManifestEntry {
-                path,
-                change,
-                lines,
-                span: located.span,
-            }),
+        let Some((path, origin, lines)) = token(spec, piece, previous.as_ref()) else {
+            errors.push(Diagnostic::error(
+                "bad-path",
+                Some(paths_span),
+                format!("`{piece}` is not a path"),
+            ));
+            continue;
+        };
+        match path {
+            Ok(path) => {
+                previous = Some((path.clone(), origin));
+                entries.push(ManifestEntry {
+                    path,
+                    change,
+                    lines,
+                    span: located.span,
+                    origin,
+                });
+            }
             Err(e) => errors.push(Diagnostic::error(
                 "bad-path",
                 Some(paths_span),
@@ -407,18 +436,53 @@ fn manifest_item(
     }
 }
 
+/// A path as spelled.
+type Spelled = Result<RepoPath, planrun_model::RepoPathError>;
+
+/// One token of a manifest item, after the token `previous` (decision D32): `:3-10` is more lines
+/// of the previous path; with `siblings`, a bare file name after a path with a directory is in
+/// that directory. `None` for a `:lines` token with no path before it.
+fn token(
+    spec: &ManifestRules,
+    piece: &str,
+    previous: Option<&(RepoPath, PathOrigin)>,
+) -> Option<(Spelled, PathOrigin, Option<LineRange>)> {
+    if let Some(range) = piece.strip_prefix(':') {
+        let (path, origin) = previous?;
+        return Some((Ok(path.clone()), *origin, line_range(range)));
+    }
+    let (path, lines) = split_lines(piece);
+    let dir = previous.and_then(|(p, _)| p.dir());
+    match dir {
+        Some(dir) if spec.siblings && !path.contains('/') => Some((
+            RepoPath::new(&format!("{dir}/{path}")),
+            PathOrigin::Sibling,
+            lines,
+        )),
+        _ => Some((RepoPath::new(path), PathOrigin::Written, lines)),
+    }
+}
+
 /// `src/a.rs:3-10` → (`src/a.rs`, 3–10); `src/a.rs:7` → (`src/a.rs`, 7–7).
 fn split_lines(path: &str) -> (&str, Option<LineRange>) {
     let Some((file, range)) = path.rsplit_once(':') else {
         return (path, None);
     };
-    let (start, end) = range.split_once('-').unwrap_or((range, range));
-    match (start.parse(), end.parse()) {
-        (Ok(start), Ok(end)) => (file, Some(LineRange { start, end })),
-        _ => (path, None),
+    match line_range(range) {
+        Some(lines) => (file, Some(lines)),
+        None => (path, None),
     }
 }
 
+/// `3-10` or `7`.
+fn line_range(range: &str) -> Option<LineRange> {
+    let (start, end) = range.split_once('-').unwrap_or((range, range));
+    Some(LineRange {
+        start: start.parse().ok()?,
+        end: end.parse().ok()?,
+    })
+}
+
 /// A step boundary's `n` capture.
 enum StepNumber {
     /// No `n` capture.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2a-t2-outline-rs.patch"
Remove-Item "$env:TEMP\p2a-t2-outline-rs.patch"
```

File `crates/planrun-parse/src/spec.rs` (exact diff; save the block as `$env:TEMP\p2a-t2-spec-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/spec.rs b/crates/planrun-parse/src/spec.rs
index 610f104..10df33f 100644
--- a/crates/planrun-parse/src/spec.rs
+++ b/crates/planrun-parse/src/spec.rs
@@ -12,7 +12,7 @@ use serde::Deserialize;
 
 use crate::diagnostic::Diagnostics;
 use crate::document::Document;
-use crate::format::{Format, FormatSettings, Outline};
+use crate::format::{CheckLevel, DOWNGRADABLE, Format, FormatSettings, Outline};
 use crate::outline::{self, HeadingRule, ManifestRules, OutlineRules, StepBlock, StepRule};
 use crate::recognizer::{Recognizer, RecognizerFactory};
 
@@ -32,6 +32,11 @@ pub struct FormatSpec {
     /// Shells and manual markers.
     #[serde(default)]
     pub settings: SettingsSpec,
+    /// Severity overrides for the manifest coverage checks (decision D31): check code →
+    /// `error`, `warning` or `off`. Only `manifest-untouched` and `manifest-undeclared` may
+    /// appear.
+    #[serde(default)]
+    pub checks: std::collections::BTreeMap<String, String>,
     /// Recognizer instances, in order; each table has `kind` plus the kind's parameters.
     #[serde(default, rename = "recognizer")]
     pub recognizers: Vec<toml::Table>,
@@ -87,14 +92,21 @@ pub struct HeadingPattern {
 #[derive(Debug, Clone, Deserialize)]
 #[serde(deny_unknown_fields)]
 pub struct ManifestSpec {
-    /// Regex over the marker paragraph (`^\*\*Files:?\*\*$`).
+    /// Regex over the marker paragraph (`^\*\*Files:?\*\*$`); may capture `change`, which then
+    /// applies to every item that does not capture its own.
     pub marker: String,
-    /// Regex over each list item; captures `change` and `paths` (split by `path_list` when set).
+    /// Regex over each list item; captures `paths` (split by `path_list` when set), and `change`
+    /// unless the marker captures it.
     pub item: String,
     /// Maps the captured `change` word to Create / Modify / Rewrite / Delete / Test.
     pub changes: std::collections::BTreeMap<String, String>,
     /// When set, the `paths` capture is split on this regex into several paths.
     pub path_list: Option<String>,
+    /// Every task has a manifest; default true. False lets a task have none (decision D27).
+    pub required: Option<bool>,
+    /// A bare file name after a path with a directory is in that directory; default false
+    /// (decision D32).
+    pub siblings: Option<bool>,
 }
 
 /// A block that starts a step.
@@ -155,7 +167,8 @@ impl SpecFormat {
             .map(|p| Ok((compile_regex("detect", &p.regex, &[])?, p.weight)))
             .collect::<Result<_, String>>()?;
         let outline = compile_outline(&spec.outline)?;
-        let settings = compile_settings(&spec.settings)?;
+        let mut settings = compile_settings(&spec.settings)?;
+        settings.checks = compile_checks(&spec.checks)?;
         let recognizers = spec
             .recognizers
             .iter()
@@ -249,15 +262,24 @@ fn compile_manifest(spec: &ManifestSpec) -> Result<ManifestRules, String> {
         .iter()
         .map(|(word, kind)| Ok((word.clone(), change_kind(kind)?)))
         .collect::<Result<_, String>>()?;
+    let marker = compile_regex("outline.manifest.marker", &spec.marker, &[])?;
+    let marker_change = marker.capture_names().flatten().any(|n| n == "change");
+    let item_needs: &[&str] = if marker_change {
+        &["paths"]
+    } else {
+        &["change", "paths"]
+    };
     Ok(ManifestRules {
-        marker: compile_regex("outline.manifest.marker", &spec.marker, &[])?,
-        item: compile_regex("outline.manifest.item", &spec.item, &["change", "paths"])?,
+        marker,
+        item: compile_regex("outline.manifest.item", &spec.item, item_needs)?,
         changes,
         path_list: spec
             .path_list
             .as_deref()
             .map(|p| compile_regex("outline.manifest.path_list", p, &[]))
             .transpose()?,
+        required: spec.required.unwrap_or(true),
+        siblings: spec.siblings.unwrap_or(false),
     })
 }
 
@@ -268,8 +290,9 @@ fn change_kind(name: &str) -> Result<ChangeKind, String> {
         "rewrite" => Ok(ChangeKind::Rewrite),
         "delete" => Ok(ChangeKind::Delete),
         "test" => Ok(ChangeKind::Test),
+        "generated" => Ok(ChangeKind::Generated),
         other => Err(format!(
-            "outline.manifest.changes: `{other}` is not create, modify, rewrite, delete or test"
+            "outline.manifest.changes: `{other}` is not create, modify, rewrite, delete, test or generated"
         )),
     }
 }
@@ -285,6 +308,34 @@ fn step_block(name: &str) -> Result<StepBlock, String> {
     }
 }
 
+/// The `[checks]` table: only the allowlisted codes, each `error`, `warning` or `off`.
+fn compile_checks(
+    table: &std::collections::BTreeMap<String, String>,
+) -> Result<Vec<(&'static str, CheckLevel)>, String> {
+    table
+        .iter()
+        .map(|(code, level)| {
+            let Some(code) = DOWNGRADABLE.iter().find(|c| **c == code.as_str()) else {
+                return Err(format!(
+                    "checks: `{code}` cannot be changed; only {} can",
+                    DOWNGRADABLE.join(" and ")
+                ));
+            };
+            let level = match level.as_str() {
+                "error" => CheckLevel::Error,
+                "warning" => CheckLevel::Warning,
+                "off" => CheckLevel::Off,
+                other => {
+                    return Err(format!(
+                        "checks.{code}: `{other}` is not error, warning or off"
+                    ));
+                }
+            };
+            Ok((*code, level))
+        })
+        .collect()
+}
+
 fn compile_settings(spec: &SettingsSpec) -> Result<FormatSettings, String> {
     let strings = |list: &[&str]| list.iter().map(|s| (*s).to_owned()).collect();
     Ok(FormatSettings {
@@ -301,6 +352,7 @@ fn compile_settings(spec: &SettingsSpec) -> Result<FormatSettings, String> {
             .bash
             .clone()
             .unwrap_or_else(|| strings(&["bash", "sh", "shell"])),
+        checks: Vec::new(),
     })
 }
 
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2a-t2-spec-rs.patch"
Remove-Item "$env:TEMP\p2a-t2-spec-rs.patch"
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
fn without_siblings_a_bare_name_is_at_the_root() {
    let spec = SPEC.replace("siblings = true", "siblings = false");
    let text = "### Task 1: a\n\n**Files:**\n- Modify: `src/x/a.rs`, `Cargo.toml`\n";
    let parsed = parse_plan(&registry(&spec), "t.md", text.as_bytes(), Some("t"))
        .unwrap_or_else(|d| panic!("{:?}", d.error_codes()));
    let paths: Vec<String> = parsed.plan.tasks[0]
        .manifest
        .entries
        .iter()
        .map(|e| e.path.to_string())
        .collect();
    assert_eq!(paths, ["src/x/a.rs", "Cargo.toml"]);
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
# a `:3-10` piece adds lines to the path before it (D32)
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

Phase 1 implements the kinds the `assay` spec uses; Phase 2 the rest. Parameters in *italics* are
optional.

| Kind | Matches | Produces | Parameters |
|---|---|---|---|
| `file-block` | a lead paragraph naming a path (and op), then one fence | `WriteFile` / `ApplyPatch` | `lead` (captures `path`, *`op`*), `ops` (op prefix → `write` \| `create` \| `overwrite` \| `patch`), *`default_op`* |
| `before-after` | lead naming a path, fence, *separator paragraph*, fence | `ReplaceExact` | `lead` (captures `path`), *`separator`* (e.g. `^After:$`; absent = two fences back to back) |
| `delete` | a sentence or inline `git rm` | `Delete` | `lead` (captures `path`), *`via_git`* |
| `generated` | a "generated, not by hand" sentence | `Generated` | `lead` (captures `path`) |
| `ceremony` | a fence directly after a given action whose body matches | nothing (absorbed) | `after` (`patch`), `body` |
| `script` | a fence in a shell language | `RunScript` / `Commit` | *`intents`* (`[{ intent, body }]`: `commit` \| `gate` \| `preview` \| `apply` \| `verify`) |
| `run-line` | `` Run: `cmd` `` in a paragraph | `RunScript` (one line, the format's default shell) | `lead` (captures `cmd`), `shell` |
| `expected-block` | `Expected (…):` paragraph + `text` fence after a script | amends the script | `lead` (captures `spec`), `exit` table, `lines = "summary"` |
| `expected-prose` | `Expected: …` sentence after a script | amends the script | `lead` (captures `text`), `exit` table (`success` \| `failure` \| `any` \| `code:N`) |
| `expected-failure-step` | a step title like "verify it fails" | marks the step's scripts as expected failures | `title` regex |
| `preview-apply` | a preview script followed by its apply | sets intents Preview/Apply | `preview` body regex, `apply` body regex |
| `manual-prose` | a paragraph + optional fence that describe an edit with no exact anchor | `Manual(reason)` | `lead`, `reason` (`prose-edit` \| `output-dependent` \| `human-check`), *`takes_fence`* |

Notes on the kinds:

- `file-block`: the longest `ops` prefix of the captured `op` wins; `write` takes Create or
  Overwrite from the manifest. Once the lead matched, an op no prefix covers is
  `Malformed(unknown-file-op)`, a lead without its fence `Malformed(missing-block)`, and for
  `patch` a patch whose paths differ from the lead's `Malformed(patch-path)`.
- `script`: `intent = "commit"` selects the `Commit` action kind, not a `ScriptIntent`; a commit
  script carries `ScriptIntent::Other`. The other intents set the `ScriptIntent`; no match is
  `verify`.
- `expected-block` and `expected-prose` amend the script directly before them. Anything else there
  is `Malformed(expected-without-script)`; a second expectation on one script is
  `Malformed(expected-twice)`. `expected-block` needs a `text` fence (`missing-block`).
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

Targeted: the manifest schema tests:

```powershell
cargo test -p planrun-parse --test manifest_schema
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
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-cli/src/view.rs crates/planrun-model/src/lib.rs crates/planrun-model/src/path.rs crates/planrun-model/src/plan.rs crates/planrun-parse/src/check.rs crates/planrun-parse/src/format.rs crates/planrun-parse/src/lib.rs crates/planrun-parse/src/outline.rs crates/planrun-parse/src/spec.rs crates/planrun-parse/tests/manifest_schema.rs docs/extending.md
  $verified = [ordered]@{
    'crates/planrun-cli/src/view.rs' = '556ecb82cbe291e97a4d4931b27994d3cb883a98'
    'crates/planrun-model/src/lib.rs' = '02cd0d02fd6d7791328d383aaf616ec2eef0b36b'
    'crates/planrun-model/src/path.rs' = '175ecca899c921ffec77d6f84f619b302f68b0f1'
    'crates/planrun-model/src/plan.rs' = '1cdeaacd4032c1dc39d325179512b08c2e2e9f5f'
    'crates/planrun-parse/src/check.rs' = '2375dbaa4c73be7f0950cf6cefdff89b8edfd31a'
    'crates/planrun-parse/src/format.rs' = 'b06b5a4c6682f8828aef299300e82f7ff4fd0f08'
    'crates/planrun-parse/src/lib.rs' = 'a6fb37525790d7a6467689b2ca78078a9738ee7d'
    'crates/planrun-parse/src/outline.rs' = '8c5fddec47d28ed58ed0b6f0166f09ab4b081fea'
    'crates/planrun-parse/src/spec.rs' = '10df33f3adcf94ebda73be1d076b1ac0586ed811'
    'crates/planrun-parse/tests/manifest_schema.rs' = '91bcbc9bdbf725f8b47b5dc09b786f8a3fadd980'
    'docs/extending.md' = '04697cc532de12401572627db5bd3b8236636055'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P2a-T2: manifest schema, Generated entries, op-manifest agreement, [checks]" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P2a-T3: script guard and intents by lead, step marks, expectations that join a Manual

**Goal:** `script` gains `leads`, `first_block` (D28) and intent rows by `lead` (D29); `[[step_marks]]` replace the `expected-failure-step` kind (D30); an `Expected:` after a Manual joins it (D38). The cursor shows the block before it; the step context says whether the step is written. docs/extending.md drops `expected-failure-step` and `preview-apply`.

**Files**

- Modify `crates/planrun-parse/src/format.rs`
- Modify `crates/planrun-parse/src/kinds/expected.rs`
- Modify `crates/planrun-parse/src/kinds/params.rs`
- Modify `crates/planrun-parse/src/kinds/script.rs`
- Modify `crates/planrun-parse/src/outline.rs`
- Modify `crates/planrun-parse/src/recognize.rs`
- Modify `crates/planrun-parse/src/recognizer.rs`
- Modify `crates/planrun-parse/src/spec.rs`
- Create `crates/planrun-parse/tests/guards_and_marks.rs`
- Modify `docs/extending.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-parse/src/outline.rs`, `crates/planrun-parse/src/spec.rs`.

**Step 1 — write the files**

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

File `crates/planrun-parse/src/kinds/expected.rs` (full contents):

```rust
//! `expected-block` (`Expected (…):` + a `text` fence of summary lines) and `expected-prose`
//! (`Expected: …`): both amend the script directly before them.

use std::sync::Arc;

use planrun_model::{Action, ActionKind, ExitExpect, Expectation, RecognizerId, Span, TextRef};
use regex::Regex;

use super::params::{ExitRow, Params, exit_rows, first_exit};
use super::{amendable, capture_ref, paragraph};
use crate::diagnostic::Diagnostic;
use crate::document::Block;
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
        let lead = p.regex("lead", &["text"])?;
        let exit = exit_rows(&mut p, "text")?;
        p.finish()?;
        Ok(Arc::new(ExpectedProse { id, lead, exit }))
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
        if let Some(manual) = joined_manual(cx, lead.span) {
            cursor.advance(1);
            return Recognized::Amend(manual);
        }
        let script = match amendable(cx, lead.span) {
            Ok(script) => script,
            Err(d) => return Recognized::Malformed(d),
        };
        let said = caps.name("text").map_or("", |m| m.as_str());
        let expect = Expectation {
            exit: first_exit(&self.exit, said).unwrap_or(ExitExpect::Unknown),
            lines: None,
            prose: capture_ref(&caps, "text", base).or(Some(TextRef::Span(lead.span))),
        };
        cursor.advance(1);
        Recognized::Amend(amended(script, expect, lead.span))
    }
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

File `crates/planrun-parse/src/kinds/params.rs` (full contents):

```rust
//! Reading a `[[recognizer]]` table: every key used exactly as the kind defines it, unknown keys
//! refused, every regex compiled with its required captures (when the spec loads).

use planrun_model::{ExitExpect, RecognizerId};
use regex::Regex;

use crate::spec::compile_regex;

/// A recognizer's parameters, with the keys read so far.
pub(super) struct Params<'a> {
    table: &'a toml::Table,
    read: Vec<&'static str>,
}

impl<'a> Params<'a> {
    pub(super) fn new(table: &'a toml::Table) -> Self {
        Self {
            table,
            read: Vec::new(),
        }
    }

    fn get(&mut self, key: &'static str) -> Option<&'a toml::Value> {
        self.read.push(key);
        self.table.get(key)
    }

    /// The required `id`.
    pub(super) fn id(&mut self) -> Result<RecognizerId, String> {
        Ok(RecognizerId(self.string("id")?.into()))
    }

    pub(super) fn string(&mut self, key: &'static str) -> Result<&'a str, String> {
        self.opt_string(key)?
            .ok_or_else(|| format!("`{key}` is required"))
    }

    pub(super) fn opt_string(&mut self, key: &'static str) -> Result<Option<&'a str>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => v
                .as_str()
                .map(Some)
                .ok_or_else(|| format!("`{key}` must be a string")),
        }
    }

    /// An optional array of strings.
    pub(super) fn opt_strings(
        &mut self,
        key: &'static str,
    ) -> Result<Option<Vec<&'a str>>, String> {
        let Some(value) = self.get(key) else {
            return Ok(None);
        };
        let wrong = || format!("`{key}` must be an array of strings");
        let list = value.as_array().ok_or_else(wrong)?;
        list.iter()
            .map(|v| v.as_str().ok_or_else(wrong))
            .collect::<Result<Vec<_>, _>>()
            .map(Some)
    }

    pub(super) fn regex(&mut self, key: &'static str, required: &[&str]) -> Result<Regex, String> {
        let pattern = self.string(key)?;
        compile_regex(key, pattern, required)
    }

    pub(super) fn opt_bool(&mut self, key: &'static str) -> Result<Option<bool>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => v
                .as_bool()
                .map(Some)
                .ok_or_else(|| format!("`{key}` must be true or false")),
        }
    }

    pub(super) fn table(&mut self, key: &'static str) -> Result<&'a toml::Table, String> {
        self.get(key)
            .ok_or_else(|| format!("`{key}` is required"))?
            .as_table()
            .ok_or_else(|| format!("`{key}` must be a table"))
    }

    /// An array of tables, each with exactly the keys `keys`.
    pub(super) fn rows(
        &mut self,
        key: &'static str,
        keys: &[&str],
    ) -> Result<Vec<&'a toml::Table>, String> {
        let Some(value) = self.get(key) else {
            return Ok(Vec::new());
        };
        let rows = value
            .as_array()
            .ok_or_else(|| format!("`{key}` must be an array of tables"))?;
        rows.iter()
            .map(|row| {
                let row = row
                    .as_table()
                    .ok_or_else(|| format!("`{key}` must be an array of tables"))?;
                if let Some(extra) = row.keys().find(|k| !keys.contains(&k.as_str())) {
                    return Err(format!("`{key}`: unknown key `{extra}`"));
                }
                Ok(row)
            })
            .collect()
    }

    /// Refuses any key the kind did not read.
    pub(super) fn finish(self) -> Result<(), String> {
        let unknown = self.table.keys().find(|k| !self.read.contains(&k.as_str()));
        match unknown {
            Some(key) => Err(format!("unknown parameter `{key}`")),
            None => Ok(()),
        }
    }
}

/// A string field of a row.
pub(super) fn row_string<'a>(row: &'a toml::Table, key: &str) -> Result<&'a str, String> {
    row.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("a row without a string `{key}`"))
}

/// One row of an exit table: a regex and what its match means.
pub(super) struct ExitRow {
    pub(super) regex: Regex,
    pub(super) exit: ExitValue,
}

/// What an exit row maps to.
#[derive(Debug, Clone, Copy)]
pub(super) enum ExitValue {
    /// A fixed expectation.
    Fixed(ExitExpect),
    /// The number the row's `code` capture holds (0 is `Success`).
    Captured,
}

/// The `exit` rows of an expectation kind, matched on `field` (`spec` or `text`).
pub(super) fn exit_rows(
    params: &mut Params<'_>,
    field: &'static str,
) -> Result<Vec<ExitRow>, String> {
    params
        .rows("exit", &[field, "exit"])?
        .into_iter()
        .map(|row| {
            let exit = exit_value(row_string(row, "exit")?)?;
            let required: &[&str] = match exit {
                ExitValue::Captured => &["code"],
                ExitValue::Fixed(_) => &[],
            };
            let regex = compile_regex(field, row_string(row, field)?, required)?;
            Ok(ExitRow { regex, exit })
        })
        .collect()
}

fn exit_value(value: &str) -> Result<ExitValue, String> {
    match value {
        "success" => Ok(ExitValue::Fixed(ExitExpect::Success)),
        "failure" => Ok(ExitValue::Fixed(ExitExpect::Failure)),
        "any" => Ok(ExitValue::Fixed(ExitExpect::Unknown)),
        "code" => Ok(ExitValue::Captured),
        other => other
            .strip_prefix("code:")
            .and_then(|n| n.parse().ok())
            .map(|n| ExitValue::Fixed(code(n)))
            .ok_or_else(|| format!("exit `{other}` is not success, failure, any, code or code:N")),
    }
}

/// `n` as an expectation: 0 is `Success`.
pub(super) fn code(n: i32) -> ExitExpect {
    if n == 0 {
        ExitExpect::Success
    } else {
        ExitExpect::Code(n)
    }
}

/// The first row that matches `text`, as an expectation.
pub(super) fn first_exit(rows: &[ExitRow], text: &str) -> Option<ExitExpect> {
    rows.iter().find_map(|row| {
        let caps = row.regex.captures(text)?;
        match row.exit {
            ExitValue::Fixed(exit) => Some(exit),
            ExitValue::Captured => caps
                .name("code")
                .and_then(|m| m.as_str().parse().ok())
                .map(code),
        }
    })
}
```

File `crates/planrun-parse/src/kinds/script.rs` (full contents):

```rust
//! `script`: a fence in a shell language → `RunScript`, or `Commit` when its intent says so.
//!
//! With `leads` set, the kind claims a fence only behind a run lead or, with `first_block`, as
//! the first block of a written step; any other shell fence falls to the fallback, Manual
//! (decision D28). An `intents` row matches the body, the lead paragraph, or both (D29).

use std::sync::Arc;

use planrun_model::{Action, ActionKind, RecognizerId, Script, ScriptIntent};
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
        p.finish()?;
        if first_block.is_some() && leads.is_none() {
            return Err("`first_block` needs `leads`".to_owned());
        }
        Ok(Arc::new(ScriptRecognizer {
            id,
            intents,
            leads,
            first_block: first_block.unwrap_or(false),
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
}

impl ScriptRecognizer {
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
            expect: None,
            intent: ScriptIntent::Verify,
        };
        let kind = script_action(script, intent_of(&self.intents, text, lead));
        cursor.advance(1);
        Recognized::Yes(vec![Action {
            kind,
            span: located.span,
            origin: self.id.clone(),
        }])
    }
}
```

File `crates/planrun-parse/src/outline.rs` (exact diff; save the block as `$env:TEMP\p2a-t3-outline-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/outline.rs b/crates/planrun-parse/src/outline.rs
index 8c5fdde..e62f668 100644
--- a/crates/planrun-parse/src/outline.rs
+++ b/crates/planrun-parse/src/outline.rs
@@ -10,7 +10,7 @@ use regex::{Captures, Regex};
 
 use crate::diagnostic::{Diagnostic, Diagnostics};
 use crate::document::{Block, BlockRange, Document};
-use crate::format::{FormatSettings, Outline, OutlineStep, OutlineTask};
+use crate::format::{FormatSettings, Outline, OutlineStep, OutlineTask, StepMark};
 
 /// The compiled `[outline]` of a spec.
 pub struct OutlineRules {
@@ -24,6 +24,8 @@ pub struct OutlineRules {
     pub manifest: Option<ManifestRules>,
     /// Step boundaries.
     pub steps: Vec<StepRule>,
+    /// Step titles that mark the whole step, first match wins (decision D30).
+    pub step_marks: Vec<(Regex, StepMark)>,
 }
 
 /// A heading of one level whose text matches.
@@ -543,6 +545,8 @@ fn steps(
             title: None,
             span: range_span(doc, intro, manifest).unwrap_or(Span::new(0, 0)),
             blocks: intro,
+            written: false,
+            mark: None,
         });
     }
     for (k, &b) in boundaries.iter().enumerate() {
@@ -566,16 +570,38 @@ fn steps(
             }
             StepNumber::Unwritten => None,
         };
+        let mark = step_mark(rules, doc, b, title.as_ref());
         steps.push(OutlineStep {
             number,
             title,
             span: Span::new(start.start, range_end(doc, blocks).unwrap_or(start.end)),
             blocks,
+            written: true,
+            mark,
         });
     }
     steps
 }
 
+/// The mark of the step that block `index` starts: the first `[[step_marks]]` title regex that
+/// matches its title (or, without a title capture, the whole boundary block).
+fn step_mark(
+    rules: &OutlineRules,
+    doc: &Document,
+    index: usize,
+    title: Option<&TextRef>,
+) -> Option<StepMark> {
+    let text = match title {
+        Some(title) => doc.text(title),
+        None => doc.source.slice(doc.blocks[index].span),
+    };
+    rules
+        .step_marks
+        .iter()
+        .find(|(regex, _)| regex.is_match(text))
+        .map(|(_, mark)| *mark)
+}
+
 /// The span from the first to the last block of `range` outside `skip`.
 fn range_span(doc: &Document, range: BlockRange, skip: Option<BlockRange>) -> Option<Span> {
     let mut inside = (range.start..range.end).filter(|i| !skip.is_some_and(|s| s.contains(*i)));
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2a-t3-outline-rs.patch"
Remove-Item "$env:TEMP\p2a-t3-outline-rs.patch"
```

File `crates/planrun-parse/src/recognize.rs` (full contents):

```rust
//! Recognition (docs/architecture.md §4.4): for each step, a cursor walks its blocks and the
//! format's recognizers are tried in order; the first that answers anything but `No` wins. A
//! block no recognizer takes goes to the built-in fallback, so every block is accounted for
//! (decision D21).

use planrun_model::{
    Action, ActionKind, ExitExpect, Expectation, ManualReason, RecognizerId, ScriptIntent, Span,
    Step, Task, TaskId, TextRef,
};

use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::document::{Block, Document};
use crate::format::{Format, Outline, OutlineStep, OutlineTask, StepMark};
use crate::recognizer::{Cursor, Recognized, StepContext};

/// Where one task's blocks went (the coverage report, docs/architecture.md §4.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskCoverage {
    /// The task.
    pub task: TaskId,
    /// Blocks that became actions, expectations included.
    pub actions: usize,
    /// Blocks absorbed without an action (the patch ceremony), with where they are.
    pub absorbed: Vec<Absorbed>,
    /// The manifest's blocks.
    pub manifest: usize,
    /// Blocks that became `Note`.
    pub notes: usize,
    /// Blocks that became `Manual`.
    pub manual: usize,
    /// HTML comments, dropped.
    pub dropped: usize,
}

impl TaskCoverage {
    /// Every block of the task's steps and manifest.
    pub fn blocks(&self) -> usize {
        self.actions + self.absorbed.len() + self.manifest + self.notes + self.manual + self.dropped
    }
}

/// One absorbed block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Absorbed {
    /// The recognizer that absorbed it.
    pub recognizer: RecognizerId,
    /// The block.
    pub span: Span,
}

/// The recognizer id of the built-in fallback.
pub const FALLBACK: &str = "fallback";

/// The tasks of `outline` with their actions, and the coverage of each. Every `Malformed` is
/// collected; any means no tasks.
pub fn recognize(
    format: &dyn Format,
    doc: &Document,
    outline: &Outline,
) -> Result<(Vec<Task>, Vec<TaskCoverage>), Diagnostics> {
    let mut errors = Vec::new();
    let mut tasks = Vec::with_capacity(outline.tasks.len());
    let mut coverage = Vec::with_capacity(outline.tasks.len());
    for task in &outline.tasks {
        let mut cov = TaskCoverage {
            task: task.id.clone(),
            actions: 0,
            absorbed: Vec::new(),
            manifest: task.manifest_blocks.map_or(0, |m| m.len()),
            notes: 0,
            manual: 0,
            dropped: 0,
        };
        let steps = task
            .steps
            .iter()
            .map(|s| step(format, doc, task, s, &mut cov, &mut errors))
            .collect();
        tasks.push(Task {
            id: task.id.clone(),
            title: task.title.clone(),
            span: task.span,
            manifest: task.manifest.clone(),
            steps,
        });
        coverage.push(cov);
    }
    if !errors.is_empty() {
        return Err(Diagnostics(errors));
    }
    Ok((tasks, coverage))
}

fn step(
    format: &dyn Format,
    doc: &Document,
    task: &OutlineTask,
    outline: &OutlineStep,
    cov: &mut TaskCoverage,
    errors: &mut Vec<Diagnostic>,
) -> Step {
    let mut cursor = Cursor::with_skip(doc, outline.blocks, task.manifest_blocks);
    let mut actions: Vec<Action> = Vec::new();
    while let Some(at) = cursor.peek(0).map(|b| b.span) {
        let before = cursor.consumed();
        let cx = StepContext {
            doc,
            task: &task.id,
            manifest: &task.manifest,
            settings: format.settings(),
            previous: &actions,
            written_step: outline.written,
        };
        let (origin, outcome) = first_match(format, &cx, &mut cursor);
        let consumed = cursor.consumed() - before;
        match outcome {
            Recognized::No => {
                if let Some(action) = fallback(&cx, &mut cursor, cov) {
                    actions.push(action);
                }
            }
            Recognized::Malformed(d) => {
                errors.push(d);
                cursor.advance(1);
            }
            _ if consumed == 0 => {
                errors.push(Diagnostic::error(
                    "recognizer-stalled",
                    Some(at),
                    format!("recognizer `{origin}` matched without consuming a block"),
                ));
                cursor.advance(1);
            }
            Recognized::Yes(new) if new.is_empty() => cov.absorbed.push(Absorbed {
                recognizer: RecognizerId(origin.into()),
                span: at,
            }),
            Recognized::Yes(new) => {
                cov.actions += consumed;
                actions.extend(new);
            }
            Recognized::Amend(action) => {
                match action.kind {
                    ActionKind::Manual { .. } => cov.manual += consumed,
                    _ => cov.actions += consumed,
                }
                actions.pop();
                actions.push(action);
            }
        }
    }
    apply_mark(outline.mark, &mut actions, cov, errors);
    Step {
        number: outline.number,
        title: outline.title.clone(),
        span: outline.span,
        actions,
    }
}

/// Applies the mark of a step's title to its actions (decision D30).
fn apply_mark(
    mark: Option<StepMark>,
    actions: &mut [Action],
    cov: &mut TaskCoverage,
    errors: &mut Vec<Diagnostic>,
) {
    match mark {
        None => {}
        Some(StepMark::ExpectedFailure) => {
            errors.extend(actions.iter_mut().filter_map(expect_failure));
        }
        Some(StepMark::HumanCheck) => {
            for action in actions.iter_mut() {
                let ActionKind::Note { text } = &action.kind else {
                    continue;
                };
                action.kind = ActionKind::Manual {
                    reason: ManualReason::HumanCheck,
                    text: text.clone(),
                };
                cov.notes = cov.notes.saturating_sub(1);
                cov.manual += 1;
            }
        }
    }
}

/// An `expected-failure` step's `Verify` script expects failure: none or `any` becomes
/// `Failure`; an expected success is `expected-conflict`. Commits and gates are left alone.
fn expect_failure(action: &mut Action) -> Option<Diagnostic> {
    let span = action.span;
    let ActionKind::RunScript(script) = &mut action.kind else {
        return None;
    };
    if script.intent != ScriptIntent::Verify {
        return None;
    }
    let Some(expect) = &mut script.expect else {
        script.expect = Some(Expectation {
            exit: ExitExpect::Failure,
            lines: None,
            prose: None,
        });
        return None;
    };
    match expect.exit {
        ExitExpect::Unknown => {
            expect.exit = ExitExpect::Failure;
            None
        }
        ExitExpect::Success => Some(Diagnostic::error(
            "expected-conflict",
            Some(span),
            "the step's title says this fails, and its expectation says it succeeds",
        )),
        ExitExpect::Failure | ExitExpect::Code(_) => None,
    }
}

/// The first recognizer that answers anything but `No`, with its id.
fn first_match<'f>(
    format: &'f dyn Format,
    cx: &StepContext<'_>,
    cursor: &mut Cursor<'_>,
) -> (&'f str, Recognized) {
    for recognizer in format.recognizers() {
        let outcome = recognizer.recognize(cx, cursor);
        if !matches!(outcome, Recognized::No) {
            return (recognizer.id(), outcome);
        }
    }
    (FALLBACK, Recognized::No)
}

/// The built-in fallback: consumes one block. Prose becomes `Note` (or `Manual(Marked)`), a
/// fence `Manual`, a `text` fence `Note`; an HTML comment is dropped.
fn fallback(
    cx: &StepContext<'_>,
    cursor: &mut Cursor<'_>,
    cov: &mut TaskCoverage,
) -> Option<Action> {
    let located = cursor.peek(0)?;
    cursor.advance(1);
    let whole = TextRef::Span(located.span);
    let prose = |text: &TextRef| {
        if cx.settings.is_marked(cx.doc.text(text)) {
            ActionKind::Manual {
                reason: ManualReason::Marked,
                text: text.clone(),
            }
        } else {
            ActionKind::Note { text: text.clone() }
        }
    };
    let kind = match &located.block {
        Block::Paragraph { text }
        | Block::ListItem { text, .. }
        | Block::Quote { text }
        | Block::Heading { text, .. } => prose(text),
        Block::Table { text } => ActionKind::Note { text: text.clone() },
        Block::Rule => ActionKind::Note { text: whole },
        Block::Html { text } if cx.doc.text(text).trim_start().starts_with("<!--") => {
            cov.dropped += 1;
            return None;
        }
        Block::Html { text } => ActionKind::Note { text: text.clone() },
        Block::Fence { .. } => fence(cx, &located.block, whole),
    };
    match kind {
        ActionKind::Manual { .. } => cov.manual += 1,
        _ => cov.notes += 1,
    }
    Some(Action {
        kind,
        span: located.span,
        origin: RecognizerId(FALLBACK.into()),
    })
}

fn fence(cx: &StepContext<'_>, block: &Block, whole: TextRef) -> ActionKind {
    let language = block.fence_language(cx.doc).unwrap_or("");
    let reason = match language {
        "text" => return ActionKind::Note { text: whole },
        "" => ManualReason::Unrecognized,
        shell if cx.settings.shell_for(shell).is_some() => ManualReason::Unrecognized,
        other => ManualReason::ForeignLanguage(other.into()),
    };
    ActionKind::Manual {
        reason,
        text: whole,
    }
}
```

File `crates/planrun-parse/src/recognizer.rs` (full contents):

```rust
//! Recognizers: small, composable matchers that turn a run of blocks into actions
//! (docs/architecture.md §4.4). A format is an ordered list of recognizer instances.

use std::sync::Arc;

use planrun_model::{Action, Manifest, TaskId};

use crate::diagnostic::Diagnostic;
use crate::document::{BlockRange, Document, Located};
use crate::format::FormatSettings;

/// What a recognizer may read besides the blocks.
pub struct StepContext<'a> {
    /// The document.
    pub doc: &'a Document,
    /// The task being recognized.
    pub task: &'a TaskId,
    /// Its manifest (a full-contents write consults it for create vs overwrite).
    pub manifest: &'a Manifest,
    /// Format settings.
    pub settings: &'a FormatSettings,
    /// Actions already produced in this step: an expectation attaches to the script before it;
    /// the patch ceremony is absorbed only right after a patch.
    pub previous: &'a [Action],
    /// True in a step that starts at a written boundary; false in the implicit first step (the
    /// blocks before the first boundary, where the manifest is).
    pub written_step: bool,
}

/// A position in a step's blocks. A recognizer that matches advances it past what it consumed.
/// Blocks in the `skip` range (the task's manifest) are invisible to it.
pub struct Cursor<'a> {
    blocks: &'a [Located],
    range: BlockRange,
    skip: Option<BlockRange>,
    position: usize,
    consumed: usize,
}

impl<'a> Cursor<'a> {
    /// A cursor at the start of `range`.
    pub fn new(doc: &'a Document, range: BlockRange) -> Self {
        Self::with_skip(doc, range, None)
    }

    /// A cursor at the start of `range` that never sees the blocks of `skip`.
    pub fn with_skip(doc: &'a Document, range: BlockRange, skip: Option<BlockRange>) -> Self {
        let mut cursor = Self {
            blocks: &doc.blocks,
            range,
            skip,
            position: range.start,
            consumed: 0,
        };
        cursor.position = cursor.settle(range.start);
        cursor
    }

    /// `index`, or the first index after the skipped range when it falls inside it.
    fn settle(&self, index: usize) -> usize {
        match self.skip {
            Some(skip) if skip.contains(index) => skip.end,
            _ => index,
        }
    }

    /// The block `offset` positions ahead, if it is still in the step.
    pub fn peek(&self, offset: usize) -> Option<&'a Located> {
        let mut index = self.position;
        for _ in 0..offset {
            index = self.settle(index + 1);
        }
        (index < self.range.end).then(|| &self.blocks[index])
    }

    /// The block before the cursor, if it is still in the step (skipped blocks are invisible).
    pub fn previous(&self) -> Option<&'a Located> {
        let mut index = self.position.checked_sub(1)?;
        if let Some(skip) = self.skip.filter(|s| s.contains(index)) {
            index = skip.start.checked_sub(1)?;
        }
        (index >= self.range.start).then(|| &self.blocks[index])
    }

    /// True when the cursor is at the step's first block.
    pub fn at_start(&self) -> bool {
        self.position == self.settle(self.range.start)
    }

    /// Consumes `count` blocks.
    pub fn advance(&mut self, count: usize) {
        for _ in 0..count {
            if self.is_done() {
                return;
            }
            self.position = self.settle(self.position + 1);
            self.consumed += 1;
        }
    }

    /// True when every block of the step is consumed.
    pub fn is_done(&self) -> bool {
        self.position >= self.range.end
    }

    /// The current position (for coverage and diagnostics).
    pub fn position(&self) -> usize {
        self.position
    }

    /// How many blocks have been consumed so far.
    pub fn consumed(&self) -> usize {
        self.consumed
    }
}

/// The outcome of one recognizer at one position.
pub enum Recognized {
    /// Not this recognizer's idiom; the cursor must be unchanged.
    No,
    /// Matched: the cursor is advanced; these actions are appended (may be empty, e.g. the
    /// absorbed patch ceremony).
    Yes(Vec<Action>),
    /// The idiom matched but is malformed (lead sentence without its fence, a patch whose
    /// paths disagree with the lead): a parse error, never a fallback to `Manual`.
    Malformed(Diagnostic),
    /// Matched an earlier action instead of producing one (an `Expected` block): replace the
    /// last action with this one.
    Amend(Action),
}

/// One configured matcher. Must be pure: no I/O, no state between calls.
pub trait Recognizer: Send + Sync {
    /// Instance id for coverage and `planrun show` (`file-block#full-contents`).
    fn id(&self) -> &str;

    /// Tries to match at the cursor.
    fn recognize(&self, cx: &StepContext<'_>, cursor: &mut Cursor<'_>) -> Recognized;
}

/// Builds recognizer instances of one kind from the `[[recognizer]]` tables of a format spec.
/// Registered by kind name in the [`crate::Registry`]; this is the code-level plugin point.
pub trait RecognizerFactory: Send + Sync {
    /// The kind name used in specs (`file-block`).
    fn kind(&self) -> &'static str;

    /// One instance from its spec table (`params` excludes `kind`). Invalid params are an
    /// error when the registry loads the format, never at parse time.
    fn build(&self, params: &toml::Table) -> Result<Arc<dyn Recognizer>, String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source;

    #[test]
    fn the_cursor_never_sees_the_skipped_blocks() {
        let text = "a\n\nb\n\nc\n\nd\n";
        let source = source::load("t", text.as_bytes()).unwrap_or_else(|e| panic!("{e:?}"));
        let doc = Document::build(source).unwrap_or_else(|e| panic!("{e:?}"));
        let all = BlockRange { start: 0, end: 4 };
        let skip = Some(BlockRange { start: 1, end: 3 });
        let mut cursor = Cursor::with_skip(&doc, all, skip);
        let at = |c: &Cursor<'_>, k| c.peek(k).map(|b| doc.source.slice(b.span).to_owned());
        assert_eq!(at(&cursor, 0).as_deref(), Some("a"));
        assert_eq!(at(&cursor, 1).as_deref(), Some("d"));
        assert_eq!(at(&cursor, 2), None);
        cursor.advance(1);
        assert_eq!(cursor.position(), 3);
        cursor.advance(5);
        assert!(cursor.is_done());
        assert_eq!(cursor.consumed(), 2);

        let from_skip = Cursor::with_skip(&doc, BlockRange { start: 1, end: 4 }, skip);
        assert_eq!(from_skip.position(), 3);
    }
}
```

File `crates/planrun-parse/src/spec.rs` (exact diff; save the block as `$env:TEMP\p2a-t3-spec-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/spec.rs b/crates/planrun-parse/src/spec.rs
index 10df33f..3b80b92 100644
--- a/crates/planrun-parse/src/spec.rs
+++ b/crates/planrun-parse/src/spec.rs
@@ -12,7 +12,7 @@ use serde::Deserialize;
 
 use crate::diagnostic::Diagnostics;
 use crate::document::Document;
-use crate::format::{CheckLevel, DOWNGRADABLE, Format, FormatSettings, Outline};
+use crate::format::{CheckLevel, DOWNGRADABLE, Format, FormatSettings, Outline, StepMark};
 use crate::outline::{self, HeadingRule, ManifestRules, OutlineRules, StepBlock, StepRule};
 use crate::recognizer::{Recognizer, RecognizerFactory};
 
@@ -37,6 +37,9 @@ pub struct FormatSpec {
     /// appear.
     #[serde(default)]
     pub checks: std::collections::BTreeMap<String, String>,
+    /// Step titles that mark a whole step (decision D30).
+    #[serde(default)]
+    pub step_marks: Vec<StepMarkSpec>,
     /// Recognizer instances, in order; each table has `kind` plus the kind's parameters.
     #[serde(default, rename = "recognizer")]
     pub recognizers: Vec<toml::Table>,
@@ -109,6 +112,16 @@ pub struct ManifestSpec {
     pub siblings: Option<bool>,
 }
 
+/// A `[[step_marks]]` row.
+#[derive(Debug, Clone, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct StepMarkSpec {
+    /// Regex over the step's title.
+    pub title: String,
+    /// `expected-failure` or `human-check`.
+    pub mark: String,
+}
+
 /// A block that starts a step.
 #[derive(Debug, Clone, Deserialize)]
 #[serde(deny_unknown_fields)]
@@ -166,7 +179,7 @@ impl SpecFormat {
             .iter()
             .map(|p| Ok((compile_regex("detect", &p.regex, &[])?, p.weight)))
             .collect::<Result<_, String>>()?;
-        let outline = compile_outline(&spec.outline)?;
+        let outline = compile_outline(&spec.outline, &spec.step_marks)?;
         let mut settings = compile_settings(&spec.settings)?;
         settings.checks = compile_checks(&spec.checks)?;
         let recognizers = spec
@@ -219,7 +232,22 @@ impl Format for SpecFormat {
     }
 }
 
-fn compile_outline(spec: &OutlineSpec) -> Result<OutlineRules, String> {
+fn compile_outline(spec: &OutlineSpec, marks: &[StepMarkSpec]) -> Result<OutlineRules, String> {
+    let step_marks = marks
+        .iter()
+        .map(|m| {
+            let mark = match m.mark.as_str() {
+                "expected-failure" => StepMark::ExpectedFailure,
+                "human-check" => StepMark::HumanCheck,
+                other => {
+                    return Err(format!(
+                        "step_marks: `{other}` is not expected-failure or human-check"
+                    ));
+                }
+            };
+            Ok((compile_regex("step_marks.title", &m.title, &[])?, mark))
+        })
+        .collect::<Result<_, String>>()?;
     let tasks_after = spec
         .tasks_after
         .as_ref()
@@ -243,6 +271,7 @@ fn compile_outline(spec: &OutlineSpec) -> Result<OutlineRules, String> {
         tasks_end_level: spec.tasks_end_level,
         manifest,
         steps,
+        step_marks,
     })
 }
 
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2a-t3-spec-rs.patch"
Remove-Item "$env:TEMP\p2a-t3-spec-rs.patch"
```

File `crates/planrun-parse/tests/guards_and_marks.rs` (full contents):

```rust
//! The script guard and intents by lead (decisions D28, D29), step marks (D30), and an
//! expectation that joins a Manual (D38), through a small list-step spec.

use planrun_model::{ActionKind, ManualReason, ScriptIntent};
use planrun_parse::{Registry, parse_plan};

const SPEC: &str = r#"
id = "g"
name = "guard test"
[detect]
patterns = [{ regex = '^## Task', weight = 60 }]
[outline]
task = { level = 2, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
tasks_end_level = 2
[outline.manifest]
marker = '^\*\*Files:\*\*$'
item = '^(?P<change>Create|Modify): `(?P<paths>[^`]+)`$'
changes = { Create = "create", Modify = "modify" }
required = false
[[outline.steps]]
block = "list-item"
regex = '^\*\*Step (?P<n>\d+): (?P<title>.+)\*\*$'
[[step_marks]]
title = '(?i)\bverify (?:it|they) fails?\b'
mark = "expected-failure"
[[step_marks]]
title = '(?i)\bby eye\b'
mark = "human-check"
[checks]
manifest-untouched = "off"
[[recognizer]]
kind = "script"
id = "script#g"
leads = ['^Run:$', '^Preview:$', '^Apply:$']
first_block = true
intents = [
  { intent = "commit", body = '(?m)\bgit commit\b' },
  { intent = "preview", lead = '^Preview:$' },
  { intent = "apply", lead = '^Apply:$' },
]
[[recognizer]]
kind = "expected-prose"
id = "expected-prose#g"
lead = '^Expected: (?P<text>.+)$'
exit = [
  { text = '(?i)\bfail', exit = "failure" },
  { text = '(?i)\bpass', exit = "success" },
]
"#;

fn parse(text: &str) -> Result<planrun_parse::Parsed, Vec<&'static str>> {
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    registry
        .add_spec("g", SPEC)
        .unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, "g.md", text.as_bytes(), Some("g")).map_err(|d| d.error_codes())
}

/// One label per action, per step.
fn labels(text: &str) -> Vec<Vec<String>> {
    let parsed = parse(text).unwrap_or_else(|codes| panic!("{codes:?}"));
    parsed.plan.tasks[0]
        .steps
        .iter()
        .map(|s| s.actions.iter().map(|a| label(&a.kind)).collect())
        .collect()
}

fn label(kind: &ActionKind) -> String {
    let exit = |s: &planrun_model::Script| s.expect.as_ref().map(|e| e.exit);
    match kind {
        ActionKind::RunScript(s) => format!("run {:?} {:?}", s.intent, exit(s)),
        ActionKind::Commit(s) => format!("commit {:?}", exit(s)),
        ActionKind::Manual { reason, .. } => format!("manual {reason:?}"),
        ActionKind::Note { .. } => "note".to_owned(),
        other => format!("{other:?}"),
    }
}

const PWSH: &str = "```powershell\ncargo test\n```";

#[test]
fn a_shell_fence_runs_only_behind_a_lead_or_first_in_a_written_step() {
    let text = format!(
        "## Task 1: a\n\n**Files:**\n- Create: `a.ps1`\n\n{PWSH}\n\n\
         - [ ] **Step 1: build**\n\n{PWSH}\n\nThen this:\n\n{PWSH}\n\nRun:\n\n{PWSH}\n\n\
         Add it to the script:\n\n```powershell\ngit commit -m x\n```\n"
    );
    assert_eq!(
        labels(&text),
        [
            vec!["manual Unrecognized"],
            vec![
                "run Verify None",
                "note",
                "manual Unrecognized",
                "note",
                "run Verify None",
                "note",
                "manual Unrecognized",
            ],
        ]
    );
}

#[test]
fn a_claimed_fence_takes_its_intent_from_its_lead() {
    let text = format!(
        "## Task 1: a\n\n- [ ] **Step 1: rename**\n\nPreview:\n\n{PWSH}\n\nApply:\n\n{PWSH}\n"
    );
    let steps = labels(&text);
    assert_eq!(
        steps[0],
        ["note", "run Preview None", "note", "run Apply None"]
    );
    let parsed = parse(&text).unwrap_or_else(|codes| panic!("{codes:?}"));
    let intents: Vec<ScriptIntent> = parsed.plan.tasks[0].steps[0]
        .actions
        .iter()
        .filter_map(|a| match &a.kind {
            ActionKind::RunScript(s) => Some(s.intent),
            _ => None,
        })
        .collect();
    assert_eq!(intents, [ScriptIntent::Preview, ScriptIntent::Apply]);
}

#[test]
fn an_expected_failure_step_expects_its_verify_scripts_to_fail() {
    let text = format!(
        "## Task 1: a\n\n- [ ] **Step 1: Run the test to verify it fails**\n\n{PWSH}\n\n\
         Run:\n\n{PWSH}\n\nExpected: some output\n\n\
         Run:\n\n```powershell\ngit commit -m x\n```\n"
    );
    assert_eq!(
        labels(&text)[0],
        [
            "run Verify Some(Failure)",
            "note",
            "run Verify Some(Failure)",
            "note",
            "commit None",
        ]
    );
    let conflict = format!(
        "## Task 1: a\n\n- [ ] **Step 1: Run the test to verify it fails**\n\n{PWSH}\n\n\
         Expected: all pass\n"
    );
    assert_eq!(parse(&conflict).err(), Some(vec!["expected-conflict"]));
}

#[test]
fn a_human_check_step_turns_its_notes_into_manual() {
    let text = "## Task 1: a\n\n- [ ] **Step 1: Check by eye**\n\nOpen the page and look.\n";
    assert_eq!(labels(text)[0], ["manual HumanCheck"]);
    let parsed = parse(text).unwrap_or_else(|codes| panic!("{codes:?}"));
    assert_eq!(parsed.coverage[0].manual, 1);
    assert_eq!(parsed.coverage[0].notes, 0);
}

#[test]
fn an_expectation_after_a_manual_joins_it() {
    let text = "## Task 1: a\n\n- [ ] **Step 1: record**\n\nOn the test database:\n\n\
                ```sql\nINSERT INTO t VALUES (1);\n```\n\nExpected: 1 row affected.\n";
    let parsed = parse(text).unwrap_or_else(|codes| panic!("{codes:?}"));
    let actions = &parsed.plan.tasks[0].steps[0].actions;
    assert_eq!(actions.len(), 2);
    let ActionKind::Manual { reason, text } = &actions[1].kind else {
        panic!("{:?}", actions[1].kind);
    };
    assert_eq!(*reason, ManualReason::ForeignLanguage("sql".into()));
    assert!(
        parsed
            .plan
            .text(text)
            .ends_with("Expected: 1 row affected.")
    );
    assert_eq!(parsed.coverage[0].manual, 2);

    let after_note = "## Task 1: a\n\n- [ ] **Step 1: x**\n\nSome text.\n\nExpected: ok\n";
    assert_eq!(
        parse(after_note).err(),
        Some(vec!["expected-without-script"])
    );
}

#[test]
fn guard_and_intent_parameters_are_validated_when_the_spec_loads() {
    for (from, to) in [
        ("leads = ['^Run:$', '^Preview:$', '^Apply:$']\n", ""),
        (
            "{ intent = \"apply\", lead = '^Apply:$' }",
            "{ intent = \"apply\" }",
        ),
        ("mark = \"human-check\"", "mark = \"by-eye\""),
    ] {
        let spec = SPEC.replace(from, to);
        assert_ne!(spec, SPEC, "{from}");
        let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
        assert!(registry.add_spec("g", &spec).is_err(), "{to:?} loads");
    }
}

#[test]
fn exit_expect_is_unchanged_without_a_mark() {
    let text = format!("## Task 1: a\n\n- [ ] **Step 1: build**\n\n{PWSH}\n\nExpected: all pass\n");
    assert_eq!(labels(&text)[0], ["run Verify Some(Success)"]);
}
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
# a `:3-10` piece adds lines to the path before it (D32)
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
# expected success is the error expected-conflict). human-check: its Notes become
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

Phase 1 implements the kinds the `assay` spec uses; Phase 2 the rest. Parameters in *italics* are
optional.

| Kind | Matches | Produces | Parameters |
|---|---|---|---|
| `file-block` | a lead paragraph naming a path (and op), then one fence | `WriteFile` / `ApplyPatch` | `lead` (captures `path`, *`op`*), `ops` (op prefix → `write` \| `create` \| `overwrite` \| `patch`), *`default_op`* |
| `before-after` | lead naming a path, fence, *separator paragraph*, fence | `ReplaceExact` | `lead` (captures `path`), *`separator`* (e.g. `^After:$`; absent = two fences back to back) |
| `delete` | a sentence or inline `git rm` | `Delete` | `lead` (captures `path`), *`via_git`* |
| `generated` | a "generated, not by hand" sentence | `Generated` | `lead` (captures `path`) |
| `ceremony` | a fence directly after a given action whose body matches | nothing (absorbed) | `after` (`patch`), `body` |
| `script` | a fence in a shell language | `RunScript` / `Commit` | *`intents`* (`[{ intent, body?, lead? }]`: `commit` \| `gate` \| `preview` \| `apply` \| `verify` \| `other`), *`leads`* (regexes), *`first_block`* |
| `run-line` | `` Run: `cmd` `` in a paragraph | `RunScript` (one line, the format's default shell) | `lead` (captures `cmd`), `shell` |
| `expected-block` | `Expected (…):` paragraph + `text` fence after a script | amends the script | `lead` (captures `spec`), `exit` table, `lines = "summary"` |
| `expected-prose` | `Expected: …` sentence after a script | amends the script | `lead` (captures `text`), `exit` table (`success` \| `failure` \| `any` \| `code:N`) |
| `manual-prose` | a paragraph + optional fence that describe an edit with no exact anchor | `Manual(reason)` | `lead`, `reason` (`prose-edit` \| `output-dependent` \| `human-check`), *`takes_fence`* |

Notes on the kinds:

- `file-block`: the longest `ops` prefix of the captured `op` wins; `write` takes Create or
  Overwrite from the manifest. Once the lead matched, an op no prefix covers is
  `Malformed(unknown-file-op)`, a lead without its fence `Malformed(missing-block)`, and for
  `patch` a patch whose paths differ from the lead's `Malformed(patch-path)`.
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

Targeted: the guard and step-mark tests:

```powershell
cargo test -p planrun-parse --test guards_and_marks
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
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-parse/src/format.rs crates/planrun-parse/src/kinds/expected.rs crates/planrun-parse/src/kinds/params.rs crates/planrun-parse/src/kinds/script.rs crates/planrun-parse/src/outline.rs crates/planrun-parse/src/recognize.rs crates/planrun-parse/src/recognizer.rs crates/planrun-parse/src/spec.rs crates/planrun-parse/tests/guards_and_marks.rs docs/extending.md
  $verified = [ordered]@{
    'crates/planrun-parse/src/format.rs' = 'd331e1ab56d6018504781e05bbc55906cc3234c2'
    'crates/planrun-parse/src/kinds/expected.rs' = '993d22a3fe8d552ff01043fb859eddf7ac346d4e'
    'crates/planrun-parse/src/kinds/params.rs' = 'ed69b6fcb92bf2819bba1f5b31cede014fea4c74'
    'crates/planrun-parse/src/kinds/script.rs' = '51ca1cb7aaff7251f4c6d71b1969499e138e8e08'
    'crates/planrun-parse/src/outline.rs' = 'e62f66886c36233eaa895072037f317ef6f59b20'
    'crates/planrun-parse/src/recognize.rs' = '7146480f53740be732f65f4464151679e70e22c0'
    'crates/planrun-parse/src/recognizer.rs' = 'a1f859d37b2499ffb570d3b4f064a522c98f7a38'
    'crates/planrun-parse/src/spec.rs' = '3b80b926ae93a838b01a4c42da29983d8550293e'
    'crates/planrun-parse/tests/guards_and_marks.rs' = 'f61adec32efea72e198fc3077742f2a594f55b30'
    'docs/extending.md' = 'f08b2db3686f292784946d5ce9f320ea17e1bca5'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P2a-T3: script guard and intents by lead, step marks, expectations that join a Manual" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P2a-T4: run-line, before-after and manual-prose; paths resolved from the manifest

**Goal:** The kinds `run-line` (D39), `before-after` and `manual-prose`; `file_name_from_manifest` on `file-block`, `before-after` and `delete`, and `infer_path` on `before-after` (D32); `Action::path_origin`, which `show` prints as "resolved from manifest". Coverage counts a recognizer's Manual as manual.

**Files**

- Modify `crates/planrun-cli/src/view.rs`
- Modify `crates/planrun-model/src/action.rs`
- Create `crates/planrun-parse/src/kinds/before_after.rs`
- Modify `crates/planrun-parse/src/kinds/delete.rs`
- Modify `crates/planrun-parse/src/kinds/file_block.rs`
- Modify `crates/planrun-parse/src/kinds/generated.rs`
- Create `crates/planrun-parse/src/kinds/manual_prose.rs`
- Modify `crates/planrun-parse/src/kinds/mod.rs`
- Create `crates/planrun-parse/src/kinds/run_line.rs`
- Modify `crates/planrun-parse/src/kinds/script.rs`
- Modify `crates/planrun-parse/src/recognize.rs`
- Create `crates/planrun-parse/tests/edit_kinds.rs`
- Modify `docs/extending.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/src/view.rs`.

**Step 1 — write the files**

File `crates/planrun-cli/src/view.rs` (exact diff; save the block as `$env:TEMP\p2a-t4-view-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/view.rs b/crates/planrun-cli/src/view.rs
index 556ecb8..1db1d53 100644
--- a/crates/planrun-cli/src/view.rs
+++ b/crates/planrun-cli/src/view.rs
@@ -2,8 +2,8 @@
 //! a line (`L37`). `planrun show` prints it; the IR snapshot is its YAML.
 
 use planrun_model::{
-    Action, ActionKind, ExitExpect, Expectation, ManifestEntry, Plan, Script, Span, Step, Task,
-    TextRef,
+    Action, ActionKind, ExitExpect, Expectation, ManifestEntry, PathOrigin, Plan, Script, Span,
+    Step, Task, TextRef,
 };
 use serde::Serialize;
 
@@ -59,6 +59,9 @@ pub struct ActionView {
     pub at: String,
     /// The recognizer that produced it.
     pub by: String,
+    /// How the path was found when the plan did not write it in full (decision D32).
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub path_from: Option<String>,
     /// What it does.
     #[serde(flatten)]
     pub what: What,
@@ -215,6 +218,7 @@ impl<'a> Viewer<'a> {
         ActionView {
             at: self.at(action.span),
             by: action.origin.0.to_string(),
+            path_from: path_from(action.path_origin),
             what: self.what(&action.kind),
         }
     }
@@ -290,6 +294,17 @@ impl<'a> Viewer<'a> {
     }
 }
 
+/// How a resolved path was found; `None` when the plan wrote it.
+fn path_from(origin: PathOrigin) -> Option<String> {
+    let how = match origin {
+        PathOrigin::Written => return None,
+        PathOrigin::Sibling => "the directory of the manifest path before it",
+        PathOrigin::FileName => "the manifest entry with this file name",
+        PathOrigin::OnlyEntry => "the task's only editable manifest entry",
+    };
+    Some(format!("resolved from manifest: {how}"))
+}
+
 fn entry(entry: &ManifestEntry) -> String {
     let lines = entry
         .lines
@@ -337,6 +352,9 @@ fn action_text(action: &ActionView, detail: Detail, out: &mut String) {
         "    {:<6} {label:<11} {summary}  ({})\n",
         action.at, action.by
     ));
+    if let Some(from) = &action.path_from {
+        out.push_str(&format!("           path {from}\n"));
+    }
     if let What::Run(s) | What::Commit(s) = &action.what
         && let Some(e) = &s.expect
     {
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2a-t4-view-rs.patch"
Remove-Item "$env:TEMP\p2a-t4-view-rs.patch"
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

File `crates/planrun-parse/src/kinds/before_after.rs` (full contents):

```rust
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
```

File `crates/planrun-parse/src/kinds/delete.rs` (full contents):

```rust
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
```

File `crates/planrun-parse/src/kinds/generated.rs` (full contents):

```rust
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
```

File `crates/planrun-parse/src/kinds/manual_prose.rs` (full contents):

```rust
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
    let unresolved =
        !by_file_name || path.as_str().contains('/') || entries.iter().any(|e| e.path == path);
    if unresolved {
        return Ok((path, PathOrigin::Written));
    }
    let same: Vec<&ManifestEntry> = entries
        .iter()
        .filter(|e| e.path.file_name() == path.as_str())
        .collect();
    let first = same.first().map(|e| &e.path);
    let one = first.filter(|p| same.iter().all(|e| &e.path == *p && e.origin.is_written()));
    match one {
        Some(entry) => Ok((entry.clone(), PathOrigin::FileName)),
        None => Ok((path, PathOrigin::Written)),
    }
}

/// The task's one manifest entry an edit with no path can mean (decision D32): not Test,
/// Generated or Delete. `None` with none or several.
fn only_entry(cx: &StepContext<'_>) -> Option<RepoPath> {
    let mut editable = cx.manifest.entries.iter().filter(|e| {
        !matches!(
            e.change,
            ChangeKind::Test | ChangeKind::Generated | ChangeKind::Delete
        )
    });
    let first = editable.next()?;
    editable
        .all(|e| e.path == first.path)
        .then(|| first.path.clone())
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
//! (decision D28). An `intents` row matches the body, the lead paragraph, or both (D29).

use std::sync::Arc;

use planrun_model::{Action, ActionKind, RecognizerId, Script, ScriptIntent};
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
        p.finish()?;
        if first_block.is_some() && leads.is_none() {
            return Err("`first_block` needs `leads`".to_owned());
        }
        Ok(Arc::new(ScriptRecognizer {
            id,
            intents,
            leads,
            first_block: first_block.unwrap_or(false),
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
}

impl ScriptRecognizer {
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
            expect: None,
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

File `crates/planrun-parse/src/recognize.rs` (full contents):

```rust
//! Recognition (docs/architecture.md §4.4): for each step, a cursor walks its blocks and the
//! format's recognizers are tried in order; the first that answers anything but `No` wins. A
//! block no recognizer takes goes to the built-in fallback, so every block is accounted for
//! (decision D21).

use planrun_model::{
    Action, ActionKind, ExitExpect, Expectation, ManualReason, RecognizerId, ScriptIntent, Span,
    Step, Task, TaskId, TextRef,
};

use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::document::{Block, Document};
use crate::format::{Format, Outline, OutlineStep, OutlineTask, StepMark};
use crate::recognizer::{Cursor, Recognized, StepContext};

/// Where one task's blocks went (the coverage report, docs/architecture.md §4.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskCoverage {
    /// The task.
    pub task: TaskId,
    /// Blocks that became actions other than `Manual`, expectations included.
    pub actions: usize,
    /// Blocks absorbed without an action (the patch ceremony), with where they are.
    pub absorbed: Vec<Absorbed>,
    /// The manifest's blocks.
    pub manifest: usize,
    /// Blocks that became `Note`.
    pub notes: usize,
    /// Blocks that became `Manual`, through the fallback or a recognizer (`manual-prose`).
    pub manual: usize,
    /// HTML comments, dropped.
    pub dropped: usize,
}

impl TaskCoverage {
    /// Every block of the task's steps and manifest.
    pub fn blocks(&self) -> usize {
        self.actions + self.absorbed.len() + self.manifest + self.notes + self.manual + self.dropped
    }
}

/// One absorbed block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Absorbed {
    /// The recognizer that absorbed it.
    pub recognizer: RecognizerId,
    /// The block.
    pub span: Span,
}

/// The recognizer id of the built-in fallback.
pub const FALLBACK: &str = "fallback";

/// The tasks of `outline` with their actions, and the coverage of each. Every `Malformed` is
/// collected; any means no tasks.
pub fn recognize(
    format: &dyn Format,
    doc: &Document,
    outline: &Outline,
) -> Result<(Vec<Task>, Vec<TaskCoverage>), Diagnostics> {
    let mut errors = Vec::new();
    let mut tasks = Vec::with_capacity(outline.tasks.len());
    let mut coverage = Vec::with_capacity(outline.tasks.len());
    for task in &outline.tasks {
        let mut cov = TaskCoverage {
            task: task.id.clone(),
            actions: 0,
            absorbed: Vec::new(),
            manifest: task.manifest_blocks.map_or(0, |m| m.len()),
            notes: 0,
            manual: 0,
            dropped: 0,
        };
        let steps = task
            .steps
            .iter()
            .map(|s| step(format, doc, task, s, &mut cov, &mut errors))
            .collect();
        tasks.push(Task {
            id: task.id.clone(),
            title: task.title.clone(),
            span: task.span,
            manifest: task.manifest.clone(),
            steps,
        });
        coverage.push(cov);
    }
    if !errors.is_empty() {
        return Err(Diagnostics(errors));
    }
    Ok((tasks, coverage))
}

fn step(
    format: &dyn Format,
    doc: &Document,
    task: &OutlineTask,
    outline: &OutlineStep,
    cov: &mut TaskCoverage,
    errors: &mut Vec<Diagnostic>,
) -> Step {
    let mut cursor = Cursor::with_skip(doc, outline.blocks, task.manifest_blocks);
    let mut actions: Vec<Action> = Vec::new();
    while let Some(at) = cursor.peek(0).map(|b| b.span) {
        let before = cursor.consumed();
        let cx = StepContext {
            doc,
            task: &task.id,
            manifest: &task.manifest,
            settings: format.settings(),
            previous: &actions,
            written_step: outline.written,
        };
        let (origin, outcome) = first_match(format, &cx, &mut cursor);
        let consumed = cursor.consumed() - before;
        match outcome {
            Recognized::No => {
                if let Some(action) = fallback(&cx, &mut cursor, cov) {
                    actions.push(action);
                }
            }
            Recognized::Malformed(d) => {
                errors.push(d);
                cursor.advance(1);
            }
            _ if consumed == 0 => {
                errors.push(Diagnostic::error(
                    "recognizer-stalled",
                    Some(at),
                    format!("recognizer `{origin}` matched without consuming a block"),
                ));
                cursor.advance(1);
            }
            Recognized::Yes(new) if new.is_empty() => cov.absorbed.push(Absorbed {
                recognizer: RecognizerId(origin.into()),
                span: at,
            }),
            Recognized::Yes(new) => {
                let manual = new
                    .iter()
                    .all(|a| matches!(a.kind, ActionKind::Manual { .. }));
                match manual {
                    true => cov.manual += consumed,
                    false => cov.actions += consumed,
                }
                actions.extend(new);
            }
            Recognized::Amend(action) => {
                match action.kind {
                    ActionKind::Manual { .. } => cov.manual += consumed,
                    _ => cov.actions += consumed,
                }
                actions.pop();
                actions.push(action);
            }
        }
    }
    apply_mark(outline.mark, &mut actions, cov, errors);
    Step {
        number: outline.number,
        title: outline.title.clone(),
        span: outline.span,
        actions,
    }
}

/// Applies the mark of a step's title to its actions (decision D30).
fn apply_mark(
    mark: Option<StepMark>,
    actions: &mut [Action],
    cov: &mut TaskCoverage,
    errors: &mut Vec<Diagnostic>,
) {
    match mark {
        None => {}
        Some(StepMark::ExpectedFailure) => {
            errors.extend(actions.iter_mut().filter_map(expect_failure));
        }
        Some(StepMark::HumanCheck) => {
            for action in actions.iter_mut() {
                let ActionKind::Note { text } = &action.kind else {
                    continue;
                };
                action.kind = ActionKind::Manual {
                    reason: ManualReason::HumanCheck,
                    text: text.clone(),
                };
                cov.notes = cov.notes.saturating_sub(1);
                cov.manual += 1;
            }
        }
    }
}

/// An `expected-failure` step's `Verify` script expects failure: none or `any` becomes
/// `Failure`; an expected success is `expected-conflict`. Commits and gates are left alone.
fn expect_failure(action: &mut Action) -> Option<Diagnostic> {
    let span = action.span;
    let ActionKind::RunScript(script) = &mut action.kind else {
        return None;
    };
    if script.intent != ScriptIntent::Verify {
        return None;
    }
    let Some(expect) = &mut script.expect else {
        script.expect = Some(Expectation {
            exit: ExitExpect::Failure,
            lines: None,
            prose: None,
        });
        return None;
    };
    match expect.exit {
        ExitExpect::Unknown => {
            expect.exit = ExitExpect::Failure;
            None
        }
        ExitExpect::Success => Some(Diagnostic::error(
            "expected-conflict",
            Some(span),
            "the step's title says this fails, and its expectation says it succeeds",
        )),
        ExitExpect::Failure | ExitExpect::Code(_) => None,
    }
}

/// The first recognizer that answers anything but `No`, with its id.
fn first_match<'f>(
    format: &'f dyn Format,
    cx: &StepContext<'_>,
    cursor: &mut Cursor<'_>,
) -> (&'f str, Recognized) {
    for recognizer in format.recognizers() {
        let outcome = recognizer.recognize(cx, cursor);
        if !matches!(outcome, Recognized::No) {
            return (recognizer.id(), outcome);
        }
    }
    (FALLBACK, Recognized::No)
}

/// The built-in fallback: consumes one block. Prose becomes `Note` (or `Manual(Marked)`), a
/// fence `Manual`, a `text` fence `Note`; an HTML comment is dropped.
fn fallback(
    cx: &StepContext<'_>,
    cursor: &mut Cursor<'_>,
    cov: &mut TaskCoverage,
) -> Option<Action> {
    let located = cursor.peek(0)?;
    cursor.advance(1);
    let whole = TextRef::Span(located.span);
    let prose = |text: &TextRef| {
        if cx.settings.is_marked(cx.doc.text(text)) {
            ActionKind::Manual {
                reason: ManualReason::Marked,
                text: text.clone(),
            }
        } else {
            ActionKind::Note { text: text.clone() }
        }
    };
    let kind = match &located.block {
        Block::Paragraph { text }
        | Block::ListItem { text, .. }
        | Block::Quote { text }
        | Block::Heading { text, .. } => prose(text),
        Block::Table { text } => ActionKind::Note { text: text.clone() },
        Block::Rule => ActionKind::Note { text: whole },
        Block::Html { text } if cx.doc.text(text).trim_start().starts_with("<!--") => {
            cov.dropped += 1;
            return None;
        }
        Block::Html { text } => ActionKind::Note { text: text.clone() },
        Block::Fence { .. } => fence(cx, &located.block, whole),
    };
    match kind {
        ActionKind::Manual { .. } => cov.manual += 1,
        _ => cov.notes += 1,
    }
    Some(Action {
        kind,
        span: located.span,
        origin: RecognizerId(FALLBACK.into()),
        path_origin: Default::default(),
    })
}

fn fence(cx: &StepContext<'_>, block: &Block, whole: TextRef) -> ActionKind {
    let language = block.fence_language(cx.doc).unwrap_or("");
    let reason = match language {
        "text" => return ActionKind::Note { text: whole },
        "" => ManualReason::Unrecognized,
        shell if cx.settings.shell_for(shell).is_some() => ManualReason::Unrecognized,
        other => ManualReason::ForeignLanguage(other.into()),
    };
    ActionKind::Manual {
        reason,
        text: whole,
    }
}
```

File `crates/planrun-parse/tests/edit_kinds.rs` (full contents):

```rust
//! The kinds `run-line`, `before-after` and `manual-prose`, and paths resolved from the manifest
//! (decisions D32, D39), through a small list-step spec.

use planrun_model::{ActionKind, ExitExpect, ManualReason, PathOrigin, Shell, WriteMode};
use planrun_parse::{Registry, parse_plan};

const SPEC: &str = r#"
id = "e"
name = "edit kinds test"
[detect]
patterns = [{ regex = '^## Task', weight = 60 }]
[outline]
task = { level = 2, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
tasks_end_level = 2
[outline.manifest]
marker = '^\*\*Files:\*\*$'
item = '^(?P<change>Create|Modify|Test): `(?P<paths>[^`]+(?:`, `[^`]+)*)`$'
changes = { Create = "create", Modify = "modify", Test = "test" }
path_list = '`, `'
required = false
siblings = true
[[outline.steps]]
block = "list-item"
regex = '^\*\*Step (?P<n>\d+): (?P<title>.+)\*\*$'
[checks]
manifest-untouched = "off"
manifest-undeclared = "warning"
[[recognizer]]
kind = "file-block"
id = "file-block#e"
lead = '^`(?P<path>[^`]+)`:$'
ops = {}
default_op = "create"
file_name_from_manifest = true
[[recognizer]]
kind = "before-after"
id = "before-after#path"
lead = '^`(?P<path>[^`]+)`, before:$'
separator = '^After:$'
file_name_from_manifest = true
[[recognizer]]
kind = "before-after"
id = "before-after#no-path"
lead = '^Before:$'
separator = '^After:$'
infer_path = true
[[recognizer]]
kind = "before-after"
id = "before-after#adjacent"
lead = '^`(?P<path>[^`]+)` \(old → new\):$'
[[recognizer]]
kind = "run-line"
id = "run-line#e"
lead = '^Run: `(?P<cmd>[^`]*)`(?:\nExpected: (?P<expected>.+))?$'
shell = "pwsh"
exit = [
  { text = '(?i)\bfail', exit = "failure" },
  { text = '(?i)\bpass', exit = "success" },
]
intents = [{ intent = "commit", body = '\bgit commit\b' }]
[[recognizer]]
kind = "manual-prose"
id = "manual-prose#edit"
lead = '(?i)^(?:Append|Add)\b.*:$'
reason = "prose-edit"
takes_fence = true
[[recognizer]]
kind = "manual-prose"
id = "manual-prose#actual"
lead = 'with the `Actual` value'
reason = "output-dependent"
"#;

fn parse(text: &str) -> Result<planrun_parse::Parsed, Vec<&'static str>> {
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    registry
        .add_spec("e", SPEC)
        .unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, "e.md", text.as_bytes(), Some("e")).map_err(|d| d.error_codes())
}

fn recognized(text: &str) -> (planrun_parse::Parsed, Vec<planrun_model::Action>) {
    let parsed = parse(text).unwrap_or_else(|codes| panic!("{codes:?}"));
    let actions = parsed.plan.tasks[0]
        .steps
        .iter()
        .flat_map(|s| s.actions.iter().cloned())
        .collect();
    (parsed, actions)
}

const MANIFEST: &str = "**Files:**\n- Modify: `src/x/a.rs`, `b.rs`\n- Test: `tests/a.rs`\n";

#[test]
fn a_run_line_runs_its_command_with_the_expectation_of_its_paragraph() {
    let text = "## Task 1: a\n\n- [ ] **Step 1: verify**\n\n\
                Run: `cargo test a`\nExpected: FAIL with \"not found\"\n\n\
                Run: `cargo test`\n\nRun: `git commit -m x`\n";
    let (parsed, actions) = recognized(text);
    let runs: Vec<(String, Shell, Option<ExitExpect>, bool)> = actions
        .iter()
        .map(|a| match &a.kind {
            ActionKind::RunScript(s) | ActionKind::Commit(s) => (
                parsed.plan.text(&s.body).to_owned(),
                s.shell,
                s.expect.as_ref().map(|e| e.exit),
                matches!(a.kind, ActionKind::Commit(_)),
            ),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        runs,
        [
            (
                "cargo test a".to_owned(),
                Shell::Pwsh,
                Some(ExitExpect::Failure),
                false
            ),
            ("cargo test".to_owned(), Shell::Pwsh, None, false),
            ("git commit -m x".to_owned(), Shell::Pwsh, None, true),
        ]
    );
    let empty = "## Task 1: a\n\n- [ ] **Step 1: v**\n\nRun: ``\n";
    assert_eq!(parse(empty).err(), Some(vec!["missing-block"]));
}

#[test]
fn before_and_after_fences_replace_exact_text() {
    let text = format!(
        "## Task 1: a\n\n{MANIFEST}\n- [ ] **Step 1: edit**\n\n`src/x/b.rs`, before:\n\n\
         ```rust\nold\n```\n\nAfter:\n\n```rust\nnew\n```\n\n\
         `src/x/a.rs` (old → new):\n\n```rust\nx\n```\n```rust\ny\n```\n"
    );
    let (parsed, actions) = recognized(&text);
    let edits: Vec<(String, String, String, PathOrigin)> = actions
        .iter()
        .map(|a| match &a.kind {
            ActionKind::ReplaceExact {
                path,
                before,
                after,
            } => (
                path.to_string(),
                parsed.plan.text(before).to_owned(),
                parsed.plan.text(after).to_owned(),
                a.path_origin,
            ),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        edits,
        [
            (
                "src/x/b.rs".to_owned(),
                "old\n".to_owned(),
                "new\n".to_owned(),
                PathOrigin::Written
            ),
            (
                "src/x/a.rs".to_owned(),
                "x\n".to_owned(),
                "y\n".to_owned(),
                PathOrigin::Written
            ),
        ]
    );
    let broken = format!(
        "## Task 1: a\n\n{MANIFEST}\n- [ ] **Step 1: edit**\n\n`src/x/b.rs`, before:\n\n\
         ```rust\nold\n```\n\n```rust\nnew\n```\n"
    );
    assert_eq!(parse(&broken).err(), Some(vec!["missing-block"]));
}

#[test]
fn an_edit_without_a_path_takes_the_only_editable_entry() {
    let one = "## Task 1: a\n\n**Files:**\n- Modify: `docs/a.md`\n- Test: `tests/a.rs`\n\n\
               - [ ] **Step 1: edit**\n\nBefore:\n\n```md\nold\n```\n\nAfter:\n\n```md\nnew\n```\n";
    let (_, actions) = recognized(one);
    assert!(matches!(
        &actions[0].kind,
        ActionKind::ReplaceExact { path, .. } if path.as_str() == "docs/a.md"
    ));
    assert_eq!(actions[0].path_origin, PathOrigin::OnlyEntry);

    let two = format!(
        "## Task 1: a\n\n{MANIFEST}\n- [ ] **Step 1: edit**\n\nBefore:\n\n```md\nold\n```\n\n\
         After:\n\n```md\nnew\n```\n"
    );
    let (_, actions) = recognized(&two);
    assert!(matches!(
        &actions[0].kind,
        ActionKind::Manual {
            reason: ManualReason::ProseEdit,
            ..
        }
    ));
    assert_eq!(actions.len(), 1);
}

#[test]
fn a_bare_file_name_resolves_only_to_one_literal_entry() {
    let lead = |name: &str| {
        format!(
            "## Task 1: a\n\n**Files:**\n- Create: `src/x/new.rs`, `sib.rs`\n- Create: `tests/new.rs`\n\
             - Create: `docs/only.md`\n\n`{name}`:\n\n```rust\nx\n```\n"
        )
    };
    let resolved = |name: &str| {
        let parsed = parse(&lead(name)).unwrap_or_else(|codes| panic!("{codes:?}"));
        let action = &parsed.plan.tasks[0].steps[0].actions[0];
        match &action.kind {
            ActionKind::WriteFile { path, mode, .. } => {
                assert_eq!(*mode, WriteMode::Create);
                (path.to_string(), action.path_origin)
            }
            other => panic!("{other:?}"),
        }
    };
    assert_eq!(
        resolved("only.md"),
        ("docs/only.md".to_owned(), PathOrigin::FileName)
    );
    // Two entries are named new.rs, and sib.rs is itself a sibling: both stay as written.
    assert_eq!(
        resolved("new.rs"),
        ("new.rs".to_owned(), PathOrigin::Written)
    );
    assert_eq!(
        resolved("sib.rs"),
        ("sib.rs".to_owned(), PathOrigin::Written)
    );
}

#[test]
fn prose_edits_and_output_dependent_edits_are_manual() {
    let text = "## Task 1: a\n\n- [ ] **Step 1: edit**\n\nAppend to the class:\n\n\
                ```powershell\nfunction X {}\n```\n\n\
                Replace the hash in `tests/h.rs` with the `Actual` value.\n";
    let (parsed, actions) = recognized(text);
    let manual: Vec<(ManualReason, usize)> = actions
        .iter()
        .map(|a| match &a.kind {
            ActionKind::Manual { reason, text } => {
                (reason.clone(), parsed.plan.text(text).lines().count())
            }
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        manual,
        [
            (ManualReason::ProseEdit, 5),
            (ManualReason::OutputDependent, 1)
        ]
    );
    assert_eq!(parsed.coverage[0].manual, 3);
}

#[test]
fn edit_kind_parameters_are_validated_when_the_spec_loads() {
    for (from, to) in [
        ("lead = '^Before:$'", "lead = '^`(?P<path>[^`]+)`:$'"),
        ("reason = \"prose-edit\"", "reason = \"by-hand\""),
        ("shell = \"pwsh\"", "shell = \"cmd\""),
        (
            "lead = '^Run: `(?P<cmd>[^`]*)`",
            "lead = '^Run: `(?P<command>[^`]*)`",
        ),
    ] {
        let spec = SPEC.replacen(from, to, 1);
        assert_ne!(spec, SPEC, "{from}");
        let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
        assert!(registry.add_spec("e", &spec).is_err(), "{to} loads");
    }
}
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
# a `:3-10` piece adds lines to the path before it (D32)
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
# expected success is the error expected-conflict). human-check: its Notes become
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

**Step 2 — verify**

Targeted: the edit-kind tests:

```powershell
cargo test -p planrun-parse --test edit_kinds
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
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-cli/src/view.rs crates/planrun-model/src/action.rs crates/planrun-parse/src/kinds/before_after.rs crates/planrun-parse/src/kinds/delete.rs crates/planrun-parse/src/kinds/file_block.rs crates/planrun-parse/src/kinds/generated.rs crates/planrun-parse/src/kinds/manual_prose.rs crates/planrun-parse/src/kinds/mod.rs crates/planrun-parse/src/kinds/run_line.rs crates/planrun-parse/src/kinds/script.rs crates/planrun-parse/src/recognize.rs crates/planrun-parse/tests/edit_kinds.rs docs/extending.md
  $verified = [ordered]@{
    'crates/planrun-cli/src/view.rs' = '1db1d5399ccc11e1158c986f2612a3a399d3fd87'
    'crates/planrun-model/src/action.rs' = '71926f34387dce8cf6b478444f2acbede33c7a1b'
    'crates/planrun-parse/src/kinds/before_after.rs' = '51b12bdc2e722f7bb06101a332765fa97fcc7f17'
    'crates/planrun-parse/src/kinds/delete.rs' = '116f1bfe88acb913c0d4e84ff4b5ea05f8590958'
    'crates/planrun-parse/src/kinds/file_block.rs' = '32f225bf643fd56ec9efecde1a214c084cc096df'
    'crates/planrun-parse/src/kinds/generated.rs' = '6b52dede91c136fc8d11988a034d43b033a6fc29'
    'crates/planrun-parse/src/kinds/manual_prose.rs' = '2bc9546b43f0d472be3ca4d2c39cd1668c8ccb23'
    'crates/planrun-parse/src/kinds/mod.rs' = 'bafd2c68c43c8ef8616dfd43b012541eb186ba8a'
    'crates/planrun-parse/src/kinds/run_line.rs' = '8cafae831fcd95c6db533dba5d5a8573d687aa9d'
    'crates/planrun-parse/src/kinds/script.rs' = 'e274574190375cebf1a2bd611d6588690c987889'
    'crates/planrun-parse/src/recognize.rs' = '715d58d871a63311613cfb952af1f1808e1ba76b'
    'crates/planrun-parse/tests/edit_kinds.rs' = 'b9ddcbe3fe9fe6f5834065cc9f5229ae977f06ea'
    'docs/extending.md' = 'c2403cc60c4143740c1e812041b53ca5086f0156'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P2a-T4: run-line, before-after and manual-prose; paths resolved from the manifest" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P2a-T5: the superpowers format

**Goal:** `formats/superpowers.toml`, embedded; the IR snapshot of `superpowers-style.md`; the invalid-corpus file `expected-conflict.md`; docs/plan-grammar.md §3.

**Files**

- Create `crates/planrun-cli/src/snapshots/planrun__view__tests__superpowers_style_ir.snap`
- Modify `crates/planrun-cli/src/view.rs`
- Modify `crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap`
- Modify `crates/planrun-parse/src/registry.rs`
- Modify `docs/plan-grammar.md`
- Create `formats/superpowers.toml`
- Create `tests/corpus/invalid/expected-conflict.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/src/view.rs`, `crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap`.

The invalid-corpus snapshot gains the `expected-conflict` entry.

**Step 1 — write the files**

File `crates/planrun-cli/src/snapshots/planrun__view__tests__superpowers_style_ir.snap` (full contents):

```text
---
source: crates/planrun-cli/src/view.rs
expression: "Viewer::new(&plan).plan()"
---
plan: tests/corpus/synthetic/superpowers-style.md
format: superpowers
title: Farewell Implementation Plan
phase: ~
preamble: []
tasks:
  - id: Task 1
    title: The farewell module
    at: L13
    files:
      - Create src/farewell.rs
      - "Modify src/lib.rs:3-4"
      - Test src/farewell.rs
    steps:
      - step: 1 — Write the failing test
        at: L20
        actions:
          - at: L22
            by: "file-block#superpowers"
            write-file:
              path: src/farewell.rs
              mode: Create
              body: "#[cfg(test)]\nmod tests {\n    #[test]\n    fn says_goodbye_by_name() {\n        assert_eq!(super::farewell(\"Rens\"), \"Goodbye, Rens.\");\n    }\n}\n"
      - step: 2 — Run test to verify it fails
        at: L34
        actions:
          - at: L36
            by: "run-line#superpowers"
            run:
              shell: pwsh
              intent: verify
              body: cargo test farewell
              expect:
                exit: fails
                lines: ~
                prose: "FAIL with \"cannot find function `farewell` in module `super`\""
      - step: 3 — Write minimal implementation
        at: L39
        actions:
          - at: L41
            by: "manual-prose#edit"
            manual:
              reason: ProseEdit
              text: "Above the test module in `src/farewell.rs`, add:\n\n```rust\n/// The farewell for `name`.\npub fn farewell(name: &str) -> String {\n    format!(\"Goodbye, {name}.\")\n}\n```"
          - at: L50
            by: "manual-prose#edit"
            manual:
              reason: ProseEdit
              text: "In `src/lib.rs`, below `pub mod greet;` (line 3), add `pub mod farewell;` and change the re-export to:\n\n```rust\npub use farewell::farewell;\npub use greet::greeting;\n```"
      - step: 4 — Run test to verify it passes
        at: L57
        actions:
          - at: L59
            by: "run-line#superpowers"
            run:
              shell: pwsh
              intent: verify
              body: cargo test farewell
              expect:
                exit: exit 0
                lines: ~
                prose: PASS
      - step: 5 — Commit
        at: L62
        actions:
          - at: L64
            by: "script#superpowers"
            commit:
              shell: bash
              intent: other
              body: "git add src/farewell.rs src/lib.rs\ngit commit -m \"feat: farewell\"\n"
              expect: ~
          - at: L69
            by: fallback
            note:
              text: "---"
  - id: Task 2
    title: Document the API
    at: L71
    files:
      - Modify README.md
    steps:
      - step: 1 — Add the section
        at: L76
        actions:
          - at: L78
            by: "manual-prose#edit"
            manual:
              reason: ProseEdit
              text: "Append to `README.md`:\n\n```markdown\n## API\n\n- `greeting(name)`: `Hello, <name>!`\n- `farewell(name)`: `Goodbye, <name>.`\n```"
      - step: 2 — Check by eye
        at: L87
        actions:
          - at: L89
            by: fallback
            manual:
              reason: HumanCheck
              text: "Open `README.md` in a Markdown preview: both bullets render, the code spans are monospace."
      - step: 3 — Commit
        at: L91
        actions:
          - at: L93
            by: "script#superpowers"
            commit:
              shell: bash
              intent: other
              body: "git add README.md\ngit commit -m \"docs: API section\"\n"
              expect: ~
          - at: L98
            by: fallback
            note:
              text: "---"
after_tasks: L100
```

File `crates/planrun-cli/src/view.rs` (exact diff; save the block as `$env:TEMP\p2a-t5-view-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/view.rs b/crates/planrun-cli/src/view.rs
index 1db1d53..58afbbb 100644
--- a/crates/planrun-cli/src/view.rs
+++ b/crates/planrun-cli/src/view.rs
@@ -456,19 +456,21 @@ mod tests {
 
     use super::*;
 
-    fn synthetic() -> Plan {
+    /// The synthetic plan `name` (`assay-style.md`), parsed with the built-in formats.
+    fn synthetic_plan(name: &str) -> Plan {
+        let relative = format!("tests/corpus/synthetic/{name}");
         let path = Path::new(env!("CARGO_MANIFEST_DIR"))
-            .join("../../tests/corpus/synthetic/assay-style.md");
+            .join("../..")
+            .join(&relative);
         let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
         let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
-        parse_plan(
-            &registry,
-            "tests/corpus/synthetic/assay-style.md",
-            &bytes,
-            None,
-        )
-        .unwrap_or_else(|d| panic!("{:?}", d.error_codes()))
-        .plan
+        parse_plan(&registry, &relative, &bytes, None)
+            .unwrap_or_else(|d| panic!("{name}: {:?}", d.error_codes()))
+            .plan
+    }
+
+    fn synthetic() -> Plan {
+        synthetic_plan("assay-style.md")
     }
 
     #[test]
@@ -477,6 +479,13 @@ mod tests {
         insta::assert_yaml_snapshot!("assay_style_ir", Viewer::new(&plan).plan());
     }
 
+    #[test]
+    fn the_ir_of_the_synthetic_superpowers_plan() {
+        let plan = synthetic_plan("superpowers-style.md");
+        assert_eq!(&*plan.format.0, "superpowers");
+        insta::assert_yaml_snapshot!("superpowers_style_ir", Viewer::new(&plan).plan());
+    }
+
     #[test]
     fn show_prints_one_line_per_action() {
         let plan = synthetic();
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2a-t5-view-rs.patch"
Remove-Item "$env:TEMP\p2a-t5-view-rs.patch"
```

File `crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap` (exact diff; save the block as `$env:TEMP\p2a-t5-cli-check-invalid-corpus-snap.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap b/crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap
index 44e270e..087affc 100644
--- a/crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap
+++ b/crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap
@@ -90,6 +90,16 @@ error[duplicate-task-id]: task `P1-T1` is defined twice
 
 planrun: tests/corpus/invalid/duplicate-task-id.md: 1 error
 
+==== expected-conflict.md
+error[expected-conflict]: the step's title says this fails, and its expectation says it succeeds
+  --> tests/corpus/invalid/expected-conflict.md:21:1
+   |
+21 | / Run: `cargo test a`
+22 | | Expected: PASS
+   | |______________^
+
+planrun: tests/corpus/invalid/expected-conflict.md: 1 error
+
 ==== expected-spec.md
 error[expected-spec]: `summary lines` names no exit status this format knows
   --> tests/corpus/invalid/expected-spec.md:30:1
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2a-t5-cli-check-invalid-corpus-snap.patch"
Remove-Item "$env:TEMP\p2a-t5-cli-check-invalid-corpus-snap.patch"
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

Hand-written; mostly automatable. Example: `before-after-style.md`.

| Element | Form |
|---|---|
| Task | `## Task 1: title` (**level 2**) |
| File map | a table before the tasks: shown, never parsed into actions |
| Manifest | `**Files:**` + `Create:`, `Modify:` (with `:lines`), `Rewrite:`, `Test:`; several paths per item allowed |
| Step | list item `- [ ] **Step 1: title**` |
| Create | `` `p`: `` + fence |
| Overwrite | `` `p` — replace the whole file: `` / ``Replace `p`:`` + fence |
| Edit | `` `p`, before: `` + fence + `After:` + fence → `ReplaceExact` (`before` must occur exactly once) |
| Partial edits | "replace the class (keep …)", "add these members to the class:" → `Manual(ProseEdit)` |
| Expected | prose `Expected: …`, including `build error` (expected failure) and `no matches` (`rg` exits 1) |
| Preview/apply | a `-DryRun` script followed by its apply: intent Preview then Apply; planrun always stops between them and shows the count next to the plan's expected count |
| Output-dependent | "Replace … with the `Actual` values" → `Manual(OutputDependent)` |
| Other languages | `sql` fences → `Manual(ForeignLanguage("sql"))` |

## 6. Recognizing a new format

Write the table above for it first, from at least two real plans. Then follow extending.md §2.
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
  { text = '(?i)\b(fail|build error|does not exist)', exit = "failure" },
  { text = '(?i)\b(pass(es)?|succeeds?|succeeded)\b', exit = "success" },
]
```

File `tests/corpus/invalid/expected-conflict.md` (full contents):

````markdown
<!-- expect: expected-conflict -->
<!-- why: the step's title says the test fails, and its Expected says it passes -->

# Invalid — expected-conflict

### Task 1: a

**Files:**
- Create: `src/a.rs`

- [ ] **Step 1: Write the test**

Create `src/a.rs`:

```rust
// x
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test a`
Expected: PASS
````

**Step 2 — verify**

Targeted: the synthetic superpowers plan:

```powershell
cargo run -q -p planrun-cli -- check tests/corpus/synthetic/superpowers-style.md
```

Expected (exit code 0; summary lines):

```text
planrun: tests/corpus/synthetic/superpowers-style.md: ok (superpowers, 2 tasks)
```

Gate:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-cli/src/snapshots/planrun__view__tests__superpowers_style_ir.snap crates/planrun-cli/src/view.rs crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap crates/planrun-parse/src/registry.rs docs/plan-grammar.md formats/superpowers.toml tests/corpus/invalid/expected-conflict.md
  $verified = [ordered]@{
    'crates/planrun-cli/src/snapshots/planrun__view__tests__superpowers_style_ir.snap' = '89dde6855516ca87af30ff5ad6fb4e17481a807a'
    'crates/planrun-cli/src/view.rs' = '58afbbb4455b9cbb7951722975e287b04ea7d898'
    'crates/planrun-cli/tests/snapshots/cli__check_invalid_corpus.snap' = '087affc76f3b4424cacc80a60188441ed3e3c921'
    'crates/planrun-parse/src/registry.rs' = '0f67e88fd996a034d672048d0a4972f51510e8b1'
    'docs/plan-grammar.md' = '184bf7c9f7e70553d1ee4336ed9e5d8d8da0e1f0'
    'formats/superpowers.toml' = '02fb03092a5cc3180985cb352a8b1df951adb385'
    'tests/corpus/invalid/expected-conflict.md' = '0232fae64d50f704af04c7b20e975dc4f392a4eb'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P2a-T5: the superpowers format" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P2a-T6: the before-after format

**Goal:** `formats/before-after.toml`, embedded; the IR snapshot of `before-after-style.md`; docs/plan-grammar.md §5. A human-check step now turns only its prose into Manual (in the fallback, which knows the block kind), so a `---` rule stays a Note; D30 says so.

**Files**

- Create `crates/planrun-cli/src/snapshots/planrun__view__tests__before_after_style_ir.snap`
- Modify `crates/planrun-cli/src/view.rs`
- Modify `crates/planrun-parse/src/recognize.rs`
- Modify `crates/planrun-parse/src/recognizer.rs`
- Modify `crates/planrun-parse/src/registry.rs`
- Modify `docs/decisions.md`
- Modify `docs/extending.md`
- Modify `docs/plan-grammar.md`
- Create `formats/before-after.toml`
- Modify `tests/corpus/synthetic/README.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/src/view.rs`.

**Step 1 — write the files**

File `crates/planrun-cli/src/snapshots/planrun__view__tests__before_after_style_ir.snap` (full contents):

```text
---
source: crates/planrun-cli/src/view.rs
expression: "Viewer::new(&plan).plan()"
---
plan: tests/corpus/synthetic/before-after-style.md
format: before-after
title: Greeter localisation — implementation plan
phase: ~
preamble: []
tasks:
  - id: Task 0
    title: Baseline
    at: L28
    files: []
    steps:
      - step: 1 — Confirm branch and green baseline
        at: L30
        actions:
          - at: L32
            by: "script#before-after"
            run:
              shell: pwsh
              intent: verify
              body: "git branch --show-current\ncargo test\n"
              expect:
                exit: exit 0
                lines: ~
                prose: "branch `feat/lang`, all tests pass."
          - at: L39
            by: fallback
            note:
              text: "---"
  - id: Task 1
    title: Lang and greeting_in
    at: L41
    files:
      - Create src/lang.rs
      - "Modify src/lib.rs:3-3"
    steps:
      - step: 1 — Create the module
        at: L47
        actions:
          - at: L49
            by: "file-block#create"
            write-file:
              path: src/lang.rs
              mode: Create
              body: "//! Languages of a greeting.\n\n/// A greeting language.\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum Lang {\n    /// English.\n    En,\n    /// Dutch.\n    Nl,\n}\n\n/// The greeting for `name` in `lang`.\npub fn greeting_in(lang: Lang, name: &str) -> String {\n    match lang {\n        Lang::En => format!(\"Hello, {name}!\"),\n        Lang::Nl => format!(\"Hallo, {name}!\"),\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn dutch_says_hallo() {\n        assert_eq!(greeting_in(Lang::Nl, \"Rens\"), \"Hallo, Rens!\");\n    }\n}\n"
      - step: 2 — Run the tests to verify they fail
        at: L82
        actions:
          - at: L84
            by: "script#before-after"
            run:
              shell: pwsh
              intent: verify
              body: "cargo test lang\n"
              expect:
                exit: fails
                lines: ~
                prose: "build error, `lang` is not a module of the crate yet."
      - step: 3 — Declare the module
        at: L90
        actions:
          - at: L92
            by: "before-after#path"
            replace-exact:
              path: src/lib.rs
              before: "pub mod greet;\n"
              after: "pub mod greet;\npub mod lang;\n"
      - step: 4 — Run the tests to verify they pass
        at: L105
        actions:
          - at: L107
            by: "script#before-after"
            run:
              shell: pwsh
              intent: verify
              body: "cargo test lang\n"
              expect:
                exit: exit 0
                lines: ~
                prose: all pass.
      - step: 5 — Commit
        at: L113
        actions:
          - at: L115
            by: "script#before-after"
            commit:
              shell: pwsh
              intent: other
              body: "git add src/lang.rs src/lib.rs\ngit commit -m \"feat(lang): Lang and greeting_in\"\n"
              expect: ~
          - at: L120
            by: fallback
            note:
              text: "---"
  - id: Task 2
    title: greeting delegates
    at: L122
    files:
      - Rewrite src/greet.rs
    steps:
      - step: 1 — Replace the module
        at: L127
        actions:
          - at: L129
            by: "file-block#replace-whole"
            write-file:
              path: src/greet.rs
              mode: Overwrite
              body: "//! Greetings: English by default.\n\nuse crate::lang::{Lang, greeting_in};\n\n/// The English greeting for `name`.\npub fn greeting(name: &str) -> String {\n    greeting_in(Lang::En, name)\n}\n"
      - step: 2 — Add the regression test to the module
        at: L142
        actions:
          - at: L144
            by: "manual-prose#edit"
            manual:
              reason: ProseEdit
              text: "`src/greet.rs`, add these members at the end of the file:\n\n```rust\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn english_is_the_default() {\n        assert_eq!(super::greeting(\"Rens\"), \"Hello, Rens!\");\n    }\n}\n```"
      - step: 3 — Verify nothing still formats a greeting by hand
        at: L156
        actions:
          - at: L158
            by: "script#before-after"
            run:
              shell: pwsh
              intent: verify
              body: "rg -n 'format!\\(\"Hello' src --glob '!src/lang.rs'\n"
              expect:
                exit: exit 1
                lines: ~
                prose: no matches.
      - step: "4 — Count the call sites, then rename them"
        at: L164
        actions:
          - at: L166
            by: fallback
            note:
              text: "Preview:"
          - at: L168
            by: "script#before-after"
            run:
              shell: pwsh
              intent: preview
              body: "replace 'greeting(' 'greeting_en(' -Path src -Extension .rs -DryRun\n"
              expect:
                exit: not checked
                lines: ~
                prose: "2 matches in 2 files. Then:"
          - at: L174
            by: "script#before-after"
            run:
              shell: pwsh
              intent: apply
              body: "replace 'greeting(' 'greeting_en(' -Path src -Extension .rs -All\n"
              expect: ~
      - step: 5 — Re-pin the README hash
        at: L178
        actions:
          - at: L180
            by: "script#before-after"
            run:
              shell: pwsh
              intent: verify
              body: "cargo test readme_hash\n"
              expect: ~
          - at: L184
            by: "manual-prose#output-dependent"
            manual:
              reason: OutputDependent
              text: "The failure prints `Expected: <old>` and `Actual: <new>`. Replace the hash in `tests/readme.rs` with the `Actual` value."
      - step: 6 — Commit
        at: L186
        actions:
          - at: L188
            by: "script#before-after"
            commit:
              shell: pwsh
              intent: other
              body: "git add src tests\ngit commit -m \"feat(lang): greeting delegates to greeting_in\"\n"
              expect: ~
          - at: L193
            by: fallback
            note:
              text: "---"
  - id: Task 3
    title: Release check on TST
    at: L195
    files: []
    steps:
      - step: 1 — Record the release
        at: L197
        actions:
          - at: L199
            by: fallback
            note:
              text: "Run on the TST database:"
          - at: L201
            by: fallback
            manual:
              reason: "ForeignLanguage(\"sql\")"
              text: "```sql\nINSERT INTO dbo.Releases (Name, ReleasedAt) VALUES (N'greeter 0.2.0', SYSUTCDATETIME());\n```\n\nExpected: 1 row affected."
      - step: 2 — Hand off
        at: L207
        actions:
          - at: L209
            by: fallback
            manual:
              reason: HumanCheck
              text: Report the TST result in the PR description.
after_tasks: ~
```

File `crates/planrun-cli/src/view.rs` (exact diff; save the block as `$env:TEMP\p2a-t6-view-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/view.rs b/crates/planrun-cli/src/view.rs
index 58afbbb..f77a9f0 100644
--- a/crates/planrun-cli/src/view.rs
+++ b/crates/planrun-cli/src/view.rs
@@ -486,6 +486,13 @@ mod tests {
         insta::assert_yaml_snapshot!("superpowers_style_ir", Viewer::new(&plan).plan());
     }
 
+    #[test]
+    fn the_ir_of_the_synthetic_before_after_plan() {
+        let plan = synthetic_plan("before-after-style.md");
+        assert_eq!(&*plan.format.0, "before-after");
+        insta::assert_yaml_snapshot!("before_after_style_ir", Viewer::new(&plan).plan());
+    }
+
     #[test]
     fn show_prints_one_line_per_action() {
         let plan = synthetic();
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2a-t6-view-rs.patch"
Remove-Item "$env:TEMP\p2a-t6-view-rs.patch"
```

File `crates/planrun-parse/src/recognize.rs` (full contents):

```rust
//! Recognition (docs/architecture.md §4.4): for each step, a cursor walks its blocks and the
//! format's recognizers are tried in order; the first that answers anything but `No` wins. A
//! block no recognizer takes goes to the built-in fallback, so every block is accounted for
//! (decision D21).

use planrun_model::{
    Action, ActionKind, ExitExpect, Expectation, ManualReason, RecognizerId, ScriptIntent, Span,
    Step, Task, TaskId, TextRef,
};

use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::document::{Block, Document};
use crate::format::{Format, Outline, OutlineStep, OutlineTask, StepMark};
use crate::recognizer::{Cursor, Recognized, StepContext};

/// Where one task's blocks went (the coverage report, docs/architecture.md §4.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskCoverage {
    /// The task.
    pub task: TaskId,
    /// Blocks that became actions other than `Manual`, expectations included.
    pub actions: usize,
    /// Blocks absorbed without an action (the patch ceremony), with where they are.
    pub absorbed: Vec<Absorbed>,
    /// The manifest's blocks.
    pub manifest: usize,
    /// Blocks that became `Note`.
    pub notes: usize,
    /// Blocks that became `Manual`, through the fallback or a recognizer (`manual-prose`).
    pub manual: usize,
    /// HTML comments, dropped.
    pub dropped: usize,
}

impl TaskCoverage {
    /// Every block of the task's steps and manifest.
    pub fn blocks(&self) -> usize {
        self.actions + self.absorbed.len() + self.manifest + self.notes + self.manual + self.dropped
    }
}

/// One absorbed block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Absorbed {
    /// The recognizer that absorbed it.
    pub recognizer: RecognizerId,
    /// The block.
    pub span: Span,
}

/// The recognizer id of the built-in fallback.
pub const FALLBACK: &str = "fallback";

/// The tasks of `outline` with their actions, and the coverage of each. Every `Malformed` is
/// collected; any means no tasks.
pub fn recognize(
    format: &dyn Format,
    doc: &Document,
    outline: &Outline,
) -> Result<(Vec<Task>, Vec<TaskCoverage>), Diagnostics> {
    let mut errors = Vec::new();
    let mut tasks = Vec::with_capacity(outline.tasks.len());
    let mut coverage = Vec::with_capacity(outline.tasks.len());
    for task in &outline.tasks {
        let mut cov = TaskCoverage {
            task: task.id.clone(),
            actions: 0,
            absorbed: Vec::new(),
            manifest: task.manifest_blocks.map_or(0, |m| m.len()),
            notes: 0,
            manual: 0,
            dropped: 0,
        };
        let steps = task
            .steps
            .iter()
            .map(|s| step(format, doc, task, s, &mut cov, &mut errors))
            .collect();
        tasks.push(Task {
            id: task.id.clone(),
            title: task.title.clone(),
            span: task.span,
            manifest: task.manifest.clone(),
            steps,
        });
        coverage.push(cov);
    }
    if !errors.is_empty() {
        return Err(Diagnostics(errors));
    }
    Ok((tasks, coverage))
}

fn step(
    format: &dyn Format,
    doc: &Document,
    task: &OutlineTask,
    outline: &OutlineStep,
    cov: &mut TaskCoverage,
    errors: &mut Vec<Diagnostic>,
) -> Step {
    let mut cursor = Cursor::with_skip(doc, outline.blocks, task.manifest_blocks);
    let mut actions: Vec<Action> = Vec::new();
    while let Some(at) = cursor.peek(0).map(|b| b.span) {
        let before = cursor.consumed();
        let cx = StepContext {
            doc,
            task: &task.id,
            manifest: &task.manifest,
            settings: format.settings(),
            previous: &actions,
            written_step: outline.written,
            mark: outline.mark,
        };
        let (origin, outcome) = first_match(format, &cx, &mut cursor);
        let consumed = cursor.consumed() - before;
        match outcome {
            Recognized::No => {
                if let Some(action) = fallback(&cx, &mut cursor, cov) {
                    actions.push(action);
                }
            }
            Recognized::Malformed(d) => {
                errors.push(d);
                cursor.advance(1);
            }
            _ if consumed == 0 => {
                errors.push(Diagnostic::error(
                    "recognizer-stalled",
                    Some(at),
                    format!("recognizer `{origin}` matched without consuming a block"),
                ));
                cursor.advance(1);
            }
            Recognized::Yes(new) if new.is_empty() => cov.absorbed.push(Absorbed {
                recognizer: RecognizerId(origin.into()),
                span: at,
            }),
            Recognized::Yes(new) => {
                let manual = new
                    .iter()
                    .all(|a| matches!(a.kind, ActionKind::Manual { .. }));
                match manual {
                    true => cov.manual += consumed,
                    false => cov.actions += consumed,
                }
                actions.extend(new);
            }
            Recognized::Amend(action) => {
                match action.kind {
                    ActionKind::Manual { .. } => cov.manual += consumed,
                    _ => cov.actions += consumed,
                }
                actions.pop();
                actions.push(action);
            }
        }
    }
    if outline.mark == Some(StepMark::ExpectedFailure) {
        errors.extend(actions.iter_mut().filter_map(expect_failure));
    }
    Step {
        number: outline.number,
        title: outline.title.clone(),
        span: outline.span,
        actions,
    }
}

/// An `expected-failure` step's `Verify` script expects failure: none or `any` becomes
/// `Failure`; an expected success is `expected-conflict`. Commits and gates are left alone.
fn expect_failure(action: &mut Action) -> Option<Diagnostic> {
    let span = action.span;
    let ActionKind::RunScript(script) = &mut action.kind else {
        return None;
    };
    if script.intent != ScriptIntent::Verify {
        return None;
    }
    let Some(expect) = &mut script.expect else {
        script.expect = Some(Expectation {
            exit: ExitExpect::Failure,
            lines: None,
            prose: None,
        });
        return None;
    };
    match expect.exit {
        ExitExpect::Unknown => {
            expect.exit = ExitExpect::Failure;
            None
        }
        ExitExpect::Success => Some(Diagnostic::error(
            "expected-conflict",
            Some(span),
            "the step's title says this fails, and its expectation says it succeeds",
        )),
        ExitExpect::Failure | ExitExpect::Code(_) => None,
    }
}

/// The first recognizer that answers anything but `No`, with its id.
fn first_match<'f>(
    format: &'f dyn Format,
    cx: &StepContext<'_>,
    cursor: &mut Cursor<'_>,
) -> (&'f str, Recognized) {
    for recognizer in format.recognizers() {
        let outcome = recognizer.recognize(cx, cursor);
        if !matches!(outcome, Recognized::No) {
            return (recognizer.id(), outcome);
        }
    }
    (FALLBACK, Recognized::No)
}

/// The built-in fallback: consumes one block. Prose becomes `Note`, or `Manual(Marked)` when it
/// matches a manual marker, or `Manual(HumanCheck)` in a `human-check` step (decision D30); a
/// fence `Manual`, a `text` fence `Note`; an HTML comment is dropped.
fn fallback(
    cx: &StepContext<'_>,
    cursor: &mut Cursor<'_>,
    cov: &mut TaskCoverage,
) -> Option<Action> {
    let located = cursor.peek(0)?;
    cursor.advance(1);
    let whole = TextRef::Span(located.span);
    let prose = |text: &TextRef| {
        let reason = match cx.mark {
            _ if cx.settings.is_marked(cx.doc.text(text)) => ManualReason::Marked,
            Some(StepMark::HumanCheck) => ManualReason::HumanCheck,
            _ => return ActionKind::Note { text: text.clone() },
        };
        ActionKind::Manual {
            reason,
            text: text.clone(),
        }
    };
    let kind = match &located.block {
        Block::Paragraph { text }
        | Block::ListItem { text, .. }
        | Block::Quote { text }
        | Block::Heading { text, .. } => prose(text),
        Block::Table { text } => ActionKind::Note { text: text.clone() },
        Block::Rule => ActionKind::Note { text: whole },
        Block::Html { text } if cx.doc.text(text).trim_start().starts_with("<!--") => {
            cov.dropped += 1;
            return None;
        }
        Block::Html { text } => ActionKind::Note { text: text.clone() },
        Block::Fence { .. } => fence(cx, &located.block, whole),
    };
    match kind {
        ActionKind::Manual { .. } => cov.manual += 1,
        _ => cov.notes += 1,
    }
    Some(Action {
        kind,
        span: located.span,
        origin: RecognizerId(FALLBACK.into()),
        path_origin: Default::default(),
    })
}

fn fence(cx: &StepContext<'_>, block: &Block, whole: TextRef) -> ActionKind {
    let language = block.fence_language(cx.doc).unwrap_or("");
    let reason = match language {
        "text" => return ActionKind::Note { text: whole },
        "" => ManualReason::Unrecognized,
        shell if cx.settings.shell_for(shell).is_some() => ManualReason::Unrecognized,
        other => ManualReason::ForeignLanguage(other.into()),
    };
    ActionKind::Manual {
        reason,
        text: whole,
    }
}
```

File `crates/planrun-parse/src/recognizer.rs` (full contents):

```rust
//! Recognizers: small, composable matchers that turn a run of blocks into actions
//! (docs/architecture.md §4.4). A format is an ordered list of recognizer instances.

use std::sync::Arc;

use planrun_model::{Action, Manifest, TaskId};

use crate::diagnostic::Diagnostic;
use crate::document::{BlockRange, Document, Located};
use crate::format::{FormatSettings, StepMark};

/// What a recognizer may read besides the blocks.
pub struct StepContext<'a> {
    /// The document.
    pub doc: &'a Document,
    /// The task being recognized.
    pub task: &'a TaskId,
    /// Its manifest (a full-contents write consults it for create vs overwrite).
    pub manifest: &'a Manifest,
    /// Format settings.
    pub settings: &'a FormatSettings,
    /// Actions already produced in this step: an expectation attaches to the script before it;
    /// the patch ceremony is absorbed only right after a patch.
    pub previous: &'a [Action],
    /// True in a step that starts at a written boundary; false in the implicit first step (the
    /// blocks before the first boundary, where the manifest is).
    pub written_step: bool,
    /// The mark of the step's title (decision D30).
    pub mark: Option<StepMark>,
}

/// A position in a step's blocks. A recognizer that matches advances it past what it consumed.
/// Blocks in the `skip` range (the task's manifest) are invisible to it.
pub struct Cursor<'a> {
    blocks: &'a [Located],
    range: BlockRange,
    skip: Option<BlockRange>,
    position: usize,
    consumed: usize,
}

impl<'a> Cursor<'a> {
    /// A cursor at the start of `range`.
    pub fn new(doc: &'a Document, range: BlockRange) -> Self {
        Self::with_skip(doc, range, None)
    }

    /// A cursor at the start of `range` that never sees the blocks of `skip`.
    pub fn with_skip(doc: &'a Document, range: BlockRange, skip: Option<BlockRange>) -> Self {
        let mut cursor = Self {
            blocks: &doc.blocks,
            range,
            skip,
            position: range.start,
            consumed: 0,
        };
        cursor.position = cursor.settle(range.start);
        cursor
    }

    /// `index`, or the first index after the skipped range when it falls inside it.
    fn settle(&self, index: usize) -> usize {
        match self.skip {
            Some(skip) if skip.contains(index) => skip.end,
            _ => index,
        }
    }

    /// The block `offset` positions ahead, if it is still in the step.
    pub fn peek(&self, offset: usize) -> Option<&'a Located> {
        let mut index = self.position;
        for _ in 0..offset {
            index = self.settle(index + 1);
        }
        (index < self.range.end).then(|| &self.blocks[index])
    }

    /// The block before the cursor, if it is still in the step (skipped blocks are invisible).
    pub fn previous(&self) -> Option<&'a Located> {
        let mut index = self.position.checked_sub(1)?;
        if let Some(skip) = self.skip.filter(|s| s.contains(index)) {
            index = skip.start.checked_sub(1)?;
        }
        (index >= self.range.start).then(|| &self.blocks[index])
    }

    /// True when the cursor is at the step's first block.
    pub fn at_start(&self) -> bool {
        self.position == self.settle(self.range.start)
    }

    /// Consumes `count` blocks.
    pub fn advance(&mut self, count: usize) {
        for _ in 0..count {
            if self.is_done() {
                return;
            }
            self.position = self.settle(self.position + 1);
            self.consumed += 1;
        }
    }

    /// True when every block of the step is consumed.
    pub fn is_done(&self) -> bool {
        self.position >= self.range.end
    }

    /// The current position (for coverage and diagnostics).
    pub fn position(&self) -> usize {
        self.position
    }

    /// How many blocks have been consumed so far.
    pub fn consumed(&self) -> usize {
        self.consumed
    }
}

/// The outcome of one recognizer at one position.
pub enum Recognized {
    /// Not this recognizer's idiom; the cursor must be unchanged.
    No,
    /// Matched: the cursor is advanced; these actions are appended (may be empty, e.g. the
    /// absorbed patch ceremony).
    Yes(Vec<Action>),
    /// The idiom matched but is malformed (lead sentence without its fence, a patch whose
    /// paths disagree with the lead): a parse error, never a fallback to `Manual`.
    Malformed(Diagnostic),
    /// Matched an earlier action instead of producing one (an `Expected` block): replace the
    /// last action with this one.
    Amend(Action),
}

/// One configured matcher. Must be pure: no I/O, no state between calls.
pub trait Recognizer: Send + Sync {
    /// Instance id for coverage and `planrun show` (`file-block#full-contents`).
    fn id(&self) -> &str;

    /// Tries to match at the cursor.
    fn recognize(&self, cx: &StepContext<'_>, cursor: &mut Cursor<'_>) -> Recognized;
}

/// Builds recognizer instances of one kind from the `[[recognizer]]` tables of a format spec.
/// Registered by kind name in the [`crate::Registry`]; this is the code-level plugin point.
pub trait RecognizerFactory: Send + Sync {
    /// The kind name used in specs (`file-block`).
    fn kind(&self) -> &'static str;

    /// One instance from its spec table (`params` excludes `kind`). Invalid params are an
    /// error when the registry loads the format, never at parse time.
    fn build(&self, params: &toml::Table) -> Result<Arc<dyn Recognizer>, String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source;

    #[test]
    fn the_cursor_never_sees_the_skipped_blocks() {
        let text = "a\n\nb\n\nc\n\nd\n";
        let source = source::load("t", text.as_bytes()).unwrap_or_else(|e| panic!("{e:?}"));
        let doc = Document::build(source).unwrap_or_else(|e| panic!("{e:?}"));
        let all = BlockRange { start: 0, end: 4 };
        let skip = Some(BlockRange { start: 1, end: 3 });
        let mut cursor = Cursor::with_skip(&doc, all, skip);
        let at = |c: &Cursor<'_>, k| c.peek(k).map(|b| doc.source.slice(b.span).to_owned());
        assert_eq!(at(&cursor, 0).as_deref(), Some("a"));
        assert_eq!(at(&cursor, 1).as_deref(), Some("d"));
        assert_eq!(at(&cursor, 2), None);
        cursor.advance(1);
        assert_eq!(cursor.position(), 3);
        cursor.advance(5);
        assert!(cursor.is_done());
        assert_eq!(cursor.consumed(), 2);

        let from_skip = Cursor::with_skip(&doc, BlockRange { start: 1, end: 4 }, skip);
        assert_eq!(from_skip.position(), 3);
    }
}
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

File `docs/decisions.md` (full contents):

```markdown
# Decisions

Numbered, never renumbered. A changed decision gets a new number that supersedes the old one;
the old entry stays with a "superseded by" line. Code and docs cite them as `D<n>`.

Agreed between Rens and Claude on 2026-11-24, from two proposals (Rens' "Deterministic Plan
Execution CLI" and Claude's pattern study of six real plans).

## D1 Priorities, and what planrun is

Correctness > execution speed > low memory > predictability > maintainability/extensibility.
planrun is not an agent: it does exactly what the plan says, shows what it cannot do
mechanically, and decides nothing. Rust.

Order superseded by D25.

## D2 Four crates; parsing never executes, execution never parses

`planrun-model` (IR), `planrun-parse` (depends on model), `planrun-exec` (depends on model, **not**
on parse), `planrun-cli`. The crate graph enforces the separation.

## D3 A plan runs only when it parsed and checked without errors

Parse the whole document first; any error means no `Plan`. No partial execution.

## D4 The action set is closed

`WriteFile`, `ApplyPatch`, `ReplaceExact`, `Delete`, `Generated`, `RunScript`, `Commit`, `Note`,
`Manual`. Formats vary, actions don't. Adding one is a cross-cutting change (extending.md §4).

## D5 Phases are metadata

One plan file is one phase in every real plan. `PlanMeta::phase`, not a tree level.

## D6 Normalize the plan once

At load: strip a BOM, CRLF/CR → LF. Every span, fence body and line number refers to the
normalized text.

## D7 RepoPath is the only path in an action

Relative, forward slashes, no `..`/`.`/empty segments, no drive letter, no backslash, no NUL.
Symlink escapes are refused by the workspace at write time. Nothing outside the repository is
written except under `.git/planrun/` and the OS temp directory.

## D8 Formats are data first; built-ins are specs too

Three extension levels: TOML spec (no rebuild) → recognizer kind (Rust) → Format (Rust). The
built-in formats are TOML specs embedded in the binary, so user formats take the same path.

## D9 Dependencies

Each dependency is justified here; add none without a line.

| Crate | Why |
|---|---|
| `pulldown-cmark` | CommonMark with byte offsets; correct nested-fence handling |
| `regex` | spec patterns; linear time, no backtracking blow-ups |
| `serde`, `toml` | format specs |
| `serde_json` | the journal (JSONL) |
| `thiserror` | error types |
| `sha1_smol` | git blob ids in-process: the `blob-chain` check from a created file's contents (planrun-parse); the fast path in planrun-exec, where `git hash-object` is the authority |
| `similar` | showing diffs of overwrites and replacements. **Not** for applying patches (D20) |
| `clap` | arguments |
| `dialoguer` | line prompts (D15) |
| `annotate-snippets` | diagnostics with source context |
| `insta` (dev) | golden snapshots of the IR |
| `sha2` (Phase 1) | the plan fingerprint (SHA-256) |

Rejected: `rusqlite` (D18), `ratatui` for now (D15), `tracing` for now (the journal is the record),
`similar` as a patch engine (D20).

## D10 Line endings of written files

Created files: as in the plan (LF). Overwrite and `ReplaceExact`: keep the target's existing
convention unless `.gitattributes` says `eol=lf`; `before` is matched in LF space. Patches: through
`git apply` unchanged.

## D11 Prose never passes a script on its own

`Expected: all pass` is mapped to an exit expectation, checked, and then shown for confirmation.
Only fenced summary lines and exit codes pass automatically.

## D12 Expected failures are skipped by default

A verification expected to fail (TDD "verify it fails", `Expected: build error`) is recorded in
the IR and skipped; `--run-expected-failures` runs it. Rens sees these failures in the IDE.

## D13 Policy classes

Read < Build < Modify < Destructive; a script takes the highest class of its commands; unknown
commands are Destructive. Defaults: file actions automatic; Build/Modify ask; Destructive needs a
typed `yes`; Manual always waits.

## D14 Resume checks the disk, never the journal alone

An action recorded `Running` or `Done` is re-examined: postimages present → done; preimages
present → not done; otherwise the user decides.

## D15 Line prompts first

`dialoguer` and plain output keep cargo's output in the terminal's scrollback. A ratatui TUI can
come later behind the `Ui` trait.

## D16 Real plans stay local

The six real plans contain customer and internal details. They live in `tests/corpus/local/`
(git-ignored) and are parsed by an `#[ignore]`d test. Committed tests use synthetic plans.

## D17 The patch ceremony is absorbed

The `git apply "$env:TEMP\….patch"` / `Remove-Item` block after an exact diff is never run:
planrun applies the patch itself. There is no `--verbatim` mode.

## D18 The journal is JSONL in `.git/planrun/`

One writer, sequential, a few hundred records per plan: SQLite adds a C dependency and a binary
file nobody can read when it matters. JSONL is appendable, crash-tolerant (a torn last line is
dropped), and readable. Inside `.git/` so `git status --porcelain` stays empty.

## D19 Windows first

Every real plan is PowerShell 7 on Windows. Linux (pwsh and bash) is supported and is where CI
can run.

## D20 Patches are applied by `git apply`

With the patch on stdin, `--whitespace=nowarn`, after `--check`. Same semantics as Rens' manual
runs; exact context; no fuzz. Blob ids from `index` lines are verified before and after.

## D21 Note vs Manual

Prose no recognizer consumed is a `Note` (shown, never blocks). An unconsumed fence, prose that
matches a format's manual markers, and edits described in prose are `Manual` (always blocks).

## D22 Commits are gated twice

The plan's commit script runs as written (it may gate itself on `$gate`), and planrun runs it only
when every verification of the task passed or was skipped by policy (D12).

## D23 File actions are transactional per step

All file actions of a step, up to the next non-file action, are validated in memory before any is
written; pre-images are stored in the git object database first; writes are temp-file + rename.

## D24 One persistent session per shell kind per run

Variables and the working directory persist across blocks (`$gate`, `$P2`), as in Rens' terminal.

## D25 Maintainability/extensibility ranks second

Supersedes D1's order; the rest of D1 (not an agent, Rust) stands.
Correctness > maintainability/extensibility > execution speed > low memory > predictability.
Reason: new plan styles keep arriving, so planrun is only valuable if adding one stays cheap
and safe; speed and memory rarely decide anything, because cargo, npm and dotnet dominate
wall time.

Decisions D26–D40 were agreed on 2026-11-24 at the start of P2, from the review of the three
hand-written formats against their real plans.

## D26 The file operation decides Create or Overwrite

A lead that says create writes with `Create`, one that says replace writes with `Overwrite`,
whatever the manifest says: Create refuses an existing target at preflight (P3), so the op
deciding is safe. Only the `write` op takes the mode from the manifest. Where a task has a
manifest and its entry disagrees with the op (Create on a Modify/Rewrite/Delete entry, Overwrite
on a Create/Delete entry), `manifest-write-mode` is an error, in every format.

## D27 Optional manifests; `Generated` entries

`[outline.manifest] required` (default true) lets a format have tasks without a Files list; the
manifest checks skip such a task. The marker may capture `change` for formats whose items are
bare paths. `ChangeKind::Generated` is a manifest change of its own (a model variant, not an
action kind, so D4 does not apply): a generated entry needs no action, and `show` reports it
as generated.

## D28 A shell fence runs only behind a run lead in hand-written formats

`script` takes `leads` (regexes over the paragraph directly before the fence) and `first_block`
(the fence is the first block of a written step). With `leads` set, the kind claims a fence only
when one of the two holds; any other shell fence falls to the fallback, `Manual(Unrecognized)`.
`first_block` never applies to the implicit first step, so never to the block right after a
manifest. A body intent (`git commit` in the body) never qualifies a fence on its own: a plan
can create a script file whose contents contain git commands. Reason: hand-written plans put
file contents and code to insert in shell fences behind prose that no recognizer can reliably
name; unguarded, `script` would execute them. The assay spec sets no `leads` and is unchanged.

## D29 Script intents by lead; no `preview-apply` kind

An `intents` row of `script` matches the body, the lead paragraph, or both (`body`, `lead`).
`leads` decides whether the kind claims a fence; the per-intent `lead` decides which intent a
claimed fence gets. Preview/apply pairs need nothing more, so the `preview-apply` kind is
dropped.

## D30 Step marks replace the `expected-failure-step` kind

A recognizer cannot see a step's title, so the title is matched by the outline:
`[[step_marks]] title = '…', mark = "expected-failure" | "human-check"`.

- `expected-failure`: every `Verify` script of the step (never a commit or a gate) expects
  failure. A script with no expectation or with `any` becomes `Failure`; one that expects success
  is an error, `expected-conflict`.
- `human-check`: every prose `Note` of the step (a paragraph, list item or quote the fallback
  takes) becomes `Manual(HumanCheck)`; rules and tables stay Notes.

## D31 A spec may downgrade only the manifest coverage checks

`[checks]` maps a check code to `error`, `warning` or `off`, and only `manifest-untouched` and
`manifest-undeclared` may appear in it. Any other key is a spec load error, so a user spec can
never switch off a safety check: `manifest-write-mode` and every outline and recognizer error
stay errors in every format. Reason: in hand-written plans, prose edits (Manual) and scripts
(`replace`) change declared files, and manifests name ranges and globs.

## D32 Paths resolved from the manifest

Deterministic and only against the task's own manifest:

- In a manifest item, a `:lines` token adds lines to the path before it (every format).
- `[outline.manifest] siblings = true`: a token without `/` after a token with a directory is a
  file in that directory. Opt-in, because `crates/x/src/a.rs`, `Cargo.toml` in one item would
  otherwise mean `crates/x/src/Cargo.toml`.
- A recognizer parameter `file_name_from_manifest`: a lead path without `/` that is not itself
  an entry resolves to the one entry with that file name, when that entry was written in full
  (not a sibling). Otherwise the path stays as written.
- `before-after` parameter `infer_path`: a lead with no path takes the task's one entry that is
  not Test, Generated or Delete. Never for writes. With none or several, the lead and its fences
  are `Manual(ProseEdit)`.

Each resolved path carries its `PathOrigin`, and `show` marks it.

## D33 The DevReady `**Commit:** \`msg\`` line is Manual

`Manual(HumanCheck)` carrying the message. planrun synthesizes no commit and never runs
`git add -A`. P5 shows `git add -- <paths>; git commit -m "<msg>"` as a suggestion only when every
manifest path of the task is literal (no glob, range, sibling or unresolved file name);
otherwise it shows the message alone.

## D34 Inline edits are Manual for now

`Edit: \`a\` → \`b\``, `Old: \`a\`` / `New: \`b\`` and similar one-line edits become
`Manual(ProseEdit)`: code spans lose whitespace. Revisit after dogfooding.

## D35 Comment expectations in DevReady scripts

A `script` parameter `line_expect` (a regex over each line) turns trailing comments such as
`# feature/p2-tooling` into one prose expectation per fence: exit `Unknown`, confirmed (D11).
DevReady only. A fence that also has an `Expected:` after it is `expected-twice`.

## D36 Imperative prose

Per-format `manual_markers` and `human-check` step marks catch the instructions that would
otherwise be Notes (D21). Each phase plan lists, by file, task and step, the imperative Notes
that remain in the real plans, so every slip is a recorded choice.

## D37 Outline fixes

- A manifest's items end at the first step boundary: a list-item step right after the manifest
  is in the same CommonMark list, and is a step, not an item.
- A heading that matches the task rule is a task; only a heading that is no task, at or above
  `tasks_end_level`, ends the tasks. A task heading after that end is an error, `task-after-end`:
  it would otherwise be text that never runs.

## D38 An expectation after a Manual joins it

An `Expected: …` sentence directly after a `Manual` (an `sql` fence, a fence no lead claimed)
becomes part of that Manual's text, instead of `expected-without-script`. After a `Note` it stays
an error.

## D39 `run-line` names its shell

Each `run-line` recognizer has a `shell` parameter; there is no format default. pwsh for the
superpowers format: its run lines share no variables with its bash commit fences (D24).

## D40 Target for DevReady plans

They are written for an agent (probes, conditional branches, "record"). The target is zero errors
and honest Manual coverage, not automation. P2 is split: P2a builds the schema, the kinds and the
superpowers and before-after formats; P2b the devready format on the same schema.
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
# a `:3-10` piece adds lines to the path before it (D32)
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
| Manifest | `**Files:**` + `Create:`, `Modify:` (with `:lines`), `Rewrite:`, `Test:`; several paths per item, separated by `, `, ` and ` or `–`; a bare file name after a path is in that path's directory and a `:lines` token adds lines to the path before it (D32); a trailing `(note)`. Optional: a task may have none |
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
`manifest-undeclared` is a warning (items name ranges such as `0001`–`0007_*.sql`).

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
# - Modify: `a/r.resx`, `r.en.resx` (lines 227-230)   - Modify: `a/0001`–`0007_*.sql` (a range: kept as written)
item = '^(?P<change>Create|Modify|Rewrite|Test): `(?P<paths>[^`]+(?:`(?:, | and |–)`[^`]+)*)`(?: \(.+\))?$'
changes = { Create = "create", Modify = "modify", Rewrite = "rewrite", Test = "test" }
path_list = '`(?:, | and |–)`'
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
| `devready-style.md` | `devready` | `Task 0`, prose Files line, `` `path`: `` + block creates, `**Verify:**` with `# expected` comments, `**Commit:** \`msg\``, `**2a.**` labels, `edit 1 (old → new)`, four-backtick fences, `git rm`, bash + powershell in one task, conventions with a manual marker |
| `before-after-style.md` | `before-after` | `## Task n:`, file map table, `Rewrite:`, `` `path`: `` create, `— replace the whole file:`, `before:`/`After:`, expected failure (prose), prose insertion (Manual), `no matches`, preview/apply pair, output-dependent edit, `sql` block |

Golden snapshots of the parsed IR live next to the tests that produce them (`insta`, in
`crates/planrun-cli/src/snapshots/`): P1 for `assay-style.md`, P2a for `superpowers-style.md`
and `before-after-style.md`, P2b for `devready-style.md`. A snapshot change is a review item, never a re-pin by
reflex.
```

**Step 2 — verify**

Targeted: the synthetic before-after plan:

```powershell
cargo run -q -p planrun-cli -- check tests/corpus/synthetic/before-after-style.md
```

Expected (exit code 0; summary lines):

```text
planrun: tests/corpus/synthetic/before-after-style.md: ok (before-after, 4 tasks)
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
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-cli/src/snapshots/planrun__view__tests__before_after_style_ir.snap crates/planrun-cli/src/view.rs crates/planrun-parse/src/recognize.rs crates/planrun-parse/src/recognizer.rs crates/planrun-parse/src/registry.rs docs/decisions.md docs/extending.md docs/plan-grammar.md formats/before-after.toml tests/corpus/synthetic/README.md
  $verified = [ordered]@{
    'crates/planrun-cli/src/snapshots/planrun__view__tests__before_after_style_ir.snap' = '4a7477445a8aff509e5c6ce3555272f2ccb1a08a'
    'crates/planrun-cli/src/view.rs' = 'f77a9f0b3ea8b6ce0f0a09993310097a03e4bbf9'
    'crates/planrun-parse/src/recognize.rs' = 'e76f6a188314b7d2ef8acca0d1db5576802b330a'
    'crates/planrun-parse/src/recognizer.rs' = '3eee7888438cb61248883b86c4e2ca08e9a1a6bd'
    'crates/planrun-parse/src/registry.rs' = '1bd39cd4eb832a375ef9521f5c8315a51546acea'
    'docs/decisions.md' = '24772399ab8acf3018d1dff7009b028919068ed5'
    'docs/extending.md' = '9373948c30215a744838b63da047656b5e4e95d3'
    'docs/plan-grammar.md' = '3a8a70c052304bcf396324bbc00daf18b5d10bca'
    'formats/before-after.toml' = 'df90fe32438ee39d5eefebde6c0cf6999b4b0f0e'
    'tests/corpus/synthetic/README.md' = 'd0177ae05fd4685c8679ebc116be312d2316c321'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P2a-T6: the before-after format" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P2a-T7: architecture, CLAUDE.md layout, local corpus report

**Goal:** docs/architecture.md §4.3–§4.5 describe the P2a outline, recognition and check levels; the CLAUDE.md Layout section lists the parse modules and the formats (review answer C13); the local-corpus test prints each plan's diagnostics.

**Files**

- Modify `CLAUDE.md`
- Modify `crates/planrun-parse/tests/local_corpus.rs`
- Modify `docs/architecture.md`

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
                       AbbrevBlobId
crates/planrun-parse   Markdown → IR: source, document, format, spec, registry, outline,
                       recognizer, recognize, kinds/ (one file per recognizer kind), patch, check,
                       diagnostic
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

File `crates/planrun-parse/tests/local_corpus.rs` (full contents):

```rust
//! The real plans in `tests/corpus/local` (git-ignored, decision D16). Ignored by default, so CI
//! and fresh clones stay green: `cargo test -- --ignored local_corpus`.
//!
//! Every plan must parse with zero errors, except plans no built-in format detects yet (the
//! devready plans until P2b), which are listed as skipped. A coverage line per task and the
//! warnings are printed; run with `--nocapture` to see them.

use std::path::Path;

use planrun_parse::{Registry, parse_plan};

#[test]
#[ignore = "needs the private plans in tests/corpus/local"]
fn local_corpus() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/local");
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap_or_else(|e| panic!("{e}")).path())
        .filter(|p| p.extension().is_some_and(|x| x == "md") && !p.ends_with("README.md"))
        .collect();
    paths.sort();
    let mut failed = Vec::new();
    let mut parsed = 0;
    for path in &paths {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let result = match parse_plan(&registry, name, &bytes, None) {
            Ok(result) => result,
            Err(d) if d.error_codes() == ["no-format"] => {
                println!("{name}: skipped, no built-in format detects it yet");
                continue;
            }
            Err(d) => {
                failed.push(format!("{name}: {:?}", d.error_codes()));
                continue;
            }
        };
        parsed += 1;
        let warnings: Vec<&str> = result.diagnostics.0.iter().map(|d| d.code).collect();
        println!(
            "{name}: {} ({} tasks), diagnostics {warnings:?}",
            result.plan.format.0,
            result.plan.tasks.len()
        );
        for c in &result.coverage {
            println!(
                "  {:<12} blocks {:>3}  actions {:>3}  absorbed {:>2}  manifest {:>2}  notes {:>2}  manual {:>2}",
                c.task.raw,
                c.blocks(),
                c.actions,
                c.absorbed.len(),
                c.manifest,
                c.notes,
                c.manual
            );
        }
    }
    assert!(failed.is_empty(), "\n{}", failed.join("\n"));
    println!("{parsed} parsed, {} skipped", paths.len() - parsed);
}
```

File `docs/architecture.md` (exact diff; save the block as `$env:TEMP\p2a-t7-architecture-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/architecture.md b/docs/architecture.md
index 0e60a40..034b4f5 100644
--- a/docs/architecture.md
+++ b/docs/architecture.md
@@ -140,14 +140,17 @@ guess.
 
 The format finds: plan metadata, tasks (heading pattern, `ord` capture for ordering), each task's
 manifest (marker + item pattern), and step boundaries. The manifest's blocks are consumed by the
-outline: they fall in no step, and coverage counts them as "manifest". A format with a manifest
-pattern requires one per task (`manifest-missing`); an item that does not match is
-`manifest-item`. Blocks between a task heading and its first step form an unnumbered **intro
+outline: they fall in no step, and coverage counts them as "manifest"; its items end at the first
+step boundary (D37). A format with a manifest pattern requires one per task (`manifest-missing`)
+unless it sets `required = false` (D27); an item that does not match is `manifest-item`. Paths in
+items may be completed from the item itself (`:lines` tokens, and sibling file names where the
+format opts in, D32). Blocks between a task heading and its first step form an unnumbered **intro
 step** (it can hold a `Generated` notice or `Manual` text). A heading at the task level inside the
-tasks that is not a task is `bad-task-heading`. A task
-with no step boundaries is one implicit step. Text after the tasks (a heading at or above
-`tasks_end_level`) is not executed: `show` lists it; `check` reports `Manual` markers in it as
-notes.
+tasks that is not a task is `bad-task-heading`. A task with no step boundaries is one implicit
+step. A step title can carry a **mark** (`[[step_marks]]`, D30): expected failure or human check.
+Text after the tasks (a heading that is no task, at or above `tasks_end_level`) is not executed:
+`show` lists it; `check` reports `Manual` markers in it as notes. A task heading after that point
+is `task-after-end` (D37).
 
 ### 4.4 Recognition
 
@@ -160,13 +163,17 @@ tried **in order**; the first that answers anything but `No` wins:
   whose paths disagree with its lead). This is a **parse error, never a fallback** to Manual.
 
 If no recognizer matches, the built-in fallback consumes one block: a paragraph or list item
-becomes `Note` (or `Manual(Marked)` on a manual marker); a fence becomes `Manual(Unrecognized)`, or
+becomes `Note` (or `Manual(Marked)` on a manual marker, `Manual(HumanCheck)` in a human-check
+step); a fence becomes `Manual(Unrecognized)`, or
 `Manual(ForeignLanguage)` for `sql` and other non-shell languages, and a `text` fence becomes
 `Note`; tables, blockquotes (or `Manual(Marked)` on a marker), headings and rules become `Note`;
 HTML comments are dropped (they are corpus metadata).
 
 Recognizers are pure (no I/O, no state between calls) and see a `StepContext`: the document, the
-task id, the manifest, the format settings, and the step's previous actions.
+task id, the manifest, the format settings, the step's previous actions, whether the step starts
+at a written boundary, and its mark. The cursor also shows the block before the current one, so
+`script` can require a run lead (D28). After recognition, an expected-failure step's verify
+scripts expect failure (`expected-conflict` when one says success).
 
 ### 4.5 Checks
 
@@ -177,7 +184,9 @@ checks only. A check that needs the repository (the first preimage against the f
 preflight in planrun-exec.
 
 Rules run only when the outline and recognition had no errors, so a broken block is reported
-once, not again as a missing manifest touch.
+once, not again as a missing manifest touch. A format's `[checks]` table may make
+`manifest-untouched` and `manifest-undeclared` warnings or switch them off, and nothing else
+(D31).
 
 The **coverage report** is part of `check`, not a rule: per task, how many blocks became actions,
 were absorbed (the patch ceremony, listed with their positions), were the manifest, or became
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p2a-t7-architecture-md.patch"
Remove-Item "$env:TEMP\p2a-t7-architecture-md.patch"
```

**Step 2 — verify**

Targeted: the local corpus (the private plans in `tests/corpus/local/`):

```powershell
cargo test -p planrun-parse --test local_corpus -- --ignored local_corpus
```

Expected (exit code 0; summary lines):

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- CLAUDE.md crates/planrun-parse/tests/local_corpus.rs docs/architecture.md
  $verified = [ordered]@{
    'CLAUDE.md' = 'bf065d7d9f9a2b14f5e488d1b082af900e75c74f'
    'crates/planrun-parse/tests/local_corpus.rs' = 'c32a9e2dc9b38fd01246955f2fda1c414f324622'
    'docs/architecture.md' = '034b4f527554e982f7ac96717d8cd66c3fd57a53'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P2a-T7: architecture, CLAUDE.md layout, local corpus report" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

## P2a green state

Run from the repository root after P2a-T7:

```powershell
cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-parse --test local_corpus -- --ignored local_corpus }
```

Expected: every `test result:` line ok; the last one `1 passed`.

User-visible change: `planrun check` and `planrun show` work on superpowers and before-after plans; devready plans still say `no-format` until P2b.

## After execution

- `git log --oneline -8` lists the commit of this plan, then `P2a-T1` to `P2a-T7`.
- `cargo run -q -p planrun-cli -- check docs/plans/2026-11-24-p2a-hand-written-formats.md` exits 0: this plan is an assay plan.
- `cargo test -p planrun-parse --test local_corpus -- --ignored local_corpus --nocapture` prints the heatmap plan as superpowers with no diagnostics and the cleanup plan as before-after with one `manifest-undeclared` warning; the two devready plans are skipped.
