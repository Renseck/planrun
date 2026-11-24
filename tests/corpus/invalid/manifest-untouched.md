<!-- expect: manifest-untouched -->
<!-- why: src/b.rs is declared but no action touches it -->

# Invalid — manifest-untouched

## Tasks

### P1-T1: a

**Goal:** a.

**Files**

- Create `src/a.rs`
- Modify `src/b.rs`

**Step 1 — write the files**

File `src/a.rs` (full contents):

```rust
// x
```

