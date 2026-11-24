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

[outline.manifest]                                       # optional; when set, every task has one
# The marker and its items are consumed by the outline: they fall in no step.
marker = '^\*\*Files\*\*$'
item = '^(?P<change>Create|Modify|Delete) `(?P<paths>[^`]+)`$'
changes = { Create = "create", Modify = "modify", Delete = "delete" }   # → create|modify|rewrite|delete|test
path_list = ', '                                         # optional: split `paths` into several

[[outline.steps]]                                        # zero or more; none = one implicit step
block = "paragraph"                                      # paragraph | list-item | heading
regex = '^\*\*Step (?P<n>\d+) — (?P<title>[^*]+)\*\*$'   # n and title optional

[settings]
manual_markers = ['(?i)\bby hand\b']                     # paragraph → Manual(Marked)
pwsh = ["powershell", "pwsh", "ps1"]                     # optional; these are the defaults
bash = ["bash", "sh", "shell"]

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