//! Context menus on the connections list and the workbench: the explorer
//! tree, the tab strip, the data grids (a table tab's rows, a query tab's
//! result rows) and the History tab's query list.
//!
//! A secondary click on a row, cell or tab, or the context key on the
//! focused widget's cursor ([`ContextConfig::keys`]), opens a
//! [`ContextMenu`] of the commands that widget already has. Each item
//! replays the key its command is bound to on the widget, so a menu choice
//! runs exactly the code the key runs; a command that cannot run here or
//! now stays in the menu with a reason.

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use junie_tui::core::event::Key;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::columns::HeaderAction;
use junie_tui::ui::context::{ContextConfig, ContextRequest};
use junie_tui::widgets::grid::DataGrid;
use junie_tui::widgets::menu::{ContextMenu, MenuItem};
use junie_tui::widgets::table::SortDir;

use crate::connections::ConnectionsScreen;
use crate::db::ObjectKind;
use crate::workbench::{WorkTab, Workbench};

/// The context menu's id.
pub const MENU: WidgetId = WidgetId::of("context-menu");

/// What an open context menu is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// An explorer row.
    Explorer(usize),
    /// A tab in the workbench strip.
    Tab(usize),
    /// A `(display row, column)` cell of the active table tab's grid.
    Grid(usize, usize),
    /// A `(display row, column)` cell of the active query tab's results.
    Results(usize, usize),
    /// A row of the active History tab's query list.
    History(usize),
    /// A row of the connections list (a group or a connection).
    Connection(usize),
    /// A column header (or its resize seam) of the active tab's grid.
    Header(usize),
    /// The app menu (the TablePro and connection segments of the status
    /// strip): its items run [`AppCommand`]s.
    App,
}

/// A command of the app menu: the pointer's way to quit, to see the open
/// tabs and to leave a connection (mouse audit #40).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppCommand {
    /// The open-tabs list (Ctrl+G).
    OpenTabs,
    /// Close the workbench and go back to the connection list, where
    /// another connection can be opened.
    Disconnect,
    /// Quit, asking first when work would be lost (Ctrl+Q).
    Quit,
}

/// The app menu at `pos`: Open tabs… and Disconnect while connected (to
/// `connection`), then Quit.
pub fn app_menu(connection: Option<&str>, pos: Position) -> CtxMenu {
    let mut items = vec![];
    let mut app = vec![];
    if connection.is_some() {
        items.push(MenuItem::new("Open tabs…").shortcut("Ctrl+G"));
        app.push(AppCommand::OpenTabs);
        items.push(MenuItem::new("Disconnect").separator());
        app.push(AppCommand::Disconnect);
    }
    items.push(MenuItem::new("Quit TablePro").shortcut("Ctrl+Q"));
    app.push(AppCommand::Quit);
    let keys = vec![None; items.len()];
    let title = connection.unwrap_or("TablePro").to_owned();
    CtxMenu {
        menu: ContextMenu::new(MENU, items).at(pos).title(title),
        target: Target::App,
        owner: crate::workbench::EXPLORER,
        keys,
        header: Vec::new(),
        app,
    }
}

/// An open context menu: the popover, what it is about, the widget its
/// commands act on and the key each item replays there.
pub struct CtxMenu {
    pub menu: ContextMenu,
    pub target: Target,
    pub owner: WidgetId,
    pub keys: Vec<Option<Key>>,
    /// For a [`Target::Header`] menu: the command of each item (header
    /// commands run on the grid, not as a replayed key).
    pub header: Vec<HeaderAction>,
    /// For a [`Target::App`] menu: the command of each item.
    pub app: Vec<AppCommand>,
}

fn plain(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

fn ctrl(c: char) -> Key {
    Key {
        code: KeyCode::Char(c),
        mods: KeyModifiers::CONTROL,
    }
}

fn alt(c: char) -> Key {
    Key {
        code: KeyCode::Char(c),
        mods: KeyModifiers::ALT,
    }
}

/// Items paired with the key each one replays (`None`: unavailable).
type Items = Vec<(MenuItem, Option<Key>)>;

/// A command bound to `key`, or unavailable for `reason`.
fn cmd(items: &mut Items, item: MenuItem, key: Key, reason: Option<&str>) {
    match reason {
        Some(r) => items.push((item.reason(r), None)),
        None => items.push((item, Some(key))),
    }
}

/// Mark the last item as ending a group.
fn group(items: &mut Items) {
    if let Some((it, _)) = items.last_mut() {
        it.separator_after = true;
    }
}

/// The menu for a secondary click on hit `id` at `pos`, moving the
/// widget's cursor there per `cfg`.
pub fn on_secondary(
    w: &mut Workbench,
    cfg: &ContextConfig,
    id: WidgetId,
    pos: Position,
) -> Option<CtxMenu> {
    if let Some(req) = cfg.on_secondary(&mut w.explorer, id, pos) {
        return Some(build(w, req.map(Target::Explorer)));
    }
    if let Some(req) = cfg.on_secondary(&mut w.strip, id, pos) {
        return Some(build(w, req.map(Target::Tab)));
    }
    if let Some(g) = w.active_grid_any_mut()
        && let Some(col) = g.locate_header_menu(id)
    {
        let owner = g.id;
        let (header, items): (Vec<HeaderAction>, Vec<MenuItem>) =
            g.header_menu(col).into_iter().unzip();
        let keys = vec![None; items.len()];
        return Some(CtxMenu {
            menu: ContextMenu::new(MENU, items)
                .at(pos)
                .title(g.columns[col].name.clone()),
            target: Target::Header(col),
            owner,
            keys,
            header,
            app: Vec::new(),
        });
    }
    let req = match w.active_tab_mut()? {
        WorkTab::Table(t) => cfg
            .on_secondary(&mut t.grid, id, pos)?
            .map(|(r, c)| Target::Grid(r, c)),
        WorkTab::Query(q) => cfg
            .on_secondary(q.active_grid_mut()?, id, pos)?
            .map(|(r, c)| Target::Results(r, c)),
        WorkTab::History(h) => cfg.on_secondary(&mut h.list, id, pos)?.map(Target::History),
    };
    Some(build(w, req))
}

/// The menu for the context key `key` on the focused widget's cursor.
pub fn on_key(
    w: &Workbench,
    cfg: &ContextConfig,
    focus: Option<WidgetId>,
    key: &Key,
) -> Option<CtxMenu> {
    let f = focus?;
    if !cfg.keys.matches(key) {
        return None;
    }
    let req = if f == w.explorer.id {
        cfg.on_key(&w.explorer, key)?.map(Target::Explorer)
    } else if f == w.strip.id {
        cfg.on_key(&w.strip, key)?.map(Target::Tab)
    } else {
        match w.active_tab()? {
            WorkTab::Table(t) if f == t.grid.id && !t.grid.is_editing() => {
                cfg.on_key(&t.grid, key)?.map(|(r, c)| Target::Grid(r, c))
            }
            WorkTab::Query(q) => {
                let g = q.active_grid().filter(|g| g.id == f && !g.is_editing())?;
                cfg.on_key(g, key)?.map(|(r, c)| Target::Results(r, c))
            }
            WorkTab::History(h) if f == h.list.id => cfg.on_key(&h.list, key)?.map(Target::History),
            _ => return None,
        }
    };
    Some(build(w, req))
}

/// Put the widget's cursor back on `target` before an item's key replays
/// there (the strip or a grid may have moved since the menu opened).
pub fn select(w: &mut Workbench, target: Target) {
    use junie_tui::ui::context::ContextSource;
    match target {
        Target::Explorer(row) => w.explorer.context_select(row),
        Target::Tab(i) => w.strip.context_select(i),
        Target::Grid(r, c) => {
            if let Some(WorkTab::Table(t)) = w.active_tab_mut() {
                t.grid.context_select((r, c));
            }
        }
        Target::Results(r, c) => {
            if let Some(WorkTab::Query(q)) = w.active_tab_mut()
                && let Some(g) = q.active_grid_mut()
            {
                g.context_select((r, c));
            }
        }
        Target::History(row) => {
            if let Some(WorkTab::History(h)) = w.active_tab_mut() {
                h.list.context_select(row);
            }
        }
        Target::Connection(_) | Target::Header(_) | Target::App => {}
    }
}

/// The connections list's menu for a secondary click on hit `id` at
/// `pos`, moving its cursor (and the selected connection) there per `cfg`.
pub fn on_secondary_connections(
    c: &mut ConnectionsScreen,
    cfg: &ContextConfig,
    id: WidgetId,
    pos: Position,
) -> Option<CtxMenu> {
    let req = cfg.on_secondary(c.tree_mut(), id, pos)?;
    c.select_row(req.target);
    Some(build_connection(c, req.map(Target::Connection)))
}

/// The connections list's menu for the context key `key` on its cursor.
pub fn on_key_connections(
    c: &ConnectionsScreen,
    cfg: &ContextConfig,
    focus: Option<WidgetId>,
    key: &Key,
) -> Option<CtxMenu> {
    if focus? != c.tree().id {
        return None;
    }
    let req = cfg.on_key(c.tree(), key)?;
    Some(build_connection(c, req.map(Target::Connection)))
}

fn build_connection(c: &ConnectionsScreen, req: ContextRequest<Target>) -> CtxMenu {
    let Target::Connection(row) = req.target else {
        unreachable!("a connections menu is about a connection row")
    };
    let (title, items) = connection_items(c, row);
    finish(req, title, c.tree().id, items)
}

fn connection_items(c: &ConnectionsScreen, row: usize) -> (String, Items) {
    let mut items = Items::new();
    let Some(r) = c.tree().rows().get(row) else {
        return (String::new(), items);
    };
    if c.connection_at_row(row).is_some() {
        let busy = c.animating().then_some("A connection is in progress");
        cmd(
            &mut items,
            MenuItem::new("Connect").shortcut("Enter"),
            plain(KeyCode::Enter),
            busy,
        );
        group(&mut items);
        cmd(
            &mut items,
            MenuItem::new("Edit…").shortcut("e"),
            plain(KeyCode::Char('e')),
            None,
        );
        cmd(
            &mut items,
            MenuItem::new("Duplicate").shortcut("Alt+D"),
            alt('d'),
            None,
        );
        cmd(
            &mut items,
            MenuItem::new("Delete…").shortcut("d").danger(),
            plain(KeyCode::Char('d')),
            None,
        );
    } else if r.has_children {
        if r.expanded {
            cmd(
                &mut items,
                MenuItem::new("Collapse").shortcut("←"),
                plain(KeyCode::Left),
                None,
            );
        } else {
            cmd(
                &mut items,
                MenuItem::new("Expand").shortcut("→"),
                plain(KeyCode::Right),
                None,
            );
        }
    }
    group(&mut items);
    cmd(
        &mut items,
        MenuItem::new("New connection…").shortcut("Ctrl+N"),
        ctrl('n'),
        None,
    );
    cmd(
        &mut items,
        MenuItem::new("Filter").shortcut("/"),
        plain(KeyCode::Char('/')),
        None,
    );
    (r.label.clone(), items)
}

fn history_items(w: &Workbench, row: usize) -> (String, WidgetId, Items) {
    let mut items = Items::new();
    let Some(WorkTab::History(h)) = w.active_tab() else {
        return (String::new(), w.explorer.id, items);
    };
    let Some(it) = h.list.items.get(row) else {
        return (String::new(), h.list.id, items);
    };
    cmd(
        &mut items,
        MenuItem::new("Open in new tab").shortcut("Enter"),
        plain(KeyCode::Enter),
        None,
    );
    cmd(
        &mut items,
        MenuItem::new("Run in new tab").shortcut("r"),
        plain(KeyCode::Char('r')),
        None,
    );
    cmd(
        &mut items,
        MenuItem::new("Copy query").shortcut("y"),
        plain(KeyCode::Char('y')),
        None,
    );
    group(&mut items);
    cmd(
        &mut items,
        MenuItem::new("Search").shortcut("/"),
        plain(KeyCode::Char('/')),
        None,
    );
    let scope = if h.scope_all {
        "Show this connection only"
    } else {
        "Show all connections"
    };
    cmd(
        &mut items,
        MenuItem::new(scope).shortcut("c"),
        plain(KeyCode::Char('c')),
        None,
    );
    let status = if h.failed_only {
        "Show every status"
    } else {
        "Show failed only"
    };
    cmd(
        &mut items,
        MenuItem::new(status).shortcut("s"),
        plain(KeyCode::Char('s')),
        None,
    );
    (it.label.clone(), h.list.id, items)
}

fn build(w: &Workbench, req: ContextRequest<Target>) -> CtxMenu {
    let (title, owner, items) = match req.target {
        Target::Explorer(row) => explorer_items(w, row),
        Target::Tab(i) => tab_items(w, i),
        Target::Grid(r, c) => match w.active_tab() {
            Some(WorkTab::Table(t)) => grid_items(&t.grid, r, c, true),
            _ => (String::new(), w.explorer.id, vec![]),
        },
        Target::Results(r, c) => match w.active_tab() {
            Some(WorkTab::Query(q)) if q.active_grid().is_some() => {
                grid_items(q.active_grid().unwrap(), r, c, false)
            }
            _ => (String::new(), w.explorer.id, vec![]),
        },
        Target::History(row) => history_items(w, row),
        Target::Connection(_) | Target::Header(_) | Target::App => {
            (String::new(), w.explorer.id, vec![])
        }
    };
    finish(req, title, owner, items)
}

/// The menu for `req`: `items` titled `title`, acting on `owner`.
fn finish(req: ContextRequest<Target>, title: String, owner: WidgetId, items: Items) -> CtxMenu {
    let (items, keys): (Vec<MenuItem>, Vec<Option<Key>>) = items.into_iter().unzip();
    let mut menu = req.menu(MENU, items);
    if !title.is_empty() {
        menu = menu.title(title);
    }
    CtxMenu {
        menu,
        target: req.target,
        owner,
        keys,
        header: Vec::new(),
        app: Vec::new(),
    }
}

fn explorer_items(w: &Workbench, row: usize) -> (String, WidgetId, Items) {
    let mut items = Items::new();
    let Some(r) = w.explorer.rows().get(row) else {
        return (String::new(), w.explorer.id, items);
    };
    let enter = plain(KeyCode::Enter);
    if let Some((schema, name)) = w.object_at(&r.path) {
        let kind = w.catalog.find(Some(&schema), &name).map(|t| t.kind);
        let opens = matches!(kind, Some(ObjectKind::Table | ObjectKind::View));
        cmd(
            &mut items,
            MenuItem::new("Open").shortcut("Enter"),
            enter,
            (!opens).then_some("Tables and views only"),
        );
    } else if w.schema_at(&r.path).is_some() {
        cmd(
            &mut items,
            MenuItem::new("Use schema").shortcut("Enter"),
            enter,
            None,
        );
    }
    if r.has_children {
        if r.expanded {
            cmd(
                &mut items,
                MenuItem::new("Collapse").shortcut("←"),
                plain(KeyCode::Left),
                None,
            );
        } else {
            cmd(
                &mut items,
                MenuItem::new("Expand").shortcut("→"),
                plain(KeyCode::Right),
                None,
            );
        }
    }
    group(&mut items);
    cmd(
        &mut items,
        MenuItem::new("Filter objects…").shortcut("/"),
        plain(KeyCode::Char('/')),
        None,
    );
    cmd(
        &mut items,
        MenuItem::new("Refresh").shortcut("r"),
        plain(KeyCode::Char('r')),
        None,
    );
    (r.label.clone(), w.explorer.id, items)
}

fn tab_items(w: &Workbench, i: usize) -> (String, WidgetId, Items) {
    let mut items = Items::new();
    let title = w.tabs.get(i).map(|t| t.label()).unwrap_or_default();
    cmd(
        &mut items,
        MenuItem::new("Switch to tab").shortcut("Enter"),
        plain(KeyCode::Enter),
        (i == w.active).then_some("Current tab"),
    );
    cmd(
        &mut items,
        MenuItem::new("Close tab").shortcut("x"),
        plain(KeyCode::Char('x')),
        None,
    );
    group(&mut items);
    cmd(
        &mut items,
        MenuItem::new("New query tab").shortcut("Ctrl+T"),
        ctrl('t'),
        None,
    );
    cmd(
        &mut items,
        MenuItem::new("Tab list…").shortcut("Ctrl+G"),
        ctrl('g'),
        None,
    );
    (title, w.strip.id, items)
}

/// A grid cell's commands. `table`: the grid of a table tab (filters,
/// reload and saving pending edits live there); otherwise query results.
fn grid_items(g: &DataGrid, row: usize, col: usize, table: bool) -> (String, WidgetId, Items) {
    let mut items = Items::new();
    let Some(spec) = g.columns.get(col) else {
        return (String::new(), g.id, items);
    };
    if row >= g.len() {
        return (String::new(), g.id, items);
    }
    let src = g.source_row(row);
    let deleted = g.pending.deleted.contains(&src);
    let read_only = (!g.editable).then_some("Read-only rows");
    let cell_lock = read_only
        .or(spec.read_only.then_some("Read-only column"))
        .or(deleted.then_some("Row is deleted"));
    let title = format!("{} · row {}", spec.name, row + 1);

    cmd(
        &mut items,
        MenuItem::new("Copy cell").shortcut("y"),
        plain(KeyCode::Char('y')),
        None,
    );
    cmd(
        &mut items,
        MenuItem::new("Copy row").shortcut("Y"),
        plain(KeyCode::Char('Y')),
        None,
    );
    group(&mut items);
    cmd(
        &mut items,
        MenuItem::new("Edit cell").shortcut("Enter"),
        plain(KeyCode::Enter),
        cell_lock,
    );
    cmd(
        &mut items,
        MenuItem::new("Set NULL").shortcut("Del"),
        plain(KeyCode::Delete),
        cell_lock
            .or((!spec.nullable).then_some("NOT NULL column"))
            .or((!g.selected_rows.is_empty()).then_some("Rows are marked")),
    );
    group(&mut items);
    cmd(
        &mut items,
        MenuItem::new("Filter by this value").shortcut("f"),
        plain(KeyCode::Char('f')),
        (!table).then_some("Table tabs only"),
    );
    let sort = match g.sort {
        Some((c, SortDir::Asc)) if c == col => "Sort descending",
        Some((c, SortDir::Desc)) if c == col => "Clear sort",
        _ => "Sort ascending",
    };
    cmd(
        &mut items,
        MenuItem::new(sort).shortcut("s"),
        plain(KeyCode::Char('s')),
        (!spec.sortable).then_some("Not sortable"),
    );
    if spec.references.is_some() {
        cmd(
            &mut items,
            MenuItem::new("Open referenced row").shortcut("Ctrl+]"),
            ctrl(']'),
            (!table).then_some("Table tabs only"),
        );
    }
    group(&mut items);
    cmd(
        &mut items,
        MenuItem::new("Insert row").shortcut("+"),
        plain(KeyCode::Char('+')),
        read_only,
    );
    cmd(
        &mut items,
        MenuItem::new("Duplicate row").shortcut("Alt+D"),
        alt('d'),
        read_only,
    );
    if deleted {
        cmd(
            &mut items,
            MenuItem::new("Restore row").shortcut("-"),
            plain(KeyCode::Char('-')),
            read_only,
        );
    } else {
        cmd(
            &mut items,
            MenuItem::new("Delete row").shortcut("-").danger(),
            plain(KeyCode::Char('-')),
            read_only,
        );
    }
    if table {
        group(&mut items);
        let none = g.pending.is_empty().then_some("No pending changes");
        cmd(
            &mut items,
            MenuItem::new("Preview SQL").shortcut("p"),
            plain(KeyCode::Char('p')),
            none,
        );
        cmd(
            &mut items,
            MenuItem::new("Save changes").shortcut("Ctrl+S"),
            ctrl('s'),
            none,
        );
        cmd(
            &mut items,
            MenuItem::new("Discard changes").shortcut("U").danger(),
            plain(KeyCode::Char('U')),
            none,
        );
        cmd(
            &mut items,
            MenuItem::new("Refresh").shortcut("r"),
            plain(KeyCode::Char('r')),
            None,
        );
    }
    (title, g.id, items)
}
