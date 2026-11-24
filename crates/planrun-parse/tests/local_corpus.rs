//! The real plans in `tests/corpus/local` (git-ignored, decision D16). Ignored by default, so CI
//! and fresh clones stay green: `cargo test -- --ignored local_corpus`.
//!
//! Every plan must parse with zero errors, except plans no built-in format detects yet (the
//! three formats of P2), which are listed as skipped. A coverage line per task is printed; run
//! with `--nocapture` to see it.

use std::path::Path;

use planrun_parse::{Registry, parse_plan};

#[test]
#[ignore = "needs the private plans in tests/corpus/local"]
fn local_corpus() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/local");
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap_or_else(|e| panic!("{e}")).path())
        .filter(|p| p.extension().is_some_and(|x| x == "md") && !p.ends_with("README.md"))
        .collect();
    paths.sort();
    let mut failed = Vec::new();
    let mut parsed = 0;
    for path in &paths {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let result = match parse_plan(&registry, name, &bytes, None) {
            Ok(result) => result,
            Err(d) if d.error_codes() == ["no-format"] => {
                println!("{name}: skipped, no built-in format detects it yet");
                continue;
            }
            Err(d) => {
                failed.push(format!("{name}: {:?}", d.error_codes()));
                continue;
            }
        };
        parsed += 1;
        println!(
            "{name}: {} ({} tasks)",
            result.plan.format.0,
            result.plan.tasks.len()
        );
        for c in &result.coverage {
            println!(
                "  {:<12} blocks {:>3}  actions {:>3}  absorbed {:>2}  manifest {:>2}  notes {:>2}  manual {:>2}",
                c.task.raw,
                c.blocks(),
                c.actions,
                c.absorbed.len(),
                c.manifest,
                c.notes,
                c.manual
            );
        }
    }
    assert!(failed.is_empty(), "\n{}", failed.join("\n"));
    println!("{parsed} parsed, {} skipped", paths.len() - parsed);
}
