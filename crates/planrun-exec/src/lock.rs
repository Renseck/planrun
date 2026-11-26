//! One planrun per repository (decision D77): `<git-dir>/planrun/lock`, held with
//! `File::try_lock` for the whole run, prompts included, by every plan's run and resume. The
//! operating system releases it when the process ends, however it ends, so there is no stale
//! lock. The holder's process id goes into `lock.pid` for the message only: on Windows a locked
//! file cannot be read by another process.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::Path;

/// The held lock; dropping it releases it.
#[derive(Debug)]
pub struct RunLock {
    _file: File,
}

/// Why the lock was not taken.
#[derive(Debug, thiserror::Error)]
pub enum LockError {
    /// Another planrun holds it.
    #[error("another planrun is running in this repository{}", pid_note(.pid))]
    Held {
        /// The holder's process id, as it wrote it.
        pid: Option<u32>,
    },
    /// The lock file could not be made or locked.
    #[error("the run lock: {0}")]
    Io(#[from] io::Error),
}

fn pid_note(pid: &Option<u32>) -> String {
    pid.map(|p| format!(" (process {p})")).unwrap_or_default()
}

/// Takes the lock in `dir` (`<git-dir>/planrun`), creating the directory, or says who holds it.
pub fn acquire(dir: &Path) -> Result<RunLock, LockError> {
    fs::create_dir_all(dir)?;
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join("lock"))?;
    match file.try_lock() {
        Ok(()) => {
            fs::write(dir.join("lock.pid"), std::process::id().to_string())?;
            Ok(RunLock { _file: file })
        }
        Err(TryLockError::WouldBlock) => {
            let pid = fs::read_to_string(dir.join("lock.pid"))
                .ok()
                .and_then(|s| s.trim().parse().ok());
            Err(LockError::Held { pid })
        }
        Err(TryLockError::Error(e)) => Err(LockError::Io(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_holder_is_refused_until_the_first_lets_go() {
        let dir = std::env::temp_dir().join(format!("planrun-lock-{}", std::process::id()));
        let first = acquire(&dir).unwrap_or_else(|e| panic!("{e}"));
        match acquire(&dir) {
            Err(LockError::Held { pid }) => assert_eq!(pid, Some(std::process::id())),
            other => panic!("{other:?}"),
        }
        drop(first);
        let again = acquire(&dir).unwrap_or_else(|e| panic!("{e}"));
        drop(again);
        let _ = fs::remove_dir_all(dir);
    }
}
