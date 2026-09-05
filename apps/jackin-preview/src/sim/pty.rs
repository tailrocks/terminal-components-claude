//! Deterministic Capsule-daemon simulation.
//!
//! This is a model of tabs, pane geometry, transcripts, and agent input.  It
//! does not open a process, socket, or terminal.  Geometry delegates its
//! arithmetic to the public `junie_tui::SplitModel`; all transcript data is
//! owned here so tests can advance virtual time without depending on a PTY.

use std::collections::VecDeque;

use junie_tui::{FgStep, Position, Rect, Role, SplitAxis, SplitModel, width};

use crate::domain::account::AccountId;
use crate::domain::agent::Agent;
use crate::domain::instance::{AgentState, DaemonSnapshot};

/// Stable identifier for one simulated pane.
pub type PaneId = u64;

/// Maximum retained transcript lines.
pub const SCROLLBACK: usize = 2_000;

/// Minimum pane width used by the split model.
pub const MIN_PANE_COLS: u16 = 20;

/// Minimum pane height used by the split model.
pub const MIN_PANE_ROWS: u16 = 4;

/// Maximum automatic label length accepted by a caller.
pub const MAX_LABEL: usize = 16;

/// Direction of a two-pane split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDir {
    /// First pane on the left.
    Horizontal,
    /// First pane on top.
    Vertical,
}

impl SplitDir {
    const fn axis(self) -> SplitAxis {
        match self {
            Self::Horizontal => SplitAxis::Horizontal,
            Self::Vertical => SplitAxis::Vertical,
        }
    }
}

/// Which side of a split is maximized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Maximized {
    /// Neither side.
    None,
    /// The first side fills the container.
    First,
    /// The second side fills the container.
    Second,
}

/// A two-pane split ratio and its minima.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    /// Percent of usable space assigned to the first pane.
    pub percent: u16,
    /// Minimum first-pane length.
    pub min_first: u16,
    /// Minimum second-pane length.
    pub min_second: u16,
    /// Maximization state.
    pub maximized: Maximized,
}

impl Split {
    /// Construct a split with a first-pane percentage.
    pub const fn new(percent: u16, min_first: u16, min_second: u16) -> Self {
        Self {
            percent,
            min_first,
            min_second,
            maximized: Maximized::None,
        }
    }

    fn model(self, dir: SplitDir) -> SplitModel {
        let percent = self.percent.clamp(5, 95) as u8;
        let mut model = SplitModel::new(dir.axis(), percent, self.min_first, self.min_second);
        match self.maximized {
            Maximized::None => {}
            Maximized::First => model.toggle_max(junie_tui::Maximized::First),
            Maximized::Second => model.toggle_max(junie_tui::Maximized::Second),
        }
        model
    }

    /// Toggle a pane's maximized state.
    pub fn toggle_max(&mut self, which: Maximized) {
        self.maximized = if self.maximized == which {
            Maximized::None
        } else {
            which
        };
    }

    /// Grow the first pane by percentage points.
    pub fn grow(&mut self, delta: i16) {
        let next = i32::from(self.percent).saturating_add(i32::from(delta));
        self.percent = next.clamp(5, 95) as u16;
    }

    /// Lay out two panes with `gap` cells between them.
    pub fn layout(&self, dir: SplitDir, area: Rect, gap: u16) -> (Rect, Rect) {
        self.model(dir).layout(area, gap)
    }

    /// Return the seam between two panes.
    pub fn handle(&self, dir: SplitDir, area: Rect, gap: u16) -> Rect {
        self.model(dir).handle(area, gap)
    }

    /// Drag the seam under a position. Returns whether the ratio changed.
    pub fn drag_to(&mut self, dir: SplitDir, area: Rect, gap: u16, pos: Position) -> bool {
        let mut model = self.model(dir);
        let changed = model.drag_to(area, gap, pos);
        if changed {
            self.percent = u16::from(model.percent);
        }
        changed
    }

    /// Resize the first pane by whole cells.
    pub fn nudge(&mut self, dir: SplitDir, area: Rect, gap: u16, delta: i16) {
        let mut model = self.model(dir);
        model.nudge(area, gap, delta);
        self.percent = u16::from(model.percent);
    }

    /// Vertical layout convenience method.
    pub fn vertical(&self, area: Rect, gap: u16) -> (Rect, Rect) {
        self.layout(SplitDir::Vertical, area, gap)
    }

    /// Horizontal layout convenience method.
    pub fn horizontal(&self, area: Rect, gap: u16) -> (Rect, Rect) {
        self.layout(SplitDir::Horizontal, area, gap)
    }
}

/// A pane tree.  The path in a [`Seam`] uses `0` for first and `1` for second.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneNode {
    /// One leaf pane.
    Leaf(PaneId),
    /// A recursively split pair.
    Split {
        /// Split direction.
        dir: SplitDir,
        /// Ratio and minima.
        split: Split,
        /// First child.
        first: Box<PaneNode>,
        /// Second child.
        second: Box<PaneNode>,
    },
}

impl PaneNode {
    /// Return leaves in visual order.
    pub fn leaves(&self) -> Vec<PaneId> {
        match self {
            Self::Leaf(id) => vec![*id],
            Self::Split { first, second, .. } => {
                let mut leaves = first.leaves();
                leaves.extend(second.leaves());
                leaves
            }
        }
    }

    /// Populate leaf geometry and split seams for a container.
    pub fn layout(
        &self,
        area: Rect,
        out: &mut Vec<(PaneId, Rect)>,
        seams: &mut Vec<Seam>,
        path: &mut Vec<u8>,
    ) {
        match self {
            Self::Leaf(id) => out.push((*id, area)),
            Self::Split {
                dir,
                split,
                first,
                second,
            } => {
                let (first_area, second_area) = split.layout(*dir, area, 1);
                seams.push(Seam {
                    path: path.clone(),
                    dir: *dir,
                    container: area,
                    handle: split.handle(*dir, area, 1),
                });
                path.push(0);
                first.layout(first_area, out, seams, path);
                path.pop();
                path.push(1);
                second.layout(second_area, out, seams, path);
                path.pop();
            }
        }
    }

    /// Find a node by a binary child path.
    pub fn node_at_mut(&mut self, path: &[u8]) -> Option<&mut PaneNode> {
        let mut current = self;
        for child in path {
            match current {
                Self::Split { first, second, .. } => {
                    current = match child {
                        0 => first,
                        1 => second,
                        _ => return None,
                    };
                }
                Self::Leaf(_) => return None,
            }
        }
        Some(current)
    }

    /// Replace a leaf with a split containing the old and new panes.
    pub fn split_leaf(
        &mut self,
        target: PaneId,
        new: PaneId,
        dir: SplitDir,
        new_first: bool,
    ) -> bool {
        match self {
            Self::Leaf(id) if *id == target => {
                let old = Self::Leaf(*id);
                let fresh = Self::Leaf(new);
                let (first, second) = if new_first {
                    (fresh, old)
                } else {
                    (old, fresh)
                };
                *self = Self::Split {
                    dir,
                    split: Split::new(50, 1, 1),
                    first: Box::new(first),
                    second: Box::new(second),
                };
                true
            }
            Self::Leaf(_) => false,
            Self::Split { first, second, .. } => {
                first.split_leaf(target, new, dir, new_first)
                    || second.split_leaf(target, new, dir, new_first)
            }
        }
    }

    /// Remove a leaf and collapse its parent into the sibling.
    pub fn remove_leaf(&mut self, target: PaneId) -> bool {
        let Self::Split { first, second, .. } = self else {
            return false;
        };
        if matches!(first.as_ref(), Self::Leaf(id) if *id == target) {
            *self = second.as_ref().clone();
            return true;
        }
        if matches!(second.as_ref(), Self::Leaf(id) if *id == target) {
            *self = first.as_ref().clone();
            return true;
        }
        first.remove_leaf(target) || second.remove_leaf(target)
    }

    /// Append the binary path to a target leaf. Returns whether it was found.
    pub fn path_to(&self, target: PaneId, path: &mut Vec<u8>) -> bool {
        match self {
            Self::Leaf(id) => *id == target,
            Self::Split { first, second, .. } => {
                path.push(0);
                if first.path_to(target, path) {
                    return true;
                }
                path.pop();
                path.push(1);
                if second.path_to(target, path) {
                    return true;
                }
                path.pop();
                false
            }
        }
    }
}

/// A rendered seam and its tree path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seam {
    /// Binary path to the split.
    pub path: Vec<u8>,
    /// Split direction.
    pub dir: SplitDir,
    /// Container rectangle.
    pub container: Rect,
    /// Interactive seam rectangle.
    pub handle: Rect,
}

/// Direction used when moving keyboard focus among panes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Move left.
    Left,
    /// Move right.
    Right,
    /// Move up.
    Up,
    /// Move down.
    Down,
}

/// Find the nearest leaf whose rectangle lies in `direction` from `from`.
pub fn nearest(leaves: &[(PaneId, Rect)], from: PaneId, direction: Direction) -> Option<PaneId> {
    let (_, from_rect) = leaves.iter().find(|(id, _)| *id == from)?;
    let from_center = (
        i32::from(from_rect.x) + i32::from(from_rect.width) / 2,
        i32::from(from_rect.y) + i32::from(from_rect.height) / 2,
    );
    leaves
        .iter()
        .filter(|(id, rect)| {
            *id != from
                && match direction {
                    Direction::Left => rect.right() <= from_rect.x,
                    Direction::Right => rect.x >= from_rect.right(),
                    Direction::Up => rect.bottom() <= from_rect.y,
                    Direction::Down => rect.y >= from_rect.bottom(),
                }
        })
        .min_by_key(|(_, rect)| {
            let center = (
                i32::from(rect.x) + i32::from(rect.width) / 2,
                i32::from(rect.y) + i32::from(rect.height) / 2,
            );
            (center.0 - from_center.0).abs() + (center.1 - from_center.1).abs()
        })
        .map(|(id, _)| *id)
}

/// One tab in the simulated daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    /// Optional operator-provided label.
    pub custom_label: Option<String>,
    /// Root pane tree.
    pub root: PaneNode,
    /// Focused pane.
    pub focused: PaneId,
    /// Zoomed pane, if any.
    pub zoomed: Option<PaneId>,
}

impl Tab {
    /// Return leaf panes in visual order.
    pub fn leaves(&self) -> Vec<PaneId> {
        self.root.leaves()
    }
}

/// A semantic transcript tone.  It maps to a public `junie_tui` role when a
/// future Capsule screen projects owned lines into the facade viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Primary text.
    Normal,
    /// Muted text.
    Muted,
    /// Secondary text.
    Secondary,
    /// Success text.
    Success,
    /// Error text.
    Error,
    /// Warning text.
    Warning,
}

impl Tone {
    /// Convert the simulation tone to a public `junie_tui` role.
    pub const fn role(self) -> Role {
        match self {
            Self::Normal => Role::Fg(FgStep::Primary),
            Self::Muted => Role::Fg(FgStep::Muted),
            Self::Secondary => Role::Fg(FgStep::Secondary),
            Self::Success => Role::Success,
            Self::Error => Role::Danger,
            Self::Warning => Role::Warning,
        }
    }
}

/// One owned styled transcript span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Span text.
    pub text: String,
    /// Semantic tone.
    pub tone: Tone,
    /// Whether the span is bold.
    pub bold: bool,
}

impl Span {
    /// Construct a span.
    pub fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
            bold: false,
        }
    }

    /// Make a span bold.
    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// The public facade role for this span.
    pub const fn role(&self) -> Role {
        self.tone.role()
    }
}

/// One owned transcript line.
pub type Line = Vec<Span>;

/// Bounded owned transcript state used by a simulated pane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextViewport {
    /// Retained transcript lines.
    pub lines: Vec<Line>,
    /// Whether new output follows the tail.
    pub follow: bool,
    /// Optional cursor position.
    pub caret: Option<junie_tui::CellPos>,
    /// Whether the cursor is visible.
    pub caret_visible: bool,
    max_lines: usize,
}

impl TextViewport {
    /// Construct an empty viewport with the default scrollback cap.
    pub const fn new() -> Self {
        Self {
            lines: Vec::new(),
            follow: true,
            caret: None,
            caret_visible: true,
            max_lines: SCROLLBACK,
        }
    }

    /// Set the maximum number of retained lines.
    #[must_use]
    pub const fn max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = max_lines;
        self
    }

    /// Toggle tail following.
    pub const fn set_follow(&mut self, follow: bool) {
        self.follow = follow;
    }

    /// Append one line and enforce bounded retention.
    pub fn push(&mut self, line: Line) {
        self.lines.push(line);
        if self.max_lines > 0 && self.lines.len() > self.max_lines {
            let drop = self.lines.len().saturating_sub(self.max_lines);
            self.lines.drain(..drop);
        }
    }

    /// Replace the last line, or append when empty.
    pub fn replace_last(&mut self, line: Line) {
        if let Some(last) = self.lines.last_mut() {
            *last = line;
        } else {
            self.push(line);
        }
    }

    /// Clear all transcript lines and cursor state.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.caret = None;
    }
}

impl Default for TextViewport {
    fn default() -> Self {
        Self::new()
    }
}

/// One deterministic process step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Emit a line after a virtual delay.
    Emit(i64, Line),
    /// Change the public agent state.
    State(AgentState),
    /// Mark a repository-relative path touched.
    Touch(&'static str),
    /// Wait for operator input.
    Await,
}

/// Deterministic agent or shell process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProcess {
    /// Agent identity, or `None` for a shell.
    pub agent: Option<Agent>,
    /// Account identity only; credential material never enters this model.
    pub account: Option<AccountId>,
    /// Current public state.
    pub state: AgentState,
    /// Boot script.
    pub script: Vec<Step>,
    /// Next script step.
    pub pc: usize,
    /// Absolute due time for the current emit step.
    pub next_at_ms: i64,
    scheduled: bool,
    /// Delayed reply lines.
    pub reply_queue: VecDeque<(i64, Line)>,
    /// Reply rotation index.
    pub reply_cursor: usize,
    /// Input prompt.
    pub prompt: String,
    /// Whether the process hides its cursor.
    pub cursor_hidden: bool,
    /// Whether input is answering a permission prompt.
    pub awaiting_permission: bool,
    /// Files touched by this process.
    pub touched: Vec<String>,
}

fn span(text: &str, tone: Tone) -> Span {
    Span::new(text, tone)
}

fn line(text: &str, tone: Tone) -> Line {
    vec![span(text, tone)]
}

fn mixed(parts: &[(&str, Tone)]) -> Line {
    parts.iter().map(|(text, tone)| span(text, *tone)).collect()
}

fn emit(delay: i64, value: Line) -> Step {
    Step::Emit(delay, value)
}

fn done_line() -> Line {
    mixed(&[("○ ", Tone::Secondary), ("Done", Tone::Muted)])
}

/// Build a deterministic, secret-free transcript for one agent.
pub fn script(agent: Option<Agent>, workspace: &str) -> Vec<Step> {
    let path = format!("~/{workspace}");
    match agent {
        Some(Agent::ClaudeCode) => vec![
            Step::State(AgentState::Working),
            emit(
                0,
                mixed(&[("▐ ", Tone::Success), ("Claude Code v2.1.14", Tone::Normal)]),
            ),
            emit(
                180,
                mixed(&[("› ", Tone::Muted), ("inspect retry policy", Tone::Normal)]),
            ),
            emit(
                360,
                mixed(&[
                    ("● ", Tone::Secondary),
                    ("Read ", Tone::Normal),
                    (&path, Tone::Secondary),
                ]),
            ),
            emit(
                220,
                line("  fixed backoff needs a bounded cap.", Tone::Normal),
            ),
            emit(
                260,
                mixed(&[
                    ("▶ ", Tone::Warning),
                    (
                        "Allow edit to src/settlement/config.rs? (y/n)",
                        Tone::Normal,
                    ),
                ]),
            ),
            Step::State(AgentState::Blocked),
            Step::Await,
        ],
        Some(Agent::Codex) => vec![
            Step::State(AgentState::Working),
            emit(
                0,
                mixed(&[
                    ("OpenAI Codex v0.48", Tone::Normal),
                    (" · ", Tone::Muted),
                    (&path, Tone::Secondary),
                ]),
            ),
            emit(
                220,
                mixed(&[
                    ("• exec  ", Tone::Secondary),
                    ("cargo test -p ledger integration", Tone::Normal),
                ]),
            ),
            emit(300, line("  running 24 tests", Tone::Muted)),
            emit(
                220,
                line("  test reconcile::daily_close ........... ok", Tone::Muted),
            ),
            emit(
                220,
                mixed(&[
                    ("  test reconcile::multi_currency ........ ", Tone::Muted),
                    ("FAILED", Tone::Error),
                ]),
            ),
            emit(
                320,
                mixed(&[("  23 passed · ", Tone::Muted), ("1 failed", Tone::Error)]),
            ),
            emit(220, done_line()),
            Step::State(AgentState::Done),
            Step::Await,
        ],
        Some(Agent::Amp) => vec![
            Step::State(AgentState::Working),
            emit(
                0,
                mixed(&[
                    ("Amp 1.9.3", Tone::Normal),
                    (" · ", Tone::Muted),
                    (&path, Tone::Secondary),
                ]),
            ),
            emit(260, line("Searching kube/controllers/**/*.go", Tone::Muted)),
            emit(
                340,
                line(
                    "The reconcile loop re-queues every object on resync.",
                    Tone::Normal,
                ),
            ),
            emit(
                280,
                mixed(&[
                    ("Apply this edit? ", Tone::Normal),
                    ("[Y/n]", Tone::Warning),
                ]),
            ),
            Step::State(AgentState::Blocked),
            Step::Await,
        ],
        Some(Agent::KimiCode) => vec![
            Step::State(AgentState::Working),
            emit(
                0,
                mixed(&[
                    ("Kimi Code 0.7.2", Tone::Normal),
                    (" · ", Tone::Muted),
                    (&path, Tone::Secondary),
                ]),
            ),
            emit(
                300,
                line(
                    "Plan: add loading skeletons to the invoices table.",
                    Tone::Normal,
                ),
            ),
            emit(
                420,
                line("Wrote components/table/SkeletonRow.tsx", Tone::Secondary),
            ),
            Step::Touch("components/table/SkeletonRow.tsx"),
            emit(300, done_line()),
            Step::State(AgentState::Done),
            Step::Await,
        ],
        Some(Agent::OpenCode) => vec![
            Step::State(AgentState::Working),
            emit(
                0,
                mixed(&[
                    ("opencode 0.5.11", Tone::Normal),
                    (" · ", Tone::Muted),
                    (&path, Tone::Secondary),
                ]),
            ),
            emit(
                300,
                line("Editing .github/workflows/release.yml", Tone::Secondary),
            ),
            Step::Touch(".github/workflows/release.yml"),
            emit(
                260,
                line("Updated action pins and node version.", Tone::Normal),
            ),
            emit(220, done_line()),
            Step::State(AgentState::Done),
            Step::Await,
        ],
        Some(Agent::GrokBuild) => vec![
            Step::State(AgentState::Working),
            emit(
                0,
                mixed(&[
                    ("Grok Build 0.3", Tone::Normal),
                    (" · ", Tone::Muted),
                    (&path, Tone::Secondary),
                ]),
            ),
            emit(
                300,
                line("Writing modules/gke/node_pool.tf", Tone::Secondary),
            ),
            Step::Touch("modules/gke/node_pool.tf"),
            emit(320, line("terraform validate … ok", Tone::Muted)),
            emit(260, done_line()),
            Step::State(AgentState::Done),
            Step::Await,
        ],
        None => vec![
            emit(
                0,
                mixed(&[
                    ("payments-platform ❯ ", Tone::Secondary),
                    ("git status -sb", Tone::Normal),
                ]),
            ),
            emit(
                260,
                line("## feature/settlement-backoff [ahead 2]", Tone::Muted),
            ),
            emit(
                100,
                mixed(&[
                    (" M ", Tone::Warning),
                    ("src/settlement/retry.rs", Tone::Normal),
                ]),
            ),
            Step::Await,
        ],
    }
}

fn replies(agent: Option<Agent>) -> Vec<Vec<Line>> {
    match agent {
        Some(Agent::ClaudeCode) => vec![vec![
            line("Read src/settlement/config.rs", Tone::Normal),
            line("The policy is wired; I will add a cap test.", Tone::Normal),
            done_line(),
        ]],
        Some(Agent::Codex) => vec![vec![
            line(
                "cargo test -p ledger reconcile::multi_currency",
                Tone::Secondary,
            ),
            line("running 1 test … ok", Tone::Muted),
            done_line(),
        ]],
        Some(Agent::Amp | Agent::KimiCode | Agent::OpenCode | Agent::GrokBuild) => vec![vec![
            line("Understood. Working on it.", Tone::Normal),
            done_line(),
        ]],
        None => vec![],
    }
}

impl AgentProcess {
    /// Construct a process at virtual time `start_ms`.
    pub fn new(
        agent: Option<Agent>,
        account: Option<AccountId>,
        workspace: &str,
        start_ms: i64,
    ) -> Self {
        let prompt = match agent {
            Some(Agent::ClaudeCode | Agent::Codex) => "❯ ".to_owned(),
            Some(Agent::OpenCode | Agent::GrokBuild) => "> ".to_owned(),
            Some(_) => "› ".to_owned(),
            None => format!("{workspace} ❯ "),
        };
        Self {
            agent,
            account,
            state: if agent.is_some() {
                AgentState::Working
            } else {
                AgentState::Unknown
            },
            script: script(agent, workspace),
            pc: 0,
            next_at_ms: start_ms,
            scheduled: false,
            reply_queue: VecDeque::new(),
            reply_cursor: 0,
            prompt,
            cursor_hidden: false,
            awaiting_permission: false,
            touched: Vec::new(),
        }
    }

    /// Whether the process is waiting for operator input after its boot script.
    pub fn awaiting(&self) -> bool {
        self.pc > 0
            && matches!(
                self.script.get(self.pc.saturating_sub(1)),
                Some(Step::Await)
            )
            && self.reply_queue.is_empty()
    }

    /// Fast-forward the boot script and return its transcript.
    pub fn boot_all(&mut self, out: &mut Vec<Line>) {
        while self.pc < self.script.len() {
            let Some(step) = self.script.get(self.pc).cloned() else {
                break;
            };
            match step {
                Step::Emit(_, value) => out.push(value),
                Step::State(state) => self.state = state,
                Step::Touch(path) => self.touched.push(path.to_owned()),
                Step::Await => {
                    self.pc = self.pc.saturating_add(1);
                    self.awaiting_permission = self.state == AgentState::Blocked;
                    return;
                }
            }
            self.pc = self.pc.saturating_add(1);
        }
    }

    /// Advance to `now_ms` and return newly available transcript lines.
    pub fn tick(&mut self, now_ms: i64) -> Vec<Line> {
        let mut out = Vec::new();
        loop {
            let ready = self
                .reply_queue
                .front()
                .is_some_and(|(due, _)| *due <= now_ms);
            if !ready {
                break;
            }
            if let Some((_, value)) = self.reply_queue.pop_front() {
                out.push(value);
            }
        }
        if self.reply_queue.is_empty()
            && self.state == AgentState::Working
            && self.awaiting()
            && !self.awaiting_permission
        {
            self.state = AgentState::Done;
        }
        while self.pc < self.script.len() {
            let Some(step) = self.script.get(self.pc).cloned() else {
                break;
            };
            match step {
                Step::Emit(delay, value) => {
                    if !self.scheduled {
                        self.next_at_ms = self.next_at_ms.saturating_add(delay.max(0));
                        self.scheduled = true;
                    }
                    if now_ms < self.next_at_ms {
                        break;
                    }
                    self.scheduled = false;
                    out.push(value);
                    self.pc = self.pc.saturating_add(1);
                }
                Step::State(state) => {
                    self.state = state;
                    self.pc = self.pc.saturating_add(1);
                }
                Step::Touch(path) => {
                    self.touched.push(path.to_owned());
                    self.pc = self.pc.saturating_add(1);
                }
                Step::Await => {
                    self.awaiting_permission = self.state == AgentState::Blocked;
                    self.pc = self.pc.saturating_add(1);
                    break;
                }
            }
        }
        out
    }

    /// Handle one committed operator input line.
    pub fn on_input(&mut self, input: &str, now_ms: i64, workspace: &str) -> Vec<Line> {
        let mut out = vec![mixed(&[
            (&self.prompt, Tone::Secondary),
            (input, Tone::Normal),
        ])];
        if self.agent.is_some() {
            if self.awaiting_permission {
                self.awaiting_permission = false;
                let yes = matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes");
                if yes {
                    self.state = AgentState::Working;
                    self.touched.push("src/settlement/config.rs".to_owned());
                    self.reply_queue.push_back((
                        now_ms.saturating_add(350),
                        line("Edit src/settlement/config.rs", Tone::Normal),
                    ));
                    self.reply_queue.push_back((
                        now_ms.saturating_add(750),
                        line("pub retry: RetryPolicy,", Tone::Secondary),
                    ));
                    self.reply_queue
                        .push_back((now_ms.saturating_add(1_100), done_line()));
                } else {
                    self.state = AgentState::Idle;
                    self.reply_queue
                        .push_back((now_ms.saturating_add(250), line("Skipped.", Tone::Muted)));
                }
                return out;
            }
            self.state = AgentState::Working;
            out.push(mixed(&[("⠋ ", Tone::Success), ("Thinking…", Tone::Muted)]));
            let choices = replies(self.agent);
            if let Some(choice) = choices.get(self.reply_cursor % choices.len().max(1)) {
                self.reply_cursor = self.reply_cursor.saturating_add(1);
                let mut due = now_ms.saturating_add(500);
                out.push(line(&format!("↳ \"{}\"", input.trim()), Tone::Muted));
                for value in choice {
                    due = due.saturating_add(300);
                    self.reply_queue.push_back((due, value.clone()));
                }
            }
            return out;
        }

        let prompt = self.prompt.clone();
        let command = input.trim();
        match command.split_whitespace().next() {
            Some("ls") => out.push(line(
                "Cargo.toml   crates/   docs/   scripts/   README.md",
                Tone::Normal,
            )),
            Some("pwd") => out.push(line(&format!("/workspace/{workspace}"), Tone::Normal)),
            Some("git") => match command.split_whitespace().nth(1) {
                Some("status") => {
                    out.push(line(
                        "## feature/settlement-backoff…origin/feature/settlement-backoff [ahead 2]",
                        Tone::Muted,
                    ));
                    for path in &self.touched {
                        out.push(mixed(&[(" M ", Tone::Warning), (path, Tone::Normal)]));
                    }
                }
                Some("push") => out.push(line(
                    "To github.com:chainargos/payments-platform.git",
                    Tone::Muted,
                )),
                Some("log") => out.push(line(
                    "3f8a1d0 settlement: exponential backoff (#482)",
                    Tone::Normal,
                )),
                _ => out.push(line("usage: git <command> [<args>]", Tone::Muted)),
            },
            Some("cargo") => out.push(line(
                "    Finished checking settlement without errors",
                Tone::Muted,
            )),
            Some("clear") => out.push(line("\u{0}clear", Tone::Normal)),
            Some(other) => out.push(line(
                &format!("zsh: command not found: {other}"),
                Tone::Error,
            )),
            None => {}
        }
        let _ = prompt;
        out
    }
}

/// One simulated Capsule pane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pane {
    /// Pane identity.
    pub id: PaneId,
    /// Process driving the pane.
    pub proc: AgentProcess,
    /// Owned transcript viewport.
    pub term: TextViewport,
    /// Input typed ahead of commit.
    pub input: String,
    /// Whether output has arrived since boot.
    pub received_output: bool,
}

impl Pane {
    /// Construct a pane.
    pub fn new(
        id: PaneId,
        agent: Option<Agent>,
        account: Option<AccountId>,
        workspace: &str,
        start_ms: i64,
    ) -> Self {
        Self {
            id,
            proc: AgentProcess::new(agent, account, workspace, start_ms),
            term: TextViewport::new().max_lines(SCROLLBACK),
            input: String::new(),
            received_output: false,
        }
    }

    /// Display label for the pane process.
    pub fn label(&self) -> String {
        self.proc
            .agent
            .map_or_else(|| "Shell".to_owned(), |agent| agent.label().to_owned())
    }

    fn prompt_line(&self) -> Line {
        vec![
            span(&self.proc.prompt, Tone::Secondary),
            span(&self.input, Tone::Normal),
        ]
    }

    fn refresh_prompt(&mut self) {
        let waiting = self.proc.awaiting() || !self.input.is_empty();
        if waiting {
            let last_is_prompt = self
                .term
                .lines
                .last()
                .is_some_and(|value| value.first().is_some_and(|s| s.text == self.proc.prompt));
            let prompt = self.prompt_line();
            if last_is_prompt {
                self.term.replace_last(prompt);
            } else {
                self.term.push(prompt);
            }
            let line_number = self.term.lines.len().saturating_sub(1);
            let col = usize::from(width(&self.proc.prompt))
                .saturating_add(usize::from(width(&self.input)));
            self.term.caret = Some(junie_tui::CellPos::new(line_number, col));
        } else {
            self.term.caret = None;
        }
        self.term.caret_visible = !self.proc.cursor_hidden;
    }

    fn push_lines(&mut self, lines: Vec<Line>) {
        if lines.is_empty() {
            return;
        }
        let pending_prompt = self
            .term
            .lines
            .last()
            .is_some_and(|value| value.first().is_some_and(|s| s.text == self.proc.prompt));
        if pending_prompt {
            self.term.lines.pop();
        }
        for value in lines {
            let first = value.first().map(|s| s.text.as_str());
            if first == Some("\u{0}clear") {
                self.term.clear();
                continue;
            }
            if first == Some("⠋ ") {
                self.term.push(value);
                continue;
            }
            let spinner = self
                .term
                .lines
                .last()
                .is_some_and(|line| line.first().is_some_and(|s| s.text == "⠋ "));
            if spinner {
                self.term.lines.pop();
            }
            self.term.push(value);
            self.received_output = true;
        }
        self.refresh_prompt();
    }

    /// Fast-forward boot output into the pane.
    pub fn boot_all(&mut self) {
        let mut output = Vec::new();
        self.proc.boot_all(&mut output);
        self.push_lines(output);
        self.refresh_prompt();
    }

    /// Advance the pane to virtual time and return whether it changed.
    pub fn tick(&mut self, now_ms: i64) -> bool {
        let output = self.proc.tick(now_ms);
        let changed = !output.is_empty();
        self.push_lines(output);
        self.refresh_prompt();
        changed
    }

    /// Type one character without committing it.
    pub fn type_char(&mut self, character: char, _now_ms: i64, _workspace: &str) {
        self.term.set_follow(true);
        self.input.push(character);
        self.refresh_prompt();
    }

    /// Delete the last typed character.
    pub fn backspace(&mut self) {
        self.term.set_follow(true);
        self.input.pop();
        self.refresh_prompt();
    }

    /// Commit the current input line.
    pub fn commit(&mut self, now_ms: i64, workspace: &str) {
        self.term.set_follow(true);
        let input = std::mem::take(&mut self.input);
        let pending_prompt = self
            .term
            .lines
            .last()
            .is_some_and(|value| value.first().is_some_and(|s| s.text == self.proc.prompt));
        if pending_prompt {
            self.term.lines.pop();
        }
        let output = self.proc.on_input(&input, now_ms, workspace);
        self.push_lines(output);
        self.refresh_prompt();
    }

    /// Clear the transcript.
    pub fn clear(&mut self) {
        self.term.clear();
        self.term.set_follow(true);
        self.refresh_prompt();
    }

    /// Current public process state.
    pub const fn state(&self) -> AgentState {
        self.proc.state
    }
}

/// Deterministic in-memory Capsule daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Daemon {
    /// Workspace display name.
    pub workspace: String,
    /// Open tabs.
    pub tabs: Vec<Tab>,
    /// Active tab index.
    pub active: usize,
    /// All panes, including panes nested in inactive tabs.
    pub panes: Vec<Pane>,
    /// Next pane identity.
    pub next_id: PaneId,
    /// Client holding the single attach.
    pub attached_by: Option<String>,
}

impl Daemon {
    /// Construct an empty daemon.
    pub fn new(workspace: &str) -> Self {
        Self {
            workspace: workspace.to_owned(),
            tabs: Vec::new(),
            active: 0,
            panes: Vec::new(),
            next_id: 1,
            attached_by: None,
        }
    }

    /// Build a live daemon model from a persisted public snapshot.
    ///
    /// Snapshot metadata is intentionally copied only into semantic pane and
    /// tab state; no terminal backend or process handle crosses the boundary.
    pub fn from_snapshot(snapshot: &DaemonSnapshot, workspace: &str, now_ms: i64) -> Self {
        let mut daemon = Self::new(workspace);
        let DaemonSnapshot::Tabs(tabs) = snapshot else {
            return daemon;
        };
        for tab in tabs {
            let first = tab.panes.first();
            let pane = daemon.new_pane(first.and_then(|pane| pane.agent), None, now_ms, false);
            daemon.tabs.push(Tab {
                custom_label: Some(tab.label.clone()),
                root: PaneNode::Leaf(pane),
                focused: pane,
                zoomed: None,
            });
            for pane_snapshot in tab.panes.iter().skip(1) {
                let extra = daemon.new_pane(pane_snapshot.agent, None, now_ms, false);
                if let Some(active) = daemon.tabs.last_mut() {
                    active.root = PaneNode::Split {
                        dir: SplitDir::Horizontal,
                        split: Split::new(50, MIN_PANE_COLS, MIN_PANE_COLS),
                        first: Box::new(active.root.clone()),
                        second: Box::new(PaneNode::Leaf(extra)),
                    };
                }
            }
        }
        daemon.active = tabs
            .iter()
            .position(|tab| tab.active)
            .unwrap_or(0)
            .min(daemon.tabs.len().saturating_sub(1));
        daemon
    }

    /// Find a pane.
    pub fn pane(&self, id: PaneId) -> Option<&Pane> {
        self.panes.iter().find(|pane| pane.id == id)
    }

    /// Find a mutable pane.
    pub fn pane_mut(&mut self, id: PaneId) -> Option<&mut Pane> {
        self.panes.iter_mut().find(|pane| pane.id == id)
    }

    /// Active tab.
    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }

    /// Mutable active tab.
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active)
    }

    /// Focused pane in the active tab.
    pub fn focused_pane(&self) -> Option<PaneId> {
        self.active_tab().map(|tab| tab.focused)
    }

    fn alloc(&mut self) -> PaneId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    /// Add a pane to the daemon.
    pub fn new_pane(
        &mut self,
        agent: Option<Agent>,
        account: Option<AccountId>,
        now_ms: i64,
        boot: bool,
    ) -> PaneId {
        let id = self.alloc();
        let mut pane = Pane::new(id, agent, account, &self.workspace, now_ms);
        if boot {
            pane.boot_all();
        }
        self.panes.push(pane);
        id
    }

    /// Add a new tab containing one pane and activate it.
    pub fn new_tab(
        &mut self,
        agent: Option<Agent>,
        account: Option<AccountId>,
        now_ms: i64,
        boot: bool,
    ) -> usize {
        let pane = self.new_pane(agent, account, now_ms, boot);
        self.tabs.push(Tab {
            custom_label: None,
            root: PaneNode::Leaf(pane),
            focused: pane,
            zoomed: None,
        });
        self.active = self.tabs.len().saturating_sub(1);
        self.active
    }

    /// Split the active tab's focused pane.
    pub fn split(
        &mut self,
        dir: SplitDir,
        new_first: bool,
        agent: Option<Agent>,
        account: Option<AccountId>,
        now_ms: i64,
        boot: bool,
    ) -> Option<PaneId> {
        let target = self.focused_pane()?;
        let pane = self.new_pane(agent, account, now_ms, boot);
        let tab = self.active_tab_mut()?;
        if tab.root.split_leaf(target, pane, dir, new_first) {
            tab.focused = pane;
            tab.zoomed = None;
            Some(pane)
        } else {
            self.panes.retain(|value| value.id != pane);
            None
        }
    }

    /// Close a pane, returning whether the containing tab was also closed.
    pub fn close_pane(&mut self, id: PaneId) -> bool {
        let Some(tab_index) = self.tabs.iter().position(|tab| tab.leaves().contains(&id)) else {
            return false;
        };
        self.panes.retain(|pane| pane.id != id);
        let Some(tab) = self.tabs.get_mut(tab_index) else {
            return false;
        };
        if tab.leaves().len() <= 1 {
            self.tabs.remove(tab_index);
            if self.active >= self.tabs.len() {
                self.active = self.tabs.len().saturating_sub(1);
            }
            return true;
        }
        tab.root.remove_leaf(id);
        if tab.focused == id
            && let Some(next) = tab.leaves().first().copied()
        {
            tab.focused = next;
        }
        if tab.zoomed == Some(id) {
            tab.zoomed = None;
        }
        false
    }

    /// Close a tab by index.
    pub fn close_tab(&mut self, index: usize) {
        let Some(tab) = self.tabs.get(index) else {
            return;
        };
        let leaves = tab.leaves();
        self.panes.retain(|pane| !leaves.contains(&pane.id));
        self.tabs.remove(index);
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len().saturating_sub(1);
        }
    }

    /// Automatic label for a tab.
    pub fn tab_label(&self, tab: &Tab, account_suffix: &dyn Fn(&Pane) -> Option<String>) -> String {
        if let Some(label) = &tab.custom_label {
            return label.clone();
        }
        let leaves = tab.leaves();
        let mut agents = Vec::new();
        let mut has_shell = false;
        for id in &leaves {
            if let Some(pane) = self.pane(*id) {
                match pane.proc.agent {
                    Some(agent) => {
                        let mut label = agent.label().to_owned();
                        if let Some(suffix) = account_suffix(pane) {
                            label.push_str(" (");
                            label.push_str(&suffix);
                            label.push(')');
                        }
                        if !agents.contains(&label) {
                            agents.push(label);
                        }
                    }
                    None => has_shell = true,
                }
            }
        }
        let base = match (agents.len(), has_shell) {
            (0, _) => "Shell".to_owned(),
            (1, false) => agents
                .first()
                .cloned()
                .unwrap_or_else(|| "Agents".to_owned()),
            (_, false) => "Agents".to_owned(),
            (_, true) => "Mix".to_owned(),
        };
        if leaves.len() > 1 {
            format!("{base} ({})", leaves.len())
        } else {
            base
        }
    }

    /// Highest attention state among panes in a tab.
    pub fn tab_state(&self, tab: &Tab) -> AgentState {
        tab.leaves()
            .iter()
            .filter_map(|id| self.pane(*id).map(Pane::state))
            .max_by_key(|state| state.rank())
            .unwrap_or(AgentState::Unknown)
    }

    /// Advance every process to virtual time.
    pub fn tick(&mut self, now_ms: i64) -> bool {
        let mut changed = false;
        for pane in &mut self.panes {
            changed |= pane.tick(now_ms);
        }
        changed
    }

    /// Unique repository paths touched by agents.
    pub fn touched_files(&self) -> Vec<String> {
        let mut paths: Vec<String> = self
            .panes
            .iter()
            .flat_map(|pane| pane.proc.touched.iter().cloned())
            .collect();
        paths.sort();
        paths.dedup();
        paths
    }

    /// Convert the daemon to the public instance snapshot model.
    pub fn snapshot(&self) -> DaemonSnapshot {
        use crate::domain::instance::{PaneSnapshot, TabSnapshot};

        if self.tabs.is_empty() {
            return DaemonSnapshot::NoTabs;
        }
        DaemonSnapshot::Tabs(
            self.tabs
                .iter()
                .enumerate()
                .map(|(index, tab)| TabSnapshot {
                    label: self.tab_label(tab, &|_| None),
                    active: index == self.active,
                    panes: tab
                        .leaves()
                        .iter()
                        .filter_map(|id| self.pane(*id))
                        .map(|pane| PaneSnapshot {
                            label: pane.label(),
                            agent: pane.proc.agent,
                            state: pane.state(),
                            focused: tab.focused == pane.id,
                        })
                        .collect(),
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_close_and_nearest_are_deterministic() {
        let mut daemon = Daemon::new("payments-platform");
        daemon.new_tab(Some(Agent::ClaudeCode), None, 0, false);
        let right = daemon
            .split(SplitDir::Horizontal, false, None, None, 0, false)
            .expect("active pane exists");
        let below = daemon
            .split(
                SplitDir::Vertical,
                false,
                Some(Agent::Codex),
                None,
                0,
                false,
            )
            .expect("active pane exists");
        let tab = daemon.active_tab().expect("tab exists");
        assert_eq!(tab.leaves().len(), 3);
        assert_eq!(tab.focused, below);
        let mut leaves = Vec::new();
        let mut seams = Vec::new();
        tab.root.layout(
            Rect::new(0, 0, 120, 40),
            &mut leaves,
            &mut seams,
            &mut Vec::new(),
        );
        assert_eq!(seams.len(), 2);
        assert_eq!(nearest(&leaves, below, Direction::Up), Some(right));
        assert_eq!(nearest(&leaves, below, Direction::Left), Some(1));
        assert_eq!(nearest(&leaves, 1, Direction::Left), None);
        assert!(!daemon.close_pane(below));
        assert_eq!(daemon.active_tab().expect("tab exists").leaves().len(), 2);
        assert_eq!(
            daemon.tab_label(daemon.active_tab().expect("tab exists"), &|_| None),
            "Mix (2)"
        );
        assert!(!daemon.close_pane(right));
        assert!(daemon.close_pane(1));
        assert!(daemon.tabs.is_empty());
    }

    #[test]
    fn process_boot_and_input_never_depend_on_wall_clock() {
        let mut pane = Pane::new(1, Some(Agent::Codex), None, "payments-platform", 0);
        let mut now = 0;
        while pane.proc.pc < pane.proc.script.len() && now < 60_000 {
            now += 100;
            pane.tick(now);
        }
        assert_eq!(pane.state(), AgentState::Done);
        assert!(
            pane.term
                .lines
                .iter()
                .any(|value| value.iter().any(|span| span.text.contains("FAILED")))
        );
        for character in "hi".chars() {
            pane.type_char(character, now, "payments-platform");
        }
        pane.commit(now, "payments-platform");
        assert_eq!(pane.state(), AgentState::Working);
        for _ in 0..40 {
            now += 100;
            pane.tick(now);
        }
        assert_eq!(pane.state(), AgentState::Done);
        assert!(
            pane.term
                .lines
                .iter()
                .any(|value| value.iter().any(|span| span.text.contains("\"hi\"")))
        );

        let mut shell = Pane::new(2, None, None, "payments-platform", 0);
        shell.boot_all();
        assert!(shell.term.caret.is_some());
        for character in "git push".chars() {
            shell.type_char(character, 0, "payments-platform");
        }
        shell.commit(0, "payments-platform");
        assert!(
            shell
                .term
                .lines
                .iter()
                .any(|value| value.iter().any(|span| span.text.contains("github.com")))
        );
    }
}
