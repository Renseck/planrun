# planrun TUI — Design

Status: adopted (r4). Its decisions are D84–D94 in docs/decisions.md; the changes outside
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

- **D93 — The TUI is tested in-process.** One loop core over any ratatui backend: the binary
  gives it crossterm, the tests a `TestBackend`, scripted keys and the real runner on a
  fixture copy. Snapshots are cell grids with style runs by token name.
- **D94 — How `planrun tui` starts.** A terminal on stdin and stdout, or exit 2; the plans
  directory defaults to `docs/plans` under the repository's root; only key presses count, and
  one loop polls keys and drains the app's channel; times are UTC.

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
check. Keys: `enter` Inspect, `R` opens Inspect and runs from there (a run is always preceded
by a visible dry run and policy), `c` re-check, `/` filter.

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
- **Policy in Inspect:** `a` changes the assumed policy (auto classes, expected failures). The
  tags are computed again from `Policy::decide`; the dry run is not run again, since it takes no
  policy. `R` uses the assumed policy, once the dry run is there; `shift+R` runs from the
  selected task.
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
  observer), the dock, History, the resume preview, the palette. Acceptance drives the TUI
  against the greeter fixture in-process, with the real runner (D93).
