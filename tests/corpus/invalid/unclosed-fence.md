<!-- expect: unclosed-fence -->
<!-- why: the full-contents block is never closed, so it would swallow the rest of the plan -->

# Invalid — unclosed-fence

## Tasks

### P1-T1: a

**Goal:** a.

**Files**

- Create `src/a.rs`

**Step 1 — write the files**

File `src/a.rs` (full contents):

```rust
// never closed


**Step 2 — verify**

```powershell
cargo test
```

