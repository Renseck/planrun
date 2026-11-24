<!-- expect: patch-path -->
<!-- why: the lead names src/a.rs, the patch changes src/b.rs -->

# Invalid — patch-path

## Tasks

### P1-T1: a

**Goal:** a.

**Files**

- Modify `src/a.rs`

**Step 1 — write the files**

File `src/a.rs` (exact diff; save the block as `$env:TEMP\p.patch`, outside the repository, then apply it):

```diff
diff --git a/src/b.rs b/src/b.rs
index 1111111..2222222 100644
--- a/src/b.rs
+++ b/src/b.rs
@@ -1 +1 @@
-a
+b
```

