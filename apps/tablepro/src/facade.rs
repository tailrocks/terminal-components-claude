//! `TablePro`'s facade-only application shell.

use tui_next::{
    ActionKey, App, Chord, Cx, Field, Grid, GridAction, GridState, Id, KeyCode, KeyMap,
    KeyModifiers, KeyPhase, Panel, PanelKind, Response, Size, StatusBar, StatusItem, TextInput,
    TextInputState, Ui, UpdateCause, Variant,
};

use crate::db::{self, Catalog, ColType, Connection, SafeMode};
use crate::domain::ResultGrid;
use crate::sql::{self, Decision, Statement};

const QUERY: Id = Id::root("tablepro.query");
const RESULTS: Id = Id::root("tablepro.results");
const STATUS: Id = Id::root("tablepro.status");
const RUN: ActionKey = ActionKey::custom("tablepro.run");
const QUIT: ActionKey = ActionKey::custom("tablepro.quit");

fn app_keymap() -> KeyMap {
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
}

/// The result of attempting to run the current query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryOutcome {
    /// A deterministic SELECT completed.
    Executed {
        /// Number of loaded rows.
        rows: usize,
        /// Whether the result supports pending cell edits.
        editable: bool,
    },
    /// The safety gate requires a deliberate user action.
    ConfirmationRequired {
        /// Whether the acknowledgement must name the target.
        deliberate: bool,
        /// Human-readable safety summary.
        summary: String,
    },
    /// The active connection refuses the statement.
    Denied {
        /// Human-readable denial reason.
        summary: String,
    },
    /// The parser or deterministic executor rejected the statement.
    Rejected {
        /// Human-readable error.
        message: String,
    },
}

/// The `TablePro` product shell.
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
}

impl core::fmt::Debug for TableProApp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TableProApp")
            .field("catalog", &self.catalog)
            .field("connections", &self.connections.len())
            .field("connection", &self.connection.name)
            .field("keymap", &"<keymap>")
            .field("safe_mode", &self.safe_mode)
            .field("query", &"[redacted from debug output]")
            .field("query_state", &"<input state>")
            .field("columns", &self.columns.len())
            .field("result", &self.result)
            .field("grid_state", &"<grid state>")
            .field("status", &self.status)
            .field("quit", &self.quit)
            .finish()
    }
}

impl Default for TableProApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TableProApp {
    /// Build the deterministic catalog and connect to the local demo entry.
    pub fn new() -> Self {
        let catalog = Catalog::acme_prod();
        let connections = db::connections();
        let connection = match connections.first().cloned() {
            Some(connection) => connection,
            None => Connection {
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
                outcome: db::ConnectOutcome::Ok,
            },
        };
        let mut app = Self {
            safe_mode: connection.safe_mode,
            catalog,
            connections,
            connection,
            keymap: app_keymap(),
            query:
                "SELECT * FROM orders WHERE status = 'pending' ORDER BY total_amount DESC LIMIT 20"
                    .to_owned(),
            query_state: TextInputState::default(),
            columns: Vec::new(),
            result: ResultGrid::empty(),
            grid_state: GridState::default(),
            status: "Ready · Ctrl+R runs · Ctrl+Q quits".to_owned(),
            quit: false,
        };
        let _ = app.execute_query();
        app
    }

    /// The active connection's safe-mode policy.
    pub fn safe_mode(&self) -> SafeMode {
        self.safe_mode
    }

    /// The current query text.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Replace the query and run it through the same safety gate as Ctrl+R.
    pub fn run_query(&mut self, query: impl Into<String>) -> QueryOutcome {
        self.query = query.into();
        self.query_state = TextInputState::default();
        self.execute_query()
    }

    /// The current result adapter.
    pub fn result(&self) -> &ResultGrid {
        &self.result
    }

    /// The latest status line.
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Select a deterministic connection by index.
    pub fn connect(&mut self, index: usize) -> bool {
        let Some(connection) = self.connections.get(index).cloned() else {
            return false;
        };
        if connection.outcome != db::ConnectOutcome::Ok {
            self.status = format!("Connection failed: {}", connection.name);
            return false;
        }
        self.safe_mode = connection.safe_mode;
        self.status = format!("Connected to {}", connection.name);
        self.connection = connection;
        true
    }

    /// Parse, safety-check and execute the current deterministic query.
    pub fn execute_query(&mut self) -> QueryOutcome {
        let statement = match sql::parse(self.query.trim()) {
            Ok(statement) => statement,
            Err(error) => {
                let outcome = QueryOutcome::Rejected {
                    message: error.message,
                };
                self.status = outcome_message(&outcome);
                return outcome;
            }
        };
        let table = match &statement {
            Statement::Select(select) => self.catalog.find(select.schema.as_deref(), &select.table),
            _ => None,
        };
        match sql::gate(self.safe_mode, &statement) {
            Decision::Deny => {
                let risk = sql::assess(&statement, table);
                let outcome = QueryOutcome::Denied {
                    summary: format!("{} is denied in Read-Only mode", risk.action),
                };
                self.status = outcome_message(&outcome);
                outcome
            }
            Decision::Confirm { deliberate } => {
                let risk = sql::assess(&statement, table);
                let outcome = QueryOutcome::ConfirmationRequired {
                    deliberate,
                    summary: format!("{} · {}", risk.action, risk.scope),
                };
                self.status = outcome_message(&outcome);
                outcome
            }
            Decision::Run => {
                if let Statement::Select(select) = statement {
                    match sql::run_select(&self.catalog, &select) {
                        Ok(result) => {
                            let outcome = QueryOutcome::Executed {
                                rows: result.rows.len(),
                                editable: result.editable,
                            };
                            self.columns.clone_from(&result.columns);
                            self.result = ResultGrid::from_result(&result);
                            self.grid_state = GridState::default();
                            self.status = outcome_message(&outcome);
                            outcome
                        }
                        Err(error) => {
                            let outcome = QueryOutcome::Rejected {
                                message: error.message,
                            };
                            self.status = outcome_message(&outcome);
                            outcome
                        }
                    }
                } else {
                    let outcome = QueryOutcome::Rejected {
                        message: "The demo executor only runs SELECT statements".to_owned(),
                    };
                    self.status = outcome_message(&outcome);
                    outcome
                }
            }
        }
    }

    fn column_specs_for(
        columns: &[(String, ColType)],
        editable: bool,
    ) -> Vec<tui_next::Column<'_>> {
        columns
            .iter()
            .take(tui_next::GRID_MAX_COLUMNS)
            .enumerate()
            .map(|(index, (name, _))| {
                let mut column = tui_next::Column::new(
                    tui_next::ColumnKey::num((index as u16).saturating_add(1)),
                    name.as_str(),
                );
                column.sortable = true;
                column.editable = editable;
                column.sticky = index == 0;
                column
            })
            .collect()
    }

    fn handle_grid_action(&mut self, action: &GridAction) {
        match action {
            GridAction::Sort(key, direction) => {
                self.result.sort(*key, *direction);
                self.status = format!("Sorted column {}", key.raw());
            }
            GridAction::Copy(text) => {
                self.status = format!("Copied {} cells", text.lines().count());
            }
            GridAction::Activated(key) => {
                self.status = format!("Activated row {key:?}");
            }
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
        Some(value) => input.value(value),
        None => input,
    }
}

fn results_grid<'a>(columns: &'a [tui_next::Column<'a>]) -> Grid<'a> {
    Grid::new(RESULTS, columns)
        .nav(tui_next::NavUnit::Cell)
        .select_mode(tui_next::SelectMode::Multi)
}

fn results_panel(meta: &str) -> Panel<'_> {
    Panel::new(RESULTS)
        .kind(PanelKind::Framed)
        .title("Results")
        .meta(meta)
}

fn status_bar<'a>(left: &'a [StatusItem<'a>], right: &'a [StatusItem<'a>]) -> StatusBar<'a> {
    StatusBar::new(STATUS)
        .left(left)
        .right(right)
        .variant(Variant::DEFAULT)
}

impl App for TableProApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        if matches!(
            cx.update_cause(),
            UpdateCause::Bootstrap | UpdateCause::Event
        ) && let Some(command) = cx.command()
        {
            if command == QUIT {
                self.quit = true;
                cx.quit();
                response |= Response::consumed();
            } else if command == RUN {
                let _ = self.execute_query();
                response |= Response::changed();
            }
        }

        response |= query_input(None)
            .update(cx, &mut self.query_state, &mut self.query)
            .erase();

        let editable = self.result.is_editable();
        let columns = Self::column_specs_for(&self.columns, editable);
        let grid = results_grid(&columns);
        let grid_response = if self.result.is_editable() {
            grid.update_editable(cx, &mut self.grid_state, &mut self.result)
        } else {
            grid.update(cx, &mut self.grid_state, &self.result)
        };
        if let Some(action) = grid_response.action_ref() {
            self.handle_grid_action(action);
        }
        response |= grid_response.erase();
        let _ = results_panel("");
        let _ = status_bar(&[], &[]);
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let rows = tui_next::layout::rows(
            ui.full(),
            &[
                tui_next::Track::Fixed(4),
                tui_next::Track::Flex(1),
                tui_next::Track::Fixed(1),
            ],
        );
        let query_area = rows.first().copied().unwrap_or_else(|| ui.full());
        let result_area = rows.get(1).copied().unwrap_or_else(|| ui.full());
        let status_area = rows.get(2).copied().unwrap_or_else(|| ui.full());

        Field::new("SQL query", query_input(Some(&self.query)))
            .plain(true)
            .draw(ui, query_area, &self.query_state);

        let columns = Self::column_specs_for(&self.columns, self.result.is_editable());
        let grid = results_grid(&columns);
        let meta = format!(
            "{} rows · {}",
            self.result.total(),
            self.result.source().unwrap_or("no relation")
        );
        results_panel(&meta).draw(ui, result_area, |ui, body| {
            grid.draw(ui, body, &self.grid_state, &self.result);
        });

        let left = [StatusItem::new(&self.connection.name).strong()];
        let right = [StatusItem::new(&self.status)];
        status_bar(&left, &right).draw(ui, status_area);
    }

    fn should_quit(&self) -> bool {
        self.quit
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }

    fn min_size(&self) -> Size {
        Size {
            min: (72, 20),
            preferred: (120, 36),
        }
    }
}
