//! One planrun per repository (decisions D77, D90): `<git-dir>/planrun/lock`, held with
//! `File::try_lock` for the whole run, prompts included, by every plan's run and resume. The
//! operating system releases it when the process ends, however it ends, so there is no stale
//! lock. The holder's process id goes into `lock.pid` for the message only: on Windows a locked
//! file cannot be read by another process, and the file stays when its run ends.
//!
//! A run also holds `lock.live`, taken with a blocking `File::lock` once `lock` is held, so that
//! [`live`] can see a run in progress without trying `lock`: a try there would make a starting
//! run fail, while a probe that holds `lock.live` for a moment only makes it wait.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::Path;

/// The held lock; dropping it releases it.
#[derive(Debug)]
pub struct RunLock {
    _file: File,
    _live: File,
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

fn open(dir: &Path, name: &str) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(name))
}

/// Takes the lock in `dir` (`<git-dir>/planrun`), creating the directory, or says who holds it.
/// Once `lock` is held, `lock.live` is taken too, waiting while a [`live`] probe holds it.
pub fn acquire(dir: &Path) -> Result<RunLock, LockError> {
    fs::create_dir_all(dir)?;
    let file = open(dir, "lock")?;
    match file.try_lock() {
        Ok(()) => {
            let live = open(dir, "lock.live")?;
            live.lock()?;
            fs::write(dir.join("lock.pid"), std::process::id().to_string())?;
            Ok(RunLock {
                _file: file,
                _live: live,
            })
        }
        Err(TryLockError::WouldBlock) => Err(LockError::Held { pid: holder(dir) }),
        Err(TryLockError::Error(e)) => Err(LockError::Io(e)),
    }
}

/// Whether a run holds the lock in `dir` now (D90): one try on `lock.live`, let go at once.
/// Creates and writes nothing; `false` when there is no `lock.live` yet. A run that is between
/// taking `lock` and `lock.live` is not seen.
pub fn live(dir: &Path) -> io::Result<bool> {
    let file = match OpenOptions::new().read(true).open(dir.join("lock.live")) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    match file.try_lock() {
        Ok(()) => Ok(false),
        Err(TryLockError::WouldBlock) => Ok(true),
        Err(TryLockError::Error(e)) => Err(e),
    }
}

/// The process id in `lock.pid`, for a message: it may name a run that has ended.
pub fn holder(dir: &Path) -> Option<u32> {
    fs::read_to_string(dir.join("lock.pid"))
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    /// A fresh directory under the OS temp directory, not made yet.
    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("planrun-lock-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_second_holder_is_refused_until_the_first_lets_go() {
        let dir = temp("second");
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

    #[test]
    fn a_run_is_live_while_it_holds_the_lock_and_the_probe_writes_nothing() {
        let dir = temp("live");
        assert!(!live(&dir).unwrap_or_else(|e| panic!("{e}")));
        assert!(!dir.exists(), "the probe created the directory");
        let run = acquire(&dir).unwrap_or_else(|e| panic!("{e}"));
        assert!(live(&dir).unwrap_or_else(|e| panic!("{e}")));
        drop(run);
        assert!(!live(&dir).unwrap_or_else(|e| panic!("{e}")));
        assert_eq!(
            holder(&dir),
            Some(std::process::id()),
            "lock.pid stays, shown only"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn a_run_that_starts_while_a_probe_holds_lock_live_waits_and_succeeds() {
        let dir = temp("wait");
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
        let probe = open(&dir, "lock.live").unwrap_or_else(|e| panic!("{e}"));
        probe.try_lock().unwrap_or_else(|e| panic!("{e}"));
        let held = Duration::from_millis(300);
        let release = std::thread::spawn(move || {
            std::thread::sleep(held);
            drop(probe);
        });
        let start = Instant::now();
        let run = acquire(&dir).unwrap_or_else(|e| panic!("{e}"));
        assert!(
            start.elapsed() >= held / 2,
            "acquire did not wait for the probe"
        );
        release
            .join()
            .unwrap_or_else(|_| panic!("the probe thread panicked"));
        drop(run);
        let _ = fs::remove_dir_all(dir);
    }
}
