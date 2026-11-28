use super::*;
use crate::source;

fn spec(id: &str, detect: &str) -> String {
    format!(
        r#"
id = "{id}"
name = "test"
[detect]
patterns = [{{ regex = '{detect}', weight = 60 }}]
[outline]
task = {{ level = 3, regex = '^(?P<id>T(?P<ord>\d+)): (?P<title>.+)$' }}
"#
    )
}

fn doc(text: &str) -> Document {
    let source = source::load("t.md", text.as_bytes()).unwrap_or_else(|e| panic!("{e:?}"));
    Document::build(source).unwrap_or_else(|e| panic!("{e:?}"))
}

fn registry(specs: &[(&str, &str)]) -> Registry {
    let mut registry = Registry::new();
    for (id, detect) in specs {
        registry
            .add_spec(id, &spec(id, detect))
            .unwrap_or_else(|e| panic!("{e}"));
    }
    registry
}

fn selected(registry: &Registry, text: &str, forced: Option<&str>) -> Result<String, &'static str> {
    registry
        .select(&doc(text), forced)
        .map(|f| f.id().0.to_string())
        .map_err(|d| d.code)
}

#[test]
fn the_highest_score_wins_and_ties_are_refused() {
    let r = registry(&[("a", "^### T"), ("b", "^Plan B$")]);
    assert_eq!(selected(&r, "### T1: x\n", None), Ok("a".to_owned()));
    assert_eq!(selected(&r, "Notes.\n", None), Err("no-format"));
    assert_eq!(
        selected(&r, "### T1: x\n\nPlan B\n", None),
        Err("ambiguous-format")
    );
    assert_eq!(selected(&r, "Notes.\n", Some("b")), Ok("b".to_owned()));
    assert_eq!(selected(&r, "Notes.\n", Some("c")), Err("unknown-format"));
}

#[test]
fn detection_skips_fenced_blocks() {
    let r = registry(&[("a", "^### T")]);
    assert_eq!(
        selected(&r, "```markdown\n### T1: x\n```\n", None),
        Err("no-format")
    );
}

#[test]
fn a_spec_is_validated_when_it_loads() {
    let mut r = Registry::new();
    let refused = |r: &mut Registry, origin: &str, text: &str| r.add_spec(origin, text).err();
    assert!(matches!(
        refused(&mut r, "x", &spec("y", "a")),
        Some(RegistryError::InvalidSpec { .. })
    ));
    assert!(matches!(
        refused(&mut r, "x", &spec("x", "(")),
        Some(RegistryError::InvalidSpec { .. })
    ));
    let no_ord = spec("x", "a").replace("(?P<ord>", "(");
    let message = refused(&mut r, "x", &no_ord).map(|e| e.to_string());
    assert_eq!(
        message.as_deref(),
        Some("format `x`: outline.task: the regex has no `ord` capture")
    );
    let unknown_kind = format!("{}\n[[recognizer]]\nkind = \"nope\"\n", spec("x", "a"));
    assert_eq!(
        refused(&mut r, "x", &unknown_kind),
        Some(RegistryError::UnknownKind {
            format: "x".to_owned(),
            kind: "nope".to_owned()
        })
    );
    let unknown_key = format!("{}\nextra = 1\n", spec("x", "a"));
    assert!(refused(&mut r, "x", &unknown_key).is_some());
    assert_eq!(refused(&mut r, "x", &spec("x", "a")), None);
    assert_eq!(
        refused(&mut r, "x", &spec("x", "a")),
        Some(RegistryError::DuplicateFormat("x".to_owned()))
    );
}

#[test]
fn a_later_layer_replaces_a_format_whole_and_says_what_it_replaced() {
    let user = FormatSource::User(PathBuf::from("/u/formats/assay.toml"));
    let repo = FormatSource::Repository(PathBuf::from("/r/.planrun/formats/assay.toml"));
    let extra = FormatSource::Repository(PathBuf::from("/r/.planrun/formats/mine.toml"));
    let (by_user, by_repo, mine) = (
        spec("assay", "^User$"),
        spec("assay", "^Repo$"),
        spec("mine", "^Mine$"),
    );
    let r = Registry::with_layers([
        ("assay", by_user.as_str(), &user),
        ("assay", by_repo.as_str(), &repo),
        ("mine", mine.as_str(), &extra),
    ])
    .unwrap_or_else(|e| panic!("{e}"));
    let builtins = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(r.formats().len(), builtins.formats().len() + 1);
    assert_eq!(selected(&r, "Repo\n", None), Ok("assay".to_owned()));
    assert_eq!(selected(&r, "User\n", None), Err("no-format"));
    assert_eq!(selected(&r, "### P1-T1: x\n", None), Err("no-format"));
    assert_eq!(
        r.origin("assay"),
        Some(&Origin {
            source: repo.clone(),
            overrides: Some(user.clone()),
        })
    );
    assert_eq!(r.origin("mine").map(|o| &o.overrides), Some(&None));
    assert_eq!(
        r.origin("superpowers").map(|o| &o.source),
        Some(&FormatSource::BuiltIn)
    );
}

#[test]
fn an_origin_describes_its_file_and_what_it_overrides() {
    let root = Path::new("/r");
    let repo = FormatSource::Repository(PathBuf::from("/r/.planrun/formats/assay.toml"));
    let user = FormatSource::User(PathBuf::from("/u/formats/assay.toml"));
    let origin = |source: &FormatSource, overrides: Option<&FormatSource>| Origin {
        source: source.clone(),
        overrides: overrides.cloned(),
    };
    let builtin = FormatSource::BuiltIn;
    assert_eq!(
        origin(&builtin, None).describe("assay", Some(root)),
        "assay"
    );
    assert_eq!(
        origin(&repo, Some(&builtin)).describe("assay", Some(root)),
        "assay (.planrun/formats/assay.toml, overrides built-in)"
    );
    assert_eq!(
        origin(&repo, Some(&user)).describe("assay", Some(root)),
        "assay (.planrun/formats/assay.toml, overrides /u/formats/assay.toml)"
    );
    assert_eq!(
        origin(&user, None).describe("mine", None),
        "mine (/u/formats/assay.toml)"
    );
}

#[test]
fn a_refused_layer_names_its_file() {
    let source = FormatSource::Repository(PathBuf::from("/r/.planrun/formats/x.toml"));
    let text = spec("y", "a");
    let message = Registry::with_layers([("x", text.as_str(), &source)])
        .err()
        .map(|e| e.to_string());
    assert_eq!(
        message.as_deref(),
        Some("/r/.planrun/formats/x.toml: format `x`: the id `y` is not the file name `x`")
    );
}
