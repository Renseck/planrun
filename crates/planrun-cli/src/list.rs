//! `planrun list` (D97): the plan files discovery finds (D96), each with its format and the
//! state of its journal, one line each. Reads; changes nothing.

use planrun_exec::journal;
use planrun_exec::workspace::Workspace;
use planrun_parse::parse_plan;
use planrun_tui::jobs::{RunState, run_state};

use crate::{OK, PLAN_ERRORS, Setup, USAGE};

/// One discovered file, as listed.
struct Row {
    path: String,
    what: String,
    run: String,
}

/// Lists the discovered plans: exit 0, 1 when a file a format detects has errors, 2 outside a
/// repository or when git fails.
pub fn list(setup: &Setup, out: &mut String) -> u8 {
    let plans = match setup.discovery.plans(&[]) {
        Ok(plans) => plans,
        Err(e) => {
            out.push_str(&format!("planrun: error[{}]: {e}\n", e.code()));
            return USAGE;
        }
    };
    let Some(root) = setup.discovery.root() else {
        return USAGE;
    };
    let git_dir = match Workspace::open(root, Vec::new()).and_then(|ws| Ok(ws.git().git_dir()?)) {
        Ok(dir) => dir,
        Err(e) => {
            out.push_str(&format!("planrun: {e}\n"));
            return USAGE;
        }
    };
    let (mut rows, mut errors, mut not_plans) = (Vec::new(), 0, 0);
    for plan in &plans {
        let path = plan.as_str().to_owned();
        let bytes = match std::fs::read(root.join(plan.as_str())) {
            Ok(bytes) => bytes,
            Err(e) => {
                out.push_str(&format!("planrun: {path}: {e}\n"));
                return USAGE;
            }
        };
        let (what, run) = match parse_plan(&setup.registry, &path, &bytes, None) {
            Ok(parsed) => {
                let id = parsed.plan.format.0.to_string();
                let format = setup
                    .registry
                    .origin(&id)
                    .map_or_else(|| id.clone(), |o| o.describe(&id, Some(root)));
                let tasks = parsed.plan.tasks.len();
                let plural = if tasks == 1 { "" } else { "s" };
                let state = journal_state(&git_dir, plan.file_name(), &path);
                (format!("{format}, {tasks} task{plural}"), state)
            }
            Err(d) if d.0.iter().any(|d| d.code == "no-format") => {
                not_plans += 1;
                ("not a plan".to_owned(), String::new())
            }
            Err(d) => {
                errors += 1;
                let n = d.error_codes().len();
                let plural = if n == 1 { "" } else { "s" };
                (
                    format!("{n} error{plural} (planrun check {path})"),
                    String::new(),
                )
            }
        };
        rows.push(Row { path, what, run });
    }
    let width = rows
        .iter()
        .map(|r| r.path.chars().count())
        .max()
        .unwrap_or(0);
    for row in &rows {
        let line = format!("{:width$}  {}  {}", row.path, row.what, row.run);
        out.push_str(line.trim_end());
        out.push('\n');
    }
    let globs = match &setup.discovery.config().file {
        Some(_) => "the globs of .planrun/config.toml".to_owned(),
        None => "the default globs".to_owned(),
    };
    let found = rows.len() - not_plans;
    out.push_str(&format!(
        "planrun: {found} plan{} and {not_plans} other file{} from {globs}\n",
        if found == 1 { "" } else { "s" },
        if not_plans == 1 { "" } else { "s" },
    ));
    if errors > 0 { PLAN_ERRORS } else { OK }
}

/// The plan's journal (D76) in words.
fn journal_state(git_dir: &std::path::Path, file_name: &str, plan_path: &str) -> String {
    let stem = file_name.strip_suffix(".md").unwrap_or(file_name);
    let lines = match journal::read(&journal::path_for(git_dir, stem)) {
        Ok(lines) => lines,
        Err(e) => return format!("journal unreadable: {e}"),
    };
    match run_state(&lines, plan_path) {
        RunState::NotRun => "not run".to_owned(),
        RunState::Done(_) => "done".to_owned(),
        RunState::Stopped => "stopped; resume goes on".to_owned(),
        RunState::Interrupted => "interrupted; resume goes on".to_owned(),
        RunState::Foreign(other) => format!("its journal is {other}'s"),
        RunState::Unreadable(e) => format!("journal unreadable: {e}"),
    }
}
