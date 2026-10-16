use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::template::{Section, Template};
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::panel::Panel;

const ID: WidgetId = WidgetId::of("buttons");
const LONG_JOB_TICKS: u8 = 28;

pub struct ButtonsPage {
    template: Template,
    buttons: Vec<Button>,
    clicks: u32,
    last: Option<String>,
    busy_ticks: Option<u8>,
}

impl ButtonsPage {
    pub fn new() -> Self {
        let buttons = vec![
            // rest the pointer on a button for its tooltip (Esc hides it)
            Button::primary(ID.child(0), "Run task").tooltip("Start the task in the background"),
            Button::secondary(ID.child(1), "Preview").tooltip("Show what the task would change"),
            Button::subtle(ID.child(2), "Cancel").tooltip("Leave without running anything"),
            Button::danger(ID.child(3), "Delete branch")
                .tooltip("Delete the branch, locally and on the remote"),
            Button::toggle(ID.child(4), "Auto-approve", false),
            Button::toggle(ID.child(5), "Verbose", true),
            Button::primary(ID.child(6), "Disabled primary").disabled(true),
            Button::secondary(ID.child(7), "Disabled").disabled(true),
            Button::secondary(ID.child(8), "Start long job"),
        ];
        Self {
            template: Template::new(ID, states_grid()),
            buttons,
            clicks: 0,
            last: None,
            busy_ticks: None,
        }
    }

    fn activated(&mut self, i: usize, cx: &mut PageCtx) {
        self.clicks += 1;
        let b = &self.buttons[i];
        let msg = match b.on {
            Some(true) => format!("{} on", b.label),
            Some(false) => format!("{} off", b.label),
            None => format!("{} ✓", b.label),
        };
        if i == 8 {
            self.buttons[8].busy = true;
            self.busy_ticks = Some(LONG_JOB_TICKS);
            cx.status("Working…".to_owned());
        } else {
            cx.status(msg.clone());
        }
        self.last = Some(msg);
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[15, 0]);

        // Interactive playground on a card
        let panel = Panel::card(Some("Playground")).meta("hover · click · Tab · Enter / Space");
        let bg = panel.bg(t);
        let inner = panel.render(rows[0], buf, t);
        let mut y = inner.y;
        let groups: [(&str, &[usize]); 4] = [
            ("Actions", &[0, 1, 2, 3]),
            ("Toggles", &[4, 5]),
            ("Disabled", &[6, 7]),
            ("Busy", &[8]),
        ];
        for (label, idx) in groups {
            if y + 1 >= inner.bottom() {
                break;
            }
            crate::pages::layout::caption(inner.x, y, buf, t, label, bg);
            let widths: Vec<u16> = idx.iter().map(|&i| self.buttons[i].width()).collect();
            let rects = row_layout(Rect::new(inner.x, y + 1, inner.width, 1), &widths, 2);
            for (&i, r) in idx.iter().zip(rects) {
                self.buttons[i].render(r, buf, ctx, bg);
            }
            y += 3;
        }
        if let Some(last) = &self.last {
            let text = format!("last: {last} · {} activations", self.clicks);
            let y = rows[0].bottom() + 1;
            if y < area.bottom() {
                buf.set_string(area.x, y, &text, t.faint());
            }
        }
    }
}

/// The States grid: every button kind in every state it has, drawn by
/// `Button` itself ([`junie_tui::ui::specimen`]).
fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    StateGrid::new(id)
        .rows(&[
            Rest,
            Hover,
            Focused,
            FocusedHover,
            Pressed,
            Disabled,
            Selected,
            Busy,
        ])
        .row(Overflow)
        .column("Primary", Button::primary(id.child(0), "Label"))
        .column("Secondary", Button::secondary(id.child(1), "Label"))
        .column("Subtle", Button::subtle(id.child(2), "Label"))
        .column("Danger", Button::danger(id.child(3), "Label"))
        .column("Toggle", Button::toggle(id.child(4), "Label", false))
}

impl Page for ButtonsPage {
    fn title(&self) -> &'static str {
        "Buttons"
    }
    fn blurb(&self) -> &'static str {
        "Primary, secondary, subtle, danger, toggle, disabled, busy"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let body = self.template.render(area, buf, ctx);
        match self.template.section() {
            Section::Playground => self.render_playground(body, buf, ctx),
            Section::Options => self.template.render_options(body, buf, ctx),
            Section::States => self.template.render_states(body, buf, ctx),
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        match ev {
            PageEvent::Tick => {
                if let Some(ticks) = self.busy_ticks {
                    if ticks > 1 {
                        self.busy_ticks = Some(ticks - 1);
                        return Outcome::Changed;
                    }
                    self.busy_ticks = None;
                    self.buttons[8].busy = false;
                    cx.status("Long job finished ✓");
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                let Some(i) = self.buttons.iter().position(|b| b.id == f) else {
                    return Outcome::Ignored;
                };
                let (out, activated) = self.buttons[i].on_key(key);
                if activated {
                    self.activated(i, cx);
                }
                out
            }
            PageEvent::Click { id, .. } => {
                let Some(i) = self.buttons.iter().position(|b| b.id == *id) else {
                    return Outcome::Ignored;
                };
                if self.buttons[i].on_click() {
                    self.activated(i, cx);
                }
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        self.template
            .hints(focus)
            .unwrap_or_else(|| vec![("Enter / Space", "Activate")])
    }

    fn animating(&self) -> bool {
        self.busy_ticks.is_some()
    }

    fn template(&self) -> Option<&Template> {
        Some(&self.template)
    }

    fn template_mut(&mut self) -> Option<&mut Template> {
        Some(&mut self.template)
    }
}
