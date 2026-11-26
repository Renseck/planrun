//! The process tree of a shell session (decisions D64, D71, D73). Every `unsafe` call of
//! planrun lives in this module: the job object, the handle wait and the console control
//! handler on Windows; `setsid`, `prctl`, `getppid`, `waitid`, `kill`, `signal` and `raise` on
//! Unix, each declared by hand. The rest of the crate sees safe operations: [`Tree::spawn`],
//! [`Tree::exited`], [`Tree::kill`], [`Tree::wait`], [`install_interrupts`] and
//! [`ignore_interrupts`].

#![allow(unsafe_code)]

use std::io;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

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

/// While set, Ctrl-C does not end planrun: the `[o]` shell runs in the foreground (D73).
static IGNORE_INTERRUPT: AtomicBool = AtomicBool::new(false);

impl Tree {
    /// Spawns `command` with piped stdout and stderr, stdin piped or null, detached from
    /// planrun's terminal (Windows: a console of its own; Unix: a session of its own, killed
    /// when planrun dies on Linux), and puts it in a tree. When the tree cannot be made
    /// (Windows: a job that refuses nesting), the process is killed and the spawn fails: there
    /// is no session without the tree kill.
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
        #[cfg(unix)]
        unix::register(child.id());
        let input = child.stdin.take();
        let pipes = match (input, child.stdout.take(), child.stderr.take()) {
            (input, Some(stdout), Some(stderr)) if input.is_some() == stdin => Pipes {
                stdin: input,
                stdout,
                stderr,
            },
            _ => {
                let _ = child.kill();
                #[cfg(unix)]
                unix::unregister(child.id());
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

    /// True when the process itself has exited, without reaping it: on Linux `waitid` with
    /// `WNOWAIT`, so its process group id stays reserved for [`Tree::kill`] (D64); on Windows a
    /// zero wait on the handle the tree keeps open. Other Unix systems always answer false
    /// (D19).
    pub(crate) fn exited(&mut self) -> io::Result<bool> {
        if self.reaped.is_some() {
            return Ok(true);
        }
        #[cfg(windows)]
        let exited = windows::exited(&self.child);
        #[cfg(target_os = "linux")]
        let exited = unix::exited(self.child.id());
        #[cfg(all(unix, not(target_os = "linux")))]
        let exited = Ok(false);
        exited
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

    /// Waits for the process itself (not its descendants) and remembers its status. On Unix its
    /// group leaves the interrupt handler's list first, so the handler never signals a group id
    /// that may be reused.
    pub(crate) fn wait(&mut self) -> io::Result<ExitStatus> {
        if let Some(status) = self.reaped {
            return Ok(status);
        }
        #[cfg(unix)]
        unix::unregister(self.child.id());
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
/// from `/dev/tty` fails instead of stopping the process group, and on Linux
/// `PR_SET_PDEATHSIG`, so the shell dies with planrun (D71).
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

/// Installs planrun's interrupt handling (D73). Unix: SIGINT, SIGTERM and SIGHUP kill every
/// session's process group, then end planrun as the signal would. Windows: a console control
/// handler that only swallows Ctrl-C while [`ignore_interrupts`] is on; otherwise Windows ends
/// planrun and the job objects kill the sessions. Call it once, from the main thread.
pub(crate) fn install_interrupts() {
    #[cfg(unix)]
    unix::install();
    #[cfg(windows)]
    windows::install();
}

/// Ctrl-C leaves planrun alone while `on` (the `[o]` shell runs in the foreground).
pub(crate) fn ignore_interrupts(on: bool) {
    IGNORE_INTERRUPT.store(on, Ordering::SeqCst);
}

#[cfg(unix)]
mod unix {
    use std::io;
    use std::os::unix::process::CommandExt;
    use std::process::Command;
    use std::sync::atomic::{AtomicI32, Ordering};

    use super::IGNORE_INTERRUPT;

    const SIGHUP: i32 = 1;
    const SIGINT: i32 = 2;
    const SIGKILL: i32 = 9;
    const SIGTERM: i32 = 15;
    const ESRCH: i32 = 3;
    /// `SIG_DFL`.
    const DEFAULT: usize = 0;

    unsafe extern "C" {
        fn setsid() -> i32;
        fn kill(pid: i32, signal: i32) -> i32;
        fn signal(signum: i32, handler: usize) -> usize;
        fn raise(signum: i32) -> i32;
    }

    /// The process groups of the live sessions, for the signal handler; 0 is a free slot.
    static GROUPS: [AtomicI32; 8] = [const { AtomicI32::new(0) }; 8];

    pub(super) fn register(pid: u32) {
        let Ok(pid) = i32::try_from(pid) else {
            return;
        };
        for slot in &GROUPS {
            if slot
                .compare_exchange(0, pid, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return;
            }
        }
    }

    pub(super) fn unregister(pid: u32) {
        let Ok(pid) = i32::try_from(pid) else {
            return;
        };
        for slot in &GROUPS {
            let _ = slot.compare_exchange(pid, 0, Ordering::SeqCst, Ordering::SeqCst);
        }
    }

    /// Kills every registered group, then ends planrun the way `signum` would. Only
    /// async-signal-safe calls: atomics, `kill`, `signal`, `raise`.
    extern "C" fn on_signal(signum: i32) {
        if signum == SIGINT && IGNORE_INTERRUPT.load(Ordering::SeqCst) {
            return;
        }
        for slot in &GROUPS {
            let pid = slot.load(Ordering::SeqCst);
            if pid > 0 {
                // SAFETY: `kill` takes plain integers; the slot is cleared before its leader
                // is reaped, so the group id is still the session's.
                unsafe { kill(-pid, SIGKILL) };
            }
        }
        // SAFETY: restoring the default action and raising the signal again ends the process
        // as the signal would have; both calls are async-signal-safe.
        unsafe {
            signal(signum, DEFAULT);
            raise(signum);
        }
    }

    pub(super) fn install() {
        let handler = on_signal as extern "C" fn(i32) as usize;
        for signum in [SIGINT, SIGTERM, SIGHUP] {
            // SAFETY: `handler` is an `extern "C" fn(i32)` that lives for the whole program and
            // does only async-signal-safe work.
            unsafe { signal(signum, handler) };
        }
    }

    pub(super) fn setsid_on_exec(command: &mut Command) {
        let parent = std::process::id();
        // SAFETY: the closure runs in the forked child before exec. It only calls `setsid`,
        // `prctl` and `getppid`, which are async-signal-safe, and builds an `io::Error` from
        // errno, which allocates nothing; it touches no lock or memory shared with the parent.
        unsafe {
            command.pre_exec(move || {
                // SAFETY: `setsid` takes no arguments and only changes the calling process.
                if setsid() == -1 {
                    return Err(io::Error::last_os_error());
                }
                #[cfg(target_os = "linux")]
                linux::die_with_parent(parent)?;
                #[cfg(not(target_os = "linux"))]
                let _ = parent;
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

    #[cfg(target_os = "linux")]
    pub(super) fn exited(pid: u32) -> io::Result<bool> {
        linux::exited(pid)
    }

    #[cfg(target_os = "linux")]
    mod linux {
        use std::io;

        const PR_SET_PDEATHSIG: i32 = 1;
        const P_PID: i32 = 1;
        const WNOHANG: i32 = 1;
        const WEXITED: i32 = 4;
        const WNOWAIT: i32 = 0x0100_0000;

        /// `siginfo_t` is 128 bytes; only `si_signo`, its first field, is read.
        #[repr(C, align(8))]
        struct SigInfo {
            signo: i32,
            rest: [i32; 31],
        }

        unsafe extern "C" {
            fn prctl(option: i32, ...) -> i32;
            fn getppid() -> i32;
            fn waitid(idtype: i32, id: u32, info: *mut SigInfo, options: i32) -> i32;
        }

        /// In the forked child: SIGKILL when the parent dies, and refuse to start when it
        /// already has (the parent then is no longer `parent`). The signal follows the thread
        /// that spawned the child, so sessions are spawned from planrun's main thread.
        pub(super) fn die_with_parent(parent: u32) -> io::Result<()> {
            // SAFETY: `prctl(PR_SET_PDEATHSIG, sig)` takes the signal as an unsigned long and
            // changes only the calling process.
            if unsafe { prctl(PR_SET_PDEATHSIG, super::SIGKILL as u64) } == -1 {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: `getppid` takes no arguments and cannot fail.
            let now = unsafe { getppid() };
            if u32::try_from(now).ok() != Some(parent) {
                return Err(io::Error::from_raw_os_error(super::ESRCH));
            }
            Ok(())
        }

        /// Whether `pid` has exited, leaving it a zombie (`WNOWAIT`): the kernel sets
        /// `si_signo` to SIGCHLD when it has and to 0 when it has not.
        pub(super) fn exited(pid: u32) -> io::Result<bool> {
            let mut info = SigInfo {
                signo: 0,
                rest: [0; 31],
            };
            // SAFETY: `info` is a writable, 8-aligned, 128-byte `siginfo_t`; the flags make the
            // call return at once and leave the child unreaped.
            let r = unsafe { waitid(P_PID, pid, &raw mut info, WEXITED | WNOHANG | WNOWAIT) };
            if r == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(info.signo != 0)
        }

        #[cfg(test)]
        mod tests {
            #[test]
            fn siginfo_is_128_bytes() {
                assert_eq!(size_of::<super::SigInfo>(), 128);
            }
        }
    }
}

#[cfg(windows)]
mod windows {
    use std::ffi::c_void;
    use std::io;
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;
    use std::sync::atomic::Ordering;

    use super::IGNORE_INTERRUPT;

    type Handle = *mut c_void;

    const EXTENDED_LIMIT_INFORMATION: i32 = 9;
    pub(super) const KILL_ON_JOB_CLOSE: u32 = 0x2000;
    const WAIT_OBJECT_0: u32 = 0;
    const WAIT_FAILED: u32 = 0xFFFF_FFFF;
    const CTRL_C_EVENT: u32 = 0;
    const CTRL_BREAK_EVENT: u32 = 1;

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

    type CtrlHandler = unsafe extern "system" fn(u32) -> i32;

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
        fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
        fn SetConsoleCtrlHandler(handler: Option<CtrlHandler>, add: i32) -> i32;
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

    /// Whether `child` has exited: a zero-timeout wait on the process handle `Child` owns.
    pub(super) fn exited(child: &Child) -> io::Result<bool> {
        // SAFETY: the handle is live for as long as `child`; a zero timeout never blocks.
        match unsafe { WaitForSingleObject(child.as_raw_handle().cast(), 0) } {
            WAIT_OBJECT_0 => Ok(true),
            WAIT_FAILED => Err(io::Error::last_os_error()),
            _ => Ok(false),
        }
    }

    /// Swallows Ctrl-C and Ctrl-Break while the `[o]` shell runs; otherwise passes them on to
    /// the default handler, which ends planrun. A handler routine, not `SetConsoleCtrlHandler`
    /// with a null routine: that flag is inherited by child processes, so the shell and its
    /// commands would ignore Ctrl-C too.
    unsafe extern "system" fn on_ctrl(kind: u32) -> i32 {
        let interrupt = kind == CTRL_C_EVENT || kind == CTRL_BREAK_EVENT;
        i32::from(interrupt && IGNORE_INTERRUPT.load(Ordering::SeqCst))
    }

    pub(super) fn install() {
        // SAFETY: `on_ctrl` is a `system` function that lives for the whole program and only
        // reads an atomic.
        unsafe { SetConsoleCtrlHandler(Some(on_ctrl), 1) };
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

        #[test]
        fn the_control_handler_swallows_ctrl_c_only_while_ignoring() {
            super::super::ignore_interrupts(true);
            // SAFETY: calling the handler directly only reads the atomic.
            let (c, close) = unsafe { (on_ctrl(CTRL_C_EVENT), on_ctrl(2)) };
            super::super::ignore_interrupts(false);
            // SAFETY: as above.
            let after = unsafe { on_ctrl(CTRL_C_EVENT) };
            assert_eq!((c, close, after), (1, 0, 0));
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
        assert!(!tree.exited().unwrap_or_else(|e| panic!("{e}")));
        tree.kill().unwrap_or_else(|e| panic!("{e}"));
        let until = Instant::now() + Duration::from_secs(10);
        while !tree.exited().unwrap_or_else(|e| panic!("{e}")) && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(20));
        }
        if cfg!(any(windows, target_os = "linux")) {
            assert!(tree.exited().unwrap_or_else(|e| panic!("{e}")));
        }
        let status = tree.wait().unwrap_or_else(|e| panic!("{e}"));
        assert!(!status.success(), "{status}");
        assert!(started.elapsed() < Duration::from_secs(10));
        tree.kill().unwrap_or_else(|e| panic!("{e}"));
        drop(pipes);
    }
}
