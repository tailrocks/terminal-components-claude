//! Plans as graphs: the review page (a stage outline with inclusion,
//! `needs` and lane columns, a consequence line after exclusions, and the
//! selected step's facts) and the executing plan tab (the same outline with
//! live states, aggregate progress and per-step retained output).

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::text::{fit, truncate, width, wrap};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::progress::{ProgressStatus, render_bar, spinner_frame};
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::statusbar::StatusItem;
use junie_tui::widgets::viewport::{Span, TextViewport};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use crate::domain::plan::{Plan, PlanPhase, StepState};
use crate::screens::{
    Cx, Go, Modal, ModalResult, ModalTag, Screen, StatusBits, heading, plural, ticks_label,
    truncate_sep,
};
use crate::sim::world::World;

pub const OUTLINE: WidgetId = WidgetId::of("plan.outline");
pub const DETAIL: WidgetId = WidgetId::of("plan.detail");
pub const CANCEL: WidgetId = WidgetId::of("plan.cancel");
pub const CONFIRM: WidgetId = WidgetId::of("plan.confirm");
pub const OUTPUT: WidgetId = WidgetId::of("plan.output");
const FOLLOW_UP: WidgetId = WidgetId::of("plan.follow");

/// Lane label for a step: `·` for roots, a letter per parallel branch,
/// `join` where branches converge.
pub fn lane(p: &Plan, i: usize) -> String {
    fn go(p: &Plan, i: usize, depth: usize) -> String {
        if depth > p.steps.len() {
            return "·".into();
        }
        let deps = &p.steps[i].deps;
        if deps.len() > 1 {
            return "join".into();
        }
        let Some(&parent) = deps.first() else {
            // independent roots are branches of their own
            let roots: Vec<usize> = (0..p.steps.len())
                .filter(|&j| p.steps[j].deps.is_empty())
                .collect();
            if roots.len() > 1 {
                let k = roots.iter().position(|&r| r == i).unwrap_or(0);
                return ((b'a' + (k as u8 % 26)) as char).to_string();
            }
            return "·".into();
        };
        let siblings = p.dependents(parent);
        if siblings.len() > 1 {
            let k = siblings.iter().position(|&s| s == i).unwrap_or(0);
            return ((b'a' + (k as u8 % 26)) as char).to_string();
        }
        go(p, parent, depth + 1)
    }
    go(p, i, 0)
}

fn state_glyph(s: StepState, tick: u64) -> (&'static str, Tone, bool) {
    match s {
        StepState::Ready | StepState::Waiting => ("·", Tone::Muted, false),
        StepState::Running => (spinner_frame(tick), Tone::Success, false),
        StepState::Succeeded => ("✓", Tone::Secondary, false),
        StepState::Failed => ("!", Tone::Error, true),
        StepState::Skipped | StepState::Excluded | StepState::Cancelled => {
            ("–", Tone::Faint, false)
        }
        StepState::Blocked => ("·", Tone::Faint, false),
    }
}

/// The selected step's facts.
fn step_props(p: &Plan, i: usize, w: &World) -> Vec<Prop> {
    let s = &p.steps[i];
    let mut v = vec![];
    for (k, c) in s.commands.iter().enumerate() {
        v.push(Prop::new(if k == 0 { "Runs" } else { "" }, c.clone()).tone(Tone::Secondary));
    }
    v.push(Prop::new(
        "In",
        format!("{} · {}", w.location.short(&s.cwd), p.host),
    ));
    let needs: Vec<String> = s
        .deps
        .iter()
        .map(|&d| format!("{:02} {}", d + 1, p.steps[d].label))
        .collect();
    v.push(
        Prop::new(
            "Needs",
            if needs.is_empty() {
                "nothing · starts first".into()
            } else {
                needs.join(" · ")
            },
        )
        .wrap(),
    );
    let unlocks: Vec<String> = p
        .dependents(i)
        .iter()
        .map(|&d| {
            format!(
                "{:02} {}{}",
                d + 1,
                p.steps[d].label,
                if !p.steps[d].included {
                    " (excluded)"
                } else {
                    ""
                }
            )
        })
        .collect();
    if !unlocks.is_empty() {
        v.push(Prop::new("Unlocks", unlocks.join(" · ")).wrap());
    }
    let l = lane(p, i);
    let beside: Vec<String> = p
        .parallel_with(i)
        .iter()
        .filter(|&&j| lane(p, j) != l && p.steps[j].deps.first() == s.deps.first())
        .map(|&j| format!("{:02}", j + 1))
        .collect();
    let lane_text = if l == "join" {
        "join · waits for every branch it needs".to_owned()
    } else if l == "·" {
        "· trunk · ordered".to_owned()
    } else if beside.is_empty() {
        format!("{l} · ordered")
    } else {
        format!("{l} · may run beside {}", beside.join(", "))
    };
    v.push(Prop::new("Lane", lane_text));
    if let Some(e) = &s.exclusive {
        v.push(Prop::new("Ordered by", e.clone()).tone(Tone::Muted));
    }
    if !s.privilege.is_empty() {
        v.push(Prop::new("Privilege", s.privilege.clone()).tone(Tone::Warning));
    }
    if !s.effects.is_empty() {
        v.push(Prop::new("Effect", s.effects.join(" · ")).wrap());
    }
    if !s.note.is_empty() {
        v.push(Prop::new("Note", s.note.clone()).tone(Tone::Warning).wrap());
    }
    let state = if s.because.is_empty() || s.because == s.state.label() {
        s.state.label().to_owned()
    } else {
        format!("{} · {}", s.state.label(), s.because)
    };
    v.push(Prop::new("State", state).tone(match s.state {
        StepState::Failed => Tone::Error,
        StepState::Blocked | StepState::Excluded => Tone::Muted,
        _ => Tone::Normal,
    }));
    v
}

/// The final line of a run: only the non-zero classes, in outcome order.
fn outcome_line(ok: usize, failed: usize, excluded: usize, skipped: usize, never: usize) -> String {
    let parts: Vec<String> = [
        (ok, "succeeded"),
        (failed, "failed"),
        (skipped, "skipped"),
        (never, "never started"),
        (excluded, "excluded"),
    ]
    .into_iter()
    .filter(|(n, _)| *n > 0)
    .map(|(n, w)| format!("{n} {w}"))
    .collect();
    if parts.is_empty() {
        "nothing ran".into()
    } else {
        parts.join(" · ")
    }
}

/// The consequence line under an outline: which steps are blocked and why.
fn consequences(p: &Plan) -> Vec<String> {
    let mut v: Vec<String> = p
        .steps
        .iter()
        .enumerate()
        .filter(|(_, s)| s.state == StepState::Blocked && s.included)
        .map(|(i, s)| format!("{:02} {} · {}", i + 1, s.label, s.because))
        .collect();
    if v.is_empty() && p.phase == PlanPhase::Review {
        let required = p.steps.iter().filter(|s| !s.optional).count();
        v.push(format!(
            "{} required · {} optional · {} run in parallel where independent",
            required,
            p.optional_count(),
            plural(p.parallel_branches(), "branch", "branches")
        ));
    }
    v
}

#[allow(clippy::too_many_arguments)]
fn render_outline(
    p: &Plan,
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    id: WidgetId,
    cursor: usize,
    scroll: &mut ScrollState,
    review: bool,
) {
    let t = ctx.theme;
    let bg = t.canvas;
    let focused = ctx.interaction.focused(id);
    scroll.set_content(p.steps.len());
    scroll.set_viewport(area.height as usize);
    scroll.ensure_visible(cursor);
    ctx.control(id, area, false);
    ctx.scrollable(id, area);
    let has_sb = scroll.overflows();
    let row_w = area.width.saturating_sub(u16::from(has_sb));
    let label_w = p.steps.iter().map(|s| width(&s.label)).max().unwrap_or(10) as u16;
    // columns: ▎ mark 01 label · needs · lane · meta
    let needs_w = p
        .steps
        .iter()
        .map(|s| {
            if s.deps.is_empty() {
                1
            } else {
                s.deps.len() * 3 - 1
            }
        })
        .max()
        .unwrap_or(2)
        .clamp(5, 14) as u16;
    let lane_w = 4u16;
    let label_col = label_w.min(
        row_w
            .saturating_sub(3 + 4 + 3 + needs_w + 2 + lane_w + 2 + 10)
            .max(12),
    );
    for (k, i) in scroll.visible_range().enumerate() {
        let y = area.y + k as u16;
        let s = &p.steps[i];
        let rid = id.child(i);
        let mut vs = ctx.state(rid);
        vs.focused = focused && i == cursor;
        vs.selected = i == cursor;
        let st = t.row(vs, bg);
        let row = Rect::new(area.x, y, row_w, 1);
        fill(buf, row, st);
        buf.set_string(row.x, y, "▎", t.gutter(vs, st.bg.unwrap_or(bg), false));
        let plain = st.remove_modifier(Modifier::BOLD);
        let mut x = row.x + 1;
        if review {
            // inclusion marker: checked in the accent, unchecked muted, required faint
            // on the selected row the faint tier would vanish into the
            // tint at 16 colours: step up one rung there
            let quiet = if vs.selected {
                t.text_muted
            } else {
                t.text_faint
            };
            let (mark, ms) = if !s.optional {
                ("[✓]", plain.fg(quiet))
            } else if s.included {
                ("[✓]", plain.fg(t.accent))
            } else {
                ("[ ]", plain.fg(t.text_muted))
            };
            buf.set_string(x, y, mark, ms);
            x += 4;
        } else {
            let (g, tone, bold) = state_glyph(s.state, ctx.interaction.tick);
            let mut gs = plain.fg(t.tone(tone));
            if bold {
                gs = gs.add_modifier(Modifier::BOLD);
            }
            buf.set_string(x + 1, y, g, gs);
            x += 3;
        }
        let num = format!("{:02}", i + 1);
        buf.set_string(
            x,
            y,
            &num,
            plain.fg(if s.state == StepState::Running || vs.selected {
                t.text_secondary
            } else {
                t.text_faint
            }),
        );
        x += 3;
        let label_style = match s.state {
            StepState::Excluded | StepState::Skipped | StepState::Cancelled => st.fg(t.text_faint),
            StepState::Blocked => st.fg(t.text_faint),
            StepState::Failed => st.fg(t.error).add_modifier(Modifier::BOLD),
            _ => st,
        };
        buf.set_string(x, y, fit(&s.label, label_col as usize), label_style);
        x += label_col + 2;
        let needs = if s.deps.is_empty() {
            "–".to_owned()
        } else {
            s.deps
                .iter()
                .map(|d| format!("{:02}", d + 1))
                .collect::<Vec<_>>()
                .join(" ")
        };
        if x + needs_w <= row.right() {
            buf.set_string(x, y, fit(&needs, needs_w as usize), plain.fg(t.text_muted));
        }
        x += needs_w + 2;
        let l = lane(p, i);
        if x + lane_w <= row.right() {
            buf.set_string(x, y, fit(&l, lane_w as usize), plain.fg(t.text_secondary));
        }
        x += lane_w + 2;
        let meta = if review {
            if !s.included {
                "excluded".to_owned()
            } else if s.state == StepState::Blocked {
                s.because.clone()
            } else if s.optional {
                "optional".to_owned()
            } else if !s.privilege.is_empty() {
                s.privilege.clone()
            } else {
                String::new()
            }
        } else {
            match s.state {
                StepState::Running => ticks_label(
                    ctx.interaction
                        .tick
                        .saturating_sub(s.started_tick.unwrap_or(0)),
                ),
                StepState::Succeeded | StepState::Failed => {
                    let d = s
                        .ended_tick
                        .unwrap_or(0)
                        .saturating_sub(s.started_tick.unwrap_or(0));
                    format!(
                        "{}{}",
                        ticks_label(d),
                        s.exit
                            .filter(|e| *e != 0)
                            .map(|e| format!(" · exit {e}"))
                            .unwrap_or_default()
                    )
                }
                StepState::Blocked | StepState::Excluded | StepState::Cancelled => {
                    s.because.clone()
                }
                StepState::Waiting => s
                    .deps
                    .iter()
                    .find(|&&d| !p.steps[d].state.finished())
                    .map(|d| format!("waits {:02}", d + 1))
                    .unwrap_or("waiting".into()),
                StepState::Ready => "ready".into(),
                StepState::Skipped => "skipped".into(),
            }
        };
        // blocked is a quiet consequence; only failure takes the error tone
        let meta_tone = match s.state {
            StepState::Failed => t.error,
            StepState::Running => t.text_secondary,
            _ if vs.selected => t.text_muted,
            _ => t.text_faint,
        };
        let avail = row.right().saturating_sub(x + 1) as usize;
        if avail >= 6 && !meta.is_empty() {
            buf.set_string(x, y, truncate_sep(&meta, avail), plain.fg(meta_tone));
        }
        ctx.clickable(rid, row);
    }
    if has_sb {
        scrollbar::render_vertical(
            Rect::new(area.right() - 1, area.y, 1, area.height),
            buf,
            ctx,
            id,
            scroll,
            focused,
        );
    }
}

// ------------------------------------------------------------- review

pub struct PlanReviewPage {
    pub plan: String,
    pub cursor: usize,
    scroll: ScrollState,
    detail_scroll: ScrollState,
    undo: Vec<usize>,
    cancel: Button,
    confirm: Button,
    drawer: bool,
}

impl PlanReviewPage {
    pub fn new(plan: &str) -> Self {
        Self {
            plan: plan.into(),
            cursor: 0,
            scroll: ScrollState::default(),
            detail_scroll: ScrollState::default(),
            undo: vec![],
            cancel: Button::subtle(CANCEL, "Cancel"),
            confirm: Button::primary(CONFIRM, "Confirm plan"),
            drawer: false,
        }
    }

    fn render_detail(
        &mut self,
        p: &Plan,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        w: &World,
    ) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(DETAIL);
        let s = &p.steps[self.cursor.min(p.steps.len().saturating_sub(1))];
        let title = format!("{:02} {}", self.cursor + 1, s.label);
        let panel = Panel::card(Some(&title))
            .focused(focused)
            .meta(if s.optional { "optional" } else { "required" });
        let bg = panel.bg(t);
        let props = step_props(p, self.cursor, w);
        // wrap into lines first: the card ends after its content
        let label_w = props.iter().map(|p| width(&p.label)).max().unwrap_or(4) as u16 + 2;
        let vw = area.width.saturating_sub(4 + label_w) as usize;
        let mut flat: Vec<(String, String, Tone)> = vec![];
        for pr in props {
            for (i, part) in wrap(&pr.value, vw.max(8)).into_iter().enumerate() {
                flat.push((
                    if i == 0 {
                        pr.label.clone()
                    } else {
                        String::new()
                    },
                    part,
                    pr.tone,
                ));
            }
        }
        let area = Rect::new(
            area.x,
            area.y,
            area.width,
            (flat.len() as u16 + 3).min(area.height),
        );
        let inner = panel.render(area, buf, t);
        ctx.control(DETAIL, area, false);
        ctx.scrollable(DETAIL, inner);
        self.detail_scroll.set_content(flat.len());
        self.detail_scroll.set_viewport(inner.height as usize);
        for (k, i) in self.detail_scroll.visible_range().enumerate() {
            let y = inner.y + k as u16;
            let (l, v, tone) = &flat[i];
            buf.set_string(inner.x, y, l, t.muted().bg(bg));
            buf.set_string(
                inner.x + label_w,
                y,
                truncate(v, vw),
                Style::new().fg(t.tone(*tone)).bg(bg),
            );
        }
        if self.detail_scroll.overflows() {
            scrollbar::render_vertical(
                Rect::new(inner.right() - 1, inner.y, 1, inner.height),
                buf,
                ctx,
                DETAIL,
                &self.detail_scroll,
                focused,
            );
        }
    }
}

impl Screen for PlanReviewPage {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        let Some(p) = w.plan_mut(&self.plan) else {
            return Outcome::Ignored;
        };
        let n = p.steps.len();
        if cx.focus.is(CANCEL) {
            let (o, fired) = self.cancel.on_key(key);
            if fired {
                cx.go(Go::Pop);
                cx.status("Plan discarded · nothing was executed");
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if cx.focus.is(CONFIRM) {
            let (o, fired) = self.confirm.on_key(key);
            if fired {
                cx.go(Go::ConfirmPlan(self.plan.clone()));
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if cx.focus.is(DETAIL) {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.detail_scroll.scroll_by(-1);
                    return Outcome::Changed;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.detail_scroll.scroll_by(1);
                    return Outcome::Changed;
                }
                KeyCode::Esc => {
                    self.drawer = false;
                    cx.focus.focus(OUTLINE);
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if cx.focus.is(OUTLINE) => {
                self.cursor = self.cursor.saturating_sub(1);
                self.detail_scroll.jump_start();
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') if cx.focus.is(OUTLINE) => {
                self.cursor = (self.cursor + 1).min(n.saturating_sub(1));
                self.detail_scroll.jump_start();
                Outcome::Changed
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.cursor = 0;
                Outcome::Changed
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.cursor = n.saturating_sub(1);
                Outcome::Changed
            }
            KeyCode::Char(' ') => {
                let i = self.cursor;
                if p.toggle(i) {
                    self.undo.push(i);
                    let s = &p.steps[i];
                    let blocked: Vec<String> = p
                        .steps
                        .iter()
                        .enumerate()
                        .filter(|(_, x)| x.state == StepState::Blocked && x.included)
                        .map(|(j, _)| format!("{:02}", j + 1))
                        .collect();
                    let msg = if s.included {
                        format!("{:02} included", i + 1)
                    } else if blocked.is_empty() {
                        format!("{:02} excluded · nothing depends on it", i + 1)
                    } else {
                        format!("{:02} excluded · {} now blocked", i + 1, blocked.join(", "))
                    };
                    cx.status(msg);
                } else {
                    cx.status(format!(
                        "{:02} {} is required by the plan and cannot be excluded",
                        i + 1,
                        p.steps[i].label
                    ));
                }
                Outcome::Changed
            }
            KeyCode::Char('u') => {
                if let Some(i) = self.undo.pop() {
                    p.toggle(i);
                    cx.status(format!("{:02} restored", i + 1));
                } else {
                    cx.status("Nothing to undo");
                }
                Outcome::Changed
            }
            KeyCode::Char('c') => {
                cx.go(Go::ConfirmPlan(self.plan.clone()));
                Outcome::Changed
            }
            KeyCode::Char('p') => {
                self.drawer = !self.drawer;
                cx.focus.focus(if self.drawer { DETAIL } else { OUTLINE });
                Outcome::Changed
            }
            KeyCode::Enter if cx.focus.is(OUTLINE) => {
                cx.focus.focus(CONFIRM);
                Outcome::Changed
            }
            KeyCode::Char('x') => {
                cx.go(Go::Pop);
                cx.status("Plan discarded · nothing was executed");
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_click(&mut self, id: WidgetId, _pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        if id == CANCEL {
            cx.go(Go::Pop);
            return Outcome::Changed;
        }
        if id == CONFIRM {
            cx.go(Go::ConfirmPlan(self.plan.clone()));
            return Outcome::Changed;
        }
        if id == DETAIL {
            cx.focus.focus(DETAIL);
            return Outcome::Changed;
        }
        let n = w.plan(&self.plan).map(|p| p.steps.len()).unwrap_or(0);
        if let Some(i) = (0..n).find(|&i| OUTLINE.child(i) == id) {
            self.cursor = i;
            cx.focus.focus(OUTLINE);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_double_click(
        &mut self,
        id: WidgetId,
        _pos: Position,
        w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        let n = w.plan(&self.plan).map(|p| p.steps.len()).unwrap_or(0);
        if let Some(i) = (0..n).find(|&i| OUTLINE.child(i) == id)
            && let Some(p) = w.plan_mut(&self.plan)
        {
            self.cursor = i;
            if p.toggle(i) {
                self.undo.push(i);
            } else {
                cx.status(format!("{:02} is required and cannot be excluded", i + 1));
            }
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if id == OUTLINE {
            self.scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        if id == DETAIL {
            self.detail_scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let Some(p) = w.plan(&self.plan).cloned() else {
            return;
        };
        self.cursor = self.cursor.min(p.steps.len().saturating_sub(1));
        // title row
        buf.set_string(area.x + 1, area.y, &p.title, t.title());
        let sub = format!(" · {}", p.host);
        buf.set_string(
            area.x + 1 + width(&p.title) as u16,
            area.y,
            &sub,
            t.secondary(),
        );
        let summary = format!(
            "{} · {} included · {} excluded · ~{}",
            plural(p.steps.len(), "step", "steps"),
            p.included_count(),
            p.steps.len() - p.included_count(),
            ticks_label(p.estimate_ticks())
        );
        let sw = width(&summary) as u16;
        if sw + width(&p.title) as u16 + width(&sub) as u16 + 4 < area.width {
            buf.set_string(
                area.right().saturating_sub(sw + 1),
                area.y,
                &summary,
                t.muted(),
            );
        }
        buf.set_string(
            area.x + 1,
            area.y + 1,
            truncate(&p.intent, area.width.saturating_sub(2) as usize),
            t.muted(),
        );
        let split = area.width >= crate::screens::finder::SPLIT_MIN;
        let buttons_y = area.bottom().saturating_sub(1);
        let cons = consequences(&p);
        let cons_h = cons.len().min(2) as u16;
        let body = Rect::new(
            area.x,
            area.y + 3,
            area.width,
            buttons_y.saturating_sub(area.y + 3 + cons_h + 2),
        );
        let (outline_area, detail_area) = if split {
            let dw = (area.width * 40 / 100).clamp(36, 54);
            (
                Rect::new(
                    body.x,
                    body.y,
                    body.width.saturating_sub(dw + 2),
                    body.height,
                ),
                Some(Rect::new(
                    body.right().saturating_sub(dw),
                    body.y,
                    dw,
                    body.height,
                )),
            )
        } else {
            (body, None)
        };
        // column headings
        heading(
            buf,
            outline_area.x + 8,
            outline_area.y,
            outline_area.width.saturating_sub(8),
            "step",
            t,
            t.canvas,
        );
        {
            let label_w = p.steps.iter().map(|s| width(&s.label)).max().unwrap_or(10) as u16;
            let needs_w = p
                .steps
                .iter()
                .map(|s| {
                    if s.deps.is_empty() {
                        1
                    } else {
                        s.deps.len() * 3 - 1
                    }
                })
                .max()
                .unwrap_or(2)
                .clamp(5, 14) as u16;
            let label_col = label_w.min(
                outline_area
                    .width
                    .saturating_sub(3 + 4 + 3 + needs_w + 2 + 4 + 2 + 10)
                    .max(12),
            );
            let x = outline_area.x + 8 + label_col + 2;
            if x + needs_w + 6 < outline_area.right() {
                buf.set_string(x, outline_area.y, "needs", t.faint());
                buf.set_string(x + needs_w + 2, outline_area.y, "lane", t.faint());
                let nx = x + needs_w + 2 + 4 + 2;
                if nx + 4 < outline_area.right() {
                    buf.set_string(nx, outline_area.y, "note", t.faint());
                }
            }
        }
        let rows = Rect::new(
            outline_area.x,
            outline_area.y + 1,
            outline_area.width,
            outline_area.height.saturating_sub(1),
        );
        if let Some(d) = detail_area {
            if self.drawer && !split {
                self.render_detail(&p, body, buf, ctx, w);
            } else {
                render_outline(
                    &p,
                    rows,
                    buf,
                    ctx,
                    OUTLINE,
                    self.cursor,
                    &mut self.scroll,
                    true,
                );
                self.render_detail(&p, d, buf, ctx, w);
            }
        } else if self.drawer {
            ctx.control(OUTLINE, Rect::ZERO, false);
            self.render_detail(&p, body, buf, ctx, w);
        } else {
            render_outline(
                &p,
                rows,
                buf,
                ctx,
                OUTLINE,
                self.cursor,
                &mut self.scroll,
                true,
            );
            ctx.control(DETAIL, Rect::ZERO, false);
        }
        // consequence lines: one blank row under the outline, never lower
        // than the space above the buttons
        let cy = (rows.y + p.steps.len() as u16 + 1).min(body.bottom() + 1);
        for (i, c) in cons.iter().take(2).enumerate() {
            let tone = if c.contains("blocked") {
                t.warning
            } else {
                t.text_muted
            };
            buf.set_string(
                area.x + 3,
                cy + i as u16,
                truncate(c, area.width.saturating_sub(4) as usize),
                Style::new().fg(tone),
            );
        }
        // buttons, right-aligned
        let label = if p.phrase.is_some() {
            "Continue…"
        } else {
            "Confirm plan"
        };
        self.confirm = if p.phrase.is_some() {
            Button::danger(CONFIRM, label)
        } else {
            Button::primary(CONFIRM, label)
        };
        let widths = [self.cancel.width(), self.confirm.width()];
        let rects = junie_tui::widgets::button::row_layout_right(
            Rect::new(area.x, buttons_y, area.width.saturating_sub(1), 1),
            &widths,
            2,
        );
        self.cancel.render(rects[0], buf, ctx, t.canvas);
        self.confirm.render(rects[1], buf, ctx, t.canvas);
    }

    fn hints(&self, focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        if focus == Some(DETAIL) {
            return vec![hint("↑↓", "Scroll"), hint("Esc", "Outline")];
        }
        if focus == Some(CANCEL) || focus == Some(CONFIRM) {
            return vec![
                hint("← →", "Choose"),
                hint("Enter", "Activate"),
                hint("Esc", "Discard plan"),
            ];
        }
        vec![
            hint("↑↓", "Step"),
            hint("Space", "Toggle"),
            hint("u", "Undo"),
            hint("c", "Confirm"),
            hint("p", "Facts"),
            hint("Esc", "Discard"),
        ]
    }

    fn crumb(&self, w: &World) -> String {
        w.plan(&self.plan)
            .map(|p| p.title.clone())
            .unwrap_or("Plan".into())
    }

    fn status(&self, w: &World) -> StatusBits {
        let mut bits = StatusBits::default();
        if let Some(p) = w.plan(&self.plan) {
            bits.center = Some(
                StatusItem::new(
                    format!(
                        "review · {} · {} parallel",
                        plural(p.included_count(), "step", "steps"),
                        plural(p.parallel_branches(), "branch", "branches")
                    ),
                    Tone::Secondary,
                )
                .priority(6),
            );
        }
        bits
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(OUTLINE)
    }
}

// ------------------------------------------------------------ executing

pub struct PlanTab {
    pub plan: String,
    pub cursor: usize,
    scroll: ScrollState,
    view: TextViewport,
    rendered: (usize, usize),
    pub maximized: bool,
    follow_ups: Vec<Button>,
    /// The cursor has been moved onto the result once the run ended.
    landed: bool,
}

impl PlanTab {
    pub fn new(plan: &str) -> Self {
        Self {
            plan: plan.into(),
            cursor: 0,
            scroll: ScrollState::default(),
            view: TextViewport::new(OUTPUT).max_lines(2000).wrap(true),
            rendered: (usize::MAX, usize::MAX),
            maximized: false,
            follow_ups: vec![],
            landed: false,
        }
    }

    fn follow_up(&self, i: usize, w: &World, cx: &mut Cx) {
        if let Some((_, item)) = w.plan(&self.plan).and_then(|p| p.follow_ups.get(i)) {
            cx.go(Go::Run {
                item: item.clone(),
                args: vec![],
            });
        }
    }

    fn sync(&mut self, w: &World) {
        let Some(p) = w.plan(&self.plan) else {
            return;
        };
        // when the run ends, land on the result: the failed step, else the
        // last step that ran (CONCEPT §6.8: the frontier is the outcome)
        if p.phase == PlanPhase::Done && !self.landed {
            self.landed = true;
            let target = p
                .steps
                .iter()
                .position(|s| s.state == StepState::Failed)
                .or_else(|| {
                    p.steps
                        .iter()
                        .rposition(|s| s.state == StepState::Succeeded)
                });
            if let Some(i) = target {
                self.cursor = i;
                self.view.follow = true;
            }
        }
        if self.follow_ups.len() != p.follow_ups.len() {
            self.follow_ups = p
                .follow_ups
                .iter()
                .enumerate()
                .map(|(i, (label, _))| Button::subtle(FOLLOW_UP.child(i), label))
                .collect();
        }
        let i = self.cursor.min(p.steps.len().saturating_sub(1));
        let s = &p.steps[i];
        if self.rendered != (i, s.output.len()) {
            let lines = s
                .output
                .iter()
                .map(|l| {
                    let tone = if l.starts_with('$') {
                        Tone::Secondary
                    } else if l.contains("ERROR")
                        || l.starts_with("exit status")
                        || l.contains("mismatch")
                    {
                        Tone::Error
                    } else {
                        Tone::Normal
                    };
                    vec![Span::new(l.clone(), tone)]
                })
                .collect();
            let follow = self.view.follow;
            self.view.set_lines(lines);
            self.view.follow = follow;
            self.rendered = (i, s.output.len());
        }
    }
}

impl Screen for PlanTab {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        // follow-up buttons after the run
        for i in 0..self.follow_ups.len() {
            if cx.focus.is(self.follow_ups[i].id) {
                let (o, fired) = self.follow_ups[i].on_key(key);
                if fired {
                    self.follow_up(i, w, cx);
                    return Outcome::Changed;
                }
                if o != Outcome::Ignored {
                    return o;
                }
            }
        }
        let Some(p) = w.plan_mut(&self.plan) else {
            return Outcome::Ignored;
        };
        let n = p.steps.len();
        if cx.focus.is(OUTPUT) {
            match key.code {
                KeyCode::Esc if self.maximized => {
                    self.maximized = false;
                    return Outcome::Changed;
                }
                KeyCode::Esc => {
                    cx.focus.focus(OUTLINE);
                    return Outcome::Changed;
                }
                _ => {}
            }
            let (o, ev) = self.view.on_key(key);
            if let Some(junie_tui::widgets::viewport::ViewportEvent::Copy(text)) = ev {
                cx.copy(text);
            }
            if o.consumed() {
                return o;
            }
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if cx.focus.is(OUTLINE) => {
                self.cursor = self.cursor.saturating_sub(1);
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') if cx.focus.is(OUTLINE) => {
                self.cursor = (self.cursor + 1).min(n.saturating_sub(1));
                Outcome::Changed
            }
            KeyCode::Enter | KeyCode::Char('z') => {
                self.maximized = !self.maximized;
                cx.focus
                    .focus(if self.maximized { OUTPUT } else { OUTLINE });
                Outcome::Changed
            }
            KeyCode::Char('r') => {
                let i = self.cursor;
                if p.retry(i) {
                    cx.status(format!(
                        "{:02} retrying · succeeded work is not repeated",
                        i + 1
                    ));
                } else {
                    cx.status("Only a failed step can be retried");
                }
                Outcome::Changed
            }
            KeyCode::Char('k') => {
                let i = self.cursor;
                if p.skip(i) {
                    cx.status(format!(
                        "{:02} skipped · its dependents stay blocked",
                        i + 1
                    ));
                } else {
                    cx.status("Only an optional failed step can be skipped");
                }
                Outcome::Changed
            }
            KeyCode::Char('f') => {
                self.view.set_follow(!self.view.follow);
                Outcome::Changed
            }
            KeyCode::Char('c') if key.ctrl() => {
                if p.phase == PlanPhase::Running {
                    let d = Dialog::destructive(
                        WidgetId::of("cancel-plan"),
                        "Cancel the remaining steps?",
                        "Running steps finish; ready and waiting steps are cancelled. Completed work stays.",
                        "Cancel remaining",
                    );
                    cx.open(Modal::Dialog(d), ModalTag::new("cancel-plan"));
                } else {
                    cx.status("The plan is not running");
                }
                Outcome::Changed
            }
            KeyCode::Char('x') => {
                cx.go(Go::CloseTab);
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_modal(
        &mut self,
        tag: &ModalTag,
        result: ModalResult,
        w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        if tag.kind == "cancel-plan"
            && let ModalResult::Dialog {
                action: Some(1), ..
            } = result
        {
            let tick = w.tick;
            if let Some(p) = w.plan_mut(&self.plan) {
                p.cancel_remaining();
                if p.running().is_empty() {
                    p.phase = PlanPhase::Done;
                    p.ended_tick = Some(tick);
                }
            }
            cx.status("Remaining steps cancelled · running steps finish");
        }
        Outcome::Changed
    }

    fn on_click(&mut self, id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        for i in 0..self.follow_ups.len() {
            if self.follow_ups[i].id == id {
                cx.focus.focus(id);
                if self.follow_ups[i].on_click() {
                    self.follow_up(i, w, cx);
                }
                return Outcome::Changed;
            }
        }
        let n = w.plan(&self.plan).map(|p| p.steps.len()).unwrap_or(0);
        if let Some(i) = (0..n).find(|&i| OUTLINE.child(i) == id) {
            self.cursor = i;
            cx.focus.focus(OUTLINE);
            return Outcome::Changed;
        }
        if id == OUTPUT {
            cx.focus.focus(OUTPUT);
            return self.view.on_click(pos).or(Outcome::Changed);
        }
        if id == scrollbar::id_for(OUTPUT) {
            return self.view.on_scrollbar(pos);
        }
        Outcome::Ignored
    }

    fn on_press(&mut self, id: WidgetId, pos: Position, _w: &mut World) -> Outcome {
        if id == OUTPUT {
            return self.view.on_click(pos);
        }
        Outcome::Ignored
    }

    fn on_drag(&mut self, pressed: WidgetId, pos: Position, _w: &mut World) -> Outcome {
        if pressed == OUTPUT {
            return self.view.on_drag(pos);
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if id == OUTLINE {
            self.scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        if self.view.owns(id) {
            return self.view.on_wheel(delta);
        }
        Outcome::Ignored
    }

    fn on_tick(&mut self, w: &mut World, _cx: &mut Cx) -> Outcome {
        let before = self.rendered;
        self.sync(w);
        if before != self.rendered
            || w.plan(&self.plan)
                .is_some_and(|p| p.phase == PlanPhase::Running)
        {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    fn enter(&mut self, w: &mut World, _cx: &mut Cx) {
        self.sync(w);
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        self.sync(w);
        let t = ctx.theme;
        let Some(p) = w.plan(&self.plan).cloned() else {
            return;
        };
        self.cursor = self.cursor.min(p.steps.len().saturating_sub(1));
        let (ok, failed, excluded, skipped, never) = p.outcome();
        let included = p.included_count();
        let running = p.running();
        // title row
        buf.set_string(area.x + 1, area.y, &p.title, t.title());
        let next_wait = p
            .steps
            .iter()
            .enumerate()
            .find(|(_, s)| s.state == StepState::Waiting)
            .and_then(|(i, s)| {
                s.deps
                    .iter()
                    .find(|&&d| !p.steps[d].state.finished())
                    .map(|d| format!("{:02} waits for {:02}", i + 1, d + 1))
            });
        let blocked = p
            .steps
            .iter()
            .filter(|s| s.state == StepState::Blocked && s.included)
            .count();
        let mut mid = format!(" · {ok} of {included} done");
        if !running.is_empty() {
            mid.push_str(&format!(" · {} running", running.len()));
        }
        if blocked > 0 {
            mid.push_str(&format!(" · {blocked} blocked"));
        }
        if failed > 0 {
            mid.push_str(&format!(" · {failed} failed"));
        }
        if let Some(nw) = next_wait
            && p.phase == PlanPhase::Running
        {
            mid.push_str(&format!(" · {nw}"));
        }
        // only the failed clause takes the error tone
        let mx = area.x + 1 + width(&p.title) as u16;
        match mid
            .find(" · ")
            .filter(|_| failed > 0)
            .and_then(|_| mid.find(&format!("{failed} failed")))
        {
            Some(at) => {
                buf.set_string(mx, area.y, &mid[..at], t.secondary());
                let fx = mx + width(&mid[..at]) as u16;
                let clause = format!("{failed} failed");
                buf.set_string(fx, area.y, &clause, t.error_fg());
                buf.set_string(
                    fx + width(&clause) as u16,
                    area.y,
                    &mid[at + clause.len()..],
                    t.secondary(),
                );
            }
            None => buf.set_string(mx, area.y, &mid, t.secondary()),
        }
        let elapsed = ticks_label(
            p.ended_tick
                .unwrap_or(w.tick)
                .saturating_sub(p.started_tick.unwrap_or(w.tick)),
        );
        let right = match p.phase {
            PlanPhase::Running => format!("{} {elapsed}", spinner_frame(ctx.interaction.tick)),
            PlanPhase::Done => format!("finished · {elapsed} · {}", p.host),
            PlanPhase::Review => "not started".into(),
        };
        let rw = width(&right) as u16;
        buf.set_string(
            area.right().saturating_sub(rw + 1),
            area.y,
            &right,
            if p.phase == PlanPhase::Running {
                t.accent_fg()
            } else {
                t.muted()
            },
        );
        // aggregate progress
        let ratio = if included == 0 {
            0.0
        } else {
            ok as f64 / included as f64
        };
        let status = if p.phase == PlanPhase::Done {
            if failed > 0 {
                ProgressStatus::Error
            } else {
                ProgressStatus::Done
            }
        } else {
            ProgressStatus::Active
        };
        render_bar(
            Rect::new(
                area.x + 1,
                area.y + 1,
                area.width.saturating_sub(2).min(60),
                1,
            ),
            buf,
            ctx,
            "",
            ratio,
            status,
            t.canvas,
        );
        let has_follow = p.phase == PlanPhase::Done && !self.follow_ups.is_empty();
        let summary_h: u16 = if p.phase == PlanPhase::Done {
            2 + u16::from(has_follow)
        } else {
            0
        };
        let body = Rect::new(
            area.x,
            area.y + 3,
            area.width,
            area.height.saturating_sub(3 + summary_h),
        );
        let split = area.width >= crate::screens::finder::SPLIT_MIN && !self.maximized;
        let rail_area = if split {
            Rect::new(body.x, body.y, (body.width * 56 / 100).max(40), body.height)
        } else {
            body
        };
        let out_area = if split {
            Rect::new(
                rail_area.right() + 2,
                body.y,
                body.width.saturating_sub(rail_area.width + 2),
                body.height,
            )
        } else {
            body
        };
        let s = &p.steps[self.cursor];
        let out_title = format!("{:02} {}", self.cursor + 1, s.label);
        let out_meta = if s.state == StepState::Running {
            ticks_label(w.tick.saturating_sub(s.started_tick.unwrap_or(w.tick)))
        } else {
            s.state.label().to_owned()
        };
        if self.maximized {
            ctx.control(OUTLINE, Rect::ZERO, false);
            let panel = Panel::framed(Some(&out_title))
                .focused(ctx.interaction.focused(OUTPUT))
                .meta(&out_meta);
            let inner = panel.render(out_area, buf, t);
            self.view.render(inner, buf, ctx, t.canvas);
        } else if split {
            heading(
                buf,
                rail_area.x + 7,
                rail_area.y,
                rail_area.width.saturating_sub(7),
                "step",
                t,
                t.canvas,
            );
            let rows = Rect::new(
                rail_area.x,
                rail_area.y + 1,
                rail_area.width,
                rail_area.height.saturating_sub(1),
            );
            render_outline(
                &p,
                rows,
                buf,
                ctx,
                OUTLINE,
                self.cursor,
                &mut self.scroll,
                false,
            );
            let panel = Panel::framed(Some(&out_title))
                .focused(ctx.interaction.focused(OUTPUT))
                .meta(&out_meta);
            let inner = panel.render(out_area, buf, t);
            self.view.render(inner, buf, ctx, t.canvas);
        } else {
            render_outline(
                &p,
                rail_area,
                buf,
                ctx,
                OUTLINE,
                self.cursor,
                &mut self.scroll,
                false,
            );
            ctx.control(OUTPUT, Rect::ZERO, false);
        }
        if summary_h > 0 {
            let y = area.bottom().saturating_sub(summary_h - 1);
            let line = outcome_line(ok, failed, excluded, skipped, never);
            buf.set_string(
                area.x + 1,
                y,
                truncate(&line, area.width.saturating_sub(2) as usize),
                t.secondary(),
            );
            if has_follow {
                let y = area.bottom().saturating_sub(1);
                buf.set_string(area.x + 1, y, "Next", t.faint());
                let mut x = area.x + 7;
                for b in &mut self.follow_ups {
                    let bw = b.width();
                    if x + bw > area.right() {
                        break;
                    }
                    b.render(Rect::new(x, y, bw, 1), buf, ctx, t.canvas);
                    x += bw + 2;
                }
            } else {
                for b in &mut self.follow_ups {
                    ctx.control(b.id, Rect::ZERO, false);
                }
            }
        } else {
            for b in &mut self.follow_ups {
                ctx.control(b.id, Rect::ZERO, false);
            }
        }
        let _ = props::render;
    }

    fn hints(&self, focus: Option<WidgetId>, w: &World) -> Vec<Hint> {
        let p = w.plan(&self.plan);
        let failed_here = p.is_some_and(|p| {
            p.steps
                .get(self.cursor)
                .is_some_and(|s| s.state == StepState::Failed)
        });
        if focus == Some(OUTPUT) {
            let mut v = vec![hint("↑↓", "Scroll"), hint("f", "Follow"), hint("y", "Copy")];
            v.push(if self.maximized {
                hint("Esc", "Un-maximise")
            } else {
                hint("Esc", "Steps")
            });
            return v;
        }
        let mut v = vec![hint("↑↓", "Step"), hint("Enter", "Maximise output")];
        if failed_here {
            v.push(hint("r", "Retry"));
            v.push(hint("k", "Skip"));
        }
        if p.is_some_and(|p| p.phase == PlanPhase::Running) {
            v.push(hint("Ctrl+C", "Cancel remaining"));
        }
        v.push(hint("x", "Close"));
        v.push(hint("Esc", "Here"));
        v
    }

    fn crumb(&self, w: &World) -> String {
        w.plan(&self.plan)
            .map(|p| p.title.clone())
            .unwrap_or_default()
    }

    fn status(&self, w: &World) -> StatusBits {
        let mut bits = StatusBits::default();
        if let Some(p) = w.plan(&self.plan) {
            let running: Vec<String> = p
                .running()
                .iter()
                .map(|&i| format!("lane {} {:02}", lane(p, i), i + 1))
                .collect();
            let (ok, failed, ..) = p.outcome();
            bits.center = Some(match p.phase {
                PlanPhase::Done if failed > 0 => StatusItem::new(
                    format!("finished · {ok} ok · {failed} failed"),
                    Tone::Secondary,
                )
                .priority(6),
                PlanPhase::Done => {
                    StatusItem::new(format!("finished · {ok} ok"), Tone::Secondary).priority(6)
                }
                _ if running.is_empty() => StatusItem::new("waiting", Tone::Secondary).priority(6),
                _ => StatusItem::new(running.join(" · "), Tone::Secondary)
                    .busy()
                    .priority(6),
            });
        }
        bits
    }

    fn animating(&self, w: &World) -> bool {
        w.plan(&self.plan)
            .is_some_and(|p| p.phase == PlanPhase::Running)
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(OUTLINE)
    }

    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        if self.maximized {
            self.maximized = false;
            return Outcome::Changed;
        }
        cx.go(Go::Here);
        Outcome::Changed
    }
}
