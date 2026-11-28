# planrun P7 — Discovery

**Goal:** planrun finds what a repository holds without being told. One `Discovery` in planrun-exec reads the user's and the repository's format specs (`<user>/formats/*.toml`, `.planrun/formats/*.toml`) and the plan globs of `.planrun/config.toml` (or the defaults `docs/plans/*.md`, `**/superpowers/**/plans/*.md`, `**/plans/*.md`), and lists the plan files they match with one `git ls-files`. The registry layers the discovered formats over the built-ins and says where each came from (D95); plans can be named on the command line (`planrun run p7`, D97); `planrun list` lists them; `planrun tui` without a directory shows them. Design: `docs/superpowers/specs/2026-11-28-discovery-design.md`.

- **P7-T1** (planrun-model): `PathPattern::discovery_glob`, globs with `**`.
- **P7-T2** (planrun-model, planrun-exec, docs): `discover`: the root, the format directories, `.planrun/config.toml`, the user directory; `FormatSource`; D96.
- **P7-T3** (planrun-parse, planrun-cli, planrun-tui, docs): `Registry::with_layers` and `Origin`; the CLI and the TUI load the discovered formats; `check` and `show` name a format's source; D95.
- **P7-T4** (planrun-exec, planrun-cli, docs): `Discovery::plans` and `resolve`; plan names on `check`, `show`, `run`, `resume`, `status`; D97.
- **P7-T5** (planrun-cli): `planrun list`.
- **P7-T6** (planrun-tui, planrun-cli, docs): the Plans view lists the discovered plans and the plans only a journal names.
- **P7-T7** (docs): architecture, extending, tui-design, the READMEs, CLAUDE.md, the roadmap.

**Base:** `a78d708` (`P7 spec: discovery of formats and plan files`), on top of `e3ed302` (`Bump workspace version to 0.2.0`); there fmt, clippy (with and without `e2e`), `cargo test` with `PLANRUN_REQUIRE_SHELLS=1` (35 `test result: ok` lines, 323 tests) and the `e2e` test pass. After P7-T7: 35 lines, 346 tests. Execute from the commit of this plan. Gate per task: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings`, `cargo test`, `cargo test -p planrun-cli --features e2e --test run`, with `PLANRUN_REQUIRE_SHELLS=1` (D82).

**Shell and environment:** PowerShell 7 on Windows; Rust 1.95.0 (stable); git on `PATH`; pwsh 7 and Git for Windows (D63). No new crate: planrun-exec takes `toml` from the workspace (P7-T2, D96 amends D9), so `Cargo.lock` gains one line and nothing is fetched.

**Before P7-T1:** this plan committed, and a clean `git status --porcelain`. To run the plan with planrun itself, run a copy outside `target/`, since every gate builds `target/debug/planrun.exe`: `cargo build --release`, `Copy-Item target/release/planrun.exe $env:TEMP\planrun-p7.exe`, then `& $env:TEMP\planrun-p7.exe run docs/plans/2026-11-28-p7-discovery.md --auto read,build,modify`. Every commit asks to confirm its prose expectation: **answer `y`, not `q`** (the P5→P6 handover's item 10). The real plans stay in `tests/corpus/local/` (D16); this plan quotes none of them, and its tests use synthetic plans and specs only.

**How this plan was verified:** every task was built in a scratch clone of `a78d708` outside this repository (`core.autocrlf=false`), one commit per task. At each commit the gate passes, and so do the task's targeted commands at that commit; the summary lines below come from those runs. `cargo clippy --all-targets --target x86_64-unknown-linux-gnu -- -D warnings` passed at P7-T1, P7-T6 and P7-T7 (not at P7-T2 to P7-T5; P7-T7's tree is the final one, and nothing in it is compiled only on Linux except the symlink test of P7-T4). P7-T3's commit first held `registry.rs` with CRLF line endings; it was made again with LF and the later commits rebased onto it (the trees differ only in those line endings), and the gate was run again on the final tree. The plan was generated from that history (full contents from `git show`, exact diffs from `git diff`, blob ids from `git rev-parse`), `planrun check` passes on it, and `planrun run --dry-run` on a fresh clone of `a78d708` reports every diff's result blob as the verified one. It was **not** replayed by planrun, unlike P6a and P6c.

## Stated decisions made while building

Where the build departs from the spec, or settles what the spec left open:

1. **`FormatSource` is in planrun-model**, not planrun-parse as the spec says: the registry's `Origin` (planrun-parse) and discovery's `FoundSpec` (planrun-exec) both carry it, and planrun-exec cannot see planrun-parse (D2). D95 says so.
2. **`toml` in planrun-exec** (the spec says no D9 line): `.planrun/config.toml` is read where discovery lives. It is a crate the workspace has; D96 amends D9.
3. **`Registry::with_layers`** takes the built-ins plus `(id, text, source)` triples in order, so the CLI and the TUI build their registry in one expression; the spec's `add_spec_layer` is `add_spec_from`. `add_spec` and `add_format` keep their duplicate error for built-in and hand-written formats. A refused discovered spec is `RegistryError::InFile`, which names the file.
4. **`plans` and `resolve` take git's environment**, not a `Git`: the repository is always discovery's root, so the two cannot disagree. `resolve` returns `Resolved` (`path`, `shown`, `by_name`).
5. **An existing file is a path** too (the spec: a separator or `.md`), so `planrun check NOTES` on a file without an extension still reads that file.
6. **`PLANRUN_CONFIG_DIR`** overrides the user directory. The binary's tests (`cli.rs`) point it at a directory that does not exist, so a user's formats cannot change their output; the `e2e` runs (`run.rs`) do not set it.
7. **The TUI lists the discovered plans again at every two-second check** with one `git ls-files` (the spec: on `c`, and when `.git/index` changes). Correctness and simplicity before speed (D1): a new untracked plan appears without `c`, and nothing watches directories. D97's TUI bullet records it.
8. **Plans only a journal names** come from the Journals job, which in discovered mode now also runs at the start. Their detail says "no glob matches it; its journal names it". The merge is a pure function (`plans::journalled`) with a unit test; no test drives it through a real journal.
9. **Where a format came from shows in `check`, `show` and `list`**, not in the TUI's inspect view (the spec mentioned it there). A later polish item.
10. **`list` reuses planrun-tui's `jobs::run_state`** for the journal's state; planrun-cli already depends on planrun-tui. A file a format detects but that has errors is counted among the plans in its last line, and shows `N errors (planrun check <path>)`.
11. **Discovery errors print as `planrun: error[<code>]: <message>`** and exit with code 2; the codes are the spec's (`format-io` also for a spec that is not UTF-8).
12. **The P6c→P7 handover comes before this plan**, not in it: `docs/handovers/2026-11-28-p6c-to-p7.md` (P6c's "After execution" item 4), with every item P6 deferred so far.

## Tasks

### P7-T1: discovery globs with **

**Goal:** `PathPattern::discovery_glob`: the D41 glob plus `**`, a whole path segment that matches zero or more segments, for the plan globs of P7-T2 (D96). `PathPattern::glob`, which manifests use, keeps refusing `**`, so no plan changes meaning. A table test pins the matches.

**Files**

- Modify `crates/planrun-model/src/pattern.rs`

Over 300 lines, so it comes as an exact diff: `crates/planrun-model/src/pattern.rs`.

**Step 1 — write the files**

File `crates/planrun-model/src/pattern.rs` (exact diff; save the block as `$env:TEMP\p7-t1-crates-planrun-model-src-pattern-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-model/src/pattern.rs b/crates/planrun-model/src/pattern.rs
index 84dc806..c5b9427 100644
--- a/crates/planrun-model/src/pattern.rs
+++ b/crates/planrun-model/src/pattern.rs
@@ -1,6 +1,7 @@
 //! Manifest patterns (decision D41): a glob or a range a plan writes in a Files list instead of
 //! one file. A pattern is never a write target: actions carry a [`RepoPath`], which refuses
 //! wildcards (D42), and a manifest entry is either a file or a pattern ([`ManifestPath`]).
+//! Discovery (D96) uses the same globs with `**` as well ([`PathPattern::discovery_glob`]).
 
 use std::fmt;
 
@@ -38,6 +39,9 @@ pub enum PatternError {
     /// `**`, which would cross directories.
     #[error("`{0}` uses `**`; a pattern matches within one directory level")]
     DoubleStar(String),
+    /// A discovery glob with `**` inside a segment (`a**`, `**b`).
+    #[error("`{0}` uses `**` inside a path segment; `**` must be a whole segment")]
+    DoubleStarSegment(String),
     /// `{` inside `{…}`, or an unbalanced brace.
     #[error("`{0}` has nested or unbalanced braces")]
     Braces(String),
@@ -58,6 +62,19 @@ impl PathPattern {
         if text.contains("**") {
             return Err(PatternError::DoubleStar(text.to_owned()));
         }
+        Self::compile(text)
+    }
+
+    /// A glob for discovering files (D96): [`PathPattern::glob`] plus `**`, a whole path
+    /// segment that matches zero or more segments (`a/**/b` matches `a/b` and `a/x/y/b`).
+    pub fn discovery_glob(text: &str) -> Result<Self, PatternError> {
+        if text.split('/').any(|s| s.contains("**") && s != "**") {
+            return Err(PatternError::DoubleStarSegment(text.to_owned()));
+        }
+        Self::compile(text)
+    }
+
+    fn compile(text: &str) -> Result<Self, PatternError> {
         let alternatives = expand_braces(text)?;
         for alternative in &alternatives {
             check_spelling(alternative)?;
@@ -197,13 +214,17 @@ fn expand_braces(text: &str) -> Result<Vec<String>, PatternError> {
         .collect())
 }
 
-/// `*` and `?` never match `/`.
+/// `*` and `?` never match `/`; `**` (a whole segment, only in discovery globs) matches zero or
+/// more segments.
 fn glob_match(pattern: &str, text: &str) -> bool {
     let (p, t): (Vec<char>, Vec<char>) = (pattern.chars().collect(), text.chars().collect());
     glob_chars(&p, &t)
 }
 
 fn glob_chars(p: &[char], t: &[char]) -> bool {
+    if let ['*', '*', rest @ ..] = p {
+        return double_star(rest, t);
+    }
     let Some((&first, rest)) = p.split_first() else {
         return t.is_empty();
     };
@@ -216,6 +237,18 @@ fn glob_chars(p: &[char], t: &[char]) -> bool {
     }
 }
 
+/// `**` at the end matches the rest; `**/` then `rest` matches `rest` at the start of `t` or
+/// after any `/` in it.
+fn double_star(rest: &[char], t: &[char]) -> bool {
+    let Some(after) = rest.strip_prefix(&['/']) else {
+        return rest.is_empty();
+    };
+    glob_chars(after, t)
+        || t.iter()
+            .enumerate()
+            .any(|(i, &c)| c == '/' && glob_chars(after, &t[i + 1..]))
+}
+
 fn in_range(start: &str, end: &str, name: &str) -> bool {
     if name < start {
         return false;
@@ -281,6 +314,48 @@ mod tests {
         assert!(PathPattern::glob("../*.rs").is_err());
     }
 
+    #[rustfmt::skip]
+    #[test]
+    fn discovery_globs_cross_directories_with_double_stars() {
+        let rows: &[(&str, &str, bool)] = &[
+            ("docs/plans/*.md",              "docs/plans/a.md",                      true),
+            ("docs/plans/*.md",              "docs/plans/old/a.md",                  false),
+            ("**/plans/*.md",                "plans/a.md",                           true),
+            ("**/plans/*.md",                "docs/plans/a.md",                      true),
+            ("**/plans/*.md",                "a/b/c/plans/a.md",                     true),
+            ("**/plans/*.md",                "docs/plans/old/a.md",                  false),
+            ("**/plans/*.md",                "docs/myplans/a.md",                    false),
+            ("**/superpowers/**/plans/*.md", "docs/superpowers/plans/a.md",          true),
+            ("**/superpowers/**/plans/*.md", "x/superpowers/y/z/plans/a.md",         true),
+            ("**/superpowers/**/plans/*.md", "superpowers/plans/a.md",               true),
+            ("**/superpowers/**/plans/*.md", "docs/plans/a.md",                      false),
+            ("work/**",                      "work/a/b.md",                          true),
+            ("work/**",                      "other/a.md",                           false),
+            ("{docs,work}/**/*.md",          "work/x/a.md",                          true),
+        ];
+        for &(pattern, file, expected) in rows {
+            let g = PathPattern::discovery_glob(pattern).unwrap_or_else(|e| panic!("{e}"));
+            assert_eq!(g.matches(&path(file)), expected, "{pattern} on {file}");
+        }
+    }
+
+    #[test]
+    fn a_double_star_is_a_whole_segment_and_only_for_discovery() {
+        assert!(matches!(
+            PathPattern::discovery_glob("a/b**/c.md"),
+            Err(PatternError::DoubleStarSegment(_))
+        ));
+        assert!(matches!(
+            PathPattern::discovery_glob("a/**c.md"),
+            Err(PatternError::DoubleStarSegment(_))
+        ));
+        assert!(PathPattern::discovery_glob("../**/*.md").is_err());
+        assert!(matches!(
+            PathPattern::glob("a/**/b.rs"),
+            Err(PatternError::DoubleStar(_))
+        ));
+    }
+
     #[test]
     fn a_range_bounds_the_file_name() {
         let r = PathPattern::range("s/m/0001", "0007_*.sql").unwrap_or_else(|e| panic!("{e}"));
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t1-crates-planrun-model-src-pattern-rs.patch"
Remove-Item "$env:TEMP\p7-t1-crates-planrun-model-src-pattern-rs.patch"
```

**Step 2 — verify**

Targeted: the model's tests, with the discovery-glob table:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-model
```

Expected (exit code 0; summary lines):

```text
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-model/src/pattern.rs
  $verified = [ordered]@{
      'crates/planrun-model/src/pattern.rs' = 'c5b94274b86bfe9a2d33b958d208740a4db67341'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P7-T1: discovery globs with **" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P7-T2: discovery of format specs and the plan globs (D96)

**Goal:** `planrun_exec::discover`: a `Discovery` opened from the current directory finds the repository root (the nearest `.git`, a directory or a file), the user's and the repository's format specs (`<user>/formats/*.toml`, `.planrun/formats/*.toml`, sorted, a BOM removed) and the plan globs of `.planrun/config.toml`, or the defaults. It starts no process and parses no plan (D2). `FormatSource` joins planrun-model; planrun-exec gains `toml` from the workspace. D96, with amended-by lines in D9 and D41.

**Files**

- Modify `Cargo.lock`
- Modify `crates/planrun-exec/Cargo.toml`
- Create `crates/planrun-exec/src/discover.rs`
- Create `crates/planrun-exec/src/discover_tests.rs`
- Modify `crates/planrun-exec/src/lib.rs`
- Modify `crates/planrun-model/src/ids.rs`
- Modify `crates/planrun-model/src/lib.rs`
- Modify `docs/decisions.md`

Over 300 lines, so they come as exact diffs: `Cargo.lock`, `docs/decisions.md`.

**Step 1 — write the files**

File `Cargo.lock` (exact diff; save the block as `$env:TEMP\p7-t2-cargo-lock.patch`, outside the repository, then apply it):

```diff
diff --git a/Cargo.lock b/Cargo.lock
index 368c752..576399c 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -703,6 +703,7 @@ dependencies = [
  "serde_json",
  "similar 3.2.1",
  "thiserror",
+ "toml",
 ]
 
 [[package]]
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t2-cargo-lock.patch"
Remove-Item "$env:TEMP\p7-t2-cargo-lock.patch"
```

File `crates/planrun-exec/Cargo.toml` (full contents):

```toml
[package]
name = "planrun-exec"
description = "Executes a typed plan against a git repository: workspace, git, shells, expectations, policy, journal, runner"
version.workspace = true
edition.workspace = true
license.workspace = true
publish.workspace = true

# Deliberately no dependency on planrun-parse: execution sees only the IR (D2).
[dependencies]
planrun-model.workspace = true
serde.workspace = true
serde_json.workspace = true
similar.workspace = true
thiserror.workspace = true
toml.workspace = true

[lints]
workspace = true
```

File `crates/planrun-exec/src/discover.rs` (full contents):

```rust
//! Discovery (D96): what the repository of the current directory and the user's config directory
//! hold for planrun. Format specs (`.planrun/formats/*.toml`, `<user>/formats/*.toml`) and the
//! repository's `.planrun/config.toml` are read when a [`Discovery`] opens; nothing here writes,
//! and nothing here parses a plan (D2): the spec texts go to planrun-parse's registry.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use planrun_model::{FormatSource, PathPattern, PatternError};
use serde::Deserialize;

/// The plan globs when `.planrun/config.toml` names none (D96).
pub const DEFAULT_GLOBS: [&str; 3] = [
    "docs/plans/*.md",
    "**/superpowers/**/plans/*.md",
    "**/plans/*.md",
];

/// The repository's config file, under the root.
pub const CONFIG_FILE: &str = ".planrun/config.toml";

/// The repository's format directory, under the root.
pub const FORMATS_DIR: &str = ".planrun/formats";

/// A format spec file found by discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundSpec {
    /// The file stem, which the spec's `id` must equal.
    pub id: String,
    /// The file's text, a byte-order mark removed.
    pub text: String,
    /// Where it was found.
    pub source: FormatSource,
}

/// The plan globs, and where they came from.
#[derive(Debug, Clone)]
pub struct Config {
    /// The globs, in the order written.
    pub globs: Vec<PathPattern>,
    /// The config file that named them; `None` for [`DEFAULT_GLOBS`].
    pub file: Option<PathBuf>,
}

/// What the repository and the user's config directory hold for planrun.
#[derive(Debug, Clone)]
pub struct Discovery {
    root: Option<PathBuf>,
    config: Config,
    formats: Vec<FoundSpec>,
}

/// Why discovery failed. Every variant has a stable [`DiscoveryError::code`]; all of them are
/// usage problems (exit code 2).
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    /// No `.git` above the current directory.
    #[error("{0} is not inside a git repository")]
    NotARepository(PathBuf),
    /// A spec file whose stem is no format id.
    #[error("{0}: a format file is named `<id>.toml`, with letters, digits, `-` and `_`")]
    FormatFileName(PathBuf),
    /// A format directory or spec file that could not be read.
    #[error("{path}: {source}")]
    FormatIo {
        /// The directory or file.
        path: PathBuf,
        /// Why.
        source: std::io::Error,
    },
    /// The config file could not be read.
    #[error("{path}: {source}")]
    ConfigIo {
        /// The file.
        path: PathBuf,
        /// Why.
        source: std::io::Error,
    },
    /// The config file is no valid config.
    #[error("{path}: {message}")]
    ConfigParse {
        /// The file.
        path: PathBuf,
        /// What is wrong.
        message: String,
    },
    /// A glob in the config file is refused.
    #[error("{path}: plans.globs: {source}")]
    ConfigGlob {
        /// The file.
        path: PathBuf,
        /// Why.
        source: PatternError,
    },
    /// `plans.globs` is empty.
    #[error("{0}: plans.globs is empty; leave it out for the defaults")]
    ConfigGlobsEmpty(PathBuf),
}

impl DiscoveryError {
    /// The stable code.
    pub fn code(&self) -> &'static str {
        match self {
            DiscoveryError::NotARepository(_) => "not-a-repository",
            DiscoveryError::FormatFileName(_) => "format-file-name",
            DiscoveryError::FormatIo { .. } => "format-io",
            DiscoveryError::ConfigIo { .. } => "config-io",
            DiscoveryError::ConfigParse { .. } => "config-parse",
            DiscoveryError::ConfigGlob { .. } => "config-glob",
            DiscoveryError::ConfigGlobsEmpty(_) => "config-globs-empty",
        }
    }
}

/// `.planrun/config.toml` (D96).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    plans: Option<PlansSection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlansSection {
    globs: Vec<String>,
}

impl Discovery {
    /// Discovers from `cwd`: the repository root is the nearest directory upwards with a `.git`
    /// (a directory, or a file for a worktree or submodule). `user` is the user's config
    /// directory ([`user_dir`]), or `None` for no user layer. Outside a repository there is no
    /// repository layer and no config file; that is not an error.
    pub fn open(cwd: &Path, user: Option<&Path>) -> Result<Self, DiscoveryError> {
        let root = find_root(cwd);
        let mut formats = Vec::new();
        if let Some(user) = user {
            let dir = user.join("formats");
            formats.extend(read_specs(&dir, FormatSource::User)?);
        }
        if let Some(root) = &root {
            let dir = root.join(FORMATS_DIR);
            formats.extend(read_specs(&dir, FormatSource::Repository)?);
        }
        let config = match &root {
            Some(root) => read_config(&root.join(CONFIG_FILE))?,
            None => default_config(),
        };
        Ok(Self {
            root,
            config,
            formats,
        })
    }

    /// The repository root, when there is one.
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    /// The plan globs.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// The format specs found: the user layer, then the repository layer, each sorted by file
    /// name.
    pub fn formats(&self) -> &[FoundSpec] {
        &self.formats
    }
}

/// The user's config directory (D96): `PLANRUN_CONFIG_DIR` when set; else `%APPDATA%\planrun` on
/// Windows, and `$XDG_CONFIG_HOME/planrun` or `$HOME/.config/planrun` elsewhere.
pub fn user_dir() -> Option<PathBuf> {
    user_dir_from(|name| std::env::var_os(name), cfg!(windows))
}

/// [`user_dir`] over the variables `var` gives.
pub fn user_dir_from(var: impl Fn(&str) -> Option<OsString>, windows: bool) -> Option<PathBuf> {
    let set = |name: &str| var(name).filter(|v| !v.is_empty()).map(PathBuf::from);
    if let Some(dir) = set("PLANRUN_CONFIG_DIR") {
        return Some(dir);
    }
    if windows {
        return set("APPDATA").map(|d| d.join("planrun"));
    }
    if let Some(dir) = set("XDG_CONFIG_HOME") {
        return Some(dir.join("planrun"));
    }
    set("HOME").map(|d| d.join(".config").join("planrun"))
}

/// The nearest ancestor of `cwd` (itself included) that holds a `.git`.
fn find_root(cwd: &Path) -> Option<PathBuf> {
    cwd.ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
}

/// Every `*.toml` file of `dir`, sorted by name; a missing directory has none.
fn read_specs(
    dir: &Path,
    source: impl Fn(PathBuf) -> FormatSource,
) -> Result<Vec<FoundSpec>, DiscoveryError> {
    let io = |path: &Path, e| DiscoveryError::FormatIo {
        path: path.to_path_buf(),
        source: e,
    };
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(io(dir, e)),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| io(dir, e))?.path();
        if path.is_file() && path.extension().is_some_and(|x| x == "toml") {
            paths.push(path);
        }
    }
    paths.sort();
    let mut specs = Vec::new();
    for path in paths {
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| is_id(s))
            .ok_or_else(|| DiscoveryError::FormatFileName(path.clone()))?
            .to_owned();
        let text = read_text(&path).map_err(|e| io(&path, e))?;
        specs.push(FoundSpec {
            id,
            text,
            source: source(path),
        });
    }
    Ok(specs)
}

/// A format id: ASCII letters, digits, `-` and `_`, at least one.
fn is_id(stem: &str) -> bool {
    !stem.is_empty()
        && stem
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// A UTF-8 file without its byte-order mark.
fn read_text(path: &Path) -> Result<String, std::io::Error> {
    let bytes = std::fs::read(path)?;
    let text = String::from_utf8(bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(match text.strip_prefix('\u{feff}') {
        Some(rest) => rest.to_owned(),
        None => text,
    })
}

fn default_config() -> Config {
    let globs = DEFAULT_GLOBS
        .iter()
        .filter_map(|g| PathPattern::discovery_glob(g).ok())
        .collect();
    Config { globs, file: None }
}

/// The config file, or the defaults when there is none.
fn read_config(path: &Path) -> Result<Config, DiscoveryError> {
    let text = match read_text(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(default_config()),
        Err(e) => {
            return Err(DiscoveryError::ConfigIo {
                path: path.to_path_buf(),
                source: e,
            });
        }
    };
    let file: ConfigFile = toml::from_str(&text).map_err(|e| DiscoveryError::ConfigParse {
        path: path.to_path_buf(),
        message: e.message().to_owned(),
    })?;
    let Some(plans) = file.plans else {
        return Ok(default_config());
    };
    if plans.globs.is_empty() {
        return Err(DiscoveryError::ConfigGlobsEmpty(path.to_path_buf()));
    }
    let globs = plans
        .globs
        .iter()
        .map(|g| PathPattern::discovery_glob(g))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| DiscoveryError::ConfigGlob {
            path: path.to_path_buf(),
            source: e,
        })?;
    Ok(Config {
        globs,
        file: Some(path.to_path_buf()),
    })
}

#[cfg(test)]
#[path = "discover_tests.rs"]
mod tests;
```

File `crates/planrun-exec/src/discover_tests.rs` (full contents):

```rust
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
```

File `crates/planrun-exec/src/lib.rs` (full contents):

```rust
//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Modules (docs/roadmap.md; P3 built the first eight, P4 the next three):
//!
//! - [`workspace`]: the repository root, [`RepoPath`](planrun_model::RepoPath) resolution with
//!   the link and case check, and the one writer (temp file, second check, rename; D52).
//! - [`eol`]: the line endings of written files (D10, D54).
//! - [`git`]: a thin wrapper over the `git` binary: `apply` with the patch on stdin,
//!   `hash-object`, `status --porcelain` without optional locks, `rev-parse`, `cat-file`,
//!   `check-attr`, `update-index`, `ls-files`; [`BlobId`](git::BlobId).
//! - [`tree`]: the virtual tree, a temporary index and object directory (D52).
//! - [`handler`]: one [`ActionHandler`] per action kind, which simulates it on a virtual tree;
//!   the [`HandlerSet`] is the execution-side plugin point.
//! - [`transaction`]: a step's file actions as prepare, then commit, with rollback (D53); the
//!   disk is checked again between the two (D78).
//! - [`done`]: a journaled step's state on the disk, for resume (D14, D77).
//! - [`preflight`]: task preflight and the dry run over one virtual tree (D50, D55).
//! - [`finding`]: what preflight, a simulation and the dry run report.
//! - [`shell`]: persistent pwsh and bash sessions behind [`ShellSession`] (D60–D65).
//! - [`expect`]: a run against its expectation: exit codes, summary lines with `…`, what asks
//!   for confirmation (D11, D66).
//! - [`policy`]: command classes and the [`CommandClassifier`]: a built-in allowlist over a
//!   splitter per shell (D13, D68).
//! - [`journal`]: one JSONL file per plan under `<git-dir>/planrun/`, the write-ahead recorder
//!   (D18, D76).
//! - [`lock`]: one planrun per repository (D77).
//! - [`runner`]: the state machine Task → Step → Action over the journal and the sessions,
//!   talking to a [`Ui`] (D73–D81).
//! - [`ui`]: the [`Ui`] trait and its questions (D15, D74).
//! - [`discover`]: what the repository and the user's config directory hold for planrun: format
//!   specs, the plan globs (P7, D96).

pub mod discover;
pub mod done;
pub mod eol;
pub mod expect;
pub mod finding;
pub mod git;
pub mod handler;
pub mod journal;
pub mod lock;
pub mod policy;
pub mod preflight;
pub mod runner;
pub mod shell;
pub mod transaction;
pub mod tree;
pub mod ui;
pub mod workspace;

pub use handler::{ActionHandler, HandlerSet};
pub use policy::{CommandClassifier, PolicyClass};
pub use shell::{ScriptRun, ShellSession};
pub use ui::Ui;

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;
#[cfg(test)]
#[path = "../tests/support/ir.rs"]
mod ir;

/// The state of an action's effects on disk, for resuming (decision D14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoneState {
    /// Every effect is present (postimages match).
    Done,
    /// No effect is present (preimages match).
    NotDone,
    /// Some are, some are not, or neither matches: the user decides.
    Unknown,
}
```

File `crates/planrun-model/src/ids.rs` (full contents):

```rust
//! Identifiers: tasks, formats, recognizers, git blobs.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A task id as written (`P1-T3`, `P3c-1-T1`, `Task 4`), with the ordinal the format extracted
/// from it. Ordinals must strictly increase through a plan (check rule `task-order`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId {
    /// The id as written in the heading.
    pub raw: Box<str>,
    /// The number the format's task pattern captured as `ord`.
    pub ordinal: u32,
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

/// The id of a plan format (`assay`, `superpowers`, …): the `id` of its spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FormatId(pub Box<str>);

/// Where a format's spec came from (D95): built into planrun, the user's config directory, or
/// the repository's `.planrun/formats/`; the two files are paths as discovery found them.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FormatSource {
    /// `formats/*.toml`, embedded.
    BuiltIn,
    /// `<user config directory>/formats/<id>.toml`.
    User(PathBuf),
    /// `<repo>/.planrun/formats/<id>.toml`.
    Repository(PathBuf),
}

/// The id of the recognizer instance that produced an action (`file-block#full-contents`), for
/// `planrun show` and the coverage report.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecognizerId(pub Box<str>);

/// A git blob id as abbreviated in a diff's `index` line: 4 to 40 hex digits. Serialized as
/// its hex string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AbbrevBlobId {
    nibbles: [u8; 40],
    len: u8,
}

/// Why a blob id did not parse.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("`{0}` is not a git blob id (4 to 40 hex digits)")]
pub struct BlobIdError(pub String);

impl AbbrevBlobId {
    /// Parses `hex` (case-insensitive).
    pub fn parse(hex: &str) -> Result<Self, BlobIdError> {
        if !(4..=40).contains(&hex.len()) {
            return Err(BlobIdError(hex.to_owned()));
        }
        let mut nibbles = [0u8; 40];
        for (slot, c) in nibbles.iter_mut().zip(hex.chars()) {
            *slot = c
                .to_digit(16)
                .and_then(|d| u8::try_from(d).ok())
                .ok_or_else(|| BlobIdError(hex.to_owned()))?;
        }
        Ok(Self {
            nibbles,
            len: u8::try_from(hex.len()).unwrap_or(40),
        })
    }

    /// The all-zero id git writes for a file that does not exist on that side of a diff.
    pub fn is_absent(&self) -> bool {
        self.nibbles[..usize::from(self.len)]
            .iter()
            .all(|&n| n == 0)
    }

    /// True when the full 20-byte `id` starts with this abbreviation.
    pub fn matches(&self, id: &[u8; 20]) -> bool {
        (0..usize::from(self.len)).all(|i| {
            let byte = id[i / 2];
            let nibble = if i % 2 == 0 { byte >> 4 } else { byte & 0x0f };
            nibble == self.nibbles[i]
        })
    }
}

impl TryFrom<String> for AbbrevBlobId {
    type Error = BlobIdError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<AbbrevBlobId> for String {
    fn from(value: AbbrevBlobId) -> Self {
        value.to_string()
    }
}

impl fmt::Display for AbbrevBlobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &n in &self.nibbles[..usize::from(self.len)] {
            write!(f, "{n:x}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_abbreviation_matches_its_full_id() {
        let id = AbbrevBlobId::parse("97bd92a").unwrap_or_else(|e| panic!("{e}"));
        let mut full = [0u8; 20];
        full[..4].copy_from_slice(&[0x97, 0xbd, 0x92, 0xa0]);
        assert!(id.matches(&full));
        full[3] = 0xb0;
        assert!(!id.matches(&full));
        assert_eq!(id.to_string(), "97bd92a");
    }

    #[test]
    fn zeros_mean_absent() {
        let id = AbbrevBlobId::parse("0000000").unwrap_or_else(|e| panic!("{e}"));
        assert!(id.is_absent());
        assert!(AbbrevBlobId::parse("xyz1").is_err());
    }
}
```

File `crates/planrun-model/src/lib.rs` (full contents):

```rust
//! The typed plan IR (docs/architecture.md §3).
//!
//! Parsing produces a [`Plan`]; execution consumes it. Neither side depends on the other: this
//! crate is the contract between them. Text is never copied out of the plan: every piece of text
//! is a [`TextRef`] into the normalized plan source held by [`PlanSource`].
//!
//! The action set ([`ActionKind`]) is closed on purpose (decision D4): plan formats vary,
//! actions don't. A new format maps its own wording onto these actions.

#![forbid(unsafe_code)]

mod action;
mod expect;
mod ids;
mod path;
mod pattern;
mod plan;
mod source;
mod span;

pub use action::{Action, ActionKind, ManualReason, Script, ScriptIntent, Shell, WriteMode};
pub use expect::{ExitExpect, Expectation};
pub use ids::{AbbrevBlobId, BlobIdError, FormatId, FormatSource, RecognizerId, TaskId};
pub use path::{PathOrigin, RepoPath, RepoPathError};
pub use pattern::{ManifestPath, PathPattern, PatternError};
pub use plan::{ChangeKind, LineRange, Manifest, ManifestEntry, Plan, PlanMeta, Step, Task};
pub use source::PlanSource;
pub use span::{LineCol, Span, TextRef};
```

File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p7-t2-docs-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index 7d433e7..5502cb2 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -59,7 +59,7 @@ Each dependency is justified here; add none without a line.
 |---|---|
 | `pulldown-cmark` | CommonMark with byte offsets; correct nested-fence handling |
 | `regex` | spec patterns; linear time, no backtracking blow-ups |
-| `serde`, `toml` | format specs |
+| `serde`, `toml` | format specs; `.planrun/config.toml` (planrun-exec, D96) |
 | `serde_json` | the journal (JSONL); the release description `planrun update` reads (D83) |
 | `thiserror` | error types |
 | `sha1_smol` | git blob ids in-process: the `blob-chain` check from a created file's contents (planrun-parse only; planrun-exec asks `git hash-object`, D56) |
@@ -80,7 +80,7 @@ Rejected: `rusqlite` (D18), `tracing` for now (the journal is the record),
 
 Amended by D56 (no `sha1_smol` in planrun-exec), D64 (the hand-declared process-tree calls), D73
 (`ctrlc` rejected), D74 (`dialoguer` dropped), D83 (`serde_json` in planrun-cli; the update
-crates rejected) and D92 (the TUI's crates, in planrun-tui only).
+crates rejected), D92 (the TUI's crates, in planrun-tui only) and D96 (`toml` in planrun-exec).
 
 ## D10 Line endings of written files
 
@@ -351,6 +351,8 @@ entry whose target is a `PathPattern`, not a `RepoPath`: `ManifestEntry.path` is
 Handover P2a→P2b open item 1 proposed a `PathOrigin` for patterns; a target type of its own
 was chosen instead, so "no write targets a wildcard" is a property of the types.
 
+Amended by D96 (discovery globs, which allow `**`; manifest globs still refuse it).
+
 ## D42 RepoPath refuses wildcards
 
 Amends D7: a `RepoPath` also refuses `*` and `?`. Both are invalid in Windows file names
@@ -1437,3 +1439,29 @@ Amends D89.
   other thread (the runner's, a background job's) it leaves the terminal alone and prints
   nothing: the thread catches it, and its message comes back to the app as that work's failure
   (a run that ended with an error, a job in its failed state).
+
+## D96 Discovery: format specs and plan files
+
+Amends D41 and D9. Decided in the P7 brainstorm (docs/superpowers/specs/2026-11-28-discovery-design.md).
+
+- **One `Discovery` value in planrun-exec** (`discover`): it reads the user's and the
+  repository's format specs and the repository's `.planrun/config.toml` when it opens, starts no
+  process, and returns spec texts, never a `Format` (D2). Callers hand the texts to the registry.
+- **The root** is the nearest directory upwards from the current directory with a `.git` (a
+  directory, or a file for a worktree or submodule), as for `run` and the TUI (D88, D94).
+  Outside a repository there is no repository layer and no config file; that is not an error.
+- **Format directories:** `<root>/.planrun/formats/` and `<user>/formats/`, where `<user>` is
+  `PLANRUN_CONFIG_DIR` when set, else `%APPDATA%\planrun` on Windows and
+  `$XDG_CONFIG_HOME/planrun` or `$HOME/.config/planrun` elsewhere; no `dirs` crate. A missing
+  directory is empty. Only `*.toml` files are read, sorted by name; a stem that is not letters,
+  digits, `-` and `_` is `format-file-name`, a file that is not UTF-8 is `format-io`. A spec that
+  cannot be used stops the command (exit code 2) and names its file: never skipped, also not a
+  user-level one that every repository sees (D3).
+- **`.planrun/config.toml`** v1 has one key, `[plans] globs`, which replaces the default globs
+  `docs/plans/*.md`, `**/superpowers/**/plans/*.md` and `**/plans/*.md`. Unknown keys are
+  `config-parse`, a refused glob `config-glob`, `globs = []` `config-globs-empty`. A default
+  format for `--format` was left out: detection picks it, and `--format` covers the rest.
+- **Discovery globs** are D41 globs plus `**`, a whole path segment that matches zero or more
+  segments (`PathPattern::discovery_glob`). `**` inside a segment is refused. Manifest globs keep
+  refusing `**`, so no plan changes meaning.
+- **`toml` in planrun-exec**, for the config file: a crate the workspace has already (D9).
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t2-docs-decisions-md.patch"
Remove-Item "$env:TEMP\p7-t2-docs-decisions-md.patch"
```

**Step 2 — verify**

Targeted: the discover module's tests:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-exec --lib discover
```

Expected (exit code 0; summary lines):

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- Cargo.lock crates/planrun-exec/Cargo.toml crates/planrun-exec/src/discover.rs crates/planrun-exec/src/discover_tests.rs crates/planrun-exec/src/lib.rs crates/planrun-model/src/ids.rs crates/planrun-model/src/lib.rs docs/decisions.md
  $verified = [ordered]@{
      'Cargo.lock' = '576399c38c7821fb4eaa9f4586bda05bff3f0336'
      'crates/planrun-exec/Cargo.toml' = '978402fd3c7300c4035972dd72477294cf14ff52'
      'crates/planrun-exec/src/discover.rs' = '95df0725f4a459f20a85bd86a324aa6a0ce7d64a'
      'crates/planrun-exec/src/discover_tests.rs' = 'acd67d843ae3376c5ad504bf8dd0ec859784f99b'
      'crates/planrun-exec/src/lib.rs' = '97a5c1b141679ae05dec4ef62b8c82e94dda47ee'
      'crates/planrun-model/src/ids.rs' = '1cf4be78fed9025fd7e01ebaf19c9198f0574308'
      'crates/planrun-model/src/lib.rs' = 'e279272dad0432435701489871edfb33b17d73e6'
      'docs/decisions.md' = '5502cb295b4a4b71f96974b64c55df6774cbce60'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P7-T2: discovery of format specs and the plan globs (D96)" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P7-T3: format layers in the registry, the CLI and the TUI (D95)

**Goal:** The registry layers discovered specs over the built-ins (`Registry::with_layers`): a spec replaces the format with its id whole, and its `Origin` says where it came from and what it replaced. `check` and `show` print it (`assay (.planrun/formats/assay.toml, overrides built-in)`); a built-in format prints as before. The CLI and the TUI build their registry from discovery, and a spec or config that cannot be used stops them with exit code 2. D95.

**Files**

- Modify `crates/planrun-cli/src/main.rs`
- Modify `crates/planrun-cli/tests/cli.rs`
- Modify `crates/planrun-exec/src/discover.rs`
- Modify `crates/planrun-parse/src/lib.rs`
- Modify `crates/planrun-parse/src/registry.rs`
- Modify `crates/planrun-parse/src/registry_tests.rs`
- Modify `crates/planrun-tui/src/app_tests.rs`
- Modify `crates/planrun-tui/src/jobs.rs`
- Modify `crates/planrun-tui/src/terminal.rs`
- Modify `crates/planrun-tui/tests/support/mod.rs`
- Modify `docs/decisions.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/src/main.rs`, `crates/planrun-exec/src/discover.rs`, `crates/planrun-parse/src/registry.rs`, `crates/planrun-tui/src/jobs.rs`, `docs/decisions.md`.

**Step 1 — write the files**

File `crates/planrun-cli/src/main.rs` (exact diff; save the block as `$env:TEMP\p7-t3-crates-planrun-cli-src-main-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/main.rs b/crates/planrun-cli/src/main.rs
index 4a73c46..6d0aa7a 100644
--- a/crates/planrun-cli/src/main.rs
+++ b/crates/planrun-cli/src/main.rs
@@ -15,6 +15,7 @@ use std::path::{Path, PathBuf};
 use std::process::ExitCode;
 
 use clap::{Parser, Subcommand};
+use planrun_exec::discover::{self, Discovery};
 use planrun_exec::preflight;
 use planrun_exec::workspace::Workspace;
 use planrun_parse::{Diagnostics, Parsed, Registry, parse_plan, source};
@@ -227,18 +228,62 @@ fn with_plan(
     f: impl FnOnce(&planrun_model::Plan, &mut String) -> u8,
 ) -> u8 {
     match load(path, format, out) {
-        Loaded::Parsed(parsed) => f(&parsed.plan, out),
+        Loaded::Parsed(parsed, _) => f(&parsed.plan, out),
         Loaded::Failed(code) => code,
     }
 }
 
-/// A plan read and parsed, or the exit code when it was not.
+/// A plan read and parsed with its format's description (D95), or the exit code when it was
+/// not.
 enum Loaded {
-    Parsed(Box<Parsed>),
+    Parsed(Box<Parsed>, String),
     Failed(u8),
 }
 
+/// What discovery found from the current directory (D96), and the registry built from it.
+struct Setup {
+    discovery: Discovery,
+    registry: Registry,
+}
+
+/// Discovers from the current directory and layers the formats found over the built-ins; a
+/// problem is a usage problem (exit code 2).
+fn setup(out: &mut String) -> Result<Setup, u8> {
+    let here = match std::env::current_dir() {
+        Ok(dir) => dir,
+        Err(e) => {
+            out.push_str(&format!("planrun: the current directory: {e}\n"));
+            return Err(USAGE);
+        }
+    };
+    let discovery = match Discovery::open(&here, discover::user_dir().as_deref()) {
+        Ok(discovery) => discovery,
+        Err(e) => {
+            out.push_str(&format!("planrun: error[{}]: {e}\n", e.code()));
+            return Err(USAGE);
+        }
+    };
+    let layers = discovery
+        .formats()
+        .iter()
+        .map(|s| (s.id.as_str(), s.text.as_str(), &s.source));
+    match Registry::with_layers(layers) {
+        Ok(registry) => Ok(Setup {
+            discovery,
+            registry,
+        }),
+        Err(e) => {
+            out.push_str(&format!("planrun: the formats: {e}\n"));
+            Err(USAGE)
+        }
+    }
+}
+
 fn load(path: &Path, format: Option<&str>, out: &mut String) -> Loaded {
+    let setup = match setup(out) {
+        Ok(setup) => setup,
+        Err(code) => return Loaded::Failed(code),
+    };
     let name = path.display().to_string();
     let bytes = match std::fs::read(path) {
         Ok(bytes) => bytes,
@@ -247,15 +292,15 @@ fn load(path: &Path, format: Option<&str>, out: &mut String) -> Loaded {
             return Loaded::Failed(USAGE);
         }
     };
-    let registry = match Registry::with_builtins() {
-        Ok(registry) => registry,
-        Err(e) => {
-            out.push_str(&format!("planrun: the built-in formats: {e}\n"));
-            return Loaded::Failed(USAGE);
+    match parse_plan(&setup.registry, &name, &bytes, format) {
+        Ok(parsed) => {
+            let id = parsed.plan.format.0.to_string();
+            let label = setup
+                .registry
+                .origin(&id)
+                .map_or_else(|| id.clone(), |o| o.describe(&id, setup.discovery.root()));
+            Loaded::Parsed(Box::new(parsed), label)
         }
-    };
-    match parse_plan(&registry, &name, &bytes, format) {
-        Ok(parsed) => Loaded::Parsed(Box::new(parsed)),
         Err(diagnostics) => {
             failed(&name, &bytes, &diagnostics, out);
             Loaded::Failed(PLAN_ERRORS)
@@ -275,8 +320,8 @@ fn failed(name: &str, bytes: &[u8], diagnostics: &Diagnostics, out: &mut String)
 }
 
 fn check(path: &Path, format: Option<&str>, out: &mut String) -> u8 {
-    let parsed = match load(path, format, out) {
-        Loaded::Parsed(parsed) => parsed,
+    let (parsed, label) = match load(path, format, out) {
+        Loaded::Parsed(parsed, label) => (parsed, label),
         Loaded::Failed(code) => return code,
     };
     let plan = &parsed.plan;
@@ -284,21 +329,22 @@ fn check(path: &Path, format: Option<&str>, out: &mut String) -> u8 {
     render::diagnostics(&name, plan.source.text(), &parsed.diagnostics.0, out);
     render::coverage(plan, &parsed.coverage, out);
     out.push_str(&format!(
-        "planrun: {name}: ok ({}, {} tasks)\n",
-        plan.format.0,
+        "planrun: {name}: ok ({label}, {} tasks)\n",
         plan.tasks.len()
     ));
     OK
 }
 
 fn show(path: &Path, target: Option<&str>, format: Option<&str>, out: &mut String) -> u8 {
-    let parsed = match load(path, format, out) {
-        Loaded::Parsed(parsed) => parsed,
+    let (parsed, label) = match load(path, format, out) {
+        Loaded::Parsed(parsed, label) => (parsed, label),
         Loaded::Failed(code) => return code,
     };
     let viewer = Viewer::new(&parsed.plan);
     let Some(target) = target else {
-        view::plan_text(&viewer.plan(), out);
+        let mut whole = viewer.plan();
+        whole.format = label;
+        view::plan_text(&whole, out);
         return OK;
     };
     let (task_id, step) = match target.split_once('/') {
@@ -330,7 +376,7 @@ fn show(path: &Path, target: Option<&str>, format: Option<&str>, out: &mut Strin
 /// repository cannot be used.
 fn run_dry(path: &Path, format: Option<&str>, out: &mut String) -> u8 {
     let parsed = match load(path, format, out) {
-        Loaded::Parsed(parsed) => parsed,
+        Loaded::Parsed(parsed, _) => parsed,
         Loaded::Failed(code) => return code,
     };
     let plan = &parsed.plan;
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t3-crates-planrun-cli-src-main-rs.patch"
Remove-Item "$env:TEMP\p7-t3-crates-planrun-cli-src-main-rs.patch"
```

File `crates/planrun-cli/tests/cli.rs` (full contents):

```rust
//! The `planrun` binary: exit codes, `check` output, the rendered message of every invalid
//! plan, and `show` of one step. Runs from the workspace root with relative paths, so the output
//! is the same on every machine; the user's config directory is one that does not exist, so
//! their formats stay out (D96).

#[path = "../../../tests/support/fixture.rs"]
mod fixture;

use std::path::{Path, PathBuf};
use std::process::Command;

use fixture::Fixture;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Runs `planrun args…`; returns the exit code and stdout + stderr.
fn planrun(args: &[&str]) -> (i32, String) {
    planrun_in(&root(), args)
}

/// A user config directory that does not exist.
fn no_user_config() -> PathBuf {
    root().join("target/planrun-tests-no-user-config")
}

/// Runs `planrun args…` in `dir`.
fn planrun_in(dir: &Path, args: &[&str]) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_planrun"))
        .args(args)
        .current_dir(dir)
        .env("PLANRUN_CONFIG_DIR", no_user_config())
        .output()
        .unwrap_or_else(|e| panic!("{e}"));
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), text)
}

const SYNTHETIC: &str = "tests/corpus/synthetic/assay-style.md";

#[test]
fn check_passes_the_synthetic_plan() {
    let (code, out) = planrun(&["check", SYNTHETIC]);
    assert_eq!(code, 0, "{out}");
    insta::assert_snapshot!("check_assay_style", out);
}

#[test]
fn check_renders_every_invalid_plan() {
    let dir = root().join("tests/corpus/invalid");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{e}"))
        .map(|e| {
            e.unwrap_or_else(|e| panic!("{e}"))
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|n| n.ends_with(".md") && n != "README.md")
        .collect();
    names.sort();
    let mut all = String::new();
    for name in &names {
        let path = format!("tests/corpus/invalid/{name}");
        let (code, out) = planrun(&["check", &path]);
        assert_eq!(code, 1, "{path}: {out}");
        all.push_str(&format!("==== {name}\n{out}\n"));
    }
    insta::assert_snapshot!("check_invalid_corpus", all);
}

#[test]
fn show_prints_a_step_in_full() {
    let (code, out) = planrun(&["show", SYNTHETIC, "P1-T2/2"]);
    assert_eq!(code, 0, "{out}");
    insta::assert_snapshot!("show_p1_t2_step_2", out);
}

#[test]
fn usage_problems_exit_with_2() {
    assert_eq!(planrun(&["check"]).0, 2);
    assert_eq!(planrun(&["check", "tests/corpus/none.md"]).0, 2);
    assert_eq!(planrun(&["show", SYNTHETIC, "P9-T9"]).0, 2);
    assert_eq!(planrun(&["show", SYNTHETIC, "P1-T1/7"]).0, 2);
    assert_eq!(planrun(&["check", SYNTHETIC, "--format", "nope"]).0, 1);
}

#[test]
fn tui_refuses_a_stdout_that_is_not_a_terminal() {
    let (code, out) = planrun(&["tui"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("planrun tui needs a terminal"), "{out}");
}

/// A fixture copy with the synthetic assay plan at its root.
fn with_plan() -> Fixture {
    let f = Fixture::greeter(false);
    let plan = std::fs::read(root().join(SYNTHETIC)).unwrap_or_else(|e| panic!("{e}"));
    f.write("assay-style.md", &plan);
    f
}

#[test]
fn a_repository_format_overrides_the_built_in_one_and_check_says_so() {
    let f = with_plan();
    let spec = std::fs::read(root().join("formats/assay.toml")).unwrap_or_else(|e| panic!("{e}"));
    f.write(".planrun/formats/assay.toml", &spec);
    let (code, out) = planrun_in(f.path(), &["check", "assay-style.md"]);
    assert_eq!(code, 0, "{out}");
    let last = out.lines().last().unwrap_or_default();
    assert_eq!(
        last,
        "planrun: assay-style.md: ok (assay (.planrun/formats/assay.toml, overrides built-in), 2 tasks)"
    );
    let (code, out) = planrun_in(f.path(), &["show", "assay-style.md"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("format  assay (.planrun/formats/assay.toml, overrides built-in)\n"),
        "{out}"
    );
}

#[test]
fn a_spec_or_config_that_cannot_be_used_stops_with_2() {
    let f = with_plan();
    f.write(".planrun/formats/broken.toml", b"id = 1\n");
    let (code, out) = planrun_in(f.path(), &["check", "assay-style.md"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("broken.toml: format `broken`:"), "{out}");
    std::fs::remove_file(f.path().join(".planrun/formats/broken.toml"))
        .unwrap_or_else(|e| panic!("{e}"));
    f.write(".planrun/config.toml", b"[plans]\nglobs = []\n");
    let (code, out) = planrun_in(f.path(), &["check", "assay-style.md"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("error[config-globs-empty]"), "{out}");
}
```

File `crates/planrun-exec/src/discover.rs` (exact diff; save the block as `$env:TEMP\p7-t3-crates-planrun-exec-src-discover-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/discover.rs b/crates/planrun-exec/src/discover.rs
index 95df072..9c9b640 100644
--- a/crates/planrun-exec/src/discover.rs
+++ b/crates/planrun-exec/src/discover.rs
@@ -125,6 +125,17 @@ struct PlansSection {
     globs: Vec<String>,
 }
 
+impl Default for Discovery {
+    /// No repository, no user directory: the default globs and no formats.
+    fn default() -> Self {
+        Self {
+            root: None,
+            config: default_config(),
+            formats: Vec::new(),
+        }
+    }
+}
+
 impl Discovery {
     /// Discovers from `cwd`: the repository root is the nearest directory upwards with a `.git`
     /// (a directory, or a file for a worktree or submodule). `user` is the user's config
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t3-crates-planrun-exec-src-discover-rs.patch"
Remove-Item "$env:TEMP\p7-t3-crates-planrun-exec-src-discover-rs.patch"
```

File `crates/planrun-parse/src/lib.rs` (full contents):

```rust
//! Markdown plan → typed [`planrun_model::Plan`] (docs/architecture.md §4–§5).
//!
//! Pipeline: read and normalize ([`source`]) → block stream ([`document`]) → pick a format
//! ([`registry`]) → outline: tasks, manifests, steps ([`format`], [`outline`]) → recognizers turn
//! each step's blocks into actions ([`recognize`], [`recognizer`], the kinds in `kinds`) →
//! static checks ([`check`]).
//!
//! This crate never touches the repository and never runs anything (decision D2). Checks that
//! need the repository (preimage blob ids against the files on disk) live in planrun-exec.
//!
//! Extension points, in the order a new plan style should try them (docs/extending.md):
//! 1. a format spec in TOML that reuses the built-in recognizer kinds ([`spec`]);
//! 2. a new [`recognizer::RecognizerFactory`] for a new idiom;
//! 3. a hand-written [`format::Format`] when the outline is not heading-shaped.

#![forbid(unsafe_code)]

pub mod check;
pub mod diagnostic;
pub mod document;
pub mod format;
mod kinds;
pub mod manifest;
pub mod outline;
pub mod patch;
pub mod recognize;
pub mod recognizer;
pub mod registry;
pub mod source;
pub mod spec;

pub use diagnostic::{Diagnostic, Diagnostics, Severity};
pub use recognize::TaskCoverage;
pub use registry::{Origin, Registry};

use planrun_model::Plan;

use crate::document::Document;
use crate::format::{FormatSettings, Outline};

/// A plan that parsed and checked without errors, with its warnings and notes and its coverage
/// report.
#[derive(Debug, Clone)]
pub struct Parsed {
    /// The plan.
    pub plan: Plan,
    /// Warnings and notes, in source order.
    pub diagnostics: Diagnostics,
    /// Per task, where its blocks went.
    pub coverage: Vec<TaskCoverage>,
}

/// Parses and checks `bytes` as a plan named `name`, with the formats in `registry`.
///
/// `format` forces a format by id; otherwise the one with the highest detection score wins, and
/// a tie or a score below the registry's threshold is an error (never a guess).
///
/// A plan is returned only when there is no error (`Never execute a partially parsed plan`,
/// decision D3); otherwise every diagnostic comes back, in source order. Check rules run only
/// when the outline and recognition had no errors.
pub fn parse_plan(
    registry: &Registry,
    name: &str,
    bytes: &[u8],
    format: Option<&str>,
) -> Result<Parsed, Diagnostics> {
    let source = source::load(name, bytes).map_err(Diagnostics::single)?;
    let doc = Document::build(source)?;
    let format = registry.select(&doc, format).map_err(Diagnostics::single)?;
    let outline = format.outline(&doc)?;
    let (tasks, coverage) = recognize::recognize(format.as_ref(), &doc, &outline)?;
    let mut found = outside_notes(&doc, format.settings(), &outline);
    let plan = Plan {
        source: doc.source,
        format: format.id().clone(),
        meta: outline.meta,
        tasks,
    };
    let mut checked = Vec::new();
    for rule in check::default_rules() {
        rule.check(&plan, &mut checked);
    }
    let mut checked = format.settings().apply_checks(checked);
    check::drop_reported_write_modes(&mut checked);
    found.extend(checked);
    let mut diagnostics = Diagnostics(found);
    diagnostics.sort();
    if diagnostics.has_errors() {
        return Err(diagnostics);
    }
    Ok(Parsed {
        plan,
        diagnostics,
        coverage,
    })
}

/// Manual markers outside the tasks: before task 1 (read them first) and after the tasks (not
/// run).
fn outside_notes(doc: &Document, settings: &FormatSettings, outline: &Outline) -> Vec<Diagnostic> {
    let before = outline.meta.preamble.iter().map(|text| {
        Diagnostic::note(
            "manual-outside-tasks",
            Some(text.span()),
            "read this before task 1: it is an instruction for you",
        )
    });
    let after = outline
        .after_tasks
        .map(|range| outline::marked(doc, settings, range))
        .unwrap_or_default();
    let after = after.into_iter().map(|text| {
        Diagnostic::note(
            "manual-outside-tasks",
            Some(text.span()),
            "after the tasks: planrun does not run this",
        )
    });
    before.chain(after).collect()
}
```

File `crates/planrun-parse/src/registry.rs` (exact diff; save the block as `$env:TEMP\p7-t3-crates-planrun-parse-src-registry-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-parse/src/registry.rs b/crates/planrun-parse/src/registry.rs
index bb5ef88..9016677 100644
--- a/crates/planrun-parse/src/registry.rs
+++ b/crates/planrun-parse/src/registry.rs
@@ -1,13 +1,17 @@
 //! The registry: recognizer factories by kind, and the formats built from specs with them.
 //!
-//! Composition root of parsing. Built-ins register in [`Registry::with_builtins`]; user formats
-//! are added from TOML (`.planrun/formats/*.toml` in the repository, then the user config
-//! directory) with [`Registry::add_spec`]. A spec that names an unknown recognizer kind, has an
-//! invalid regex or duplicates a format id fails here, before any plan is read.
+//! Composition root of parsing. Built-ins register in [`Registry::with_builtins`]; the formats
+//! discovery found (planrun-exec's `discover`, D96) are layered on top with
+//! [`Registry::with_layers`]: a user spec replaces a built-in format with its id, a repository
+//! spec replaces either (D95). A spec that names an unknown recognizer kind or has an invalid
+//! regex fails here, before any plan is read.
 
 use std::collections::HashMap;
+use std::path::{Path, PathBuf};
 use std::sync::Arc;
 
+use planrun_model::FormatSource;
+
 use crate::diagnostic::Diagnostic;
 use crate::document::Document;
 use crate::format::Format;
@@ -19,10 +23,53 @@ use crate::spec::{FormatSpec, SpecFormat};
 pub struct Registry {
     factories: HashMap<&'static str, Arc<dyn RecognizerFactory>>,
     formats: Vec<Arc<dyn Format>>,
+    /// Where each format of `formats` came from, at the same index.
+    origins: Vec<Origin>,
     /// Detection scores below this never select a format.
     pub min_score: u8,
 }
 
+/// Where a registered format came from, and what it replaced (D95).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Origin {
+    /// Its spec's place.
+    pub source: FormatSource,
+    /// The format with its id that it replaced, if any.
+    pub overrides: Option<FormatSource>,
+}
+
+impl Origin {
+    /// `id` when it is built in and replaces nothing; else `id (path, overrides …)`, with paths
+    /// relative to `root` when they are under it.
+    pub fn describe(&self, id: &str, root: Option<&Path>) -> String {
+        let mut parts = Vec::new();
+        if let Some(path) = place(&self.source, root) {
+            parts.push(path);
+        }
+        if let Some(old) = &self.overrides {
+            let what = place(old, root).unwrap_or_else(|| "built-in".to_owned());
+            parts.push(format!("overrides {what}"));
+        }
+        if parts.is_empty() {
+            return id.to_owned();
+        }
+        format!("{id} ({})", parts.join(", "))
+    }
+}
+
+/// A spec file's path for display, or `None` for a built-in format.
+fn place(source: &FormatSource, root: Option<&Path>) -> Option<String> {
+    let path = match source {
+        FormatSource::BuiltIn => return None,
+        FormatSource::User(path) | FormatSource::Repository(path) => path,
+    };
+    let relative = root.and_then(|r| path.strip_prefix(r).ok());
+    Some(match relative {
+        Some(rel) => rel.to_string_lossy().replace('\\', "/"),
+        None => path.display().to_string(),
+    })
+}
+
 /// Why a factory, spec or format was refused.
 #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
 pub enum RegistryError {
@@ -48,6 +95,14 @@ pub enum RegistryError {
         /// What is wrong.
         message: String,
     },
+    /// A discovered spec file was refused.
+    #[error("{path}: {error}")]
+    InFile {
+        /// The spec file.
+        path: PathBuf,
+        /// Why.
+        error: Box<RegistryError>,
+    },
 }
 
 impl Default for Registry {
@@ -62,6 +117,7 @@ impl Registry {
         Self {
             factories: HashMap::new(),
             formats: Vec::new(),
+            origins: Vec::new(),
             min_score: 50,
         }
     }
@@ -86,6 +142,60 @@ impl Registry {
         Ok(registry)
     }
 
+    /// The built-ins with discovered specs layered on top, in order (D95): each replaces the
+    /// format with its id that is already registered, whole. Discovery gives the user layer
+    /// before the repository layer, so the repository wins. A refused spec is an error that
+    /// names its file.
+    pub fn with_layers<'a>(
+        specs: impl IntoIterator<Item = (&'a str, &'a str, &'a FormatSource)>,
+    ) -> Result<Self, RegistryError> {
+        let mut registry = Self::with_builtins()?;
+        for (origin, text, source) in specs {
+            registry
+                .add_spec_from(origin, text, source.clone())
+                .map_err(|e| match source {
+                    FormatSource::BuiltIn => e,
+                    FormatSource::User(path) | FormatSource::Repository(path) => {
+                        RegistryError::InFile {
+                            path: path.clone(),
+                            error: Box::new(e),
+                        }
+                    }
+                })?;
+        }
+        Ok(registry)
+    }
+
+    /// Builds a format from a spec found at `source` and registers it; a registered format with
+    /// its id is replaced in place, and its source becomes the new one's `overrides`.
+    pub fn add_spec_from(
+        &mut self,
+        origin: &str,
+        toml_text: &str,
+        source: FormatSource,
+    ) -> Result<(), RegistryError> {
+        let format: Arc<dyn Format> = Arc::new(self.compile_spec(origin, toml_text)?);
+        let at = self.formats.iter().position(|f| f.id() == format.id());
+        let Some(at) = at else {
+            self.formats.push(format);
+            self.origins.push(Origin {
+                source,
+                overrides: None,
+            });
+            return Ok(());
+        };
+        let old = std::mem::replace(&mut self.origins[at].source, source);
+        self.origins[at].overrides = Some(old);
+        self.formats[at] = format;
+        Ok(())
+    }
+
+    /// Where the format `id` came from.
+    pub fn origin(&self, id: &str) -> Option<&Origin> {
+        let at = self.formats.iter().position(|f| &*f.id().0 == id)?;
+        self.origins.get(at)
+    }
+
     /// Registers a recognizer factory.
     pub fn add_factory(
         &mut self,
@@ -98,18 +208,29 @@ impl Registry {
         Ok(())
     }
 
-    /// Registers a hand-written format.
+    /// Registers a hand-written format, as a built-in one.
     pub fn add_format(&mut self, format: Arc<dyn Format>) -> Result<(), RegistryError> {
         if self.formats.iter().any(|f| f.id() == format.id()) {
             return Err(RegistryError::DuplicateFormat(format.id().0.to_string()));
         }
         self.formats.push(format);
+        self.origins.push(Origin {
+            source: FormatSource::BuiltIn,
+            overrides: None,
+        });
         Ok(())
     }
 
-    /// Builds a format from TOML spec text and registers it. `origin` is the spec's file stem,
-    /// which must equal its `id` (docs/extending.md §2).
+    /// Builds a built-in format from TOML spec text and registers it. `origin` is the spec's
+    /// file stem, which must equal its `id` (docs/extending.md §2); a second format with one id
+    /// is an error.
     pub fn add_spec(&mut self, origin: &str, toml_text: &str) -> Result<(), RegistryError> {
+        let format = self.compile_spec(origin, toml_text)?;
+        self.add_format(Arc::new(format))
+    }
+
+    /// Compiles a spec with the registered recognizer kinds.
+    fn compile_spec(&self, origin: &str, toml_text: &str) -> Result<SpecFormat, RegistryError> {
         let invalid = |message: String| RegistryError::InvalidSpec {
             format: origin.to_owned(),
             message,
@@ -132,9 +253,7 @@ impl Registry {
                 kind: kind.to_owned(),
             });
         }
-        let format =
-            SpecFormat::compile(spec, |kind| self.factories.get(kind).cloned()).map_err(invalid)?;
-        self.add_format(Arc::new(format))
+        SpecFormat::compile(spec, |kind| self.factories.get(kind).cloned()).map_err(invalid)
     }
 
     /// The factory for `kind`.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t3-crates-planrun-parse-src-registry-rs.patch"
Remove-Item "$env:TEMP\p7-t3-crates-planrun-parse-src-registry-rs.patch"
```

File `crates/planrun-parse/src/registry_tests.rs` (full contents):

````rust
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
````

File `crates/planrun-tui/src/app_tests.rs` (full contents):

```rust
//! The app's own tests: what a terminal below the minimum shows.

use std::sync::mpsc::channel;

use planrun_exec::discover::Discovery;
use planrun_exec::ui::{Choice, Question, QuestionKind};
use planrun_parse::{Registry, parse_plan};

use super::*;
use crate::grid::grid;

const PLAN: &str = include_str!("../../../tests/corpus/synthetic/assay-style.md");

fn app() -> App {
    let ctx = Context {
        root: PathBuf::from("."),
        git_dir: PathBuf::from(".git"),
        repo: "greeter".to_owned(),
        target: "greeter".to_owned(),
        plans_dir: PathBuf::from("plans"),
        plans_label: "plans".to_owned(),
        version: "0.1.0".to_owned(),
        env: Vec::new(),
        discovery: Discovery::default(),
    };
    App::new(ctx, Glyphs::unicode(), Arc::new(|| 0), None)
}

#[test]
fn below_the_minimum_an_open_dock_is_drawn_alone_and_answers() {
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let parsed = parse_plan(&registry, "assay-style.md", PLAN.as_bytes(), None)
        .unwrap_or_else(|e| panic!("{e:?}"));
    let mut app = app();
    let mut run = Run::new(
        PathBuf::from("assay-style.md"),
        Arc::new(parsed),
        Assumed::default(),
        false,
        0,
    );
    let (tx, rx) = channel();
    run.ask(
        Question {
            kind: QuestionKind::Manual,
            at: None,
            text: vec!["check it by hand".to_owned()],
            choices: vec![Choice::Continue, Choice::Quit],
            typed: None,
        },
        tx,
    );
    app.run = Some(run);
    app.view = View::Plan;
    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
    app.draw(&mut buf);
    let text = grid(&buf);
    assert!(text.contains("MANUAL"), "{text}");
    assert!(!text.contains("terminal too small"), "{text}");
    app.key(Key::Enter);
    assert_eq!(
        rx.recv().ok(),
        Some(planrun_exec::ui::Reply::Choice(Choice::Continue))
    );
    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
    app.draw(&mut buf);
    assert!(grid(&buf).contains("terminal too small"));
}
```

File `crates/planrun-tui/src/jobs.rs` (exact diff; save the block as `$env:TEMP\p7-t3-crates-planrun-tui-src-jobs-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-tui/src/jobs.rs b/crates/planrun-tui/src/jobs.rs
index 2b18dd8..a92f41c 100644
--- a/crates/planrun-tui/src/jobs.rs
+++ b/crates/planrun-tui/src/jobs.rs
@@ -8,6 +8,7 @@ use std::path::{Path, PathBuf};
 use std::sync::Arc;
 use std::time::UNIX_EPOCH;
 
+use planrun_exec::discover::Discovery;
 use planrun_exec::journal::{self, End, Line, Record};
 use planrun_exec::lock;
 use planrun_exec::preflight::{self, DryRun};
@@ -34,6 +35,8 @@ pub struct Context {
     pub version: String,
     /// Added to every git call and every session (the tests keep the user's configuration out).
     pub env: Vec<(OsString, OsString)>,
+    /// What discovery found (D96): the format specs every check layers over the built-ins.
+    pub discovery: Discovery,
 }
 
 impl Context {
@@ -42,6 +45,16 @@ impl Context {
         journal::dir(&self.git_dir)
     }
 
+    /// The built-in formats with the discovered ones layered on top (D95).
+    pub fn registry(&self) -> Result<Registry, String> {
+        let layers = self
+            .discovery
+            .formats()
+            .iter()
+            .map(|s| (s.id.as_str(), s.text.as_str(), &s.source));
+        Registry::with_layers(layers).map_err(|e| format!("the formats: {e}"))
+    }
+
     /// The plan's path as a journal records it: repository-relative with forward slashes, or
     /// canonical when it is outside the repository (D76).
     pub fn plan_path(&self, file: &Path) -> String {
@@ -384,7 +397,7 @@ fn preview(
         ctx.root.join(plan)
     };
     let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
-    let registry = Registry::with_builtins().map_err(|e| format!("the built-in formats: {e}"))?;
+    let registry = ctx.registry()?;
     let name = path.display().to_string();
     let parsed = parse_plan(&registry, &name, &bytes, None)
         .map_err(|_| format!("{plan} does not parse any more: resume would refuse it"))?;
@@ -454,7 +467,7 @@ fn scan(dir: &Path) -> Result<Vec<(PathBuf, Stamp)>, String> {
 
 fn check(ctx: &Context, path: &Path) -> Result<PlanFile, String> {
     let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
-    let registry = Registry::with_builtins().map_err(|e| format!("the built-in formats: {e}"))?;
+    let registry = ctx.registry()?;
     let name = path.file_name().map_or_else(
         || path.display().to_string(),
         |n| n.to_string_lossy().into_owned(),
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t3-crates-planrun-tui-src-jobs-rs.patch"
Remove-Item "$env:TEMP\p7-t3-crates-planrun-tui-src-jobs-rs.patch"
```

File `crates/planrun-tui/src/terminal.rs` (full contents):

```rust
//! `planrun tui` on a real terminal (decision D94): stdin and stdout must be terminals; the
//! repository is the one of the current directory (D88); the plans directory defaults to
//! `docs/plans` under its root. One loop polls crossterm's events and drains the app's channel
//! between polls; there is no input thread, which would read the keys meant for the `[o]` shell.
//! Interrupts are installed once, before the terminal is set up; a panic restores the terminal
//! only when it is the render thread's.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::thread::ThreadId;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{self, EnterAlternateScreen};

use planrun_exec::discover::Discovery;
use planrun_exec::shell::{InterruptsIgnored, install_interrupt_handler};
use planrun_exec::workspace::Workspace;

use crate::app::{App, system_clock};
use crate::driver::{Core, Hooks};
use crate::glyph::Glyphs;
use crate::jobs::{Context, UpdateCheck};
use crate::keys;

/// How long one poll for a key waits before the app's channel is drained again.
const POLL: Duration = Duration::from_millis(50);

/// What planrun-cli hands the TUI.
pub struct Host {
    /// The current directory: its repository is the target.
    pub dir: PathBuf,
    /// The plans directory as given (relative to `dir`), or `None` for `docs/plans` under the
    /// repository's root.
    pub plans: Option<PathBuf>,
    /// planrun's version.
    pub version: String,
    /// The update check.
    pub update: Option<UpdateCheck>,
}

/// The default plans directory, under the repository's root (D94).
pub const DEFAULT_PLANS: &str = "docs/plans";

/// Builds the context for the repository of `dir`.
/// Discovery runs from `dir` with `user` as the user's config directory (D96); a spec that cannot
/// be used stops the TUI before it starts.
pub fn context(
    dir: &Path,
    plans: Option<&Path>,
    user: Option<&Path>,
    version: &str,
    env: Vec<(std::ffi::OsString, std::ffi::OsString)>,
) -> Result<Context, String> {
    let ws = Workspace::open(dir, env.clone()).map_err(|e| e.to_string())?;
    let discovery = Discovery::open(dir, user).map_err(|e| format!("error[{}]: {e}", e.code()))?;
    let git_dir = ws.git().git_dir().map_err(|e| e.to_string())?;
    let root = ws.root().to_path_buf();
    let plans_dir = match plans {
        Some(p) if p.is_absolute() => p.to_path_buf(),
        Some(p) => dir.join(p),
        None => root.join(DEFAULT_PLANS),
    };
    let canonical = std::fs::canonicalize(&plans_dir).unwrap_or_else(|_| plans_dir.clone());
    let plans_label = match canonical.strip_prefix(&root) {
        Ok(rel) if rel.as_os_str().is_empty() => ".".to_owned(),
        Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
        Err(_) => plans_dir.display().to_string(),
    };
    let repo = root.file_name().map_or_else(
        || root.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    Ok(Context {
        target: root.display().to_string(),
        repo,
        root,
        git_dir,
        plans_dir,
        plans_label,
        version: version.to_owned(),
        env,
        discovery,
    })
    .and_then(|ctx| ctx.registry().map(|_| ctx))
}

/// Runs the TUI until it quits. `Err` is the message for a usage problem: not a terminal, not a
/// repository, or a terminal that could not be set up.
pub fn run(host: Host) -> Result<(), String> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(
            "planrun tui needs a terminal on stdin and stdout; `planrun run` is the line version"
                .to_owned(),
        );
    }
    let user = planrun_exec::discover::user_dir();
    let ctx = context(
        &host.dir,
        host.plans.as_deref(),
        user.as_deref(),
        &host.version,
        Vec::new(),
    )?;
    let app = App::new(ctx, Glyphs::from_env(), system_clock(), host.update);
    install_interrupt_handler();
    let report = std::panic::take_hook();
    let terminal = ratatui::init();
    install_panic_hook(report);
    let result = drive(Core::new(app, terminal, Box::new(Crossterm)));
    ratatui::restore();
    result
}

/// What a panic does to the terminal (D94).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnPanic {
    /// The render thread's: restore the terminal, then report as Rust does.
    Restore,
    /// Another thread's (the runner's, a job's): leave the terminal and print nothing; the
    /// thread catches the panic and the app shows it as that work's failure.
    Leave,
}

/// What a panic on `current` does, when `render` owns the terminal.
pub fn on_panic(render: ThreadId, current: ThreadId) -> OnPanic {
    if render == current {
        OnPanic::Restore
    } else {
        OnPanic::Leave
    }
}

/// The hook a panic report goes through.
type Report = Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Sync + Send>;

/// Replaces ratatui's hook, which would restore the terminal on every thread's panic, caught or
/// not: `report` (the hook before `ratatui::init`) runs only for the render thread's.
fn install_panic_hook(report: Report) {
    let render = std::thread::current().id();
    std::panic::set_hook(Box::new(move |info| {
        if on_panic(render, std::thread::current().id()) == OnPanic::Restore {
            ratatui::restore();
            report(info);
        }
    }));
}

/// The terminal's hooks.
struct Crossterm;

impl Hooks for Crossterm {
    fn shell(&mut self, root: &Path) {
        ratatui::restore();
        let program = if cfg!(windows) { "pwsh" } else { "bash" };
        println!(
            "{program} at {}; this is not the plan's session (its variables are not here). Exit to return.",
            root.display()
        );
        {
            let _quiet = InterruptsIgnored::new();
            if let Err(e) = std::process::Command::new(program)
                .current_dir(root)
                .status()
            {
                println!("{program} did not start: {e}");
            }
        }
        let _ = terminal::enable_raw_mode();
        let _ = execute!(std::io::stdout(), EnterAlternateScreen);
        while event::poll(Duration::ZERO).unwrap_or(false) {
            let _ = event::read();
        }
    }

    fn end_now(&mut self) {
        ratatui::restore();
        planrun_exec::shell::end_now(130);
    }
}

fn drive<B: ratatui::backend::Backend>(mut core: Core<B>) -> Result<(), String> {
    let term = |e: B::Error| format!("the terminal: {e}");
    let io = |e: std::io::Error| format!("the terminal: {e}");
    core.start();
    core.draw().map_err(term)?;
    while !core.done() {
        core.pump();
        if event::poll(POLL).map_err(io)? {
            match event::read().map_err(io)? {
                Event::Key(ev) => {
                    if let Some(key) = keys::key(&ev) {
                        core.key(key);
                    }
                }
                Event::Resize(..) => core.redraw().map_err(term)?,
                _ => {}
            }
        }
        core.tick();
        core.pump();
        core.draw().map_err(term)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_render_thread_s_panic_restores_the_terminal() {
        let render = std::thread::current().id();
        let other = std::thread::spawn(|| std::thread::current().id())
            .join()
            .unwrap_or_else(|_| panic!("the thread panicked"));
        assert_eq!(on_panic(render, render), OnPanic::Restore);
        assert_eq!(on_panic(render, other), OnPanic::Leave);
    }
}
```

File `crates/planrun-tui/tests/support/mod.rs` (full contents):

```rust
//! The TUI's test harness (decision D93): a greeter fixture copy, the synthetic plans in a
//! plans directory beside it, an app with a fixed clock and fixed labels, and a core over a
//! 120 × 36 `TestBackend`.

#![allow(dead_code)]

#[path = "../../../../tests/support/fixture.rs"]
pub mod fixture;

use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ratatui::Terminal;
use ratatui::backend::TestBackend;

use fixture::Fixture;
use planrun_tui::app::App;
use planrun_tui::driver::Core;
use planrun_tui::glyph::Glyphs;
use planrun_tui::grid::grid;
use planrun_tui::jobs::Bg;
use planrun_tui::terminal::context;

/// 2026-11-27 12:34:56 UTC.
pub const NOW: u64 = 1_795_782_896_000;

/// The synthetic plans, and the corpus README, which is not a plan.
pub const PLANS: [&str; 5] = [
    "README.md",
    "assay-style.md",
    "before-after-style.md",
    "devready-style.md",
    "superpowers-style.md",
];

/// The tests' hooks: they count the shell round trips and the ends.
#[derive(Clone, Default)]
pub struct Counted {
    /// `[o]` shells run.
    pub shells: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    /// Confirmed Ctrl-Cs.
    pub ends: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl planrun_tui::driver::Hooks for Counted {
    fn shell(&mut self, _root: &std::path::Path) {
        self.shells
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    fn end_now(&mut self) {
        self.ends.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

/// A fixture with its plans directory.
pub struct Setup {
    /// The repository.
    pub fixture: Fixture,
    /// The plans directory, outside the repository.
    pub plans: PathBuf,
}

/// The greeter fixture and a plans directory with `plans`, each file's time set to [`NOW`].
pub fn setup(plans: &[&str]) -> Setup {
    let fixture = Fixture::greeter(false);
    let dir = fixture.outside().join("plans");
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    let corpus =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/synthetic");
    for name in plans {
        let target = dir.join(name);
        std::fs::copy(corpus.join(name), &target).unwrap_or_else(|e| panic!("{name}: {e}"));
        let file = File::options()
            .write(true)
            .open(&target)
            .unwrap_or_else(|e| panic!("{e}"));
        file.set_modified(UNIX_EPOCH + Duration::from_millis(NOW))
            .unwrap_or_else(|e| panic!("{e}"));
    }
    Setup {
        fixture,
        plans: dir,
    }
}

impl Setup {
    /// The app on this setup, with the labels of the design's artboards.
    pub fn app(&self) -> App {
        self.app_with(None)
    }

    /// The app with `update` as its update check.
    pub fn app_with(&self, update: Option<planrun_tui::jobs::UpdateCheck>) -> App {
        let mut ctx = context(
            self.fixture.path(),
            Some(&self.plans),
            None,
            "0.1.0",
            self.fixture.env(),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        ctx.repo = "greeter".to_owned();
        ctx.target = r"C:\src\greeter".to_owned();
        ctx.plans_label = "./plans".to_owned();
        App::new(ctx, Glyphs::unicode(), Arc::new(|| NOW), update)
    }

    /// A started core whose plans are all checked.
    pub fn core(&self) -> Core<TestBackend> {
        self.core_with(Counted::default())
    }

    /// A started core with `hooks`, its plans all checked.
    pub fn core_with(&self, hooks: Counted) -> Core<TestBackend> {
        let terminal = Terminal::new(TestBackend::new(120, 36)).unwrap_or_else(|e| panic!("{e}"));
        let mut core = Core::new(self.app(), terminal, Box::new(hooks));
        core.start();
        let n = std::fs::read_dir(&self.plans).map_or(0, |d| d.count());
        let checked = core.wait(Duration::from_secs(60), |app| {
            let entries = &app.plans().entries;
            entries.len() == n && entries.iter().all(|e| matches!(e.check, Bg::Current(_)))
        });
        assert!(checked, "the plans were not checked in time");
        core
    }
}

/// The screen as text, for a snapshot.
pub fn screen(core: &mut Core<TestBackend>) -> String {
    core.draw().unwrap_or_else(|e| panic!("{e}"));
    grid(core.terminal().backend().buffer())
}

/// Whether pwsh is there; without it a test is skipped unless shells are required (D63).
pub fn pwsh() -> bool {
    use planrun_exec::shell::{ShellError, locate};
    match locate(planrun_model::Shell::Pwsh) {
        Ok(_) => true,
        Err(e @ ShellError::Unavailable { .. })
            if std::env::var_os("PLANRUN_REQUIRE_SHELLS").is_none() =>
        {
            eprintln!("skipped: {e}");
            false
        }
        Err(e) => panic!("{e}"),
    }
}

/// The current time, for tests that wait.
pub fn now() -> SystemTime {
    SystemTime::now()
}
```

File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p7-t3-docs-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index 5502cb2..f3439b8 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -1440,6 +1440,24 @@ Amends D89.
   nothing: the thread catches it, and its message comes back to the app as that work's failure
   (a run that ended with an error, a job in its failed state).
 
+## D95 Format layers
+
+Supersedes "a duplicate id is an error" for discovered formats (architecture.md §5, extending.md
+§2). Decided in the P7 brainstorm (docs/superpowers/specs/2026-11-28-discovery-design.md).
+
+- **Three layers:** the built-in formats, then the user's (`<user>/formats/`), then the
+  repository's (`.planrun/formats/`), as D96 finds them. A spec whose id a lower layer has
+  already replaces that format whole: its detection, outline and recognizers; nothing is merged.
+  In one layer two specs cannot share an id, since the file name is the id.
+- **The source is shown.** `check` ends with `ok (<id> (<file>, overrides <what>), <n> tasks)`
+  and `show` prints the same after `format`, the file relative to the repository root when it is
+  under it; a built-in format that replaces nothing is its id alone, so their output does not
+  change for it.
+- **A hand-written `Format`** (`Registry::add_format`) is built in, and a second one with an id
+  stays `DuplicateFormat`.
+- `FormatSource` (built in, user file, repository file) is in planrun-model, which both
+  planrun-parse (the registry's `Origin`) and planrun-exec (discovery) use (D2).
+
 ## D96 Discovery: format specs and plan files
 
 Amends D41 and D9. Decided in the P7 brainstorm (docs/superpowers/specs/2026-11-28-discovery-design.md).
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t3-docs-decisions-md.patch"
Remove-Item "$env:TEMP\p7-t3-docs-decisions-md.patch"
```

**Step 2 — verify**

Targeted: the registry's tests:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-parse --lib registry
```

Expected (exit code 0; summary lines):

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 41 filtered out; finished in …s
```

Targeted: the binary's tests, with a repository format that overrides `assay`:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-cli --test cli
```

Expected (exit code 0; summary lines):

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-cli/src/main.rs crates/planrun-cli/tests/cli.rs crates/planrun-exec/src/discover.rs crates/planrun-parse/src/lib.rs crates/planrun-parse/src/registry.rs crates/planrun-parse/src/registry_tests.rs crates/planrun-tui/src/app_tests.rs crates/planrun-tui/src/jobs.rs crates/planrun-tui/src/terminal.rs crates/planrun-tui/tests/support/mod.rs docs/decisions.md
  $verified = [ordered]@{
      'crates/planrun-cli/src/main.rs' = '6d0aa7a71b95e83389177757be3e88c7971f93dd'
      'crates/planrun-cli/tests/cli.rs' = '4404304834bd65c262029a5366b33f16533d358d'
      'crates/planrun-exec/src/discover.rs' = '9c9b64033d98821ccbaae13e9c3583b97fc4484f'
      'crates/planrun-parse/src/lib.rs' = '717cf1530265793dcdfe8a53e338e6533e4da4ba'
      'crates/planrun-parse/src/registry.rs' = '9016677d9fad8938a39110a9616d87eec97ae509'
      'crates/planrun-parse/src/registry_tests.rs' = '4071594527678e0cbb7225cac083a86b86ce7a54'
      'crates/planrun-tui/src/app_tests.rs' = 'aa0202deeb79a7f3a508ceea50b5086fe0b439fa'
      'crates/planrun-tui/src/jobs.rs' = 'a92f41cc097bd5ce6c44dd90b21992c65f4de9f3'
      'crates/planrun-tui/src/terminal.rs' = '5ade38ef22822349aa0c008c547df0fc9505d357'
      'crates/planrun-tui/tests/support/mod.rs' = '4699933feb638e82c1db0c5cb60bfeaecf8461f1'
      'docs/decisions.md' = 'f3439b84f5eb0cd076a98e7b858c3a2bcf44978c'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P7-T3: format layers in the registry, the CLI and the TUI (D95)" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P7-T4: plans by name on the command line (D97)

**Goal:** `Discovery::plans`: the repository's tracked and untracked files that are not ignored and that a glob matches, from one `git ls-files`, without missing files and symbolic links. `Discovery::resolve`: a path when the argument has a directory part, ends in `.md` or exists; otherwise a name, matched without case against the plans' file stems (`plan-not-found`, `plan-ambiguous`). `check`, `show`, `run`, `resume` and `status` take a name. D97.

**Files**

- Modify `crates/planrun-cli/src/main.rs`
- Modify `crates/planrun-cli/tests/cli.rs`
- Modify `crates/planrun-exec/src/discover.rs`
- Modify `crates/planrun-exec/src/discover_tests.rs`
- Modify `docs/decisions.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/src/main.rs`, `crates/planrun-exec/src/discover.rs`, `docs/decisions.md`.

**Step 1 — write the files**

File `crates/planrun-cli/src/main.rs` (exact diff; save the block as `$env:TEMP\p7-t4-crates-planrun-cli-src-main-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/main.rs b/crates/planrun-cli/src/main.rs
index 6d0aa7a..aaa4e5e 100644
--- a/crates/planrun-cli/src/main.rs
+++ b/crates/planrun-cli/src/main.rs
@@ -34,7 +34,7 @@ struct Cli {
 enum Command {
     /// Parse and check a plan; print diagnostics and the coverage report. Writes nothing.
     Check {
-        /// The plan file.
+        /// The plan file, or a name the discovered plans' file names contain (D97).
         plan: PathBuf,
         /// Force a format id instead of detecting one.
         #[arg(long)]
@@ -42,7 +42,7 @@ enum Command {
     },
     /// Print the typed plan, or one task (`P1-T2`) or step (`P1-T2/1`, `P1-T2/intro`).
     Show {
-        /// The plan file.
+        /// The plan file, or a name the discovered plans' file names contain (D97).
         plan: PathBuf,
         /// A task or task/step.
         target: Option<String>,
@@ -52,7 +52,7 @@ enum Command {
     },
     /// Execute a plan in the git repository of the current directory, task by task.
     Run {
-        /// The plan file.
+        /// The plan file, or a name the discovered plans' file names contain (D97).
         plan: PathBuf,
         /// Parse, check and validate against a virtual tree; change nothing, run nothing.
         #[arg(long)]
@@ -78,7 +78,7 @@ enum Command {
     },
     /// Go on with the plan's interrupted or stopped run (D14).
     Resume {
-        /// The plan file.
+        /// The plan file, or a name the discovered plans' file names contain (D97).
         plan: PathBuf,
         /// Force a format id.
         #[arg(long)]
@@ -98,7 +98,7 @@ enum Command {
     },
     /// What the plan's journal says: the tasks done, the one interrupted. Changes nothing.
     Status {
-        /// The plan file.
+        /// The plan file, or a name the discovered plans' file names contain (D97).
         plan: PathBuf,
         /// Force a format id.
         #[arg(long)]
@@ -157,8 +157,8 @@ fn main() -> ExitCode {
                 to: to.clone(),
                 restart: *restart,
             };
-            with_plan(plan, format.as_deref(), &mut out, |p, out| {
-                run::run(plan, p, &opts, out)
+            with_plan(plan, format.as_deref(), &mut out, |file, p, out| {
+                run::run(file, p, &opts, out)
             })
         }
         Command::Resume {
@@ -176,13 +176,13 @@ fn main() -> ExitCode {
                 to: to.clone(),
                 restart: false,
             };
-            with_plan(plan, format.as_deref(), &mut out, |p, out| {
-                run::resume(plan, p, &opts, out)
+            with_plan(plan, format.as_deref(), &mut out, |file, p, out| {
+                run::resume(file, p, &opts, out)
             })
         }
         Command::Status { plan, format } => {
-            with_plan(plan, format.as_deref(), &mut out, |p, out| {
-                run::status(plan, p, out)
+            with_plan(plan, format.as_deref(), &mut out, |file, p, out| {
+                run::status(file, p, out)
             })
         }
         Command::Update { check } => update::run(*check, &mut out),
@@ -225,18 +225,18 @@ fn with_plan(
     path: &Path,
     format: Option<&str>,
     out: &mut String,
-    f: impl FnOnce(&planrun_model::Plan, &mut String) -> u8,
+    f: impl FnOnce(&Path, &planrun_model::Plan, &mut String) -> u8,
 ) -> u8 {
     match load(path, format, out) {
-        Loaded::Parsed(parsed, _) => f(&parsed.plan, out),
+        Loaded::Parsed(parsed, _, file) => f(&file, &parsed.plan, out),
         Loaded::Failed(code) => code,
     }
 }
 
-/// A plan read and parsed with its format's description (D95), or the exit code when it was
-/// not.
+/// A plan read and parsed, with its format's description (D95) and the file it was read from
+/// (D97), or the exit code when it was not.
 enum Loaded {
-    Parsed(Box<Parsed>, String),
+    Parsed(Box<Parsed>, String, PathBuf),
     Failed(u8),
 }
 
@@ -284,8 +284,18 @@ fn load(path: &Path, format: Option<&str>, out: &mut String) -> Loaded {
         Ok(setup) => setup,
         Err(code) => return Loaded::Failed(code),
     };
-    let name = path.display().to_string();
-    let bytes = match std::fs::read(path) {
+    let resolved = match setup.discovery.resolve(path, &[]) {
+        Ok(resolved) => resolved,
+        Err(e) => {
+            out.push_str(&format!("planrun: error[{}]: {e}\n", e.code()));
+            return Loaded::Failed(USAGE);
+        }
+    };
+    if resolved.by_name {
+        eprintln!("planrun: `{}` is {}", path.display(), resolved.shown);
+    }
+    let name = resolved.shown.clone();
+    let bytes = match std::fs::read(&resolved.path) {
         Ok(bytes) => bytes,
         Err(e) => {
             out.push_str(&format!("planrun: {name}: {e}\n"));
@@ -299,7 +309,7 @@ fn load(path: &Path, format: Option<&str>, out: &mut String) -> Loaded {
                 .registry
                 .origin(&id)
                 .map_or_else(|| id.clone(), |o| o.describe(&id, setup.discovery.root()));
-            Loaded::Parsed(Box::new(parsed), label)
+            Loaded::Parsed(Box::new(parsed), label, resolved.path)
         }
         Err(diagnostics) => {
             failed(&name, &bytes, &diagnostics, out);
@@ -321,7 +331,7 @@ fn failed(name: &str, bytes: &[u8], diagnostics: &Diagnostics, out: &mut String)
 
 fn check(path: &Path, format: Option<&str>, out: &mut String) -> u8 {
     let (parsed, label) = match load(path, format, out) {
-        Loaded::Parsed(parsed, label) => (parsed, label),
+        Loaded::Parsed(parsed, label, _) => (parsed, label),
         Loaded::Failed(code) => return code,
     };
     let plan = &parsed.plan;
@@ -337,7 +347,7 @@ fn check(path: &Path, format: Option<&str>, out: &mut String) -> u8 {
 
 fn show(path: &Path, target: Option<&str>, format: Option<&str>, out: &mut String) -> u8 {
     let (parsed, label) = match load(path, format, out) {
-        Loaded::Parsed(parsed, label) => (parsed, label),
+        Loaded::Parsed(parsed, label, _) => (parsed, label),
         Loaded::Failed(code) => return code,
     };
     let viewer = Viewer::new(&parsed.plan);
@@ -376,7 +386,7 @@ fn show(path: &Path, target: Option<&str>, format: Option<&str>, out: &mut Strin
 /// repository cannot be used.
 fn run_dry(path: &Path, format: Option<&str>, out: &mut String) -> u8 {
     let parsed = match load(path, format, out) {
-        Loaded::Parsed(parsed, _) => parsed,
+        Loaded::Parsed(parsed, ..) => parsed,
         Loaded::Failed(code) => return code,
     };
     let plan = &parsed.plan;
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t4-crates-planrun-cli-src-main-rs.patch"
Remove-Item "$env:TEMP\p7-t4-crates-planrun-cli-src-main-rs.patch"
```

File `crates/planrun-cli/tests/cli.rs` (full contents):

```rust
//! The `planrun` binary: exit codes, `check` output, the rendered message of every invalid
//! plan, and `show` of one step. Runs from the workspace root with relative paths, so the output
//! is the same on every machine; the user's config directory is one that does not exist, so
//! their formats stay out (D96).

#[path = "../../../tests/support/fixture.rs"]
mod fixture;

use std::path::{Path, PathBuf};
use std::process::Command;

use fixture::Fixture;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Runs `planrun args…`; returns the exit code and stdout + stderr.
fn planrun(args: &[&str]) -> (i32, String) {
    planrun_in(&root(), args)
}

/// A user config directory that does not exist.
fn no_user_config() -> PathBuf {
    root().join("target/planrun-tests-no-user-config")
}

/// Runs `planrun args…` in `dir`.
fn planrun_in(dir: &Path, args: &[&str]) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_planrun"))
        .args(args)
        .current_dir(dir)
        .env("PLANRUN_CONFIG_DIR", no_user_config())
        .output()
        .unwrap_or_else(|e| panic!("{e}"));
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), text)
}

const SYNTHETIC: &str = "tests/corpus/synthetic/assay-style.md";

#[test]
fn check_passes_the_synthetic_plan() {
    let (code, out) = planrun(&["check", SYNTHETIC]);
    assert_eq!(code, 0, "{out}");
    insta::assert_snapshot!("check_assay_style", out);
}

#[test]
fn check_renders_every_invalid_plan() {
    let dir = root().join("tests/corpus/invalid");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{e}"))
        .map(|e| {
            e.unwrap_or_else(|e| panic!("{e}"))
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|n| n.ends_with(".md") && n != "README.md")
        .collect();
    names.sort();
    let mut all = String::new();
    for name in &names {
        let path = format!("tests/corpus/invalid/{name}");
        let (code, out) = planrun(&["check", &path]);
        assert_eq!(code, 1, "{path}: {out}");
        all.push_str(&format!("==== {name}\n{out}\n"));
    }
    insta::assert_snapshot!("check_invalid_corpus", all);
}

#[test]
fn show_prints_a_step_in_full() {
    let (code, out) = planrun(&["show", SYNTHETIC, "P1-T2/2"]);
    assert_eq!(code, 0, "{out}");
    insta::assert_snapshot!("show_p1_t2_step_2", out);
}

#[test]
fn usage_problems_exit_with_2() {
    assert_eq!(planrun(&["check"]).0, 2);
    assert_eq!(planrun(&["check", "tests/corpus/none.md"]).0, 2);
    assert_eq!(planrun(&["show", SYNTHETIC, "P9-T9"]).0, 2);
    assert_eq!(planrun(&["show", SYNTHETIC, "P1-T1/7"]).0, 2);
    assert_eq!(planrun(&["check", SYNTHETIC, "--format", "nope"]).0, 1);
}

#[test]
fn tui_refuses_a_stdout_that_is_not_a_terminal() {
    let (code, out) = planrun(&["tui"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("planrun tui needs a terminal"), "{out}");
}

/// A fixture copy with the synthetic assay plan at its root.
fn with_plan() -> Fixture {
    let f = Fixture::greeter(false);
    let plan = std::fs::read(root().join(SYNTHETIC)).unwrap_or_else(|e| panic!("{e}"));
    f.write("assay-style.md", &plan);
    f
}

#[test]
fn a_repository_format_overrides_the_built_in_one_and_check_says_so() {
    let f = with_plan();
    let spec = std::fs::read(root().join("formats/assay.toml")).unwrap_or_else(|e| panic!("{e}"));
    f.write(".planrun/formats/assay.toml", &spec);
    let (code, out) = planrun_in(f.path(), &["check", "assay-style.md"]);
    assert_eq!(code, 0, "{out}");
    let last = out.lines().last().unwrap_or_default();
    assert_eq!(
        last,
        "planrun: assay-style.md: ok (assay (.planrun/formats/assay.toml, overrides built-in), 2 tasks)"
    );
    let (code, out) = planrun_in(f.path(), &["show", "assay-style.md"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("format  assay (.planrun/formats/assay.toml, overrides built-in)\n"),
        "{out}"
    );
}

#[test]
fn a_spec_or_config_that_cannot_be_used_stops_with_2() {
    let f = with_plan();
    f.write(".planrun/formats/broken.toml", b"id = 1\n");
    let (code, out) = planrun_in(f.path(), &["check", "assay-style.md"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("broken.toml: format `broken`:"), "{out}");
    std::fs::remove_file(f.path().join(".planrun/formats/broken.toml"))
        .unwrap_or_else(|e| panic!("{e}"));
    f.write(".planrun/config.toml", b"[plans]\nglobs = []\n");
    let (code, out) = planrun_in(f.path(), &["check", "assay-style.md"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("error[config-globs-empty]"), "{out}");
}

#[test]
fn a_plan_is_found_by_a_part_of_its_name() {
    let f = Fixture::greeter(false);
    let plan = std::fs::read(root().join(SYNTHETIC)).unwrap_or_else(|e| panic!("{e}"));
    f.write("docs/plans/2026-01-01-p1-greet.md", &plan);
    f.write("docs/plans/2026-01-02-p1-other.md", b"# Not a plan\n");
    let (code, out) = planrun_in(f.path(), &["check", "GREET"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("planrun: `GREET` is docs/plans/2026-01-01-p1-greet.md\n"),
        "{out}"
    );
    assert!(
        out.contains("planrun: docs/plans/2026-01-01-p1-greet.md: ok (assay, 2 tasks)\n"),
        "{out}"
    );
    let (code, out) = planrun_in(f.path(), &["status", "p1"]);
    assert_eq!(code, 2, "{out}");
    assert!(
        out.contains("error[plan-ambiguous]: `p1` names 2 plan files"),
        "{out}"
    );
    let (code, out) = planrun_in(f.path(), &["show", "farewell"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("error[plan-not-found]"), "{out}");
}
```

File `crates/planrun-exec/src/discover.rs` (exact diff; save the block as `$env:TEMP\p7-t4-crates-planrun-exec-src-discover-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-exec/src/discover.rs b/crates/planrun-exec/src/discover.rs
index 9c9b640..539ff4c 100644
--- a/crates/planrun-exec/src/discover.rs
+++ b/crates/planrun-exec/src/discover.rs
@@ -1,14 +1,18 @@
 //! Discovery (D96): what the repository of the current directory and the user's config directory
 //! hold for planrun. Format specs (`.planrun/formats/*.toml`, `<user>/formats/*.toml`) and the
-//! repository's `.planrun/config.toml` are read when a [`Discovery`] opens; nothing here writes,
-//! and nothing here parses a plan (D2): the spec texts go to planrun-parse's registry.
+//! repository's `.planrun/config.toml` are read when a [`Discovery`] opens; the plan files the
+//! globs match are listed on request with one `git ls-files` ([`Discovery::plans`]), and a plan
+//! named on the command line is found among them ([`Discovery::resolve`], D97). Nothing here
+//! writes, and nothing here parses a plan (D2): the spec texts go to planrun-parse's registry.
 
 use std::ffi::OsString;
 use std::path::{Path, PathBuf};
 
-use planrun_model::{FormatSource, PathPattern, PatternError};
+use planrun_model::{FormatSource, PathPattern, PatternError, RepoPath};
 use serde::Deserialize;
 
+use crate::git::{Git, GitError};
+
 /// The plan globs when `.planrun/config.toml` names none (D96).
 pub const DEFAULT_GLOBS: [&str; 3] = [
     "docs/plans/*.md",
@@ -45,6 +49,7 @@ pub struct Config {
 /// What the repository and the user's config directory hold for planrun.
 #[derive(Debug, Clone)]
 pub struct Discovery {
+    cwd: PathBuf,
     root: Option<PathBuf>,
     config: Config,
     formats: Vec<FoundSpec>,
@@ -95,6 +100,33 @@ pub enum DiscoveryError {
     /// `plans.globs` is empty.
     #[error("{0}: plans.globs is empty; leave it out for the defaults")]
     ConfigGlobsEmpty(PathBuf),
+    /// `git ls-files` failed.
+    #[error("listing the plan files: {0}")]
+    Git(GitError),
+    /// No discovered plan has the name in its file stem.
+    #[error("no plan file the globs find has `{0}` in its name; give its path instead")]
+    PlanNotFound(String),
+    /// Several do.
+    #[error("`{name}` names {} plan files: {}; give more of the name, or the path",
+        .candidates.len(), .candidates.join(", "))]
+    PlanAmbiguous {
+        /// The name as given.
+        name: String,
+        /// The plans it names, repository-relative.
+        candidates: Vec<String>,
+    },
+}
+
+/// A plan argument, resolved (D97).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Resolved {
+    /// The file to read.
+    pub path: PathBuf,
+    /// How to name it in messages: the argument as given, or the repository-relative path of
+    /// the plan a name found.
+    pub shown: String,
+    /// Whether the argument was a name.
+    pub by_name: bool,
 }
 
 impl DiscoveryError {
@@ -108,6 +140,9 @@ impl DiscoveryError {
             DiscoveryError::ConfigParse { .. } => "config-parse",
             DiscoveryError::ConfigGlob { .. } => "config-glob",
             DiscoveryError::ConfigGlobsEmpty(_) => "config-globs-empty",
+            DiscoveryError::Git(_) => "git-ls-files",
+            DiscoveryError::PlanNotFound(_) => "plan-not-found",
+            DiscoveryError::PlanAmbiguous { .. } => "plan-ambiguous",
         }
     }
 }
@@ -129,6 +164,7 @@ impl Default for Discovery {
     /// No repository, no user directory: the default globs and no formats.
     fn default() -> Self {
         Self {
+            cwd: PathBuf::new(),
             root: None,
             config: default_config(),
             formats: Vec::new(),
@@ -157,6 +193,7 @@ impl Discovery {
             None => default_config(),
         };
         Ok(Self {
+            cwd: cwd.to_path_buf(),
             root,
             config,
             formats,
@@ -178,6 +215,87 @@ impl Discovery {
     pub fn formats(&self) -> &[FoundSpec] {
         &self.formats
     }
+
+    /// The plan files: the repository's tracked and untracked files that are not ignored
+    /// (`git ls-files -z --cached --others --exclude-standard`, git with `env` added), those a
+    /// glob matches, without a file that is missing (deleted but still in the index) or a
+    /// symbolic link. Sorted by path, each once. No format detection: that is the caller's.
+    pub fn plans(&self, env: &[(OsString, OsString)]) -> Result<Vec<RepoPath>, DiscoveryError> {
+        let root = self.repository()?;
+        let git = Git::new(root, env.to_vec());
+        let args = [
+            "ls-files",
+            "-z",
+            "--cached",
+            "--others",
+            "--exclude-standard",
+        ];
+        let out = git.run(&args, None, &[]).map_err(DiscoveryError::Git)?;
+        let listed = String::from_utf8_lossy(&out);
+        let mut plans: Vec<RepoPath> = listed
+            .split('\0')
+            .filter(|f| !f.is_empty())
+            .filter_map(|f| RepoPath::new(f).ok())
+            .filter(|p| self.config.globs.iter().any(|g| g.matches(p)))
+            .filter(|p| std::fs::symlink_metadata(root.join(p.as_str())).is_ok_and(|m| m.is_file()))
+            .collect();
+        plans.sort_by(|a, b| a.as_str().cmp(b.as_str()));
+        plans.dedup();
+        Ok(plans)
+    }
+
+    /// A plan argument (D97): a path when it has a directory part, ends in `.md` or names a
+    /// file that exists; otherwise a name, matched case-insensitively against the file stems of
+    /// [`Discovery::plans`]. One match is the plan; none is `plan-not-found`, several are
+    /// `plan-ambiguous`.
+    pub fn resolve(
+        &self,
+        arg: &Path,
+        env: &[(OsString, OsString)],
+    ) -> Result<Resolved, DiscoveryError> {
+        let is_path = arg.components().count() != 1
+            || arg.extension().is_some_and(|x| x == "md")
+            || arg.exists();
+        if is_path {
+            return Ok(Resolved {
+                path: arg.to_path_buf(),
+                shown: arg.display().to_string(),
+                by_name: false,
+            });
+        }
+        let name = arg.to_string_lossy().into_owned();
+        let needle = name.to_lowercase();
+        let found: Vec<RepoPath> = self
+            .plans(env)?
+            .into_iter()
+            .filter(|p| stem(p).to_lowercase().contains(&needle))
+            .collect();
+        match found.as_slice() {
+            [] => Err(DiscoveryError::PlanNotFound(name)),
+            [one] => Ok(Resolved {
+                path: self.repository()?.join(one.as_str()),
+                shown: one.as_str().to_owned(),
+                by_name: true,
+            }),
+            many => Err(DiscoveryError::PlanAmbiguous {
+                name,
+                candidates: many.iter().map(|p| p.as_str().to_owned()).collect(),
+            }),
+        }
+    }
+
+    /// The root, or `not-a-repository`.
+    fn repository(&self) -> Result<&Path, DiscoveryError> {
+        self.root
+            .as_deref()
+            .ok_or_else(|| DiscoveryError::NotARepository(self.cwd.clone()))
+    }
+}
+
+/// A plan file's stem: its file name without `.md`.
+fn stem(path: &RepoPath) -> &str {
+    let name = path.file_name();
+    name.strip_suffix(".md").unwrap_or(name)
 }
 
 /// The user's config directory (D96): `PLANRUN_CONFIG_DIR` when set; else `%APPDATA%\planrun` on
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t4-crates-planrun-exec-src-discover-rs.patch"
Remove-Item "$env:TEMP\p7-t4-crates-planrun-exec-src-discover-rs.patch"
```

File `crates/planrun-exec/src/discover_tests.rs` (full contents):

```rust
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
```

File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p7-t4-docs-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index f3439b8..e15a6fe 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -1483,3 +1483,17 @@ Amends D41 and D9. Decided in the P7 brainstorm (docs/superpowers/specs/2026-11-
   segments (`PathPattern::discovery_glob`). `**` inside a segment is refused. Manifest globs keep
   refusing `**`, so no plan changes meaning.
 - **`toml` in planrun-exec**, for the config file: a crate the workspace has already (D9).
+
+## D97 Plans by name
+
+Amends D94. Decided in the P7 brainstorm (docs/superpowers/specs/2026-11-28-discovery-design.md).
+
+- **`check`, `show`, `run`, `resume` and `status` take a path or a name.** An argument with a
+  directory part, one that ends in `.md`, or one that names an existing file is a path, used as
+  given; a missing file is the usual I/O error, never a name lookup.
+- **Otherwise it is a name:** the discovered plans (D96) whose file stem contains it, compared
+  without case. One is the plan, and planrun says which on stderr (`` planrun: `p6a` is
+  docs/plans/…md ``) before anything else; none is `plan-not-found`, several are
+  `plan-ambiguous`, which lists them. Both exit with code 2. A file no format detects counts as
+  a candidate too: it can make a name ambiguous, never make it find the wrong plan.
+- **Messages name the plan** by its repository-relative path when a name found it.
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t4-docs-decisions-md.patch"
Remove-Item "$env:TEMP\p7-t4-docs-decisions-md.patch"
```

**Step 2 — verify**

Targeted: the discover module's tests:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-exec --lib discover
```

Expected (exit code 0; summary lines):

```text
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in …s
```

Targeted: the binary's tests, with a plan found by name:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-cli --test cli
```

Expected (exit code 0; summary lines):

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-cli/src/main.rs crates/planrun-cli/tests/cli.rs crates/planrun-exec/src/discover.rs crates/planrun-exec/src/discover_tests.rs docs/decisions.md
  $verified = [ordered]@{
      'crates/planrun-cli/src/main.rs' = 'aaa4e5e59cd222b34394f421f5e85447d20abb8d'
      'crates/planrun-cli/tests/cli.rs' = '840261f9ad90a1e6fb2b45eeae32d2fe19a4229d'
      'crates/planrun-exec/src/discover.rs' = '539ff4c2aea15888e64ee30665f4f9cb500c89a7'
      'crates/planrun-exec/src/discover_tests.rs' = 'edc870f6169c18d76a82f85c8a5d385603fcbb1e'
      'docs/decisions.md' = 'e15a6fe80c6cdac8f18e21ce437b533035c10e1b'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P7-T4: plans by name on the command line (D97)" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P7-T5: planrun list

**Goal:** `planrun list`: each discovered file with its format (and source) and its journal's state, or "not a plan", or its error count; exit 1 when a file a format detects has errors, 2 outside a repository.

**Files**

- Create `crates/planrun-cli/src/list.rs`
- Modify `crates/planrun-cli/src/main.rs`
- Modify `crates/planrun-cli/tests/cli.rs`
- Create `crates/planrun-cli/tests/snapshots/cli__list_fixture.snap`

Over 300 lines, so it comes as an exact diff: `crates/planrun-cli/src/main.rs`.

**Step 1 — write the files**

File `crates/planrun-cli/src/list.rs` (full contents):

```rust
//! `planrun list` (D97): the plan files discovery finds (D96), each with its format and the
//! state of its journal, one line each. Reads; changes nothing.

use planrun_exec::journal;
use planrun_exec::workspace::Workspace;
use planrun_parse::parse_plan;
use planrun_tui::jobs::{RunState, run_state};

use crate::{OK, PLAN_ERRORS, Setup, USAGE};

/// One discovered file, as listed.
struct Row {
    path: String,
    what: String,
    run: String,
}

/// Lists the discovered plans: exit 0, 1 when a file a format detects has errors, 2 outside a
/// repository or when git fails.
pub fn list(setup: &Setup, out: &mut String) -> u8 {
    let plans = match setup.discovery.plans(&[]) {
        Ok(plans) => plans,
        Err(e) => {
            out.push_str(&format!("planrun: error[{}]: {e}\n", e.code()));
            return USAGE;
        }
    };
    let Some(root) = setup.discovery.root() else {
        return USAGE;
    };
    let git_dir = match Workspace::open(root, Vec::new()).and_then(|ws| Ok(ws.git().git_dir()?)) {
        Ok(dir) => dir,
        Err(e) => {
            out.push_str(&format!("planrun: {e}\n"));
            return USAGE;
        }
    };
    let (mut rows, mut errors, mut not_plans) = (Vec::new(), 0, 0);
    for plan in &plans {
        let path = plan.as_str().to_owned();
        let bytes = match std::fs::read(root.join(plan.as_str())) {
            Ok(bytes) => bytes,
            Err(e) => {
                out.push_str(&format!("planrun: {path}: {e}\n"));
                return USAGE;
            }
        };
        let (what, run) = match parse_plan(&setup.registry, &path, &bytes, None) {
            Ok(parsed) => {
                let id = parsed.plan.format.0.to_string();
                let format = setup
                    .registry
                    .origin(&id)
                    .map_or_else(|| id.clone(), |o| o.describe(&id, Some(root)));
                let tasks = parsed.plan.tasks.len();
                let plural = if tasks == 1 { "" } else { "s" };
                let state = journal_state(&git_dir, plan.file_name(), &path);
                (format!("{format}, {tasks} task{plural}"), state)
            }
            Err(d) if d.0.iter().any(|d| d.code == "no-format") => {
                not_plans += 1;
                ("not a plan".to_owned(), String::new())
            }
            Err(d) => {
                errors += 1;
                let n = d.error_codes().len();
                let plural = if n == 1 { "" } else { "s" };
                (
                    format!("{n} error{plural} (planrun check {path})"),
                    String::new(),
                )
            }
        };
        rows.push(Row { path, what, run });
    }
    let width = rows
        .iter()
        .map(|r| r.path.chars().count())
        .max()
        .unwrap_or(0);
    for row in &rows {
        let line = format!("{:width$}  {}  {}", row.path, row.what, row.run);
        out.push_str(line.trim_end());
        out.push('\n');
    }
    let globs = match &setup.discovery.config().file {
        Some(_) => "the globs of .planrun/config.toml".to_owned(),
        None => "the default globs".to_owned(),
    };
    let found = rows.len() - not_plans;
    out.push_str(&format!(
        "planrun: {found} plan{} and {not_plans} other file{} from {globs}\n",
        if found == 1 { "" } else { "s" },
        if not_plans == 1 { "" } else { "s" },
    ));
    if errors > 0 { PLAN_ERRORS } else { OK }
}

/// The plan's journal (D76) in words.
fn journal_state(git_dir: &std::path::Path, file_name: &str, plan_path: &str) -> String {
    let stem = file_name.strip_suffix(".md").unwrap_or(file_name);
    let lines = match journal::read(&journal::path_for(git_dir, stem)) {
        Ok(lines) => lines,
        Err(e) => return format!("journal unreadable: {e}"),
    };
    match run_state(&lines, plan_path) {
        RunState::NotRun => "not run".to_owned(),
        RunState::Done(_) => "done".to_owned(),
        RunState::Stopped => "stopped; resume goes on".to_owned(),
        RunState::Interrupted => "interrupted; resume goes on".to_owned(),
        RunState::Foreign(other) => format!("its journal is {other}'s"),
        RunState::Unreadable(e) => format!("journal unreadable: {e}"),
    }
}
```

File `crates/planrun-cli/src/main.rs` (exact diff; save the block as `$env:TEMP\p7-t5-crates-planrun-cli-src-main-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/main.rs b/crates/planrun-cli/src/main.rs
index aaa4e5e..deeddff 100644
--- a/crates/planrun-cli/src/main.rs
+++ b/crates/planrun-cli/src/main.rs
@@ -5,6 +5,7 @@
 mod dry_run;
 #[cfg(feature = "e2e")]
 mod e2e;
+mod list;
 mod render;
 mod run;
 mod ui;
@@ -104,6 +105,9 @@ enum Command {
         #[arg(long)]
         format: Option<String>,
     },
+    /// The plan files of the current directory's repository, with their formats and journals
+    /// (D96, D97). Changes nothing.
+    List,
     /// The TUI over the repository of the current directory (docs/tui-design.md, D84–D94).
     Tui {
         /// The directory of plan files; `docs/plans` under the repository's root when not
@@ -185,6 +189,10 @@ fn main() -> ExitCode {
                 run::status(file, p, out)
             })
         }
+        Command::List => match setup(&mut out) {
+            Ok(setup) => list::list(&setup, &mut out),
+            Err(code) => code,
+        },
         Command::Update { check } => update::run(*check, &mut out),
         Command::Tui { plans_dir } => tui(plans_dir.clone(), &mut out),
     };
@@ -241,14 +249,16 @@ enum Loaded {
 }
 
 /// What discovery found from the current directory (D96), and the registry built from it.
-struct Setup {
-    discovery: Discovery,
-    registry: Registry,
+pub(crate) struct Setup {
+    /// What discovery found.
+    pub(crate) discovery: Discovery,
+    /// The built-in formats with the discovered ones on top (D95).
+    pub(crate) registry: Registry,
 }
 
 /// Discovers from the current directory and layers the formats found over the built-ins; a
 /// problem is a usage problem (exit code 2).
-fn setup(out: &mut String) -> Result<Setup, u8> {
+pub(crate) fn setup(out: &mut String) -> Result<Setup, u8> {
     let here = match std::env::current_dir() {
         Ok(dir) => dir,
         Err(e) => {
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t5-crates-planrun-cli-src-main-rs.patch"
Remove-Item "$env:TEMP\p7-t5-crates-planrun-cli-src-main-rs.patch"
```

File `crates/planrun-cli/tests/cli.rs` (full contents):

```rust
//! The `planrun` binary: exit codes, `check` output, the rendered message of every invalid
//! plan, and `show` of one step. Runs from the workspace root with relative paths, so the output
//! is the same on every machine; the user's config directory is one that does not exist, so
//! their formats stay out (D96).

#[path = "../../../tests/support/fixture.rs"]
mod fixture;

use std::path::{Path, PathBuf};
use std::process::Command;

use fixture::Fixture;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Runs `planrun args…`; returns the exit code and stdout + stderr.
fn planrun(args: &[&str]) -> (i32, String) {
    planrun_in(&root(), args)
}

/// A user config directory that does not exist.
fn no_user_config() -> PathBuf {
    root().join("target/planrun-tests-no-user-config")
}

/// Runs `planrun args…` in `dir`.
fn planrun_in(dir: &Path, args: &[&str]) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_planrun"))
        .args(args)
        .current_dir(dir)
        .env("PLANRUN_CONFIG_DIR", no_user_config())
        .output()
        .unwrap_or_else(|e| panic!("{e}"));
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), text)
}

const SYNTHETIC: &str = "tests/corpus/synthetic/assay-style.md";

#[test]
fn check_passes_the_synthetic_plan() {
    let (code, out) = planrun(&["check", SYNTHETIC]);
    assert_eq!(code, 0, "{out}");
    insta::assert_snapshot!("check_assay_style", out);
}

#[test]
fn check_renders_every_invalid_plan() {
    let dir = root().join("tests/corpus/invalid");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{e}"))
        .map(|e| {
            e.unwrap_or_else(|e| panic!("{e}"))
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|n| n.ends_with(".md") && n != "README.md")
        .collect();
    names.sort();
    let mut all = String::new();
    for name in &names {
        let path = format!("tests/corpus/invalid/{name}");
        let (code, out) = planrun(&["check", &path]);
        assert_eq!(code, 1, "{path}: {out}");
        all.push_str(&format!("==== {name}\n{out}\n"));
    }
    insta::assert_snapshot!("check_invalid_corpus", all);
}

#[test]
fn show_prints_a_step_in_full() {
    let (code, out) = planrun(&["show", SYNTHETIC, "P1-T2/2"]);
    assert_eq!(code, 0, "{out}");
    insta::assert_snapshot!("show_p1_t2_step_2", out);
}

#[test]
fn usage_problems_exit_with_2() {
    assert_eq!(planrun(&["check"]).0, 2);
    assert_eq!(planrun(&["check", "tests/corpus/none.md"]).0, 2);
    assert_eq!(planrun(&["show", SYNTHETIC, "P9-T9"]).0, 2);
    assert_eq!(planrun(&["show", SYNTHETIC, "P1-T1/7"]).0, 2);
    assert_eq!(planrun(&["check", SYNTHETIC, "--format", "nope"]).0, 1);
}

#[test]
fn tui_refuses_a_stdout_that_is_not_a_terminal() {
    let (code, out) = planrun(&["tui"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("planrun tui needs a terminal"), "{out}");
}

/// A fixture copy with the synthetic assay plan at its root.
fn with_plan() -> Fixture {
    let f = Fixture::greeter(false);
    let plan = std::fs::read(root().join(SYNTHETIC)).unwrap_or_else(|e| panic!("{e}"));
    f.write("assay-style.md", &plan);
    f
}

#[test]
fn a_repository_format_overrides_the_built_in_one_and_check_says_so() {
    let f = with_plan();
    let spec = std::fs::read(root().join("formats/assay.toml")).unwrap_or_else(|e| panic!("{e}"));
    f.write(".planrun/formats/assay.toml", &spec);
    let (code, out) = planrun_in(f.path(), &["check", "assay-style.md"]);
    assert_eq!(code, 0, "{out}");
    let last = out.lines().last().unwrap_or_default();
    assert_eq!(
        last,
        "planrun: assay-style.md: ok (assay (.planrun/formats/assay.toml, overrides built-in), 2 tasks)"
    );
    let (code, out) = planrun_in(f.path(), &["show", "assay-style.md"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("format  assay (.planrun/formats/assay.toml, overrides built-in)\n"),
        "{out}"
    );
}

#[test]
fn a_spec_or_config_that_cannot_be_used_stops_with_2() {
    let f = with_plan();
    f.write(".planrun/formats/broken.toml", b"id = 1\n");
    let (code, out) = planrun_in(f.path(), &["check", "assay-style.md"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("broken.toml: format `broken`:"), "{out}");
    std::fs::remove_file(f.path().join(".planrun/formats/broken.toml"))
        .unwrap_or_else(|e| panic!("{e}"));
    f.write(".planrun/config.toml", b"[plans]\nglobs = []\n");
    let (code, out) = planrun_in(f.path(), &["check", "assay-style.md"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("error[config-globs-empty]"), "{out}");
}

#[test]
fn a_plan_is_found_by_a_part_of_its_name() {
    let f = Fixture::greeter(false);
    let plan = std::fs::read(root().join(SYNTHETIC)).unwrap_or_else(|e| panic!("{e}"));
    f.write("docs/plans/2026-01-01-p1-greet.md", &plan);
    f.write("docs/plans/2026-01-02-p1-other.md", b"# Not a plan\n");
    let (code, out) = planrun_in(f.path(), &["check", "GREET"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("planrun: `GREET` is docs/plans/2026-01-01-p1-greet.md\n"),
        "{out}"
    );
    assert!(
        out.contains("planrun: docs/plans/2026-01-01-p1-greet.md: ok (assay, 2 tasks)\n"),
        "{out}"
    );
    let (code, out) = planrun_in(f.path(), &["status", "p1"]);
    assert_eq!(code, 2, "{out}");
    assert!(
        out.contains("error[plan-ambiguous]: `p1` names 2 plan files"),
        "{out}"
    );
    let (code, out) = planrun_in(f.path(), &["show", "farewell"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("error[plan-not-found]"), "{out}");
}

#[test]
fn list_shows_each_discovered_file_with_its_format_and_journal() {
    let f = Fixture::greeter(false);
    let plan = std::fs::read(root().join(SYNTHETIC)).unwrap_or_else(|e| panic!("{e}"));
    let broken = std::fs::read(root().join("tests/corpus/invalid/task-order.md"))
        .unwrap_or_else(|e| panic!("{e}"));
    f.write("docs/plans/2026-01-01-p1-greet.md", &plan);
    f.write("docs/plans/2026-01-02-p2-broken.md", &broken);
    f.write("docs/plans/README.md", b"# Plans\n");
    f.write("tools/superpowers/plans/2026-01-03-p3-tools.md", &plan);
    let (code, out) = planrun_in(f.path(), &["list"]);
    assert_eq!(code, 1, "{out}");
    insta::assert_snapshot!("list_fixture", out);
    let (code, out) = planrun_in(&f.outside(), &["list"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("error[not-a-repository]"), "{out}");
}
```

File `crates/planrun-cli/tests/snapshots/cli__list_fixture.snap` (full contents):

```text
---
source: crates/planrun-cli/tests/cli.rs
expression: out
---
docs/plans/2026-01-01-p1-greet.md               assay, 2 tasks  not run
docs/plans/2026-01-02-p2-broken.md              1 error (planrun check docs/plans/2026-01-02-p2-broken.md)
docs/plans/README.md                            not a plan
tools/superpowers/plans/2026-01-03-p3-tools.md  assay, 2 tasks  not run
planrun: 3 plans and 1 other file from the default globs
```

**Step 2 — verify**

Targeted: the binary's tests, with the list snapshot:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-cli --test cli
```

Expected (exit code 0; summary lines):

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-cli/src/list.rs crates/planrun-cli/src/main.rs crates/planrun-cli/tests/cli.rs crates/planrun-cli/tests/snapshots/cli__list_fixture.snap
  $verified = [ordered]@{
      'crates/planrun-cli/src/list.rs' = '627b2114f1a9fc8c120bd156214a85401b8cf1a0'
      'crates/planrun-cli/src/main.rs' = 'deeddff276bbbc194d77f7013eea64a307cc009c'
      'crates/planrun-cli/tests/cli.rs' = 'c6a443e7b72fd9a9a0c48116d4981bdd239dc975'
      'crates/planrun-cli/tests/snapshots/cli__list_fixture.snap' = '8c3aa96da0c149bc5dd1f3ef1936c3c34f03eb60'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P7-T5: planrun list" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P7-T6: the Plans view lists the discovered plans

**Goal:** `planrun tui` without a directory lists the discovered plans, made again at every two-second check; `planrun tui <dir>` keeps D94's listing. Plans a journal names that no glob matches are listed too, and their detail says so. The footer says `default globs` or `.planrun/config.toml`. D97 gains its TUI bullet.

**Files**

- Modify `crates/planrun-cli/src/main.rs`
- Modify `crates/planrun-tui/src/app.rs`
- Modify `crates/planrun-tui/src/app_tests.rs`
- Modify `crates/planrun-tui/src/jobs.rs`
- Modify `crates/planrun-tui/src/plans.rs`
- Modify `crates/planrun-tui/src/plans_tests.rs`
- Modify `crates/planrun-tui/src/terminal.rs`
- Modify `crates/planrun-tui/tests/plans.rs`
- Create `crates/planrun-tui/tests/snapshots/plans__without_a_directory_the_plans_view_lists_the_discovered_plans.snap`
- Modify `docs/decisions.md`

Over 300 lines, so they come as exact diffs: `crates/planrun-cli/src/main.rs`, `crates/planrun-tui/src/app.rs`, `crates/planrun-tui/src/jobs.rs`, `crates/planrun-tui/src/plans.rs`, `docs/decisions.md`.

**Step 1 — write the files**

File `crates/planrun-cli/src/main.rs` (exact diff; save the block as `$env:TEMP\p7-t6-crates-planrun-cli-src-main-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-cli/src/main.rs b/crates/planrun-cli/src/main.rs
index deeddff..955b31d 100644
--- a/crates/planrun-cli/src/main.rs
+++ b/crates/planrun-cli/src/main.rs
@@ -110,8 +110,8 @@ enum Command {
     List,
     /// The TUI over the repository of the current directory (docs/tui-design.md, D84–D94).
     Tui {
-        /// The directory of plan files; `docs/plans` under the repository's root when not
-        /// given.
+        /// A directory of plan files (its `*.md` files, D94); the discovered plans when not
+        /// given (D97).
         plans_dir: Option<PathBuf>,
     },
     /// Install the latest GitHub release over this executable when it is newer (D83).
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t6-crates-planrun-cli-src-main-rs.patch"
Remove-Item "$env:TEMP\p7-t6-crates-planrun-cli-src-main-rs.patch"
```

File `crates/planrun-tui/src/app.rs` (exact diff; save the block as `$env:TEMP\p7-t6-crates-planrun-tui-src-app-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-tui/src/app.rs b/crates/planrun-tui/src/app.rs
index 5aee28a..f72b1e9 100644
--- a/crates/planrun-tui/src/app.rs
+++ b/crates/planrun-tui/src/app.rs
@@ -14,7 +14,7 @@ use crate::draw::fill;
 use crate::glyph::{Glyph, Glyphs};
 use crate::history::{self, History};
 use crate::inspect::{Assumed, Did, Inspect};
-use crate::jobs::{Bg, Context, Done, Job, UpdateCheck, UpdateState};
+use crate::jobs::{Bg, Context, Done, Job, Listing, UpdateCheck, UpdateState};
 use crate::keys::Key;
 use crate::palette::{Cmd, Command, Palette, Pick};
 use crate::plans::{self, Labels, Plans};
@@ -188,6 +188,10 @@ impl App {
     pub fn start(&mut self) -> Vec<Effect> {
         self.next_poll = self.now() + POLL_MS;
         let mut effects = vec![Effect::Job(Job::Scan), Effect::Job(Job::Live)];
+        if self.ctx.listing == Listing::Discovered {
+            // The plans only a journal knows join the list (D97).
+            effects.push(Effect::Job(Job::Journals));
+        }
         if let Some(check) = self.update_check.take() {
             effects.push(Effect::Job(Job::Update(check)));
         }
@@ -254,6 +258,10 @@ impl App {
                 Vec::new()
             }
             Done::Journals(runs) => {
+                if let (Listing::Discovered, Ok(runs)) = (&self.ctx.listing, &runs) {
+                    let recorded = runs.iter().map(|r| r.plan.as_str());
+                    self.plans.journalled = plans::journalled(&self.ctx.root, recorded);
+                }
                 let n = runs.as_ref().map_or(0, Vec::len);
                 self.history.runs = Some(runs);
                 self.history.selected = self.history.selected.min(n.saturating_sub(1));
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t6-crates-planrun-tui-src-app-rs.patch"
Remove-Item "$env:TEMP\p7-t6-crates-planrun-tui-src-app-rs.patch"
```

File `crates/planrun-tui/src/app_tests.rs` (full contents):

```rust
//! The app's own tests: what a terminal below the minimum shows.

use std::sync::mpsc::channel;

use planrun_exec::discover::Discovery;
use planrun_exec::ui::{Choice, Question, QuestionKind};
use planrun_parse::{Registry, parse_plan};

use super::*;
use crate::grid::grid;
use crate::jobs::Listing;

const PLAN: &str = include_str!("../../../tests/corpus/synthetic/assay-style.md");

fn app() -> App {
    let ctx = Context {
        root: PathBuf::from("."),
        git_dir: PathBuf::from(".git"),
        repo: "greeter".to_owned(),
        target: "greeter".to_owned(),
        listing: Listing::Dir(PathBuf::from("plans")),
        plans_label: "plans".to_owned(),
        version: "0.1.0".to_owned(),
        env: Vec::new(),
        discovery: Discovery::default(),
    };
    App::new(ctx, Glyphs::unicode(), Arc::new(|| 0), None)
}

#[test]
fn below_the_minimum_an_open_dock_is_drawn_alone_and_answers() {
    let registry = Registry::with_builtins().unwrap_or_else(|e| panic!("{e}"));
    let parsed = parse_plan(&registry, "assay-style.md", PLAN.as_bytes(), None)
        .unwrap_or_else(|e| panic!("{e:?}"));
    let mut app = app();
    let mut run = Run::new(
        PathBuf::from("assay-style.md"),
        Arc::new(parsed),
        Assumed::default(),
        false,
        0,
    );
    let (tx, rx) = channel();
    run.ask(
        Question {
            kind: QuestionKind::Manual,
            at: None,
            text: vec!["check it by hand".to_owned()],
            choices: vec![Choice::Continue, Choice::Quit],
            typed: None,
        },
        tx,
    );
    app.run = Some(run);
    app.view = View::Plan;
    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
    app.draw(&mut buf);
    let text = grid(&buf);
    assert!(text.contains("MANUAL"), "{text}");
    assert!(!text.contains("terminal too small"), "{text}");
    app.key(Key::Enter);
    assert_eq!(
        rx.recv().ok(),
        Some(planrun_exec::ui::Reply::Choice(Choice::Continue))
    );
    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
    app.draw(&mut buf);
    assert!(grid(&buf).contains("terminal too small"));
}
```

File `crates/planrun-tui/src/jobs.rs` (exact diff; save the block as `$env:TEMP\p7-t6-crates-planrun-tui-src-jobs-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-tui/src/jobs.rs b/crates/planrun-tui/src/jobs.rs
index a92f41c..e991667 100644
--- a/crates/planrun-tui/src/jobs.rs
+++ b/crates/planrun-tui/src/jobs.rs
@@ -27,9 +27,9 @@ pub struct Context {
     pub repo: String,
     /// The target as the footer shows it.
     pub target: String,
-    /// The plans directory (absolute).
-    pub plans_dir: PathBuf,
-    /// The plans directory as the footer shows it.
+    /// Where the Plans view's files come from.
+    pub listing: Listing,
+    /// Where they come from, as the footer shows it.
     pub plans_label: String,
     /// planrun's version.
     pub version: String,
@@ -66,6 +66,15 @@ impl Context {
     }
 }
 
+/// Where the Plans view's files come from.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Listing {
+    /// `planrun tui <dir>`: the directory's `*.md` files, not recursive (D94); absolute.
+    Dir(PathBuf),
+    /// `planrun tui`: the plans discovery finds (D96, D97), listed again at every scan.
+    Discovered,
+}
+
 /// A file's fingerprint for change detection: its length and modification time.
 #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
 pub struct Stamp {
@@ -196,7 +205,7 @@ impl PlanFile {
 
 /// One piece of background work.
 pub enum Job {
-    /// Lists the plans directory's `*.md` files.
+    /// Lists the plan files: a directory's `*.md` files, or the discovered plans.
     Scan,
     /// Parses and checks a plan, and reads its journal.
     Check {
@@ -241,7 +250,7 @@ pub enum Job {
 /// What a job found.
 #[derive(Debug, Clone)]
 pub enum Done {
-    /// The plans directory's files with their stamps, sorted by name.
+    /// The plan files with their stamps, sorted by path.
     Scanned(Result<Vec<(PathBuf, Stamp)>, String>),
     /// A plan's check.
     Checked {
@@ -346,7 +355,7 @@ impl Job {
     /// Does the work.
     pub fn run(self, ctx: &Context) -> Done {
         match self {
-            Job::Scan => Done::Scanned(scan(&ctx.plans_dir)),
+            Job::Scan => Done::Scanned(scan(ctx)),
             Job::Check { path, stamp } => {
                 let file = check(ctx, &path);
                 Done::Checked { path, stamp, file }
@@ -453,12 +462,26 @@ fn changes(ctx: &Context, paths: &[String]) -> Vec<crate::diff::FileDiff> {
     }
 }
 
-fn scan(dir: &Path) -> Result<Vec<(PathBuf, Stamp)>, String> {
-    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
-    let mut files: Vec<(PathBuf, Stamp)> = entries
-        .flatten()
-        .map(|e| e.path())
-        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "md"))
+/// The plan files with their stamps: one `git ls-files` for the discovered plans (D97), or one
+/// directory listing.
+fn scan(ctx: &Context) -> Result<Vec<(PathBuf, Stamp)>, String> {
+    let paths: Vec<PathBuf> = match &ctx.listing {
+        Listing::Dir(dir) => std::fs::read_dir(dir)
+            .map_err(|e| format!("{}: {e}", dir.display()))?
+            .flatten()
+            .map(|e| e.path())
+            .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "md"))
+            .collect(),
+        Listing::Discovered => ctx
+            .discovery
+            .plans(&ctx.env)
+            .map_err(|e| format!("error[{}]: {e}", e.code()))?
+            .iter()
+            .map(|p| ctx.root.join(p.as_str()))
+            .collect(),
+    };
+    let mut files: Vec<(PathBuf, Stamp)> = paths
+        .into_iter()
         .filter_map(|p| Stamp::of(&p).map(|s| (p, s)))
         .collect();
     files.sort_by(|a, b| a.0.cmp(&b.0));
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t6-crates-planrun-tui-src-jobs-rs.patch"
Remove-Item "$env:TEMP\p7-t6-crates-planrun-tui-src-jobs-rs.patch"
```

File `crates/planrun-tui/src/plans.rs` (exact diff; save the block as `$env:TEMP\p7-t6-crates-planrun-tui-src-plans-rs.patch`, outside the repository, then apply it):

```diff
diff --git a/crates/planrun-tui/src/plans.rs b/crates/planrun-tui/src/plans.rs
index fa28a0f..db5e8a8 100644
--- a/crates/planrun-tui/src/plans.rs
+++ b/crates/planrun-tui/src/plans.rs
@@ -3,7 +3,7 @@
 //! coverage strip per task, the check's diagnostics, the journal's status, and a word on
 //! preflight, which is never a background check.
 
-use std::path::PathBuf;
+use std::path::{Path, PathBuf};
 use std::sync::Arc;
 
 use ratatui::buffer::Buffer;
@@ -36,6 +36,8 @@ pub struct Entry {
     pub check: Bg<PlanFile>,
     /// The stamp of the check in `check`.
     pub checked: Option<Stamp>,
+    /// Listed because a journal names it, not because a glob matches it (D97).
+    pub outside: bool,
 }
 
 /// What the Plans view holds.
@@ -51,6 +53,9 @@ pub struct Plans {
     pub editing: bool,
     /// Why the directory could not be listed.
     pub error: Option<String>,
+    /// The plans the journals name that exist ([`journalled`]); listed after the scanned files
+    /// when no scan lists them.
+    pub journalled: Vec<PathBuf>,
 }
 
 /// What a key did in the Plans view.
@@ -95,17 +100,31 @@ impl Plans {
             }
         };
         self.error = None;
+        let mut files = files;
+        let mut outside = Vec::new();
+        for path in &self.journalled {
+            if files.iter().any(|(p, _)| p == path) {
+                continue;
+            }
+            if let Some(stamp) = Stamp::of(path) {
+                files.push((path.clone(), stamp));
+                outside.push(path.clone());
+            }
+        }
         let mut jobs = Vec::new();
         let mut entries = Vec::new();
         for (path, stamp) in files {
+            let is_outside = outside.contains(&path);
             let old = self.entries.iter().find(|e| e.path == path).cloned();
             let entry = match old {
                 Some(mut e) if e.checked == Some(stamp) || e.stamp == stamp => {
                     e.stamp = stamp;
+                    e.outside = is_outside;
                     e
                 }
                 Some(mut e) => {
                     e.stamp = stamp;
+                    e.outside = is_outside;
                     e.check = e.check.stale();
                     jobs.push(Job::Check {
                         path: path.clone(),
@@ -126,6 +145,7 @@ impl Plans {
                         stamp,
                         check: Bg::Running,
                         checked: None,
+                        outside: is_outside,
                     }
                 }
             };
@@ -429,6 +449,11 @@ impl Plans {
             );
             Pen::new(buf, area.x, y, right, g).text(&line, fg(Token::Fg3));
         }
+        if entry.outside {
+            y += 1;
+            Pen::new(buf, area.x, y, right, g)
+                .text("no glob matches it; its journal names it", fg(Token::Fg3));
+        }
         y += 2;
         if let Some(p) = parsed {
             y = draw_outline(buf, area, y, p, g);
@@ -513,6 +538,25 @@ impl Plans {
     }
 }
 
+/// The plans that journals name (`plan` as recorded: repository-relative, or absolute when the
+/// plan is outside the repository, D76) and that are files now, each once.
+pub fn journalled<'a>(root: &Path, recorded: impl IntoIterator<Item = &'a str>) -> Vec<PathBuf> {
+    let mut plans: Vec<PathBuf> = recorded
+        .into_iter()
+        .map(|p| {
+            if Path::new(p).is_absolute() {
+                PathBuf::from(p)
+            } else {
+                root.join(p)
+            }
+        })
+        .filter(|p| p.is_file())
+        .collect();
+    plans.sort();
+    plans.dedup();
+    plans
+}
+
 /// The labels the views show for paths.
 #[derive(Debug, Clone)]
 pub struct Labels {
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t6-crates-planrun-tui-src-plans-rs.patch"
Remove-Item "$env:TEMP\p7-t6-crates-planrun-tui-src-plans-rs.patch"
```

File `crates/planrun-tui/src/plans_tests.rs` (full contents):

```rust
//! The Plans view's tests: the coverage strip, and dates and times in UTC.

use super::*;

fn cov(actions: usize, manifest: usize, notes: usize, manual: usize) -> TaskCoverage {
    TaskCoverage {
        task: planrun_model::TaskId {
            raw: "P1-T1".into(),
            ordinal: 1,
        },
        actions,
        absorbed: Vec::new(),
        manifest,
        notes,
        manual,
        dropped: 0,
    }
}

#[test]
fn the_strip_has_sixteen_cells_and_a_manual_cell_when_there_is_one() {
    for c in [
        cov(10, 3, 2, 0),
        cov(30, 1, 1, 1),
        cov(1, 0, 0, 0),
        cov(5, 5, 5, 5),
    ] {
        let cells: usize = strip(&c).iter().map(|s| s.0).sum();
        assert_eq!(cells, 16);
    }
    let s = strip(&cov(30, 1, 1, 1));
    assert_eq!(s.last().map(|p| (p.0 >= 1, p.2)), Some((true, Token::Warn)));
    assert_eq!(
        strip(&cov(1, 0, 0, 0)),
        vec![(16, Glyph::CovAction, Token::Fg2)]
    );
}

#[test]
fn dates_and_times_are_utc() {
    assert_eq!(civil(0), (1970, 1, 1));
    // 2026-11-27 12:34:56 UTC
    let ms = 1_795_782_896_000;
    assert_eq!(civil(ms), (2026, 11, 27));
    assert_eq!(date_utc(ms), "27 Nov");
    assert_eq!(clock_utc(ms), "12:34");
}

#[test]
fn journalled_plans_are_the_recorded_files_that_exist() {
    let dir = std::env::temp_dir().join(format!("planrun-journalled-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("notes")).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(dir.join("notes/p1.md"), "# P1\n").unwrap_or_else(|e| panic!("{e}"));
    let absolute = dir.join("notes/p1.md").display().to_string();
    let found = journalled(&dir, ["notes/p1.md", "notes/gone.md", absolute.as_str()]);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(found, [dir.join("notes/p1.md")]);
}
```

File `crates/planrun-tui/src/terminal.rs` (full contents):

```rust
//! `planrun tui` on a real terminal (decision D94): stdin and stdout must be terminals; the
//! repository is the one of the current directory (D88); without a plans directory the Plans
//! view lists the discovered plans (D97). One loop polls crossterm's events and drains the app's channel
//! between polls; there is no input thread, which would read the keys meant for the `[o]` shell.
//! Interrupts are installed once, before the terminal is set up; a panic restores the terminal
//! only when it is the render thread's.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::thread::ThreadId;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{self, EnterAlternateScreen};

use planrun_exec::discover::Discovery;
use planrun_exec::shell::{InterruptsIgnored, install_interrupt_handler};
use planrun_exec::workspace::Workspace;

use crate::app::{App, system_clock};
use crate::driver::{Core, Hooks};
use crate::glyph::Glyphs;
use crate::jobs::{Context, Listing, UpdateCheck};
use crate::keys;

/// How long one poll for a key waits before the app's channel is drained again.
const POLL: Duration = Duration::from_millis(50);

/// What planrun-cli hands the TUI.
pub struct Host {
    /// The current directory: its repository is the target.
    pub dir: PathBuf,
    /// The plans directory as given (relative to `dir`), or `None` for the discovered plans
    /// (D97).
    pub plans: Option<PathBuf>,
    /// planrun's version.
    pub version: String,
    /// The update check.
    pub update: Option<UpdateCheck>,
}

/// Builds the context for the repository of `dir`.
/// Discovery runs from `dir` with `user` as the user's config directory (D96); a spec that cannot
/// be used stops the TUI before it starts.
pub fn context(
    dir: &Path,
    plans: Option<&Path>,
    user: Option<&Path>,
    version: &str,
    env: Vec<(std::ffi::OsString, std::ffi::OsString)>,
) -> Result<Context, String> {
    let ws = Workspace::open(dir, env.clone()).map_err(|e| e.to_string())?;
    let discovery = Discovery::open(dir, user).map_err(|e| format!("error[{}]: {e}", e.code()))?;
    let git_dir = ws.git().git_dir().map_err(|e| e.to_string())?;
    let root = ws.root().to_path_buf();
    let (listing, plans_label) = match plans {
        Some(p) => {
            let dir = if p.is_absolute() {
                p.to_path_buf()
            } else {
                dir.join(p)
            };
            let label = dir_label(&root, &dir);
            (Listing::Dir(dir), label)
        }
        None => {
            let label = match discovery.config().file {
                Some(_) => planrun_exec::discover::CONFIG_FILE.to_owned(),
                None => "default globs".to_owned(),
            };
            (Listing::Discovered, label)
        }
    };
    let repo = root.file_name().map_or_else(
        || root.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    Ok(Context {
        target: root.display().to_string(),
        repo,
        root,
        git_dir,
        listing,
        plans_label,
        version: version.to_owned(),
        env,
        discovery,
    })
    .and_then(|ctx| ctx.registry().map(|_| ctx))
}

/// The plans directory as the footer shows it: relative to the root when it is under it.
fn dir_label(root: &Path, dir: &Path) -> String {
    let canonical = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    match canonical.strip_prefix(root) {
        Ok(rel) if rel.as_os_str().is_empty() => ".".to_owned(),
        Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
        Err(_) => dir.display().to_string(),
    }
}

/// Runs the TUI until it quits. `Err` is the message for a usage problem: not a terminal, not a
/// repository, or a terminal that could not be set up.
pub fn run(host: Host) -> Result<(), String> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(
            "planrun tui needs a terminal on stdin and stdout; `planrun run` is the line version"
                .to_owned(),
        );
    }
    let user = planrun_exec::discover::user_dir();
    let ctx = context(
        &host.dir,
        host.plans.as_deref(),
        user.as_deref(),
        &host.version,
        Vec::new(),
    )?;
    let app = App::new(ctx, Glyphs::from_env(), system_clock(), host.update);
    install_interrupt_handler();
    let report = std::panic::take_hook();
    let terminal = ratatui::init();
    install_panic_hook(report);
    let result = drive(Core::new(app, terminal, Box::new(Crossterm)));
    ratatui::restore();
    result
}

/// What a panic does to the terminal (D94).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnPanic {
    /// The render thread's: restore the terminal, then report as Rust does.
    Restore,
    /// Another thread's (the runner's, a job's): leave the terminal and print nothing; the
    /// thread catches the panic and the app shows it as that work's failure.
    Leave,
}

/// What a panic on `current` does, when `render` owns the terminal.
pub fn on_panic(render: ThreadId, current: ThreadId) -> OnPanic {
    if render == current {
        OnPanic::Restore
    } else {
        OnPanic::Leave
    }
}

/// The hook a panic report goes through.
type Report = Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Sync + Send>;

/// Replaces ratatui's hook, which would restore the terminal on every thread's panic, caught or
/// not: `report` (the hook before `ratatui::init`) runs only for the render thread's.
fn install_panic_hook(report: Report) {
    let render = std::thread::current().id();
    std::panic::set_hook(Box::new(move |info| {
        if on_panic(render, std::thread::current().id()) == OnPanic::Restore {
            ratatui::restore();
            report(info);
        }
    }));
}

/// The terminal's hooks.
struct Crossterm;

impl Hooks for Crossterm {
    fn shell(&mut self, root: &Path) {
        ratatui::restore();
        let program = if cfg!(windows) { "pwsh" } else { "bash" };
        println!(
            "{program} at {}; this is not the plan's session (its variables are not here). Exit to return.",
            root.display()
        );
        {
            let _quiet = InterruptsIgnored::new();
            if let Err(e) = std::process::Command::new(program)
                .current_dir(root)
                .status()
            {
                println!("{program} did not start: {e}");
            }
        }
        let _ = terminal::enable_raw_mode();
        let _ = execute!(std::io::stdout(), EnterAlternateScreen);
        while event::poll(Duration::ZERO).unwrap_or(false) {
            let _ = event::read();
        }
    }

    fn end_now(&mut self) {
        ratatui::restore();
        planrun_exec::shell::end_now(130);
    }
}

fn drive<B: ratatui::backend::Backend>(mut core: Core<B>) -> Result<(), String> {
    let term = |e: B::Error| format!("the terminal: {e}");
    let io = |e: std::io::Error| format!("the terminal: {e}");
    core.start();
    core.draw().map_err(term)?;
    while !core.done() {
        core.pump();
        if event::poll(POLL).map_err(io)? {
            match event::read().map_err(io)? {
                Event::Key(ev) => {
                    if let Some(key) = keys::key(&ev) {
                        core.key(key);
                    }
                }
                Event::Resize(..) => core.redraw().map_err(term)?,
                _ => {}
            }
        }
        core.tick();
        core.pump();
        core.draw().map_err(term)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_render_thread_s_panic_restores_the_terminal() {
        let render = std::thread::current().id();
        let other = std::thread::spawn(|| std::thread::current().id())
            .join()
            .unwrap_or_else(|_| panic!("the thread panicked"));
        assert_eq!(on_panic(render, render), OnPanic::Restore);
        assert_eq!(on_panic(render, other), OnPanic::Leave);
    }
}
```

File `crates/planrun-tui/tests/plans.rs` (full contents):

```rust
//! The Plans view (docs/tui-design.md §6.A) on the greeter fixture with the synthetic plans.

mod support;

use planrun_tui::keys::Key;
use support::{PLANS, screen, setup};

#[test]
fn the_plans_view_lists_every_file_with_its_check_and_run() {
    let setup = setup(&PLANS);
    let mut core = setup.core();
    core.key(Key::Down);
    insta::assert_snapshot!(screen(&mut core));
}

#[test]
fn the_filter_narrows_the_table_and_esc_clears_it() {
    let setup = setup(&PLANS);
    let mut core = setup.core();
    for key in [
        Key::Char('/'),
        Key::Char('d'),
        Key::Char('e'),
        Key::Char('v'),
        Key::Enter,
    ] {
        core.key(key);
    }
    let names: Vec<String> = core
        .app()
        .plans()
        .shown()
        .iter()
        .map(|e| e.name.clone())
        .collect();
    assert_eq!(names, ["devready-style.md"]);
    core.key(Key::Esc);
    assert_eq!(core.app().plans().shown().len(), PLANS.len());
}

#[test]
fn below_100_by_30_the_tui_says_it_is_too_small() {
    let setup = setup(&PLANS[..1]);
    let mut core = setup.core();
    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut small = planrun_tui::driver::Core::new(
        setup.app(),
        ratatui::Terminal::new(backend).unwrap_or_else(|e| panic!("{e}")),
        Box::new(support::Counted::default()),
    );
    let text = screen(&mut small);
    assert!(text.contains("terminal too small"), "{text}");
    assert!(text.contains("needs 100 × 30"), "{text}");
    let _ = screen(&mut core);
}

#[test]
fn a_job_that_panics_comes_back_in_its_failed_state() {
    let setup = setup(&PLANS[..1]);
    let app = setup.app_with(Some(Box::new(|| panic!("the update check broke"))));
    let terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 36))
        .unwrap_or_else(|e| panic!("{e}"));
    let mut core =
        planrun_tui::driver::Core::new(app, terminal, Box::new(support::Counted::default()));
    core.start();
    let failed = core.wait(std::time::Duration::from_secs(30), |app| {
        app.update() == &planrun_tui::jobs::UpdateState::Failed
    });
    assert!(failed, "the panicked job never reported");
    assert!(screen(&mut core).contains("update check failed"));
}

#[test]
fn q_quits_from_plans() {
    let setup = setup(&PLANS[..1]);
    let mut core = setup.core();
    core.key(Key::Char('q'));
    assert!(core.done());
}

#[test]
fn without_a_directory_the_plans_view_lists_the_discovered_plans() {
    use std::time::{Duration, UNIX_EPOCH};

    let fixture = support::fixture::Fixture::greeter(false);
    let corpus =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/synthetic");
    let files = [
        ("docs/plans/2026-01-01-p1-greet.md", "assay-style.md"),
        ("docs/plans/README.md", "README.md"),
        (
            "tools/superpowers/plans/2026-01-02-tools.md",
            "superpowers-style.md",
        ),
        ("notes/2026-01-03-elsewhere.md", "devready-style.md"),
    ];
    for (to, from) in files {
        let bytes = std::fs::read(corpus.join(from)).unwrap_or_else(|e| panic!("{e}"));
        fixture.write(to, &bytes);
        std::fs::File::options()
            .write(true)
            .open(fixture.path().join(to))
            .and_then(|f| f.set_modified(UNIX_EPOCH + Duration::from_millis(support::NOW)))
            .unwrap_or_else(|e| panic!("{e}"));
    }
    let mut ctx =
        planrun_tui::terminal::context(fixture.path(), None, None, "0.1.0", fixture.env())
            .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(ctx.plans_label, "default globs");
    ctx.repo = "greeter".to_owned();
    ctx.target = r"C:\src\greeter".to_owned();
    let app = planrun_tui::app::App::new(
        ctx,
        planrun_tui::glyph::Glyphs::unicode(),
        std::sync::Arc::new(|| support::NOW),
        None,
    );
    let terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 36))
        .unwrap_or_else(|e| panic!("{e}"));
    let mut core =
        planrun_tui::driver::Core::new(app, terminal, Box::new(support::Counted::default()));
    core.start();
    let checked = core.wait(Duration::from_secs(60), |app| {
        let entries = &app.plans().entries;
        entries.len() == 3
            && entries
                .iter()
                .all(|e| matches!(e.check, planrun_tui::jobs::Bg::Current(_)))
    });
    assert!(checked, "the discovered plans were not checked in time");
    let names: Vec<String> = core
        .app()
        .plans()
        .entries
        .iter()
        .map(|e| e.name.clone())
        .collect();
    assert_eq!(
        names,
        ["2026-01-01-p1-greet.md", "README.md", "2026-01-02-tools.md"]
    );
    insta::assert_snapshot!(screen(&mut core));
}
```

File `crates/planrun-tui/tests/snapshots/plans__without_a_directory_the_plans_view_lists_the_discovered_plans.snap` (full contents):

```text
---
source: crates/planrun-tui/tests/plans.rs
expression: screen(&mut core)
---
 0  PLANRUN  greeter › plans                                                                                        ○ idle
   0-0 fg/bg1, 1-7 fg/bg1 b, 8-9 fg/bg1, 10-16 fg2/bg1, 17-19 fg3/bg1, 20-24 fg/bg1 b, 25-112 fg/bg1, 113-114 fg3/bg1, 115-118 fg2/bg1, 119-119 fg/bg1
 1 ▎ PLANS  3 files in default globs                              / filter │  PLAN  2026-01-01-p1-greet.md
   0-0 act/bg1, 1-1 fg/bg1, 2-6 act/bg1 b, 7-8 fg/bg1, 9-32 fg3/bg1, 33-62 fg/bg1, 63-70 fg3/bg1, 71-71 fg/bg1, 72-72 rule/bg0, 73-74 fg/bg1, 75-78 fg2/bg1 b, 79-80 fg/bg1, 81-102 fg3/bg1, 103-119 fg/bg1
 2                                                                         │
   72-72 rule/bg0
 3   PLAN                     FORMAT       TASKS CHECK       RUN           │ Greeter Plan — Part 1: the greet module
   2-5 fg3/bg0 b, 27-32 fg3/bg0 b, 40-44 fg3/bg0 b, 46-50 fg3/bg0 b, 58-60 fg3/bg0 b, 72-72 rule/bg0, 74-112 fg/bg0 b
 4 ▌ 2026-01-01-p1-greet.md   assay           2  ✓ valid     ○ not run     │ assay · 2 tasks · 8 steps · 12 actions
   0-0 act/bg2, 1-1 fg/bg2, 2-23 fg/bg2 b, 24-26 fg/bg2, 27-31 fg2/bg2, 32-39 fg/bg2, 40-43 fg2/bg2, 44-45 fg/bg2, 46-46 ok/bg2, 47-47 fg/bg2, 48-52 fg2/bg2, 53-57 fg/bg2, 58-58 fg3/bg2, 59-59 fg/bg2, 60-66 fg3/bg2, 67-71 fg/bg2, 72-72 rule/bg0, 74-111 fg2/bg0
 5   README.md                ─               ─  not a plan  ─             │ docs/plans/2026-01-01-p1-greet.md · 214 line…
   27-27 fg2/bg0, 40-43 fg2/bg0, 46-55 fg2/bg0, 58-58 fg3/bg0, 72-72 rule/bg0, 74-118 fg3/bg0
 6   2026-01-02-tools.md      superpowers     2  ✓ valid     ○ not run     │
   27-37 fg2/bg0, 40-43 fg2/bg0, 46-46 ok/bg0, 48-52 fg2/bg0, 58-58 fg3/bg0, 60-66 fg3/bg0, 72-72 rule/bg0
 7                                                                         │ OUTLINE ──────────────────────────── coverage
   72-72 rule/bg0, 74-80 fg3/bg0 b, 82-109 rule/bg0, 111-118 fg3/bg0
 8                                                                         │ P1-T1  the greet module      ██████████▪▒▒░░░
   72-72 rule/bg0, 74-78 meta/bg0, 103-112 fg2/bg0, 113-113 meta/bg0, 114-118 fg3/bg0
 9                                                                         │ P1-T2  re-export greeting, … █████████▪▒▒▒░░░
   72-72 rule/bg0, 74-78 meta/bg0, 103-111 fg2/bg0, 112-112 meta/bg0, 113-118 fg3/bg0
10                                                                         │ █ action ▪ absorbed ▒ manifest ░ note █ manual
   72-72 rule/bg0, 74-74 fg2/bg0, 75-82 fg3/bg0, 83-83 meta/bg0, 84-111 fg3/bg0, 112-112 warn/bg0, 113-119 fg3/bg0
11                                                                         │
   72-72 rule/bg0
12                                                                         │ CHECK ─────────────────────── ✓ valid · 12:34
   72-72 rule/bg0, 74-78 fg3/bg0 b, 80-102 rule/bg0, 104-105 ok/bg0, 106-110 fg2/bg0, 111-118 fg3/bg0
13                                                                         │
   72-72 rule/bg0
14                                                                         │ STATUS ──────────────────────────── ○ not run
   72-72 rule/bg0, 74-79 fg3/bg0 b, 81-108 rule/bg0, 110-111 fg3/bg0, 112-118 fg2/bg0
15                                                                         │ no journal in .git/planrun for this plan
   72-72 rule/bg0, 74-113 fg2/bg0
16                                                                         │
   72-72 rule/bg0
17                                                                         │ PREFLIGHT ───────────────────────────────────
   72-72 rule/bg0, 74-82 fg3/bg0 b, 84-118 rule/bg0
18                                                                         │ never a background check: it runs right
   72-72 rule/bg0, 74-112 fg2/bg0
19                                                                         │ before each task, and in every dry run
   72-72 rule/bg0, 74-111 fg2/bg0
20                                                                         │
   72-72 rule/bg0
21                                                                         │  enter  inspect with a dry run
   72-72 rule/bg0, 74-80 fg/bg2 b, 81-103 fg2/bg0
22                                                                         │
   72-72 rule/bg0
23                                                                         │
   72-72 rule/bg0
24                                                                         │
   72-72 rule/bg0
25                                                                         │
   72-72 rule/bg0
26                                                                         │
   72-72 rule/bg0
27                                                                         │
   72-72 rule/bg0
28                                                                         │
   72-72 rule/bg0
29                                                                         │
   72-72 rule/bg0
30  ────────────────────────────────────────────────────────────────────── │
   1-70 rule/bg0, 72-72 rule/bg0
31   check ✓ valid  ! warnings  ✕ errors  ◌ checking  grey ✓ stale         │
   2-7 fg3/bg0, 8-8 ok/bg0, 9-16 fg3/bg0, 17-17 warn/bg0, 18-28 fg3/bg0, 29-29 err/bg0, 30-38 fg3/bg0, 39-39 fg2/bg0, 40-62 fg3/bg0, 72-72 rule/bg0
32   background checks on start and on file change · c re-checks           │
   2-49 fg3/bg0, 50-50 fg2/bg0 b, 51-60 fg3/bg0, 72-72 rule/bg0
33                                                                         │
   72-72 rule/bg0
34   enter  inspect    R  run…    c  re-check    /  filter    ctrl+k  commands    ?  help
   0-0 fg/bg1, 1-7 fg/bg2 b, 8-15 fg2/bg1, 16-18 fg/bg1, 19-21 fg/bg2 b, 22-26 fg2/bg1, 27-29 fg/bg1, 30-32 fg/bg2 b, 33-41 fg2/bg1, 42-44 fg/bg1, 45-47 fg/bg2 b, 48-54 fg2/bg1, 55-57 fg/bg1, 58-65 fg/bg2 b, 66-74 fg2/bg1, 75-77 fg/bg1, 78-80 fg/bg2 b, 81-85 fg2/bg1, 86-119 fg/bg1
35   F1 Help   F2 Plans   F3 Plan   F4 History     target C:\src\greeter  ·  plans default globs             planrun 0.1.0
   0-0 fg/bg1, 1-9 fg3/bg1, 10-10 fg/bg1, 11-20 act/bg2 b, 21-21 fg/bg1, 22-30 fg3/bg1, 31-31 fg/bg1, 32-43 fg3/bg1, 44-47 fg/bg1, 48-54 fg3/bg1, 55-68 fg2/bg1, 69-79 fg3/bg1, 80-92 fg2/bg1, 93-105 fg/bg1, 106-118 fg3/bg1, 119-119 fg/bg1
```

File `docs/decisions.md` (exact diff; save the block as `$env:TEMP\p7-t6-docs-decisions-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/decisions.md b/docs/decisions.md
index e15a6fe..4f615ac 100644
--- a/docs/decisions.md
+++ b/docs/decisions.md
@@ -1497,3 +1497,9 @@ Amends D94. Decided in the P7 brainstorm (docs/superpowers/specs/2026-11-28-disc
   `plan-ambiguous`, which lists them. Both exit with code 2. A file no format detects counts as
   a candidate too: it can make a name ambiguous, never make it find the wrong plan.
 - **Messages name the plan** by its repository-relative path when a name found it.
+- **`planrun tui` without a directory lists the discovered plans** (`planrun tui <dir>` keeps
+  D94's listing): the footer says `default globs` or `.planrun/config.toml`. A plan that a
+  journal in `<git-dir>/planrun/` names and that still exists is listed too when no glob matches
+  it, and its detail says so. The list is made again at every two-second check with one
+  `git ls-files`, so a new untracked plan appears without `c`: a process every two seconds is
+  cheap next to a missed plan, and simpler than watching directories (D1).
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t6-docs-decisions-md.patch"
Remove-Item "$env:TEMP\p7-t6-docs-decisions-md.patch"
```

**Step 2 — verify**

Targeted: the Plans view's unit tests:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-tui --lib plans
```

Expected (exit code 0; summary lines):

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in …s
```

Targeted: the Plans view on a fixture, discovered:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-tui --test plans
```

Expected (exit code 0; summary lines):

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in …s
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- crates/planrun-cli/src/main.rs crates/planrun-tui/src/app.rs crates/planrun-tui/src/app_tests.rs crates/planrun-tui/src/jobs.rs crates/planrun-tui/src/plans.rs crates/planrun-tui/src/plans_tests.rs crates/planrun-tui/src/terminal.rs crates/planrun-tui/tests/plans.rs crates/planrun-tui/tests/snapshots/plans__without_a_directory_the_plans_view_lists_the_discovered_plans.snap docs/decisions.md
  $verified = [ordered]@{
      'crates/planrun-cli/src/main.rs' = '955b31d90768a077c4fd430a1dd0a99a21ec3db0'
      'crates/planrun-tui/src/app.rs' = 'f72b1e9974191b60af2439f1759a433adf0fc3ff'
      'crates/planrun-tui/src/app_tests.rs' = '1a474d13b8e2c460898c7b74c9f2fd29ce973861'
      'crates/planrun-tui/src/jobs.rs' = 'e991667a9321a5b754de15e73156d906bc7f48a7'
      'crates/planrun-tui/src/plans.rs' = 'db5e8a8f9988aea41c1891397cddd14d31b621a2'
      'crates/planrun-tui/src/plans_tests.rs' = '6d3d3ceeb1c972ae4b7331540758c4bd15ccd441'
      'crates/planrun-tui/src/terminal.rs' = '42edf918b5a012603e2414ac5fc6822f7c3b1f00'
      'crates/planrun-tui/tests/plans.rs' = 'da7a56c49c22cb1eb450b5f1511b4a3b4ed5f0a8'
      'crates/planrun-tui/tests/snapshots/plans__without_a_directory_the_plans_view_lists_the_discovered_plans.snap' = 'd169c5ae517691cd4ec56aa742a1dc1f4465961b'
      'docs/decisions.md' = '4f615acdd442c36b4c29745990b77bcab90acd87'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P7-T6: the Plans view lists the discovered plans" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

### P7-T7: discovery in architecture, extending, the READMEs, CLAUDE.md and the roadmap

**Goal:** The docs for what P7 built: architecture.md §5 (format layers) and §7 (names, `list`, the TUI's default), extending.md §2 (the layers, `PLANRUN_CONFIG_DIR`, `.planrun/config.toml`), tui-design.md's D94 line, the two READMEs, CLAUDE.md's layout and the roadmap's P7.

**Files**

- Modify `CLAUDE.md`
- Modify `README.md`
- Modify `crates/planrun-tui/README.md`
- Modify `docs/architecture.md`
- Modify `docs/extending.md`
- Modify `docs/roadmap.md`
- Modify `docs/tui-design.md`

Over 300 lines, so they come as exact diffs: `docs/architecture.md`, `docs/tui-design.md`.

**Step 1 — write the files**

File `CLAUDE.md` (full contents):

````markdown
# planrun

A Rust CLI that executes structured Markdown implementation plans against a git repository, task by
task, step by step: exactly what the plan says, nothing more. It is not an agent. Read
`docs/architecture.md` before changing code; `docs/decisions.md` holds the rules below with their
reasons (cited as `D<n>`).

## Priorities (D1, D25)

Correctness > maintainability/extensibility > execution speed > low memory > predictability.
When two of these conflict, the earlier one wins. Speed means no wasted process starts or repeated
reads; don't optimize I/O speculatively (cargo/npm dominate wall time).

## Layout

```
crates/planrun-model   the IR: Plan, Task, Step, Action, Span/TextRef, RepoPath, PathOrigin,
                       ManifestPath/PathPattern,
                       AbbrevBlobId
crates/planrun-parse   Markdown → IR: source, document, format, spec, registry, outline,
                       manifest, recognizer, recognize, kinds/ (one file per recognizer kind),
                       patch, check, diagnostic
crates/planrun-exec    IR → effects: git, workspace, eol, tree, handler, transaction,
                       preflight, finding (P3); shell/ (sessions, kill.rs: the only unsafe
                       code), expect, policy/ (classifier, decisions) (P4); journal, lock,
                       done, ui, runner/ (tasks, scripts, sessions, gate, resume) (P5);
                       lock.live, runner/preview (P6b); discover (P7: format specs, plan
                       globs, plan names)
crates/planrun-cli     the `planrun` binary: check, show (view, render), run (dry_run, run,
                       ui: line prompts), resume, status, update (P6a, D83), tui, list (P7);
                       the `e2e` test feature (e2e.rs)
crates/planrun-tui     the TUI (P6c, docs/tui-design.md): app, driver, terminal, jobs,
                       worker (the runner's thread and its Ui); views plans, inspect, run,
                       dock, history, preview, palette, help; theme, glyph, draw, chrome,
                       tree, diff, grid (snapshots); README.md: the manual check list
formats/*.toml         built-in plan formats (embedded): assay, superpowers, before-after; the
                       file name is the format id
docs/                  architecture, plan-grammar, extending, decisions, roadmap,
                       tui-design, plans/, handovers/ (one per finished phase)
tests/corpus/          synthetic/ (one plan per format), invalid/ (one error each), local/ (private)
tests/fixtures/greeter the repo the synthetic assay plan applies to
tests/support          fixture.rs: a temporary git copy of the greeter fixture, included
                       with #[path] by the exec and cli tests
scripts/               install.ps1, install.sh (release or --from-source, D83); unplus.ps1
.github/workflows/     ci.yml (the gate on Windows and Linux), release.yml (on a v*.*.* tag)
```

## Hard rules

1. **planrun-parse never executes and never touches the repository. planrun-exec never depends on
   planrun-parse** (D2). Don't add a dependency that crosses these lines.
2. **No partial plans** (D3). Any error means no `Plan`, nothing runs.
3. **Nothing is dropped.** Every block of a task becomes an action, `Note` or `Manual` (D21). Once
   a recognizer's distinctive lead matched, a broken idiom is a `Malformed` error, never a
   fallback.
4. **The action set is closed** (D4). New plan wording maps onto existing actions. Adding an
   `ActionKind` follows docs/extending.md §4, all in one change, and only with Rens' agreement.
5. **New plan styles are data first** (D8): a TOML spec, then a recognizer kind, then a hand-written
   `Format`, in that order of preference (docs/extending.md).
6. **Paths in actions are `RepoPath`** (D7). Nothing outside the repository is written except
   `.git/planrun/` and the OS temp directory. The working tree gets only what the plan writes.
7. **Patches go through `git apply`** with exact context (D20). No fuzz, no `similar`-based
   patching.
8. **Prose never passes a script on its own** (D11); expected failures are skipped by default (D12).
9. **No new dependency without a line in docs/decisions.md D9.**
10. **Real plans stay in `tests/corpus/local/`** (git-ignored, D16). Never copy their content into
    committed files, tests, snapshots, docs or commit messages. Synthetic plans only.

## Commands

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test -- --ignored local_corpus      # parses the private real plans, when present
cargo test -p planrun-cli --test local_classes -- --ignored --nocapture   # their script classes
cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings
cargo test -p planrun-cli --features e2e --test run   # the end-to-end runs (D82)
cargo run -p planrun-cli -- check tests/corpus/synthetic/assay-style.md
```

The gate for every task is fmt, clippy (warnings are errors) and test, each also with the `e2e`
feature of planrun-cli (D82).

## Code conventions

- Library code: no `unwrap`/`expect` (clippy warns; the gate fails). Tests may use
  `unwrap_or_else(|e| panic!("{e}"))`.
- Every public item has a doc comment (`missing_docs` warns).
- Diagnostics have a stable kebab-case `code`; tests pin codes, snapshots pin messages.
- Text from the plan is a `TextRef`, never a copied `String`, except `TextRef::Owned` where the
  source isn't contiguous.
- Prefer enums over strings, traits for extension points, composition over deep generic layering.
- Keep functions flat: guard clauses and early returns, at most three levels of nesting.
- Recognizers are pure; parameters are validated when a spec loads, not when a plan is parsed.

## Tests

- Unit tests sit next to the code (`mod tests`), or in `<module>_tests.rs` once a file passes
  ~400 lines.
- `insta` snapshots for IR and CLI output. A changed snapshot is a review item: explain it in the
  commit, never re-pin by reflex.
- Adding a check rule adds a file to `tests/corpus/invalid/`. Adding a format adds a plan to
  `tests/corpus/synthetic/` with its snapshot.
- Execution tests run on a temporary copy of `tests/fixtures/greeter` (`git init` in the copy),
  never on this repository: `tests/support/fixture.rs` makes one under the OS temp directory,
  with an environment that keeps the user's git configuration out. Exec tests build the IR by
  hand (`crates/planrun-exec/tests/support/ir.rs`, D2); tests that need a parsed plan live in
  planrun-cli.
- Shell tests (`crates/planrun-exec/tests/shell.rs`) skip a missing shell with a message;
  `PLANRUN_REQUIRE_SHELLS=1` makes it a failure, and every plan's verification sets it (D63).
- Runner tests (`crates/planrun-exec/tests/runner.rs`, `resume.rs`) drive the runner with a
  scripted `Ui` and crash it from the journal's observer; the binary's end-to-end runs are
  `crates/planrun-cli/tests/run.rs`, behind the `e2e` feature (D82).
- Table tests (classifier, matcher) keep their rows on one line each under `#[rustfmt::skip]`.

## Docs stay in step

- A changed decision: a new `D<n>` in docs/decisions.md that supersedes the old one.
- A changed spec schema: `spec.rs`, docs/extending.md §2 and `formats/*.toml` together (the
  `spec_files` test checks the files).
- A new recognizer kind: docs/extending.md §3.
- A new format: docs/plan-grammar.md.

## Working with Rens

- Workflow: brainstorm → design spec → implementation plan (`docs/plans/`, assay format, see
  `docs/plans/README.md`) → execution. Propose before making non-trivial edits; stop after writing
  a plan.
- Direct, technically precise; no flattery. Put review text and anything meant to be pasted in
  code blocks.
- Rens works on Windows with PowerShell 7. Commands in plans are PowerShell 7.
- Rulings relayed from a review seat (another Claude session with less context) are checked
  against the code and the plans before acting. Where one rests on a wrong premise, keep the
  correct position and state the evidence (file, task, what the plan says) in the plan's
  stated decisions or the reply, so a further review can settle it.
- One commit per task, message `P<N>-T<n>: <title>`. Don't push.
- When a phase is done, write a handover for the next session:
  `docs/handovers/YYYY-MM-DD-p<N>-to-p<N+1>.md` (state, what the phase built, open items, notes
  for the next phase, how plans are verified). A new session starts from the latest one.
  
````

File `README.md` (full contents):

````markdown
# planrun

Executes structured Markdown implementation plans against a git repository, task by task and step
by step, the way you would by hand: write the files the plan gives, apply its exact diffs, run its
commands, check their expected output, commit when the gate passes. Anything it cannot do
mechanically it shows and waits for. It is not an agent.

```
planrun list                   the plans of this repository, their formats and journals
planrun check  plan.md         parse, check, coverage report; changes nothing
planrun show   plan.md P1-T2   the typed form of a task
planrun run    plan.md         execute interactively (--dry-run: change nothing)
planrun resume plan.md         continue after an interruption, re-checking the disk
planrun status plan.md         what the plan's journal says
planrun update                 install the latest release (--check: only report)
```

A plan can be named instead of given by path: `planrun run p7` runs the one plan whose file name
contains `p7`. Plans are found with the globs of `.planrun/config.toml`, or in `docs/plans/`,
`**/superpowers/**/plans/` and `**/plans/` by default; formats of your own go in
`.planrun/formats/` ([docs/extending.md](docs/extending.md) §2).

Status: P0–P5 are done; P6 is planrun running its own plans. See
[docs/roadmap.md](docs/roadmap.md).

## Install

Windows (PowerShell 7 or Windows PowerShell 5.1), the latest release into
`%LOCALAPPDATA%\Programs\planrun\bin`, added to the user `PATH`:

```powershell
irm https://raw.githubusercontent.com/Renseck/planrun/main/scripts/install.ps1 | iex
```

Linux x86_64, the latest release into `~/.local/bin`:

```sh
curl -fsSL https://raw.githubusercontent.com/Renseck/planrun/main/scripts/install.sh | sh
```

From a clone, built with cargo into the same directory:

```powershell
.\scripts\install.ps1 -FromSource      # Windows
sh scripts/install.sh --from-source    # Linux
```

`-InstallDir` / `--install-dir` choose another directory. Then keep it current:

```
planrun update --check   is a newer release out?
planrun update           install it over this executable
```

`planrun update` uses curl and tar (Windows 10 1803 and later ship both) and replaces only its
own executable (decision D83).

## Documents

| | |
|---|---|
| [CLAUDE.md](CLAUDE.md) | rules for working on this code |
| [docs/architecture.md](docs/architecture.md) | how it works |
| [docs/plan-grammar.md](docs/plan-grammar.md) | the plan formats it reads |
| [docs/extending.md](docs/extending.md) | adding a plan format |
| [docs/decisions.md](docs/decisions.md) | decisions D1–D83 |
| [docs/roadmap.md](docs/roadmap.md) | phases P0–P6 |
| [docs/plans/](docs/plans/) | phase plans (assay format) |
| [docs/handovers/](docs/handovers/) | the state at the end of each phase |

## Build

```powershell
cargo build
cargo test
```

Rust 2024 edition, stable toolchain. Windows with PowerShell 7 is the primary platform; Linux
works. CI runs the full gate on both (`.github/workflows/ci.yml`); a pushed `v*.*.*` tag
releases (`.github/workflows/release.yml`).

## License

MIT; see [LICENSE](LICENSE).
````

File `crates/planrun-tui/README.md` (full contents):

````markdown
# planrun-tui

The planrun TUI, `planrun tui [<plans-dir>]`: Plans, the plan view in Inspect and Run, the
decision dock, History, the resume preview, the command palette and help. The design is
`docs/tui-design.md`; its decisions are D84–D94 in `docs/decisions.md`; its parts are described
in `docs/architecture.md` §7.1.

## How it is tested (D93)

Everything a test can see runs in-process: `driver::Core` over ratatui's `TestBackend`, fed
scripted keys, with the real runner on a copy of the greeter fixture (`tests/support/fixture.rs`).
Snapshots are cell grids (`grid::grid`): each row's characters, then its style runs by token
name, so they read against the design canvas's artboards cell for cell.

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-tui
```

## Manual check list

What only a real terminal shows. Run it in Windows Terminal with Cascadia Mono, from a copy of
the greeter fixture or any repository with plans the default globs find (or pass a directory), after
`cargo build -p planrun-cli`. Each item is a manual step, checked by hand.

1. **Start and quit.** `planrun tui` shows Plans; `q` quits and the terminal is as it was
   (prompt, scrollback, cursor). `planrun tui | Out-Host` and `planrun tui < $null` refuse with
   exit code 2.
2. **Glyphs** (§5). Every status, kind and chrome glyph is one cell wide and none is a box or
   a question mark; look hardest at `⚑` (U+2691) and the spinner `◐◓◑◒` while a script runs.
   With `$env:PLANRUN_ASCII = '1'` every glyph is its ASCII fallback. A glyph that fails here is
   replaced by its fallback in the table (§5).
3. **Colours** (§4). The header, footer and bands are bg1; the selected row bg2 with `▌` in
   blue; green only on verified things, amber only where planrun waits for you.
4. **Keys** (§7). F1–F4, `tab`/`shift+tab`, `j`/`k`/arrows, `enter`, `esc`, `R` and `shift+R`,
   `a`, `c`, `/`, `ctrl+k`, `ctrl+p`. A held arrow repeats; a held letter does not. In VS Code's
   terminal `ctrl+k` and `ctrl+p` reach the TUI only with
   `terminal.integrated.sendKeybindingsToShell` set (D94).
5. **A run.** `R` on a plan runs it; the navigator, CHANGES and the console follow the run; a
   question opens the dock over the console. A typed question takes the whole word and enter,
   and refuses a word it does not take.
6. **The `[o]` shell.** At a question that offers it, `o` leaves the screen and starts pwsh at
   the repository root; Ctrl-C in that shell reaches the shell (a running `ping -t` stops, the
   shell stays); `exit` comes back to the same question, the screen redrawn.
7. **Ctrl-C** (D91). During a run, Ctrl-C opens the END PLANRUN confirm; `n` keeps running; `y`
   restores the terminal and ends planrun mid-block, the session's processes gone (Task
   Manager); `planrun resume` (or History, F4) goes on from the journal.
8. **Resize.** Shrinking the window below 100 × 30 shows "terminal too small"; a question that
   opens then is shown alone and can be answered; growing it back restores the layout.
9. **A panic.** One on the render thread restores the terminal before Rust's report, and the
   shell prompt is usable after it. One on another thread (a background job, the runner)
   leaves the screen as it is and prints nothing over it: the job shows as failed, the run as
   ended with the panic's message (D94).
````

File `docs/architecture.md` (exact diff; save the block as `$env:TEMP\p7-t7-docs-architecture-md.patch`, outside the repository, then apply it):

````diff
diff --git a/docs/architecture.md b/docs/architecture.md
index 9d87fb7..3e20c4f 100644
--- a/docs/architecture.md
+++ b/docs/architecture.md
@@ -213,9 +213,11 @@ Three levels, cheapest first. A new plan style should use the lowest level that
 | 3. Format | a `Format` implementation in Rust | yes | a plan whose tasks are a table, or JSON |
 
 Built-in formats are **level-1 specs** too (`formats/*.toml`, embedded with `include_str!`): the
-path a user format takes is the path every built-in format takes (D8). User formats load from
-`.planrun/formats/*.toml` in the target repository, then the user config directory; a duplicate id
-is an error.
+path a user format takes is the path every built-in format takes (D8). Discovery (`discover` in
+planrun-exec, D96) finds the user's formats (`formats/*.toml` in the user config directory) and
+the repository's (`.planrun/formats/*.toml`); the registry layers them over the built-ins in that
+order, and a spec replaces the format with its id whole (D95). `check` and `show` name the file a
+format came from and what it replaced.
 
 On the execution side, the extension points are traits with registries too: `ActionHandler` per
 action kind (`HandlerSet`), `ShellSession` per shell, `CommandClassifier` for policy, `Ui` for the
@@ -423,6 +425,7 @@ append and each write of a step's commit in the order they were added: the `e2e`
 ```
 planrun check  <plan> [--format <id>]          parse + checks + coverage; writes nothing
 planrun show   <plan> [P1-T2[/1]]              the IR, a task, or a step
+planrun list                                   the discovered plans, their formats and journals
 planrun run    <plan> [--from P1-T2] [--to P1-T3] [--auto read,build] [--dry-run]
                [--run-expected-failures] [--restart]
 planrun resume <plan> [--auto …] [--from …] [--to …] [--run-expected-failures]
@@ -430,6 +433,15 @@ planrun status <plan>                          what the journal says; changes no
 planrun update [--check]                       install the latest release over this executable
 ```
 
+`<plan>` is a path, or a name (D97): an argument without a directory part or `.md` that names
+no existing file is matched, without case, against the file stems of the plans discovery finds
+(the repository's tracked and untracked files a glob of `.planrun/config.toml` matches, or the
+default globs `docs/plans/*.md`, `**/superpowers/**/plans/*.md` and `**/plans/*.md`; D96). One
+match is the plan, and planrun names it on stderr; none or several stop with exit code 2.
+`list` prints each discovered file with its format and its journal's state, or "not a plan";
+exit 1 when a file a format detects has errors. A spec or config file that cannot be used stops
+every command with exit code 2.
+
 `update` (D83) is the one command that is not about a plan: it compares the latest GitHub release
 with the build's version, and when it is newer downloads the build's target's archive with curl,
 unpacks it with tar under the OS temp directory, checks that the new executable prints the
@@ -476,7 +488,9 @@ Step 2 — verify
 ### 7.1 The TUI
 
 `planrun tui [<plans-dir>]` (docs/tui-design.md, D84–D94) runs from the repository of the current
-directory and needs a terminal on stdin and stdout (exit 2 otherwise). Its parts:
+directory and needs a terminal on stdin and stdout (exit 2 otherwise). Without a directory its
+Plans view lists the discovered plans, and the plans a journal names that no glob matches (D97).
+Its parts:
 
 - **`app::App`** holds all of its state and never blocks: a key or a message changes it and
   returns `Effect`s (a background job, the runner's start, the `[o]` shell, `end_now`, quit);
````

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t7-docs-architecture-md.patch"
Remove-Item "$env:TEMP\p7-t7-docs-architecture-md.patch"
```

File `docs/extending.md` (full contents):

````markdown
# Extending planrun

New plan styles will keep appearing. They are added without touching execution, in three levels
(architecture.md §5). Use the lowest level that works.

## 1. Decide the level

1. Describe the style in a table like the ones in [plan-grammar.md](plan-grammar.md), from at
   least two real plans.
2. For each row, find the built-in recognizer kind that covers it (§3). If every row is covered,
   write a **format spec** (§2). Done, no rebuild.
3. If a row is a new idiom, write a **recognizer kind** (§5) and use it from a spec.
4. Only if the plan's *structure* is not headings-with-steps (tasks in a table, a JSON plan),
   write a **Format** (§6).
5. A new **action kind** (§4) is almost never the answer: map the wording onto an existing action.

## 2. Format specs (TOML)

Location: built-ins in `formats/<id>.toml` (embedded); per user in `<user>/formats/<id>.toml`;
per repository in `<repo>/.planrun/formats/<id>.toml`. The file name is the id: letters, digits,
`-` and `_`. `<user>` is `PLANRUN_CONFIG_DIR` when it is set, else `%APPDATA%\planrun` on
Windows and `$XDG_CONFIG_HOME/planrun` or `$HOME/.config/planrun` elsewhere (D96).

The layers stack in that order (D95): a user spec with a built-in format's id replaces that
format, and a repository spec replaces either, whole (no merging). `planrun check` then says
`ok (assay (.planrun/formats/assay.toml, overrides built-in), 2 tasks)`. To change a built-in
format for one repository, copy `formats/<id>.toml` to `.planrun/formats/` and edit the copy. A
spec that does not load stops every command with exit code 2 and names its file.

Where plan files are found is the repository's `.planrun/config.toml` (D96), optional:

```toml
[plans]
globs = ["docs/plans/*.md", "work/**/plans/*.md"]   # replaces the defaults
```

Without it the globs are `docs/plans/*.md`, `**/superpowers/**/plans/*.md` and `**/plans/*.md`.
A glob is a manifest glob (D41) plus `**`, a whole segment that matches any number of
directories. `planrun list`, `planrun tui` and plan names (`planrun run p7`, D97) use them.

```toml
id = "assay"                      # unique; the file name
name = "Assay plan (planbuild.py)"

[detect]                          # a pattern that matches a line outside fenced blocks adds
patterns = [ { regex = '^### P\d+-T\d+: ', weight = 60 } ]   # its weight once; capped at 100

[outline]
tasks_after = { level = 2, regex = '^Tasks$' }          # optional
task = { level = 3, regex = '^(?P<id>(?P<phase>P\d+)-T(?P<ord>\d+)): (?P<title>.+)$' }
tasks_end_level = 2                                      # optional
# captures: id, ord (a number), title (all required); phase (optional, from the first task)

[outline.manifest]                                       # optional
# The marker and its items are consumed by the outline: they fall in no step. The items end at
# the first block that is no list item at the marker's depth, or that is a step boundary (D37).
marker = '^\*\*Files\*\*$'                                # may capture `change` (`**Files:** create`)
# and `items`: then the marker paragraph holds the manifest, split by `clause_list`, each
# clause matched by `item`; text that declares nothing is shown as a Note (D43)
item = '^(?P<change>Create|Modify|Delete) `(?P<paths>[^`]+)`$'   # `change` unless the marker has it
changes = { Create = "create", Modify = "modify", Delete = "delete" }
# → create | modify | rewrite | delete | test | generated (D27)
path_list = ', '                                         # optional: split `paths` into several;
# a `:3-10` piece adds lines to the path before it (D32). Built in, before the split, in every
# format: `a`–`b` (an en dash between code spans) is one range entry, so `path_list` must not
# list the en dash; a piece with `*`, `?` or `{` is a glob entry (D41). Pattern entries declare
# the files they match and are never a write target.
required = true                                          # optional, default true: false lets a
                                                         # task have no manifest (D27)
siblings = false                                         # optional, default false: a bare file
                                                         # name after `a/b.rs` is `a/<name>` (D32);
                                                         # without siblings it is Bare (D45)
clause_list = '; '                                       # optional: needs an `items` capture (D43)
path_change = '^` \((?P<change>full replacement)\)'     # optional (D44): over the text right
# after a path token; `change` goes through `changes` and wins over the item's change, which
# wins over the marker's

[[outline.steps]]                                        # zero or more; none = one implicit step
block = "paragraph"                                      # paragraph | list-item | heading
regex = '^\*\*Step (?P<n>\d+) — (?P<title>[^*]+)\*\*$'   # n and title optional
keep = false                                             # optional: true keeps the boundary
                                                         # block in its step (D46)

[settings]
manual_markers = ['(?i)\bby hand\b']                     # paragraph → Manual(Marked)
pwsh = ["powershell", "pwsh", "ps1"]                     # optional; these are the defaults
bash = ["bash", "sh", "shell"]

[[step_marks]]                                           # zero or more (D30), first match wins
title = '(?i)\bverify (?:it|they) fails?\b'              # regex over the step's title
mark = "expected-failure"                                # expected-failure | human-check
# expected-failure: the step's Verify scripts expect failure (none or `any` becomes Failure; an
# expected success is the error expected-conflict). human-check: its prose Notes become
# Manual(HumanCheck).

[checks]                                                 # optional (D31): only these two codes,
manifest-untouched = "off"                               # each error | warning | off; any
manifest-undeclared = "warning"                          # other key fails the spec load

[[recognizer]]                                           # ordered; first match wins
kind = "file-block"                                      # a registered kind (§3)
id = "file-block#assay"                                  # shown by `show` and coverage
# … the kind's own parameters
```

Rules:

- Unknown keys are errors (`deny_unknown_fields`). A spec is fully validated when the registry
  loads it (unknown kind, bad regex, bad parameter), before any plan is read.
- Regexes use the `regex` crate syntax; `(?P<name>…)` captures. A regex that lacks a capture
  its reader requires (`path`, `id`, …) is refused when the spec loads.
- `crates/planrun-parse/tests/spec_files.rs` checks every built-in spec against the schema.

## 3. Built-in recognizer kinds

Parameters in *italics* are optional.

| Kind | Matches | Produces | Parameters |
|---|---|---|---|
| `file-block` | a lead paragraph naming a path (and op), then one fence | `WriteFile` / `ApplyPatch` | `lead` (captures `path`, *`op`*), `ops` (op prefix → `write` \| `create` \| `overwrite` \| `patch`), *`default_op`*, *`file_name_from_manifest`* |
| `before-after` | lead (naming a path), fence, *separator paragraph*, fence | `ReplaceExact`, or `Manual(ProseEdit)` when `infer_path` finds no single entry | `lead` (captures `path`, or none with `infer_path = true`), *`separator`* (e.g. `^After:$`; absent = two fences back to back), *`file_name_from_manifest`*, *`series`* (a regex capturing `n`; the lead then captures `n` too, D49); a pathless lead may capture *`stem`* (D59) |
| `delete` | a sentence or inline `git rm` | `Delete` | `lead` (captures `path`), *`via_git`*, *`file_name_from_manifest`* |
| `generated` | a "generated, not by hand" sentence | `Generated` | `lead` (captures `path`) |
| `ceremony` | a fence directly after a given action whose body matches | nothing (absorbed) | `after` (`patch`), `body` |
| `script` | a fence in a shell language | `RunScript` / `Commit` | *`intents`* (`[{ intent, body?, lead? }]`: `commit` \| `gate` \| `preview` \| `apply` \| `verify` \| `other`), *`leads`* (regexes), *`first_block`*, *`line_expect`* (a regex capturing `expect`, D35, D48) |
| `run-line` | `` Run: `cmd` `` in a paragraph, with an optional expectation in the same paragraph | `RunScript` / `Commit` (one line) | `lead` (captures `cmd`, *`expected`*), `shell` (`pwsh` \| `bash`), *`exit`* table over `expected` (field `text`), *`intents`* (`[{ intent, body }]`) |
| `expected-block` | `Expected (…):` paragraph + `text` fence after a script | amends the script | `lead` (captures `spec`), `exit` table, `lines = "summary"` |
| `expected-prose` | `Expected: …` sentence after a script; with `list`, an `Expected:` paragraph and its list | amends the script | `lead` (captures `text`, unless `list`), `exit` table (`success` \| `failure` \| `any` \| `code:N`), *`list`* (D47) |
| `manual-prose` | a paragraph (+ the fence after it, with `takes_fence`) that describes what planrun cannot do exactly | `Manual(reason)` | `lead`, `reason` (`prose-edit` \| `output-dependent` \| `human-check`), *`takes_fence`* |

Notes on the kinds:

- `file-block`: the longest `ops` prefix of the captured `op` wins; `write` takes Create or
  Overwrite from the manifest. Once the lead matched, an op no prefix covers is
  `Malformed(unknown-file-op)`, a lead without its fence `Malformed(missing-block)`, and for
  `patch` a patch whose paths differ from the lead's `Malformed(patch-path)`.
- Paths from the manifest (D32): `file_name_from_manifest = true` resolves a lead path without
  `/` that is not an entry itself to the one entry with that file name, when that entry was
  written in full (not a sibling, not Bare); otherwise the path stays as written. `before-after`
  with `infer_path = true` takes the task's one entry that is not Test, Generated or Delete; with
  none or several (a Bare entry counts, but is never taken, D45), the lead and both fences
  become one `Manual(ProseEdit)`. Pattern entries are never candidates (D41). `infer_path` is never
  available for writes. Each resolved action carries its `PathOrigin`; `show` marks it.
- `run-line`: the command is the `cmd` capture, run as one line in `shell` (D39). The `expected`
  capture, when the lead has one and it matched, becomes a prose expectation through the `exit`
  table (no row matches: `any`). An empty command is `Malformed(missing-block)`.
- `before-after`: once the lead matched, a missing fence or separator is
  `Malformed(missing-block)`.
- `script`: `intent = "commit"` selects the `Commit` action kind, not a `ScriptIntent`; a commit
  script carries `ScriptIntent::Other`. The other intents set the `ScriptIntent`; no match is
  `verify`. A row matches the fence body (`body`), the paragraph directly before the fence
  (`lead`), or both; at least one is required (D29). Preview/apply pairs are rows like these.
- `script` `line_expect` (D35, D48): every line of a claimed fence that is not a whole-line
  comment and matches gives the script a comment expectation, exit not checked; an
  `expected-prose` right after it extends it instead of being `expected-twice`.
- `before-after` `series` (D49): a lead without a path continues the numbered edit directly
  before it in the step (`PathOrigin::PreviousEdit`), before `infer_path` is tried. A
  pathless lead's `stem` capture (`README edit 2`) must equal the file-name stem of the path
  it continues or infers, or the edit is `Manual(ProseEdit)` (D59).
- `expected-prose` `list` (D47): the list items at the lead's depth and the paragraphs, list
  items and `text` fences nested in them belong to the expectation; no list is
  `missing-block`.
- `script` guard (D28): with `leads` set, the kind claims a fence only when the paragraph
  directly before it matches one of `leads`, or, with `first_block = true`, when the fence is the
  first block of a written step (never the implicit first step, so never right after the
  manifest, and never in a step with a kept label, D46). A body intent never qualifies a fence
  on its own. Any other shell fence falls to the fallback, `Manual(Unrecognized)`. Without
  `leads` (the assay spec) every shell fence is a script.
- `expected-block` and `expected-prose` amend the script directly before them. Anything else there
  is `Malformed(expected-without-script)`; a second expectation on one script is
  `Malformed(expected-twice)`. `expected-block` needs a `text` fence (`missing-block`). An
  `expected-prose` sentence directly after a `Manual` joins that Manual's text instead (D38).
- Coverage counts the blocks of a recognizer's `Manual` (and of an expectation joined to a
  Manual) as manual, not as actions.
- **Exit tables are first match**: rows are tried in order and the first whose regex matches
  decides, so their order matters. Values: `success`, `failure`, `any` (`ExitExpect::Unknown`: not
  checked; prose is still confirmed, D11), `code:N`, and `code` (the row's regex captures `code`;
  0 is `success`). An `expected-block` spec no row matches is `Malformed(expected-spec)`; an
  `expected-prose` text no row matches is `any`.

The **fallback** (not configurable) runs when nothing matched: paragraph, list item, blockquote →
`Note` or `Manual(Marked)`; fence → `Manual(Unrecognized)` / `Manual(ForeignLanguage)`, a `text`
fence → `Note`; table, heading, rule → `Note`; HTML comment → dropped.

## 4. Adding an action kind

Rare (D4). The checklist, all in one change:

1. The variant in `planrun-model::ActionKind`, with docs.
2. Its handler in planrun-exec: `simulate` on the virtual tree (its dry-run behaviour and
   its part of a step transaction, D53), and its `DoneState` logic for resume (P5).
3. Its policy class.
4. Its journal record.
5. Its `show` rendering and coverage label.
6. A corpus plan that produces it, and an invalid-corpus file for each new error.

## 5. Adding a recognizer kind

```rust
pub struct InsertAfterFactory;

impl RecognizerFactory for InsertAfterFactory {
    fn kind(&self) -> &'static str { "insert-after" }

    fn build(&self, params: &toml::Table) -> Result<Arc<dyn Recognizer>, String> {
        // Read and validate every parameter here: a bad regex fails when the spec loads.
        Ok(Arc::new(InsertAfter { lead: regex(params, "lead")?, id: id(params)? }))
    }
}

struct InsertAfter { lead: Regex, id: String }

impl Recognizer for InsertAfter {
    fn id(&self) -> &str { &self.id }

    fn recognize(&self, cx: &StepContext<'_>, cursor: &mut Cursor<'_>) -> Recognized {
        // 1. Peek; return Recognized::No without advancing unless the lead matches.
        // 2. Once the lead matched, a missing or wrong block is Recognized::Malformed, not No.
        // 3. Advance past exactly what was consumed; return the actions.
        Recognized::No
    }
}
```

Register it in `Registry::with_builtins`. Rules:

- **Pure.** No I/O, no clock, no state between calls. Same input, same output.
- **`No` leaves the cursor where it was.** Advance only on `Yes`/`Amend`.
- **Commit to the idiom early.** Once the distinctive lead matched, failures are `Malformed`
  errors. Falling through to `Manual` would hide a broken plan.
- **Tests**: unit tests on hand-built blocks, plus a corpus plan in `tests/corpus/synthetic` and
  an invalid-corpus file per `Malformed` code.

## 6. Adding a Format by hand

Implement `Format` (`detect`, `outline`, `recognizers`, `settings`) and register it with
`Registry::add_format`. Reuse recognizer instances from the factories for the step contents: only
the outline should be new. Every block of every task must fall in exactly one step range.

## 7. Execution-side extension points

| Trait | Registry | For |
|---|---|---|
| `ActionHandler` | `HandlerSet` | how an action kind is simulated on the virtual tree (D52, D53); the transaction writes the result, and resume compares the journaled step with the disk (`done`, D77), so a handler has no write and no `DoneState` of its own |
| `ShellSession` | one per `Shell` | another shell (`cmd`, `zsh`): a new `Shell` variant, a `Dialect` in `shell/` (launch arguments, prologue, block line, sentinel; D60), its discovery and probe (D63), and its splitter in `policy/` |
| `CommandClassifier` | the runner's | the tests' own classifier; `DefaultClassifier` is the built-in allowlist (D68), extended by adding rows to `policy/heads.rs` and their test rows, never from a format spec |
| `Ui` | the CLI | `status`, `output`, `ask` (a `Question`: its `QuestionKind` with what the run found (D86), named `Choice`s, or the words a typed line may be), `view`, `diff`, `shell`; the runner decides what is asked and acts on the answer (D74). planrun-cli's `LineUi` is line prompts on std; planrun-tui's `ChannelUi` forwards to the TUI's render thread, `ask`, `view`, `diff` and `shell` as blocking round trips (D90); the runner tests script their answers; a non-interactive batch UI later. A new question is a new `QuestionKind` variant, and every UI's exhaustive match shows where it must be shown |
````

File `docs/roadmap.md` (full contents):

`````markdown
# Roadmap

Each phase gets its own plan in `docs/plans/` (conventions: [plans/README.md](plans/README.md)),
reviewed by Rens before it is executed. A phase is done when its acceptance list holds and
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` are green.

The order puts the riskiest assumption first: that real plans can be parsed strictly, with full
accounting of every block.

## P0 — Starter package (done)

Workspace of four crates; the IR in `planrun-model` with its tests; the parse and exec extension
points as traits; the CLI argument surface; `formats/assay.toml`; the synthetic and invalid
corpus; the greeter fixture; these docs.

## P1 — Parse the assay format strictly (done)

The deliverable: `planrun check` and `planrun show` on assay plans, with precise diagnostics.

Suggested tasks (the plan may split or merge them):

1. **Source and document.** `source::load` (UTF-8, ≤ 64 MiB, BOM, CRLF/CR → LF, SHA-256 of the
   original bytes; adds `sha2`, D9). The document builder over pulldown-cmark's offset iterator:
   every `Block` variant, spans, list depth, `TextRef::Owned` for fences inside list items, and
   the `unclosed-fence` error. Unit tests on small strings, including a ```` ```` ```` fence that
   contains ```` ``` ```` fences, a fence inside a list item, CRLF input, and an empty fence body.
2. **Specs, registry, format selection, outline.** `Registry::add_spec` (deserialize, compile
   every regex, build recognizers through factories, reject unknown kinds and duplicate ids),
   `with_builtins` embedding `formats/assay.toml`, the generic `SpecFormat` (`detect`, `outline`:
   tasks after `tasks_after`, task ids and ordinals, phase, intro step, manifest, step ranges,
   `tasks_end_level`), selection with threshold and ties (`no-format`, `ambiguous-format`).
3. **Recognition.** The cursor loop with `No`/`Yes`/`Amend`/`Malformed` and the fallback; the
   kinds the assay spec uses: `file-block`, `ceremony`, `delete`, `generated`, `script`,
   `expected-block`, `expected-prose`. Patch parsing for `ApplyPatch` (target path from
   `diff --git`/`---`/`+++` agrees with the lead: `patch-path`; the `index` line → pre/post).
4. **Checks.** The rules `task-order` (with `duplicate-task-id`), `step-order`, `manifest`
   (`manifest-undeclared`, `manifest-untouched`, `manifest-write-mode`), `patch-shape`,
   `blob-chain`, `commit` (`commit-twice`, `commit-not-last`); the coverage report. The
   invalid-corpus test (`tests/corpus/invalid/README.md`), which also pins the codes of the
   outline and the recognizers (`bad-path`, `unknown-file-op`, `missing-block`, `patch-path`, …).
5. **CLI.** `planrun check` (diagnostics via `annotate-snippets`, coverage table, exit 0 / 1 on
   errors / 2 on usage) and `planrun show` (plan, task, task/step). An `insta` snapshot of the IR
   of `assay-style.md` and of `planrun check` output. The `#[ignore]`d local-corpus test.

Acceptance:

- `planrun check tests/corpus/synthetic/assay-style.md` exits 0; every block of every task is an
  action except the `**Goal:**` lines and prose (Notes); the absorbed ceremonies are listed as
  absorbed.
- Every `tests/corpus/invalid/*.md` yields exactly its pinned error code.
- The two local assay plans (`2026-11-05-s5-1-p1-routes-seam.md`,
  `2026-11-11-s5-1-p3c1-builder-core.md`) parse with zero errors. Expect to widen the `generated`
  pattern for "`Cargo.lock` changes: it is regenerated by cargo in the next step", and to see
  the P3c-1 plan's prose-heavy preamble as Notes.
- Every diagnostic shows file:line:col and the source line.

## P2 — The other three formats (done)

The decisions are D26–D40. Split in two plans on one schema (D40):

- **P2a (done).** The P1 outline fixes and `task-after-end` (D37); the spec schema (`required`,
  a marker that captures `change`, `siblings`, `ChangeKind::Generated`, `[checks]`,
  `[[step_marks]]`, `script` `leads`/`first_block`/intent `lead`); the kinds `run-line`,
  `before-after`, `manual-prose`; path resolution from the manifest (D32);
  `formats/superpowers.toml` and `before-after.toml` with IR snapshots of their synthetic plans.
  The superpowers and before-after local plans parse with zero errors. Plan:
  `docs/plans/2026-11-24-p2a-hand-written-formats.md`.
- **P2b (done).** First, pattern manifest entries: a range or glob item is one entry whose
  target is a `PathPattern` (D41, D42). Then the schema for hand-written DevReady plans: inline
  Files clauses (D43), a change per path (D44), Bare file names (D45), kept step labels (D46),
  `Expected:` with a list (D47), comment expectations that a sentence extends (D35, D48),
  numbered edit series (D49). Then `formats/devready.toml` with the `**Commit:**` Manual (D33),
  detection that outscores superpowers on both devready plans, the IR snapshot of its synthetic
  plan, and the Apply invariant in plan-grammar §5. All six local plans parse with zero errors.
  Plan: `docs/plans/2026-11-25-p2b-devready-format.md`.

Each plan lists the imperative Notes that remain in the real plans (D36). `expected-failure-step`
became step marks (D30); `preview-apply` became script intents by lead (D29).

## P3 — Workspace, git and dry-run (done)

`RepoPath` resolution with the symlink check; the per-task file cache; step transactions (D23);
line-ending policy (D10); pre-images in the git object database; the `git` wrapper (`apply
--check`/`apply` on stdin, `hash-object --path`, `status --porcelain`, `cat-file`); blob-id checks;
task preflight; `planrun run --dry-run` against a virtual tree. Integration tests on a temporary
copy of `tests/fixtures/greeter`: the dry run of `assay-style.md` passes; a fixture with
`src/lib.rs` edited fails preflight with the blob mismatch; a Create onto an existing file fails
before anything is written.

The decisions are D50–D59. Plan: `docs/plans/2026-11-25-p3-workspace-dry-run.md`. First the P2b
review items (series stems D59, `run-from`, negated failures, `write-mode-unknown` D58) and the
`RepoPath` hardening (D51); then the git wrapper, the workspace and its one writer, line endings
(D54), the virtual tree (D52), step transactions as prepare and commit (D53), preflight with
scope (D50) and the dry run (D55). The per-task file cache is the virtual tree: a task's paths are
read once, on first use.

## P4 — Shells, expectations, policy (done)

Persistent pwsh and bash sessions (D24) with the sentinel protocol: variables persist across
blocks, a `throw` ends the block and not the session, output streams. The expectation matcher
(summary-line subsequence with `…`, exit expectations, prose → confirmation, D11). The command
classifier (D13). Expected failures skipped by default (D12). Tests for each, with pwsh and bash
when present (skipped with a message when not).

The decisions are D60–D70. Plan: `docs/plans/2026-11-25-p4-shells-expectations-policy.md`. First
the decisions and the `unsafe` lints (D64); then the sessions (the protocol D60, how a block
ends D61, the environment D62, which shells D63, the process tree D64); the matcher (D66); the
classifier (D68) and the policy decisions (D67, D69); the classes in the dry run. The candidate
(a script working directory from a run lead) is deferred (D70). The P4 review items P5 must fix
first are in `docs/handovers/2026-11-25-p4-to-p5.md`.

## P5 — Runner, journal, resume, interactive CLI (done)

The runner state machine; the commit gate (D22); the JSONL journal (D18); resume with `DoneState`
(D14); `run`, `resume`, `status`; the line-prompt UI (D15) with view/diff/shell/skip/quit. End to
end: `planrun run tests/corpus/synthetic/assay-style.md --auto read,build,modify` on a fixture copy
produces the same two commits (same trees) as the fixture's history; killing the process mid-task
and resuming completes it without repeating a write.

The decisions are D71–D82. Plan: `docs/plans/2026-11-25-p5-runner-journal-resume.md`. First the
P4 review items for sessions (D71) and sed (D72); then the transaction hardening and the step's
state on disk (D78), the journal and the lock (D76, D77), the runner with the commit gate and the
checks around a commit (D73–D75, D79–D81), resume (D77), and the CLI with line prompts (D74).
The fixture has no history of its own: the acceptance test builds the two expected trees, and
drives the binary through the test-only `e2e` feature (D82), uninterrupted, killed at a
question, and ended at three seams. The synthetic plan writes `docs/modules.txt` with LF only,
and the greeter ignores `Cargo.lock`.

## P6 — Dogfood and polish

planrun runs its own later phase plans. Candidates, each its own decision: a ratatui UI, a
sidecar IR from planbuild.py, a non-interactive batch mode for CI. `.planrun/formats/` discovery
became P7.

- **P6a — release and update (done).** The CI gate on Windows and Linux, the release workflow on
  a `v*.*.*` tag, `scripts/install.ps1` and `install.sh` (a release, or `--from-source`), and
  `planrun update [--check]` (D83). Plan: `docs/plans/2026-11-26-p6a-release-update.md`; the
  first plan planrun executes on itself. Open items: `docs/handovers/2026-11-26-p5-to-p6.md`.
- **P6b — the TUI's prerequisites.** The TUI's design (`docs/tui-design.md`) and its decisions
  (D84–D92), then what it needs from planrun-exec (its §9): a journal with any number of
  observers, `shell::end_now`, typed effects in the dry-run report, the matcher's alignment,
  `QuestionKind`, and the read-only resume preview. The CLI's behaviour and the journal format do
  not change. Plan: `docs/plans/2026-11-26-p6b-tui-prerequisites.md`.
- **P6c — the TUI.** The `planrun-tui` crate and `planrun tui`: theme and glyphs, the shell, the
  background work, Plans, the plan view (Inspect, then Run through its `Ui` and a journal
  observer), the dock with every `QuestionKind`, History, the resume preview and the command
  palette (`docs/tui-design.md` §11). Its acceptance drives the TUI against the greeter fixture
  in-process, with the real runner (D93). Plan: `docs/plans/2026-11-27-p6c-tui.md`.

## P7 — Discovery

One `Discovery` in planrun-exec (D96) finds what the repository and the user's config directory
hold: format specs in `<user>/formats/` and `.planrun/formats/`, layered over the built-ins with
the source shown (D95); the plan globs of `.planrun/config.toml` or the defaults; and the plan
files they match, with one `git ls-files`. Plans can be named on the command line (D97),
`planrun list` lists them, and `planrun tui` without a directory shows them. Design:
`docs/superpowers/specs/2026-11-28-discovery-design.md`; plan:
`docs/plans/2026-11-28-p7-discovery.md`.
`````

File `docs/tui-design.md` (exact diff; save the block as `$env:TEMP\p7-t7-docs-tui-design-md.patch`, outside the repository, then apply it):

```diff
diff --git a/docs/tui-design.md b/docs/tui-design.md
index f51038f..2280b41 100644
--- a/docs/tui-design.md
+++ b/docs/tui-design.md
@@ -52,8 +52,9 @@ docs/decisions.md holds them as D84–D92, with what P6b settled.
   gives it crossterm, the tests a `TestBackend`, scripted keys and the real runner on a
   fixture copy. Snapshots are cell grids with style runs by token name.
 - **D94 — How `planrun tui` starts.** A terminal on stdin and stdout, or exit 2; the plans
-  directory defaults to `docs/plans` under the repository's root; only key presses count, and
-  one loop polls keys and drains the app's channel; times are UTC.
+  directory defaults to `docs/plans` under the repository's root (D97: without a directory, the
+  discovered plans); only key presses count, and one loop polls keys and drains the app's
+  channel; times are UTC.
 
 Not in v1: **pause**. Asking under `--auto` turns Auto into Ask, a decision change the UI may not
 make. If wanted, it is its own decision with runner support: a flag read before each action,
```

```powershell
git apply --whitespace=nowarn "$env:TEMP\p7-t7-docs-tui-design-md.patch"
Remove-Item "$env:TEMP\p7-t7-docs-tui-design-md.patch"
```

**Step 2 — verify**

Targeted: the new sections are there:

```powershell
(Select-String -Path docs/roadmap.md -Pattern '^## P7 — Discovery').Count
(Select-String -Path docs/extending.md -Pattern '\[plans\]').Count
```

Expected (exit code 0; summary lines):

```text
1
1
```

Gate:

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo fmt --check; if ($LASTEXITCODE -eq 0) { cargo clippy --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test }; if ($LASTEXITCODE -eq 0) { cargo test -p planrun-cli --features e2e --test run }; $gate = $LASTEXITCODE; "gate exit code: $gate"
```

Expected (exit code 0; summary lines):

```text
gate exit code: 0
```

**Step 3 — commit**

Only the task's paths are staged, only when the gate's exit code above was 0, and only when every staged file is the verified one:

```powershell
if ($gate -eq 0) {
  git add -- CLAUDE.md README.md crates/planrun-tui/README.md docs/architecture.md docs/extending.md docs/roadmap.md docs/tui-design.md
  $verified = [ordered]@{
      'CLAUDE.md' = '9483d2e7a0af664f3c443742d981a49dd8d8561f'
      'README.md' = '77cc915d53de0978a1cec6034f59cff1448efc15'
      'crates/planrun-tui/README.md' = '6efa318d8958d3a26b192b57da2d3243992eb749'
      'docs/architecture.md' = '3e20c4f4bbde23a440ac86fbb6c379172d50e2c6'
      'docs/extending.md' = '8f8445828adab673ecbdee182ab5f2a91eee8826'
      'docs/roadmap.md' = '9cfb5898f59664635ae36f679a447d0c5ab45038'
      'docs/tui-design.md' = '2280b41262788f0f4ef6d152d13247ff7b3a452c'
  }
  $differ = @($verified.Keys | Where-Object { (git rev-parse ":$_") -ne $verified[$_] })
  if ($differ.Count -eq 0) { git commit -m "P7-T7: discovery in architecture, extending, the READMEs, CLAUDE.md and the roadmap" }
  else { "staged files differ from the verified tree: $($differ -join ', ')" }
} else { "the gate failed (exit code $gate): fix the task before committing" }
git status --porcelain
```

Expected: the commit, then no output from `git status --porcelain`. A line naming files instead of the commit means those files differ from the verified tree.

## After execution

Each step is a manual step for Rens; none is run by planrun.

1. By hand: in this repository, run `planrun list` and `planrun check p6a`, and check that `list` shows every phase plan in `docs/plans/` with `assay` and `docs/plans/README.md` as "not a plan".
2. By hand: start `planrun tui` without a directory and check that the footer says `plans default globs` and the Plans view lists the same files as `planrun list`; create an untracked `docs/plans/scratch.md` and check that it appears within a few seconds, then delete it.
3. By hand: push `main`; the `ci` job on `ubuntu-latest` runs P7-T4's symlink test for the first time.
4. Claude writes the P7→P8 handover (CLAUDE.md, "When a phase is done"), with the open items of `docs/handovers/2026-11-28-p6c-to-p7.md` that P7 did not close and this plan's stated decisions 8 and 9.
