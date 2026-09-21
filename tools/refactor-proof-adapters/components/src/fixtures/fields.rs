//! Field fixtures: chrome, text input/area, secrets, select, form.

use junie_tui::{
    Action, ActionKey, App, Checkbox, Cx, Field, FieldKind, FieldMut, FieldRef, FieldSpec, Form,
    FormData, FormState, Id, Response, SecretPolicy, Select, SelectState, TextArea, TextAreaState,
    TextInput, TextInputState, Ui,
};

use crate::fixtures::{FixtureApp, FixtureCfg};
use crate::states::ComponentState;

const FIELD_CHILD: Id = Id::root("oracle.components.field.child");
const INPUT: Id = Id::root("oracle.components.input");
const AREA: Id = Id::root("oracle.components.area");
const SECRET: Id = Id::root("oracle.components.secret");
const SELECT: Id = Id::root("oracle.components.select");
const FORM: Id = Id::root("oracle.components.form");
const FORM_NAME: Id = Id::root("oracle.components.form.name");
const FORM_ENABLED: Id = Id::root("oracle.components.form.enabled");

/// Unicode corpus shared by text fixtures (CJK, emoji, combining mark).
// The decomposed e+combining-acute is the point: the corpus must carry a
// non-NFC sequence so grapheme handling is observed, not assumed.
#[allow(
    clippy::unicode_not_nfc,
    reason = "decomposed sequence is the test corpus"
)]
pub const UNICODE_VALUE: &str = "Ada Lovelace 世界 🌍 é";
/// Multiline Unicode corpus for the text area fixture.
#[allow(
    clippy::unicode_not_nfc,
    reason = "decomposed sequence is the test corpus"
)]
pub const UNICODE_AREA: &str = "Ada Lovelace 世界 🌍\nsecond line é\nthird";

const SELECT_ITEMS: [&str; 4] = ["Ada Lovelace", "Grace Hopper", "Hedy Lamarr", "Alan Turing"];

/// Field chrome fixture: `Field` plus one child input, no second focus stop.
#[derive(Debug)]
pub struct FieldFixture {
    cfg: FixtureCfg,
    state: TextInputState,
    value: String,
}

impl FieldFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: TextInputState::default(),
            value: UNICODE_VALUE.to_owned(),
        }
    }

    fn child(&self) -> TextInput<'static> {
        TextInput::new(FIELD_CHILD)
            .placeholder("Type a name")
            .disabled(self.cfg.state == ComponentState::Disabled)
    }
}

impl App for FieldFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.child()
            .update(cx, &mut self.state, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let error = if self.cfg.state == ComponentState::Error {
            Some("Name is required")
        } else {
            None
        };
        Field::new("Name", self.child().value(&self.value))
            .required(true)
            .help("The person's display name.")
            .error(error)
            .draw(ui, self.cfg.widget_rect(), &self.state);
    }
}

impl FixtureApp for FieldFixture {
    fn fixture_id(&self) -> Id {
        FIELD_CHILD
    }
}

/// Single-line text input fixture.
#[derive(Debug)]
pub struct InputFixture {
    cfg: FixtureCfg,
    state: TextInputState,
    value: String,
}

impl InputFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        let value = if cfg.state == ComponentState::Empty {
            String::new()
        } else {
            UNICODE_VALUE.to_owned()
        };
        Self {
            cfg,
            state: TextInputState::default(),
            value,
        }
    }

    fn widget(&self) -> TextInput<'static> {
        TextInput::new(INPUT)
            .placeholder("Type a name")
            .disabled(self.cfg.state == ComponentState::Disabled)
            .read_only(self.cfg.state == ComponentState::ReadOnly)
    }
}

impl App for InputFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.widget()
            .update(cx, &mut self.state, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget()
            .value(&self.value)
            .draw(ui, self.cfg.widget_rect(), &self.state);
    }
}

impl FixtureApp for InputFixture {
    fn fixture_id(&self) -> Id {
        INPUT
    }

    fn check(&self, state: ComponentState) -> bool {
        match state {
            ComponentState::Editing | ComponentState::Selected => self.state.is_editing(),
            _ => true,
        }
    }
}

/// Multiline text area fixture.
#[derive(Debug)]
pub struct AreaFixture {
    cfg: FixtureCfg,
    state: TextAreaState,
    value: String,
}

impl AreaFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        let value = if cfg.state == ComponentState::Empty {
            String::new()
        } else if let crate::expansion::Facet::FadeViewport { position, .. } = cfg.facet {
            let count = match position {
                crate::expansion::FadePosition::Fits => 2,
                _ => 40,
            };
            (0..count)
                .map(|index| format!("area line {index:02} 世界"))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            UNICODE_AREA.to_owned()
        };
        Self {
            cfg,
            state: TextAreaState::default(),
            value,
        }
    }

    fn widget(&self) -> TextArea<'static> {
        TextArea::new(AREA, 4)
            .placeholder("Type a note")
            .disabled(self.cfg.state == ComponentState::Disabled)
            .read_only(self.cfg.state == ComponentState::ReadOnly)
    }
}

impl App for AreaFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.widget()
            .update(cx, &mut self.state, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget()
            .value(&self.value)
            .draw(ui, self.cfg.widget_rect(), &self.state);
    }
}

impl FixtureApp for AreaFixture {
    fn fixture_id(&self) -> Id {
        AREA
    }

    fn check(&self, state: ComponentState) -> bool {
        match state {
            ComponentState::Editing | ComponentState::Selected => self.state.is_editing(),
            _ => true,
        }
    }
}

/// Masked-secret input fixture with a validation error state.
#[derive(Debug)]
pub struct SecretFixture {
    cfg: FixtureCfg,
    state: TextInputState,
    value: String,
}

impl SecretFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: TextInputState::default(),
            value: "s3cr3t-token".to_owned(),
        }
    }

    fn child(&self) -> TextInput<'static> {
        TextInput::new(SECRET)
            .secret(SecretPolicy::default())
            .disabled(self.cfg.state == ComponentState::Disabled)
    }
}

impl App for SecretFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.child()
            .update(cx, &mut self.state, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let error = if self.cfg.state == ComponentState::Error {
            Some("Token must be at least 16 characters")
        } else {
            None
        };
        Field::new("API token", self.child().value(&self.value))
            .required(true)
            .error(error)
            .draw(ui, self.cfg.widget_rect(), &self.state);
    }
}

impl FixtureApp for SecretFixture {
    fn fixture_id(&self) -> Id {
        SECRET
    }

    fn check(&self, state: ComponentState) -> bool {
        if state == ComponentState::Editing {
            return self.state.is_editing();
        }
        true
    }
}

/// Dropdown select fixture.
#[derive(Debug)]
pub struct SelectFixture {
    cfg: FixtureCfg,
    state: SelectState,
}

impl SelectFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: SelectState::default(),
        }
    }

    fn widget(&self) -> Select<'static, &'static str, junie_tui::ByIndex, junie_tui::DefaultRow> {
        Select::new(SELECT)
            .placeholder("Choose a person")
            .popup_rows(5)
            .disabled(self.cfg.state == ComponentState::Disabled)
    }
}

impl App for SelectFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.widget()
            .update(cx, &mut self.state, &SELECT_ITEMS)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget()
            .draw(ui, self.cfg.widget_rect(), &self.state, &SELECT_ITEMS);
    }
}

impl FixtureApp for SelectFixture {
    fn fixture_id(&self) -> Id {
        SELECT
    }
}

const FORM_ACTIONS: [Action<'static>; 2] = [
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ActionKey::CONFIRM, "OK"),
];

#[derive(Debug)]
struct FormValues {
    name: String,
    enabled: bool,
    disabled: bool,
}

impl FormData for FormValues {
    fn value(&self, id: Id) -> FieldRef<'_> {
        if id == FORM_NAME {
            FieldRef::Text(&self.name)
        } else {
            FieldRef::Flag(self.enabled)
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        if id == FORM_NAME {
            FieldMut::Text(&mut self.name)
        } else {
            FieldMut::Flag(&mut self.enabled)
        }
    }

    fn disabled(&self, _id: Id) -> bool {
        self.disabled
    }
}

/// ID-keyed form fixture with a validation error state.
#[derive(Debug)]
pub struct FormFixture {
    cfg: FixtureCfg,
    state: FormState,
    data: FormValues,
    fields: [FieldSpec<'static>; 2],
}

impl FormFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        let error_name = cfg.state == ComponentState::Error;
        Self {
            cfg,
            state: FormState::default(),
            data: FormValues {
                name: if error_name {
                    String::new()
                } else {
                    "Ada Lovelace".to_owned()
                },
                enabled: false,
                disabled: cfg.state == ComponentState::Disabled,
            },
            fields: [
                FieldSpec::new(
                    FORM_NAME,
                    "Name",
                    FieldKind::Text(TextInput::new(FORM_NAME)),
                ),
                FieldSpec::new(
                    FORM_ENABLED,
                    "",
                    FieldKind::Check(Checkbox::new(FORM_ENABLED, "Enabled")),
                ),
            ],
        }
    }

    fn widget(&self) -> Form<'_> {
        Form::new(FORM, &self.fields).actions(&FORM_ACTIONS)
    }
}

impl App for FormFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let Self {
            state,
            data,
            fields,
            ..
        } = self;
        Form::new(FORM, fields)
            .actions(&FORM_ACTIONS)
            .update(cx, state, data)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget()
            .draw(ui, self.cfg.widget_rect(), &self.state, &self.data);
    }
}

impl FixtureApp for FormFixture {
    fn fixture_id(&self) -> Id {
        FORM_NAME
    }

    fn check(&self, state: ComponentState) -> bool {
        if state == ComponentState::Editing {
            return self.state.is_dirty();
        }
        true
    }
}
