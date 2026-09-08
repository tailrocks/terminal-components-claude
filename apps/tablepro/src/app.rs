//! `TablePro` application shell built only on the public `junie-tui` facade.

use junie_tui::{
    Action, ActionKey, App, Chord, Color, ColorLevel, Cx, FgStep, Field, Focusability, Form,
    FormAction, FormState, FrameRead, Grid, GridAction, GridEditor, GridState, Id, Intent, ItemKey,
    KeyCode, KeyMap, KeyModifiers, KeyPhase, Modifier, NodeKind, Panel, PanelKind, Part, Phase,
    Response, Role, RowUi, Size, Span, SplitAxis, SplitPane, SplitPaneState, StylePatch, Tabs,
    TabsAction, TabsState, TextInput, TextInputState, Theme, Tree, TreeAction, TreeNode, TreeState,
    Ui, UpdateCause, wrap,
};

use crate::connections::{self, ConnectionDraft, ConnectionsScreen};
use crate::db::{
    self, Catalog, ColType, ConnectOutcome, Connection, Environment, ObjectKind, SafeMode,
};
use crate::domain::ResultGrid;
use crate::tabs::{ExplorerItem, Tab, TableTab};
use crate::workbench::Workbench;

/// Minimum terminal width.
pub const MIN_WIDTH: u16 = 72;
/// Minimum terminal height.
pub const MIN_HEIGHT: u16 = 20;
const QUERY: Id = Id::root("tablepro.query");
const RESULTS: Id = Id::root("tablepro.results");
const CONNECTIONS: Id = Id::root("tablepro.connections.list");
const CONNECTIONS_PANEL: Id = Id::root("tablepro.connections.panel");
const EXPLORER: Id = Id::root("tablepro.workbench.explorer.tree");
const EXPLORER_PANEL: Id = Id::root("tablepro.workbench.explorer.panel");
const TAB_STRIP: Id = Id::root("tablepro.workbench.tab-strip");
const WORKBENCH_SPLIT: Id = Id::root("tablepro.workbench.split");
const RUN: ActionKey = ActionKey::custom("tablepro.run");
const QUIT: ActionKey = ActionKey::custom("tablepro.quit");
const OPEN: ActionKey = ActionKey::custom("tablepro.open");
const NEW_QUERY: ActionKey = ActionKey::custom("tablepro.new-query");
const HISTORY: ActionKey = ActionKey::custom("tablepro.history");
const STRUCTURE: ActionKey = ActionKey::custom("tablepro.structure");
const FORM: ActionKey = ActionKey::custom("tablepro.form");
const HELP: ActionKey = ActionKey::custom("tablepro.help");
const TAB_LIST: ActionKey = ActionKey::custom("tablepro.tab-list");
const FILTER: ActionKey = ActionKey::custom("tablepro.filter");
const PREVIEW: ActionKey = ActionKey::custom("tablepro.preview");
const SAVE: ActionKey = ActionKey::custom("tablepro.save");
const EXPLAIN: ActionKey = ActionKey::custom("tablepro.explain");
const CLEAR_QUERY: ActionKey = ActionKey::custom("tablepro.clear-query");
const COMPLETE: ActionKey = ActionKey::custom("tablepro.complete");
const PALETTE: ActionKey = ActionKey::custom("tablepro.palette");

const CONNECTION_DETAILS: Id = Id::root("tablepro.connections.details");
const CONTENT_FRAME: Id = Id::root("tablepro.workbench.content.frame");

const CONNECTION_DETAILS_TITLE_PATCH: [(Part, StylePatch); 1] = [(
    Part::TITLE,
    StylePatch::new()
        .set_fg(Role::Fg(FgStep::Secondary))
        .remove(Modifier::BOLD),
)];
const FRAMED_PANEL_PATCH: [(Part, StylePatch); 1] =
    [(Part::DETAIL, StylePatch::new().set_fg(Role::BorderStrong))];

/// Product-level screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Connection list and initial landing screen.
    Connections,
    /// Connected database workbench.
    Workbench,
}

/// Named visual surfaces retained from the historical showcase matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// Connection list.
    Connections,
    /// Failed connection state.
    ConnectionsFailed,
    /// Default workbench.
    WorkbenchDefault,
    /// Explorer focus.
    ExplorerFocused,
    /// Table data grid.
    TableGrid,
    /// Inline cell editing.
    GridCellEditing,
    /// Pending-change bar.
    PendingChangeBar,
    /// Structure view.
    StructureView,
    /// Query editor.
    QueryEditing,
    /// Completion popup state.
    CompletionPopup,
    /// Successful results.
    ResultsGrid,
    /// Error results.
    ErrorResult,
    /// Explain plan.
    ExplainPlan,
    /// History tab.
    HistoryTab,
    /// Quick switcher.
    QuickSwitcher,
    /// Tab-list picker.
    TabListPicker,
    /// Safe-mode picker.
    SafeModePicker,
    /// Filter editor.
    FilterEditor,
    /// Safety acknowledgement dialog.
    SafetyDialogTypedAck,
    /// Help dialog.
    HelpDialog,
    /// Maximised tab.
    MaximisedTab,
}

#[derive(Debug, Clone)]
enum ConnectionNode {
    Group {
        name: String,
    },
    Connection {
        index: usize,
        connection: Connection,
    },
}

#[derive(Debug, Clone)]
enum ExplorerNode {
    Database {
        name: String,
    },
    Schema {
        name: String,
    },
    Group {
        schema: String,
        name: String,
    },
    Object {
        item: ExplorerItem,
        prefix: &'static str,
    },
}

impl Surface {
    /// All matrix surfaces in stable order.
    pub const ALL: [Self; 21] = [
        Self::Connections,
        Self::ConnectionsFailed,
        Self::WorkbenchDefault,
        Self::ExplorerFocused,
        Self::TableGrid,
        Self::GridCellEditing,
        Self::PendingChangeBar,
        Self::StructureView,
        Self::QueryEditing,
        Self::CompletionPopup,
        Self::ResultsGrid,
        Self::ErrorResult,
        Self::ExplainPlan,
        Self::HistoryTab,
        Self::QuickSwitcher,
        Self::TabListPicker,
        Self::SafeModePicker,
        Self::FilterEditor,
        Self::SafetyDialogTypedAck,
        Self::HelpDialog,
        Self::MaximisedTab,
    ];
    /// Stable matrix label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Connections => "connections",
            Self::ConnectionsFailed => "connections-failed",
            Self::WorkbenchDefault => "workbench-default",
            Self::ExplorerFocused => "explorer-focused",
            Self::TableGrid => "table-grid",
            Self::GridCellEditing => "grid-cell-editing",
            Self::PendingChangeBar => "pending-change-bar",
            Self::StructureView => "structure-view",
            Self::QueryEditing => "query-editing",
            Self::CompletionPopup => "completion-popup",
            Self::ResultsGrid => "results-grid",
            Self::ErrorResult => "error-result",
            Self::ExplainPlan => "explain-plan",
            Self::HistoryTab => "history-tab",
            Self::QuickSwitcher => "quick-switcher",
            Self::TabListPicker => "tab-list-picker",
            Self::SafeModePicker => "safe-mode-picker",
            Self::FilterEditor => "filter-editor",
            Self::SafetyDialogTypedAck => "safety-dialog-typed-ack",
            Self::HelpDialog => "help-dialog",
            Self::MaximisedTab => "maximised-tab",
        }
    }
}

fn keymap() -> KeyMap {
    KeyMap::new()
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('r'), KeyModifiers::CONTROL),
            RUN,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            QUIT,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('o'), KeyModifiers::CONTROL),
            OPEN,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('t'), KeyModifiers::CONTROL),
            NEW_QUERY,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('y'), KeyModifiers::CONTROL),
            HISTORY,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('d'), KeyModifiers::CONTROL),
            STRUCTURE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('n'), KeyModifiers::CONTROL),
            FORM,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('g'), KeyModifiers::CONTROL),
            TAB_LIST,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('l'), KeyModifiers::CONTROL),
            CLEAR_QUERY,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('x'), KeyModifiers::CONTROL),
            EXPLAIN,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('x'), KeyModifiers::ALT),
            EXPLAIN,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char(' '), KeyModifiers::CONTROL),
            COMPLETE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('p'), KeyModifiers::CONTROL),
            PREVIEW,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL),
            SAVE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('\\'), KeyModifiers::CONTROL),
            PALETTE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::F(5), KeyModifiers::NONE),
            RUN,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('?'), KeyModifiers::NONE),
            HELP,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('f'), KeyModifiers::NONE),
            FILTER,
        )
}

fn fallback_connection(catalog: &Catalog) -> Connection {
    Connection {
        name: "Local PostgreSQL".to_owned(),
        engine: db::Engine::Postgres,
        host: "localhost".to_owned(),
        port: 5432,
        database: catalog.database.clone(),
        user: "postgres".to_owned(),
        environment: Environment::Local,
        safe_mode: SafeMode::Silent,
        ssl: false,
        ssh: None,
        group: "Personal".to_owned(),
        last_used: "never".to_owned(),
        outcome: ConnectOutcome::Ok,
    }
}

/// `TablePro` state and app-owned adapters.
pub struct TableProApp {
    catalog: Catalog,
    connections: Vec<Connection>,
    connection: Connection,
    keymap: KeyMap,
    safe_mode: SafeMode,
    query: String,
    query_state: TextInputState,
    columns: Vec<(String, ColType)>,
    result: ResultGrid,
    grid_state: GridState,
    status: String,
    quit: bool,
    /// Current product screen.
    pub screen: Screen,
    /// Current visual matrix surface.
    pub surface: Surface,
    /// Connection list state.
    pub connections_screen: ConnectionsScreen,
    /// Connected workbench.
    pub workbench: Workbench,
    connection_nodes: Vec<ConnectionNode>,
    connection_tree_state: TreeState,
    connection_visual_tree_state: TreeState,
    explorer_nodes: Vec<ExplorerNode>,
    explorer_tree_state: TreeState,
    tabs_state: TabsState,
    split_state: SplitPaneState,
    draft: Option<ConnectionDraft>,
    form_state: FormState,
    form_fields: Box<[junie_tui::FieldSpec<'static>]>,
    form_actions: Box<[Action<'static>]>,
    form_open: bool,
}

impl core::fmt::Debug for TableProApp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TableProApp")
            .field("catalog", &self.catalog)
            .field("screen", &self.screen)
            .field("surface", &self.surface)
            .field("connections", &self.connections.len())
            .field("connection", &self.connection.name)
            .field("keymap", &"<keymap>")
            .field("safe_mode", &self.safe_mode)
            .field("query", &"[redacted]")
            .field("query_state", &"<input state>")
            .field("columns", &self.columns.len())
            .field("result", &self.result)
            .field("grid_state", &"<grid state>")
            .field("status", &self.status)
            .field("quit", &self.quit)
            .field("connections_screen", &self.connections_screen)
            .field("workbench", &self.workbench)
            .field("connection_nodes", &self.connection_nodes.len())
            .field("connection_tree_state", &self.connection_tree_state)
            .field(
                "connection_visual_tree_state",
                &self.connection_visual_tree_state,
            )
            .field("explorer_nodes", &self.explorer_nodes.len())
            .field("explorer_tree_state", &self.explorer_tree_state)
            .field("tabs_state", &"<tabs state>")
            .field("split_state", &"<split state>")
            .field("draft", &self.draft.as_ref().map(|_| "[redacted]"))
            .field("form_state", &"<form state>")
            .field("form_fields", &self.form_fields.len())
            .field("form_actions", &self.form_actions.len())
            .field("form_open", &self.form_open)
            .finish()
    }
}

impl Default for TableProApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TableProApp {
    /// Construct the deterministic demo app.
    pub fn new() -> Self {
        let catalog = Catalog::acme_prod();
        let connections = db::connections();
        let connection = connections
            .first()
            .cloned()
            .unwrap_or_else(|| fallback_connection(&catalog));
        let connection_nodes = build_connection_nodes(&connections);
        let connection_tree_state = initial_connection_tree_state(&connection_nodes);
        let connection_visual_tree_state = initial_connection_visual_tree_state(&connection_nodes);
        let explorer_nodes = build_explorer_nodes(&catalog);
        let explorer_tree_state = initial_explorer_tree_state(&explorer_nodes);
        let mut app = Self {
            safe_mode: connection.safe_mode,
            catalog: catalog.clone(),
            connections: connections.clone(),
            connection: connection.clone(),
            keymap: keymap(),
            query:
                "SELECT * FROM orders WHERE status = 'pending' ORDER BY total_amount DESC LIMIT 20"
                    .to_owned(),
            query_state: TextInputState::default(),
            columns: Vec::new(),
            result: ResultGrid::empty(),
            grid_state: GridState::default(),
            status: "Ready · Ctrl+R runs · Ctrl+Q quits".to_owned(),
            quit: false,
            screen: Screen::Connections,
            surface: Surface::Connections,
            connections_screen: ConnectionsScreen::new(connections),
            workbench: Workbench::new(connection, catalog),
            connection_nodes,
            connection_tree_state,
            connection_visual_tree_state,
            explorer_nodes,
            explorer_tree_state,
            tabs_state: TabsState::default(),
            split_state: SplitPaneState::default(),
            draft: None,
            form_state: FormState::default(),
            form_fields: Box::from(connections::form_fields()),
            form_actions: Box::from(connections::form_actions()),
            form_open: false,
        };
        let _ = app.execute_query();
        app
    }
    /// Active safe-mode policy.
    pub const fn safe_mode(&self) -> SafeMode {
        self.safe_mode
    }
    /// Current SQL text.
    pub fn query(&self) -> &str {
        &self.query
    }
    /// Current result adapter.
    pub const fn result(&self) -> &ResultGrid {
        &self.result
    }
    /// Latest status text.
    pub fn status(&self) -> &str {
        &self.status
    }
    /// Current screen.
    pub const fn screen(&self) -> Screen {
        self.screen
    }
    /// Current visual surface.
    pub const fn surface(&self) -> Surface {
        self.surface
    }
    /// Set a named surface and materialize its deterministic capture state.
    ///
    /// Visual captures enter through this method instead of mutating the
    /// surface marker after constructing an unrelated screen.  That keeps the
    /// renderer on the same connection/workbench route as the product.
    pub fn set_surface(&mut self, surface: Surface) {
        self.form_open = false;
        self.draft = None;

        match surface {
            Surface::Connections => {
                self.screen = Screen::Connections;
                self.connections_screen.error = None;
                self.surface = surface;
            }
            Surface::ConnectionsFailed => {
                let _ = self.connect(3);
                self.screen = Screen::Connections;
                self.surface = surface;
            }
            Surface::WorkbenchDefault
            | Surface::ExplorerFocused
            | Surface::QuickSwitcher
            | Surface::TabListPicker
            | Surface::SafeModePicker
            | Surface::HelpDialog => {
                self.reset_visual_workbench();
                if surface == Surface::ExplorerFocused
                    && let Some((index, node)) = self
                        .explorer_nodes
                        .iter()
                        .enumerate()
                        .find(|(_, node)| matches!(node, ExplorerNode::Object { .. }))
                {
                    self.explorer_tree_state
                        .set_cursor(index, explorer_node_key(node));
                }
                if surface == Surface::TabListPicker {
                    for _ in 0..12 {
                        self.workbench.new_query("SELECT 1");
                    }
                    self.sync_tabs_state();
                }
                if surface == Surface::SafeModePicker {
                    self.safe_mode = SafeMode::SafeFull;
                    self.connection.safe_mode = self.safe_mode;
                    self.workbench.connection.safe_mode = self.safe_mode;
                }
                self.surface = surface;
            }
            Surface::TableGrid
            | Surface::GridCellEditing
            | Surface::PendingChangeBar
            | Surface::StructureView
            | Surface::FilterEditor
            | Surface::MaximisedTab => self.set_table_surface(surface),
            Surface::QueryEditing | Surface::CompletionPopup => {
                self.reset_visual_workbench();
                self.set_visual_query(if surface == Surface::CompletionPopup {
                    "SELECT * FROM ord"
                } else {
                    "SELECT * FROM orders"
                });
                self.surface = surface;
            }
            Surface::ResultsGrid => {
                self.reset_visual_workbench();
                self.set_visual_query("SELECT * FROM orders LIMIT 25");
                let _ = self.execute_query();
                self.surface = surface;
            }
            Surface::ErrorResult => {
                self.reset_visual_workbench();
                self.set_visual_query("SELECT nope FROM orders");
                let _ = self.execute_query();
                self.surface = surface;
            }
            Surface::ExplainPlan => {
                self.reset_visual_workbench();
                self.set_visual_query("SELECT * FROM orders LIMIT 10");
                if let Some(Tab::Query(query)) = self.workbench.active_mut() {
                    let _ = query.explain(&self.catalog);
                }
                "Explain plan ready".clone_into(&mut self.status);
                self.surface = surface;
            }
            Surface::HistoryTab => {
                self.reset_visual_workbench();
                self.workbench.open_history();
                self.sync_active_tab();
                self.surface = surface;
            }
            Surface::SafetyDialogTypedAck => {
                self.reset_visual_workbench();
                self.safe_mode = SafeMode::Safe;
                self.connection.safe_mode = self.safe_mode;
                self.workbench.connection.safe_mode = self.safe_mode;
                self.set_visual_query("DELETE FROM orders");
                let _ = self.execute_query();
                self.surface = surface;
            }
        }
    }

    fn set_table_surface(&mut self, surface: Surface) {
        self.reset_visual_workbench();
        let _ = self.workbench.open_table("orders");
        self.sync_active_table();
        self.sync_tabs_state();
        if surface == Surface::GridCellEditing || surface == Surface::PendingChangeBar {
            if let Some(tab) = self.workbench.active_table_mut() {
                let _ = tab.result.commit_cell(0, 6, "EUR");
            }
            self.sync_active_table();
        }
        if surface == Surface::StructureView {
            let _ = self.workbench.toggle_structure();
            self.sync_active_table();
        }
        if surface == Surface::MaximisedTab {
            self.workbench.maximized = true;
        }
        self.surface = surface;
    }

    fn reset_visual_workbench(&mut self) {
        if let Some(index) = self
            .connections
            .iter()
            .position(|connection| connection.name == "Production")
        {
            let _ = self.connect(index);
        }
    }

    fn connections_panel<'a>(title: &'a str, meta: Option<&'a str>) -> Panel<'a> {
        let panel = Panel::new(CONNECTIONS_PANEL)
            .kind(PanelKind::Framed)
            .title(title)
            .focused(true)
            .patch_part(&FRAMED_PANEL_PATCH)
            .slot(Part::GUTTER, &preserve_frame_gutter);
        match meta {
            Some(meta) => panel.meta(meta),
            None => panel,
        }
    }

    fn connection_details_panel(title: &str) -> Panel<'_> {
        Panel::new(CONNECTION_DETAILS)
            .kind(PanelKind::Card)
            .title(title)
            .patch_part(&CONNECTION_DETAILS_TITLE_PATCH)
            .focused(false)
    }

    fn explorer_panel() -> Panel<'static> {
        Panel::new(EXPLORER_PANEL)
            .kind(PanelKind::Framed)
            .title(" Explorer ")
            .meta("public ")
            .focused(true)
            .patch_part(&FRAMED_PANEL_PATCH)
            .slot(Part::GUTTER, &preserve_frame_gutter)
    }

    fn content_panel<'a>(title: &'a str, meta: Option<&'a str>) -> Panel<'a> {
        let panel = Panel::new(CONTENT_FRAME)
            .kind(PanelKind::Framed)
            .title(title)
            .focused(true)
            .patch_part(&FRAMED_PANEL_PATCH)
            .slot(Part::GUTTER, &preserve_frame_gutter);
        match meta {
            Some(meta) => panel.meta(meta),
            None => panel,
        }
    }

    fn rebuild_connection_nodes(&mut self) {
        self.connection_nodes = build_connection_nodes(&self.connections);
        self.connection_tree_state = initial_connection_tree_state(&self.connection_nodes);
        self.connection_visual_tree_state =
            initial_connection_visual_tree_state(&self.connection_nodes);
    }

    fn sync_connection_selection(&mut self) {
        let Some(cursor) = self.connection_tree_state.cursor() else {
            return;
        };
        if let Some(ConnectionNode::Connection { index, .. }) = self
            .connection_nodes
            .iter()
            .find(|node| connection_node_key(node) == cursor)
        {
            self.connections_screen.selected = *index;
        }
    }

    fn set_visual_query(&mut self, query: &str) {
        query.clone_into(&mut self.query);
        self.query_state = TextInputState::default();
        self.sync_query_tab();
    }
    /// Borrow the connected workbench.
    pub const fn workbench(&self) -> &Workbench {
        &self.workbench
    }
    /// Whether the public connection form is open.
    pub const fn connection_form_open(&self) -> bool {
        self.form_open
    }
    /// Borrow the draft without exposing a password string.
    pub const fn connection_draft(&self) -> Option<&ConnectionDraft> {
        self.draft.as_ref()
    }
    /// Close the form (kept small so deterministic tests can model Esc).
    pub fn form_open_for_test(&mut self, open: bool) {
        self.form_open = open;
        if !open {
            self.draft = None;
        }
    }
    /// Open the connection form with the active connection as its draft.
    pub fn begin_connection_form(&mut self) {
        self.draft = Some(ConnectionDraft::from_connection(&self.connection));
        self.form_state = FormState::default();
        self.form_open = true;
        self.surface = Surface::Connections;
    }
    /// Select a connection and open its workbench.
    pub fn connect(&mut self, index: usize) -> bool {
        let Some(connection) = self.connections.get(index).cloned() else {
            return false;
        };
        self.connections_screen.selected = index;
        if connection.outcome != ConnectOutcome::Ok {
            self.status = format!("Connection failed: {}", connection.name);
            self.connections_screen.error = Some("Connection failed; press r to retry".to_owned());
            self.surface = Surface::ConnectionsFailed;
            return false;
        }
        self.safe_mode = connection.safe_mode;
        self.connection = connection.clone();
        self.connections_screen.selected = index;
        self.connections_screen.error = None;
        self.workbench = Workbench::new(connection.clone(), self.catalog.clone());
        self.workbench.new_query("");
        self.explorer_tree_state = initial_explorer_tree_state(&self.explorer_nodes);
        self.tabs_state = TabsState::default();
        self.split_state = SplitPaneState::default();
        self.sync_tabs_state();
        self.query.clear();
        self.query_state = TextInputState::default();
        self.columns.clear();
        self.result = ResultGrid::empty();
        self.grid_state = GridState::default();
        self.screen = Screen::Workbench;
        self.surface = Surface::WorkbenchDefault;
        self.status = format!("Connected to {}", connection.name);
        true
    }

    fn sync_active_table(&mut self) {
        let Some(Tab::Table(tab)) = self.workbench.active() else {
            self.columns.clear();
            self.result = ResultGrid::empty();
            self.grid_state = GridState::default();
            return;
        };
        self.columns = if tab.is_structure() {
            tab.structure_columns()
        } else {
            tab.table
                .columns
                .iter()
                .map(|column| (column.name.clone(), column.ty))
                .collect()
        };
        self.result = if tab.is_structure() {
            structure_grid(tab)
        } else {
            tab.result.clone()
        };
        self.grid_state = GridState::default();
    }

    fn sync_tabs_state(&mut self) {
        let active = self.workbench.active;
        if let Some(key) = self.workbench.tabs.get(active).map(tab_key) {
            self.tabs_state.set_active(active, key);
        } else {
            self.tabs_state = TabsState::default();
        }
    }

    fn open_table(&mut self, item: &ExplorerItem) -> bool {
        let opened = self.workbench.open_explorer_item(item);
        if opened {
            self.sync_active_table();
            self.sync_tabs_state();
            self.surface = Surface::TableGrid;
        }
        opened
    }

    fn new_query(&mut self, query: impl Into<String>) {
        self.workbench.new_query(query);
        self.sync_tabs_state();
        self.query.clear();
        self.query_state = TextInputState::default();
        self.columns.clear();
        self.result = ResultGrid::empty();
        self.grid_state = GridState::default();
        self.surface = Surface::QueryEditing;
    }

    fn sync_query_tab(&mut self) {
        let query = self.query.clone();
        if let Some(Tab::Query(tab)) = self.workbench.active_mut() {
            tab.query = query;
        }
    }

    fn commit_query_edit(&mut self) {
        let _ = self
            .query_state
            .commit(&mut self.query, &junie_tui::NoValidate);
        self.sync_query_tab();
    }

    fn sync_active_tab(&mut self) {
        match self.workbench.active() {
            Some(Tab::Table(tab)) => {
                self.columns = if tab.is_structure() {
                    tab.structure_columns()
                } else {
                    tab.table
                        .columns
                        .iter()
                        .map(|column| (column.name.clone(), column.ty))
                        .collect()
                };
                self.result = if tab.is_structure() {
                    structure_grid(tab)
                } else {
                    tab.result.clone()
                };
                self.surface = if tab.is_structure() {
                    Surface::StructureView
                } else {
                    Surface::TableGrid
                };
            }
            Some(Tab::Query(tab)) => {
                self.query.clone_from(&tab.query);
                self.query_state = TextInputState::default();
                self.columns.clear();
                self.result = tab.result.clone().unwrap_or_else(ResultGrid::empty);
                self.surface = Surface::QueryEditing;
            }
            Some(Tab::History(_)) => {
                self.columns.clear();
                self.result = ResultGrid::empty();
                self.surface = Surface::HistoryTab;
            }
            None => {
                self.columns.clear();
                self.result = ResultGrid::empty();
            }
        }
        self.grid_state = GridState::default();
        self.sync_tabs_state();
    }
    /// Change the active safe-mode policy.
    pub fn set_safe_mode(&mut self, mode: SafeMode) {
        self.safe_mode = mode;
        self.connection.safe_mode = mode;
        self.workbench.connection.safe_mode = mode;
        self.surface = Surface::SafeModePicker;
    }
    /// Run a query through the same parser, gate and executor as Ctrl+R.
    pub fn run_query(&mut self, query: impl Into<String>) -> QueryOutcome {
        self.query = query.into();
        self.query_state = TextInputState::default();
        self.sync_query_tab();
        self.execute_query()
    }
    /// Parse, gate and execute the current query.
    pub fn execute_query(&mut self) -> QueryOutcome {
        let statement = match crate::sql::parse(self.query.trim()) {
            Ok(statement) => statement,
            Err(error) => {
                let out = QueryOutcome::Rejected {
                    message: error.message,
                };
                self.status = outcome_message(&out);
                return out;
            }
        };
        let table = match &statement {
            crate::sql::Statement::Select(select) => {
                self.catalog.find(select.schema.as_deref(), &select.table)
            }
            _ => None,
        };
        match crate::sql::gate(self.safe_mode, &statement) {
            crate::sql::Decision::Deny => {
                let risk = crate::sql::assess(&statement, table);
                let out = QueryOutcome::Denied {
                    summary: format!("{} is denied in Read-Only mode", risk.action),
                };
                self.status = outcome_message(&out);
                out
            }
            crate::sql::Decision::Confirm { deliberate } => {
                let risk = crate::sql::assess(&statement, table);
                let out = QueryOutcome::ConfirmationRequired {
                    deliberate,
                    summary: format!("{} · {}", risk.action, risk.scope),
                };
                self.status = outcome_message(&out);
                out
            }
            crate::sql::Decision::Run => {
                if let crate::sql::Statement::Select(select) = statement {
                    match crate::sql::run_select(&self.catalog, &select) {
                        Ok(result) => {
                            let out = QueryOutcome::Executed {
                                rows: result.rows.len(),
                                editable: result.editable,
                            };
                            self.columns.clone_from(&result.columns);
                            self.result = ResultGrid::from_result(&result);
                            if let Some(Tab::Query(tab)) = self.workbench.active_mut() {
                                tab.result = Some(self.result.clone());
                            }
                            self.grid_state = GridState::default();
                            self.status = outcome_message(&out);
                            out
                        }
                        Err(error) => {
                            let out = QueryOutcome::Rejected {
                                message: error.message,
                            };
                            self.status = outcome_message(&out);
                            out
                        }
                    }
                } else {
                    let out = QueryOutcome::Rejected {
                        message: "The demo executor only runs SELECT statements".to_owned(),
                    };
                    self.status = outcome_message(&out);
                    out
                }
            }
        }
    }
    fn column_specs(
        columns: &[(String, ColType)],
        editable: bool,
    ) -> ([junie_tui::Column<'_>; junie_tui::GRID_MAX_COLUMNS], usize) {
        let count = columns.len().min(junie_tui::GRID_MAX_COLUMNS);
        let mut specs =
            [junie_tui::Column::new(junie_tui::ColumnKey::num(0), ""); junie_tui::GRID_MAX_COLUMNS];
        for (index, (name, _)) in columns.iter().take(count).enumerate() {
            let mut col = junie_tui::Column::new(
                junie_tui::ColumnKey::num((index as u16).saturating_add(1)),
                name.as_str(),
            );
            col.sortable = true;
            col.editable = editable;
            col.sticky = index == 0;
            if let Some(slot) = specs.get_mut(index) {
                *slot = col;
            }
        }
        (specs, count)
    }
    fn connection_form<'a>(
        fields: &'a [junie_tui::FieldSpec<'a>],
        actions: &'a [Action<'a>],
    ) -> Form<'a> {
        Form::new(connections::FORM, fields)
            .actions(actions)
            .submit(connections::SAVE_CONNECT)
    }
    fn handle_grid(&mut self, action: &GridAction) {
        match action {
            GridAction::Sort(key, direction) => {
                self.result.sort(*key, *direction);
                self.status = format!("Sorted column {}", key.raw());
            }
            GridAction::Copy(text) => {
                self.status = format!("Copied {} cells", text.lines().count());
            }
            GridAction::Activated(key) => self.status = format!("Activated row {key:?}"),
            GridAction::EditRequested(key, column) => {
                self.status = format!("Edit requested for {key:?}, column {column:?}");
            }
            GridAction::CellAction(key, column, action) => {
                self.status = format!("Cell action {action:?} on {key:?}/{column:?}");
            }
            GridAction::FetchMore => {
                "All deterministic demo rows are loaded".clone_into(&mut self.status);
            }
            GridAction::Moved | GridAction::LeaveForward | GridAction::LeaveBackward => {}
        }
    }

    fn draw_result_grid(&self, ui: &mut Ui<'_>, area: junie_tui::Rect) {
        let (columns, column_count) = Self::column_specs(&self.columns, self.result.is_editable());
        let visible_columns = columns.get(..column_count).unwrap_or(&[]);
        result_grid(visible_columns).draw(ui, area, &self.grid_state, &self.result);
    }

    fn draw_connection_details(&self, ui: &mut Ui<'_>, area: junie_tui::Rect) {
        let index = self.connections_screen.selected;
        let Some(connection) = self.connections_screen.connections.get(index) else {
            return;
        };
        let port = (connection.port != 0).then(|| connection.port.to_string());
        let host = port.as_deref().map_or_else(
            || connection.host.clone(),
            |port| format!("{}:{port}", connection.host),
        );
        let ssl_ssh = format!(
            "{} / {}",
            if connection.ssl { "on" } else { "off" },
            if connection.ssh.is_some() {
                "on"
            } else {
                "off"
            }
        );
        let properties = [
            ("Engine", connection.engine.label().to_owned()),
            ("Host", host),
            ("Database", connection.database.clone()),
            (
                "User",
                if connection.user.is_empty() {
                    "—".to_owned()
                } else {
                    connection.user.clone()
                },
            ),
            ("Environment", connection.environment.label().to_owned()),
            (
                "Safe Mode",
                format!(
                    "{} · {}",
                    connection.safe_mode.label(),
                    connection.safe_mode.description()
                ),
            ),
            ("SSL / SSH", ssl_ssh),
            ("Last used", connection.last_used.clone()),
        ];
        let card_area = junie_tui::Rect {
            width: area.width.min(70),
            height: area.height.min(17),
            ..area
        };
        Self::connection_details_panel(&connection.name).draw(ui, card_area, |ui, area| {
            draw_connection_properties(ui, area, &properties);
        });
        if !ui.is_inert() {
            ui.register_control(CONNECTION_DETAILS, card_area, Focusability::ClickOnly);
        }
    }

    fn draw_connections(&self, ui: &mut Ui<'_>, area: junie_tui::Rect) {
        let count = format!("{} ", self.connections_screen.connections.len());
        let list_width = (area.width / 3).clamp(26, 40).min(area.width);
        let list_area = junie_tui::Rect {
            x: area.x,
            y: area.y,
            width: if area.width < 80 {
                area.width
            } else {
                list_width
            },
            height: area.height,
        };
        let panel = Self::connections_panel(" Connections ", Some(&count));
        let inner = panel.inner(ui, list_area);
        let body = legacy_tree_body(inner);
        panel.draw(ui, list_area, |_, _| {});
        paint_frame_title_tail(ui, list_area, " Connections ");
        paint_panel_tail(ui, list_area);
        ui.with_area(body, |ui| {
            let filter = junie_tui::Rect {
                x: body.x,
                y: inner.y.saturating_add(1),
                width: body.width,
                height: 1.min(inner.height),
            };
            paint_legacy_filter(ui, filter, "Filter connections");
            let tree_area = junie_tui::Rect {
                y: body.y.saturating_add(2),
                height: inner.height.saturating_sub(2),
                ..body
            };
            connection_tree().draw(
                ui,
                tree_area,
                &self.connection_visual_tree_state,
                &self.connection_nodes,
            );
            paint_legacy_tree_gutters(
                ui,
                tree_area,
                &self.connection_visual_tree_state,
                &self.connection_nodes,
                connection_node,
                connection_node_key,
            );
        });
        let mut blank = ui.surface_style();
        blank.fg = Some(
            ui.theme()
                .color
                .fg
                .get(FgStep::Secondary.index())
                .copied()
                .unwrap_or_default(),
        );
        ui.fill(
            junie_tui::Rect {
                x: body.x.saturating_add(2),
                y: body.y,
                width: body.width.saturating_sub(1),
                height: 1,
            },
            blank,
        );
        let mut field = ui.surface_style();
        field.bg = Some(ui.theme().color.field);
        ui.fill(
            junie_tui::Rect {
                x: body.right(),
                y: inner.y.saturating_add(1),
                width: 1,
                height: 1,
            },
            field,
        );
        if self.connection_visual_tree_state.cursor()
            == self.connection_nodes.first().map(connection_node_key)
        {
            ui.fill(
                junie_tui::Rect {
                    x: body.right(),
                    y: body.y.saturating_add(2),
                    width: 1,
                    height: 1,
                },
                {
                    let mut style = ui.surface_style();
                    style.add_modifier |= Modifier::BOLD;
                    style
                },
            );
        }
        if area.width >= 80 {
            let details = junie_tui::Rect {
                x: area.x.saturating_add(list_width).saturating_add(2),
                y: area.y,
                width: area.width.saturating_sub(list_width).saturating_sub(2),
                height: area.height,
            };
            self.draw_connection_details(ui, details);
        }
    }

    fn draw_explorer(&self, ui: &mut Ui<'_>, area: junie_tui::Rect) {
        let panel = Self::explorer_panel();
        let inner = panel.inner(ui, area);
        let body = legacy_tree_body(inner);
        panel.draw(ui, area, |_, _| {});
        paint_frame_title_tail(ui, area, " Explorer ");
        paint_panel_tail(ui, area);
        ui.with_area(body, |ui| {
            let filter = junie_tui::Rect {
                x: body.x,
                y: inner.y.saturating_add(1),
                width: body.width,
                height: 1.min(inner.height),
            };
            paint_legacy_filter(ui, filter, "Filter objects");
            let tree_area = junie_tui::Rect {
                y: body.y.saturating_add(2),
                height: inner.height.saturating_sub(2),
                ..body
            };
            explorer_tree().draw(
                ui,
                tree_area,
                &self.explorer_tree_state,
                &self.explorer_nodes,
            );
            paint_legacy_tree_gutters(
                ui,
                tree_area,
                &self.explorer_tree_state,
                &self.explorer_nodes,
                explorer_node,
                explorer_node_key,
            );
        });
    }

    fn draw_content(&self, ui: &mut Ui<'_>, area: junie_tui::Rect) {
        let (title, meta) = match self.workbench.active() {
            Some(Tab::Table(table)) => (
                format!(" public › {}", table.table.name),
                Some(format!("{} cols ", table.table.columns.len())),
            ),
            Some(Tab::Query(query)) => (format!(" {}", query.name), None),
            Some(Tab::History(_)) => (
                " Query history".to_owned(),
                Some(format!("{} entries ", self.workbench.history.entries.len())),
            ),
            None => (" Workbench".to_owned(), None),
        };
        let panel = Self::content_panel(&title, meta.as_deref());
        panel.draw(ui, area, |ui, inner| match self.workbench.active() {
            Some(Tab::Query(query)) => {
                let rows = fixed_flex_pair(inner, 3);
                Field::new("SQL query", query_input(Some(&self.query)))
                    .plain(true)
                    .draw(ui, rows[0], &self.query_state);
                if query.plan.is_some() {
                    ui.paint_str(rows[1], "Explain plan ready", ui.surface_style());
                } else if query.error.is_some() {
                    ui.paint_str(rows[1], "Query error", ui.surface_style());
                } else if query.result.is_some() {
                    self.draw_result_grid(ui, rows[1]);
                } else {
                    let message = junie_tui::Rect {
                        x: rows[1].x,
                        y: rows[1].y.saturating_add(rows[1].height / 2),
                        width: rows[1].width,
                        height: 1,
                    };
                    ui.paint_str(message, "No results yet", ui.surface_style());
                }
            }
            Some(Tab::Table(table)) => {
                let rows = fixed_flex_pair(inner, 2);
                let mode = if table.is_structure() {
                    "▎ Data    Structure"
                } else {
                    "Data    ▎ Structure"
                };
                ui.paint_str(rows[0], mode, ui.surface_style());
                self.draw_result_grid(ui, rows[1]);
            }
            Some(Tab::History(history)) => {
                ui.paint_str(inner, "Query history", ui.surface_style());
                for (offset, entry) in history.entries.iter().take(6).enumerate() {
                    let row = junie_tui::Rect {
                        y: inner.y.saturating_add(1).saturating_add(offset as u16),
                        height: 1,
                        ..inner
                    };
                    ui.paint_str(row, &entry.sql, ui.surface_style());
                }
            }
            None => {
                ui.paint_str(inner, "No tab open", ui.surface_style());
            }
        });
        paint_frame_title_tail(ui, area, &title);
        if meta.is_some() {
            paint_panel_tail(ui, area);
        }
    }
}

/// Result of executing a query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryOutcome {
    /// Query returned rows.
    Executed {
        /// Number of returned rows.
        rows: usize,
        /// Whether the result supports cell edits.
        editable: bool,
    },
    /// Safety gate requires confirmation.
    ConfirmationRequired {
        /// Whether the acknowledgement must name the target.
        deliberate: bool,
        /// Human-readable safety summary.
        summary: String,
    },
    /// Read-only policy denied a write.
    Denied {
        /// Human-readable denial reason.
        summary: String,
    },
    /// Parse/execution rejection.
    Rejected {
        /// Parser or executor message.
        message: String,
    },
}

fn outcome_message(outcome: &QueryOutcome) -> String {
    match outcome {
        QueryOutcome::Executed { rows, editable } => format!(
            "Loaded {rows} rows{}",
            if *editable {
                " · editable"
            } else {
                " · read-only"
            }
        ),
        QueryOutcome::ConfirmationRequired {
            deliberate,
            summary,
        } => format!(
            "Confirmation required{}: {summary}",
            if *deliberate { " · type target" } else { "" }
        ),
        QueryOutcome::Denied { summary } => summary.clone(),
        QueryOutcome::Rejected { message } => format!("Query rejected: {message}"),
    }
}

fn query_input(value: Option<&str>) -> TextInput<'_> {
    let input =
        TextInput::new(QUERY).placeholder("Type SQL. Ctrl+R runs the statement under the cursor.");
    match value {
        Some(text) => input.value(text),
        None => input,
    }
}

fn fixed_flex_pair(area: junie_tui::Rect, first_height: u16) -> [junie_tui::Rect; 2] {
    let first = first_height.min(area.height);
    [
        junie_tui::Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: first,
        },
        junie_tui::Rect {
            x: area.x,
            y: area.y.saturating_add(first),
            width: area.width,
            height: area.height.saturating_sub(first),
        },
    ]
}

fn structure_grid(tab: &TableTab) -> ResultGrid {
    let rows = tab.structure();
    let result = crate::sql::ResultSet {
        columns: tab.structure_columns(),
        total: rows.len(),
        rows,
        source: Some(tab.table.qualified()),
        duration_ms: 0,
        editable: false,
    };
    ResultGrid::from_result(&result)
}

fn stable_key(parts: &[&str]) -> ItemKey {
    let hash = parts.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, part| {
        let hash = (hash ^ 0xff).wrapping_mul(0x0000_0100_0000_01b3);
        part.as_bytes().iter().fold(hash, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
    });
    ItemKey::pair(hash, 0)
}

fn build_connection_nodes(connections: &[Connection]) -> Vec<ConnectionNode> {
    let mut nodes = Vec::with_capacity(connections.len().saturating_mul(2));
    let mut groups: Vec<&str> = Vec::new();
    for (index, connection) in connections.iter().enumerate() {
        if !groups.contains(&connection.group.as_str()) {
            groups.push(&connection.group);
            nodes.push(ConnectionNode::Group {
                name: connection.group.clone(),
            });
        }
        nodes.push(ConnectionNode::Connection {
            index,
            connection: connection.clone(),
        });
    }
    nodes
}

fn initial_connection_tree_state(nodes: &[ConnectionNode]) -> TreeState {
    let mut state = TreeState::default();
    state.expand_all();
    if let Some((index, node)) = nodes
        .iter()
        .enumerate()
        .find(|(_, node)| matches!(node, ConnectionNode::Connection { .. }))
    {
        state.set_cursor(index, connection_node_key(node));
    }
    state
}

fn initial_connection_visual_tree_state(nodes: &[ConnectionNode]) -> TreeState {
    let mut state = TreeState::default();
    state.expand_all();
    if let Some((index, node)) = nodes
        .iter()
        .enumerate()
        .find(|(_, node)| matches!(node, ConnectionNode::Group { .. }))
    {
        state.set_cursor(index, connection_node_key(node));
    }
    state
}

fn connection_node_key(node: &ConnectionNode) -> ItemKey {
    match node {
        ConnectionNode::Group { name } => stable_key(&["connection-group", name]),
        ConnectionNode::Connection { connection, .. } => {
            stable_key(&["connection", &connection.group, &connection.name])
        }
    }
}

fn connection_node(node: &ConnectionNode) -> TreeNode {
    match node {
        ConnectionNode::Group { .. } => TreeNode::parent(0).keyed(connection_node_key(node)),
        ConnectionNode::Connection { .. } => TreeNode::leaf(1).keyed(connection_node_key(node)),
    }
}

fn connection_row(node: &ConnectionNode, row: &mut RowUi<'_>) {
    match node {
        ConnectionNode::Group { name } => row.label(name),
        ConnectionNode::Connection { connection, .. } => {
            let glyph = match connection.environment {
                Environment::Production => '◆',
                Environment::Staging => '◇',
                Environment::Local | Environment::Development => '·',
            };
            let glyph = match glyph {
                '◆' => Span::new("◆"),
                '◇' => Span::new("◇"),
                _ => Span::new("·"),
            }
            .role(Role::Fg(FgStep::Muted));
            row.label_spans(&[glyph, Span::new(" "), Span::new(&connection.name)]);
            row.meta(connection.engine.short());
        }
    }
}

fn build_explorer_nodes(catalog: &Catalog) -> Vec<ExplorerNode> {
    let mut nodes = Vec::with_capacity(catalog.tables.len().saturating_add(8));
    nodes.push(ExplorerNode::Database {
        name: catalog.database.clone(),
    });
    for schema in &catalog.schemas {
        nodes.push(ExplorerNode::Schema {
            name: schema.clone(),
        });
        for (kind, label, prefix) in [
            (ObjectKind::Table, "Tables", "T"),
            (ObjectKind::View, "Views", "V"),
            (ObjectKind::Function, "Functions", "ƒ"),
            (ObjectKind::Sequence, "Sequences", "S"),
        ] {
            let objects = catalog
                .tables
                .iter()
                .filter(|table| table.schema == *schema && table.kind == kind)
                .map(|table| ExplorerItem {
                    schema: table.schema.clone(),
                    name: table.name.clone(),
                    kind: table.kind,
                    rows: table.row_count,
                });
            let objects: Vec<_> = objects.collect();
            if objects.is_empty() {
                continue;
            }
            nodes.push(ExplorerNode::Group {
                schema: schema.clone(),
                name: label.to_owned(),
            });
            nodes.extend(
                objects
                    .into_iter()
                    .map(|item| ExplorerNode::Object { item, prefix }),
            );
        }
    }
    nodes
}

fn initial_explorer_tree_state(nodes: &[ExplorerNode]) -> TreeState {
    let mut state = TreeState::default();
    for node in nodes.iter().filter(|node| {
        matches!(node, ExplorerNode::Database { .. })
            || matches!(node, ExplorerNode::Schema { name } if name == "public")
            || matches!(node, ExplorerNode::Group { name, .. } if name == "Tables")
    }) {
        state.expand(explorer_node_key(node));
    }
    if let Some((index, node)) = nodes
        .iter()
        .enumerate()
        .find(|(_, node)| matches!(node, ExplorerNode::Schema { name } if name == "public"))
    {
        state.set_cursor(index, explorer_node_key(node));
    }
    state
}

fn explorer_node_key(node: &ExplorerNode) -> ItemKey {
    match node {
        ExplorerNode::Database { name } => stable_key(&["database", name]),
        ExplorerNode::Schema { name } => stable_key(&["schema", name]),
        ExplorerNode::Group { schema, name } => stable_key(&["object-group", schema, name]),
        ExplorerNode::Object { item, prefix } => {
            stable_key(&["object", &item.schema, prefix, &item.name])
        }
    }
}

fn explorer_node(node: &ExplorerNode) -> TreeNode {
    match node {
        ExplorerNode::Database { .. } => TreeNode::parent(0).keyed(explorer_node_key(node)),
        ExplorerNode::Schema { .. } => TreeNode::parent(1).keyed(explorer_node_key(node)),
        ExplorerNode::Group { .. } => TreeNode::parent(2).keyed(explorer_node_key(node)),
        ExplorerNode::Object { .. } => TreeNode::leaf(3).keyed(explorer_node_key(node)),
    }
}

fn compact_count(rows: usize) -> String {
    if rows >= 1_000_000 {
        format!("{:.1}m", rows as f64 / 1_000_000.0)
    } else if rows >= 1_000 {
        format!("{:.1}k", rows as f64 / 1_000.0)
    } else {
        rows.to_string()
    }
}

fn explorer_row(node: &ExplorerNode, row: &mut RowUi<'_>) {
    match node {
        ExplorerNode::Database { name } => row.label_fmt(format_args!("▣ {name}")),
        ExplorerNode::Schema { name } => row.label_fmt(format_args!("▾ {name}")),
        ExplorerNode::Group { name, .. } => row.label(name),
        ExplorerNode::Object { item, prefix } => {
            row.label_spans(&[
                Span::new(prefix).role(Role::Fg(FgStep::Muted)),
                Span::new(" "),
                Span::new(&item.name),
            ]);
            let count = compact_count(item.rows);
            row.meta(&count);
        }
    }
}

fn tab_key(tab: &Tab) -> ItemKey {
    ItemKey::num(tab.key().get())
}

fn tab_row(tab: &Tab, row: &mut RowUi<'_>) {
    match tab {
        Tab::Table(table) => {
            row.label_fmt(format_args!("T {}", table.table.name));
        }
        Tab::Query(query) => {
            row.label_fmt(format_args!("≡ {}", query.name));
        }
        Tab::History(_) => row.label("H History"),
    }
}

fn connection_tree() -> Tree<
    'static,
    ConnectionNode,
    impl Fn(&ConnectionNode) -> ItemKey,
    impl Fn(&ConnectionNode, &mut RowUi<'_>),
> {
    Tree::new(CONNECTIONS)
        .key(connection_node_key)
        .node(&connection_node)
        .row(connection_row)
}

fn explorer_tree() -> Tree<
    'static,
    ExplorerNode,
    impl Fn(&ExplorerNode) -> ItemKey,
    impl Fn(&ExplorerNode, &mut RowUi<'_>),
> {
    Tree::new(EXPLORER)
        .key(explorer_node_key)
        .node(&explorer_node)
        .row(explorer_row)
}

fn tab_strip() -> Tabs<'static, Tab, impl Fn(&Tab) -> ItemKey, impl Fn(&Tab, &mut RowUi<'_>)> {
    Tabs::new(TAB_STRIP)
        .key(tab_key)
        .row(tab_row)
        .allow_new(true)
        .closable(true)
}

fn legacy_tree_body(inner: junie_tui::Rect) -> junie_tui::Rect {
    junie_tui::Rect {
        x: inner.x.saturating_sub(1),
        width: inner.width.saturating_add(1),
        ..inner
    }
}

fn paint_legacy_tree_gutters<T>(
    ui: &mut Ui<'_>,
    area: junie_tui::Rect,
    state: &TreeState,
    nodes: &[T],
    node: impl Fn(&T) -> TreeNode,
    key: impl Fn(&T) -> ItemKey,
) {
    if area.is_empty() {
        return;
    }
    let mut ancestors_expanded = Vec::new();
    let mut visible_row = 0usize;
    let first_visible = state.scroll().offset();
    let cursor = state.cursor();
    let mut gutter_style = ui.surface_style();
    gutter_style.fg = Some(ui.theme().color.on_surface_inverse);

    for item in nodes {
        let descriptor = node(item);
        let depth = usize::from(descriptor.depth());
        ancestors_expanded.truncate(depth);
        let visible = ancestors_expanded.iter().all(|expanded| *expanded);
        if visible {
            let row = visible_row.saturating_sub(first_visible);
            if visible_row >= first_visible
                && row < usize::from(area.height)
                && cursor != Some(key(item))
            {
                ui.paint_str(
                    junie_tui::Rect {
                        x: area.x,
                        y: area.y.saturating_add(row as u16),
                        width: 1,
                        height: 1,
                    },
                    "▎",
                    gutter_style,
                );
            }
            if matches!(descriptor.kind(), NodeKind::Leaf)
                && visible_row >= first_visible
                && row < usize::from(area.height)
            {
                ui.fill(
                    junie_tui::Rect {
                        x: area
                            .x
                            .saturating_add(1)
                            .saturating_add(depth.saturating_mul(2) as u16),
                        y: area.y.saturating_add(row as u16),
                        width: 1,
                        height: 1,
                    },
                    ui.surface_style(),
                );
            }
            visible_row = visible_row.saturating_add(1);
        }
        if matches!(descriptor.kind(), NodeKind::Parent | NodeKind::Lazy) {
            ancestors_expanded.push(state.is_expanded(key(item)));
        }
    }
}

fn workbench_split() -> SplitPane<'static> {
    SplitPane::new(WORKBENCH_SPLIT, SplitAxis::Horizontal)
        .min_first(28)
        .min_second(20)
}
fn result_grid<'a>(columns: &'a [junie_tui::Column<'a>]) -> Grid<'a> {
    Grid::new(RESULTS, columns)
        .nav(junie_tui::NavUnit::Cell)
        .select_mode(junie_tui::SelectMode::Multi)
}

fn shell_parts(area: junie_tui::Rect) -> [junie_tui::Rect; 3] {
    let body_y = area.y.saturating_add(2);
    let body_height = area.height.saturating_sub(4);
    [
        junie_tui::Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1.min(area.height),
        },
        junie_tui::Rect {
            x: area.x.saturating_add(1),
            y: body_y,
            width: area.width.saturating_sub(2),
            height: body_height,
        },
        junie_tui::Rect {
            x: area.x,
            y: area.bottom().saturating_sub(1),
            width: area.width,
            height: 1.min(area.height),
        },
    ]
}

fn preserve_frame_gutter(_: &mut Ui<'_>, _: junie_tui::Rect) {}

fn paint_legacy_filter(ui: &mut Ui<'_>, area: junie_tui::Rect, text: &str) {
    if area.is_empty() {
        return;
    }
    let field_bg = ui.theme().color.field;
    let mut field = ui.surface_style();
    field.bg = Some(field_bg);
    ui.fill(area, field);

    let mut gutter = field;
    gutter.fg = Some(field_bg);
    ui.paint_str(
        junie_tui::Rect {
            width: 1.min(area.width),
            ..area
        },
        "▎",
        gutter,
    );

    if area.width > 2 {
        let mut label = field;
        label.fg = Some(
            ui.theme()
                .color
                .fg
                .get(FgStep::Muted.index())
                .copied()
                .unwrap_or_default(),
        );
        ui.paint_str(
            junie_tui::Rect {
                x: area.x.saturating_add(2),
                width: area.width.saturating_sub(2),
                ..area
            },
            text,
            label,
        );
    }
}

fn paint_panel_tail(ui: &mut Ui<'_>, area: junie_tui::Rect) {
    if area.width < 2 || area.height == 0 {
        return;
    }
    let mut style = ui.surface_style();
    style.fg = Some(ui.theme().color.border_strong);
    ui.paint_str(
        junie_tui::Rect {
            x: area.right().saturating_sub(2),
            y: area.y,
            width: 1,
            height: 1,
        },
        "─",
        style,
    );
}

fn paint_frame_title_tail(ui: &mut Ui<'_>, area: junie_tui::Rect, title: &str) {
    let x = area
        .x
        .saturating_add(2)
        .saturating_add(junie_tui::width(title));
    if x >= area.right().saturating_sub(1) || area.height == 0 {
        return;
    }
    let mut style = ui.surface_style();
    style.fg = Some(ui.theme().color.border_strong);
    ui.paint_str(
        junie_tui::Rect {
            x,
            y: area.y,
            width: 1,
            height: 1,
        },
        "─",
        style,
    );
}

fn draw_header(ui: &mut Ui<'_>, area: junie_tui::Rect, app: &TableProApp) {
    let base = ui.surface_style();
    ui.fill(area, base);
    if app.screen == Screen::Connections {
        let saved = format!("{} saved", app.connections_screen.connections.len());
        if app.surface == Surface::Connections {
            ui.paint_spans(
                area,
                &[
                    Span::new(" "),
                    Span::new("▪").role(Role::Accent),
                    Span::new("  "),
                    Span::new("TablePro").bold(),
                    Span::new("  "),
                    Span::new("Connections").role(Role::Fg(FgStep::Secondary)),
                    Span::new("  "),
                    Span::new(&saved).role(Role::Fg(FgStep::Muted)),
                ],
                base,
            );
        } else {
            let surface = format!(" · {}", app.surface.label());
            ui.paint_spans(
                area,
                &[
                    Span::new(" "),
                    Span::new("▪").role(Role::Accent),
                    Span::new("  "),
                    Span::new("TablePro").bold(),
                    Span::new("  "),
                    Span::new("Connections").role(Role::Fg(FgStep::Secondary)),
                    Span::new(&surface).role(Role::Fg(FgStep::Secondary)),
                    Span::new("  "),
                    Span::new(&saved).role(Role::Fg(FgStep::Muted)),
                ],
                base,
            );
        }
    } else {
        let glyph = match app.connection.environment {
            Environment::Production => "◆",
            Environment::Staging => "◇",
            Environment::Local | Environment::Development => "·",
        };
        let environment = app.connection.environment.label();
        let path = format!("{} › public", app.connection.database);
        ui.paint_spans(
            area,
            &[
                Span::new(" "),
                Span::new("▪").role(Role::Accent),
                Span::new("  "),
                Span::new("TablePro").bold(),
                Span::new("  "),
                Span::new(&app.connection.name).bold(),
                Span::new("  "),
                Span::new(glyph).bold(),
                Span::new(" "),
                Span::new(environment).bold(),
                Span::new("  "),
                Span::new(&path).role(Role::Fg(FgStep::Secondary)),
                Span::new("  "),
                Span::new(app.safe_mode.token()).bold(),
            ],
            base,
        );
    }

    let capability = ui.theme().capability.color.label();
    let dimensions = format!("{}×{}", area.width, ui.full().height);
    let right_width = junie_tui::width(capability)
        .saturating_add(3)
        .saturating_add(junie_tui::width(&dimensions))
        .saturating_add(2)
        .saturating_add(6);
    let right = junie_tui::Rect {
        x: area.right().saturating_sub(right_width).saturating_sub(1),
        width: right_width.saturating_add(1),
        ..area
    };
    ui.paint_spans(
        right,
        &[
            Span::new(capability).role(Role::BorderStrong),
            Span::new(" · ").role(Role::BorderStrong),
            Span::new(&dimensions).role(Role::BorderStrong),
            Span::new(" "),
            Span::new(" ? help ").role(Role::Fg(FgStep::Muted)),
        ],
        base,
    );
}

fn draw_footer(ui: &mut Ui<'_>, area: junie_tui::Rect, app: &TableProApp) {
    let base = ui.surface_style();
    ui.fill(area, base);
    let spans = if app.screen == Screen::Connections {
        vec![
            Span::new(" "),
            Span::new("↑ ↓").bold(),
            Span::new(" "),
            Span::new("Move").role(Role::Fg(FgStep::Muted)),
            Span::new("  "),
            Span::new("Enter").bold(),
            Span::new(" "),
            Span::new("Connect").role(Role::Fg(FgStep::Muted)),
            Span::new("  "),
            Span::new("/").bold(),
            Span::new(" "),
            Span::new("Filter").role(Role::Fg(FgStep::Muted)),
            Span::new("  "),
            Span::new("Ctrl+N").bold(),
            Span::new(" "),
            Span::new("New").role(Role::Fg(FgStep::Muted)),
            Span::new("  "),
            Span::new("Tab").bold(),
            Span::new(" "),
            Span::new("Next").role(Role::Fg(FgStep::Muted)),
        ]
    } else {
        match app.workbench.active() {
            Some(Tab::Table(_)) => vec![
                Span::new(" "),
                Span::new("↑ ↓←→").bold(),
                Span::new(" "),
                Span::new("Cell").role(Role::Fg(FgStep::Muted)),
                Span::new("  "),
                Span::new("Enter").bold(),
                Span::new(" "),
                Span::new("Edit").role(Role::Fg(FgStep::Muted)),
                Span::new("  s").bold(),
                Span::new(" "),
                Span::new("Sort").role(Role::Fg(FgStep::Muted)),
                Span::new("  f").bold(),
                Span::new(" "),
                Span::new("Filter").role(Role::Fg(FgStep::Muted)),
                Span::new("  Space").bold(),
                Span::new(" "),
                Span::new("Select row").role(Role::Fg(FgStep::Muted)),
                Span::new("  Tab").bold(),
                Span::new(" "),
                Span::new("Next").role(Role::Fg(FgStep::Muted)),
            ],
            Some(Tab::Query(_)) => vec![
                Span::new(" "),
                Span::new("Enter").bold(),
                Span::new(" "),
                Span::new("Edit").role(Role::Fg(FgStep::Muted)),
                Span::new("  Ctrl+R").bold(),
                Span::new(" "),
                Span::new("Run").role(Role::Fg(FgStep::Muted)),
                Span::new("  Alt+R").bold(),
                Span::new(" "),
                Span::new("Run all").role(Role::Fg(FgStep::Muted)),
                Span::new("  Ctrl+O").bold(),
                Span::new(" "),
                Span::new("Quick open").role(Role::Fg(FgStep::Muted)),
                Span::new("  Tab").bold(),
                Span::new(" "),
                Span::new("Next").role(Role::Fg(FgStep::Muted)),
            ],
            Some(Tab::History(_)) => vec![
                Span::new(" "),
                Span::new("↑ ↓").bold(),
                Span::new(" "),
                Span::new("Move").role(Role::Fg(FgStep::Muted)),
                Span::new("  Enter").bold(),
                Span::new(" "),
                Span::new("Open").role(Role::Fg(FgStep::Muted)),
                Span::new("  /").bold(),
                Span::new(" "),
                Span::new("Filter").role(Role::Fg(FgStep::Muted)),
                Span::new("  Tab").bold(),
                Span::new(" "),
                Span::new("Next").role(Role::Fg(FgStep::Muted)),
            ],
            None => vec![
                Span::new(" "),
                Span::new("Ctrl+N").bold(),
                Span::new(" "),
                Span::new("New query").role(Role::Fg(FgStep::Muted)),
                Span::new("  Ctrl+O").bold(),
                Span::new(" "),
                Span::new("Quick open").role(Role::Fg(FgStep::Muted)),
                Span::new("  Tab").bold(),
                Span::new(" "),
                Span::new("Next").role(Role::Fg(FgStep::Muted)),
            ],
        }
    };
    ui.paint_spans(area, &spans, base);
    if app.screen == Screen::Workbench {
        let right_text = format!("Connected to {}", app.connection.name);
        let right = junie_tui::Rect {
            x: area.right().saturating_sub(junie_tui::width(&right_text)),
            width: junie_tui::width(&right_text),
            ..area
        };
        ui.paint_spans(
            right,
            &[Span::new(&right_text).role(Role::Fg(FgStep::Muted))],
            base,
        );
    }
}

fn draw_too_small(ui: &mut Ui<'_>, area: junie_tui::Rect) {
    let style = ui.surface_style();
    let lines = [
        "TablePro".to_owned(),
        "Terminal too small".to_owned(),
        format!(
            "Need {}×{}, have {}×{}",
            MIN_WIDTH, MIN_HEIGHT, area.width, area.height
        ),
        String::new(),
        "q Quit".to_owned(),
    ];
    let start = area
        .y
        .saturating_add(area.height.saturating_sub(lines.len() as u16) / 2);
    for (offset, line) in lines.iter().enumerate() {
        let width = junie_tui::width(line);
        let row = junie_tui::Rect {
            x: area.x.saturating_add(area.width.saturating_sub(width) / 2),
            y: start.saturating_add(offset as u16),
            width,
            height: 1,
        };
        ui.paint_str(row, line, style);
    }
}

fn draw_connection_properties(
    ui: &mut Ui<'_>,
    area: junie_tui::Rect,
    properties: &[(&str, String)],
) {
    let value_x = area.x.saturating_add(13);
    let value_width = area.width.saturating_sub(18).max(1);
    let mut label_style = ui.surface_style();
    label_style.fg = Some(
        ui.theme()
            .color
            .fg
            .get(FgStep::Muted.index())
            .copied()
            .unwrap_or_default(),
    );
    let mut y = area.y;
    for (label, value) in properties {
        let value_step = match *label {
            "Environment" => FgStep::Faint,
            "Safe Mode" | "SSL / SSH" => FgStep::Secondary,
            "Last used" => FgStep::Muted,
            _ => FgStep::Primary,
        };
        let mut value_style = ui.surface_style();
        value_style.fg = Some(
            ui.theme()
                .color
                .fg
                .get(value_step.index())
                .copied()
                .unwrap_or_default(),
        );
        let lines = if *label == "Safe Mode" {
            wrap(value, value_width)
        } else {
            vec![value.clone()]
        };
        for (line_index, line) in lines.iter().enumerate() {
            if y >= area.bottom().saturating_sub(2) {
                break;
            }
            if line_index == 0 {
                ui.paint_str(
                    junie_tui::Rect {
                        x: area.x,
                        y,
                        width: 15.min(area.width),
                        height: 1,
                    },
                    label,
                    label_style,
                );
            }
            ui.paint_str(
                junie_tui::Rect {
                    x: value_x,
                    y,
                    width: value_width,
                    height: 1,
                },
                line,
                value_style,
            );
            y = y.saturating_add(1);
        }
    }
    let y = area.bottom().saturating_sub(1);
    let mut x = area.x;
    paint_action_button(
        ui,
        x,
        y,
        "Connect",
        ui.theme().color.accent,
        ui.theme().color.on_accent,
        true,
    );
    x = x.saturating_add(11);
    paint_action_button(
        ui,
        x,
        y,
        "Edit",
        ui.theme().color.surfaces[3],
        ui.theme()
            .color
            .fg
            .get(FgStep::Primary.index())
            .copied()
            .unwrap_or_default(),
        false,
    );
    x = x.saturating_add(8);
    paint_action_button(
        ui,
        x,
        y,
        "Duplicate",
        ui.theme().color.surfaces[1],
        ui.theme()
            .color
            .fg
            .get(FgStep::Secondary.index())
            .copied()
            .unwrap_or_default(),
        false,
    );
    x = x.saturating_add(13);
    paint_action_button(
        ui,
        x,
        y,
        "Delete…",
        ui.theme().color.surfaces[3],
        ui.theme().color.danger,
        false,
    );
}

fn paint_action_button(
    ui: &mut Ui<'_>,
    x: u16,
    y: u16,
    label: &str,
    background: Color,
    foreground: Color,
    bold: bool,
) {
    let width = junie_tui::width(label).saturating_add(2);
    let mut button = ui.surface_style();
    button.bg = Some(background);
    button.fg = Some(foreground);
    if bold {
        button.add_modifier |= Modifier::BOLD;
    }
    ui.fill(
        junie_tui::Rect {
            x,
            y,
            width,
            height: 1,
        },
        button,
    );
    let mut gutter = button;
    gutter.add_modifier.remove(Modifier::BOLD);
    gutter.fg = Some(background);
    ui.paint_str(
        junie_tui::Rect {
            x,
            y,
            width: 1,
            height: 1,
        },
        "▎",
        gutter,
    );
    ui.paint_str(
        junie_tui::Rect {
            x: x.saturating_add(1),
            y,
            width: width.saturating_sub(1),
            height: 1,
        },
        label,
        button,
    );
}

impl App for TableProApp {
    #[expect(
        clippy::too_many_lines,
        reason = "update keeps public component routing and product command arbitration in one phase"
    )]
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        // Stateless props have no update method, but their factories remain
        // the single source of configuration for both runtime phases.
        let _ = Self::connections_panel("", None);
        let _ = Self::connection_details_panel("");
        let _ = Self::explorer_panel();
        let _ = Self::content_panel("", None);
        if matches!(
            cx.update_cause(),
            UpdateCause::Bootstrap | UpdateCause::Event
        ) && let Some(command) = cx.command()
        {
            match command {
                c if c == QUIT => {
                    self.quit = true;
                    cx.quit();
                    response |= Response::consumed();
                }
                c if c == RUN => {
                    self.commit_query_edit();
                    let _ = self.execute_query();
                    response |= Response::changed();
                }
                c if c == OPEN => {
                    self.surface = Surface::QuickSwitcher;
                    response |= Response::changed();
                }
                c if c == NEW_QUERY => {
                    self.new_query("");
                    response |= Response::changed();
                }
                c if c == HISTORY => {
                    self.workbench.open_history();
                    self.sync_active_tab();
                    response |= Response::changed();
                }
                c if c == STRUCTURE => {
                    let _ = self.workbench.toggle_structure();
                    self.sync_active_tab();
                    response |= Response::changed();
                }
                c if c == FORM => {
                    self.begin_connection_form();
                    response |= Response::changed();
                }
                c if c == HELP => {
                    self.surface = Surface::HelpDialog;
                    response |= Response::changed();
                }
                _ => {}
            }
        }
        if self.form_open {
            let fields = &self.form_fields;
            let actions = &self.form_actions;
            if let Some(draft) = self.draft.as_mut() {
                let form = Self::connection_form(fields, actions);
                let form_response = form.update(cx, &mut self.form_state, draft);
                if let Some(action) = form_response.action_ref() {
                    match action {
                        FormAction::Action(ActionKey::CANCEL) => {
                            self.form_open = false;
                            self.draft = None;
                        }
                        FormAction::Action(ActionKey::SAVE | connections::SAVE_CONNECT) => {
                            if draft.validate_all().is_ok()
                                && let Some(connection) =
                                    draft.to_connection(Some(&self.connection))
                            {
                                self.connections.push(connection.clone());
                                self.connections_screen.connections.push(connection.clone());
                                self.rebuild_connection_nodes();
                                if action == &FormAction::Action(connections::SAVE_CONNECT) {
                                    let _ = self.connect(self.connections.len().saturating_sub(1));
                                    self.form_open = false;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                response |= form_response.erase();
            }
            return response;
        }
        if self.screen == Screen::Connections {
            let details_clicked = cx.intents(CONNECTION_DETAILS).any(|intent| {
                matches!(
                    intent,
                    Intent::Pointer {
                        phase: Phase::Click | Phase::DoubleClick,
                        ..
                    }
                )
            });
            if details_clicked {
                let _ = self.connect(self.connections_screen.selected);
                return response | Response::changed();
            }
            let tree_response = connection_tree().update(
                cx,
                &mut self.connection_tree_state,
                &self.connection_nodes,
            );
            self.sync_connection_selection();
            if tree_response.action_ref().is_some() {
                self.connection_visual_tree_state = self.connection_tree_state.clone();
            }
            if let Some(action) = tree_response.action_ref()
                && let TreeAction::Activated(key) | TreeAction::Chose(key) = action
                && let Some(ConnectionNode::Connection { index, .. }) = self
                    .connection_nodes
                    .iter()
                    .find(|node| connection_node_key(node) == *key)
            {
                let _ = self.connect(*index);
            }
            response |= tree_response.erase();
            return response;
        }

        let split = workbench_split();
        response |= split.update(cx, &mut self.split_state).erase();
        let tree_response =
            explorer_tree().update(cx, &mut self.explorer_tree_state, &self.explorer_nodes);
        if let Some(TreeAction::Activated(key) | TreeAction::Chose(key)) =
            tree_response.action_ref()
            && let Some(ExplorerNode::Object { item, .. }) = self
                .explorer_nodes
                .iter()
                .find(|node| explorer_node_key(node) == *key)
        {
            let item = item.clone();
            let _ = self.open_table(&item);
        }
        response |= tree_response.erase();

        let tabs_response = tab_strip().update(cx, &mut self.tabs_state, &self.workbench.tabs);
        if let Some(action) = tabs_response.action_ref() {
            match *action {
                TabsAction::Activated(key) => {
                    if let Some(index) = self
                        .workbench
                        .tabs
                        .iter()
                        .position(|tab| tab_key(tab) == key)
                    {
                        self.workbench.active = index;
                        self.sync_active_tab();
                    }
                }
                TabsAction::Close(key) => {
                    if let Some(index) = self
                        .workbench
                        .tabs
                        .iter()
                        .position(|tab| tab_key(tab) == key)
                    {
                        let _ = self.workbench.close_tab(index);
                        self.sync_active_tab();
                    }
                }
                TabsAction::New => self.new_query(""),
            }
        }
        response |= tabs_response.erase();

        if matches!(self.workbench.active(), Some(Tab::Query(_))) {
            response |= query_input(None)
                .update(cx, &mut self.query_state, &mut self.query)
                .erase();
            self.sync_query_tab();
        }
        let grid_route = matches!(self.workbench.active(), Some(Tab::Table(_)))
            || matches!(self.workbench.active(), Some(Tab::Query(tab)) if tab.result.is_some());
        if grid_route {
            let editable = self.result.is_editable();
            let (columns, column_count) = Self::column_specs(&self.columns, editable);
            let visible_columns = columns.get(..column_count).unwrap_or(&[]);
            let grid = result_grid(visible_columns);
            let grid_response = if self.result.is_editable() {
                grid.update_editable(cx, &mut self.grid_state, &mut self.result)
            } else {
                grid.update(cx, &mut self.grid_state, &self.result)
            };
            if let Some(action) = grid_response.action_ref() {
                self.handle_grid(action);
            }
            response |= grid_response.erase();
        }
        response
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        if full.width < MIN_WIDTH || full.height < MIN_HEIGHT {
            draw_too_small(ui, full);
            return;
        }
        ui.fill(full, ui.surface_style());
        let rows = shell_parts(full);
        draw_header(ui, rows[0], self);
        if self.form_open {
            Self::connections_panel(" Connect to database", None).draw(ui, rows[1], |ui, area| {
                if let Some(draft) = self.draft.as_ref() {
                    Self::connection_form(&self.form_fields, &self.form_actions).draw(
                        ui,
                        area,
                        &self.form_state,
                        draft,
                    );
                }
            });
        } else if self.screen == Screen::Connections {
            self.draw_connections(ui, rows[1]);
        } else {
            let workbench_rows = fixed_flex_pair(rows[1], 2);
            tab_strip().draw(
                ui,
                workbench_rows[0],
                &self.tabs_state,
                &self.workbench.tabs,
            );
            if self.workbench.maximized {
                self.draw_content(ui, workbench_rows[1]);
            } else {
                workbench_split().draw(
                    ui,
                    workbench_rows[1],
                    &self.split_state,
                    |ui, explorer_area, content_area| {
                        self.draw_explorer(ui, explorer_area);
                        self.draw_content(ui, content_area);
                    },
                );
            }
        }
        draw_footer(ui, rows[2], self);
    }
    fn should_quit(&self) -> bool {
        self.quit
    }
    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
    fn min_size(&self) -> Size {
        Size {
            min: (MIN_WIDTH, MIN_HEIGHT),
            preferred: (120, 36),
        }
    }
    fn on_esc(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        if self.form_open {
            self.form_open = false;
            self.draft = None;
            self.surface = Surface::Connections;
            Response::changed()
        } else {
            Response::ignored()
        }
    }
}

/// Start the interactive `TablePro` binary.
///
/// # Errors
///
/// Returns the terminal runtime's I/O error when the session cannot start
/// or restore the terminal.
pub fn run() -> std::io::Result<()> {
    let (theme, connect) = parse_args(std::env::args().skip(1))?;
    run_with(theme, connect.as_deref())
}

/// Start the app with an explicit theme and optional connection name.
///
/// # Errors
///
/// Returns an invalid-input error when the requested connection does not
/// exist, or the terminal runtime's I/O error when the session cannot start
/// or restore the terminal.
pub fn run_with(theme: Theme, connect: Option<&str>) -> std::io::Result<()> {
    let mut app = TableProApp::default();
    if let Some(name) = connect {
        let Some(index) = app
            .connections
            .iter()
            .position(|connection| connection.name.eq_ignore_ascii_case(name))
        else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("unknown connection: {name}"),
            ));
        };
        let _ = app.connect(index);
    }
    junie_tui::run(app, theme)
}

fn parse_args<I>(args: I) -> std::io::Result<(Theme, Option<String>)>
where
    I: IntoIterator<Item = String>,
{
    let mut theme = Theme::junie();
    let mut requested_color = None;
    let mut connect = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--theme" => {
                let value = args
                    .next()
                    .ok_or_else(|| invalid_arg("--theme requires a value"))?;
                theme = match value.to_ascii_lowercase().as_str() {
                    "junie" => Theme::junie(),
                    "paper" => Theme::paper(),
                    _ => return Err(invalid_arg("--theme must be junie or paper")),
                };
            }
            "--color" => {
                let value = args
                    .next()
                    .ok_or_else(|| invalid_arg("--color requires a value"))?;
                requested_color =
                    Some(parse_color_level(&value).ok_or_else(|| {
                        invalid_arg("--color must be truecolor, 256, 16, or none")
                    })?);
            }
            "--connect" => {
                connect = Some(
                    args.next()
                        .ok_or_else(|| invalid_arg("--connect requires a connection name"))?,
                );
            }
            "-h" | "--help" => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "usage: tablepro [--theme junie|paper] [--color truecolor|256|16|none] [--connect NAME]",
                ));
            }
            _ => return Err(invalid_arg("unknown option")),
        }
    }
    if let Some(level) = requested_color {
        theme = theme.for_level(level);
    }
    Ok((theme, connect))
}

fn invalid_arg(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
}

fn parse_color_level(value: &str) -> Option<ColorLevel> {
    match value.to_ascii_lowercase().as_str() {
        "truecolor" | "24bit" => Some(ColorLevel::TrueColor),
        "256" | "ansi256" => Some(ColorLevel::Ansi256),
        "16" | "ansi16" => Some(ColorLevel::Ansi16),
        "none" | "mono" => Some(ColorLevel::Mono),
        _ => None,
    }
}
