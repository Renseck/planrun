//! Every `formats/*.toml` deserializes into `FormatSpec` and names its recognizer kinds. Keeps
//! the files and docs/extending.md §2 in step with the schema in `spec.rs`.

use std::path::Path;

use planrun_parse::spec::FormatSpec;

#[test]
fn every_format_file_matches_the_schema() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../formats");
    let mut seen = 0;
    for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let path = entry.unwrap_or_else(|e| panic!("{e}")).path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
        let spec: FormatSpec =
            toml::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(
            path.file_stem().and_then(|s| s.to_str()),
            Some(spec.id.as_str()),
            "the file name is the format id"
        );
        for table in &spec.recognizers {
            assert!(
                table.get("kind").and_then(|k| k.as_str()).is_some(),
                "{}: a [[recognizer]] without `kind`",
                path.display()
            );
        }
        seen += 1;
    }
    assert!(seen >= 1, "no format files in {}", dir.display());
}
