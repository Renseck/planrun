//! The plan view in Inspect (docs/tui-design.md §6.B, D84) on the synthetic assay plan.

mod support;

use std::time::Duration;

use planrun_tui::jobs::Bg;
use planrun_tui::keys::Key;
use planrun_tui::tree::Node;
use support::{screen, setup};

/// A core with `assay-style.md` open in Inspect and its dry run done.
fn inspected(setup: &support::Setup) -> planrun_tui::driver::Core<ratatui::backend::TestBackend> {
    let mut core = setup.core();
    core.key(Key::Enter);
    let done = core.wait(Duration::from_secs(60), |app| {
        app.inspect()
            .is_some_and(|i| matches!(i.dry, Bg::Current(_)))
    });
    assert!(done, "the dry run did not finish");
    core
}

#[test]
fn inspect_shows_the_selected_patch_its_verdict_and_the_dry_run() {
    let setup = setup(&["assay-style.md"]);
    let mut core = inspected(&setup);
    for _ in 0..3 {
        core.key(Key::Down);
    }
    let node = core.app().inspect().and_then(|i| i.tree.current(i.plan()));
    assert_eq!(node, Some(Node::Action(0, 1, 1)));
    insta::assert_snapshot!(screen(&mut core));
}

#[test]
fn the_assumed_policy_changes_a_script_s_tag() {
    let setup = setup(&["assay-style.md"]);
    let mut core = inspected(&setup);
    let tag = |core: &planrun_tui::driver::Core<_>| {
        let inspect = core.app().inspect().unwrap_or_else(|| panic!("no inspect"));
        inspect.mark(Node::Action(0, 2, 1)).tag
    };
    assert_eq!(tag(&core), "ask");
    for key in [Key::Char('a'), Key::Char('b'), Key::Enter] {
        core.key(key);
    }
    assert_eq!(tag(&core), "auto");
    let described = core.app().inspect().map(|i| i.assumed.describe());
    assert_eq!(described.as_deref(), Some("auto build"));
}

#[test]
fn esc_goes_back_to_plans_and_f3_returns() {
    let setup = setup(&["assay-style.md"]);
    let mut core = inspected(&setup);
    core.key(Key::Esc);
    assert!(screen(&mut core).contains("PLANS"));
    core.key(Key::F(3));
    assert!(screen(&mut core).contains("INSPECT"));
}
