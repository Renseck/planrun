//! PowerShell 7 (decisions D60, D61, D63). A native command started by pwsh inherits pwsh's
//! own stdin, so the session gets none (`git hash-object --stdin` would read the protocol):
//! its `-Command` script is the prologue and a loop at the top level that waits for the next
//! block file in the run directory and dot-sources it inside `do`/`try`/`catch`.

use std::path::PathBuf;
use std::process::Command;

use super::ShellError;
use super::kill::detach;
use super::session::{DIR_VAR, Dialect};

/// The pwsh dialect.
pub(super) struct Pwsh;

/// `pwsh` from `PATH`, when it is PowerShell 7 or later; Windows PowerShell 5.1 is
/// `powershell.exe` and never used.
pub(super) fn locate() -> Result<PathBuf, ShellError> {
    let unavailable = |reason: String| ShellError::Unavailable {
        shell: "pwsh",
        reason,
    };
    let mut probe = Command::new("pwsh");
    probe.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$PSVersionTable.PSVersion.Major",
    ]);
    detach(&mut probe);
    let out = probe
        .output()
        .map_err(|e| unavailable(format!("`pwsh` did not start: {e}")))?;
    let major = String::from_utf8_lossy(&out.stdout).trim().parse::<u32>();
    match major {
        Ok(major) if out.status.success() && major >= 7 => Ok(PathBuf::from("pwsh")),
        Ok(major) => Err(unavailable(format!("PowerShell {major} is older than 7"))),
        Err(_) => Err(unavailable("`pwsh` did not report its version".to_owned())),
    }
}

impl Dialect for Pwsh {
    fn name(&self) -> &'static str {
        "pwsh"
    }

    fn args(&self, marker: &str) -> Vec<String> {
        let mut args = vec!["-NoProfile", "-NonInteractive", "-NoLogo"];
        if cfg!(windows) {
            args.extend(["-ExecutionPolicy", "RemoteSigned"]);
        }
        args.push("-Command");
        let mut args: Vec<String> = args.into_iter().map(str::to_owned).collect();
        args.push(script(marker));
        args
    }

    fn extension(&self) -> &'static str {
        "ps1"
    }

    fn prologue(&self, _marker: &str) -> Option<String> {
        None
    }

    fn block_line(&self, _n: u64) -> Option<String> {
        None
    }

    fn cwd(&self, reported: String) -> PathBuf {
        PathBuf::from(reported)
    }
}

/// The session's whole `-Command` script: the prologue, the sentinel of block 0, then the loop.
/// It has no double quotes, so the command line needs no escaping beyond Windows' own.
fn script(marker: &str) -> String {
    [
        format!("$global:__planrun_dir = $env:{DIR_VAR}"),
        format!("Remove-Item Env:{DIR_VAR}"),
        "if ($PSStyle) { $PSStyle.OutputRendering = 'PlainText' }".to_owned(),
        "$global:ProgressPreference = 'SilentlyContinue'".to_owned(),
        "$global:ErrorActionPreference = 'Continue'".to_owned(),
        "$global:PSNativeCommandUseErrorActionPreference = $false".to_owned(),
        "[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)".to_owned(),
        "$global:OutputEncoding = [Text.UTF8Encoding]::new($false)".to_owned(),
        end_function(marker),
        "$global:__planrun_n = 0".to_owned(),
        "$global:__planrun_w = [IO.FileSystemWatcher]::new($global:__planrun_dir, 'b*.ps1')"
            .to_owned(),
        RESET.to_owned(),
        "__planrun_end 0".to_owned(),
        run_loop(),
    ]
    .join("; ")
}

/// Resets every wrapper variable and `$LASTEXITCODE` before a block (D61).
const RESET: &str =
    "$global:__planrun_t = 0; $global:__planrun_e = $global:Error[0]; $global:LASTEXITCODE = 0";

/// Waits for `b<n>.ps1` (renamed into place whole), runs it in the global scope, and prints its
/// sentinel. `do … while ($false)` keeps a `break` or `continue` of the block inside it.
fn run_loop() -> String {
    format!(
        "while ($true) {{ $global:__planrun_n++; \
         $global:__planrun_f = [IO.Path]::Combine($global:__planrun_dir, 'b' + $global:__planrun_n + '.ps1'); \
         while (-not [IO.File]::Exists($global:__planrun_f)) {{ \
         [void]$global:__planrun_w.WaitForChanged([IO.WatcherChangeTypes]::Renamed, 250) }}; \
         {RESET}; \
         do {{ try {{ . $global:__planrun_f }} catch {{ $global:__planrun_t = 1; \
         [Console]::Error.WriteLine(($_ | Out-String).TrimEnd()) }} }} while ($false); \
         __planrun_end $global:__planrun_n }}"
    )
}

/// `__planrun_end <n>`: the sentinel on stdout (kind, status, errored, cwd as hex of UTF-8) and
/// on stderr, each written with LF and flushed.
fn end_function(marker: &str) -> String {
    format!(
        "function global:__planrun_end([long]$n) {{ \
         $s = $global:LASTEXITCODE; if ($null -eq $s) {{ $s = 0 }}; \
         $k = if ($global:__planrun_t) {{ 'threw' }} else {{ 'ok' }}; \
         $e = if ([object]::ReferenceEquals($global:__planrun_e, $global:Error[0])) {{ 0 }} else {{ 1 }}; \
         $c = [BitConverter]::ToString([Text.Encoding]::UTF8.GetBytes($PWD.ProviderPath)).Replace('-', ''); \
         $m = '{marker}' + $n + '__'; \
         [Console]::Out.Write($m + ' ' + $k + ' ' + $s + ' ' + $e + ' ' + $c + [char]10); [Console]::Out.Flush(); \
         [Console]::Error.Write($m + [char]10); [Console]::Error.Flush() }}"
    )
}
