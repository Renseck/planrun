//! The dock's round trips and Ctrl-C (decisions D73, D90, D91): `[o]` runs the shell through
//! the hooks and the runner asks again, `[v]` and `[d]` are answered in the TUI with the dock
//! left open, and Ctrl-C ends planrun only after its confirm.

mod support;

use std::sync::atomic::Ordering;
use std::time::Duration;

use planrun_exec::ui::{Choice, Question, QuestionKind, Reply};
use planrun_tui::dock::Dock;
use planrun_tui::driver::Core;
use planrun_tui::jobs::Bg;
use planrun_tui::keys::Key;
use planrun_tui::run::{Inspector, Phase};
use ratatui::backend::TestBackend;
use support::{Counted, screen, setup};

/// The journal name of the next question, once it is asked.
fn question(core: &mut Core<TestBackend>) -> String {
    let asked = core.wait(Duration::from_secs(300), |app| {
        app.run()
            .is_some_and(|r| r.dock.is_some() || matches!(r.phase, Phase::Ended(_)))
    });
    assert!(asked, "no question in time");
    let run = core.app().run().unwrap_or_else(|| panic!("no run"));
    let dock = run
        .dock
        .as_ref()
        .unwrap_or_else(|| panic!("ended: {:?}", run.phase));
    dock.question.kind.name().to_owned()
}

#[test]
fn shell_view_and_diff_come_back_to_the_question_and_ctrl_c_ends_after_its_confirm() {
    if std::env::var_os("PLANRUN_REQUIRE_SHELLS").is_none()
        && planrun_exec::shell::locate(planrun_model::Shell::Pwsh).is_err()
    {
        eprintln!("skipped: no pwsh");
        return;
    }
    let setup = setup(&["assay-style.md"]);
    let hooks = Counted::default();
    let mut core = setup.core_with(hooks.clone());
    core.key(Key::Enter);
    let inspected = core.wait(Duration::from_secs(60), |app| {
        app.inspect()
            .is_some_and(|i| matches!(i.dry, Bg::Current(_)))
    });
    assert!(inspected);
    core.key(Key::Char('R'));
    assert_eq!(
        question(&mut core),
        "run",
        "cargo test greet asks under the default policy"
    );
    core.key(Key::Char('o'));
    assert_eq!(question(&mut core), "run");
    assert_eq!(hooks.shells.load(Ordering::SeqCst), 1);
    core.key(Key::Char('v'));
    assert_eq!(question(&mut core), "run");
    let viewed = core.app().run().map(|r| r.inspector.clone());
    assert!(
        matches!(viewed, Some(Inspector::Action((0, 2, 1)))),
        "{viewed:?}"
    );
    assert!(screen(&mut core).contains("ACTION"));
    core.key(Key::Char('d'));
    let shown = core.wait(Duration::from_secs(30), |app| {
        app.run()
            .is_some_and(|r| matches!(r.inspector, Inspector::Tree(..)))
    });
    assert!(shown, "the diff was not shown");
    assert!(screen(&mut core).contains("WORKING TREE"));
    core.key(Key::Ctrl('c'));
    assert!(screen(&mut core).contains("END PLANRUN"));
    core.key(Key::Char('n'));
    assert!(!screen(&mut core).contains("END PLANRUN"));
    assert_eq!(hooks.ends.load(Ordering::SeqCst), 0);
    core.key(Key::Ctrl('c'));
    core.key(Key::Ctrl('c'));
    assert_eq!(hooks.ends.load(Ordering::SeqCst), 1);
    assert!(core.done());
}

#[test]
fn q_at_a_commit_s_confirmation_needs_pressing_twice() {
    let (tx, rx) = std::sync::mpsc::channel();
    let q = Question {
        kind: QuestionKind::Confirm {
            reasons: Vec::new(),
            errored: false,
        },
        at: None,
        text: Vec::new(),
        choices: vec![Choice::Yes, Choice::No, Choice::Quit],
        typed: None,
    };
    let mut dock = Dock::new(q, tx);
    dock.guard_quit = true;
    assert_eq!(dock.key(Key::Char('q')), None);
    assert!(dock.armed);
    assert_eq!(dock.key(Key::Char('q')), Some(Reply::Choice(Choice::Quit)));
    let mut plain = Dock::new(
        Question {
            kind: QuestionKind::Manual,
            at: None,
            text: Vec::new(),
            choices: vec![Choice::Continue, Choice::Quit],
            typed: None,
        },
        std::sync::mpsc::channel().0,
    );
    assert_eq!(plain.key(Key::Char('q')), Some(Reply::Choice(Choice::Quit)));
    assert_eq!(plain.key(Key::Enter), Some(Reply::Choice(Choice::Continue)));
    drop(rx);
}
