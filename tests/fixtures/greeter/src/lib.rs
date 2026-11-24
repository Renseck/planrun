//! A tiny fixture crate for the planrun corpus.

pub mod old;

/// The greeting for `name`.
pub fn greeting(name: &str) -> String {
    old::legacy_greeting(name)
}
