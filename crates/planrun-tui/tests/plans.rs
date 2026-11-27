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
