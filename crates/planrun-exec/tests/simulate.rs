//! File actions simulated on the virtual tree (D52, D53, D54) over a fixture copy: what each
//! handler accepts and refuses, and that the repository stays unchanged.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[path = "support/ir.rs"]
mod ir;

use fixture::Fixture;
use ir::{Ir, PATCH_T1, PATCH_T2, path};
use planrun_exec::HandlerSet;
use planrun_exec::finding::Finding;
use planrun_exec::handler::Simulated;
use planrun_exec::tree::VirtualTree;
use planrun_exec::workspace::Workspace;
use planrun_model::{Action, WriteMode};

fn open(f: &Fixture) -> Workspace {
    Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
}

/// Simulates `actions` in order on one tree; the outcome of each, as `ok <detail>` or the code.
fn run(ws: &Workspace, ir: &Ir, actions: &[Action]) -> Vec<String> {
    let handlers = HandlerSet::files();
    let source = ir.source();
    let mut tree = VirtualTree::new(ws).unwrap_or_else(|e| panic!("{e}"));
    actions
        .iter()
        .map(|a| {
            let handler = handlers.find(a).unwrap_or_else(|| panic!("no handler"));
            let outcome: Result<Simulated, Finding> = handler.simulate(a, &source, &mut tree);
            match outcome {
                Ok(s) => format!("ok {}", s.effect),
                Err(f) => f.code.to_owned(),
            }
        })
        .collect()
}

/// The bytes the tree holds for `p` after `actions`.
fn bytes_after(ws: &Workspace, ir: &Ir, actions: &[Action], p: &str) -> Vec<u8> {
    let handlers = HandlerSet::files();
    let source = ir.source();
    let mut tree = VirtualTree::new(ws).unwrap_or_else(|e| panic!("{e}"));
    for a in actions {
        let handler = handlers.find(a).unwrap_or_else(|| panic!("no handler"));
        handler
            .simulate(a, &source, &mut tree)
            .unwrap_or_else(|f| panic!("{}: {}", f.code, f.message));
    }
    tree.bytes(&path(p), None)
        .unwrap_or_else(|f| panic!("{}", f.message))
}

#[test]
fn the_assay_patches_chain_on_the_tree_and_the_repository_stays_unchanged() {
    for autocrlf in [false, true] {
        let f = Fixture::greeter(autocrlf);
        let before = f.snapshot();
        let ws = open(&f);
        let mut ir = Ir::default();
        let actions = [
            ir.write("src/greet.rs", "pub fn greeting() {}\n", WriteMode::Create),
            ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a"),
            ir.patch("src/lib.rs", PATCH_T2, "97bd92a", "6b269fc"),
            ir.delete("src/old.rs", true),
        ];
        assert_eq!(
            run(&ws, &ir, &actions),
            [
                "ok create",
                "ok aa80cf1 → 97bd92a",
                "ok 97bd92a → 6b269fc",
                "ok delete (git rm)"
            ],
            "autocrlf={autocrlf}"
        );
        assert_eq!(
            f.snapshot(),
            before,
            "autocrlf={autocrlf}: the tree wrote to the repository"
        );
    }
}

#[test]
fn a_patched_file_reads_as_a_checkout_would_write_it() {
    for (autocrlf, crlf) in [(false, false), (true, true)] {
        let f = Fixture::greeter(autocrlf);
        let ws = open(&f);
        let mut ir = Ir::default();
        let actions = [ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a")];
        let bytes = bytes_after(&ws, &ir, &actions, "src/lib.rs");
        assert_eq!(bytes.windows(2).any(|w| w == b"\r\n"), crlf);
        assert!(String::from_utf8_lossy(&bytes).contains("pub mod greet;"));
    }
}

#[test]
fn a_changed_file_fails_the_preimage_check() {
    let f = Fixture::greeter(false);
    f.write("src/lib.rs", b"//! changed\n");
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [ir.patch("src/lib.rs", PATCH_T1, "aa80cf1", "97bd92a")];
    assert_eq!(run(&ws, &ir, &actions), ["blob-mismatch"]);
}

#[test]
fn a_patch_without_its_preimage_does_not_apply() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [ir.patch("src/lib.rs", PATCH_T2, "aa80cf1", "6b269fc")];
    assert_eq!(run(&ws, &ir, &actions), ["patch-does-not-apply"]);
}

#[test]
fn writes_check_existence_and_from_manifest_is_refused() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.write("src/lib.rs", "x\n", WriteMode::Create),
        ir.write("src/none.rs", "x\n", WriteMode::Overwrite),
        ir.write("src/x.rs", "x\n", WriteMode::FromManifest),
        ir.delete("src/none.rs", false),
        ir.replace("src/none.rs", "a", "b"),
    ];
    assert_eq!(
        run(&ws, &ir, &actions),
        [
            "create-exists",
            "target-missing",
            "write-mode-unknown",
            "target-missing",
            "target-missing"
        ]
    );
}

#[test]
fn a_deleted_path_can_be_created_again() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.delete("src/old.rs", false),
        ir.write("src/old.rs", "new\n", WriteMode::Create),
        ir.write("src/old.rs", "newer\n", WriteMode::Overwrite),
    ];
    assert_eq!(
        run(&ws, &ir, &actions),
        ["ok delete", "ok create", "ok overwrite"]
    );
}

#[test]
fn a_replace_keeps_crlf_and_needs_one_occurrence() {
    let f = Fixture::greeter(true);
    let ws = open(&f);
    let mut ir = Ir::default();
    let edit = [ir.replace(
        "src/old.rs",
        "format!(\"Hello {name}\")",
        "format!(\"Hi {name}\")",
    )];
    let bytes = bytes_after(&ws, &ir, &edit, "src/old.rs");
    assert!(String::from_utf8_lossy(&bytes).contains("Hi {name}"));
    assert!(!bytes.windows(2).any(|w| w[1] == b'\n' && w[0] != b'\r'));
    let mut ir = Ir::default();
    let actions = [
        ir.replace("src/old.rs", "name", "x"),
        ir.replace("src/old.rs", "absent text", "x"),
    ];
    assert_eq!(
        run(&ws, &ir, &actions),
        ["before-not-unique", "before-not-unique"]
    );
}

#[test]
fn a_mixed_file_refuses_an_exact_edit_and_is_overwritten_by_checkout() {
    let f = Fixture::greeter(true);
    f.write("mixed.txt", b"a\r\nb\n");
    let ws = open(&f);
    let mut ir = Ir::default();
    let edit = [ir.replace("mixed.txt", "a", "c")];
    assert_eq!(run(&ws, &ir, &edit), ["eol-mixed"]);
    let mut ir = Ir::default();
    let overwrite = [ir.write("mixed.txt", "x\ny\n", WriteMode::Overwrite)];
    assert_eq!(
        bytes_after(&ws, &ir, &overwrite, "mixed.txt"),
        b"x\r\ny\r\n"
    );
}

#[test]
fn a_file_without_line_endings_takes_the_checkout_convention() {
    for (autocrlf, expected) in [(false, &b"x\ny\n"[..]), (true, &b"x\r\ny\r\n"[..])] {
        let f = Fixture::greeter(autocrlf);
        f.write("one.txt", b"one line");
        let ws = open(&f);
        let mut ir = Ir::default();
        let actions = [ir.write("one.txt", "x\ny\n", WriteMode::Overwrite)];
        assert_eq!(
            bytes_after(&ws, &ir, &actions, "one.txt"),
            expected,
            "autocrlf={autocrlf}"
        );
    }
}

#[test]
fn an_eol_attribute_wins_and_created_files_stay_lf() {
    let f = Fixture::greeter(true);
    f.write(".gitattributes", b"*.lf eol=lf\n");
    f.write("a.lf", b"a\r\nb\r\n");
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.write("a.lf", "x\ny\n", WriteMode::Overwrite),
        ir.write("new.txt", "x\ny\n", WriteMode::Create),
    ];
    assert_eq!(bytes_after(&ws, &ir, &actions, "a.lf"), b"x\ny\n");
    assert_eq!(bytes_after(&ws, &ir, &actions, "new.txt"), b"x\ny\n");
}

#[test]
fn two_spellings_of_one_path_collide_with_ignorecase() {
    let f = Fixture::greeter(false);
    f.git(&["config", "core.ignorecase", "true"]);
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.write("src/new.rs", "a\n", WriteMode::Create),
        ir.write("src/NEW.rs", "b\n", WriteMode::Create),
    ];
    assert_eq!(run(&ws, &ir, &actions), ["ok create", "path-collision"]);
}

#[test]
fn spellings_stay_apart_without_ignorecase() {
    let f = Fixture::greeter(false);
    f.git(&["config", "core.ignorecase", "false"]);
    if f.path().join("SRC").exists() {
        eprintln!("skipped: this file system ignores case, so the paths are one file on disk");
        return;
    }
    let ws = open(&f);
    let mut ir = Ir::default();
    let actions = [
        ir.write("src/new.rs", "a\n", WriteMode::Create),
        ir.write("src/NEW.rs", "b\n", WriteMode::Create),
    ];
    assert_eq!(run(&ws, &ir, &actions), ["ok create", "ok create"]);
}
