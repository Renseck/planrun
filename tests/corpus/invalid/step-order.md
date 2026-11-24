<!-- expect: step-order -->
<!-- why: steps 1 then 3 -->

# Invalid — step-order

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

**Step 3 — verify**

```powershell
cargo test
```

