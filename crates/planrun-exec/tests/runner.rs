//! The runner on hand-built IR (D2) over a fixture copy, with scripted answers: the two assay
//! tasks end to end, what each decision and verdict asks, the commit gate (D75), dead sessions
//! (D81), and the checks around a commit (D80).

#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[path = "support/ir.rs"]
mod ir;

use std::collections::VecDeque;
use std::time::Duration;

use fixture::Fixture;
use ir::{Ir, PATCH_T1, PATCH_T2};
use planrun_exec::journal::{self, End, Journal, Outcome, Record};
use planrun_exec::policy::{Policy, PolicyClass};
use planrun_exec::runner::{Finished, RunConfig, Runner};
use planrun_exec::shell::{OutLine, RunStatus, SessionConfig};
use planrun_exec::ui::{
    Choice, Position, Question, QuestionKind, RUN_SKIP_OR_QUIT, Reply, Ui, YES_OR_QUIT,
};
use planrun_exec::workspace::Workspace;
use planrun_model::{
    Action, ActionKind, ChangeKind, ExitExpect, Expectation, PathOrigin, Plan, ScriptIntent, Shell,
    WriteMode,
};

/// Answers from a queue; what was asked and shown is kept.
#[derive(Default)]
struct Scripted {
    answers: VecDeque<Reply>,
    asked: Vec<String>,
    kinds: Vec<QuestionKind>,
    words: Vec<Option<&'static [&'static str]>>,
    shown: Vec<String>,
}

impl Scripted {
    fn new(answers: Vec<Reply>) -> Scripted {
        Scripted {
            answers: answers.into(),
            ..Scripted::default()
        }
    }
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
        self.words.push(question.typed);
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

fn typed(s: &str) -> Reply {
    Reply::Typed(s.to_owned())
}

const AUTO: &[PolicyClass] = &[PolicyClass::Read, PolicyClass::Build, PolicyClass::Modify];

/// Runs `plan` on `f` with `auto` and `answers`; returns how it ended, the UI and the records.
fn run(
    f: &Fixture,
    plan: &Plan,
    auto: &[PolicyClass],
    answers: Vec<Reply>,
) -> (Finished, Scripted, Vec<Record>) {
    let ws = Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"));
    let path = f.outside().join("journal.jsonl");
    let mut journal = Journal::open(&path).unwrap_or_else(|e| panic!("{e}"));
    let mut ui = Scripted::new(answers);
    let config = RunConfig {
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
    };
    let finished = Runner::new(plan, &ws, &mut journal, &mut ui, config)
        .run()
        .unwrap_or_else(|e| panic!("{e}"));
    let records = journal::read(&path)
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|l| l.record)
        .collect();
    (finished, ui, records)
}

fn expecting(
    mut action: Action,
    exit: ExitExpect,
    intent: ScriptIntent,
    prose: bool,
    ir: &mut Ir,
) -> Action {
    let prose = prose.then(|| planrun_model::TextRef::Span(ir.text("all pass")));
    if let ActionKind::RunScript(s) | ActionKind::Commit(s) = &mut action.kind {
        s.intent = intent;
        s.expect = Some(Expectation {
            exit,
            lines: None,
            prose,
            comments: None,
        });
    }
    action
}

fn bash(mut action: Action) -> Action {
    if let ActionKind::RunScript(s) | ActionKind::Commit(s) = &mut action.kind {
        s.shell = Shell::Bash;
    }
    action
}

const W: PathOrigin = PathOrigin::Written;

/// The two tasks of `assay-style.md` by hand, with scripts that need no cargo.
fn assay(ir: &mut Ir) -> Plan {
    let m1 = ir.manifest(&[
        ("src/greet.rs", ChangeKind::Create, W),
        ("src/lib.rs", ChangeKind::Modify, W),
    ]);
    let gate1 = ir.script("$gate = 0; 'gate exit code: 0'");
    let gate1 = expecting(gate1, ExitExpect::Success, ScriptIntent::Gate, false, ir);
    let t1 = vec![
        vec![
            ir.write("src/greet.rs", "pub fn greeting() {}\n", WriteMode::Create),
            ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a"),
        ],
        vec![gate1],
        vec![ir.commit(
            "if ($gate -eq 0) { git add -- src/greet.rs src/lib.rs; git commit -q -m 'P1-T1' }",
        )],
    ];
    let m2 = ir.manifest(&[
        ("src/lib.rs", ChangeKind::Modify, W),
        ("src/old.rs", ChangeKind::Delete, W),
        ("docs/modules.txt", ChangeKind::Create, W),
    ]);
    let gate2 = ir.script("$gate = 0");
    let gate2 = expecting(gate2, ExitExpect::Success, ScriptIntent::Gate, false, ir);
    let t2 = vec![
        vec![
            ir.patch("src/lib.rs", PATCH_T2, "97bd92a", "6b269fc"),
            ir.delete("src/old.rs", false),
            ir.generated("docs/modules.txt"),
        ],
        vec![
            ir.script("New-Item -ItemType Directory -Force docs | Out-Null; Set-Content -NoNewline -Path docs/modules.txt -Value \"greet.rs`nlib.rs`n\""),
            gate2,
        ],
        vec![ir.commit(
            "if ($gate -eq 0) { git add -- src/lib.rs src/old.rs docs/modules.txt; git commit -q -m 'P1-T2' }",
        )],
    ];
    let tasks = vec![ir.task("P1-T1", m1, t1), ir.task("P1-T2", m2, t2)];
    ir.plan(tasks)
}

/// A plan of one task, `T1`.
fn one(ir: &mut Ir, manifest: planrun_model::Manifest, steps: Vec<Vec<Action>>) -> Plan {
    let task = ir.task("T1", manifest, steps);
    ir.plan(vec![task])
}

fn kinds(records: &[Record]) -> Vec<String> {
    records
        .iter()
        .map(|r| {
            let json = serde_json::to_value(r).unwrap_or_else(|e| panic!("{e}"));
            json["t"].as_str().unwrap_or("?").to_owned()
        })
        .collect()
}

fn subjects(f: &Fixture) -> Vec<String> {
    f.git(&["log", "--format=%s"])
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_assay_tasks_run_to_two_commits_under_auto() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let (finished, ui, records) = run(&f, &plan, AUTO, vec![]);
    assert_eq!(
        finished.end,
        End::Completed,
        "{:?}\n{}",
        finished.reason,
        ui.shown.join("\n")
    );
    assert!(ui.asked.is_empty(), "{:?}", ui.asked);
    assert_eq!(subjects(&f), ["P1-T2", "P1-T1", "fixture"]);
    assert_eq!(f.status(), "");
    let ls = f.git(&["ls-tree", "-r", "--name-only", "HEAD"]);
    assert_eq!(
        ls.lines().collect::<Vec<_>>(),
        [
            ".gitignore",
            "Cargo.toml",
            "docs/modules.txt",
            "src/greet.rs",
            "src/lib.rs"
        ]
    );
    let k = kinds(&records);
    let first = |name: &str| k.iter().position(|x| x == name).unwrap_or(usize::MAX);
    assert_eq!(k.first().map(String::as_str), Some("run"));
    assert_eq!(k.last().map(String::as_str), Some("run-end"));
    assert!(first("step-prepared") < first("step-written"));
    assert_eq!(k.iter().filter(|x| *x == "step-prepared").count(), 2);
    assert_eq!(k.iter().filter(|x| *x == "task-end").count(), 2);
    assert!(k.contains(&"session".to_owned()));
    let generated = records.iter().any(|r| {
        matches!(
            r,
            Record::ActionEnd {
                outcome: Outcome::Present,
                ..
            }
        )
    });
    assert!(generated, "the generated file was checked");
}

#[test]
fn without_auto_every_script_asks_and_a_skipped_gate_closes_the_gate() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let plan = assay(&mut ir);
    let answers = vec![choice(Choice::Skip), typed("no"), typed("quit")];
    let (finished, ui, records) = run(&f, &plan, &[], answers);
    assert_eq!(finished.end, End::Stopped);
    assert_eq!(ui.asked, ["run", "gate", "gate"]);
    assert!(
        ui.shown.iter().any(|l| l.contains("was skipped by you")),
        "{:?}",
        ui.shown
    );
    assert_eq!(subjects(&f), ["fixture"]);
    let skipped = records.iter().any(|r| {
        matches!(
            r,
            Record::ActionEnd {
                outcome: Outcome::SkippedUser,
                ..
            }
        )
    });
    assert!(skipped);
}

#[test]
fn a_closed_gate_opens_only_with_a_typed_yes_and_is_journaled() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let failing = ir.script("$gate = 0; git rev-parse -q --verify refs/heads/none");
    let failing = expecting(
        failing,
        ExitExpect::Success,
        ScriptIntent::Gate,
        false,
        &mut ir,
    );
    let manifest = ir.manifest(&[("a.txt", ChangeKind::Create, W)]);
    let steps = vec![
        vec![ir.write("a.txt", "a\n", WriteMode::Create)],
        vec![failing],
        vec![ir.commit("git add -- a.txt; git commit -q -m 'T1'")],
    ];
    let plan = one(&mut ir, manifest, steps);
    let answers = vec![
        choice(Choice::Run),
        choice(Choice::Accept),
        typed("Yes"),
        typed("yes"),
        choice(Choice::Run),
    ];
    let (finished, ui, records) = run(&f, &plan, &[], answers);
    assert_eq!(
        finished.end,
        End::Completed,
        "{:?} {:?}",
        finished.reason,
        ui.shown
    );
    assert_eq!(ui.asked, ["run", "mismatch", "gate", "gate", "run"]);
    let override_reasons = records.iter().find_map(|r| match r {
        Record::GateOverride { reasons, .. } => Some(reasons.clone()),
        _ => None,
    });
    let reasons = override_reasons.unwrap_or_else(|| panic!("no gate-override"));
    assert_eq!(reasons.len(), 1);
    assert!(reasons[0].contains("accepted"), "{reasons:?}");
    assert_eq!(subjects(&f), ["T1", "fixture"]);
}

#[test]
fn prose_confirms_and_no_closes_the_gate() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let verify = ir.script("'all pass'");
    let verify = expecting(
        verify,
        ExitExpect::Success,
        ScriptIntent::Verify,
        true,
        &mut ir,
    );
    let steps = vec![
        vec![verify],
        vec![ir.commit("git commit -q --allow-empty -m 'T1'")],
    ];
    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
    let answers = vec![choice(Choice::No), typed("quit")];
    let (finished, ui, _) = run(&f, &plan, AUTO, answers);
    assert_eq!(finished.end, End::Stopped);
    assert_eq!(ui.asked, ["confirm", "gate"]);
    assert!(
        ui.shown.iter().any(|l| l.contains("all pass")),
        "{:?}",
        ui.shown
    );
}

#[test]
fn a_commit_that_does_not_move_head_is_a_mismatch() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let steps = vec![vec![ir.commit("'nothing committed'")]];
    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
    let (finished, ui, _) = run(&f, &plan, AUTO, vec![choice(Choice::Quit)]);
    assert_eq!(finished.end, End::Stopped);
    assert_eq!(ui.asked, ["mismatch"]);
    assert!(
        ui.shown.iter().any(|l| l.contains("HEAD did not move")),
        "{:?}",
        ui.shown
    );
}

#[test]
fn questions_carry_what_the_run_found() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let lines = planrun_model::TextRef::Span(ir.text("ok: …\nmissing\n"));
    let mut check = ir.script("'ok: 3'; 'done'");
    if let ActionKind::RunScript(s) = &mut check.kind {
        s.expect = Some(Expectation {
            exit: ExitExpect::Success,
            lines: Some(lines),
            prose: None,
            comments: None,
        });
    }
    let plan = one(
        &mut ir,
        planrun_model::Manifest::default(),
        vec![vec![check]],
    );
    let (_, ui, _) = run(&f, &plan, AUTO, vec![choice(Choice::Quit)]);
    let [
        QuestionKind::Mismatch {
            status,
            mismatches,
            output,
            aligned,
            commit,
            dead,
        },
    ] = &ui.kinds[..]
    else {
        panic!("{:?}", ui.kinds);
    };
    assert_eq!(*status, RunStatus::Exited(0));
    assert_eq!(mismatches.len(), 1, "{mismatches:?}");
    let shown: Vec<(usize, Option<&str>)> = aligned
        .iter()
        .map(|a| {
            (
                a.number,
                a.output.map(|i| output[i].text.trim_end_matches('\r')),
            )
        })
        .collect();
    assert_eq!(shown, [(1, Some("ok: 3")), (2, None)]);
    assert_eq!((commit, *dead), (&None, false));

    let f = Fixture::greeter(false);
    f.write("notes.txt", b"mine\n");
    let mut ir = Ir::default();
    let steps = vec![vec![ir.commit("'nothing committed'")]];
    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
    let answers = vec![choice(Choice::Continue), choice(Choice::Quit)];
    let (_, ui, _) = run(&f, &plan, AUTO, answers);
    let [
        QuestionKind::Dirt { paths },
        QuestionKind::Mismatch { commit, .. },
    ] = &ui.kinds[..]
    else {
        panic!("{:?}", ui.kinds);
    };
    let dirty: Vec<(&str, &str)> = paths
        .iter()
        .map(|e| (e.code.as_str(), e.path.as_str()))
        .collect();
    assert_eq!(dirty, [("??", "notes.txt")]);
    assert!(
        commit
            .as_deref()
            .is_some_and(|c| c.contains("HEAD did not move")),
        "{commit:?}"
    );
}

#[test]
fn a_typed_question_lists_the_words_it_accepts() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let failing = ir.script("git rev-parse -q --verify refs/heads/none");
    let failing = expecting(
        failing,
        ExitExpect::Success,
        ScriptIntent::Gate,
        false,
        &mut ir,
    );
    let steps = vec![
        vec![ir.script("git push --dry-run")],
        vec![failing],
        vec![ir.commit("git commit -q --allow-empty -m 'T1'")],
    ];
    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
    let answers = vec![
        typed("skip"),
        choice(Choice::Accept),
        typed("skip"),
        typed("quit"),
    ];
    let (finished, ui, _) = run(&f, &plan, AUTO, answers);
    assert_eq!(finished.end, End::Stopped, "{:?}", ui.shown);
    assert_eq!(ui.asked, ["typed-yes", "mismatch", "gate", "gate"]);
    assert_eq!(
        ui.words,
        [
            Some(RUN_SKIP_OR_QUIT),
            None,
            Some(YES_OR_QUIT),
            Some(YES_OR_QUIT)
        ]
    );
}

#[test]
fn a_dead_session_offers_retry_and_the_new_session_is_announced() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let first = bash(ir.script("x=1; exit 3"));
    let second = bash(ir.script("echo \"x=${x:-unset}\""));
    let steps = vec![vec![first], vec![second]];
    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
    let answers = vec![
        choice(Choice::Accept),
        choice(Choice::Retry),
        choice(Choice::Continue),
        choice(Choice::Quit),
    ];
    let (finished, ui, records) = run(&f, &plan, AUTO, answers);
    assert_eq!(finished.end, End::Stopped);
    assert_eq!(
        ui.asked,
        ["mismatch", "mismatch", "session", "mismatch"],
        "{:?}",
        ui.shown
    );
    let restarted = records
        .iter()
        .any(|r| matches!(r, Record::Session { reason, .. } if reason == "restarted"));
    assert!(restarted);
    assert!(
        ui.shown
            .iter()
            .any(|l| l.contains("retry starts a new one"))
    );
}

#[test]
fn a_working_directory_outside_the_repository_asks_before_the_next_block() {
    let f = Fixture::greeter(false);
    let outside = f.outside().display().to_string();
    let mut ir = Ir::default();
    let steps = vec![vec![
        ir.script(&format!("Set-Location '{outside}'")),
        ir.script("'here'"),
    ]];
    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
    let (finished, ui, _) = run(&f, &plan, AUTO, vec![choice(Choice::Continue)]);
    assert_eq!(finished.end, End::Completed, "{:?}", ui.shown);
    assert_eq!(ui.asked, ["outside"]);
}

#[test]
fn a_manual_waits_and_an_unscoped_write_asks_even_with_auto() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let manifest = ir.manifest(&[("a.txt", ChangeKind::Create, W)]);
    let steps = vec![
        vec![ir.manual("edit it by hand")],
        vec![ir.write("b.txt", "b\n", WriteMode::Create)],
    ];
    let plan = one(&mut ir, manifest, steps);
    let answers = vec![choice(Choice::Continue), choice(Choice::Run)];
    let (finished, ui, _) = run(&f, &plan, AUTO, answers);
    assert_eq!(finished.end, End::Completed, "{:?}", ui.shown);
    assert_eq!(ui.asked, ["manual", "unscoped"]);
    assert_eq!(f.read("b.txt").as_deref(), Some(&b"b\n"[..]));
}

#[test]
fn dirt_outside_the_task_asks_before_the_commit_and_a_missing_generated_file_closes_the_gate() {
    let f = Fixture::greeter(false);
    f.write("notes.txt", b"mine\n");
    let mut ir = Ir::default();
    let manifest = ir.manifest(&[("gen.txt", ChangeKind::Generated, W)]);
    let steps = vec![
        vec![ir.generated("gen.txt")],
        vec![ir.commit("git commit -q --allow-empty -m 'T1'")],
    ];
    let plan = one(&mut ir, manifest, steps);
    let answers = vec![choice(Choice::Continue), typed("quit")];
    let (finished, ui, _) = run(&f, &plan, AUTO, answers);
    assert_eq!(finished.end, End::Stopped);
    assert_eq!(ui.asked, ["dirt", "gate"]);
    assert!(
        ui.shown.iter().any(|l| l.contains("?? notes.txt")),
        "{:?}",
        ui.shown
    );
    assert!(
        ui.shown
            .iter()
            .any(|l| l.contains("`gen.txt` should have been generated"))
    );
}

#[test]
fn a_preflight_error_stops_before_anything_runs() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let manifest = ir.manifest(&[("src/lib.rs", ChangeKind::Create, W)]);
    let steps = vec![vec![ir.write("src/lib.rs", "x\n", WriteMode::Create)]];
    let plan = one(&mut ir, manifest, steps);
    let (finished, ui, records) = run(&f, &plan, AUTO, vec![]);
    assert_eq!(finished.end, End::Stopped);
    assert!(
        ui.shown.iter().any(|l| l.contains("create-exists")),
        "{:?}",
        ui.shown
    );
    assert_eq!(kinds(&records), ["run", "run-end"]);
}

#[test]
fn a_gate_from_a_session_that_ended_runs_again_before_the_commit() {
    let f = Fixture::greeter(false);
    let mut ir = Ir::default();
    let gate = bash(ir.script("g=0"));
    let gate = expecting(
        gate,
        ExitExpect::Success,
        ScriptIntent::Gate,
        false,
        &mut ir,
    );
    let dies_once = bash(ir.script("test -e .git/died || { echo x > .git/died; exit 3; }"));
    let commit = bash(ir.commit("test \"$g\" = 0 && git commit -q --allow-empty -m T1"));
    let steps = vec![vec![gate], vec![dies_once], vec![commit]];
    let plan = one(&mut ir, planrun_model::Manifest::default(), steps);
    let answers = vec![choice(Choice::Retry), choice(Choice::Continue)];
    let (finished, ui, _) = run(&f, &plan, AUTO, answers);
    assert_eq!(finished.end, End::Completed, "{:?}", ui.shown);
    assert_eq!(ui.asked, ["mismatch", "session"]);
    let rerun = ui.shown.iter().any(|l| l.contains("it runs again"));
    assert!(rerun, "{:?}", ui.shown);
    assert_eq!(subjects(&f), ["T1", "fixture"]);
}
