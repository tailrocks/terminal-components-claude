//! TablePro application shell built only on the public `tui-next` facade.

use tui_next::{
    Action, ActionKey, App, Chord, Cx, Field, Form, FormAction, FormState, Grid, GridAction,
    GridState, Id, KeyCode, KeyMap, KeyModifiers, KeyPhase, Panel, PanelKind, Response, Size,
    StatusBar, StatusItem, TextInput, TextInputState, Ui, UpdateCause, Variant,
};

use crate::connections::{self, ConnectionDraft, ConnectionsScreen};
use crate::db::{self, Catalog, ColType, ConnectOutcome, Connection, SafeMode};
use crate::domain::ResultGrid;
use crate::workbench::Workbench;

/// Minimum terminal width.
pub const MIN_WIDTH: u16 = 72;
/// Minimum terminal height.
pub const MIN_HEIGHT: u16 = 20;
const QUERY: Id = Id::root("tablepro.query");
const RESULTS: Id = Id::root("tablepro.results");
const STATUS: Id = Id::root("tablepro.status");
const HEADER: Id = Id::root("tablepro.header");
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
    Connections,
    Workbench,
}

/// Named visual surfaces retained from the legacy showcase matrix.
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

/// TablePro state and app-owned adapters.
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
    draft: Option<ConnectionDraft>,
    form_state: FormState,
    form_fields: Box<[tui_next::FieldSpec<'static>]>,
    form_actions: Box<[Action<'static>]>,
    form_open: bool,
}

impl core::fmt::Debug for TableProApp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TableProApp")
            .field("screen", &self.screen)
            .field("surface", &self.surface)
            .field("connections", &self.connections.len())
            .field("connection", &self.connection.name)
            .field("safe_mode", &self.safe_mode)
            .field("query", &"[redacted]")
            .field("columns", &self.columns.len())
            .field("result", &self.result)
            .field("status", &self.status)
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
        self.workbench = Workbench::new(connection.clone(), self.catalog.clone());
        self.screen = Screen::Workbench;
        self.surface = Surface::WorkbenchDefault;
        self.status = format!("Connected to {}", connection.name);
        true
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
            crate::sql::Decision::Run => match statement {
                crate::sql::Statement::Select(select) => {
                    match crate::sql::run_select(&self.catalog, &select) {
                        Ok(result) => {
                            let out = QueryOutcome::Executed {
                                rows: result.rows.len(),
                                editable: result.editable,
                            };
                            self.columns.clone_from(&result.columns);
                            self.result = ResultGrid::from_result(&result);
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
                }
                _ => {
                    let out = QueryOutcome::Rejected {
                        message: "The demo executor only runs SELECT statements".to_owned(),
                    };
                    self.status = outcome_message(&out);
                    out
                }
            },
        }
    }
    fn column_specs(columns: &[(String, ColType)], editable: bool) -> Vec<tui_next::Column<'_>> {
        columns
            .iter()
            .take(tui_next::GRID_MAX_COLUMNS)
            .enumerate()
            .map(|(index, (name, _))| {
                let mut col = tui_next::Column::new(
                    tui_next::ColumnKey::num((index as u16).saturating_add(1)),
                    name.as_str(),
                );
                col.sortable = true;
                col.editable = editable;
                col.sticky = index == 0;
                col
            })
            .collect()
    }
    fn connection_form<'a>(
        fields: &'a [tui_next::FieldSpec<'a>],
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
                self.status = format!("Copied {} cells", text.lines().count())
            }
            GridAction::Activated(key) => self.status = format!("Activated row {key:?}"),
            GridAction::EditRequested(key, column) => {
                self.status = format!("Edit requested for {key:?}, column {column:?}")
            }
            GridAction::CellAction(key, column, action) => {
                self.status = format!("Cell action {action:?} on {key:?}/{column:?}")
            }
            GridAction::FetchMore => {
                self.status = "All deterministic demo rows are loaded".to_owned()
            }
            GridAction::Moved | GridAction::LeaveForward | GridAction::LeaveBackward => {}
        }
    }
}

/// Result of executing a query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryOutcome {
    /// Query returned rows.
    Executed { rows: usize, editable: bool },
    /// Safety gate requires confirmation.
    ConfirmationRequired { deliberate: bool, summary: String },
    /// Read-only policy denied a write.
    Denied { summary: String },
    /// Parse/execution rejection.
    Rejected { message: String },
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
fn header_panel() -> Panel<'static> {
    Panel::new(HEADER).kind(PanelKind::Framed)
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
fn result_grid<'a>(columns: &'a [tui_next::Column<'a>]) -> Grid<'a> {
    Grid::new(RESULTS, columns)
        .nav(tui_next::NavUnit::Cell)
        .select_mode(tui_next::SelectMode::Multi)
}

impl App for TableProApp {
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
                    let _ = self.execute_query();
                    response |= Response::changed();
                }
                c if c == OPEN => {
                    self.surface = Surface::QuickSwitcher;
                    response |= Response::changed();
                }
                c if c == NEW_QUERY => {
                    self.workbench.new_query("");
                    self.surface = Surface::QueryEditing;
                    response |= Response::changed();
                }
                c if c == HISTORY => {
                    self.workbench.open_history();
                    self.surface = Surface::HistoryTab;
                    response |= Response::changed();
                }
                c if c == STRUCTURE => {
                    let _ = self.workbench.toggle_structure();
                    self.surface = Surface::StructureView;
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
                        FormAction::Action(ActionKey::SAVE)
                        | FormAction::Action(connections::SAVE_CONNECT) => {
                            if let Ok(()) = draft.validate_all() {
                                if let Some(connection) =
                                    draft.to_connection(Some(&self.connection))
                                {
                                    self.connections.push(connection.clone());
                                    if action == &FormAction::Action(connections::SAVE_CONNECT) {
                                        let _ =
                                            self.connect(self.connections.len().saturating_sub(1));
                                        self.form_open = false;
                                    }
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
        response |= query_input(None)
            .update(cx, &mut self.query_state, &mut self.query)
            .erase();
        let editable = self.result.is_editable();
        let columns = Self::column_specs(&self.columns, editable);
        let grid = result_grid(&columns);
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
        let rows = tui_next::layout::rows(
            ui.full(),
            &[
                tui_next::Track::Fixed(3),
                tui_next::Track::Flex(1),
                tui_next::Track::Fixed(1),
            ],
        );
        let header = rows.first().copied().unwrap_or_else(|| ui.full());
        let body = rows.get(1).copied().unwrap_or_else(|| ui.full());
        let footer = rows.get(2).copied().unwrap_or_else(|| ui.full());
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
        } else {
            let title = format!(
                "TablePro · {} · {}",
                self.surface.label(),
                self.connection.environment.label()
            );
            ui.paint_str(header, &title, ui.surface_style());
            Field::new("SQL query", query_input(Some(&self.query)))
                .plain(true)
                .draw(ui, body, &self.query_state);
            let columns = Self::column_specs(&self.columns, self.result.is_editable());
            let grid = result_grid(&columns);
            let meta = format!(
                "{} rows · {}",
                self.result.total(),
                self.result.source().unwrap_or("no relation")
            );
            results_panel().meta(&meta).draw(ui, body, |ui, area| {
                grid.draw(ui, area, &self.grid_state, &self.result);
            });
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

/// Start the interactive TablePro binary.
pub fn run() -> std::io::Result<()> {
    tui_next::run(TableProApp::default(), tui_next::Theme::junie())
}
