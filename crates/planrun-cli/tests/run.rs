//! The P5 acceptance (docs/roadmap.md): `planrun run tests/corpus/synthetic/assay-style.md
//! --auto read,build,modify` on a fixture copy makes the two commits with the expected trees,
//! and a run killed mid-task resumes and completes without writing anything twice. Built only
//! with the `e2e` feature (D82): answers come from a pipe, and `PLANRUN_E2E_ABORT` ends the
//! process at a named seam. Each leg spells out its prompts.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant, SystemTime};

use fixture::Fixture;
use planrun_exec::journal::{self, Record};
use planrun_exec::shell::{ShellError, locate};
use planrun_model::Shell;

const PLAN: &str = "assay-style.md";
const AUTO: &str = "read,build,modify";

/// `src/greet.rs` as the plan writes it.
const GREET: &str = "//! Greetings (P1-T1).

/// The greeting for `name`, with punctuation.
pub fn greeting(name: &str) -> String {
    format!(\"Hello, {name}!\")
}

#[cfg(test)]
mod tests {
    #[test]
    fn greets_by_name() {
        assert_eq!(super::greeting(\"Rens\"), \"Hello, Rens!\");
    }
}
";

/// `src/lib.rs` after P1-T1 (blob 97bd92a) and after P1-T2 (blob 6b269fc).
const LIB_T1: &str = "//! A tiny fixture crate for the planrun corpus.

pub mod greet;
pub mod old;

/// The greeting for `name`.
pub fn greeting(name: &str) -> String {
    greet::greeting(name)
}
";
const LIB_T2: &str = "//! A tiny fixture crate for the planrun corpus.

pub mod greet;

pub use greet::greeting;
";

/// Whether pwsh is there; without it the test is skipped unless shells are required (D63).
fn pwsh() -> bool {
    match locate(Shell::Pwsh) {
        Ok(_) => true,
        Err(e @ ShellError::Unavailable { .. })
            if std::env::var_os("PLANRUN_REQUIRE_SHELLS").is_none() =>
        {
            eprintln!("skipped: {e}");
            false
        }
        Err(e) => panic!("{e}"),
    }
}

/// A fixture copy with the plan next to it, outside the repository.
fn fixture(autocrlf: bool) -> Fixture {
    let f = Fixture::greeter(autocrlf);
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/corpus/synthetic")
        .join(PLAN);
    let text = std::fs::read_to_string(&source).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(f.outside().join(PLAN), text).unwrap_or_else(|e| panic!("{e}"));
    f
}

/// A running `planrun` with its stdout read line by line.
struct Planrun {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<String>,
    seen: Vec<String>,
}

impl Planrun {
    fn start(f: &Fixture, args: &[&str], abort: Option<&str>) -> Planrun {
        let mut command = Command::new(env!("CARGO_BIN_EXE_planrun"));
        command
            .args(args)
            .arg(format!("../outside/{PLAN}"))
            .current_dir(f.path())
            .envs(f.env())
            .env("CARGO_TARGET_DIR", f.outside().join("target"))
            .env("CARGO_NET_OFFLINE", "true")
            .env_remove("PLANRUN_E2E_ABORT")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(abort) = abort {
            command.env("PLANRUN_E2E_ABORT", abort);
        }
        let mut child = command.spawn().unwrap_or_else(|e| panic!("{e}"));
        let (tx, lines) = channel();
        for pipe in [
            child
                .stdout
                .take()
                .map(|p| Box::new(p) as Box<dyn std::io::Read + Send>),
            child
                .stderr
                .take()
                .map(|p| Box::new(p) as Box<dyn std::io::Read + Send>),
        ]
        .into_iter()
        .flatten()
        {
            let tx = tx.clone();
            std::thread::spawn(move || {
                for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                    if tx.send(line).is_err() {
                        return;
                    }
                }
            });
        }
        let stdin = child.stdin.take();
        Planrun {
            child,
            stdin,
            lines,
            seen: Vec::new(),
        }
    }

    /// Reads until a line containing `prompt`; panics after five minutes or at the end.
    fn wait_for(&mut self, prompt: &str) {
        let until = Instant::now() + Duration::from_secs(300);
        loop {
            let left = until.saturating_duration_since(Instant::now());
            match self.lines.recv_timeout(left) {
                Ok(line) => {
                    let hit = line.contains(prompt);
                    self.seen.push(line);
                    if hit {
                        return;
                    }
                }
                Err(_) => panic!("no `{prompt}`; output:\n{}", self.seen.join("\n")),
            }
        }
    }

    /// Waits for `prompt`, then answers `answer`.
    fn answer(&mut self, prompt: &str, answer: &str) {
        self.wait_for(prompt);
        let stdin = self
            .stdin
            .as_mut()
            .unwrap_or_else(|| panic!("stdin closed"));
        writeln!(stdin, "{answer}").unwrap_or_else(|e| panic!("{e}"));
        stdin.flush().unwrap_or_else(|e| panic!("{e}"));
    }

    /// Waits for the process to end; returns its exit code and everything it printed.
    fn finish(mut self) -> (Option<i32>, String) {
        drop(self.stdin.take());
        let status = self.child.wait().unwrap_or_else(|e| panic!("{e}"));
        while let Ok(line) = self.lines.recv_timeout(Duration::from_secs(5)) {
            self.seen.push(line);
        }
        (status.code(), self.seen.join("\n"))
    }

    /// Kills the process, as a crash would.
    fn kill(mut self) -> String {
        self.child.kill().unwrap_or_else(|e| panic!("{e}"));
        let _ = self.child.wait();
        self.seen.join("\n")
    }
}

/// The prompts: a confirmation, a run question, the session notice, an Unknown step.
const CONFIRM: &str = "[y] yes";
const RUN: &str = "[enter] run   [s] skip";
const NOTICE: &str = "[enter] continue   [o] open shell   [q] quit";
const UNKNOWN: &str = "[r] restore and redo";

/// The tree id of `files` committed on top of the fixture, in a fresh copy.
fn expected_tree(files: &[(&str, Option<&str>)]) -> String {
    let f = Fixture::greeter(false);
    for (path, text) in files {
        match text {
            Some(text) => f.write(path, text.as_bytes()),
            None => {
                f.git(&["rm", "-q", path]);
            }
        }
    }
    f.git(&["add", "-A"]);
    f.git(&["commit", "-q", "-m", "expected"]);
    f.git(&["rev-parse", "HEAD^{tree}"]).trim().to_owned()
}

/// The two commits are there, with the subjects and trees of the plan's two tasks.
fn assert_two_commits(f: &Fixture, output: &str) {
    let subjects = f.git(&["log", "--format=%s"]);
    let subjects: Vec<&str> = subjects.lines().collect();
    assert_eq!(
        subjects,
        [
            "P1-T2: re-export greeting, remove the legacy module",
            "P1-T1: the greet module",
            "fixture"
        ],
        "{output}"
    );
    let t1 = expected_tree(&[("src/greet.rs", Some(GREET)), ("src/lib.rs", Some(LIB_T1))]);
    let t2 = expected_tree(&[
        ("src/greet.rs", Some(GREET)),
        ("src/lib.rs", Some(LIB_T2)),
        ("src/old.rs", None),
        ("docs/modules.txt", Some("greet.rs\nlib.rs\n")),
    ]);
    assert_eq!(f.git(&["rev-parse", "HEAD~1^{tree}"]).trim(), t1);
    assert_eq!(f.git(&["rev-parse", "HEAD^{tree}"]).trim(), t2);
    let lib = f.git(&["rev-parse", "HEAD:src/lib.rs"]);
    assert!(lib.starts_with("6b269fc"), "{lib}");
    assert_eq!(f.status(), "", "{output}");
}

fn records(f: &Fixture) -> Vec<Record> {
    journal::read(&f.path().join(".git/planrun/assay-style.jsonl"))
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|l| l.record)
        .collect()
}

/// Across every run of the journal: at most one `step-written` per step, and no
/// `step-prepared` for a step after it was verified on disk.
fn assert_written_once(f: &Fixture) {
    let mut written = Vec::new();
    let mut verified = Vec::new();
    for record in records(f) {
        match record {
            Record::StepWritten { at } => {
                assert!(!written.contains(&at), "written twice: {at:?}");
                written.push(at);
            }
            Record::StepVerified { at } => verified.push(at),
            Record::StepPrepared { at, .. } => {
                assert!(
                    !verified.contains(&at),
                    "prepared again after it was verified: {at:?}"
                );
            }
            _ => {}
        }
    }
}

/// Sets `paths`' mtimes to a sentinel; returns it.
fn sentinel(f: &Fixture, paths: &[&str]) -> SystemTime {
    let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
    for path in paths {
        let file = std::fs::File::options()
            .write(true)
            .open(f.path().join(path))
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        file.set_modified(time).unwrap_or_else(|e| panic!("{e}"));
    }
    time
}

fn assert_untouched(f: &Fixture, paths: &[&str], time: SystemTime) {
    for path in paths {
        let modified = std::fs::metadata(f.path().join(path))
            .and_then(|m| m.modified())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(modified, time, "{path} was written again");
    }
}

/// Leg 1: one run, no interruption. Prompts: the two commits' prose expectations.
#[test]
fn the_assay_plan_runs_to_two_commits() {
    if !pwsh() {
        return;
    }
    for autocrlf in [false, true] {
        let f = fixture(autocrlf);
        let mut p = Planrun::start(&f, &["run", "--auto", AUTO], None);
        p.answer(CONFIRM, "y");
        p.answer(CONFIRM, "y");
        let (code, output) = p.finish();
        assert_eq!(code, Some(0), "{output}");
        assert_two_commits(&f, &output);
        assert_written_once(&f);
    }
}

/// Leg 2: killed at P1-T1's commit question (`--auto read,build`: the commit is Modify), then
/// resumed with `--auto read,build,modify`. Prompts: run (killed); then the session notice and
/// the two confirmations.
#[test]
fn a_run_killed_at_the_commit_question_resumes_without_writing_again() {
    if !pwsh() {
        return;
    }
    let f = fixture(false);
    let mut p = Planrun::start(&f, &["run", "--auto", "read,build"], None);
    p.wait_for(RUN);
    p.kill();
    let written = ["src/greet.rs", "src/lib.rs"];
    let time = sentinel(&f, &written);
    let mut p = Planrun::start(&f, &["resume", "--auto", AUTO], None);
    p.answer(NOTICE, "");
    p.answer(CONFIRM, "y");
    p.answer(CONFIRM, "y");
    let (code, output) = p.finish();
    assert_eq!(code, Some(0), "{output}");
    assert!(
        output.contains("done on disk, not written again"),
        "{output}"
    );
    assert_eq!(
        f.git(&["rev-parse", "HEAD~1:src/greet.rs"]).trim(),
        f.git(&["hash-object", "src/greet.rs"]).trim()
    );
    assert_two_commits(&f, &output);
    assert_written_once(&f);
    assert_untouched(&f, &["src/greet.rs"], time);
}

/// Leg 3: ended right after P1-T2's write-ahead record (nothing written). Prompts: one
/// confirmation (P1-T1's commit); then the notice and P1-T2's confirmation.
#[test]
fn a_crash_after_the_write_ahead_record_writes_the_step_once() {
    if !pwsh() {
        return;
    }
    let f = fixture(false);
    let mut p = Planrun::start(&f, &["run", "--auto", AUTO], Some("step-prepared@P1-T2"));
    p.answer(CONFIRM, "y");
    let (code, output) = p.finish();
    assert_eq!(code, Some(134), "{output}");
    assert!(
        f.read("src/old.rs").is_some(),
        "nothing of P1-T2 was written"
    );
    let mut p = Planrun::start(&f, &["resume"], None);
    p.answer(NOTICE, "");
    p.answer(CONFIRM, "y");
    let (code, output) = p.finish();
    assert_eq!(code, Some(0), "{output}");
    let prepared = records(&f)
        .iter()
        .filter(|r| matches!(r, Record::StepPrepared { at, .. } if at.task == "P1-T2"))
        .count();
    assert_eq!(prepared, 2);
    assert_two_commits(&f, &output);
    assert_written_once(&f);
}

/// Leg 4: ended after the first of P1-T2 step 1's two writes. Prompts: one confirmation; then
/// the Unknown step (restore), the notice, and P1-T2's confirmation.
#[test]
fn a_crash_between_two_writes_is_restored_and_completed() {
    if !pwsh() {
        return;
    }
    let f = fixture(true);
    let mut p = Planrun::start(&f, &["run", "--auto", AUTO], Some("write1@P1-T2"));
    p.answer(CONFIRM, "y");
    let (code, output) = p.finish();
    assert_eq!(code, Some(134), "{output}");
    let mut p = Planrun::start(&f, &["resume"], None);
    p.answer(UNKNOWN, "r");
    p.answer(NOTICE, "");
    p.answer(CONFIRM, "y");
    let (code, output) = p.finish();
    assert_eq!(code, Some(0), "{output}");
    assert!(
        records(&f)
            .iter()
            .any(|r| matches!(r, Record::StepRestored { .. }))
    );
    assert_two_commits(&f, &output);
    assert_written_once(&f);
}

/// Leg 5: ended right after P1-T1's Commit `action-start`: HEAD did not move, so the commit
/// runs on resume. Prompts: none; then the notice and the two confirmations.
#[test]
fn a_crash_after_the_commit_s_start_runs_the_commit_on_resume() {
    if !pwsh() {
        return;
    }
    let f = fixture(false);
    let p = Planrun::start(&f, &["run", "--auto", AUTO], Some("commit-start@P1-T1"));
    let (code, output) = p.finish();
    assert_eq!(code, Some(134), "{output}");
    assert_eq!(f.git(&["log", "--format=%s"]).trim(), "fixture");
    let mut p = Planrun::start(&f, &["status"], None);
    p.wait_for("[>] P1-T1");
    let (code, _) = p.finish();
    assert_eq!(code, Some(0));
    let mut p = Planrun::start(&f, &["resume"], None);
    p.answer(NOTICE, "");
    p.answer(CONFIRM, "y");
    p.answer(CONFIRM, "y");
    let (code, output) = p.finish();
    assert_eq!(code, Some(0), "{output}");
    assert_two_commits(&f, &output);
    assert_written_once(&f);
}

/// `run` refuses a plan with an unfinished run, and `--restart` keeps its journal aside.
#[test]
fn run_refuses_an_unfinished_run_and_restart_keeps_its_journal() {
    if !pwsh() {
        return;
    }
    let f = fixture(false);
    let mut p = Planrun::start(&f, &["run"], None);
    p.answer(RUN, "q");
    let (code, output) = p.finish();
    assert_eq!(code, Some(1), "{output}");
    let p = Planrun::start(&f, &["run", "--auto", AUTO], None);
    let (code, output) = p.finish();
    assert_eq!(code, Some(2), "{output}");
    assert!(output.contains("planrun resume"), "{output}");
    let p = Planrun::start(&f, &["run", "--restart"], None);
    let (_, output) = p.finish();
    assert!(output.contains("is kept as"), "{output}");
    let kept = std::fs::read_dir(f.path().join(".git/planrun"))
        .unwrap_or_else(|e| panic!("{e}"))
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("assay-style."))
        .count();
    assert_eq!(kept, 2);
}
