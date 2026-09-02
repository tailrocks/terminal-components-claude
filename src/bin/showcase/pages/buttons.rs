use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::ButtonKind;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::panel::Panel;

const ID: WidgetId = WidgetId::of("buttons");

pub struct ButtonsPage {
    buttons: Vec<Button>,
    clicks: u32,
    last: Option<String>,
    busy_until: Option<std::time::Instant>,
}

impl ButtonsPage {
    pub fn new() -> Self {
        let buttons = vec![
            Button::primary(ID.child(0), "Run task"),
            Button::secondary(ID.child(1), "Preview"),
            Button::subtle(ID.child(2), "Cancel"),
            Button::danger(ID.child(3), "Delete branch"),
            Button::toggle(ID.child(4), "Auto-approve", false),
            Button::toggle(ID.child(5), "Verbose", true),
            Button::primary(ID.child(6), "Disabled primary").disabled(true),
            Button::secondary(ID.child(7), "Disabled").disabled(true),
            Button::secondary(ID.child(8), "Start long job"),
        ];
        Self {
            buttons,
            clicks: 0,
            last: None,
            busy_until: None,
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
            self.busy_until =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(2200));
            cx.status("Working…".to_owned());
        } else {
            cx.status(msg.clone());
        }
        self.last = Some(msg);
    }
}

impl Page for ButtonsPage {
    fn title(&self) -> &'static str {
        "Buttons"
    }
    fn blurb(&self) -> &'static str {
        "Primary, secondary, subtle, danger, toggle, disabled, busy"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[15, 1, 11]);

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

        // Static state matrix (rendered, not interactive) so every state is
        // visible at once regardless of the pointer.
        let panel = Panel::card(Some("State matrix")).meta("reference rendering");
        let bg = panel.bg(t);
        let inner = panel.render(rows[2], buf, t);
        let states: [(&str, junie_tui::ui::ctx::VisualState); 6] = [
            ("default", Default::default()),
            (
                "hover",
                junie_tui::ui::ctx::VisualState {
                    hovered: true,
                    ..Default::default()
                },
            ),
            (
                "focus",
                junie_tui::ui::ctx::VisualState {
                    focused: true,
                    ..Default::default()
                },
            ),
            (
                "focus + hover",
                junie_tui::ui::ctx::VisualState {
                    focused: true,
                    hovered: true,
                    ..Default::default()
                },
            ),
            (
                "pressed",
                junie_tui::ui::ctx::VisualState {
                    pressed: true,
                    focused: true,
                    ..Default::default()
                },
            ),
            (
                "disabled",
                junie_tui::ui::ctx::VisualState {
                    disabled: true,
                    ..Default::default()
                },
            ),
        ];
        let kinds = [
            (ButtonKind::Primary, "Primary"),
            (ButtonKind::Secondary, "Secondary"),
            (ButtonKind::Subtle, "Subtle"),
            (ButtonKind::Danger, "Danger"),
        ];
        let col_w = 15u16;
        let label_w = 15u16;
        // header
        for (k, (_, name)) in kinds.iter().enumerate() {
            let x = inner.x + label_w + k as u16 * col_w;
            if x + col_w > inner.right() + 1 {
                break;
            }
            buf.set_string(x, inner.y, name, t.muted().bg(bg));
        }
        for (si, (sname, s)) in states.iter().enumerate() {
            let y = inner.y + 1 + si as u16;
            if y >= inner.bottom() {
                break;
            }
            buf.set_string(inner.x, y, sname, t.secondary().bg(bg));
            for (k, (kind, _)) in kinds.iter().enumerate() {
                let x = inner.x + label_w + k as u16 * col_w;
                if x + col_w > inner.right() + 1 {
                    break;
                }
                let style = t.button(*kind, *s, bg);
                let on_accent = *kind == ButtonKind::Primary && !s.disabled;
                let gutter = t.gutter(*s, style.bg.unwrap_or(bg), on_accent);
                buf.set_string(x, y, "▎", gutter);
                buf.set_string(x + 1, y, " Label ", style);
            }
        }
        if let Some(last) = &self.last {
            let text = format!("last: {last} · {} activations", self.clicks);
            let y = rows[2].bottom() + 1;
            if y < area.bottom() {
                buf.set_string(area.x, y, &text, t.faint());
            }
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Tick => {
                if let Some(until) = self.busy_until
                    && std::time::Instant::now() >= until
                {
                    self.busy_until = None;
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

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        vec![("Enter / Space", "Activate")]
    }

    fn animating(&self) -> bool {
        self.busy_until.is_some()
    }
}
