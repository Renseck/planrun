<!-- expect: task-order -->
<!-- why: P1-T2 comes after P1-T3; task ordinals must strictly increase (gaps are allowed) -->

# Invalid — task-order

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

### P1-T3: c

**Goal:** c.

**Files**

- Create `src/c.rs`

**Step 1 — write the files**

File `src/c.rs` (full contents):

```rust
// x
```

### P1-T2: b

**Goal:** b.

**Files**

- Create `src/b.rs`

**Step 1 — write the files**

File `src/b.rs` (full contents):

```rust
// x
```

