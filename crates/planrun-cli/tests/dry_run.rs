//! `planrun run --dry-run` on a temporary copy of the greeter fixture, and the roadmap's P3
//! acceptance cases on parsed plans: the dry run of `assay-style.md` passes and writes nothing;
//! a changed `src/lib.rs` fails preflight with the blob mismatch; a Create onto an existing file
//! fails before anything is written. The CLI has only `--dry-run` in P3, so the last two call
//! planrun-exec's preflight and prepare directly.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;

use std::path::Path;
use std::process::Command;

use fixture::Fixture;
use planrun_exec::HandlerSet;
use planrun_exec::handler::file_path;
use planrun_exec::preflight::{Outcome, dry_run, preflight};
use planrun_exec::transaction::prepare;
use planrun_exec::workspace::Workspace;
use planrun_model::{Action, Plan};
use planrun_parse::{Registry, parse_plan};

const PLAN: &str = "assay-style.md";

fn plan_text() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/corpus/synthetic")
        .join(PLAN);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn parse(name: &str, text: &str, format: Option<&str>) -> Plan {
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    parse_plan(&registry, name, text.as_bytes(), format)
        .unwrap_or_else(|d| panic!("{:?}", d.error_codes()))
        .plan
}

fn open(f: &Fixture) -> Workspace {
    Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
}

/// Runs `planrun run --dry-run` in the copy on a copy of the plan next to it (outside the
/// repository), so the plan's name in the output is the same on every machine.
fn planrun_dry_run(f: &Fixture) -> (i32, String) {
    let plan = f.outside().join(PLAN);
    std::fs::write(&plan, plan_text()).unwrap_or_else(|e| panic!("{e}"));
    let output = Command::new(env!("CARGO_BIN_EXE_planrun"))
        .args(["run", "--dry-run", "../outside/assay-style.md"])
        .current_dir(f.path())
        .envs(f.env())
        .output()
        .unwrap_or_else(|e| panic!("{e}"));
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), text)
}

#[test]
fn the_dry_run_of_the_assay_plan_passes() {
    let f = Fixture::greeter(false);
    let (code, out) = planrun_dry_run(&f);
    assert_eq!(code, 0, "{out}");
    insta::assert_snapshot!("dry_run_assay_style", out);
}

#[test]
fn the_dry_run_writes_nothing_with_autocrlf() {
    let f = Fixture::greeter(true);
    f.outside();
    let before = f.snapshot();
    let (code, out) = planrun_dry_run(&f);
    assert_eq!(code, 0, "{out}");
    let after = f.snapshot();
    let changed: Vec<&String> = before
        .keys()
        .chain(after.keys())
        .filter(|k| before.get(*k) != after.get(*k))
        .collect();
    assert!(changed.is_empty(), "the dry run changed {changed:?}");
}

#[test]
fn a_dry_run_that_fails_renders_the_finding_and_exits_1() {
    let f = Fixture::greeter(false);
    f.write("src/greet.rs", b"//! already here\n");
    f.git(&["add", "src/greet.rs"]);
    f.git(&["commit", "-q", "-m", "greet exists"]);
    let (code, out) = planrun_dry_run(&f);
    assert_eq!(code, 1, "{out}");
    insta::assert_snapshot!("dry_run_create_exists", out);
}

#[test]
fn a_changed_lib_rs_fails_preflight_with_the_blob_mismatch() {
    let f = Fixture::greeter(false);
    f.write("src/lib.rs", b"//! changed\n");
    f.git(&["commit", "-q", "-am", "changed"]);
    let plan = parse(PLAN, &plan_text(), None);
    let found = preflight(&open(&f), &plan, &plan.tasks[0]).unwrap_or_else(|e| panic!("{e}"));
    let codes: Vec<_> = found.iter().map(|f| f.code).collect();
    assert_eq!(codes, ["blob-mismatch"]);
}

#[test]
fn a_create_onto_an_existing_file_fails_before_anything_is_written() {
    let f = Fixture::greeter(false);
    f.write("src/greet.rs", b"//! already here\n");
    let before = f.snapshot();
    let plan = parse(PLAN, &plan_text(), None);
    let step: Vec<&Action> = plan.tasks[0]
        .steps
        .iter()
        .map(|s| {
            s.actions
                .iter()
                .filter(|a| file_path(&a.kind).is_some())
                .collect::<Vec<_>>()
        })
        .find(|actions| !actions.is_empty())
        .unwrap_or_default();
    assert_eq!(
        step.len(),
        2,
        "step 1 writes src/greet.rs and patches src/lib.rs"
    );
    let ws = open(&f);
    let err = prepare(&ws, &plan.source, &HandlerSet::files(), &step).err();
    assert_eq!(err.map(|f| f.code), Some("create-exists"));
    assert_eq!(f.snapshot(), before, "the tree or .git changed");
}

const DEVREADY: &str = "### Task 1: tools

**Files:** modify `tools/a.ps1` and `b.ps1`

**1a.** `b.ps1` (replaces the whole file):

```powershell
'b'
```
";

#[test]
fn an_overwrite_of_a_bare_entry_is_ambiguous_in_the_devready_format() {
    let f = Fixture::greeter(false);
    f.write("b.ps1", b"'old'\n");
    f.git(&["add", "b.ps1"]);
    f.git(&["commit", "-q", "-m", "a root b.ps1"]);
    let plan = parse("devready.md", DEVREADY, Some("devready"));
    let report = dry_run(&open(&f), &plan).unwrap_or_else(|e| panic!("{e}"));
    let outcomes: Vec<_> = report.tasks[0]
        .actions
        .iter()
        .filter_map(|a| match &a.outcome {
            Outcome::Failed(f) => Some(f.code),
            Outcome::Shown => None,
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(outcomes, ["path-ambiguous"]);
}

#[test]
fn run_without_dry_run_is_not_there_yet() {
    let f = Fixture::greeter(false);
    let output = Command::new(env!("CARGO_BIN_EXE_planrun"))
        .args(["run", "../outside/none.md"])
        .current_dir(f.path())
        .envs(f.env())
        .output()
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("P5"));
}
