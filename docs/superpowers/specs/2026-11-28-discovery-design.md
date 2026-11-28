# P7 — Discovery: repository formats, user formats and plan files (design)

**Status:** approved in brainstorming on 2026-11-28; the plan follows in
`docs/plans/2026-11-28-p7-discovery.md` (assay format, so planrun can execute it).

## Goal

planrun finds what a repository holds without being told:

- format specs in `<repo>/.planrun/formats/*.toml` and in the user config directory, as
  architecture.md §5 and extending.md §2 already promise (today only the built-ins load);
- plan files wherever the repository keeps them (`docs/plans/`, `docs/superpowers/plans/`,
  `*/superpowers/*/plans/`, …), for the TUI's Plans view and a new `planrun list`;
- a plan given by name instead of path: `planrun run p6c`.

One object, `Discovery`, serves all three.

## Decisions taken in brainstorming

| Topic | Choice | Reason |
|---|---|---|
| Scope | Formats, plan globs with built-in defaults, `.planrun/config.toml`, CLI `list` and name arguments | One plan of ordered tasks |
| Same id in two layers | Override, precedence repository > user > built-in, whole replacement, source printed | Local fixes to a built-in format without renaming; shadowing is visible |
| `plans.globs` in the config | Replaces the defaults | Predictable; lets a repository narrow the set |
| Name matching | Case-insensitive substring of the file stem; ambiguity is an error | Short to type; "exactly one" is never a guess |
| Structure | One `Discovery` value: config and formats eager, plans on request, no cache | No git process for `check <path>` (D25); no invalidation state |
| Not included | `[run] format` default in the config, provider traits, a cached file list, automatic listing of new untracked plans in the TUI | YAGNI; `--format` covers exceptions; `c` refreshes |

## Where it lives

A `discover` module in **planrun-exec**. It already owns `Git` and `Workspace`, and both
planrun-cli and planrun-tui depend on it. It returns paths and raw spec text, never a `Format`, so
it does not depend on planrun-parse (D2). The callers hand the text to the registry.

No new crate: one `git ls-files` replaces a directory walk, globs reuse `PathPattern` (D41), and the
user config directory is computed from environment variables. No D9 line.

## The object

```rust
pub struct Discovery {
    root: Option<PathBuf>,       // the repository root; None outside a repository
    config: Config,              // .planrun/config.toml, or the defaults
    formats: Vec<FoundSpec>,     // user then repository layer, in that order
}

pub struct FoundSpec {
    pub id: String,              // the file stem
    pub text: String,            // loaded like a plan: UTF-8, BOM stripped, CRLF → LF
    pub path: PathBuf,
    pub layer: Layer,            // User | Repository
}

impl Discovery {
    pub fn open(cwd: &Path, env: &[(OsString, OsString)]) -> Result<Self, DiscoveryError>;
    pub fn root(&self) -> Option<&Path>;
    pub fn formats(&self) -> &[FoundSpec];
    pub fn plans(&self, git: &Git) -> Result<Vec<RepoPath>, DiscoveryError>;
    pub fn resolve(&self, git: &Git, arg: &str) -> Result<PathBuf, DiscoveryError>;
}
```

`open` reads a few directories and at most one config file; it starts no process. `plans` runs
one `git ls-files` per call and keeps nothing; the caller holds the result.

### Root

`open` walks up from `cwd` to the first `.git`, a directory or a file (worktree, submodule). The
repository is the current directory's (as for `run` and the TUI, D88, D94), also for `check` and `show` of a plan elsewhere. No
`.git` means no repository layer and no config: user and built-in formats only, not an error.
`plans` and `resolve` by name then fail with `not-a-repository`.

### Format directories

- `<root>/.planrun/formats/` and the user directory: `%APPDATA%\planrun\formats` on Windows;
  `$XDG_CONFIG_HOME/planrun/formats`, else `$HOME/.config/planrun/formats`, elsewhere.
- A missing directory is empty. Only `*.toml` files are read; subdirectories and other files are
  ignored. Sorted by file name.
- A file stem that is not UTF-8 or not a valid id is `format-file-name`. `add_spec` already
  requires the spec `id` to equal the stem (registry.rs).
- Any failure stops the command (exit 2) and names the file. A broken spec is never skipped, also
  not a broken user-level spec that every repository sees (D3, D1).

### Config

```toml
# .planrun/config.toml
[plans]
globs = ["docs/plans/*.md", "work/**/plans/*.md"]
```

- `deny_unknown_fields`; loaded like a spec (UTF-8, BOM, CRLF).
- `globs` replaces the defaults. Each glob compiles when the config loads: a bad one is
  `config-glob` with the pattern; `globs = []` is `config-globs-empty`.
- No config file: the defaults `docs/plans/*.md`, `**/superpowers/**/plans/*.md`, `**/plans/*.md`.

### Discovery globs

`PathPattern::discovery_glob(text)`: the D41 glob plus `**`, which matches zero or more whole
path segments (`a/**/b` matches `a/b` and `a/x/y/b`). `**` must be a whole segment; `a**` is an
error. Manifest globs (`PathPattern::glob`) keep refusing `**`, so plans do not change.

### Plans

- `git ls-files -z --cached --others --exclude-standard` from the root: tracked and untracked
  files, ignored ones (`tests/corpus/local/`) left out; `-z` avoids quoting non-ASCII paths.
- Kept: paths a glob matches, then one `symlink_metadata` each; a path that is missing (deleted
  but still indexed) or a symlink is dropped.
- Sorted by path, deduplicated across globs.
- No detection here. The candidates go to the registry in the caller; "not a plan" stays the
  caller's concern (D2).

### Resolve

- An argument that contains `/` or `\`, or ends in `.md`, is a path: used as given; a missing
  file is the existing I/O error. No fallback to names.
- Otherwise a name: the plans whose file stem contains it, case-insensitive. One is used; `run`
  and `resume` print the resolved path first. Several is `plan-ambiguous` listing them; none is
  `plan-not-found`. Both exit 2.
- Every candidate counts, a README too: an unwanted match makes a name ambiguous, never wrong.

## The registry's layers

`FormatSource` in planrun-parse: `BuiltIn`, `User(PathBuf)`, `Repository(PathBuf)`.

- `Registry::add_spec_layer(origin, text, source)` compiles the spec like `add_spec`; a format
  with the same id from an earlier layer is replaced whole, its source remembered as overridden.
  Two specs with one id in the same layer are impossible (file name = id).
- `Registry::source(id)` gives the source and what it overrides. `check`, `show`, `list` and the
  TUI's inspect view print it: `superpowers (.planrun/formats/superpowers.toml, overrides
  built-in)`.
- `add_format` keeps its duplicate error for hand-written formats.
- A shared builder (`with_discovered(&Discovery)`) in the caller: built-ins, then user, then
  repository. It replaces both `Registry::with_builtins()` call sites in planrun-cli and the TUI's.

## Callers

### CLI

- `check`, `show`, `run`, `resume`, `status` take a plan path or a name (Resolve).
- `planrun list`: each discovered plan with its format and source, or "not a plan", and its
  journal state (never run, done, interrupted at a task), one line each. Exit 0; 1 when a
  file a format detects has errors ("not a plan" is not an error); 2 outside a repository.

### TUI

- `planrun tui` with no argument lists discovered plans; `planrun tui <dir>` keeps D94's
  behaviour (that directory's `*.md`, not recursive).
- Plans with a journal (`.git/planrun/`) that still exist but no glob matches are listed too,
  marked as outside the globs.
- The footer's plans label: `globs: <n>` (defaults) or `.planrun/config.toml`, or the directory
  when one was given.
- Refresh: `c`, and the 2-second check also when `.git/index`'s modification time changed. A new
  untracked plan appears on `c`.

## Errors

Every discovery error has a stable kebab-case code and exits 2: `not-a-repository`,
`format-file-name`, `format-io`, `config-io`, `config-parse`, `config-glob`, `config-globs-empty`,
`git-ls-files`, `plan-ambiguous`, `plan-not-found`. A spec that fails to compile keeps the
registry's `invalid-spec` / `unknown-kind`, with the file path added.

## Testing

- `discover` unit tests: the root walk (`.git` directory, file, none), format directories (missing,
  non-`.toml`, bad stem, BOM, CRLF), config (unknown key, bad glob, empty globs), the user
  directory from the environment.
- `discovery_glob`: a table test, one row per line under `#[rustfmt::skip]`; `PathPattern::glob`
  still refuses `**`.
- Exec integration on the greeter fixture copy: an untracked plan listed, an ignored one not, a
  deleted-but-indexed one dropped, a symlink dropped (Unix; skipped with a message on Windows),
  dedupe and order across globs, `resolve` for path, one, several and none.
- `registry_tests`: precedence repository > user > built-in, whole replacement, the source and
  override reported.
- CLI e2e (`e2e` feature): `check <name>`; `list` as an insta snapshot; `check` printing
  `overrides built-in` with a synthetic `.planrun/formats/` in the fixture copy.
- TUI: a grid snapshot of the Plans view with discovered, journal-only and not-a-plan rows and the
  globs footer.
- Synthetic specs and plans only (rule 10). No new `tests/corpus/invalid/` files: these are
  discovery errors, not plan checks.

## Decisions and docs

Each lands with the task that implements it:

- **D95 Format layers**: built-in < user < repository, whole replacement, source printed.
  Supersedes the duplicate-id sentence (architecture.md §5, extending.md §2).
- **D96 Discovery**: one `Discovery` in planrun-exec; one `git ls-files`, no walk crate; the
  discovery-only `**` (amends D41); the default globs; `config.toml` v1 with `plans.globs`
  replacing the defaults; the user config directory without `dirs`; broken specs stop the command.
- **D97 Plans by name**: path first, then a stem substring, ambiguity an error; the TUI defaults
  to discovery, an explicit directory keeps D94's listing (amends D94).

Docs: architecture.md §5, extending.md §2 (config file, layers), tui-design.md (Plans view),
CLAUDE.md's layout block (`discover` in planrun-exec, `list` in planrun-cli), roadmap.md (P7
entry; the `.planrun/formats/` polish item moves there). plan-grammar.md is unchanged.

## Task order

Build, clippy and tests pass after each, also with the `e2e` feature.

1. `PathPattern::discovery_glob` with `**`, and its table test.
2. `exec::discover`: root, config, format directories (no git). D96.
3. `FormatSource`, `Registry::add_spec_layer`, the shared builder; CLI and TUI load discovered
   formats; `check` and `show` print the source. D95.
4. `Discovery::plans` and `resolve`; name arguments on `check`, `show`, `run`, `resume`,
   `status`. D97.
5. `planrun list`.
6. The TUI's Plans view: discovery, journal-only plans, footer, refresh on `.git/index`.
7. Docs: architecture, extending, tui-design, CLAUDE.md, roadmap, handover.
