//! The process tree of a shell session (decision D64). Every `unsafe` call of planrun lives in
//! this module: the job object on Windows, `setsid` and `kill` on Unix, each declared by hand.
//! The rest of the crate sees three safe operations: [`Tree::spawn`], [`Tree::kill`] and
//! [`Tree::wait`].

#![allow(unsafe_code)]

use std::io;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};

/// The three pipes of a spawned session.
pub(crate) struct Pipes {
    /// The protocol channel (bash: one line per block); `None` when the session got none.
    pub stdin: Option<ChildStdin>,
    /// The session's stdout.
    pub stdout: ChildStdout,
    /// The session's stderr.
    pub stderr: ChildStderr,
}

/// A spawned process and every process it starts. Killing the tree kills them all; dropping it
/// kills them too, and never waits for anything but the process itself.
pub(crate) struct Tree {
    child: Child,
    reaped: Option<ExitStatus>,
    #[cfg(windows)]
    job: windows::Job,
}

impl Tree {
    /// Spawns `command` with piped stdout and stderr, stdin piped or null, detached from planrun's terminal
    /// (Windows: a console of its own; Unix: a session of its own), and puts it in a tree. When
    /// the tree cannot be made (Windows: a job that refuses nesting), the process is killed and
    /// the spawn fails: there is no session without the tree kill.
    pub(crate) fn spawn(command: &mut Command, stdin: bool) -> io::Result<(Tree, Pipes)> {
        let input = if stdin { Stdio::piped() } else { Stdio::null() };
        command
            .stdin(input)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        detach(command);
        #[cfg(windows)]
        let job = windows::Job::new()?;
        let mut child = command.spawn()?;
        #[cfg(windows)]
        if let Err(e) = job.assign(&child) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
        let input = child.stdin.take();
        let pipes = match (input, child.stdout.take(), child.stderr.take()) {
            (input, Some(stdout), Some(stderr)) if input.is_some() == stdin => Pipes {
                stdin: input,
                stdout,
                stderr,
            },
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::Error::other("the session's pipes were not created"));
            }
        };
        let tree = Tree {
            child,
            reaped: None,
            #[cfg(windows)]
            job,
        };
        Ok((tree, pipes))
    }

    /// Kills the process and all its descendants. Does nothing once the process was waited on:
    /// on Unix its process group id could then belong to someone else.
    pub(crate) fn kill(&mut self) -> io::Result<()> {
        if self.reaped.is_some() {
            return Ok(());
        }
        #[cfg(windows)]
        let killed = self.job.terminate();
        #[cfg(unix)]
        let killed = unix::kill_group(self.child.id());
        if killed.is_err() {
            let _ = self.child.kill();
        }
        killed
    }

    /// Waits for the process itself (not its descendants) and remembers its status.
    pub(crate) fn wait(&mut self) -> io::Result<ExitStatus> {
        if let Some(status) = self.reaped {
            return Ok(status);
        }
        let status = self.child.wait()?;
        self.reaped = Some(status);
        Ok(status)
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = self.kill();
        let _ = self.wait();
    }
}

/// Detaches a command from planrun's terminal: on Windows `CREATE_NO_WINDOW`, so the session
/// gets a console of its own and its code page changes stay there; on Unix `setsid`, so a read
/// from `/dev/tty` fails instead of stopping the process group.
pub(crate) fn detach(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(unix)]
    unix::setsid_on_exec(command);
}

#[cfg(unix)]
mod unix {
    use std::io;
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    const SIGKILL: i32 = 9;
    const ESRCH: i32 = 3;

    unsafe extern "C" {
        fn setsid() -> i32;
        fn kill(pid: i32, signal: i32) -> i32;
    }

    pub(super) fn setsid_on_exec(command: &mut Command) {
        // SAFETY: the closure runs in the forked child before exec. It only calls `setsid`,
        // which is async-signal-safe, and builds an `io::Error` from errno, which allocates
        // nothing; it touches no lock or memory shared with the parent.
        unsafe {
            command.pre_exec(|| {
                // SAFETY: `setsid` takes no arguments and only changes the calling process.
                if setsid() == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    /// Kills the process group led by `pid`. The caller guarantees `pid` was not waited on yet,
    /// so the group id is still this tree's (a zombie leader keeps it reserved).
    pub(super) fn kill_group(pid: u32) -> io::Result<()> {
        let Ok(pid) = i32::try_from(pid) else {
            return Err(io::Error::other("process id out of range"));
        };
        // SAFETY: `kill` takes plain integers; a negative pid names the process group, which
        // `setsid` made equal to the session's pid.
        if unsafe { kill(-pid, SIGKILL) } == 0 {
            return Ok(());
        }
        let e = io::Error::last_os_error();
        if e.raw_os_error() == Some(ESRCH) {
            return Ok(());
        }
        Err(e)
    }
}

#[cfg(windows)]
mod windows {
    use std::ffi::c_void;
    use std::io;
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;

    type Handle = *mut c_void;

    const EXTENDED_LIMIT_INFORMATION: i32 = 9;
    pub(super) const KILL_ON_JOB_CLOSE: u32 = 0x2000;

    #[repr(C)]
    #[derive(Default)]
    struct BasicLimitInformation {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: u32,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct IoCounters {
        read_operation_count: u64,
        write_operation_count: u64,
        other_operation_count: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }

    /// `JOBOBJECT_EXTENDED_LIMIT_INFORMATION`, laid out by hand; the size is tested.
    #[repr(C)]
    #[derive(Default)]
    pub(super) struct ExtendedLimitInformation {
        basic: BasicLimitInformation,
        io: IoCounters,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> Handle;
        fn SetInformationJobObject(job: Handle, class: i32, info: *const c_void, len: u32) -> i32;
        fn QueryInformationJobObject(
            job: Handle,
            class: i32,
            info: *mut c_void,
            len: u32,
            returned: *mut u32,
        ) -> i32;
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn TerminateJobObject(job: Handle, exit_code: u32) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
    }

    /// A job object that kills its processes when its last handle closes.
    pub(super) struct Job(Handle);

    // SAFETY: a job object handle is a kernel handle; any thread may use or close it.
    unsafe impl Send for Job {}

    impl Job {
        /// A new anonymous job with `KILL_ON_JOB_CLOSE`, read back to check it is set.
        pub(super) fn new() -> io::Result<Job> {
            // SAFETY: both arguments may be null (default security, no name).
            let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
            let job = Job(handle);
            let mut info = ExtendedLimitInformation::default();
            info.basic.limit_flags = KILL_ON_JOB_CLOSE;
            let len = size_of::<ExtendedLimitInformation>() as u32;
            // SAFETY: `info` is a live, correctly sized `JOBOBJECT_EXTENDED_LIMIT_INFORMATION`
            // for the class passed, and `job.0` is the handle created above.
            let set = unsafe {
                SetInformationJobObject(
                    job.0,
                    EXTENDED_LIMIT_INFORMATION,
                    (&raw const info).cast(),
                    len,
                )
            };
            if set == 0 {
                return Err(io::Error::last_os_error());
            }
            if job.limit_flags()? & KILL_ON_JOB_CLOSE == 0 {
                return Err(io::Error::other(
                    "the job object did not keep KILL_ON_JOB_CLOSE",
                ));
            }
            Ok(job)
        }

        /// The job's limit flags, as Windows reports them.
        pub(super) fn limit_flags(&self) -> io::Result<u32> {
            let mut info = ExtendedLimitInformation::default();
            let mut returned = 0u32;
            let len = size_of::<ExtendedLimitInformation>() as u32;
            // SAFETY: `info` is writable and `len` bytes long; `returned` is a valid out
            // pointer; `self.0` is a live job handle.
            let ok = unsafe {
                QueryInformationJobObject(
                    self.0,
                    EXTENDED_LIMIT_INFORMATION,
                    (&raw mut info).cast(),
                    len,
                    &raw mut returned,
                )
            };
            if ok == 0 {
                return Err(io::Error::last_os_error());
            }
            if returned != len {
                return Err(io::Error::other(
                    "the job's limit information has another size",
                ));
            }
            Ok(info.basic.limit_flags)
        }

        /// Puts `child` in the job. The session is assigned right after its spawn and before
        /// planrun writes its first line; a shell starts no child before it reads a block, so
        /// no descendant can escape between the two.
        pub(super) fn assign(&self, child: &Child) -> io::Result<()> {
            let process = child.as_raw_handle();
            // SAFETY: `self.0` is a live job handle and `process` the live handle `Child`
            // owns; the call keeps neither.
            if unsafe { AssignProcessToJobObject(self.0, process.cast()) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }

        /// Kills every process in the job.
        pub(super) fn terminate(&self) -> io::Result<()> {
            // SAFETY: `self.0` is a live job handle.
            if unsafe { TerminateJobObject(self.0, 1) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            // SAFETY: the handle was created by `new` and is closed only here, once.
            unsafe { CloseHandle(self.0) };
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[cfg(target_pointer_width = "64")]
        #[test]
        fn the_limit_information_has_windows_size() {
            assert_eq!(size_of::<ExtendedLimitInformation>(), 144);
        }

        #[test]
        fn a_new_job_kills_on_close() {
            let job = Job::new().unwrap_or_else(|e| panic!("{e}"));
            let flags = job.limit_flags().unwrap_or_else(|e| panic!("{e}"));
            assert_ne!(flags & KILL_ON_JOB_CLOSE, 0, "limit flags {flags:#x}");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::time::{Duration, Instant};

    use super::Tree;

    #[test]
    fn a_killed_tree_ends_and_kill_after_wait_does_nothing() {
        // `git cat-file --batch` waits on its stdin, which the tree keeps open.
        let mut command = Command::new("git");
        command.args(["cat-file", "--batch"]);
        let (mut tree, pipes) = Tree::spawn(&mut command, true).unwrap_or_else(|e| panic!("{e}"));
        let started = Instant::now();
        tree.kill().unwrap_or_else(|e| panic!("{e}"));
        let status = tree.wait().unwrap_or_else(|e| panic!("{e}"));
        assert!(!status.success(), "{status}");
        assert!(started.elapsed() < Duration::from_secs(10));
        tree.kill().unwrap_or_else(|e| panic!("{e}"));
        drop(pipes);
    }
}
