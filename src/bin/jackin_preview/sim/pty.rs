//! Simulated Capsule daemon: tabs, a nested pane tree, PTY-backed panes
//! whose content is a `TextViewport`, and deterministic agent processes
//! that emit believable transcripts over virtual time and answer input.

use std::collections::VecDeque;

use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::layout::{Split, SplitDir};
use junie_tui::widgets::viewport::{Line, Span, TextViewport};
use ratatui::layout::Rect;

use crate::domain::account::AccountId;
use crate::domain::agent::Agent;
use crate::domain::instance::AgentState;

pub type PaneId = u64;

pub const SCROLLBACK: usize = 2_000;
pub const MIN_PANE_COLS: u16 = 20;
pub const MIN_PANE_ROWS: u16 = 4;
pub const MAX_LABEL: usize = 16;

#[derive(Debug, Clone)]
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
            PaneNode::Leaf(id) => vec![*id],
            PaneNode::Split { first, second, .. } => {
                let mut v = first.leaves();
                v.extend(second.leaves());
                v
            }
        }
    }

    /// Geometry of every leaf and every seam (path, dir, container rect).
    pub fn layout(&self, area: Rect, out: &mut Vec<(PaneId, Rect)>, seams: &mut Vec<Seam>, path: &mut Vec<u8>) {
        match self {
            PaneNode::Leaf(id) => out.push((*id, area)),
            PaneNode::Split {
                dir,
                split,
                first,
                second,
            } => {
                let (a, b) = split.layout(*dir, area, 1);
                let handle = split.handle(*dir, area, 1);
                seams.push(Seam {
                    path: path.clone(),
                    dir: *dir,
                    container: area,
                    handle,
                });
                path.push(0);
                first.layout(a, out, seams, path);
                path.pop();
                path.push(1);
                second.layout(b, out, seams, path);
                path.pop();
            }
        }
    }

    pub fn node_at_mut(&mut self, path: &[u8]) -> Option<&mut PaneNode> {
        let mut cur = self;
        for p in path {
            match cur {
                PaneNode::Split { first, second, .. } => {
                    cur = if *p == 0 { first } else { second };
                }
                PaneNode::Leaf(_) => return None,
            }
        }
        Some(cur)
    }

    /// Replace the leaf `target` with a split holding it and `new`.
    pub fn split_leaf(&mut self, target: PaneId, new: PaneId, dir: SplitDir, new_first: bool) -> bool {
        match self {
            PaneNode::Leaf(id) if *id == target => {
                let old = PaneNode::Leaf(*id);
                let fresh = PaneNode::Leaf(new);
                let (first, second) = if new_first { (fresh, old) } else { (old, fresh) };
                *self = PaneNode::Split {
                    dir,
                    split: Split::new(50, 1, 1),
                    first: Box::new(first),
                    second: Box::new(second),
                };
                true
            }
            PaneNode::Leaf(_) => false,
            PaneNode::Split { first, second, .. } => {
                first.split_leaf(target, new, dir, new_first) || second.split_leaf(target, new, dir, new_first)
            }
        }
    }

    /// Remove a leaf; the parent split collapses into the sibling.
    pub fn remove_leaf(&mut self, target: PaneId) -> bool {
        if let PaneNode::Split { first, second, .. } = self {
            if matches!(**first, PaneNode::Leaf(id) if id == target) {
                let s = std::mem::replace(&mut **second, PaneNode::Leaf(0));
                *self = s;
                return true;
            }
            if matches!(**second, PaneNode::Leaf(id) if id == target) {
                let f = std::mem::replace(&mut **first, PaneNode::Leaf(0));
                *self = f;
                return true;
            }
            return first.remove_leaf(target) || second.remove_leaf(target);
        }
        false
    }

    /// Path (0/1 steps) from the root to a leaf.
    pub fn path_to(&self, target: PaneId, path: &mut Vec<u8>) -> bool {
        match self {
            PaneNode::Leaf(id) => *id == target,
            PaneNode::Split { first, second, .. } => {
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

#[derive(Debug, Clone)]
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

/// Nearest leaf in a direction from `from`, by centre distance.
pub fn nearest(leaves: &[(PaneId, Rect)], from: PaneId, dir: Direction) -> Option<PaneId> {
    let (_, fr) = leaves.iter().find(|(id, _)| *id == from)?;
    let (fx, fy) = (fr.x as i32 + fr.width as i32 / 2, fr.y as i32 + fr.height as i32 / 2);
    leaves
        .iter()
        .filter(|(id, r)| {
            *id != from
                && match dir {
                    Direction::Left => r.right() as i32 <= fr.x as i32,
                    Direction::Right => r.x as i32 >= fr.right() as i32,
                    Direction::Up => r.bottom() as i32 <= fr.y as i32,
                    Direction::Down => r.y as i32 >= fr.bottom() as i32,
                }
        })
        .min_by_key(|(_, r)| {
            let cx = r.x as i32 + r.width as i32 / 2;
            let cy = r.y as i32 + r.height as i32 / 2;
            (cx - fx).abs() + (cy - fy).abs()
        })
        .map(|(id, _)| *id)
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: u64,
    pub custom_label: Option<String>,
    pub root: PaneNode,
    pub focused: PaneId,
    pub zoomed: Option<PaneId>,
}

impl Tab {
    pub fn leaves(&self) -> Vec<PaneId> {
        self.root.leaves()
    }
}

// ------------------------------------------------------------ processes

#[derive(Debug, Clone)]
pub enum Step {
    /// Emit a line after `delay` virtual ms.
    Emit(i64, Line),
    State(AgentState),
    HideCursor(bool),
    /// Touch a file: marks the repo dirty.
    Touch(&'static str),
    /// Wait for operator input.
    Await,
}

#[derive(Debug, Clone)]
pub struct AgentProcess {
    pub agent: Option<Agent>,
    pub account: Option<AccountId>,
    pub state: AgentState,
    pub script: Vec<Step>,
    pub pc: usize,
    pub next_at_ms: i64,
    /// The current Emit step's delay has been scheduled into `next_at_ms`.
    scheduled: bool,
    pub reply_queue: VecDeque<(i64, Line)>,
    pub reply_cursor: usize,
    pub prompt: String,
    pub cursor_hidden: bool,
    pub awaiting_permission: bool,
    pub touched: Vec<String>,
}

fn sp(text: &str, tone: Tone) -> Span {
    Span::new(text, tone)
}

fn line(text: &str, tone: Tone) -> Line {
    vec![sp(text, tone)]
}

fn bold(text: &str) -> Line {
    vec![sp(text, Tone::Normal).bold()]
}

fn muted(text: &str) -> Line {
    line(text, Tone::Muted)
}

fn err(text: &str) -> Line {
    line(text, Tone::Error)
}

fn mixed(parts: &[(&str, Tone)]) -> Line {
    parts.iter().map(|(t, tone)| sp(t, *tone)).collect()
}

fn e(delay: i64, l: Line) -> Step {
    Step::Emit(delay, l)
}

/// Boot transcript per agent (paths synthetic, no secrets).
pub fn script(agent: Option<Agent>, workspace: &str) -> Vec<Step> {
    let ws = format!("~/{workspace}");
    match agent {
        Some(Agent::ClaudeCode) => vec![
            Step::State(AgentState::Working),
            e(0, mixed(&[("▐ ", Tone::Success), ("Claude Code v2.1.14", Tone::Normal), (" · Opus 4.5 · ", Tone::Muted), (&ws, Tone::Secondary)])),
            e(200, vec![]),
            e(300, mixed(&[("› ", Tone::Muted), ("Refactor the settlement retry loop so failed batches", Tone::Normal)])),
            e(60, line("  back off exponentially and cap at 5 attempts.", Tone::Normal)),
            e(400, vec![]),
            e(500, mixed(&[("● ", Tone::Secondary), ("I'll read the retry loop first.", Tone::Normal)])),
            e(500, vec![]),
            e(600, mixed(&[("● ", Tone::Secondary), ("Read ", Tone::Normal), ("src/settlement/retry.rs", Tone::Secondary), (" (142 lines)", Tone::Muted)])),
            e(400, mixed(&[("● ", Tone::Secondary), ("Read ", Tone::Normal), ("src/settlement/mod.rs", Tone::Secondary), (" (88 lines)", Tone::Muted)])),
            e(700, vec![]),
            e(200, mixed(&[("● ", Tone::Secondary), ("The loop retries with a fixed 3 attempts. I'll add", Tone::Normal)])),
            e(60, line("  exponential backoff with jitter and cap it at 5.", Tone::Normal)),
            e(800, vec![]),
            e(200, mixed(&[("● ", Tone::Secondary), ("Edit ", Tone::Normal), ("src/settlement/retry.rs", Tone::Secondary)])),
            Step::Touch("src/settlement/retry.rs"),
            e(200, mixed(&[("  +  ", Tone::Success), ("const MAX_ATTEMPTS: u32 = 5;", Tone::Secondary)])),
            e(120, mixed(&[("  +  ", Tone::Success), ("let delay = BASE * 2u32.pow(attempt) + jitter();", Tone::Secondary)])),
            e(120, mixed(&[("  -  ", Tone::Error), ("for attempt in 0..3 {", Tone::Muted)])),
            e(120, mixed(&[("  +  ", Tone::Success), ("for attempt in 0..MAX_ATTEMPTS {", Tone::Secondary)])),
            e(900, vec![]),
            e(200, mixed(&[("● ", Tone::Secondary), ("Bash ", Tone::Normal), ("cargo test -p settlement retry", Tone::Secondary)])),
            e(1400, muted("  running 6 tests … 6 passed (1.42 s)")),
            e(600, vec![]),
            e(200, mixed(&[("● ", Tone::Secondary), ("Edit ", Tone::Normal), ("src/settlement/mod.rs", Tone::Secondary)])),
            Step::Touch("src/settlement/mod.rs"),
            e(200, mixed(&[("  +  ", Tone::Success), ("pub use retry::{MAX_ATTEMPTS, RetryPolicy};", Tone::Secondary)])),
            e(800, vec![]),
            e(200, mixed(&[("● ", Tone::Secondary), ("Retries now back off 250 ms → 4 s, capped at 5 tries.", Tone::Normal)])),
            e(80, line("  One more edit: expose the policy in settlement config.", Tone::Normal)),
            e(600, vec![]),
            e(300, mixed(&[("▶ ", Tone::Warning), ("Allow edit to src/settlement/config.rs? (y/n)", Tone::Normal)])),
            Step::State(AgentState::Blocked),
            Step::Await,
        ],
        Some(Agent::Codex) => vec![
            Step::State(AgentState::Working),
            e(0, mixed(&[("OpenAI Codex v0.48", Tone::Normal), (" · gpt-5.5-codex · ", Tone::Muted), (&ws, Tone::Secondary)])),
            e(200, vec![]),
            e(300, mixed(&[("› ", Tone::Muted), ("run the ledger integration tests and summarise failures", Tone::Normal)])),
            e(500, vec![]),
            e(300, mixed(&[("• exec  ", Tone::Secondary), ("cargo test -p ledger --test integration", Tone::Normal)])),
            e(700, muted("  running 24 tests")),
            e(500, muted("  test reconcile::daily_close ........... ok")),
            e(400, mixed(&[("  test reconcile::multi_currency ........ ", Tone::Muted), ("FAILED", Tone::Error)])),
            e(400, muted("  test settle::partial_refund ........... ok")),
            e(900, mixed(&[("  22 passed · ", Tone::Muted), ("1 failed", Tone::Error), (" · 1 ignored (6.8 s)", Tone::Muted)])),
            e(600, vec![]),
            e(300, mixed(&[("• ", Tone::Secondary), ("1 failure: reconcile::multi_currency", Tone::Normal)])),
            e(200, line("  expected 1,204.50 EUR, got 1,204.49 EUR", Tone::Normal)),
            e(200, mixed(&[("  rounding precedes FX conversion in ", Tone::Normal), ("ledger/fx.rs:71", Tone::Secondary)])),
            e(700, vec![]),
            e(200, mixed(&[("○ ", Tone::Secondary), ("Done · 38 s · 12.4k tokens", Tone::Muted)])),
            Step::State(AgentState::Done),
            e(300, vec![]),
            Step::Await,
        ],
        Some(Agent::Amp) => vec![
            Step::State(AgentState::Working),
            e(0, mixed(&[("Amp 1.9.3", Tone::Normal), (" · ", Tone::Muted), (&ws, Tone::Secondary)])),
            e(200, vec![]),
            e(300, mixed(&[("› ", Tone::Muted), ("Why is the controller reconcile loop hot?", Tone::Normal)])),
            e(500, vec![]),
            e(300, mixed(&[("⠿ ", Tone::Success), ("Searching kube/controllers/**/*.go", Tone::Muted)])),
            e(600, mixed(&[("⠿ ", Tone::Success), ("Read kube/controllers/node_pool.go:118-164", Tone::Muted)])),
            e(800, vec![]),
            e(200, line("The loop re-queues every object on every informer resync", Tone::Normal)),
            e(80, line("(resyncPeriod = 30s) instead of only on spec changes.", Tone::Normal)),
            e(600, vec![]),
            e(200, line("Suggested change:", Tone::Normal)),
            e(150, mixed(&[("  - ", Tone::Error), (".WithEventFilter(predicate.ResourceVersionChangedPredicate{})", Tone::Muted)])),
            e(150, mixed(&[("  + ", Tone::Success), (".WithEventFilter(predicate.GenerationChangedPredicate{})", Tone::Secondary)])),
            e(600, vec![]),
            e(200, mixed(&[("Apply this edit? ", Tone::Normal), ("[Y/n]", Tone::Warning)])),
            Step::State(AgentState::Blocked),
            Step::Await,
        ],
        Some(Agent::KimiCode) => vec![
            Step::State(AgentState::Working),
            e(0, mixed(&[("Kimi Code 0.7.2", Tone::Normal), (" · kimi-k2 · ", Tone::Muted), (&ws, Tone::Secondary)])),
            e(200, vec![]),
            e(300, mixed(&[("› ", Tone::Muted), ("add loading skeletons to the invoices table", Tone::Normal)])),
            e(500, vec![]),
            e(300, bold("▸ Plan")),
            e(150, line("  1. Add <SkeletonRow/> in components/table/", Tone::Normal)),
            e(150, line("  2. Render 8 rows while `isLoading`", Tone::Normal)),
            e(150, line("  3. Story for the loading state", Tone::Normal)),
            e(700, vec![]),
            e(300, mixed(&[("▸ ", Tone::Secondary), ("Wrote components/table/SkeletonRow.tsx (41 lines)", Tone::Normal)])),
            Step::Touch("components/table/SkeletonRow.tsx"),
            e(500, mixed(&[("▸ ", Tone::Secondary), ("Edited pages/invoices/index.tsx (+9 −2)", Tone::Normal)])),
            e(500, mixed(&[("▸ ", Tone::Secondary), ("Wrote components/table/SkeletonRow.stories.tsx", Tone::Normal)])),
            e(600, vec![]),
            e(200, mixed(&[("✔ ", Tone::Success), ("Done · 3 files", Tone::Muted)])),
            Step::State(AgentState::Done),
            Step::Await,
        ],
        Some(Agent::OpenCode) => vec![
            Step::State(AgentState::Working),
            e(0, mixed(&[("opencode 0.5.11", Tone::Normal), (" · ", Tone::Muted), (&ws, Tone::Secondary)])),
            e(200, vec![]),
            e(300, mixed(&[("> ", Tone::Muted), ("bump the release workflow to node 22 and pin actions", Tone::Normal)])),
            e(500, vec![]),
            e(300, mixed(&[("┃ read  ", Tone::Secondary), (".github/workflows/release.yml", Tone::Normal)])),
            e(500, mixed(&[("┃ edit  ", Tone::Secondary), (".github/workflows/release.yml", Tone::Normal)])),
            Step::Touch(".github/workflows/release.yml"),
            e(150, mixed(&[("┃   - ", Tone::Error), ("uses: actions/setup-node@v3", Tone::Muted)])),
            e(150, mixed(&[("┃   + ", Tone::Success), ("uses: actions/setup-node@v4.0.3", Tone::Secondary)])),
            e(150, mixed(&[("┃   -   ", Tone::Error), ("node-version: 18", Tone::Muted)])),
            e(150, mixed(&[("┃   +   ", Tone::Success), ("node-version: 22", Tone::Secondary)])),
            e(500, mixed(&[("┃ edit  ", Tone::Secondary), (".github/workflows/publish.yml (same pin)", Tone::Normal)])),
            e(500, mixed(&[("┃ bash  ", Tone::Secondary), ("act -j release --dryrun", Tone::Normal)])),
            e(1200, mixed(&[("┃       ", Tone::Secondary), ("✓ dry run ok (0 errors, 1 warning)", Tone::Muted)])),
            e(600, vec![]),
            e(200, mixed(&[("Two workflows updated. ", Tone::Normal), ("Warning: cache key still says node18.", Tone::Warning)])),
            Step::State(AgentState::Idle),
            Step::Await,
        ],
        Some(Agent::GrokBuild) => vec![
            Step::State(AgentState::Working),
            e(0, mixed(&[("Grok Build 0.3", Tone::Normal), (" · grok-4-code · ", Tone::Muted), (&ws, Tone::Secondary)])),
            e(200, vec![]),
            e(300, mixed(&[("> ", Tone::Muted), ("generate terraform for a private GKE node pool", Tone::Normal)])),
            e(500, vec![]),
            e(300, mixed(&[("⟩ ", Tone::Secondary), ("writing modules/gke/node_pool.tf", Tone::Normal)])),
            Step::Touch("modules/gke/node_pool.tf"),
            e(500, mixed(&[("⟩ ", Tone::Secondary), ("writing modules/gke/variables.tf", Tone::Normal)])),
            e(700, mixed(&[("⟩ ", Tone::Secondary), ("terraform fmt -check … ok", Tone::Muted)])),
            e(700, mixed(&[("⟩ ", Tone::Secondary), ("terraform validate … ok", Tone::Muted)])),
            e(600, vec![]),
            e(200, line("Created 2 files (118 lines). Private nodes, 3 zones,", Tone::Normal)),
            e(80, line("e2-standard-8, autoscaling 1–6.", Tone::Normal)),
            Step::State(AgentState::Done),
            Step::Await,
        ],
        None => {
            let p = format!("{workspace} ❯ ");
            vec![
                e(0, mixed(&[(&p, Tone::Secondary), ("git status -sb", Tone::Normal)])),
                e(300, muted("## feature/settlement-backoff…origin/feature/settlement-backoff [ahead 2]")),
                e(60, mixed(&[(" M ", Tone::Warning), ("src/settlement/retry.rs", Tone::Normal)])),
                e(60, mixed(&[(" M ", Tone::Warning), ("src/settlement/mod.rs", Tone::Normal)])),
                e(60, mixed(&[("?? ", Tone::Muted), ("docs/adr/0007-retry-backoff.md", Tone::Normal)])),
                e(900, mixed(&[(&p, Tone::Secondary), ("cargo clippy -p settlement", Tone::Normal)])),
                e(500, muted("    Checking settlement v0.9.2 (crates/settlement)")),
                e(1500, muted("    Finished dev [unoptimized] target(s) in 3.12s")),
                e(700, mixed(&[(&p, Tone::Secondary), ("ls docs/adr", Tone::Normal)])),
                e(200, line("0001-record-architecture.md  0004-ledger-precision.md", Tone::Normal)),
                e(40, line("0002-settlement-batches.md   0005-fx-rounding.md", Tone::Normal)),
                e(40, line("0003-retry-policy.md         0007-retry-backoff.md", Tone::Normal)),
                Step::Await,
            ]
        }
    }
}

/// Canned replies (rotating) after operator input.
fn replies(agent: Option<Agent>) -> Vec<Vec<Line>> {
    match agent {
        Some(Agent::ClaudeCode) => vec![
            vec![
                mixed(&[("● ", Tone::Secondary), ("Read src/settlement/config.rs (61 lines)", Tone::Normal)]),
                mixed(&[("● ", Tone::Secondary), ("The policy is already wired; I'll add a test for the cap.", Tone::Normal)]),
                mixed(&[("● ", Tone::Secondary), ("Bash cargo test -p settlement cap", Tone::Normal)]),
                muted("  running 1 test … 1 passed (0.31 s)"),
                mixed(&[("○ ", Tone::Secondary), ("Done · 1 file changed", Tone::Muted)]),
            ],
            vec![
                mixed(&[("● ", Tone::Secondary), ("Grep MAX_ATTEMPTS src/ (3 matches)", Tone::Normal)]),
                mixed(&[("● ", Tone::Secondary), ("All callers use the constant; nothing else references 3.", Tone::Normal)]),
                mixed(&[("○ ", Tone::Secondary), ("Done", Tone::Muted)]),
            ],
            vec![
                mixed(&[("● ", Tone::Secondary), ("I can also open a PR with these changes.", Tone::Normal)]),
                mixed(&[("▶ ", Tone::Warning), ("Run gh pr create? (y/n)", Tone::Normal)]),
            ],
        ],
        Some(Agent::Codex) => vec![
            vec![
                mixed(&[("• exec  ", Tone::Secondary), ("cargo test -p ledger reconcile::multi_currency", Tone::Normal)]),
                muted("  running 1 test … ok (0.9 s)"),
                mixed(&[("○ ", Tone::Secondary), ("Done · 4 s · 1.1k tokens", Tone::Muted)]),
            ],
            vec![
                mixed(&[("• ", Tone::Secondary), ("Patched ledger/fx.rs:71 to round after conversion.", Tone::Normal)]),
                mixed(&[("○ ", Tone::Secondary), ("Done · 6 s · 2.4k tokens", Tone::Muted)]),
            ],
        ],
        None => vec![],
        _ => vec![
            vec![line("Understood. Working on it.", Tone::Normal), mixed(&[("○ ", Tone::Secondary), ("Done", Tone::Muted)])],
            vec![line("Nothing else to change here.", Tone::Normal)],
        ],
    }
}

impl AgentProcess {
    pub fn new(agent: Option<Agent>, account: Option<AccountId>, workspace: &str, start_ms: i64) -> Self {
        let prompt = match agent {
            Some(Agent::ClaudeCode) => "❯ ".to_owned(),
            Some(Agent::Codex) => "❯ ".to_owned(),
            Some(Agent::OpenCode) | Some(Agent::GrokBuild) => "> ".to_owned(),
            Some(_) => "› ".to_owned(),
            None => format!("{workspace} ❯ "),
        };
        Self {
            agent,
            account,
            state: if agent.is_some() { AgentState::Working } else { AgentState::Unknown },
            script: script(agent, workspace),
            pc: 0,
            next_at_ms: start_ms,
            scheduled: false,
            reply_queue: VecDeque::new(),
            reply_cursor: 0,
            prompt,
            cursor_hidden: false,
            awaiting_permission: false,
            touched: vec![],
        }
    }

    /// Fast-forward the boot script (fixtures that start mid-session).
    pub fn boot_all(&mut self, out: &mut Vec<Line>) {
        while self.pc < self.script.len() {
            match &self.script[self.pc] {
                Step::Emit(_, l) => out.push(l.clone()),
                Step::State(s) => self.state = *s,
                Step::HideCursor(h) => self.cursor_hidden = *h,
                Step::Touch(p) => self.touched.push((*p).to_owned()),
                Step::Await => {
                    self.pc += 1;
                    self.awaiting_permission = self.state == AgentState::Blocked;
                    return;
                }
            }
            self.pc += 1;
        }
    }

    fn awaiting(&self) -> bool {
        self.pc > 0 && matches!(self.script.get(self.pc - 1), Some(Step::Await)) && self.reply_queue.is_empty()
    }

    /// Advance; returns lines to emit now.
    pub fn tick(&mut self, now_ms: i64) -> Vec<Line> {
        let mut out = vec![];
        while let Some((due, _)) = self.reply_queue.front() {
            if *due <= now_ms {
                let (_, l) = self.reply_queue.pop_front().unwrap();
                out.push(l);
            } else {
                break;
            }
        }
        if self.reply_queue.is_empty() && self.state == AgentState::Working && self.awaiting() && !self.awaiting_permission {
            // reply finished
            self.state = AgentState::Done;
        }
        while self.pc < self.script.len() {
            match self.script[self.pc].clone() {
                Step::Emit(delay, l) => {
                    if !self.scheduled {
                        self.next_at_ms += delay;
                        self.scheduled = true;
                    }
                    if now_ms < self.next_at_ms {
                        break;
                    }
                    self.scheduled = false;
                    out.push(l);
                    self.pc += 1;
                }
                Step::State(s) => {
                    self.state = s;
                    self.pc += 1;
                }
                Step::HideCursor(h) => {
                    self.cursor_hidden = h;
                    self.pc += 1;
                }
                Step::Touch(p) => {
                    self.touched.push(p.to_owned());
                    self.pc += 1;
                }
                Step::Await => {
                    self.awaiting_permission = self.state == AgentState::Blocked;
                    self.pc += 1;
                    break;
                }
            }
        }
        out
    }

    /// Operator committed a line.
    pub fn on_input(&mut self, input: &str, now_ms: i64, workspace: &str) -> Vec<Line> {
        let mut out = vec![];
        let is_shell = self.agent.is_none();
        if !is_shell {
            out.push(mixed(&[(&self.prompt, Tone::Secondary), (input, Tone::Normal)]));
            if self.awaiting_permission {
                self.awaiting_permission = false;
                let yes = input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes");
                if yes {
                    self.state = AgentState::Working;
                    self.reply_queue.push_back((now_ms + 400, mixed(&[("● ", Tone::Secondary), ("Edit src/settlement/config.rs", Tone::Normal)])));
                    self.touched.push("src/settlement/config.rs".into());
                    self.reply_queue.push_back((now_ms + 900, mixed(&[("  +  ", Tone::Success), ("pub retry: RetryPolicy,", Tone::Secondary)])));
                    self.reply_queue.push_back((now_ms + 1500, mixed(&[("○ ", Tone::Secondary), ("Done · 3 files changed", Tone::Muted)])));
                } else {
                    self.state = AgentState::Idle;
                    self.reply_queue.push_back((now_ms + 300, muted("Skipped.")));
                }
                return out;
            }
            self.state = AgentState::Working;
            out.push(mixed(&[("⠋ ", Tone::Success), ("Thinking…", Tone::Muted)]));
            let rs = replies(self.agent);
            if !rs.is_empty() {
                let r = &rs[self.reply_cursor % rs.len()];
                self.reply_cursor += 1;
                let mut t = now_ms + 700;
                self.reply_queue.push_back((t, mixed(&[("↳ ", Tone::Muted), (&format!("\"{}\"", input.trim()), Tone::Muted)])));
                for l in r {
                    t += 350;
                    self.reply_queue.push_back((t, l.clone()));
                }
                if r.iter().any(|l| l.iter().any(|s| s.text.contains("(y/n)"))) {
                    self.awaiting_permission = true;
                    self.state = AgentState::Blocked;
                }
            }
            return out;
        }
        let p = self.prompt.clone();
        out.push(mixed(&[(&p, Tone::Secondary), (input, Tone::Normal)]));
        let cmd = input.trim();
        let mut words = cmd.split_whitespace();
        match words.next() {
            Some("ls") => {
                out.push(line("Cargo.toml   crates/   docs/   scripts/   README.md", Tone::Normal));
            }
            Some("pwd") => out.push(line(&format!("/workspace/{workspace}"), Tone::Normal)),
            Some("git") => match words.next() {
                Some("status") => {
                    out.push(muted("## feature/settlement-backoff…origin/feature/settlement-backoff [ahead 2]"));
                    for t in &self.touched {
                        out.push(mixed(&[(" M ", Tone::Warning), (t, Tone::Normal)]));
                    }
                }
                Some("push") => {
                    out.push(muted("To github.com:chainargos/payments-platform.git"));
                    out.push(mixed(&[("   9c41e2f..3f8a1d0  ", Tone::Muted), ("feature/settlement-backoff -> feature/settlement-backoff", Tone::Normal)]));
                }
                Some("log") => {
                    out.push(mixed(&[("3f8a1d0 ", Tone::Secondary), ("settlement: exponential backoff (#482)", Tone::Normal)]));
                    out.push(mixed(&[("9c41e2f ", Tone::Secondary), ("ledger: fx rounding order", Tone::Normal)]));
                }
                _ => out.push(muted("usage: git <command> [<args>]")),
            },
            Some("cargo") => {
                out.push(muted("    Checking settlement v0.9.2 (crates/settlement)"));
                out.push(muted("    Finished dev [unoptimized] target(s) in 2.04s"));
            }
            Some("clear") => {
                out.push(vec![sp("\u{0}clear", Tone::Normal)]);
            }
            Some("") | None => {}
            Some(other) => out.push(err(&format!("zsh: command not found: {other}"))),
        }
        out
    }
}

// ----------------------------------------------------------------- pane

#[derive(Debug, Clone)]
pub struct Pane {
    pub id: PaneId,
    pub proc: AgentProcess,
    pub term: TextViewport,
    pub input: String,
    pub cols: u16,
    pub rows: u16,
    pub received_output: bool,
}

impl Pane {
    pub fn new(id: PaneId, agent: Option<Agent>, account: Option<AccountId>, workspace: &str, start_ms: i64) -> Self {
        let mut term = TextViewport::new(WidgetId::of("capsule.pane").child(id as usize)).max_lines(SCROLLBACK);
        term.follow = true;
        Self {
            id,
            proc: AgentProcess::new(agent, account, workspace, start_ms),
            term,
            input: String::new(),
            cols: 80,
            rows: 24,
            received_output: false,
        }
    }

    pub fn label(&self) -> String {
        match self.proc.agent {
            Some(a) => a.label().to_owned(),
            None => "Shell".into(),
        }
    }

    fn prompt_line(&self) -> Line {
        vec![sp(&self.proc.prompt, Tone::Secondary), sp(&self.input, Tone::Normal)]
    }

    /// The live prompt row is always the last line while the process waits,
    /// and while the operator has typed ahead.
    fn refresh_prompt(&mut self) {
        let waiting = self.proc.awaiting() || !self.input.is_empty();
        if waiting {
            let last_is_prompt = self
                .term
                .lines
                .last()
                .is_some_and(|l| l.first().is_some_and(|s| s.text == self.proc.prompt));
            let pl = self.prompt_line();
            if last_is_prompt {
                self.term.replace_last(pl);
            } else {
                self.term.push(pl);
            }
            let n = self.term.lines.len().saturating_sub(1);
            let col = junie_tui::ui::text::width(&self.proc.prompt) + junie_tui::ui::text::width(&self.input);
            self.term.caret = Some(junie_tui::widgets::viewport::CellPos { line: n, col });
        } else {
            self.term.caret = None;
        }
        self.term.caret_visible = !self.proc.cursor_hidden;
    }

    fn push_lines(&mut self, lines: Vec<Line>) {
        if lines.is_empty() {
            return;
        }
        // remove a pending prompt row before appending output
        if self
            .term
            .lines
            .last()
            .is_some_and(|l| l.first().is_some_and(|s| s.text == self.proc.prompt))
        {
            self.term.lines.pop();
        }
        for l in lines {
            if l.first().is_some_and(|s| s.text == "\u{0}clear") {
                self.term.clear();
                continue;
            }
            if l.first().is_some_and(|s| s.text == "⠋ ") {
                // a spinner line replaces itself when the reply lands
                self.term.push(l);
                continue;
            }
            // drop a previous spinner row
            if self
                .term
                .lines
                .last()
                .is_some_and(|x| x.first().is_some_and(|s| s.text == "⠋ "))
            {
                self.term.lines.pop();
            }
            self.term.push(l);
            self.received_output = true;
        }
        self.refresh_prompt();
    }

    pub fn boot_all(&mut self) {
        let mut out = vec![];
        self.proc.boot_all(&mut out);
        self.push_lines(out);
        self.refresh_prompt();
    }

    pub fn tick(&mut self, now_ms: i64) -> bool {
        let out = self.proc.tick(now_ms);
        let changed = !out.is_empty();
        self.push_lines(out);
        self.refresh_prompt();
        changed
    }

    pub fn type_char(&mut self, c: char, now_ms: i64, workspace: &str) {
        // typing snaps to live
        self.term.set_follow(true);
        let _ = now_ms;
        let _ = workspace;
        if !self.proc.awaiting() && self.proc.agent.is_some() {
            // agent is working: queue the character into the input anyway
        }
        self.input.push(c);
        self.refresh_prompt();
    }

    pub fn backspace(&mut self) {
        self.term.set_follow(true);
        self.input.pop();
        self.refresh_prompt();
    }

    pub fn commit(&mut self, now_ms: i64, workspace: &str) {
        self.term.set_follow(true);
        let text = std::mem::take(&mut self.input);
        if self
            .term
            .lines
            .last()
            .is_some_and(|l| l.first().is_some_and(|s| s.text == self.proc.prompt))
        {
            self.term.lines.pop();
        }
        let out = self.proc.on_input(&text, now_ms, workspace);
        self.push_lines(out);
        self.refresh_prompt();
    }

    pub fn clear(&mut self) {
        self.term.clear();
        self.term.set_follow(true);
        self.refresh_prompt();
    }

    pub fn state(&self) -> AgentState {
        self.proc.state
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
    }
}

// --------------------------------------------------------------- daemon

#[derive(Debug, Clone)]
pub struct Daemon {
    pub instance: String,
    pub workspace: String,
    pub tabs: Vec<Tab>,
    pub active: usize,
    pub panes: Vec<Pane>,
    pub next_id: u64,
    /// Which client holds the single attach.
    pub attached_by: Option<String>,
    pub started_ms: i64,
}

impl Daemon {
    pub fn new(instance: &str, workspace: &str, now_ms: i64) -> Self {
        Self {
            instance: instance.to_owned(),
            workspace: workspace.to_owned(),
            tabs: vec![],
            active: 0,
            panes: vec![],
            next_id: 1,
            attached_by: None,
            started_ms: now_ms,
        }
    }

    pub fn pane(&self, id: PaneId) -> Option<&Pane> {
        self.panes.iter().find(|p| p.id == id)
    }

    pub fn pane_mut(&mut self, id: PaneId) -> Option<&mut Pane> {
        self.panes.iter_mut().find(|p| p.id == id)
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active)
    }

    pub fn focused_pane(&self) -> Option<PaneId> {
        self.active_tab().map(|t| t.focused)
    }

    fn alloc(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn new_pane(&mut self, agent: Option<Agent>, account: Option<AccountId>, now_ms: i64, boot: bool) -> PaneId {
        let id = self.alloc();
        let mut p = Pane::new(id, agent, account, &self.workspace, now_ms);
        if boot {
            p.boot_all();
        }
        self.panes.push(p);
        id
    }

    pub fn new_tab(&mut self, agent: Option<Agent>, account: Option<AccountId>, now_ms: i64, boot: bool) -> usize {
        let pid = self.new_pane(agent, account, now_ms, boot);
        let tid = self.alloc();
        self.tabs.push(Tab {
            id: tid,
            custom_label: None,
            root: PaneNode::Leaf(pid),
            focused: pid,
            zoomed: None,
        });
        self.active = self.tabs.len() - 1;
        self.active
    }

    /// Split the focused pane of the active tab. `new_first` places the new
    /// pane left/above.
    pub fn split(&mut self, dir: SplitDir, new_first: bool, agent: Option<Agent>, account: Option<AccountId>, now_ms: i64, boot: bool) -> Option<PaneId> {
        let target = self.focused_pane()?;
        let pid = self.new_pane(agent, account, now_ms, boot);
        let tab = self.active_tab_mut()?;
        if tab.root.split_leaf(target, pid, dir, new_first) {
            tab.focused = pid;
            tab.zoomed = None;
            Some(pid)
        } else {
            self.panes.retain(|p| p.id != pid);
            None
        }
    }

    /// Close a pane; returns true when the tab closed too.
    pub fn close_pane(&mut self, id: PaneId) -> bool {
        let Some(ti) = self.tabs.iter().position(|t| t.leaves().contains(&id)) else {
            return false;
        };
        self.panes.retain(|p| p.id != id);
        let tab = &mut self.tabs[ti];
        if tab.leaves().len() <= 1 {
            self.tabs.remove(ti);
            if self.active >= self.tabs.len() {
                self.active = self.tabs.len().saturating_sub(1);
            }
            return true;
        }
        tab.root.remove_leaf(id);
        if tab.focused == id {
            tab.focused = tab.leaves()[0];
        }
        if tab.zoomed == Some(id) {
            tab.zoomed = None;
        }
        false
    }

    pub fn close_tab(&mut self, i: usize) {
        if i >= self.tabs.len() {
            return;
        }
        let leaves = self.tabs[i].leaves();
        self.panes.retain(|p| !leaves.contains(&p.id));
        self.tabs.remove(i);
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len().saturating_sub(1);
        }
    }

    /// Auto label: Shell / agent label / Agents / Mix, with a count.
    pub fn tab_label(&self, tab: &Tab, account_suffix: &dyn Fn(&Pane) -> Option<String>) -> String {
        if let Some(c) = &tab.custom_label {
            return c.clone();
        }
        let leaves = tab.leaves();
        let mut agents: Vec<String> = vec![];
        let mut has_shell = false;
        for id in &leaves {
            if let Some(p) = self.pane(*id) {
                match p.proc.agent {
                    Some(a) => {
                        let mut l = a.label().to_owned();
                        if let Some(s) = account_suffix(p) {
                            l = format!("{l} ({s})");
                        }
                        if !agents.contains(&l) {
                            agents.push(l);
                        }
                    }
                    None => has_shell = true,
                }
            }
        }
        let base = match (agents.len(), has_shell) {
            (0, _) => "Shell".to_owned(),
            (1, false) => agents[0].clone(),
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
            .filter_map(|id| self.pane(*id).map(|p| p.state()))
            .max_by_key(|s| s.rank())
            .unwrap_or(AgentState::Unknown)
    }

    pub fn tick(&mut self, now_ms: i64) -> bool {
        let mut changed = false;
        for p in &mut self.panes {
            changed |= p.tick(now_ms);
        }
        changed
    }

    /// Repos dirtied by agents in this instance.
    pub fn touched_files(&self) -> Vec<String> {
        let mut v: Vec<String> = self.panes.iter().flat_map(|p| p.proc.touched.clone()).collect();
        v.sort();
        v.dedup();
        v
    }

    pub fn snapshot(&self) -> crate::domain::instance::DaemonSnapshot {
        use crate::domain::instance::{DaemonSnapshot, PaneSnapshot, TabSnapshot};
        if self.tabs.is_empty() {
            return DaemonSnapshot::NoTabs;
        }
        DaemonSnapshot::Tabs(
            self.tabs
                .iter()
                .enumerate()
                .map(|(i, t)| TabSnapshot {
                    label: self.tab_label(t, &|_| None),
                    active: i == self.active,
                    panes: t
                        .leaves()
                        .iter()
                        .filter_map(|id| self.pane(*id))
                        .map(|p| PaneSnapshot {
                            label: p.label(),
                            agent: p.proc.agent,
                            state: p.state(),
                            focused: t.focused == p.id,
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
    fn split_close_and_nearest() {
        let mut d = Daemon::new("jk-1", "payments-platform", 0);
        d.new_tab(Some(Agent::ClaudeCode), None, 0, false);
        let right = d.split(SplitDir::Horizontal, false, None, None, 0, false).unwrap();
        let below = d.split(SplitDir::Vertical, false, Some(Agent::Codex), None, 0, false).unwrap();
        let tab = d.active_tab().unwrap();
        assert_eq!(tab.leaves().len(), 3);
        assert_eq!(tab.focused, below);
        let mut leaves = vec![];
        let mut seams = vec![];
        tab.root.layout(Rect::new(0, 0, 120, 40), &mut leaves, &mut seams, &mut vec![]);
        assert_eq!(seams.len(), 2);
        assert_eq!(nearest(&leaves, below, Direction::Up), Some(right));
        assert_eq!(nearest(&leaves, below, Direction::Left), Some(1));
        assert_eq!(nearest(&leaves, 1, Direction::Left), None);
        assert!(!d.close_pane(below));
        assert_eq!(d.active_tab().unwrap().leaves().len(), 2);
        assert_eq!(d.tab_label(d.active_tab().unwrap(), &|_| None), "Mix (2)");
        assert!(!d.close_pane(right));
        assert!(d.close_pane(1));
        assert!(d.tabs.is_empty());
    }

    #[test]
    fn agent_process_emits_boots_and_replies() {
        let mut p = Pane::new(1, Some(Agent::Codex), None, "payments-platform", 0);
        let mut t = 0;
        while p.proc.pc < p.proc.script.len() && t < 60_000 {
            t += 100;
            p.tick(t);
        }
        assert_eq!(p.state(), AgentState::Done);
        assert!(p.term.lines.iter().any(|l| l.iter().any(|s| s.text.contains("FAILED"))));
        for c in "hi".chars() {
            p.type_char(c, t, "payments-platform");
        }
        p.commit(t, "payments-platform");
        assert_eq!(p.state(), AgentState::Working);
        for _ in 0..40 {
            t += 100;
            p.tick(t);
        }
        assert_eq!(p.state(), AgentState::Done);
        assert!(p.term.lines.iter().any(|l| l.iter().any(|s| s.text.contains("\"hi\""))));
        let mut sh = Pane::new(2, None, None, "payments-platform", 0);
        sh.boot_all();
        assert!(sh.term.caret.is_some());
        for c in "git push".chars() {
            sh.type_char(c, 0, "payments-platform");
        }
        sh.commit(0, "payments-platform");
        assert!(sh.term.lines.iter().any(|l| l.iter().any(|s| s.text.contains("github.com"))));
    }
}
