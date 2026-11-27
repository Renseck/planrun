//! `planrun update` (decision D83): the latest GitHub release, compared with this build, and
//! installed over the running executable when it is newer. curl fetches and downloads, tar
//! unpacks (both external tools, like git); the new executable must report the release's
//! version before it replaces the running one.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use crate::{OK, PLAN_ERRORS};

/// GitHub's description of the latest release (pre-releases and drafts are never "latest").
const LATEST: &str = "https://api.github.com/repos/Renseck/planrun/releases/latest";

/// `planrun update [--check]`: exit 0 when up to date, reported or updated; 1 when the update
/// failed, with the installed executable unchanged.
pub fn run(check: bool, out: &mut String) -> u8 {
    match update(check, out) {
        Ok(()) => OK,
        Err(e) => {
            out.push_str(&format!("planrun: {e}\n"));
            PLAN_ERRORS
        }
    }
}

/// The update check for the TUI's indicator (D83): what `planrun update --check` would say.
pub fn state() -> planrun_tui::jobs::UpdateState {
    use planrun_tui::jobs::UpdateState;
    let Some(installed) = Version::parse(env!("CARGO_PKG_VERSION")) else {
        return UpdateState::Failed;
    };
    let latest = Tools::find()
        .and_then(|tools| tools.fetch(LATEST))
        .and_then(|json| Release::from_json(&json));
    match latest {
        Ok(release) if release.version > installed => {
            UpdateState::Available(release.version.to_string())
        }
        Ok(_) => UpdateState::Current,
        Err(_) => UpdateState::Failed,
    }
}

fn update(check: bool, out: &mut String) -> Result<(), UpdateError> {
    let Some(installed) = Version::parse(env!("CARGO_PKG_VERSION")) else {
        return Err(UpdateError::new(
            "update-unsupported",
            "this build's version is not major.minor.patch",
        ));
    };
    let tools = Tools::find()?;
    let release = Release::from_json(&tools.fetch(LATEST)?)?;
    if release.version <= installed {
        out.push_str(&format!("planrun {installed} is up to date\n"));
        return Ok(());
    }
    if check {
        out.push_str(&format!(
            "planrun {} is available (installed: {installed}); run planrun update\n",
            release.version
        ));
        return Ok(());
    }
    let Some(target) = TARGET else {
        return Err(UpdateError::new(
            "update-unsupported",
            "no release is built for this platform; build it from source (scripts/install.*)",
        ));
    };
    let asset = asset_for(&release, target)?;
    let exe = std::env::current_exe()
        .map_err(|e| UpdateError::new("update-swap", format!("the running executable: {e}")))?;
    let scratch = Scratch::create()?;
    let archive = scratch.0.join(&asset.name);
    tools.download(&asset.url, &archive)?;
    tools.extract(&archive, &scratch.0, target.archive)?;
    let binary = scratch
        .0
        .join(format!("planrun{}", std::env::consts::EXE_SUFFIX));
    verify(&binary, release.version)?;
    swap(&exe, &binary)?;
    out.push_str(&format!(
        "updated planrun {installed} → {}\n",
        release.version
    ));
    Ok(())
}

/// curl and tar. On Windows the ones in System32, by absolute path: an MSYS curl or GNU tar
/// earlier on `PATH` (Git Bash) must not run, and GNU tar does not read zip.
struct Tools {
    curl: PathBuf,
    tar: PathBuf,
}

impl Tools {
    fn find() -> Result<Tools, UpdateError> {
        if !cfg!(windows) {
            return Ok(Tools {
                curl: PathBuf::from("curl"),
                tar: PathBuf::from("tar"),
            });
        }
        let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
        let system = PathBuf::from(root).join("System32");
        let tools = Tools {
            curl: system.join("curl.exe"),
            tar: system.join("tar.exe"),
        };
        for tool in [&tools.curl, &tools.tar] {
            if !tool.is_file() {
                return Err(UpdateError::new(
                    "update-tools",
                    format!("update needs {} (Windows 10 1803 or later)", tool.display()),
                ));
            }
        }
        Ok(tools)
    }

    /// The body of `url`.
    fn fetch(&self, url: &str) -> Result<String, UpdateError> {
        let mut curl = Command::new(&self.curl);
        curl.args([
            "-fsSL",
            "-H",
            "Accept: application/vnd.github+json",
            "-A",
            "planrun",
        ])
        .arg(url);
        let stdout = output(curl, "update-fetch", &format!("fetching {url}"))?;
        String::from_utf8(stdout)
            .map_err(|e| UpdateError::new("update-fetch", format!("{url}: {e}")))
    }

    fn download(&self, url: &str, to: &Path) -> Result<(), UpdateError> {
        let mut curl = Command::new(&self.curl);
        curl.args(["-fsSL", "-A", "planrun", "-o"]).arg(to).arg(url);
        output(curl, "update-download", &format!("downloading {url}"))?;
        Ok(())
    }

    fn extract(&self, archive: &Path, into: &Path, kind: Archive) -> Result<(), UpdateError> {
        let mut tar = Command::new(&self.tar);
        // bsdtar (Windows) reads zip with -x; GNU tar needs -z for gzip.
        tar.arg(match kind {
            Archive::Zip => "-xf",
            Archive::TarGz => "-xzf",
        })
        .arg(archive)
        .arg("-C")
        .arg(into);
        output(
            tar,
            "update-download",
            &format!("unpacking {}", archive.display()),
        )?;
        Ok(())
    }
}

/// Runs `command`; its stdout when it exits 0. Not found is `update-tools`; any other failure
/// is `code`, with the command's stderr.
fn output(mut command: Command, code: &'static str, what: &str) -> Result<Vec<u8>, UpdateError> {
    let program = command.get_program().to_string_lossy().into_owned();
    let ran = command.output().map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            UpdateError::new("update-tools", format!("update needs `{program}` on PATH"))
        } else {
            UpdateError::new(code, format!("{what}: {program}: {e}"))
        }
    })?;
    if ran.status.success() {
        return Ok(ran.stdout);
    }
    let stderr = String::from_utf8_lossy(&ran.stderr);
    Err(UpdateError::new(
        code,
        format!("{what}: {program} {}: {}", ran.status, stderr.trim()),
    ))
}

/// The unpacked executable runs and reports `version`, before it replaces anything.
fn verify(binary: &Path, version: Version) -> Result<(), UpdateError> {
    let fail =
        |why: String| UpdateError::new("update-verify", format!("{}: {why}", binary.display()));
    let ran = Command::new(binary)
        .arg("--version")
        .output()
        .map_err(|e| fail(e.to_string()))?;
    let printed = String::from_utf8_lossy(&ran.stdout);
    if !ran.status.success() || !version_matches(&printed, version) {
        return Err(fail(format!(
            "`--version` printed `{}`, not {version}",
            printed.trim()
        )));
    }
    Ok(())
}

/// `planrun 0.2.0` names `version`; `planrun 0.2.01` does not.
fn version_matches(printed: &str, version: Version) -> bool {
    printed
        .split_whitespace()
        .any(|word| Version::parse(word) == Some(version))
}

/// A directory of its own under the OS temp directory, removed on drop, on every path.
struct Scratch(PathBuf);

impl Scratch {
    fn create() -> Result<Scratch, UpdateError> {
        let dir = std::env::temp_dir().join(format!("planrun-update-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)
            .map_err(|e| UpdateError::new("update-download", format!("{}: {e}", dir.display())))?;
        Ok(Scratch(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// [`cleanup`] beside the running executable; called at every start (D83).
pub fn cleanup_beside_current_exe() {
    if let Ok(exe) = std::env::current_exe() {
        cleanup(&exe);
    }
}

/// A release version: `major.minor.patch`, compared numerically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    /// `0.2.0` or `v0.2.0`; anything else (a pre-release, a fourth part) is `None`.
    pub fn parse(text: &str) -> Option<Version> {
        let text = text.strip_prefix('v').unwrap_or(text);
        let mut parts = text.split('.');
        let mut next = || parts.next().and_then(number);
        let version = Version {
            major: next()?,
            minor: next()?,
            patch: next()?,
        };
        if parts.next().is_some() {
            return None;
        }
        Some(version)
    }
}

/// A number of digits only: `+1`, `-1` and an empty part are not versions.
fn number(part: &str) -> Option<u64> {
    if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    part.parse().ok()
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Why an update did not happen. The installed executable is unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateError {
    /// A stable kebab-case code (`update-fetch`, …).
    pub code: &'static str,
    /// What went wrong, for the user.
    pub message: String,
}

impl UpdateError {
    fn new(code: &'static str, message: impl Into<String>) -> UpdateError {
        UpdateError {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// One file attached to a release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    /// The file name, `planrun-v0.2.0-x86_64-pc-windows-msvc.zip`.
    pub name: String,
    /// Where to download it.
    pub url: String,
}

/// The latest release, as GitHub's `releases/latest` describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// The tag as written, `v0.2.0`; the asset names carry it.
    pub tag: String,
    /// The tag's version.
    pub version: Version,
    /// The attached files.
    pub assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct RawRelease {
    tag_name: String,
    assets: Vec<RawAsset>,
}

#[derive(Deserialize)]
struct RawAsset {
    name: String,
    browser_download_url: String,
}

impl Release {
    /// Reads GitHub's release JSON; unknown fields are ignored. A tag that is not a version is
    /// an error, never "not newer".
    pub fn from_json(text: &str) -> Result<Release, UpdateError> {
        let raw: RawRelease = serde_json::from_str(text).map_err(|e| {
            UpdateError::new("update-fetch", format!("the release description: {e}"))
        })?;
        let Some(version) = Version::parse(&raw.tag_name) else {
            return Err(UpdateError::new(
                "update-fetch",
                format!(
                    "the latest release's tag `{}` is not a version",
                    raw.tag_name
                ),
            ));
        };
        let assets = raw
            .assets
            .into_iter()
            .map(|a| Asset {
                name: a.name,
                url: a.browser_download_url,
            })
            .collect();
        Ok(Release {
            tag: raw.tag_name,
            version,
            assets,
        })
    }
}

/// How a release archive is packed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Archive {
    /// `.zip` (Windows).
    Zip,
    /// `.tar.gz` (Linux).
    TarGz,
}

impl Archive {
    fn extension(self) -> &'static str {
        match self {
            Archive::Zip => "zip",
            Archive::TarGz => "tar.gz",
        }
    }
}

/// A platform the release workflow builds for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    /// The Rust target triple.
    pub triple: &'static str,
    /// The archive it is packed in.
    pub archive: Archive,
}

/// The target of this build, when the release workflow builds one for it.
pub const TARGET: Option<Target> = if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
    Some(Target {
        triple: "x86_64-pc-windows-msvc",
        archive: Archive::Zip,
    })
} else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
    Some(Target {
        triple: "x86_64-unknown-linux-gnu",
        archive: Archive::TarGz,
    })
} else {
    None
};

/// The name of `target`'s asset of the release tagged `tag`.
pub fn asset_name(tag: &str, target: Target) -> String {
    format!(
        "planrun-{tag}-{}.{}",
        target.triple,
        target.archive.extension()
    )
}

/// The asset of `release` for `target`.
pub fn asset_for(release: &Release, target: Target) -> Result<&Asset, UpdateError> {
    let name = asset_name(&release.tag, target);
    release
        .assets
        .iter()
        .find(|a| a.name == name)
        .ok_or_else(|| {
            UpdateError::new(
                "update-no-asset",
                format!("release {} has no `{name}`", release.tag),
            )
        })
}

/// `exe` with `.<what>` before its extension: `planrun.exe` → `planrun.old.exe`.
pub fn sibling(exe: &Path, what: &str) -> PathBuf {
    let stem = exe.file_stem().unwrap_or_default().to_string_lossy();
    let name = match exe.extension() {
        Some(ext) => format!("{stem}.{what}.{}", ext.to_string_lossy()),
        None => format!("{stem}.{what}"),
    };
    exe.with_file_name(name)
}

/// Installs `new_binary` as `exe`: copied beside it as `.new`, then renamed over it. On Windows
/// the running executable cannot be replaced, only renamed: it goes aside as `.old` first, and
/// comes back when the second rename fails. [`cleanup`] removes the `.old` at the next start.
pub fn swap(exe: &Path, new_binary: &Path) -> Result<(), UpdateError> {
    let staged = sibling(exe, "new");
    fs::copy(new_binary, &staged).map_err(|e| swap_error(&staged, &e))?;
    if let Err(e) = make_executable(&staged) {
        let _ = fs::remove_file(&staged);
        return Err(swap_error(&staged, &e));
    }
    let renamed = if cfg!(windows) {
        replace_aside(exe, &staged, |from, to| fs::rename(from, to))
    } else {
        replace_atomic(exe, &staged, |from, to| fs::rename(from, to))
    };
    renamed.map_err(|e| swap_error(exe, &e))
}

fn swap_error(path: &Path, e: &io::Error) -> UpdateError {
    UpdateError::new("update-swap", format!("{}: {e}", path.display()))
}

#[cfg(unix)]
fn make_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

/// One rename of `staged` over `exe` (Unix: atomic on one filesystem, and a running executable
/// may be replaced). On failure `staged` is removed.
fn replace_atomic(
    exe: &Path,
    staged: &Path,
    mut rename: impl FnMut(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    let result = rename(staged, exe);
    if result.is_err() {
        let _ = fs::remove_file(staged);
    }
    result
}

/// `exe` aside to `.old`, then `staged` to `exe`; when the second rename fails, `.old` goes back
/// and `staged` is removed (Windows).
fn replace_aside(
    exe: &Path,
    staged: &Path,
    mut rename: impl FnMut(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    let old = sibling(exe, "old");
    if let Err(e) = remove_if_present(&old) {
        let _ = fs::remove_file(staged);
        return Err(e);
    }
    if let Err(e) = rename(exe, &old) {
        let _ = fs::remove_file(staged);
        return Err(e);
    }
    let Err(e) = rename(staged, exe) else {
        return Ok(());
    };
    let _ = rename(&old, exe);
    let _ = fs::remove_file(staged);
    Err(e)
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// Removes what an update left beside `exe` (`.old`, `.new`), best effort: a file that cannot
/// be removed (the old executable still running) stays for the next start.
pub fn cleanup(exe: &Path) {
    let _ = fs::remove_file(sibling(exe, "old"));
    let _ = fs::remove_file(sibling(exe, "new"));
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
