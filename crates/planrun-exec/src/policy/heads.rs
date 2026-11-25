//! The command tables (decision D68): which command, with which arguments, is which class.
//! A command that is not here is Destructive. Build may fetch dependencies into caches and
//! ignored directories; Modify changes tracked files, manifests or lockfiles.

use super::PolicyClass::{self, Build, Destructive, Modify, Read};
use super::Word;

/// A class, with the reason when it is Destructive.
pub(super) type Verdict = (PolicyClass, Option<String>);

fn ok(class: PolicyClass) -> Verdict {
    (class, None)
}

fn no(reason: impl Into<String>) -> Verdict {
    (Destructive, Some(reason.into()))
}

/// The DevReady version the pwsh list below was taken from (its agent modes).
pub const DEVREADY_VERSION: &str = "1.19.1";

/// DevReady functions whose agent mode is `safe`: Read.
#[rustfmt::skip]
const DEVREADY_SAFE: &[&str] = &[
    "brief", "decode", "deps", "depsaudit", "depscheck", "depsoutdated", "depswhy", "dimages",
    "dps", "dstats", "encode", "functions", "gitinfo", "json2yaml", "jsonvalidate", "lines",
    "logerrors", "logfilter", "logsearch", "logstats", "markers", "netdns", "nettest", "ports",
    "test-devreadyplan", "tfchanges", "tfdrift", "tferrors", "tfparse", "tfrisk", "tfshow",
    "tfsummary", "todos", "unixtime", "urlinfo", "what", "yaml2json", "yamlvalidate",
];

/// DevReady functions whose mode is `dry-run-only` or `interactive`, and `killport` (safe for
/// DevReady, but it kills a process): Destructive.
#[rustfmt::skip]
const DEVREADY_DESTRUCTIVE: &[&str] = &[
    "dbconnect", "depsclean", "dprune", "gbackup", "gprune", "gtag", "killport", "mergeback",
    "newgo", "newpython", "newrust", "note", "notify", "purge", "rewind", "secrets",
];

/// pwsh aliases, resolved before the table.
#[rustfmt::skip]
const PWSH_ALIASES: &[(&str, &str)] = &[
    ("%", "foreach-object"), ("?", "where-object"), ("ac", "add-content"),
    ("cat", "get-content"), ("cd", "set-location"), ("chdir", "set-location"),
    ("clc", "clear-content"), ("clear", "clear-host"), ("cls", "clear-host"),
    ("compare", "compare-object"), ("copy", "copy-item"), ("cp", "copy-item"),
    ("cpi", "copy-item"), ("del", "remove-item"), ("dir", "get-childitem"),
    ("echo", "write-output"), ("erase", "remove-item"), ("fl", "format-list"),
    ("foreach", "foreach-object"), ("ft", "format-table"), ("fw", "format-wide"),
    ("gc", "get-content"), ("gci", "get-childitem"), ("gcm", "get-command"),
    ("gi", "get-item"), ("gl", "get-location"), ("gm", "get-member"),
    ("gp", "get-itemproperty"), ("group", "group-object"), ("gv", "get-variable"),
    ("iex", "invoke-expression"), ("irm", "invoke-restmethod"), ("iwr", "invoke-webrequest"),
    ("ls", "get-childitem"), ("md", "mkdir"), ("measure", "measure-object"),
    ("mi", "move-item"), ("move", "move-item"), ("mv", "move-item"), ("ni", "new-item"),
    ("popd", "pop-location"), ("pushd", "push-location"), ("pwd", "get-location"),
    ("rd", "remove-item"), ("ren", "rename-item"), ("ri", "remove-item"),
    ("rm", "remove-item"), ("rmdir", "remove-item"), ("rni", "rename-item"),
    ("rv", "remove-variable"), ("saps", "start-process"), ("select", "select-object"),
    ("set", "set-variable"), ("sl", "set-location"), ("sleep", "start-sleep"),
    ("sort", "sort-object"), ("start", "start-process"), ("sv", "set-variable"),
    ("tee", "tee-object"), ("type", "get-content"), ("where", "where-object"),
    ("write", "write-output"),
];

/// pwsh cmdlets and functions that only read, print or change the session's own state.
#[rustfmt::skip]
const PWSH_READ: &[&str] = &[
    "add-member", "clear-host", "clear-variable", "compare-object", "convertfrom-csv",
    "convertfrom-json", "convertfrom-stringdata", "convertto-csv", "convertto-json",
    "format-custom", "format-list", "format-table", "format-wide", "get-alias",
    "get-childitem", "get-command", "get-content", "get-date", "get-filehash", "get-help",
    "get-item", "get-itemproperty", "get-location", "get-member", "get-module", "get-process",
    "get-random", "get-unique", "get-variable", "group-object", "join-path", "join-string",
    "measure-object", "new-guid", "new-timespan", "new-variable", "out-default", "out-host",
    "out-null", "out-string", "pop-location", "push-location", "remove-variable",
    "resolve-path", "select-object", "select-string", "select-xml", "set-location",
    "set-variable", "sort-object", "split-path", "start-sleep", "test-json", "test-path",
    "where-object", "write-debug", "write-error", "write-host", "write-information",
    "write-output", "write-progress", "write-verbose", "write-warning",
];

/// pwsh cmdlets that write files in place.
#[rustfmt::skip]
const PWSH_MODIFY: &[&str] = &[
    "add-content", "clear-content", "copy-item", "mkdir", "move-item", "out-file",
    "rename-item", "set-content",
];

/// bash commands that only read or print, or change the session's own state.
#[rustfmt::skip]
const BASH_READ: &[&str] = &[
    "[", "basename", "cat", "cd", "cmp", "column", "comm", "cut", "date", "declare", "df",
    "diff", "dirname", "dirs", "du", "echo", "egrep", "exit", "export", "false", "fgrep",
    "file", "grep", "head", "hostname", "id", "jq", "local", "ls", "md5sum", "nl", "od",
    "paste", "popd", "printf", "pushd", "pwd", "read", "readlink", "readonly", "realpath",
    "return", "rev", "seq", "set", "sha1sum", "sha256sum", "shift", "shopt", "sleep", "stat",
    "tac", "tail", "test", "tr", "true", "type", "typeset", "uname", "uniq", "unset", "wait",
    "wc", "which", "whoami", "xxd",
];

/// bash commands that write files in place.
const BASH_MODIFY: &[&str] = &["chmod", "cp", "mkdir", "mv", "rmdir", "tee", "touch"];

/// A pwsh command by its name (lowercase, `.exe`/`.cmd` already dropped).
pub(super) fn pwsh(name: &str, args: &[Word]) -> Verdict {
    let name = PWSH_ALIASES
        .iter()
        .find(|(alias, _)| *alias == name)
        .map_or(name, |(_, cmdlet)| cmdlet);
    if let Some(verdict) = tool(name, args) {
        return verdict;
    }
    if PWSH_READ.contains(&name) || DEVREADY_SAFE.contains(&name) {
        return ok(Read);
    }
    if PWSH_MODIFY.contains(&name) {
        return ok(Modify);
    }
    match name {
        "remove-item" => switched(args, Modify, &["recurse"], "Remove-Item -Recurse"),
        "new-item" => new_item(args),
        "tee-object" => tee_object(args),
        "foreach-object" => foreach_object(args),
        "replace" => replace(args),
        "test" | "testfailed" => ok(Build),
        _ if DEVREADY_DESTRUCTIVE.contains(&name) => no(format!("`{name}`")),
        _ => no(format!("unknown command `{name}`")),
    }
}

/// A bash command by its name.
pub(super) fn bash(name: &str, args: &[Word]) -> Verdict {
    if let Some(verdict) = tool(name, args) {
        return verdict;
    }
    if BASH_READ.contains(&name) {
        return ok(Read);
    }
    if BASH_MODIFY.contains(&name) {
        return ok(Modify);
    }
    match name {
        "rm" if args
            .iter()
            .any(|a| short(a, &['r', 'R']) || long(a, "--recursive")) =>
        {
            no("rm -r")
        }
        "rm" => ok(Modify),
        "sed"
            if args
                .iter()
                .any(|a| short(a, &['i']) || long(a, "--in-place")) =>
        {
            ok(Modify)
        }
        "sed" => ok(Read),
        "sort" if args.iter().any(|a| short(a, &['o']) || long(a, "--output")) => ok(Modify),
        "sort" => ok(Read),
        "find" => find(args),
        "command"
            if args
                .first()
                .and_then(Word::bare)
                .is_some_and(|a| a == "-v" || a == "-V") =>
        {
            ok(Read)
        }
        "env" if args.is_empty() => ok(Read),
        _ => no(format!("unknown command `{name}`")),
    }
}

/// The tools both shells share.
fn tool(name: &str, args: &[Word]) -> Option<Verdict> {
    let verdict = match name {
        "git" => git(args),
        "cargo" => cargo(args),
        "npm" => npm(args),
        "dotnet" => dotnet(args),
        "go" => go(args),
        "rg" if args.iter().any(|a| long(a, "--pre")) => no("rg --pre"),
        "rg" | "jq" => ok(Read),
        _ => return None,
    };
    Some(verdict)
}

/// A bash short-option cluster (`-rf`) that holds one of `letters`.
fn short(arg: &Word, letters: &[char]) -> bool {
    arg.bare().is_some_and(|a| {
        a.starts_with('-') && !a.starts_with("--") && a[1..].chars().any(|c| letters.contains(&c))
    })
}

/// `--name` or `--name=…`.
fn long(arg: &Word, name: &str) -> bool {
    arg.bare()
        .is_some_and(|a| a == name || a.strip_prefix(name).is_some_and(|r| r.starts_with('=')))
}

/// A pwsh parameter matches `name` as pwsh matches it: any prefix, any case, with or without
/// `:value` (`-r`, `-Rec`, `-Recurse:$true`).
fn param(arg: &Word, name: &str) -> bool {
    if arg.quoted {
        return false;
    }
    let Some(given) = arg.text.strip_prefix('-') else {
        return false;
    };
    let (given, value) = given.split_once(':').unwrap_or((given, ""));
    let given = given.to_ascii_lowercase();
    let literal = arg.literal || !value.is_empty();
    literal && !given.is_empty() && name.starts_with(&given)
}

/// A splatted argument (`@args`): its parameters are unknown.
fn splat(arg: &Word) -> bool {
    !arg.literal && !arg.quoted && arg.text.starts_with('@')
}

/// `base`, unless one of `switches` (lowercase parameter names) is given, or splatted.
fn switched(args: &[Word], base: PolicyClass, switches: &[&str], what: &str) -> Verdict {
    let given = args
        .iter()
        .any(|a| splat(a) || switches.iter().any(|s| param(a, s)));
    if given {
        return no(what);
    }
    ok(base)
}

fn new_item(args: &[Word]) -> Verdict {
    let link = args.windows(2).any(|w| {
        param(&w[0], "itemtype")
            && ["symboliclink", "junction", "hardlink"]
                .iter()
                .any(|t| w[1].text.eq_ignore_ascii_case(t))
    });
    if link || args.iter().any(splat) {
        return no("New-Item of a link");
    }
    ok(Modify)
}

fn tee_object(args: &[Word]) -> Verdict {
    let file = args.iter().any(|a| {
        ["filepath", "path", "literalpath"]
            .iter()
            .any(|p| param(a, p))
    });
    ok(if file { Modify } else { Read })
}

/// `ForEach-Object` with script blocks reads; with a member name it calls a method by name.
fn foreach_object(args: &[Word]) -> Verdict {
    let by_name = args.iter().any(|a| {
        param(a, "membername")
            || splat(a)
            || a.quoted
            || a.bare().is_some_and(|t| !t.starts_with('-'))
    });
    if by_name {
        return no("ForEach-Object with a member name");
    }
    ok(Read)
}

/// DevReady `replace`: `-DryRun` previews, `-All` writes, neither stops to ask.
fn replace(args: &[Word]) -> Verdict {
    if args.iter().any(|a| param(a, "all")) {
        return ok(Modify);
    }
    if args.iter().any(|a| param(a, "dryrun")) {
        return ok(Read);
    }
    no("replace without -DryRun or -All")
}

fn find(args: &[Word]) -> Verdict {
    let acting = [
        "-delete", "-exec", "-execdir", "-ok", "-okdir", "-fprint", "-fprint0", "-fprintf", "-fls",
    ];
    match args
        .iter()
        .find_map(|a| a.bare().filter(|t| acting.contains(t)))
    {
        Some(action) => no(format!("find {action}")),
        None => ok(Read),
    }
}

/// Literal option words before the subcommand; `takes` lists the options that take a value.
/// Returns the subcommand's index, or the Destructive verdict for an option not in `skip`.
fn subcommand(tool: &str, args: &[Word], skip: &[&str], takes: &[&str]) -> Result<usize, Verdict> {
    let mut i = 0;
    while let Some(arg) = args.get(i) {
        let Some(text) = arg.bare() else {
            return Err(no(format!("{tool} with a dynamic argument")));
        };
        if !text.starts_with(['-', '+']) {
            return Ok(i);
        }
        let name = text.split('=').next().unwrap_or_default();
        if takes.contains(&name) {
            i += if text.contains('=') { 1 } else { 2 };
            continue;
        }
        if !skip.contains(&text) && !text.starts_with('+') {
            return Err(no(format!("{tool} {text}")));
        }
        i += 1;
    }
    Ok(i)
}

fn has(rest: &[Word], flags: &[&str]) -> bool {
    rest.iter().any(|a| {
        a.bare().is_some_and(|t| {
            flags
                .iter()
                .any(|f| t == *f || t.strip_prefix(f).is_some_and(|r| r.starts_with('=')))
        })
    })
}

const GIT_CONFIG_ALLOWED: &[&str] = &["core.quotepath", "user.name", "user.email"];

fn git(args: &[Word]) -> Verdict {
    let mut i = 0;
    while let Some(text) = args.get(i).map(|a| a.bare()) {
        let Some(text) = text else {
            return no("git with a dynamic argument");
        };
        match text {
            "-C" => i += 2,
            "-c" => {
                let kv = args.get(i + 1).and_then(Word::bare).unwrap_or_default();
                let key = kv
                    .split('=')
                    .next()
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                let allowed = key.starts_with("color.")
                    || key.starts_with("advice.")
                    || GIT_CONFIG_ALLOWED.contains(&key.as_str());
                if !allowed {
                    return no(format!("git -c {key}"));
                }
                i += 2;
            }
            "--no-pager" | "--no-optional-locks" | "--literal-pathspecs" | "-P" => i += 1,
            "--version" | "-v" | "--help" | "-h" => return ok(Read),
            t if t.starts_with('-') => return no(format!("git {t}")),
            _ => break,
        }
    }
    let Some(sub) = args.get(i).and_then(Word::bare) else {
        return no("git without a literal subcommand");
    };
    let rest = &args[i + 1..];
    git_subcommand(sub, rest)
}

fn git_subcommand(sub: &str, rest: &[Word]) -> Verdict {
    let first = rest.first().and_then(Word::bare);
    let positional = rest
        .iter()
        .any(|a| a.bare().is_none_or(|t| !t.starts_with('-')));
    match sub {
        "status" | "rev-parse" | "ls-files" | "ls-tree" | "blame" | "describe" | "cat-file"
        | "shortlog" | "rev-list" | "name-rev" | "merge-base" | "show-ref" | "for-each-ref"
        | "version" | "help" | "var" | "check-ignore" | "check-attr" | "count-objects"
        | "show-branch" | "whatchanged" => ok(Read),
        "grep" if has(rest, &["-O", "--open-files-in-pager"]) => no("git grep -O"),
        "grep" => ok(Read),
        "diff" | "log" | "show" if has(rest, &["--ext-diff"]) => {
            no(format!("git {sub} --ext-diff"))
        }
        "diff" | "log" | "show" if has(rest, &["--output"]) => ok(Modify),
        "diff" | "log" | "show" => ok(Read),
        "hash-object" if has(rest, &["-w"]) => ok(Modify),
        "hash-object" => ok(Read),
        "config" if has(rest, &["--global", "--system", "--file", "-f"]) => {
            no("git config outside the repository")
        }
        "config"
            if has(
                rest,
                &["--get", "--get-all", "--get-regexp", "--list", "-l"],
            ) =>
        {
            ok(Read)
        }
        "config" => ok(Modify),
        "branch" => git_branch(rest, positional),
        "tag" if rest.is_empty() || has(rest, &["-l", "--list", "-n"]) => ok(Read),
        "tag" => ok(Modify),
        "remote" => match first {
            None | Some("-v" | "--verbose" | "show" | "get-url") => ok(Read),
            Some("add" | "rename" | "set-url" | "set-head" | "set-branches") => ok(Modify),
            Some(other) => no(format!("git remote {other}")),
        },
        "stash" => match first {
            None | Some("push" | "save" | "pop" | "apply" | "create" | "store" | "branch") => {
                ok(Modify)
            }
            Some("list" | "show") => ok(Read),
            Some(other) => no(format!("git stash {other}")),
        },
        "worktree" => match first {
            Some("list") => ok(Read),
            Some("add" | "lock" | "unlock" | "move" | "repair") => ok(Modify),
            _ => no("git worktree"),
        },
        "reflog" => match first {
            None | Some("show") => ok(Read),
            Some(other) => no(format!("git reflog {other}")),
        },
        "add" | "commit" | "apply" | "rm" | "mv" | "merge" | "cherry-pick" | "revert" | "notes"
        | "update-index" | "am" | "init" => ok(Modify),
        "switch" if has(rest, &["-f", "--force", "--discard-changes"]) => no("git switch --force"),
        "switch" => ok(Modify),
        "checkout" if has(rest, &["-f", "--force", "--", "-p", "--patch"]) => {
            no("git checkout of paths or with --force")
        }
        "checkout" if has(rest, &["-b", "-B", "--orphan"]) => ok(Modify),
        "checkout" => no("git checkout <name> (the name can be a path)"),
        "reset" if has(rest, &["--hard", "--merge", "--keep"]) => no("git reset --hard"),
        "reset" => ok(Modify),
        _ => no(format!("git {sub}")),
    }
}

fn git_branch(rest: &[Word], positional: bool) -> Verdict {
    if has(rest, &["-D", "-f", "--force", "-M", "-C"]) {
        return no("git branch --force");
    }
    let changes = [
        "-d",
        "--delete",
        "-m",
        "--move",
        "-c",
        "--copy",
        "-u",
        "--set-upstream-to",
        "--unset-upstream",
        "--edit-description",
    ];
    if has(rest, &changes) {
        return ok(Modify);
    }
    if has(rest, &["-l", "--list"]) || !positional {
        return ok(Read);
    }
    ok(Modify)
}

fn cargo(args: &[Word]) -> Verdict {
    if has(args, &["--config"]) {
        return no("cargo --config");
    }
    let skip = [
        "-q",
        "--quiet",
        "-v",
        "-vv",
        "--verbose",
        "--locked",
        "--offline",
        "--frozen",
        "--version",
        "-V",
        "--list",
        "-h",
        "--help",
    ];
    let takes = ["--manifest-path", "--color", "-Z", "-C"];
    let i = match subcommand("cargo", args, &skip, &takes) {
        Ok(i) => i,
        Err(verdict) => return verdict,
    };
    let Some(sub) = args.get(i).and_then(Word::bare) else {
        return ok(Read);
    };
    let rest = &args[i + 1..];
    match sub {
        "build" | "b" | "test" | "t" | "check" | "c" | "doc" | "d" | "bench" | "tree"
        | "metadata" | "verify-project" | "version" | "pkgid" | "locate-project" | "search"
        | "fetch" => ok(Build),
        "clippy" if has(rest, &["--fix"]) => ok(Modify),
        "clippy" => ok(Build),
        "fmt" if has(rest, &["--check"]) => ok(Build),
        "fmt" | "fix" | "add" | "update" | "remove" | "rm" | "generate-lockfile" | "new"
        | "init" | "vendor" => ok(Modify),
        _ => no(format!("cargo {sub}")),
    }
}

fn npm(args: &[Word]) -> Verdict {
    let skip = ["-s", "--silent", "-q", "--quiet", "--no-fund", "--no-audit"];
    let takes = ["--prefix", "-w", "--workspace", "--loglevel"];
    let i = match subcommand("npm", args, &skip, &takes) {
        Ok(i) => i,
        Err(verdict) => return verdict,
    };
    let Some(sub) = args.get(i).and_then(Word::bare) else {
        return ok(Read);
    };
    let rest = &args[i + 1..];
    match sub {
        "audit" if rest.first().and_then(Word::bare) == Some("fix") => ok(Modify),
        "ci" | "test" | "t" | "tst" | "run" | "run-script" | "ls" | "list" | "outdated"
        | "audit" | "view" | "info" | "explain" | "why" => ok(Build),
        "install" | "i" | "add" | "uninstall" | "un" | "remove" | "rm" | "r" | "update" | "up"
        | "upgrade" | "dedupe" => ok(Modify),
        _ => no(format!("npm {sub}")),
    }
}

fn dotnet(args: &[Word]) -> Verdict {
    let skip = ["--info", "--version", "--list-sdks", "--list-runtimes"];
    let i = match subcommand("dotnet", args, &skip, &[]) {
        Ok(i) => i,
        Err(verdict) => return verdict,
    };
    let Some(sub) = args.get(i).and_then(Word::bare) else {
        return ok(Read);
    };
    let rest = &args[i + 1..];
    match sub {
        "build" | "test" | "restore" | "msbuild" | "pack" | "clean" | "vstest" | "list" => {
            ok(Build)
        }
        "format" if has(rest, &["--verify-no-changes"]) => ok(Build),
        "format" | "add" | "remove" | "new" | "sln" => ok(Modify),
        _ => no(format!("dotnet {sub}")),
    }
}

fn go(args: &[Word]) -> Verdict {
    let Some(sub) = args.first().and_then(Word::bare) else {
        return no("go without a literal subcommand");
    };
    let rest = &args[1..];
    let first = rest.first().and_then(Word::bare);
    match sub {
        "env" if has(rest, &["-w", "-u"]) => ok(Modify),
        "build" | "test" | "vet" | "list" | "version" | "env" | "doc" => ok(Build),
        "fmt" | "get" => ok(Modify),
        "mod" => match first {
            Some("tidy" | "edit" | "vendor" | "init") => ok(Modify),
            Some("download" | "graph" | "verify" | "why") => ok(Build),
            _ => no("go mod"),
        },
        _ => no(format!("go {sub}")),
    }
}
