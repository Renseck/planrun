//! What a script is expected to do.

use serde::{Deserialize, Serialize};

use crate::span::TextRef;

/// The expectation attached to a script (docs/architecture.md §6.4).
///
/// `lines` come from a fenced `Expected …:` block and are matched mechanically. `prose` comes
/// from a sentence (`Expected: all pass`). A prose expectation never passes a script on its
/// own: the matcher derives `exit` from it and the user confirms (decision D11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expectation {
    /// The expected exit status.
    pub exit: ExitExpect,
    /// Summary lines: an ordered subsequence of the output; `…` matches any run of characters.
    pub lines: Option<TextRef>,
    /// The expectation as a sentence.
    pub prose: Option<TextRef>,
}

/// The expected exit status of a script.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitExpect {
    /// Exit code 0.
    Success,
    /// Any non-zero exit code: a test that must fail before its implementation exists. Such a
    /// verification is skipped by default (decision D12).
    Failure,
    /// This exact code (`rg` with no match exits 1).
    Code(i32),
    /// The plan does not say.
    Unknown,
}
