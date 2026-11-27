//! The journal's writer and reader (D76): one line per record, a torn tail dropped and cut off
//! before the next append, a bad line in the middle an error.

use super::*;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "planrun-journal-{}-{name}-{}",
        std::process::id(),
        now_ms()
    ));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    dir
}

fn head() -> Head {
    Head {
        commit: Some("0123456789abcdef0123456789abcdef01234567".to_owned()),
        reference: Some("refs/heads/main".to_owned()),
    }
}

fn task_start(task: &str) -> Record {
    Record::TaskStart {
        task: task.to_owned(),
        head: head(),
    }
}

fn records(path: &Path) -> Vec<Record> {
    read(path)
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|l| l.record)
        .collect()
}

#[test]
fn records_round_trip_one_per_line_with_the_schema_version() {
    let dir = temp_dir("round");
    let path = path_for(&dir, "plan");
    let mut j = Journal::open(&path).unwrap_or_else(|e| panic!("{e}"));
    let at = At {
        task: "P1-T1".to_owned(),
        step: 0,
        action: 1,
    };
    let all = [
        task_start("P1-T1"),
        Record::ActionStart {
            at: at.clone(),
            kind: "commit".to_owned(),
            decision: "Auto".to_owned(),
            head: head(),
            expect: vec![PathBlob {
                path: RepoPath::new("src/a.rs").unwrap_or_else(|e| panic!("{e}")),
                blob: None,
            }],
        },
        Record::RunEnd {
            end: End::Stopped,
            reason: Some("quit".to_owned()),
        },
    ];
    for r in all.clone() {
        j.append(r).unwrap_or_else(|e| panic!("{e}"));
    }
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(text.lines().count(), 3);
    assert!(text.lines().all(|l| l.starts_with("{\"v\":1,")), "{text}");
    assert!(text.contains("\"t\":\"action-start\""), "{text}");
    assert_eq!(records(&path), all);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_torn_last_line_is_dropped_and_cut_before_the_next_append() {
    let dir = temp_dir("torn");
    let path = path_for(&dir, "plan");
    let mut j = Journal::open(&path).unwrap_or_else(|e| panic!("{e}"));
    j.append(task_start("P1-T1"))
        .unwrap_or_else(|e| panic!("{e}"));
    drop(j);
    let whole = serde_json::to_string(&Line {
        v: SCHEMA,
        ms: 1,
        record: task_start("P1-T2"),
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let mut file = OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap_or_else(|e| panic!("{e}"));
    file.write_all(whole.as_bytes())
        .unwrap_or_else(|e| panic!("{e}"));
    drop(file);
    assert_eq!(
        records(&path),
        [task_start("P1-T1")],
        "a parseable tail without \\n"
    );
    let mut j = Journal::open(&path).unwrap_or_else(|e| panic!("{e}"));
    j.append(task_start("P1-T3"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(records(&path), [task_start("P1-T1"), task_start("P1-T3")]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_bad_line_in_the_middle_is_an_error_and_a_missing_file_is_empty() {
    let dir = temp_dir("bad");
    let path = path_for(&dir, "plan");
    assert!(read(&path).unwrap_or_else(|e| panic!("{e}")).is_empty());
    let mut j = Journal::open(&path).unwrap_or_else(|e| panic!("{e}"));
    j.append(task_start("P1-T1"))
        .unwrap_or_else(|e| panic!("{e}"));
    drop(j);
    let mut text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
    text.push_str("{\"v\":1,\"ms\":1,\"t\":\"no-such\"}\n");
    text.push_str(&fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}")));
    fs::write(&path, &text).unwrap_or_else(|e| panic!("{e}"));
    match read(&path) {
        Err(JournalError::Corrupt { line: 2, .. }) => {}
        other => panic!("{other:?}"),
    }
    fs::write(&path, text.replace("\"v\":1", "\"v\":9")).unwrap_or_else(|e| panic!("{e}"));
    assert!(matches!(
        read(&path),
        Err(JournalError::Corrupt { line: 1, .. })
    ));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn the_observer_sees_appends_and_writes() {
    let dir = temp_dir("observer");
    let mut j = Journal::open(&path_for(&dir, "plan")).unwrap_or_else(|e| panic!("{e}"));
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let log = seen.clone();
    j.observe(Box::new(move |seam| {
        log.borrow_mut().push(match seam {
            Seam::Appended(r) => format!("{r:?}")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_owned(),
            Seam::Wrote(n) => format!("wrote {n}"),
        });
    }));
    j.append(task_start("P1-T1"))
        .unwrap_or_else(|e| panic!("{e}"));
    let at = At {
        task: "P1-T1".to_owned(),
        step: 0,
        action: 0,
    };
    j.ahead(&at).after_write(2);
    assert_eq!(*seen.borrow(), ["TaskStart", "wrote 2"]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn every_observer_sees_every_seam_in_the_order_they_were_added() {
    let dir = temp_dir("observers");
    let mut j = Journal::open(&path_for(&dir, "plan")).unwrap_or_else(|e| panic!("{e}"));
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    for name in ["first", "second"] {
        let log = seen.clone();
        j.observe(Box::new(move |seam| {
            let what = match seam {
                Seam::Appended(_) => "appended".to_owned(),
                Seam::Wrote(n) => format!("wrote {n}"),
            };
            log.borrow_mut().push(format!("{name}: {what}"));
        }));
    }
    j.append(task_start("P1-T1"))
        .unwrap_or_else(|e| panic!("{e}"));
    let at = At {
        task: "P1-T1".to_owned(),
        step: 0,
        action: 0,
    };
    j.ahead(&at).after_write(1);
    assert_eq!(
        *seen.borrow(),
        [
            "first: appended",
            "second: appended",
            "first: wrote 1",
            "second: wrote 1"
        ]
    );
    let _ = fs::remove_dir_all(dir);
}
