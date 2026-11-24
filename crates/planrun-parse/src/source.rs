//! Reading a plan: UTF-8 check, BOM, line-ending normalization, fingerprint (decision D6).
//!
//! Phase 1 implements `load`: reject non-UTF-8 and input over 64 MiB with a diagnostic, strip a
//! leading BOM, turn CRLF and lone CR into LF, hash the original bytes (SHA-256; add the `sha2`
//! crate with a line in docs/decisions.md D9), and build a [`planrun_model::PlanSource`].
