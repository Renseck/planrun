//! The Plans view (docs/tui-design.md §6.A, decision D87): a table of the plan files,
//! `PLAN · FORMAT · TASKS · CHECK · RUN`, and the selected plan's detail: its outline with a
//! coverage strip per task, the check's diagnostics, the journal's status, and a word on
//! preflight, which is never a background check.

use std::path::PathBuf;
use std::sync::Arc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use planrun_model::ActionKind;
use planrun_parse::{Parsed, TaskCoverage};

use crate::draw::{Pen, band, hrule, section, select_row};
use crate::glyph::{Glyph, Glyphs};
use crate::jobs::{Bg, Job, PlanFile, RunState, Stamp};
use crate::keys::Key;
use crate::text;
use crate::theme::{Token, bold, fg};

/// The detail panel's width, its hairline included.
const DETAIL: u16 = 48;

/// One plan file.
#[derive(Debug, Clone)]
pub struct Entry {
    /// Its path.
    pub path: PathBuf,
    /// Its file name.
    pub name: String,
    /// Its stamp as last scanned.
    pub stamp: Stamp,
    /// Its check.
    pub check: Bg<PlanFile>,
    /// The stamp of the check in `check`.
    pub checked: Option<Stamp>,
}

/// What the Plans view holds.
#[derive(Debug, Clone, Default)]
pub struct Plans {
    /// The files, by name.
    pub entries: Vec<Entry>,
    /// The selected entry, among the shown ones.
    pub selected: usize,
    /// The filter: `Some` while it is typed or applied.
    pub filter: Option<String>,
    /// Typing the filter.
    pub editing: bool,
    /// Why the directory could not be listed.
    pub error: Option<String>,
}

/// What a key did in the Plans view.
pub enum Outcome {
    /// Nothing for the app.
    None,
    /// Open the selected plan in Inspect.
    Inspect(PathBuf),
    /// `R`: open the selected plan in Inspect to run it from there; a run is always preceded by
    /// a visible dry run and policy (D84).
    Run(PathBuf),
    /// Check these files again.
    Check(Vec<Job>),
    /// The key is not the view's.
    Unused,
}

impl Plans {
    /// The shown entries: all, or those whose name holds the filter.
    pub fn shown(&self) -> Vec<&Entry> {
        let filter = self.filter.as_deref().unwrap_or("").to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.name.to_lowercase().contains(&filter))
            .collect()
    }

    /// The selected entry.
    pub fn current(&self) -> Option<&Entry> {
        self.shown().get(self.selected).copied()
    }

    /// The scan's result: new and changed files are checked (again); a changed file's check is
    /// stale until then.
    pub fn scanned(&mut self, files: Result<Vec<(PathBuf, Stamp)>, String>) -> Vec<Job> {
        let files = match files {
            Ok(files) => files,
            Err(e) => {
                self.error = Some(e);
                self.entries.clear();
                return Vec::new();
            }
        };
        self.error = None;
        let mut jobs = Vec::new();
        let mut entries = Vec::new();
        for (path, stamp) in files {
            let old = self.entries.iter().find(|e| e.path == path).cloned();
            let entry = match old {
                Some(mut e) if e.checked == Some(stamp) || e.stamp == stamp => {
                    e.stamp = stamp;
                    e
                }
                Some(mut e) => {
                    e.stamp = stamp;
                    e.check = e.check.stale();
                    jobs.push(Job::Check {
                        path: path.clone(),
                        stamp,
                    });
                    e
                }
                None => {
                    jobs.push(Job::Check {
                        path: path.clone(),
                        stamp,
                    });
                    Entry {
                        name: path
                            .file_name()
                            .map_or_else(String::new, |n| n.to_string_lossy().into_owned()),
                        path,
                        stamp,
                        check: Bg::Running,
                        checked: None,
                    }
                }
            };
            entries.push(entry);
        }
        self.entries = entries;
        self.selected = self.selected.min(self.shown().len().saturating_sub(1));
        jobs
    }

    /// A check's result; one for a stamp that has changed since stays stale.
    pub fn checked(&mut self, path: &PathBuf, stamp: Stamp, file: Result<PlanFile, String>) {
        let Some(entry) = self.entries.iter_mut().find(|e| &e.path == path) else {
            return;
        };
        entry.checked = Some(stamp);
        entry.check = match file {
            Ok(file) if entry.stamp == stamp => Bg::Current(file),
            Ok(file) => Bg::Stale(file),
            Err(e) => Bg::Failed(e),
        };
    }

    /// The parsed plan of `path`, when its check found one.
    pub fn parsed(&self, path: &PathBuf) -> Option<Arc<Parsed>> {
        let entry = self.entries.iter().find(|e| &e.path == path)?;
        entry.check.value()?.parsed.clone()
    }

    /// How many shown plans have a journal without `run-end`.
    pub fn interrupted(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e.check.value().map(|f| &f.run), Some(RunState::Interrupted)))
            .count()
    }

    /// Handles `key`.
    pub fn key(&mut self, key: Key) -> Outcome {
        if self.editing {
            return self.filter_key(key);
        }
        let shown = self.shown().len();
        match key {
            Key::Up | Key::Char('k') => self.selected = self.selected.saturating_sub(1),
            Key::Down | Key::Char('j') => {
                self.selected = (self.selected + 1).min(shown.saturating_sub(1));
            }
            Key::Home => self.selected = 0,
            Key::End => self.selected = shown.saturating_sub(1),
            Key::Char('/') => {
                self.editing = true;
                self.filter.get_or_insert_with(String::new);
            }
            Key::Esc if self.filter.is_some() => {
                self.filter = None;
                self.selected = 0;
            }
            Key::Enter => return self.with_current(Outcome::Inspect),
            Key::Char('R') => return self.with_current(Outcome::Run),
            Key::Char('c') => {
                let jobs = self
                    .shown()
                    .iter()
                    .map(|e| Job::Check {
                        path: e.path.clone(),
                        stamp: e.stamp,
                    })
                    .collect();
                for e in &mut self.entries {
                    e.check = std::mem::replace(&mut e.check, Bg::Running).stale();
                }
                return Outcome::Check(jobs);
            }
            _ => return Outcome::Unused,
        }
        Outcome::None
    }

    fn with_current(&self, f: fn(PathBuf) -> Outcome) -> Outcome {
        match self.current() {
            Some(e) if e.check.value().is_some_and(|f| f.parsed.is_some()) => f(e.path.clone()),
            _ => Outcome::None,
        }
    }

    fn filter_key(&mut self, key: Key) -> Outcome {
        let filter = self.filter.get_or_insert_with(String::new);
        match key {
            Key::Char(c) => filter.push(c),
            Key::Backspace => {
                filter.pop();
            }
            Key::Enter => self.editing = false,
            Key::Esc => {
                self.editing = false;
                self.filter = None;
            }
            _ => return Outcome::None,
        }
        self.selected = 0;
        Outcome::None
    }

    /// The keys the footer shows.
    pub fn footer(&self) -> Vec<(String, String)> {
        if self.editing {
            return vec![
                ("enter".into(), "apply".into()),
                ("esc".into(), "clear".into()),
            ];
        }
        vec![
            ("enter".into(), "inspect".into()),
            ("R".into(), "run…".into()),
            ("c".into(), "re-check".into()),
            ("/".into(), "filter".into()),
            ("ctrl+k".into(), "commands".into()),
            ("?".into(), "help".into()),
        ]
    }

    /// Draws the view in `area` (the body between the header and the footer).
    pub fn draw(&self, buf: &mut Buffer, area: Rect, labels: &Labels, g: Glyphs) {
        let split = area.right().saturating_sub(DETAIL);
        let table = Rect::new(area.x, area.y, split - area.x, area.height);
        let detail = Rect::new(split + 2, area.y, area.right() - split - 2, area.height);
        crate::draw::vrule(buf, split, area.y, area.bottom(), g);
        self.draw_table(buf, table, labels, g);
        self.draw_detail(buf, detail, labels, g);
    }

    fn draw_table(&self, buf: &mut Buffer, area: Rect, labels: &Labels, g: Glyphs) {
        let count = self.entries.len();
        let what = format!(
            "{count} file{} in {}",
            if count == 1 { "" } else { "s" },
            labels.plans
        );
        band(buf, area, true, "PLANS", &what, g);
        let mut pen = Pen::new(buf, area.x, area.y, area.right().saturating_sub(1), g);
        match (&self.filter, self.editing) {
            (Some(f), true) => pen.right(
                &format!("/ {f}{}", g.get(Glyph::Caret)),
                fg(Token::Act).bg(Token::Bg1.color()),
            ),
            (Some(f), false) => pen.right(&format!("/ {f}"), fg(Token::Fg2).bg(Token::Bg1.color())),
            (None, _) => pen.right("/ filter", fg(Token::Fg3).bg(Token::Bg1.color())),
        };
        let y = area.y + 2;
        let head = bold(Token::Fg3);
        let mut pen = Pen::new(buf, area.x + 2, y, area.right(), g);
        pen.cell("PLAN", 25, head)
            .cell("FORMAT", 13, head)
            .cell("TASKS", 6, head)
            .cell("CHECK", 12, head)
            .text("RUN", head);
        if let Some(e) = &self.error {
            let mut pen = Pen::new(buf, area.x + 2, y + 1, area.right(), g);
            pen.text(e, fg(Token::Err));
        }
        let legend = area.bottom().saturating_sub(4);
        for (i, entry) in self.shown().iter().enumerate() {
            let row = y + 1 + u16::try_from(i).unwrap_or(u16::MAX);
            if row >= legend {
                break;
            }
            if i == self.selected {
                select_row(buf, area, row, g);
            }
            self.draw_entry(buf, area, row, entry, i == self.selected, g);
        }
        hrule(buf, area.x + 1, area.right().saturating_sub(1), legend, g);
        let mut pen = Pen::new(buf, area.x + 2, legend + 1, area.right(), g);
        let dim = fg(Token::Fg3);
        pen.text("check ", dim)
            .glyph(Glyph::Done, fg(Token::Ok))
            .text(" valid  ", dim)
            .glyph(Glyph::Warning, fg(Token::Warn))
            .text(" warnings  ", dim)
            .glyph(Glyph::Failed, fg(Token::Err))
            .text(" errors  ", dim)
            .glyph(Glyph::Background, fg(Token::Fg2))
            .text(" checking  grey ", dim)
            .glyph(Glyph::Done, dim)
            .text(" stale", dim);
        let mut pen = Pen::new(buf, area.x + 2, legend + 2, area.right(), g);
        pen.text("background checks on start and on file change ", dim)
            .glyph(Glyph::Dot, dim)
            .text(" ", dim)
            .text("c", bold(Token::Fg2))
            .text(" re-checks", dim);
    }

    fn draw_entry(
        &self,
        buf: &mut Buffer,
        area: Rect,
        y: u16,
        e: &Entry,
        selected: bool,
        g: Glyphs,
    ) {
        let bg = if selected { Token::Bg2 } else { Token::Bg0 };
        let on = |t: Token| fg(t).bg(bg.color());
        let name = if selected {
            bold(Token::Fg).bg(bg.color())
        } else {
            on(Token::Fg)
        };
        let mut pen = Pen::new(buf, area.x + 2, y, area.right(), g);
        pen.cell(&e.name, 25, name);
        let file = e.check.value();
        let parsed = file.and_then(|f| f.parsed.as_ref());
        let format = match (parsed, file) {
            (Some(p), _) => p.plan.format.0.to_string(),
            (None, Some(_)) | (None, None) => g.get(Glyph::HRule).to_owned(),
        };
        pen.cell(&format, 13, on(Token::Fg2));
        let tasks = parsed.map_or_else(
            || g.get(Glyph::HRule).to_owned(),
            |p| p.plan.tasks.len().to_string(),
        );
        pen.cell(&text::right(&tasks, 4), 6, on(Token::Fg2));
        let start = pen.x;
        let (glyph, word, style) = check_cell(&e.check);
        if let Some(glyph) = glyph {
            pen.glyph(glyph, on(style)).gap(1);
        }
        let word_style = match &e.check {
            Bg::Current(f) if f.counts().0 == 0 && f.counts().1 > 0 => on(Token::Warn),
            Bg::Current(f) if f.counts().0 > 0 && !f.not_a_plan => on(Token::Err),
            Bg::Running => on(Token::Fg2),
            _ => on(Token::Fg2),
        };
        pen.text(&word, word_style);
        pen.to(start + 12);
        let run = file.filter(|f| !f.not_a_plan).map(|f| &f.run);
        let (glyph, word, style) = run_cell(run, g);
        if let Some(glyph) = glyph {
            pen.glyph(glyph, on(style)).gap(1);
        }
        pen.text(
            &word,
            on(if style == Token::Ok {
                Token::Fg2
            } else {
                style
            }),
        );
    }

    fn draw_detail(&self, buf: &mut Buffer, area: Rect, labels: &Labels, g: Glyphs) {
        let name = self.current().map_or("", |e| e.name.as_str());
        let band_area = Rect::new(area.x - 1, area.y, area.width + 1, 1);
        band(buf, band_area, false, "PLAN", name, g);
        let Some(entry) = self.current() else {
            return;
        };
        let right = area.right().saturating_sub(1);
        let mut y = area.y + 2;
        let file = entry.check.value();
        let parsed = file.and_then(|f| f.parsed.as_ref());
        if let Some(p) = parsed {
            let plan = &p.plan;
            let title = plan
                .meta
                .title
                .as_ref()
                .map_or(entry.name.as_str(), |t| plan.text(t));
            Pen::new(buf, area.x, y, right, g).text(title, bold(Token::Fg));
            y += 1;
            let steps: usize = plan.tasks.iter().map(|t| t.steps.len()).sum();
            let actions: usize = plan
                .tasks
                .iter()
                .flat_map(|t| t.steps.iter())
                .map(|s| {
                    s.actions
                        .iter()
                        .filter(|a| !matches!(a.kind, ActionKind::Note { .. }))
                        .count()
                })
                .sum();
            let dot = g.get(Glyph::Dot);
            let line = format!(
                "{} {dot} {} tasks {dot} {steps} steps {dot} {actions} actions",
                plan.format.0,
                plan.tasks.len()
            );
            Pen::new(buf, area.x, y, right, g).text(&line, fg(Token::Fg2));
            y += 1;
        }
        if let Some(f) = file {
            let dot = g.get(Glyph::Dot);
            let rel = labels.relative(&entry.path);
            let line = format!(
                "{rel} {dot} {} lines {dot} {}",
                f.lines,
                clock_utc(entry.stamp.modified)
            );
            Pen::new(buf, area.x, y, right, g).text(&line, fg(Token::Fg3));
        }
        y += 2;
        if let Some(p) = parsed {
            y = draw_outline(buf, area, y, p, g);
        }
        y = self.draw_check(buf, area, y, entry, g);
        y = draw_status(buf, area, y, file.map(|f| &f.run), labels, g);
        section(
            buf,
            Rect::new(area.x, area.y, right - area.x, area.height),
            y,
            "PREFLIGHT",
            &[],
            g,
        );
        Pen::new(buf, area.x, y + 1, right, g)
            .text("never a background check: it runs right", fg(Token::Fg2));
        Pen::new(buf, area.x, y + 2, right, g)
            .text("before each task, and in every dry run", fg(Token::Fg2));
        if parsed.is_some() && y + 4 < area.bottom() {
            Pen::new(buf, area.x, y + 4, right, g).chip(
                "enter",
                "inspect with a dry run",
                Token::Bg0,
            );
        }
    }

    fn draw_check(&self, buf: &mut Buffer, area: Rect, y: u16, entry: &Entry, g: Glyphs) -> u16 {
        let right = area.right().saturating_sub(1);
        let width = Rect::new(area.x, area.y, right - area.x, area.height);
        let (glyph, word, style) = check_cell(&entry.check);
        let mut tail = Vec::new();
        if let Some(glyph) = glyph {
            tail.push((format!("{} ", g.get(glyph)), fg(style)));
        }
        tail.push((word, fg(Token::Fg2)));
        if let Some(s) = entry.checked {
            tail.push((
                format!(" {} {}", g.get(Glyph::Dot), clock_utc(s.modified)),
                fg(Token::Fg3),
            ));
        }
        section(buf, width, y, "CHECK", &tail, g);
        let mut y = y + 1;
        let Some(file) = entry.check.value() else {
            return y + 1;
        };
        let limit = area.bottom().saturating_sub(12);
        for d in file.diags.iter().take(4) {
            if y >= limit {
                break;
            }
            let (glyph, style) = if d.error {
                (Glyph::Failed, Token::Err)
            } else {
                (Glyph::Warning, Token::Warn)
            };
            let mut pen = Pen::new(buf, area.x, y, right, g);
            pen.glyph(glyph, fg(style))
                .gap(1)
                .text(d.code, fg(Token::Fg));
            if let Some(line) = d.line {
                pen.gap(2).text(&format!("line {line}"), fg(Token::Meta));
            }
            Pen::new(buf, area.x + 2, y + 1, right, g).text(&d.message, fg(Token::Fg2));
            y += 2;
        }
        if let Some(p) = &file.parsed {
            let manual: usize = p.coverage.iter().map(|c| c.manual).sum();
            if manual > 0 && y < limit {
                let mut pen = Pen::new(buf, area.x, y, right, g);
                let s = if manual == 1 { "" } else { "s" };
                pen.glyph(Glyph::Manual, fg(Token::Warn))
                    .gap(1)
                    .text(&format!("{manual} manual action{s}"), fg(Token::Fg));
                Pen::new(buf, area.x + 2, y + 1, right, g)
                    .text("planrun waits for you there", fg(Token::Fg2));
                y += 2;
            }
        }
        y + 1
    }
}

/// The labels the views show for paths.
#[derive(Debug, Clone)]
pub struct Labels {
    /// The plans directory, as the footer shows it.
    pub plans: String,
    /// The repository root, to show paths relative to it.
    pub root: PathBuf,
}

impl Labels {
    /// `path` relative to the root with forward slashes, or as it is.
    pub fn relative(&self, path: &std::path::Path) -> String {
        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        match canonical.strip_prefix(&self.root) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            ),
        }
    }
}

fn draw_outline(buf: &mut Buffer, area: Rect, y: u16, p: &Parsed, g: Glyphs) -> u16 {
    let right = area.right().saturating_sub(1);
    let width = Rect::new(area.x, area.y, right - area.x, area.height);
    section(
        buf,
        width,
        y,
        "OUTLINE",
        &[("coverage".into(), fg(Token::Fg3))],
        g,
    );
    let mut y = y + 1;
    let strip_x = right.saturating_sub(16);
    let rows = usize::from(area.bottom().saturating_sub(y + 16)).max(2);
    for (task, cov) in p.plan.tasks.iter().zip(&p.coverage).take(rows) {
        let mut pen = Pen::new(buf, area.x, y, strip_x.saturating_sub(1), g);
        pen.text(&task.id.raw, fg(Token::Meta))
            .gap(2)
            .text(p.plan.text(&task.title), fg(Token::Fg));
        let mut pen = Pen::new(buf, strip_x, y, right, g);
        for (cell, glyph, token) in strip(cov) {
            pen.text(&g.get(glyph).repeat(cell), fg(token));
        }
        y += 1;
    }
    let mut pen = Pen::new(buf, area.x, y, area.right(), g);
    let dim = fg(Token::Fg3);
    pen.glyph(Glyph::CovAction, fg(Token::Fg2))
        .text(" action ", dim)
        .glyph(Glyph::CovAbsorbed, fg(Token::Meta))
        .text(" absorbed ", dim)
        .glyph(Glyph::CovManifest, dim)
        .text(" manifest ", dim)
        .glyph(Glyph::CovNote, dim)
        .text(" note ", dim)
        .glyph(Glyph::CovAction, fg(Token::Warn))
        .text(" manual", dim);
    y + 2
}

/// The coverage strip of a task: 16 cells shared out by block count (largest remainder), in the
/// order action, absorbed, manifest, note, manual; a non-zero manual count is at least one cell.
pub fn strip(cov: &TaskCoverage) -> Vec<(usize, Glyph, Token)> {
    let parts = [
        (cov.actions, Glyph::CovAction, Token::Fg2),
        (cov.absorbed.len(), Glyph::CovAbsorbed, Token::Meta),
        (cov.manifest, Glyph::CovManifest, Token::Fg3),
        (cov.notes, Glyph::CovNote, Token::Fg3),
        (cov.manual, Glyph::CovAction, Token::Warn),
    ];
    let total: usize = parts.iter().map(|p| p.0).sum();
    if total == 0 {
        return vec![(16, Glyph::CovNote, Token::Rule)];
    }
    let reserved = usize::from(cov.manual > 0);
    let cells = 16 - reserved;
    let mut shares: Vec<(usize, usize)> = parts
        .iter()
        .map(|p| (p.0 * cells / total, p.0 * cells % total))
        .collect();
    let mut left = cells - shares.iter().map(|s| s.0).sum::<usize>();
    while left > 0 {
        let Some(best) = (0..shares.len())
            .filter(|&i| parts[i].0 > 0)
            .max_by_key(|&i| (shares[i].1, std::cmp::Reverse(i)))
        else {
            break;
        };
        shares[best].0 += 1;
        shares[best].1 = 0;
        left -= 1;
    }
    shares[4].0 += reserved;
    parts
        .iter()
        .zip(shares)
        .filter(|(_, s)| s.0 > 0)
        .map(|(p, s)| (s.0, p.1, p.2))
        .collect()
}

fn draw_status(
    buf: &mut Buffer,
    area: Rect,
    y: u16,
    run: Option<&RunState>,
    labels: &Labels,
    g: Glyphs,
) -> u16 {
    let right = area.right().saturating_sub(1);
    let width = Rect::new(area.x, area.y, right - area.x, area.height);
    let (glyph, word, style) = run_cell(run, g);
    let mut tail = Vec::new();
    if let Some(glyph) = glyph {
        tail.push((format!("{} ", g.get(glyph)), fg(style)));
    }
    tail.push((word, fg(Token::Fg2)));
    section(buf, width, y, "STATUS", &tail, g);
    let text = match run {
        None | Some(RunState::NotRun) => {
            format!("no journal in {} for this plan", labels_journal(labels))
        }
        Some(RunState::Done(_)) => "the last run completed".to_owned(),
        Some(RunState::Stopped) => "the last run stopped: resume goes on (F4)".to_owned(),
        Some(RunState::Interrupted) => "no run-end: killed, or running now (F4)".to_owned(),
        Some(RunState::Foreign(other)) => format!("its journal belongs to {other}"),
        Some(RunState::Unreadable(e)) => e.clone(),
    };
    Pen::new(buf, area.x, y + 1, right, g).text(&text, fg(Token::Fg2));
    y + 3
}

fn labels_journal(_labels: &Labels) -> &'static str {
    ".git/planrun"
}

/// The CHECK column: glyph, word and the glyph's colour.
fn check_cell(check: &Bg<PlanFile>) -> (Option<Glyph>, String, Token) {
    match check {
        Bg::Running => (Some(Glyph::Background), "checking".into(), Token::Fg2),
        Bg::Stale(_) => (Some(Glyph::Done), "stale".into(), Token::Fg3),
        Bg::Unavailable(e) | Bg::Failed(e) => (Some(Glyph::Failed), fit_word(e), Token::Err),
        Bg::Current(f) if f.not_a_plan => (None, "not a plan".into(), Token::Fg3),
        Bg::Current(f) => match f.counts() {
            (0, 0) => (Some(Glyph::Done), "valid".into(), Token::Ok),
            (0, w) => (Some(Glyph::Done), format!("{w} warn"), Token::Ok),
            (1, _) => (Some(Glyph::Failed), "1 error".into(), Token::Err),
            (e, _) => (Some(Glyph::Failed), format!("{e} errors"), Token::Err),
        },
    }
}

fn fit_word(e: &str) -> String {
    if e.len() > 9 {
        "unreadable".into()
    } else {
        e.to_owned()
    }
}

/// The RUN column: glyph, word and the glyph's colour.
fn run_cell(run: Option<&RunState>, g: Glyphs) -> (Option<Glyph>, String, Token) {
    match run {
        None => (None, g.get(Glyph::HRule).to_owned(), Token::Fg3),
        Some(RunState::NotRun) => (Some(Glyph::Pending), "not run".into(), Token::Fg3),
        Some(RunState::Done(ms)) => (
            Some(Glyph::Done),
            format!("done {}", date_utc(*ms)),
            Token::Ok,
        ),
        Some(RunState::Stopped) => (Some(Glyph::Failed), "stopped".into(), Token::Err),
        Some(RunState::Interrupted) => (Some(Glyph::Waiting), "interrupted".into(), Token::Warn),
        Some(RunState::Foreign(_)) => (Some(Glyph::Failed), "foreign".into(), Token::Err),
        Some(RunState::Unreadable(_)) => (Some(Glyph::Failed), "unreadable".into(), Token::Err),
    }
}

/// `HH:MM` in UTC (std has no time zone; decision D94).
pub fn clock_utc(ms: u64) -> String {
    let secs = ms / 1000;
    format!("{:02}:{:02}", (secs / 3600) % 24, (secs / 60) % 60)
}

/// `25 Nov` in UTC.
pub fn date_utc(ms: u64) -> String {
    let (_, month, day) = civil(ms);
    format!("{day} {}", MONTHS[month - 1])
}

/// The months, short.
pub const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// The UTC date of `ms`: year, month (1–12), day (1–31). Howard Hinnant's days-to-civil.
pub fn civil(ms: u64) -> (i64, usize, u64) {
    let days = i64::try_from(ms / 86_400_000).unwrap_or(0);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (
        year,
        usize::try_from(month).unwrap_or(1),
        u64::try_from(day).unwrap_or(1),
    )
}

/// A style for `token` on the selection's background when `selected`.
pub fn row_style(token: Token, selected: bool) -> Style {
    let bg = if selected { Token::Bg2 } else { Token::Bg0 };
    fg(token).bg(bg.color())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cov(actions: usize, manifest: usize, notes: usize, manual: usize) -> TaskCoverage {
        TaskCoverage {
            task: planrun_model::TaskId {
                raw: "P1-T1".into(),
                ordinal: 1,
            },
            actions,
            absorbed: Vec::new(),
            manifest,
            notes,
            manual,
            dropped: 0,
        }
    }

    #[test]
    fn the_strip_has_sixteen_cells_and_a_manual_cell_when_there_is_one() {
        for c in [
            cov(10, 3, 2, 0),
            cov(30, 1, 1, 1),
            cov(1, 0, 0, 0),
            cov(5, 5, 5, 5),
        ] {
            let cells: usize = strip(&c).iter().map(|s| s.0).sum();
            assert_eq!(cells, 16);
        }
        let s = strip(&cov(30, 1, 1, 1));
        assert_eq!(s.last().map(|p| (p.0 >= 1, p.2)), Some((true, Token::Warn)));
        assert_eq!(
            strip(&cov(1, 0, 0, 0)),
            vec![(16, Glyph::CovAction, Token::Fg2)]
        );
    }

    #[test]
    fn dates_and_times_are_utc() {
        assert_eq!(civil(0), (1970, 1, 1));
        // 2026-11-27 12:34:56 UTC
        let ms = 1_795_782_896_000;
        assert_eq!(civil(ms), (2026, 11, 27));
        assert_eq!(date_utc(ms), "27 Nov");
        assert_eq!(clock_utc(ms), "12:34");
    }
}
