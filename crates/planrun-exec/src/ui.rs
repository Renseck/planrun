//! The interactive surface the runner talks to (decisions D15, D74). planrun-cli implements it
//! with line prompts on std; the tests with scripted answers. The runner decides what is asked
//! and what an answer does; the UI shows, reads, and handles `[v]`, `[d]` and `[o]` itself.

use crate::shell::OutLine;

/// An action's place in the plan, for the UI to show it: positions from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// The task's position in the plan.
    pub task: usize,
    /// The step's position in the task.
    pub step: usize,
    /// The action's position in the step.
    pub action: usize,
}

/// One answer a question offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// `[enter]` run it.
    Run,
    /// `[s]` skip it.
    Skip,
    /// `[v]` view the action in full.
    View,
    /// `[d]` the working tree's diff against HEAD.
    Diff,
    /// `[o]` an interactive shell at the repository root.
    Shell,
    /// `[q]` stop the run.
    Quit,
    /// `[y]` yes.
    Yes,
    /// `[n]` no.
    No,
    /// `[a]` accept the mismatch and go on.
    Accept,
    /// `[r]` run it again.
    Retry,
    /// `[enter]` go on.
    Continue,
    /// `[k]` keep the earlier run's result (resume).
    Keep,
    /// `[r]` restore the step from its pre-images (resume).
    Restore,
    /// `[p]` prepare the step again.
    PrepareAgain,
}

impl Choice {
    /// The key a line prompt reads for it; `""` is a bare enter.
    pub fn key(self) -> &'static str {
        match self {
            Choice::Run | Choice::Continue => "",
            Choice::Skip => "s",
            Choice::View => "v",
            Choice::Diff => "d",
            Choice::Shell => "o",
            Choice::Quit => "q",
            Choice::Yes => "y",
            Choice::No => "n",
            Choice::Accept => "a",
            Choice::Retry | Choice::Restore => "r",
            Choice::Keep => "k",
            Choice::PrepareAgain => "p",
        }
    }

    /// Its label (`run`, `skip`, …).
    pub fn label(self) -> &'static str {
        match self {
            Choice::Run => "run",
            Choice::Skip => "skip",
            Choice::View => "view action",
            Choice::Diff => "diff so far",
            Choice::Shell => "open shell",
            Choice::Quit => "quit",
            Choice::Yes => "yes",
            Choice::No => "no",
            Choice::Accept => "accept",
            Choice::Retry => "retry",
            Choice::Continue => "continue",
            Choice::Keep => "keep",
            Choice::Restore => "restore and redo",
            Choice::PrepareAgain => "prepare again",
        }
    }
}

/// What the runner asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// Its journal name: `run`, `unscoped`, `typed-yes`, `confirm`, `mismatch`, `manual`,
    /// `dirt`, `outside`, `session`, `gate`, `changed`, `again`, `unknown`, `accept-done`.
    pub kind: &'static str,
    /// The action it is about, when there is one (for `[v]`).
    pub at: Option<Position>,
    /// What to show above the prompt, one line per entry.
    pub text: Vec<String>,
    /// The answers offered, in order. Empty with `typed`.
    pub choices: Vec<Choice>,
    /// The answer is a typed line: only `yes` exactly agrees (D74).
    pub typed: bool,
}

/// What the UI read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// One of the question's choices.
    Choice(Choice),
    /// For a typed question: the line without its line ending, nothing else removed.
    Typed(String),
    /// No answer can be had (stdin is not a terminal, or it closed): the run stops.
    Stop(String),
}

/// The interactive surface (D15, D74).
pub trait Ui {
    /// Shows a line of progress.
    fn status(&mut self, line: &str);
    /// Shows one line of a running block's output, as it arrives.
    fn output(&mut self, line: &OutLine);
    /// Asks `question`; `[v]`, `[d]` and `[o]` come back as choices for the runner to pass to
    /// [`Ui::view`], [`Ui::diff`] and [`Ui::shell`].
    fn ask(&mut self, question: &Question) -> Reply;
    /// Shows the action at `at` in full.
    fn view(&mut self, at: Position);
    /// Shows `git --no-optional-locks diff HEAD` and the untracked files.
    fn diff(&mut self);
    /// Runs an interactive shell at the repository root until it exits; Ctrl-C reaches the
    /// shell, not planrun (D73).
    fn shell(&mut self);
}
