# P2 — Greeter tooling Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Phase:** P2 · **Spec:** docs/design/greeter.md v0.2 (§3, §4)

**Goal:**
1. A `greet` PowerShell helper next to the crate.
2. A config file the helper reads.

**Repo / branch:** greeter (`$GR`) → `feature/p2-tooling`, from `main`.

**Decisions taken in this plan (Rens: review):**
- **D1 The helper is a script, not a module.** One file, dot-sourced.
- **D2 Config is JSON.** ConvertFrom-Json is built in.

## Conventions for this plan
- Blocks marked `powershell` are PowerShell 7. Blocks marked `bash` run with the Bash tool as-is.
- Every block starts from the `$GR` root.
- **Don't push from this session.** Pushing is a manual step (M2).
- Commit at the end of each task that says so.

---

### Task 0: Branch

```powershell
git switch main
git status --short            # empty
git switch -c feature/p2-tooling
```

**Verify:**
```powershell
git branch --show-current     # feature/p2-tooling
```

---

### Task 1: The helper and its config

**Files:** create
- `tools/greet.ps1`
- `tools/greet.json`

`tools/greet.json`:
```json
{
  "punctuation": "!",
  "names": ["Rens"]
}
```

`tools/greet.ps1`:
```powershell
# Greets every name in greet.json (P2 D1: a script, not a module).
$config = Get-Content (Join-Path $PSScriptRoot 'greet.json') -Raw | ConvertFrom-Json
foreach ($name in $config.names) { "Hello, $name$($config.punctuation)" }
```

**Verify:**
```powershell
pwsh -NoProfile -File tools/greet.ps1   # Hello, Rens!
Get-Content tools/greet.json -Raw | ConvertFrom-Json | Out-Null; 'json ok'
```

**Commit:** `Greet helper and config`

---

### Task 2: Punctuation from the crate, drop the duplicate

**Files:** modify `tools/greet.ps1`, `README.md`; delete `tools/greet.json`.

**2a.** `tools/greet.ps1`, edit 1 (old → new):
```powershell
$config = Get-Content (Join-Path $PSScriptRoot 'greet.json') -Raw | ConvertFrom-Json
foreach ($name in $config.names) { "Hello, $name$($config.punctuation)" }
```
```powershell
param([string[]]$Names = @('Rens'))
foreach ($name in $Names) { "Hello, $name!" }
```

**2b.** `git rm tools/greet.json`

**2c.** `README.md` edit (old → new; the old text includes the closing fence of the Examples block):
````markdown
cargo test
```
````
````markdown
cargo test
pwsh -NoProfile -File tools/greet.ps1 -Names Ada,Linus
```
````

**Verify:**
```bash
pwsh -NoProfile -File tools/greet.ps1 -Names Ada | tail -1   # Hello, Ada!
test ! -e tools/greet.json && echo gone
```

Check by eye: the README example renders as one code block.

**Commit:** `Greet helper: names as parameters`

---

### Task 3: Names from a library

**Files:**
- Create (Write tool): `tools/greet.ps1` (full replacement), `tools/lib/names.ps1`
- Modify: `tools/lib/format.ps1`

**3a.** `tools/greet.ps1` (replaces the whole file):
```powershell
. (Join-Path $PSScriptRoot 'lib/names.ps1')
foreach ($name in (Get-GreetNames)) { "Hello, $name!" }
```

**3b.** `tools/lib/names.ps1`:
```powershell
function Get-GreetNames { @('Ada', 'Linus') }
```

**3c.** `tools/lib/format.ps1`. Old:
```powershell
'{0}!'
```
New:
```powershell
'{0}.'
```

**Verify** (Bash tool, the shell CI uses):
```bash
pwsh -NoProfile -File tools/greet.ps1 | head -1   # Hello, Ada!
```
Expected:
- two greetings, one per name;

  the order follows `Get-GreetNames`.
- nothing on stderr:

  ```bash
  pwsh -NoProfile -File tools/greet.ps1 2>&1 >/dev/null | wc -l
  ```

Commit: `git add tools/greet.ps1 tools/lib/names.ps1 tools/lib/format.ps1; git commit -m "Greet: names from a library"`

---

### Task 4: Punctuation as a parameter

**Files:** modify `tools/greet.ps1`; regenerate the README examples by hand.

`greet.ps1`, edit 1 (old → new):
```powershell
. (Join-Path $PSScriptRoot 'lib/names.ps1')
```
```powershell
param([string]$Punctuation = '!')
. (Join-Path $PSScriptRoot 'lib/names.ps1')
```

Edit 2 (old → new):
```powershell
"Hello, $name!"
```
```powershell
"Hello, $name$Punctuation"
```

**Verify:** `pwsh -NoProfile -File tools/greet.ps1 -Punctuation ?` → all pass (one line per name).

Commit:
```powershell
git add tools/greet.ps1
git commit -m "Greet: punctuation as a parameter"
```

---

## Manual steps for Rens

### M1 Before execution
- Review D1–D2.

### M2 After Claude Code finishes
- Push and squash-merge.