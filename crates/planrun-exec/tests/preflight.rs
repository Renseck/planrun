//! Preflight and the dry run on hand-built IR (D2) over a fixture copy: scope (D50), Bare
//! targets, results that depend on a step that did not run (D55), and real dirt.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[path = "support/ir.rs"]
mod ir;

use fixture::Fixture;
use ir::{Ir, PATCH_T1, PATCH_T2};
use planrun_exec::preflight::{DryRun, Outcome, dry_run, preflight};
use planrun_exec::workspace::Workspace;
use planrun_model::{ChangeKind, Manifest, PathOrigin, Plan, WriteMode};

fn open(f: &Fixture) -> Workspace {
    Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
}

fn run(f: &Fixture, plan: &Plan) -> DryRun {
    dry_run(&open(f), plan).unwrap_or_else(|e| panic!("{e}"))
}

/// One label per action of task `t`: `ok`, `ask`, the code of a failure, `depends <code>`, …
fn labels(report: &DryRun, t: usize) -> Vec<String> {
    report.tasks[t]
        .actions
        .iter()
        .map(|a| match &a.outcome {
            Outcome::Ok(_) => "ok".to_owned(),
            Outcome::WouldAsk(_) => "ask".to_owned(),
            Outcome::Failed(f) => f.code.to_owned(),
            Outcome::Depends(f) => format!("depends {}", f.code),
            Outcome::NotRun => "not run".to_owned(),
            Outcome::NotChecked => "not checked".to_owned(),
            Outcome::Waits => "waits".to_owned(),
            Outcome::Shown => "note".to_owned(),
        })
        .collect()
}

const W: PathOrigin = PathOrigin::Written;

/// The two tasks of `assay-style.md`, by hand.
fn assay(ir: &mut Ir) -> Plan {
    let m1 = ir.manifest(&[
        ("src/greet.rs", ChangeKind::Create, W),
        ("src/lib.rs", ChangeKind::Modify, W),
    ]);
    let t1 = vec![
        vec![
            ir.write("src/greet.rs", "pub fn greeting() {}\n", WriteMode::Create),
            ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a"),
        ],
        vec![ir.script("cargo test")],
        vec![ir.commit("git commit")],
    ];
    let m2 = ir.manifest(&[
        ("src/lib.rs", ChangeKind::Modify, W),
        ("src/old.rs", ChangeKind::Delete, W),
        ("docs/modules.txt", ChangeKind::Create, W),
    ]);
    let t2 = vec![
        vec![
            ir.patch("src/lib.rs", PATCH_T2, "97bd92a", "6b269fc"),
            ir.delete("src/old.rs", false),
            ir.generated("docs/modules.txt"),
        ],
        vec![ir.script("cargo test")],
        vec![ir.commit("git commit")],
    ];
    let tasks = vec![ir.task("P1-T1", m1, t1), ir.task("P1-T2", m2, t2)];
    ir.plan(tasks)
}

#[test]
fn the_assay_plan_passes_and_writes_nothing() {
    for autocrlf in [false, true] {
        let f = Fixture::greeter(autocrlf);
        let before = f.snapshot();
        let plan = assay(&mut Ir::default());
        let report = run(&f, &plan);
        assert!(
            !report.has_errors(),
            "{:?}",
            report.findings().collect::<Vec<_>>()
        );
        assert_eq!(labels(&report, 0), ["ok", "ok", "not run", "not run"]);
        assert_eq!(
            labels(&report, 1),
            ["ok", "ok", "not checked", "not run", "not run"]
        );
        assert_eq!(f.snapshot(), before, "autocrlf={autocrlf}");
    }
}

#[test]
fn preflight_finds_the_blob_mismatch_of_a_changed_file() {
    let f = Fixture::greeter(false);
    f.write("src/lib.rs", b"//! changed\n");
    f.git(&["commit", "-q", "-am", "changed"]);
    let plan = assay(&mut Ir::default());
    let found = preflight(&open(&f), &plan, &plan.tasks[0]).unwrap_or_else(|e| panic!("{e}"));
    let codes: Vec<_> = found.iter().map(|f| f.code).collect();
    assert_eq!(codes, ["blob-mismatch"]);
}

#[test]
fn a_dirty_touched_path_is_an_error_and_other_dirt_a_warning() {
    let f = Fixture::greeter(false);
    f.write("src/lib.rs", b"//! changed\n");
    f.write("unrelated.txt", b"x\n");
    let plan = assay(&mut Ir::default());
    let report = run(&f, &plan);
    let start: Vec<_> = report
        .start
        .iter()
        .map(|f| (f.code, f.is_error()))
        .collect();
    assert_eq!(start, [("path-dirty", true), ("tree-dirty", false)]);
}

#[test]
fn an_unscoped_write_would_ask() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let declared = ir.manifest(&[("a.txt", ChangeKind::Create, W)]);
    let steps = vec![vec![
        ir.write("a.txt", "a\n", WriteMode::Create),
        ir.write("b.txt", "b\n", WriteMode::Create),
    ]];
    let without = vec![vec![ir.write("c.txt", "c\n", WriteMode::Create)]];
    let tasks = vec![
        ir.task("T1", declared, steps),
        ir.task("T2", Manifest::default(), without),
    ];
    let report = run(&f, &ir.plan(tasks));
    assert_eq!(labels(&report, 0), ["ok", "ask"]);
    assert_eq!(labels(&report, 1), ["ask"]);
    assert!(!report.has_errors());
}

#[test]
fn a_bare_target_is_ambiguous_for_an_overwrite_or_a_delete() {
    let f = Fixture::greeter(false);
    f.write("lib.rs", b"a root file that happens to share the name\n");
    f.git(&["add", "lib.rs"]);
    f.git(&["commit", "-q", "-m", "root lib.rs"]);
    let mut ir = Ir::default();
    let m = ir.manifest(&[
        ("src/greet.rs", ChangeKind::Modify, W),
        ("lib.rs", ChangeKind::Modify, PathOrigin::Bare),
        ("missing.rs", ChangeKind::Modify, PathOrigin::Bare),
    ]);
    let steps = vec![vec![
        ir.write("lib.rs", "x\n", WriteMode::Overwrite),
        ir.delete("lib.rs", false),
        ir.replace("lib.rs", "a root file", "a file"),
        ir.replace("missing.rs", "x", "y"),
    ]];
    let tasks = vec![ir.task("T1", m, steps)];
    let report = run(&f, &ir.plan(tasks));
    assert_eq!(
        labels(&report, 0),
        ["path-ambiguous", "path-ambiguous", "ask", "target-missing"]
    );
    let Outcome::Failed(missing) = &report.tasks[0].actions[3].outcome else {
        panic!("target-missing expected");
    };
    assert!(missing.message.contains("D45"), "{}", missing.message);
}

#[test]
fn after_a_step_that_did_not_run_a_failure_is_a_warning_and_its_path_unknown() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let steps = vec![
        vec![ir.replace("src/old.rs", "absent text", "x")],
        vec![ir.script("cargo fmt")],
        vec![
            ir.replace("src/lib.rs", "absent text", "x"),
            ir.replace("src/lib.rs", "pub mod old;", "pub mod older;"),
            ir.write("src/old.rs", "x\n", WriteMode::Overwrite),
        ],
    ];
    let tasks = vec![ir.task("T1", Manifest::default(), steps)];
    let report = run(&f, &ir.plan(tasks));
    assert_eq!(
        labels(&report, 0),
        [
            "before-not-unique",
            "not run",
            "depends before-not-unique",
            "depends unverified",
            "ask"
        ]
    );
    let findings: Vec<_> = report.findings().map(|f| (f.code, f.is_error())).collect();
    assert_eq!(
        findings,
        [
            ("before-not-unique", true),
            ("before-not-unique", false),
            ("unverified", false)
        ]
    );
}

#[test]
fn a_manual_counts_as_a_step_that_did_not_run() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let steps = vec![vec![
        ir.manual("add a function by hand"),
        ir.replace("src/lib.rs", "a function added by hand", "x"),
    ]];
    let tasks = vec![ir.task("T1", Manifest::default(), steps)];
    let report = run(&f, &ir.plan(tasks));
    assert_eq!(labels(&report, 0), ["waits", "depends before-not-unique"]);
    assert!(!report.has_errors());
}

#[test]
fn preflight_stops_at_the_first_step_that_does_not_run() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let steps = vec![
        vec![ir.write("a.txt", "a\n", WriteMode::Create)],
        vec![ir.script("cargo build")],
        vec![ir.write("src/lib.rs", "x\n", WriteMode::Create)],
    ];
    let tasks = vec![ir.task("T1", Manifest::default(), steps)];
    let plan = ir.plan(tasks);
    let found = preflight(&open(&f), &plan, &plan.tasks[0]).unwrap_or_else(|e| panic!("{e}"));
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn manifest_entries_are_checked_at_the_start_of_their_task() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let m = ir.manifest(&[
        ("src/lib.rs", ChangeKind::Create, W),
        ("src/none.rs", ChangeKind::Modify, W),
        ("src/old.rs", ChangeKind::Test, W),
        ("nowhere.rs", ChangeKind::Modify, PathOrigin::Bare),
    ]);
    let tasks = vec![ir.task("T1", m, vec![vec![]])];
    let report = run(&f, &ir.plan(tasks));
    let codes: Vec<_> = report.tasks[0].manifest.iter().map(|f| f.code).collect();
    assert_eq!(codes, ["create-exists", "target-missing"]);
}
