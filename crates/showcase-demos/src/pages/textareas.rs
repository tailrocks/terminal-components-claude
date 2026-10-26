use termrock::fields::TextEditCommand;
use termrock::{
    BlurPolicy, Cx, Family, Field, FieldError, FrameRead, Id, ItemKey, Panel, PanelKind, Part,
    Rect, ReferenceState, ReferenceTarget, Response, ScrollRegion, ScrollState, Select,
    SelectState, StateFlags, Tabs, TabsState, TextAction, TextArea, TextAreaState, Track, Ui,
    Variant, author::VerticalWindow, layout, truncate,
};

use super::{Page, PageUpdate, frame};

const BODY: Id = termrock::Id::root("showcase_ui::pages::textareas::textareas.body");
const NOTES: Id = termrock::Id::root("showcase_ui::pages::textareas::textareas.notes");
const TRANSCRIPT: Id = termrock::Id::root("showcase_ui::pages::textareas::textareas.transcript");
const COMMIT: Id = termrock::Id::root("showcase_ui::pages::textareas::textareas.commit");
const PAGE_TABS: Id = termrock::Id::root("showcase_ui::pages::textareas::page_tabs");
const BLUR_ACTION: Id = termrock::Id::root("showcase_ui::pages::textareas::option.blur_action");
const STATES_SCROLL: Id = termrock::Id::root("showcase_ui::pages::textareas::states_scroll");
const PLAYGROUND: Id = termrock::Id::root("showcase_ui::pages::textareas::textareas.playground");
const OPTIONS: Id = termrock::Id::root("showcase_ui::pages::textareas::options");
const STATES: Id = termrock::Id::root("showcase_ui::pages::textareas::textareas.states");
const STATE_REFERENCES: Id = termrock::Id::root("showcase_ui::pages::textareas::states.references");
const EDITING_REFERENCE: Id = STATE_REFERENCES.index(3);

const PAGE_TABS_ITEMS: &[&str] = &["Playground", "Options", "States"];
const BLUR_ACTION_ITEMS: &[&str] = &["Commit", "Cancel", "Keep"];
const PAGE_TITLE_TABS_X: u16 = 12;
const STATE_SAMPLE_COUNT: usize = 7;
const STATE_SAMPLE_STRIDE: usize = 5;
const EXISTING_STATES_HEIGHT: usize = 10;
const LONG_UNICODE_LINE: &str = " 4. Run the integration suite and attach the report. Review 日本語 · 👩‍💻 · cafe\u{301} through this long Unicode line.";

fn body_area(blur: BlurPolicy) -> TextArea<'static> {
    TextArea::new(BODY, 8)
        .placeholder("Write a checklist")
        .blur(blur)
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

fn task_field(value: &str, blur: BlurPolicy) -> Field<'_, TextArea<'_>> {
    Field::new("Task description", body_area(blur).value(value)).optional_suffix(false)
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
    states_scroll: ScrollState,
    value: String,
    state: TextAreaState,
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
            states_scroll: ScrollState::default(),
            value: checklist(),
            state: TextAreaState::default(),
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

    fn update_blur_action(&mut self, cx: &mut Cx<'_>) -> Response<()> {
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

    fn draw_tabs(&self, ui: &mut Ui<'_>, page_area: Rect) {
        let tabs_area = Rect {
            x: page_area.x.saturating_add(PAGE_TITLE_TABS_X),
            y: page_area.y,
            width: page_area.width.saturating_sub(PAGE_TITLE_TABS_X),
            height: 1,
        };
        if tabs_area.is_empty() {
            return;
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
        if tabs.draw_layout(ui, &layout).is_err() {
            let _ = tabs.draw(ui, tabs_area, &self.page_tabs, PAGE_TABS_ITEMS);
        }
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
                let rows = layout::rows(inner, &[Track::Fixed(1), Track::Fixed(3)]);
                let label = rows.first().copied().unwrap_or(inner);
                let field = rows.get(1).copied().unwrap_or(inner);
                let _ = ui.paint_str(label, "On blur", ui.surface_style());
                let _ = Select::new(BLUR_ACTION).popup_rows(3).draw(
                    ui,
                    field,
                    &self.blur_action_state,
                    BLUR_ACTION_ITEMS,
                );
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
        playground_panel(if body.width < 70 {
            "Enter Edit · Esc Done · Tab Next"
        } else {
            "Enter Edit · Esc Done · Tab Next "
        })
        .draw(ui, playground, |ui, inner| {
            let columns = layout::columns(inner, &[Track::Flex(1), Track::Flex(1)], 3);
            let task = {
                let area = columns.first().copied().unwrap_or(inner);
                Rect {
                    width: area.width.saturating_sub(1),
                    ..area
                }
            };
            task_field(&self.value, self.blur_policy).draw(ui, task, &self.state);
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
        });
        if let Some(states) = regions.get(2).copied() {
            states_panel().draw(ui, states, |ui, inner| {
                Self::draw_states(ui, inner);
            });
        }
    }

    fn hints_for_focus(&self, focus: Option<Id>) -> &'static [(&'static str, &'static str)] {
        if focus == Some(PAGE_TABS) {
            return &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
        }
        if focus == Some(BLUR_ACTION) {
            return &[("↑ ↓", "Choose"), ("Enter", "Open / choose")];
        }
        match (self.section(), self.state.is_editing()) {
            (0, true) => &[
                ("Enter", "Newline"),
                ("Esc", "Done"),
                ("Shift+↑↓", "Select"),
                ("Tab", "Next"),
                ("PgUp/PgDn", "Scroll"),
            ],
            (0, false) => &[("Enter", "Edit"), ("↑ ↓", "Scroll")],
            (1, _) => &[("↑ ↓", "Choose"), ("Enter", "Open / choose")],
            _ => &[("Wheel", "Scroll")],
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
        let edit = body_area(self.blur_policy).update(cx, &mut self.state, &mut self.value);
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
        frame(ui, area, self.title(), "", |ui, body| {
            match self.section() {
                0 => self.draw_playground(ui, body),
                1 => self.draw_options(ui, body),
                _ => self.draw_reference_states(ui, body),
            }
            self.draw_tabs(ui, area);
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(PAGE_TABS).contains(StateFlags::FOCUSED) {
            return self.hints_for_focus(Some(PAGE_TABS));
        }
        if ui.state(BLUR_ACTION).contains(StateFlags::FOCUSED) {
            return self.hints_for_focus(Some(BLUR_ACTION));
        }
        if ui.state(BODY).contains(StateFlags::FOCUSED) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use termrock::{App, Axis, KeyCode, KeyModifiers, PartRef, Theme};
    use termrock_test_support::Harness;

    const PAGE_AREA: Rect = Rect::new(11, 2, 96, 34);

    struct Host {
        page: TextAreasPage,
        area: Rect,
        hints: Cell<&'static [(&'static str, &'static str)]>,
    }

    impl App for Host {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.page.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.page.draw(ui, self.area);
            self.hints.set(self.page.hints(ui));
        }
    }

    fn host(area: Rect) -> Harness<Host> {
        Harness::new(
            Host {
                page: TextAreasPage::new(),
                area,
                hints: Cell::new(&[]),
            },
            Theme::junie(),
            120,
            40,
        )
    }

    fn tab(index: usize) -> PartRef {
        PartRef::item(Part::TAB, ItemKey::index(index))
    }

    fn render_states_at(harness: &mut Harness<Host>, offset: usize) {
        harness.app_mut().page.states_scroll.scroll_to(offset);
        harness.draw();
        assert_eq!(harness.app().page.states_scroll.offset(), offset);
    }

    #[test]
    fn section_tabs_use_the_page_area_and_retire_hidden_controls() {
        let mut harness = host(PAGE_AREA);
        let options_tab = harness
            .area_of_part(PAGE_TABS, tab(1))
            .expect("the Options tab is registered");
        assert!(options_tab.x >= PAGE_AREA.x);
        assert!(options_tab.right() <= PAGE_AREA.right());
        assert_eq!(options_tab.y, PAGE_AREA.y);

        let _ = harness.click_part(PAGE_TABS, tab(1));
        assert_eq!(harness.app().page.section(), 1);
        assert!(harness.area_of(BLUR_ACTION).is_some());

        assert!(harness.tab_to(PAGE_TABS));
        let _ = harness.key(KeyCode::Char('3'));
        assert_eq!(harness.app().page.section(), 2);
        assert!(harness.area_of(BLUR_ACTION).is_none());
    }

    #[test]
    fn blur_option_changes_the_native_textarea_and_preserves_committed_value() {
        let mut harness = host(PAGE_AREA);
        let _ = harness.click_part(PAGE_TABS, tab(1));
        assert!(harness.tab_to(BLUR_ACTION));
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.key(KeyCode::Down);
        let _ = harness.key(KeyCode::Enter);
        assert_eq!(harness.app().page.blur_policy, BlurPolicy::Cancel);

        let committed = harness.app().page.value.clone();
        let _ = harness.click_part(PAGE_TABS, tab(0));
        assert!(harness.tab_to(BODY));
        let _ = harness.type_str("x");
        assert!(harness.app().page.state.is_editing());
        assert_eq!(harness.app().page.value, committed);

        let _ = harness.click_part(PAGE_TABS, tab(1));
        assert!(!harness.app().page.state.is_editing());
        assert_eq!(harness.app().page.value, committed);
        assert_eq!(harness.app().page.last, "draft cancelled");
    }

    #[test]
    fn hidden_open_blur_select_reconciles_after_outside_dismissal() {
        let mut harness = host(PAGE_AREA);
        let _ = harness.click_part(PAGE_TABS, tab(1));
        assert!(harness.tab_to(BLUR_ACTION));
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.app().page.blur_action_state.is_open());
        assert!(harness.is_open(BLUR_ACTION));

        let _ = harness.click_part(PAGE_TABS, tab(0));
        assert!(!harness.app().page.blur_action_state.is_open());
        assert!(!harness.is_open(BLUR_ACTION));
        let _ = harness.click_part(PAGE_TABS, tab(0));
        assert_eq!(harness.app().page.section(), 0);
        assert!(harness.area_of(BLUR_ACTION).is_none());
    }

    #[test]
    fn textarea_page_down_reveals_the_focused_document_row() {
        let mut harness = host(PAGE_AREA);
        assert!(harness.tab_to(BODY));
        let _ = harness.key_mod(KeyCode::Home, KeyModifiers::CONTROL);
        assert_eq!(harness.app().page.state.scroll().offset(), 0);
        let _ = harness.key(KeyCode::PageDown);
        assert!(harness.app().page.state.scroll().offset() > 0);
    }

    #[test]
    fn narrow_state_gallery_scrolls_with_the_wheel() {
        let mut harness = host(Rect::new(0, 0, 51, 16));
        let _ = harness.click_part(PAGE_TABS, tab(2));
        let view = harness
            .area_of(STATES_SCROLL)
            .expect("the states gallery registers its scroll region");
        let _ = harness.wheel(
            Axis::V,
            3,
            view.x.saturating_add(1),
            view.y.saturating_add(1),
        );
        assert_eq!(harness.app().page.states_scroll.offset(), 9);
    }

    #[test]
    fn partial_state_windows_keep_native_cells_and_reach_existing_examples() {
        let mut harness = host(Rect::new(0, 0, 90, 10));
        let _ = harness.click_part(PAGE_TABS, tab(2));
        let view = harness
            .area_of(STATES_SCROLL)
            .expect("the short States viewport is registered");
        assert!(view.height > 0 && view.height < 10);

        let partial_bottom = harness.text();
        assert!(partial_bottom.contains("A focused text area reference."));
        assert!(partial_bottom.contains("Run the integration suite"));

        render_states_at(&mut harness, 2);
        let partial = harness.text();
        assert!(partial.contains("Add one clear step per line."));
        assert!(partial.contains("A hovered text area reference."));

        render_states_at(&mut harness, 3);
        assert!(harness.text().contains("Keep the scope small."));

        render_states_at(&mut harness, 10);
        let lower_samples = harness.text();
        assert!(lower_samples.contains("Nothing to add yet"));
        assert!(lower_samples.contains("Read src/api/billing.rs"));
        assert!(lower_samples.contains("Narrow"));

        render_states_at(&mut harness, 22);
        let existing = harness.text();
        assert!(existing.contains("Junie: Reading 14 files…"));
        assert!(existing.contains("imperative mood"));
    }

    #[test]
    fn section_digits_are_hinted_for_tabs_and_remain_text_in_the_editor() {
        let mut harness = host(PAGE_AREA);
        assert!(harness.tab_to(BODY));
        assert_eq!(harness.focus(), Some(BODY));
        assert!(
            !harness
                .app()
                .hints
                .get()
                .iter()
                .any(|(key, _)| *key == "1–3")
        );

        let before = harness.app().page.value.clone();
        let _ = harness.key(KeyCode::Char('1'));
        assert!(harness.app().page.state.is_editing());
        assert_eq!(harness.app().page.last, "draft changed");

        assert!(harness.tab_to(PAGE_TABS));
        assert!(
            harness
                .app()
                .hints
                .get()
                .iter()
                .any(|(key, _)| *key == "1–3")
        );
        let committed = harness.app().page.value.clone();
        assert_ne!(committed, before);

        let _ = harness.key(KeyCode::Char('2'));
        assert_eq!(harness.app().page.section(), 1);
        assert_eq!(harness.app().page.value, committed);
    }
}
