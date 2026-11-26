//! The run's shell sessions (D24, D81): at most one per shell, started lazily at the repository
//! root, never restarted silently. A restart bumps the shell's generation, so a Gate verdict
//! from the session before no longer counts (D75).

use std::path::{Path, PathBuf};

use planrun_model::Shell;

use crate::shell::{OutLine, ScriptRun, Session, SessionConfig, ShellError, ShellSession};

/// The sessions of one run.
pub(crate) struct Sessions {
    config: SessionConfig,
    live: [Option<Session>; 2],
    generation: [u64; 2],
    cwd: [Option<PathBuf>; 2],
    ever: [bool; 2],
}

/// Why a block did not run.
pub(crate) enum NotRun {
    /// The shell could not be started.
    Start(ShellError),
    /// The session refused the block (it has ended, or the block file could not be written).
    Refused(std::io::Error),
}

pub(crate) fn slot(shell: Shell) -> usize {
    match shell {
        Shell::Pwsh => 0,
        Shell::Bash => 1,
    }
}

pub(crate) fn name(shell: Shell) -> &'static str {
    match shell {
        Shell::Pwsh => "pwsh",
        Shell::Bash => "bash",
    }
}

impl Sessions {
    pub(crate) fn new(config: SessionConfig) -> Sessions {
        Sessions {
            config,
            live: [None, None],
            generation: [0, 0],
            cwd: [None, None],
            ever: [false, false],
        }
    }

    /// The generation of `shell`'s current session: 0 for the first, one more per restart.
    pub(crate) fn generation(&self, shell: Shell) -> u64 {
        self.generation[slot(shell)]
    }

    /// The working directory `shell`'s session reported after its last block.
    pub(crate) fn cwd(&self, shell: Shell) -> Option<&Path> {
        let i = slot(shell);
        self.live[i].as_ref()?;
        self.cwd[i].as_deref()
    }

    /// True when the next block of `shell` starts a session: `Some(true)` for a restart.
    pub(crate) fn starts(&self, shell: Shell) -> Option<bool> {
        let i = slot(shell);
        self.live[i].is_none().then_some(self.ever[i])
    }

    /// Runs `script` in `shell`'s session, starting it first when there is none.
    pub(crate) fn run(
        &mut self,
        shell: Shell,
        script: &str,
        on_line: &mut dyn FnMut(&OutLine),
    ) -> Result<ScriptRun, NotRun> {
        let i = slot(shell);
        if self.live[i].is_none() {
            let session = Session::start(shell, &self.config).map_err(NotRun::Start)?;
            self.live[i] = Some(session);
            self.ever[i] = true;
            self.cwd[i] = None;
        }
        let Some(session) = self.live[i].as_mut() else {
            return Err(NotRun::Refused(std::io::Error::other("no session")));
        };
        let run = session.run(script, on_line).map_err(NotRun::Refused)?;
        self.cwd[i] = run.cwd.clone();
        Ok(run)
    }

    /// Whether `shell`'s session can run another block.
    pub(crate) fn alive(&self, shell: Shell) -> bool {
        self.live[slot(shell)]
            .as_ref()
            .is_some_and(Session::is_alive)
    }

    /// Ends `shell`'s session; the next block starts a new one, a generation later.
    pub(crate) fn restart(&mut self, shell: Shell) {
        let i = slot(shell);
        self.live[i] = None;
        self.cwd[i] = None;
        self.generation[i] += 1;
    }
}
