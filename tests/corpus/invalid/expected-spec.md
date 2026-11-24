<!-- expect: expected-spec -->
<!-- why: the parenthesis names no exit status the assay format knows -->

# Invalid — expected-spec

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

Expected (summary lines):

```text
test result: ok. …
```