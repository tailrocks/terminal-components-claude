//! Account & Usage Center.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::keyhint::{Hint, hint};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{Cx, Go, Screen};
use crate::domain::account::AccountId;
use crate::sim::world::World;

#[derive(Default)]
pub struct AccountsScreen {
    pub selected: Option<AccountId>,
}

impl AccountsScreen {
    pub fn select(&mut self, id: Option<AccountId>) {
        self.selected = id;
    }
}

impl Screen for AccountsScreen {
    fn on_key(&mut self, key: &Key, _w: &mut World, cx: &mut Cx) -> Outcome {
        if key.is(ratatui::crossterm::event::KeyCode::Esc) {
            cx.go(Go::Manager);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _w: &World) {
        let t = ctx.theme;
        empty::render(area, buf, t, &EmptyState::new("Accounts").hint("Coming in slice 2"), t.canvas);
        ctx.control(WidgetId::of("accounts"), Rect::new(area.x, area.y, 1, 1), false);
    }

    fn hints(&self, _focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        vec![hint("Esc", "Back")]
    }

    fn crumb(&self, _w: &World) -> String {
        "Accounts".into()
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(WidgetId::of("accounts"))
    }
}
