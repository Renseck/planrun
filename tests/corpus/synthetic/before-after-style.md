# Greeter localisation — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Greetings in Dutch and English, chosen by a `Lang` value.

**Architecture:** A `Lang` enum and a `greeting_in(lang, name)` function; `greeting` keeps English.

**Tech Stack:** Rust 2024.

**Repo:** `greeter`, branch `feat/lang` (exists, clean). All paths below are relative to the repo root. Run every command from the repo root in PowerShell 7.

**Do not release between tasks.** Until Task 3 the README describes an API that does not exist yet.

---

## File map

| File | Status | Responsibility |
|---|---|---|
| `src/lang.rs` | new | `Lang`, `greeting_in` |
| `src/lib.rs` | modify | `pub mod lang;` |
| `src/greet.rs` | rewrite | delegates to `greeting_in` |
| `README.md` | modify | usage |

---

## Task 0: Baseline

- [ ] **Step 1: Confirm branch and green baseline**

```powershell
git branch --show-current
cargo test
```

Expected: branch `feat/lang`, all tests pass.

---

## Task 1: Lang and greeting_in

**Files:**
- Create: `src/lang.rs`
- Modify: `src/lib.rs:3`

- [ ] **Step 1: Create the module**

`src/lang.rs`:

```rust
//! Languages of a greeting.

/// A greeting language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    /// English.
    En,
    /// Dutch.
    Nl,
}

/// The greeting for `name` in `lang`.
pub fn greeting_in(lang: Lang, name: &str) -> String {
    match lang {
        Lang::En => format!("Hello, {name}!"),
        Lang::Nl => format!("Hallo, {name}!"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dutch_says_hallo() {
        assert_eq!(greeting_in(Lang::Nl, "Rens"), "Hallo, Rens!");
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

```powershell
cargo test lang
```

Expected: build error, `lang` is not a module of the crate yet.

- [ ] **Step 3: Declare the module**

`src/lib.rs`, before:

```rust
pub mod greet;
```

After:

```rust
pub mod greet;
pub mod lang;
```

- [ ] **Step 4: Run the tests to verify they pass**

```powershell
cargo test lang
```

Expected: all pass.

- [ ] **Step 5: Commit**

```powershell
git add src/lang.rs src/lib.rs
git commit -m "feat(lang): Lang and greeting_in"
```

---

## Task 2: greeting delegates

**Files:**
- Rewrite: `src/greet.rs`

- [ ] **Step 1: Replace the module**

`src/greet.rs` — replace the whole file:

```rust
//! Greetings: English by default.

use crate::lang::{Lang, greeting_in};

/// The English greeting for `name`.
pub fn greeting(name: &str) -> String {
    greeting_in(Lang::En, name)
}
```

- [ ] **Step 2: Add the regression test to the module**

`src/greet.rs`, add these members at the end of the file:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn english_is_the_default() {
        assert_eq!(super::greeting("Rens"), "Hello, Rens!");
    }
}
```

- [ ] **Step 3: Verify nothing still formats a greeting by hand**

```powershell
rg -n 'format!\("Hello' src --glob '!src/lang.rs'
```

Expected: no matches.

- [ ] **Step 4: Count the call sites, then rename them**

Preview:

```powershell
replace 'greeting(' 'greeting_en(' -Path src -Extension .rs -DryRun
```

Expected: 2 matches in 2 files. Then:

```powershell
replace 'greeting(' 'greeting_en(' -Path src -Extension .rs -All
```

- [ ] **Step 5: Re-pin the README hash**

```powershell
cargo test readme_hash
```

The failure prints `Expected: <old>` and `Actual: <new>`. Replace the hash in `tests/readme.rs` with the `Actual` value.

- [ ] **Step 6: Commit**

```powershell
git add src tests
git commit -m "feat(lang): greeting delegates to greeting_in"
```

---

## Task 3: Release check on TST

- [ ] **Step 1: Record the release**

Run on the TST database:

```sql
INSERT INTO dbo.Releases (Name, ReleasedAt) VALUES (N'greeter 0.2.0', SYSUTCDATETIME());
```

Expected: 1 row affected.

- [ ] **Step 2: Hand off**

Report the TST result in the PR description.
