# Local corpus (not committed)

Real plans go here: they contain customer names, internal paths and session ids, so everything in
this folder except this README is git-ignored (decision D16).

Copy in the plans Rens supplied when the project started:

- `2026-11-05-s5-1-p1-routes-seam.md` (assay)
- `2026-11-11-s5-1-p3c1-builder-core.md` (assay)
- `2026-11-09-p2-plugin-core.md` (devready)
- `2026-11-09-p4-rules-logging.md` (devready)
- `2026-09-19-heatmap-mode-toggle.md` (superpowers)
- `2026-11-19-cleanup-batched-runner.md` (before-after)

`cargo test -- --ignored local_corpus` parses every `*.md` here and prints a coverage report per
task; it fails on any parse error. It is `#[ignore]` so CI and fresh clones (no local files) stay
green.
