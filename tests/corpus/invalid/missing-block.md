<!-- expect: missing-block -->
<!-- why: a file lead sentence with no fenced block after it -->

# Invalid — missing-block

## Tasks

### P1-T1: a

**Goal:** a.

**Files**

- Create `src/a.rs`

**Step 1 — write the files**

File `src/a.rs` (full contents):

The block was forgotten.

**Step 2 — verify**

```powershell
cargo test
```

