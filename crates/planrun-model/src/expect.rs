//! What a script is expected to do.

use serde::{Deserialize, Serialize};

use crate::span::TextRef;

/// The expectation attached to a script (docs/architecture.md §6.4).
///
/// `lines` come from a fenced `Expected …:` block and are matched mechanically. `prose` comes
/// from a sentence (`Expected: all pass`), `comments` from trailing comments on the script's
/// command lines (`git branch --show-current   # main`, decisions D35, D48). Neither ever
/// passes a script on its own: the matcher derives `exit` from the sentence and the user
/// confirms both, labelled apart (decision D11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expectation {
    /// The expected exit status.
    pub exit: ExitExpect,
    /// Summary lines: an ordered subsequence of the output; `…` matches any run of characters.
    pub lines: Option<TextRef>,
    /// The expectation as a sentence.
    pub prose: Option<TextRef>,
    /// The script's lines that carry a trailing comment expectation, one per line.
    pub comments: Option<TextRef>,
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
