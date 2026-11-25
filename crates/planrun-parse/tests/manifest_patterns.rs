//! Pattern manifest entries (decisions D41, D42): a range or a glob item is one entry that is
//! never a write target; `manifest-undeclared` and `manifest-untouched` match paths against it.

use planrun_model::{ChangeKind, ManifestPath};
use planrun_parse::{Registry, parse_plan};

const SPEC: &str = r#"
id = "t"
name = "manifest pattern test"
[detect]
patterns = [{ regex = '^### Task', weight = 60 }]
[outline]
task = { level = 3, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
[outline.manifest]
marker = '^\*\*Files:\*\*$'
item = '^(?P<change>Create|Modify): `(?P<paths>[^`]+(?:`(?:, | and |–)`[^`]+)*)`$'
changes = { Create = "create", Modify = "modify" }
path_list = '`(?:, | and )`'
siblings = true
[checks]
manifest-untouched = "warning"
manifest-undeclared = "warning"
[[recognizer]]
kind = "file-block"
id = "file-block#t"
lead = '^Replace `(?P<path>[^`]+)`:$'
ops = {}
default_op = "overwrite"
"#;

fn parse(text: &str) -> Result<planrun_parse::Parsed, Vec<&'static str>> {
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    registry
        .add_spec("t", SPEC)
        .unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, "t.md", text.as_bytes(), Some("t")).map_err(|d| d.error_codes())
}

fn task(files: &str, body: &str) -> String {
    format!("### Task 1: a\n\n**Files:**\n{files}\n\n{body}")
}

fn write(path: &str) -> String {
    format!("Replace `{path}`:\n\n```sql\nx\n```\n\n")
}

/// (path as written, is a pattern, change) per entry of the first task.
fn entries(parsed: &planrun_parse::Parsed) -> Vec<(String, bool, ChangeKind)> {
    parsed.plan.tasks[0]
        .manifest
        .entries
        .iter()
        .map(|e| {
            let pattern = matches!(e.path, ManifestPath::Pattern(_));
            (e.path.to_string(), pattern, e.change)
        })
        .collect()
}

fn warnings(parsed: &planrun_parse::Parsed) -> Vec<&str> {
    parsed.diagnostics.0.iter().map(|d| d.code).collect()
}

#[test]
fn a_range_item_is_one_pattern_entry_that_declares_its_files() {
    let text = task(
        "- Modify: `db/m/0001`–`0007_*.sql`",
        &write("db/m/0002_init.sql"),
    );
    let parsed = parse(&text).unwrap_or_else(|codes| panic!("{codes:?}"));
    let expected = [("db/m/0001–0007_*.sql".to_owned(), true, ChangeKind::Modify)];
    assert_eq!(entries(&parsed), expected);
    assert!(warnings(&parsed).is_empty(), "{:?}", warnings(&parsed));
}

#[test]
fn a_path_outside_the_range_is_undeclared_and_the_range_untouched() {
    let text = task(
        "- Modify: `db/m/0001`–`0007_*.sql`",
        &write("db/m/0008_late.sql"),
    );
    let parsed = parse(&text).unwrap_or_else(|codes| panic!("{codes:?}"));
    let mut found = warnings(&parsed);
    found.sort_unstable();
    assert_eq!(found, ["manifest-undeclared", "manifest-untouched"]);
}

#[test]
fn ranges_are_found_before_the_path_list_splits() {
    let text = task("- Modify: `db/a.cs`, `db/m/0001`–`0003.sql` and `b.cs`", "");
    let parsed = parse(&text).unwrap_or_else(|codes| panic!("{codes:?}"));
    let found: Vec<(String, bool)> = entries(&parsed)
        .into_iter()
        .map(|(p, pattern, _)| (p, pattern))
        .collect();
    let expected = [
        ("db/a.cs".to_owned(), false),
        ("db/m/0001–0003.sql".to_owned(), true),
        ("b.cs".to_owned(), false),
    ];
    assert_eq!(found, expected);
}

#[test]
fn a_glob_item_is_a_pattern_entry() {
    let text = task("- Modify: `src/{a,b}/*.rs`", &write("src/b/x.rs"));
    let parsed = parse(&text).unwrap_or_else(|codes| panic!("{codes:?}"));
    assert!(entries(&parsed)[0].1);
    assert!(warnings(&parsed).is_empty(), "{:?}", warnings(&parsed));
}

#[test]
fn broken_patterns_are_manifest_item_errors() {
    for files in [
        "- Modify: `db/0001`–`other/0007.sql`",
        "- Modify: `src/**/*.rs`",
        "- Modify: `src/{a,{b,c}}.rs`",
    ] {
        let codes = parse(&task(files, "")).err();
        assert_eq!(codes, Some(vec!["manifest-item"]), "{files}");
    }
}

#[test]
fn a_write_never_targets_a_wildcard() {
    let text = task("- Modify: `src/*.rs`", &write("src/*.rs"));
    assert_eq!(parse(&text).err(), Some(vec!["bad-path"]));
}
