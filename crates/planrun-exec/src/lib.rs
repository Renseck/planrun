//! Executes a typed plan against a git repository (docs/architecture.md §6). Phases 3–5.
//!
//! Planned modules (docs/roadmap.md):
//!
//! - `workspace`: repository root, [`RepoPath`](planrun_model::RepoPath) resolution with the
//!   symlink check, per-task file cache, step transactions (validate every file action of a step
//!   in memory, then write), atomic writes, line-ending policy (D10), pre-images stored with
//!   `git hash-object -w`.
//! - [`git`]: a thin wrapper over the `git` binary: `apply` with the patch on stdin,
//!   `hash-object`, `status --porcelain` without optional locks, `rev-parse`, `cat-file`,
//!   `check-attr`, `update-index`, `ls-files`; [`BlobId`](git::BlobId).
//! - `handler`: one [`ActionHandler`] per action kind; the [`HandlerSet`] is the execution-side
//!   plugin point.
//! - `shell`: persistent pwsh and bash sessions behind [`ShellSession`].
//! - `expect`: exit-code and summary-line matching.
//! - `policy`: command classes and the [`CommandClassifier`].
//! - `journal`: JSONL under `.git/planrun/`.
//! - `runner`: the state machine Task → Step → Action, talking to a [`Ui`].

use planrun_model::Action;

pub mod git;

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod fixture;

/// The command classes of decision D13, least to most dangerous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PolicyClass {
    /// Reads only (`git status`, `rg`).
    Read,
    /// Builds or tests (`cargo test`, `dotnet build`, `npm ci`).
    Build,
    /// Changes the repository (`git add`, `git commit`, `replace … -All`, file actions).
    Modify,
    /// Destroys or reaches outside (`Remove-Item -Recurse`, `git reset --hard`, `git push`,
    /// anything unclassified).
    Destructive,
}

/// Classifies a script: the most dangerous class of any command in it.
pub trait CommandClassifier: Send + Sync {
    /// The class of `script` run by `shell`.
    fn classify(&self, shell: planrun_model::Shell, script: &str) -> PolicyClass;
}

/// The result of running one shell block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRun {
    /// `$LASTEXITCODE` / `$?` as the session reported it; `None` when the block threw.
    pub exit: Option<i32>,
    /// Merged stdout and stderr, as UTF-8 (lossy).
    pub output: String,
}

/// A long-lived shell: variables, the working directory and functions persist across blocks.
pub trait ShellSession: Send {
    /// Runs `script`, streaming each output line to `on_line` as it arrives.
    fn run(&mut self, script: &str, on_line: &mut dyn FnMut(&str)) -> std::io::Result<ScriptRun>;
}

/// The state of an action's effects on disk, for resuming (decision D14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoneState {
    /// Every effect is present (postimages match).
    Done,
    /// No effect is present (preimages match).
    NotDone,
    /// Some are, some are not, or neither matches: the user decides.
    Unknown,
}

/// Executes one kind of action. Registered per kind in the [`HandlerSet`].
pub trait ActionHandler: Send + Sync {
    /// True for the action kinds this handler executes.
    fn handles(&self, action: &Action) -> bool;
}

/// The handlers, tried in order; the first that handles an action executes it.
#[derive(Default)]
pub struct HandlerSet {
    handlers: Vec<Box<dyn ActionHandler>>,
}

impl HandlerSet {
    /// Adds a handler.
    pub fn add(&mut self, handler: Box<dyn ActionHandler>) {
        self.handlers.push(handler);
    }

    /// The handler for `action`.
    pub fn find(&self, action: &Action) -> Option<&dyn ActionHandler> {
        self.handlers
            .iter()
            .find(|h| h.handles(action))
            .map(|h| h.as_ref())
    }
}

/// The interactive surface the runner talks to (decision D15: line prompts first).
pub trait Ui {
    /// Shows a line of progress.
    fn status(&mut self, line: &str);
}
