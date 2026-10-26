use termrock::{
    Cx, DiffLineKind, DiffMode, DiffRow, DiffSource, DiffView, DiffViewState, FrameRead, Id,
    ItemKey, List, ListAction, ListState, Part, PartRef, Rect, Response, Role, ScrollRegion,
    ScrollState, Select, SelectMode, SelectState, Span, StateFlags, Tabs, TabsState, Toggle, Ui,
    author::VerticalWindow,
};

use super::{Page, PageStatus, PageUpdate, card_panel, frame};

const PAGE_TABS: Id = Id::root("showcase_ui::pages::diff.page_tabs");
const OPTIONS_SCROLL: Id = Id::root("showcase_ui::pages::diff.options_scroll");
const STATES_SCROLL: Id = Id::root("showcase_ui::pages::diff.states_scroll");
const FILE_LIST: Id = Id::root("showcase_ui::pages::diff.files");
const LIVE_VIEW: Id = Id::root("showcase_ui::pages::diff.view");
const REVIEW: Id = Id::root("showcase_ui::pages::diff.review");
const EMPTY: Id = Id::root("showcase_ui::pages::diff.empty");
const MODE: Id = Id::root("showcase_ui::pages::diff.option.mode");
const FILES: Id = Id::root("showcase_ui::pages::diff.option.files");
const WIDTH: Id = Id::root("showcase_ui::pages::diff.option.width");
const FILES_PANEL: Id = Id::root("showcase_ui::pages::diff.files_panel");
const DIFF_PANEL: Id = Id::root("showcase_ui::pages::diff.diff_panel");
const STATE_ROOT: Id = Id::root("showcase_ui::pages::diff.states");
const HORIZONTAL_TRACK: Part = Part::custom("viewport.horizontal-track");

const PAGE_TABS_ITEMS: &[&str] = &["Playground", "Options", "States"];
const MODE_ITEMS: &[&str] = &["Unified", "Review"];
const FILE_OPTION_ITEMS: &[&str] = &["Changes", "None"];
const WIDTH_ITEMS: &[&str] = &["Full", "Narrow"];

const LIST_WIDTH: u16 = 38;
const COLUMN_GAP: u16 = 2;
const NARROW_WIDTH: u16 = 48;
const PLAY_CONTROL_ROWS: u16 = 2;
const STATE_ROWS: usize = 4;
const STATE_ROW_HEIGHT: usize = 6;
const STATE_LABEL_WIDTH: u16 = 9;

const TAB_HINTS: &[(&str, &str)] = &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
const TOGGLE_HINTS: &[(&str, &str)] = &[("Space", "Toggle"), ("Enter", "Toggle")];
const LIST_HINTS: &[(&str, &str)] = &[("↑ ↓", "View file"), ("Wheel", "Scroll")];
const SELECT_HINTS: &[(&str, &str)] = &[("↑ ↓", "Choose"), ("Enter", "Open / choose")];
const SCROLL_HINTS: &[(&str, &str)] = &[("Wheel", "Scroll")];
const DIFF_START_HINTS: &[(&str, &str)] = &[
    ("Drag", "Select"),
    ("Shift+← →", "Start / extend (default)"),
    ("↑ ↓", "Scroll"),
    ("End", "Tail"),
    ("f", "Follow"),
];
const DIFF_START_PAN_HINTS: &[(&str, &str)] = &[
    ("Drag", "Select"),
    ("Shift+← →", "Start / extend (default)"),
    ("↑ ↓", "Scroll"),
    ("← →", "Pan"),
    ("End", "Tail"),
    ("f", "Follow"),
];
const DIFF_SELECTION_HINTS: &[(&str, &str)] =
    &[("Shift+← →", "Extend"), ("y", "Capture"), ("Esc", "Clear")];
const DIFF_SELECTION_PAN_HINTS: &[(&str, &str)] = &[
    ("Shift+← →", "Extend"),
    ("y", "Capture"),
    ("Esc", "Clear"),
    ("← →", "Pan"),
];

const fn line(kind: DiffLineKind, text: &'static str) -> DiffRow<'static> {
    DiffRow::Line { kind, text }
}

const fn hunk(old_start: usize, new_start: usize) -> DiffRow<'static> {
    DiffRow::Hunk {
        old_start,
        new_start,
    }
}

const fn context(text: &'static str) -> DiffRow<'static> {
    line(DiffLineKind::Context, text)
}

const fn add(text: &'static str) -> DiffRow<'static> {
    line(DiffLineKind::Add, text)
}

const fn remove(text: &'static str) -> DiffRow<'static> {
    line(DiffLineKind::Remove, text)
}

const CONFIG_ROWS: &[DiffRow<'static>] = &[
    hunk(1, 1),
    context("[service.worker_0]"),
    remove("attempts = 3"),
    add("attempts = 5"),
    add("backoff = \"exponential\""),
    context("region = \"東京\""),
    context("label = \"cafe\u{301} ☕\""),
    context("endpoint = \"https://api.example.test/workers/health\""),
    context("enabled = true"),
    hunk(9, 10),
    context("[service.worker_1]"),
    remove("attempts = 3"),
    add("attempts = 5"),
    add("backoff = \"exponential\""),
    context("region = \"東京\""),
    context("label = \"cafe\u{301} ☕\""),
    context("endpoint = \"https://api.example.test/workers/health\""),
    context("enabled = true"),
    hunk(17, 19),
    context("[service.worker_2]"),
    remove("attempts = 3"),
    add("attempts = 5"),
    add("backoff = \"exponential\""),
    context("region = \"東京\""),
    context("label = \"cafe\u{301} ☕\""),
    context("endpoint = \"https://api.example.test/workers/health\""),
    context("enabled = true"),
    hunk(25, 28),
    context("[service.worker_3]"),
    remove("attempts = 3"),
    add("attempts = 5"),
    add("backoff = \"exponential\""),
    context("region = \"東京\""),
    context("label = \"cafe\u{301} ☕\""),
    context("endpoint = \"https://api.example.test/workers/health\""),
    context("enabled = true"),
    hunk(33, 37),
    context("[service.worker_4]"),
    remove("attempts = 3"),
    add("attempts = 5"),
    add("backoff = \"exponential\""),
    context("region = \"東京\""),
    context("label = \"cafe\u{301} ☕\""),
    context("endpoint = \"https://api.example.test/workers/health\""),
    context("enabled = true"),
];

const ADDED_ROWS: &[DiffRow<'static>] = &[
    hunk(0, 1),
    add("# Retries"),
    add(""),
    add("Workers retry five times with exponential backoff."),
];

const DELETED_ROWS: &[DiffRow<'static>] = &[
    hunk(1, 0),
    remove("pub fn linear(attempt: u32) -> u64 {"),
    remove("    200 * attempt as u64"),
    remove("}"),
];

const RENAMED_ROWS: &[DiffRow<'static>] = &[
    hunk(1, 1),
    remove("use crate::legacy_backoff::linear;"),
    add("use super::exponential;"),
    context("pub async fn fetch(url: &str) -> Result<Body, Error> {"),
];

#[derive(Clone, Copy, Debug)]
struct Source {
    revision: u64,
    path: &'static str,
    marker: &'static str,
    status_role: Role,
    status: &'static str,
    rename_from: Option<&'static str>,
    rows: &'static [DiffRow<'static>],
    meta: &'static str,
}

impl DiffSource for Source {
    fn revision(&self) -> u64 {
        self.revision
    }

    fn path(&self) -> &str {
        self.path
    }

    fn status_marker(&self) -> &str {
        self.marker
    }

    fn status_role(&self) -> Role {
        self.status_role
    }

    fn status_label(&self) -> &str {
        self.status
    }

    fn renamed_from(&self) -> Option<&str> {
        self.rename_from
    }

    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row(&self, index: usize) -> Option<DiffRow<'_>> {
        self.rows.get(index).copied()
    }
}

static CHANGES: [Source; 4] = [
    Source {
        revision: 1,
        path: "config/service.toml",
        marker: "M",
        status_role: Role::Warning,
        status: "modified",
        rename_from: None,
        rows: CONFIG_ROWS,
        meta: "+10 −5",
    },
    Source {
        revision: 1,
        path: "docs/retries.md",
        marker: "A",
        status_role: Role::Success,
        status: "added",
        rename_from: None,
        rows: ADDED_ROWS,
        meta: "+3 −0",
    },
    Source {
        revision: 1,
        path: "src/legacy_backoff.rs",
        marker: "D",
        status_role: Role::Danger,
        status: "deleted",
        rename_from: None,
        rows: DELETED_ROWS,
        meta: "+0 −3",
    },
    Source {
        revision: 1,
        path: "src/backoff/retry.rs",
        marker: "R",
        status_role: Role::Fg(termrock::FgStep::Secondary),
        status: "renamed",
        rename_from: Some("src/retry.rs"),
        rows: RENAMED_ROWS,
        meta: "+1 −1",
    },
];

const STATE_ROWS_SOURCE: Source = Source {
    revision: 1,
    path: "retry.toml",
    marker: "M",
    status_role: Role::Warning,
    status: "modified",
    rename_from: None,
    rows: &[
        hunk(1, 1),
        remove("attempts = 3"),
        add("attempts = 5"),
        context("enabled = true"),
    ],
    meta: "+1 −1",
};

fn file_row(source: &Source, row: &mut termrock::RowUi<'_>) {
    row.meta(source.meta);
    let mut spans = Vec::with_capacity(5);
    spans.push(Span::new(source.marker).role(source.status_role).bold());
    spans.push(Span::new(" "));
    if let Some(rename_from) = source.rename_from {
        spans.push(Span::new(rename_from));
        spans.push(Span::new(" → "));
    }
    spans.push(Span::new(source.path));
    row.label_spans(&spans);
}

fn files_list()
-> List<'static, Source, termrock::ByIndex, impl Fn(&Source, &mut termrock::RowUi<'_>)> {
    List::new(FILE_LIST)
        .row(file_row)
        .select_mode(SelectMode::Single)
}

#[derive(Debug)]
pub struct DiffPage {
    tabs: TabsState,
    view_state: DiffViewState,
    files: ListState,
    mode_option: SelectState,
    file_option_state: SelectState,
    width_option_state: SelectState,
    file_option: usize,
    width_option: usize,
    empty: bool,
    states_scroll: ScrollState,
    options_scroll: ScrollState,
    copied: Option<String>,
}

impl DiffPage {
    pub fn new() -> Self {
        Self {
            tabs: TabsState::default(),
            view_state: DiffViewState::default(),
            files: ListState::default(),
            mode_option: SelectState::default(),
            file_option_state: SelectState::default(),
            width_option_state: SelectState::default(),
            file_option: 0,
            width_option: 0,
            empty: false,
            states_scroll: ScrollState::default(),
            options_scroll: ScrollState::default(),
            copied: None,
        }
    }

    fn section(&self) -> usize {
        match self.tabs.active().or(self.tabs.cursor()) {
            Some(ItemKey::Index(index)) => index,
            _ => 0,
        }
    }

    fn selected_file(&self) -> usize {
        match self.files.cursor() {
            Some(ItemKey::Index(index)) => index.min(CHANGES.len().saturating_sub(1)),
            _ => 0,
        }
    }

    fn source(&self) -> Option<&'static dyn DiffSource> {
        if self.empty || self.file_option == 1 {
            None
        } else {
            CHANGES
                .get(self.selected_file())
                .map(|source| source as &dyn DiffSource)
        }
    }

    fn live_view(&self) -> DiffView<'static> {
        DiffView::new(LIVE_VIEW, self.source())
            .horizontal_pan(true)
            .responsive_review(true)
    }

    fn set_mode(&mut self, index: usize) -> bool {
        let mode = if index == 1 {
            DiffMode::Review
        } else {
            DiffMode::Unified
        };
        let changed = self.view_state.mode() != mode;
        self.view_state.set_mode(mode);
        changed
    }

    fn clear_feedback(&mut self) {
        let _ = self.view_state.viewport_mut().clear_selection();
        self.copied = None;
    }

    fn update_playground(&mut self, cx: &mut Cx<'_>) -> (Response<()>, bool, Option<PageStatus>) {
        let mut response = Response::ignored();
        let mut controls_changed = false;
        let mut status = None;

        let mut review = self.view_state.mode() == DiffMode::Review;
        let review_response = Toggle::new(REVIEW, "Review")
            .on(review)
            .update(cx, &mut review);
        if review_response.activated() {
            controls_changed |= self.set_mode(usize::from(review));
        }
        response |= review_response.erase();

        let empty_response = Toggle::new(EMPTY, "Empty")
            .on(self.empty)
            .update(cx, &mut self.empty);
        if empty_response.activated() {
            controls_changed = true;
        }
        response |= empty_response.erase();

        let old_file = self.selected_file();
        let list_response = files_list().update(cx, &mut self.files, &CHANGES);
        let list_action = list_response.action_ref().copied();
        if matches!(
            list_action,
            Some(ListAction::Moved | ListAction::Chose(_) | ListAction::Activated(_))
        ) && old_file != self.selected_file()
        {
            controls_changed = true;
        }
        response |= list_response.erase();

        if controls_changed {
            self.clear_feedback();
        }

        let view_response = self.live_view().update(cx, &mut self.view_state);
        if let Some(termrock::ViewportAction::Copy(text)) = view_response.action_ref() {
            let text = text.clone();
            self.copied = Some(text.clone());
            status = Some(PageStatus(format!("Captured selection: {text}")));
        }
        response |= view_response.erase();
        (response, controls_changed, status)
    }

    fn update_options(&mut self, cx: &mut Cx<'_>) -> (Response<()>, bool) {
        let mut response = Response::ignored();
        let mut controls_changed = false;

        let old_mode = usize::from(self.view_state.mode() == DiffMode::Review);
        let mut mode = old_mode;
        let mode_response = Select::new(MODE).popup_rows(2).update_in_form(
            cx,
            &mut self.mode_option,
            &mut mode,
            MODE_ITEMS,
            false,
        );
        if mode != old_mode {
            controls_changed |= self.set_mode(mode);
        }
        response |= mode_response.erase();

        let old_file_option = self.file_option;
        let mut file_option = old_file_option;
        let file_response = Select::new(FILES).popup_rows(2).update_in_form(
            cx,
            &mut self.file_option_state,
            &mut file_option,
            FILE_OPTION_ITEMS,
            false,
        );
        if file_option != old_file_option {
            self.file_option = file_option;
            controls_changed = true;
        }
        response |= file_response.erase();

        let old_width = self.width_option;
        let mut width_option = old_width;
        let width_response = Select::new(WIDTH).popup_rows(2).update_in_form(
            cx,
            &mut self.width_option_state,
            &mut width_option,
            WIDTH_ITEMS,
            false,
        );
        if width_option != old_width {
            self.width_option = width_option;
            controls_changed = true;
        }
        response |= width_response.erase();

        if controls_changed {
            self.clear_feedback();
        }
        (response, controls_changed)
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let review = self.view_state.mode() == DiffMode::Review;
        let review_area = Rect::new(area.x, area.y, area.width.min(14), 1);
        let empty_x = area.x.saturating_add(15);
        let empty_area = Rect::new(
            empty_x,
            area.y,
            area.right().saturating_sub(empty_x).min(14),
            1,
        );
        if !review_area.is_empty() {
            Toggle::new(REVIEW, "Review")
                .on(review)
                .draw(ui, review_area);
        }
        if !empty_area.is_empty() {
            Toggle::new(EMPTY, "Empty")
                .on(self.empty)
                .draw(ui, empty_area);
        }

        let status_y = area.y.saturating_add(1);
        let status = Rect::new(area.x, status_y, area.width, 1);
        let status_style = ui.surface_style().fg(ui.theme_ref().color.fg[2]);
        let status_text = if self.copied.is_some() {
            "Selection captured"
        } else {
            "Four changes · select a file, then capture a selection with y"
        };
        ui.paint_str(status, status_text, status_style);

        let work_y = area.y.saturating_add(PLAY_CONTROL_ROWS);
        let mut work = Rect {
            y: work_y,
            height: area.bottom().saturating_sub(work_y),
            ..area
        };
        if self.width_option == 1 {
            work.width = work.width.min(NARROW_WIDTH);
        }
        if work.is_empty() {
            return;
        }

        if work.width >= 60 {
            let list_area = Rect {
                width: work.width.min(LIST_WIDTH),
                ..work
            };
            let diff_x = list_area.right().saturating_add(COLUMN_GAP);
            let diff_area = Rect {
                x: diff_x,
                width: work.right().saturating_sub(diff_x),
                ..work
            };
            self.draw_file_list(ui, list_area);
            self.draw_live_diff(ui, diff_area);
        } else {
            let list_height = work.height / 2;
            let list_area = Rect {
                height: list_height,
                ..work
            };
            let diff_y = work.y.saturating_add(list_height);
            let diff_area = Rect {
                y: diff_y,
                height: work.bottom().saturating_sub(diff_y),
                ..work
            };
            self.draw_file_list(ui, list_area);
            self.draw_live_diff(ui, diff_area);
        }
    }

    fn draw_file_list(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        card_panel(
            ui,
            FILES_PANEL,
            "Changes",
            Some("4 files"),
            area,
            |ui, inner| {
                let _ = files_list().draw(ui, inner, &self.files, &CHANGES);
            },
        );
    }

    fn draw_live_diff(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        card_panel(ui, DIFF_PANEL, "Diff", None, area, |ui, inner| {
            let _ = self.live_view().draw(ui, inner, &self.view_state);
        });
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let view = ScrollRegion::new(OPTIONS_SCROLL).draw(
            ui,
            area,
            &self.options_scroll,
            OPTIONS_CONTENT_ROWS,
        );
        let Some(window) = scroll_window(view, OPTIONS_CONTENT_ROWS, &self.options_scroll) else {
            return;
        };
        ui.with_area(view, |ui| {
            for (row, (label, id)) in [
                ("Mode · diff.mode", MODE),
                ("File · diff.file", FILES),
                ("Width · diff.width", WIDTH),
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(y) = window.physical_row_for_source(row.saturating_mul(2)) {
                    let label_area = Rect::new(view.x, y, view.width, 1);
                    ui.paint_str(label_area, label, ui.surface_style());
                }
                let Some(y) =
                    window.physical_row_for_source(row.saturating_mul(2).saturating_add(1))
                else {
                    continue;
                };
                let input_area = Rect::new(view.x, y, view.width, 1);
                match id {
                    MODE => {
                        let _ =
                            Select::new(MODE).draw(ui, input_area, &self.mode_option, MODE_ITEMS);
                    }
                    FILES => {
                        let _ = Select::new(FILES).draw(
                            ui,
                            input_area,
                            &self.file_option_state,
                            FILE_OPTION_ITEMS,
                        );
                    }
                    WIDTH => {
                        let _ = Select::new(WIDTH).draw(
                            ui,
                            input_area,
                            &self.width_option_state,
                            WIDTH_ITEMS,
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
        let two_columns = area.width >= 60;
        let content_rows = state_content_rows(two_columns);
        let view =
            ScrollRegion::new(STATES_SCROLL).draw(ui, area, &self.states_scroll, content_rows);
        let Some(window) = scroll_window(view, content_rows, &self.states_scroll) else {
            return;
        };
        ui.with_area(view, |ui| {
            if two_columns {
                self.draw_state_matrix(ui, window);
            } else {
                self.draw_state_stacks(ui, window);
            }
        });
    }

    fn draw_state_matrix(&self, ui: &mut Ui<'_>, window: VerticalWindow) {
        let area = window.visible_body();
        if let Some(y) = window.physical_row_for_source(0) {
            ui.paint_str(
                Rect::new(area.x, y, STATE_LABEL_WIDTH, 1),
                "State",
                ui.surface_style(),
            );
            let columns = state_columns(area);
            ui.paint_str(
                Rect::new(columns[0].x, y, columns[0].width, 1),
                "Unified",
                ui.surface_style(),
            );
            ui.paint_str(
                Rect::new(columns[1].x, y, columns[1].width, 1),
                "Review",
                ui.surface_style(),
            );
        }
        let visible = window.visible_source_range();
        for state_index in 0..STATE_ROWS {
            let source_start = 1usize.saturating_add(state_index.saturating_mul(STATE_ROW_HEIGHT));
            let source_end = source_start.saturating_add(STATE_ROW_HEIGHT);
            if source_start < visible.start || source_end > visible.end {
                continue;
            }
            let Some(y) = window.physical_row_for_source(source_start) else {
                continue;
            };
            let columns = state_columns(area);
            draw_state_label(ui, Rect::new(area.x, y, STATE_LABEL_WIDTH, 1), state_index);
            for (mode_index, mode) in [DiffMode::Unified, DiffMode::Review]
                .into_iter()
                .enumerate()
            {
                let diff_area = Rect {
                    y: y.saturating_add(1),
                    height: state_specimen_diff_height(mode, state_index),
                    ..columns[mode_index]
                };
                self.draw_specimen(ui, diff_area, mode, state_index, mode_index);
            }
        }
    }

    fn draw_state_stacks(&self, ui: &mut Ui<'_>, window: VerticalWindow) {
        let visible = window.visible_source_range();
        for (mode_index, mode) in [DiffMode::Unified, DiffMode::Review]
            .into_iter()
            .enumerate()
        {
            let group_start = if mode_index == 0 {
                0
            } else {
                state_mode_rows(DiffMode::Unified)
            };
            if let Some(y) = window.physical_row_for_source(group_start) {
                ui.paint_str(
                    Rect::new(window.visible_body().x, y, window.visible_body().width, 1),
                    mode.label(),
                    ui.surface_style(),
                );
            }
            for state_index in 0..STATE_ROWS {
                let source_start = state_specimen_source_start(group_start, mode, state_index);
                let diff_height = state_specimen_diff_height(mode, state_index);
                let source_end = source_start
                    .saturating_add(1)
                    .saturating_add(usize::from(diff_height));
                if source_start < visible.start || source_end > visible.end {
                    continue;
                }
                let Some(y) = window.physical_row_for_source(source_start) else {
                    continue;
                };
                let body = window.visible_body();
                draw_state_label(
                    ui,
                    Rect::new(body.x, y, body.width.min(STATE_LABEL_WIDTH), 1),
                    state_index,
                );
                let diff_area = Rect {
                    y: y.saturating_add(1),
                    height: diff_height,
                    ..Rect::new(body.x, y, body.width, diff_height.saturating_add(1))
                };
                self.draw_specimen(ui, diff_area, mode, state_index, mode_index);
            }
        }
    }

    fn draw_specimen(
        &self,
        ui: &mut Ui<'_>,
        mut area: Rect,
        mode: DiffMode,
        state_index: usize,
        mode_index: usize,
    ) {
        if area.is_empty() {
            return;
        }
        if state_index == 3 {
            area.width = area.width.min(42);
        }
        if area.is_empty() {
            return;
        }
        let source = (state_index != 2).then_some(&STATE_ROWS_SOURCE as &dyn DiffSource);
        let id = specimen_id(state_index, mode_index);
        let view = DiffView::new(id, source).responsive_review(true);
        let mut state = DiffViewState::default();
        state.set_mode(mode);
        let target = (state_index == 1)
            .then(|| termrock::ReferenceTarget::new(id, termrock::ReferenceState::FOCUSED));
        ui.reference(target, |ui| {
            let _ = view.draw(ui, area, &state);
        });
    }
}

impl Default for DiffPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for DiffPage {
    fn title(&self) -> &'static str {
        "Diff viewer"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Tabs::new(PAGE_TABS)
            .update(cx, &mut self.tabs, PAGE_TABS_ITEMS)
            .erase();
        let mut status = None;
        match self.section() {
            0 => {
                let (playground, _, copy_status) = self.update_playground(cx);
                response |= playground;
                status = copy_status;
            }
            1 => {
                let (options, _) = self.update_options(cx);
                response |= options;
                response |= ScrollRegion::new(OPTIONS_SCROLL).update(
                    cx,
                    &mut self.options_scroll,
                    OPTIONS_CONTENT_ROWS,
                );
            }
            _ => {
                let content_rows = cx
                    .layout(STATES_SCROLL)
                    .map_or_else(|| state_content_rows(true), |layout| layout.content_len);
                response |= ScrollRegion::new(STATES_SCROLL).update(
                    cx,
                    &mut self.states_scroll,
                    content_rows,
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
                "real shared DiffView · captured selection only",
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
                    let layout = tabs.layout_default(
                        tabs_area,
                        ui.full(),
                        &self.tabs,
                        PAGE_TABS_ITEMS,
                        previous,
                    );
                    if tabs.draw_layout(ui, &layout).is_err() {
                        let _ = tabs.draw(ui, tabs_area, &self.tabs, PAGE_TABS_ITEMS);
                    }

                    let content_offset = if self.section() == 2 { 2 } else { 3 };
                    let content = Rect {
                        y: body.y.saturating_add(content_offset),
                        height: body.height.saturating_sub(content_offset),
                        ..body
                    };
                    if content.is_empty() {
                        return;
                    }
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
        if ui.state(REVIEW).contains(StateFlags::FOCUSED)
            || ui.state(EMPTY).contains(StateFlags::FOCUSED)
        {
            return TOGGLE_HINTS;
        }
        if ui.state(FILE_LIST).contains(StateFlags::FOCUSED) {
            return LIST_HINTS;
        }
        if [MODE, FILES, WIDTH]
            .iter()
            .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            return SELECT_HINTS;
        }
        if ui.state(STATES_SCROLL).contains(StateFlags::FOCUSED)
            || ui.state(OPTIONS_SCROLL).contains(StateFlags::FOCUSED)
        {
            return SCROLL_HINTS;
        }
        if ui.state(LIVE_VIEW).contains(StateFlags::FOCUSED) {
            let has_pan = ui
                .area_of_part(LIVE_VIEW, PartRef::of(HORIZONTAL_TRACK))
                .is_some();
            if self.view_state.viewport().selection().is_some() {
                return if has_pan {
                    DIFF_SELECTION_PAN_HINTS
                } else {
                    DIFF_SELECTION_HINTS
                };
            }
            return if has_pan {
                DIFF_START_PAN_HINTS
            } else {
                DIFF_START_HINTS
            };
        }
        match self.section() {
            0 => LIST_HINTS,
            1 => SELECT_HINTS,
            _ => SCROLL_HINTS,
        }
    }
}

const OPTIONS_CONTENT_ROWS: usize = 6;

fn state_content_rows(two_columns: bool) -> usize {
    if two_columns {
        1usize.saturating_add(STATE_ROWS.saturating_mul(STATE_ROW_HEIGHT))
    } else {
        state_mode_rows(DiffMode::Unified).saturating_add(state_mode_rows(DiffMode::Review))
    }
}

fn state_mode_rows(mode: DiffMode) -> usize {
    (0..STATE_ROWS).fold(1usize, |rows, state_index| {
        rows.saturating_add(
            1usize.saturating_add(usize::from(state_specimen_diff_height(mode, state_index))),
        )
    })
}

fn state_specimen_source_start(group_start: usize, mode: DiffMode, state_index: usize) -> usize {
    (0..state_index).fold(group_start.saturating_add(1), |start, previous| {
        start.saturating_add(
            1usize.saturating_add(usize::from(state_specimen_diff_height(mode, previous))),
        )
    })
}

fn state_specimen_diff_height(mode: DiffMode, _state_index: usize) -> u16 {
    match mode {
        DiffMode::Unified => 5,

        DiffMode::Review => 5,
    }
}

fn specimen_id(state_index: usize, mode_index: usize) -> Id {
    STATE_ROOT.index(state_index.saturating_mul(2).saturating_add(mode_index))
}

fn state_columns(area: Rect) -> [Rect; 2] {
    let content_x = area.x.saturating_add(STATE_LABEL_WIDTH.saturating_add(1));
    let content_width = area.right().saturating_sub(content_x);
    let left_width = content_width.saturating_sub(2) / 2;
    let right_x = content_x.saturating_add(left_width).saturating_add(2);
    [
        Rect::new(content_x, area.y, left_width, area.height),
        Rect::new(
            right_x,
            area.y,
            area.right().saturating_sub(right_x),
            area.height,
        ),
    ]
}

fn draw_state_label(ui: &mut Ui<'_>, area: Rect, index: usize) {
    let label = ["Rest", "Focused", "Empty", "Narrow"][index.min(3)];
    ui.paint_str(
        area,
        label,
        ui.surface_style().fg(ui.theme_ref().color.fg[1]),
    );
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
