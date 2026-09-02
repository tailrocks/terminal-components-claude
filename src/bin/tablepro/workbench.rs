//! Workbench: explorer pane + tab strip + tab bodies for one connection.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::empty::EmptyState;
use junie_tui::widgets::input::{InputEvent, TextInput};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::table::SortDir;
use junie_tui::widgets::tabs::{TabEvent, TabItem, Tabs};
use junie_tui::widgets::tree::{TreeEvent, TreeNode, TreeView};

use crate::app::Cx;
use crate::db::{Catalog, Connection, ObjectKind};
use crate::model::History;
use crate::tabs::{HistoryTab, QueryTab, TableTab};

const ID: WidgetId = WidgetId::of("workbench");
pub const EXPLORER: WidgetId = ID.sub("explorer");
pub const TABSTRIP: WidgetId = ID.sub("tabstrip");

#[allow(clippy::large_enum_variant)] // a handful of tabs; boxing buys nothing
pub type PendingRun = (
    usize,
    Vec<(String, std::ops::Range<usize>)>,
    bool,
    Option<bool>,
);

#[allow(clippy::large_enum_variant)] // a handful of tabs; boxing buys nothing
pub enum WorkTab {
    Table(TableTab),
    Query(QueryTab),
    History(HistoryTab),
}

impl WorkTab {
    pub fn label(&self) -> String {
        match self {
            WorkTab::Table(t) => t.label(),
            WorkTab::Query(q) => q.name.clone(),
            WorkTab::History(_) => "History".into(),
        }
    }
    pub fn is_editing(&self) -> bool {
        match self {
            WorkTab::Table(t) => t.is_editing(),
            WorkTab::Query(q) => q.is_editing(),
            WorkTab::History(h) => h.search.editing,
        }
    }
    pub fn dirty(&self) -> bool {
        match self {
            WorkTab::Table(t) => t.dirty_count() > 0,
            WorkTab::Query(q) => q.dirty(),
            WorkTab::History(_) => false,
        }
    }
}

pub struct Workbench {
    pub connection: Connection,
    pub catalog: Catalog,
    pub schema: String,
    pub explorer: TreeView,
    pub explorer_filter: TextInput,
    pub explorer_visible: bool,
    pub maximized: bool,
    pub strip: Tabs,
    pub tabs: Vec<WorkTab>,
    pub active: usize,
    pub query_counter: usize,
    pending_loads: Vec<(Vec<usize>, u32)>,
    /// Focus to restore when a maximized pane is restored.
    pub open_objects: Vec<(String, String)>,
    /// Explorer `current` object: the one behind the active tab.
    pub current_object: Option<(String, String)>,
    /// Statements waiting for a safety decision: (tab index, statements, all, explain)
    pub pending_run: Option<PendingRun>,
    pub closed: Vec<(String, String)>,
}

fn kind_glyph(k: ObjectKind) -> &'static str {
    match k {
        ObjectKind::Table => "T",
        ObjectKind::View => "V",
        ObjectKind::Function => "ƒ",
        ObjectKind::Sequence => "#",
    }
}

impl Workbench {
    pub fn new(connection: Connection, catalog: Catalog) -> Self {
        let mut wb = Self {
            connection,
            catalog,
            schema: "public".into(),
            explorer: TreeView::new(EXPLORER, vec![]),
            explorer_filter: TextInput::new(EXPLORER.sub("filter"), "")
                .placeholder("Filter objects")
                .plain_label(),
            explorer_visible: true,
            maximized: false,
            strip: Tabs::new(TABSTRIP, &[]),
            tabs: vec![],
            active: 0,
            query_counter: 0,
            pending_loads: vec![],
            open_objects: vec![],
            current_object: None,
            pending_run: None,
            closed: vec![],
        };
        wb.strip.allow_new = true;
        wb.build_explorer();
        wb
    }

    // ---- explorer ----------------------------------------------------

    /// Rebuild the explorer around the current schema (keeps tabs).
    pub fn rebuild_for_schema(&mut self) {
        self.build_explorer();
        self.sync_strip();
    }

    fn build_explorer(&mut self) {
        let db = TreeNode::dir(
            &self.catalog.database.clone(),
            self.catalog
                .schemas
                .iter()
                .map(|s| {
                    // schemas load lazily except the current one
                    if *s == self.schema {
                        TreeNode::dir(s, self.schema_children(s)).glyph("S")
                    } else {
                        TreeNode::lazy(s).glyph("S")
                    }
                })
                .collect(),
        )
        .glyph("D");
        self.explorer = TreeView::new(EXPLORER, vec![db]);
        self.explorer.expanded.insert(vec![0]);
        if let Some(i) = self.catalog.schemas.iter().position(|s| *s == self.schema) {
            self.explorer.expanded.insert(vec![0, i]);
            // Tables section open by default
            self.explorer.expanded.insert(vec![0, i, 0]);
        }
        self.explorer.flatten();
        self.explorer.cursor = 1;
    }

    fn schema_children(&self, schema: &str) -> Vec<TreeNode> {
        let mut sections = vec![];
        for (kind, label) in [
            (ObjectKind::Table, "Tables"),
            (ObjectKind::View, "Views"),
            (ObjectKind::Function, "Functions"),
            (ObjectKind::Sequence, "Sequences"),
        ] {
            let objs: Vec<&crate::db::Table> = self.catalog.tables_in(schema, kind).collect();
            if objs.is_empty() && kind != ObjectKind::Table {
                continue;
            }
            let children: Vec<TreeNode> = if objs.is_empty() {
                vec![TreeNode::note("No tables")]
            } else {
                objs.iter()
                    .map(|t| {
                        let meta = if t.row_count > 0 {
                            crate::sql::fmt_rows(t.row_count)
                        } else {
                            String::new()
                        };
                        if kind == ObjectKind::Table || kind == ObjectKind::View {
                            TreeNode::lazy(&t.name).glyph(kind_glyph(kind)).meta(&meta)
                        } else {
                            TreeNode::leaf(&t.name)
                                .glyph(kind_glyph(kind))
                                .meta(t.comment.as_deref().unwrap_or(""))
                        }
                    })
                    .collect()
            };
            let mut section = TreeNode::dir(label, children);
            section.meta = Some(objs.len().to_string());
            sections.push(section);
        }
        sections
    }

    fn table_children(&self, t: &crate::db::Table) -> Vec<TreeNode> {
        let mut out = vec![];
        let cols: Vec<TreeNode> = t
            .columns
            .iter()
            .map(|c| {
                let mut meta = c.ty.sql().to_owned();
                if c.primary {
                    meta.push_str(" · pk");
                } else if c.references.is_some() {
                    meta.push_str(" · fk");
                }
                TreeNode::leaf(&c.name).glyph("·").meta(&meta)
            })
            .collect();
        let mut cn = TreeNode::dir("Columns", cols);
        cn.meta = Some(t.columns.len().to_string());
        out.push(cn);
        let idx: Vec<TreeNode> = t
            .indexes
            .iter()
            .map(|i| {
                TreeNode::leaf(&i.name).meta(&format!(
                    "{}{}",
                    if i.unique { "unique · " } else { "" },
                    i.columns.join(", ")
                ))
            })
            .collect();
        let mut ix = TreeNode::dir(
            "Indexes",
            if idx.is_empty() {
                vec![TreeNode::note("No indexes")]
            } else {
                idx
            },
        );
        ix.meta = Some(t.indexes.len().to_string());
        out.push(ix);
        let keys: Vec<TreeNode> = t
            .constraints
            .iter()
            .filter(|k| k.kind.contains("KEY"))
            .map(|k| TreeNode::leaf(&k.name).meta(k.kind))
            .collect();
        let mut kn = TreeNode::dir(
            "Keys",
            if keys.is_empty() {
                vec![TreeNode::note("No keys")]
            } else {
                keys
            },
        );
        kn.meta = Some(
            t.constraints
                .iter()
                .filter(|k| k.kind.contains("KEY"))
                .count()
                .to_string(),
        );
        out.push(kn);
        let checks: Vec<TreeNode> = t
            .constraints
            .iter()
            .filter(|k| !k.kind.contains("KEY"))
            .map(|k| TreeNode::leaf(&k.name).meta(&k.definition))
            .collect();
        let mut chk = TreeNode::dir(
            "Constraints",
            if checks.is_empty() {
                vec![TreeNode::note("No constraints")]
            } else {
                checks
            },
        );
        chk.meta = Some(
            t.constraints
                .iter()
                .filter(|k| !k.kind.contains("KEY"))
                .count()
                .to_string(),
        );
        out.push(chk);
        let trg: Vec<TreeNode> = t
            .triggers
            .iter()
            .map(|tr| {
                let (n, rest) = tr.split_once(' ').unwrap_or((tr, ""));
                TreeNode::leaf(n).meta(rest)
            })
            .collect();
        let mut tn = TreeNode::dir(
            "Triggers",
            if trg.is_empty() {
                vec![TreeNode::note("No triggers")]
            } else {
                trg
            },
        );
        tn.meta = Some(t.triggers.len().to_string());
        out.push(tn);
        out
    }

    /// Resolve an explorer path to (schema, object) when it points at an object.
    fn object_at(&self, path: &[usize]) -> Option<(String, String)> {
        if path.len() < 4 || path[0] != 0 {
            return None;
        }
        let schema = self.catalog.schemas.get(path[1])?.clone();
        let sections: Vec<ObjectKind> = [
            ObjectKind::Table,
            ObjectKind::View,
            ObjectKind::Function,
            ObjectKind::Sequence,
        ]
        .into_iter()
        .filter(|k| *k == ObjectKind::Table || self.catalog.tables_in(&schema, *k).next().is_some())
        .collect();
        let kind = *sections.get(path[2])?;
        let obj = self.catalog.tables_in(&schema, kind).nth(path[3])?;
        Some((schema, obj.name.clone()))
    }

    fn schema_at(&self, path: &[usize]) -> Option<String> {
        if path.len() == 2 && path[0] == 0 {
            self.catalog.schemas.get(path[1]).cloned()
        } else {
            None
        }
    }

    pub fn tick_explorer(&mut self) -> bool {
        let mut changed = false;
        let mut done = vec![];
        for (i, (_, ticks)) in self.pending_loads.iter_mut().enumerate() {
            *ticks = ticks.saturating_sub(1);
            if *ticks == 0 {
                done.push(i);
            }
        }
        for i in done.into_iter().rev() {
            let (path, _) = self.pending_loads.remove(i);
            let children = if let Some(schema) = self.schema_at(&path) {
                self.schema_children(&schema)
            } else if let Some((schema, name)) = self.object_at(&path) {
                match self.catalog.find(Some(&schema), &name) {
                    Some(t) => self.table_children(t),
                    None => vec![],
                }
            } else {
                vec![]
            };
            self.explorer.set_children(&path, children);
            changed = true;
        }
        changed
    }

    pub fn animating(&self) -> bool {
        !self.pending_loads.is_empty()
            || self
                .tabs
                .iter()
                .any(|t| matches!(t, WorkTab::Query(q) if q.is_running()))
    }

    // ---- tabs --------------------------------------------------------

    fn sync_strip(&mut self) {
        let mut labels: Vec<String> = self.tabs.iter().map(|t| t.label()).collect();
        // disambiguate duplicates with the schema
        for i in 0..labels.len() {
            if labels.iter().filter(|l| **l == labels[i]).count() > 1
                && let WorkTab::Table(t) = &self.tabs[i]
            {
                labels[i] = t.qualified();
            }
        }
        let items: Vec<TabItem> = self
            .tabs
            .iter()
            .zip(labels)
            .map(|(t, label)| {
                let mut it = TabItem::new(&label).closable();
                match t {
                    WorkTab::Table(tt) => {
                        it.prefix = Some("T".into());
                        it.dirty = tt.dirty_count() > 0;
                    }
                    WorkTab::Query(q) => {
                        it.prefix = Some("≡".into());
                        it.busy = q.is_running();
                        it.dirty = q.dirty() && !q.is_running();
                        it.error = q.last_status.as_ref().is_some_and(|s| s.1)
                            && !q.is_running()
                            && q.unseen;
                    }
                    WorkTab::History(_) => {
                        it.prefix = Some("H".into());
                    }
                }
                it
            })
            .collect();
        let active = self.active.min(items.len().saturating_sub(1));
        let first = self.strip.first;
        self.strip = Tabs::with_items(TABSTRIP, items);
        self.strip.allow_new = true;
        self.strip.first = first;
        self.strip.set_active(active);
        self.current_object = match self.tabs.get(self.active) {
            Some(WorkTab::Table(t)) => Some((t.schema.clone(), t.name.clone())),
            _ => None,
        };
        self.open_objects = self
            .tabs
            .iter()
            .filter_map(|t| {
                if let WorkTab::Table(t) = t {
                    Some((t.schema.clone(), t.name.clone()))
                } else {
                    None
                }
            })
            .collect();
    }

    pub fn open_table(&mut self, schema: &str, name: &str, preview: bool) -> Option<usize> {
        // reuse an existing tab for the object
        if let Some(i) = self
            .tabs
            .iter()
            .position(|t| matches!(t, WorkTab::Table(tt) if tt.schema == schema && tt.name == name))
        {
            self.active = i;
            if let WorkTab::Table(tt) = &mut self.tabs[i]
                && !preview
            {
                tt.preview = false;
            }
            self.sync_strip();
            return Some(i);
        }
        let table = self.catalog.find(Some(schema), name)?.clone();
        let tab = TableTab::new(
            ID.sub("tab")
                .child(self.tabs.len() + self.query_counter + 1000),
            &self.catalog,
            &table,
            preview,
        );
        // a preview tab is reused by the next single-click
        if preview
            && let Some(i) = self
                .tabs
                .iter()
                .position(|t| matches!(t, WorkTab::Table(tt) if tt.preview))
        {
            self.tabs[i] = WorkTab::Table(tab);
            self.active = i;
            self.sync_strip();
            return Some(i);
        }
        self.tabs.push(WorkTab::Table(tab));
        self.active = self.tabs.len() - 1;
        self.sync_strip();
        Some(self.active)
    }

    pub fn new_query(&mut self, text: &str) -> usize {
        self.query_counter += 1;
        let name = format!("Query {}", self.query_counter);
        let tab = QueryTab::new(ID.sub("query").child(self.query_counter), &name, text);
        self.tabs.push(WorkTab::Query(tab));
        self.active = self.tabs.len() - 1;
        self.sync_strip();
        self.active
    }

    pub fn open_history(&mut self, history: &History) -> usize {
        if let Some(i) = self
            .tabs
            .iter()
            .position(|t| matches!(t, WorkTab::History(_)))
        {
            self.active = i;
            if let WorkTab::History(h) = &mut self.tabs[i] {
                h.refresh(history, &self.connection.name);
            }
            self.sync_strip();
            return i;
        }
        let mut h = HistoryTab::new(ID.sub("history"));
        h.refresh(history, &self.connection.name);
        self.tabs.push(WorkTab::History(h));
        self.active = self.tabs.len() - 1;
        self.sync_strip();
        self.active
    }

    pub fn close_tab(&mut self, i: usize) {
        if i < self.tabs.len() {
            if let WorkTab::Table(t) = &self.tabs[i] {
                self.closed.push((t.schema.clone(), t.name.clone()));
            }
            self.tabs.remove(i);
            if self.active >= self.tabs.len() {
                self.active = self.tabs.len().saturating_sub(1);
            } else if self.active > i {
                self.active -= 1;
            }
            self.sync_strip();
        }
    }

    pub fn set_active(&mut self, i: usize) {
        if i < self.tabs.len() {
            self.active = i;
            if let WorkTab::Query(q) = &mut self.tabs[i] {
                q.unseen = false;
            }
            self.sync_strip();
        }
    }

    pub fn active_tab(&self) -> Option<&WorkTab> {
        self.tabs.get(self.active)
    }
    pub fn active_tab_mut(&mut self) -> Option<&mut WorkTab> {
        self.tabs.get_mut(self.active)
    }

    /// First focusable control of the active tab.
    pub fn primary_focus(&self) -> Option<WidgetId> {
        match self.tabs.get(self.active)? {
            WorkTab::Table(t) => Some(t.grid.id),
            WorkTab::Query(q) => Some(q.editor.id),
            WorkTab::History(h) => Some(h.list.id),
        }
    }

    pub fn is_editing(&self) -> bool {
        self.explorer_filter.editing || self.active_tab().is_some_and(|t| t.is_editing())
    }

    pub fn pending_total(&self) -> usize {
        self.tabs
            .iter()
            .map(|t| {
                if let WorkTab::Table(t) = t {
                    t.dirty_count()
                } else {
                    0
                }
            })
            .sum()
    }

    pub fn running(&self) -> Option<u32> {
        self.tabs.iter().find_map(|t| match t {
            WorkTab::Query(q) => q.running.as_ref().map(|r| r.started_ticks * 80),
            _ => None,
        })
    }

    // ---- hints -------------------------------------------------------

    pub fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if focus == Some(self.explorer.id) {
            return vec![
                hint("↑ ↓", "Move"),
                hint("Enter", "Open"),
                hint("→", "Expand"),
                hint("/", "Filter"),
                hint("Ctrl+O", "Quick open"),
            ];
        }
        if focus == Some(self.explorer_filter.id) {
            return vec![
                hint("Type", "Filter"),
                hint("↓", "Into tree"),
                hint("Esc", "Clear"),
            ];
        }
        if focus == Some(self.strip.id) {
            return vec![
                hint("← →", "Switch"),
                hint("Ctrl+T", "New query"),
                hint("x", "Close"),
                hint("Ctrl+G", "Tab list"),
                hint("z", "Zoom"),
            ];
        }
        match self.active_tab() {
            Some(WorkTab::Table(t)) => t.hints(focus),
            Some(WorkTab::Query(q)) => q.hints(focus),
            Some(WorkTab::History(h)) => h.hints(focus),
            None => vec![hint("Ctrl+T", "New query"), hint("0", "Explorer")],
        }
    }

    // ---- input -------------------------------------------------------

    pub fn on_key(&mut self, key: &Key, cx: &mut Cx, history: &mut History) -> Outcome {
        let Some(f) = cx.focus.current() else {
            return Outcome::Ignored;
        };
        if f == self.explorer_filter.id {
            let (o, ev) = self.explorer_filter.on_key(key);
            match ev {
                Some(InputEvent::Changed)
                | Some(InputEvent::Committed)
                | Some(InputEvent::Cancelled) => {
                    let q = self.explorer_filter.text().to_owned();
                    self.explorer
                        .set_filter(if q.is_empty() { None } else { Some(&q) });
                }
                Some(InputEvent::CommittedTab { backward }) => {
                    if backward {
                        cx.focus_prev()
                    } else {
                        cx.focus_next()
                    }
                }
                None => {}
            }
            if !o.consumed() && key.is(KeyCode::Down) {
                cx.focus.focus(self.explorer.id);
                return Outcome::Changed;
            }
            return o;
        }
        if f == self.explorer.id {
            if key.is_char('/') {
                cx.focus.focus(self.explorer_filter.id);
                self.explorer_filter.begin_edit();
                return Outcome::Changed;
            }
            // Enter on a table/view opens it; the tree would only fold it
            if key.is(KeyCode::Enter) {
                let path = self
                    .explorer
                    .rows()
                    .get(self.explorer.cursor)
                    .map(|r| r.path.clone());
                if let Some(path) = path {
                    if let Some((schema, name)) = self.object_at(&path) {
                        let obj = self.catalog.find(Some(&schema), &name).map(|t| t.kind);
                        if matches!(obj, Some(ObjectKind::Table | ObjectKind::View)) {
                            self.explorer.selected = Some(path);
                            self.open_table(&schema, &name, false);
                            if let Some(pf) = self.primary_focus() {
                                cx.focus.focus(pf);
                            }
                            return Outcome::Changed;
                        }
                    }
                    if let Some(schema) = self.schema_at(&path) {
                        self.schema = schema;
                        cx.status(format!("Schema {}", self.schema));
                    }
                }
            }
            if key.is(KeyCode::F(5)) || key.is_char('r') {
                self.build_explorer();
                cx.status("Explorer refreshed");
                return Outcome::Changed;
            }
            let (o, ev) = self.explorer.on_key(key);
            if let Some(TreeEvent::Expand(path)) = ev {
                self.pending_loads.push((path, 3));
            }
            return o;
        }
        if f == self.strip.id {
            let (o, ev) = self.strip.on_key(key);
            match ev {
                Some(TabEvent::Activated(i)) => self.set_active(i),
                Some(TabEvent::Close(i)) => return self.request_close(i, cx),
                Some(TabEvent::New) => {
                    self.new_query("");
                    if let Some(pf) = self.primary_focus() {
                        cx.focus.focus(pf);
                    }
                }
                None => {}
            }
            return o;
        }
        let cat = &self.catalog;
        let conn = self.connection.name.clone();
        let db = self.catalog.database.clone();
        match self.tabs.get_mut(self.active) {
            Some(WorkTab::Table(t)) => Self::table_key(t, key, f, cx, cat, &conn, &db, history),
            Some(WorkTab::Query(q)) => q.on_key(key, cx, cat),
            Some(WorkTab::History(h)) => Self::history_key(h, key, f, cx, history, &conn),
            None => Outcome::Ignored,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn table_key(
        t: &mut TableTab,
        key: &Key,
        f: WidgetId,
        cx: &mut Cx,
        cat: &Catalog,
        conn: &str,
        db: &str,
        history: &mut History,
    ) -> Outcome {
        if f == t.mode_tabs.id {
            let (o, _) = t.mode_tabs.on_key(key);
            return o;
        }
        if f == t.structure_tabs.id {
            let (o, ev) = t.structure_tabs.on_key(key);
            if ev.is_some() {
                t.structure_refresh(cat);
            }
            return o;
        }
        if f == t.structure.id {
            return t.structure.on_key(key).0;
        }
        if f == t.ddl.id {
            return t.ddl.on_key(key);
        }
        if f == t.chips.id {
            let (o, ev) = t.chips.on_key(key);
            match ev {
                Some(junie_tui::widgets::chips::ChipEvent::Remove(i)) => {
                    t.filters.remove(i);
                    t.load(cat);
                    cx.status("Filter removed");
                }
                Some(junie_tui::widgets::chips::ChipEvent::Toggle(i)) => {
                    t.filters[i].enabled = !t.filters[i].enabled;
                    t.load(cat);
                }
                Some(junie_tui::widgets::chips::ChipEvent::Activate(i)) => {
                    cx.requests.push(crate::app::Request::EditFilter(Some(i)));
                }
                Some(junie_tui::widgets::chips::ChipEvent::Add) => {
                    cx.requests.push(crate::app::Request::EditFilter(None))
                }
                Some(junie_tui::widgets::chips::ChipEvent::ClearAll) => {
                    t.filters.clear();
                    t.load(cat);
                }
                Some(junie_tui::widgets::chips::ChipEvent::Lead) => {
                    t.match_all = !t.match_all;
                    t.load(cat);
                }
                None => {}
            }
            return o;
        }
        for bid in t.grid.bar_ids() {
            if f == bid {
                let (o, ev) = t.grid.on_bar_key(bid, key);
                if let Some(ev) = ev {
                    return Self::grid_event(t, ev, cx, cat, conn, db, history).or(o);
                }
                return o;
            }
        }
        if f == t.grid.id {
            let (o, ev) = t.grid.on_key(key);
            if let Some(ev) = ev {
                return Self::grid_event(t, ev, cx, cat, conn, db, history).or(o);
            }
            return o;
        }
        Outcome::Ignored
    }

    fn grid_event(
        t: &mut TableTab,
        ev: junie_tui::widgets::grid::GridEvent,
        cx: &mut Cx,
        cat: &Catalog,
        _conn: &str,
        _db: &str,
        _history: &mut History,
    ) -> Outcome {
        use junie_tui::widgets::grid::GridEvent;
        match ev {
            GridEvent::SortRequested(s) => {
                t.sort = s;
                t.load(cat);
                match s {
                    Some((c, d)) => cx.status(format!(
                        "Sorted by {} {}",
                        t.columns[c].0,
                        if d == SortDir::Asc {
                            "ascending"
                        } else {
                            "descending"
                        }
                    )),
                    None => cx.status("Sort cleared"),
                }
            }
            GridEvent::FetchMore => {
                cx.status("Fetch more: the demo engine caps results at 500 rows");
                t.grid.set_loading(false);
            }
            GridEvent::Refresh => {
                if t.dirty_count() > 0 {
                    cx.requests.push(crate::app::Request::ConfirmDiscard);
                } else {
                    t.load(cat);
                    cx.status("Refreshed");
                }
            }
            GridEvent::CommitRequested => cx.requests.push(crate::app::Request::CommitPending),
            GridEvent::DiscardRequested => cx.requests.push(crate::app::Request::ConfirmDiscard),
            GridEvent::PreviewSql => cx.requests.push(crate::app::Request::PreviewSql),
            GridEvent::Copy(s) => cx.status(format!("Copied {} chars", s.len())),
            GridEvent::FilterOnCell { col, value } => {
                cx.requests
                    .push(crate::app::Request::FilterOnCell(col, value));
            }
            GridEvent::OpenFilters => cx.requests.push(crate::app::Request::EditFilter(None)),
            GridEvent::ClearFilters => {
                t.filters.clear();
                t.load(cat);
                cx.status("Filters cleared");
            }
            GridEvent::FollowReference { row, col } => {
                let name = t.columns[col].0.clone();
                let v = t.grid.value(row, col).text();
                if let Some(target) = cat
                    .find(Some(&t.schema), &name)
                    .and(None::<String>)
                    .or_else(|| t.grid.columns[col].references.clone())
                {
                    cx.requests.push(crate::app::Request::OpenTableFiltered(
                        target,
                        "id".into(),
                        v,
                    ));
                }
            }
            GridEvent::OpenViewer { row, col } => {
                let v = t.grid.value(row, col).clone();
                cx.requests
                    .push(crate::app::Request::OpenViewer(t.columns[col].0.clone(), v));
            }
            GridEvent::LeaveForward => cx.focus_next(),
            GridEvent::LeaveBackward => cx.focus_prev(),
            GridEvent::CellChanged { .. }
            | GridEvent::RowInserted(_)
            | GridEvent::RowDeleted(_)
            | GridEvent::Activated(_) => {}
        }
        Outcome::Changed
    }

    fn history_key(
        h: &mut HistoryTab,
        key: &Key,
        f: WidgetId,
        cx: &mut Cx,
        history: &mut History,
        conn: &str,
    ) -> Outcome {
        if f == h.search.id {
            let (o, ev) = h.search.on_key(key);
            match ev {
                Some(InputEvent::Changed)
                | Some(InputEvent::Committed)
                | Some(InputEvent::Cancelled) => h.refresh(history, conn),
                Some(InputEvent::CommittedTab { backward }) => {
                    if backward {
                        cx.focus_prev()
                    } else {
                        cx.focus_next()
                    }
                }
                None => {}
            }
            if !o.consumed() && key.is(KeyCode::Down) {
                cx.focus.focus(h.list.id);
                return Outcome::Changed;
            }
            return o;
        }
        if f == h.list.id {
            match key.code {
                KeyCode::Char('/') => {
                    cx.focus.focus(h.search.id);
                    h.search.begin_edit();
                    return Outcome::Changed;
                }
                KeyCode::Char('c') => {
                    h.scope_all = !h.scope_all;
                    h.refresh(history, conn);
                    return Outcome::Changed;
                }
                KeyCode::Char('s') => {
                    h.failed_only = !h.failed_only;
                    h.refresh(history, conn);
                    return Outcome::Changed;
                }
                KeyCode::Enter => {
                    if let Some(e) = h.current_entry(history) {
                        cx.requests
                            .push(crate::app::Request::OpenQuery(e.sql.clone(), false));
                    }
                    return Outcome::Changed;
                }
                KeyCode::Char('r') => {
                    if let Some(e) = h.current_entry(history) {
                        cx.requests
                            .push(crate::app::Request::OpenQuery(e.sql.clone(), true));
                    }
                    return Outcome::Changed;
                }
                KeyCode::Char('y') => {
                    cx.status("Query copied");
                    return Outcome::Changed;
                }
                _ => {}
            }
            let o = h.list.on_key(key);
            h.sync_detail_public(history);
            return o;
        }
        if f == h.detail.id {
            return h.detail.on_key(key).0;
        }
        let mut hit: Option<(Outcome, bool, bool)> = None;
        for (b, run) in [(&mut h.open_btn, false), (&mut h.rerun_btn, true)] {
            if f == b.id {
                let (o, act) = b.on_key(key);
                hit = Some((o, act, run));
                break;
            }
        }
        if let Some((o, act, run)) = hit {
            if act && let Some(e) = h.current_entry(history) {
                cx.requests
                    .push(crate::app::Request::OpenQuery(e.sql.clone(), run));
            }
            return o;
        }
        if f == h.copy_btn.id {
            let (o, act) = h.copy_btn.on_key(key);
            if act {
                cx.status("Query copied");
            }
            return o;
        }
        Outcome::Ignored
    }

    fn request_close(&mut self, i: usize, cx: &mut Cx) -> Outcome {
        if self.tabs.get(i).is_some_and(|t| t.dirty()) {
            cx.requests.push(crate::app::Request::ConfirmCloseTab(i));
        } else {
            self.close_tab(i);
        }
        Outcome::Changed
    }

    pub fn on_click(
        &mut self,
        id: WidgetId,
        pos: Position,
        cx: &mut Cx,
        history: &mut History,
    ) -> Outcome {
        if id == self.explorer_filter.id {
            let was = cx.focus.is(id);
            cx.focus.focus(id);
            return self.explorer_filter.on_click(pos, was);
        }
        if let Some((row, toggle)) = self.explorer.locate(id) {
            cx.focus.focus(self.explorer.id);
            let path = self.explorer.rows().get(row).map(|r| r.path.clone());
            if !toggle
                && let Some(path) = &path
                && let Some((schema, name)) = self.object_at(path)
            {
                let kind = self.catalog.find(Some(&schema), &name).map(|t| t.kind);
                if matches!(kind, Some(ObjectKind::Table | ObjectKind::View)) {
                    // single click: preview tab; click on the already-current object promotes it
                    let promote =
                        self.current_object.as_ref() == Some(&(schema.clone(), name.clone()));
                    self.explorer.cursor = row;
                    self.explorer.selected = Some(path.clone());
                    self.open_table(&schema, &name, !promote);
                    return Outcome::Changed;
                }
            }
            let (o, ev) = if toggle {
                self.explorer.on_click_toggle(row)
            } else {
                self.explorer.on_click_row(row)
            };
            if let Some(TreeEvent::Expand(path)) = ev {
                self.pending_loads.push((path, 3));
            }
            return o;
        }
        if id == scrollbar::id_for(self.explorer.id) {
            return self.explorer.on_scrollbar(pos);
        }
        if self.strip.owns(id) {
            cx.focus.focus(self.strip.id);
            let (o, ev) = self.strip.on_click(id);
            match ev {
                Some(TabEvent::Activated(i)) => {
                    self.set_active(i);
                    if let Some(pf) = self.primary_focus() {
                        cx.focus.focus(pf);
                    }
                }
                Some(TabEvent::Close(i)) => return self.request_close(i, cx),
                Some(TabEvent::New) => {
                    self.new_query("");
                    if let Some(pf) = self.primary_focus() {
                        cx.focus.focus(pf);
                    }
                }
                None => {}
            }
            return o;
        }
        let cat = &self.catalog;
        let conn = self.connection.name.clone();
        let db = self.catalog.database.clone();
        match self.tabs.get_mut(self.active) {
            Some(WorkTab::Table(t)) => {
                if t.mode_tabs.locate(id).is_some() {
                    cx.focus.focus(t.mode_tabs.id);
                    return t.mode_tabs.on_click(id).0;
                }
                if t.structure_tabs.locate(id).is_some() {
                    cx.focus.focus(t.structure_tabs.id);
                    let (o, ev) = t.structure_tabs.on_click(id);
                    if ev.is_some() {
                        t.structure_refresh(cat);
                    }
                    return o;
                }
                if t.structure.owns(id) {
                    cx.focus.focus(t.structure.id);
                    if let Some(c) = t.structure.locate_header(id) {
                        return t.structure.on_click_header(c);
                    }
                    if let Some((r, c)) = t.structure.locate(id) {
                        return t.structure.on_click_cell(r, c.unwrap_or(0), pos).0;
                    }
                    return Outcome::Changed;
                }
                if id == scrollbar::id_for(t.ddl.id) {
                    return t.ddl.on_scrollbar(pos);
                }
                if t.chips.owns(id) {
                    cx.focus.focus(t.chips.id);
                    let (o, ev) = t.chips.on_click(id);
                    match ev {
                        Some(junie_tui::widgets::chips::ChipEvent::Remove(i)) => {
                            t.filters.remove(i);
                            t.load(cat);
                        }
                        Some(junie_tui::widgets::chips::ChipEvent::Activate(i)) => {
                            cx.requests.push(crate::app::Request::EditFilter(Some(i)))
                        }
                        Some(junie_tui::widgets::chips::ChipEvent::Add) => {
                            cx.requests.push(crate::app::Request::EditFilter(None))
                        }
                        Some(junie_tui::widgets::chips::ChipEvent::Lead) => {
                            t.match_all = !t.match_all;
                            t.load(cat);
                        }
                        _ => {}
                    }
                    return o;
                }
                if t.grid.owns(id) {
                    if !t.grid.bar_ids().contains(&id) {
                        cx.focus.focus(t.grid.id);
                    }
                    let (o, ev) = t.grid.on_click(id, pos);
                    if let Some(ev) = ev {
                        return Self::grid_event(t, ev, cx, cat, &conn, &db, history).or(o);
                    }
                    return o;
                }
                Outcome::Ignored
            }
            Some(WorkTab::Query(q)) => q.on_click(id, pos, cx, cat),
            Some(WorkTab::History(h)) => {
                if id == h.search.id {
                    let was = cx.focus.is(id);
                    cx.focus.focus(id);
                    return h.search.on_click(pos, was);
                }
                if let Some(row) = h.list.locate(id) {
                    cx.focus.focus(h.list.id);
                    let o = h.list.on_click(row);
                    h.sync_detail_public(history);
                    return o;
                }
                if id == scrollbar::id_for(h.list.id) {
                    return h.list.on_scrollbar(pos);
                }
                if id == h.detail.id || id == scrollbar::id_for(h.detail.id) {
                    cx.focus.focus(h.detail.id);
                    return h.detail.on_click(pos, true);
                }
                let mut hit: Option<bool> = None;
                for (b, run) in [(&mut h.open_btn, false), (&mut h.rerun_btn, true)] {
                    if b.id == id && b.on_click() {
                        hit = Some(run);
                        break;
                    }
                }
                if let Some(run) = hit {
                    if let Some(e) = h.current_entry(history) {
                        cx.requests
                            .push(crate::app::Request::OpenQuery(e.sql.clone(), run));
                    }
                    return Outcome::Changed;
                }
                if h.copy_btn.id == id && h.copy_btn.on_click() {
                    cx.status("Query copied");
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            None => Outcome::Ignored,
        }
    }

    pub fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        if pressed == scrollbar::id_for(self.explorer.id) {
            return self.explorer.on_scrollbar(pos);
        }
        match self.tabs.get_mut(self.active) {
            Some(WorkTab::Table(t)) => {
                if t.grid.owns(pressed) {
                    return t.grid.on_drag(pressed, pos);
                }
                if pressed == scrollbar::id_for(t.structure.id) {
                    return t.structure.on_scrollbar(pos);
                }
                Outcome::Ignored
            }
            Some(WorkTab::Query(q)) => q.on_drag(pressed, pos),
            Some(WorkTab::History(h)) => {
                if pressed == scrollbar::id_for(h.list.id) {
                    return h.list.on_scrollbar(pos);
                }
                Outcome::Ignored
            }
            None => Outcome::Ignored,
        }
    }

    pub fn on_wheel(&mut self, id: WidgetId, delta: i32, horizontal: bool) -> Outcome {
        if self.explorer.owns(id) {
            return self.explorer.on_wheel(delta);
        }
        match self.tabs.get_mut(self.active) {
            Some(WorkTab::Table(t)) => {
                if t.grid.owns(id) {
                    return t.grid.on_wheel(delta, horizontal);
                }
                if t.structure.owns(id) {
                    return t.structure.on_wheel(delta);
                }
                if id == t.ddl.id || id == scrollbar::id_for(t.ddl.id) {
                    return t.ddl.on_wheel(delta);
                }
                Outcome::Ignored
            }
            Some(WorkTab::Query(q)) => q.on_wheel(id, delta, horizontal),
            Some(WorkTab::History(h)) => {
                if h.list.owns(id) {
                    return h.list.on_wheel(delta);
                }
                if id == h.detail.id || id == scrollbar::id_for(h.detail.id) {
                    return h.detail.on_wheel(delta, horizontal);
                }
                Outcome::Ignored
            }
            None => Outcome::Ignored,
        }
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if self.explorer_filter.editing {
            return self.explorer_filter.on_paste(text);
        }
        match self.tabs.get_mut(self.active) {
            Some(WorkTab::Table(t)) => t.grid.on_paste(text),
            Some(WorkTab::Query(q)) => q.on_paste(text),
            Some(WorkTab::History(h)) => h.search.on_paste(text),
            None => Outcome::Ignored,
        }
    }

    // ---- render ------------------------------------------------------

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, history: &History) {
        let t = ctx.theme;
        // tab strip
        self.strip
            .render(Rect::new(area.x, area.y, area.width, 2), buf, ctx, t.canvas);
        let body = Rect::new(
            area.x,
            area.y + 2,
            area.width,
            area.height.saturating_sub(2),
        );
        // Below 100 columns the explorer becomes a drawer: it takes the whole
        // body while it has focus and steps aside as soon as focus leaves.
        let narrow = area.width < 100;
        let explorer_focused = ctx.interaction.focused(self.explorer.id)
            || ctx.interaction.focused(self.explorer_filter.id);
        let show_explorer =
            self.explorer_visible && !self.maximized && (!narrow || explorer_focused);
        let explorer_w = (body.width / 4).clamp(28, 40);
        let (ex, main) = if show_explorer && narrow {
            (body, Rect::ZERO)
        } else if show_explorer {
            (
                Rect::new(body.x, body.y, explorer_w, body.height),
                Rect::new(
                    body.x + explorer_w + 1,
                    body.y,
                    body.width.saturating_sub(explorer_w + 1),
                    body.height,
                ),
            )
        } else {
            (Rect::ZERO, body)
        };
        if !ex.is_empty() {
            let ef = ctx.interaction.focused(self.explorer.id)
                || ctx.interaction.focused(self.explorer_filter.id);
            let panel = Panel::framed(Some("Explorer"))
                .focused(ef)
                .meta(&self.schema);
            let bg = panel.bg(t);
            let inner = panel.render(ex, buf, t);
            self.explorer_filter.render(
                Rect::new(inner.x.saturating_sub(1), inner.y, inner.width + 1, 2),
                buf,
                ctx,
                bg,
            );
            let tree_area = Rect::new(
                inner.x.saturating_sub(1),
                inner.y + 2,
                inner.width + 1,
                inner.height.saturating_sub(2),
            );
            // mark current / open objects via meta glyphs
            self.explorer.render(tree_area, buf, ctx, bg);
        } else if self.explorer_visible && !self.maximized {
            // still a focus stop: Tab reaches it, and focusing it opens the drawer
            ctx.control(self.explorer.id, Rect::ZERO, false);
        }
        // tab body pane
        if main.is_empty() {
            // the drawer covers the tab; keep its primary control in the ring so
            // Tab leaves the drawer and lands where work continues
            if let Some(pf) = self.primary_focus() {
                ctx.control(pf, Rect::ZERO, false);
            }
            return;
        }
        let Some(active) = self.tabs.get(self.active) else {
            let panel = Panel::framed(None);
            let inner = panel.render(main, buf, t);
            junie_tui::widgets::empty::render(inner, buf, t, &EmptyState::new("No open tabs").hint("Enter on a table in the explorer opens it · Ctrl+T starts a query · Ctrl+O opens anything"), t.canvas);
            return;
        };
        let title = match active {
            WorkTab::Table(tt) => format!("{} › {}", tt.schema, tt.name),
            WorkTab::Query(q) => q.name.clone(),
            WorkTab::History(_) => "Query history".into(),
        };
        let focus_in_tab = ctx.interaction.focus.is_some_and(|f| {
            f != self.explorer.id && f != self.explorer_filter.id && f != self.strip.id
        });
        let meta = match active {
            WorkTab::Table(tt) => format!(
                "{} · {} cols",
                if tt.preview { "preview" } else { "" },
                tt.columns.len()
            )
            .trim_start_matches(" · ")
            .to_owned(),
            WorkTab::Query(q) => q
                .last_duration
                .map(crate::tabs::duration_label)
                .unwrap_or_default(),
            WorkTab::History(_) => format!("{} entries", history.entries.len()),
        };
        let panel = Panel::framed(Some(&title))
            .focused(focus_in_tab)
            .meta(&meta);
        let bg = panel.bg(t);
        let inner = panel.render(main, buf, t);
        let conn = self.connection.name.clone();
        match self.tabs.get_mut(self.active) {
            Some(WorkTab::Table(tt)) => tt.render(inner, buf, ctx, bg),
            Some(WorkTab::Query(q)) => q.render(inner, buf, ctx, bg),
            Some(WorkTab::History(h)) => h.render(inner, buf, ctx, bg, history, &conn),
            None => {}
        }
    }
}

impl HistoryTab {
    pub fn sync_detail_public(&mut self, history: &History) {
        let text = self
            .current_entry(history)
            .map(|e| e.sql.clone())
            .unwrap_or_default();
        if self.detail.text() != text {
            self.detail.set_text(&text);
        }
    }
}
