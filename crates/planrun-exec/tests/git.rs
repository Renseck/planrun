//! The git wrapper on a temporary copy of the greeter fixture, with `core.autocrlf` off and on.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;

use fixture::{Fixture, canonical};
use planrun_exec::git::{Attributes, BlobId, Git};
use planrun_model::{AbbrevBlobId, RepoPath};

fn path(p: &str) -> RepoPath {
    RepoPath::new(p).unwrap_or_else(|e| panic!("{e}"))
}

fn git(f: &Fixture) -> Git {
    Git::new(f.path(), f.env())
}

fn abbrev(hex: &str) -> AbbrevBlobId {
    AbbrevBlobId::parse(hex).unwrap_or_else(|e| panic!("{e}"))
}

/// The P1-T1 patch of `tests/corpus/synthetic/assay-style.md`.
const PATCH: &str = "diff --git a/src/lib.rs b/src/lib.rs
index aa80cf1..97bd92a 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,8 +1,9 @@
 //! A tiny fixture crate for the planrun corpus.
\x20
+pub mod greet;
 pub mod old;
\x20
 /// The greeting for `name`.
 pub fn greeting(name: &str) -> String {
-    old::legacy_greeting(name)
+    greet::greeting(name)
 }
";

#[test]
fn the_copy_is_a_repository_of_its_own() {
    let f = Fixture::greeter(false);
    let top = Git::toplevel(f.path(), &f.env()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(canonical(&top), canonical(f.path()));
    assert_eq!(
        git(&f).config_bool("core.autocrlf").ok().flatten(),
        Some(false)
    );
}

#[test]
fn blob_ids_are_git_s_view_with_and_without_autocrlf() {
    for autocrlf in [false, true] {
        let f = Fixture::greeter(autocrlf);
        let bytes = f.read("src/lib.rs").unwrap_or_default();
        assert_eq!(bytes.windows(2).any(|w| w == b"\r\n"), autocrlf);
        let g = git(&f);
        let filtered = g
            .hash_object(&path("src/lib.rs"), &bytes, true, false, &[])
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(filtered.matches(&abbrev("aa80cf1")), "{filtered}");
        let raw = g
            .hash_object(&path("src/lib.rs"), &bytes, false, false, &[])
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(raw == filtered, !autocrlf);
    }
}

#[test]
fn a_patch_checks_against_the_working_tree_with_and_without_autocrlf() {
    for autocrlf in [false, true] {
        let f = Fixture::greeter(autocrlf);
        let before = f.snapshot();
        git(&f)
            .apply(PATCH.as_bytes(), false, true, &[])
            .unwrap_or_else(|e| panic!("autocrlf={autocrlf}: {e}"));
        assert_eq!(f.snapshot(), before, "--check wrote something");
    }
}

#[test]
fn status_reports_dirt_and_writes_nothing() {
    let f = Fixture::greeter(true);
    let g = git(&f);
    let before = f.snapshot();
    assert!(g.status(None).unwrap_or_else(|e| panic!("{e}")).is_empty());
    assert_eq!(f.snapshot(), before);
    f.write("src/lib.rs", b"changed\n");
    f.write("docs/new file.txt", b"x\n");
    let all = g.status(None).unwrap_or_else(|e| panic!("{e}"));
    let codes: Vec<_> = all
        .iter()
        .map(|e| (e.code.as_str(), e.path.as_str()))
        .collect();
    assert_eq!(codes, [(" M", "src/lib.rs"), ("??", "docs/new file.txt")]);
    let one = g
        .status(Some(&[&path("src/old.rs")]))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(one.is_empty());
}

#[test]
fn objects_round_trip_through_cat_file() {
    let f = Fixture::greeter(true);
    let g = git(&f);
    let crlf = b"a\r\nb\r\n";
    let raw = g
        .hash_object(&path("x.txt"), crlf, false, true, &[])
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(g.cat_blob(&raw, &[]).ok().as_deref(), Some(&crlf[..]));
    let lf = g
        .hash_object(&path("x.txt"), b"a\nb\n", false, true, &[])
        .unwrap_or_else(|e| panic!("{e}"));
    let checkout = g
        .cat_filtered(&lf, &path("x.txt"), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(checkout, crlf, "autocrlf=true checks text out with CRLF");
}

#[test]
fn attributes_report_eol_and_text() {
    let f = Fixture::greeter(false);
    f.write(
        ".gitattributes",
        b"*.crlf eol=crlf\n*.lf eol=lf\n*.bin binary\n",
    );
    let paths = [path("a.crlf"), path("a.lf"), path("a.bin"), path("a.rs")];
    let refs: Vec<&RepoPath> = paths.iter().collect();
    let found = git(&f).attributes(&refs).unwrap_or_else(|e| panic!("{e}"));
    let attr = |eol: Option<&str>, text: bool| Attributes {
        eol: eol.map(str::to_owned),
        text,
    };
    assert_eq!(
        found,
        [
            attr(Some("crlf"), true),
            attr(Some("lf"), true),
            attr(None, false),
            attr(None, true)
        ]
    );
}

#[test]
fn the_index_entry_of_a_tracked_file() {
    let f = Fixture::greeter(false);
    let entry = git(&f)
        .index_entry(&path("src/lib.rs"), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    let (mode, id) = entry.unwrap_or_else(|| panic!("src/lib.rs is tracked"));
    assert_eq!(mode, "100644");
    assert!(id.matches(&abbrev("aa80cf1")));
    let none = git(&f)
        .index_entry(&path("nothing.rs"), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(none, None::<(String, BlobId)>);
}
