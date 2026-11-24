<!-- expect: expected-twice -->
<!-- why: two expectations for one script -->

# Invalid — expected-twice

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

**Step 2 — verify**

```powershell
cargo test
```

Expected (exit code 0; summary lines):

```text
test result: ok. …
```

Expected: all tests pass.