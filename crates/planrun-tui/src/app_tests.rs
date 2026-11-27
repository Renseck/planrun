//! The app's own tests: what a terminal below the minimum shows.

use std::sync::mpsc::channel;

use planrun_exec::ui::{Choice, Question, QuestionKind};
use planrun_parse::{Registry, parse_plan};

use super::*;
use crate::grid::grid;

const PLAN: &str = include_str!("../../../tests/corpus/synthetic/assay-style.md");

fn app() -> App {
    let ctx = Context {
        root: PathBuf::from("."),
        git_dir: PathBuf::from(".git"),
        repo: "greeter".to_owned(),
        target: "greeter".to_owned(),
        plans_dir: PathBuf::from("plans"),
        plans_label: "plans".to_owned(),
        version: "0.1.0".to_owned(),
        env: Vec::new(),
    };
    App::new(ctx, Glyphs::unicode(), Arc::new(|| 0), None)
}

#[test]
fn below_the_minimum_an_open_dock_is_drawn_alone_and_answers() {
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let parsed = parse_plan(&registry, "assay-style.md", PLAN.as_bytes(), None)
        .unwrap_or_else(|e| panic!("{e:?}"));
    let mut app = app();
    let mut run = Run::new(
        PathBuf::from("assay-style.md"),
        Arc::new(parsed),
        Assumed::default(),
        false,
        0,
    );
    let (tx, rx) = channel();
    run.ask(
        Question {
            kind: QuestionKind::Manual,
            at: None,
            text: vec!["check it by hand".to_owned()],
            choices: vec![Choice::Continue, Choice::Quit],
            typed: None,
        },
        tx,
    );
    app.run = Some(run);
    app.view = View::Plan;
    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
    app.draw(&mut buf);
    let text = grid(&buf);
    assert!(text.contains("MANUAL"), "{text}");
    assert!(!text.contains("terminal too small"), "{text}");
    app.key(Key::Enter);
    assert_eq!(
        rx.recv().ok(),
        Some(planrun_exec::ui::Reply::Choice(Choice::Continue))
    );
    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
    app.draw(&mut buf);
    assert!(grid(&buf).contains("terminal too small"));
}
