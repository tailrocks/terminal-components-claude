use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::tree::TreeView;

const ID: WidgetId = WidgetId::of("trees");

pub struct TreesPage {
    tree: TreeView,
}

impl TreesPage {
    pub fn new() -> Self {
        Self {
            tree: TreeView::new(ID.sub("tree"), crate::data::project_tree()),
        }
    }
}

impl Page for TreesPage {
    fn title(&self) -> &'static str {
        "Trees"
    }
    fn blurb(&self) -> &'static str {
        "Indent carries hierarchy; the focus bar never moves"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let (l, r) = crate::pages::layout::columns(area, (area.width * 3 / 5).max(30), 2);
        let pos = scrollbar::position_label(&self.tree.scroll);
        let panel = Panel::card(Some("Project"))
            .meta(&pos)
            .focused(ctx.interaction.focused(self.tree.id));
        let bg = panel.bg(t);
        let inner = panel.render(Rect::new(l.x, l.y, l.width, l.height.min(18)), buf, t);
        self.tree.render(inner, buf, ctx, bg);

        let panel = Panel::card(Some("Selection"));
        let bg = panel.bg(t);
        let inner = panel.render(Rect::new(r.x, r.y, r.width, r.height.min(10)), buf, t);
        let sel = self.tree.selected.clone();
        let mut y = inner.y;
        match sel {
            Some(path) => {
                let mut nodes = &self.tree.nodes;
                let mut parts = Vec::new();
                for &i in &path {
                    parts.push(nodes[i].label.clone());
                    nodes = &nodes[i].children;
                }
                buf.set_string(
                    inner.x,
                    y,
                    junie_tui::ui::text::truncate(&parts.join("/"), inner.width as usize),
                    t.primary().bg(bg),
                );
                y += 1;
                buf.set_string(
                    inner.x,
                    y,
                    format!("depth {}", path.len() - 1),
                    t.muted().bg(bg),
                );
            }
            None => {
                buf.set_string(inner.x, y, "Nothing selected", t.muted().bg(bg));
                y += 1;
                buf.set_string(inner.x, y, "Enter on a file selects it", t.faint().bg(bg));
            }
        }
        y += 2;
        let rows = self.tree.rows();
        let cur = &rows[self.tree.cursor.min(rows.len().saturating_sub(1))];
        buf.set_string(inner.x, y, "cursor", t.faint().bg(bg));
        buf.set_string(
            inner.x + 8,
            y,
            junie_tui::ui::text::truncate(&cur.label, inner.width.saturating_sub(8) as usize),
            t.secondary().bg(bg),
        );
        y += 1;
        buf.set_string(inner.x, y, "visible", t.faint().bg(bg));
        buf.set_string(
            inner.x + 8,
            y,
            format!("{} rows", rows.len()),
            t.secondary().bg(bg),
        );
        y += 1;
        buf.set_string(inner.x, y, "open", t.faint().bg(bg));
        buf.set_string(
            inner.x + 8,
            y,
            format!("{} folders", self.tree.expanded.len()),
            t.secondary().bg(bg),
        );
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Key(key) => {
                if cx.focus.is(self.tree.id) {
                    self.tree.on_key(key).0
                } else {
                    Outcome::Ignored
                }
            }
            PageEvent::Click { id, pos } => {
                if let Some((row, toggle)) = self.tree.locate(*id) {
                    cx.focus.focus(self.tree.id);
                    return if toggle {
                        self.tree.on_click_toggle(row).0
                    } else {
                        self.tree.on_click_row(row).0
                    };
                }
                if *id == scrollbar::id_for(self.tree.id) {
                    return self.tree.on_scrollbar(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } if *pressed == scrollbar::id_for(self.tree.id) => {
                self.tree.on_scrollbar(*pos)
            }
            PageEvent::Wheel { id, delta } if self.tree.owns(*id) => self.tree.on_wheel(*delta),
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        vec![
            ("↑ ↓", "Move"),
            ("← →", "Fold / unfold"),
            ("Enter", "Open"),
            ("*", "Expand all"),
        ]
    }
}
