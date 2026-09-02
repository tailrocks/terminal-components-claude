//! Global Settings.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::keyhint::{Hint, hint};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{Cx, Go, Screen};
use crate::sim::world::World;

pub struct SettingsScreen;

impl SettingsScreen {
    pub fn new(_w: &World) -> Self {
        Self
    }
}

impl Screen for SettingsScreen {
    fn on_key(&mut self, key: &Key, _w: &mut World, cx: &mut Cx) -> Outcome {
        if key.is(ratatui::crossterm::event::KeyCode::Esc) {
            cx.go(Go::Manager);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _w: &World) {
        let t = ctx.theme;
        empty::render(area, buf, t, &EmptyState::new("Settings").hint("Coming in slice 3"), t.canvas);
        ctx.control(WidgetId::of("settings"), Rect::new(area.x, area.y, 1, 1), false);
    }

    fn hints(&self, _focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        vec![hint("Esc", "Back")]
    }

    fn crumb(&self, _w: &World) -> String {
        "Settings › global".into()
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(WidgetId::of("settings"))
    }
}
