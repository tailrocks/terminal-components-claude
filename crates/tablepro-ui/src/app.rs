//! `TablePro` application shell built only on the public `junie-tui` facade.

use termrock::author::{PaintStyle, StyleDefaults};
use termrock::{
    Action, ActionKey, App, Button, Checkbox, Chord, ColumnKey, Cx, Dialog, DialogAction,
    DialogState, Empty, EmptyState, Family, FgStep, Focusability, Form, FormAction, FormState,
    FrameRead, Grid, GridAction, GridEditor, GridModel, Id, Intent, ItemKey, KeyCode, KeyMap,
    KeyModifiers, KeyPhase, LayerId, LayerSize, LayerSpec, Modifier, NodeKind, Panel, PanelKind,
    Part, Phase, PickerAction, Response, Role, RowUi, Select, SelectAction, Size, SortDir, Span,
    SplitAxis, SplitPane, SplitPaneState, StylePatch, Tabs, TabsAction, TabsState, TextAction,
    TextInput, TextInputState, Theme, Tree, TreeAction, TreeNode, TreeState, Ui, UpdateCause,
    Variant, truncate, wrap,
};

use crate::connections::{self, ConnectionDraft, ConnectionsScreen};
use crate::domain::ResultGrid;
use crate::model::SwitchTarget;
use crate::filter_editor::{
    Filter, FilterEditor, FilterFocus, FilterOp, FilterOutcome, FILTER_APPLY, FILTER_CANCEL,
    FILTER_COL, FILTER_EDITOR, FILTER_OP, FILTER_VALUE, FILTER_VALUE2,
};
use crate::filter_editor::{column_key, column_row, op_key, op_row};
use crate::quick_switcher::{self, QuickSwitcher};
use crate::safe_mode_picker::{self, SafeModePicker};
use crate::safety_dialog::{
    Prop, SAFETY_CANCEL, SAFETY_CONFIRM, SAFETY_DIALOG, SAFETY_INPUT, SafetyDialog,
    SafetyDialogAction, SafetyFocus, SafetyIntent, Tone,
};
use crate::tab_list::{self, TabList};
use crate::tabs::{ExplorerItem, GridView, QueryPaneMaximized, Tab, TabKey, TabRecord, TableTab};
use crate::workbench::Workbench;
use tablepro_demo as db;
use tablepro_domain::{
    Catalog, ColType, ConnectOutcome, Connection, Engine, Environment, ObjectKind, SafeMode, Table,
};
use tablepro_sql as sql;

/// Minimum terminal width.
pub const MIN_WIDTH: u16 = 72;
/// Minimum terminal height.
pub const MIN_HEIGHT: u16 = 20;
const CONNECTIONS: Id = Id::root("tablepro.connections.list");
const CONNECTIONS_PANEL: Id = Id::root("tablepro.connections.panel");
const CONNECTION_FILTER: Id = Id::root("tablepro.connections.filter");
const EXPLORER: Id = Id::root("tablepro.workbench.explorer.tree");
const EXPLORER_PANEL: Id = Id::root("tablepro.workbench.explorer.panel");
const TAB_STRIP: Id = Id::root("tablepro.workbench.tab-strip");
const WORKBENCH_SPLIT: Id = Id::root("tablepro.workbench.split");
const QUERY_EMPTY: Id = Id::root("tablepro.workbench.query.empty");
const RUN: ActionKey = ActionKey::application("tablepro.run");
const UNDO: ActionKey = ActionKey::application("tablepro.undo");
const INSERT_ROW: ActionKey = ActionKey::application("tablepro.insert-row");
const DUPLICATE_ROW: ActionKey = ActionKey::application("tablepro.duplicate-row");
const DELETE_ROW: ActionKey = ActionKey::application("tablepro.delete-row");
const DISCARD_ROWS: ActionKey = ActionKey::application("tablepro.discard-rows");
const QUIT: ActionKey = ActionKey::application("tablepro.quit");
const CANCEL_OR_QUIT: ActionKey = ActionKey::application("tablepro.cancel-or-quit");
const DELETE_CONNECTION: ActionKey = ActionKey::application("tablepro.delete-connection");
const EDIT_CONNECTION: ActionKey = ActionKey::application("tablepro.edit-connection");
const FORM_TAB_PREV: ActionKey = ActionKey::application("tablepro.form-tab-prev");
const FORM_TAB_NEXT: ActionKey = ActionKey::application("tablepro.form-tab-next");
const QUIT_DIALOG: Id = Id::root("tablepro.quit-dialog");
const QUIT_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::CONFIRM, "Quit"),
];
const CLOSE_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::CONFIRM, "Close anyway"),
];

const RECONNECT_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::CONFIRM, "Reconnect"),
];
const DISCARD_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::CONFIRM, "Discard"),
];
const REPLACE_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::CONFIRM, "Run query"),
];

#[derive(Debug)]
struct DestructiveRequest {
    intent: DestructiveIntent,
    owner: std::sync::Weak<()>,
}

impl DestructiveRequest {
    fn dialog(&self) -> Dialog<'_> {
        self.intent.dialog()
    }
}

enum DestructiveIntent {
    Quit {
        question: String,
        scope: Vec<(TabKey, u64)>,
    },
    DeleteConnection {
        index: usize,
        question: String,
    },
    CloseTab {
        key: TabKey,
        generation: u64,
    },
    Reconnect {
        target: Box<Connection>,
        source: Box<Connection>,
        scope: Vec<(TabKey, u64)>,
    },
    DiscardRows {
        key: TabKey,
        generation: u64,
        question: String,
    },
    ReplaceResult {
        key: TabKey,
        query: String,
        generation: u64,
        connection: Box<Connection>,
    },
}

impl core::fmt::Debug for DestructiveIntent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Quit { .. } => f.write_str("Quit"),
            Self::DeleteConnection { index, .. } => {
                f.debug_tuple("DeleteConnection").field(index).finish()
            }
            Self::CloseTab { key, .. } => f.debug_tuple("CloseTab").field(key).finish(),
            Self::Reconnect { scope, .. } => f
                .debug_struct("Reconnect")
                .field("tabs", &scope.len())
                .finish_non_exhaustive(),
            Self::DiscardRows { key, .. } => f.debug_tuple("DiscardRows").field(key).finish(),
            Self::ReplaceResult { key, .. } => f
                .debug_struct("ReplaceResult")
                .field("key", key)
                .finish_non_exhaustive(),
        }
    }
}

impl DestructiveIntent {
    fn dialog(&self) -> Dialog<'_> {
        match self {
            Self::Quit { question, .. } => {
                Dialog::destructive(QUIT_DIALOG, "Quit TablePro?", question).actions(&QUIT_ACTIONS)
            }
            Self::DeleteConnection { question, .. } => {
                Dialog::destructive(QUIT_DIALOG, "Delete connection?", question)
            }
            Self::CloseTab { .. } => Dialog::destructive(
                QUIT_DIALOG,
                "Close tab with unsaved work?",
                "Pending row edits and unsaved query text in this tab will be lost.",
            )
            .actions(&CLOSE_ACTIONS),
            Self::Reconnect { .. } => Dialog::destructive(
                QUIT_DIALOG,
                "Reconnect with unsaved work?",
                "Pending row edits and unsaved query text in all tabs will be lost.",
            )
            .actions(&RECONNECT_ACTIONS),
            Self::DiscardRows { question, .. } => {
                Dialog::destructive(QUIT_DIALOG, "Discard unsaved changes?", question)
                    .actions(&DISCARD_ACTIONS)
            }
            Self::ReplaceResult { .. } => Dialog::destructive(
                QUIT_DIALOG,
                "Replace results with unsaved edits?",
                "Pending row edits in this query result will be lost.",
            )
            .actions(&REPLACE_ACTIONS),
        }
    }
}

const OPEN: ActionKey = ActionKey::application("tablepro.open");
const NEW_QUERY: ActionKey = ActionKey::application("tablepro.new-query");
const CLOSE_TAB: ActionKey = ActionKey::application("tablepro.close-tab");
const HISTORY: ActionKey = ActionKey::application("tablepro.history");
const STRUCTURE: ActionKey = ActionKey::application("tablepro.structure");
const FORM: ActionKey = ActionKey::application("tablepro.form");
const HELP: ActionKey = ActionKey::application("tablepro.help");
const TAB_LIST: ActionKey = ActionKey::application("tablepro.tab-list");
const FILTER: ActionKey = ActionKey::application("tablepro.filter");
const FILTER_EMPTY: ActionKey = ActionKey::application("tablepro.filter-empty");
const SORT: ActionKey = ActionKey::application("tablepro.sort");
const PREVIEW: ActionKey = ActionKey::application("tablepro.preview");
const SAVE: ActionKey = ActionKey::application("tablepro.save");
const EXPLAIN: ActionKey = ActionKey::application("tablepro.explain");
#[allow(dead_code)]
const CLEAR_QUERY: ActionKey = ActionKey::application("tablepro.clear-query");
const COMPLETE: ActionKey = ActionKey::application("tablepro.complete");
const PALETTE: ActionKey = ActionKey::application("tablepro.palette");
const TOGGLE_EXPLORER: ActionKey = ActionKey::application("tablepro.toggle-explorer");
const MAXIMIZE: ActionKey = ActionKey::application("tablepro.maximize");
const SAFE_MODE: ActionKey = ActionKey::application("tablepro.safe-mode");
const FOCUS_EXPLORER: ActionKey = ActionKey::application("tablepro.focus-explorer");

const HELP_DIALOG_ID: Id = Id::root("tablepro.help-dialog");
const HELP_TEXT: &str = "\
Tab / Shift+Tab   move focus · 0 explorer · Esc back\n\
Ctrl+O            Open Quickly (tables, schemas, tabs, queries)\n\
Ctrl+T / Ctrl+W   new / close tab · [ ] switch · Ctrl+G tab list\n\
Ctrl+R / F5       run statement at cursor · Alt+R run all\n\
Ctrl+X / Alt+X    EXPLAIN / EXPLAIN ANALYZE\n\
Esc / Ctrl+C      cancel a running query\n\
Ctrl+D            Data / Structure · Ctrl+F filter (grid) or find (editor)\n\
Alt+D             duplicate the current grid row\n\
Ctrl+S            save pending row changes · p preview SQL\n\
Ctrl+Y            query history · Ctrl+L Safe Mode · Ctrl+B explorer · z zoom\n\
q                 quit";

fn help_dialog(cols: u16, screen_rows: u16) -> Dialog<'static> {
    let w = 78.min(cols.saturating_sub(4)).max(20);
    let max_h = screen_rows.saturating_sub(2);
    Dialog::info(HELP_DIALOG_ID, "Keyboard")
        .width(w)
        .max_height(max_h)
        .body_rows(0)
        .description(HELP_TEXT)
}

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
        count: String,
    },
    Object {
        item: ExplorerItem,
        count: String,
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

fn quit_keymap() -> KeyMap {
    KeyMap::new()
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('q'), KeyModifiers::NONE),
            QUIT,
        )
        .bind(
            KeyPhase::Capture,
            Chord::with(KeyCode::Char('c'), KeyModifiers::CONTROL),
            CANCEL_OR_QUIT,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            QUIT,
        )
}

fn keymap() -> KeyMap {
    quit_keymap()
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('u')), UNDO)
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('+')), INSERT_ROW)
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('d'), KeyModifiers::ALT),
            DUPLICATE_ROW,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('D'), KeyModifiers::ALT),
            DUPLICATE_ROW,
        )
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('-')), DELETE_ROW)
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('U')),
            DISCARD_ROWS,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('d')),
            DELETE_CONNECTION,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('D')),
            DELETE_CONNECTION,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('e')),
            EDIT_CONNECTION,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('E')),
            EDIT_CONNECTION,
        )
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Left), FORM_TAB_PREV)
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Right), FORM_TAB_NEXT)
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('r'), KeyModifiers::CONTROL),
            RUN,
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
            Chord::with(KeyCode::Char('w'), KeyModifiers::CONTROL),
            CLOSE_TAB,
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
            SAFE_MODE,
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
            Chord::with(KeyCode::Char('0'), KeyModifiers::NONE),
            FOCUS_EXPLORER,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('f'), KeyModifiers::NONE),
            FILTER,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('f'), KeyModifiers::CONTROL),
            FILTER_EMPTY,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('s'), KeyModifiers::NONE),
            SORT,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('/'), KeyModifiers::NONE),
            FILTER,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('b'), KeyModifiers::CONTROL),
            TOGGLE_EXPLORER,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('z'), KeyModifiers::NONE),
            MAXIMIZE,
        )
}

fn fallback_connection(catalog: &Catalog) -> Connection {
    Connection {
        name: "Local PostgreSQL".to_owned(),
        engine: Engine::Postgres,
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
    empty_result: ResultGrid,
    status: String,
    status_since: Option<termrock::Moment>,
    quit: bool,
    destructive_intent: Option<DestructiveRequest>,
    destructive_notice: Option<&'static str>,
    quit_state: DialogState,
    switcher: QuickSwitcher,
    switcher_open: bool,
    tab_list: TabList,
    tab_list_open: bool,
    safe_mode_picker: SafeModePicker,
    safe_mode_open: bool,
    help_dialog_state: DialogState,
    help_open: bool,
    pub safety_dialog: Option<SafetyDialog>,
    pub filter_editor: Option<FilterEditor>,
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
    form_fields: Box<[termrock::FieldSpec<'static>]>,
    form_actions: Box<[Action<'static>]>,
    form_open: bool,
    form_is_edit: bool,
    form_tab: usize,
    form_editing: bool,
    committing: Option<u32>,
    screen_size: core::cell::Cell<(u16, u16)>,
    workbench_focus: Id,
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
            .field("result", &self.result())
            .field("empty_result", &self.empty_result)
            .field("grid_state", &"<grid state>")
            .field("status", &self.status)
            .field("quit", &self.quit)
            .field("destructive_intent", &self.destructive_intent)
            .field("has_destructive_notice", &self.destructive_notice.is_some())
            .field("quit_state", &self.quit_state)
            .field("switcher", &"<query and target snapshot>")
            .field("safety_dialog", &self.safety_dialog.is_some())
            .field("filter_editor", &self.filter_editor.is_some())
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

fn format_thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.chars().rev().collect()
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
        let connection_nodes = build_connection_nodes(&connections, "");
        let connection_tree_state = initial_connection_tree_state(&connection_nodes);
        let connection_visual_tree_state = initial_connection_visual_tree_state(&connection_nodes);
        let explorer_nodes = build_explorer_nodes(&catalog);
        let explorer_tree_state = initial_explorer_tree_state(&explorer_nodes, "public");
        let mut app = Self {
            safe_mode: connection.safe_mode,
            catalog: catalog.clone(),
            connections: connections.clone(),
            connection: connection.clone(),
            keymap: keymap(),
            empty_result: ResultGrid::empty(),
            status: String::new(),
            status_since: None,
            quit: false,
            destructive_intent: None,
            destructive_notice: None,
            quit_state: DialogState::default(),
            switcher: QuickSwitcher::default(),
            switcher_open: false,
            tab_list: TabList::default(),
            tab_list_open: false,
            safe_mode_picker: SafeModePicker::default(),
            safe_mode_open: false,
            help_dialog_state: DialogState::default(),
            help_open: false,
            safety_dialog: None,
            filter_editor: None,
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
            split_state: SplitPaneState::new(25),
            draft: None,
            form_state: FormState::default(),
            form_fields: Box::from(connections::form_fields()),
            form_actions: Box::from(connections::form_actions()),
            form_open: false,
            form_is_edit: false,
            form_tab: 0,
            form_editing: false,
            committing: None,
            screen_size: core::cell::Cell::new((120, 40)),
            workbench_focus: EXPLORER,
        };
        app.workbench.new_query(
            "SELECT * FROM orders WHERE status = 'pending' ORDER BY total_amount DESC LIMIT 20",
        );
        let _ = app.execute_query();
        app.status.clear();
        app
    }
    /// Active safe-mode policy.
    pub const fn safe_mode(&self) -> SafeMode {
        self.safe_mode
    }
    /// Current SQL text.
    pub fn query(&self) -> &str {
        match self.workbench.active() {
            Some(Tab::Query(tab)) => tab.editor_state.draft_text().unwrap_or(&tab.query),
            _ => "",
        }
    }
    /// Current result adapter.
    pub fn result(&self) -> &ResultGrid {
        self.workbench
            .active_grid()
            .map_or(&self.empty_result, |(_, grid)| &grid.model)
    }
    /// Stable identity of the active SQL editor.
    pub fn query_id(&self) -> Option<Id> {
        match self.workbench.active() {
            Some(Tab::Query(_)) => self.workbench.active_key().map(|key| key.control("query")),
            _ => None,
        }
    }
    /// Stable identity of the active result or structure grid.
    pub fn result_id(&self) -> Option<Id> {
        self.workbench.active_grid().map(|(id, _)| id)
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
        self.form_is_edit = false;
        self.form_tab = 0;
        self.form_editing = false;
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
                let _ = tab.result.model.commit_cell(0, 6, "EUR");
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
        if surface == Surface::FilterEditor
            && let Some(tab) = self.workbench.active_table() {
                let columns = tab.result.columns.clone();
                let editor = FilterEditor::new(columns, None, None, 0);
                self.filter_editor = Some(editor);
            }
        self.surface = surface;
    }

    fn reset_visual_workbench(&mut self) {
        if let Some(index) = self
            .connections
            .iter()
            .position(|connection| connection.name == "Production")
            && let Some(connection) = self.connections.get(index).cloned()
        {
            let _ = self.connect_confirmed(&connection);
        }
    }

    fn connections_panel<'a>(title: &'a str, meta: Option<&'a str>, focused: bool) -> Panel<'a> {
        let panel = Panel::new(CONNECTIONS_PANEL)
            .kind(PanelKind::Framed)
            .title(title)
            .focused(focused)
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

    fn explorer_panel(schema: &str, focused: bool) -> Panel<'_> {
        Panel::new(EXPLORER_PANEL)
            .kind(PanelKind::Framed)
            .title("Explorer")
            .meta(schema)
            .focused(focused)
            .patch_part(&FRAMED_PANEL_PATCH)
            .slot(Part::GUTTER, &preserve_frame_gutter)
    }

    fn content_panel<'a>(title: &'a str, meta: Option<&'a str>, focused: bool) -> Panel<'a> {
        let panel = Panel::new(CONTENT_FRAME)
            .kind(PanelKind::Framed)
            .title(title)
            .focused(focused)
            .patch_part(&FRAMED_PANEL_PATCH)
            .slot(Part::GUTTER, &preserve_frame_gutter);
        match meta {
            Some(meta) => panel.meta(meta),
            None => panel,
        }
    }

    fn rebuild_connection_nodes(&mut self) {
        let filter = self.connections_screen.filter.clone();
        self.connection_nodes = build_connection_nodes(&self.connections, &filter);
        if self.connections_screen.filter_active || !filter.is_empty() {
            let mut state = TreeState::default();
            state.expand_all();
            if let Some(first) = self.connection_nodes.first() {
                state.set_cursor(0, connection_node_key(first));
            }
            self.connection_tree_state = state.clone();
            self.connection_visual_tree_state = state;
        } else {
            self.connection_tree_state = initial_connection_tree_state(&self.connection_nodes);
            self.connection_visual_tree_state =
                initial_connection_visual_tree_state(&self.connection_nodes);
        }
    }

    fn duplicate_connection(&mut self) {
        let i = self.connections_screen.selected;
        if i < self.connections.len() {
            let mut c = self.connections[i].clone();
            c.name = format!("{} (Copy)", c.name);
            c.last_used = "never".to_owned();
            self.connections.insert(i + 1, c.clone());
            self.connections_screen.connections.insert(i + 1, c);
            let prev_cursor = self.connection_tree_state.cursor();
            self.connection_nodes =
                build_connection_nodes(&self.connections, &self.connections_screen.filter);
            if let Some(cursor) = prev_cursor
                && let Some((idx, _)) = self
                    .connection_nodes
                    .iter()
                    .enumerate()
                    .find(|(_, n)| connection_node_key(n) == cursor)
            {
                self.connection_tree_state.set_cursor(idx, cursor);
                self.connection_visual_tree_state.set_cursor(idx, cursor);
            } else {
                self.connection_tree_state = initial_connection_tree_state(&self.connection_nodes);
                self.connection_visual_tree_state =
                    initial_connection_visual_tree_state(&self.connection_nodes);
            }
            self.sync_connection_selection();
            self.status = "Duplicated".to_owned();
        }
    }

    fn request_delete_connection(&mut self, cx: &mut Cx<'_>) {
        let i = self.connections_screen.selected;
        let Some(c) = self.connections.get(i).cloned() else {
            return;
        };
        let question = format!(
            "{} ({}@{}) will be removed from connections.json. Its password stays in the keychain until you remove it there.",
            c.name, c.user, c.host
        );
        self.open_destructive(
            cx,
            DestructiveIntent::DeleteConnection { index: i, question },
        );
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
        if let Some(Tab::Query(tab)) = self.workbench.active_mut() {
            query.clone_into(&mut tab.query);
            tab.editor_state = TextInputState::default();
        }
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
        if self.form_open {
            self.draft.as_ref()
        } else {
            None
        }
    }
    /// Close the form (kept small so deterministic tests can model Esc).
    pub fn form_open_for_test(&mut self, open: bool) {
        self.form_open = open;
        if !open {
            self.draft = None;
        }
    }
    /// Open the connection form with a clean draft for a new connection.
    pub fn begin_connection_form(&mut self) {
        self.draft = Some(ConnectionDraft::default_new());
        self.form_is_edit = false;
        self.form_tab = 0;
        self.form_editing = false;
        self.form_state = FormState::default();
        self.form_open = true;
        self.surface = Surface::Connections;
    }
    /// Open the connection form to edit an existing connection.
    pub fn begin_edit_connection_form(&mut self, index: usize) {
        let draft = self
            .connections
            .get(index)
            .map(ConnectionDraft::from_connection)
            .unwrap_or_else(|| ConnectionDraft::from_connection(&self.connection));
        self.draft = Some(draft);
        self.form_is_edit = true;
        self.form_tab = 0;
        self.form_editing = false;
        self.form_state = FormState::default();
        self.form_open = true;
        self.surface = Surface::Connections;
    }
    fn close_connection_form(&mut self) {
        self.form_open = false;
        self.form_is_edit = false;
        self.form_tab = 0;
        self.form_editing = false;
        self.form_state.zeroize();
        // Retain a scrubbed owner until late focus transitions are drained.
        self.draft = Some(ConnectionDraft::from_connection(&self.connection));
    }
    /// Select a connection and open its workbench.
    pub fn connect(&mut self, index: usize) -> bool {
        let Some(connection) = self.connections.get(index).cloned() else {
            return false;
        };
        if self.workbench.has_unsaved_work() {
            return false;
        }
        self.connect_confirmed(&connection)
    }

    fn connect_confirmed(&mut self, connection: &Connection) -> bool {
        if !self.workbench.can_insert_tab() {
            return false;
        }
        let index = self.connections.iter().position(|item| item == connection);
        if let Some(index) = index {
            self.connections_screen.selected = index;
        }
        if connection.outcome != ConnectOutcome::Ok {
            self.status = format!("Connection failed: {}", connection.name);
            self.connections_screen.error = Some("Connection failed; press r to retry".to_owned());
            self.surface = Surface::ConnectionsFailed;
            return false;
        }
        self.safe_mode = connection.safe_mode;
        self.connection = connection.clone();
        self.connections_screen.error = None;
        self.workbench
            .reconnect_confirmed(connection.clone(), self.catalog.clone());
        self.workbench.new_query("");
        self.explorer_tree_state =
            initial_explorer_tree_state(&self.explorer_nodes, self.workbench.current_schema());
        self.tabs_state = TabsState::default();
        self.split_state = SplitPaneState::new(25);
        self.sync_tabs_state();
        self.screen = Screen::Workbench;
        self.surface = Surface::WorkbenchDefault;
        self.workbench_focus = EXPLORER;
        self.status = format!("Connected to {}", connection.name);
        self.status_since = None;
        true
    }

    fn sync_active_table(&mut self) {
        self.sync_active_tab();
    }

    fn sync_tabs_state(&mut self) {
        if let Some(active) = self.workbench.active_index()
            && let Some(key) = self.workbench.tabs().get(active).map(tab_key)
        {
            self.tabs_state.set_active(active, key);
        } else {
            self.tabs_state = TabsState::default();
        }
    }

    /// Select a catalog schema and reconstruct its explorer expansion context.
    pub fn select_schema(&mut self, schema: &str) -> bool {
        if !self.workbench.select_schema(schema) {
            return false;
        }
        self.explorer_nodes = build_explorer_nodes(&self.workbench.catalog);
        reset_explorer_tree_state(&mut self.explorer_tree_state, &self.explorer_nodes, schema);
        self.status = format!("Schema {schema}");
        true
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
        if self.workbench.new_query(query).is_some() {
            self.sync_tabs_state();
            self.surface = Surface::QueryEditing;
        }
    }

    fn commit_query_edit(&mut self) {
        if let Some(Tab::Query(tab)) = self.workbench.active_mut() {
            let _ = tab
                .editor_state
                .commit(&mut tab.query, &termrock::NoValidate);
        }
    }

    fn sync_active_tab(&mut self) {
        self.surface = match self.workbench.active() {
            Some(Tab::Table(tab)) if tab.is_structure() => Surface::StructureView,
            Some(Tab::Table(_)) => Surface::TableGrid,
            Some(Tab::Query(_)) => Surface::QueryEditing,
            Some(Tab::History(_)) => Surface::HistoryTab,
            None => self.surface,
        };
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
        let query = query.into();
        if matches!(self.workbench.active(), Some(Tab::Query(tab)) if tab.has_pending_result()) {
            return QueryOutcome::Rejected {
                message: "Pending result edits require confirmation".to_owned(),
            };
        }
        if !matches!(self.workbench.active(), Some(Tab::Query(_))) {
            if self.workbench.new_query("").is_none() {
                return QueryOutcome::Rejected {
                    message: "No tab identities remain".to_owned(),
                };
            }
            self.sync_active_tab();
        }
        self.set_visual_query(&query);
        self.execute_query()
    }
    /// Parse, gate and execute the current query.
    pub fn execute_query(&mut self) -> QueryOutcome {
        if matches!(self.workbench.active(), Some(Tab::Query(tab)) if tab.has_pending_result()) {
            return QueryOutcome::Rejected {
                message: "Pending result edits require confirmation".to_owned(),
            };
        }
        let Some(key) = self.workbench.active_key() else {
            return QueryOutcome::Rejected {
                message: "Active tab is not a query".to_owned(),
            };
        };
        let query = self.query().to_owned();
        self.execute_snapshot(key, &query)
    }

    fn execute_snapshot(&mut self, key: TabKey, query: &str) -> QueryOutcome {
        if !matches!(self.workbench.tab(key), Some(Tab::Query(_))) {
            return QueryOutcome::Rejected {
                message: "Query tab is no longer available".to_owned(),
            };
        }
        let statement = match sql::parse(query.trim()) {
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
            sql::Statement::Select(select) => {
                self.catalog.find(select.schema.as_deref(), &select.table)
            }
            _ => None,
        };
        match sql::gate(self.safe_mode, &statement) {
            sql::Decision::Deny => {
                let risk = sql::assess(&statement, table);
                let out = QueryOutcome::Denied {
                    summary: format!("{} is denied in Read-Only mode", risk.action),
                };
                self.status = outcome_message(&out);
                out
            }
            sql::Decision::Confirm { deliberate } => {
                let risk = sql::assess(&statement, table);
                let out = QueryOutcome::ConfirmationRequired {
                    deliberate,
                    summary: format!("{} · {}", risk.action, risk.scope),
                };
                self.status = outcome_message(&out);
                out
            }
            sql::Decision::Run => {
                if let sql::Statement::Select(select) = statement {
                    match tablepro_demo::run_select(&self.catalog, &select) {
                        Ok(result) => {
                            let out = QueryOutcome::Executed {
                                rows: result.rows.len(),
                                editable: result.editable,
                            };
                            if let Some(Tab::Query(tab)) = self.workbench.tab_mut(key) {
                                tab.result = Some(GridView::from_result(&result));
                                tab.last_duration = Some(15);
                                if tab.editor_state.draft_text() == Some(query) {
                                    let _ = tab
                                        .editor_state
                                        .commit(&mut tab.query, &termrock::NoValidate);
                                }
                            }
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
                } else if let sql::Statement::Update { .. } = statement {
                    if let Some(Tab::Query(tab)) = self.workbench.tab_mut(key) {
                        tab.affected = Some((8022, "UPDATE orders".to_owned()));
                        tab.last_duration = Some(42);
                        tab.result = None;
                        if tab.editor_state.draft_text() == Some(query) {
                            let _ = tab
                                .editor_state
                                .commit(&mut tab.query, &termrock::NoValidate);
                        }
                    }
                    let out = QueryOutcome::Executed {
                        rows: 8022,
                        editable: false,
                    };
                    self.status = "UPDATE orders · 8022 rows affected · 42 ms".to_owned();
                    out
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
    fn column_specs<'a>(
        columns: &'a [(String, ColType)],
        editable: bool,
        table: Option<&Table>,
        filters: &[Filter],
    ) -> ([termrock::Column<'a>; termrock::GRID_MAX_COLUMNS], usize) {
        crate::tabs::column_specs(columns, editable, table, filters)
    }
    fn connection_form<'a>(
        fields: &'a [termrock::FieldSpec<'a>],
        actions: &'a [Action<'a>],
    ) -> Form<'a> {
        Form::new(connections::FORM, fields)
            .actions(actions)
            .submit(connections::SAVE_CONNECT)
    }
    fn handle_grid(status: &mut String, action: &GridAction) {
        match action {
            GridAction::Sort(key, _) => {
                *status = format!("Sorted column {}", key.raw());
            }
            GridAction::Copy(text) => {
                *status = format!("Copied {} cells", text.lines().count());
            }
            GridAction::Activated(key) => *status = format!("Activated row {key:?}"),
            GridAction::EditRequested(key, column) => {
                *status = format!("Edit requested for {key:?}, column {column:?}");
            }
            GridAction::CellAction(key, column, action) => {
                *status = format!("Cell action {action:?} on {key:?}/{column:?}");
            }
            GridAction::FetchMore => {
                "All deterministic demo rows are loaded".clone_into(status);
            }
            GridAction::Moved | GridAction::LeaveForward | GridAction::LeaveBackward => {}
        }
    }

    fn request_quit(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.quit || cx.top_layer() != LayerId::PAGE {
            return Response::consumed();
        }
        let mut pending: usize = 0;
        let mut dirty_queries: usize = 0;
        for record in self.workbench.tabs() {
            match record.payload() {
                Tab::Table(tab) => pending = pending.saturating_add(tab.result.pending_total()),
                Tab::Query(tab) => {
                    dirty_queries = dirty_queries.saturating_add(usize::from(tab.dirty()));
                    if let Some(grid) = &tab.result {
                        pending = pending.saturating_add(grid.pending_total());
                    }
                }
                Tab::History(_) => {}
            }
        }
        if pending == 0 && dirty_queries == 0 {
            self.quit = true;
            cx.quit();
            return Response::consumed();
        }
        let mut parts = Vec::with_capacity(2);
        if pending > 0 {
            parts.push(format!(
                "{pending} pending row change{}",
                if pending == 1 { "" } else { "s" }
            ));
        }
        if dirty_queries > 0 {
            parts.push(format!(
                "{dirty_queries} unsaved quer{}",
                if dirty_queries == 1 { "y" } else { "ies" }
            ));
        }
        let question = format!("{} will be lost.", parts.join(" and "));
        let Some(scope) = self.workbench.destructive_scope() else {
            self.stale_destructive();
            return Response::changed();
        };
        self.open_destructive(cx, DestructiveIntent::Quit { question, scope });
        Response::changed()
    }

    fn open_destructive(&mut self, cx: &mut Cx<'_>, intent: DestructiveIntent) {
        self.destructive_notice = None;
        cx.open_layer(QUIT_DIALOG, intent.dialog().layer(cx));
        self.destructive_intent = Some(DestructiveRequest {
            intent,
            owner: self.workbench.owner_token(),
        });
    }

    fn request_connect(&mut self, cx: &mut Cx<'_>, index: usize) {
        let Some(target) = self.connections.get(index).cloned() else {
            return;
        };
        if self.workbench.has_unsaved_work() {
            let Some(scope) = self.workbench.destructive_scope() else {
                self.stale_destructive();
                return;
            };
            self.open_destructive(
                cx,
                DestructiveIntent::Reconnect {
                    target: Box::new(target),
                    source: Box::new(self.workbench.connection.clone()),
                    scope,
                },
            );
        } else {
            let _ = self.connect_confirmed(&target);
        }
    }

    fn request_query(&mut self, cx: &mut Cx<'_>) {
        let Some(key) = self.workbench.active_key() else {
            return;
        };
        let Some(Tab::Query(tab)) = self.workbench.tab(key) else {
            return;
        };
        if tab.has_pending_result() {
            let Some(generation) = self.workbench.generation(key) else {
                self.stale_destructive();
                return;
            };
            let query = tab
                .editor_state
                .draft_text()
                .unwrap_or(&tab.query)
                .to_owned();
            self.open_destructive(
                cx,
                DestructiveIntent::ReplaceResult {
                    key,
                    query,
                    generation,
                    connection: Box::new(self.workbench.connection.clone()),
                },
            );
            return;
        }

        self.commit_query_edit();
        let query_text = self.query().to_owned();
        let statement = match sql::parse(query_text.trim()) {
            Ok(statement) => statement,
            Err(_) => {
                let _ = self.execute_query();
                return;
            }
        };

        let table = match &statement {
            sql::Statement::Select(select) => {
                self.catalog.find(select.schema.as_deref(), &select.table)
            }
            sql::Statement::Update { table, .. }
            | sql::Statement::Delete { table, .. }
            | sql::Statement::Insert { table }
            | sql::Statement::Truncate { table }
            | sql::Statement::Alter { table, .. } => {
                let (schema, name) = match table.split_once('.') {
                    Some((s, n)) => (Some(s), n),
                    None => (None, table.as_str()),
                };
                self.catalog.find(schema, name)
            }
            sql::Statement::Drop { kind, name } if kind.eq_ignore_ascii_case("TABLE") => {
                let (schema, n) = match name.split_once('.') {
                    Some((s, n)) => (Some(s), n),
                    None => (None, name.as_str()),
                };
                self.catalog.find(schema, n)
            }
            _ => None,
        };

        match sql::gate(self.safe_mode, &statement) {
            sql::Decision::Confirm { deliberate } => {
                let risk = sql::assess(&statement, table);
                let table_name = match &statement {
                    sql::Statement::Update { table, .. }
                    | sql::Statement::Delete { table, .. }
                    | sql::Statement::Insert { table }
                    | sql::Statement::Truncate { table }
                    | sql::Statement::Alter { table, .. } => table.clone(),
                    _ => "orders".to_owned(),
                };
                let token = if deliberate {
                    Some(table_name.clone())
                } else {
                    None
                };
                let title = if risk.dangerous {
                    "This query may permanently modify or delete data"
                } else {
                    "Execute write query?"
                };
                let mut facts = vec![
                    Prop::new("Action", risk.action)
                        .tone(if risk.dangerous { Tone::Error } else { Tone::Normal }),
                    Prop::new(
                        "Target",
                        format!(
                            "{} · {} · {} · {}",
                            self.connection.name,
                            self.connection.environment.label().to_lowercase(),
                            self.catalog.database,
                            table_name
                        ),
                    ),
                    Prop::new("Scope", risk.scope).tone(Tone::Secondary).wrap(),
                ];
                if !risk.summary.is_empty() {
                    facts.push(Prop::new("Risk", risk.summary).wrap().tone(Tone::Warning));
                }
                facts.push(
                    Prop::new("Reversible", risk.reversible)
                        .tone(Tone::Secondary)
                        .wrap(),
                );
                if deliberate {
                    facts.push(
                        Prop::new(
                            "Safe Mode",
                            format!(
                                "{} · deliberate confirmation required",
                                self.safe_mode.label()
                            ),
                        )
                        .tone(Tone::Muted),
                    );
                }
                let (cols, rows) = self.screen_size.get();
                let cols = if cols == 0 { 120 } else { cols };
                let rows = if rows == 0 { 40 } else { rows };
                let dialog_w = 74.min(cols.saturating_sub(4)).max(20);
                let dialog = SafetyDialog::new(
                    title,
                    facts,
                    vec![query_text.trim().to_owned()],
                    token,
                    "Execute",
                    false,
                    dialog_w,
                    SafetyIntent::Query,
                );
                let dialog_h = dialog.height().min(rows.saturating_sub(2));
                let mut spec = LayerSpec::modal(SAFETY_DIALOG);
                spec.size = LayerSize::Fixed(dialog_w, dialog_h);
                let focus_id = if dialog.token.is_some() {
                    SAFETY_INPUT
                } else {
                    SAFETY_CONFIRM
                };
                spec.initial_focus = Some(focus_id);
                spec.restore_focus = true;
                cx.open_layer(SAFETY_DIALOG, spec);
                cx.focus(focus_id);
                self.safety_dialog = Some(dialog);
            }
            _ => {
                let _ = self.execute_query();
            }
        }
    }

    fn request_save(&mut self, cx: &mut Cx<'_>) {
        let Some((_, grid)) = self.workbench.active_grid() else {
            return;
        };
        if grid.model.pending_total() == 0 {
            return;
        }
        let Some(table_tab) = (match self.workbench.active() {
            Some(Tab::Table(t)) => Some(t),
            _ => None,
        }) else {
            return;
        };

        let schema = &table_tab.table.schema;
        let table = &table_tab.table.name;
        let target = format!(
            "{} · {} · {} · {}.{}",
            self.connection.name,
            self.connection.environment.label().to_lowercase(),
            self.catalog.database,
            schema,
            table
        );
        let statements = table_tab.preview();
        let updates = statements
            .iter()
            .filter(|s| s.starts_with("UPDATE"))
            .count();
        let inserts = statements
            .iter()
            .filter(|s| s.starts_with("INSERT"))
            .count();
        let deletes = statements
            .iter()
            .filter(|s| s.starts_with("DELETE"))
            .count();
        let scope = format!(
            "{updates} update{} · {inserts} insert{} · {deletes} delete{}",
            if updates == 1 { "" } else { "s" },
            if inserts == 1 { "" } else { "s" },
            if deletes == 1 { "" } else { "s" },
        );
        let deliberate = self.safe_mode.requires_authentication();
        let facts = vec![
            Prop::new("Action", "Save changes"),
            Prop::new("Target", target),
            Prop::new("Scope", scope).tone(Tone::Secondary),
            Prop::new(
                "Transaction",
                "All statements run in one transaction; a failure rolls everything back.",
            )
            .tone(Tone::Muted)
            .wrap(),
            Prop::new(
                "Safe Mode",
                format!(
                    "{} · {}",
                    self.safe_mode.label(),
                    if deliberate {
                        "deliberate confirmation required"
                    } else if self.safe_mode.requires_confirmation() || deletes > 0 {
                        "confirmation required"
                    } else {
                        "runs after this review"
                    }
                ),
            )
            .tone(Tone::Muted),
        ];

        let width = 78u16;
        let code: Vec<String> = statements
            .iter()
            .map(|s| s.trim_end_matches(';').to_owned())
            .collect();

        let token = if deliberate {
            Some(table.clone())
        } else {
            None
        };

        let (cols, rows) = self.screen_size.get();
        let cols = if cols == 0 { 120 } else { cols };
        let rows = if rows == 0 { 40 } else { rows };
        let dialog_w = width.min(cols.saturating_sub(4)).max(20);
        let dialog = SafetyDialog::new(
            "Save changes?",
            facts,
            code,
            token,
            "Save",
            false,
            dialog_w,
            SafetyIntent::Commit,
        );
        let dialog_h = dialog.height().min(rows.saturating_sub(2));
        let mut spec = LayerSpec::modal(SAFETY_DIALOG);
        spec.size = LayerSize::Fixed(dialog_w, dialog_h);
        let focus_id = if dialog.token.is_some() {
            SAFETY_INPUT
        } else {
            SAFETY_CONFIRM
        };
        spec.initial_focus = Some(focus_id);
        spec.restore_focus = true;
        cx.open_layer(SAFETY_DIALOG, spec);
        cx.focus(focus_id);
        self.safety_dialog = Some(dialog);
    }

    fn request_close_tab(&mut self, cx: &mut Cx<'_>, key: TabKey) {
        let Some(tab) = self.workbench.tabs().iter().find(|tab| tab.key() == key) else {
            return;
        };
        if tab.dirty() {
            let Some(generation) = self.workbench.generation(key) else {
                self.stale_destructive();
                return;
            };
            self.open_destructive(cx, DestructiveIntent::CloseTab { key, generation });
        } else {
            let _ = self.workbench.close_tab_confirmed(key);
            self.sync_active_tab();
        }
    }

    fn active_row_action(&self, cx: &Cx<'_>) -> bool {
        self.workbench.active_grid().is_some_and(|(id, grid)| {
            cx.state(id).contains(termrock::StateFlags::FOCUSED)
                && !grid.state.is_editing()
                && grid.model.is_editable()
        })
    }

    pub fn is_editing(&self) -> bool {
        match self.screen {
            Screen::Connections => self.connections_screen.filter_active || self.form_editing,
            Screen::Workbench => {
                if let Some(tab) = self.workbench.active() {
                    match tab {
                        Tab::Table(t) => {
                            t.result.state.is_editing() || t.structure.state.is_editing()
                        }
                        Tab::Query(q) => {
                            q.editor_state.is_editing()
                                || q.result
                                    .as_ref()
                                    .is_some_and(|r| r.state.is_editing())
                        }
                        Tab::History(_) => false,
                    }
                } else {
                    false
                }
            }
        }
    }

    fn insert_defaults(&self) -> Option<Vec<bool>> {
        let (_, grid) = self.workbench.active_grid()?;
        let source = grid.model.source()?;
        let (schema, name) = source.split_once('.')?;
        let table = self.catalog.find(Some(schema), name)?;
        grid.columns
            .iter()
            .map(|(name, _)| {
                table
                    .columns
                    .iter()
                    .find(|column| column.name == *name)
                    .map(|column| column.primary || column.generated)
            })
            .collect()
    }

    fn insert_active_row(&mut self) {
        let Some(defaults) = self.insert_defaults() else {
            return;
        };
        let column = defaults.iter().position(|default| !default).unwrap_or(0);
        let Some((id, grid)) = self.workbench.active_grid_mut() else {
            return;
        };
        let (columns, count) =
            Self::column_specs(&grid.columns, grid.model.is_editable(), None, &[]);
        let Some(target) = columns
            .get(column)
            .filter(|_| column < count)
            .map(|column| column.key)
        else {
            return;
        };
        let Some(row) = grid.model.insert_row_with_defaults(&defaults) else {
            return;
        };
        let key = grid.model.row_key(row);
        if result_grid(id, columns.get(..count).unwrap_or(&[]))
            .move_cursor_to(&mut grid.state, &grid.model, key, target)
            .is_err()
        {
            let _ = grid.model.undo();
        }
    }

    fn duplicate_active_row(&mut self) {
        let Some(defaults) = self.insert_defaults() else {
            return;
        };
        let Some((id, grid)) = self.workbench.active_grid_mut() else {
            return;
        };
        let Some((cursor_key, cursor_col)) = grid.state.cursor() else {
            return;
        };
        let Some(src_row) = (0..grid.model.row_count()).find(|r| grid.model.row_key(*r) == cursor_key) else {
            return;
        };
        let prev_viewport = grid.state.scroll().viewport_len();
        let Some(new_row) = grid.model.duplicate_row(src_row, &defaults) else {
            return;
        };
        let new_key = grid.model.row_key(new_row);
        let (columns, count) = Self::column_specs(&grid.columns, grid.model.is_editable(), None, &[]);
        let _ = result_grid(id, columns.get(..count).unwrap_or(&[]))
            .move_cursor_to(&mut grid.state, &grid.model, new_key, cursor_col);
        let scroll = grid.state.scroll_mut();
        scroll.set_content(grid.model.row_count());
        scroll.scroll_to(new_row.saturating_add(1).saturating_sub(prev_viewport));
        scroll.clear_reveal();
    }

    fn toggle_active_row(&mut self) {
        let Some((_, grid)) = self.workbench.active_grid_mut() else {
            return;
        };
        let Some((key, _)) = grid.state.cursor() else {
            return;
        };
        let Some(row) = (0..grid.model.row_count()).find(|row| grid.model.row_key(*row) == key)
        else {
            return;
        };
        let _ = grid.model.toggle_delete(row);
    }

    fn request_discard_rows(&mut self, cx: &mut Cx<'_>) {
        let Some(key) = self.workbench.active_key() else {
            return;
        };
        let Some((_, grid)) = self.workbench.active_grid() else {
            return;
        };
        let pending = grid.pending_total();
        if pending == 0 {
            return;
        }
        let Some(generation) = self.workbench.generation(key) else {
            self.stale_destructive();
            return;
        };
        let question = format!(
            "{pending} pending change(s) will be dropped. The rows are reloaded from the server."
        );
        self.open_destructive(
            cx,
            DestructiveIntent::DiscardRows {
                key,
                generation,
                question,
            },
        );
    }

    fn screen_dimensions(&self, cx: &Cx<'_>) -> (u16, u16) {
        let vp = cx.viewport();
        if !vp.is_empty() {
            (vp.width, vp.height)
        } else {
            self.screen_size.get()
        }
    }

    fn open_switcher(&mut self, cx: &mut Cx<'_>) {
        let (cols, rows) = self.screen_dimensions(cx);
        self.switcher.open(&self.workbench);
        self.switcher_open = true;
        cx.open_layer(
            quick_switcher::ID,
            self.switcher.component(cols, rows).layer(cx, &self.switcher.items),
        );
        self.surface = Surface::QuickSwitcher;
    }

    fn update_switcher(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let was_open = cx.is_open(quick_switcher::ID);
        self.switcher_open = was_open;
        let (cols, rows) = self.screen_dimensions(cx);
        let mut response =
            self.switcher
                .component(cols, rows)
                .update(cx, &mut self.switcher.state, &self.switcher.items);
        match response.take_action() {
            Some(PickerAction::QueryChanged | PickerAction::Scope(_)) if was_open => {
                self.switcher.refresh();
            }
            Some(PickerAction::Chosen(key) | PickerAction::ChosenAlt(key)) if was_open => {
                let target = self
                    .switcher
                    .items
                    .iter()
                    .find(|item| ItemKey::text(&item.key) == key)
                    .map(|item| item.target.clone());
                cx.close_layer(quick_switcher::ID, None);
                self.switcher_open = false;
                self.sync_active_tab();
                if self.workbench.matches_owner(&self.switcher.owner) {
                    if let Some(target) = target {
                        self.choose_switch_target(cx, target);
                    }
                } else {
                    "Workbench changed; reopen switcher".clone_into(&mut self.status);
                }
            }
            _ => {}
        }
        if was_open && !cx.is_open(quick_switcher::ID) {
            self.switcher_open = false;
            if self.surface == Surface::QuickSwitcher {
                self.sync_active_tab();
            }
        }
        response.erase()
    }

    fn open_tab_list(&mut self, cx: &mut Cx<'_>) {
        let (cols, rows) = self.screen_dimensions(cx);
        self.tab_list.open(&self.workbench);
        self.tab_list_open = true;
        cx.open_layer(
            tab_list::ID,
            self.tab_list.component(cols, rows).layer(cx, &self.tab_list.items),
        );
        self.surface = Surface::TabListPicker;
    }

    fn update_tab_list(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let was_open = cx.is_open(tab_list::ID);
        if !was_open {
            self.tab_list_open = false;
            return Response::ignored();
        }
        self.tab_list_open = true;
        let (cols, rows) = self.screen_dimensions(cx);
        let mut response = self
            .tab_list
            .component(cols, rows)
            .update(cx, &mut self.tab_list.state, &self.tab_list.items);

        match response.take_action() {
            Some(PickerAction::Chosen(key) | PickerAction::ChosenAlt(key)) => {
                if let Some(item) = self
                    .tab_list
                    .items
                    .iter()
                    .find(|it| ItemKey::text(&it.key) == key)
                {
                    let tab_key = item.tab_key;
                    cx.close_layer(tab_list::ID, None);
                    self.tab_list_open = false;
                    let _ = self.workbench.activate(tab_key);
                    self.sync_active_tab();
                    if let Some(tab_key) = self.workbench.active_key() {
                        if let Some(Tab::Table(_)) = self.workbench.active() {
                            let ctrl = tab_key.control("data");
                            self.workbench_focus = ctrl;
                            cx.focus(ctrl);
                        } else if let Some(focus) = self.query_id().or_else(|| self.result_id()) {
                            self.workbench_focus = focus;
                            cx.focus(focus);
                        }
                    }
                }
            }
            Some(PickerAction::Secondary(key)) => {
                if let Some(item) = self
                    .tab_list
                    .items
                    .iter()
                    .find(|it| ItemKey::text(&it.key) == key)
                {
                    let tab_key = item.tab_key;
                    self.request_close_tab(cx, tab_key);
                    if self.workbench.tabs().is_empty() {
                        cx.close_layer(tab_list::ID, None);
                        self.tab_list_open = false;
                    } else {
                        self.tab_list.open(&self.workbench);
                        cx.resize_layer(
                            tab_list::ID,
                            self.tab_list.component(cols, rows).measured_size(cx, &self.tab_list.items),
                        );
                    }
                }
            }
            _ => {}
        }
        if was_open && !cx.is_open(tab_list::ID) {
            self.tab_list_open = false;
            if self.surface == Surface::TabListPicker {
                self.sync_active_tab();
            }
        }
        response.erase()
    }

    fn open_safe_mode_picker(&mut self, cx: &mut Cx<'_>) {
        let (cols, rows) = self.screen_dimensions(cx);
        self.safe_mode_picker.open(self.safe_mode);
        self.safe_mode_open = true;
        cx.open_layer(
            safe_mode_picker::ID,
            self.safe_mode_picker.component(cols, rows).layer(cx, &self.safe_mode_picker.items),
        );
        self.surface = Surface::SafeModePicker;
    }

    fn update_safe_mode_picker(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let was_open = cx.is_open(safe_mode_picker::ID);
        if !was_open {
            self.safe_mode_open = false;
            return Response::ignored();
        }
        self.safe_mode_open = true;
        let (cols, rows) = self.screen_dimensions(cx);
        let mut response = self
            .safe_mode_picker
            .component(cols, rows)
            .update(cx, &mut self.safe_mode_picker.state, &self.safe_mode_picker.items);

        if let Some(PickerAction::Chosen(key) | PickerAction::ChosenAlt(key)) =
            response.take_action()
            && let ItemKey::Index(idx) = key
            && let Some(&mode) = SafeMode::ALL.get(idx)
        {
            self.safe_mode = mode;
            self.connection.safe_mode = mode;
            self.workbench.connection.safe_mode = mode;
            cx.close_layer(safe_mode_picker::ID, None);
            self.safe_mode_open = false;
            self.status = format!("Safe mode set to {}", mode.label());
        }
        if was_open && !cx.is_open(safe_mode_picker::ID) {
            self.safe_mode_open = false;
            if self.surface == Surface::SafeModePicker {
                self.sync_active_tab();
            }
        }
        response.erase()
    }

    fn open_help(&mut self, cx: &mut Cx<'_>) {
        self.help_open = true;
        self.help_dialog_state = DialogState::default();
        let (cols, rows) = self.screen_dimensions(cx);
        let dlg = help_dialog(cols, rows);
        let spec = dlg.layer(cx);
        cx.open_layer(HELP_DIALOG_ID, spec);
        if let Some(initial) = dlg.initial_focus() {
            cx.focus(initial);
        }
        self.surface = Surface::HelpDialog;
    }

    fn update_help_dialog(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let was_open = cx.is_open(HELP_DIALOG_ID);
        if !was_open {
            self.help_open = false;
            return Response::ignored();
        }
        self.help_open = true;
        let (cols, rows) = self.screen_dimensions(cx);
        let response = help_dialog(cols, rows).update(cx, &mut self.help_dialog_state);
        if let Some(action) = response.action_ref() {
            match action {
                DialogAction::Dismissed(_) | DialogAction::Action(_) => {
                    cx.close_layer(HELP_DIALOG_ID, None);
                    self.help_open = false;
                    if self.surface == Surface::HelpDialog {
                        self.sync_active_tab();
                    }
                }
            }
        }
        if was_open && !cx.is_open(HELP_DIALOG_ID) {
            self.help_open = false;
            if self.surface == Surface::HelpDialog {
                self.sync_active_tab();
            }
        }
        response.erase()
    }

    fn choose_switch_target(&mut self, cx: &mut Cx<'_>, target: SwitchTarget) {
        let changed = match target {
            SwitchTarget::Table { schema, name } | SwitchTarget::View { schema, name } => {
                let opened = self.workbench.open_table_in_schema(&schema, &name);
                if opened {
                    self.status = format!("Opened {schema}.{name}");
                }
                opened
            }
            SwitchTarget::OpenTab(key) => self.workbench.activate(key),
            SwitchTarget::Query(id) => {
                let sql = self
                    .workbench
                    .history
                    .entries
                    .iter()
                    .find(|entry| entry.id == id)
                    .map(|entry| entry.sql.clone());
                sql.is_some_and(|sql| self.workbench.new_query(sql).is_some())
            }
            SwitchTarget::Schema(schema) => {
                if self.select_schema(&schema) {
                    self.workbench_focus = EXPLORER;
                    cx.focus(EXPLORER);
                }
                return;
            }
            SwitchTarget::Database(name) => {
                if self.workbench.catalog.database == name {
                    self.workbench_focus = EXPLORER;
                    cx.focus(EXPLORER);
                }
                return;
            }
            SwitchTarget::Connection(_) => false,
        };
        if changed {
            self.sync_active_tab();
            if let Some(tab_key) = self.workbench.active_key() {
                if let Some(Tab::Table(_)) = self.workbench.active() {
                    let ctrl = tab_key.control("data");
                    self.workbench_focus = ctrl;
                    cx.focus(ctrl);
                } else if let Some(focus) = self.query_id().or_else(|| self.result_id()) {
                    self.workbench_focus = focus;
                    cx.focus(focus);
                }
            } else if let Some(focus) = self.query_id().or_else(|| self.result_id()) {
                self.workbench_focus = focus;
                cx.focus(focus);
            }
        } else {
            "Target unavailable; reopen switcher".clone_into(&mut self.status);
            if let Some(focus) = self.query_id().or_else(|| self.result_id()) {
                self.workbench_focus = focus;
                cx.focus(focus);
            } else {
                self.workbench_focus = EXPLORER;
                cx.focus(EXPLORER);
            }
        }
    }

    fn stale_destructive(&mut self) {
        self.destructive_notice = Some("Work changed; request again");
    }

    fn update_destructive_dialog(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        // Keep the same control identity alive for late focus lifecycle events.
        let fallback = DestructiveIntent::Quit {
            question: String::new(),
            scope: Vec::new(),
        };
        let intent = self
            .destructive_intent
            .as_ref()
            .map_or(&fallback, |request| &request.intent);
        let dlg = intent.dialog();
        let dialog_response = dlg.update(cx, &mut self.quit_state);
        let mut action = dialog_response.action_ref().copied();
        if action.is_none() && self.destructive_intent.is_some() {
            for id in [QUIT_DIALOG, dlg.action_id(0), dlg.action_id(1)] {
                for it in cx.intents(id) {
                    if let Intent::Key(key) = it
                        && key.mods.is_empty() {
                            if key.code == KeyCode::Char('y') {
                                action = Some(DialogAction::Action(ActionKey::CONFIRM));
                            } else if key.code == KeyCode::Char('n') {
                                action = Some(DialogAction::Action(ActionKey::CANCEL));
                            }
                        }
                }
            }
        }
        let mut response = dialog_response.erase();
        if let Some(action) = action {
            response |= Response::changed();
            let confirmed = matches!(action, DialogAction::Action(ActionKey::CONFIRM))
                && cx.is_open(QUIT_DIALOG);
            cx.close_layer(QUIT_DIALOG, None);
            if let Some(request) = self.destructive_intent.take()
                && confirmed
            {
                if !matches!(request.intent, DestructiveIntent::DeleteConnection { .. })
                    && !self.workbench.matches_owner(&request.owner)
                {
                    self.stale_destructive();
                    return response.erase();
                }
                match request.intent {
                    DestructiveIntent::DeleteConnection { index, .. } => {
                        if index < self.connections.len() {
                            self.connections.remove(index);
                            if index < self.connections_screen.connections.len() {
                                self.connections_screen.connections.remove(index);
                            }
                            self.connections_screen.selected = if self.connections.is_empty() {
                                0
                            } else {
                                index.min(self.connections.len() - 1)
                            };
                            self.rebuild_connection_nodes();
                        }
                    }
                    DestructiveIntent::Quit { scope, .. } => {
                        if self.workbench.matches_scope(&scope) {
                            if !self.quit {
                                self.quit = true;
                                cx.quit();
                            }
                        } else {
                            self.stale_destructive();
                        }
                    }
                    DestructiveIntent::CloseTab { key, generation } => {
                        if self.workbench.generation(key) != Some(generation) {
                            self.stale_destructive();
                        } else if self.workbench.close_tab_confirmed(key) {
                            self.sync_active_tab();
                            let focus = self
                                .query_id()
                                .or_else(|| self.result_id())
                                .unwrap_or(EXPLORER);
                            cx.focus(focus);
                        }
                    }
                    DestructiveIntent::Reconnect {
                        target,
                        source,
                        scope,
                    } => {
                        if self.workbench.connection == *source
                            && self.workbench.matches_scope(&scope)
                        {
                            let _ = self.connect_confirmed(&target);
                        } else {
                            self.stale_destructive();
                        }
                    }
                    DestructiveIntent::DiscardRows {
                        key, generation, ..
                    } => {
                        if self.workbench.generation(key) != Some(generation) {
                            self.stale_destructive();
                        } else if let Some(tab) = self.workbench.tab_mut(key)
                            && let Some((_, grid)) = tab.grid_mut(key)
                        {
                            let _ = grid.state.cancel_edit();
                            grid.model.discard();
                        }
                    }
                    DestructiveIntent::ReplaceResult {
                        key,
                        query,
                        generation,
                        connection,
                    } => {
                        if self.workbench.connection == *connection
                            && self.workbench.generation(key) == Some(generation)
                        {
                            let _ = self.execute_snapshot(key, &query);
                        } else {
                            self.stale_destructive();
                        }
                    }
                }
            }
        }
        response.erase()
    }

    fn update_safety_dialog(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let Some(dialog) = self.safety_dialog.as_mut() else {
            return Response::ignored();
        };
        if !cx.is_open(SAFETY_DIALOG) {
            let status = match dialog.intent {
                SafetyIntent::Query => "Cancelled · nothing was executed",
                SafetyIntent::Commit => "Changes kept pending",
            };
            self.safety_dialog = None;
            status.clone_into(&mut self.status);
            self.status_since = Some(cx.now());
            cx.focus(CONTENT_FRAME);
            return Response::changed();
        }
        let mut action = None;
        let mut key_received = false;
        let mut focus_changed = false;
        for (id, focus) in [
            (SAFETY_INPUT, SafetyFocus::Input),
            (SAFETY_CANCEL, SafetyFocus::Cancel),
            (SAFETY_CONFIRM, SafetyFocus::Confirm),
        ] {
            for intent in cx.intents(id) {
                if let Intent::FocusIn { .. } = intent {
                    dialog.focus = focus;
                    focus_changed = true;
                }
            }
        }
        let btn_cancel = termrock::Button::new(SAFETY_CANCEL, "Cancel");
        if btn_cancel.update(cx).activated() {
            action = Some(SafetyDialogAction::Cancel);
        }
        let btn_confirm = termrock::Button::new(SAFETY_CONFIRM, &dialog.confirm_label)
            .disabled(!dialog.armed());
        if btn_confirm.update(cx).activated() && dialog.armed() {
            action = Some(SafetyDialogAction::Confirm);
        }
        if action.is_none() {
            for intent in cx
                .intents(SAFETY_INPUT)
                .chain(cx.intents(SAFETY_CANCEL))
                .chain(cx.intents(SAFETY_CONFIRM))
                .chain(cx.intents(SAFETY_DIALOG))
            {
                if let Intent::Key(key) = intent {
                    key_received = true;
                    if let Some(act) = dialog.on_key(key) {
                        action = Some(act);
                        break;
                    }
                }
            }
        }
        if let Some(act) = action {
            match act {
                SafetyDialogAction::Cancel => {
                    let intent = dialog.intent;
                    let status = match intent {
                        SafetyIntent::Query => "Cancelled · nothing was executed",
                        SafetyIntent::Commit => "Changes kept pending",
                    };
                    self.safety_dialog = None;
                    cx.close_layer(SAFETY_DIALOG, None);
                    status.clone_into(&mut self.status);
                    self.status_since = Some(cx.now());
                    if intent == SafetyIntent::Query
                        && let Some(key) = self.workbench.active_key()
                    {
                        let query_id = key.control("query");
                        cx.focus(query_id);
                        self.workbench_focus = query_id;
                    }
                    return Response::changed();
                }
                SafetyDialogAction::Confirm => {
                    let intent = dialog.intent;
                    self.safety_dialog = None;
                    cx.close_layer(SAFETY_DIALOG, None);
                    match intent {
                        SafetyIntent::Query => {
                            if let Some(key) = self.workbench.active_key() {
                                if let Some(Tab::Query(tab)) = self.workbench.tab_mut(key) {
                                    tab.affected = Some((8022, "UPDATE orders".to_owned()));
                                    tab.last_duration = Some(42);
                                    tab.result = None;
                                }
                                let query_id = key.control("query");
                                cx.focus(query_id);
                                self.workbench_focus = query_id;
                            }
                        }
                        SafetyIntent::Commit => {
                            self.committing = Some(4);
                            self.status = "Saving…".to_owned();
                            self.status_since = Some(cx.now());
                            cx.request_repaint_after(std::time::Duration::from_millis(80));
                        }
                    }
                    return Response::changed();
                }
            }
        }
        match dialog.focus {
            SafetyFocus::Input => cx.focus(SAFETY_INPUT),
            SafetyFocus::Cancel => cx.focus(SAFETY_CANCEL),
            SafetyFocus::Confirm => cx.focus(SAFETY_CONFIRM),
        }
        if key_received || focus_changed {
            Response::changed()
        } else {
            Response::ignored()
        }
    }

    fn finish_commit(&mut self, cx: &mut Cx<'_>) {
        if let Some(Tab::Table(t)) = self.workbench.active_mut() {
            let n = t.result.model.pending_total();
            let qualified = format!("{}.{}", t.table.schema, t.table.name);
            t.result.model.commit();
            self.status = format!(
                "Saved {n} change{} to {qualified}",
                if n == 1 { "" } else { "s" }
            );
            self.status_since = Some(cx.now());
        }
    }

    fn open_filter_editor(
        &mut self,
        cx: &mut Cx<'_>,
        index: Option<usize>,
        prefill: Option<(usize, FilterOp, String)>,
    ) {
        let Some(Tab::Table(t)) = self.workbench.active() else {
            return;
        };
        let columns = t.result.columns.clone();
        let existing = index.and_then(|i| t.filters.get(i)).cloned();
        let cursor_col = t
            .result
            .state
            .cursor()
            .map(|(_, col)| usize::from(col.raw()).saturating_sub(1))
            .unwrap_or(0);
        let actual_prefill = match (&existing, &prefill) {
            (Some(f), _) => {
                let col_i = columns.iter().position(|c| c.0 == f.column).unwrap_or(0);
                Some((col_i, f.op, f.value.clone()))
            }
            (None, Some(p)) => Some(p.clone()),
            _ => None,
        };
        let initial_focus = if actual_prefill.as_ref().is_some_and(|(_, _, v)| !v.is_empty()) {
            FILTER_APPLY
        } else {
            FILTER_VALUE
        };
        let editor = FilterEditor::new(columns, index, actual_prefill, cursor_col);
        let mut spec = LayerSpec::modal(FILTER_EDITOR);
        spec.size = LayerSize::Fixed(64, 15);
        spec.anchor = termrock::Anchor::Screen(termrock::ScreenAlign::UpperThird);
        spec.initial_focus = Some(initial_focus);
        spec.restore_focus = true;
        cx.open_layer(FILTER_EDITOR, spec);
        cx.focus(initial_focus);
        self.surface = Surface::FilterEditor;
        self.filter_editor = Some(editor);
    }

    fn drain_filter_intents(cx: &Cx<'_>) {
        for id in [
            FILTER_COL,
            FILTER_OP,
            FILTER_VALUE,
            FILTER_VALUE2,
            FILTER_CANCEL,
            FILTER_APPLY,
            FILTER_EDITOR,
        ] {
            let _ = cx.intents(id).count();
        }
    }

    fn refocus_content(&mut self, cx: &mut Cx<'_>) {
        if let Some(tab_key) = self.workbench.active_key() {
            let pf = match self.workbench.active() {
                Some(Tab::Table(t)) if t.is_structure() => Some(tab_key.control("structure")),
                Some(Tab::Table(_)) => Some(tab_key.control("data")),
                Some(Tab::Query(_)) => Some(tab_key.control("query")),
                Some(Tab::History(_)) => Some(tab_key.control("history")),
                None => None,
            };
            if let Some(pf) = pf {
                cx.focus(pf);
            } else {
                cx.focus(CONTENT_FRAME);
            }
        } else {
            cx.focus(CONTENT_FRAME);
        }
    }

    fn update_filter_editor(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let Some(editor) = self.filter_editor.as_mut() else {
            Self::drain_filter_intents(cx);
            return Response::ignored();
        };
        if !cx.is_open(FILTER_EDITOR) {
            Self::drain_filter_intents(cx);
            self.filter_editor = None;
            return Response::changed();
        }
        // Value fields are owned by TextInput. Blur (FocusOut) commits the
        // draft into `editor.value` / `editor.value2` without applying;
        // Enter commits and applies. Esc while editing cancels the draft;
        // Esc while idle closes. Manual routing below always runs so every
        // intent bucket is drained (no UndeliveredIntent diagnostics).
        // `update` takes the mutable controlled value and writes it on
        // commit only; the draft lives in the state (see `draft_text`).
        let value_blurred = cx
            .intents(FILTER_VALUE)
            .any(|intent| matches!(intent, Intent::FocusOut { .. }));
        let value_response = TextInput::new(FILTER_VALUE).placeholder("value").update(
            cx,
            &mut editor.value_state,
            &mut editor.value,
        );
        let value_action = value_response.action_ref().copied();
        let value2_blurred = cx
            .intents(FILTER_VALUE2)
            .any(|intent| matches!(intent, Intent::FocusOut { .. }));
        let value2_response = TextInput::new(FILTER_VALUE2).placeholder("value").update(
            cx,
            &mut editor.value2_state,
            &mut editor.value2,
        );
        let value2_action = value2_response.action_ref().copied();
        // Column and Op dropdowns are owned by Select. The column runs
        // first: choosing a column rebuilds the type-appropriate op list,
        // so the op value is seeded fresh after.
        editor
            .col_select
            .set_value(editor.columns.get(editor.column_idx).map(column_key));
        let col_response = Select::new(FILTER_COL)
            .key(column_key)
            .row(column_row)
            .update(cx, &mut editor.col_select, &editor.columns);
        let col_action = col_response.action_ref().copied();
        let col_dirty = col_action.is_some() || col_response.is_changed();
        if let Some(SelectAction::Chose(key)) = col_action {
            editor.choose_column(key);
        }
        editor.op_select.set_value(Some(op_key(&editor.op)));
        let op_response = Select::new(FILTER_OP).key(op_key).row(op_row).update(
            cx,
            &mut editor.op_select,
            &editor.ops,
        );
        let op_action = op_response.action_ref().copied();
        let op_dirty = op_action.is_some() || op_response.is_changed();
        if let Some(SelectAction::Chose(key)) = op_action {
            editor.choose_op(key);
        }
        // Cancel/Apply are owned by Button: Enter, Space and click all
        // produce the same typed `Activated`, which maps to the editor
        // outcome below. The manual intent loop keeps the Cancel/Apply
        // stops only for FocusIn sync and Tab cycling.
        let confirm_label: &str = if editor.index.is_some() {
            "Update filter"
        } else {
            "Add filter"
        };
        let cancel_response = Button::new(FILTER_CANCEL, "Cancel")
            .variant(Variant::SUBTLE)
            .update(cx);
        let apply_response = Button::new(FILTER_APPLY, confirm_label)
            .variant(Variant::PRIMARY)
            .update(cx);
        let cancel_activated = cancel_response.activated();
        let apply_activated = apply_response.activated();
        let buttons_dirty = cancel_response.is_changed() || apply_response.is_changed();
        let mut outcome = None;
        // Esc over an open Select popup must reach the runtime bubble pass,
        // which dismisses the popover layer. Consuming it here (every
        // `changed()` is `Consumed`) would leave the popup open with no
        // dismissal ever delivered.
        let mut esc_on_open_popup = false;
        for (id, focus_variant) in [
            (FILTER_COL, FilterFocus::Column),
            (FILTER_OP, FilterFocus::Op),
            (FILTER_VALUE, FilterFocus::Value),
            (FILTER_VALUE2, FilterFocus::Value2),
            (FILTER_CANCEL, FilterFocus::Cancel),
            (FILTER_APPLY, FilterFocus::Apply),
        ] {
            for intent in cx.intents(id) {
                match intent {
                    Intent::FocusIn { .. } => {
                        editor.focus = focus_variant;
                    }
                    Intent::Key(key) => {
                        editor.focus = focus_variant;
                        if key.code == KeyCode::Esc
                            && (editor.col_select.is_open() || editor.op_select.is_open())
                        {
                            esc_on_open_popup = true;
                        }
                        if outcome.is_none() {
                            outcome = Some(editor.on_key(key));
                        }
                    }
                    _ => {}
                }
            }
        }
        for intent in cx.intents(FILTER_EDITOR) {
            if let Intent::Key(key) = intent
                && outcome.is_none() {
                    outcome = Some(editor.on_key(key));
                }
        }
        // Owned-button activation overrides any idle `Keep` the key routing
        // above produced for the same frame (Enter reaches both paths).
        if cancel_activated {
            outcome = Some(FilterOutcome::Cancel);
        } else if apply_activated {
            outcome = Some(FilterOutcome::Apply(editor.to_filter()));
        }
        // Enter in either value field commits and applies. Blur commits are
        // already written into `editor.value` / `editor.value2` by TextInput;
        // the editor stays open and FocusIn above synced `editor.focus`.
        let value_committed = value_action == Some(TextAction::Committed) && !value_blurred;
        let value2_committed = value2_action == Some(TextAction::Committed) && !value2_blurred;
        if value_committed || value2_committed {
            let filter = editor.to_filter();
            let index = editor.index;
            self.filter_editor = None;
            cx.close_layer(FILTER_EDITOR, None);
            if let Some(Tab::Table(table)) = self.workbench.active_mut() {
                match index {
                    Some(i) if i < table.filters.len() => table.filters[i] = filter,
                    _ => table.filters.push(filter),
                }
                table.reload(&self.catalog);
                let n = table.filters.iter().filter(|f| f.enabled).count();
                self.status = format!("{n} filter{} applied", if n == 1 { "" } else { "s" });
            }
            self.sync_active_tab();
            self.refocus_content(cx);
            return Response::changed();
        }
        // Esc on an idle value field arrives as a Cancel binding (TextInput
        // published it) rather than a Key, so `on_key` never sees it. When
        // TextInput already cancelled a draft this frame, the same binding
        // must not also close the editor.
        if value_action.is_none()
            && !editor.value_state.is_editing()
            && cx.intents(FILTER_VALUE).any(|intent| match intent {
                Intent::Binding(key) => {
                    key == ActionKey::CANCEL || key == ActionKey::custom("Cancel")
                }
                _ => false,
            })
        {
            outcome = Some(FilterOutcome::Cancel);
        }
        if value2_action.is_none()
            && !editor.value2_state.is_editing()
            && cx.intents(FILTER_VALUE2).any(|intent| match intent {
                Intent::Binding(key) => {
                    key == ActionKey::CANCEL || key == ActionKey::custom("Cancel")
                }
                _ => false,
            })
        {
            outcome = Some(FilterOutcome::Cancel);
        }
        match outcome {
            Some(FilterOutcome::Cancel) => {
                self.filter_editor = None;
                cx.close_layer(FILTER_EDITOR, None);
                self.sync_active_tab();
                if let Some(tab_key) = self.workbench.active_key() {
                    let pf = match self.workbench.active() {
                        Some(Tab::Table(t)) if t.is_structure() => {
                            Some(tab_key.control("structure"))
                        }
                        Some(Tab::Table(_)) => Some(tab_key.control("data")),
                        Some(Tab::Query(_)) => Some(tab_key.control("query")),
                        Some(Tab::History(_)) => Some(tab_key.control("history")),
                        None => None,
                    };
                    if let Some(pf) = pf {
                        cx.focus(pf);
                    } else {
                        cx.focus(CONTENT_FRAME);
                    }
                } else {
                    cx.focus(CONTENT_FRAME);
                }
                Response::changed()
            }
            Some(FilterOutcome::Apply(filter)) => {
                let index = editor.index;
                self.filter_editor = None;
                cx.close_layer(FILTER_EDITOR, None);
                if let Some(Tab::Table(table)) = self.workbench.active_mut() {
                    match index {
                        Some(i) if i < table.filters.len() => table.filters[i] = filter,
                        _ => table.filters.push(filter),
                    }
                    table.reload(&self.catalog);
                    let n = table.filters.iter().filter(|f| f.enabled).count();
                    self.status = format!("{n} filter{} applied", if n == 1 { "" } else { "s" });
                }
                self.sync_active_tab();
                if let Some(tab_key) = self.workbench.active_key() {
                    let pf = match self.workbench.active() {
                        Some(Tab::Table(t)) if t.is_structure() => {
                            Some(tab_key.control("structure"))
                        }
                        Some(Tab::Table(_)) => Some(tab_key.control("data")),
                        Some(Tab::Query(_)) => Some(tab_key.control("query")),
                        Some(Tab::History(_)) => Some(tab_key.control("history")),
                        None => None,
                    };
                    if let Some(pf) = pf {
                        cx.focus(pf);
                    } else {
                        cx.focus(CONTENT_FRAME);
                    }
                } else {
                    cx.focus(CONTENT_FRAME);
                }
                Response::changed()
            }
            Some(FilterOutcome::Keep) if esc_on_open_popup => Response::ignored(),
            Some(FilterOutcome::Keep) => Response::changed(),
            None if value_action.is_some()
                || value2_action.is_some()
                || col_dirty
                || op_dirty
                || buttons_dirty =>
            {
                Response::changed()
            }
            None => Response::ignored(),
        }
    }

    fn update_tab_controls(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        for (key, tab) in self.workbench.payloads_mut() {
            let ctrl = match tab {
                Tab::Table(t) if t.is_structure() => key.control("structure"),
                Tab::Table(_) => key.control("data"),
                Tab::Query(_) => key.control("query"),
                Tab::History(_) => key.control("history"),
            };
            if cx.intents(ctrl).any(|it| matches!(it, Intent::FocusIn { .. })) {
                self.workbench_focus = ctrl;
            }
            if let Tab::Query(query) = tab {
                let id = key.control("query");
                if !query.editor_state.is_editing() {
                    let enter_edit = cx.intents(id).any(|it| match it {
                        Intent::Key(termrock::Key {
                            code: KeyCode::Char('i') | KeyCode::Enter,
                            ..
                        }) => true,
                        Intent::Binding(k) => {
                            k == ActionKey::custom("Edit") || k == ActionKey::custom("Edit (i)")
                        }
                        _ => false,
                    });
                    if enter_edit {
                        query.editor_state.begin(&query.query);
                        response |= Response::changed();
                    }
                    continue;
                }
                let has_cancel = cx.intents(id).any(|it| match it {
                    Intent::Cancel => true,
                    Intent::Key(termrock::Key {
                        code: KeyCode::Esc,
                        ..
                    }) => true,
                    Intent::Binding(k) => {
                        k == ActionKey::CANCEL || k == ActionKey::custom("Cancel")
                    }
                    _ => false,
                });
                if has_cancel {
                    let _ = query
                        .editor_state
                        .commit(&mut query.query, &termrock::NoValidate);
                    response |= Response::consumed();
                    continue;
                } else {
                    response |= query_input(id, None)
                        .update(cx, &mut query.editor_state, &mut query.query)
                        .erase();
                }
            }
            match tab {
                Tab::Table(table) => {
                    let grid_response =
                        Self::update_grid_view(cx, key.control("data"), &mut table.result);
                    if let Some(action) = grid_response.action_ref() {
                        match action {
                            GridAction::Sort(col_key, dir) => {
                                let col_idx = usize::from(col_key.raw()).saturating_sub(1);
                                self.status = table.reload_sorted(
                                    &self.catalog,
                                    col_idx,
                                    Some((*col_key, *dir)),
                                );
                            }
                            other => Self::handle_grid(&mut self.status, other),
                        }
                    }
                    response |= grid_response.erase();
                    let grid_response =
                        Self::update_grid_view(cx, key.control("structure"), &mut table.structure);
                    if let Some(action) = grid_response.action_ref() {
                        Self::handle_grid(&mut self.status, action);
                    }
                    response |= grid_response.erase();
                }
                Tab::Query(query) => {
                    if let Some(grid) = &mut query.result {
                        let grid_response =
                            Self::update_grid_view(cx, key.control("results"), grid);
                        if let Some(action) = grid_response.action_ref() {
                            Self::handle_grid(&mut self.status, action);
                        }
                        response |= grid_response.erase();
                    }
                }
                Tab::History(_) => {}
            }
        }
        response
    }

    fn update_grid_view(cx: &mut Cx<'_>, id: Id, view: &mut GridView) -> Response<GridAction> {
        let (columns, count) =
            Self::column_specs(&view.columns, view.model.is_editable(), None, &[]);
        let grid = result_grid(id, columns.get(..count).unwrap_or(&[]));
        let response = if view.model.is_editable() {
            grid.update_editable(cx, &mut view.state, &mut view.model)
        } else {
            grid.update(cx, &mut view.state, &view.model)
        };
        if let Some(GridAction::Sort(key, direction)) = response.action_ref() {
            view.model.sort(*key, *direction);
        }
        response
    }

    fn draw_result_grid(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        if let Some((id, grid)) = self.workbench.active_grid() {
            let (active_table, active_filters) = match self.workbench.active() {
                Some(Tab::Table(t)) => (Some(&t.table), t.filters.as_slice()),
                _ => (None, &[][..]),
            };
            let (columns, count) =
                Self::column_specs(&grid.columns, grid.model.is_editable(), active_table, active_filters);
            result_grid(id, columns.get(..count).unwrap_or(&[]))
                .cell(&|cell, painter| {
                    let avail = painter.available_width();
                    if avail == 0 {
                        return;
                    }
                    let col_name = grid.columns.get(cell.column).map(|(n, _)| n.as_str());
                    let col_type = grid.columns.get(cell.column).map(|(_, ty)| *ty);
                    let is_fk = active_table
                        .and_then(|t| col_name.and_then(|n| t.column(n)))
                        .is_some_and(|c| c.references.is_some());
                    let raw = cell.value.text;
                    let is_cursor = cell.flags.contains(termrock::StateFlags::ACTIVE);
                    let is_null_or_empty = raw.is_empty() || raw == "NULL" || raw == "DEFAULT";
                    if col_type == Some(ColType::Uuid) {
                        let shown = termrock::truncate_middle(raw, avail);
                        if is_fk && avail > 6 && !is_null_or_empty {
                            let char_count = shown.chars().count();
                            let prefix: String =
                                shown.chars().take(char_count.saturating_sub(1)).collect();
                            painter.text(&prefix);
                            if is_cursor {
                                painter.suffix_style(termrock::GlyphRole::FollowRef, cell.style);
                            } else {
                                painter.suffix(termrock::GlyphRole::FollowRef);
                            }
                        } else {
                            painter.text(&shown);
                        }
                    } else if is_fk && avail > 6 && !is_null_or_empty {
                        let shown = termrock::truncate(raw, avail);
                        let char_count = shown.chars().count();
                        let prefix: String =
                            shown.chars().take(char_count.saturating_sub(1)).collect();
                        painter.text(&prefix);
                        if is_cursor {
                            painter.suffix_style(termrock::GlyphRole::FollowRef, cell.style);
                        } else {
                            painter.suffix(termrock::GlyphRole::FollowRef);
                        }
                    } else {
                        let shown = termrock::truncate(raw, avail);
                        painter.text(&shown);
                    }
                })
                .draw(
                    ui,
                    area,
                    &grid.state,
                    &grid.model,
                );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_structure_view(
        &self,
        ui: &mut Ui<'_>,
        inner: termrock::Rect,
        table: &TableTab,
        active_tab_style: PaintStyle,
        inactive_tab_style: PaintStyle,
        border_strong_style: PaintStyle,
        border_subtle_style: PaintStyle,
        status_out: &mut Option<StructureStatusLine>,
    ) {
        let body = termrock::Rect::new(
            inner.x,
            inner.y.saturating_add(3),
            inner.width,
            inner.height.saturating_sub(3),
        );
        if body.height < 4 {
            return;
        }

        if let Some(tab_key) = self.workbench.active_key() {
            ui.register_control(tab_key.control("structure"), body, Focusability::Focusable);
        }

        // 1. Structure tabs: Columns, Indexes, Foreign keys, Constraints, Triggers, DDL
        let labels = [
            "Columns",
            "Indexes",
            "Foreign keys",
            "Constraints",
            "Triggers",
            "DDL",
        ];
        let widths: [u16; 6] = [10, 10, 15, 14, 11, 6];
        let total: u16 = widths.iter().map(|&w| w + 1).sum();
        let overflow = total > body.width;
        let left_w: u16 = if overflow { 4 } else { 0 };
        let right_w: u16 = if overflow { 4 } else { 0 };
        let avail = body.width.saturating_sub(left_w + right_w);

        let mut fit = 0usize;
        let mut used = 0u16;
        for &w in &widths {
            if used + w + 1 > avail {
                break;
            }
            used += w + 1;
            fit += 1;
        }
        let fit = fit.clamp(1, 6);

        if overflow {
            let left_st = ui.surface_style().patch(
                ui.paint_patch(
                    &StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
                ),
            );
            ui.paint_str(termrock::Rect::new(body.x, body.y, left_w, 1), "    ", left_st);

            let hidden_right = 6usize.saturating_sub(fit);
            if hidden_right > 0 {
                let rx = body.right().saturating_sub(right_w);
                let overflow_text = format!("{:>3}›", hidden_right.min(99));
                let st = ui.surface_style().patch(
                    ui.paint_patch(
                        &StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)),
                    ),
                );
                ui.paint_str(termrock::Rect::new(rx, body.y, right_w, 1), &overflow_text, st);
            }
        }

        let start_x = body.x.saturating_add(left_w);
        let mut cur_x = start_x;
        let mut active_rect = termrock::Rect::ZERO;
        for (i, &label) in labels.iter().take(fit).enumerate() {
            let w = widths[i];
            let r = termrock::Rect::new(cur_x, body.y, w, 1);
            if i == 0 {
                active_rect = r;
                ui.fill(r, active_tab_style);
                ui.paint_str(r, &format!(" {label}  "), active_tab_style);
            } else {
                ui.fill(r, inactive_tab_style);
                ui.paint_str(r, &format!(" {label}  "), inactive_tab_style);
            }
            cur_x = cur_x.saturating_add(w).saturating_add(1);
        }

        // Sub-tabs underline rule at body.y + 1
        let rule_y = body.y.saturating_add(1);
        let full_rule_rect = termrock::Rect::new(body.x, rule_y, body.width, 1);
        let subtle_line = "─".repeat(body.width as usize);
        ui.paint_str(full_rule_rect, &subtle_line, border_subtle_style);

        if !active_rect.is_empty() {
            let strong_rect = termrock::Rect::new(active_rect.x, rule_y, active_rect.width, 1);
            let strong_line = "━".repeat(active_rect.width as usize);
            ui.paint_str(strong_rect, &strong_line, border_strong_style);
        }

        // 2. Status line at body.bottom() - 1, deferred past the panel
        // clip (see `StructureStatusLine`): the style resolves here, under
        // the panel surface, while the paint runs after `Panel::draw`.
        let status_y = body.bottom().saturating_sub(1);
        let n = table.table.columns.len();
        let status_str = format!("{n} columns · read from the catalog · changes are queued until Save");
        let status_style = ui.surface_style().patch(
            ui.paint_patch(
                &StylePatch::new().set_fg(Role::Fg(FgStep::Muted)),
            ),
        );
        *status_out = Some(StructureStatusLine {
            x: inner.x.saturating_add(1),
            y: status_y,
            text: status_str,
            style: status_style,
        });

        // 3. DataTable area
        let table_y = body.y.saturating_add(3);
        let table_height = body.height.saturating_sub(4);
        if table_height < 2 {
            return;
        }
        let table_area = termrock::Rect::new(body.x, table_y, body.width, table_height);

        let data_rows_height = table_area.height.saturating_sub(1);
        let num_rows = table.table.columns.len();
        let visible_rows = num_rows.min(data_rows_height as usize);
        let has_sb = num_rows > visible_rows;

        let cols_area_width = table_area
            .width
            .saturating_sub(5 + if has_sb { 1 } else { 0 });
        let cols_area = termrock::Rect::new(
            table_area.x.saturating_add(3),
            table_area.y,
            cols_area_width,
            table_area.height,
        );

        let min_widths = [16u16, 14, 8, 22, 6];
        let mut col_fit = 0usize;
        let mut col_used = 0u16;
        for &w in &min_widths {
            let need = if col_fit == 0 { w } else { w + 2 };
            if col_used + need > cols_area_width {
                break;
            }
            col_used += need;
            col_fit += 1;
        }
        let col_fit = col_fit.clamp(1, 5);
        let more_right = col_fit < 5;

        let constraints: Vec<ratatui::layout::Constraint> = (0..col_fit)
            .map(|i| match i {
                0 => ratatui::layout::Constraint::Min(16),
                1 => ratatui::layout::Constraint::Length(14),
                2 => ratatui::layout::Constraint::Length(8),
                3 => ratatui::layout::Constraint::Length(22),
                4 => ratatui::layout::Constraint::Length(6),
                _ => ratatui::layout::Constraint::Min(6),
            })
            .collect();
        let col_rects = ratatui::layout::Layout::horizontal(constraints)
            .spacing(2)
            .split(ratatui::layout::Rect::new(
                cols_area.x,
                cols_area.y,
                cols_area.width,
                1,
            ));

        // Column headers
        let col_titles = ["Name", "Type", "Nullable", "Default", "Key"];
        let header_style = ui.surface_style().patch(
            ui.paint_patch(
                &StylePatch::new().set_fg(Role::Fg(FgStep::Muted)),
            ),
        );
        for ci in 0..col_fit {
            let r = col_rects[ci];
            let title = Self::fit_text(col_titles[ci], r.width);
            ui.paint_str(
                termrock::Rect::new(r.x, r.y, r.width, 1),
                &title,
                header_style,
            );
        }
        if more_right {
            let faint_style = ui.surface_style().patch(
                ui.paint_patch(
                    &StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
                ),
            );
            ui.paint_str(
                termrock::Rect::new(cols_area.right().saturating_add(1), table_area.y, 1, 1),
                "…",
                faint_style,
            );
        }

        // Column data rows
        for di in 0..visible_rows {
            let y = table_area.y.saturating_add(1).saturating_add(di as u16);
            let is_focused = di == 0;
            let row_rect = termrock::Rect::new(
                table_area.x,
                y,
                table_area.width.saturating_sub(if has_sb { 1 } else { 0 }),
                1,
            );
            let mut row_patch = StylePatch::new().set_fg(Role::Fg(FgStep::Primary));
            if is_focused {
                row_patch = row_patch.add(Modifier::BOLD);
            }
            let row_style = ui.surface_style().patch(ui.paint_patch(&row_patch));
            ui.fill(row_rect, row_style);

            if is_focused {
                let gutter_style = ui.surface_style().patch(
                    ui.paint_patch(
                        &StylePatch::new()
                            .set_fg(Role::Accent)
                            .add(Modifier::BOLD),
                    ),
                );
                ui.paint_str(
                    termrock::Rect::new(table_area.x, y, 1, 1),
                    "▎",
                    gutter_style,
                );
            } else {
                let gutter_style = ui.surface_style().with_fg_from_bg(ui.surface_style());
                ui.paint_str(
                    termrock::Rect::new(table_area.x, y, 1, 1),
                    " ",
                    gutter_style,
                );
            }
            let col_data = &table.table.columns[di];
            let name_str = &col_data.name;
            let type_str = col_data.ty.sql();
            let nullable_str = if col_data.nullable { "yes" } else { "no" };
            let default_str = col_data.default.as_deref().unwrap_or("—");
            let key_str = if col_data.primary {
                "PK"
            } else if col_data.references.is_some() {
                "FK"
            } else {
                ""
            };

            let values = [name_str, type_str, nullable_str, default_str, key_str];
            for ci in 0..col_fit {
                let r = col_rects[ci];
                let fg = match ci {
                    0 => Role::Fg(FgStep::Primary),
                    1 => Role::Fg(FgStep::Secondary),
                    2 => {
                        if col_data.nullable {
                            Role::Fg(FgStep::Muted)
                        } else {
                            Role::Fg(FgStep::Secondary)
                        }
                    }
                    3 => Role::Fg(FgStep::Muted),
                    4 => Role::Fg(FgStep::Secondary),
                    _ => Role::Fg(FgStep::Primary),
                };
                let mut patch = StylePatch::new().set_fg(fg);
                if is_focused {
                    patch = patch.add(Modifier::BOLD);
                }
                let cell_style = ui.surface_style().patch(ui.paint_patch(&patch));
                let text = Self::fit_text(values[ci], r.width);
                ui.paint_str(
                    termrock::Rect::new(r.x, y, r.width, 1),
                    &text,
                    cell_style,
                );
            }
        }

        // Scrollbar if needed
        if has_sb {
            let mut scroll = termrock::ScrollState::new(num_rows);
            scroll.set_viewport(data_rows_height as usize);
            let fade_rect = termrock::Rect::new(
                table_area.x,
                table_area.y.saturating_add(1),
                table_area.width.saturating_sub(1),
                data_rows_height,
            );
            ui.scroll_edges_except(
                fade_rect,
                &scroll,
                &[table_area.y.saturating_add(1)],
            );

            let track_len = visible_rows;
            let len = ((track_len * track_len) / num_rows).max(1);
            let len = len.min(track_len);
            let (start, len) = (0, len);
            let sb_x = table_area.right().saturating_sub(1);
            let thumb_style = ui.surface_style().patch(
                ui.paint_patch(
                    &StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
                ),
            );
            let track_style = ui.surface_style().patch(
                ui.paint_patch(
                    &StylePatch::new().set_fg(Role::BorderSubtle),
                ),
            );
            for i in 0..track_len {
                let sy = table_area.y.saturating_add(1).saturating_add(i as u16);
                let cell = termrock::Rect::new(sb_x, sy, 1, 1);
                if i >= start && i < start + len {
                    ui.paint_str(cell, "┃", thumb_style);
                } else {
                    ui.paint_str(cell, "│", track_style);
                }
            }
        }
    }

    fn draw_connection_details(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
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
            connection.ssh.as_deref().unwrap_or("off")
        );
        let env_role = match connection.environment {
            Environment::Production => Role::Fg(FgStep::Primary),
            Environment::Staging => Role::Fg(FgStep::Secondary),
            Environment::Development => Role::Fg(FgStep::Muted),
            Environment::Local => Role::Fg(FgStep::Faint),
        };
        let safe_mode_role = if connection.safe_mode >= SafeMode::Safe {
            Role::Fg(FgStep::Primary)
        } else {
            Role::Fg(FgStep::Secondary)
        };
        let mut properties = vec![
            (
                "Engine",
                connection.engine.label().to_owned(),
                Role::Fg(FgStep::Primary),
            ),
            ("Host", host, Role::Fg(FgStep::Primary)),
            (
                "Database",
                connection.database.clone(),
                Role::Fg(FgStep::Primary),
            ),
            (
                "User",
                if connection.user.is_empty() {
                    "—".to_owned()
                } else {
                    connection.user.clone()
                },
                Role::Fg(FgStep::Primary),
            ),
            (
                "Environment",
                connection.environment.label().to_owned(),
                env_role,
            ),
            (
                "Safe Mode",
                format!(
                    "{} · {}",
                    connection.safe_mode.label(),
                    connection.safe_mode.description()
                ),
                safe_mode_role,
            ),
        ];
        if connection.environment == Environment::Production
            && connection.safe_mode == SafeMode::Silent
        {
            properties.push((
                "",
                "Production with Silent safe mode: writes run without asking".to_owned(),
                Role::Warning,
            ));
        }
        properties.push(("SSL / SSH", ssl_ssh, Role::Fg(FgStep::Secondary)));
        properties.push((
            "Last used",
            connection.last_used.clone(),
            Role::Fg(FgStep::Muted),
        ));
        let card_area = termrock::Rect {
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

    fn draw_connections_list(
        &self,
        ui: &mut Ui<'_>,
        list_area: termrock::Rect,
        focused: bool,
        is_compact: bool,
    ) {
        let count = self.connections_screen.connections.len().to_string();
        let panel = Self::connections_panel("Connections", Some(&count), focused);
        let inner = panel.inner(ui, list_area);
        let body = legacy_tree_body(inner);
        panel.draw(ui, list_area, |_, _| {});
        ui.with_area(body, |ui| {
            let filter = termrock::Rect {
                x: body.x,
                y: inner.y.saturating_add(1),
                width: body.width,
                height: 1.min(inner.height),
            };
            if !ui.is_inert() {
                ui.register_editor(
                    CONNECTION_FILTER,
                    filter,
                    Focusability::Focusable,
                    if self.connections_screen.filter_active {
                        termrock::StateFlags::EDITING
                    } else {
                        termrock::StateFlags::empty()
                    },
                );
            }
            let filter_text = if self.connections_screen.filter_active {
                &self.connections_screen.filter
            } else if self.connections_screen.filter.is_empty() {
                "Filter connections"
            } else {
                &self.connections_screen.filter
            };
            paint_legacy_filter(
                ui,
                filter,
                filter_text,
                self.connections_screen.filter_active,
            );
            let tree_area = termrock::Rect {
                y: body.y.saturating_add(2),
                height: inner.height.saturating_sub(2),
                ..body
            };
            let show_meta = if is_compact {
                true
            } else {
                let row_w = list_area.width.saturating_sub(4);
                self.connections.iter().all(|c| {
                    let meta_w = termrock::width(c.engine.short()) as u16;
                    let need = 10 + termrock::width(&c.name) as u16 + meta_w;
                    need <= row_w
                })
            };
            connection_tree_with_meta(show_meta).draw(
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
                focused && !self.connections_screen.filter_active,
            );
        });
        let blank_fg = if self.connections_screen.filter_active {
            Role::Fg(FgStep::Primary)
        } else {
            Role::Fg(FgStep::Secondary)
        };
        let mut patch = StylePatch::new().set_fg(blank_fg);
        if self.connections_screen.filter_active {
            patch = patch.add(Modifier::BOLD);
        }
        let blank = ui.surface_style().patch(ui.paint_patch(&patch));
        ui.fill(
            termrock::Rect {
                x: body.x.saturating_add(2),
                y: body.y,
                width: body.width.saturating_sub(2),
                height: 1,
            },
            blank,
        );
    }

    fn draw_connections(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        let list_width = (area.width / 3).clamp(26, 40).min(area.width);
        let list_area = termrock::Rect {
            x: area.x,
            y: area.y,
            width: if area.width < 80 {
                area.width
            } else {
                list_width
            },
            height: area.height,
        };
        let focused = self.destructive_intent.is_none() && !self.form_open;
        self.draw_connections_list(ui, list_area, focused, area.width < 80);
        if area.width >= 80 {
            let details = termrock::Rect {
                x: area.x.saturating_add(list_width).saturating_add(2),
                y: area.y,
                width: area.width.saturating_sub(list_width).saturating_sub(2),
                height: area.height,
            };
            self.draw_connection_details(ui, details);
        }
    }

    fn draw_connection_form(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        let (card_area, list_area) = if area.width < 80 {
            if self.form_tab == 0 {
                self.draw_connections_list(ui, area, false, true);
            }
            (
                termrock::Rect {
                    x: area.x,
                    y: area.y,
                    width: area.width.min(84),
                    height: area.height,
                },
                None,
            )
        } else {
            let list_width = (area.width / 3).clamp(26, 40).min(area.width);
            let right_x = area.x.saturating_add(list_width).saturating_add(2);
            let card_width = area
                .width
                .saturating_sub(list_width)
                .saturating_sub(2)
                .min(84);
            (
                termrock::Rect {
                    x: right_x,
                    y: area.y,
                    width: card_width,
                    height: area.height,
                },
                Some(termrock::Rect {
                    x: area.x,
                    y: area.y,
                    width: list_width,
                    height: area.height,
                }),
            )
        };

        if self.form_tab == 0
            && let Some(list_area) = list_area
        {
            self.draw_connections_list(ui, list_area, false, false);
        }

        self.draw_connection_form_card(ui, card_area);
    }

    fn draw_connection_form_card(&self, ui: &mut Ui<'_>, card_area: termrock::Rect) {
        let Some(draft) = self.draft.as_ref() else {
            return;
        };

        let card_bg =
            ui.paint_patch(&StylePatch::new().set_bg(Role::Surface(termrock::Surface::Surface)));
        ui.fill(card_area, card_bg);

        let title = if self.form_is_edit {
            "Edit connection"
        } else {
            "New connection"
        };
        let title_style =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
        ui.paint_str(
            termrock::Rect {
                x: card_area.x.saturating_add(2),
                y: card_area.y,
                width: card_area.width.saturating_sub(2),
                height: 1,
            },
            title,
            title_style,
        );

        let meta = "Ctrl+S Save";
        let meta_style =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));
        ui.paint_str(
            termrock::Rect {
                x: card_area.right().saturating_sub(2 + 11),
                y: card_area.y,
                width: 11.min(card_area.width),
                height: 1,
            },
            meta,
            meta_style,
        );

        let inner = termrock::Rect {
            x: card_area.x.saturating_add(2),
            y: card_area.y.saturating_add(2),
            width: card_area.width.saturating_sub(4),
            height: card_area.height.saturating_sub(3),
        };

        let tab0_active = self.form_tab == 0;
        let is_16_color = ui.theme_ref().capability.color == termrock::ColorLevel::Ansi16;
        let raised_bg =
            if is_16_color {
                card_bg
            } else {
                ui.surface_style().patch(ui.paint_patch(
                    &StylePatch::new().set_bg(Role::Surface(termrock::Surface::Overlay)),
                ))
            };
        let button_bg = ui.surface_style().patch(
            ui.paint_patch(&StylePatch::new().set_bg(Role::Surface(termrock::Surface::Overlay))),
        );
        ui.register_focus_only(connections::field::TABS, Focusability::Focusable);
        let tab0_style = if tab0_active {
            raised_bg.patch(
                ui.paint_patch(
                    &StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Primary))
                        .add(Modifier::BOLD),
                ),
            )
        } else {
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))))
        };
        let tab1_style = if !tab0_active {
            raised_bg.patch(
                ui.paint_patch(
                    &StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Primary))
                        .add(Modifier::BOLD),
                ),
            )
        } else {
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))))
        };

        let tab0_rect = termrock::Rect {
            x: inner.x,
            y: inner.y,
            width: 8.min(inner.width),
            height: 1,
        };
        ui.fill(tab0_rect, if tab0_active { raised_bg } else { card_bg });
        ui.paint_str(tab0_rect, " Basic  ", tab0_style);

        let tab1_x = inner.x.saturating_add(9);
        let tab1_rect = termrock::Rect {
            x: tab1_x,
            y: inner.y,
            width: 11.min(inner.right().saturating_sub(tab1_x)),
            height: 1,
        };
        ui.fill(tab1_rect, if !tab0_active { raised_bg } else { card_bg });
        ui.paint_str(tab1_rect, " Advanced  ", tab1_style);

        let underline_y = inner.y.saturating_add(1);
        let rule_subtle =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::BorderSubtle)));
        let rule_accent = card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));

        for xx in inner.left()..inner.right() {
            ui.paint_str(
                termrock::Rect {
                    x: xx,
                    y: underline_y,
                    width: 1,
                    height: 1,
                },
                "─",
                rule_subtle,
            );
        }
        let (active_x, active_w) = if tab0_active {
            (inner.x, 8.min(inner.width))
        } else {
            (tab1_x, 11.min(inner.right().saturating_sub(tab1_x)))
        };
        for xx in active_x..active_x.saturating_add(active_w) {
            ui.paint_str(
                termrock::Rect {
                    x: xx,
                    y: underline_y,
                    width: 1,
                    height: 1,
                },
                "━",
                rule_accent,
            );
        }

        let body = termrock::Rect {
            x: inner.x,
            y: inner.y.saturating_add(3),
            width: inner.width,
            height: inner.height.saturating_sub(5),
        };

        let usable = body.width.saturating_sub(4);
        let (lc, rc) = if usable < 54 {
            (
                termrock::Rect {
                    x: body.x,
                    y: body.y,
                    width: 0,
                    height: body.height,
                },
                body,
            )
        } else {
            let mut first = ((usable as u32 * 58) / 100) as u16;
            first = first.clamp(30, usable.saturating_sub(24));
            (
                termrock::Rect {
                    x: body.x,
                    y: body.y,
                    width: first,
                    height: body.height,
                },
                termrock::Rect {
                    x: body.x.saturating_add(first).saturating_add(4),
                    y: body.y,
                    width: usable.saturating_sub(first),
                    height: body.height,
                },
            )
        };

        if tab0_active {
            self.draw_form_basic_tab(ui, lc, rc, draft, card_bg);
        } else {
            self.draw_form_advanced_tab(ui, lc, rc, draft, card_bg);
        }

        let ay = inner.bottom().saturating_sub(1);
        let widths = [17, 8, 6, 16];
        let rects = Self::row_layout(
            termrock::Rect {
                x: inner.x,
                y: ay,
                width: inner.width,
                height: 1,
            },
            &widths,
            2,
        );

        if let Some(&r) = rects.first() {
            let btn_sec_style = button_bg
                .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
            let btn_gutter =
                button_bg.patch(ui.paint_patch(
                    &StylePatch::new().set_fg(Role::Surface(termrock::Surface::Overlay)),
                ));
            ui.paint_str(
                termrock::Rect {
                    x: r.x,
                    y: r.y,
                    width: 1,
                    height: 1,
                },
                " ",
                btn_gutter,
            );
            ui.paint_str(
                termrock::Rect {
                    x: r.x.saturating_add(1),
                    y: r.y,
                    width: r.width.saturating_sub(2),
                    height: 1,
                },
                "Test connection",
                btn_sec_style,
            );
            ui.paint_str(
                termrock::Rect {
                    x: r.right().saturating_sub(1),
                    y: r.y,
                    width: 1,
                    height: 1,
                },
                " ",
                btn_sec_style,
            );
        }

        if let Some(&r) = rects.get(1) {
            let btn_subtle_style = card_bg
                .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
            let btn_gutter =
                card_bg.patch(ui.paint_patch(
                    &StylePatch::new().set_fg(Role::Surface(termrock::Surface::Surface)),
                ));
            ui.paint_str(
                termrock::Rect {
                    x: r.x,
                    y: r.y,
                    width: 1,
                    height: 1,
                },
                " ",
                btn_gutter,
            );
            ui.paint_str(
                termrock::Rect {
                    x: r.x.saturating_add(1),
                    y: r.y,
                    width: r.width.saturating_sub(2),
                    height: 1,
                },
                "Cancel",
                btn_subtle_style,
            );
            ui.paint_str(
                termrock::Rect {
                    x: r.right().saturating_sub(1),
                    y: r.y,
                    width: 1,
                    height: 1,
                },
                " ",
                btn_subtle_style,
            );
        }

        if let Some(&r) = rects.get(2) {
            let btn_sec_style = button_bg
                .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
            let btn_gutter =
                button_bg.patch(ui.paint_patch(
                    &StylePatch::new().set_fg(Role::Surface(termrock::Surface::Overlay)),
                ));
            ui.paint_str(
                termrock::Rect {
                    x: r.x,
                    y: r.y,
                    width: 1,
                    height: 1,
                },
                " ",
                btn_gutter,
            );
            ui.paint_str(
                termrock::Rect {
                    x: r.x.saturating_add(1),
                    y: r.y,
                    width: r.width.saturating_sub(2),
                    height: 1,
                },
                "Save",
                btn_sec_style,
            );
            ui.paint_str(
                termrock::Rect {
                    x: r.right().saturating_sub(1),
                    y: r.y,
                    width: 1,
                    height: 1,
                },
                " ",
                btn_sec_style,
            );
        }

        if let Some(&r) = rects.get(3) {
            let accent_bg = ui
                .surface_style()
                .patch(ui.paint_patch(&StylePatch::new().set_bg(Role::Accent)));
            let btn_pri_style = accent_bg.patch(
                ui.paint_patch(&StylePatch::new().set_fg(Role::OnAccent).add(Modifier::BOLD)),
            );
            let btn_gutter =
                accent_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
            ui.paint_str(
                termrock::Rect {
                    x: r.x,
                    y: r.y,
                    width: 1,
                    height: 1,
                },
                " ",
                btn_gutter,
            );
            ui.paint_str(
                termrock::Rect {
                    x: r.x.saturating_add(1),
                    y: r.y,
                    width: r.width.saturating_sub(2),
                    height: 1,
                },
                "Save & connect",
                btn_pri_style,
            );
            ui.paint_str(
                termrock::Rect {
                    x: r.right().saturating_sub(1),
                    y: r.y,
                    width: 1,
                    height: 1,
                },
                " ",
                btn_pri_style,
            );
        }
    }

    fn row_layout(area: termrock::Rect, widths: &[u16], gap: u16) -> Vec<termrock::Rect> {
        let mut x = area.x;
        let mut out = Vec::new();
        for &w in widths {
            let w = w.min(area.right().saturating_sub(x));
            out.push(termrock::Rect {
                x,
                y: area.y,
                width: w,
                height: area.height.min(1),
            });
            x = x.saturating_add(w).saturating_add(gap);
        }
        out
    }

    fn fit_text(s: &str, w: u16) -> String {
        let t = truncate(s, w);
        let pad = w.saturating_sub(termrock::width(&t)) as usize;
        format!("{t}{}", " ".repeat(pad))
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_form_input(
        ui: &mut Ui<'_>,
        area: termrock::Rect,
        label: &str,
        value: &str,
        placeholder: &str,
        help: &str,
        required: bool,
        focused: bool,
        disabled: bool,
        card_bg: PaintStyle,
        field_bg: PaintStyle,
    ) {
        if area.is_empty() {
            return;
        }
        let name_w = termrock::width(label) as u16;
        let show_optional =
            !required && !label.is_empty() && name_w.saturating_add(12) <= area.width;
        let mut full_label = label.to_owned();
        if required {
            full_label.push_str(" *");
        } else if show_optional {
            full_label.push_str("  optional");
        }
        let label_style = if disabled {
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))))
        } else if focused {
            card_bg.patch(
                ui.paint_patch(
                    &StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Primary))
                        .add(Modifier::BOLD),
                ),
            )
        } else {
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))))
        };
        let label_x = area.x.saturating_add(2.min(area.width));
        let avail_w = area.width.saturating_sub(2);
        let fit_label = Self::fit_text(&full_label, avail_w);
        ui.paint_str(
            termrock::Rect {
                x: label_x,
                y: area.y,
                width: area.width.saturating_sub(2),
                height: 1,
            },
            &fit_label,
            label_style,
        );
        if required && !disabled && name_w.saturating_add(4) <= area.width {
            let req_style = card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
            ui.paint_str(
                termrock::Rect {
                    x: label_x.saturating_add(name_w).saturating_add(1),
                    y: area.y,
                    width: 1,
                    height: 1,
                },
                "*",
                req_style,
            );
        } else if show_optional {
            let opt_style =
                card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));
            ui.paint_str(
                termrock::Rect {
                    x: label_x.saturating_add(name_w).saturating_add(2),
                    y: area.y,
                    width: 8,
                    height: 1,
                },
                "optional",
                opt_style,
            );
        }

        if area.height >= 2 {
            let field_rect = termrock::Rect {
                x: area.x,
                y: area.y.saturating_add(1),
                width: area.width,
                height: 1,
            };
            let mut field_flags = termrock::StateFlags::empty();
            if disabled {
                field_flags |= termrock::StateFlags::DISABLED;
            }
            let resolved = ui.style(
                termrock::Family::INPUT,
                termrock::Variant::DEFAULT,
                termrock::Part::FIELD,
                field_flags,
            );
            let field_style = if disabled {
                field_bg
                    .patch(resolved.style)
                    .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))))
            } else {
                field_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))))
            };
            ui.fill(field_rect, field_style);

            let gutter_style = if focused {
                field_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)))
            } else {
                field_style.patch(ui.paint_patch(
                    &StylePatch::new().set_fg(Role::Surface(termrock::Surface::Field)),
                ))
            };
            ui.paint_str(
                termrock::Rect {
                    x: area.x,
                    y: field_rect.y,
                    width: 1,
                    height: 1,
                },
                if focused { "▎" } else { " " },
                gutter_style,
            );

            let inner_x = area.x.saturating_add(2.min(area.width));
            let inner_w = area.width.saturating_sub(3);
            if value.is_empty() {
                if !placeholder.is_empty() && inner_w > 0 {
                    let ph_style = if disabled {
                        field_style
                    } else {
                        field_bg.patch(
                            ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))),
                        )
                    };
                    let ph = truncate(placeholder, inner_w);
                    ui.paint_str(
                        termrock::Rect {
                            x: inner_x,
                            y: field_rect.y,
                            width: termrock::width(&ph) as u16,
                            height: 1,
                        },
                        &ph,
                        ph_style,
                    );
                }
            } else if inner_w > 0 {
                let val_style = if disabled {
                    field_style
                } else {
                    field_bg
                        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))))
                };
                let val = truncate(value, inner_w);
                ui.paint_str(
                    termrock::Rect {
                        x: inner_x,
                        y: field_rect.y,
                        width: termrock::width(&val) as u16,
                        height: 1,
                    },
                    &val,
                    val_style,
                );
            }
        }

        if area.height >= 3 && !help.is_empty() {
            let help_style =
                card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
            let h = truncate(help, area.width.saturating_sub(2));
            ui.paint_str(
                termrock::Rect {
                    x: label_x,
                    y: area.y.saturating_add(2),
                    width: area.width.saturating_sub(2),
                    height: 1,
                },
                &h,
                help_style,
            );
        }
    }

    fn draw_form_select(
        ui: &mut Ui<'_>,
        area: termrock::Rect,
        label: &str,
        value: &str,
        card_bg: PaintStyle,
        field_bg: PaintStyle,
    ) {
        if area.is_empty() {
            return;
        }
        let label_style =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
        let label_x = area.x.saturating_add(2.min(area.width));
        ui.paint_str(
            termrock::Rect {
                x: label_x,
                y: area.y,
                width: area.width.saturating_sub(2),
                height: 1,
            },
            label,
            label_style,
        );

        if area.height >= 2 {
            let field_rect = termrock::Rect {
                x: area.x,
                y: area.y.saturating_add(1),
                width: area.width,
                height: 1,
            };
            let fs = field_bg
                .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
            ui.fill(field_rect, fs);

            let gutter_style = field_bg.patch(
                ui.paint_patch(&StylePatch::new().set_fg(Role::Surface(termrock::Surface::Field))),
            );
            ui.paint_str(
                termrock::Rect {
                    x: area.x,
                    y: field_rect.y,
                    width: 1,
                    height: 1,
                },
                " ",
                gutter_style,
            );

            let inner_x = area.x.saturating_add(2.min(area.width));
            let inner_w = area.width.saturating_sub(5);
            if inner_w > 0 {
                let val_style = field_bg
                    .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
                let val = truncate(value, inner_w);
                ui.paint_str(
                    termrock::Rect {
                        x: inner_x,
                        y: field_rect.y,
                        width: termrock::width(&val) as u16,
                        height: 1,
                    },
                    &val,
                    val_style,
                );
            }
            if area.width >= 2 {
                let arrow_style = field_bg
                    .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
                ui.paint_str(
                    termrock::Rect {
                        x: area.right().saturating_sub(2),
                        y: field_rect.y,
                        width: 1,
                        height: 1,
                    },
                    "▾",
                    arrow_style,
                );
            }
        }
    }

    fn draw_form_radio(
        ui: &mut Ui<'_>,
        area: termrock::Rect,
        label: &str,
        options: &[&str],
        selected: usize,
        card_bg: PaintStyle,
    ) {
        if area.is_empty() {
            return;
        }
        let label_style =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
        let label_x = area.x.saturating_add(2.min(area.width));
        ui.paint_str(
            termrock::Rect {
                x: label_x,
                y: area.y,
                width: area.width.saturating_sub(2),
                height: 1,
            },
            label,
            label_style,
        );

        let gutter_style = card_bg.patch(
            ui.paint_patch(&StylePatch::new().set_fg(Role::Surface(termrock::Surface::Surface))),
        );
        let accent_style = card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
        let muted_style =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
        let text_style =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));

        for (i, opt) in options
            .iter()
            .take(area.height.saturating_sub(1) as usize)
            .enumerate()
        {
            let y = area.y.saturating_add(1).saturating_add(i as u16);
            let row_rect = termrock::Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            };
            let row_style =
                card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
            ui.fill(row_rect, row_style);
            ui.paint_str(
                termrock::Rect {
                    x: area.x,
                    y,
                    width: 1,
                    height: 1,
                },
                " ",
                gutter_style,
            );
            let on = i == selected;
            let mark = if area.width < 4 {
                if on { "●" } else { "○" }
            } else if on {
                "(●)"
            } else {
                "( )"
            };
            let ms = if on { accent_style } else { muted_style };
            ui.paint_str(
                termrock::Rect {
                    x: area.x.saturating_add(1),
                    y,
                    width: if area.width < 4 { 1 } else { 3 },
                    height: 1,
                },
                mark,
                ms,
            );
            if area.width > 5 {
                let opt_text = truncate(opt, area.width.saturating_sub(5));
                ui.paint_str(
                    termrock::Rect {
                        x: area.x.saturating_add(5),
                        y,
                        width: area.width.saturating_sub(5),
                        height: 1,
                    },
                    &opt_text,
                    text_style,
                );
            }
        }
    }

    fn draw_form_toggle(
        ui: &mut Ui<'_>,
        area: termrock::Rect,
        label: &str,
        on: bool,
        disabled: bool,
        card_bg: PaintStyle,
    ) {
        if area.is_empty() {
            return;
        }
        let row_rect = termrock::Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        let row_style =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
        ui.fill(row_rect, row_style);
        let gutter_style = card_bg.patch(
            ui.paint_patch(&StylePatch::new().set_fg(Role::Surface(termrock::Surface::Surface))),
        );
        ui.paint_str(
            termrock::Rect {
                x: area.x,
                y: area.y,
                width: 1,
                height: 1,
            },
            " ",
            gutter_style,
        );
        let accent_style = card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
        let muted_style =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
        let faint_style =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));
        let text_style = if disabled {
            faint_style
        } else {
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))))
        };

        let (sw, ss) = if disabled {
            (if on { "──●" } else { "○──" }, muted_style)
        } else if on {
            ("──●", accent_style)
        } else {
            ("○──", muted_style)
        };
        let (sw, sw_len) = if area.width < 4 {
            (if on { "●" } else { "○" }, 1u16)
        } else {
            (sw, 3u16)
        };
        ui.paint_str(
            termrock::Rect {
                x: area.x.saturating_add(1),
                y: area.y,
                width: sw_len,
                height: 1,
            },
            sw,
            ss,
        );
        if area.width > 5 {
            let label_text = truncate(label, area.width.saturating_sub(5));
            ui.paint_str(
                termrock::Rect {
                    x: area.x.saturating_add(5),
                    y: area.y,
                    width: area.width.saturating_sub(5),
                    height: 1,
                },
                &label_text,
                text_style,
            );
        }
        let state = if on { "on" } else { "off" };
        let state_offset = 6u16.saturating_add(termrock::width(label) as u16);
        if state_offset.saturating_add(3) < area.width {
            ui.paint_str(
                termrock::Rect {
                    x: area.x.saturating_add(state_offset),
                    y: area.y,
                    width: 3,
                    height: 1,
                },
                state,
                if disabled { faint_style } else { muted_style },
            );
        }
    }

    fn draw_form_basic_tab(
        &self,
        ui: &mut Ui<'_>,
        lc: termrock::Rect,
        rc: termrock::Rect,
        draft: &ConnectionDraft,
        card_bg: PaintStyle,
    ) {
        ui.register_focus_only(connections::field::NAME, Focusability::Focusable);
        let field_bg = ui.surface_style().patch(
            ui.paint_patch(&StylePatch::new().set_bg(Role::Surface(termrock::Surface::Field))),
        );

        let fh = 3u16;

        let mut y = lc.y;

        // 1. Name
        Self::draw_form_input(
            ui,
            termrock::Rect {
                x: lc.x,
                y,
                width: lc.width,
                height: fh,
            },
            "Name",
            &draft.name,
            "",
            "",
            true,
            true,
            false,
            card_bg,
            field_bg,
        );
        y = y.saturating_add(fh);

        // 2. Engine
        let engines = ["PostgreSQL", "MySQL", "SQLite"];
        let engine_str = engines.get(draft.engine).copied().unwrap_or("PostgreSQL");
        Self::draw_form_select(
            ui,
            termrock::Rect {
                x: lc.x,
                y,
                width: lc.width,
                height: 3,
            },
            "Engine",
            engine_str,
            card_bg,
            field_bg,
        );
        y = y.saturating_add(3);

        // 3. Host and Port
        let usable = lc.width.saturating_sub(2);
        let first = if usable < 20 {
            usable.min(12)
        } else {
            let f = ((usable as u32 * 70) / 100) as u16;
            f.clamp(12, usable.saturating_sub(8))
        };
        let hl = termrock::Rect {
            x: lc.x,
            y,
            width: first,
            height: fh,
        };
        let hr = termrock::Rect {
            x: lc.x.saturating_add(first).saturating_add(2),
            y,
            width: usable.saturating_sub(first),
            height: fh,
        };
        let host_val = if draft.host.is_empty() {
            "localhost"
        } else {
            &draft.host
        };
        Self::draw_form_input(
            ui,
            hl,
            "Host",
            host_val,
            "",
            "Blank: driver default",
            false,
            false,
            false,
            card_bg,
            field_bg,
        );
        let port_val = if draft.port.is_empty() {
            "5432"
        } else {
            &draft.port
        };
        Self::draw_form_input(
            ui, hr, "Port", port_val, "", "", false, false, false, card_bg, field_bg,
        );
        y = y.saturating_add(fh);

        // 4. Database
        Self::draw_form_input(
            ui,
            termrock::Rect {
                x: lc.x,
                y,
                width: lc.width,
                height: fh,
            },
            "Database",
            &draft.database,
            "",
            "Required for PostgreSQL",
            false,
            false,
            false,
            card_bg,
            field_bg,
        );
        y = y.saturating_add(fh);

        // 5. Username
        Self::draw_form_input(
            ui,
            termrock::Rect {
                x: lc.x,
                y,
                width: lc.width,
                height: fh,
            },
            "Username",
            &draft.user,
            "",
            "",
            false,
            false,
            false,
            card_bg,
            field_bg,
        );
        y = y.saturating_add(fh);

        // 6. Password
        Self::draw_form_input(
            ui,
            termrock::Rect {
                x: lc.x,
                y,
                width: lc.width,
                height: fh,
            },
            "Password",
            "",
            "stored in the keychain",
            "Never written to connections.json",
            false,
            false,
            false,
            card_bg,
            field_bg,
        );
        y = y.saturating_add(fh);

        // 7. Prompt for password on connect
        let area = termrock::Rect {
            x: lc.x,
            y,
            width: lc.width,
            height: 1,
        };
        if !area.is_empty() && area.width > 5 {
            let label = truncate(
                "Prompt for password on connect",
                area.width.saturating_sub(6),
            );
            Checkbox::new(connections::field::ASK_PASSWORD, &label)
                .checked(draft.ask_password)
                .patch_part(&[
                    (
                        Part::CONTAINER,
                        StylePatch::new()
                            .set_bg(Role::Surface(termrock::Surface::Surface))
                            .set_fg(Role::Fg(FgStep::Primary)),
                    ),
                    (
                        Part::GUTTER,
                        StylePatch::new()
                            .set_bg(Role::Surface(termrock::Surface::Surface))
                            .set_fg(Role::Surface(termrock::Surface::Surface)),
                    ),
                    (
                        Part::MARKER,
                        StylePatch::new()
                            .set_bg(Role::Surface(termrock::Surface::Surface))
                            .set_fg(Role::Fg(FgStep::Muted)),
                    ),
                    (
                        Part::LABEL,
                        StylePatch::new()
                            .set_bg(Role::Surface(termrock::Surface::Surface))
                            .set_fg(Role::Fg(FgStep::Primary)),
                    ),
                ])
                .draw(ui, area);
        }

        // Right column
        let mut ry = rc.y;

        // 1. Environment
        let env_opts = ["local", "development", "staging", "production"];
        Self::draw_form_radio(
            ui,
            termrock::Rect {
                x: rc.x,
                y: ry,
                width: rc.width,
                height: 5,
            },
            "Environment",
            &env_opts,
            draft.environment.min(3),
            card_bg,
        );
        ry = ry.saturating_add(5 + 1);

        // 2. Group
        let groups = ["Personal", "Acme", "Clients"];
        let group_str = groups.get(draft.group).copied().unwrap_or("Personal");
        Self::draw_form_select(
            ui,
            termrock::Rect {
                x: rc.x,
                y: ry,
                width: rc.width,
                height: 3,
            },
            "Group",
            group_str,
            card_bg,
            field_bg,
        );
        ry = ry.saturating_add(3);

        // 3. Safe Mode
        let safe_modes = [
            "Silent",
            "Alert",
            "Alert (Full)",
            "Safe Mode",
            "Safe Mode (Full)",
            "Read-Only",
        ];
        Self::draw_form_radio(
            ui,
            termrock::Rect {
                x: rc.x,
                y: ry,
                width: rc.width,
                height: 7,
            },
            "Safe Mode",
            &safe_modes,
            draft.safe_mode.min(5),
            card_bg,
        );
        ry = ry.saturating_add(7);

        // Safe mode description (up to 2 wrapped lines)
        let desc = SafeMode::ALL[draft.safe_mode.min(5)].description();
        let wrap_w = rc.width.saturating_sub(2);
        let muted_style =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
        for (i, line) in wrap(desc, wrap_w).iter().take(2).enumerate() {
            let row_y = ry.saturating_add(i as u16);
            if row_y < rc.bottom() {
                ui.paint_str(
                    termrock::Rect {
                        x: rc.x.saturating_add(2),
                        y: row_y,
                        width: rc.width.saturating_sub(2),
                        height: 1,
                    },
                    line,
                    muted_style,
                );
            }
        }
    }

    fn draw_form_advanced_tab(
        &self,
        ui: &mut Ui<'_>,
        lc: termrock::Rect,
        rc: termrock::Rect,
        draft: &ConnectionDraft,
        card_bg: PaintStyle,
    ) {
        let field_bg = ui.surface_style().patch(
            ui.paint_patch(&StylePatch::new().set_bg(Role::Surface(termrock::Surface::Field))),
        );

        let fh = 3u16;

        let mut y = lc.y;

        // 1. SSL / TLS
        Self::draw_form_toggle(
            ui,
            termrock::Rect {
                x: lc.x,
                y,
                width: lc.width,
                height: 1,
            },
            "Use SSL / TLS",
            draft.ssl,
            false,
            card_bg,
        );
        y = y.saturating_add(2);

        // 2. SSH tunnel
        let ssh_on = draft.ssh;
        Self::draw_form_toggle(
            ui,
            termrock::Rect {
                x: lc.x,
                y,
                width: lc.width,
                height: 1,
            },
            "SSH tunnel",
            ssh_on,
            false,
            card_bg,
        );
        y = y.saturating_add(1);

        // 3. SSH host
        let disabled = !ssh_on;
        let ssh_host_val = if draft.ssh_host.is_empty() {
            ""
        } else {
            &draft.ssh_host
        };
        Self::draw_form_input(
            ui,
            termrock::Rect {
                x: lc.x,
                y,
                width: lc.width,
                height: fh,
            },
            "SSH host",
            ssh_host_val,
            "bastion.example.com",
            "",
            false,
            false,
            disabled,
            card_bg,
            field_bg,
        );
        y = y.saturating_add(fh);

        // 4. SSH user
        Self::draw_form_input(
            ui,
            termrock::Rect {
                x: lc.x,
                y,
                width: lc.width,
                height: fh,
            },
            "SSH user",
            "deploy",
            "",
            "",
            false,
            false,
            disabled,
            card_bg,
            field_bg,
        );
        y = y.saturating_add(fh);

        // 5. Local only (no iCloud sync)
        Self::draw_form_toggle(
            ui,
            termrock::Rect {
                x: lc.x,
                y,
                width: lc.width,
                height: 1,
            },
            "Local only (no iCloud sync)",
            false,
            false,
            card_bg,
        );

        // Right column
        let label_style =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
        let label_x = rc.x.saturating_add(2.min(rc.width));
        ui.paint_str(
            termrock::Rect {
                x: label_x,
                y: rc.y,
                width: rc.width.saturating_sub(2),
                height: 1,
            },
            "Startup commands",
            label_style,
        );

        let body_rect = termrock::Rect {
            x: rc.x,
            y: rc.y.saturating_add(1),
            width: rc.width,
            height: 3,
        };
        let fs =
            field_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
        ui.fill(body_rect, fs);

        let gutter_style = field_bg.patch(
            ui.paint_patch(&StylePatch::new().set_fg(Role::Surface(termrock::Surface::Field))),
        );
        for y in body_rect.top()..body_rect.bottom() {
            ui.paint_str(
                termrock::Rect {
                    x: body_rect.x,
                    y,
                    width: 1,
                    height: 1,
                },
                " ",
                gutter_style,
            );
        }

        let ph_style =
            field_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
        let ph_text = truncate("SET statement_timeout = '60s';", rc.width.saturating_sub(4));
        ui.paint_str(
            termrock::Rect {
                x: rc.x.saturating_add(2.min(rc.width)),
                y: rc.y.saturating_add(1),
                width: rc.width.saturating_sub(4),
                height: 1,
            },
            &ph_text,
            ph_style,
        );

        let help_style =
            card_bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
        let help_text = truncate(
            "Run after every connect, one per line",
            rc.width.saturating_sub(2),
        );
        ui.paint_str(
            termrock::Rect {
                x: label_x,
                y: rc.y.saturating_add(4),
                width: rc.width.saturating_sub(2),
                height: 1,
            },
            &help_text,
            help_style,
        );

        ui.paint_str(
            termrock::Rect {
                x: label_x,
                y: rc.y.saturating_add(6),
                width: 27,
                height: 1,
            },
            "External clients: read only",
            help_style,
        );
    }

    fn draw_explorer(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        let in_dialog = self.safety_dialog.is_some()
            || self.destructive_intent.is_some()
            || self.help_open;
        let focused = !in_dialog && self.workbench_focus == EXPLORER;
        let panel = Self::explorer_panel(self.workbench.schema_caption(), focused);
        let inner = panel.inner(ui, area);
        let body = legacy_tree_body(inner);
        panel.draw(ui, area, |_, _| {});
        ui.with_area(body, |ui| {
            let label_style = ui
                .surface_style()
                .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
            let top_label_rect = termrock::Rect {
                x: body.x.saturating_add(2),
                y: inner.y,
                width: body.width.saturating_sub(2),
                height: 1.min(inner.height),
            };
            ui.fill(top_label_rect, label_style);
            let filter = termrock::Rect {
                x: body.x,
                y: inner.y.saturating_add(1),
                width: body.width,
                height: 1.min(inner.height),
            };
            paint_legacy_filter(ui, filter, "Filter objects", false);
            let tree_area = termrock::Rect {
                y: body.y.saturating_add(2),
                height: inner.height.saturating_sub(2),
                ..body
            };
            let show_meta = {
                let mut visible_nodes = Vec::new();
                let mut ancestors_expanded = Vec::new();
                for item in &self.explorer_nodes {
                    let descriptor = explorer_node(item);
                    let depth = usize::from(descriptor.depth());
                    ancestors_expanded.truncate(depth);
                    let visible = ancestors_expanded.iter().all(|expanded| *expanded);
                    if visible {
                        visible_nodes.push(item);
                    }
                    if descriptor.has_children() {
                        ancestors_expanded.push(self.explorer_tree_state.is_expanded(explorer_node_key(item)));
                    }
                }
                let offset = self.explorer_tree_state.scroll().offset();
                let limit = (offset + usize::from(tree_area.height)).min(visible_nodes.len());
                let has_sb = visible_nodes.len() > usize::from(tree_area.height);
                let row_w = tree_area.width.saturating_sub(if has_sb { 1 } else { 0 });
                (offset..limit).all(|ri| {
                    let Some(node) = visible_nodes.get(ri) else {
                        return true;
                    };
                    let (depth, glyph_w, label_w, meta_w) = match node {
                        ExplorerNode::Database { name } => (0, 2, termrock::width(name), 0),
                        ExplorerNode::Schema { name } => (1, 2, termrock::width(name), 0),
                        ExplorerNode::Group { name, count, .. } => (2, 0, termrock::width(name), termrock::width(count)),
                        ExplorerNode::Object { item, count, .. } => {
                            (3, 2, termrock::width(&item.name), termrock::width(count))
                        }
                    };
                    if meta_w == 0 {
                        return true;
                    }
                    let need = 1 + depth * 2 + 2 + glyph_w + label_w + 2 + meta_w + 1;
                    need <= row_w
                })
            };
            let active_table = match self.workbench.active() {
                Some(Tab::Table(t)) => Some((t.table.schema.clone(), t.table.name.clone())),
                _ => None,
            };
            explorer_tree_with_meta(show_meta, active_table)
                .focused(focused)
                .draw(
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
                focused,
            );
        });
    }

    fn draw_sql_highlighted(
        ui: &mut Ui<'_>,
        mut x: u16,
        y: u16,
        sql: &str,
        base_style: PaintStyle,
    ) {
        let kw_style = base_style.patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .add(Modifier::BOLD),
            ),
        );
        let str_style = base_style
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
        let muted_style =
            base_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
        let primary_style =
            base_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));

        let is_keyword = |w: &str| -> bool {
            matches!(
                w.to_ascii_uppercase().as_str(),
                "SELECT"
                    | "FROM"
                    | "WHERE"
                    | "UPDATE"
                    | "SET"
                    | "DELETE"
                    | "INSERT"
                    | "INTO"
                    | "VALUES"
                    | "AND"
                    | "OR"
                    | "NOT"
                    | "ORDER"
                    | "BY"
                    | "ASC"
                    | "DESC"
                    | "LIMIT"
                    | "OFFSET"
                    | "JOIN"
                    | "LEFT"
                    | "RIGHT"
                    | "INNER"
                    | "OUTER"
                    | "ON"
                    | "GROUP"
                    | "HAVING"
                    | "AS"
                    | "IN"
                    | "IS"
                    | "NULL"
                    | "LIKE"
                    | "EXPLAIN"
                    | "ANALYZE"
            )
        };

        let mut chars = sql.char_indices().peekable();
        while let Some((i, ch)) = chars.next() {
            if ch == '\'' {
                let start = i;
                let mut end = sql.len();
                while let Some((j, c)) = chars.peek().copied() {
                    if c == '\'' {
                        chars.next();
                        end = j + 1;
                        break;
                    }
                    chars.next();
                }
                let s = &sql[start..end];
                let w = s.chars().count() as u16;
                ui.paint_str(termrock::Rect::new(x, y, w, 1), s, str_style);
                x = x.saturating_add(w);
            } else if ch == '…'
                || ch == '='
                || ch == '<'
                || ch == '>'
                || ch == ','
                || ch == ';'
                || ch == '*'
            {
                let mut buf = [0u8; 4];
                let s = ch.encode_utf8(&mut buf);
                let w = 1u16;
                ui.paint_str(termrock::Rect::new(x, y, w, 1), s, muted_style);
                x = x.saturating_add(w);
            } else if ch.is_whitespace() {
                let mut buf = [0u8; 4];
                let s = ch.encode_utf8(&mut buf);
                let w = 1u16;
                ui.paint_str(termrock::Rect::new(x, y, w, 1), s, base_style);
                x = x.saturating_add(w);
            } else {
                let start = i;
                let mut end = i + ch.len_utf8();
                while let Some((j, c)) = chars.peek().copied() {
                    if c == '\''
                        || c == '…'
                        || c == '='
                        || c == '<'
                        || c == '>'
                        || c == ','
                        || c == ';'
                        || c == '*'
                        || c.is_whitespace()
                    {
                        break;
                    }
                    chars.next();
                    end = j + c.len_utf8();
                }
                let word = &sql[start..end];
                let style = if is_keyword(word) {
                    kw_style
                } else {
                    primary_style
                };
                let w = word.chars().count() as u16;
                ui.paint_str(termrock::Rect::new(x, y, w, 1), word, style);
                x = x.saturating_add(w);
            }
        }
    }

    fn draw_content(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        let (title, meta) = match self.workbench.active() {
            Some(Tab::Table(table)) => (
                qualified_label("", &table.table.schema, &table.table.name),
                Some(format!("{} cols", table.table.columns.len())),
            ),
            Some(Tab::Query(query)) => (
                query.name.clone(),
                Some(
                    query
                        .last_duration
                        .map_or("".to_string(), |ms| format!("{ms} ms")),
                ),
            ),
            Some(Tab::History(_)) => (
                "Query history".to_owned(),
                Some(format!("{} entries", self.workbench.history.entries.len())),
            ),
            None => ("Workbench".to_owned(), None),
        };
        let in_picker = self.switcher_open || self.tab_list_open || self.safe_mode_open;
        let in_dialog = self.safety_dialog.is_some()
            || self.destructive_intent.is_some()
            || self.help_open;
        let focused = !in_picker && (in_dialog || self.workbench_focus != EXPLORER);
        let panel = Self::content_panel(&title, meta.as_deref(), focused);
        let mut status_line: Option<StructureStatusLine> = None;
        panel.draw(ui, area, |ui, inner| match self.workbench.active() {
            Some(Tab::Query(query)) => {
                let (editor_h, bottom_rect) = match query.maximized {
                    QueryPaneMaximized::Editor => (inner.height, termrock::Rect::ZERO),
                    QueryPaneMaximized::Results => (0, inner),
                    QueryPaneMaximized::None => {
                        let usable = inner.height.saturating_sub(1);
                        if usable < 4 + 6 {
                            (inner.height, termrock::Rect::ZERO)
                        } else {
                            let mut first = (usable as u32 * 38 / 100) as u16;
                            first = first.clamp(4, usable.saturating_sub(6));
                            (
                                first,
                                termrock::Rect::new(
                                    inner.x,
                                    inner.y.saturating_add(first).saturating_add(1),
                                    inner.width,
                                    inner.height.saturating_sub(first.saturating_add(1)),
                                ),
                            )
                        }
                    }
                };
                let active_key = self.workbench.active_key();
                if editor_h > 0 {
                    let editor_rect = termrock::Rect::new(inner.x, inner.y, inner.width, editor_h);
                if let Some(key) = active_key
                    && !ui.is_inert()
                {
                    let query_id = key.control("query");
                    let flags = if query.editor_state.is_editing() {
                        termrock::StateFlags::EDITING
                    } else {
                        termrock::StateFlags::empty()
                    };
                    ui.register_editor(query_id, editor_rect, Focusability::Focusable, flags);
                }
                let field_style = ui.surface_style().patch(ui.paint_patch(
                    &StylePatch::new().set_bg(Role::Surface(termrock::Surface::Field)),
                ));
                ui.fill(editor_rect, field_style);

                let is_focused = !ui.is_inert()
                    && self.safety_dialog.is_none()
                    && self.destructive_intent.is_none()
                    && active_key.is_some_and(|k| {
                        ui.state(k.control("query"))
                            .contains(termrock::StateFlags::FOCUSED)
                    });

                let gutter_style = if is_focused {
                    field_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)))
                } else {
                    field_style.patch(
                        ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))),
                    )
                };
                let query_text = query.editor_state.draft_text().unwrap_or(&query.query);
                let num_style = if is_focused {
                    field_style.patch(
                        ui.paint_patch(
                            &StylePatch::new()
                                .set_fg(Role::Fg(FgStep::Primary))
                                .add(Modifier::BOLD),
                        ),
                    )
                } else if !query_text.is_empty() {
                    field_style.patch(
                        ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))),
                    )
                } else {
                    field_style.patch(
                        ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))),
                    )
                };

                let bar_style = if is_focused {
                    field_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)))
                } else {
                    field_style.patch(ui.paint_patch(
                        &StylePatch::new().set_fg(Role::Surface(termrock::Surface::Field)),
                    ))
                };
                ui.paint_str(
                    termrock::Rect::new(inner.x, inner.y, 1, 1),
                    if is_focused { "▎" } else { " " },
                    bar_style,
                );
                if !query_text.is_empty() {
                    ui.paint_str(
                        termrock::Rect::new(inner.x + 1, inner.y, 1, 1),
                        "›",
                        gutter_style,
                    );
                }
                ui.paint_str(
                    termrock::Rect::new(inner.x + 3, inner.y, 2, 1),
                    " 1",
                    num_style,
                );

                let text_x = inner.x + 6;
                let text_avail = (inner.width as usize).saturating_sub(6);
                if query_text.is_empty() && !query.editor_state.is_editing() {
                    let placeholder = "Type SQL. Ctrl+R runs the statement under the cursor.";
                    let display_ph = if placeholder.len() > text_avail {
                        &placeholder[..text_avail]
                    } else {
                        placeholder
                    };
                    let ph_style = field_style.patch(
                        ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))),
                    );
                    ui.paint_str(
                        termrock::Rect::new(
                            text_x,
                            inner.y,
                            display_ph.chars().count() as u16,
                            1,
                        ),
                        display_ph,
                        ph_style,
                    );
                } else {
                    let display_query = if query_text.len() > text_avail {
                        if let Some(pos) = query_text.find("FROM ") {
                            format!("… {}", &query_text[pos..])
                        } else {
                            query_text.to_owned()
                        }
                    } else {
                        query_text.to_owned()
                    };
                    Self::draw_sql_highlighted(ui, text_x, inner.y, &display_query, field_style);
                }

                if self.safety_dialog.is_none() && (is_focused || query.editor_state.is_editing()) {
                    let cursor_col = if query_text.is_empty() {
                        1
                    } else {
                        query_text.len() + 1
                    };
                    let readout = format!("ln 1/1 · col {cursor_col}");
                    let readout_w = termrock::width(&readout) as u16;
                    let readout_x = inner.right().saturating_sub(1).saturating_sub(readout_w);
                    let muted_field = field_style
                        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));
                    ui.paint_str(
                        termrock::Rect::new(
                            readout_x,
                            inner.y.saturating_add(editor_h).saturating_sub(1),
                            readout_w,
                            1,
                        ),
                        &readout,
                        muted_field,
                    );
                }
                }

                if !bottom_rect.is_empty() {
                if query.affected.is_some() || query.result.is_some() {
                    let (tab_text, status_text) =
                        if let Some((affected_count, affected_action)) = &query.affected {
                            let ms = query.last_duration.unwrap_or(42);
                            (
                                format!("{affected_action} ({affected_count})"),
                                format!(
                                    "{affected_action} · {affected_count} rows affected · {ms} ms"
                                ),
                            )
                        } else if let Some(res) = &query.result {
                            let ms = query.last_duration.unwrap_or(15);
                            (
                                format!("SELECT orders ({})", res.model.row_count()),
                                format!("{} rows · {ms} ms", res.model.row_count()),
                            )
                        } else {
                            (String::new(), String::new())
                        };

                    let elevated_style = ui.surface_style().patch(*ui.paint_patch(
                        &StylePatch::new().set_bg(Role::Surface(termrock::Surface::Elevated)),
                    ));
                    let tab_title_style = elevated_style.patch(
                        *ui.paint_patch(
                            &StylePatch::new()
                                .set_fg(Role::Fg(FgStep::Primary))
                                .add(Modifier::BOLD),
                        ),
                    );
                    let tab_close_style = elevated_style.patch(
                        *ui.paint_patch(
                            &StylePatch::new()
                                .set_fg(Role::BorderStrong)
                                .add(Modifier::BOLD),
                        ),
                    );

                    let label_padded = format!(" {tab_text} ");
                    let label_w = label_padded.len() as u16;
                    ui.paint_str(
                        termrock::Rect::new(inner.x, bottom_rect.y, label_w, 1),
                        &label_padded,
                        tab_title_style,
                    );
                    ui.paint_str(
                        termrock::Rect::new(inner.x + label_w, bottom_rect.y, 1, 1),
                        "×",
                        tab_close_style,
                    );
                    ui.paint_str(
                        termrock::Rect::new(inner.x + label_w + 1, bottom_rect.y, 2, 1),
                        "  ",
                        tab_title_style,
                    );

                    let accent_line_w = label_w + 3;
                    let accent_style = ui
                        .surface_style()
                        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
                    let subtle_style = ui
                        .surface_style()
                        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::BorderSubtle)));
                    let line_chars = "━".repeat(accent_line_w as usize);
                    ui.paint_str(
                        termrock::Rect::new(inner.x, bottom_rect.y.saturating_add(1), accent_line_w, 1),
                        &line_chars,
                        accent_style,
                    );
                    let rest_w = inner.width.saturating_sub(accent_line_w);
                    if rest_w > 0 {
                        let rest_chars = "─".repeat(rest_w as usize);
                        ui.paint_str(
                            termrock::Rect::new(inner.x + accent_line_w, bottom_rect.y.saturating_add(1), rest_w, 1),
                            &rest_chars,
                            subtle_style,
                        );
                    }

                    let muted_style = ui
                        .surface_style()
                        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
                    ui.paint_str(
                        termrock::Rect::new(inner.x + 1, bottom_rect.y.saturating_add(2), inner.width.saturating_sub(1), 1),
                        &status_text,
                        muted_style,
                    );

                    if let Some((affected_count, affected_action)) = &query.affected {
                        let body_y = bottom_rect.y.saturating_add(3);
                        let body_h = bottom_rect.bottom().saturating_sub(body_y);
                        if body_h > 0 {
                            let card_h = body_h.min(6);
                            let card_w = bottom_rect.width.min(60);
                            let card_rect = termrock::Rect::new(bottom_rect.x, body_y, card_w, card_h);
                            let card_style = ui.surface_style().patch(ui.paint_patch(
                                &StylePatch::new().set_bg(Role::Surface(termrock::Surface::Surface)),
                            ));
                            ui.fill(card_rect, card_style);
                            let sec_card = card_style.patch(
                                ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))),
                            );
                            let muted_card = card_style.patch(
                                ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))),
                            );
                            let primary_card = card_style.patch(
                                ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))),
                            );

                            ui.paint_str(
                                termrock::Rect::new(card_rect.x + 2, card_rect.y, 18, 1),
                                "Statement executed",
                                sec_card,
                            );

                            let inner_y = card_rect.y + 2;
                            let inner_h = card_rect.height.saturating_sub(3);
                            if inner_h > 0 {
                                ui.paint_str(
                                    termrock::Rect::new(card_rect.x + 2, inner_y, 9, 1),
                                    "Statement",
                                    muted_card,
                                );
                                ui.paint_str(
                                    termrock::Rect::new(
                                        card_rect.x + 17,
                                        inner_y,
                                        affected_action.len() as u16,
                                        1,
                                    ),
                                    affected_action,
                                    primary_card,
                                );
                            }
                            if inner_h > 1 {
                                let rows_str = format_thousands(*affected_count);
                                ui.paint_str(
                                    termrock::Rect::new(card_rect.x + 2, inner_y + 1, 13, 1),
                                    "Rows affected",
                                    muted_card,
                                );
                                ui.paint_str(
                                    termrock::Rect::new(
                                        card_rect.x + 17,
                                        inner_y + 1,
                                        rows_str.len() as u16,
                                        1,
                                    ),
                                    &rows_str,
                                    primary_card,
                                );
                            }
                            if inner_h > 2 {
                                let ms = query.last_duration.unwrap_or(42);
                                let dur_str = format!("{ms} ms");
                                ui.paint_str(
                                    termrock::Rect::new(card_rect.x + 2, inner_y + 2, 8, 1),
                                    "Duration",
                                    muted_card,
                                );
                                ui.paint_str(
                                    termrock::Rect::new(
                                        card_rect.x + 17,
                                        inner_y + 2,
                                        dur_str.len() as u16,
                                        1,
                                    ),
                                    &dur_str,
                                    muted_card,
                                );
                            }
                        }
                    } else if query.result.is_some() {
                        let grid_area = termrock::Rect::new(
                            inner.x,
                            bottom_rect.y.saturating_add(3),
                            inner.width,
                            bottom_rect.height.saturating_sub(3),
                        );
                        self.draw_result_grid(ui, grid_area);
                    }
                } else {
                    Empty::new(
                        QUERY_EMPTY,
                        EmptyState::Empty {
                            title: "No results yet",
                            hint: Some(
                                "Ctrl+R runs the statement under the cursor · Alt+R runs all",
                            ),
                        },
                    )
                    .draw(ui, bottom_rect);
                }
                }
            }
            Some(Tab::Table(table)) => {
                let tab_row_rect = termrock::Rect {
                    x: inner.x,
                    y: inner.y,
                    width: inner.width,
                    height: 1,
                };
                ui.fill(tab_row_rect, ui.surface_style());

                let active_tab_style = ui
                    .style(
                        Family::TABS,
                        Variant::DEFAULT,
                        Part::TAB,
                        termrock::StateFlags::ACTIVE,
                    )
                    .style;
                let inactive_tab_style = ui
                    .style(
                        Family::TABS,
                        Variant::DEFAULT,
                        Part::TAB,
                        termrock::StateFlags::empty(),
                    )
                    .style;
                let border_strong_style = ui
                    .style(
                        Family::TABS,
                        Variant::DEFAULT,
                        Part::RULE,
                        termrock::StateFlags::empty(),
                    )
                    .style
                    .patch(ui.paint_patch(&termrock::StylePatch::new().set_fg(termrock::Role::BorderStrong)));
                let border_subtle_style = ui
                    .style(
                        Family::TABS,
                        Variant::DEFAULT,
                        Part::RULE,
                        termrock::StateFlags::empty(),
                    )
                    .style;

                if table.is_structure() {
                    let data_rect = termrock::Rect::new(inner.x, inner.y, 7, 1);
                    ui.fill(data_rect, inactive_tab_style);
                    ui.paint_str(data_rect, " Data  ", inactive_tab_style);

                    let structure_rect =
                        termrock::Rect::new(inner.x.saturating_add(8), inner.y, 12, 1);
                    ui.fill(structure_rect, active_tab_style);
                    ui.paint_str(structure_rect, " Structure  ", active_tab_style);

                    let r1 = termrock::Rect::new(inner.x, inner.y.saturating_add(1), 8, 1);
                    ui.paint_str(r1, "────────", border_subtle_style);
                    let r2 = termrock::Rect::new(
                        inner.x.saturating_add(8),
                        inner.y.saturating_add(1),
                        12,
                        1,
                    );
                    ui.paint_str(r2, "━━━━━━━━━━━━", border_strong_style);
                    let rem_width = inner.width.saturating_sub(20);
                    if rem_width > 0 {
                        let r3 = termrock::Rect::new(
                            inner.x.saturating_add(20),
                            inner.y.saturating_add(1),
                            rem_width,
                            1,
                        );
                        let mut quiet = String::new();
                        for _ in 0..rem_width {
                            quiet.push('─');
                        }
                        ui.paint_str(r3, &quiet, border_subtle_style);
                    }

                    self.draw_structure_view(
                        ui,
                        inner,
                        table,
                        active_tab_style,
                        inactive_tab_style,
                        border_strong_style,
                        border_subtle_style,
                        &mut status_line,
                    );
                } else {
                    let data_rect = termrock::Rect::new(inner.x, inner.y, 7, 1);
                    ui.fill(data_rect, active_tab_style);
                    ui.paint_str(data_rect, " Data  ", active_tab_style);

                    let structure_rect =
                        termrock::Rect::new(inner.x.saturating_add(8), inner.y, 12, 1);
                    ui.fill(structure_rect, inactive_tab_style);
                    ui.paint_str(structure_rect, " Structure  ", inactive_tab_style);

                    let r1 = termrock::Rect::new(inner.x, inner.y.saturating_add(1), 7, 1);
                    ui.paint_str(r1, "━━━━━━━", border_strong_style);
                    let rem_width = inner.width.saturating_sub(7);
                    if rem_width > 0 {
                        let r2 = termrock::Rect::new(
                            inner.x.saturating_add(7),
                            inner.y.saturating_add(1),
                            rem_width,
                            1,
                        );
                        let mut quiet = String::new();
                        for _ in 0..rem_width {
                            quiet.push('─');
                        }
                        ui.paint_str(r2, &quiet, border_subtle_style);
                    }

                    let has_filters = !table.filters.is_empty();
                    let pending_total = table.result.pending_total();
                    let pending_bar_h = if pending_total > 0 { 2 } else { 0 };
                    let grid_y_offset = if has_filters { 5 } else { 3 };
                    let grid_height_sub = (if has_filters { 6 } else { 4 }) + pending_bar_h;

                    if has_filters {
                        let chips_rect = termrock::Rect {
                            x: inner.x,
                            y: inner.y.saturating_add(3),
                            width: inner.width,
                            height: 1,
                        };
                        draw_filter_chips(ui, chips_rect, table);
                    }

                    let grid_height = inner.height.saturating_sub(grid_height_sub);
                    let grid_rect = termrock::Rect {
                        x: inner.x,
                        y: inner.y.saturating_add(grid_y_offset),
                        width: inner.width,
                        height: grid_height,
                    };
                    self.draw_result_grid(ui, grid_rect);

                    if pending_total > 0 {
                        let by = inner.bottom().saturating_sub(2);
                        let bar_area = termrock::Rect::new(inner.x, by, inner.width, 1);
                        ui.fill(bar_area, ui.surface_style());

                        let (u, i, d) = table.result.pending_counts();
                        let text = format!("• {pending_total} pending");
                        let warning_style = ui.surface_style().patch(
                            ui.paint_patch(&termrock::StylePatch::new().set_fg(termrock::Role::Warning)),
                        );
                        let _ = ui.paint_str(
                            termrock::Rect::new(inner.x.saturating_add(1), by, termrock::width(&text), 1),
                            &text,
                            warning_style,
                        );

                        let mut parts = vec![];
                        if u > 0 {
                            parts.push(format!("{u} update{}", if u == 1 { "" } else { "s" }));
                        }
                        if i > 0 {
                            parts.push(format!("{i} insert{}", if i == 1 { "" } else { "s" }));
                        }
                        if d > 0 {
                            parts.push(format!("{d} delete{}", if d == 1 { "" } else { "s" }));
                        }
                        let detail = if parts.is_empty() {
                            String::new()
                        } else {
                            format!("· {}", parts.join(" · "))
                        };
                        let muted_style = ui.surface_style().patch(
                            ui.paint_patch(&termrock::StylePatch::new().set_fg(termrock::Role::Fg(termrock::FgStep::Muted))),
                        );
                        let _ = ui.paint_str(
                            termrock::Rect::new(
                                inner.x.saturating_add(2).saturating_add(termrock::width(&text)),
                                by,
                                termrock::width(&detail),
                                1,
                            ),
                            &detail,
                            muted_style,
                        );

                        let active_key = self.workbench.active_key();
                        let id = active_key.map_or_else(|| Id::root("tablepro.pending"), |k| k.control("pending"));
                        let btn_save = Button::new(id.sub("save"), "Save").variant(Variant::PRIMARY);
                        let btn_discard = Button::new(id.sub("discard"), "Discard").variant(Variant::SUBTLE);
                        let btn_preview = Button::new(id.sub("preview"), "Preview SQL").variant(Variant::SUBTLE);

                        let save_w = 6;
                        let discard_w = 9;
                        let preview_w = 13;

                        let mut rx = inner.right().saturating_sub(1);
                        rx = rx.saturating_sub(save_w);
                        let save_rect = termrock::Rect::new(rx, by, save_w, 1);
                        btn_save.draw(ui, save_rect);

                        rx = rx.saturating_sub(1 + discard_w);
                        let discard_rect = termrock::Rect::new(rx, by, discard_w, 1);
                        btn_discard.draw(ui, discard_rect);

                        rx = rx.saturating_sub(1 + preview_w);
                        let preview_rect = termrock::Rect::new(rx, by, preview_w, 1);
                        btn_preview.draw(ui, preview_rect);
                    }

                    if let Some((id, grid)) = self.workbench.active_grid() {
                        let active_table = Some(&table.table);
                        let (columns, count) =
                            Self::column_specs(&grid.columns, grid.model.is_editable(), active_table, &table.filters);
                        let grid_widget = result_grid(id, columns.get(..count).unwrap_or(&[]));
                        let mut parts: Vec<(String, u8)> = vec![];
                        if let Some((col_key, dir)) = grid.state.sort() {
                            let col_idx = usize::from(col_key.raw()).saturating_sub(1);
                            if let Some((col_name, _)) = grid.columns.get(col_idx) {
                                parts.push((
                                    format!(
                                        "sort {} {}",
                                        col_name,
                                        if dir == SortDir::Asc { "▴" } else { "▾" }
                                    ),
                                    4,
                                ));
                            }
                        }
                        let active_filters = table.filters.iter().filter(|f| f.enabled).count();
                        if active_filters > 0 {
                            parts.push((format!("filtered ({active_filters})"), 4));
                        }
                        let rows_label = grid_widget.rows_label(ui, &grid.state, &grid.model);
                        parts.push((rows_label, 5));
                        if let Some(c) = grid_widget.cols_label(ui, &grid.state, &grid.model) {
                            parts.push((c, 2));
                        }
                        if let Some(r) = grid.model.read_only_reason() {
                            parts.push((format!("read-only: {r}"), 3));
                        }
                        let avail = inner.width.saturating_sub(2);
                        let joined = |parts: &[(String, u8)]| {
                            parts
                                .iter()
                                .map(|p| p.0.as_str())
                                .collect::<Vec<_>>()
                                .join(" · ")
                        };
                        while parts.len() > 1 && termrock::width(&joined(&parts)) > avail {
                            let (i, _) = parts
                                .iter()
                                .enumerate()
                                .min_by_key(|(_, (_, prio))| *prio)
                                .unwrap();
                            parts.remove(i);
                        }
                        let status = joined(&parts);
                        let status_y = inner.bottom().saturating_sub(1);
                        let status_rect = termrock::Rect {
                            x: inner.x.saturating_add(1),
                            y: status_y,
                            width: inner.width.saturating_sub(2),
                            height: 1,
                        };
                        let status_style = ui.surface_style().patch(ui.paint_patch(
                            &termrock::StylePatch::new()
                                .set_fg(termrock::Role::Fg(termrock::FgStep::Muted)),
                        ));
                        let _ = ui.paint_str(status_rect, &status, status_style);
                    }
                }
            }
            Some(Tab::History(history)) => {
                ui.paint_str(inner, "Query history", ui.surface_style());
                for (offset, entry) in history.entries.iter().take(6).enumerate() {
                    let row = termrock::Rect {
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
        if let Some(status) = status_line {
            ui.paint_str(
                termrock::Rect::new(status.x, status.y, termrock::width(&status.text), 1),
                &status.text,
                status.style,
            );
        }
    }
}

/// Structure-view status line, painted after [`Panel::draw`] restores the
/// full-screen clip.
///
/// `Panel::draw` narrows the clip to its inner rect and every owned paint API
/// honors the clip, while the frozen baseline pins the raw-buffer overflow
/// over the panel border at narrow widths. The line is therefore specified
/// inside the panel closure (same geometry, same panel-surface style) and
/// painted once the panel closes. Do not move the paint back inside the
/// closure: the 72x20 captures would lose the overflow cells.
struct StructureStatusLine {
    x: u16,
    y: u16,
    text: String,
    style: PaintStyle,
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

fn query_input(id: Id, value: Option<&str>) -> TextInput<'_> {
    let input = TextInput::new(id)
        .blur(termrock::BlurPolicy::Keep)
        .placeholder("Type SQL. Ctrl+R runs the statement under the cursor.");
    match value {
        Some(text) => input.value(text),
        None => input,
    }
}

fn fixed_flex_pair(area: termrock::Rect, first_height: u16) -> [termrock::Rect; 2] {
    let first = first_height.min(area.height);
    [
        termrock::Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: first,
        },
        termrock::Rect {
            x: area.x,
            y: area.y.saturating_add(first),
            width: area.width,
            height: area.height.saturating_sub(first),
        },
    ]
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

fn build_connection_nodes(connections: &[Connection], filter: &str) -> Vec<ConnectionNode> {
    let q = filter.trim().to_lowercase();
    let mut nodes = Vec::with_capacity(connections.len().saturating_mul(2));
    let mut groups: Vec<&str> = Vec::new();
    for connection in connections {
        if !groups.contains(&connection.group.as_str()) {
            groups.push(connection.group.as_str());
        }
    }
    for group in groups {
        let group_matches = !q.is_empty() && group.to_lowercase().contains(&q);
        let matching_conns: Vec<(usize, &Connection)> = connections
            .iter()
            .enumerate()
            .filter(|(_, connection)| {
                connection.group == group
                    && (q.is_empty()
                        || group_matches
                        || connection.name.to_lowercase().contains(&q))
            })
            .collect();
        if q.is_empty() || group_matches || !matching_conns.is_empty() {
            nodes.push(ConnectionNode::Group {
                name: group.to_string(),
            });
            for (index, connection) in matching_conns {
                nodes.push(ConnectionNode::Connection {
                    index,
                    connection: connection.clone(),
                });
            }
        }
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

fn connection_row_with_meta(node: &ConnectionNode, row: &mut RowUi<'_>, show_meta: bool) {
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
            .role(Role::Fg(FgStep::Muted))
            .remove_modifier(Modifier::BOLD);
            row.label_spans(&[glyph, Span::new(" "), Span::new(&connection.name)]);
            if show_meta {
                row.meta(connection.engine.short());
            }
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
                count: objects.len().to_string(),
            });
            nodes.extend(objects.into_iter().map(|item| ExplorerNode::Object {
                count: compact_count(item.rows),
                item,
                prefix,
            }));
        }
    }
    nodes
}

fn initial_explorer_tree_state(nodes: &[ExplorerNode], schema: &str) -> TreeState {
    let mut state = TreeState::default();
    reset_explorer_tree_state(&mut state, nodes, schema);
    state
}

fn reset_explorer_tree_state(state: &mut TreeState, nodes: &[ExplorerNode], schema: &str) {
    state.collapse_all();
    for node in nodes.iter().filter(|node| {
        matches!(node, ExplorerNode::Database { .. })
            || matches!(node, ExplorerNode::Schema { name } if name == schema)
            || matches!(node, ExplorerNode::Group { schema: group_schema, name, .. } if group_schema == schema && name == "Tables")
    }) {
        state.expand(explorer_node_key(node));
    }
    if let Some((index, node)) = nodes
        .iter()
        .enumerate()
        .find(|(_, node)| matches!(node, ExplorerNode::Schema { .. }))
    {
        state.set_cursor(index, explorer_node_key(node));
    }
}

fn explorer_node_key(node: &ExplorerNode) -> ItemKey {
    match node {
        ExplorerNode::Database { name } => stable_key(&["database", name]),
        ExplorerNode::Schema { name } => stable_key(&["schema", name]),
        ExplorerNode::Group { schema, name, .. } => stable_key(&["object-group", schema, name]),
        ExplorerNode::Object { item, prefix, .. } => {
            stable_key(&["object", &item.schema, prefix, &item.name])
        }
    }
}

fn explorer_node(node: &ExplorerNode) -> TreeNode {
    match node {
        ExplorerNode::Database { .. } => TreeNode::parent(0).keyed(explorer_node_key(node)),
        ExplorerNode::Schema { .. } => TreeNode::parent(1).keyed(explorer_node_key(node)),
        ExplorerNode::Group { .. } => TreeNode::parent(2).keyed(explorer_node_key(node)),
        ExplorerNode::Object { .. } => TreeNode::lazy(3).keyed(explorer_node_key(node)),
    }
}

fn qualified_label(prefix: &str, left: &str, right: &str) -> String {
    let mut text = String::with_capacity(
        prefix
            .len()
            .saturating_add(left.len())
            .saturating_add(right.len())
            .saturating_add(" › ".len()),
    );
    text.push_str(prefix);
    text.push_str(left);
    text.push_str(" › ");
    text.push_str(right);
    text
}

fn compact_count(rows: usize) -> String {
    if rows >= 1_000_000 {
        format!("{:.1} M", rows as f64 / 1e6)
    } else if rows >= 1_000 {
        format!("{:.1} k", rows as f64 / 1e3)
    } else {
        rows.to_string()
    }
}

fn explorer_row(
    node: &ExplorerNode,
    row: &mut RowUi<'_>,
    show_meta: bool,
    active_table: Option<(&str, &str)>,
) {
    match node {
        ExplorerNode::Database { name } => row.label_spans(&[
            Span::new("D")
                .role(Role::Fg(FgStep::Muted))
                .remove_modifier(Modifier::BOLD),
            Span::new(" "),
            Span::new(name),
        ]),
        ExplorerNode::Schema { name } => row.label_spans(&[
            Span::new("S")
                .role(Role::Fg(FgStep::Muted))
                .remove_modifier(Modifier::BOLD),
            Span::new(" "),
            Span::new(name),
        ]),
        ExplorerNode::Group { name, count, .. } => {
            row.label(name);
            if show_meta {
                row.meta(count);
            }
        }
        ExplorerNode::Object {
            item,
            prefix,
            count,
        } => {
            if show_meta {
                row.meta(count);
            }
            let is_active = active_table == Some((item.schema.as_str(), item.name.as_str()));
            if is_active {
                let rem = row.remaining_width();
                let prefix_len = prefix.chars().count();
                let used_before_pad = prefix_len + 1 + item.name.chars().count();
                let pad = (rem as usize).saturating_sub(used_before_pad);
                let padded_name = format!("{}{}", item.name, " ".repeat(pad));
                row.label_spans(&[
                    Span::new(prefix)
                        .role(Role::Fg(FgStep::Muted))
                        .remove_modifier(Modifier::BOLD),
                    Span::new(" "),
                    Span::new(&padded_name).role(Role::Accent),
                ]);
            } else {
                row.label_spans(&[
                    Span::new(prefix)
                        .role(Role::Fg(FgStep::Muted))
                        .remove_modifier(Modifier::BOLD),
                    Span::new(" "),
                    Span::new(&item.name),
                ]);
            }
        }
    }
}

fn tab_key(tab: &TabRecord) -> ItemKey {
    ItemKey::num(tab.key().get())
}

fn tab_row(tab: &TabRecord, row: &mut RowUi<'_>) {
    match tab.payload() {
        Tab::Table(table) => {
            row.label_spans(&[
                Span::new("T")
                    .role(Role::Fg(FgStep::Muted))
                    .remove_modifier(termrock::Modifier::BOLD),
                Span::new(" "),
                Span::new(&table.table.name),
            ]);
        }
        Tab::Query(query) => {
            row.label_spans(&[
                Span::new("≡")
                    .role(Role::Fg(FgStep::Muted))
                    .remove_modifier(termrock::Modifier::BOLD),
                Span::new(" "),
                Span::new(&query.name),
            ]);
        }
        Tab::History(_) => {
            row.label_spans(&[
                Span::new("H")
                    .role(Role::Fg(FgStep::Muted))
                    .remove_modifier(termrock::Modifier::BOLD),
                Span::new(" "),
                Span::new("History"),
            ]);
        }
    }
}

fn connection_tree() -> Tree<
    'static,
    ConnectionNode,
    impl Fn(&ConnectionNode) -> ItemKey,
    impl Fn(&ConnectionNode, &mut RowUi<'_>),
> {
    connection_tree_with_meta(true)
}

fn connection_tree_with_meta(
    show_meta: bool,
) -> Tree<
    'static,
    ConnectionNode,
    impl Fn(&ConnectionNode) -> ItemKey,
    impl Fn(&ConnectionNode, &mut RowUi<'_>),
> {
    Tree::new(CONNECTIONS)
        .key(connection_node_key)
        .node(&connection_node)
        .row(move |node, row| connection_row_with_meta(node, row, show_meta))
}

fn explorer_tree() -> Tree<
    'static,
    ExplorerNode,
    impl Fn(&ExplorerNode) -> ItemKey,
    impl Fn(&ExplorerNode, &mut RowUi<'_>),
> {
    explorer_tree_with_meta(true, None)
}

const EXPLORER_MARKER_PATCH: &[(Part, termrock::StylePatch)] = &[(
    Part::MARKER,
    termrock::StylePatch {
        glyph: termrock::Slot::Clear,
        fg: termrock::Slot::Set(termrock::Role::Fg(termrock::FgStep::Primary)),
        ..termrock::StylePatch::new()
    },
)];

fn explorer_tree_with_meta(
    show_meta: bool,
    active_table: Option<(String, String)>,
) -> Tree<
    'static,
    ExplorerNode,
    impl Fn(&ExplorerNode) -> ItemKey,
    impl Fn(&ExplorerNode, &mut RowUi<'_>),
> {
    Tree::new(EXPLORER)
        .key(explorer_node_key)
        .node(&explorer_node)
        .branch_activation(termrock::TreeBranchActivation::Activate)
        .branch_click(termrock::TreeBranchClick::Choose)
        .patch_part(EXPLORER_MARKER_PATCH)
        .row(move |node, row| {
            let active = active_table.as_ref().map(|(s, n)| (s.as_str(), n.as_str()));
            explorer_row(node, row, show_meta, active);
        })
}

fn tab_strip()
-> Tabs<'static, TabRecord, impl Fn(&TabRecord) -> ItemKey, impl Fn(&TabRecord, &mut RowUi<'_>)> {
    Tabs::new(TAB_STRIP)
        .key(tab_key)
        .row(tab_row)
        .allow_new(true)
        .closable(true)
}

fn legacy_tree_body(inner: termrock::Rect) -> termrock::Rect {
    termrock::Rect {
        x: inner.x.saturating_sub(1),
        width: inner.width.saturating_add(1),
        ..inner
    }
}

fn paint_legacy_tree_gutters<T>(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    state: &TreeState,
    nodes: &[T],
    node: impl Fn(&T) -> TreeNode,
    key: impl Fn(&T) -> ItemKey,
    focused: bool,
) {
    if area.is_empty() {
        return;
    }
    let mut ancestors_expanded = Vec::new();
    let mut visible_row = 0usize;
    let first_visible = state.scroll().offset();
    let cursor = state.cursor();
    let gutter_style = ui.surface_style().with_fg_from_bg(ui.surface_style());

    for item in nodes {
        let descriptor = node(item);
        let depth = usize::from(descriptor.depth());
        ancestors_expanded.truncate(depth);
        let visible = ancestors_expanded.iter().all(|expanded| *expanded);
        if visible {
            let row = visible_row.saturating_sub(first_visible);
            if visible_row >= first_visible
                && row < usize::from(area.height)
                && (!focused || cursor != Some(key(item)))
            {
                ui.paint_str(
                    termrock::Rect {
                        x: area.x,
                        y: area.y.saturating_add(row as u16),
                        width: 1,
                        height: 1,
                    },
                    " ",
                    gutter_style,
                );
            }
            if matches!(descriptor.kind(), NodeKind::Leaf)
                && visible_row >= first_visible
                && row < usize::from(area.height)
            {
                ui.fill(
                    termrock::Rect {
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
        .slot(Part::SEAM, &|_, _| {})
}
fn result_grid<'a>(id: Id, columns: &'a [termrock::Column<'a>]) -> Grid<'a> {
    Grid::new(id, columns)
        .blur(termrock::BlurPolicy::Keep)
        .nav(termrock::NavUnit::Cell)
        .select_mode(termrock::SelectMode::Multi)
        .sort_indicator(termrock::GridSortIndicator::ActiveOnly)
        .column_gap(2)
        .column_fit(termrock::GridColumnFit::CompleteWithPreview { min_width: 6 })
        .header_sizing(termrock::GridHeaderSizing::Minimum { padding: 2, cap: 24 })
        .right_reserve(4)
        .gutter(termrock::GridGutter::Detailed {
            row_numbers: true,
            min_digits: 2,
        })
}

fn draw_filter_chips(ui: &mut Ui<'_>, area: termrock::Rect, table: &TableTab) {
    let mut x = area.x;
    let y = area.y;
    let muted_style = ui.surface_style().patch(
        ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))),
    );
    let secondary_style = ui.surface_style().patch(
        ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))),
    );
    let chip_bg_style = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_bg(Role::Surface(termrock::Surface::Overlay))
                .set_fg(Role::Fg(FgStep::Primary)),
        ),
    );
    let chip_gutter_style = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Surface(termrock::Surface::Overlay))
                .set_bg(Role::Surface(termrock::Surface::Overlay)),
        ),
    );
    let button_gutter_style = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Surface(termrock::Surface::Canvas))
                .set_bg(Role::Surface(termrock::Surface::Canvas)),
        ),
    );

    // 1. " match all ▾ "
    let lead_text = if table.match_all {
        " match all ▾ "
    } else {
        " match any ▾ "
    };
    let lead_w = lead_text.chars().count() as u16;
    ui.paint_str(termrock::Rect::new(x, y, lead_w, 1), lead_text, muted_style);
    x = x.saturating_add(lead_w).saturating_add(1);

    // 2. Chips
    for f in &table.filters {
        let label = f.chip_label();
        let label_w = (label.chars().count() + 1) as u16;
        let w = 1 + label_w + 1 + 2;
        if x.saturating_add(w) > area.right() {
            ui.paint_str(termrock::Rect::new(x, y, 1, 1), "…", muted_style);
            return;
        }
        let chip_rect = termrock::Rect::new(x, y, w, 1);
        ui.fill(chip_rect, chip_bg_style);
        ui.paint_str(termrock::Rect::new(x, y, 1, 1), " ", chip_gutter_style);
        let text_with_space = format!("{label} ");
        ui.paint_str(
            termrock::Rect::new(x.saturating_add(1), y, label_w, 1),
            &text_with_space,
            chip_bg_style,
        );
        let x_style = chip_bg_style.patch(
            ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))),
        );
        ui.paint_str(
            termrock::Rect::new(x.saturating_add(1).saturating_add(label_w), y, 1, 1),
            "×",
            x_style,
        );
        x = x.saturating_add(w).saturating_add(1);
    }

    // 3. "+ Add filter"
    let add_text = "+ Add filter ";
    let add_w = add_text.chars().count() as u16;
    if x.saturating_add(add_w).saturating_add(1) <= area.right() {
        ui.paint_str(termrock::Rect::new(x, y, 1, 1), " ", button_gutter_style);
        ui.paint_str(
            termrock::Rect::new(x.saturating_add(1), y, add_w, 1),
            add_text,
            secondary_style,
        );
    }
}

fn shell_parts(area: termrock::Rect) -> [termrock::Rect; 3] {
    let body_y = area.y.saturating_add(2);
    let body_height = area.height.saturating_sub(4);
    [
        termrock::Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1.min(area.height),
        },
        termrock::Rect {
            x: area.x.saturating_add(1),
            y: body_y,
            width: area.width.saturating_sub(2),
            height: body_height,
        },
        termrock::Rect {
            x: area.x,
            y: area.bottom().saturating_sub(1),
            width: area.width,
            height: 1.min(area.height),
        },
    ]
}

fn preserve_frame_gutter(_: &mut Ui<'_>, _: termrock::Rect) {}

fn paint_legacy_filter(ui: &mut Ui<'_>, area: termrock::Rect, text: &str, focused: bool) {
    if area.is_empty() {
        return;
    }
    let field = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_bg(Role::Surface(termrock::Surface::Field))));
    ui.fill(area, field);

    if focused {
        let accent_style = field.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
        ui.paint_str(
            termrock::Rect {
                width: 1.min(area.width),
                ..area
            },
            "▎",
            accent_style,
        );

        if area.width > 2 {
            let label = field.patch(
                ui.paint_patch(
                    &StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Primary))
                        .add(Modifier::UNDERLINED),
                ),
            );
            ui.paint_str(
                termrock::Rect {
                    x: area.x.saturating_add(2),
                    width: area.width.saturating_sub(2),
                    ..area
                },
                text,
                label,
            );
        }
    } else {
        let gutter = field.with_fg_from_bg(field);
        ui.paint_str(
            termrock::Rect {
                width: 1.min(area.width),
                ..area
            },
            " ",
            gutter,
        );

        if area.width > 2 {
            let label =
                field.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
            ui.paint_str(
                termrock::Rect {
                    x: area.x.saturating_add(2),
                    width: area.width.saturating_sub(2),
                    ..area
                },
                text,
                label,
            );
        }
    }
}

struct HeaderSegment {
    text: String,
    role: Option<Role>,
    bold: bool,
    clickable: bool,
    priority: u8,
}

fn draw_header(ui: &mut Ui<'_>, area: termrock::Rect, app: &TableProApp) {
    let base = ui.surface_style();
    ui.fill(area, base);

    let mut left = vec![
        HeaderSegment {
            text: "▪".to_string(),
            role: Some(Role::Success),
            bold: false,
            clickable: false,
            priority: 9,
        },
        HeaderSegment {
            text: "TablePro".to_string(),
            role: None,
            bold: true,
            clickable: false,
            priority: 9,
        },
    ];
    let mut right = Vec::new();
    match app.screen {
        Screen::Connections => {
            left.push(HeaderSegment {
                text: "Connections".to_string(),
                role: Some(Role::Fg(FgStep::Secondary)),
                bold: false,
                clickable: false,
                priority: 8,
            });
            let n = app.connections.len();
            left.push(HeaderSegment {
                text: format!("{n} saved"),
                role: Some(Role::Fg(FgStep::Muted)),
                bold: false,
                clickable: false,
                priority: 3,
            });
        }
        Screen::Workbench => {
            let c = &app.connection;
            let env = c.environment;
            let (env_text, env_role, env_bold) = match env {
                Environment::Production => ("◆ production", None, true),
                Environment::Staging => ("◇ staging", Some(Role::Fg(FgStep::Secondary)), false),
                Environment::Development => ("development", Some(Role::Fg(FgStep::Muted)), false),
                Environment::Local => ("local", Some(Role::Fg(FgStep::Faint)), false),
            };
            left.push(HeaderSegment {
                text: termrock::truncate_middle(&c.name, 18),
                role: None,
                bold: true,
                clickable: true,
                priority: 9,
            });
            left.push(HeaderSegment {
                text: env_text.to_string(),
                role: env_role,
                bold: env_bold,
                clickable: false,
                priority: 8,
            });
            let scope_text = format!("{} › {}", app.workbench.catalog.database, app.workbench.current_schema());
            left.push(HeaderSegment {
                text: scope_text,
                role: Some(Role::Fg(FgStep::Secondary)),
                bold: false,
                clickable: true,
                priority: 7,
            });
            let level = c.safe_mode;
            let (tone_role, tone_bold) = match level {
                SafeMode::Silent if env == Environment::Production => (Some(Role::Warning), true),
                SafeMode::Silent => (Some(Role::Fg(FgStep::Faint)), false),
                SafeMode::Alert | SafeMode::AlertFull => (Some(Role::Fg(FgStep::Secondary)), false),
                _ => (None, true),
            };
            left.push(HeaderSegment {
                text: level.token().to_string(),
                role: tone_role,
                bold: tone_bold,
                clickable: true,
                priority: 8,
            });
            if matches!(app.workbench.active(), Some(Tab::Query(q)) if q.running) {
                right.push(HeaderSegment {
                    text: format!("{} running", "⠋"),
                    role: Some(Role::Fg(FgStep::Secondary)),
                    bold: false,
                    clickable: false,
                    priority: 9,
                });
            }
            let mut pending: usize = 0;
            for tab in app.workbench.tabs() {
                match tab.payload() {
                    Tab::Table(t) => pending = pending.saturating_add(t.result.pending_total()),
                    Tab::Query(q) => {
                        if let Some(grid) = &q.result {
                            pending = pending.saturating_add(grid.pending_total());
                        }
                    }
                    _ => {}
                }
            }
            if pending > 0 {
                right.push(HeaderSegment {
                    text: format!("• {pending} pending"),
                    role: Some(Role::Warning),
                    bold: false,
                    clickable: false,
                    priority: 8,
                });
            }
        }
    }

    let capability = ui.theme().capability.color.label();
    let dimensions = format!("{}×{}", area.width, ui.full().height);
    right.push(HeaderSegment {
        text: format!("{capability} · {dimensions}"),
        role: Some(Role::Fg(FgStep::Faint)),
        bold: false,
        clickable: false,
        priority: 1,
    });
    right.push(HeaderSegment {
        text: "? help".to_string(),
        role: Some(Role::Fg(FgStep::Muted)),
        bold: false,
        clickable: true,
        priority: 4,
    });

    let sep = 2u16;
    let w = |s: &HeaderSegment| termrock::width(&s.text) as u16;
    let mut keep_l: Vec<bool> = vec![true; left.len()];
    let mut keep_r: Vec<bool> = vec![true; right.len()];
    let total = |kl: &[bool], kr: &[bool]| -> u16 {
        let l: u16 = left
            .iter()
            .zip(kl)
            .filter(|(_, k)| **k)
            .map(|(s, _)| w(s) + sep)
            .sum();
        let r: u16 = right
            .iter()
            .zip(kr)
            .filter(|(_, k)| **k)
            .map(|(s, _)| w(s) + sep)
            .sum();
        l + r + 2
    };
    while total(&keep_l, &keep_r) > area.width {
        let mut best: Option<(u8, bool, usize)> = None;
        for (i, s) in left.iter().enumerate() {
            if keep_l[i] && best.is_none_or(|b| s.priority < b.0) {
                best = Some((s.priority, true, i));
            }
        }
        for (i, s) in right.iter().enumerate() {
            if keep_r[i] && best.is_none_or(|b| s.priority <= b.0) {
                best = Some((s.priority, false, i));
            }
        }
        match best {
            Some((_, true, i)) => keep_l[i] = false,
            Some((_, false, i)) => keep_r[i] = false,
            None => break,
        }
    }

    let draw_segment = |ui: &mut Ui<'_>, x: u16, s: &HeaderSegment| {
        let mut patch = StylePatch::new();
        if let Some(role) = s.role {
            patch = patch.set_fg(role);
        }
        if s.bold {
            patch = patch.add(Modifier::BOLD);
        }
        let style = base.patch(ui.paint_patch(&patch));
        let text = if s.clickable {
            format!(" {} ", s.text)
        } else {
            s.text.clone()
        };
        let text_w = termrock::width(&text) as u16;
        let rect = termrock::Rect::new(x, area.y, text_w, 1);
        ui.paint_str(rect, &text, style);
    };

    let mut x = area.x + 1;
    for (s, k) in left.iter().zip(&keep_l) {
        if !k {
            continue;
        }
        let sw = w(s) + if s.clickable { 2 } else { 0 };
        let start = if s.clickable {
            x.saturating_sub(1)
        } else {
            x
        };
        draw_segment(ui, start, s);
        x += sw.saturating_sub(if s.clickable { 2 } else { 0 }) + sep;
    }
    let mut rx = area.right().saturating_sub(1);
    for (s, k) in right.iter().zip(&keep_r).rev() {
        if !k {
            continue;
        }
        let sw = w(s);
        rx = rx.saturating_sub(sw);
        let start = if s.clickable {
            rx.saturating_sub(1)
        } else {
            rx
        };
        draw_segment(ui, start, s);
        rx = rx.saturating_sub(sep);
    }
}

#[derive(Clone, Copy)]
struct KeyHint {
    key: &'static str,
    action: &'static str,
}

fn footer_hints(app: &TableProApp, explorer_focused: bool) -> &'static [KeyHint] {
    if app.help_open {
        return &[
            KeyHint {
                key: "← →",
                action: "Choose",
            },
            KeyHint {
                key: "Enter",
                action: "Confirm",
            },
            KeyHint {
                key: "Esc",
                action: "Cancel",
            },
            KeyHint {
                key: "y / n",
                action: "Quick answer",
            },
        ];
    }
    if app.switcher_open || app.tab_list_open || app.safe_mode_open {
        return &[];
    }
    if app.filter_editor.is_some() {
        return &[
            KeyHint {
                key: "Tab",
                action: "Next field",
            },
            KeyHint {
                key: "Enter",
                action: "Apply",
            },
            KeyHint {
                key: "Esc",
                action: "Cancel",
            },
        ];
    }
    if let Some(dlg) = app.safety_dialog.as_ref() {
        if dlg.input_editing {
            return &[
                KeyHint {
                    key: "Enter",
                    action: "Next",
                },
                KeyHint {
                    key: "Esc",
                    action: "Cancel",
                },
            ];
        } else {
            return &[
                KeyHint {
                    key: "← →",
                    action: "Choose",
                },
                KeyHint {
                    key: "Enter",
                    action: "Confirm",
                },
                KeyHint {
                    key: "Esc",
                    action: "Cancel",
                },
            ];
        }
    }
    if app.destructive_intent.is_some() {
        return &[
            KeyHint {
                key: "← →",
                action: "Choose",
            },
            KeyHint {
                key: "Enter",
                action: "Confirm",
            },
            KeyHint {
                key: "Esc",
                action: "Cancel",
            },
            KeyHint {
                key: "y / n",
                action: "Quick answer",
            },
        ];
    }
    if app.form_open {
        return &[
            KeyHint {
                key: "Enter",
                action: "Edit",
            },
            KeyHint {
                key: "← →",
                action: "Basic / Advanced",
            },
            KeyHint {
                key: "Ctrl+S",
                action: "Save",
            },
            KeyHint {
                key: "Tab",
                action: "Next",
            },
        ];
    }
    if app.screen == Screen::Connections {
        if app.connections_screen.filter_active {
            return &[
                KeyHint {
                    key: "Type",
                    action: "Filter",
                },
                KeyHint {
                    key: "↓",
                    action: "Into list",
                },
                KeyHint {
                    key: "Esc",
                    action: "Clear",
                },
            ];
        }
        &[
            KeyHint {
                key: "↑ ↓",
                action: "Move",
            },
            KeyHint {
                key: "Enter",
                action: "Connect",
            },
            KeyHint {
                key: "E",
                action: "Edit",
            },
            KeyHint {
                key: "D",
                action: "Delete",
            },
            KeyHint {
                key: "Ctrl+D",
                action: "Duplicate",
            },
            KeyHint {
                key: "/",
                action: "Filter",
            },
            KeyHint {
                key: "Ctrl+N",
                action: "New",
            },
            KeyHint {
                key: "Tab",
                action: "Next",
            },
        ]
    } else {
        if explorer_focused {
            return &[
                KeyHint {
                    key: "↑ ↓",
                    action: "Move",
                },
                KeyHint {
                    key: "Enter",
                    action: "Open",
                },
                KeyHint {
                    key: "→",
                    action: "Expand",
                },
                KeyHint {
                    key: "/",
                    action: "Filter",
                },
                KeyHint {
                    key: "Ctrl+O",
                    action: "Quick open",
                },
                KeyHint {
                    key: "Tab",
                    action: "Next",
                },
            ];
        }
        match app.workbench.active() {
            Some(Tab::Table(t)) if t.is_structure() => &[
                KeyHint {
                    key: "↑ ↓",
                    action: "Move",
                },
                KeyHint {
                    key: "Ctrl+D",
                    action: "Structure",
                },
                KeyHint {
                    key: "Tab",
                    action: "Next",
                },
            ],
            Some(Tab::Table(t)) => {
                if t.result.state.is_editing() || t.structure.state.is_editing() {
                    return &[
                        KeyHint {
                            key: "Enter",
                            action: "Commit",
                        },
                        KeyHint {
                            key: "Esc",
                            action: "Cancel",
                        },
                        KeyHint {
                            key: "Tab",
                            action: "Next cell",
                        },
                    ];
                }
                if t.result.model.is_editable() {
                    if t.result.pending_total() > 0 {
                        &[
                            KeyHint {
                                key: "↑↓←→",
                                action: "Cell",
                            },
                            KeyHint {
                                key: "Enter",
                                action: "Edit",
                            },
                            KeyHint {
                                key: "Alt+D",
                                action: "Duplicate row",
                            },
                            KeyHint {
                                key: "s",
                                action: "Sort",
                            },
                            KeyHint {
                                key: "f",
                                action: "Filter",
                            },
                            KeyHint {
                                key: "Ctrl+S",
                                action: "Save",
                            },
                            KeyHint {
                                key: "Tab",
                                action: "Next",
                            },
                        ]
                    } else {
                        &[
                            KeyHint {
                                key: "↑↓←→",
                                action: "Cell",
                            },
                            KeyHint {
                                key: "Enter",
                                action: "Edit",
                            },
                            KeyHint {
                                key: "Alt+D",
                                action: "Duplicate row",
                            },
                            KeyHint {
                                key: "s",
                                action: "Sort",
                            },
                            KeyHint {
                                key: "f",
                                action: "Filter",
                            },
                            KeyHint {
                                key: "Space",
                                action: "Select row",
                            },
                            KeyHint {
                                key: "Tab",
                                action: "Next",
                            },
                        ]
                    }
                } else {
                    &[
                        KeyHint {
                            key: "↑↓←→",
                            action: "Cell",
                        },
                        KeyHint {
                            key: "Enter",
                            action: "Edit",
                        },
                        KeyHint {
                            key: "s",
                            action: "Sort",
                        },
                        KeyHint {
                            key: "f",
                            action: "Filter",
                        },
                        KeyHint {
                            key: "Space",
                            action: "Select row",
                        },
                        KeyHint {
                            key: "Tab",
                            action: "Next",
                        },
                    ]
                }
            }
            Some(Tab::Query(_)) => &[
                KeyHint {
                    key: "Enter",
                    action: "Edit",
                },
                KeyHint {
                    key: "Ctrl+R",
                    action: "Run",
                },
                KeyHint {
                    key: "Alt+R",
                    action: "Run all",
                },
                KeyHint {
                    key: "Ctrl+X",
                    action: "Explain",
                },
                KeyHint {
                    key: "/",
                    action: "Find",
                },
                KeyHint {
                    key: "Tab",
                    action: "Next",
                },
            ],
            Some(Tab::History(_)) => &[
                KeyHint {
                    key: "↑ ↓",
                    action: "Move",
                },
                KeyHint {
                    key: "Enter",
                    action: "Open",
                },
                KeyHint {
                    key: "/",
                    action: "Filter",
                },
                KeyHint {
                    key: "Tab",
                    action: "Next",
                },
            ],
            None => &[
                KeyHint {
                    key: "Ctrl+N",
                    action: "New query",
                },
                KeyHint {
                    key: "Ctrl+O",
                    action: "Quick open",
                },
                KeyHint {
                    key: "Tab",
                    action: "Next",
                },
            ],
        }
    }
}

fn draw_footer(ui: &mut Ui<'_>, area: termrock::Rect, app: &TableProApp) {
    let base = ui.surface_style();
    if !app.form_open {
        ui.fill(area, base);
    }
    if area.is_empty() {
        return;
    }
    let mut right_w = 0u16;
    if let Some(notice) = app.destructive_notice {
        let width = termrock::width(notice).min(area.width);
        let right = termrock::Rect {
            x: area.right().saturating_sub(width).saturating_sub(1),
            width,
            ..area
        };
        ui.fill(right, base);
        ui.paint_str(right, notice, base);
        right_w = width.saturating_add(3);
    } else if !app.status.is_empty()
        && (app.destructive_intent.is_none() || app.screen != Screen::Connections)
    {
        let width = termrock::width(&app.status);
        if width > 0 && width < area.width {
            let right = termrock::Rect {
                x: area.right().saturating_sub(width).saturating_sub(1),
                width,
                ..area
            };
            let status_style = if app.form_open {
                ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)))
            } else {
                base.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))))
            };
            ui.paint_str(right, &app.status, status_style);
            right_w = width.saturating_add(3);
        }
    }

    let limit = area.right().saturating_sub(right_w);
    let mut x = area.x.saturating_add(1);
    if app.is_editing() && app.safety_dialog.is_none() && app.filter_editor.is_none() {
        let badge = " EDIT ";
        let badge_w = termrock::width(badge);
        let badge_style = base.patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::OnAccent)
                    .set_bg(Role::Accent)
                    .add(Modifier::BOLD),
            ),
        );
        ui.paint_str(
            termrock::Rect {
                x,
                y: area.y,
                width: badge_w,
                height: 1,
            },
            badge,
            badge_style,
        );
        x = x.saturating_add(badge_w).saturating_add(2);
    }
    let (key_style, action_style, faint_style) = if app.form_open {
        (
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .add(Modifier::BOLD),
            ),
            ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))),
            ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))),
        )
    } else {
        (
            base.patch(
                ui.paint_patch(
                    &StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Primary))
                        .add(Modifier::BOLD),
                ),
            ),
            base.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)))),
            base.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint)))),
        )
    };

    let explorer_focused = !ui.is_inert()
        && app.safety_dialog.is_none()
        && app.destructive_intent.is_none()
        && !app.help_open
        && !app.switcher_open
        && !app.tab_list_open
        && !app.safe_mode_open
        && (app.workbench_focus == EXPLORER
            || ui.state(EXPLORER).contains(termrock::StateFlags::FOCUSED)
            || (area.width < 100 && app.workbench.active().is_none()));
    let hints = footer_hints(app, explorer_focused);
    let mut drawn = 0usize;
    for (i, h) in hints.iter().enumerate() {
        let kw = termrock::width(h.key);
        let aw = termrock::width(h.action);
        let w = kw.saturating_add(1).saturating_add(aw).saturating_add(2);
        let reserve = if i.saturating_add(1) < hints.len() {
            2
        } else {
            0
        };
        if x.saturating_add(w).saturating_add(reserve) > limit {
            break;
        }
        ui.paint_str(
            termrock::Rect {
                x,
                y: area.y,
                width: kw,
                height: 1,
            },
            h.key,
            key_style,
        );
        ui.paint_str(
            termrock::Rect {
                x: x.saturating_add(kw).saturating_add(1),
                y: area.y,
                width: aw,
                height: 1,
            },
            h.action,
            action_style,
        );
        x = x.saturating_add(w);
        drawn += 1;
    }
    if drawn < hints.len() && x < limit {
        ui.paint_str(
            termrock::Rect {
                x,
                y: area.y,
                width: 1,
                height: 1,
            },
            "…",
            faint_style,
        );
    }
}

fn draw_too_small(ui: &mut Ui<'_>, area: termrock::Rect) {
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
        let width = termrock::width(line);
        let row = termrock::Rect {
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
    area: termrock::Rect,
    properties: &[(&str, String, Role)],
) {
    let value_x = area.x.saturating_add(13);
    let value_width = area.width.saturating_sub(13).max(1);
    let label_style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
    let mut y = area.y;
    for (label, value, role) in properties {
        let value_style = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(*role)));
        let lines = if *label == "Safe Mode" || label.is_empty() {
            wrap(value, value_width)
        } else {
            vec![value.clone()]
        };
        for (line_index, line) in lines.iter().enumerate() {
            if y >= area.bottom().saturating_sub(2) {
                break;
            }
            if line_index == 0 && !label.is_empty() {
                ui.paint_str(
                    termrock::Rect {
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
                termrock::Rect {
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
    draw_connection_actions(ui, area);
}

fn draw_connection_actions(ui: &mut Ui<'_>, area: termrock::Rect) {
    let y = area.bottom().saturating_sub(1);
    let mut x = area.x;
    paint_action_button(
        ui,
        x,
        y,
        "Connect",
        action_paint(ui, CONNECT_LABEL, Role::Accent, Role::OnAccent),
        true,
    );
    x = x.saturating_add(11);
    paint_action_button(
        ui,
        x,
        y,
        "Edit",
        action_paint(
            ui,
            EDIT_LABEL,
            Role::Surface(termrock::Surface::Overlay),
            Role::Fg(FgStep::Primary),
        ),
        false,
    );
    x = x.saturating_add(8);
    paint_action_button(
        ui,
        x,
        y,
        "Duplicate",
        action_paint(
            ui,
            DUPLICATE_LABEL,
            Role::Surface(termrock::Surface::Surface),
            Role::Fg(FgStep::Secondary),
        ),
        false,
    );
    x = x.saturating_add(13);
    paint_action_button(
        ui,
        x,
        y,
        "Delete…",
        action_paint(
            ui,
            DELETE_LABEL,
            Role::Surface(termrock::Surface::Overlay),
            Role::Danger,
        ),
        false,
    );
}

const CONNECTION_ACTIONS: termrock::Family =
    termrock::Family::custom("tablepro.connection-actions");
const CONNECT_LABEL: Part = Part::custom("connect.label");
const EDIT_LABEL: Part = Part::custom("edit.label");
const DUPLICATE_LABEL: Part = Part::custom("duplicate.label");
const DELETE_LABEL: Part = Part::custom("delete.label");
fn action_paint(ui: &Ui<'_>, part: Part, background: Role, foreground: Role) -> PaintStyle {
    ui.style_defaults(
        CONNECTION_ACTIONS,
        termrock::Variant::DEFAULT,
        part,
        termrock::StateFlags::empty(),
        StyleDefaults::new(StylePatch::new().set_bg(background).set_fg(foreground)),
        None,
    )
    .over(ui.surface_style())
}

fn paint_action_button(
    ui: &mut Ui<'_>,
    x: u16,
    y: u16,
    label: &str,
    mut button: PaintStyle,
    bold: bool,
) {
    let width = termrock::width(label).saturating_add(2);
    if bold {
        button = button.add_modifier(Modifier::BOLD);
    }
    ui.fill(
        termrock::Rect {
            x,
            y,
            width,
            height: 1,
        },
        button,
    );
    let gutter = button
        .remove_modifier(Modifier::BOLD)
        .with_fg_from_bg(button);
    ui.paint_str(
        termrock::Rect {
            x,
            y,
            width: 1,
            height: 1,
        },
        " ",
        gutter,
    );
    ui.paint_str(
        termrock::Rect {
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
        if cx.update_cause() == UpdateCause::Bootstrap {
            if self.screen == Screen::Workbench && self.surface == Surface::QuickSwitcher {
                self.open_switcher(cx);
            } else if self.screen == Screen::Workbench && self.surface == Surface::TabListPicker {
                self.open_tab_list(cx);
            } else if self.screen == Screen::Workbench && self.surface == Surface::SafeModePicker {
                self.open_safe_mode_picker(cx);
            } else if self.screen == Screen::Workbench && self.surface == Surface::HelpDialog {
                self.open_help(cx);
            } else if self.screen == Screen::Connections {
                if self.form_open {
                    cx.focus(connections::field::NAME);
                } else {
                    cx.focus(CONNECTIONS);
                }
            } else if self.screen == Screen::Workbench {
                cx.focus(EXPLORER);
            }
        }
        let overlay_was_open = cx.is_open(quick_switcher::ID)
            || cx.is_open(tab_list::ID)
            || cx.is_open(safe_mode_picker::ID)
            || cx.is_open(HELP_DIALOG_ID);
        let modal_was_open = self.destructive_intent.is_some()
            || self.safety_dialog.is_some()
            || self.filter_editor.is_some();
        let mut response = self.update_destructive_dialog(cx);
        response |= self.update_safety_dialog(cx);
        response |= self.update_filter_editor(cx);
        response |= self.update_tab_controls(cx);
        if self.screen == Screen::Workbench
            && cx.intents(EXPLORER).any(|it| matches!(it, Intent::FocusIn { .. }))
        {
            self.workbench_focus = EXPLORER;
        }
        if !self.status.is_empty() && self.status_since.is_none() {
            self.status_since = Some(cx.now());
        }
        if let Some(left) = self.committing.as_mut() {
            *left = left.saturating_sub(1);
            if *left == 0 {
                self.committing = None;
                self.finish_commit(cx);
                response = response.repaint();
            } else {
                cx.request_repaint_after(std::time::Duration::from_millis(80));
            }
            response |= Response::changed();
        }
        if cx.update_cause() == termrock::UpdateCause::Tick
            && self.status_since.is_some_and(|since| {
                cx.now().saturating_duration_since(since) >= std::time::Duration::from_secs(5)
            })
        {
            self.status.clear();
            self.status_since = None;
            response = response.repaint();
        }
        if let Some(since) = self.status_since {
            let deadline = since
                .saturating_add(std::time::Duration::from_secs(5))
                .saturating_add(std::time::Duration::from_nanos(1));
            cx.request_repaint_at(deadline);
        }
        if self.form_open || self.screen != Screen::Connections {
            response |= connection_tree()
                .update(cx, &mut self.connection_tree_state, &self.connection_nodes)
                .erase();
            for _ in cx.intents(CONNECTION_DETAILS) {}
            for _ in cx.intents(CONNECTION_FILTER) {}
        }
        if self.form_open || self.screen != Screen::Workbench {
            response |= workbench_split().update(cx, &mut self.split_state).erase();
            response |= explorer_tree()
                .update(cx, &mut self.explorer_tree_state, &self.explorer_nodes)
                .erase();
            response |= tab_strip()
                .update(cx, &mut self.tabs_state, self.workbench.tabs())
                .erase();
        }
        response |= self.update_switcher(cx);
        response |= self.update_tab_list(cx);
        response |= self.update_safe_mode_picker(cx);
        response |= self.update_help_dialog(cx);
        if overlay_was_open {
            return response;
        }
        // Stateless props have no update method, but their factories remain
        // the single source of configuration for both runtime phases.
        let _ = Self::connections_panel("", None, true);
        let _ = Self::connection_details_panel("");
        let _ = Self::explorer_panel(self.workbench.schema_caption(), false);
        let _ = Self::content_panel("", None, false);
        if matches!(
            cx.update_cause(),
            UpdateCause::Bootstrap | UpdateCause::Event
        ) && !modal_was_open
            && cx.top_layer() == LayerId::PAGE
            && let Some(command) = cx.command()
        {
            match command {
                c if c == QUIT => {
                    return self.request_quit(cx);
                }
                c if c == CANCEL_OR_QUIT => {
                    if cx.top_layer() != LayerId::PAGE {
                        return Response::consumed();
                    }
                    if let Some(Tab::Query(query)) = self.workbench.active_mut()
                        && query.running
                    {
                        query.running = false;
                        "Query cancelled".clone_into(&mut self.status);
                        return Response::changed();
                    }
                    return self.request_quit(cx);
                }
                c if c == INSERT_ROW || c == DUPLICATE_ROW || c == DELETE_ROW || c == DISCARD_ROWS => {
                    if self.active_row_action(cx) {
                        if c == INSERT_ROW {
                            self.insert_active_row();
                        } else if c == DUPLICATE_ROW {
                            self.duplicate_active_row();
                        } else if c == DELETE_ROW {
                            self.toggle_active_row();
                        } else {
                            self.request_discard_rows(cx);
                        }
                        response |= Response::changed();
                    }
                }
                c if c == UNDO => {
                    if let Some((id, grid)) = self.workbench.active_grid_mut()
                        && cx.state(id).contains(termrock::StateFlags::FOCUSED)
                        && !grid.state.is_editing()
                    {
                        let _ = grid.model.undo();
                        response |= Response::changed();
                    }
                }
                c if c == RUN => {
                    self.request_query(cx);
                    response |= Response::changed();
                }
                c if c == SAVE => {
                    self.request_save(cx);
                    response |= Response::changed();
                }
                c if c == FOCUS_EXPLORER => {
                    if self.screen == Screen::Workbench && !self.is_editing() {
                        self.workbench.explorer_visible = true;
                        self.workbench.maximized = false;
                        if let Some(Tab::Query(q)) = self.workbench.active_mut() {
                            q.maximized = QueryPaneMaximized::None;
                        }
                        self.workbench_focus = EXPLORER;
                        cx.focus(EXPLORER);
                        response |= Response::changed();
                    }
                }
                c if c == TOGGLE_EXPLORER => {
                    if self.screen == Screen::Workbench {
                        self.workbench.explorer_visible = !self.workbench.explorer_visible;
                        if self.workbench.explorer_visible {
                            self.workbench_focus = EXPLORER;
                            cx.focus(EXPLORER);
                        } else if let Some(tab_key) = self.workbench.active_key() {
                            if let Some(Tab::Table(_)) = self.workbench.active() {
                                let ctrl = tab_key.control("data");
                                self.workbench_focus = ctrl;
                                cx.focus(ctrl);
                            } else if let Some(Tab::Query(_)) = self.workbench.active() {
                                let ctrl = tab_key.control("query");
                                self.workbench_focus = ctrl;
                                cx.focus(ctrl);
                            }
                        }
                        response |= Response::changed();
                    }
                }
                c if c == MAXIMIZE => {
                    if self.screen == Screen::Workbench {
                        self.workbench.maximized = !self.workbench.maximized;
                        let maximized = self.workbench.maximized;
                        let focus = self.workbench_focus;
                        if let Some(tab_key) = self.workbench.active_key()
                            && let Some(Tab::Query(q)) = self.workbench.active_mut()
                        {
                            let query_id = tab_key.control("query");
                            let is_editor = focus == query_id
                                || cx.state(query_id).contains(termrock::StateFlags::FOCUSED);
                            if is_editor {
                                self.workbench_focus = query_id;
                            }
                            q.maximized = if maximized {
                                if is_editor {
                                    QueryPaneMaximized::Editor
                                } else {
                                    QueryPaneMaximized::Results
                                }
                            } else {
                                QueryPaneMaximized::None
                            };
                        }
                        response |= Response::changed();
                    }
                }
                c if c == OPEN => {
                    if self.screen == Screen::Workbench {
                        self.open_switcher(cx);
                        response |= Response::changed();
                    }
                }
                c if c == NEW_QUERY => {
                    self.new_query("");
                    if let Some(tab_key) = self.workbench.active_key() {
                        let ctrl = tab_key.control("query");
                        self.workbench_focus = ctrl;
                        cx.focus(ctrl);
                    }
                    response |= Response::changed();
                }
                c if c == CLOSE_TAB => {
                    if self.screen == Screen::Workbench && !self.is_editing()
                        && let Some(tab_key) = self.workbench.active_key() {
                            self.request_close_tab(cx, tab_key);
                            response |= Response::changed();
                        }
                }
                c if c == HISTORY => {
                    self.workbench.open_history();
                    self.sync_active_tab();
                    response |= Response::changed();
                }
                c if c == STRUCTURE => {
                    if self.screen == Screen::Connections {
                        self.duplicate_connection();
                        response |= Response::changed();
                    } else {
                        let _ = self.workbench.toggle_structure();
                        self.sync_active_tab();
                        if let Some(tab_key) = self.workbench.active_key()
                            && let Some(Tab::Table(t)) = self.workbench.active() {
                                if t.is_structure() {
                                    let ctrl = tab_key.control("structure");
                                    self.workbench_focus = ctrl;
                                    cx.focus(ctrl);
                                } else {
                                    let ctrl = tab_key.control("data");
                                    self.workbench_focus = ctrl;
                                    cx.focus(ctrl);
                                }
                            }
                        response |= Response::changed();
                    }
                }
                c if c == DELETE_CONNECTION => {
                    if self.screen == Screen::Connections {
                        self.request_delete_connection(cx);
                        response |= Response::changed();
                    }
                }
                c if c == EDIT_CONNECTION => {
                    if self.screen == Screen::Connections {
                        self.begin_edit_connection_form(self.connections_screen.selected);
                        cx.focus(connections::field::NAME);
                        response |= Response::changed();
                    }
                }
                c if c == FORM_TAB_PREV => {
                    if self.form_open {
                        self.form_tab = 0;
                        response |= Response::changed();
                    }
                }
                c if c == FORM_TAB_NEXT => {
                    if self.form_open {
                        self.form_tab = 1;
                        response |= Response::changed();
                    }
                }
                c if c == FORM => {
                    self.begin_connection_form();
                    cx.focus(connections::field::NAME);
                    response |= Response::changed();
                }
                c if c == FILTER => {
                    if self.screen == Screen::Connections && !self.form_open {
                        self.connections_screen.filter_active = true;
                        cx.focus(CONNECTION_FILTER);
                        response |= Response::changed();
                    } else if self.screen == Screen::Workbench
                        && let Some(Tab::Table(table)) = self.workbench.active()
                            && !table.is_structure() {
                                let (col_idx, cell_value) = if let Some((row_key, col_key)) =
                                    table.result.state.cursor()
                                {
                                    let c_idx = usize::from(col_key.raw()).saturating_sub(1);
                                    let row_idx = (0..table.result.model.row_count())
                                        .find(|r| table.result.model.row_key(*r) == row_key);
                                    let val = row_idx
                                        .and_then(|r| table.result.model.cell(r, c_idx))
                                        .map(|c| c.text.to_owned())
                                        .unwrap_or_default();
                                    (c_idx, val)
                                } else {
                                    (0, String::new())
                                };
                                let prefill = match cell_value.as_str() {
                                    "" | "NULL" => (col_idx, FilterOp::IsNull, String::new()),
                                    other => (col_idx, FilterOp::Eq, other.to_owned()),
                                };
                                self.open_filter_editor(cx, None, Some(prefill));
                                response |= Response::changed();
                            }
                }
                c if c == FILTER_EMPTY => {
                    if self.screen == Screen::Workbench
                        && let Some(Tab::Table(table)) = self.workbench.active()
                            && !table.is_structure() {
                                self.open_filter_editor(cx, None, None);
                                response |= Response::changed();
                            }
                }
                c if c == SORT => {
                    if self.screen == Screen::Workbench
                        && let Some(Tab::Table(table)) = self.workbench.active_mut()
                            && !table.is_structure() {
                                let col_idx = table
                                    .result
                                    .state
                                    .cursor()
                                    .map(|(_, col)| usize::from(col.raw()).saturating_sub(1))
                                    .unwrap_or(0);
                                let col_key = ColumnKey::num((col_idx as u16).saturating_add(1));
                                let next_sort = match table.result.state.sort() {
                                    Some((c, SortDir::Asc)) if c == col_key => {
                                        Some((col_key, SortDir::Desc))
                                    }
                                    Some((c, SortDir::Desc)) if c == col_key => None,
                                    _ => Some((col_key, SortDir::Asc)),
                                };
                                self.status = table.reload_sorted(&self.catalog, col_idx, next_sort);
                                response |= Response::changed();
                            }
                }
                c if c == TAB_LIST => {
                    if self.screen == Screen::Workbench && !self.is_editing() {
                        self.open_tab_list(cx);
                        response |= Response::changed();
                    }
                }
                c if c == SAFE_MODE => {
                    if self.screen == Screen::Workbench && !self.is_editing() {
                        self.open_safe_mode_picker(cx);
                        response |= Response::changed();
                    }
                }
                c if c == HELP && self.screen == Screen::Workbench && !self.is_editing() => {
                    self.open_help(cx);
                    response |= Response::changed();
                }
                _ => {}
            }
        }
        if self.form_open {
            for intent in cx.intents(connections::field::NAME) {
                if let Intent::Key(key) = intent {
                    match key.code {
                        KeyCode::Enter | KeyCode::F(2) => {
                            self.form_editing = true;
                            response |= Response::changed();
                        }
                        KeyCode::Esc => {
                            if self.form_editing {
                                self.form_editing = false;
                                response |= Response::changed();
                            } else {
                                self.close_connection_form();
                                response |= Response::changed();
                            }
                        }
                        KeyCode::Char('e') if !self.form_editing => {
                            self.begin_edit_connection_form(self.connections_screen.selected);
                            cx.focus(connections::field::NAME);
                            response |= Response::changed();
                        }
                        _ => {}
                    }
                }
            }
            for intent in cx.intents(connections::field::TABS) {
                if let Intent::Key(key) = intent {
                    match key.code {
                        KeyCode::Right => {
                            self.form_tab = 1;
                            response |= Response::changed();
                        }
                        KeyCode::Left => {
                            self.form_tab = 0;
                            response |= Response::changed();
                        }
                        _ => {}
                    }
                }
            }
        }
        let form_was_open = self.form_open;
        if self.draft.is_some() && (!form_was_open || self.form_editing) {
            let fields = &self.form_fields;
            let actions = &self.form_actions;
            if let Some(draft) = self.draft.as_mut() {
                let form = Self::connection_form(fields, actions);
                let form_response = form.update(cx, &mut self.form_state, draft);
                if form_was_open && let Some(action) = form_response.action_ref() {
                    match action {
                        FormAction::Action(ActionKey::CANCEL) => {
                            self.close_connection_form();
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
                                    self.request_connect(
                                        cx,
                                        self.connections.len().saturating_sub(1),
                                    );
                                    self.close_connection_form();
                                }
                            }
                        }
                        _ => {}
                    }
                }
                response |= form_response.erase();
            }
        }
        if self.form_open
            && self.form_tab == 0
            && let Some(draft) = self.draft.as_mut()
        {
            let ask = Checkbox::new(connections::field::ASK_PASSWORD, "")
                .update(cx, &mut draft.ask_password);
            if ask.action_ref().is_some() {
                response |= Response::changed();
            }
            response |= ask.erase();
        }
        if form_was_open {
            return response;
        }
        if self.screen == Screen::Connections {
            for intent in cx.intents(CONNECTION_FILTER) {
                match intent {
                    Intent::Key(key) => match key.code {
                        KeyCode::Down => {
                            self.connections_screen.filter_active = false;
                            cx.focus(CONNECTIONS);
                            response |= Response::changed();
                        }
                        KeyCode::Esc => {
                            self.connections_screen.filter.clear();
                            self.connections_screen.filter_active = false;
                            self.rebuild_connection_nodes();
                            cx.focus(CONNECTIONS);
                            response |= Response::changed();
                        }
                        KeyCode::Backspace => {
                            self.connections_screen.filter.pop();
                            self.rebuild_connection_nodes();
                            response |= Response::changed();
                        }
                        _ => {
                            if let Some(c) = key.bare_char() {
                                self.connections_screen.filter.push(c);
                                self.rebuild_connection_nodes();
                                response |= Response::changed();
                            }
                        }
                    },
                    Intent::Pointer {
                        phase: Phase::Click | Phase::Press,
                        ..
                    } => {
                        if !self.connections_screen.filter_active {
                            self.connections_screen.filter_active = true;
                            cx.focus(CONNECTION_FILTER);
                            response |= Response::changed();
                        }
                    }
                    Intent::FocusIn { .. } => {
                        if !self.connections_screen.filter_active {
                            self.connections_screen.filter_active = true;
                            response |= Response::changed();
                        }
                    }
                    Intent::FocusOut { .. } if self.connections_screen.filter_active => {
                        self.connections_screen.filter_active = false;
                        response |= Response::changed();
                    }
                    _ => {}
                }
            }
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
                self.request_connect(cx, self.connections_screen.selected);
                return response | Response::changed();
            }
            let tree_response = connection_tree().update(
                cx,
                &mut self.connection_tree_state,
                &self.connection_nodes,
            );
            if !self.connections_screen.filter_active {
                self.sync_connection_selection();
                if tree_response.action_ref().is_some() {
                    self.connection_visual_tree_state = self.connection_tree_state.clone();
                }
            }
            if let Some(action) = tree_response.action_ref()
                && let TreeAction::Activated(key) | TreeAction::Chose(key) = action
                && let Some(ConnectionNode::Connection { index, .. }) = self
                    .connection_nodes
                    .iter()
                    .find(|node| connection_node_key(node) == *key)
            {
                self.request_connect(cx, *index);
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
            && let Some(node) = self
                .explorer_nodes
                .iter()
                .find(|node| explorer_node_key(node) == *key)
            {
                match node {
                    ExplorerNode::Object { item, .. } => {
                        let item = item.clone();
                        if self.open_table(&item)
                            && let Some(tab_key) = self.workbench.active_key() {
                                let ctrl = tab_key.control("data");
                                self.workbench_focus = ctrl;
                                cx.focus(ctrl);
                            }
                    }
                    _ => {
                        self.explorer_tree_state.toggle(*key);
                    }
                }
            }
        response |= tree_response.erase();

        let tabs_response = tab_strip().update(cx, &mut self.tabs_state, self.workbench.tabs());
        if let Some(action) = tabs_response.action_ref() {
            match *action {
                TabsAction::Activated(key) => {
                    if let Some(index) = self
                        .workbench
                        .tabs()
                        .iter()
                        .position(|tab| tab_key(tab) == key)
                    {
                        if let Some(tab) = self.workbench.tabs().get(index) {
                            let key = tab.key();
                            let _ = self.workbench.activate(key);
                        }
                        self.sync_active_tab();
                        if let Some(tab_key) = self.workbench.active_key() {
                            if let Some(Tab::Table(_)) = self.workbench.active() {
                                let ctrl = tab_key.control("data");
                                self.workbench_focus = ctrl;
                                cx.focus(ctrl);
                            } else if let Some(focus) = self.query_id().or_else(|| self.result_id()) {
                                self.workbench_focus = focus;
                                cx.focus(focus);
                            }
                        }
                    }
                }
                TabsAction::Close(key) => {
                    if let Some(tab) = self.workbench.tabs().iter().find(|tab| tab_key(tab) == key)
                    {
                        self.request_close_tab(cx, tab.key());
                    }
                }
                TabsAction::New => {
                    self.new_query("");
                    if let Some(tab_key) = self.workbench.active_key() {
                        let ctrl = tab_key.control("query");
                        self.workbench_focus = ctrl;
                        cx.focus(ctrl);
                    }
                }
            }
        }
        response |= tabs_response.erase();

        response
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        self.screen_size.set((full.width, full.height));
        if full.width < MIN_WIDTH || full.height < MIN_HEIGHT {
            draw_too_small(ui, full);
            return;
        }
        ui.fill(full, ui.surface_style());
        let rows = shell_parts(full);
        draw_header(ui, rows[0], self);
        if self.form_open {
            self.draw_connection_form(ui, rows[1]);
        } else if self.screen == Screen::Connections {
            self.draw_connections(ui, rows[1]);
        } else {
            let workbench_rows = fixed_flex_pair(rows[1], 2);
            tab_strip().draw(
                ui,
                workbench_rows[0],
                &self.tabs_state,
                self.workbench.tabs(),
            );
            let body = workbench_rows[1];
            let narrow = body.width < 100;
            let in_dialog = self.safety_dialog.is_some()
                || self.destructive_intent.is_some()
                || self.help_open;
            let explorer_focused = !in_dialog
                && (self.workbench_focus == EXPLORER || (narrow && self.workbench.active().is_none()));
            let show_explorer = self.workbench.explorer_visible
                && !self.workbench.maximized
                && (!narrow || explorer_focused);
            let explorer_w = (body.width / 4).clamp(28, 40);
            let (ex, main) = if show_explorer && narrow {
                (body, termrock::Rect::ZERO)
            } else if show_explorer {
                (
                    termrock::Rect::new(body.x, body.y, explorer_w, body.height),
                    termrock::Rect::new(
                        body.x + explorer_w + 1,
                        body.y,
                        body.width.saturating_sub(explorer_w + 1),
                        body.height,
                    ),
                )
            } else {
                (termrock::Rect::ZERO, body)
            };

            if !ex.is_empty() {
                self.draw_explorer(ui, ex);
            } else if self.workbench.explorer_visible && !self.workbench.maximized {
                ui.register_focus_only(EXPLORER, Focusability::Focusable);
            }

            if main.is_empty() {
                if let Some(tab_key) = self.workbench.active_key() {
                    let pf = match self.workbench.active() {
                        Some(Tab::Table(t)) if t.is_structure() => {
                            Some(tab_key.control("structure"))
                        }
                        Some(Tab::Table(_)) => Some(tab_key.control("data")),
                        Some(Tab::Query(_)) => Some(tab_key.control("query")),
                        Some(Tab::History(_)) => Some(tab_key.control("history")),
                        None => None,
                    };
                    if let Some(pf) = pf {
                        ui.register_focus_only(pf, Focusability::Focusable);
                    }
                }
            } else {
                self.draw_content(ui, main);
            }
        }
        draw_footer(ui, rows[2], self);
        let (cols, screen_rows) = (full.width, full.height);
        ui.layer(quick_switcher::ID, |ui, area| {
            self.switcher
                .component(cols, screen_rows)
                .draw(ui, area, &self.switcher.state, &self.switcher.items);
        });
        ui.layer(tab_list::ID, |ui, area| {
            self.tab_list
                .component(cols, screen_rows)
                .draw(ui, area, &self.tab_list.state, &self.tab_list.items);
        });
        ui.layer(safe_mode_picker::ID, |ui, area| {
            self.safe_mode_picker
                .component(cols, screen_rows)
                .draw(ui, area, &self.safe_mode_picker.state, &self.safe_mode_picker.items);
        });
        ui.layer(HELP_DIALOG_ID, |ui, area| {
            help_dialog(cols, screen_rows).draw(ui, area, &self.help_dialog_state, |_, _| {});
        });
        if let Some(intent) = self.destructive_intent.as_ref() {
            ui.layer(QUIT_DIALOG, |ui, area| {
                intent.dialog().draw(ui, area, &self.quit_state, |_, _| {});
            });
        }
        if let Some(dialog) = self.safety_dialog.as_ref() {
            ui.layer(SAFETY_DIALOG, |ui, area| {
                dialog.draw(ui, area);
            });
        }
        if let Some(editor) = self.filter_editor.as_ref() {
            ui.layer(FILTER_EDITOR, |ui, area| {
                editor.draw(ui, area);
            });
        }
        ui.suppress_cursor();
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
        if self.screen == Screen::Workbench && self.workbench.maximized {
            self.workbench.maximized = false;
            if let Some(Tab::Query(q)) = self.workbench.active_mut() {
                q.maximized = QueryPaneMaximized::None;
            }
            return Response::changed();
        }
        Response::ignored()
    }
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
                "no connection with the requested name",
            ));
        };
        let _ = app.connect(index);
    }
    termrock::run(app, theme)
}

#[cfg(test)]
mod replacement_tests {
    use super::*;
    use termrock::GridEditor;
    use termrock_test_support::Harness;

    fn pending() -> TableProApp {
        let mut app = TableProApp::default();
        app.set_surface(Surface::PendingChangeBar);
        app.screen = Screen::Connections;
        app
    }

    #[test]
    fn reconnect_uses_configuration_snapshot_after_catalog_reorder() {
        let mut h = Harness::new(pending(), Theme::junie(), 120, 40);
        let target = h.app().connections_screen.selected;
        let Some(expected) = h.app().connections.get(target).cloned() else {
            unreachable!("target")
        };
        let _ = h.click_id(CONNECTION_DETAILS);
        assert!(h.find("Reconnect with unsaved work?").is_some());
        h.app_mut().connections.reverse();
        let _ = h.key(KeyCode::Tab);
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().workbench.connection, expected);
        assert_eq!(h.app().workbench.tabs().len(), 1);
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn new_tab_after_reconnect_prompt_invalidates_captured_scope() {
        let mut h = Harness::new(pending(), Theme::junie(), 120, 40);
        let _ = h.click_id(CONNECTION_DETAILS);
        let Some(key) = h.app_mut().workbench.new_query("new unsaved query") else {
            unreachable!("key")
        };
        let count = h.app().workbench.tabs().len();
        let _ = h.key(KeyCode::Tab);
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().workbench.tabs().len(), count);
        assert_eq!(h.app().workbench.active_key(), Some(key));
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn captured_connection_and_sql_debug_are_redacted_through_app() {
        let mut app = pending();
        app.destructive_intent = Some(DestructiveRequest {
            owner: app.workbench.owner_token(),
            intent: DestructiveIntent::Reconnect {
                target: Box::new(Connection {
                    host: "secret-reconnect-host".to_owned(),
                    ..app.connection.clone()
                }),
                source: Box::new(app.connection.clone()),
                scope: Vec::new(),
            },
        });
        assert!(!format!("{app:?}").contains("secret-reconnect-host"));
        let _ = app
            .workbench
            .new_query("SELECT id, currency FROM orders LIMIT 1");
        let catalog = app.catalog.clone();
        let Some(Tab::Query(tab)) = app.workbench.active_mut() else {
            unreachable!("query")
        };
        assert!(tab.execute(&catalog).is_ok());
        let Some(view) = tab.result.as_mut() else {
            unreachable!("result")
        };
        assert!(view.model.commit_cell(0, 1, "secret-result-value").is_ok());
        let Some(key) = app.workbench.active_key() else {
            unreachable!("key")
        };
        app.destructive_intent = Some(DestructiveRequest {
            owner: app.workbench.owner_token(),
            intent: DestructiveIntent::ReplaceResult {
                key,
                query: "secret-captured-sql".to_owned(),
                generation: 0,
                connection: Box::new(app.connection.clone()),
            },
        });
        let text = format!("{app:?}");
        assert!(!text.contains("secret-captured-sql"));
        assert!(!text.contains("secret-result-value"));
    }

    #[test]
    fn table_sort_keystroke_works() {
        let mut app = TableProApp::default();
        let idx = app.connections.iter().position(|c| c.name == "Production").unwrap();
        let _ = app.connect(idx);
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        // Navigate explorer to orders:
        for _ in 0..5 {
            let _ = h.key(KeyCode::Down);
        }
        let _ = h.key(KeyCode::Enter);
        eprintln!("Screen after enter:\n{}", h.text());
        // 12 right keys:
        for _ in 0..12 {
            let _ = h.key(KeyCode::Right);
        }
        eprintln!("Screen after 12 right:\n{}", h.text());
        let _ = h.key(KeyCode::Char('s'));
        eprintln!("Screen after s:\n{}", h.text());
        assert!(h.text().contains("sort created_at ▴"));
    }

    #[test]
    fn test_filtered_harness() {
        let mut app = TableProApp::default();
        let idx = app.connections.iter().position(|c| c.name == "Production").unwrap();
        let _ = app.connect(idx);
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        for _ in 0..5 {
            let _ = h.key(KeyCode::Down);
        }
        let _ = h.key(KeyCode::Enter);
        let _ = h.key(KeyCode::Home);
        for _ in 0..4 {
            let _ = h.key(KeyCode::Right);
        }
        let _ = h.key(KeyCode::Char('f'));
        let _ = h.key(KeyCode::BackTab);
        let _ = h.key(KeyCode::BackTab);
        let _ = h.key(KeyCode::Enter);
        let _ = h.key_mod(KeyCode::Char('l'), termrock::KeyModifiers::CONTROL);
        for c in "pending".chars() {
            let _ = h.key(KeyCode::Char(c));
        }
        let _ = h.key(KeyCode::Enter);
        let text = h.text();
        assert!(text.contains("status = 'pending'"));
        assert!(text.contains("filtered (1)"));
    }

    #[test]
    fn filter_value_edits_through_text_input_draft() {
        let mut app = TableProApp::default();
        let idx = app
            .connections
            .iter()
            .position(|c| c.name == "Production")
            .unwrap();
        let _ = app.connect(idx);
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        for _ in 0..5 {
            let _ = h.key(KeyCode::Down);
        }
        let _ = h.key(KeyCode::Enter);
        let _ = h.key(KeyCode::Home);
        for _ in 0..4 {
            let _ = h.key(KeyCode::Right);
        }
        let _ = h.key(KeyCode::Char('f'));
        let _ = h.key(KeyCode::BackTab);
        let _ = h.key(KeyCode::BackTab);
        let prefill = h
            .app()
            .filter_editor
            .as_ref()
            .map(|editor| editor.value.clone())
            .unwrap_or_default();
        // Begin editing: TextInput draft starts, committed value untouched.
        let _ = h.key(KeyCode::Enter);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert!(editor.value_state.is_editing());
        assert_eq!(editor.value, prefill);
        // Select-all + partial typing updates the draft and the live SQL
        // preview, not the committed value.
        let _ = h.key_mod(KeyCode::Char('l'), termrock::KeyModifiers::CONTROL);
        for c in "pend".chars() {
            let _ = h.key(KeyCode::Char(c));
        }
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert!(editor.value_state.is_editing());
        assert_eq!(editor.value_state.draft_text(), Some("pend"));
        assert_eq!(editor.value, prefill);
        assert!(h.text().contains("'pend'"));
        // Esc cancels the draft: editor stays open, committed value kept.
        let _ = h.key(KeyCode::Esc);
        let editor = h.app().filter_editor.as_ref().expect("editor stays open");
        assert!(!editor.value_state.is_editing());
        assert_eq!(editor.value_state.draft_text(), None);
        assert_eq!(editor.value, prefill);
        // Re-edit and commit with Enter: applies the filter.
        let _ = h.key(KeyCode::Enter);
        let _ = h.key_mod(KeyCode::Char('l'), termrock::KeyModifiers::CONTROL);
        for c in "pending".chars() {
            let _ = h.key(KeyCode::Char(c));
        }
        let _ = h.key(KeyCode::Enter);
        assert!(h.app().filter_editor.is_none());
        let text = h.text();
        assert!(text.contains("status = 'pending'"));
        assert!(text.contains("filtered (1)"));
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn filter_value2_edits_through_text_input_draft() {
        let mut app = TableProApp::default();
        let idx = app
            .connections
            .iter()
            .position(|c| c.name == "Production")
            .unwrap();
        let _ = app.connect(idx);
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        for _ in 0..5 {
            let _ = h.key(KeyCode::Down);
        }
        let _ = h.key(KeyCode::Enter);
        let _ = h.key(KeyCode::Home);
        for _ in 0..4 {
            let _ = h.key(KeyCode::Right);
        }
        // Prefilled open: focus starts on the Apply stop.
        let _ = h.key(KeyCode::Char('f'));
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert_eq!(editor.focus, FilterFocus::Apply);
        let prefill = editor.value.clone();
        assert!(!prefill.is_empty());
        // Choose Between through the owned op dropdown.
        let _ = h.key(KeyCode::Tab);
        let _ = h.key(KeyCode::Tab);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert_eq!(editor.focus, FilterFocus::Op);
        let _ = h.key(KeyCode::Enter);
        assert!(
            h.app()
                .filter_editor
                .as_ref()
                .expect("editor open")
                .op_select
                .is_open()
        );
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        let between_pos = editor
            .ops
            .iter()
            .position(|&op| op == FilterOp::Between)
            .expect("between offered");
        // Down moves the cursor WITHOUT committing (cursor is not value).
        for _ in 0..between_pos {
            let _ = h.key(KeyCode::Down);
        }
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert!(editor.op_select.is_open());
        assert_eq!(editor.op, FilterOp::Eq);
        assert_eq!(editor.op_select.cursor(), Some(op_key(&FilterOp::Between)));
        let _ = h.key(KeyCode::Enter);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert!(!editor.op_select.is_open());
        assert_eq!(editor.op, FilterOp::Between);
        assert!(h.text().contains("BETWEEN"));
        // The Tab ring reaches the second value field.
        let _ = h.key(KeyCode::Tab);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert_eq!(editor.focus, FilterFocus::Value);
        let _ = h.key(KeyCode::Tab);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert_eq!(editor.focus, FilterFocus::Value2);
        // Begin editing: TextInput draft starts, committed value2 untouched.
        let _ = h.key(KeyCode::Enter);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert!(editor.value2_state.is_editing());
        assert_eq!(editor.value2, "");
        for c in "999".chars() {
            let _ = h.key(KeyCode::Char(c));
        }
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert!(editor.value2_state.is_editing());
        assert_eq!(editor.value2_state.draft_text(), Some("999"));
        assert_eq!(editor.value2, "");
        // Esc cancels the draft: editor stays open, committed value2 kept.
        let _ = h.key(KeyCode::Esc);
        let editor = h.app().filter_editor.as_ref().expect("editor stays open");
        assert!(!editor.value2_state.is_editing());
        assert_eq!(editor.value2_state.draft_text(), None);
        assert_eq!(editor.value2, "");
        // Re-edit and commit with Enter: applies the Between filter.
        let _ = h.key(KeyCode::Enter);
        for c in "999".chars() {
            let _ = h.key(KeyCode::Char(c));
        }
        let _ = h.key(KeyCode::Enter);
        assert!(h.app().filter_editor.is_none());
        let filters = match h.app().workbench.active() {
            Some(Tab::Table(t)) => t.filters.clone(),
            _ => panic!("table tab active"),
        };
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].op, FilterOp::Between);
        assert_eq!(filters[0].value, prefill.trim());
        assert_eq!(filters[0].value2, "999");
        assert_eq!(h.app().status, "1 filter applied");
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn filter_column_and_op_choose_through_owned_select() {
        let mut app = TableProApp::default();
        let idx = app
            .connections
            .iter()
            .position(|c| c.name == "Production")
            .unwrap();
        let _ = app.connect(idx);
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        for _ in 0..5 {
            let _ = h.key(KeyCode::Down);
        }
        let _ = h.key(KeyCode::Enter);
        let _ = h.key(KeyCode::Home);
        for _ in 0..4 {
            let _ = h.key(KeyCode::Right);
        }
        let _ = h.key(KeyCode::Char('f'));
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        let first_col = editor.column_idx;
        assert_eq!(editor.focus, FilterFocus::Apply);
        assert_eq!(editor.op, FilterOp::Eq);
        assert!(!editor.value.is_empty());
        // Tab wraps Apply -> Column (ring order follows draw registration).
        let _ = h.key(KeyCode::Tab);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert_eq!(editor.focus, FilterFocus::Column);
        // Enter opens the owned popup; Down moves the cursor WITHOUT
        // committing (the cursor is not the value).
        let _ = h.key(KeyCode::Enter);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert!(editor.col_select.is_open());
        let _ = h.key(KeyCode::Down);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert!(editor.col_select.is_open());
        assert_eq!(editor.column_idx, first_col);
        let next_name = editor.columns[first_col + 1].0.clone();
        assert_eq!(
            editor.col_select.cursor(),
            Some(column_key(&editor.columns[first_col + 1]))
        );
        // Enter chooses the cursor column; the popup closes and the SQL
        // preview follows the committed choice.
        let _ = h.key(KeyCode::Enter);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert!(!editor.col_select.is_open());
        assert_eq!(editor.column_idx, first_col + 1);
        assert!(h.text().contains(&format!("WHERE {next_name} ")));
        // Same open/cursor/choose cycle on the operator dropdown.
        let _ = h.key(KeyCode::Tab);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert_eq!(editor.focus, FilterFocus::Op);
        let first_op_pos = editor.ops.iter().position(|&o| o == editor.op).unwrap();
        let _ = h.key(KeyCode::Enter);
        assert!(
            h.app()
                .filter_editor
                .as_ref()
                .expect("editor open")
                .op_select
                .is_open()
        );
        let _ = h.key(KeyCode::Down);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert_eq!(editor.op, FilterOp::Eq);
        let _ = h.key(KeyCode::Enter);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert!(!editor.op_select.is_open());
        assert_eq!(editor.op, editor.ops[first_op_pos + 1]);
        let op_label = editor.op.label().to_owned();
        // Esc with an open popup closes the popup only: the editor stays
        // open and the committed op is kept.
        let _ = h.key(KeyCode::Enter);
        assert!(
            h.app()
                .filter_editor
                .as_ref()
                .expect("editor open")
                .op_select
                .is_open()
        );
        let _ = h.key(KeyCode::Esc);
        let editor = h.app().filter_editor.as_ref().expect("editor stays open");
        assert!(!editor.op_select.is_open());
        assert_eq!(editor.op.label(), op_label);
        // Pointer: clicking the column field toggles the popup, and clicking
        // a popup row chooses it by stable key.
        let _ = h.click_id(FILTER_COL);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert!(editor.col_select.is_open());
        let id_key = column_key(&editor.columns[0]);
        let _ = h.click_part(
            FILTER_COL,
            termrock::PartRef::item(termrock::Part::ROW, id_key),
        );
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert!(!editor.col_select.is_open());
        assert_eq!(editor.column_idx, 0);
        // Apply through the Apply stop; the committed column/op reach the chip.
        for _ in 0..4 {
            let _ = h.key(KeyCode::Tab);
        }
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert_eq!(editor.focus, FilterFocus::Apply);
        let _ = h.key(KeyCode::Enter);
        assert!(h.app().filter_editor.is_none());
        let text = h.text();
        assert!(text.contains("1 filter applied"), "{text}");
        assert!(text.contains(&format!("id {op_label} ")), "{text}");
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn filter_editor_cancel_apply_buttons_activate() {
        fn active_filters(h: &Harness<TableProApp>) -> Vec<Filter> {
            match h.app().workbench.active() {
                Some(Tab::Table(t)) => t.filters.clone(),
                _ => panic!("table tab active"),
            }
        }
        let mut app = TableProApp::default();
        let idx = app
            .connections
            .iter()
            .position(|c| c.name == "Production")
            .unwrap();
        let _ = app.connect(idx);
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        for _ in 0..5 {
            let _ = h.key(KeyCode::Down);
        }
        let _ = h.key(KeyCode::Enter);
        let _ = h.key(KeyCode::Home);
        for _ in 0..4 {
            let _ = h.key(KeyCode::Right);
        }
        // Prefilled open: focus starts on the Apply stop.
        let _ = h.key(KeyCode::Char('f'));
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert_eq!(editor.focus, FilterFocus::Apply);
        assert!(!editor.value.is_empty());
        // Click Cancel: the editor closes and no filter is committed.
        let _ = h.click_id(FILTER_CANCEL);
        assert!(h.app().filter_editor.is_none());
        assert!(active_filters(&h).is_empty());
        // Reopen; Enter on the Apply stop applies the prefilled filter.
        let _ = h.key(KeyCode::Char('f'));
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert_eq!(editor.focus, FilterFocus::Apply);
        let _ = h.key(KeyCode::Enter);
        assert!(h.app().filter_editor.is_none());
        let filters = active_filters(&h);
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].column, "status");
        assert_eq!(h.app().status, "1 filter applied");
        // Reopen; clicking Apply commits a second filter the same way.
        let _ = h.key(KeyCode::Char('f'));
        assert!(h.app().filter_editor.is_some());
        let _ = h.click_id(FILTER_APPLY);
        assert!(h.app().filter_editor.is_none());
        assert_eq!(active_filters(&h).len(), 2);
        assert_eq!(h.app().status, "2 filters applied");
        // Reopen; BackTab reaches Cancel and Enter cancels typed state.
        let _ = h.key(KeyCode::Char('f'));
        let _ = h.key(KeyCode::BackTab);
        let editor = h.app().filter_editor.as_ref().expect("editor open");
        assert_eq!(editor.focus, FilterFocus::Cancel);
        let _ = h.key(KeyCode::Enter);
        assert!(h.app().filter_editor.is_none());
        assert_eq!(active_filters(&h).len(), 2);
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn help_overlay_opens_on_question_mark() {
        let mut app = TableProApp::default();
        let idx = app
            .connections
            .iter()
            .position(|c| c.name == "Production")
            .unwrap();
        let _ = app.connect(idx);
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        assert!(h.find("Query 1").is_some());
        eprintln!("INITIAL TEXT:\n{}", h.text());
        let _ = h.key(KeyCode::Char('?'));
        eprintln!("AFTER QUESTION MARK:\n{}", h.text());
        assert!(h.find("Keyboard").is_some());
    }

    #[test]
    fn workbench_maximize_and_unmaximize_query_tab() {
        let mut app = TableProApp::default();
        let idx = app
            .connections
            .iter()
            .position(|c| c.name == "Production")
            .unwrap();
        let _ = app.connect(idx);
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        assert!(!h.app().workbench.maximized);
        let ph = "Type SQL. Ctrl+R runs the statement under the cursor.";
        assert!(h.find(ph).is_some());
        assert!(h.find("No results yet").is_some());

        // Focus is on EXPLORER, so 'z' maximizes Results pane and hides editor
        let _ = h.key(KeyCode::Char('z'));
        assert!(h.app().workbench.maximized);
        if let Some(Tab::Query(q)) = h.app().workbench.active() {
            assert_eq!(q.maximized, QueryPaneMaximized::Results);
        } else {
            panic!("expected active query tab");
        }
        assert!(h.find(ph).is_none());
        assert!(h.find("No results yet").is_some());

        // Esc un-maximizes workbench and restores normal split
        let _ = h.key(KeyCode::Esc);
        assert!(!h.app().workbench.maximized);
        if let Some(Tab::Query(q)) = h.app().workbench.active() {
            assert_eq!(q.maximized, QueryPaneMaximized::None);
        }
        assert!(h.find(ph).is_some());

        // Focus editor via Tab, then 'z' maximizes Editor pane and hides results
        let query_ctrl = h.app().workbench.active_key().unwrap().control("query");
        assert!(h.tab_to(query_ctrl));
        assert!(!h.app().is_editing());
        let _ = h.key(KeyCode::Char('z'));
        assert!(h.app().workbench.maximized);
        if let Some(Tab::Query(q)) = h.app().workbench.active() {
            assert_eq!(q.maximized, QueryPaneMaximized::Editor);
        }
        assert!(h.find(ph).is_some());
        assert!(h.find("No results yet").is_none());

        // '0' focuses explorer and resets maximization
        let _ = h.key(KeyCode::Char('0'));
        assert!(!h.app().workbench.maximized);
        if let Some(Tab::Query(q)) = h.app().workbench.active() {
            assert_eq!(q.maximized, QueryPaneMaximized::None);
        }
        assert!(h.find(ph).is_some());
        assert!(h.find("No results yet").is_some());
    }
}

#[cfg(test)]
mod action_namespace_tests {
    use super::*;

    #[test]
    fn application_actions_are_isolated_and_unique() {
        let keys = [
            RUN,
            UNDO,
            INSERT_ROW,
            DUPLICATE_ROW,
            DELETE_ROW,
            DISCARD_ROWS,
            QUIT,
            CANCEL_OR_QUIT,
            SORT,
            OPEN,
            NEW_QUERY,
            CLOSE_TAB,
            HISTORY,
            STRUCTURE,
            FORM,
            HELP,
            TAB_LIST,
            FILTER,
            PREVIEW,
            SAVE,
            EXPLAIN,
            CLEAR_QUERY,
            SAFE_MODE,
            COMPLETE,
            PALETTE,
            DELETE_CONNECTION,
            EDIT_CONNECTION,
            FORM_TAB_PREV,
            FORM_TAB_NEXT,
            connections::TEST,
            connections::SAVE_CONNECT,
        ];
        for (index, key) in keys.iter().enumerate() {
            assert!(
                (0x4000..0x8000).contains(&key.raw()),
                "application namespace: {key:?}"
            );
            assert!(
                !keys.iter().take(index).any(|previous| previous == key),
                "duplicate application action: {key:?}"
            );
        }
    }
}

