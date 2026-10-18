use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Cx, Hint, Page, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::empty::{self, EmptyState};

pub struct StubPage {
    title: &'static str,

    step: &'static str,

    what: &'static str,
}

impl StubPage {
    pub fn new(title: &'static str, step: &'static str, what: &'static str) -> Self {
        Self { title, step, what }
    }
}

impl Page for StubPage {
    fn title(&self) -> &'static str {
        self.title
    }

    fn blurb(&self) -> &'static str {
        self.what
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let e = EmptyState::new(&format!("{} arrives with {}", self.title, self.step))
            .hint("A planned page: its step fills it (plan §7)");
        empty::render_ctx(area, buf, ctx, &e, t.canvas);
    }

    fn handle(&mut self, _ev: &PageEvent, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        Vec::new()
    }
}
