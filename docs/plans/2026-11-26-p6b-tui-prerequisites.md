# planrun P6b — The TUI's prerequisites

**Goal:** Everything the TUI (P6c) needs before its own crate exists, and nothing of the TUI itself. First the design and its decisions: `docs/tui-design.md` (the reviewed spec, r4, with this plan's review rulings), D84–D92 in docs/decisions.md amending D15, and P6b and P6c in the roadmap. Then the changes in planrun-exec that the design's §9 lists, each with its tests: a journal with any number of observers, `shell::end_now`, typed effects in the dry-run report, the matcher's alignment, `QuestionKind` with explicit typed words, a liveness lock, and the read-only resume preview. Last, the architecture, extending and CLAUDE.md lines for them. The CLI's behaviour, its output and the journal format do not change. Spec: the r4 TUI design, which P6b-T1 writes as `docs/tui-design.md`.

- **P6b-T1** (docs): `docs/tui-design.md`, D84–D92 with amended-by lines in D9, D15, D73, D74, D76 and D77, and the roadmap's P6b and P6c.
- **P6b-T2** (planrun-exec): `Journal::observe` adds an observer; every observer sees every seam (§9 item 1, D90).
- **P6b-T3** (planrun-exec): `shell::end_now`, Ctrl-C for a UI that reads it as a key (§9 item 5, D91).
- **P6b-T4** (planrun-exec): `handler::Effect` in the dry-run report's `Outcome::Ok` and `WouldAsk` (§9 item 6, D84).
- **P6b-T5** (planrun-exec): `expect::align`, the expected lines against the output (§9 item 3, D86).
- **P6b-T6** (planrun-exec, planrun-cli): `QuestionKind`, the question with what the run found, and `Question.typed` as the words a typed question accepts (§9 item 2, D86); the two broken `resume-dirt` strings of the P5→P6 handover's item 5.
- **P6b-T7** (planrun-exec): `lock.live` and `lock::live`, a live run seen without trying the run lock (§9 item 7, D90).
- **P6b-T8** (planrun-exec): `runner::preview::preview`, what resume would find, read only, through the judgements resume itself uses (§9 item 4, D85).
- **P6b-T9** (docs): architecture.md §6.3, §6.4, §6.6 and §7, extending.md §7, CLAUDE.md's layout.

**Base:** `1abe6f5` (`Properly name the autocrlf value`); there fmt, clippy (with and without `e2e`), `cargo test` with `PLANRUN_REQUIRE_SHELLS=1` (27 `test result: ok` lines, 280 tests) and the `e2e` test pass. After P6b-T9: 27 lines, 291 tests. Execute from the commit of this plan. Gate per task: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings`, `cargo test`, `cargo test -p planrun-cli --features e2e --test run`, with `PLANRUN_REQUIRE_SHELLS=1` (D82).

**Shell and environment:** PowerShell 7 on Windows; Rust 1.95.0 (stable; `File::lock` and `File::try_lock` need 1.89); git on `PATH`; pwsh 7 and Git for Windows (D63). No new crate: D92 records the TUI's crates for P6c, and this plan adds none.

**Before P6b-T1:** this plan committed, and a clean `git status --porcelain`; the untracked spec copy `docs/superpowers/specs/2026-11-26-tui-design.md` is gone (P6b-T1 writes the design as `docs/tui-design.md`, as the spec's §11 says, and an untracked file outside a task's paths is asked about before every commit, D80). To run the plan with planrun itself, run a copy outside `target/`, since every gate builds `target/debug/planrun.exe`: `cargo build --release`, `Copy-Item target/release/planrun.exe $env:TEMP\planrun-p6b.exe`, then `& $env:TEMP\planrun-p6b.exe run docs/plans/2026-11-26-p6b-tui-prerequisites.md --auto read,build,modify`. Every commit asks to confirm its prose expectation: **answer `y`, not `q`** (the P5→P6 handover's item 10). The real plans stay in `tests/corpus/local/` (D16); this plan quotes none of them, and it changes nothing in planrun-parse.

**How this plan was verified:** every task was built in a scratch clone of `1abe6f5` outside this repository (`core.autocrlf=false`), one commit per task. At each commit the gate passes, and so do the task's targeted commands; the summary lines below come from those runs. Also at each commit: `cargo clippy -p planrun-exec --all-targets --target x86_64-unknown-linux-gnu -- -D warnings`, for the Unix half of `end_now` (P6b-T3), which this machine compiles but cannot run. The plan was generated from that history (full contents from `git show`, exact diffs from `git diff`, blob ids from `git rev-parse`), `planrun check` passes on it, and it was replayed by planrun on a fresh clone with `core.autocrlf=true`: every commit step committed, `git status --porcelain` was empty after each, and the replay's final tree is the scratch tree.

## Review rulings

From the designer's review of the first version of this plan (`e7be88a`), as decided by Rens; each is in the tasks below.

1. **R1, a liveness lock** (P6b-T7, D90 amending D77). A run takes `lock` as before (a try), then `lock.live` with a blocking `File::lock`, and holds both; `lock::live` is one try on `lock.live`, let go at once. A probe never makes a run fail, only wait; `lock.pid` is shown, never trusted. Taking `lock` first is what keeps a second run failing at once with "locked" rather than waiting for the first run to end; the cost is one gap, stated in D90: a probe in the instant between a starting run's two locks sees no live run. Its tests: live while a run holds it, not after; the probe creates nothing; a run that starts while a probe holds `lock.live` waits and succeeds.
2. **R2, typed answers are explicit** (P6b-T6). `Question.typed` is `Option<&'static [&'static str]>`: `RUN_SKIP_OR_QUIT` (`yes`, `skip`, `quit`) for a Destructive script, `YES_OR_QUIT` for the commit gate and for accepting a step as done, `None` for a question of choices. `typed_yes` reads `skip` from that list instead of taking a flag, so the words shown and the words acted on are one list. `LineUi`'s output does not change. Test: `a_typed_question_lists_the_words_it_accepts`.
3. **R3, the mismatch carries its output** (P6b-T6): `QuestionKind::Mismatch::output` is the block's `OutLine`s, which `Aligned::output` indexes; the payload test checks the matched line's text.
4. **R4, a complete resume preview** (P6b-T8). Its HEAD verdict is `resume::head_verdict`, now the function `check_head` itself calls (`Unchanged`, `CommitMade`, `MovedByRunning` with the action, its kind and the HEAD before it, `Moved`); the scripts that ask again come from `again`, the rule `replayed` now calls; the `resume-dirt` paths come from `preflight_resumed` itself, which is read only (`git status --no-optional-locks` and the virtual tree), fed by `resumed_for`, which `replay_steps` now calls; `live` is R1's probe. To share them, `commit_problem` became a free function over the workspace. **One change to the ruling:** an Unknown step counts as done for the `resume-dirt` list, not as not done. Its paths are never asked about by resume: `[r]` puts them back as they were, and `[a]` verifies them. Counting it as not done would list paths that resume never asks about (the first preview test checks that the list is empty after a crash between two writes). Both preview tests check that the repository (`.git` included) and the journal are unchanged.
5. **S1, resume's questions name their place** (P6b-T6): `unknown` and `accept-done` carry the step's journal `At`. **Extended to `head-moved`**, which had `at: None` too and did not say which action moved HEAD: it carries the action's `At`. Test: the `unknown` question in `a_crash_between_two_writes_is_unknown_and_restore_completes_it`.
6. **S2** (P6b-T1): `docs/tui-design.md` §1 says what D89 says, 24-bit only in v1, with the 256-colour fallback and `NO_COLOR` designed in §4 and deferred (§11). P6b-T1 also brings §2 (the lock and preview rows, the background work), §3 (the header's other runs), §6.E, §6.F and §9 (item 7) in line with R1 and R4.
7. **M1:** the `expect_tests.rs` move goes into the P6b→P6c handover (After execution, step 3).
8. **M2:** `end_now_child`'s doc comment says it returns at once, and counts as a passed test, in every run but its own.

## Stated decisions made while building

1. **D84–D92 are the spec's TD1–TD9, in order** (TD*n* is D(83+*n*)), so a citation in the design and in the code reads the same. D84 also records the typed dry-run report (§9 item 6), D86 the matcher's alignment (§9 item 3) and D90 the liveness lock (§9 item 7, added by the review). `docs/tui-design.md` is the spec with its TD numbers replaced, its status line updated and the review's changes (ruling 6).
2. **§9's order is not the plan's.** `QuestionKind` comes after the alignment, because its `mismatch` variant carries it; the liveness lock comes before the preview, which uses it; the observers, `end_now` and the typed report come first, as the smallest changes.
3. **A `QuestionKind` payload holds what the run found, never what the plan says.** The plan's facts (the action, its path, its `PathOrigin`, a Manual's reason and text) are reached through `Question.at`, so `unscoped` and `manual` have no payload. `Question.text` stays, so `LineUi` and the line prompts' output do not change.
4. **`expect::align` is a function of its own, not a field of `Verdict`.** `check` and its callers stay as they are; the runner calls `align` only when there is a mismatch, and the expected block is matched a second time then (a few lines against the output already in memory). `check` and `align` share one matcher (`aligned`), so they cannot disagree.
5. **The session notice keeps its reason and the earlier tasks as they were when it was set** (`Notice` in runner/mod.rs), so the `session` question's payload and its text say the same thing even if a task ends between the two.
6. **`Simulated.detail: String` became `Simulated.effect: Effect`** (handler.rs), since `Outcome::Ok` took the handler's text. `Effect`'s `Display` prints the old text, so `planrun run --dry-run` prints what it printed (its snapshot is unchanged), and so does `tests/simulate.rs`.
7. **The P5→P6 handover's item 5 is fixed in P6b-T6**, because P6b-T6 rewrites one of the two strings: both `resume-dirt` strings get their `\` line continuation back. A search of the source for runs of three or more spaces inside string literals found no other (a table header in render.rs and test inputs in planrun-parse are meant).
8. **`end_now`'s test runs it in a child process**: the test binary starts itself with an environment variable set, and the child test (`end_now_child`) starts a pwsh session whose block starts a grandchild, then calls `end_now(7)`. The parent checks the exit code and that the grandchild is gone. On Windows that tests the job objects; the Unix path (`kill_registered`) is compiled and linted for Linux here, and runs in CI on `ubuntu-latest`.
9. **The preview is `runner/preview.rs`**, not part of resume.rs, and holds no judgement of its own: where resume judges, it calls the same function (ruling 4). `done::temps` is the read-only half of `remove_temps`.
10. **expect.rs passes 400 lines** (465) with its new test. CLAUDE.md moves tests to `<module>_tests.rs` "once a file passes ~400 lines"; that move is mechanical and left out of this plan (ruling 7).
11. **The docs come in two tasks**, as in P5: P6b-T1 records the decisions before the code, P6b-T9 describes the code after it.

## Tasks

### P6b-T1: the TUI's design and decisions

**Goal:** The design in the repository and its decisions agreed in writing before any code: `docs/tui-design.md` (the r4 spec, its TD numbers replaced by D84–D92, and the review's changes to §1, §2, §3, §6.E, §6.F and §9), D84–D92 at the end of docs/decisions.md, amended-by lines in D15 (the TUI), D9 (the TUI's crates, planrun-tui only), D73 (`end_now`), D74 (`QuestionKind`), D76 (any number of observers) and D77 (`lock.live`), and P6b and P6c in the roadmap.

**Files**

- Modify `docs/decisions.md`
- Modify `docs/roadmap.md`
- Create `docs/tui-design.md`

Over 300 lines, so it comes as an exact diff: `docs/decisions.md`.

**Step 1 — write the files**

File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p6b-t1-docs-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index d9d86c5..eda85c1 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -68,16 +68,19 @@ Each dependency is justified here; add none without a line.
 | `annotate-snippets` | diagnostics with source context |
 | `insta` (dev) | golden snapshots of the IR |
 | `sha2` (Phase 1) | the plan fingerprint (SHA-256) |
+| `ratatui` | the TUI's layout and widgets (planrun-tui only, D92) |
+| `crossterm` | the TUI's terminal: raw mode, the alternate screen, key events (planrun-tui only, D92) |
+| `unicode-width` | the TUI's cell widths, and the test that pins its glyphs to one cell (planrun-tui only, D89) |
 
-Rejected: `rusqlite` (D18), `ratatui` for now (D15), `tracing` for now (the journal is the record),
+Rejected: `rusqlite` (D18), `tracing` for now (the journal is the record),
 `similar` as a patch engine (D20), `windows-sys` and `libc` for the process tree (D64), `dialoguer`
 (D74: it refuses a stdin that is not a terminal), `ctrlc` (D73: it brings `nix` and
 `windows-sys`), `ureq`, `reqwest`, `self_update`, `self-replace`, `zip` and `flate2` for
 `planrun update` (D83: curl and tar do it, like git, with no TLS stack in the build).
 
 Amended by D56 (no `sha1_smol` in planrun-exec), D64 (the hand-declared process-tree calls), D73
-(`ctrlc` rejected), D74 (`dialoguer` dropped) and D83 (`serde_json` in planrun-cli; the update
-crates rejected).
+(`ctrlc` rejected), D74 (`dialoguer` dropped), D83 (`serde_json` in planrun-cli; the update
+crates rejected) and D92 (the TUI's crates, in planrun-tui only).
 
 ## D10 Line endings of written files
 
@@ -121,7 +124,8 @@ Amended by D77 (resume, per action kind).
 `dialoguer` and plain output keep cargo's output in the terminal's scrollback. A ratatui TUI can
 come later behind the `Ui` trait.
 
-Amended by D74 (line prompts on std, without `dialoguer`).
+Amended by D74 (line prompts on std, without `dialoguer`) and D84–D92 (the TUI,
+docs/tui-design.md).
 
 ## D16 Real plans stay local
 
@@ -974,6 +978,8 @@ Amends D64 and D65; D9 rejects `ctrlc`.
   solved.
 - `ctrlc` is rejected (D9): it brings `nix` and `windows-sys`, which D64 already rejected.
 
+Amended by D91 (`shell::end_now`, for a UI that reads Ctrl-C as a key).
+
 ## D74 Line prompts on std
 
 Amends D15; D9 drops `dialoguer`.
@@ -1000,6 +1006,8 @@ Amends D15; D9 drops `dialoguer`.
   (D75), a step changed between prepare and commit (D78), and resume's Unknown step,
   interrupted script and dirt a remaining file action writes over (D77). Each answer is journaled with the question's name.
 
+Amended by D86 (`QuestionKind`).
+
 ## D75 The commit gate
 
 Amends D22. The commit script still runs as written, and may gate itself on `$gate`.
@@ -1055,6 +1063,8 @@ Amends D18; corrects architecture.md §6.6.
 - **An observer** sees every appended record and every path a step's commit wrote; planrun-cli
   uses it for its fault seam (D82).
 
+Amended by D90 (a journal takes any number of observers).
+
 ## D77 Resume, and one planrun per repository
 
 Amends D14 and D57.
@@ -1112,6 +1122,8 @@ Amends D14 and D57.
   `status` and `--dry-run` take no lock. Its limit: on Unix, after a SIGKILL of planrun, a
   session's descendants can outlive the lock (D73).
 
+Amended by D90 (`lock.live`: a live run is seen without trying the lock).
+
 ## D78 Step transactions in a real run
 
 Amends D53. Settles the P3 open items on the gap between prepare and commit, file modes, a git
@@ -1234,3 +1246,141 @@ step needs a token.
 - **Cleanup.** Every planrun start removes `planrun.old[.exe]` and `planrun.new[.exe]` beside its
   executable, best effort and silently: one or two file removals, no process. On Windows the
   `.old` of the update just made stays until the next start, because it is still running.
+
+## D84 The TUI: one plan view, two modes
+
+Amends D15. The design is docs/tui-design.md; D84–D92 are its decisions.
+
+- **`planrun tui`** is a second `Ui` over the same crates, in a crate of its own, `planrun-tui`
+  (ratatui and crossterm), composed by planrun-cli. It shows the domain model and adds no model
+  of its own. Resume, policy and preflight stay authoritative: the TUI explains and asks; it never
+  decides and never turns one decision into another (D74). The line prompts stay (D74).
+- **One plan view, two modes.** Inspect is filled from a dry run (D55); Run from the live runner.
+  Same layout, same tree. Inspect states the policy it assumes (`--auto`,
+  `--run-expected-failures`, what `Policy::new` takes), and `R` runs with exactly that policy.
+  The dry run is advisory: the real preflight decides at run time, so a stale Inspect never
+  authorises anything.
+- **What Inspect reads.** The dry run's report is typed: `Outcome::Ok` and `Outcome::WouldAsk`
+  carry an `Effect` (create, overwrite, a patch with its blob ids, replace, delete), whose
+  `Display` is the text `planrun run --dry-run` prints, so the CLI's output does not change. The
+  class and policy column come from `DefaultClassifier::explain` and `Policy::decide`, both pure,
+  called by the TUI.
+
+## D85 Questions open in a dock; the resume preview only shows
+
+Amends D15.
+
+- A question opens in a dock over the console's region and leaves the plan tree and the change
+  inspector visible. There are no centred modals.
+- **The resume preview** is a read-only full view, built from `analyse` and
+  `runner::preview::preview`: whether a run is live (D90), the plan's fingerprint against the
+  journal's, `.git/index.lock`, resume's own verdict on HEAD (unchanged, moved by the action that
+  was running, or moved so that resume stops; computed by the function resume uses), each
+  prepared step of the interrupted task as it is on disk (Done, NotDone or Unknown, whether it
+  could be restored, the writer's temp files), the scripts that will ask "run again or keep",
+  and the paths resume will ask about as `resume-dirt` (never an Unknown step's: restoring puts
+  them back, accepting verifies them).
+  It takes no lock and writes nothing, and it shows when it was read. Every question in it is labelled "asked when resume reaches it":
+  `Runner::resume` takes the lock, checks everything again against the disk as it is then, and
+  asks in the dock, each answer journaled when asked (D74, D77). Answering in the preview would
+  be a decision of its own, with the runner checking that the state still matches: not now.
+
+## D86 `QuestionKind`
+
+Amends D74.
+
+- `Question.kind` is an enum, `QuestionKind`, one variant per question the runner asks, with what
+  the run knows about it that the plan does not say as its payload: the decision (`run`), the
+  classifier's reasons (`typed-yes`), the matcher's reasons and `errored` (`confirm`), the status,
+  the mismatches, the block's output lines, the expected lines aligned to them, the commit
+  problem and whether the session is dead (`mismatch`), the dirty paths (`dirt`,
+  `resume-dirt`), the shell and directory (`outside`), why a session is new and the earlier
+  tasks (`session`), the gate's reasons (`gate`), the changed paths (`changed`), whether the
+  script was running (`again`), the step's journal place with each path's state and whether it
+  can be restored (`unknown`, and the place for `accept-done`), and the running action's place
+  with both HEADs (`head-moved`). What the plan says is reached through `Question.at`; resume's
+  questions name their step or action by its journal place, since they have no `at`.
+- **Typed answers are explicit.** `Question.typed` is the list of words a typed question
+  accepts (`yes`, `skip`, `quit` for a Destructive script; `yes`, `quit` for the commit gate and
+  for accepting a step as done), and `None` for a question of choices. The runner reads it back
+  to decide what a word does, so the list and the behaviour cannot drift.
+- `QuestionKind::name()` is the journal name (`run`, `typed-yes`, …): the journal format does not
+  change.
+- A UI matches on it exhaustively, so the compiler finds a question it does not show. Its keys
+  and typed input come from `Question.choices` and `Question.typed`; it shows every answer the
+  runner offers and no other.
+- **The matcher's alignment:** `expect::align` returns each non-blank expected line with the
+  output line it matched (or none), the way `expect::check` matches them (D66); `check` is
+  unchanged.
+
+## D87 Phases are not a navigator level
+
+Amends D15. One plan file is one phase (D5): the TUI's Plans view is a table of plan files, and
+the plan view's tree is Task → Step → Action.
+
+## D88 No workspaces yet
+
+Amends D15. `planrun tui [<plans-dir>]` runs from the target repository, like `run`. A list of
+repositories is a later decision.
+
+## D89 Colour and glyphs
+
+Amends D15.
+
+- 24-bit colour, chosen when `COLORTERM` is `truecolor` or `24bit` or `WT_SESSION` is set. The
+  256-colour fallback and `NO_COLOR` (bold for focus, reverse for selection) come after the first
+  version; the colour tokens keep their 256-colour indices for it.
+- Green only after verification, amber only when planrun waits for you (or for a warning), red
+  for failure and the Destructive class, blue for focus and the running action, purple for
+  identifiers. Bold and reverse only.
+- The glyphs are single-width in `unicode-width` (a unit test) and checked by hand in Windows
+  Terminal with Cascadia Mono; each has an ASCII fallback, and `PLANRUN_ASCII=1` switches to them
+  all. No Nerd Font, no wide or emoji glyphs.
+
+## D90 The runner stays synchronous, on a worker thread
+
+Amends D15, D76 and D77.
+
+- The runner runs on a worker thread; the render thread owns the terminal and never waits on git,
+  a shell or the network. The runner reaches the terminal only through blocking round trips:
+  `ask`, `shell`, `view` and `diff`. For `shell` the render thread leaves the alternate screen,
+  turns raw mode off, sets D73's "shell running" flag, runs the shell, restores the terminal and
+  replies; `view` and `diff` are answered in the TUI with the dock left open.
+- **The journal drives the live view.** The TUI's tree, its outcome tags, its change states and
+  its history come from the journal's records (D76), which a journal observer forwards over a
+  channel; there is no event stream beside it. A journal takes any number of observers, called
+  in the order they were added, so the TUI's and the `e2e` seam's (D82) coexist. `Ui::status` is
+  shown as prose and never parsed; `Ui::output` carries each output line with its stream.
+- **Background work** (plan checks, `update --check`, `git status --porcelain
+  --no-optional-locks`, journal scans, the liveness probe) runs on threads; each result carries
+  the fingerprint of its input. Background checks never replace preflight.
+- **A live run is seen through a second lock, `<git-dir>/planrun/lock.live`.** A run takes
+  `lock` as before (a try: held means "another planrun is running", D77), then `lock.live` with a
+  blocking lock, and holds both until it ends; the operating system releases them however it
+  ends. `lock::live` probes with a try on `lock.live` only and lets go at once: a run starting
+  while a probe holds it waits a moment, never fails. A successful try means no run is live;
+  `lock.pid` is then only shown, never trusted (it stays when its run ends). The one gap: a probe
+  in the instant between a starting run's two locks sees no live run. Never probe `lock` itself:
+  a try there would make a starting run fail.
+
+## D91 Ctrl-C in the TUI ends planrun, as it does on the line prompts
+
+Amends D73.
+
+- Raw mode makes Ctrl-C a key. The TUI confirms in the dock, restores the terminal, and calls
+  `shell::end_now(code)`, which ends planrun the way an interrupt does: every session's process
+  tree is killed, no destructor runs and no `run-end` is written, so `resume` goes on from the
+  journal (D73, D77). On Windows it is `process::exit`: exiting closes every job handle, and
+  `KILL_ON_JOB_CLOSE` kills each tree (D64). On Unix it first kills every registered process
+  group with the loop the signal handler uses, now one function that both call; no `unsafe` is
+  added.
+- Cancelling the run while planrun stays up would kill a running block from inside (D64): not
+  offered. Nor is a pause: asking under `--auto` would turn Auto into Ask, a decision change the
+  UI may not make. If it is wanted, it is a decision of its own with runner support.
+
+## D92 The TUI's dependencies
+
+Amends D9. `ratatui`, `crossterm` and `unicode-width`, in planrun-tui only. crossterm
+brings `windows-sys`, `mio` and `signal-hook`, which D64 and D73 keep out of planrun-exec; they
+stay out of it: planrun-exec's dependencies and its `unsafe` surface (`shell/kill.rs`) do not
+change.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t1-docs-decisions-md.patch"
Remove-Item "$env:TEMP\p6b-t1-docs-decisions-md.patch"
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

## P3 — Workspace, git and dry-run (done)

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

## P4 — Shells, expectations, policy (done)

Persistent pwsh and bash sessions (D24) with the sentinel protocol: variables persist across
blocks, a `throw` ends the block and not the session, output streams. The expectation matcher
(summary-line subsequence with `…`, exit expectations, prose → confirmation, D11). The command
classifier (D13). Expected failures skipped by default (D12). Tests for each, with pwsh and bash
when present (skipped with a message when not).

The decisions are D60–D70. Plan: `docs/plans/2026-11-25-p4-shells-expectations-policy.md`. First
the decisions and the `unsafe` lints (D64); then the sessions (the protocol D60, how a block
ends D61, the environment D62, which shells D63, the process tree D64); the matcher (D66); the
classifier (D68) and the policy decisions (D67, D69); the classes in the dry run. The candidate
(a script working directory from a run lead) is deferred (D70). The P4 review items P5 must fix
first are in `docs/handovers/2026-11-25-p4-to-p5.md`.

## P5 — Runner, journal, resume, interactive CLI (done)

The runner state machine; the commit gate (D22); the JSONL journal (D18); resume with `DoneState`
(D14); `run`, `resume`, `status`; the line-prompt UI (D15) with view/diff/shell/skip/quit. End to
end: `planrun run tests/corpus/synthetic/assay-style.md --auto read,build,modify` on a fixture copy
produces the same two commits (same trees) as the fixture's history; killing the process mid-task
and resuming completes it without repeating a write.

The decisions are D71–D82. Plan: `docs/plans/2026-11-25-p5-runner-journal-resume.md`. First the
P4 review items for sessions (D71) and sed (D72); then the transaction hardening and the step's
state on disk (D78), the journal and the lock (D76, D77), the runner with the commit gate and the
checks around a commit (D73–D75, D79–D81), resume (D77), and the CLI with line prompts (D74).
The fixture has no history of its own: the acceptance test builds the two expected trees, and
drives the binary through the test-only `e2e` feature (D82), uninterrupted, killed at a
question, and ended at three seams. The synthetic plan writes `docs/modules.txt` with LF only,
and the greeter ignores `Cargo.lock`.

## P6 — Dogfood and polish

planrun runs its own later phase plans. Candidates, each its own decision: a ratatui UI, a
sidecar IR from planbuild.py, `.planrun/formats/` discovery polish, a non-interactive batch mode
for CI.

- **P6a — release and update (done).** The CI gate on Windows and Linux, the release workflow on
  a `v*.*.*` tag, `scripts/install.ps1` and `install.sh` (a release, or `--from-source`), and
  `planrun update [--check]` (D83). Plan: `docs/plans/2026-11-26-p6a-release-update.md`; the
  first plan planrun executes on itself. Open items: `docs/handovers/2026-11-26-p5-to-p6.md`.
- **P6b — the TUI's prerequisites.** The TUI's design (`docs/tui-design.md`) and its decisions
  (D84–D92), then what it needs from planrun-exec (its §9): a journal with any number of
  observers, `shell::end_now`, typed effects in the dry-run report, the matcher's alignment,
  `QuestionKind`, and the read-only resume preview. The CLI's behaviour and the journal format do
  not change. Plan: `docs/plans/2026-11-26-p6b-tui-prerequisites.md`.
- **P6c — the TUI.** The `planrun-tui` crate and `planrun tui`: theme and glyphs, the shell, the
  background work, Plans, the plan view (Inspect, then Run through its `Ui` and a journal
  observer), the dock with every `QuestionKind`, History, the resume preview and the command
  palette (`docs/tui-design.md` §11). Its acceptance drives the TUI against the greeter fixture
  through the `e2e` seam (D82).
```

File `docs/tui-design.md` (full contents):

````markdown
# planrun TUI — Design

Status: adopted (r4). Its decisions are D84–D92 in docs/decisions.md; the changes outside
planrun-tui (§9) are P6b, the TUI itself P6c (docs/roadmap.md). Mockups: the "Planrun TUI —
Design" canvas (https://claude.ai/artifact/2nYJNnJv2YeMu6VpVxmbij). Every artboard is a real cell
grid (120 × 36, or 80 × 24).

The TUI is a front end over the existing crates. It shows the domain model and adds no model of
its own. Resume, policy and preflight stay authoritative: the TUI explains and asks; it never
decides, and it never turns one decision into another (D74).

## 1. Decisions

These amend D15 ("a ratatui TUI can come later behind the Ui trait") rather than stand alone.
docs/decisions.md holds them as D84–D92, with what P6b settled.

- **D84 — One plan view, two modes.** Inspect is filled from a dry run (D55); Run is filled from
  the live runner. Same layout, same tree. Inspect states the policy it assumes (`--auto`,
  `--run-expected-failures`, which `Policy::new` takes), and `R` runs with exactly that policy.
  The dry run's verdict is advisory: the real preflight decides at run time, so a stale Inspect
  never authorises anything.
- **D85 — Questions open in a dock.** It takes the console's region and leaves the navigator and
  the Change Inspector visible. There are no centred modals. The resume preview (§6.F) is a
  read-only full view, not a place where questions are answered.
- **D86 — `Question.kind` becomes an enum, `QuestionKind`, with the per-kind detail as payload.**
  `fn name(&self) -> &'static str` gives the journal name, so the journal format is unchanged.
  The dock matches on it exhaustively: the compiler enforces full coverage, and there is no
  fallback layout. Keys and typed input come from `Question.choices` and `Question.typed`; the
  body comes from `Question.text` plus the payload. An answer the runner offers is always shown;
  one it doesn't offer never is.
- **D87 — Phases are not a navigator level** (D5). Plans is a table of plan files.
- **D88 — Workspaces are deferred.** `planrun tui [<plans-dir>]` runs from the target repository.
- **D89 — Colour.** 24-bit only in v1; the 256-colour fallback and monochrome under `NO_COLOR`
  are designed (§4) and come in the polish phase (§11). The glyph set is pinned by a test and a
  manual check (§5).
- **D90 — The runner stays synchronous on a worker thread.** The render thread owns the terminal;
  the runner reaches it only through blocking round trips: `ask`, `shell`, `view` and `diff`
  (§2).
- **D91 — Ctrl-C ends planrun the way D73 does: mid-block, and resume continues.** Raw mode makes
  Ctrl-C a key, so the TUI confirms in the dock, restores the terminal, and calls
  `planrun_exec::shell::end_now(code) -> !`, a new safe function. On Windows it is
  `process::exit`: exiting closes every job handle, and `KILL_ON_JOB_CLOSE` kills each session
  tree (D64, D73). On Unix the group-kill loop is factored out of `on_signal` (kill.rs:222) into a
  plain function that both `on_signal` and `end_now` call, then `process::exit`. The existing
  `kill(-pid, SIGKILL)` block moves; no `unsafe` is added. As with Ctrl-C today, no `run-end` is
  written. Cancelling the run while planrun stays up would be a runner kill (D64): not offered.
- **D92 — Dependencies (D9).** `ratatui`, `crossterm` and `unicode-width`, in planrun-tui only.
  crossterm brings windows-sys, mio and signal-hook, which D64 and D73 kept out of planrun-exec.
  They stay out: planrun-exec's dependencies and its `unsafe` surface are unchanged.

Not in v1: **pause**. Asking under `--auto` turns Auto into Ask, a decision change the UI may not
make. If wanted, it is its own decision with runner support: a flag read before each action,
journaled as the decision it produced.

## 2. Where the data comes from

| UI need | source | change needed |
|---|---|---|
| navigator states, durations, outcome tags, change-state chips (APPLIED from `step-written`, COMMITTED from the commit's HEAD), History | the journal: `Journal::observe` (journal.rs:374) sees every Record (task-start, step-prepared, step-written, action-start with decision and HEAD, action-end with outcome, status, errored, cwd, tail and HEAD, answer, gate-override, task-end, run-end) plus `Seam::Wrote` | the observer slot is single and the e2e seam (D82) uses it: make it a `Vec` or chain the two. The observer is built on the worker thread and captures a `Sender` (it is not `Send`) |
| live console output | `Ui::output(OutLine)` (shell/mod.rs:63): `{ stream: Stdout\|Stderr, text, newline }`, exact order within a stream, arrival order across the two | none |
| prose: task headers, notes, findings as text | `Ui::status(&str)`, shown as console `EVENT` lines | none; never parsed |
| question bodies | `Question.text` plus the `QuestionKind` payload (D86): the policy `Decision` with its `Classification` reasons, preflight `Finding`s, expectation detail | `QuestionKind` enum |
| mismatch alignment (expected line ↔ output line) | `expect::check` | return the alignment; today `Mismatch::Line` (expect.rs:48) holds only the unmatched line |
| Inspect: per-action verdicts | `dry_run` (preflight.rs:126) returns `DryRun { start: Vec<Finding>, tasks: Vec<TaskReport> }`, with a per-action `Outcome` (Ok / WouldAsk / Failed / Depends / NotRun / NotChecked / Waits / Shown) | type `Outcome::Ok` and `WouldAsk` (blob ids, the decision) instead of `String`; `planrun-cli/src/dry_run.rs` formats them as today |
| Inspect: class and policy column | `DefaultClassifier::explain` + `Policy::decide`, both pure, called by the TUI | none |
| resume preview | `analyse` (resume.rs:108, pure, journal only) | `runner::preview::preview`: whether a run is live, resume's own HEAD verdict, Done / NotDone / Unknown per prepared step, the scripts that ask again, the `resume-dirt` paths; no writes, no lock |
| a run in progress elsewhere | `lock::live`: a try on `<git-dir>/planrun/lock.live`, which a run holds with a blocking lock after taking `lock` (D90) | the second lock. Never try `lock` itself: it would make a starting `planrun run` fail. `lock.pid` is only shown, never trusted |

No parallel Event stream: the journal plus `status`, `output` and `Question` cover it.

**Round trips** (worker → render thread, blocking, like `ask`):

- `Ui::shell`: the render thread leaves the alternate screen, disables raw mode, sets D73's "shell
  running" flag, runs the shell, restores the terminal and replies.
- `Ui::view` / `Ui::diff` (routed for `[v]`/`[d]`, runner/mod.rs:239): answered in the TUI
  itself; the inspector shows the action or diff and the dock stays open.

**Background work** (threads plus a channel): plan checks, `update --check`, `git status
--porcelain --no-optional-locks`, journal scans, the liveness probe. Each result carries the
fingerprint of its input and has five states: running, current, stale, unavailable, failed.
Background checks never replace preflight.

## 3. Shell (every view)

```
row 0      header band   PLANRUN  crumb › crumb › current           mode chip · state
rows 1..   view body
row h-2    footer 1      contextual keys as chips                   (only when h ≥ 30)
row h-1    footer 2      F1 Help F2 Plans F3 Plan F4 History · target · plans     UPDATE
```

- **Header state:** `○ idle`, ` INSPECT  policy default`, ` RUN  ◉ executing`, `◆ waiting for you`,
  `✕ stopped`. Other runs: `· running in pid 4812` (fg2) when `lock::live` says a run is live
  (the pid from `lock.pid`, shown only), `· 1 interrupted run` (warn) only when no run is live
  and a journal has no run-end.
- **Update indicator:** pinned bottom-right. `planrun 0.1.0` (fg3) when up to date or unchecked,
  `↑ 0.2.0 available` (meta), `update check failed` (fg3). Never green or red.
- Below 30 rows the footer is one line.

## 4. Tokens

| token | hex | 256 | use |
|---|---|---|---|
| bg0 | #111820 | 234 | app background |
| bg1 | #19232E | 235 | header, footer, panel titles, dock band |
| bg2 | #263442 | 236 | selection, key chips, inputs, overlays |
| rule | #2E3D4C | 237 | hairlines between panels |
| border | #526579 | 60 | overlay frame |
| fg | #D8E1E9 | 254 | primary text |
| fg2 | #91A3B5 | 109 | secondary text |
| fg3 | #6C8094 | 66 | tertiary text, line numbers, timestamps |
| ok | #69B995 | 72 | verified, done |
| warn | #D9A85F | 179 | planrun waits for you; warnings |
| err | #D87575 | 174 | failure, error, Destructive class |
| act | #75A9D5 | 110 | focus, the running action |
| meta | #B69BD9 | 140 | hashes, ids, versions, absorbed blocks |
| addbg | #1B3229 | 235 | diff addition row |
| delbg | #3A2228 | 236 | diff deletion row |

In 256-colour mode the diff rows lose their tint; the `+`/`-` sign colours carry the meaning.
Mode: truecolor when `COLORTERM` is `truecolor`/`24bit` or `WT_SESSION` is set; otherwise 256;
`NO_COLOR` means monochrome (bold for focus, reverse for selection). Colour rules: green only
after verification, amber only when planrun waits for you or for a warning, red for failure and
Destructive, blue for focus and the running action, purple for identifiers. Bold and reverse
only.

## 5. Glyphs

| role | glyph | fallback | colour | meaning |
|---|---|---|---|---|
| status | ○ | `o` | fg3 | pending |
| | ◉ | `*` | act | running |
| | ✓ | `v` | ok | done, verified (grey = stale check) |
| | ✕ | `x` | err | failed |
| | ⊘ | `-` | fg3 | skipped |
| | ◆ | `?` | warn | waiting for you |
| | ! | — | warn | mismatch, warning |
| | · | — | fg3 | dry run: not simulated |
| | ◌ | `~` | fg2 | background work |
| spinner | ◐◓◑◒ | `-\|/` | act | the console's live line |
| kind | + ± ≈ − ∗ $ ⊙ ¶ | `+ ~ = - * $ @ .` | fg2 | WriteFile, ApplyPatch, ReplaceExact, Delete, Generated, RunScript, Commit, Note |
| | ⚑ | `M` | warn | Manual |

**Pinned:** a unit test asserts `unicode-width == 1` for every glyph in the table. A manual check
list in the crate README covers rendering in Windows Terminal with Cascadia Mono (⚑ U+2691 and
◐◓◑◒ are the uncertain ones). A glyph that fails the check is replaced by its fallback in the
table, and the whole set has one switch (`PLANRUN_ASCII=1`) for terminals that render badly.

Change-state chips: `PLANNED` (fg2), `APPLIED` (act), `COMMITTED` (ok, with the hash in meta).
Policy tags in Inspect: `auto`, `ask`, `type yes`, `skip`, `waits`, `gate`. In Run the tag
becomes the outcome (duration, `applied`, hash, `mismatch`).

## 6. Views

### A. Plans (F2)

A table of plan files, `PLAN · FORMAT · TASKS · CHECK · RUN`, with check and run as separate
columns. The selected plan's detail shows the outline with a 16-cell coverage strip per task
(action, absorbed, manifest, note, manual; a non-zero manual count is always at least one cell),
the CHECK diagnostics, the journal STATUS, and a PREFLIGHT line saying it is not a background
check. Keys: `enter` Inspect, `R` run, `c` re-check, `/` filter.

### B/C. Plan view (F3)

- **Navigator** (36 columns): Task → Step → Action. Completed tasks fold to one line with their
  hash. A row is status glyph, kind glyph, label, tag; the selected row has a bg2 background and
  a `▌`.
- **Upper right.** Inspect: ACTION (kind, path, class and reasons, the policy decision under the
  assumed policy, the dry-run verdict, the plan source line, the planned diff). Run: CHANGES (tabs
  of the task's files with `+n −m`, `ctrl+p`, then the diff with its change-state chip). Diffs are
  unified; split (`s`) only at 160 columns or more. No syntax highlighting in v1.
- **Lower right.** Inspect: DRY RUN, marked "advisory · preflight decides at run time". Run:
  CONSOLE with columns `time · label · text`. Labels: `TASK`, `PRE`, `FILE`, `RUN`, `out`, `err`
  (warn, with a gutter mark), `VERIFY`, `COMMIT`, `EVENT` (status prose). Filters 1–5;
  `⇣ following` until you scroll; `f` resumes.
- **Policy in Inspect:** `a` changes the assumed policy (auto classes, expected failures). The dry
  run reruns; `R` uses it.
- **Proportions:** `<`/`>` resize the navigator, `+`/`-` move the right split. Kept per session.

### D. Decision dock

It covers the console's region. It has an amber band (`▐`, title, and on the right the task, step
and shell), the body, and the answers. The answers come from `Question.choices`; when
`Question.typed` is set there is an input field and the choice words are listed beside it.
Footer and header switch to the dock. `tab` moves focus into the tree and inspector to look
around before answering.

One layout per `QuestionKind` variant; the match is exhaustive (D86). The answers column is
illustrative; the runner's choices decide.

| kind | band title | body (`text` + payload) | answers, per D74/D75/D77/D78/D81 |
|---|---|---|---|
| script ask | `◆ DECISION` | the block, class and reasons, cwd | from choices |
| Destructive script | `◆ DECISION` typed answer | the command that set the class, the block with line numbers, if yes / if skip / if quit | typed `yes` · `skip` · `quit` |
| mismatch | `! MISMATCH` | expected lines aligned to output lines, exit status | accept only for Exited/Threw (D75) · retry · shell · view · quit |
| dead session | `! SESSION ENDED` | how the block ended (D61) | retry · shell · quit (no accept) |
| confirmation (D11) | `◆ CONFIRM` | the derived exit check and the prose, labelled apart | from choices |
| `session` (D81 notice) | `◆ NEW SESSION` | why a new session starts; acknowledged even under `--auto` | acknowledge |
| Manual | `⚑ MANUAL` | the plan's text, quoted | from choices |
| unscoped file action (D50) | `◆ OUTSIDE SCOPE` | the path, the manifest | from choices |
| dirt outside the task (D80) | `◆ OTHER CHANGES` | the dirty paths | from choices |
| commit gate (D75) | `◆ COMMIT GATE` | why it is closed, the task's paths | typed |
| cwd outside the repo (D81) | `◆ WORKING DIRECTORY` | the directory | from choices |
| `changed` (D78) | `◆ CHANGED SINCE PREPARE` | the paths that moved | prepare again · quit |
| `again` (D77) | `◆ RAN BEFORE THE INTERRUPTION` | the action and its last outcome | run again · keep |
| `head-moved` (D77) | `! HEAD MOVED` | recorded vs current HEAD | keep · retry · diff · quit (deliberately no shell) |
| `unknown` (D77) | `◆ UNKNOWN STEP STATE` | per path: planned / before / now; whether the pre-images are present | restore · accept (typed) |
| `accept-done` (D77) | `◆ ACCEPT AS DONE` | what is accepted | typed |
| `resume-dirt` (D77) | `◆ YOUR CHANGES` | the path and the action that writes over it | from choices |

The variant set is whatever `QuestionKind` holds after the refactor. This table is the design
for each one; the compiler catches a missing one.

### E. History (F4)

Runs from `<git-dir>/planrun/*.jsonl` (journals set aside by `--restart` included), grouped by
date. The end of a run: its `run-end` record; or "running in pid N" when `lock::live` says a run is
live; or "killed" only when no run is live and there is no `run-end`. `R` opens F.

### F. Resume preview

A read-only full view, built from `analyse` plus the read-only disk probe. It holds no lock and
writes nothing, and it shows when its snapshot was taken (`g` refreshes it).

- **journal against repository:** whether a run is live, the fingerprint, `index.lock`, and
  resume's own verdict on HEAD (unchanged, moved by the action that was running, or moved so
  that resume stops);
- **what resume will do:** the task that runs again, each prepared step's state, and the scripts
  that will ask "run again or keep" (and whether each was running);
- **your files:** the `resume-dirt` paths (never an Unknown step's: restoring puts them back,
  accepting verifies them) and untracked files.

Every question is labelled "asked when resume reaches it". `enter` starts `Runner::resume`, which
takes the lock, checks everything again against the disk at that moment, and asks its questions
in the dock, each journaled when asked (D74). Answering in the preview would be a new decision,
with the runner re-checking that the state still matches: not in v1.

### G. Command palette (ctrl+k)

An overlay on a dimmed view, with grouped results, matched characters in act, and shortcuts on
the right. Commands come from a registry. `ctrl+p` is the same overlay limited to the current
plan's files and actions.

## 7. Focus and keys

Focus: `tab` / `shift+tab`. The focused panel's band shows `▎` and a title in act, bold.

| key | action |
|---|---|
| F1 / ? | help / contextual help |
| F2 F3 F4 | Plans · Plan · History |
| ↑↓ j k · ←→ h l | move · fold and unfold |
| enter | open / inspect |
| esc | back (in a dock: nothing; answers are explicit) |
| R / shift+R | run / run from this task, with Inspect's policy |
| a | Inspect: assumed policy |
| c | re-check plan |
| o | open shell (round trip, §2) |
| ctrl+c | end planrun after a confirm in the dock (D91) |
| f · 1–5 · s | follow · console filter · split diff |
| ctrl+k · ctrl+p | commands · go to file or action |
| q | back; quits only from Plans with no run active |

Typed questions take the whole word and `enter`; a single key never answers them.

## 8. Sizes

| width × height | layout |
|---|---|
| ≥ 160 × 40 | as at 120 × 36; split diff allowed |
| 100–159 × ≥ 30 | three panels |
| < 100 or < 30 rows | one pane at a time (`1 plan · 2 changes · 3 console`); the navigator becomes a task strip; one footer line; a dock takes the whole pane |
| < 60 × 16 | "terminal too small"; the run continues |

## 9. Changes outside planrun-tui

All in planrun-exec, all small, none of them changes a decision:

1. The journal observer as a `Vec` (or chained), so the TUI and the e2e seam (D82) coexist.
2. `QuestionKind`: an enum replacing `kind: &'static str`, with each kind's detail as payload and
   `name()` for the journal (D86). This replaces the doc-comment fix.
3. `expect::check` returns the expected ↔ output alignment.
4. The resume preview, beside `analyse`: read only, with resume's own HEAD verdict, the step
   states, the scripts that ask again and the `resume-dirt` paths.
5. `shell::end_now(code) -> !`, with the Unix group-kill loop factored out of `on_signal` (D91).
6. Typed `Outcome::Ok` and `Outcome::WouldAsk` in the dry-run report; the CLI's output is unchanged.
7. `lock.live` and `lock::live`, so a live run is seen without trying the run lock (D90).

## 10. Open questions

1. Out of v1 but worth a decision later: pause (§1), and answering in the resume preview (§6.F).

## 11. v1 scope and delivery

**In v1:** the shell, Plans, the plan view in both modes, the dock with every `QuestionKind`,
History, the resume preview, and the command palette (`ctrl+k`, `ctrl+p`).

**Deferred to a polish phase**, each designed here already:

- the compact layout below 100 × 30. In v1, below 100 × 30 the TUI shows "terminal too small,
  needs 100 × 30" and the run continues underneath. A dock that opens while it is too small is
  still answerable: the notice turns into the dock alone.
- adjustable proportions: v1 has a fixed 36-column navigator and a 55/45 right split.
- 256-colour and `NO_COLOR` modes: v1 is truecolor only. The tokens keep their 256 indices so
  the fallback is a lookup table later.

**Delivery, two plans:**

- **P6b — exec prerequisites and docs.** First task: this spec into `docs/tui-design.md`, the
  decisions into `decisions.md` (amending D15), and P6b/P6c into `roadmap.md`. Then the §9 changes
  in planrun-exec, each with its tests; the CLI's behaviour and the journal format are unchanged.
- **P6c — the TUI.** The `planrun-tui` crate and `planrun tui`: theme and glyphs, the shell,
  background work, Plans, the plan view (Inspect, then Run through the `Ui` impl and the journal
  observer), the dock, History, the resume preview, the palette. Acceptance includes driving
  the TUI against the greeter fixture through the same `e2e` seam as P5 (D82).
````

**Step 2 — verify**

Targeted: D84 to D92 are there once each, and the design cites no candidate number:

```powershell
(Select-String -Path docs/decisions.md -Pattern '^## D(8[4-9]|9[0-2]) ').Count
(Select-String -Path docs/tui-design.md -Pattern '\bTD[1-9]\b').Count
```

Expected (exit code 0; summary lines):

```text
9
0
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- docs/decisions.md docs/roadmap.md docs/tui-design.md
  $verified = [ordered]@{
      'docs/decisions.md' = 'eda85c121068e9bb7cd90396384ea1d5afc7b206'
      'docs/roadmap.md' = 'c2e658809358d7a866722b118c608864f7e41cc7'
      'docs/tui-design.md' = 'c9511fb72da440bf4bae19246afaa7ad229657bf'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6b-T1: the TUI's design and decisions" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P6b-T2: journal observers as a list

**Goal:** A journal takes any number of observers, called after every append and every write of a step's commit in the order they were added, so the TUI's live view and the `e2e` fault seam (D82) can watch the same run (D90). `Seam` becomes `Copy` to be passed to each.

**Files**

- Modify `crates/planrun-exec/src/journal.rs`
- Modify `crates/planrun-exec/src/journal_tests.rs`

Over 300 lines, so it comes as an exact diff: `crates/planrun-exec/src/journal.rs`.

**Step 1 — write the files**

File `crates/planrun-exec/src/journal.rs` (exact diff; save the block as `$env:TEMP\p6b-t2-src-journal-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/journal.rs b/crates/planrun-exec/src/journal.rs
index ffd22de..ccd9b54 100644
--- a/crates/planrun-exec/src/journal.rs
+++ b/crates/planrun-exec/src/journal.rs
@@ -324,7 +324,7 @@ pub fn read(path: &Path) -> Result<Vec<Line>, JournalError> {
 
 /// What a journal observer sees: a record that is now with the OS (and synced, when durable),
 /// or the number of paths a step's commit has written so far.
-#[derive(Debug)]
+#[derive(Debug, Clone, Copy)]
 pub enum Seam<'a> {
     /// A record was appended.
     Appended(&'a Record),
@@ -332,14 +332,15 @@ pub enum Seam<'a> {
     Wrote(usize),
 }
 
-/// A callback that sees every [`Seam`]: planrun-cli's fault seam for its e2e tests (D82).
+/// A callback that sees every [`Seam`]: planrun-cli's fault seam for its e2e tests (D82), and
+/// the TUI's view of a live run (D90).
 pub type Observer = Box<dyn FnMut(Seam<'_>)>;
 
 /// The appending writer.
 pub struct Journal {
     file: File,
     path: PathBuf,
-    observer: Option<Observer>,
+    observers: Vec<Observer>,
 }
 
 impl Journal {
@@ -366,17 +367,18 @@ impl Journal {
         Ok(Journal {
             file,
             path: path.to_path_buf(),
-            observer: None,
+            observers: Vec::new(),
         })
     }
 
-    /// Calls `observer` after every append and every write of a step's commit.
+    /// Adds `observer`: it is called after every append and every write of a step's commit,
+    /// after the observers added before it.
     pub fn observe(&mut self, observer: Observer) {
-        self.observer = Some(observer);
+        self.observers.push(observer);
     }
 
     fn seam(&mut self, seam: Seam<'_>) {
-        if let Some(observer) = self.observer.as_mut() {
+        for observer in &mut self.observers {
             observer(seam);
         }
     }
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t2-src-journal-rs.patch"
Remove-Item "$env:TEMP\p6b-t2-src-journal-rs.patch"
```

File `crates/planrun-exec/src/journal_tests.rs` (full contents):

```rust
//! The journal's writer and reader (D76): one line per record, a torn tail dropped and cut off
//! before the next append, a bad line in the middle an error.

use super::*;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "planrun-journal-{}-{name}-{}",
        std::process::id(),
        now_ms()
    ));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    dir
}

fn head() -> Head {
    Head {
        commit: Some("0123456789abcdef0123456789abcdef01234567".to_owned()),
        reference: Some("refs/heads/main".to_owned()),
    }
}

fn task_start(task: &str) -> Record {
    Record::TaskStart {
        task: task.to_owned(),
        head: head(),
    }
}

fn records(path: &Path) -> Vec<Record> {
    read(path)
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|l| l.record)
        .collect()
}

#[test]
fn records_round_trip_one_per_line_with_the_schema_version() {
    let dir = temp_dir("round");
    let path = path_for(&dir, "plan");
    let mut j = Journal::open(&path).unwrap_or_else(|e| panic!("{e}"));
    let at = At {
        task: "P1-T1".to_owned(),
        step: 0,
        action: 1,
    };
    let all = [
        task_start("P1-T1"),
        Record::ActionStart {
            at: at.clone(),
            kind: "commit".to_owned(),
            decision: "Auto".to_owned(),
            head: head(),
            expect: vec![PathBlob {
                path: RepoPath::new("src/a.rs").unwrap_or_else(|e| panic!("{e}")),
                blob: None,
            }],
        },
        Record::RunEnd {
            end: End::Stopped,
            reason: Some("quit".to_owned()),
        },
    ];
    for r in all.clone() {
        j.append(r).unwrap_or_else(|e| panic!("{e}"));
    }
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(text.lines().count(), 3);
    assert!(text.lines().all(|l| l.starts_with("{\"v\":1,")), "{text}");
    assert!(text.contains("\"t\":\"action-start\""), "{text}");
    assert_eq!(records(&path), all);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_torn_last_line_is_dropped_and_cut_before_the_next_append() {
    let dir = temp_dir("torn");
    let path = path_for(&dir, "plan");
    let mut j = Journal::open(&path).unwrap_or_else(|e| panic!("{e}"));
    j.append(task_start("P1-T1"))
        .unwrap_or_else(|e| panic!("{e}"));
    drop(j);
    let whole = serde_json::to_string(&Line {
        v: SCHEMA,
        ms: 1,
        record: task_start("P1-T2"),
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let mut file = OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap_or_else(|e| panic!("{e}"));
    file.write_all(whole.as_bytes())
        .unwrap_or_else(|e| panic!("{e}"));
    drop(file);
    assert_eq!(
        records(&path),
        [task_start("P1-T1")],
        "a parseable tail without \\n"
    );
    let mut j = Journal::open(&path).unwrap_or_else(|e| panic!("{e}"));
    j.append(task_start("P1-T3"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(records(&path), [task_start("P1-T1"), task_start("P1-T3")]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_bad_line_in_the_middle_is_an_error_and_a_missing_file_is_empty() {
    let dir = temp_dir("bad");
    let path = path_for(&dir, "plan");
    assert!(read(&path).unwrap_or_else(|e| panic!("{e}")).is_empty());
    let mut j = Journal::open(&path).unwrap_or_else(|e| panic!("{e}"));
    j.append(task_start("P1-T1"))
        .unwrap_or_else(|e| panic!("{e}"));
    drop(j);
    let mut text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
    text.push_str("{\"v\":1,\"ms\":1,\"t\":\"no-such\"}\n");
    text.push_str(&fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}")));
    fs::write(&path, &text).unwrap_or_else(|e| panic!("{e}"));
    match read(&path) {
        Err(JournalError::Corrupt { line: 2, .. }) => {}
        other => panic!("{other:?}"),
    }
    fs::write(&path, text.replace("\"v\":1", "\"v\":9")).unwrap_or_else(|e| panic!("{e}"));
    assert!(matches!(
        read(&path),
        Err(JournalError::Corrupt { line: 1, .. })
    ));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn the_observer_sees_appends_and_writes() {
    let dir = temp_dir("observer");
    let mut j = Journal::open(&path_for(&dir, "plan")).unwrap_or_else(|e| panic!("{e}"));
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let log = seen.clone();
    j.observe(Box::new(move |seam| {
        log.borrow_mut().push(match seam {
            Seam::Appended(r) => format!("{r:?}")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_owned(),
            Seam::Wrote(n) => format!("wrote {n}"),
        });
    }));
    j.append(task_start("P1-T1"))
        .unwrap_or_else(|e| panic!("{e}"));
    let at = At {
        task: "P1-T1".to_owned(),
        step: 0,
        action: 0,
    };
    j.ahead(&at).after_write(2);
    assert_eq!(*seen.borrow(), ["TaskStart", "wrote 2"]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn every_observer_sees_every_seam_in_the_order_they_were_added() {
    let dir = temp_dir("observers");
    let mut j = Journal::open(&path_for(&dir, "plan")).unwrap_or_else(|e| panic!("{e}"));
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    for name in ["first", "second"] {
        let log = seen.clone();
        j.observe(Box::new(move |seam| {
            let what = match seam {
                Seam::Appended(_) => "appended".to_owned(),
                Seam::Wrote(n) => format!("wrote {n}"),
            };
            log.borrow_mut().push(format!("{name}: {what}"));
        }));
    }
    j.append(task_start("P1-T1"))
        .unwrap_or_else(|e| panic!("{e}"));
    let at = At {
        task: "P1-T1".to_owned(),
        step: 0,
        action: 0,
    };
    j.ahead(&at).after_write(1);
    assert_eq!(
        *seen.borrow(),
        [
            "first: appended",
            "second: appended",
            "first: wrote 1",
            "second: wrote 1"
        ]
    );
    let _ = fs::remove_dir_all(dir);
}
```

**Step 2 — verify**

Targeted: the journal's unit tests:

```powershell
cargo test -p planrun-exec --lib journal::tests
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-exec/src/journal.rs crates/planrun-exec/src/journal_tests.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/journal.rs' = 'ccd9b5445a775d50cb4f3fe7081a30fc41233179'
      'crates/planrun-exec/src/journal_tests.rs' = 'c6b9351ac8f4649aaf61e0bdee8dc24ae336b3ef'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6b-T2: journal observers as a list" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P6b-T3: shell::end_now

**Goal:** `planrun_exec::shell::end_now(code) -> !` ends planrun as Ctrl-C does (D73, D91): every session's tree is killed, no destructor runs, no `run-end` is written. Windows: `process::exit`, and the job objects kill the trees. Unix: the group-kill loop moves out of the signal handler into `kill_registered`, which both call; no `unsafe` block is added.

**Files**

- Modify `crates/planrun-exec/src/shell/kill.rs`
- Modify `crates/planrun-exec/src/shell/mod.rs`
- Modify `crates/planrun-exec/tests/shell.rs`

Over 300 lines, so they come as exact diffs: `crates/planrun-exec/src/shell/kill.rs`, `crates/planrun-exec/tests/shell.rs`.

**Step 1 — write the files**

File `crates/planrun-exec/src/shell/kill.rs` (exact diff; save the block as `$env:TEMP\p6b-t3-shell-kill-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/shell/kill.rs b/crates/planrun-exec/src/shell/kill.rs
index a3a49d6..5b34fea 100644
--- a/crates/planrun-exec/src/shell/kill.rs
+++ b/crates/planrun-exec/src/shell/kill.rs
@@ -2,8 +2,8 @@
 //! planrun lives in this module: the job object, the handle wait and the console control
 //! handler on Windows; `setsid`, `prctl`, `getppid`, `waitid`, `kill`, `signal` and `raise` on
 //! Unix, each declared by hand. The rest of the crate sees safe operations: [`Tree::spawn`],
-//! [`Tree::exited`], [`Tree::kill`], [`Tree::wait`], [`install_interrupts`] and
-//! [`ignore_interrupts`].
+//! [`Tree::exited`], [`Tree::kill`], [`Tree::wait`], [`install_interrupts`],
+//! [`ignore_interrupts`] and [`end_now`].
 
 #![allow(unsafe_code)]
 
@@ -167,6 +167,16 @@ pub(crate) fn ignore_interrupts(on: bool) {
     IGNORE_INTERRUPT.store(on, Ordering::SeqCst);
 }
 
+/// Ends planrun with `code` the way an interrupt does (D73, D91): every session's tree is
+/// killed, no destructor runs, nothing more is journaled. Windows: exiting closes every job
+/// handle, and `KILL_ON_JOB_CLOSE` kills the trees. Unix: every registered process group is
+/// killed first, as the signal handler does.
+pub(crate) fn end_now(code: i32) -> ! {
+    #[cfg(unix)]
+    unix::kill_registered();
+    std::process::exit(code)
+}
+
 #[cfg(unix)]
 mod unix {
     use std::io;
@@ -217,12 +227,9 @@ mod unix {
         }
     }
 
-    /// Kills every registered group, then ends planrun the way `signum` would. Only
-    /// async-signal-safe calls: atomics, `kill`, `signal`, `raise`.
-    extern "C" fn on_signal(signum: i32) {
-        if signum == SIGINT && IGNORE_INTERRUPT.load(Ordering::SeqCst) {
-            return;
-        }
+    /// Kills every registered group. Only async-signal-safe calls (atomics and `kill`), so the
+    /// signal handler may call it.
+    pub(super) fn kill_registered() {
         for slot in &GROUPS {
             let pid = slot.load(Ordering::SeqCst);
             if pid > 0 {
@@ -231,6 +238,15 @@ mod unix {
                 unsafe { kill(-pid, SIGKILL) };
             }
         }
+    }
+
+    /// Kills every registered group, then ends planrun the way `signum` would. Only
+    /// async-signal-safe calls: atomics, `kill`, `signal`, `raise`.
+    extern "C" fn on_signal(signum: i32) {
+        if signum == SIGINT && IGNORE_INTERRUPT.load(Ordering::SeqCst) {
+            return;
+        }
+        kill_registered();
         // SAFETY: restoring the default action and raising the signal again ends the process
         // as the signal would have; both calls are async-signal-safe.
         unsafe {
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t3-shell-kill-rs.patch"
Remove-Item "$env:TEMP\p6b-t3-shell-kill-rs.patch"
```

File `crates/planrun-exec/src/shell/mod.rs` (full contents):

```rust
//! Persistent pwsh and bash sessions (docs/architecture.md §6.3; decisions D24, D60–D65).
//!
//! One [`Session`] is one shell process. Each block is written to a file in a run directory
//! under the OS temp directory and run by one ASCII line on the session's stdin: dot-sourced in
//! pwsh, `source`d at the top level in bash, with stdin taken away from the block. A sentinel
//! with a per-run secret and the block's number follows it on stdout and on stderr; the block
//! ends when both arrive.

mod bash;
mod kill;
mod pwsh;
mod session;

use std::io;
use std::path::PathBuf;

pub use session::{Session, SessionConfig, locate};

/// Installs planrun's interrupt handling once, from the main thread, before the first session
/// starts (D73). On Unix SIGINT, SIGTERM and SIGHUP kill every session's process group and then
/// end planrun as the signal would; on Windows Ctrl-C ends planrun and the job objects kill the
/// sessions.
pub fn install_interrupt_handler() {
    kill::install_interrupts();
}

/// Ends planrun now with exit code `code`, as Ctrl-C would (D73, D91): every session's process
/// tree is killed, no destructor runs and no `run-end` is written, so `resume` goes on from the
/// journal. For a UI that reads Ctrl-C as a key (the TUI's raw mode); restore the terminal
/// first.
pub fn end_now(code: i32) -> ! {
    kill::end_now(code)
}

/// While the guard lives, Ctrl-C does not end planrun: the `[o]` shell runs in the foreground
/// and gets it instead (D73).
pub struct InterruptsIgnored(());

impl InterruptsIgnored {
    /// Starts ignoring Ctrl-C until the guard is dropped.
    pub fn new() -> InterruptsIgnored {
        kill::ignore_interrupts(true);
        InterruptsIgnored(())
    }
}

impl Default for InterruptsIgnored {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for InterruptsIgnored {
    fn drop(&mut self) {
        kill::ignore_interrupts(false);
    }
}

/// Which stream a line came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// One line of a block's output. Order within a stream is exact; order across the two streams
/// is the order of arrival, best effort.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutLine {
    /// The stream.
    pub stream: Stream,
    /// The text without its LF (a CR before it stays), UTF-8 decoded lossily.
    pub text: String,
    /// False for the block's last output on this stream when it did not end with a newline.
    pub newline: bool,
}

/// How a block ended (decision D61).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunStatus {
    /// It ran to its end. pwsh: `$LASTEXITCODE`, reset to 0 before the block (`exit n` ends only
    /// the block); bash: `$?` of the `source`.
    Exited(i32),
    /// pwsh only: a terminating error ended the block.
    Threw,
    /// The session ended during the block: bash `exit n` (with its code), or the shell went away
    /// without its sentinel (`None`). The session is dead.
    SessionEnded(Option<i32>),
    /// The block ran past the session's timeout; its process tree was killed.
    TimedOut,
    /// The protocol broke (a sentinel for another block, unexpected fields). The session is
    /// dead.
    Broken(Box<str>),
}

/// The result of running one shell block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRun {
    /// How it ended.
    pub status: RunStatus,
    /// pwsh: `$Error[0]` is another object after the block than before it (a non-terminating
    /// error was recorded, decision D61). Always false in bash. Never changes `status`.
    pub errored: bool,
    /// Its output, both streams.
    pub lines: Vec<OutLine>,
    /// The session's working directory after the block, when the sentinel reported it.
    pub cwd: Option<PathBuf>,
}

/// A long-lived shell: variables, the working directory and functions persist across blocks.
pub trait ShellSession: Send {
    /// Runs `script`, streaming each output line to `on_line` as it arrives. An error means the
    /// block was not started (its file could not be written, or the session is dead).
    fn run(&mut self, script: &str, on_line: &mut dyn FnMut(&OutLine)) -> io::Result<ScriptRun>;
}

/// Why a session could not be had.
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    /// The shell is not installed, or not the one planrun accepts (decision D63).
    #[error("{shell} is not available: {reason}")]
    Unavailable {
        /// `pwsh` or `bash`.
        shell: &'static str,
        /// What the probe found.
        reason: String,
    },
    /// The shell started but its prologue did not answer as expected.
    #[error("the {shell} session did not start: {reason}")]
    Start {
        /// `pwsh` or `bash`.
        shell: &'static str,
        /// What went wrong.
        reason: String,
    },
    /// Spawning, the run directory, or the process tree failed.
    #[error(transparent)]
    Io(#[from] io::Error),
}
```

File `crates/planrun-exec/tests/shell.rs` (exact diff; save the block as `$env:TEMP\p6b-t3-tests-shell-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/tests/shell.rs b/crates/planrun-exec/tests/shell.rs
index 1339124..30a7857 100644
--- a/crates/planrun-exec/tests/shell.rs
+++ b/crates/planrun-exec/tests/shell.rs
@@ -425,6 +425,88 @@ fn a_timeout_kills_the_whole_tree() {
     }
 }
 
+/// Set in the child process of [`end_now_kills_every_session_tree`]: the fixture's root, the
+/// directory for the session's run directory, the file for the grandchild's pid, and the exit
+/// code, separated by `|`.
+const END_NOW_CHILD: &str = "PLANRUN_TEST_END_NOW_CHILD";
+
+/// Runs only as the child of [`end_now_kills_every_session_tree`]: starts a pwsh session whose
+/// block starts a grandchild and sleeps, then calls `end_now` while the block runs. In every
+/// other run it returns at once and counts as one passed test.
+#[test]
+fn end_now_child() {
+    let Some(spec) = std::env::var_os(END_NOW_CHILD) else {
+        return;
+    };
+    let spec = spec.to_string_lossy().into_owned();
+    let [root, temp, pid_file, code] = spec.split('|').collect::<Vec<_>>()[..] else {
+        panic!("{END_NOW_CHILD}: {spec}");
+    };
+    let config = SessionConfig {
+        root: PathBuf::from(root),
+        env: Vec::new(),
+        timeout: None,
+        temp: PathBuf::from(temp),
+    };
+    let mut s = Session::start(Shell::Pwsh, &config).unwrap_or_else(|e| panic!("{e}"));
+    let script = format!(
+        "$p = Start-Process pwsh -ArgumentList '-NoProfile', '-Command', 'Start-Sleep 300' -PassThru\n\
+         Set-Content -LiteralPath '{pid_file}' -Value $p.Id\nStart-Sleep 300\n"
+    );
+    std::thread::spawn(move || {
+        let _ = s.run(&script, &mut |_| {});
+    });
+    let until = Instant::now() + Duration::from_secs(60);
+    while !Path::new(pid_file).exists() && Instant::now() < until {
+        std::thread::sleep(Duration::from_millis(50));
+    }
+    std::thread::sleep(Duration::from_millis(200));
+    planrun_exec::shell::end_now(code.parse().unwrap_or_else(|e| panic!("{e}")));
+}
+
+#[test]
+fn end_now_kills_every_session_tree() {
+    if std::env::var_os(END_NOW_CHILD).is_some() {
+        return;
+    }
+    let f = Fixture::greeter(false);
+    let Some(probe) = session(Shell::Pwsh, &f) else {
+        return;
+    };
+    drop(probe);
+    let pid_file = f.outside().join("end-now-pid");
+    let spec = format!(
+        "{}|{}|{}|7",
+        f.path().display(),
+        f.outside().display(),
+        pid_file.display()
+    );
+    let exe = std::env::current_exe().unwrap_or_else(|e| panic!("{e}"));
+    let out = Command::new(exe)
+        .args([
+            "end_now_child",
+            "--exact",
+            "--nocapture",
+            "--test-threads=1",
+        ])
+        .env(END_NOW_CHILD, &spec)
+        .output()
+        .unwrap_or_else(|e| panic!("{e}"));
+    let report = format!(
+        "{}{}",
+        String::from_utf8_lossy(&out.stdout),
+        String::from_utf8_lossy(&out.stderr)
+    );
+    assert_eq!(out.status.code(), Some(7), "{report}");
+    let pid = std::fs::read_to_string(&pid_file).unwrap_or_else(|e| panic!("{e}: {report}"));
+    let pid = pid.trim().to_owned();
+    let until = Instant::now() + Duration::from_secs(10);
+    while alive(&pid) && Instant::now() < until {
+        std::thread::sleep(Duration::from_millis(100));
+    }
+    assert!(!alive(&pid), "the grandchild {pid} survived end_now");
+}
+
 #[cfg(windows)]
 #[test]
 fn a_session_leaves_the_terminal_s_code_page_alone() {
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t3-tests-shell-rs.patch"
Remove-Item "$env:TEMP\p6b-t3-tests-shell-rs.patch"
```

**Step 2 — verify**

Targeted: the child-process test of `end_now` (a pwsh session with a grandchild):

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-exec --test shell end_now
```

Expected (exit code 0; summary lines):

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-exec/src/shell/kill.rs crates/planrun-exec/src/shell/mod.rs crates/planrun-exec/tests/shell.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/shell/kill.rs' = '5b34fead499812406a3adf9beaaf5a95997c45cb'
      'crates/planrun-exec/src/shell/mod.rs' = '1efec934afcb61d633e36d35a36702222d8b1f28'
      'crates/planrun-exec/tests/shell.rs' = '30a78574072d54852d10bd25ed797100cfcc822b'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6b-T3: shell::end_now" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P6b-T4: typed effects in the dry-run report

**Goal:** The dry-run report says what a file action does as data: `handler::Effect` (create, overwrite, a patch with its blob ids, replace, delete) in `Simulated` and in `Outcome::Ok` and `Outcome::WouldAsk`, for the TUI's Inspect mode (D84). Its `Display` is the text the report prints, so the CLI's output does not change.

**Files**

- Modify `crates/planrun-exec/src/handler.rs`
- Modify `crates/planrun-exec/src/preflight.rs`
- Modify `crates/planrun-exec/tests/preflight.rs`
- Modify `crates/planrun-exec/tests/simulate.rs`

Over 300 lines, so they come as exact diffs: `crates/planrun-exec/src/handler.rs`, `crates/planrun-exec/src/preflight.rs`, `crates/planrun-exec/tests/preflight.rs`.

**Step 1 — write the files**

File `crates/planrun-exec/src/handler.rs` (exact diff; save the block as `$env:TEMP\p6b-t4-src-handler-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/handler.rs b/crates/planrun-exec/src/handler.rs
index 27b5349..4a05a34 100644
--- a/crates/planrun-exec/src/handler.rs
+++ b/crates/planrun-exec/src/handler.rs
@@ -2,6 +2,8 @@
 //! plugin point, docs/extending.md §7). A handler simulates its action on a virtual tree (the
 //! dry-run behaviour of extending.md §4); writing is the transaction's job (D53).
 
+use std::fmt;
+
 use planrun_model::{AbbrevBlobId, Action, ActionKind, PlanSource, RepoPath, Span, WriteMode};
 
 use crate::eol;
@@ -13,8 +15,47 @@ use crate::tree::{State, VirtualTree};
 pub struct Simulated {
     /// The path it changed.
     pub path: RepoPath,
-    /// A short description (`create`, `aa80cf1 → 97bd92a`).
-    pub detail: String,
+    /// What it did to the path.
+    pub effect: Effect,
+}
+
+/// What a file action does to its path. Its `Display` is the dry run's short description
+/// (`create`, `aa80cf1 → 97bd92a`).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Effect {
+    /// Full contents for a path that does not exist.
+    Create,
+    /// Full contents for a path that exists.
+    Overwrite,
+    /// A patch, with the blob ids the plan states before and after it.
+    Patch {
+        /// The `index` line's pre-image id, when the patch has one.
+        pre: Option<AbbrevBlobId>,
+        /// Its post-image id.
+        post: Option<AbbrevBlobId>,
+    },
+    /// An exact replacement.
+    Replace,
+    /// A delete; `via_git` for `git rm`.
+    Delete {
+        /// Through `git rm`.
+        via_git: bool,
+    },
+}
+
+impl fmt::Display for Effect {
+    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
+        let id =
+            |a: &Option<AbbrevBlobId>| a.as_ref().map_or_else(|| "?".to_owned(), |a| a.to_string());
+        match self {
+            Effect::Create => f.write_str("create"),
+            Effect::Overwrite => f.write_str("overwrite"),
+            Effect::Patch { pre, post } => write!(f, "{} → {}", id(pre), id(post)),
+            Effect::Replace => f.write_str("replace"),
+            Effect::Delete { via_git: true } => f.write_str("delete (git rm)"),
+            Effect::Delete { via_git: false } => f.write_str("delete"),
+        }
+    }
 }
 
 /// Executes one kind of action. Registered per kind in the [`HandlerSet`].
@@ -147,16 +188,16 @@ impl ActionHandler for WriteHandler {
         }
         let present = presence(tree, path, span)?;
         let body = source.resolve(body).as_bytes();
-        let (bytes, detail) = match mode {
+        let (bytes, effect) = match mode {
             WriteMode::Create if present => return Err(exists(path, span)),
-            WriteMode::Create => (body.to_vec(), "create"),
+            WriteMode::Create => (body.to_vec(), Effect::Create),
             _ if !present => return Err(missing(path, span, "; the plan overwrites it")),
-            _ => (overwrite_bytes(tree, path, body, span)?, "overwrite"),
+            _ => (overwrite_bytes(tree, path, body, span)?, Effect::Overwrite),
         };
         tree.put(path, bytes, span)?;
         Ok(Simulated {
             path: path.clone(),
-            detail: detail.to_owned(),
+            effect,
         })
     }
 }
@@ -212,10 +253,12 @@ impl ActionHandler for PatchHandler {
             )
         })?;
         check_post(tree, path, post.as_ref(), span)?;
-        let id = |a: Option<&AbbrevBlobId>| a.map_or_else(|| "?".to_owned(), |a| a.to_string());
         Ok(Simulated {
             path: path.clone(),
-            detail: format!("{} → {}", id(pre.as_ref()), id(post.as_ref())),
+            effect: Effect::Patch {
+                pre: *pre,
+                post: *post,
+            },
         })
     }
 }
@@ -331,7 +374,7 @@ impl ActionHandler for ReplaceHandler {
         tree.put(path, eol::render(&replaced, convention), span)?;
         Ok(Simulated {
             path: path.clone(),
-            detail: "replace".to_owned(),
+            effect: Effect::Replace,
         })
     }
 }
@@ -382,12 +425,7 @@ impl ActionHandler for DeleteHandler {
         tree.delete(path, span)?;
         Ok(Simulated {
             path: path.clone(),
-            detail: if *via_git {
-                "delete (git rm)"
-            } else {
-                "delete"
-            }
-            .to_owned(),
+            effect: Effect::Delete { via_git: *via_git },
         })
     }
 }
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t4-src-handler-rs.patch"
Remove-Item "$env:TEMP\p6b-t4-src-handler-rs.patch"
```

File `crates/planrun-exec/src/preflight.rs` (exact diff; save the block as `$env:TEMP\p6b-t4-src-preflight-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/preflight.rs b/crates/planrun-exec/src/preflight.rs
index 1701b7b..e0063cf 100644
--- a/crates/planrun-exec/src/preflight.rs
+++ b/crates/planrun-exec/src/preflight.rs
@@ -13,17 +13,17 @@ use planrun_model::{
 };
 
 use crate::finding::{Finding, Severity};
-use crate::handler::{HandlerSet, file_path};
+use crate::handler::{Effect, HandlerSet, file_path};
 use crate::tree::VirtualTree;
 use crate::workspace::{Workspace, WorkspaceError};
 
 /// What became of one action in a dry run.
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub enum Outcome {
-    /// A file action that would run on its own.
-    Ok(String),
+    /// A file action that would run on its own, and what it would do.
+    Ok(Effect),
     /// A file action that would run after you confirm it: unscoped (D50).
-    WouldAsk(String),
+    WouldAsk(Effect),
     /// A file action that would fail.
     Failed(Finding),
     /// A file action whose result depends on a script or Manual step that did not run (D55).
@@ -322,8 +322,8 @@ impl Walk<'_, '_> {
             return Outcome::Failed(Finding::error("internal", span, Some(path), "no handler"));
         };
         match handler.simulate(action, &plan.source, self.tree) {
-            Ok(done) if task.manifest.scopes(path) => Outcome::Ok(done.detail),
-            Ok(done) => Outcome::WouldAsk(done.detail),
+            Ok(done) if task.manifest.scopes(path) => Outcome::Ok(done.effect),
+            Ok(done) => Outcome::WouldAsk(done.effect),
             Err(f) if f.severity == Severity::Warning => Outcome::Depends(f),
             Err(f) if self.unrun && depends_on_state(f.code) => {
                 self.tree.mark_unknown(path);
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t4-src-preflight-rs.patch"
Remove-Item "$env:TEMP\p6b-t4-src-preflight-rs.patch"
```

File `crates/planrun-exec/tests/preflight.rs` (exact diff; save the block as `$env:TEMP\p6b-t4-tests-preflight-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/tests/preflight.rs b/crates/planrun-exec/tests/preflight.rs
index c2bc331..dcd638a 100644
--- a/crates/planrun-exec/tests/preflight.rs
+++ b/crates/planrun-exec/tests/preflight.rs
@@ -8,9 +8,10 @@ mod ir;
 
 use fixture::Fixture;
 use ir::{Ir, PATCH_T1, PATCH_T2};
+use planrun_exec::handler::Effect;
 use planrun_exec::preflight::{DryRun, Outcome, dry_run, preflight};
 use planrun_exec::workspace::Workspace;
-use planrun_model::{ChangeKind, Manifest, PathOrigin, Plan, WriteMode};
+use planrun_model::{AbbrevBlobId, ChangeKind, Manifest, PathOrigin, Plan, WriteMode};
 
 fn open(f: &Fixture) -> Workspace {
     Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
@@ -93,6 +94,43 @@ fn the_assay_plan_passes_and_writes_nothing() {
     }
 }
 
+#[test]
+fn the_report_says_what_each_file_action_does() {
+    let f = Fixture::greeter(false);
+    let plan = assay(&mut Ir::default());
+    let report = run(&f, &plan);
+    let effects: Vec<Effect> = report
+        .tasks
+        .iter()
+        .flat_map(|t| &t.actions)
+        .filter_map(|a| match &a.outcome {
+            Outcome::Ok(effect) => Some(effect.clone()),
+            _ => None,
+        })
+        .collect();
+    let id = |hex: &str| Some(AbbrevBlobId::parse(hex).unwrap_or_else(|e| panic!("{e}")));
+    assert_eq!(
+        effects,
+        [
+            Effect::Create,
+            Effect::Patch {
+                pre: id("aa80cf1"),
+                post: id("97bd92a")
+            },
+            Effect::Patch {
+                pre: id("97bd92a"),
+                post: id("6b269fc")
+            },
+            Effect::Delete { via_git: false },
+        ]
+    );
+    let shown: Vec<String> = effects.iter().map(ToString::to_string).collect();
+    assert_eq!(
+        shown,
+        ["create", "aa80cf1 → 97bd92a", "97bd92a → 6b269fc", "delete"]
+    );
+}
+
 #[test]
 fn preflight_finds_the_blob_mismatch_of_a_changed_file() {
     let f = Fixture::greeter(false);
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t4-tests-preflight-rs.patch"
Remove-Item "$env:TEMP\p6b-t4-tests-preflight-rs.patch"
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
                Ok(s) => format!("ok {}", s.effect),
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

**Step 2 — verify**

Targeted: the dry-run and simulation tests:

```powershell
cargo test -p planrun-exec --test preflight --test simulate
```

Expected (exit code 0; summary lines):

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-exec/src/handler.rs crates/planrun-exec/src/preflight.rs crates/planrun-exec/tests/preflight.rs crates/planrun-exec/tests/simulate.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/handler.rs' = '4a05a34c90d5428f5b4c45069a01775efad4d04c'
      'crates/planrun-exec/src/preflight.rs' = 'e0063cf52d5ccceeb4be2e05317359e88ff15b8d'
      'crates/planrun-exec/tests/preflight.rs' = 'dcd638ad91625a9e48643b32342433457d932209'
      'crates/planrun-exec/tests/simulate.rs' = 'a1f5369ddda068822dd20a4f24de1b63e1bfc345'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6b-T4: typed effects in the dry-run report" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P6b-T5: the matcher's alignment

**Goal:** `expect::align` gives each non-blank expected summary line the output line it matched, or none, the way `check` matches them (D66, D86), for the TUI's mismatch dock. `check` and `align` share one matcher; `check`'s result does not change.

**Files**

- Modify `crates/planrun-exec/src/expect.rs`

Over 300 lines, so it comes as an exact diff: `crates/planrun-exec/src/expect.rs`.

**Step 1 — write the files**

File `crates/planrun-exec/src/expect.rs` (exact diff; save the block as `$env:TEMP\p6b-t5-src-expect-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/expect.rs b/crates/planrun-exec/src/expect.rs
index 4e09c23..442bf24 100644
--- a/crates/planrun-exec/src/expect.rs
+++ b/crates/planrun-exec/src/expect.rs
@@ -63,6 +63,29 @@ pub enum Mismatch {
     },
 }
 
+/// One non-blank line of the expected block, and the output line it matched.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Aligned {
+    /// Its 1-based line number within the expected block.
+    pub number: usize,
+    /// The index into [`ScriptRun::lines`] of the output line it matched; `None` when no output
+    /// line after the previous match matches it.
+    pub output: Option<usize>,
+}
+
+/// The expected summary lines of `expect` against `run`'s output, as [`check`] matches them
+/// (D66): each non-blank line with the output line it matched, for a UI to show the two side
+/// by side. Empty when the plan expects no lines.
+pub fn align(expect: Option<&Expectation>, run: &ScriptRun, source: &PlanSource) -> Vec<Aligned> {
+    let Some(lines) = expect.and_then(|e| e.lines.as_ref()) else {
+        return Vec::new();
+    };
+    aligned(source.resolve(lines), run)
+        .into_iter()
+        .map(|(a, _)| a)
+        .collect()
+}
+
 /// Checks `run` against `expect` (`None`: the plan expects nothing in particular).
 pub fn check(expect: Option<&Expectation>, run: &ScriptRun, source: &PlanSource) -> Verdict {
     let mut mismatches = Vec::new();
@@ -123,30 +146,42 @@ fn reasons(expect: Option<&Expectation>, exit: ExitExpect, run: &ScriptRun) -> V
     reasons
 }
 
-/// The expected lines that do not match, as a subsequence of the output: each non-blank line
-/// must match an output line after the one the line before it matched. The earliest match is
-/// taken for each, which finds a subsequence whenever one exists.
+/// The expected lines that do not match.
 fn unmatched(expected: &str, run: &ScriptRun) -> Vec<Mismatch> {
+    aligned(expected, run)
+        .into_iter()
+        .filter(|(a, _)| a.output.is_none())
+        .map(|(a, line)| Mismatch::Line {
+            number: a.number,
+            line: line.to_owned(),
+        })
+        .collect()
+}
+
+/// The expected lines as a subsequence of the output: each non-blank line must match an output
+/// line after the one the line before it matched. The earliest match is taken for each, which
+/// finds a subsequence whenever one exists. A line that matches nothing leaves the position
+/// where it was.
+fn aligned<'e>(expected: &'e str, run: &ScriptRun) -> Vec<(Aligned, &'e str)> {
     let output: Vec<Cow<'_, str>> = run.lines.iter().map(|l| strip_ansi(&l.text)).collect();
     let mut next = 0;
-    let mut missing = Vec::new();
+    let mut found = Vec::new();
     for (i, line) in expected.split('\n').enumerate() {
         let pattern = line.trim_end();
         if pattern.is_empty() {
             continue;
         }
-        let found = output[next..]
+        let at = output[next..]
             .iter()
-            .position(|out| line_matches(pattern, out.trim_end()));
-        match found {
-            Some(at) => next += at + 1,
-            None => missing.push(Mismatch::Line {
-                number: i + 1,
-                line: line.to_owned(),
-            }),
+            .position(|out| line_matches(pattern, out.trim_end()))
+            .map(|at| next + at);
+        if let Some(at) = at {
+            next = at + 1;
         }
+        let number = i + 1;
+        found.push((Aligned { number, output: at }, line));
     }
-    missing
+    found
 }
 
 /// Whether `line` matches `pattern`: equal, except that each `…` matches any run of characters,
@@ -315,6 +350,47 @@ mod tests {
         }
     }
 
+    #[test]
+    fn alignment() {
+        let mut text = String::from("ok: …\n\nmissing\ndone\n");
+        let lines = TextRef::Span(Span::new(0, text.len() as u32));
+        text.push_str("tail");
+        let source = PlanSource::from_normalized("t.md", text, String::new());
+        let expect = Expectation {
+            exit: ExitExpect::Success,
+            lines: Some(lines),
+            prose: None,
+            comments: None,
+        };
+        let out = ["noise", "ok: 3", "done", "done"];
+        let run = ScriptRun {
+            status: RunStatus::Exited(0),
+            errored: false,
+            lines: out
+                .iter()
+                .map(|t| OutLine {
+                    stream: Stream::Stdout,
+                    text: (*t).to_owned(),
+                    newline: true,
+                })
+                .collect(),
+            cwd: None,
+        };
+        let shown: Vec<(usize, Option<usize>)> = align(Some(&expect), &run, &source)
+            .iter()
+            .map(|a| (a.number, a.output))
+            .collect();
+        assert_eq!(shown, [(1, Some(1)), (3, None), (4, Some(2))]);
+        assert_eq!(
+            check(Some(&expect), &run, &source),
+            Verdict::Mismatch(vec![Mismatch::Line {
+                number: 3,
+                line: "missing".to_owned()
+            }])
+        );
+        assert!(align(None, &run, &source).is_empty());
+    }
+
     #[test]
     fn verdicts() {
         use ExitExpect::{Code, Failure, Success, Unknown};
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t5-src-expect-rs.patch"
Remove-Item "$env:TEMP\p6b-t5-src-expect-rs.patch"
```

**Step 2 — verify**

Targeted: the matcher's unit tests:

```powershell
cargo test -p planrun-exec --lib expect::tests
```

Expected (exit code 0; summary lines):

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-exec/src/expect.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/expect.rs' = '442bf2491f7dfffedf0cfb8657ab848d25ed89b6'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6b-T5: the matcher's alignment" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P6b-T6: QuestionKind

**Goal:** `Question.kind` becomes `QuestionKind`: one variant per question the runner asks, carrying what the run found (the decision, the classifier's reasons, the matcher's reasons, mismatches, the block's output and the alignment into it, the commit problem, dirty paths, the session's directory, the gate's reasons, changed paths, the step's or action's journal place with step states or both HEADs), with `name()` for the journal, which does not change (D86). `Question.typed` becomes the words a typed question accepts (review ruling 2). The line UI and the scripted test UIs read `name()`. The two `resume-dirt` strings get their line continuation back (handover item 5).

**Files**

- Modify `crates/planrun-cli/src/ui.rs`
- Modify `crates/planrun-exec/src/preflight.rs`
- Modify `crates/planrun-exec/src/runner/mod.rs`
- Modify `crates/planrun-exec/src/runner/resume.rs`
- Modify `crates/planrun-exec/src/runner/script.rs`
- Modify `crates/planrun-exec/src/runner/task.rs`
- Modify `crates/planrun-exec/src/ui.rs`
- Modify `crates/planrun-exec/tests/resume.rs`
- Modify `crates/planrun-exec/tests/runner.rs`

Over 300 lines, so they come as exact diffs: `crates/planrun-exec/src/preflight.rs`, `crates/planrun-exec/src/runner/mod.rs`, `crates/planrun-exec/src/runner/resume.rs`, `crates/planrun-exec/src/runner/script.rs`, `crates/planrun-exec/src/runner/task.rs`, `crates/planrun-exec/tests/resume.rs`, `crates/planrun-exec/tests/runner.rs`.

**Step 1 — write the files**

File `crates/planrun-cli/src/ui.rs` (full contents):

```rust
//! The line-prompt [`Ui`] (decisions D15, D74): plain lines on stdout, so cargo's output stays
//! in the terminal's scrollback, and answers read a line at a time from stdin. Only a terminal
//! answers: when stdin is not one, the first question stops the run (journaled), never with a
//! default. The `e2e` feature lifts that for planrun-cli's own end-to-end tests (D82).

use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::process::Command;

use planrun_exec::shell::{InterruptsIgnored, OutLine};
use planrun_exec::ui::{Choice, Position, Question, Reply, Ui};
use planrun_model::Plan;

use crate::view::{self, Detail, Viewer};

/// Line prompts on the terminal.
pub struct LineUi<'p> {
    plan: &'p Plan,
    root: PathBuf,
}

impl<'p> LineUi<'p> {
    /// Prompts for `plan`, run in the repository at `root`.
    pub fn new(plan: &'p Plan, root: PathBuf) -> LineUi<'p> {
        LineUi { plan, root }
    }

    fn print(&self, text: &str) {
        let mut out = io::stdout().lock();
        let _ = writeln!(out, "{text}");
        let _ = out.flush();
    }

    /// A line from stdin without its line ending (`\n` or `\r\n`), nothing else removed;
    /// `None` at end of input.
    fn read_line(&self) -> Option<String> {
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => None,
            Ok(_) => {
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Some(line)
            }
        }
    }
}

/// Whether answers may come from something that is not a terminal (the `e2e` feature only).
fn piped_answers_allowed() -> bool {
    cfg!(feature = "e2e")
}

/// The prompt line: `[enter] run  [s] skip  …`.
fn prompt(choices: &[Choice]) -> String {
    let parts: Vec<String> = choices
        .iter()
        .map(|c| match c.key() {
            "" => format!("[enter] {}", c.label()),
            key => format!("[{key}] {}", c.label()),
        })
        .collect();
    parts.join("   ")
}

impl Ui for LineUi<'_> {
    fn status(&mut self, line: &str) {
        self.print(line);
    }

    fn output(&mut self, line: &OutLine) {
        self.print(&format!("    {}", line.text.trim_end_matches('\r')));
    }

    fn ask(&mut self, question: &Question) -> Reply {
        if !io::stdin().is_terminal() && !piped_answers_allowed() {
            return Reply::Stop(format!(
                "planrun needs an answer (`{}`), and stdin is not a terminal",
                question.kind.name()
            ));
        }
        self.print("");
        for line in &question.text {
            self.print(line);
        }
        loop {
            if question.typed.is_none() {
                self.print(&prompt(&question.choices));
            }
            let _ = io::stdout().flush();
            let Some(line) = self.read_line() else {
                return Reply::Stop("stdin closed before an answer".to_owned());
            };
            if question.typed.is_some() {
                return Reply::Typed(line);
            }
            let found = question.choices.iter().find(|c| c.key() == line.trim());
            if let Some(choice) = found {
                return Reply::Choice(*choice);
            }
            self.print(&format!("`{line}` is not one of the answers"));
        }
    }

    fn view(&mut self, at: Position) {
        let Some(step) = self
            .plan
            .tasks
            .get(at.task)
            .and_then(|t| t.steps.get(at.step))
        else {
            return;
        };
        let viewer = Viewer::new(self.plan);
        let mut out = String::new();
        view::step_text(&viewer.step(step), Detail::Full, &mut out);
        self.print(out.trim_end());
    }

    fn diff(&mut self) {
        let run = |args: &[&str]| {
            let _ = Command::new("git")
                .args(args)
                .current_dir(&self.root)
                .status();
        };
        run(&["--no-optional-locks", "--no-pager", "diff", "HEAD"]);
        self.print("untracked:");
        run(&[
            "--no-optional-locks",
            "ls-files",
            "--others",
            "--exclude-standard",
        ]);
    }

    fn shell(&mut self) {
        let _quiet = InterruptsIgnored::new();
        let program = if cfg!(windows) { "pwsh" } else { "bash" };
        self.print(&format!(
            "{program} at {}; this is not the plan's session (its variables are not here). Exit to return.",
            self.root.display()
        ));
        let started = Command::new(program).current_dir(&self.root).status();
        if let Err(e) = started {
            self.print(&format!("{program} did not start: {e}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prompt_names_each_key() {
        let line = prompt(&[Choice::Run, Choice::Skip, Choice::Quit]);
        assert_eq!(line, "[enter] run   [s] skip   [q] quit");
    }
}
```

File `crates/planrun-exec/src/preflight.rs` (exact diff; save the block as `$env:TEMP\p6b-t6-src-preflight-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/preflight.rs b/crates/planrun-exec/src/preflight.rs
index e0063cf..436d8a3 100644
--- a/crates/planrun-exec/src/preflight.rs
+++ b/crates/planrun-exec/src/preflight.rs
@@ -187,7 +187,8 @@ pub fn preflight_resumed(
         } else if remaining.contains(path) {
             f.code = "resume-dirt";
             f.message = format!(
-                "{} (not a verified write of the interrupted run; a remaining file action                  writes it)",
+                "{} (not a verified write of the interrupted run; a remaining file action \
+                 writes it)",
                 f.message
             );
         } else {
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t6-src-preflight-rs.patch"
Remove-Item "$env:TEMP\p6b-t6-src-preflight-rs.patch"
```

File `crates/planrun-exec/src/runner/mod.rs` (exact diff; save the block as `$env:TEMP\p6b-t6-runner-mod-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/runner/mod.rs b/crates/planrun-exec/src/runner/mod.rs
index e292f97..646fda7 100644
--- a/crates/planrun-exec/src/runner/mod.rs
+++ b/crates/planrun-exec/src/runner/mod.rs
@@ -23,7 +23,7 @@ use crate::handler::{HandlerSet, file_path};
 use crate::journal::{At, End, Journal, Record};
 use crate::policy::{DefaultClassifier, Policy, PolicyClass};
 use crate::shell::{SessionConfig, ShellError};
-use crate::ui::{Choice, Position, Question, Reply, Ui};
+use crate::ui::{Choice, Position, Question, QuestionKind, Reply, Ui};
 use crate::workspace::{Workspace, WorkspaceError};
 
 use sessions::Sessions;
@@ -89,7 +89,7 @@ pub struct Runner<'r> {
     handlers: HandlerSet,
     sessions: Sessions,
     /// A notice to acknowledge before the next block: the session is new (D81).
-    notice: Option<Vec<String>>,
+    notice: Option<Notice>,
     /// Tasks whose scripts ran before, for the notice.
     earlier: Vec<String>,
     /// What resume replays of the interrupted task (D77).
@@ -147,7 +147,7 @@ impl<'r> Runner<'r> {
                 .iter()
                 .map(|t| t.id.raw.to_string())
                 .collect();
-            self.notice = Some(self.notice_text("the run starts at a later task (--from)"));
+            self.notice = Some(self.notice("the run starts at a later task (--from)"));
         }
         self.tasks_from(from)
     }
@@ -213,21 +213,12 @@ impl<'r> Runner<'r> {
         self.plan.source.line_col(action.span.start).line
     }
 
-    /// The notice that a session is new (D81): what is lost, and which tasks' scripts ran
-    /// before.
-    fn notice_text(&self, why: &str) -> Vec<String> {
-        let mut text = vec![
-            format!("This is a new shell session: {why}."),
-            "Variables, functions and the working directory set by earlier blocks are gone."
-                .to_owned(),
-        ];
-        if !self.earlier.is_empty() {
-            text.push(format!(
-                "Scripts of these earlier tasks ran in another session: {}.",
-                self.earlier.join(", ")
-            ));
+    /// The notice that a session is new (D81), with the tasks whose scripts ran before.
+    fn notice(&self, why: &str) -> Notice {
+        Notice {
+            why: why.to_owned(),
+            earlier: self.earlier.clone(),
         }
-        text
     }
 
     /// Asks `question` until it gets a choice or a typed line: `[v]`, `[d]` and `[o]` are
@@ -265,14 +256,15 @@ impl<'r> Runner<'r> {
         let at = question.at.map(|p| self.at(p));
         self.append(Record::Answer {
             at,
-            question: question.kind.to_owned(),
+            question: question.kind.name().to_owned(),
             answer: answer.to_owned(),
         })
     }
 
-    /// Asks for a typed `yes` (D74): exactly `yes` agrees, `quit` stops, `skip` skips when
-    /// `skippable`; anything else does not run and asks again.
-    fn typed_yes(&mut self, question: Question, skippable: bool) -> Result<Typed> {
+    /// Asks for a typed `yes` (D74): exactly `yes` agrees, `quit` stops, `skip` skips when the
+    /// question's words offer it; anything else does not run and asks again.
+    fn typed_yes(&mut self, question: Question) -> Result<Typed> {
+        let skippable = question.typed.is_some_and(|words| words.contains(&"skip"));
         loop {
             match self.ask(question.clone())? {
                 Answer::Typed(line) if line == "yes" => return Ok(Typed::Yes),
@@ -290,15 +282,18 @@ impl<'r> Runner<'r> {
 
     /// Asks the pending session notice, if any; it is acknowledged even with `--auto` (D81).
     fn acknowledge_notice(&mut self, at: Option<Position>) -> Result<Flow> {
-        let Some(text) = self.notice.take() else {
+        let Some(notice) = self.notice.take() else {
             return Ok(Flow::Go);
         };
         let question = Question {
-            kind: "session",
+            text: notice.text(),
+            kind: QuestionKind::Session {
+                why: notice.why,
+                earlier: notice.earlier,
+            },
             at,
-            text,
             choices: vec![Choice::Continue, Choice::Shell, Choice::Quit],
-            typed: false,
+            typed: None,
         };
         Ok(match self.ask(question)? {
             Answer::Choice(Choice::Continue) => Flow::Go,
@@ -308,6 +303,31 @@ impl<'r> Runner<'r> {
     }
 }
 
+/// A pending session notice (D81): why the session is new, and the tasks whose scripts ran
+/// before.
+pub(crate) struct Notice {
+    why: String,
+    earlier: Vec<String>,
+}
+
+impl Notice {
+    /// What is lost, and which tasks' scripts ran before.
+    fn text(&self) -> Vec<String> {
+        let mut text = vec![
+            format!("This is a new shell session: {}.", self.why),
+            "Variables, functions and the working directory set by earlier blocks are gone."
+                .to_owned(),
+        ];
+        if !self.earlier.is_empty() {
+            text.push(format!(
+                "Scripts of these earlier tasks ran in another session: {}.",
+                self.earlier.join(", ")
+            ));
+        }
+        text
+    }
+}
+
 /// An answer the runner acts on.
 pub(crate) enum Answer {
     Choice(Choice),
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t6-runner-mod-rs.patch"
Remove-Item "$env:TEMP\p6b-t6-runner-mod-rs.patch"
```

File `crates/planrun-exec/src/runner/resume.rs` (exact diff; save the block as `$env:TEMP\p6b-t6-runner-resume-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/runner/resume.rs b/crates/planrun-exec/src/runner/resume.rs
index d3f468b..16ccb8b 100644
--- a/crates/planrun-exec/src/runner/resume.rs
+++ b/crates/planrun-exec/src/runner/resume.rs
@@ -23,7 +23,7 @@ use crate::git::Head;
 use crate::journal::{At, End, Line, Outcome, PathBlob, Record};
 use crate::preflight::Resumed;
 use crate::transaction::StepRecord;
-use crate::ui::{Choice, Question};
+use crate::ui::{Choice, Question, QuestionKind, YES_OR_QUIT};
 
 /// What the run record of the chain says.
 #[derive(Debug, Clone, PartialEq, Eq)]
@@ -315,7 +315,7 @@ impl Runner<'_> {
             .iter()
             .map(|t| t.id.raw.to_string())
             .collect();
-        self.notice = Some(self.notice_text("planrun resumes an interrupted run"));
+        self.notice = Some(self.notice("planrun resumes an interrupted run"));
         if analysis.interrupted.is_some() {
             self.replay = Some(replay);
         }
@@ -367,7 +367,11 @@ impl Runner<'_> {
             return Ok(Flow::Go);
         }
         let question = Question {
-            kind: "head-moved",
+            kind: QuestionKind::HeadMoved {
+                action: at.clone(),
+                recorded: head.clone(),
+                now: now.clone(),
+            },
             at: None,
             text: vec![
                 format!(
@@ -381,7 +385,7 @@ impl Runner<'_> {
             ],
             // No `[o]`: it journals HEAD as seen (D80), which would pass this check unanswered.
             choices: vec![Choice::Keep, Choice::Retry, Choice::Diff, Choice::Quit],
-            typed: false,
+            typed: None,
         };
         Ok(match self.ask(question)? {
             Answer::Choice(Choice::Keep) => {
@@ -499,11 +503,15 @@ impl Runner<'_> {
         }
         choices.extend([Choice::Accept, Choice::Diff, Choice::Shell, Choice::Quit]);
         let question = Question {
-            kind: "unknown",
+            kind: QuestionKind::Unknown {
+                step: at.clone(),
+                paths: paths.to_vec(),
+                restorable: can_restore,
+            },
             at: None,
             text,
             choices,
-            typed: false,
+            typed: None,
         };
         match self.ask(question)? {
             Answer::Choice(Choice::Restore) => {
@@ -519,16 +527,16 @@ impl Runner<'_> {
             }
             Answer::Choice(Choice::Accept) => {
                 let question = Question {
-                    kind: "accept-done",
+                    kind: QuestionKind::AcceptDone { step: at.clone() },
                     at: None,
                     text: vec![
                         "accepting the step as done overrides the disk check".to_owned(),
                         "type `yes` to accept it, or `quit`".to_owned(),
                     ],
                     choices: Vec::new(),
-                    typed: true,
+                    typed: Some(YES_OR_QUIT),
                 };
-                match self.typed_yes(question, false)? {
+                match self.typed_yes(question)? {
                     Typed::Yes => {
                         self.append(Record::StepAccepted { at: at.clone() })?;
                         Ok(Ok(true))
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t6-runner-resume-rs.patch"
Remove-Item "$env:TEMP\p6b-t6-runner-resume-rs.patch"
```

File `crates/planrun-exec/src/runner/script.rs` (exact diff; save the block as `$env:TEMP\p6b-t6-runner-script-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/runner/script.rs b/crates/planrun-exec/src/runner/script.rs
index 927aec1..7225c5b 100644
--- a/crates/planrun-exec/src/runner/script.rs
+++ b/crates/planrun-exec/src/runner/script.rs
@@ -9,19 +9,24 @@ use super::resume::Past;
 use super::sessions::{NotRun, name};
 use super::task::TaskRun;
 use super::{Answer, Flow, Result, RunError, Runner, Typed, canonical, describe};
-use crate::expect::{self, Mismatch, Reason, Verdict};
+use crate::expect::{self, Aligned, Confirm, Mismatch, Reason, Verdict};
 use crate::journal::{Outcome, PathBlob, Record};
 use crate::policy::{Decision, PolicyClass, Skip};
 use crate::shell::{RunStatus, ScriptRun};
-use crate::ui::{Choice, Position, Question};
+use crate::ui::{Choice, Position, Question, QuestionKind, RUN_SKIP_OR_QUIT};
 
 /// What the matcher, and for a commit HEAD, concluded.
 enum Judged {
     Pass,
-    Confirm(Vec<String>),
-    /// What does not hold; `dead` when the session ended, timed out or broke.
+    /// Why it asks, and the lines that say so.
+    Confirm(Confirm, Vec<String>),
+    /// What does not hold: the matcher's mismatches, the expected lines against the output,
+    /// and for a commit why it does not count as made; `dead` when the session ended, timed
+    /// out or broke.
     Mismatch {
-        lines: Vec<String>,
+        mismatches: Vec<Mismatch>,
+        aligned: Vec<Aligned>,
+        commit: Option<String>,
         dead: bool,
     },
 }
@@ -149,6 +154,7 @@ impl Runner<'_> {
             ));
             return Ok(Replayed::Done);
         }
+        let running = matches!(past, Past::Running { .. });
         let why = match past {
             Past::Ended(Outcome::Kept) => return Ok(Replayed::Keep),
             Past::Ended(Outcome::SkippedPolicy) => return Ok(Replayed::Again),
@@ -169,7 +175,7 @@ impl Runner<'_> {
             }
         };
         let question = Question {
-            kind: "again",
+            kind: QuestionKind::Again { running },
             at: Some(pos),
             text: vec![
                 format!("L{line} {} {why}", describe(action)),
@@ -184,7 +190,7 @@ impl Runner<'_> {
                 Choice::Shell,
                 Choice::Quit,
             ],
-            typed: false,
+            typed: None,
         };
         Ok(match self.ask(question)? {
             Answer::Choice(Choice::Retry) => Replayed::Again,
@@ -262,13 +268,15 @@ impl Runner<'_> {
                 text.extend(found.reasons.iter().map(|r| format!("  because: {r}")));
                 text.push("type `yes` to run it, `skip` to skip it, `quit` to stop".to_owned());
                 let question = Question {
-                    kind: "typed-yes",
+                    kind: QuestionKind::TypedYes {
+                        reasons: found.reasons,
+                    },
                     at: Some(pos),
                     text,
                     choices: Vec::new(),
-                    typed: true,
+                    typed: Some(RUN_SKIP_OR_QUIT),
                 };
-                Ok(match self.typed_yes(question, true)? {
+                Ok(match self.typed_yes(question)? {
                     Typed::Yes => Permit::Run,
                     Typed::Skip => Permit::Skip,
                     Typed::Quit => Permit::Stop("quit".to_owned()),
@@ -283,7 +291,7 @@ impl Runner<'_> {
                 let mut text = vec![format!("{head} ({class})")];
                 text.extend(body.lines().take(6).map(|l| format!("  | {l}")));
                 let question = Question {
-                    kind: "run",
+                    kind: QuestionKind::Run { decision },
                     at: Some(pos),
                     text,
                     choices: vec![
@@ -294,7 +302,7 @@ impl Runner<'_> {
                         Choice::Shell,
                         Choice::Quit,
                     ],
-                    typed: false,
+                    typed: None,
                 };
                 Ok(match self.ask(question)? {
                     Answer::Choice(Choice::Run) => Permit::Run,
@@ -313,7 +321,7 @@ impl Runner<'_> {
         if let Some(restart) = self.sessions.starts(script.shell) {
             if restart && self.notice.is_none() {
                 let why = format!("the {} session ended", name(script.shell));
-                self.notice = Some(self.notice_text(&why));
+                self.notice = Some(self.notice(&why));
             }
             let reason = if restart { "restarted" } else { "started" };
             if let Flow::Stop(why) = self.acknowledge_notice(Some(pos))? {
@@ -333,7 +341,10 @@ impl Runner<'_> {
             return Ok(Flow::Go);
         }
         let question = Question {
-            kind: "outside",
+            kind: QuestionKind::Outside {
+                shell: script.shell,
+                cwd: cwd.clone(),
+            },
             at: Some(pos),
             text: vec![format!(
                 "the {} session's working directory is {}, outside the repository; the next \
@@ -342,7 +353,7 @@ impl Runner<'_> {
                 cwd.display()
             )],
             choices: vec![Choice::Continue, Choice::View, Choice::Shell, Choice::Quit],
-            typed: false,
+            typed: None,
         };
         Ok(match self.ask(question)? {
             Answer::Choice(Choice::Continue) => Flow::Go,
@@ -388,21 +399,22 @@ impl Runner<'_> {
             ran.status,
             RunStatus::SessionEnded(_) | RunStatus::TimedOut | RunStatus::Broken(_)
         );
-        let verdict = expect::check(script.expect.as_ref(), ran, &self.plan.source);
+        let source = &self.plan.source;
+        let verdict = expect::check(script.expect.as_ref(), ran, source);
         let problem = if commit && !dead {
             self.commit_problem(before, expect)?
         } else {
             None
         };
+        let mismatch = |mismatches, commit| Judged::Mismatch {
+            mismatches,
+            aligned: expect::align(script.expect.as_ref(), ran, source),
+            commit,
+            dead,
+        };
         Ok(match (verdict, problem) {
-            (Verdict::Mismatch(ms), problem) => Judged::Mismatch {
-                lines: ms.iter().map(mismatch_line).chain(problem).collect(),
-                dead,
-            },
-            (_, Some(problem)) => Judged::Mismatch {
-                lines: vec![problem],
-                dead,
-            },
+            (Verdict::Mismatch(ms), problem) => mismatch(ms, problem),
+            (_, Some(problem)) => mismatch(Vec::new(), Some(problem)),
             (Verdict::Pass, None) => Judged::Pass,
             (Verdict::Confirm(c), None) => {
                 let mut lines: Vec<String> = c
@@ -411,7 +423,7 @@ impl Runner<'_> {
                     .map(|r| self.reason_line(script, *r))
                     .collect();
                 lines.push(format!("errored: {}", c.errored));
-                Judged::Confirm(lines)
+                Judged::Confirm(c, lines)
             }
         })
     }
@@ -452,10 +464,13 @@ impl Runner<'_> {
                 self.ui.status("    passed");
                 Ok(Resolved::Outcome(Outcome::Passed))
             }
-            Judged::Confirm(mut text) => {
+            Judged::Confirm(confirm, mut text) => {
                 text.insert(0, "confirm the result:".to_owned());
                 let question = Question {
-                    kind: "confirm",
+                    kind: QuestionKind::Confirm {
+                        reasons: confirm.reasons,
+                        errored: confirm.errored,
+                    },
                     at: Some(pos),
                     text,
                     choices: vec![
@@ -466,7 +481,7 @@ impl Runner<'_> {
                         Choice::Shell,
                         Choice::Quit,
                     ],
-                    typed: false,
+                    typed: None,
                 };
                 Ok(match self.ask(question)? {
                     Answer::Choice(Choice::Yes) => Resolved::Outcome(Outcome::Confirmed),
@@ -475,9 +490,15 @@ impl Runner<'_> {
                     _ => Resolved::Stop("quit".to_owned()),
                 })
             }
-            Judged::Mismatch { lines, dead } => {
+            Judged::Mismatch {
+                mismatches,
+                aligned,
+                commit,
+                dead,
+            } => {
                 let mut text = vec![format!("MISMATCH ({:?}):", ran.status)];
-                text.extend(lines.into_iter().map(|l| format!("  {l}")));
+                let lines = mismatches.iter().map(mismatch_line).chain(commit.clone());
+                text.extend(lines.map(|l| format!("  {l}")));
                 let mut choices = if dead {
                     text.push(format!(
                         "the {} session is gone; retry starts a new one",
@@ -489,11 +510,18 @@ impl Runner<'_> {
                 };
                 choices.extend([Choice::View, Choice::Diff, Choice::Shell, Choice::Quit]);
                 let question = Question {
-                    kind: "mismatch",
+                    kind: QuestionKind::Mismatch {
+                        status: ran.status.clone(),
+                        mismatches,
+                        output: ran.lines.clone(),
+                        aligned,
+                        commit,
+                        dead,
+                    },
                     at: Some(pos),
                     text,
                     choices,
-                    typed: false,
+                    typed: None,
                 };
                 match self.ask(question)? {
                     Answer::Choice(Choice::Accept) => Ok(Resolved::Outcome(Outcome::Accepted)),
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t6-runner-script-rs.patch"
Remove-Item "$env:TEMP\p6b-t6-runner-script-rs.patch"
```

File `crates/planrun-exec/src/runner/task.rs` (exact diff; save the block as `$env:TEMP\p6b-t6-runner-task-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/runner/task.rs b/crates/planrun-exec/src/runner/task.rs
index c9cae89..eb33bb2 100644
--- a/crates/planrun-exec/src/runner/task.rs
+++ b/crates/planrun-exec/src/runner/task.rs
@@ -7,11 +7,12 @@ use super::gate::Gate;
 use super::resume::Past;
 use super::{Answer, Flow, Result, RunError, Runner, Typed, describe, is_file_action, task_paths};
 use crate::finding::{Finding, Severity};
+use crate::git::StatusEntry;
 use crate::journal::{Outcome, PathBlob, Record};
 use crate::policy::Decision;
 use crate::preflight;
 use crate::transaction::{self, CommitCause};
-use crate::ui::{Choice, Position, Question};
+use crate::ui::{Choice, Position, Question, QuestionKind, YES_OR_QUIT};
 
 /// The state of the task being run.
 pub(crate) struct TaskRun {
@@ -110,27 +111,27 @@ impl Runner<'_> {
     /// made and that a remaining file action writes over. They may be the user's, so they are
     /// asked about once, whatever `--auto` says.
     fn resume_dirt(&mut self, found: &[Finding]) -> Result<Flow> {
-        let paths: Vec<String> = found
+        let paths: Vec<RepoPath> = found
             .iter()
             .filter(|f| f.code == "resume-dirt")
-            .filter_map(|f| f.path.as_ref())
-            .map(|p| format!("  {p}"))
+            .filter_map(|f| f.path.clone())
             .collect();
         if paths.is_empty() {
             return Ok(Flow::Go);
         }
         let mut text = vec![format!(
-            "{} path(s) of the task have changes that the interrupted run did not verify as its              own, and a remaining file action writes over them:",
+            "{} path(s) of the task have changes that the interrupted run did not verify as its \
+             own, and a remaining file action writes over them:",
             paths.len()
         )];
-        text.extend(paths);
+        text.extend(paths.iter().map(|p| format!("  {p}")));
         text.push("continue writes over them; quit leaves them as they are".to_owned());
         let question = Question {
-            kind: "resume-dirt",
+            kind: QuestionKind::ResumeDirt { paths },
             at: None,
             text,
             choices: vec![Choice::Continue, Choice::Diff, Choice::Shell, Choice::Quit],
-            typed: false,
+            typed: None,
         };
         Ok(match self.ask(question)? {
             Answer::Choice(Choice::Continue) => Flow::Go,
@@ -196,7 +197,7 @@ impl Runner<'_> {
                 continue;
             }
             let question = Question {
-                kind: "unscoped",
+                kind: QuestionKind::Unscoped,
                 at: Some(here),
                 text: vec![format!(
                     "L{} {} ({:?}): not in the task's files (D50)",
@@ -212,7 +213,7 @@ impl Runner<'_> {
                     Choice::Shell,
                     Choice::Quit,
                 ],
-                typed: false,
+                typed: None,
             };
             match self.ask(question)? {
                 Answer::Choice(Choice::Run) => chosen.push(action),
@@ -265,7 +266,9 @@ impl Runner<'_> {
                 CommitCause::Changed(paths) => {
                     let names: Vec<&str> = paths.iter().map(RepoPath::as_str).collect();
                     let question = Question {
-                        kind: "changed",
+                        kind: QuestionKind::Changed {
+                            paths: paths.clone(),
+                        },
                         at: Some(pos),
                         text: vec![
                             format!("changed since the step was prepared: {}", names.join(", ")),
@@ -277,7 +280,7 @@ impl Runner<'_> {
                             Choice::Shell,
                             Choice::Quit,
                         ],
-                        typed: false,
+                        typed: None,
                     };
                     match self.ask(question)? {
                         Answer::Choice(Choice::PrepareAgain) => continue,
@@ -331,7 +334,7 @@ impl Runner<'_> {
         )];
         lines.extend(self.plan.text(text).lines().map(|l| format!("  {l}")));
         let question = Question {
-            kind: "manual",
+            kind: QuestionKind::Manual,
             at: Some(pos),
             text: lines,
             choices: vec![
@@ -341,7 +344,7 @@ impl Runner<'_> {
                 Choice::Shell,
                 Choice::Quit,
             ],
-            typed: false,
+            typed: None,
         };
         match self.ask(question)? {
             Answer::Choice(Choice::Continue) => {}
@@ -445,13 +448,15 @@ impl Runner<'_> {
         text.extend(reasons.iter().map(|r| format!("  {r}")));
         text.push("type `yes` to commit anyway, or `quit`".to_owned());
         let question = Question {
-            kind: "gate",
+            kind: QuestionKind::Gate {
+                reasons: reasons.clone(),
+            },
             at: Some(pos),
             text,
             choices: Vec::new(),
-            typed: true,
+            typed: Some(YES_OR_QUIT),
         };
-        match self.typed_yes(question, false)? {
+        match self.typed_yes(question)? {
             Typed::Yes => {
                 self.append(Record::GateOverride {
                     task: task.id.raw.to_string(),
@@ -491,13 +496,12 @@ impl Runner<'_> {
     /// Dirt outside the task's paths before a commit (D80): listed and asked about.
     fn outside_dirt(&mut self, pos: Position) -> Result<Flow> {
         let task = &self.plan.tasks[pos.task];
-        let outside: Vec<String> = self
+        let outside: Vec<StatusEntry> = self
             .ws
             .git()
             .status(None)?
             .into_iter()
             .filter(|e| !self.in_task(task, &e.path))
-            .map(|e| format!("  {} {}", e.code, e.path))
             .collect();
         if outside.is_empty() {
             return Ok(Flow::Go);
@@ -507,13 +511,13 @@ impl Runner<'_> {
              would take them:",
             outside.len()
         )];
-        text.extend(outside);
+        text.extend(outside.iter().map(|e| format!("  {} {}", e.code, e.path)));
         let question = Question {
-            kind: "dirt",
+            kind: QuestionKind::Dirt { paths: outside },
             at: Some(pos),
             text,
             choices: vec![Choice::Continue, Choice::Diff, Choice::Shell, Choice::Quit],
-            typed: false,
+            typed: None,
         };
         Ok(match self.ask(question)? {
             Answer::Choice(Choice::Continue) => Flow::Go,
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t6-runner-task-rs.patch"
Remove-Item "$env:TEMP\p6b-t6-runner-task-rs.patch"
```

File `crates/planrun-exec/src/ui.rs` (full contents):

```rust
//! The interactive surface the runner talks to (decisions D15, D74, D86). planrun-cli
//! implements it with line prompts on std; the tests with scripted answers. The runner decides
//! what is asked and what an answer does; the UI shows, reads, and handles `[v]`, `[d]` and
//! `[o]` itself.

use std::path::PathBuf;

use planrun_model::{RepoPath, Shell};

use crate::done::Seen;
use crate::expect::{Aligned, Mismatch, Reason};
use crate::git::{Head, StatusEntry};
use crate::journal::At;
use crate::policy::Decision;
use crate::shell::{OutLine, RunStatus};

/// The words a Destructive script's typed question accepts (D74).
pub const RUN_SKIP_OR_QUIT: &[&str] = &["yes", "skip", "quit"];

/// The words the commit gate's and an accept-as-done typed question accept (D75, D77).
pub const YES_OR_QUIT: &[&str] = &["yes", "quit"];

/// An action's place in the plan, for the UI to show it: positions from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// The task's position in the plan.
    pub task: usize,
    /// The step's position in the task.
    pub step: usize,
    /// The action's position in the step.
    pub action: usize,
}

/// One answer a question offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// `[enter]` run it.
    Run,
    /// `[s]` skip it.
    Skip,
    /// `[v]` view the action in full.
    View,
    /// `[d]` the working tree's diff against HEAD.
    Diff,
    /// `[o]` an interactive shell at the repository root.
    Shell,
    /// `[q]` stop the run.
    Quit,
    /// `[y]` yes.
    Yes,
    /// `[n]` no.
    No,
    /// `[a]` accept the mismatch and go on.
    Accept,
    /// `[r]` run it again.
    Retry,
    /// `[enter]` go on.
    Continue,
    /// `[k]` keep the earlier run's result (resume).
    Keep,
    /// `[r]` restore the step from its pre-images (resume).
    Restore,
    /// `[p]` prepare the step again.
    PrepareAgain,
}

impl Choice {
    /// The key a line prompt reads for it; `""` is a bare enter.
    pub fn key(self) -> &'static str {
        match self {
            Choice::Run | Choice::Continue => "",
            Choice::Skip => "s",
            Choice::View => "v",
            Choice::Diff => "d",
            Choice::Shell => "o",
            Choice::Quit => "q",
            Choice::Yes => "y",
            Choice::No => "n",
            Choice::Accept => "a",
            Choice::Retry | Choice::Restore => "r",
            Choice::Keep => "k",
            Choice::PrepareAgain => "p",
        }
    }

    /// Its label (`run`, `skip`, …).
    pub fn label(self) -> &'static str {
        match self {
            Choice::Run => "run",
            Choice::Skip => "skip",
            Choice::View => "view action",
            Choice::Diff => "diff so far",
            Choice::Shell => "open shell",
            Choice::Quit => "quit",
            Choice::Yes => "yes",
            Choice::No => "no",
            Choice::Accept => "accept",
            Choice::Retry => "retry",
            Choice::Continue => "continue",
            Choice::Keep => "keep",
            Choice::Restore => "restore and redo",
            Choice::PrepareAgain => "prepare again",
        }
    }
}

/// Which question it is, with what the run knows about it that the plan does not say (D86).
/// What the plan says (the action, its text, its path) is reached through [`Question::at`];
/// resume's questions, which have no `at`, name their step or action by its journal place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionKind {
    /// `run`: a script whose decision asks (D69).
    Run {
        /// The decision: `Ask` with the class that made it ask.
        decision: Decision,
    },
    /// `unscoped`: a file action outside the task's files (D50).
    Unscoped,
    /// `typed-yes`: a Destructive script (D69, D74).
    TypedYes {
        /// What made it Destructive, from the classifier.
        reasons: Vec<String>,
    },
    /// `confirm`: a result the plan states in prose or comments, or an exit it does not state
    /// (D11, D66).
    Confirm {
        /// Why it asks, in the matcher's order.
        reasons: Vec<Reason>,
        /// pwsh recorded an error.
        errored: bool,
    },
    /// `mismatch`: what was checked does not hold (D66, D75, D79, D81).
    Mismatch {
        /// How the block ended.
        status: RunStatus,
        /// What the matcher found.
        mismatches: Vec<Mismatch>,
        /// The block's output, both streams, as the matcher saw it.
        output: Vec<OutLine>,
        /// The expected summary lines against `output`, whose lines [`Aligned::output`]
        /// indexes (empty when the plan states none).
        aligned: Vec<Aligned>,
        /// For a commit, why it does not count as made (D79).
        commit: Option<String>,
        /// The session ended, timed out or broke: no accept (D75).
        dead: bool,
    },
    /// `manual`: a Manual waits until acknowledged (D21).
    Manual,
    /// `dirt`: changes outside the task's paths before a commit (D80).
    Dirt {
        /// The changed paths, as `git status` reports them.
        paths: Vec<StatusEntry>,
    },
    /// `outside`: the session's working directory is outside the repository (D81).
    Outside {
        /// The session's shell.
        shell: Shell,
        /// Its working directory.
        cwd: PathBuf,
    },
    /// `session`: the notice that a session is new (D81).
    Session {
        /// Why the session is new.
        why: String,
        /// The earlier tasks whose scripts ran in another session.
        earlier: Vec<String>,
    },
    /// `gate`: the commit gate is closed (D75).
    Gate {
        /// Why.
        reasons: Vec<String>,
    },
    /// `changed`: the disk changed between a step's prepare and its commit (D78).
    Changed {
        /// The paths that changed.
        paths: Vec<RepoPath>,
    },
    /// `again`: resume reached a script that ran before the interruption (D77).
    Again {
        /// It was running when planrun was interrupted, not ended.
        running: bool,
    },
    /// `unknown`: resume found a step neither as before nor as planned (D77).
    Unknown {
        /// The step, as the journal names it (its first action).
        step: At,
        /// Each path of the step as it is now.
        paths: Vec<(RepoPath, Seen)>,
        /// Every pre-image is there, so `[r]` can restore it (D57).
        restorable: bool,
    },
    /// `accept-done`: accepting an unknown step as done (D77).
    AcceptDone {
        /// The step, as the journal names it.
        step: At,
    },
    /// `head-moved`: HEAD moved by one commit while an action was running (D77).
    HeadMoved {
        /// The action that was running, as the journal names it.
        action: At,
        /// HEAD before the action.
        recorded: Head,
        /// HEAD now.
        now: Head,
    },
    /// `resume-dirt`: changes a remaining file action writes over, which may be the user's
    /// (D77).
    ResumeDirt {
        /// The paths.
        paths: Vec<RepoPath>,
    },
}

impl QuestionKind {
    /// Its journal name (`run`, `typed-yes`, …): the journal format does not change.
    pub fn name(&self) -> &'static str {
        match self {
            QuestionKind::Run { .. } => "run",
            QuestionKind::Unscoped => "unscoped",
            QuestionKind::TypedYes { .. } => "typed-yes",
            QuestionKind::Confirm { .. } => "confirm",
            QuestionKind::Mismatch { .. } => "mismatch",
            QuestionKind::Manual => "manual",
            QuestionKind::Dirt { .. } => "dirt",
            QuestionKind::Outside { .. } => "outside",
            QuestionKind::Session { .. } => "session",
            QuestionKind::Gate { .. } => "gate",
            QuestionKind::Changed { .. } => "changed",
            QuestionKind::Again { .. } => "again",
            QuestionKind::Unknown { .. } => "unknown",
            QuestionKind::AcceptDone { .. } => "accept-done",
            QuestionKind::HeadMoved { .. } => "head-moved",
            QuestionKind::ResumeDirt { .. } => "resume-dirt",
        }
    }
}

/// What the runner asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// Which question, with its detail.
    pub kind: QuestionKind,
    /// The action it is about, when there is one (for `[v]`).
    pub at: Option<Position>,
    /// What to show above the prompt, one line per entry.
    pub text: Vec<String>,
    /// The answers offered, in order. Empty with `typed`.
    pub choices: Vec<Choice>,
    /// For a typed question, the words it accepts, each exactly as written (D74, D86); `None`
    /// for a question of choices. The runner acts on these words and no other.
    pub typed: Option<&'static [&'static str]>,
}

/// What the UI read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// One of the question's choices.
    Choice(Choice),
    /// For a typed question: the line without its line ending, nothing else removed.
    Typed(String),
    /// No answer can be had (stdin is not a terminal, or it closed): the run stops.
    Stop(String),
}

/// The interactive surface (D15, D74).
pub trait Ui {
    /// Shows a line of progress.
    fn status(&mut self, line: &str);
    /// Shows one line of a running block's output, as it arrives.
    fn output(&mut self, line: &OutLine);
    /// Asks `question`; `[v]`, `[d]` and `[o]` come back as choices for the runner to pass to
    /// [`Ui::view`], [`Ui::diff`] and [`Ui::shell`].
    fn ask(&mut self, question: &Question) -> Reply;
    /// Shows the action at `at` in full.
    fn view(&mut self, at: Position);
    /// Shows `git --no-optional-locks diff HEAD` and the untracked files.
    fn diff(&mut self);
    /// Runs an interactive shell at the repository root until it exits; Ctrl-C reaches the
    /// shell, not planrun (D73).
    fn shell(&mut self);
}
```

File `crates/planrun-exec/tests/resume.rs` (exact diff; save the block as `$env:TEMP\p6b-t6-tests-resume-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/tests/resume.rs b/crates/planrun-exec/tests/resume.rs
index 021a60e..22476fe 100644
--- a/crates/planrun-exec/tests/resume.rs
+++ b/crates/planrun-exec/tests/resume.rs
@@ -18,7 +18,7 @@ use planrun_exec::policy::{Policy, PolicyClass};
 use planrun_exec::runner::resume::analyse;
 use planrun_exec::runner::{Finished, RunConfig, Runner};
 use planrun_exec::shell::{OutLine, SessionConfig};
-use planrun_exec::ui::{Choice, Position, Question, Reply, Ui};
+use planrun_exec::ui::{Choice, Position, Question, QuestionKind, Reply, Ui};
 use planrun_exec::workspace::Workspace;
 use planrun_model::{
     Action, ActionKind, ChangeKind, ExitExpect, Expectation, PathOrigin, Plan, ScriptIntent,
@@ -29,6 +29,7 @@ use planrun_model::{
 struct Scripted {
     answers: VecDeque<Reply>,
     asked: Vec<String>,
+    kinds: Vec<QuestionKind>,
     shown: Vec<String>,
 }
 
@@ -40,11 +41,12 @@ impl Ui for Scripted {
         self.shown.push(format!("| {}", line.text));
     }
     fn ask(&mut self, question: &Question) -> Reply {
-        self.asked.push(question.kind.to_owned());
+        self.asked.push(question.kind.name().to_owned());
+        self.kinds.push(question.kind.clone());
         self.shown.extend(question.text.iter().cloned());
         self.answers
             .pop_front()
-            .unwrap_or_else(|| Reply::Stop(format!("no answer for `{}`", question.kind)))
+            .unwrap_or_else(|| Reply::Stop(format!("no answer for `{}`", question.kind.name())))
     }
     fn view(&mut self, _at: Position) {}
     fn diff(&mut self) {}
@@ -279,6 +281,13 @@ fn a_crash_between_two_writes_is_unknown_and_restore_completes_it() {
         ui.shown
     );
     assert_eq!(ui.asked, ["unknown", "session"]);
+    let QuestionKind::Unknown { step, .. } = &ui.kinds[0] else {
+        panic!("{:?}", ui.kinds);
+    };
+    assert_eq!(
+        (step.task.as_str(), step.step, step.action),
+        ("P1-T1", 0, 0)
+    );
     let all = records(&f);
     assert_eq!(count(&all, "step-restored"), 1);
     assert_done(&f);
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t6-tests-resume-rs.patch"
Remove-Item "$env:TEMP\p6b-t6-tests-resume-rs.patch"
```

File `crates/planrun-exec/tests/runner.rs` (exact diff; save the block as `$env:TEMP\p6b-t6-tests-runner-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/tests/runner.rs b/crates/planrun-exec/tests/runner.rs
index 84f382c..6056d87 100644
--- a/crates/planrun-exec/tests/runner.rs
+++ b/crates/planrun-exec/tests/runner.rs
@@ -15,8 +15,10 @@ use ir::{Ir, PATCH_T1, PATCH_T2};
 use planrun_exec::journal::{self, End, Journal, Outcome, Record};
 use planrun_exec::policy::{Policy, PolicyClass};
 use planrun_exec::runner::{Finished, RunConfig, Runner};
-use planrun_exec::shell::{OutLine, SessionConfig};
-use planrun_exec::ui::{Choice, Position, Question, Reply, Ui};
+use planrun_exec::shell::{OutLine, RunStatus, SessionConfig};
+use planrun_exec::ui::{
+    Choice, Position, Question, QuestionKind, RUN_SKIP_OR_QUIT, Reply, Ui, YES_OR_QUIT,
+};
 use planrun_exec::workspace::Workspace;
 use planrun_model::{
     Action, ActionKind, ChangeKind, ExitExpect, Expectation, PathOrigin, Plan, ScriptIntent, Shell,
@@ -28,6 +30,8 @@ use planrun_model::{
 struct Scripted {
     answers: VecDeque<Reply>,
     asked: Vec<String>,
+    kinds: Vec<QuestionKind>,
+    words: Vec<Option<&'static [&'static str]>>,
     shown: Vec<String>,
 }
 
@@ -48,11 +52,13 @@ impl Ui for Scripted {
         self.shown.push(format!("| {}", line.text));
     }
     fn ask(&mut self, question: &Question) -> Reply {
-        self.asked.push(question.kind.to_owned());
+        self.asked.push(question.kind.name().to_owned());
+        self.kinds.push(question.kind.clone());
+        self.words.push(question.typed);
         self.shown.extend(question.text.iter().cloned());
         self.answers
             .pop_front()
-            .unwrap_or_else(|| Reply::Stop(format!("no answer for `{}`", question.kind)))
+            .unwrap_or_else(|| Reply::Stop(format!("no answer for `{}`", question.kind.name())))
     }
     fn view(&mut self, _at: Position) {}
     fn diff(&mut self) {}
@@ -361,6 +367,118 @@ fn a_commit_that_does_not_move_head_is_a_mismatch() {
     );
 }
 
+#[test]
+fn questions_carry_what_the_run_found() {
+    let f = Fixture::greeter(false);
+    let mut ir = Ir::default();
+    let lines = planrun_model::TextRef::Span(ir.text("ok: …\nmissing\n"));
+    let mut check = ir.script("'ok: 3'; 'done'");
+    if let ActionKind::RunScript(s) = &mut check.kind {
+        s.expect = Some(Expectation {
+            exit: ExitExpect::Success,
+            lines: Some(lines),
+            prose: None,
+            comments: None,
+        });
+    }
+    let plan = one(
+        &mut ir,
+        planrun_model::Manifest::default(),
+        vec![vec![check]],
+    );
+    let (_, ui, _) = run(&f, &plan, AUTO, vec![choice(Choice::Quit)]);
+    let [
+        QuestionKind::Mismatch {
+            status,
+            mismatches,
+            output,
+            aligned,
+            commit,
+            dead,
+        },
+    ] = &ui.kinds[..]
+    else {
+        panic!("{:?}", ui.kinds);
+    };
+    assert_eq!(*status, RunStatus::Exited(0));
+    assert_eq!(mismatches.len(), 1, "{mismatches:?}");
+    let shown: Vec<(usize, Option<&str>)> = aligned
+        .iter()
+        .map(|a| {
+            (
+                a.number,
+                a.output.map(|i| output[i].text.trim_end_matches('\r')),
+            )
+        })
+        .collect();
+    assert_eq!(shown, [(1, Some("ok: 3")), (2, None)]);
+    assert_eq!((commit, *dead), (&None, false));
+
+    let f = Fixture::greeter(false);
+    f.write("notes.txt", b"mine\n");
+    let mut ir = Ir::default();
+    let steps = vec![vec![ir.commit("'nothing committed'")]];
+    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
+    let answers = vec![choice(Choice::Continue), choice(Choice::Quit)];
+    let (_, ui, _) = run(&f, &plan, AUTO, answers);
+    let [
+        QuestionKind::Dirt { paths },
+        QuestionKind::Mismatch { commit, .. },
+    ] = &ui.kinds[..]
+    else {
+        panic!("{:?}", ui.kinds);
+    };
+    let dirty: Vec<(&str, &str)> = paths
+        .iter()
+        .map(|e| (e.code.as_str(), e.path.as_str()))
+        .collect();
+    assert_eq!(dirty, [("??", "notes.txt")]);
+    assert!(
+        commit
+            .as_deref()
+            .is_some_and(|c| c.contains("HEAD did not move")),
+        "{commit:?}"
+    );
+}
+
+#[test]
+fn a_typed_question_lists_the_words_it_accepts() {
+    let f = Fixture::greeter(false);
+    let mut ir = Ir::default();
+    let failing = ir.script("git rev-parse -q --verify refs/heads/none");
+    let failing = expecting(
+        failing,
+        ExitExpect::Success,
+        ScriptIntent::Gate,
+        false,
+        &mut ir,
+    );
+    let steps = vec![
+        vec![ir.script("git push --dry-run")],
+        vec![failing],
+        vec![ir.commit("git commit -q --allow-empty -m 'T1'")],
+    ];
+    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
+    let answers = vec![
+        typed("skip"),
+        choice(Choice::Accept),
+        typed("skip"),
+        typed("quit"),
+    ];
+    let (finished, ui, _) = run(&f, &plan, AUTO, answers);
+    assert_eq!(finished.end, End::Stopped, "{:?}", ui.shown);
+    assert_eq!(ui.asked, ["typed-yes", "mismatch", "gate", "gate"]);
+    assert_eq!(
+        ui.words,
+        [
+            Some(RUN_SKIP_OR_QUIT),
+            None,
+            Some(YES_OR_QUIT),
+            Some(YES_OR_QUIT)
+        ]
+    );
+}
+
 #[test]
 fn a_dead_session_offers_retry_and_the_new_session_is_announced() {
     let f = Fixture::greeter(false);
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t6-tests-runner-rs.patch"
Remove-Item "$env:TEMP\p6b-t6-tests-runner-rs.patch"
```

**Step 2 — verify**

Targeted: the runner and resume tests, with new ones on the payloads and the typed words:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-exec --test runner --test resume
```

Expected (exit code 0; summary lines):

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-cli/src/ui.rs crates/planrun-exec/src/preflight.rs crates/planrun-exec/src/runner/mod.rs crates/planrun-exec/src/runner/resume.rs crates/planrun-exec/src/runner/script.rs crates/planrun-exec/src/runner/task.rs crates/planrun-exec/src/ui.rs crates/planrun-exec/tests/resume.rs crates/planrun-exec/tests/runner.rs
  $verified = [ordered]@{
      'crates/planrun-cli/src/ui.rs' = '1868d70714288cdd9f5b1ee86b12b3fdf7fb2ade'
      'crates/planrun-exec/src/preflight.rs' = '436d8a351aedb22c7c5e30e6015014b31e9a2414'
      'crates/planrun-exec/src/runner/mod.rs' = '646fda7f74dc2553aa3f655c5a16fe0831033480'
      'crates/planrun-exec/src/runner/resume.rs' = '16ccb8bddc7bc9b4b28608357563220d17cfc8c9'
      'crates/planrun-exec/src/runner/script.rs' = '7225c5b149f4535eeb9d9b5e4e6edccea580c0e0'
      'crates/planrun-exec/src/runner/task.rs' = 'eb33bb23f8cf9ba00005e904aaba063aecfc2f15'
      'crates/planrun-exec/src/ui.rs' = 'c6201657f4b238c86ffc0289793a42d5b4f26a07'
      'crates/planrun-exec/tests/resume.rs' = '22476fe5903d521b3b55b6c82f9902bba0bcaab3'
      'crates/planrun-exec/tests/runner.rs' = '6056d8761ee244f0bb6d99d9dffe21dd071a18f7'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6b-T6: QuestionKind" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P6b-T7: the liveness lock

**Goal:** A run holds `<git-dir>/planrun/lock.live` besides `lock`, taken with a blocking lock once `lock` is held, and `lock::live` sees a run in progress with one try on `lock.live`, let go at once, creating nothing (D90, review ruling 1). `lock::holder` reads `lock.pid` for messages only.

**Files**

- Modify `crates/planrun-exec/src/lock.rs`

**Step 1 — write the files**

File `crates/planrun-exec/src/lock.rs` (full contents):

```rust
//! One planrun per repository (decisions D77, D90): `<git-dir>/planrun/lock`, held with
//! `File::try_lock` for the whole run, prompts included, by every plan's run and resume. The
//! operating system releases it when the process ends, however it ends, so there is no stale
//! lock. The holder's process id goes into `lock.pid` for the message only: on Windows a locked
//! file cannot be read by another process, and the file stays when its run ends.
//!
//! A run also holds `lock.live`, taken with a blocking `File::lock` once `lock` is held, so that
//! [`live`] can see a run in progress without trying `lock`: a try there would make a starting
//! run fail, while a probe that holds `lock.live` for a moment only makes it wait.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::Path;

/// The held lock; dropping it releases it.
#[derive(Debug)]
pub struct RunLock {
    _file: File,
    _live: File,
}

/// Why the lock was not taken.
#[derive(Debug, thiserror::Error)]
pub enum LockError {
    /// Another planrun holds it.
    #[error("another planrun is running in this repository{}", pid_note(.pid))]
    Held {
        /// The holder's process id, as it wrote it.
        pid: Option<u32>,
    },
    /// The lock file could not be made or locked.
    #[error("the run lock: {0}")]
    Io(#[from] io::Error),
}

fn pid_note(pid: &Option<u32>) -> String {
    pid.map(|p| format!(" (process {p})")).unwrap_or_default()
}

fn open(dir: &Path, name: &str) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(name))
}

/// Takes the lock in `dir` (`<git-dir>/planrun`), creating the directory, or says who holds it.
/// Once `lock` is held, `lock.live` is taken too, waiting while a [`live`] probe holds it.
pub fn acquire(dir: &Path) -> Result<RunLock, LockError> {
    fs::create_dir_all(dir)?;
    let file = open(dir, "lock")?;
    match file.try_lock() {
        Ok(()) => {
            let live = open(dir, "lock.live")?;
            live.lock()?;
            fs::write(dir.join("lock.pid"), std::process::id().to_string())?;
            Ok(RunLock {
                _file: file,
                _live: live,
            })
        }
        Err(TryLockError::WouldBlock) => Err(LockError::Held { pid: holder(dir) }),
        Err(TryLockError::Error(e)) => Err(LockError::Io(e)),
    }
}

/// Whether a run holds the lock in `dir` now (D90): one try on `lock.live`, let go at once.
/// Creates and writes nothing; `false` when there is no `lock.live` yet. A run that is between
/// taking `lock` and `lock.live` is not seen.
pub fn live(dir: &Path) -> io::Result<bool> {
    let file = match OpenOptions::new().read(true).open(dir.join("lock.live")) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    match file.try_lock() {
        Ok(()) => Ok(false),
        Err(TryLockError::WouldBlock) => Ok(true),
        Err(TryLockError::Error(e)) => Err(e),
    }
}

/// The process id in `lock.pid`, for a message: it may name a run that has ended.
pub fn holder(dir: &Path) -> Option<u32> {
    fs::read_to_string(dir.join("lock.pid"))
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    /// A fresh directory under the OS temp directory, not made yet.
    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("planrun-lock-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_second_holder_is_refused_until_the_first_lets_go() {
        let dir = temp("second");
        let first = acquire(&dir).unwrap_or_else(|e| panic!("{e}"));
        match acquire(&dir) {
            Err(LockError::Held { pid }) => assert_eq!(pid, Some(std::process::id())),
            other => panic!("{other:?}"),
        }
        drop(first);
        let again = acquire(&dir).unwrap_or_else(|e| panic!("{e}"));
        drop(again);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn a_run_is_live_while_it_holds_the_lock_and_the_probe_writes_nothing() {
        let dir = temp("live");
        assert!(!live(&dir).unwrap_or_else(|e| panic!("{e}")));
        assert!(!dir.exists(), "the probe created the directory");
        let run = acquire(&dir).unwrap_or_else(|e| panic!("{e}"));
        assert!(live(&dir).unwrap_or_else(|e| panic!("{e}")));
        drop(run);
        assert!(!live(&dir).unwrap_or_else(|e| panic!("{e}")));
        assert_eq!(
            holder(&dir),
            Some(std::process::id()),
            "lock.pid stays, shown only"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn a_run_that_starts_while_a_probe_holds_lock_live_waits_and_succeeds() {
        let dir = temp("wait");
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
        let probe = open(&dir, "lock.live").unwrap_or_else(|e| panic!("{e}"));
        probe.try_lock().unwrap_or_else(|e| panic!("{e}"));
        let held = Duration::from_millis(300);
        let release = std::thread::spawn(move || {
            std::thread::sleep(held);
            drop(probe);
        });
        let start = Instant::now();
        let run = acquire(&dir).unwrap_or_else(|e| panic!("{e}"));
        assert!(
            start.elapsed() >= held / 2,
            "acquire did not wait for the probe"
        );
        release
            .join()
            .unwrap_or_else(|_| panic!("the probe thread panicked"));
        drop(run);
        let _ = fs::remove_dir_all(dir);
    }
}
```

**Step 2 — verify**

Targeted: the lock's unit tests:

```powershell
cargo test -p planrun-exec --lib lock::tests
```

Expected (exit code 0; summary lines):

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 52 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-exec/src/lock.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/lock.rs' = '86f8827fec45096354d37fddaff4b7d8a73f96fb'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6b-T7: the liveness lock" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P6b-T8: the resume preview

**Goal:** `runner::preview::preview` reads what resume would find (whether a run is live, the plan's fingerprint against the journal's, `.git/index.lock`, resume's verdict on HEAD, each prepared step of the interrupted task on disk with whether it can be restored and its temp files, the scripts that will ask again, the `resume-dirt` paths) without the run lock and without writing anything, for the TUI's resume preview (D85, review ruling 4). Resume's judgements become functions both call: `head_verdict`, `again`, `resumed_for` and `commit_problem`. `done::temps` is the read-only half of `remove_temps`.

**Files**

- Modify `crates/planrun-exec/src/done.rs`
- Modify `crates/planrun-exec/src/done_tests.rs`
- Modify `crates/planrun-exec/src/runner/mod.rs`
- Create `crates/planrun-exec/src/runner/preview.rs`
- Modify `crates/planrun-exec/src/runner/resume.rs`
- Modify `crates/planrun-exec/src/runner/script.rs`
- Modify `crates/planrun-exec/src/runner/task.rs`
- Modify `crates/planrun-exec/tests/resume.rs`

Over 300 lines, so they come as exact diffs: `crates/planrun-exec/src/runner/mod.rs`, `crates/planrun-exec/src/runner/resume.rs`, `crates/planrun-exec/src/runner/script.rs`, `crates/planrun-exec/src/runner/task.rs`, `crates/planrun-exec/tests/resume.rs`.

**Step 1 — write the files**

File `crates/planrun-exec/src/done.rs` (full contents):

```rust
//! A journaled step's state on the disk, for resume (decisions D14, D77, D78): whether its
//! writes are all there (Done), none of them are (NotDone), or anything else (Unknown, the user
//! decides); whether its pre-images survive to restore it; and the writer's temp files it may
//! have left behind.

use std::fs;
use std::path::PathBuf;

use planrun_model::RepoPath;

use crate::DoneState;
use crate::git::BlobId;
use crate::transaction::{PreImage, StepRecord, WriteRecord, restore};
use crate::workspace::{Workspace, WorkspaceError};

/// What one path of a step looks like now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seen {
    /// As the step leaves it (bytes, mode under `core.filemode`, and for a `via_git` delete
    /// the index entry gone).
    Post,
    /// As it was before the step.
    Pre,
    /// Neither.
    Neither,
}

/// A step's paths as they are now, and what that makes the step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepOnDisk {
    /// Each planned path, in the step's order.
    pub paths: Vec<(RepoPath, Seen)>,
    /// Done when every path is as the step leaves it, NotDone when every path is as it was,
    /// Unknown otherwise. A path whose pre- and post-image are the same counts for both.
    pub state: DoneState,
}

/// One path's disk state.
struct Now {
    present: bool,
    raw: Option<BlobId>,
    executable: bool,
    in_index: Option<bool>,
}

/// Compares the disk with `step`: one stat per path, one `hash-object --stdin-paths` over the
/// present ones, one `ls-files` per index removal.
pub fn step_state(ws: &Workspace, step: &StepRecord) -> Result<StepOnDisk, WorkspaceError> {
    let mut nows = Vec::new();
    for record in &step.writes {
        let resolved = ws.resolve(&record.path)?;
        let present = resolved.exists() && !resolved.is_dir;
        let in_index = if step.index_removals.contains(&record.path) {
            Some(ws.git().index_entry(&record.path, &[])?.is_some())
        } else {
            None
        };
        nows.push(Now {
            present,
            raw: None,
            executable: present && Workspace::is_executable(&resolved.full),
            in_index,
        });
    }
    let present: Vec<&RepoPath> = step
        .writes
        .iter()
        .zip(&nows)
        .filter(|(_, now)| now.present)
        .map(|(r, _)| &r.path)
        .collect();
    let mut ids = ws.git().hash_paths(&present, false)?.into_iter();
    for now in nows.iter_mut().filter(|n| n.present) {
        now.raw = ids.next();
    }
    let mut paths = Vec::new();
    let (mut all_post, mut all_pre) = (true, true);
    for (record, now) in step.writes.iter().zip(&nows) {
        let post = is_post(ws, record, now);
        let pre = is_pre(ws, record, now);
        all_post &= post;
        all_pre &= pre;
        let seen = match (post, pre) {
            (true, _) => Seen::Post,
            (false, true) => Seen::Pre,
            (false, false) => Seen::Neither,
        };
        paths.push((record.path.clone(), seen));
    }
    let state = match (all_post, all_pre) {
        (true, _) => DoneState::Done,
        (false, true) => DoneState::NotDone,
        (false, false) => DoneState::Unknown,
    };
    Ok(StepOnDisk { paths, state })
}

fn mode_holds(ws: &Workspace, mode: Option<&str>, now: &Now) -> bool {
    !ws.filemode() || mode.is_none_or(|m| (m == "100755") == now.executable)
}

fn is_post(ws: &Workspace, record: &WriteRecord, now: &Now) -> bool {
    match record.post {
        Some(id) => now.raw == Some(id) && mode_holds(ws, record.mode.as_deref(), now),
        None => !now.present && now.in_index != Some(true),
    }
}

fn is_pre(ws: &Workspace, record: &WriteRecord, now: &Now) -> bool {
    match &record.pre {
        PreImage::Absent => !now.present,
        PreImage::Stored(id) => {
            now.raw == Some(*id)
                && mode_holds(ws, record.pre_mode.as_deref(), now)
                && now.in_index != Some(false)
        }
    }
}

/// The pre-image objects of `step` that the object database no longer has (D57).
pub fn missing_pre_images(
    ws: &Workspace,
    step: &StepRecord,
) -> Result<Vec<BlobId>, WorkspaceError> {
    let ids: Vec<BlobId> = step
        .writes
        .iter()
        .filter_map(|w| match w.pre {
            PreImage::Stored(id) => Some(id),
            PreImage::Absent => None,
        })
        .collect();
    Ok(ws.git().missing_objects(&ids)?)
}

/// Whether [`restore_step`] can put `step` back as it was: every pre-image object is there,
/// and no index removal has happened yet (its entry would be lost).
pub fn restorable(ws: &Workspace, step: &StepRecord) -> Result<bool, WorkspaceError> {
    if !missing_pre_images(ws, step)?.is_empty() {
        return Ok(false);
    }
    for path in &step.index_removals {
        if ws.git().index_entry(path, &[])?.is_none() {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Puts every path of `step` back as it was before it (resume's `[r]`, D77): pre-image bytes
/// and mode, created files removed, created directories removed while empty. Returns the paths
/// it could not restore.
pub fn restore_step(ws: &Workspace, step: &StepRecord) -> Vec<RepoPath> {
    let mut unrestored = Vec::new();
    for record in step.writes.iter().rev() {
        if restore(ws, record).is_err() {
            unrestored.push(record.path.clone());
        }
        for dir in record.creates_dirs.iter().rev() {
            let _ = fs::remove_dir(ws.root().join(dir));
        }
    }
    unrestored
}

/// Removes the writer's temp files `step` names, where they exist (a crash between the temp
/// write and the rename, D77). Nothing is searched for by pattern. Returns what it removed, as
/// repository-relative paths.
pub fn remove_temps(ws: &Workspace, step: &StepRecord) -> Vec<String> {
    temp_files(ws, step)
        .into_iter()
        .filter(|(_, full)| fs::remove_file(full).is_ok())
        .map(|(rel, _)| rel)
        .collect()
}

/// The writer's temp files `step` names that exist, as repository-relative paths: what
/// [`remove_temps`] would remove. Reads only.
pub fn temps(ws: &Workspace, step: &StepRecord) -> Vec<String> {
    temp_files(ws, step)
        .into_iter()
        .map(|(rel, _)| rel)
        .collect()
}

/// The temp files of `step` that exist: the repository-relative path and the full one.
fn temp_files(ws: &Workspace, step: &StepRecord) -> Vec<(String, PathBuf)> {
    let mut found = Vec::new();
    for record in &step.writes {
        let rel = match record.path.dir() {
            Some(dir) => format!("{dir}/{}", record.temp),
            None => record.temp.clone(),
        };
        let Ok(path) = RepoPath::new(&rel) else {
            continue;
        };
        let Ok(resolved) = ws.resolve(&path) else {
            continue;
        };
        if fs::symlink_metadata(&resolved.full).is_ok_and(|m| m.is_file()) {
            found.push((rel, resolved.full));
        }
    }
    found
}

#[cfg(test)]
#[path = "done_tests.rs"]
mod tests;
```

File `crates/planrun-exec/src/done_tests.rs` (full contents):

```rust
//! A step's state on the disk on a fixture copy: Done after its commit, NotDone before, Unknown
//! when half written; restore, the temp files, the check between prepare and commit (D77, D78).

use std::collections::BTreeMap;

use planrun_model::{Action, WriteMode};

use super::*;
use crate::fixture::Fixture;
use crate::handler::HandlerSet;
use crate::ir::{Ir, PATCH_T1, path};
use crate::transaction::{CommitCause, PreparedStep, WriteAhead, commit, prepare};

#[derive(Default)]
struct Recorder(usize);

impl WriteAhead for Recorder {
    fn record(&mut self, _step: &PreparedStep) -> std::io::Result<()> {
        self.0 += 1;
        Ok(())
    }
}

fn open(f: &Fixture) -> Workspace {
    Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
}

fn prepared(
    ws: &Workspace,
    ir: &mut Ir,
    make: impl FnOnce(&mut Ir) -> Vec<Action>,
) -> PreparedStep {
    let actions = make(ir);
    let refs: Vec<&Action> = actions.iter().collect();
    prepare(ws, &ir.source(), &HandlerSet::files(), &refs)
        .unwrap_or_else(|f| panic!("{}", f.message))
}

fn state(ws: &Workspace, step: &PreparedStep) -> DoneState {
    step_state(ws, &step.record())
        .unwrap_or_else(|e| panic!("{e}"))
        .state
}

fn work_tree(f: &Fixture) -> BTreeMap<String, Vec<u8>> {
    f.snapshot()
        .into_iter()
        .filter(|(k, _)| !k.starts_with(".git/"))
        .collect()
}

fn three(ir: &mut Ir) -> Vec<Action> {
    vec![
        ir.write("src/greet.rs", "pub fn greeting() {}\n", WriteMode::Create),
        ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a"),
        ir.delete("src/old.rs", true),
    ]
}

#[test]
fn a_step_is_not_done_before_its_commit_and_done_after_it() {
    for autocrlf in [false, true] {
        let f = Fixture::greeter(autocrlf);
        let ws = open(&f);
        let mut ir = Ir::default();
        let step = prepared(&ws, &mut ir, three);
        assert_eq!(state(&ws, &step), DoneState::NotDone, "{autocrlf}");
        commit(&ws, &step, &mut Recorder::default()).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(state(&ws, &step), DoneState::Done, "{autocrlf}");
    }
}

#[test]
fn a_half_written_step_is_unknown_and_restore_puts_it_back() {
    let f = Fixture::greeter(true);
    let ws = open(&f);
    let before = work_tree(&f);
    let mut ir = Ir::default();
    let step = prepared(&ws, &mut ir, three);
    f.write("src/greet.rs", b"pub fn greeting() {}\n");
    let found = step_state(&ws, &step.record()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(found.state, DoneState::Unknown);
    assert_eq!(
        found.paths.iter().map(|(_, s)| *s).collect::<Vec<_>>(),
        [Seen::Post, Seen::Pre, Seen::Pre]
    );
    assert!(restorable(&ws, &step.record()).unwrap_or_else(|e| panic!("{e}")));
    assert!(restore_step(&ws, &step.record()).is_empty());
    assert_eq!(work_tree(&f), before);
    assert_eq!(state(&ws, &step), DoneState::NotDone);
}

#[test]
fn a_git_delete_with_its_index_entry_still_there_is_unknown() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let step = prepared(&ws, &mut ir, |ir| vec![ir.delete("src/old.rs", true)]);
    std::fs::remove_file(f.path().join("src/old.rs")).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(state(&ws, &step), DoneState::Unknown);
    f.git(&["rm", "-q", "--cached", "src/old.rs"]);
    assert_eq!(state(&ws, &step), DoneState::Done);
    assert!(!restorable(&ws, &step.record()).unwrap_or_else(|e| panic!("{e}")));
}

#[test]
fn a_path_changed_between_prepare_and_commit_writes_and_records_nothing() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let step = prepared(&ws, &mut ir, three);
    f.write("src/lib.rs", b"//! someone else\n");
    f.write("src/greet.rs", b"appeared\n");
    let before = work_tree(&f);
    let mut recorder = Recorder::default();
    let err = commit(&ws, &step, &mut recorder).err();
    let changed = match err.map(|e| e.cause) {
        Some(CommitCause::Changed(paths)) => paths,
        other => panic!("{other:?}"),
    };
    assert_eq!(changed, [path("src/greet.rs"), path("src/lib.rs")]);
    assert_eq!(recorder.0, 0);
    assert_eq!(work_tree(&f), before);
}

#[test]
fn a_directory_the_step_creates_that_appeared_is_a_change() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let step = prepared(&ws, &mut ir, |ir| {
        vec![ir.write("docs/a.txt", "a\n", WriteMode::Create)]
    });
    std::fs::create_dir(f.path().join("docs")).unwrap_or_else(|e| panic!("{e}"));
    let err = commit(&ws, &step, &mut Recorder::default()).err();
    assert!(matches!(
        err.map(|e| e.cause),
        Some(CommitCause::Changed(_))
    ));
}

#[test]
fn only_the_temp_files_a_step_names_are_removed() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let step = prepared(&ws, &mut ir, three).record();
    let named = format!("src/{}", step.writes[1].temp);
    f.write(&named, b"half");
    f.write("src/.planrun-0123456789abcdef.tmp", b"not this run's");
    assert_eq!(temps(&ws, &step), vec![named.clone()]);
    assert!(f.read(&named).is_some(), "temps only reads");
    assert_eq!(remove_temps(&ws, &step), vec![named.clone()]);
    assert!(f.read(&named).is_none());
    assert!(f.read("src/.planrun-0123456789abcdef.tmp").is_some());
}

#[test]
fn a_pruned_pre_image_is_reported_missing() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let mut step = prepared(&ws, &mut ir, three).record();
    assert!(
        missing_pre_images(&ws, &step)
            .unwrap_or_else(|e| panic!("{e}"))
            .is_empty()
    );
    let gone = crate::git::BlobId::parse(&"1".repeat(40)).unwrap_or_else(|| panic!("hex"));
    step.writes[1].pre = PreImage::Stored(gone);
    assert_eq!(
        missing_pre_images(&ws, &step).unwrap_or_else(|e| panic!("{e}")),
        [gone]
    );
    assert!(!restorable(&ws, &step).unwrap_or_else(|e| panic!("{e}")));
}

#[cfg(unix)]
#[test]
fn an_executable_file_stays_executable_and_its_mode_counts() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::greeter(false);
    f.write("run.sh", b"echo a\n");
    let full = f.path().join("run.sh");
    let mut perms = std::fs::metadata(&full)
        .unwrap_or_else(|e| panic!("{e}"))
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&full, perms).unwrap_or_else(|e| panic!("{e}"));
    f.git(&["add", "run.sh"]);
    f.git(&["commit", "-q", "-m", "x"]);
    let ws = open(&f);
    let mut ir = Ir::default();
    let step = prepared(&ws, &mut ir, |ir| {
        vec![ir.write("run.sh", "echo b\n", WriteMode::Overwrite)]
    });
    assert_eq!(step.writes[0].record.mode.as_deref(), Some("100755"));
    commit(&ws, &step, &mut Recorder::default()).unwrap_or_else(|e| panic!("{e}"));
    assert!(Workspace::is_executable(&full));
    assert_eq!(state(&ws, &step), DoneState::Done);
    let mut perms = std::fs::metadata(&full)
        .unwrap_or_else(|e| panic!("{e}"))
        .permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(&full, perms).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(state(&ws, &step), DoneState::Unknown);
}
```

File `crates/planrun-exec/src/runner/mod.rs` (exact diff; save the block as `$env:TEMP\p6b-t8-runner-mod-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/runner/mod.rs b/crates/planrun-exec/src/runner/mod.rs
index 646fda7..8eaaa7a 100644
--- a/crates/planrun-exec/src/runner/mod.rs
+++ b/crates/planrun-exec/src/runner/mod.rs
@@ -8,6 +8,7 @@
 //! P4's [`Policy`] decides; the runner prompts (D69) and never turns one decision into another.
 
 mod gate;
+pub mod preview;
 pub mod resume;
 mod script;
 mod sessions;
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t8-runner-mod-rs.patch"
Remove-Item "$env:TEMP\p6b-t8-runner-mod-rs.patch"
```

File `crates/planrun-exec/src/runner/preview.rs` (full contents):

```rust
//! The resume preview (decisions D77, D85): what [`Runner::resume`](super::Runner::resume)
//! will find, read without the run lock and without writing anything, for a UI to show before
//! the user resumes. It decides nothing: resume takes the lock, checks everything again against
//! the disk as it is then, and asks its own questions. Where resume judges, the preview calls
//! the same function: [`head_verdict`], `again`, `resumed_for` and
//! [`preflight_resumed`](crate::preflight::preflight_resumed).

use planrun_model::{ActionKind, Plan, RepoPath};

use super::resume::{Again, Analysis, HeadVerdict, Past, again, head_verdict, resumed_for};
use crate::DoneState;
use crate::done::{StepOnDisk, restorable, step_state, temps};
use crate::git::Head;
use crate::journal::{self, At};
use crate::lock;
use crate::policy::DefaultClassifier;
use crate::preflight::preflight_resumed;
use crate::workspace::{Workspace, WorkspaceError};

/// What resume would find now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    /// A run holds the repository's lock now (D90): resume would be refused.
    pub live: bool,
    /// The plan's fingerprint is not the journal's: resume would stop.
    pub plan_changed: bool,
    /// `.git/index.lock` exists: resume would stop.
    pub index_lock: bool,
    /// HEAD now.
    pub now: Head,
    /// What resume makes of it (D77).
    pub head: HeadVerdict,
    /// The interrupted task's prepared steps, in the journal's order, as they are on disk.
    pub steps: Vec<StepPreview>,
    /// The scripts and commits that will ask "run again or keep" (D77).
    pub again: Vec<AgainPreview>,
    /// The paths resume will ask about as `resume-dirt` (D77). An Unknown step counts as done
    /// here: its paths are never asked about, since `[r]` puts them back and `[a]` verifies
    /// them.
    pub resume_dirt: Vec<RepoPath>,
}

/// One prepared step as it is now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepPreview {
    /// Where.
    pub at: At,
    /// Each path, and what that makes the step: Done is not written again, NotDone is, and
    /// Unknown is asked about.
    pub disk: StepOnDisk,
    /// For an Unknown step, whether `[r]` could restore it (D57); false otherwise.
    pub restorable: bool,
    /// The writer's temp files of the step that exist; resume removes them.
    pub temps: Vec<String>,
}

/// A script or commit that will ask "run again or keep".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgainPreview {
    /// Where.
    pub at: At,
    /// It was running when planrun was interrupted, not ended.
    pub running: bool,
}

/// Reads what resuming `analysis` on `plan` in `ws` would find: git and the disk are read,
/// nothing is written, and no lock is taken.
pub fn preview(
    ws: &Workspace,
    plan: &Plan,
    analysis: &Analysis,
) -> Result<Preview, WorkspaceError> {
    let git = ws.git();
    let git_dir = git.git_dir()?;
    let now = git.head()?;
    let head = head_verdict(ws, analysis, &now)?;
    let mut steps = Vec::new();
    for prepared in &analysis.prepared {
        let disk = step_state(ws, &prepared.step)?;
        let restorable = disk.state == DoneState::Unknown && restorable(ws, &prepared.step)?;
        steps.push(StepPreview {
            at: prepared.at.clone(),
            disk,
            restorable,
            temps: temps(ws, &prepared.step),
        });
    }
    let task = analysis
        .interrupted
        .as_deref()
        .and_then(|id| plan.tasks.iter().find(|t| &*t.id.raw == id));
    let mut again_list = Vec::new();
    let mut resume_dirt = Vec::new();
    if let Some(task) = task {
        again_list = asks_again(plan, task, analysis, &head);
        let done: Vec<bool> = steps
            .iter()
            .map(|s| s.disk.state != DoneState::NotDone)
            .collect();
        let resumed = resumed_for(task, &analysis.prepared, &done);
        resume_dirt = preflight_resumed(ws, plan, task, &resumed)?
            .into_iter()
            .filter(|f| f.code == "resume-dirt")
            .filter_map(|f| f.path)
            .collect();
    }
    let lock_dir = journal::dir(&git_dir);
    let live = lock::live(&lock_dir).map_err(|source| WorkspaceError::Io {
        path: lock_dir.join("lock.live").display().to_string(),
        source,
    })?;
    Ok(Preview {
        live,
        plan_changed: analysis.run.fingerprint != *plan.source.fingerprint,
        index_lock: git_dir.join("index.lock").exists(),
        now,
        head,
        steps,
        again: again_list,
        resume_dirt,
    })
}

/// The scripts and commits of the interrupted task that will ask "run again or keep". An action
/// that was running is left to the HEAD verdict when HEAD moved while it ran, and so is a
/// Commit that was running (resume runs it again or finds it made).
fn asks_again(
    plan: &Plan,
    task: &planrun_model::Task,
    analysis: &Analysis,
    head: &HeadVerdict,
) -> Vec<AgainPreview> {
    let mut found = Vec::new();
    for (at, past) in &analysis.past {
        let Some(action) = task
            .steps
            .get(at.step)
            .and_then(|s| s.actions.get(at.action))
        else {
            continue;
        };
        let (script, commit) = match &action.kind {
            ActionKind::RunScript(s) => (s, false),
            ActionKind::Commit(s) => (s, true),
            _ => continue,
        };
        let moved_it = matches!(head, HeadVerdict::MovedByRunning { at: a, .. } if a == at);
        if matches!(past, Past::Running { .. }) && (commit || moved_it) {
            continue;
        }
        let body = plan.source.resolve(&script.body);
        let class = DefaultClassifier.explain(script.shell, body).class;
        if let Again::Asks { running } = again(past, commit, class) {
            found.push(AgainPreview {
                at: at.clone(),
                running,
            });
        }
    }
    found
}
```

File `crates/planrun-exec/src/runner/resume.rs` (exact diff; save the block as `$env:TEMP\p6b-t8-runner-resume-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/runner/resume.rs b/crates/planrun-exec/src/runner/resume.rs
index 16ccb8b..393af50 100644
--- a/crates/planrun-exec/src/runner/resume.rs
+++ b/crates/planrun-exec/src/runner/resume.rs
@@ -14,16 +14,19 @@
 
 use std::collections::HashMap;
 
-use planrun_model::RepoPath;
+use planrun_model::{RepoPath, Task};
 
+use super::task::commit_problem;
 use super::{Answer, Flow, Result, Runner, Typed, task_paths};
 use crate::DoneState;
 use crate::done::{Seen, remove_temps, restorable, restore_step, step_state};
-use crate::git::Head;
+use crate::git::{GitError, Head};
 use crate::journal::{At, End, Line, Outcome, PathBlob, Record};
+use crate::policy::PolicyClass;
 use crate::preflight::Resumed;
 use crate::transaction::StepRecord;
 use crate::ui::{Choice, Question, QuestionKind, YES_OR_QUIT};
+use crate::workspace::Workspace;
 
 /// What the run record of the chain says.
 #[derive(Debug, Clone, PartialEq, Eq)]
@@ -104,6 +107,128 @@ impl Analysis {
     }
 }
 
+/// What resume makes of HEAD against the journal (D77, D79, D80).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum HeadVerdict {
+    /// HEAD is what the journal last recorded.
+    Unchanged,
+    /// HEAD moved by one commit on the HEAD before the Commit that was running, and holds what
+    /// that Commit was to hold: it is done.
+    CommitMade {
+        /// The Commit.
+        at: At,
+    },
+    /// HEAD moved by one commit while `at` was running: the user decides (keep or retry).
+    MovedByRunning {
+        /// The action that was running.
+        at: At,
+        /// Its kind as journaled (`script`, `commit`, `manual`).
+        kind: String,
+        /// HEAD before it.
+        recorded: Head,
+    },
+    /// HEAD moved otherwise: resume stops.
+    Moved,
+}
+
+/// What resume makes of HEAD being `now` (D77): the one judgement resume and the preview share.
+/// Reads git only.
+pub fn head_verdict(
+    ws: &Workspace,
+    analysis: &Analysis,
+    now: &Head,
+) -> std::result::Result<HeadVerdict, GitError> {
+    if *now == analysis.head {
+        return Ok(HeadVerdict::Unchanged);
+    }
+    let Some((at, Past::Running { kind, head, expect })) = analysis.running() else {
+        return Ok(HeadVerdict::Moved);
+    };
+    let Some(commit) = now.commit.as_deref() else {
+        return Ok(HeadVerdict::Moved);
+    };
+    let on_top = ws.git().parent(commit)?.as_deref() == head.commit.as_deref();
+    if !on_top || now.reference != head.reference {
+        return Ok(HeadVerdict::Moved);
+    }
+    if kind == "commit" && commit_problem(ws, head.commit.as_deref(), expect)?.is_none() {
+        return Ok(HeadVerdict::CommitMade { at: at.clone() });
+    }
+    Ok(HeadVerdict::MovedByRunning {
+        at: at.clone(),
+        kind: kind.clone(),
+        recorded: head.clone(),
+    })
+}
+
+/// What resume does with a script or commit that has a journaled past (D77).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub(crate) enum Again {
+    /// A Commit that ended well: it is done.
+    Done,
+    /// The user kept its result before: it is not run again.
+    Keep,
+    /// It runs again under its decision; `announce` says so first (a Read or Build script that
+    /// ran).
+    Runs { announce: bool },
+    /// It asks "run again or keep"; `running` when it was running, not ended.
+    Asks { running: bool },
+}
+
+/// What resume does with a script of class `class` whose journal says `past` (D77): the one
+/// rule resume and the preview share.
+pub(crate) fn again(past: &Past, commit: bool, class: PolicyClass) -> Again {
+    let ended_ok = matches!(
+        past,
+        Past::Ended(Outcome::Passed | Outcome::Confirmed | Outcome::Accepted)
+    );
+    if commit && ended_ok {
+        return Again::Done;
+    }
+    match past {
+        Past::Ended(Outcome::Kept) => Again::Keep,
+        Past::Ended(Outcome::SkippedPolicy) => Again::Runs { announce: false },
+        Past::Running { .. } => Again::Asks { running: true },
+        Past::Ended(_) if class < PolicyClass::Modify => Again::Runs { announce: true },
+        Past::Ended(_) => Again::Asks { running: false },
+    }
+}
+
+/// What preflight leaves out of the resumed task (D77): every path of the task and of its
+/// prepared steps may have been written by the interrupted run; the paths and actions of the
+/// steps in `done` (one flag per prepared step) are verified. Resume and the preview share it.
+pub(crate) fn resumed_for(task: &Task, prepared: &[Prepared], done: &[bool]) -> Resumed {
+    let mut resumed = Resumed {
+        done: Vec::new(),
+        own: task_paths(task),
+        verified: Vec::new(),
+    };
+    for (p, &done) in prepared.iter().zip(done) {
+        for w in &p.step.writes {
+            if !resumed.own.contains(&w.path) {
+                resumed.own.push(w.path.clone());
+            }
+        }
+        if !done {
+            continue;
+        }
+        for w in &p.step.writes {
+            if !resumed.verified.contains(&w.path) {
+                resumed.verified.push(w.path.clone());
+            }
+        }
+        let step = &task.steps[p.at.step];
+        let group = step.actions[p.at.action..]
+            .iter()
+            .take_while(|a| super::is_file_action(a))
+            .count();
+        resumed
+            .done
+            .extend((p.at.action..p.at.action + group).map(|a| (p.at.step, a)));
+    }
+    resumed
+}
+
 /// Reads the journal's last chain. `None` when the journal has no run.
 pub fn analyse(lines: &[Line]) -> Option<Analysis> {
     let start = lines
@@ -327,45 +452,37 @@ impl Runner<'_> {
     /// `action-end`, or `head-seen` for a retry), so a crash after it resumes from it.
     fn check_head(&mut self, analysis: &Analysis, replay: &mut Replay) -> Result<Flow> {
         let now = self.head()?;
-        if now == analysis.head {
-            if let Some((at, Past::Running { kind, .. })) = analysis.running()
-                && kind == "commit"
-            {
-                replay.past.remove(&key(at));
+        let (at, kind, head) = match head_verdict(self.ws, analysis, &now)? {
+            HeadVerdict::Unchanged => {
+                if let Some((at, Past::Running { kind, .. })) = analysis.running()
+                    && kind == "commit"
+                {
+                    replay.past.remove(&key(at));
+                }
+                return Ok(Flow::Go);
             }
-            return Ok(Flow::Go);
-        }
-        let stop = Flow::Stop(format!(
-            "HEAD is {} ({}), but the journal last recorded {} ({}): something else changed \
-             the repository",
-            now.commit.as_deref().unwrap_or("none"),
-            now.reference.as_deref().unwrap_or("detached"),
-            analysis.head.commit.as_deref().unwrap_or("none"),
-            analysis.head.reference.as_deref().unwrap_or("detached"),
-        ));
-        let Some((at, Past::Running { kind, head, expect })) = analysis.running() else {
-            return Ok(stop);
-        };
-        let Some(commit) = now.commit.as_deref() else {
-            return Ok(stop);
+            HeadVerdict::Moved => {
+                return Ok(Flow::Stop(format!(
+                    "HEAD is {} ({}), but the journal last recorded {} ({}): something else \
+                     changed the repository",
+                    now.commit.as_deref().unwrap_or("none"),
+                    now.reference.as_deref().unwrap_or("detached"),
+                    analysis.head.commit.as_deref().unwrap_or("none"),
+                    analysis.head.reference.as_deref().unwrap_or("detached"),
+                )));
+            }
+            HeadVerdict::CommitMade { at } => {
+                self.ui.status(&format!(
+                    "  the commit of {} was made before the interruption",
+                    at.task
+                ));
+                self.accepted(&at, Outcome::Passed, &now)?;
+                replay.commit_done = Some(key(&at));
+                return Ok(Flow::Go);
+            }
+            HeadVerdict::MovedByRunning { at, kind, recorded } => (at, kind, recorded),
         };
-        let on_top = self.ws.git().parent(commit)?.as_deref() == head.commit.as_deref();
-        if !on_top || now.reference != head.reference {
-            return Ok(stop);
-        }
-        if kind == "commit"
-            && self
-                .commit_problem(head.commit.as_deref(), expect)?
-                .is_none()
-        {
-            self.ui.status(&format!(
-                "  the commit of {} was made before the interruption",
-                at.task
-            ));
-            self.accepted(at, Outcome::Passed, &now)?;
-            replay.commit_done = Some(key(at));
-            return Ok(Flow::Go);
-        }
+        let at = &at;
         let question = Question {
             kind: QuestionKind::HeadMoved {
                 action: at.clone(),
@@ -432,22 +549,13 @@ impl Runner<'_> {
         // Any path of the task may have been changed by the interrupted run, by a step or by a
         // script: its dirt is a warning, and its manifest checks are left out. Dirt outside the
         // verified steps that a remaining file action writes over is asked about (D77).
-        let mut resumed = Resumed {
-            done: Vec::new(),
-            own: task_paths(task),
-            verified: Vec::new(),
-        };
+        let mut done_flags = Vec::new();
         for p in &prepared {
             for removed in remove_temps(self.ws, &p.step) {
                 self.ui.status(&format!(
                     "  removed a temp file of the interrupted run: {removed}"
                 ));
             }
-            for w in &p.step.writes {
-                if !resumed.own.contains(&w.path) {
-                    resumed.own.push(w.path.clone());
-                }
-            }
             let found = step_state(self.ws, &p.step)?;
             let done = match found.state {
                 DoneState::Done => true,
@@ -457,27 +565,15 @@ impl Runner<'_> {
                     Err(why) => return Ok(Err(why)),
                 },
             };
+            done_flags.push(done);
             if !done {
                 continue;
             }
-            for w in &p.step.writes {
-                if !resumed.verified.contains(&w.path) {
-                    resumed.verified.push(w.path.clone());
-                }
-            }
-            let step = &task.steps[p.at.step];
-            let group = step.actions[p.at.action..]
-                .iter()
-                .take_while(|a| super::is_file_action(a))
-                .count();
-            resumed
-                .done
-                .extend((p.at.action..p.at.action + group).map(|a| (p.at.step, a)));
             if let Some(r) = self.replay.as_mut() {
                 r.done_groups.push(key(&p.at));
             }
         }
-        Ok(Ok(resumed))
+        Ok(Ok(resumed_for(task, &prepared, &done_flags)))
     }
 
     /// A step that is neither done nor undone on disk (D14): restore it from its pre-images
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t8-runner-resume-rs.patch"
Remove-Item "$env:TEMP\p6b-t8-runner-resume-rs.patch"
```

File `crates/planrun-exec/src/runner/script.rs` (exact diff; save the block as `$env:TEMP\p6b-t8-runner-script-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/runner/script.rs b/crates/planrun-exec/src/runner/script.rs
index 7225c5b..690ba50 100644
--- a/crates/planrun-exec/src/runner/script.rs
+++ b/crates/planrun-exec/src/runner/script.rs
@@ -5,13 +5,13 @@
 use planrun_model::{Action, Script, ScriptIntent};
 
 use super::gate::Attempt;
-use super::resume::Past;
+use super::resume::{Again, again};
 use super::sessions::{NotRun, name};
 use super::task::TaskRun;
 use super::{Answer, Flow, Result, RunError, Runner, Typed, canonical, describe};
 use crate::expect::{self, Aligned, Confirm, Mismatch, Reason, Verdict};
 use crate::journal::{Outcome, PathBlob, Record};
-use crate::policy::{Decision, PolicyClass, Skip};
+use crate::policy::{Decision, Skip};
 use crate::shell::{RunStatus, ScriptRun};
 use crate::ui::{Choice, Position, Question, QuestionKind, RUN_SKIP_OR_QUIT};
 
@@ -143,36 +143,32 @@ impl Runner<'_> {
         let Some(past) = self.take_past(pos.task, (pos.step, pos.action)) else {
             return Ok(Replayed::Again);
         };
-        let ended_ok = matches!(
-            past,
-            Past::Ended(Outcome::Passed | Outcome::Confirmed | Outcome::Accepted)
-        );
-        if commit && ended_ok {
-            self.ui.status(&format!(
-                "  L{line:<5} {}: made before the interruption",
-                describe(action)
-            ));
-            return Ok(Replayed::Done);
-        }
-        let running = matches!(past, Past::Running { .. });
-        let why = match past {
-            Past::Ended(Outcome::Kept) => return Ok(Replayed::Keep),
-            Past::Ended(Outcome::SkippedPolicy) => return Ok(Replayed::Again),
-            Past::Running { .. } => {
-                "was running when planrun was interrupted; it may have done part of its work"
+        let body = self.plan.source.resolve(&script.body);
+        let class = self.classifier.explain(script.shell, body).class;
+        let running = match again(&past, commit, class) {
+            Again::Done => {
+                self.ui.status(&format!(
+                    "  L{line:<5} {}: made before the interruption",
+                    describe(action)
+                ));
+                return Ok(Replayed::Done);
             }
-            Past::Ended(_) => {
-                let body = self.plan.source.resolve(&script.body);
-                let class = self.classifier.explain(script.shell, body).class;
-                if class < PolicyClass::Modify {
+            Again::Keep => return Ok(Replayed::Keep),
+            Again::Runs { announce } => {
+                if announce {
                     self.ui.status(&format!(
                         "  L{line:<5} {}: ran before the interruption; it runs again",
                         describe(action)
                     ));
-                    return Ok(Replayed::Again);
                 }
-                "ran before the interruption"
+                return Ok(Replayed::Again);
             }
+            Again::Asks { running } => running,
+        };
+        let why = if running {
+            "was running when planrun was interrupted; it may have done part of its work"
+        } else {
+            "ran before the interruption"
         };
         let question = Question {
             kind: QuestionKind::Again { running },
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t8-runner-script-rs.patch"
Remove-Item "$env:TEMP\p6b-t8-runner-script-rs.patch"
```

File `crates/planrun-exec/src/runner/task.rs` (exact diff; save the block as `$env:TEMP\p6b-t8-runner-task-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/runner/task.rs b/crates/planrun-exec/src/runner/task.rs
index eb33bb2..9f84da6 100644
--- a/crates/planrun-exec/src/runner/task.rs
+++ b/crates/planrun-exec/src/runner/task.rs
@@ -7,12 +7,13 @@ use super::gate::Gate;
 use super::resume::Past;
 use super::{Answer, Flow, Result, RunError, Runner, Typed, describe, is_file_action, task_paths};
 use crate::finding::{Finding, Severity};
-use crate::git::StatusEntry;
+use crate::git::{GitError, StatusEntry};
 use crate::journal::{Outcome, PathBlob, Record};
 use crate::policy::Decision;
 use crate::preflight;
 use crate::transaction::{self, CommitCause};
 use crate::ui::{Choice, Position, Question, QuestionKind, YES_OR_QUIT};
+use crate::workspace::Workspace;
 
 /// The state of the task being run.
 pub(crate) struct TaskRun {
@@ -571,45 +572,14 @@ impl Runner<'_> {
         Ok(expect)
     }
 
-    /// Why a commit script did not commit what it should (D79): HEAD did not move, moved by
-    /// other than one commit on `before`, or its tree does not hold `expect`.
+    /// Why a commit script did not commit what it should (D79); see [`commit_problem`].
     pub(crate) fn commit_problem(
         &self,
         before: Option<&str>,
         expect: &[PathBlob],
     ) -> Result<Option<String>> {
-        let git = self.ws.git();
-        let head = git.head()?;
-        let Some(now) = head.commit.as_deref() else {
-            return Ok(Some("there is no commit at HEAD".to_owned()));
-        };
-        if Some(now) == before {
-            return Ok(Some(
-                "the commit script ran, but HEAD did not move".to_owned(),
-            ));
-        }
-        if git.parent(now)?.as_deref() != before {
-            return Ok(Some(
-                "HEAD moved, but not by one commit on top of the one before".to_owned(),
-            ));
-        }
-        let paths: Vec<&RepoPath> = expect.iter().map(|e| &e.path).collect();
-        let held = git.tree_blobs(now, &paths)?;
-        let wrong: Vec<&str> = expect
-            .iter()
-            .zip(&held)
-            .filter(|(e, h)| e.blob != **h)
-            .map(|(e, _)| e.path.as_str())
-            .collect();
-        if wrong.is_empty() {
-            return Ok(None);
-        }
-        Ok(Some(format!(
-            "the commit does not hold the task's files as they were: {}",
-            wrong.join(", ")
-        )))
+        Ok(commit_problem(self.ws, before, expect)?)
     }
-
     /// After a commit: paths it changed outside the task's, reported loudly (D80).
     pub(crate) fn commit_outside(&mut self, task: &Task, before: Option<&str>) -> Result<()> {
         let head = self.head()?;
@@ -656,3 +626,42 @@ fn generated(task: &Task, index: usize) -> Vec<(RepoPath, Position)> {
 fn first_line(text: &str) -> &str {
     text.lines().next().unwrap_or("")
 }
+
+/// Why a commit script did not commit what it should (D79): HEAD did not move, moved by other
+/// than one commit on `before`, or its tree does not hold `expect`. Reads git only.
+pub(crate) fn commit_problem(
+    ws: &Workspace,
+    before: Option<&str>,
+    expect: &[PathBlob],
+) -> std::result::Result<Option<String>, GitError> {
+    let git = ws.git();
+    let head = git.head()?;
+    let Some(now) = head.commit.as_deref() else {
+        return Ok(Some("there is no commit at HEAD".to_owned()));
+    };
+    if Some(now) == before {
+        return Ok(Some(
+            "the commit script ran, but HEAD did not move".to_owned(),
+        ));
+    }
+    if git.parent(now)?.as_deref() != before {
+        return Ok(Some(
+            "HEAD moved, but not by one commit on top of the one before".to_owned(),
+        ));
+    }
+    let paths: Vec<&RepoPath> = expect.iter().map(|e| &e.path).collect();
+    let held = git.tree_blobs(now, &paths)?;
+    let wrong: Vec<&str> = expect
+        .iter()
+        .zip(&held)
+        .filter(|(e, h)| e.blob != **h)
+        .map(|(e, _)| e.path.as_str())
+        .collect();
+    if wrong.is_empty() {
+        return Ok(None);
+    }
+    Ok(Some(format!(
+        "the commit does not hold the task's files as they were: {}",
+        wrong.join(", ")
+    )))
+}
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t8-runner-task-rs.patch"
Remove-Item "$env:TEMP\p6b-t8-runner-task-rs.patch"
```

File `crates/planrun-exec/tests/resume.rs` (exact diff; save the block as `$env:TEMP\p6b-t8-tests-resume-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/tests/resume.rs b/crates/planrun-exec/tests/resume.rs
index 22476fe..ca5f8c4 100644
--- a/crates/planrun-exec/tests/resume.rs
+++ b/crates/planrun-exec/tests/resume.rs
@@ -13,9 +13,13 @@ use std::time::Duration;
 
 use fixture::Fixture;
 use ir::{Ir, PATCH_T1, PATCH_T2};
+use planrun_exec::DoneState;
+use planrun_exec::done::Seen;
 use planrun_exec::journal::{self, End, Journal, Observer, Record, Seam};
+use planrun_exec::lock;
 use planrun_exec::policy::{Policy, PolicyClass};
-use planrun_exec::runner::resume::analyse;
+use planrun_exec::runner::preview::{Preview, preview};
+use planrun_exec::runner::resume::{HeadVerdict, analyse};
 use planrun_exec::runner::{Finished, RunConfig, Runner};
 use planrun_exec::shell::{OutLine, SessionConfig};
 use planrun_exec::ui::{Choice, Position, Question, QuestionKind, Reply, Ui};
@@ -293,6 +297,136 @@ fn a_crash_between_two_writes_is_unknown_and_restore_completes_it() {
     assert_done(&f);
 }
 
+#[test]
+fn the_preview_reads_what_resume_will_find_and_writes_nothing() {
+    let f = Fixture::greeter(false);
+    let mut ir = Ir::default();
+    let plan = assay(&mut ir);
+    let first_write = |s: &Seam<'_>| matches!(s, Seam::Wrote(1));
+    let (first, _) = go(&f, &plan, AUTO, vec![], crash_at(first_write), false);
+    assert!(first.is_none());
+    let path = f.outside().join("journal.jsonl");
+    let journal_before = std::fs::read(&path).unwrap_or_else(|e| panic!("{e}"));
+    let tree_before = f.snapshot();
+    let (seen, analysis) = preview_of(&f, &plan);
+    assert!(!seen.live && !seen.plan_changed && !seen.index_lock);
+    assert_eq!(
+        (&seen.now, &seen.head),
+        (&analysis.head, &HeadVerdict::Unchanged)
+    );
+    assert!(seen.again.is_empty(), "{:?}", seen.again);
+    assert!(
+        seen.resume_dirt.is_empty(),
+        "an Unknown step's paths are never resume-dirt: {:?}",
+        seen.resume_dirt
+    );
+    let [step] = &seen.steps[..] else {
+        panic!("{:?}", seen.steps);
+    };
+    assert_eq!((step.at.task.as_str(), step.at.step), ("P1-T1", 0));
+    assert_eq!(step.disk.state, DoneState::Unknown);
+    let paths: Vec<(&str, Seen)> = step
+        .disk
+        .paths
+        .iter()
+        .map(|(p, s)| (p.as_str(), *s))
+        .collect();
+    assert_eq!(
+        paths,
+        [("src/greet.rs", Seen::Post), ("src/lib.rs", Seen::Pre)]
+    );
+    assert!(step.restorable);
+    assert!(step.temps.is_empty(), "{:?}", step.temps);
+    assert_eq!(
+        f.snapshot(),
+        tree_before,
+        "the preview wrote to the repository"
+    );
+    let journal_after = std::fs::read(&path).unwrap_or_else(|e| panic!("{e}"));
+    assert_eq!(
+        journal_after, journal_before,
+        "the preview wrote to the journal"
+    );
+}
+
+/// The preview of the fixture's journal, with the analysis it was made from.
+fn preview_of(f: &Fixture, plan: &Plan) -> (Preview, planrun_exec::runner::resume::Analysis) {
+    let ws = Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"));
+    let lines = journal::read(&f.outside().join("journal.jsonl")).unwrap_or_else(|e| panic!("{e}"));
+    let analysis = analyse(&lines).unwrap_or_else(|| panic!("nothing to resume"));
+    let seen = preview(&ws, plan, &analysis).unwrap_or_else(|e| panic!("{e}"));
+    (seen, analysis)
+}
+
+#[test]
+fn the_preview_names_what_resume_will_ask_and_sees_a_live_run() {
+    let f = Fixture::greeter(false);
+    let mut ir = Ir::default();
+    let m = ir.manifest(&[
+        ("src/greet.rs", ChangeKind::Create, W),
+        ("a.txt", ChangeKind::Create, W),
+        ("src/lib.rs", ChangeKind::Modify, W),
+    ]);
+    let steps = vec![
+        vec![ir.write("src/greet.rs", "pub fn greeting() {}\n", WriteMode::Create)],
+        vec![ir.script("Add-Content -Path a.txt -Value line")],
+        vec![ir.manual("look at it by hand")],
+        vec![ir.write("src/lib.rs", "pub mod greet;\n", WriteMode::Overwrite)],
+    ];
+    let task = ir.task("T1", m, steps);
+    let plan = ir.plan(vec![task]);
+    let at_manual = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::ActionStart { kind, .. }) if kind == "manual");
+    let (first, _) = go(&f, &plan, AUTO, vec![], crash_at(at_manual), false);
+    assert!(first.is_none());
+    f.write("src/lib.rs", b"// mine\n");
+    let journal_before =
+        std::fs::read(f.outside().join("journal.jsonl")).unwrap_or_else(|e| panic!("{e}"));
+    let tree_before = f.snapshot();
+    let (seen, _) = preview_of(&f, &plan);
+    assert_eq!(seen.head, HeadVerdict::Unchanged);
+    let again: Vec<(usize, bool)> = seen.again.iter().map(|a| (a.at.step, a.running)).collect();
+    assert_eq!(again, [(1, false)], "the Modify script that ran asks again");
+    let dirt: Vec<&str> = seen.resume_dirt.iter().map(|p| p.as_str()).collect();
+    assert_eq!(
+        dirt,
+        ["src/lib.rs"],
+        "the verified step's path is not asked about"
+    );
+    assert_eq!(
+        f.snapshot(),
+        tree_before,
+        "the preview wrote to the repository"
+    );
+    let journal_after =
+        std::fs::read(f.outside().join("journal.jsonl")).unwrap_or_else(|e| panic!("{e}"));
+    assert_eq!(
+        journal_after, journal_before,
+        "the preview wrote to the journal"
+    );
+
+    f.git(&["commit", "-q", "--allow-empty", "-m", "by hand"]);
+    let (seen, analysis) = preview_of(&f, &plan);
+    let HeadVerdict::MovedByRunning { at, kind, recorded } = &seen.head else {
+        panic!("{:?}", seen.head);
+    };
+    assert_eq!(
+        (at.step, kind.as_str(), recorded),
+        (2, "manual", &analysis.head)
+    );
+    let again: Vec<(usize, bool)> = seen.again.iter().map(|a| (a.at.step, a.running)).collect();
+    assert_eq!(
+        again,
+        [(1, false)],
+        "a moved HEAD leaves the script's question as it was"
+    );
+
+    let git_dir = f.path().join(".git");
+    let run = lock::acquire(&journal::dir(&git_dir)).unwrap_or_else(|e| panic!("{e}"));
+    assert!(preview_of(&f, &plan).0.live);
+    drop(run);
+    assert!(!preview_of(&f, &plan).0.live);
+}
+
 #[test]
 fn a_crash_after_the_commit_s_start_runs_the_commit() {
     let f = Fixture::greeter(false);
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t8-tests-resume-rs.patch"
Remove-Item "$env:TEMP\p6b-t8-tests-resume-rs.patch"
```

**Step 2 — verify**

Targeted: the two preview tests, and the step-state tests:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-exec --test resume preview
cargo test -p planrun-exec --lib done::tests
```

Expected (exit code 0; summary lines):

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 48 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-exec/src/done.rs crates/planrun-exec/src/done_tests.rs crates/planrun-exec/src/runner/mod.rs crates/planrun-exec/src/runner/preview.rs crates/planrun-exec/src/runner/resume.rs crates/planrun-exec/src/runner/script.rs crates/planrun-exec/src/runner/task.rs crates/planrun-exec/tests/resume.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/done.rs' = 'b7b95fed80e9d46b50f893cb0fd7bcfcec58b135'
      'crates/planrun-exec/src/done_tests.rs' = '8ccb4ab41b23d561ff2deb6e8f4b49227174aa91'
      'crates/planrun-exec/src/runner/mod.rs' = '8eaaa7ac717627a689111e26b9923a5d472f36f1'
      'crates/planrun-exec/src/runner/preview.rs' = '71114de36e92ed55d5ad74a542d39179c63f4b66'
      'crates/planrun-exec/src/runner/resume.rs' = '393af508f53f00ece38bfc734fea8f9e307cb066'
      'crates/planrun-exec/src/runner/script.rs' = '690ba509900ed4c3e58a6540b31f386ecf66699d'
      'crates/planrun-exec/src/runner/task.rs' = '9f84da67e286b6e461df2672ef4a165c1546067e'
      'crates/planrun-exec/tests/resume.rs' = 'ca5f8c4eb75865c41e6402711b6b081f137cb0f9'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6b-T8: the resume preview" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P6b-T9: architecture, extending and CLAUDE.md for the seams

**Goal:** The docs describe the new seams: architecture.md §6.3 (`end_now`), §6.4 (`align`), §6.6 (the liveness lock, the preview, the observers) and §7 (the typed dry-run report); extending.md §7 (`QuestionKind`, the typed words, and a new question is a new variant); CLAUDE.md's layout (`lock.live`, `runner/preview`, `docs/tui-design.md`).

**Files**

- Modify `CLAUDE.md`
- Modify `docs/architecture.md`
- Modify `docs/extending.md`

Over 300 lines, so it comes as an exact diff: `docs/architecture.md`; `CLAUDE.md` ends without a newline, so it comes as an exact diff too.

**Step 1 — write the files**

File `CLAUDE.md` (exact diff; save the block as `$env:TEMP\p6b-t9-claude-md.patch`, outside the repository, then apply it):

```diff
diff --git a/CLAUDE.md b/CLAUDE.md
index f73d216..8b85bba 100644
--- a/CLAUDE.md
+++ b/CLAUDE.md
@@ -23,14 +23,15 @@ crates/planrun-parse   Markdown → IR: source, document, format, spec, registry
 crates/planrun-exec    IR → effects: git, workspace, eol, tree, handler, transaction,
                        preflight, finding (P3); shell/ (sessions, kill.rs: the only unsafe
                        code), expect, policy/ (classifier, decisions) (P4); journal, lock,
-                       done, ui, runner/ (tasks, scripts, sessions, gate, resume) (P5)
+                       done, ui, runner/ (tasks, scripts, sessions, gate, resume) (P5);
+                       lock.live, runner/preview (P6b)
 crates/planrun-cli     the `planrun` binary: check, show (view, render), run (dry_run, run,
                        ui: line prompts), resume, status, update (P6a, D83); the `e2e` test
                        feature (e2e.rs)
 formats/*.toml         built-in plan formats (embedded): assay, superpowers, before-after; the
                        file name is the format id
-docs/                  architecture, plan-grammar, extending, decisions, roadmap, plans/,
-                       handovers/ (one per finished phase)
+docs/                  architecture, plan-grammar, extending, decisions, roadmap,
+                       tui-design, plans/, handovers/ (one per finished phase)
 tests/corpus/          synthetic/ (one plan per format), invalid/ (one error each), local/ (private)
 tests/fixtures/greeter the repo the synthetic assay plan applies to
 tests/support          fixture.rs: a temporary git copy of the greeter fixture, included
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t9-claude-md.patch"
Remove-Item "$env:TEMP\p6b-t9-claude-md.patch"
```

File `docs/architecture.md` (exact diff; save the block as `$env:TEMP\p6b-t9-docs-architecture-md.patch`, outside the repository, then apply it):

````diff
diff --git a/docs/architecture.md b/docs/architecture.md
index b2c61d1..3a5433a 100644
--- a/docs/architecture.md
+++ b/docs/architecture.md
@@ -317,8 +317,9 @@ implements `ShellSession`; P5 keeps one per shell.
 - **The process tree** (D64, D71, D73): a job object on Windows, `setsid` and `PR_SET_PDEATHSIG`
   on Linux; a timeout (none by default, D65), a drop, or a shell that exited while a descendant
   holds its pipes kills every descendant. On Unix SIGINT, SIGTERM and SIGHUP kill the sessions'
-  groups before planrun ends; while the `[o]` shell runs, Ctrl-C goes to it. `shell/kill.rs`
-  holds planrun's only `unsafe` code.
+  groups before planrun ends; while the `[o]` shell runs, Ctrl-C goes to it. `shell::end_now`
+  ends planrun the same way for a UI that reads Ctrl-C as a key (D91). `shell/kill.rs` holds
+  planrun's only `unsafe` code.
 - **In a run** (D81): one session per shell, started at the first block; a dead one is a
   mismatch whose retry starts a new one, announced by a notice that must be acknowledged (also
   on resume and with `--from`); a working directory outside the repository asks before the
@@ -351,6 +352,8 @@ Windows with PowerShell 7 is the primary platform (D19); Linux with pwsh and bas
   the run; `--run-expected-failures` runs it. Gates, previews and applies always run.
 - A mismatch stops and offers accept (journaled as `accepted`; it closes the commit gate), retry,
   open a shell, abort; a dead session offers only retry, shell and abort (D75).
+- `expect::align` gives each non-blank expected line the output line it matched, or none, as
+  `check` matches them; the `mismatch` question carries it (D86).
 
 ### 6.5 Policy
 
@@ -386,7 +389,8 @@ as on disk), `action-end` (outcome, status, the last output lines, HEAD), the us
 `gate-override`, `task-end`, `run-end`. One write per record; `step-prepared`, `action-start`
 and `run-end` are synced. A torn last line is dropped, and cut off before the next append; any
 other bad line is an error. One planrun runs per repository at a time: `<git-dir>/planrun/lock`
-(D77).
+(D77). A run also holds `lock.live`, with a blocking lock once `lock` is held, so `lock::live`
+sees a run in progress with a try on `lock.live` alone, never on `lock` (D90).
 
 Resume never trusts the journal alone (D14, D77). It reads the last chain (a `run` and the
 `resume`s after it) and stops when the plan's fingerprint differs, when `.git/index.lock`
@@ -403,6 +407,15 @@ recorded HEAD; where resume accepts a moved HEAD it journals the running action'
 `head-seen` for a retry) at once, so a stopped or crashed resume leaves the next one the same
 check.
 
+`runner::preview::preview` reads what resume would find without the lock and without writing,
+for the TUI's resume preview (D85): whether a run is live, the fingerprint, `index.lock`,
+resume's verdict on HEAD (`resume::head_verdict`, the function resume uses), each prepared step
+on disk with whether it can be restored and its temp files, the scripts that will ask "run
+again or keep" (the rule `replayed` uses), and the `resume-dirt` paths (`preflight_resumed`
+itself). Resume checks all of it again when it runs. A journal takes any number of observers (`Journal::observe`), called after each
+append and each write of a step's commit in the order they were added: the `e2e` fault seam
+(D82) and the TUI's live view (D90).
+
 ## 7. CLI
 
 ```
@@ -437,7 +450,9 @@ writes nothing: not the working tree, not `.git`. Once a script, commit or Manua
 without running, a failing file action is a warning ("depends on a step that did not run") and
 its path is unknown from then on; every ok after that point assumes those steps change no files.
 Each script and commit shows its class and what the default policy would do with it ("would ask
-(Build)", "typed yes (Destructive)", "skipped: expected failure", D69). Exit 0 when nothing fails, 1 when something does, 2 when git or the repository cannot be used.
+(Build)", "typed yes (Destructive)", "skipped: expected failure", D69). The report is typed
+(`preflight::DryRun`): a file action that would run carries its `handler::Effect`, whose text is
+what the report prints (D84). Exit 0 when nothing fails, 1 when something does, 2 when git or the repository cannot be used.
 
 The prompts are lines on stdout, answered by a line on stdin (D15, D74); only a terminal
 answers: a stdin that is not one stops the run at its first question. A sketch of the screen:
````

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6b-t9-docs-architecture-md.patch"
Remove-Item "$env:TEMP\p6b-t9-docs-architecture-md.patch"
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
| `ActionHandler` | `HandlerSet` | how an action kind is simulated on the virtual tree (D52, D53); the transaction writes the result, and resume compares the journaled step with the disk (`done`, D77), so a handler has no write and no `DoneState` of its own |
| `ShellSession` | one per `Shell` | another shell (`cmd`, `zsh`): a new `Shell` variant, a `Dialect` in `shell/` (launch arguments, prologue, block line, sentinel; D60), its discovery and probe (D63), and its splitter in `policy/` |
| `CommandClassifier` | the runner's | the tests' own classifier; `DefaultClassifier` is the built-in allowlist (D68), extended by adding rows to `policy/heads.rs` and their test rows, never from a format spec |
| `Ui` | the CLI | `status`, `output`, `ask` (a `Question`: its `QuestionKind` with what the run found (D86), named `Choice`s, or the words a typed line may be), `view`, `diff`, `shell`; the runner decides what is asked and acts on the answer (D74). planrun-cli's `LineUi` is line prompts on std; the runner tests script their answers; the TUI (docs/tui-design.md, D84–D92) and a non-interactive batch UI later. A new question is a new `QuestionKind` variant, and every UI's exhaustive match shows where it must be shown |
````

**Step 2 — verify**

Targeted: the new lines are there:

```powershell
(Select-String -Path docs/architecture.md -Pattern 'runner::preview::preview').Count
(Select-String -Path docs/extending.md -Pattern 'QuestionKind').Count
```

Expected (exit code 0; summary lines):

```text
1
1
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- CLAUDE.md docs/architecture.md docs/extending.md
  $verified = [ordered]@{
      'CLAUDE.md' = '8b85bba17c601d4602e3c1304750ff74f107b3ed'
      'docs/architecture.md' = '3a5433ac9944d205e9b20c67a8711424c3b4b7d4'
      'docs/extending.md' = '706633d8d413e78688dcf2c5e2e925b7992e8158'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6b-T9: architecture, extending and CLAUDE.md for the seams" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

## After execution

Each step is a manual step for Rens; none is run by planrun.

1. By hand: compare the executed tree with the verified one (`git ls-tree -r HEAD`) if the run was stopped and resumed.
2. By hand: push `main`; the `ci` job on `ubuntu-latest` runs `end_now`'s Unix path (`kill_registered`) and the flock half of `lock.live` for the first time.
3. Claude writes the P6b→P6c handover (CLAUDE.md, "When a phase is done"), with two open items: moving expect.rs's tests to `expect_tests.rs` (465 lines, stated decision 10), and the gap of D90 (a probe in the instant between a starting run's two locks sees no live run).
