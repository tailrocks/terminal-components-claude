use std::cell::RefCell;

use termrock::author::{PaintStyle, StyleDefaults};
use termrock::{
    Action, ActionKey, Align, App, CellAction, CellUi, Chord, ColumnKey, Cx, Dialog, DialogAction,
    DialogState, FgStep, Field, Focusability, Form, FormAction, FormState, FrameRead, GlyphRole,
    Grid, GridAction, GridCell, GridColumnFit, GridEditor, GridGutter, GridModel,
    GridSortIndicator, Id, Intent, ItemKey, KeyCode, KeyMap, KeyModifiers, KeyPhase, LayerId,
    Modifier, Panel, PanelKind, Part, Phase, PickerAction, Response, Role, RowDecor, RowTotal,
    RowUi, Size, SortDir, Span, SplitAxis, SplitPane, SplitPaneState, StateFlags, StylePatch, Tabs,
    TabsAction, TabsState, TextInput, TextInputState, Theme, Tree, TreeAction,
    TreeBranchActivation, TreeNode, TreeState, Ui, UpdateCause, truncate, truncate_middle, width,
    wrap,
};

use crate::quick_switcher::{self, QuickSwitcher};
use crate::tabs::{ExplorerItem, GridView, Tab, TabKey, TabRecord};
use crate::workbench::Workbench;
use tablepro_demo::db::{
    self, Catalog, ColType, ConnectOutcome, Connection, Environment, ObjectKind, SafeMode,
};
use tablepro_domain::connections::{self, ConnectionDraft, ConnectionsScreen};
use tablepro_domain::domain::ResultGrid;
use tablepro_domain::model::SwitchTarget;

pub const MIN_WIDTH: u16 = 72;

pub const MIN_HEIGHT: u16 = 20;
const CONNECTIONS: Id = Id::root("tablepro.connections.list");
const CONNECTIONS_PANEL: Id = Id::root("tablepro.connections.panel");
const EXPLORER: Id = Id::root("tablepro.workbench.explorer.tree");
const EXPLORER_PANEL: Id = Id::root("tablepro.workbench.explorer.panel");
const TAB_STRIP: Id = Id::root("tablepro.workbench.tab-strip");
const WORKBENCH_SPLIT: Id = Id::root("tablepro.workbench.split");
const RUN: ActionKey = ActionKey::application("tablepro.run");
const UNDO: ActionKey = ActionKey::application("tablepro.undo");
const INSERT_ROW: ActionKey = ActionKey::application("tablepro.insert-row");
const DELETE_ROW: ActionKey = ActionKey::application("tablepro.delete-row");
const DISCARD_ROWS: ActionKey = ActionKey::application("tablepro.discard-rows");
const QUIT: ActionKey = ActionKey::application("tablepro.quit");
const CANCEL_OR_QUIT: ActionKey = ActionKey::application("tablepro.cancel-or-quit");
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
const HISTORY: ActionKey = ActionKey::application("tablepro.history");
const STRUCTURE: ActionKey = ActionKey::application("tablepro.structure");
const FORM: ActionKey = ActionKey::application("tablepro.form");
const HELP: ActionKey = ActionKey::application("tablepro.help");
const TAB_LIST: ActionKey = ActionKey::application("tablepro.tab-list");
const FILTER: ActionKey = ActionKey::application("tablepro.filter");
const SORT: ActionKey = ActionKey::application("tablepro.sort");
const PREVIEW: ActionKey = ActionKey::application("tablepro.preview");
const SAVE: ActionKey = ActionKey::application("tablepro.save");
const EXPLAIN: ActionKey = ActionKey::application("tablepro.explain");
const CLEAR_QUERY: ActionKey = ActionKey::application("tablepro.clear-query");
const COMPLETE: ActionKey = ActionKey::application("tablepro.complete");
const PALETTE: ActionKey = ActionKey::application("tablepro.palette");

const CONNECTION_DETAILS: Id = Id::root("tablepro.connections.details");
const CONTENT_FRAME: Id = Id::root("tablepro.workbench.content.frame");

const CONNECTION_DETAILS_TITLE_PATCH: [(Part, StylePatch); 1] = [(
    Part::TITLE,
    StylePatch::new()
        .set_fg(Role::Fg(FgStep::Secondary))
        .remove(Modifier::BOLD),
)];
// Frozen junie ink. Current steps are Muted #808080, Faint/BorderStrong #4d4d4d,
// Info #8787ff, and Danger #e44545. None of those roles match the frame.
const TAG_CHROME_FG: Role = Role::Custom(termrock::Color::from_u32(0x008a_8a8a));
const TAG_PATH_FG: Role = Role::Custom(termrock::Color::from_u32(0x00b3_b3b3));
const TAG_OK_FG: Role = Role::Custom(termrock::Color::from_u32(0x003f_b950));
const TAG_RULE_FG: Role = Role::Custom(termrock::Color::from_u32(0x003a_3a3a));
const LEGACY_FAINT: Role = Role::Custom(termrock::Color::from_u32(0x0073_7373));
const LEGACY_INFO: Role = Role::Custom(termrock::Color::from_u32(0x0079_b8ff));
const LEGACY_DANGER_LABEL: Role = Role::Custom(termrock::Color::from_u32(0x00ff_6b6b));
const FRAMED_PANEL_PATCH: [(Part, StylePatch); 2] = [
    (Part::DETAIL, StylePatch::new().set_fg(LEGACY_FAINT)),
    (Part::BORDER, StylePatch::new().set_fg(Role::BorderStrong)),
];
const EXPLORER_PANEL_PATCH: [(Part, StylePatch); 4] = [
    (Part::DETAIL, StylePatch::new().set_fg(LEGACY_FAINT)),
    (Part::BORDER, StylePatch::new().set_fg(TAG_RULE_FG)),
    (
        Part::TITLE,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .remove(Modifier::BOLD),
    ),
    (Part::META, StylePatch::new().set_fg(LEGACY_FAINT)),
];
const EXPLORER_FOCUSED_PATCH: [(Part, StylePatch); 3] = [
    (Part::DETAIL, StylePatch::new().set_fg(LEGACY_FAINT)),
    (Part::BORDER, StylePatch::new().set_fg(Role::BorderStrong)),
    (Part::META, StylePatch::new().set_fg(LEGACY_FAINT)),
];
const CONTENT_IDLE_PATCH: [(Part, StylePatch); 3] = [
    (Part::DETAIL, StylePatch::new().set_fg(LEGACY_FAINT)),
    (Part::BORDER, StylePatch::new().set_fg(TAG_RULE_FG)),
    (
        Part::TITLE,
        StylePatch::new().set_fg(TAG_PATH_FG).remove(Modifier::BOLD),
    ),
];
const TAG_FIELD_BG: Role = Role::Custom(termrock::Color::from_u32(0x001e_1e22));
const TAG_BUTTON_BG: Role = Role::Custom(termrock::Color::from_u32(0x0048_e054));
const TAG_BUTTON_FG: Role = Role::Custom(termrock::Color::from_u32(0x0019_191c));
const TAB_STRIP_PATCH: [(Part, StylePatch); 2] = [
    (Part::NEW, StylePatch::new().set_fg(TAG_PATH_FG)),
    (
        Part::CLOSE,
        StylePatch::new().set_fg(TAG_PATH_FG).remove(Modifier::BOLD),
    ),
];
const GRID_PART_PATCH: [(Part, StylePatch); 2] = [
    (Part::OVERFLOW, StylePatch::new().set_fg(TAG_PATH_FG)),
    (Part::ACTIONS, StylePatch::new().set_fg(TAG_CHROME_FG)),
];
const LEGACY_SELECT_BG: Role = Role::Custom(termrock::Color::from_u32(0x0017_3f1c));
const LEGACY_SCROLL_TRACK: Role = Role::Custom(termrock::Color::from_u32(0x0047_4747));
const LEGACY_SCROLL_THUMB: Role = Role::Custom(termrock::Color::from_u32(0x0080_8080));
const LEGACY_CELL_BG: Role = Role::Custom(termrock::Color::from_u32(0x002f_5aa8));
const FOLLOW_REF: CellAction = CellAction::new(ActionKey::application("tablepro.follow-ref"));
const FOLLOW_REFS: [CellAction; 1] = [FOLLOW_REF];
const CONNECTIONS_PANEL_PATCH: [(Part, StylePatch); 1] =
    [(Part::DETAIL, StylePatch::new().set_fg(LEGACY_FAINT))];
const CONNECTION_TREE_PATCH: [(Part, StylePatch); 1] = [(
    Part::GUTTER,
    StylePatch::new().set_fg(Role::Surface(termrock::Surface::Canvas)),
)];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Connections,

    Workbench,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    Connections,

    ConnectionsFailed,

    WorkbenchDefault,

    ExplorerFocused,

    TableGrid,

    GridCellEditing,

    PendingChangeBar,

    StructureView,

    QueryEditing,

    CompletionPopup,

    ResultsGrid,

    ErrorResult,

    ExplainPlan,

    HistoryTab,

    QuickSwitcher,

    TabListPicker,

    SafeModePicker,

    FilterEditor,

    SafetyDialogTypedAck,

    HelpDialog,

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
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('-')), DELETE_ROW)
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('U')),
            DISCARD_ROWS,
        )
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
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('s')), SORT)
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

pub struct TableProApp {
    catalog: Catalog,
    connections: Vec<Connection>,
    connection: Connection,
    keymap: KeyMap,
    safe_mode: SafeMode,
    empty_result: ResultGrid,
    status: String,
    quit: bool,
    destructive_intent: Option<DestructiveRequest>,
    destructive_notice: Option<&'static str>,
    quit_state: DialogState,
    switcher: QuickSwitcher,

    pub screen: Screen,

    pub surface: Surface,

    pub connections_screen: ConnectionsScreen,

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
    motion: Motion,
    tick: u64,
    explorer_pin: u8,
    grid_sort: Option<(TabKey, u16, SortDir)>,
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
            .field("motion", &self.motion)
            .field("tick", &self.tick)
            .field("explorer_pin", &self.explorer_pin)
            .field("grid_sort", &self.grid_sort)
            .finish()
    }
}

impl Default for TableProApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TableProApp {
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
        let explorer_tree_state = initial_explorer_tree_state(&explorer_nodes, "public");
        let mut app = Self {
            safe_mode: connection.safe_mode,
            catalog: catalog.clone(),
            connections: connections.clone(),
            connection: connection.clone(),
            keymap: keymap(),
            empty_result: ResultGrid::empty(),
            status: "Ready · Ctrl+R runs · Ctrl+Q quits".to_owned(),
            quit: false,
            destructive_intent: None,
            destructive_notice: None,
            quit_state: DialogState::default(),
            switcher: QuickSwitcher::default(),
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
            motion: Motion::Full,
            tick: 0,
            explorer_pin: 0,
            grid_sort: None,
        };
        app.workbench.new_query(
            "SELECT * FROM orders WHERE status = 'pending' ORDER BY total_amount DESC LIMIT 20",
        );
        let _ = app.execute_query();
        app
    }

    pub const fn safe_mode(&self) -> SafeMode {
        self.safe_mode
    }

    pub fn query(&self) -> &str {
        match self.workbench.active() {
            Some(Tab::Query(tab)) => &tab.query,
            _ => "",
        }
    }

    pub fn result(&self) -> &ResultGrid {
        self.workbench
            .active_grid()
            .map_or(&self.empty_result, |(_, grid)| &grid.model)
    }

    pub fn query_id(&self) -> Option<Id> {
        match self.workbench.active() {
            Some(Tab::Query(_)) => self.workbench.active_key().map(|key| key.control("query")),
            _ => None,
        }
    }

    pub fn result_id(&self) -> Option<Id> {
        self.workbench.active_grid().map(|(id, _)| id)
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub const fn screen(&self) -> Screen {
        self.screen
    }

    pub const fn surface(&self) -> Surface {
        self.surface
    }

    pub(crate) const fn set_motion(&mut self, motion: Motion, frame: u64) {
        if motion.is_paused() {
            self.tick = frame;
        }
        self.motion = motion;
    }

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

    fn connections_panel<'a>(title: &'a str, meta: Option<&'a str>) -> Panel<'a> {
        let panel = Panel::new(CONNECTIONS_PANEL)
            .kind(PanelKind::Framed)
            .title(title)
            .focused(true)
            .patch_part(&CONNECTIONS_PANEL_PATCH)
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
        let panel = Panel::new(EXPLORER_PANEL)
            .kind(PanelKind::Framed)
            .title(" Explorer")
            .meta(schema)
            .focused(focused)
            .slot(Part::GUTTER, &preserve_frame_gutter);
        if focused {
            panel.patch_part(&EXPLORER_FOCUSED_PATCH)
        } else {
            panel.patch_part(&EXPLORER_PANEL_PATCH)
        }
    }

    fn content_panel<'a>(title: &'a str, meta: Option<&'a str>, focused: bool) -> Panel<'a> {
        let panel = Panel::new(CONTENT_FRAME)
            .kind(PanelKind::Framed)
            .title(title)
            .focused(focused)
            .patch_part(if focused {
                &FRAMED_PANEL_PATCH[..]
            } else {
                &CONTENT_IDLE_PATCH[..]
            })
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
        if let Some(Tab::Query(tab)) = self.workbench.active_mut() {
            query.clone_into(&mut tab.query);
            tab.editor_state = TextInputState::default();
        }
    }

    pub const fn workbench(&self) -> &Workbench {
        &self.workbench
    }

    pub const fn connection_form_open(&self) -> bool {
        self.form_open
    }

    pub const fn connection_draft(&self) -> Option<&ConnectionDraft> {
        if self.form_open {
            self.draft.as_ref()
        } else {
            None
        }
    }

    pub fn form_open_for_test(&mut self, open: bool) {
        self.form_open = open;
        if !open {
            self.draft = None;
        }
    }

    pub fn begin_connection_form(&mut self) {
        self.draft = Some(ConnectionDraft::from_connection(&self.connection));
        self.form_state = FormState::default();
        self.form_open = true;
        self.surface = Surface::Connections;
    }
    fn close_connection_form(&mut self) {
        self.form_open = false;
        self.form_state.zeroize();

        self.draft = Some(ConnectionDraft::from_connection(&self.connection));
    }

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
        self.split_state = SplitPaneState::default();
        self.sync_tabs_state();
        self.screen = Screen::Workbench;
        self.surface = Surface::WorkbenchDefault;
        // The query input registers before the explorer tree. Two updates let
        // the tree enter the focus ring before the idle frame is captured.
        self.explorer_pin = 2;
        self.split_state = SplitPaneState::new(25);
        self.status = format!("Connected to {}", connection.name);
        true
    }

    fn sync_active_table(&mut self) {
        self.sync_active_tab();
    }

    fn pin_tab_window(&mut self) {
        let Some(active) = self.tabs_state.active() else {
            return;
        };
        let Some(index) = self
            .workbench
            .tabs()
            .iter()
            .position(|tab| tab_key(tab) == active)
        else {
            return;
        };
        self.tabs_state = TabsState::default();
        self.tabs_state.set_active(index, active);
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

    pub fn set_safe_mode(&mut self, mode: SafeMode) {
        self.safe_mode = mode;
        self.connection.safe_mode = mode;
        self.workbench.connection.safe_mode = mode;
        self.surface = Surface::SafeModePicker;
    }

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
        let statement = match tablepro_sql::sql::parse(query.trim()) {
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
            tablepro_sql::sql::Statement::Select(select) => {
                self.catalog.find(select.schema.as_deref(), &select.table)
            }
            _ => None,
        };
        match tablepro_sql::sql::gate(self.safe_mode, &statement) {
            tablepro_sql::sql::Decision::Deny => {
                let risk = tablepro_sql::sql::assess(&statement, table);
                let out = QueryOutcome::Denied {
                    summary: format!("{} is denied in Read-Only mode", risk.action),
                };
                self.status = outcome_message(&out);
                out
            }
            tablepro_sql::sql::Decision::Confirm { deliberate } => {
                let risk = tablepro_sql::sql::assess(&statement, table);
                let out = QueryOutcome::ConfirmationRequired {
                    deliberate,
                    summary: format!("{} · {}", risk.action, risk.scope),
                };
                self.status = outcome_message(&out);
                out
            }
            tablepro_sql::sql::Decision::Run => {
                if let tablepro_sql::sql::Statement::Select(select) = statement {
                    match tablepro_sql::sql::run_select(&self.catalog, &select) {
                        Ok(result) => {
                            let out = QueryOutcome::Executed {
                                rows: result.rows.len(),
                                editable: result.editable,
                            };
                            if let Some(Tab::Query(tab)) = self.workbench.tab_mut(key) {
                                tab.result = Some(GridView::from_result(&result));
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
    ) -> ([termrock::Column<'_>; termrock::GRID_MAX_COLUMNS], usize) {
        let count = columns.len().min(termrock::GRID_MAX_COLUMNS);
        let mut specs = [termrock::Column::new(ColumnKey::num(0), ""); termrock::GRID_MAX_COLUMNS];
        for (index, (name, ty)) in columns.iter().take(count).enumerate() {
            let mut col = termrock::Column::new(
                ColumnKey::num((index as u16).saturating_add(1)),
                name.as_str(),
            );
            col.sortable = true;
            col.editable = editable;
            col.sticky = index == 0;
            if matches!(ty, ColType::Int | ColType::Numeric) {
                col.align = Align::Right;
            }
            if name == "id" {
                col.min_width = 38;
            }
            if let Some(slot) = specs.get_mut(index) {
                *slot = col;
            }
        }
        (specs, count)
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
        } else {
            self.commit_query_edit();
            let _ = self.execute_query();
        }
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
            cx.state(id).contains(StateFlags::FOCUSED)
                && !grid.state.is_editing()
                && grid.model.is_editable()
        })
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
        let (columns, count) = Self::column_specs(&grid.columns, grid.model.is_editable());
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

    fn open_switcher(&mut self, cx: &mut Cx<'_>) {
        self.switcher.open(&self.workbench);
        cx.open_layer(
            quick_switcher::ID,
            self.switcher.component().layer(cx, &self.switcher.items),
        );
        self.surface = Surface::QuickSwitcher;
    }

    fn update_switcher(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let was_open = cx.is_open(quick_switcher::ID);
        let mut response =
            self.switcher
                .component()
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
        if was_open && !cx.is_open(quick_switcher::ID) && self.surface == Surface::QuickSwitcher {
            self.sync_active_tab();
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
                    cx.focus(EXPLORER);
                }
                return;
            }
            SwitchTarget::Database(name) => {
                if self.workbench.catalog.database == name {
                    cx.focus(EXPLORER);
                }
                return;
            }
            SwitchTarget::Connection(_) => false,
        };
        if changed {
            self.sync_active_tab();
            if let Some(focus) = self.query_id().or_else(|| self.result_id()) {
                cx.focus(focus);
            }
        } else {
            "Target unavailable; reopen switcher".clone_into(&mut self.status);
            if let Some(focus) = self.query_id().or_else(|| self.result_id()) {
                cx.focus(focus);
            } else {
                cx.focus(EXPLORER);
            }
        }
    }

    fn stale_destructive(&mut self) {
        self.destructive_notice = Some("Work changed; request again");
    }

    fn update_destructive_dialog(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let fallback = DestructiveIntent::Quit {
            question: String::new(),
            scope: Vec::new(),
        };
        let intent = self
            .destructive_intent
            .as_ref()
            .map_or(&fallback, |request| &request.intent);
        let response = intent.dialog().update(cx, &mut self.quit_state);
        if let Some(action) = response.action_ref() {
            let confirmed = matches!(action, DialogAction::Action(ActionKey::CONFIRM))
                && cx.is_open(QUIT_DIALOG);
            cx.close_layer(QUIT_DIALOG, None);
            if let Some(request) = self.destructive_intent.take()
                && confirmed
            {
                if !self.workbench.matches_owner(&request.owner) {
                    self.stale_destructive();
                    return response.erase();
                }
                match request.intent {
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

    fn update_tab_controls(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        for (key, tab) in self.workbench.payloads_mut() {
            if let Tab::Query(query) = tab {
                response |= query_input(key.control("query"), None)
                    .update(cx, &mut query.editor_state, &mut query.query)
                    .erase();
            }
            match tab {
                Tab::Table(table) => {
                    let grid_response =
                        Self::update_grid_view(cx, key.control("data"), &mut table.result);
                    if let Some(action) = grid_response.action_ref() {
                        Self::handle_grid(&mut self.status, action);
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
        let (columns, count) = Self::column_specs(&view.columns, view.model.is_editable());
        let grid = result_grid(id, columns.get(..count).unwrap_or(&[]));
        let response = if view.model.is_editable() {
            grid.update_editable(cx, &mut view.state, &mut view.model)
        } else {
            grid.update(cx, &mut view.state, &view.model)
        };
        if let Some(GridAction::Sort(column, direction)) = response.action_ref() {
            view.model.sort(*column, *direction);
        }
        response
    }

    fn sort_focused_column(&mut self, cx: &Cx<'_>) -> bool {
        if self
            .workbench
            .active_table()
            .is_some_and(crate::tabs::TableTab::is_structure)
        {
            return false;
        }
        let Some(tab) = self.workbench.active_key() else {
            return false;
        };
        let table = self
            .workbench
            .active_table()
            .map(|table| table.table.clone());
        let direction = {
            let Some((id, grid)) = self.workbench.active_grid_mut() else {
                return false;
            };
            if !cx.state(id).contains(StateFlags::FOCUSED) || grid.state.is_editing() {
                return false;
            }
            let Some((_, column)) = grid.state.cursor() else {
                return false;
            };
            let Some(index) = usize::from(column.raw()).checked_sub(1) else {
                return false;
            };
            if grid
                .columns
                .get(index)
                .is_some_and(|(_, ty)| matches!(ty, ColType::Json))
            {
                return false;
            }
            let direction = match self.grid_sort {
                Some((sorted_tab, sorted_column, SortDir::Asc))
                    if sorted_tab == tab && sorted_column == column.raw() =>
                {
                    SortDir::Desc
                }
                _ => SortDir::Asc,
            };
            if grid.model.pending_total() == 0
                && let Some(table) = table.as_ref()
            {
                grid.model = sorted_scan(table, grid, index, direction);
            } else {
                grid.model.sort(column, direction);
            }
            let row = GridModel::row_key(&grid.model, 0);
            let (columns, count) = Self::column_specs(&grid.columns, grid.model.is_editable());
            let _ = result_grid(id, columns.get(..count).unwrap_or(&[])).move_cursor_to(
                &mut grid.state,
                &grid.model,
                row,
                column,
            );
            direction
        };
        let Some((_, column)) = self
            .workbench
            .active_grid()
            .and_then(|(_, grid)| grid.state.cursor())
        else {
            return false;
        };
        self.grid_sort = Some((tab, column.raw(), direction));
        true
    }

    fn sort_status_prefix(&self) -> Option<String> {
        let (tab, column, direction) = self.grid_sort?;
        if self.workbench.active_key()? != tab {
            return None;
        }
        let (_, grid) = self.workbench.active_grid()?;
        let index = usize::from(column).checked_sub(1)?;
        let name = grid.columns.get(index)?.0.as_str();
        Some(format!("sort {name} {}", sort_mark(direction)))
    }

    fn draw_table_data(&self, ui: &mut Ui<'_>, inner: termrock::Rect) {
        paint_data_mode(ui, inner);
        let grid_area = termrock::Rect {
            y: inner.y.saturating_add(3),
            height: inner.height.saturating_sub(5),
            ..inner
        };
        let Some((id, grid_view)) = self.workbench.active_grid() else {
            return;
        };
        let (columns, count) =
            Self::column_specs(&grid_view.columns, grid_view.model.is_editable());
        draw_painted_grid(
            ui,
            grid_area,
            id,
            &grid_view.columns,
            columns.get(..count).unwrap_or(&[]),
            &grid_view.state,
            &grid_view.model,
        );
        let shown = usize::from(grid_area.height.saturating_sub(1));
        let loaded = grid_view.model.row_count();
        let last = shown.min(loaded);
        let cols = grid_column_status(grid_view.columns.len());
        let mut status = format!(
            "rows 1–{} of {} loaded · {} total · {cols}",
            grouped(last),
            grouped(loaded),
            grouped(grid_view.model.total()),
        );
        if let Some(prefix) = self.sort_status_prefix() {
            status = format!("{prefix} · {status}");
        }
        let status_area = termrock::Rect {
            y: inner.bottom().saturating_sub(1),
            height: 1,
            ..inner
        };
        paint_horizontal_bar(
            ui,
            termrock::Rect {
                y: status_area.y.saturating_sub(1),
                height: 1,
                ..status_area
            },
        );
        let style = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
        ui.paint_str(
            termrock::Rect {
                x: status_area.x.saturating_add(1),
                width: status_area.width.saturating_sub(1),
                ..status_area
            },
            &status,
            style,
        );
    }

    fn draw_result_grid(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        if let Some((id, grid)) = self.workbench.active_grid() {
            let (columns, count) = Self::column_specs(&grid.columns, grid.model.is_editable());
            draw_painted_grid(
                ui,
                area,
                id,
                &grid.columns,
                columns.get(..count).unwrap_or(&[]),
                &grid.state,
                &grid.model,
            );
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
        let ssl = if connection.ssl { "on" } else { "off" };
        let ssh = connection.ssh.clone().unwrap_or_else(|| "off".to_owned());
        let properties = [
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
                legacy_environment_role(connection.environment),
            ),
            (
                "Safe Mode",
                format!(
                    "{} · {}",
                    connection.safe_mode.label(),
                    connection.safe_mode.description()
                ),
                Role::Fg(FgStep::Secondary),
            ),
            ("SSL", ssl.to_owned(), Role::Fg(FgStep::Secondary)),
            ("SSH", ssh, Role::Fg(FgStep::Secondary)),
            ("Last used", connection.last_used.clone(), TAG_CHROME_FG),
        ];
        let card_area = termrock::Rect {
            width: area.width.min(70),
            height: area.height.min(17),
            ..area
        };
        Self::connection_details_panel(&connection.name).draw(ui, card_area, |ui, area| {
            draw_connection_properties(ui, area, &properties);
        });
        paint_detail_rule(ui, card_area);
        if !ui.is_inert() {
            ui.register_control(CONNECTION_DETAILS, card_area, Focusability::ClickOnly);
        }
    }

    fn draw_connections(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        let count = format!("{} ", self.connections_screen.connections.len());
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
        let panel = Self::connections_panel(" Connections ", Some(&count));
        let inner = panel.inner(ui, list_area);
        let body = legacy_tree_body(inner);
        panel.draw(ui, list_area, |_, _| {});
        paint_frame_title_tail(ui, list_area, " Connections ", Role::BorderStrong);
        paint_panel_tail(ui, list_area, Role::BorderStrong);
        ui.with_area(body, |ui| {
            let filter = termrock::Rect {
                x: body.x,
                y: inner.y.saturating_add(1),
                width: body.width,
                height: 1.min(inner.height),
            };
            paint_legacy_filter(ui, filter, "Filter connections", TAG_CHROME_FG);
            let tree_area = termrock::Rect {
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
            paint_connection_focus_bar(
                ui,
                tree_area,
                &self.connection_visual_tree_state,
                &self.connection_nodes,
            );
        });
        let blank = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
        ui.fill(
            termrock::Rect {
                x: body.x.saturating_add(2),
                y: body.y,
                width: body.width.saturating_sub(1),
                height: 1,
            },
            blank,
        );
        let field = ui.surface_style().patch(
            ui.paint_patch(&StylePatch::new().set_bg(Role::Surface(termrock::Surface::Field))),
        );
        ui.fill(
            termrock::Rect {
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
                termrock::Rect {
                    x: body.right(),
                    y: body.y.saturating_add(2),
                    width: 1,
                    height: 1,
                },
                ui.surface_style().add_modifier(Modifier::BOLD),
            );
        }
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

    fn draw_explorer(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        // One extra trailing space shifts the framed meta one cell left.
        let meta = format!("{}     ", self.workbench.current_schema());
        let focused = ui.state(EXPLORER).contains(StateFlags::FOCUSED);
        let panel = Self::explorer_panel(&meta, focused);
        let inner = panel.inner(ui, area);
        let body = legacy_tree_body(inner);
        panel.draw(ui, area, |_, _| {});
        paint_panel_tail(
            ui,
            area,
            if focused {
                Role::BorderStrong
            } else {
                TAG_RULE_FG
            },
        );
        if focused {
            paint_explorer_title_space(ui, area);
        } else {
            paint_explorer_title_gap(ui, area);
        }
        paint_explorer_meta_band(ui, area);
        let tree_area = termrock::Rect {
            y: body.y.saturating_add(2),
            height: inner.height.saturating_sub(2),
            ..body
        };
        ui.with_area(body, |ui| {
            let filter = termrock::Rect {
                x: body.x,
                y: inner.y.saturating_add(1),
                width: body.width,
                height: 1.min(inner.height),
            };
            paint_legacy_filter(ui, filter, "Filter objects", TAG_CHROME_FG);
            explorer_tree().draw(
                ui,
                tree_area,
                &self.explorer_tree_state,
                &self.explorer_nodes,
            );
            paint_explorer_indent(
                ui,
                tree_area,
                &self.explorer_tree_state,
                &self.explorer_nodes,
                focused,
            );
            paint_chosen_explorer_row(
                ui,
                tree_area,
                &self.explorer_tree_state,
                &self.explorer_nodes,
            );
        });
        paint_explorer_collapse(ui, area);
        paint_explorer_chevron_gap(ui, area);
        paint_explorer_outside(
            ui,
            &self.explorer_tree_state,
            &self.explorer_nodes,
            inner,
            body,
            tree_area,
        );
        if focused {
            paint_focused_schema_mark(
                ui,
                area,
                tree_area,
                &self.explorer_tree_state,
                &self.explorer_nodes,
            );
        }
    }

    fn draw_content(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        let explorer_focused = ui.state(EXPLORER).contains(StateFlags::FOCUSED);
        let (title, meta) = match self.workbench.active() {
            Some(Tab::Table(table)) => (
                format!(
                    " {} ",
                    qualified_label("", &table.table.schema, &table.table.name)
                ),
                Some(format!("{} cols ", table.table.columns.len())),
            ),
            Some(Tab::Query(query)) if !explorer_focused => (format!(" {}", query.name), None),
            Some(Tab::Query(query)) => (format!(" {} ", query.name), None),
            Some(Tab::History(_)) => (
                " Query history".to_owned(),
                Some(format!("{} entries ", self.workbench.history.entries.len())),
            ),
            None => (" Workbench".to_owned(), None),
        };
        let panel = Self::content_panel(&title, meta.as_deref(), !explorer_focused);
        panel.draw(ui, area, |ui, inner| {
            self.draw_tab_body(ui, area, inner, explorer_focused);
        });
        paint_content_seam(
            ui,
            area,
            if explorer_focused {
                TAG_RULE_FG
            } else {
                Role::BorderStrong
            },
        );
        if let Some(Tab::Table(table)) = self.workbench.active() {
            let x = area
                .x
                .saturating_add(area.width)
                .saturating_sub(ui.design().space.frame_inset);
            let quiet = ui
                .surface_style()
                .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
            if table.is_structure() {
                for y in [area.y.saturating_add(2), area.y.saturating_add(5)] {
                    ui.paint_str(termrock::Rect::new(x, y, 1, 1), "─", quiet);
                }
                ui.paint_str(
                    termrock::Rect::new(x, area.y.saturating_add(8), 1, 1),
                    " ",
                    ui.surface_style().add_modifier(Modifier::BOLD),
                );
            } else {
                ui.paint_str(
                    termrock::Rect::new(x, area.y.saturating_add(2), 1, 1),
                    "─",
                    quiet,
                );
            }
        }
        if let Some((grid_area, rows)) = GRID_SCROLL.with(|slot| *slot.borrow()) {
            paint_grid_scrollbar(ui, grid_area, rows);
        }
        if explorer_focused
            && matches!(self.workbench.active(), Some(Tab::Query(query)) if query.result.is_none())
        {
            paint_idle_query_tail(ui, area);
        }
        paint_frame_title_tail(
            ui,
            area,
            &title,
            if explorer_focused {
                TAG_RULE_FG
            } else {
                Role::BorderStrong
            },
        );
        if meta.is_some() {
            paint_panel_tail(ui, area, Role::BorderStrong);
        }
    }

    fn draw_tab_body(
        &self,
        ui: &mut Ui<'_>,
        area: termrock::Rect,
        inner: termrock::Rect,
        explorer_focused: bool,
    ) {
        match self.workbench.active() {
            Some(Tab::Query(query)) if explorer_focused && query.result.is_none() => {
                // Keep the input in the focus ring. The idle chrome covers it.
                if let Some(id) = self.query_id() {
                    Field::new("SQL query", query_input(id, Some(&query.query)))
                        .plain(true)
                        .draw(ui, inner, &query.editor_state);
                }
                paint_idle_query(ui, area, inner);
            }
            Some(Tab::Query(query)) => {
                let rows = fixed_flex_pair(inner, 3);
                if let Some(id) = self.query_id() {
                    Field::new("SQL query", query_input(id, Some(&query.query)))
                        .plain(true)
                        .draw(ui, rows[0], &query.editor_state);
                }
                if query.plan.is_some() {
                    ui.paint_str(rows[1], "Explain plan ready", ui.surface_style());
                } else if query.error.is_some() {
                    ui.paint_str(rows[1], "Query error", ui.surface_style());
                } else if query.result.is_some() {
                    self.draw_result_grid(ui, rows[1]);
                } else {
                    let message = termrock::Rect {
                        x: rows[1].x,
                        y: rows[1].y.saturating_add(rows[1].height / 2),
                        width: rows[1].width,
                        height: 1,
                    };
                    ui.paint_str(message, "No results yet", ui.surface_style());
                }
            }
            Some(Tab::Table(table)) => {
                if table.is_structure() {
                    if let Some(key) = self.workbench.active_key() {
                        ui.register_control(key.control("data"), inner, Focusability::Focusable);
                    }
                    paint_structure_view(ui, inner, &table.table);
                } else {
                    self.draw_table_data(ui, inner);
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryOutcome {
    Executed { rows: usize, editable: bool },

    ConfirmationRequired { deliberate: bool, summary: String },

    Denied { summary: String },

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

fn query_input(id: Id, value: Option<&str>) -> TextInput<'_> {
    let input = TextInput::new(id)
        .blur(termrock::BlurPolicy::Keep)
        .placeholder("Type SQL. Ctrl+R runs the statement under the cursor.");
    match value {
        Some(text) => input.value(text),
        None => input,
    }
}

fn paint_idle_query(ui: &mut Ui<'_>, area: termrock::Rect, inner: termrock::Rect) {
    if inner.height < 8 || inner.width == 0 {
        return;
    }
    let inset = ui.design().space.frame_inset;
    let body = termrock::Rect {
        x: area.x.saturating_add(inset),
        y: inner.y,
        width: area.width.saturating_sub(inset),
        height: inner.height,
    };
    let top = idle_query_top(inner.height);
    let editor_rows = top.saturating_sub(2);
    paint_query_toolbar(ui, termrock::Rect { height: 1, ..body });
    let editor = termrock::Rect {
        y: body.y.saturating_add(1),
        height: editor_rows,
        ..body
    };
    paint_query_editor(ui, editor);
    let seam_y = editor.y.saturating_add(editor.height);
    paint_query_seam(
        ui,
        termrock::Rect {
            y: seam_y,
            height: 1,
            ..body
        },
    );
    let results_y = seam_y.saturating_add(1);
    let lane = termrock::Rect {
        x: area.x.saturating_add(1),
        y: results_y,
        width: area.width.saturating_sub(2),
        height: inner.bottom().saturating_sub(results_y),
    };
    paint_query_empty(ui, lane);
}

fn idle_query_top(height: u16) -> u16 {
    let ratio = u16::try_from(u32::from(height).saturating_mul(38) / 100).unwrap_or(height);
    ratio.max(4).min(height.saturating_sub(6).max(4))
}

fn paint_query_toolbar(ui: &mut Ui<'_>, area: termrock::Rect) {
    let button = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_BUTTON_FG)
                .set_bg(TAG_BUTTON_BG)
                .add(Modifier::BOLD),
        ),
    );
    let cap = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_BUTTON_BG)
                .set_bg(TAG_BUTTON_BG),
        ),
    );
    let quiet = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    let gap = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Custom(termrock::Color::from_u32(0)))
                .set_bg(Role::Custom(termrock::Color::from_u32(0))),
        ),
    );
    ui.paint_str(
        termrock::Rect {
            width: 1,
            height: 1,
            ..area
        },
        " ",
        cap,
    );
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(1),
            width: 4,
            height: 1,
            ..area
        },
        "Run ",
        button,
    );
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(5),
            width: 1,
            height: 1,
            ..area
        },
        " ",
        ui.surface_style(),
    );
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(6),
            width: 1,
            height: 1,
            ..area
        },
        " ",
        gap,
    );
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(7),
            width: 8,
            height: 1,
            ..area
        },
        "Run all ",
        quiet,
    );
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(16),
            width: 1,
            height: 1,
            ..area
        },
        " ",
        gap,
    );
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(17),
            width: 8,
            height: 1,
            ..area
        },
        "Explain ",
        quiet,
    );
}

fn paint_query_editor(ui: &mut Ui<'_>, area: termrock::Rect) {
    let field = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(TAG_FIELD_BG),
        ),
    );
    let muted = field.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
    let edge = field.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_FIELD_BG)));
    ui.fill(area, field);
    if area.height == 0 || area.width < 8 {
        return;
    }
    let line = termrock::Rect { height: 1, ..area };
    ui.paint_str(termrock::Rect { width: 1, ..line }, " ", edge);
    ui.paint_str(
        termrock::Rect {
            x: line.x.saturating_add(3),
            width: 2,
            ..line
        },
        " 1",
        muted,
    );
    ui.paint_str(
        termrock::Rect {
            x: line.x.saturating_add(6),
            width: area.width.saturating_sub(6),
            ..line
        },
        "Type SQL. Ctrl+R runs the statement under the cursor.",
        muted,
    );
}

fn paint_query_seam(ui: &mut Ui<'_>, area: termrock::Rect) {
    if area.width == 0 {
        return;
    }
    let style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::BorderStrong)));
    ui.paint_str(area, &"─".repeat(usize::from(area.width)), style);
}

fn paint_query_empty(ui: &mut Ui<'_>, area: termrock::Rect) {
    if area.height < 3 || area.width == 0 {
        return;
    }
    let title = "No results yet";
    let hint = "Ctrl+R runs the statement under the cursor · Alt+R runs all";
    let top = area.y.saturating_add(area.height.saturating_sub(3) / 2);
    let title_style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
    let hint_style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(LEGACY_FAINT)));
    paint_centered(
        ui,
        termrock::Rect {
            y: top,
            height: 1,
            ..area
        },
        title,
        width(title),
        title_style,
    );
    // `·` is one cell here. `width` counts it wider and shifts the line.
    let hint_columns = u16::try_from(hint.chars().count()).unwrap_or(0);
    let hint_row = termrock::Rect {
        y: top.saturating_add(2),
        height: 1,
        ..area
    };
    // Integer centering lands one cell left of the historical hint.
    let hint_x = hint_row
        .x
        .saturating_add(hint_row.width.saturating_sub(hint_columns) / 2)
        .saturating_add(1);
    ui.paint_str(
        termrock::Rect {
            x: hint_x,
            width: hint_columns,
            ..hint_row
        },
        hint,
        hint_style,
    );
}

fn paint_centered(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    text: &str,
    text_width: u16,
    style: PaintStyle,
) {
    if text_width > area.width {
        ui.paint_str(area, text, style);
        return;
    }
    ui.paint_str(
        termrock::Rect {
            x: area
                .x
                .saturating_add(area.width.saturating_sub(text_width) / 2),
            width: text_width,
            ..area
        },
        text,
        style,
    );
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

fn build_connection_nodes(connections: &[Connection]) -> Vec<ConnectionNode> {
    let mut nodes = Vec::with_capacity(connections.len().saturating_mul(2));
    let mut groups: Vec<&str> = Vec::new();
    for connection in connections {
        if !groups.contains(&connection.group.as_str()) {
            groups.push(connection.group.as_str());
        }
    }
    for group in groups {
        nodes.push(ConnectionNode::Group {
            name: group.to_owned(),
        });
        for (index, connection) in connections.iter().enumerate() {
            if connection.group == group {
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

fn legacy_environment_role(environment: Environment) -> Role {
    match environment {
        Environment::Local => TAG_CHROME_FG,
        Environment::Development => LEGACY_INFO,
        Environment::Staging => Role::Warning,
        Environment::Production => LEGACY_DANGER_LABEL,
    }
}

fn connection_mark(connection: &Connection) -> Span<'static> {
    match connection.environment {
        Environment::Local => Span::new(" "),
        Environment::Development => Span::new("·").role(LEGACY_INFO),
        Environment::Staging => Span::new("◇").role(Role::Warning),
        Environment::Production => Span::new("◆").role(LEGACY_DANGER_LABEL),
    }
}

fn connection_row(node: &ConnectionNode, row: &mut RowUi<'_>) {
    match node {
        ConnectionNode::Group { name } => row.label(name),
        ConnectionNode::Connection { connection, .. } => {
            row.label_spans(&[
                connection_mark(connection),
                Span::new(" "),
                Span::new(&connection.name),
            ]);
            row.trailing(
                connection.engine.short(),
                &StylePatch::new().set_fg(TAG_CHROME_FG),
            );
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
            (ObjectKind::Sequence, "Sequences", "#"),
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
            || matches!(node, ExplorerNode::Group { schema: group_schema, name } if group_schema == schema && name == "Tables")
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
        ExplorerNode::Group { schema, name } => stable_key(&["object-group", schema, name]),
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

fn explorer_row(node: &ExplorerNode, row: &mut RowUi<'_>) {
    match node {
        ExplorerNode::Database { name } => row.label_spans(&[
            Span::new("D").role(TAG_CHROME_FG),
            Span::new(" "),
            Span::new(name.as_str()),
        ]),
        ExplorerNode::Schema { name } => row.label_spans(&[
            Span::new("S").role(TAG_CHROME_FG),
            Span::new(" "),
            Span::new(name.as_str()),
        ]),
        ExplorerNode::Group { name, .. } => row.label(name),
        ExplorerNode::Object { item, prefix, .. } => {
            row.label_spans(&[
                Span::new(prefix).role(TAG_CHROME_FG),
                Span::new(" "),
                Span::new(&item.name),
            ]);
        }
    }
}

fn tab_key(tab: &TabRecord) -> ItemKey {
    ItemKey::num(tab.key().get())
}

fn tab_row(tab: &TabRecord, row: &mut RowUi<'_>) {
    match tab.payload() {
        Tab::Table(table) => {
            row.label_fmt(format_args!("T {}", table.table.name));
        }
        Tab::Query(query) => {
            row.label_fmt(format_args!("≡ {}", query.name));
        }
        Tab::History(_) => row.label("H History"),
    }
}

fn paint_connection_focus_bar(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    state: &TreeState,
    nodes: &[ConnectionNode],
) {
    let Some(cursor) = state.cursor() else {
        return;
    };
    let Some(index) = nodes
        .iter()
        .position(|node| connection_node_key(node) == cursor)
    else {
        return;
    };
    let offset = state.scroll().offset();
    let Some(row) = index.checked_sub(offset) else {
        return;
    };
    let y = area
        .y
        .saturating_add(u16::try_from(row).unwrap_or(u16::MAX));
    if y >= area.bottom() {
        return;
    }
    let style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent).add(Modifier::BOLD)));
    ui.paint_str(
        termrock::Rect {
            x: area.x,
            y,
            width: 1,
            height: 1,
        },
        "▎",
        style,
    );
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
        .patch_part(&CONNECTION_TREE_PATCH)
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
        .branch_activation(TreeBranchActivation::Activate)
        .slot(Part::MARKER, &paint_quiet_gutter)
}

const ID_HEADER_PREFIX: [(ColumnKey, GlyphRole, Role); 1] =
    [(ColumnKey::num(1), GlyphRole::PrimaryKey, LEGACY_FAINT)];

fn tab_strip()
-> Tabs<'static, TabRecord, impl Fn(&TabRecord) -> ItemKey, impl Fn(&TabRecord, &mut RowUi<'_>)> {
    Tabs::new(TAB_STRIP)
        .key(tab_key)
        .row(tab_row)
        .allow_new(true)
        .closable(true)
        .patch_part(&TAB_STRIP_PATCH)
}

fn legacy_tree_body(inner: termrock::Rect) -> termrock::Rect {
    termrock::Rect {
        x: inner.x.saturating_sub(1),
        width: inner.width.saturating_add(1),
        ..inner
    }
}

fn workbench_split() -> SplitPane<'static> {
    SplitPane::new(WORKBENCH_SPLIT, SplitAxis::Horizontal)
        .min_first(28)
        .min_second(20)
}

#[derive(Clone, Copy)]
struct ColBudget {
    kind: u8,
    text: u16,
    arrow: bool,
}

#[derive(Clone, Copy)]
struct ColPlace {
    x: u16,
    width: u16,
    primary: bool,
}

#[derive(Clone)]
struct GridChrome {
    gutter: u16,
    content_width: u16,
    complete: usize,
    columns: usize,
    hidden_left: usize,
    places: Vec<ColPlace>,
}

const KIND_OTHER: u8 = 0;
const KIND_PRIMARY: u8 = 1;
const KIND_FOREIGN: u8 = 2;

thread_local! {
    static COL_BUDGETS: RefCell<Vec<ColBudget>> = const { RefCell::new(Vec::new()) };
    static GRID_CHROME: RefCell<Option<GridChrome>> = const { RefCell::new(None) };
    static GRID_SCROLL: RefCell<Option<(termrock::Rect, usize)>> = const { RefCell::new(None) };
    static GRID_SORT: RefCell<Option<(usize, SortDir)>> = const { RefCell::new(None) };
}

fn install_grid_budgets(area_width: u16, columns: &[(String, ColType)], model: &ResultGrid) {
    let row_count = model.row_count().max(1).to_string();
    let digits = width(&row_count);
    let gutter = digits.saturating_add(4);
    let avail = area_width
        .saturating_sub(1)
        .saturating_sub(gutter)
        .saturating_sub(3);
    let mut budgets = Vec::with_capacity(columns.len());
    let mut places = Vec::new();
    let mut used = 0u16;
    let mut closed = false;
    let mut complete = 0usize;
    let sample_rows = model.row_count().min(24);
    for (index, (name, ty)) in columns.iter().enumerate() {
        let primary = name == "id";
        let foreign = *ty == ColType::Uuid && !primary;
        let kind = if primary {
            KIND_PRIMARY
        } else if foreign {
            KIND_FOREIGN
        } else {
            KIND_OTHER
        };
        let mut natural = width(name);
        if primary {
            natural = natural.saturating_add(2).max(38);
        }
        for row in 0..sample_rows {
            if let Some(cell) = GridModel::cell(model, row, index) {
                natural = natural.max(width(cell.text));
            }
        }
        natural = natural.clamp(3, 40);
        if closed {
            budgets.push(ColBudget {
                kind,
                text: 0,
                arrow: false,
            });
            continue;
        }
        let gap = if places.is_empty() { 0 } else { 2 };
        let needed = used.saturating_add(gap).saturating_add(natural);
        if !places.is_empty() && needed > avail {
            let room = avail.saturating_sub(used).saturating_sub(gap);
            if room >= 4 {
                let arrow = foreign;
                let text = room.saturating_sub(if arrow { 2 } else { 0 });
                budgets.push(ColBudget { kind, text, arrow });
                places.push(ColPlace {
                    x: used.saturating_add(gap),
                    width: room,
                    primary,
                });
            } else {
                budgets.push(ColBudget {
                    kind,
                    text: 0,
                    arrow: false,
                });
            }
            closed = true;
            continue;
        }
        budgets.push(ColBudget {
            kind,
            text: natural,
            arrow: false,
        });
        places.push(ColPlace {
            x: used.saturating_add(gap),
            width: natural,
            primary,
        });
        used = needed;
        complete = complete.saturating_add(1);
    }
    let chrome = GridChrome {
        gutter,
        content_width: avail,
        complete,
        columns: columns.len(),
        hidden_left: 0,
        places,
    };
    COL_BUDGETS.with(|budgets_cell| *budgets_cell.borrow_mut() = budgets);
    GRID_CHROME.with(|chrome_cell| *chrome_cell.borrow_mut() = Some(chrome));
}

struct PaintedGrid<'a> {
    inner: &'a ResultGrid,
}

impl GridModel for PaintedGrid<'_> {
    fn row_count(&self) -> usize {
        self.inner.row_count()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        GridModel::row_key(self.inner, row)
    }

    fn cell(&self, row: usize, col: usize) -> Option<termrock::CellRef<'_>> {
        GridModel::cell(self.inner, row, col)
    }

    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        GridModel::row_decor(self.inner, row)
    }

    fn cell_decor(&self, row: usize, col: usize) -> termrock::CellDecor<'_> {
        GridModel::cell_decor(self.inner, row, col)
    }

    fn total(&self) -> RowTotal {
        GridModel::total(self.inner)
    }

    fn read_only_reason(&self) -> Option<&str> {
        GridModel::read_only_reason(self.inner)
    }

    fn actions(&self, row: usize, col: usize) -> &[CellAction] {
        let arrow = COL_BUDGETS
            .with(|budgets| budgets.borrow().get(col).is_some_and(|budget| budget.arrow));
        if arrow && GridModel::cell(self.inner, row, col).is_some_and(|cell| !cell.text.is_empty())
        {
            &FOLLOW_REFS
        } else {
            &[]
        }
    }
}

fn tab_label(tab: &TabRecord) -> (&'static str, &str) {
    match tab.payload() {
        Tab::Table(table) => ("T", table.table.name.as_str()),
        Tab::Query(query) => ("≡", query.name.as_str()),
        Tab::History(_) => ("H", "History"),
    }
}

fn legacy_tab_width(prefix: &str, label: &str) -> u16 {
    1u16.saturating_add(width(label))
        .saturating_add(2)
        .saturating_add(width(prefix))
        .saturating_add(1)
        .saturating_add(2)
}

#[allow(
    clippy::too_many_lines,
    reason = "legacy tab geometry, prefix ink, and the active rule are one paint"
)]
fn paint_legacy_tabs(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    tabs: &[TabRecord],
    active: Option<ItemKey>,
) {
    if area.height < 2 || tabs.is_empty() {
        return;
    }
    let base = ui.surface_style();
    let inactive = base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    let active_fill = base.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_bg(Role::Surface(termrock::Surface::Overlay))
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
    );
    let prefix_idle = base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
    let prefix_active = active_fill
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG).remove(Modifier::BOLD)));
    let close_active = active_fill
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG).remove(Modifier::BOLD)));
    let mut x = area.x;
    let mut active_span: Option<(u16, u16)> = None;
    for tab in tabs {
        let (prefix, label) = tab_label(tab);
        let tab_w = legacy_tab_width(prefix, label);
        if x.saturating_add(tab_w) > area.right().saturating_sub(6) {
            break;
        }
        let on = active == Some(tab_key(tab));
        let body = if on { active_fill } else { inactive };
        let prefix_style = if on { prefix_active } else { prefix_idle };
        let close_style = if on { close_active } else { inactive };
        ui.paint_str(
            termrock::Rect {
                x,
                y: area.y,
                width: tab_w,
                height: 1,
            },
            &" ".repeat(usize::from(tab_w)),
            body,
        );
        let mut cursor = x.saturating_add(1);
        ui.paint_str(
            termrock::Rect {
                x: cursor,
                y: area.y,
                width: width(prefix),
                height: 1,
            },
            prefix,
            prefix_style,
        );
        cursor = cursor.saturating_add(width(prefix)).saturating_add(1);
        ui.paint_str(
            termrock::Rect {
                x: cursor,
                y: area.y,
                width: width(label),
                height: 1,
            },
            label,
            body,
        );
        let close_x = x.saturating_add(tab_w).saturating_sub(3);
        ui.paint_str(
            termrock::Rect {
                x: close_x,
                y: area.y,
                width: 1,
                height: 1,
            },
            "×",
            close_style,
        );
        if on {
            active_span = Some((x, tab_w));
        }
        x = x.saturating_add(tab_w);
        ui.paint_str(
            termrock::Rect {
                x,
                y: area.y,
                width: 1,
                height: 1,
            },
            " ",
            base.remove_modifier(Modifier::BOLD),
        );
        x = x.saturating_add(1);
    }
    let quiet = base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    let hot = base.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
    let rule = termrock::Rect {
        y: area.y.saturating_add(1),
        height: 1,
        ..area
    };
    if rule.width > 0 {
        ui.paint_str(rule, &"─".repeat(usize::from(rule.width)), quiet);
    }
    if let Some((start, tab_w)) = active_span {
        ui.paint_str(
            termrock::Rect {
                x: start,
                y: rule.y,
                width: tab_w.min(rule.right().saturating_sub(start)),
                height: 1,
            },
            &"━".repeat(usize::from(tab_w)),
            hot,
        );
    }
}

fn explorer_depth(node: &ExplorerNode) -> usize {
    match node {
        ExplorerNode::Database { .. } => 0,
        ExplorerNode::Schema { .. } => 1,
        ExplorerNode::Group { .. } => 2,
        ExplorerNode::Object { .. } => 3,
    }
}

fn visible_explorer_index(
    nodes: &[ExplorerNode],
    state: &TreeState,
    key: ItemKey,
) -> Option<usize> {
    let mut show = [false; 8];
    show[0] = true;
    let mut index = 0usize;
    for node in nodes {
        let depth = explorer_depth(node);
        if !show.get(depth).copied().unwrap_or(false) {
            continue;
        }
        let node_key = explorer_node_key(node);
        if node_key == key {
            return Some(index);
        }
        index = index.saturating_add(1);
        let expanded = state.is_expanded(node_key);
        if let Some(child) = show.get_mut(depth.saturating_add(1)) {
            *child = expanded;
        }
        for deeper in show.iter_mut().skip(depth.saturating_add(2)) {
            *deeper = false;
        }
    }
    None
}

fn paint_chosen_explorer_row(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    state: &TreeState,
    nodes: &[ExplorerNode],
) {
    let Some(key) = state.chosen() else {
        return;
    };
    let Some(index) = visible_explorer_index(nodes, state, key) else {
        return;
    };
    let Some(node) = nodes.iter().find(|node| explorer_node_key(node) == key) else {
        return;
    };
    let y = area.y.saturating_add(index as u16);
    if y >= area.bottom() {
        return;
    }
    let row = termrock::Rect {
        x: area.x,
        y,
        width: area.width.saturating_add(1),
        height: 1,
    };
    let fill = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_bg(LEGACY_SELECT_BG)
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
    );
    ui.fill(row, fill);
    ui.paint_str(
        termrock::Rect {
            x: area.x,
            y,
            width: 1,
            height: 1,
        },
        " ",
        fill.with_fg_from_bg(fill),
    );
    let depth = explorer_depth(node);
    let fold_x = area
        .x
        .saturating_add(1)
        .saturating_add((depth as u16).saturating_mul(ui.design().space.tree_indent));
    let mark = fill.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    ui.paint_str(
        termrock::Rect {
            x: fold_x,
            y,
            width: 1,
            height: 1,
        },
        "▸",
        mark,
    );
    let ExplorerNode::Object { item, prefix, .. } = node else {
        return;
    };
    let prefix_style = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_CHROME_FG)
                .set_bg(LEGACY_SELECT_BG)
                .remove(Modifier::BOLD),
        ),
    );
    let name_style = fill.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
    let text_x = fold_x.saturating_add(2);
    ui.paint_str(
        termrock::Rect {
            x: text_x,
            y,
            width: 1,
            height: 1,
        },
        prefix,
        prefix_style,
    );
    ui.paint_str(
        termrock::Rect {
            x: text_x.saturating_add(2),
            y,
            width: width(&item.name).saturating_add(7),
            height: 1,
        },
        &format!("{}       ", item.name),
        name_style,
    );
}

fn paint_quiet_gutter(ui: &mut Ui<'_>, area: termrock::Rect) {
    let base = ui.surface_style();
    ui.paint_str(area, " ", base.with_fg_from_bg(base));
}

fn paint_result_cell(cell: GridCell<'_>, ui: &mut CellUi<'_>) {
    let budget = COL_BUDGETS.with(|budgets| budgets.borrow().get(cell.column).copied());
    let Some(budget) = budget else {
        ui.text(cell.value.text);
        return;
    };
    if budget.text == 0 {
        return;
    }
    let shown = if budget.kind == KIND_FOREIGN {
        truncate_middle(cell.value.text, budget.text)
    } else if width(cell.value.text) > budget.text {
        truncate(cell.value.text, budget.text)
    } else {
        cell.value.text.to_owned()
    };
    ui.text(&shown);
    if cell.flags.contains(StateFlags::ACTIVE) {
        ui.patch(
            &StylePatch::new()
                .set_bg(LEGACY_CELL_BG)
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        );
    } else if budget.kind == KIND_PRIMARY {
        ui.patch(&StylePatch::new().set_fg(TAG_PATH_FG));
    } else if cell.flags.contains(StateFlags::FOCUSED) {
        ui.patch(&StylePatch::new().add(Modifier::BOLD));
    }
}

fn paint_grid_focus_bar(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    state: &termrock::GridState,
    model: &ResultGrid,
) {
    let Some((key, _)) = state.cursor() else {
        return;
    };
    let Some(row) = (0..model.row_count()).find(|row| GridModel::row_key(model, *row) == key)
    else {
        return;
    };
    let y = area.y.saturating_add(1).saturating_add(row as u16);
    if y >= area.bottom() {
        return;
    }
    let style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent).add(Modifier::BOLD)));
    ui.paint_str(
        termrock::Rect {
            x: area.x,
            y,
            width: 1,
            height: 1,
        },
        "▎",
        style,
    );
}

fn paint_grid_scrollbar(ui: &mut Ui<'_>, area: termrock::Rect, rows: usize) {
    if area.height < 2 || area.width == 0 {
        return;
    }
    let track_len = usize::from(area.height.saturating_sub(1));
    let thumb_len = track_len
        .saturating_mul(track_len)
        .checked_div(rows.max(1))
        .unwrap_or(1)
        .max(1)
        .min(track_len);
    let Some(chrome) = GRID_CHROME.with(|chrome| chrome.borrow().clone()) else {
        return;
    };
    let Some(last) = chrome.places.last() else {
        return;
    };
    let x = area
        .x
        .saturating_add(chrome.gutter)
        .saturating_add(last.x)
        .saturating_add(last.width)
        .saturating_add(4);
    let covered = x.saturating_sub(1);
    let plain = ui.surface_style();
    let track = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(LEGACY_SCROLL_TRACK)));
    let thumb = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(LEGACY_SCROLL_THUMB)));
    for offset in 0..track_len {
        let y = area.y.saturating_add(1).saturating_add(offset as u16);
        let from_end = track_len.saturating_sub(offset).saturating_sub(1);
        let cover = if from_end == 0 {
            plain.patch(ui.paint_patch(
                &StylePatch::new().set_fg(Role::Custom(termrock::Color::from_u32(0x008c_8c8c))),
            ))
        } else if from_end == 1 {
            plain.patch(ui.paint_patch(
                &StylePatch::new().set_fg(Role::Custom(termrock::Color::from_u32(0x00cc_cccc))),
            ))
        } else {
            plain
        };
        ui.paint_str(
            termrock::Rect {
                x: covered,
                y,
                width: 1,
                height: 1,
            },
            " ",
            cover,
        );
        let glyph = if offset < thumb_len { "┃" } else { "┆" };
        let style = if offset < thumb_len { thumb } else { track };
        ui.paint_str(
            termrock::Rect {
                x,
                y,
                width: 1,
                height: 1,
            },
            glyph,
            style,
        );
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "header gutter, primary key, and muted titles share one pass"
)]
fn sorted_scan(table: &db::Table, grid: &GridView, index: usize, direction: SortDir) -> ResultGrid {
    // The demo executor scans 2_000 rows, then pages the first 500. A local sort
    // has to use that scan or the first page hides earlier keys.
    let scan = table.row_count.min(2_000);
    let keep = grid.model.row_count();
    let mut rows = db::rows(table, 0, scan);
    rows.sort_by(|left, right| {
        let null = db::Value::Null;
        let cmp = crate::sql::cmp_values(
            left.get(index).unwrap_or(&null),
            right.get(index).unwrap_or(&null),
        );
        if direction == SortDir::Asc {
            cmp
        } else {
            cmp.reverse()
        }
    });
    rows.truncate(keep);
    ResultGrid::from_result(&crate::sql::ResultSet {
        columns: grid.columns.clone(),
        rows,
        total: grid.model.total(),
        source: grid.model.source().map(str::to_owned),
        duration_ms: 0,
        editable: grid.model.is_editable(),
    })
}

fn sort_mark(direction: SortDir) -> char {
    match direction {
        SortDir::Asc => '▴',
        SortDir::Desc => '▾',
    }
}

fn sort_footer(app: &TableProApp) -> Option<String> {
    let (tab, column, direction) = app.grid_sort?;
    if app.workbench.active_key()? != tab {
        return None;
    }
    let (_, grid) = app.workbench.active_grid()?;
    let index = usize::from(column).checked_sub(1)?;
    let name = grid.columns.get(index)?.0.as_str();
    let word = match direction {
        SortDir::Asc => "ascending",
        SortDir::Desc => "descending",
    };
    Some(format!("Sorted by {name} {word}"))
}

fn grid_column_status(columns: usize) -> String {
    let chrome = GRID_CHROME.with(|chrome| chrome.borrow().clone());
    let Some(chrome) = chrome else {
        return format!("cols 1–2 of {}", grouped(columns));
    };
    if chrome.hidden_left == 0 {
        return format!("cols 1–2 of {}", grouped(chrome.columns));
    }
    let first = chrome.hidden_left.saturating_add(1);
    let last = chrome.hidden_left.saturating_add(chrome.complete.max(1));
    format!(
        "cols {}–{} of {}",
        grouped(first),
        grouped(last),
        grouped(chrome.columns)
    )
}

fn column_natural_width(name: &str, ty: ColType, model: &ResultGrid, col: usize) -> u16 {
    let numeric = matches!(ty, ColType::Int | ColType::Numeric);
    let mut natural = width(name);
    if name == "id" {
        natural = natural.saturating_add(2).max(38);
    } else if !numeric {
        natural = natural.saturating_add(2);
    }
    let sample = model.row_count().min(64);
    for row in 0..sample {
        if let Some(cell) = GridModel::cell(model, row, col) {
            natural = natural.max(width(cell.text));
        }
    }
    natural.clamp(3, 40)
}

fn column_budget_avail(area_width: u16, gutter: u16) -> u16 {
    area_width
        .saturating_sub(1)
        .saturating_sub(gutter)
        .saturating_sub(3)
}

struct WindowFit {
    complete: usize,
    places: Vec<ColPlace>,
}

fn layout_window(widths: &[u16], start: usize, avail: u16) -> WindowFit {
    let mut places = Vec::new();
    let mut used = 0u16;
    let mut complete = 0usize;
    let mut closed = false;
    for width in widths.iter().copied().skip(start) {
        if closed {
            break;
        }
        let gap = if places.is_empty() { 0 } else { 2 };
        let needed = used.saturating_add(gap).saturating_add(width);
        if !places.is_empty() && needed > avail {
            let room = avail.saturating_sub(used).saturating_sub(gap);
            if room >= 4 {
                places.push(ColPlace {
                    x: used.saturating_add(gap),
                    width: room,
                    primary: false,
                });
            }
            closed = true;
            continue;
        }
        places.push(ColPlace {
            x: used.saturating_add(gap),
            width,
            primary: false,
        });
        used = needed;
        complete = complete.saturating_add(1);
    }
    WindowFit { complete, places }
}

fn focus_is_complete(widths: &[u16], start: usize, focus: usize, avail: u16) -> bool {
    if start > focus {
        return false;
    }
    let fit = layout_window(widths, start, avail);
    focus.saturating_sub(start) < fit.complete
}

fn window_start(widths: &[u16], focus: usize, avail: u16) -> usize {
    if focus == 0 || widths.is_empty() {
        return 0;
    }
    let focus = focus.min(widths.len().saturating_sub(1));
    let mut start = focus;
    while start > 0 && focus_is_complete(widths, start.saturating_sub(1), focus, avail) {
        start = start.saturating_sub(1);
    }
    start
}

fn install_scrolled_budgets(
    area_width: u16,
    columns: &[(String, ColType)],
    model: &ResultGrid,
    state: &termrock::GridState,
) -> bool {
    let Some((_, column)) = state.cursor() else {
        return false;
    };
    let Some(focus) = usize::from(column.raw()).checked_sub(1) else {
        return false;
    };
    if focus == 0 || focus >= columns.len() {
        return false;
    }
    let row_count = model.row_count().max(1).to_string();
    let gutter = width(&row_count).saturating_add(4);
    let avail = column_budget_avail(area_width, gutter);
    let widths: Vec<u16> = columns
        .iter()
        .enumerate()
        .map(|(index, (name, ty))| column_natural_width(name, *ty, model, index))
        .collect();
    let start = window_start(&widths, focus, avail);
    if start == 0 {
        return false;
    }
    let fit = layout_window(&widths, start, avail);
    let mut budgets = Vec::with_capacity(columns.len());
    for index in 0..columns.len() {
        let shown = index
            .checked_sub(start)
            .and_then(|offset| fit.places.get(offset));
        budgets.push(ColBudget {
            kind: if columns.get(index).is_some_and(|(name, _)| name == "id") {
                KIND_PRIMARY
            } else if columns
                .get(index)
                .is_some_and(|(_, ty)| *ty == ColType::Uuid)
            {
                KIND_FOREIGN
            } else {
                KIND_OTHER
            },
            text: shown.map_or(0, |place| place.width),
            arrow: false,
        });
    }
    let chrome = GridChrome {
        gutter,
        content_width: avail,
        complete: fit.complete,
        columns: columns.len(),
        hidden_left: start,
        places: fit.places,
    };
    COL_BUDGETS.with(|budgets_cell| *budgets_cell.borrow_mut() = budgets);
    GRID_CHROME.with(|chrome_cell| *chrome_cell.borrow_mut() = Some(chrome));
    true
}

fn scaled_channel(channel: u8, numerator: u16) -> u8 {
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "channel times at most 255 plus 127 fits in u32"
    )]
    let value = u32::from(channel) * u32::from(numerator) + 127;
    u8::try_from(value / 255).unwrap_or(u8::MAX)
}

fn scaled_rgb(color: u32, numerator: u16) -> u32 {
    let red = scaled_channel(u8::try_from((color >> 16) & 0xff).unwrap_or(0), numerator);
    let green = scaled_channel(u8::try_from((color >> 8) & 0xff).unwrap_or(0), numerator);
    let blue = scaled_channel(u8::try_from(color & 0xff).unwrap_or(0), numerator);
    u32::from(red) << 16 | u32::from(green) << 8 | u32::from(blue)
}

fn paint_scrolled_header(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    names: &[String],
    chrome: &GridChrome,
) {
    let base = ui.surface_style().remove_modifier(Modifier::BOLD);
    ui.fill(
        termrock::Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        },
        base,
    );
    let muted = base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
    let sorted_style = base.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
    );
    let overflow = base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    let origin = area.x.saturating_add(chrome.gutter);
    if chrome.hidden_left > 0 {
        let label = format!("‹{}", chrome.hidden_left);
        ui.paint_str(
            termrock::Rect {
                x: area.x.saturating_add(1),
                y: area.y,
                width: width(&label).min(chrome.gutter.saturating_sub(1)),
                height: 1,
            },
            &label,
            overflow,
        );
    }
    let sort_at = GRID_SORT.with(|slot| *slot.borrow());
    for (offset, place) in chrome.places.iter().enumerate() {
        let index = chrome.hidden_left.saturating_add(offset);
        let Some(name) = names.get(index) else {
            continue;
        };
        let sorted = sort_at.is_some_and(|(column, _)| column == index);
        let style = if sorted { sorted_style } else { muted };
        let mut title = name.clone();
        if let Some((_, direction)) = sort_at.filter(|(column, _)| *column == index) {
            title.push(' ');
            title.push(sort_mark(direction));
        }
        ui.fill(
            termrock::Rect {
                x: origin.saturating_add(place.x),
                y: area.y,
                width: place.width,
                height: 1,
            },
            style,
        );
        ui.paint_str(
            termrock::Rect {
                x: origin.saturating_add(place.x),
                y: area.y,
                width: place.width,
                height: 1,
            },
            &title,
            style,
        );
    }
    if chrome.hidden_left.saturating_add(chrome.complete) < chrome.columns {
        let hidden = chrome
            .columns
            .saturating_sub(chrome.hidden_left)
            .saturating_sub(chrome.complete);
        let label = format!("{hidden}›");
        let slot = width(&label).max(3);
        let end = area
            .x
            .saturating_add(chrome.gutter)
            .saturating_add(chrome.content_width)
            .saturating_add(4);
        ui.paint_str(
            termrock::Rect {
                x: end.saturating_sub(slot),
                y: area.y,
                width: width(&label),
                height: 1,
            },
            &label,
            overflow,
        );
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "scrolled cell color, fade, and alignment stay in one pass"
)]
fn paint_scrolled_cells(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    columns: &[(String, ColType)],
    state: &termrock::GridState,
    model: &ResultGrid,
) {
    let Some(chrome) = GRID_CHROME.with(|chrome| chrome.borrow().clone()) else {
        return;
    };
    let cursor = state.cursor().and_then(|(key, column)| {
        let row = (0..model.row_count()).find(|row| GridModel::row_key(model, *row) == key)?;
        usize::from(column.raw())
            .checked_sub(1)
            .map(|col| (row, col))
    });
    let visible = usize::from(area.height.saturating_sub(1)).min(model.row_count());
    let origin = area.x.saturating_add(chrome.gutter);
    let plain = ui.surface_style().remove_modifier(Modifier::BOLD);
    for row in 0..visible {
        let y = area.y.saturating_add(1).saturating_add(row as u16);
        let fade = match visible.saturating_sub(row) {
            1 => Some(140),
            2 => Some(204),
            _ => None,
        };
        let on_cursor = cursor.is_some_and(|(cursor_row, _)| cursor_row == row);
        let mut row_style = plain;
        if on_cursor {
            row_style = row_style.add_modifier(Modifier::BOLD);
        }
        if let Some(numerator) = fade {
            let color = scaled_rgb(0x00ff_ffff, numerator);
            row_style = row_style.patch(ui.paint_patch(
                &StylePatch::new().set_fg(Role::Custom(termrock::Color::from_u32(color))),
            ));
        }
        ui.fill(
            termrock::Rect {
                x: origin,
                y,
                width: chrome.content_width.saturating_add(4),
                height: 1,
            },
            row_style,
        );
        for (offset, place) in chrome.places.iter().enumerate() {
            let col = chrome.hidden_left.saturating_add(offset);
            let Some(text) = GridModel::cell(model, row, col).map(|cell| cell.text.to_owned())
            else {
                continue;
            };
            let literal = matches!(text.as_str(), "NULL" | "true" | "false");
            let active = cursor
                .is_some_and(|(cursor_row, cursor_col)| cursor_row == row && cursor_col == col);
            let mut style = row_style;
            if active {
                style = style.patch(
                    ui.paint_patch(
                        &StylePatch::new()
                            .set_fg(Role::Fg(FgStep::Primary))
                            .set_bg(LEGACY_CELL_BG)
                            .add(Modifier::BOLD),
                    ),
                );
            } else if literal {
                let mut color = 0x00c6_a0f6;
                if let Some(numerator) = fade {
                    color = scaled_rgb(color, numerator);
                }
                style = style.patch(
                    ui.paint_patch(
                        &StylePatch::new()
                            .set_fg(Role::Custom(termrock::Color::from_u32(color)))
                            .add(Modifier::ITALIC),
                    ),
                );
                if on_cursor {
                    style = style.add_modifier(Modifier::BOLD);
                }
                ui.fill(
                    termrock::Rect {
                        x: origin.saturating_add(place.x),
                        y,
                        width: place.width,
                        height: 1,
                    },
                    style,
                );
            }
            if !active
                && !literal
                && let Some(numerator) = fade
            {
                let color = scaled_rgb(0x00ff_ffff, numerator);
                style = style.patch(ui.paint_patch(
                    &StylePatch::new().set_fg(Role::Custom(termrock::Color::from_u32(color))),
                ));
            }
            let shown = if width(&text) > place.width {
                truncate(&text, place.width)
            } else {
                text
            };
            let numeric = columns
                .get(col)
                .is_some_and(|(_, ty)| matches!(ty, ColType::Int | ColType::Numeric));
            let text_width = width(&shown).min(place.width);
            let indent = if numeric {
                place.width.saturating_sub(text_width)
            } else {
                0
            };
            ui.paint_str(
                termrock::Rect {
                    x: origin.saturating_add(place.x).saturating_add(indent),
                    y,
                    width: text_width,
                    height: 1,
                },
                &shown,
                style,
            );
        }
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "header gutter, primary key, and muted titles share one pass"
)]
fn paint_grid_header(ui: &mut Ui<'_>, area: termrock::Rect, names: &[String]) {
    let Some(chrome) = GRID_CHROME.with(|chrome| chrome.borrow().clone()) else {
        return;
    };
    if chrome.hidden_left > 0 {
        paint_scrolled_header(ui, area, names, &chrome);
        return;
    }
    let base = ui.surface_style().remove_modifier(Modifier::BOLD);
    ui.fill(
        termrock::Rect {
            x: area.x,
            y: area.y,
            width: chrome.gutter,
            height: 1,
        },
        base,
    );
    let primary = base.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
    );
    let muted = base.remove_modifier(Modifier::BOLD).patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_CHROME_FG)
                .remove(Modifier::BOLD),
        ),
    );
    ui.fill(
        termrock::Rect {
            x: area.x.saturating_add(chrome.gutter),
            y: area.y,
            width: chrome.content_width,
            height: 1,
        },
        base,
    );
    let key =
        base.patch(ui.paint_patch(&StylePatch::new().set_fg(LEGACY_FAINT).add(Modifier::BOLD)));
    for (index, place) in chrome.places.iter().enumerate() {
        let origin = area.x.saturating_add(chrome.gutter).saturating_add(place.x);
        let rect = termrock::Rect {
            x: origin,
            y: area.y,
            width: place.width,
            height: 1,
        };
        let fill_width = if place.primary {
            place.width.saturating_sub(2)
        } else {
            place.width
        };
        ui.fill(
            termrock::Rect {
                width: fill_width,
                ..rect
            },
            if place.primary { primary } else { muted },
        );
        if place.primary {
            ui.fill(
                termrock::Rect {
                    x: origin.saturating_add(place.width),
                    width: 2,
                    ..rect
                },
                muted,
            );
        }
        let Some(name) = names.get(index) else {
            continue;
        };
        if place.primary {
            ui.paint_str(
                termrock::Rect {
                    x: origin,
                    y: area.y,
                    width: 1,
                    height: 1,
                },
                "⚷",
                key,
            );
            ui.paint_str(
                termrock::Rect {
                    x: origin.saturating_add(2),
                    y: area.y,
                    width: place.width.saturating_sub(2),
                    height: 1,
                },
                name,
                primary,
            );
        } else {
            ui.paint_str(
                termrock::Rect {
                    x: origin,
                    y: area.y,
                    width: place.width,
                    height: 1,
                },
                name,
                muted,
            );
        }
    }
    if chrome.columns > chrome.complete {
        let hidden = chrome.columns.saturating_sub(chrome.complete);
        let label = format!("{hidden}›");
        let label_w = width(&label);
        let end = area
            .x
            .saturating_add(chrome.gutter)
            .saturating_add(chrome.content_width)
            .saturating_add(4);
        let label_x = end.saturating_sub(label_w);
        if label_x > area.x {
            ui.paint_str(
                termrock::Rect {
                    x: label_x.saturating_sub(1),
                    y: area.y,
                    width: 1,
                    height: 1,
                },
                " ",
                base,
            );
        }
        ui.paint_str(
            termrock::Rect {
                x: label_x,
                y: area.y,
                width: label_w,
                height: 1,
            },
            &label,
            base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG))),
        );
    }
}

fn paint_row_numbers(ui: &mut Ui<'_>, area: termrock::Rect, rows: usize, cursor: Option<usize>) {
    let Some(chrome) = GRID_CHROME.with(|chrome| chrome.borrow().clone()) else {
        return;
    };
    let digits = usize::from(chrome.gutter.saturating_sub(4)).max(1);
    let visible = usize::from(area.height.saturating_sub(1)).min(rows);
    let plain = ui.surface_style().remove_modifier(Modifier::BOLD);
    for row in 0..visible {
        let y = area.y.saturating_add(1).saturating_add(row as u16);
        let gutter = if cursor == Some(row) {
            plain.add_modifier(Modifier::BOLD)
        } else {
            plain
        };
        for dx in 1..3 {
            ui.paint_str(
                termrock::Rect {
                    x: area.x.saturating_add(dx),
                    y,
                    width: 1,
                    height: 1,
                },
                " ",
                gutter,
            );
        }
        let fade = visible.saturating_sub(row);
        let role = if fade == 1 {
            Role::Custom(termrock::Color::from_u32(0x003f_3f3f))
        } else if fade == 2 {
            Role::Custom(termrock::Color::from_u32(0x005c_5c5c))
        } else if cursor == Some(row) {
            TAG_PATH_FG
        } else {
            LEGACY_FAINT
        };
        let text = format!("{:>width$}", row.saturating_add(1), width = digits);
        ui.paint_str(
            termrock::Rect {
                x: area.x.saturating_add(3),
                y,
                width: digits as u16,
                height: 1,
            },
            &text,
            plain.patch(ui.paint_patch(&StylePatch::new().set_fg(role))),
        );
    }
}

fn paint_bottom_fade(ui: &mut Ui<'_>, area: termrock::Rect, model: &ResultGrid) {
    let Some(chrome) = GRID_CHROME.with(|chrome| chrome.borrow().clone()) else {
        return;
    };
    let visible = usize::from(area.height.saturating_sub(1)).min(model.row_count());
    if visible < 2 {
        return;
    }
    let steps = [
        (
            Role::Custom(termrock::Color::from_u32(0x00cc_cccc)),
            Role::Custom(termrock::Color::from_u32(0x008f_8f8f)),
            Role::Custom(termrock::Color::from_u32(0x006e_6e6e)),
        ),
        (
            Role::Custom(termrock::Color::from_u32(0x008c_8c8c)),
            Role::Custom(termrock::Color::from_u32(0x0062_6262)),
            Role::Custom(termrock::Color::from_u32(0x004c_4c4c)),
        ),
    ];
    for (step, (white, secondary, arrow)) in steps.into_iter().enumerate() {
        let row = visible.saturating_sub(2).saturating_add(step);
        let y = area.y.saturating_add(1).saturating_add(row as u16);
        let origin = area.x.saturating_add(chrome.gutter);
        let restyle = |ui: &mut Ui<'_>, x, width, role| {
            if width == 0 {
                return;
            }
            let style = ui
                .surface_style()
                .patch(ui.paint_patch(&StylePatch::new().set_fg(role)));
            ui.paint_style(
                termrock::Rect {
                    x,
                    y,
                    width,
                    height: 1,
                },
                style,
            );
        };
        if chrome.hidden_left == 0 {
            restyle(ui, origin, chrome.content_width.saturating_add(4), white);
        }
        restyle(ui, area.x.saturating_add(1), 2, white);
        if chrome.gutter > 0 {
            restyle(
                ui,
                area.x.saturating_add(chrome.gutter).saturating_sub(1),
                1,
                white,
            );
        }
        if chrome.hidden_left == 0 {
            for place in &chrome.places {
                if place.primary {
                    restyle(
                        ui,
                        origin.saturating_add(place.x),
                        place.width.saturating_sub(2),
                        secondary,
                    );
                }
            }
        }
        if chrome.hidden_left == 0
            && let Some(place) = chrome.places.last()
            && chrome.places.len() < chrome.columns
        {
            let arrow_x = origin
                .saturating_add(place.x)
                .saturating_add(place.width)
                .saturating_sub(1);
            restyle(ui, arrow_x, 1, arrow);
        }
    }
}

fn paint_horizontal_bar(ui: &mut Ui<'_>, row: termrock::Rect) {
    let Some(chrome) = GRID_CHROME.with(|chrome| chrome.borrow().clone()) else {
        return;
    };
    if chrome.content_width == 0 || chrome.columns == 0 {
        return;
    }
    let x = row.x.saturating_add(chrome.gutter);
    let track_w = chrome
        .content_width
        .saturating_add(4)
        .min(row.right().saturating_sub(x));
    if track_w == 0 {
        return;
    }
    let columns = chrome.columns.max(1);
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "columns is at least 1 and the product fits a terminal width"
    )]
    let thumb = (usize::from(track_w) * chrome.complete.max(1) / columns)
        .max(1)
        .min(usize::from(track_w));
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "columns is at least 1 and the product fits a terminal width"
    )]
    let thumb_at = if chrome.hidden_left == 0 {
        0
    } else {
        (usize::from(track_w) * chrome.hidden_left + columns / 2) / columns
    }
    .min(usize::from(track_w).saturating_sub(thumb));
    let track = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(LEGACY_SCROLL_TRACK)));
    let thumb_style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(LEGACY_SCROLL_THUMB)));
    ui.paint_str(
        termrock::Rect {
            x,
            y: row.y,
            width: track_w,
            height: 1,
        },
        &"┄".repeat(usize::from(track_w)),
        track,
    );
    ui.paint_str(
        termrock::Rect {
            x: x.saturating_add(thumb_at as u16),
            y: row.y,
            width: thumb as u16,
            height: 1,
        },
        &"━".repeat(thumb),
        thumb_style,
    );
}

fn draw_painted_grid(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    id: Id,
    columns: &[(String, ColType)],
    specs: &[termrock::Column<'_>],
    state: &termrock::GridState,
    model: &ResultGrid,
) {
    install_grid_budgets(area.width, columns, model);
    let scrolled = install_scrolled_budgets(area.width, columns, model, state);
    let grid = result_grid(id, specs);
    grid.draw(ui, area, state, &PaintedGrid { inner: model });
    paint_grid_focus_bar(ui, area, state, model);
    if let Some((key, _)) = state.cursor()
        && let Some(row) = (0..model.row_count()).find(|row| GridModel::row_key(model, *row) == key)
    {
        let bold = PaintStyle::new().add_modifier(Modifier::BOLD);
        ui.paint_style(
            termrock::Rect {
                x: area.x.saturating_add(1),
                y: area.y.saturating_add(1).saturating_add(row as u16),
                width: area.width.saturating_sub(1),
                height: 1,
            },
            bold,
        );
    }
    GRID_SCROLL.with(|slot| *slot.borrow_mut() = Some((area, model.row_count())));
    paint_row_numbers(
        ui,
        area,
        model.row_count(),
        state.cursor().and_then(|(key, _)| {
            (0..model.row_count()).find(|row| GridModel::row_key(model, *row) == key)
        }),
    );
    paint_column_gap(ui, area);
    paint_bottom_fade(ui, area, model);
    paint_grid_header(
        ui,
        area,
        &columns
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>(),
    );
    if scrolled {
        paint_scrolled_cells(ui, area, columns, state, model);
    }
}
fn result_grid<'a>(id: Id, columns: &'a [termrock::Column<'a>]) -> Grid<'a> {
    let grid = Grid::new(id, columns)
        .blur(termrock::BlurPolicy::Keep)
        .nav(termrock::NavUnit::Cell)
        .select_mode(termrock::SelectMode::Multi)
        .gutter(GridGutter::Detailed {
            row_numbers: true,
            min_digits: 1,
        })
        .column_fit(GridColumnFit::CompleteWithPreview { min_width: 4 })
        .column_gap(2)
        .right_reserve(3)
        .sort_indicator(GridSortIndicator::ActiveOnly)
        .patch_part(&GRID_PART_PATCH)
        .slot(Part::GUTTER, &paint_quiet_gutter)
        .cell(&paint_result_cell);
    if columns.first().is_some_and(|column| column.title == "id") {
        grid.header_prefixes(&ID_HEADER_PREFIX)
    } else {
        grid
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

fn paint_legacy_filter(ui: &mut Ui<'_>, area: termrock::Rect, text: &str, ink: Role) {
    if area.is_empty() {
        return;
    }
    let field = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_bg(Role::Surface(termrock::Surface::Field))));
    ui.fill(area, field);

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
        let label = field.patch(ui.paint_patch(&StylePatch::new().set_fg(ink)));
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

fn paint_panel_tail(ui: &mut Ui<'_>, area: termrock::Rect, ink: Role) {
    if area.width < 2 || area.height == 0 {
        return;
    }
    let style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(ink)));
    ui.paint_str(
        termrock::Rect {
            x: area.right().saturating_sub(2),
            y: area.y,
            width: 1,
            height: 1,
        },
        "─",
        style,
    );
}

fn paint_explorer_title_space(ui: &mut Ui<'_>, area: termrock::Rect) {
    let style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().add(Modifier::BOLD)));
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(2).saturating_add(width(" Explorer")),
            y: area.y,
            width: 1,
            height: 1,
        },
        " ",
        style,
    );
}

fn paint_explorer_title_gap(ui: &mut Ui<'_>, area: termrock::Rect) {
    let style = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Secondary))
                .remove(Modifier::BOLD),
        ),
    );
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(2).saturating_add(width(" Explorer")),
            y: area.y,
            width: 1,
            height: 1,
        },
        " ",
        style,
    );
}

fn paint_explorer_meta_band(ui: &mut Ui<'_>, area: termrock::Rect) {
    let inset = ui.design().space.frame_inset;
    let style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG).remove(Modifier::BOLD)));
    let width = area.width.saturating_sub(inset.saturating_mul(2));
    if width == 0 || area.height < 2 {
        return;
    }
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(inset).saturating_add(1),
            y: area.y.saturating_add(1),
            width,
            height: 1,
        },
        &" ".repeat(usize::from(width)),
        style,
    );
}

fn paint_explorer_chevron_gap(ui: &mut Ui<'_>, area: termrock::Rect) {
    let mark = area.right().saturating_sub(4);
    if mark == 0 {
        return;
    }
    let style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG).remove(Modifier::BOLD)));
    for x in [mark.saturating_sub(1), mark.saturating_add(1)] {
        ui.paint_str(
            termrock::Rect {
                x,
                y: area.y,
                width: 1,
                height: 1,
            },
            " ",
            style,
        );
    }
}

fn paint_explorer_indent(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    state: &TreeState,
    nodes: &[ExplorerNode],
    focused: bool,
) {
    let rows = visible_explorer_count(nodes, state).min(usize::from(area.height));
    let skip = focused
        .then(|| state.cursor())
        .flatten()
        .and_then(|key| visible_explorer_index(nodes, state, key));
    let base = ui.surface_style();
    let quiet = base.with_fg_from_bg(base);
    for offset in 0..rows {
        if Some(offset) == skip {
            continue;
        }
        ui.paint_str(
            termrock::Rect {
                x: area.x,
                y: area.y.saturating_add(offset as u16),
                width: 1,
                height: 1,
            },
            " ",
            quiet,
        );
    }
}

fn paint_focused_schema_mark(
    ui: &mut Ui<'_>,
    panel: termrock::Rect,
    area: termrock::Rect,
    state: &TreeState,
    nodes: &[ExplorerNode],
) {
    let Some(key) = state.cursor() else {
        return;
    };
    let Some(ExplorerNode::Schema { .. }) =
        nodes.iter().find(|node| explorer_node_key(node) == key)
    else {
        return;
    };
    let Some(index) = visible_explorer_index(nodes, state, key) else {
        return;
    };
    let y = area.y.saturating_add(index as u16);
    if y >= area.bottom() {
        return;
    }
    let prefix = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_CHROME_FG)
                .remove(Modifier::BOLD),
        ),
    );
    let indent = ui.design().space.tree_indent;
    let x = area
        .x
        .saturating_add(1)
        .saturating_add(indent)
        .saturating_add(2);
    ui.paint_str(
        termrock::Rect {
            x,
            y,
            width: 1,
            height: 1,
        },
        "S",
        prefix,
    );
    let bold = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().add(Modifier::BOLD)));
    ui.paint_str(
        termrock::Rect {
            x: panel.right().saturating_sub(3),
            y,
            width: 1,
            height: 1,
        },
        " ",
        bold,
    );
}

fn paint_idle_query_tail(ui: &mut Ui<'_>, area: termrock::Rect) {
    let editor = idle_query_top(area.height.saturating_sub(2)).saturating_sub(2);
    let x = area.right().saturating_sub(3);
    let y0 = area.y.saturating_add(2);
    let field = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(TAG_FIELD_BG),
        ),
    );
    for row in 0..editor {
        ui.paint_str(
            termrock::Rect {
                x,
                y: y0.saturating_add(row),
                width: 1,
                height: 1,
            },
            " ",
            field,
        );
    }
    let rule = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::BorderStrong)));
    ui.paint_str(
        termrock::Rect {
            x,
            y: y0.saturating_add(editor),
            width: 1,
            height: 1,
        },
        "─",
        rule,
    );
}

fn visible_explorer_count(nodes: &[ExplorerNode], state: &TreeState) -> usize {
    let mut show = [false; 8];
    show[0] = true;
    let mut count = 0usize;
    for node in nodes {
        let depth = explorer_depth(node);
        if !show.get(depth).copied().unwrap_or(false) {
            continue;
        }
        count = count.saturating_add(1);
        let expanded = state.is_expanded(explorer_node_key(node));
        if let Some(child) = show.get_mut(depth.saturating_add(1)) {
            *child = expanded;
        }
        for deeper in show.iter_mut().skip(depth.saturating_add(2)) {
            *deeper = false;
        }
    }
    count
}

fn paint_column_gap(ui: &mut Ui<'_>, area: termrock::Rect) {
    let Some(chrome) = GRID_CHROME.with(|chrome| chrome.borrow().clone()) else {
        return;
    };
    if chrome.hidden_left > 0 {
        return;
    }
    let Some(place) = chrome.places.get(1) else {
        return;
    };
    let width = 5.min(place.width);
    if width == 0 {
        return;
    }
    let style = ui.surface_style();
    let rows = usize::from(area.height.saturating_sub(1));
    let x = area.x.saturating_add(chrome.gutter).saturating_add(place.x);
    for row in 0..rows {
        ui.paint_style(
            termrock::Rect {
                x,
                y: area.y.saturating_add(1).saturating_add(row as u16),
                width,
                height: 1,
            },
            style,
        );
    }
}

fn paint_explorer_outside(
    ui: &mut Ui<'_>,
    state: &TreeState,
    nodes: &[ExplorerNode],
    inner: termrock::Rect,
    body: termrock::Rect,
    tree_area: termrock::Rect,
) {
    let field = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_bg(Role::Surface(termrock::Surface::Field))));
    ui.paint_str(
        termrock::Rect {
            x: body.right(),
            y: inner.y.saturating_add(1),
            width: 1,
            height: 1,
        },
        " ",
        field,
    );
    let Some(key) = state.chosen() else {
        return;
    };
    let Some(index) = visible_explorer_index(nodes, state, key) else {
        return;
    };
    let y = tree_area.y.saturating_add(index as u16);
    if y >= tree_area.bottom() {
        return;
    }
    let fill = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_bg(LEGACY_SELECT_BG)
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
    );
    ui.paint_str(
        termrock::Rect {
            x: tree_area.right(),
            y,
            width: 1,
            height: 1,
        },
        " ",
        fill,
    );
}

fn paint_explorer_collapse(ui: &mut Ui<'_>, area: termrock::Rect) {
    if area.width < 6 || area.height == 0 {
        return;
    }
    let style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    ui.paint_str(
        termrock::Rect {
            x: area.right().saturating_sub(4),
            y: area.y,
            width: 1,
            height: 1,
        },
        "‹",
        style,
    );
}

const STRUCTURE_SECTIONS: &[&str] = &[
    "Columns",
    "Indexes",
    "Foreign keys",
    "Constraints",
    "Triggers",
    "DDL",
];

const STRUCTURE_FIELDS: &[(&str, u16)] = &[
    ("Name", 20),
    ("Type", 14),
    ("Nullable", 8),
    ("Default", 22),
    ("Key", 6),
];

fn paint_structure_view(ui: &mut Ui<'_>, inner: termrock::Rect, table: &db::Table) {
    paint_structure_mode(ui, inner);
    if inner.height >= 5 {
        paint_structure_sections(ui, inner);
    }
    if inner.height >= 8 {
        paint_structure_columns(ui, inner, table);
    }
}

fn paint_structure_mode(ui: &mut Ui<'_>, inner: termrock::Rect) {
    if inner.height < 2 || inner.width == 0 {
        return;
    }
    let rest = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    let chip = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_bg(Role::Surface(termrock::Surface::Overlay))
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
    );
    let plain = ui.surface_style().remove_modifier(Modifier::BOLD);
    ui.paint_str(
        termrock::Rect {
            x: inner.x.saturating_add(1),
            width: inner.width.saturating_sub(1),
            height: 1,
            ..inner
        },
        "Data    Structure  ",
        rest,
    );
    ui.paint_str(
        termrock::Rect {
            width: 1.min(inner.width),
            height: 1,
            ..inner
        },
        " ",
        rest,
    );
    ui.paint_str(
        termrock::Rect {
            x: inner.x.saturating_add(7),
            width: 1,
            height: 1,
            ..inner
        },
        " ",
        plain,
    );
    let structure_x = inner.x.saturating_add(8);
    if structure_x < inner.right() {
        ui.paint_str(
            termrock::Rect {
                x: structure_x,
                width: 12.min(inner.right().saturating_sub(structure_x)),
                height: 1,
                ..inner
            },
            " Structure  ",
            chip,
        );
    }
    let rule = termrock::Rect {
        y: inner.y.saturating_add(1),
        height: 1,
        ..inner
    };
    let active = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
    let quiet = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    if rule.width > 0 {
        ui.paint_str(rule, &"─".repeat(usize::from(rule.width)), quiet);
    }
    if rule.width > 8 {
        ui.paint_str(
            termrock::Rect {
                x: rule.x.saturating_add(8),
                width: 12.min(rule.width.saturating_sub(8)),
                ..rule
            },
            "━━━━━━━━━━━━",
            active,
        );
    }
}

fn paint_structure_sections(ui: &mut Ui<'_>, inner: termrock::Rect) {
    let row = termrock::Rect {
        y: inner.y.saturating_add(3),
        height: 1,
        ..inner
    };
    let rest = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    let chip = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_bg(Role::Surface(termrock::Surface::Overlay))
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
    );
    let mut x = row.x;
    let mut active_width = 0u16;
    for (index, label) in STRUCTURE_SECTIONS.iter().enumerate() {
        if x >= row.right() {
            break;
        }
        let text = format!(" {label}  ");
        let text_width = width(&text);
        let slot = text_width.min(row.right().saturating_sub(x));
        if index == 0 {
            active_width = slot;
        }
        ui.paint_str(
            termrock::Rect {
                x,
                y: row.y,
                width: slot,
                height: 1,
            },
            &text,
            if index == 0 { chip } else { rest },
        );
        x = x.saturating_add(slot).saturating_add(1);
    }
    let rule = termrock::Rect {
        y: row.y.saturating_add(1),
        height: 1,
        ..inner
    };
    if rule.y >= inner.bottom() || rule.width == 0 {
        return;
    }
    let active = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
    let quiet = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    ui.paint_str(rule, &"─".repeat(usize::from(rule.width)), quiet);
    if active_width > 0 {
        ui.paint_str(
            termrock::Rect {
                width: active_width.min(rule.width),
                ..rule
            },
            &"━".repeat(usize::from(active_width.min(rule.width))),
            active,
        );
    }
}

fn paint_structure_columns(ui: &mut Ui<'_>, inner: termrock::Rect, table: &db::Table) {
    let header_y = inner.y.saturating_add(6);
    let status_y = inner.bottom().saturating_sub(1);
    if header_y >= status_y {
        return;
    }
    let header = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
    paint_structure_header(ui, inner, header_y, header);
    paint_structure_rows(ui, inner, table, header_y, status_y);
    let count = table.columns.len();
    let status = format!("{count} columns · read from the catalog · changes are queued until Save");
    ui.paint_str(
        termrock::Rect {
            x: inner.x.saturating_add(1),
            y: status_y,
            width: inner.width.saturating_sub(1),
            height: 1,
        },
        &status,
        header,
    );
}

fn paint_structure_header(ui: &mut Ui<'_>, inner: termrock::Rect, y: u16, style: PaintStyle) {
    let mut x = inner.x.saturating_add(3);
    let mut fields = STRUCTURE_FIELDS.iter().peekable();
    while let Some((title, slot)) = fields.next() {
        if x >= inner.right() {
            break;
        }
        let slot = (*slot).min(inner.right().saturating_sub(x));
        ui.paint_str(
            termrock::Rect {
                x,
                y,
                width: slot,
                height: 1,
            },
            &fit_cols(title, slot),
            style,
        );
        x = x.saturating_add(slot);
        if fields.peek().is_some() {
            x = x.saturating_add(2);
        }
    }
}

fn paint_structure_rows(
    ui: &mut Ui<'_>,
    inner: termrock::Rect,
    table: &db::Table,
    header_y: u16,
    status_y: u16,
) {
    let plain = ui.surface_style();
    let bold = plain.add_modifier(Modifier::BOLD);
    let mut y = header_y.saturating_add(1);
    for (index, column) in table.columns.iter().enumerate() {
        if y >= status_y {
            break;
        }
        let selected = index == 0;
        paint_structure_gutter(ui, inner.x, y, selected);
        ui.paint_str(
            termrock::Rect {
                x: inner.x.saturating_add(1),
                y,
                width: 2.min(inner.width.saturating_sub(1)),
                height: 1,
            },
            "  ",
            if selected { bold } else { plain },
        );
        let values = structure_values(column);
        let mut x = inner.x.saturating_add(3);
        let mut fields = values.iter().zip(STRUCTURE_FIELDS).peekable();
        while let Some(((text, role), (_, slot))) = fields.next() {
            if x >= inner.right() {
                break;
            }
            let slot = (*slot).min(inner.right().saturating_sub(x));
            let mut patch = StylePatch::new().set_fg(*role);
            if selected {
                patch = patch.add(Modifier::BOLD);
            }
            let style = ui.surface_style().patch(ui.paint_patch(&patch));
            ui.paint_str(
                termrock::Rect {
                    x,
                    y,
                    width: slot,
                    height: 1,
                },
                &fit_cols(text, slot),
                style,
            );
            x = x.saturating_add(slot);
            if fields.peek().is_some() {
                let gap = 2.min(inner.right().saturating_sub(x));
                if gap > 0 {
                    ui.paint_str(
                        termrock::Rect {
                            x,
                            y,
                            width: gap,
                            height: 1,
                        },
                        "  ",
                        if selected { bold } else { plain },
                    );
                }
                x = x.saturating_add(2);
            }
        }
        if selected {
            let end = inner.right().saturating_add(1);
            if x < end {
                let span = end.saturating_sub(x);
                ui.paint_str(
                    termrock::Rect {
                        x,
                        y,
                        width: span,
                        height: 1,
                    },
                    &" ".repeat(usize::from(span)),
                    bold,
                );
            }
        }
        y = y.saturating_add(1);
    }
}

fn paint_structure_gutter(ui: &mut Ui<'_>, x: u16, y: u16, selected: bool) {
    if selected {
        let caret = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent).add(Modifier::BOLD)));
        ui.paint_str(termrock::Rect::new(x, y, 1, 1), "▎", caret);
        return;
    }
    let blank = ui.surface_style().patch(
        ui.paint_patch(&StylePatch::new().set_fg(Role::Custom(termrock::Color::from_u32(0)))),
    );
    ui.paint_str(termrock::Rect::new(x, y, 1, 1), " ", blank);
}

fn structure_values(column: &db::Column) -> [(String, Role); 5] {
    let key = if column.primary {
        "PK"
    } else if column.references.is_some() {
        "FK"
    } else {
        ""
    };
    let nullable_role = if column.nullable {
        TAG_CHROME_FG
    } else {
        TAG_PATH_FG
    };
    [
        (column.name.clone(), Role::Fg(FgStep::Primary)),
        (column.ty.sql().to_owned(), TAG_PATH_FG),
        (
            if column.nullable { "yes" } else { "no" }.to_owned(),
            nullable_role,
        ),
        (
            column.default.clone().unwrap_or_else(|| "—".to_owned()),
            TAG_CHROME_FG,
        ),
        (key.to_owned(), TAG_PATH_FG),
    ]
}

fn fit_cols(text: &str, cols: u16) -> String {
    let clipped = truncate(text, cols);
    let extra = usize::from(cols.saturating_sub(width(&clipped)));
    let mut out = clipped;
    out.extend(std::iter::repeat_n(' ', extra));
    out
}

fn paint_data_mode(ui: &mut Ui<'_>, inner: termrock::Rect) {
    if inner.height < 2 {
        return;
    }
    let rest = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    let chip = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_bg(Role::Surface(termrock::Surface::Overlay))
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
    );
    ui.paint_str(
        termrock::Rect {
            x: inner.x.saturating_add(1),
            width: inner.width.saturating_sub(1),
            height: 1,
            ..inner
        },
        "Data    Structure  ",
        rest,
    );
    ui.paint_str(
        termrock::Rect {
            width: 1.min(inner.width),
            height: 1,
            ..inner
        },
        " ",
        chip,
    );
    ui.paint_str(
        termrock::Rect {
            x: inner.x.saturating_add(1),
            width: 6.min(inner.width),
            height: 1,
            ..inner
        },
        "Data  ",
        chip,
    );
    ui.paint_str(
        termrock::Rect {
            x: inner.x.saturating_add(7),
            width: 1,
            height: 1,
            ..inner
        },
        " ",
        ui.surface_style().remove_modifier(Modifier::BOLD),
    );
    let rule = termrock::Rect {
        y: inner.y.saturating_add(1),
        height: 1,
        ..inner
    };
    let active = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
    let quiet = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    ui.paint_str(
        termrock::Rect {
            width: 7.min(rule.width),
            ..rule
        },
        "━━━━━━━",
        active,
    );
    if rule.width > 7 {
        ui.paint_str(
            termrock::Rect {
                x: rule.x.saturating_add(7),
                width: rule.width.saturating_sub(7),
                ..rule
            },
            &"─".repeat(usize::from(rule.width.saturating_sub(7))),
            quiet,
        );
    }
}

fn paint_content_seam(ui: &mut Ui<'_>, area: termrock::Rect, panel_border: Role) {
    if area.height < 3 {
        return;
    }
    let seam = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::BorderStrong)));
    let edge = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(panel_border)));
    for y in area.y..area.bottom() {
        if area.x > 0 {
            ui.paint_str(
                termrock::Rect {
                    x: area.x.saturating_sub(1),
                    y,
                    width: 1,
                    height: 1,
                },
                "│",
                seam,
            );
        }
        if y == area.y || y.saturating_add(1) == area.bottom() {
            continue;
        }
        ui.paint_str(
            termrock::Rect {
                x: area.x,
                y,
                width: 1,
                height: 1,
            },
            "│",
            edge,
        );
    }
}

fn paint_frame_title_tail(ui: &mut Ui<'_>, area: termrock::Rect, title: &str, ink: Role) {
    let x = area.x.saturating_add(2).saturating_add(width(title));
    if x >= area.right().saturating_sub(1) || area.height == 0 {
        return;
    }
    let style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(ink)));
    ui.paint_str(
        termrock::Rect {
            x,
            y: area.y,
            width: 1,
            height: 1,
        },
        "─",
        style,
    );
}

fn paint_detail_rule(ui: &mut Ui<'_>, card: termrock::Rect) {
    let width = card.width.saturating_sub(4);
    if card.height <= 2 || width == 0 {
        return;
    }
    let style = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_RULE_FG)
                .set_bg(Role::Surface(termrock::Surface::Surface)),
        ),
    );
    ui.paint_str(
        termrock::Rect {
            x: card.x.saturating_add(2),
            y: card.y.saturating_add(1),
            width,
            height: 1,
        },
        &"─".repeat(usize::from(width)),
        style,
    );
}

fn safe_mode_mark(mode: SafeMode) -> Span<'static> {
    match mode {
        SafeMode::Safe | SafeMode::SafeFull => Span::new(" ✓ safe ").role(TAG_OK_FG),
        _ => Span::new(mode.token()),
    }
}

fn grouped(n: usize) -> String {
    let raw = n.to_string();
    let mut out = String::new();
    for (index, ch) in raw.chars().rev().enumerate() {
        if index > 0 && index.is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn draw_header(ui: &mut Ui<'_>, area: termrock::Rect, app: &TableProApp) {
    let base = ui.surface_style();
    ui.fill(area, base);
    if app.screen == Screen::Connections {
        let saved = format!("{} saved", app.connections_screen.connections.len());
        if app.surface == Surface::Connections {
            ui.paint_spans(
                area,
                &[
                    Span::new("  "),
                    Span::new("▪").role(Role::Accent),
                    Span::new(" "),
                    Span::new(" TablePro ").bold(),
                    Span::new(" "),
                    Span::new("Connections").role(Role::Fg(FgStep::Secondary)),
                    Span::new("  "),
                    Span::new(&saved).role(TAG_CHROME_FG),
                ],
                base,
            );
        } else {
            let surface = format!(" · {}", app.surface.label());
            ui.paint_spans(
                area,
                &[
                    Span::new("  "),
                    Span::new("▪").role(Role::Accent),
                    Span::new(" "),
                    Span::new(" TablePro ").bold(),
                    Span::new(" "),
                    Span::new("Connections").role(Role::Fg(FgStep::Secondary)),
                    Span::new(&surface).role(Role::Fg(FgStep::Secondary)),
                    Span::new("  "),
                    Span::new(&saved).role(TAG_CHROME_FG),
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
        let path = format!(
            " {} ",
            qualified_label("", &app.connection.database, app.workbench.current_schema())
        );
        ui.paint_spans(
            area,
            &[
                Span::new("  "),
                Span::new("▪").role(Role::Accent),
                Span::new(" "),
                Span::new(" TablePro  ").bold(),
                Span::new(&app.connection.name).bold(),
                Span::new(" ").bold(),
                Span::new(" "),
                Span::new(glyph).role(LEGACY_DANGER_LABEL).bold(),
                Span::new(" ").role(LEGACY_DANGER_LABEL).bold(),
                Span::new(environment).role(LEGACY_DANGER_LABEL).bold(),
                Span::new(" "),
                Span::new(&path).role(TAG_PATH_FG),
                safe_mode_mark(app.safe_mode),
            ],
            base,
        );
    }

    let capability = ui.theme().capability.color.label();
    let dimensions = format!("{}×{}", area.width, ui.full().height);
    let right_width = width(capability)
        .saturating_add(3)
        .saturating_add(width(&dimensions))
        .saturating_add(1)
        .saturating_add(8);
    let right = termrock::Rect {
        x: area.right().saturating_sub(right_width).saturating_sub(1),
        width: right_width.saturating_add(1),
        ..area
    };
    ui.paint_spans(
        right,
        &[
            Span::new(capability).role(TAG_CHROME_FG),
            Span::new(" · ").role(TAG_CHROME_FG),
            Span::new(&dimensions).role(TAG_CHROME_FG),
            Span::new(" "),
            Span::new(" ? help ").role(TAG_CHROME_FG),
        ],
        base,
    );
}

fn connection_footer_spans() -> [Span<'static>; 36] {
    [
        Span::new(" "),
        Span::new("↑↓").bold(),
        Span::new(" "),
        Span::new("Move").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("Enter").bold(),
        Span::new(" "),
        Span::new("Connect").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("e").bold(),
        Span::new(" "),
        Span::new("Edit…").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("d").bold(),
        Span::new(" "),
        Span::new("Delete…").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("Alt+D").bold(),
        Span::new(" "),
        Span::new("Duplicate").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("/").bold(),
        Span::new(" "),
        Span::new("Filter").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("Ctrl+N").bold(),
        Span::new(" "),
        Span::new("New").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("Shift+F10").bold(),
        Span::new(" "),
        Span::new("Menu").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("Tab").bold(),
        Span::new(" "),
        Span::new("Next").role(TAG_CHROME_FG),
    ]
}

fn explorer_footer_spans() -> [Span<'static>; 28] {
    [
        Span::new(" "),
        Span::new("↑↓").bold(),
        Span::new(" "),
        Span::new("Move").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("Enter").bold(),
        Span::new(" "),
        Span::new("Open").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("→").bold(),
        Span::new(" "),
        Span::new("Expand").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("/").bold(),
        Span::new(" "),
        Span::new("Filter").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("Ctrl+O").bold(),
        Span::new(" "),
        Span::new("Quick open").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("Shift+F10").bold(),
        Span::new(" "),
        Span::new("Menu").role(TAG_CHROME_FG),
        Span::new("  "),
        Span::new("Tab").bold(),
        Span::new(" "),
        Span::new("Next").role(TAG_CHROME_FG),
    ]
}

fn with_footer_spans(
    app: &TableProApp,
    explorer_focused: bool,
    paint: impl FnOnce(&[Span<'static>]),
) {
    if app.screen == Screen::Connections {
        paint(&connection_footer_spans());
    } else if explorer_focused {
        paint(&explorer_footer_spans());
    } else {
        match app.workbench.active() {
            Some(Tab::Table(table)) if table.is_structure() => paint(&[
                Span::new(" "),
                Span::new("↑↓").bold(),
                Span::new(" "),
                Span::new("Move").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("Ctrl+D").bold(),
                Span::new(" "),
                Span::new("Data / Structure").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("Tab").bold(),
                Span::new(" "),
                Span::new("Next").role(TAG_CHROME_FG),
            ]),
            Some(Tab::Table(_)) => paint(&[
                Span::new(" "),
                Span::new("↑↓←→").bold(),
                Span::new(" "),
                Span::new("Cell").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("Enter").bold(),
                Span::new(" "),
                Span::new("Edit").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("Alt+D").bold(),
                Span::new(" "),
                Span::new("Duplicate row").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("s").bold(),
                Span::new(" "),
                Span::new("Sort").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("f").bold(),
                Span::new(" "),
                Span::new("Filter").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("Space").bold(),
                Span::new(" "),
                Span::new("Select row").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("…").role(LEGACY_FAINT),
            ]),
            Some(Tab::Query(_)) => paint(&[
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
            ]),
            Some(Tab::History(_)) => paint(&[
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
            ]),
            None => paint(&[
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
            ]),
        }
    }
}

fn draw_footer(ui: &mut Ui<'_>, area: termrock::Rect, app: &TableProApp) {
    let base = ui.surface_style();
    let explorer_focused = ui.state(EXPLORER).contains(StateFlags::FOCUSED);
    ui.fill(area, base);
    with_footer_spans(app, explorer_focused, |spans| {
        ui.paint_spans(area, spans, base);
    });
    if let Some(notice) = app.destructive_notice {
        let width = width(notice).min(area.width);
        let right = termrock::Rect {
            x: area.right().saturating_sub(width),
            width,
            ..area
        };
        ui.fill(right, base);
        ui.paint_str(right, notice, base);
    } else if let Some(sorted) = sort_footer(app) {
        let text_width = width(&sorted);
        let right = termrock::Rect {
            x: area.right().saturating_sub(text_width).saturating_sub(1),
            width: text_width,
            ..area
        };
        ui.paint_str(
            right,
            &sorted,
            base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG))),
        );
    } else if app.screen == Screen::Workbench {
        let checked =
            matches!(app.surface, Surface::TableGrid | Surface::StructureView) || explorer_focused;
        let prefix = if checked {
            "✓ Connected to "
        } else {
            "Connected to "
        };
        let mut right_text =
            String::with_capacity(prefix.len().saturating_add(app.connection.name.len()));
        right_text.push_str(prefix);
        right_text.push_str(&app.connection.name);
        let text_width = width(&right_text);
        let right = termrock::Rect {
            x: area.right().saturating_sub(text_width).saturating_sub(1),
            width: text_width,
            ..area
        };
        ui.paint_spans(
            right,
            &[Span::new(&right_text).role(if checked {
                TAG_OK_FG
            } else {
                Role::Fg(FgStep::Muted)
            })],
            base,
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
        let width = width(line);
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
    let value_width = area.width.saturating_sub(18).max(1);
    let label_style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
    let mut y = area.y;
    for (label, value, value_role) in properties {
        let value_style = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(*value_role)));
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
            LEGACY_DANGER_LABEL,
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
const ACTION_MONO: &[termrock::MonoRule] = &[
    (
        CONNECT_LABEL,
        StateFlags::empty(),
        StylePatch::new()
            .set_fg(Role::Surface(termrock::Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary)),
    ),
    (
        EDIT_LABEL,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        DUPLICATE_LABEL,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        DELETE_LABEL,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
];

fn action_paint(ui: &Ui<'_>, part: Part, background: Role, foreground: Role) -> PaintStyle {
    ui.style_defaults(
        CONNECTION_ACTIONS,
        termrock::Variant::DEFAULT,
        part,
        StateFlags::empty(),
        StyleDefaults::new(StylePatch::new().set_bg(background).set_fg(foreground))
            .mono(ACTION_MONO),
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
    let width = width(label).saturating_add(2);
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
        if cx.update_cause() == UpdateCause::Bootstrap
            && self.screen == Screen::Workbench
            && self.surface == Surface::QuickSwitcher
        {
            self.open_switcher(cx);
        }
        let switcher_was_open = cx.is_open(quick_switcher::ID);
        let modal_was_open = self.destructive_intent.is_some();
        let mut response = self.update_destructive_dialog(cx);
        response |= self.update_tab_controls(cx);
        if self.form_open || self.screen != Screen::Connections {
            response |= connection_tree()
                .update(cx, &mut self.connection_tree_state, &self.connection_nodes)
                .erase();
            for _ in cx.intents(CONNECTION_DETAILS) {}
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
        if switcher_was_open {
            return response;
        }

        let _ = Self::connections_panel("", None);
        let _ = Self::connection_details_panel("");
        let _ = Self::explorer_panel(self.workbench.schema_caption(), true);
        let _ = Self::content_panel("", None, true);
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
                c if c == INSERT_ROW || c == DELETE_ROW || c == DISCARD_ROWS => {
                    if self.active_row_action(cx) {
                        if c == INSERT_ROW {
                            self.insert_active_row();
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
                        && cx.state(id).contains(StateFlags::FOCUSED)
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
                c if c == OPEN => {
                    if self.screen == Screen::Workbench {
                        self.open_switcher(cx);
                        response |= Response::changed();
                    }
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
                c if c == SORT && self.sort_focused_column(cx) => {
                    response |= Response::changed();
                }
                _ => {}
            }
        }
        let form_was_open = self.form_open;
        if self.draft.is_some() {
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
        if form_was_open {
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
                self.request_connect(cx, self.connections_screen.selected);
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
            && let Some(ExplorerNode::Object { item, .. }) = self
                .explorer_nodes
                .iter()
                .find(|node| explorer_node_key(node) == *key)
        {
            let item = item.clone();
            if self.open_table(&item)
                && let Some(id) = self.result_id()
            {
                cx.focus(id);
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
                    }
                }
                TabsAction::Close(key) => {
                    if let Some(tab) = self.workbench.tabs().iter().find(|tab| tab_key(tab) == key)
                    {
                        self.request_close_tab(cx, tab.key());
                    }
                }
                TabsAction::New => self.new_query(""),
            }
        }
        response |= tabs_response.erase();
        if self.workbench.tabs().len() <= 3 {
            self.pin_tab_window();
        }
        if cx.update_cause() == UpdateCause::Bootstrap && self.screen == Screen::Workbench {
            cx.focus(EXPLORER);
        }
        if self.explorer_pin > 0
            && self.screen == Screen::Workbench
            && self.surface == Surface::WorkbenchDefault
        {
            cx.focus(EXPLORER);
            self.explorer_pin = self.explorer_pin.saturating_sub(1);
        }

        response
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        GRID_SCROLL.with(|slot| *slot.borrow_mut() = None);
        let shown_sort = self.grid_sort.and_then(|(tab, column, direction)| {
            if self.workbench.active_key()? != tab {
                return None;
            }
            usize::from(column)
                .checked_sub(1)
                .map(|index| (index, direction))
        });
        GRID_SORT.with(|slot| *slot.borrow_mut() = shown_sort);
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
                self.workbench.tabs(),
            );
            paint_legacy_tabs(
                ui,
                workbench_rows[0],
                self.workbench.tabs(),
                self.tabs_state.active(),
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
        ui.layer(quick_switcher::ID, |ui, area| {
            self.switcher
                .component()
                .draw(ui, area, &self.switcher.state, &self.switcher.items);
        });
        if let Some(intent) = self.destructive_intent.as_ref() {
            ui.layer(QUIT_DIALOG, |ui, area| {
                intent.dialog().draw(ui, area, &self.quit_state, |_, _| {});
            });
        }
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
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Motion {
    #[default]
    Full,
    Reduced,
    Paused,
}

impl Motion {
    pub(crate) const fn is_paused(self) -> bool {
        matches!(self, Self::Paused)
    }
}

pub fn run_with(theme: Theme, connect: Option<&str>) -> std::io::Result<()> {
    run_session(theme, connect, Motion::Full, 0)
}

pub(crate) fn run_session(
    theme: Theme,
    connect: Option<&str>,
    motion: Motion,
    frame: u64,
) -> std::io::Result<()> {
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
    app.set_motion(motion, frame);
    // Paused frames keep press flash on a frozen simulation clock. The tag held flash time at zero.
    termrock::run_with_feedback_clock(app, theme, feedback_clock(motion))
}

pub(crate) fn feedback_clock(motion: Motion) -> termrock::FeedbackClock {
    if motion.is_paused() {
        termrock::FeedbackClock::Simulation {
            initial: termrock::SimulationMoment::from_millis(0),
        }
    } else {
        termrock::FeedbackClock::Elapsed
    }
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
            DELETE_ROW,
            DISCARD_ROWS,
            QUIT,
            CANCEL_OR_QUIT,
            OPEN,
            NEW_QUERY,
            HISTORY,
            STRUCTURE,
            FORM,
            HELP,
            TAB_LIST,
            FILTER,
            SORT,
            PREVIEW,
            SAVE,
            EXPLAIN,
            CLEAR_QUERY,
            COMPLETE,
            PALETTE,
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
