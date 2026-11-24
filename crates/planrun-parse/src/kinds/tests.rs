//! The built-in kinds through the assay spec: outline and recognition only, no check rules.

use planrun_model::{
    Action, ActionKind, ExitExpect, ManualReason, Script, ScriptIntent, Shell, WriteMode,
};

use crate::document::Document;
use crate::recognize::{TaskCoverage, recognize};
use crate::{Registry, source};

struct Recognized {
    doc: Document,
    actions: Vec<Action>,
    coverage: TaskCoverage,
}

/// One task with the manifest `files` and one step holding `step`.
fn plan(files: &str, step: &str) -> String {
    format!("## Tasks\n\n### P1-T1: t\n\n**Files**\n\n{files}\n\n**Step 1 — s**\n\n{step}")
}

fn run(text: &str) -> Result<Recognized, Vec<&'static str>> {
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let source = source::load("t.md", text.as_bytes()).unwrap_or_else(|e| panic!("{e:?}"));
    let doc = Document::build(source).unwrap_or_else(|e| panic!("{e:?}"));
    let format = registry
        .select(&doc, Some("assay"))
        .unwrap_or_else(|e| panic!("{e:?}"));
    let outline = format.outline(&doc).unwrap_or_else(|e| panic!("{e:?}"));
    let (tasks, coverage) =
        recognize(format.as_ref(), &doc, &outline).map_err(|d| d.error_codes())?;
    let actions = tasks
        .into_iter()
        .flat_map(|t| t.steps)
        .flat_map(|s| s.actions)
        .collect();
    let coverage = coverage
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no task"));
    Ok(Recognized {
        doc,
        actions,
        coverage,
    })
}

fn ok(text: &str) -> Recognized {
    run(text).unwrap_or_else(|codes| panic!("{codes:?}"))
}

fn errors(text: &str) -> Vec<&'static str> {
    run(text).err().unwrap_or_default()
}

fn script(action: &Action) -> &Script {
    match &action.kind {
        ActionKind::RunScript(s) | ActionKind::Commit(s) => s,
        other => panic!("not a script: {other:?}"),
    }
}

const PATCH: &str = "```diff
diff --git a/src/a.rs b/src/a.rs
index 1111111..2222222 100644
--- a/src/a.rs
+++ b/src/a.rs
@@ -1 +1 @@
-a
+b
```";

const CEREMONY: &str = "```powershell
git apply --whitespace=nowarn \"$env:TEMP\\p.patch\"
Remove-Item \"$env:TEMP\\p.patch\"
```";

#[test]
fn full_contents_take_the_mode_from_the_manifest() {
    let step = "File `a.rs` (full contents):\n\n```rust\nA\n```\n\n\
                File `b.rs` (full contents):\n\n```rust\nB\n```\n\n\
                File `c.rs` (full contents):\n\n```rust\nC\n```\n";
    let r = ok(&plan("- Create `a.rs`\n- Modify `b.rs`", step));
    let modes: Vec<(String, WriteMode, String)> = r
        .actions
        .iter()
        .map(|a| match &a.kind {
            ActionKind::WriteFile { path, body, mode } => {
                (path.to_string(), *mode, r.doc.text(body).to_owned())
            }
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        modes,
        [
            ("a.rs".to_owned(), WriteMode::Create, "A\n".to_owned()),
            ("b.rs".to_owned(), WriteMode::Overwrite, "B\n".to_owned()),
            ("c.rs".to_owned(), WriteMode::FromManifest, "C\n".to_owned()),
        ]
    );
    assert_eq!(r.coverage.actions, 6);
    assert_eq!(r.coverage.manifest, 3);
}

#[test]
fn a_patch_absorbs_its_ceremony() {
    let lead = "File `src/a.rs` (exact diff; save the block as `$env:TEMP\\p (1).patch`, outside \
                the repository, then apply it):";
    let r = ok(&plan(
        "- Modify `src/a.rs`",
        &format!("{lead}\n\n{PATCH}\n\n{CEREMONY}\n"),
    ));
    assert_eq!(r.actions.len(), 1);
    let ActionKind::ApplyPatch {
        path, pre, post, ..
    } = &r.actions[0].kind
    else {
        panic!("{:?}", r.actions[0].kind);
    };
    assert_eq!(path.as_str(), "src/a.rs");
    assert_eq!(pre.map(|i| i.to_string()).as_deref(), Some("1111111"));
    assert_eq!(post.map(|i| i.to_string()).as_deref(), Some("2222222"));
    assert_eq!(r.coverage.absorbed.len(), 1);
    assert_eq!(
        &*r.coverage.absorbed[0].recognizer.0,
        "ceremony#git-apply-temp"
    );
}

#[test]
fn a_ceremony_not_after_a_patch_is_a_script() {
    let r = ok(&plan("- Modify `a.rs`", &format!("{CEREMONY}\n")));
    assert!(matches!(r.actions[0].kind, ActionKind::RunScript(_)));
    assert!(r.coverage.absorbed.is_empty());
}

#[test]
fn broken_file_blocks_are_errors() {
    let files = "- Modify `src/a.rs`";
    let lead = "File `src/a.rs` (exact diff; save it):";
    assert_eq!(
        errors(&plan(
            files,
            "File `src/a.rs` (partial contents):\n\n```rust\nx\n```\n"
        )),
        ["unknown-file-op"]
    );
    assert_eq!(
        errors(&plan(
            files,
            "File `src/a.rs` (full contents):\n\nForgotten.\n"
        )),
        ["missing-block"]
    );
    let other = PATCH.replace("src/a.rs", "src/b.rs");
    assert_eq!(
        errors(&plan(files, &format!("{lead}\n\n{other}\n"))),
        ["patch-path"]
    );
    assert_eq!(
        errors(&plan(
            files,
            &format!("{lead}\n\n```diff\nnot a patch\n```\n")
        )),
        ["patch-shape"]
    );
    assert_eq!(
        errors(&plan(
            files,
            "File `../a.rs` (full contents):\n\n```rust\nx\n```\n"
        )),
        ["bad-path"]
    );
}

#[test]
fn deletes_and_generated_files() {
    let step = "Delete `src/old.rs`.\n\n\
                `docs/a.txt` is generated by step 2's first command, never written by hand.\n\n\
                `Cargo.lock` changes: it is regenerated by cargo in the next step. Do not edit it \
                by hand; commit it.\n\n\
                Restore `web/lock.json` from the verified scratch commit `abc` (never written by \
                hand; step 2 checks it):\n\n```powershell\ngit restore --source=abc -- web/lock.json\n```\n";
    let r = ok(&plan("- Delete `src/old.rs`", step));
    let kinds: Vec<String> = r
        .actions
        .iter()
        .map(|a| match &a.kind {
            ActionKind::Delete { path, via_git } => format!("delete {path} {via_git}"),
            ActionKind::Generated { path } => format!("generated {path}"),
            ActionKind::RunScript(s) => format!("script {:?}", s.intent),
            other => format!("{other:?}"),
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "delete src/old.rs false",
            "generated docs/a.txt",
            "generated Cargo.lock",
            "generated web/lock.json",
            "script Verify",
        ]
    );
}

#[test]
fn script_intents() {
    let step = "```powershell\ncargo test; $gate = $LASTEXITCODE\n```\n\n\
                ```bash\ncargo build\n```\n\n\
                ```powershell\nif ($gate -eq 0) {\n  git add -- a\n  if ($LASTEXITCODE -eq 0) { git commit -m \"x\" }\n}\n```\n";
    let r = ok(&plan("- Modify `a`", step));
    let shapes: Vec<(Shell, ScriptIntent, bool)> = r
        .actions
        .iter()
        .map(|a| {
            let commit = matches!(a.kind, ActionKind::Commit(_));
            (script(a).shell, script(a).intent, commit)
        })
        .collect();
    assert_eq!(
        shapes,
        [
            (Shell::Pwsh, ScriptIntent::Gate, false),
            (Shell::Bash, ScriptIntent::Verify, false),
            (Shell::Pwsh, ScriptIntent::Other, true),
        ]
    );
}

#[test]
fn expected_blocks_amend_their_script() {
    let step = "```powershell\ncargo test\n```\n\n\
                Expected (exit code 0; summary lines):\n\n```text\nok\n```\n\n\
                ```powershell\nrg x\n```\n\n\
                Expected (exit code 1; summary lines):\n\n```text\n```\n";
    let r = ok(&plan("- Modify `a`", step));
    let expects: Vec<(ExitExpect, Option<String>)> = r
        .actions
        .iter()
        .map(|a| {
            let e = script(a)
                .expect
                .clone()
                .unwrap_or_else(|| panic!("no expectation"));
            (e.exit, e.lines.map(|l| r.doc.text(&l).to_owned()))
        })
        .collect();
    assert_eq!(
        expects,
        [
            (ExitExpect::Success, Some("ok\n".to_owned())),
            (ExitExpect::Code(1), Some(String::new())),
        ]
    );
    assert_eq!(r.coverage.actions, 6);
}

#[test]
fn broken_expectations_are_errors() {
    let files = "- Modify `a`";
    let script = "```powershell\ncargo test\n```\n\n";
    let block = "Expected (exit code 0; summary lines):\n\n```text\nok\n```\n";
    assert_eq!(
        errors(&plan(
            files,
            &format!("{script}Expected (summary lines):\n\n```text\nok\n```\n")
        )),
        ["expected-spec"]
    );
    assert_eq!(
        errors(&plan(
            files,
            &format!("{script}Expected (exit code 0):\n\n```powershell\nx\n```\n")
        )),
        ["missing-block"]
    );
    assert_eq!(errors(&plan(files, block)), ["expected-without-script"]);
    assert_eq!(
        errors(&plan(
            files,
            &format!("{script}{block}\nExpected: all pass.\n")
        )),
        ["expected-twice"]
    );
}

#[test]
fn prose_expectations_are_first_match() {
    let cases = [
        (
            "Expected: the commit, then no output from `git status --porcelain`.",
            ExitExpect::Unknown,
        ),
        ("Expected: all tests pass.", ExitExpect::Success),
        ("Expected: a build error.", ExitExpect::Failure),
        ("Expected: something else.", ExitExpect::Unknown),
    ];
    for (sentence, exit) in cases {
        let r = ok(&plan(
            "- Modify `a`",
            &format!("```powershell\nx\n```\n\n{sentence}\n"),
        ));
        let e = script(&r.actions[0])
            .expect
            .clone()
            .unwrap_or_else(|| panic!("{sentence}"));
        assert_eq!(e.exit, exit, "{sentence}");
        let prose = e.prose.map(|p| r.doc.text(&p).to_owned());
        assert_eq!(prose.as_deref(), sentence.strip_prefix("Expected: "));
    }
}

#[test]
fn the_fallback_accounts_for_every_other_block() {
    let step = "Plain prose.\n\nRun it by hand.\n\n```rust\nfn x() {}\n```\n\n```text\nout\n```\n\n\
                ```\nbare\n```\n\n<!-- a comment -->\n\n| a |\n|---|\n| 1 |\n";
    let r = ok(&plan("- Modify `a`", step));
    let kinds: Vec<String> = r
        .actions
        .iter()
        .map(|a| match &a.kind {
            ActionKind::Note { .. } => "note".to_owned(),
            ActionKind::Manual { reason, .. } => format!("manual {reason:?}"),
            other => format!("{other:?}"),
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "note",
            "manual Marked",
            "manual ForeignLanguage(\"rust\")",
            "note",
            "manual Unrecognized",
            "note",
        ]
    );
    assert_eq!(r.coverage.dropped, 1);
    assert_eq!((r.coverage.notes, r.coverage.manual), (3, 3));
    assert!(matches!(
        &r.actions[1].kind,
        ActionKind::Manual {
            reason: ManualReason::Marked,
            ..
        }
    ));
}

#[test]
fn kind_parameters_are_validated_when_the_spec_loads() {
    let base = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../formats/assay.toml"
    ))
    .unwrap_or_else(|e| panic!("{e}"));
    let base = base.replace("id = \"assay\"", "id = \"x\"");
    let refused = |extra: &str| {
        let mut registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
        registry
            .add_spec("x", &format!("{base}\n[[recognizer]]\n{extra}\n"))
            .err()
            .map(|e| e.to_string())
    };
    assert_eq!(
        refused("kind = \"delete\"\nid = \"d\"\nlead = 'x'\nextra = 1"),
        Some("format `x`: recognizer `delete`: lead: the regex has no `path` capture".to_owned())
    );
    assert_eq!(
        refused("kind = \"delete\"\nid = \"d\"\nlead = '(?P<path>x)'\nextra = 1"),
        Some("format `x`: recognizer `delete`: unknown parameter `extra`".to_owned())
    );
    assert!(refused("kind = \"file-block\"\nid = \"f\"\nlead = '(?P<path>x)(?P<op>y)'\nops = { y = \"move\" }").is_some());
    assert!(
        refused("kind = \"script\"\nid = \"s\"\nintents = [{ intent = \"deploy\", body = 'x' }]")
            .is_some()
    );
    assert!(refused("kind = \"expected-block\"\nid = \"e\"\nlead = '(?P<spec>x)'\nexit = [{ spec = 'x', exit = \"code\" }]").is_some());
    assert!(refused("kind = \"ceremony\"\nid = \"c\"\nafter = \"script\"\nbody = 'x'").is_some());
    assert_eq!(
        refused("kind = \"delete\"\nid = \"d\"\nlead = '(?P<path>x)'"),
        None
    );
}
