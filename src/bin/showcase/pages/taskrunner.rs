//! Composed screen: a target tree, per-task progress and a following log.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::Scrollable;
use junie_tui::theme::{EnvKind, Glyphs, StatusKind, Tone};
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layout::StackChild;
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::dialog::{Dialog, DialogResult};
use junie_tui::widgets::panel::{Panel, ScrollPanel};
use junie_tui::widgets::progress::{ProgressStatus, render_bar, render_indeterminate};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::tree::TreeNode;
use junie_tui::widgets::tree::TreeView;

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
    sections: crate::pages::layout::Sections,
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
                    vec![
                        env("staging", EnvKind::Staging),
                        env("production", EnvKind::Production),
                    ],
                ),
            ],
        ),
        TreeNode::dir(
            "shared-libs",
            vec![TreeNode::leaf("compile"), TreeNode::leaf("publish")],
        ),
    ]
}

/// A deploy target: its environment's glyph and tone from the library
/// ([`Glyphs::env`], [`EnvKind::tone`]); production's label takes the
/// danger tone, so the riskiest target reads first.
fn env(name: &str, kind: EnvKind) -> TreeNode {
    let node = TreeNode::leaf(name)
        .glyph(Glyphs::default().env(kind))
        .glyph_tone(kind.tone());
    if kind == EnvKind::Production {
        node.label_tone(Tone::Danger)
    } else {
        node
    }
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
        let log = ScrollPanel::new(
            ID.sub("log"),
            vec!["Ready. Press r or Run to start the pipeline.".into()],
        )
        .tail(true);
        Self {
            tree,
            tasks,
            log,
            run: Button::primary(ID.sub("run"), "Run pipeline"),
            cancel: Button::secondary(ID.sub("cancel"), "Cancel"),
            running: false,
            ticks: 0,
            sections: crate::pages::layout::Sections::new(ID.sub("page")),
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
        // sections keep their full height and the page scrolls by section
        // on a short terminal: squeezed, the pipeline's rows ran into its
        // own buttons ("Run pipeline qu Cancel")
        let tree_h = self.tree.rows().len() as u16 + 3;
        let pipe_h = self.tasks.len() as u16 + 5;
        let has = |id| ctx.interaction.has_focus(id);
        let in_tree = has(self.tree.id);
        let in_pipe = has(self.run.id) || has(self.cancel.id);
        let in_log = has(self.log.id);
        self.run.disabled = self.running;
        self.cancel.disabled = !self.running;
        let log_kid = |h: u16| StackChild::flex(5, h.max(5));
        let (tree_slot, pipe_slot, log_slot) = if area.width >= 30 + 2 + 20 {
            let l = Rect::new(area.x, area.y, 30, area.height);
            let r = Rect::new(area.x + 32, area.y, area.width - 32, area.height);
            let focus = if in_pipe {
                Some(0)
            } else if in_log {
                Some(1)
            } else {
                None
            };
            let s = self.sections.layout(
                r,
                &[
                    StackChild::fixed(pipe_h),
                    log_kid(r.height.saturating_sub(pipe_h + 1)),
                ],
                focus,
                buf,
                ctx,
            );
            (
                Some(Rect::new(l.x, l.y, l.width, tree_h.min(l.height))),
                s.slot(0),
                s.slot(1),
            )
        } else {
            let focus = if in_tree {
                Some(0)
            } else if in_pipe {
                Some(1)
            } else if in_log {
                Some(2)
            } else {
                None
            };
            let s = self.sections.layout(
                area,
                &[
                    StackChild::flex(6, tree_h),
                    StackChild::fixed(pipe_h),
                    log_kid(area.height.saturating_sub(tree_h + pipe_h + 2)),
                ],
                focus,
                buf,
                ctx,
            );
            (s.slot(0), s.slot(1), s.slot(2))
        };
        match tree_slot {
            Some(r) => self.render_targets(r, buf, ctx),
            None => ctx.focus_stop(self.tree.id),
        }
        match pipe_slot {
            Some(r) => self.render_pipeline(r, buf, ctx),
            None => {
                for b in [&self.run, &self.cancel] {
                    if !b.disabled {
                        ctx.focus_stop(b.id);
                    }
                }
            }
        }
        match log_slot {
            Some(r) => self.render_log(r, buf, ctx),
            None => ctx.focus_stop(self.log.id),
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        if let Some(o) = self.sections.handle(ev) {
            return o;
        }
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
                    return self.tree.on_key(key).0;
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
                        )
                        .cancel_label("Keep running"));
                    }
                    return o;
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, pos } => {
                if let Some((row, toggle)) = self.tree.locate(*id) {
                    cx.focus.focus(self.tree.id);
                    return if toggle {
                        self.tree.on_click_toggle(row).0
                    } else {
                        self.tree.on_click_row(row).0
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
                        )
                        .cancel_label("Keep running"));
                    }
                    return Outcome::Changed;
                }
                if scrollbar::is_bar_of(*id, self.log.id) {
                    return self.log.on_scrollbar(*pos);
                }
                if *id == scrollbar::id_for(self.tree.id) {
                    return self.tree.on_scrollbar(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } => {
                if scrollbar::is_bar_of(*pressed, self.log.id) {
                    return self.log.on_bar_drag(*pressed, *pos);
                }
                if *pressed == scrollbar::id_for(self.tree.id) {
                    return self.tree.on_scrollbar_drag(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Wheel { id, delta } => {
                if self.tree.owns(*id) {
                    return self.tree.on_wheel(*delta);
                }
                if self.log.owns(*id) {
                    return self.log.on_wheel(*delta);
                }
                Outcome::Ignored
            }
            PageEvent::WheelH { id, delta } if self.log.owns(*id) => self.log.on_wheel_h(*delta),
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
            vec![("↑↓", "Scroll"), ("f", "Follow"), ("r", "Run")]
        } else if focus == Some(self.tree.id) {
            vec![("↑↓", "Move"), ("←→", "Fold")]
        } else {
            vec![("r", "Run pipeline"), ("Enter", "Activate")]
        }
    }
}

impl TaskRunnerPage {
    fn render_targets(&mut self, r: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let panel = Panel::card(Some("Targets")).focused(ctx.interaction.focused(self.tree.id));
        let bg = panel.bg(ctx.theme);
        if let Some(inner) = panel.try_render(r, buf, ctx.theme) {
            self.tree.render(inner, buf, ctx, bg);
        }
    }

    fn render_pipeline(&mut self, r: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
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
        let inner = panel.render(r, buf, t);
        for (i, task) in self.tasks.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            let label = format!("{:<12}", task.name);
            let name = junie_tui::ui::text::truncate(&label, 10);
            let bar = Rect::new(inner.x + 14, y, inner.width.saturating_sub(14).min(50), 1);
            // every state leads with its `Theme::status` glyph, so colour
            // is never the only cue (colour follow-ups, showcase)
            let kind = match task.state {
                TaskState::Queued => StatusKind::Queued,
                TaskState::Running => StatusKind::Running,
                TaskState::Done => StatusKind::Ok,
                TaskState::Failed => StatusKind::Fail,
                TaskState::Cancelled => StatusKind::Skipped,
            };
            let (glyph, glyph_style) = t.status_at(kind, ctx.interaction.tick);
            buf.set_string(inner.x, y, glyph, glyph_style.bg(bg));
            let name_style = match task.state {
                TaskState::Queued | TaskState::Cancelled => t.muted(),
                TaskState::Done => t.secondary(),
                // the running label is information, as `Theme::status_label`
                TaskState::Running => t.status_label(kind),
                TaskState::Failed => t.primary(),
            };
            buf.set_string(inner.x + 2, y, &name, name_style.bg(bg));
            let word = |buf: &mut Buffer, text: &str| {
                // the state word in the status hue, stepped up to read on
                // the card (skipped is faint)
                let fg = t.tone_on(kind.tone(), bg);
                buf.set_string(bar.x, y, text, ratatui::style::Style::new().fg(fg).bg(bg));
            };
            match task.state {
                TaskState::Queued => word(buf, "queued"),
                TaskState::Cancelled => word(buf, "cancelled"),
                TaskState::Running => {
                    render_bar(bar, buf, ctx, "", task.progress, ProgressStatus::Active, bg);
                }
                TaskState::Done => {
                    render_bar(bar, buf, ctx, "", 1.0, ProgressStatus::Done, bg);
                }
                TaskState::Failed => {
                    render_bar(bar, buf, ctx, "", task.progress, ProgressStatus::Error, bg);
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
    }

    fn render_log(&mut self, r: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let lf = ctx.interaction.focused(self.log.id);
        let panel = Panel::card(Some("Log")).focused(lf);
        let bg = panel.bg(t);
        let inner = panel.render(r, buf, t);
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
        let pos = scrollbar::position_label(&self.log.scroll);
        let meta = if self.log.follow {
            format!("{pos} · following")
        } else {
            pos
        };
        panel.draw_meta(r, buf, t, &meta);
    }
}
