//! The command palette (docs/tui-design.md §6.G) and help (§7).

mod support;

use std::time::Duration;

use planrun_tui::jobs::Bg;
use planrun_tui::keys::Key;
use planrun_tui::tree::Node;
use support::{PLANS, screen, setup};

#[test]
fn the_palette_lists_the_registry_and_runs_the_chosen_command() {
    let setup = setup(&PLANS);
    let mut core = setup.core();
    core.key(Key::Ctrl('k'));
    for c in "insp ass".chars() {
        core.key(Key::Char(c));
    }
    insta::assert_snapshot!(screen(&mut core));
    core.key(Key::Enter);
    assert!(core.app().palette().is_none());
    let name = core
        .app()
        .inspect()
        .map(|i| i.path.file_name().map(|n| n.to_string_lossy().into_owned()));
    assert_eq!(name, Some(Some("assay-style.md".to_owned())));
}

#[test]
fn ctrl_p_goes_to_an_action_of_the_plan() {
    let setup = setup(&["assay-style.md"]);
    let mut core = setup.core();
    core.key(Key::Enter);
    let inspected = core.wait(Duration::from_secs(60), |app| {
        app.inspect()
            .is_some_and(|i| matches!(i.dry, Bg::Current(_)))
    });
    assert!(inspected);
    core.key(Key::Ctrl('p'));
    for c in "old".chars() {
        core.key(Key::Char(c));
    }
    core.key(Key::Enter);
    let node = core.app().inspect().and_then(|i| i.tree.current(i.plan()));
    assert_eq!(node, Some(Node::Action(1, 1, 1)), "− src/old.rs");
}

#[test]
fn f1_shows_the_keys_and_esc_closes_them() {
    let setup = setup(&PLANS[..1]);
    let mut core = setup.core();
    core.key(Key::F(1));
    let text = screen(&mut core);
    assert!(text.contains("KEYS"), "{text}");
    assert!(text.contains("THIS VIEW"), "{text}");
    core.key(Key::Esc);
    assert!(!screen(&mut core).contains("THIS VIEW"));
    core.key(Key::Char('/'));
    core.key(Key::Char('?'));
    assert!(
        !screen(&mut core).contains("THIS VIEW"),
        "? is typed into the filter"
    );
}
