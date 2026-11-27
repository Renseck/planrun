//! The navigator (docs/tui-design.md §6.B/C, decision D87): Task → Step → Action, 36 columns. A
//! row is a status glyph, a kind glyph, a label and a tag; tasks fold. The mode decides each
//! row's status and tag: Inspect from the dry run and the assumed policy, Run from the journal.

use std::collections::BTreeSet;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use planrun_model::{Action, ActionKind, Plan};

use crate::draw::{Pen, band, select_row};
use crate::glyph::{Glyph, Glyphs};
use crate::keys::Key;
use crate::text;
use crate::theme::{Token, bold, fg};

/// The navigator's width, its hairline included.
pub const WIDTH: u16 = 37;

/// Where a row points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Node {
    /// A task, by position.
    Task(usize),
    /// A step of a task.
    Step(usize, usize),
    /// An action of a step.
    Action(usize, usize, usize),
}

impl Node {
    /// The task's position.
    pub fn task(self) -> usize {
        match self {
            Node::Task(t) | Node::Step(t, _) | Node::Action(t, _, _) => t,
        }
    }
}

/// What a mode says about a row: its status glyph and colour, and its tag and colour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mark {
    /// The status glyph.
    pub glyph: Glyph,
    /// Its colour.
    pub color: Token,
    /// The tag on the right (`auto`, `0.1s`, a hash).
    pub tag: String,
    /// Its colour.
    pub tag_color: Token,
}

impl Mark {
    /// A mark without a tag.
    pub fn plain(glyph: Glyph, color: Token) -> Mark {
        Mark {
            glyph,
            color,
            tag: String::new(),
            tag_color: Token::Fg3,
        }
    }
}

/// The navigator's state: the folded tasks and the selected row.
#[derive(Debug, Clone, Default)]
pub struct Tree {
    /// Folded tasks, by position.
    pub folded: BTreeSet<usize>,
    /// The selected node.
    pub selected: Option<Node>,
}

impl Tree {
    /// The visible nodes of `plan`, in order. Notes are shown in their step's panel, not as
    /// rows, and a step of notes only has no row.
    pub fn rows(&self, plan: &Plan) -> Vec<Node> {
        let mut rows = Vec::new();
        for (t, task) in plan.tasks.iter().enumerate() {
            rows.push(Node::Task(t));
            if self.folded.contains(&t) {
                continue;
            }
            for (s, step) in task.steps.iter().enumerate() {
                let shown: Vec<usize> = (0..step.actions.len())
                    .filter(|&a| !matches!(step.actions[a].kind, ActionKind::Note { .. }))
                    .collect();
                if shown.is_empty() {
                    continue;
                }
                rows.push(Node::Step(t, s));
                rows.extend(shown.into_iter().map(|a| Node::Action(t, s, a)));
            }
        }
        rows
    }

    /// The selected node, or the first row.
    pub fn current(&self, plan: &Plan) -> Option<Node> {
        let rows = self.rows(plan);
        match self.selected {
            Some(n) if rows.contains(&n) => Some(n),
            Some(n) => rows
                .iter()
                .rev()
                .find(|r| **r <= n)
                .copied()
                .or(rows.first().copied()),
            None => rows.first().copied(),
        }
    }

    /// Selects `node`, unfolding its task.
    pub fn select(&mut self, node: Node) {
        self.folded.remove(&node.task());
        self.selected = Some(node);
    }

    /// Moves, folds and unfolds; false when the key is not the navigator's.
    pub fn key(&mut self, key: Key, plan: &Plan) -> bool {
        let rows = self.rows(plan);
        let Some(current) = self.current(plan) else {
            return false;
        };
        let at = rows.iter().position(|r| *r == current).unwrap_or(0);
        let last = rows.len().saturating_sub(1);
        let next = match key {
            Key::Up | Key::Char('k') => rows[at.saturating_sub(1)],
            Key::Down | Key::Char('j') => rows[(at + 1).min(last)],
            Key::PageUp => rows[at.saturating_sub(10)],
            Key::PageDown => rows[(at + 10).min(last)],
            Key::Home => rows[0],
            Key::End => rows[last],
            Key::Left | Key::Char('h') => {
                let t = current.task();
                if current == Node::Task(t) {
                    self.folded.insert(t);
                }
                Node::Task(t)
            }
            Key::Right | Key::Char('l') => {
                self.folded.remove(&current.task());
                current
            }
            Key::Enter => {
                if let Node::Task(t) = current
                    && !self.folded.remove(&t)
                {
                    self.folded.insert(t);
                }
                current
            }
            _ => return false,
        };
        self.selected = Some(next);
        true
    }

    /// Draws the navigator in `area`; `mark` gives each row's status and tag.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        buf: &mut Buffer,
        area: Rect,
        plan: &Plan,
        title: &str,
        focused: bool,
        mark: &dyn Fn(Node) -> Mark,
        g: Glyphs,
    ) {
        let inner = Rect::new(area.x, area.y, area.width.saturating_sub(1), area.height);
        crate::draw::vrule(
            buf,
            area.right().saturating_sub(1),
            area.y,
            area.bottom(),
            g,
        );
        band(buf, inner, focused, "PLAN", title, g);
        let rows = self.rows(plan);
        let current = self.current(plan);
        let height = usize::from(area.height.saturating_sub(1));
        let at = current
            .and_then(|c| rows.iter().position(|r| *r == c))
            .unwrap_or(0);
        let top = (at + 1).saturating_sub(height);
        for (i, node) in rows.iter().enumerate().skip(top).take(height) {
            let y = area.y + 1 + u16::try_from(i - top).unwrap_or(0);
            let selected = Some(*node) == current;
            if selected {
                select_row(buf, inner, y, g);
            }
            draw_row(
                buf,
                inner,
                y,
                plan,
                *node,
                &mark(*node),
                self.folded.contains(&node.task()),
                selected,
                g,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_row(
    buf: &mut Buffer,
    area: Rect,
    y: u16,
    plan: &Plan,
    node: Node,
    mark: &Mark,
    folded: bool,
    selected: bool,
    g: Glyphs,
) {
    let bg = if selected { Token::Bg2 } else { Token::Bg0 };
    let on = |t: Token| -> Style { fg(t).bg(bg.color()) };
    let tag_w = u16::try_from(text::width(&mark.tag)).unwrap_or(0);
    let label_end = area.right().saturating_sub(tag_w + 2);
    let mut pen = Pen::new(buf, area.x + 1, y, label_end, g);
    pen.glyph(mark.glyph, bold(mark.color).bg(bg.color()))
        .gap(1);
    match node {
        Node::Task(t) => {
            let task = &plan.tasks[t];
            let fold = if folded { Glyph::Closed } else { Glyph::Open };
            pen.glyph(fold, bold(Token::Fg3).bg(bg.color())).gap(1);
            let style = bold(Token::Fg).bg(bg.color());
            pen.text(
                &format!("{} {}", task.id.raw, plan.text(&task.title)),
                style,
            );
        }
        Node::Step(t, s) => {
            let step = &plan.tasks[t].steps[s];
            pen.gap(2);
            let number = step
                .number
                .map_or_else(|| g.get(Glyph::Dot).to_owned(), |n| n.to_string());
            let title = step.title.as_ref().map_or("", |t| plan.text(t));
            pen.text(&number, bold(Token::Fg3).bg(bg.color())).gap(1);
            pen.text(title, on(Token::Fg2));
        }
        Node::Action(t, s, a) => {
            let action = &plan.tasks[t].steps[s].actions[a];
            pen.gap(4);
            pen.glyph(kind_glyph(action), bold(kind_color(action)).bg(bg.color()))
                .gap(1);
            pen.text(&label(plan, action), on(Token::Fg2));
        }
    }
    let mut pen = Pen::new(buf, label_end, y, area.right().saturating_sub(1), g);
    pen.right(&mark.tag, on(mark.tag_color));
}

/// The kind glyph of an action.
pub fn kind_glyph(action: &Action) -> Glyph {
    match &action.kind {
        ActionKind::WriteFile { .. } => Glyph::Write,
        ActionKind::ApplyPatch { .. } => Glyph::Patch,
        ActionKind::ReplaceExact { .. } => Glyph::Replace,
        ActionKind::Delete { .. } => Glyph::Delete,
        ActionKind::Generated { .. } => Glyph::Generated,
        ActionKind::RunScript(_) => Glyph::Script,
        ActionKind::Commit(_) => Glyph::Commit,
        ActionKind::Note { .. } => Glyph::Note,
        ActionKind::Manual { .. } => Glyph::Manual,
    }
}

/// The kind glyph's colour: fg2, amber for a Manual.
pub fn kind_color(action: &Action) -> Token {
    match action.kind {
        ActionKind::Manual { .. } => Token::Warn,
        _ => Token::Fg2,
    }
}

/// An action's label: its path, a script's first line, `commit`, a Manual's or Note's first
/// words.
pub fn label(plan: &Plan, action: &Action) -> String {
    match &action.kind {
        ActionKind::WriteFile { path, .. }
        | ActionKind::ApplyPatch { path, .. }
        | ActionKind::ReplaceExact { path, .. }
        | ActionKind::Delete { path, .. }
        | ActionKind::Generated { path } => path.to_string(),
        ActionKind::RunScript(s) => first_line(plan.text(&s.body)),
        ActionKind::Commit(_) => "commit".to_owned(),
        ActionKind::Note { text } | ActionKind::Manual { text, .. } => first_line(plan.text(text)),
    }
}

/// The first non-blank line of `s`, trimmed.
pub fn first_line(s: &str) -> String {
    s.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .to_owned()
}
