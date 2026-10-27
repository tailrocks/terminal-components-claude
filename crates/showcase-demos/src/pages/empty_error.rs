use termrock::{
    ActionKey, Anchor, BindingState, Bindings, Button, CellRef, Column, ColumnKey, ContextMenu,
    CrossAlign, Cx, EditIntent, Empty, EmptyState, FieldError, FrameRead, Grid, GridAction,
    GridCmd, GridEditor, GridModel, GridState, Id, Intent, ItemKey, List, ListState, MenuAction,
    MenuItem, MenuState, NavUnit, Part, PartRef, Rect, ReferenceState, ReferenceTarget, Response,
    RowTotal, ScreenAlign, ScrollRegion, ScrollState, Select, SelectState, Side, StateFlags, Tabs,
    TabsState, Ui, Variant, author::VerticalWindow,
};

use super::{Page, PageStatus, PageUpdate, card_panel, frame};

const PAGE_TABS: Id = Id::root("showcase_ui::pages::empty_error.page_tabs");
const OPTION_KIND: Id = Id::root("showcase_ui::pages::empty_error.option.kind");
const OPTION_HINT: Id = Id::root("showcase_ui::pages::empty_error.option.hint");
const OPTION_ACTIONS: Id = Id::root("showcase_ui::pages::empty_error.option.actions");
const OPTION_REASON: Id = Id::root("showcase_ui::pages::empty_error.option.reason");
const SAVED_PANEL: Id = Id::root("showcase_ui::pages::empty_error.saved_panel");
const SAVED_LIST: Id = Id::root("showcase_ui::pages::empty_error.saved_queries");
const SEARCH_PANEL: Id = Id::root("showcase_ui::pages::empty_error.search_panel");
const SEARCH_EMPTY: Id = Id::root("showcase_ui::pages::empty_error.search_results");
const CONNECTION_PANEL: Id = Id::root("showcase_ui::pages::empty_error.connection_panel");
const CONNECTION_EMPTY: Id = Id::root("showcase_ui::pages::empty_error.connections");
const CONNECTION_PRIMARY: Id = Id::root("showcase_ui::pages::empty_error.connection.primary");
const CONNECTION_IMPORT: Id = Id::root("showcase_ui::pages::empty_error.connection.import");
const ORDERS_PANEL: Id = Id::root("showcase_ui::pages::empty_error.orders.panel");
const ORDERS_GRID: Id = Id::root("showcase_ui::pages::empty_error.orders.grid");
const ROW_ACTIONS: Id = Id::root("showcase_ui::pages::empty_error.orders.actions");
const ROW_MENU: Id = Id::root("showcase_ui::pages::empty_error.orders.menu");
const STATES_SCROLL: Id = Id::root("showcase_ui::pages::empty_error.states.scroll");
const GALLERY_EMPTY: Id = Id::root("showcase_ui::pages::empty_error.gallery.empty");
const GALLERY_ACTION: Id = Id::root("showcase_ui::pages::empty_error.gallery.action");

const KIND_ITEMS: &[&str] = &["Empty", "Error"];
const HINT_ITEMS: &[&str] = &["Off", "On"];
const ACTION_ITEMS: &[&str] = &["None", "One", "Two"];
const REASON_ITEMS: &[&str] = &["View", "No primary key", "Query result"];
const PAGE_SECTIONS: &[&str] = &["Playground", "Options", "States"];
const EMPTY_SAVED_QUERIES: &[&str] = &[];
const GALLERY_COLUMN_LABELS: [&str; 3] = ["Title", "Title + hint", "With actions"];
const GALLERY_ROW_LABELS: [&str; 7] = [
    "Rest", "Hover", "Pressed", "Focused", "Error", "Overflow", "Narrow",
];
const GALLERY_LABEL_WIDTH: u16 = 14;
const GALLERY_HEADER_ROWS: usize = 1;
const GALLERY_SPECIMEN_ROWS: usize = 3;
const STATES_HEIGHT: usize = GALLERY_HEADER_ROWS + GALLERY_ROW_LABELS.len() * GALLERY_SPECIMEN_ROWS;

const GRID_HINTS: &[(&str, &str)] = &[
    ("↑↓←→", "Cell"),
    ("Enter / F2", "Read-only reason"),
    ("Ctrl+C", "Copy preview"),
];
const BUTTON_HINTS: &[(&str, &str)] = &[("Enter", "Activate")];
const SELECT_HINTS: &[(&str, &str)] = &[("↑↓", "Choose"), ("Enter", "Open / choose")];
const MENU_HINTS: &[(&str, &str)] = &[("↑↓", "Choose"), ("Enter", "Activate"), ("Esc", "Close")];
const PAGE_HINTS: &[(&str, &str)] = &[("← →", "Section"), ("Enter", "Activate")];

const ORDERS: [OrderRow; 2] = [
    OrderRow {
        key: 1041,
        id: "1041",
        customer: "Acme Corp",
        total: "1204.5",
    },
    OrderRow {
        key: 1042,
        id: "1042",
        customer: "Globex",
        total: "88.0",
    },
];

const ORDER_COLUMNS: [Column<'static>; 3] = [
    Column {
        key: ColumnKey::num(0),
        title: "Order",
        subtitle: None,
        align: termrock::Align::Right,
        min_width: 6,
        max_width: 8,
        sortable: false,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
    Column {
        key: ColumnKey::num(1),
        title: "Customer",
        subtitle: None,
        align: termrock::Align::Left,
        min_width: 14,
        max_width: 24,
        sortable: false,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
    Column {
        key: ColumnKey::num(2),
        title: "Total",
        subtitle: None,
        align: termrock::Align::Right,
        min_width: 9,
        max_width: 12,
        sortable: false,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
];

const EDIT_CELL: ActionKey = ActionKey::custom("showcase.empty_error.orders.edit_cell");
const DUPLICATE_ROW: ActionKey = ActionKey::custom("showcase.empty_error.orders.duplicate_row");
const DELETE_ROW: ActionKey = ActionKey::custom("showcase.empty_error.orders.delete_row");
const COPY_ROW: ActionKey = ActionKey::custom("showcase.empty_error.orders.copy_row");
const COPY_CSV: ActionKey = ActionKey::custom("showcase.empty_error.orders.copy_csv");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OrderRow {
    key: u64,
    id: &'static str,
    customer: &'static str,
    total: &'static str,
}

#[derive(Clone, Debug)]
struct OrdersModel {
    reason: &'static str,
}

impl GridModel for OrdersModel {
    fn row_count(&self) -> usize {
        ORDERS.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        ORDERS
            .get(row)
            .map_or(ItemKey::num(0), |order| ItemKey::num(order.key))
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let order = ORDERS.get(row)?;
        match col {
            0 => Some(CellRef::new(order.id).align(termrock::Align::Right)),
            1 => Some(CellRef::new(order.customer)),
            2 => Some(CellRef::new(order.total).align(termrock::Align::Right)),
            _ => None,
        }
    }

    fn total(&self) -> RowTotal {
        RowTotal::Exact(ORDERS.len())
    }

    fn read_only_reason(&self) -> Option<&str> {
        Some(self.reason)
    }

    fn revision(&self) -> Option<u64> {
        Some(1)
    }
}

impl GridEditor for OrdersModel {
    fn edit_intent(&self, _row: usize, _col: usize) -> EditIntent<'_> {
        EditIntent::Refuse {
            reason: self.reason,
        }
    }

    fn apply_cycle(&mut self, _row: usize, _col: usize) {}

    fn commit_cell(&mut self, _row: usize, _col: usize, _text: &str) -> Result<(), FieldError> {
        Err(FieldError::new("Orders are read-only"))
    }

    fn is_editable(&self, row: usize, col: usize) -> bool {
        row < ORDERS.len() && col < ORDER_COLUMNS.len()
    }
}

#[derive(Debug)]
pub struct EmptyErrorPage {
    page_tabs: TabsState,
    kind: usize,
    hint: usize,
    actions: usize,
    reason: usize,
    kind_state: SelectState,
    hint_state: SelectState,
    actions_state: SelectState,
    reason_state: SelectState,
    saved_state: ListState,
    grid_state: GridState,
    orders: OrdersModel,
    menu_items: [MenuItem<'static>; 5],
    menu_state: MenuState,
    states_scroll: ScrollState,
}

impl EmptyErrorPage {
    #[must_use]
    pub fn new() -> Self {
        let reason = "orders_view is a view: views cannot be edited";
        Self {
            page_tabs: TabsState::default(),
            kind: 0,
            hint: 1,
            actions: 2,
            reason: 0,
            kind_state: SelectState::default(),
            hint_state: SelectState::default(),
            actions_state: SelectState::default(),
            reason_state: SelectState::default(),
            saved_state: ListState::default(),
            grid_state: GridState::default(),
            orders: OrdersModel { reason },
            menu_items: menu_items("read-only view"),
            menu_state: MenuState::default(),
            states_scroll: ScrollState::default(),
        }
    }

    fn section(&self) -> usize {
        match self.page_tabs.active().or(self.page_tabs.cursor()) {
            Some(ItemKey::Index(index)) => index,
            _ => 0,
        }
    }

    fn read_only_reason(&self) -> &'static str {
        match self.reason.min(2) {
            1 => "orders_view has no primary key, so rows cannot be told apart",
            2 => "A query result is read-only: edit the table instead",
            _ => "orders_view is a view: views cannot be edited",
        }
    }

    fn short_reason(&self) -> &'static str {
        match self.reason.min(2) {
            1 => "no primary key",
            2 => "query result",
            _ => "read-only view",
        }
    }

    fn connection(&self) -> EmptyState<'static> {
        if self.kind == 1 {
            EmptyState::Error {
                message: "Could not load connections",
                detail: (self.hint == 1)
                    .then_some("The config file is unreadable: check its permissions"),
            }
        } else {
            EmptyState::Empty {
                title: "No connections yet",
                hint: (self.hint == 1)
                    .then_some("A connection holds a host, a user and a database"),
            }
        }
    }

    fn context_menu(&self) -> ContextMenu<'_> {
        context_menu(&self.menu_items)
    }

    fn update_options(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Select::new(OPTION_KIND)
            .popup_rows(2)
            .update_in_form(cx, &mut self.kind_state, &mut self.kind, KIND_ITEMS, false)
            .erase();
        response |= Select::new(OPTION_HINT)
            .popup_rows(2)
            .update_in_form(cx, &mut self.hint_state, &mut self.hint, HINT_ITEMS, false)
            .erase();
        response |= Select::new(OPTION_ACTIONS)
            .popup_rows(3)
            .update_in_form(
                cx,
                &mut self.actions_state,
                &mut self.actions,
                ACTION_ITEMS,
                false,
            )
            .erase();
        response |= Select::new(OPTION_REASON)
            .popup_rows(3)
            .update_in_form(
                cx,
                &mut self.reason_state,
                &mut self.reason,
                REASON_ITEMS,
                false,
            )
            .erase();
        self.orders.reason = self.read_only_reason();
        self.menu_items = menu_items(self.short_reason());
        response
    }

    fn update_playground(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<PageStatus>) {
        let mut response = saved_list()
            .update(cx, &mut self.saved_state, EMPTY_SAVED_QUERIES)
            .erase();

        let mut status = None;
        if self.actions > 0 {
            let primary = connection_primary(self.kind).update(cx);
            if primary.activated() {
                status = Some(PageStatus(if self.kind == 1 {
                    "Retry selected for the connection sample".to_owned()
                } else {
                    "New connection action selected".to_owned()
                }));
            }
            response |= primary.erase();
        }

        if self.actions >= 2 {
            let import = Button::new(CONNECTION_IMPORT, "Import…").variant(Variant::SECONDARY);
            let update = import.update(cx);
            if update.activated() {
                status = Some(PageStatus("Import action selected".to_owned()));
            }
            response |= update.erase();
        }

        let menu_items = menu_items(self.short_reason());
        let menu = context_menu(&menu_items);
        let menu_response = menu.update(cx, &mut self.menu_state);
        if let Some(MenuAction::Chosen(action)) = menu_response.action_ref().copied() {
            status = Some(PageStatus(match action {
                COPY_ROW => self.copy_row_preview(),
                COPY_CSV => self.copy_csv_preview(),
                _ => "Row action selected".to_owned(),
            }));
        }
        response |= menu_response.erase();

        let actions_button = Button::new(ROW_ACTIONS, "Row actions ▾").variant(Variant::SECONDARY);
        let actions_update = actions_button.update(cx);
        if actions_update.activated()
            && let Some(layer) = self.context_menu().try_layer(cx)
        {
            cx.open_layer(ROW_MENU, layer);
            status = Some(PageStatus("Row actions opened".to_owned()));
        }
        response |= actions_update.erase();

        let grid = orders_grid();
        let refusal_requested = cx.intents(ORDERS_GRID).any(|intent| match intent {
            Intent::Binding(action) => {
                grid.bindings(BindingState::default())
                    .iter()
                    .any(|binding| {
                        binding.action == action
                            && matches!(binding.cmd, GridCmd::Activate | GridCmd::BeginEdit)
                    })
            }
            _ => false,
        });
        let grid_response = grid.update_editable(cx, &mut self.grid_state, &mut self.orders);
        if refusal_requested
            && grid_response.is_changed()
            && let Some(error) = self.grid_state.edit_error()
        {
            status = Some(PageStatus(error.to_string()));
        }
        if let Some(GridAction::Copy(text)) = grid_response.action_ref() {
            status = Some(PageStatus(format!(
                "Grid copy preview (not written): {text}"
            )));
        }
        response |= grid_response.erase();
        (response, status)
    }

    fn copy_row_preview(&self) -> String {
        let key = self
            .grid_state
            .cursor()
            .and_then(|(key, _)| match key {
                ItemKey::Num(key) => Some(key),
                _ => None,
            })
            .unwrap_or(ORDERS[0].key);
        let row = ORDERS
            .iter()
            .find(|row| row.key == key)
            .unwrap_or(&ORDERS[0]);
        format!(
            "Copy preview (not written): {}\t{}\t{}",
            row.id, row.customer, row.total
        )
    }

    fn copy_csv_preview(&self) -> String {
        let mut text = String::from("CSV preview (not written): order,customer,total");
        for row in ORDERS {
            text.push_str(&format!("\n{},{},{}", row.id, row.customer, row.total));
        }
        text
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let gap = 2u16.min(area.width);
        let usable_width = area.width.saturating_sub(gap);
        let left_width = (u32::from(usable_width).saturating_mul(3) / 5) as u16;
        let right_x = area.x.saturating_add(left_width).saturating_add(gap);
        let right_width = area.right().saturating_sub(right_x);
        let left = Rect::new(area.x, area.y, left_width, area.height);
        let right = Rect::new(right_x, area.y, right_width, area.height);

        let saved_height = 11.min(left.height);
        card_panel(
            ui,
            SAVED_PANEL,
            "Saved queries",
            Some("List::empty"),
            Rect {
                height: saved_height,
                ..left
            },
            |ui, inner| {
                saved_list().draw(ui, inner, &self.saved_state, EMPTY_SAVED_QUERIES);
            },
        );
        let search_y = left.y.saturating_add(saved_height).saturating_add(1);
        let preferred_search_height = if left.height <= 21 { 3 } else { 5 };
        let search_height = preferred_search_height.min(left.bottom().saturating_sub(search_y));
        card_panel(
            ui,
            SEARCH_PANEL,
            "Search results",
            Some("no hint, no action"),
            Rect::new(left.x, search_y, left.width, search_height),
            |ui, inner| {
                Empty::new(
                    SEARCH_EMPTY,
                    EmptyState::Empty {
                        title: "Nothing matches “ordrs”",
                        hint: None,
                    },
                )
                .draw(ui, inner);
            },
        );
        let connection_y = search_y.saturating_add(search_height).saturating_add(1);
        let connection_height = left.bottom().saturating_sub(connection_y);
        card_panel(
            ui,
            CONNECTION_PANEL,
            "Connections",
            Some("EmptyState + actions"),
            Rect::new(left.x, connection_y, left.width, connection_height),
            |ui, inner| self.draw_connection(ui, inner),
        );

        card_panel(
            ui,
            ORDERS_PANEL,
            "Orders",
            Some("orders_view · read-only"),
            right,
            |ui, inner| self.draw_orders(ui, inner),
        );
    }

    fn draw_connection(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let button_row = self.actions > 0 && area.height >= 2;
        let empty_height = if button_row {
            area.height.saturating_sub(2)
        } else {
            area.height
        };
        Empty::new(CONNECTION_EMPTY, self.connection()).draw(
            ui,
            Rect {
                height: empty_height,
                ..area
            },
        );
        if !button_row {
            return;
        }
        let y = area.bottom().saturating_sub(1);
        if self.actions >= 2 {
            let gap = 1u16.min(area.width);
            let first_width = area.width.saturating_sub(gap) / 2;
            connection_primary(self.kind).draw(ui, Rect::new(area.x, y, first_width, 1));
            Button::new(CONNECTION_IMPORT, "Import…")
                .variant(Variant::SECONDARY)
                .draw(
                    ui,
                    Rect::new(
                        area.x.saturating_add(first_width).saturating_add(gap),
                        y,
                        area.right()
                            .saturating_sub(area.x.saturating_add(first_width).saturating_add(gap)),
                        1,
                    ),
                );
        } else {
            connection_primary(self.kind).draw(ui, Rect::new(area.x, y, area.width, 1));
        }
    }

    fn draw_orders(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let action_row = Rect::new(area.x, area.y, area.width, 1);
        Button::new(ROW_ACTIONS, "Row actions ▾")
            .variant(Variant::SECONDARY)
            .draw(ui, action_row);
        if area.height < 3 {
            return;
        }
        let note_rows = usize::from(area.height >= 4);
        let grid_y = area.y.saturating_add(1);
        let grid_height = area
            .height
            .saturating_sub(1)
            .saturating_sub(note_rows as u16);
        if grid_height > 0 {
            orders_grid().draw(
                ui,
                Rect::new(area.x, grid_y, area.width, grid_height),
                &self.grid_state,
                &self.orders,
            );
        }
        if note_rows > 0 {
            let y = area.bottom().saturating_sub(1);
            let note = "Enter / F2 explains the read-only rule · Ctrl+C previews a copy";
            let _ = ui.paint_str(
                Rect::new(area.x, y, area.width, 1),
                note,
                ui.surface_style(),
            );
        }
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let label_width = 18.min(area.width / 3);
        let field_x = area.x.saturating_add(label_width).saturating_add(1);
        let field_width = area.right().saturating_sub(field_x);
        let rows = [
            ("Connection state", OPTION_KIND),
            ("Show hint", OPTION_HINT),
            ("Actions", OPTION_ACTIONS),
            ("Read-only reason", OPTION_REASON),
        ];
        for (index, (label, id)) in rows.into_iter().enumerate() {
            let Ok(offset) = u16::try_from(index) else {
                break;
            };
            let y = area.y.saturating_add(offset);
            if y >= area.bottom() {
                break;
            }
            let _ = ui.paint_str(
                Rect::new(area.x, y, label_width, 1),
                label,
                ui.surface_style(),
            );
            let field = Rect::new(field_x, y, field_width, 1);
            match id {
                OPTION_KIND => {
                    let _ = Select::new(id).draw(ui, field, &self.kind_state, KIND_ITEMS);
                }
                OPTION_HINT => {
                    let _ = Select::new(id).draw(ui, field, &self.hint_state, HINT_ITEMS);
                }
                OPTION_ACTIONS => {
                    let _ = Select::new(id).draw(ui, field, &self.actions_state, ACTION_ITEMS);
                }
                OPTION_REASON => {
                    let _ = Select::new(id).draw(ui, field, &self.reason_state, REASON_ITEMS);
                }
                _ => {}
            }
        }
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let view =
            ScrollRegion::new(STATES_SCROLL).draw(ui, area, &self.states_scroll, STATES_HEIGHT);
        if view.is_empty() {
            return;
        }
        let projected = ScrollRegion::view(&self.states_scroll, view, STATES_HEIGHT);
        let first_row = projected.offset().min(STATES_HEIGHT);
        let logical_height = u16::try_from(STATES_HEIGHT).unwrap_or(u16::MAX);
        let visible = Rect {
            height: view.height.min(logical_height),
            ..view
        };
        let Some(window) = VerticalWindow::new(visible, STATES_HEIGHT, first_row).ok() else {
            return;
        };
        let source_rows = window.visible_source_range();
        let view_width = view.width;
        let label_width = GALLERY_LABEL_WIDTH.min(view_width);
        let grid_x = view.x.saturating_add(label_width);
        let grid_width = view_width.saturating_sub(label_width);
        let col_width = grid_width / GALLERY_COLUMN_LABELS.len() as u16;
        let remainder = grid_width % GALLERY_COLUMN_LABELS.len() as u16;

        if let Some(y) = window.physical_row_for_source(0) {
            let _ = ui.paint_str(
                Rect::new(view.x, y, label_width, 1),
                "State",
                ui.surface_style(),
            );
            let mut x = grid_x;
            for (col, label) in GALLERY_COLUMN_LABELS.iter().enumerate() {
                let width = col_width.saturating_add(u16::from((col as u16) < remainder));
                let _ = ui.paint_str(Rect::new(x, y, width, 1), label, ui.surface_style());
                x = x.saturating_add(width);
            }
        }

        for (row, label) in GALLERY_ROW_LABELS.iter().enumerate() {
            let source_y =
                GALLERY_HEADER_ROWS.saturating_add(row.saturating_mul(GALLERY_SPECIMEN_ROWS));
            let source_end = source_y.saturating_add(GALLERY_SPECIMEN_ROWS);
            if source_y < source_rows.start || source_end > source_rows.end {
                continue;
            }
            let Some(y) = window.physical_row_for_source(source_y) else {
                continue;
            };
            if let Some(label_y) = window.physical_row_for_source(source_y.saturating_add(1)) {
                let _ = ui.paint_str(
                    Rect::new(view.x, label_y, label_width, 1),
                    label,
                    ui.surface_style(),
                );
            }
            let mut x = grid_x;
            for col in 0..GALLERY_COLUMN_LABELS.len() {
                let width = col_width.saturating_add(u16::from((col as u16) < remainder));
                if width == 0 {
                    x = x.saturating_add(width);
                    continue;
                }
                let cell = Rect::new(x, y, width, GALLERY_SPECIMEN_ROWS as u16);
                self.draw_specimen(ui, cell, row, col);
                x = x.saturating_add(width);
            }
        }
    }

    fn draw_specimen(&self, ui: &mut Ui<'_>, mut area: Rect, row: usize, col: usize) {
        let interactive_row = matches!(row, 1..=3);
        if interactive_row && col < 2 {
            let _ = ui.paint_str(
                Rect::new(area.x, area.y.saturating_add(1), area.width, 1),
                "N/A · no action",
                ui.surface_style(),
            );
            return;
        }
        let empty_id = specimen_id(GALLERY_EMPTY, row, col);
        let action_id = specimen_id(GALLERY_ACTION, row, col);
        if row == 6 {
            let narrow = area.width.min(12);
            area.x = area.x.saturating_add(area.width.saturating_sub(narrow) / 2);
            area.width = narrow;
        }
        let (state, action) = specimen_state(row, col);
        let reference = if col == 2 {
            match row {
                1 => Some(ReferenceState::HOVERED),
                2 => Some(ReferenceState::PRESSED),
                3 => Some(ReferenceState::FOCUSED),
                _ => None,
            }
            .map(|state| ReferenceTarget::new(action_id, state))
        } else {
            None
        };
        ui.reference(reference, |ui| {
            let empty_area = if col == 2 {
                Rect {
                    height: area.height.saturating_sub(1),
                    ..area
                }
            } else {
                area
            };
            Empty::new(empty_id, state).draw(ui, empty_area);
            if col == 2 && area.height > 0 {
                Button::new(action_id, action)
                    .variant(Variant::SECONDARY)
                    .draw(
                        ui,
                        Rect {
                            y: area.bottom().saturating_sub(1),
                            height: 1,
                            ..area
                        },
                    );
            }
        });
    }
}

impl Default for EmptyErrorPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for EmptyErrorPage {
    fn title(&self) -> &'static str {
        "Empty & error"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Tabs::new(PAGE_TABS)
            .update(cx, &mut self.page_tabs, PAGE_SECTIONS)
            .erase();
        let mut status = None;
        match self.section() {
            0 => {
                let (playground, page_status) = self.update_playground(cx);
                response |= playground;
                status = page_status;
            }
            1 => response |= self.update_options(cx),
            _ => {
                response |= ScrollRegion::new(STATES_SCROLL)
                    .update(cx, &mut self.states_scroll, STATES_HEIGHT)
                    .erase();
            }
        }
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Empty recovery, read-only explanations, and applicable reference states",
            |ui, body| {
                if body.is_empty() {
                    return;
                }
                let tabs_area = Rect { height: 1, ..body };
                let tabs = Tabs::new(PAGE_TABS);
                let previous = ui.layout(PAGE_TABS).map(|facts| {
                    termrock::navigation::tabs::TabWindowReceipt {
                        viewport_len: facts.viewport_len,
                        content_len: facts.content_len,
                    }
                });
                let layout = tabs.layout_default(
                    tabs_area,
                    ui.full(),
                    &self.page_tabs,
                    PAGE_SECTIONS,
                    previous,
                );
                if tabs.draw_layout(ui, &layout).is_err() {
                    let _ = tabs.draw(ui, tabs_area, &self.page_tabs, PAGE_SECTIONS);
                }
                let content = Rect {
                    y: body.y.saturating_add(1),
                    height: body.height.saturating_sub(1),
                    ..body
                };
                match self.section() {
                    0 => self.draw_playground(ui, content),
                    1 => self.draw_options(ui, content),
                    _ => self.draw_states(ui, content),
                }
            },
        );
        let menu = self.context_menu();
        let _ = ui.layer(ROW_MENU, |ui, area| {
            menu.draw(ui, area, &self.menu_state);
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(ROW_MENU).contains(StateFlags::FOCUSED) {
            return MENU_HINTS;
        }
        if ui.state(PAGE_TABS).contains(StateFlags::FOCUSED) {
            return PAGE_HINTS;
        }
        if ui.state(ORDERS_GRID).contains(StateFlags::FOCUSED) {
            return GRID_HINTS;
        }
        if [CONNECTION_PRIMARY, CONNECTION_IMPORT, ROW_ACTIONS]
            .iter()
            .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            return BUTTON_HINTS;
        }
        if [OPTION_KIND, OPTION_HINT, OPTION_ACTIONS, OPTION_REASON]
            .iter()
            .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            return SELECT_HINTS;
        }
        &[]
    }
}

fn saved_list() -> List<'static, &'static str> {
    List::new(SAVED_LIST).empty(EmptyState::Empty {
        title: "No saved queries",
        hint: Some("Queries you save appear here"),
    })
}

fn context_menu<'a>(items: &'a [MenuItem<'a>]) -> ContextMenu<'a> {
    ContextMenu::new(ROW_MENU, items, Anchor::Screen(ScreenAlign::UpperThird))
        .title("Row actions")
        .skip_disabled(false)
        .anchor_to_part(
            ROW_ACTIONS,
            PartRef::of(Part::CONTAINER),
            Side::Below,
            CrossAlign::Start,
        )
}

fn connection_primary(kind: usize) -> Button<'static> {
    Button::new(
        CONNECTION_PRIMARY,
        if kind == 1 { "Retry" } else { "New connection" },
    )
    .variant(Variant::PRIMARY)
}

fn orders_grid() -> Grid<'static> {
    Grid::new(ORDERS_GRID, &ORDER_COLUMNS).nav(NavUnit::Cell)
}

fn menu_items(reason: &'static str) -> [MenuItem<'static>; 5] {
    [
        MenuItem::new(EDIT_CELL, "Edit cell")
            .hint(reason)
            .reason(reason),
        MenuItem::new(DUPLICATE_ROW, "Duplicate row")
            .hint(reason)
            .reason(reason),
        MenuItem::new(DELETE_ROW, "Delete row")
            .danger()
            .hint(reason)
            .reason(reason)
            .separator(),
        MenuItem::new(COPY_ROW, "Copy row"),
        MenuItem::new(COPY_CSV, "Copy as CSV"),
    ]
}

fn specimen_id(base: Id, row: usize, col: usize) -> Id {
    base.item(ItemKey::index(row.saturating_mul(3).saturating_add(col)))
}

fn specimen_state(row: usize, col: usize) -> (EmptyState<'static>, &'static str) {
    let base_title = if col == 2 {
        "No connections"
    } else {
        "No rows"
    };
    let title = match row {
        5 if col == 2 => "No connections in a view far too narrow for it",
        5 => "No rows in a view far too narrow for it",
        _ => base_title,
    };
    let detail = (col == 1).then_some("Run the query to fill it");
    let state = if row == 4 {
        EmptyState::Error {
            message: title,
            detail,
        }
    } else {
        EmptyState::Empty {
            title,
            hint: detail,
        }
    };
    (state, "New")
}
