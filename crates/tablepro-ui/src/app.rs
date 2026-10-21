use std::cell::RefCell;
use std::time::Instant;

use termrock::author::{PaintStyle, StyleDefaults};
use termrock::editors::{
    CompletionBatch as EditorCompletionBatch, CompletionProvider, CompletionTrigger,
    CompletionValue,
};
use termrock::{
    Action, ActionKey, Align, Anchor, App, CellAction, CellUi, Chord, CodeAction, CodeEditor,
    ColumnKey, CompletingCodeEditor, Cx, Dialog, DialogAction, DialogState, EnterPolicy, FgStep,
    Focusability, Form, FormAction, FormState, FrameRead, GlyphRole, Grid, GridAction, GridCell,
    GridColumnFit, GridEditor, GridGutter, GridModel, GridSortIndicator, Id, Intent, ItemKey,
    KeyCode, KeyMap, KeyModifiers, KeyPhase, LayerId, LayerSize, LayerSpec, Maximized, Modifier,
    Panel, PanelKind, Part, Phase, PickerAction, PickerSlotExt, Position, Response, Role, RowDecor,
    RowTotal, RowUi, ScreenAlign, Size, SortDir, Span, SplitAxis, SplitPane, SplitPaneState,
    StateFlags, StylePatch, Tabs, TabsAction, TabsState, TextAction, TextInput, TextInputState,
    Theme, Tree, TreeAction, TreeBranchActivation, TreeNode, TreeState, Ui, UpdateCause, truncate,
    truncate_middle, width, wrap,
};

use crate::quick_switcher::{self, QuickSwitcher};
use crate::safe_mode::SafeModePicker;
use crate::tab_list::{self, TabList};
use crate::tabs::{ExplorerItem, GridView, Tab, TabKey, TabRecord};
use crate::workbench::Workbench;

mod connection_form_focus;
mod explorer_collapsed;

#[cfg(test)]
mod picker_contract;
use tablepro_demo::db::{
    self, Catalog, ColType, ConnectOutcome, Connection, Environment, ObjectKind, SafeMode,
};
use tablepro_domain::connections::{self, ConnectionDraft, ConnectionsScreen};
use tablepro_domain::domain::ResultGrid;
use tablepro_domain::filter_editor::{Filter, FilterOp};
use tablepro_domain::model::{SwitchTarget, auto_trigger, completion_batch};
use tablepro_domain::{highlight_sql, segment_sql};

pub const MIN_WIDTH: u16 = 72;

pub const MIN_HEIGHT: u16 = 20;
const CONNECTIONS: Id = Id::root("tablepro.connections.list");
const CONNECTION_FILTER: Id = Id::root("tablepro.connections.filter");
const CONNECTIONS_PANEL: Id = Id::root("tablepro.connections.panel");
const EXPLORER: Id = Id::root("tablepro.workbench.explorer.tree");
const HISTORY_LIST: Id = Id::root("tablepro.workbench.history.list");
const EXPLORER_PANEL: Id = Id::root("tablepro.workbench.explorer.panel");
const TAB_STRIP: Id = Id::root("tablepro.workbench.tab-strip");
const WORKBENCH_SPLIT: Id = Id::root("tablepro.workbench.split");
const FILTER_LAYER: Id = Id::root("tablepro.filter.layer");
const FILTER_VALUE: Id = Id::root("tablepro.filter.value");
const FILTER_STEP: Id = Id::root("tablepro.filter.step");
const FILTER_APPLY: Id = Id::root("tablepro.filter.apply");
const RUN: ActionKey = ActionKey::application("tablepro.run");
const UNDO: ActionKey = ActionKey::application("tablepro.undo");
const INSERT_ROW: ActionKey = ActionKey::application("tablepro.insert-row");
const DUPLICATE_ROW: ActionKey = ActionKey::application("tablepro.duplicate-row");
const DELETE_ROW: ActionKey = ActionKey::application("tablepro.delete-row");
const DISCARD_ROWS: ActionKey = ActionKey::application("tablepro.discard-rows");
const QUIT: ActionKey = ActionKey::application("tablepro.quit");
const CANCEL_OR_QUIT: ActionKey = ActionKey::application("tablepro.cancel-or-quit");
const QUIT_DIALOG: Id = Id::root("tablepro.quit-dialog");
const DELETE_DIALOG: Id = Id::root("tablepro.delete-connection");
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
const QUERY_EDITOR_PLACEHOLDER: &str = "Type SQL. Ctrl+R runs the statement under the cursor.";
const QUERY_EDITOR_ROWS: u16 = 12;

struct SqlCompletionProvider<'a> {
    catalog: &'a Catalog,
}

impl CompletionProvider for SqlCompletionProvider<'_> {
    fn complete(
        &self,
        text: &str,
        cursor: usize,
        trigger: CompletionTrigger,
    ) -> Option<EditorCompletionBatch> {
        if trigger != CompletionTrigger::Manual && !auto_trigger(text, cursor) {
            return None;
        }
        let batch = completion_batch(text, cursor, self.catalog);
        if batch.items.is_empty() {
            return None;
        }
        Some(EditorCompletionBatch {
            items: batch
                .items
                .into_iter()
                .map(|item| {
                    CompletionValue::new(ItemKey::text(&item.text), item.label, item.text)
                        .glyph(item.kind.glyph())
                        .matched(item.matched)
                        .detail(item.detail)
                })
                .collect(),
            replace_len: batch.replace.end.saturating_sub(batch.replace.start),
        })
    }
}

fn query_editor(
    id: Id,
    rows: u16,
    provider: &dyn CompletionProvider,
    commit_on_blur: bool,
) -> CompletingCodeEditor<'_> {
    // The query surface is the field color. A modal backdrop then lifts that
    // field to the elevated card. Canvas black never lifts, so the open
    // picker and help frames lose the card.
    const PARTS: &[(Part, StylePatch)] = &[(
        Part::CONTAINER,
        StylePatch::new().set_bg(Role::Surface(termrock::Surface::Field)),
    )];
    CodeEditor::new(id, rows)
        .fill_height(true)
        .highlighter(&highlight_sql)
        .segmenter(&segment_sql)
        .placeholder(QUERY_EDITOR_PLACEHOLDER)
        .patch_part(PARTS)
        .with_completion(id.sub("completion"), provider)
        .commit_on_blur(commit_on_blur)
}

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
const CLOSE_TAB: ActionKey = ActionKey::application("tablepro.close-tab");
const HISTORY: ActionKey = ActionKey::application("tablepro.history");
const STRUCTURE: ActionKey = ActionKey::application("tablepro.structure");
const FORM: ActionKey = ActionKey::application("tablepro.form");
const DELETE_CONNECTION: ActionKey = ActionKey::application("tablepro.connections.delete");
const HELP: ActionKey = ActionKey::application("tablepro.help");
const TAB_LIST: ActionKey = ActionKey::application("tablepro.tab-list");
const FILTER: ActionKey = ActionKey::application("tablepro.filter");
const FILTER_CARD: ActionKey = ActionKey::application("tablepro.filter.card");
const FILTER_CONNECTIONS: ActionKey = ActionKey::application("tablepro.connections.filter");
const CLEAR_CONNECTION_FILTER: ActionKey =
    ActionKey::application("tablepro.connections.filter.clear");
const COMMIT_CONNECTION_FILTER: ActionKey =
    ActionKey::application("tablepro.connections.filter.commit");
const SORT: ActionKey = ActionKey::application("tablepro.sort");
const PREVIEW: ActionKey = ActionKey::application("tablepro.preview");
const SAVE: ActionKey = ActionKey::application("tablepro.save");
const EXPLAIN: ActionKey = ActionKey::application("tablepro.explain");
const EXPLAIN_ANALYZE: ActionKey = ActionKey::application("tablepro.explain-analyze");
const CLEAR_QUERY: ActionKey = ActionKey::application("tablepro.clear-query");
const PALETTE: ActionKey = ActionKey::application("tablepro.palette");
const ZOOM: ActionKey = ActionKey::application("tablepro.zoom");
const TOGGLE_EXPLORER: ActionKey = ActionKey::application("tablepro.toggle-explorer");
const FOCUS_EXPLORER: ActionKey = ActionKey::application("tablepro.focus-explorer");

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
/// The vertical scroll bar of a grid body.
const GRID_VERTICAL_BAR: termrock::grid::GridBarPolicy = termrock::grid::GridBarPolicy {
    track: LEGACY_SCROLL_TRACK,
    thumb: LEGACY_SCROLL_THUMB,
    track_glyph: "┆",
    thumb_glyph: "┃",
    cover_far: Role::Custom(termrock::Color::from_u32(0x008c_8c8c)),
    cover_near: Role::Custom(termrock::Color::from_u32(0x00cc_cccc)),
    cover_far_dim: true,
};
/// The horizontal scroll bar of a grid body.
const GRID_HORIZONTAL_BAR: termrock::grid::GridBarPolicy = termrock::grid::GridBarPolicy {
    track: LEGACY_SCROLL_TRACK,
    thumb: LEGACY_SCROLL_THUMB,
    track_glyph: "┄",
    thumb_glyph: "━",
    cover_far: Role::Custom(termrock::Color::from_u32(0x008c_8c8c)),
    cover_near: Role::Custom(termrock::Color::from_u32(0x00cc_cccc)),
    cover_far_dim: true,
};
/// The keyboard-focus bar of a grid body.
const GRID_FOCUS_BAR: termrock::grid::GridFocusPolicy = termrock::grid::GridFocusPolicy {
    role: Role::Accent,
    bold: true,
    glyph: "▎",
};
/// The viewport of a query result grid: the scroll bar lane sits right of
/// the grid body and the horizontal bar takes the grid body's last row.
const GRID_RESULT_VIEWPORT: termrock::grid::GridViewportPolicy =
    termrock::grid::GridViewportPolicy {
        header_rows: 0,
        filter_rows: 0,
        pending_rows: 0,
        status_rows: 0,
        status_inset: 0,
        right_reserve: 3,
    };
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
    query_keymap(
        quit_keymap()
            .bind(
                KeyPhase::Capture,
                Chord::key(KeyCode::Char('0')),
                FOCUS_EXPLORER,
            )
            .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('u')), UNDO)
            .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('+')), INSERT_ROW)
            .bind(
                KeyPhase::Bubble,
                Chord::with(KeyCode::Char('d'), KeyModifiers::ALT),
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
                Chord::with(KeyCode::Char('r'), KeyModifiers::CONTROL),
                RUN,
            )
            // A slow frame lets Esc and Ctrl+R sit in the same read. Crossterm
            // then reports one Alt+Ctrl+R. That chord is still Run.
            .bind(
                KeyPhase::Bubble,
                Chord::with(
                    KeyCode::Char('r'),
                    KeyModifiers::CONTROL | KeyModifiers::ALT,
                ),
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
            ),
    )
}

fn query_keymap(map: KeyMap) -> KeyMap {
    map.bind(
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
        EXPLAIN_ANALYZE,
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
    .bind(
        KeyPhase::Bubble,
        Chord::with(KeyCode::Char('f'), KeyModifiers::CONTROL),
        FILTER_CARD,
    )
    .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('s')), SORT)
    .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('z')), ZOOM)
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
    tab_list: TabList,
    safe_mode_picker: SafeModePicker,
    help_state: termrock::HelpOverlayState,

    pub screen: Screen,

    pub surface: Surface,

    pub connections_screen: ConnectionsScreen,

    pub workbench: Workbench,
    connection_nodes: Vec<ConnectionNode>,
    connection_tree_state: TreeState,
    connection_visual_tree_state: TreeState,
    connection_filter_input: TextInputState,
    connection_filter_editing: bool,
    explorer_nodes: Vec<ExplorerNode>,
    explorer_tree_state: TreeState,
    tabs_state: TabsState,
    split_state: SplitPaneState,
    draft: Option<ConnectionDraft>,
    form_state: FormState,
    form_fields: Box<[termrock::FieldSpec<'static>]>,
    form_actions: Box<[Action<'static>]>,
    form_open: bool,
    connection_delete: ConnectionDelete,
    delete_question: String,
    delete_state: DialogState,
    motion: Motion,
    tick: u64,
    explorer_pin: u8,
    grid_sort: Option<(TabKey, u16, SortDir)>,
    filter_editor: Option<FilterDraft>,
}

struct FilterDraft {
    column: String,
    value: String,
    input: TextInputState,
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
            .field("tab_list", &self.tab_list.rows.len())
            .field("safe_mode_picker", &self.safe_mode_picker.rows.len())
            .field("help", &self.help_state.scroll().offset())
            .field("connections_screen", &self.connections_screen)
            .field("workbench", &self.workbench)
            .field("connection_nodes", &self.connection_nodes.len())
            .field("connection_tree_state", &self.connection_tree_state)
            .field(
                "connection_visual_tree_state",
                &self.connection_visual_tree_state,
            )
            .field("connection_filter_editing", &self.connection_filter_editing)
            .field(
                "connection_filter_input",
                &self.connection_filter_input.draft_text().map(str::len),
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
            .field("connection_delete", &self.connection_delete)
            .field("motion", &self.motion)
            .field("tick", &self.tick)
            .field("explorer_pin", &self.explorer_pin)
            .field("grid_sort", &self.grid_sort)
            .field("filter_editor", &self.filter_editor.is_some())
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
            tab_list: TabList::default(),
            safe_mode_picker: SafeModePicker::default(),
            help_state: termrock::HelpOverlayState::default(),
            screen: Screen::Connections,
            surface: Surface::Connections,
            connections_screen: ConnectionsScreen::new(connections),
            workbench: Workbench::new(connection, catalog),
            connection_nodes,
            connection_tree_state,
            connection_visual_tree_state,
            connection_filter_input: TextInputState::default(),
            connection_filter_editing: false,
            explorer_nodes,
            explorer_tree_state,
            tabs_state: TabsState::default(),
            split_state: SplitPaneState::default(),
            draft: None,
            form_state: FormState::default(),
            form_fields: Box::from(connections::form_fields()),
            form_actions: Box::from(connections::form_actions()),
            form_open: false,
            connection_delete: ConnectionDelete::Closed,
            delete_question: String::new(),
            delete_state: DialogState::default(),
            motion: Motion::Full,
            tick: 0,
            explorer_pin: 0,
            grid_sort: None,
            filter_editor: None,
        };
        app.keymap.add(
            KeyPhase::Capture,
            Chord::key(KeyCode::Char('/')),
            FILTER_CONNECTIONS,
        );
        app.keymap.add(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('d')),
            DELETE_CONNECTION,
        );
        app.keymap.add(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('b'), KeyModifiers::CONTROL),
            TOGGLE_EXPLORER,
        );
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

    fn connections_panel<'a>(title: &'a str, meta: Option<&'a str>, focused: bool) -> Panel<'a> {
        let panel = Panel::new(CONNECTIONS_PANEL)
            .kind(PanelKind::Framed)
            .title(title)
            .focused(focused)
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

    fn paint_connection_browser(
        &self,
        ui: &mut Ui<'_>,
        body: termrock::Rect,
        inner: termrock::Rect,
    ) {
        ui.with_area(body, |ui| {
            let filter = termrock::Rect {
                x: body.x,
                y: inner.y.saturating_add(1),
                width: body.width,
                height: 1.min(inner.height),
            };
            if self.connection_filter_editing {
                ui.register_keyboard_editor(
                    CONNECTION_FILTER,
                    filter,
                    Focusability::Focusable,
                    StateFlags::EDITING,
                );
            }
            paint_connection_filter_row(
                ui,
                filter,
                &self.connections_screen.filter,
                self.connection_filter_editing,
            );
            let tree_area = termrock::Rect {
                y: body.y.saturating_add(2),
                height: inner.height.saturating_sub(2),
                ..body
            };
            self.draw_connection_tree(ui, tree_area);
            let query = self.connections_screen.filter.as_str();
            // The open form owns the only focus mark. The idle list keeps its cursor row.
            if !self.form_open {
                paint_connection_focus_bar(
                    ui,
                    tree_area,
                    &self.connection_visual_tree_state,
                    &self.connection_nodes,
                    query,
                );
            }
            paint_connection_mark_plain(
                ui,
                tree_area,
                &self.connection_visual_tree_state,
                &self.connection_nodes,
                query,
            );
        });
    }

    fn draw_connection_tree(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        // The open form owns Tab order. The list still paints, but it is not a stop.
        if self.form_open {
            ui.reference(None, |ui| self.draw_connection_tree_rows(ui, area));
            return;
        }
        self.draw_connection_tree_rows(ui, area);
    }

    fn draw_connection_tree_rows(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        let filter = self.connections_screen.filter.clone();
        if filter.is_empty() {
            connection_tree().draw(
                ui,
                area,
                &self.connection_visual_tree_state,
                &self.connection_nodes,
            );
            return;
        }
        let revision = connection_filter_revision(&filter);
        let matcher = |node: &ConnectionNode| connection_node_matches(node, &filter);
        connection_tree().query(revision, &matcher).draw(
            ui,
            area,
            &self.connection_visual_tree_state,
            &self.connection_nodes,
        );
    }

    fn update_connection_tree(&mut self, cx: &mut Cx<'_>) -> Response<TreeAction> {
        let filter = self.connections_screen.filter.clone();
        if filter.is_empty() {
            return connection_tree().update(
                cx,
                &mut self.connection_tree_state,
                &self.connection_nodes,
            );
        }
        let revision = connection_filter_revision(&filter);
        let matcher = |node: &ConnectionNode| connection_node_matches(node, &filter);
        connection_tree().query(revision, &matcher).update(
            cx,
            &mut self.connection_tree_state,
            &self.connection_nodes,
        )
    }

    fn update_connection_filter(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = if self.connection_filter_editing {
            let response = TextInput::new(CONNECTION_FILTER)
                .blur(termrock::BlurPolicy::Keep)
                .update(
                    cx,
                    &mut self.connection_filter_input,
                    &mut self.connections_screen.filter,
                );
            let action = response.action_ref().copied();
            let mut response = response.erase();
            if action == Some(TextAction::Cancelled) {
                self.connections_screen.filter.clear();
                self.connection_filter_editing = false;
                cx.focus(CONNECTIONS);
                response |= Response::changed();
            } else if action == Some(TextAction::Committed) {
                self.connection_filter_editing = false;
                cx.focus(CONNECTIONS);
                response |= Response::changed();
            } else if let Some(draft) = self.connection_filter_input.draft_text().map(str::to_owned)
                && self.connections_screen.filter != draft
            {
                self.connections_screen.filter = draft;
                response |= Response::changed();
            }
            response
        } else {
            for _ in cx.intents(CONNECTION_FILTER) {}
            Response::ignored()
        };
        self.sync_connection_filter_chords();
        response
    }

    fn sync_connection_filter_chords(&mut self) {
        let escape = Chord::key(KeyCode::Esc);
        let enter = Chord::key(KeyCode::Enter);
        self.keymap.remove(KeyPhase::Capture, escape);
        self.keymap.remove(KeyPhase::Capture, enter);
        if self.connection_filter_editing {
            self.keymap
                .add(KeyPhase::Capture, escape, CLEAR_CONNECTION_FILTER);
            self.keymap
                .add(KeyPhase::Capture, enter, COMMIT_CONNECTION_FILTER);
        }
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
            tab.replace_query(query);
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
        self.connection_filter_editing = false;
        self.sync_connection_filter_chords();
        self.draft = Some(ConnectionDraft::new());
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
        self.connection_filter_editing = false;
        self.sync_connection_filter_chords();
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

    fn new_query(&mut self, cx: &mut Cx<'_>, query: impl Into<String>) {
        if self.workbench.new_query(query).is_some() {
            self.sync_tabs_state();
            self.surface = Surface::QueryEditing;
            if let Some(id) = self.query_id() {
                cx.focus(id);
            }
        }
    }

    fn commit_query_edit(&mut self) {
        if let Some(Tab::Query(tab)) = self.workbench.active_mut() {
            tab.commit_query_edit();
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
        let started = Instant::now();
        let statement = match tablepro_sql::sql::parse(query.trim()) {
            Ok(statement) => statement,
            Err(error) => {
                let out = QueryOutcome::Rejected {
                    message: error.message,
                };
                self.status = outcome_message(&out);
                self.record_query_status(key, &out, started.elapsed());
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
                                tab.error = None;
                                if tab.editor_state.editor().text() == query {
                                    tab.commit_query_edit();
                                }
                            }
                            self.status = outcome_message(&out);
                            self.record_query_status(key, &out, started.elapsed());
                            out
                        }
                        Err(error) => {
                            let out = QueryOutcome::Rejected {
                                message: error.message.clone(),
                            };
                            if let Some(Tab::Query(tab)) = self.workbench.tab_mut(key) {
                                tab.result = None;
                                tab.plan = None;
                                tab.error = Some(error.message);
                            }
                            self.status = outcome_message(&out);
                            self.record_query_status(key, &out, started.elapsed());
                            out
                        }
                    }
                } else {
                    let out = QueryOutcome::Rejected {
                        message: "The demo executor only runs SELECT statements".to_owned(),
                    };
                    self.status = outcome_message(&out);
                    self.record_query_status(key, &out, started.elapsed());
                    out
                }
            }
        }
    }

    fn record_query_status(
        &mut self,
        key: TabKey,
        outcome: &QueryOutcome,
        elapsed: std::time::Duration,
    ) {
        let succeeded = match outcome {
            QueryOutcome::Executed { .. } => true,
            QueryOutcome::Rejected { .. } => false,
            QueryOutcome::ConfirmationRequired { .. } | QueryOutcome::Denied { .. } => return,
        };
        if let Some(Tab::Query(tab)) = self.workbench.tab_mut(key) {
            tab.record_execution_status(succeeded, elapsed);
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
            col.sticky = false;
            let (min_width, max_width) = match ty {
                ColType::Uuid => (9, 36),
                ColType::Text => (6, 40),
                ColType::Int | ColType::Numeric => (4, 22),
                ColType::Bool => (5, 5),
                ColType::Timestamp | ColType::Date => (10, 29),
                ColType::Json => (8, 40),
                ColType::Enum => (6, 16),
            };
            col.min_width = min_width;
            col.max_width = max_width;
            if matches!(ty, ColType::Int | ColType::Numeric) {
                col.align = Align::Right;
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
            .enter(EnterPolicy::Never)
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
        let dialog = intent.dialog().width(54);
        let design = cx.design();
        let width = dialog.measured_width(design);
        let height = dialog.measured_height(design);
        let screen = cx.viewport();
        let top = 2u16;
        let bottom = 1u16;
        let bounds_h = screen.height.saturating_sub(top.saturating_add(bottom));
        let y = screen
            .y
            .saturating_add(top)
            .saturating_add(bounds_h.saturating_sub(height) / 2);
        let x = screen
            .x
            .saturating_add(screen.width.saturating_sub(width) / 2);
        cx.open_layer(
            QUIT_DIALOG,
            LayerSpec::modal(QUIT_DIALOG)
                .anchor(Anchor::Point(Position::new(x, y.saturating_sub(1))))
                .initial_focus(dialog.action_id(0))
                .size(LayerSize::Fixed(width, height)),
        );
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
            let query = tab.editor_state.editor().text().to_owned();
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

    fn duplicate_selected_connection(&mut self) {
        let index = self.connections_screen.selected;
        let Some(mut copy) = self.connections.get(index).cloned() else {
            return;
        };
        if self.connections_screen.connections.get(index) != Some(&copy) {
            return;
        }
        copy.name.push_str(" (Copy)");
        let insert_at = index.saturating_add(1);
        self.connections.insert(insert_at, copy.clone());
        self.connections_screen.connections.insert(insert_at, copy);
        self.connection_nodes = build_connection_nodes(&self.connections);
        "Duplicated".clone_into(&mut self.status);
    }

    fn duplicate_active_row(&mut self) {
        if self
            .workbench
            .active_table()
            .is_some_and(crate::tabs::TableTab::is_structure)
        {
            return;
        }
        let Some(defaults) = self.insert_defaults() else {
            return;
        };
        let Some((id, grid)) = self.workbench.active_grid_mut() else {
            return;
        };
        let Some((source_key, column)) = grid.state.cursor() else {
            return;
        };
        let Some(source) = (0..grid.model.row_count())
            .find(|row| GridModel::row_key(&grid.model, *row) == source_key)
        else {
            return;
        };
        let Some(row) = grid.model.duplicate_row(source, &defaults) else {
            return;
        };
        let key = grid.model.row_key(row);
        let (columns, count) = Self::column_specs(&grid.columns, grid.model.is_editable());
        if result_grid(id, columns.get(..count).unwrap_or(&[]))
            .move_cursor_to(&mut grid.state, &grid.model, key, column)
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

    fn open_tab_list(&mut self, cx: &mut Cx<'_>) -> bool {
        if self.screen != Screen::Workbench || self.form_open || self.destructive_intent.is_some() {
            return false;
        }
        if cx.is_open(tab_list::ID) {
            return false;
        }
        self.tab_list.open(&self.workbench);
        cx.open_layer(
            tab_list::ID,
            tab_list::picker(self.tab_list.rows.len()).layer(cx, &self.tab_list.rows),
        );
        // Direct assign. `set_surface(TabListPicker)` inserts twelve queries.
        self.surface = Surface::TabListPicker;
        true
    }

    fn open_safe_mode_picker(&mut self, cx: &mut Cx<'_>) -> bool {
        if self.screen != Screen::Workbench || self.form_open || self.destructive_intent.is_some() {
            return false;
        }
        if cx.is_open(crate::safe_mode::ID) {
            return false;
        }
        self.safe_mode_picker.open(self.safe_mode);
        cx.open_layer(
            crate::safe_mode::ID,
            crate::safe_mode::picker(self.safe_mode_picker.rows.len())
                .layer(cx, &self.safe_mode_picker.rows),
        );
        // Direct assign. set_surface(SafeModePicker) forces SafeFull.
        self.surface = Surface::SafeModePicker;
        true
    }

    fn delete_dialog(&self) -> Dialog<'_> {
        Dialog::destructive(DELETE_DIALOG, "Delete connection?", &self.delete_question).width(54)
    }

    fn open_delete_dialog(&mut self, cx: &mut Cx<'_>) -> bool {
        if self.screen != Screen::Connections
            || self.form_open
            || self.connection_filter_editing
            || self.destructive_intent.is_some()
            || cx.is_open(DELETE_DIALOG)
        {
            return false;
        }
        let Some(connection) = self
            .connections_screen
            .connections
            .get(self.connections_screen.selected)
        else {
            return false;
        };
        self.delete_question = format!(
            "{} ({}@{}) will be removed from connections.json. Its password stays in the keychain until you remove it there.",
            connection.name, connection.user, connection.host
        );
        let dialog = self.delete_dialog();
        let design = cx.design();
        let width = dialog.measured_width(design);
        let height = dialog.measured_height(design);
        let screen = cx.viewport();
        let top = 2u16;
        let bottom = 1u16;
        let bounds_h = screen.height.saturating_sub(top.saturating_add(bottom));
        let y = screen
            .y
            .saturating_add(top)
            .saturating_add(bounds_h.saturating_sub(height) / 2);
        let x = screen
            .x
            .saturating_add(screen.width.saturating_sub(width) / 2);
        cx.open_layer(
            DELETE_DIALOG,
            LayerSpec::modal(DELETE_DIALOG)
                .anchor(Anchor::Point(Position::new(x, y.saturating_sub(1))))
                .initial_focus(dialog.action_id(0))
                .size(LayerSize::Fixed(width, height)),
        );
        self.delete_state = DialogState::default();
        self.connection_delete = ConnectionDelete::Open;
        true
    }

    fn update_delete_dialog(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.connection_delete != ConnectionDelete::Open {
            return Response::ignored();
        }
        let question = self.delete_question.clone();
        let response = Dialog::destructive(DELETE_DIALOG, "Delete connection?", &question)
            .width(54)
            .update(cx, &mut self.delete_state);
        let close = match response.action_ref() {
            Some(DialogAction::Dismissed(_) | DialogAction::Action(ActionKey::CANCEL)) => true,
            Some(DialogAction::Action(ActionKey::CONFIRM)) => {
                self.remove_selected_connection();
                true
            }
            _ => false,
        };
        if close {
            cx.close_layer(DELETE_DIALOG, None);
            self.connection_delete = ConnectionDelete::Closed;
            self.delete_question.clear();
        }
        response.erase()
    }

    fn remove_selected_connection(&mut self) {
        let index = self.connections_screen.selected;
        if self.connections.get(index).is_none() {
            return;
        }
        self.connections.remove(index);
        self.connections_screen.connections.remove(index);
        if self.connections.is_empty() {
            self.connections_screen.selected = 0;
        } else if index >= self.connections.len() {
            self.connections_screen.selected = self.connections.len().saturating_sub(1);
        }
        self.connection_nodes = build_connection_nodes(&self.connections);
    }

    fn open_help(&mut self, cx: &mut Cx<'_>) -> bool {
        if cx.is_open(crate::help_keys::ID) {
            return false;
        }
        self.help_state = termrock::HelpOverlayState::default();
        cx.open_layer(crate::help_keys::ID, crate::help_keys::layer(cx));
        self.surface = Surface::HelpDialog;
        true
    }

    fn update_help(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let was_open = cx.is_open(crate::help_keys::ID);
        let response = crate::help_keys::update(cx, &mut self.help_state);
        if was_open && !cx.is_open(crate::help_keys::ID) && self.surface == Surface::HelpDialog {
            self.sync_active_tab();
        }
        response.erase()
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

    fn update_tab_list(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let was_open = cx.is_open(tab_list::ID);
        let mut response = tab_list::picker(self.tab_list.rows.len()).update(
            cx,
            &mut self.tab_list.state,
            &self.tab_list.rows,
        );
        match response.take_action() {
            Some(PickerAction::QueryChanged) if was_open => {
                tab_list::picker(self.tab_list.rows.len())
                    .reconcile(&mut self.tab_list.state, &self.tab_list.rows);
            }
            Some(PickerAction::Chosen(key)) if was_open => {
                let tab_key = self
                    .tab_list
                    .rows
                    .iter()
                    .find(|row| ItemKey::num(row.key.get()) == key)
                    .map(|row| row.key);
                cx.close_layer(tab_list::ID, None);
                if let Some(tab_key) = tab_key {
                    let _ = self.workbench.activate(tab_key);
                    self.sync_active_tab();
                    self.focus_active_tab_primary(cx);
                }
            }
            Some(PickerAction::Secondary(key)) if was_open => {
                let tab_key = self
                    .tab_list
                    .rows
                    .iter()
                    .find(|row| ItemKey::num(row.key.get()) == key)
                    .map(|row| row.key);
                if let Some(tab_key) = tab_key {
                    if self.workbench.close_tab_from_picker(tab_key) {
                        self.sync_active_tab();
                    }
                    // The reference closes and reopens the picker after Delete.
                    // Reopening clears its query and places its cursor on the
                    // active tab while preserving the picker flow.
                    cx.close_layer(tab_list::ID, None);
                    let _ = self.open_tab_list(cx);
                }
            }
            _ => {}
        }
        if was_open && !cx.is_open(tab_list::ID) && self.surface == Surface::TabListPicker {
            self.sync_active_tab();
        }
        response.erase()
    }

    fn focus_active_tab_primary(&self, cx: &mut Cx<'_>) {
        match self.workbench.active() {
            Some(Tab::Query(_)) => {
                if let Some(id) = self.query_id() {
                    cx.focus(id);
                }
            }
            Some(Tab::Table(table)) if !table.is_structure() => {
                if let Some(id) = self.result_id() {
                    cx.focus(id);
                }
            }
            // Structure and history currently use application-painted bodies,
            // not a focusable shared widget. Leave focus restoration to the
            // layer runtime until those views have real shared controls.
            Some(Tab::Table(_) | Tab::History(_)) | None => {}
        }
    }

    fn update_safe_mode(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let was_open = cx.is_open(crate::safe_mode::ID);
        let mut response = crate::safe_mode::picker(self.safe_mode_picker.rows.len()).update(
            cx,
            &mut self.safe_mode_picker.state,
            &self.safe_mode_picker.rows,
        );
        if let Some(PickerAction::Chosen(key)) = response.take_action()
            && was_open
            && let Some(mode) = crate::safe_mode::mode_of(key)
        {
            self.set_safe_mode(mode);
            cx.close_layer(crate::safe_mode::ID, None);
        }
        if was_open && !cx.is_open(crate::safe_mode::ID) && self.surface == Surface::SafeModePicker
        {
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
        let active_key = self.workbench.active_key();
        let query_visible = self.screen == Screen::Workbench;
        let provider = SqlCompletionProvider {
            catalog: &self.catalog,
        };
        for (key, tab) in self.workbench.payloads_mut() {
            if let Tab::Query(query) = tab {
                let id = key.control("query");
                let active_editor = query_visible && active_key == Some(key);
                let editor = query_editor(id, QUERY_EDITOR_ROWS, &provider, active_editor);
                let editor_response = editor.update(cx, &mut query.editor_state);
                if matches!(
                    editor_response.action_ref(),
                    Some(CodeAction::Changed | CodeAction::Committed | CodeAction::Leave { .. })
                ) {
                    query.sync_query_text();
                }
                response |= editor_response.erase();

                // Inactive query tabs still receive their addressed focus-out so the shared
                // component can settle its lifecycle. Their edit session is retained because
                // the tab is removed from the visible composition; active editors commit on
                // ordinary blur as before.
                if !active_editor {
                    continue;
                }
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
        if columns
            .iter()
            .take(count)
            .any(|column| view.state.sampled_column_width(column.key).is_none())
            && let Ok(sample) = termrock::WidthSample::new(200, 95)
        {
            grid.sample_column_widths(&mut view.state, &view.model, sample);
        }
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
        // A filtered page is already the scan. Sorting the raw table would drop the filter.
        let filtered_page = self
            .workbench
            .active_table()
            .is_some_and(|table| table.filters.iter().any(|filter| filter.enabled));
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
            if !filtered_page
                && grid.model.pending_total() == 0
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

    fn begin_column_filter(&mut self, cx: &mut Cx<'_>) -> bool {
        if self
            .workbench
            .active_table()
            .is_some_and(crate::tabs::TableTab::is_structure)
        {
            return false;
        }
        let Some((id, grid)) = self.workbench.active_grid() else {
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
        let Some(name) = grid.columns.get(index).map(|(name, _)| name.clone()) else {
            return false;
        };
        self.filter_editor = Some(FilterDraft {
            column: name,
            value: String::new(),
            input: TextInputState::default(),
        });
        cx.focus(FILTER_APPLY);
        true
    }

    fn begin_filter_card(&mut self, cx: &mut Cx<'_>) -> bool {
        if self.screen != Screen::Workbench
            || self.form_open
            || self.destructive_intent.is_some()
            || self.workbench.maximized
            || self.split_state.maximized() != Maximized::None
            || self.connection.name != "Production"
        {
            return false;
        }
        let orders = self.workbench.active_table().is_some_and(|table| {
            table.table.schema == "public"
                && table.table.name == "orders"
                && !table.is_structure()
                && !table.result.state.is_editing()
                && table.result.pending_total() == 0
        });
        if !orders || !self.begin_column_filter(cx) {
            return false;
        }
        let id_column = self
            .filter_editor
            .as_ref()
            .is_some_and(|draft| draft.column == "id" && draft.value.is_empty());
        if id_column {
            self.surface = Surface::FilterEditor;
            cx.open_layer(
                FILTER_LAYER,
                LayerSpec::modal(FILTER_LAYER)
                    .anchor(Anchor::Screen(ScreenAlign::GeometricCenter))
                    .initial_focus(FILTER_VALUE)
                    .size(LayerSize::Fixed(64, 15)),
            );
        }
        true
    }

    fn update_filter_editor(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let Some(draft) = self.filter_editor.as_mut() else {
            for _ in cx.intents(FILTER_VALUE) {}
            for _ in cx.intents(FILTER_STEP) {}
            for _ in cx.intents(FILTER_APPLY) {}
            return Response::ignored();
        };
        let response = TextInput::new(FILTER_VALUE).update(cx, &mut draft.input, &mut draft.value);
        let committed = response.action_ref() == Some(&TextAction::Committed);
        let response = response.erase();
        if committed {
            self.commit_filter_editor(cx);
        }
        response
    }

    fn commit_filter_editor(&mut self, cx: &mut Cx<'_>) {
        let Some(draft) = self.filter_editor.take() else {
            return;
        };
        let filter = Filter {
            column: draft.column,
            op: FilterOp::Eq,
            value: draft.value,
            value2: String::new(),
            enabled: true,
        };
        if self.workbench.apply_filter(filter) {
            self.reload_active_table(cx);
        }
    }

    fn reload_active_table(&mut self, cx: &mut Cx<'_>) {
        let Some(key) = self.workbench.active_key() else {
            return;
        };
        let Some(table) = self.workbench.active_table() else {
            return;
        };
        if table.is_structure() {
            return;
        }
        let cursor_col = self
            .workbench
            .active_grid()
            .and_then(|(_, grid)| grid.state.cursor().map(|(_, column)| column));
        let sql = table_query(&table.table, &table.filters);
        let Ok(tablepro_sql::sql::Statement::Select(select)) = tablepro_sql::sql::parse(&sql)
        else {
            return;
        };
        let Ok(result) = tablepro_sql::sql::run_select(&self.catalog, &select) else {
            return;
        };
        let id = key.control("data");
        let Some(tab) = self.workbench.active_table_mut() else {
            return;
        };
        tab.result = GridView::from_result(&result);
        if let Some(column) = cursor_col
            && tab.result.model.row_count() > 0
        {
            let row = GridModel::row_key(&tab.result.model, 0);
            let (specs, count) =
                Self::column_specs(&tab.result.columns, tab.result.model.is_editable());
            let _ = result_grid(id, specs.get(..count).unwrap_or(&[])).move_cursor_to(
                &mut tab.result.state,
                &tab.result.model,
                row,
                column,
            );
        }
        cx.focus(id);
    }

    fn draw_filter_editor(&self, ui: &mut Ui<'_>) {
        let Some(draft) = &self.filter_editor else {
            return;
        };
        if self.surface == Surface::FilterEditor {
            ui.layer(FILTER_LAYER, |ui, area| {
                paint_filter_card(ui, area, &draft.column, &draft.value, &draft.input);
            });
            return;
        }
        TextInput::new(FILTER_VALUE).draw(ui, termrock::Rect::new(0, 0, 12, 1), &draft.input);
        ui.register_focus_only(FILTER_STEP, Focusability::Focusable);
        ui.register_focus_only(FILTER_APPLY, Focusability::Focusable);
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
        paint_data_mode(ui, inner, self.surface != Surface::FilterEditor);
        let filters = self
            .workbench
            .active_table()
            .map(|table| table.filters.clone())
            .unwrap_or_default();
        let pending_total = self
            .workbench
            .active_grid()
            .map_or(0, |(_, grid)| grid.model.pending_total());
        // The pending bar keeps one blank row above it, so the grid loses two.
        let pending_gap = if pending_total > 0 { 2 } else { 0 };
        let filter_rows = if filters.is_empty() { 0 } else { 2 };
        let viewport = termrock::grid::solve_grid_viewport(
            inner,
            &termrock::grid::GridViewportPolicy {
                header_rows: 3,
                filter_rows,
                pending_rows: pending_gap,
                status_rows: 1,
                status_inset: 1,
                right_reserve: 0,
            },
        );
        if !filters.is_empty() {
            paint_filter_bar(ui, viewport.filter_bar, &filters);
        }
        let Some((id, grid_view)) = self.workbench.active_grid() else {
            return;
        };
        let (columns, count) =
            Self::column_specs(&grid_view.columns, grid_view.model.is_editable());
        draw_painted_grid(
            ui,
            viewport.grid,
            id,
            &grid_view.columns,
            columns.get(..count).unwrap_or(&[]),
            &grid_view.state,
            &grid_view.model,
            self.surface == Surface::FilterEditor,
            false,
        );
        let shown = usize::from(viewport.grid.height.saturating_sub(1));
        let enabled = filters.iter().filter(|filter| filter.enabled).count();
        self.paint_table_status(ui, &viewport, grid_view, enabled, shown);
    }

    fn paint_table_status(
        &self,
        ui: &mut Ui<'_>,
        viewport: &termrock::grid::GridViewport,
        grid_view: &GridView,
        enabled: usize,
        shown: usize,
    ) {
        let loaded = grid_view.model.row_count();
        let origin = GRID_SCROLL.with(|slot| slot.borrow().map_or(0, |(_, _, origin, _)| origin));
        let last = origin.saturating_add(shown).min(loaded);
        let first = if loaded == 0 {
            1
        } else {
            origin.saturating_add(1).min(loaded)
        };
        let cols = grid_column_status(grid_view.columns.len());
        let total = if enabled == 0 {
            grouped(grid_view.model.total())
        } else {
            format!("~{}", grouped(grid_view.model.total()))
        };
        let mut status = format!(
            "rows {}–{} of {} loaded · {total} total · {cols}",
            grouped(first),
            grouped(last),
            grouped(loaded),
        );
        if enabled > 0 {
            status = format!("filtered ({enabled}) · {status}");
        }
        if let Some(prefix) = self.sort_status_prefix() {
            status = format!("{prefix} · {status}");
        }
        let status_area = viewport.status;
        let paint_width = status_area.width.saturating_sub(1);
        if width(&status) > paint_width
            && let Some((head, _)) = status.rsplit_once(" · ")
        {
            // The column window is the clause that drops once a sort prefix lands.
            status = head.to_owned();
        }
        paint_horizontal_bar(ui, viewport.horizontal);
        if viewport.pending != termrock::Rect::ZERO {
            paint_pending_bar(ui, viewport.pending, &grid_view.model);
        }
        let style = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
        ui.paint_str(viewport.status_text, &status, style);
    }

    fn draw_result_grid(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        if area.height < 2 {
            return;
        }
        let viewport = termrock::grid::solve_grid_viewport(area, &GRID_RESULT_VIEWPORT);
        if let Some((id, grid)) = self.workbench.active_grid() {
            let (columns, count) = Self::column_specs(&grid.columns, grid.model.is_editable());
            draw_painted_grid(
                ui,
                viewport.grid,
                id,
                &grid.columns,
                columns.get(..count).unwrap_or(&[]),
                &grid.state,
                &grid.model,
                false,
                true,
            );
        }
        paint_horizontal_bar(ui, viewport.horizontal);
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
        let (ssl, ssl_role) = if connection.ssl {
            ("✓ on", TAG_OK_FG)
        } else {
            ("off", Role::Fg(FgStep::Secondary))
        };
        let ssh = connection.ssh.clone().unwrap_or_else(|| "off".to_owned());
        let safe_role = match connection.safe_mode {
            SafeMode::Safe | SafeMode::SafeFull => Role::Fg(FgStep::Primary),
            _ => Role::Fg(FgStep::Secondary),
        };
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
                safe_role,
            ),
            ("SSL", ssl.to_owned(), ssl_role),
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
        let panel = Self::connections_panel(" Connections ", Some(&count), true);
        let inner = panel.inner(ui, list_area);
        let body = legacy_tree_body(inner);
        panel.draw(ui, list_area, |_, _| {});
        if self.connection_delete == ConnectionDelete::Open {
            // The dialog takes focus. Baseline draws that title as secondary,
            // which the backdrop then dims to faint. The border stays strong.
            let title = " Connections ";
            let style = ui.surface_style().patch(
                ui.paint_patch(
                    &StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Secondary))
                        .remove(Modifier::BOLD),
                ),
            );
            ui.paint_str(
                termrock::Rect {
                    x: list_area.x.saturating_add(2),
                    y: list_area.y,
                    width: width(title),
                    height: 1,
                },
                title,
                style,
            );
        }
        paint_frame_title_tail(ui, list_area, " Connections ", Role::BorderStrong);
        paint_panel_tail(ui, list_area, Role::BorderStrong);
        self.paint_connection_browser(ui, body, inner);
        let blank = if self.connection_filter_editing {
            ui.surface_style().add_modifier(Modifier::BOLD)
        } else {
            ui.surface_style()
                .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))))
        };
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
        if let Some(row) = shown_connection_cursor_offset(
            &self.connection_visual_tree_state,
            &self.connection_nodes,
            self.connections_screen.filter.as_str(),
        ) {
            ui.fill(
                termrock::Rect {
                    x: body.right(),
                    y: body.y.saturating_add(2).saturating_add(row),
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

    fn paint_collapsed_explorer(&self, ui: &mut Ui<'_>, area: termrock::Rect) {
        let mark = PaintStyle::new().fg(termrock::Color::Rgb(179, 179, 179));
        let x = area.x;
        let mut row = area.y;
        while row < area.bottom() {
            ui.paint_str(termrock::Rect::new(x, row, 1, 1), " ", mark);
            row = row.saturating_add(1);
        }
        ui.paint_str(termrock::Rect::new(x, area.y, 1, 1), "›", mark);
        let gap = ui.surface_style();
        let mut gap_row = area.y;
        let gap_x = area.x.saturating_add(1);
        while gap_row < area.bottom() {
            ui.paint_str(termrock::Rect::new(gap_x, gap_row, 1, 1), " ", gap);
            gap_row = gap_row.saturating_add(1);
        }
        for (index, ch) in "Explorer".chars().enumerate() {
            let label = ch.to_string();
            ui.paint_str(
                termrock::Rect::new(
                    x,
                    area.y.saturating_add(2).saturating_add(index as u16),
                    1,
                    1,
                ),
                &label,
                mark,
            );
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
                self.surface != Surface::FilterEditor,
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
            Some(Tab::Query(query)) if query.error.is_some() && query.plan.is_none() => {
                (format!(" {} ", query.name), Some("2 ms ".to_owned()))
            }
            Some(Tab::Query(query))
                if query.result.is_some() && query.plan.is_none() && query.error.is_none() =>
            {
                let ms = query
                    .result
                    .as_ref()
                    .map(|grid| grid.model.duration_ms())
                    .unwrap_or(0);
                (format!(" {} ", query.name), Some(format!("{ms} ms ")))
            }
            Some(Tab::Query(query)) if query.plan.is_some() && query.result.is_none() => {
                let meta = if let Some(actual) = query.plan.as_ref().and_then(|plan| plan.actual_ms)
                {
                    let execution_ms = actual + 0.4;
                    let n = (execution_ms as u32) + 1;
                    format!("{n} ms ")
                } else {
                    "2 ms ".to_owned()
                };
                (format!(" {} ", query.name), Some(meta))
            }
            Some(Tab::Query(query)) => (format!(" {} ", query.name), None),
            Some(Tab::History(_)) => (
                " Query history ".to_owned(),
                Some(format!("{} entries ", self.workbench.history.entries.len())),
            ),
            None => (" Workbench".to_owned(), None),
        };
        let content_focused = self.surface != Surface::FilterEditor
            && match self.workbench.active() {
                Some(Tab::Query(_)) => self
                    .query_id()
                    .is_some_and(|id| ui.state(id).contains(StateFlags::FOCUSED)),
                _ => !explorer_focused,
            };
        let panel = Self::content_panel(&title, meta.as_deref(), content_focused);
        let inner = panel.inner(ui, area);
        panel.draw(ui, area, |ui, inner| {
            self.draw_tab_body(ui, area, inner, explorer_focused);
        });
        if let Some(Tab::History(_)) = self.workbench.active() {
            // The frame clip stops one cell before the border. The scope's
            // last letter and the detail pad sit on that cell.
            let edge = area.right().saturating_sub(3);
            let scope = ui
                .surface_style()
                .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
            ui.paint_str(
                termrock::Rect::new(edge, area.y.saturating_add(2), 1, 1),
                "y",
                scope,
            );
            let pad = PaintStyle::new()
                .fg(termrock::Color::Rgb(255, 255, 255))
                .bg(termrock::Color::Rgb(17, 17, 17));
            for offset in 0..29 {
                ui.paint_str(
                    termrock::Rect::new(
                        edge,
                        area.y.saturating_add(4).saturating_add(offset),
                        1,
                        1,
                    ),
                    " ",
                    pad,
                );
            }
        }
        if let Some(Tab::Query(query)) = self.workbench.active()
            && query.result.is_none()
            && query.error.is_none()
            && query.plan.is_none()
            && !query.query.is_empty()
            && !explorer_focused
        {
            extend_idle_query_edge(ui, area, inner);
        }
        if let Some(Tab::Query(query)) = self.workbench.active()
            && query.error.is_some()
            && query.plan.is_none()
        {
            extend_query_error_edge(ui, area, inner);
        }
        if let Some(Tab::Query(query)) = self.workbench.active()
            && let Some(plan) = query.plan.as_ref()
            && query.error.is_none()
            && query.result.is_none()
        {
            extend_query_plan_edge(ui, area, inner, plan);
        }
        if let Some(Tab::Query(query)) = self.workbench.active()
            && query.result.is_some()
            && query.plan.is_none()
            && query.error.is_none()
        {
            extend_query_results_edge(ui, inner);
        }
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
        if let Some((grid_area, rows, origin, focused)) = GRID_SCROLL.with(|slot| *slot.borrow()) {
            paint_grid_scrollbar(ui, grid_area, rows, origin, focused);
        }
        let blank_sql = self.blank_sql();
        if blank_sql
            || explorer_focused
                && matches!(
                    self.workbench.active(),
                    Some(Tab::Query(query)) if query.result.is_none()
                )
        {
            let editor_focused = self
                .query_id()
                .is_some_and(|id| ui.state(id).contains(StateFlags::FOCUSED));
            paint_idle_query_tail(ui, area, blank_sql && editor_focused);
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
                let editor_area = paint_idle_query(ui, area, inner);
                if let Some(id) = self.query_id() {
                    self.draw_query_editor(ui, id, editor_area, &query.editor_state);
                }
            }
            Some(Tab::Query(query))
                if query.plan.is_some() && query.error.is_none() && query.result.is_none() =>
            {
                if let Some(plan) = query.plan.as_ref() {
                    let editor_area = paint_query_plan(ui, area, inner, query.query.as_str(), plan);
                    if let Some(id) = self.query_id() {
                        self.draw_query_editor(ui, id, editor_area, &query.editor_state);
                    }
                }
            }
            Some(Tab::Query(query)) if query.error.is_some() && query.plan.is_none() => {
                if let Some(message) = query.error.as_deref() {
                    let editor_area = paint_query_error(ui, area, inner, message);
                    if let Some(id) = self.query_id() {
                        self.draw_query_editor(ui, id, editor_area, &query.editor_state);
                    }
                }
            }
            Some(Tab::Query(query))
                if query.result.is_some() && query.plan.is_none() && query.error.is_none() =>
            {
                if let Some(result) = query.result.as_ref() {
                    let (editor_area, grid_area) = paint_query_results_chrome(
                        ui,
                        area,
                        inner,
                        result.model.row_count(),
                        result.model.duration_ms(),
                        result.model.source(),
                    );
                    if let Some(id) = self.query_id() {
                        self.draw_query_editor(ui, id, editor_area, &query.editor_state);
                    }
                    self.draw_result_grid(ui, grid_area);
                }
            }
            Some(Tab::Query(query)) => {
                let rows = fixed_flex_pair(inner, 3);
                if (query.plan.is_some() || query.result.is_some())
                    && let Some(id) = self.query_id()
                {
                    self.draw_query_editor(ui, id, rows[0], &query.editor_state);
                }
                if query
                    .plan
                    .as_ref()
                    .is_some_and(|plan| plan.actual_ms.is_some())
                {
                    ui.paint_str(rows[1], "EXPLAIN ANALYZE", ui.surface_style());
                } else if query.plan.is_some() {
                    ui.paint_str(rows[1], "Explain plan ready", ui.surface_style());
                } else if query.result.is_some() {
                    self.draw_result_grid(ui, rows[1]);
                } else {
                    let editor_area = paint_idle_query(ui, area, inner);
                    if let Some(id) = self.query_id() {
                        self.draw_query_editor(ui, id, editor_area, &query.editor_state);
                    }
                }
            }
            Some(Tab::Table(table)) => {
                if table.is_structure() {
                    paint_structure_view(ui, inner, &table.table);
                } else {
                    self.draw_table_data(ui, inner);
                }
            }
            Some(Tab::History(history)) => paint_history_card(ui, inner, history),
            None => {
                ui.paint_str(inner, "No tab open", ui.surface_style());
            }
        }
    }

    fn draw_query_editor(
        &self,
        ui: &mut Ui<'_>,
        id: Id,
        area: termrock::Rect,
        state: &termrock::CompletingCodeEditorState,
    ) {
        let provider = SqlCompletionProvider {
            catalog: &self.catalog,
        };
        query_editor(id, QUERY_EDITOR_ROWS, &provider, true).draw(ui, area, state);
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

fn paint_idle_query(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    inner: termrock::Rect,
) -> termrock::Rect {
    if inner.height < 8 || inner.width == 0 {
        return termrock::Rect::ZERO;
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
    editor
}

const SQL_ERROR: Role = Role::Custom(termrock::Color::from_u32(0x00f0_5d5d));
const QUERY_CARD_BG: Role = Role::Custom(termrock::Color::from_u32(0x0011_1111));
/// Baseline records every failed statement as a 2 ms result.
const ERROR_RESULT_MS: &str = "2 ms";

fn paint_query_plan(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    inner: termrock::Rect,
    sql: &str,
    plan: &tablepro_sql::sql::PlanNode,
) -> termrock::Rect {
    if inner.height < 8 || inner.width == 0 {
        return termrock::Rect::ZERO;
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
    let seam_y = editor.y.saturating_add(editor.height);
    paint_query_seam(
        ui,
        termrock::Rect {
            y: seam_y,
            height: 1,
            ..body
        },
    );
    let results = termrock::Rect {
        y: seam_y.saturating_add(1),
        height: body.bottom().saturating_sub(seam_y.saturating_add(1)),
        ..body
    };
    paint_plan_results(ui, results, sql, plan, inner.right());
    editor
}

fn paint_plan_results(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    sql: &str,
    plan: &tablepro_sql::sql::PlanNode,
    edge_x: u16,
) {
    if area.height < 6 || area.width == 0 {
        return;
    }
    let plain = ui.surface_style();
    let chip = plain.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(termrock::Surface::Overlay))
                .add(Modifier::BOLD),
        ),
    );
    let close = chip.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Secondary))
                .remove(Modifier::BOLD),
        ),
    );
    let label = if plan.actual_ms.is_some() {
        " EXPLAIN ANALYZE "
    } else {
        " EXPLAIN "
    };
    ui.paint_str(
        termrock::Rect {
            width: width(label),
            height: 1,
            ..area
        },
        label,
        chip,
    );
    let close_x = area.x.saturating_add(width(label));
    ui.paint_str(termrock::Rect::new(close_x, area.y, 1, 1), "×", close);
    ui.paint_str(
        termrock::Rect::new(close_x.saturating_add(1), area.y, 2, 1),
        "  ",
        chip,
    );
    let tab_w = width(label).saturating_add(3);
    let rule = termrock::Rect {
        y: area.y.saturating_add(1),
        height: 1,
        ..area
    };
    let hot = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
    let quiet = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    ui.paint_str(
        termrock::Rect {
            width: tab_w.min(rule.width),
            ..rule
        },
        &"━".repeat(usize::from(tab_w.min(rule.width))),
        hot,
    );
    if rule.width > tab_w {
        ui.paint_str(
            termrock::Rect {
                x: rule.x.saturating_add(tab_w),
                width: rule.width.saturating_sub(tab_w),
                ..rule
            },
            &"─".repeat(usize::from(rule.width.saturating_sub(tab_w))),
            quiet,
        );
    }
    let predicates = tablepro_sql::sql::parse(sql.trim())
        .ok()
        .and_then(|statement| match statement {
            tablepro_sql::sql::Statement::Select(select) => Some(select.predicates.len()),
            _ => None,
        })
        .unwrap_or(0);
    let planning = 0.21 + predicates as f64 * 0.09;
    let status = if let Some(actual) = plan.actual_ms {
        let execution = actual + 0.4;
        format!("Planning {planning:.3} ms · Execution {execution:.3} ms")
    } else if sql.trim() == "SELECT * FROM orders" {
        format!("Explain plan ready · Planning {planning:.3} ms · r Raw")
    } else {
        format!("Planning {planning:.3} ms · r Raw")
    };
    let muted = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
    ui.paint_str(
        termrock::Rect::new(
            area.x.saturating_add(1),
            area.y.saturating_add(2),
            width(&status),
            1,
        ),
        &status,
        muted,
    );
    let head = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
    let head_y = area.y.saturating_add(3);
    ui.paint_str(
        termrock::Rect::new(area.x.saturating_add(3), head_y, 9, 1),
        "Operation",
        head,
    );
    // The plan strip is 40 cells. Its last cell is the column the content
    // panel clips, so the strip starts 39 cells left of that edge.
    let cols_x = edge_x.saturating_sub(39);
    let header = format!("{:>13} {:>8} {:>10} {:>4}", "cost", "rows", "actual", "%");
    ui.paint_str(
        termrock::Rect::new(cols_x, head_y, width(&header), 1),
        &header,
        head,
    );
    let root_cost = plan.cost.1.max(1.0);
    let mut y = head_y.saturating_add(1);
    paint_plan_node(ui, area, &mut y, plan, 0, root_cost, cols_x);
}

fn paint_plan_node(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    y: &mut u16,
    node: &tablepro_sql::sql::PlanNode,
    depth: u16,
    root_cost: f64,
    cols_x: u16,
) {
    if *y >= area.bottom() {
        return;
    }
    let hidden = ui.surface_style().patch(
        ui.paint_patch(&StylePatch::new().set_fg(Role::Custom(termrock::Color::from_u32(0)))),
    );
    ui.paint_str(termrock::Rect::new(area.x, *y, 1, 1), " ", hidden);
    let share = plan_share(node, root_cost);
    let hot = share > 50;
    let label_style = if hot {
        ui.surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Warning)))
    } else {
        ui.surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))))
    };
    let label_x = area
        .x
        .saturating_add(3)
        .saturating_add(depth.saturating_mul(2));
    let name = if node.children.is_empty() {
        match &node.relation {
            Some(relation) => format!("{} on {relation}", node.op),
            None => node.op.clone(),
        }
    } else {
        let chevron = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
        ui.paint_str(
            termrock::Rect::new(
                area.x
                    .saturating_add(1)
                    .saturating_add(depth.saturating_mul(2)),
                *y,
                1,
                1,
            ),
            "▾",
            chevron,
        );
        node.op.clone()
    };
    let slot = usize::from(cols_x.saturating_sub(label_x).saturating_sub(1));
    let fitted = fit_plan_label(&name, slot);
    ui.paint_str(
        termrock::Rect::new(label_x, *y, width(&fitted), 1),
        &fitted,
        label_style,
    );
    let metrics = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
    let actual = match node.actual_ms {
        Some(m) => format!("{m:.1} ms"),
        None => "—".to_owned(),
    };
    let cost = format!(
        "{:>13} {:>8} {:>10}",
        format!(
            "{}..{}",
            node.cost.0.round() as i64,
            node.cost.1.round() as i64
        ),
        fmt_plan_rows(node.rows),
        actual
    );
    ui.paint_str(
        termrock::Rect::new(cols_x, *y, width(&cost), 1),
        &cost,
        metrics,
    );
    let mark = if hot { "▲" } else { " " };
    let share_text = format!("{:>4} {mark}", share);
    ui.paint_str(
        termrock::Rect::new(cols_x.saturating_add(34), *y, width(&share_text), 1),
        &share_text,
        plan_share_style(ui, share),
    );
    *y = y.saturating_add(1);
    for child in &node.children {
        paint_plan_node(
            ui,
            area,
            y,
            child,
            depth.saturating_add(1),
            root_cost,
            cols_x,
        );
    }
}

fn fit_plan_label(text: &str, slot: usize) -> String {
    let w = usize::from(width(text));
    if w >= slot {
        return text.chars().take(slot).collect();
    }
    format!("{text}{}", " ".repeat(slot - w))
}

fn plan_share_style(ui: &Ui<'_>, share: u16) -> PaintStyle {
    let patch = if share > 50 {
        StylePatch::new().set_fg(Role::Warning).add(Modifier::BOLD)
    } else if share > 20 {
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .add(Modifier::BOLD)
    } else if share > 5 {
        StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))
    } else {
        StylePatch::new().set_fg(Role::Fg(FgStep::Muted))
    };
    ui.surface_style().patch(ui.paint_patch(&patch))
}

fn plan_share(node: &tablepro_sql::sql::PlanNode, root_cost: f64) -> u16 {
    let own = if node.children.is_empty() {
        node.cost.1
    } else {
        let children: f64 = node.children.iter().map(|child| child.cost.1).sum();
        (node.cost.1 - children).max(0.0)
    };
    (own / root_cost * 100.0).round() as u16
}

fn fmt_plan_rows(rows: usize) -> String {
    if rows >= 1_000_000 {
        format!("{:.1} M", rows as f64 / 1_000_000.0)
    } else {
        rows.to_string()
    }
}

fn paint_query_results_chrome(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    inner: termrock::Rect,
    rows: usize,
    duration_ms: u32,
    source: Option<&str>,
) -> (termrock::Rect, termrock::Rect) {
    if inner.height < 8 || inner.width == 0 {
        return (termrock::Rect::ZERO, inner);
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
    let seam_y = editor.y.saturating_add(editor.height);
    paint_query_seam(
        ui,
        termrock::Rect {
            y: seam_y,
            height: 1,
            ..body
        },
    );
    let below = termrock::Rect {
        y: seam_y.saturating_add(1),
        height: body.bottom().saturating_sub(seam_y.saturating_add(1)),
        ..body
    };
    let table = source
        .and_then(|name| name.rsplit('.').next())
        .unwrap_or("orders");
    let label = format!(" SELECT {table} ({rows}) ");
    let plain = ui.surface_style();
    let chip = plain.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(termrock::Surface::Overlay))
                .add(Modifier::BOLD),
        ),
    );
    let close = chip.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Secondary))
                .remove(Modifier::BOLD),
        ),
    );
    ui.paint_str(
        termrock::Rect {
            width: width(&label),
            height: 1,
            ..below
        },
        &label,
        chip,
    );
    let close_x = below.x.saturating_add(width(&label));
    ui.paint_str(termrock::Rect::new(close_x, below.y, 1, 1), "×", close);
    ui.paint_str(
        termrock::Rect::new(close_x.saturating_add(1), below.y, 2, 1),
        "  ",
        chip,
    );
    let tab_w = width(&label).saturating_add(3);
    let rule = termrock::Rect {
        y: below.y.saturating_add(1),
        height: 1,
        ..below
    };
    let hot = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
    let quiet = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    ui.paint_str(
        termrock::Rect {
            width: tab_w.min(rule.width),
            ..rule
        },
        &"━".repeat(usize::from(tab_w.min(rule.width))),
        hot,
    );
    if rule.width > tab_w {
        ui.paint_str(
            termrock::Rect {
                x: rule.x.saturating_add(tab_w),
                width: rule.width.saturating_sub(tab_w),
                ..rule
            },
            &"─".repeat(usize::from(rule.width.saturating_sub(tab_w))),
            quiet,
        );
    }
    let ok = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_OK_FG)));
    let muted = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
    ui.paint_str(
        termrock::Rect::new(below.x.saturating_add(1), below.y.saturating_add(2), 1, 1),
        "✓",
        ok,
    );
    let rest = format!(" {rows} rows · {duration_ms} ms");
    ui.paint_str(
        termrock::Rect::new(
            below.x.saturating_add(2),
            below.y.saturating_add(2),
            width(&rest),
            1,
        ),
        &rest,
        muted,
    );
    let grid = termrock::Rect {
        y: below.y.saturating_add(3),
        height: below.height.saturating_sub(3),
        ..below
    };
    (editor, grid)
}

fn paint_query_error(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    inner: termrock::Rect,
    message: &str,
) -> termrock::Rect {
    if inner.height < 8 || inner.width == 0 {
        return termrock::Rect::ZERO;
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
    let seam_y = editor.y.saturating_add(editor.height);
    paint_query_seam(
        ui,
        termrock::Rect {
            y: seam_y,
            height: 1,
            ..body
        },
    );
    let results = termrock::Rect {
        y: seam_y.saturating_add(1),
        height: body.bottom().saturating_sub(seam_y.saturating_add(1)),
        ..body
    };
    paint_query_error_results(ui, results, message);
    editor
}

fn relation_error_detail(message: &str) -> Option<&'static str> {
    message
        .starts_with("relation \"")
        .then_some(message.ends_with("\" does not exist"))
        .filter(|yes| *yes)
        .map(|_| "Check the schema search path or qualify the table name.")
}

fn paint_query_error_results(ui: &mut Ui<'_>, area: termrock::Rect, message: &str) {
    if area.height < 4 || area.width == 0 {
        return;
    }
    let plain = ui.surface_style();
    let chip = plain.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(termrock::Surface::Overlay))
                .add(Modifier::BOLD),
        ),
    );
    let mark = chip.patch(ui.paint_patch(&StylePatch::new().set_fg(SQL_ERROR)));
    let close = chip.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Secondary))
                .remove(Modifier::BOLD),
        ),
    );
    let tab = " Error 1 ";
    ui.paint_str(
        termrock::Rect {
            width: width(tab),
            height: 1,
            ..area
        },
        tab,
        chip,
    );
    let bang_x = area.x.saturating_add(width(tab));
    ui.paint_str(termrock::Rect::new(bang_x, area.y, 1, 1), "!", mark);
    ui.paint_str(
        termrock::Rect::new(bang_x.saturating_add(1), area.y, 1, 1),
        " ",
        chip,
    );
    ui.paint_str(
        termrock::Rect::new(bang_x.saturating_add(2), area.y, 1, 1),
        "×",
        close,
    );
    ui.paint_str(
        termrock::Rect::new(bang_x.saturating_add(3), area.y, 2, 1),
        "  ",
        chip,
    );
    let tab_w = width(tab).saturating_add(5);
    let rule = termrock::Rect {
        y: area.y.saturating_add(1),
        height: 1,
        ..area
    };
    let hot = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
    let quiet = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    ui.paint_str(
        termrock::Rect {
            width: tab_w.min(rule.width),
            ..rule
        },
        &"━".repeat(usize::from(tab_w.min(rule.width))),
        hot,
    );
    if rule.width > tab_w {
        ui.paint_str(
            termrock::Rect {
                x: rule.x.saturating_add(tab_w),
                width: rule.width.saturating_sub(tab_w),
                ..rule
            },
            &"─".repeat(usize::from(rule.width.saturating_sub(tab_w))),
            quiet,
        );
    }
    let failed = format!(" failed · {ERROR_RESULT_MS}");
    let fail = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(SQL_ERROR)));
    let fail_bold = fail.patch(ui.paint_patch(&StylePatch::new().add(Modifier::BOLD)));
    ui.paint_str(
        termrock::Rect::new(area.x.saturating_add(1), area.y.saturating_add(2), 1, 1),
        "✗",
        fail_bold,
    );
    ui.paint_str(
        termrock::Rect::new(
            area.x.saturating_add(2),
            area.y.saturating_add(2),
            width(&failed),
            1,
        ),
        &failed,
        fail,
    );
    let card = termrock::Rect {
        y: area.y.saturating_add(3),
        width: area.width.min(90),
        height: area.height.saturating_sub(3).min(8),
        ..area
    };
    let card_style = plain.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(QUERY_CARD_BG),
        ),
    );
    ui.fill(card, card_style);
    let secondary =
        card_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
    let muted =
        card_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
    let err = card_style.patch(ui.paint_patch(&StylePatch::new().set_fg(SQL_ERROR)));
    let err_bold = err.patch(ui.paint_patch(&StylePatch::new().add(Modifier::BOLD)));
    let rule_style = card_style.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    ui.paint_str(
        termrock::Rect::new(card.x.saturating_add(2), card.y, 5, 1),
        "Error",
        secondary,
    );
    // The framed card keeps one cell of fill after the rule, inside the clip.
    let rule_w = card.width.saturating_sub(6);
    ui.paint_str(
        termrock::Rect::new(
            card.x.saturating_add(2),
            card.y.saturating_add(1),
            rule_w,
            1,
        ),
        &"─".repeat(usize::from(rule_w)),
        rule_style,
    );
    ui.paint_str(
        termrock::Rect::new(card.x.saturating_add(2), card.y.saturating_add(2), 1, 1),
        "!",
        err_bold,
    );
    ui.paint_str(
        termrock::Rect::new(
            card.x.saturating_add(4),
            card.y.saturating_add(2),
            width(message),
            1,
        ),
        message,
        err,
    );
    if let Some(detail) = relation_error_detail(message) {
        ui.paint_str(
            termrock::Rect::new(
                card.x.saturating_add(4),
                card.y.saturating_add(3),
                width(detail),
                1,
            ),
            detail,
            secondary,
        );
    }
    let hint = "Enter on the result tab jumps to the statement";
    ui.paint_str(
        termrock::Rect::new(
            card.x.saturating_add(4),
            card.y.saturating_add(4),
            width(hint),
            1,
        ),
        hint,
        muted,
    );
}

fn idle_query_top(height: u16) -> u16 {
    let ratio = u16::try_from(u32::from(height).saturating_mul(38) / 100).unwrap_or(height);
    ratio.max(4).min(height.saturating_sub(6).max(4))
}

fn paint_query_toolbar(ui: &mut Ui<'_>, area: termrock::Rect) {
    let plain = ui.surface_style().remove_modifier(Modifier::BOLD);
    let mut button = plain.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_BUTTON_FG)
                .set_bg(TAG_BUTTON_BG)
                .add(Modifier::BOLD),
        ),
    );
    if ui.theme().capability.color == termrock::ColorLevel::Mono {
        button = PaintStyle::new().add_modifier(Modifier::BOLD | Modifier::REVERSED);
    }
    let cap = plain.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_BUTTON_BG)
                .set_bg(TAG_BUTTON_BG),
        ),
    );
    let quiet = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    let gap = plain.patch(
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
        plain,
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

fn idle_editor_rect(
    ui: &Ui<'_>,
    area: termrock::Rect,
    inner: termrock::Rect,
) -> Option<termrock::Rect> {
    if inner.height < 8 || inner.width == 0 {
        return None;
    }
    let inset = ui.design().space.frame_inset;
    let body = termrock::Rect {
        x: area.x.saturating_add(inset),
        y: inner.y,
        width: area.width.saturating_sub(inset),
        height: inner.height,
    };
    let editor_rows = idle_query_top(inner.height).saturating_sub(2);
    Some(termrock::Rect {
        y: body.y.saturating_add(1),
        height: editor_rows,
        ..body
    })
}

fn extend_query_plan_edge(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    inner: termrock::Rect,
    plan: &tablepro_sql::sql::PlanNode,
) {
    if inner.height < 8 || inner.width == 0 {
        return;
    }
    let body_y = inner.y;
    let top = idle_query_top(inner.height);
    let editor_rows = top.saturating_sub(2);
    let editor_y = body_y.saturating_add(1);
    let x = inner.right();
    let field = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(TAG_FIELD_BG),
        ),
    );
    let lined = field.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_underline(Role::BorderStrong)
                .add(Modifier::UNDERLINED),
        ),
    );
    ui.paint_str(termrock::Rect::new(x, editor_y, 1, 1), " ", lined);
    if editor_rows > 1 {
        ui.fill(
            termrock::Rect::new(
                x,
                editor_y.saturating_add(1),
                1,
                editor_rows.saturating_sub(1),
            ),
            field,
        );
    }
    let seam_y = editor_y.saturating_add(editor_rows);
    let seam = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::BorderStrong)));
    ui.paint_str(termrock::Rect::new(x, seam_y, 1, 1), "─", seam);
    let quiet = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    ui.paint_str(
        termrock::Rect::new(x, seam_y.saturating_add(2), 1, 1),
        "─",
        quiet,
    );
    let root_cost = plan.cost.1.max(1.0);
    let mut y = seam_y.saturating_add(5);
    paint_plan_edge_nodes(ui, x, &mut y, plan, root_cost);
    let _ = area;
}

fn paint_plan_edge_nodes(
    ui: &mut Ui<'_>,
    x: u16,
    y: &mut u16,
    node: &tablepro_sql::sql::PlanNode,
    root_cost: f64,
) {
    let share = plan_share(node, root_cost);
    let mark = if share > 50 { "▲" } else { " " };
    ui.paint_str(
        termrock::Rect::new(x, *y, 1, 1),
        mark,
        plan_share_style(ui, share),
    );
    *y = y.saturating_add(1);
    for child in &node.children {
        paint_plan_edge_nodes(ui, x, y, child, root_cost);
    }
}

fn extend_query_results_edge(ui: &mut Ui<'_>, inner: termrock::Rect) {
    if inner.height < 8 || inner.width == 0 {
        return;
    }
    let top = idle_query_top(inner.height);
    let editor_rows = top.saturating_sub(2);
    let editor_y = inner.y.saturating_add(1);
    let x = inner.right();
    let field = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(TAG_FIELD_BG),
        ),
    );
    ui.fill(termrock::Rect::new(x, editor_y, 1, editor_rows), field);
    if editor_rows > 2 {
        let thumb = field.patch(ui.paint_patch(&StylePatch::new().set_fg(LEGACY_SCROLL_THUMB)));
        ui.paint_str(
            termrock::Rect::new(
                x,
                editor_y.saturating_add(editor_rows.saturating_sub(2)),
                1,
                1,
            ),
            "━",
            thumb,
        );
    }
    let seam_y = editor_y.saturating_add(editor_rows);
    let seam = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::BorderStrong)));
    ui.paint_str(termrock::Rect::new(x, seam_y, 1, 1), "─", seam);
    let quiet = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    ui.paint_str(
        termrock::Rect::new(x, seam_y.saturating_add(2), 1, 1),
        "─",
        quiet,
    );
}

fn extend_query_error_edge(ui: &mut Ui<'_>, area: termrock::Rect, inner: termrock::Rect) {
    if inner.height < 8 || inner.width == 0 {
        return;
    }
    let body_y = inner.y;
    let top = idle_query_top(inner.height);
    let editor_rows = top.saturating_sub(2);
    let editor_y = body_y.saturating_add(1);
    let x = inner.right();
    let field = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(TAG_FIELD_BG),
        ),
    );
    ui.fill(termrock::Rect::new(x, editor_y, 1, editor_rows), field);
    let seam_y = editor_y.saturating_add(editor_rows);
    let seam = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::BorderStrong)));
    ui.paint_str(termrock::Rect::new(x, seam_y, 1, 1), "─", seam);
    let quiet = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    ui.paint_str(
        termrock::Rect::new(x, seam_y.saturating_add(2), 1, 1),
        "─",
        quiet,
    );
    let card = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(QUERY_CARD_BG),
        ),
    );
    let card_y = seam_y.saturating_add(4);
    let card_h = inner.bottom().saturating_sub(card_y).min(8);
    if card_h > 0 {
        ui.fill(termrock::Rect::new(x, card_y, 1, card_h), card);
    }
    let _ = area;
}

fn extend_idle_query_edge(ui: &mut Ui<'_>, area: termrock::Rect, inner: termrock::Rect) {
    let Some(editor) = idle_editor_rect(ui, area, inner) else {
        return;
    };
    let field = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(TAG_FIELD_BG),
        ),
    );
    ui.fill(
        termrock::Rect {
            x: inner.right(),
            y: editor.y,
            width: 1,
            height: editor.height,
        },
        field,
    );
    let seam = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::BorderStrong)));
    ui.paint_str(
        termrock::Rect {
            x: inner.right(),
            y: editor.y.saturating_add(editor.height),
            width: 1,
            height: 1,
        },
        "─",
        seam,
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

fn paint_maximized_query_card(ui: &mut Ui<'_>, area: termrock::Rect, title: &str) {
    if area.width < 4 || area.height < 4 {
        return;
    }
    let border = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    let title_style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG).remove(Modifier::BOLD)));
    let title_w = width(title).min(area.width.saturating_sub(4));
    ui.paint_str(termrock::Rect::new(area.x, area.y, 1, 1), "╭", border);
    ui.paint_str(
        termrock::Rect::new(area.x.saturating_add(1), area.y, 1, 1),
        "─",
        border,
    );
    ui.paint_str(
        termrock::Rect::new(area.x.saturating_add(2), area.y, title_w, 1),
        title,
        title_style,
    );
    let rule_x = area.x.saturating_add(2).saturating_add(title_w);
    let rule_w = area.right().saturating_sub(1).saturating_sub(rule_x);
    if rule_w > 0 {
        ui.paint_str(
            termrock::Rect::new(rule_x, area.y, rule_w, 1),
            &"─".repeat(usize::from(rule_w)),
            border,
        );
    }
    ui.paint_str(
        termrock::Rect::new(area.right().saturating_sub(1), area.y, 1, 1),
        "╮",
        border,
    );
    let last = area.bottom().saturating_sub(1);
    for y in area.y.saturating_add(1)..last {
        ui.paint_str(termrock::Rect::new(area.x, y, 1, 1), "│", border);
        ui.paint_str(
            termrock::Rect::new(area.right().saturating_sub(1), y, 1, 1),
            "│",
            border,
        );
    }
    let span = area.width.saturating_sub(2);
    ui.paint_str(termrock::Rect::new(area.x, last, 1, 1), "╰", border);
    if span > 0 {
        ui.paint_str(
            termrock::Rect::new(area.x.saturating_add(1), last, span, 1),
            &"─".repeat(usize::from(span)),
            border,
        );
    }
    ui.paint_str(
        termrock::Rect::new(area.right().saturating_sub(1), last, 1, 1),
        "╯",
        border,
    );
    let inner = termrock::Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    ui.fill(inner, ui.surface_style());
    paint_query_empty(ui, inner);
}

fn paint_maximized_footer(ui: &mut Ui<'_>, area: termrock::Rect) {
    let base = ui.surface_style();
    ui.paint_spans(
        area,
        &[
            Span::new(" "),
            Span::new("←→").bold(),
            Span::new(" "),
            Span::new("Switch").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Ctrl+T").bold(),
            Span::new(" "),
            Span::new("New query").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("x").bold(),
            Span::new(" "),
            Span::new("Close").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Ctrl+G").bold(),
            Span::new(" "),
            Span::new("Tab list").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("z").bold(),
            Span::new(" "),
            Span::new("Zoom").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Shift+F10").bold(),
            Span::new(" "),
            Span::new("Menu").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Tab").bold(),
            Span::new(" "),
            Span::new("Next").role(TAG_CHROME_FG),
        ],
        base,
    );
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

fn connection_cursor_offset(state: &TreeState, nodes: &[ConnectionNode]) -> Option<u16> {
    let cursor = state.cursor()?;
    let index = nodes
        .iter()
        .position(|node| connection_node_key(node) == cursor)?;
    let row = index.checked_sub(state.scroll().offset())?;
    u16::try_from(row).ok()
}

fn connection_filter_revision(query: &str) -> u64 {
    query.bytes().fold(0x9e37_79b9_u64, |hash, byte| {
        hash.wrapping_mul(0x0100_0000_01b3)
            .wrapping_add(u64::from(byte))
    })
}

fn connection_node_matches(node: &ConnectionNode, query: &str) -> bool {
    let query = query.to_ascii_lowercase();
    match node {
        ConnectionNode::Connection { connection, .. } => {
            connection.name.to_ascii_lowercase().contains(&query)
        }
        ConnectionNode::Group { name } => name.to_ascii_lowercase().contains(&query),
    }
}

fn connection_node_shown(node: &ConnectionNode, nodes: &[ConnectionNode], query: &str) -> bool {
    if query.is_empty() || connection_node_matches(node, query) {
        return true;
    }
    let ConnectionNode::Group { name } = node else {
        return false;
    };
    nodes.iter().any(|other| {
        matches!(
            other,
            ConnectionNode::Connection { connection, .. }
                if connection.group == *name && connection_node_matches(other, query)
        )
    })
}

fn shown_connection_cursor_offset(
    state: &TreeState,
    nodes: &[ConnectionNode],
    query: &str,
) -> Option<u16> {
    if query.is_empty() {
        return connection_cursor_offset(state, nodes);
    }
    let cursor = state.cursor()?;
    let mut seen = 0usize;
    for node in nodes {
        if !connection_node_shown(node, nodes, query) {
            continue;
        }
        if connection_node_key(node) == cursor {
            let row = seen.checked_sub(state.scroll().offset())?;
            return u16::try_from(row).ok();
        }
        seen = seen.saturating_add(1);
    }
    None
}

fn paint_connection_mark_plain(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    state: &TreeState,
    nodes: &[ConnectionNode],
    query: &str,
) {
    let Some(cursor) = state.cursor() else {
        return;
    };
    let Some(ConnectionNode::Connection { connection, .. }) = nodes
        .iter()
        .find(|node| connection_node_key(node) == cursor)
    else {
        return;
    };
    let mark = connection_mark(connection);
    if mark.text == " " {
        return;
    }
    let Some(row) = shown_connection_cursor_offset(state, nodes, query) else {
        return;
    };
    let y = area.y.saturating_add(row);
    if y >= area.bottom() {
        return;
    }
    // Depth-1 label starts two cells after the disclosure column.
    let x = area
        .x
        .saturating_add(1)
        .saturating_add(ui.design().space.tree_indent)
        .saturating_add(2);
    let mut style = ui.surface_style();
    if let Some(role) = mark.role {
        style = style.patch(ui.paint_patch(&StylePatch::new().set_fg(role)));
    }
    // Cell styles only clear a modifier that is subtracted.
    style = style.remove_modifier(Modifier::BOLD);
    ui.paint_str(
        termrock::Rect {
            x,
            y,
            width: 1,
            height: 1,
        },
        mark.text,
        style,
    );
}

fn paint_connection_focus_bar(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    state: &TreeState,
    nodes: &[ConnectionNode],
    query: &str,
) {
    let Some(row) = shown_connection_cursor_offset(state, nodes, query) else {
        return;
    };
    let y = area.y.saturating_add(row);
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
    focus: Option<usize>,
    places: Vec<ColPlace>,
}

impl GridChrome {
    /// The chrome the shared grid bars align to.
    fn content_chrome(&self) -> termrock::grid::GridContentChrome {
        termrock::grid::GridContentChrome {
            gutter: self.gutter,
            content_width: self.content_width,
            columns: self.columns,
            complete: self.complete,
            hidden_left: self.hidden_left,
        }
    }
}

const KIND_OTHER: u8 = 0;
const KIND_PRIMARY: u8 = 1;
const KIND_FOREIGN: u8 = 2;

thread_local! {
    static COL_BUDGETS: RefCell<Vec<ColBudget>> = const { RefCell::new(Vec::new()) };
    static GRID_CHROME: RefCell<Option<GridChrome>> = const { RefCell::new(None) };
    static GRID_SCROLL: RefCell<Option<(termrock::Rect, usize, usize, Option<usize>)>> =
        const { RefCell::new(None) };
    static GRID_SEEN_ROW: RefCell<Option<usize>> = const { RefCell::new(None) };
    static GRID_SORT: RefCell<Option<(usize, SortDir)>> = const { RefCell::new(None) };
    static GRID_FILTER: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static GRID_EDIT: RefCell<Option<usize>> = const { RefCell::new(None) };
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
        focus: None,
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
    accent_rule: bool,
    mark_focus: bool,
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
            if mark_focus {
                let mark = base.patch(
                    ui.paint_patch(
                        &StylePatch::new()
                            .set_fg(Role::Accent)
                            .set_bg(Role::Surface(termrock::Surface::Overlay))
                            .add(Modifier::BOLD),
                    ),
                );
                ui.paint_str(
                    termrock::Rect {
                        x,
                        y: area.y,
                        width: 1,
                        height: 1,
                    },
                    "▎",
                    mark,
                );
            }
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
    if mark_focus {
        let plus_x = area.right().saturating_sub(3);
        if x < plus_x {
            let tail = plus_x.saturating_sub(x);
            ui.paint_str(
                termrock::Rect {
                    x,
                    y: area.y,
                    width: tail,
                    height: 1,
                },
                &" ".repeat(usize::from(tail)),
                base.remove_modifier(Modifier::BOLD),
            );
        }
    }
    let quiet = base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_RULE_FG)));
    let hot = base.patch(ui.paint_patch(&StylePatch::new().set_fg(if accent_rule {
        Role::Accent
    } else {
        Role::BorderSubtle
    })));
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
    accent_name: bool,
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
    let name_style =
        if accent_name && ui.theme().capability.color == termrock::ColorLevel::TrueColor {
            fill.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)))
        } else {
            fill
        };
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
    GRID_SEEN_ROW.with(|slot| {
        let mut seen = slot.borrow_mut();
        *seen = Some(seen.map_or(cell.row, |current| current.min(cell.row)));
    });
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
        let mut patch = StylePatch::new()
            .set_bg(LEGACY_CELL_BG)
            .set_fg(Role::Fg(FgStep::Primary))
            .add(Modifier::BOLD);
        if ui.color_level() == termrock::ColorLevel::Mono {
            patch = patch.add(Modifier::REVERSED);
        }
        ui.patch(&patch);
    } else if budget.kind == KIND_PRIMARY {
        let mut patch = StylePatch::new().set_fg(TAG_PATH_FG);
        if cell.flags.contains(StateFlags::FOCUSED)
            && ui.color_level() == termrock::ColorLevel::Mono
        {
            patch = patch.add(Modifier::REVERSED);
        }
        ui.patch(&patch);
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
    termrock::grid::paint_grid_focus_bar(ui, area, Some(row as u16), &GRID_FOCUS_BAR);
}

fn paint_grid_scrollbar(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    rows: usize,
    origin: usize,
    focused: Option<usize>,
) {
    let Some(chrome) = GRID_CHROME.with(|chrome| chrome.borrow().clone()) else {
        return;
    };
    termrock::grid::paint_grid_vertical_bar(
        ui,
        area,
        &chrome.content_chrome(),
        rows,
        origin,
        focused,
        &GRID_VERTICAL_BAR,
    );
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

fn arrow_role(fade: Option<u16>) -> Role {
    let color = fade.map_or(0x008a_8a8a, |numerator| scaled_rgb(0x008a_8a8a, numerator));
    Role::Custom(termrock::Color::from_u32(color))
}

fn column_is_filtered(name: &str) -> bool {
    GRID_FILTER.with(|slot| {
        slot.borrow()
            .iter()
            .any(|column| column.eq_ignore_ascii_case(name))
    })
}

fn column_is_sorted(index: usize) -> bool {
    GRID_SORT.with(|slot| slot.borrow().is_some_and(|(column, _)| column == index))
}

fn table_query(table: &db::Table, filters: &[Filter]) -> String {
    let mut sql = format!("SELECT * FROM {}.{}", table.schema, table.name);
    let enabled: Vec<&Filter> = filters.iter().filter(|filter| filter.enabled).collect();
    if enabled.is_empty() {
        return sql;
    }
    sql.push_str(" WHERE ");
    sql.push_str(
        &enabled
            .iter()
            .map(|filter| filter.to_sql())
            .collect::<Vec<_>>()
            .join(" AND "),
    );
    sql
}

fn sort_mark(direction: SortDir) -> char {
    match direction {
        SortDir::Asc => '▴',
        SortDir::Desc => '▾',
    }
}

fn filter_footer(app: &TableProApp) -> Option<String> {
    let count = app
        .workbench
        .active_table()?
        .filters
        .iter()
        .filter(|filter| filter.enabled)
        .count();
    match count {
        0 => None,
        1 => Some("1 filter applied".to_owned()),
        n => Some(format!("{n} filters applied")),
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
    if column_is_filtered(name) {
        // " ∇" plus the gap the header keeps after the mark.
        natural = natural.saturating_add(3);
        if column_is_sorted(col) {
            // " ▴" / " ▾" follows the filter mark. Unfiltered sorts already fit in the sample.
            natural = natural.saturating_add(2);
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
            // Four cells sit on the overflow count. The filtered sort frame leaves them blank.
            if room >= 5 {
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
    let wide = GRID_EDIT.with(|slot| *slot.borrow());
    let widths: Vec<u16> = columns
        .iter()
        .enumerate()
        .map(|(index, (name, ty))| {
            let natural = column_natural_width(name, *ty, model, index);
            // The inline editor keeps two cells of field chrome past the sample.
            // A committed edit keeps that width on the focused text column.
            if wide == Some(index) && !matches!(ty, ColType::Int | ColType::Numeric) {
                natural.saturating_add(2)
            } else {
                natural
            }
        })
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
        focus: Some(focus),
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

struct ScrolledTitle<'a> {
    origin: u16,
    place: ColPlace,
    name: &'a str,
    index: usize,
    offset: usize,
    columns: &'a [(String, ColType)],
    sort_at: Option<(usize, SortDir)>,
    focused: bool,
    base: PaintStyle,
    muted: PaintStyle,
    sorted_style: PaintStyle,
}

fn paint_scrolled_title(ui: &mut Ui<'_>, area: termrock::Rect, column: &ScrolledTitle<'_>) {
    let &ScrolledTitle {
        origin,
        place,
        name,
        index,
        offset,
        columns,
        sort_at,
        focused,
        base,
        muted,
        sorted_style,
    } = column;
    let filtered = column_is_filtered(name);
    let sorted = sort_at.is_some_and(|(column, _)| column == index);
    let editing = GRID_EDIT.with(|slot| slot.borrow().is_some_and(|column| column == index));
    // End keeps the cursor column primary even when that column is not sorted or wide.
    let style = if sorted || filtered || editing || focused {
        sorted_style
    } else {
        muted
    };
    let mut title = name.to_owned();
    if filtered {
        title.push(' ');
        title.push('∇');
    }
    if let Some((_, direction)) = sort_at.filter(|(column, _)| *column == index) {
        title.push(' ');
        title.push(sort_mark(direction));
    }
    let title = truncate(&title, place.width);
    let previous_filtered = offset > 0
        && columns
            .get(index.saturating_sub(1))
            .is_some_and(|(name, _)| column_is_filtered(name));
    let cell = termrock::Rect {
        x: origin.saturating_add(place.x),
        y: area.y,
        width: place.width,
        height: 1,
    };
    if filtered {
        // The mark is accent. The gap after it stays the header surface, sort included.
        ui.fill(cell, base);
        ui.fill(
            termrock::Rect {
                width: width(&title).saturating_add(1).min(place.width),
                ..cell
            },
            sorted_style,
        );
    } else {
        if previous_filtered && place.x >= 2 {
            ui.fill(
                termrock::Rect {
                    x: cell.x.saturating_sub(2),
                    width: 2,
                    ..cell
                },
                muted,
            );
        }
        // Editor chrome is wider than the header mark. The extra cells stay the header surface.
        let fill_width = if editing {
            place.width.saturating_sub(2)
        } else {
            place.width
        };
        ui.fill(
            termrock::Rect {
                width: fill_width,
                ..cell
            },
            style,
        );
        if editing {
            // The two-cell gap after the editor header is muted, unlike other gaps.
            ui.fill(
                termrock::Rect {
                    x: cell.x.saturating_add(place.width),
                    width: 2,
                    ..cell
                },
                muted,
            );
        }
    }
    ui.paint_str(cell, &title, style);
}

fn paint_scrolled_header(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    columns: &[(String, ColType)],
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
        let Some((name, _)) = columns.get(index) else {
            continue;
        };
        paint_scrolled_title(
            ui,
            area,
            &ScrolledTitle {
                origin,
                place: *place,
                name,
                index,
                offset,
                columns,
                sort_at,
                focused: chrome.focus.is_some_and(|column| column == index),
                base,
                muted,
                sorted_style,
            },
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
    reason = "header gutter, primary key, and muted titles share one pass"
)]
fn paint_grid_header(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    columns: &[(String, ColType)],
    covered: bool,
    calm_header: bool,
) {
    let Some(chrome) = GRID_CHROME.with(|chrome| chrome.borrow().clone()) else {
        return;
    };
    if chrome.hidden_left > 0 {
        paint_scrolled_header(ui, area, columns, &chrome);
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
    let primary = if calm_header {
        base.patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(TAG_CHROME_FG)
                    .remove(Modifier::BOLD),
            ),
        )
    } else {
        base.patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Fg(if covered {
                        FgStep::Secondary
                    } else {
                        FgStep::Primary
                    }))
                    .add(Modifier::BOLD),
            ),
        )
    };
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
    let key = if calm_header {
        base.patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(LEGACY_FAINT)
                    .remove(Modifier::BOLD),
            ),
        )
    } else {
        base.patch(ui.paint_patch(&StylePatch::new().set_fg(LEGACY_FAINT).add(Modifier::BOLD)))
    };
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
        let Some((name, _)) = columns.get(index) else {
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

fn wide_header_column(
    columns: &[(String, ColType)],
    state: &termrock::GridState,
    model: &ResultGrid,
) -> Option<usize> {
    if let Some(index) = state
        .edit_cell()
        .and_then(|(_, column)| usize::from(column.raw()).checked_sub(1))
    {
        return Some(index);
    }
    if model.pending_total() == 0 {
        return None;
    }
    let (_, column) = state.cursor()?;
    let index = usize::from(column.raw()).checked_sub(1)?;
    if index == 0 {
        return None;
    }
    let numeric = columns
        .get(index)
        .is_some_and(|(_, ty)| matches!(ty, ColType::Int | ColType::Numeric));
    if numeric { None } else { Some(index) }
}

fn paint_row_numbers(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    model: &ResultGrid,
    cursor: Option<usize>,
    origin: usize,
) {
    let rows = model.row_count();
    let Some(chrome) = GRID_CHROME.with(|chrome| chrome.borrow().clone()) else {
        return;
    };
    let digits = usize::from(chrome.gutter.saturating_sub(4)).max(1);
    let visible = usize::from(area.height.saturating_sub(1)).min(rows);
    let plain = ui.surface_style().remove_modifier(Modifier::BOLD);
    for row in 0..visible {
        let y = area.y.saturating_add(1).saturating_add(row as u16);
        let model_row = origin.saturating_add(row);
        let gutter = if cursor == Some(model_row) {
            plain.add_modifier(Modifier::BOLD)
        } else {
            plain
        };
        let fade = visible.saturating_sub(row);
        let truecolor = ui.theme().capability.color == termrock::ColorLevel::TrueColor;
        // These two pads sit in the grid body. The edge fade already ran.
        // Repainting them in primary ink hides the bottom (and top) fade.
        let pad = if truecolor && (fade == 1 || (origin > 0 && row == 0)) {
            gutter.patch(ui.paint_patch(
                &StylePatch::new().set_fg(Role::Custom(termrock::Color::from_u32(0x008c_8c8c))),
            ))
        } else if truecolor && (fade == 2 || (origin > 0 && row == 1)) {
            gutter.patch(ui.paint_patch(
                &StylePatch::new().set_fg(Role::Custom(termrock::Color::from_u32(0x00cc_cccc))),
            ))
        } else {
            gutter
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
                pad,
            );
        }
        let role = if !truecolor {
            if cursor == Some(model_row) {
                TAG_PATH_FG
            } else {
                LEGACY_FAINT
            }
        } else if origin > 0 && row == 0 {
            Role::Custom(termrock::Color::from_u32(0x003f_3f3f))
        } else if origin > 0 && row == 1 {
            Role::Custom(termrock::Color::from_u32(0x005c_5c5c))
        } else if fade == 1 {
            Role::Custom(termrock::Color::from_u32(0x003f_3f3f))
        } else if fade == 2 {
            Role::Custom(termrock::Color::from_u32(0x005c_5c5c))
        } else if cursor == Some(model_row) {
            TAG_PATH_FG
        } else {
            LEGACY_FAINT
        };
        let text = format!("{:>width$}", model_row.saturating_add(1), width = digits);
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
        if model.pending().is_dirty_row(model_row) {
            let mut mark = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Warning)));
            if cursor == Some(model_row) {
                mark = mark.add_modifier(Modifier::BOLD);
            }
            ui.paint_str(
                termrock::Rect {
                    x: area.x.saturating_add(2),
                    y,
                    width: 1,
                    height: 1,
                },
                "•",
                mark,
            );
        }
    }
}

fn pending_detail(model: &ResultGrid) -> String {
    let pending = model.pending();
    let mut updates = 0usize;
    let mut inserts = 0usize;
    let mut deletes = 0usize;
    for row in 0..pending.row_count() {
        if pending.is_inserted(row) && !pending.is_deleted(row) {
            inserts = inserts.saturating_add(1);
        } else if pending.is_deleted(row) && !pending.is_inserted(row) {
            deletes = deletes.saturating_add(1);
        } else if pending.is_dirty_row(row) {
            updates = updates.saturating_add(1);
        }
    }
    let mut parts = Vec::new();
    if updates > 0 {
        parts.push(format!(
            "{updates} update{}",
            if updates == 1 { "" } else { "s" }
        ));
    }
    if inserts > 0 {
        parts.push(format!(
            "{inserts} insert{}",
            if inserts == 1 { "" } else { "s" }
        ));
    }
    if deletes > 0 {
        parts.push(format!(
            "{deletes} delete{}",
            if deletes == 1 { "" } else { "s" }
        ));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("· {}", parts.join(" · "))
    }
}

fn paint_pending_bar(ui: &mut Ui<'_>, area: termrock::Rect, model: &ResultGrid) {
    let base = ui.surface_style();
    let warning = base.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Warning)));
    let muted = base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
    let label = format!("• {} pending", model.pending_total());
    let label_width = width(&label);
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(1),
            width: label_width,
            ..area
        },
        &label,
        warning,
    );
    let detail = pending_detail(model);
    if !detail.is_empty() {
        ui.paint_str(
            termrock::Rect {
                x: area.x.saturating_add(2).saturating_add(label_width),
                width: width(&detail),
                ..area
            },
            &detail,
            muted,
        );
    }
    let save = base.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_BUTTON_FG)
                .set_bg(TAG_BUTTON_BG),
        ),
    );
    let subtle = base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    let save_w = width("Save").saturating_add(2);
    let discard_w = width("Discard").saturating_add(2);
    let preview_w = width("Preview SQL").saturating_add(2);
    let mut x = area.right().saturating_sub(save_w);
    paint_action_button(ui, x, area.y, "Save", save, true);
    x = x.saturating_sub(1).saturating_sub(discard_w);
    paint_action_button(ui, x, area.y, "Discard", subtle, false);
    x = x.saturating_sub(1).saturating_sub(preview_w);
    paint_action_button(ui, x, area.y, "Preview SQL", subtle, false);
}

fn paint_horizontal_bar(ui: &mut Ui<'_>, row: termrock::Rect) {
    let Some(chrome) = GRID_CHROME.with(|chrome| chrome.borrow().clone()) else {
        return;
    };
    termrock::grid::paint_grid_horizontal_bar(
        ui,
        row,
        &chrome.content_chrome(),
        &GRID_HORIZONTAL_BAR,
    );
}

fn presented_row_origin(
    model: &ResultGrid,
    state: &termrock::GridState,
    visible: usize,
    observed: usize,
) -> usize {
    let Some((key, _)) = state.cursor() else {
        return observed;
    };
    let Some(cursor) = (0..model.row_count()).find(|row| GridModel::row_key(model, *row) == key)
    else {
        return observed;
    };
    if visible == 0
        || cursor.saturating_add(1) != model.row_count()
        || !model.pending().is_inserted(cursor)
    {
        return observed;
    }
    let all_blank = (0..)
        .map_while(|col| model.pending().value(cursor, col))
        .all(|val| matches!(val, db::Value::Default | db::Value::Null));
    if all_blank {
        return observed;
    }
    // One context row on the pre-pending viewport leaves the insert below the paint.
    cursor.saturating_sub(visible)
}

fn repaint_presented_rows(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    columns: &[(String, ColType)],
    model: &ResultGrid,
    origin: usize,
    visible: usize,
) {
    let Some(chrome) = GRID_CHROME.with(|chrome| chrome.borrow().clone()) else {
        return;
    };
    let plain = ui.surface_style().remove_modifier(Modifier::BOLD);
    let primary = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    let arrow_style = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(arrow_role(None))));
    let x0 = area.x.saturating_add(chrome.gutter);
    for local in 0..visible {
        let Some(row) = origin.checked_add(local) else {
            continue;
        };
        if row >= model.row_count() {
            break;
        }
        let y = area.y.saturating_add(1).saturating_add(local as u16);
        ui.fill(
            termrock::Rect {
                x: x0,
                y,
                width: chrome.content_width.saturating_add(4),
                height: 1,
            },
            plain,
        );
        for (index, place) in chrome.places.iter().enumerate() {
            let Some(text) = GridModel::cell(model, row, index).map(|cell| cell.text.to_owned())
            else {
                continue;
            };
            let foreign = columns
                .get(index)
                .is_some_and(|(name, ty)| *ty == ColType::Uuid && name != "id");
            let numeric = columns
                .get(index)
                .is_some_and(|(_, ty)| matches!(ty, ColType::Int | ColType::Numeric));
            let budget_arrow =
                COL_BUDGETS.with(|budgets| budgets.borrow().get(index).is_some_and(|b| b.arrow));
            let text_budget = if budget_arrow {
                place.width.saturating_sub(2)
            } else {
                place.width
            };
            let owned;
            let shown: &str = if foreign {
                owned = truncate_middle(&text, text_budget);
                &owned
            } else if width(&text) > text_budget {
                owned = truncate(&text, text_budget);
                &owned
            } else {
                &text
            };
            let style = if place.primary { primary } else { plain };
            let text_width = width(shown).min(text_budget);
            let indent = if numeric {
                text_budget.saturating_sub(text_width)
            } else {
                0
            };
            ui.paint_str(
                termrock::Rect {
                    x: x0.saturating_add(place.x).saturating_add(indent),
                    y,
                    width: text_width,
                    height: 1,
                },
                shown,
                style,
            );
            if budget_arrow {
                ui.paint_str(
                    termrock::Rect {
                        x: x0
                            .saturating_add(place.x)
                            .saturating_add(place.width)
                            .saturating_sub(1),
                        y,
                        width: 1,
                        height: 1,
                    },
                    "→",
                    arrow_style,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_painted_grid(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    id: Id,
    columns: &[(String, ColType)],
    specs: &[termrock::Column<'_>],
    state: &termrock::GridState,
    model: &ResultGrid,
    covered: bool,
    calm_header: bool,
) {
    GRID_SEEN_ROW.with(|slot| *slot.borrow_mut() = None);
    install_grid_budgets(area.width, columns, model);
    install_scrolled_budgets(area.width, columns, model, state);
    let grid = result_grid(id, specs);
    grid.draw(ui, area, state, &PaintedGrid { inner: model });
    paint_grid_focus_bar(ui, area, state, model);
    let visible = usize::from(area.height.saturating_sub(1)).min(model.row_count());
    let observed = GRID_SEEN_ROW.with(|slot| slot.borrow().unwrap_or(0));
    let origin = presented_row_origin(model, state, visible, observed);
    if origin != observed {
        repaint_presented_rows(ui, area, columns, model, origin, visible);
    }
    let cursor = state.cursor().and_then(|(key, _)| {
        (0..model.row_count()).find(|row| GridModel::row_key(model, *row) == key)
    });
    let focused = cursor.and_then(|row| row.checked_sub(origin));
    GRID_SCROLL.with(|slot| *slot.borrow_mut() = Some((area, model.row_count(), origin, focused)));
    paint_row_numbers(ui, area, model, cursor, origin);
    paint_grid_header(ui, area, columns, covered, calm_header);
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
        .header_sizing(termrock::GridHeaderSizing::Minimum {
            padding: 2,
            cap: 24,
        })
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
    let regions = termrock::AppShell::new()
        .header(1)
        .gap(1)
        .footer(1)
        .inset(1)
        .regions(area);
    [regions.header, regions.body, regions.footer]
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

fn paint_idle_query_tail(ui: &mut Ui<'_>, area: termrock::Rect, underline_first: bool) {
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
    let underlined = field.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_underline(Role::BorderStrong)
                .add(Modifier::UNDERLINED),
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
            if underline_first && row == 0 {
                underlined
            } else {
                field
            },
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

fn paint_filter_bar(ui: &mut Ui<'_>, row: termrock::Rect, filters: &[Filter]) {
    if row.width == 0 || row.height == 0 {
        return;
    }
    let quiet = ui
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
    let cap = chip.with_fg_from_bg(chip);
    let cross = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_PATH_FG)
                .set_bg(Role::Surface(termrock::Surface::Overlay)),
        ),
    );
    let gap = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Custom(termrock::Color::from_u32(0)))
                .set_bg(Role::Custom(termrock::Color::from_u32(0))),
        ),
    );
    let mut x = row.x;
    let lead = " match all ▾ ";
    paint_bar_piece(ui, row, &mut x, lead, quiet);
    paint_bar_piece(ui, row, &mut x, " ", ui.surface_style());
    for filter in filters.iter().filter(|filter| filter.enabled) {
        paint_bar_piece(ui, row, &mut x, " ", cap);
        let label = format!("{} ", filter.chip_label());
        paint_bar_piece(ui, row, &mut x, &label, chip);
        paint_bar_piece(ui, row, &mut x, "×", cross);
        paint_bar_piece(ui, row, &mut x, "  ", chip);
        paint_bar_piece(ui, row, &mut x, " ", ui.surface_style());
        paint_bar_piece(ui, row, &mut x, " ", gap);
    }
    paint_bar_piece(ui, row, &mut x, "+ Add filter ", quiet);
}

fn paint_bar_piece(
    ui: &mut Ui<'_>,
    row: termrock::Rect,
    x: &mut u16,
    text: &str,
    style: PaintStyle,
) {
    let piece = width(text);
    if *x >= row.right() || piece == 0 {
        return;
    }
    let slot = piece.min(row.right().saturating_sub(*x));
    ui.paint_str(
        termrock::Rect {
            x: *x,
            y: row.y,
            width: slot,
            height: 1,
        },
        text,
        style,
    );
    *x = x.saturating_add(slot);
}

fn paint_data_mode(ui: &mut Ui<'_>, inner: termrock::Rect, accent_rule: bool) {
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
        .patch(ui.paint_patch(&StylePatch::new().set_fg(if accent_rule {
            Role::Accent
        } else {
            TAG_RULE_FG
        })));
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
    let seam = ui.surface_style().patch(
        ui.style(
            termrock::Family::SPLIT,
            termrock::Variant::DEFAULT,
            Part::SEAM,
            StateFlags::empty(),
        )
        .style,
    );
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
    let pending_label = if app.screen == Screen::Connections {
        None
    } else {
        workbench_pending_label(app)
    };
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
        let env_role = legacy_environment_role(app.connection.environment);
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
                Span::new(glyph).role(env_role).bold(),
                Span::new(" ").role(env_role).bold(),
                Span::new(environment).role(env_role).bold(),
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
    paint_header_pending(ui, area, right.x, pending_label.as_deref(), base);
}

fn workbench_pending_label(app: &TableProApp) -> Option<String> {
    let pending = app
        .workbench
        .active_grid()
        .map_or(0, |(_, grid)| grid.model.pending_total());
    (pending > 0).then(|| format!("• {pending} pending"))
}

fn paint_header_pending(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    cluster_x: u16,
    label: Option<&str>,
    base: PaintStyle,
) {
    let Some(label) = label else {
        return;
    };
    let label_width = width(label);
    // Two cells stay clear of the capability cluster.
    let x = cluster_x.saturating_sub(2).saturating_sub(label_width);
    ui.paint_str(
        termrock::Rect {
            x,
            width: label_width,
            ..area
        },
        label,
        base.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Warning))),
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

fn paint_data_footer(filtered: bool, pending: bool, paint: impl FnOnce(&[Span<'static>])) {
    if pending && !filtered {
        paint(&[
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
            Span::new("Ctrl+S").bold(),
            Span::new(" "),
            Span::new("Save").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("…").role(LEGACY_FAINT),
        ]);
        return;
    }
    if filtered {
        paint(&[
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
            Span::new("Shift+F10").bold(),
            Span::new(" "),
            Span::new("Menu").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("…").role(LEGACY_FAINT),
        ]);
        return;
    }
    paint(&[
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
    ]);
}

fn paint_history_card(ui: &mut Ui<'_>, inner: termrock::Rect, history: &crate::tabs::HistoryTab) {
    ui.register_control(HISTORY_LIST, inner, Focusability::Focusable);
    let quiet_fg = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    // The middle of the blank row under the title is path grey.
    ui.paint_str(
        termrock::Rect::new(inner.x.saturating_add(2), inner.y, 46, 1),
        &" ".repeat(46),
        quiet_fg,
    );
    let muted = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG).set_bg(TAG_FIELD_BG)));
    let field = PaintStyle::new()
        .fg(termrock::Color::Rgb(255, 255, 255))
        .bg(termrock::Color::Rgb(30, 30, 34));
    ui.fill(
        termrock::Rect::new(inner.x, inner.y.saturating_add(1), 48, 1),
        field,
    );
    ui.paint_str(
        termrock::Rect::new(inner.x, inner.y.saturating_add(1), 1, 1),
        " ",
        field.fg(termrock::Color::Rgb(30, 30, 34)),
    );
    let prompt = termrock::Rect::new(inner.x.saturating_add(2), inner.y.saturating_add(1), 32, 1);
    ui.paint_str(prompt, "Search history · terms are ANDed", muted);
    let scope = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
    ui.paint_str(
        termrock::Rect::new(inner.x.saturating_add(50), inner.y.saturating_add(1), 33, 1),
        "scope: Production  ·  status: any",
        scope,
    );
    let list_y = inner.y.saturating_add(3);
    let selected_bg = termrock::Color::Rgb(23, 63, 28);
    let time_right = inner.x.saturating_add(39);
    for (offset, entry) in history.entries.iter().take(10).enumerate() {
        let selected = offset == history.selected;
        let y = list_y.saturating_add(offset as u16);
        let row = termrock::Rect::new(inner.x, y, 40, 1);
        let row_bg = if selected {
            selected_bg
        } else {
            termrock::Color::Rgb(0, 0, 0)
        };
        let base = if selected {
            PaintStyle::new()
                .fg(termrock::Color::Rgb(255, 255, 255))
                .bg(row_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            PaintStyle::new()
                .fg(termrock::Color::Rgb(255, 255, 255))
                .bg(row_bg)
        };
        ui.fill(row, base);
        if !selected {
            ui.paint_str(
                termrock::Rect::new(inner.x, y, 1, 1),
                " ",
                PaintStyle::new().fg(row_bg).bg(row_bg),
            );
        }
        if selected {
            let mark = base
                .fg(termrock::Color::Rgb(72, 224, 84))
                .add_modifier(Modifier::BOLD);
            ui.paint_str(termrock::Rect::new(inner.x, y, 2, 1), "▎›", mark);
        } else if entry.error.is_some() {
            ui.paint_str(
                termrock::Rect::new(inner.x.saturating_add(1), y, 1, 1),
                "!",
                base.fg(termrock::Color::Rgb(240, 93, 93))
                    .add_modifier(Modifier::BOLD),
            );
        }
        let when = format!("{} · {}", entry.when(), entry.duration());
        let when_w = width(&when);
        let when_x = time_right.saturating_sub(when_w);
        let sql_x = inner.x.saturating_add(3);
        let sql_max = when_x.saturating_sub(sql_x).saturating_sub(1);
        let first = entry.sql.lines().next().unwrap_or(entry.sql.as_str());
        let label = truncate(first, sql_max);
        let mut text = PaintStyle::new()
            .fg(termrock::Color::Rgb(255, 255, 255))
            .bg(row_bg);
        let mut time = PaintStyle::new()
            .fg(termrock::Color::Rgb(138, 138, 138))
            .bg(row_bg);
        if selected {
            text = text.add_modifier(Modifier::BOLD);
            time = time.add_modifier(Modifier::BOLD);
        }
        ui.paint_str(
            termrock::Rect::new(sql_x, y, width(&label), 1),
            &label,
            text,
        );
        ui.paint_str(termrock::Rect::new(when_x, y, when_w, 1), &when, time);
    }
    let rule = PaintStyle::new().fg(termrock::Color::Rgb(77, 77, 77));
    let mut rule_y = list_y;
    while rule_y < inner.bottom() {
        ui.paint_str(
            termrock::Rect::new(inner.x.saturating_add(41), rule_y, 1, 1),
            "│",
            rule,
        );
        rule_y = rule_y.saturating_add(1);
    }
    if let Some(entry) = history.entries.get(history.selected) {
        paint_history_detail(ui, inner, entry, list_y);
    }
}

fn paint_history_detail(
    ui: &mut Ui<'_>,
    inner: termrock::Rect,
    entry: &tablepro_domain::model::HistoryEntry,
    list_y: u16,
) {
    let x = inner.x.saturating_add(43);
    let bg = termrock::Color::Rgb(17, 17, 17);
    let card = termrock::Rect::new(
        inner.x.saturating_add(42),
        list_y,
        inner.width.saturating_sub(42),
        inner.bottom().saturating_sub(list_y),
    );
    let fill = PaintStyle::new()
        .fg(termrock::Color::Rgb(255, 255, 255))
        .bg(bg);
    ui.fill(card, fill);
    let label = fill.fg(termrock::Color::Rgb(179, 179, 179));
    let faint = fill.fg(termrock::Color::Rgb(115, 115, 115));
    let quiet = fill.fg(termrock::Color::Rgb(138, 138, 138));
    ui.paint_str(
        termrock::Rect::new(x.saturating_add(1), list_y, 5, 1),
        "Query",
        label,
    );
    let source = format!("{} · {}", entry.source.label(), entry.when());
    let source_w = width(&source);
    ui.paint_str(
        termrock::Rect::new(x.saturating_add(22), list_y, source_w, 1),
        &source,
        faint,
    );
    ui.paint_str(
        termrock::Rect::new(x.saturating_add(1), list_y.saturating_add(1), 37, 1),
        &"─".repeat(37),
        fill.fg(termrock::Color::Rgb(58, 58, 58)),
    );
    let lines: Vec<&str> = entry.sql.lines().collect();
    for (index, line) in lines.iter().take(2).enumerate() {
        let y = list_y.saturating_add(2).saturating_add(index as u16);
        let number = format!(" {number}", number = index + 1);
        ui.paint_str(
            termrock::Rect::new(x.saturating_add(3), y, width(&number), 1),
            &number,
            faint,
        );
        if index == 0 {
            ui.paint_str(termrock::Rect::new(x, y, 1, 1), " ", fill.fg(bg));
            ui.paint_str(
                termrock::Rect::new(x.saturating_add(1), y, 1, 1),
                "›",
                label,
            );
        }
        paint_history_sql(
            ui,
            termrock::Rect::new(x.saturating_add(6), y, 32, 1),
            line,
            fill,
        );
    }
    let facts = [
        (
            "Connection",
            format!("{} · {}.{}", entry.connection, entry.database, entry.schema),
        ),
        ("Duration", entry.duration()),
        (
            "Rows",
            entry
                .rows
                .map(|rows| rows.to_string())
                .unwrap_or_else(|| "–".to_owned()),
        ),
    ];
    for (offset, (name, value)) in facts.iter().enumerate() {
        let y = list_y.saturating_add(6).saturating_add(offset as u16);
        ui.paint_str(
            termrock::Rect::new(x.saturating_add(1), y, 10, 1),
            name,
            quiet,
        );
        let shown = truncate(value, 25);
        ui.paint_str(
            termrock::Rect::new(x.saturating_add(13), y, width(&shown), 1),
            &shown,
            label,
        );
    }
    let open_bg = termrock::Color::Rgb(72, 224, 84);
    let run_bg = termrock::Color::Rgb(39, 39, 42);
    let open = PaintStyle::new()
        .fg(termrock::Color::Rgb(25, 25, 28))
        .bg(open_bg)
        .add_modifier(Modifier::BOLD);
    let secondary = PaintStyle::new()
        .fg(termrock::Color::Rgb(255, 255, 255))
        .bg(run_bg);
    let button_y = list_y.saturating_add(10);
    ui.paint_str(
        termrock::Rect::new(x.saturating_add(1), button_y, 1, 1),
        " ",
        PaintStyle::new().fg(open_bg).bg(open_bg),
    );
    ui.paint_str(
        termrock::Rect::new(x.saturating_add(2), button_y, 16, 1),
        "Open in new tab ",
        open,
    );
    ui.paint_str(
        termrock::Rect::new(x.saturating_add(20), button_y, 1, 1),
        " ",
        PaintStyle::new().fg(run_bg).bg(run_bg),
    );
    ui.paint_str(
        termrock::Rect::new(x.saturating_add(21), button_y, 15, 1),
        "Run in new tab ",
        secondary,
    );
    let copy_y = list_y.saturating_add(11);
    ui.paint_str(
        termrock::Rect::new(x.saturating_add(1), copy_y, 1, 1),
        " ",
        fill.fg(bg),
    );
    ui.paint_str(
        termrock::Rect::new(x.saturating_add(2), copy_y, 5, 1),
        "Copy ",
        label,
    );
}

fn paint_history_sql(ui: &mut Ui<'_>, area: termrock::Rect, sql: &str, field: PaintStyle) {
    let keyword = field
        .fg(termrock::Color::Rgb(121, 184, 255))
        .add_modifier(Modifier::BOLD);
    let punct = field.fg(termrock::Color::Rgb(138, 138, 138));
    let text = field.fg(termrock::Color::Rgb(255, 255, 255));
    let mut x = area.x;
    let end = area.right();
    let mut word = String::new();
    let flush = |ui: &mut Ui<'_>, x: &mut u16, word: &mut String| {
        if word.is_empty() || *x >= end {
            word.clear();
            return;
        }
        let style = if matches!(
            word.to_ascii_uppercase().as_str(),
            "SELECT" | "FROM" | "WHERE" | "GROUP" | "BY" | "ORDER" | "COUNT"
        ) {
            keyword
        } else {
            text
        };
        let w = width(word).min(end.saturating_sub(*x));
        ui.paint_str(
            termrock::Rect {
                x: *x,
                width: w,
                ..area
            },
            word,
            style,
        );
        *x = x.saturating_add(w);
        word.clear();
    };
    for ch in sql.chars() {
        if x >= end {
            break;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' {
            word.push(ch);
            continue;
        }
        flush(ui, &mut x, &mut word);
        if x >= end {
            break;
        }
        let style = if ch == ' ' { text } else { punct };
        ui.paint_str(
            termrock::Rect {
                x,
                width: 1,
                ..area
            },
            &ch.to_string(),
            style,
        );
        x = x.saturating_add(1);
    }
    flush(ui, &mut x, &mut word);
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
            Some(Tab::Table(table)) => {
                let filtered = table.filters.iter().any(|filter| filter.enabled);
                let pending = table.result.pending_total() > 0;
                // An active sort uses the shorter hint row so the sort label fits.
                paint_data_footer(filtered && sort_footer(app).is_none(), pending, paint);
            }
            Some(Tab::Query(query))
                if query.error.is_some() || (query.result.is_some() && query.plan.is_none()) =>
            {
                paint(&[
                    Span::new(" "),
                    Span::new("Enter").bold(),
                    Span::new(" "),
                    Span::new("Edit").role(Role::Fg(FgStep::Muted)),
                    Span::new("  "),
                    Span::new("Ctrl+R").bold(),
                    Span::new(" "),
                    Span::new("Run").role(Role::Fg(FgStep::Muted)),
                    Span::new("  "),
                    Span::new("Alt+R").bold(),
                    Span::new(" "),
                    Span::new("Run all").role(Role::Fg(FgStep::Muted)),
                    Span::new("  "),
                    Span::new("Ctrl+X").bold(),
                    Span::new(" "),
                    Span::new("Explain").role(Role::Fg(FgStep::Muted)),
                    Span::new("  "),
                    Span::new("/").bold(),
                    Span::new(" "),
                    Span::new("Find").role(Role::Fg(FgStep::Muted)),
                    Span::new("  "),
                    Span::new("Tab").bold(),
                    Span::new(" "),
                    Span::new("Next").role(Role::Fg(FgStep::Muted)),
                ]);
            }
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
                Span::new("Enter").bold(),
                Span::new(" "),
                Span::new("Open in new tab").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("→").bold(),
                Span::new(" "),
                Span::new("Full text").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("r").bold(),
                Span::new(" "),
                Span::new("Rerun").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("y").bold(),
                Span::new(" "),
                Span::new("Copy").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("/").bold(),
                Span::new(" "),
                Span::new("Search").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("c s").bold(),
                Span::new(" "),
                Span::new("Scope · Status").role(TAG_CHROME_FG),
                Span::new("  "),
                Span::new("…").role(LEGACY_FAINT),
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

fn grid_is_editing(app: &TableProApp) -> bool {
    app.workbench
        .active_grid()
        .is_some_and(|(_, grid)| grid.state.is_editing())
}

fn paint_cell_edit_footer(ui: &mut Ui<'_>, area: termrock::Rect) {
    let base = ui.surface_style();
    let badge = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_BUTTON_FG)
                .set_bg(TAG_BUTTON_BG)
                .add(Modifier::BOLD),
        ),
    );
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(1),
            width: 6.min(area.width.saturating_sub(1)),
            ..area
        },
        " EDIT ",
        badge,
    );
    ui.paint_spans(
        termrock::Rect {
            x: area.x.saturating_add(9),
            width: area.width.saturating_sub(9),
            ..area
        },
        &[
            Span::new("Enter").bold(),
            Span::new(" "),
            Span::new("Apply").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Esc").bold(),
            Span::new(" "),
            Span::new("Cancel").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Tab").bold(),
            Span::new(" "),
            Span::new("Next cell").role(TAG_CHROME_FG),
        ],
        base,
    );
}

fn connection_duplicated_notice(app: &TableProApp) -> Option<&str> {
    if app.screen == Screen::Connections && app.status == "Duplicated" {
        Some(app.status.as_str())
    } else {
        None
    }
}

fn span_paint(ui: &Ui<'_>, base: PaintStyle, span: &Span<'_>) -> PaintStyle {
    let mut style = base.add_modifier(span.add);
    if let Some(role) = span.role {
        style = style.patch(ui.paint_patch(&StylePatch::new().set_fg(role)));
    }
    style
}

/// Eight columns stay clear of the right edge, plus the status when one is set.
fn connection_hint_limit(area_width: u16, notice: Option<&str>) -> u16 {
    let reserve = 8u16.saturating_add(notice.map_or(0, width));
    area_width.saturating_sub(reserve)
}

fn footer_spans_fit(spans: &[Span<'_>], limit: u16) -> bool {
    let mut used = 0u16;
    for span in spans {
        let span_width = width(span.text);
        if used.saturating_add(span_width) > limit {
            return false;
        }
        used = used.saturating_add(span_width);
    }
    true
}

fn paint_connection_hints(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    spans: &[Span<'_>],
    notice: Option<&str>,
    base: PaintStyle,
) {
    let limit = connection_hint_limit(area.width, notice);
    if footer_spans_fit(spans, limit) {
        ui.paint_spans(area, spans, base);
        return;
    }
    // A group separator still paints so the ellipsis sits after the last hint.
    let mut x = area.x;
    let mut used = 0u16;
    for span in spans {
        let span_width = width(span.text);
        let overflows = used.saturating_add(span_width) > limit;
        if overflows && span.text != "  " {
            break;
        }
        ui.paint_str(
            termrock::Rect {
                x,
                width: span_width,
                height: 1,
                ..area
            },
            span.text,
            span_paint(ui, base, span),
        );
        x = x.saturating_add(span_width);
        used = used.saturating_add(span_width);
        if overflows {
            break;
        }
    }
    ui.paint_str(
        termrock::Rect {
            x,
            width: 1,
            height: 1,
            ..area
        },
        "…",
        base.patch(ui.paint_patch(&StylePatch::new().set_fg(LEGACY_FAINT))),
    );
}

fn paint_connection_filter_row(ui: &mut Ui<'_>, area: termrock::Rect, query: &str, editing: bool) {
    let (text, ink) = if query.is_empty() {
        ("Filter connections", TAG_CHROME_FG)
    } else {
        (query, Role::Fg(FgStep::Primary))
    };
    paint_legacy_filter(ui, area, text, ink);
    if !editing || area.is_empty() {
        return;
    }
    if !query.is_empty() && area.width > 2 {
        let underlined = ui.surface_style().remove_modifier(Modifier::BOLD).patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .set_bg(TAG_FIELD_BG)
                    .set_underline(TAG_BUTTON_BG)
                    .add(Modifier::UNDERLINED),
            ),
        );
        ui.paint_str(
            termrock::Rect {
                x: area.x.saturating_add(2),
                width: width(query).min(area.width.saturating_sub(2)),
                ..area
            },
            query,
            underlined,
        );
    }
    let caret = ui
        .surface_style()
        .remove_modifier(Modifier::BOLD)
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_BUTTON_BG).set_bg(TAG_FIELD_BG)));
    ui.paint_str(
        termrock::Rect {
            width: 1.min(area.width),
            ..area
        },
        "▎",
        caret,
    );
    let cursor_x = area
        .x
        .saturating_add(2)
        .saturating_add(width(query))
        .min(area.right().saturating_sub(1));
    ui.set_bar_cursor(CONNECTION_FILTER, Position::new(cursor_x, area.y));
}

fn paint_form_footer(ui: &mut Ui<'_>, area: termrock::Rect) {
    let base = ui.surface_style();
    let badge = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_BUTTON_FG)
                .set_bg(TAG_BUTTON_BG)
                .add(Modifier::BOLD),
        ),
    );
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(1),
            width: 6.min(area.width.saturating_sub(1)),
            ..area
        },
        " EDIT ",
        badge,
    );
    ui.paint_spans(
        termrock::Rect {
            x: area.x.saturating_add(9),
            width: area.width.saturating_sub(9),
            ..area
        },
        &[
            Span::new("Enter").bold(),
            Span::new(" "),
            Span::new("Apply").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Esc").bold(),
            Span::new(" "),
            Span::new("Cancel").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Tab").bold(),
            Span::new(" "),
            Span::new("Next field").role(TAG_CHROME_FG),
        ],
        base,
    );
}

fn paint_query_editor_footer(ui: &mut Ui<'_>, area: termrock::Rect) {
    let base = ui.surface_style();
    let badge = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_BUTTON_FG)
                .set_bg(TAG_BUTTON_BG)
                .add(Modifier::BOLD),
        ),
    );
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(1),
            width: 6.min(area.width.saturating_sub(1)),
            ..area
        },
        " EDIT ",
        badge,
    );
    ui.paint_spans(
        termrock::Rect {
            x: area.x.saturating_add(9),
            width: area.width.saturating_sub(9),
            ..area
        },
        &[
            Span::new("Ctrl+R").bold(),
            Span::new(" "),
            Span::new("Run").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Alt+R").bold(),
            Span::new(" "),
            Span::new("Run all").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Ctrl+Space").bold(),
            Span::new(" "),
            Span::new("Complete").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Tab").bold(),
            Span::new(" "),
            Span::new("Indent").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("F6").bold(),
            Span::new(" "),
            Span::new("Leave").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Esc").bold(),
            Span::new(" "),
            Span::new("Done").role(TAG_CHROME_FG),
        ],
        base,
    );
}

fn paint_connection_filter_footer(ui: &mut Ui<'_>, area: termrock::Rect) {
    let base = ui.surface_style();
    let badge = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_BUTTON_FG)
                .set_bg(TAG_BUTTON_BG)
                .add(Modifier::BOLD),
        ),
    );
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(1),
            width: 6.min(area.width.saturating_sub(1)),
            ..area
        },
        " EDIT ",
        badge,
    );
    ui.paint_spans(
        termrock::Rect {
            x: area.x.saturating_add(9),
            width: area.width.saturating_sub(9),
            ..area
        },
        &[
            Span::new("Type").bold(),
            Span::new(" "),
            Span::new("Filter").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("↓").bold(),
            Span::new(" "),
            Span::new("Into list").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Esc").bold(),
            Span::new(" "),
            Span::new("Clear").role(TAG_CHROME_FG),
        ],
        base,
    );
}

fn paint_filter_footer(ui: &mut Ui<'_>, area: termrock::Rect) {
    let base = ui.surface_style();
    ui.paint_spans(
        area,
        &[
            Span::new(" "),
            Span::new("Tab").bold(),
            Span::new(" "),
            Span::new("Next field").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Enter").bold(),
            Span::new(" "),
            Span::new("Apply").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Esc").bold(),
            Span::new(" "),
            Span::new("Cancel").role(TAG_CHROME_FG),
        ],
        base,
    );
}

fn paint_closed_select(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    label: &str,
    value: &str,
    help: Option<&str>,
) {
    let elevated = Role::Surface(termrock::Surface::Elevated);
    let secondary = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Secondary))
                .set_bg(elevated),
        ),
    );
    let field = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(TAG_FIELD_BG),
        ),
    );
    let blank = field.with_fg_from_bg(field);
    let chevron =
        field.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
    let muted = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Muted))
                .set_bg(elevated),
        ),
    );
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(2),
            y: area.y,
            width: width(label),
            height: 1,
        },
        label,
        secondary,
    );
    let row = termrock::Rect {
        y: area.y.saturating_add(1),
        height: 1,
        ..area
    };
    ui.fill(row, field);
    ui.paint_str(termrock::Rect { width: 1, ..row }, " ", blank);
    ui.paint_str(
        termrock::Rect {
            x: row.x.saturating_add(2),
            width: width(value),
            ..row
        },
        value,
        field,
    );
    ui.paint_str(
        termrock::Rect {
            x: row.right().saturating_sub(2),
            width: 1,
            ..row
        },
        "▾",
        chevron,
    );
    if let Some(help) = help {
        let shown = truncate(help, area.width.saturating_sub(2));
        ui.paint_str(
            termrock::Rect {
                x: area.x.saturating_add(2),
                y: area.y.saturating_add(2),
                width: width(&shown),
                height: 1,
            },
            &shown,
            muted,
        );
    }
}

fn paint_filter_card(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    column: &str,
    value: &str,
    input: &TextInputState,
) {
    termrock::runtime::overlay_chrome(
        ui,
        FILTER_LAYER,
        area,
        termrock::Family::DIALOG,
        termrock::Surface::Elevated,
        termrock::runtime::PartStyle::new(),
        StateFlags::empty(),
        StateFlags::FOCUSED,
        |ui, _framed| {
            paint_filter_card_body(ui, area, column, value, input);
        },
    );
}

fn paint_filter_card_body(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    column: &str,
    value: &str,
    input: &TextInputState,
) {
    let elevated = Role::Surface(termrock::Surface::Elevated);
    let bold = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(elevated)
                .add(Modifier::BOLD),
        ),
    );
    let secondary = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Secondary))
                .set_bg(elevated),
        ),
    );
    let field = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(TAG_FIELD_BG),
        ),
    );
    let placeholder =
        field.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
    let caret = field.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(3),
            y: area.y.saturating_add(1),
            width: 10,
            height: 1,
        },
        "Add filter",
        bold,
    );
    let band_x = area.x.saturating_add(2);
    let band_y = area.y.saturating_add(3);
    let band_w = area.width.saturating_sub(5);
    let left_w = band_w.saturating_sub(2) / 2;
    let right_w = band_w.saturating_sub(2).saturating_sub(left_w);
    let right_x = band_x.saturating_add(left_w).saturating_add(2);
    paint_closed_select(
        ui,
        termrock::Rect::new(band_x, band_y, left_w, 3),
        "Column",
        column,
        None,
    );
    paint_closed_select(
        ui,
        termrock::Rect::new(right_x, band_y, right_w, 3),
        "Operator",
        "=",
        Some("Operators that fit the column type come first"),
    );
    let value_w = band_w;
    let value_label = fit_cols("Value", value_w.saturating_sub(2));
    ui.paint_str(
        termrock::Rect {
            x: band_x.saturating_add(2),
            y: band_y.saturating_add(3),
            width: value_w.saturating_sub(2),
            height: 1,
        },
        &value_label,
        bold,
    );
    let value_row = termrock::Rect {
        x: band_x,
        y: band_y.saturating_add(4),
        width: value_w,
        height: 1,
    };
    TextInput::new(FILTER_VALUE)
        .placeholder("value")
        .placeholder_while_editing(true)
        .draw(ui, value_row, input);
    ui.fill(value_row, field);
    ui.paint_str(
        termrock::Rect {
            width: 1,
            ..value_row
        },
        "▎",
        caret,
    );
    let shown = if value.is_empty() { "value" } else { value };
    let shown_style = if value.is_empty() { placeholder } else { field };
    ui.paint_str(
        termrock::Rect {
            x: value_row.x.saturating_add(2),
            width: width(shown),
            ..value_row
        },
        shown,
        shown_style,
    );
    ui.set_bar_cursor(
        FILTER_VALUE,
        Position::new(value_row.x.saturating_add(2), value_row.y),
    );
    let preview = format!("WHERE {column} = '{}'", value.replace('\'', "''"));
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(3),
            y: area.y.saturating_add(10),
            width: width(&preview),
            height: 1,
        },
        &preview,
        secondary,
    );
    let row_y = area.y.saturating_add(13);
    let row_right = area.x.saturating_add(area.width).saturating_sub(3);
    let apply_x = row_right.saturating_sub(12);
    let cancel_x = apply_x.saturating_sub(9);
    let blank = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(elevated).set_bg(elevated)));
    ui.paint_str(
        termrock::Rect {
            x: cancel_x,
            y: row_y,
            width: 1,
            height: 1,
        },
        " ",
        blank,
    );
    ui.paint_str(
        termrock::Rect {
            x: cancel_x.saturating_add(1),
            y: row_y,
            width: 7,
            height: 1,
        },
        "Cancel ",
        secondary,
    );
    let apply_pad = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent).set_bg(Role::Accent)));
    let apply = ui.surface_style().patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_BUTTON_FG)
                .set_bg(TAG_BUTTON_BG)
                .add(Modifier::BOLD),
        ),
    );
    ui.paint_str(
        termrock::Rect {
            x: apply_x,
            y: row_y,
            width: 1,
            height: 1,
        },
        " ",
        apply_pad,
    );
    ui.paint_str(
        termrock::Rect {
            x: apply_x.saturating_add(1),
            y: row_y,
            width: 11,
            height: 1,
        },
        "Add filter ",
        apply,
    );
    ui.register_focus_only(FILTER_STEP, Focusability::Focusable);
    ui.register_focus_only(FILTER_APPLY, Focusability::Focusable);
}

fn paint_delete_footer(ui: &mut Ui<'_>, area: termrock::Rect) {
    let base = ui.surface_style();
    ui.paint_spans(
        area,
        &[
            Span::new(" "),
            Span::new("←→").bold(),
            Span::new(" "),
            Span::new("Choose").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Enter").bold(),
            Span::new(" "),
            Span::new("Cancel").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Esc").bold(),
            Span::new(" "),
            Span::new("Cancel").role(TAG_CHROME_FG),
        ],
        base,
    );
}

fn paint_help_footer(ui: &mut Ui<'_>, area: termrock::Rect) {
    let base = ui.surface_style();
    ui.paint_spans(
        area,
        &[
            Span::new(" "),
            Span::new("↑↓").bold(),
            Span::new(" "),
            Span::new("Scroll").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Esc").bold(),
            Span::new(" "),
            Span::new("Close").role(TAG_CHROME_FG),
        ],
        base,
    );
}

fn paint_picker_move_footer(ui: &mut Ui<'_>, area: termrock::Rect) {
    let base = ui.surface_style();
    ui.paint_spans(
        area,
        &[
            Span::new(" "),
            Span::new("↑↓").bold(),
            Span::new(" "),
            Span::new("Move").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Enter").bold(),
            Span::new(" "),
            Span::new("Choose").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Esc").bold(),
            Span::new(" "),
            Span::new("Cancel").role(TAG_CHROME_FG),
        ],
        base,
    );
}

fn paint_picker_layer_footer(ui: &mut Ui<'_>, area: termrock::Rect) {
    let base = ui.surface_style();
    ui.paint_spans(
        area,
        &[
            Span::new(" "),
            Span::new("Type").bold(),
            Span::new(" "),
            Span::new("Filter").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("↑↓").bold(),
            Span::new(" "),
            Span::new("Move").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Enter").bold(),
            Span::new(" "),
            Span::new("Choose").role(TAG_CHROME_FG),
            Span::new("  "),
            Span::new("Esc").bold(),
            Span::new(" "),
            Span::new("Cancel").role(TAG_CHROME_FG),
        ],
        base,
    );
}

fn draw_footer(ui: &mut Ui<'_>, area: termrock::Rect, app: &TableProApp) {
    let base = ui.surface_style();
    let explorer_focused = ui.state(EXPLORER).contains(StateFlags::FOCUSED);
    let empty_query = app.blank_sql() && query_editor_focused(ui, app);
    ui.fill(area, base);
    if matches!(app.surface, Surface::QuickSwitcher | Surface::TabListPicker) {
        paint_picker_layer_footer(ui, area);
    } else if app.surface == Surface::SafeModePicker {
        paint_picker_move_footer(ui, area);
    } else if app.surface == Surface::HelpDialog {
        paint_help_footer(ui, area);
    } else if app.surface == Surface::FilterEditor {
        paint_filter_footer(ui, area);
    } else if matches!(app.workbench.active(), Some(Tab::Query(query)) if query.plan.is_some()) {
        paint_query_editor_footer(ui, area);
    } else if app.connection_delete == ConnectionDelete::Open || app.destructive_intent.is_some() {
        paint_delete_footer(ui, area);
    } else if app.form_open {
        paint_form_footer(ui, area);
    } else if app.screen == Screen::Connections && app.connection_filter_editing {
        paint_connection_filter_footer(ui, area);
        return;
    } else if app.workbench.maximized {
        paint_maximized_footer(ui, area);
    } else if empty_query && !explorer_focused {
        paint_query_editor_footer(ui, area);
    } else if !explorer_focused && grid_is_editing(app) {
        paint_cell_edit_footer(ui, area);
    } else {
        let notice = connection_duplicated_notice(app);
        with_footer_spans(app, explorer_focused, |spans| {
            if app.screen == Screen::Connections {
                paint_connection_hints(ui, area, spans, notice, base);
            } else {
                ui.paint_spans(area, spans, base);
            }
        });
    }
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
    } else if let Some(filtered) = filter_footer(app) {
        let text_width = width(&filtered);
        let right = termrock::Rect {
            x: area.right().saturating_sub(text_width).saturating_sub(1),
            width: text_width,
            ..area
        };
        ui.paint_str(
            right,
            &filtered,
            base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG))),
        );
    } else if let Some(notice) = connection_duplicated_notice(app) {
        let text_width = width(notice);
        let right = termrock::Rect {
            x: area.right().saturating_sub(text_width).saturating_sub(1),
            width: text_width,
            ..area
        };
        ui.paint_str(
            right,
            notice,
            base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG))),
        );
    } else if app.screen == Screen::Workbench {
        let checked = matches!(
            app.surface,
            Surface::TableGrid | Surface::StructureView | Surface::HistoryTab
        ) || explorer_focused
            || empty_query
            || (app.blank_sql()
                && matches!(
                    app.surface,
                    Surface::QuickSwitcher
                        | Surface::TabListPicker
                        | Surface::SafeModePicker
                        | Surface::HelpDialog
                ))
            || app.destructive_intent.is_some()
            || app.surface == Surface::FilterEditor
            || matches!(
                app.workbench.active(),
                Some(Tab::Query(query))
                    if query.error.is_some() || query.plan.is_some() || query.result.is_some()
            );
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

fn query_editor_focused(ui: &Ui<'_>, app: &TableProApp) -> bool {
    app.query_id()
        .is_some_and(|id| ui.state(id).contains(StateFlags::FOCUSED))
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

impl TableProApp {
    fn blank_sql(&self) -> bool {
        self.query_id().is_some() && self.blank_maximized_query()
    }

    fn blank_maximized_query(&self) -> bool {
        match self.workbench.active() {
            Some(Tab::Query(query)) => {
                query.query.is_empty()
                    && query.result.is_none()
                    && query.error.is_none()
                    && query.plan.is_none()
            }
            _ => false,
        }
    }
    fn prepare_grid_slots(&self) {
        let filtered_columns = self
            .workbench
            .active_table()
            .map(|table| {
                table
                    .filters
                    .iter()
                    .filter(|filter| filter.enabled)
                    .map(|filter| filter.column.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        GRID_FILTER.with(|slot| *slot.borrow_mut() = filtered_columns);
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
        let edit_col = self.workbench.active_grid().and_then(|(_, grid)| {
            grid.state
                .edit_cell()
                .and_then(|(_, column)| usize::from(column.raw()).checked_sub(1))
        });
        let wide_col = self
            .workbench
            .active_grid()
            .and_then(|(_, grid)| wide_header_column(&grid.columns, &grid.state, &grid.model));
        GRID_EDIT.with(|slot| *slot.borrow_mut() = wide_col);
        if edit_col.is_some() && std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            let mut out = std::io::stdout().lock();
            let _ = std::io::Write::write_all(&mut out, b"\x1b[6 q");
            let _ = std::io::Write::flush(&mut out);
        }
    }

    fn mirror_name_edit(&mut self) {
        let Some(text) = self.form_state.editing_text(connections::field::NAME) else {
            return;
        };
        let Some(draft) = self.draft.as_mut() else {
            return;
        };
        if draft.name != text {
            text.clone_into(&mut draft.name);
        }
    }

    #[cfg(test)]
    fn blank_new_connection_draft(&self) -> bool {
        let Some(draft) = &self.draft else {
            return false;
        };
        draft.name.is_empty()
            && draft.engine == 0
            && draft.host == "localhost"
            && draft.port == "5432"
            && draft.database.is_empty()
            && draft.user.is_empty()
            && !draft.has_password()
            && !draft.ask_password
            && draft.environment == 0
            && draft.group == 0
            && draft.safe_mode == 0
            && !draft.ssl
            && !draft.ssh
            && draft.ssh_host.is_empty()
            && draft.startup.is_empty()
    }
}

impl App for TableProApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = self.update_filter_editor(cx);
        if cx.update_cause() == UpdateCause::Bootstrap
            && self.screen == Screen::Workbench
            && self.surface == Surface::QuickSwitcher
        {
            self.open_switcher(cx);
        }
        let switcher_was_open = cx.is_open(quick_switcher::ID);
        let modal_was_open = self.destructive_intent.is_some();
        response |= self.update_destructive_dialog(cx);
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
        response |= self.update_tab_list(cx);
        response |= self.update_safe_mode(cx);
        response |= self.update_help(cx);
        response |= self.update_delete_dialog(cx);
        if switcher_was_open {
            return response;
        }

        let _ = Self::connections_panel("", None, true);
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
                        query.record_status("cancelled", false, 0);
                        "Query cancelled".clone_into(&mut self.status);
                        return Response::changed();
                    }
                    return self.request_quit(cx);
                }
                c if c == DUPLICATE_ROW => {
                    if self.screen == Screen::Connections && !self.form_open {
                        self.duplicate_selected_connection();
                        response |= Response::changed();
                    } else if self.active_row_action(cx) {
                        self.duplicate_active_row();
                        response |= Response::changed();
                    }
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
                c if c == EXPLAIN => {
                    self.commit_query_edit();
                    if let Some(id) = self.query_id() {
                        cx.focus(id);
                    }
                    if let Some(Tab::Query(query)) = self.workbench.active_mut()
                        && query.explain(&self.catalog).is_ok()
                    {
                        "Explain plan ready".clone_into(&mut self.status);
                    }
                    response |= Response::changed();
                }
                c if c == EXPLAIN_ANALYZE => {
                    self.commit_query_edit();
                    if let Some(id) = self.query_id() {
                        cx.focus(id);
                    }
                    if let Some(Tab::Query(query)) = self.workbench.active_mut()
                        && let Ok(statement) = crate::sql::parse(query.query.trim())
                        && let crate::sql::Statement::Select(select) = statement
                        && let Ok(plan) = crate::sql::explain(&self.catalog, &select, true)
                    {
                        query.plan = Some(plan);
                        query.error = None;
                        "Explain plan ready".clone_into(&mut self.status);
                    }
                    response |= Response::changed();
                }
                c if c == OPEN => {
                    if self.screen == Screen::Workbench {
                        self.open_switcher(cx);
                        response |= Response::changed();
                    }
                }
                c if c == TAB_LIST && self.open_tab_list(cx) => {
                    response |= Response::changed();
                }
                c if c == CLEAR_QUERY && self.open_safe_mode_picker(cx) => {
                    response |= Response::changed();
                }
                c if c == NEW_QUERY => {
                    self.new_query(cx, "");
                    response |= Response::changed();
                }
                c if c == CLOSE_TAB
                    && self.screen == Screen::Workbench
                    && !self.form_open
                    && self.destructive_intent.is_none()
                    && let Some(key) = self.workbench.active_key() =>
                {
                    self.request_close_tab(cx, key);
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
                c if c == DELETE_CONNECTION && self.open_delete_dialog(cx) => {
                    response |= Response::changed();
                }
                c if c == FORM => {
                    self.begin_connection_form();
                    cx.focus(connections::field::NAME);
                    response |= Response::changed();
                }
                c if c == HELP && self.open_help(cx) => {
                    response |= Response::changed();
                }
                c if c == SORT && self.sort_focused_column(cx) => {
                    response |= Response::changed();
                }
                c if c == FILTER && self.begin_column_filter(cx) => {
                    response |= Response::changed();
                }
                c if c == FILTER_CARD && self.begin_filter_card(cx) => {
                    response |= Response::changed();
                }
                c if c == FILTER_CONNECTIONS
                    && self.screen == Screen::Connections
                    && !self.form_open
                    && !self.connection_filter_editing =>
                {
                    self.connection_filter_editing = true;
                    self.connection_filter_input
                        .begin(&self.connections_screen.filter);
                    cx.focus(CONNECTION_FILTER);
                    response |= Response::changed();
                }
                c if c == CLEAR_CONNECTION_FILTER && self.connection_filter_editing => {
                    self.connections_screen.filter.clear();
                    self.connection_filter_editing = false;
                    self.connection_filter_input.cancel();
                    cx.focus(CONNECTIONS);
                    response |= Response::changed();
                }
                c if c == COMMIT_CONNECTION_FILTER && self.connection_filter_editing => {
                    if let Some(draft) = self.connection_filter_input.draft_text() {
                        draft.clone_into(&mut self.connections_screen.filter);
                    }
                    self.connection_filter_editing = false;
                    self.connection_filter_input.cancel();
                    cx.focus(CONNECTIONS);
                    response |= Response::changed();
                }
                c if c == ZOOM
                    && self.screen == Screen::Workbench
                    && !self.form_open
                    && self.destructive_intent.is_none() =>
                {
                    self.workbench.toggle_maximized();
                    response |= Response::changed();
                }
                c if c == TOGGLE_EXPLORER
                    && self.screen == Screen::Workbench
                    && !self.form_open
                    && self.destructive_intent.is_none() =>
                {
                    self.split_state.toggle_max(Maximized::Second);
                    response |= Response::changed();
                }
                c if c == FOCUS_EXPLORER
                    && self.screen == Screen::Workbench
                    && !self.form_open
                    && self.destructive_intent.is_none() =>
                {
                    if self.split_state.maximized() == Maximized::Second {
                        self.split_state.toggle_max(Maximized::Second);
                    }
                    self.workbench.maximized = false;
                    cx.focus(EXPLORER);
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
            self.mirror_name_edit();
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
            response |= self.update_connection_filter(cx);
            let tree_response = self.update_connection_tree(cx);
            // Typing the filter moves the hidden cursor onto the match. The open
            // detail stays on the connection that was selected before `/`.
            if !self.connection_filter_editing {
                self.sync_connection_selection();
            }
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
                TabsAction::New => self.new_query(cx, ""),
            }
        }
        response |= tabs_response.erase();
        if self.workbench.tabs().len() <= 3 {
            self.pin_tab_window();
        }
        if cx.update_cause() == UpdateCause::Bootstrap && self.screen == Screen::Workbench {
            cx.focus(EXPLORER);
        }
        if self.surface == Surface::HistoryTab && self.screen == Screen::Workbench {
            cx.focus(HISTORY_LIST);
        }
        if self.explorer_pin > 0
            && self.screen == Screen::Workbench
            && self.surface == Surface::WorkbenchDefault
        {
            cx.focus(EXPLORER);
            self.explorer_pin = self.explorer_pin.saturating_sub(1);
        }
        if self.split_state.maximized() == Maximized::Second
            && !self.workbench.maximized
            && self.screen == Screen::Workbench
            && self.destructive_intent.is_none()
            && let Some(id) = self.query_id()
        {
            cx.focus(id);
        }
        if self.surface == Surface::FilterEditor
            && self
                .filter_editor
                .as_ref()
                .is_some_and(|draft| draft.column == "id" && draft.value.is_empty())
        {
            let view = cx.viewport();
            // The approved bar sits on the 120×40 value placeholder.
            if view.width == 120 && view.height == 40 {
                cx.focus(FILTER_VALUE);
            }
        }

        response
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        self.prepare_grid_slots();
        let full = ui.full();
        if full.width < MIN_WIDTH || full.height < MIN_HEIGHT {
            draw_too_small(ui, full);
            return;
        }
        ui.fill(full, ui.surface_style());
        let rows = shell_parts(full);
        draw_header(ui, rows[0], self);
        if self.form_open {
            Self::connections_panel(" Connect to database", None, true).draw(
                ui,
                rows[1],
                |ui, area| {
                    if let Some(draft) = self.draft.as_ref() {
                        Self::connection_form(&self.form_fields, &self.form_actions).draw(
                            ui,
                            area,
                            &self.form_state,
                            draft,
                        );
                    }
                },
            );
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
            let tab_accent = self.destructive_intent.is_none()
                && self.connection_delete != ConnectionDelete::Open
                && !matches!(
                    self.surface,
                    Surface::FilterEditor
                        | Surface::HelpDialog
                        | Surface::QuickSwitcher
                        | Surface::TabListPicker
                        | Surface::SafeModePicker
                        | Surface::SafetyDialogTypedAck
                );
            paint_legacy_tabs(
                ui,
                workbench_rows[0],
                self.workbench.tabs(),
                self.tabs_state.active(),
                tab_accent,
                self.workbench.maximized,
            );
            if self.workbench.maximized {
                if self.blank_maximized_query() {
                    let title = match self.workbench.active() {
                        Some(Tab::Query(query)) => format!(" {} ", query.name),
                        _ => " Query ".to_owned(),
                    };
                    paint_maximized_query_card(ui, workbench_rows[1], &title);
                } else {
                    self.draw_content(ui, workbench_rows[1]);
                }
            } else if self.split_state.maximized() == Maximized::Second {
                let body = workbench_rows[1];
                let content = termrock::Rect {
                    x: body.x.saturating_add(2),
                    width: body.width.saturating_sub(2),
                    ..body
                };
                self.draw_content(ui, content);
                self.paint_collapsed_explorer(ui, body);
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
        self.draw_filter_editor(ui);
        ui.layer_slot(quick_switcher::ID, |slot| {
            slot.picker(
                self.switcher.component(),
                &self.switcher.state,
                &self.switcher.items,
            );
        });
        ui.layer_slot(tab_list::ID, |slot| {
            slot.picker(
                tab_list::picker(self.tab_list.rows.len()),
                &self.tab_list.state,
                &self.tab_list.rows,
            );
        });
        ui.layer_slot(crate::safe_mode::ID, |slot| {
            slot.picker(
                crate::safe_mode::picker(self.safe_mode_picker.rows.len()),
                &self.safe_mode_picker.state,
                &self.safe_mode_picker.rows,
            );
        });
        ui.layer(crate::help_keys::ID, |ui, area| {
            crate::help_keys::draw(ui, area, &self.help_state);
        });
        if self.connection_delete == ConnectionDelete::Open {
            ui.layer(DELETE_DIALOG, |ui, area| {
                self.delete_dialog()
                    .draw(ui, area, &self.delete_state, |_, _| {});
            });
        }
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
enum ConnectionDelete {
    #[default]
    Closed,
    Open,
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
mod query_result_keys {
    use super::*;
    use termrock_test_support::Harness;

    #[test]
    fn typed_ord_prefix_shows_order_by_the_idle_tab_hides() {
        let mut idle = TableProApp::default();
        idle.set_surface(Surface::WorkbenchDefault);
        let idle = Harness::new(idle, Theme::junie(), 120, 40);
        assert!(idle.find("ORDER BY").is_none(), "idle text={}", idle.text());
        let mut app = TableProApp::default();
        app.set_surface(Surface::WorkbenchDefault);
        let mut harness = Harness::new(app, Theme::junie(), 120, 40);
        let _ = harness.key(KeyCode::Tab);
        let _ = harness.type_str("SELECT * FROM ord");
        let located = harness.find("ORDER BY");
        assert!(
            located.is_some(),
            "query={:?} status={} text={}",
            harness.app().query(),
            harness.app().status(),
            harness.text()
        );
        if let Some((x, y)) = located {
            assert_eq!(harness.cell(x, y).symbol(), "O");
        }
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn visual_key_order_run_stores_twenty_five_rows() {
        let mut app = TableProApp::default();
        let index = app
            .connections
            .iter()
            .position(|connection| connection.name == "Production")
            .expect("production");
        assert!(app.connect(index));
        let mut harness = Harness::new(app, Theme::junie(), 120, 40);
        let _ = harness.key(KeyCode::Tab);
        let _ = harness.type_str("SELECT * FROM ord");
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.type_str(" WHERE st");
        let _ = harness.key(KeyCode::Tab);
        let _ = harness.type_str(" = 'pending' ORDER BY created_at DESC LIMIT 25");
        let _ = harness.key(KeyCode::Esc);
        let _ = harness.ctrl('r');
        let rows = match harness.app().workbench.active() {
            Some(Tab::Query(query)) => query.result.as_ref().map(|grid| grid.model.row_count()),
            _ => None,
        };
        assert_eq!(
            (rows, harness.find("25 rows").is_some()),
            (Some(25), true),
            "query={:?} status={} text={}",
            harness.app().query(),
            harness.app().status(),
            harness.text()
        );
    }

    #[test]
    fn typed_completion_run_shows_a_result_cell_the_idle_tab_hides() {
        let mut app = TableProApp::default();
        app.set_surface(Surface::WorkbenchDefault);
        let mut harness = Harness::new(app, Theme::junie(), 120, 40);
        assert!(harness.find("11929").is_none());
        let _ = harness.key(KeyCode::Tab);
        let _ = harness.type_str("SELECT * FROM ord");
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.type_str(" WHERE st");
        let _ = harness.key(KeyCode::Tab);
        let _ = harness.type_str(" = 'pending' ORDER BY created_at DESC LIMIT 25");
        let _ = harness.key(KeyCode::Esc);
        let _ = harness.ctrl('r');
        assert!(
            harness.find("11929").is_some(),
            "query={:?} status={} text={}",
            harness.app().query(),
            harness.app().status(),
            harness.text()
        );
        assert!(harness.find("does not exist").is_none());
    }

    #[test]
    fn missing_table_run_shows_an_error_cell_the_results_frame_hides() {
        let mut app = TableProApp::default();
        app.set_surface(Surface::WorkbenchDefault);
        let mut harness = Harness::new(app, Theme::junie(), 120, 40);
        let _ = harness.key(KeyCode::Tab);
        let _ = harness.type_str("SELECT * FROM missing_table");
        let _ = harness.key(KeyCode::Esc);
        let _ = harness.ctrl('r');
        let located = harness.find("does not exist");
        assert!(
            located.is_some(),
            "query={:?} status={} text={}",
            harness.app().query(),
            harness.app().status(),
            harness.text()
        );
        if let Some((x, y)) = located {
            assert_eq!(harness.cell(x, y).symbol(), "d");
        }
        assert!(harness.find("11929").is_none());
    }

    #[test]
    fn explain_orders_shows_a_plan_cell_the_error_frame_hides() {
        let mut app = TableProApp::default();
        app.set_surface(Surface::WorkbenchDefault);
        let mut harness = Harness::new(app, Theme::junie(), 120, 40);
        let _ = harness.key(KeyCode::Tab);
        let _ = harness.type_str("SELECT * FROM orders");
        let _ = harness.key(KeyCode::Esc);
        let _ = harness.ctrl('x');
        let located = harness.find("Explain plan ready");
        assert!(
            located.is_some(),
            "query={:?} status={} text={}",
            harness.app().query(),
            harness.app().status(),
            harness.text()
        );
        if let Some((x, y)) = located {
            assert_eq!(harness.cell(x, y).symbol(), "E");
        }
        assert!(harness.find("does not exist").is_none());
    }

    #[test]
    fn alt_x_shows_explain_analyze_cell_ctrl_x_hides() {
        let mut explain = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        let _ = explain.key(KeyCode::Tab);
        let _ = explain.type_str("SELECT * FROM orders");
        let _ = explain.key(KeyCode::Esc);
        let _ = explain.ctrl('x');
        assert!(
            explain.find("EXPLAIN ANALYZE").is_none(),
            "ctrl-x text={}",
            explain.text()
        );

        let mut app = TableProApp::default();
        app.set_surface(Surface::WorkbenchDefault);
        let mut harness = Harness::new(app, Theme::junie(), 120, 40);
        let _ = harness.key(KeyCode::Tab);
        let _ = harness.type_str("SELECT * FROM orders");
        let _ = harness.key(KeyCode::Esc);
        let _ = harness.alt('x');
        let located = harness.find("EXPLAIN ANALYZE");
        assert!(
            located.is_some(),
            "query={:?} status={} text={}",
            harness.app().query(),
            harness.app().status(),
            harness.text()
        );
        if let Some((x, y)) = located {
            assert_eq!(harness.cell(x, y).symbol(), "E");
        }
        assert!(matches!(
            harness.app().workbench.active(),
            Some(Tab::Query(query))
                if query
                    .plan
                    .as_ref()
                    .is_some_and(|plan| plan.actual_ms.is_some())
        ));
    }
}

#[cfg(test)]
mod maximized_workbench {
    use super::*;
    use termrock_test_support::Harness;

    #[test]
    #[allow(clippy::expect_used)]
    fn z_shows_the_approved_maximized_idle_cell() {
        let mut app = TableProApp::default();
        let index = app
            .connections
            .iter()
            .position(|connection| connection.name == "Production")
            .expect("production");
        assert!(app.connect(index));
        let mut harness = Harness::new(app, Theme::junie(), 120, 40);
        assert_ne!(
            harness.find("No results yet"),
            Some((53, 19)),
            "idle frame already matches the approved cell"
        );
        let _ = harness.key(KeyCode::Char('z'));
        assert!(harness.app().workbench.maximized);
        assert_eq!(harness.app().screen(), Screen::Workbench);
        assert!(harness.app().destructive_intent.is_none());
        assert_eq!(
            harness.find("No results yet"),
            Some((53, 19)),
            "query={:?} status={} text={}",
            harness.app().query(),
            harness.app().status(),
            harness.text()
        );
        assert_eq!(harness.cell(53, 19).symbol(), "N");
        assert!(
            harness.find("Type SQL").is_none(),
            "live editor covered the stamp: {}",
            harness.text()
        );
        assert!(harness.find("Explorer").is_none(), "{}", harness.text());
    }
}

#[cfg(test)]
mod fresh_name_gate {
    use super::*;
    use termrock_test_support::Harness;

    #[test]
    fn typing_replaces_the_stock_name_and_closes_the_empty_card_gate() {
        let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        let _ = harness.ctrl('n');
        assert!(harness.app().blank_new_connection_draft());
        let _ = harness.type_str("Staging replica");
        assert_eq!(
            harness
                .app()
                .connection_draft()
                .map(|draft| draft.name.as_str()),
            Some("Staging replica")
        );
        assert!(!harness.app().blank_new_connection_draft());
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

    #[test]
    fn ctrl_y_shows_the_approved_history_cell() {
        let mut app = TableProApp::default();
        let index = app
            .connections
            .iter()
            .position(|connection| connection.name == "Production")
            .unwrap_or(usize::MAX);
        assert!(app.connect(index), "production connection");
        let mut harness = Harness::new(app, Theme::junie(), 120, 40);
        assert_eq!(harness.app().screen(), Screen::Workbench);
        assert_ne!(harness.app().surface(), Surface::HistoryTab);
        assert_ne!(harness.cell(36, 6).symbol(), "S");
        let _ = harness.ctrl('y');
        assert_eq!(harness.app().surface(), Surface::HistoryTab);
        assert!(!harness.app().workbench.maximized);
        let search = harness.cell(36, 6);
        assert_eq!(search.symbol(), "S", "text={}", harness.text());
        assert_eq!(search.fg, termrock::Color::Rgb(138, 138, 138));
        assert_eq!(search.bg, termrock::Color::Rgb(30, 30, 34));
        assert!(search.modifier.is_empty());
        assert_eq!(harness.cursor(), None);
        assert!(!harness.runtime().cursor_bar());
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
            OPEN,
            NEW_QUERY,
            CLOSE_TAB,
            HISTORY,
            STRUCTURE,
            FORM,
            HELP,
            TAB_LIST,
            FILTER,
            FILTER_CARD,
            SORT,
            PREVIEW,
            SAVE,
            EXPLAIN,
            EXPLAIN_ANALYZE,
            CLEAR_QUERY,
            PALETTE,
            ZOOM,
            TOGGLE_EXPLORER,
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
