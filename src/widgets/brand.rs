use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::core::id::WidgetId;
use crate::theme::{ButtonKind, Kind, Plane, Theme};
use crate::ui::ctx::{RenderCtx, VisualState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lockup {
    pub text: String,

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
        Self::state_style(t, VisualState::default())
    }

    fn state_style(t: &Theme, s: VisualState) -> Style {
        t.interactive(Kind::Button(ButtonKind::Primary), s, Plane::Canvas)
            .style
    }

    pub fn render(&self, x: u16, y: u16, buf: &mut Buffer, t: &Theme) -> u16 {
        let label = self.label();
        buf.set_string(x, y, &label, Self::style(t));
        crate::ui::text::width(&label) as u16
    }

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
        let mut s = ctx.state(id);
        s.focused = false;
        let st = Self::state_style(t, s);
        buf.set_string(x, y, &label, st);
        let w = crate::ui::text::width(&label) as u16;
        ctx.clickable(id, Rect::new(x, y, w, 1));
        w
    }
}
