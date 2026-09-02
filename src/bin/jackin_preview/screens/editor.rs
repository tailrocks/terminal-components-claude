//! Workspace Editor.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::keyhint::{Hint, hint};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{Cx, Go, Screen};
use crate::domain::workspace::{Workspace, WorkspaceId};
use crate::sim::world::World;

pub struct EditorScreen {
    pub workspace: Option<WorkspaceId>,
    pub pending: Workspace,
}

impl EditorScreen {
    pub fn new(w: &World, workspace: Option<WorkspaceId>, pending: Option<Workspace>) -> Self {
        let pending = pending
            .or_else(|| workspace.and_then(|id| w.workspace(id)).cloned())
            .unwrap_or_else(|| Workspace::new(0, "new", "/workspace"));
        Self { workspace, pending }
    }
}

impl Screen for EditorScreen {
    fn on_key(&mut self, key: &Key, _w: &mut World, cx: &mut Cx) -> Outcome {
        if key.is(ratatui::crossterm::event::KeyCode::Esc) {
            cx.go(Go::Manager);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _w: &World) {
        let t = ctx.theme;
        empty::render(area, buf, t, &EmptyState::new(&format!("Editor · {}", self.pending.name)).hint("Coming in slice 3"), t.canvas);
        ctx.control(WidgetId::of("editor"), Rect::new(area.x, area.y, 1, 1), false);
    }

    fn hints(&self, _focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        vec![hint("Esc", "Back")]
    }

    fn crumb(&self, _w: &World) -> String {
        format!("Workspaces › {} › edit", self.pending.name)
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(WidgetId::of("editor"))
    }
}
