# planrun P6a — Release, install and `planrun update`

**Goal:** The first P6 plan, and the first one planrun executes on its own repository. planrun becomes easy to install and keep current from GitHub: a CI gate on Windows and Linux on every push and pull request; a release workflow that gates, builds and publishes one archive per target on a `v*.*.*` tag; install scripts that download the latest release or build the clone; and `planrun update [--check]`, which installs a newer release over the running executable and removes what the previous update left behind (D83). Design: `docs/superpowers/specs/2026-11-26-release-update-design.md`. First the P5→P6 handover that CLAUDE.md requires at the end of a phase.

- **P6a-T1** (docs): the P5→P6 handover with its open items; D83, with amended-by lines in D7, D9 and D19; `LICENSE` (MIT).
- **P6a-T2** (`.github`): `ci.yml`, the gate of every task on `windows-latest` and `ubuntu-latest`.
- **P6a-T3** (`.github`): `release.yml`, on a `v*.*.*` tag: the gate, the tag against the workspace version, a build per target, the GitHub release.
- **P6a-T4** (`scripts`): `install.ps1` and `install.sh`, a release or `--from-source`.
- **P6a-T5** (`planrun-cli`, docs): `planrun update` and the cleanup at every start; README, architecture.md §7, the roadmap and CLAUDE.md.

**Base:** `1586bac` (`P5-T9: architecture, extending, plan grammar, roadmap, CLAUDE.md and the plan gate for the runner`); there fmt, clippy (with and without `e2e`), `cargo test` with `PLANRUN_REQUIRE_SHELLS=1` (27 `test result: ok` lines, 267 tests) and the `e2e` test pass. Execute from the commit of this plan and of its spec. Gate per task: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings`, `cargo test`, `cargo test -p planrun-cli --features e2e --test run`, with `PLANRUN_REQUIRE_SHELLS=1` (D82).

**Shell and environment:** PowerShell 7 on Windows; Rust 1.95.0 (stable); git on `PATH`; pwsh 7 and Git for Windows (D63); the DevReady module for `yamlvalidate` (P6a-T2, P6a-T3). Windows 10 1803 or later, for `curl.exe` and `tar.exe` in System32 (D83). P6a-T4 builds planrun with `cargo install` into a directory under the OS temp directory and removes it again. No new crate: planrun-cli takes `serde_json` from the workspace (D9).

**Before P6a-T1:** this plan and its spec committed, and a clean `git status --porcelain`. To run it with planrun itself, run a copy outside `target/`: every gate builds `target/debug/planrun.exe`, and Windows does not let the linker replace a running executable. From the repository root: `cargo build --release`, `Copy-Item target/release/planrun.exe $env:TEMP\planrun-p6a.exe`, then `& $env:TEMP\planrun-p6a.exe run docs/plans/2026-11-26-p6a-release-update.md` (add `--auto read,build,modify` to run the gates and commits without a question each). Even then five blocks ask for a typed `yes` (Destructive): the `yamlvalidate` checks of P6a-T2 and P6a-T3, both checks of P6a-T4 and the targeted block of P6a-T5 (`cargo run`); and every commit asks to confirm its prose expectation. **At that confirmation answer `y`, not `q`**: a quit there after the commit was made makes resume offer retry or keep, and only keep is right (the handover's item 10). The real plans stay in `tests/corpus/local/` (git-ignored, D16); this plan quotes none of them.

**How this plan was verified:** every task was built in a scratch clone of `1586bac` outside this repository (`core.autocrlf=false`), one commit per task. At each commit the gate passes, and so does the task's targeted command; the summary lines below come from those runs. P6a-T1 to P6a-T4 change no Rust file, so their gate is the base's. Also run while building, outside the plan: `install.ps1 -FromSource` and `install.sh --from-source` (Git Bash) into directories under the scratch folder, both ending in `planrun 0.1.0`; `System32\tar.exe -xf` on a zip made by `Compress-Archive`, and `System32\curl.exe -fsSL -o` on a `file://` URL; a running executable on Windows renamed aside (allowed) and deleted (refused until it exits), which is why the swap renames; `planrun update --check` against the private repository, which ends in `update-fetch` with curl's 404; `cargo clippy --all-targets --target x86_64-unknown-linux-gnu -- -D warnings`. The plan was generated from that history (full contents from `git show`, exact diffs from `git diff`, blob ids from `git rev-parse`), `planrun check` passes on it, and it was replayed by planrun itself on a fresh clone with `core.autocrlf=true` (an `e2e` build of the scratch tree, with answers from a pipe: enter, `y` at confirmations, `yes` at typed questions, `--auto read,build,modify`): every commit step committed, `git status --porcelain` was empty after each, and the replay's final tree is the scratch tree. That run was stopped three times and resumed each time: a rustc crash in P6a-T1's gate (`STATUS_ACCESS_VIOLATION`, retried), the quit of the handover's item 10, and a git crash (`0xC0000374`) inside P6a-T5's end-to-end tests (retried). Its findings were then added to the handover; P6a-T1 was replayed again with them (`--to P6a-T1`), and its commit's tree is the scratch commit's.

## Stated decisions made while building

1. **The spec's T5 and T6 are one task, P6a-T5.** The pure parts of `update.rs` without the command that calls them are dead code, and `-D warnings` fails the gate on dead code; a task whose gate fails cannot be committed. The README moved to P6a-T5 too, since it documents `planrun update`.
2. **`install.ps1 -NoPath`**, not in the spec: the script adds its directory to the user `PATH`, and P6a-T4's verification installs into a temporary directory, which must not stay on Rens' `PATH`.
3. **The user `PATH` is written through the registry** (`HKCU:\Environment`, read with `DoNotExpandEnvironmentNames`, written as `ExpandString`), not with `[Environment]::SetEnvironmentVariable`, which would expand every `%VARIABLE%` in it and store a plain string. Setting and clearing `PLANRUN_INSTALL` through .NET afterwards sends the settings-changed broadcast, so a terminal started from Explorer sees the new `PATH`.
4. **`-FromSource` needs an `-InstallDir` named `bin`** (`--install-dir` the same): `cargo install --root R` installs into `R\bin`, and the download half must put the executable in the same directory, so `planrun update` replaces the one on `PATH`.
5. **`install.sh` has no executable bit.** A full-contents block carries no file mode, so the README and the script's header call it as `sh scripts/install.sh`; the one-liner pipes it into `sh`.
6. **The Linux release is built on `ubuntu-22.04`**, the oldest hosted runner: a gnu binary needs at least the glibc it was linked against. CI's gate runs on `ubuntu-latest`.
7. **`.new` and `.old` are named after the running executable's stem** (`sibling` in `update.rs`): `planrun.exe` → `planrun.new.exe`, `planrun.old.exe`. For the installed executable that is D83's list; a renamed copy gets names after its own stem and nothing else is touched.
8. **The swap's two strategies are both compiled on every platform** and chosen with `cfg!(windows)`: each one then has a caller everywhere (no dead code on Linux), and both are tested on Windows and Linux through an injected rename (`a_failed_second_rename_puts_the_old_binary_back`, `a_failed_atomic_rename_leaves_the_binary_and_removes_the_staged_copy`).
9. **`update`'s output is buffered like every other command's** (`main` prints `out` at the end). The download is a few megabytes; a progress line is not worth a second output path.
10. **A tag that is no version is an error** (`update-fetch`), and so is a `--version` that does not print the release's version as a whole word (`planrun 0.2.01` is not `0.2.0`; `version_matches`).
11. **Not checked while building:** actionlint (its download was not allowed in the building session; both workflows pass `yamlvalidate` and get their first real run on GitHub after the push), the download half of the install scripts and a real `planrun update` (no release exists yet), and anything on a Linux machine. "After execution" lists the checks.
12. **The replay's findings went into the handover** (items 10 and 11), since the handover is what the next session reads; the plan only warns about item 10 under "Before P6a-T1".
13. **`LICENSE` names `Renseck`**, the git user name of this repository, as the copyright holder. Change the line in P6a-T1 before executing if it should be a full name.

## Tasks

### P6a-T1: the P5 handover, D83 and the license

**Goal:** The handover CLAUDE.md requires at the end of P5, with every open item of the P5 review (the P5 plan's stated decision 22, the two broken message strings, `resume-dirt` and scripts) and the ones carried; D83 at the end of docs/decisions.md, with amended-by lines in D7 (what `update` writes), D9 (`serde_json` in planrun-cli, the update crates rejected) and D19 (CI on Linux); the MIT `LICENSE` that `Cargo.toml` already names.

**Files**

- Create `docs/handovers/2026-11-26-p5-to-p6.md`
- Modify `docs/decisions.md`
- Create `LICENSE`

Over 300 lines, so it comes as an exact diff: `docs/decisions.md`.

**Step 1 — write the files**

File `docs/handovers/2026-11-26-p5-to-p6.md` (full contents):

```markdown
# Handover: P5 → P6

The state of planrun at the end of P5, for the session that starts P6. Read CLAUDE.md first, then
this file, then docs/roadmap.md (P6) and docs/architecture.md §6 and §7. The latest handover in
this folder is the current one.

## Where things stand (2026-11-26)

- **P0–P5 are done.** `main` is at `1586bac` (`P5-T9: architecture, extending, plan grammar,
  roadmap, CLAUDE.md and the plan gate for the runner`), on top of `da91500` (`Missing T2
  commits`) and `832c903` (P5-T8). Rens executed `docs/plans/2026-11-25-p5-runner-journal-resume.md`
  task by task. Green state on Windows: fmt, clippy (with and without `e2e`), `cargo test` with
  `PLANRUN_REQUIRE_SHELLS=1` (27 `test result: ok` lines, 267 tests) and
  `cargo test -p planrun-cli --features e2e --test run`.
- **Commands:** `planrun check`, `show`, `run` (with `--dry-run`), `resume` and `status`. planrun
  can execute its own plans from P6 on.
- **Local corpus:** on this machine `tests/corpus/local/` holds only its README; the six real
  plans are not there, so `cargo test -- --ignored local_corpus` passes without parsing any.
  Copy them back before a change to planrun-parse.
- **Decisions D71–D82** were agreed at the start of P5 and are in docs/decisions.md.
- **No release exists yet,** and the repository is private. P6a (the first P6 plan) adds CI, the
  release workflow, install scripts and `planrun update` (D83).

## What P5 built

| Task | What |
|---|---|
| P5-T1 | D71–D82, with amended-by lines; D9 drops `dialoguer` and rejects `ctrlc` |
| P5-T2 | sessions end when the shell is gone while a descendant holds its pipes, and when a block cannot be written; the bash wrapper calls its commands through `builtin`; pwsh runs with `RemoteSigned`; `PR_SET_PDEATHSIG` on Linux; the interrupt handlers (D71, D73) |
| P5-T3 | sed is Read only when its script neither writes nor runs commands (D72) |
| P5-T4 | the step record, the disk checked again between prepare and commit, modes, temp names chosen at prepare, `delete-untracked`, a journaled step's state on disk (`done`, D77, D78) |
| P5-T5 | the journal, the run lock, HEAD and batch hashes from git (D76, D77) |
| P5-T6 | the `Ui` trait and the runner: tasks, file transactions with the journal, scripts in sessions, prompts per decision, the commit gate, the checks around a commit (D73–D75, D79–D81) |
| P5-T7 | resume: the journal's last chain against the disk and HEAD, the interrupted task run again (D77) |
| P5-T8 | `run`, `resume` and `status` with line prompts; the `e2e` feature and the end-to-end acceptance (D82) |
| P5-T9 | architecture.md §6.1–§6.6 and §7, extending.md §7, plan-grammar §1.6, the roadmap, CLAUDE.md, the plan gate in docs/plans/README.md |

## Open items

### Deferred by the P5 review (the P5 plan's stated decision 22)

1. **D79 and git-ignored paths.** `commit_expect` hashes every present task path, also one git
   ignores (a Generated build output, a lockfile in `.gitignore`), so such a commit is always a
   mismatch. It fails closed. Later: leave ignored paths out (`git check-ignore`).
2. **Accept on a D79 problem** (HEAD did not move, or not one commit on top) journals the Commit
   as accepted, and the next task runs on top. Later: a typed `yes` for that accept, or no
   accept.
3. **`Tree::wait` unregisters the group before the reap:** a SIGINT in that window spares a dead
   shell's live descendants. Note it with D73's stated limit.
4. **The Linux paths** (`waitid`, `prctl`, the signal handler, executable bits) were compiled and
   linted for `x86_64-unknown-linux-gnu`, never run. P6a's CI job on `ubuntu-latest` runs them;
   P5 counts as done on Linux when that job is green.

### Found after the P5 review

5. **Two P5-T7 message strings lost their `\` line continuation**, so a run of spaces sits inside
   the text: the `resume-dirt` finding in `crates/planrun-exec/src/preflight.rs` ("…a remaining
   file action⟨spaces⟩writes it)") and the `resume-dirt` question in
   `crates/planrun-exec/src/runner/task.rs` ("…did not verify as its⟨spaces⟩own…"). Cosmetic.
   Fix them, and search the source for other runs of three or more spaces inside string
   literals.
6. **`resume-dirt` covers only paths a file action writes.** Under `--auto`, a Modify script that
   has not run yet can overwrite a user's edit made between the crash and the resume without
   asking.

### Carried

7. **Classes for `cargo xtask` and a nested `pwsh -File <repo script>`** (P5 review ruling Q3,
   D72): both stay Destructive; in the local plans they account for 49 Destructive reasons.
   Decide with the data from dogfooding.
8. **No default timeout** (D65, D73): a hung block is ended by the user.
9. **The run-from candidate** (D70) stays deferred.

### Found by the first run of planrun on its own plan (the P6a replay)

The P6a plan was replayed by planrun itself (an `e2e` build, answers from a pipe) on a fresh
clone; the final tree is the verified one. Two things came up:

10. **Quitting at a commit's confirmation after the commit was made loses the commit for
    resume.** P6a-T1's commit script committed, and the prose confirmation that follows was
    answered with `q`. The journal holds that Commit's `action-end` with `"outcome":"failed"`,
    and the resumed task's `task-start` records the new HEAD. Resume then did not see the
    Commit as made: it printed "ran before the interruption" and asked retry or keep; retry ran
    the commit script again, which printed `nothing to commit, working tree clean`, and the
    check said `the commit script ran, but HEAD did not move`. Keep was the right answer, and
    nothing said so. Likely fix (not checked against the code): journal a quit at a
    confirmation as stopped, not failed, and judge a Commit by its own `action-start` HEAD and
    expected blobs (D79), not by the task's latest HEAD.
11. **Classes seen in a real run** (data for item 7): `Import-Module DevReady; yamlvalidate …`,
    `& ./scripts/install.ps1 …`, `sh -n scripts/install.sh` (`unknown command sh`) and a block
    with `cargo run` are Destructive, so each asks for a typed `yes` even under
    `--auto read,build,modify`.

Not planrun's: during the replay rustc crashed once (`STATUS_ACCESS_VIOLATION`) and git once
(exit code `0xC0000374`) inside gates; `[r] retry` and `resume` got past both, as intended.

## Notes for P6

- **Scope** (docs/roadmap.md P6): planrun runs its own phase plans. P6a is
  `docs/plans/2026-11-26-p6a-release-update.md`: the CI gate, releases, install scripts and
  `planrun update` (D83). Execute it with planrun itself, from a release build or `cargo run`,
  and note what the run shows: that is the dogfooding data for later P6 decisions.
- **After P6a,** Rens makes the repository public, pushes, and tags `v0.1.0`; the release
  workflow publishes the first archives. The download half of the install scripts and a real
  `planrun update` can only be checked once a second release (`v0.1.1`) exists; the P6a plan's
  "After execution" lists the checks.
- Other P6 candidates (roadmap): a ratatui UI, a sidecar IR from planbuild.py,
  `.planrun/formats/` discovery, a non-interactive batch mode for CI.

## How plans are written and verified here

As in P1–P5: read what the phase needs, ask the open questions, and stop. Then build in a scratch
clone outside the repository (`git clone -c core.autocrlf=false`, and copy the local plans in when
they exist), one commit per task, with the gate at every commit. Generate the plan from the
scratch history (full contents from `git show`, exact diffs from `git diff`, blob ids from
`git rev-parse`), run `planrun check` on it, and replay it on a fresh clone with
`core.autocrlf=true`; the replay's final tree must equal the scratch tree. Copy the plan to
`docs/plans/` and stop for review. The P4→P5 handover lists the scripts and the gotchas; they
still hold. One more from P5: write Python edit scripts with the Write tool, never as a heredoc in
Git Bash, which drops `\` line continuations inside Rust string literals (item 5).
```

File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p6a-t1-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index 52426c1..d9d86c5 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -43,8 +43,8 @@ Relative, forward slashes, no `..`/`.`/empty segments, no drive letter, no backs
 Symlink escapes are refused by the workspace at write time. Nothing outside the repository is
 written except under `.git/planrun/` and the OS temp directory.
 
-Amended by D42 (wildcards), D51 (Windows-reserved spellings) and D52 (pre-images in the object
-database, the writer's temp file).
+Amended by D42 (wildcards), D51 (Windows-reserved spellings), D52 (pre-images in the object
+database, the writer's temp file) and D83 (`planrun update` writes its own install directory).
 
 ## D8 Formats are data first; built-ins are specs too
 
@@ -60,7 +60,7 @@ Each dependency is justified here; add none without a line.
 | `pulldown-cmark` | CommonMark with byte offsets; correct nested-fence handling |
 | `regex` | spec patterns; linear time, no backtracking blow-ups |
 | `serde`, `toml` | format specs |
-| `serde_json` | the journal (JSONL) |
+| `serde_json` | the journal (JSONL); the release description `planrun update` reads (D83) |
 | `thiserror` | error types |
 | `sha1_smol` | git blob ids in-process: the `blob-chain` check from a created file's contents (planrun-parse only; planrun-exec asks `git hash-object`, D56) |
 | `similar` | showing diffs of overwrites and replacements. **Not** for applying patches (D20) |
@@ -72,10 +72,12 @@ Each dependency is justified here; add none without a line.
 Rejected: `rusqlite` (D18), `ratatui` for now (D15), `tracing` for now (the journal is the record),
 `similar` as a patch engine (D20), `windows-sys` and `libc` for the process tree (D64), `dialoguer`
 (D74: it refuses a stdin that is not a terminal), `ctrlc` (D73: it brings `nix` and
-`windows-sys`).
+`windows-sys`), `ureq`, `reqwest`, `self_update`, `self-replace`, `zip` and `flate2` for
+`planrun update` (D83: curl and tar do it, like git, with no TLS stack in the build).
 
 Amended by D56 (no `sha1_smol` in planrun-exec), D64 (the hand-declared process-tree calls), D73
-(`ctrlc` rejected) and D74 (`dialoguer` dropped).
+(`ctrlc` rejected), D74 (`dialoguer` dropped) and D83 (`serde_json` in planrun-cli; the update
+crates rejected).
 
 ## D10 Line endings of written files
 
@@ -144,6 +146,8 @@ Amended by D76 (one file per plan, the records, when they are synced).
 Every real plan is PowerShell 7 on Windows. Linux (pwsh and bash) is supported and is where CI
 can run.
 
+Amended by D83 (CI runs the gate on Windows and on Linux).
+
 ## D20 Patches are applied by `git apply`
 
 With the patch on stdin, `--whitespace=nowarn`, after `--check`. Same semantics as Rens' manual
@@ -1185,3 +1189,48 @@ k-th write), `commit-start` (after a Commit's `action-start`). It ends with `exi
 `abort()`: no destructor runs, as in a kill, and on Windows no Windows Error Reporting is raised
 for the test machine to answer. A kill of `git commit` itself (HEAD moved, `action-end` missing)
 is tested in planrun-exec, where the observer can make the commit before it crashes.
+
+## D83 Releases, install scripts and `planrun update`
+
+planrun is installed and kept current from GitHub releases; `Renseck/planrun` is public, so no
+step needs a token.
+
+- **CI.** `.github/workflows/ci.yml` runs the gate of every task (fmt on Linux; clippy, test, and
+  both with the `e2e` feature, D82) on `windows-latest` and `ubuntu-latest`, with
+  `PLANRUN_REQUIRE_SHELLS=1`, on pushes to `main`, pull requests, and when the release calls it.
+  The private local corpus (D16) is not there, so its ignored tests do not run.
+- **Releases.** A pushed tag `v<major>.<minor>.<patch>` whose version equals
+  `[workspace.package]`'s runs `.github/workflows/release.yml`: the CI gate, the tag check, a
+  `--release --locked` build per target, and a GitHub release with generated notes and one
+  archive per target: `planrun-<tag>-x86_64-pc-windows-msvc.zip` and
+  `planrun-<tag>-x86_64-unknown-linux-gnu.tar.gz`, the executable at the archive's root. The
+  Linux build runs on the oldest supported Ubuntu runner, so the binary needs an older glibc.
+  Other targets are built from source. No checksum file: TLS carries the download, and the
+  `--version` check below catches a wrong or broken binary.
+- **Install scripts.** `scripts/install.ps1` and `scripts/install.sh` download the latest
+  release's archive into `%LOCALAPPDATA%\Programs\planrun\bin` or `~/.local/bin`, or build the
+  clone they are in with `cargo install --locked --root` into the same directory
+  (`-FromSource`, `--from-source`). The PowerShell script adds the directory to the user `PATH`
+  (`-NoPath`: not); the shell script only warns.
+- **`planrun update [--check]`** (planrun-cli, `update.rs`). It asks
+  `api.github.com/repos/Renseck/planrun/releases/latest` with curl, compares the tag with the
+  build's `CARGO_PKG_VERSION` (`major.minor.patch`, numerically; a tag that is no version is an
+  error), and when the release is newer downloads the build's own target's archive into
+  `planrun-update-<pid>` under the OS temp directory, unpacks it with tar, runs the unpacked
+  `planrun --version`, which must print the release's version, and only then swaps it in.
+  `--check` reports and changes nothing. Exit 0 up to date, reported or updated; 1 otherwise,
+  with a code: `update-tools`, `update-fetch`, `update-unsupported`, `update-no-asset`,
+  `update-download`, `update-verify`, `update-swap`.
+- **curl and tar are external tools**, like git: no HTTP, TLS or archive crate (D9). On Windows
+  they are `%SystemRoot%\System32\curl.exe` and `tar.exe` by absolute path (Windows 10 1803 and
+  later), so Git for Windows' curl or GNU tar earlier on `PATH` never runs; GNU tar does not read
+  zip.
+- **What it writes.** D7's rule is about plan execution. `update` is not one: it writes its
+  directory under the OS temp directory, and in the directory of `current_exe()` only
+  `planrun.new[.exe]`, `planrun.old[.exe]` and the executable itself. The new binary is copied
+  in as `.new` (mode 755 on Unix) and renamed over the executable: on Unix one atomic rename; on
+  Windows, where a running executable can be renamed but not replaced or deleted, the executable
+  goes aside as `.old` first, and comes back when the second rename fails.
+- **Cleanup.** Every planrun start removes `planrun.old[.exe]` and `planrun.new[.exe]` beside its
+  executable, best effort and silently: one or two file removals, no process. On Windows the
+  `.old` of the update just made stays until the next start, because it is still running.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6a-t1-decisions-md.patch"
Remove-Item "$env:TEMP\p6a-t1-decisions-md.patch"
```

File `LICENSE` (full contents):

```text
MIT License

Copyright (c) 2026 Renseck

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

**Step 2 — verify**

Targeted: D83 is there once, and the handover is the latest:

```powershell
(Select-String -Path docs/decisions.md -Pattern '^## D83 ').Count
(Get-ChildItem docs/handovers -Name | Sort-Object | Select-Object -Last 1)
```

Expected (exit code 0; summary lines):

```text
1
2026-11-26-p5-to-p6.md
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
  git add -- docs/handovers/2026-11-26-p5-to-p6.md docs/decisions.md LICENSE
  $verified = [ordered]@{
      'docs/handovers/2026-11-26-p5-to-p6.md' = '727bbf7131114114894792a39667b93b01aa6bc5'
      'docs/decisions.md' = 'd9d86c5a504f32afb794b15f9830f6f3cdb601d5'
      'LICENSE' = '065523ee6980399b45c800b7851b3bb9b9a2ad38'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6a-T1: the P5 handover, D83 and the license" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P6a-T2: the CI gate on Windows and Linux

**Goal:** `.github/workflows/ci.yml`: the gate of every task (fmt on Linux; clippy, test, and both with the `e2e` feature) on `windows-latest` and `ubuntu-latest`, with `PLANRUN_REQUIRE_SHELLS=1`, on pushes to `main`, on pull requests, and as a reusable workflow for the release (D83). The Linux job closes the handover's item 4 once it is green.

**Files**

- Create `.github/workflows/ci.yml`

**Step 1 — write the files**

File `.github/workflows/ci.yml` (full contents):

```yaml
# The gate of every task (CLAUDE.md, D82) on Windows and Linux. The release workflow calls it
# before it builds anything (D83).
name: ci

on:
  push:
    branches: [main]
  pull_request:
  workflow_call:

permissions:
  contents: read

env:
  CARGO_TERM_COLOR: always
  # A missing pwsh or bash fails the shell tests instead of skipping them (D63).
  PLANRUN_REQUIRE_SHELLS: "1"

jobs:
  gate:
    name: gate (${{ matrix.os }})
    strategy:
      fail-fast: false
      matrix:
        os: [windows-latest, ubuntu-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - name: fmt
        if: runner.os == 'Linux'
        run: cargo fmt --check
      - name: clippy
        run: cargo clippy --all-targets -- -D warnings
      - name: clippy (e2e)
        run: cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings
      - name: test
        run: cargo test
      - name: test (e2e)
        run: cargo test -p planrun-cli --features e2e --test run
```

**Step 2 — verify**

Targeted: the workflow is valid YAML (GitHub checks the rest on the first push):

```powershell
Import-Module DevReady; yamlvalidate .github/workflows/ci.yml
```

Expected (exit code 0; summary lines):

```text
Valid YAML: .github/workflows/ci.yml
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
  git add -- .github/workflows/ci.yml
  $verified = [ordered]@{
      '.github/workflows/ci.yml' = 'd80381e515dedb3586d795e3804d29d0fe9c8ab3'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6a-T2: the CI gate on Windows and Linux" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P6a-T3: the release workflow

**Goal:** `.github/workflows/release.yml`, on a pushed `v*.*.*` tag: the CI gate (`uses: ./.github/workflows/ci.yml`), the tag against planrun-cli's version from `cargo metadata`, a `--release --locked` build per target packed as `planrun-<tag>-<target>.zip` or `.tar.gz` with the executable at the root, and `gh release create` with generated notes and both archives (D83).

**Files**

- Create `.github/workflows/release.yml`

**Step 1 — write the files**

File `.github/workflows/release.yml` (full contents):

```yaml
# A pushed tag v<major>.<minor>.<patch> releases planrun (D83): the gate, the tag against the
# workspace version, one archive per target, and the GitHub release with both archives.
name: release

on:
  push:
    tags: ["v*.*.*"]

permissions:
  contents: read

env:
  CARGO_TERM_COLOR: always

jobs:
  gate:
    uses: ./.github/workflows/ci.yml

  version:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: the tag is the workspace version
        shell: bash
        run: |
          version=$(cargo metadata --no-deps --format-version 1 \
            | jq -r '.packages[] | select(.name == "planrun-cli") | .version')
          tag="${GITHUB_REF_NAME#v}"
          if [ "$tag" != "$version" ]; then
            echo "::error::tag $GITHUB_REF_NAME is not the workspace version $version"
            exit 1
          fi

  build:
    needs: [gate, version]
    strategy:
      matrix:
        include:
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          # The oldest supported runner: the binary needs the glibc it was linked against.
          - os: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: build
        run: cargo build --release --locked -p planrun-cli --target ${{ matrix.target }}
      - name: package (zip)
        if: runner.os == 'Windows'
        shell: pwsh
        run: |
          $name = "planrun-$env:GITHUB_REF_NAME-${{ matrix.target }}.zip"
          Compress-Archive -Path "target/${{ matrix.target }}/release/planrun.exe" -DestinationPath $name
          "ASSET=$name" | Out-File -Append -Encoding utf8 $env:GITHUB_ENV
      - name: package (tar.gz)
        if: runner.os == 'Linux'
        shell: bash
        run: |
          name="planrun-$GITHUB_REF_NAME-${{ matrix.target }}.tar.gz"
          tar -czf "$name" -C "target/${{ matrix.target }}/release" planrun
          echo "ASSET=$name" >> "$GITHUB_ENV"
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.target }}
          path: ${{ env.ASSET }}
          if-no-files-found: error

  publish:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/download-artifact@v4
        with:
          path: dist
          merge-multiple: true
      - name: the GitHub release
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          gh release create "$GITHUB_REF_NAME" dist/* \
            --repo "$GITHUB_REPOSITORY" --verify-tag --generate-notes --title "$GITHUB_REF_NAME"
```

**Step 2 — verify**

Targeted: the workflow is valid YAML:

```powershell
Import-Module DevReady; yamlvalidate .github/workflows/release.yml
```

Expected (exit code 0; summary lines):

```text
Valid YAML: .github/workflows/release.yml
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
  git add -- .github/workflows/release.yml
  $verified = [ordered]@{
      '.github/workflows/release.yml' = '1f483a95549e2c24ba31cd3fa99851ed2a636ff1'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6a-T3: the release workflow" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P6a-T4: install scripts

**Goal:** `scripts/install.ps1` and `scripts/install.sh`: the latest release's archive into `%LOCALAPPDATA%\Programs\planrun\bin` or `~/.local/bin`, or this clone built with `cargo install --locked --root` into the same directory (`-FromSource`, `--from-source`); the PowerShell script adds the directory to the user `PATH` (`-NoPath`: not), the shell script warns (D83).

**Files**

- Create `scripts/install.ps1`
- Create `scripts/install.sh`

**Step 1 — write the files**

File `scripts/install.ps1` (full contents):

```powershell
<#
.SYNOPSIS
Installs planrun on Windows: the latest release, or a build of this clone (D83).

.DESCRIPTION
Without -FromSource: downloads the latest release's x86_64-pc-windows-msvc archive from GitHub
and puts planrun.exe in -InstallDir. With -FromSource: builds this clone with
`cargo install --locked` into the same directory. Either way -InstallDir is added to the user
PATH when it is missing, and `planrun update` keeps the install current.

One line, without a clone:
  irm https://raw.githubusercontent.com/Renseck/planrun/main/scripts/install.ps1 | iex

.PARAMETER FromSource
Build the clone this script is in, instead of downloading a release. Needs cargo.

.PARAMETER InstallDir
Where planrun.exe goes. Default: $env:LOCALAPPDATA\Programs\planrun\bin. With -FromSource its
name must be bin (cargo installs into <root>\bin).

.PARAMETER NoPath
Leave the user PATH as it is.
#>
param(
    [switch]$FromSource,
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\planrun\bin'),
    [switch]$NoPath
)

$ErrorActionPreference = 'Stop'
$repo = 'Renseck/planrun'
$target = 'x86_64-pc-windows-msvc'

function Install-FromSource {
    param([string]$Dir)
    if (-not $PSScriptRoot) {
        throw 'planrun: -FromSource builds a clone: git clone https://github.com/Renseck/planrun, then run scripts\install.ps1 -FromSource in it'
    }
    $root = Split-Path -Parent $PSScriptRoot
    if (-not (Test-Path (Join-Path $root 'crates\planrun-cli\Cargo.toml'))) {
        throw "planrun: $root is not a planrun clone"
    }
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw 'planrun: -FromSource needs cargo on PATH (https://rustup.rs)'
    }
    if ((Split-Path -Leaf $Dir) -ne 'bin') {
        throw "planrun: -FromSource needs an -InstallDir named bin (cargo installs into <root>\bin): $Dir"
    }
    $cargoRoot = Split-Path -Parent $Dir
    cargo install --path (Join-Path $root 'crates\planrun-cli') --locked --force --root $cargoRoot
    if ($LASTEXITCODE -ne 0) { throw "planrun: cargo install failed (exit code $LASTEXITCODE)" }
}

function Install-Release {
    param([string]$Dir)
    # Windows PowerShell 5.1 (the one-liner may run there) must be told to use TLS 1.2.
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    $headers = @{ 'User-Agent' = 'planrun-install'; 'Accept' = 'application/vnd.github+json' }
    $release = Invoke-RestMethod -Headers $headers "https://api.github.com/repos/$repo/releases/latest"
    $name = "planrun-$($release.tag_name)-$target.zip"
    $asset = $release.assets | Where-Object name -eq $name
    if (-not $asset) { throw "planrun: release $($release.tag_name) has no $name" }
    $temp = Join-Path ([System.IO.Path]::GetTempPath()) "planrun-install-$PID"
    New-Item -ItemType Directory -Force $temp | Out-Null
    try {
        $zip = Join-Path $temp $name
        Invoke-WebRequest -Headers $headers -UseBasicParsing -Uri $asset.browser_download_url -OutFile $zip
        Expand-Archive -Path $zip -DestinationPath $temp -Force
        New-Item -ItemType Directory -Force $Dir | Out-Null
        Copy-Item (Join-Path $temp 'planrun.exe') (Join-Path $Dir 'planrun.exe') -Force
    } finally {
        Remove-Item -Recurse -Force $temp -ErrorAction SilentlyContinue
    }
}

function Add-ToUserPath {
    param([string]$Dir)
    # Read and write the registry value as it is: [Environment]::SetEnvironmentVariable would
    # expand the %VARIABLES% in the user PATH and store it as a plain string.
    $key = Get-Item 'HKCU:\Environment'
    $path = $key.GetValue('Path', '', 'DoNotExpandEnvironmentNames')
    $entries = @($path -split ';' | Where-Object { $_ })
    if ($entries -contains $Dir) { return }
    Set-ItemProperty 'HKCU:\Environment' -Name Path -Value (($entries + $Dir) -join ';') -Type ExpandString
    # Setting and clearing a variable through .NET tells running programs that the environment
    # changed, so a terminal started from Explorer sees the new PATH.
    [Environment]::SetEnvironmentVariable('PLANRUN_INSTALL', '1', 'User')
    [Environment]::SetEnvironmentVariable('PLANRUN_INSTALL', $null, 'User')
    $env:Path = "$env:Path;$Dir"
    "added $Dir to the user PATH (new terminals see it)"
}

$InstallDir = [System.IO.Path]::GetFullPath($InstallDir).TrimEnd('\')
if ($FromSource) { Install-FromSource $InstallDir } else { Install-Release $InstallDir }
if (-not $NoPath) { Add-ToUserPath $InstallDir }
& (Join-Path $InstallDir 'planrun.exe') --version
```

File `scripts/install.sh` (full contents):

```sh
#!/bin/sh
# Installs planrun on Linux: the latest release, or a build of this clone (D83).
#
#   install.sh [--from-source] [--install-dir DIR]
#
# Without --from-source: downloads the latest release's x86_64-unknown-linux-gnu archive from
# GitHub and puts planrun in DIR (default ~/.local/bin). With --from-source: builds this clone
# with `cargo install --locked` into the same directory (its name must be bin). `planrun update`
# keeps the install current. The script warns when DIR is not on PATH; it edits no rc file.
#
# One line, without a clone:
#   curl -fsSL https://raw.githubusercontent.com/Renseck/planrun/main/scripts/install.sh | sh
set -eu

repo=Renseck/planrun
target=x86_64-unknown-linux-gnu
from_source=no
dir="${HOME}/.local/bin"

fail() {
    echo "planrun: $*" >&2
    exit 1
}

while [ $# -gt 0 ]; do
    case "$1" in
        --from-source) from_source=yes ;;
        --install-dir)
            [ $# -ge 2 ] || fail "--install-dir needs a directory"
            dir="$2"
            shift
            ;;
        -h | --help)
            sed -n '2,12s/^# \{0,1\}//p' "$0"
            exit 0
            ;;
        *) fail "unknown argument: $1" ;;
    esac
    shift
done

mkdir -p "$dir"
dir=$(cd "$dir" && pwd)

install_from_source() {
    # $0 is the script when it runs from a file; under `curl … | sh` it is the shell.
    root=$(cd "$(dirname "$0")/.." 2>/dev/null && pwd) || root=
    [ -n "$root" ] && [ -f "$root/crates/planrun-cli/Cargo.toml" ] \
        || fail "--from-source builds a clone: git clone https://github.com/$repo, then run scripts/install.sh --from-source in it"
    command -v cargo >/dev/null 2>&1 || fail "--from-source needs cargo on PATH (https://rustup.rs)"
    [ "$(basename "$dir")" = bin ] \
        || fail "--from-source needs an --install-dir named bin (cargo installs into <root>/bin): $dir"
    cargo install --path "$root/crates/planrun-cli" --locked --force --root "$(dirname "$dir")"
}

install_release() {
    [ "$(uname -s)" = Linux ] && [ "$(uname -m)" = x86_64 ] \
        || fail "releases are built for Linux x86_64 only; use --from-source"
    command -v curl >/dev/null 2>&1 || fail "the download needs curl"
    command -v tar >/dev/null 2>&1 || fail "the download needs tar"
    json=$(curl -fsSL -H "Accept: application/vnd.github+json" -A planrun-install \
        "https://api.github.com/repos/$repo/releases/latest")
    tag=$(printf '%s\n' "$json" | sed -n 's/^ *"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)
    [ -n "$tag" ] || fail "the latest release has no tag"
    name="planrun-$tag-$target.tar.gz"
    temp=$(mktemp -d)
    trap 'rm -rf "$temp"' EXIT
    curl -fsSL -A planrun-install -o "$temp/$name" \
        "https://github.com/$repo/releases/download/$tag/$name" \
        || fail "release $tag has no $name"
    tar -xzf "$temp/$name" -C "$temp"
    install -m 755 "$temp/planrun" "$dir/planrun"
}

if [ "$from_source" = yes ]; then install_from_source; else install_release; fi

case ":$PATH:" in
    *":$dir:"*) ;;
    *) echo "planrun: $dir is not on PATH; add it in your shell's profile" >&2 ;;
esac
"$dir/planrun" --version
```

**Step 2 — verify**

Targeted: a build from source into a directory under the OS temp directory, without touching `PATH`, then that directory removed (a release build: a few minutes):

```powershell
$check = Join-Path ([System.IO.Path]::GetTempPath()) 'planrun-install-check'
& ./scripts/install.ps1 -FromSource -InstallDir (Join-Path $check 'bin') -NoPath
Remove-Item -Recurse -Force $check
```

Expected (exit code 0; summary lines):

```text
planrun 0.1.0
```

The shell script parses:

```bash
sh -n scripts/install.sh && echo "install.sh: syntax ok"
```

Expected (exit code 0; summary lines):

```text
install.sh: syntax ok
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
  git add -- scripts/install.ps1 scripts/install.sh
  $verified = [ordered]@{
      'scripts/install.ps1' = '1e13616f406f349e25d0edcc603feb2c3ac712d3'
      'scripts/install.sh' = '2f4d2c1fabc2397f544d2e16a4bab303b087c95a'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6a-T4: install scripts" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P6a-T5: planrun update

**Goal:** `planrun update [--check]` in `crates/planrun-cli/src/update.rs` (D83): the latest release from GitHub's API with curl, compared with `CARGO_PKG_VERSION`; when newer, the build's target's archive downloaded and unpacked under the OS temp directory, its `planrun --version` checked, and the swap over the running executable (on Windows aside as `.old`, put back when the second rename fails). Every start removes a leftover `.old` and `.new`. Unit tests in `update_tests.rs`, the network only in one ignored test. planrun-cli takes `serde_json` from the workspace. README (status, Install, update), architecture.md §7, the roadmap (P6a) and CLAUDE.md's layout.

**Files**

- Create `crates/planrun-cli/src/update.rs`
- Create `crates/planrun-cli/src/update_tests.rs`
- Modify `crates/planrun-cli/src/main.rs`
- Modify `crates/planrun-cli/Cargo.toml`
- Modify `Cargo.lock`
- Modify `README.md`
- Modify `docs/architecture.md`
- Modify `docs/roadmap.md`
- Modify `CLAUDE.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/src/main.rs`, `Cargo.lock`, `docs/architecture.md`; `CLAUDE.md` too, since it ends without a newline. `Cargo.lock` gains the one `serde_json` line of planrun-cli's dependencies; the diff is what cargo writes.

**Step 1 — write the files**

File `crates/planrun-cli/src/update.rs` (full contents):

```rust
//! `planrun update` (decision D83): the latest GitHub release, compared with this build, and
//! installed over the running executable when it is newer. curl fetches and downloads, tar
//! unpacks (both external tools, like git); the new executable must report the release's
//! version before it replaces the running one.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use crate::{OK, PLAN_ERRORS};

/// GitHub's description of the latest release (pre-releases and drafts are never "latest").
const LATEST: &str = "https://api.github.com/repos/Renseck/planrun/releases/latest";

/// `planrun update [--check]`: exit 0 when up to date, reported or updated; 1 when the update
/// failed, with the installed executable unchanged.
pub fn run(check: bool, out: &mut String) -> u8 {
    match update(check, out) {
        Ok(()) => OK,
        Err(e) => {
            out.push_str(&format!("planrun: {e}\n"));
            PLAN_ERRORS
        }
    }
}

fn update(check: bool, out: &mut String) -> Result<(), UpdateError> {
    let Some(installed) = Version::parse(env!("CARGO_PKG_VERSION")) else {
        return Err(UpdateError::new(
            "update-unsupported",
            "this build's version is not major.minor.patch",
        ));
    };
    let tools = Tools::find()?;
    let release = Release::from_json(&tools.fetch(LATEST)?)?;
    if release.version <= installed {
        out.push_str(&format!("planrun {installed} is up to date\n"));
        return Ok(());
    }
    if check {
        out.push_str(&format!(
            "planrun {} is available (installed: {installed}); run planrun update\n",
            release.version
        ));
        return Ok(());
    }
    let Some(target) = TARGET else {
        return Err(UpdateError::new(
            "update-unsupported",
            "no release is built for this platform; build it from source (scripts/install.*)",
        ));
    };
    let asset = asset_for(&release, target)?;
    let exe = std::env::current_exe()
        .map_err(|e| UpdateError::new("update-swap", format!("the running executable: {e}")))?;
    let scratch = Scratch::create()?;
    let archive = scratch.0.join(&asset.name);
    tools.download(&asset.url, &archive)?;
    tools.extract(&archive, &scratch.0, target.archive)?;
    let binary = scratch
        .0
        .join(format!("planrun{}", std::env::consts::EXE_SUFFIX));
    verify(&binary, release.version)?;
    swap(&exe, &binary)?;
    out.push_str(&format!(
        "updated planrun {installed} → {}\n",
        release.version
    ));
    Ok(())
}

/// curl and tar. On Windows the ones in System32, by absolute path: an MSYS curl or GNU tar
/// earlier on `PATH` (Git Bash) must not run, and GNU tar does not read zip.
struct Tools {
    curl: PathBuf,
    tar: PathBuf,
}

impl Tools {
    fn find() -> Result<Tools, UpdateError> {
        if !cfg!(windows) {
            return Ok(Tools {
                curl: PathBuf::from("curl"),
                tar: PathBuf::from("tar"),
            });
        }
        let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
        let system = PathBuf::from(root).join("System32");
        let tools = Tools {
            curl: system.join("curl.exe"),
            tar: system.join("tar.exe"),
        };
        for tool in [&tools.curl, &tools.tar] {
            if !tool.is_file() {
                return Err(UpdateError::new(
                    "update-tools",
                    format!("update needs {} (Windows 10 1803 or later)", tool.display()),
                ));
            }
        }
        Ok(tools)
    }

    /// The body of `url`.
    fn fetch(&self, url: &str) -> Result<String, UpdateError> {
        let mut curl = Command::new(&self.curl);
        curl.args([
            "-fsSL",
            "-H",
            "Accept: application/vnd.github+json",
            "-A",
            "planrun",
        ])
        .arg(url);
        let stdout = output(curl, "update-fetch", &format!("fetching {url}"))?;
        String::from_utf8(stdout)
            .map_err(|e| UpdateError::new("update-fetch", format!("{url}: {e}")))
    }

    fn download(&self, url: &str, to: &Path) -> Result<(), UpdateError> {
        let mut curl = Command::new(&self.curl);
        curl.args(["-fsSL", "-A", "planrun", "-o"]).arg(to).arg(url);
        output(curl, "update-download", &format!("downloading {url}"))?;
        Ok(())
    }

    fn extract(&self, archive: &Path, into: &Path, kind: Archive) -> Result<(), UpdateError> {
        let mut tar = Command::new(&self.tar);
        // bsdtar (Windows) reads zip with -x; GNU tar needs -z for gzip.
        tar.arg(match kind {
            Archive::Zip => "-xf",
            Archive::TarGz => "-xzf",
        })
        .arg(archive)
        .arg("-C")
        .arg(into);
        output(
            tar,
            "update-download",
            &format!("unpacking {}", archive.display()),
        )?;
        Ok(())
    }
}

/// Runs `command`; its stdout when it exits 0. Not found is `update-tools`; any other failure
/// is `code`, with the command's stderr.
fn output(mut command: Command, code: &'static str, what: &str) -> Result<Vec<u8>, UpdateError> {
    let program = command.get_program().to_string_lossy().into_owned();
    let ran = command.output().map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            UpdateError::new("update-tools", format!("update needs `{program}` on PATH"))
        } else {
            UpdateError::new(code, format!("{what}: {program}: {e}"))
        }
    })?;
    if ran.status.success() {
        return Ok(ran.stdout);
    }
    let stderr = String::from_utf8_lossy(&ran.stderr);
    Err(UpdateError::new(
        code,
        format!("{what}: {program} {}: {}", ran.status, stderr.trim()),
    ))
}

/// The unpacked executable runs and reports `version`, before it replaces anything.
fn verify(binary: &Path, version: Version) -> Result<(), UpdateError> {
    let fail =
        |why: String| UpdateError::new("update-verify", format!("{}: {why}", binary.display()));
    let ran = Command::new(binary)
        .arg("--version")
        .output()
        .map_err(|e| fail(e.to_string()))?;
    let printed = String::from_utf8_lossy(&ran.stdout);
    if !ran.status.success() || !version_matches(&printed, version) {
        return Err(fail(format!(
            "`--version` printed `{}`, not {version}",
            printed.trim()
        )));
    }
    Ok(())
}

/// `planrun 0.2.0` names `version`; `planrun 0.2.01` does not.
fn version_matches(printed: &str, version: Version) -> bool {
    printed
        .split_whitespace()
        .any(|word| Version::parse(word) == Some(version))
}

/// A directory of its own under the OS temp directory, removed on drop, on every path.
struct Scratch(PathBuf);

impl Scratch {
    fn create() -> Result<Scratch, UpdateError> {
        let dir = std::env::temp_dir().join(format!("planrun-update-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)
            .map_err(|e| UpdateError::new("update-download", format!("{}: {e}", dir.display())))?;
        Ok(Scratch(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// [`cleanup`] beside the running executable; called at every start (D83).
pub fn cleanup_beside_current_exe() {
    if let Ok(exe) = std::env::current_exe() {
        cleanup(&exe);
    }
}

/// A release version: `major.minor.patch`, compared numerically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    /// `0.2.0` or `v0.2.0`; anything else (a pre-release, a fourth part) is `None`.
    pub fn parse(text: &str) -> Option<Version> {
        let text = text.strip_prefix('v').unwrap_or(text);
        let mut parts = text.split('.');
        let mut next = || parts.next().and_then(number);
        let version = Version {
            major: next()?,
            minor: next()?,
            patch: next()?,
        };
        if parts.next().is_some() {
            return None;
        }
        Some(version)
    }
}

/// A number of digits only: `+1`, `-1` and an empty part are not versions.
fn number(part: &str) -> Option<u64> {
    if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    part.parse().ok()
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Why an update did not happen. The installed executable is unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateError {
    /// A stable kebab-case code (`update-fetch`, …).
    pub code: &'static str,
    /// What went wrong, for the user.
    pub message: String,
}

impl UpdateError {
    fn new(code: &'static str, message: impl Into<String>) -> UpdateError {
        UpdateError {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// One file attached to a release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    /// The file name, `planrun-v0.2.0-x86_64-pc-windows-msvc.zip`.
    pub name: String,
    /// Where to download it.
    pub url: String,
}

/// The latest release, as GitHub's `releases/latest` describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// The tag as written, `v0.2.0`; the asset names carry it.
    pub tag: String,
    /// The tag's version.
    pub version: Version,
    /// The attached files.
    pub assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct RawRelease {
    tag_name: String,
    assets: Vec<RawAsset>,
}

#[derive(Deserialize)]
struct RawAsset {
    name: String,
    browser_download_url: String,
}

impl Release {
    /// Reads GitHub's release JSON; unknown fields are ignored. A tag that is not a version is
    /// an error, never "not newer".
    pub fn from_json(text: &str) -> Result<Release, UpdateError> {
        let raw: RawRelease = serde_json::from_str(text).map_err(|e| {
            UpdateError::new("update-fetch", format!("the release description: {e}"))
        })?;
        let Some(version) = Version::parse(&raw.tag_name) else {
            return Err(UpdateError::new(
                "update-fetch",
                format!(
                    "the latest release's tag `{}` is not a version",
                    raw.tag_name
                ),
            ));
        };
        let assets = raw
            .assets
            .into_iter()
            .map(|a| Asset {
                name: a.name,
                url: a.browser_download_url,
            })
            .collect();
        Ok(Release {
            tag: raw.tag_name,
            version,
            assets,
        })
    }
}

/// How a release archive is packed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Archive {
    /// `.zip` (Windows).
    Zip,
    /// `.tar.gz` (Linux).
    TarGz,
}

impl Archive {
    fn extension(self) -> &'static str {
        match self {
            Archive::Zip => "zip",
            Archive::TarGz => "tar.gz",
        }
    }
}

/// A platform the release workflow builds for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    /// The Rust target triple.
    pub triple: &'static str,
    /// The archive it is packed in.
    pub archive: Archive,
}

/// The target of this build, when the release workflow builds one for it.
pub const TARGET: Option<Target> = if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
    Some(Target {
        triple: "x86_64-pc-windows-msvc",
        archive: Archive::Zip,
    })
} else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
    Some(Target {
        triple: "x86_64-unknown-linux-gnu",
        archive: Archive::TarGz,
    })
} else {
    None
};

/// The name of `target`'s asset of the release tagged `tag`.
pub fn asset_name(tag: &str, target: Target) -> String {
    format!(
        "planrun-{tag}-{}.{}",
        target.triple,
        target.archive.extension()
    )
}

/// The asset of `release` for `target`.
pub fn asset_for(release: &Release, target: Target) -> Result<&Asset, UpdateError> {
    let name = asset_name(&release.tag, target);
    release
        .assets
        .iter()
        .find(|a| a.name == name)
        .ok_or_else(|| {
            UpdateError::new(
                "update-no-asset",
                format!("release {} has no `{name}`", release.tag),
            )
        })
}

/// `exe` with `.<what>` before its extension: `planrun.exe` → `planrun.old.exe`.
pub fn sibling(exe: &Path, what: &str) -> PathBuf {
    let stem = exe.file_stem().unwrap_or_default().to_string_lossy();
    let name = match exe.extension() {
        Some(ext) => format!("{stem}.{what}.{}", ext.to_string_lossy()),
        None => format!("{stem}.{what}"),
    };
    exe.with_file_name(name)
}

/// Installs `new_binary` as `exe`: copied beside it as `.new`, then renamed over it. On Windows
/// the running executable cannot be replaced, only renamed: it goes aside as `.old` first, and
/// comes back when the second rename fails. [`cleanup`] removes the `.old` at the next start.
pub fn swap(exe: &Path, new_binary: &Path) -> Result<(), UpdateError> {
    let staged = sibling(exe, "new");
    fs::copy(new_binary, &staged).map_err(|e| swap_error(&staged, &e))?;
    if let Err(e) = make_executable(&staged) {
        let _ = fs::remove_file(&staged);
        return Err(swap_error(&staged, &e));
    }
    let renamed = if cfg!(windows) {
        replace_aside(exe, &staged, |from, to| fs::rename(from, to))
    } else {
        replace_atomic(exe, &staged, |from, to| fs::rename(from, to))
    };
    renamed.map_err(|e| swap_error(exe, &e))
}

fn swap_error(path: &Path, e: &io::Error) -> UpdateError {
    UpdateError::new("update-swap", format!("{}: {e}", path.display()))
}

#[cfg(unix)]
fn make_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

/// One rename of `staged` over `exe` (Unix: atomic on one filesystem, and a running executable
/// may be replaced). On failure `staged` is removed.
fn replace_atomic(
    exe: &Path,
    staged: &Path,
    mut rename: impl FnMut(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    let result = rename(staged, exe);
    if result.is_err() {
        let _ = fs::remove_file(staged);
    }
    result
}

/// `exe` aside to `.old`, then `staged` to `exe`; when the second rename fails, `.old` goes back
/// and `staged` is removed (Windows).
fn replace_aside(
    exe: &Path,
    staged: &Path,
    mut rename: impl FnMut(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    let old = sibling(exe, "old");
    if let Err(e) = remove_if_present(&old) {
        let _ = fs::remove_file(staged);
        return Err(e);
    }
    if let Err(e) = rename(exe, &old) {
        let _ = fs::remove_file(staged);
        return Err(e);
    }
    let Err(e) = rename(staged, exe) else {
        return Ok(());
    };
    let _ = rename(&old, exe);
    let _ = fs::remove_file(staged);
    Err(e)
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// Removes what an update left beside `exe` (`.old`, `.new`), best effort: a file that cannot
/// be removed (the old executable still running) stays for the next start.
pub fn cleanup(exe: &Path) {
    let _ = fs::remove_file(sibling(exe, "old"));
    let _ = fs::remove_file(sibling(exe, "new"));
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
```

File `crates/planrun-cli/src/update_tests.rs` (full contents):

```rust
//! Tests of `planrun update`'s pure parts; the network only in the ignored live test.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::*;

const RELEASE: &str = r#"{
  "url": "https://api.github.com/repos/Renseck/planrun/releases/1",
  "tag_name": "v0.2.0",
  "name": "v0.2.0",
  "draft": false,
  "prerelease": false,
  "assets": [
    {
      "name": "planrun-v0.2.0-x86_64-pc-windows-msvc.zip",
      "size": 1234,
      "browser_download_url": "https://github.com/Renseck/planrun/releases/download/v0.2.0/planrun-v0.2.0-x86_64-pc-windows-msvc.zip"
    },
    {
      "name": "planrun-v0.2.0-x86_64-unknown-linux-gnu.tar.gz",
      "size": 1234,
      "browser_download_url": "https://github.com/Renseck/planrun/releases/download/v0.2.0/planrun-v0.2.0-x86_64-unknown-linux-gnu.tar.gz"
    }
  ]
}"#;

const WINDOWS: Target = Target {
    triple: "x86_64-pc-windows-msvc",
    archive: Archive::Zip,
};

const LINUX: Target = Target {
    triple: "x86_64-unknown-linux-gnu",
    archive: Archive::TarGz,
};

fn version(text: &str) -> Version {
    Version::parse(text).unwrap_or_else(|| panic!("`{text}` is a version"))
}

/// An empty directory of the test's own under the OS temp directory.
fn scratch(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("planrun-update-test-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    dir
}

fn write(path: &Path, text: &str) {
    fs::write(path, text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[rustfmt::skip]
#[test]
fn versions_parse_with_or_without_a_v() {
    let rows: &[(&str, Option<[u64; 3]>)] = &[
        ("0.1.0", Some([0, 1, 0])),
        ("v0.1.0", Some([0, 1, 0])),
        ("v10.20.30", Some([10, 20, 30])),
        ("0.1", None),
        ("0.1.0.1", None),
        ("0.1.0-rc.1", None),
        ("v", None),
        ("", None),
        ("1..0", None),
        ("+1.0.0", None),
        ("V0.1.0", None),
    ];
    for (text, want) in rows {
        let got = Version::parse(text).map(|v| [v.major, v.minor, v.patch]);
        assert_eq!(got, *want, "{text}");
    }
}

#[test]
fn versions_compare_numerically() {
    assert!(version("0.10.0") > version("0.9.9"));
    assert!(version("1.0.0") > version("0.99.99"));
    assert!(version("0.1.1") > version("0.1.0"));
    assert_eq!(version("v0.1.0"), version("0.1.0"));
    assert_eq!(version("v0.2.0").to_string(), "0.2.0");
}

#[test]
fn the_printed_version_must_name_the_release() {
    assert!(version_matches("planrun 0.2.0\n", version("0.2.0")));
    assert!(!version_matches("planrun 0.2.01\n", version("0.2.0")));
    assert!(!version_matches("planrun 0.1.0\n", version("0.2.0")));
    assert!(!version_matches("", version("0.2.0")));
}

#[test]
fn a_release_reads_its_tag_and_assets() {
    let release = Release::from_json(RELEASE).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(release.tag, "v0.2.0");
    assert_eq!(release.version, version("0.2.0"));
    assert_eq!(release.assets.len(), 2);
    assert_eq!(
        release.assets[1].url,
        "https://github.com/Renseck/planrun/releases/download/v0.2.0/planrun-v0.2.0-x86_64-unknown-linux-gnu.tar.gz"
    );
}

#[test]
fn a_tag_that_is_no_version_or_broken_json_is_an_error() {
    let nightly = RELEASE.replace("\"tag_name\": \"v0.2.0\"", "\"tag_name\": \"nightly\"");
    let e = Release::from_json(&nightly).err();
    assert_eq!(e.map(|e| e.code), Some("update-fetch"));
    let e = Release::from_json("{\"message\": \"Not Found\"}").err();
    assert_eq!(e.map(|e| e.code), Some("update-fetch"));
}

#[test]
fn each_target_has_its_asset() {
    assert_eq!(
        asset_name("v0.2.0", WINDOWS),
        "planrun-v0.2.0-x86_64-pc-windows-msvc.zip"
    );
    assert_eq!(
        asset_name("v0.2.0", LINUX),
        "planrun-v0.2.0-x86_64-unknown-linux-gnu.tar.gz"
    );
    let release = Release::from_json(RELEASE).unwrap_or_else(|e| panic!("{e}"));
    for target in [WINDOWS, LINUX] {
        let asset = asset_for(&release, target).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(asset.name, asset_name("v0.2.0", target));
    }
    let mac = Target {
        triple: "aarch64-apple-darwin",
        archive: Archive::TarGz,
    };
    assert_eq!(
        asset_for(&release, mac).err().map(|e| e.code),
        Some("update-no-asset")
    );
}

#[test]
fn this_build_knows_its_target() {
    let triple = TARGET.map(|t| t.triple);
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        assert_eq!(triple, Some("x86_64-pc-windows-msvc"));
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        assert_eq!(triple, Some("x86_64-unknown-linux-gnu"));
    } else {
        assert_eq!(triple, None);
    }
}

#[test]
fn siblings_go_before_the_extension() {
    assert_eq!(
        sibling(Path::new("bin/planrun.exe"), "old"),
        Path::new("bin/planrun.old.exe")
    );
    assert_eq!(
        sibling(Path::new("bin/planrun"), "new"),
        Path::new("bin/planrun.new")
    );
}

#[test]
fn swap_installs_the_new_binary_and_cleanup_removes_the_old_one() {
    let dir = scratch("swap");
    let exe = dir.join(format!("planrun{}", std::env::consts::EXE_SUFFIX));
    let new = dir.join("download");
    write(&exe, "old build");
    write(&new, "new build");
    swap(&exe, &new).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(read(&exe), "new build");
    assert!(!sibling(&exe, "new").exists());
    if cfg!(windows) {
        assert_eq!(read(&sibling(&exe, "old")), "old build");
    }
    cleanup(&exe);
    assert!(!sibling(&exe, "old").exists());
    assert_eq!(read(&exe), "new build");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_failed_second_rename_puts_the_old_binary_back() {
    let dir = scratch("rollback");
    let exe = dir.join("planrun.exe");
    let staged = sibling(&exe, "new");
    write(&exe, "old build");
    write(&staged, "new build");
    let mut calls = 0;
    let result = replace_aside(&exe, &staged, |from, to| {
        calls += 1;
        if calls == 2 {
            return Err(io::Error::other("in use"));
        }
        fs::rename(from, to)
    });
    assert!(result.is_err());
    assert_eq!(calls, 3);
    assert_eq!(read(&exe), "old build");
    assert!(!staged.exists());
    assert!(!sibling(&exe, "old").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_failed_atomic_rename_leaves_the_binary_and_removes_the_staged_copy() {
    let dir = scratch("atomic");
    let exe = dir.join("planrun");
    let staged = sibling(&exe, "new");
    write(&exe, "old build");
    write(&staged, "new build");
    let result = replace_atomic(&exe, &staged, |_, _| Err(io::Error::other("read-only")));
    assert!(result.is_err());
    assert_eq!(read(&exe), "old build");
    assert!(!staged.exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_stale_old_binary_is_replaced_by_the_next_swap() {
    let dir = scratch("stale");
    let exe = dir.join("planrun.exe");
    let staged = sibling(&exe, "new");
    write(&exe, "second build");
    write(&sibling(&exe, "old"), "first build");
    write(&staged, "third build");
    replace_aside(&exe, &staged, |from, to| fs::rename(from, to)).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(read(&exe), "third build");
    assert_eq!(read(&sibling(&exe, "old")), "second build");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn cleanup_without_leftovers_does_nothing() {
    let dir = scratch("clean");
    let exe = dir.join("planrun.exe");
    write(&exe, "build");
    cleanup(&exe);
    assert_eq!(read(&exe), "build");
    let _ = fs::remove_dir_all(&dir);
}

/// Needs the network and a published release: `cargo test -p planrun-cli -- --ignored
/// the_latest_release`.
#[test]
#[ignore]
fn the_latest_release_is_fetched_and_has_this_builds_asset() {
    let tools = Tools::find().unwrap_or_else(|e| panic!("{e}"));
    let json = tools.fetch(LATEST).unwrap_or_else(|e| panic!("{e}"));
    let release = Release::from_json(&json).unwrap_or_else(|e| panic!("{e}"));
    if let Some(target) = TARGET {
        asset_for(&release, target).unwrap_or_else(|e| panic!("{e}"));
    }
}
```

File `crates/planrun-cli/src/main.rs` (exact diff; save the block as `$env:TEMP\p6a-t5-main-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/main.rs b/crates/planrun-cli/src/main.rs
index 03112e9..7ab41e2 100644
--- a/crates/planrun-cli/src/main.rs
+++ b/crates/planrun-cli/src/main.rs
@@ -8,6 +8,7 @@ mod e2e;
 mod render;
 mod run;
 mod ui;
+mod update;
 mod view;
 
 use std::path::{Path, PathBuf};
@@ -102,6 +103,12 @@ enum Command {
         #[arg(long)]
         format: Option<String>,
     },
+    /// Install the latest GitHub release over this executable when it is newer (D83).
+    Update {
+        /// Report whether a newer release exists; change nothing.
+        #[arg(long)]
+        check: bool,
+    },
 }
 
 /// Exit codes: 0 fine, 1 the plan has errors or the run stopped, 2 a usage or I/O problem.
@@ -110,6 +117,7 @@ const PLAN_ERRORS: u8 = 1;
 const USAGE: u8 = 2;
 
 fn main() -> ExitCode {
+    update::cleanup_beside_current_exe();
     let cli = Cli::parse();
     let mut out = String::new();
     let code = match &cli.command {
@@ -170,6 +178,7 @@ fn main() -> ExitCode {
                 run::status(plan, p, out)
             })
         }
+        Command::Update { check } => update::run(*check, &mut out),
     };
     if code == OK {
         print!("{out}");
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6a-t5-main-rs.patch"
Remove-Item "$env:TEMP\p6a-t5-main-rs.patch"
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
serde_json.workspace = true

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

File `Cargo.lock` (exact diff; save the block as `$env:TEMP\p6a-t5-cargo-lock.patch`, outside the repository, then apply it):

```diff
diff --git a/Cargo.lock b/Cargo.lock
index e37ae23..357df71 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -331,6 +331,7 @@ dependencies = [
  "planrun-model",
  "planrun-parse",
  "serde",
+ "serde_json",
 ]
 
 [[package]]
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6a-t5-cargo-lock.patch"
Remove-Item "$env:TEMP\p6a-t5-cargo-lock.patch"
```

File `README.md` (full contents):

````markdown
# planrun

Executes structured Markdown implementation plans against a git repository, task by task and step
by step, the way you would by hand: write the files the plan gives, apply its exact diffs, run its
commands, check their expected output, commit when the gate passes. Anything it cannot do
mechanically it shows and waits for. It is not an agent.

```
planrun check  plan.md         parse, check, coverage report; changes nothing
planrun show   plan.md P1-T2   the typed form of a task
planrun run    plan.md         execute interactively (--dry-run: change nothing)
planrun resume plan.md         continue after an interruption, re-checking the disk
planrun status plan.md         what the plan's journal says
planrun update                 install the latest release (--check: only report)
```

Status: P0–P5 are done; P6 is planrun running its own plans. See
[docs/roadmap.md](docs/roadmap.md).

## Install

Windows (PowerShell 7 or Windows PowerShell 5.1), the latest release into
`%LOCALAPPDATA%\Programs\planrun\bin`, added to the user `PATH`:

```powershell
irm https://raw.githubusercontent.com/Renseck/planrun/main/scripts/install.ps1 | iex
```

Linux x86_64, the latest release into `~/.local/bin`:

```sh
curl -fsSL https://raw.githubusercontent.com/Renseck/planrun/main/scripts/install.sh | sh
```

From a clone, built with cargo into the same directory:

```powershell
.\scripts\install.ps1 -FromSource      # Windows
sh scripts/install.sh --from-source    # Linux
```

`-InstallDir` / `--install-dir` choose another directory. Then keep it current:

```
planrun update --check   is a newer release out?
planrun update           install it over this executable
```

`planrun update` uses curl and tar (Windows 10 1803 and later ship both) and replaces only its
own executable (decision D83).

## Documents

| | |
|---|---|
| [CLAUDE.md](CLAUDE.md) | rules for working on this code |
| [docs/architecture.md](docs/architecture.md) | how it works |
| [docs/plan-grammar.md](docs/plan-grammar.md) | the plan formats it reads |
| [docs/extending.md](docs/extending.md) | adding a plan format |
| [docs/decisions.md](docs/decisions.md) | decisions D1–D83 |
| [docs/roadmap.md](docs/roadmap.md) | phases P0–P6 |
| [docs/plans/](docs/plans/) | phase plans (assay format) |
| [docs/handovers/](docs/handovers/) | the state at the end of each phase |

## Build

```powershell
cargo build
cargo test
```

Rust 2024 edition, stable toolchain. Windows with PowerShell 7 is the primary platform; Linux
works. CI runs the full gate on both (`.github/workflows/ci.yml`); a pushed `v*.*.*` tag
releases (`.github/workflows/release.yml`).

## License

MIT; see [LICENSE](LICENSE).
````

File `docs/architecture.md` (exact diff; save the block as `$env:TEMP\p6a-t5-architecture-md.patch`, outside the repository, then apply it):

````diff
diff --git a/docs/architecture.md b/docs/architecture.md
index 083f1a0..b2c61d1 100644
--- a/docs/architecture.md
+++ b/docs/architecture.md
@@ -412,8 +412,17 @@ planrun run    <plan> [--from P1-T2] [--to P1-T3] [--auto read,build] [--dry-run
                [--run-expected-failures] [--restart]
 planrun resume <plan> [--auto …] [--from …] [--to …] [--run-expected-failures]
 planrun status <plan>                          what the journal says; changes nothing
+planrun update [--check]                       install the latest release over this executable
 ```
 
+`update` (D83) is the one command that is not about a plan: it compares the latest GitHub release
+with the build's version, and when it is newer downloads the build's target's archive with curl,
+unpacks it with tar under the OS temp directory, checks that the new executable prints the
+release's version, and renames it over the running one (on Windows the running one goes aside as
+`planrun.old.exe`, which the next start removes). `--check` only reports. Exit 0 when up to
+date, reported or updated, 1 when the update failed; the installed executable is then
+unchanged.
+
 `run` executes the plan in the repository of the current directory, under the lock (D77). It
 refuses a plan whose journal holds an unfinished run; `--restart` keeps that journal under
 another name and starts again. `resume` goes on with the unfinished run, with its policy and
````

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6a-t5-architecture-md.patch"
Remove-Item "$env:TEMP\p6a-t5-architecture-md.patch"
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
```

File `CLAUDE.md` (exact diff; save the block as `$env:TEMP\p6a-t5-claude-md.patch`, outside the repository, then apply it):

````diff
diff --git a/CLAUDE.md b/CLAUDE.md
index 4eba8cb..f73d216 100644
--- a/CLAUDE.md
+++ b/CLAUDE.md
@@ -25,7 +25,8 @@ crates/planrun-exec    IR → effects: git, workspace, eol, tree, handler, trans
                        code), expect, policy/ (classifier, decisions) (P4); journal, lock,
                        done, ui, runner/ (tasks, scripts, sessions, gate, resume) (P5)
 crates/planrun-cli     the `planrun` binary: check, show (view, render), run (dry_run, run,
-                       ui: line prompts), resume, status; the `e2e` test feature (e2e.rs)
+                       ui: line prompts), resume, status, update (P6a, D83); the `e2e` test
+                       feature (e2e.rs)
 formats/*.toml         built-in plan formats (embedded): assay, superpowers, before-after; the
                        file name is the format id
 docs/                  architecture, plan-grammar, extending, decisions, roadmap, plans/,
@@ -34,6 +35,8 @@ tests/corpus/          synthetic/ (one plan per format), invalid/ (one error eac
 tests/fixtures/greeter the repo the synthetic assay plan applies to
 tests/support          fixture.rs: a temporary git copy of the greeter fixture, included
                        with #[path] by the exec and cli tests
+scripts/               install.ps1, install.sh (release or --from-source, D83); unplus.ps1
+.github/workflows/     ci.yml (the gate on Windows and Linux), release.yml (on a v*.*.* tag)
 ```
 
 ## Hard rules
````

```powershell
git apply --whitespace=nowarn "$env:TEMP\p6a-t5-claude-md.patch"
Remove-Item "$env:TEMP\p6a-t5-claude-md.patch"
```

**Step 2 — verify**

Targeted: the update tests (the live one stays ignored), and the command's help:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-cli --bin planrun update
cargo run -q -p planrun-cli -- update --help
```

Expected (exit code 0; summary lines):

```text
test result: ok. 13 passed; 0 failed; 1 ignored; 0 measured; 6 filtered out; finished in …s
Install the latest GitHub release over this executable when it is newer (D83)
      --check  Report whether a newer release exists; change nothing
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
  git add -- crates/planrun-cli/src/update.rs crates/planrun-cli/src/update_tests.rs crates/planrun-cli/src/main.rs crates/planrun-cli/Cargo.toml Cargo.lock README.md docs/architecture.md docs/roadmap.md CLAUDE.md
  $verified = [ordered]@{
      'crates/planrun-cli/src/update.rs' = 'e454c2b9745e4014e995317e02c0bd0e059679dd'
      'crates/planrun-cli/src/update_tests.rs' = '2b8664fac5f0eb5272beb161771e641e0dec8b6a'
      'crates/planrun-cli/src/main.rs' = '7ab41e20f3d56688369ec595235a19c0e24eda23'
      'crates/planrun-cli/Cargo.toml' = '4219945c4b82570f4bf44e097963a3a3effeffef'
      'Cargo.lock' = '357df71e440ab8686303608a17dd5c7b7507a38a'
      'README.md' = '6be14877074e1791bb0c3422cd49f8190e138b5d'
      'docs/architecture.md' = 'b2c61d127237ee6d5c36668196f268bf81266594'
      'docs/roadmap.md' = 'c06e035d413999f89083073dba627635eda81f2f'
      'CLAUDE.md' = 'f73d2167422d6efd088a8e5c4bc3e006ec08acd7'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P6a-T5: planrun update" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

## After execution

Each step is a manual step for Rens, in this order; none is run by planrun.

1. By hand: make `Renseck/planrun` public (Settings → General → Danger Zone). The history holds nothing from `tests/corpus/local/` except its README (checked by file name while building; D16).
2. By hand: push `main`. Both `ci` jobs must be green; the `ubuntu-latest` one closes the handover's item 4 (the Linux paths of P5 have run).
3. By hand: tag the first release and push the tag: `git tag v0.1.0`, `git push origin v0.1.0`. The `release` run publishes `planrun-v0.1.0-x86_64-pc-windows-msvc.zip` and `planrun-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`.
4. By hand, in a new terminal: `irm https://raw.githubusercontent.com/Renseck/planrun/main/scripts/install.ps1 | iex` prints `planrun 0.1.0`; `planrun update --check` prints `planrun 0.1.0 is up to date`; `cargo test -p planrun-cli -- --ignored the_latest_release` passes.
5. By hand, with the next release (`v0.1.1`, a version bump of its own): `planrun update --check` reports it, `planrun update` prints `updated planrun 0.1.0 → 0.1.1` and leaves `planrun.old.exe` beside `planrun.exe`, and the next `planrun --version` removes it. The same on Linux with `install.sh`, where no `.old` is left.
