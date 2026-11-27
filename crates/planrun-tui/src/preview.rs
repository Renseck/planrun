//! The resume preview (docs/tui-design.md §6.F, decision D85): a read-only full view of what
//! `Runner::resume` will find, built from `analyse` and `runner::preview::preview`. It takes no
//! lock and writes nothing, and it shows when it was read (`g` reads again). Every question in
//! it is asked when resume reaches it; `enter` starts resume, which takes the lock, checks
//! everything again against the disk, and asks in the dock.

use std::path::PathBuf;
use std::sync::Arc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use planrun_exec::DoneState;
use planrun_exec::done::Seen;
use planrun_exec::runner::preview::Preview;
use planrun_exec::runner::resume::{Analysis, HeadVerdict};
use planrun_parse::Parsed;

use crate::draw::{Pen, section};
use crate::glyph::{Glyph, Glyphs};
use crate::history::RunEntry;
use crate::jobs::Bg;
use crate::theme::{Token, bold, fg, on};
use crate::tree;

/// What the preview read.
#[derive(Debug, Clone)]
pub struct PreviewData {
    /// When it was read.
    pub taken: u64,
    /// The plan file.
    pub path: PathBuf,
    /// The plan, parsed now.
    pub parsed: Arc<Parsed>,
    /// The journal's last chain.
    pub analysis: Analysis,
    /// What resume would find.
    pub preview: Preview,
    /// Untracked files, left alone by resume.
    pub untracked: Vec<String>,
}

/// The preview view.
#[derive(Debug, Clone)]
pub struct ResumeView {
    /// The run it previews.
    pub entry: RunEntry,
    /// What it read.
    pub data: Bg<Box<PreviewData>>,
}

impl ResumeView {
    /// Whether resume would stop at once on what the preview read.
    pub fn blocked(&self) -> Option<&'static str> {
        let p = &self.data.value()?.preview;
        if p.live {
            return Some("a run is live: resume would be refused");
        }
        if p.plan_changed {
            return Some("the plan changed since the run: resume would stop");
        }
        if p.index_lock {
            return Some(".git/index.lock exists: resume would stop");
        }
        if p.head == HeadVerdict::Moved {
            return Some("HEAD moved: resume would stop");
        }
        None
    }

    /// The keys the footer shows.
    pub fn footer(&self) -> Vec<(String, String)> {
        vec![
            ("enter".into(), "start resume".into()),
            ("g".into(), "refresh preview".into()),
            ("esc".into(), "back".into()),
        ]
    }

    /// The header's right side.
    pub fn state() -> Vec<(String, Style)> {
        vec![
            (
                " READ-ONLY ".to_owned(),
                bold(Token::Meta).bg(Token::Bg1.color()),
            ),
            (
                "  nothing locked, run or written".to_owned(),
                on(Token::Fg3, Token::Bg1),
            ),
        ]
    }

    /// Draws the preview in `area`.
    pub fn draw(&self, buf: &mut Buffer, area: Rect, g: Glyphs) {
        let x = area.x + 6;
        let right = area.right().saturating_sub(6);
        let width = Rect::new(x, area.y, right - x, area.height);
        let mut y = area.y + 1;
        let data = match &self.data {
            Bg::Current(d) | Bg::Stale(d) => d,
            Bg::Running => {
                Pen::new(buf, x, y, right, g).text(
                    &format!(
                        "{} reading the journal and the disk",
                        g.get(Glyph::Background)
                    ),
                    fg(Token::Fg2),
                );
                return;
            }
            Bg::Failed(e) | Bg::Unavailable(e) => {
                Pen::new(buf, x, y, right, g).text(e, fg(Token::Err));
                return;
            }
        };
        let mut pen = Pen::new(buf, x, y, right, g);
        pen.text("RESUME PREVIEW", bold(Token::Fg)).gap(2);
        let secs = data.taken / 1000;
        let taken = format!(
            "{:02}:{:02}:{:02}",
            (secs / 3600) % 24,
            (secs / 60) % 60,
            secs % 60
        );
        pen.text(
            &format!(
                "from the journal (analyse) and a read-only look at the disk, taken {taken} UTC"
            ),
            fg(Token::Fg3),
        );
        y += 1;
        Pen::new(buf, x, y, right, g).text(
            &format!("Resume itself decides again when it runs; questions marked {} are asked in the dock when it reaches them.", g.get(Glyph::Waiting)),
            fg(Token::Fg2),
        );
        y += 2;
        let (glyph, word, color) = match self.blocked() {
            Some(why) => (Glyph::Failed, why.to_owned(), Token::Err),
            None => (Glyph::Done, "nothing blocks a resume".to_owned(), Token::Ok),
        };
        section(
            buf,
            width,
            y,
            "JOURNAL AGAINST REPOSITORY",
            &[
                (format!("{} ", g.get(glyph)), fg(color)),
                (word, fg(Token::Fg2)),
            ],
            g,
        );
        y += 1;
        let p = &data.preview;
        let fingerprint = &data.analysis.run.fingerprint;
        let short = &fingerprint[..fingerprint.len().min(7)];
        let checks: [(bool, &str, String); 3] = [
            (
                !p.plan_changed,
                if p.plan_changed {
                    "plan changed"
                } else {
                    "plan unchanged"
                },
                format!(
                    "fingerprint {short} {} the run record",
                    if p.plan_changed {
                        "differs from"
                    } else {
                        "matches"
                    }
                ),
            ),
            (
                !p.index_lock,
                if p.index_lock {
                    "index.lock"
                } else {
                    "no index.lock"
                },
                if p.index_lock {
                    "git is busy, or crashed: planrun never removes it".into()
                } else {
                    "git is not busy".into()
                },
            ),
            (
                !p.live,
                if p.live {
                    "a run is live"
                } else {
                    "no live run"
                },
                if p.live {
                    "another planrun holds .git/planrun/lock".into()
                } else {
                    "no planrun holds .git/planrun/lock".into()
                },
            ),
        ];
        for (ok, what, detail) in checks {
            let (glyph, color) = if ok {
                (Glyph::Done, Token::Ok)
            } else {
                (Glyph::Failed, Token::Err)
            };
            let mut pen = Pen::new(buf, x, y, right, g);
            pen.glyph(glyph, bold(color))
                .gap(1)
                .cell(what, 20, fg(Token::Fg))
                .text(&detail, fg(Token::Fg3));
            y += 1;
        }
        y = self.draw_head(buf, x, y, right, data, g);
        y += 1;
        section(buf, width, y, "WHAT RESUME WILL DO", &[], g);
        y += 1;
        y = draw_steps(buf, x, y, right, data, g);
        y += 1;
        section(buf, width, y, "YOUR FILES", &[], g);
        y += 1;
        for path in &p.resume_dirt {
            let mut pen = Pen::new(buf, x, y, right, g);
            pen.glyph(Glyph::Waiting, bold(Token::Warn))
                .gap(1)
                .text(&path.to_string(), fg(Token::Fg));
            pen.text(
                &format!(
                    " {} no verified step wrote it {} a remaining action writes over it",
                    g.get(Glyph::Dot),
                    g.get(Glyph::Dot)
                ),
                fg(Token::Fg2),
            );
            Pen::new(buf, x + 2, y + 1, right, g).text(
                "asked once when resume reaches the task (resume-dirt)",
                fg(Token::Fg3),
            );
            y += 2;
        }
        for path in &data.untracked {
            if y + 6 >= area.bottom() {
                break;
            }
            let mut pen = Pen::new(buf, x, y, right, g);
            pen.glyph(Glyph::Dot, bold(Token::Fg3)).gap(1).text(
                &format!("{path} untracked {} left alone", g.get(Glyph::Dot)),
                fg(Token::Fg3),
            );
            y += 1;
        }
        if p.resume_dirt.is_empty() && data.untracked.is_empty() {
            Pen::new(buf, x, y, right, g).text(
                "no changes of yours that resume would touch",
                fg(Token::Fg2),
            );
        }
        let y = area.bottom().saturating_sub(5);
        crate::draw::hrule(buf, x, right, y, g);
        let mut pen = Pen::new(buf, x, y + 1, right, g);
        pen.chip(
            "enter",
            "start resume: takes the lock and checks everything again; questions come in the dock",
            Token::Bg0,
        );
        Pen::new(buf, x, y + 3, right, g).text("This preview explains what resume (D14, D77) will find; it holds no answers and decides nothing.", fg(Token::Fg3));
    }

    fn draw_head(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        right: u16,
        data: &PreviewData,
        g: Glyphs,
    ) -> u16 {
        let now = data
            .preview
            .now
            .commit
            .as_deref()
            .map_or("(none)", |c| &c[..c.len().min(7)]);
        let (glyph, color, what, detail) = match &data.preview.head {
            HeadVerdict::Unchanged => (Glyph::Done, Token::Ok, "HEAD as recorded", now.to_owned()),
            HeadVerdict::CommitMade { at } => (
                Glyph::Done,
                Token::Ok,
                "HEAD moved",
                format!("{now}: the commit at {} was made; done", place(at)),
            ),
            HeadVerdict::MovedByRunning { at, kind, .. } => (
                Glyph::Waiting,
                Token::Warn,
                "HEAD moved",
                format!(
                    "{now} while the {kind} at {} ran: asked (keep or retry)",
                    place(at)
                ),
            ),
            HeadVerdict::Moved => (
                Glyph::Failed,
                Token::Err,
                "HEAD moved",
                format!("{now}: not by the running action; resume stops"),
            ),
        };
        let mut pen = Pen::new(buf, x, y, right, g);
        pen.glyph(glyph, bold(color))
            .gap(1)
            .cell(what, 20, fg(Token::Fg))
            .text(&detail, fg(Token::Meta));
        y + 1
    }
}

fn place(at: &planrun_exec::journal::At) -> String {
    format!("{} step {} action {}", at.task, at.step + 1, at.action + 1)
}

fn draw_steps(
    buf: &mut Buffer,
    x: u16,
    mut y: u16,
    right: u16,
    data: &PreviewData,
    g: Glyphs,
) -> u16 {
    let plan = &data.parsed.plan;
    let Some(task_id) = &data.analysis.interrupted else {
        let next = plan
            .tasks
            .iter()
            .find(|t| !data.analysis.done.contains(&t.id.raw.to_string()));
        let text = match next {
            Some(t) => format!("{} starts: no task was interrupted", t.id.raw),
            None => "every task is done: resume ends the run".to_owned(),
        };
        Pen::new(buf, x, y, right, g).text(&text, fg(Token::Fg2));
        return y + 1;
    };
    let mut pen = Pen::new(buf, x, y, right, g);
    pen.glyph(Glyph::Running, bold(Token::Act))
        .gap(1)
        .text(task_id, fg(Token::Meta));
    pen.text(
        " runs again from its start, in a new session",
        fg(Token::Fg),
    );
    y += 1;
    let task = plan.tasks.iter().find(|t| &*t.id.raw == task_id.as_str());
    for step in &data.preview.steps {
        let number = task
            .and_then(|t| t.steps.get(step.at.step))
            .and_then(|s| s.number)
            .map_or_else(|| (step.at.step + 1).to_string(), |n| n.to_string());
        let paths: Vec<String> = step.disk.paths.iter().map(|(p, _)| p.to_string()).collect();
        let (glyph, color, what) = match step.disk.state {
            DoneState::Done => (
                Glyph::Done,
                Token::Ok,
                "on disk as planned → done, not written again",
            ),
            DoneState::NotDone => (
                Glyph::Pending,
                Token::Fg3,
                "as before → prepared and written again",
            ),
            DoneState::Unknown => (
                Glyph::Waiting,
                Token::Warn,
                "neither as planned nor as before → unknown",
            ),
        };
        let mut pen = Pen::new(buf, x + 2, y, right, g);
        pen.glyph(glyph, bold(color))
            .gap(1)
            .text(&format!("step {number}  "), fg(Token::Fg2));
        pen.text(&paths.join("  "), fg(Token::Fg)).gap(2).text(
            what,
            fg(if color == Token::Warn {
                Token::Warn
            } else {
                Token::Fg2
            }),
        );
        y += 1;
        if step.disk.state == DoneState::Unknown {
            let seen: Vec<String> = step
                .disk
                .paths
                .iter()
                .map(|(p, s)| format!("{p} {}", seen_word(*s)))
                .collect();
            Pen::new(buf, x + 6, y, right, g).text(&seen.join(", "), fg(Token::Fg3));
            y += 1;
            let how = if step.restorable {
                "asked when resume reaches it: restore from the pre-images (present) or accept with a typed yes"
            } else {
                "asked when resume reaches it: accept with a typed yes (a pre-image is gone, no restore)"
            };
            Pen::new(buf, x + 6, y, right, g).text(how, fg(Token::Fg3));
            y += 1;
        }
        if !step.temps.is_empty() {
            Pen::new(buf, x + 6, y, right, g).text(
                &format!("removed: the writer's temp files {}", step.temps.join(", ")),
                fg(Token::Fg3),
            );
            y += 1;
        }
    }
    for again in &data.preview.again {
        let label = task
            .and_then(|t| t.steps.get(again.at.step))
            .and_then(|s| s.actions.get(again.at.action))
            .map_or_else(
                || place(&again.at),
                |a| format!("{} {}", g.get(tree::kind_glyph(a)), tree::label(plan, a)),
            );
        let when = if again.running {
            "was running when interrupted"
        } else {
            "ran before the interruption"
        };
        let mut pen = Pen::new(buf, x + 2, y, right, g);
        pen.glyph(Glyph::Waiting, bold(Token::Warn))
            .gap(1)
            .text(&label, fg(Token::Fg))
            .gap(2);
        pen.text(
            &format!("{when} → asked again (run again or keep)"),
            fg(Token::Fg2),
        );
        y += 1;
    }
    y
}

fn seen_word(seen: Seen) -> &'static str {
    match seen {
        Seen::Post => "as planned",
        Seen::Pre => "as before",
        Seen::Neither => "neither",
    }
}
