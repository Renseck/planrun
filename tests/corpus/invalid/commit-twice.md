<!-- expect: commit-twice -->
<!-- why: the task commits twice -->

# Invalid — commit-twice

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

```powershell
git commit --amend --no-edit
```