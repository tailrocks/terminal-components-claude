use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;

const ID: WidgetId = WidgetId::of("lists");

pub struct ListsPage {
    single: ListBox,
    multi: ListBox,
    empty: ListBox,
}

impl ListsPage {
    pub fn new() -> Self {
        let langs: Vec<ListItem> = crate::data::languages()
            .iter()
            .map(|l| ListItem::new(l))
            .collect();
        let mut single = ListBox::new(ID.sub("single"), langs, SelectMode::Single);
        single.chosen = Some(0);
        let files = [
            ("src/api/auth.rs", "modified", false),
            ("src/api/billing.rs", "modified", false),
            ("src/db/schema.rs", "generated", true),
            ("tests/checkout.rs", "new", false),
            ("Cargo.lock", "locked", true),
            ("docs/webhooks.md", "modified", false),
            ("src/workers/mailer.rs", "modified", false),
            ("src/config.rs", "modified", false),
            ("README.md", "modified", false),
            ("src/main.rs", "modified", false),
            ("tests/auth_flow.rs", "new", false),
            ("src/db/pool.rs", "modified", false),
        ];
        let items = files
            .iter()
            .map(|(n, m, d)| ListItem::new(n).meta(m).disabled(*d))
            .collect();
        let mut multi = ListBox::new(ID.sub("multi"), items, SelectMode::Multi);
        multi.checked[0] = true;
        multi.checked[1] = true;
        let empty = ListBox::new(ID.sub("empty"), vec![], SelectMode::Single)
            .empty_text("No results for “retry”");
        Self {
            single,
            multi,
            empty,
        }
    }

    fn lists(&mut self) -> [&mut ListBox; 3] {
        [&mut self.single, &mut self.multi, &mut self.empty]
    }
}

impl Page for ListsPage {
    fn title(&self) -> &'static str {
        "Lists"
    }
    fn blurb(&self) -> &'static str {
        "Single and multiple selection, disabled items, scrolling, empty state"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let third = area.width / 3;
        let cols = [
            Rect::new(area.x, area.y, third.saturating_sub(1), area.height.min(18)),
            Rect::new(
                area.x + third + 1,
                area.y,
                third.saturating_sub(1),
                area.height.min(18),
            ),
            Rect::new(
                area.x + 2 * third + 2,
                area.y,
                area.width.saturating_sub(2 * third + 2),
                area.height.min(18),
            ),
        ];
        let chosen = self
            .single
            .chosen
            .map(|i| self.single.items[i].label.clone())
            .unwrap_or_default();
        let pos = scrollbar::position_label(&self.single.scroll);
        let panel = Panel::card(Some("Language"))
            .meta(&pos)
            .focused(ctx.interaction.focused(self.single.id));
        let bg = panel.bg(t);
        let inner = panel.render(cols[0], buf, t);
        buf.set_string(
            inner.x,
            inner.y,
            format!("Chosen: {chosen}"),
            t.muted().bg(bg),
        );
        self.single.render(
            Rect::new(
                inner.x,
                inner.y + 2,
                inner.width,
                inner.height.saturating_sub(2),
            ),
            buf,
            ctx,
            bg,
        );

        let count = format!("{} selected", self.multi.checked_count());
        let panel = Panel::card(Some("Files to include"))
            .meta(&count)
            .focused(ctx.interaction.focused(self.multi.id));
        let bg = panel.bg(t);
        let inner = panel.render(cols[1], buf, t);
        buf.set_string(
            inner.x,
            inner.y,
            junie_tui::ui::text::truncate(
                "Space toggle · a all · Shift+↓ range",
                inner.width as usize,
            ),
            t.muted().bg(bg),
        );
        self.multi.render(
            Rect::new(
                inner.x,
                inner.y + 2,
                inner.width,
                inner.height.saturating_sub(2),
            ),
            buf,
            ctx,
            bg,
        );

        let panel = Panel::card(Some("Search results"));
        let bg = panel.bg(t);
        let inner = panel.render(cols[2], buf, t);
        self.empty.render(inner, buf, ctx, bg);
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                for l in self.lists() {
                    if l.id == f {
                        return l.on_key(key);
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, pos } => {
                for l in self.lists() {
                    if let Some(row) = l.locate(*id) {
                        cx.focus.focus(l.id);
                        return l.on_click(row);
                    }
                    if scrollbar::id_for(l.id) == *id {
                        return l.on_scrollbar(*pos);
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } => {
                for l in self.lists() {
                    if scrollbar::id_for(l.id) == *pressed {
                        return l.on_scrollbar(*pos);
                    }
                }
                Outcome::Ignored
            }
            PageEvent::Wheel { id, delta } => {
                for l in self.lists() {
                    if l.owns(*id) {
                        return l.on_wheel(*delta);
                    }
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if focus == Some(self.multi.id) {
            vec![
                ("↑ ↓", "Move"),
                ("Space", "Toggle"),
                ("a", "All / none"),
                ("Shift+↓", "Range"),
            ]
        } else {
            vec![("↑ ↓", "Move"), ("Enter", "Choose"), ("g G", "Ends")]
        }
    }
}
