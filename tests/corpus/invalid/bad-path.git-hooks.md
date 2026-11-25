<!-- expect: bad-path -->
<!-- why: a write into the git directory: a hook would run on the next commit (D51) -->

# Invalid — bad-path

## Tasks

### P1-T1: a

**Goal:** a.

**Files**

- Create `.git/hooks/pre-commit`

**Step 1 — write the files**

File `.git/hooks/pre-commit` (full contents):

```sh
exit 0
```

