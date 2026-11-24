<!-- expect: patch-shape -->
<!-- why: the hunk header says 2 lines removed, the body removes 1 -->

# Invalid — patch-shape

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
@@ -1,2 +1 @@
-a
+b
```

