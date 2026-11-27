//! A crash, History, the resume preview and resume in the TUI (decisions D85, D93). A crash is
//! a panic from a journal observer, as in planrun-exec's resume tests: here at the start of
//! P1-T1's first script, after its step was written.

mod support;

use std::time::Duration;

use planrun_exec::journal::{Observer, Record, Seam};
use planrun_exec::ui::Choice;
use planrun_tui::driver::Core;
use planrun_tui::jobs::Bg;
use planrun_tui::keys::Key;
use planrun_tui::run::Phase;
use ratatui::backend::TestBackend;
use support::{screen, setup};

fn crash_at_the_first_script() -> Observer {
    Box::new(|seam| {
        if let Seam::Appended(Record::ActionStart { at, kind, .. }) = seam
            && kind == "script"
            && at.task == "P1-T1"
        {
            panic!("crash at {at:?}");
        }
    })
}

fn ended(core: &mut Core<TestBackend>) -> bool {
    core.wait(Duration::from_secs(300), |app| {
        app.run()
            .is_some_and(|r| r.dock.is_some() || matches!(r.phase, Phase::Ended(_)))
    })
}

fn answer_all(core: &mut Core<TestBackend>) -> Vec<String> {
    let mut asked = Vec::new();
    loop {
        assert!(ended(core), "the run neither asked nor ended in time");
        let Some(dock) = core.app().run().and_then(|r| r.dock.as_ref()) else {
            return asked;
        };
        asked.push(dock.question.kind.name().to_owned());
        assert!(asked.len() < 20, "{asked:?}");
        let keys: Vec<Key> = match dock.question.typed {
            Some(_) => "yes".chars().map(Key::Char).chain([Key::Enter]).collect(),
            None if dock.question.choices.contains(&Choice::Yes) => vec![Key::Char('y')],
            None if dock.question.choices.contains(&Choice::Retry) => vec![Key::Char('r')],
            None => vec![Key::Enter],
        };
        for key in keys {
            core.key(key);
        }
    }
}

#[test]
fn a_crashed_run_shows_in_history_its_preview_reads_only_and_resume_completes_it() {
    if !support::pwsh() {
        return;
    }
    let setup = setup(&["assay-style.md"]);
    let mut core = setup.core();
    core.app_mut()
        .observe_next_run(Box::new(crash_at_the_first_script));
    core.key(Key::Enter);
    let inspected = core.wait(Duration::from_secs(60), |app| {
        app.inspect()
            .is_some_and(|i| matches!(i.dry, Bg::Current(_)))
    });
    assert!(inspected);
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
    assert!(ended(&mut core));
    let phase = core.app().run().map(|r| r.phase.clone());
    assert!(matches!(phase, Some(Phase::Ended(Err(_)))), "{phase:?}");

    core.key(Key::F(4));
    let listed = core.wait(Duration::from_secs(30), |app| app.history().runs.is_some());
    assert!(listed);
    let text = screen(&mut core);
    assert!(text.contains("killed"), "{text}");
    assert!(
        text.contains("no run-end, no live run: the process was killed"),
        "{text}"
    );

    let before = setup.fixture.snapshot();
    core.key(Key::Char('R'));
    let read = core.wait(Duration::from_secs(60), |app| {
        app.resume_view()
            .is_some_and(|v| matches!(v.data, Bg::Current(_)))
    });
    assert!(read, "the preview did not read in time");
    assert_eq!(
        setup.fixture.snapshot(),
        before,
        "the preview wrote nothing, .git included"
    );
    let head = setup.fixture.git(&["rev-parse", "--short=7", "HEAD"]);
    // The plan's fingerprint is of its bytes, which a checkout with `core.autocrlf=true`
    // changes; the commit's id has the fixture's commit time in it.
    let fingerprint = core
        .app()
        .resume_view()
        .and_then(|v| v.data.value())
        .map(|d| d.analysis.run.fingerprint[..7].to_owned())
        .unwrap_or_default();
    let text = screen(&mut core)
        .replace(head.trim(), "<head7>")
        .replace(&fingerprint, "<plan7>");
    insta::assert_snapshot!(text);

    core.key(Key::Enter);
    let asked = answer_all(&mut core);
    assert!(
        asked.contains(&"again".to_owned()),
        "the running script asks again: {asked:?}"
    );
    let run = core.app().run().unwrap_or_else(|| panic!("no run"));
    match &run.phase {
        Phase::Ended(Ok(f)) => assert_eq!(f.reason, None, "{f:?}"),
        other => panic!("{other:?}"),
    }
    let log = setup.fixture.git(&["log", "--format=%s"]);
    assert_eq!(log.lines().count(), 3, "{log}");
}
