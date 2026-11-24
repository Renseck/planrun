<!-- expect: manifest-write-mode -->
<!-- why: full contents for a file the task declares as deleted: neither create nor overwrite -->

# Invalid — manifest-write-mode

## Tasks

### P1-T1: a

**Goal:** a.

**Files**

- Delete `src/a.rs`

**Step 1 — write the files**

File `src/a.rs` (full contents):

```rust
// x
```