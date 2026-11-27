//! The plan view in Inspect mode (docs/tui-design.md §6.B, decision D84): filled from a dry run
//! (D55) and the assumed policy. ACTION shows the selected action's class and reasons, the
//! policy's decision, the dry run's verdict, its plan line and its planned diff; DRY RUN lists
//! every verdict, marked advisory: the real preflight decides at run time. `a` changes the
//! assumed policy, and `R` runs with exactly that policy.

use std::path::PathBuf;
use std::sync::Arc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use planrun_exec::finding::{Finding, Severity};
use planrun_exec::policy::{Decision, DefaultClassifier, Policy, PolicyClass};
use planrun_exec::preflight::{ActionReport, DryRun, Outcome};
use planrun_model::{Action, ActionKind, PathOrigin, Plan};
use planrun_parse::Parsed;

use crate::diff::{self, FileDiff};
use crate::draw::{Pen, band};
use crate::glyph::{Glyph, Glyphs};
use crate::jobs::Bg;
use crate::keys::Key;
use crate::theme::{Token, bold, fg, on};
use crate::tree::{self, Mark, Node, Tree};

/// The panel with focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    /// The navigator.
    #[default]
    Tree,
    /// ACTION.
    Action,
    /// DRY RUN.
    DryRun,
}

/// The policy Inspect assumes, and `R` runs with (D84).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Assumed {
    /// `--auto`.
    pub auto: Vec<PolicyClass>,
    /// `--run-expected-failures`.
    pub expected: bool,
}

impl Assumed {
    /// The policy; Destructive is never among `auto`, so it always builds.
    pub fn policy(&self) -> Policy {
        Policy::new(&self.auto, self.expected).unwrap_or_default()
    }

    /// `default`, or `auto read,build` and `expected failures run`.
    pub fn describe(&self) -> String {
        if self.auto.is_empty() && !self.expected {
            return "default".to_owned();
        }
        let mut parts = Vec::new();
        if !self.auto.is_empty() {
            parts.push(format!("auto {}", self.auto_words()));
        }
        if self.expected {
            parts.push("expected failures run".to_owned());
        }
        parts.join(" · ")
    }

    /// `read,build`, as `--auto` takes it.
    pub fn auto_words(&self) -> String {
        let words: Vec<&str> = self.auto.iter().map(|c| class_word(*c)).collect();
        words.join(",")
    }

    fn toggle(&mut self, class: PolicyClass) {
        if let Some(i) = self.auto.iter().position(|c| *c == class) {
            self.auto.remove(i);
        } else {
            self.auto.push(class);
            self.auto.sort();
        }
    }
}

/// A class as `--auto` spells it.
pub fn class_word(class: PolicyClass) -> &'static str {
    match class {
        PolicyClass::Read => "read",
        PolicyClass::Build => "build",
        PolicyClass::Modify => "modify",
        PolicyClass::Destructive => "destructive",
    }
}

/// A class as the panels name it.
pub fn class_name(class: PolicyClass) -> &'static str {
    match class {
        PolicyClass::Read => "Read",
        PolicyClass::Build => "Build",
        PolicyClass::Modify => "Modify",
        PolicyClass::Destructive => "Destructive",
    }
}

/// What a key did in Inspect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Did {
    /// Nothing for the app.
    None,
    /// Run with the assumed policy, from this task when `Some`.
    Run(Option<usize>),
    /// Back to Plans.
    Back,
    /// Not Inspect's key.
    Unused,
}

/// The Inspect state of one plan.
#[derive(Debug, Clone)]
pub struct Inspect {
    /// The plan file.
    pub path: PathBuf,
    /// The parsed plan.
    pub parsed: Arc<Parsed>,
    /// The assumed policy.
    pub assumed: Assumed,
    /// The dry run.
    pub dry: Bg<DryRun>,
    /// The navigator.
    pub tree: Tree,
    /// The panel with focus.
    pub focus: Focus,
    /// The first row of ACTION's body shown.
    pub action_top: usize,
    /// The first row of DRY RUN shown.
    pub dry_top: usize,
    /// The policy picker is open.
    pub picking: bool,
}

impl Inspect {
    /// Inspect of `parsed`, its dry run still running.
    pub fn new(path: PathBuf, parsed: Arc<Parsed>) -> Inspect {
        Inspect {
            path,
            parsed,
            assumed: Assumed::default(),
            dry: Bg::Running,
            tree: Tree::default(),
            focus: Focus::Tree,
            action_top: 0,
            dry_top: 0,
            picking: false,
        }
    }

    /// The plan.
    pub fn plan(&self) -> &Plan {
        &self.parsed.plan
    }

    /// Handles `key`.
    pub fn key(&mut self, key: Key) -> Did {
        if self.picking {
            match key {
                Key::Char('r') => self.assumed.toggle(PolicyClass::Read),
                Key::Char('b') => self.assumed.toggle(PolicyClass::Build),
                Key::Char('m') => self.assumed.toggle(PolicyClass::Modify),
                Key::Char('e') => self.assumed.expected = !self.assumed.expected,
                Key::Enter | Key::Esc | Key::Char('a') => self.picking = false,
                _ => {}
            }
            return Did::None;
        }
        match key {
            Key::Tab => {
                self.focus = match self.focus {
                    Focus::Tree => Focus::Action,
                    Focus::Action => Focus::DryRun,
                    Focus::DryRun => Focus::Tree,
                };
                return Did::None;
            }
            Key::BackTab => {
                self.focus = match self.focus {
                    Focus::Tree => Focus::DryRun,
                    Focus::Action => Focus::Tree,
                    Focus::DryRun => Focus::Action,
                };
                return Did::None;
            }
            Key::Char('a') => {
                self.picking = true;
                return Did::None;
            }
            Key::Char('R') if self.dry.value().is_some() => {
                let from = self.tree.current(self.plan()).map(Node::task);
                return Did::Run(from.filter(|t| *t > 0));
            }
            Key::Esc => return Did::Back,
            _ => {}
        }
        let scroll = |top: &mut usize| match key {
            Key::Up | Key::Char('k') => *top = top.saturating_sub(1),
            Key::Down | Key::Char('j') => *top += 1,
            Key::PageUp => *top = top.saturating_sub(10),
            Key::PageDown => *top += 10,
            Key::Home => *top = 0,
            _ => {}
        };
        match self.focus {
            Focus::Tree => {
                let plan = self.parsed.plan.clone();
                if self.tree.key(key, &plan) {
                    self.action_top = 0;
                    return Did::None;
                }
                Did::Unused
            }
            Focus::Action => {
                scroll(&mut self.action_top);
                Did::None
            }
            Focus::DryRun => {
                scroll(&mut self.dry_top);
                Did::None
            }
        }
    }

    /// The report of the action at (`t`, `s`, `a`).
    fn report(&self, t: usize, s: usize, a: usize) -> Option<&ActionReport> {
        let dry = self.dry.value()?;
        let action = &self.plan().tasks.get(t)?.steps.get(s)?.actions.get(a)?;
        dry.tasks
            .get(t)?
            .actions
            .iter()
            .find(|r| r.step == s && r.action.span == action.span)
    }

    /// The decision for the action at (`t`, `s`, `a`) under the assumed policy.
    fn decision(&self, t: usize, s: usize, a: usize) -> Decision {
        let plan = self.plan();
        let task = &plan.tasks[t];
        self.assumed.policy().decide(
            task,
            &task.steps[s].actions[a],
            &plan.source,
            &DefaultClassifier,
        )
    }

    /// The navigator's mark of `node`.
    pub fn mark(&self, node: Node) -> Mark {
        match node {
            Node::Action(t, s, a) => {
                let (glyph, color) = self.status(t, s, a);
                let action = &self.plan().tasks[t].steps[s].actions[a];
                let (tag, tag_color) = tag(action, self.decision(t, s, a));
                Mark {
                    glyph,
                    color,
                    tag,
                    tag_color,
                }
            }
            Node::Step(t, s) => {
                let n = self.plan().tasks[t].steps[s].actions.len();
                let all: Vec<(Glyph, Token)> = (0..n).map(|a| self.status(t, s, a)).collect();
                let (glyph, color) = worst(&all);
                Mark::plain(glyph, color)
            }
            Node::Task(t) => {
                let task = &self.plan().tasks[t];
                let mut all: Vec<(Glyph, Token)> = Vec::new();
                for (s, step) in task.steps.iter().enumerate() {
                    all.extend((0..step.actions.len()).map(|a| self.status(t, s, a)));
                }
                if let Some(report) = self.dry.value().and_then(|d| d.tasks.get(t)) {
                    all.extend(report.manifest.iter().map(finding_mark));
                }
                let (glyph, color) = worst(&all);
                Mark::plain(glyph, color)
            }
        }
    }

    /// The status glyph of an action: the dry run's verdict, or ⊘ when the policy skips it.
    fn status(&self, t: usize, s: usize, a: usize) -> (Glyph, Token) {
        if matches!(self.decision(t, s, a), Decision::Skip(_)) {
            return (Glyph::Skipped, Token::Fg3);
        }
        match self.report(t, s, a).map(|r| &r.outcome) {
            None if self.dry.value().is_none() => (Glyph::Background, Token::Fg2),
            Some(Outcome::Ok(_)) => (Glyph::Done, Token::Ok),
            Some(Outcome::Failed(_)) => (Glyph::Failed, Token::Err),
            Some(Outcome::Depends(_)) => (Glyph::Warning, Token::Warn),
            _ => (Glyph::Dot, Token::Fg3),
        }
    }

    /// The header's right side.
    pub fn state(&self, g: Glyphs) -> Vec<(String, Style)> {
        let bar = |t: Token| on(t, Token::Bg1);
        let mut out = vec![
            (
                " INSPECT ".to_owned(),
                bold(Token::Meta).bg(Token::Bg1.color()),
            ),
            ("  policy ".to_owned(), bar(Token::Fg3)),
            (
                self.assumed.describe(),
                bold(Token::Fg).bg(Token::Bg1.color()),
            ),
            ("  dry run ".to_owned(), bar(Token::Fg3)),
        ];
        let Some(dry) = self.dry.value() else {
            out.push((
                format!("{} running", g.get(Glyph::Background)),
                bar(Token::Fg2),
            ));
            return out;
        };
        let (ok, warn, err) = counts(dry);
        out.push((format!("{} ", g.get(Glyph::Done)), bar(Token::Ok)));
        out.push((format!("{ok} ok"), bar(Token::Fg2)));
        if warn > 0 {
            out.push((format!("  {} ", g.get(Glyph::Warning)), bar(Token::Warn)));
            out.push((format!("{warn} warn"), bar(Token::Warn)));
        }
        if err > 0 {
            out.push((format!("  {} ", g.get(Glyph::Failed)), bar(Token::Err)));
            out.push((format!("{err} failed"), bar(Token::Err)));
        }
        out
    }

    /// The crumbs after the repository: the plan file, the task, the selected step or action.
    pub fn crumbs(&self, g: Glyphs) -> Vec<String> {
        let name = self
            .path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let mut out = vec![name];
        let plan = self.plan();
        let Some(node) = self.tree.current(plan) else {
            return out;
        };
        out.push(plan.tasks[node.task()].id.raw.to_string());
        match node {
            Node::Task(_) => {}
            Node::Step(t, s) => out.push(step_name(plan, t, s, g)),
            Node::Action(t, s, a) => {
                let action = &plan.tasks[t].steps[s].actions[a];
                out.push(format!(
                    "{} {}",
                    g.get(tree::kind_glyph(action)),
                    crumb_label(plan, action)
                ));
            }
        }
        out
    }

    /// The keys the footer shows.
    pub fn footer(&self) -> Vec<(String, String)> {
        if self.picking {
            return vec![
                ("r b m".into(), "auto classes".into()),
                ("e".into(), "expected failures".into()),
                ("enter".into(), "done".into()),
            ];
        }
        vec![
            ("R".into(), "run with this policy".into()),
            ("shift+R".into(), "from this task".into()),
            ("a".into(), "policy…".into()),
            ("enter".into(), "fold".into()),
            ("tab".into(), "panel".into()),
            ("esc".into(), "plans".into()),
        ]
    }

    /// Draws the plan view in `area`.
    pub fn draw(&self, buf: &mut Buffer, area: Rect, rel: &str, g: Glyphs) {
        let plan = self.plan();
        let nav = Rect::new(area.x, area.y, tree::WIDTH, area.height);
        let n = plan.tasks.len();
        let title = format!("{n} task{}", if n == 1 { "" } else { "s" });
        self.tree.draw(
            buf,
            nav,
            plan,
            &title,
            self.focus == Focus::Tree,
            &|node| self.mark(node),
            g,
        );
        let right = Rect::new(
            area.x + tree::WIDTH,
            area.y,
            area.width - tree::WIDTH,
            area.height,
        );
        let upper_h = right.height * 55 / 100;
        let upper = Rect::new(right.x, right.y, right.width, upper_h);
        let lower = Rect::new(
            right.x,
            right.y + upper_h,
            right.width,
            right.height - upper_h,
        );
        self.draw_action(buf, upper, rel, g);
        if self.picking {
            self.draw_picker(buf, lower, g);
        } else {
            self.draw_dry(buf, lower, g);
        }
    }

    fn draw_action(&self, buf: &mut Buffer, area: Rect, rel: &str, g: Glyphs) {
        let plan = self.plan();
        let Some(node) = self.tree.current(plan) else {
            band(buf, area, false, "ACTION", "", g);
            return;
        };
        let line = |span: planrun_model::Span| plan.source.line_col(span.start).line;
        let (title, detail, place) = match node {
            Node::Task(t) => (
                "TASK",
                plan.tasks[t].id.raw.to_string(),
                line(plan.tasks[t].span),
            ),
            Node::Step(t, s) => (
                "STEP",
                step_name(plan, t, s, g),
                line(plan.tasks[t].steps[s].span),
            ),
            Node::Action(t, s, a) => {
                let step = &plan.tasks[t].steps[s];
                let number = step
                    .number
                    .map_or_else(|| g.get(Glyph::Dot).to_owned(), |n| n.to_string());
                let detail = format!("{} of {} in step {number}", a + 1, step.actions.len());
                ("ACTION", detail, line(step.actions[a].span))
            }
        };
        band(buf, area, self.focus == Focus::Action, title, &detail, g);
        Pen::new(buf, area.x, area.y, area.right().saturating_sub(1), g)
            .right(&format!("{rel}:{place}"), on(Token::Fg3, Token::Bg1));
        let body = Rect::new(
            area.x + 2,
            area.y + 2,
            area.width.saturating_sub(3),
            area.height.saturating_sub(2),
        );
        match node {
            Node::Action(t, s, a) => self.draw_action_body(buf, body, t, s, a, g),
            Node::Step(t, s) => self.draw_step_body(buf, body, t, s, g),
            Node::Task(t) => self.draw_task_body(buf, body, t, g),
        }
    }

    fn draw_action_body(
        &self,
        buf: &mut Buffer,
        area: Rect,
        t: usize,
        s: usize,
        a: usize,
        g: Glyphs,
    ) {
        let plan = self.plan();
        let task = &plan.tasks[t];
        let action = &task.steps[s].actions[a];
        let right = area.right();
        let mut y = area.y;
        let mut pen = Pen::new(buf, area.x, y, right, g);
        pen.glyph(tree::kind_glyph(action), bold(tree::kind_color(action)))
            .gap(1);
        pen.text(kind_name(action), bold(Token::Fg));
        if let Some(path) = planrun_exec::handler::file_path(&action.kind) {
            pen.gap(2).text(&path.to_string(), bold(Token::Fg));
        }
        y += 2;
        let label = |buf: &mut Buffer, y: u16, name: &str| -> u16 {
            let mut pen = Pen::new(buf, area.x, y, right, g);
            pen.cell(name, 9, fg(Token::Fg3));
            pen.x
        };
        let decision = self.decision(t, s, a);
        let x = label(buf, y, "class");
        let (class_text, class_color) = class_line(plan, task, action);
        Pen::new(buf, x, y, right, g).text(&class_text, fg(class_color));
        y += 1;
        let x = label(buf, y, "policy");
        Pen::new(buf, x, y, right, g).text(&policy_line(action, decision), fg(Token::Fg2));
        y += 1;
        let x = label(buf, y, "dry run");
        let mut pen = Pen::new(buf, x, y, right, g);
        match self.report(t, s, a) {
            Some(report) => verdict(&mut pen, &report.outcome, g),
            None => {
                pen.glyph(Glyph::Background, fg(Token::Fg2))
                    .text(" running", fg(Token::Fg2));
            }
        }
        y += 2;
        let rest = Rect::new(
            area.x.saturating_sub(1),
            y,
            area.width + 1,
            area.bottom().saturating_sub(y),
        );
        if rest.height == 0 {
            return;
        }
        if let Some(diff) = diff::planned(plan, &action.kind) {
            self.draw_diff(buf, rest, &diff, g);
            return;
        }
        let body = match &action.kind {
            ActionKind::RunScript(script) | ActionKind::Commit(script) => plan.text(&script.body),
            ActionKind::Manual { text, .. } | ActionKind::Note { text } => plan.text(text),
            _ => "",
        };
        for (i, l) in body
            .lines()
            .skip(self.action_top)
            .take(usize::from(rest.height))
            .enumerate()
        {
            let y = rest.y + u16::try_from(i).unwrap_or(0);
            let mut pen = Pen::new(buf, rest.x + 1, y, rest.right(), g);
            pen.text(
                &crate::text::right(&(i + 1 + self.action_top).to_string(), 4),
                fg(Token::Fg3),
            );
            pen.gap(2).text(l, fg(Token::Fg));
        }
    }

    fn draw_diff(&self, buf: &mut Buffer, area: Rect, diff: &FileDiff, g: Glyphs) {
        diff::header(buf, area, diff, ("PLANNED", Token::Fg2), "", g);
        let rows = Rect::new(
            area.x,
            area.y + 1,
            area.width,
            area.height.saturating_sub(1),
        );
        diff::draw_rows(buf, rows, &diff.rows, self.action_top, g);
    }

    fn draw_step_body(&self, buf: &mut Buffer, area: Rect, t: usize, s: usize, g: Glyphs) {
        let plan = self.plan();
        let step = &plan.tasks[t].steps[s];
        let mut y = area.y;
        let title = step.title.as_ref().map_or("", |t| plan.text(t));
        Pen::new(buf, area.x, y, area.right(), g).text(title, bold(Token::Fg));
        y += 2;
        for (a, action) in step.actions.iter().enumerate() {
            if y >= area.bottom() {
                break;
            }
            let mark = self.mark(Node::Action(t, s, a));
            let mut pen = Pen::new(buf, area.x, y, area.right(), g);
            pen.glyph(mark.glyph, bold(mark.color)).gap(1);
            pen.glyph(tree::kind_glyph(action), bold(tree::kind_color(action)))
                .gap(1);
            pen.cell(&tree::label(plan, action), 48, fg(Token::Fg2))
                .gap(1);
            pen.text(&mark.tag, fg(mark.tag_color));
            y += 1;
        }
    }

    fn draw_task_body(&self, buf: &mut Buffer, area: Rect, t: usize, g: Glyphs) {
        let plan = self.plan();
        let task = &plan.tasks[t];
        let mut y = area.y;
        Pen::new(buf, area.x, y, area.right(), g).text(plan.text(&task.title), bold(Token::Fg));
        y += 2;
        Pen::new(buf, area.x, y, area.right(), g).text("FILES", bold(Token::Fg3));
        y += 1;
        if task.manifest.entries.is_empty() {
            Pen::new(buf, area.x, y, area.right(), g)
                .text("no manifest: every file action asks (D50)", fg(Token::Fg2));
            y += 1;
        }
        for entry in &task.manifest.entries {
            if y >= area.bottom() {
                return;
            }
            let mut pen = Pen::new(buf, area.x, y, area.right(), g);
            pen.cell(
                &format!("{:?}", entry.change).to_lowercase(),
                10,
                fg(Token::Fg3),
            );
            pen.text(&entry.path.to_string(), fg(Token::Fg));
            if entry.origin == PathOrigin::Bare {
                pen.gap(2).text("bare: never in scope", fg(Token::Warn));
            }
            y += 1;
        }
        let Some(report) = self.dry.value().and_then(|d| d.tasks.get(t)) else {
            return;
        };
        y += 1;
        for f in &report.manifest {
            if y >= area.bottom() {
                return;
            }
            let (glyph, color) = finding_mark(f);
            let mut pen = Pen::new(buf, area.x, y, area.right(), g);
            pen.glyph(glyph, bold(color))
                .gap(1)
                .text(f.code, fg(Token::Fg))
                .gap(2)
                .text(&f.message, fg(Token::Fg2));
            y += 1;
        }
    }

    fn draw_dry(&self, buf: &mut Buffer, area: Rect, g: Glyphs) {
        band(
            buf,
            area,
            self.focus == Focus::DryRun,
            "DRY RUN",
            &format!(
                "advisory {} preflight decides at run time",
                g.get(Glyph::Dot)
            ),
            g,
        );
        let Some(dry) = self.dry.value() else {
            let text = match &self.dry {
                Bg::Failed(e) | Bg::Unavailable(e) => e.clone(),
                _ => format!("{} running", g.get(Glyph::Background)),
            };
            Pen::new(buf, area.x + 2, area.y + 1, area.right(), g).text(&text, fg(Token::Fg2));
            return;
        };
        let (exit, color) = if dry.has_errors() {
            ("exit 1", Token::Err)
        } else {
            ("exit 0", Token::Ok)
        };
        Pen::new(buf, area.x, area.y, area.right().saturating_sub(1), g)
            .right(exit, on(color, Token::Bg1));
        let lines = self.dry_lines(dry, g);
        for (i, line) in lines
            .iter()
            .skip(self.dry_top)
            .take(usize::from(area.height.saturating_sub(1)))
            .enumerate()
        {
            let y = area.y + 1 + u16::try_from(i).unwrap_or(0);
            let mut pen = Pen::new(buf, area.x + 2, y, area.right(), g);
            pen.glyph(line.glyph, bold(line.color)).gap(1);
            pen.cell(&line.task, 7, fg(Token::Meta));
            pen.cell(&line.place, 11, fg(Token::Fg3));
            pen.text(&line.text, fg(line.text_color));
        }
    }

    fn dry_lines(&self, dry: &DryRun, g: Glyphs) -> Vec<DryLine> {
        let plan = self.plan();
        let mut lines = Vec::new();
        for f in &dry.start {
            let (glyph, color) = finding_mark(f);
            lines.push(DryLine {
                glyph,
                color,
                task: String::new(),
                place: f.code.to_owned(),
                text: f.message.clone(),
                text_color: Token::Fg2,
            });
        }
        if dry.start.is_empty() {
            lines.push(DryLine {
                glyph: Glyph::Done,
                color: Token::Ok,
                task: String::new(),
                place: "dirt".to_owned(),
                text: "none in the paths the plan touches".to_owned(),
                text_color: Token::Fg2,
            });
        }
        for report in &dry.tasks {
            let task = &plan.tasks[report.task];
            for f in &report.manifest {
                let (glyph, color) = finding_mark(f);
                lines.push(DryLine {
                    glyph,
                    color,
                    task: task.id.raw.to_string(),
                    place: "manifest".to_owned(),
                    text: f.message.clone(),
                    text_color: color,
                });
            }
            for r in &report.actions {
                if matches!(r.action.kind, ActionKind::Note { .. }) {
                    continue;
                }
                let step = &task.steps[r.step];
                let number = step
                    .number
                    .map_or_else(|| g.get(Glyph::Dot).to_owned(), |n| n.to_string());
                let a = step
                    .actions
                    .iter()
                    .position(|x| x.span == r.action.span)
                    .unwrap_or(0);
                let decision = self.decision(report.task, r.step, a);
                let (glyph, color) = if matches!(decision, Decision::Skip(_)) {
                    (Glyph::Skipped, Token::Fg3)
                } else {
                    self.status(report.task, r.step, a)
                };
                let glyph = if matches!(r.action.kind, ActionKind::Manual { .. }) {
                    Glyph::Manual
                } else {
                    glyph
                };
                let color = if glyph == Glyph::Manual {
                    Token::Warn
                } else {
                    color
                };
                let (what, text_color) = dry_text(&r.outcome, decision);
                lines.push(DryLine {
                    glyph,
                    color,
                    task: task.id.raw.to_string(),
                    place: format!("step {number}"),
                    text: format!(
                        "{} {}  {what}",
                        g.get(tree::kind_glyph(&r.action)),
                        crate::text::fit(&tree::label(plan, &r.action), 28, g.get(Glyph::Ellipsis))
                    ),
                    text_color,
                });
            }
        }
        lines
    }

    fn draw_picker(&self, buf: &mut Buffer, area: Rect, g: Glyphs) {
        band(
            buf,
            area,
            true,
            "POLICY",
            "assumed for the tags above, and for R",
            g,
        );
        let mut y = area.y + 2;
        let rows = [
            ("r", "read", self.assumed.auto.contains(&PolicyClass::Read)),
            (
                "b",
                "build",
                self.assumed.auto.contains(&PolicyClass::Build),
            ),
            (
                "m",
                "modify",
                self.assumed.auto.contains(&PolicyClass::Modify),
            ),
            ("e", "run expected failures", self.assumed.expected),
        ];
        for (key, name, on_) in rows {
            let mut pen = Pen::new(buf, area.x + 2, y, area.right(), g);
            pen.chip(key, "", Token::Bg0).gap(1);
            let (glyph, color) = if on_ {
                (Glyph::Done, Token::Act)
            } else {
                (Glyph::Pending, Token::Fg3)
            };
            pen.glyph(glyph, bold(color))
                .gap(1)
                .text(name, fg(Token::Fg));
            if key != "e" {
                pen.gap(2).text("runs without asking", fg(Token::Fg3));
            }
            y += 1;
        }
        y += 1;
        Pen::new(buf, area.x + 2, y, area.right(), g).text(
            "Destructive scripts always need a typed yes (D69).",
            fg(Token::Fg2),
        );
        Pen::new(buf, area.x + 2, y + 1, area.right(), g).text(
            &format!(
                "--auto {}",
                if self.assumed.auto.is_empty() {
                    "(none)".to_owned()
                } else {
                    self.assumed.auto_words()
                }
            ),
            fg(Token::Meta),
        );
    }
}

struct DryLine {
    glyph: Glyph,
    color: Token,
    task: String,
    place: String,
    text: String,
    text_color: Token,
}

fn counts(dry: &DryRun) -> (usize, usize, usize) {
    let mut ok = 0;
    for t in &dry.tasks {
        ok += t
            .actions
            .iter()
            .filter(|a| matches!(a.outcome, Outcome::Ok(_) | Outcome::WouldAsk(_)))
            .count();
    }
    let warn = dry
        .findings()
        .filter(|f| f.severity == Severity::Warning)
        .count();
    let err = dry.findings().filter(|f| f.is_error()).count();
    (ok, warn, err)
}

fn finding_mark(f: &Finding) -> (Glyph, Token) {
    if f.is_error() {
        (Glyph::Failed, Token::Err)
    } else {
        (Glyph::Warning, Token::Warn)
    }
}

/// The worst of the marks: failed, then a warning, then done, then all skipped, else not
/// simulated.
fn worst(all: &[(Glyph, Token)]) -> (Glyph, Token) {
    for glyph in [Glyph::Failed, Glyph::Warning, Glyph::Background] {
        if let Some(m) = all.iter().find(|m| m.0 == glyph) {
            return *m;
        }
    }
    if all.iter().any(|m| m.0 == Glyph::Done) {
        return (Glyph::Done, Token::Ok);
    }
    if !all.is_empty() && all.iter().all(|m| m.0 == Glyph::Skipped) {
        return (Glyph::Skipped, Token::Fg3);
    }
    (Glyph::Dot, Token::Fg3)
}

/// The tag of an action in Inspect: what the policy will do.
pub fn tag(action: &Action, decision: Decision) -> (String, Token) {
    if matches!(action.kind, ActionKind::Commit(_)) && decision != Decision::TypedYes {
        return ("gate".into(), Token::Fg2);
    }
    match decision {
        Decision::Auto => ("auto".into(), Token::Fg3),
        Decision::Ask(_) | Decision::AskUnscoped => ("ask".into(), Token::Fg2),
        Decision::TypedYes => ("type yes".into(), Token::Err),
        Decision::Wait => ("waits".into(), Token::Warn),
        Decision::Show => (String::new(), Token::Fg3),
        Decision::Skip(_) => ("skip".into(), Token::Fg3),
    }
}

fn kind_name(action: &Action) -> &'static str {
    match action.kind {
        ActionKind::WriteFile { .. } => "WRITE FILE",
        ActionKind::ApplyPatch { .. } => "APPLY PATCH",
        ActionKind::ReplaceExact { .. } => "REPLACE EXACT",
        ActionKind::Delete { .. } => "DELETE",
        ActionKind::Generated { .. } => "GENERATED",
        ActionKind::RunScript(_) => "RUN SCRIPT",
        ActionKind::Commit(_) => "COMMIT",
        ActionKind::Note { .. } => "NOTE",
        ActionKind::Manual { .. } => "MANUAL",
    }
}

fn class_line(plan: &Plan, task: &planrun_model::Task, action: &Action) -> (String, Token) {
    match &action.kind {
        ActionKind::RunScript(s) | ActionKind::Commit(s) => {
            let c = DefaultClassifier.explain(s.shell, plan.text(&s.body));
            let color = if c.class == PolicyClass::Destructive {
                Token::Err
            } else {
                Token::Fg2
            };
            let mut text = class_name(c.class).to_owned();
            if !c.reasons.is_empty() {
                text.push_str(": ");
                text.push_str(&c.reasons.join("; "));
            }
            (text, color)
        }
        ActionKind::Manual { reason, .. } => (format!("Manual: {reason:?}"), Token::Warn),
        ActionKind::Note { .. } => ("Note: shown, never run".into(), Token::Fg2),
        ActionKind::Generated { .. } => ("checked to exist before the commit".into(), Token::Fg2),
        _ => {
            let Some(path) = planrun_exec::handler::file_path(&action.kind) else {
                return (String::new(), Token::Fg2);
            };
            let entry = task
                .manifest
                .entries
                .iter()
                .find(|e| e.path.file() == Some(path));
            match entry {
                Some(e) if task.manifest.scopes(path) => (
                    format!(
                        "Modify · in task scope (manifest: {} {path})",
                        format!("{:?}", e.change).to_lowercase()
                    ),
                    Token::Fg2,
                ),
                _ => (
                    "Modify · outside the task's files (D50)".into(),
                    Token::Warn,
                ),
            }
        }
    }
}

fn policy_line(action: &Action, decision: Decision) -> String {
    let file = planrun_exec::handler::file_path(&action.kind).is_some()
        && !matches!(action.kind, ActionKind::Generated { .. });
    let what = match decision {
        Decision::Auto if file => {
            "runs without asking: a file action inside the task's scope".to_owned()
        }
        Decision::Auto => "runs without asking".to_owned(),
        Decision::Ask(c) => format!("asks first: {} is not in --auto", class_name(c)),
        Decision::AskUnscoped => "asks first: outside the task's files (D50)".to_owned(),
        Decision::TypedYes => "needs a typed yes: Destructive".to_owned(),
        Decision::Wait => "waits for you: planrun cannot do it".to_owned(),
        Decision::Show => "shown in place".to_owned(),
        Decision::Skip(_) => "skipped: an expected failure (a = run expected failures)".to_owned(),
    };
    if matches!(action.kind, ActionKind::Commit(_)) {
        return format!("behind the commit gate; {what}");
    }
    what
}

fn verdict(pen: &mut Pen<'_>, outcome: &Outcome, _g: Glyphs) {
    match outcome {
        Outcome::Ok(effect) => {
            pen.glyph(Glyph::Done, fg(Token::Ok)).gap(1).text(
                &format!("{effect} {} simulates cleanly", _g.get(Glyph::Dot)),
                fg(Token::Fg2),
            );
        }
        Outcome::WouldAsk(effect) => {
            pen.glyph(Glyph::Dot, fg(Token::Fg3)).gap(1).text(
                &format!("{effect} {} would ask (unscoped)", _g.get(Glyph::Dot)),
                fg(Token::Fg2),
            );
        }
        Outcome::Failed(f) => {
            pen.glyph(Glyph::Failed, fg(Token::Err))
                .gap(1)
                .text(&format!("{}: {}", f.code, f.message), fg(Token::Err));
        }
        Outcome::Depends(f) => {
            pen.glyph(Glyph::Warning, fg(Token::Warn))
                .gap(1)
                .text(&f.message, fg(Token::Warn));
        }
        Outcome::NotRun => {
            pen.glyph(Glyph::Dot, fg(Token::Fg3))
                .gap(1)
                .text("not run in a dry run", fg(Token::Fg2));
        }
        Outcome::NotChecked => {
            pen.glyph(Glyph::Dot, fg(Token::Fg3))
                .gap(1)
                .text("not checked: its script did not run", fg(Token::Fg2));
        }
        Outcome::Waits => {
            pen.glyph(Glyph::Dot, fg(Token::Fg3))
                .gap(1)
                .text("waits for you", fg(Token::Fg2));
        }
        Outcome::Shown => {
            pen.glyph(Glyph::Dot, fg(Token::Fg3))
                .gap(1)
                .text("shown", fg(Token::Fg2));
        }
    }
}

fn dry_text(outcome: &Outcome, decision: Decision) -> (String, Token) {
    if let Decision::Skip(_) = decision {
        return ("skipped: expected failure".into(), Token::Fg2);
    }
    match outcome {
        Outcome::Ok(e) => (format!("{e}, simulates"), Token::Fg2),
        Outcome::WouldAsk(e) => (format!("{e}, would ask"), Token::Fg2),
        Outcome::Failed(f) => (format!("{}: {}", f.code, f.message), Token::Err),
        Outcome::Depends(f) => (f.message.clone(), Token::Warn),
        Outcome::NotRun => match decision {
            Decision::TypedYes => ("typed yes (Destructive)".into(), Token::Fg2),
            Decision::Ask(c) => (format!("would ask ({})", class_name(c)), Token::Fg2),
            _ => ("not run in a dry run".into(), Token::Fg2),
        },
        Outcome::NotChecked => ("not checked".into(), Token::Fg2),
        Outcome::Waits => ("manual: waits for you".into(), Token::Fg2),
        Outcome::Shown => ("shown".into(), Token::Fg2),
    }
}

fn step_name(plan: &Plan, t: usize, s: usize, g: Glyphs) -> String {
    let step = &plan.tasks[t].steps[s];
    match step.number {
        Some(n) => format!("step {n}"),
        None => format!("step {}", g.get(Glyph::Dot)),
    }
}

fn crumb_label(plan: &Plan, action: &Action) -> String {
    let label = tree::label(plan, action);
    match &action.kind {
        ActionKind::RunScript(_) => label.split_whitespace().next().unwrap_or("").to_owned(),
        _ => crate::text::fit(&label, 24, "…"),
    }
}
