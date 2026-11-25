//! The manifest keys of a format spec (docs/extending.md §2, decisions D26, D27, D32): an
//! optional manifest, a marker that captures the change, sibling file names, `:lines` tokens,
//! `Generated` entries, and a write whose op contradicts the manifest.

use planrun_model::{ChangeKind, PathOrigin};
use planrun_parse::{Registry, parse_plan};

const SPEC: &str = r#"
id = "t"
name = "manifest schema test"
[detect]
patterns = [{ regex = '^### Task', weight = 60 }]
[outline]
task = { level = 3, regex = '^(?P<id>Task (?P<ord>\d+)): (?P<title>.+)$' }
[outline.manifest]
marker = '^\*\*Files:\*\*(?: (?P<change>create|modify))?$'
item = '^(?:(?P<change>Create|Modify|Generated): )?`(?P<paths>[^`]+(?:`(?:, | and )`[^`]+)*)`$'
changes = { Create = "create", Modify = "modify", Generated = "generated", create = "create", modify = "modify" }
path_list = '`(?:, | and )`'
required = false
siblings = true
[checks]
manifest-untouched = "off"
manifest-undeclared = "warning"
[[recognizer]]
kind = "file-block"
id = "file-block#t"
lead = '^Create `(?P<path>[^`]+)`:$'
ops = {}
default_op = "create"
"#;

fn registry(spec: &str) -> Registry {
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    registry
        .add_spec("t", spec)
        .unwrap_or_else(|e| panic!("{e}"));
    registry
}

fn parse(text: &str) -> Result<planrun_parse::Parsed, Vec<&'static str>> {
    parse_plan(&registry(SPEC), "t.md", text.as_bytes(), Some("t")).map_err(|d| d.error_codes())
}

/// One manifest entry: path, change, lines, origin.
type Entry = (String, ChangeKind, Option<(u32, u32)>, PathOrigin);

/// The first task's entries.
fn entries(text: &str) -> Vec<Entry> {
    let parsed = parse(text).unwrap_or_else(|codes| panic!("{codes:?}"));
    parsed.plan.tasks[0]
        .manifest
        .entries
        .iter()
        .map(|e| {
            let lines = e.lines.map(|l| (l.start, l.end));
            (e.path.to_string(), e.change, lines, e.origin)
        })
        .collect()
}

#[test]
fn a_task_may_have_no_manifest() {
    let parsed = parse("### Task 0: baseline\n\nNothing to write.\n")
        .unwrap_or_else(|codes| panic!("{codes:?}"));
    assert!(parsed.plan.tasks[0].manifest.span.is_none());
}

#[test]
fn the_marker_can_carry_the_change() {
    let text = "### Task 1: a\n\n**Files:** create\n- `src/a.rs`\n- `src/b.rs`\n\n\
                Create `src/a.rs`:\n\n```rust\na\n```\n\nCreate `src/b.rs`:\n\n```rust\nb\n```\n";
    let changes: Vec<ChangeKind> = entries(text).into_iter().map(|e| e.1).collect();
    assert_eq!(changes, [ChangeKind::Create, ChangeKind::Create]);
}

#[test]
fn siblings_and_line_tokens_follow_the_path_before_them() {
    let text = "### Task 1: a\n\n**Files:**\n\
                - Modify: `src/x/a.rs:3-4`, `:9` and `b.rs`\n\
                - Generated: `docs/out.txt`\n";
    assert_eq!(
        entries(text),
        [
            (
                "src/x/a.rs".to_owned(),
                ChangeKind::Modify,
                Some((3, 4)),
                PathOrigin::Written
            ),
            (
                "src/x/a.rs".to_owned(),
                ChangeKind::Modify,
                Some((9, 9)),
                PathOrigin::Written
            ),
            (
                "src/x/b.rs".to_owned(),
                ChangeKind::Modify,
                None,
                PathOrigin::Sibling
            ),
            (
                "docs/out.txt".to_owned(),
                ChangeKind::Generated,
                None,
                PathOrigin::Written
            ),
        ]
    );
}

#[test]
fn without_siblings_a_bare_name_is_bare() {
    let spec = SPEC.replace("siblings = true", "siblings = false");
    let text = "### Task 1: a\n\n**Files:**\n- Modify: `src/x/a.rs`, `Cargo.toml`\n";
    let parsed = parse_plan(&registry(&spec), "t.md", text.as_bytes(), Some("t"))
        .unwrap_or_else(|d| panic!("{:?}", d.error_codes()));
    let paths: Vec<String> = parsed.plan.tasks[0]
        .manifest
        .entries
        .iter()
        .map(|e| format!("{} {:?}", e.path, e.origin))
        .collect();
    assert_eq!(paths, ["src/x/a.rs Written", "Cargo.toml Bare"]);
}

#[test]
fn a_create_on_a_declared_modify_is_an_error() {
    let text = "### Task 1: a\n\n**Files:** modify\n- `src/a.rs`\n\n\
                Create `src/a.rs`:\n\n```rust\na\n```\n";
    assert_eq!(parse(text).err(), Some(vec!["manifest-write-mode"]));
}

#[test]
fn an_item_needs_a_change_from_itself_or_the_marker() {
    let text = "### Task 1: a\n\n**Files:**\n- `src/a.rs`\n";
    assert_eq!(parse(text).err(), Some(vec!["manifest-item"]));
    let no_change = SPEC.replace("(?: (?P<change>create|modify))?", "");
    let no_change = no_change.replace("(?:(?P<change>Create|Modify|Generated): )?", "");
    let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let refused = registry.add_spec("t", &no_change);
    assert!(
        refused.is_err_and(|e| e.to_string().contains("no `change` capture")),
        "a spec with no change capture loads"
    );
}

#[test]
fn a_spec_downgrades_only_the_manifest_coverage_checks() {
    let text = "### Task 1: a\n\n**Files:** create\n- `src/a.rs`\n\n\
                Create `src/b.rs`:\n\n```rust\nb\n```\n";
    let parsed = parse(text).unwrap_or_else(|codes| panic!("{codes:?}"));
    let warnings: Vec<&str> = parsed.diagnostics.0.iter().map(|d| d.code).collect();
    assert_eq!(warnings, ["manifest-undeclared"]);

    for bad in [
        "manifest-write-mode = \"off\"",
        "blob-chain = \"warning\"",
        "manifest-untouched = \"ignore\"",
    ] {
        let spec = SPEC.replace("manifest-untouched = \"off\"", bad);
        let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
        assert!(registry.add_spec("t", &spec).is_err(), "{bad} loads");
    }
}
