use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::theme::{ButtonKind, Kind};
use crate::ui::ctx::RenderCtx;
use crate::ui::text::width;
use ratatui::crossterm::event::KeyCode;

/// A button is rendered as ` label ` with one cell of padding. When focused
/// the left padding cell becomes the accent gutter bar. No box, ever.
#[derive(Debug, Clone)]
pub struct Button {
    pub id: WidgetId,
    pub label: String,
    pub kind: ButtonKind,
    pub disabled: bool,
    /// For toggle buttons: current on/off state.
    pub on: Option<bool>,
    pub busy: bool,
    pub area: Rect,
}

impl Button {
    pub fn new(id: WidgetId, label: &str, kind: ButtonKind) -> Self {
        Self {
            id,
            label: label.to_owned(),
            kind,
            disabled: false,
            on: None,
            busy: false,
            area: Rect::ZERO,
        }
    }

    pub fn primary(id: WidgetId, label: &str) -> Self {
        Self::new(id, label, ButtonKind::Primary)
    }
    pub fn secondary(id: WidgetId, label: &str) -> Self {
        Self::new(id, label, ButtonKind::Secondary)
    }
    pub fn subtle(id: WidgetId, label: &str) -> Self {
        Self::new(id, label, ButtonKind::Subtle)
    }
    pub fn danger(id: WidgetId, label: &str) -> Self {
        Self::new(id, label, ButtonKind::Danger)
    }
    pub fn toggle(id: WidgetId, label: &str, on: bool) -> Self {
        let mut b = Self::new(id, label, ButtonKind::Toggle);
        b.on = Some(on);
        b
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Display width including padding and toggle marker.
    pub fn width(&self) -> u16 {
        let marker = if self.on.is_some() || self.busy { 2 } else { 0 };
        (width(&self.label) + 2 + marker) as u16
    }

    /// The footer hint for this button while it has focus: Enter names
    /// what it does (`Enter Save…`, `Enter Cancel`), never a generic verb.
    pub fn hint(&self) -> crate::widgets::keyhint::Hint {
        crate::widgets::keyhint::Hint::new("Enter", self.label.clone())
    }

    pub fn can_activate(&self) -> bool {
        !self.disabled && !self.busy
    }

    /// Keyboard activation while focused. Returns `true` when activated.
    pub fn on_key(&mut self, key: &Key) -> (Outcome, bool) {
        if key.is(KeyCode::Enter) || key.is_char(' ') {
            if self.can_activate() {
                self.toggle_if_needed();
                (Outcome::Changed, true)
            } else {
                (Outcome::Consumed, false)
            }
        } else {
            (Outcome::Ignored, false)
        }
    }

    /// Mouse click activation. Returns `true` when activated.
    pub fn on_click(&mut self) -> bool {
        if self.can_activate() {
            self.toggle_if_needed();
            true
        } else {
            false
        }
    }

    fn toggle_if_needed(&mut self) {
        if let Some(on) = self.on.as_mut() {
            *on = !*on;
        }
    }

    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: ratatui::style::Color,
    ) {
        let area = area.intersection(*buf.area());
        let w = self.width().min(area.width);
        let area = Rect::new(area.x, area.y, w, 1.min(area.height));
        self.area = area;
        if area.is_empty() {
            return;
        }
        let mut s = ctx.state(self.id);
        s.disabled = self.disabled;
        s.busy = self.busy;
        s.selected = self.on.unwrap_or(false);
        if self.disabled {
            s.hovered = false;
            s.pressed = false;
        }
        if self.busy {
            s.pressed = false;
        }
        let t = ctx.theme;
        // the resolver owns every state, busy included: a busy label dims,
        // on the accent fill it keeps the on-accent ink
        let it = t.interactive(Kind::Button(self.kind), s, t.plane_of(bg));
        let style = it.style;
        let filled = it.on_fill;
        let busy_ink = style.fg.unwrap_or(t.text_secondary);
        let gutter = it.gutter_style;
        let mut text = String::new();
        if self.busy {
            text.push_str(super::progress::spinner_frame(ctx.interaction.tick));
            text.push(' ');
        } else if let Some(on) = self.on {
            text.push(if on { '●' } else { '○' });
            text.push(' ');
        }
        text.push_str(&self.label);
        let text = crate::ui::text::fit(&text, (w as usize).saturating_sub(2));
        buf.set_string(area.x, area.y, it.gutter, gutter);
        buf.set_string(area.x + 1, area.y, &text, style);
        // marker colour: accent when on, muted when off
        if let Some(on) = self.on
            && !self.disabled
        {
            let ms = style.fg(if on { t.accent } else { t.text_muted });
            let ms = if s.pressed { style } else { ms };
            buf.set_string(area.x + 1, area.y, if on { "●" } else { "○" }, ms);
        }
        if self.busy {
            buf.set_string(
                area.x + 1,
                area.y,
                super::progress::spinner_frame(ctx.interaction.tick),
                style.fg(if filled { busy_ink } else { t.accent }),
            );
        }
        buf.set_string(area.x + w - 1, area.y, " ", style);
        ctx.control(self.id, area, self.disabled);
    }
}

/// Lay out buttons in a row with a 1-cell gap, returning their areas.
pub fn row_layout(area: Rect, widths: &[u16], gap: u16) -> Vec<Rect> {
    let mut x = area.x;
    let mut out = Vec::new();
    for &w in widths {
        let w = w.min(area.right().saturating_sub(x));
        out.push(Rect::new(x, area.y, w, area.height.min(1)));
        x = x.saturating_add(w).saturating_add(gap);
    }
    out
}

/// Right-aligned row layout (used by dialog action bars).
pub fn row_layout_right(area: Rect, widths: &[u16], gap: u16) -> Vec<Rect> {
    let total: u16 = widths.iter().sum::<u16>() + gap * widths.len().saturating_sub(1) as u16;
    let x = area.right().saturating_sub(total).max(area.x);
    row_layout(
        Rect::new(x, area.y, area.right().saturating_sub(x), area.height),
        widths,
        gap,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    #[test]
    fn a_busy_primary_button_keeps_its_label_and_spinner_visible() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 1));
        let mut b = Button::primary(WidgetId::of("b"), "Connect");
        b.busy = true;
        let w = b.width();
        b.render(Rect::new(0, 0, w, 1), &mut buf, &mut ctx, t.canvas);
        for x in 1..w - 1 {
            let c = &buf[(x, 0)];
            if c.symbol() != " " {
                assert_ne!(c.fg, c.bg, "cell {x} {:?} is invisible", c.symbol());
            }
        }
    }

    #[test]
    fn the_focused_hint_names_the_button() {
        let b = Button::subtle(WidgetId::of("b"), "Cancel");
        let h = b.hint();
        assert_eq!((h.key, h.action.as_ref()), ("Enter", "Cancel"));
    }
}
