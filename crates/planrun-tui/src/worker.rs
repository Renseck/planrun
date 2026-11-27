//! The runner on its worker thread (decision D90). The runner stays synchronous: it reaches the
//! terminal only through the [`Ui`] here, whose `ask`, `shell`, `view` and `diff` are blocking
//! round trips to the render thread; `status` and `output` are sent and not waited on. The live
//! view comes from the journal: an observer, built on this thread (it is not `Send`), forwards
//! every record over the same channel.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Sender, channel};

use planrun_exec::journal::{self, Journal, Observer, Record, Seam};
use planrun_exec::lock;
use planrun_exec::policy::{Policy, PolicyClass};
use planrun_exec::runner::resume::analyse;
use planrun_exec::runner::{Finished, RunConfig, Runner};
use planrun_exec::shell::{OutLine, SessionConfig};
use planrun_exec::ui::{Position, Question, Reply, Ui};
use planrun_exec::workspace::Workspace;
use planrun_parse::Parsed;

use crate::app::Msg;
use crate::jobs::Context;

/// Builds an extra journal observer on the worker thread (the tests' crash seam, D93).
pub type ObserverFactory = Box<dyn FnOnce() -> Observer + Send>;

/// What to run.
pub struct RunRequest {
    /// The plan file.
    pub path: PathBuf,
    /// Its parsed plan.
    pub parsed: Arc<Parsed>,
    /// `--auto`.
    pub auto: Vec<PolicyClass>,
    /// `--run-expected-failures`.
    pub expected: bool,
    /// The first task, by position.
    pub from: Option<usize>,
    /// Resume the journal's unfinished run instead of starting one.
    pub resume: bool,
    /// More journal observers, after the TUI's own.
    pub observers: Vec<ObserverFactory>,
}

/// What the worker tells the render thread.
pub enum RunMsg {
    /// The run took the lock and opened its journal.
    Started,
    /// A journal record was appended.
    Record(Record),
    /// A step's commit wrote its n-th path.
    Wrote(usize),
    /// `Ui::status`.
    Status(String),
    /// `Ui::output`.
    Output(OutLine),
    /// `Ui::ask`: answer on the sender.
    Ask(Question, Sender<Reply>),
    /// `Ui::view`: show the action; reply when shown.
    View(Position, Sender<()>),
    /// `Ui::diff`: show the working tree's diff; reply when shown.
    Diff(Sender<()>),
    /// `Ui::shell`: run the `[o]` shell; reply when it exited.
    Shell(Sender<()>),
    /// The run ended, or could not start.
    Ended(Result<Finished, String>),
}

/// The [`Ui`] on the worker thread.
struct ChannelUi {
    tx: Sender<Msg>,
}

impl ChannelUi {
    fn send(&self, msg: RunMsg) {
        let _ = self.tx.send(Msg::Run(msg));
    }

    fn round_trip(&self, msg: impl FnOnce(Sender<()>) -> RunMsg) {
        let (tx, rx) = channel();
        self.send(msg(tx));
        let _ = rx.recv();
    }
}

impl Ui for ChannelUi {
    fn status(&mut self, line: &str) {
        self.send(RunMsg::Status(line.to_owned()));
    }

    fn output(&mut self, line: &OutLine) {
        self.send(RunMsg::Output(line.clone()));
    }

    fn ask(&mut self, question: &Question) -> Reply {
        let (tx, rx) = channel();
        self.send(RunMsg::Ask(question.clone(), tx));
        rx.recv()
            .unwrap_or_else(|_| Reply::Stop("the TUI closed before an answer".to_owned()))
    }

    fn view(&mut self, at: Position) {
        self.round_trip(|tx| RunMsg::View(at, tx));
    }

    fn diff(&mut self) {
        self.round_trip(RunMsg::Diff);
    }

    fn shell(&mut self) {
        self.round_trip(RunMsg::Shell);
    }
}

/// Starts `request` on a thread of its own; everything it has to say arrives as
/// [`Msg::Run`].
pub fn start(request: RunRequest, ctx: Arc<Context>, tx: Sender<Msg>) {
    std::thread::spawn(move || {
        let done = tx.clone();
        let result = catch_unwind(AssertUnwindSafe(|| run(request, &ctx, tx)));
        let ended = match result {
            Ok(ended) => ended,
            Err(payload) => Err(format!(
                "the run's thread panicked: {}",
                crate::jobs::panic_message(payload.as_ref())
            )),
        };
        let _ = done.send(Msg::Run(RunMsg::Ended(ended)));
    });
}

fn run(request: RunRequest, ctx: &Context, tx: Sender<Msg>) -> Result<Finished, String> {
    let ws = Workspace::open(&ctx.root, ctx.env.clone()).map_err(|e| e.to_string())?;
    let _lock = lock::acquire(&ctx.journal_dir()).map_err(|e| e.to_string())?;
    let stem = request
        .path
        .file_stem()
        .map_or_else(|| "plan".to_owned(), |s| s.to_string_lossy().into_owned());
    let path = journal::path_for(&ctx.git_dir, &stem);
    let plan_path = ctx.plan_path(&request.path);
    let lines = journal::read(&path).map_err(|e| e.to_string())?;
    if let Some(other) = lines.iter().find_map(|l| match &l.record {
        Record::Run { plan, .. } if *plan != plan_path => Some(plan.clone()),
        _ => None,
    }) {
        return Err(format!(
            "the journal {} belongs to the plan {other}: two plans share a file name",
            path.display()
        ));
    }
    let analysis = analyse(&lines).filter(|a| a.resumable());
    if analysis.is_some() && !request.resume {
        return Err("the plan has an unfinished run: resume it from History (F4)".to_owned());
    }
    if analysis.is_none() && request.resume {
        return Err("there is no unfinished run to resume".to_owned());
    }
    let mut journal = Journal::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    journal.observe(forward(tx.clone()));
    for factory in request.observers {
        journal.observe(factory());
    }
    let policy = Policy::new(&request.auto, request.expected).map_err(|e| e.to_string())?;
    let config = RunConfig {
        plan_path,
        policy,
        from: request.from,
        to: None,
        session: SessionConfig {
            root: ws.root().to_path_buf(),
            env: ctx.env.clone(),
            timeout: None,
            temp: std::env::temp_dir(),
        },
    };
    let _ = tx.send(Msg::Run(RunMsg::Started));
    let mut ui = ChannelUi { tx };
    let plan = &request.parsed.plan;
    let mut runner = Runner::new(plan, &ws, &mut journal, &mut ui, config);
    let finished = match &analysis {
        Some(analysis) => runner.resume(analysis),
        None => runner.run(),
    };
    finished.map_err(|e| e.to_string())
}

/// The TUI's observer: every record and write, forwarded.
fn forward(tx: Sender<Msg>) -> Observer {
    Box::new(move |seam| {
        let msg = match seam {
            Seam::Appended(record) => RunMsg::Record(record.clone()),
            Seam::Wrote(n) => RunMsg::Wrote(n),
        };
        let _ = tx.send(Msg::Run(msg));
    })
}
