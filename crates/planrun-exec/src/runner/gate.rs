//! The commit gate (decisions D22, D75): the last attempt of every script of the task, the
//! file actions the user skipped and the `Generated` checks. It is closed by an unresolved
//! mismatch, a confirmation answered no, a user skip, an accepted mismatch, a missing
//! `Generated` path, and a Gate that ran only in a session that has since ended. A skip by
//! policy (D67) keeps it open.

use planrun_model::Shell;

use crate::journal::Outcome;
use crate::runner::sessions::{Sessions, name};

/// One script's last attempt.
#[derive(Debug, Clone)]
pub(crate) struct Attempt {
    /// Where, as `L<line>` of the plan.
    pub line: u32,
    /// How it ended.
    pub outcome: Outcome,
    /// A Gate script (its verdict only counts from the current session).
    pub gate: bool,
    /// Its shell, and the session generation it ran in.
    pub shell: Shell,
    /// The generation.
    pub generation: u64,
}

/// What closes the gate, per task.
#[derive(Debug, Default)]
pub(crate) struct Gate {
    attempts: Vec<((usize, usize), Attempt)>,
    other: Vec<String>,
}

impl Gate {
    /// Records the last attempt of the script at (step, action), replacing an earlier one.
    pub(crate) fn record(&mut self, at: (usize, usize), attempt: Attempt) {
        self.attempts.retain(|(k, _)| *k != at);
        self.attempts.push((at, attempt));
    }

    /// Records something else that closes the gate (a skipped file action, a missing
    /// `Generated` path).
    pub(crate) fn close(&mut self, reason: String) {
        self.other.push(reason);
    }

    /// The Gate scripts whose verdict comes from a session that has ended (D75): they run
    /// again before the commit.
    pub(crate) fn stale_gates(&self, sessions: &Sessions) -> Vec<(usize, usize)> {
        self.attempts
            .iter()
            .filter(|(_, a)| a.gate && a.generation != sessions.generation(a.shell))
            .filter(|(_, a)| a.outcome != Outcome::SkippedPolicy)
            .map(|(k, _)| *k)
            .collect()
    }

    /// Every reason the gate is closed, in plan order; empty when it is open.
    pub(crate) fn reasons(&self, sessions: &Sessions) -> Vec<String> {
        let mut attempts: Vec<&((usize, usize), Attempt)> = self.attempts.iter().collect();
        attempts.sort_by_key(|(k, _)| *k);
        let mut reasons: Vec<String> = attempts
            .into_iter()
            .filter_map(|(_, a)| reason(a, sessions))
            .collect();
        reasons.extend(self.other.iter().cloned());
        reasons
    }
}

fn reason(a: &Attempt, sessions: &Sessions) -> Option<String> {
    let what = match a.outcome {
        Outcome::Failed => "failed",
        Outcome::Refused => "was not confirmed",
        Outcome::SkippedUser => "was skipped by you",
        Outcome::Accepted => "did not match, and you accepted it",
        _ if a.gate && a.generation != sessions.generation(a.shell) => {
            return Some(format!(
                "L{}: the gate ran in a {} session that has ended",
                a.line,
                name(a.shell)
            ));
        }
        _ => return None,
    };
    Some(format!("L{}: the script {what}", a.line))
}
