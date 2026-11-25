# Invalid corpus

Each `*.md` here must fail `planrun check` with **exactly one error**, whose code is the one on
the file's first line:

```
<!-- expect: <code> -->
<!-- why: <what is wrong, for the reader> -->
```

The test (`crates/planrun-parse/tests/invalid_corpus.rs`, Phase 1) walks this folder, parses each
file with the built-in registry, and asserts the set of error codes equals `{<code>}`. Codes are
pinned, messages are not (they are snapshot-tested separately, so a wording change is reviewed
but does not break this test).

Adding a check rule means adding a file here. Adding a file here never needs a code change.
A file is named after its code (`bad-path.md`); a second case of the same code adds a word
after a dot (`bad-path.git-hooks.md`).
````

File `tests/corpus/invalid/bad-path.git-hooks.md` (full contents):
