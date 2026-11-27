//! Resume on hand-built IR over a fixture copy (D14, D77): the journal checked against the
//! disk and HEAD. A crash is a panic from the journal's observer at a seam, so the journal and
//! the disk are what a killed process leaves.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[path = "support/ir.rs"]
mod ir;

use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

use fixture::Fixture;
use ir::{Ir, PATCH_T1, PATCH_T2};
use planrun_exec::journal::{self, End, Journal, Observer, Record, Seam};
use planrun_exec::policy::{Policy, PolicyClass};
use planrun_exec::runner::resume::analyse;
use planrun_exec::runner::{Finished, RunConfig, Runner};
use planrun_exec::shell::{OutLine, SessionConfig};
use planrun_exec::ui::{Choice, Position, Question, QuestionKind, Reply, Ui};
use planrun_exec::workspace::Workspace;
use planrun_model::{
    Action, ActionKind, ChangeKind, ExitExpect, Expectation, PathOrigin, Plan, ScriptIntent,
    WriteMode,
};

#[derive(Default)]
struct Scripted {
    answers: VecDeque<Reply>,
    asked: Vec<String>,
    kinds: Vec<QuestionKind>,
    shown: Vec<String>,
}

impl Ui for Scripted {
    fn status(&mut self, line: &str) {
        self.shown.push(line.to_owned());
    }
    fn output(&mut self, line: &OutLine) {
        self.shown.push(format!("| {}", line.text));
    }
    fn ask(&mut self, question: &Question) -> Reply {
        self.asked.push(question.kind.name().to_owned());
        self.kinds.push(question.kind.clone());
        self.shown.extend(question.text.iter().cloned());
        self.answers
            .pop_front()
            .unwrap_or_else(|| Reply::Stop(format!("no answer for `{}`", question.kind.name())))
    }
    fn view(&mut self, _at: Position) {}
    fn diff(&mut self) {}
    fn shell(&mut self) {}
}

fn choice(c: Choice) -> Reply {
    Reply::Choice(c)
}

const AUTO: &[PolicyClass] = &[PolicyClass::Read, PolicyClass::Build, PolicyClass::Modify];
const W: PathOrigin = PathOrigin::Written;

fn config(f: &Fixture, auto: &[PolicyClass]) -> RunConfig {
    RunConfig {
        plan_path: "test.md".to_owned(),
        policy: Policy::new(auto, false).unwrap_or_else(|e| panic!("{e}")),
        from: None,
        to: None,
        session: SessionConfig {
            root: f.path().to_path_buf(),
            env: f.env(),
            timeout: Some(Duration::from_secs(60)),
            temp: f.outside(),
        },
    }
}

/// A run (or, with `resume`, a resume) of `plan`; `observer` may crash it. `None` when it
/// crashed.
fn go(
    f: &Fixture,
    plan: &Plan,
    auto: &[PolicyClass],
    answers: Vec<Reply>,
    observer: Option<Observer>,
    resume: bool,
) -> (Option<Finished>, Scripted) {
    let ws = Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"));
    let path = f.outside().join("journal.jsonl");
    let lines = journal::read(&path).unwrap_or_else(|e| panic!("{e}"));
    let mut journal = Journal::open(&path).unwrap_or_else(|e| panic!("{e}"));
    if let Some(observer) = observer {
        journal.observe(observer);
    }
    let mut ui = Scripted {
        answers: answers.into(),
        ..Scripted::default()
    };
    let finished = catch_unwind(AssertUnwindSafe(|| {
        let mut runner = Runner::new(plan, &ws, &mut journal, &mut ui, config(f, auto));
        if resume {
            let analysis = analyse(&lines).unwrap_or_else(|| panic!("nothing to resume"));
            runner.resume(&analysis)
        } else {
            runner.run()
        }
    }));
    let finished = finished.ok().map(|r| r.unwrap_or_else(|e| panic!("{e}")));
    (finished, ui)
}

fn records(f: &Fixture) -> Vec<Record> {
    journal::read(&f.outside().join("journal.jsonl"))
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|l| l.record)
        .collect()
}

fn count(records: &[Record], name: &str) -> usize {
    records
        .iter()
        .filter(|r| {
            let json = serde_json::to_value(r).unwrap_or_else(|e| panic!("{e}"));
            json["t"] == name
        })
        .count()
}

fn subjects(f: &Fixture) -> Vec<String> {
    f.git(&["log", "--format=%s"])
        .lines()
        .map(str::to_owned)
        .collect()
}

fn gate(ir: &mut Ir, body: &str) -> Action {
    let mut action = ir.script(body);
    if let ActionKind::RunScript(s) = &mut action.kind {
        s.intent = ScriptIntent::Gate;
        s.expect = Some(Expectation {
            exit: ExitExpect::Success,
            lines: None,
            prose: None,
            comments: None,
        });
    }
    action
}

/// The two assay tasks by hand; T1's step 1 writes two files.
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
        vec![gate(ir, "$gate = 0")],
        vec![ir.commit(
            "if ($gate -eq 0) { git add -- src/greet.rs src/lib.rs; git commit -q -m 'P1-T1' }",
        )],
    ];
    let m2 = ir.manifest(&[
        ("src/lib.rs", ChangeKind::Modify, W),
        ("src/old.rs", ChangeKind::Delete, W),
    ]);
    let t2 = vec![
        vec![
            ir.patch("src/lib.rs", PATCH_T2, "97bd92a", "6b269fc"),
            ir.delete("src/old.rs", false),
        ],
        vec![gate(ir, "$gate = 0")],
        vec![ir.commit(
            "if ($gate -eq 0) { git add -- src/lib.rs src/old.rs; git commit -q -m 'P1-T2' }",
        )],
    ];
    let tasks = vec![ir.task("P1-T1", m1, t1), ir.task("P1-T2", m2, t2)];
    ir.plan(tasks)
}

/// An observer that panics at the first seam `crash` accepts.
fn crash_at(crash: impl Fn(&Seam<'_>) -> bool + 'static) -> Option<Observer> {
    Some(Box::new(move |seam| {
        if crash(&seam) {
            panic!("crash at {seam:?}");
        }
    }))
}

fn assert_done(f: &Fixture) {
    assert_eq!(subjects(f), ["P1-T2", "P1-T1", "fixture"]);
    assert_eq!(f.status(), "");
    let ls = f.git(&["ls-tree", "-r", "--name-only", "HEAD"]);
    assert_eq!(
        ls.lines().collect::<Vec<_>>(),
        [".gitignore", "Cargo.toml", "src/greet.rs", "src/lib.rs"]
    );
}

#[test]
fn a_run_stopped_at_the_commit_prompt_resumes_without_writing_again() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let read_build = &[PolicyClass::Read, PolicyClass::Build];
    let (first, ui) = go(
        &f,
        &plan,
        read_build,
        vec![choice(Choice::Quit)],
        None,
        false,
    );
    assert_eq!(first.map(|f| f.end), Some(End::Stopped));
    assert_eq!(ui.asked, ["run"]);
    let (second, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Continue)], None, true);
    assert_eq!(
        second.map(|f| f.end),
        Some(End::Completed),
        "{:?}",
        ui.shown
    );
    assert_eq!(ui.asked, ["session"]);
    assert!(
        ui.shown
            .iter()
            .any(|l| l.contains("done on disk, not written again"))
    );
    let all = records(&f);
    assert_eq!(
        count(&all, "step-prepared"),
        2,
        "T1 step 1 once, T2 step 1 once"
    );
    assert_eq!(count(&all, "step-written"), 2);
    assert_eq!(count(&all, "step-verified"), 1);
    assert_done(&f);
}

#[test]
fn a_crash_after_the_write_ahead_record_prepares_the_step_again_and_writes_it_once() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let prepared = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::StepPrepared { .. }));
    let (first, _) = go(&f, &plan, AUTO, vec![], crash_at(prepared), false);
    assert!(first.is_none(), "it crashed");
    assert!(f.read("src/greet.rs").is_none(), "nothing was written");
    let (second, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Continue)], None, true);
    assert_eq!(
        second.map(|f| f.end),
        Some(End::Completed),
        "{:?}",
        ui.shown
    );
    let all = records(&f);
    assert_eq!(count(&all, "step-prepared"), 3);
    assert_eq!(count(&all, "step-written"), 2);
    assert_done(&f);
}

#[test]
fn a_crash_between_two_writes_is_unknown_and_restore_completes_it() {
    let f = Fixture::greeter(true);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let first_write = |s: &Seam<'_>| matches!(s, Seam::Wrote(1));
    let (first, _) = go(&f, &plan, AUTO, vec![], crash_at(first_write), false);
    assert!(first.is_none());
    assert!(f.read("src/greet.rs").is_some(), "the first write happened");
    let answers = vec![choice(Choice::Restore), choice(Choice::Continue)];
    let (second, ui) = go(&f, &plan, AUTO, answers, None, true);
    assert_eq!(
        second.map(|f| f.end),
        Some(End::Completed),
        "{:?}",
        ui.shown
    );
    assert_eq!(ui.asked, ["unknown", "session"]);
    let QuestionKind::Unknown { step, .. } = &ui.kinds[0] else {
        panic!("{:?}", ui.kinds);
    };
    assert_eq!(
        (step.task.as_str(), step.step, step.action),
        ("P1-T1", 0, 0)
    );
    let all = records(&f);
    assert_eq!(count(&all, "step-restored"), 1);
    assert_done(&f);
}

#[test]
fn a_crash_after_the_commit_s_start_runs_the_commit() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let commit_start = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::ActionStart { kind, at, .. }) if kind == "commit" && at.task == "P1-T1");
    let (first, _) = go(&f, &plan, AUTO, vec![], crash_at(commit_start), false);
    assert!(first.is_none());
    assert_eq!(subjects(&f), ["fixture"]);
    let (second, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Continue)], None, true);
    assert_eq!(
        second.map(|f| f.end),
        Some(End::Completed),
        "{:?}",
        ui.shown
    );
    assert_done(&f);
}

#[test]
fn a_commit_made_before_its_end_was_journaled_is_done() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let root = f.path().to_path_buf();
    let env = f.env();
    let observer: Observer = Box::new(move |seam| {
        let Seam::Appended(Record::ActionStart { kind, at, .. }) = seam else {
            return;
        };
        if kind != "commit" || at.task != "P1-T1" {
            return;
        }
        for args in [
            &["add", "--", "src/greet.rs", "src/lib.rs"][..],
            &["commit", "-q", "-m", "P1-T1"],
        ] {
            let ok = std::process::Command::new("git")
                .args(args)
                .current_dir(&root)
                .envs(env.iter().map(|(k, v)| (k, v)))
                .status()
                .is_ok_and(|s| s.success());
            assert!(ok);
        }
        panic!("crash inside git commit");
    });
    let (first, _) = go(&f, &plan, AUTO, vec![], Some(observer), false);
    assert!(first.is_none());
    assert_eq!(subjects(&f), ["P1-T1", "fixture"]);
    let (second, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Continue)], None, true);
    assert_eq!(
        second.map(|f| f.end),
        Some(End::Completed),
        "{:?}",
        ui.shown
    );
    assert!(
        ui.shown
            .iter()
            .any(|l| l.contains("was made before the interruption"))
    );
    assert_done(&f);
}

#[test]
fn a_commit_found_made_by_resume_is_not_run_again_after_a_second_crash() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let root = f.path().to_path_buf();
    let env = f.env();
    let observer: Observer = Box::new(move |seam| {
        let Seam::Appended(Record::ActionStart { kind, at, .. }) = seam else {
            return;
        };
        if kind != "commit" || at.task != "P1-T1" {
            return;
        }
        git_in(&root, &env, &["add", "--", "src/greet.rs", "src/lib.rs"]);
        git_in(&root, &env, &["commit", "-q", "-m", "P1-T1"]);
        panic!("crash inside git commit");
    });
    let (first, _) = go(&f, &plan, AUTO, vec![], Some(observer), false);
    assert!(first.is_none());
    let verified = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::StepVerified { .. }));
    let (second, _) = go(&f, &plan, AUTO, vec![], crash_at(verified), true);
    assert!(second.is_none(), "it crashed after finding the commit made");
    let (third, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Continue)], None, true);
    assert_eq!(third.map(|f| f.end), Some(End::Completed), "{:?}", ui.shown);
    assert_eq!(ui.asked, ["session"]);
    assert_done(&f);
}

#[test]
fn resume_stops_on_another_head_a_changed_plan_or_an_index_lock() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let read_build = &[PolicyClass::Read, PolicyClass::Build];
    go(
        &f,
        &plan,
        read_build,
        vec![choice(Choice::Quit)],
        None,
        false,
    );
    std::fs::write(f.path().join(".git/index.lock"), "").unwrap_or_else(|e| panic!("{e}"));
    let (r, _) = go(&f, &plan, AUTO, vec![], None, true);
    let reason = r.and_then(|r| r.reason).unwrap_or_default();
    assert!(reason.contains("index.lock"), "{reason}");
    std::fs::remove_file(f.path().join(".git/index.lock")).unwrap_or_else(|e| panic!("{e}"));
    let mut changed = plan.clone();
    changed.source = planrun_model::PlanSource::from_normalized(
        "test.md",
        plan.source.text().to_owned(),
        "1".to_owned(),
    );
    let (r, _) = go(&f, &changed, AUTO, vec![], None, true);
    let reason = r.and_then(|r| r.reason).unwrap_or_default();
    assert!(reason.contains("fingerprint"), "{reason}");
    f.git(&["commit", "-q", "--allow-empty", "-m", "someone else"]);
    let (r, _) = go(&f, &plan, AUTO, vec![], None, true);
    let reason = r.and_then(|r| r.reason).unwrap_or_default();
    assert!(reason.contains("something else changed"), "{reason}");
    let (again, _) = go(&f, &plan, AUTO, vec![], None, true);
    let again = again.and_then(|r| r.reason).unwrap_or_default();
    assert_eq!(
        again, reason,
        "a stopped resume does not record the moved HEAD"
    );
}

/// A git command run in the fixture from an observer.
fn git_in(root: &std::path::Path, env: &[(std::ffi::OsString, std::ffi::OsString)], args: &[&str]) {
    let ok = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .envs(env.iter().map(|(k, v)| (k, v)))
        .status()
        .is_ok_and(|s| s.success());
    assert!(ok, "git {args:?}");
}

fn resume_runs(records: &[Record]) -> Vec<usize> {
    records
        .iter()
        .enumerate()
        .filter(|(_, r)| matches!(r, Record::Run { resume: true, .. }))
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn a_second_resume_after_a_foreign_commit_asks_again_and_never_commits() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let commit_start = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::ActionStart { kind, at, .. }) if kind == "commit" && at.task == "P1-T1");
    let (first, _) = go(&f, &plan, AUTO, vec![], crash_at(commit_start), false);
    assert!(first.is_none());
    f.git(&["commit", "-q", "--allow-empty", "-m", "someone else"]);
    let mut reasons = Vec::new();
    for _ in 0..2 {
        let (r, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Quit)], None, true);
        assert_eq!(ui.asked, ["head-moved"], "{:?}", ui.shown);
        reasons.push(r.and_then(|r| r.reason).unwrap_or_default());
    }
    assert_eq!(reasons[0], reasons[1]);
    let all = records(&f);
    let runs = resume_runs(&all);
    assert_eq!(runs.len(), 2);
    let after = &all[runs[0]..];
    assert_eq!(
        count(after, "action-start"),
        0,
        "nothing ran after the crash"
    );
    assert_eq!(subjects(&f), ["someone else", "fixture"]);
}

#[test]
fn a_kept_head_move_survives_a_crash_right_after_the_answer() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let root = f.path().to_path_buf();
    let env = f.env();
    let observer: Observer = Box::new(move |seam| {
        let Seam::Appended(Record::ActionStart { kind, at, .. }) = seam else {
            return;
        };
        if kind != "script" || at.task != "P1-T1" {
            return;
        }
        git_in(
            &root,
            &env,
            &["commit", "-q", "--allow-empty", "-m", "inside"],
        );
        panic!("crash inside the script that committed");
    });
    let (first, _) = go(&f, &plan, AUTO, vec![], Some(observer), false);
    assert!(first.is_none());
    // The first record after the keep's own: the replayed step's `step-verified`, before the
    // kept script's records.
    let kept = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::StepVerified { .. }));
    let (second, ui) = go(
        &f,
        &plan,
        AUTO,
        vec![choice(Choice::Keep)],
        crash_at(kept),
        true,
    );
    assert!(second.is_none(), "it crashed right after the keep");
    assert_eq!(ui.asked, ["head-moved"]);
    let (third, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Continue)], None, true);
    assert_eq!(third.map(|f| f.end), Some(End::Completed), "{:?}", ui.shown);
    assert_eq!(ui.asked, ["session"], "neither head-moved nor again");
    assert_eq!(subjects(&f), ["P1-T2", "P1-T1", "inside", "fixture"]);
    assert_eq!(f.status(), "");
}

#[test]
fn a_change_after_the_crash_to_a_path_a_remaining_step_writes_is_asked_about() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let m = ir.manifest(&[
        ("src/greet.rs", ChangeKind::Create, W),
        ("src/lib.rs", ChangeKind::Modify, W),
    ]);
    let steps = vec![
        vec![ir.write(
            "src/greet.rs",
            "pub fn greeting() {}
",
            WriteMode::Create,
        )],
        vec![ir.write(
            "src/lib.rs",
            "pub mod greet;
",
            WriteMode::Overwrite,
        )],
        vec![gate(&mut ir, "$gate = 0")],
        vec![ir.commit(
            "if ($gate -eq 0) { git add -- src/greet.rs src/lib.rs; git commit -q -m 'T1' }",
        )],
    ];
    let task = ir.task("T1", m, steps);
    let plan = ir.plan(vec![task]);
    let written = |s: &Seam<'_>| matches!(s, Seam::Appended(Record::StepWritten { .. }));
    let (first, _) = go(&f, &plan, AUTO, vec![], crash_at(written), false);
    assert!(first.is_none());
    std::fs::write(
        f.path().join("src/lib.rs"),
        "// mine
",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let (second, ui) = go(&f, &plan, AUTO, vec![choice(Choice::Quit)], None, true);
    assert_eq!(second.map(|f| f.end), Some(End::Stopped));
    assert_eq!(ui.asked, ["resume-dirt"], "asked although --auto covers it");
    assert!(
        ui.shown.iter().any(|l| l == "  src/lib.rs"),
        "{:?}",
        ui.shown
    );
    assert!(
        !ui.shown.iter().any(|l| l == "  src/greet.rs"),
        "a verified step's path is not asked about"
    );
    assert_eq!(
        f.read("src/lib.rs").as_deref(),
        Some(
            &b"// mine
"[..]
        )
    );
    let answers = vec![choice(Choice::Continue), choice(Choice::Continue)];
    let (third, ui) = go(&f, &plan, AUTO, answers, None, true);
    assert_eq!(third.map(|f| f.end), Some(End::Completed), "{:?}", ui.shown);
    assert_eq!(ui.asked, ["resume-dirt", "session"]);
    assert_eq!(
        f.read("src/lib.rs").as_deref(),
        Some(
            &b"pub mod greet;
"[..]
        )
    );
    assert_eq!(subjects(&f), ["T1", "fixture"]);
}

#[test]
fn a_modify_script_that_ran_before_asks_and_keep_leaves_it() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let m = ir.manifest(&[("a.txt", ChangeKind::Create, W)]);
    let steps = vec![
        vec![ir.script("Add-Content -Path a.txt -Value line")],
        vec![ir.script("git status --short")],
        vec![ir.manual("look at it by hand")],
    ];
    let task = ir.task("T1", m, steps);
    let plan = ir.plan(vec![task]);
    let (first, _) = go(&f, &plan, AUTO, vec![choice(Choice::Quit)], None, false);
    assert_eq!(first.map(|f| f.end), Some(End::Stopped));
    let answers = vec![
        choice(Choice::Keep),
        choice(Choice::Continue),
        choice(Choice::Continue),
    ];
    let (second, ui) = go(&f, &plan, AUTO, answers, None, true);
    assert_eq!(
        second.map(|f| f.end),
        Some(End::Completed),
        "{:?}",
        ui.shown
    );
    assert_eq!(ui.asked, ["again", "session", "manual"]);
    assert!(
        ui.shown
            .iter()
            .any(|l| l.contains("ran before the interruption; it runs again"))
    );
    assert_eq!(
        f.read("a.txt")
            .map(|b| String::from_utf8_lossy(&b).lines().count()),
        Some(1)
    );
}
