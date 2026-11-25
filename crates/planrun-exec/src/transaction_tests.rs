//! Step transactions on a fixture copy: prepare refuses before writing, commit writes what was
//! prepared, and an injected fault is rolled back byte for byte (D53).

use std::collections::BTreeMap;

use planrun_model::{Action, WriteMode};

use super::*;
use crate::fixture::Fixture;
use crate::ir::{Ir, PATCH_T1, abbrev, path};

#[derive(Default)]
struct Recorder {
    steps: Vec<PreparedStep>,
    fail: bool,
}

impl WriteAhead for Recorder {
    fn record(&mut self, step: &PreparedStep) -> std::io::Result<()> {
        if self.fail {
            return Err(std::io::Error::other("the journal is full"));
        }
        self.steps.push(step.clone());
        Ok(())
    }
}

fn open(f: &Fixture) -> Workspace {
    Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
}

fn prepared(ws: &Workspace, ir: &Ir, actions: &[Action]) -> Result<PreparedStep, Finding> {
    let refs: Vec<&Action> = actions.iter().collect();
    prepare(ws, &ir.source(), &HandlerSet::files(), &refs)
}

/// The working tree without `.git` (prepare stores pre-images there by design).
fn work_tree(f: &Fixture) -> BTreeMap<String, Vec<u8>> {
    f.snapshot()
        .into_iter()
        .filter(|(k, _)| !k.starts_with(".git/"))
        .collect()
}

#[test]
fn a_create_onto_an_existing_file_fails_before_anything_is_written() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let before = f.snapshot();
    let mut ir = Ir::default();
    let actions = [
        ir.write("src/greet.rs", "a\n", WriteMode::Create),
        ir.write("src/lib.rs", "b\n", WriteMode::Create),
    ];
    let err = prepared(&ws, &ir, &actions).err();
    assert_eq!(err.map(|f| f.code), Some("create-exists"));
    assert_eq!(
        f.snapshot(),
        before,
        "nothing, not even a pre-image, was written"
    );
}

#[test]
fn commit_writes_the_prepared_step_after_recording_it() {
    let f = Fixture::greeter(true);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.write("src/greet.rs", "pub fn greeting() {}\n", WriteMode::Create),
        ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a"),
        ir.delete("src/old.rs", true),
    ];
    let step = prepared(&ws, &ir, &actions).unwrap_or_else(|f| panic!("{}", f.message));
    assert_eq!(step.writes.len(), 3);
    assert_eq!(step.index_removals, [path("src/old.rs")]);
    let mut recorder = Recorder::default();
    commit(&ws, &step, &mut recorder).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(recorder.steps, [step]);
    let lib = f.read("src/lib.rs").unwrap_or_default();
    assert!(
        lib.windows(2).any(|w| w == b"\r\n"),
        "checked out as git apply would"
    );
    let id = ws
        .git()
        .hash_object(&path("src/lib.rs"), &lib, true, false, &[])
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(id.matches(&abbrev("97bd92a")));
    assert_eq!(
        f.read("src/greet.rs").as_deref(),
        Some(&b"pub fn greeting() {}\n"[..])
    );
    let mut status: Vec<String> = f.status().lines().map(str::to_owned).collect();
    status.sort();
    assert_eq!(
        status,
        [" M src/lib.rs", "?? src/greet.rs", "D  src/old.rs"]
    );
}

#[test]
fn a_failed_record_writes_nothing() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [ir.write("src/new.rs", "a\n", WriteMode::Create)];
    let step = prepared(&ws, &ir, &actions).unwrap_or_else(|f| panic!("{}", f.message));
    let before = work_tree(&f);
    let mut recorder = Recorder {
        fail: true,
        ..Recorder::default()
    };
    let err = commit(&ws, &step, &mut recorder).err();
    assert!(matches!(err.map(|e| e.cause), Some(CommitCause::Record(_))));
    assert_eq!(work_tree(&f), before);
}

#[test]
fn a_fault_in_a_step_is_rolled_back_byte_for_byte() {
    for failing in [2, 3] {
        let f = Fixture::greeter(true);
        f.write("notes.txt", b"untracked\r\nnotes\n");
        let ws = open(&f);
        let before = work_tree(&f);
        let mut ir = Ir::default();
        let actions = [
            ir.write("src/lib.rs", "//! replaced\n", WriteMode::Overwrite),
            ir.write("notes.txt", "replaced\n", WriteMode::Overwrite),
            ir.write("new/dir/x.txt", "x\n", WriteMode::Create),
        ];
        let step = prepared(&ws, &ir, &actions).unwrap_or_else(|f| panic!("{}", f.message));
        ws.faults.fail_write.set(Some(failing));
        let err = commit(&ws, &step, &mut Recorder::default())
            .err()
            .unwrap_or_else(|| panic!("write {failing} should fail"));
        assert!(err.unrestored.is_empty(), "{err}");
        assert_eq!(work_tree(&f), before, "write {failing}: not restored");
        assert!(
            !f.path().join("new").exists(),
            "write {failing}: directory left"
        );
        let temps = f
            .snapshot()
            .into_keys()
            .filter(|k| k.contains(".planrun-"))
            .count();
        assert_eq!(temps, 0, "write {failing}: temp files left");
    }
}

#[test]
fn a_path_that_ends_as_it_began_is_not_written() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.write("tmp.txt", "a\n", WriteMode::Create),
        ir.delete("tmp.txt", false),
    ];
    let step = prepared(&ws, &ir, &actions).unwrap_or_else(|f| panic!("{}", f.message));
    assert!(step.writes.is_empty());
}

#[test]
fn prepare_lists_the_directories_a_write_creates() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [ir.write("docs/a/b.txt", "x\n", WriteMode::Create)];
    let step = prepared(&ws, &ir, &actions).unwrap_or_else(|f| panic!("{}", f.message));
    assert_eq!(step.writes[0].creates_dirs, ["docs", "docs/a"]);
    assert_eq!(step.writes[0].pre, PreImage::Absent);
}
