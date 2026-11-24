<!-- expect: expected-without-script -->
<!-- why: an Expected block with no script directly before it -->

# Invalid — expected-without-script

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

Expected (exit code 0; summary lines):

```text
ok
```