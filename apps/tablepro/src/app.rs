//! `TablePro` application shell built only on the public `junie-tui` facade.

use junie_tui::{
    Action, ActionKey, App, Chord, ColorLevel, Cx, Field, Form, FormAction, FormState, Grid,
    GridAction, GridState, Id, ItemKey, KeyCode, KeyMap, KeyModifiers, KeyPhase, List, ListAction,
    ListState, Panel, PanelKind, Response, RowUi, Size, SplitAxis, SplitPane, SplitPaneState,
    StatusBar, StatusItem, Tabs, TabsAction, TabsState, TextInput, TextInputState, Theme, Tree,
    TreeAction, TreeNode, TreeState, Ui, UpdateCause, Variant,
};

use crate::connections::{self, ConnectionDraft, ConnectionsScreen};
use crate::db::{self, Catalog, ColType, ConnectOutcome, Connection, SafeMode};
use crate::domain::ResultGrid;
use crate::tabs::{ExplorerItem, Tab, TableTab};
use crate::workbench::Workbench;

/// Minimum terminal width.
pub const MIN_WIDTH: u16 = 72;
/// Minimum terminal height.
pub const MIN_HEIGHT: u16 = 20;
const QUERY: Id = Id::root("tablepro.query");
const RESULTS: Id = Id::root("tablepro.results");
const STATUS: Id = Id::root("tablepro.status");
const HEADER: Id = Id::root("tablepro.header");
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
            Chord::with(KeyCode::Char('?'), KeyModifiers::NONE),
            HELP,
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
        environment: db::Environment::Local,
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
    connection_list_state: ListState,
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
            .field("connection_list_state", &"<list state>")
            .field("explorer_tree_state", &"<tree state>")
            .field("tabs_state", &"<tabs state>")
            .field("split_state", &"<split state>")
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
            connection_list_state: ListState::default(),
            explorer_tree_state: TreeState::default(),
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
    /// Set a named surface for capture/tests.
    pub const fn set_surface(&mut self, surface: Surface) {
        self.surface = surface;
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
        if connection.outcome != ConnectOutcome::Ok {
            self.status = format!("Connection failed: {}", connection.name);
            self.connections_screen.error = Some("Connection failed; press r to retry".to_owned());
            self.surface = Surface::ConnectionsFailed;
            return false;
        }
        self.safe_mode = connection.safe_mode;
        self.connection = connection.clone();
        self.connections_screen.error = None;
        self.workbench = Workbench::new(connection.clone(), self.catalog.clone());
        self.workbench.new_query("");
        self.explorer_tree_state = TreeState::default();
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
            specs[index] = col;
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
    let input = TextInput::new(QUERY).placeholder("SELECT …");
    match value {
        Some(text) => input.value(text),
        None => input,
    }
}

fn fixed_flex_rows(
    area: junie_tui::Rect,
    first_height: u16,
    last_height: u16,
) -> [junie_tui::Rect; 3] {
    let first = first_height.min(area.height);
    let remaining = area.height.saturating_sub(first);
    let last = last_height.min(remaining);
    let middle = remaining.saturating_sub(last);
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
            height: middle,
        },
        junie_tui::Rect {
            x: area.x,
            y: area.y.saturating_add(first).saturating_add(middle),
            width: area.width,
            height: last,
        },
    ]
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

fn connection_key(connection: &Connection) -> ItemKey {
    ItemKey::text(&connection.name)
}

fn connection_row(connection: &Connection, row: &mut RowUi<'_>) {
    row.label(&connection.name);
    row.meta(connection.environment.label());
}

fn explorer_key(item: &ExplorerItem) -> ItemKey {
    // Include schema: equal table names in different schemas are distinct
    // catalog objects and must not share collection state.
    ItemKey::pair(stable_text_key(&item.schema), stable_text_key(&item.name))
}

fn stable_text_key(value: &str) -> u64 {
    value
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

fn explorer_node(item: &ExplorerItem) -> TreeNode {
    TreeNode::leaf(0).keyed(explorer_key(item))
}

fn explorer_row(item: &ExplorerItem, row: &mut RowUi<'_>) {
    row.label(&item.name);
    row.meta(&item.schema);
}

fn tab_key(tab: &Tab) -> ItemKey {
    ItemKey::num(tab.key().get())
}

fn tab_row(tab: &Tab, row: &mut RowUi<'_>) {
    match tab {
        Tab::Table(table) => row.label(&table.table.name),
        Tab::Query(query) => row.label(&query.name),
        Tab::History(_) => row.label("History"),
    }
    if tab.dirty() {
        row.meta("*");
    }
}

fn connection_list()
-> List<'static, Connection, impl Fn(&Connection) -> ItemKey, impl Fn(&Connection, &mut RowUi<'_>)>
{
    List::new(CONNECTIONS)
        .key(connection_key)
        .row(connection_row)
}

fn explorer_tree() -> Tree<
    'static,
    ExplorerItem,
    impl Fn(&ExplorerItem) -> ItemKey,
    impl Fn(&ExplorerItem, &mut RowUi<'_>),
> {
    Tree::new(EXPLORER)
        .key(explorer_key)
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

fn header_panel() -> Panel<'static> {
    Panel::new(HEADER).kind(PanelKind::Framed)
}
fn connections_panel() -> Panel<'static> {
    Panel::new(CONNECTIONS_PANEL)
        .kind(PanelKind::Framed)
        .title("Connections")
}
fn results_panel() -> Panel<'static> {
    Panel::new(RESULTS).kind(PanelKind::Framed).title("Results")
}
fn status_bar<'a>(left: &'a [StatusItem<'a>], right: &'a [StatusItem<'a>]) -> StatusBar<'a> {
    StatusBar::new(STATUS)
        .left(left)
        .right(right)
        .variant(Variant::DEFAULT)
}
fn result_grid<'a>(columns: &'a [junie_tui::Column<'a>]) -> Grid<'a> {
    Grid::new(RESULTS, columns)
        .nav(junie_tui::NavUnit::Cell)
        .select_mode(junie_tui::SelectMode::Multi)
}

impl App for TableProApp {
    #[expect(
        clippy::too_many_lines,
        reason = "update keeps public component routing and product command arbitration in one phase"
    )]
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
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
        // Props are built once through these constructors and reused by both
        // update and draw.  This keeps the app phase-pure and avoids a
        // render-time configuration drift between the two paths.
        let _ = (header_panel(), results_panel(), status_bar(&[], &[]));
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
            let list_response = connection_list().update(
                cx,
                &mut self.connection_list_state,
                &self.connections_screen.connections,
            );
            if let Some(action) = list_response.action_ref()
                && let ListAction::Activated(key) | ListAction::Chose(key) = action
                && let Some(index) = self
                    .connections_screen
                    .connections
                    .iter()
                    .position(|connection| connection_key(connection) == *key)
            {
                let _ = self.connect(index);
            }
            response |= list_response.erase();
            return response;
        }

        let split = SplitPane::new(WORKBENCH_SPLIT, SplitAxis::Horizontal)
            .min_first(28)
            .min_second(20);
        response |= split.update(cx, &mut self.split_state).erase();
        let tree_response =
            explorer_tree().update(cx, &mut self.explorer_tree_state, &self.workbench.explorer);
        if let Some(TreeAction::Activated(key)) = tree_response.action_ref()
            && let Some(item) = self
                .workbench
                .explorer
                .iter()
                .find(|item| explorer_key(item) == *key)
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

        response |= query_input(None)
            .update(cx, &mut self.query_state, &mut self.query)
            .erase();
        self.sync_query_tab();
        let editable = self.result.is_editable();
        let (columns, column_count) = Self::column_specs(&self.columns, editable);
        let grid = result_grid(&columns[..column_count]);
        let grid_response = if self.result.is_editable() {
            grid.update_editable(cx, &mut self.grid_state, &mut self.result)
        } else {
            grid.update(cx, &mut self.grid_state, &self.result)
        };
        if let Some(action) = grid_response.action_ref() {
            self.handle_grid(action);
        }
        response |= grid_response.erase();
        response
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let rows = fixed_flex_rows(ui.full(), 3, 1);
        let header = rows[0];
        let body = rows[1];
        let footer = rows[2];
        if self.form_open {
            header_panel()
                .title("Connect to database")
                .draw(ui, ui.full(), |ui, area| {
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
            let title = format!(
                "TablePro · connections · {} configured",
                self.connections_screen.connections.len()
            );
            ui.paint_str(header, &title, ui.surface_style());
            connections_panel().draw(ui, body, |ui, area| {
                connection_list().draw(
                    ui,
                    area,
                    &self.connection_list_state,
                    &self.connections_screen.connections,
                );
            });
        } else {
            let title = format!(
                "TablePro · {} · {}",
                self.surface.label(),
                self.connection.environment.label()
            );
            ui.paint_str(header, &title, ui.surface_style());
            let workbench_rows = fixed_flex_pair(body, 2);
            let tabs_area = workbench_rows[0];
            let content = workbench_rows[1];
            tab_strip().draw(ui, tabs_area, &self.tabs_state, &self.workbench.tabs);
            let split = SplitPane::new(WORKBENCH_SPLIT, SplitAxis::Horizontal)
                .min_first(28)
                .min_second(20);
            split.draw(
                ui,
                content,
                &self.split_state,
                |ui, explorer_area, work_area| {
                    Panel::new(EXPLORER_PANEL)
                        .kind(PanelKind::Framed)
                        .title("Explorer")
                        .draw(ui, explorer_area, |ui, area| {
                            explorer_tree().draw(
                                ui,
                                area,
                                &self.explorer_tree_state,
                                &self.workbench.explorer,
                            );
                        });
                    let work_rows = fixed_flex_pair(work_area, 3);
                    let query_area = work_rows[0];
                    let result_area = work_rows[1];
                    Field::new("SQL query", query_input(Some(&self.query)))
                        .plain(true)
                        .draw(ui, query_area, &self.query_state);
                    let (columns, column_count) =
                        Self::column_specs(&self.columns, self.result.is_editable());
                    let grid = result_grid(&columns[..column_count]);
                    let meta = format!(
                        "{} rows · {}",
                        self.result.total(),
                        self.result.source().unwrap_or("no relation")
                    );
                    results_panel()
                        .meta(&meta)
                        .draw(ui, result_area, |ui, area| {
                            grid.draw(ui, area, &self.grid_state, &self.result);
                        });
                },
            );
        }
        let left = [StatusItem::new(&self.connection.name).strong()];
        let right = [StatusItem::new(&self.status)];
        status_bar(&left, &right).draw(ui, footer);
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
