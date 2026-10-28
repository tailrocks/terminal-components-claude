use termrock::{
    BlurPolicy, Cx, Family, Field, FieldError, FocusSelect, Focusability, FrameRead, Id, Intent,
    ItemKey, KeyCode, Panel, PanelKind, Part, Rect, ReferenceState, ReferenceTarget, ScopeId,
    ScrollRegion, ScrollState, Select, SelectState, StateFlags, Status, Tabs, TabsState,
    TextAction, TextInput, TextInputState, Toggle, Track, Ui, Variant, author::VerticalWindow,
    layout, truncate,
};

use super::{Page, PageUpdate, frame};

const NAME: Id = termrock::Id::root("showcase_ui::pages::inputs::inputs.name");
const BRANCH: Id = termrock::Id::root("showcase_ui::pages::inputs::inputs.branch");
const CARD: Id = termrock::Id::root("showcase_ui::pages::inputs::inputs.card");
const OWNER: Id = termrock::Id::root("showcase_ui::pages::inputs::inputs.owner");
const TOKEN: Id = termrock::Id::root("showcase_ui::pages::inputs::inputs.token");
const SEARCH: Id = termrock::Id::root("showcase_ui::pages::inputs::inputs.search");
const API_KEY: Id = termrock::Id::root("showcase_ui::pages::inputs::inputs.api_key");
const STATE_REFERENCE: Id =
    termrock::Id::root("showcase_ui::pages::inputs::inputs.state_reference");
const FOCUS_SELECT: Id = termrock::Id::root("showcase_ui::pages::inputs.focus_select");
const PAGE_TABS: Id = termrock::Id::root("showcase_ui::pages::inputs.page_tabs");
const OPTIONS_SCROLL: Id = termrock::Id::root("showcase_ui::pages::inputs.options_scroll");
const STATES_SCROLL: Id = termrock::Id::root("showcase_ui::pages::inputs.states_scroll");
const BLUR_ACTION: Id = termrock::Id::root("showcase_ui::pages::inputs.option.blur_action");
const PLAIN_LABEL: Id = termrock::Id::root("showcase_ui::pages::inputs.option.plain_label");
const REQUIRED_MESSAGE: Id =
    termrock::Id::root("showcase_ui::pages::inputs.option.required_message");

const PAGE_SECTIONS: &[&str] = &["Playground", "Options", "States"];
const FOCUS_SELECT_ITEMS: &[&str] = &[
    "Focus selects: Select all",
    "Focus selects: Caret end",
    "Focus selects: Caret start",
];
const BLUR_ACTION_ITEMS: &[&str] = &["Commit + validate", "Commit", "Cancel", "Keep"];
const REQUIRED_MESSAGE_ITEMS: &[&str] = &["Required", "Name the project to continue"];
const OPTIONS_ROWS: usize = 4;
const OPTIONS_CONTROLS: [Id; OPTIONS_ROWS] =
    [FOCUS_SELECT, BLUR_ACTION, PLAIN_LABEL, REQUIRED_MESSAGE];
const STATE_ROWS: usize = 11;
const TAB_HINTS: &[(&str, &str)] = &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
const SELECT_HINTS: &[(&str, &str)] = &[("↑ ↓", "Choose"), ("Enter", "Open / choose")];
const TOGGLE_HINTS: &[(&str, &str)] = &[("Space", "Toggle"), ("Enter", "Toggle")];
const SCROLL_HINTS: &[(&str, &str)] = &[("Wheel", "Scroll"), ("PgUp / PgDn", "Page")];

fn email(value: &str) -> Result<(), FieldError> {
    if value.contains('@') && value.contains('.') {
        Ok(())
    } else {
        Err(FieldError::new("Enter a valid email address"))
    }
}

fn focus_select_policy(index: usize) -> FocusSelect {
    match index {
        1 => FocusSelect::CaretEnd,
        2 => FocusSelect::CaretStart,
        _ => FocusSelect::SelectAll,
    }
}

fn name_input<'a>(policy: FocusSelect, blur: BlurPolicy) -> TextInput<'a> {
    TextInput::new(NAME).focus_select(policy).blur(blur)
}

fn branch_input<'a>(policy: FocusSelect, blur: BlurPolicy) -> TextInput<'a> {
    TextInput::new(BRANCH)
        .focus_select(policy)
        .placeholder("feat/…")
        .blur(blur)
}

fn required_name(value: &str) -> Result<(), FieldError> {
    if value.trim().is_empty() {
        Err(FieldError::new("Required"))
    } else {
        Ok(())
    }
}

fn custom_required_name(value: &str) -> Result<(), FieldError> {
    if value.trim().is_empty() {
        Err(FieldError::new("Name the project to continue"))
    } else {
        Ok(())
    }
}

fn blur_policy(index: usize) -> BlurPolicy {
    match index {
        1 => BlurPolicy::Commit,
        2 => BlurPolicy::Cancel,
        3 => BlurPolicy::Keep,
        _ => BlurPolicy::CommitAndValidate,
    }
}

fn options_control_row(id: Id) -> Option<usize> {
    OPTIONS_CONTROLS
        .iter()
        .position(|candidate| *candidate == id)
}

fn card_panel() -> Panel<'static> {
    Panel::new(CARD).kind(PanelKind::Card)
}

fn playground_panel() -> Panel<'static> {
    card_panel()
        .title("Playground")
        .meta("Enter Edit · Esc Cancel · Tab Commit + next ")
}

fn state_reference_panel() -> Panel<'static> {
    Panel::new(STATE_REFERENCE)
        .kind(PanelKind::Card)
        .title("State reference")
        .meta("static ")
}

fn project_input<'a>(
    policy: FocusSelect,
    blur: BlurPolicy,
    required_message: usize,
) -> TextInput<'a> {
    if required_message == 1 {
        name_input(policy, blur).validate(&custom_required_name)
    } else {
        name_input(policy, blur).validate(&required_name)
    }
}

fn project_field<'a>(
    value: &'a str,
    policy: FocusSelect,
    blur: BlurPolicy,
    required_message: usize,
    error: Option<&'a str>,
) -> Field<'a, TextInput<'a>> {
    Field::new(
        "Project name",
        project_input(policy, blur, required_message).value(value),
    )
    .required(true)
    .error(error)
    .help("Used as the working directory name")
}

fn branch_field<'a>(
    value: &'a str,
    policy: FocusSelect,
    blur: BlurPolicy,
    plain_label: bool,
) -> Field<'a, TextInput<'a>> {
    Field::new("Branch", branch_input(policy, blur).value(value))
        .plain(plain_label)
        .help("Leave empty to work on a detached checkout")
}

fn owner_field() -> Field<'static, TextInput<'static>> {
    Field::new(
        "Owner email",
        TextInput::new(OWNER).value("mira@example").validate(&email),
    )
    .required(true)
    .help("Enter a valid email address")
}

fn token_field() -> Field<'static, TextInput<'static>> {
    Field::new(
        "API token",
        TextInput::new(TOKEN)
            .value("jb_live_••••••••••••")
            .disabled(true),
    )
    .help("Managed by the organization")
}

fn search_field() -> Field<'static, TextInput<'static>> {
    Field::new(
        "Search files",
        TextInput::new(SEARCH).placeholder("Type a path or symbol…"),
    )
    .help("Selection: Shift+← →  ·  words: Ctrl+← →  ·  clear: Ctrl+U")
}

fn api_key_field() -> Field<'static, TextInput<'static>> {
    Field::new(
        "API key",
        TextInput::new(API_KEY).value("••••••••••••••••••••••••c1f2"),
    )
    .help("Masked while typing; the last four characters show once committed")
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
    let _ = ui.paint_str(Rect { width: 1, ..area }, "▎", gutter);
}

fn legacy_field_help(
    ui: &mut Ui<'_>,
    area: Rect,
    flags: StateFlags,
    message: &str,
    width_delta: i16,
) {
    let clear_row = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(2),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    let row = Rect {
        width: if width_delta < 0 {
            area.width
                .saturating_sub(2)
                .saturating_sub(width_delta.unsigned_abs())
        } else {
            area.width
                .saturating_sub(2)
                .saturating_add(width_delta as u16)
        },
        ..clear_row
    };
    if clear_row.is_empty() || row.is_empty() {
        return;
    }
    let style = ui
        .style(Family::FIELD, Variant::DEFAULT, Part::HELP, flags)
        .style;
    let mut fitted = truncate(message, row.width);
    while fitted.ends_with(' ') {
        fitted.pop();
    }
    let blank = " ".repeat(usize::from(clear_row.width));
    let _ = ui.paint_str(clear_row, &blank, style);
    let _ = ui.paint_str(row, &fitted, style);
}

#[derive(Debug)]
pub struct InputsPage {
    name: String,
    branch: String,
    name_state: TextInputState,
    branch_state: TextInputState,
    page_tabs: TabsState,
    focus_select_index: usize,
    focus_select_state: SelectState,
    blur_action_index: usize,
    blur_action_state: SelectState,
    plain_label: bool,
    required_message_index: usize,
    required_message_state: SelectState,
    options_scroll: ScrollState,
    options_focus: Option<Id>,
    states_scroll: ScrollState,
    last: &'static str,
}

impl InputsPage {
    pub fn new() -> Self {
        Self {
            name: String::from("payments-gateway"),
            branch: String::new(),
            name_state: TextInputState::default(),
            branch_state: TextInputState::default(),
            page_tabs: TabsState::default(),
            focus_select_index: 0,
            focus_select_state: SelectState::default(),
            blur_action_index: 0,
            blur_action_state: SelectState::default(),
            plain_label: false,
            required_message_index: 0,
            required_message_state: SelectState::default(),
            options_scroll: ScrollState::default(),
            options_focus: None,
            states_scroll: ScrollState::default(),
            last: "ready",
        }
    }

    fn focus_select_policy(&self) -> FocusSelect {
        focus_select_policy(self.focus_select_index)
    }

    fn section(&self) -> usize {
        match self.page_tabs.active().or(self.page_tabs.cursor()) {
            Some(ItemKey::Index(index)) => index.min(PAGE_SECTIONS.len().saturating_sub(1)),
            _ => 0,
        }
    }
}

impl Default for InputsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for InputsPage {
    fn title(&self) -> &'static str {
        "Inputs"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let old_section = self.section();
        let mut response = Tabs::new(PAGE_TABS)
            .update(cx, &mut self.page_tabs, PAGE_SECTIONS)
            .erase();
        let section = self.section();
        if section != old_section {
            match section {
                1 => self.options_scroll.scroll_to(0),
                2 => self.states_scroll.scroll_to(0),
                _ => (),
            }
        }

        response |= Select::new(FOCUS_SELECT)
            .popup_rows(3)
            .update_in_form(
                cx,
                &mut self.focus_select_state,
                &mut self.focus_select_index,
                FOCUS_SELECT_ITEMS,
                false,
            )
            .erase();
        response |= Select::new(BLUR_ACTION)
            .popup_rows(4)
            .update_in_form(
                cx,
                &mut self.blur_action_state,
                &mut self.blur_action_index,
                BLUR_ACTION_ITEMS,
                false,
            )
            .erase();
        response |= Select::new(REQUIRED_MESSAGE)
            .popup_rows(2)
            .update_in_form(
                cx,
                &mut self.required_message_state,
                &mut self.required_message_index,
                REQUIRED_MESSAGE_ITEMS,
                false,
            )
            .erase();
        response |= Toggle::new(PLAIN_LABEL, "Hide optional")
            .on(self.plain_label)
            .update(cx, &mut self.plain_label)
            .erase();
        response |=
            ScrollRegion::new(OPTIONS_SCROLL).update(cx, &mut self.options_scroll, OPTIONS_ROWS);
        response |= page_scroll_keys(cx, OPTIONS_SCROLL, &mut self.options_scroll);
        let focused_option = (section == 1)
            .then(|| cx.focused_in_scope(ScopeId::ROOT))
            .flatten()
            .filter(|id| options_control_row(*id).is_some());
        let focus_changed = focused_option != self.options_focus;
        self.options_focus = focused_option;
        let wheel_delta = cx
            .intents(OPTIONS_SCROLL)
            .filter_map(|intent| match intent {
                Intent::Wheel { delta, .. } => Some(i32::from(delta)),
                _ => None,
            })
            .fold(0i32, i32::saturating_add);
        if wheel_delta == 0
            && focus_changed
            && let Some(id) = focused_option
            && let Some(row) = options_control_row(id)
        {
            let _ = ScrollRegion::new(OPTIONS_SCROLL).prepare(
                cx,
                &mut self.options_scroll,
                OPTIONS_ROWS,
            );
            let _ = self.options_scroll.reveal_span(row, 1);
        }
        response |=
            ScrollRegion::new(STATES_SCROLL).update(cx, &mut self.states_scroll, STATE_ROWS);
        response |= page_scroll_keys(cx, STATES_SCROLL, &mut self.states_scroll);

        let focus_select = self.focus_select_policy();
        let blur = blur_policy(self.blur_action_index);
        let name = project_input(focus_select, blur, self.required_message_index).update(
            cx,
            &mut self.name_state,
            &mut self.name,
        );
        if let Some(action) = name.action_ref() {
            self.last = match action {
                TextAction::Committed => "name committed",
                TextAction::Cancelled => "name reverted",
                TextAction::Changed => "name draft changed",
                TextAction::MoveNext | TextAction::MovePrev => "name focus moved",
            };
        }
        response |= name.erase();
        let branch =
            branch_input(focus_select, blur).update(cx, &mut self.branch_state, &mut self.branch);
        if let Some(action) = branch.action_ref() {
            self.last = match action {
                TextAction::Committed => "branch committed",
                TextAction::Cancelled => "branch reverted",
                TextAction::Changed => "branch draft changed",
                TextAction::MoveNext | TextAction::MovePrev => "branch focus moved",
            };
        }
        response |= branch.erase();
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let meta = "Focus is a bar; editing is a cursor. Enter to edit, Esc to revert.";
        frame(ui, area, self.title(), meta, |ui, body| {
            if body.is_empty() {
                return;
            }

            match self.section() {
                0 => self.draw_playground(ui, body),
                1 => self.draw_options(ui, body),
                _ => self.draw_states(ui, body),
            }
            let tabs_area = Rect {
                y: body.y.saturating_sub(1),
                height: 1,
                ..body
            };
            let tabs = Tabs::new(PAGE_TABS);
            let previous =
                ui.layout(PAGE_TABS)
                    .map(|facts| termrock::navigation::tabs::TabWindowReceipt {
                        viewport_len: facts.viewport_len,
                        content_len: facts.content_len,
                    });
            let layout =
                tabs.layout_default(tabs_area, area, &self.page_tabs, PAGE_SECTIONS, previous);
            if tabs.draw_layout(ui, &layout).is_err() {
                let _ = tabs.draw(ui, tabs_area, &self.page_tabs, PAGE_SECTIONS);
            }
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(PAGE_TABS).contains(StateFlags::FOCUSED) {
            return TAB_HINTS;
        }
        if ui.state(FOCUS_SELECT).contains(StateFlags::FOCUSED)
            || ui.state(BLUR_ACTION).contains(StateFlags::FOCUSED)
            || ui.state(REQUIRED_MESSAGE).contains(StateFlags::FOCUSED)
        {
            return SELECT_HINTS;
        }
        if ui.state(PLAIN_LABEL).contains(StateFlags::FOCUSED) {
            return TOGGLE_HINTS;
        }
        if ui.state(OPTIONS_SCROLL).contains(StateFlags::FOCUSED)
            || ui.state(STATES_SCROLL).contains(StateFlags::FOCUSED)
        {
            return SCROLL_HINTS;
        }
        let editing = if ui.state(NAME).contains(StateFlags::FOCUSED) {
            self.name_state.is_editing()
        } else if ui.state(BRANCH).contains(StateFlags::FOCUSED) {
            self.branch_state.is_editing()
        } else {
            false
        };
        if editing {
            &[
                ("Enter", "Commit"),
                ("Esc", "Cancel"),
                ("Shift+← →", "Select"),
                ("Ctrl+U", "Clear"),
            ]
        } else {
            match self.section() {
                1 => SELECT_HINTS,
                2 => SCROLL_HINTS,
                _ => &[("Enter", "Edit")],
            }
        }
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.name_state.is_editing() || self.branch_state.is_editing()
    }
}

fn page_scroll_keys(cx: &mut Cx<'_>, id: Id, scroll: &mut ScrollState) -> termrock::Response<()> {
    let mut response = termrock::Response::ignored();
    for intent in cx.intents(id) {
        let Intent::Key(key) = intent else {
            continue;
        };
        let before = scroll.offset();
        match key.code {
            KeyCode::PageUp => scroll.page_up(),
            KeyCode::PageDown => scroll.page_down(),
            _ => continue,
        }
        response |= if scroll.offset() == before {
            termrock::Response::consumed()
        } else {
            termrock::Response::changed()
        };
    }
    response
}

impl InputsPage {
    fn draw_playground(&self, ui: &mut Ui<'_>, body: Rect) {
        if body.is_empty() {
            return;
        }
        let focus_select = self.focus_select_policy();
        let blur = blur_policy(self.blur_action_index);
        let regions = layout::rows(body, &[Track::Fixed(17), Track::Fixed(1), Track::Flex(1)]);
        let fields = regions.first().copied().unwrap_or(body);
        let small = body.width < 70;
        playground_panel().draw(ui, fields, |ui, inner| {
            let columns = layout::columns(inner, &[Track::Flex(1), Track::Flex(1)], 3);
            let left = columns.first().copied().unwrap_or(inner);
            let right = columns.get(1).copied().unwrap_or(inner);
            let (left, right) = if small {
                (
                    Rect {
                        width: left.width.saturating_sub(1),
                        ..left
                    },
                    Rect {
                        x: right.x.saturating_sub(1),
                        width: right.width.saturating_add(1),
                        ..right
                    },
                )
            } else {
                (left, right)
            };
            let left_rows = layout::rows(left, &[Track::Fixed(3), Track::Fixed(3), Track::Flex(1)]);
            let right_rows =
                layout::rows(right, &[Track::Fixed(3), Track::Fixed(3), Track::Flex(1)]);
            let project_area = left_rows.first().copied().unwrap_or(inner);
            let name_error = self.name_state.error().map(|error| error.message.as_ref());
            project_field(
                &self.name,
                focus_select,
                blur,
                self.required_message_index,
                name_error,
            )
            .draw(ui, project_area, &self.name_state);
            legacy_field_gutter(
                ui,
                Rect {
                    y: project_area.y.saturating_add(1),
                    ..project_area
                },
                ui.state(NAME),
            );
            if name_error.is_none() {
                legacy_field_help(
                    ui,
                    project_area,
                    ui.state(NAME),
                    "Used as the working directory name",
                    -1,
                );
            }
            let branch_area = right_rows.first().copied().unwrap_or(inner);
            branch_field(&self.branch, focus_select, blur, self.plain_label).draw(
                ui,
                branch_area,
                &self.branch_state,
            );
            legacy_field_gutter(
                ui,
                Rect {
                    y: branch_area.y.saturating_add(1),
                    ..branch_area
                },
                ui.state(BRANCH),
            );
            legacy_field_help(
                ui,
                branch_area,
                ui.state(BRANCH),
                "Leave empty to work on a detached checkout",
                i16::from(body.width < 70),
            );
            Self::draw_reference_fields(ui, inner, &left_rows, &right_rows, body);
        });
        if let Some(select_area) = regions.get(1).copied().filter(|area| !area.is_empty()) {
            let _ = Select::new(FOCUS_SELECT).popup_rows(3).draw(
                ui,
                select_area,
                &self.focus_select_state,
                FOCUS_SELECT_ITEMS,
            );
        }
        if let Some(reference_area) = regions.get(2).copied() {
            state_reference_panel().draw(ui, reference_area, |ui, inner| {
                Self::draw_state_reference(ui, inner);
            });
        }
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let view =
            ScrollRegion::new(OPTIONS_SCROLL).draw(ui, area, &self.options_scroll, OPTIONS_ROWS);
        if view.is_empty() {
            return;
        }
        ui.register_focus_only(OPTIONS_SCROLL, Focusability::Focusable);
        let projected = ScrollRegion::view(&self.options_scroll, view, OPTIONS_ROWS);
        let first_row = projected.offset().min(OPTIONS_ROWS);
        let logical_height = u16::try_from(OPTIONS_ROWS).unwrap_or(u16::MAX);
        let visible = Rect {
            height: view.height.min(logical_height),
            ..view
        };
        let Ok(window) = VerticalWindow::new(visible, OPTIONS_ROWS, first_row) else {
            return;
        };
        let source_rows = window.visible_source_range();
        let label_width = view.width.saturating_sub(14).min(22);
        let control_x = view.x.saturating_add(label_width).saturating_add(1);
        let control_width = view.right().saturating_sub(control_x);
        let rows = [
            "Focus selects",
            "On blur",
            "Plain label",
            "Required message",
        ];
        for (source_y, label) in rows.into_iter().enumerate() {
            if source_y < source_rows.start || source_y >= source_rows.end {
                if control_width > 0 {
                    ui.register_focus_only(OPTIONS_CONTROLS[source_y], Focusability::Focusable);
                }
                continue;
            }
            let Some(y) = window.physical_row_for_source(source_y) else {
                continue;
            };
            if label_width > 0 {
                let _ = ui.paint_str(
                    Rect::new(view.x, y, label_width, 1),
                    label,
                    ui.surface_style(),
                );
            }
            let control_area = Rect::new(control_x, y, control_width, 1);
            if control_area.is_empty() {
                continue;
            }
            match source_y {
                0 => {
                    let _ = Select::new(FOCUS_SELECT).popup_rows(3).draw(
                        ui,
                        control_area,
                        &self.focus_select_state,
                        FOCUS_SELECT_ITEMS,
                    );
                }
                1 => {
                    let _ = Select::new(BLUR_ACTION).popup_rows(4).draw(
                        ui,
                        control_area,
                        &self.blur_action_state,
                        BLUR_ACTION_ITEMS,
                    );
                }
                2 => {
                    Toggle::new(PLAIN_LABEL, "Hide optional")
                        .on(self.plain_label)
                        .draw(ui, control_area);
                }
                3 => {
                    let _ = Select::new(REQUIRED_MESSAGE).popup_rows(2).draw(
                        ui,
                        control_area,
                        &self.required_message_state,
                        REQUIRED_MESSAGE_ITEMS,
                    );
                }
                _ => (),
            }
        }
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let view = ScrollRegion::new(STATES_SCROLL).draw(ui, area, &self.states_scroll, STATE_ROWS);
        if view.is_empty() {
            return;
        }
        ui.register_focus_only(STATES_SCROLL, Focusability::Focusable);
        let projected = ScrollRegion::view(&self.states_scroll, view, STATE_ROWS);
        let first_row = projected.offset().min(STATE_ROWS);
        let logical_height = u16::try_from(STATE_ROWS).unwrap_or(u16::MAX);
        let visible = Rect {
            height: view.height.min(logical_height),
            ..view
        };
        let Ok(window) = VerticalWindow::new(visible, STATE_ROWS, first_row) else {
            return;
        };
        let source_rows = window.visible_source_range();
        for source_y in source_rows {
            let Some(y) = window.physical_row_for_source(source_y) else {
                continue;
            };
            Self::draw_state_sample(
                ui,
                Rect {
                    y,
                    height: 1,
                    ..view
                },
                source_y,
            );
        }
    }

    fn draw_state_sample(ui: &mut Ui<'_>, row: Rect, index: usize) {
        let (label, value, placeholder, status, disabled) = match index {
            0 => ("default", "payments-gateway", None, Status::Ready, false),
            1 => ("placeholder", "", Some("feat/…"), Status::Ready, false),
            2 => ("hover", "payments-gateway", None, Status::Ready, false),
            3 => ("focused", "payments-gateway", None, Status::Ready, false),
            4 => ("editing", "payments-gateway", None, Status::Ready, false),
            5 => ("error", "mira@example", None, Status::Error, false),
            6 => ("error + focus", "mira@example", None, Status::Error, false),
            7 => ("disabled", "jb_live_••••", None, Status::Ready, true),
            8 => ("empty", "", None, Status::Ready, false),
            9 => (
                "overflow",
                "payments-gateway-name-that-does-not-fit",
                None,
                Status::Ready,
                false,
            ),
            _ => ("narrow", "payments-gateway", None, Status::Ready, false),
        };
        let _ = ui.paint_str(row, label, ui.surface_style());
        let field_x = row.x.saturating_add(16);
        let mut field_width = row.width.saturating_sub(16).min(33);
        if index == 9 {
            field_width = field_width.min(18);
        } else if index == 10 {
            field_width = field_width.min(10);
        }
        let field_area = Rect::new(field_x, row.y, field_width, row.height);
        let id = STATE_REFERENCE.index(index);
        let reference_state = match index {
            2 => Some(ReferenceState::HOVERED),
            3 | 4 | 6 => Some(ReferenceState::FOCUSED),
            _ => None,
        };
        let target = reference_state.map(|state| ReferenceTarget::new(id, state));
        let state = state_sample_input_state(index, value);
        ui.reference(target, |ui| {
            let mut input = TextInput::new(id)
                .value(value)
                .status(status)
                .disabled(disabled);
            if let Some(placeholder) = placeholder {
                input = input.placeholder(placeholder);
            }
            let _ = input.draw(ui, field_area, &state);
        });
    }
}

fn state_sample_input_state(index: usize, value: &str) -> TextInputState {
    let mut state = TextInputState::default();
    match index {
        4 => state.begin(value),
        5 | 6 => state.set_error(Some(FieldError::new("Enter a valid project name"))),
        _ => (),
    }
    state
}

impl InputsPage {
    fn draw_reference_fields(
        ui: &mut Ui<'_>,
        inner: Rect,
        left_rows: &[Rect],
        right_rows: &[Rect],
        body: Rect,
    ) {
        ui.reference(None, |ui| {
            let mut owner_state = TextInputState::default();
            owner_state.set_error(Some(FieldError::new("Enter a valid email address")));
            let owner_area = {
                let area = left_rows.get(1).copied().unwrap_or(inner);
                Rect {
                    width: area.width.saturating_sub(2),
                    ..area
                }
            };
            owner_field().draw(ui, owner_area, &owner_state);
            legacy_field_gutter(
                ui,
                Rect {
                    y: owner_area.y.saturating_add(1),
                    ..owner_area
                },
                StateFlags::ERROR,
            );
            legacy_field_help(
                ui,
                owner_area,
                StateFlags::ERROR,
                "Enter a valid email address",
                1,
            );
            let token_area = right_rows.get(1).copied().unwrap_or(inner);
            token_field().draw(ui, token_area, &TextInputState::default());
            legacy_field_gutter(
                ui,
                Rect {
                    y: token_area.y.saturating_add(1),
                    ..token_area
                },
                StateFlags::DISABLED,
            );
            legacy_field_help(
                ui,
                token_area,
                StateFlags::DISABLED,
                "Managed by the organization",
                i16::from(body.width < 70),
            );
            let search_area = left_rows.get(2).copied().unwrap_or(inner);
            search_field().draw(ui, search_area, &TextInputState::default());
            legacy_field_gutter(
                ui,
                Rect {
                    y: search_area.y.saturating_add(1),
                    ..search_area
                },
                StateFlags::empty(),
            );
            legacy_field_help(
                ui,
                search_area,
                StateFlags::empty(),
                "Selection: Shift+← →  ·  words: Ctrl+← →  ·  clear: Ctrl+U",
                -1,
            );
            let api_key_area = right_rows.get(2).copied().unwrap_or(inner);
            api_key_field().draw(ui, api_key_area, &TextInputState::default());
            legacy_field_gutter(
                ui,
                Rect {
                    y: api_key_area.y.saturating_add(1),
                    ..api_key_area
                },
                StateFlags::empty(),
            );
            legacy_field_help(
                ui,
                api_key_area,
                StateFlags::empty(),
                "Masked while typing; the last four characters show once committed",
                i16::from(body.width < 70),
            );
        });
    }
}

impl InputsPage {
    fn draw_state_reference(ui: &mut Ui<'_>, inner: Rect) {
        let states = [
            ("default", "payments-gateway", Status::Ready),
            ("placeholder", "(feat/…)", Status::Ready),
            ("hover", "payments-gateway", Status::Ready),
            ("focused", "payments-gateway", Status::Ready),
            ("editing", "payments-gateway", Status::Ready),
            ("error", "mira@example", Status::Error),
            ("error + focus", "mira@example", Status::Error),
            ("disabled", "jb_live_••••", Status::Ready),
        ];
        for (index, (label, value, status)) in states.iter().enumerate() {
            let Ok(offset) = u16::try_from(index) else {
                break;
            };
            if offset >= inner.height {
                break;
            }
            let row = Rect {
                y: inner.y.saturating_add(offset),
                height: 1,
                ..inner
            };
            let _ = ui.paint_str(row, label, ui.surface_style());
            let field_area = Rect {
                x: row.x.saturating_add(16),
                width: row.width.saturating_sub(16).min(33),
                ..row
            };
            let mut flags = StateFlags::empty();
            if matches!(*status, Status::Error) {
                flags |= StateFlags::ERROR;
            }
            if matches!(index, 3 | 4 | 6) {
                flags |= StateFlags::FOCUSED;
            }
            ui.reference(None, |ui| {
                TextInput::new(STATE_REFERENCE.index(index))
                    .value(value)
                    .status(*status)
                    .draw(ui, field_area, &TextInputState::default());
                legacy_field_gutter(ui, field_area, flags);
            });
        }
    }
}
