use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::row_menu::{MENU_HINTS, OPEN_HINT, RowMenu, plain};
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::context::ContextRequest;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::menu::MenuItem;
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::tree::TreeView;
use ratatui::crossterm::event::KeyCode;

const ID: WidgetId = WidgetId::of("trees");
const MENU: WidgetId = WidgetId::of("trees.menu");

pub struct TreesPage {
    tree: TreeView,
    /// The row menu and the visible row it is about.
    menu: RowMenu<usize>,
}

/// What the row menu's items do.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RowAction {
    Open,
    Fold,
    ExpandAll,
    CollapseAll,
}

impl TreesPage {
    pub fn new() -> Self {
        Self {
            tree: TreeView::new(ID.sub("tree"), crate::data::project_tree()),
            menu: RowMenu::default(),
        }
    }

    /// What visible row `i` offers: only what the tree's keys already do.
    fn row_items(&self, i: usize) -> Vec<(RowAction, MenuItem)> {
        let row = &self.tree.rows()[i];
        let first = if row.has_children {
            let (label, key) = if row.expanded {
                ("Collapse", "←")
            } else {
                ("Expand", "→")
            };
            (RowAction::Fold, MenuItem::new(label).shortcut(key))
        } else {
            let open = MenuItem::new("Open").shortcut("Enter");
            let open = if row.note {
                open.reason("Not a file")
            } else if self.tree.selected.as_ref() == Some(&row.path) {
                open.reason("Already open")
            } else {
                open
            };
            (RowAction::Open, open)
        };
        let collapse = MenuItem::new("Collapse all").shortcut("-");
        vec![
            (first.0, first.1.separator()),
            (
                RowAction::ExpandAll,
                MenuItem::new("Expand all").shortcut("*"),
            ),
            (
                RowAction::CollapseAll,
                if self.tree.expanded.is_empty() {
                    collapse.reason("All folded")
                } else {
                    collapse
                },
            ),
        ]
    }

    fn open_menu(&mut self, req: ContextRequest<usize>) {
        let row = req.target;
        let title = self.tree.rows()[row].label.clone();
        let items = self.row_items(row).into_iter().map(|(_, m)| m).collect();
        self.menu.open(req.menu(MENU, items).title(title), row);
    }

    /// Item `i` of the menu about visible row `row`: the same path the
    /// item's key takes.
    fn choose(&mut self, row: usize, i: usize) -> Outcome {
        let Some(&(action, _)) = self.row_items(row).get(i) else {
            return Outcome::Consumed;
        };
        self.tree.cursor = row;
        match action {
            RowAction::Open => self.tree.on_key(&plain(KeyCode::Enter)).0,
            RowAction::Fold => self.tree.toggle(row).0,
            RowAction::ExpandAll => self.tree.on_key(&plain(KeyCode::Char('*'))).0,
            RowAction::CollapseAll => self.tree.on_key(&plain(KeyCode::Char('-'))).0,
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
        let panel = Panel::card(Some("Project")).focused(ctx.interaction.focused(self.tree.id));
        let bg = panel.bg(t);
        let card = Rect::new(l.x, l.y, l.width, l.height.min(18));
        let inner = panel.render(card, buf, t);
        self.tree.render(inner, buf, ctx, bg);
        panel.draw_meta(card, buf, t, &scrollbar::position_label(&self.tree.scroll));

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
        self.menu.render(area, buf, ctx);
    }

    fn layer_open(&self) -> bool {
        self.menu.hides_page_focus()
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        if let Some((out, chosen)) = self.menu.handle(ev) {
            return match chosen {
                Some((row, i)) => self.choose(row, i).or(out),
                None => out,
            };
        }
        match ev {
            PageEvent::Key(key) => {
                if !cx.focus.is(self.tree.id) {
                    return Outcome::Ignored;
                }
                if let Some(req) = self.menu.config.on_key(&self.tree, key) {
                    self.open_menu(req);
                    return Outcome::Changed;
                }
                self.tree.on_key(key).0
            }
            PageEvent::Secondary { id, pos } => {
                let cfg = self.menu.config;
                let Some(req) = cfg.on_secondary(&mut self.tree, *id, *pos) else {
                    return Outcome::Ignored;
                };
                cx.focus.focus(self.tree.id);
                self.open_menu(req);
                Outcome::Changed
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
                self.tree.on_scrollbar_drag(*pos)
            }
            PageEvent::Wheel { id, delta } if self.tree.owns(*id) => self.tree.on_wheel(*delta),
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        if self.menu.is_open() {
            return MENU_HINTS.to_vec();
        }
        vec![
            ("↑↓", "Move"),
            ("←→", "Fold / unfold"),
            ("Enter", "Open"),
            ("*", "Expand all"),
            OPEN_HINT,
        ]
    }
}
