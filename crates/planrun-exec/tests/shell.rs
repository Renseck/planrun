//! pwsh and bash sessions on a fixture copy (D60–D65): state across blocks, the wrapper's own
//! names, exit and throw, stdin isolation, the environment, the timeout's tree kill. A missing
//! shell skips its tests with a message, unless `PLANRUN_REQUIRE_SHELLS` is set.

#[path = "../../../tests/support/fixture.rs"]
mod fixture;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use fixture::Fixture;
use planrun_exec::ShellSession;
use planrun_exec::shell::{
    OutLine, RunStatus, ScriptRun, Session, SessionConfig, ShellError, Stream,
};
use planrun_model::Shell;

const EMPTY_BLOB: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";

/// A session in the fixture copy with its git isolation, or `None` (and a message) when the
/// shell is missing and shells are not required.
fn start(shell: Shell, f: &Fixture, timeout: Duration) -> Option<Session> {
    start_in(shell, f, timeout, f.outside())
}

fn start_in(shell: Shell, f: &Fixture, timeout: Duration, temp: PathBuf) -> Option<Session> {
    let config = SessionConfig {
        root: f.path().to_path_buf(),
        env: f.env(),
        timeout: Some(timeout),
        temp,
    };
    match Session::start(shell, &config) {
        Ok(session) => Some(session),
        Err(e @ ShellError::Unavailable { .. })
            if std::env::var_os("PLANRUN_REQUIRE_SHELLS").is_none() =>
        {
            eprintln!("skipped: {e}");
            None
        }
        Err(e) => panic!("{e}"),
    }
}

fn session(shell: Shell, f: &Fixture) -> Option<Session> {
    start(shell, f, Duration::from_secs(60))
}

fn run(s: &mut Session, script: &str) -> ScriptRun {
    s.run(script, &mut |_| {}).unwrap_or_else(|e| panic!("{e}"))
}

/// The run's stdout lines, without a trailing CR.
fn stdout(r: &ScriptRun) -> Vec<&str> {
    r.lines
        .iter()
        .filter(|l| l.stream == Stream::Stdout)
        .map(|l| l.text.trim_end_matches('\r'))
        .collect()
}

fn same_dir(a: &Path, b: &Path) -> bool {
    let canon =
        |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    canon(a) == canon(b)
}

#[test]
fn pwsh_keeps_variables_functions_and_the_working_directory() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    let first = run(
        &mut s,
        "$x = 41\nfunction F { 'f-called' }\nNew-Item -ItemType Directory sub | Out-Null\nSet-Location sub\n",
    );
    assert_eq!(first.status, RunStatus::Exited(0));
    let second = run(&mut s, "$x + 1\nF\n");
    assert_eq!(stdout(&second), ["42", "f-called"]);
    let cwd = second.cwd.unwrap_or_else(|| panic!("no cwd"));
    assert!(same_dir(&cwd, &f.path().join("sub")), "{}", cwd.display());
}

#[test]
fn pwsh_wrapper_names_do_not_touch_the_plan_s_and_a_throw_is_reset() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    let threw = run(
        &mut s,
        "$t = 'mine'; $s = 'own'; $e = 'kept'\nthrow 'boom'\n",
    );
    assert_eq!(threw.status, RunStatus::Threw);
    assert!(threw.errored);
    assert!(
        threw
            .lines
            .iter()
            .any(|l| l.stream == Stream::Stderr && l.text.contains("boom"))
    );
    let after = run(&mut s, "\"$t $s $e\"\n");
    assert_eq!(after.status, RunStatus::Exited(0));
    assert!(!after.errored);
    assert_eq!(stdout(&after), ["mine own kept"]);
}

#[test]
fn pwsh_exit_ends_the_block_only_and_lastexitcode_is_reset() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    assert_eq!(
        run(&mut s, "$v = 'kept'\nexit 7\n'not reached'\n").status,
        RunStatus::Exited(7)
    );
    let next = run(&mut s, "$v\n");
    assert_eq!(
        (next.status.clone(), stdout(&next)),
        (RunStatus::Exited(0), vec!["kept"])
    );
    let native = run(&mut s, "git rev-parse --verify --quiet refs/heads/nosuch\n");
    assert_eq!(native.status, RunStatus::Exited(1));
    assert_eq!(
        run(&mut s, "'no native command'\n").status,
        RunStatus::Exited(0)
    );
}

#[test]
fn pwsh_break_trap_and_a_parse_error_still_reach_the_sentinel() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    let broke = run(&mut s, "'before'\nbreak\n'after'\n");
    assert_eq!(stdout(&broke), ["before"]);
    let trapped = run(&mut s, "trap { 'trapped' }\nthrow 'x'\n'after the throw'\n");
    assert_eq!(stdout(&trapped), ["trapped", "after the throw"]);
    let parse = run(&mut s, "'start'\nif (\n");
    assert_eq!(parse.status, RunStatus::Threw);
    assert!(stdout(&parse).is_empty(), "a parse error runs nothing");
    assert_eq!(run(&mut s, "'alive'\n").status, RunStatus::Exited(0));
}

#[test]
fn pwsh_errored_follows_error_zero() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    let cases = [
        ("Get-Item nosuch -ErrorAction SilentlyContinue\n", true),
        ("Get-Item nosuch -ErrorAction Ignore\n", false),
        ("git rev-parse --verify nosuch 2>$null\n", false),
        ("$x = git rev-parse --verify nosuch 2>&1\n", false),
        ("Write-Error 'non-terminating'\n", true),
        ("'clean'\n", false),
    ];
    for (script, errored) in cases {
        let r = run(&mut s, script);
        assert_eq!(r.errored, errored, "{script:?}");
        assert!(!matches!(r.status, RunStatus::Threw), "{script:?}");
    }
}

#[test]
fn pwsh_output_order_partial_lines_and_unicode() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    let order = run(&mut s, "Write-Output a; git --version; Write-Output b\n");
    let out = stdout(&order);
    assert_eq!(out.len(), 3, "{out:?}");
    assert_eq!((out[0], out[2]), ("a", "b"));
    assert!(out[1].starts_with("git version"), "{out:?}");
    let mut seen = Vec::new();
    let partial = s
        .run(
            "'René ✓'\n[Console]::Out.Write('tail')\n",
            &mut |l: &OutLine| seen.push(l.clone()),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(seen, partial.lines, "every line is streamed");
    assert_eq!(stdout(&partial), ["René ✓", "tail"]);
    assert!(!partial.lines[1].newline);
}

#[test]
fn pwsh_and_bash_blocks_get_no_stdin() {
    let f = Fixture::greeter(false);
    let script = "git hash-object --stdin\n";
    for shell in [Shell::Pwsh, Shell::Bash] {
        let Some(mut s) = session(shell, &f) else {
            continue;
        };
        let r = run(&mut s, script);
        assert_eq!(stdout(&r), [EMPTY_BLOB], "{shell:?}");
        let cat = if shell == Shell::Bash {
            "cat; echo next\n"
        } else {
            "'next'\n"
        };
        assert_eq!(stdout(&run(&mut s, cat)), ["next"], "{shell:?}");
    }
}

#[test]
fn sessions_get_planrun_s_environment_without_the_git_wrapper_s() {
    let f = Fixture::greeter(false);
    let pwsh = "foreach ($n in 'GIT_LITERAL_PATHSPECS', 'GIT_INDEX_FILE', 'GIT_OBJECT_DIRECTORY', \
                'GIT_ALTERNATE_OBJECT_DIRECTORIES', 'BASH_ENV', 'ENV', 'PLANRUN_SESSION_DIR') \
                { if (Test-Path \"Env:$n\") { \"set: $n\" } }\n\"prompt=$env:GIT_TERMINAL_PROMPT\"\n\
                git config --show-origin --list\n";
    let bash = "for n in GIT_LITERAL_PATHSPECS GIT_INDEX_FILE GIT_OBJECT_DIRECTORY \
                GIT_ALTERNATE_OBJECT_DIRECTORIES BASH_ENV ENV PLANRUN_SESSION_DIR; do \
                [ -n \"${!n+x}\" ] && echo \"set: $n\"; done; echo \"prompt=$GIT_TERMINAL_PROMPT\"\n\
                git config --show-origin --list\n";
    for (shell, script) in [(Shell::Pwsh, pwsh), (Shell::Bash, bash)] {
        let Some(mut s) = session(shell, &f) else {
            continue;
        };
        let r = run(&mut s, script);
        let out = stdout(&r);
        assert_eq!(out.first().copied(), Some("prompt=0"), "{shell:?}: {out:?}");
        assert!(out.len() > 1, "{shell:?}: no configuration listed");
        for line in &out[1..] {
            assert!(line.starts_with("file:.git/config\t"), "{shell:?}: {line}");
        }
    }
}

#[test]
fn bash_keeps_declared_variables_and_functions_and_has_no_arguments() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Bash, &f) else {
        return;
    };
    let first = run(
        &mut s,
        "declare -A m=([k]=v)\ny=5\ng() { echo g-called; }\nmkdir -p sub && cd sub\n",
    );
    assert_eq!(first.status, RunStatus::Exited(0));
    let second = run(&mut s, "echo \"${m[k]} $y\"\ng\necho \"[$1][$#]\"\n");
    assert_eq!(stdout(&second), ["v 5", "g-called", "[][0]"]);
    let cwd = second.cwd.unwrap_or_else(|| panic!("no cwd"));
    assert!(same_dir(&cwd, &f.path().join("sub")), "{}", cwd.display());
}

#[test]
fn bash_wrapper_names_do_not_touch_the_plan_s_and_status_is_the_last_command_s() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Bash, &f) else {
        return;
    };
    assert_eq!(
        run(&mut s, "s=mine; t=own; n=kept\nfalse\n").status,
        RunStatus::Exited(1)
    );
    let after = run(&mut s, "echo \"$s $t $n\"\n");
    assert_eq!(after.status, RunStatus::Exited(0));
    assert_eq!(stdout(&after), ["mine own kept"]);
    let syntax = run(&mut s, "echo before\nif then fi\n");
    assert_eq!(
        syntax.status,
        RunStatus::Exited(2),
        "a syntax error ends the block only"
    );
    assert_eq!(stdout(&run(&mut s, "echo alive\n")), ["alive"]);
}

#[test]
fn bash_exit_set_e_a_replaced_trap_and_exec_end_the_session() {
    let f = Fixture::greeter(false);
    let cases: [(&str, RunStatus); 4] = [
        ("echo bye\nexit 3\n", RunStatus::SessionEnded(Some(3))),
        ("set -e\n", RunStatus::SessionEnded(Some(1))),
        ("trap - EXIT\nexit 4\n", RunStatus::SessionEnded(None)),
        ("exec true\n", RunStatus::SessionEnded(None)),
    ];
    for (script, status) in cases {
        let Some(mut s) = session(Shell::Bash, &f) else {
            return;
        };
        let mut r = run(&mut s, script);
        if script.starts_with("set -e") {
            assert_eq!(r.status, RunStatus::Exited(0));
            r = run(&mut s, "false\n");
        }
        assert_eq!(r.status, status, "{script:?}");
        assert!(!s.is_alive(), "{script:?}");
        assert!(s.run("echo x\n", &mut |_| {}).is_err(), "{script:?}");
    }
}

#[test]
fn a_shell_that_exits_while_a_descendant_holds_its_pipes_ends_the_session() {
    let f = Fixture::greeter(false);
    let pwsh = "$null = [Diagnostics.Process]::Start((Get-Process -Id $PID).Path, '-NoProfile -Command Start-Sleep 120')\n[Environment]::Exit(0)\n";
    let bash = "sleep 120 &\ntrap - EXIT\nexit 0\n";
    for (shell, script) in [(Shell::Pwsh, pwsh), (Shell::Bash, bash)] {
        let Some(mut s) = start(shell, &f, Duration::from_secs(60)) else {
            continue;
        };
        let started = Instant::now();
        let r = run(&mut s, script);
        assert_eq!(r.status, RunStatus::SessionEnded(None), "{shell:?}");
        assert!(started.elapsed() < Duration::from_secs(30), "{shell:?}");
        assert!(!s.is_alive());
    }
}

#[test]
fn bash_plan_functions_named_like_the_wrapper_s_commands_change_nothing() {
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Bash, &f) else {
        return;
    };
    let define = "source() { echo hijacked; }\nlocal() { echo hijacked; }\nprintf() { echo hijacked; }\npwd() { echo hijacked; }\nv=kept\n";
    assert_eq!(run(&mut s, define).status, RunStatus::Exited(0));
    let next = run(&mut s, "echo \"$v\"\n");
    assert_eq!(next.status, RunStatus::Exited(0));
    assert_eq!(stdout(&next), ["kept"]);
    assert!(next.cwd.is_some_and(|c| same_dir(&c, f.path())));
}

#[test]
fn a_block_that_cannot_be_written_ends_the_session() {
    let f = Fixture::greeter(false);
    let temp = f.outside().join("runs");
    std::fs::create_dir_all(&temp).unwrap_or_else(|e| panic!("{e}"));
    for shell in [Shell::Pwsh, Shell::Bash] {
        let Some(mut s) = start_in(shell, &f, Duration::from_secs(60), temp.clone()) else {
            continue;
        };
        for entry in std::fs::read_dir(&temp)
            .unwrap_or_else(|e| panic!("{e}"))
            .flatten()
        {
            std::fs::remove_dir_all(entry.path()).unwrap_or_else(|e| panic!("{e}"));
        }
        assert!(s.run("echo x\n", &mut |_| {}).is_err(), "{shell:?}");
        assert!(!s.is_alive(), "{shell:?}");
    }
}

#[test]
fn a_forged_sentinel_breaks_the_session() {
    let f = Fixture::greeter(false);
    let pwsh =
        "$m = ([regex]'__planrun_[0-9a-f]+_').Match(${function:__planrun_end}.ToString()).Value\n";
    let cases = [
        (Shell::Pwsh, format!("{pwsh}[Console]::Out.WriteLine($m + '99__ ok 0 0 ')\n")),
        (Shell::Pwsh, format!("{pwsh}[Console]::Out.WriteLine($m + '1__ ok 0 0  extra')\n")),
        (
            Shell::Bash,
            "m=$(declare -f __planrun_end | grep -o '__planrun_[0-9a-f]*_' | head -1)\necho \"${m}1__ ok 0 0 zz\"\n"
                .to_owned(),
        ),
    ];
    for (shell, script) in cases {
        let Some(mut s) = session(shell, &f) else {
            continue;
        };
        let r = run(&mut s, &script);
        assert!(
            matches!(r.status, RunStatus::Broken(_)),
            "{shell:?}: {:?}",
            r.status
        );
        assert!(!s.is_alive());
    }
}

/// Whether the process `pid` still runs.
fn alive(pid: &str) -> bool {
    let out = if cfg!(windows) {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
    } else {
        Command::new("kill").args(["-0", pid]).output()
    }
    .unwrap_or_else(|e| panic!("{e}"));
    if cfg!(windows) {
        return String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .any(|w| w == pid);
    }
    out.status.success()
}

#[test]
fn a_timeout_kills_the_whole_tree() {
    let f = Fixture::greeter(false);
    let pid_file = f.outside().join("pid");
    let file = pid_file.display().to_string();
    let pwsh = format!(
        "$p = Start-Process pwsh -ArgumentList '-NoProfile', '-Command', 'Start-Sleep 300' -PassThru\n\
         Set-Content -LiteralPath '{file}' -Value $p.Id\nStart-Sleep 300\n"
    );
    let bash = format!(
        "sleep 300 &\nif [ -r /proc/$!/winpid ]; then cat /proc/$!/winpid; else echo $!; fi > '{}'\nsleep 300\n",
        file.replace('\\', "/")
    );
    for (shell, script) in [(Shell::Pwsh, pwsh), (Shell::Bash, bash)] {
        let Some(mut s) = start(shell, &f, Duration::from_secs(5)) else {
            continue;
        };
        let r = run(&mut s, &script);
        assert_eq!(r.status, RunStatus::TimedOut, "{shell:?}");
        assert!(!s.is_alive());
        let pid = std::fs::read_to_string(&pid_file).unwrap_or_else(|e| panic!("{shell:?}: {e}"));
        let pid = pid.trim().to_owned();
        let until = Instant::now() + Duration::from_secs(10);
        while alive(&pid) && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(!alive(&pid), "{shell:?}: the grandchild {pid} survived");
        let _ = std::fs::remove_file(&pid_file);
    }
}

/// Set in the child process of [`end_now_kills_every_session_tree`]: the fixture's root, the
/// directory for the session's run directory, the file for the grandchild's pid, and the exit
/// code, separated by `|`.
const END_NOW_CHILD: &str = "PLANRUN_TEST_END_NOW_CHILD";

/// Runs only as the child of [`end_now_kills_every_session_tree`]: starts a pwsh session whose
/// block starts a grandchild and sleeps, then calls `end_now` while the block runs. In every
/// other run it returns at once and counts as one passed test.
#[test]
fn end_now_child() {
    let Some(spec) = std::env::var_os(END_NOW_CHILD) else {
        return;
    };
    let spec = spec.to_string_lossy().into_owned();
    let [root, temp, pid_file, code] = spec.split('|').collect::<Vec<_>>()[..] else {
        panic!("{END_NOW_CHILD}: {spec}");
    };
    let config = SessionConfig {
        root: PathBuf::from(root),
        env: Vec::new(),
        timeout: None,
        temp: PathBuf::from(temp),
    };
    let mut s = Session::start(Shell::Pwsh, &config).unwrap_or_else(|e| panic!("{e}"));
    let script = format!(
        "$p = Start-Process pwsh -ArgumentList '-NoProfile', '-Command', 'Start-Sleep 300' -PassThru\n\
         Set-Content -LiteralPath '{pid_file}' -Value $p.Id\nStart-Sleep 300\n"
    );
    std::thread::spawn(move || {
        let _ = s.run(&script, &mut |_| {});
    });
    let until = Instant::now() + Duration::from_secs(60);
    while !Path::new(pid_file).exists() && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(50));
    }
    std::thread::sleep(Duration::from_millis(200));
    planrun_exec::shell::end_now(code.parse().unwrap_or_else(|e| panic!("{e}")));
}

#[test]
fn end_now_kills_every_session_tree() {
    if std::env::var_os(END_NOW_CHILD).is_some() {
        return;
    }
    let f = Fixture::greeter(false);
    let Some(probe) = session(Shell::Pwsh, &f) else {
        return;
    };
    drop(probe);
    let pid_file = f.outside().join("end-now-pid");
    let spec = format!(
        "{}|{}|{}|7",
        f.path().display(),
        f.outside().display(),
        pid_file.display()
    );
    let exe = std::env::current_exe().unwrap_or_else(|e| panic!("{e}"));
    let out = Command::new(exe)
        .args([
            "end_now_child",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(END_NOW_CHILD, &spec)
        .output()
        .unwrap_or_else(|e| panic!("{e}"));
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.status.code(), Some(7), "{report}");
    let pid = std::fs::read_to_string(&pid_file).unwrap_or_else(|e| panic!("{e}: {report}"));
    let pid = pid.trim().to_owned();
    let until = Instant::now() + Duration::from_secs(10);
    while alive(&pid) && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(!alive(&pid), "the grandchild {pid} survived end_now");
}

#[cfg(windows)]
#[test]
fn a_session_leaves_the_terminal_s_code_page_alone() {
    let chcp = || {
        let out = Command::new("cmd").args(["/c", "chcp"]).output();
        String::from_utf8_lossy(&out.unwrap_or_else(|e| panic!("{e}")).stdout).into_owned()
    };
    let before = chcp();
    let f = Fixture::greeter(false);
    let Some(mut s) = session(Shell::Pwsh, &f) else {
        return;
    };
    assert_eq!(run(&mut s, "'x'\n").status, RunStatus::Exited(0));
    drop(s);
    assert_eq!(chcp(), before);
}

#[test]
fn a_run_directory_with_non_ascii_spaces_and_quotes_works_and_is_removed() {
    let f = Fixture::greeter(false);
    let temp = f.outside().join("René 'x' ✓");
    std::fs::create_dir_all(&temp).unwrap_or_else(|e| panic!("{e}"));
    for shell in [Shell::Pwsh, Shell::Bash] {
        let Some(mut s) = start_in(shell, &f, Duration::from_secs(60), temp.clone()) else {
            continue;
        };
        let r = run(
            &mut s, "echo ok
",
        );
        assert_eq!(
            (r.status.clone(), stdout(&r)),
            (RunStatus::Exited(0), vec!["ok"]),
            "{shell:?}"
        );
        drop(s);
        let left = std::fs::read_dir(&temp)
            .unwrap_or_else(|e| panic!("{e}"))
            .count();
        assert_eq!(left, 0, "{shell:?}: the run directory stayed");
    }
}
