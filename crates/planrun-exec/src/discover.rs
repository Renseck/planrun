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
