//! The classifier over the scripts of the real plans in `tests/corpus/local` (git-ignored,
//! decision D16). Ignored by default: `cargo test -p planrun-cli --test local_classes --
//! --ignored --nocapture`. It prints, per plan and in total, how many scripts and commits fall
//! in each class, and which reasons made scripts Destructive. Nothing it prints goes into a
//! committed file or snapshot.

use std::collections::BTreeMap;
use std::path::Path;

use planrun_exec::policy::{DefaultClassifier, PolicyClass};
use planrun_model::ActionKind;
use planrun_parse::{Registry, parse_plan};

#[test]
#[ignore = "needs the private plans in tests/corpus/local"]
fn local_classes() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/local");
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap_or_else(|e| panic!("{e}")).path())
        .filter(|p| p.extension().is_some_and(|x| x == "md") && !p.ends_with("README.md"))
        .collect();
    paths.sort();
    let mut total: BTreeMap<PolicyClass, usize> = BTreeMap::new();
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    for path in &paths {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let parsed = parse_plan(&registry, name, &bytes, None)
            .unwrap_or_else(|d| panic!("{name}: {:?}", d.error_codes()));
        let plan = parsed.plan;
        let mut counts: BTreeMap<PolicyClass, usize> = BTreeMap::new();
        let scripts = plan
            .tasks
            .iter()
            .flat_map(|t| &t.steps)
            .flat_map(|s| &s.actions)
            .filter_map(|a| match &a.kind {
                ActionKind::RunScript(s) | ActionKind::Commit(s) => Some(s),
                _ => None,
            });
        for script in scripts {
            let found = DefaultClassifier.explain(script.shell, plan.text(&script.body));
            *counts.entry(found.class).or_default() += 1;
            *total.entry(found.class).or_default() += 1;
            for reason in found.reasons {
                *reasons.entry(reason).or_default() += 1;
            }
        }
        println!("{name}: {counts:?}");
    }
    println!("total: {total:?}");
    println!("what made scripts Destructive:");
    for (reason, n) in &reasons {
        println!("  {n:>4}  {reason}");
    }
}
