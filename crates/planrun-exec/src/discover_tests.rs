use super::*;
use crate::fixture::{Fixture, canonical};

fn open(dir: &Path, user: Option<&Path>) -> Discovery {
    Discovery::open(dir, user).unwrap_or_else(|e| panic!("{e}"))
}

fn code(dir: &Path, user: Option<&Path>) -> &'static str {
    match Discovery::open(dir, user) {
        Ok(_) => panic!("discovery in {} did not fail", dir.display()),
        Err(e) => e.code(),
    }
}

fn write(path: &Path, bytes: &[u8]) {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("{e}"));
    }
    std::fs::write(path, bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

fn globs(d: &Discovery) -> Vec<&str> {
    d.config().globs.iter().map(PathPattern::as_str).collect()
}

#[test]
fn the_root_is_the_nearest_directory_with_a_git_entry() {
    let f = Fixture::greeter(false);
    let found = open(&f.path().join("src"), None);
    assert_eq!(found.root().map(canonical), Some(canonical(f.path())));
    let worktree = f.outside().join("wt");
    write(&worktree.join(".git"), b"gitdir: elsewhere\n");
    std::fs::create_dir_all(worktree.join("a/b")).unwrap_or_else(|e| panic!("{e}"));
    let found = open(&worktree.join("a/b"), None);
    assert_eq!(found.root().map(canonical), Some(canonical(&worktree)));
}

#[test]
fn outside_a_repository_there_is_no_repository_layer() {
    let f = Fixture::greeter(false);
    let dir = f.outside().join("plain");
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    if find_root(&dir).is_some() {
        eprintln!("skipped: a directory above the OS temp directory holds a .git");
        return;
    }
    let user = f.outside().join("user");
    write(&user.join("formats/mine.toml"), b"id = \"mine\"\n");
    let found = open(&dir, Some(&user));
    assert_eq!(found.root(), None);
    assert_eq!(globs(&found), DEFAULT_GLOBS);
    let ids: Vec<&str> = found.formats().iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, ["mine"]);
}

#[test]
fn formats_come_from_the_user_layer_then_the_repository_layer() {
    let f = Fixture::greeter(false);
    let user = f.outside().join("user");
    write(
        &user.join("formats/zeta.toml"),
        b"\xEF\xBB\xBFid = \"zeta\"\n",
    );
    f.write(".planrun/formats/b.toml", b"id = \"b\"\n");
    f.write(".planrun/formats/a.toml", b"id = \"a\"\r\n");
    f.write(".planrun/formats/notes.txt", b"not a spec");
    f.write(".planrun/formats/old/c.toml", b"id = \"c\"\n");
    let found = open(f.path(), Some(&user));
    let seen: Vec<(&str, &str, bool)> = found
        .formats()
        .iter()
        .map(|s| {
            let user = matches!(s.source, FormatSource::User(_));
            (s.id.as_str(), s.text.as_str(), user)
        })
        .collect();
    assert_eq!(
        seen,
        [
            ("zeta", "id = \"zeta\"\n", true),
            ("a", "id = \"a\"\r\n", false),
            ("b", "id = \"b\"\n", false),
        ]
    );
    let missing = f.outside().join("nobody");
    assert_eq!(open(f.path(), Some(&missing)).formats().len(), 2);
}

#[test]
fn a_spec_file_that_cannot_be_used_stops_discovery() {
    let f = Fixture::greeter(false);
    f.write(".planrun/formats/my format.toml", b"id = \"x\"\n");
    assert_eq!(code(f.path(), None), "format-file-name");
    std::fs::remove_file(f.path().join(".planrun/formats/my format.toml"))
        .unwrap_or_else(|e| panic!("{e}"));
    f.write(".planrun/formats/latin.toml", b"id = \"caf\xE9\"\n");
    assert_eq!(code(f.path(), None), "format-io");
}

#[test]
fn without_a_config_file_the_default_globs_apply() {
    let f = Fixture::greeter(false);
    let found = open(f.path(), None);
    assert_eq!(globs(&found), DEFAULT_GLOBS);
    assert_eq!(found.config().file, None);
    f.write(CONFIG_FILE, b"# nothing yet\n");
    assert_eq!(globs(&open(f.path(), None)), DEFAULT_GLOBS);
}

#[test]
fn the_config_file_s_globs_replace_the_defaults() {
    let f = Fixture::greeter(false);
    f.write(
        CONFIG_FILE,
        b"\xEF\xBB\xBF[plans]\r\nglobs = [\"work/**/plans/*.md\", \"notes/*.md\"]\r\n",
    );
    let found = open(f.path(), None);
    assert_eq!(globs(&found), ["work/**/plans/*.md", "notes/*.md"]);
    let file = found.config().file.as_deref().map(canonical);
    assert_eq!(file, Some(canonical(&f.path().join(CONFIG_FILE))));
}

#[rustfmt::skip]
#[test]
fn a_config_file_that_cannot_be_used_stops_discovery() {
    let rows: &[(&str, &str)] = &[
        ("[plans]\nglobs = []\n",              "config-globs-empty"),
        ("[plans]\nglobs = [\"a/b**/*.md\"]\n", "config-glob"),
        ("[plans]\nglobs = [\"../*.md\"]\n",    "config-glob"),
        ("[plans]\nglob = [\"a/*.md\"]\n",      "config-parse"),
        ("[run]\nformat = \"assay\"\n",         "config-parse"),
        ("[plans\n",                            "config-parse"),
    ];
    let f = Fixture::greeter(false);
    for &(text, expected) in rows {
        f.write(CONFIG_FILE, text.as_bytes());
        assert_eq!(code(f.path(), None), expected, "{text}");
    }
}

#[rustfmt::skip]
#[test]
fn the_user_directory_comes_from_the_environment() {
    type Vars<'a> = &'a [(&'a str, &'a str)];
    let rows: &[(Vars, bool, Option<&str>)] = &[
        (&[("PLANRUN_CONFIG_DIR", "/cfg"), ("APPDATA", "/app")], true,  Some("/cfg")),
        (&[("APPDATA", "/app")],                                  true,  Some("/app/planrun")),
        (&[("HOME", "/home/r")],                                  true,  None),
        (&[("XDG_CONFIG_HOME", "/xdg"), ("HOME", "/home/r")],     false, Some("/xdg/planrun")),
        (&[("XDG_CONFIG_HOME", ""), ("HOME", "/home/r")],         false, Some("/home/r/.config/planrun")),
        (&[],                                                     false, None),
    ];
    for &(vars, windows, expected) in rows {
        let var = |name: &str| vars.iter().find(|(k, _)| *k == name).map(|(_, v)| OsString::from(v));
        assert_eq!(user_dir_from(var, windows), expected.map(PathBuf::from), "{vars:?}");
    }
}

/// The greeter fixture with plan files tracked, untracked, ignored, deleted and elsewhere.
fn with_plans() -> Fixture {
    let f = Fixture::greeter(false);
    f.write(".gitignore", b"local/\n");
    f.write("docs/plans/2026-01-01-p1-greet.md", b"# P1\n");
    f.write("docs/plans/gone.md", b"# gone\n");
    f.git(&["add", "-A"]);
    f.git(&["commit", "-q", "-m", "plans"]);
    std::fs::remove_file(f.path().join("docs/plans/gone.md")).unwrap_or_else(|e| panic!("{e}"));
    f.write("docs/plans/2026-01-02-p2-farewell.md", b"# P2\n");
    f.write("docs/plans/README.md", b"# Plans\n");
    f.write("docs/plans/old/2025-p0.md", b"# P0\n");
    f.write("tools/superpowers/x/plans/2026-01-03-p1-tools.md", b"# T\n");
    f.write("local/plans/2026-01-04-private.md", b"# private\n");
    f.write("src/notes.md", b"# notes\n");
    f
}

#[test]
fn plans_are_the_listed_files_a_glob_matches() {
    let f = with_plans();
    let found = open(f.path(), None);
    let plans = found.plans(&f.env()).unwrap_or_else(|e| panic!("{e}"));
    let paths: Vec<&str> = plans.iter().map(RepoPath::as_str).collect();
    assert_eq!(
        paths,
        [
            "docs/plans/2026-01-01-p1-greet.md",
            "docs/plans/2026-01-02-p2-farewell.md",
            "docs/plans/README.md",
            "tools/superpowers/x/plans/2026-01-03-p1-tools.md",
        ]
    );
    f.write(CONFIG_FILE, b"[plans]\nglobs = [\"docs/plans/**/*.md\"]\n");
    let narrowed = open(f.path(), None)
        .plans(&f.env())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(narrowed.len(), 4);
    assert!(
        narrowed
            .iter()
            .any(|p| p.as_str() == "docs/plans/old/2025-p0.md")
    );
}

#[cfg(unix)]
#[test]
fn a_symbolic_link_is_no_plan() {
    let f = with_plans();
    std::os::unix::fs::symlink(
        f.path().join("docs/plans/2026-01-01-p1-greet.md"),
        f.path().join("docs/plans/link.md"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let plans = open(f.path(), None)
        .plans(&f.env())
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(plans.iter().all(|p| p.as_str() != "docs/plans/link.md"));
}

#[test]
fn a_name_finds_one_plan_by_its_stem() {
    let f = with_plans();
    let found = open(f.path(), None);
    let resolve = |arg: &str| found.resolve(Path::new(arg), &f.env());
    let one = resolve("FAREWELL").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(one.shown, "docs/plans/2026-01-02-p2-farewell.md");
    assert!(one.by_name);
    assert_eq!(
        canonical(&one.path),
        canonical(&f.path().join("docs/plans/2026-01-02-p2-farewell.md"))
    );
    let code = |arg: &str| resolve(arg).err().map(|e| e.code());
    assert_eq!(code("p1"), Some("plan-ambiguous"));
    assert_eq!(code("private"), Some("plan-not-found"));
    assert_eq!(code("gone"), Some("plan-not-found"));
    let message = resolve("p1").err().map(|e| e.to_string());
    assert_eq!(
        message.as_deref(),
        Some(
            "`p1` names 2 plan files: docs/plans/2026-01-01-p1-greet.md, \
             tools/superpowers/x/plans/2026-01-03-p1-tools.md; give more of the name, or the path"
        )
    );
}

#[test]
fn a_path_is_used_as_given() {
    let f = with_plans();
    let found = open(f.path(), None);
    for arg in ["p1.md", "docs/p1", "../elsewhere/plan.txt"] {
        let resolved = found
            .resolve(Path::new(arg), &f.env())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(resolved.path, PathBuf::from(arg));
        assert!(!resolved.by_name);
    }
}

#[test]
fn a_name_outside_a_repository_is_refused() {
    let found = Discovery::default();
    let refused = found.resolve(Path::new("p1"), &[]).err().map(|e| e.code());
    assert_eq!(refused, Some("not-a-repository"));
}
