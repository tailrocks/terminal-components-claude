use std::cell::Cell;
use termrock::fields::TextEditCommand;

use termrock::{
    BlurPolicy, Constraints, Cx, EditEntry, Family, Field, FieldError, FocusSelect, FrameRead, Id,
    ItemKey, Panel, PanelKind, Part, Rect, ReferenceState, ReferenceTarget, Response, ScrollRegion,
    ScrollState, Select, SelectState, StateFlags, Tabs, TabsState, TextAction, TextArea,
    TextAreaState, Track, Ui, Variant, author::VerticalWindow, layout, truncate,
};

use super::{Page, PageUpdate, frame};

const BODY: Id = termrock::Id::root("showcase_ui::pages::textareas::textareas.body");
const NOTES: Id = termrock::Id::root("showcase_ui::pages::textareas::textareas.notes");
const TRANSCRIPT: Id = termrock::Id::root("showcase_ui::pages::textareas::textareas.transcript");
const COMMIT: Id = termrock::Id::root("showcase_ui::pages::textareas::textareas.commit");
const PAGE_TABS: Id = termrock::Id::root("showcase_ui::pages::textareas::page_tabs");
const BLUR_ACTION: Id = termrock::Id::root("showcase_ui::pages::textareas::option.blur_action");
const EDIT_ENTRY: Id = termrock::Id::root("showcase_ui::pages::textareas::option.edit_entry");
const FOCUS_SELECT: Id = termrock::Id::root("showcase_ui::pages::textareas::option.focus_select");
const ESCAPE_BEHAVIOR: Id =
    termrock::Id::root("showcase_ui::pages::textareas::option.escape_behavior");
const STATES_SCROLL: Id = termrock::Id::root("showcase_ui::pages::textareas::states_scroll");
const PLAYGROUND: Id = termrock::Id::root("showcase_ui::pages::textareas::textareas.playground");
const OPTIONS: Id = termrock::Id::root("showcase_ui::pages::textareas::options");
const STATES: Id = termrock::Id::root("showcase_ui::pages::textareas::textareas.states");
const STATE_REFERENCES: Id = termrock::Id::root("showcase_ui::pages::textareas::states.references");
const EDITING_REFERENCE: Id = STATE_REFERENCES.index(3);

const PAGE_TABS_ITEMS: &[&str] = &["Playground", "Options", "States"];
const BLUR_ACTION_ITEMS: &[&str] = &["Commit", "Cancel", "Keep"];
const EDIT_ENTRY_ITEMS: &[&str] = &["Immediate", "Type to edit", "Explicit"];
const FOCUS_SELECT_ITEMS: &[&str] = &["Caret end", "Select all", "Caret start"];
const ESCAPE_BEHAVIOR_ITEMS: &[&str] = &["Commit", "Revert"];
const PAGE_TITLE_TABS_X: u16 = 12;
const STATE_SAMPLE_COUNT: usize = 7;
const STATE_SAMPLE_STRIDE: usize = 5;
const EXISTING_STATES_HEIGHT: usize = 10;
const LONG_UNICODE_LINE: &str = " 4. Run the integration suite and attach the report. Review 日本語 · 👩‍💻 · cafe\u{301} through this long Unicode line.";

#[derive(Clone, Copy, Debug, Default)]
struct HintDrawFacts {
    body: Option<Rect>,
    blur_action: Option<Rect>,
    edit_entry: Option<Rect>,
    focus_select: Option<Rect>,
    escape_behavior: Option<Rect>,
    states: Option<Rect>,
    tabs: Option<Rect>,
}

fn body_area(
    blur: BlurPolicy,
    edit_entry: EditEntry,
    focus_select: FocusSelect,
    esc_reverts: bool,
) -> TextArea<'static> {
    TextArea::new(BODY, 8)
        .placeholder("Write a checklist")
        .blur(blur)
        .edit_entry(edit_entry)
        .focus_select(focus_select)
        .esc_reverts(esc_reverts)
}

fn checklist() -> String {
    (1..=28)
        .map(|line| match line % 4 {
            0 => format!("{line:>2}. Run the integration suite and attach the report."),
            1 => format!("{line:>2}. Read src/api/billing.rs before touching invoices."),
            2 => format!("{line:>2}. Keep the public API stable; add, never rename."),
            _ => format!("{line:>2}. Open a PR against main with a clear summary."),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn task_field(
    value: &str,
    blur: BlurPolicy,
    edit_entry: EditEntry,
    focus_select: FocusSelect,
    esc_reverts: bool,
) -> Field<'_, TextArea<'_>> {
    Field::new(
        "Task description",
        body_area(blur, edit_entry, focus_select, esc_reverts).value(value),
    )
    .optional_suffix(false)
}

fn notes_field() -> Field<'static, TextArea<'static>> {
    Field::new(
        "Notes",
        TextArea::new(NOTES, 8).placeholder("Anything the agent should know…"),
    )
    .optional_suffix(false)
    .help("Optional")
}

fn transcript_field() -> Field<'static, TextArea<'static>> {
    Field::new(
        "Read-only transcript",
        TextArea::new(TRANSCRIPT, 4)
            .value("Junie: Reading 14 files…\nJunie: Plan ready. 3 steps.")
            .disabled(true),
    )
    .optional_suffix(false)
}

fn commit_field() -> Field<'static, TextArea<'static>> {
    Field::new(
        "Commit message",
        TextArea::new(COMMIT, 4).value("fix stuff"),
    )
    .optional_suffix(false)
    .error(Some("Use the imperative mood and explain why"))
}

fn playground_panel(meta: &'static str) -> Panel<'static> {
    Panel::new(PLAYGROUND)
        .kind(PanelKind::Card)
        .title("Playground")
        .meta(meta)
}

fn states_panel() -> Panel<'static> {
    Panel::new(STATES)
        .kind(PanelKind::Card)
        .title("Disabled and error")
}

fn blur_policy_index(policy: BlurPolicy) -> usize {
    match policy {
        BlurPolicy::Commit | BlurPolicy::CommitAndValidate => 0,
        BlurPolicy::Cancel => 1,
        BlurPolicy::Keep => 2,
    }
}

fn blur_policy_at(index: usize) -> BlurPolicy {
    match index.min(BLUR_ACTION_ITEMS.len().saturating_sub(1)) {
        1 => BlurPolicy::Cancel,
        2 => BlurPolicy::Keep,
        _ => BlurPolicy::Commit,
    }
}

fn edit_entry_index(policy: EditEntry) -> usize {
    match policy {
        EditEntry::Immediate => 0,
        EditEntry::TypeToEdit => 1,
        EditEntry::Explicit => 2,
    }
}

fn edit_entry_at(index: usize) -> EditEntry {
    match index.min(EDIT_ENTRY_ITEMS.len().saturating_sub(1)) {
        1 => EditEntry::TypeToEdit,
        2 => EditEntry::Explicit,
        _ => EditEntry::Immediate,
    }
}

fn focus_select_index(policy: FocusSelect) -> usize {
    match policy {
        FocusSelect::CaretEnd => 0,
        FocusSelect::SelectAll => 1,
        FocusSelect::CaretStart => 2,
    }
}

fn focus_select_at(index: usize) -> FocusSelect {
    match index.min(FOCUS_SELECT_ITEMS.len().saturating_sub(1)) {
        1 => FocusSelect::SelectAll,
        2 => FocusSelect::CaretStart,
        _ => FocusSelect::CaretEnd,
    }
}

fn escape_behavior_index(esc_reverts: bool) -> usize {
    usize::from(esc_reverts)
}

fn escape_behavior_at(index: usize) -> bool {
    index.min(ESCAPE_BEHAVIOR_ITEMS.len().saturating_sub(1)) == 1
}

fn lower_option_area(area: Rect) -> Rect {
    Rect {
        y: area.y.saturating_add(4),
        height: area.height.saturating_sub(4),
        ..area
    }
}

fn draw_option(
    ui: &mut Ui<'_>,
    area: Rect,
    label: &str,
    id: Id,
    state: &SelectState,
    items: &[&str],
    popup_rows: u16,
) -> Option<Rect> {
    if area.is_empty() {
        return None;
    }
    let rows = layout::rows(area, &[Track::Fixed(1), Track::Fixed(3)]);
    let label_area = rows.first().copied().unwrap_or(area);
    let field = rows.get(1).copied().unwrap_or(area);
    let _ = ui.paint_str(label_area, label, ui.surface_style());
    let drawn = Select::new(id)
        .popup_rows(popup_rows)
        .draw(ui, field, state, items);
    let visible = drawn.intersection(ui.full());
    if ui.is_inert() {
        return None;
    }
    (!visible.is_empty()).then_some(visible)
}

fn visible_textarea_control(ui: &Ui<'_>, area: Rect, control: &TextArea<'_>) -> Option<Rect> {
    if ui.is_inert() || area.is_empty() || area.height < 2 {
        return None;
    }
    let height = control
        .measure(
            ui,
            Constraints::loose(area.width, area.height.saturating_sub(1)),
        )
        .preferred
        .1
        .max(1)
        .min(area.height.saturating_sub(1));
    let control_area = Rect {
        x: area.x,
        y: area.y.saturating_add(1),
        width: area.width,
        height,
    };
    let visible = control_area.intersection(ui.full());
    (!visible.is_empty()).then_some(visible)
}

fn playground_meta(edit_entry: EditEntry, narrow: bool) -> &'static str {
    match edit_entry {
        EditEntry::Immediate if narrow => "Enter Edit · Esc Done · Tab Next",
        EditEntry::Immediate => "Enter Edit · Esc Done · Tab Next ",
        EditEntry::TypeToEdit => "Type / Paste Edit · Esc Done · Tab Next",
        EditEntry::Explicit => "Enter Begin · Esc Done · Tab Next",
    }
}

fn state_content_rows(two_columns: bool) -> usize {
    let columns = if two_columns { 2 } else { 1 };
    let sample_rows = STATE_SAMPLE_COUNT.saturating_add(columns - 1) / columns;
    let existing_start = 1usize
        .saturating_add(sample_rows.saturating_mul(STATE_SAMPLE_STRIDE))
        .saturating_add(1);
    existing_start.saturating_add(EXISTING_STATES_HEIGHT)
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

fn window_for_source(
    parent: VerticalWindow,
    x: u16,
    width: u16,
    source_start: usize,
    logical_rows: usize,
) -> Option<VerticalWindow> {
    if width == 0 || logical_rows == 0 {
        return None;
    }
    let visible = parent.visible_source_range();
    let start = visible.start.max(source_start);
    let end = visible.end.min(source_start.saturating_add(logical_rows));
    if start >= end {
        return None;
    }
    let y = parent.physical_row_for_source(start)?;
    let visible_body = Rect::new(x, y, width, u16::try_from(end.saturating_sub(start)).ok()?);
    VerticalWindow::new(
        visible_body,
        logical_rows,
        start.saturating_sub(source_start),
    )
    .ok()
}

#[derive(Clone, Copy)]
struct PartialTextAreaOptions {
    control_rows: usize,
    derived: StateFlags,
}

fn draw_partial_textarea_field(
    ui: &mut Ui<'_>,
    window: VerticalWindow,
    id: Id,
    label: &str,
    control: &TextArea<'_>,
    state: &TextAreaState,
    options: PartialTextAreaOptions,
) {
    let area = window.visible_body();
    if area.is_empty() {
        return;
    }
    let flags = ui.state(id) | options.derived;
    let container = ui
        .style(Family::FIELD, Variant::DEFAULT, Part::CONTAINER, flags)
        .style;
    ui.fill(area, container);
    if let Some(y) = window.physical_row_for_source(0) {
        let label_area = Rect::new(area.x.saturating_add(2), y, area.width.saturating_sub(2), 1);
        let label_style = ui
            .style(Family::FIELD, Variant::DEFAULT, Part::LABEL, flags)
            .style;
        let _ = ui.paint_str(label_area, label, label_style);
    }
    if let Some(control_window) =
        window_for_source(window, area.x, area.width, 1, options.control_rows)
    {
        let _ = control.draw_window(ui, control_window, state);
    }
}

fn draw_partial_field_gutter(
    ui: &mut Ui<'_>,
    field_window: VerticalWindow,
    flags: StateFlags,
    control_rows: usize,
) {
    let area = field_window.visible_body();
    let Some(control_window) = window_for_source(field_window, area.x, area.width, 1, control_rows)
    else {
        return;
    };
    for source_row in control_window.visible_source_range() {
        if let Some(y) = control_window.physical_row_for_source(source_row) {
            legacy_field_gutter(ui, Rect::new(area.x, y, area.width, 1), flags);
        }
    }
}

fn legacy_field_gutter(ui: &mut Ui<'_>, area: Rect, flags: StateFlags) {
    if area.is_empty() {
        return;
    }
    let field = ui.style(Family::FIELD, Variant::DEFAULT, Part::FIELD, flags);
    let mut gutter = ui
        .style(Family::FIELD, Variant::DEFAULT, Part::GUTTER, flags)
        .style;
    gutter = gutter.with_bg_from(field.style);
    if !flags.contains(StateFlags::FOCUSED) {
        gutter = gutter.with_fg_from_bg(field.style);
    }
    for offset in 0..area.height {
        let _ = ui.paint_str(
            Rect {
                y: area.y.saturating_add(offset),
                height: 1,
                ..area
            },
            "▎",
            gutter,
        );
    }
}

fn legacy_scroll_help(ui: &mut Ui<'_>, area: Rect) {
    let row = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(9),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    if row.is_empty() {
        return;
    }
    let style = ui
        .style(
            Family::FIELD,
            Variant::DEFAULT,
            Part::HELP,
            StateFlags::empty(),
        )
        .style;
    let meta_x = row.right().saturating_sub(10);
    let help_width = meta_x.saturating_sub(row.x).saturating_sub(2);
    let help = truncate("Enter inserts a newline · Esc finishes", help_width);
    let blank = " ".repeat(usize::from(row.width));
    let _ = ui.paint_str(row, &blank, style);
    let _ = ui.paint_str(row, &help, style);
    let _ = ui.paint_str(
        Rect {
            x: meta_x,
            width: 9,
            ..row
        },
        "1–8 of 28",
        style,
    );
}

fn legacy_textarea_placeholder(ui: &mut Ui<'_>, area: Rect, text: &str) {
    let inner = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(3),
        height: 1,
    };
    if inner.is_empty() {
        return;
    }
    let style = ui
        .style(
            Family::TEXTAREA,
            Variant::DEFAULT,
            Part::PLACEHOLDER,
            StateFlags::empty(),
        )
        .style;
    let fitted = truncate(text, inner.width);
    let blank = " ".repeat(usize::from(inner.width));
    let _ = ui.paint_str(inner, &blank, style);
    let _ = ui.paint_str(inner, &fitted, style);
}

fn legacy_field_error(ui: &mut Ui<'_>, area: Rect, text: &str) {
    let row = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(5),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    legacy_field_error_row(ui, row, text);
}

fn legacy_field_error_row(ui: &mut Ui<'_>, row: Rect, text: &str) {
    if row.is_empty() {
        return;
    }
    let style = ui
        .style(
            Family::FIELD,
            Variant::DEFAULT,
            Part::HELP,
            StateFlags::ERROR,
        )
        .style;
    let fitted = truncate(text, row.width);
    let blank = " ".repeat(usize::from(row.width));
    let _ = ui.paint_str(row, &blank, style);
    let _ = ui.paint_str(row, &fitted, style);
}

#[derive(Debug)]
pub struct TextAreasPage {
    page_tabs: TabsState,
    blur_action_state: SelectState,
    blur_policy: BlurPolicy,
    edit_entry_state: SelectState,
    edit_entry: EditEntry,
    focus_select_state: SelectState,
    focus_select: FocusSelect,
    escape_behavior_state: SelectState,
    esc_reverts: bool,
    states_scroll: ScrollState,
    value: String,
    state: TextAreaState,
    hint_draw: Cell<HintDrawFacts>,
    editing_reference_value: String,
    editing_reference_state: TextAreaState,
    editing_reference_ready: bool,
    last: &'static str,
}

impl TextAreasPage {
    pub fn new() -> Self {
        Self {
            page_tabs: TabsState::default(),
            blur_action_state: SelectState::default(),
            blur_policy: BlurPolicy::Commit,
            edit_entry_state: SelectState::default(),
            edit_entry: EditEntry::Immediate,
            focus_select_state: SelectState::default(),
            focus_select: FocusSelect::CaretEnd,
            escape_behavior_state: SelectState::default(),
            esc_reverts: false,
            states_scroll: ScrollState::default(),
            value: checklist(),
            state: TextAreaState::default(),
            hint_draw: Cell::new(HintDrawFacts::default()),
            editing_reference_value: LONG_UNICODE_LINE.to_owned(),
            editing_reference_state: TextAreaState::default(),
            editing_reference_ready: false,
            last: "ready",
        }
    }

    fn section(&self) -> usize {
        match self.page_tabs.active().or(self.page_tabs.cursor()) {
            Some(ItemKey::Index(index)) => index.min(PAGE_TABS_ITEMS.len().saturating_sub(1)),
            _ => 0,
        }
    }

    fn drain_hidden_closed_option(
        cx: &mut Cx<'_>,
        section: usize,
        id: Id,
        state: &SelectState,
    ) -> bool {
        if section != 1 || cx.area(id).is_some() || state.is_open() || cx.is_open(id) {
            return false;
        }
        for _ in cx.intents(id) {}
        true
    }

    fn update_blur_action(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if Self::drain_hidden_closed_option(
            cx,
            self.section(),
            BLUR_ACTION,
            &self.blur_action_state,
        ) {
            return Response::ignored();
        }
        let mut selected = blur_policy_index(self.blur_policy);
        let response = Select::new(BLUR_ACTION).popup_rows(3).update_in_form(
            cx,
            &mut self.blur_action_state,
            &mut selected,
            BLUR_ACTION_ITEMS,
            false,
        );
        self.blur_policy = blur_policy_at(selected);
        response.erase()
    }

    fn update_edit_entry(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if Self::drain_hidden_closed_option(cx, self.section(), EDIT_ENTRY, &self.edit_entry_state)
        {
            return Response::ignored();
        }
        let mut selected = edit_entry_index(self.edit_entry);
        let response = Select::new(EDIT_ENTRY).popup_rows(3).update_in_form(
            cx,
            &mut self.edit_entry_state,
            &mut selected,
            EDIT_ENTRY_ITEMS,
            false,
        );
        self.edit_entry = edit_entry_at(selected);
        response.erase()
    }

    fn update_focus_select(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if Self::drain_hidden_closed_option(
            cx,
            self.section(),
            FOCUS_SELECT,
            &self.focus_select_state,
        ) {
            return Response::ignored();
        }
        let mut selected = focus_select_index(self.focus_select);
        let response = Select::new(FOCUS_SELECT).popup_rows(3).update_in_form(
            cx,
            &mut self.focus_select_state,
            &mut selected,
            FOCUS_SELECT_ITEMS,
            false,
        );
        self.focus_select = focus_select_at(selected);
        response.erase()
    }

    fn update_escape_behavior(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if Self::drain_hidden_closed_option(
            cx,
            self.section(),
            ESCAPE_BEHAVIOR,
            &self.escape_behavior_state,
        ) {
            return Response::ignored();
        }
        let mut selected = escape_behavior_index(self.esc_reverts);
        let response = Select::new(ESCAPE_BEHAVIOR).popup_rows(2).update_in_form(
            cx,
            &mut self.escape_behavior_state,
            &mut selected,
            ESCAPE_BEHAVIOR_ITEMS,
            false,
        );
        self.esc_reverts = escape_behavior_at(selected);
        response.erase()
    }

    fn draw_tabs(&self, ui: &mut Ui<'_>, page_area: Rect) -> Option<Rect> {
        let tabs_area = Rect {
            x: page_area.x.saturating_add(PAGE_TITLE_TABS_X),
            y: page_area.y,
            width: page_area.width.saturating_sub(PAGE_TITLE_TABS_X),
            height: 1,
        };
        if tabs_area.is_empty() {
            return None;
        }
        let tabs = Tabs::new(PAGE_TABS);
        let previous =
            ui.layout(PAGE_TABS)
                .map(|facts| termrock::navigation::tabs::TabWindowReceipt {
                    viewport_len: facts.viewport_len,
                    content_len: facts.content_len,
                });

        let layout = tabs.layout_default(
            tabs_area,
            page_area,
            &self.page_tabs,
            PAGE_TABS_ITEMS,
            previous,
        );
        let drawn = tabs
            .draw_layout(ui, &layout)
            .unwrap_or_else(|_| tabs.draw(ui, tabs_area, &self.page_tabs, PAGE_TABS_ITEMS));
        let visible = drawn.intersection(ui.full());
        if ui.is_inert() {
            return None;
        }
        (!visible.is_empty()).then_some(visible)
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        Panel::new(OPTIONS)
            .kind(PanelKind::Card)
            .title("Options")
            .draw(ui, area, |ui, inner| {
                if inner.is_empty() {
                    return;
                }
                let controls = Rect {
                    height: inner.height.min(8),
                    ..inner
                };
                let columns = layout::columns(controls, &[Track::Flex(1), Track::Flex(1)], 2);
                let left = columns.first().copied().unwrap_or(controls);
                let right = columns.get(1).copied().unwrap_or(controls);
                let edit_entry = draw_option(
                    ui,
                    left,
                    "Edit entry",
                    EDIT_ENTRY,
                    &self.edit_entry_state,
                    EDIT_ENTRY_ITEMS,
                    3,
                );
                let focus_select = draw_option(
                    ui,
                    lower_option_area(left),
                    "New-session selection",
                    FOCUS_SELECT,
                    &self.focus_select_state,
                    FOCUS_SELECT_ITEMS,
                    3,
                );
                let escape_behavior = draw_option(
                    ui,
                    right,
                    "Escape behavior (Explicit commits)",
                    ESCAPE_BEHAVIOR,
                    &self.escape_behavior_state,
                    ESCAPE_BEHAVIOR_ITEMS,
                    2,
                );
                let blur_action = draw_option(
                    ui,
                    lower_option_area(right),
                    "On blur",
                    BLUR_ACTION,
                    &self.blur_action_state,
                    BLUR_ACTION_ITEMS,
                    3,
                );
                self.hint_draw.set(HintDrawFacts {
                    edit_entry,
                    focus_select,
                    escape_behavior,
                    blur_action,
                    ..self.hint_draw.get()
                });
            });
    }

    fn draw_reference_sample(&self, ui: &mut Ui<'_>, window: VerticalWindow, index: usize) {
        if window.visible_body().is_empty() {
            return;
        }
        let id = STATE_REFERENCES.index(index);
        let idle = TextAreaState::default();
        let (label, value, placeholder, state) = match index {
            0 => (
                "Rest",
                "Add one clear step per line.\nKeep the scope small.",
                None,
                &idle,
            ),
            1 => ("Hover", "A hovered text area reference.", None, &idle),
            2 => ("Focused", "A focused text area reference.", None, &idle),
            3 => (
                "Editing · long Unicode line",
                self.editing_reference_value.as_str(),
                None,
                &self.editing_reference_state,
            ),
            4 => ("Empty", "", Some("Nothing to add yet"), &idle),
            5 => ("Overflow", self.value.as_str(), None, &idle),
            _ => ("Narrow", "Review the short field layout.", None, &idle),
        };
        let mut sample_window = window;
        if index == 6 {
            let area = window.visible_body();
            let width = area.width.min(24);
            let narrow = Rect {
                x: area.x.saturating_add(area.width.saturating_sub(width) / 2),
                width,
                ..area
            };
            let Ok(narrow_window) =
                VerticalWindow::new(narrow, STATE_SAMPLE_STRIDE, window.first_source_row())
            else {
                return;
            };
            sample_window = narrow_window;
        }
        let mut control = TextArea::new(id, 3).blur(BlurPolicy::Commit).value(value);
        if let Some(placeholder) = placeholder {
            control = control.placeholder(placeholder);
        }
        let field = Field::new(label, control).optional_suffix(false);
        let target = match index {
            1 => Some(ReferenceTarget::new(id, ReferenceState::HOVERED)),
            2 | 3 => Some(ReferenceTarget::new(
                id,
                ReferenceState::FOCUSED.union(ReferenceState::FOCUS_VISIBLE),
            )),
            _ => None,
        };
        ui.reference(target, |ui| {
            if sample_window.first_source_row() == 0
                && usize::from(sample_window.visible_body().height) == STATE_SAMPLE_STRIDE
            {
                let _ = field.draw(ui, sample_window.visible_body(), state);
            } else {
                draw_partial_textarea_field(
                    ui,
                    sample_window,
                    id,
                    label,
                    field.control(),
                    state,
                    PartialTextAreaOptions {
                        control_rows: 3,
                        derived: StateFlags::empty(),
                    },
                );
            }
        });
    }

    fn draw_reference_states(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let two_columns = area.width >= 72;
        let logical_rows = state_content_rows(two_columns);
        let view =
            ScrollRegion::new(STATES_SCROLL).draw(ui, area, &self.states_scroll, logical_rows);
        let visible_view = view.intersection(ui.full());
        self.hint_draw.set(HintDrawFacts {
            states: (logical_rows > usize::from(view.height)
                && !ui.is_inert()
                && !visible_view.is_empty())
            .then_some(visible_view),
            ..self.hint_draw.get()
        });
        let Some(window) = scroll_window(view, logical_rows, &self.states_scroll) else {
            return;
        };
        ui.with_area(view, |ui| {
            let body = window.visible_body();
            if let Some(y) = window.physical_row_for_source(0) {
                let _ = ui.paint_str(
                    Rect::new(body.x, y, body.width, 1),
                    "Native references",
                    ui.surface_style(),
                );
            }
            let columns = if two_columns {
                layout::columns(body, &[Track::Flex(1), Track::Flex(1)], 2)
            } else {
                vec![body]
            };
            let column_count = columns.len().max(1);
            let sample_rows = STATE_SAMPLE_COUNT.saturating_add(column_count - 1) / column_count;
            for index in 0..STATE_SAMPLE_COUNT {
                let row = index / column_count;
                let column = index % column_count;
                let source_start = 1usize.saturating_add(row.saturating_mul(STATE_SAMPLE_STRIDE));
                let Some(column_area) = columns.get(column).copied() else {
                    continue;
                };
                if let Some(sample_window) = window_for_source(
                    window,
                    column_area.x,
                    column_area.width,
                    source_start,
                    STATE_SAMPLE_STRIDE,
                ) {
                    self.draw_reference_sample(ui, sample_window, index);
                }
            }
            let existing_start = 1usize
                .saturating_add(sample_rows.saturating_mul(STATE_SAMPLE_STRIDE))
                .saturating_add(1);
            if let Some(states_window) = window_for_source(
                window,
                body.x,
                body.width,
                existing_start,
                EXISTING_STATES_HEIGHT,
            ) {
                let panel_y = body
                    .y
                    .saturating_add(u16::try_from(existing_start).unwrap_or(u16::MAX));
                let panel_area = Rect {
                    y: panel_y,
                    height: u16::try_from(EXISTING_STATES_HEIGHT).unwrap_or(u16::MAX),
                    ..body
                };
                let _ = states_panel().draw_window(
                    ui,
                    panel_area,
                    states_window,
                    |ui, inner_window| {
                        let Some(inner_window) = inner_window else {
                            return;
                        };
                        if inner_window.first_source_row() == 0
                            && usize::from(inner_window.visible_body().height)
                                == inner_window.logical_rows()
                        {
                            Self::draw_states(ui, inner_window.visible_body());
                        } else {
                            Self::draw_states_window(ui, inner_window);
                        }
                    },
                );
            }
        });
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, body: Rect) {
        let regions = layout::rows(body, &[Track::Fixed(13), Track::Fixed(1), Track::Fixed(10)]);
        let playground = regions.first().copied().unwrap_or(body);
        let playground = if body.width < 70 {
            Rect {
                width: playground.width.saturating_sub(1),
                ..playground
            }
        } else {
            playground
        };
        playground_panel(playground_meta(self.edit_entry, body.width < 70)).draw(
            ui,
            playground,
            |ui, inner| {
                let columns = layout::columns(inner, &[Track::Flex(1), Track::Flex(1)], 3);
                let task = {
                    let area = columns.first().copied().unwrap_or(inner);
                    Rect {
                        width: area.width.saturating_sub(1),
                        ..area
                    }
                };
                let task_field = task_field(
                    &self.value,
                    self.blur_policy,
                    self.edit_entry,
                    self.focus_select,
                    self.esc_reverts,
                );
                let body_control = visible_textarea_control(ui, task, task_field.control());
                let _ = task_field.draw(ui, task, &self.state);
                self.hint_draw.set(HintDrawFacts {
                    body: body_control,
                    ..self.hint_draw.get()
                });
                legacy_field_gutter(
                    ui,
                    Rect {
                        y: task.y.saturating_add(1),
                        height: 8,
                        ..task
                    },
                    StateFlags::empty(),
                );
                legacy_scroll_help(ui, task);
                let notes = {
                    let area = columns.get(1).copied().unwrap_or(inner);
                    Rect {
                        width: area.width.saturating_sub(u16::from(body.width >= 70)),
                        ..area
                    }
                };
                ui.reference(None, |ui| {
                    notes_field().draw(ui, notes, &TextAreaState::default());
                    legacy_textarea_placeholder(ui, notes, "Anything the agent should know…");
                    legacy_field_gutter(
                        ui,
                        Rect {
                            y: notes.y.saturating_add(1),
                            height: 8,
                            ..notes
                        },
                        StateFlags::empty(),
                    );
                });
            },
        );
        if let Some(states) = regions.get(2).copied() {
            states_panel().draw(ui, states, |ui, inner| {
                Self::draw_states(ui, inner);
            });
        }
    }

    fn body_hints(&self) -> &'static [(&'static str, &'static str)] {
        if self.state.is_editing() {
            return &[
                ("Enter", "Newline"),
                ("Esc", "Done"),
                ("Shift+↑↓", "Select"),
                ("Tab", "Next"),
                ("PgUp/PgDn", "Scroll"),
            ];
        }
        match self.edit_entry {
            EditEntry::Immediate => &[("Enter", "Edit"), ("↑ ↓", "Scroll")],
            EditEntry::TypeToEdit => &[
                ("Type", "Begin edit"),
                ("Paste", "Begin edit"),
                ("Ctrl+L", "Select all"),
            ],
            EditEntry::Explicit => &[("Enter", "Begin edit"), ("Ctrl+L", "Select all")],
        }
    }

    fn options_available(&self, facts: HintDrawFacts) -> bool {
        facts.blur_action.is_some()
            || facts.edit_entry.is_some()
            || facts.focus_select.is_some()
            || facts.escape_behavior.is_some()
    }

    fn hints_for_focus(&self, focus: Option<Id>) -> &'static [(&'static str, &'static str)] {
        if focus == Some(PAGE_TABS) {
            return &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
        }
        if matches!(
            focus,
            Some(BLUR_ACTION) | Some(EDIT_ENTRY) | Some(FOCUS_SELECT) | Some(ESCAPE_BEHAVIOR)
        ) {
            return &[("↑ ↓", "Choose"), ("Enter", "Open / choose")];
        }
        if focus == Some(BODY) {
            return self.body_hints();
        }
        let facts = self.hint_draw.get();
        match self.section() {
            0 if facts.body.is_some() => self.body_hints(),
            1 if self.options_available(facts) => &[("↑ ↓", "Choose"), ("Enter", "Open / choose")],
            2 if facts.states.is_some() => &[("Wheel", "Scroll")],
            _ => &[],
        }
    }
}

impl Default for TextAreasPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for TextAreasPage {
    fn title(&self) -> &'static str {
        "Text areas"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Tabs::new(PAGE_TABS)
            .update(cx, &mut self.page_tabs, PAGE_TABS_ITEMS)
            .erase();
        let edit = body_area(
            self.blur_policy,
            self.edit_entry,
            self.focus_select,
            self.esc_reverts,
        )
        .update(cx, &mut self.state, &mut self.value);
        if let Some(action) = edit.action_ref() {
            self.last = match action {
                TextAction::Changed => "draft changed",
                TextAction::Committed => "document committed",
                TextAction::Cancelled => "draft cancelled",
                TextAction::MoveNext | TextAction::MovePrev => "focus moved",
            };
        }
        response |= edit.erase();
        response |= self.update_blur_action(cx);
        response |= self.update_edit_entry(cx);
        response |= self.update_focus_select(cx);
        response |= self.update_escape_behavior(cx);
        if self.section() == 2 {
            let content_rows = cx
                .layout(STATES_SCROLL)
                .map_or_else(|| state_content_rows(false), |layout| layout.content_len);
            response |=
                ScrollRegion::new(STATES_SCROLL).update(cx, &mut self.states_scroll, content_rows);
        }
        if !self.editing_reference_ready {
            let _ = TextArea::new(EDITING_REFERENCE, 3).execute_edit_command(
                cx,
                &mut self.editing_reference_state,
                &mut self.editing_reference_value,
                TextEditCommand::SelectAll,
            );
            self.editing_reference_ready = true;
        }
        PageUpdate {
            response,
            status: None,
        }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        self.hint_draw.set(HintDrawFacts::default());
        frame(ui, area, self.title(), "", |ui, body| {
            match self.section() {
                0 => self.draw_playground(ui, body),
                1 => self.draw_options(ui, body),
                _ => self.draw_reference_states(ui, body),
            }
            let tabs = self.draw_tabs(ui, area);
            self.hint_draw.set(HintDrawFacts {
                tabs,
                ..self.hint_draw.get()
            });
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        let facts = self.hint_draw.get();
        if facts.tabs.is_some() && ui.state(PAGE_TABS).contains(StateFlags::FOCUSED) {
            return self.hints_for_focus(Some(PAGE_TABS));
        }
        for (id, visible) in [
            (BLUR_ACTION, facts.blur_action),
            (EDIT_ENTRY, facts.edit_entry),
            (FOCUS_SELECT, facts.focus_select),
            (ESCAPE_BEHAVIOR, facts.escape_behavior),
        ] {
            if visible.is_some() && ui.state(id).contains(StateFlags::FOCUSED) {
                return self.hints_for_focus(Some(id));
            }
        }
        if facts.body.is_some() && ui.state(BODY).contains(StateFlags::FOCUSED) {
            return self.hints_for_focus(Some(BODY));
        }
        self.hints_for_focus(None)
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.state.is_editing()
    }
}

impl TextAreasPage {
    fn draw_states(ui: &mut Ui<'_>, inner: Rect) {
        let columns = layout::columns(inner, &[Track::Flex(1), Track::Flex(1)], 3);
        let mut commit_state = TextAreaState::default();
        commit_state.set_error(Some(FieldError::new(
            "Use the imperative mood and explain why",
        )));
        ui.reference(None, |ui| {
            let transcript = {
                let area = columns.first().copied().unwrap_or(inner);
                Rect {
                    width: area.width.saturating_sub(1),
                    ..area
                }
            };
            transcript_field().draw(ui, transcript, &TextAreaState::default());
            legacy_field_gutter(
                ui,
                Rect {
                    y: transcript.y.saturating_add(1),
                    height: 4,
                    ..transcript
                },
                StateFlags::DISABLED,
            );
            let commit = {
                let area = columns.get(1).copied().unwrap_or(inner);
                Rect {
                    width: area.width.saturating_sub(1),
                    ..area
                }
            };
            commit_field().draw(ui, commit, &commit_state);
            legacy_field_error(ui, commit, "Use the imperative mood and explain why");
            legacy_field_gutter(
                ui,
                Rect {
                    y: commit.y.saturating_add(1),
                    height: 4,
                    ..commit
                },
                StateFlags::ERROR,
            );
        });
    }

    fn draw_states_window(ui: &mut Ui<'_>, inner: VerticalWindow) {
        let visible = inner.visible_body();
        let columns = layout::columns(visible, &[Track::Flex(1), Track::Flex(1)], 3);
        let mut commit_state = TextAreaState::default();
        commit_state.set_error(Some(FieldError::new(
            "Use the imperative mood and explain why",
        )));
        ui.reference(None, |ui| {
            let transcript_area = {
                let area = columns.first().copied().unwrap_or(visible);
                Rect {
                    width: area.width.saturating_sub(1),
                    ..area
                }
            };
            let transcript = transcript_field();
            if let Some(field_window) = window_for_source(
                inner,
                transcript_area.x,
                transcript_area.width,
                0,
                inner.logical_rows(),
            ) {
                draw_partial_textarea_field(
                    ui,
                    field_window,
                    TRANSCRIPT,
                    "Read-only transcript",
                    transcript.control(),
                    &TextAreaState::default(),
                    PartialTextAreaOptions {
                        control_rows: 4,
                        derived: StateFlags::empty(),
                    },
                );
                draw_partial_field_gutter(ui, field_window, StateFlags::DISABLED, 4);
            }

            let commit_area = {
                let area = columns.get(1).copied().unwrap_or(visible);
                Rect {
                    width: area.width.saturating_sub(1),
                    ..area
                }
            };
            let commit = commit_field();
            if let Some(field_window) = window_for_source(
                inner,
                commit_area.x,
                commit_area.width,
                0,
                inner.logical_rows(),
            ) {
                draw_partial_textarea_field(
                    ui,
                    field_window,
                    COMMIT,
                    "Commit message",
                    commit.control(),
                    &commit_state,
                    PartialTextAreaOptions {
                        control_rows: 4,
                        derived: StateFlags::ERROR,
                    },
                );
                draw_partial_field_gutter(ui, field_window, StateFlags::ERROR, 4);
                if let Some(y) = field_window.physical_row_for_source(5) {
                    legacy_field_error_row(
                        ui,
                        Rect::new(
                            commit_area.x.saturating_add(2),
                            y,
                            commit_area.width.saturating_sub(2),
                            1,
                        ),
                        "Use the imperative mood and explain why",
                    );
                }
            }
        });
    }
}
