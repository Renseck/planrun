//! The `planrun` binary: exit codes, `check` output, the rendered message of every invalid
//! plan, and `show` of one step. Runs from the workspace root with relative paths, so the output
//! is the same on every machine; the user's config directory is one that does not exist, so
//! their formats stay out (D96).

#[path = "../../../tests/support/fixture.rs"]
mod fixture;

use std::path::{Path, PathBuf};
use std::process::Command;

use fixture::Fixture;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Runs `planrun args…`; returns the exit code and stdout + stderr.
fn planrun(args: &[&str]) -> (i32, String) {
    planrun_in(&root(), args)
}

/// A user config directory that does not exist.
fn no_user_config() -> PathBuf {
    root().join("target/planrun-tests-no-user-config")
}

/// Runs `planrun args…` in `dir`.
fn planrun_in(dir: &Path, args: &[&str]) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_planrun"))
        .args(args)
        .current_dir(dir)
        .env("PLANRUN_CONFIG_DIR", no_user_config())
        .output()
        .unwrap_or_else(|e| panic!("{e}"));
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), text)
}

const SYNTHETIC: &str = "tests/corpus/synthetic/assay-style.md";

#[test]
fn check_passes_the_synthetic_plan() {
    let (code, out) = planrun(&["check", SYNTHETIC]);
    assert_eq!(code, 0, "{out}");
    insta::assert_snapshot!("check_assay_style", out);
}

#[test]
fn check_renders_every_invalid_plan() {
    let dir = root().join("tests/corpus/invalid");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{e}"))
        .map(|e| {
            e.unwrap_or_else(|e| panic!("{e}"))
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|n| n.ends_with(".md") && n != "README.md")
        .collect();
    names.sort();
    let mut all = String::new();
    for name in &names {
        let path = format!("tests/corpus/invalid/{name}");
        let (code, out) = planrun(&["check", &path]);
        assert_eq!(code, 1, "{path}: {out}");
        all.push_str(&format!("==== {name}\n{out}\n"));
    }
    insta::assert_snapshot!("check_invalid_corpus", all);
}

#[test]
fn show_prints_a_step_in_full() {
    let (code, out) = planrun(&["show", SYNTHETIC, "P1-T2/2"]);
    assert_eq!(code, 0, "{out}");
    insta::assert_snapshot!("show_p1_t2_step_2", out);
}

#[test]
fn usage_problems_exit_with_2() {
    assert_eq!(planrun(&["check"]).0, 2);
    assert_eq!(planrun(&["check", "tests/corpus/none.md"]).0, 2);
    assert_eq!(planrun(&["show", SYNTHETIC, "P9-T9"]).0, 2);
    assert_eq!(planrun(&["show", SYNTHETIC, "P1-T1/7"]).0, 2);
    assert_eq!(planrun(&["check", SYNTHETIC, "--format", "nope"]).0, 1);
}

#[test]
fn tui_refuses_a_stdout_that_is_not_a_terminal() {
    let (code, out) = planrun(&["tui"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("planrun tui needs a terminal"), "{out}");
}

/// A fixture copy with the synthetic assay plan at its root.
fn with_plan() -> Fixture {
    let f = Fixture::greeter(false);
    let plan = std::fs::read(root().join(SYNTHETIC)).unwrap_or_else(|e| panic!("{e}"));
    f.write("assay-style.md", &plan);
    f
}

#[test]
fn a_repository_format_overrides_the_built_in_one_and_check_says_so() {
    let f = with_plan();
    let spec = std::fs::read(root().join("formats/assay.toml")).unwrap_or_else(|e| panic!("{e}"));
    f.write(".planrun/formats/assay.toml", &spec);
    let (code, out) = planrun_in(f.path(), &["check", "assay-style.md"]);
    assert_eq!(code, 0, "{out}");
    let last = out.lines().last().unwrap_or_default();
    assert_eq!(
        last,
        "planrun: assay-style.md: ok (assay (.planrun/formats/assay.toml, overrides built-in), 2 tasks)"
    );
    let (code, out) = planrun_in(f.path(), &["show", "assay-style.md"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("format  assay (.planrun/formats/assay.toml, overrides built-in)\n"),
        "{out}"
    );
}

#[test]
fn a_spec_or_config_that_cannot_be_used_stops_with_2() {
    let f = with_plan();
    f.write(".planrun/formats/broken.toml", b"id = 1\n");
    let (code, out) = planrun_in(f.path(), &["check", "assay-style.md"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("broken.toml: format `broken`:"), "{out}");
    std::fs::remove_file(f.path().join(".planrun/formats/broken.toml"))
        .unwrap_or_else(|e| panic!("{e}"));
    f.write(".planrun/config.toml", b"[plans]\nglobs = []\n");
    let (code, out) = planrun_in(f.path(), &["check", "assay-style.md"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("error[config-globs-empty]"), "{out}");
}
