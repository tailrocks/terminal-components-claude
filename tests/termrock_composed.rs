//! Termrock Composed Application Scenarios and Integration Conformance.
//!
//! Exercises full cross-component compositions matching the four reference applications:
//! 1. Showcase — Full multi-pane application layout, forms, overlays, pickers, and rich output.
//! 2. TablePro — Workbench layout with connection tree, tabs, grid with cell editing, SQL preview, and status chrome.
//! 3. Jackin Preview — Cockpit telemetry, capsule configuration, agent inspect tree, and terminal presentation.
//! 4. Holla — Multi-step cleanup wizard, activity file filter list, diff view, and confirmation modals.
//! 5. Cross-component edge scenarios — Nested overlays, narrow geometry, resize, pointer routing, and key reconciliation.

#![allow(unused_imports, unused_variables, dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::termrock::author::{TerminalCell, TerminalCursor, TerminalSource};
use junie_tui::termrock::brand::*;
use junie_tui::termrock::button::*;
use junie_tui::termrock::checkbox::*;
use junie_tui::termrock::chip_bar::*;
use junie_tui::termrock::code_editor::*;
use junie_tui::termrock::collections::*;
use junie_tui::termrock::command_palette::*;
use junie_tui::termrock::completion::*;
use junie_tui::termrock::context_menu::*;
use junie_tui::termrock::dialog::*;
use junie_tui::termrock::diff_view::*;
use junie_tui::termrock::empty::*;
use junie_tui::termrock::field::*;
use junie_tui::termrock::filter_list::*;
use junie_tui::termrock::form::*;
use junie_tui::termrock::grid::*;
use junie_tui::termrock::help_overlay::*;
use junie_tui::termrock::hint_bar::*;
use junie_tui::termrock::identity::*;
use junie_tui::termrock::key_hint::*;
use junie_tui::termrock::layers::*;
use junie_tui::termrock::layout::*;
use junie_tui::termrock::list::*;
use junie_tui::termrock::menu::*;
use junie_tui::termrock::menu_bar::*;
use junie_tui::termrock::meter::*;
use junie_tui::termrock::nav_list::*;
use junie_tui::termrock::panel::*;
use junie_tui::termrock::picker::*;
use junie_tui::termrock::picker_chain::*;
use junie_tui::termrock::progress_bar::*;
use junie_tui::termrock::props::*;
use junie_tui::termrock::props_list::*;
use junie_tui::termrock::radio_group::*;
use junie_tui::termrock::response::*;
use junie_tui::termrock::runtime::*;
use junie_tui::termrock::scroll::*;
use junie_tui::termrock::secret::*;
use junie_tui::termrock::select::*;
use junie_tui::termrock::spinner::*;
use junie_tui::termrock::split_pane::*;
use junie_tui::termrock::status_bar::*;
use junie_tui::termrock::steps::*;
use junie_tui::termrock::tabs::*;
use junie_tui::termrock::terminal_view::*;
use junie_tui::termrock::text::*;
use junie_tui::termrock::text_area::*;
use junie_tui::termrock::text_input::*;
use junie_tui::termrock::text_viewport::*;
use junie_tui::termrock::theme::*;
use junie_tui::termrock::toggle::*;
use junie_tui::termrock::too_small::*;
use junie_tui::termrock::tree::*;
use junie_tui::termrock::wizard::*;

fn make_cx<'a>(
    cause: &'a UpdateCause,
    id: Id,
    layers: &'a mut LayerStack,
    geom: &'a HashMap<Id, Rect>,
) -> Cx<'a> {
    Cx {
        cause,
        moment: Moment::from_millis(100),
        intended_owner: Some(id.clone()),
        focus: Some(id),
        pointer_capture: None,
        invalidate: Invalidate::None,
        layer_stack: layers,
        cursor_request: None,
        new_focus: None,
        new_capture: None,
        feedback_requests: Vec::new(),
        published_geometry: Some(geom),
    }
}

// -----------------------------------------------------------------------------
// 1. Showcase Composed Application Test
// -----------------------------------------------------------------------------

#[test]
fn test_showcase_composed_architecture() {
    let theme = Theme::termrock();
    let area = Rect::new(0, 0, 120, 40);
    let mut layers = LayerStack::new();

    // Navigation items
    let nav_items = [
        NavItem::new(ItemKey::new(1), "Overview"),
        NavItem::new(ItemKey::new(2), "Buttons & Chrome"),
        NavItem::new(ItemKey::new(3), "Inputs & Forms"),
        NavItem::new(ItemKey::new(4), "Data Grid & Tables"),
        NavItem::new(ItemKey::new(5), "Modals & Overlays"),
    ];
    let nav_state = NavListState {
        cursor: Some(ItemKey::new(1)),
        ..Default::default()
    };

    // Form inputs state
    let text_state = TextInputState::new();
    let toggle_state = true;

    // Command palette state
    let palette_items = [
        CommandItem::new(1u64, "Switch to Dark Theme").category("Preferences"),
        CommandItem::new(2u64, "Switch to Light Theme").category("Preferences"),
        CommandItem::new(3u64, "Export Current View").category("File"),
    ];
    let palette_state = CommandPaletteState::default();

    // Draw frame 1: standard overview
    let mut ui = Ui::new(&theme, area, &mut layers);

    // Header: Brand + Status
    let brand = Brand::new(Id::new("header_brand"), "SHOWCASE")
        .meta("v1.0")
        .interactive(true);
    brand.draw(&mut ui, Rect::new(0, 0, 20, 1));

    // Sidebar Navigation
    let nav = NavList::new(Id::new("sidebar_nav"), &nav_items, Revision::new(1));
    nav.draw(&mut ui, Rect::new(0, 2, 24, 36), &nav_state);

    // Main Viewport: SplitPane
    let split = SplitPane::new(Id::new("main_split"), Axis::Horizontal);
    let split_state = SplitPaneState::new(0.6);
    let areas = split.layout(Rect::new(26, 2, 94, 36), &split_state);

    // Left Panel: Form
    let form_panel = Panel::new(Id::new("form_panel")).title("Configuration");
    form_panel.draw(&mut ui, areas.first, |ui, inner_form| {
        let name_field = TextInput::new(Id::new("input_name"), "Acme Corp", Revision::new(1));
        name_field.draw(
            ui,
            Rect::new(inner_form.x + 2, inner_form.y + 2, 30, 1),
            &text_state,
        );

        let toggle_ctrl = Toggle::new(Id::new("toggle_ssl"), "Enable SSL", toggle_state);
        toggle_ctrl.draw(ui, Rect::new(inner_form.x + 2, inner_form.y + 4, 25, 1));
    });

    // Right Panel: Props
    let props_panel = Panel::new(Id::new("props_panel")).title("Telemetry");
    props_panel.draw(&mut ui, areas.second, |ui, inner_props| {
        let prop_rows = [
            PropsRow::new(ItemKey::new(10), "Status", PropsValue::Text("Healthy")),
            PropsRow::new(ItemKey::new(11), "Uptime", PropsValue::Text("99.98%")),
        ];
        let props = Props::new(Id::new("telemetry_props"), &prop_rows);
        props.draw(
            ui,
            Rect::new(
                inner_props.x + 2,
                inner_props.y + 2,
                inner_props.width.saturating_sub(4),
                4,
            ),
        );
    });

    // Footer: StatusBar
    let status_left = [
        StatusItem::new(1u64, "READY").tone(Tone::Success),
        StatusItem::new(2u64, "OVERVIEW").tone(Tone::Accent),
    ];
    let status_right = [StatusItem::new(3u64, "120x40").tone(Tone::Muted)];
    let status = StatusBar::new(Id::new("app_status"))
        .left(&status_left)
        .right(&status_right);
    status.draw(&mut ui, Rect::new(0, 39, 120, 1));

    // Hit region registration verification
    assert!(ui.hit_regions.contains_key(&Id::new("header_brand")));
    assert!(ui.hit_regions.contains_key(&Id::new("sidebar_nav")));
    assert!(ui.hit_regions.contains_key(&Id::new("input_name")));

    // Draw frame 2: modal command palette overlay
    let mut ui2 = Ui::new(&theme, area, &mut layers);
    let palette = CommandPalette::new(Id::new("cmd_palette"), &palette_items, Revision::new(1));
    palette.draw(&mut ui2, Rect::new(20, 8, 80, 20), &palette_state);
    assert!(ui2.hit_regions.contains_key(&Id::new("cmd_palette")));
}

// -----------------------------------------------------------------------------
// 2. TablePro Composed Workbench Test
// -----------------------------------------------------------------------------

struct DemoGridModel {
    cols: Vec<GridColumn>,
    rows: Vec<(ItemKey, Vec<CellValue>)>,
}

impl GridModel for DemoGridModel {
    fn revision(&self) -> Revision {
        Revision::new(1)
    }
    fn row_count(&self) -> usize {
        self.rows.len()
    }
    fn row_key(&self, index: usize) -> ItemKey {
        self.rows[index].0
    }
    fn columns(&self) -> &[GridColumn] {
        &self.cols
    }
    fn cell_value(&self, row: ItemKey, col: ColumnKey) -> Option<CellValue> {
        let r_idx = self.row_index(row)?;
        let c_idx = self.column_index(col)?;
        self.rows
            .get(r_idx)
            .and_then(|(_, r)| r.get(c_idx))
            .cloned()
    }
}

impl GridEditor for DemoGridModel {
    fn is_editable(&self, _row: ItemKey, col: ColumnKey) -> bool {
        col != ColumnKey::new(1)
    }
    fn commit_cell(&mut self, row: ItemKey, col: ColumnKey, value: CellValue) -> bool {
        if let Some(r_idx) = self.row_index(row)
            && let Some(c_idx) = self.column_index(col)
            && let Some((_, r)) = self.rows.get_mut(r_idx)
            && let Some(c) = r.get_mut(c_idx)
        {
            *c = value;
            return true;
        }
        false
    }
}

struct StaticTextSource {
    rev: Revision,
    lines: Vec<String>,
}

impl StaticTextSource {
    fn new(text: &str) -> Self {
        Self {
            rev: Revision::new(1),
            lines: text.lines().map(|s| s.to_string()).collect(),
        }
    }
}

impl TextSource for StaticTextSource {
    fn revision(&self) -> Revision {
        self.rev
    }
    fn line_count(&self) -> usize {
        self.lines.len()
    }
    fn line(&self, index: usize) -> Option<TextLine<'_>> {
        self.lines
            .get(index)
            .map(|s| TextLine::plain(ItemKey::new(index as u64), s.as_str()))
    }
}

#[test]
fn test_tablepro_composed_architecture() {
    let theme = Theme::termrock();
    let area = Rect::new(0, 0, 120, 40);
    let mut layers = LayerStack::new();

    // Workbench tabs
    let tab_items = [
        TabItem::new(ItemKey::new(1), "public.orders"),
        TabItem::new(ItemKey::new(2), "public.customers"),
        TabItem::new(ItemKey::new(3), "Query 1"),
    ];
    let tabs_state = TabsState::new().with_active(ItemKey::new(1));

    // Grid data setup
    let grid_cols = vec![
        GridColumn::new(ColumnKey::new(1), "id", 12).primary(true),
        GridColumn::new(ColumnKey::new(2), "customer", 16),
        GridColumn::new(ColumnKey::new(3), "status", 14),
        GridColumn::new(ColumnKey::new(4), "total", 12).kind(CellKind::Number),
    ];
    let grid_rows = vec![
        (
            ItemKey::new(101),
            vec![
                CellValue::Text("ord-001".into()),
                CellValue::Text("Alice".into()),
                CellValue::Text("pending".into()),
                CellValue::Num(120.50),
            ],
        ),
        (
            ItemKey::new(102),
            vec![
                CellValue::Text("ord-002".into()),
                CellValue::Text("Bob".into()),
                CellValue::Text("shipped".into()),
                CellValue::Num(45.00),
            ],
        ),
        (
            ItemKey::new(103),
            vec![
                CellValue::Text("ord-003".into()),
                CellValue::Text("Carol".into()),
                CellValue::Text("pending".into()),
                CellValue::Num(320.00),
            ],
        ),
    ];
    let mut model = DemoGridModel {
        cols: grid_cols,
        rows: grid_rows,
    };
    let mut grid_state = GridState {
        cursor_row: Some(ItemKey::new(101)),
        cursor_col: Some(ColumnKey::new(3)),
        ..Default::default()
    };

    // Filter chip items
    let chips = [
        ChipItem::new(ItemKey::new(1), "status = 'pending'").checked(Some(true)),
        ChipItem::new(ItemKey::new(2), "currency = 'USD'"),
    ];
    let chip_state = ChipBarState::default();

    // Draw workbench frame
    let mut ui = Ui::new(&theme, area, &mut layers);

    // Header: Tabs
    let tabs = Tabs::new(Id::new("table_tabs"), &tab_items, Revision::new(1));
    tabs.draw(&mut ui, Rect::new(0, 0, 120, 2), &tabs_state);

    // Filter chips bar
    let chip_bar =
        ChipBar::new(Id::new("table_filters"), &chips, Revision::new(1)).lead(Some("FILTERS:"));
    chip_bar.draw(&mut ui, Rect::new(0, 2, 120, 1), &chip_state);

    // Grid Area
    let grid = Grid::new(Id::new("data_grid")).mode(GridMode::Cell);
    grid.draw(&mut ui, Rect::new(0, 3, 120, 35), &grid_state, &model);

    // Status Bar
    let status_left = [
        StatusItem::new(1u64, "PostgreSQL · Production").tone(Tone::Success),
        StatusItem::new(2u64, "3 rows").tone(Tone::Normal),
        StatusItem::new(3u64, "1 pending edit").tone(Tone::Warning),
    ];
    let status = StatusBar::new(Id::new("table_status")).left(&status_left);
    status.draw(&mut ui, Rect::new(0, 39, 120, 1));

    // Verify cell editing interaction
    let cause_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let geom = ui.hit_regions.clone();
    let mut cx = make_cx(&cause_enter, Id::new("data_grid"), &mut layers, &geom);
    let grid_comp = Grid::new(Id::new("data_grid")).mode(GridMode::Cell);
    grid_comp.update_editable(&mut cx, &mut grid_state, &mut model);
    assert!(
        grid_state.editing.is_some(),
        "Enter on cell must activate edit mode"
    );

    // Commit edit: type "completed" then Enter
    if let Some(draft) = &mut grid_state.editing {
        draft.buffer.set_text("completed");
    }
    let mut cx_commit = make_cx(&cause_enter, Id::new("data_grid"), &mut layers, &geom);
    let resp = grid_comp.update_editable(&mut cx_commit, &mut grid_state, &mut model);
    assert!(
        grid_state.editing.is_none(),
        "Enter in edit mode must commit draft"
    );
    if let Some(GridAction::EditCommitted { cell, value }) = resp.action {
        model.commit_cell(cell.row, cell.col, value);
    }
    assert_eq!(
        model.rows[0].1[2],
        CellValue::Text("completed".into()),
        "Model must receive committed cell value"
    );

    // Verify SQL Preview Dialog presentation
    let preview_statements = "UPDATE public.orders SET status = 'completed' WHERE id = 'ord-001';";
    let text_source = StaticTextSource::new(preview_statements);
    let mut ui_dlg = Ui::new(&theme, area, &mut layers);
    let actions = [
        ActionMeta::new(ActionKey::new("cancel"), "Cancel"),
        ActionMeta::new(ActionKey::new("save"), "Save Changes"),
    ];
    let dialog = Dialog::new(
        Id::new("sql_preview_dialog"),
        "Preview Changes (1 statement)",
        &actions,
    );
    let dlg_state = DialogState::default();

    dialog.draw(
        &mut ui_dlg,
        Rect::new(20, 5, 80, 25),
        &dlg_state,
        |ui, inner| {
            let viewport = TextViewport::new(Id::new("sql_view"), &text_source);
            let vp_state = ViewportState::default();
            viewport.draw(ui, inner, &vp_state);
        },
    );

    assert!(
        ui_dlg
            .hit_regions
            .contains_key(&Id::new("sql_preview_dialog"))
    );
}

// -----------------------------------------------------------------------------
// 3. Jackin Preview Composed Cockpit & Capsule Test
// -----------------------------------------------------------------------------

struct DemoTreeSource {
    roots: Vec<ItemKey>,
    nodes: BTreeMap<ItemKey, (String, Vec<ItemKey>)>,
}

impl TreeSource for DemoTreeSource {
    fn revision(&self) -> Revision {
        Revision::new(1)
    }
    fn roots(&self) -> &[ItemKey] {
        &self.roots
    }
    fn node(&self, key: ItemKey) -> Option<TreeNode<'_>> {
        let (label, children) = self.nodes.get(&key)?;
        Some(TreeNode {
            key,
            label,
            children,
            expanded: true,
            leaf: children.is_empty(),
            icon: None,
            meta: None,
            disabled: false,
            busy: false,
        })
    }
}

struct DemoTerminalSource {
    rev: Revision,
    dim: Size,
    lines: Vec<String>,
}

impl DemoTerminalSource {
    fn new(lines: &[&str]) -> Self {
        Self {
            rev: Revision::new(1),
            dim: Size::new(80, lines.len() as u16),
            lines: lines.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl TerminalSource for DemoTerminalSource {
    fn revision(&self) -> Revision {
        self.rev
    }
    fn size(&self) -> Size {
        self.dim
    }
    fn cursor(&self) -> Option<TerminalCursor> {
        Some(TerminalCursor {
            pos: Position::new(0, 0),
            visible: true,
        })
    }
    fn cell(&self, position: Position) -> Option<TerminalCell<'_>> {
        let text = self.lines.get(position.y as usize)?;
        let (byte_idx, ch) = text.char_indices().nth(position.x as usize)?;
        let end_idx = byte_idx + ch.len_utf8();
        let s = &text[byte_idx..end_idx];
        Some(TerminalCell::new(s))
    }
}

#[test]
fn test_jackin_preview_composed_architecture() {
    let theme = Theme::termrock();
    let area = Rect::new(0, 0, 120, 40);
    let mut layers = LayerStack::new();

    // Tree source setup
    let mut nodes = BTreeMap::new();
    nodes.insert(
        ItemKey::new(1),
        (
            "Prod Cluster (3 agents)".to_string(),
            vec![ItemKey::new(2), ItemKey::new(3)],
        ),
    );
    nodes.insert(
        ItemKey::new(2),
        ("agent-core-01 (Running)".to_string(), vec![]),
    );
    nodes.insert(
        ItemKey::new(3),
        ("agent-worker-02 (Running)".to_string(), vec![]),
    );

    let tree_source = DemoTreeSource {
        roots: vec![ItemKey::new(1)],
        nodes,
    };
    let mut tree_state = TreeState::default();
    tree_state.expanded.insert(ItemKey::new(1));
    tree_state.cursor = Some(ItemKey::new(2));

    // Telemetry meters
    let cpu_meter = Meter::new(Id::new("cpu_meter"), 0.72, 0.0, 1.0)
        .label("CPU LOAD")
        .readout("72%");
    let mem_meter = Meter::new(Id::new("mem_meter"), 0.45, 0.0, 1.0)
        .label("MEMORY")
        .readout("45%");

    // Terminal source setup
    let term_source = DemoTerminalSource::new(&[
        "2026-10-04T09:45:12Z [INFO] capsule initialized",
        "2026-10-04T09:45:13Z [INFO] worker started listening on port 8080",
    ]);
    let term_view = TerminalView::new(Id::new("agent_term"), &term_source);
    let mut term_state = TerminalViewState::default();

    // Render Cockpit Layout
    let mut ui = Ui::new(&theme, area, &mut layers);

    // Header
    let brand = Brand::new(Id::new("jackin_brand"), "JACKIN COCKPIT").meta("cluster: us-west-1");
    brand.draw(&mut ui, Rect::new(0, 0, 40, 1));

    // Top telemetry bar
    cpu_meter.draw(&mut ui, Rect::new(45, 0, 30, 1));
    mem_meter.draw(&mut ui, Rect::new(80, 0, 30, 1));

    // Split layout
    let split = SplitPane::new(Id::new("cockpit_split"), Axis::Horizontal);
    let split_state = SplitPaneState::new(0.35);
    let areas = split.layout(Rect::new(0, 2, 120, 36), &split_state);

    // Left Panel: Tree
    let tree_panel = Panel::new(Id::new("tree_panel")).title("Agent Topology");
    tree_panel.draw(&mut ui, areas.first, |ui, inner_tree| {
        let tree = Tree::new(Id::new("agent_tree"), &tree_source);
        tree.draw(ui, inner_tree, &tree_state);
    });

    // Right Panel: Terminal View
    let term_panel = Panel::new(Id::new("term_panel")).title("Live Agent Console");
    term_panel.draw(&mut ui, areas.second, |ui, inner_term| {
        term_view.draw(ui, inner_term, &term_state);
    });

    // Footer
    let status_left = [
        StatusItem::new(1u64, "ALL SYSTEMS NORMAL").tone(Tone::Success),
        StatusItem::new(2u64, "PTY CONNECTED").tone(Tone::Accent),
    ];
    let status = StatusBar::new(Id::new("cockpit_status")).left(&status_left);
    status.draw(&mut ui, Rect::new(0, 39, 120, 1));

    // Verify tree selection and keyboard navigation
    let cause_down = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Down,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let geom = ui.hit_regions.clone();
    let mut cx_down = make_cx(&cause_down, Id::new("agent_tree"), &mut layers, &geom);
    let tree = Tree::new(Id::new("agent_tree"), &tree_source);
    tree.update(&mut cx_down, &mut tree_state);
    assert_eq!(tree_state.cursor, Some(ItemKey::new(3)));

    // Verify terminal view mouse selection
    let cause_mouse = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: ratatui::layout::Position { x: 50, y: 10 },
        }),
        Moment::from_millis(100),
    );
    let mut cx_term = make_cx(&cause_mouse, Id::new("agent_term"), &mut layers, &geom);
    term_view.update(&mut cx_term, &mut term_state);
    assert!(term_state.selection.is_some());
}

// -----------------------------------------------------------------------------
// 4. Holla Composed Cleanup & Wizard Test
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileItem {
    key: ItemKey,
    label: &'static str,
}

impl Keyed for FileItem {
    fn key(&self) -> ItemKey {
        self.key
    }
}

#[test]
fn test_holla_composed_architecture() {
    let theme = Theme::termrock();
    let area = Rect::new(0, 0, 120, 40);
    let mut layers = LayerStack::new();

    // Wizard steps rail
    let steps = [
        StepItem::new(ItemKey::new(1), "1. Analyze", StepStatus::Done),
        StepItem::new(ItemKey::new(2), "2. Review Diffs", StepStatus::Running),
        StepItem::new(ItemKey::new(3), "3. Confirm Clean", StepStatus::Queued),
    ];
    let steps_rail = Steps::new(Id::new("cleanup_steps"), &steps, Revision::new(1));
    let steps_state = StepsState::default();

    // Review files list
    let file_rows = [
        FileItem {
            key: ItemKey::new(101),
            label: "~/Library/Caches/com.docker.docker (14.2 GB)",
        },
        FileItem {
            key: ItemKey::new(102),
            label: "~/.cargo/registry/cache (3.8 GB)",
        },
        FileItem {
            key: ItemKey::new(103),
            label: "~/.rustup/toolchains/old (2.1 GB)",
        },
    ];
    let mut file_list_state = ListState {
        cursor: Some(ItemKey::new(101)),
        ..Default::default()
    };

    // Diff view for selected file
    let mut diff_source = MemoryDiffSource::new()
        .with_header("~/Library/Caches/com.docker.docker")
        .with_status(DiffStatus::Deleted);
    diff_source.add_removal(ItemKey::new(1), 1, "data.raw (10 GB)");
    diff_source.add_removal(ItemKey::new(2), 2, "log.vmdk (4.2 GB)");

    let diff = DiffView::new(Id::new("file_diff"), &diff_source);
    let diff_state = DiffViewState::default();

    // Render Holla review frame
    let mut ui = Ui::new(&theme, area, &mut layers);

    // Header
    let brand = Brand::new(Id::new("holla_brand"), "HOLLA CLEANUP").meta("Clean disk space safely");
    brand.draw(&mut ui, Rect::new(0, 0, 40, 1));

    // Steps rail
    steps_rail.draw(&mut ui, Rect::new(45, 0, 75, 1), &steps_state);

    // Split view
    let split = SplitPane::new(Id::new("review_split"), Axis::Horizontal);
    let split_state = SplitPaneState::new(0.4);
    let areas = split.layout(Rect::new(0, 2, 120, 34), &split_state);

    let list_panel =
        Panel::new(Id::new("list_panel")).title("Reclaimable Files (3 targets · 20.1 GB)");
    list_panel.draw(&mut ui, areas.first, |ui, inner_list| {
        let list = List::new(Id::new("file_list"), &file_rows, Revision::new(1))
            .selection_mode(SelectionMode::Multiple);
        list.draw(ui, inner_list, &file_list_state);
    });

    let diff_panel = Panel::new(Id::new("diff_panel")).title("Target Manifest Diff");
    diff_panel.draw(&mut ui, areas.second, |ui, inner_diff| {
        diff.draw(ui, inner_diff, &diff_state);
    });

    // Controls bar: Next / Back buttons
    let btn_back = Button::new(Id::new("btn_back"), "‹ Back").variant(ButtonVariant::Secondary);
    let btn_next =
        Button::new(Id::new("btn_next"), "Proceed to Cleanup ›").variant(ButtonVariant::Primary);
    btn_back.draw(&mut ui, Rect::new(80, 37, 16, 1));
    btn_next.draw(&mut ui, Rect::new(98, 37, 22, 1));

    // Status bar
    let status_left = [
        StatusItem::new(1u64, "READY TO RECLAIM 20.1 GB").tone(Tone::Warning),
        StatusItem::new(2u64, "Space free after cleanup: 84.5 GB").tone(Tone::Normal),
    ];
    let status = StatusBar::new(Id::new("holla_status")).left(&status_left);
    status.draw(&mut ui, Rect::new(0, 39, 120, 1));

    // Verify list navigation and space selection
    let cause_space = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Char(' '),
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let geom = ui.hit_regions.clone();
    let mut cx_toggle = make_cx(&cause_space, Id::new("file_list"), &mut layers, &geom);
    let list = List::new(Id::new("file_list"), &file_rows, Revision::new(1))
        .selection_mode(SelectionMode::Multiple);
    let resp = list.update(&mut cx_toggle, &mut file_list_state);
    assert!(matches!(
        resp.action,
        Some(ListAction::SelectionRequested(_))
    ));

    // Click on Proceed button: pointer down captures, pointer up activates
    let cause_down = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: ratatui::layout::Position { x: 105, y: 37 },
        }),
        Moment::from_millis(100),
    );
    let mut cx_down = make_cx(&cause_down, Id::new("btn_next"), &mut layers, &geom);
    let _ = btn_next.update(&mut cx_down);
    assert_eq!(cx_down.new_capture, Some(Some(Id::new("btn_next"))));

    let cause_up = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: ratatui::layout::Position { x: 105, y: 37 },
        }),
        Moment::from_millis(150),
    );
    let mut cx_up = make_cx(&cause_up, Id::new("btn_next"), &mut layers, &geom);
    cx_up.pointer_capture = Some(Id::new("btn_next"));
    let btn_resp = btn_next.update(&mut cx_up);
    assert!(
        matches!(btn_resp.action, Some(Activated { .. })),
        "Clicking next button must activate"
    );
}

// -----------------------------------------------------------------------------
// 5. Cross-Component Edge & Parity Invariants (AC-002)
// -----------------------------------------------------------------------------

#[test]
fn test_cross_component_edge_and_parity_invariants() {
    let theme = Theme::termrock();
    let mut layers = LayerStack::new();

    // 1. Nested overlays: Dialog over Menu over SplitPane
    let area = Rect::new(0, 0, 100, 30);
    let mut ui = Ui::new(&theme, area, &mut layers);

    // Base surface
    let panel = Panel::new(Id::new("base_panel")).title("Base Surface");
    panel.draw(&mut ui, area, |_ui, _inner| {});

    // Nested overlay 1: Menu
    let menu_items = [MenuItem::action(
        ItemKey::new(1),
        ActionKey::new("opt1"),
        "Option 1",
    )];
    let menu = Menu::new(Id::new("overlay_menu"), &menu_items, Revision::new(1));
    let menu_state = MenuState::default();
    menu.draw(&mut ui, Rect::new(10, 5, 30, 10), &menu_state);

    // Nested overlay 2: Topmost Dialog
    let actions = [ActionMeta::new(ActionKey::new("confirm"), "Confirm")];
    let dialog = Dialog::new(Id::new("top_dialog"), "Topmost Confirmation", &actions);
    let dlg_state = DialogState::default();
    dialog.draw(
        &mut ui,
        Rect::new(25, 8, 50, 12),
        &dlg_state,
        |_ui, _inner| {},
    );

    // Registered hit regions must include both dialog and menu
    assert!(ui.hit_regions.contains_key(&Id::new("top_dialog")));
    assert!(ui.hit_regions.contains_key(&Id::new("overlay_menu")));

    // 2. Narrow Geometry & Zero Area Resilience
    let zero_area = Rect::zero();
    let mut zero_ui = Ui::new(&theme, zero_area, &mut layers);

    let too_small = TooSmall::new(Id::new("too_small"), Size::new(80, 24));
    too_small.draw(&mut zero_ui, zero_area);

    let button = Button::new(Id::new("zero_btn"), "Test");
    button.draw(&mut zero_ui, zero_area);

    let split = SplitPane::new(Id::new("zero_split"), Axis::Horizontal);
    let split_state = SplitPaneState::new(0.5);
    let areas = split.layout(zero_area, &split_state);
    assert_eq!(areas.first.width, 0);
    assert_eq!(areas.second.width, 0);

    // 3. Dynamic Keyed Collection Reconciliation
    let initial_keys = [ItemKey::new(1), ItemKey::new(2), ItemKey::new(3)];
    let cursor = Some(ItemKey::new(2));
    let reconciled = reconcile_cursor(cursor, &initial_keys);
    assert_eq!(reconciled, Some(ItemKey::new(2)));

    // When item 2 is deleted, cursor falls back to nearest successor (item 3)
    let reduced_keys = [ItemKey::new(1), ItemKey::new(3)];
    let reconciled_after_del = reconcile_cursor_with_fallback(cursor, &initial_keys, &reduced_keys);
    assert_eq!(reconciled_after_del, Some(ItemKey::new(3)));
}
