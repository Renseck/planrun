//! The classifier's tables, row by row: one test per shell, and one for what the splitter
//! refuses.

use planrun_model::Shell;

use super::PolicyClass::{Build, Destructive, Modify, Read};
use super::{DefaultClassifier, PolicyClass};

fn check(shell: Shell, rows: &[(&str, PolicyClass)]) {
    for (script, class) in rows {
        let found = DefaultClassifier.explain(shell, script);
        assert_eq!(
            found.class, *class,
            "{shell:?} {script:?}: {:?}",
            found.reasons
        );
        assert_eq!(
            found.reasons.is_empty(),
            *class != Destructive,
            "{script:?}"
        );
    }
}

#[test]
fn pwsh_commands() {
    #[rustfmt::skip]
    let rows = [
        ("", Read),
        ("git status --short            # empty", Read),
        ("Get-Content docs/modules.txt", Read),
        ("rg -n 'format!\\(\"Hello' src --glob '!src/lang.rs'", Read),
        ("$x = 41\nfunction F { 'f-called' }", Read),
        ("\"Hello, $name$($config.punctuation)\"", Read),
        ("$config = Get-Content (Join-Path $PSScriptRoot 'greet.json') -Raw | ConvertFrom-Json", Read),
        ("foreach ($name in $config.names) { \"Hello, $name\" }", Read),
        ("if ($gate -eq 0) { 'ok' }\nelse { 'no' }", Read),
        ("replace 'a' 'b' -Path src -Extension .rs -DryRun", Read),
        ("gitinfo; markers; Test-DevReadyPlan x.md", Read),
        ("[IO.Path]::Combine('a', 'b'); [Console]::Out; $s.Trim().Split(',').Count", Read),
        ("@{ a = 1; 'b' = 'x' }; @{a=1}", Read),
        ("git -C sub status; git -c color.ui=always --no-pager log --oneline", Read),
        ("git branch --show-current; git branch; git branch --list 'p*'", Read),
        ("Get-ChildItem -Recurse src | Sort-Object | Select-Object -First 3", Read),
        ("@'\nRemove-Item -Recurse x\n'@", Read),
        ("<# rm -rf #> 'x'; exit $gate", Read),
        ("& git status; Get-Item x | % { $_.Name } | ? { $_ }", Read),
        ("Get-Item x -ErrorAction SilentlyContinue; $a[0]; Write-Output a > $null 2>&1", Read),
        ("cargo --version; git --version; Set-Location sub; cd ..", Read),
        ("cargo test", Build),
        ("cargo test; $gate = $LASTEXITCODE; \"gate exit code: $gate\"", Build),
        ("cargo +nightly --manifest-path x/Cargo.toml build -q", Build),
        ("cargo fmt --check; cargo clippy --all-targets -- -D warnings", Build),
        ("dotnet build; dotnet restore; npm ci; npm run build; go test ./...", Build),
        ("brief { cargo test -p x }; test", Build),
        ("git add -- src/greet.rs src/lib.rs", Modify),
        ("if ($gate -eq 0) {\n  git add -- a\n  git commit -m \"m\"\n} else { 'no' }\ngit status --porcelain", Modify),
        ("git switch -c feature/x; git checkout -b y; git branch new", Modify),
        ("replace 'greeting(' 'greeting_en(' -Path src -Extension .rs -All", Modify),
        ("New-Item -ItemType Directory -Force docs | Out-Null; (Get-ChildItem src -Name | Sort-Object) -join \"`n\" | Set-Content -NoNewline docs/modules.txt", Modify),
        ("Remove-Item x.txt; Copy-Item a b; ni x; Tee-Object -FilePath x", Modify),
        ("'x' > out.txt; 'y' >> logs/out.txt", Modify),
        ("cargo fmt; cargo clippy --fix; npm install; dotnet format; go mod tidy", Modify),
        ("git diff --output=x.diff", Modify),
        ("Remove-Item -Recurse x", Destructive),
        ("Remove-Item –Recurse x", Destructive),
        ("rm -r x", Destructive),
        ("ri -rec x", Destructive),
        ("Remove-Item -Recurse:$true x", Destructive),
        ("Remove-Item @rest", Destructive),
        ("git push", Destructive),
        ("git reset --hard", Destructive),
        ("git -C x reset --hard", Destructive),
        ("git -c core.pager=less log", Destructive),
        ("git -c alias.x='!rm -rf .' x", Destructive),
        ("git --git-dir=x status", Destructive),
        ("git checkout main", Destructive),
        ("git checkout -- src/lib.rs", Destructive),
        ("git switch -f main", Destructive),
        ("git clean -fdx", Destructive),
        ("git stash drop", Destructive),
        ("git branch -D x", Destructive),
        ("git diff --ext-diff", Destructive),
        ("git config --global user.name x", Destructive),
        ("GIT.EXE PUSH", Destructive),
        ("iex 'x'", Destructive),
        ("Start-Process x", Destructive),
        ("irm http://x", Destructive),
        ("pwsh -NoProfile -File tools/greet.ps1   # Hello, Rens!", Destructive),
        ("bash -c 'ls'", Destructive),
        ("cmd /c dir", Destructive),
        ("& $cmd", Destructive),
        ("& 'git' status", Destructive),
        ("& .\\x.ps1", Destructive),
        (". ./x.ps1", Destructive),
        ("./script.ps1", Destructive),
        ("C:\\tools\\x.exe", Destructive),
        ("Microsoft.PowerShell.Management\\Remove-Item x", Destructive),
        ("frobnicate", Destructive),
        ("[IO.File]::Delete('x')", Destructive),
        ("$p.Kill()", Destructive),
        ("Get-ChildItem | % Delete", Destructive),
        ("'x' > C:\\out.txt", Destructive),
        ("'x' > ../out.txt", Destructive),
        ("'x' > $path", Destructive),
        ("cargo run; cargo install x", Destructive),
        ("npx x", Destructive),
        ("dotnet run", Destructive),
        ("go run .", Destructive),
        ("Import-Module ./module/x.psd1 -Force", Destructive),
        ("Add-Type -TypeDefinition $s", Destructive),
        ("New-Object Net.WebClient", Destructive),
        ("New-Item -ItemType SymbolicLink -Path l -Target x", Destructive),
        ("killport 8080", Destructive),
        ("replace 'a' 'b'", Destructive),
        ("\"$(Remove-Item -Recurse x)\"", Destructive),
        ("@\"\n$(git push)\n\"@", Destructive),
        ("@(git push)", Destructive),
        ("Write-Output ([IO.File]::ReadAllText('x'))", Destructive),
    ];
    check(Shell::Pwsh, &rows);
}

#[test]
fn bash_commands() {
    #[rustfmt::skip]
    let rows = [
        ("git branch --show-current     # feature/x", Read),
        ("cat x | head -1; wc -l < x", Read),
        ("test ! -e x && echo gone", Read),
        ("echo \"$(git status --short)\"; x=$(git rev-parse HEAD)", Read),
        ("for f in a b; do echo \"$f\"; done", Read),
        ("if [ -f x ]; then echo y; else echo n; fi", Read),
        ("arr=(a \"b c\" $(git status)); declare -A m=([k]=v)", Read),
        ("[[ -n $x ]]; (( i++ ))", Read),
        ("find . -name '*.rs'; sed 's/a/b/' x; command -v git", Read),
        ("cat <<'EOF'\nrm -rf /\nEOF\necho after", Read),
        ("echo x > /dev/null 2>&1; echo y >&2; echo z 2>/dev/null", Read),
        ("(cd sub && ls)", Read),
        ("cargo test 2>&1 | tail -3", Build),
        ("git add src && git commit -m 'x'", Modify),
        ("mkdir -p sub && cd sub", Modify),
        ("echo x > out.txt; echo y >> out.txt", Modify),
        ("sed -i 's/a/b/' x; rm x; touch y; sort -o x y; cp a b", Modify),
        ("cat <<EOF > out.txt\n$(git status)\nEOF", Modify),
        ("g() { echo; }", Read),
        ("rm -rf x", Destructive),
        ("rm --recursive x", Destructive),
        ("find . -delete", Destructive),
        ("find . -exec rm {} \\;", Destructive),
        ("git push origin main", Destructive),
        ("curl http://x", Destructive),
        ("bash x.sh", Destructive),
        ("./x.sh", Destructive),
        ("source x.sh", Destructive),
        (". x.sh", Destructive),
        ("eval \"$x\"", Destructive),
        ("$cmd", Destructive),
        ("FOO=1 git status", Destructive),
        ("pwsh -NoProfile -File tools/greet.ps1 -Names Ada | tail -1   # Hello, Ada!", Destructive),
        ("echo `rm -rf x`", Destructive),
        ("echo \"$(rm -rf x)\"", Destructive),
        ("cat <(curl x)", Destructive),
        ("cat <<EOF\n$(rm -rf x)\nEOF", Destructive),
        ("echo x > /etc/x", Destructive),
        ("echo x > ~/x", Destructive),
        ("echo x > \"$f\"", Destructive),
        ("g() { rm -rf x; }", Destructive),
        ("function g { git push; }", Destructive),
        ("xargs rm", Destructive),
        ("exec true", Destructive),
        ("\\rm x", Destructive),
    ];
    check(Shell::Bash, &rows);
}

#[test]
fn what_the_splitter_cannot_take_apart_is_destructive() {
    #[rustfmt::skip]
    let rows = [
        (Shell::Pwsh, "'unterminated"),
        (Shell::Pwsh, "\"unterminated"),
        (Shell::Pwsh, "@'\nnever closed"),
        (Shell::Pwsh, "@'not on its own line'@"),
        (Shell::Pwsh, "Write-Output ("),
        (Shell::Pwsh, "Write-Output )"),
        (Shell::Pwsh, "<# open"),
        (Shell::Pwsh, "$(Get-Item x"),
        (Shell::Pwsh, "[IO.Path"),
        (Shell::Pwsh, "Get-Content < x"),
        (Shell::Pwsh, "'x' >"),
        (Shell::Bash, "echo 'x"),
        (Shell::Bash, "echo \"x"),
        (Shell::Bash, "case $x in a) echo;; esac"),
        (Shell::Bash, "cat <<EOF\nnever"),
        (Shell::Bash, "echo ${x:-$(rm -rf /)}"),
        (Shell::Bash, "echo )"),
        (Shell::Bash, "echo $("),
        (Shell::Bash, "echo `x"),
        (Shell::Bash, "f ("),
    ];
    for (shell, script) in rows {
        let found = DefaultClassifier.explain(shell, script);
        assert_eq!(found.class, Destructive, "{shell:?} {script:?}");
        let unparsed = found.reasons.iter().any(|r| r.starts_with("unparsed: "));
        assert!(unparsed, "{shell:?} {script:?}: {:?}", found.reasons);
    }
}
