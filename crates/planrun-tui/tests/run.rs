//! A run in the TUI (decisions D90, D93): the synthetic assay plan is inspected, run with the
//! policy Inspect assumes, answered in the dock, and committed, on a copy of the greeter fixture,
//! with the real runner on its worker thread.

mod support;

use std::time::Duration;

use planrun_exec::shell::{ShellError, locate};
use planrun_exec::ui::{Choice, Reply};
use planrun_model::Shell;
use planrun_tui::driver::Core;
use planrun_tui::jobs::Bg;
use planrun_tui::keys::Key;
use planrun_tui::run::Phase;
use ratatui::backend::TestBackend;
use support::{Setup, screen, setup};

/// Whether pwsh is there; without it the test is skipped unless shells are required (D63).
pub fn pwsh() -> bool {
    match locate(Shell::Pwsh) {
        Ok(_) => true,
        Err(e @ ShellError::Unavailable { .. })
            if std::env::var_os("PLANRUN_REQUIRE_SHELLS").is_none() =>
        {
            eprintln!("skipped: {e}");
            false
        }
        Err(e) => panic!("{e}"),
    }
}

/// Opens the assay plan, assumes `--auto read,build,modify` and presses `R`.
fn started(setup: &Setup) -> Core<TestBackend> {
    let mut core = setup.core();
    core.key(Key::Enter);
    let inspected = core.wait(Duration::from_secs(60), |app| {
        app.inspect()
            .is_some_and(|i| matches!(i.dry, Bg::Current(_)))
    });
    assert!(inspected, "the dry run did not finish");
    for key in [
        Key::Char('a'),
        Key::Char('r'),
        Key::Char('b'),
        Key::Char('m'),
        Key::Enter,
        Key::Char('R'),
    ] {
        core.key(key);
    }
    core
}

/// Waits for the next question or the end; `None` at the end.
fn next_question(core: &mut Core<TestBackend>) -> Option<String> {
    let came = core.wait(Duration::from_secs(300), |app| {
        app.run()
            .is_some_and(|r| r.dock.is_some() || matches!(r.phase, Phase::Ended(_)))
    });
    assert!(came, "the run neither asked nor ended in time");
    let run = core.app().run()?;
    run.dock.as_ref().map(|d| d.question.kind.name().to_owned())
}

/// Answers the open question as the plans' replays do: a typed question `yes`, a confirmation
/// `y`, anything else enter.
fn answer(core: &mut Core<TestBackend>) {
    let Some(dock) = core.app().run().and_then(|r| r.dock.as_ref()) else {
        return;
    };
    let keys: Vec<Key> = match dock.question.typed {
        Some(_) => "yes".chars().map(Key::Char).chain([Key::Enter]).collect(),
        None if dock.question.choices.contains(&Choice::Yes) => vec![Key::Char('y')],
        None => vec![Key::Enter],
    };
    for key in keys {
        core.key(key);
    }
}

#[test]
fn the_assay_plan_runs_to_two_commits_through_the_dock() {
    if !pwsh() {
        return;
    }
    let setup = setup(&["assay-style.md"]);
    let mut core = started(&setup);
    let mut asked = Vec::new();
    while let Some(kind) = next_question(&mut core) {
        if asked.is_empty() {
            let text = screen(&mut core);
            assert!(text.contains("waiting for you"), "{text}");
            assert!(text.contains("CHANGES"), "{text}");
        }
        asked.push(kind);
        assert!(asked.len() < 20, "too many questions: {asked:?}");
        answer(&mut core);
    }
    assert_eq!(
        asked,
        ["confirm", "confirm"],
        "the commits' prose confirmations"
    );
    let run = core.app().run().unwrap_or_else(|| panic!("no run"));
    match &run.phase {
        Phase::Ended(Ok(f)) => assert_eq!(f.reason, None, "{f:?}"),
        other => panic!("{other:?}"),
    }
    assert_eq!(run.tasks_done.len(), 2);
    let log = setup.fixture.git(&["log", "--format=%s"]);
    assert_eq!(log.lines().count(), 3, "{log}");
    assert_eq!(
        setup.fixture.status(),
        "",
        "the tree is clean after the run"
    );
    let text = screen(&mut core);
    assert!(text.contains("done"), "{text}");
}

#[test]
fn a_typed_question_refuses_a_word_it_does_not_take() {
    let (tx, rx) = std::sync::mpsc::channel();
    let question = planrun_exec::ui::Question {
        kind: planrun_exec::ui::QuestionKind::Gate {
            reasons: vec!["a skip by you".to_owned()],
        },
        at: None,
        text: Vec::new(),
        choices: Vec::new(),
        typed: Some(planrun_exec::ui::YES_OR_QUIT),
    };
    let mut dock = planrun_tui::dock::Dock::new(question, tx);
    for c in "y".chars() {
        assert_eq!(dock.key(Key::Char(c)), None);
    }
    assert_eq!(dock.key(Key::Enter), None, "a single key never answers");
    assert_eq!(dock.refused.as_deref(), Some("y"));
    for c in "yes".chars() {
        dock.key(Key::Char(c));
    }
    let reply = dock.key(Key::Enter).unwrap_or_else(|| panic!("no reply"));
    assert_eq!(reply, Reply::Typed("yes".to_owned()));
    dock.answer(reply);
    assert_eq!(rx.recv().ok(), Some(Reply::Typed("yes".to_owned())));
}
