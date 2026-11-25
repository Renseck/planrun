//! Every plan in `tests/corpus/invalid` fails with exactly the error code on its first line
//! (`<!-- expect: <code> -->`, tests/corpus/invalid/README.md). Codes are pinned, messages are
//! not.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use planrun_parse::{Registry, parse_plan};

fn invalid_plans() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/invalid");
    let mut plans: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap_or_else(|e| panic!("{e}")).path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .filter(|p| !p.ends_with("README.md"))
        .collect();
    plans.sort();
    plans
}

fn expected_code(text: &str) -> Option<&str> {
    text.lines()
        .next()?
        .strip_prefix("<!-- expect: ")?
        .strip_suffix(" -->")
}

#[test]
fn every_invalid_plan_yields_exactly_its_code() {
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let plans = invalid_plans();
    assert!(plans.len() >= 20, "only {} invalid plans", plans.len());
    let mut wrong = Vec::new();
    for path in &plans {
        let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let text = String::from_utf8_lossy(&bytes);
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let Some(expected) = expected_code(&text) else {
            wrong.push(format!("{name}: no `<!-- expect: … -->` first line"));
            continue;
        };
        let codes: BTreeSet<&str> = match parse_plan(&registry, name, &bytes, None) {
            Ok(_) => BTreeSet::new(),
            Err(d) => d.error_codes().into_iter().collect(),
        };
        if codes != BTreeSet::from([expected]) {
            wrong.push(format!("{name}: expected {{{expected}}}, got {codes:?}"));
        }
    }
    assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
}

/// `<code>.md`, or `<code>.<case>.md` for a second case of one code (`bad-path.git-hooks.md`).
#[test]
fn the_file_name_is_the_code() {
    for path in invalid_plans() {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
        let stem = path.file_stem().and_then(|s| s.to_str());
        let code = stem.map(|s| s.split_once('.').map_or(s, |(code, _)| code));
        assert_eq!(expected_code(&text), code, "{}", path.display());
    }
}
