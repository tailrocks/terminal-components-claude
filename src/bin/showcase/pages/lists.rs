use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::row_menu::{MENU_HINTS, OPEN_HINT, RowMenu, plain};
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::context::ContextRequest;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::menu::MenuItem;
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use ratatui::crossterm::event::KeyCode;

const ID: WidgetId = WidgetId::of("lists");
const MENU: WidgetId = WidgetId::of("lists.menu");

pub struct ListsPage {
    single: ListBox,
    multi: ListBox,
    empty: ListBox,
    /// The row menu: which list (by id) and which row it is about.
    menu: RowMenu<(WidgetId, usize)>,
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
            menu: RowMenu::default(),
        }
    }

    fn lists(&mut self) -> [&mut ListBox; 3] {
        [&mut self.single, &mut self.multi, &mut self.empty]
    }

    fn list_mut(&mut self, id: WidgetId) -> Option<&mut ListBox> {
        self.lists().into_iter().find(|l| l.id == id)
    }

    /// What a row of `l` offers: only what its keys already do.
    fn row_items(l: &ListBox, row: usize) -> Vec<MenuItem> {
        let item = &l.items[row];
        match l.mode {
            SelectMode::Single => {
                let choose = MenuItem::new("Choose").shortcut("Enter");
                vec![if item.disabled {
                    choose.reason("Unavailable")
                } else if l.chosen == Some(row) {
                    choose.reason("Already chosen")
                } else {
                    choose
                }]
            }
            SelectMode::Multi => {
                let toggle = MenuItem::new(if l.checked[row] { "Exclude" } else { "Include" })
                    .shortcut("Space");
                let toggle = if item.disabled {
                    toggle.reason(format!("{} file", item.meta.as_deref().unwrap_or("Locked")))
                } else {
                    toggle
                };
                let all = l
                    .items
                    .iter()
                    .zip(&l.checked)
                    .filter(|(it, _)| !it.disabled)
                    .all(|(_, c)| *c);
                vec![
                    toggle.separator(),
                    MenuItem::new(if all { "Select none" } else { "Select all" }).shortcut("a"),
                ]
            }
        }
    }

    /// Open the row menu `req` asks for on list `id`.
    fn open_menu(&mut self, id: WidgetId, req: ContextRequest<usize>) {
        let Some(l) = self.list_mut(id) else {
            return;
        };
        let title = l.items[req.target].label.clone();
        let items = Self::row_items(l, req.target);
        self.menu
            .open(req.menu(MENU, items).title(title), (id, req.target));
    }

    /// Item `i` of the menu about `row` of list `id`: the same path the
    /// item's key takes.
    fn choose(&mut self, (id, row): (WidgetId, usize), i: usize) -> Outcome {
        let Some(l) = self.list_mut(id) else {
            return Outcome::Consumed;
        };
        l.cursor = row;
        match i {
            0 => l.activate(row),
            _ => l.on_key(&plain(KeyCode::Char('a'))),
        }
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
        let panel = Panel::card(Some("Language")).focused(ctx.interaction.focused(self.single.id));
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
        panel.draw_meta(
            cols[0],
            buf,
            t,
            &scrollbar::position_label(&self.single.scroll),
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
        self.menu.render(area, buf, ctx);
    }

    fn layer_open(&self) -> bool {
        self.menu.hides_page_focus()
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        if let Some((out, chosen)) = self.menu.handle(ev) {
            return match chosen {
                Some((target, i)) => self.choose(target, i).or(out),
                None => out,
            };
        }
        match ev {
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                let cfg = self.menu.config;
                let Some(l) = self.list_mut(f) else {
                    return Outcome::Ignored;
                };
                if let Some(req) = cfg.on_key(l, key) {
                    self.open_menu(f, req);
                    return Outcome::Changed;
                }
                l.on_key(key)
            }
            PageEvent::Secondary { id, pos } => {
                let cfg = self.menu.config;
                let hit = self
                    .lists()
                    .into_iter()
                    .find_map(|l| cfg.on_secondary(l, *id, *pos).map(|req| (l.id, req)));
                let Some((list, req)) = hit else {
                    return Outcome::Ignored;
                };
                cx.focus.focus(list);
                self.open_menu(list, req);
                Outcome::Changed
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
                        return l.on_scrollbar_drag(*pos);
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
        if self.menu.is_open() {
            MENU_HINTS.to_vec()
        } else if focus == Some(self.multi.id) {
            vec![
                ("↑↓", "Move"),
                ("Space", "Toggle"),
                ("a", "All / none"),
                ("Shift+↓", "Range"),
                OPEN_HINT,
            ]
        } else {
            vec![
                ("↑↓", "Move"),
                ("Enter", "Choose"),
                ("g G", "Ends"),
                OPEN_HINT,
            ]
        }
    }
}
