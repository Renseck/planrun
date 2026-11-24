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
