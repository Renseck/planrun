//! The synthetic assay plan parses with every block accounted for: actions, the absorbed
//! ceremonies, the manifest, and Notes for the goal and the prose between blocks.

use std::path::Path;

use planrun_model::{ActionKind, ExitExpect, ScriptIntent, WriteMode};
use planrun_parse::{Registry, parse_plan};

fn parse() -> planrun_parse::Parsed {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/synthetic/assay-style.md");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, "assay-style.md", &bytes, None)
        .unwrap_or_else(|d| panic!("{:?}", d.error_codes()))
}

/// One short label per action.
fn label(kind: &ActionKind) -> String {
    match kind {
        ActionKind::WriteFile { path, mode, .. } => format!("write {path} {mode:?}"),
        ActionKind::ApplyPatch {
            path, pre, post, ..
        } => {
            let id = |i: &Option<planrun_model::AbbrevBlobId>| {
                i.map(|i| i.to_string()).unwrap_or_default()
            };
            format!("patch {path} {}..{}", id(pre), id(post))
        }
        ActionKind::Delete { path, .. } => format!("delete {path}"),
        ActionKind::Generated { path } => format!("generated {path}"),
        ActionKind::RunScript(s) => {
            let exit = s.expect.as_ref().map_or(ExitExpect::Unknown, |e| e.exit);
            format!("run {:?} {exit:?}", s.intent)
        }
        ActionKind::Commit(s) => {
            let exit = s.expect.as_ref().map_or(ExitExpect::Unknown, |e| e.exit);
            format!("commit {exit:?}")
        }
        ActionKind::Note { .. } => "note".to_owned(),
        ActionKind::Manual { reason, .. } => format!("manual {reason:?}"),
        ActionKind::ReplaceExact { path, .. } => format!("replace {path}"),
    }
}

#[test]
fn the_assay_plan_is_fully_recognized() {
    let parsed = parse();
    assert_eq!(&*parsed.plan.format.0, "assay");
    assert_eq!(parsed.plan.meta.phase.as_deref(), Some("P1"));
    assert!(parsed.diagnostics.0.is_empty(), "{:?}", parsed.diagnostics);

    let tasks: Vec<(String, Vec<Vec<String>>)> = parsed
        .plan
        .tasks
        .iter()
        .map(|t| {
            let steps = t
                .steps
                .iter()
                .map(|s| s.actions.iter().map(|a| label(&a.kind)).collect())
                .collect();
            (t.id.raw.to_string(), steps)
        })
        .collect();
    let strings = |items: &[&str]| items.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    assert_eq!(
        tasks,
        [
            (
                "P1-T1".to_owned(),
                vec![
                    strings(&["note"]),
                    strings(&[
                        "write src/greet.rs Create",
                        "patch src/lib.rs aa80cf1..97bd92a"
                    ]),
                    strings(&["note", "run Verify Success", "note", "run Gate Success"]),
                    strings(&["note", "commit Unknown"]),
                ]
            ),
            (
                "P1-T2".to_owned(),
                vec![
                    strings(&["note", "generated docs/modules.txt"]),
                    strings(&[
                        "patch src/lib.rs 97bd92a..6b269fc",
                        "delete src/old.rs",
                        "generated docs/modules.txt",
                    ]),
                    strings(&["note", "run Verify Success", "note", "run Gate Success"]),
                    strings(&["note", "commit Unknown"]),
                ]
            ),
        ]
    );
}

#[test]
fn coverage_has_no_manual_block_and_lists_the_absorbed_ceremonies() {
    let parsed = parse();
    for c in &parsed.coverage {
        assert_eq!(c.manual, 0, "{}", c.task);
        assert_eq!(c.absorbed.len(), 1, "{}", c.task);
        assert_eq!(&*c.absorbed[0].recognizer.0, "ceremony#git-apply-temp");
        assert_eq!(c.notes, 4, "{}", c.task);
    }
    let lines: Vec<u32> = parsed
        .coverage
        .iter()
        .map(|c| parsed.plan.source.line_col(c.absorbed[0].span.start).line)
        .collect();
    assert_eq!(lines, [74, 154]);
}

#[test]
fn write_modes_and_intents_come_from_the_plan() {
    let parsed = parse();
    let first = &parsed.plan.tasks[0].steps[1].actions[0];
    assert!(matches!(
        first.kind,
        ActionKind::WriteFile {
            mode: WriteMode::Create,
            ..
        }
    ));
    let commit = &parsed.plan.tasks[0].steps[3].actions[1];
    let ActionKind::Commit(script) = &commit.kind else {
        panic!("{:?}", commit.kind);
    };
    assert_eq!(script.intent, ScriptIntent::Other);
    let prose = script.expect.as_ref().and_then(|e| e.prose.as_ref());
    assert_eq!(
        prose.map(|p| parsed.plan.text(p)),
        Some("the commit, then no output from `git status --porcelain`.")
    );
}
