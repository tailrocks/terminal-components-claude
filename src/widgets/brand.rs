//! Brand lockup: the one filled pill an application uses for its identity.
//! It is the only control that fills with the accent, so it reads as the
//! product mark rather than as a button; every screen that shows the
//! identity draws this same lockup rather than composing its own glyphs.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::core::id::WidgetId;
use crate::theme::Theme;
use crate::ui::ctx::RenderCtx;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lockup {
    /// The mark text, supplied by the application (never baked in here).
    pub text: String,
    /// Compact drops the outer padding for tight strips; the treatment
    /// (accent fill, on-accent bold text) never changes.
    pub compact: bool,
}

impl Lockup {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            compact: false,
        }
    }

    pub fn compact(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            compact: true,
        }
    }

    fn label(&self) -> String {
        if self.compact {
            self.text.clone()
        } else {
            format!(" {} ", self.text)
        }
    }

    pub fn width(&self) -> u16 {
        crate::ui::text::width(&self.label()) as u16
    }

    pub fn style(t: &Theme) -> Style {
        Style::new()
            .fg(t.text_on_accent)
            .bg(t.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Draw at a position; returns the width used.
    pub fn render(&self, x: u16, y: u16, buf: &mut Buffer, t: &Theme) -> u16 {
        let label = self.label();
        buf.set_string(x, y, &label, Self::style(t));
        crate::ui::text::width(&label) as u16
    }

    /// Draw as a clickable region (hover lifts to `accent_hover`).
    pub fn render_clickable(
        &self,
        x: u16,
        y: u16,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        id: WidgetId,
    ) -> u16 {
        let t = ctx.theme;
        let label = self.label();
        let mut st = Self::style(t);
        if ctx.interaction.hovered(id) {
            st = st.bg(t.accent_hover);
        }
        if ctx.interaction.pressed(id) {
            st = st.bg(t.accent_pressed);
        }
        buf.set_string(x, y, &label, st);
        let w = crate::ui::text::width(&label) as u16;
        ctx.clickable(id, Rect::new(x, y, w, 1));
        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::ui::ctx::Interaction;

    #[test]
    fn lockup_is_padded_accent_and_bold() {
        let t = Theme::junie();
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
        let l = Lockup::new("mark❯");
        assert_eq!(l.width(), 7);
        assert_eq!(l.render(0, 0, &mut buf, &t), 7);
        assert_eq!(buf[(0, 0)].symbol(), " ");
        assert_eq!(buf[(1, 0)].symbol(), "m");
        assert_eq!(buf[(1, 0)].bg, t.accent);
        assert_eq!(buf[(1, 0)].fg, t.text_on_accent);
        assert!(buf[(1, 0)].modifier.contains(Modifier::BOLD));
        let c = Lockup::compact("mark❯");
        assert_eq!(c.width(), 5);
    }

    #[test]
    fn clickable_lockup_registers_and_lifts_on_hover() {
        let t = Theme::junie();
        let id = WidgetId::of("brand");
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &t,
            Interaction {
                hover: Some(id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
        Lockup::new("x").render_clickable(2, 0, &mut buf, &mut ctx, id);
        assert_eq!(buf[(3, 0)].bg, t.accent_hover);
        assert_eq!(hits.hit(ratatui::layout::Position::new(3, 0)), Some(id));
    }
}
