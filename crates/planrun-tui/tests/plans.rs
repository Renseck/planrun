//! The Plans view (docs/tui-design.md §6.A) on the greeter fixture with the synthetic plans.

mod support;

use planrun_tui::keys::Key;
use support::{PLANS, screen, setup};

#[test]
fn the_plans_view_lists_every_file_with_its_check_and_run() {
    let setup = setup(&PLANS);
    let mut core = setup.core();
    core.key(Key::Down);
    insta::assert_snapshot!(screen(&mut core));
}

#[test]
fn the_filter_narrows_the_table_and_esc_clears_it() {
    let setup = setup(&PLANS);
    let mut core = setup.core();
    for key in [
        Key::Char('/'),
        Key::Char('d'),
        Key::Char('e'),
        Key::Char('v'),
        Key::Enter,
    ] {
        core.key(key);
    }
    let names: Vec<String> = core
        .app()
        .plans()
        .shown()
        .iter()
        .map(|e| e.name.clone())
        .collect();
    assert_eq!(names, ["devready-style.md"]);
    core.key(Key::Esc);
    assert_eq!(core.app().plans().shown().len(), PLANS.len());
}

#[test]
fn below_100_by_30_the_tui_says_it_is_too_small() {
    let setup = setup(&PLANS[..1]);
    let mut core = setup.core();
    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut small = planrun_tui::driver::Core::new(
        setup.app(),
        ratatui::Terminal::new(backend).unwrap_or_else(|e| panic!("{e}")),
        Box::new(support::Counted::default()),
    );
    let text = screen(&mut small);
    assert!(text.contains("terminal too small"), "{text}");
    assert!(text.contains("needs 100 × 30"), "{text}");
    let _ = screen(&mut core);
}

#[test]
fn a_job_that_panics_comes_back_in_its_failed_state() {
    let setup = setup(&PLANS[..1]);
    let app = setup.app_with(Some(Box::new(|| panic!("the update check broke"))));
    let terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 36))
        .unwrap_or_else(|e| panic!("{e}"));
    let mut core =
        planrun_tui::driver::Core::new(app, terminal, Box::new(support::Counted::default()));
    core.start();
    let failed = core.wait(std::time::Duration::from_secs(30), |app| {
        app.update() == &planrun_tui::jobs::UpdateState::Failed
    });
    assert!(failed, "the panicked job never reported");
    assert!(screen(&mut core).contains("update check failed"));
}

#[test]
fn q_quits_from_plans() {
    let setup = setup(&PLANS[..1]);
    let mut core = setup.core();
    core.key(Key::Char('q'));
    assert!(core.done());
}

#[test]
fn without_a_directory_the_plans_view_lists_the_discovered_plans() {
    use std::time::{Duration, UNIX_EPOCH};

    let fixture = support::fixture::Fixture::greeter(false);
    let corpus =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/synthetic");
    let files = [
        ("docs/plans/2026-01-01-p1-greet.md", "assay-style.md"),
        ("docs/plans/README.md", "README.md"),
        (
            "tools/superpowers/plans/2026-01-02-tools.md",
            "superpowers-style.md",
        ),
        ("notes/2026-01-03-elsewhere.md", "devready-style.md"),
    ];
    for (to, from) in files {
        let bytes = std::fs::read(corpus.join(from)).unwrap_or_else(|e| panic!("{e}"));
        fixture.write(to, &bytes);
        std::fs::File::options()
            .write(true)
            .open(fixture.path().join(to))
            .and_then(|f| f.set_modified(UNIX_EPOCH + Duration::from_millis(support::NOW)))
            .unwrap_or_else(|e| panic!("{e}"));
    }
    let mut ctx =
        planrun_tui::terminal::context(fixture.path(), None, None, "0.1.0", fixture.env())
            .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(ctx.plans_label, "default globs");
    ctx.repo = "greeter".to_owned();
    ctx.target = r"C:\src\greeter".to_owned();
    let app = planrun_tui::app::App::new(
        ctx,
        planrun_tui::glyph::Glyphs::unicode(),
        std::sync::Arc::new(|| support::NOW),
        None,
    );
    let terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 36))
        .unwrap_or_else(|e| panic!("{e}"));
    let mut core =
        planrun_tui::driver::Core::new(app, terminal, Box::new(support::Counted::default()));
    core.start();
    let checked = core.wait(Duration::from_secs(60), |app| {
        let entries = &app.plans().entries;
        entries.len() == 3
            && entries
                .iter()
                .all(|e| matches!(e.check, planrun_tui::jobs::Bg::Current(_)))
    });
    assert!(checked, "the discovered plans were not checked in time");
    let names: Vec<String> = core
        .app()
        .plans()
        .entries
        .iter()
        .map(|e| e.name.clone())
        .collect();
    assert_eq!(
        names,
        ["2026-01-01-p1-greet.md", "README.md", "2026-01-02-tools.md"]
    );
    insta::assert_snapshot!(screen(&mut core));
}
