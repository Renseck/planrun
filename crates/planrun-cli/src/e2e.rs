//! The fault seam of planrun-cli's end-to-end tests (decision D82), compiled only with the
//! `e2e` feature, never into a release binary. `PLANRUN_E2E_ABORT=<seam>@<task>` aborts the
//! process right after the seam, once the journal record before it is with the OS (and synced,
//! when durable):
//!
//! - `step-prepared@P1-T2`: after a step's write-ahead record, before its first write;
//! - `write<k>@P1-T1`: after the k-th path a step's commit wrote;
//! - `commit-start@P1-T1`: after a Commit's `action-start`, before its block runs.
//!
//! It ends the process with `exit(134)`: no destructor runs (the journal, the sessions and the
//! lock are left as a kill leaves them; the job objects die with the process).

use planrun_exec::journal::{Observer, Record, Seam};

/// The observer `PLANRUN_E2E_ABORT` asks for, if any.
pub fn observer() -> Option<Observer> {
    let spec = std::env::var("PLANRUN_E2E_ABORT").ok()?;
    let (seam, task) = spec.split_once('@')?;
    let (seam, task) = (seam.to_owned(), task.to_owned());
    let mut current = String::new();
    Some(Box::new(move |s| {
        let hit = match s {
            Seam::Appended(Record::TaskStart { task, .. }) => {
                current.clone_from(task);
                false
            }
            Seam::Appended(Record::StepPrepared { at, .. }) => {
                seam == "step-prepared" && at.task == task
            }
            Seam::Appended(Record::ActionStart { at, kind, .. }) => {
                seam == "commit-start" && kind == "commit" && at.task == task
            }
            Seam::Wrote(n) => seam == format!("write{n}") && current == task,
            Seam::Appended(_) => false,
        };
        if hit {
            eprintln!("planrun: e2e abort at {spec}");
            // `exit` rather than `abort`: no destructor runs, as in a kill, and on Windows it
            // raises no Windows Error Reporting for the test machine to answer.
            std::process::exit(134);
        }
    }))
}
