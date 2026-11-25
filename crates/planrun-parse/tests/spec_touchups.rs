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
