use std::collections::VecDeque;

use termrock::{FgStep, Position, Rect, Role, SplitAxis, SplitModel, width};

use jackin_preview_domain::account::AccountId;
use jackin_preview_domain::agent::Agent;
use jackin_preview_domain::instance::{AgentState, DaemonSnapshot};

pub type PaneId = u64;

pub const SCROLLBACK: usize = 2_000;

pub const MIN_PANE_COLS: u16 = 20;

pub const MIN_PANE_ROWS: u16 = 4;

pub const MAX_LABEL: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDir {
    Horizontal,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Maximized {
    None,
    First,
    Second,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    pub percent: u16,
    pub min_first: u16,
    pub min_second: u16,
    pub maximized: Maximized,
}

impl Split {
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
            Maximized::First => model.toggle_max(termrock::Maximized::First),
            Maximized::Second => model.toggle_max(termrock::Maximized::Second),
        }
        model
    }

    pub fn toggle_max(&mut self, which: Maximized) {
        self.maximized = if self.maximized == which {
            Maximized::None
        } else {
            which
        };
    }

    pub fn grow(&mut self, delta: i16) {
        let next = i32::from(self.percent).saturating_add(i32::from(delta));
        self.percent = next.clamp(5, 95) as u16;
    }

    pub fn layout(&self, dir: SplitDir, area: Rect, gap: u16) -> (Rect, Rect) {
        self.model(dir).layout(area, gap)
    }

    pub fn handle(&self, dir: SplitDir, area: Rect, gap: u16) -> Rect {
        self.model(dir).handle(area, gap)
    }

    pub fn drag_to(&mut self, dir: SplitDir, area: Rect, gap: u16, pos: Position) -> bool {
        let mut model = self.model(dir);
        let changed = model.drag_to(area, gap, pos);
        if changed {
            self.percent = u16::from(model.percent);
        }
        changed
    }

    pub fn nudge(&mut self, dir: SplitDir, area: Rect, gap: u16, delta: i16) {
        let mut model = self.model(dir);
        model.nudge(area, gap, delta);
        self.percent = u16::from(model.percent);
    }

    pub fn vertical(&self, area: Rect, gap: u16) -> (Rect, Rect) {
        self.layout(SplitDir::Vertical, area, gap)
    }

    pub fn horizontal(&self, area: Rect, gap: u16) -> (Rect, Rect) {
        self.layout(SplitDir::Horizontal, area, gap)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneNode {
    Leaf(PaneId),
    Split {
        dir: SplitDir,
        split: Split,
        first: Box<PaneNode>,
        second: Box<PaneNode>,
    },
}

impl PaneNode {
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

    pub fn leaf_count(&self) -> usize {
        match self {
            Self::Leaf(_) => 1,
            Self::Split { first, second, .. } => {
                first.leaf_count().saturating_add(second.leaf_count())
            }
        }
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seam {
    pub path: Vec<u8>,
    pub dir: SplitDir,
    pub container: Rect,
    pub handle: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    pub custom_label: Option<String>,
    pub root: PaneNode,
    pub focused: PaneId,
    pub zoomed: Option<PaneId>,
}

impl Tab {
    pub fn leaves(&self) -> Vec<PaneId> {
        self.root.leaves()
    }

    pub fn leaf_count(&self) -> usize {
        self.root.leaf_count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Normal,
    Muted,
    Secondary,
    Success,
    Error,
    Warning,
}

impl Tone {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub tone: Tone,
    pub bold: bool,
}

impl Span {
    pub fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
            bold: false,
        }
    }

    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub const fn role(&self) -> Role {
        self.tone.role()
    }
}

pub type Line = Vec<Span>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextViewport {
    pub lines: Vec<Line>,
    pub follow: bool,
    pub caret: Option<termrock::CellPos>,
    pub caret_visible: bool,
    max_lines: usize,
    revision: u64,
    retained: usize,
}

impl TextViewport {
    pub const fn new() -> Self {
        Self {
            lines: Vec::new(),
            follow: true,
            caret: None,
            caret_visible: true,
            max_lines: SCROLLBACK,
            revision: 0,
            retained: 0,
        }
    }

    #[must_use]
    pub const fn max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = max_lines;
        self
    }

    pub const fn set_follow(&mut self, follow: bool) {
        self.follow = follow;
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub const fn retained(&self) -> usize {
        self.retained
    }

    pub fn push(&mut self, line: Line) {
        self.lines.push(line);
        if self.max_lines > 0 && self.lines.len() > self.max_lines {
            let drop = self.lines.len().saturating_sub(self.max_lines);
            self.lines.drain(..drop);
            self.retained = self.retained.saturating_add(drop);
        }
        self.revision = self.revision.saturating_add(1);
    }

    pub fn replace_last(&mut self, line: Line) {
        if let Some(last) = self.lines.last_mut() {
            *last = line;
            self.revision = self.revision.saturating_add(1);
        } else {
            self.push(line);
        }
    }

    fn remove_last(&mut self) -> Option<Line> {
        let removed = self.lines.pop();
        if removed.is_some() {
            self.revision = self.revision.saturating_add(1);
        }
        removed
    }

    pub fn clear(&mut self) {
        self.retained = self.retained.saturating_add(self.lines.len());
        self.lines.clear();
        self.caret = None;
        self.revision = self.revision.saturating_add(1);
    }
}

impl Default for TextViewport {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Emit(i64, Line),
    State(AgentState),
    Touch(&'static str),
    Await,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProcess {
    pub agent: Option<Agent>,
    pub account: Option<AccountId>,
    pub state: AgentState,
    pub script: Vec<Step>,
    pub pc: usize,
    pub next_at_ms: i64,
    scheduled: bool,
    pub reply_queue: VecDeque<(i64, Line)>,
    pub reply_cursor: usize,
    pub prompt: String,
    pub cursor_hidden: bool,
    pub awaiting_permission: bool,
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

fn e(delay: i64, value: Line) -> Step {
    emit(delay, value)
}

fn bold(text: &str) -> Line {
    vec![span(text, Tone::Normal).bold()]
}

fn muted(text: &str) -> Line {
    line(text, Tone::Muted)
}

fn done_line() -> Line {
    mixed(&[("○ ", Tone::Secondary), ("Done", Tone::Muted)])
}

pub fn script(agent: Option<Agent>, workspace: &str) -> Vec<Step> {
    let ws = format!("~/{workspace}");
    match agent {
        Some(Agent::ClaudeCode) => vec![
            Step::State(AgentState::Working),
            e(
                0,
                mixed(&[
                    ("▐ ", Tone::Success),
                    ("Claude Code v2.1.14", Tone::Normal),
                    (" · Opus 4.5 · ", Tone::Muted),
                    (&ws, Tone::Secondary),
                ]),
            ),
            e(200, vec![]),
            e(
                300,
                mixed(&[
                    ("› ", Tone::Muted),
                    (
                        "Refactor the settlement retry loop so failed batches",
                        Tone::Normal,
                    ),
                ]),
            ),
            e(
                60,
                line(
                    "  back off exponentially and cap at 5 attempts.",
                    Tone::Normal,
                ),
            ),
            e(400, vec![]),
            e(
                500,
                mixed(&[
                    ("● ", Tone::Secondary),
                    ("I'll read the retry loop first.", Tone::Normal),
                ]),
            ),
            e(500, vec![]),
            e(
                600,
                mixed(&[
                    ("● ", Tone::Secondary),
                    ("Read ", Tone::Normal),
                    ("src/settlement/retry.rs", Tone::Secondary),
                    (" (142 lines)", Tone::Muted),
                ]),
            ),
            e(
                400,
                mixed(&[
                    ("● ", Tone::Secondary),
                    ("Read ", Tone::Normal),
                    ("src/settlement/mod.rs", Tone::Secondary),
                    (" (88 lines)", Tone::Muted),
                ]),
            ),
            e(700, vec![]),
            e(
                200,
                mixed(&[
                    ("● ", Tone::Secondary),
                    (
                        "The loop retries with a fixed 3 attempts. I'll add",
                        Tone::Normal,
                    ),
                ]),
            ),
            e(
                60,
                line(
                    "  exponential backoff with jitter and cap it at 5.",
                    Tone::Normal,
                ),
            ),
            e(800, vec![]),
            e(
                200,
                mixed(&[
                    ("● ", Tone::Secondary),
                    ("Edit ", Tone::Normal),
                    ("src/settlement/retry.rs", Tone::Secondary),
                ]),
            ),
            Step::Touch("src/settlement/retry.rs"),
            e(
                200,
                mixed(&[
                    ("  +  ", Tone::Success),
                    ("const MAX_ATTEMPTS: u32 = 5;", Tone::Secondary),
                ]),
            ),
            e(
                120,
                mixed(&[
                    ("  +  ", Tone::Success),
                    (
                        "let delay = BASE * 2u32.pow(attempt) + jitter();",
                        Tone::Secondary,
                    ),
                ]),
            ),
            e(
                120,
                mixed(&[
                    ("  -  ", Tone::Error),
                    ("for attempt in 0..3 {", Tone::Muted),
                ]),
            ),
            e(
                120,
                mixed(&[
                    ("  +  ", Tone::Success),
                    ("for attempt in 0..MAX_ATTEMPTS {", Tone::Secondary),
                ]),
            ),
            e(900, vec![]),
            e(
                200,
                mixed(&[
                    ("● ", Tone::Secondary),
                    ("Bash ", Tone::Normal),
                    ("cargo test -p settlement retry", Tone::Secondary),
                ]),
            ),
            e(1400, muted("  running 6 tests … 6 passed (1.42 s)")),
            e(600, vec![]),
            e(
                200,
                mixed(&[
                    ("● ", Tone::Secondary),
                    ("Edit ", Tone::Normal),
                    ("src/settlement/mod.rs", Tone::Secondary),
                ]),
            ),
            Step::Touch("src/settlement/mod.rs"),
            e(
                200,
                mixed(&[
                    ("  +  ", Tone::Success),
                    (
                        "pub use retry::{MAX_ATTEMPTS, RetryPolicy};",
                        Tone::Secondary,
                    ),
                ]),
            ),
            e(800, vec![]),
            e(
                200,
                mixed(&[
                    ("● ", Tone::Secondary),
                    (
                        "Retries now back off 250 ms → 4 s, capped at 5 tries.",
                        Tone::Normal,
                    ),
                ]),
            ),
            e(
                80,
                line(
                    "  One more edit: expose the policy in settlement config.",
                    Tone::Normal,
                ),
            ),
            e(600, vec![]),
            e(
                300,
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
            e(
                0,
                mixed(&[
                    ("OpenAI Codex v0.48", Tone::Normal),
                    (" · gpt-5.5-codex · ", Tone::Muted),
                    (&ws, Tone::Secondary),
                ]),
            ),
            e(200, vec![]),
            e(
                300,
                mixed(&[
                    ("› ", Tone::Muted),
                    (
                        "run the ledger integration tests and summarise failures",
                        Tone::Normal,
                    ),
                ]),
            ),
            e(500, vec![]),
            e(
                300,
                mixed(&[
                    ("• exec  ", Tone::Secondary),
                    ("cargo test -p ledger --test integration", Tone::Normal),
                ]),
            ),
            e(700, muted("  running 24 tests")),
            e(500, muted("  test reconcile::daily_close ........... ok")),
            e(
                400,
                mixed(&[
                    ("  test reconcile::multi_currency ........ ", Tone::Muted),
                    ("FAILED", Tone::Error),
                ]),
            ),
            e(400, muted("  test settle::partial_refund ........... ok")),
            e(
                900,
                mixed(&[
                    ("  22 passed · ", Tone::Muted),
                    ("1 failed", Tone::Error),
                    (" · 1 ignored (6.8 s)", Tone::Muted),
                ]),
            ),
            e(600, vec![]),
            e(
                300,
                mixed(&[
                    ("• ", Tone::Secondary),
                    ("1 failure: reconcile::multi_currency", Tone::Normal),
                ]),
            ),
            e(
                200,
                line("  expected 1,204.50 EUR, got 1,204.49 EUR", Tone::Normal),
            ),
            e(
                200,
                mixed(&[
                    ("  rounding precedes FX conversion in ", Tone::Normal),
                    ("ledger/fx.rs:71", Tone::Secondary),
                ]),
            ),
            e(700, vec![]),
            e(
                200,
                mixed(&[
                    ("○ ", Tone::Secondary),
                    ("Done · 38 s · 12.4k tokens", Tone::Muted),
                ]),
            ),
            Step::State(AgentState::Done),
            e(300, vec![]),
            Step::Await,
        ],
        Some(Agent::Amp) => vec![
            Step::State(AgentState::Working),
            e(
                0,
                mixed(&[
                    ("Amp 1.9.3", Tone::Normal),
                    (" · ", Tone::Muted),
                    (&ws, Tone::Secondary),
                ]),
            ),
            e(200, vec![]),
            e(
                300,
                mixed(&[
                    ("› ", Tone::Muted),
                    ("Why is the controller reconcile loop hot?", Tone::Normal),
                ]),
            ),
            e(500, vec![]),
            e(
                300,
                mixed(&[
                    ("⠿ ", Tone::Success),
                    ("Searching kube/controllers/**/*.go", Tone::Muted),
                ]),
            ),
            e(
                600,
                mixed(&[
                    ("⠿ ", Tone::Success),
                    ("Read kube/controllers/node_pool.go:118-164", Tone::Muted),
                ]),
            ),
            e(800, vec![]),
            e(
                200,
                line(
                    "The loop re-queues every object on every informer resync",
                    Tone::Normal,
                ),
            ),
            e(
                80,
                line(
                    "(resyncPeriod = 30s) instead of only on spec changes.",
                    Tone::Normal,
                ),
            ),
            e(600, vec![]),
            e(200, line("Suggested change:", Tone::Normal)),
            e(
                150,
                mixed(&[
                    ("  - ", Tone::Error),
                    (
                        ".WithEventFilter(predicate.ResourceVersionChangedPredicate{})",
                        Tone::Muted,
                    ),
                ]),
            ),
            e(
                150,
                mixed(&[
                    ("  + ", Tone::Success),
                    (
                        ".WithEventFilter(predicate.GenerationChangedPredicate{})",
                        Tone::Secondary,
                    ),
                ]),
            ),
            e(600, vec![]),
            e(
                200,
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
            e(
                0,
                mixed(&[
                    ("Kimi Code 0.7.2", Tone::Normal),
                    (" · kimi-k2 · ", Tone::Muted),
                    (&ws, Tone::Secondary),
                ]),
            ),
            e(200, vec![]),
            e(
                300,
                mixed(&[
                    ("› ", Tone::Muted),
                    ("add loading skeletons to the invoices table", Tone::Normal),
                ]),
            ),
            e(500, vec![]),
            e(300, bold("▸ Plan")),
            e(
                150,
                line("  1. Add <SkeletonRow/> in components/table/", Tone::Normal),
            ),
            e(
                150,
                line("  2. Render 8 rows while `isLoading`", Tone::Normal),
            ),
            e(150, line("  3. Story for the loading state", Tone::Normal)),
            e(700, vec![]),
            e(
                300,
                mixed(&[
                    ("▸ ", Tone::Secondary),
                    (
                        "Wrote components/table/SkeletonRow.tsx (41 lines)",
                        Tone::Normal,
                    ),
                ]),
            ),
            Step::Touch("components/table/SkeletonRow.tsx"),
            e(
                500,
                mixed(&[
                    ("▸ ", Tone::Secondary),
                    ("Edited pages/invoices/index.tsx (+9 −2)", Tone::Normal),
                ]),
            ),
            e(
                500,
                mixed(&[
                    ("▸ ", Tone::Secondary),
                    (
                        "Wrote components/table/SkeletonRow.stories.tsx",
                        Tone::Normal,
                    ),
                ]),
            ),
            e(600, vec![]),
            e(
                200,
                mixed(&[("✔ ", Tone::Success), ("Done · 3 files", Tone::Muted)]),
            ),
            Step::State(AgentState::Done),
            Step::Await,
        ],
        Some(Agent::OpenCode) => vec![
            Step::State(AgentState::Working),
            e(
                0,
                mixed(&[
                    ("opencode 0.5.11", Tone::Normal),
                    (" · ", Tone::Muted),
                    (&ws, Tone::Secondary),
                ]),
            ),
            e(200, vec![]),
            e(
                300,
                mixed(&[
                    ("> ", Tone::Muted),
                    (
                        "bump the release workflow to node 22 and pin actions",
                        Tone::Normal,
                    ),
                ]),
            ),
            e(500, vec![]),
            e(
                300,
                mixed(&[
                    ("┃ read  ", Tone::Secondary),
                    (".github/workflows/release.yml", Tone::Normal),
                ]),
            ),
            e(
                500,
                mixed(&[
                    ("┃ edit  ", Tone::Secondary),
                    (".github/workflows/release.yml", Tone::Normal),
                ]),
            ),
            Step::Touch(".github/workflows/release.yml"),
            e(
                150,
                mixed(&[
                    ("┃   - ", Tone::Error),
                    ("uses: actions/setup-node@v3", Tone::Muted),
                ]),
            ),
            e(
                150,
                mixed(&[
                    ("┃   + ", Tone::Success),
                    ("uses: actions/setup-node@v4.0.3", Tone::Secondary),
                ]),
            ),
            e(
                150,
                mixed(&[("┃   -   ", Tone::Error), ("node-version: 18", Tone::Muted)]),
            ),
            e(
                150,
                mixed(&[
                    ("┃   +   ", Tone::Success),
                    ("node-version: 22", Tone::Secondary),
                ]),
            ),
            e(
                500,
                mixed(&[
                    ("┃ edit  ", Tone::Secondary),
                    (".github/workflows/publish.yml (same pin)", Tone::Normal),
                ]),
            ),
            e(
                500,
                mixed(&[
                    ("┃ bash  ", Tone::Secondary),
                    ("act -j release --dryrun", Tone::Normal),
                ]),
            ),
            e(
                1200,
                mixed(&[
                    ("┃       ", Tone::Secondary),
                    ("✓ dry run ok (0 errors, 1 warning)", Tone::Muted),
                ]),
            ),
            e(600, vec![]),
            e(
                200,
                mixed(&[
                    ("Two workflows updated. ", Tone::Normal),
                    ("Warning: cache key still says node18.", Tone::Warning),
                ]),
            ),
            Step::State(AgentState::Idle),
            Step::Await,
        ],
        Some(Agent::GrokBuild) => vec![
            Step::State(AgentState::Working),
            e(
                0,
                mixed(&[
                    ("Grok Build 0.3", Tone::Normal),
                    (" · grok-4-code · ", Tone::Muted),
                    (&ws, Tone::Secondary),
                ]),
            ),
            e(200, vec![]),
            e(
                300,
                mixed(&[
                    ("> ", Tone::Muted),
                    (
                        "generate terraform for a private GKE node pool",
                        Tone::Normal,
                    ),
                ]),
            ),
            e(500, vec![]),
            e(
                300,
                mixed(&[
                    ("⟩ ", Tone::Secondary),
                    ("writing modules/gke/node_pool.tf", Tone::Normal),
                ]),
            ),
            Step::Touch("modules/gke/node_pool.tf"),
            e(
                500,
                mixed(&[
                    ("⟩ ", Tone::Secondary),
                    ("writing modules/gke/variables.tf", Tone::Normal),
                ]),
            ),
            e(
                700,
                mixed(&[
                    ("⟩ ", Tone::Secondary),
                    ("terraform fmt -check … ok", Tone::Muted),
                ]),
            ),
            e(
                700,
                mixed(&[
                    ("⟩ ", Tone::Secondary),
                    ("terraform validate … ok", Tone::Muted),
                ]),
            ),
            e(600, vec![]),
            e(
                200,
                line(
                    "Created 2 files (118 lines). Private nodes, 3 zones,",
                    Tone::Normal,
                ),
            ),
            e(80, line("e2-standard-8, autoscaling 1–6.", Tone::Normal)),
            Step::State(AgentState::Done),
            Step::Await,
        ],
        None => {
            let p = format!("{workspace} ❯ ");
            vec![
                e(
                    0,
                    mixed(&[(&p, Tone::Secondary), ("git status -sb", Tone::Normal)]),
                ),
                e(
                    300,
                    muted(
                        "## feature/settlement-backoff…origin/feature/settlement-backoff [ahead 2]",
                    ),
                ),
                e(
                    60,
                    mixed(&[
                        (" M ", Tone::Warning),
                        ("src/settlement/retry.rs", Tone::Normal),
                    ]),
                ),
                e(
                    60,
                    mixed(&[
                        (" M ", Tone::Warning),
                        ("src/settlement/mod.rs", Tone::Normal),
                    ]),
                ),
                e(
                    60,
                    mixed(&[
                        ("?? ", Tone::Muted),
                        ("docs/adr/0007-retry-backoff.md", Tone::Normal),
                    ]),
                ),
                e(
                    900,
                    mixed(&[
                        (&p, Tone::Secondary),
                        ("cargo clippy -p settlement", Tone::Normal),
                    ]),
                ),
                e(
                    500,
                    muted("    Checking settlement v0.9.2 (crates/settlement)"),
                ),
                e(
                    1500,
                    muted("    Finished dev [unoptimized] target(s) in 3.12s"),
                ),
                e(
                    700,
                    mixed(&[(&p, Tone::Secondary), ("ls docs/adr", Tone::Normal)]),
                ),
                e(
                    200,
                    line(
                        "0001-record-architecture.md  0004-ledger-precision.md",
                        Tone::Normal,
                    ),
                ),
                e(
                    40,
                    line(
                        "0002-settlement-batches.md   0005-fx-rounding.md",
                        Tone::Normal,
                    ),
                ),
                e(
                    40,
                    line(
                        "0003-retry-policy.md         0007-retry-backoff.md",
                        Tone::Normal,
                    ),
                ),
                Step::Await,
            ]
        }
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

    pub fn awaiting(&self) -> bool {
        self.pc > 0
            && matches!(
                self.script.get(self.pc.saturating_sub(1)),
                Some(Step::Await)
            )
            && self.reply_queue.is_empty()
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pane {
    pub id: PaneId,
    pub proc: AgentProcess,
    pub term: TextViewport,
    pub input: String,
    pub received_output: bool,
}

impl Pane {
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
            self.term.caret = Some(termrock::CellPos::new(line_number, col));
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
            self.term.remove_last();
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
                self.term.remove_last();
            }
            self.term.push(value);
            self.received_output = true;
        }
        self.refresh_prompt();
    }

    pub fn boot_all(&mut self) {
        let mut output = Vec::new();
        self.proc.boot_all(&mut output);
        self.push_lines(output);
        self.refresh_prompt();
    }

    pub fn tick(&mut self, now_ms: i64) -> bool {
        let output = self.proc.tick(now_ms);
        let changed = !output.is_empty();
        self.push_lines(output);
        self.refresh_prompt();
        changed
    }

    pub fn type_char(&mut self, character: char, _now_ms: i64, _workspace: &str) {
        self.term.set_follow(true);
        self.input.push(character);
        self.refresh_prompt();
    }

    pub fn backspace(&mut self) {
        self.term.set_follow(true);
        self.input.pop();
        self.refresh_prompt();
    }

    pub fn commit(&mut self, now_ms: i64, workspace: &str) {
        self.term.set_follow(true);
        let input = std::mem::take(&mut self.input);
        let pending_prompt = self
            .term
            .lines
            .last()
            .is_some_and(|value| value.first().is_some_and(|s| s.text == self.proc.prompt));
        if pending_prompt {
            self.term.remove_last();
        }
        let output = self.proc.on_input(&input, now_ms, workspace);
        self.push_lines(output);
        self.refresh_prompt();
    }

    pub fn clear(&mut self) {
        self.term.clear();
        self.term.set_follow(true);
        self.refresh_prompt();
    }

    pub const fn state(&self) -> AgentState {
        self.proc.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Daemon {
    pub workspace: String,
    pub tabs: Vec<Tab>,
    pub active: usize,
    pub panes: Vec<Pane>,
    pub next_id: PaneId,
    pub attached_by: Option<String>,
}

impl Daemon {
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

    pub fn from_snapshot(snapshot: &DaemonSnapshot, workspace: &str, now_ms: i64) -> Self {
        let workspace = workspace.strip_prefix("jackin-").unwrap_or(workspace);
        let mut daemon = Self::new(workspace);
        let DaemonSnapshot::Tabs(tabs) = snapshot else {
            return daemon;
        };
        for tab in tabs {
            let pane_ids = tab
                .panes
                .iter()
                .map(|pane| daemon.new_pane(pane.agent, None, now_ms, true))
                .collect::<Vec<_>>();
            let Some(first) = pane_ids.first().copied() else {
                continue;
            };
            let focused = tab
                .panes
                .iter()
                .position(|pane| pane.focused)
                .and_then(|index| pane_ids.get(index).copied())
                .unwrap_or(first);
            daemon.tabs.push(Tab {
                custom_label: None,
                root: Self::snapshot_root(&pane_ids),
                focused,
                zoomed: None,
            });
        }
        daemon.active = tabs
            .iter()
            .position(|tab| tab.active)
            .unwrap_or(0)
            .min(daemon.tabs.len().saturating_sub(1));
        daemon
    }

    fn snapshot_root(panes: &[PaneId]) -> PaneNode {
        match panes {
            [] => unreachable!("snapshot tabs without panes are skipped"),
            [pane] => PaneNode::Leaf(*pane),
            [first, second] => PaneNode::Split {
                dir: SplitDir::Horizontal,
                split: Split::new(50, MIN_PANE_COLS, MIN_PANE_COLS),
                first: Box::new(PaneNode::Leaf(*first)),
                second: Box::new(PaneNode::Leaf(*second)),
            },
            [first, second, third] => PaneNode::Split {
                dir: SplitDir::Horizontal,
                split: Split::new(50, MIN_PANE_COLS, MIN_PANE_COLS),
                first: Box::new(PaneNode::Leaf(*first)),
                second: Box::new(PaneNode::Split {
                    dir: SplitDir::Vertical,
                    split: Split::new(50, MIN_PANE_ROWS, MIN_PANE_ROWS),
                    first: Box::new(PaneNode::Leaf(*second)),
                    second: Box::new(PaneNode::Leaf(*third)),
                }),
            },
            [first, rest @ ..] => {
                let mut root = PaneNode::Leaf(*first);
                for pane in rest {
                    root = PaneNode::Split {
                        dir: SplitDir::Horizontal,
                        split: Split::new(50, MIN_PANE_COLS, MIN_PANE_COLS),
                        first: Box::new(root),
                        second: Box::new(PaneNode::Leaf(*pane)),
                    };
                }
                root
            }
        }
    }

    pub fn pane(&self, id: PaneId) -> Option<&Pane> {
        self.panes.iter().find(|pane| pane.id == id)
    }

    pub fn pane_mut(&mut self, id: PaneId) -> Option<&mut Pane> {
        self.panes.iter_mut().find(|pane| pane.id == id)
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active)
    }

    pub fn focused_pane(&self) -> Option<PaneId> {
        self.active_tab().map(|tab| tab.focused)
    }

    fn alloc(&mut self) -> PaneId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }

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

    pub fn tab_state(&self, tab: &Tab) -> AgentState {
        tab.leaves()
            .iter()
            .filter_map(|id| self.pane(*id).map(Pane::state))
            .max_by_key(|state| state.rank())
            .unwrap_or(AgentState::Unknown)
    }

    pub fn tick(&mut self, now_ms: i64) -> bool {
        let mut changed = false;
        for pane in &mut self.panes {
            changed |= pane.tick(now_ms);
        }
        changed
    }

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

    pub fn snapshot(&self) -> DaemonSnapshot {
        use jackin_preview_domain::instance::{PaneSnapshot, TabSnapshot};

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
