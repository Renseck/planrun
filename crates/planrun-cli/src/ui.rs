//! The line-prompt [`Ui`] (decisions D15, D74): plain lines on stdout, so cargo's output stays
//! in the terminal's scrollback, and answers read a line at a time from stdin. Only a terminal
//! answers: when stdin is not one, the first question stops the run (journaled), never with a
//! default. The `e2e` feature lifts that for planrun-cli's own end-to-end tests (D82).

use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::process::Command;

use planrun_exec::shell::{InterruptsIgnored, OutLine};
use planrun_exec::ui::{Choice, Position, Question, Reply, Ui};
use planrun_model::Plan;

use crate::view::{self, Detail, Viewer};

/// Line prompts on the terminal.
pub struct LineUi<'p> {
    plan: &'p Plan,
    root: PathBuf,
}

impl<'p> LineUi<'p> {
    /// Prompts for `plan`, run in the repository at `root`.
    pub fn new(plan: &'p Plan, root: PathBuf) -> LineUi<'p> {
        LineUi { plan, root }
    }

    fn print(&self, text: &str) {
        let mut out = io::stdout().lock();
        let _ = writeln!(out, "{text}");
        let _ = out.flush();
    }

    /// A line from stdin without its line ending (`\n` or `\r\n`), nothing else removed;
    /// `None` at end of input.
    fn read_line(&self) -> Option<String> {
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => None,
            Ok(_) => {
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Some(line)
            }
        }
    }
}

/// Whether answers may come from something that is not a terminal (the `e2e` feature only).
fn piped_answers_allowed() -> bool {
    cfg!(feature = "e2e")
}

/// The prompt line: `[enter] run  [s] skip  …`.
fn prompt(choices: &[Choice]) -> String {
    let parts: Vec<String> = choices
        .iter()
        .map(|c| match c.key() {
            "" => format!("[enter] {}", c.label()),
            key => format!("[{key}] {}", c.label()),
        })
        .collect();
    parts.join("   ")
}

impl Ui for LineUi<'_> {
    fn status(&mut self, line: &str) {
        self.print(line);
    }

    fn output(&mut self, line: &OutLine) {
        self.print(&format!("    {}", line.text.trim_end_matches('\r')));
    }

    fn ask(&mut self, question: &Question) -> Reply {
        if !io::stdin().is_terminal() && !piped_answers_allowed() {
            return Reply::Stop(format!(
                "planrun needs an answer (`{}`), and stdin is not a terminal",
                question.kind
            ));
        }
        self.print("");
        for line in &question.text {
            self.print(line);
        }
        loop {
            if !question.typed {
                self.print(&prompt(&question.choices));
            }
            let _ = io::stdout().flush();
            let Some(line) = self.read_line() else {
                return Reply::Stop("stdin closed before an answer".to_owned());
            };
            if question.typed {
                return Reply::Typed(line);
            }
            let found = question.choices.iter().find(|c| c.key() == line.trim());
            if let Some(choice) = found {
                return Reply::Choice(*choice);
            }
            self.print(&format!("`{line}` is not one of the answers"));
        }
    }

    fn view(&mut self, at: Position) {
        let Some(step) = self
            .plan
            .tasks
            .get(at.task)
            .and_then(|t| t.steps.get(at.step))
        else {
            return;
        };
        let viewer = Viewer::new(self.plan);
        let mut out = String::new();
        view::step_text(&viewer.step(step), Detail::Full, &mut out);
        self.print(out.trim_end());
    }

    fn diff(&mut self) {
        let run = |args: &[&str]| {
            let _ = Command::new("git")
                .args(args)
                .current_dir(&self.root)
                .status();
        };
        run(&["--no-optional-locks", "--no-pager", "diff", "HEAD"]);
        self.print("untracked:");
        run(&[
            "--no-optional-locks",
            "ls-files",
            "--others",
            "--exclude-standard",
        ]);
    }

    fn shell(&mut self) {
        let _quiet = InterruptsIgnored::new();
        let program = if cfg!(windows) { "pwsh" } else { "bash" };
        self.print(&format!(
            "{program} at {}; this is not the plan's session (its variables are not here). Exit to return.",
            self.root.display()
        ));
        let started = Command::new(program).current_dir(&self.root).status();
        if let Err(e) = started {
            self.print(&format!("{program} did not start: {e}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prompt_names_each_key() {
        let line = prompt(&[Choice::Run, Choice::Skip, Choice::Quit]);
        assert_eq!(line, "[enter] run   [s] skip   [q] quit");
    }
}
