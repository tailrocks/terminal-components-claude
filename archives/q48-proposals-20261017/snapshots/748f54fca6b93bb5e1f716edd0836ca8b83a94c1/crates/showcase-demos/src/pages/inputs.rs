//! Single-line controlled editing with commit, cancel and validation feedback.

use termrock::{
    BlurPolicy, Cx, Family, FgStep, Field, FieldError, FrameRead, Id, Modifier, Panel, PanelKind,
    Part, Rect, Response, Role, SecretPolicy, StateFlags, StylePatch, TextAction, TextInput,
    TextInputState, Track, Ui, Variant, id, layout,
};

use super::{Page, PageStatus, PageUpdate, frame};

const NAME: Id = id!("inputs.name");
const BRANCH: Id = id!("inputs.branch");
const CARD: Id = id!("inputs.card");
const OWNER: Id = id!("inputs.owner");
const TOKEN: Id = id!("inputs.token");
const SEARCH: Id = id!("inputs.search");
const API_KEY: Id = id!("inputs.api_key");
const STATE_REFERENCE: Id = id!("inputs.state_reference");

const FIELD_PATCH: &[(Part, StylePatch)] =
    &[(Part::MARKER, StylePatch::new().set_fg(Role::Accent))];
const DETAIL_PATCH: &[(Part, StylePatch)] = &[(
    Part::DETAIL,
    StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
)];
const TOKEN_PATCH: &[(Part, StylePatch)] = &[(
    Part::LABEL,
    StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
)];

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

fn owner_input<'a>() -> TextInput<'a> {
    TextInput::new(OWNER)
        .validate(&email)
        .blur(BlurPolicy::CommitAndValidate)
}

fn token_input<'a>() -> TextInput<'a> {
    TextInput::new(TOKEN).disabled(true)
}

fn search_input<'a>() -> TextInput<'a> {
    TextInput::new(SEARCH)
        .placeholder("Type a path or symbol…")
        .blur(BlurPolicy::Commit)
}

fn api_key_input<'a>() -> TextInput<'a> {
    TextInput::new(API_KEY)
        .secret(SecretPolicy {
            synthetic_tail: 4,
            ..Default::default()
        })
        .blur(BlurPolicy::Commit)
}

/// The one card constructor for this page (§13): both phase paths build the
/// same `Panel::new(CARD)` props and vary only the heading text on top.
fn card_panel() -> Panel<'static> {
    Panel::new(CARD)
        .kind(PanelKind::Card)
        .patch_part(DETAIL_PATCH)
}

fn fields_panel() -> Panel<'static> {
    card_panel().title("Edit fields")
}

fn playground_panel() -> Panel<'static> {
    card_panel()
        .title("Playground")
        .meta("Enter Edit · Esc Cancel · Tab Commit + next")
}

fn state_reference_panel() -> Panel<'static> {
    Panel::new(STATE_REFERENCE)
        .kind(PanelKind::Card)
        .title("State reference")
        .meta("static")
        .patch_part(DETAIL_PATCH)
}

fn project_field(value: &str) -> Field<'_, TextInput<'_>> {
    Field::new("Project name", name_input().value(value))
        .required(true)
        .patch_part(FIELD_PATCH)
        .help("Used as the working directory name")
}

fn branch_field(value: &str) -> Field<'_, TextInput<'_>> {
    Field::new("Branch", branch_input().value(value))
        .help("Leave empty to work on a detached checkout")
}

fn owner_field<'a>(value: &'a str, error: Option<&'a str>) -> Field<'a, TextInput<'a>> {
    Field::new("Owner email", owner_input().value(value))
        .required(true)
        .patch_part(FIELD_PATCH)
        .error(error)
}

fn token_field<'a>(value: &'a str) -> Field<'a, TextInput<'a>> {
    Field::new("API token", token_input().value(value))
        .patch_part(TOKEN_PATCH)
        .help("Managed by the organization")
}

fn search_field<'a>(value: &'a str) -> Field<'a, TextInput<'a>> {
    Field::new("Search files", search_input().value(value))
        .help("Selection: Shift+← →  ·  words: Ctrl+← →  ·  clear: Ctrl+U")
}

fn api_key_field(value: &str) -> Field<'_, TextInput<'_>> {
    Field::new("API key", api_key_input().value(value))
        .help("Masked while typing; the last four characters show once committed")
}

fn columns(area: Rect, left_w: u16, gap: u16) -> (Rect, Rect) {
    if area.width < left_w.saturating_add(gap).saturating_add(20) {
        let h = area.height / 2;
        return (
            Rect::new(area.x, area.y, area.width, h),
            Rect::new(
                area.x,
                area.y.saturating_add(h),
                area.width,
                area.height.saturating_sub(h),
            ),
        );
    }
    (
        Rect::new(area.x, area.y, left_w, area.height),
        Rect::new(
            area.x.saturating_add(left_w).saturating_add(gap),
            area.y,
            area.width.saturating_sub(left_w).saturating_sub(gap),
            area.height,
        ),
    )
}

fn static_field(
    ui: &mut Ui<'_>,
    at: Rect,
    label: &str,
    text: &str,
    flags: StateFlags,
    editing: bool,
) {
    let (x, y, w) = (at.x, at.y, at.width);
    let label_style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
    let _ = ui.paint_str(Rect::new(x, y, 16.min(at.width), 1), label, label_style);
    if at.width <= 16 {
        return;
    }
    let field = Rect::new(x.saturating_add(16), y, w, 1);
    let mut fs = ui
        .style(Family::FIELD, Variant::DEFAULT, Part::FIELD, flags)
        .style;
    if !editing {
        fs = fs.remove_modifier(Modifier::UNDERLINED);
    }
    ui.fill(field, fs);
    if flags.contains(StateFlags::FOCUSED) && !flags.contains(StateFlags::DISABLED) {
        let gutter_style = ui
            .style(Family::FIELD, Variant::DEFAULT, Part::GUTTER, flags)
            .style;
        let _ = ui.paint_str(
            Rect::new(field.x, y, 1, 1),
            "▎",
            gutter_style.with_bg_from(fs),
        );
    } else {
        let _ = ui.paint_str(Rect::new(field.x, y, 1, 1), " ", fs.with_fg_from_bg(fs));
    }
    let style = if text.starts_with('(') {
        ui.style(Family::FIELD, Variant::DEFAULT, Part::PLACEHOLDER, flags)
            .style
    } else {
        fs
    };
    let style = if editing {
        style.add_modifier(Modifier::UNDERLINED)
    } else {
        style
    };
    let text_area = Rect::new(
        field.x.saturating_add(2),
        y,
        field.width.saturating_sub(2),
        1,
    );
    let _ = ui.paint_str(text_area, text, style);
    if editing {
        let cx = field
            .x
            .saturating_add(2)
            .saturating_add(termrock::width(text) as u16);
        let cursor_style = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_bg(Role::Fg(FgStep::Primary))));
        let _ = ui.paint_str(Rect::new(cx, y, 1, 1), " ", cursor_style);
    }
    if flags.contains(StateFlags::ERROR) && field.width >= 2 {
        let err_style = fs
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Danger)))
            .add_modifier(Modifier::BOLD);
        let _ = ui.paint_str(
            Rect::new(field.right().saturating_sub(2), y, 1, 1),
            "!",
            err_style,
        );
    }
}

/// Six controlled fields, matching the legacy input page: project, branch,
/// owner (invalid at boot), a disabled token, search, and the masked API
/// key (oracle reveal rule: 24 bullets plus the real last four).
#[derive(Debug)]
pub struct InputsPage {
    name: String,
    branch: String,
    owner: String,
    token: String,
    search: String,
    api_key: String,
    name_state: TextInputState,
    branch_state: TextInputState,
    owner_state: TextInputState,
    token_state: TextInputState,
    search_state: TextInputState,
    api_key_state: TextInputState,
    last: &'static str,
}

impl InputsPage {
    pub fn new() -> Self {
        let mut owner_state = TextInputState::default();
        // The legacy page validates the owner at construction: the boot
        // value is invalid, so the error shows before any edit.
        owner_state.set_error(Some(FieldError::new("Enter a valid email address")));
        Self {
            name: String::from("payments-gateway"),
            branch: String::new(),
            owner: String::from("mira@example"),
            token: String::from("jb_live_••••••••••••"),
            search: String::new(),
            // 28 graphemes ending in the registry tail: committed paint
            // shows 24 bullets plus `c1f2`, matching the approved frame.
            api_key: String::from("sk_live_0123456789abcdefc1f2"),
            name_state: TextInputState::default(),
            branch_state: TextInputState::default(),
            owner_state,
            token_state: TextInputState::default(),
            search_state: TextInputState::default(),
            api_key_state: TextInputState::default(),
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
        let mut status: Option<PageStatus> = None;
        // Both phases build the same card props (§13).
        let _ = fields_panel();
        let _ = playground_panel();
        let _ = state_reference_panel();
        let name = name_input().update(cx, &mut self.name_state, &mut self.name);
        if let Some(action) = name.action_ref() {
            self.last = match action {
                TextAction::Committed => "name committed",
                TextAction::Cancelled => "name reverted",
                TextAction::Changed => "name draft changed",
                TextAction::MoveNext | TextAction::MovePrev => "name focus moved",
            };
            Self::report(action, "Project name", &mut status);
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
            Self::report(action, "Branch", &mut status);
        }
        response |= branch.erase();
        let owner = owner_input().update(cx, &mut self.owner_state, &mut self.owner);
        if let Some(action) = owner.action_ref() {
            self.last = match action {
                TextAction::Committed => "owner committed",
                TextAction::Cancelled => "owner reverted",
                TextAction::Changed => "owner draft changed",
                TextAction::MoveNext | TextAction::MovePrev => "owner focus moved",
            };
            Self::report(action, "Owner email", &mut status);
        }
        response |= owner.erase();
        let token = token_input().update(cx, &mut self.token_state, &mut self.token);
        if let Some(action) = token.action_ref() {
            self.last = match action {
                TextAction::Committed => "token committed",
                TextAction::Cancelled => "token reverted",
                TextAction::Changed => "token draft changed",
                TextAction::MoveNext | TextAction::MovePrev => "token focus moved",
            };
            Self::report(action, "API token", &mut status);
        }
        response |= token.erase();
        let search = search_input().update(cx, &mut self.search_state, &mut self.search);
        if let Some(action) = search.action_ref() {
            self.last = match action {
                TextAction::Committed => "search committed",
                TextAction::Cancelled => "search reverted",
                TextAction::Changed => "search draft changed",
                TextAction::MoveNext | TextAction::MovePrev => "search focus moved",
            };
            Self::report(action, "Search files", &mut status);
        }
        response |= search.erase();
        let api_key = api_key_input().update(cx, &mut self.api_key_state, &mut self.api_key);
        if let Some(action) = api_key.action_ref() {
            self.last = match action {
                TextAction::Committed => "api_key committed",
                TextAction::Cancelled => "api_key reverted",
                TextAction::Changed => "api_key draft changed",
                TextAction::MoveNext | TextAction::MovePrev => "api_key focus moved",
            };
            Self::report(action, "API key", &mut status);
        }
        response |= api_key.erase();
        let mut update = PageUpdate::from(response);
        update.status = status;
        update
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let meta = "Focus is a bar; editing is a cursor. Enter to edit, Esc to revert.";
        frame(ui, area, self.title(), meta, |ui, body| {
            let regions = layout::rows(body, &[Track::Fixed(17), Track::Fixed(1), Track::Flex(1)]);
            let fields = regions.first().copied().unwrap_or(body);
            playground_panel().draw(ui, fields, |ui, inner| {
                let (l, r) = columns(inner, inner.width / 2 - 2, 4);
                let fh = 3;
                let slots = [
                    Rect::new(l.x, l.y, l.width, fh),
                    Rect::new(r.x, r.y, r.width, fh),
                    Rect::new(l.x, l.y.saturating_add(fh), l.width, fh),
                    Rect::new(r.x, r.y.saturating_add(fh), r.width, fh),
                    Rect::new(l.x, l.y.saturating_add(fh * 2), l.width, fh),
                    Rect::new(r.x, r.y.saturating_add(fh * 2), r.width, fh),
                ];
                if slots[0].bottom() <= inner.bottom() {
                    project_field(&self.name).draw(ui, slots[0], &self.name_state);
                }
                if slots[1].bottom() <= inner.bottom() {
                    branch_field(&self.branch).draw(ui, slots[1], &self.branch_state);
                }
                if slots[2].bottom() <= inner.bottom() {
                    owner_field(
                        &self.owner,
                        self.owner_state.error().map(|e| e.message.as_ref()),
                    )
                    .draw(ui, slots[2], &self.owner_state);
                }
                if slots[3].bottom() <= inner.bottom() {
                    token_field(&self.token).draw(ui, slots[3], &self.token_state);
                }
                if slots[4].bottom() <= inner.bottom() {
                    search_field(&self.search).draw(ui, slots[4], &self.search_state);
                }
                if slots[5].bottom() <= inner.bottom() {
                    api_key_field(&self.api_key).draw(ui, slots[5], &self.api_key_state);
                }
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
        } else if ui.state(OWNER).contains(StateFlags::FOCUSED) {
            self.owner_state.is_editing()
        } else if ui.state(SEARCH).contains(StateFlags::FOCUSED) {
            self.search_state.is_editing()
        } else if ui.state(API_KEY).contains(StateFlags::FOCUSED) {
            self.api_key_state.is_editing()
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
        self.name_state.is_editing()
            || self.branch_state.is_editing()
            || self.owner_state.is_editing()
            || self.search_state.is_editing()
            || self.api_key_state.is_editing()
    }
}

impl InputsPage {
    /// Legacy commit feedback: `{label} saved` on commit, `Reverted` on
    /// cancel. The shell owns the status display lifetime.
    fn report(action: &TextAction, label: &str, status: &mut Option<PageStatus>) {
        match action {
            TextAction::Committed => {
                *status = Some(PageStatus(format!("{label} saved")));
            }
            TextAction::Cancelled => {
                *status = Some(PageStatus("Reverted".to_owned()));
            }
            _ => {}
        }
    }

    fn draw_state_reference(ui: &mut Ui<'_>, inner: Rect) {
        let w = inner.width.saturating_sub(18).min(34);
        let states = [
            ("default", "payments-gateway", StateFlags::empty(), false),
            ("placeholder", "(feat/…)", StateFlags::empty(), false),
            ("hover", "payments-gateway", StateFlags::HOVERED, false),
            ("focused", "payments-gateway", StateFlags::FOCUSED, false),
            ("editing", "payments-gateway", StateFlags::FOCUSED, true),
            ("error", "mira@example", StateFlags::ERROR, false),
            (
                "error + focus",
                "mira@example",
                StateFlags::ERROR | StateFlags::FOCUSED,
                false,
            ),
            ("disabled", "jb_live_••••", StateFlags::DISABLED, false),
        ];
        for (i, (name, text, flags, editing)) in states.into_iter().enumerate() {
            let Ok(offset) = u16::try_from(i) else {
                break;
            };
            let y = inner.y.saturating_add(offset);
            if y >= inner.bottom() {
                break;
            }
            static_field(ui, Rect::new(inner.x, y, w, 1), name, text, flags, editing);
        }
    }
}
