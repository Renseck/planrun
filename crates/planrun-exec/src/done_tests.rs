//! A step's state on the disk on a fixture copy: Done after its commit, NotDone before, Unknown
//! when half written; restore, the temp files, the check between prepare and commit (D77, D78).

use std::collections::BTreeMap;

use planrun_model::{Action, WriteMode};

use super::*;
use crate::fixture::Fixture;
use crate::handler::HandlerSet;
use crate::ir::{Ir, PATCH_T1, path};
use crate::transaction::{CommitCause, PreparedStep, WriteAhead, commit, prepare};

#[derive(Default)]
struct Recorder(usize);

impl WriteAhead for Recorder {
    fn record(&mut self, _step: &PreparedStep) -> std::io::Result<()> {
        self.0 += 1;
        Ok(())
    }
}

fn open(f: &Fixture) -> Workspace {
    Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
}

fn prepared(
    ws: &Workspace,
    ir: &mut Ir,
    make: impl FnOnce(&mut Ir) -> Vec<Action>,
) -> PreparedStep {
    let actions = make(ir);
    let refs: Vec<&Action> = actions.iter().collect();
    prepare(ws, &ir.source(), &HandlerSet::files(), &refs)
        .unwrap_or_else(|f| panic!("{}", f.message))
}

fn state(ws: &Workspace, step: &PreparedStep) -> DoneState {
    step_state(ws, &step.record())
        .unwrap_or_else(|e| panic!("{e}"))
        .state
}

fn work_tree(f: &Fixture) -> BTreeMap<String, Vec<u8>> {
    f.snapshot()
        .into_iter()
        .filter(|(k, _)| !k.starts_with(".git/"))
        .collect()
}

fn three(ir: &mut Ir) -> Vec<Action> {
    vec![
        ir.write("src/greet.rs", "pub fn greeting() {}\n", WriteMode::Create),
        ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a"),
        ir.delete("src/old.rs", true),
    ]
}

#[test]
fn a_step_is_not_done_before_its_commit_and_done_after_it() {
    for autocrlf in [false, true] {
        let f = Fixture::greeter(autocrlf);
        let ws = open(&f);
        let mut ir = Ir::default();
        let step = prepared(&ws, &mut ir, three);
        assert_eq!(state(&ws, &step), DoneState::NotDone, "{autocrlf}");
        commit(&ws, &step, &mut Recorder::default()).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(state(&ws, &step), DoneState::Done, "{autocrlf}");
    }
}

#[test]
fn a_half_written_step_is_unknown_and_restore_puts_it_back() {
    let f = Fixture::greeter(true);
    let ws = open(&f);
    let before = work_tree(&f);
    let mut ir = Ir::default();
    let step = prepared(&ws, &mut ir, three);
    f.write("src/greet.rs", b"pub fn greeting() {}\n");
    let found = step_state(&ws, &step.record()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(found.state, DoneState::Unknown);
    assert_eq!(
        found.paths.iter().map(|(_, s)| *s).collect::<Vec<_>>(),
        [Seen::Post, Seen::Pre, Seen::Pre]
    );
    assert!(restorable(&ws, &step.record()).unwrap_or_else(|e| panic!("{e}")));
    assert!(restore_step(&ws, &step.record()).is_empty());
    assert_eq!(work_tree(&f), before);
    assert_eq!(state(&ws, &step), DoneState::NotDone);
}

#[test]
fn a_git_delete_with_its_index_entry_still_there_is_unknown() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let step = prepared(&ws, &mut ir, |ir| vec![ir.delete("src/old.rs", true)]);
    std::fs::remove_file(f.path().join("src/old.rs")).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(state(&ws, &step), DoneState::Unknown);
    f.git(&["rm", "-q", "--cached", "src/old.rs"]);
    assert_eq!(state(&ws, &step), DoneState::Done);
    assert!(!restorable(&ws, &step.record()).unwrap_or_else(|e| panic!("{e}")));
}

#[test]
fn a_path_changed_between_prepare_and_commit_writes_and_records_nothing() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let step = prepared(&ws, &mut ir, three);
    f.write("src/lib.rs", b"//! someone else\n");
    f.write("src/greet.rs", b"appeared\n");
    let before = work_tree(&f);
    let mut recorder = Recorder::default();
    let err = commit(&ws, &step, &mut recorder).err();
    let changed = match err.map(|e| e.cause) {
        Some(CommitCause::Changed(paths)) => paths,
        other => panic!("{other:?}"),
    };
    assert_eq!(changed, [path("src/greet.rs"), path("src/lib.rs")]);
    assert_eq!(recorder.0, 0);
    assert_eq!(work_tree(&f), before);
}

#[test]
fn a_directory_the_step_creates_that_appeared_is_a_change() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let step = prepared(&ws, &mut ir, |ir| {
        vec![ir.write("docs/a.txt", "a\n", WriteMode::Create)]
    });
    std::fs::create_dir(f.path().join("docs")).unwrap_or_else(|e| panic!("{e}"));
    let err = commit(&ws, &step, &mut Recorder::default()).err();
    assert!(matches!(
        err.map(|e| e.cause),
        Some(CommitCause::Changed(_))
    ));
}

#[test]
fn only_the_temp_files_a_step_names_are_removed() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let step = prepared(&ws, &mut ir, three).record();
    let named = format!("src/{}", step.writes[1].temp);
    f.write(&named, b"half");
    f.write("src/.planrun-0123456789abcdef.tmp", b"not this run's");
    assert_eq!(temps(&ws, &step), vec![named.clone()]);
    assert!(f.read(&named).is_some(), "temps only reads");
    assert_eq!(remove_temps(&ws, &step), vec![named.clone()]);
    assert!(f.read(&named).is_none());
    assert!(f.read("src/.planrun-0123456789abcdef.tmp").is_some());
}

#[test]
fn a_pruned_pre_image_is_reported_missing() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let mut step = prepared(&ws, &mut ir, three).record();
    assert!(
        missing_pre_images(&ws, &step)
            .unwrap_or_else(|e| panic!("{e}"))
            .is_empty()
    );
    let gone = crate::git::BlobId::parse(&"1".repeat(40)).unwrap_or_else(|| panic!("hex"));
    step.writes[1].pre = PreImage::Stored(gone);
    assert_eq!(
        missing_pre_images(&ws, &step).unwrap_or_else(|e| panic!("{e}")),
        [gone]
    );
    assert!(!restorable(&ws, &step).unwrap_or_else(|e| panic!("{e}")));
}

#[cfg(unix)]
#[test]
fn an_executable_file_stays_executable_and_its_mode_counts() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::greeter(false);
    f.write("run.sh", b"echo a\n");
    let full = f.path().join("run.sh");
    let mut perms = std::fs::metadata(&full)
        .unwrap_or_else(|e| panic!("{e}"))
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&full, perms).unwrap_or_else(|e| panic!("{e}"));
    f.git(&["add", "run.sh"]);
    f.git(&["commit", "-q", "-m", "x"]);
    let ws = open(&f);
    let mut ir = Ir::default();
    let step = prepared(&ws, &mut ir, |ir| {
        vec![ir.write("run.sh", "echo b\n", WriteMode::Overwrite)]
    });
    assert_eq!(step.writes[0].record.mode.as_deref(), Some("100755"));
    commit(&ws, &step, &mut Recorder::default()).unwrap_or_else(|e| panic!("{e}"));
    assert!(Workspace::is_executable(&full));
    assert_eq!(state(&ws, &step), DoneState::Done);
    let mut perms = std::fs::metadata(&full)
        .unwrap_or_else(|e| panic!("{e}"))
        .permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(&full, perms).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(state(&ws, &step), DoneState::Unknown);
}
