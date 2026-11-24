<!-- expect: blob-chain -->
<!-- why: P1-T2's preimage 3333333 is not P1-T1's postimage 2222222 -->

# Invalid — blob-chain

## Tasks

### P1-T1: a

**Goal:** a.

**Files**

- Modify `src/a.rs`

**Step 1 — write the files**

File `src/a.rs` (exact diff; save the block as `$env:TEMP\p.patch`, outside the repository, then apply it):

```diff
diff --git a/src/a.rs b/src/a.rs
index 1111111..2222222 100644
--- a/src/a.rs
+++ b/src/a.rs
@@ -1 +1 @@
-a
+b
```

### P1-T2: b

**Goal:** b.

**Files**

- Modify `src/a.rs`

**Step 1 — write the files**

File `src/a.rs` (exact diff; save the block as `$env:TEMP\p2.patch`, outside the repository, then apply it):

```diff
diff --git a/src/a.rs b/src/a.rs
index 3333333..4444444 100644
--- a/src/a.rs
+++ b/src/a.rs
@@ -1 +1 @@
-b
+c
```

