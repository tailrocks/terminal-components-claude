//! Menus (Navigation, plan §7.5 SC16): the library `ContextMenu` opened
//! from rows (`ui::context`) and from a control, with every `MenuItem`
//! kind and every menu builder.
//!
//! - **Playground**: a file list whose rows open a context menu (secondary
//!   click, press-drag-release, Shift+F10, the Menu key, an optional extra
//!   key), with shortcuts, disabled items with their reason, separators
//!   and a danger item decided per file (`danger_if`, `separator_if`); an
//!   "Actions" button whose menu opens under it (`anchor_control`), long
//!   enough to scroll under `max_rows`; a readout of the last request and
//!   action.
//! - **Options**: one control per builder (`menus.<key>`): placement, fit,
//!   edge margin, min width, max rows, title, release selects, owner focus,
//!   nav, wrap, type-ahead, dismiss, unknown keys, and the
//!   `ContextConfig` keys.
//! - **States**: two menus in every state a menu has, drawn by the menu.
//!
//! The page is wiring and demo data only: placement, keys, pointer and
//! dismissal are the menu's own; `RowMenu` routes page events to it.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;

use crate::app::PageId;
use crate::pages::row_menu::{MENU_HINTS, RowMenu};
use crate::pages::template::{Section, Template};
use crate::pages::{Cx, Hint, Page, PageEvent};
use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::{Part, WidgetId};
use junie_tui::core::intent::Phase;
use junie_tui::core::keymap::NavKeys;
use junie_tui::core::pointer::MouseTarget;
use junie_tui::ui::context::{ContextAnchor, ContextConfig, ContextKeys, ContextRequest};
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::popup::{Dismiss, Place, Placement};
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::menu::{ContextMenu, MenuAction, MenuItem, UnknownKey};
use junie_tui::widgets::panel::Panel;

const ID: WidgetId = WidgetId::of("menus");
const ROW_MENU: WidgetId = ID.sub("row-menu");
const ACTIONS_MENU: WidgetId = ID.sub("actions-menu");

/// The demo files: name, kind, read-only.
const FILES: &[(&str, &str, bool)] = &[
    ("report.pdf", "document", false),
    ("notes.md", "text", false),
    ("build.log", "log", false),
    ("archive.zip", "archive", false),
    ("photo.png", "image", false),
    ("README.md", "read-only", true),
    ("LICENSE", "read-only", true),
];

/// The Actions menu: long enough to scroll under `menus.max_rows`, with
/// every item kind.
fn action_items() -> Vec<MenuItem> {
    vec![
        MenuItem::new("New file").shortcut("⌘N"),
        MenuItem::new("New folder").shortcut("⇧⌘N").separator(),
        MenuItem::new("Import…"),
        MenuItem::new("Export…").reason("Nothing selected"),
        MenuItem::new("Refresh").shortcut("F5").separator(),
        MenuItem::new("Sort by name"),
        MenuItem::new("Sort by kind"),
        MenuItem::new("Sort by size").separator(),
        MenuItem::new("Show hidden files"),
        MenuItem::new("Empty Trash").danger(),
    ]
}

/// What a file row offers: only what this page can do with it.
fn row_items(read_only: bool) -> Vec<MenuItem> {
    let rename = MenuItem::new("Rename").shortcut("F2");
    let trash = MenuItem::new("Move to Trash")
        .shortcut("⌘⌫")
        .danger_if(!read_only);
    vec![
        MenuItem::new("Open").shortcut("Enter"),
        if read_only {
            rename.reason("Read-only file")
        } else {
            rename
        },
        MenuItem::new("Duplicate").separator(),
        MenuItem::new("Copy path"),
        MenuItem::new("Share…")
            .reason("Sharing is off")
            .separator_if(!read_only),
        if read_only {
            trash.disabled(true)
        } else {
            trash
        },
    ]
}

/// The page's option values (indices into `options::MENUS` values).
struct Opts {
    values: Vec<(&'static str, usize)>,
}

impl Opts {
    fn new() -> Self {
        Self {
            values: crate::options::page_options(PageId::Menus)
                .iter()
                .map(|o| (o.key, o.default))
                .collect(),
        }
    }
    fn get(&self, key: &str) -> usize {
        self.values
            .iter()
            .find(|(k, _)| *k == key)
            .map_or(0, |(_, v)| *v)
    }
    fn on(&self, key: &str) -> bool {
        self.get(key) == 1
    }

    /// The menu builders an app calls, from the options.
    fn style(&self, m: ContextMenu) -> ContextMenu {
        let fit = match self.get("menus.fit") {
            1 => Place::FLUSH,
            2 => Place::DEFAULT,
            _ => Place::MENU,
        };
        let mut m = m
            .fit(fit)
            .edge_margin(self.get("menus.edge_margin") as u16)
            .min_width([8, 16, 24][self.get("menus.min_width").min(2)])
            .release_selects(self.on("menus.release_selects"))
            .hide_owner_focus(self.on("menus.hide_owner_focus"))
            .wrap(self.on("menus.wrap"))
            .type_ahead(self.on("menus.type_ahead"))
            .nav(match self.get("menus.nav") {
                1 => NavKeys::VIM,
                _ => NavKeys::DESKTOP,
            })
            .dismiss(match self.get("menus.dismiss") {
                1 => Dismiss::ESCAPE,
                _ => Dismiss::LIGHT,
            })
            .unknown_key(match self.get("menus.unknown_key") {
                1 => UnknownKey::Swallow,
                _ => UnknownKey::CloseAndPass,
            });
        match self.get("menus.max_rows") {
            1 => m = m.max_rows(8),
            2 => m = m.max_rows(4),
            _ => {}
        }
        m
    }

    /// Which keys open a row's menu and whether a click moves the cursor.
    fn context(&self) -> ContextConfig {
        let extra = (self.get("menus.extra_key") == 1).then_some(Key {
            code: KeyCode::Char('m'),
            mods: KeyModifiers::NONE,
        });
        ContextConfig::DEFAULT
            .keys(
                ContextKeys::DEFAULT
                    .shift_f10(self.on("menus.shift_f10"))
                    .menu_key(self.on("menus.menu_key"))
                    .extra(extra),
            )
            .moves_cursor(self.on("menus.moves_cursor"))
    }

    fn placement(&self) -> Placement {
        match self.get("menus.placement") {
            1 => Placement::Above,
            2 => Placement::Right,
            3 => Placement::Left,
            4 => Placement::Center,
            _ => Placement::Below,
        }
    }
}

pub struct MenusPage {
    template: Template,
    opts: Opts,
    pub files: ListBox,
    pub actions: Button,
    /// The row menu, about a file (an index into [`FILES`]).
    pub row_menu: RowMenu<usize>,
    /// The Actions menu (about nothing in particular).
    pub actions_menu: RowMenu<()>,
    /// The last request: what and where.
    pub request: String,
    pub last: String,
}

impl MenusPage {
    pub fn new() -> Self {
        let items = FILES
            .iter()
            .map(|(n, k, _)| ListItem::new(n).meta(k))
            .collect();
        let opts = Opts::new();
        let mut row_menu = RowMenu::default();
        row_menu.config = opts.context();
        Self {
            template: Template::new(ID, states_grid()),
            files: ListBox::new(ID.sub("files"), items, SelectMode::Single),
            actions: Button::secondary(ID.sub("actions"), "Actions ▾"),
            row_menu,
            actions_menu: RowMenu::default(),
            opts,
            request: "none yet · Pointer at a right-click, Control under the row for a key".into(),
            last: "none yet · Chosen(i) or Dismissed".into(),
        }
    }

    /// Open the menu `req` asks for. The list's display row maps to the
    /// file it shows (`ContextRequest::map`).
    fn open_row_menu(&mut self, req: ContextRequest<usize>) {
        let req = req.map(|row| row.min(FILES.len() - 1));
        let (name, _, read_only) = FILES[req.target];
        self.request = match req.anchor {
            ContextAnchor::Pointer(p) => format!("{name} · at the pointer ({}, {})", p.x, p.y),
            ContextAnchor::Control { .. } => format!("{name} · under its row (keyboard)"),
        };
        let mut m = self.opts.style(req.menu(ROW_MENU, row_items(read_only)));
        if self.opts.on("menus.title") {
            m = m.title(name);
        }
        self.row_menu.open(m, req.target);
    }

    fn open_actions(&mut self) {
        let m = ContextMenu::new(ACTIONS_MENU, action_items())
            .anchor_control(self.actions.id, Rect::ZERO);
        let mut m = self.opts.style(m);
        m.list.placement = self.opts.placement();
        if self.opts.on("menus.title") {
            m = m.title("Actions");
        }
        self.request = "Actions · under its button".into();
        self.actions_menu.open(m, ());
    }

    fn chose(&mut self, what: &str, i: usize, items: &[MenuItem]) {
        let label = items.get(i).map_or("?", |it| it.item.label.as_str());
        self.last = format!("{:?} · {label} on {what}", MenuAction::Chosen(i));
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[11, 1, 6, 0]);
        let (left, right) = crate::pages::layout::columns(rows[0], rows[0].width / 2, 2);

        let panel = Panel::card(Some("Files"))
            .meta("right-click · Shift+F10")
            .focused(ctx.interaction.focused(self.files.id));
        let bg = panel.bg(t);
        let inner = panel.render(left, buf, t);
        self.files.render(inner, buf, ctx, bg);

        let panel = Panel::card(Some("Actions menu")).meta("click · Enter");
        let bg = panel.bg(t);
        let inner = panel.render(right, buf, t);
        let bw = self.actions.width().min(inner.width);
        self.actions
            .render(Rect::new(inner.x, inner.y, bw, 1), buf, ctx, bg);
        let notes = [
            "Right-press a file, drag onto an item,",
            "release: the release chooses it.",
            "A right-click outside an open menu",
            "closes it and opens the one beneath.",
        ];
        for (i, n) in notes.iter().enumerate() {
            let y = inner.y + 2 + i as u16;
            if y < inner.bottom() {
                buf.set_string(inner.x, y, n, t.muted().bg(bg));
            }
        }

        let card = Panel::card(Some("Last"));
        let bg = card.bg(t);
        let inner = card.render(rows[2], buf, t);
        let keys = keys_readout(&self.row_menu.config);
        let lines = [
            ("Request", self.request.clone()),
            ("Action", self.last.clone()),
            ("Keys", keys),
        ];
        for (i, (k, v)) in lines.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            buf.set_string(inner.x, y, k, t.muted().bg(bg));
            let w = inner.width.saturating_sub(10) as usize;
            buf.set_string(
                inner.x + 10,
                y,
                junie_tui::ui::text::truncate(v, w),
                t.primary().bg(bg),
            );
        }
        // the open menus float above the page, placed on the whole screen
        let screen = *buf.area();
        self.row_menu.render(screen, buf, ctx);
        self.actions_menu.render(screen, buf, ctx);
    }
}

/// Which keys open a row's menu.
fn keys_readout(c: &ContextConfig) -> String {
    let mut parts = vec!["right-click".to_owned()];
    if c.keys.shift_f10 {
        parts.push("Shift+F10".into());
    }
    if c.keys.menu_key {
        parts.push("Menu key".into());
    }
    if let Some(k) = c.keys.extra
        && let KeyCode::Char(ch) = k.code
    {
        parts.push(format!("{ch} (extra key)"));
    }
    let mut s = parts.join(" · ");
    s.push_str(if c.moves_cursor {
        " — a click moves the cursor"
    } else {
        " — a click leaves the cursor"
    });
    s
}

/// The States grid: a command menu and one with reasons and a separator.
fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    StateGrid::new(id)
        .rows(&[Rest, Hover, Pressed, Disabled, Overflow, Narrow])
        .column_width(30)
        .narrow_width(14)
        .column(
            "Commands",
            ContextMenu::new(
                id.child(0),
                vec![
                    MenuItem::new("Open").shortcut("Enter"),
                    MenuItem::new("Rename").shortcut("F2"),
                    MenuItem::new("Delete").danger(),
                ],
            ),
        )
        .column(
            "Reasons",
            ContextMenu::new(
                id.child(1),
                vec![
                    MenuItem::new("Copy path"),
                    MenuItem::new("Share…").reason("Off").separator_if(true),
                    MenuItem::new("Trash").danger_if(true),
                ],
            )
            .title("README.md"),
        )
}

impl Page for MenusPage {
    fn title(&self) -> &'static str {
        "Menus"
    }

    fn blurb(&self) -> &'static str {
        "Context menus: anchors, placement, reasons, danger, separators, keys and press-drag-release"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let body = self.template.render(area, buf, ctx);
        match self.template.section() {
            Section::Playground => self.render_playground(body, buf, ctx),
            Section::Options => self.template.render_options(body, buf, ctx),
            Section::States => self.template.render_states(body, buf, ctx),
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut Cx) -> Outcome {
        if let Some((out, chosen)) = self.row_menu.handle(ev) {
            match chosen {
                Some((file, i)) => {
                    let items = row_items(FILES[file].2);
                    self.chose(FILES[file].0, i, &items);
                }
                None if !self.row_menu.is_open() => {
                    self.last = format!("{:?}", MenuAction::Dismissed);
                }
                None => {}
            }
            return out;
        }
        if let Some((out, chosen)) = self.actions_menu.handle(ev) {
            match chosen {
                Some(((), i)) => self.chose("the folder", i, &action_items()),
                None if !self.actions_menu.is_open() => {
                    self.last = format!("{:?}", MenuAction::Dismissed);
                }
                None => {}
            }
            return out;
        }
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        if self.template.section() != Section::Playground {
            return Outcome::Ignored;
        }
        match ev {
            PageEvent::Key(key) => {
                let f = cx.focus.current();
                if f == Some(self.files.id) {
                    if let Some(req) = self.row_menu.config.on_key(&self.files, key) {
                        self.open_row_menu(req);
                        return Outcome::Changed;
                    }
                    return self.files.on_key(key);
                }
                if f == Some(self.actions.id) {
                    let (out, activated) = self.actions.on_key(key);
                    if activated {
                        self.open_actions();
                        return Outcome::Changed;
                    }
                    return out;
                }
                Outcome::Ignored
            }
            PageEvent::Pointer(ev) if ev.is(Phase::Secondary) => {
                let cfg = self.row_menu.config;
                match cfg.on_secondary(&mut self.files, ev) {
                    Some(req) => {
                        cx.focus.focus(self.files.id);
                        self.open_row_menu(req);
                        Outcome::Changed
                    }
                    None => Outcome::Ignored,
                }
            }
            PageEvent::Pointer(ev) => {
                if let Some(r) = self.actions.route(ev) {
                    if !ev.is(Phase::Click) {
                        return r.outcome();
                    }
                    cx.focus.focus(self.actions.id);
                    if r.into_action().is_some() {
                        self.open_actions();
                    }
                    return Outcome::Changed;
                }
                if ev.on(self.files.id) && ev.is(Phase::Click) && ev.index_of(Part::ROW).is_some() {
                    cx.focus.focus(self.files.id);
                }
                if let Some(r) = self.files.route(ev) {
                    return r.outcome();
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if self.row_menu.is_open() || self.actions_menu.is_open() {
            return MENU_HINTS.to_vec();
        }
        if let Some(h) = self.template.hints(focus) {
            return h;
        }
        if focus == Some(self.files.id) {
            let mut h = vec![("↑↓", "Move")];
            if self.row_menu.config.keys.shift_f10 {
                h.push(("Shift+F10", "Menu"));
            }
            if self.row_menu.config.keys.extra.is_some() {
                h.push(("m", "Menu"));
            }
            return h;
        }
        if focus == Some(self.actions.id) {
            return vec![("Enter / Space", "Open menu")];
        }
        Vec::new()
    }

    fn layer_open(&self) -> bool {
        self.row_menu.hides_page_focus()
            || self.actions_menu.hides_page_focus()
            || self.template.popup_open()
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        self.opts
            .values
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
    }

    fn set_option(&mut self, key: &str, value: usize) {
        if let Some(slot) = self.opts.values.iter_mut().find(|(k, _)| *k == key) {
            slot.1 = value;
            self.row_menu.config = self.opts.context();
        }
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn template(&self) -> Option<&Template> {
        Some(&self.template)
    }

    fn template_mut(&mut self) -> Option<&mut Template> {
        Some(&mut self.template)
    }
}

pub fn page() -> Box<dyn Page> {
    Box::new(MenusPage::new())
}
