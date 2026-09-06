//! Plan: gate 1 of the two-gate treatment — a full-body review surface for
//! a compound intent. The DAG shows branches, dependencies and per-step
//! state; Space excludes optional steps and dependents recalculate live.
//! Enter opens gate 2 (the typed target-bound phrase); nothing runs before.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::text::{truncate, width};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::props::{self, Prop};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::domain::plan::{Plan, StepState};
use crate::screens::{Cx, Modal, ModalResult, ModalTag, Screen};
use crate::sim::plans;
use crate::sim::world::World;

const STEPS: WidgetId = WidgetId::of("plan.step");
const GATE: WidgetId = WidgetId::of("plan.gate");

pub struct PlanScreen {
    pub plan: Plan,
    /// First visible step row; keeps the focused row on screen.
    scroll: usize,
}

impl PlanScreen {
    pub fn new(plan: Plan) -> Self {
        Self { plan, scroll: 0 }
    }

    fn focused_step(&self, cx: &Cx) -> Option<usize> {
        let f = cx.focus.current()?;
        (0..self.plan.steps.len()).find(|&i| f == STEPS.child(i))
    }

    /// Gate 2: the typed target-bound phrase. The confirming action stays
    /// disabled until the input matches exactly (dialog enforces).
    fn gate_dialog(&self) -> Dialog {
        let p = &self.plan;
        let facts = vec![
            Prop::new(
                "Will happen",
                format!(
                    "{} of {} steps run on {}",
                    p.included_count(),
                    p.steps.len(),
                    p.host
                ),
            ),
            Prop::new("Will change", &p.will_change),
            Prop::new(
                "Bound to",
                format!("host {} · phrase names the target", p.host),
            ),
            Prop::new("Safety", "simulated · nothing on any real host executes"),
        ];
        let code: Vec<String> = p
            .steps
            .iter()
            .filter(|s| !matches!(s.state, StepState::Excluded | StepState::PolicySkipped(_)))
            .map(|s| s.command.clone())
            .collect();
        let mut d = Dialog::facts(
            GATE,
            &format!("Confirm: {}", p.title),
            facts,
            code,
            Some(&p.phrase),
            Button::danger(GATE.sub("run"), "Run plan (simulated)"),
        );
        // the phrase is the whole point of gate 2: typing lands immediately
        if let junie_tui::widgets::dialog::DialogBody::Facts { ack: Some(a), .. } = &mut d.body {
            a.input.begin_edit();
        }
        d
    }

    /// The step list occupies what the facts and the output pane leave.
    fn layout(&self, area: Rect, focused: Option<usize>) -> (u16, u16, u16) {
        let facts_h = 1 + 1 + self.plan.facts().len() as u16 + 1 + 1; // title+gap+facts+gap+header
        let lines = focused.map(|i| self.plan.steps[i].lines.len()).unwrap_or(0);
        let out_h = if lines == 0 {
            0
        } else {
            (lines as u16 + 2).min(8)
        };
        let steps_top = area.y + facts_h;
        let steps_h = area
            .height
            .saturating_sub(facts_h + out_h + u16::from(out_h > 0));
        (steps_top, steps_h, out_h)
    }

    fn keep_visible(&mut self, focused: Option<usize>, steps_h: u16) {
        let Some(i) = focused else { return };
        let vis = steps_h as usize;
        if vis == 0 {
            return;
        }
        if i < self.scroll {
            self.scroll = i;
        } else if i >= self.scroll + vis {
            self.scroll = i + 1 - vis;
        }
    }
}

impl Screen for PlanScreen {
    fn on_key(&mut self, key: &Key, _w: &mut World, cx: &mut Cx) -> Outcome {
        match key.code {
            KeyCode::Down => {
                cx.focus.next(cx.ring);
                Outcome::Changed
            }
            KeyCode::Up => {
                cx.focus.prev(cx.ring);
                Outcome::Changed
            }
            KeyCode::Char(' ') if key.plain() => {
                let Some(i) = self.focused_step(cx) else {
                    return Outcome::Ignored;
                };
                match self.plan.toggle(i) {
                    Ok(msg) => cx.status(msg),
                    Err(why) => cx.status(why),
                }
                Outcome::Changed
            }
            KeyCode::Enter => {
                if self.plan.ran {
                    cx.status("Plan already ran · Esc returns home");
                    return Outcome::Changed;
                }
                if self.plan.included_count() == 0 {
                    cx.error("Nothing included · every step is excluded");
                    return Outcome::Changed;
                }
                cx.open(
                    Modal::Dialog(self.gate_dialog()),
                    ModalTag::new("plan.gate").key(&self.plan.action_id),
                );
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
        if let (
            "plan.gate",
            ModalResult::Dialog {
                action: Some(1), ..
            },
        ) = (tag.kind, &result)
        {
            self.plan.run();
            plans::apply_effect(&self.plan, w);
            cx.status(format!("Finished: {} · simulated", self.plan.summary()));
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        fill(buf, area, t.base());
        if area.height < 8 {
            return;
        }
        // title + review posture
        let title = truncate(&self.plan.title, area.width as usize);
        buf.set_string(
            area.x,
            area.y,
            &title,
            t.primary().add_modifier(Modifier::BOLD),
        );
        let posture = if self.plan.ran {
            format!("ran · {}", self.plan.summary())
        } else {
            "review · Space excludes · dependents recalculate · nothing runs yet".to_owned()
        };
        let pw = width(&posture) as u16;
        let tw = width(&title) as u16;
        // the posture line never overprints the title; narrow widths drop it
        if tw + 2 + pw < area.width {
            buf.set_string(area.right().saturating_sub(pw), area.y, &posture, t.faint());
        }
        let facts = self.plan.facts();
        let fy = area.y + 2;
        props::render(
            Rect::new(area.x, fy, area.width, facts.len() as u16),
            buf,
            t,
            &facts,
            t.canvas,
        );
        let focused = ctx
            .interaction
            .focus
            .and_then(|f| (0..self.plan.steps.len()).find(|&i| f == STEPS.child(i)));
        let (steps_top, steps_h, out_h) = self.layout(area, focused);
        self.keep_visible(focused, steps_h);
        buf.set_string(
            area.x,
            steps_top - 1,
            if self.plan.ran {
                "Steps — outcome per step"
            } else {
                "Steps — branches run in parallel where no dependency holds"
            },
            t.faint(),
        );
        // step rows
        let n = self.plan.steps.len();
        let vis = steps_h as usize;
        for (row, i) in (self.scroll..n).take(vis).enumerate() {
            let y = steps_top + row as u16;
            let s = &self.plan.steps[i];
            let id = STEPS.child(i);
            let r = Rect::new(area.x, y, area.width, 1);
            ctx.control(id, r, false);
            let st = t.row(ctx.state(id), t.canvas);
            fill(buf, r, st);
            buf.set_string(r.x, y, "▎", t.gutter(ctx.state(id), t.canvas, false));
            let blocked = self.plan.blocked_by_exclusion(i);
            let (glyph, tone) = if blocked.is_some() {
                ("▲", Tone::Warning)
            } else {
                match &s.state {
                    StepState::Pending => ("·", Tone::Muted),
                    StepState::Excluded | StepState::PolicySkipped(_) | StepState::Skipped(_) => {
                        ("−", Tone::Muted)
                    }
                    StepState::Succeeded => ("✓", Tone::Success),
                    StepState::Failed(_) => ("✗", Tone::Error),
                }
            };
            buf.set_string(
                r.x + 2,
                y,
                glyph,
                Style::new().fg(t.tone(tone)).bg(st.bg.unwrap_or(t.canvas)),
            );
            let mut x = r.x + 4;
            // required steps say so during review; optional is the default
            let title = if !s.optional && !self.plan.ran {
                format!("{} · required", s.title)
            } else {
                s.title.clone()
            };
            buf.set_string(x, y, truncate(&title, 46), st);
            x += (width(&title) as u16).min(46);
            let branch = format!("· {}", s.branch);
            if x + width(&branch) as u16 + 26 < r.right() {
                buf.set_string(
                    x + 1,
                    y,
                    &branch,
                    Style::new()
                        .fg(t.tone(Tone::Faint))
                        .bg(st.bg.unwrap_or(t.canvas)),
                );
            }
            // right note: state truth
            let (note, ntone) = if let Some(dep) = blocked {
                (format!("needs {dep}"), Tone::Warning)
            } else {
                match &s.state {
                    StepState::Pending => (s.command.clone(), Tone::Muted),
                    StepState::Excluded => ("excluded · Space restores".to_owned(), Tone::Muted),
                    StepState::PolicySkipped(why) | StepState::Skipped(why) => {
                        (why.clone(), Tone::Muted)
                    }
                    StepState::Succeeded => ("done".to_owned(), Tone::Success),
                    StepState::Failed(why) => (why.clone(), Tone::Error),
                }
            };
            let nw = width(&note) as u16;
            if r.width > nw + 30 {
                buf.set_string(
                    r.right().saturating_sub(nw + 1),
                    y,
                    &note,
                    Style::new().fg(t.tone(ntone)).bg(st.bg.unwrap_or(t.canvas)),
                );
            }
        }
        if self.scroll + vis < n && steps_h > 0 {
            let y = steps_top + steps_h - 1;
            buf.set_string(
                area.x + 4,
                y,
                format!("… {} more steps", n - self.scroll - vis + 1),
                t.faint(),
            );
        }
        // output pane: the focused step's lines, after the run
        if out_h > 0
            && let Some(i) = focused
        {
            let s = &self.plan.steps[i];
            let oy = area.bottom() - out_h;
            buf.set_string(
                area.x,
                oy,
                truncate(
                    &format!("Output — {} · ${}", s.title, s.command),
                    area.width as usize,
                ),
                t.faint(),
            );
            for (j, line) in s.lines.iter().take(out_h as usize - 2).enumerate() {
                buf.set_string(
                    area.x,
                    oy + 1 + j as u16,
                    truncate(line, area.width as usize),
                    t.secondary(),
                );
            }
        }
        let _ = w;
    }

    fn hints(&self, _focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        if self.plan.ran {
            vec![hint("↑ ↓", "Steps"), hint("Esc", "Back home")]
        } else {
            vec![
                hint("↑ ↓", "Steps"),
                hint("Space", "Exclude / include"),
                hint("Enter", "Continue to confirmation"),
                hint("Esc", "Back"),
            ]
        }
    }

    fn crumb(&self, _w: &World) -> String {
        format!("Plan · {}", self.plan.title)
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(STEPS.child(0))
    }
}
