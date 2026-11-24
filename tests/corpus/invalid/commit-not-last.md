<!-- expect: commit-not-last -->
<!-- why: a test runs after the task's commit -->

# Invalid — commit-not-last

## Tasks

### P1-T1: a

**Goal:** a.

**Files**

- Create `src/a.rs`

**Step 1 — write the files**

File `src/a.rs` (full contents):

```rust
// x
```

**Step 2 — commit**

```powershell
git add -- src/a.rs
git commit -m "P1-T1: a"
```

**Step 3 — verify**

```powershell
cargo test
```