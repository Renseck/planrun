//! bash (decisions D60, D61, D63): `bash --noprofile --norc`, one top-level line per block on
//! stdin, the block `source`d from its file with stdin from `/dev/null`. On Windows, Git for
//! Windows' bash only, found from git itself, never from `PATH`.

use std::path::PathBuf;
use std::process::Command;

use super::ShellError;
use super::kill::detach;
use super::session::{DIR_VAR, Dialect};

/// The bash dialect.
pub(super) struct Bash;

fn unavailable(reason: String) -> ShellError {
    ShellError::Unavailable {
        shell: "bash",
        reason,
    }
}

/// Windows: `<root>\usr\bin\bash.exe`, where `<root>` is the first of `git --exec-path` and its
/// parents (at most four levels up) that has one; checked with `uname -s`, which must report
/// MINGW or MSYS. Elsewhere: `bash` from `PATH`, checked with `$BASH_VERSION`.
pub(super) fn locate() -> Result<PathBuf, ShellError> {
    let program = if cfg!(windows) {
        git_bash()?
    } else {
        PathBuf::from("bash")
    };
    let check = if cfg!(windows) {
        "PATH=/usr/bin:$PATH uname -s"
    } else {
        "echo \"$BASH_VERSION\""
    };
    let mut probe = Command::new(&program);
    probe.args(["--noprofile", "--norc", "-c", check]);
    detach(&mut probe);
    let out = probe
        .output()
        .map_err(|e| unavailable(format!("{} did not start: {e}", program.display())))?;
    let said = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let accepted = if cfg!(windows) {
        said.starts_with("MINGW") || said.starts_with("MSYS")
    } else {
        !said.is_empty()
    };
    if !out.status.success() || !accepted {
        return Err(unavailable(format!(
            "{} answered {said:?} to `{check}`",
            program.display()
        )));
    }
    Ok(program)
}

fn git_bash() -> Result<PathBuf, ShellError> {
    let out = Command::new("git")
        .arg("--exec-path")
        .output()
        .map_err(|e| unavailable(format!("`git --exec-path` did not run: {e}")))?;
    let exec_path = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    exec_path
        .ancestors()
        .take(5)
        .map(|dir| dir.join("usr").join("bin").join("bash.exe"))
        .find(|bash| bash.is_file())
        .ok_or_else(|| {
            unavailable(format!(
                "no usr\\bin\\bash.exe at or above {}",
                exec_path.display()
            ))
        })
}

impl Dialect for Bash {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn args(&self, _marker: &str) -> Vec<String> {
        vec!["--noprofile".to_owned(), "--norc".to_owned()]
    }

    fn extension(&self) -> &'static str {
        "sh"
    }

    /// On Windows `/usr/bin` is prepended to `PATH`, as `/etc/profile` would (only that
    /// directory: `git` keeps resolving through the Windows `PATH`, as planrun's own does), and
    /// the working directory is `pwd -W`: `$PWD` is an MSYS path, and `/tmp` is a mount.
    fn prologue(&self, marker: &str) -> Option<String> {
        let (path, pwd) = if cfg!(windows) {
            ("PATH=/usr/bin:$PATH; ", "$(builtin pwd -W)")
        } else {
            ("", "$PWD")
        };
        Some(format!(
            "__planrun_dir=${DIR_VAR}; unset {DIR_VAR}; {path}\
             __planrun_hex() {{ builtin local LC_ALL=C __planrun_i __planrun_c; __planrun_h=; \
             for ((__planrun_i = 0; __planrun_i < ${{#1}}; __planrun_i++)); do \
             builtin printf -v __planrun_c '%02x' \"'${{1:__planrun_i:1}}\"; \
             __planrun_h+=$__planrun_c; done; }}; \
             __planrun_end() {{ __planrun_hex \"{pwd}\"; \
             builtin printf '%s\\n' \"{marker}${{__planrun_n}}__ $1 $__planrun_s 0 $__planrun_h\"; \
             builtin printf '%s\\n' \"{marker}${{__planrun_n}}__\" >&2; }}; \
             trap '__planrun_s=$?; __planrun_end exit' EXIT; \
             __planrun_n=0; __planrun_s=0; __planrun_end ok"
        ))
    }

    fn block_line(&self, n: u64) -> Option<String> {
        Some(format!(
            "__planrun_n={n}; builtin source \"$__planrun_dir/b{n}.sh\" </dev/null; __planrun_s=$?; \
             __planrun_end ok"
        ))
    }

    /// On Windows `pwd -W` reports `C:/x`: its slashes become backslashes.
    fn cwd(&self, reported: String) -> PathBuf {
        if cfg!(windows) {
            return PathBuf::from(reported.replace('/', "\\"));
        }
        PathBuf::from(reported)
    }
}
