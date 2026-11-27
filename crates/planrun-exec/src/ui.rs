//! The interactive surface the runner talks to (decisions D15, D74, D86). planrun-cli
//! implements it with line prompts on std; the tests with scripted answers. The runner decides
//! what is asked and what an answer does; the UI shows, reads, and handles `[v]`, `[d]` and
//! `[o]` itself.

use std::path::PathBuf;

use planrun_model::{RepoPath, Shell};

use crate::done::Seen;
use crate::expect::{Aligned, Mismatch, Reason};
use crate::git::{Head, StatusEntry};
use crate::journal::At;
use crate::policy::Decision;
use crate::shell::{OutLine, RunStatus};

/// The words a Destructive script's typed question accepts (D74).
pub const RUN_SKIP_OR_QUIT: &[&str] = &["yes", "skip", "quit"];

/// The words the commit gate's and an accept-as-done typed question accept (D75, D77).
pub const YES_OR_QUIT: &[&str] = &["yes", "quit"];

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

/// Which question it is, with what the run knows about it that the plan does not say (D86).
/// What the plan says (the action, its text, its path) is reached through [`Question::at`];
/// resume's questions, which have no `at`, name their step or action by its journal place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionKind {
    /// `run`: a script whose decision asks (D69).
    Run {
        /// The decision: `Ask` with the class that made it ask.
        decision: Decision,
    },
    /// `unscoped`: a file action outside the task's files (D50).
    Unscoped,
    /// `typed-yes`: a Destructive script (D69, D74).
    TypedYes {
        /// What made it Destructive, from the classifier.
        reasons: Vec<String>,
    },
    /// `confirm`: a result the plan states in prose or comments, or an exit it does not state
    /// (D11, D66).
    Confirm {
        /// Why it asks, in the matcher's order.
        reasons: Vec<Reason>,
        /// pwsh recorded an error.
        errored: bool,
    },
    /// `mismatch`: what was checked does not hold (D66, D75, D79, D81).
    Mismatch {
        /// How the block ended.
        status: RunStatus,
        /// What the matcher found.
        mismatches: Vec<Mismatch>,
        /// The block's output, both streams, as the matcher saw it.
        output: Vec<OutLine>,
        /// The expected summary lines against `output`, whose lines [`Aligned::output`]
        /// indexes (empty when the plan states none).
        aligned: Vec<Aligned>,
        /// For a commit, why it does not count as made (D79).
        commit: Option<String>,
        /// The session ended, timed out or broke: no accept (D75).
        dead: bool,
    },
    /// `manual`: a Manual waits until acknowledged (D21).
    Manual,
    /// `dirt`: changes outside the task's paths before a commit (D80).
    Dirt {
        /// The changed paths, as `git status` reports them.
        paths: Vec<StatusEntry>,
    },
    /// `outside`: the session's working directory is outside the repository (D81).
    Outside {
        /// The session's shell.
        shell: Shell,
        /// Its working directory.
        cwd: PathBuf,
    },
    /// `session`: the notice that a session is new (D81).
    Session {
        /// Why the session is new.
        why: String,
        /// The earlier tasks whose scripts ran in another session.
        earlier: Vec<String>,
    },
    /// `gate`: the commit gate is closed (D75).
    Gate {
        /// Why.
        reasons: Vec<String>,
    },
    /// `changed`: the disk changed between a step's prepare and its commit (D78).
    Changed {
        /// The paths that changed.
        paths: Vec<RepoPath>,
    },
    /// `again`: resume reached a script that ran before the interruption (D77).
    Again {
        /// It was running when planrun was interrupted, not ended.
        running: bool,
    },
    /// `unknown`: resume found a step neither as before nor as planned (D77).
    Unknown {
        /// The step, as the journal names it (its first action).
        step: At,
        /// Each path of the step as it is now.
        paths: Vec<(RepoPath, Seen)>,
        /// Every pre-image is there, so `[r]` can restore it (D57).
        restorable: bool,
    },
    /// `accept-done`: accepting an unknown step as done (D77).
    AcceptDone {
        /// The step, as the journal names it.
        step: At,
    },
    /// `head-moved`: HEAD moved by one commit while an action was running (D77).
    HeadMoved {
        /// The action that was running, as the journal names it.
        action: At,
        /// HEAD before the action.
        recorded: Head,
        /// HEAD now.
        now: Head,
    },
    /// `resume-dirt`: changes a remaining file action writes over, which may be the user's
    /// (D77).
    ResumeDirt {
        /// The paths.
        paths: Vec<RepoPath>,
    },
}

impl QuestionKind {
    /// Its journal name (`run`, `typed-yes`, …): the journal format does not change.
    pub fn name(&self) -> &'static str {
        match self {
            QuestionKind::Run { .. } => "run",
            QuestionKind::Unscoped => "unscoped",
            QuestionKind::TypedYes { .. } => "typed-yes",
            QuestionKind::Confirm { .. } => "confirm",
            QuestionKind::Mismatch { .. } => "mismatch",
            QuestionKind::Manual => "manual",
            QuestionKind::Dirt { .. } => "dirt",
            QuestionKind::Outside { .. } => "outside",
            QuestionKind::Session { .. } => "session",
            QuestionKind::Gate { .. } => "gate",
            QuestionKind::Changed { .. } => "changed",
            QuestionKind::Again { .. } => "again",
            QuestionKind::Unknown { .. } => "unknown",
            QuestionKind::AcceptDone { .. } => "accept-done",
            QuestionKind::HeadMoved { .. } => "head-moved",
            QuestionKind::ResumeDirt { .. } => "resume-dirt",
        }
    }
}

/// What the runner asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// Which question, with its detail.
    pub kind: QuestionKind,
    /// The action it is about, when there is one (for `[v]`).
    pub at: Option<Position>,
    /// What to show above the prompt, one line per entry.
    pub text: Vec<String>,
    /// The answers offered, in order. Empty with `typed`.
    pub choices: Vec<Choice>,
    /// For a typed question, the words it accepts, each exactly as written (D74, D86); `None`
    /// for a question of choices. The runner acts on these words and no other.
    pub typed: Option<&'static [&'static str]>,
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
