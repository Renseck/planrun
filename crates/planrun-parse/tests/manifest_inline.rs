//! Inline Files clauses (decision D43), a change per path (D44) and bare file names without
//! siblings (D45).

use planrun_model::{ActionKind, ChangeKind, PathOrigin};
use planrun_parse::{Registry, parse_plan};

const SPEC: &str = r#"
id = "t"
name = "inline manifest test"
[detect]
patterns = [{ regex = '^### Task', weight = 60 }]
[outline]
task = { level = 3, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
[outline.manifest]
marker = '^\*\*Files:\*\*(?:$| (?P<items>(?:[^`.\n]|`[^`]*`)+))'
item = '^(?P<change>Create|Modify|create|modify)(?: \([^)]*\))?:? `(?P<paths>[^`]+(?:`(?: \([^)]*\))?(?:, | and )`[^`]+)*)`(?: \(.+\))?$'
changes = { Create = "create", Modify = "modify", create = "create", modify = "modify", "full replacement" = "rewrite" }
path_list = '`(?: \([^)]*\))?(?:, | and )`'
clause_list = '; '
path_change = '^` \((?P<change>full replacement)\)'
required = false
[settings]
manual_markers = ['(?i)\bby hand\b']
[checks]
manifest-untouched = "off"
manifest-undeclared = "warning"
[[recognizer]]
kind = "file-block"
id = "file-block#t"
lead = '^`(?P<path>[^`]+)`(?: \((?P<op>replaces the whole file)\))?:$'
ops = { "replaces the whole file" = "overwrite" }
default_op = "create"
[[recognizer]]
kind = "before-after"
id = "before-after#t"
lead = '^`(?P<path>[^`]+)` edit:$'
[[recognizer]]
kind = "before-after"
id = "before-after#no-path"
lead = '^Edit:$'
infer_path = true
"#;

fn parse(text: &str) -> Result<planrun_parse::Parsed, Vec<&'static str>> {
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    registry
        .add_spec("t", SPEC)
        .unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, "t.md", text.as_bytes(), Some("t")).map_err(|d| d.error_codes())
}

fn ok(text: &str) -> planrun_parse::Parsed {
    parse(text).unwrap_or_else(|codes| panic!("{codes:?}"))
}

/// (path, change, origin) per entry of the first task.
fn entries(parsed: &planrun_parse::Parsed) -> Vec<(String, ChangeKind, PathOrigin)> {
    parsed.plan.tasks[0]
        .manifest
        .entries
        .iter()
        .map(|e| (e.path.to_string(), e.change, e.origin))
        .collect()
}

/// The actions of the first task, as the text of each Note and the kind of everything else.
fn actions(parsed: &planrun_parse::Parsed) -> Vec<String> {
    let plan = &parsed.plan;
    plan.tasks[0]
        .steps
        .iter()
        .flat_map(|s| s.actions.iter())
        .map(|a| match &a.kind {
            ActionKind::Note { text } => format!("note {}", plan.text(text)),
            ActionKind::Manual { reason, text } => format!("manual {reason:?} {}", plan.text(text)),
            ActionKind::WriteFile { path, mode, .. } => format!("write {path} {mode:?}"),
            ActionKind::ReplaceExact { path, .. } => format!("replace {path}"),
            other => format!("{other:?}"),
        })
        .collect()
}

fn fence(body: &str) -> String {
    format!("```text\n{body}\n```\n\n")
}

#[test]
fn clauses_split_on_the_clause_list() {
    let parsed = ok("### Task 1: a\n\n**Files:** create `a/x.ps1`; modify `a/y.ps1`.\n");
    let expected = [
        (
            "a/x.ps1".to_owned(),
            ChangeKind::Create,
            PathOrigin::Written,
        ),
        (
            "a/y.ps1".to_owned(),
            ChangeKind::Modify,
            PathOrigin::Written,
        ),
    ];
    assert_eq!(entries(&parsed), expected);
    assert!(actions(&parsed).is_empty(), "{:?}", actions(&parsed));
}

#[test]
fn text_that_declares_nothing_is_a_note_at_the_start_of_the_task() {
    let text = "### Task 1: a\n\n**Files:** modify `a/x.ps1`; regenerate the README examples.\n\
                Then run it by hand.\n\nMore prose.\n";
    let parsed = ok(text);
    assert_eq!(entries(&parsed).len(), 1);
    let expected = [
        "note regenerate the README examples",
        "manual Marked Then run it by hand.",
        "note More prose.",
    ];
    assert_eq!(actions(&parsed), expected);
    let coverage = &parsed.coverage[0];
    assert_eq!((coverage.manifest, coverage.notes), (1, 1));
}

#[test]
fn a_task_with_only_an_inline_manifest_still_shows_its_leftover() {
    let parsed = ok("### Task 1: a\n\n**Files:** create `a/x.ps1`. It generates `a/y.json`.\n");
    assert_eq!(actions(&parsed), ["note It generates `a/y.json`."]);
}

#[test]
fn an_inline_manifest_takes_no_list() {
    let text = "### Task 1: a\n\n**Files:** create `a/x.ps1`\n- `a/y.ps1`\n";
    assert_eq!(parse(text).err(), Some(vec!["manifest-item"]));
}

#[test]
fn a_clause_with_a_code_span_must_be_an_item() {
    let text = "### Task 1: a\n\n**Files:** create `a/x.ps1`; rename `a/y.ps1`.\n";
    assert_eq!(parse(text).err(), Some(vec!["manifest-item"]));
}

#[test]
fn a_note_after_one_path_changes_only_that_path() {
    let text = format!(
        "### Task 1: a\n\n**Files:**\n\
         - Create (Write tool): `lib/base.ps1` (full replacement), `lib/one.ps1`, `lib/two.ps1`\n\n\
         `lib/base.ps1` (replaces the whole file):\n\n{}`lib/one.ps1`:\n\n{}",
        fence("a"),
        fence("b")
    );
    let parsed = ok(&text);
    let changes: Vec<ChangeKind> = entries(&parsed).into_iter().map(|e| e.1).collect();
    assert_eq!(
        changes,
        [ChangeKind::Rewrite, ChangeKind::Create, ChangeKind::Create]
    );
    let writes = ["write lib/base.ps1 Overwrite", "write lib/one.ps1 Create"];
    assert_eq!(actions(&parsed), writes);
    assert!(parsed.diagnostics.0.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn without_siblings_a_bare_name_after_a_directory_is_bare() {
    let parsed = ok("### Task 1: a\n\n**Files:** modify `src/x/a.rs` and `README.md`\n");
    let origins: Vec<PathOrigin> = entries(&parsed).into_iter().map(|e| e.2).collect();
    assert_eq!(origins, [PathOrigin::Written, PathOrigin::Bare]);
    let first = ok("### Task 1: a\n\n**Files:** modify `README.md` and `src/x/a.rs`\n");
    assert_eq!(entries(&first)[0].2, PathOrigin::Written);
}

#[test]
fn a_create_on_a_bare_entry_is_manual() {
    let text = format!(
        "### Task 1: a\n\n**Files:** create `src/x/a.rs`, `notes.md`\n\n`notes.md`:\n\n{}",
        fence("n")
    );
    let found = actions(&ok(&text));
    assert_eq!(found.len(), 1);
    assert!(found[0].starts_with("manual AmbiguousPath"), "{found:?}");
}

#[test]
fn an_edit_of_a_bare_entry_stays_an_action() {
    let text = format!(
        "### Task 1: a\n\n**Files:** modify `src/x/a.rs` and `README.md`\n\n`README.md` edit:\n\n{}{}",
        fence("old"),
        fence("new")
    );
    assert_eq!(actions(&ok(&text)), ["replace README.md"]);
}

#[test]
fn a_bare_entry_makes_an_edit_without_a_path_ambiguous() {
    let text = format!(
        "### Task 1: a\n\n**Files:** modify `src/x/a.rs` and `README.md`\n\nEdit:\n\n{}{}",
        fence("old"),
        fence("new")
    );
    let found = actions(&ok(&text));
    assert!(found[0].starts_with("manual ProseEdit"), "{found:?}");
    let only_bare = format!(
        "### Task 1: a\n\n**Files:** create `src/x/a.rs`; modify `x/b.rs`, `c.rs`\n\nEdit:\n\n{}{}",
        fence("old"),
        fence("new")
    );
    let found = actions(&ok(&only_bare));
    assert!(found[0].starts_with("manual ProseEdit"), "{found:?}");
}

#[test]
fn the_mode_check_still_sees_the_path_change() {
    let text = format!(
        "### Task 1: a\n\n**Files:**\n- Create: `h/a.ps1`, `h/b.ps1` (full replacement)\n\n\
         `h/b.ps1`:\n\n{}",
        fence("b")
    );
    assert_eq!(parse(&text).err(), Some(vec!["manifest-write-mode"]));
}
