//! Single-line controlled editing with commit, cancel and validation feedback.

use junie_tui::{
    BlurPolicy, Cx, Family, Field, FieldError, FrameRead, Id, Panel, PanelKind, Part, Rect,
    Response, StateFlags, Status, TextAction, TextInput, TextInputState, Track, Ui, Variant, id,
    layout, truncate,
};

use super::{Page, PageUpdate, frame};

const NAME: Id = id!("inputs.name");
const BRANCH: Id = id!("inputs.branch");
const CARD: Id = id!("inputs.card");
const OWNER: Id = id!("inputs.owner");
const TOKEN: Id = id!("inputs.token");
const SEARCH: Id = id!("inputs.search");
const API_KEY: Id = id!("inputs.api_key");
const STATE_REFERENCE: Id = id!("inputs.state_reference");

fn email(value: &str) -> Result<(), FieldError> {
    if value.contains('@') && value.contains('.') {
        Ok(())
    } else {
        Err(FieldError::new("Enter a valid email address"))
    }
}

fn name_input<'a>() -> TextInput<'a> {
    TextInput::new(NAME).blur(BlurPolicy::CommitAndValidate)
}

fn branch_input<'a>() -> TextInput<'a> {
    TextInput::new(BRANCH)
        .placeholder("feat/…")
        .blur(BlurPolicy::Commit)
}

/// The one card constructor for this page (§13): both phase paths build the
/// same `Panel::new(CARD)` props and vary only the heading text on top.
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

fn project_field(value: &str) -> Field<'_, TextInput<'_>> {
    Field::new("Project name", name_input().value(value))
        .required(true)
        .help("Used as the working directory name")
}

fn branch_field(value: &str) -> Field<'_, TextInput<'_>> {
    Field::new("Branch", branch_input().value(value))
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

/// A pair of independent controlled fields, matching the legacy input page.
#[derive(Debug)]
pub(crate) struct InputsPage {
    name: String,
    branch: String,
    name_state: TextInputState,
    branch_state: TextInputState,
    last: &'static str,
}

impl InputsPage {
    pub(crate) fn new() -> Self {
        Self {
            name: String::from("payments-gateway"),
            branch: String::new(),
            name_state: TextInputState::default(),
            branch_state: TextInputState::default(),
            last: "ready",
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
        let mut response = Response::ignored();
        // Both phases build the same card and reference-field props (§13); the
        // draw pass paints the reference fields inside a frozen state scope.
        let _ = fields_panel();
        let _ = playground_panel();
        let _ = state_reference_panel();
        let _ = owner_field();
        let _ = token_field();
        let _ = search_field();
        let _ = api_key_field();
        let name = name_input().update(cx, &mut self.name_state, &mut self.name);
        if let Some(action) = name.action_ref() {
            self.last = match action {
                TextAction::Committed => "name committed",
                TextAction::Cancelled => "name reverted",
                TextAction::Changed => "name draft changed",
                TextAction::MoveNext | TextAction::MovePrev => "name focus moved",
            };
        }
        response |= name.erase();
        let branch = branch_input().update(cx, &mut self.branch_state, &mut self.branch);
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
                project_field(&self.name).draw(ui, project_area, &self.name_state);
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
                branch_field(&self.branch).draw(ui, branch_area, &self.branch_state);
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
