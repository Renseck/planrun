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
