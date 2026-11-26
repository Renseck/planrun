# planrun P5 — Runner, journal, resume, interactive CLI

**Goal:** P5 of docs/roadmap.md: `planrun run`, `resume` and `status`. The runner executes a plan task by task on the repository: preflight, each step's file actions as one transaction whose write-ahead record is the journal, scripts in persistent sessions judged by the matcher, every decision prompted on line prompts, and the plan's commit only behind the commit gate. The journal records every transition, and resume checks it against the disk and HEAD, so a run killed anywhere completes without writing anything twice. First the P4 review items the runner needs (sessions, sed). Decisions D71–D82. After P5, `planrun run tests/corpus/synthetic/assay-style.md --auto read,build,modify` on a fixture copy makes the plan's two commits with the expected trees, and the end-to-end tests kill it at a question and end it at three seams and resume it to the same result.

- **P5-T1** (docs): decisions D71–D82, with amended-by lines in D9, D14, D15, D18, D22, D57, D60, D61, D64, D65 and D68; D9 drops `dialoguer` and rejects `ctrlc`.
- **P5-T2** (`planrun-exec`): sessions end when the shell is gone while a descendant holds its pipes, and when a block cannot be written; the bash wrapper calls its commands through `builtin`; pwsh runs with `RemoteSigned`; `PR_SET_PDEATHSIG` on Linux; the interrupt handlers (D71, D73).
- **P5-T3** (`planrun-exec`): sed is Read only when its script neither writes nor runs commands (D72).
- **P5-T4** (`planrun-exec`): the step record, the disk checked again between prepare and commit, modes, temp names chosen at prepare, `delete-untracked`, and a journaled step's state on disk (`done`, D77, D78).
- **P5-T5** (`planrun-exec`): the journal, the run lock, HEAD and batch hashes from git (D76, D77).
- **P5-T6** (`planrun-exec`): the `Ui` trait and the runner: tasks, file transactions with the journal, scripts in sessions, prompts per decision, the commit gate, the checks around a commit (D73–D75, D79–D81).
- **P5-T7** (`planrun-exec`): resume: the journal's last chain against the disk and HEAD, the interrupted task run again (D77).
- **P5-T8** (`planrun-cli`, corpus, fixture): `run`, `resume` and `status` with line prompts; the `e2e` feature and the end-to-end acceptance; the synthetic plan writes `docs/modules.txt` with LF only, and the greeter ignores `Cargo.lock` (four snapshots change).
- **P5-T9** (docs, `CLAUDE.md`): architecture.md §6.1–§6.6 and §7, extending.md §7, plan-grammar §1.6 (the Apply invariant), the roadmap, CLAUDE.md, and the plan gate in docs/plans/README.md.

**Base:** `924095a` (`Handover and p4 done`), the P4 handover on top of `8f5639a` (P4-T7); there fmt, clippy, `cargo test` with `PLANRUN_REQUIRE_SHELLS=1` (25 `test result: ok` lines, 224 tests) and the local-corpus test pass. Execute from the commit of this plan. Gate per task: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, with `PLANRUN_REQUIRE_SHELLS=1`; from P5-T8 on also `cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings` and `cargo test -p planrun-cli --features e2e --test run` (D82; the feature does not exist before P5-T8).

**Shell and environment:** PowerShell 7 on Windows; Rust 1.95.0 (stable; `File::try_lock` needs 1.89); git on `PATH`; pwsh 7 on `PATH` and Git for Windows (D63); `cargo` on `PATH` for the end-to-end tests, which build the greeter fixture offline into a target directory of their own. No new crate; `dialoguer` leaves `Cargo.toml` and `Cargo.lock` in P5-T8 (an exact diff).

**Before P5-T1:** this plan committed, and a clean `git status --porcelain`. The real plans stay in `tests/corpus/local/` (git-ignored, D16); this plan quotes none of them. Execution tests run on temporary copies of `tests/fixtures/greeter` under the OS temp directory, never on this repository; the runner's sessions and the `planrun` binary of the end-to-end tests run in such copies, with their git isolation.

**How this plan was verified:** every task was built in a scratch clone of `924095a` outside this repository (`core.autocrlf=false`), one commit per task. At each commit fmt, clippy and `cargo test` (with `PLANRUN_REQUIRE_SHELLS=1`) pass, and from P5-T8 on the `e2e` clippy and test too, and so does the task's targeted command; the summary lines below come from those runs. After every task, `planrun show` and `planrun check` on the six local plans printed exactly what they printed at `924095a`, and the local-corpus test passed. The Linux code paths (`waitid`, `prctl`, the signal handler, executable bits) were compiled and linted with `cargo clippy --all-targets --target x86_64-unknown-linux-gnu -- -D warnings` (with and without `e2e`) but not run: no Linux machine with cargo and pwsh was at hand (stated decision 13). The plan was generated from that history (full contents from `git show`, exact diffs from `git diff`, blob ids from `git rev-parse`), `planrun check` passes on it, and it was replayed task by task on a fresh clone with `core.autocrlf=true`: every commit step committed, `git status --porcelain` was empty after each, and the replay's final tree is the scratch tree.

## Review rulings (P5 questions, 2026-11-25)

Each ruling is a decision in docs/decisions.md (all written in P5-T1); this list maps them. Where the build kept a different position, the stated decisions give the evidence.

1. **Gate at `924095a`** accepted; it is the commit that marks P4 done (stated decision 1).
2. **Q1** resume of the interrupted task, as refined: it runs again from its start in a new session; Done file steps are verified, not written again; a Read or Build script that ran runs again under its decision; a Modify or Destructive one asks "run again / keep" first; one journaled as running always asks; the session notice; Gates from an ended session run again before the commit; `[a]` needs a typed `yes`; `[r]` only when every pre-image is there (D77, D81, D75). The Commit's Done test is D79, with one change (stated decision 3).
3. **Q2** the commit gate as a question of its own, journaled as `gate-override`; its inputs are the last attempt of every script; accept only for `Exited`/`Threw` (D75).
4. **Q3** `cargo xtask` and `pwsh -File` stay Destructive, with the 49-reason count; the question moves to the P6 handover (D72).
5. **Q4** `PR_SET_PDEATHSIG` with the `getppid` check, Linux only (D71).
6. **Q5** `RemoteSigned`, with the Process-scope difference stated (D71).
7. **Q6** `dialoguer` dropped; prompts check `IsTerminal`; a non-terminal stdin stops the run at the first question; a typed `yes` exactly (D74; stated decision 6).
8. **Q7** the journal named by the plan's file stem, the path and fingerprint in every run record, a mismatch a hard stop, `--restart` to give a run up (D76).
9. **Q8** D57 by detection only (D77).
10. **Q9** both synthetic fixes; the snapshots that change are listed in stated decision 12; no test depended on `Cargo.lock` being untracked.
11. **Q10** a test-only feature, `e2e`, with a fault seam and five legs, each with its prompts spelled out (D82, P5-T8; stated decision 5).
12. **A** the lock, `File::try_lock`, `lock.pid` for the message (D77; stated decision 8). **B** the journal writer and reader (D76). **C** the interrupt handlers (D73; stated decisions 2 and 4). **D** `waitid(…WNOWAIT)` and a zero wait on Windows (D71). **E** `builtin` for every wrapper command (D71). **F** temp names journaled per path (D78). **G** the order re-hash → record → writes, with `[p]`/`[q]` (D78). **H** HEAD after every script, Manual, `[o]` and Commit; `index.lock` stops a resume (D80, D77). **I** dirt before and paths after a commit (D80). **J** the working directory outside the repository (D81). **K** modes from the virtual tree, journaled, compared (D78). **L** `[d]`, HEAD after `[o]`, resume's journaled flags, `status` read-only (D74, D76, D77).

## Stated decisions made while building

1. **Base.** `924095a`'s parent is `8f5639a` (P4-T7); it adds the P4→P5 handover and nothing else in the code. The gate passed there as listed under Base.
2. **Ctrl-C during `[o]`: a handler routine, not `SetConsoleCtrlHandler(NULL, TRUE)`** (the C ruling's premise that does not hold). Microsoft's documentation of `SetConsoleCtrlHandler` says of the NULL form: "This attribute of ignoring or processing CTRL+C is inherited by child processes." The `[o]` shell and every command started in it would ignore Ctrl-C. So `kill.rs` registers a handler routine once, which returns TRUE for Ctrl-C and Ctrl-Break while a flag is set and FALSE otherwise (Windows then ends planrun as before); handler routines are not inherited. Tested: `the_control_handler_swallows_ctrl_c_only_while_ignoring`.
3. **A Commit is made when HEAD's new tree holds the task's paths as they were on disk when the Commit started**, journaled in its `action-start`, not "the task's journaled post blob ids" (Q1). The post-images are not what gets committed when a script changes a written file between the step and the commit (`cargo fmt`, which D55 names), and a `Generated` path has no post-image at all; `docs/modules.txt` in the synthetic plan is one. The rest of the ruling holds: HEAD must have moved by one commit on `head_before`, and an exit 0 without that is a mismatch (D79; `a_commit_that_does_not_move_head_is_a_mismatch`, `a_commit_made_before_its_end_was_journaled_is_done`).
4. **`signal`, not `sigaction`**, for the Unix handler. `struct sigaction` is laid out differently per architecture and C library; declaring it by hand would be the riskiest declaration in `kill.rs`. glibc's `signal` installs a handler that stays installed (BSD semantics) and restarts interrupted calls; inside the handler only atomics, `kill`, `signal` and `raise` run, all async-signal-safe.
5. **The fault seam ends the process with `exit(134)`, not `abort()`.** No destructor runs either way (the journal, the sessions and the lock stay as a kill leaves them; the job objects die with the process). On Windows `abort()` is a fail-fast that raises Windows Error Reporting, which can wait for an answer on a test machine.
6. **A typed `yes` also knows `skip` and `quit`.** The exact line `yes` runs; `skip` skips a Destructive script and `quit` stops; anything else does not run and asks again, as ruled. Without them a typed question could only be left with Ctrl-C.
7. **Resume treats every path of the interrupted task as possibly written by the interrupted run**, not only the paths its steps prepared. Found while building: a Modify script of the task (`Add-Content`) had written a manifest Create path, and the resumed preflight stopped with `path-dirty` and `create-exists` (`a_modify_script_that_ran_before_asks_and_keep_leaves_it`). Their dirt is a warning and their manifest checks are left out; every remaining file action is still simulated exactly, and the steps found done are left out (D77). After review: dirt in a path that no step found done or accepted wrote, and that a remaining file action writes over, is `resume-dirt` and asked about once before the task's first action, whatever `--auto` says (it may be the user's edit between the crash and the resume); the verified steps' paths are left out of it (`a_change_after_the_crash_to_a_path_a_remaining_step_writes_is_asked_about`).
8. **`lock.pid` is a file of its own.** On Windows `File::try_lock` locks the whole file, and another process cannot read it, so a pid inside `lock` could never be shown.
9. **Resume takes the journaled `--auto`, `--from`, `--to` and `--run-expected-failures`**, each unless its flag is given; `--auto` can widen or narrow the policy on resume (the end-to-end legs use it). `run --restart` renames an unfinished journal to `<stem>.<milliseconds>.jsonl`.
10. **`[v]` shows the whole step** of the action: planrun-cli's view renders steps, and a step is short. An answer a question did not offer asks again.
11. **The `Ui` trait moved to `ui.rs`** (still `planrun_exec::Ui`) and grew `output`, `ask`, `view`, `diff` and `shell`; nothing implemented the P0 trait. `DoneState` stays in lib.rs; resume computes it per step from the journal (`done::step_state`), not per handler, so `ActionHandler` gets no new method.
12. **Snapshots.** Four change in P5-T8, each reviewed: `planrun__view__tests__assay_style_ir.snap` and `planrun__view__tests__assay_style_show.snap` (the generate script of the synthetic plan's P1-T2: `((Get-ChildItem src -Name | Sort-Object) -join "`n") + "`n" | Set-Content -NoNewline docs/modules.txt` instead of `Set-Content` plus `Add-Content ''`, which writes `[Environment]::NewLine` and so a CRLF on Windows), `cli__show_p1_t2_step_2.snap` (the same line), and `dry_run__dry_run_create_exists.snap` (`  the task's files: 1 error(s), 0 warning(s)` instead of `  files: 1 problem(s)`, the handover's item 23). The class of the changed script is still Modify, so no dry-run decision changes. No other snapshot changes, and no test or snapshot named an untracked `Cargo.lock`.
13. **Linux was compiled, not run.** The code behind `#[cfg(target_os = "linux")]` and `#[cfg(unix)]` passes clippy for `x86_64-unknown-linux-gnu`; the Unix-only tests (`siginfo_is_128_bytes`, `an_executable_file_stays_executable_and_its_mode_counts`) and the PDEATHSIG and signal paths have not run. They need a Linux run before P5 is called done there (D19: Windows first).
14. **The expected trees** of the acceptance are built in the test: the expected files committed on top of a second fixture copy, whose tree ids the two commits must equal; `src/lib.rs` at the second is asserted to be blob `6b269fc`, the plan's own id. The fixture has no history of its own, so the roadmap's "as the fixture's history" is read this way (the report's contradiction 1).
15. **The end-to-end legs, their prompts.** Leg 1 (run, `autocrlf` off and on): the two commits' prose confirmations. Leg 2: `--auto read,build`, killed at P1-T1's commit question (Modify); resume with `--auto read,build,modify`: the session notice, two confirmations; `src/greet.rs`'s mtime, set to a sentinel before the resume, is unchanged after it. Leg 3 (`step-prepared@P1-T2`): one confirmation; resume: the notice, one confirmation; P1-T2's step is prepared twice and written once. Leg 4 (`write1@P1-T2`, `autocrlf` on): one confirmation; resume: the Unknown step (`r`), the notice, one confirmation. Leg 5 (`commit-start@P1-T1`): no question; `status` shows `[>] P1-T1`; resume: the notice, two confirmations. Every leg checks the two subjects, both tree ids, an empty `status --porcelain`, and from the journal at most one `step-written` per step and no `step-prepared` after a `step-verified`. A kill inside `git commit` itself is planrun-exec's `a_commit_made_before_its_end_was_journaled_is_done`: the observer makes the commit, then panics.
16. **Nested cargo.** The end-to-end tests set `CARGO_TARGET_DIR` to a directory next to the fixture copy (an inherited one would block on the outer test's build lock) and `CARGO_NET_OFFLINE=true`; the greeter has no dependencies. The five legs take about 17 s.
17. **Local corpus.** `planrun show` and `planrun check` on all six local plans print exactly what they printed at `924095a` after every task. `local_classes` still reports 12 Read, 35 Build, 31 Modify and 33 Destructive scripts and commits: no local script uses sed in a way the new reader classes differently.
18. **Exit codes.** `run` and `resume`: 0 completed, 1 stopped (a failure, a check, `quit`, or no terminal to answer), 2 usage, git, the lock or the journal. `status`: 0.
19. **The P5 review's HEAD fix, journaled as an `action-end`, not `head-seen`** (required fix 1). A resume's `run` record no longer sets the recorded HEAD, as ruled. Where resume accepts a moved HEAD, a Commit found made gets an `action-end` with outcome `passed`, and the user's `keep` an `action-end` with `kept`, both with the new HEAD; only `retry` gets `head-seen`. Evidence: `check_head` treats an equal HEAD with a Commit journaled as running as not made and runs it again (`if now == analysis.head` … `replay.past.remove(&key(at))`, runner/resume.rs). With `head-seen` alone (tried), `a_commit_found_made_by_resume_is_not_run_again_after_a_second_crash` fails: the resumed commit prints `nothing to commit, working tree clean` and `the commit script ran, but HEAD did not move`; and `a_kept_head_move_survives_a_crash_right_after_the_answer` asks `head-moved` again. Both pass with the `action-end`. `retry` needs `head-seen`: without it the retried action's `action-start` sits on the moved HEAD, and a crash during it would stop the next resume.
20. **The `head-moved` question offers no `[o]`.** Found while making the fix: after `[o]` the runner journals `head-seen` with the current HEAD (D80), so opening a shell at that question and then quitting would record the moved HEAD, and the next resume would pass the check: the same bug by another way. `[d]` stays; to look around, quit (the lock is released) and resume after.
21. **`a_modify_script_that_ran_before_asks_and_keep_leaves_it` does not get the `resume-dirt` question**, unlike the review expected. The ruled rule asks about a path that a remaining file action of the task writes; that test's task is a script (`Add-Content -Path a.txt -Value line`), a script and a Manual (tests/resume.rs), so no file action writes `a.txt`, and its asked list stays `again`, `session`, `manual`. The case the fix is for is the new test `a_change_after_the_crash_to_a_path_a_remaining_step_writes_is_asked_about`: a step written and verified, then the user edits the next step's Overwrite target; resume under `--auto read,build,modify` asks `resume-dirt` (quit leaves the edit, continue writes over it), and the verified step's path is not listed.
22. **Deferred by the P5 review to the P5→P6 handover** (no change in this plan): (1) D79 and ignored paths: `commit_expect` hashes every present task path, also one git ignores (a Generated build output, a lockfile in `.gitignore`), so such a commit is always a mismatch; it fails closed; later, leave ignored paths out (`check-ignore`). (2) Accept on a D79 problem (HEAD did not move, not one commit on top) journals the Commit as accepted and the next task runs on top; later, a typed `yes` for that accept, or no accept. (3) `Tree::wait` unregisters the group before the reap: a SIGINT in that window spares a dead shell's live descendants; note it with D73's stated limit. (4) The Linux paths (stated decision 13) run on a Linux machine before P5 counts as done there.

## Tasks

### P5-T1: decisions D71-D82: sessions, sed, interrupts, prompts, the commit gate, the journal, resume, transactions, commits, the e2e feature

**Goal:** The decisions first, so the code can cite them: D71–D82 at the end of docs/decisions.md, amended-by lines in the decisions they change, and D9's table without `dialoguer` (it leaves the manifests in P5-T8), with `dialoguer` and `ctrlc` among the rejected crates.

**Files**

- Modify `docs/decisions.md`

Over 300 lines, so they come as exact diffs: `docs/decisions.md`.

**Step 1 — write the files**

File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p5-t1-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index 023f31b..1fe6fb7 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -65,15 +65,17 @@ Each dependency is justified here; add none without a line.
 | `sha1_smol` | git blob ids in-process: the `blob-chain` check from a created file's contents (planrun-parse only; planrun-exec asks `git hash-object`, D56) |
 | `similar` | showing diffs of overwrites and replacements. **Not** for applying patches (D20) |
 | `clap` | arguments |
-| `dialoguer` | line prompts (D15) |
 | `annotate-snippets` | diagnostics with source context |
 | `insta` (dev) | golden snapshots of the IR |
 | `sha2` (Phase 1) | the plan fingerprint (SHA-256) |
 
 Rejected: `rusqlite` (D18), `ratatui` for now (D15), `tracing` for now (the journal is the record),
-`similar` as a patch engine (D20), `windows-sys` and `libc` for the process tree (D64).
+`similar` as a patch engine (D20), `windows-sys` and `libc` for the process tree (D64), `dialoguer`
+(D74: it refuses a stdin that is not a terminal), `ctrlc` (D73: it brings `nix` and
+`windows-sys`).
 
-Amended by D56 (no `sha1_smol` in planrun-exec) and D64 (the hand-declared process-tree calls).
+Amended by D56 (no `sha1_smol` in planrun-exec), D64 (the hand-declared process-tree calls), D73
+(`ctrlc` rejected) and D74 (`dialoguer` dropped).
 
 ## D10 Line endings of written files
 
@@ -110,11 +112,15 @@ Amended by D68 (the classifier, a built-in allowlist) and D69 (`--auto`, Read as
 An action recorded `Running` or `Done` is re-examined: postimages present → done; preimages
 present → not done; otherwise the user decides.
 
+Amended by D77 (resume, per action kind).
+
 ## D15 Line prompts first
 
 `dialoguer` and plain output keep cargo's output in the terminal's scrollback. A ratatui TUI can
 come later behind the `Ui` trait.
 
+Amended by D74 (line prompts on std, without `dialoguer`).
+
 ## D16 Real plans stay local
 
 The six real plans contain customer and internal details. They live in `tests/corpus/local/`
@@ -131,6 +137,8 @@ One writer, sequential, a few hundred records per plan: SQLite adds a C dependen
 file nobody can read when it matters. JSONL is appendable, crash-tolerant (a torn last line is
 dropped), and readable. Inside `.git/` so `git status --porcelain` stays empty.
 
+Amended by D76 (one file per plan, the records, when they are synced).
+
 ## D19 Windows first
 
 Every real plan is PowerShell 7 on Windows. Linux (pwsh and bash) is supported and is where CI
@@ -151,6 +159,8 @@ matches a format's manual markers, and edits described in prose are `Manual` (al
 The plan's commit script runs as written (it may gate itself on `$gate`), and planrun runs it only
 when every verification of the task passed or was skipped by policy (D12).
 
+Amended by D75 (the commit gate, a question of its own).
+
 ## D23 File actions are transactional per step
 
 All file actions of a step, up to the next non-file action, are validated in memory before any is
@@ -588,6 +598,8 @@ than `gc.pruneExpire` (two weeks by default). P5 decides whether a resume past t
 detected (the object is gone: the user decides, D14) or whether the objects are pinned (for
 example by a ref under `refs/planrun/`). Recorded now, not solved in P3.
 
+Settled by D77: detection, no pin.
+
 ## D58 A write without a mode is an error
 
 A `WriteFile` whose mode is still `FromManifest` at a path no manifest entry declares (a task
@@ -658,6 +670,8 @@ the terminal it carries over between commands); a terminating error ends the blo
 pwsh `$PSScriptRoot` and `$MyInvocation` name the block's temp file; pwsh `exit n` ends only the
 block (D61); `GIT_TERMINAL_PROMPT=0` (D62).
 
+Amended by D71 (`builtin` for the wrapper's commands, RemoteSigned).
+
 ## D61 How a block ends
 
 `RunStatus` is `Exited(code)`, `Threw` (pwsh), `SessionEnded(Option<code>)`, `TimedOut` or
@@ -686,6 +700,8 @@ block (D61); `GIT_TERMINAL_PROMPT=0` (D62).
   In the six local plans at most 6 of 222 pwsh fences use a silencing probe, all
   `SilentlyContinue`: accepted as noise.
 
+Amended by D71 (the shell gone while its pipes are held) and D81 (a dead session in a run).
+
 ## D62 The session environment
 
 A session gets planrun's own process environment minus `BASH_ENV` and `ENV` (bash reads
@@ -745,6 +761,8 @@ Amends D9 (no crate).
   144 bytes on 64-bit targets, and the read-back flag is tested, so a wrong layout fails a test.
   `windows-sys` would bring hundreds of generated declarations for five functions.
 
+Amended by D71 (PDEATHSIG, `waitid`) and D73 (the interrupt handlers).
+
 ## D65 No default timeout yet
 
 A block may run as long as it runs: cargo, dotnet and npm runs are long, and Rens' terminal has
@@ -752,6 +770,8 @@ no timeout. `SessionConfig::timeout` exists; the tests use 60 seconds (5 for the
 which checks that a grandchild dies with the tree, on both platforms). Whether the runner has a
 default is a P5 decision.
 
+Amended by D73: still no default timeout.
+
 ## D66 Matching a run against its expectation
 
 Amends D11; corrects architecture.md §6.4.
@@ -847,6 +867,8 @@ per format. The tables are Rust constants in `crates/planrun-exec/src/policy/hea
 - The local plans' scripts are classified by an ignored test in planrun-cli
   (`local_classes`), whose output stays in the terminal (D16).
 
+Amended by D72 (sed; xtask and repository scripts stay Destructive).
+
 ## D69 `--auto`, and what P4 decides for P5
 
 Amends D13. `--auto` takes `read`, `build` and `modify`; `destructive` is a usage error. Without
@@ -868,3 +890,286 @@ Such fences stay `Manual(HumanCheck)` (`manual-prose#run-from`). What it would t
 `Script.workdir` with `skip_serializing_if`, so no IR snapshot changes; the run-from lead
 becoming a script lead; and the directory resolved from the session (D60 reports the working
 directory after every block) and checked against the repository root before the block runs.
+
+## D71 Sessions that cannot hang or drift
+
+Amends D60, D61 and D64.
+
+- **The shell gone, its pipes held.** `Session::collect` waits for events in slices of 100 ms.
+  Between slices it asks whether the shell itself has exited, without reaping it: on Linux
+  `waitid(P_PID, pid, WEXITED | WNOHANG | WNOWAIT)` (std's `try_wait` would reap the shell, and
+  D64 allows `kill(-pgid)` only before the reap, when the group id cannot have been reused); on
+  Windows a zero-timeout `WaitForSingleObject` on the process handle the tree keeps open. When
+  it has, the tree is killed so that a descendant holding stdout or stderr (a build server, a
+  background job) lets go, and the block ends as `SessionEnded(None)`. Other Unix systems answer
+  "not exited" and keep the P4 behaviour (D19).
+- **A block that cannot be written ends the session.** Any I/O error while `ShellSession::run`
+  writes or renames `b<n>` kills the session: the shell would wait for that file forever, and
+  the next block would be numbered past it.
+- **Every command the bash wrapper runs goes through `builtin`:** `source`, `local`, `printf`,
+  `pwd`. A plan function of any of those names changes nothing. The prologue's own `unset` and
+  `trap` run before any plan code. Not defended: a plan function named `builtin`, or one named
+  like the wrapper's `__planrun_*` functions.
+- **The shell dies with planrun on Linux.** The `pre_exec` of every session sets
+  `PR_SET_PDEATHSIG` to SIGKILL after `setsid`, then compares `getppid()` with planrun's process
+  id read before the fork and refuses to start when they differ (the parent died in between).
+  The signal follows the thread that spawned the child, so sessions are spawned from planrun's
+  main thread. It reaches the shell only, not its descendants (D73). Other Unix systems are out
+  of scope (D19). `prctl`, `getppid` and `waitid` are declared by hand in `shell/kill.rs`, with
+  SAFETY comments; `siginfo_t` is read only for its first field and its size is tested.
+- **`-ExecutionPolicy RemoteSigned`** replaces `Bypass` for the pwsh session on Windows: the
+  block files are local and run; a downloaded script a block calls keeps the Mark-of-the-Web
+  check of Rens' terminal. A `-ExecutionPolicy` argument sets the Process scope, which outranks
+  the user's CurrentUser and LocalMachine settings: a terminal set to Bypass or Unrestricted is
+  less strict than the session. A group policy (MachinePolicy, UserPolicy) still wins over both.
+
+## D72 sed is read; xtask and repository scripts stay Destructive
+
+Amends D68.
+
+- **sed** without `-i` is Read only when its script neither writes nor runs anything. The
+  script (`-e`, `--expression`, or the first operand) is read command by command with GNU's
+  syntax: addresses, `!`, `{ }`, `s` with any delimiter and its flags, `y`, the text of `a`,
+  `i`, `c` (continued lines included), labels, `r`/`R`. `w`, `W` and the `s///w` flag are
+  Modify; `e` and the `s///e` flag are Destructive; so is `-f`/`--file` (a script the
+  classifier never sees), a script with an expansion, an unknown option, and anything the
+  reader does not recognise with confidence. `-i` stays Modify.
+- **`cargo xtask …` and `pwsh -File <relative path>`** stay Destructive. `cargo test` is Build
+  because a test runner is what it is; `xtask` is a general task runner, and a repository script
+  can do anything, deploys and releases included. Under `--auto modify` either would run
+  unasked. D13 holds: a command whose effect the tables do not know is Destructive. In the six
+  local plans these two account for 49 of the Destructive reasons; the question goes to the P6
+  handover with the dogfooding data.
+
+## D73 Interrupts and timeouts
+
+Amends D64 and D65; D9 rejects `ctrlc`.
+
+- **No default timeout** (D65 holds). A hung block is ended by the user: Ctrl-C ends planrun,
+  and `resume` goes on from the journal.
+- **Windows:** no handler on the normal path. Ctrl-C ends planrun and the job objects
+  (`KILL_ON_JOB_CLOSE`) kill every session's tree.
+- **Unix:** `setsid` keeps SIGINT and SIGHUP from the sessions, and PDEATHSIG (D71) kills only
+  the shell, so a running build would keep writing after planrun died. planrun installs a
+  handler for SIGINT, SIGTERM and SIGHUP that does only async-signal-safe things: `kill(-pgid,
+  SIGKILL)` for every registered session group (a static array of atomics; a slot is cleared
+  before its leader is reaped, as D64 requires), then restores the default action and raises
+  the signal again. It is installed with `signal`, not `sigaction`: `struct sigaction` differs
+  between architectures and C libraries, and a hand-declared layout would be the riskiest
+  declaration in `kill.rs`; glibc's `signal` keeps the handler installed and restarts
+  interrupted calls.
+- **While the `[o]` shell runs** planrun ignores Ctrl-C, so it reaches the shell and not
+  planrun. Windows: a console control handler routine that returns TRUE for Ctrl-C and
+  Ctrl-Break while a flag is set. Not `SetConsoleCtrlHandler(NULL, TRUE)`: that attribute is
+  inherited by child processes, so the shell and every command started in it would ignore
+  Ctrl-C too. Unix: the same flag makes the handler ignore SIGINT.
+- **What an interrupt leaves.** It can land anywhere, inside a step's commit too. Resume
+  recognises every state it can leave (D77): a step as before, as planned, or mixed (Unknown);
+  a journal whose last line is torn (dropped, D76); a writer's temp file (removed by name,
+  D77). A SIGKILL of planrun on Unix leaves a session's descendants running: stated, not
+  solved.
+- `ctrlc` is rejected (D9): it brings `nix` and `windows-sys`, which D64 already rejected.
+
+## D74 Line prompts on std
+
+Amends D15; D9 drops `dialoguer`.
+
+- **std only.** `dialoguer` refuses a stdin that is not a terminal, and its prompts redraw lines.
+  The prompts are plain lines on stdout and a line read from stdin, so cargo's output stays in
+  the scrollback. planrun-cli implements `Ui`; planrun-exec's runner decides what is asked.
+- **Only a terminal answers.** When stdin is not a terminal (`IsTerminal`), the first question
+  stops the run with an explanation, journaled as `run-end` with the reason; there is never a
+  default answer. planrun-cli's `e2e` feature lifts this for its own tests (D82).
+- **A typed `yes`** is the line with only its trailing `\n` or `\r\n` removed, equal to `yes`
+  exactly: case-sensitive, nothing trimmed. `skip` skips a Destructive script and `quit`
+  stops; anything else does not run and asks again.
+- **The prompt per decision** (D69): `Auto` shows one line and runs; `Ask` shows the script's
+  head and offers `[enter] run [s] skip [v] view [d] diff [o] shell [q] quit`; `AskUnscoped`
+  shows op, path, PathOrigin and plan line with the same keys; `TypedYes` shows the whole
+  script and the classifier's reasons; `Wait` (a Manual) shows its text and waits for
+  `[enter]`; `Show` prints a Note; `Skip` prints why. `[v]` shows the step in full, `[d]` runs
+  `git --no-optional-locks diff HEAD` and lists untracked files, `[o]` starts an interactive
+  shell at the root that is not the plan's session (it says so). An answer the question did
+  not offer asks again. Every answer is journaled.
+- **Questions of their own**, asked whatever `--auto` says: the session notice (D81), a working
+  directory outside the repository (D81), dirt before a commit (D80), a closed commit gate
+  (D75), a step changed between prepare and commit (D78), and resume's Unknown step and
+  interrupted script (D77). Each answer is journaled with the question's name.
+
+## D75 The commit gate
+
+Amends D22. The commit script still runs as written, and may gate itself on `$gate`.
+
+- **A question of its own**, before the Commit: when the gate is closed, the user sees every
+  reason and opens it only with a typed `yes`, whatever `--auto` says, journaled as
+  `gate-override` with the reasons. The Commit's own decision then applies unchanged: P4
+  decides, P5 prompts, and P5 never turns a decision into another one.
+- **Its inputs** are the last attempt of every script of the task (a passing retry replaces
+  the mismatch), not only Verify and Gate, the file actions the user skipped, and the
+  `Generated` checks. It is closed by an unresolved mismatch, a confirmation answered no, a user
+  skip of any script or file action, an accepted mismatch, a missing `Generated` path, and a
+  Gate that ran only in a session that has ended. A skip by policy (D67) keeps it open.
+- **A Gate from an ended session runs again** before the commit, under its decision: its
+  `$gate` is gone with the session, so its verdict no longer counts.
+- **Accept** is offered only for an `Exited` or `Threw` mismatch. For `SessionEnded`, `TimedOut`
+  and `Broken` the answers are retry (a new session, D81), shell and abort: accepting a dead
+  session would send the commit block into a session without its variables.
+- A Gate's confirmation always asks (D11); "no" counts as failed.
+
+## D76 The journal
+
+Amends D18; corrects architecture.md §6.6.
+
+- **One file per plan:** `<git-dir>/planrun/<plan file stem>.jsonl` (`rev-parse
+  --absolute-git-dir`, so a linked worktree has its own). Every `run` record stores the plan's
+  path (repository-relative, or canonical when the plan is outside the repository) and its
+  fingerprint. A journal of that stem whose records name another path is a hard stop that names
+  both: two plans never share a file, and planrun never picks another name on its own. The
+  fingerprint is not the name: a changed plan must find its journal to be told it changed.
+- **Runs are appended.** A `run` record starts a chain; each `resume` appends its own `run`
+  record (`resume: true`) to the chain. `run` refuses a plan whose last chain did not complete
+  and points to `resume`; `run --restart` gives the unfinished run up by renaming its journal to
+  `<stem>.<milliseconds>.jsonl`. Nothing deletes a journal.
+- **Records**, each with the schema version and the time: `run`, `task-start`,
+  `step-prepared` (the write-ahead record: per path the pre-image id or absent, the post-image's
+  exact and filtered ids, the modes, the directories it creates and the writer's temp name),
+  `step-written`, `step-failed`, `step-verified`, `step-restored`, `step-accepted`,
+  `action-start` (with the decision, `Auto` included, HEAD before it, and for a Commit the
+  task's paths as they are on disk), `action-end` (outcome, status, `errored`, the working
+  directory, the last 20 output lines, HEAD after it), `answer`, `gate-override`, `head-seen`,
+  `session`, `task-end`, `run-end`. No output digest: nothing reads one.
+- **Writes:** one `write_all` per record (the JSON and its `\n`), no buffer across records.
+  `step-prepared`, every `action-start` and `run-end` are followed by `sync_data`; the rest
+  reach the OS only. A killed process loses nothing the OS has; `sync_data` matters only after a
+  power cut, and those three are the records whose loss would change what resume does. The
+  plan's own file writes are not synced: after a power cut a step is neither as before nor as
+  planned, and resume asks (Unknown, D77).
+- **Reads:** a line counts only when it ends in `\n`. A last line without one is dropped even
+  when it parses, and before the first append the file is cut back to its last `\n`, so the next
+  record never joins a fragment. A bad line that is not the last one is an error, never
+  skipped. `status` reads the same way and takes no lock.
+- **An observer** sees every appended record and every path a step's commit wrote; planrun-cli
+  uses it for its fault seam (D82).
+
+## D77 Resume, and one planrun per repository
+
+Amends D14 and D57.
+
+- **What is read:** the journal's last chain. It stops, with an explanation, when the plan's
+  fingerprint differs from the chain's (both are shown), when `.git/index.lock` exists (planrun
+  never removes it), and when HEAD or the ref it points to is not what the journal last recorded
+  (HEAD is recorded after every script, Manual, `[o]` and Commit, D80). The one exception is
+  the action that was running: when HEAD moved by one commit on the recorded one, a Commit
+  whose new tree holds what it was to hold is done (D79); for any other action the user
+  decides (keep it as done, or run it again).
+- **The interrupted task runs again from its start**, in a new session, under the journaled
+  policy and range unless the flags say otherwise. Before its first script a notice is shown
+  that must be acknowledged even with `--auto` (D81).
+- **File steps** the journal prepared are checked against the disk, never trusted from it: every
+  path as planned (bytes, mode under `core.filemode`, and for a git delete the index entry
+  gone) is Done, and the step is not written again (`step-verified`); every path as before is
+  NotDone, and the step is prepared and written again; anything else is Unknown: `[r]` restores
+  it from its pre-images and writes it again, offered only when every pre-image object is still
+  there (`cat-file --batch-check`; `git gc` may have pruned them, D57) and no git delete of the
+  step has left the index yet; `[a]` accepts it as done, which overrides the disk check and so
+  needs a typed `yes`. The writer's temp files the step named are removed by name, never found
+  by pattern.
+- **D57 is settled by detection.** Done and NotDone compare hashes and need no object; only
+  `[r]` needs the pre-images. No ref pins them. Rollback inside a run is unaffected: its objects
+  are minutes old.
+- **Scripts:** one that ended runs again under its decision; a Read or Build one is only
+  announced (without `--auto` it asks as always), a Modify or Destructive one first asks "ran
+  before the interruption: run again or keep". One journaled as running asks the same, whatever
+  its class and `--auto`, shown as "was running when interrupted". A kept script keeps its
+  result; a kept Gate still runs again before the commit (D75). A Manual acknowledged before is
+  not asked again. A Commit that was running when HEAD did not move runs again under its
+  decision.
+- **Preflight of the resumed task** leaves out the file steps found done, and every path of the
+  task may have been changed by the interrupted run (by a step or by a script): dirt in those
+  paths is a warning, and their manifest checks are left out. The remaining file actions are
+  simulated exactly.
+- **One planrun per repository:** `<git-dir>/planrun/lock`, held with `File::try_lock` (stable
+  since Rust 1.89; the workspace sets no `rust-version`) for the whole of `run` and `resume`,
+  prompts included, for every plan of the repository. The operating system releases it when the
+  process ends, however it ends: there is no stale lock. The holder's process id goes into
+  `lock.pid` for the message only (on Windows another process cannot read a locked file).
+  `status` and `--dry-run` take no lock. Its limit: on Unix, after a SIGKILL of planrun, a
+  session's descendants can outlive the lock (D73).
+
+## D78 Step transactions in a real run
+
+Amends D53. Settles the P3 open items on the gap between prepare and commit, file modes, a git
+delete of an untracked file, and the writer's temp files.
+
+- **The order inside commit:** check the disk again, then the write-ahead record (synced,
+  D76), then the writes. Each path is stat'ed first (`hash-object --stdin-paths` fails the whole
+  batch on a missing file), then the present ones are hashed in one call and compared with the
+  pre-images; the directories the step will create must still be absent. On any difference
+  nothing is recorded or written, the paths are shown, and the user chooses `[p]` prepare again
+  or `[q]` quit. planrun never prepares again silently: something else is writing the tree, and
+  an Overwrite prepared again would clobber it. Nothing that prompts lies between the check and
+  the writes.
+- **Modes:** the post-image's mode is the virtual tree's index entry after prepare (a patch's
+  mode lines included), journaled in `step-prepared`, and set on the temp file before the
+  rename when `core.filemode` is true (unset: true on Unix). Windows has no executable bit.
+  Rollback and restore put the pre-image's mode back; Done compares the mode too.
+- **Ids for resume:** `step-prepared` holds each post-image's exact-bytes id (`--no-filters`,
+  not stored) beside its filtered blob id, so Done and NotDone compare like with like.
+- **Temp names** are chosen at prepare and journaled per path; resume removes exactly those
+  (D77).
+- **A git delete of an untracked file** (`Delete` with `via_git`) is `delete-untracked`, an
+  error of a real run's preflight: the plan's own `git rm` would refuse it. The dry run keeps
+  simulating it (P3's stated decision 16).
+
+## D79 When a commit counts as made
+
+A Commit is made only when HEAD moved by exactly one commit on the HEAD before it and that
+commit's tree holds the task's paths as they were on disk when the Commit started: its file
+actions, `Generated` paths and manifest file entries, journaled in the Commit's `action-start`
+as git's blob ids (absent paths absent). This is not the journaled post-images of the task's
+steps: a script between a step and the commit may change a written file (`cargo fmt`, D55),
+and a `Generated` path has no post-image at all. A commit script that exits 0 without making
+that commit (a gated script with `$gate` unset) is a mismatch, not done; resume applies the
+same test to a Commit that was running (D77).
+
+## D80 The repository around a commit
+
+- **HEAD and the ref it points to** are recorded after every script, every Manual, every `[o]`
+  and every Commit, not only at a task's end: any of them can move HEAD (a commit by hand, D33,
+  or a Modify script with `git commit`).
+- **Before every Commit**, one `status` over the tree: dirt outside the task's paths (its file
+  actions, manifest entries, patterns included, and `Generated` paths) is listed and asked about,
+  whatever `--auto` says. A commit script that stages everything would take it.
+- **After the Commit**, `git diff-tree -r --name-only` between the HEAD before and after: a
+  path outside the task's paths is reported loudly.
+- **After a Manual** that moved HEAD (a commit by hand), the task's paths that still have
+  changes are reported.
+
+## D81 Sessions within a run
+
+Amends D61.
+
+- **A dead session** (`SessionEnded`, `TimedOut`, `Broken`) is a mismatch whose answers are
+  retry, shell and abort (D75). Retry starts a new session; nothing restarts one silently.
+- **The session notice:** before the first block of a new session that is not the run's first
+  (a restart), of a resume, and of a run with `--from`, planrun says that the session is new and
+  that variables, functions and the working directory of earlier blocks are gone, and lists the
+  earlier tasks whose scripts ran in another session. It must be acknowledged even with
+  `--auto`.
+- **A working directory outside the repository** (compared canonically; a pwsh provider path
+  that is no file-system path counts as outside) asks before each next block of that session,
+  whatever `--auto` says: D68 classes a relative redirection as Modify on the assumption that
+  the directory is inside the repository. The block's decision is unchanged.
+
+## D82 The end-to-end tests drive the binary
+
+planrun-cli has a test-only feature, `e2e`. Its end-to-end test target (`tests/run.rs`) has
+`required-features = ["e2e"]`, and every plan's gate runs `cargo clippy -p planrun-cli
+--all-targets --features e2e -- -D warnings` and `cargo test -p planrun-cli --features e2e
+--test run` (with `PLANRUN_REQUIRE_SHELLS=1`). No release build compiles it. Under the feature
+only, prompts read answers from a stdin that is not a terminal, and `PLANRUN_E2E_ABORT=<seam>@<task>`
+ends the process right after a seam: `step-prepared` (no write yet), `write<k>` (after a step's
+k-th write), `commit-start` (after a Commit's `action-start`). It ends with `exit(134)`, not
+`abort()`: no destructor runs, as in a kill, and on Windows no Windows Error Reporting is raised
+for the test machine to answer. A kill of `git commit` itself (HEAD moved, `action-end` missing)
+is tested in planrun-exec, where the observer can make the commit before it crashes.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t1-decisions-md.patch"
Remove-Item "$env:TEMP\p5-t1-decisions-md.patch"
```

**Step 2 — verify**

Targeted: the model's tests (nothing but docs changes):

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
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- docs/decisions.md
  $verified = [ordered]@{
      'docs/decisions.md' = '1fe6fb71178dc65a913bfa851a15b227f17e7e52'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P5-T1: decisions D71-D82: sessions, sed, interrupts, prompts, the commit gate, the journal, resume, transactions, commits, the e2e feature" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P5-T2: sessions end when the shell is gone, wrapper commands through builtin, RemoteSigned, PDEATHSIG and interrupts

**Goal:** The P4 review items the runner needs from its sessions (D71) and the interrupt handlers (D73). `Session::collect` waits in 100 ms slices and kills the tree when the shell itself exited while a descendant holds its pipes (`Tree::exited`: `waitid` with `WNOWAIT` on Linux, a zero wait on Windows); a block whose file cannot be written ends the session; the bash wrapper calls `source`, `local`, `printf` and `pwd` through `builtin`; pwsh runs with `-ExecutionPolicy RemoteSigned`; on Linux the shell gets `PR_SET_PDEATHSIG` with the `getppid` check. `install_interrupt_handler` and `InterruptsIgnored` are the safe surface of the new handlers in `kill.rs`.

**Files**

- Modify `crates/planrun-exec/src/shell/bash.rs`
- Modify `crates/planrun-exec/src/shell/kill.rs`
- Modify `crates/planrun-exec/src/shell/mod.rs`
- Modify `crates/planrun-exec/src/shell/pwsh.rs`
- Modify `crates/planrun-exec/src/shell/session.rs`
- Modify `crates/planrun-exec/tests/shell.rs`

Over 300 lines, so they come as exact diffs: `crates/planrun-exec/src/shell/kill.rs`, `crates/planrun-exec/src/shell/session.rs`, `crates/planrun-exec/tests/shell.rs`.

**Step 1 — write the files**

File `crates/planrun-exec/src/shell/bash.rs` (full contents):

```rust
//! bash (decisions D60, D61, D63): `bash --noprofile --norc`, one top-level line per block on
//! stdin, the block `source`d from its file with stdin from `/dev/null`. On Windows, Git for
//! Windows' bash only, found from git itself, never from `PATH`.

use std::path::PathBuf;
use std::process::Command;

use super::ShellError;
use super::kill::detach;
use super::session::{DIR_VAR, Dialect};

/// The bash dialect.
pub(super) struct Bash;

fn unavailable(reason: String) -> ShellError {
    ShellError::Unavailable {
        shell: "bash",
        reason,
    }
}

/// Windows: `<root>\usr\bin\bash.exe`, where `<root>` is the first of `git --exec-path` and its
/// parents (at most four levels up) that has one; checked with `uname -s`, which must report
/// MINGW or MSYS. Elsewhere: `bash` from `PATH`, checked with `$BASH_VERSION`.
pub(super) fn locate() -> Result<PathBuf, ShellError> {
    let program = if cfg!(windows) {
        git_bash()?
    } else {
        PathBuf::from("bash")
    };
    let check = if cfg!(windows) {
        "PATH=/usr/bin:$PATH uname -s"
    } else {
        "echo \"$BASH_VERSION\""
    };
    let mut probe = Command::new(&program);
    probe.args(["--noprofile", "--norc", "-c", check]);
    detach(&mut probe);
    let out = probe
        .output()
        .map_err(|e| unavailable(format!("{} did not start: {e}", program.display())))?;
    let said = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let accepted = if cfg!(windows) {
        said.starts_with("MINGW") || said.starts_with("MSYS")
    } else {
        !said.is_empty()
    };
    if !out.status.success() || !accepted {
        return Err(unavailable(format!(
            "{} answered {said:?} to `{check}`",
            program.display()
        )));
    }
    Ok(program)
}

fn git_bash() -> Result<PathBuf, ShellError> {
    let out = Command::new("git")
        .arg("--exec-path")
        .output()
        .map_err(|e| unavailable(format!("`git --exec-path` did not run: {e}")))?;
    let exec_path = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    exec_path
        .ancestors()
        .take(5)
        .map(|dir| dir.join("usr").join("bin").join("bash.exe"))
        .find(|bash| bash.is_file())
        .ok_or_else(|| {
            unavailable(format!(
                "no usr\\bin\\bash.exe at or above {}",
                exec_path.display()
            ))
        })
}

impl Dialect for Bash {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn args(&self, _marker: &str) -> Vec<String> {
        vec!["--noprofile".to_owned(), "--norc".to_owned()]
    }

    fn extension(&self) -> &'static str {
        "sh"
    }

    /// On Windows `/usr/bin` is prepended to `PATH`, as `/etc/profile` would (only that
    /// directory: `git` keeps resolving through the Windows `PATH`, as planrun's own does), and
    /// the working directory is `pwd -W`: `$PWD` is an MSYS path, and `/tmp` is a mount.
    fn prologue(&self, marker: &str) -> Option<String> {
        let (path, pwd) = if cfg!(windows) {
            ("PATH=/usr/bin:$PATH; ", "$(builtin pwd -W)")
        } else {
            ("", "$PWD")
        };
        Some(format!(
            "__planrun_dir=${DIR_VAR}; unset {DIR_VAR}; {path}\
             __planrun_hex() {{ builtin local LC_ALL=C __planrun_i __planrun_c; __planrun_h=; \
             for ((__planrun_i = 0; __planrun_i < ${{#1}}; __planrun_i++)); do \
             builtin printf -v __planrun_c '%02x' \"'${{1:__planrun_i:1}}\"; \
             __planrun_h+=$__planrun_c; done; }}; \
             __planrun_end() {{ __planrun_hex \"{pwd}\"; \
             builtin printf '%s\\n' \"{marker}${{__planrun_n}}__ $1 $__planrun_s 0 $__planrun_h\"; \
             builtin printf '%s\\n' \"{marker}${{__planrun_n}}__\" >&2; }}; \
             trap '__planrun_s=$?; __planrun_end exit' EXIT; \
             __planrun_n=0; __planrun_s=0; __planrun_end ok"
        ))
    }

    fn block_line(&self, n: u64) -> Option<String> {
        Some(format!(
            "__planrun_n={n}; builtin source \"$__planrun_dir/b{n}.sh\" </dev/null; __planrun_s=$?; \
             __planrun_end ok"
        ))
    }

    /// On Windows `pwd -W` reports `C:/x`: its slashes become backslashes.
    fn cwd(&self, reported: String) -> PathBuf {
        if cfg!(windows) {
            return PathBuf::from(reported.replace('/', "\\"));
        }
        PathBuf::from(reported)
    }
}
```

File `crates/planrun-exec/src/shell/kill.rs` (exact diff; save the block as `$env:TEMP\p5-t2-kill-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/shell/kill.rs b/crates/planrun-exec/src/shell/kill.rs
index 7d230d8..a3a49d6 100644
--- a/crates/planrun-exec/src/shell/kill.rs
+++ b/crates/planrun-exec/src/shell/kill.rs
@@ -1,12 +1,15 @@
-//! The process tree of a shell session (decision D64). Every `unsafe` call of planrun lives in
-//! this module: the job object on Windows, `setsid` and `kill` on Unix, each declared by hand.
-//! The rest of the crate sees three safe operations: [`Tree::spawn`], [`Tree::kill`] and
-//! [`Tree::wait`].
+//! The process tree of a shell session (decisions D64, D71, D73). Every `unsafe` call of
+//! planrun lives in this module: the job object, the handle wait and the console control
+//! handler on Windows; `setsid`, `prctl`, `getppid`, `waitid`, `kill`, `signal` and `raise` on
+//! Unix, each declared by hand. The rest of the crate sees safe operations: [`Tree::spawn`],
+//! [`Tree::exited`], [`Tree::kill`], [`Tree::wait`], [`install_interrupts`] and
+//! [`ignore_interrupts`].
 
 #![allow(unsafe_code)]
 
 use std::io;
 use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
+use std::sync::atomic::{AtomicBool, Ordering};
 
 /// The three pipes of a spawned session.
 pub(crate) struct Pipes {
@@ -27,11 +30,15 @@ pub(crate) struct Tree {
     job: windows::Job,
 }
 
+/// While set, Ctrl-C does not end planrun: the `[o]` shell runs in the foreground (D73).
+static IGNORE_INTERRUPT: AtomicBool = AtomicBool::new(false);
+
 impl Tree {
-    /// Spawns `command` with piped stdout and stderr, stdin piped or null, detached from planrun's terminal
-    /// (Windows: a console of its own; Unix: a session of its own), and puts it in a tree. When
-    /// the tree cannot be made (Windows: a job that refuses nesting), the process is killed and
-    /// the spawn fails: there is no session without the tree kill.
+    /// Spawns `command` with piped stdout and stderr, stdin piped or null, detached from
+    /// planrun's terminal (Windows: a console of its own; Unix: a session of its own, killed
+    /// when planrun dies on Linux), and puts it in a tree. When the tree cannot be made
+    /// (Windows: a job that refuses nesting), the process is killed and the spawn fails: there
+    /// is no session without the tree kill.
     pub(crate) fn spawn(command: &mut Command, stdin: bool) -> io::Result<(Tree, Pipes)> {
         let input = if stdin { Stdio::piped() } else { Stdio::null() };
         command
@@ -48,6 +55,8 @@ impl Tree {
             let _ = child.wait();
             return Err(e);
         }
+        #[cfg(unix)]
+        unix::register(child.id());
         let input = child.stdin.take();
         let pipes = match (input, child.stdout.take(), child.stderr.take()) {
             (input, Some(stdout), Some(stderr)) if input.is_some() == stdin => Pipes {
@@ -57,6 +66,8 @@ impl Tree {
             },
             _ => {
                 let _ = child.kill();
+                #[cfg(unix)]
+                unix::unregister(child.id());
                 let _ = child.wait();
                 return Err(io::Error::other("the session's pipes were not created"));
             }
@@ -70,6 +81,23 @@ impl Tree {
         Ok((tree, pipes))
     }
 
+    /// True when the process itself has exited, without reaping it: on Linux `waitid` with
+    /// `WNOWAIT`, so its process group id stays reserved for [`Tree::kill`] (D64); on Windows a
+    /// zero wait on the handle the tree keeps open. Other Unix systems always answer false
+    /// (D19).
+    pub(crate) fn exited(&mut self) -> io::Result<bool> {
+        if self.reaped.is_some() {
+            return Ok(true);
+        }
+        #[cfg(windows)]
+        let exited = windows::exited(&self.child);
+        #[cfg(target_os = "linux")]
+        let exited = unix::exited(self.child.id());
+        #[cfg(all(unix, not(target_os = "linux")))]
+        let exited = Ok(false);
+        exited
+    }
+
     /// Kills the process and all its descendants. Does nothing once the process was waited on:
     /// on Unix its process group id could then belong to someone else.
     pub(crate) fn kill(&mut self) -> io::Result<()> {
@@ -86,11 +114,15 @@ impl Tree {
         killed
     }
 
-    /// Waits for the process itself (not its descendants) and remembers its status.
+    /// Waits for the process itself (not its descendants) and remembers its status. On Unix its
+    /// group leaves the interrupt handler's list first, so the handler never signals a group id
+    /// that may be reused.
     pub(crate) fn wait(&mut self) -> io::Result<ExitStatus> {
         if let Some(status) = self.reaped {
             return Ok(status);
         }
+        #[cfg(unix)]
+        unix::unregister(self.child.id());
         let status = self.child.wait()?;
         self.reaped = Some(status);
         Ok(status)
@@ -106,7 +138,8 @@ impl Drop for Tree {
 
 /// Detaches a command from planrun's terminal: on Windows `CREATE_NO_WINDOW`, so the session
 /// gets a console of its own and its code page changes stay there; on Unix `setsid`, so a read
-/// from `/dev/tty` fails instead of stopping the process group.
+/// from `/dev/tty` fails instead of stopping the process group, and on Linux
+/// `PR_SET_PDEATHSIG`, so the shell dies with planrun (D71).
 pub(crate) fn detach(command: &mut Command) {
     #[cfg(windows)]
     {
@@ -118,30 +151,118 @@ pub(crate) fn detach(command: &mut Command) {
     unix::setsid_on_exec(command);
 }
 
+/// Installs planrun's interrupt handling (D73). Unix: SIGINT, SIGTERM and SIGHUP kill every
+/// session's process group, then end planrun as the signal would. Windows: a console control
+/// handler that only swallows Ctrl-C while [`ignore_interrupts`] is on; otherwise Windows ends
+/// planrun and the job objects kill the sessions. Call it once, from the main thread.
+pub(crate) fn install_interrupts() {
+    #[cfg(unix)]
+    unix::install();
+    #[cfg(windows)]
+    windows::install();
+}
+
+/// Ctrl-C leaves planrun alone while `on` (the `[o]` shell runs in the foreground).
+pub(crate) fn ignore_interrupts(on: bool) {
+    IGNORE_INTERRUPT.store(on, Ordering::SeqCst);
+}
+
 #[cfg(unix)]
 mod unix {
     use std::io;
     use std::os::unix::process::CommandExt;
     use std::process::Command;
+    use std::sync::atomic::{AtomicI32, Ordering};
 
+    use super::IGNORE_INTERRUPT;
+
+    const SIGHUP: i32 = 1;
+    const SIGINT: i32 = 2;
     const SIGKILL: i32 = 9;
+    const SIGTERM: i32 = 15;
     const ESRCH: i32 = 3;
+    /// `SIG_DFL`.
+    const DEFAULT: usize = 0;
 
     unsafe extern "C" {
         fn setsid() -> i32;
         fn kill(pid: i32, signal: i32) -> i32;
+        fn signal(signum: i32, handler: usize) -> usize;
+        fn raise(signum: i32) -> i32;
+    }
+
+    /// The process groups of the live sessions, for the signal handler; 0 is a free slot.
+    static GROUPS: [AtomicI32; 8] = [const { AtomicI32::new(0) }; 8];
+
+    pub(super) fn register(pid: u32) {
+        let Ok(pid) = i32::try_from(pid) else {
+            return;
+        };
+        for slot in &GROUPS {
+            if slot
+                .compare_exchange(0, pid, Ordering::SeqCst, Ordering::SeqCst)
+                .is_ok()
+            {
+                return;
+            }
+        }
+    }
+
+    pub(super) fn unregister(pid: u32) {
+        let Ok(pid) = i32::try_from(pid) else {
+            return;
+        };
+        for slot in &GROUPS {
+            let _ = slot.compare_exchange(pid, 0, Ordering::SeqCst, Ordering::SeqCst);
+        }
+    }
+
+    /// Kills every registered group, then ends planrun the way `signum` would. Only
+    /// async-signal-safe calls: atomics, `kill`, `signal`, `raise`.
+    extern "C" fn on_signal(signum: i32) {
+        if signum == SIGINT && IGNORE_INTERRUPT.load(Ordering::SeqCst) {
+            return;
+        }
+        for slot in &GROUPS {
+            let pid = slot.load(Ordering::SeqCst);
+            if pid > 0 {
+                // SAFETY: `kill` takes plain integers; the slot is cleared before its leader
+                // is reaped, so the group id is still the session's.
+                unsafe { kill(-pid, SIGKILL) };
+            }
+        }
+        // SAFETY: restoring the default action and raising the signal again ends the process
+        // as the signal would have; both calls are async-signal-safe.
+        unsafe {
+            signal(signum, DEFAULT);
+            raise(signum);
+        }
+    }
+
+    pub(super) fn install() {
+        let handler = on_signal as extern "C" fn(i32) as usize;
+        for signum in [SIGINT, SIGTERM, SIGHUP] {
+            // SAFETY: `handler` is an `extern "C" fn(i32)` that lives for the whole program and
+            // does only async-signal-safe work.
+            unsafe { signal(signum, handler) };
+        }
     }
 
     pub(super) fn setsid_on_exec(command: &mut Command) {
+        let parent = std::process::id();
         // SAFETY: the closure runs in the forked child before exec. It only calls `setsid`,
-        // which is async-signal-safe, and builds an `io::Error` from errno, which allocates
-        // nothing; it touches no lock or memory shared with the parent.
+        // `prctl` and `getppid`, which are async-signal-safe, and builds an `io::Error` from
+        // errno, which allocates nothing; it touches no lock or memory shared with the parent.
         unsafe {
-            command.pre_exec(|| {
+            command.pre_exec(move || {
                 // SAFETY: `setsid` takes no arguments and only changes the calling process.
                 if setsid() == -1 {
                     return Err(io::Error::last_os_error());
                 }
+                #[cfg(target_os = "linux")]
+                linux::die_with_parent(parent)?;
+                #[cfg(not(target_os = "linux"))]
+                let _ = parent;
                 Ok(())
             });
         }
@@ -164,6 +285,76 @@ mod unix {
         }
         Err(e)
     }
+
+    #[cfg(target_os = "linux")]
+    pub(super) fn exited(pid: u32) -> io::Result<bool> {
+        linux::exited(pid)
+    }
+
+    #[cfg(target_os = "linux")]
+    mod linux {
+        use std::io;
+
+        const PR_SET_PDEATHSIG: i32 = 1;
+        const P_PID: i32 = 1;
+        const WNOHANG: i32 = 1;
+        const WEXITED: i32 = 4;
+        const WNOWAIT: i32 = 0x0100_0000;
+
+        /// `siginfo_t` is 128 bytes; only `si_signo`, its first field, is read.
+        #[repr(C, align(8))]
+        struct SigInfo {
+            signo: i32,
+            rest: [i32; 31],
+        }
+
+        unsafe extern "C" {
+            fn prctl(option: i32, ...) -> i32;
+            fn getppid() -> i32;
+            fn waitid(idtype: i32, id: u32, info: *mut SigInfo, options: i32) -> i32;
+        }
+
+        /// In the forked child: SIGKILL when the parent dies, and refuse to start when it
+        /// already has (the parent then is no longer `parent`). The signal follows the thread
+        /// that spawned the child, so sessions are spawned from planrun's main thread.
+        pub(super) fn die_with_parent(parent: u32) -> io::Result<()> {
+            // SAFETY: `prctl(PR_SET_PDEATHSIG, sig)` takes the signal as an unsigned long and
+            // changes only the calling process.
+            if unsafe { prctl(PR_SET_PDEATHSIG, super::SIGKILL as u64) } == -1 {
+                return Err(io::Error::last_os_error());
+            }
+            // SAFETY: `getppid` takes no arguments and cannot fail.
+            let now = unsafe { getppid() };
+            if u32::try_from(now).ok() != Some(parent) {
+                return Err(io::Error::from_raw_os_error(super::ESRCH));
+            }
+            Ok(())
+        }
+
+        /// Whether `pid` has exited, leaving it a zombie (`WNOWAIT`): the kernel sets
+        /// `si_signo` to SIGCHLD when it has and to 0 when it has not.
+        pub(super) fn exited(pid: u32) -> io::Result<bool> {
+            let mut info = SigInfo {
+                signo: 0,
+                rest: [0; 31],
+            };
+            // SAFETY: `info` is a writable, 8-aligned, 128-byte `siginfo_t`; the flags make the
+            // call return at once and leave the child unreaped.
+            let r = unsafe { waitid(P_PID, pid, &raw mut info, WEXITED | WNOHANG | WNOWAIT) };
+            if r == -1 {
+                return Err(io::Error::last_os_error());
+            }
+            Ok(info.signo != 0)
+        }
+
+        #[cfg(test)]
+        mod tests {
+            #[test]
+            fn siginfo_is_128_bytes() {
+                assert_eq!(size_of::<super::SigInfo>(), 128);
+            }
+        }
+    }
 }
 
 #[cfg(windows)]
@@ -172,11 +363,18 @@ mod windows {
     use std::io;
     use std::os::windows::io::AsRawHandle;
     use std::process::Child;
+    use std::sync::atomic::Ordering;
+
+    use super::IGNORE_INTERRUPT;
 
     type Handle = *mut c_void;
 
     const EXTENDED_LIMIT_INFORMATION: i32 = 9;
     pub(super) const KILL_ON_JOB_CLOSE: u32 = 0x2000;
+    const WAIT_OBJECT_0: u32 = 0;
+    const WAIT_FAILED: u32 = 0xFFFF_FFFF;
+    const CTRL_C_EVENT: u32 = 0;
+    const CTRL_BREAK_EVENT: u32 = 1;
 
     #[repr(C)]
     #[derive(Default)]
@@ -215,6 +413,8 @@ mod windows {
         peak_job_memory_used: usize,
     }
 
+    type CtrlHandler = unsafe extern "system" fn(u32) -> i32;
+
     #[link(name = "kernel32")]
     unsafe extern "system" {
         fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> Handle;
@@ -229,6 +429,8 @@ mod windows {
         fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
         fn TerminateJobObject(job: Handle, exit_code: u32) -> i32;
         fn CloseHandle(handle: Handle) -> i32;
+        fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
+        fn SetConsoleCtrlHandler(handler: Option<CtrlHandler>, add: i32) -> i32;
     }
 
     /// A job object that kills its processes when its last handle closes.
@@ -327,6 +529,31 @@ mod windows {
         }
     }
 
+    /// Whether `child` has exited: a zero-timeout wait on the process handle `Child` owns.
+    pub(super) fn exited(child: &Child) -> io::Result<bool> {
+        // SAFETY: the handle is live for as long as `child`; a zero timeout never blocks.
+        match unsafe { WaitForSingleObject(child.as_raw_handle().cast(), 0) } {
+            WAIT_OBJECT_0 => Ok(true),
+            WAIT_FAILED => Err(io::Error::last_os_error()),
+            _ => Ok(false),
+        }
+    }
+
+    /// Swallows Ctrl-C and Ctrl-Break while the `[o]` shell runs; otherwise passes them on to
+    /// the default handler, which ends planrun. A handler routine, not `SetConsoleCtrlHandler`
+    /// with a null routine: that flag is inherited by child processes, so the shell and its
+    /// commands would ignore Ctrl-C too.
+    unsafe extern "system" fn on_ctrl(kind: u32) -> i32 {
+        let interrupt = kind == CTRL_C_EVENT || kind == CTRL_BREAK_EVENT;
+        i32::from(interrupt && IGNORE_INTERRUPT.load(Ordering::SeqCst))
+    }
+
+    pub(super) fn install() {
+        // SAFETY: `on_ctrl` is a `system` function that lives for the whole program and only
+        // reads an atomic.
+        unsafe { SetConsoleCtrlHandler(Some(on_ctrl), 1) };
+    }
+
     #[cfg(test)]
     mod tests {
         use super::*;
@@ -343,6 +570,17 @@ mod windows {
             let flags = job.limit_flags().unwrap_or_else(|e| panic!("{e}"));
             assert_ne!(flags & KILL_ON_JOB_CLOSE, 0, "limit flags {flags:#x}");
         }
+
+        #[test]
+        fn the_control_handler_swallows_ctrl_c_only_while_ignoring() {
+            super::super::ignore_interrupts(true);
+            // SAFETY: calling the handler directly only reads the atomic.
+            let (c, close) = unsafe { (on_ctrl(CTRL_C_EVENT), on_ctrl(2)) };
+            super::super::ignore_interrupts(false);
+            // SAFETY: as above.
+            let after = unsafe { on_ctrl(CTRL_C_EVENT) };
+            assert_eq!((c, close, after), (1, 0, 0));
+        }
     }
 }
 
@@ -360,7 +598,15 @@ mod tests {
         command.args(["cat-file", "--batch"]);
         let (mut tree, pipes) = Tree::spawn(&mut command, true).unwrap_or_else(|e| panic!("{e}"));
         let started = Instant::now();
+        assert!(!tree.exited().unwrap_or_else(|e| panic!("{e}")));
         tree.kill().unwrap_or_else(|e| panic!("{e}"));
+        let until = Instant::now() + Duration::from_secs(10);
+        while !tree.exited().unwrap_or_else(|e| panic!("{e}")) && Instant::now() < until {
+            std::thread::sleep(Duration::from_millis(20));
+        }
+        if cfg!(any(windows, target_os = "linux")) {
+            assert!(tree.exited().unwrap_or_else(|e| panic!("{e}")));
+        }
         let status = tree.wait().unwrap_or_else(|e| panic!("{e}"));
         assert!(!status.success(), "{status}");
         assert!(started.elapsed() < Duration::from_secs(10));
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t2-kill-rs.patch"
Remove-Item "$env:TEMP\p5-t2-kill-rs.patch"
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

File `crates/planrun-exec/src/shell/pwsh.rs` (full contents):

```rust
//! PowerShell 7 (decisions D60, D61, D63). A native command started by pwsh inherits pwsh's
//! own stdin, so the session gets none (`git hash-object --stdin` would read the protocol):
//! its `-Command` script is the prologue and a loop at the top level that waits for the next
//! block file in the run directory and dot-sources it inside `do`/`try`/`catch`.

use std::path::PathBuf;
use std::process::Command;

use super::ShellError;
use super::kill::detach;
use super::session::{DIR_VAR, Dialect};

/// The pwsh dialect.
pub(super) struct Pwsh;

/// `pwsh` from `PATH`, when it is PowerShell 7 or later; Windows PowerShell 5.1 is
/// `powershell.exe` and never used.
pub(super) fn locate() -> Result<PathBuf, ShellError> {
    let unavailable = |reason: String| ShellError::Unavailable {
        shell: "pwsh",
        reason,
    };
    let mut probe = Command::new("pwsh");
    probe.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$PSVersionTable.PSVersion.Major",
    ]);
    detach(&mut probe);
    let out = probe
        .output()
        .map_err(|e| unavailable(format!("`pwsh` did not start: {e}")))?;
    let major = String::from_utf8_lossy(&out.stdout).trim().parse::<u32>();
    match major {
        Ok(major) if out.status.success() && major >= 7 => Ok(PathBuf::from("pwsh")),
        Ok(major) => Err(unavailable(format!("PowerShell {major} is older than 7"))),
        Err(_) => Err(unavailable("`pwsh` did not report its version".to_owned())),
    }
}

impl Dialect for Pwsh {
    fn name(&self) -> &'static str {
        "pwsh"
    }

    fn args(&self, marker: &str) -> Vec<String> {
        let mut args = vec!["-NoProfile", "-NonInteractive", "-NoLogo"];
        if cfg!(windows) {
            args.extend(["-ExecutionPolicy", "RemoteSigned"]);
        }
        args.push("-Command");
        let mut args: Vec<String> = args.into_iter().map(str::to_owned).collect();
        args.push(script(marker));
        args
    }

    fn extension(&self) -> &'static str {
        "ps1"
    }

    fn prologue(&self, _marker: &str) -> Option<String> {
        None
    }

    fn block_line(&self, _n: u64) -> Option<String> {
        None
    }

    fn cwd(&self, reported: String) -> PathBuf {
        PathBuf::from(reported)
    }
}

/// The session's whole `-Command` script: the prologue, the sentinel of block 0, then the loop.
/// It has no double quotes, so the command line needs no escaping beyond Windows' own.
fn script(marker: &str) -> String {
    [
        format!("$global:__planrun_dir = $env:{DIR_VAR}"),
        format!("Remove-Item Env:{DIR_VAR}"),
        "if ($PSStyle) { $PSStyle.OutputRendering = 'PlainText' }".to_owned(),
        "$global:ProgressPreference = 'SilentlyContinue'".to_owned(),
        "$global:ErrorActionPreference = 'Continue'".to_owned(),
        "$global:PSNativeCommandUseErrorActionPreference = $false".to_owned(),
        "[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)".to_owned(),
        "$global:OutputEncoding = [Text.UTF8Encoding]::new($false)".to_owned(),
        end_function(marker),
        "$global:__planrun_n = 0".to_owned(),
        "$global:__planrun_w = [IO.FileSystemWatcher]::new($global:__planrun_dir, 'b*.ps1')"
            .to_owned(),
        RESET.to_owned(),
        "__planrun_end 0".to_owned(),
        run_loop(),
    ]
    .join("; ")
}

/// Resets every wrapper variable and `$LASTEXITCODE` before a block (D61).
const RESET: &str =
    "$global:__planrun_t = 0; $global:__planrun_e = $global:Error[0]; $global:LASTEXITCODE = 0";

/// Waits for `b<n>.ps1` (renamed into place whole), runs it in the global scope, and prints its
/// sentinel. `do … while ($false)` keeps a `break` or `continue` of the block inside it.
fn run_loop() -> String {
    format!(
        "while ($true) {{ $global:__planrun_n++; \
         $global:__planrun_f = [IO.Path]::Combine($global:__planrun_dir, 'b' + $global:__planrun_n + '.ps1'); \
         while (-not [IO.File]::Exists($global:__planrun_f)) {{ \
         [void]$global:__planrun_w.WaitForChanged([IO.WatcherChangeTypes]::Renamed, 250) }}; \
         {RESET}; \
         do {{ try {{ . $global:__planrun_f }} catch {{ $global:__planrun_t = 1; \
         [Console]::Error.WriteLine(($_ | Out-String).TrimEnd()) }} }} while ($false); \
         __planrun_end $global:__planrun_n }}"
    )
}

/// `__planrun_end <n>`: the sentinel on stdout (kind, status, errored, cwd as hex of UTF-8) and
/// on stderr, each written with LF and flushed.
fn end_function(marker: &str) -> String {
    format!(
        "function global:__planrun_end([long]$n) {{ \
         $s = $global:LASTEXITCODE; if ($null -eq $s) {{ $s = 0 }}; \
         $k = if ($global:__planrun_t) {{ 'threw' }} else {{ 'ok' }}; \
         $e = if ([object]::ReferenceEquals($global:__planrun_e, $global:Error[0])) {{ 0 }} else {{ 1 }}; \
         $c = [BitConverter]::ToString([Text.Encoding]::UTF8.GetBytes($PWD.ProviderPath)).Replace('-', ''); \
         $m = '{marker}' + $n + '__'; \
         [Console]::Out.Write($m + ' ' + $k + ' ' + $s + ' ' + $e + ' ' + $c + [char]10); [Console]::Out.Flush(); \
         [Console]::Error.Write($m + [char]10); [Console]::Error.Flush() }}"
    )
}
```

File `crates/planrun-exec/src/shell/session.rs` (exact diff; save the block as `$env:TEMP\p5-t2-session-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/shell/session.rs b/crates/planrun-exec/src/shell/session.rs
index 0800169..da0714f 100644
--- a/crates/planrun-exec/src/shell/session.rs
+++ b/crates/planrun-exec/src/shell/session.rs
@@ -80,8 +80,13 @@ pub struct Session {
     block: u64,
     timeout: Option<Duration>,
     alive: bool,
+    /// The shell itself has exited and its tree was killed; the pipes close soon (D71).
+    gone: bool,
 }
 
+/// How often `collect` checks whether the shell itself is still there (D71).
+const SLICE: Duration = Duration::from_millis(100);
+
 enum Event {
     Line {
         stream: Stream,
@@ -149,6 +154,7 @@ impl Session {
             block: 0,
             timeout: config.timeout,
             alive: true,
+            gone: false,
         };
         session.handshake(prologue)?;
         Ok(session)
@@ -252,15 +258,26 @@ impl Session {
         Ok(c)
     }
 
-    fn next(&self, deadline: Option<Instant>) -> Result<Event, RecvTimeoutError> {
-        let Some(deadline) = deadline else {
-            return self
-                .events
-                .recv()
-                .map_err(|_| RecvTimeoutError::Disconnected);
-        };
-        let left = deadline.saturating_duration_since(Instant::now());
-        self.events.recv_timeout(left)
+    /// The next event, in slices: when the shell itself has exited without its sentinel while
+    /// a descendant still holds its pipes, the tree is killed so that the pipes close, and the
+    /// block ends as `SessionEnded(None)` (D71).
+    fn next(&mut self, deadline: Option<Instant>) -> Result<Event, RecvTimeoutError> {
+        loop {
+            let wait = deadline.map_or(SLICE, |d| {
+                d.saturating_duration_since(Instant::now()).min(SLICE)
+            });
+            match self.events.recv_timeout(wait) {
+                Err(RecvTimeoutError::Timeout) => {}
+                other => return other,
+            }
+            if deadline.is_some_and(|d| Instant::now() >= d) {
+                return Err(RecvTimeoutError::Timeout);
+            }
+            if !self.gone && self.tree.exited().unwrap_or(false) {
+                self.gone = true;
+                let _ = self.tree.kill();
+            }
+        }
     }
 
     /// Parses what follows the marker: `<n>__`, then on stdout ` <kind> <status> <errored>
@@ -358,6 +375,23 @@ impl ShellSession for Session {
             )));
         }
         self.block += 1;
+        if let Err(e) = self.stage(script) {
+            self.end(Vec::new(), RunStatus::SessionEnded(None));
+            let block = self.block;
+            return Err(io::Error::new(
+                e.kind(),
+                format!("block {block} could not be written, so the session was ended: {e}"),
+            ));
+        }
+        let line = self.dialect.block_line(self.block);
+        self.send(line, on_line)
+    }
+}
+
+impl Session {
+    /// Writes the current block's file: `b<n>.tmp`, then renamed whole. An error leaves the
+    /// shell waiting for a block that never comes, so the caller ends the session (D71).
+    fn stage(&self, script: &str) -> io::Result<()> {
         let staged = self.dir.path().join(format!("b{}.tmp", self.block));
         let mut file = OpenOptions::new()
             .write(true)
@@ -366,9 +400,7 @@ impl ShellSession for Session {
         file.write_all(script.as_bytes())?;
         drop(file);
         let name = format!("b{}.{}", self.block, self.dialect.extension());
-        std::fs::rename(&staged, self.dir.path().join(name))?;
-        let line = self.dialect.block_line(self.block);
-        self.send(line, on_line)
+        std::fs::rename(&staged, self.dir.path().join(name))
     }
 }
 
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t2-session-rs.patch"
Remove-Item "$env:TEMP\p5-t2-session-rs.patch"
```

File `crates/planrun-exec/tests/shell.rs` (exact diff; save the block as `$env:TEMP\p5-t2-shell-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/tests/shell.rs b/crates/planrun-exec/tests/shell.rs
index 6a58c06..1339124 100644
--- a/crates/planrun-exec/tests/shell.rs
+++ b/crates/planrun-exec/tests/shell.rs
@@ -297,6 +297,57 @@ fn bash_exit_set_e_a_replaced_trap_and_exec_end_the_session() {
     }
 }
 
+#[test]
+fn a_shell_that_exits_while_a_descendant_holds_its_pipes_ends_the_session() {
+    let f = Fixture::greeter(false);
+    let pwsh = "$null = [Diagnostics.Process]::Start((Get-Process -Id $PID).Path, '-NoProfile -Command Start-Sleep 120')\n[Environment]::Exit(0)\n";
+    let bash = "sleep 120 &\ntrap - EXIT\nexit 0\n";
+    for (shell, script) in [(Shell::Pwsh, pwsh), (Shell::Bash, bash)] {
+        let Some(mut s) = start(shell, &f, Duration::from_secs(60)) else {
+            continue;
+        };
+        let started = Instant::now();
+        let r = run(&mut s, script);
+        assert_eq!(r.status, RunStatus::SessionEnded(None), "{shell:?}");
+        assert!(started.elapsed() < Duration::from_secs(30), "{shell:?}");
+        assert!(!s.is_alive());
+    }
+}
+
+#[test]
+fn bash_plan_functions_named_like_the_wrapper_s_commands_change_nothing() {
+    let f = Fixture::greeter(false);
+    let Some(mut s) = session(Shell::Bash, &f) else {
+        return;
+    };
+    let define = "source() { echo hijacked; }\nlocal() { echo hijacked; }\nprintf() { echo hijacked; }\npwd() { echo hijacked; }\nv=kept\n";
+    assert_eq!(run(&mut s, define).status, RunStatus::Exited(0));
+    let next = run(&mut s, "echo \"$v\"\n");
+    assert_eq!(next.status, RunStatus::Exited(0));
+    assert_eq!(stdout(&next), ["kept"]);
+    assert!(next.cwd.is_some_and(|c| same_dir(&c, f.path())));
+}
+
+#[test]
+fn a_block_that_cannot_be_written_ends_the_session() {
+    let f = Fixture::greeter(false);
+    let temp = f.outside().join("runs");
+    std::fs::create_dir_all(&temp).unwrap_or_else(|e| panic!("{e}"));
+    for shell in [Shell::Pwsh, Shell::Bash] {
+        let Some(mut s) = start_in(shell, &f, Duration::from_secs(60), temp.clone()) else {
+            continue;
+        };
+        for entry in std::fs::read_dir(&temp)
+            .unwrap_or_else(|e| panic!("{e}"))
+            .flatten()
+        {
+            std::fs::remove_dir_all(entry.path()).unwrap_or_else(|e| panic!("{e}"));
+        }
+        assert!(s.run("echo x\n", &mut |_| {}).is_err(), "{shell:?}");
+        assert!(!s.is_alive(), "{shell:?}");
+    }
+}
+
 #[test]
 fn a_forged_sentinel_breaks_the_session() {
     let f = Fixture::greeter(false);
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t2-shell-rs.patch"
Remove-Item "$env:TEMP\p5-t2-shell-rs.patch"
```

**Step 2 — verify**

Targeted: the shell tests, with both shells required:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-exec --test shell
```

Expected (exit code 0; summary lines):

```text
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-exec/src/shell/bash.rs crates/planrun-exec/src/shell/kill.rs crates/planrun-exec/src/shell/mod.rs crates/planrun-exec/src/shell/pwsh.rs crates/planrun-exec/src/shell/session.rs crates/planrun-exec/tests/shell.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/shell/bash.rs' = 'a33e3c8e8356cf5341f92551a6ea64f82e915632'
      'crates/planrun-exec/src/shell/kill.rs' = 'a3a49d6755cb035f15309c3848047b38d074a210'
      'crates/planrun-exec/src/shell/mod.rs' = 'df88b13f4030f74841552dd9bf62015449b28c39'
      'crates/planrun-exec/src/shell/pwsh.rs' = '979b6e8607c33255d00a7845d1e7eff15c4a7e1f'
      'crates/planrun-exec/src/shell/session.rs' = 'da0714f7782022f9f9c92a47feb2e0785a99f3b2'
      'crates/planrun-exec/tests/shell.rs' = '1339124e5a7f01ac398e724311a51630f7f722a6'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P5-T2: sessions end when the shell is gone, wrapper commands through builtin, RemoteSigned, PDEATHSIG and interrupts" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P5-T3: sed is Read only when its script neither writes nor runs commands

**Goal:** The handover's item 6 (D72): sed is classified by reading its script, in `policy/sed.rs`: Read without `w`, `W`, `e` and the `s///w` and `s///e` flags, Modify when it writes or with `-i`, Destructive when it runs commands, comes from `-f`, has an expansion or cannot be read with confidence.

**Files**

- Modify `crates/planrun-exec/src/policy/heads.rs`
- Modify `crates/planrun-exec/src/policy/mod.rs`
- Create `crates/planrun-exec/src/policy/sed.rs`
- Modify `crates/planrun-exec/src/policy/tests.rs`

Over 300 lines, so they come as exact diffs: `crates/planrun-exec/src/policy/heads.rs`.

**Step 1 — write the files**

File `crates/planrun-exec/src/policy/heads.rs` (exact diff; save the block as `$env:TEMP\p5-t3-heads-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/policy/heads.rs b/crates/planrun-exec/src/policy/heads.rs
index 7ab15b7..91d9773 100644
--- a/crates/planrun-exec/src/policy/heads.rs
+++ b/crates/planrun-exec/src/policy/heads.rs
@@ -148,14 +148,7 @@ pub(super) fn bash(name: &str, args: &[Word]) -> Verdict {
             no("rm -r")
         }
         "rm" => ok(Modify),
-        "sed"
-            if args
-                .iter()
-                .any(|a| short(a, &['i']) || long(a, "--in-place")) =>
-        {
-            ok(Modify)
-        }
-        "sed" => ok(Read),
+        "sed" => super::sed::sed(args),
         "sort" if args.iter().any(|a| short(a, &['o']) || long(a, "--output")) => ok(Modify),
         "sort" => ok(Read),
         "find" => find(args),
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t3-heads-rs.patch"
Remove-Item "$env:TEMP\p5-t3-heads-rs.patch"
```

File `crates/planrun-exec/src/policy/mod.rs` (full contents):

```rust
//! Policy classes and the command classifier (docs/architecture.md §6.5; decisions D13, D68).
//!
//! The classifier is a built-in allowlist: a script's class is the highest class of any command
//! in it, a command the tables do not know is Destructive, and so is a script the splitter
//! cannot take apart with confidence. No format spec, `[checks]` entry or flag changes a class,
//! and a runner has one classifier, not one per format.

mod bash;
mod decide;
mod heads;
mod pwsh;
mod sed;

pub use decide::{Decision, Policy, PolicyError, Skip};
pub use heads::DEVREADY_VERSION;

use planrun_model::Shell;

/// The command classes of decision D13, least to most dangerous.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolicyClass {
    /// Reads only (`git status`, `rg`).
    #[default]
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
    fn classify(&self, shell: Shell, script: &str) -> PolicyClass;
}

/// A script's class with the reasons that made it Destructive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classification {
    /// The highest class of any command.
    pub class: PolicyClass,
    /// What made it Destructive (`git push`, `unknown command \`foo\``, `unparsed: …`), in
    /// script order; empty below Destructive.
    pub reasons: Vec<String>,
}

/// The built-in classifier (D68), with the tables of `heads.rs`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultClassifier;

impl DefaultClassifier {
    /// The class of `script` and why.
    pub fn explain(&self, shell: Shell, script: &str) -> Classification {
        let mut acc = Acc::default();
        let parsed = match shell {
            Shell::Pwsh => pwsh::classify(script, &mut acc),
            Shell::Bash => bash::classify(script, &mut acc),
        };
        if let Err(why) = parsed {
            acc.add(PolicyClass::Destructive, Some(format!("unparsed: {why}")));
        }
        Classification {
            class: acc.class,
            reasons: acc.reasons,
        }
    }
}

impl CommandClassifier for DefaultClassifier {
    fn classify(&self, shell: Shell, script: &str) -> PolicyClass {
        self.explain(shell, script).class
    }
}

/// One word of a command as the splitter saw it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Word {
    /// Its text: the value without quotes when `literal`, the raw spelling otherwise.
    pub text: String,
    /// No variable, subexpression or other expansion: `text` is exactly the value.
    pub literal: bool,
    /// Some part was quoted (a quoted word is never a parameter or a keyword).
    pub quoted: bool,
}

impl Word {
    /// A literal word that was not quoted.
    pub fn bare(&self) -> Option<&str> {
        (self.literal && !self.quoted).then_some(self.text.as_str())
    }
}

/// What the splitter found so far: the highest class, and the reasons for Destructive.
#[derive(Debug, Default)]
pub(crate) struct Acc {
    pub class: PolicyClass,
    pub reasons: Vec<String>,
}

impl Acc {
    /// Raises the class to `class`; a reason is kept only for Destructive.
    pub fn add(&mut self, class: PolicyClass, reason: Option<String>) {
        self.class = self.class.max(class);
        if class == PolicyClass::Destructive {
            self.reasons
                .push(reason.unwrap_or_else(|| "destructive".to_owned()));
        }
    }
}

/// A redirection target's class (D68): nothing for the null device, Modify for a literal
/// relative path without `..`, Destructive for anything else.
pub(crate) fn redirect_target(word: &Word, acc: &mut Acc) {
    let text = word.text.as_str();
    let null = ["$null", "/dev/null"].contains(&text) || text.eq_ignore_ascii_case("nul");
    if null && (word.literal || text == "$null") {
        return;
    }
    let relative = word.literal
        && !text.is_empty()
        && !text.starts_with(['/', '\\', '~'])
        && !text.contains(':')
        && !text.split(['/', '\\']).any(|s| s == "..");
    if relative {
        acc.add(PolicyClass::Modify, None);
        return;
    }
    acc.add(
        PolicyClass::Destructive,
        Some(format!("a redirection to `{text}`")),
    );
}

#[cfg(test)]
mod tests;
```

File `crates/planrun-exec/src/policy/sed.rs` (full contents):

```rust
//! GNU sed's class (decision D72): a sed script can write files (`w`, `W`, the `s///w` flag) and
//! run commands (`e`, the `s///e` flag), and `-f` reads a script the classifier never sees. So
//! the script is read command by command: Read when it does neither, Modify when it writes (or
//! with `-i`), Destructive when it runs commands, comes from a file, or cannot be read with
//! confidence.

use super::PolicyClass::{Destructive, Modify, Read};
use super::Word;
use super::heads::Verdict;

/// What a script does besides printing.
#[derive(Debug, Default, PartialEq, Eq)]
struct Effects {
    writes: bool,
    runs: bool,
}

fn no(reason: impl Into<String>) -> Verdict {
    (Destructive, Some(reason.into()))
}

/// The class of `sed` with `args`.
pub(super) fn sed(args: &[Word]) -> Verdict {
    let mut scripts: Vec<&str> = Vec::new();
    let mut operands: Vec<&Word> = Vec::new();
    let mut in_place = false;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        i += 1;
        let Some(text) = arg.bare() else {
            operands.push(arg);
            continue;
        };
        if text == "--" {
            operands.extend(&args[i..]);
            break;
        }
        let option = if let Some(long) = text.strip_prefix("--") {
            long_option(long)
        } else if text.len() > 1 && text.starts_with('-') {
            short_cluster(&text[1..])
        } else {
            operands.push(arg);
            continue;
        };
        match option {
            Opt::Flag => {}
            Opt::InPlace => in_place = true,
            Opt::Script(Some(script)) => scripts.push(script),
            Opt::Script(None) | Opt::Number => {
                let Some(next) = args.get(i) else {
                    return no(format!("sed option `{text}` without its value"));
                };
                i += 1;
                if matches!(option, Opt::Script(None)) {
                    let Some(script) = next.literal.then_some(next.text.as_str()) else {
                        return no("a sed script with an expansion");
                    };
                    scripts.push(script);
                }
            }
            Opt::File => return no("sed -f"),
            Opt::Unknown => return no(format!("sed option `{text}`")),
        }
    }
    if scripts.is_empty() {
        let Some(first) = operands.first() else {
            return no("sed without a script");
        };
        let Some(script) = first.literal.then_some(first.text.as_str()) else {
            return no("a sed script with an expansion");
        };
        scripts.push(script);
    }
    let mut effects = Effects::default();
    for script in scripts {
        match read_script(script) {
            Ok(found) => {
                effects.writes |= found.writes;
                effects.runs |= found.runs;
            }
            Err(why) => return no(format!("a sed script it cannot read: {why}")),
        }
    }
    if effects.runs {
        return no("sed runs a command (`e`)");
    }
    if effects.writes || in_place {
        return (Modify, None);
    }
    (Read, None)
}

/// What one option is.
enum Opt<'a> {
    Flag,
    InPlace,
    /// `-e`/`--expression`: the script glued to it, or `None` for the next argument.
    Script(Option<&'a str>),
    /// `-l N`: the next argument is a number.
    Number,
    File,
    Unknown,
}

fn long_option(name: &str) -> Opt<'_> {
    let (name, value) = match name.split_once('=') {
        Some((n, v)) => (n, Some(v)),
        None => (name, None),
    };
    match (name, value) {
        ("in-place", _) => Opt::InPlace,
        ("expression", value) => Opt::Script(value),
        ("file", _) => Opt::File,
        ("line-length", Some(_)) => Opt::Flag,
        ("line-length", None) => Opt::Number,
        (
            "quiet" | "silent" | "regexp-extended" | "separate" | "unbuffered" | "null-data"
            | "posix" | "debug" | "sandbox" | "follow-symlinks",
            None,
        ) => Opt::Flag,
        _ => Opt::Unknown,
    }
}

/// A short-option cluster without its dash: `n`, `nE`, `i.bak`, `ne`, `es/a/b/`, `l80`.
fn short_cluster(cluster: &str) -> Opt<'_> {
    for (at, c) in cluster.char_indices() {
        let rest = &cluster[at + 1..];
        match c {
            'n' | 'E' | 'r' | 's' | 'u' | 'z' => {}
            // `-i` takes the rest of the cluster as its backup suffix.
            'i' => return Opt::InPlace,
            'e' if rest.is_empty() => return Opt::Script(None),
            'e' => return Opt::Script(Some(rest)),
            'l' if rest.is_empty() => return Opt::Number,
            'l' => return Opt::Flag,
            'f' => return Opt::File,
            _ => return Opt::Unknown,
        }
    }
    Opt::Flag
}

/// Reads a sed script command by command (GNU syntax). An error is anything it does not
/// recognise with confidence.
fn read_script(script: &str) -> Result<Effects, String> {
    let chars: Vec<char> = script.chars().collect();
    let mut p = Parser {
        chars: &chars,
        at: 0,
        effects: Effects::default(),
    };
    p.commands()?;
    Ok(p.effects)
}

struct Parser<'a> {
    chars: &'a [char],
    at: usize,
    effects: Effects,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        self.at += usize::from(c.is_some());
        c
    }

    fn skip_blanks(&mut self) {
        while self.peek().is_some_and(|c| c == ' ' || c == '\t') {
            self.at += 1;
        }
    }

    fn commands(&mut self) -> Result<(), String> {
        loop {
            while self
                .peek()
                .is_some_and(|c| c.is_whitespace() || c == ';' || c == '}')
            {
                self.at += 1;
            }
            if self.peek().is_none() {
                return Ok(());
            }
            self.address()?;
            self.skip_blanks();
            if self.peek() == Some(',') {
                self.at += 1;
                self.skip_blanks();
                self.address()?;
            }
            self.skip_blanks();
            while self.peek() == Some('!') {
                self.at += 1;
                self.skip_blanks();
            }
            self.command()?;
        }
    }

    /// One address: a line number (`3`, `0`, `1~2`, `+3`, `~4`), `$`, `/re/` or `\cREc`, each
    /// with optional `I` and `M` flags after a regex. No address is fine.
    fn address(&mut self) -> Result<(), String> {
        match self.peek() {
            Some(c) if c.is_ascii_digit() || c == '+' || c == '~' => {
                self.at += 1;
                while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '~') {
                    self.at += 1;
                }
            }
            Some('$') => self.at += 1,
            Some('/') => {
                self.at += 1;
                self.delimited('/')?;
                self.regex_flags();
            }
            Some('\\') => {
                self.at += 1;
                let delim = self.bump().ok_or("an address ends after `\\`")?;
                self.delimited(delim)?;
                self.regex_flags();
            }
            _ => {}
        }
        Ok(())
    }

    fn regex_flags(&mut self) {
        while self.peek().is_some_and(|c| c == 'I' || c == 'M') {
            self.at += 1;
        }
    }

    /// Up to the next `delim` that no backslash escapes; the delimiter is consumed. A newline
    /// inside is refused: GNU sed accepts some, and nothing here needs them.
    fn delimited(&mut self, delim: char) -> Result<(), String> {
        loop {
            match self.bump() {
                None | Some('\n') => return Err(format!("an unterminated `{delim}` part")),
                Some('\\') => {
                    self.bump().ok_or("a part ends after `\\`")?;
                }
                Some(c) if c == delim => return Ok(()),
                Some(_) => {}
            }
        }
    }

    /// The rest of the line, for a text, file name or label argument.
    fn rest_of_line(&mut self) {
        while self.peek().is_some_and(|c| c != '\n') {
            self.at += 1;
        }
    }

    /// The text of `a`, `i` and `c`: to the end of the line, and on through every line that
    /// ends in an unescaped backslash.
    fn text(&mut self) {
        loop {
            let mut escaped = false;
            while let Some(c) = self.peek() {
                if c == '\n' {
                    break;
                }
                escaped = c == '\\' && !escaped;
                self.at += 1;
            }
            if !escaped || self.bump().is_none() {
                return;
            }
        }
    }

    /// A label: up to `;` or the end of the line.
    fn label(&mut self) {
        while self.peek().is_some_and(|c| c != '\n' && c != ';') {
            self.at += 1;
        }
    }

    fn command(&mut self) -> Result<(), String> {
        let Some(c) = self.bump() else {
            return Err("an address without a command".to_owned());
        };
        match c {
            '{' => {}
            '#' | 'r' | 'R' => self.rest_of_line(),
            'a' | 'i' | 'c' => self.text(),
            'w' | 'W' => {
                self.effects.writes = true;
                self.rest_of_line();
            }
            'e' => {
                self.effects.runs = true;
                self.rest_of_line();
            }
            ':' | 'b' | 't' | 'T' | 'v' => self.label(),
            '=' | 'd' | 'D' | 'g' | 'G' | 'h' | 'H' | 'n' | 'N' | 'p' | 'P' | 'x' | 'z' | 'F' => {}
            'l' | 'L' | 'q' | 'Q' => {
                self.skip_blanks();
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.at += 1;
                }
            }
            's' => self.substitute()?,
            'y' => {
                let delim = self.delimiter()?;
                self.delimited(delim)?;
                self.delimited(delim)?;
            }
            other => return Err(format!("the command `{other}`")),
        }
        Ok(())
    }

    fn delimiter(&mut self) -> Result<char, String> {
        match self.bump() {
            Some(c) if c != '\n' && c != '\\' => Ok(c),
            _ => Err("a missing delimiter".to_owned()),
        }
    }

    /// `s/re/replacement/flags`: `w file` writes, `e` runs the pattern space as a command.
    fn substitute(&mut self) -> Result<(), String> {
        let delim = self.delimiter()?;
        self.delimited(delim)?;
        self.delimited(delim)?;
        while let Some(c) = self.peek() {
            match c {
                'g' | 'p' | 'i' | 'I' | 'm' | 'M' => self.at += 1,
                c if c.is_ascii_digit() => self.at += 1,
                'e' => {
                    self.effects.runs = true;
                    self.at += 1;
                }
                'w' => {
                    self.effects.writes = true;
                    self.rest_of_line();
                }
                _ => break,
            }
        }
        match self.peek() {
            None | Some(';' | '\n' | '}' | ' ' | '\t' | '#') => Ok(()),
            Some(other) => Err(format!("the `s` flag `{other}`")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn scripts() {
        let rows: &[(&str, Result<Effects, ()>)] = &[
            ("s/a/b/", Ok(Effects::default())),
            ("s/a/b/g; 1,5p; $d", Ok(Effects::default())),
            ("/start/,/end/{ s|x|y|2; p }", Ok(Effects::default())),
            ("s/test/\\/e/g", Ok(Effects::default())),
            (r"\,a,d", Ok(Effects::default())),
            ("y/abc/xyz/", Ok(Effects::default())),
            ("1i\\\nheader", Ok(Effects::default())),
            ("r other.txt", Ok(Effects::default())),
            ("s/a/b/w out.txt", Ok(Effects { writes: true, runs: false })),
            ("/x/w out.txt", Ok(Effects { writes: true, runs: false })),
            ("W out.txt", Ok(Effects { writes: true, runs: false })),
            ("s/.*/date/e", Ok(Effects { writes: false, runs: true })),
            ("1e date", Ok(Effects { writes: false, runs: true })),
            ("s/a/b", Err(())),
            ("s/a/b/k", Err(())),
            ("1,2", Err(())),
            ("K", Err(())),
        ];
        for (script, expected) in rows {
            let got = read_script(script).map_err(|_| ());
            assert_eq!(&got, expected, "{script:?}");
        }
    }
}
```

File `crates/planrun-exec/src/policy/tests.rs` (full contents):

```rust
//! The classifier's tables, row by row: one test per shell, and one for what the splitter
//! refuses.

use planrun_model::Shell;

use super::PolicyClass::{Build, Destructive, Modify, Read};
use super::{DefaultClassifier, PolicyClass};

fn check(shell: Shell, rows: &[(&str, PolicyClass)]) {
    for (script, class) in rows {
        let found = DefaultClassifier.explain(shell, script);
        assert_eq!(
            found.class, *class,
            "{shell:?} {script:?}: {:?}",
            found.reasons
        );
        assert_eq!(
            found.reasons.is_empty(),
            *class != Destructive,
            "{script:?}"
        );
    }
}

#[test]
fn pwsh_commands() {
    #[rustfmt::skip]
    let rows = [
        ("", Read),
        ("git status --short            # empty", Read),
        ("Get-Content docs/modules.txt", Read),
        ("rg -n 'format!\\(\"Hello' src --glob '!src/lang.rs'", Read),
        ("$x = 41\nfunction F { 'f-called' }", Read),
        ("\"Hello, $name$($config.punctuation)\"", Read),
        ("$config = Get-Content (Join-Path $PSScriptRoot 'greet.json') -Raw | ConvertFrom-Json", Read),
        ("foreach ($name in $config.names) { \"Hello, $name\" }", Read),
        ("if ($gate -eq 0) { 'ok' }\nelse { 'no' }", Read),
        ("replace 'a' 'b' -Path src -Extension .rs -DryRun", Read),
        ("gitinfo; markers; Test-DevReadyPlan x.md", Read),
        ("[IO.Path]::Combine('a', 'b'); [Console]::Out; $s.Trim().Split(',').Count", Read),
        ("@{ a = 1; 'b' = 'x' }; @{a=1}", Read),
        ("git -C sub status; git -c color.ui=always --no-pager log --oneline", Read),
        ("git branch --show-current; git branch; git branch --list 'p*'", Read),
        ("Get-ChildItem -Recurse src | Sort-Object | Select-Object -First 3", Read),
        ("@'\nRemove-Item -Recurse x\n'@", Read),
        ("<# rm -rf #> 'x'; exit $gate", Read),
        ("& git status; Get-Item x | % { $_.Name } | ? { $_ }", Read),
        ("Get-Item x -ErrorAction SilentlyContinue; $a[0]; Write-Output a > $null 2>&1", Read),
        ("cargo --version; git --version; Set-Location sub; cd ..", Read),
        ("cargo test", Build),
        ("cargo test; $gate = $LASTEXITCODE; \"gate exit code: $gate\"", Build),
        ("cargo +nightly --manifest-path x/Cargo.toml build -q", Build),
        ("cargo fmt --check; cargo clippy --all-targets -- -D warnings", Build),
        ("dotnet build; dotnet restore; npm ci; npm run build; go test ./...", Build),
        ("brief { cargo test -p x }; test", Build),
        ("git add -- src/greet.rs src/lib.rs", Modify),
        ("if ($gate -eq 0) {\n  git add -- a\n  git commit -m \"m\"\n} else { 'no' }\ngit status --porcelain", Modify),
        ("git switch -c feature/x; git checkout -b y; git branch new", Modify),
        ("replace 'greeting(' 'greeting_en(' -Path src -Extension .rs -All", Modify),
        ("New-Item -ItemType Directory -Force docs | Out-Null; (Get-ChildItem src -Name | Sort-Object) -join \"`n\" | Set-Content -NoNewline docs/modules.txt", Modify),
        ("Remove-Item x.txt; Copy-Item a b; ni x; Tee-Object -FilePath x", Modify),
        ("'x' > out.txt; 'y' >> logs/out.txt", Modify),
        ("cargo fmt; cargo clippy --fix; npm install; dotnet format; go mod tidy", Modify),
        ("git diff --output=x.diff", Modify),
        ("Remove-Item -Recurse x", Destructive),
        ("Remove-Item –Recurse x", Destructive),
        ("rm -r x", Destructive),
        ("ri -rec x", Destructive),
        ("Remove-Item -Recurse:$true x", Destructive),
        ("Remove-Item @rest", Destructive),
        ("git push", Destructive),
        ("git reset --hard", Destructive),
        ("git -C x reset --hard", Destructive),
        ("git -c core.pager=less log", Destructive),
        ("git -c alias.x='!rm -rf .' x", Destructive),
        ("git --git-dir=x status", Destructive),
        ("git checkout main", Destructive),
        ("git checkout -- src/lib.rs", Destructive),
        ("git switch -f main", Destructive),
        ("git clean -fdx", Destructive),
        ("git stash drop", Destructive),
        ("git branch -D x", Destructive),
        ("git diff --ext-diff", Destructive),
        ("git config --global user.name x", Destructive),
        ("GIT.EXE PUSH", Destructive),
        ("iex 'x'", Destructive),
        ("Start-Process x", Destructive),
        ("irm http://x", Destructive),
        ("pwsh -NoProfile -File tools/greet.ps1   # Hello, Rens!", Destructive),
        ("bash -c 'ls'", Destructive),
        ("cmd /c dir", Destructive),
        ("& $cmd", Destructive),
        ("& 'git' status", Destructive),
        ("& .\\x.ps1", Destructive),
        (". ./x.ps1", Destructive),
        ("./script.ps1", Destructive),
        ("C:\\tools\\x.exe", Destructive),
        ("Microsoft.PowerShell.Management\\Remove-Item x", Destructive),
        ("frobnicate", Destructive),
        ("[IO.File]::Delete('x')", Destructive),
        ("$p.Kill()", Destructive),
        ("Get-ChildItem | % Delete", Destructive),
        ("'x' > C:\\out.txt", Destructive),
        ("'x' > ../out.txt", Destructive),
        ("'x' > $path", Destructive),
        ("cargo run; cargo install x", Destructive),
        ("npx x", Destructive),
        ("dotnet run", Destructive),
        ("go run .", Destructive),
        ("Import-Module ./module/x.psd1 -Force", Destructive),
        ("Add-Type -TypeDefinition $s", Destructive),
        ("New-Object Net.WebClient", Destructive),
        ("New-Item -ItemType SymbolicLink -Path l -Target x", Destructive),
        ("killport 8080", Destructive),
        ("replace 'a' 'b'", Destructive),
        ("\"$(Remove-Item -Recurse x)\"", Destructive),
        ("@\"\n$(git push)\n\"@", Destructive),
        ("@(git push)", Destructive),
        ("Write-Output ([IO.File]::ReadAllText('x'))", Destructive),
    ];
    check(Shell::Pwsh, &rows);
}

#[test]
fn bash_commands() {
    #[rustfmt::skip]
    let rows = [
        ("git branch --show-current     # feature/x", Read),
        ("cat x | head -1; wc -l < x", Read),
        ("test ! -e x && echo gone", Read),
        ("echo \"$(git status --short)\"; x=$(git rev-parse HEAD)", Read),
        ("for f in a b; do echo \"$f\"; done", Read),
        ("if [ -f x ]; then echo y; else echo n; fi", Read),
        ("arr=(a \"b c\" $(git status)); declare -A m=([k]=v)", Read),
        ("[[ -n $x ]]; (( i++ ))", Read),
        ("find . -name '*.rs'; sed 's/a/b/' x; command -v git", Read),
        ("cat <<'EOF'\nrm -rf /\nEOF\necho after", Read),
        ("echo x > /dev/null 2>&1; echo y >&2; echo z 2>/dev/null", Read),
        ("(cd sub && ls)", Read),
        ("cargo test 2>&1 | tail -3", Build),
        ("git add src && git commit -m 'x'", Modify),
        ("mkdir -p sub && cd sub", Modify),
        ("echo x > out.txt; echo y >> out.txt", Modify),
        ("sed -i 's/a/b/' x; rm x; touch y; sort -o x y; cp a b", Modify),
        ("cat <<EOF > out.txt\n$(git status)\nEOF", Modify),
        ("sed -n '1,5p' x; sed -E -e 's/a/b/g' -e '$d' x; sed --quiet '/x/p' x", Read),
        ("sed 's/a/b/w out.txt' x; sed -n '/x/w out.txt' x; sed -i.bak 's/a/b/' x", Modify),
        ("sed 's/.*/date/e' x", Destructive),
        ("sed '1e date' x", Destructive),
        ("sed -f script.sed x", Destructive),
        ("sed --file=script.sed x", Destructive),
        ("sed \"$expr\" x", Destructive),
        ("sed 's/a/b/k' x", Destructive),
        ("g() { echo; }", Read),
        ("rm -rf x", Destructive),
        ("rm --recursive x", Destructive),
        ("find . -delete", Destructive),
        ("find . -exec rm {} \\;", Destructive),
        ("git push origin main", Destructive),
        ("curl http://x", Destructive),
        ("bash x.sh", Destructive),
        ("./x.sh", Destructive),
        ("source x.sh", Destructive),
        (". x.sh", Destructive),
        ("eval \"$x\"", Destructive),
        ("$cmd", Destructive),
        ("FOO=1 git status", Destructive),
        ("pwsh -NoProfile -File tools/greet.ps1 -Names Ada | tail -1   # Hello, Ada!", Destructive),
        ("echo `rm -rf x`", Destructive),
        ("echo \"$(rm -rf x)\"", Destructive),
        ("cat <(curl x)", Destructive),
        ("cat <<EOF\n$(rm -rf x)\nEOF", Destructive),
        ("echo x > /etc/x", Destructive),
        ("echo x > ~/x", Destructive),
        ("echo x > \"$f\"", Destructive),
        ("g() { rm -rf x; }", Destructive),
        ("function g { git push; }", Destructive),
        ("xargs rm", Destructive),
        ("exec true", Destructive),
        ("\\rm x", Destructive),
    ];
    check(Shell::Bash, &rows);
}

#[test]
fn what_the_splitter_cannot_take_apart_is_destructive() {
    #[rustfmt::skip]
    let rows = [
        (Shell::Pwsh, "'unterminated"),
        (Shell::Pwsh, "\"unterminated"),
        (Shell::Pwsh, "@'\nnever closed"),
        (Shell::Pwsh, "@'not on its own line'@"),
        (Shell::Pwsh, "Write-Output ("),
        (Shell::Pwsh, "Write-Output )"),
        (Shell::Pwsh, "<# open"),
        (Shell::Pwsh, "$(Get-Item x"),
        (Shell::Pwsh, "[IO.Path"),
        (Shell::Pwsh, "Get-Content < x"),
        (Shell::Pwsh, "'x' >"),
        (Shell::Bash, "echo 'x"),
        (Shell::Bash, "echo \"x"),
        (Shell::Bash, "case $x in a) echo;; esac"),
        (Shell::Bash, "cat <<EOF\nnever"),
        (Shell::Bash, "echo ${x:-$(rm -rf /)}"),
        (Shell::Bash, "echo )"),
        (Shell::Bash, "echo $("),
        (Shell::Bash, "echo `x"),
        (Shell::Bash, "f ("),
    ];
    for (shell, script) in rows {
        let found = DefaultClassifier.explain(shell, script);
        assert_eq!(found.class, Destructive, "{shell:?} {script:?}");
        let unparsed = found.reasons.iter().any(|r| r.starts_with("unparsed: "));
        assert!(unparsed, "{shell:?} {script:?}: {:?}", found.reasons);
    }
}
```

**Step 2 — verify**

Targeted: the classifier's tables and the sed reader:

```powershell
cargo test -p planrun-exec --lib policy
```

Expected (exit code 0; summary lines):

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 32 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-exec/src/policy/heads.rs crates/planrun-exec/src/policy/mod.rs crates/planrun-exec/src/policy/sed.rs crates/planrun-exec/src/policy/tests.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/policy/heads.rs' = '91d97734411ef5e343a8d9925d56d51293ddf23a'
      'crates/planrun-exec/src/policy/mod.rs' = '3feae9935382c81bd6e65519ec63a3209f747b78'
      'crates/planrun-exec/src/policy/sed.rs' = 'e8891d72dda19cb11ab392736bedb613bd05cb84'
      'crates/planrun-exec/src/policy/tests.rs' = '9f9410c1d5da40b0e5ce161b0417d92f93ef319d'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P5-T3: sed is Read only when its script neither writes nor runs commands" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P5-T4: the step record, the disk checked again before commit, modes, temp names and the step's state on disk

**Goal:** The step transaction for a real run (D78) and the step's state on disk for resume (D77). `PreparedStep` writes carry a `WriteRecord`: pre-image id and mode, the post-image's exact and filtered ids and mode, the directories to create, and the writer's temp name chosen at prepare; `StepRecord` is what the journal will keep. `commit` checks the disk against the pre-images first (`CommitCause::Changed`, nothing recorded or written), then records, then writes with the mode under `core.filemode`, and calls `WriteAhead::after_write` after each path. A real preflight refuses a git delete of an untracked file (`delete-untracked`). `done.rs` compares a step record with the disk (Done, NotDone, Unknown), says whether it can be restored, restores it, and removes its temp files by name. git gains `hash_paths` and `missing_objects`; `BlobId` serializes as hex.

**Files**

- Create `crates/planrun-exec/src/done.rs`
- Create `crates/planrun-exec/src/done_tests.rs`
- Modify `crates/planrun-exec/src/git.rs`
- Modify `crates/planrun-exec/src/lib.rs`
- Modify `crates/planrun-exec/src/preflight.rs`
- Modify `crates/planrun-exec/src/transaction.rs`
- Modify `crates/planrun-exec/src/transaction_tests.rs`
- Modify `crates/planrun-exec/src/workspace.rs`
- Modify `crates/planrun-exec/tests/preflight.rs`

Over 300 lines, so they come as exact diffs: `crates/planrun-exec/src/git.rs`, `crates/planrun-exec/src/preflight.rs`, `crates/planrun-exec/src/transaction.rs`, `crates/planrun-exec/src/workspace.rs`.

**Step 1 — write the files**

File `crates/planrun-exec/src/done.rs` (full contents):

```rust
//! A journaled step's state on the disk, for resume (decisions D14, D77, D78): whether its
//! writes are all there (Done), none of them are (NotDone), or anything else (Unknown, the user
//! decides); whether its pre-images survive to restore it; and the writer's temp files it may
//! have left behind.

use std::fs;

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
    let mut ids = ws.git().hash_paths(&present)?.into_iter();
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
    let mut removed = Vec::new();
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
        let is_file = fs::symlink_metadata(&resolved.full).is_ok_and(|m| m.is_file());
        if is_file && fs::remove_file(&resolved.full).is_ok() {
            removed.push(rel);
        }
    }
    removed
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

File `crates/planrun-exec/src/git.rs` (exact diff; save the block as `$env:TEMP\p5-t4-git-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/git.rs b/crates/planrun-exec/src/git.rs
index 222bc1c..a1f3754 100644
--- a/crates/planrun-exec/src/git.rs
+++ b/crates/planrun-exec/src/git.rs
@@ -42,6 +42,20 @@ impl fmt::Display for BlobId {
     }
 }
 
+/// 40 lowercase hex digits in the journal (D76).
+impl serde::Serialize for BlobId {
+    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
+        s.collect_str(self)
+    }
+}
+
+impl<'de> serde::Deserialize<'de> for BlobId {
+    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
+        let hex = String::deserialize(d)?;
+        BlobId::parse(&hex).ok_or_else(|| serde::de::Error::custom(format!("not a blob id: {hex}")))
+    }
+}
+
 /// Why a git call failed.
 #[derive(Debug, thiserror::Error)]
 pub enum GitError {
@@ -222,6 +236,47 @@ impl Git {
         blob_id(&args, &out)
     }
 
+    /// The exact-bytes blob ids (`--no-filters`) of the files at `paths`, in one
+    /// `hash-object --stdin-paths` call, nothing stored. Every path must exist: git fails the
+    /// whole batch on a missing one, so callers stat first.
+    pub fn hash_paths(&self, paths: &[&RepoPath]) -> Result<Vec<BlobId>, GitError> {
+        if paths.is_empty() {
+            return Ok(Vec::new());
+        }
+        let args = ["hash-object", "--no-filters", "--stdin-paths"];
+        let input: String = paths.iter().map(|p| format!("{p}\n")).collect();
+        let out = self.run(&args, Some(input.as_bytes()), &[])?;
+        let ids: Vec<BlobId> = text(&out)
+            .lines()
+            .map(|line| BlobId::parse(line).ok_or_else(|| output_error(&args, line)))
+            .collect::<Result<_, _>>()?;
+        if ids.len() != paths.len() {
+            return Err(output_error(&args, &text(&out)));
+        }
+        Ok(ids)
+    }
+
+    /// The ids among `ids` that the object database does not have (pruned by `git gc`, D57),
+    /// from one `cat-file --batch-check` call.
+    pub fn missing_objects(&self, ids: &[BlobId]) -> Result<Vec<BlobId>, GitError> {
+        if ids.is_empty() {
+            return Ok(Vec::new());
+        }
+        let input: String = ids.iter().map(|id| format!("{id}\n")).collect();
+        let out = self.run(
+            &["cat-file", "--batch-check=%(objectname)"],
+            Some(input.as_bytes()),
+            &[],
+        )?;
+        let out = text(&out);
+        Ok(ids
+            .iter()
+            .zip(out.lines())
+            .filter(|(_, line)| line.ends_with(" missing"))
+            .map(|(id, _)| *id)
+            .collect())
+    }
+
     /// The bytes of blob `id`, unconverted.
     pub fn cat_blob(&self, id: &BlobId, extra: Env<'_>) -> Result<Vec<u8>, GitError> {
         self.run(&["cat-file", "blob", &id.to_string()], None, extra)
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t4-git-rs.patch"
Remove-Item "$env:TEMP\p5-t4-git-rs.patch"
```

File `crates/planrun-exec/src/lib.rs` (full contents):

```rust
//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Modules (docs/roadmap.md; P3 built the first eight, P4 the next three):
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
//! - [`transaction`]: a step's file actions as prepare, then commit, with rollback (D53); the
//!   disk is checked again between the two (D78).
//! - [`done`]: a journaled step's state on the disk, for resume (D14, D77).
//! - [`preflight`]: task preflight and the dry run over one virtual tree (D50, D55).
//! - [`finding`]: what preflight, a simulation and the dry run report.
//! - [`shell`]: persistent pwsh and bash sessions behind [`ShellSession`] (D60–D65).
//! - [`expect`]: a run against its expectation: exit codes, summary lines with `…`, what asks
//!   for confirmation (D11, D66).
//! - [`policy`]: command classes and the [`CommandClassifier`]: a built-in allowlist over a
//!   splitter per shell (D13, D68).
//! - `journal`: JSONL under `.git/planrun/`.
//! - `runner`: the state machine Task → Step → Action, talking to a [`Ui`].

pub mod done;
pub mod eol;
pub mod expect;
pub mod finding;
pub mod git;
pub mod handler;
pub mod policy;
pub mod preflight;
pub mod shell;
pub mod transaction;
pub mod tree;
pub mod workspace;

pub use handler::{ActionHandler, HandlerSet};
pub use policy::{CommandClassifier, PolicyClass};
pub use shell::{ScriptRun, ShellSession};

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "../tests/support/ir.rs"]
mod ir;

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

File `crates/planrun-exec/src/preflight.rs` (exact diff; save the block as `$env:TEMP\p5-t4-preflight-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/preflight.rs b/crates/planrun-exec/src/preflight.rs
index a09fd75..68eac14 100644
--- a/crates/planrun-exec/src/preflight.rs
+++ b/crates/planrun-exec/src/preflight.rs
@@ -98,6 +98,8 @@ struct Walk<'t, 'w> {
     tree: &'t mut VirtualTree<'w>,
     handlers: HandlerSet,
     unrun: bool,
+    /// A real run's preflight: the repository is about to be changed, not simulated (D78).
+    real: bool,
 }
 
 /// The dry run of `plan` (D55): parse and checks are done; this preflights every task against
@@ -110,6 +112,7 @@ pub fn dry_run(ws: &Workspace, plan: &Plan) -> Result<DryRun, WorkspaceError> {
         tree: &mut tree,
         handlers: HandlerSet::files(),
         unrun: false,
+        real: false,
     };
     let tasks = plan
         .tasks
@@ -142,6 +145,7 @@ pub fn preflight(ws: &Workspace, plan: &Plan, task: &Task) -> Result<Vec<Finding
         tree: &mut tree,
         handlers: HandlerSet::files(),
         unrun: false,
+        real: true,
     };
     let report = walk.task(plan, 0, task, true);
     found.extend(report.manifest);
@@ -252,6 +256,9 @@ impl Walk<'_, '_> {
         if self.tree.is_unknown(path) {
             return Outcome::Depends(depends(path, action));
         }
+        if let Some(f) = self.untracked_delete(action, path) {
+            return Outcome::Failed(f);
+        }
         let Some(handler) = self.handlers.find(action) else {
             return Outcome::Failed(Finding::error("internal", span, Some(path), "no handler"));
         };
@@ -267,6 +274,27 @@ impl Walk<'_, '_> {
         }
     }
 
+    /// A real run's `git rm` of a path git does not track: the plan's own command would fail
+    /// (D78). The dry run keeps simulating it (P3's stated decision 16).
+    fn untracked_delete(&self, action: &Action, path: &RepoPath) -> Option<Finding> {
+        let via_git = matches!(action.kind, ActionKind::Delete { via_git: true, .. });
+        if !self.real || !via_git {
+            return None;
+        }
+        let tracked = self.tree.workspace().git().index_entry(path, &[]);
+        let message = match tracked {
+            Ok(Some(_)) => return None,
+            Ok(None) => format!("`{path}` is not tracked by git, so `git rm` would refuse it"),
+            Err(e) => e.to_string(),
+        };
+        Some(Finding::error(
+            "delete-untracked",
+            Some(action.span),
+            Some(path),
+            message,
+        ))
+    }
+
     /// After a script or Manual that did not run, a failure of the tree's state is a warning.
     fn after_unrun(&self, f: Finding) -> Finding {
         if !self.unrun || !depends_on_state(f.code) {
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t4-preflight-rs.patch"
Remove-Item "$env:TEMP\p5-t4-preflight-rs.patch"
```

File `crates/planrun-exec/src/transaction.rs` (exact diff; save the block as `$env:TEMP\p5-t4-transaction-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/transaction.rs b/crates/planrun-exec/src/transaction.rs
index 3f192cc..c8f4150 100644
--- a/crates/planrun-exec/src/transaction.rs
+++ b/crates/planrun-exec/src/transaction.rs
@@ -1,20 +1,22 @@
-//! A step's file actions as one transaction (decisions D23, D53): **prepare** validates them on
-//! a virtual tree seeded from the disk, stores the pre-images and builds every post-image;
-//! **commit** hands the prepared step to a write-ahead recorder and then writes it with the one
-//! writer, rolling back on any error.
+//! A step's file actions as one transaction (decisions D23, D53, D78): **prepare** validates
+//! them on a virtual tree seeded from the disk, stores the pre-images and builds every
+//! post-image; **commit** checks that the disk is still what prepare saw, hands the step to a
+//! write-ahead recorder, and then writes it with the one writer, rolling back on any error.
 
 use std::path::PathBuf;
 
 use planrun_model::{Action, ActionKind, PlanSource, RepoPath};
+use serde::{Deserialize, Serialize};
 
 use crate::finding::Finding;
 use crate::git::BlobId;
 use crate::handler::{HandlerSet, file_path};
 use crate::tree::{State, VirtualTree};
-use crate::workspace::{Workspace, WorkspaceError, remove_dirs};
+use crate::workspace::{Workspace, WorkspaceError, remove_dirs, temp_name};
 
 /// What a path was before the step.
-#[derive(Debug, Clone, PartialEq, Eq)]
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "lowercase")]
 pub enum PreImage {
     /// It did not exist.
     Absent,
@@ -34,19 +36,36 @@ pub enum PostImage {
 /// One path the step changes.
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub struct PlannedWrite {
+    /// The path, and everything resume needs about it.
+    pub record: WriteRecord,
+    /// After: the bytes, or a delete.
+    pub post: PostImage,
+}
+
+/// A planned write without its bytes: what the journal keeps (D76) and resume checks the disk
+/// against (D14, D77).
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+pub struct WriteRecord {
     /// The path.
     pub path: RepoPath,
     /// Before.
     pub pre: PreImage,
-    /// After.
-    pub post: PostImage,
-    /// The post-image's blob id as git will see it; `None` for a delete.
+    /// The index mode before (`100644`), when the path was present.
+    pub pre_mode: Option<String>,
+    /// The exact-bytes id of the post-image (`--no-filters`, not stored); `None` for a delete.
+    pub post: Option<BlobId>,
+    /// The post-image's blob id as git will see it (filtered); `None` for a delete.
     pub post_blob: Option<BlobId>,
+    /// The index mode after (a patch's mode lines included); `None` for a delete.
+    pub mode: Option<String>,
     /// The directories the write will create, from the root, outermost first.
     pub creates_dirs: Vec<String>,
+    /// The writer's temp file name in the target's directory, chosen here so that resume can
+    /// remove exactly it (D77).
+    pub temp: String,
 }
 
-/// A validated step, ready to commit: what P5 journals before writing (write-ahead, D53).
+/// A validated step, ready to commit.
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub struct PreparedStep {
     /// The paths, in the order their first action names them.
@@ -55,11 +74,34 @@ pub struct PreparedStep {
     pub index_removals: Vec<RepoPath>,
 }
 
+/// A prepared step without its bytes: the write-ahead record (D53, D76).
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+pub struct StepRecord {
+    /// The planned writes.
+    pub writes: Vec<WriteRecord>,
+    /// The index removals.
+    pub index_removals: Vec<RepoPath>,
+}
+
+impl PreparedStep {
+    /// What the journal records before the step is written.
+    pub fn record(&self) -> StepRecord {
+        StepRecord {
+            writes: self.writes.iter().map(|w| w.record.clone()).collect(),
+            index_removals: self.index_removals.clone(),
+        }
+    }
+}
+
 /// Records a prepared step before it is written. P5's recorder is the journal; a real run
 /// without that record is not allowed (D53).
 pub trait WriteAhead {
     /// Stores `step` durably; an error stops the commit before anything is written.
     fn record(&mut self, step: &PreparedStep) -> std::io::Result<()>;
+
+    /// Called after each path is written, with how many are written so far. The default does
+    /// nothing; planrun-cli's fault seam aborts here in its e2e tests (D82).
+    fn after_write(&mut self, _written: usize) {}
 }
 
 /// Why a commit failed, and what rollback could not undo.
@@ -75,6 +117,10 @@ pub struct CommitError {
 /// The first failure of a commit.
 #[derive(Debug, thiserror::Error)]
 pub enum CommitCause {
+    /// Between prepare and commit something changed these paths (or created a directory the
+    /// step would create); nothing was recorded or written (D78).
+    #[error("changed since the step was prepared: {}", list(.0))]
+    Changed(Vec<RepoPath>),
     /// The write-ahead record failed; nothing was written.
     #[error("the write-ahead record failed: {0}")]
     Record(#[source] std::io::Error),
@@ -83,12 +129,16 @@ pub enum CommitCause {
     Workspace(#[from] WorkspaceError),
 }
 
+fn list(paths: &[RepoPath]) -> String {
+    let names: Vec<&str> = paths.iter().map(RepoPath::as_str).collect();
+    names.join(", ")
+}
+
 fn unrestored_note(paths: &[RepoPath]) -> String {
     if paths.is_empty() {
         return String::new();
     }
-    let list: Vec<&str> = paths.iter().map(RepoPath::as_str).collect();
-    format!("; rollback could not restore {}", list.join(", "))
+    format!("; rollback could not restore {}", list(paths))
 }
 
 /// Validates `actions` (a step's file actions) on a fresh virtual tree, stores the pre-image of
@@ -130,6 +180,10 @@ pub fn prepare(
     })
 }
 
+fn git_finding(path: &RepoPath, e: impl std::fmt::Display) -> Finding {
+    Finding::error("git", None, Some(path), e.to_string())
+}
+
 /// The planned write of one touched path; `None` when it ends as it began (absent).
 fn planned(
     ws: &Workspace,
@@ -139,18 +193,35 @@ fn planned(
     let on_disk = ws
         .read(&path)
         .map_err(|e| Finding::workspace(&e, None, Some(&path)))?;
-    let (post, post_blob) = match tree.state(&path, None)?.clone() {
-        State::Present { blob, .. } => (PostImage::Bytes(tree.bytes(&path, None)?), Some(blob)),
+    let (post, post_blob, mode) = match tree.state(&path, None)?.clone() {
+        State::Present { blob, mode, .. } => (
+            PostImage::Bytes(tree.bytes(&path, None)?),
+            Some(blob),
+            Some(mode),
+        ),
         _ if on_disk.is_none() => return Ok(None),
-        _ => (PostImage::Delete, None),
+        _ => (PostImage::Delete, None, None),
     };
-    let pre = match &on_disk {
-        None => PreImage::Absent,
-        Some(bytes) => PreImage::Stored(
-            ws.git()
+    let git = ws.git();
+    let (pre, pre_mode) = match &on_disk {
+        None => (PreImage::Absent, None),
+        Some(bytes) => {
+            let id = git
                 .hash_object(&path, bytes, false, true, &[])
-                .map_err(|e| Finding::error("git", None, Some(&path), e.to_string()))?,
+                .map_err(|e| git_finding(&path, e))?;
+            let mode = git
+                .index_entry(&path, &[])
+                .map_err(|e| git_finding(&path, e))?
+                .map_or_else(|| "100644".to_owned(), |(mode, _)| mode);
+            (PreImage::Stored(id), Some(mode))
+        }
+    };
+    let post_raw = match &post {
+        PostImage::Bytes(bytes) => Some(
+            git.hash_object(&path, bytes, false, false, &[])
+                .map_err(|e| git_finding(&path, e))?,
         ),
+        PostImage::Delete => None,
     };
     let resolved = ws
         .resolve(&path)
@@ -160,16 +231,23 @@ fn planned(
         .map(|i| segments[..=i].join("/"))
         .collect();
     Ok(Some(PlannedWrite {
-        path,
-        pre,
+        record: WriteRecord {
+            path,
+            pre,
+            pre_mode,
+            post: post_raw,
+            post_blob,
+            mode,
+            creates_dirs,
+            temp: temp_name(),
+        },
         post,
-        post_blob,
-        creates_dirs,
     }))
 }
 
-/// Writes `step`: first the write-ahead record, then every path in order, then the index
-/// removals in one call. On an error every path already written is restored from its
+/// Writes `step`. First the disk is checked against what prepare saw (D78): a difference
+/// writes and records nothing. Then the write-ahead record, then every path in order, then the
+/// index removals in one call. On an error every path already written is restored from its
 /// pre-image, created files and directories are removed, and the error names what could not be
 /// restored.
 pub fn commit(
@@ -177,20 +255,32 @@ pub fn commit(
     step: &PreparedStep,
     ahead: &mut dyn WriteAhead,
 ) -> Result<(), CommitError> {
-    ahead.record(step).map_err(|e| CommitError {
-        cause: CommitCause::Record(e),
+    let fail = |cause| CommitError {
+        cause,
         unrestored: Vec::new(),
-    })?;
+    };
+    let changed = changed_since_prepare(ws, step).map_err(|e| fail(e.into()))?;
+    if !changed.is_empty() {
+        return Err(fail(CommitCause::Changed(changed)));
+    }
+    ahead
+        .record(step)
+        .map_err(|e| fail(CommitCause::Record(e)))?;
     let mut done: Vec<(&PlannedWrite, Vec<PathBuf>)> = Vec::new();
     for write in &step.writes {
+        let record = &write.record;
         let result = match &write.post {
-            PostImage::Bytes(bytes) => ws.write(&write.path, bytes),
-            PostImage::Delete => ws.remove(&write.path).map(|_| Vec::new()),
+            PostImage::Bytes(bytes) => {
+                let executable = record.mode.as_deref().map(|m| m == "100755");
+                ws.write_as(&record.path, bytes, &record.temp, executable)
+            }
+            PostImage::Delete => ws.remove(&record.path).map(|_| Vec::new()),
         };
         match result {
             Ok(dirs) => done.push((write, dirs)),
             Err(e) => return Err(rollback(ws, &done, e)),
         }
+        ahead.after_write(done.len());
     }
     let removals: Vec<&RepoPath> = step.index_removals.iter().collect();
     if let Err(e) = ws.git().index_remove(&removals, &[]) {
@@ -199,6 +289,42 @@ pub fn commit(
     Ok(())
 }
 
+/// The paths whose disk state is no longer the pre-image prepare stored, and those whose
+/// directories to create appeared: stat first (absent stays absent, present must be a file),
+/// then one `hash-object --stdin-paths` over the present ones.
+fn changed_since_prepare(
+    ws: &Workspace,
+    step: &PreparedStep,
+) -> Result<Vec<RepoPath>, WorkspaceError> {
+    let mut changed = Vec::new();
+    let mut to_hash: Vec<(&RepoPath, &BlobId)> = Vec::new();
+    for write in &step.writes {
+        let record = &write.record;
+        let resolved = ws.resolve(&record.path)?;
+        let present = resolved.exists() && !resolved.is_dir;
+        let dir_appeared = record
+            .creates_dirs
+            .iter()
+            .any(|d| ws.root().join(d).exists());
+        match &record.pre {
+            PreImage::Absent if present || resolved.is_dir || dir_appeared => {
+                changed.push(record.path.clone());
+            }
+            PreImage::Absent => {}
+            PreImage::Stored(_) if !present => changed.push(record.path.clone()),
+            PreImage::Stored(id) => to_hash.push((&record.path, id)),
+        }
+    }
+    let paths: Vec<&RepoPath> = to_hash.iter().map(|(p, _)| *p).collect();
+    let ids = ws.git().hash_paths(&paths)?;
+    for ((path, pre), now) in to_hash.iter().zip(ids) {
+        if **pre != now {
+            changed.push((*path).clone());
+        }
+    }
+    Ok(changed)
+}
+
 /// Restores `done` in reverse order and reports what it could not restore.
 fn rollback(
     ws: &Workspace,
@@ -207,8 +333,8 @@ fn rollback(
 ) -> CommitError {
     let mut unrestored = Vec::new();
     for (write, dirs) in done.iter().rev() {
-        if restore(ws, write).is_err() {
-            unrestored.push(write.path.clone());
+        if restore(ws, &write.record).is_err() {
+            unrestored.push(write.record.path.clone());
         }
         remove_dirs(dirs);
     }
@@ -218,12 +344,16 @@ fn rollback(
     }
 }
 
-fn restore(ws: &Workspace, write: &PlannedWrite) -> Result<(), WorkspaceError> {
-    match &write.pre {
-        PreImage::Absent => ws.remove(&write.path).map(|_| ()),
+/// Puts one path back as it was before the step: the pre-image's exact bytes and mode, or no
+/// file at all.
+pub(crate) fn restore(ws: &Workspace, record: &WriteRecord) -> Result<(), WorkspaceError> {
+    match &record.pre {
+        PreImage::Absent => ws.remove(&record.path).map(|_| ()),
         PreImage::Stored(id) => {
             let bytes = ws.git().cat_blob(id, &[])?;
-            ws.write(&write.path, &bytes).map(|_| ())
+            let executable = record.pre_mode.as_deref().map(|m| m == "100755");
+            ws.write_as(&record.path, &bytes, &temp_name(), executable)
+                .map(|_| ())
         }
     }
 }
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t4-transaction-rs.patch"
Remove-Item "$env:TEMP\p5-t4-transaction-rs.patch"
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
    assert_eq!(step.writes[0].record.creates_dirs, ["docs", "docs/a"]);
    assert_eq!(step.writes[0].record.pre, PreImage::Absent);
}
```

File `crates/planrun-exec/src/workspace.rs` (exact diff; save the block as `$env:TEMP\p5-t4-workspace-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/workspace.rs b/crates/planrun-exec/src/workspace.rs
index 0e137cb..794c709 100644
--- a/crates/planrun-exec/src/workspace.rs
+++ b/crates/planrun-exec/src/workspace.rs
@@ -88,6 +88,7 @@ pub struct Workspace {
     root: PathBuf,
     git: Git,
     ignorecase: bool,
+    filemode: bool,
     #[cfg(test)]
     pub(crate) faults: Faults,
 }
@@ -106,10 +107,12 @@ impl Workspace {
             return Err(WorkspaceError::ObjectFormat(format));
         }
         let ignorecase = git.config_bool("core.ignorecase")?.unwrap_or(false);
+        let filemode = git.config_bool("core.filemode")?.unwrap_or(cfg!(unix));
         Ok(Self {
             root,
             git,
             ignorecase,
+            filemode,
             #[cfg(test)]
             faults: Faults::default(),
         })
@@ -130,6 +133,26 @@ impl Workspace {
         self.ignorecase
     }
 
+    /// `core.filemode` (unset: true on Unix, false elsewhere): the executable bit counts (D78).
+    /// Always false on Windows, which has no executable bit.
+    pub fn filemode(&self) -> bool {
+        self.filemode && cfg!(unix)
+    }
+
+    /// Whether the file at `full` has its executable bit set; false on Windows.
+    pub fn is_executable(full: &Path) -> bool {
+        #[cfg(unix)]
+        {
+            use std::os::unix::fs::PermissionsExt;
+            fs::metadata(full).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
+        }
+        #[cfg(not(unix))]
+        {
+            let _ = full;
+            false
+        }
+    }
+
     /// Resolves `path`: every existing segment must be a real directory (the last one may be a
     /// file), never a link, and the existing part must canonicalize to the root plus the same
     /// segments, compared exactly. That refuses symlinks, junctions, 8.3 short names and case
@@ -222,12 +245,26 @@ impl Workspace {
     /// needs one segment at a time, each checked. Returns the directories it created, deepest
     /// last. On an error nothing of this write remains: no temp file, no new directory.
     pub fn write(&self, path: &RepoPath, bytes: &[u8]) -> Result<Vec<PathBuf>, WorkspaceError> {
+        self.write_as(path, bytes, &temp_name(), None)
+    }
+
+    /// [`Workspace::write`] with the temp file's name chosen beforehand (a step transaction
+    /// journals it, D77) and, under `core.filemode`, the executable bit set or cleared on the
+    /// temp file before the rename (`Some(true)` for mode 100755, D78).
+    pub fn write_as(
+        &self,
+        path: &RepoPath,
+        bytes: &[u8],
+        temp: &str,
+        executable: Option<bool>,
+    ) -> Result<Vec<PathBuf>, WorkspaceError> {
         let resolved = self.resolve(path)?;
         if resolved.is_dir {
             return Err(unsafe_path(path, "it is a directory"));
         }
         let created = self.make_dirs(path, &resolved)?;
-        if let Err(e) = self.replace_file(path, &resolved.full, bytes) {
+        let executable = executable.filter(|_| self.filemode());
+        if let Err(e) = self.replace_file(path, &resolved.full, bytes, temp, executable) {
             remove_dirs(&created);
             return Err(e);
         }
@@ -284,10 +321,12 @@ impl Workspace {
         path: &RepoPath,
         full: &Path,
         bytes: &[u8],
+        temp: &str,
+        executable: Option<bool>,
     ) -> Result<(), WorkspaceError> {
         let parent = full.parent().unwrap_or(&self.root);
-        let temp = parent.join(format!(".planrun-{:016x}.tmp", random()));
-        if let Err(e) = write_new(&temp, bytes) {
+        let temp = parent.join(temp);
+        if let Err(e) = write_new(&temp, bytes).and_then(|()| set_executable(&temp, executable)) {
             let _ = fs::remove_file(&temp);
             return Err(io_error(path, e));
         }
@@ -353,6 +392,42 @@ impl Faults {
     }
 }
 
+/// A fresh name for a writer's temp file: `.planrun-<16 hex>.tmp`.
+pub fn temp_name() -> String {
+    format!(".planrun-{:016x}.tmp", random())
+}
+
+/// True for a name [`temp_name`] makes.
+pub fn is_temp_name(name: &str) -> bool {
+    name.strip_prefix(".planrun-")
+        .and_then(|r| r.strip_suffix(".tmp"))
+        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
+}
+
+/// Sets or clears the executable bits (for every class that may read the file); nothing when
+/// `executable` is `None` or on Windows.
+fn set_executable(file: &Path, executable: Option<bool>) -> std::io::Result<()> {
+    let Some(executable) = executable else {
+        return Ok(());
+    };
+    #[cfg(unix)]
+    {
+        use std::os::unix::fs::PermissionsExt;
+        let mut permissions = fs::metadata(file)?.permissions();
+        let mode = permissions.mode();
+        let mode = if executable {
+            mode | ((mode & 0o444) >> 2)
+        } else {
+            mode & !0o111
+        };
+        permissions.set_mode(mode);
+        fs::set_permissions(file, permissions)?;
+    }
+    #[cfg(not(unix))]
+    let _ = (file, executable);
+    Ok(())
+}
+
 fn write_new(temp: &Path, bytes: &[u8]) -> std::io::Result<()> {
     let mut file = fs::OpenOptions::new()
         .write(true)
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t4-workspace-rs.patch"
Remove-Item "$env:TEMP\p5-t4-workspace-rs.patch"
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
fn a_real_preflight_refuses_a_git_delete_of_an_untracked_file() {
    let f = Fixture::greeter(false);
    f.write("notes.txt", b"untracked\n");
    let mut ir = Ir::default();
    let steps = vec![vec![ir.delete("notes.txt", true)]];
    let tasks = vec![ir.task("T1", Manifest::default(), steps)];
    let plan = ir.plan(tasks);
    let found = preflight(&open(&f), &plan, &plan.tasks[0]).unwrap_or_else(|e| panic!("{e}"));
    let codes: Vec<&str> = found.iter().map(|f| f.code).collect();
    assert!(codes.contains(&"delete-untracked"), "{codes:?}");
    assert!(!labels(&run(&f, &plan), 0).contains(&"delete-untracked".to_owned()));
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

Targeted: planrun-exec's unit tests:

```powershell
cargo test -p planrun-exec --lib
```

Expected (exit code 0; summary lines):

```text
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-exec/src/done.rs crates/planrun-exec/src/done_tests.rs crates/planrun-exec/src/git.rs crates/planrun-exec/src/lib.rs crates/planrun-exec/src/preflight.rs crates/planrun-exec/src/transaction.rs crates/planrun-exec/src/transaction_tests.rs crates/planrun-exec/src/workspace.rs crates/planrun-exec/tests/preflight.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/done.rs' = 'cb00994d7a692fc9f765a14c0c4328373db74f16'
      'crates/planrun-exec/src/done_tests.rs' = '90c173a59bfccd569c210bc8b6bc52c95aa253a7'
      'crates/planrun-exec/src/git.rs' = 'a1f3754302e102522ea80e48488f30588c035df6'
      'crates/planrun-exec/src/lib.rs' = '1a14a909efe2545df6392029cca3cd2c7f73ef07'
      'crates/planrun-exec/src/preflight.rs' = '68eac1455965515c8dd37b4232a60187f4eb3b86'
      'crates/planrun-exec/src/transaction.rs' = 'c8f415069060a2e2d092db1a6c8bfcb169622eeb'
      'crates/planrun-exec/src/transaction_tests.rs' = '6e8a4f7adcfd57ccf3aa32cd328a59a8df963fd8'
      'crates/planrun-exec/src/workspace.rs' = '794c709285e7187181b5bd4e5c79c3ef993f098c'
      'crates/planrun-exec/tests/preflight.rs' = 'c2bc3316b9c2138c2e370547004a1b5516ce96f6'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P5-T4: the step record, the disk checked again before commit, modes, temp names and the step's state on disk" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P5-T5: the journal, the run lock, HEAD and batch hashes from git

**Goal:** The journal (D76) and the lock (D77). `journal.rs`: the records, one `write_all` per line with the schema version and the time, `sync_data` after `step-prepared`, `action-start` and `run-end`, the reader that drops a torn last line and refuses a bad one in the middle, the writer that cuts a torn tail before appending, the `WriteAhead` for a step, and the observer the e2e fault seam uses. `lock.rs`: `File::try_lock` on `<git-dir>/planrun/lock`. git gains `git_dir` and `head` (`Head`).

**Files**

- Modify `crates/planrun-exec/src/done.rs`
- Modify `crates/planrun-exec/src/git.rs`
- Create `crates/planrun-exec/src/journal.rs`
- Create `crates/planrun-exec/src/journal_tests.rs`
- Modify `crates/planrun-exec/src/lib.rs`
- Create `crates/planrun-exec/src/lock.rs`
- Modify `crates/planrun-exec/src/transaction.rs`
- Modify `crates/planrun-exec/tests/git.rs`

Over 300 lines, so they come as exact diffs: `crates/planrun-exec/src/git.rs`, `crates/planrun-exec/src/transaction.rs`.

**Step 1 — write the files**

File `crates/planrun-exec/src/done.rs` (full contents):

```rust
//! A journaled step's state on the disk, for resume (decisions D14, D77, D78): whether its
//! writes are all there (Done), none of them are (NotDone), or anything else (Unknown, the user
//! decides); whether its pre-images survive to restore it; and the writer's temp files it may
//! have left behind.

use std::fs;

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
    let mut removed = Vec::new();
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
        let is_file = fs::symlink_metadata(&resolved.full).is_ok_and(|m| m.is_file());
        if is_file && fs::remove_file(&resolved.full).is_ok() {
            removed.push(rel);
        }
    }
    removed
}

#[cfg(test)]
#[path = "done_tests.rs"]
mod tests;
```

File `crates/planrun-exec/src/git.rs` (exact diff; save the block as `$env:TEMP\p5-t5-git-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/git.rs b/crates/planrun-exec/src/git.rs
index a1f3754..8e49192 100644
--- a/crates/planrun-exec/src/git.rs
+++ b/crates/planrun-exec/src/git.rs
@@ -56,6 +56,16 @@ impl<'de> serde::Deserialize<'de> for BlobId {
     }
 }
 
+/// HEAD as git reports it: the commit (none in an empty repository) and the ref it points to
+/// (none when detached).
+#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
+pub struct Head {
+    /// The commit id.
+    pub commit: Option<String>,
+    /// `refs/heads/main`.
+    pub reference: Option<String>,
+}
+
 /// Why a git call failed.
 #[derive(Debug, thiserror::Error)]
 pub enum GitError {
@@ -174,6 +184,26 @@ impl Git {
         Ok(text(&self.run(&all, None, &[])?).trim().to_owned())
     }
 
+    /// The absolute path of this work tree's git directory (`--absolute-git-dir`; a linked
+    /// worktree has its own).
+    pub fn git_dir(&self) -> Result<PathBuf, GitError> {
+        Ok(PathBuf::from(self.rev_parse(&["--absolute-git-dir"])?))
+    }
+
+    /// HEAD: `rev-parse -q --verify HEAD` (none when unborn) and `symbolic-ref -q HEAD` (none
+    /// when detached).
+    pub fn head(&self) -> Result<Head, GitError> {
+        let optional = |args: &[&str]| match self.run(args, None, &[]) {
+            Ok(out) => Ok(Some(text(&out).trim().to_owned())),
+            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
+            Err(e) => Err(e),
+        };
+        Ok(Head {
+            commit: optional(&["rev-parse", "-q", "--verify", "HEAD"])?,
+            reference: optional(&["symbolic-ref", "-q", "HEAD"])?,
+        })
+    }
+
     /// The absolute path of the repository's object directory.
     pub fn object_dir(&self) -> Result<PathBuf, GitError> {
         let objects = PathBuf::from(self.rev_parse(&["--git-path", "objects"])?);
@@ -236,22 +266,27 @@ impl Git {
         blob_id(&args, &out)
     }
 
-    /// The exact-bytes blob ids (`--no-filters`) of the files at `paths`, in one
-    /// `hash-object --stdin-paths` call, nothing stored. Every path must exist: git fails the
-    /// whole batch on a missing one, so callers stat first.
-    pub fn hash_paths(&self, paths: &[&RepoPath]) -> Result<Vec<BlobId>, GitError> {
+    /// The blob ids of the files at `paths`, in one `hash-object --stdin-paths` call, nothing
+    /// stored: git's view with `filters` (what a commit would hold), the exact bytes
+    /// (`--no-filters`) without. Every path must exist: git fails the whole batch on a missing
+    /// one, so callers stat first.
+    pub fn hash_paths(&self, paths: &[&RepoPath], filters: bool) -> Result<Vec<BlobId>, GitError> {
         if paths.is_empty() {
             return Ok(Vec::new());
         }
-        let args = ["hash-object", "--no-filters", "--stdin-paths"];
+        let args: &[&str] = if filters {
+            &["hash-object", "--stdin-paths"]
+        } else {
+            &["hash-object", "--no-filters", "--stdin-paths"]
+        };
         let input: String = paths.iter().map(|p| format!("{p}\n")).collect();
-        let out = self.run(&args, Some(input.as_bytes()), &[])?;
+        let out = self.run(args, Some(input.as_bytes()), &[])?;
         let ids: Vec<BlobId> = text(&out)
             .lines()
-            .map(|line| BlobId::parse(line).ok_or_else(|| output_error(&args, line)))
+            .map(|line| BlobId::parse(line).ok_or_else(|| output_error(args, line)))
             .collect::<Result<_, _>>()?;
         if ids.len() != paths.len() {
-            return Err(output_error(&args, &text(&out)));
+            return Err(output_error(args, &text(&out)));
         }
         Ok(ids)
     }
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t5-git-rs.patch"
Remove-Item "$env:TEMP\p5-t5-git-rs.patch"
```

File `crates/planrun-exec/src/journal.rs` (full contents):

```rust
//! The journal (docs/architecture.md §6.6; decisions D18, D76): one JSONL file per plan at
//! `<git-dir>/planrun/<plan file stem>.jsonl`, one record per line, appended by one writer.
//!
//! - Every line carries the schema version and the time. One `write_all` per record (the JSON
//!   and its `\n`); no buffer spans two records.
//! - `step-prepared`, every `action-start` and `run-end` are followed by `sync_data`: they are
//!   the records whose loss after a power cut would change what resume does. The rest reach the
//!   OS only, which a killed process cannot lose.
//! - A line counts only when it ends in `\n`: a last line without one is dropped, even if it
//!   parses, and before the first append the file is cut back to its last `\n`. A bad line that
//!   is not the last one is an error, never skipped.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use planrun_model::RepoPath;
use serde::{Deserialize, Serialize};

use crate::git::BlobId;
pub use crate::git::Head;
use crate::transaction::{PreparedStep, StepRecord, WriteAhead};

/// The schema version every line carries.
pub const SCHEMA: u32 = 1;

/// Where an action is: the task's id, the step's and the action's positions from 0.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct At {
    /// The task id (`P1-T2`).
    pub task: String,
    /// The step's position in the task.
    pub step: usize,
    /// The action's position in the step (for a file group: its first action).
    pub action: usize,
}

/// A path and the blob id git would commit for it (`None`: absent).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathBlob {
    /// The path.
    pub path: RepoPath,
    /// Its filtered blob id on disk, or `None` when it is absent.
    pub blob: Option<BlobId>,
}

/// How an action ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    /// Everything stated was checked and holds.
    Passed,
    /// The user confirmed what prose or comments say (or an unchecked exit).
    Confirmed,
    /// The user answered no to a confirmation.
    Refused,
    /// The user accepted a mismatch.
    Accepted,
    /// It failed and the user stopped, or retried (a later attempt follows).
    Failed,
    /// Skipped by policy (D67).
    SkippedPolicy,
    /// Skipped by the user.
    SkippedUser,
    /// A Manual, acknowledged.
    Acknowledged,
    /// A `Generated` path exists.
    Present,
    /// A `Generated` path is missing.
    Missing,
    /// A script that ran before an interruption, kept without running it again (resume, D77).
    Kept,
}

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum End {
    /// Every task of the run is done.
    Completed,
    /// It stopped: the user quit, a step failed, a check stopped it.
    Stopped,
}

/// One journal record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "kebab-case")]
pub enum Record {
    /// A run or a resume starts.
    Run {
        /// The plan's path: repository-relative, or canonical when outside the repository.
        plan: String,
        /// The SHA-256 of the plan's bytes.
        fingerprint: String,
        /// The format id.
        format: String,
        /// The repository root.
        root: String,
        /// HEAD at the start.
        head: Head,
        /// The `--auto` classes.
        auto: Vec<String>,
        /// `--run-expected-failures`.
        run_expected_failures: bool,
        /// `--from`.
        from: Option<String>,
        /// `--to`.
        to: Option<String>,
        /// True for `resume`.
        resume: bool,
        /// planrun's version.
        planrun: String,
    },
    /// A task starts.
    TaskStart {
        /// Its id.
        task: String,
        /// HEAD.
        head: Head,
    },
    /// The write-ahead record of a step's file actions (D53), before any is written.
    StepPrepared {
        /// The group's first action.
        at: At,
        /// What will be written.
        step: StepRecord,
    },
    /// Every write of the step is done.
    StepWritten {
        /// The group.
        at: At,
    },
    /// The step's commit failed and was rolled back; `unrestored` remain changed.
    StepFailed {
        /// The group.
        at: At,
        /// The error.
        error: String,
        /// Paths rollback could not restore.
        unrestored: Vec<RepoPath>,
    },
    /// Resume found the step done on disk; nothing is written again (D77).
    StepVerified {
        /// The group.
        at: At,
    },
    /// Resume put the step back from its pre-images (`[r]`).
    StepRestored {
        /// The group.
        at: At,
    },
    /// The user accepted an Unknown step as done (`[a]`, a typed `yes`).
    StepAccepted {
        /// The group.
        at: At,
    },
    /// A script, commit, Manual or Generated check starts.
    ActionStart {
        /// Where.
        at: At,
        /// `script`, `commit`, `manual`, `generated`.
        kind: String,
        /// The policy's decision (`Auto` included).
        decision: String,
        /// HEAD before it.
        head: Head,
        /// For a Commit: the task's paths as they are on disk now, which the commit must hold.
        expect: Vec<PathBlob>,
    },
    /// It ended.
    ActionEnd {
        /// Where.
        at: At,
        /// How.
        outcome: Outcome,
        /// The block's status (`Exited(0)`), for a script or commit.
        status: Option<String>,
        /// pwsh's `errored`.
        errored: bool,
        /// The session's working directory after it.
        cwd: Option<String>,
        /// The last lines of its output.
        tail: Vec<String>,
        /// HEAD after it.
        head: Head,
    },
    /// The user answered a question.
    Answer {
        /// The action it was about, if any.
        at: Option<At>,
        /// Which question (`confirm`, `mismatch`, `dirt`, `outside`, `session`, `gate`, …).
        question: String,
        /// The answer.
        answer: String,
    },
    /// The user opened the commit gate with a typed `yes` (D75).
    GateOverride {
        /// The task.
        task: String,
        /// Every reason the gate was closed.
        reasons: Vec<String>,
    },
    /// HEAD as read after something that may move it (`[o]`, D80).
    HeadSeen {
        /// HEAD.
        head: Head,
    },
    /// A shell session started.
    Session {
        /// `pwsh` or `bash`.
        shell: String,
        /// `started` or `restarted`.
        reason: String,
    },
    /// A task is done.
    TaskEnd {
        /// Its id.
        task: String,
        /// HEAD.
        head: Head,
    },
    /// The run ends.
    RunEnd {
        /// How.
        end: End,
        /// Why it stopped.
        reason: Option<String>,
    },
}

impl Record {
    /// Whether the record is followed by `sync_data` (D76).
    fn durable(&self) -> bool {
        matches!(
            self,
            Record::StepPrepared { .. } | Record::ActionStart { .. } | Record::RunEnd { .. }
        )
    }
}

/// One line as stored: the schema version, the time, the record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    /// The schema version.
    pub v: u32,
    /// Milliseconds since the Unix epoch.
    pub ms: u64,
    /// The record.
    #[serde(flatten)]
    pub record: Record,
}

/// Why a journal could not be read.
#[derive(Debug, thiserror::Error)]
pub enum JournalError {
    /// Reading the file failed.
    #[error("{path}: {source}")]
    Io {
        /// The journal.
        path: String,
        /// The error.
        #[source]
        source: io::Error,
    },
    /// A line that is not the last one is not a record of this schema.
    #[error("{path}:{line}: not a journal record: {reason}")]
    Corrupt {
        /// The journal.
        path: String,
        /// Its 1-based line number.
        line: usize,
        /// What is wrong.
        reason: String,
    },
}

/// The journal's directory: `<git-dir>/planrun`.
pub fn dir(git_dir: &Path) -> PathBuf {
    git_dir.join("planrun")
}

/// The journal of the plan whose file stem is `stem`.
pub fn path_for(git_dir: &Path, stem: &str) -> PathBuf {
    dir(git_dir).join(format!("{stem}.jsonl"))
}

/// Reads every complete line of the journal at `path`; an absent file is empty. A last line
/// without its `\n` is dropped (a torn write); any other bad line is an error.
pub fn read(path: &Path) -> Result<Vec<Line>, JournalError> {
    let name = path.display().to_string();
    let io = |source| JournalError::Io {
        path: name.clone(),
        source,
    };
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(io(e)),
    };
    let complete = match bytes.iter().rposition(|&b| b == b'\n') {
        Some(end) => &bytes[..=end],
        None => &[][..],
    };
    let mut lines = Vec::new();
    for (i, raw) in complete.split_inclusive(|&b| b == b'\n').enumerate() {
        let raw = &raw[..raw.len() - 1];
        let corrupt = |reason: String| JournalError::Corrupt {
            path: name.clone(),
            line: i + 1,
            reason,
        };
        let line: Line = serde_json::from_slice(raw).map_err(|e| corrupt(e.to_string()))?;
        if line.v != SCHEMA {
            return Err(corrupt(format!(
                "schema {} (this planrun reads {SCHEMA})",
                line.v
            )));
        }
        lines.push(line);
    }
    Ok(lines)
}

/// What a journal observer sees: a record that is now with the OS (and synced, when durable),
/// or the number of paths a step's commit has written so far.
#[derive(Debug)]
pub enum Seam<'a> {
    /// A record was appended.
    Appended(&'a Record),
    /// A step's commit wrote its n-th path.
    Wrote(usize),
}

/// A callback that sees every [`Seam`]: planrun-cli's fault seam for its e2e tests (D82).
pub type Observer = Box<dyn FnMut(Seam<'_>)>;

/// The appending writer.
pub struct Journal {
    file: File,
    path: PathBuf,
    observer: Option<Observer>,
}

impl Journal {
    /// Opens the journal at `path` for appending, creating it and its directory. A torn last
    /// line is cut off first, so the next record starts on a line of its own.
    pub fn open(path: &Path) -> io::Result<Journal> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let keep = bytes.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
        if keep != bytes.len() {
            file.set_len(keep as u64)?;
            file.sync_data()?;
        }
        file.seek(SeekFrom::End(0))?;
        Ok(Journal {
            file,
            path: path.to_path_buf(),
            observer: None,
        })
    }

    /// Calls `observer` after every append and every write of a step's commit.
    pub fn observe(&mut self, observer: Observer) {
        self.observer = Some(observer);
    }

    fn seam(&mut self, seam: Seam<'_>) {
        if let Some(observer) = self.observer.as_mut() {
            observer(seam);
        }
    }

    /// The journal's path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends `record` as one line in one write; durable records are synced (D76).
    pub fn append(&mut self, record: Record) -> io::Result<()> {
        let durable = record.durable();
        let line = Line {
            v: SCHEMA,
            ms: now_ms(),
            record,
        };
        let mut bytes = serde_json::to_vec(&line).map_err(io::Error::other)?;
        bytes.push(b'\n');
        self.file.write_all(&bytes)?;
        if durable {
            self.file.sync_data()?;
        }
        self.seam(Seam::Appended(&line.record));
        Ok(())
    }

    /// The write-ahead recorder for the step at `at` (D53): `record` appends `step-prepared`.
    pub fn ahead<'j>(&'j mut self, at: &'j At) -> StepAhead<'j> {
        StepAhead { journal: self, at }
    }
}

/// [`WriteAhead`] over the journal, for one step.
pub struct StepAhead<'j> {
    journal: &'j mut Journal,
    at: &'j At,
}

impl WriteAhead for StepAhead<'_> {
    fn record(&mut self, step: &PreparedStep) -> io::Result<()> {
        self.journal.append(Record::StepPrepared {
            at: self.at.clone(),
            step: step.record(),
        })
    }

    fn after_write(&mut self, written: usize) {
        self.journal.seam(Seam::Wrote(written));
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

#[cfg(test)]
#[path = "journal_tests.rs"]
mod tests;
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
```

File `crates/planrun-exec/src/lib.rs` (full contents):

```rust
//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Modules (docs/roadmap.md; P3 built the first eight, P4 the next three):
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
//! - [`transaction`]: a step's file actions as prepare, then commit, with rollback (D53); the
//!   disk is checked again between the two (D78).
//! - [`done`]: a journaled step's state on the disk, for resume (D14, D77).
//! - [`preflight`]: task preflight and the dry run over one virtual tree (D50, D55).
//! - [`finding`]: what preflight, a simulation and the dry run report.
//! - [`shell`]: persistent pwsh and bash sessions behind [`ShellSession`] (D60–D65).
//! - [`expect`]: a run against its expectation: exit codes, summary lines with `…`, what asks
//!   for confirmation (D11, D66).
//! - [`policy`]: command classes and the [`CommandClassifier`]: a built-in allowlist over a
//!   splitter per shell (D13, D68).
//! - [`journal`]: one JSONL file per plan under `<git-dir>/planrun/`, the write-ahead recorder
//!   (D18, D76).
//! - [`lock`]: one planrun per repository (D77).
//! - `runner`: the state machine Task → Step → Action, talking to a [`Ui`].

pub mod done;
pub mod eol;
pub mod expect;
pub mod finding;
pub mod git;
pub mod handler;
pub mod journal;
pub mod lock;
pub mod policy;
pub mod preflight;
pub mod shell;
pub mod transaction;
pub mod tree;
pub mod workspace;

pub use handler::{ActionHandler, HandlerSet};
pub use policy::{CommandClassifier, PolicyClass};
pub use shell::{ScriptRun, ShellSession};

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "../tests/support/ir.rs"]
mod ir;

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

File `crates/planrun-exec/src/lock.rs` (full contents):

```rust
//! One planrun per repository (decision D77): `<git-dir>/planrun/lock`, held with
//! `File::try_lock` for the whole run, prompts included, by every plan's run and resume. The
//! operating system releases it when the process ends, however it ends, so there is no stale
//! lock. The holder's process id goes into `lock.pid` for the message only: on Windows a locked
//! file cannot be read by another process.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::Path;

/// The held lock; dropping it releases it.
#[derive(Debug)]
pub struct RunLock {
    _file: File,
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

/// Takes the lock in `dir` (`<git-dir>/planrun`), creating the directory, or says who holds it.
pub fn acquire(dir: &Path) -> Result<RunLock, LockError> {
    fs::create_dir_all(dir)?;
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join("lock"))?;
    match file.try_lock() {
        Ok(()) => {
            fs::write(dir.join("lock.pid"), std::process::id().to_string())?;
            Ok(RunLock { _file: file })
        }
        Err(TryLockError::WouldBlock) => {
            let pid = fs::read_to_string(dir.join("lock.pid"))
                .ok()
                .and_then(|s| s.trim().parse().ok());
            Err(LockError::Held { pid })
        }
        Err(TryLockError::Error(e)) => Err(LockError::Io(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_holder_is_refused_until_the_first_lets_go() {
        let dir = std::env::temp_dir().join(format!("planrun-lock-{}", std::process::id()));
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
}
```

File `crates/planrun-exec/src/transaction.rs` (exact diff; save the block as `$env:TEMP\p5-t5-transaction-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/transaction.rs b/crates/planrun-exec/src/transaction.rs
index c8f4150..9029292 100644
--- a/crates/planrun-exec/src/transaction.rs
+++ b/crates/planrun-exec/src/transaction.rs
@@ -316,7 +316,7 @@ fn changed_since_prepare(
         }
     }
     let paths: Vec<&RepoPath> = to_hash.iter().map(|(p, _)| *p).collect();
-    let ids = ws.git().hash_paths(&paths)?;
+    let ids = ws.git().hash_paths(&paths, false)?;
     for ((path, pre), now) in to_hash.iter().zip(ids) {
         if **pre != now {
             changed.push((*path).clone());
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t5-transaction-rs.patch"
Remove-Item "$env:TEMP\p5-t5-transaction-rs.patch"
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

#[test]
fn head_git_dir_and_hashes_of_paths() {
    let f = Fixture::greeter(false);
    let g = git(&f);
    let head = g.head().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(head.reference.as_deref(), Some("refs/heads/main"));
    assert_eq!(head.commit.map(|c| c.len()), Some(40));
    f.git(&["checkout", "-q", "--detach"]);
    assert_eq!(g.head().unwrap_or_else(|e| panic!("{e}")).reference, None);
    let dir = g.git_dir().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(canonical(&dir), canonical(&f.path().join(".git")));
    let ids = g
        .hash_paths(&[&path("src/lib.rs"), &path("src/old.rs")], false)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(ids[0].matches(&abbrev("aa80cf1")));
    assert_eq!(ids.len(), 2);
    let missing = BlobId::parse(&"2".repeat(40)).unwrap_or_else(|| panic!("hex"));
    let found = g
        .missing_objects(&[ids[0], missing])
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(found, [missing]);
}
```

**Step 2 — verify**

Targeted: planrun-exec's unit tests:

```powershell
cargo test -p planrun-exec --lib
```

Expected (exit code 0; summary lines):

```text
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-exec/src/done.rs crates/planrun-exec/src/git.rs crates/planrun-exec/src/journal.rs crates/planrun-exec/src/journal_tests.rs crates/planrun-exec/src/lib.rs crates/planrun-exec/src/lock.rs crates/planrun-exec/src/transaction.rs crates/planrun-exec/tests/git.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/done.rs' = 'bcc45915eff7793eb30ec8390f2452f7df5775b5'
      'crates/planrun-exec/src/git.rs' = '8e49192f7cf1a80c505c4bb8daf7d60eb9c16298'
      'crates/planrun-exec/src/journal.rs' = 'ffd22de6de3c253d601ff452364caada8a75677f'
      'crates/planrun-exec/src/journal_tests.rs' = '289e82a082067a6375492daa324f7da2d9a061e4'
      'crates/planrun-exec/src/lib.rs' = '9dcabcb4934086bb90d7bf15170aef32a76bb4f2'
      'crates/planrun-exec/src/lock.rs' = 'c28a1887b312ff08d028edad6ce480f9149d1992'
      'crates/planrun-exec/src/transaction.rs' = '90292921ac79eb7ebc00301d052a00bf9761b7c9'
      'crates/planrun-exec/tests/git.rs' = 'b3ee0f60e80b31719905d8ce1530452bd96055ae'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P5-T5: the journal, the run lock, HEAD and batch hashes from git" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P5-T6: the runner: tasks, step transactions with the journal, scripts in sessions, prompts and the commit gate

**Goal:** The runner (architecture §6.1). `ui.rs`: the `Ui` trait with `Question`, `Choice`, `Reply` and `Position`. `runner/`: `Runner::run` journals the run, walks the tasks (preflight, `task-start`, the steps, `task-end`), commits each step's file actions through the journal, runs each script per its decision in its shell's session (`sessions.rs`), judges it with the matcher, asks what the verdict calls for, and before a Commit checks the `Generated` paths, runs stale Gates again, asks about dirt outside the task and opens a closed gate only with a typed `yes` (`gate.rs`); a Commit is made only when HEAD moved by one commit holding the task's paths. Dead sessions, the session notice and a working directory outside the repository ask as D81 says. Tests on hand-built IR with a scripted `Ui`.

**Files**

- Modify `crates/planrun-exec/src/git.rs`
- Modify `crates/planrun-exec/src/lib.rs`
- Modify `crates/planrun-exec/src/policy/decide.rs`
- Create `crates/planrun-exec/src/runner/gate.rs`
- Create `crates/planrun-exec/src/runner/mod.rs`
- Create `crates/planrun-exec/src/runner/script.rs`
- Create `crates/planrun-exec/src/runner/sessions.rs`
- Create `crates/planrun-exec/src/runner/task.rs`
- Create `crates/planrun-exec/src/ui.rs`
- Create `crates/planrun-exec/tests/runner.rs`

Over 300 lines, so they come as exact diffs: `crates/planrun-exec/src/git.rs`.

**Step 1 — write the files**

File `crates/planrun-exec/src/git.rs` (exact diff; save the block as `$env:TEMP\p5-t6-git-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/git.rs b/crates/planrun-exec/src/git.rs
index 8e49192..0604446 100644
--- a/crates/planrun-exec/src/git.rs
+++ b/crates/planrun-exec/src/git.rs
@@ -204,6 +204,57 @@ impl Git {
         })
     }
 
+    /// The first parent of `commit`; `None` for a root commit.
+    pub fn parent(&self, commit: &str) -> Result<Option<String>, GitError> {
+        let spec = format!("{commit}^");
+        match self.run(&["rev-parse", "-q", "--verify", &spec], None, &[]) {
+            Ok(out) => Ok(Some(text(&out).trim().to_owned())),
+            Err(GitError::Failed { code: Some(1), .. }) => Ok(None),
+            Err(e) => Err(e),
+        }
+    }
+
+    /// The blob id of each of `paths` in `commit`'s tree, in order; `None` where the tree has
+    /// no file there.
+    pub fn tree_blobs(
+        &self,
+        commit: &str,
+        paths: &[&RepoPath],
+    ) -> Result<Vec<Option<BlobId>>, GitError> {
+        if paths.is_empty() {
+            return Ok(Vec::new());
+        }
+        let mut args = vec!["ls-tree", "-r", "-z", commit, "--"];
+        args.extend(paths.iter().map(|p| p.as_str()));
+        let out = text(&self.run(&args, None, &[])?).into_owned();
+        let mut found = vec![None; paths.len()];
+        for entry in out.split('\0').filter(|e| !e.is_empty()) {
+            let Some((meta, path)) = entry.split_once('\t') else {
+                return Err(output_error(&args, entry));
+            };
+            let id = meta.split_whitespace().nth(2).and_then(BlobId::parse);
+            if let Some(i) = paths.iter().position(|p| p.as_str() == path) {
+                found[i] = id;
+            }
+        }
+        Ok(found)
+    }
+
+    /// The paths that differ between the trees of `from` and `to`
+    /// (`diff-tree -r --name-only -z`).
+    pub fn changed_paths(&self, from: &str, to: &str) -> Result<Vec<String>, GitError> {
+        let out = self.run(
+            &["diff-tree", "-r", "--name-only", "-z", from, to],
+            None,
+            &[],
+        )?;
+        Ok(text(&out)
+            .split('\0')
+            .filter(|p| !p.is_empty())
+            .map(str::to_owned)
+            .collect())
+    }
+
     /// The absolute path of the repository's object directory.
     pub fn object_dir(&self) -> Result<PathBuf, GitError> {
         let objects = PathBuf::from(self.rev_parse(&["--git-path", "objects"])?);
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t6-git-rs.patch"
Remove-Item "$env:TEMP\p5-t6-git-rs.patch"
```

File `crates/planrun-exec/src/lib.rs` (full contents):

```rust
//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Modules (docs/roadmap.md; P3 built the first eight, P4 the next three):
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
//! - [`transaction`]: a step's file actions as prepare, then commit, with rollback (D53); the
//!   disk is checked again between the two (D78).
//! - [`done`]: a journaled step's state on the disk, for resume (D14, D77).
//! - [`preflight`]: task preflight and the dry run over one virtual tree (D50, D55).
//! - [`finding`]: what preflight, a simulation and the dry run report.
//! - [`shell`]: persistent pwsh and bash sessions behind [`ShellSession`] (D60–D65).
//! - [`expect`]: a run against its expectation: exit codes, summary lines with `…`, what asks
//!   for confirmation (D11, D66).
//! - [`policy`]: command classes and the [`CommandClassifier`]: a built-in allowlist over a
//!   splitter per shell (D13, D68).
//! - [`journal`]: one JSONL file per plan under `<git-dir>/planrun/`, the write-ahead recorder
//!   (D18, D76).
//! - [`lock`]: one planrun per repository (D77).
//! - [`runner`]: the state machine Task → Step → Action over the journal and the sessions,
//!   talking to a [`Ui`] (D73–D81).
//! - [`ui`]: the [`Ui`] trait and its questions (D15, D74).

pub mod done;
pub mod eol;
pub mod expect;
pub mod finding;
pub mod git;
pub mod handler;
pub mod journal;
pub mod lock;
pub mod policy;
pub mod preflight;
pub mod runner;
pub mod shell;
pub mod transaction;
pub mod tree;
pub mod ui;
pub mod workspace;

pub use handler::{ActionHandler, HandlerSet};
pub use policy::{CommandClassifier, PolicyClass};
pub use shell::{ScriptRun, ShellSession};
pub use ui::Ui;

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "../tests/support/ir.rs"]
mod ir;

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
```

File `crates/planrun-exec/src/policy/decide.rs` (full contents):

```rust
//! What happens to an action before it runs (decisions D13, D50, D67, D69). P4 computes the
//! decision; P5's runner acts on it and prompts.

use planrun_model::{Action, ActionKind, ExitExpect, PlanSource, Script, ScriptIntent, Task};

use super::{CommandClassifier, PolicyClass};

/// What the runner does with an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// It runs without asking.
    Auto,
    /// It asks first; the class that made it ask.
    Ask(PolicyClass),
    /// A file action outside the task's declared files (D50): it always asks.
    AskUnscoped,
    /// A Destructive script: shown whole, it needs a typed `yes`.
    TypedYes,
    /// A Manual: it waits until acknowledged.
    Wait,
    /// A Note: shown in place.
    Show,
    /// It does not run (D67).
    Skip(Skip),
}

/// Why an action does not run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    /// A verification expected to fail, without `--run-expected-failures` (D12, D67).
    ExpectedFailure,
}

/// A refused `--auto` value.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    /// Not a class name.
    #[error("`--auto` takes read, build and modify; `{0}` is not one of them")]
    Unknown(String),
    /// `destructive`: a Destructive script always needs a typed `yes` (D69).
    #[error("`--auto destructive` is refused: a Destructive script always needs a typed `yes`")]
    Destructive,
}

/// The run's policy: which classes run without asking, and whether expected failures run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Policy {
    auto: Vec<PolicyClass>,
    run_expected_failures: bool,
}

impl Policy {
    /// A policy; `auto` may hold Read, Build and Modify, never Destructive.
    pub fn new(auto: &[PolicyClass], run_expected_failures: bool) -> Result<Policy, PolicyError> {
        if auto.contains(&PolicyClass::Destructive) {
            return Err(PolicyError::Destructive);
        }
        Ok(Policy {
            auto: auto.to_vec(),
            run_expected_failures,
        })
    }

    /// Parses `--auto`'s value: `read,build,modify`, any of them, in any case.
    pub fn parse_auto(text: &str) -> Result<Vec<PolicyClass>, PolicyError> {
        text.split(',')
            .map(|part| match part.trim().to_ascii_lowercase().as_str() {
                "read" => Ok(PolicyClass::Read),
                "build" => Ok(PolicyClass::Build),
                "modify" => Ok(PolicyClass::Modify),
                "destructive" => Err(PolicyError::Destructive),
                _ => Err(PolicyError::Unknown(part.trim().to_owned())),
            })
            .collect()
    }

    /// The classes that run without asking (`--auto`).
    pub fn auto(&self) -> &[PolicyClass] {
        &self.auto
    }

    /// Whether Verify scripts that expect failure run (`--run-expected-failures`).
    pub fn runs_expected_failures(&self) -> bool {
        self.run_expected_failures
    }

    /// The decision for `action` of `task`; scripts are classified by `classifier`.
    pub fn decide(
        &self,
        task: &Task,
        action: &Action,
        source: &PlanSource,
        classifier: &dyn CommandClassifier,
    ) -> Decision {
        match &action.kind {
            ActionKind::Note { .. } => Decision::Show,
            ActionKind::Manual { .. } => Decision::Wait,
            ActionKind::Generated { .. } => Decision::Auto,
            ActionKind::WriteFile { path, .. }
            | ActionKind::ApplyPatch { path, .. }
            | ActionKind::ReplaceExact { path, .. }
            | ActionKind::Delete { path, .. } => {
                if task.manifest.scopes(path) {
                    Decision::Auto
                } else {
                    Decision::AskUnscoped
                }
            }
            ActionKind::RunScript(script) if self.skips(script) => {
                Decision::Skip(Skip::ExpectedFailure)
            }
            ActionKind::RunScript(script) | ActionKind::Commit(script) => {
                let class = classifier.classify(script.shell, source.resolve(&script.body));
                self.script(script, class)
            }
        }
    }

    /// D67: a Verify script that expects failure, unless expected failures run. A Gate, a
    /// Preview or an Apply runs (a skipped Gate leaves `$gate` unset for the commit), and a
    /// `Code(n)` is never an expected failure.
    fn skips(&self, script: &Script) -> bool {
        let failure = script
            .expect
            .as_ref()
            .is_some_and(|e| e.exit == ExitExpect::Failure);
        failure && script.intent == ScriptIntent::Verify && !self.run_expected_failures
    }

    fn script(&self, script: &Script, class: PolicyClass) -> Decision {
        if class == PolicyClass::Destructive {
            return Decision::TypedYes;
        }
        if script.intent == ScriptIntent::Apply || !self.auto.contains(&class) {
            return Decision::Ask(class);
        }
        Decision::Auto
    }
}

#[cfg(test)]
mod tests {
    use planrun_model::{ActionKind, ChangeKind, Expectation, PathOrigin, WriteMode};

    use super::*;
    use crate::ir::Ir;
    use crate::policy::DefaultClassifier;

    fn expecting(mut action: Action, exit: ExitExpect, intent: ScriptIntent) -> Action {
        if let ActionKind::RunScript(script) | ActionKind::Commit(script) = &mut action.kind {
            script.intent = intent;
            script.expect = Some(Expectation {
                exit,
                lines: None,
                prose: None,
                comments: None,
            });
        }
        action
    }

    #[test]
    fn decisions() {
        use Decision::{Ask, AskUnscoped, Auto, Show, TypedYes, Wait};
        use PolicyClass::{Build, Modify, Read};
        use ScriptIntent::{Apply, Gate, Preview, Verify};
        let mut ir = Ir::default();
        let manifest = ir.manifest(&[
            ("src/lib.rs", ChangeKind::Modify, PathOrigin::Written),
            ("README.md", ChangeKind::Modify, PathOrigin::Bare),
        ]);
        let none = Policy::default();
        let build = Policy::new(&[Read, Build], false).unwrap_or_else(|e| panic!("{e}"));
        let all = Policy::new(&[Read, Build, Modify], true).unwrap_or_else(|e| panic!("{e}"));
        let fails =
            |ir: &mut Ir, intent| expecting(ir.script("cargo test"), ExitExpect::Failure, intent);
        #[rustfmt::skip]
        let rows: Vec<(Action, &Policy, Decision)> = vec![
            (ir.note("a note"), &none, Show),
            (ir.manual("by hand"), &all, Wait),
            (ir.generated("Cargo.lock"), &none, Auto),
            (ir.write("src/lib.rs", "x", WriteMode::Overwrite), &none, Auto),
            (ir.write("src/new.rs", "x", WriteMode::Create), &all, AskUnscoped),
            (ir.replace("README.md", "a", "b"), &all, AskUnscoped),
            (ir.delete("src/lib.rs", true), &none, Auto),
            (ir.script("git status"), &none, Ask(Read)),
            (ir.script("git status"), &build, Auto),
            (ir.script("cargo test"), &build, Auto),
            (ir.script("git add -A"), &build, Ask(Modify)),
            (ir.script("git add -A"), &all, Auto),
            (ir.script("git push"), &all, TypedYes),
            (ir.commit("git add -A; git commit -m x"), &all, Auto),
            (expecting(ir.script("replace a b -All"), ExitExpect::Success, Apply), &all, Ask(Modify)),
            (fails(&mut ir, Verify), &none, Decision::Skip(Skip::ExpectedFailure)),
            (fails(&mut ir, Verify), &build, Decision::Skip(Skip::ExpectedFailure)),
            (fails(&mut ir, Verify), &all, Auto),
            (fails(&mut ir, Gate), &build, Auto),
            (fails(&mut ir, Preview), &none, Ask(Build)),
            (expecting(ir.script("rg x"), ExitExpect::Code(1), Verify), &none, Ask(Read)),
            (expecting(ir.commit("git commit -m x"), ExitExpect::Failure, Verify), &all, Auto),
        ];
        let task = ir.task("P1-T1", manifest, vec![]);
        let source = ir.source();
        for (action, policy, expected) in rows {
            let got = policy.decide(&task, &action, &source, &DefaultClassifier);
            assert_eq!(got, expected, "{:?}", source.slice(action.span));
        }
    }

    #[test]
    fn auto_values() {
        use PolicyClass::{Build, Modify, Read};
        assert_eq!(
            Policy::parse_auto("read,Build, MODIFY"),
            Ok(vec![Read, Build, Modify])
        );
        assert_eq!(
            Policy::parse_auto("read,destructive"),
            Err(PolicyError::Destructive)
        );
        let unknown = PolicyError::Unknown("all".to_owned());
        assert_eq!(Policy::parse_auto("all"), Err(unknown));
        let refused = Policy::new(&[PolicyClass::Destructive], false);
        assert_eq!(refused, Err(PolicyError::Destructive));
    }
}
```

File `crates/planrun-exec/src/runner/gate.rs` (full contents):

```rust
//! The commit gate (decisions D22, D75): the last attempt of every script of the task, the
//! file actions the user skipped and the `Generated` checks. It is closed by an unresolved
//! mismatch, a confirmation answered no, a user skip, an accepted mismatch, a missing
//! `Generated` path, and a Gate that ran only in a session that has since ended. A skip by
//! policy (D67) keeps it open.

use planrun_model::Shell;

use crate::journal::Outcome;
use crate::runner::sessions::{Sessions, name};

/// One script's last attempt.
#[derive(Debug, Clone)]
pub(crate) struct Attempt {
    /// Where, as `L<line>` of the plan.
    pub line: u32,
    /// How it ended.
    pub outcome: Outcome,
    /// A Gate script (its verdict only counts from the current session).
    pub gate: bool,
    /// Its shell, and the session generation it ran in.
    pub shell: Shell,
    /// The generation.
    pub generation: u64,
}

/// What closes the gate, per task.
#[derive(Debug, Default)]
pub(crate) struct Gate {
    attempts: Vec<((usize, usize), Attempt)>,
    other: Vec<String>,
}

impl Gate {
    /// Records the last attempt of the script at (step, action), replacing an earlier one.
    pub(crate) fn record(&mut self, at: (usize, usize), attempt: Attempt) {
        self.attempts.retain(|(k, _)| *k != at);
        self.attempts.push((at, attempt));
    }

    /// Records something else that closes the gate (a skipped file action, a missing
    /// `Generated` path).
    pub(crate) fn close(&mut self, reason: String) {
        self.other.push(reason);
    }

    /// The Gate scripts whose verdict comes from a session that has ended (D75): they run
    /// again before the commit.
    pub(crate) fn stale_gates(&self, sessions: &Sessions) -> Vec<(usize, usize)> {
        self.attempts
            .iter()
            .filter(|(_, a)| a.gate && a.generation != sessions.generation(a.shell))
            .filter(|(_, a)| a.outcome != Outcome::SkippedPolicy)
            .map(|(k, _)| *k)
            .collect()
    }

    /// Every reason the gate is closed, in plan order; empty when it is open.
    pub(crate) fn reasons(&self, sessions: &Sessions) -> Vec<String> {
        let mut attempts: Vec<&((usize, usize), Attempt)> = self.attempts.iter().collect();
        attempts.sort_by_key(|(k, _)| *k);
        let mut reasons: Vec<String> = attempts
            .into_iter()
            .filter_map(|(_, a)| reason(a, sessions))
            .collect();
        reasons.extend(self.other.iter().cloned());
        reasons
    }
}

fn reason(a: &Attempt, sessions: &Sessions) -> Option<String> {
    let what = match a.outcome {
        Outcome::Failed => "failed",
        Outcome::Refused => "was not confirmed",
        Outcome::SkippedUser => "was skipped by you",
        Outcome::Accepted => "did not match, and you accepted it",
        _ if a.gate && a.generation != sessions.generation(a.shell) => {
            return Some(format!(
                "L{}: the gate ran in a {} session that has ended",
                a.line,
                name(a.shell)
            ));
        }
        _ => return None,
    };
    Some(format!("L{}: the script {what}", a.line))
}
```

File `crates/planrun-exec/src/runner/mod.rs` (full contents):

```rust
//! The runner (docs/architecture.md §6.1; decisions D14, D22, D69, D73–D81): Task → Step →
//! Action over the disk, the journal, the shell sessions and a [`Ui`].
//!
//! Per task: preflight, `task-start`, the steps in order, the `Generated` checks before the
//! commit (or at the end), `task-end`. Per step: consecutive file actions are one transaction
//! whose write-ahead record is the journal's `step-prepared`; scripts run one at a time in their
//! shell's session and are judged by the matcher; a Commit runs only behind the commit gate.
//! P4's [`Policy`] decides; the runner prompts (D69) and never turns one decision into another.

mod gate;
mod script;
mod sessions;
mod task;

use std::io;
use std::path::PathBuf;

use planrun_model::{Action, ActionKind, Plan, Task};

use crate::git::{GitError, Head};
use crate::handler::{HandlerSet, file_path};
use crate::journal::{At, End, Journal, Record};
use crate::policy::{DefaultClassifier, Policy, PolicyClass};
use crate::shell::{SessionConfig, ShellError};
use crate::ui::{Choice, Position, Question, Reply, Ui};
use crate::workspace::{Workspace, WorkspaceError};

use sessions::Sessions;

/// What a run is given besides the plan.
#[derive(Debug, Clone)]
pub struct RunConfig {
    /// The plan's path as the journal records it.
    pub plan_path: String,
    /// The policy (`--auto`, `--run-expected-failures`).
    pub policy: Policy,
    /// The first task to run, by position (`--from`).
    pub from: Option<usize>,
    /// The last task to run, by position (`--to`).
    pub to: Option<usize>,
    /// How sessions start (root, environment, timeout, temp directory).
    pub session: SessionConfig,
}

/// How a run ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finished {
    /// Completed or stopped.
    pub end: End,
    /// Why it stopped.
    pub reason: Option<String>,
}

/// What stops a run that is not a finding: the journal, git, the file system, a shell.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// The journal could not be written.
    #[error("the journal: {0}")]
    Journal(#[source] io::Error),
    /// A git call failed.
    #[error(transparent)]
    Git(#[from] GitError),
    /// The repository or a path could not be used.
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    /// A shell could not be started.
    #[error(transparent)]
    Shell(#[from] ShellError),
}

pub(crate) type Result<T> = std::result::Result<T, RunError>;

/// Whether the run goes on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Flow {
    Go,
    Stop(String),
}

/// A plan being run.
pub struct Runner<'r> {
    plan: &'r Plan,
    ws: &'r Workspace,
    journal: &'r mut Journal,
    ui: &'r mut dyn Ui,
    config: RunConfig,
    classifier: DefaultClassifier,
    handlers: HandlerSet,
    sessions: Sessions,
    /// A notice to acknowledge before the next block: the session is new (D81).
    notice: Option<Vec<String>>,
    /// Tasks whose scripts ran before, for the notice.
    earlier: Vec<String>,
}

impl<'r> Runner<'r> {
    /// A runner over `plan` in `ws`, journaling to `journal` and asking `ui`.
    pub fn new(
        plan: &'r Plan,
        ws: &'r Workspace,
        journal: &'r mut Journal,
        ui: &'r mut dyn Ui,
        config: RunConfig,
    ) -> Runner<'r> {
        let sessions = Sessions::new(config.session.clone());
        Runner {
            plan,
            ws,
            journal,
            ui,
            config,
            classifier: DefaultClassifier,
            handlers: HandlerSet::files(),
            sessions,
            notice: None,
            earlier: Vec::new(),
        }
    }

    /// Runs the plan's tasks from `--from` to `--to`. Every way it ends is journaled
    /// (`run-end`), except a kill.
    pub fn run(&mut self) -> Result<Finished> {
        let result = self.start().and_then(|()| self.tasks());
        let (end, reason) = match &result {
            Ok(Flow::Go) => (End::Completed, None),
            Ok(Flow::Stop(reason)) => (End::Stopped, Some(reason.clone())),
            Err(e) => (End::Stopped, Some(e.to_string())),
        };
        let ended = self.append(Record::RunEnd {
            end,
            reason: reason.clone(),
        });
        result?;
        ended?;
        Ok(Finished { end, reason })
    }

    fn start(&mut self) -> Result<()> {
        let policy = &self.config.policy;
        let auto = policy.auto().iter().map(|c| class_name(*c)).collect();
        let record = Record::Run {
            plan: self.config.plan_path.clone(),
            fingerprint: self.plan.source.fingerprint.to_string(),
            format: self.plan.format.0.to_string(),
            root: self.ws.root().display().to_string(),
            head: self.ws.git().head()?,
            auto,
            run_expected_failures: policy.runs_expected_failures(),
            from: self
                .config
                .from
                .map(|i| self.plan.tasks[i].id.raw.to_string()),
            to: self
                .config
                .to
                .map(|i| self.plan.tasks[i].id.raw.to_string()),
            resume: false,
            planrun: env!("CARGO_PKG_VERSION").to_owned(),
        };
        self.append(record)?;
        let from = self.config.from.unwrap_or(0);
        if from > 0 {
            self.earlier = self.plan.tasks[..from]
                .iter()
                .map(|t| t.id.raw.to_string())
                .collect();
            self.notice = Some(self.notice_text("the run starts at a later task (--from)"));
        }
        Ok(())
    }

    fn tasks(&mut self) -> Result<Flow> {
        let from = self.config.from.unwrap_or(0);
        let to = self
            .config
            .to
            .unwrap_or(self.plan.tasks.len().saturating_sub(1));
        for index in from..=to.min(self.plan.tasks.len().saturating_sub(1)) {
            if let Flow::Stop(reason) = self.task(index)? {
                return Ok(Flow::Stop(reason));
            }
            self.earlier.push(self.plan.tasks[index].id.raw.to_string());
        }
        Ok(Flow::Go)
    }

    fn append(&mut self, record: Record) -> Result<()> {
        self.journal.append(record).map_err(RunError::Journal)
    }

    fn head(&self) -> Result<Head> {
        Ok(self.ws.git().head()?)
    }

    fn at(&self, pos: Position) -> At {
        At {
            task: self.plan.tasks[pos.task].id.raw.to_string(),
            step: pos.step,
            action: pos.action,
        }
    }

    fn line(&self, action: &Action) -> u32 {
        self.plan.source.line_col(action.span.start).line
    }

    /// The notice that a session is new (D81): what is lost, and which tasks' scripts ran
    /// before.
    fn notice_text(&self, why: &str) -> Vec<String> {
        let mut text = vec![
            format!("This is a new shell session: {why}."),
            "Variables, functions and the working directory set by earlier blocks are gone."
                .to_owned(),
        ];
        if !self.earlier.is_empty() {
            text.push(format!(
                "Scripts of these earlier tasks ran in another session: {}.",
                self.earlier.join(", ")
            ));
        }
        text
    }

    /// Asks `question` until it gets a choice or a typed line: `[v]`, `[d]` and `[o]` are
    /// handled here, and HEAD is read again after `[o]` (D80). Every answer is journaled.
    fn ask(&mut self, question: Question) -> Result<Answer> {
        loop {
            match self.ui.ask(&question) {
                Reply::Choice(Choice::View) => match question.at {
                    Some(at) => self.ui.view(at),
                    None => self.ui.status("there is no action to view here"),
                },
                Reply::Choice(Choice::Diff) => self.ui.diff(),
                Reply::Choice(Choice::Shell) => {
                    self.ui.shell();
                    let head = self.head()?;
                    self.append(Record::HeadSeen { head })?;
                }
                Reply::Choice(choice) if !question.choices.contains(&choice) => {
                    self.ui.status("that is not one of the answers offered");
                }
                Reply::Choice(choice) => {
                    self.answered(&question, choice.label())?;
                    return Ok(Answer::Choice(choice));
                }
                Reply::Typed(line) => {
                    self.answered(&question, &line)?;
                    return Ok(Answer::Typed(line));
                }
                Reply::Stop(why) => return Ok(Answer::Stop(why)),
            }
        }
    }

    fn answered(&mut self, question: &Question, answer: &str) -> Result<()> {
        let at = question.at.map(|p| self.at(p));
        self.append(Record::Answer {
            at,
            question: question.kind.to_owned(),
            answer: answer.to_owned(),
        })
    }

    /// Asks for a typed `yes` (D74): exactly `yes` agrees, `quit` stops, `skip` skips when
    /// `skippable`; anything else does not run and asks again.
    fn typed_yes(&mut self, question: Question, skippable: bool) -> Result<Typed> {
        loop {
            match self.ask(question.clone())? {
                Answer::Typed(line) if line == "yes" => return Ok(Typed::Yes),
                Answer::Typed(line) if line == "quit" => return Ok(Typed::Quit),
                Answer::Typed(line) if line == "skip" && skippable => return Ok(Typed::Skip),
                Answer::Stop(why) => return Ok(Typed::Stop(why)),
                _ => self.ui.status(if skippable {
                    "type `yes` to run it, `skip` to skip it, `quit` to stop"
                } else {
                    "type `yes` to go on, or `quit` to stop"
                }),
            }
        }
    }

    /// Asks the pending session notice, if any; it is acknowledged even with `--auto` (D81).
    fn acknowledge_notice(&mut self, at: Option<Position>) -> Result<Flow> {
        let Some(text) = self.notice.take() else {
            return Ok(Flow::Go);
        };
        let question = Question {
            kind: "session",
            at,
            text,
            choices: vec![Choice::Continue, Choice::Shell, Choice::Quit],
            typed: false,
        };
        Ok(match self.ask(question)? {
            Answer::Choice(Choice::Continue) => Flow::Go,
            Answer::Stop(why) => Flow::Stop(why),
            _ => Flow::Stop("stopped at the session notice".to_owned()),
        })
    }
}

/// An answer the runner acts on.
pub(crate) enum Answer {
    Choice(Choice),
    Typed(String),
    Stop(String),
}

/// What a typed question came to.
pub(crate) enum Typed {
    Yes,
    Skip,
    Quit,
    Stop(String),
}

fn class_name(class: PolicyClass) -> String {
    format!("{class:?}").to_lowercase()
}

/// A short description of an action: `write src/a.rs`, `script (pwsh)`.
pub(crate) fn describe(action: &Action) -> String {
    let path = file_path(&action.kind).map(|p| p.to_string());
    let what = match &action.kind {
        ActionKind::WriteFile { .. } => "write",
        ActionKind::ApplyPatch { .. } => "patch",
        ActionKind::ReplaceExact { .. } => "replace",
        ActionKind::Delete { .. } => "delete",
        ActionKind::Generated { path } => return format!("generated {path}"),
        ActionKind::RunScript(s) => return format!("script ({})", sessions::name(s.shell)),
        ActionKind::Commit(s) => return format!("commit ({})", sessions::name(s.shell)),
        ActionKind::Note { .. } => "note",
        ActionKind::Manual { .. } => "manual",
    };
    match path {
        Some(path) => format!("{what} {path}"),
        None => what.to_owned(),
    }
}

/// True when `task` is a file action.
pub(crate) fn is_file_action(action: &Action) -> bool {
    file_path(&action.kind).is_some()
}

/// The canonical form of `dir`; `None` when it is no file-system path planrun can resolve
/// (a pwsh provider path such as `HKLM:\`).
pub(crate) fn canonical(dir: &std::path::Path) -> Option<PathBuf> {
    std::fs::canonicalize(dir).ok()
}

/// The paths a task names: its file actions, its `Generated` paths, and its manifest's file
/// entries (patterns are matched separately).
pub(crate) fn task_paths(task: &Task) -> Vec<planrun_model::RepoPath> {
    let mut paths: Vec<planrun_model::RepoPath> = Vec::new();
    let actions = task.steps.iter().flat_map(|s| s.actions.iter());
    let named = actions.filter_map(|a| match &a.kind {
        ActionKind::Generated { path } => Some(path),
        kind => file_path(kind),
    });
    let entries = task.manifest.entries.iter().filter_map(|e| e.path.file());
    for path in named.chain(entries) {
        if !paths.contains(path) {
            paths.push(path.clone());
        }
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_names_are_the_auto_spellings() {
        assert_eq!(class_name(PolicyClass::Build), "build");
    }
}
```

File `crates/planrun-exec/src/runner/script.rs` (full contents):

```rust
//! One script or commit: its decision and prompt (D69, D74), the questions before its block
//! (the session notice, a working directory outside the repository; D81), the run, the
//! matcher's verdict and what each verdict asks (D66, D75), and the commit's own checks.

use planrun_model::{Action, Script, ScriptIntent};

use super::gate::Attempt;
use super::sessions::{NotRun, name};
use super::task::TaskRun;
use super::{Answer, Flow, Result, RunError, Runner, Typed, canonical, describe};
use crate::expect::{self, Mismatch, Reason, Verdict};
use crate::journal::{Outcome, PathBlob, Record};
use crate::policy::{Decision, Skip};
use crate::shell::{RunStatus, ScriptRun};
use crate::ui::{Choice, Position, Question};

/// What the matcher, and for a commit HEAD, concluded.
enum Judged {
    Pass,
    Confirm(Vec<String>),
    /// What does not hold; `dead` when the session ended, timed out or broke.
    Mismatch {
        lines: Vec<String>,
        dead: bool,
    },
}

/// How many output lines the journal keeps.
const TAIL: usize = 20;

impl Runner<'_> {
    /// Runs the script (or commit) at `pos` per its decision.
    pub(crate) fn script(
        &mut self,
        run: &mut TaskRun,
        pos: Position,
        action: &Action,
        script: &Script,
        commit: bool,
    ) -> Result<Flow> {
        let plan = self.plan;
        let task = &plan.tasks[pos.task];
        let decision = self
            .config
            .policy
            .decide(task, action, &plan.source, &self.classifier);
        let line = self.line(action);
        if let Decision::Skip(Skip::ExpectedFailure) = decision {
            self.ui.status(&format!(
                "  L{line:<5} {}: skipped: expected failure (D67)",
                describe(action)
            ));
            return self.skipped(run, pos, action, script, decision, Outcome::SkippedPolicy);
        }
        match self.permission(pos, action, script, decision)? {
            Permit::Run => {}
            Permit::Skip => {
                return self.skipped(run, pos, action, script, decision, Outcome::SkippedUser);
            }
            Permit::Stop(why) => return Ok(Flow::Stop(why)),
        }
        loop {
            if let Flow::Stop(why) = self.before_block(script, pos)? {
                return Ok(Flow::Stop(why));
            }
            let head = self.head()?;
            let expect = if commit {
                self.commit_expect(task)?
            } else {
                Vec::new()
            };
            self.append(Record::ActionStart {
                at: self.at(pos),
                kind: if commit { "commit" } else { "script" }.to_owned(),
                decision: format!("{decision:?}"),
                head: head.clone(),
                expect: expect.clone(),
            })?;
            self.ui
                .status(&format!("  L{line:<5} {}: running", describe(action)));
            let ran = self.block(script)?;
            let before = head.commit.as_deref();
            let judged = self.judge(script, &ran, commit, before, &expect)?;
            let outcome = match self.verdict(pos, script, &ran, judged)? {
                Resolved::Outcome(outcome) => outcome,
                Resolved::Retry => {
                    self.ended(pos, Outcome::Failed, &ran)?;
                    continue;
                }
                Resolved::Stop(why) => {
                    self.ended(pos, Outcome::Failed, &ran)?;
                    return Ok(Flow::Stop(why));
                }
            };
            self.ended(pos, outcome, &ran)?;
            let attempt = Attempt {
                line,
                outcome,
                gate: script.intent == ScriptIntent::Gate,
                shell: script.shell,
                generation: self.sessions.generation(script.shell),
            };
            run.gate.record((pos.step, pos.action), attempt);
            if commit {
                self.commit_outside(task, before)?;
            }
            return Ok(Flow::Go);
        }
    }

    /// A skipped script, journaled as started and ended, and recorded for the gate.
    fn skipped(
        &mut self,
        run: &mut TaskRun,
        pos: Position,
        action: &Action,
        script: &Script,
        decision: Decision,
        outcome: Outcome,
    ) -> Result<Flow> {
        let head = self.head()?;
        self.append(Record::ActionStart {
            at: self.at(pos),
            kind: "script".to_owned(),
            decision: format!("{decision:?}"),
            head: head.clone(),
            expect: Vec::new(),
        })?;
        self.append(Record::ActionEnd {
            at: self.at(pos),
            outcome,
            status: None,
            errored: false,
            cwd: None,
            tail: Vec::new(),
            head,
        })?;
        let attempt = Attempt {
            line: self.line(action),
            outcome,
            gate: script.intent == ScriptIntent::Gate,
            shell: script.shell,
            generation: self.sessions.generation(script.shell),
        };
        run.gate.record((pos.step, pos.action), attempt);
        Ok(Flow::Go)
    }

    /// The prompt the decision calls for (D69): none for Auto, a choice for Ask, the whole
    /// script and a typed `yes` for Destructive.
    fn permission(
        &mut self,
        pos: Position,
        action: &Action,
        script: &Script,
        decision: Decision,
    ) -> Result<Permit> {
        let plan = self.plan;
        let body = plan.source.resolve(&script.body);
        let head = format!("L{} {}", self.line(action), describe(action));
        match decision {
            Decision::Auto => {
                let class = self.classifier.explain(script.shell, body).class;
                self.ui
                    .status(&format!("  {head}: runs ({class:?}, --auto)"));
                Ok(Permit::Run)
            }
            Decision::TypedYes => {
                let found = self.classifier.explain(script.shell, body);
                let mut text = vec![format!("{head}: Destructive, the whole script:")];
                text.extend(body.lines().map(|l| format!("  | {l}")));
                text.extend(found.reasons.iter().map(|r| format!("  because: {r}")));
                text.push("type `yes` to run it, `skip` to skip it, `quit` to stop".to_owned());
                let question = Question {
                    kind: "typed-yes",
                    at: Some(pos),
                    text,
                    choices: Vec::new(),
                    typed: true,
                };
                Ok(match self.typed_yes(question, true)? {
                    Typed::Yes => Permit::Run,
                    Typed::Skip => Permit::Skip,
                    Typed::Quit => Permit::Stop("quit".to_owned()),
                    Typed::Stop(why) => Permit::Stop(why),
                })
            }
            _ => {
                let class = match decision {
                    Decision::Ask(class) => format!("{class:?}"),
                    other => format!("{other:?}"),
                };
                let mut text = vec![format!("{head} ({class})")];
                text.extend(body.lines().take(6).map(|l| format!("  | {l}")));
                let question = Question {
                    kind: "run",
                    at: Some(pos),
                    text,
                    choices: vec![
                        Choice::Run,
                        Choice::Skip,
                        Choice::View,
                        Choice::Diff,
                        Choice::Shell,
                        Choice::Quit,
                    ],
                    typed: false,
                };
                Ok(match self.ask(question)? {
                    Answer::Choice(Choice::Run) => Permit::Run,
                    Answer::Choice(Choice::Skip) => Permit::Skip,
                    Answer::Stop(why) => Permit::Stop(why),
                    _ => Permit::Stop("quit".to_owned()),
                })
            }
        }
    }

    /// The questions before a block (D81): the session notice, and while the session's working
    /// directory is outside the repository, whether to run there. Neither changes the
    /// block's decision.
    fn before_block(&mut self, script: &Script, pos: Position) -> Result<Flow> {
        if let Some(restart) = self.sessions.starts(script.shell) {
            if restart && self.notice.is_none() {
                let why = format!("the {} session ended", name(script.shell));
                self.notice = Some(self.notice_text(&why));
            }
            let reason = if restart { "restarted" } else { "started" };
            if let Flow::Stop(why) = self.acknowledge_notice(Some(pos))? {
                return Ok(Flow::Stop(why));
            }
            self.append(Record::Session {
                shell: name(script.shell).to_owned(),
                reason: reason.to_owned(),
            })?;
            return Ok(Flow::Go);
        }
        let Some(cwd) = self.sessions.cwd(script.shell).map(|p| p.to_path_buf()) else {
            return Ok(Flow::Go);
        };
        let root = self.ws.root();
        if canonical(&cwd).is_some_and(|c| c.starts_with(root)) {
            return Ok(Flow::Go);
        }
        let question = Question {
            kind: "outside",
            at: Some(pos),
            text: vec![format!(
                "the {} session's working directory is {}, outside the repository; the next \
                 block runs there",
                name(script.shell),
                cwd.display()
            )],
            choices: vec![Choice::Continue, Choice::View, Choice::Shell, Choice::Quit],
            typed: false,
        };
        Ok(match self.ask(question)? {
            Answer::Choice(Choice::Continue) => Flow::Go,
            Answer::Stop(why) => Flow::Stop(why),
            _ => Flow::Stop("quit".to_owned()),
        })
    }

    /// Runs the block, streaming its output. A block the session refused is a run whose
    /// session ended.
    fn block(&mut self, script: &Script) -> Result<ScriptRun> {
        let plan = self.plan;
        let body = plan.source.resolve(&script.body);
        let ui = &mut *self.ui;
        let result = self
            .sessions
            .run(script.shell, body, &mut |line| ui.output(line));
        match result {
            Ok(run) => Ok(run),
            Err(NotRun::Start(e)) => Err(RunError::Shell(e)),
            Err(NotRun::Refused(e)) => {
                self.ui.status(&format!("  the block did not run: {e}"));
                Ok(ScriptRun {
                    status: RunStatus::SessionEnded(None),
                    errored: false,
                    lines: Vec::new(),
                    cwd: None,
                })
            }
        }
    }

    /// The matcher's verdict (D66), and for a commit whether HEAD moved as it should (D79).
    fn judge(
        &self,
        script: &Script,
        ran: &ScriptRun,
        commit: bool,
        before: Option<&str>,
        expect: &[PathBlob],
    ) -> Result<Judged> {
        let dead = matches!(
            ran.status,
            RunStatus::SessionEnded(_) | RunStatus::TimedOut | RunStatus::Broken(_)
        );
        let verdict = expect::check(script.expect.as_ref(), ran, &self.plan.source);
        let problem = if commit && !dead {
            self.commit_problem(before, expect)?
        } else {
            None
        };
        Ok(match (verdict, problem) {
            (Verdict::Mismatch(ms), problem) => Judged::Mismatch {
                lines: ms.iter().map(mismatch_line).chain(problem).collect(),
                dead,
            },
            (_, Some(problem)) => Judged::Mismatch {
                lines: vec![problem],
                dead,
            },
            (Verdict::Pass, None) => Judged::Pass,
            (Verdict::Confirm(c), None) => {
                let mut lines: Vec<String> = c
                    .reasons
                    .iter()
                    .map(|r| self.reason_line(script, *r))
                    .collect();
                lines.push(format!("errored: {}", c.errored));
                Judged::Confirm(lines)
            }
        })
    }

    fn reason_line(&self, script: &Script, reason: Reason) -> String {
        let expect = script.expect.as_ref();
        let text = |t: Option<&planrun_model::TextRef>| {
            t.map(|t| self.plan.source.resolve(t).trim().to_owned())
                .unwrap_or_default()
        };
        match reason {
            Reason::Prose => format!(
                "the plan expects (prose): {}",
                text(expect.and_then(|e| e.prose.as_ref()))
            ),
            Reason::Comments => format!(
                "the script's comments expect: {}",
                text(expect.and_then(|e| e.comments.as_ref()))
            ),
            Reason::UncheckedExit => {
                "the plan states no exit status, and the block did not exit 0".to_owned()
            }
            Reason::Errored => "pwsh recorded an error ($Error[0] changed)".to_owned(),
        }
    }

    /// What a verdict asks (D66, D75): a confirmation for prose; for a mismatch accept, retry,
    /// shell or abort, and only retry, shell or abort when the session is dead.
    fn verdict(
        &mut self,
        pos: Position,
        script: &Script,
        ran: &ScriptRun,
        judged: Judged,
    ) -> Result<Resolved> {
        match judged {
            Judged::Pass => {
                self.ui.status("    passed");
                Ok(Resolved::Outcome(Outcome::Passed))
            }
            Judged::Confirm(mut text) => {
                text.insert(0, "confirm the result:".to_owned());
                let question = Question {
                    kind: "confirm",
                    at: Some(pos),
                    text,
                    choices: vec![
                        Choice::Yes,
                        Choice::No,
                        Choice::View,
                        Choice::Diff,
                        Choice::Shell,
                        Choice::Quit,
                    ],
                    typed: false,
                };
                Ok(match self.ask(question)? {
                    Answer::Choice(Choice::Yes) => Resolved::Outcome(Outcome::Confirmed),
                    Answer::Choice(Choice::No) => Resolved::Outcome(Outcome::Refused),
                    Answer::Stop(why) => Resolved::Stop(why),
                    _ => Resolved::Stop("quit".to_owned()),
                })
            }
            Judged::Mismatch { lines, dead } => {
                let mut text = vec![format!("MISMATCH ({:?}):", ran.status)];
                text.extend(lines.into_iter().map(|l| format!("  {l}")));
                let mut choices = if dead {
                    text.push(format!(
                        "the {} session is gone; retry starts a new one",
                        name(script.shell)
                    ));
                    vec![Choice::Retry]
                } else {
                    vec![Choice::Accept, Choice::Retry]
                };
                choices.extend([Choice::View, Choice::Diff, Choice::Shell, Choice::Quit]);
                let question = Question {
                    kind: "mismatch",
                    at: Some(pos),
                    text,
                    choices,
                    typed: false,
                };
                match self.ask(question)? {
                    Answer::Choice(Choice::Accept) => Ok(Resolved::Outcome(Outcome::Accepted)),
                    Answer::Choice(Choice::Retry) => {
                        if dead || !self.sessions.alive(script.shell) {
                            self.sessions.restart(script.shell);
                        }
                        Ok(Resolved::Retry)
                    }
                    Answer::Stop(why) => Ok(Resolved::Stop(why)),
                    _ => Ok(Resolved::Stop("quit".to_owned())),
                }
            }
        }
    }

    /// `action-end` for the attempt at `pos`, with HEAD after it (D80).
    fn ended(&mut self, pos: Position, outcome: Outcome, ran: &ScriptRun) -> Result<()> {
        let head = self.head()?;
        let start = ran.lines.len().saturating_sub(TAIL);
        self.append(Record::ActionEnd {
            at: self.at(pos),
            outcome,
            status: Some(format!("{:?}", ran.status)),
            errored: ran.errored,
            cwd: ran.cwd.as_ref().map(|p| p.display().to_string()),
            tail: ran.lines[start..].iter().map(|l| l.text.clone()).collect(),
            head,
        })
    }
}

/// What the run prompt came to.
enum Permit {
    Run,
    Skip,
    Stop(String),
}

/// What a verdict came to.
enum Resolved {
    Outcome(Outcome),
    Retry,
    Stop(String),
}

fn mismatch_line(m: &Mismatch) -> String {
    match m {
        Mismatch::Exit { expected, got } => match expected {
            Some(e) => format!("exit: expected {e:?}, got {got:?}"),
            None => format!("the block ended as {got:?}"),
        },
        Mismatch::Line { number, line } => format!("expected line {number} not found: {line}"),
    }
}
```

File `crates/planrun-exec/src/runner/sessions.rs` (full contents):

```rust
//! The run's shell sessions (D24, D81): at most one per shell, started lazily at the repository
//! root, never restarted silently. A restart bumps the shell's generation, so a Gate verdict
//! from the session before no longer counts (D75).

use std::path::{Path, PathBuf};

use planrun_model::Shell;

use crate::shell::{OutLine, ScriptRun, Session, SessionConfig, ShellError, ShellSession};

/// The sessions of one run.
pub(crate) struct Sessions {
    config: SessionConfig,
    live: [Option<Session>; 2],
    generation: [u64; 2],
    cwd: [Option<PathBuf>; 2],
    ever: [bool; 2],
}

/// Why a block did not run.
pub(crate) enum NotRun {
    /// The shell could not be started.
    Start(ShellError),
    /// The session refused the block (it has ended, or the block file could not be written).
    Refused(std::io::Error),
}

pub(crate) fn slot(shell: Shell) -> usize {
    match shell {
        Shell::Pwsh => 0,
        Shell::Bash => 1,
    }
}

pub(crate) fn name(shell: Shell) -> &'static str {
    match shell {
        Shell::Pwsh => "pwsh",
        Shell::Bash => "bash",
    }
}

impl Sessions {
    pub(crate) fn new(config: SessionConfig) -> Sessions {
        Sessions {
            config,
            live: [None, None],
            generation: [0, 0],
            cwd: [None, None],
            ever: [false, false],
        }
    }

    /// The generation of `shell`'s current session: 0 for the first, one more per restart.
    pub(crate) fn generation(&self, shell: Shell) -> u64 {
        self.generation[slot(shell)]
    }

    /// The working directory `shell`'s session reported after its last block.
    pub(crate) fn cwd(&self, shell: Shell) -> Option<&Path> {
        let i = slot(shell);
        self.live[i].as_ref()?;
        self.cwd[i].as_deref()
    }

    /// True when the next block of `shell` starts a session: `Some(true)` for a restart.
    pub(crate) fn starts(&self, shell: Shell) -> Option<bool> {
        let i = slot(shell);
        self.live[i].is_none().then_some(self.ever[i])
    }

    /// Runs `script` in `shell`'s session, starting it first when there is none.
    pub(crate) fn run(
        &mut self,
        shell: Shell,
        script: &str,
        on_line: &mut dyn FnMut(&OutLine),
    ) -> Result<ScriptRun, NotRun> {
        let i = slot(shell);
        if self.live[i].is_none() {
            let session = Session::start(shell, &self.config).map_err(NotRun::Start)?;
            self.live[i] = Some(session);
            self.ever[i] = true;
            self.cwd[i] = None;
        }
        let Some(session) = self.live[i].as_mut() else {
            return Err(NotRun::Refused(std::io::Error::other("no session")));
        };
        let run = session.run(script, on_line).map_err(NotRun::Refused)?;
        self.cwd[i] = run.cwd.clone();
        Ok(run)
    }

    /// Whether `shell`'s session can run another block.
    pub(crate) fn alive(&self, shell: Shell) -> bool {
        self.live[slot(shell)]
            .as_ref()
            .is_some_and(Session::is_alive)
    }

    /// Ends `shell`'s session; the next block starts a new one, a generation later.
    pub(crate) fn restart(&mut self, shell: Shell) {
        let i = slot(shell);
        self.live[i] = None;
        self.cwd[i] = None;
        self.generation[i] += 1;
    }
}
```

File `crates/planrun-exec/src/runner/task.rs` (full contents):

```rust
//! One task: preflight, its steps, file transactions, Manuals, `Generated` checks, and what is
//! checked around its commit (D75, D78, D80).

use planrun_model::{Action, ActionKind, RepoPath, Task};

use super::gate::Gate;
use super::{Answer, Flow, Result, RunError, Runner, Typed, describe, is_file_action, task_paths};
use crate::finding::{Finding, Severity};
use crate::journal::{Outcome, PathBlob, Record};
use crate::policy::Decision;
use crate::preflight;
use crate::transaction::{self, CommitCause};
use crate::ui::{Choice, Position, Question};

/// The state of the task being run.
pub(crate) struct TaskRun {
    pub index: usize,
    pub gate: Gate,
    /// The task's `Generated` paths, once each (architecture §3), with the first action naming them.
    pub generated: Vec<(RepoPath, Position)>,
    pub generated_checked: bool,
}

impl Runner<'_> {
    /// Runs task `index`.
    pub(crate) fn task(&mut self, index: usize) -> Result<Flow> {
        let plan = self.plan;
        let task = &plan.tasks[index];
        self.ui
            .status(&format!("{}  {}", task.id, plan.text(&task.title)));
        let found = preflight::preflight(self.ws, plan, task)?;
        if let Flow::Stop(why) = self.findings(task, &found) {
            return Ok(Flow::Stop(why));
        }
        let head = self.head()?;
        self.append(Record::TaskStart {
            task: task.id.raw.to_string(),
            head,
        })?;
        let mut run = TaskRun {
            index,
            gate: Gate::default(),
            generated: generated(task, index),
            generated_checked: false,
        };
        for (si, step) in task.steps.iter().enumerate() {
            let mut ai = 0;
            while ai < step.actions.len() {
                let pos = Position {
                    task: index,
                    step: si,
                    action: ai,
                };
                let group: Vec<&Action> = step.actions[ai..]
                    .iter()
                    .take_while(|a| is_file_action(a))
                    .collect();
                let flow = if group.is_empty() {
                    ai += 1;
                    self.action(&mut run, pos, &step.actions[pos.action])?
                } else {
                    ai += group.len();
                    self.files(&mut run, pos, &group)?
                };
                if let Flow::Stop(why) = flow {
                    return Ok(Flow::Stop(why));
                }
            }
        }
        if let Flow::Stop(why) = self.check_generated(&mut run)? {
            return Ok(Flow::Stop(why));
        }
        let head = self.head()?;
        self.append(Record::TaskEnd {
            task: task.id.raw.to_string(),
            head,
        })?;
        Ok(Flow::Go)
    }

    /// Shows preflight's findings; any error stops the run.
    fn findings(&mut self, task: &Task, found: &[Finding]) -> Flow {
        for f in found {
            let line = f
                .span
                .map(|s| format!("L{} ", self.plan.source.line_col(s.start).line))
                .unwrap_or_default();
            let severity = match f.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };
            self.ui
                .status(&format!("  {severity}: {line}{}: {}", f.code, f.message));
        }
        if found.iter().any(Finding::is_error) {
            return Flow::Stop(format!("{}: preflight found errors", task.id));
        }
        Flow::Go
    }

    /// One action that is not a file action.
    fn action(&mut self, run: &mut TaskRun, pos: Position, action: &Action) -> Result<Flow> {
        let task = &self.plan.tasks[pos.task];
        match &action.kind {
            ActionKind::Note { text } => {
                let text = self.plan.text(text);
                self.ui.status(&format!("  note: {}", first_line(text)));
                Ok(Flow::Go)
            }
            ActionKind::Generated { path } => {
                self.ui
                    .status(&format!("  generated {path}: checked before the commit"));
                Ok(Flow::Go)
            }
            ActionKind::Manual { .. } => self.manual(task, pos, action),
            ActionKind::RunScript(script) => self.script(run, pos, action, script, false),
            ActionKind::Commit(script) => {
                if let Flow::Stop(why) = self.before_commit(run, pos)? {
                    return Ok(Flow::Stop(why));
                }
                self.script(run, pos, action, script, true)
            }
            _ => Ok(Flow::Go),
        }
    }

    /// A step's consecutive file actions as one transaction (D53, D78). An unscoped one asks
    /// first (D50); a skipped one is left out and closes the gate.
    fn files(&mut self, run: &mut TaskRun, pos: Position, group: &[&Action]) -> Result<Flow> {
        let plan = self.plan;
        let task = &plan.tasks[pos.task];
        let mut chosen: Vec<&Action> = Vec::new();
        for (k, action) in group.iter().enumerate() {
            let here = Position {
                action: pos.action + k,
                ..pos
            };
            let decision = self
                .config
                .policy
                .decide(task, action, &plan.source, &self.classifier);
            if decision != Decision::AskUnscoped {
                chosen.push(action);
                continue;
            }
            let question = Question {
                kind: "unscoped",
                at: Some(here),
                text: vec![format!(
                    "L{} {} ({:?}): not in the task's files (D50)",
                    self.line(action),
                    describe(action),
                    action.path_origin
                )],
                choices: vec![
                    Choice::Run,
                    Choice::Skip,
                    Choice::View,
                    Choice::Diff,
                    Choice::Shell,
                    Choice::Quit,
                ],
                typed: false,
            };
            match self.ask(question)? {
                Answer::Choice(Choice::Run) => chosen.push(action),
                Answer::Choice(Choice::Skip) => run.gate.close(format!(
                    "L{}: {} was skipped by you",
                    self.line(action),
                    describe(action)
                )),
                Answer::Stop(why) => return Ok(Flow::Stop(why)),
                _ => return Ok(Flow::Stop("quit".to_owned())),
            }
        }
        if chosen.is_empty() {
            return Ok(Flow::Go);
        }
        self.write_group(pos, &chosen)
    }

    /// Prepares and commits `actions`; asks when the disk changed since prepare (D78).
    pub(crate) fn write_group(&mut self, pos: Position, actions: &[&Action]) -> Result<Flow> {
        let at = self.at(pos);
        loop {
            let step =
                match transaction::prepare(self.ws, &self.plan.source, &self.handlers, actions) {
                    Ok(step) => step,
                    Err(f) => {
                        let line = f
                            .span
                            .map(|s| self.plan.source.line_col(s.start).line)
                            .unwrap_or_default();
                        self.ui
                            .status(&format!("  FAILED L{line} {}: {}", f.code, f.message));
                        return Ok(Flow::Stop(format!("{}: {}", f.code, f.message)));
                    }
                };
            let result = transaction::commit(self.ws, &step, &mut self.journal.ahead(&at));
            let err = match result {
                Ok(()) => {
                    self.append(Record::StepWritten { at })?;
                    for action in actions {
                        let line = self.line(action);
                        self.ui
                            .status(&format!("  L{line:<5} {}: done", describe(action)));
                    }
                    return Ok(Flow::Go);
                }
                Err(err) => err,
            };
            match err.cause {
                CommitCause::Changed(paths) => {
                    let names: Vec<&str> = paths.iter().map(RepoPath::as_str).collect();
                    let question = Question {
                        kind: "changed",
                        at: Some(pos),
                        text: vec![
                            format!("changed since the step was prepared: {}", names.join(", ")),
                            "something else is writing the tree; nothing was written".to_owned(),
                        ],
                        choices: vec![
                            Choice::PrepareAgain,
                            Choice::Diff,
                            Choice::Shell,
                            Choice::Quit,
                        ],
                        typed: false,
                    };
                    match self.ask(question)? {
                        Answer::Choice(Choice::PrepareAgain) => continue,
                        Answer::Stop(why) => return Ok(Flow::Stop(why)),
                        _ => return Ok(Flow::Stop("quit".to_owned())),
                    }
                }
                CommitCause::Record(e) => return Err(RunError::Journal(e)),
                CommitCause::Workspace(e) => {
                    let error = e.to_string();
                    self.append(Record::StepFailed {
                        at,
                        error: error.clone(),
                        unrestored: err.unrestored.clone(),
                    })?;
                    self.ui.status(&format!("  FAILED: {error}"));
                    for path in &err.unrestored {
                        self.ui.status(&format!("  not restored: {path}"));
                    }
                    return Ok(Flow::Stop(error));
                }
            }
        }
    }

    /// A Manual waits until acknowledged (D21), whatever `--auto` says. When HEAD moved (a
    /// commit by hand, D33), the task's paths that are still dirty are reported (D80).
    pub(crate) fn manual(&mut self, task: &Task, pos: Position, action: &Action) -> Result<Flow> {
        let ActionKind::Manual { reason, text } = &action.kind else {
            return Ok(Flow::Go);
        };
        let head = self.head()?;
        self.append(Record::ActionStart {
            at: self.at(pos),
            kind: "manual".to_owned(),
            decision: format!("{:?}", Decision::Wait),
            head: head.clone(),
            expect: Vec::new(),
        })?;
        let mut lines = vec![format!(
            "L{} manual ({reason:?}): do this by hand, then continue",
            self.line(action)
        )];
        lines.extend(self.plan.text(text).lines().map(|l| format!("  {l}")));
        let question = Question {
            kind: "manual",
            at: Some(pos),
            text: lines,
            choices: vec![
                Choice::Continue,
                Choice::View,
                Choice::Diff,
                Choice::Shell,
                Choice::Quit,
            ],
            typed: false,
        };
        match self.ask(question)? {
            Answer::Choice(Choice::Continue) => {}
            Answer::Stop(why) => return Ok(Flow::Stop(why)),
            _ => return Ok(Flow::Stop("quit".to_owned())),
        }
        let after = self.head()?;
        self.append(Record::ActionEnd {
            at: self.at(pos),
            outcome: Outcome::Acknowledged,
            status: None,
            errored: false,
            cwd: None,
            tail: Vec::new(),
            head: after.clone(),
        })?;
        if after.commit != head.commit {
            self.still_dirty(task)?;
        }
        Ok(Flow::Go)
    }

    /// After a commit by hand: the task's paths that still have changes.
    fn still_dirty(&mut self, task: &Task) -> Result<()> {
        let paths = task_paths(task);
        let refs: Vec<&RepoPath> = paths.iter().collect();
        let dirty = self.ws.git().status(Some(&refs))?;
        for entry in dirty {
            self.ui.status(&format!(
                "  warning: HEAD moved, but `{}` still has changes ({})",
                entry.path, entry.code
            ));
        }
        Ok(())
    }

    /// The `Generated` paths, checked once each after the task's last non-commit action (D27,
    /// architecture §3). A missing one closes the gate.
    pub(crate) fn check_generated(&mut self, run: &mut TaskRun) -> Result<Flow> {
        if run.generated_checked {
            return Ok(Flow::Go);
        }
        run.generated_checked = true;
        for (path, pos) in run.generated.clone() {
            let head = self.head()?;
            self.append(Record::ActionStart {
                at: self.at(pos),
                kind: "generated".to_owned(),
                decision: format!("{:?}", Decision::Auto),
                head: head.clone(),
                expect: Vec::new(),
            })?;
            let present = self.ws.read(&path)?.is_some();
            let outcome = if present {
                Outcome::Present
            } else {
                Outcome::Missing
            };
            self.append(Record::ActionEnd {
                at: self.at(pos),
                outcome,
                status: None,
                errored: false,
                cwd: None,
                tail: Vec::new(),
                head,
            })?;
            if present {
                self.ui.status(&format!("  generated {path}: present"));
                continue;
            }
            self.ui.status(&format!(
                "  generated {path}: MISSING (the plan says a step makes it)"
            ));
            run.gate.close(format!(
                "`{path}` should have been generated, and is missing"
            ));
        }
        Ok(Flow::Go)
    }

    /// What comes before a Commit (D75, D80): the `Generated` checks, stale Gates run again,
    /// dirt outside the task's paths asked about, and a closed gate opened only by a typed
    /// `yes` naming every reason.
    fn before_commit(&mut self, run: &mut TaskRun, pos: Position) -> Result<Flow> {
        if let Flow::Stop(why) = self.check_generated(run)? {
            return Ok(Flow::Stop(why));
        }
        if let Flow::Stop(why) = self.rerun_stale_gates(run)? {
            return Ok(Flow::Stop(why));
        }
        if let Flow::Stop(why) = self.outside_dirt(pos)? {
            return Ok(Flow::Stop(why));
        }
        let reasons = run.gate.reasons(&self.sessions);
        if reasons.is_empty() {
            return Ok(Flow::Go);
        }
        let task = &self.plan.tasks[pos.task];
        let mut text = vec![format!("the commit gate of {} is closed:", task.id)];
        text.extend(reasons.iter().map(|r| format!("  {r}")));
        text.push("type `yes` to commit anyway, or `quit`".to_owned());
        let question = Question {
            kind: "gate",
            at: Some(pos),
            text,
            choices: Vec::new(),
            typed: true,
        };
        match self.typed_yes(question, false)? {
            Typed::Yes => {
                self.append(Record::GateOverride {
                    task: task.id.raw.to_string(),
                    reasons,
                })?;
                Ok(Flow::Go)
            }
            Typed::Stop(why) => Ok(Flow::Stop(why)),
            Typed::Skip | Typed::Quit => Ok(Flow::Stop("the commit gate is closed".to_owned())),
        }
    }

    /// Each Gate whose verdict comes from a session that has ended runs again (D75).
    fn rerun_stale_gates(&mut self, run: &mut TaskRun) -> Result<Flow> {
        let task = &self.plan.tasks[run.index];
        for (si, ai) in run.gate.stale_gates(&self.sessions) {
            let action = &task.steps[si].actions[ai];
            let ActionKind::RunScript(script) = &action.kind else {
                continue;
            };
            self.ui.status(&format!(
                "  L{}: the gate ran in a session that has ended; it runs again",
                self.line(action)
            ));
            let pos = Position {
                task: run.index,
                step: si,
                action: ai,
            };
            if let Flow::Stop(why) = self.script(run, pos, action, script, false)? {
                return Ok(Flow::Stop(why));
            }
        }
        Ok(Flow::Go)
    }

    /// Dirt outside the task's paths before a commit (D80): listed and asked about.
    fn outside_dirt(&mut self, pos: Position) -> Result<Flow> {
        let task = &self.plan.tasks[pos.task];
        let outside: Vec<String> = self
            .ws
            .git()
            .status(None)?
            .into_iter()
            .filter(|e| !self.in_task(task, &e.path))
            .map(|e| format!("  {} {}", e.code, e.path))
            .collect();
        if outside.is_empty() {
            return Ok(Flow::Go);
        }
        let mut text = vec![format!(
            "{} path(s) outside the task's files have changes; a commit that stages everything \
             would take them:",
            outside.len()
        )];
        text.extend(outside);
        let question = Question {
            kind: "dirt",
            at: Some(pos),
            text,
            choices: vec![Choice::Continue, Choice::Diff, Choice::Shell, Choice::Quit],
            typed: false,
        };
        Ok(match self.ask(question)? {
            Answer::Choice(Choice::Continue) => Flow::Go,
            Answer::Stop(why) => Flow::Stop(why),
            _ => Flow::Stop("quit".to_owned()),
        })
    }

    /// Whether `path` (as git prints it) is one of the task's paths or matches a pattern entry.
    pub(crate) fn in_task(&self, task: &Task, path: &str) -> bool {
        let fold = |p: &str| {
            if self.ws.ignorecase() {
                p.to_lowercase()
            } else {
                p.to_owned()
            }
        };
        let wanted = fold(path);
        if task_paths(task).iter().any(|p| fold(p.as_str()) == wanted) {
            return true;
        }
        let Ok(path) = RepoPath::new(path) else {
            return false;
        };
        task.manifest.entries.iter().any(|e| e.path.covers(&path))
    }

    /// The task's paths as they are on disk now (git's view), which the commit must hold.
    pub(crate) fn commit_expect(&self, task: &Task) -> Result<Vec<PathBlob>> {
        let paths = task_paths(task);
        let mut present = Vec::new();
        let mut expect = Vec::new();
        for path in &paths {
            let resolved = self.ws.resolve(path)?;
            if resolved.exists() && !resolved.is_dir {
                present.push(path);
            }
        }
        let mut ids = self.ws.git().hash_paths(&present, true)?.into_iter();
        for path in &paths {
            let blob = if present.contains(&path) {
                ids.next()
            } else {
                None
            };
            expect.push(PathBlob {
                path: path.clone(),
                blob,
            });
        }
        Ok(expect)
    }

    /// Why a commit script did not commit what it should (D79): HEAD did not move, moved by
    /// other than one commit on `before`, or its tree does not hold `expect`.
    pub(crate) fn commit_problem(
        &self,
        before: Option<&str>,
        expect: &[PathBlob],
    ) -> Result<Option<String>> {
        let git = self.ws.git();
        let head = git.head()?;
        let Some(now) = head.commit.as_deref() else {
            return Ok(Some("there is no commit at HEAD".to_owned()));
        };
        if Some(now) == before {
            return Ok(Some(
                "the commit script ran, but HEAD did not move".to_owned(),
            ));
        }
        if git.parent(now)?.as_deref() != before {
            return Ok(Some(
                "HEAD moved, but not by one commit on top of the one before".to_owned(),
            ));
        }
        let paths: Vec<&RepoPath> = expect.iter().map(|e| &e.path).collect();
        let held = git.tree_blobs(now, &paths)?;
        let wrong: Vec<&str> = expect
            .iter()
            .zip(&held)
            .filter(|(e, h)| e.blob != **h)
            .map(|(e, _)| e.path.as_str())
            .collect();
        if wrong.is_empty() {
            return Ok(None);
        }
        Ok(Some(format!(
            "the commit does not hold the task's files as they were: {}",
            wrong.join(", ")
        )))
    }

    /// After a commit: paths it changed outside the task's, reported loudly (D80).
    pub(crate) fn commit_outside(&mut self, task: &Task, before: Option<&str>) -> Result<()> {
        let head = self.head()?;
        let (Some(before), Some(now)) = (before, head.commit.as_deref()) else {
            return Ok(());
        };
        let changed = self.ws.git().changed_paths(before, now)?;
        let outside: Vec<String> = changed
            .into_iter()
            .filter(|p| !self.in_task(task, p))
            .collect();
        if !outside.is_empty() {
            self.ui.status(&format!(
                "  WARNING: the commit also changed paths outside the task's files: {}",
                outside.join(", ")
            ));
        }
        Ok(())
    }
}

/// The task's `Generated` paths, once each, with the first action naming each.
fn generated(task: &Task, index: usize) -> Vec<(RepoPath, Position)> {
    let mut found: Vec<(RepoPath, Position)> = Vec::new();
    for (si, step) in task.steps.iter().enumerate() {
        for (ai, action) in step.actions.iter().enumerate() {
            let ActionKind::Generated { path } = &action.kind else {
                continue;
            };
            if found.iter().any(|(p, _)| p == path) {
                continue;
            }
            let pos = Position {
                task: index,
                step: si,
                action: ai,
            };
            found.push((path.clone(), pos));
        }
    }
    found
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}
```

File `crates/planrun-exec/src/ui.rs` (full contents):

```rust
//! The interactive surface the runner talks to (decisions D15, D74). planrun-cli implements it
//! with line prompts on std; the tests with scripted answers. The runner decides what is asked
//! and what an answer does; the UI shows, reads, and handles `[v]`, `[d]` and `[o]` itself.

use crate::shell::OutLine;

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

/// What the runner asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// Its journal name: `run`, `unscoped`, `typed-yes`, `confirm`, `mismatch`, `manual`,
    /// `dirt`, `outside`, `session`, `gate`, `changed`, `again`, `unknown`, `accept-done`.
    pub kind: &'static str,
    /// The action it is about, when there is one (for `[v]`).
    pub at: Option<Position>,
    /// What to show above the prompt, one line per entry.
    pub text: Vec<String>,
    /// The answers offered, in order. Empty with `typed`.
    pub choices: Vec<Choice>,
    /// The answer is a typed line: only `yes` exactly agrees (D74).
    pub typed: bool,
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

File `crates/planrun-exec/tests/runner.rs` (full contents):

```rust
//! The runner on hand-built IR (D2) over a fixture copy, with scripted answers: the two assay
//! tasks end to end, what each decision and verdict asks, the commit gate (D75), dead sessions
//! (D81), and the checks around a commit (D80).

#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[path = "support/ir.rs"]
mod ir;

use std::collections::VecDeque;
use std::time::Duration;

use fixture::Fixture;
use ir::{Ir, PATCH_T1, PATCH_T2};
use planrun_exec::journal::{self, End, Journal, Outcome, Record};
use planrun_exec::policy::{Policy, PolicyClass};
use planrun_exec::runner::{Finished, RunConfig, Runner};
use planrun_exec::shell::{OutLine, SessionConfig};
use planrun_exec::ui::{Choice, Position, Question, Reply, Ui};
use planrun_exec::workspace::Workspace;
use planrun_model::{
    Action, ActionKind, ChangeKind, ExitExpect, Expectation, PathOrigin, Plan, ScriptIntent, Shell,
    WriteMode,
};

/// Answers from a queue; what was asked and shown is kept.
#[derive(Default)]
struct Scripted {
    answers: VecDeque<Reply>,
    asked: Vec<String>,
    shown: Vec<String>,
}

impl Scripted {
    fn new(answers: Vec<Reply>) -> Scripted {
        Scripted {
            answers: answers.into(),
            ..Scripted::default()
        }
    }
}

impl Ui for Scripted {
    fn status(&mut self, line: &str) {
        self.shown.push(line.to_owned());
    }
    fn output(&mut self, line: &OutLine) {
        self.shown.push(format!("| {}", line.text));
    }
    fn ask(&mut self, question: &Question) -> Reply {
        self.asked.push(question.kind.to_owned());
        self.shown.extend(question.text.iter().cloned());
        self.answers
            .pop_front()
            .unwrap_or_else(|| Reply::Stop(format!("no answer for `{}`", question.kind)))
    }
    fn view(&mut self, _at: Position) {}
    fn diff(&mut self) {}
    fn shell(&mut self) {}
}

fn choice(c: Choice) -> Reply {
    Reply::Choice(c)
}

fn typed(s: &str) -> Reply {
    Reply::Typed(s.to_owned())
}

const AUTO: &[PolicyClass] = &[PolicyClass::Read, PolicyClass::Build, PolicyClass::Modify];

/// Runs `plan` on `f` with `auto` and `answers`; returns how it ended, the UI and the records.
fn run(
    f: &Fixture,
    plan: &Plan,
    auto: &[PolicyClass],
    answers: Vec<Reply>,
) -> (Finished, Scripted, Vec<Record>) {
    let ws = Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"));
    let path = f.outside().join("journal.jsonl");
    let mut journal = Journal::open(&path).unwrap_or_else(|e| panic!("{e}"));
    let mut ui = Scripted::new(answers);
    let config = RunConfig {
        plan_path: "test.md".to_owned(),
        policy: Policy::new(auto, false).unwrap_or_else(|e| panic!("{e}")),
        from: None,
        to: None,
        session: SessionConfig {
            root: f.path().to_path_buf(),
            env: f.env(),
            timeout: Some(Duration::from_secs(60)),
            temp: f.outside(),
        },
    };
    let finished = Runner::new(plan, &ws, &mut journal, &mut ui, config)
        .run()
        .unwrap_or_else(|e| panic!("{e}"));
    let records = journal::read(&path)
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|l| l.record)
        .collect();
    (finished, ui, records)
}

fn expecting(
    mut action: Action,
    exit: ExitExpect,
    intent: ScriptIntent,
    prose: bool,
    ir: &mut Ir,
) -> Action {
    let prose = prose.then(|| planrun_model::TextRef::Span(ir.text("all pass")));
    if let ActionKind::RunScript(s) | ActionKind::Commit(s) = &mut action.kind {
        s.intent = intent;
        s.expect = Some(Expectation {
            exit,
            lines: None,
            prose,
            comments: None,
        });
    }
    action
}

fn bash(mut action: Action) -> Action {
    if let ActionKind::RunScript(s) | ActionKind::Commit(s) = &mut action.kind {
        s.shell = Shell::Bash;
    }
    action
}

const W: PathOrigin = PathOrigin::Written;

/// The two tasks of `assay-style.md` by hand, with scripts that need no cargo.
fn assay(ir: &mut Ir) -> Plan {
    let m1 = ir.manifest(&[
        ("src/greet.rs", ChangeKind::Create, W),
        ("src/lib.rs", ChangeKind::Modify, W),
    ]);
    let gate1 = ir.script("$gate = 0; 'gate exit code: 0'");
    let gate1 = expecting(gate1, ExitExpect::Success, ScriptIntent::Gate, false, ir);
    let t1 = vec![
        vec![
            ir.write("src/greet.rs", "pub fn greeting() {}\n", WriteMode::Create),
            ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a"),
        ],
        vec![gate1],
        vec![ir.commit(
            "if ($gate -eq 0) { git add -- src/greet.rs src/lib.rs; git commit -q -m 'P1-T1' }",
        )],
    ];
    let m2 = ir.manifest(&[
        ("src/lib.rs", ChangeKind::Modify, W),
        ("src/old.rs", ChangeKind::Delete, W),
        ("docs/modules.txt", ChangeKind::Create, W),
    ]);
    let gate2 = ir.script("$gate = 0");
    let gate2 = expecting(gate2, ExitExpect::Success, ScriptIntent::Gate, false, ir);
    let t2 = vec![
        vec![
            ir.patch("src/lib.rs", PATCH_T2, "97bd92a", "6b269fc"),
            ir.delete("src/old.rs", false),
            ir.generated("docs/modules.txt"),
        ],
        vec![
            ir.script("New-Item -ItemType Directory -Force docs | Out-Null; Set-Content -NoNewline -Path docs/modules.txt -Value \"greet.rs`nlib.rs`n\""),
            gate2,
        ],
        vec![ir.commit(
            "if ($gate -eq 0) { git add -- src/lib.rs src/old.rs docs/modules.txt; git commit -q -m 'P1-T2' }",
        )],
    ];
    let tasks = vec![ir.task("P1-T1", m1, t1), ir.task("P1-T2", m2, t2)];
    ir.plan(tasks)
}

/// A plan of one task, `T1`.
fn one(ir: &mut Ir, manifest: planrun_model::Manifest, steps: Vec<Vec<Action>>) -> Plan {
    let task = ir.task("T1", manifest, steps);
    ir.plan(vec![task])
}

fn kinds(records: &[Record]) -> Vec<String> {
    records
        .iter()
        .map(|r| {
            let json = serde_json::to_value(r).unwrap_or_else(|e| panic!("{e}"));
            json["t"].as_str().unwrap_or("?").to_owned()
        })
        .collect()
}

fn subjects(f: &Fixture) -> Vec<String> {
    f.git(&["log", "--format=%s"])
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_assay_tasks_run_to_two_commits_under_auto() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let (finished, ui, records) = run(&f, &plan, AUTO, vec![]);
    assert_eq!(
        finished.end,
        End::Completed,
        "{:?}\n{}",
        finished.reason,
        ui.shown.join("\n")
    );
    assert!(ui.asked.is_empty(), "{:?}", ui.asked);
    assert_eq!(subjects(&f), ["P1-T2", "P1-T1", "fixture"]);
    assert_eq!(f.status(), "");
    let ls = f.git(&["ls-tree", "-r", "--name-only", "HEAD"]);
    assert_eq!(
        ls.lines().collect::<Vec<_>>(),
        [
            ".gitignore",
            "Cargo.toml",
            "docs/modules.txt",
            "src/greet.rs",
            "src/lib.rs"
        ]
    );
    let k = kinds(&records);
    let first = |name: &str| k.iter().position(|x| x == name).unwrap_or(usize::MAX);
    assert_eq!(k.first().map(String::as_str), Some("run"));
    assert_eq!(k.last().map(String::as_str), Some("run-end"));
    assert!(first("step-prepared") < first("step-written"));
    assert_eq!(k.iter().filter(|x| *x == "step-prepared").count(), 2);
    assert_eq!(k.iter().filter(|x| *x == "task-end").count(), 2);
    assert!(k.contains(&"session".to_owned()));
    let generated = records.iter().any(|r| {
        matches!(
            r,
            Record::ActionEnd {
                outcome: Outcome::Present,
                ..
            }
        )
    });
    assert!(generated, "the generated file was checked");
}

#[test]
fn without_auto_every_script_asks_and_a_skipped_gate_closes_the_gate() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let answers = vec![choice(Choice::Skip), typed("no"), typed("quit")];
    let (finished, ui, records) = run(&f, &plan, &[], answers);
    assert_eq!(finished.end, End::Stopped);
    assert_eq!(ui.asked, ["run", "gate", "gate"]);
    assert!(
        ui.shown.iter().any(|l| l.contains("was skipped by you")),
        "{:?}",
        ui.shown
    );
    assert_eq!(subjects(&f), ["fixture"]);
    let skipped = records.iter().any(|r| {
        matches!(
            r,
            Record::ActionEnd {
                outcome: Outcome::SkippedUser,
                ..
            }
        )
    });
    assert!(skipped);
}

#[test]
fn a_closed_gate_opens_only_with_a_typed_yes_and_is_journaled() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let failing = ir.script("$gate = 0; git rev-parse -q --verify refs/heads/none");
    let failing = expecting(
        failing,
        ExitExpect::Success,
        ScriptIntent::Gate,
        false,
        &mut ir,
    );
    let manifest = ir.manifest(&[("a.txt", ChangeKind::Create, W)]);
    let steps = vec![
        vec![ir.write("a.txt", "a\n", WriteMode::Create)],
        vec![failing],
        vec![ir.commit("git add -- a.txt; git commit -q -m 'T1'")],
    ];
    let plan = one(&mut ir, manifest, steps);
    let answers = vec![
        choice(Choice::Run),
        choice(Choice::Accept),
        typed("Yes"),
        typed("yes"),
        choice(Choice::Run),
    ];
    let (finished, ui, records) = run(&f, &plan, &[], answers);
    assert_eq!(
        finished.end,
        End::Completed,
        "{:?} {:?}",
        finished.reason,
        ui.shown
    );
    assert_eq!(ui.asked, ["run", "mismatch", "gate", "gate", "run"]);
    let override_reasons = records.iter().find_map(|r| match r {
        Record::GateOverride { reasons, .. } => Some(reasons.clone()),
        _ => None,
    });
    let reasons = override_reasons.unwrap_or_else(|| panic!("no gate-override"));
    assert_eq!(reasons.len(), 1);
    assert!(reasons[0].contains("accepted"), "{reasons:?}");
    assert_eq!(subjects(&f), ["T1", "fixture"]);
}

#[test]
fn prose_confirms_and_no_closes_the_gate() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let verify = ir.script("'all pass'");
    let verify = expecting(
        verify,
        ExitExpect::Success,
        ScriptIntent::Verify,
        true,
        &mut ir,
    );
    let steps = vec![
        vec![verify],
        vec![ir.commit("git commit -q --allow-empty -m 'T1'")],
    ];
    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
    let answers = vec![choice(Choice::No), typed("quit")];
    let (finished, ui, _) = run(&f, &plan, AUTO, answers);
    assert_eq!(finished.end, End::Stopped);
    assert_eq!(ui.asked, ["confirm", "gate"]);
    assert!(
        ui.shown.iter().any(|l| l.contains("all pass")),
        "{:?}",
        ui.shown
    );
}

#[test]
fn a_commit_that_does_not_move_head_is_a_mismatch() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let steps = vec![vec![ir.commit("'nothing committed'")]];
    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
    let (finished, ui, _) = run(&f, &plan, AUTO, vec![choice(Choice::Quit)]);
    assert_eq!(finished.end, End::Stopped);
    assert_eq!(ui.asked, ["mismatch"]);
    assert!(
        ui.shown.iter().any(|l| l.contains("HEAD did not move")),
        "{:?}",
        ui.shown
    );
}

#[test]
fn a_dead_session_offers_retry_and_the_new_session_is_announced() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let first = bash(ir.script("x=1; exit 3"));
    let second = bash(ir.script("echo \"x=${x:-unset}\""));
    let steps = vec![vec![first], vec![second]];
    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
    let answers = vec![
        choice(Choice::Accept),
        choice(Choice::Retry),
        choice(Choice::Continue),
        choice(Choice::Quit),
    ];
    let (finished, ui, records) = run(&f, &plan, AUTO, answers);
    assert_eq!(finished.end, End::Stopped);
    assert_eq!(
        ui.asked,
        ["mismatch", "mismatch", "session", "mismatch"],
        "{:?}",
        ui.shown
    );
    let restarted = records
        .iter()
        .any(|r| matches!(r, Record::Session { reason, .. } if reason == "restarted"));
    assert!(restarted);
    assert!(
        ui.shown
            .iter()
            .any(|l| l.contains("retry starts a new one"))
    );
}

#[test]
fn a_working_directory_outside_the_repository_asks_before_the_next_block() {
    let f = Fixture::greeter(false);
    let outside = f.outside().display().to_string();
    let mut ir = Ir::default();
    let steps = vec![vec![
        ir.script(&format!("Set-Location '{outside}'")),
        ir.script("'here'"),
    ]];
    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
    let (finished, ui, _) = run(&f, &plan, AUTO, vec![choice(Choice::Continue)]);
    assert_eq!(finished.end, End::Completed, "{:?}", ui.shown);
    assert_eq!(ui.asked, ["outside"]);
}

#[test]
fn a_manual_waits_and_an_unscoped_write_asks_even_with_auto() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let manifest = ir.manifest(&[("a.txt", ChangeKind::Create, W)]);
    let steps = vec![
        vec![ir.manual("edit it by hand")],
        vec![ir.write("b.txt", "b\n", WriteMode::Create)],
    ];
    let plan = one(&mut ir, manifest, steps);
    let answers = vec![choice(Choice::Continue), choice(Choice::Run)];
    let (finished, ui, _) = run(&f, &plan, AUTO, answers);
    assert_eq!(finished.end, End::Completed, "{:?}", ui.shown);
    assert_eq!(ui.asked, ["manual", "unscoped"]);
    assert_eq!(f.read("b.txt").as_deref(), Some(&b"b\n"[..]));
}

#[test]
fn dirt_outside_the_task_asks_before_the_commit_and_a_missing_generated_file_closes_the_gate() {
    let f = Fixture::greeter(false);
    f.write("notes.txt", b"mine\n");
    let mut ir = Ir::default();
    let manifest = ir.manifest(&[("gen.txt", ChangeKind::Generated, W)]);
    let steps = vec![
        vec![ir.generated("gen.txt")],
        vec![ir.commit("git commit -q --allow-empty -m 'T1'")],
    ];
    let plan = one(&mut ir, manifest, steps);
    let answers = vec![choice(Choice::Continue), typed("quit")];
    let (finished, ui, _) = run(&f, &plan, AUTO, answers);
    assert_eq!(finished.end, End::Stopped);
    assert_eq!(ui.asked, ["dirt", "gate"]);
    assert!(
        ui.shown.iter().any(|l| l.contains("?? notes.txt")),
        "{:?}",
        ui.shown
    );
    assert!(
        ui.shown
            .iter()
            .any(|l| l.contains("`gen.txt` should have been generated"))
    );
}

#[test]
fn a_preflight_error_stops_before_anything_runs() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let manifest = ir.manifest(&[("src/lib.rs", ChangeKind::Create, W)]);
    let steps = vec![vec![ir.write("src/lib.rs", "x\n", WriteMode::Create)]];
    let plan = one(&mut ir, manifest, steps);
    let (finished, ui, records) = run(&f, &plan, AUTO, vec![]);
    assert_eq!(finished.end, End::Stopped);
    assert!(
        ui.shown.iter().any(|l| l.contains("create-exists")),
        "{:?}",
        ui.shown
    );
    assert_eq!(kinds(&records), ["run", "run-end"]);
}

#[test]
fn a_gate_from_a_session_that_ended_runs_again_before_the_commit() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let gate = bash(ir.script("g=0"));
    let gate = expecting(
        gate,
        ExitExpect::Success,
        ScriptIntent::Gate,
        false,
        &mut ir,
    );
    let dies_once = bash(ir.script("test -e .git/died || { echo x > .git/died; exit 3; }"));
    let commit = bash(ir.commit("test \"$g\" = 0 && git commit -q --allow-empty -m T1"));
    let steps = vec![vec![gate], vec![dies_once], vec![commit]];
    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
    let answers = vec![choice(Choice::Retry), choice(Choice::Continue)];
    let (finished, ui, _) = run(&f, &plan, AUTO, answers);
    assert_eq!(finished.end, End::Completed, "{:?}", ui.shown);
    assert_eq!(ui.asked, ["mismatch", "session"]);
    let rerun = ui.shown.iter().any(|l| l.contains("it runs again"));
    assert!(rerun, "{:?}", ui.shown);
    assert_eq!(subjects(&f), ["T1", "fixture"]);
}
```

**Step 2 — verify**

Targeted: the runner's tests, with both shells required:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-exec --test runner
```

Expected (exit code 0; summary lines):

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-exec/src/git.rs crates/planrun-exec/src/lib.rs crates/planrun-exec/src/policy/decide.rs crates/planrun-exec/src/runner/gate.rs crates/planrun-exec/src/runner/mod.rs crates/planrun-exec/src/runner/script.rs crates/planrun-exec/src/runner/sessions.rs crates/planrun-exec/src/runner/task.rs crates/planrun-exec/src/ui.rs crates/planrun-exec/tests/runner.rs
  $verified = [ordered]@{
      'crates/planrun-exec/src/git.rs' = '0604446b38a75801b85af1b1eb64a5044dc5ab1d'
      'crates/planrun-exec/src/lib.rs' = '12e1202a9599465386e4996d7df8b02a3fe0cef7'
      'crates/planrun-exec/src/policy/decide.rs' = 'e701dcf8e9dea6706e769d2c62223afd34c73c8b'
      'crates/planrun-exec/src/runner/gate.rs' = 'd57ac6c9a9ff94ac97918cc93abce3acf8216e25'
      'crates/planrun-exec/src/runner/mod.rs' = '569d3ec57da2066d05970cae3743000b7ec7f52c'
      'crates/planrun-exec/src/runner/script.rs' = '159378c6f73189f29aae8375771155b946d3ffbf'
      'crates/planrun-exec/src/runner/sessions.rs' = 'f7b81188e8c8d4e98a91290a65cf1143afbf3976'
      'crates/planrun-exec/src/runner/task.rs' = '9c43d4351b081befa32697bba99c0e3185dba1a8'
      'crates/planrun-exec/src/ui.rs' = 'd5b7c2d715795b7309bcc6e628017eaa330c61de'
      'crates/planrun-exec/tests/runner.rs' = '84f382c4891bffc77d73e6ffa397f7eda210ec2f'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P5-T6: the runner: tasks, step transactions with the journal, scripts in sessions, prompts and the commit gate" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P5-T7: resume: the journal's last chain against the disk and HEAD, the interrupted task run again

**Goal:** Resume (D77): `runner/resume.rs` reads the journal's last chain into an `Analysis`, stops on a changed fingerprint, an `index.lock` or an unexpected HEAD, finds a Commit that was made, and runs the interrupted task again: prepared steps compared with the disk (Done skipped, NotDone written, Unknown restored or accepted), scripts that ran or were running asked about by class, Manuals acknowledged before skipped. `preflight_resumed` leaves out the steps found done, and dirt that a remaining file action writes over outside the verified steps is `resume-dirt`, asked about once. A resume's own `run` record never sets the recorded HEAD, and an accepted HEAD move is journaled at once. Tests crash the runner from the journal's observer at each seam.

**Files**

- Modify `crates/planrun-exec/src/preflight.rs`
- Modify `crates/planrun-exec/src/runner/mod.rs`
- Create `crates/planrun-exec/src/runner/resume.rs`
- Modify `crates/planrun-exec/src/runner/script.rs`
- Modify `crates/planrun-exec/src/runner/task.rs`
- Create `crates/planrun-exec/tests/resume.rs`
- Modify `docs/decisions.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-exec/src/preflight.rs`, `crates/planrun-exec/src/runner/mod.rs`, `crates/planrun-exec/src/runner/script.rs`, `crates/planrun-exec/src/runner/task.rs`, `docs/decisions.md`.

**Step 1 — write the files**

File `crates/planrun-exec/src/preflight.rs` (exact diff; save the block as `$env:TEMP\p5-t7-preflight-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/preflight.rs b/crates/planrun-exec/src/preflight.rs
index 68eac14..1701b7b 100644
--- a/crates/planrun-exec/src/preflight.rs
+++ b/crates/planrun-exec/src/preflight.rs
@@ -100,6 +100,24 @@ struct Walk<'t, 'w> {
     unrun: bool,
     /// A real run's preflight: the repository is about to be changed, not simulated (D78).
     real: bool,
+    /// Resume (D77): the file actions to leave out, as (step, action) positions.
+    skip: Vec<(usize, usize)>,
+    /// Resume: the paths the interrupted run wrote, whose manifest checks are left out.
+    own: Vec<RepoPath>,
+}
+
+/// What resume tells preflight about the interrupted task (D77): the file actions it found
+/// done on disk, the paths the interrupted run may have changed (all of the task's), and the
+/// paths of the steps found done or accepted.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Resumed {
+    /// The (step, action) positions of the file actions done on disk.
+    pub done: Vec<(usize, usize)>,
+    /// The task's paths and every path the journal prepared.
+    pub own: Vec<RepoPath>,
+    /// The paths of the steps found done on disk or accepted: their dirt is the interrupted
+    /// run's own writes, verified.
+    pub verified: Vec<RepoPath>,
 }
 
 /// The dry run of `plan` (D55): parse and checks are done; this preflights every task against
@@ -113,6 +131,8 @@ pub fn dry_run(ws: &Workspace, plan: &Plan) -> Result<DryRun, WorkspaceError> {
         handlers: HandlerSet::files(),
         unrun: false,
         real: false,
+        skip: Vec::new(),
+        own: Vec::new(),
     };
     let tasks = plan
         .tasks
@@ -139,13 +159,49 @@ pub fn dry_run(ws: &Workspace, plan: &Plan) -> Result<DryRun, WorkspaceError> {
 /// dirt in the task's paths, the manifest checks, and the file actions up to the first script
 /// or Manual. Every error finding stops the task.
 pub fn preflight(ws: &Workspace, plan: &Plan, task: &Task) -> Result<Vec<Finding>, WorkspaceError> {
+    preflight_resumed(ws, plan, task, &Resumed::default())
+}
+
+/// The preflight of a task that resume runs again (D77): the file actions done on disk are
+/// left out, the manifest checks of the paths the interrupted run may have written too, and
+/// dirt in those paths is a warning. Dirt that a remaining file action would write over, in a
+/// path no verified step wrote, is `resume-dirt`: it may be the user's, and the runner asks.
+pub fn preflight_resumed(
+    ws: &Workspace,
+    plan: &Plan,
+    task: &Task,
+    resumed: &Resumed,
+) -> Result<Vec<Finding>, WorkspaceError> {
     let mut found = dirt(ws, std::iter::once(task))?;
+    let remaining = remaining_writes(task, &resumed.done);
+    for f in &mut found {
+        let Some(path) = f.path.as_ref() else {
+            continue;
+        };
+        if f.code != "path-dirty" || !resumed.own.contains(path) {
+            continue;
+        }
+        f.severity = Severity::Warning;
+        if resumed.verified.contains(path) {
+            f.message = format!("{} (the interrupted run wrote it; verified)", f.message);
+        } else if remaining.contains(path) {
+            f.code = "resume-dirt";
+            f.message = format!(
+                "{} (not a verified write of the interrupted run; a remaining file action                  writes it)",
+                f.message
+            );
+        } else {
+            f.message = format!("{} (the interrupted run may have written it)", f.message);
+        }
+    }
     let mut tree = VirtualTree::new(ws)?;
     let mut walk = Walk {
         tree: &mut tree,
         handlers: HandlerSet::files(),
         unrun: false,
         real: true,
+        skip: resumed.done.clone(),
+        own: resumed.own.clone(),
     };
     let report = walk.task(plan, 0, task, true);
     found.extend(report.manifest);
@@ -166,7 +222,10 @@ impl Walk<'_, '_> {
             .collect();
         let mut actions = Vec::new();
         for (step, s) in task.steps.iter().enumerate() {
-            for action in &s.actions {
+            for (at, action) in s.actions.iter().enumerate() {
+                if self.skip.contains(&(step, at)) {
+                    continue;
+                }
                 if stop_at_unrun && self.unrun {
                     return TaskReport {
                         task: index,
@@ -195,7 +254,7 @@ impl Walk<'_, '_> {
         let ManifestPath::File(path) = &entry.path else {
             return None;
         };
-        if entry.origin == PathOrigin::Bare {
+        if entry.origin == PathOrigin::Bare || self.own.contains(path) {
             return None;
         }
         let must_exist = match entry.change {
@@ -383,6 +442,25 @@ fn touched(task: &Task) -> impl Iterator<Item = &RepoPath> {
     actions.chain(task.manifest.entries.iter().filter_map(|e| e.path.file()))
 }
 
+/// The paths the task's file actions write, leaving out those at the `done` positions.
+fn remaining_writes(task: &Task, done: &[(usize, usize)]) -> Vec<RepoPath> {
+    let mut paths: Vec<RepoPath> = Vec::new();
+    for (step, s) in task.steps.iter().enumerate() {
+        for (at, action) in s.actions.iter().enumerate() {
+            if done.contains(&(step, at)) {
+                continue;
+            }
+            let Some(path) = file_path(&action.kind) else {
+                continue;
+            };
+            if !paths.contains(path) {
+                paths.push(path.clone());
+            }
+        }
+    }
+    paths
+}
+
 /// Real dirt (D55), from one `git status`: a touched path with uncommitted changes is
 /// `path-dirty`; any other dirt is one `tree-dirty` warning.
 fn dirt<'p>(
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t7-preflight-rs.patch"
Remove-Item "$env:TEMP\p5-t7-preflight-rs.patch"
```

File `crates/planrun-exec/src/runner/mod.rs` (exact diff; save the block as `$env:TEMP\p5-t7-mod-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/runner/mod.rs b/crates/planrun-exec/src/runner/mod.rs
index 569d3ec..e292f97 100644
--- a/crates/planrun-exec/src/runner/mod.rs
+++ b/crates/planrun-exec/src/runner/mod.rs
@@ -8,6 +8,7 @@
 //! P4's [`Policy`] decides; the runner prompts (D69) and never turns one decision into another.
 
 mod gate;
+pub mod resume;
 mod script;
 mod sessions;
 mod task;
@@ -91,6 +92,8 @@ pub struct Runner<'r> {
     notice: Option<Vec<String>>,
     /// Tasks whose scripts ran before, for the notice.
     earlier: Vec<String>,
+    /// What resume replays of the interrupted task (D77).
+    replay: Option<resume::Replay>,
 }
 
 impl<'r> Runner<'r> {
@@ -114,13 +117,14 @@ impl<'r> Runner<'r> {
             sessions,
             notice: None,
             earlier: Vec::new(),
+            replay: None,
         }
     }
 
     /// Runs the plan's tasks from `--from` to `--to`. Every way it ends is journaled
     /// (`run-end`), except a kill.
     pub fn run(&mut self) -> Result<Finished> {
-        let result = self.start().and_then(|()| self.tasks());
+        let result = self.start();
         let (end, reason) = match &result {
             Ok(Flow::Go) => (End::Completed, None),
             Ok(Flow::Stop(reason)) => (End::Stopped, Some(reason.clone())),
@@ -135,7 +139,21 @@ impl<'r> Runner<'r> {
         Ok(Finished { end, reason })
     }
 
-    fn start(&mut self) -> Result<()> {
+    fn start(&mut self) -> Result<Flow> {
+        self.start_record(false)?;
+        let from = self.config.from.unwrap_or(0);
+        if from > 0 {
+            self.earlier = self.plan.tasks[..from]
+                .iter()
+                .map(|t| t.id.raw.to_string())
+                .collect();
+            self.notice = Some(self.notice_text("the run starts at a later task (--from)"));
+        }
+        self.tasks_from(from)
+    }
+
+    /// The `run` record: a run, or a resume (D76).
+    fn start_record(&mut self, resume: bool) -> Result<()> {
         let policy = &self.config.policy;
         let auto = policy.auto().iter().map(|c| class_name(*c)).collect();
         let record = Record::Run {
@@ -154,23 +172,14 @@ impl<'r> Runner<'r> {
                 .config
                 .to
                 .map(|i| self.plan.tasks[i].id.raw.to_string()),
-            resume: false,
+            resume,
             planrun: env!("CARGO_PKG_VERSION").to_owned(),
         };
-        self.append(record)?;
-        let from = self.config.from.unwrap_or(0);
-        if from > 0 {
-            self.earlier = self.plan.tasks[..from]
-                .iter()
-                .map(|t| t.id.raw.to_string())
-                .collect();
-            self.notice = Some(self.notice_text("the run starts at a later task (--from)"));
-        }
-        Ok(())
+        self.append(record)
     }
 
-    fn tasks(&mut self) -> Result<Flow> {
-        let from = self.config.from.unwrap_or(0);
+    /// Runs the tasks from position `from` to `--to`.
+    fn tasks_from(&mut self, from: usize) -> Result<Flow> {
         let to = self
             .config
             .to
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t7-mod-rs.patch"
Remove-Item "$env:TEMP\p5-t7-mod-rs.patch"
```

File `crates/planrun-exec/src/runner/resume.rs` (full contents):

```rust
//! Resume (docs/architecture.md §6.6; decisions D14, D57, D77): what the journal says, checked
//! against the disk and HEAD, never trusted alone.
//!
//! The last chain of the journal (a `run`, and every `resume` after it) is read into an
//! [`Analysis`]. Resume stops when the plan's fingerprint differs, when `.git/index.lock`
//! exists, and when HEAD is not what the journal last recorded (except for the one action that
//! was running: a Commit whose new HEAD sits on the recorded one and holds the task's files is
//! done; any other action that moved HEAD is the user's call). A resume's own `run` record
//! never sets the recorded HEAD: only the records written after HEAD was checked or accepted
//! do, so a resume that stopped leaves the check as it found it. The interrupted task runs again
//! from its start in a new session: file steps done on disk are verified, not written again;
//! scripts run again under their decisions, Modify and Destructive ones asking first; a script
//! that was running asks whatever its class.

use std::collections::HashMap;

use planrun_model::RepoPath;

use super::{Answer, Flow, Result, Runner, Typed, task_paths};
use crate::DoneState;
use crate::done::{Seen, remove_temps, restorable, restore_step, step_state};
use crate::git::Head;
use crate::journal::{At, End, Line, Outcome, PathBlob, Record};
use crate::preflight::Resumed;
use crate::transaction::StepRecord;
use crate::ui::{Choice, Question};

/// What the run record of the chain says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunInfo {
    /// The plan's path as recorded.
    pub plan: String,
    /// The plan's fingerprint.
    pub fingerprint: String,
    /// `--auto`.
    pub auto: Vec<String>,
    /// `--run-expected-failures`.
    pub run_expected_failures: bool,
    /// `--from`.
    pub from: Option<String>,
    /// `--to`.
    pub to: Option<String>,
}

/// What became of an action, as journaled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Past {
    /// `action-start` without its end: it was running when planrun stopped.
    Running {
        /// `script`, `commit`, `manual`, `generated`.
        kind: String,
        /// HEAD before it.
        head: Head,
        /// For a Commit, the task's paths it was to hold.
        expect: Vec<PathBlob>,
    },
    /// It ended so.
    Ended(Outcome),
}

/// A step the journal prepared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    /// Where.
    pub at: At,
    /// What it writes.
    pub step: StepRecord,
    /// `step-written`, `step-verified` or `step-accepted` followed it.
    pub written: bool,
}

/// The last chain of a journal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    /// The chain's first `run` record.
    pub run: RunInfo,
    /// The tasks with a `task-end`, in order.
    pub done: Vec<String>,
    /// The task with a `task-start` and no `task-end`, if any.
    pub interrupted: Option<String>,
    /// HEAD as last recorded: by the first `run`, `task-start`, `action-end`, `head-seen` or
    /// `task-end`; never by a resume's `run` record, which is written before HEAD is checked.
    pub head: Head,
    /// How the chain's last process ended; `None` when it was killed.
    pub end: Option<End>,
    /// The interrupted task's prepared steps, the latest per step.
    pub prepared: Vec<Prepared>,
    /// The interrupted task's actions, the latest record per action.
    pub past: Vec<(At, Past)>,
}

impl Analysis {
    /// Whether there is anything to resume: the chain's last process did not complete it.
    pub fn resumable(&self) -> bool {
        self.end != Some(End::Completed)
    }

    /// The action that was running when planrun stopped, if any.
    pub fn running(&self) -> Option<(&At, &Past)> {
        self.past
            .iter()
            .find(|(_, p)| matches!(p, Past::Running { .. }))
            .map(|(at, p)| (at, p))
    }
}

/// Reads the journal's last chain. `None` when the journal has no run.
pub fn analyse(lines: &[Line]) -> Option<Analysis> {
    let start = lines
        .iter()
        .rposition(|l| matches!(l.record, Record::Run { resume: false, .. }))?;
    let mut a: Option<Analysis> = None;
    let mut prepared: Vec<Prepared> = Vec::new();
    let mut past: Vec<(At, Past)> = Vec::new();
    for line in &lines[start..] {
        let record = &line.record;
        if let Record::Run {
            plan,
            fingerprint,
            head,
            auto,
            run_expected_failures,
            from,
            to,
            resume,
            ..
        } = record
            && !resume
        {
            a = Some(Analysis {
                run: RunInfo {
                    plan: plan.clone(),
                    fingerprint: fingerprint.clone(),
                    auto: auto.clone(),
                    run_expected_failures: *run_expected_failures,
                    from: from.clone(),
                    to: to.clone(),
                },
                done: Vec::new(),
                interrupted: None,
                head: head.clone(),
                end: None,
                prepared: Vec::new(),
                past: Vec::new(),
            });
        }
        let Some(a) = a.as_mut() else {
            continue;
        };
        apply(a, record, &mut prepared, &mut past);
    }
    let mut a = a?;
    let task = a.interrupted.clone();
    a.prepared = prepared
        .into_iter()
        .filter(|p| Some(&p.at.task) == task.as_ref())
        .collect();
    a.past = past
        .into_iter()
        .filter(|(at, _)| Some(&at.task) == task.as_ref())
        .collect();
    Some(a)
}

fn upsert<T>(list: &mut Vec<(At, T)>, at: &At, value: T) {
    list.retain(|(k, _)| k != at);
    list.push((at.clone(), value));
}

fn apply(
    a: &mut Analysis,
    record: &Record,
    prepared: &mut Vec<Prepared>,
    past: &mut Vec<(At, Past)>,
) {
    let mark = |prepared: &mut Vec<Prepared>, at: &At, written: bool| {
        if let Some(p) = prepared.iter_mut().find(|p| p.at == *at) {
            p.written = written;
        }
    };
    match record {
        Record::Run { head, resume, .. } => {
            if !resume {
                a.head = head.clone();
            }
            a.end = None;
        }
        Record::HeadSeen { head } => a.head = head.clone(),
        Record::TaskStart { task, head } => {
            a.interrupted = Some(task.clone());
            a.head = head.clone();
        }
        Record::TaskEnd { task, head } => {
            if !a.done.contains(task) {
                a.done.push(task.clone());
            }
            a.interrupted = None;
            a.head = head.clone();
        }
        Record::StepPrepared { at, step } => {
            prepared.retain(|p| p.at != *at);
            prepared.push(Prepared {
                at: at.clone(),
                step: step.clone(),
                written: false,
            });
        }
        Record::StepWritten { at } | Record::StepVerified { at } | Record::StepAccepted { at } => {
            mark(prepared, at, true);
        }
        Record::StepRestored { at } => mark(prepared, at, false),
        Record::ActionStart {
            at,
            kind,
            head,
            expect,
            ..
        } => {
            let running = Past::Running {
                kind: kind.clone(),
                head: head.clone(),
                expect: expect.clone(),
            };
            upsert(past, at, running);
        }
        Record::ActionEnd {
            at, outcome, head, ..
        } => {
            upsert(past, at, Past::Ended(*outcome));
            a.head = head.clone();
        }
        Record::RunEnd { end, .. } => a.end = Some(*end),
        _ => {}
    }
}

/// What the runner replays for the interrupted task.
#[derive(Debug, Default)]
pub(crate) struct Replay {
    pub task: usize,
    pub prepared: Vec<Prepared>,
    pub past: HashMap<(usize, usize), Past>,
    /// File groups found done on disk (or accepted), by their first action.
    pub done_groups: Vec<(usize, usize)>,
    /// A Commit that was running and is found made.
    pub commit_done: Option<(usize, usize)>,
}

fn key(at: &At) -> (usize, usize) {
    (at.step, at.action)
}

impl Runner<'_> {
    /// Resumes the chain `analysis` describes (D14, D77).
    pub fn resume(&mut self, analysis: &Analysis) -> Result<super::Finished> {
        let result = self.start_resume(analysis);
        let (end, reason) = match &result {
            Ok(Flow::Go) => (End::Completed, None),
            Ok(Flow::Stop(reason)) => (End::Stopped, Some(reason.clone())),
            Err(e) => (End::Stopped, Some(e.to_string())),
        };
        let ended = self.append(Record::RunEnd {
            end,
            reason: reason.clone(),
        });
        result?;
        ended?;
        Ok(super::Finished { end, reason })
    }

    fn start_resume(&mut self, analysis: &Analysis) -> Result<Flow> {
        self.start_record(true)?;
        let plan = self.plan;
        if analysis.run.fingerprint != *plan.source.fingerprint {
            return Ok(Flow::Stop(format!(
                "the plan changed since the run: its fingerprint is {}, the journal's {}",
                plan.source.fingerprint, analysis.run.fingerprint
            )));
        }
        let lock = self.ws.git().git_dir()?.join("index.lock");
        if lock.exists() {
            return Ok(Flow::Stop(format!(
                "{} exists: another git command is running, or one crashed; planrun never \
                 removes it",
                lock.display()
            )));
        }
        let index_of = |id: &str| plan.tasks.iter().position(|t| &*t.id.raw == id);
        let mut replay = Replay::default();
        if let Some(task) = analysis.interrupted.as_deref().and_then(index_of) {
            replay.task = task;
            replay.prepared = analysis.prepared.clone();
            replay.past = analysis
                .past
                .iter()
                .map(|(at, p)| (key(at), p.clone()))
                .collect();
        }
        if let Flow::Stop(why) = self.check_head(analysis, &mut replay)? {
            return Ok(Flow::Stop(why));
        }
        let after_done = analysis
            .done
            .iter()
            .filter_map(|id| index_of(id))
            .max()
            .map(|i| i + 1);
        let start = match analysis.interrupted.as_deref().and_then(index_of) {
            Some(i) => i,
            None => after_done
                .or_else(|| analysis.run.from.as_deref().and_then(index_of))
                .unwrap_or(0),
        };
        self.earlier = plan.tasks[..start]
            .iter()
            .map(|t| t.id.raw.to_string())
            .collect();
        self.notice = Some(self.notice_text("planrun resumes an interrupted run"));
        if analysis.interrupted.is_some() {
            self.replay = Some(replay);
        }
        self.tasks_from(start)
    }

    /// HEAD against the journal (D14, D80): equal, or moved by the one action that was
    /// running. Where a moved HEAD is accepted, the acceptance is journaled at once (an
    /// `action-end`, or `head-seen` for a retry), so a crash after it resumes from it.
    fn check_head(&mut self, analysis: &Analysis, replay: &mut Replay) -> Result<Flow> {
        let now = self.head()?;
        if now == analysis.head {
            if let Some((at, Past::Running { kind, .. })) = analysis.running()
                && kind == "commit"
            {
                replay.past.remove(&key(at));
            }
            return Ok(Flow::Go);
        }
        let stop = Flow::Stop(format!(
            "HEAD is {} ({}), but the journal last recorded {} ({}): something else changed \
             the repository",
            now.commit.as_deref().unwrap_or("none"),
            now.reference.as_deref().unwrap_or("detached"),
            analysis.head.commit.as_deref().unwrap_or("none"),
            analysis.head.reference.as_deref().unwrap_or("detached"),
        ));
        let Some((at, Past::Running { kind, head, expect })) = analysis.running() else {
            return Ok(stop);
        };
        let Some(commit) = now.commit.as_deref() else {
            return Ok(stop);
        };
        let on_top = self.ws.git().parent(commit)?.as_deref() == head.commit.as_deref();
        if !on_top || now.reference != head.reference {
            return Ok(stop);
        }
        if kind == "commit"
            && self
                .commit_problem(head.commit.as_deref(), expect)?
                .is_none()
        {
            self.ui.status(&format!(
                "  the commit of {} was made before the interruption",
                at.task
            ));
            self.accepted(at, Outcome::Passed, &now)?;
            replay.commit_done = Some(key(at));
            return Ok(Flow::Go);
        }
        let question = Question {
            kind: "head-moved",
            at: None,
            text: vec![
                format!(
                    "HEAD moved by one commit while the {kind} at step {}, action {} of {} was \
                     running",
                    at.step + 1,
                    at.action + 1,
                    at.task
                ),
                "keep treats it as done; retry runs it again".to_owned(),
            ],
            // No `[o]`: it journals HEAD as seen (D80), which would pass this check unanswered.
            choices: vec![Choice::Keep, Choice::Retry, Choice::Diff, Choice::Quit],
            typed: false,
        };
        Ok(match self.ask(question)? {
            Answer::Choice(Choice::Keep) => {
                self.accepted(at, Outcome::Kept, &now)?;
                replay.past.insert(key(at), Past::Ended(Outcome::Kept));
                Flow::Go
            }
            Answer::Choice(Choice::Retry) => {
                self.append(Record::HeadSeen { head: now })?;
                replay.past.remove(&key(at));
                Flow::Go
            }
            Answer::Stop(why) => Flow::Stop(why),
            _ => Flow::Stop("quit".to_owned()),
        })
    }

    /// The end of the action that was running, journaled when resume accepts the HEAD it
    /// moved: a later resume finds the action ended and HEAD recorded.
    fn accepted(&mut self, at: &At, outcome: Outcome, head: &Head) -> Result<()> {
        self.append(Record::ActionEnd {
            at: at.clone(),
            outcome,
            status: None,
            errored: false,
            cwd: None,
            tail: vec!["accepted by resume: HEAD moved while it was running".to_owned()],
            head: head.clone(),
        })
    }

    /// The interrupted task's prepared steps against the disk (D14, D77): temp files removed,
    /// Done steps not written again, an Unknown step restored or accepted by the user. Returns
    /// what preflight must leave out.
    pub(crate) fn replay_steps(
        &mut self,
        index: usize,
    ) -> Result<std::result::Result<Resumed, String>> {
        let Some(replay) = self.replay.as_ref().filter(|r| r.task == index) else {
            return Ok(Ok(Resumed::default()));
        };
        let prepared = replay.prepared.clone();
        let task = &self.plan.tasks[index];
        // Any path of the task may have been changed by the interrupted run, by a step or by a
        // script: its dirt is a warning, and its manifest checks are left out. Dirt outside the
        // verified steps that a remaining file action writes over is asked about (D77).
        let mut resumed = Resumed {
            done: Vec::new(),
            own: task_paths(task),
            verified: Vec::new(),
        };
        for p in &prepared {
            for removed in remove_temps(self.ws, &p.step) {
                self.ui.status(&format!(
                    "  removed a temp file of the interrupted run: {removed}"
                ));
            }
            for w in &p.step.writes {
                if !resumed.own.contains(&w.path) {
                    resumed.own.push(w.path.clone());
                }
            }
            let found = step_state(self.ws, &p.step)?;
            let done = match found.state {
                DoneState::Done => true,
                DoneState::NotDone => false,
                DoneState::Unknown => match self.unknown_step(&p.at, &p.step, &found.paths)? {
                    Ok(done) => done,
                    Err(why) => return Ok(Err(why)),
                },
            };
            if !done {
                continue;
            }
            for w in &p.step.writes {
                if !resumed.verified.contains(&w.path) {
                    resumed.verified.push(w.path.clone());
                }
            }
            let step = &task.steps[p.at.step];
            let group = step.actions[p.at.action..]
                .iter()
                .take_while(|a| super::is_file_action(a))
                .count();
            resumed
                .done
                .extend((p.at.action..p.at.action + group).map(|a| (p.at.step, a)));
            if let Some(r) = self.replay.as_mut() {
                r.done_groups.push(key(&p.at));
            }
        }
        Ok(Ok(resumed))
    }

    /// A step that is neither done nor undone on disk (D14): restore it from its pre-images
    /// when they are all there (D57), accept it as done with a typed `yes`, or stop.
    fn unknown_step(
        &mut self,
        at: &At,
        step: &StepRecord,
        paths: &[(RepoPath, Seen)],
    ) -> Result<std::result::Result<bool, String>> {
        let can_restore = restorable(self.ws, step)?;
        let mut text = vec![format!(
            "step {} of {} is partly written (the run was interrupted while writing it):",
            at.step + 1,
            at.task
        )];
        text.extend(paths.iter().map(|(p, s)| format!("  {p}: {}", seen(*s))));
        let mut choices = Vec::new();
        if can_restore {
            choices.push(Choice::Restore);
        } else {
            text.push("its pre-images are gone (pruned by git gc, D57), or a git delete already left the index: it cannot be restored".to_owned());
        }
        choices.extend([Choice::Accept, Choice::Diff, Choice::Shell, Choice::Quit]);
        let question = Question {
            kind: "unknown",
            at: None,
            text,
            choices,
            typed: false,
        };
        match self.ask(question)? {
            Answer::Choice(Choice::Restore) => {
                let unrestored = restore_step(self.ws, step);
                if !unrestored.is_empty() {
                    let names: Vec<&str> = unrestored.iter().map(RepoPath::as_str).collect();
                    return Ok(Err(format!("could not restore {}", names.join(", "))));
                }
                self.append(Record::StepRestored { at: at.clone() })?;
                self.ui
                    .status("  restored from the pre-images; the step is written again");
                Ok(Ok(false))
            }
            Answer::Choice(Choice::Accept) => {
                let question = Question {
                    kind: "accept-done",
                    at: None,
                    text: vec![
                        "accepting the step as done overrides the disk check".to_owned(),
                        "type `yes` to accept it, or `quit`".to_owned(),
                    ],
                    choices: Vec::new(),
                    typed: true,
                };
                match self.typed_yes(question, false)? {
                    Typed::Yes => {
                        self.append(Record::StepAccepted { at: at.clone() })?;
                        Ok(Ok(true))
                    }
                    Typed::Stop(why) => Ok(Err(why)),
                    _ => Ok(Err("quit".to_owned())),
                }
            }
            Answer::Stop(why) => Ok(Err(why)),
            _ => Ok(Err("quit".to_owned())),
        }
    }

    /// The replay entry of the action at (step, action) in task `index`, taken once.
    pub(crate) fn take_past(&mut self, index: usize, at: (usize, usize)) -> Option<Past> {
        let replay = self.replay.as_mut().filter(|r| r.task == index)?;
        replay.past.remove(&at)
    }

    /// Whether the file group at (step, action) of task `index` was found done on disk.
    pub(crate) fn group_done(&self, index: usize, at: (usize, usize)) -> bool {
        self.replay
            .as_ref()
            .is_some_and(|r| r.task == index && r.done_groups.contains(&at))
    }

    /// Whether the Commit at (step, action) of task `index` was made before the interruption.
    pub(crate) fn commit_made(&self, index: usize, at: (usize, usize)) -> bool {
        self.replay
            .as_ref()
            .is_some_and(|r| r.task == index && r.commit_done == Some(at))
    }
}

fn seen(s: Seen) -> &'static str {
    match s {
        Seen::Post => "written",
        Seen::Pre => "not written",
        Seen::Neither => "neither as before nor as planned",
    }
}
```

File `crates/planrun-exec/src/runner/script.rs` (exact diff; save the block as `$env:TEMP\p5-t7-script-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/runner/script.rs b/crates/planrun-exec/src/runner/script.rs
index 159378c..927aec1 100644
--- a/crates/planrun-exec/src/runner/script.rs
+++ b/crates/planrun-exec/src/runner/script.rs
@@ -5,12 +5,13 @@
 use planrun_model::{Action, Script, ScriptIntent};
 
 use super::gate::Attempt;
+use super::resume::Past;
 use super::sessions::{NotRun, name};
 use super::task::TaskRun;
 use super::{Answer, Flow, Result, RunError, Runner, Typed, canonical, describe};
 use crate::expect::{self, Mismatch, Reason, Verdict};
 use crate::journal::{Outcome, PathBlob, Record};
-use crate::policy::{Decision, Skip};
+use crate::policy::{Decision, PolicyClass, Skip};
 use crate::shell::{RunStatus, ScriptRun};
 use crate::ui::{Choice, Position, Question};
 
@@ -45,6 +46,14 @@ impl Runner<'_> {
             .policy
             .decide(task, action, &plan.source, &self.classifier);
         let line = self.line(action);
+        match self.replayed(pos, action, script, commit)? {
+            Replayed::Again => {}
+            Replayed::Keep => {
+                return self.skipped(run, pos, action, script, decision, Outcome::Kept);
+            }
+            Replayed::Done => return Ok(Flow::Go),
+            Replayed::Stop(why) => return Ok(Flow::Stop(why)),
+        }
         if let Decision::Skip(Skip::ExpectedFailure) = decision {
             self.ui.status(&format!(
                 "  L{line:<5} {}: skipped: expected failure (D67)",
@@ -108,6 +117,83 @@ impl Runner<'_> {
         }
     }
 
+    /// What resume does with a script that ran, or was running, before the interruption
+    /// (D77): a made Commit is done; one that was running asks whatever its class; one that
+    /// ended runs again, Modify and Destructive ones asking first.
+    fn replayed(
+        &mut self,
+        pos: Position,
+        action: &Action,
+        script: &Script,
+        commit: bool,
+    ) -> Result<Replayed> {
+        let line = self.line(action);
+        if commit && self.commit_made(pos.task, (pos.step, pos.action)) {
+            self.ui.status(&format!(
+                "  L{line:<5} {}: made before the interruption",
+                describe(action)
+            ));
+            return Ok(Replayed::Done);
+        }
+        let Some(past) = self.take_past(pos.task, (pos.step, pos.action)) else {
+            return Ok(Replayed::Again);
+        };
+        let ended_ok = matches!(
+            past,
+            Past::Ended(Outcome::Passed | Outcome::Confirmed | Outcome::Accepted)
+        );
+        if commit && ended_ok {
+            self.ui.status(&format!(
+                "  L{line:<5} {}: made before the interruption",
+                describe(action)
+            ));
+            return Ok(Replayed::Done);
+        }
+        let why = match past {
+            Past::Ended(Outcome::Kept) => return Ok(Replayed::Keep),
+            Past::Ended(Outcome::SkippedPolicy) => return Ok(Replayed::Again),
+            Past::Running { .. } => {
+                "was running when planrun was interrupted; it may have done part of its work"
+            }
+            Past::Ended(_) => {
+                let body = self.plan.source.resolve(&script.body);
+                let class = self.classifier.explain(script.shell, body).class;
+                if class < PolicyClass::Modify {
+                    self.ui.status(&format!(
+                        "  L{line:<5} {}: ran before the interruption; it runs again",
+                        describe(action)
+                    ));
+                    return Ok(Replayed::Again);
+                }
+                "ran before the interruption"
+            }
+        };
+        let question = Question {
+            kind: "again",
+            at: Some(pos),
+            text: vec![
+                format!("L{line} {} {why}", describe(action)),
+                "retry runs it again (its decision applies); keep leaves its result as it is"
+                    .to_owned(),
+            ],
+            choices: vec![
+                Choice::Retry,
+                Choice::Keep,
+                Choice::View,
+                Choice::Diff,
+                Choice::Shell,
+                Choice::Quit,
+            ],
+            typed: false,
+        };
+        Ok(match self.ask(question)? {
+            Answer::Choice(Choice::Retry) => Replayed::Again,
+            Answer::Choice(Choice::Keep) => Replayed::Keep,
+            Answer::Stop(why) => Replayed::Stop(why),
+            _ => Replayed::Stop("quit".to_owned()),
+        })
+    }
+
     /// A skipped script, journaled as started and ended, and recorded for the gate.
     fn skipped(
         &mut self,
@@ -140,7 +226,11 @@ impl Runner<'_> {
             outcome,
             gate: script.intent == ScriptIntent::Gate,
             shell: script.shell,
-            generation: self.sessions.generation(script.shell),
+            generation: if outcome == Outcome::Kept {
+                u64::MAX
+            } else {
+                self.sessions.generation(script.shell)
+            },
         };
         run.gate.record((pos.step, pos.action), attempt);
         Ok(Flow::Go)
@@ -436,6 +526,14 @@ impl Runner<'_> {
     }
 }
 
+/// What resume does with a script.
+enum Replayed {
+    Again,
+    Keep,
+    Done,
+    Stop(String),
+}
+
 /// What the run prompt came to.
 enum Permit {
     Run,
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t7-script-rs.patch"
Remove-Item "$env:TEMP\p5-t7-script-rs.patch"
```

File `crates/planrun-exec/src/runner/task.rs` (exact diff; save the block as `$env:TEMP\p5-t7-task-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/runner/task.rs b/crates/planrun-exec/src/runner/task.rs
index 9c43d43..c9cae89 100644
--- a/crates/planrun-exec/src/runner/task.rs
+++ b/crates/planrun-exec/src/runner/task.rs
@@ -4,6 +4,7 @@
 use planrun_model::{Action, ActionKind, RepoPath, Task};
 
 use super::gate::Gate;
+use super::resume::Past;
 use super::{Answer, Flow, Result, RunError, Runner, Typed, describe, is_file_action, task_paths};
 use crate::finding::{Finding, Severity};
 use crate::journal::{Outcome, PathBlob, Record};
@@ -28,10 +29,17 @@ impl Runner<'_> {
         let task = &plan.tasks[index];
         self.ui
             .status(&format!("{}  {}", task.id, plan.text(&task.title)));
-        let found = preflight::preflight(self.ws, plan, task)?;
+        let resumed = match self.replay_steps(index)? {
+            Ok(resumed) => resumed,
+            Err(why) => return Ok(Flow::Stop(why)),
+        };
+        let found = preflight::preflight_resumed(self.ws, plan, task, &resumed)?;
         if let Flow::Stop(why) = self.findings(task, &found) {
             return Ok(Flow::Stop(why));
         }
+        if let Flow::Stop(why) = self.resume_dirt(&found)? {
+            return Ok(Flow::Stop(why));
+        }
         let head = self.head()?;
         self.append(Record::TaskStart {
             task: task.id.raw.to_string(),
@@ -98,6 +106,39 @@ impl Runner<'_> {
         Flow::Go
     }
 
+    /// Resume's dirt (D77): paths with changes that no verified step of the interrupted run
+    /// made and that a remaining file action writes over. They may be the user's, so they are
+    /// asked about once, whatever `--auto` says.
+    fn resume_dirt(&mut self, found: &[Finding]) -> Result<Flow> {
+        let paths: Vec<String> = found
+            .iter()
+            .filter(|f| f.code == "resume-dirt")
+            .filter_map(|f| f.path.as_ref())
+            .map(|p| format!("  {p}"))
+            .collect();
+        if paths.is_empty() {
+            return Ok(Flow::Go);
+        }
+        let mut text = vec![format!(
+            "{} path(s) of the task have changes that the interrupted run did not verify as its              own, and a remaining file action writes over them:",
+            paths.len()
+        )];
+        text.extend(paths);
+        text.push("continue writes over them; quit leaves them as they are".to_owned());
+        let question = Question {
+            kind: "resume-dirt",
+            at: None,
+            text,
+            choices: vec![Choice::Continue, Choice::Diff, Choice::Shell, Choice::Quit],
+            typed: false,
+        };
+        Ok(match self.ask(question)? {
+            Answer::Choice(Choice::Continue) => Flow::Go,
+            Answer::Stop(why) => Flow::Stop(why),
+            _ => Flow::Stop("quit".to_owned()),
+        })
+    }
+
     /// One action that is not a file action.
     fn action(&mut self, run: &mut TaskRun, pos: Position, action: &Action) -> Result<Flow> {
         let task = &self.plan.tasks[pos.task];
@@ -129,6 +170,17 @@ impl Runner<'_> {
     fn files(&mut self, run: &mut TaskRun, pos: Position, group: &[&Action]) -> Result<Flow> {
         let plan = self.plan;
         let task = &plan.tasks[pos.task];
+        if self.group_done(pos.task, (pos.step, pos.action)) {
+            self.append(Record::StepVerified { at: self.at(pos) })?;
+            for action in group {
+                let line = self.line(action);
+                self.ui.status(&format!(
+                    "  L{line:<5} {}: done on disk, not written again",
+                    describe(action)
+                ));
+            }
+            return Ok(Flow::Go);
+        }
         let mut chosen: Vec<&Action> = Vec::new();
         for (k, action) in group.iter().enumerate() {
             let here = Position {
@@ -257,6 +309,14 @@ impl Runner<'_> {
         let ActionKind::Manual { reason, text } = &action.kind else {
             return Ok(Flow::Go);
         };
+        let past = self.take_past(pos.task, (pos.step, pos.action));
+        if past == Some(Past::Ended(Outcome::Acknowledged)) {
+            self.ui.status(&format!(
+                "  L{} manual: acknowledged before the interruption",
+                self.line(action)
+            ));
+            return Ok(Flow::Go);
+        }
         let head = self.head()?;
         self.append(Record::ActionStart {
             at: self.at(pos),
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t7-task-rs.patch"
Remove-Item "$env:TEMP\p5-t7-task-rs.patch"
```

File `crates/planrun-exec/tests/resume.rs` (full contents):

```rust
//! Resume on hand-built IR over a fixture copy (D14, D77): the journal checked against the
//! disk and HEAD. A crash is a panic from the journal's observer at a seam, so the journal and
//! the disk are what a killed process leaves.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[path = "support/ir.rs"]
mod ir;

use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

use fixture::Fixture;
use ir::{Ir, PATCH_T1, PATCH_T2};
use planrun_exec::journal::{self, End, Journal, Observer, Record, Seam};
use planrun_exec::policy::{Policy, PolicyClass};
use planrun_exec::runner::resume::analyse;
use planrun_exec::runner::{Finished, RunConfig, Runner};
use planrun_exec::shell::{OutLine, SessionConfig};
use planrun_exec::ui::{Choice, Position, Question, Reply, Ui};
use planrun_exec::workspace::Workspace;
use planrun_model::{
    Action, ActionKind, ChangeKind, ExitExpect, Expectation, PathOrigin, Plan, ScriptIntent,
    WriteMode,
};

#[derive(Default)]
struct Scripted {
    answers: VecDeque<Reply>,
    asked: Vec<String>,
    shown: Vec<String>,
}

impl Ui for Scripted {
    fn status(&mut self, line: &str) {
        self.shown.push(line.to_owned());
    }
    fn output(&mut self, line: &OutLine) {
        self.shown.push(format!("| {}", line.text));
    }
    fn ask(&mut self, question: &Question) -> Reply {
        self.asked.push(question.kind.to_owned());
        self.shown.extend(question.text.iter().cloned());
        self.answers
            .pop_front()
            .unwrap_or_else(|| Reply::Stop(format!("no answer for `{}`", question.kind)))
    }
    fn view(&mut self, _at: Position) {}
    fn diff(&mut self) {}
    fn shell(&mut self) {}
}

fn choice(c: Choice) -> Reply {
    Reply::Choice(c)
}

const AUTO: &[PolicyClass] = &[PolicyClass::Read, PolicyClass::Build, PolicyClass::Modify];
const W: PathOrigin = PathOrigin::Written;

fn config(f: &Fixture, auto: &[PolicyClass]) -> RunConfig {
    RunConfig {
        plan_path: "test.md".to_owned(),
        policy: Policy::new(auto, false).unwrap_or_else(|e| panic!("{e}")),
        from: None,
        to: None,
        session: SessionConfig {
            root: f.path().to_path_buf(),
            env: f.env(),
            timeout: Some(Duration::from_secs(60)),
            temp: f.outside(),
        },
    }
}

/// A run (or, with `resume`, a resume) of `plan`; `observer` may crash it. `None` when it
/// crashed.
fn go(
    f: &Fixture,
    plan: &Plan,
    auto: &[PolicyClass],
    answers: Vec<Reply>,
    observer: Option<Observer>,
    resume: bool,
) -> (Option<Finished>, Scripted) {
    let ws = Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"));
    let path = f.outside().join("journal.jsonl");
    let lines = journal::read(&path).unwrap_or_else(|e| panic!("{e}"));
    let mut journal = Journal::open(&path).unwrap_or_else(|e| panic!("{e}"));
    if let Some(observer) = observer {
        journal.observe(observer);
    }
    let mut ui = Scripted {
        answers: answers.into(),
        ..Scripted::default()
    };
    let finished = catch_unwind(AssertUnwindSafe(|| {
        let mut runner = Runner::new(plan, &ws, &mut journal, &mut ui, config(f, auto));
        if resume {
            let analysis = analyse(&lines).unwrap_or_else(|| panic!("nothing to resume"));
            runner.resume(&analysis)
        } else {
            runner.run()
        }
    }));
    let finished = finished.ok().map(|r| r.unwrap_or_else(|e| panic!("{e}")));
    (finished, ui)
}

fn records(f: &Fixture) -> Vec<Record> {
    journal::read(&f.outside().join("journal.jsonl"))
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|l| l.record)
        .collect()
}

fn count(records: &[Record], name: &str) -> usize {
    records
        .iter()
        .filter(|r| {
            let json = serde_json::to_value(r).unwrap_or_else(|e| panic!("{e}"));
            json["t"] == name
        })
        .count()
}

fn subjects(f: &Fixture) -> Vec<String> {
    f.git(&["log", "--format=%s"])
        .lines()
        .map(str::to_owned)
        .collect()
}

fn gate(ir: &mut Ir, body: &str) -> Action {
    let mut action = ir.script(body);
    if let ActionKind::RunScript(s) = &mut action.kind {
        s.intent = ScriptIntent::Gate;
        s.expect = Some(Expectation {
            exit: ExitExpect::Success,
            lines: None,
            prose: None,
            comments: None,
        });
    }
    action
}

/// The two assay tasks by hand; T1's step 1 writes two files.
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
        vec![gate(ir, "$gate = 0")],
        vec![ir.commit(
            "if ($gate -eq 0) { git add -- src/greet.rs src/lib.rs; git commit -q -m 'P1-T1' }",
        )],
    ];
    let m2 = ir.manifest(&[
        ("src/lib.rs", ChangeKind::Modify, W),
        ("src/old.rs", ChangeKind::Delete, W),
    ]);
    let t2 = vec![
        vec![
            ir.patch("src/lib.rs", PATCH_T2, "97bd92a", "6b269fc"),
            ir.delete("src/old.rs", false),
        ],
        vec![gate(ir, "$gate = 0")],
        vec![ir.commit(
            "if ($gate -eq 0) { git add -- src/lib.rs src/old.rs; git commit -q -m 'P1-T2' }",
        )],
    ];
    let tasks = vec![ir.task("P1-T1", m1, t1), ir.task("P1-T2", m2, t2)];
    ir.plan(tasks)
}

/// An observer that panics at the first seam `crash` accepts.
fn crash_at(crash: impl Fn(&Seam<'_>) -> bool + 'static) -> Option<Observer> {
    Some(Box::new(move |seam| {
        if crash(&seam) {
            panic!("crash at {seam:?}");
        }
    }))
}

fn assert_done(f: &Fixture) {
    assert_eq!(subjects(f), ["P1-T2", "P1-T1", "fixture"]);
    assert_eq!(f.status(), "");
    let ls = f.git(&["ls-tree", "-r", "--name-only", "HEAD"]);
    assert_eq!(
        ls.lines().collect::<Vec<_>>(),
        [".gitignore", "Cargo.toml", "src/greet.rs", "src/lib.rs"]
    );
}

#[test]
fn a_run_stopped_at_the_commit_prompt_resumes_without_writing_again() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let read_build = &[PolicyClass::Read, PolicyClass::Build];
    let (first, ui) = go(
        &f,
        &plan,
        read_build,
        vec![choice(Choice::Quit)],
        None,
        false,
    );
    assert_eq!(first.map(|f| f.end), Some(End::Stopped));
    assert_eq!(ui.asked, ["run"]);
    let (second, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Continue)], None, true);
    assert_eq!(
        second.map(|f| f.end),
        Some(End::Completed),
        "{:?}",
        ui.shown
    );
    assert_eq!(ui.asked, ["session"]);
    assert!(
        ui.shown
            .iter()
            .any(|l| l.contains("done on disk, not written again"))
    );
    let all = records(&f);
    assert_eq!(
        count(&all, "step-prepared"),
        2,
        "T1 step 1 once, T2 step 1 once"
    );
    assert_eq!(count(&all, "step-written"), 2);
    assert_eq!(count(&all, "step-verified"), 1);
    assert_done(&f);
}

#[test]
fn a_crash_after_the_write_ahead_record_prepares_the_step_again_and_writes_it_once() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let prepared = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::StepPrepared { .. }));
    let (first, _) = go(&f, &plan, AUTO, vec![], crash_at(prepared), false);
    assert!(first.is_none(), "it crashed");
    assert!(f.read("src/greet.rs").is_none(), "nothing was written");
    let (second, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Continue)], None, true);
    assert_eq!(
        second.map(|f| f.end),
        Some(End::Completed),
        "{:?}",
        ui.shown
    );
    let all = records(&f);
    assert_eq!(count(&all, "step-prepared"), 3);
    assert_eq!(count(&all, "step-written"), 2);
    assert_done(&f);
}

#[test]
fn a_crash_between_two_writes_is_unknown_and_restore_completes_it() {
    let f = Fixture::greeter(true);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let first_write = |s: &Seam<'_>| matches!(s, Seam::Wrote(1));
    let (first, _) = go(&f, &plan, AUTO, vec![], crash_at(first_write), false);
    assert!(first.is_none());
    assert!(f.read("src/greet.rs").is_some(), "the first write happened");
    let answers = vec![choice(Choice::Restore), choice(Choice::Continue)];
    let (second, ui) = go(&f, &plan, AUTO, answers, None, true);
    assert_eq!(
        second.map(|f| f.end),
        Some(End::Completed),
        "{:?}",
        ui.shown
    );
    assert_eq!(ui.asked, ["unknown", "session"]);
    let all = records(&f);
    assert_eq!(count(&all, "step-restored"), 1);
    assert_done(&f);
}

#[test]
fn a_crash_after_the_commit_s_start_runs_the_commit() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let commit_start = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::ActionStart { kind, at, .. }) if kind == "commit" && at.task == "P1-T1");
    let (first, _) = go(&f, &plan, AUTO, vec![], crash_at(commit_start), false);
    assert!(first.is_none());
    assert_eq!(subjects(&f), ["fixture"]);
    let (second, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Continue)], None, true);
    assert_eq!(
        second.map(|f| f.end),
        Some(End::Completed),
        "{:?}",
        ui.shown
    );
    assert_done(&f);
}

#[test]
fn a_commit_made_before_its_end_was_journaled_is_done() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let root = f.path().to_path_buf();
    let env = f.env();
    let observer: Observer = Box::new(move |seam| {
        let Seam::Appended(Record::ActionStart { kind, at, .. }) = seam else {
            return;
        };
        if kind != "commit" || at.task != "P1-T1" {
            return;
        }
        for args in [
            &["add", "--", "src/greet.rs", "src/lib.rs"][..],
            &["commit", "-q", "-m", "P1-T1"],
        ] {
            let ok = std::process::Command::new("git")
                .args(args)
                .current_dir(&root)
                .envs(env.iter().map(|(k, v)| (k, v)))
                .status()
                .is_ok_and(|s| s.success());
            assert!(ok);
        }
        panic!("crash inside git commit");
    });
    let (first, _) = go(&f, &plan, AUTO, vec![], Some(observer), false);
    assert!(first.is_none());
    assert_eq!(subjects(&f), ["P1-T1", "fixture"]);
    let (second, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Continue)], None, true);
    assert_eq!(
        second.map(|f| f.end),
        Some(End::Completed),
        "{:?}",
        ui.shown
    );
    assert!(
        ui.shown
            .iter()
            .any(|l| l.contains("was made before the interruption"))
    );
    assert_done(&f);
}

#[test]
fn a_commit_found_made_by_resume_is_not_run_again_after_a_second_crash() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let root = f.path().to_path_buf();
    let env = f.env();
    let observer: Observer = Box::new(move |seam| {
        let Seam::Appended(Record::ActionStart { kind, at, .. }) = seam else {
            return;
        };
        if kind != "commit" || at.task != "P1-T1" {
            return;
        }
        git_in(&root, &env, &["add", "--", "src/greet.rs", "src/lib.rs"]);
        git_in(&root, &env, &["commit", "-q", "-m", "P1-T1"]);
        panic!("crash inside git commit");
    });
    let (first, _) = go(&f, &plan, AUTO, vec![], Some(observer), false);
    assert!(first.is_none());
    let verified = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::StepVerified { .. }));
    let (second, _) = go(&f, &plan, AUTO, vec![], crash_at(verified), true);
    assert!(second.is_none(), "it crashed after finding the commit made");
    let (third, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Continue)], None, true);
    assert_eq!(third.map(|f| f.end), Some(End::Completed), "{:?}", ui.shown);
    assert_eq!(ui.asked, ["session"]);
    assert_done(&f);
}

#[test]
fn resume_stops_on_another_head_a_changed_plan_or_an_index_lock() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let read_build = &[PolicyClass::Read, PolicyClass::Build];
    go(
        &f,
        &plan,
        read_build,
        vec![choice(Choice::Quit)],
        None,
        false,
    );
    std::fs::write(f.path().join(".git/index.lock"), "").unwrap_or_else(|e| panic!("{e}"));
    let (r, _) = go(&f, &plan, AUTO, vec![], None, true);
    let reason = r.and_then(|r| r.reason).unwrap_or_default();
    assert!(reason.contains("index.lock"), "{reason}");
    std::fs::remove_file(f.path().join(".git/index.lock")).unwrap_or_else(|e| panic!("{e}"));
    let mut changed = plan.clone();
    changed.source = planrun_model::PlanSource::from_normalized(
        "test.md",
        plan.source.text().to_owned(),
        "1".to_owned(),
    );
    let (r, _) = go(&f, &changed, AUTO, vec![], None, true);
    let reason = r.and_then(|r| r.reason).unwrap_or_default();
    assert!(reason.contains("fingerprint"), "{reason}");
    f.git(&["commit", "-q", "--allow-empty", "-m", "someone else"]);
    let (r, _) = go(&f, &plan, AUTO, vec![], None, true);
    let reason = r.and_then(|r| r.reason).unwrap_or_default();
    assert!(reason.contains("something else changed"), "{reason}");
    let (again, _) = go(&f, &plan, AUTO, vec![], None, true);
    let again = again.and_then(|r| r.reason).unwrap_or_default();
    assert_eq!(
        again, reason,
        "a stopped resume does not record the moved HEAD"
    );
}

/// A git command run in the fixture from an observer.
fn git_in(root: &std::path::Path, env: &[(std::ffi::OsString, std::ffi::OsString)], args: &[&str]) {
    let ok = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .envs(env.iter().map(|(k, v)| (k, v)))
        .status()
        .is_ok_and(|s| s.success());
    assert!(ok, "git {args:?}");
}

fn resume_runs(records: &[Record]) -> Vec<usize> {
    records
        .iter()
        .enumerate()
        .filter(|(_, r)| matches!(r, Record::Run { resume: true, .. }))
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn a_second_resume_after_a_foreign_commit_asks_again_and_never_commits() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let commit_start = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::ActionStart { kind, at, .. }) if kind == "commit" && at.task == "P1-T1");
    let (first, _) = go(&f, &plan, AUTO, vec![], crash_at(commit_start), false);
    assert!(first.is_none());
    f.git(&["commit", "-q", "--allow-empty", "-m", "someone else"]);
    let mut reasons = Vec::new();
    for _ in 0..2 {
        let (r, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Quit)], None, true);
        assert_eq!(ui.asked, ["head-moved"], "{:?}", ui.shown);
        reasons.push(r.and_then(|r| r.reason).unwrap_or_default());
    }
    assert_eq!(reasons[0], reasons[1]);
    let all = records(&f);
    let runs = resume_runs(&all);
    assert_eq!(runs.len(), 2);
    let after = &all[runs[0]..];
    assert_eq!(
        count(after, "action-start"),
        0,
        "nothing ran after the crash"
    );
    assert_eq!(subjects(&f), ["someone else", "fixture"]);
}

#[test]
fn a_kept_head_move_survives_a_crash_right_after_the_answer() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let root = f.path().to_path_buf();
    let env = f.env();
    let observer: Observer = Box::new(move |seam| {
        let Seam::Appended(Record::ActionStart { kind, at, .. }) = seam else {
            return;
        };
        if kind != "script" || at.task != "P1-T1" {
            return;
        }
        git_in(
            &root,
            &env,
            &["commit", "-q", "--allow-empty", "-m", "inside"],
        );
        panic!("crash inside the script that committed");
    });
    let (first, _) = go(&f, &plan, AUTO, vec![], Some(observer), false);
    assert!(first.is_none());
    // The first record after the keep's own: the replayed step's `step-verified`, before the
    // kept script's records.
    let kept = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::StepVerified { .. }));
    let (second, ui) = go(
        &f,
        &plan,
        AUTO,
        vec![choice(Choice::Keep)],
        crash_at(kept),
        true,
    );
    assert!(second.is_none(), "it crashed right after the keep");
    assert_eq!(ui.asked, ["head-moved"]);
    let (third, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Continue)], None, true);
    assert_eq!(third.map(|f| f.end), Some(End::Completed), "{:?}", ui.shown);
    assert_eq!(ui.asked, ["session"], "neither head-moved nor again");
    assert_eq!(subjects(&f), ["P1-T2", "P1-T1", "inside", "fixture"]);
    assert_eq!(f.status(), "");
}

#[test]
fn a_change_after_the_crash_to_a_path_a_remaining_step_writes_is_asked_about() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let m = ir.manifest(&[
        ("src/greet.rs", ChangeKind::Create, W),
        ("src/lib.rs", ChangeKind::Modify, W),
    ]);
    let steps = vec![
        vec![ir.write(
            "src/greet.rs",
            "pub fn greeting() {}
",
            WriteMode::Create,
        )],
        vec![ir.write(
            "src/lib.rs",
            "pub mod greet;
",
            WriteMode::Overwrite,
        )],
        vec![gate(&mut ir, "$gate = 0")],
        vec![ir.commit(
            "if ($gate -eq 0) { git add -- src/greet.rs src/lib.rs; git commit -q -m 'T1' }",
        )],
    ];
    let task = ir.task("T1", m, steps);
    let plan = ir.plan(vec![task]);
    let written = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::StepWritten { .. }));
    let (first, _) = go(&f, &plan, AUTO, vec![], crash_at(written), false);
    assert!(first.is_none());
    std::fs::write(
        f.path().join("src/lib.rs"),
        "// mine
",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let (second, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Quit)], None, true);
    assert_eq!(second.map(|f| f.end), Some(End::Stopped));
    assert_eq!(ui.asked, ["resume-dirt"], "asked although --auto covers it");
    assert!(
        ui.shown.iter().any(|l| l == "  src/lib.rs"),
        "{:?}",
        ui.shown
    );
    assert!(
        !ui.shown.iter().any(|l| l == "  src/greet.rs"),
        "a verified step's path is not asked about"
    );
    assert_eq!(
        f.read("src/lib.rs").as_deref(),
        Some(
            &b"// mine
"[..]
        )
    );
    let answers = vec![choice(Choice::Continue), choice(Choice::Continue)];
    let (third, ui) = go(&f, &plan, AUTO, answers, None, true);
    assert_eq!(third.map(|f| f.end), Some(End::Completed), "{:?}", ui.shown);
    assert_eq!(ui.asked, ["resume-dirt", "session"]);
    assert_eq!(
        f.read("src/lib.rs").as_deref(),
        Some(
            &b"pub mod greet;
"[..]
        )
    );
    assert_eq!(subjects(&f), ["T1", "fixture"]);
}

#[test]
fn a_modify_script_that_ran_before_asks_and_keep_leaves_it() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let m = ir.manifest(&[("a.txt", ChangeKind::Create, W)]);
    let steps = vec![
        vec![ir.script("Add-Content -Path a.txt -Value line")],
        vec![ir.script("git status --short")],
        vec![ir.manual("look at it by hand")],
    ];
    let task = ir.task("T1", m, steps);
    let plan = ir.plan(vec![task]);
    let (first, _) = go(&f, &plan, AUTO, vec![choice(Choice::Quit)], None, false);
    assert_eq!(first.map(|f| f.end), Some(End::Stopped));
    let answers = vec![
        choice(Choice::Keep),
        choice(Choice::Continue),
        choice(Choice::Continue),
    ];
    let (second, ui) = go(&f, &plan, AUTO, answers, None, true);
    assert_eq!(
        second.map(|f| f.end),
        Some(End::Completed),
        "{:?}",
        ui.shown
    );
    assert_eq!(ui.asked, ["again", "session", "manual"]);
    assert!(
        ui.shown
            .iter()
            .any(|l| l.contains("ran before the interruption; it runs again"))
    );
    assert_eq!(
        f.read("a.txt")
            .map(|b| String::from_utf8_lossy(&b).lines().count()),
        Some(1)
    );
}
```

File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p5-t7-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index 1fe6fb7..52426c1 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -993,8 +993,8 @@ Amends D15; D9 drops `dialoguer`.
   not offer asks again. Every answer is journaled.
 - **Questions of their own**, asked whatever `--auto` says: the session notice (D81), a working
   directory outside the repository (D81), dirt before a commit (D80), a closed commit gate
-  (D75), a step changed between prepare and commit (D78), and resume's Unknown step and
-  interrupted script (D77). Each answer is journaled with the question's name.
+  (D75), a step changed between prepare and commit (D78), and resume's Unknown step,
+  interrupted script and dirt a remaining file action writes over (D77). Each answer is journaled with the question's name.
 
 ## D75 The commit gate
 
@@ -1061,7 +1061,15 @@ Amends D14 and D57.
   (HEAD is recorded after every script, Manual, `[o]` and Commit, D80). The one exception is
   the action that was running: when HEAD moved by one commit on the recorded one, a Commit
   whose new tree holds what it was to hold is done (D79); for any other action the user
-  decides (keep it as done, or run it again).
+  decides (keep it as done, or run it again; the question offers no `[o]`, whose `head-seen`
+  would pass the check unanswered).
+- **The recorded HEAD** is set by the chain's first `run`, `task-start`, `action-end`,
+  `head-seen` and `task-end`, never by a resume's own `run` record: that one is written before
+  the checks, so a resume that stops leaves the next one the same check. Where resume accepts a
+  moved HEAD it journals that at once: an `action-end` for the running action (`passed` for a
+  Commit found made, `kept` for the user's keep), or `head-seen` for a retry. A crash after it
+  resumes from there; `head-seen` alone would leave a made Commit "running" at an equal HEAD,
+  and run it again.
 - **The interrupted task runs again from its start**, in a new session, under the journaled
   policy and range unless the flags say otherwise. Before its first script a notice is shown
   that must be acknowledged even with `--auto` (D81).
@@ -1087,7 +1095,11 @@ Amends D14 and D57.
 - **Preflight of the resumed task** leaves out the file steps found done, and every path of the
   task may have been changed by the interrupted run (by a step or by a script): dirt in those
   paths is a warning, and their manifest checks are left out. The remaining file actions are
-  simulated exactly.
+  simulated exactly. Dirt in a path that no step found done or accepted wrote, and that a
+  remaining file action writes over, may be the user's (an edit between the crash and the
+  resume): it is `resume-dirt`, listed and asked about once before the task's first action,
+  whatever `--auto` says. A path that only a script writes is not asked about; a script that
+  ran before the interruption asks on its own (above).
 - **One planrun per repository:** `<git-dir>/planrun/lock`, held with `File::try_lock` (stable
   since Rust 1.89; the workspace sets no `rust-version`) for the whole of `run` and `resume`,
   prompts included, for every plan of the repository. The operating system releases it when the
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t7-decisions-md.patch"
Remove-Item "$env:TEMP\p5-t7-decisions-md.patch"
```

**Step 2 — verify**

Targeted: the resume tests, with both shells required:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-exec --test resume
```

Expected (exit code 0; summary lines):

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
  git add -- crates/planrun-exec/src/preflight.rs crates/planrun-exec/src/runner/mod.rs crates/planrun-exec/src/runner/resume.rs crates/planrun-exec/src/runner/script.rs crates/planrun-exec/src/runner/task.rs crates/planrun-exec/tests/resume.rs docs/decisions.md
  $verified = [ordered]@{
      'crates/planrun-exec/src/preflight.rs' = '1701b7bd336fedd7f52c7d78d85d16097a1f6e48'
      'crates/planrun-exec/src/runner/mod.rs' = 'e292f974e07efb0abae1a72b45ad0347f53cd3da'
      'crates/planrun-exec/src/runner/resume.rs' = 'd3f468b32d8841d18014baa01a07d7ce187655d6'
      'crates/planrun-exec/src/runner/script.rs' = '927aec1d578905c35651e6d49f2eac508b9c7723'
      'crates/planrun-exec/src/runner/task.rs' = 'c9cae89975eea3755343b974863653b61f1ba349'
      'crates/planrun-exec/tests/resume.rs' = '021a60e1064fe79920f478813eefba0160e2fd7b'
      'docs/decisions.md' = '52426c164e05341da76fe436f281b40c71b40c3d'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P5-T7: resume: the journal's last chain against the disk and HEAD, the interrupted task run again" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P5-T8: run, resume and status with line prompts; the e2e acceptance behind a test feature

**Goal:** The CLI (D74, D76, D77, D82): `run` (with `--auto`, `--run-expected-failures`, `--from`, `--to`, `--restart`), `resume` and `status` in `run.rs`; the line prompts in `ui.rs`; the `e2e` feature with its fault seam in `e2e.rs` and the end-to-end acceptance in `tests/run.rs`. `dialoguer` leaves the manifests. The synthetic plan writes `docs/modules.txt` with LF only and the greeter ignores `Cargo.lock` (Q9); the dry run says `the task's files: N error(s), M warning(s)` (item 23). Four snapshots change (stated decision 12).

**Files**

- Modify `Cargo.lock`
- Modify `Cargo.toml`
- Modify `crates/planrun-cli/Cargo.toml`
- Modify `crates/planrun-cli/src/dry_run.rs`
- Create `crates/planrun-cli/src/e2e.rs`
- Modify `crates/planrun-cli/src/main.rs`
- Create `crates/planrun-cli/src/run.rs`
- Modify `crates/planrun-cli/src/snapshots/planrun__view__tests__assay_style_ir.snap`
- Modify `crates/planrun-cli/src/snapshots/planrun__view__tests__assay_style_show.snap`
- Create `crates/planrun-cli/src/ui.rs`
- Modify `crates/planrun-cli/tests/dry_run.rs`
- Create `crates/planrun-cli/tests/run.rs`
- Modify `crates/planrun-cli/tests/snapshots/cli__show_p1_t2_step_2.snap`
- Modify `crates/planrun-cli/tests/snapshots/dry_run__dry_run_create_exists.snap`
- Modify `tests/corpus/synthetic/assay-style.md`
- Modify `tests/fixtures/greeter/.gitignore`

Over 300 lines, so they come as exact diffs: `Cargo.lock`, `crates/planrun-cli/src/main.rs`.

**Step 1 — write the files**

File `Cargo.lock` (exact diff; save the block as `$env:TEMP\p5-t8-cargo-lock.patch`, outside the repository, then apply it):

```diff
diff --git a/Cargo.lock b/Cargo.lock
index cc9a88f..e37ae23 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -157,7 +157,6 @@ checksum = "e96a4956774c13c126a8b5af4daa79384f4d826534c95a02d76afb39e2ab64e3"
 dependencies = [
  "encode_unicode",
  "libc",
- "unicode-width",
  "windows-sys",
 ]
 
@@ -185,16 +184,6 @@ dependencies = [
  "hybrid-array",
 ]
 
-[[package]]
-name = "dialoguer"
-version = "0.12.0"
-source = "registry+https://github.com/rust-lang/crates.io-index"
-checksum = "25f104b501bf2364e78d0d3974cbc774f738f5865306ed128e1e0d7499c0ad96"
-dependencies = [
- "console",
- "shell-words",
-]
-
 [[package]]
 name = "digest"
 version = "0.11.3"
@@ -337,7 +326,6 @@ version = "0.1.0"
 dependencies = [
  "annotate-snippets",
  "clap",
- "dialoguer",
  "insta",
  "planrun-exec",
  "planrun-model",
@@ -525,12 +513,6 @@ dependencies = [
  "digest",
 ]
 
-[[package]]
-name = "shell-words"
-version = "1.1.1"
-source = "registry+https://github.com/rust-lang/crates.io-index"
-checksum = "dc6fe69c597f9c37bfeeeeeb33da3530379845f10be461a66d16d03eca2ded77"
-
 [[package]]
 name = "similar"
 version = "2.7.0"
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t8-cargo-lock.patch"
Remove-Item "$env:TEMP\p5-t8-cargo-lock.patch"
```

File `Cargo.toml` (full contents):

```toml
[workspace]
resolver = "3"
members = ["crates/*"]
# The fixture repos are inputs of the tests, not workspace members.
exclude = ["tests/fixtures"]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
publish = false

[workspace.dependencies]
planrun-model = { path = "crates/planrun-model" }
planrun-parse = { path = "crates/planrun-parse" }
planrun-exec = { path = "crates/planrun-exec" }

# Each dependency is justified in docs/decisions.md (D9). Add none without a line there.
annotate-snippets = "0.12"
clap = { version = "4.6", features = ["derive"] }
insta = { version = "1.49", features = ["yaml"] }
pulldown-cmark = { version = "0.13", default-features = false }
regex = "1.13"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha1_smol = "1.0"
sha2 = "0.11"
similar = "3.2"
thiserror = "2.0"
toml = "1.2"

[workspace.lints.rust]
unsafe_code = "deny"
missing_docs = "warn"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
unwrap_used = "warn"
expect_used = "warn"
undocumented_unsafe_blocks = "deny"
```

File `crates/planrun-cli/Cargo.toml` (full contents):

```toml
[package]
name = "planrun-cli"
description = "The planrun binary: check, show, run, resume, status"
version.workspace = true
edition.workspace = true
license.workspace = true
publish.workspace = true

[[bin]]
name = "planrun"
path = "src/main.rs"

[dependencies]
annotate-snippets.workspace = true
clap.workspace = true
planrun-exec.workspace = true
planrun-model.workspace = true
planrun-parse.workspace = true
serde.workspace = true

[features]
# Test-only (D82): answers from a pipe, and the PLANRUN_E2E_ABORT fault seam. No release build
# compiles it.
e2e = []

[[test]]
name = "run"
required-features = ["e2e"]

[dev-dependencies]
insta.workspace = true

[lints]
workspace = true
```

File `crates/planrun-cli/src/dry_run.rs` (full contents):

```rust
//! `planrun run --dry-run` (docs/architecture.md §7, decision D55): one line per action with its
//! outcome, a summary, then every finding with its source context. A script or commit shows its
//! class and what the default policy would do with it (D68, D69).

use planrun_exec::finding::{Finding, Severity as ExecSeverity};
use planrun_exec::policy::{Decision, DefaultClassifier, Policy, Skip};
use planrun_exec::preflight::{DryRun, Outcome};
use planrun_model::{ActionKind, Plan, Task};
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
            let errors = task.manifest.iter().filter(|f| f.is_error()).count();
            let warnings = task.manifest.len() - errors;
            out.push_str(&format!(
                "  the task's files: {errors} error(s), {warnings} warning(s)\n"
            ));
        }
        for a in &task.actions {
            if let Some(line) = action_line(plan, t, a, &mut counts) {
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
    task: &Task,
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
            let decision =
                Policy::default().decide(task, &a.action, &plan.source, &DefaultClassifier);
            format!("not run; {}", decided(decision))
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

/// What the default policy (no `--auto`) would do with a script or commit.
fn decided(decision: Decision) -> String {
    match decision {
        Decision::Ask(class) => format!("would ask ({class:?})"),
        Decision::TypedYes => "typed yes (Destructive)".to_owned(),
        Decision::Skip(Skip::ExpectedFailure) => "skipped: expected failure".to_owned(),
        other => format!("{other:?}"),
    }
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

File `crates/planrun-cli/src/e2e.rs` (full contents):

```rust
//! The fault seam of planrun-cli's end-to-end tests (decision D82), compiled only with the
//! `e2e` feature, never into a release binary. `PLANRUN_E2E_ABORT=<seam>@<task>` aborts the
//! process right after the seam, once the journal record before it is with the OS (and synced,
//! when durable):
//!
//! - `step-prepared@P1-T2`: after a step's write-ahead record, before its first write;
//! - `write<k>@P1-T1`: after the k-th path a step's commit wrote;
//! - `commit-start@P1-T1`: after a Commit's `action-start`, before its block runs.
//!
//! It ends the process with `exit(134)`: no destructor runs (the journal, the sessions and the
//! lock are left as a kill leaves them; the job objects die with the process).

use planrun_exec::journal::{Observer, Record, Seam};

/// The observer `PLANRUN_E2E_ABORT` asks for, if any.
pub fn observer() -> Option<Observer> {
    let spec = std::env::var("PLANRUN_E2E_ABORT").ok()?;
    let (seam, task) = spec.split_once('@')?;
    let (seam, task) = (seam.to_owned(), task.to_owned());
    let mut current = String::new();
    Some(Box::new(move |s| {
        let hit = match s {
            Seam::Appended(Record::TaskStart { task, .. }) => {
                current.clone_from(task);
                false
            }
            Seam::Appended(Record::StepPrepared { at, .. }) => {
                seam == "step-prepared" && at.task == task
            }
            Seam::Appended(Record::ActionStart { at, kind, .. }) => {
                seam == "commit-start" && kind == "commit" && at.task == task
            }
            Seam::Wrote(n) => seam == format!("write{n}") && current == task,
            Seam::Appended(_) => false,
        };
        if hit {
            eprintln!("planrun: e2e abort at {spec}");
            // `exit` rather than `abort`: no destructor runs, as in a kill, and on Windows it
            // raises no Windows Error Reporting for the test machine to answer.
            std::process::exit(134);
        }
    }))
}
```

File `crates/planrun-cli/src/main.rs` (exact diff; save the block as `$env:TEMP\p5-t8-main-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/main.rs b/crates/planrun-cli/src/main.rs
index a8b84cb..03112e9 100644
--- a/crates/planrun-cli/src/main.rs
+++ b/crates/planrun-cli/src/main.rs
@@ -3,7 +3,11 @@
 #![forbid(unsafe_code)]
 
 mod dry_run;
+#[cfg(feature = "e2e")]
+mod e2e;
 mod render;
+mod run;
+mod ui;
 mod view;
 
 use std::path::{Path, PathBuf};
@@ -44,8 +48,7 @@ enum Command {
         #[arg(long)]
         format: Option<String>,
     },
-    /// Execute a plan in the git repository of the current directory. Only `--dry-run` exists
-    /// so far (Phase 3); the run itself is Phase 5.
+    /// Execute a plan in the git repository of the current directory, task by task.
     Run {
         /// The plan file.
         plan: PathBuf,
@@ -55,10 +58,53 @@ enum Command {
         /// Force a format id.
         #[arg(long)]
         format: Option<String>,
+        /// Script classes that run without asking: read, build, modify (D69).
+        #[arg(long)]
+        auto: Option<String>,
+        /// Run the verifications that expect to fail (D67).
+        #[arg(long)]
+        run_expected_failures: bool,
+        /// The first task to run.
+        #[arg(long)]
+        from: Option<String>,
+        /// The last task to run.
+        #[arg(long)]
+        to: Option<String>,
+        /// Give up the plan's unfinished run; its journal is kept under another name.
+        #[arg(long)]
+        restart: bool,
+    },
+    /// Go on with the plan's interrupted or stopped run (D14).
+    Resume {
+        /// The plan file.
+        plan: PathBuf,
+        /// Force a format id.
+        #[arg(long)]
+        format: Option<String>,
+        /// Script classes that run without asking; the journaled run's when not given.
+        #[arg(long)]
+        auto: Option<String>,
+        /// Run the verifications that expect to fail.
+        #[arg(long)]
+        run_expected_failures: bool,
+        /// The first task (the journaled run's when not given).
+        #[arg(long)]
+        from: Option<String>,
+        /// The last task (the journaled run's when not given).
+        #[arg(long)]
+        to: Option<String>,
+    },
+    /// What the plan's journal says: the tasks done, the one interrupted. Changes nothing.
+    Status {
+        /// The plan file.
+        plan: PathBuf,
+        /// Force a format id.
+        #[arg(long)]
+        format: Option<String>,
     },
 }
 
-/// Exit codes: 0 fine, 1 the plan has errors, 2 a usage or I/O problem.
+/// Exit codes: 0 fine, 1 the plan has errors or the run stopped, 2 a usage or I/O problem.
 const OK: u8 = 0;
 const PLAN_ERRORS: u8 = 1;
 const USAGE: u8 = 2;
@@ -77,12 +123,52 @@ fn main() -> ExitCode {
             plan,
             dry_run: true,
             format,
+            ..
         } => run_dry(plan, format.as_deref(), &mut out),
-        Command::Run { .. } => {
-            out.push_str(
-                "planrun: run without --dry-run is not implemented yet (docs/roadmap.md, P5)\n",
-            );
-            USAGE
+        Command::Run {
+            plan,
+            format,
+            auto,
+            run_expected_failures,
+            from,
+            to,
+            restart,
+            ..
+        } => {
+            let opts = run::Options {
+                auto: auto.clone(),
+                run_expected_failures: *run_expected_failures,
+                from: from.clone(),
+                to: to.clone(),
+                restart: *restart,
+            };
+            with_plan(plan, format.as_deref(), &mut out, |p, out| {
+                run::run(plan, p, &opts, out)
+            })
+        }
+        Command::Resume {
+            plan,
+            format,
+            auto,
+            run_expected_failures,
+            from,
+            to,
+        } => {
+            let opts = run::Options {
+                auto: auto.clone(),
+                run_expected_failures: *run_expected_failures,
+                from: from.clone(),
+                to: to.clone(),
+                restart: false,
+            };
+            with_plan(plan, format.as_deref(), &mut out, |p, out| {
+                run::resume(plan, p, &opts, out)
+            })
+        }
+        Command::Status { plan, format } => {
+            with_plan(plan, format.as_deref(), &mut out, |p, out| {
+                run::status(plan, p, out)
+            })
         }
     };
     if code == OK {
@@ -93,6 +179,19 @@ fn main() -> ExitCode {
     ExitCode::from(code)
 }
 
+/// Loads the plan and hands it to `f`; a plan that does not parse runs nothing (D3).
+fn with_plan(
+    path: &Path,
+    format: Option<&str>,
+    out: &mut String,
+    f: impl FnOnce(&planrun_model::Plan, &mut String) -> u8,
+) -> u8 {
+    match load(path, format, out) {
+        Loaded::Parsed(parsed) => f(&parsed.plan, out),
+        Loaded::Failed(code) => code,
+    }
+}
+
 /// A plan read and parsed, or the exit code when it was not.
 enum Loaded {
     Parsed(Box<Parsed>),
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t8-main-rs.patch"
Remove-Item "$env:TEMP\p5-t8-main-rs.patch"
```

File `crates/planrun-cli/src/run.rs` (full contents):

```rust
//! `planrun run`, `resume` and `status` (docs/architecture.md §7; decisions D74, D76, D77):
//! the lock, the journal of this plan, the policy, and the runner with the line prompts.

use std::path::{Path, PathBuf};

use planrun_exec::journal::{self, End, Journal, Line, Record};
use planrun_exec::lock::{self, LockError};
use planrun_exec::policy::{Policy, PolicyClass};
use planrun_exec::runner::resume::{Analysis, analyse};
use planrun_exec::runner::{Finished, RunConfig, Runner};
use planrun_exec::shell::{SessionConfig, install_interrupt_handler};
use planrun_exec::workspace::Workspace;
use planrun_model::Plan;

use crate::ui::LineUi;
use crate::{OK, PLAN_ERRORS, USAGE};

/// The flags of `run` and `resume`.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// `--auto`.
    pub auto: Option<String>,
    /// `--run-expected-failures`.
    pub run_expected_failures: bool,
    /// `--from`.
    pub from: Option<String>,
    /// `--to`.
    pub to: Option<String>,
    /// `--restart` (run only): give up an unfinished journal, keeping it under another name.
    pub restart: bool,
}

/// The repository, the plan's journal and its recorded path.
struct Context {
    ws: Workspace,
    git_dir: PathBuf,
    journal: PathBuf,
    plan_path: String,
}

fn context(plan_file: &Path, out: &mut String) -> Result<Context, u8> {
    let here = std::env::current_dir().map_err(|e| {
        out.push_str(&format!("planrun: the current directory: {e}\n"));
        USAGE
    })?;
    let ws = Workspace::open(&here, Vec::new()).map_err(|e| {
        out.push_str(&format!("planrun: {e}\n"));
        USAGE
    })?;
    let git_dir = ws.git().git_dir().map_err(|e| {
        out.push_str(&format!("planrun: {e}\n"));
        USAGE
    })?;
    let stem = plan_file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "plan".to_owned());
    let canonical = std::fs::canonicalize(plan_file).unwrap_or_else(|_| plan_file.to_path_buf());
    let plan_path = match canonical.strip_prefix(ws.root()) {
        Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
        Err(_) => canonical.display().to_string(),
    };
    Ok(Context {
        journal: journal::path_for(&git_dir, &stem),
        ws,
        git_dir,
        plan_path,
    })
}

/// The journal's lines, after checking that it belongs to this plan (D76): a journal of the
/// same stem that another plan path wrote is a hard stop.
fn lines(ctx: &Context, out: &mut String) -> Result<Vec<Line>, u8> {
    let lines = journal::read(&ctx.journal).map_err(|e| {
        out.push_str(&format!("planrun: {e}\n"));
        USAGE
    })?;
    let other = lines.iter().find_map(|l| match &l.record {
        Record::Run { plan, .. } if *plan != ctx.plan_path => Some(plan.clone()),
        _ => None,
    });
    if let Some(other) = other {
        out.push_str(&format!(
            "planrun: the journal {} belongs to the plan {other}, not to {}: two plans share \
             a file name; rename one of them\n",
            ctx.journal.display(),
            ctx.plan_path
        ));
        return Err(USAGE);
    }
    Ok(lines)
}

fn take_lock(ctx: &Context, out: &mut String) -> Result<lock::RunLock, u8> {
    lock::acquire(&journal::dir(&ctx.git_dir)).map_err(|e| {
        match e {
            LockError::Held { .. } => out.push_str(&format!("planrun: {e}\n")),
            LockError::Io(_) => out.push_str(&format!("planrun: {e}\n")),
        }
        USAGE
    })
}

fn task_index(
    plan: &Plan,
    id: Option<&str>,
    flag: &str,
    out: &mut String,
) -> Result<Option<usize>, u8> {
    let Some(id) = id else {
        return Ok(None);
    };
    match plan.tasks.iter().position(|t| &*t.id.raw == id) {
        Some(i) => Ok(Some(i)),
        None => {
            out.push_str(&format!(
                "planrun: {flag} {id}: the plan has no such task\n"
            ));
            Err(USAGE)
        }
    }
}

fn policy(auto: Option<&str>, run_expected_failures: bool, out: &mut String) -> Result<Policy, u8> {
    let classes: Vec<PolicyClass> = match auto {
        Some(text) if !text.is_empty() => Policy::parse_auto(text).map_err(|e| {
            out.push_str(&format!("planrun: {e}\n"));
            USAGE
        })?,
        _ => Vec::new(),
    };
    Policy::new(&classes, run_expected_failures).map_err(|e| {
        out.push_str(&format!("planrun: {e}\n"));
        USAGE
    })
}

fn session(ctx: &Context) -> SessionConfig {
    SessionConfig {
        root: ctx.ws.root().to_path_buf(),
        env: Vec::new(),
        timeout: None,
        temp: std::env::temp_dir(),
    }
}

fn open_journal(ctx: &Context, out: &mut String) -> Result<Journal, u8> {
    let mut journal = Journal::open(&ctx.journal).map_err(|e| {
        out.push_str(&format!("planrun: {}: {e}\n", ctx.journal.display()));
        USAGE
    })?;
    #[cfg(feature = "e2e")]
    if let Some(observer) = crate::e2e::observer() {
        journal.observe(observer);
    }
    #[cfg(not(feature = "e2e"))]
    let _ = &mut journal;
    Ok(journal)
}

fn finished(
    name: &str,
    result: Result<Finished, planrun_exec::runner::RunError>,
    out: &mut String,
) -> u8 {
    match result {
        Ok(Finished {
            end: End::Completed,
            ..
        }) => {
            out.push_str(&format!("planrun: {name}: done\n"));
            OK
        }
        Ok(Finished { reason, .. }) => {
            out.push_str(&format!(
                "planrun: {name}: stopped: {}; `planrun resume {name}` goes on from here\n",
                reason.unwrap_or_default()
            ));
            PLAN_ERRORS
        }
        Err(e) => {
            out.push_str(&format!("planrun: {name}: {e}\n"));
            USAGE
        }
    }
}

/// `planrun run`: refuses a plan with an unfinished journal unless `--restart` keeps it aside.
pub fn run(plan_file: &Path, plan: &Plan, opts: &Options, out: &mut String) -> u8 {
    let name = plan_file.display().to_string();
    let ctx = match context(plan_file, out) {
        Ok(ctx) => ctx,
        Err(code) => return code,
    };
    let _lock = match take_lock(&ctx, out) {
        Ok(lock) => lock,
        Err(code) => return code,
    };
    let lines = match lines(&ctx, out) {
        Ok(lines) => lines,
        Err(code) => return code,
    };
    let unfinished = analyse(&lines).is_some_and(|a| a.resumable());
    if unfinished && !opts.restart {
        out.push_str(&format!(
            "planrun: {name} has an unfinished run: `planrun resume {name}` goes on with it, \
             `planrun run --restart {name}` gives it up (its journal is kept)\n"
        ));
        return USAGE;
    }
    if unfinished {
        let kept = ctx.journal.with_extension(format!("{}.jsonl", now_ms()));
        if let Err(e) = std::fs::rename(&ctx.journal, &kept) {
            out.push_str(&format!("planrun: {}: {e}\n", ctx.journal.display()));
            return USAGE;
        }
        println!(
            "planrun: the unfinished journal is kept as {}",
            kept.display()
        );
    }
    let (from, to) = match (
        task_index(plan, opts.from.as_deref(), "--from", out),
        task_index(plan, opts.to.as_deref(), "--to", out),
    ) {
        (Ok(from), Ok(to)) => (from, to),
        (Err(code), _) | (_, Err(code)) => return code,
    };
    let policy = match policy(opts.auto.as_deref(), opts.run_expected_failures, out) {
        Ok(policy) => policy,
        Err(code) => return code,
    };
    let mut journal = match open_journal(&ctx, out) {
        Ok(journal) => journal,
        Err(code) => return code,
    };
    install_interrupt_handler();
    let mut ui = LineUi::new(plan, ctx.ws.root().to_path_buf());
    let config = RunConfig {
        plan_path: ctx.plan_path.clone(),
        policy,
        from,
        to,
        session: session(&ctx),
    };
    let result = Runner::new(plan, &ctx.ws, &mut journal, &mut ui, config).run();
    finished(&name, result, out)
}

/// `planrun resume`: the journal's last chain, with its policy and range unless the flags say
/// otherwise.
pub fn resume(plan_file: &Path, plan: &Plan, opts: &Options, out: &mut String) -> u8 {
    let name = plan_file.display().to_string();
    let ctx = match context(plan_file, out) {
        Ok(ctx) => ctx,
        Err(code) => return code,
    };
    let _lock = match take_lock(&ctx, out) {
        Ok(lock) => lock,
        Err(code) => return code,
    };
    let lines = match lines(&ctx, out) {
        Ok(lines) => lines,
        Err(code) => return code,
    };
    let Some(analysis) = analyse(&lines).filter(Analysis::resumable) else {
        out.push_str(&format!(
            "planrun: {name}: there is no unfinished run to resume\n"
        ));
        return USAGE;
    };
    let journaled = analysis.run.auto.join(",");
    let auto = opts.auto.clone().unwrap_or(journaled);
    let expected = opts.run_expected_failures || analysis.run.run_expected_failures;
    let policy = match policy(Some(&auto), expected, out) {
        Ok(policy) => policy,
        Err(code) => return code,
    };
    let from = opts.from.clone().or_else(|| analysis.run.from.clone());
    let to = opts.to.clone().or_else(|| analysis.run.to.clone());
    let (from, to) = match (
        task_index(plan, from.as_deref(), "--from", out),
        task_index(plan, to.as_deref(), "--to", out),
    ) {
        (Ok(from), Ok(to)) => (from, to),
        (Err(code), _) | (_, Err(code)) => return code,
    };
    let mut journal = match open_journal(&ctx, out) {
        Ok(journal) => journal,
        Err(code) => return code,
    };
    install_interrupt_handler();
    let mut ui = LineUi::new(plan, ctx.ws.root().to_path_buf());
    let config = RunConfig {
        plan_path: ctx.plan_path.clone(),
        policy,
        from,
        to,
        session: session(&ctx),
    };
    let result = Runner::new(plan, &ctx.ws, &mut journal, &mut ui, config).resume(&analysis);
    finished(&name, result, out)
}

/// `planrun status`: what the journal says, read-only; takes no lock.
pub fn status(plan_file: &Path, plan: &Plan, out: &mut String) -> u8 {
    let name = plan_file.display().to_string();
    let ctx = match context(plan_file, out) {
        Ok(ctx) => ctx,
        Err(code) => return code,
    };
    let lines = match lines(&ctx, out) {
        Ok(lines) => lines,
        Err(code) => return code,
    };
    out.push_str(&format!(
        "planrun · {name} · journal {}\n",
        ctx.journal.display()
    ));
    let Some(a) = analyse(&lines) else {
        out.push_str("no run yet\n");
        return OK;
    };
    let state = match a.end {
        Some(End::Completed) => "completed",
        Some(End::Stopped) => "stopped (resume goes on)",
        None => "interrupted (resume goes on)",
    };
    out.push_str(&format!("last run: {state}\n"));
    if a.run.fingerprint != *plan.source.fingerprint {
        out.push_str("the plan changed since that run: resume will refuse it\n");
    }
    let head = ctx.ws.git().head().ok();
    if head.as_ref() != Some(&a.head) {
        out.push_str("HEAD is not where the journal left it\n");
    }
    for task in &plan.tasks {
        let id = task.id.raw.to_string();
        let mark = if a.done.contains(&id) {
            "[✓]"
        } else if a.interrupted.as_deref() == Some(id.as_str()) {
            "[>]"
        } else {
            "[ ]"
        };
        out.push_str(&format!("  {mark} {id}  {}\n", plan.text(&task.title)));
    }
    if let Some((at, _)) = a.running() {
        out.push_str(&format!(
            "was running: step {}, action {} of {}\n",
            at.step + 1,
            at.action + 1,
            at.task
        ));
    }
    OK
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
}
```

File `crates/planrun-cli/src/snapshots/planrun__view__tests__assay_style_ir.snap` (full contents):

```text
---
source: crates/planrun-cli/src/view.rs
expression: "Viewer::new(&plan).plan()"
---
plan: tests/corpus/synthetic/assay-style.md
format: assay
title: "Greeter Plan — Part 1: the greet module"
phase: P1
preamble: []
tasks:
  - id: P1-T1
    title: the greet module
    at: L24
    files:
      - Create src/greet.rs
      - Modify src/lib.rs
    steps:
      - step: intro
        at: L26
        actions:
          - at: L26
            by: fallback
            note:
              text: "**Goal:** `src/greet.rs` with `greeting`, and `lib.rs` delegating to it."
      - step: 1 — write the files
        at: L33
        actions:
          - at: L35
            by: "file-block#assay"
            write-file:
              path: src/greet.rs
              mode: Create
              body: "//! Greetings (P1-T1).\n\n/// The greeting for `name`, with punctuation.\npub fn greeting(name: &str) -> String {\n    format!(\"Hello, {name}!\")\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn greets_by_name() {\n        assert_eq!(super::greeting(\"Rens\"), \"Hello, Rens!\");\n    }\n}\n"
          - at: L54
            by: "file-block#assay"
            apply-patch:
              path: src/lib.rs
              pre: aa80cf1
              post: 97bd92a
              patch: "diff --git a/src/lib.rs b/src/lib.rs\nindex aa80cf1..97bd92a 100644\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,8 +1,9 @@\n //! A tiny fixture crate for the planrun corpus.\n \n+pub mod greet;\n pub mod old;\n \n /// The greeting for `name`.\n pub fn greeting(name: &str) -> String {\n-    old::legacy_greeting(name)\n+    greet::greeting(name)\n }\n"
      - step: 2 — verify
        at: L79
        actions:
          - at: L81
            by: fallback
            note:
              text: "Targeted: the greet tests:"
          - at: L83
            by: "script#assay"
            run:
              shell: pwsh
              intent: verify
              body: "cargo test greet\n"
              expect:
                exit: exit 0
                lines: "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s\n"
                prose: ~
          - at: L93
            by: fallback
            note:
              text: "Gate:"
          - at: L95
            by: "script#assay"
            run:
              shell: pwsh
              intent: gate
              body: "cargo test; $gate = $LASTEXITCODE; \"gate exit code: $gate\"\n"
              expect:
                exit: exit 0
                lines: "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s\ngate exit code: 0\n"
                prose: ~
      - step: 3 — commit
        at: L106
        actions:
          - at: L108
            by: fallback
            note:
              text: "Only the task's paths are staged, only when the gate's exit code above was 0:"
          - at: L110
            by: "script#assay"
            commit:
              shell: pwsh
              intent: other
              body: "if ($gate -eq 0) {\n  git add -- src/greet.rs src/lib.rs\n  git commit -m \"P1-T1: the greet module\"\n} else { \"the gate failed (exit code $gate): fix the task before committing\" }\ngit status --porcelain\n"
              expect:
                exit: not checked
                lines: ~
                prose: "the commit, then no output from `git status --porcelain`."
  - id: P1-T2
    title: "re-export greeting, remove the legacy module"
    at: L120
    files:
      - Modify src/lib.rs
      - Delete src/old.rs
      - Create docs/modules.txt
    steps:
      - step: intro
        at: L122
        actions:
          - at: L122
            by: fallback
            note:
              text: "**Goal:** `lib.rs` re-exports `greet::greeting`; `src/old.rs` is deleted; `docs/modules.txt` lists the modules."
          - at: L130
            by: "generated#notice"
            generated:
              path: docs/modules.txt
      - step: 1 — write the files
        at: L132
        actions:
          - at: L134
            by: "file-block#assay"
            apply-patch:
              path: src/lib.rs
              pre: 97bd92a
              post: 6b269fc
              patch: "diff --git a/src/lib.rs b/src/lib.rs\nindex 97bd92a..6b269fc 100644\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,9 +1,5 @@\n //! A tiny fixture crate for the planrun corpus.\n \n pub mod greet;\n-pub mod old;\n \n-/// The greeting for `name`.\n-pub fn greeting(name: &str) -> String {\n-    greet::greeting(name)\n-}\n+pub use greet::greeting;\n"
          - at: L159
            by: "delete#sentence"
            delete:
              path: src/old.rs
              via_git: false
          - at: L161
            by: "generated#notice"
            generated:
              path: docs/modules.txt
      - step: 2 — verify
        at: L163
        actions:
          - at: L165
            by: fallback
            note:
              text: "Generate the module list:"
          - at: L167
            by: "script#assay"
            run:
              shell: pwsh
              intent: verify
              body: "New-Item -ItemType Directory -Force docs | Out-Null; ((Get-ChildItem src -Name | Sort-Object) -join \"`n\") + \"`n\" | Set-Content -NoNewline docs/modules.txt\nGet-Content docs/modules.txt\n"
              expect:
                exit: exit 0
                lines: "greet.rs\nlib.rs\n"
                prose: ~
          - at: L179
            by: fallback
            note:
              text: "Gate:"
          - at: L181
            by: "script#assay"
            run:
              shell: pwsh
              intent: gate
              body: "cargo test; $gate = $LASTEXITCODE; \"gate exit code: $gate\"\n"
              expect:
                exit: exit 0
                lines: "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s\ngate exit code: 0\n"
                prose: ~
      - step: 3 — commit
        at: L192
        actions:
          - at: L194
            by: fallback
            note:
              text: "Only the task's paths are staged, only when the gate's exit code above was 0:"
          - at: L196
            by: "script#assay"
            commit:
              shell: pwsh
              intent: other
              body: "if ($gate -eq 0) {\n  git add -- src/lib.rs src/old.rs docs/modules.txt\n  git commit -m \"P1-T2: re-export greeting, remove the legacy module\"\n} else { \"the gate failed (exit code $gate): fix the task before committing\" }\ngit status --porcelain\n"
              expect:
                exit: not checked
                lines: ~
                prose: "the commit, then no output from `git status --porcelain`."
after_tasks: L206
```

File `crates/planrun-cli/src/snapshots/planrun__view__tests__assay_style_show.snap` (full contents):

```text
---
source: crates/planrun-cli/src/view.rs
expression: out
---
plan    tests/corpus/synthetic/assay-style.md
format  assay
title   Greeter Plan — Part 1: the greet module
phase   P1

P1-T1  the greet module  L24
  files  Create src/greet.rs, Modify src/lib.rs
  step intro  L26
    L26    note        **Goal:** `src/greet.rs` with `greeting`, and `lib.rs` delegating to it.  (fallback)
  step 1 — write the files  L33
    L35    write-file  src/greet.rs (Create)  (file-block#assay)
    L54    apply-patch src/lib.rs aa80cf1..97bd92a  (file-block#assay)
  step 2 — verify  L79
    L81    note        Targeted: the greet tests:  (fallback)
    L83    run         pwsh verify  cargo test greet  (script#assay)
           expect: exit 0; 1 summary line(s)
    L93    note        Gate:  (fallback)
    L95    run         pwsh gate  cargo test; $gate = $LASTEXITCODE; "gate exit code: $gate"  (script#assay)
           expect: exit 0; 2 summary line(s)
  step 3 — commit  L106
    L108   note        Only the task's paths are staged, only when the gate's exit code above w…  (fallback)
    L110   commit      pwsh  if ($gate -eq 0) {  (script#assay)
           expect: not checked; "the commit, then no output from `git status --porcelain`." (confirmed by you)

P1-T2  re-export greeting, remove the legacy module  L120
  files  Modify src/lib.rs, Delete src/old.rs, Create docs/modules.txt
  step intro  L122
    L122   note        **Goal:** `lib.rs` re-exports `greet::greeting`; `src/old.rs` is deleted…  (fallback)
    L130   generated   docs/modules.txt  (generated#notice)
  step 1 — write the files  L132
    L134   apply-patch src/lib.rs 97bd92a..6b269fc  (file-block#assay)
    L159   delete      src/old.rs  (delete#sentence)
    L161   generated   docs/modules.txt  (generated#notice)
  step 2 — verify  L163
    L165   note        Generate the module list:  (fallback)
    L167   run         pwsh verify  New-Item -ItemType Directory -Force docs | Out-Null; ((Get-ChildItem src…  (script#assay)
           expect: exit 0; 2 summary line(s)
    L179   note        Gate:  (fallback)
    L181   run         pwsh gate  cargo test; $gate = $LASTEXITCODE; "gate exit code: $gate"  (script#assay)
           expect: exit 0; 2 summary line(s)
  step 3 — commit  L192
    L194   note        Only the task's paths are staged, only when the gate's exit code above w…  (fallback)
    L196   commit      pwsh  if ($gate -eq 0) {  (script#assay)
           expect: not checked; "the commit, then no output from `git status --porcelain`." (confirmed by you)

after the tasks (never run): from L206
```

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
                question.kind
            ));
        }
        self.print("");
        for line in &question.text {
            self.print(line);
        }
        loop {
            if !question.typed {
                self.print(&prompt(&question.choices));
            }
            let _ = io::stdout().flush();
            let Some(line) = self.read_line() else {
                return Reply::Stop("stdin closed before an answer".to_owned());
            };
            if question.typed {
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
fn a_real_run_without_a_terminal_stops_at_its_first_question() {
    let f = Fixture::greeter(false);
    let plan = f.outside().join(PLAN);
    std::fs::write(&plan, plan_text()).unwrap_or_else(|e| panic!("{e}"));
    let output = Command::new(env!("CARGO_BIN_EXE_planrun"))
        .args(["run", "../outside/assay-style.md"])
        .current_dir(f.path())
        .envs(f.env())
        .env("CARGO_TARGET_DIR", f.outside().join("target"))
        .output()
        .unwrap_or_else(|e| panic!("{e}"));
    let text = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("stdin"), "{text}");
    let journal = f.path().join(".git/planrun/assay-style.jsonl");
    let journal = std::fs::read_to_string(&journal).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        journal
            .lines()
            .last()
            .is_some_and(|l| l.contains("\"run-end\"")),
        "{journal}"
    );
    let missing = Command::new(env!("CARGO_BIN_EXE_planrun"))
        .args(["run", "../outside/none.md"])
        .current_dir(f.path())
        .envs(f.env())
        .output()
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(missing.status.code(), Some(2));
}
````

File `crates/planrun-cli/tests/run.rs` (full contents):

```rust
//! The P5 acceptance (docs/roadmap.md): `planrun run tests/corpus/synthetic/assay-style.md
//! --auto read,build,modify` on a fixture copy makes the two commits with the expected trees,
//! and a run killed mid-task resumes and completes without writing anything twice. Built only
//! with the `e2e` feature (D82): answers come from a pipe, and `PLANRUN_E2E_ABORT` ends the
//! process at a named seam. Each leg spells out its prompts.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant, SystemTime};

use fixture::Fixture;
use planrun_exec::journal::{self, Record};
use planrun_exec::shell::{ShellError, locate};
use planrun_model::Shell;

const PLAN: &str = "assay-style.md";
const AUTO: &str = "read,build,modify";

/// `src/greet.rs` as the plan writes it.
const GREET: &str = "//! Greetings (P1-T1).

/// The greeting for `name`, with punctuation.
pub fn greeting(name: &str) -> String {
    format!(\"Hello, {name}!\")
}

#[cfg(test)]
mod tests {
    #[test]
    fn greets_by_name() {
        assert_eq!(super::greeting(\"Rens\"), \"Hello, Rens!\");
    }
}
";

/// `src/lib.rs` after P1-T1 (blob 97bd92a) and after P1-T2 (blob 6b269fc).
const LIB_T1: &str = "//! A tiny fixture crate for the planrun corpus.

pub mod greet;
pub mod old;

/// The greeting for `name`.
pub fn greeting(name: &str) -> String {
    greet::greeting(name)
}
";
const LIB_T2: &str = "//! A tiny fixture crate for the planrun corpus.

pub mod greet;

pub use greet::greeting;
";

/// Whether pwsh is there; without it the test is skipped unless shells are required (D63).
fn pwsh() -> bool {
    match locate(Shell::Pwsh) {
        Ok(_) => true,
        Err(e @ ShellError::Unavailable { .. })
            if std::env::var_os("PLANRUN_REQUIRE_SHELLS").is_none() =>
        {
            eprintln!("skipped: {e}");
            false
        }
        Err(e) => panic!("{e}"),
    }
}

/// A fixture copy with the plan next to it, outside the repository.
fn fixture(autocrlf: bool) -> Fixture {
    let f = Fixture::greeter(autocrlf);
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/corpus/synthetic")
        .join(PLAN);
    let text = std::fs::read_to_string(&source).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(f.outside().join(PLAN), text).unwrap_or_else(|e| panic!("{e}"));
    f
}

/// A running `planrun` with its stdout read line by line.
struct Planrun {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<String>,
    seen: Vec<String>,
}

impl Planrun {
    fn start(f: &Fixture, args: &[&str], abort: Option<&str>) -> Planrun {
        let mut command = Command::new(env!("CARGO_BIN_EXE_planrun"));
        command
            .args(args)
            .arg(format!("../outside/{PLAN}"))
            .current_dir(f.path())
            .envs(f.env())
            .env("CARGO_TARGET_DIR", f.outside().join("target"))
            .env("CARGO_NET_OFFLINE", "true")
            .env_remove("PLANRUN_E2E_ABORT")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(abort) = abort {
            command.env("PLANRUN_E2E_ABORT", abort);
        }
        let mut child = command.spawn().unwrap_or_else(|e| panic!("{e}"));
        let (tx, lines) = channel();
        for pipe in [
            child
                .stdout
                .take()
                .map(|p| Box::new(p) as Box<dyn std::io::Read + Send>),
            child
                .stderr
                .take()
                .map(|p| Box::new(p) as Box<dyn std::io::Read + Send>),
        ]
        .into_iter()
        .flatten()
        {
            let tx = tx.clone();
            std::thread::spawn(move || {
                for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                    if tx.send(line).is_err() {
                        return;
                    }
                }
            });
        }
        let stdin = child.stdin.take();
        Planrun {
            child,
            stdin,
            lines,
            seen: Vec::new(),
        }
    }

    /// Reads until a line containing `prompt`; panics after five minutes or at the end.
    fn wait_for(&mut self, prompt: &str) {
        let until = Instant::now() + Duration::from_secs(300);
        loop {
            let left = until.saturating_duration_since(Instant::now());
            match self.lines.recv_timeout(left) {
                Ok(line) => {
                    let hit = line.contains(prompt);
                    self.seen.push(line);
                    if hit {
                        return;
                    }
                }
                Err(_) => panic!("no `{prompt}`; output:\n{}", self.seen.join("\n")),
            }
        }
    }

    /// Waits for `prompt`, then answers `answer`.
    fn answer(&mut self, prompt: &str, answer: &str) {
        self.wait_for(prompt);
        let stdin = self
            .stdin
            .as_mut()
            .unwrap_or_else(|| panic!("stdin closed"));
        writeln!(stdin, "{answer}").unwrap_or_else(|e| panic!("{e}"));
        stdin.flush().unwrap_or_else(|e| panic!("{e}"));
    }

    /// Waits for the process to end; returns its exit code and everything it printed.
    fn finish(mut self) -> (Option<i32>, String) {
        drop(self.stdin.take());
        let status = self.child.wait().unwrap_or_else(|e| panic!("{e}"));
        while let Ok(line) = self.lines.recv_timeout(Duration::from_secs(5)) {
            self.seen.push(line);
        }
        (status.code(), self.seen.join("\n"))
    }

    /// Kills the process, as a crash would.
    fn kill(mut self) -> String {
        self.child.kill().unwrap_or_else(|e| panic!("{e}"));
        let _ = self.child.wait();
        self.seen.join("\n")
    }
}

/// The prompts: a confirmation, a run question, the session notice, an Unknown step.
const CONFIRM: &str = "[y] yes";
const RUN: &str = "[enter] run   [s] skip";
const NOTICE: &str = "[enter] continue   [o] open shell   [q] quit";
const UNKNOWN: &str = "[r] restore and redo";

/// The tree id of `files` committed on top of the fixture, in a fresh copy.
fn expected_tree(files: &[(&str, Option<&str>)]) -> String {
    let f = Fixture::greeter(false);
    for (path, text) in files {
        match text {
            Some(text) => f.write(path, text.as_bytes()),
            None => {
                f.git(&["rm", "-q", path]);
            }
        }
    }
    f.git(&["add", "-A"]);
    f.git(&["commit", "-q", "-m", "expected"]);
    f.git(&["rev-parse", "HEAD^{tree}"]).trim().to_owned()
}

/// The two commits are there, with the subjects and trees of the plan's two tasks.
fn assert_two_commits(f: &Fixture, output: &str) {
    let subjects = f.git(&["log", "--format=%s"]);
    let subjects: Vec<&str> = subjects.lines().collect();
    assert_eq!(
        subjects,
        [
            "P1-T2: re-export greeting, remove the legacy module",
            "P1-T1: the greet module",
            "fixture"
        ],
        "{output}"
    );
    let t1 = expected_tree(&[("src/greet.rs", Some(GREET)), ("src/lib.rs", Some(LIB_T1))]);
    let t2 = expected_tree(&[
        ("src/greet.rs", Some(GREET)),
        ("src/lib.rs", Some(LIB_T2)),
        ("src/old.rs", None),
        ("docs/modules.txt", Some("greet.rs\nlib.rs\n")),
    ]);
    assert_eq!(f.git(&["rev-parse", "HEAD~1^{tree}"]).trim(), t1);
    assert_eq!(f.git(&["rev-parse", "HEAD^{tree}"]).trim(), t2);
    let lib = f.git(&["rev-parse", "HEAD:src/lib.rs"]);
    assert!(lib.starts_with("6b269fc"), "{lib}");
    assert_eq!(f.status(), "", "{output}");
}

fn records(f: &Fixture) -> Vec<Record> {
    journal::read(&f.path().join(".git/planrun/assay-style.jsonl"))
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|l| l.record)
        .collect()
}

/// Across every run of the journal: at most one `step-written` per step, and no
/// `step-prepared` for a step after it was verified on disk.
fn assert_written_once(f: &Fixture) {
    let mut written = Vec::new();
    let mut verified = Vec::new();
    for record in records(f) {
        match record {
            Record::StepWritten { at } => {
                assert!(!written.contains(&at), "written twice: {at:?}");
                written.push(at);
            }
            Record::StepVerified { at } => verified.push(at),
            Record::StepPrepared { at, .. } => {
                assert!(
                    !verified.contains(&at),
                    "prepared again after it was verified: {at:?}"
                );
            }
            _ => {}
        }
    }
}

/// Sets `paths`' mtimes to a sentinel; returns it.
fn sentinel(f: &Fixture, paths: &[&str]) -> SystemTime {
    let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
    for path in paths {
        let file = std::fs::File::options()
            .write(true)
            .open(f.path().join(path))
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        file.set_modified(time).unwrap_or_else(|e| panic!("{e}"));
    }
    time
}

fn assert_untouched(f: &Fixture, paths: &[&str], time: SystemTime) {
    for path in paths {
        let modified = std::fs::metadata(f.path().join(path))
            .and_then(|m| m.modified())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(modified, time, "{path} was written again");
    }
}

/// Leg 1: one run, no interruption. Prompts: the two commits' prose expectations.
#[test]
fn the_assay_plan_runs_to_two_commits() {
    if !pwsh() {
        return;
    }
    for autocrlf in [false, true] {
        let f = fixture(autocrlf);
        let mut p = Planrun::start(&f, &["run", "--auto", AUTO], None);
        p.answer(CONFIRM, "y");
        p.answer(CONFIRM, "y");
        let (code, output) = p.finish();
        assert_eq!(code, Some(0), "{output}");
        assert_two_commits(&f, &output);
        assert_written_once(&f);
    }
}

/// Leg 2: killed at P1-T1's commit question (`--auto read,build`: the commit is Modify), then
/// resumed with `--auto read,build,modify`. Prompts: run (killed); then the session notice and
/// the two confirmations.
#[test]
fn a_run_killed_at_the_commit_question_resumes_without_writing_again() {
    if !pwsh() {
        return;
    }
    let f = fixture(false);
    let mut p = Planrun::start(&f, &["run", "--auto", "read,build"], None);
    p.wait_for(RUN);
    p.kill();
    let written = ["src/greet.rs", "src/lib.rs"];
    let time = sentinel(&f, &written);
    let mut p = Planrun::start(&f, &["resume", "--auto", AUTO], None);
    p.answer(NOTICE, "");
    p.answer(CONFIRM, "y");
    p.answer(CONFIRM, "y");
    let (code, output) = p.finish();
    assert_eq!(code, Some(0), "{output}");
    assert!(
        output.contains("done on disk, not written again"),
        "{output}"
    );
    assert_eq!(
        f.git(&["rev-parse", "HEAD~1:src/greet.rs"]).trim(),
        f.git(&["hash-object", "src/greet.rs"]).trim()
    );
    assert_two_commits(&f, &output);
    assert_written_once(&f);
    assert_untouched(&f, &["src/greet.rs"], time);
}

/// Leg 3: ended right after P1-T2's write-ahead record (nothing written). Prompts: one
/// confirmation (P1-T1's commit); then the notice and P1-T2's confirmation.
#[test]
fn a_crash_after_the_write_ahead_record_writes_the_step_once() {
    if !pwsh() {
        return;
    }
    let f = fixture(false);
    let mut p = Planrun::start(&f, &["run", "--auto", AUTO], Some("step-prepared@P1-T2"));
    p.answer(CONFIRM, "y");
    let (code, output) = p.finish();
    assert_eq!(code, Some(134), "{output}");
    assert!(
        f.read("src/old.rs").is_some(),
        "nothing of P1-T2 was written"
    );
    let mut p = Planrun::start(&f, &["resume"], None);
    p.answer(NOTICE, "");
    p.answer(CONFIRM, "y");
    let (code, output) = p.finish();
    assert_eq!(code, Some(0), "{output}");
    let prepared = records(&f)
        .iter()
        .filter(|r| matches!(r, Record::StepPrepared { at, .. } if at.task == "P1-T2"))
        .count();
    assert_eq!(prepared, 2);
    assert_two_commits(&f, &output);
    assert_written_once(&f);
}

/// Leg 4: ended after the first of P1-T2 step 1's two writes. Prompts: one confirmation; then
/// the Unknown step (restore), the notice, and P1-T2's confirmation.
#[test]
fn a_crash_between_two_writes_is_restored_and_completed() {
    if !pwsh() {
        return;
    }
    let f = fixture(true);
    let mut p = Planrun::start(&f, &["run", "--auto", AUTO], Some("write1@P1-T2"));
    p.answer(CONFIRM, "y");
    let (code, output) = p.finish();
    assert_eq!(code, Some(134), "{output}");
    let mut p = Planrun::start(&f, &["resume"], None);
    p.answer(UNKNOWN, "r");
    p.answer(NOTICE, "");
    p.answer(CONFIRM, "y");
    let (code, output) = p.finish();
    assert_eq!(code, Some(0), "{output}");
    assert!(
        records(&f)
            .iter()
            .any(|r| matches!(r, Record::StepRestored { .. }))
    );
    assert_two_commits(&f, &output);
    assert_written_once(&f);
}

/// Leg 5: ended right after P1-T1's Commit `action-start`: HEAD did not move, so the commit
/// runs on resume. Prompts: none; then the notice and the two confirmations.
#[test]
fn a_crash_after_the_commit_s_start_runs_the_commit_on_resume() {
    if !pwsh() {
        return;
    }
    let f = fixture(false);
    let p = Planrun::start(&f, &["run", "--auto", AUTO], Some("commit-start@P1-T1"));
    let (code, output) = p.finish();
    assert_eq!(code, Some(134), "{output}");
    assert_eq!(f.git(&["log", "--format=%s"]).trim(), "fixture");
    let mut p = Planrun::start(&f, &["status"], None);
    p.wait_for("[>] P1-T1");
    let (code, _) = p.finish();
    assert_eq!(code, Some(0));
    let mut p = Planrun::start(&f, &["resume"], None);
    p.answer(NOTICE, "");
    p.answer(CONFIRM, "y");
    p.answer(CONFIRM, "y");
    let (code, output) = p.finish();
    assert_eq!(code, Some(0), "{output}");
    assert_two_commits(&f, &output);
    assert_written_once(&f);
}

/// `run` refuses a plan with an unfinished run, and `--restart` keeps its journal aside.
#[test]
fn run_refuses_an_unfinished_run_and_restart_keeps_its_journal() {
    if !pwsh() {
        return;
    }
    let f = fixture(false);
    let mut p = Planrun::start(&f, &["run"], None);
    p.answer(RUN, "q");
    let (code, output) = p.finish();
    assert_eq!(code, Some(1), "{output}");
    let p = Planrun::start(&f, &["run", "--auto", AUTO], None);
    let (code, output) = p.finish();
    assert_eq!(code, Some(2), "{output}");
    assert!(output.contains("planrun resume"), "{output}");
    let p = Planrun::start(&f, &["run", "--restart"], None);
    let (_, output) = p.finish();
    assert!(output.contains("is kept as"), "{output}");
    let kept = std::fs::read_dir(f.path().join(".git/planrun"))
        .unwrap_or_else(|e| panic!("{e}"))
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("assay-style."))
        .count();
    assert_eq!(kept, 2);
}
```

File `crates/planrun-cli/tests/snapshots/cli__show_p1_t2_step_2.snap` (full contents):

```text
---
source: crates/planrun-cli/tests/cli.rs
expression: out
---
  step 2 — verify  L163
    L165   note        Generate the module list:  (fallback)
           | Generate the module list:
    L167   run         pwsh verify  New-Item -ItemType Directory -Force docs | Out-Null; ((Get-ChildItem src…  (script#assay)
           expect: exit 0; 2 summary line(s)
           | New-Item -ItemType Directory -Force docs | Out-Null; ((Get-ChildItem src -Name | Sort-Object) -join "`n") + "`n" | Set-Content -NoNewline docs/modules.txt
           | Get-Content docs/modules.txt
    L179   note        Gate:  (fallback)
           | Gate:
    L181   run         pwsh gate  cargo test; $gate = $LASTEXITCODE; "gate exit code: $gate"  (script#assay)
           expect: exit 0; 2 summary line(s)
           | cargo test; $gate = $LASTEXITCODE; "gate exit code: $gate"
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
  the task's files: 1 error(s), 0 warning(s)
  L35    write     src/greet.rs             FAILED create-exists
  L54    patch     src/lib.rs               aa80cf1 → 97bd92a: ok
  L83    script                             not run; would ask (Build)
  L95    script                             not run; would ask (Build)
  L110   commit                             not run; would ask (Modify)

P1-T2  re-export greeting, remove the legacy module
  L130   generated docs/modules.txt         not checked (its script did not run)
  L134   patch     src/lib.rs               97bd92a → 6b269fc: ok
  L159   delete    src/old.rs               delete: ok
  L161   generated docs/modules.txt         not checked (its script did not run)
  L167   script                             not run; would ask (Modify)
  L181   script                             not run; would ask (Build)
  L196   commit                             not run; would ask (Modify)

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

File `tests/corpus/synthetic/assay-style.md` (full contents):

````markdown
# Greeter Plan — Part 1: the greet module

> **For agentic workers:** synthetic corpus plan in the assay (planbuild) format. It applies to
> `tests/fixtures/greeter` at its base state; every diff below was generated by git from that
> fixture, so its `index` lines are real blob ids.

**Goal:** a `greet` module replaces the legacy greeting.

- **P1-T1** (`greeter`): `src/greet.rs`; `lib.rs` delegates to it.
- **P1-T2** (`greeter`): `lib.rs` re-exports `greeting`; `src/old.rs` removed; `docs/modules.txt`.

**Base:** the fixture as copied. `cargo test` per task.

**Shell and environment:** PowerShell 7; Rust stable.

**Before P1-T1:** commit or stash unrelated local files: every task expects no output from `git status --porcelain`.

## Stated decisions made while building

1. **The legacy module goes in P1-T2**, not P1-T1, so P1-T1 changes one behaviour at a time.

## Tasks

### P1-T1: the greet module

**Goal:** `src/greet.rs` with `greeting`, and `lib.rs` delegating to it.

**Files**

- Create `src/greet.rs`
- Modify `src/lib.rs`

**Step 1 — write the files**

File `src/greet.rs` (full contents):

```rust
//! Greetings (P1-T1).

/// The greeting for `name`, with punctuation.
pub fn greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
    #[test]
    fn greets_by_name() {
        assert_eq!(super::greeting("Rens"), "Hello, Rens!");
    }
}
```

File `src/lib.rs` (exact diff; save the block as `$env:TEMP\p1-t1.patch`, outside the repository, then apply it):

```diff
diff --git a/src/lib.rs b/src/lib.rs
index aa80cf1..97bd92a 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,8 +1,9 @@
 //! A tiny fixture crate for the planrun corpus.
 
+pub mod greet;
 pub mod old;
 
 /// The greeting for `name`.
 pub fn greeting(name: &str) -> String {
-    old::legacy_greeting(name)
+    greet::greeting(name)
 }
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p1-t1.patch"
Remove-Item "$env:TEMP\p1-t1.patch"
```

**Step 2 — verify**

Targeted: the greet tests:

```powershell
cargo test greet
```

Expected (exit code 0; summary lines):

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
cargo test; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0:

```powershell
if ($gate -eq 0) {
  git add -- src/greet.rs src/lib.rs
  git commit -m "P1-T1: the greet module"
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`.

### P1-T2: re-export greeting, remove the legacy module

**Goal:** `lib.rs` re-exports `greet::greeting`; `src/old.rs` is deleted; `docs/modules.txt` lists the modules.

**Files**

- Modify `src/lib.rs`
- Delete `src/old.rs`
- Create `docs/modules.txt`

`docs/modules.txt` is generated by step 2's first command, never written by hand.

**Step 1 — write the files**

File `src/lib.rs` (exact diff; save the block as `$env:TEMP\p1-t2.patch`, outside the repository, then apply it):

```diff
diff --git a/src/lib.rs b/src/lib.rs
index 97bd92a..6b269fc 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,9 +1,5 @@
 //! A tiny fixture crate for the planrun corpus.
 
 pub mod greet;
-pub mod old;
 
-/// The greeting for `name`.
-pub fn greeting(name: &str) -> String {
-    greet::greeting(name)
-}
+pub use greet::greeting;
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p1-t2.patch"
Remove-Item "$env:TEMP\p1-t2.patch"
```

Delete `src/old.rs`.

`docs/modules.txt` is generated by the commands in the next step, not written by hand; commit it.

**Step 2 — verify**

Generate the module list:

```powershell
New-Item -ItemType Directory -Force docs | Out-Null; ((Get-ChildItem src -Name | Sort-Object) -join "`n") + "`n" | Set-Content -NoNewline docs/modules.txt
Get-Content docs/modules.txt
```

Expected (exit code 0; summary lines):

```text
greet.rs
lib.rs
```

Gate:

```powershell
cargo test; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0:

```powershell
if ($gate -eq 0) {
  git add -- src/lib.rs src/old.rs docs/modules.txt
  git commit -m "P1-T2: re-export greeting, remove the legacy module"
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`.

## Part 1 green state

Run from the repository root after P1-T2:

```powershell
cargo test
```

Expected: `test result: ok. 1 passed`.
````

File `tests/fixtures/greeter/.gitignore` (full contents):

```text
target/
Cargo.lock
```

**Step 2 — verify**

Targeted: the end-to-end runs of the binary, with both shells required:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-cli --features e2e --test run
```

Expected (exit code 0; summary lines):

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- Cargo.lock Cargo.toml crates/planrun-cli/Cargo.toml crates/planrun-cli/src/dry_run.rs crates/planrun-cli/src/e2e.rs crates/planrun-cli/src/main.rs crates/planrun-cli/src/run.rs crates/planrun-cli/src/snapshots/planrun__view__tests__assay_style_ir.snap crates/planrun-cli/src/snapshots/planrun__view__tests__assay_style_show.snap crates/planrun-cli/src/ui.rs crates/planrun-cli/tests/dry_run.rs crates/planrun-cli/tests/run.rs crates/planrun-cli/tests/snapshots/cli__show_p1_t2_step_2.snap crates/planrun-cli/tests/snapshots/dry_run__dry_run_create_exists.snap tests/corpus/synthetic/assay-style.md tests/fixtures/greeter/.gitignore
  $verified = [ordered]@{
      'Cargo.lock' = 'e37ae23f49b3ad15dbe14a249fb6bbfee51bb80c'
      'Cargo.toml' = '3f5506ae09cf3ef5a1e5863b88c54644d70da9ac'
      'crates/planrun-cli/Cargo.toml' = 'ad2a5f1364de3144470dcde9cf3f457594598bc6'
      'crates/planrun-cli/src/dry_run.rs' = 'f0ac633789534e058dbc197815460f285d0c04e1'
      'crates/planrun-cli/src/e2e.rs' = '265014db2a0c8c2a379200cb9867e9d9d4e2073d'
      'crates/planrun-cli/src/main.rs' = '03112e92e9fe181a8c0fce76effecf14bd36a2d5'
      'crates/planrun-cli/src/run.rs' = '5110820e8c6c403676b513ccdc0d3217c3ab19e4'
      'crates/planrun-cli/src/snapshots/planrun__view__tests__assay_style_ir.snap' = 'd10544aa049ef1bf5807390fef6b6d0986cce28f'
      'crates/planrun-cli/src/snapshots/planrun__view__tests__assay_style_show.snap' = '01577887961d7a392d067aeabef43d22016d0f11'
      'crates/planrun-cli/src/ui.rs' = '0f32f8fe52d4b8cba861320cfb60d2a5d53f43b4'
      'crates/planrun-cli/tests/dry_run.rs' = '3e17e84ae7c97e3d20480b46e662377f91981e3d'
      'crates/planrun-cli/tests/run.rs' = '8c04fa25813c6027546ef449fb5aa3688c8d7d4f'
      'crates/planrun-cli/tests/snapshots/cli__show_p1_t2_step_2.snap' = 'd35f87a6930babf5c937fd572b3861ba2491dcb6'
      'crates/planrun-cli/tests/snapshots/dry_run__dry_run_create_exists.snap' = '6b57306f63e7aaff2c949dc04417b94239cd8d23'
      'tests/corpus/synthetic/assay-style.md' = 'a153585603b9d801cac75c1e2d442134eb945aa1'
      'tests/fixtures/greeter/.gitignore' = '2c96eb1b6517f2617f9ddeae9f07f5fd7bd7ddef'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P5-T8: run, resume and status with line prompts; the e2e acceptance behind a test feature" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P5-T9: architecture, extending, plan grammar, roadmap, CLAUDE.md and the plan gate for the runner

**Goal:** The docs follow the code: architecture.md §6.1 (the runner), §6.2–§6.5 where P5 settles what they deferred, §6.6 (journal and resume) and §7 (the commands, exit codes, prompts); extending.md §7 (`ActionHandler` and `Ui`); plan-grammar §1.6, the Apply invariant, now enforced (item 24); the roadmap marks P5 done; CLAUDE.md's layout, commands and test conventions; the gate of the plan template in docs/plans/README.md runs the e2e clippy and test (D82).

**Files**

- Modify `CLAUDE.md`
- Modify `docs/architecture.md`
- Modify `docs/extending.md`
- Modify `docs/plan-grammar.md`
- Modify `docs/plans/README.md`
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
                       preflight, finding (P3); shell/ (sessions, kill.rs: the only unsafe
                       code), expect, policy/ (classifier, decisions) (P4); journal, lock,
                       done, ui, runner/ (tasks, scripts, sessions, gate, resume) (P5)
crates/planrun-cli     the `planrun` binary: check, show (view, render), run (dry_run, run,
                       ui: line prompts), resume, status; the `e2e` test feature (e2e.rs)
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
cargo test -p planrun-cli --test local_classes -- --ignored --nocapture   # their script classes
cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings
cargo test -p planrun-cli --features e2e --test run   # the end-to-end runs (D82)
cargo run -p planrun-cli -- check tests/corpus/synthetic/assay-style.md
```

The gate for every task is fmt, clippy (warnings are errors) and test, each also with the `e2e`
feature of planrun-cli (D82).

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
- Shell tests (`crates/planrun-exec/tests/shell.rs`) skip a missing shell with a message;
  `PLANRUN_REQUIRE_SHELLS=1` makes it a failure, and every plan's verification sets it (D63).
- Runner tests (`crates/planrun-exec/tests/runner.rs`, `resume.rs`) drive the runner with a
  scripted `Ui` and crash it from the journal's observer; the binary's end-to-end runs are
  `crates/planrun-cli/tests/run.rs`, behind the `e2e` feature (D82).
- Table tests (classifier, matcher) keep their rows on one line each under `#[rustfmt::skip]`.

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

File `docs/architecture.md` (exact diff; save the block as `$env:TEMP\p5-t9-architecture-md.patch`, outside the repository, then apply it):

````diff
diff --git a/docs/architecture.md b/docs/architecture.md
index a05fd7b..083f1a0 100644
--- a/docs/architecture.md
+++ b/docs/architecture.md
@@ -223,21 +223,28 @@ front end. They change rarely; formats change often. Details: [extending.md](ext
 
 ### 6.1 Runner
 
-A state machine over Task → Step → Action. Per action: `Pending → Previewed → Running →
-Done | Failed | Skipped`. Per task, the lifecycle of your §6.1 proposal:
+`runner::Runner`: a state machine over Task → Step → Action, talking to a `Ui` (§7). Per action:
+`Pending → Running → Done | Failed | Skipped`; per task:
 
-1. Show the task, its manifest and its actions (with `Note`/`Manual` in place).
+1. Show the task.
 2. **Preflight** (`preflight::preflight`): no touched path has uncommitted changes
    (`path-dirty`; other dirt is a warning, D55); manifest Create entries absent, Modify, Rewrite
    and Delete entries present (patterns and Bare entries are skipped: they name no single known
    file, D41, D45); then the task's file actions up to its first script or Manual, simulated on
    a fresh virtual tree: every preimage blob, every patch, every `before`. A Bare-targeted
-   Overwrite or Delete is `path-ambiguous`; an unscoped file action is confirmed (D50).
-3. Execute steps in order; each step's file actions as one transaction (§6.2), scripts in sessions
-   (§6.3), stopping per policy (§6.5).
-4. Commit only through the plan's own commit script, and only when the task's gate/verify passed
-   (D22).
-5. Journal every transition (§6.6).
+   Overwrite or Delete is `path-ambiguous`; a git delete of an untracked file is
+   `delete-untracked` (D78). Any error stops the run. `task-start` is journaled.
+3. The steps in order. Consecutive file actions are one transaction (§6.2): an unscoped one asks
+   first (D50), `step-prepared` is the write-ahead record, `step-written` follows. Scripts run
+   one at a time in their shell's session (§6.3) per their decision (§6.5): `action-start`, the
+   block streamed to the `Ui`, the matcher's verdict (§6.4) and what it asks, `action-end`. A
+   Manual waits; a Note is shown.
+4. Before the Commit: the task's `Generated` paths are checked once each, a Gate from a session
+   that has ended runs again, dirt outside the task's paths is asked about (D80), and the commit
+   gate (D75) must be open or opened with a typed `yes`. The plan's own commit script then runs
+   as written; it is made only when HEAD moved by one commit holding the task's paths (D79).
+   After it, paths the commit changed outside the task are reported.
+5. `task-end`. Every way a run ends is journaled as `run-end`, except a kill (§6.6).
 
 ### 6.2 Workspace and files
 
@@ -258,7 +265,8 @@ Done | Failed | Skipped`. Per task, the lifecycle of your §6.1 proposal:
 - **Pre-images** of every file about to change are stored in git's object database
   (`git hash-object -w --no-filters`: the exact bytes) and their ids journaled: rollback is
   `git cat-file`, untracked files included, no extra storage. They are unreachable objects;
-  pruning is a P5 decision (D57).
+  `git gc` may prune them after two weeks; only resume's `[r]` needs them, and it checks first
+  (D57, D77).
 - **One writer** (D52): the path is resolved first (no link, junction, short name or case
   mismatch in its existing part); directories are created one segment at a time, each checked;
   the contents go to `.planrun-<random>.tmp` in the target's directory (`create_new`); the parent
@@ -306,9 +314,15 @@ implements `ShellSession`; P5 keeps one per shell.
   `Broken` (a sentinel that does not fit). pwsh also reports `errored`: `$Error[0]` changed.
 - **Environment** (D62): planrun's own, without `BASH_ENV`, `ENV` and the git wrapper's
   variables, with `GIT_TERMINAL_PROMPT=0`.
-- **The process tree** (D64): a job object on Windows, `setsid` on Unix; a timeout (none by
-  default, D65) or a drop kills every descendant. `shell/kill.rs` holds planrun's only
-  `unsafe` code.
+- **The process tree** (D64, D71, D73): a job object on Windows, `setsid` and `PR_SET_PDEATHSIG`
+  on Linux; a timeout (none by default, D65), a drop, or a shell that exited while a descendant
+  holds its pipes kills every descendant. On Unix SIGINT, SIGTERM and SIGHUP kill the sessions'
+  groups before planrun ends; while the `[o]` shell runs, Ctrl-C goes to it. `shell/kill.rs`
+  holds planrun's only `unsafe` code.
+- **In a run** (D81): one session per shell, started at the first block; a dead one is a
+  mismatch whose retry starts a new one, announced by a notice that must be acknowledged (also
+  on resume and with `--from`); a working directory outside the repository asks before the
+  next block.
 - **Stated differences from the terminal:** `$LASTEXITCODE` is reset to 0 before each block; a
   terminating error ends the block; `$PSScriptRoot` and `$MyInvocation` name the block's temp
   file; pwsh `exit n` ends only the block, bash `exit n` the session; `GIT_TERMINAL_PROMPT=0`.
@@ -335,8 +349,8 @@ Windows with PowerShell 7 is the primary platform (D19); Linux with pwsh and bas
 - **Expected failures** (`Expected: build error`, "run the test to verify it fails"): a Verify
   script expecting failure is **skipped by default** (D12, D67), decided by the policy before
   the run; `--run-expected-failures` runs it. Gates, previews and applies always run.
-- A mismatch stops and offers (P5): accept (journaled as accepted-with-mismatch), retry, open a
-  shell, abort.
+- A mismatch stops and offers accept (journaled as `accepted`; it closes the commit gate), retry,
+  open a shell, abort; a dead session offers only retry, shell and abort (D75).
 
 ### 6.5 Policy
 
@@ -358,18 +372,36 @@ Defaults (D13, D69): file actions within the task's scope run automatically; eve
 Read ones included; Destructive scripts show the script and ask for a typed `yes`; `Manual`
 always waits; a file action outside the scope (D50) and an Apply script always ask. `--auto
 read,build,modify` (any of them) lets those classes run without asking; `destructive` is
-refused. `Policy::decide` computes the decision in P4; the runner prompts in P5.
+refused. `Policy::decide` computes the decision; the runner prompts for it (D74) and never turns one
+decision into another.
 
 ### 6.6 Journal and resume
 
-JSONL at `.git/planrun/<plan-fingerprint>.jsonl` (D18), one record per transition: run start
-(plan fingerprint, HEAD, repo root), action start (`Running`), action end (outcome, output digest,
-pre-image ids, post-state file hashes), task end, user decisions.
-
-Resume never trusts the journal alone (D14). For each action recorded `Running` or `Done`, the
-handler reports `DoneState` from the disk: postimages present → `Done`; preimages present →
-`NotDone`; neither or mixed → `Unknown`, and the user decides. A changed HEAD or a plan whose
-fingerprint differs from the journal's stops the resume with an explanation.
+JSONL at `<git-dir>/planrun/<plan file stem>.jsonl` (D18, D76), one record per transition, each
+with the schema version and the time: `run` (the plan's path and fingerprint, HEAD, the policy
+and range), `task-start`, `step-prepared` (the write-ahead record of a step: per path the
+pre-image id, the post-image's ids and mode, the directories it creates, the writer's temp
+name), `step-written`, `action-start` (with the decision and HEAD; for a Commit the task's paths
+as on disk), `action-end` (outcome, status, the last output lines, HEAD), the user's answers,
+`gate-override`, `task-end`, `run-end`. One write per record; `step-prepared`, `action-start`
+and `run-end` are synced. A torn last line is dropped, and cut off before the next append; any
+other bad line is an error. One planrun runs per repository at a time: `<git-dir>/planrun/lock`
+(D77).
+
+Resume never trusts the journal alone (D14, D77). It reads the last chain (a `run` and the
+`resume`s after it) and stops when the plan's fingerprint differs, when `.git/index.lock`
+exists, or when HEAD is not what the journal last recorded, except for the one action that was
+running (a Commit found made is done, D79; for anything else the user decides). The interrupted
+task runs again from its start in a new session. Each prepared step is compared with the disk
+(`done::step_state`): every path as planned → `Done`, not written again; every path as before →
+`NotDone`, written again; anything else → `Unknown`: restore it from its pre-images (when `git
+gc` has left them, D57) or accept it with a typed `yes`. Scripts run again under their
+decisions; a Modify or Destructive one that ran, and any that was running, asks first. Dirt in
+a path that no verified step wrote and that a remaining file action writes over may be the
+user's: it is asked about once (`resume-dirt`). A resume's own `run` record never sets the
+recorded HEAD; where resume accepts a moved HEAD it journals the running action's end (or
+`head-seen` for a retry) at once, so a stopped or crashed resume leaves the next one the same
+check.
 
 ## 7. CLI
 
@@ -377,11 +409,18 @@ fingerprint differs from the journal's stops the resume with an explanation.
 planrun check  <plan> [--format <id>]          parse + checks + coverage; writes nothing
 planrun show   <plan> [P1-T2[/1]]              the IR, a task, or a step
 planrun run    <plan> [--from P1-T2] [--to P1-T3] [--auto read,build] [--dry-run]
-               [--run-expected-failures]
-planrun resume <plan>
-planrun status <plan>
+               [--run-expected-failures] [--restart]
+planrun resume <plan> [--auto …] [--from …] [--to …] [--run-expected-failures]
+planrun status <plan>                          what the journal says; changes nothing
 ```
 
+`run` executes the plan in the repository of the current directory, under the lock (D77). It
+refuses a plan whose journal holds an unfinished run; `--restart` keeps that journal under
+another name and starts again. `resume` goes on with the unfinished run, with its policy and
+range unless the flags say otherwise. Exit 0 when the run completed, 1 when it stopped (a
+failure, a check, a question answered with quit, or no terminal to answer), 2 for usage, git,
+the lock or the journal.
+
 `--dry-run` (D55): parse, check, check real dirt once for the paths of every task, then preflight
 every task against **one virtual tree** carried across the tasks (D52), so a task sees what the
 tasks before it wrote, and simulate every file action. Commits are assumed. Runs no script and
@@ -391,7 +430,8 @@ its path is unknown from then on; every ok after that point assumes those steps
 Each script and commit shows its class and what the default policy would do with it ("would ask
 (Build)", "typed yes (Destructive)", "skipped: expected failure", D69). Exit 0 when nothing fails, 1 when something does, 2 when git or the repository cannot be used.
 
-The interactive screen (line prompts first, D15):
+The prompts are lines on stdout, answered by a line on stdin (D15, D74); only a terminal
+answers: a stdin that is not one stops the run at its first question. A sketch of the screen:
 
 ```
 planrun · greeter · plan assay-style.md (assay) · repo C:\src\greeter
````

```powershell
git apply --whitespace=nowarn "$env:TEMP\p5-t9-architecture-md.patch"
Remove-Item "$env:TEMP\p5-t9-architecture-md.patch"
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
| `Ui` | the CLI | `status`, `output`, `ask` (a `Question` of named `Choice`s, or a typed line), `view`, `diff`, `shell`; the runner decides what is asked and acts on the answer (D74). planrun-cli's `LineUi` is line prompts on std; the runner tests script their answers; a TUI (ratatui) or a non-interactive batch UI later |
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
`bad-path`, and so is a spelling Windows reserves or git refuses (D51): `:`, `<`, `>`, `"`, `|`
or a control character; a `.git` segment in any spelling (`.GIT`, `.git.`, `git~1`); a device
name (`con`, `nul.txt`, `com1`); a segment ending in a dot or a space.

### 1.5 What is never executed

- Text before the first task (the preamble) and after the tasks: shown by `show`, never run.
  Manual markers in the preamble are listed by `check` and shown before task 1.
- HTML comments.
- `Note`s: shown in place.

### 1.6 The Apply invariant

planrun always stops before an Apply-intent script, whatever `--auto` says, whether or not a
Preview ran before it: the stop never depends on a Preview action directly before it, because
the preview may be Manual (D28). `Policy::decide` makes every Apply ask (D69), and the runner
asks it (D74).

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
| Preview/apply | a `Preview…:` / `Apply:` lead, or a `-DryRun` / `-All` body: intent Preview then Apply (D29); planrun always stops between them and shows the count next to the plan's expected count. The Apply invariant (§1.5) holds |
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

File `docs/plans/README.md` (full contents):

`````markdown
# planrun's own plans

planrun's phase plans are written **in the `assay` format** (docs/plan-grammar.md §2), so that
from P5 on planrun can execute its own plans. The format is strict; follow it exactly.

## Workflow

1. Claude writes `docs/plans/YYYY-MM-DD-p<N>-<slug>.md` and stops.
2. Rens reviews it; review rulings go into the plan's "Review rulings" section.
3. The plan is executed task by task (by Rens, by Claude Code, or later by planrun), one commit
   per task.

## Shape

````markdown
# planrun P<N> — <title>

**Goal:** …
- **P<N>-T1** (<crates>): one line per task.

**Base:** <commit>. `cargo xtask`-style gate: cargo fmt --check, clippy -D warnings, cargo test.
**Shell and environment:** PowerShell 7 on Windows; Rust stable.
**Before P<N>-T1:** a clean `git status --porcelain`.

## Stated decisions made while building
1. …

## Tasks

### P<N>-T1: <title>

**Goal:** …

**Files**

- Create `path`
- Modify `path`

**Step 1 — write the files**

File `path` (full contents):

```rust
…
```

**Step 2 — verify**

Targeted: …:

```powershell
cargo test -p planrun-parse
```

Expected (exit code 0; summary lines):

```text
test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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

```powershell
if ($gate -eq 0) {
  git add -- <the task's paths>
  git commit -m "P<N>-T1: <title>"
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`.
````

## Rules

- **Full contents** for new files and for any file under about 300 lines. **Exact diffs** only when
  generated by `git diff` from a verified tree (so `index` lines are real blob ids); never
  hand-written.
- Every file a task touches is in its `**Files**` list; nothing else is touched.
- No step runs a test to see it fail (D12). Write the test and the code in the same task.
- Expected counts (`N passed`) come from an actual run, not an estimate. If the plan is written
  before the code exists, write the line as `test result: ok. … passed; 0 failed; …` and say so in
  the plan.
- Prose that is an instruction to the human contains "by hand" or "manual step", so planrun shows
  it as Manual.
`````

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
```

**Step 2 — verify**

Targeted: the spec files against extending.md:

```powershell
cargo test -p planrun-parse --test spec_files
```

Expected (exit code 0; summary lines):

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
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
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one (its blob id):

```powershell
if ($gate -eq 0) {
  git add -- CLAUDE.md docs/architecture.md docs/extending.md docs/plan-grammar.md docs/plans/README.md docs/roadmap.md
  $verified = [ordered]@{
      'CLAUDE.md' = '436e7c76db57f828f0ce457d1c839857bd8bb05e'
      'docs/architecture.md' = '083f1a05cde510ecf194884f182ebe7a72a1a0dd'
      'docs/extending.md' = 'c08b9e38eaf34ddd34ea9af8601fe3df6451a3e1'
      'docs/plan-grammar.md' = 'dc4de1d5f9312ec8ce7b12fa300a35a93872a323'
      'docs/plans/README.md' = '57fea7f54db4f9070502853649fa37bd7339b1e3'
      'docs/roadmap.md' = '39f482d759108e0b24c99d97ad7eddf294c1159b'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P5-T9: architecture, extending, plan grammar, roadmap, CLAUDE.md and the plan gate for the runner" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.
