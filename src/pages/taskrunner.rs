//! Composed screen: a target tree, per-task progress and a following log.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::core::event::Outcome;
use crate::core::id::WidgetId;
use crate::data::TreeNode;
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use crate::ui::ctx::RenderCtx;
use crate::widgets::button::{Button, row_layout};
use crate::widgets::dialog::{Dialog, DialogResult};
use crate::widgets::panel::{Panel, ScrollPanel};
use crate::widgets::progress::{ProgressStatus, render_bar, render_indeterminate, spinner_frame};
use crate::widgets::scrollbar;
use crate::widgets::tree::TreeView;

const ID: WidgetId = WidgetId::of("taskrunner");
const CANCEL_DLG: WidgetId = ID.sub("cancel-dialog");

#[derive(Debug, Clone)]
struct Task {
    name: &'static str,
    progress: f64,
    state: TaskState,
    speed: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskState {
    Queued,
    Running,
    Done,
    Failed,
    Cancelled,
}

pub struct TaskRunnerPage {
    tree: TreeView,
    tasks: Vec<Task>,
    log: ScrollPanel,
    run: Button,
    cancel: Button,
    running: bool,
    ticks: u64,
}

fn targets() -> Vec<TreeNode> {
    vec![
        TreeNode::dir(
            "payments-gateway",
            vec![
                TreeNode::dir(
                    "build",
                    vec![
                        TreeNode::leaf("compile"),
                        TreeNode::leaf("lint"),
                        TreeNode::leaf("typecheck"),
                    ],
                ),
                TreeNode::dir(
                    "test",
                    vec![
                        TreeNode::leaf("unit"),
                        TreeNode::leaf("integration"),
                        TreeNode::leaf("e2e"),
                    ],
                ),
                TreeNode::dir(
                    "deploy",
                    vec![TreeNode::leaf("staging"), TreeNode::leaf("production")],
                ),
            ],
        ),
        TreeNode::dir(
            "shared-libs",
            vec![TreeNode::leaf("compile"), TreeNode::leaf("publish")],
        ),
    ]
}

impl TaskRunnerPage {
    pub fn new() -> Self {
        let mut tree = TreeView::new(ID.sub("tree"), targets());
        tree.expand_all();
        let tasks = ["compile", "lint", "typecheck", "unit", "integration", "e2e"]
            .iter()
            .enumerate()
            .map(|(i, n)| Task {
                name: n,
                progress: 0.0,
                state: TaskState::Queued,
                speed: 0.012 + (i as f64 % 3.0) * 0.006,
            })
            .collect();
        let mut log = ScrollPanel::new(
            ID.sub("log"),
            vec!["Ready. Press r or Run to start the pipeline.".into()],
        );
        log.follow = true;
        Self {
            tree,
            tasks,
            log,
            run: Button::primary(ID.sub("run"), "Run pipeline"),
            cancel: Button::secondary(ID.sub("cancel"), "Cancel"),
            running: false,
            ticks: 0,
        }
    }

    fn start(&mut self, cx: &mut PageCtx) {
        for t in &mut self.tasks {
            t.progress = 0.0;
            t.state = TaskState::Queued;
        }
        self.running = true;
        self.log.lines.clear();
        self.log.push("Pipeline started".into());
        self.log.follow = true;
        cx.status("Pipeline running");
    }

    fn cancel_now(&mut self, cx: &mut PageCtx) {
        self.running = false;
        for t in &mut self.tasks {
            if t.state == TaskState::Running || t.state == TaskState::Queued {
                t.state = TaskState::Cancelled;
            }
        }
        self.log.push("Pipeline cancelled by user".into());
        cx.status("Cancelled");
    }

    fn step(&mut self, cx: &mut PageCtx) -> bool {
        if !self.running {
            return false;
        }
        self.ticks += 1;
        // run up to two tasks concurrently
        let running = self
            .tasks
            .iter()
            .filter(|t| t.state == TaskState::Running)
            .count();
        if running < 2
            && let Some(t) = self.tasks.iter_mut().find(|t| t.state == TaskState::Queued)
        {
            t.state = TaskState::Running;
            self.log.push(format!("▶ {} started", t.name));
        }
        let mut changed = false;
        for i in 0..self.tasks.len() {
            let t = &mut self.tasks[i];
            if t.state != TaskState::Running {
                continue;
            }
            t.progress = (t.progress + t.speed).min(1.0);
            changed = true;
            if self.ticks.is_multiple_of(9) {
                self.log.push(format!(
                    "  {}: step {} of 12",
                    t.name,
                    ((t.progress * 12.0) as u32).min(12)
                ));
            }
            if t.progress >= 1.0 {
                if t.name == "integration" {
                    t.state = TaskState::Failed;
                    self.log.push(format!(
                        "✗ {} failed: checkout::places_order (assertion)",
                        t.name
                    ));
                } else {
                    t.state = TaskState::Done;
                    self.log.push(format!("✓ {} finished", t.name));
                }
            }
        }
        if self.tasks.iter().all(|t| {
            matches!(
                t.state,
                TaskState::Done | TaskState::Failed | TaskState::Cancelled
            )
        }) {
            self.running = false;
            let failed = self
                .tasks
                .iter()
                .filter(|t| t.state == TaskState::Failed)
                .count();
            if failed > 0 {
                self.log
                    .push(format!("Pipeline finished with {failed} failure"));
                cx.status(format!("{failed} task failed"));
            } else {
                self.log.push("Pipeline finished ✓".into());
                cx.status("Pipeline finished ✓");
            }
        }
        changed
    }
}

impl Page for TaskRunnerPage {
    fn title(&self) -> &'static str {
        "Task runner"
    }
    fn blurb(&self) -> &'static str {
        "Composed: tree, live progress, following log, busy states"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let (l, r) = crate::pages::layout::columns(area, 30, 2);
        let panel = Panel::card(Some("Targets")).focused(ctx.interaction.focused(self.tree.id));
        let bg = panel.bg(t);
        let th = (self.tree.rows().len() as u16 + 3).min(l.height);
        let inner = panel.render(Rect::new(l.x, l.y, l.width, th), buf, ctx.theme);
        self.tree.render(inner, buf, ctx, bg);

        let rrows = crate::pages::layout::rows(r, &[self.tasks.len() as u16 + 5, 1, 0]);
        let done = self
            .tasks
            .iter()
            .filter(|t| t.state == TaskState::Done)
            .count();
        let meta = format!("{done} of {} done", self.tasks.len());
        let title = if self.running {
            "Pipeline · running"
        } else {
            "Pipeline"
        };
        let panel = Panel::card(Some(title)).meta(&meta);
        let bg = panel.bg(t);
        let inner = panel.render(rrows[0], buf, t);
        for (i, task) in self.tasks.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            let label = format!("{:<12}", task.name);
            match task.state {
                TaskState::Queued => {
                    buf.set_string(inner.x, y, &label, t.muted().bg(bg));
                    buf.set_string(inner.x + 14, y, "queued", t.faint().bg(bg));
                }
                TaskState::Cancelled => {
                    buf.set_string(inner.x, y, &label, t.muted().bg(bg));
                    buf.set_string(inner.x + 14, y, "cancelled", t.faint().bg(bg));
                }
                TaskState::Running => {
                    buf.set_string(
                        inner.x,
                        y,
                        spinner_frame(ctx.interaction.tick),
                        t.accent_fg().bg(bg),
                    );
                    buf.set_string(
                        inner.x + 2,
                        y,
                        &label[..label.len().min(10)],
                        t.primary().bg(bg),
                    );
                    render_bar(
                        Rect::new(inner.x + 14, y, inner.width.saturating_sub(14).min(50), 1),
                        buf,
                        ctx,
                        "",
                        task.progress,
                        ProgressStatus::Active,
                        bg,
                    );
                }
                TaskState::Done => {
                    buf.set_string(inner.x, y, "✓", t.secondary().bg(bg));
                    buf.set_string(
                        inner.x + 2,
                        y,
                        &label[..label.len().min(10)],
                        t.secondary().bg(bg),
                    );
                    render_bar(
                        Rect::new(inner.x + 14, y, inner.width.saturating_sub(14).min(50), 1),
                        buf,
                        ctx,
                        "",
                        1.0,
                        ProgressStatus::Done,
                        bg,
                    );
                }
                TaskState::Failed => {
                    buf.set_string(inner.x, y, "!", t.error_fg().bg(bg));
                    buf.set_string(
                        inner.x + 2,
                        y,
                        &label[..label.len().min(10)],
                        t.primary().bg(bg),
                    );
                    render_bar(
                        Rect::new(inner.x + 14, y, inner.width.saturating_sub(14).min(50), 1),
                        buf,
                        ctx,
                        "",
                        task.progress,
                        ProgressStatus::Error,
                        bg,
                    );
                }
            }
        }
        let ay = inner.bottom().saturating_sub(1);
        if self.running {
            render_indeterminate(
                Rect::new(inner.x, ay - 1, inner.width.min(64), 1),
                buf,
                ctx,
                "Overall   ",
                bg,
            );
        }
        self.run.disabled = self.running;
        self.cancel.disabled = !self.running;
        let widths = [self.run.width(), self.cancel.width()];
        if widths[0] + 2 + widths[1] <= inner.width {
            let rects = row_layout(Rect::new(inner.x, ay, inner.width, 1), &widths, 2);
            self.run.render(rects[0], buf, ctx, bg);
            self.cancel.render(rects[1], buf, ctx, bg);
        } else if self.running {
            // narrow: only the actionable button; never truncate a label
            self.cancel
                .render(Rect::new(inner.x, ay, inner.width, 1), buf, ctx, bg);
        } else {
            self.run
                .render(Rect::new(inner.x, ay, inner.width, 1), buf, ctx, bg);
        }

        let lf = ctx.interaction.focused(self.log.id);
        let pos = scrollbar::position_label(&self.log.scroll);
        let meta = if self.log.follow {
            format!("{pos} · following")
        } else {
            pos
        };
        let panel = Panel::card(Some("Log")).focused(lf).meta(&meta);
        let bg = panel.bg(t);
        let inner = panel.render(rrows[2], buf, t);
        self.log.render(inner, buf, ctx, bg, |t, line| {
            if line.starts_with('✗') || line.contains("failure") || line.contains("cancelled") {
                t.error_fg()
            } else if line.starts_with('✓') || line.ends_with('✓') {
                t.accent_fg()
            } else if line.starts_with('▶') {
                t.primary()
            } else {
                t.secondary()
            }
        });
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Tick => {
                if self.step(cx) {
                    Outcome::Changed
                } else {
                    Outcome::Ignored
                }
            }
            PageEvent::Key(key) => {
                if key.is_char('r')
                    && !self.running
                    && !cx.focus.is(self.log.id)
                    && !cx.focus.is(self.tree.id)
                {
                    self.start(cx);
                    return Outcome::Changed;
                }
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                if f == self.tree.id {
                    return self.tree.on_key(key);
                }
                if f == self.log.id {
                    return self.log.on_key(key);
                }
                if f == self.run.id {
                    let (o, act) = self.run.on_key(key);
                    if act {
                        self.start(cx);
                    }
                    return o;
                }
                if f == self.cancel.id {
                    let (o, act) = self.cancel.on_key(key);
                    if act {
                        cx.open(Dialog::destructive(
                            CANCEL_DLG,
                            "Cancel pipeline?",
                            "Running tasks stop immediately. Finished tasks keep their results.",
                            "Cancel pipeline",
                        ));
                    }
                    return o;
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, pos } => {
                if let Some((row, toggle)) = self.tree.locate(*id) {
                    cx.focus.focus(self.tree.id);
                    return if toggle {
                        self.tree.on_click_toggle(row)
                    } else {
                        self.tree.on_click_row(row)
                    };
                }
                if *id == self.run.id {
                    if self.run.on_click() {
                        self.start(cx);
                    }
                    return Outcome::Changed;
                }
                if *id == self.cancel.id {
                    if self.cancel.on_click() {
                        cx.open(Dialog::destructive(
                            CANCEL_DLG,
                            "Cancel pipeline?",
                            "Running tasks stop immediately. Finished tasks keep their results.",
                            "Cancel pipeline",
                        ));
                    }
                    return Outcome::Changed;
                }
                if *id == scrollbar::id_for(self.log.id) {
                    return self.log.on_scrollbar(*pos);
                }
                if *id == scrollbar::id_for(self.tree.id) {
                    return self.tree.on_scrollbar(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } => {
                if *pressed == scrollbar::id_for(self.log.id) {
                    return self.log.on_scrollbar(*pos);
                }
                if *pressed == scrollbar::id_for(self.tree.id) {
                    return self.tree.on_scrollbar(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Wheel { id, delta } => {
                if self.tree.owns(*id) {
                    return self.tree.on_wheel(*delta);
                }
                if *id == self.log.id || *id == scrollbar::id_for(self.log.id) {
                    return self.log.on_wheel(*delta);
                }
                Outcome::Ignored
            }
            PageEvent::DialogClosed { id, result, .. } if *id == CANCEL_DLG => {
                if *result == DialogResult::Action(1) {
                    self.cancel_now(cx);
                }
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn animating(&self) -> bool {
        self.running
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if focus == Some(self.log.id) {
            vec![("↑ ↓", "Scroll"), ("f", "Follow"), ("r", "Run")]
        } else if focus == Some(self.tree.id) {
            vec![("↑ ↓", "Move"), ("← →", "Fold")]
        } else {
            vec![("r", "Run pipeline"), ("Enter", "Activate")]
        }
    }
}
