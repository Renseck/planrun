//! History (docs/tui-design.md §6.E): the runs in `<git-dir>/planrun/*.jsonl`, journals set
//! aside by `--restart` included, newest first and grouped by date (UTC, D94). A run ends with
//! its `run-end`; without one it is "running in pid N" while `lock.live` says a run is live, and
//! "killed" otherwise (D90). `R` opens the resume preview of an unfinished run.

use std::path::{Path, PathBuf};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use planrun_exec::journal::{self, End, Line, Record};
use planrun_exec::runner::resume::analyse;

use crate::draw::{Pen, band, section, select_row};
use crate::glyph::{Glyph, Glyphs};
use crate::keys::Key;
use crate::plans::{MONTHS, civil, clock_utc};
use crate::theme::{Token, bold, fg, on};

/// One task as a run's journal has it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSeen {
    /// Its id.
    pub id: String,
    /// Its `task-end` HEAD, when it ended.
    pub commit: Option<String>,
    /// Milliseconds from `task-start` to `task-end`.
    pub took: Option<u64>,
}

/// One run: a chain of a journal, its first `run` record and the resumes appended to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunEntry {
    /// The journal file.
    pub journal: PathBuf,
    /// Set aside by `--restart` (`<stem>.<ms>.jsonl`).
    pub set_aside: bool,
    /// The last chain of its journal: the one resume would go on with.
    pub last: bool,
    /// The plan's path as recorded.
    pub plan: String,
    /// When it started.
    pub started: u64,
    /// When its last record was written.
    pub last_ms: u64,
    /// How its last process ended; `None` without a `run-end`.
    pub end: Option<End>,
    /// Why it stopped.
    pub reason: Option<String>,
    /// `--auto`.
    pub auto: Vec<String>,
    /// HEAD at the start.
    pub head: Option<String>,
    /// Resumes appended.
    pub resumes: usize,
    /// Its records.
    pub records: usize,
    /// The tasks it started, in order.
    pub tasks: Vec<TaskSeen>,
}

impl RunEntry {
    /// The plan file's name.
    pub fn name(&self) -> String {
        Path::new(&self.plan)
            .file_name()
            .map_or_else(|| self.plan.clone(), |n| n.to_string_lossy().into_owned())
    }

    /// Whether resume would go on with it.
    pub fn resumable(&self) -> bool {
        self.last && !self.set_aside && self.end != Some(End::Completed)
    }
}

/// Every run in the journal directory `dir`, newest first.
pub fn scan(dir: &Path) -> Result<Vec<RunEntry>, String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("{}: {e}", dir.display())),
    };
    let mut runs = Vec::new();
    for path in entries.flatten().map(|e| e.path()) {
        if path.extension().is_none_or(|x| x != "jsonl") {
            continue;
        }
        let lines = journal::read(&path).map_err(|e| e.to_string())?;
        let set_aside = path
            .file_stem()
            .and_then(|s| Path::new(s).extension())
            .is_some_and(|x| x.to_string_lossy().chars().all(|c| c.is_ascii_digit()));
        runs.extend(chains(&path, set_aside, &lines));
    }
    runs.sort_by_key(|r| std::cmp::Reverse(r.started));
    Ok(runs)
}

/// The chains of one journal.
pub fn chains(path: &Path, set_aside: bool, lines: &[Line]) -> Vec<RunEntry> {
    let starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| matches!(l.record, Record::Run { resume: false, .. }))
        .map(|(i, _)| i)
        .collect();
    let mut out = Vec::new();
    for (n, &start) in starts.iter().enumerate() {
        let end = starts.get(n + 1).copied().unwrap_or(lines.len());
        let chain = &lines[start..end];
        let Some(analysis) = analyse(chain) else {
            continue;
        };
        let mut entry = RunEntry {
            journal: path.to_path_buf(),
            set_aside,
            last: n + 1 == starts.len(),
            plan: analysis.run.plan.clone(),
            started: chain[0].ms,
            last_ms: chain.last().map_or(0, |l| l.ms),
            end: analysis.end,
            reason: None,
            auto: analysis.run.auto.clone(),
            head: None,
            resumes: 0,
            records: chain.len(),
            tasks: Vec::new(),
        };
        let mut started_at: Vec<(String, u64)> = Vec::new();
        for line in chain {
            match &line.record {
                Record::Run { resume: true, .. } => entry.resumes += 1,
                Record::Run { head, .. } => entry.head.clone_from(&head.commit),
                Record::TaskStart { task, .. } => {
                    started_at.push((task.clone(), line.ms));
                    if !entry.tasks.iter().any(|t| &t.id == task) {
                        entry.tasks.push(TaskSeen {
                            id: task.clone(),
                            commit: None,
                            took: None,
                        });
                    }
                }
                Record::TaskEnd { task, head } => {
                    let since = started_at
                        .iter()
                        .rev()
                        .find(|(t, _)| t == task)
                        .map(|(_, ms)| *ms);
                    if let Some(t) = entry.tasks.iter_mut().find(|t| &t.id == task) {
                        t.commit.clone_from(&head.commit);
                        t.took = since.map(|s| line.ms.saturating_sub(s));
                    }
                }
                Record::RunEnd { reason, .. } => entry.reason.clone_from(reason),
                _ => {}
            }
        }
        out.push(entry);
    }
    out
}

/// The History view.
#[derive(Debug, Clone, Default)]
pub struct History {
    /// The runs, newest first; `None` until scanned.
    pub runs: Option<Result<Vec<RunEntry>, String>>,
    /// The selected run.
    pub selected: usize,
}

/// What a key did in History.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Did {
    /// Nothing for the app.
    None,
    /// Open the resume preview of the selected run.
    Preview(RunEntry),
    /// Back.
    Back,
}

impl History {
    /// The selected run.
    pub fn current(&self) -> Option<&RunEntry> {
        self.runs.as_ref()?.as_ref().ok()?.get(self.selected)
    }

    /// Handles `key`.
    pub fn key(&mut self, key: Key) -> Did {
        let n = self
            .runs
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .map_or(0, Vec::len);
        match key {
            Key::Up | Key::Char('k') => self.selected = self.selected.saturating_sub(1),
            Key::Down | Key::Char('j') => {
                self.selected = (self.selected + 1).min(n.saturating_sub(1))
            }
            Key::Char('R') => {
                if let Some(run) = self.current().filter(|r| r.resumable()) {
                    return Did::Preview(run.clone());
                }
            }
            Key::Esc | Key::Char('q') => return Did::Back,
            _ => {}
        }
        Did::None
    }

    /// The keys the footer shows.
    pub fn footer(&self) -> Vec<(String, String)> {
        vec![
            ("R".into(), "resume preview…".into()),
            ("↑↓".into(), "select".into()),
            ("esc".into(), "back".into()),
        ]
    }

    /// The header's right side.
    pub fn state(&self, ctx_journals: &str) -> Vec<(String, Style)> {
        let n = self
            .runs
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .map_or(0, Vec::len);
        let journals: std::collections::BTreeSet<&PathBuf> = self
            .runs
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .map(|r| r.iter().map(|e| &e.journal).collect())
            .unwrap_or_default();
        vec![
            (
                format!("{n} runs, {} journals in ", journals.len()),
                on(Token::Fg3, Token::Bg1),
            ),
            (ctx_journals.to_owned(), on(Token::Fg2, Token::Bg1)),
        ]
    }

    /// Draws History in `area`; `live` is the pid of a live run, if one is live.
    pub fn draw(&self, buf: &mut Buffer, area: Rect, live: Option<Option<u32>>, g: Glyphs) {
        let split = area.x + 58;
        let list = Rect::new(area.x, area.y, split - area.x, area.height);
        let detail = Rect::new(split + 1, area.y, area.right() - split - 1, area.height);
        crate::draw::vrule(buf, split, area.y, area.bottom(), g);
        band(buf, list, true, "RUNS", "newest first, dates in UTC", g);
        let runs = match &self.runs {
            None => {
                Pen::new(buf, list.x + 2, list.y + 2, list.right(), g).text(
                    &format!("{} reading the journals", g.get(Glyph::Background)),
                    fg(Token::Fg2),
                );
                return;
            }
            Some(Err(e)) => {
                Pen::new(buf, list.x + 2, list.y + 2, list.right(), g).text(e, fg(Token::Err));
                return;
            }
            Some(Ok(runs)) => runs,
        };
        if runs.is_empty() {
            Pen::new(buf, list.x + 2, list.y + 2, list.right(), g)
                .text("no journal yet: nothing has run here", fg(Token::Fg2));
        }
        let mut y = list.y + 2;
        let mut day = None;
        for (i, run) in runs.iter().enumerate() {
            if y + 1 >= list.bottom() {
                break;
            }
            let (yy, mm, dd) = civil(run.started);
            if day != Some((yy, mm, dd)) {
                if day.is_some() {
                    y += 1;
                }
                day = Some((yy, mm, dd));
                let label = format!("{dd} {} {yy}", MONTHS[mm - 1].to_uppercase());
                Pen::new(buf, list.x + 2, y, list.right(), g).text(&label, bold(Token::Fg3));
                y += 1;
            }
            let selected = i == self.selected;
            if selected {
                select_row(buf, list, y, g);
            }
            draw_run(buf, list, y, run, live, selected, g);
            y += 1;
        }
        if let Some(run) = self.current() {
            draw_detail(buf, detail, run, live, g);
        }
    }
}

/// The run's end: glyph, word, colour.
pub fn ending(run: &RunEntry, live: Option<Option<u32>>) -> (Glyph, String, Token) {
    if run.set_aside && run.end != Some(End::Completed) {
        return (Glyph::Skipped, "set aside".into(), Token::Fg3);
    }
    match run.end {
        Some(End::Completed) => (Glyph::Done, "completed".into(), Token::Ok),
        Some(End::Stopped) => (Glyph::Failed, "stopped".into(), Token::Err),
        None if run.last && live.is_some() => {
            let pid = live
                .flatten()
                .map_or_else(|| "running".to_owned(), |p| format!("running in pid {p}"));
            (Glyph::Running, pid, Token::Act)
        }
        None => (Glyph::Waiting, "killed".into(), Token::Warn),
    }
}

fn draw_run(
    buf: &mut Buffer,
    area: Rect,
    y: u16,
    run: &RunEntry,
    live: Option<Option<u32>>,
    selected: bool,
    g: Glyphs,
) {
    let bg = if selected { Token::Bg2 } else { Token::Bg0 };
    let on_bg = |t: Token| fg(t).bg(bg.color());
    let (glyph, word, color) = ending(run, live);
    let mut pen = Pen::new(buf, area.x + 2, y, area.right().saturating_sub(1), g);
    pen.glyph(glyph, bold(color).bg(bg.color())).gap(1);
    let name = if selected {
        bold(Token::Fg).bg(bg.color())
    } else {
        on_bg(Token::Fg)
    };
    pen.cell(&run.name(), 25, name);
    pen.cell(
        &word,
        18,
        on_bg(if color == Token::Ok {
            Token::Fg2
        } else {
            color
        }),
    );
    pen.text(&clock_utc(run.started), on_bg(Token::Fg3)).gap(1);
    let done = run.tasks.iter().filter(|t| t.commit.is_some()).count();
    let detail = if run.set_aside {
        "--restart".to_owned()
    } else {
        format!("{done}/{} tasks", run.tasks.len())
    };
    pen.text(&detail, on_bg(Token::Fg2));
}

fn draw_detail(buf: &mut Buffer, area: Rect, run: &RunEntry, live: Option<Option<u32>>, g: Glyphs) {
    let band_area = Rect::new(area.x, area.y, area.width, 1);
    band(buf, band_area, false, "RUN", &run.name(), g);
    let x = area.x + 1;
    let right = area.right().saturating_sub(1);
    let mut y = area.y + 2;
    Pen::new(buf, x, y, right, g).text(&run.plan, bold(Token::Fg));
    y += 1;
    let (yy, mm, dd) = civil(run.started);
    let auto = if run.auto.is_empty() {
        "policy default".to_owned()
    } else {
        format!("auto {}", run.auto.join(","))
    };
    let mut pen = Pen::new(buf, x, y, right, g);
    pen.cell("started", 9, fg(Token::Fg3));
    pen.text(
        &format!(
            "{dd} {} {yy} {} UTC {} {auto}",
            MONTHS[mm - 1],
            clock_utc(run.started),
            g.get(Glyph::Dot)
        ),
        fg(Token::Fg2),
    );
    if let Some(head) = &run.head {
        pen.text(&format!(" {} HEAD ", g.get(Glyph::Dot)), fg(Token::Fg2))
            .text(&head[..head.len().min(7)], fg(Token::Meta));
    }
    y += 1;
    let (glyph, word, color) = ending(run, live);
    let mut pen = Pen::new(buf, x, y, right, g);
    pen.cell("ended", 9, fg(Token::Fg3));
    let why = match (&run.end, &run.reason) {
        (None, _) if word == "killed" => {
            "no run-end, no live run: the process was killed".to_owned()
        }
        (_, Some(reason)) => format!("{word}: {reason}"),
        _ => word.clone(),
    };
    pen.glyph(glyph, fg(color)).gap(1).text(&why, fg(color));
    y += 1;
    let mut pen = Pen::new(buf, x, y, right, g);
    pen.cell("journal", 9, fg(Token::Fg3));
    let file = run
        .journal
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let resumes = if run.resumes == 0 {
        String::new()
    } else {
        format!(
            " {} {} resume{}",
            g.get(Glyph::Dot),
            run.resumes,
            if run.resumes == 1 { "" } else { "s" }
        )
    };
    pen.text(
        &format!(
            "{file} {} {} records{resumes}",
            g.get(Glyph::Dot),
            run.records
        ),
        fg(Token::Fg3),
    );
    y += 2;
    let width = Rect::new(x, area.y, right - x, area.height);
    section(buf, width, y, "TASKS", &[], g);
    y += 1;
    for task in &run.tasks {
        if y + 6 >= area.bottom() {
            break;
        }
        let mut pen = Pen::new(buf, x, y, right, g);
        match &task.commit {
            Some(commit) => {
                pen.glyph(Glyph::Done, bold(Token::Ok))
                    .gap(1)
                    .text(&task.id, bold(Token::Fg))
                    .gap(2);
                pen.glyph(Glyph::Commit, fg(Token::Fg2))
                    .gap(1)
                    .text(&commit[..commit.len().min(7)], fg(Token::Meta));
                if let Some(took) = task.took {
                    pen.gap(2)
                        .text(&crate::text::duration(took), fg(Token::Fg3));
                }
            }
            None => {
                pen.glyph(Glyph::Waiting, bold(Token::Warn))
                    .gap(1)
                    .text(&task.id, bold(Token::Fg))
                    .gap(2)
                    .text("did not end", fg(Token::Warn));
            }
        }
        y += 1;
    }
    y += 1;
    section(buf, width, y, "RESUME", &[], g);
    let text = if run.resumable() {
        [
            "R opens the resume preview. It reads only; resume itself",
            "takes the lock, checks again and asks in the dock.",
        ]
    } else if run.set_aside {
        ["set aside by --restart: nothing resumes it", ""]
    } else {
        ["nothing to resume", ""]
    };
    for (i, l) in text.iter().enumerate() {
        Pen::new(buf, x, y + 1 + u16::try_from(i).unwrap_or(0), right, g).text(l, fg(Token::Fg2));
    }
}
