//! Terminal-style surfaces: a text viewport with scrollback, follow-tail,
//! selection and copy; a draggable splitter between two panes; and a step
//! rail that reports a multi-stage job.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layout::{Split, SplitDir};
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::splitter::Splitter;
use junie_tui::widgets::steps::{Step, StepRail, StepState};
use junie_tui::widgets::viewport::{Span, TextViewport, ViewportEvent};

const ID: WidgetId = WidgetId::of("terminal");
const SEAM: WidgetId = WidgetId::of("terminal.seam");

const STAGES: [&str; 7] = [
    "Resolve workspace",
    "Pull base image",
    "Build container",
    "Mount sources",
    "Resolve credentials",
    "Start agent",
    "Ready",
];

const DURATIONS: [u32; 7] = [10, 26, 40, 8, 18, 14, 1];

pub struct TerminalPage {
    term: TextViewport,
    rail: StepRail,
    split: Split,
    seam: Splitter,
    container: Rect,
    run: Button,
    fail: Button,
    tick: u32,
    stage: usize,
    stage_tick: u32,
    running: bool,
    fail_at: Option<usize>,
    copied: Option<String>,
}

fn stage_lines(stage: usize, tick: u32) -> Vec<Vec<Span>> {
    let mut v = vec![];
    if tick == 0 {
        v.push(vec![
            Span::new(format!("▶ {}", STAGES[stage]), Tone::Secondary).bold(),
        ]);
    }
    match stage {
        1 if tick.is_multiple_of(4) => v.push(vec![Span::muted(format!(
            "  layer {:02}/12  {:>3}%",
            (tick / 4 + 1).min(12),
            ((tick + 1) * 100 / DURATIONS[1]).min(100)
        ))]),
        2 if tick.is_multiple_of(3) => v.push(vec![
            Span::muted("  #"),
            Span::plain(format!("{} ", tick / 3 + 1)),
            Span::muted(match tick / 3 % 4 {
                0 => "RUN apt-get install -y build-essential",
                1 => "COPY rust-toolchain.toml ./",
                2 => "RUN cargo fetch --locked",
                _ => "RUN cargo build --release",
            }),
        ]),
        3 if tick == 2 => v.push(vec![Span::muted(
            "  ~/src/payments-platform → /workspace/payments-platform  rw",
        )]),
        3 if tick == 5 => v.push(vec![Span::muted(
            "  ~/src/shared-libs → /workspace/libs  ro",
        )]),
        4 if tick == 6 => v.push(vec![Span::muted(
            "  1Password  Engineering › Anthropic · Work › credential  ok",
        )]),
        5 if tick == 4 => v.push(vec![Span::muted("  claude --resume  pid 4188")]),
        _ => {}
    }
    v
}

impl TerminalPage {
    pub fn new() -> Self {
        let mut term = TextViewport::new(ID.sub("term")).max_lines(2000);
        term.push(vec![
            Span::plain("payments-platform ❯ ").bold(),
            Span::plain("jackin launch"),
        ]);
        let rail = StepRail::new(
            ID.sub("rail"),
            STAGES.iter().map(|s| Step::new(s)).collect(),
        );
        let mut p = Self {
            term,
            rail,
            split: Split::new(62, 30, 28),
            seam: Splitter::new(SEAM, SplitDir::Horizontal),
            container: Rect::ZERO,
            run: Button::primary(ID.sub("run"), "Run"),
            fail: Button::secondary(ID.sub("fail"), "Run with a failure"),
            tick: 0,
            stage: 0,
            stage_tick: 0,
            running: false,
            fail_at: None,
            copied: None,
        };
        p.reset(None);
        p
    }

    fn reset(&mut self, fail_at: Option<usize>) {
        self.tick = 0;
        self.stage = 0;
        self.stage_tick = 0;
        self.running = true;
        self.fail_at = fail_at;
        for i in 0..STAGES.len() {
            self.rail.set_state(i, StepState::Queued);
            self.rail.set_meta(i, None);
        }
        self.rail.set_state(0, StepState::Running);
        self.term.clear();
        self.term.push(vec![
            Span::plain("payments-platform ❯ ").bold(),
            Span::plain("jackin launch"),
        ]);
        self.term.set_follow(true);
    }

    fn advance(&mut self) -> Outcome {
        if !self.running {
            return Outcome::Ignored;
        }
        self.tick += 1;
        for line in stage_lines(self.stage, self.stage_tick) {
            self.term.push(line);
        }
        self.stage_tick += 1;
        self.rail.set_meta(
            self.stage,
            Some(format!("{:.1} s", self.stage_tick as f32 * 0.08)),
        );
        if self.fail_at == Some(self.stage) && self.stage_tick >= DURATIONS[self.stage] / 2 {
            self.rail.set_state(self.stage, StepState::Failed);
            self.rail.set_meta(self.stage, Some("exit 1".into()));
            for i in self.stage + 1..STAGES.len() {
                self.rail.set_state(i, StepState::Blocked);
            }
            self.term.push(vec![
                Span::new("✗ ", Tone::Error),
                Span::new(
                    format!(
                        "{} failed: network unreachable (curl: 6)",
                        STAGES[self.stage]
                    ),
                    Tone::Error,
                ),
            ]);
            self.running = false;
            return Outcome::Changed;
        }
        if self.stage_tick >= DURATIONS[self.stage] {
            self.rail.set_state(self.stage, StepState::Done);
            self.term.push(vec![
                Span::new("✓ ", Tone::Secondary),
                Span::muted(STAGES[self.stage]),
            ]);
            self.stage += 1;
            self.stage_tick = 0;
            if self.stage >= STAGES.len() {
                self.running = false;
                self.term
                    .push(vec![Span::plain("payments-platform ❯ ").bold()]);
            } else {
                if self.stage == 3 {
                    // one stage is skipped when nothing needs mounting twice
                    self.rail.set_state(self.stage, StepState::Skipped);
                    self.rail.set_meta(self.stage, Some("cached".into()));
                    self.stage += 1;
                }
                self.rail.set_state(self.stage, StepState::Running);
            }
        }
        Outcome::Changed
    }
}

impl Page for TerminalPage {
    fn title(&self) -> &'static str {
        "Terminal"
    }
    fn blurb(&self) -> &'static str {
        "Viewport with scrollback, selection and copy · drag the seam · step rail reports a job"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        self.container = area;
        let (left, right) = self.split.horizontal(area, 2);
        let handle = self.split.handle(SplitDir::Horizontal, area, 2);
        let depth = self.term.scrollback_depth();
        let meta = if self.term.has_selection() {
            "selection · y copies".to_owned()
        } else if self.term.is_at_tail() {
            format!("{} lines · following", self.term.len())
        } else {
            format!("scrollback ↑{depth}")
        };
        let panel = Panel::framed(Some("Viewport"))
            .meta(&meta)
            .focused(ctx.interaction.focused(self.term.id));
        let bg = panel.bg(t);
        let inner = panel.render(left, buf, t);
        self.term.render(inner, buf, ctx, bg);
        self.seam.render(
            Rect::new(handle.x + 1, handle.y, 1, handle.height),
            buf,
            ctx,
            t.canvas,
        );

        let (done, skipped, failed) = self.rail.counts();
        let total = STAGES.len();
        let done = done + skipped;
        let meta = if failed > 0 {
            format!("{done} of {total} · failed")
        } else if self.running {
            format!("{done} of {total} · running")
        } else {
            format!("{done} of {total}")
        };
        let panel = Panel::card(Some("Step rail"))
            .meta(&meta)
            .focused(ctx.interaction.focused(self.rail.id));
        let bg = panel.bg(t);
        let inner = panel.render(right, buf, t);
        let rail_area = Rect::new(
            inner.x,
            inner.y,
            inner.width,
            inner.height.saturating_sub(3),
        );
        self.rail.render(rail_area, buf, ctx, bg);
        let rects = row_layout(
            Rect::new(inner.x, inner.bottom().saturating_sub(1), inner.width, 1),
            &[self.run.width(), self.fail.width()],
            2,
        );
        self.run.render(rects[0], buf, ctx, bg);
        self.fail.render(rects[1], buf, ctx, bg);
        if let Some(c) = &self.copied {
            let y = inner.bottom().saturating_sub(3);
            buf.set_string(
                inner.x,
                y,
                junie_tui::ui::text::truncate(&format!("copied: {c}"), inner.width as usize),
                t.muted().bg(bg),
            );
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Tick => self.advance(),
            PageEvent::Key(key) => {
                if cx.focus.is(self.term.id) {
                    let (o, ev) = self.term.on_key(key);
                    match ev {
                        Some(ViewportEvent::Copy(text)) => {
                            let n = text.lines().count();
                            self.copied = Some(text.lines().next().unwrap_or("").to_owned());
                            cx.status(format!("Copied {n} line{}", if n == 1 { "" } else { "s" }));
                            return Outcome::Changed;
                        }
                        Some(ViewportEvent::FollowChanged(on)) => {
                            cx.status(if on {
                                "Following the tail"
                            } else {
                                "Paused at the scrollback position"
                            });
                            return Outcome::Changed;
                        }
                        _ => {}
                    }
                    return o;
                }
                if cx.focus.is(self.rail.id) {
                    return self.rail.on_key(key);
                }
                if cx.focus.is(self.run.id) {
                    let (o, act) = self.run.on_key(key);
                    if act {
                        self.reset(None);
                    }
                    return o;
                }
                if cx.focus.is(self.fail.id) {
                    let (o, act) = self.fail.on_key(key);
                    if act {
                        self.reset(Some(1));
                    }
                    return o;
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, pos } => {
                if *id == self.term.id {
                    cx.focus.focus(self.term.id);
                    return self.term.on_click(*pos);
                }
                if *id == scrollbar::id_for(self.term.id) {
                    return self.term.on_scrollbar(*pos);
                }
                if let Some(row) = self.rail.locate(*id) {
                    cx.focus.focus(self.rail.id);
                    return self.rail.on_click(row);
                }
                if *id == self.run.id && self.run.on_click() {
                    self.reset(None);
                    return Outcome::Changed;
                }
                if *id == self.fail.id && self.fail.on_click() {
                    self.reset(Some(1));
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } => {
                if *pressed == SEAM {
                    return self.seam.on_drag(&mut self.split, self.container, 2, *pos);
                }
                if *pressed == self.term.id {
                    // the page sees no press event: anchor on the first drag
                    let o = self.term.on_drag(*pos);
                    if o == Outcome::Ignored {
                        self.term.on_click(*pos);
                        return self.term.on_drag(*pos);
                    }
                    return o;
                }
                if *pressed == scrollbar::id_for(self.term.id) {
                    return self.term.on_scrollbar(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Wheel { id, delta } => {
                if self.term.owns(*id) {
                    return self.term.on_wheel(*delta);
                }
                if self.rail.owns(*id) {
                    return self.rail.on_wheel(*delta);
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn animating(&self) -> bool {
        self.running
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if focus == Some(self.term.id) {
            vec![
                ("↑ ↓", "Scroll"),
                ("Home End", "Oldest / live"),
                ("f", "Follow"),
                ("drag", "Select"),
                ("y", "Copy"),
                ("Esc", "Clear"),
            ]
        } else if focus == Some(self.rail.id) {
            vec![("↑ ↓", "Move"), ("wheel", "Scroll")]
        } else {
            vec![("Enter", "Activate"), ("drag ┃", "Resize")]
        }
    }
}
