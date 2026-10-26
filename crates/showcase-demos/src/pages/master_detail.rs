use termrock::controls::{
    Binding, MasterDetail as ResponsiveMasterDetail, MasterDetailLayout, MasterDetailMode,
    MasterDetailState, SplitCmd,
};
use termrock::{
    ActionKey, Button, Chord, Cx, FrameRead, Id, ItemKey, KeyCode, KeyModifiers, List, ListState,
    Part, PartRef, PropsAction, PropsList, PropsRow, PropsState, Rect, Response, ScrollRegion,
    ScrollState, Select, SelectMode, SelectState, StateFlags, Tabs, TabsState, Ui, Variant,
    author::VerticalWindow,
};

use super::{Page, PageStatus, PageUpdate, card_panel, frame};

const PAGE_TABS: Id = Id::root("showcase_ui::pages::master_detail.page_tabs");
const OPTIONS_SCROLL: Id = Id::root("showcase_ui::pages::master_detail.options_scroll");
const STATES_SCROLL: Id = Id::root("showcase_ui::pages::master_detail.states_scroll");
const BASE: Id = Id::root("showcase_ui::pages::master_detail.widget");

const TABS: &[&str] = &["Playground", "Options", "States"];
const WIDTH_ITEMS: &[&str] = &["fill", "100", "72"];
const COLLAPSE_ITEMS: &[&str] = &["80", "108", "140"];
const SIZE_ITEMS: &[&str] = &["resizable", "share", "master"];
const SUMMARY_ITEMS: &[&str] = &["1", "0", "3"];
const ESC_ITEMS: &[&str] = &["off", "on"];
const KEY_ITEMS: &[&str] = &["default", "custom", "none"];
const STEP_ITEMS: &[&str] = &["2 cells", "1 cell", "5 cells"];
const GAP_ITEMS: &[&str] = &["2", "1", "4"];

const OPTION_ROWS: usize = 16;
const STATE_COUNT: usize = 4;
const STATE_STRIDE: usize = 6;
const SPECIMEN_HEIGHT: u16 = 4;

const TAB_HINTS: &[(&str, &str)] = &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
const SELECT_HINTS: &[(&str, &str)] = &[("↑ ↓", "Choose"), ("Enter", "Open / choose")];
const SCROLL_HINTS: &[(&str, &str)] = &[("Wheel", "Scroll")];
const LIST_HINTS: &[(&str, &str)] = &[("↑ ↓", "Select table"), ("Tab", "Details")];
const LIST_HINTS_RESIZE_DEFAULT: &[(&str, &str)] = &[
    ("↑ ↓", "Select table"),
    ("Tab", "Details"),
    ("Alt+Shift+← →", "Resize panes"),
];
const LIST_HINTS_RESIZE_CUSTOM: &[(&str, &str)] = &[
    ("↑ ↓", "Select table"),
    ("Tab", "Details"),
    ("Ctrl+h / Ctrl+l", "Resize panes"),
];
const FACT_HINTS: &[(&str, &str)] = &[("↑ ↓", "Select fact")];
const COPY_HINTS: &[(&str, &str)] = &[("↑ ↓", "Select fact"), ("y", "Copy path")];
const OPEN_HINTS: &[(&str, &str)] = &[("Enter", "Open table")];
const TOGGLE_HINTS: &[(&str, &str)] = &[("Enter", "Show / hide details")];
const SPLIT_HINTS_DEFAULT: &[(&str, &str)] = &[("Alt+Shift+← →", "Resize panes")];
const SPLIT_HINTS_CUSTOM: &[(&str, &str)] = &[("Ctrl+h / Ctrl+l", "Resize panes")];

const DEFAULT_RESIZE_KEYS: &[Binding<SplitCmd>] = &[
    Binding {
        action: ActionKey::custom("master_detail.resize.shrink.default"),
        chord: Some(Chord::with(
            KeyCode::Left,
            KeyModifiers::ALT.union(KeyModifiers::SHIFT),
        )),
        cmd: SplitCmd::Shrink,
        label: "Narrower",
        priority: 50,
        visible: true,
    },
    Binding {
        action: ActionKey::custom("master_detail.resize.grow.default"),
        chord: Some(Chord::with(
            KeyCode::Right,
            KeyModifiers::ALT.union(KeyModifiers::SHIFT),
        )),
        cmd: SplitCmd::Grow,
        label: "Wider",
        priority: 50,
        visible: true,
    },
];

const CUSTOM_RESIZE_KEYS: &[Binding<SplitCmd>] = &[
    Binding {
        action: ActionKey::custom("master_detail.resize.shrink.custom"),
        chord: Some(Chord::with(KeyCode::Char('h'), KeyModifiers::CONTROL)),
        cmd: SplitCmd::Shrink,
        label: "Narrower",
        priority: 50,
        visible: true,
    },
    Binding {
        action: ActionKey::custom("master_detail.resize.grow.custom"),
        chord: Some(Chord::with(KeyCode::Char('l'), KeyModifiers::CONTROL)),
        cmd: SplitCmd::Grow,
        label: "Wider",
        priority: 50,
        visible: true,
    },
];

#[derive(Clone, Copy, Debug)]
struct Table {
    name: &'static str,
    rows: &'static str,
    size: &'static str,
    columns: &'static str,
    indexes: &'static str,
    owner: &'static str,
    analyzed: &'static str,
    path: &'static str,
}

type TableList = List<'static, Table, fn(&Table) -> ItemKey, fn(&Table, &mut termrock::RowUi<'_>)>;

const TABLES: &[Table] = &[
    Table {
        name: "orders",
        rows: "12,408",
        size: "3.1 MB",
        columns: "14",
        indexes: "4",
        owner: "app",
        analyzed: "2 h ago",
        path: "shop.public.orders",
    },
    Table {
        name: "order_items",
        rows: "48,221",
        size: "7.9 MB",
        columns: "9",
        indexes: "3",
        owner: "app",
        analyzed: "2 h ago",
        path: "shop.public.order_items",
    },
    Table {
        name: "customers",
        rows: "3,112",
        size: "820 kB",
        columns: "18",
        indexes: "5",
        owner: "app",
        analyzed: "1 d ago",
        path: "shop.public.customers",
    },
    Table {
        name: "products",
        rows: "942",
        size: "312 kB",
        columns: "12",
        indexes: "2",
        owner: "catalog",
        analyzed: "3 d ago",
        path: "shop.public.products",
    },
    Table {
        name: "invoices",
        rows: "10,870",
        size: "2.4 MB",
        columns: "11",
        indexes: "3",
        owner: "billing",
        analyzed: "5 h ago",
        path: "shop.public.invoices",
    },
    Table {
        name: "payments",
        rows: "10,412",
        size: "1.9 MB",
        columns: "10",
        indexes: "3",
        owner: "billing",
        analyzed: "5 h ago",
        path: "shop.public.payments",
    },
    Table {
        name: "shipments",
        rows: "9,655",
        size: "1.6 MB",
        columns: "13",
        indexes: "2",
        owner: "logistics",
        analyzed: "never",
        path: "shop.public.shipments",
    },
    Table {
        name: "audit_log",
        rows: "1,204,377",
        size: "412 MB",
        columns: "7",
        indexes: "1",
        owner: "ops",
        analyzed: "12 d ago",
        path: "shop.public.audit_log",
    },
    Table {
        name: "sessions",
        rows: "88,014",
        size: "21 MB",
        columns: "6",
        indexes: "2",
        owner: "auth",
        analyzed: "1 h ago",
        path: "shop.public.sessions",
    },
    Table {
        name: "feature_flags",
        rows: "37",
        size: "16 kB",
        columns: "5",
        indexes: "1",
        owner: "ops",
        analyzed: "9 d ago",
        path: "shop.public.feature_flags",
    },
];

#[derive(Clone, Copy)]
struct WidgetIds {
    root: Id,
    master: Id,
    detail: Id,
    props: Id,
    open: Id,
    toggle: Id,
    master_panel: Id,
    detail_panel: Id,
}

impl WidgetIds {
    const fn new(index: usize) -> Self {
        let root = BASE.index(index);
        let props = root.index(3);
        Self {
            root,
            master: root.index(1),

            detail: props,
            props,
            open: root.index(4),
            toggle: root.index(5),
            master_panel: root.index(6),
            detail_panel: root.index(7),
        }
    }
}

#[derive(Debug)]
pub struct MasterDetailPage {
    tabs: TabsState,
    detail_state: MasterDetailState,
    list_state: ListState,
    props_state: PropsState,
    width: usize,
    collapse: usize,
    size: usize,
    summary: usize,
    esc_returns: usize,
    keys: usize,
    step: usize,
    gap: usize,
    width_state: SelectState,
    collapse_state: SelectState,
    size_state: SelectState,
    summary_state: SelectState,
    esc_state: SelectState,
    keys_state: SelectState,
    step_state: SelectState,
    gap_state: SelectState,
    options_scroll: ScrollState,
    states_scroll: ScrollState,
}

impl MasterDetailPage {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tabs: TabsState::default(),
            detail_state: MasterDetailState::default(),
            list_state: ListState::default(),
            props_state: PropsState::default(),
            width: 0,
            collapse: 0,
            size: 0,
            summary: 0,
            esc_returns: 1,
            keys: 0,
            step: 0,
            gap: 0,
            width_state: SelectState::default(),
            collapse_state: SelectState::default(),
            size_state: SelectState::default(),
            summary_state: SelectState::default(),
            esc_state: SelectState::default(),
            keys_state: SelectState::default(),
            step_state: SelectState::default(),
            gap_state: SelectState::default(),
            options_scroll: ScrollState::default(),
            states_scroll: ScrollState::default(),
        }
    }

    fn section(&self) -> usize {
        match self.tabs.active().or(self.tabs.cursor()) {
            Some(ItemKey::Index(index)) => index,
            _ => 0,
        }
    }

    fn selected_table(&self) -> &'static Table {
        let key = self.list_state.cursor();
        TABLES
            .iter()
            .find(|table| key == Some(table_key(table)))
            .unwrap_or(&TABLES[0])
    }

    fn active_width(&self, available: u16) -> u16 {
        match self.width {
            1 => available.min(100),
            2 => available.min(72),
            _ => available,
        }
    }

    fn collapse_width(&self) -> u16 {
        [80, 108, 140][self.collapse.min(2)]
    }

    fn split_bindings(&self) -> &'static [Binding<SplitCmd>] {
        match self.keys {
            1 => CUSTOM_RESIZE_KEYS,
            2 => &[],
            _ => DEFAULT_RESIZE_KEYS,
        }
    }

    fn control(&self, ids: WidgetIds) -> ResponsiveMasterDetail {
        let owners = [ids.open];
        let control = ResponsiveMasterDetail::new(ids.root, ids.master, ids.detail)
            .master_share(40)
            .min_master(24)
            .min_detail(34)
            .collapse_below(self.collapse_width())
            .preserve_drag_offset(true)
            .summary_rows([1, 0, 3][self.summary.min(2)])
            .esc_returns(self.esc_returns == 1)
            .resize_step_cells([2, 1, 5][self.step.min(2)])
            .gap([2, 1, 4][self.gap.min(2)])
            .detail_owns(&owners);
        match self.size {
            1 => control
                .fixed_detail_share(40, 34, 56)
                .resize_bindings(self.split_bindings()),
            2 => control
                .fixed_master_share(33, 26, 40)
                .resize_bindings(self.split_bindings()),
            _ => control.resizable().resize_bindings(self.split_bindings()),
        }
    }

    fn update_options(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        response |= Select::new(WIDTH_ITEMS_ID)
            .popup_rows(3)
            .update_in_form(
                cx,
                &mut self.width_state,
                &mut self.width,
                WIDTH_ITEMS,
                false,
            )
            .erase();
        response |= Select::new(COLLAPSE_ITEMS_ID)
            .popup_rows(3)
            .update_in_form(
                cx,
                &mut self.collapse_state,
                &mut self.collapse,
                COLLAPSE_ITEMS,
                false,
            )
            .erase();
        response |= Select::new(SIZE_ITEMS_ID)
            .popup_rows(3)
            .update_in_form(cx, &mut self.size_state, &mut self.size, SIZE_ITEMS, false)
            .erase();
        response |= Select::new(SUMMARY_ITEMS_ID)
            .popup_rows(3)
            .update_in_form(
                cx,
                &mut self.summary_state,
                &mut self.summary,
                SUMMARY_ITEMS,
                self.width != 2,
            )
            .erase();
        response |= Select::new(ESC_ITEMS_ID)
            .popup_rows(2)
            .update_in_form(
                cx,
                &mut self.esc_state,
                &mut self.esc_returns,
                ESC_ITEMS,
                false,
            )
            .erase();
        response |= Select::new(KEY_ITEMS_ID)
            .popup_rows(3)
            .update_in_form(cx, &mut self.keys_state, &mut self.keys, KEY_ITEMS, false)
            .erase();
        response |= Select::new(STEP_ITEMS_ID)
            .popup_rows(3)
            .update_in_form(cx, &mut self.step_state, &mut self.step, STEP_ITEMS, false)
            .erase();
        response |= Select::new(GAP_ITEMS_ID)
            .popup_rows(3)
            .update_in_form(cx, &mut self.gap_state, &mut self.gap, GAP_ITEMS, false)
            .erase();
        response |=
            ScrollRegion::new(OPTIONS_SCROLL).update(cx, &mut self.options_scroll, OPTION_ROWS);
        response
    }

    fn update_playground(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<PageStatus>) {
        let ids = WidgetIds::new(0);
        let control = self.control(ids);
        let mut response = control.update(cx, &mut self.detail_state).erase();
        let list_response = tables_list(ids.master).update(cx, &mut self.list_state, TABLES);
        response |= list_response.erase();

        let table = self.selected_table();
        let rows = table_facts(table);
        let props_response =
            PropsList::new(ids.props)
                .sheet()
                .update(cx, &mut self.props_state, &rows);
        let mut status = None;
        if let Some(PropsAction::Copy(key)) = props_response.action_ref().copied()
            && key == path_key()
        {
            cx.clipboard_copy(table.path);
            status = Some(PageStatus(format!("Copied {}", table.path)));
        }
        response |= props_response.erase();

        let narrow = self.active_width(
            cx.layout(ids.root)
                .map_or(0, |facts| facts.logical_area.map_or(0, |area| area.width)),
        ) < self.collapse_width();
        let toggle_label = if self.detail_state.drawer_open() {
            "Hide details"
        } else if narrow {
            "Show details"
        } else {
            "Split view"
        };
        let toggle = Button::new(ids.toggle, toggle_label)
            .variant(Variant::SECONDARY)
            .disabled(!narrow);
        let toggle_response = toggle.update(cx);
        if toggle_response.activated() && narrow {
            control.toggle_drawer(&mut self.detail_state, cx);
        }
        response |= toggle_response.erase();

        if Button::new(ids.open, "Open table")
            .variant(Variant::SECONDARY)
            .update(cx)
            .activated()
        {
            status = Some(PageStatus(format!("Opened {}", table.name)));
            response |= Response::changed().for_id(ids.open);
        }
        (response, status)
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let ids = WidgetIds::new(0);
        let width = self.active_width(area.width);
        let area = Rect { width, ..area };
        let controls = Rect {
            height: 1.min(area.height),
            ..area
        };
        let control = self.control(ids);
        let narrow = width < self.collapse_width();
        let toggle_label = if self.detail_state.drawer_open() {
            "Hide details"
        } else if narrow {
            "Show details"
        } else {
            "Split view"
        };
        let _ = Button::new(ids.toggle, toggle_label)
            .variant(Variant::SECONDARY)
            .disabled(!narrow)
            .draw(ui, controls);
        let status_x = controls.x.saturating_add(20);
        let status_area = Rect::new(
            status_x,
            controls.y,
            controls.right().saturating_sub(status_x),
            controls.height,
        );
        if !status_area.is_empty() {
            let hint = if narrow {
                "10 tables · Tab opens the selected facts"
            } else {
                "10 tables · select a row to inspect its facts"
            };
            ui.paint_str(status_area, hint, ui.surface_style());
        }
        let body = Rect {
            y: area.y.saturating_add(1),
            height: area.height.saturating_sub(1),
            ..area
        };
        if body.is_empty() {
            return;
        }
        control.draw(ui, body, &self.detail_state, |ui, layout| {
            self.draw_layout(ui, layout, ids, self.selected_table());
        });
    }

    fn draw_layout(
        &self,
        ui: &mut Ui<'_>,
        layout: MasterDetailLayout,
        ids: WidgetIds,
        table: &Table,
    ) {
        match layout.mode {
            MasterDetailMode::Split => {
                self.draw_master(ui, layout.master, ids, table);
                self.draw_detail(ui, layout.detail, ids, table, false);
            }
            MasterDetailMode::Master => {
                self.draw_master(ui, layout.master, ids, table);
                self.draw_summary(ui, layout.summary, table);
            }
            MasterDetailMode::Drawer => self.draw_detail(ui, layout.detail, ids, table, true),
        }
    }

    fn draw_master(&self, ui: &mut Ui<'_>, area: Rect, ids: WidgetIds, _table: &Table) {
        if area.is_empty() {
            return;
        }
        card_panel(
            ui,
            ids.master_panel,
            "Tables",
            Some("10"),
            area,
            |ui, inner| {
                let _ = tables_list(ids.master).draw(ui, inner, &self.list_state, TABLES);
            },
        );
    }

    fn draw_detail(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        ids: WidgetIds,
        table: &Table,
        drawer: bool,
    ) {
        if area.is_empty() {
            return;
        }
        let meta = if drawer {
            (self.esc_returns == 1).then_some("Esc returns to tables")
        } else {
            Some("facts")
        };
        let rows = table_facts(table);
        card_panel(ui, ids.detail_panel, table.name, meta, area, |ui, inner| {
            if inner.is_empty() {
                return;
            }
            let button_height = u16::from(inner.height > 1);
            let fact_area = Rect {
                height: inner.height.saturating_sub(button_height),
                ..inner
            };
            let _ = PropsList::new(ids.props)
                .sheet()
                .draw(ui, fact_area, &self.props_state, &rows);
            if button_height > 0 {
                let button_area = Rect {
                    y: inner.bottom().saturating_sub(1),
                    height: 1,
                    ..inner
                };
                let _ = Button::new(ids.open, "Open table")
                    .variant(Variant::SECONDARY)
                    .draw(ui, button_area);
            }
        });
    }

    fn draw_summary(&self, ui: &mut Ui<'_>, area: Rect, table: &Table) {
        if area.is_empty() {
            return;
        }
        let lines = [
            format!("→ {} · {} rows · {}", table.name, table.rows, table.size),
            format!(
                "{} columns · {} indexes · owner {}",
                table.columns, table.indexes, table.owner
            ),
            "Tab shows details".to_owned(),
        ];
        for (index, line) in lines.iter().take(usize::from(area.height)).enumerate() {
            let Ok(y_offset) = u16::try_from(index) else {
                break;
            };
            let row = Rect {
                y: area.y.saturating_add(y_offset),
                height: 1,
                ..area
            };
            let clipped = termrock::truncate(line, area.width);
            ui.paint_str(row, &clipped, ui.surface_style());
        }
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let view =
            ScrollRegion::new(OPTIONS_SCROLL).draw(ui, area, &self.options_scroll, OPTION_ROWS);
        let Some(window) = scroll_window(view, OPTION_ROWS, &self.options_scroll) else {
            return;
        };
        ui.with_area(view, |ui| {
            for (index, (label, id)) in OPTION_LABELS.iter().enumerate() {
                if let Some(y) = window.physical_row_for_source(index.saturating_mul(2)) {
                    ui.paint_str(
                        Rect::new(view.x, y, view.width, 1),
                        label,
                        ui.surface_style(),
                    );
                }
                let Some(y) =
                    window.physical_row_for_source(index.saturating_mul(2).saturating_add(1))
                else {
                    continue;
                };
                let input = Rect::new(view.x, y, view.width, 1);
                let disabled = *id == SUMMARY_ITEMS_ID && self.width != 2;
                match *id {
                    WIDTH_ITEMS_ID => {
                        let _ = Select::new(*id).disabled(disabled).draw(
                            ui,
                            input,
                            &self.width_state,
                            WIDTH_ITEMS,
                        );
                    }
                    COLLAPSE_ITEMS_ID => {
                        let _ = Select::new(*id).disabled(disabled).draw(
                            ui,
                            input,
                            &self.collapse_state,
                            COLLAPSE_ITEMS,
                        );
                    }
                    SIZE_ITEMS_ID => {
                        let _ = Select::new(*id).disabled(disabled).draw(
                            ui,
                            input,
                            &self.size_state,
                            SIZE_ITEMS,
                        );
                    }
                    SUMMARY_ITEMS_ID => {
                        let _ = Select::new(*id).disabled(disabled).draw(
                            ui,
                            input,
                            &self.summary_state,
                            SUMMARY_ITEMS,
                        );
                    }
                    ESC_ITEMS_ID => {
                        let _ = Select::new(*id).disabled(disabled).draw(
                            ui,
                            input,
                            &self.esc_state,
                            ESC_ITEMS,
                        );
                    }
                    KEY_ITEMS_ID => {
                        let _ = Select::new(*id).disabled(disabled).draw(
                            ui,
                            input,
                            &self.keys_state,
                            KEY_ITEMS,
                        );
                    }
                    STEP_ITEMS_ID => {
                        let _ = Select::new(*id).disabled(disabled).draw(
                            ui,
                            input,
                            &self.step_state,
                            STEP_ITEMS,
                        );
                    }
                    GAP_ITEMS_ID => {
                        let _ = Select::new(*id).disabled(disabled).draw(
                            ui,
                            input,
                            &self.gap_state,
                            GAP_ITEMS,
                        );
                    }
                    _ => {}
                }
            }
        });
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let rows = state_content_rows();
        let view = ScrollRegion::new(STATES_SCROLL).draw(ui, area, &self.states_scroll, rows);
        let Some(window) = scroll_window(view, rows, &self.states_scroll) else {
            return;
        };
        let visible = window.visible_source_range();
        ui.with_area(view, |ui| {
            for (index, label) in STATE_LABELS.iter().copied().enumerate() {
                let start = index.saturating_mul(STATE_STRIDE);
                let end = start.saturating_add(1 + usize::from(SPECIMEN_HEIGHT));
                if start < visible.start || end > visible.end {
                    continue;
                }
                let Some(label_y) = window.physical_row_for_source(start) else {
                    continue;
                };
                ui.paint_str(
                    Rect::new(view.x, label_y, view.width, 1),
                    label,
                    ui.surface_style(),
                );
                let Some(sample_y) = window.physical_row_for_source(start.saturating_add(1)) else {
                    continue;
                };
                let sample_width = if index == 3 {
                    view.width.min(72)
                } else {
                    view.width
                };
                let sample = Rect::new(view.x, sample_y, sample_width, SPECIMEN_HEIGHT);
                self.draw_specimen(ui, sample, index);
            }
        });
    }

    fn draw_specimen(&self, ui: &mut Ui<'_>, area: Rect, index: usize) {
        if area.is_empty() {
            return;
        }
        let ids = WidgetIds::new(index.saturating_add(1));
        let state = match index {
            1 => Some(termrock::ReferenceState::HOVERED),
            2 => Some(termrock::ReferenceState::PRESSED),
            _ => None,
        }
        .map(|state| termrock::ReferenceTarget::new(ids.root, state).part(PartRef::of(Part::SEAM)));
        let control = ResponsiveMasterDetail::new(ids.root, ids.master, ids.detail)
            .master_share(40)
            .min_master(24)
            .min_detail(34)
            .gap(2)
            .collapse_below(if index == 3 { u16::MAX } else { 0 })
            .summary_rows(1)
            .detail_owns(&[ids.props, ids.open]);
        let selected = &TABLES[index.min(TABLES.len() - 1)];
        let list_state = ListState::default();
        let props_state = PropsState::default();
        ui.reference(state, |ui| {
            control.draw(
                ui,
                area,
                &MasterDetailState::default(),
                |ui, layout| match layout.mode {
                    MasterDetailMode::Split => {
                        let rows = table_facts(selected);
                        let _ =
                            tables_list(ids.master).draw(ui, layout.master, &list_state, TABLES);
                        let _ = PropsList::new(ids.props).sheet().draw(
                            ui,
                            layout.detail,
                            &props_state,
                            &rows,
                        );
                    }
                    MasterDetailMode::Master => {
                        let _ =
                            tables_list(ids.master).draw(ui, layout.master, &list_state, TABLES);
                        self.draw_summary(ui, layout.summary, selected);
                    }
                    MasterDetailMode::Drawer => {
                        let rows = table_facts(selected);
                        let _ = PropsList::new(ids.props).sheet().draw(
                            ui,
                            layout.detail,
                            &props_state,
                            &rows,
                        );
                    }
                },
            );
        });
    }
}

impl Default for MasterDetailPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for MasterDetailPage {
    fn title(&self) -> &'static str {
        "Master-detail"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Tabs::new(PAGE_TABS)
            .update(cx, &mut self.tabs, TABS)
            .erase();
        let mut status = None;
        match self.section() {
            0 => {
                let (page_response, page_status) = self.update_playground(cx);
                response |= page_response;
                status = page_status;
            }
            1 => response |= self.update_options(cx),
            _ => {
                response |= ScrollRegion::new(STATES_SCROLL).update(
                    cx,
                    &mut self.states_scroll,
                    state_content_rows(),
                );
            }
        }
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.with_area(area, |ui| {
            frame(
                ui,
                area,
                self.title(),
                "10 database tables · 8 live detail facts",
                |ui, body| {
                    if body.is_empty() {
                        return;
                    }
                    let tabs_height = body.height.min(2);
                    let tabs_area = Rect {
                        height: tabs_height,
                        ..body
                    };
                    let tabs = Tabs::new(PAGE_TABS);
                    let previous = ui.layout(PAGE_TABS).map(|facts| {
                        termrock::navigation::tabs::TabWindowReceipt {
                            viewport_len: facts.viewport_len,
                            content_len: facts.content_len,
                        }
                    });
                    let layout =
                        tabs.layout_default(tabs_area, ui.full(), &self.tabs, TABS, previous);
                    if tabs.draw_layout(ui, &layout).is_err() {
                        let _ = tabs.draw(ui, tabs_area, &self.tabs, TABS);
                    }
                    let content_offset = if self.section() == 2 { 2 } else { 3 };
                    let content = Rect {
                        y: body.y.saturating_add(content_offset),
                        height: body.height.saturating_sub(content_offset),
                        ..body
                    };
                    match self.section() {
                        0 => self.draw_playground(ui, content),
                        1 => self.draw_options(ui, content),
                        _ => self.draw_states(ui, content),
                    }
                },
            );
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(PAGE_TABS).contains(StateFlags::FOCUSED) {
            return TAB_HINTS;
        }
        if [
            WIDTH_ITEMS_ID,
            COLLAPSE_ITEMS_ID,
            SIZE_ITEMS_ID,
            SUMMARY_ITEMS_ID,
            ESC_ITEMS_ID,
            KEY_ITEMS_ID,
            STEP_ITEMS_ID,
            GAP_ITEMS_ID,
        ]
        .iter()
        .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            return SELECT_HINTS;
        }
        let ids = WidgetIds::new(0);
        if ui.state(ids.toggle).contains(StateFlags::FOCUSED) {
            return TOGGLE_HINTS;
        }
        if ui.state(ids.open).contains(StateFlags::FOCUSED) {
            return OPEN_HINTS;
        }
        if ui.state(ids.props).contains(StateFlags::FOCUSED) {
            return if self.props_state.cursor() == Some(path_key()) {
                COPY_HINTS
            } else {
                FACT_HINTS
            };
        }
        if ui.state(ids.master).contains(StateFlags::FOCUSED) {
            if self.size != 0
                || self.active_width(ui.full().width) < self.collapse_width()
                || self.keys == 2
            {
                return LIST_HINTS;
            }
            return match self.keys {
                1 => LIST_HINTS_RESIZE_CUSTOM,
                _ => LIST_HINTS_RESIZE_DEFAULT,
            };
        }
        if ui.state(ids.root).contains(StateFlags::FOCUSED) {
            return match self.keys {
                1 => SPLIT_HINTS_CUSTOM,
                2 => &[],
                _ => SPLIT_HINTS_DEFAULT,
            };
        }
        if ui.state(OPTIONS_SCROLL).contains(StateFlags::FOCUSED)
            || ui.state(STATES_SCROLL).contains(StateFlags::FOCUSED)
        {
            return SCROLL_HINTS;
        }
        match self.section() {
            0 => LIST_HINTS,
            1 => SELECT_HINTS,
            _ => SCROLL_HINTS,
        }
    }
}

const WIDTH_ITEMS_ID: Id = Id::root("showcase_ui::pages::master_detail.option.width");
const COLLAPSE_ITEMS_ID: Id = Id::root("showcase_ui::pages::master_detail.option.collapse");
const SIZE_ITEMS_ID: Id = Id::root("showcase_ui::pages::master_detail.option.size");
const SUMMARY_ITEMS_ID: Id = Id::root("showcase_ui::pages::master_detail.option.summary");
const ESC_ITEMS_ID: Id = Id::root("showcase_ui::pages::master_detail.option.esc");
const KEY_ITEMS_ID: Id = Id::root("showcase_ui::pages::master_detail.option.keys");
const STEP_ITEMS_ID: Id = Id::root("showcase_ui::pages::master_detail.option.step");
const GAP_ITEMS_ID: Id = Id::root("showcase_ui::pages::master_detail.option.gap");

const OPTION_LABELS: [(&str, Id); 8] = [
    ("Demo width · masterdetail.width", WIDTH_ITEMS_ID),
    ("Collapses below · masterdetail.collapse", COLLAPSE_ITEMS_ID),
    ("Pane sizing · masterdetail.size", SIZE_ITEMS_ID),
    ("Narrow summary rows · width 72 only", SUMMARY_ITEMS_ID),
    (
        "Esc returns to tables · masterdetail.esc_returns",
        ESC_ITEMS_ID,
    ),
    ("Resize keys · masterdetail.keys", KEY_ITEMS_ID),
    ("Resize step · masterdetail.step", STEP_ITEMS_ID),
    ("Gap between panes · masterdetail.gap", GAP_ITEMS_ID),
];

const STATE_LABELS: [&str; STATE_COUNT] = ["Rest", "Hover", "Pressed", "Narrow"];

fn table_key(table: &Table) -> ItemKey {
    ItemKey::text(table.name)
}

fn table_row(table: &Table, row: &mut termrock::RowUi<'_>) {
    let meta = format!("{} rows · {}", table.rows, table.size);
    row.meta(&meta);
    row.label(table.name);
}

fn tables_list(id: Id) -> TableList {
    let key: fn(&Table) -> ItemKey = table_key;
    let row: fn(&Table, &mut termrock::RowUi<'_>) = table_row;
    List::new(id)
        .key(key)
        .row(row)
        .select_mode(SelectMode::Single)
}

fn path_key() -> ItemKey {
    ItemKey::text("path")
}

fn table_facts(table: &Table) -> [PropsRow<'static>; 8] {
    [
        PropsRow::new(ItemKey::text("schema"), "Schema", "public"),
        PropsRow::new(ItemKey::text("rows"), "Rows", table.rows),
        PropsRow::new(ItemKey::text("size"), "Size", table.size),
        PropsRow::new(ItemKey::text("columns"), "Columns", table.columns),
        PropsRow::new(ItemKey::text("indexes"), "Indexes", table.indexes),
        PropsRow::new(ItemKey::text("owner"), "Owner", table.owner),
        PropsRow::new(ItemKey::text("analyzed"), "Analyzed", table.analyzed),
        PropsRow::new(path_key(), "Path", table.path).copyable(),
    ]
}

fn state_content_rows() -> usize {
    STATE_COUNT.saturating_mul(STATE_STRIDE)
}

fn scroll_window(view: Rect, logical_rows: usize, state: &ScrollState) -> Option<VerticalWindow> {
    if view.width == 0 || view.height == 0 || logical_rows == 0 {
        return None;
    }
    let first_source_row = ScrollRegion::view(state, view, logical_rows)
        .offset()
        .min(logical_rows);
    let visible_rows = usize::from(view.height).min(logical_rows.saturating_sub(first_source_row));
    if visible_rows == 0 {
        return None;
    }
    let visible_body = Rect {
        height: u16::try_from(visible_rows).ok()?,
        ..view
    };
    VerticalWindow::new(visible_body, logical_rows, first_source_row).ok()
}
