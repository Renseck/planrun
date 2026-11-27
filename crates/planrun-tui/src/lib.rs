//! The planrun TUI (docs/tui-design.md; decisions D84–D94): a second [`Ui`](planrun_exec::Ui)
//! over the same crates as the line prompts, composed by planrun-cli as `planrun tui`. It shows
//! the domain model and adds no model of its own: resume, policy and preflight stay
//! authoritative; the TUI explains and asks, and never decides.

#![forbid(unsafe_code)]

pub mod app;
pub mod chrome;
pub mod diff;
pub mod draw;
pub mod driver;
pub mod glyph;
pub mod grid;
pub mod inspect;
pub mod jobs;
pub mod keys;
pub mod plans;
pub mod terminal;
pub mod text;
pub mod theme;
pub mod tree;

pub use terminal::{Host, run};
