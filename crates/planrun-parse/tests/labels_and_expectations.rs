//! Step labels that stay in their step (decision D46), `Expected:` with a list (D47), comment
//! expectations and what extends them (D35, D48), and numbered edit series (D49).

use planrun_model::{ActionKind, ExitExpect};
use planrun_parse::{Registry, parse_plan};

const SPEC: &str = r#"
id = "t"
name = "labels and expectations test"
[detect]
patterns = [{ regex = '^### Task', weight = 60 }]
[outline]
task = { level = 3, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
[outline.manifest]
marker = '^\*\*Files:\*\* (?P<change>modify)$'
item = '^`(?P<paths>[^`]+)`$'
changes = { modify = "modify" }
required = false
[[outline.steps]]
block = "paragraph"
regex = '^\*\*(?P<title>\d+[a-z]\.[^*]*|Verify:)\*\*'
keep = true
[checks]
manifest-untouched = "off"
manifest-undeclared = "warning"
[[recognizer]]
kind = "before-after"
id = "before-after#path"
lead = '^(?:\*\*\d+[a-z]\.\*\* )?`(?P<path>[^`]+)`, edit (?P<n>\d+) \(old → new\):$'
[[recognizer]]
kind = "before-after"
id = "before-after#series"
lead = '^Edit (?P<n>\d+) \(old → new\):$'
series = '(?i)\bedit (?P<n>\d+) \(old → new\)'
infer_path = true
[[recognizer]]
kind = "script"
id = "script#t"
leads = ['^\*\*Verify:\*\*$']
first_block = true
line_expect = '\s#\s(?P<expect>\S.*)$'
[[recognizer]]
kind = "expected-block"
id = "expected-block#t"
lead = '^Expected \((?P<spec>[^)]+)\):$'
exit = [{ spec = '^exit code (?P<code>\d+)$', exit = "code" }]
[[recognizer]]
kind = "expected-prose"
id = "expected-prose#t"
lead = '^Expected: (?P<text>.+)$'
exit = [{ text = '(?i)\ball pass\b', exit = "success" }]
[[recognizer]]
kind = "expected-prose"
id = "expected-prose#list"
lead = '^Expected:$'
list = true
exit = [{ text = '(?i)\b0 failed\b', exit = "success" }]
"#;

fn registry(spec: &str) -> Result<Registry, String> {
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    registry.add_spec("t", spec).map_err(|e| e.to_string())?;
    Ok(registry)
}

fn parse(text: &str) -> Result<planrun_parse::Parsed, Vec<&'static str>> {
    let registry = registry(SPEC).unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, "t.md", text.as_bytes(), Some("t")).map_err(|d| d.error_codes())
}

fn ok(text: &str) -> planrun_parse::Parsed {
    parse(text).unwrap_or_else(|codes| panic!("{codes:?}"))
}

/// One label per action of the first task, by step.
fn labels(parsed: &planrun_parse::Parsed) -> Vec<Vec<String>> {
    let plan = &parsed.plan;
    plan.tasks[0]
        .steps
        .iter()
        .map(|s| {
            s.actions
                .iter()
                .map(|a| match &a.kind {
                    ActionKind::Note { .. } => "note".to_owned(),
                    ActionKind::Manual { reason, .. } => format!("manual {reason:?}"),
                    ActionKind::ReplaceExact { path, .. } => {
                        format!("replace {path} {:?}", a.path_origin)
                    }
                    ActionKind::RunScript(s) => {
                        let e = s.expect.as_ref();
                        let exit = e.map_or(ExitExpect::Unknown, |e| e.exit);
                        let comments = e.and_then(|e| e.comments.as_ref()).map(|c| plan.text(c));
                        let prose = e.and_then(|e| e.prose.as_ref()).map(|p| plan.text(p));
                        format!("run {exit:?} comments={comments:?} prose={prose:?}")
                    }
                    other => format!("{other:?}"),
                })
                .collect()
        })
        .collect()
}

fn fence(lang: &str, body: &str) -> String {
    format!("```{lang}\n{body}\n```\n\n")
}

fn task(body: &str) -> String {
    format!("### Task 1: a\n\n**Files:** modify\n- `a/x.ps1`\n- `a/y.ps1`\n\n{body}")
}

#[test]
fn a_kept_label_is_the_first_block_of_its_step() {
    let text = task(&format!(
        "**2a.** `a/x.ps1`, edit 1 (old → new):\n\n{}{}",
        fence("powershell", "old"),
        fence("powershell", "new")
    ));
    assert_eq!(
        labels(&ok(&text)),
        [vec!["replace a/x.ps1 Written".to_owned()]]
    );
}

#[test]
fn first_block_never_claims_the_fence_after_a_kept_label() {
    let text = task(&format!(
        "**2b.** Something else\n\n{}",
        fence("powershell", "rm x")
    ));
    assert_eq!(
        labels(&ok(&text)),
        [vec!["note".to_owned(), "manual Unrecognized".to_owned()]]
    );
}

#[test]
fn trailing_comments_become_a_comment_expectation() {
    let body = "# a whole-line comment\ngit status --short   # empty\ncargo build\n";
    let text = task(&format!("**Verify:**\n\n{}", fence("powershell", body)));
    let expected = "run Unknown comments=Some(\"git status --short   # empty\") prose=None";
    assert_eq!(labels(&ok(&text))[0], ["note", expected]);
}

#[test]
fn a_script_without_comment_lines_has_no_expectation() {
    let text = task(&format!(
        "**Verify:**\n\n{}",
        fence("powershell", "cargo build")
    ));
    assert_eq!(
        labels(&ok(&text))[0][1],
        "run Unknown comments=None prose=None"
    );
}

#[test]
fn an_expected_sentence_extends_the_comment_expectation() {
    let text = task(&format!(
        "**Verify:**\n\n{}Expected: all pass\n",
        fence("powershell", "cargo test   # 3 passed")
    ));
    let expected =
        "run Success comments=Some(\"cargo test   # 3 passed\") prose=Some(\"all pass\")";
    assert_eq!(labels(&ok(&text))[0][1], expected);
}

#[test]
fn an_expected_list_extends_it_too() {
    let text = task(&format!(
        "**Verify:**\n\n{}Expected:\n- `3 passed, 0 failed`\n- no warnings\n",
        fence("bash", "cargo test   # 3 passed")
    ));
    let found = &labels(&ok(&text))[0][1];
    assert!(
        found.starts_with("run Success comments=Some(\"cargo test   # 3 passed\") prose=Some(\"Expected:\\n- `3 passed"),
        "{found}"
    );
}

#[test]
fn a_second_expectation_is_still_expected_twice() {
    let two = task(&format!(
        "**Verify:**\n\n{}Expected: all pass\n\nExpected: all pass\n",
        fence("powershell", "cargo test   # 3 passed")
    ));
    assert_eq!(parse(&two).err(), Some(vec!["expected-twice"]));
    let block = task(&format!(
        "**Verify:**\n\n{}Expected (exit code 0):\n\n{}",
        fence("powershell", "cargo test   # 3 passed"),
        fence("text", "ok")
    ));
    assert_eq!(parse(&block).err(), Some(vec!["expected-twice"]));
}

#[test]
fn an_expected_list_takes_nested_prose_and_stops_at_a_shell_fence() {
    let text = task(&format!(
        "**Verify:**\n\n{}Expected:\n- one\n  - nested\n\n  more text\n\n- two\n\n  {}\nAfter.\n",
        fence("powershell", "cargo test"),
        "```powershell\n  cargo build\n  ```\n"
    ));
    let found = labels(&ok(&text));
    assert!(found[0][1].contains("- two"), "{found:?}");
    assert_eq!(found[0].len(), 4, "{found:?}");
    assert_eq!(found[0][3], "note");
    assert!(found[0][2].starts_with("manual Unrecognized"), "{found:?}");
}

#[test]
fn an_expected_list_lead_without_a_list_is_malformed() {
    let text = task(&format!(
        "**Verify:**\n\n{}Expected:\n\nAfter.\n",
        fence("powershell", "cargo test")
    ));
    assert_eq!(parse(&text).err(), Some(vec!["missing-block"]));
}

#[test]
fn an_expected_list_after_a_manual_joins_it() {
    let text = task(&format!(
        "Some prose:\n\n{}Expected:\n- it works\n",
        fence("powershell", "cargo test")
    ));
    assert_eq!(
        labels(&ok(&text)),
        [vec!["note".to_owned(), "manual Unrecognized".to_owned()]]
    );
}

fn edit(lead: &str) -> String {
    format!(
        "{lead}\n\n{}{}",
        fence("powershell", "old"),
        fence("powershell", "new")
    )
}

#[test]
fn a_numbered_series_continues_the_path_of_the_edit_before_it() {
    let text = task(&format!(
        "{}{}{}",
        edit("`a/y.ps1`, edit 1 (old → new):"),
        edit("Edit 2 (old → new):"),
        edit("Edit 3 (old → new):")
    ));
    let expected = [
        "replace a/y.ps1 Written",
        "replace a/y.ps1 PreviousEdit",
        "replace a/y.ps1 PreviousEdit",
    ];
    assert_eq!(labels(&ok(&text))[0], expected);
}

#[test]
fn anything_between_or_a_gap_in_the_numbers_breaks_the_series() {
    let between = task(&format!(
        "{}A note.\n\n{}",
        edit("`a/y.ps1`, edit 1 (old → new):"),
        edit("Edit 2 (old → new):")
    ));
    assert_eq!(
        labels(&ok(&between))[0],
        ["replace a/y.ps1 Written", "note", "manual ProseEdit"]
    );
    let gap = task(&format!(
        "{}{}",
        edit("`a/y.ps1`, edit 1 (old → new):"),
        edit("Edit 3 (old → new):")
    ));
    assert_eq!(
        labels(&ok(&gap))[0],
        ["replace a/y.ps1 Written", "manual ProseEdit"]
    );
}

#[test]
fn a_series_needs_a_lead_that_captures_its_number() {
    let spec = SPEC.replace(
        "lead = '^Edit (?P<n>\\d+) \\(old → new\\):$'",
        "lead = '^Edit \\d+ \\(old → new\\):$'",
    );
    assert_ne!(spec, SPEC);
    let err = registry(&spec).err().unwrap_or_default();
    assert!(err.contains("`series` needs a lead"), "{err}");
}
