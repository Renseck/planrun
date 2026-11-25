//! The workspace on a fixture copy: resolution, the writer, links and the test seam.

use std::path::Path;

use planrun_model::RepoPath;

use super::*;
use crate::fixture::Fixture;

fn path(p: &str) -> RepoPath {
    RepoPath::new(p).unwrap_or_else(|e| panic!("{e}"))
}

fn open(f: &Fixture) -> Workspace {
    Workspace::open(f.path(), f.env()).unwrap_or_else(|e| panic!("{e}"))
}

fn temp_files(f: &Fixture) -> Vec<String> {
    f.snapshot()
        .into_keys()
        .filter(|k| k.contains(".planrun-"))
        .collect()
}

/// A directory link: a symlink on Unix, a junction on Windows (no privilege needed).
fn link_dir(link: &Path, target: &Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
            .is_ok_and(|o| o.status.success())
    }
}

/// A file symlink; on Windows it needs a privilege the test may not have.
fn link_file(link: &Path, target: &Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link).is_ok()
    }
}

#[test]
fn a_path_resolves_and_reads() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let lib = ws
        .read(&path("src/lib.rs"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(lib, f.read("src/lib.rs"));
    assert_eq!(ws.read(&path("src/none.rs")).ok(), Some(None));
    let missing = ws
        .resolve(&path("a/b/c.rs"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!((missing.existing, missing.segments), (0, 3));
    assert!(ws.read(&path("src")).is_err(), "a directory is no file");
}

#[test]
fn a_write_creates_its_directories_and_leaves_no_temp_file() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let created = ws
        .write(&path("a/b/c.txt"), b"x\r\n")
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(created.len(), 2);
    assert_eq!(f.read("a/b/c.txt").as_deref(), Some(&b"x\r\n"[..]));
    ws.write(&path("src/lib.rs"), b"replaced\n")
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.read("src/lib.rs").as_deref(), Some(&b"replaced\n"[..]));
    assert!(temp_files(&f).is_empty());
    assert_eq!(ws.remove(&path("a/b/c.txt")).ok(), Some(true));
    assert_eq!(ws.remove(&path("a/b/c.txt")).ok(), Some(false));
}

#[test]
fn a_case_mismatch_is_refused_where_the_disk_ignores_case() {
    let f = Fixture::greeter(false);
    if !f.path().join("SRC/LIB.RS").exists() {
        eprintln!("skipped: this file system is case-sensitive");
        return;
    }
    let ws = open(&f);
    let err = ws.resolve(&path("SRC/lib.rs")).err();
    assert_eq!(err.as_ref().map(WorkspaceError::code), Some("path-case"));
    assert!(ws.write(&path("src/LIB.rs"), b"x").is_err());
    assert_eq!(f.status(), "");
}

#[test]
fn a_linked_directory_is_refused() {
    let f = Fixture::greeter(false);
    let outside = f.outside();
    if !link_dir(&f.path().join("linked"), &outside) {
        eprintln!("skipped: could not create a directory link");
        return;
    }
    let ws = open(&f);
    let err = ws.write(&path("linked/x.txt"), b"x").err();
    assert_eq!(err.as_ref().map(WorkspaceError::code), Some("path-unsafe"));
    assert!(std::fs::read_dir(&outside).map_or(0, |d| d.count()) == 0);
}

#[test]
fn a_target_turned_into_a_link_before_the_rename_is_replaced_not_followed() {
    let f = Fixture::greeter(false);
    let outside = f.outside().join("secret.txt");
    std::fs::write(&outside, b"secret\n").unwrap_or_else(|e| panic!("{e}"));
    let ws = open(&f);
    let target = outside.clone();
    let made = std::rc::Rc::new(std::cell::Cell::new(false));
    let flag = made.clone();
    *ws.faults.hook.borrow_mut() = Some(Box::new(move |full: &Path| {
        let _ = std::fs::remove_file(full);
        flag.set(link_file(full, &target));
    }));
    let written = ws.write(&path("src/lib.rs"), b"new\n");
    if !made.get() {
        eprintln!("skipped: could not create a file symlink");
        return;
    }
    written.unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read(&outside).ok().as_deref(),
        Some(&b"secret\n"[..])
    );
    let meta = std::fs::symlink_metadata(f.path().join("src/lib.rs"));
    assert!(
        meta.is_ok_and(|m| m.is_file()),
        "the link itself was replaced"
    );
    assert_eq!(f.read("src/lib.rs").as_deref(), Some(&b"new\n"[..]));
}

#[test]
fn a_parent_swapped_for_a_link_before_the_rename_is_refused() {
    let f = Fixture::greeter(false);
    let outside = f.outside();
    let ws = open(&f);
    let made = std::rc::Rc::new(std::cell::Cell::new(false));
    let flag = made.clone();
    let target = outside.clone();
    *ws.faults.hook.borrow_mut() = Some(Box::new(move |full: &Path| {
        let Some(parent) = full.parent() else {
            return;
        };
        let moved = parent.with_extension("old");
        if std::fs::rename(parent, &moved).is_ok() {
            flag.set(link_dir(parent, &target));
        }
    }));
    let written = ws.write(&path("src/lib.rs"), b"new\n");
    if !made.get() {
        eprintln!("skipped: could not create a directory link");
        return;
    }
    let err = written.err();
    assert_eq!(err.as_ref().map(WorkspaceError::code), Some("path-unsafe"));
    assert!(std::fs::read_dir(&outside).map_or(0, |d| d.count()) == 0);
}

#[test]
fn an_injected_fault_leaves_nothing_behind() {
    let f = Fixture::greeter(false);
    let ws = open(&f);
    let before = f.snapshot();
    ws.faults.fail_write.set(Some(1));
    let err = ws.write(&path("new/dir/x.txt"), b"x").err();
    assert_eq!(err.as_ref().map(WorkspaceError::code), Some("io"));
    assert_eq!(f.snapshot(), before);
    assert!(!f.path().join("new").exists());
}
