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

Location: built-ins in `formats/<id>.toml` (embedded); per user in `<user>/formats/<id>.toml`;
per repository in `<repo>/.planrun/formats/<id>.toml`. The file name is the id: letters, digits,
`-` and `_`. `<user>` is `PLANRUN_CONFIG_DIR` when it is set, else `%APPDATA%\planrun` on
Windows and `$XDG_CONFIG_HOME/planrun` or `$HOME/.config/planrun` elsewhere (D96).

The layers stack in that order (D95): a user spec with a built-in format's id replaces that
format, and a repository spec replaces either, whole (no merging). `planrun check` then says
`ok (assay (.planrun/formats/assay.toml, overrides built-in), 2 tasks)`. To change a built-in
format for one repository, copy `formats/<id>.toml` to `.planrun/formats/` and edit the copy. A
spec that does not load stops every command with exit code 2 and names its file.

Where plan files are found is the repository's `.planrun/config.toml` (D96), optional:

```toml
[plans]
globs = ["docs/plans/*.md", "work/**/plans/*.md"]   # replaces the defaults
```

Without it the globs are `docs/plans/*.md`, `**/superpowers/**/plans/*.md` and `**/plans/*.md`.
A glob is a manifest glob (D41) plus `**`, a whole segment that matches any number of
directories. `planrun list`, `planrun tui` and plan names (`planrun run p7`, D97) use them.

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
| `Ui` | the CLI | `status`, `output`, `ask` (a `Question`: its `QuestionKind` with what the run found (D86), named `Choice`s, or the words a typed line may be), `view`, `diff`, `shell`; the runner decides what is asked and acts on the answer (D74). planrun-cli's `LineUi` is line prompts on std; planrun-tui's `ChannelUi` forwards to the TUI's render thread, `ask`, `view`, `diff` and `shell` as blocking round trips (D90); the runner tests script their answers; a non-interactive batch UI later. A new question is a new `QuestionKind` variant, and every UI's exhaustive match shows where it must be shown |
