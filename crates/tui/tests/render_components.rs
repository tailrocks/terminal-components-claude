//! The component digest matrix — `render::components::<component>::<state>`
//! (`COMPONENT_ARCHITECTURE.md` §16.3, Slice 2 acceptance condition 5).
//!
//! Forty components × eight states × `{junie, paper}` × `{truecolor, mono}` ×
//! `{120×40, 40×10}` = 2560 checked-in digest lines in
//! `tests/baselines/components.txt`. The theme, colour level and size are part
//! of the baseline **key**, so one test function owns eight lines and a
//! regression names the exact cell of the matrix that moved.
//!
//! **File placement.** §16.3 puts this matrix in `tests/render.rs`. It is a
//! separate target because `tests/render.rs` is owned by the foundations work
//! package and this file by the components one; the test *paths*
//! (`render::components::…`) are what §16.3 and the acceptance condition name,
//! and they are identical either way. Merge the two targets at Slice 5 if the
//! split stops paying for itself.
//!
//! Runtime-owned states are injected into exactly one target by
//! [`Ui::reference`]; selected, editing and disabled remain real component
//! state or props. A digest is therefore a pure function of
//! `(props, theme, colour, size)` with no live focus, pointer or frame counter.
//! `empty` is expressed as the absence of content.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]

use ratatui_core::style::Modifier;
use tui_next::{
    Action, ActionKey, App, Brand, Button, CellPos, CellRef, Checkbox, ChipBar, ChipBarState,
    Chord, CodeEditor, CodeEditorState, ColorLevel, Column, ColumnKey, Completion, CompletionState,
    ContextMenu, Cx, Dialog, DialogState, DiffLineKind, DiffRow, DiffSource, DiffView,
    DiffViewState, Empty, EmptyState, Family, Field, FieldKind, FieldMut, FieldRef, FieldSpec,
    FilterList, FilterListState, Form, FormData, FormState, GlyphRole, Grid, GridModel, GridState,
    HelpOverlay, HelpOverlayState, HelpSection, Hint, HintBar, HintLayer, Id, Item, ItemKey,
    KeyCode, KeyHint, List, ListState, Menu, MenuBar, MenuItem, MenuState, Meter, NavList,
    NavListState, Panel, PanelKind, Part, PartRef, Picker, PickerChain, PickerChainState,
    PickerStage, PickerState, Position, ProgressBar, RadioGroup, RadioGroupState, Rect,
    ReferenceState, ReferenceTarget, Response, RowUi, ScrollRegion, ScrollState, Select,
    SelectMode, SelectState, Spinner, SplitAxis, SplitPane, SplitPaneState, StateFlags, Status,
    StatusBar, StatusItem, StepState, Steps, StepsState, Tabs, TabsState, TextArea, TextAreaState,
    TextInput, TextInputState, TextViewport, Theme, Toggle, TooSmall, Tree, TreeNode, TreeState,
    Ui, Variant, ViewportLine, ViewportState, Wizard, WizardState, WizardStep,
};
use tui_next_testing::{Baseline, Harness, Scene};

const BASELINE: Baseline = Baseline::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/baselines/components.txt"
));

const BTN: Id = Id::root("render.button");
const INPUT: Id = Id::root("render.text_input");
const FIELD: Id = Id::root("render.field");
const LIST: Id = Id::root("render.list");
const TABS: Id = Id::root("render.tabs");
const DLG: Id = Id::root("render.dialog");
const SCROLL_REGION: Id = Id::root("render.scroll_region");
const TEXT_AREA: Id = Id::root("render.text_area");
const SELECT: Id = Id::root("render.select");
const RADIO_GROUP: Id = Id::root("render.radio_group");
const CHECKBOX: Id = Id::root("render.checkbox");
const TOGGLE: Id = Id::root("render.toggle");
const CHIP_BAR: Id = Id::root("render.chip_bar");
const STATUS_BAR: Id = Id::root("render.status_bar");
const HINT_BAR: Id = Id::root("render.hint_bar");
const KEY_HINT: Id = Id::root("render.key_hint");
const PROGRESS_BAR: Id = Id::root("render.progress_bar");
const SPINNER: Id = Id::root("render.spinner");
const METER: Id = Id::root("render.meter");
const EMPTY: Id = Id::root("render.empty");
const BRAND: Id = Id::root("render.brand");
const PANEL: Id = Id::root("render.panel");
const SPLIT_PANE: Id = Id::root("render.split_pane");
const TEXT_VIEWPORT: Id = Id::root("render.text_viewport");
const DIFF_VIEW: Id = Id::root("render.diff_view");
const CODE_EDITOR: Id = Id::root("render.code_editor");
const TREE: Id = Id::root("render.tree");
const NAV_LIST: Id = Id::root("render.nav_list");
const STEPS: Id = Id::root("render.steps");
const TOO_SMALL: Id = Id::root("render.too_small");
const GRID: Id = Id::root("render.grid");
const FILTER_LIST: Id = Id::root("render.filter_list");
const PICKER: Id = Id::root("render.picker");
const COMPLETION: Id = Id::root("render.completion");
const FORM: Id = Id::root("render.form");
const CONTEXT_MENU: Id = Id::root("render.context_menu");
const HELP_OVERLAY: Id = Id::root("render.help_overlay");
const MENU_BAR: Id = Id::root("render.menu_bar");
const PICKER_CHAIN: Id = Id::root("render.picker_chain");
const WIZARD: Id = Id::root("render.wizard");
const SCROLL_FIXTURE_ROWS: usize = 80;

/// `(label, meta)` rows for the list.
const ROWS: [(&str, &str); 6] = [
    ("Ada Lovelace", "analyst"),
    ("Grace Hopper", "rear admiral"),
    ("Alan Turing", "logician"),
    ("Edsger Dijkstra", "professor"),
    ("Barbara Liskov", "professor"),
    ("Ken Thompson", "engineer"),
];

const TAB_LABELS: [&str; 5] = ["General", "Mounts", "Roles", "Audit", "Advanced"];

const VIEWPORT_LINES: [ViewportLine<'static>; 8] = [
    ViewportLine::Plain("2026-09-04 10:00 connected"),
    ViewportLine::Plain("2026-09-04 10:01 loading workspace"),
    ViewportLine::Plain("2026-09-04 10:02 indexed 128 files"),
    ViewportLine::Plain("2026-09-04 10:03 running checks"),
    ViewportLine::Plain("2026-09-04 10:04 check 1 passed"),
    ViewportLine::Plain("2026-09-04 10:05 check 2 passed"),
    ViewportLine::Plain("2026-09-04 10:06 check 3 passed"),
    ViewportLine::Plain("2026-09-04 10:07 ready"),
];

const DIFF_ROWS: [DiffRow<'static>; 5] = [
    DiffRow::Hunk {
        old_start: 12,
        new_start: 12,
    },
    DiffRow::Line {
        kind: DiffLineKind::Context,
        text: "fn retry() {",
    },
    DiffRow::Line {
        kind: DiffLineKind::Remove,
        text: "    let attempts = 3;",
    },
    DiffRow::Line {
        kind: DiffLineKind::Add,
        text: "    let attempts = 5;",
    },
    DiffRow::Line {
        kind: DiffLineKind::Context,
        text: "}",
    },
];

struct RenderDiff;

impl DiffSource for RenderDiff {
    fn revision(&self) -> u64 {
        1
    }
    fn path(&self) -> &'static str {
        "src/retry.rs"
    }
    fn status_marker(&self) -> &'static str {
        "M"
    }
    fn status_label(&self) -> &'static str {
        "modified"
    }
    fn row_count(&self) -> usize {
        DIFF_ROWS.len()
    }
    fn row(&self, index: usize) -> Option<DiffRow<'_>> {
        DIFF_ROWS.get(index).copied()
    }
}

#[derive(Clone, Copy)]
struct RenderTreeNode {
    key: ItemKey,
    label: &'static str,
    meta: &'static str,
    node: TreeNode,
}

const TREE_ROWS: [RenderTreeNode; 6] = [
    RenderTreeNode {
        key: ItemKey::num(1),
        label: "Workspace",
        meta: "root",
        node: TreeNode::parent(0).keyed(ItemKey::num(1)),
    },
    RenderTreeNode {
        key: ItemKey::num(2),
        label: "crates",
        meta: "dir",
        node: TreeNode::parent(1).keyed(ItemKey::num(2)),
    },
    RenderTreeNode {
        key: ItemKey::num(3),
        label: "tui",
        meta: "dir",
        node: TreeNode::leaf(2).keyed(ItemKey::num(3)),
    },
    RenderTreeNode {
        key: ItemKey::num(4),
        label: "tests",
        meta: "dir",
        node: TreeNode::leaf(2).keyed(ItemKey::num(4)),
    },
    RenderTreeNode {
        key: ItemKey::num(5),
        label: "Cargo.toml",
        meta: "file",
        node: TreeNode::leaf(1).keyed(ItemKey::num(5)),
    },
    RenderTreeNode {
        key: ItemKey::num(6),
        label: "README.md",
        meta: "file",
        node: TreeNode::leaf(1).keyed(ItemKey::num(6)),
    },
];

const GRID_COLUMNS: [Column<'static>; 2] = [
    Column::new(ColumnKey::num(1), "Name"),
    Column::new(ColumnKey::num(2), "Role"),
];

const DIALOG_ACTIONS: [Action<'static>; 2] = [
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ActionKey::CONFIRM, "OK"),
];

const RENDER_ITEMS: [Item<'static>; 3] = [
    Item::new(ItemKey::num(1), "Ada Lovelace").detail("analyst"),
    Item::new(ItemKey::num(2), "Grace Hopper").detail("rear admiral"),
    Item::new(ItemKey::num(3), "Alan Turing").detail("logician"),
];

struct RenderFormData {
    name: String,
    enabled: bool,
    disabled: bool,
}

impl FormData for RenderFormData {
    fn value(&self, id: Id) -> FieldRef<'_> {
        if id == INPUT {
            FieldRef::Text(&self.name)
        } else {
            FieldRef::Flag(self.enabled)
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        if id == INPUT {
            FieldMut::Text(&mut self.name)
        } else {
            FieldMut::Flag(&mut self.enabled)
        }
    }

    fn disabled(&self, _id: Id) -> bool {
        self.disabled
    }
}

/// The eight states the matrix renders.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum St {
    Default,
    Focused,
    Hovered,
    Pressed,
    Disabled,
    Selected,
    Editing,
    /// Not a state but the absence of content: no rows, no tabs, no value.
    Empty,
}

impl St {
    /// The runtime-owned reference state; semantic states use real data.
    const fn reference_state(self) -> Option<ReferenceState> {
        match self {
            St::Focused => Some(ReferenceState::FOCUSED.union(ReferenceState::FOCUS_VISIBLE)),
            St::Hovered => Some(ReferenceState::HOVERED),
            St::Pressed => Some(ReferenceState::PRESSED.union(ReferenceState::FOCUSED)),
            St::Default | St::Disabled | St::Selected | St::Editing | St::Empty => None,
        }
    }

    const fn is_empty(self) -> bool {
        matches!(self, St::Empty)
    }

    /// The `::<state>` half of the baseline key, matching `matrix!`'s
    /// `stringify!`d function names.
    const fn name(self) -> &'static str {
        match self {
            St::Default => "default",
            St::Focused => "focused",
            St::Hovered => "hovered",
            St::Pressed => "pressed",
            St::Disabled => "disabled",
            St::Selected => "selected",
            St::Editing => "editing",
            St::Empty => "empty",
        }
    }
}

/// The forty components the matrix covers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Comp {
    Button,
    TextInput,
    Field,
    List,
    Tabs,
    Dialog,
    ScrollRegion,
    TextArea,
    Select,
    RadioGroup,
    Checkbox,
    Toggle,
    ChipBar,
    StatusBar,
    HintBar,
    KeyHint,
    ProgressBar,
    Spinner,
    Meter,
    Empty,
    Brand,
    Panel,
    SplitPane,
    TextViewport,
    DiffView,
    CodeEditor,
    Tree,
    NavList,
    Steps,
    TooSmall,
    Grid,
    FilterList,
    Picker,
    Completion,
    Form,
    ContextMenu,
    HelpOverlay,
    MenuBar,
    PickerChain,
    Wizard,
}

impl Comp {
    /// Every component the matrix registers, with the `render::components::`
    /// prefix's middle segment.
    ///
    /// Hand-written, and cross-checked against the baseline's own line count
    /// by `theme::readiness_states_are_digest_distinct`: a component added to
    /// `matrix!` but not here makes the baseline hold 64 more digest lines
    /// than this list accounts for, and that check fails.
    const ALL: [(&'static str, Comp); 40] = [
        ("button", Comp::Button),
        ("text_input", Comp::TextInput),
        ("field", Comp::Field),
        ("list", Comp::List),
        ("tabs", Comp::Tabs),
        ("dialog", Comp::Dialog),
        ("scroll_region", Comp::ScrollRegion),
        ("text_area", Comp::TextArea),
        ("select", Comp::Select),
        ("radio_group", Comp::RadioGroup),
        ("checkbox", Comp::Checkbox),
        ("toggle", Comp::Toggle),
        ("chip_bar", Comp::ChipBar),
        ("status_bar", Comp::StatusBar),
        ("hint_bar", Comp::HintBar),
        ("key_hint", Comp::KeyHint),
        ("progress_bar", Comp::ProgressBar),
        ("spinner", Comp::Spinner),
        ("meter", Comp::Meter),
        ("empty", Comp::Empty),
        ("brand", Comp::Brand),
        ("panel", Comp::Panel),
        ("split_pane", Comp::SplitPane),
        ("text_viewport", Comp::TextViewport),
        ("diff_view", Comp::DiffView),
        ("code_editor", Comp::CodeEditor),
        ("tree", Comp::Tree),
        ("nav_list", Comp::NavList),
        ("steps", Comp::Steps),
        ("too_small", Comp::TooSmall),
        ("grid", Comp::Grid),
        ("filter_list", Comp::FilterList),
        ("picker", Comp::Picker),
        ("completion", Comp::Completion),
        ("form", Comp::Form),
        ("context_menu", Comp::ContextMenu),
        ("help_overlay", Comp::HelpOverlay),
        ("menu_bar", Comp::MenuBar),
        ("picker_chain", Comp::PickerChain),
        ("wizard", Comp::Wizard),
    ];

    const fn id(self) -> Id {
        match self {
            Comp::Button => BTN,
            Comp::TextInput => INPUT,
            Comp::Field => FIELD,
            Comp::List => LIST,
            Comp::Tabs => TABS,
            Comp::Dialog => DLG,
            Comp::ScrollRegion => SCROLL_REGION,
            Comp::TextArea => TEXT_AREA,
            Comp::Select => SELECT,
            Comp::RadioGroup => RADIO_GROUP,
            Comp::Checkbox => CHECKBOX,
            Comp::Toggle => TOGGLE,
            Comp::ChipBar => CHIP_BAR,
            Comp::StatusBar => STATUS_BAR,
            Comp::HintBar => HINT_BAR,
            Comp::KeyHint => KEY_HINT,
            Comp::ProgressBar => PROGRESS_BAR,
            Comp::Spinner => SPINNER,
            Comp::Meter => METER,
            Comp::Empty => EMPTY,
            Comp::Brand => BRAND,
            Comp::Panel => PANEL,
            Comp::SplitPane => SPLIT_PANE,
            Comp::TextViewport => TEXT_VIEWPORT,
            Comp::DiffView => DIFF_VIEW,
            Comp::CodeEditor => CODE_EDITOR,
            Comp::Tree => TREE,
            Comp::NavList => NAV_LIST,
            Comp::Steps => STEPS,
            Comp::TooSmall => TOO_SMALL,
            Comp::Grid => GRID,
            Comp::FilterList => FILTER_LIST,
            Comp::Picker => PICKER,
            Comp::Completion => COMPLETION,
            Comp::Form => FORM,
            Comp::ContextMenu => CONTEXT_MENU,
            Comp::HelpOverlay => HELP_OVERLAY,
            Comp::MenuBar => MENU_BAR,
            Comp::PickerChain => PICKER_CHAIN,
            Comp::Wizard => WIZARD,
        }
    }

    fn reference_target(self, state: St) -> Option<ReferenceTarget> {
        let reference = state.reference_state()?;
        let owner = match (self, state) {
            (Comp::Dialog, _) => DLG.part(Part::ACTIONS).index(0),
            (Comp::Form, St::Pressed) => CHECKBOX,
            (Comp::Form, _) => INPUT,
            _ => self.id(),
        };
        let part = match (self, state) {
            (
                Comp::Form,
                St::Default
                | St::Focused
                | St::Hovered
                | St::Disabled
                | St::Selected
                | St::Editing
                | St::Empty,
            ) => PartRef::of(Part::FIELD),
            (Comp::TextInput | Comp::Field | Comp::TextArea | Comp::Select, _) => {
                PartRef::of(Part::FIELD)
            }
            (Comp::List | Comp::NavList | Comp::Steps, _) => {
                PartRef::item(Part::ROW, ItemKey::text("Ada Lovelace"))
            }
            (Comp::Tabs, _) => PartRef::item(Part::TAB, ItemKey::text("General")),
            (Comp::ScrollRegion, _) => PartRef::of(Part::THUMB),
            (Comp::RadioGroup, _) => PartRef::item(Part::ROW, ItemKey::text("Ada Lovelace")),
            (Comp::ChipBar, _) => PartRef::item(Part::LABEL, ItemKey::text("Ada Lovelace")),
            (Comp::StatusBar | Comp::PickerChain, _) => PartRef::item(Part::LABEL, ItemKey::num(1)),
            (Comp::SplitPane, _) => PartRef::of(Part::SEAM),
            (Comp::TextViewport | Comp::DiffView | Comp::CodeEditor, _) => PartRef::of(Part::TEXT),
            (Comp::Tree, _) => PartRef::item(Part::ROW, ItemKey::num(1)),
            (Comp::Grid, _) => PartRef::item(Part::CELL, ItemKey::text("Ada Lovelace")),
            (Comp::FilterList | Comp::Picker | Comp::Completion, _) => {
                PartRef::item(Part::ROW, ItemKey::num(1))
            }
            (Comp::ContextMenu, _) => PartRef::item(Part::ROW, ItemKey::index(0)),
            (Comp::MenuBar, _) => PartRef::item(Part::TITLE, ItemKey::index(0)),
            (Comp::Wizard, _) => PartRef::item(Part::LABEL, ItemKey::num(2)),
            (Comp::Form, St::Pressed)
            | (
                Comp::Button
                | Comp::Dialog
                | Comp::Checkbox
                | Comp::Toggle
                | Comp::HintBar
                | Comp::KeyHint
                | Comp::ProgressBar
                | Comp::Spinner
                | Comp::Meter
                | Comp::Empty
                | Comp::Brand
                | Comp::Panel
                | Comp::TooSmall
                | Comp::HelpOverlay,
                _,
            ) => PartRef::of(Part::CONTAINER),
        };
        Some(ReferenceTarget::new(owner, reference).part(part))
    }

    /// The [`Status`] this fixture hands `comp` in state `st`, or `None` when
    /// the fixture drives it with no status prop at all.
    ///
    /// **This is the single declaration of which components are
    /// status-driven.** The four `draw_*` functions that paint a status take
    /// it as an argument rather than calling `status_for` themselves, so the
    /// set `theme::readiness_states_are_digest_distinct` iterates cannot drift
    /// away from the set the fixture actually drives. The match is exhaustive
    /// with no `_` arm, so a twenty-first component has to be classified here
    /// before it compiles.
    ///
    /// `Empty` is deliberately `None`: it is driven by an [`EmptyState`]
    /// variant and re-derives its own `BUSY`/`LOADING`/`ERROR` flags, so it
    /// carries no `Status` prop for the property to be about.
    const fn status_prop(self, st: St) -> Option<Status> {
        match self {
            Comp::StatusBar | Comp::HintBar | Comp::ProgressBar | Comp::Meter => {
                Some(status_for(st))
            }
            Comp::Button
            | Comp::TextInput
            | Comp::Field
            | Comp::List
            | Comp::Tabs
            | Comp::Dialog
            | Comp::ScrollRegion
            | Comp::TextArea
            | Comp::Select
            | Comp::RadioGroup
            | Comp::Checkbox
            | Comp::Toggle
            | Comp::ChipBar
            | Comp::KeyHint
            | Comp::Spinner
            | Comp::Empty
            | Comp::Brand
            | Comp::Panel
            | Comp::SplitPane
            | Comp::TextViewport
            | Comp::DiffView
            | Comp::CodeEditor
            | Comp::Tree
            | Comp::NavList
            | Comp::Steps
            | Comp::TooSmall
            | Comp::Grid
            | Comp::FilterList
            | Comp::Picker
            | Comp::Completion
            | Comp::Form
            | Comp::ContextMenu
            | Comp::HelpOverlay
            | Comp::MenuBar
            | Comp::Wizard => None,
            Comp::PickerChain => Some(status_for(st)),
        }
    }
}

fn row_key(r: &(&str, &str)) -> ItemKey {
    ItemKey::text(r.0)
}

fn row_paint(r: &(&str, &str), u: &mut RowUi<'_>) {
    u.label(r.0);
    u.meta(r.1);
}

fn disabled_row(_: &(&str, &str)) -> bool {
    true
}

fn tab_paint(r: &&'static str, u: &mut RowUi<'_>) {
    u.label(r);
}

fn tab_key(r: &&'static str) -> ItemKey {
    ItemKey::text(r)
}

fn render_tree_key(node: &RenderTreeNode) -> ItemKey {
    node.key
}

fn render_tree_node(node: &RenderTreeNode) -> TreeNode {
    node.node
}

fn render_tree_row(node: &RenderTreeNode, ui: &mut RowUi<'_>) {
    ui.label(node.label);
    ui.meta(node.meta);
}

fn render_nav_section<'a>(row: &'a (&'static str, &'static str)) -> &'a str {
    if row.0 == "Ada Lovelace" {
        "Core"
    } else {
        "People"
    }
}

fn render_nav_icon<'a>(_row: &'a (&'static str, &'static str)) -> &'a str {
    "›"
}

fn render_nav_badge<'a>(row: &'a (&'static str, &'static str)) -> Option<&'a str> {
    (row.0 == "Ada Lovelace").then_some("3")
}

fn render_step(row: &(&str, &str)) -> StepState {
    match row.0 {
        "Ada Lovelace" => StepState::Done,
        "Grace Hopper" => StepState::Running,
        "Alan Turing" => StepState::Failed,
        "Edsger Dijkstra" => StepState::Blocked,
        "Barbara Liskov" => StepState::Skipped,
        _ => StepState::Queued,
    }
}

struct RenderGridModel<'a> {
    rows: &'a [(&'static str, &'static str)],
}

impl GridModel for RenderGridModel<'_> {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        self.rows
            .get(row)
            .map_or(ItemKey::index(row), |item| ItemKey::text(item.0))
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let row = self.rows.get(row)?;
        match col {
            0 => Some(CellRef::new(row.0)),
            1 => Some(CellRef::new(row.1)),
            _ => None,
        }
    }
}

#[derive(Default)]
struct SelectedGridFixture {
    state: GridState,
}

impl App for SelectedGridFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let model = RenderGridModel { rows: &ROWS };
        Grid::new(GRID, &GRID_COLUMNS)
            .select_mode(SelectMode::Multi)
            .update(cx, &mut self.state, &model)
            .map_action(|_| ())
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let model = RenderGridModel { rows: &ROWS };
        Grid::new(GRID, &GRID_COLUMNS)
            .select_mode(SelectMode::Multi)
            .draw(ui, ui.full(), &self.state, &model);
    }
}

fn selected_grid_state() -> GridState {
    let mut harness = Harness::new(SelectedGridFixture::default(), Theme::junie(), 40, 10);
    assert!(harness.tab_to(GRID));
    let _ = harness.key(KeyCode::Char(' '));
    let state = harness.app().state.clone();
    assert!(
        state
            .selected_rows()
            .contains(ItemKey::text("Ada Lovelace"))
    );
    state
}

fn text_input(id: Id, st: St) -> TextInput<'static> {
    let mut t = TextInput::new(id).placeholder("Type a name");
    if !st.is_empty() {
        t = t.value("Ada Lovelace");
    }
    t.disabled(matches!(st, St::Disabled))
}

fn text_area(st: St) -> TextArea<'static> {
    let mut t = TextArea::new(TEXT_AREA, 4).placeholder("Type a note");
    if !st.is_empty() {
        t = t.value("Ada Lovelace\nanalyst");
    }
    t.disabled(matches!(st, St::Disabled))
}

/// The readiness the fixture reports for each matrix state. Reached only
/// through [`Comp::status_prop`], so the mapping and the set of components it
/// applies to are declared in one place.
const fn status_for(st: St) -> Status {
    match st {
        St::Pressed => Status::Busy,
        St::Editing => Status::Loading,
        St::Disabled => Status::Error,
        St::Default | St::Focused | St::Hovered | St::Selected | St::Empty => Status::Ready,
    }
}

fn empty_surface(st: St) -> EmptyState<'static> {
    match st {
        St::Empty => EmptyState::Empty {
            title: "",
            hint: None,
        },
        St::Editing => EmptyState::Loading { label: "Loading" },
        St::Disabled => EmptyState::Error {
            message: "Unable to load",
            detail: Some("Try again"),
        },
        St::Default | St::Focused | St::Hovered | St::Pressed | St::Selected => EmptyState::Empty {
            title: "No results",
            hint: Some("Try a different filter"),
        },
    }
}

const STATUS_LEFT: [StatusItem<'static>; 2] = [
    StatusItem::new("Workspace").strong(),
    StatusItem::new("main").key(ItemKey::num(1)),
];
const STATUS_CENTER: [StatusItem<'static>; 1] = [StatusItem::new("Ready")];
const STATUS_RIGHT: [StatusItem<'static>; 1] = [StatusItem::new("0 changes").key(ItemKey::num(2))];

/// The `(label, meta)` rows a collection renders in state `st`: `St::Empty` is
/// the absence of content, every other state gets the full set.
fn rows_for(st: St) -> &'static [(&'static str, &'static str)] {
    if st.is_empty() { &[] } else { &ROWS }
}

/// The tab labels for state `st`.
fn tabs_for(st: St) -> &'static [&'static str] {
    if st.is_empty() { &[] } else { &TAB_LABELS }
}

fn draw_button(st: St, ui: &mut Ui<'_>, area: Rect) {
    let label = if st.is_empty() { "" } else { "Run task" };
    let mut button = Button::new(BTN, label)
        .variant(Variant::PRIMARY)
        .disabled(matches!(st, St::Disabled));
    if matches!(st, St::Selected) {
        button = button.checked(true);
    }
    button.draw(ui, area);
}

fn draw_text_input(st: St, ui: &mut Ui<'_>, area: Rect) {
    let mut state = TextInputState::default();
    if matches!(st, St::Editing) {
        state.begin("Ada Lovelace");
    }
    text_input(INPUT, st).draw(ui, area, &state);
}

fn draw_field(st: St, ui: &mut Ui<'_>, area: Rect) {
    let label = if st.is_empty() { "" } else { "Name" };
    let mut f = Field::new(label, text_input(FIELD, st)).required(true);
    if !st.is_empty() {
        f = f.help("The person's display name.");
    }
    let mut state = TextInputState::default();
    if matches!(st, St::Editing) {
        state.begin("Ada Lovelace");
    }
    f.draw(ui, area, &state);
}

fn draw_list(st: St, ui: &mut Ui<'_>, area: Rect) {
    let key: fn(&(&str, &str)) -> ItemKey = row_key;
    let row: fn(&(&str, &str), &mut RowUi<'_>) = row_paint;
    let mut state = ListState::default();
    if matches!(st, St::Focused | St::Pressed) {
        state.set_cursor(0, ItemKey::text("Ada Lovelace"));
    }
    if matches!(st, St::Selected) {
        state.choose(Some(ItemKey::text("Ada Lovelace")));
    }
    let mut list = List::new(LIST).key(key).row(row);
    if matches!(st, St::Disabled) {
        list = list.disabled_item(&disabled_row);
    }
    list.draw(ui, area, &state, rows_for(st));
}

fn draw_tabs(st: St, ui: &mut Ui<'_>, area: Rect) {
    let key: fn(&&'static str) -> ItemKey = tab_key;
    let row: fn(&&'static str, &mut RowUi<'_>) = tab_paint;
    let mut state = TabsState::default();
    if matches!(st, St::Focused | St::Pressed) {
        state.set_cursor(0, ItemKey::text("General"));
    }
    if matches!(st, St::Selected) {
        state.set_active(0, ItemKey::text("General"));
    }
    Tabs::new(TABS)
        .key(key)
        .row(row)
        .closable(true)
        .allow_new(true)
        .draw(ui, area, &state, tabs_for(st));
}

fn draw_dialog(st: St, ui: &mut Ui<'_>, area: Rect) {
    let disabled_actions = DIALOG_ACTIONS.map(|action| action.enabled(false));
    let actions = if matches!(st, St::Disabled) {
        &disabled_actions
    } else {
        &DIALOG_ACTIONS
    };
    let d = if st.is_empty() {
        Dialog::new(DLG).body_rows(0)
    } else {
        Dialog::new(DLG)
            .title("Delete table")
            .description("This cannot be undone. Every row and every index goes with it.")
            .body_rows(0)
            .actions(actions)
            .cancel(ActionKey::CANCEL)
    };
    d.draw(ui, area, &DialogState::default(), |_, _| {});
}

fn draw_scroll_region(st: St, ui: &mut Ui<'_>, area: Rect) {
    let rows = if st.is_empty() {
        0
    } else {
        SCROLL_FIXTURE_ROWS
    };
    let state = ScrollState::default();
    let content = ScrollRegion::new(SCROLL_REGION).draw(ui, area, &state, rows);
    if rows != 0 {
        let style = ui
            .style(
                Family::LIST,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::empty(),
            )
            .style;
        let view = ScrollRegion::view(&state, content, rows);
        for (row, index) in content.rows().zip(view.visible_range()) {
            ui.paint_str(row, &format!("row {index}"), style);
        }
    }
}

fn draw_text_area(st: St, ui: &mut Ui<'_>, area: Rect) {
    let mut state = TextAreaState::default();
    if matches!(st, St::Editing) {
        state.begin("Ada Lovelace\nanalyst");
    }
    text_area(st).draw(ui, area, &state);
}

fn draw_select(st: St, ui: &mut Ui<'_>, area: Rect) {
    let key: fn(&(&str, &str)) -> ItemKey = row_key;
    let row: fn(&(&str, &str), &mut RowUi<'_>) = row_paint;
    let mut state = SelectState::default();
    if !st.is_empty() {
        state.set_value(Some(ItemKey::text("Ada Lovelace")));
    }
    Select::new(SELECT)
        .key(key)
        .row(row)
        .placeholder("Choose a person")
        .popup_rows(5)
        .disabled(matches!(st, St::Disabled))
        .draw(ui, area, &state, rows_for(st));
}

fn draw_radio_group(st: St, ui: &mut Ui<'_>, area: Rect) {
    let key: fn(&(&str, &str)) -> ItemKey = row_key;
    let row: fn(&(&str, &str), &mut RowUi<'_>) = row_paint;
    let mut state = RadioGroupState::default();
    if matches!(st, St::Focused | St::Pressed) {
        state.set_cursor(0, ItemKey::text("Ada Lovelace"));
    }
    let mut radio = RadioGroup::new(RADIO_GROUP)
        .key(key)
        .row(row)
        .disabled(matches!(st, St::Disabled));
    if matches!(st, St::Selected) {
        radio = radio.value(ItemKey::text("Ada Lovelace"));
    }
    radio.draw(ui, area, &state, rows_for(st));
}

fn draw_checkbox(st: St, ui: &mut Ui<'_>, area: Rect) {
    Checkbox::new(CHECKBOX, "Accept terms")
        .checked(matches!(st, St::Selected))
        .disabled(matches!(st, St::Disabled))
        .draw(ui, area);
}

fn draw_toggle(st: St, ui: &mut Ui<'_>, area: Rect) {
    Toggle::new(TOGGLE, "Notifications")
        .on(matches!(st, St::Selected))
        .disabled(matches!(st, St::Disabled))
        .draw(ui, area);
}

fn draw_chip_bar(st: St, ui: &mut Ui<'_>, area: Rect) {
    let key: fn(&(&str, &str)) -> ItemKey = row_key;
    let row: fn(&(&str, &str), &mut RowUi<'_>) = row_paint;
    let mut state = ChipBarState::default();
    if matches!(st, St::Focused | St::Pressed) {
        state.set_cursor(0, ItemKey::text("Ada Lovelace"));
    }
    if matches!(st, St::Selected) {
        state.checked_mut().insert(ItemKey::text("Ada Lovelace"));
    }
    ChipBar::new(CHIP_BAR)
        .key(key)
        .row(row)
        .select_mode(SelectMode::Multi)
        .closable(true)
        .disabled(matches!(st, St::Disabled))
        .draw(ui, area, &state, rows_for(st));
}

fn draw_status_bar(st: St, status: Status, ui: &mut Ui<'_>, area: Rect) {
    let left: &[StatusItem<'static>] = if st.is_empty() { &[] } else { &STATUS_LEFT };
    let center: &[StatusItem<'static>] = if st.is_empty() { &[] } else { &STATUS_CENTER };
    let right: &[StatusItem<'static>] = if st.is_empty() { &[] } else { &STATUS_RIGHT };
    StatusBar::new(STATUS_BAR)
        .left(left)
        .center(center)
        .right(right)
        .status(status)
        .draw(ui, area);
}

fn draw_hint_bar(st: St, status: Status, ui: &mut Ui<'_>, area: Rect) {
    let layer = HintLayer {
        hints: if st.is_empty() {
            Vec::new()
        } else {
            vec![
                Hint {
                    chord: Chord::key(KeyCode::Enter),
                    label: "Open",
                    priority: 80,
                },
                Hint {
                    chord: Chord::key(KeyCode::Esc),
                    label: "Close",
                    priority: 70,
                },
            ]
        },
        badge: if st.is_empty() { None } else { Some("F1") },
        status: if st.is_empty() {
            None
        } else {
            Some(std::borrow::Cow::Borrowed("Ready"))
        },
        centered: false,
    };
    HintBar::new(HINT_BAR, &layer).status(status).draw(ui, area);
}

fn draw_key_hint(_st: St, ui: &mut Ui<'_>, area: Rect) {
    KeyHint::new(KEY_HINT, Chord::key(KeyCode::Enter), "Open").draw(ui, area);
}

fn draw_progress_bar(st: St, status: Status, ui: &mut Ui<'_>, area: Rect) {
    ProgressBar::new(PROGRESS_BAR)
        .label(if st.is_empty() { "" } else { "Uploading" })
        .ratio(if st.is_empty() { 0.0 } else { 0.65 })
        .status(status)
        .draw(ui, area);
}

fn draw_spinner(st: St, ui: &mut Ui<'_>, area: Rect) {
    Spinner::new(SPINNER)
        .label(if st.is_empty() { "" } else { "Working" })
        .frame(1)
        .draw(ui, area);
}

fn draw_meter(st: St, status: Status, ui: &mut Ui<'_>, area: Rect) {
    Meter::new(METER)
        .ratio(if st.is_empty() { 0.0 } else { 0.65 })
        .value(if st.is_empty() { "" } else { "65%" })
        .status(status)
        .draw(ui, area);
}

fn draw_empty(st: St, ui: &mut Ui<'_>, area: Rect) {
    Empty::new(EMPTY, empty_surface(st)).draw(ui, area);
}

fn draw_brand(st: St, ui: &mut Ui<'_>, area: Rect) {
    Brand::new(BRAND, if st.is_empty() { "" } else { "Junie" })
        .tagline(if st.is_empty() { "" } else { "Terminal tools" })
        .draw(ui, area);
}

fn draw_panel(st: St, ui: &mut Ui<'_>, area: Rect) {
    let empty = st.is_empty();
    let mut panel = Panel::new(PANEL)
        .kind(PanelKind::Framed)
        .focused(matches!(st, St::Focused));
    if !empty {
        panel = panel.title("Inspector").meta("read only");
    }
    panel.draw(ui, area, |ui, body| {
        if !empty {
            let style = ui
                .style(
                    Family::PANEL,
                    Variant::DEFAULT,
                    Part::TITLE,
                    StateFlags::empty(),
                )
                .style;
            ui.paint_str(body, "Selected object details", style);
        }
    });
}

fn draw_split_pane(st: St, ui: &mut Ui<'_>, area: Rect) {
    let split = SplitPane::new(SPLIT_PANE, SplitAxis::Horizontal)
        .gap(u16::from(!st.is_empty()))
        .min_first(8)
        .min_second(8)
        .resizable(true);
    split.draw(ui, area, &SplitPaneState::default(), |ui, first, second| {
        if !st.is_empty() {
            let style = ui
                .style(
                    Family::SPLIT,
                    Variant::DEFAULT,
                    Part::SEAM,
                    StateFlags::empty(),
                )
                .style;
            ui.paint_str(first, "Primary pane", style);
            ui.paint_str(second, "Secondary pane", style);
        }
    });
}

fn draw_text_viewport(st: St, ui: &mut Ui<'_>, area: Rect) {
    let lines: &[ViewportLine<'static>] = if st.is_empty() { &[] } else { &VIEWPORT_LINES };
    let mut state = ViewportState::default();
    if matches!(st, St::Selected) {
        state.select(CellPos::new(0, 11), CellPos::new(1, 10));
    }
    TextViewport::new(TEXT_VIEWPORT)
        .wrap(true)
        .draw(ui, area, &state, lines);
}

fn draw_diff_view(st: St, ui: &mut Ui<'_>, area: Rect) {
    let source: Option<&dyn DiffSource> = (!st.is_empty()).then_some(&RenderDiff);
    DiffView::new(DIFF_VIEW, source).draw(ui, area, &DiffViewState::default());
}

fn draw_code_editor(st: St, ui: &mut Ui<'_>, area: Rect) {
    let mut state = CodeEditorState::new(if st.is_empty() {
        ""
    } else {
        "fn retry() {\n  let attempts = 5;\n}"
    });
    if matches!(st, St::Editing) {
        state.begin_edit();
    }
    CodeEditor::new(CODE_EDITOR, area.height.max(2))
        .placeholder("No source")
        .disabled(matches!(st, St::Disabled))
        .draw(ui, area, &state);
}

fn draw_tree(st: St, ui: &mut Ui<'_>, area: Rect) {
    let rows: &[RenderTreeNode] = if st.is_empty() { &[] } else { &TREE_ROWS };
    let mut state = TreeState::default();
    state.expand(ItemKey::num(1));
    state.expand(ItemKey::num(2));
    if matches!(st, St::Focused | St::Pressed) {
        state.set_cursor(0, ItemKey::num(1));
    }
    if matches!(st, St::Selected) {
        state.choose(Some(ItemKey::num(3)));
    }
    Tree::new(TREE)
        .key(render_tree_key as fn(&RenderTreeNode) -> ItemKey)
        .node(&(render_tree_node as fn(&RenderTreeNode) -> TreeNode))
        .row(render_tree_row as fn(&RenderTreeNode, &mut RowUi<'_>))
        .disabled(matches!(st, St::Disabled))
        .draw(ui, area, &state, rows);
}

fn draw_nav_list(st: St, ui: &mut Ui<'_>, area: Rect) {
    let rows = rows_for(st);
    let mut state = NavListState::default();
    if matches!(st, St::Focused | St::Pressed) {
        state.set_cursor(0, ItemKey::text("Ada Lovelace"));
    }
    if matches!(st, St::Selected) {
        state.set_current(Some(ItemKey::text("Ada Lovelace")));
    }
    NavList::new(NAV_LIST)
        .key(row_key as fn(&(&str, &str)) -> ItemKey)
        .row(row_paint as fn(&(&str, &str), &mut RowUi<'_>))
        .section(&render_nav_section)
        .icon(&render_nav_icon)
        .badge(&render_nav_badge)
        .disabled(matches!(st, St::Disabled))
        .draw(ui, area, &state, rows);
}

fn draw_steps(st: St, ui: &mut Ui<'_>, area: Rect) {
    let mut state = StepsState::default();
    if matches!(st, St::Focused | St::Pressed) {
        state.set_cursor(0, ItemKey::text("Ada Lovelace"));
    }
    Steps::navigable(STEPS)
        .key(row_key as fn(&(&str, &str)) -> ItemKey)
        .row(row_paint as fn(&(&str, &str), &mut RowUi<'_>))
        .step(&(render_step as fn(&(&str, &str)) -> StepState))
        .disabled(matches!(st, St::Disabled))
        .draw(ui, area, &state, rows_for(st));
}

fn draw_too_small(_st: St, ui: &mut Ui<'_>, area: Rect) {
    TooSmall::new(TOO_SMALL, "Junie").draw(ui, area);
}

fn draw_grid(st: St, ui: &mut Ui<'_>, area: Rect) {
    let model = RenderGridModel { rows: rows_for(st) };
    let state = if matches!(st, St::Selected) {
        selected_grid_state()
    } else {
        GridState::default()
    };
    Grid::new(GRID, &GRID_COLUMNS)
        .select_mode(SelectMode::Multi)
        .draw(ui, area, &state, &model);
}

fn render_items(st: St) -> ([Item<'static>; 3], usize) {
    let items = RENDER_ITEMS.map(|item| item.disabled(matches!(st, St::Disabled)));
    let len = if st.is_empty() { 0 } else { items.len() };
    (items, len)
}

fn draw_filter_list(st: St, ui: &mut Ui<'_>, area: Rect) {
    let (items, len) = render_items(st);
    let mut state = FilterListState::default();
    if matches!(st, St::Focused | St::Pressed) {
        state.set_cursor(0, ItemKey::num(1));
    }
    if matches!(st, St::Selected) {
        state.set_selected(Some(ItemKey::num(1)));
    }
    FilterList::new(FILTER_LIST).draw(ui, area, &state, &items[..len]);
}

fn draw_picker(st: St, ui: &mut Ui<'_>, area: Rect) {
    let (items, len) = render_items(st);
    let mut state = PickerState::default();
    if matches!(st, St::Focused | St::Pressed) {
        state.set_cursor(0, ItemKey::num(1));
    }
    if matches!(st, St::Selected) {
        state.set_selected(Some(ItemKey::num(1)));
    }
    Picker::new(PICKER).draw(ui, area, &state, &items[..len]);
}

fn draw_completion(st: St, ui: &mut Ui<'_>, area: Rect) {
    let (items, len) = render_items(st);
    let mut state = CompletionState::default();
    if matches!(st, St::Focused | St::Pressed) {
        state.set_cursor(0, ItemKey::num(1));
    }
    if matches!(st, St::Selected) {
        state.set_selected(Some(ItemKey::num(1)));
    }
    Completion::new(COMPLETION).draw(ui, area, &state, &items[..len]);
}

fn draw_form(st: St, ui: &mut Ui<'_>, area: Rect) {
    let input = TextInput::new(INPUT);
    let checkbox = Checkbox::new(CHECKBOX, "Enabled");
    let fields = [
        FieldSpec::new(INPUT, "Name", FieldKind::Text(input)),
        FieldSpec::new(CHECKBOX, "", FieldKind::Check(checkbox)),
    ];
    let data = RenderFormData {
        name: if st.is_empty() {
            String::new()
        } else {
            "Ada Lovelace".into()
        },
        enabled: matches!(st, St::Selected),
        disabled: matches!(st, St::Disabled),
    };
    let fields: &[FieldSpec<'_>] = if st.is_empty() { &[] } else { &fields };
    let actions: &[Action<'_>] = if st.is_empty() { &[] } else { &DIALOG_ACTIONS };
    Form::new(FORM, fields)
        .actions(actions)
        .draw(ui, area, &FormState::default(), &data);
}

const RENDER_MENU_ITEMS: [MenuItem<'static>; 2] = [
    MenuItem::new(ActionKey::SAVE, "Save").chord(Chord::key(KeyCode::Char('s'))),
    MenuItem::new(ActionKey::CLOSE, "Close"),
];
const RENDER_MENUS: [Menu<'static>; 1] = [Menu::new("File", &RENDER_MENU_ITEMS)];

fn draw_context_menu(st: St, ui: &mut Ui<'_>, area: Rect) {
    let disabled_items = RENDER_MENU_ITEMS.map(|item| item.disabled(true));
    let items: &[MenuItem<'_>] = if st.is_empty() {
        &[]
    } else if matches!(st, St::Disabled) {
        &disabled_items
    } else {
        &RENDER_MENU_ITEMS
    };
    let mut state = MenuState::default();
    if matches!(st, St::Focused | St::Pressed) {
        state.set_cursor(0);
    }
    if matches!(st, St::Selected) {
        state.set_selected(Some(0));
    }
    ContextMenu::new(
        CONTEXT_MENU,
        items,
        tui_next::Anchor::Screen(tui_next::ScreenAlign::Center),
    )
    .draw(ui, area, &state);
}

fn draw_menu_bar(st: St, ui: &mut Ui<'_>, area: Rect) {
    let disabled_items = RENDER_MENU_ITEMS.map(|item| item.disabled(true));
    let disabled_menus = [Menu::new("File", &disabled_items)];
    let menus: &[Menu<'_>] = if st.is_empty() {
        &[]
    } else if matches!(st, St::Disabled) {
        &disabled_menus
    } else {
        &RENDER_MENUS
    };
    let mut state = MenuState::default();
    if matches!(st, St::Selected) {
        state.set_selected(Some(0));
    }
    MenuBar::new(MENU_BAR, menus).draw(ui, area, &state);
}

fn draw_help_overlay(st: St, ui: &mut Ui<'_>, area: Rect) {
    let layer = HintLayer {
        hints: vec![Hint {
            chord: Chord::key(KeyCode::Enter),
            label: "Choose",
            priority: 80,
        }],
        ..HintLayer::default()
    };
    let sections = [HelpSection::new("General", &layer)];
    let sections: &[HelpSection<'_>] = if st.is_empty() { &[] } else { &sections };
    HelpOverlay::new(HELP_OVERLAY, "Application", sections).draw(
        ui,
        area,
        &HelpOverlayState::default(),
    );
}

fn draw_picker_chain(st: St, status: Status, ui: &mut Ui<'_>, area: Rect) {
    let stages = [
        PickerStage::new(ItemKey::num(1), "Account"),
        PickerStage::new(ItemKey::num(2), "Vault").status(status),
    ];
    let stages: &[PickerStage<'_>] = if st.is_empty() { &[] } else { &stages };
    let mut state = PickerChainState::default();
    if !st.is_empty() {
        state.enter(ItemKey::num(1));
        if !matches!(st, St::Selected) {
            state.enter(ItemKey::num(2));
        }
        if matches!(st, St::Selected) {
            state.set_selected(Some(ItemKey::num(1)));
        }
    }
    PickerChain::new(PICKER_CHAIN, stages).draw(ui, area, &state);
}

fn draw_wizard(st: St, ui: &mut Ui<'_>, area: Rect) {
    let steps = [
        WizardStep::new(ItemKey::num(1), "Account"),
        WizardStep::new(ItemKey::num(2), "Details"),
        WizardStep::new(ItemKey::num(3), "Review").enabled(!matches!(st, St::Disabled)),
    ];
    let steps: &[WizardStep<'_>] = if st.is_empty() { &[] } else { &steps };
    let mut state = WizardState::<()>::default();
    if matches!(st, St::Focused | St::Pressed) {
        state.set_current(ItemKey::num(2));
    }
    Wizard::new(WIZARD, steps).draw(ui, area, &state);
}

/// Draw `comp` in state `st` into `area`.
///
/// Dispatch only: every arm is one `draw_*` above, so adding a component adds
/// a function and a line rather than growing one function without bound.
fn draw(comp: Comp, st: St, ui: &mut Ui<'_>, area: Rect) {
    // the four status-driven components receive their status as an argument,
    // so `Comp::status_prop` is the only place the fixture decides which
    // components report readiness
    let status = |c: Comp| {
        c.status_prop(st).unwrap_or_else(|| {
            panic!("{c:?} paints a status prop but Comp::status_prop returns None for {st:?}")
        })
    };
    ui.reference(comp.reference_target(st), |ui| match comp {
        Comp::Button => draw_button(st, ui, area),
        Comp::TextInput => draw_text_input(st, ui, area),
        Comp::Field => draw_field(st, ui, area),
        Comp::List => draw_list(st, ui, area),
        Comp::Tabs => draw_tabs(st, ui, area),
        Comp::Dialog => draw_dialog(st, ui, area),
        Comp::ScrollRegion => draw_scroll_region(st, ui, area),
        Comp::TextArea => draw_text_area(st, ui, area),
        Comp::Select => draw_select(st, ui, area),
        Comp::RadioGroup => draw_radio_group(st, ui, area),
        Comp::Checkbox => draw_checkbox(st, ui, area),
        Comp::Toggle => draw_toggle(st, ui, area),
        Comp::ChipBar => draw_chip_bar(st, ui, area),
        Comp::StatusBar => draw_status_bar(st, status(comp), ui, area),
        Comp::HintBar => draw_hint_bar(st, status(comp), ui, area),
        Comp::KeyHint => draw_key_hint(st, ui, area),
        Comp::ProgressBar => draw_progress_bar(st, status(comp), ui, area),
        Comp::Spinner => draw_spinner(st, ui, area),
        Comp::Meter => draw_meter(st, status(comp), ui, area),
        Comp::Empty => draw_empty(st, ui, area),
        Comp::Brand => draw_brand(st, ui, area),
        Comp::Panel => draw_panel(st, ui, area),
        Comp::SplitPane => draw_split_pane(st, ui, area),
        Comp::TextViewport => draw_text_viewport(st, ui, area),
        Comp::DiffView => draw_diff_view(st, ui, area),
        Comp::CodeEditor => draw_code_editor(st, ui, area),
        Comp::Tree => draw_tree(st, ui, area),
        Comp::NavList => draw_nav_list(st, ui, area),
        Comp::Steps => draw_steps(st, ui, area),
        Comp::TooSmall => draw_too_small(st, ui, area),
        Comp::Grid => draw_grid(st, ui, area),
        Comp::FilterList => draw_filter_list(st, ui, area),
        Comp::Picker => draw_picker(st, ui, area),
        Comp::Completion => draw_completion(st, ui, area),
        Comp::Form => draw_form(st, ui, area),
        Comp::ContextMenu => draw_context_menu(st, ui, area),
        Comp::HelpOverlay => draw_help_overlay(st, ui, area),
        Comp::MenuBar => draw_menu_bar(st, ui, area),
        Comp::PickerChain => draw_picker_chain(st, status(comp), ui, area),
        Comp::Wizard => draw_wizard(st, ui, area),
    });
}

/// Render one matrix cell name across both themes, both colour levels and
/// both sizes, and compare every digest against the checked-in baseline.
fn run(name: &'static str, comp: Comp, st: St) {
    for theme in [Theme::junie(), Theme::paper()] {
        for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
            for (w, h) in [(120u16, 40u16), (40, 10)] {
                let mut scene = Scene::new(name, theme.clone(), color, w, h);
                scene.draw(|ui, area| draw(comp, st, ui, area));
                scene.assert_against(&BASELINE);
            }
        }
    }
}

macro_rules! matrix {
    ($comp:ident, $c:expr) => {
        mod $comp {
            use super::super::super::*;
            #[test]
            fn default() {
                run(
                    concat!("render::components::", stringify!($comp), "::default"),
                    $c,
                    St::Default,
                );
            }
            #[test]
            fn focused() {
                run(
                    concat!("render::components::", stringify!($comp), "::focused"),
                    $c,
                    St::Focused,
                );
            }
            #[test]
            fn hovered() {
                run(
                    concat!("render::components::", stringify!($comp), "::hovered"),
                    $c,
                    St::Hovered,
                );
            }
            #[test]
            fn pressed() {
                run(
                    concat!("render::components::", stringify!($comp), "::pressed"),
                    $c,
                    St::Pressed,
                );
            }
            #[test]
            fn disabled() {
                run(
                    concat!("render::components::", stringify!($comp), "::disabled"),
                    $c,
                    St::Disabled,
                );
            }
            #[test]
            fn selected() {
                run(
                    concat!("render::components::", stringify!($comp), "::selected"),
                    $c,
                    St::Selected,
                );
            }
            #[test]
            fn editing() {
                run(
                    concat!("render::components::", stringify!($comp), "::editing"),
                    $c,
                    St::Editing,
                );
            }
            #[test]
            fn empty() {
                run(
                    concat!("render::components::", stringify!($comp), "::empty"),
                    $c,
                    St::Empty,
                );
            }
        }
    };
}

/// A pair of readiness states that is **known** to render identically, and
/// must stay that way: `(component, a, b, colour)`.
///
/// **This list inverts.** A listed pair that turns out to be *distinct* is a
/// failure, not a pass. An exemption is a recorded fact about the design; a
/// stale one is how a gate quietly stops looking at the thing it was written
/// for, which is the defect class this whole matrix has spent the refactor
/// cataloguing.
///
/// The four entries record one fact: `BUSY` and `LOADING` both mean *an
/// operation is in flight*, every one of these components paints the same
/// `design.motion.spinner_frames[0]` affordance for the union
/// `BUSY | LOADING`, and no `junie`/`paper` rule separates them by colour. They
/// diverge only under `mono`, where §11.4 gives `PRESSED` and `EDITING` their
/// own symbols — which is why the colour is part of the key here and why the
/// mono half of each pair is *not* exempt.
const READINESS_COLLISIONS: [(&str, Status, Status, &str); 4] = [
    ("status_bar", Status::Busy, Status::Loading, "truecolor"),
    ("hint_bar", Status::Busy, Status::Loading, "truecolor"),
    ("progress_bar", Status::Busy, Status::Loading, "truecolor"),
    ("meter", Status::Busy, Status::Loading, "truecolor"),
];

/// The four readiness values and the matrix state that carries each.
const READINESS: [(Status, St); 4] = [
    (Status::Ready, St::Default),
    (Status::Busy, St::Pressed),
    (Status::Loading, St::Editing),
    (Status::Error, St::Disabled),
];

const THEME_NAMES: [&str; 2] = ["junie", "paper"];
const COLOR_NAMES: [&str; 2] = ["truecolor", "mono"];
const SIZES: [(u16, u16); 2] = [(120, 40), (40, 10)];

/// `tests/baselines/components.txt` as `key -> hash`, where the key is the
/// line minus its last whitespace field — `Baseline`'s own rule, and the one
/// `xtask bless-guard` uses, so all three agree about what a key is.
fn baseline_entries() -> std::collections::BTreeMap<String, String> {
    let text = std::fs::read_to_string(BASELINE.path())
        .unwrap_or_else(|e| panic!("read {}: {e}", BASELINE.path()));
    let mut out = std::collections::BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, hash)) = line.rsplit_once(' ') {
            out.insert(key.to_owned(), hash.to_owned());
        }
    }
    out
}

/// Every readiness cell of the matrix as `key -> hash`, rendered by the code
/// in this tree. `BLESS=1` writes exactly these values, so this is what the
/// baseline will hold after the next bless.
fn live_entries() -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    for (name, comp) in Comp::ALL {
        if comp.status_prop(St::Default).is_none() {
            continue;
        }
        for (_, st) in READINESS {
            for (theme_name, theme) in [("junie", Theme::junie()), ("paper", Theme::paper())] {
                for (color_name, color) in [
                    ("truecolor", ColorLevel::TrueColor),
                    ("mono", ColorLevel::Mono),
                ] {
                    for (w, h) in SIZES {
                        let mut scene = Scene::new("readiness", theme.clone(), color, w, h);
                        scene.draw(|ui, area| draw(comp, st, ui, area));
                        out.insert(
                            format!(
                                "render::components::{name}::{} {w} {h} {theme_name} {color_name}",
                                st.name()
                            ),
                            format!("{:016x}", scene.digest()),
                        );
                    }
                }
            }
        }
    }
    out
}

/// Every readiness pair of `entries` that violates §49.5, as a report line.
///
/// Separated from the test so the same rule can be run against the recorded
/// baseline, against freshly rendered digests, and against deliberately broken
/// inputs. A missing key is a failure, never a skip.
fn readiness_failures(entries: &std::collections::BTreeMap<String, String>) -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    let mut used = [false; READINESS_COLLISIONS.len()];
    for (name, comp) in Comp::ALL {
        if comp.status_prop(St::Default).is_none() {
            continue;
        }
        for theme in THEME_NAMES {
            for color in COLOR_NAMES {
                for (w, h) in SIZES {
                    for (i, (sa, st_a)) in READINESS.iter().enumerate() {
                        for (sb, st_b) in READINESS.iter().skip(i + 1) {
                            let cell = format!("{w} {h} {theme} {color}");
                            let ka = format!("render::components::{name}::{} {cell}", st_a.name());
                            let kb = format!("render::components::{name}::{} {cell}", st_b.name());
                            let (Some(ha), Some(hb)) = (entries.get(&ka), entries.get(&kb)) else {
                                failures.push(format!(
                                    "  {name} {cell}: no baseline line for `{ka}` or `{kb}`"
                                ));
                                continue;
                            };
                            let exempt = READINESS_COLLISIONS.iter().position(|(c, x, y, l)| {
                                *c == name
                                    && *l == color
                                    && ((x == sa && y == sb) || (x == sb && y == sa))
                            });
                            match exempt {
                                Some(idx) => {
                                    used[idx] = true;
                                    if ha != hb {
                                        failures.push(format!(
                                            "  {name} {cell}: `{sa:?}` and `{sb:?}` are listed in \
                                             READINESS_COLLISIONS as indistinguishable but their \
                                             digests differ ({ha} vs {hb}); the exemption is \
                                             stale — remove it, the list inverts"
                                        ));
                                    }
                                }
                                None => {
                                    if ha == hb {
                                        failures.push(format!(
                                            "  {name} {cell}: `{sa:?}` and `{sb:?}` share the \
                                             digest {ha}; the fixture supplies a different \
                                             `Status` for each, so this cell pins two states of \
                                             one component as one picture"
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    for (i, (name, a, b, color)) in READINESS_COLLISIONS.iter().enumerate() {
        if !used[i] {
            failures.push(format!(
                "  READINESS_COLLISIONS lists ({name}, {a:?}, {b:?}, {color}), which names no cell \
                 of the matrix; a listed pair that is never reached exempts nothing and hides the \
                 next one that is"
            ));
        }
    }
    failures
}

#[derive(Clone, Copy)]
struct VisualExemption {
    component: &'static str,
    state: St,
    color: Option<&'static str>,
    reason: &'static str,
}

impl VisualExemption {
    const fn all(component: &'static str, state: St, reason: &'static str) -> Self {
        Self {
            component,
            state,
            color: None,
            reason,
        }
    }

    const fn color(
        component: &'static str,
        state: St,
        color: &'static str,
        reason: &'static str,
    ) -> Self {
        Self {
            component,
            state,
            color: Some(color),
            reason,
        }
    }

    fn matches(self, component: &str, state: St, color: &str) -> bool {
        self.component == component
            && self.state == state
            && self.color.is_none_or(|expected| expected == color)
    }
}

/// A deliberately closed roster of states that have no corresponding
/// component-owned visual affordance. Every entry is checked in both
/// directions by `visual_state_failures`: a collision without an entry fails,
/// and an entry whose output becomes distinct fails as stale. The one
/// colour-qualified entry records the existing mono fallback on `StatusBar`;
/// its truecolor surface has no focus affordance.
const VISUAL_EXEMPTIONS: &[VisualExemption] = &[
    VisualExemption::color(
        "status_bar",
        St::Focused,
        "truecolor",
        "StatusBar has no focus stop; mono's generic label fallback is incidental",
    ),
    VisualExemption::all(
        "hint_bar",
        St::Focused,
        "HintBar is a passive hint strip, not a focus stop",
    ),
    VisualExemption::all(
        "key_hint",
        St::Focused,
        "KeyHint is a passive hint, not a focus stop",
    ),
    VisualExemption::all("progress_bar", St::Focused, "ProgressBar is display-only"),
    VisualExemption::all("spinner", St::Focused, "Spinner is display-only"),
    VisualExemption::all("meter", St::Focused, "Meter is display-only"),
    VisualExemption::all("empty", St::Focused, "Empty is display-only"),
    VisualExemption::all("brand", St::Focused, "Brand is display-only"),
    VisualExemption::all(
        "scroll_region",
        St::Focused,
        "ScrollRegion owns scrolling, not focus",
    ),
    VisualExemption::all(
        "too_small",
        St::Focused,
        "TooSmall is a passive recovery notice",
    ),
    VisualExemption::all("tabs", St::Disabled, "Tabs has no disabled prop"),
    VisualExemption::all(
        "scroll_region",
        St::Disabled,
        "ScrollRegion has no disabled prop",
    ),
    VisualExemption::all("spinner", St::Disabled, "Spinner has no disabled prop"),
    VisualExemption::all("key_hint", St::Disabled, "KeyHint has no disabled prop"),
    VisualExemption::all("brand", St::Disabled, "Brand has no disabled prop"),
    VisualExemption::all("panel", St::Disabled, "Panel has no disabled prop"),
    VisualExemption::all("split_pane", St::Disabled, "SplitPane has no disabled prop"),
    VisualExemption::all(
        "text_viewport",
        St::Disabled,
        "TextViewport has no disabled prop",
    ),
    VisualExemption::all("diff_view", St::Disabled, "DiffView has no disabled prop"),
    VisualExemption::all("too_small", St::Disabled, "TooSmall has no disabled prop"),
    VisualExemption::all("grid", St::Disabled, "Grid has no whole-grid disabled prop"),
    VisualExemption::all(
        "help_overlay",
        St::Disabled,
        "HelpOverlay has no disabled prop",
    ),
    VisualExemption::all(
        "text_input",
        St::Selected,
        "TextInput has no selection model",
    ),
    VisualExemption::all(
        "field",
        St::Selected,
        "Field delegates to a non-selecting editor",
    ),
    VisualExemption::all(
        "dialog",
        St::Selected,
        "Dialog actions are not a selection model",
    ),
    VisualExemption::all(
        "scroll_region",
        St::Selected,
        "ScrollRegion has no selection model",
    ),
    VisualExemption::all(
        "text_area",
        St::Selected,
        "TextArea has no selection affordance",
    ),
    VisualExemption::all(
        "select",
        St::Selected,
        "Select's value is not a selected row",
    ),
    VisualExemption::all("status_bar", St::Selected, "StatusBar is display-only"),
    VisualExemption::all("hint_bar", St::Selected, "HintBar is display-only"),
    VisualExemption::all("key_hint", St::Selected, "KeyHint is display-only"),
    VisualExemption::all("progress_bar", St::Selected, "ProgressBar is display-only"),
    VisualExemption::all("spinner", St::Selected, "Spinner is display-only"),
    VisualExemption::all("meter", St::Selected, "Meter is display-only"),
    VisualExemption::all("empty", St::Selected, "Empty is display-only"),
    VisualExemption::all("brand", St::Selected, "Brand is display-only"),
    VisualExemption::all("panel", St::Selected, "Panel has no selection model"),
    VisualExemption::all(
        "split_pane",
        St::Selected,
        "SplitPane has no selection model",
    ),
    VisualExemption::all(
        "diff_view",
        St::Selected,
        "DiffView has no selection affordance in this fixture",
    ),
    VisualExemption::all(
        "code_editor",
        St::Selected,
        "CodeEditor selection is an edit range, not a selected component state",
    ),
    VisualExemption::all(
        "steps",
        St::Selected,
        "Steps is a lifecycle rail, not a selector",
    ),
    VisualExemption::all(
        "too_small",
        St::Selected,
        "TooSmall is a passive recovery notice",
    ),
    VisualExemption::all(
        "help_overlay",
        St::Selected,
        "HelpOverlay has no selection model",
    ),
    VisualExemption::all(
        "wizard",
        St::Selected,
        "Wizard tracks progress, not a selected row",
    ),
];

const VISUAL_STATES: [(St, &str); 3] = [
    (St::Focused, "focused"),
    (St::Disabled, "disabled"),
    (St::Selected, "selected"),
];

/// Render the three visual states used by the focused state-distinctness
/// gate. This intentionally does not read or write the baseline file.
fn visual_entries() -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    for (name, comp) in Comp::ALL {
        for (state, state_name) in VISUAL_STATES.into_iter().chain([(St::Default, "default")]) {
            for (theme_name, theme) in [("junie", Theme::junie()), ("paper", Theme::paper())] {
                for (color_name, color) in [
                    ("truecolor", ColorLevel::TrueColor),
                    ("mono", ColorLevel::Mono),
                ] {
                    for (w, h) in SIZES {
                        let mut scene = Scene::new("visual-state", theme.clone(), color, w, h);
                        scene.draw(|ui, area| draw(comp, state, ui, area));
                        out.insert(
                            format!(
                                "render::components::{name}::{state_name} {w} {h} {theme_name} {color_name}"
                            ),
                            format!("{:016x}", scene.digest()),
                        );
                    }
                }
            }
        }
    }
    out
}

/// Return every focused/disabled/selected collision in fresh renders.
fn visual_state_failures(entries: &std::collections::BTreeMap<String, String>) -> Vec<String> {
    let mut failures = Vec::new();
    let mut used = vec![false; VISUAL_EXEMPTIONS.len()];
    for (name, _) in Comp::ALL {
        for (state, state_name) in VISUAL_STATES {
            for theme in THEME_NAMES {
                for color in COLOR_NAMES {
                    for (w, h) in SIZES {
                        let cell = format!("{w} {h} {theme} {color}");
                        let default_key = format!("render::components::{name}::default {cell}");
                        let state_key = format!("render::components::{name}::{state_name} {cell}");
                        let (Some(default), Some(state_digest)) =
                            (entries.get(&default_key), entries.get(&state_key))
                        else {
                            failures.push(format!(
                                "  {name} {state_name} {cell}: missing `{default_key}` or `{state_key}`"
                            ));
                            continue;
                        };
                        let exemption = VISUAL_EXEMPTIONS
                            .iter()
                            .enumerate()
                            .find(|(_, item)| item.matches(name, state, color));
                        match exemption {
                            Some((index, item)) => {
                                used[index] = true;
                                if default != state_digest {
                                    failures.push(format!(
                                        "  {name} {state_name} {cell}: exemption is stale ({}) — {default} != {state_digest}",
                                        item.reason
                                    ));
                                }
                            }
                            None if default == state_digest => failures.push(format!(
                                "  {name} {state_name} {cell}: digest {default} collides with default; add a narrowly reasoned exemption or repair the fixture"
                            )),
                            None => {}
                        }
                    }
                }
            }
        }
        for (first, (state_a, name_a)) in VISUAL_STATES.iter().copied().enumerate() {
            for (state_b, name_b) in VISUAL_STATES.iter().copied().skip(first.saturating_add(1)) {
                for theme in THEME_NAMES {
                    for color in COLOR_NAMES {
                        for (w, h) in SIZES {
                            let cell = format!("{w} {h} {theme} {color}");
                            let key_a = format!("render::components::{name}::{name_a} {cell}");
                            let key_b = format!("render::components::{name}::{name_b} {cell}");
                            let (Some(a), Some(b)) = (entries.get(&key_a), entries.get(&key_b))
                            else {
                                continue;
                            };
                            if a == b {
                                let a_exempt = VISUAL_EXEMPTIONS
                                    .iter()
                                    .any(|item| item.matches(name, state_a, color));
                                let b_exempt = VISUAL_EXEMPTIONS
                                    .iter()
                                    .any(|item| item.matches(name, state_b, color));
                                if !(a_exempt && b_exempt) {
                                    failures.push(format!(
                                        "  {name} {name_a}/{name_b} {cell}: digest {a} collides between visual states; only states with exact exemptions may share a frame"
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    for (index, item) in VISUAL_EXEMPTIONS.iter().enumerate() {
        if !used[index] {
            failures.push(format!(
                "  VISUAL_EXEMPTIONS entry ({}, {:?}, {:?}) names no collision cell: {}",
                item.component, item.state, item.color, item.reason
            ));
        }
    }
    failures
}

mod theme {
    use super::*;

    /// §20.10 item 20 / §49.5. For every component the matrix drives with a
    /// `Status` prop, the `Ready`, `Busy`, `Loading` and `Error` digests are
    /// **pairwise distinct** at every size, theme and colour level.
    ///
    /// **Why this is asserted about the recorded values and not about a fresh
    /// render, and why that generalises.** A first-generation baseline line is
    /// unprotected by any diff-based gate: `xtask bless-guard` compares a tree
    /// with its base, and the *first* recording of a key has no before-image,
    /// so there is nothing for a diff to refuse. That is exactly how §20.10
    /// item 19's 896 lines came to pin `progress_bar::disabled` as a bar that
    /// is in error and paints no error glyph — at `truecolor` all four of its
    /// `::disabled` digests were byte-identical to its own `::default` ones,
    /// although the fixture supplies `Status::Error` to one and `Status::Ready`
    /// to the other. A component declaring `Caps::REPORTS_STATUS` reporting
    /// nothing. **So the properties that pin a first generation have to be
    /// asserted about the values themselves**, read out of the baseline file,
    /// rather than about the movement of those values.
    ///
    /// Item 19's own review could not have caught it and did not claim to: its
    /// six rejection conditions include textual identity only under `mono`,
    /// and these cells are identical at `truecolor`.
    ///
    /// The set of components checked comes from [`Comp::status_prop`], not
    /// from a list written here, and every failing cell is reported rather
    /// than only the first — §49.1 records what reading a single panic as "the
    /// moved set" cost.
    #[test]
    fn readiness_states_are_digest_distinct() {
        let entries = baseline_entries();

        // A component registered by `matrix!` but missing from `Comp::ALL`
        // would be silently unchecked. It cannot be: the baseline holds
        // 8 states × 2 themes × 2 colours × 2 sizes = 64 lines per component.
        let recorded = entries
            .keys()
            .filter(|k| k.starts_with("render::components::"))
            .count();
        assert_eq!(
            recorded,
            Comp::ALL.len() * 64,
            "the baseline holds {recorded} `render::components::` lines but `Comp::ALL` accounts \
             for {} components × 64; a component registered in `matrix!` is missing from \
             `Comp::ALL` and would go unchecked here",
            Comp::ALL.len()
        );

        // the state each readiness value is carried by must be the state the
        // fixture actually maps onto it
        for (status, st) in READINESS {
            assert_eq!(
                Comp::ProgressBar.status_prop(st),
                Some(status),
                "`READINESS` says `{st:?}` carries `{status:?}`, `status_for` disagrees"
            );
        }

        let failures = readiness_failures(&entries);
        assert!(
            failures.is_empty(),
            "{} readiness cell(s) are not digest-distinct (§20.10 item 20, §49.5):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    /// The green half of the demonstration, run without a bless.
    ///
    /// `BLESS=1` writes exactly `Scene::digest()` for each key, so rendering
    /// every status-driven cell here produces the values the re-bless will
    /// record. The property must hold on those. That splits "the code is
    /// right" from "the file is right": this test is green today and
    /// `readiness_states_are_digest_distinct` is red, and the difference
    /// between them is the eight stale `truecolor` lines §49.1 identified.
    #[test]
    fn readiness_distinctness_holds_on_the_digests_the_code_produces_now() {
        let failures = readiness_failures(&live_entries());
        assert!(
            failures.is_empty(),
            "{} readiness cell(s) rendered by the current code are not digest-distinct:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    /// The check can fail in both of its directions, on inputs built here.
    ///
    /// COORDINATION.md: a check that has never been observed red is not
    /// evidence. The inverted exemption is the half that is easy to get wrong,
    /// because a stale entry silently stops the gate looking at a real pair.
    #[test]
    fn readiness_distinctness_fails_on_a_collision_and_on_a_stale_exemption() {
        let mut collided = live_entries();
        let ready = "render::components::meter::default 120 40 junie truecolor";
        let error = "render::components::meter::disabled 120 40 junie truecolor";
        let v = collided.get(ready).cloned().expect("a rendered Ready cell");
        collided.insert(error.to_owned(), v);
        let failures = readiness_failures(&collided);
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(failures[0].contains("Ready"), "{failures:?}");
        assert!(failures[0].contains("Error"), "{failures:?}");

        let mut split = live_entries();
        let busy = "render::components::meter::pressed 120 40 junie truecolor";
        split.insert(busy.to_owned(), "0000000000000000".to_owned());
        let failures = readiness_failures(&split);
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(
            failures[0].contains("the exemption is stale"),
            "{failures:?}"
        );
    }

    /// A missing key is a failure, never a skip: an exemption list that
    /// silently passes over absent cells is a gate that stops looking.
    #[test]
    fn readiness_distinctness_fails_on_a_missing_baseline_line() {
        let mut entries = live_entries();
        entries.remove("render::components::hint_bar::disabled 40 10 paper mono");
        let failures = readiness_failures(&entries);
        assert_eq!(
            failures.len(),
            3,
            "one per pair the missing key is in: {failures:?}"
        );
        assert!(failures[0].contains("no baseline line"), "{failures:?}");
    }

    /// Focus, disabled and selected are checked against fresh frame digests,
    /// not the baseline. A first-generation baseline cannot prove that a
    /// state is visible because it has no before-image; the exact exemptions
    /// above are the only accepted equalities.
    #[test]
    fn focused_disabled_selected_states_are_digest_distinct() {
        let failures = visual_state_failures(&visual_entries());
        assert!(
            failures.is_empty(),
            "{} visual-state collision(s) or stale exemptions:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn visual_state_gate_rejects_collisions_and_stale_exemptions() {
        let mut collided = visual_entries();
        let default = "render::components::button::default 120 40 junie truecolor";
        let selected = "render::components::button::selected 120 40 junie truecolor";
        let digest = collided
            .get(default)
            .cloned()
            .expect("default button digest");
        collided.insert(selected.to_owned(), digest);
        let failures = visual_state_failures(&collided);
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(failures[0].contains("button selected"), "{failures:?}");

        let mut stale = visual_entries();
        let selected = "render::components::text_input::selected 120 40 junie truecolor";
        stale.insert(selected.to_owned(), "0000000000000000".to_owned());
        let failures = visual_state_failures(&stale);
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(failures[0].contains("exemption is stale"), "{failures:?}");
    }
}

#[test]
fn selected_picker_rows_use_the_canonical_chosen_marker() {
    let chosen = Theme::junie().design.glyphs.get(GlyphRole::Chosen);
    for comp in [
        Comp::FilterList,
        Comp::Picker,
        Comp::Completion,
        Comp::ContextMenu,
    ] {
        let mut scene = Scene::new("selected-marker", Theme::junie(), ColorLevel::Mono, 120, 40);
        scene.draw(|ui, area| draw(comp, St::Selected, ui, area));
        assert!(
            scene.text().contains(chosen),
            "{comp:?} selected state omitted the chosen marker {chosen:?}"
        );
    }
}

#[test]
fn selected_menu_titles_use_the_canonical_chosen_marker() {
    let chosen = Theme::junie().design.glyphs.get(GlyphRole::Chosen);
    for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
        let mut scene = Scene::new("selected-menu-title", Theme::junie(), color, 120, 40);
        scene.draw(|ui, area| draw(Comp::MenuBar, St::Selected, ui, area));
        assert!(
            scene.text().contains(chosen),
            "selected menu title omitted the chosen marker {chosen:?} at {color:?}"
        );
    }
}

#[test]
fn button_checked_glyph_is_stable_across_colour_levels() {
    let chosen = Theme::junie().design.glyphs.get(GlyphRole::Checked);
    for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
        let mut scene = Scene::new("button-selected", Theme::junie(), color, 40, 10);
        scene.draw(|ui, area| draw(Comp::Button, St::Selected, ui, area));
        assert_eq!(
            scene
                .buffer()
                .cell(Position::new(1, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(chosen),
            "checked button marker changed at {color:?}"
        );
    }
}

#[test]
fn too_small_and_panel_fixtures_keep_their_stateful_content() {
    let mut too_small = Scene::new("too-small", Theme::junie(), ColorLevel::Mono, 40, 10);
    too_small.draw(|ui, area| draw(Comp::TooSmall, St::Empty, ui, area));
    assert!(too_small.text().contains("Junie"));

    let mut panel_default = Scene::new("panel-default", Theme::junie(), ColorLevel::Mono, 40, 10);
    panel_default.draw(|ui, area| draw(Comp::Panel, St::Default, ui, area));
    let mut panel_focused = Scene::new("panel-focused", Theme::junie(), ColorLevel::Mono, 40, 10);
    panel_focused.draw(|ui, area| draw(Comp::Panel, St::Focused, ui, area));
    assert!(panel_default.text().contains("Inspector"));
    assert!(panel_focused.text().contains("Inspector"));
    assert_ne!(panel_default.digest(), panel_focused.digest());
    let mut panel_empty = Scene::new("panel-empty", Theme::junie(), ColorLevel::Mono, 40, 10);
    panel_empty.draw(|ui, area| draw(Comp::Panel, St::Empty, ui, area));
    assert!(!panel_empty.text().contains("Inspector"));
    assert!(!panel_empty.text().contains("Selected object details"));
}

#[test]
fn scroll_region_fixture_exposes_the_complete_bar_at_both_matrix_sizes() {
    let theme = Theme::junie();
    let glyphs = theme.design.glyphs.scrollbar();
    for st in [St::Default, St::Focused, St::Pressed] {
        for (width, height) in SIZES {
            let mut scene = Scene::new(
                "scroll fixture",
                theme.clone(),
                ColorLevel::Mono,
                width,
                height,
            );
            scene.draw(|ui, area| draw(Comp::ScrollRegion, st, ui, area));

            let x = width.saturating_sub(1);
            let symbol_at = |y| {
                scene
                    .buffer()
                    .cell(Position::new(x, y))
                    .expect("matrix position is inside the scene")
                    .symbol()
            };
            assert_eq!(symbol_at(0), glyphs.begin, "{st:?} at {width}x{height}");
            assert_eq!(
                symbol_at(height.saturating_sub(1)),
                glyphs.end,
                "{st:?} at {width}x{height}"
            );
            assert!(
                (1..height.saturating_sub(1)).any(|y| symbol_at(y) == glyphs.track),
                "{st:?} at {width}x{height} has no visible track"
            );
            assert!(
                (1..height.saturating_sub(1)).any(|y| symbol_at(y) == glyphs.thumb),
                "{st:?} at {width}x{height} has no visible thumb"
            );
            let bold: Vec<Position> = scene
                .area()
                .positions()
                .filter(|pos| {
                    scene
                        .buffer()
                        .cell(*pos)
                        .is_some_and(|cell| cell.modifier.contains(Modifier::BOLD))
                })
                .collect();
            let expected: Vec<Position> = if st == St::Pressed {
                let thumb_end = match (width, height) {
                    (120, 40) => 20,
                    (40, 10) => 2,
                    _ => unreachable!("SIZES contains only the two matrix sizes"),
                };
                (1..thumb_end).map(|y| Position::new(x, y)).collect()
            } else {
                Vec::new()
            };
            assert_eq!(bold, expected, "{st:?} at {width}x{height}");
        }
    }
}

#[test]
fn form_fixture_references_exact_child_visual_states() {
    for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
        let digest = |st| {
            let mut scene = Scene::new("form reference state", Theme::junie(), color, 120, 40);
            scene.draw(|ui, area| draw(Comp::Form, st, ui, area));
            scene.digest()
        };
        let default = digest(St::Default);
        let focused = digest(St::Focused);
        let pressed = digest(St::Pressed);
        assert_ne!(default, focused, "focused form at {color:?}");
        assert_ne!(default, pressed, "pressed form at {color:?}");
        assert_ne!(focused, pressed, "focused/pressed form at {color:?}");
    }
}

#[test]
fn form_fixture_targets_exactly_one_owned_child() {
    let text = |st| {
        let mut scene = Scene::new("form target", Theme::junie(), ColorLevel::Mono, 120, 40);
        scene.draw(|ui, area| draw(Comp::Form, st, ui, area));
        scene.text()
    };

    let focused = text(St::Focused);
    assert_eq!(focused.matches('▎').count(), 1, "{focused}");
    assert!(focused.contains("▎ Ada Lovelace"));

    let pressed = text(St::Pressed);
    assert_eq!(pressed.matches("[Enabled]").count(), 1);
    assert!(!pressed.contains("[Cancel]"));
    assert!(!pressed.contains("[OK]"));
    assert!(!pressed.contains("[Ada Lovelace]"));

    let selected = text(St::Selected);
    assert_eq!(selected.matches("[✓] Enabled").count(), 1);
    assert!(!selected.contains("[Enabled]"));
}

struct RenderFormFixture {
    state: St,
}

impl App for RenderFormFixture {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        draw(Comp::Form, self.state, ui, ui.full());
    }
}

#[test]
fn form_fixture_subtree_has_no_live_regions_or_focus_stops() {
    for state in [
        St::Default,
        St::Focused,
        St::Hovered,
        St::Pressed,
        St::Disabled,
        St::Selected,
        St::Editing,
    ] {
        let harness = Harness::new(RenderFormFixture { state }, Theme::junie(), 120, 40);
        assert!(harness.ring().entries().is_empty(), "{state:?} focus stop");
        for id in [FORM, INPUT, CHECKBOX, FORM.part(Part::ACTIONS).index(0)] {
            assert!(harness.area_of(id).is_none(), "{state:?} live {id:?}");
        }
    }
}

#[test]
fn selected_component_fixtures_supply_semantic_state() {
    for (name, comp) in [
        ("button", Comp::Button),
        ("list", Comp::List),
        ("tabs", Comp::Tabs),
        ("grid", Comp::Grid),
    ] {
        for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
            let digest = |st| {
                let mut scene = Scene::new(name, Theme::junie(), color, 120, 40);
                scene.draw(|ui, area| draw(comp, st, ui, area));
                scene.digest()
            };
            assert_ne!(
                digest(St::Default),
                digest(St::Selected),
                "{name} selected fixture at {color:?}"
            );
        }
    }
}

#[test]
fn select_selected_fixture_matches_default_in_every_matrix_cell() {
    for (theme_name, theme) in [("junie", Theme::junie()), ("paper", Theme::paper())] {
        for (color_name, color) in [
            ("truecolor", ColorLevel::TrueColor),
            ("mono", ColorLevel::Mono),
        ] {
            for (width, height) in SIZES {
                let digest = |st| {
                    let mut scene = Scene::new(
                        "select semantic selection",
                        theme.clone(),
                        color,
                        width,
                        height,
                    );
                    scene.draw(|ui, area| draw_select(st, ui, area));
                    scene.digest()
                };
                assert_eq!(
                    digest(St::Default),
                    digest(St::Selected),
                    "select selected at {width}x{height} {theme_name} {color_name}"
                );
            }
        }
    }
}

mod render {
    mod components {
        matrix!(button, Comp::Button);
        matrix!(text_input, Comp::TextInput);
        matrix!(field, Comp::Field);
        matrix!(list, Comp::List);
        matrix!(tabs, Comp::Tabs);
        matrix!(dialog, Comp::Dialog);
        matrix!(scroll_region, Comp::ScrollRegion);
        matrix!(text_area, Comp::TextArea);
        matrix!(select, Comp::Select);
        matrix!(radio_group, Comp::RadioGroup);
        matrix!(checkbox, Comp::Checkbox);
        matrix!(toggle, Comp::Toggle);
        matrix!(chip_bar, Comp::ChipBar);
        matrix!(status_bar, Comp::StatusBar);
        matrix!(hint_bar, Comp::HintBar);
        matrix!(key_hint, Comp::KeyHint);
        matrix!(progress_bar, Comp::ProgressBar);
        matrix!(spinner, Comp::Spinner);
        matrix!(meter, Comp::Meter);
        matrix!(empty, Comp::Empty);
        matrix!(brand, Comp::Brand);
        matrix!(panel, Comp::Panel);
        matrix!(split_pane, Comp::SplitPane);
        matrix!(text_viewport, Comp::TextViewport);
        matrix!(diff_view, Comp::DiffView);
        matrix!(code_editor, Comp::CodeEditor);
        matrix!(tree, Comp::Tree);
        matrix!(nav_list, Comp::NavList);
        matrix!(steps, Comp::Steps);
        matrix!(too_small, Comp::TooSmall);
        matrix!(grid, Comp::Grid);
        matrix!(filter_list, Comp::FilterList);
        matrix!(picker, Comp::Picker);
        matrix!(completion, Comp::Completion);
        matrix!(form, Comp::Form);
        matrix!(context_menu, Comp::ContextMenu);
        matrix!(help_overlay, Comp::HelpOverlay);
        matrix!(menu_bar, Comp::MenuBar);
        matrix!(picker_chain, Comp::PickerChain);
        matrix!(wizard, Comp::Wizard);
    }
}
