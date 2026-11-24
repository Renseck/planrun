# Farewell Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a `farewell` function next to `greeting`.

**Architecture:** One pure function in its own module; `lib.rs` re-exports it. No state.

**Tech Stack:** Rust 2024, cargo test.

---

### Task 1: The farewell module

**Files:**
- Create: `src/farewell.rs`
- Modify: `src/lib.rs:3-4`
- Test: `src/farewell.rs`

- [ ] **Step 1: Write the failing test**

Create `src/farewell.rs` with only the test module:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn says_goodbye_by_name() {
        assert_eq!(super::farewell("Rens"), "Goodbye, Rens.");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test farewell`
Expected: FAIL with "cannot find function `farewell` in module `super`"

- [ ] **Step 3: Write minimal implementation**

Above the test module in `src/farewell.rs`, add:

```rust
/// The farewell for `name`.
pub fn farewell(name: &str) -> String {
    format!("Goodbye, {name}.")
}
```

In `src/lib.rs`, below `pub mod greet;` (line 3), add `pub mod farewell;` and change the re-export to:

```rust
pub use farewell::farewell;
pub use greet::greeting;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test farewell`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/farewell.rs src/lib.rs
git commit -m "feat: farewell"
```

---

### Task 2: Document the API

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add the section**

Append to `README.md`:

```markdown
## API

- `greeting(name)`: `Hello, <name>!`
- `farewell(name)`: `Goodbye, <name>.`
```

- [ ] **Step 2: Check by eye**

Open `README.md` in a Markdown preview: both bullets render, the code spans are monospace.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: API section"
```

---

## Self-Review

**Spec coverage:** farewell ✓, README ✓.

**Placeholder scan:** none.
