<!-- expect: duplicate-task-id -->
<!-- why: two tasks are P1-T1 -->

# Invalid — duplicate-task-id

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

### P1-T1: b

**Goal:** b.

**Files**

- Create `src/b.rs`

**Step 1 — write the files**

File `src/b.rs` (full contents):

```rust
// x
```

