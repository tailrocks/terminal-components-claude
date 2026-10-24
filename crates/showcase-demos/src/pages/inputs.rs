use termrock::{
    BlurPolicy, Cx, Family, Field, FieldError, FocusSelect, FrameRead, Id, Panel, PanelKind, Part,
    Rect, Select, SelectState, StateFlags, Status, TextAction, TextInput, TextInputState, Track,
    Ui, Variant, layout, truncate,
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

const FOCUS_SELECT_ITEMS: &[&str] = &[
    "Focus selects: Select all",
    "Focus selects: Caret end",
    "Focus selects: Caret start",
];

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

fn name_input<'a>(policy: FocusSelect) -> TextInput<'a> {
    TextInput::new(NAME)
        .focus_select(policy)
        .blur(BlurPolicy::CommitAndValidate)
}

fn branch_input<'a>(policy: FocusSelect) -> TextInput<'a> {
    TextInput::new(BRANCH)
        .focus_select(policy)
        .placeholder("feat/…")
        .blur(BlurPolicy::Commit)
}

fn card_panel() -> Panel<'static> {
    Panel::new(CARD).kind(PanelKind::Card)
}

fn fields_panel() -> Panel<'static> {
    card_panel().title("Edit fields")
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

fn project_field(value: &str, policy: FocusSelect) -> Field<'_, TextInput<'_>> {
    Field::new("Project name", name_input(policy).value(value))
        .required(true)
        .help("Used as the working directory name")
}

fn branch_field(value: &str, policy: FocusSelect) -> Field<'_, TextInput<'_>> {
    Field::new("Branch", branch_input(policy).value(value))
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
    focus_select_index: usize,
    focus_select_state: SelectState,
    last: &'static str,
}

impl InputsPage {
    pub fn new() -> Self {
        Self {
            name: String::from("payments-gateway"),
            branch: String::new(),
            name_state: TextInputState::default(),
            branch_state: TextInputState::default(),
            focus_select_index: 0,
            focus_select_state: SelectState::default(),
            last: "ready",
        }
    }

    fn focus_select_policy(&self) -> FocusSelect {
        focus_select_policy(self.focus_select_index)
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
        let mut response = Select::new(FOCUS_SELECT)
            .popup_rows(3)
            .update_in_form(
                cx,
                &mut self.focus_select_state,
                &mut self.focus_select_index,
                FOCUS_SELECT_ITEMS,
                false,
            )
            .erase();
        let focus_select = self.focus_select_policy();

        let _ = fields_panel();
        let _ = playground_panel();
        let _ = state_reference_panel();
        let _ = owner_field();
        let _ = token_field();
        let _ = search_field();
        let _ = api_key_field();
        let name = name_input(focus_select).update(cx, &mut self.name_state, &mut self.name);
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
            branch_input(focus_select).update(cx, &mut self.branch_state, &mut self.branch);
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
        let focus_select = self.focus_select_policy();
        frame(ui, area, self.title(), meta, |ui, body| {
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
                let left_rows =
                    layout::rows(left, &[Track::Fixed(3), Track::Fixed(3), Track::Flex(1)]);
                let right_rows =
                    layout::rows(right, &[Track::Fixed(3), Track::Fixed(3), Track::Flex(1)]);
                let project_area = left_rows.first().copied().unwrap_or(inner);
                project_field(&self.name, focus_select).draw(ui, project_area, &self.name_state);
                legacy_field_gutter(
                    ui,
                    Rect {
                        y: project_area.y.saturating_add(1),
                        ..project_area
                    },
                    ui.state(NAME),
                );
                legacy_field_help(
                    ui,
                    project_area,
                    ui.state(NAME),
                    "Used as the working directory name",
                    -1,
                );
                let branch_area = right_rows.first().copied().unwrap_or(inner);
                branch_field(&self.branch, focus_select).draw(ui, branch_area, &self.branch_state);
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
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
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
            &[("Enter", "Edit")]
        }
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.name_state.is_editing() || self.branch_state.is_editing()
    }
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

#[cfg(test)]
mod focus_select_tests {
    use super::*;
    use termrock::{App, Button, ItemKey, KeyCode, PartRef, Response, Theme};
    use termrock_test_support::Harness;

    const OWNER_PEER: Id =
        termrock::Id::root("showcase_ui::pages::inputs.focus_select_tests.owner_peer");
    const INPUTS_AREA: Rect = Rect::new(0, 0, 100, 35);
    const PEER_AREA: Rect = Rect::new(104, 2, 16, 1);

    struct Host {
        page: InputsPage,
        area: Option<Rect>,
        paint_sentinels: bool,
        inputs_active: bool,
        persistent_peer: bool,
        inputs_updates: usize,
        peer_activations: usize,
    }

    impl App for Host {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let mut response = Response::ignored();
            if self.persistent_peer || !self.inputs_active {
                let peer_response = Button::new(OWNER_PEER, "Replacement page").update(cx);
                if peer_response.activated() {
                    self.peer_activations = self.peer_activations.saturating_add(1);
                    if self.persistent_peer {
                        self.inputs_active = true;
                    }
                }
                response |= peer_response.erase();
            }
            if self.inputs_active {
                self.inputs_updates = self.inputs_updates.saturating_add(1);
                response |= self.page.update(cx).response;
            }
            response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let full = ui.full();
            if self.paint_sentinels {
                let row = "S".repeat(usize::from(full.width));
                for y in full.y..full.bottom() {
                    let _ = ui.paint_str(
                        Rect {
                            y,
                            height: 1,
                            ..full
                        },
                        &row,
                        ui.surface_style(),
                    );
                }
            }
            if self.inputs_active {
                self.page.draw(ui, self.area.unwrap_or(full));
            }
            if self.persistent_peer || !self.inputs_active {
                Button::new(OWNER_PEER, "Replacement page").draw(ui, PEER_AREA);
            }
        }
    }

    fn host(width: u16, height: u16) -> Harness<Host> {
        Harness::new(
            Host {
                page: InputsPage::new(),
                area: None,
                paint_sentinels: false,
                inputs_active: true,
                persistent_peer: false,
                inputs_updates: 0,
                peer_activations: 0,
            },
            Theme::junie(),
            width,
            height,
        )
    }

    fn offset_host(width: u16, height: u16, area: Rect) -> Harness<Host> {
        Harness::new(
            Host {
                page: InputsPage::new(),
                area: Some(area),
                paint_sentinels: true,
                inputs_active: true,
                persistent_peer: false,
                inputs_updates: 0,
                peer_activations: 0,
            },
            Theme::junie(),
            width,
            height,
        )
    }

    fn choose_by_keyboard(h: &mut Harness<Host>, index: usize) {
        assert!(h.tab_to(FOCUS_SELECT), "focus the Inputs policy selector");
        let _ = h.key(KeyCode::Enter);
        for _ in 0..index {
            let _ = h.key(KeyCode::Down);
        }
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().page.focus_select_index, index);
    }

    fn choose_by_pointer(h: &mut Harness<Host>, index: usize) {
        let _ = h.click_part(FOCUS_SELECT, PartRef::of(Part::FIELD));
        assert!(h.is_open(FOCUS_SELECT), "the real Select popup opens");
        let _ = h.click_part(
            FOCUS_SELECT,
            PartRef::item(Part::ROW, ItemKey::index(index)),
        );
        assert_eq!(h.app().page.focus_select_index, index);
        assert!(!h.is_open(FOCUS_SELECT), "choosing closes the Select popup");
    }

    fn assert_sentinels_outside(h: &Harness<Host>, area: Rect, width: u16, height: u16) {
        let sentinel = h.cell(0, 0);
        assert_eq!(sentinel.symbol(), "S", "the underlay sentinel is present");
        for y in 0..height {
            for x in 0..width {
                if x < area.x || x >= area.right() || y < area.y || y >= area.bottom() {
                    assert_eq!(h.cell(x, y), sentinel, "outside page at ({x}, {y})");
                }
            }
        }
    }

    #[test]
    fn inputs_defaults_to_select_all_while_pointer_focus_remains_caret_driven() {
        let mut h = host(120, 40);
        assert_eq!(h.app().page.focus_select_index, 0);
        assert_eq!(
            h.app().page.focus_select_state.value(),
            Some(ItemKey::index(0))
        );
        assert!(h.tab_to(NAME));
        assert_eq!(h.focus(), Some(NAME));
        assert_eq!(
            h.app().page.name_state.selection(),
            Some(0.."payments-gateway".len())
        );
        let _ = h.type_str("x");
        assert_eq!(h.app().page.name_state.draft_text(), Some("x"));
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().page.name, "x");

        let mut pointer = host(120, 40);
        let name_text = pointer
            .area_of_part(NAME, PartRef::of(Part::TEXT))
            .expect("the visible NAME text has its mandatory text part");
        assert!(name_text.width > 1, "the NAME text has a second cell");
        assert!(name_text.height > 0, "the NAME text has a visible row");
        let _ = pointer.click(name_text.x + 1, name_text.y);
        assert_eq!(pointer.focus(), Some(NAME));
        assert!(
            pointer
                .state_of(NAME)
                .contains(StateFlags::FOCUSED | StateFlags::EDITING)
        );
        assert!(pointer.app().page.name_state.is_editing());
        assert_eq!(pointer.app().page.name_state.selection(), None);
        assert_eq!(pointer.app().page.name_state.cursor_offset(), Some(1));
        assert_eq!(
            pointer.app().page.name_state.draft_text(),
            Some("payments-gateway")
        );
        assert_eq!(pointer.app().page.name, "payments-gateway");
        assert!(pointer.diagnostics().is_empty());
    }

    #[test]
    fn keyboard_caret_end_policy_applies_to_a_fresh_name_session() {
        let mut h = host(120, 40);
        choose_by_keyboard(&mut h, 1);
        assert_eq!(
            h.app().page.focus_select_state.value(),
            Some(ItemKey::index(1))
        );

        assert!(h.tab_to(NAME));
        assert_eq!(h.app().page.name_state.selection(), None);
        assert_eq!(
            h.app().page.name_state.cursor_offset(),
            Some("payments-gateway".len())
        );
        let _ = h.type_str("x");
        assert_eq!(
            h.app().page.name_state.draft_text(),
            Some("payments-gatewayx")
        );
    }

    #[test]
    fn pointer_caret_start_policy_keeps_branch_data_and_applies_to_a_fresh_session() {
        let mut h = host(120, 40);
        assert!(h.tab_to(BRANCH));
        let _ = h.type_str("topic");
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().page.branch, "topic");

        choose_by_pointer(&mut h, 2);
        assert_eq!(h.app().page.name, "payments-gateway");
        assert_eq!(h.app().page.branch, "topic");

        assert!(h.tab_to(BRANCH));
        assert_eq!(h.app().page.branch_state.selection(), None);
        assert_eq!(h.app().page.branch_state.cursor_offset(), Some(0));
        let _ = h.type_str("x");
        assert_eq!(h.app().page.branch_state.draft_text(), Some("xtopic"));
    }

    #[test]
    fn selector_stays_in_its_page_row_and_keeps_policy_when_the_row_is_absent() {
        const WIDTH: u16 = 120;
        const HEIGHT: u16 = 40;
        let page_area = Rect::new(7, 5, 104, 29);
        let mut h = offset_host(WIDTH, HEIGHT, page_area);
        let selector = h.area_of(FOCUS_SELECT).expect("selector row is registered");
        let selector_field = PartRef::of(Part::FIELD);
        assert!(h.area_of_part(FOCUS_SELECT, selector_field).is_some());
        assert!(h.ring().is_registered(FOCUS_SELECT));
        assert!(h.ring().contains(FOCUS_SELECT));
        assert_eq!(selector.height, 1);
        assert!(selector.x >= page_area.x && selector.y >= page_area.y);
        assert!(selector.right() <= page_area.right());
        assert!(selector.bottom() <= page_area.bottom());
        assert_sentinels_outside(&h, page_area, WIDTH, HEIGHT);

        choose_by_pointer(&mut h, 2);
        let chosen = h.app().page.focus_select_index;
        let short_area = Rect::new(61, 25, 45, 9);
        h.app_mut().area = Some(short_area);
        h.draw();
        assert!(h.area_of(FOCUS_SELECT).is_none());
        assert!(h.area_of_part(FOCUS_SELECT, selector_field).is_none());
        assert!(!h.ring().is_registered(FOCUS_SELECT));
        assert!(!h.ring().contains(FOCUS_SELECT));
        assert_eq!(h.app().page.focus_select_index, chosen);
        assert_eq!(
            h.app().page.focus_select_state.value(),
            Some(ItemKey::index(chosen))
        );
        assert_sentinels_outside(&h, short_area, WIDTH, HEIGHT);

        h.app_mut().area = Some(page_area);
        h.draw();
        assert!(h.area_of(FOCUS_SELECT).is_some());
        assert!(h.area_of_part(FOCUS_SELECT, selector_field).is_some());
        assert!(h.ring().is_registered(FOCUS_SELECT));
        assert_eq!(h.app().page.focus_select_index, chosen);
    }

    #[test]
    fn inactive_inputs_owner_closes_open_selector_and_reconciles_on_return() {
        // This host models an inactive page owner, not Showcase navigation itself.
        let mut h = Harness::new(
            Host {
                page: InputsPage::new(),
                area: Some(INPUTS_AREA),
                paint_sentinels: false,
                inputs_active: true,
                persistent_peer: true,
                inputs_updates: 0,
                peer_activations: 0,
            },
            Theme::junie(),
            120,
            40,
        );

        choose_by_keyboard(&mut h, 1);
        assert!(h.tab_to(BRANCH));
        let _ = h.type_str("saved-branch");
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().page.branch, "saved-branch");
        assert!(!h.app().page.branch_state.is_editing());

        let committed_policy = h.app().page.focus_select_index;
        let committed_select_value = h.app().page.focus_select_state.value();
        let committed_name = h.app().page.name.clone();
        let committed_branch = h.app().page.branch.clone();
        let name_state = h.app().page.name_state.clone();
        let branch_state = h.app().page.branch_state.clone();
        assert_eq!(committed_policy, 1);
        assert_eq!(committed_select_value, Some(ItemKey::index(1)));

        let _ = h.click_part(FOCUS_SELECT, PartRef::of(Part::FIELD));
        assert!(h.app().page.focus_select_state.is_open());
        assert!(h.is_open(FOCUS_SELECT));
        let _ = h.key(KeyCode::Down);
        assert_eq!(
            h.app().page.focus_select_state.cursor(),
            Some(ItemKey::index(2))
        );
        assert_eq!(h.app().page.focus_select_index, committed_policy);
        assert_eq!(
            h.app().page.focus_select_state.value(),
            committed_select_value
        );
        let popup_row = h
            .area_of_part(FOCUS_SELECT, PartRef::item(Part::ROW, ItemKey::index(2)))
            .expect("the open popup publishes its CaretStart row hit");
        assert!(!popup_row.is_empty());
        let popup_area = h
            .layer_area(FOCUS_SELECT)
            .expect("the open selector has a live popup layer");
        assert!(popup_area.right() <= PEER_AREA.x || popup_area.x >= PEER_AREA.right());
        assert!(h.diagnostics().is_empty());
        assert_eq!(h.runtime().capture_owner(), None);

        let open_select_state = h.app().page.focus_select_state.clone();
        let active_update_count = h.app().inputs_updates;
        h.app_mut().inputs_active = false;
        // Harness::draw performs the ordinary prepare/draw/ack path; prepare settles focus work.
        h.draw();

        assert_eq!(h.runtime().capture_owner(), None);
        assert_eq!(h.app().inputs_updates, active_update_count);
        assert_eq!(h.app().page.focus_select_state, open_select_state);
        assert_eq!(h.app().page.name_state, name_state);
        assert_eq!(h.app().page.branch_state, branch_state);
        assert_eq!(h.app().page.name, committed_name);
        assert_eq!(h.app().page.branch, committed_branch);
        assert!(!h.is_open(FOCUS_SELECT));
        assert!(h.layer_area(FOCUS_SELECT).is_none());
        assert!(h.area_of(FOCUS_SELECT).is_none());
        assert!(
            h.area_of_part(FOCUS_SELECT, PartRef::of(Part::FIELD))
                .is_none()
        );
        assert!(
            h.area_of_part(FOCUS_SELECT, PartRef::item(Part::ROW, ItemKey::index(2)),)
                .is_none()
        );
        assert!(!h.ring().is_registered(FOCUS_SELECT));
        assert!(!h.ring().contains(FOCUS_SELECT));
        assert_ne!(h.focus(), Some(FOCUS_SELECT));
        assert!(h.find("Focus selects: Caret start").is_none());
        let blank = h.cell(119, 39).clone();
        for y in popup_area.y..popup_area.bottom() {
            for x in popup_area.x..popup_area.right() {
                assert_eq!(h.cell(x, y), &blank, "former popup ink at ({x}, {y})");
            }
        }
        let peer = h.area_of(OWNER_PEER).expect("replacement page is drawn");
        assert!(peer.x >= INPUTS_AREA.right());
        assert!(
            h.area_of_part(OWNER_PEER, PartRef::of(Part::CONTAINER))
                .is_some()
        );
        assert!(h.ring().is_registered(OWNER_PEER));
        assert!(h.ring().contains(OWNER_PEER));
        assert!(h.tab_to(OWNER_PEER));
        assert_eq!(h.focus(), Some(OWNER_PEER));
        assert!(!h.app().inputs_active);
        assert_eq!(h.app().inputs_updates, active_update_count);
        let _ = h.click_part(OWNER_PEER, PartRef::of(Part::CONTAINER));
        assert_eq!(h.app().peer_activations, 1);
        assert!(
            h.app().inputs_active,
            "the native peer activation routes back"
        );
        assert!(
            h.app().inputs_updates > active_update_count,
            "the newly active Inputs page receives the same ordinary update pass"
        );
        assert!(h.diagnostics().is_empty());

        assert_eq!(h.runtime().capture_owner(), None);
        assert!(!h.app().page.focus_select_state.is_open());
        assert!(!h.is_open(FOCUS_SELECT));
        assert_eq!(
            h.app().page.focus_select_state.cursor(),
            Some(ItemKey::index(1)),
            "dismissal restores the cursor to the committed CaretEnd value"
        );
        assert_eq!(h.app().page.focus_select_index, committed_policy);
        assert_eq!(
            h.app().page.focus_select_state.value(),
            committed_select_value
        );
        assert_eq!(h.app().page.name, committed_name);
        assert_eq!(h.app().page.branch, committed_branch);
        assert!(h.area_of(FOCUS_SELECT).is_some());
        assert!(
            h.area_of_part(FOCUS_SELECT, PartRef::of(Part::FIELD))
                .is_some()
        );
        let peer = h
            .area_of(OWNER_PEER)
            .expect("persistent shell peer remains drawn on return");
        assert!(peer.x >= INPUTS_AREA.right());
        assert!(
            h.area_of_part(OWNER_PEER, PartRef::of(Part::CONTAINER))
                .is_some()
        );
        assert!(h.ring().is_registered(OWNER_PEER));
        assert!(h.ring().contains(OWNER_PEER));
        assert!(h.ring().is_registered(FOCUS_SELECT));
        assert!(h.ring().contains(FOCUS_SELECT));
        assert!(
            h.area_of_part(FOCUS_SELECT, PartRef::item(Part::ROW, ItemKey::index(2)),)
                .is_none()
        );
        assert!(
            h.diagnostics().is_empty(),
            "Inputs owner return diagnostics: {:?}",
            h.diagnostics()
        );

        assert_eq!(h.focus(), Some(OWNER_PEER));
        let _ = h.click_part(FOCUS_SELECT, PartRef::of(Part::FIELD));
        assert!(h.is_open(FOCUS_SELECT));
        assert_eq!(h.focus(), Some(FOCUS_SELECT));
        assert!(h.ring().is_registered(OWNER_PEER));
        assert_eq!(h.app().peer_activations, 1);
        assert_eq!(h.runtime().capture_owner(), None);
        assert!(h.diagnostics().is_empty());
        let _ = h.click_part(FOCUS_SELECT, PartRef::item(Part::ROW, ItemKey::index(2)));
        assert_eq!(h.app().page.focus_select_index, 2);
        assert!(!h.is_open(FOCUS_SELECT));
        assert!(h.tab_to(BRANCH));
        assert_eq!(h.app().page.branch_state.selection(), None);
        assert_eq!(h.app().page.branch_state.cursor_offset(), Some(0));
        let _ = h.type_str("x");
        assert_eq!(
            h.app().page.branch_state.draft_text(),
            Some("xsaved-branch")
        );
        assert_eq!(h.app().page.branch, committed_branch);
        assert_eq!(h.app().peer_activations, 1);
        assert_eq!(h.runtime().capture_owner(), None);
        assert!(h.diagnostics().is_empty());
    }
}
