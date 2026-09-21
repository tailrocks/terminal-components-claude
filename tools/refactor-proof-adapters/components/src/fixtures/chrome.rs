//! Chrome fixtures: button, brand, choice, radio, chips, readiness, meters.

use junie_tui::{
    App, Brand, Button, Checkbox, ChipBar, ChipBarState, Chord, Cx, Empty, EmptyState, Hint,
    HintBar, HintLayer, Id, ItemKey, KeyCode, KeyHint, Meter, ProgressBar, RadioGroup,
    RadioGroupAction, RadioGroupState, Rect, Response, SelectMode, Spinner, Status, StatusBar,
    StatusItem, Toggle, Ui, Variant,
};

use crate::fixtures::{FixtureApp, FixtureCfg};
use crate::states::ComponentState;

const BTN: Id = Id::root("oracle.components.button");
const BRAND: Id = Id::root("oracle.components.brand");
const CHECKBOX: Id = Id::root("oracle.components.checkbox");
const TOGGLE: Id = Id::root("oracle.components.toggle");
const RADIO: Id = Id::root("oracle.components.radio");
const CHIPS: Id = Id::root("oracle.components.chips");
const EMPTY: Id = Id::root("oracle.components.empty");
const PROGRESS: Id = Id::root("oracle.components.progress");
const SPINNER: Id = Id::root("oracle.components.spinner");
const METER: Id = Id::root("oracle.components.meter");
const STATUS: Id = Id::root("oracle.components.status");
const HINT: Id = Id::root("oracle.components.hint");
const KEYHINT: Id = Id::root("oracle.components.keyhint");

const PEOPLE: [&str; 3] = ["Ada Lovelace", "Grace Hopper", "Hedy Lamarr"];

/// Button fixture.
#[derive(Debug)]
pub struct ButtonFixture {
    cfg: FixtureCfg,
}

impl ButtonFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self { cfg }
    }

    fn widget(&self) -> Button<'static> {
        Button::new(BTN, "Run task")
            .variant(Variant::PRIMARY)
            .disabled(self.cfg.state == ComponentState::Disabled)
    }
}

impl App for ButtonFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.widget().update(cx).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget().draw(ui, self.cfg.widget_rect());
    }
}

impl FixtureApp for ButtonFixture {
    fn fixture_id(&self) -> Id {
        BTN
    }
}

/// Click-only brand lockup fixture.
#[derive(Debug)]
pub struct BrandFixture {
    cfg: FixtureCfg,
}

impl BrandFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self { cfg }
    }

    fn widget() -> Brand<'static> {
        Brand::new(BRAND, "Junie")
            .tagline("Terminal tools")
            .clickable(true)
    }
}

impl App for BrandFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Self::widget().update(cx).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Self::widget().draw(ui, self.cfg.widget_rect());
    }
}

impl FixtureApp for BrandFixture {
    fn fixture_id(&self) -> Id {
        BRAND
    }
}

/// Checkbox plus toggle fixture (one family, both choice widgets).
#[derive(Debug)]
pub struct CheckboxFixture {
    cfg: FixtureCfg,
    checked: bool,
    on: bool,
}

impl CheckboxFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            checked: false,
            on: false,
        }
    }

    const fn disabled(&self) -> bool {
        matches!(self.cfg.state, ComponentState::Disabled)
    }
}

impl App for CheckboxFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let checkbox = Checkbox::new(CHECKBOX, "Accept terms").disabled(self.disabled());
        let toggle = Toggle::new(TOGGLE, "Notifications").disabled(self.disabled());
        let first = checkbox.update(cx, &mut self.checked).erase();
        let second = toggle.update(cx, &mut self.on).erase();
        first | second
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = self.cfg.widget_rect();
        let half = area.height / 2;
        let top = Rect::new(area.x, area.y, area.width, half);
        let bottom = Rect::new(
            area.x,
            area.y.saturating_add(half),
            area.width,
            area.height.saturating_sub(half),
        );
        Checkbox::new(CHECKBOX, "Accept terms")
            .checked(self.checked)
            .disabled(self.disabled())
            .draw(ui, top);
        Toggle::new(TOGGLE, "Notifications")
            .on(self.on)
            .disabled(self.disabled())
            .draw(ui, bottom);
    }
}

impl FixtureApp for CheckboxFixture {
    fn fixture_id(&self) -> Id {
        CHECKBOX
    }

    fn check(&self, state: ComponentState) -> bool {
        if state == ComponentState::Selected {
            return self.checked && self.on;
        }
        true
    }
}

/// Keyed radio group fixture.
#[derive(Debug)]
pub struct RadioFixture {
    cfg: FixtureCfg,
    state: RadioGroupState,
    chosen: Option<ItemKey>,
}

impl RadioFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: RadioGroupState::default(),
            chosen: None,
        }
    }

    fn widget(
        &self,
    ) -> RadioGroup<'static, &'static str, junie_tui::ByIndex, junie_tui::DefaultRow> {
        let widget = RadioGroup::new(RADIO).disabled(self.cfg.state == ComponentState::Disabled);
        if let Some(value) = self.chosen {
            widget.value(value)
        } else {
            widget
        }
    }
}

impl App for RadioFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = self.widget().update(cx, &mut self.state, &PEOPLE);
        if let Some(RadioGroupAction::Chose(key)) = response.action_ref() {
            self.chosen = Some(*key);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget()
            .draw(ui, self.cfg.widget_rect(), &self.state, &PEOPLE);
    }
}

impl FixtureApp for RadioFixture {
    fn fixture_id(&self) -> Id {
        RADIO
    }

    fn check(&self, state: ComponentState) -> bool {
        match state {
            ComponentState::Current => self.state.cursor_index() == 1,
            ComponentState::Selected => self.chosen.is_some(),
            _ => true,
        }
    }
}

/// Keyed chip bar fixture.
#[derive(Debug)]
pub struct ChipsFixture {
    cfg: FixtureCfg,
    state: ChipBarState,
}

impl ChipsFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: ChipBarState::default(),
        }
    }

    fn widget(&self) -> ChipBar<'static, &'static str, junie_tui::ByIndex, junie_tui::DefaultRow> {
        ChipBar::new(CHIPS)
            .select_mode(SelectMode::Multi)
            .closable(true)
            .disabled(self.cfg.state == ComponentState::Disabled)
    }
}

impl App for ChipsFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.widget().update(cx, &mut self.state, &PEOPLE).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget()
            .draw(ui, self.cfg.widget_rect(), &self.state, &PEOPLE);
    }
}

impl FixtureApp for ChipsFixture {
    fn fixture_id(&self) -> Id {
        CHIPS
    }

    fn check(&self, state: ComponentState) -> bool {
        if state == ComponentState::Selected {
            return !self.state.checked().is_empty();
        }
        true
    }
}

/// Readiness fixture across `EmptyState` variants.
#[derive(Debug)]
pub struct EmptyFixture {
    cfg: FixtureCfg,
}

impl EmptyFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self { cfg }
    }

    const fn surface(&self) -> EmptyState<'static> {
        match self.cfg.state {
            ComponentState::Loading => EmptyState::Loading { label: "Loading" },
            ComponentState::Error => EmptyState::Error {
                message: "Unable to load",
                detail: Some("Try again"),
            },
            ComponentState::Empty => EmptyState::Empty {
                title: "",
                hint: None,
            },
            _ => EmptyState::Empty {
                title: "No results",
                hint: Some("Try a different filter"),
            },
        }
    }
}

impl App for EmptyFixture {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Empty::new(EMPTY, self.surface()).draw(ui, self.cfg.widget_rect());
    }
}

impl FixtureApp for EmptyFixture {
    fn fixture_id(&self) -> Id {
        EMPTY
    }
}

/// Progress bar fixture.
#[derive(Debug)]
pub struct ProgressFixture {
    cfg: FixtureCfg,
}

impl ProgressFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self { cfg }
    }
}

impl App for ProgressFixture {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let status = if self.cfg.state == ComponentState::Busy {
            Status::Busy
        } else {
            Status::Ready
        };
        ProgressBar::new(PROGRESS)
            .label("Uploading")
            .ratio(0.65)
            .status(status)
            .draw(ui, self.cfg.widget_rect());
    }
}

impl FixtureApp for ProgressFixture {
    fn fixture_id(&self) -> Id {
        PROGRESS
    }
}

/// Spinner fixture.
#[derive(Debug)]
pub struct SpinnerFixture {
    cfg: FixtureCfg,
}

impl SpinnerFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self { cfg }
    }
}

impl App for SpinnerFixture {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Spinner::new(SPINNER)
            .label("Working")
            .frame(0)
            .draw(ui, self.cfg.widget_rect());
    }
}

impl FixtureApp for SpinnerFixture {
    fn fixture_id(&self) -> Id {
        SPINNER
    }
}

/// Capacity meter fixture.
#[derive(Debug)]
pub struct MeterFixture {
    cfg: FixtureCfg,
}

impl MeterFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self { cfg }
    }
}

impl App for MeterFixture {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let status = match self.cfg.state {
            ComponentState::Busy => Status::Busy,
            ComponentState::Error => Status::Error,
            _ => Status::Ready,
        };
        Meter::new(METER)
            .ratio(0.65)
            .value("65%")
            .status(status)
            .draw(ui, self.cfg.widget_rect());
    }
}

impl FixtureApp for MeterFixture {
    fn fixture_id(&self) -> Id {
        METER
    }
}

const STATUS_LEFT: [StatusItem<'static>; 2] = [
    StatusItem::new("Workspace").strong(),
    StatusItem::new("main").key(ItemKey::num(1)),
];
const STATUS_CENTER: [StatusItem<'static>; 1] = [StatusItem::new("Ready")];
const STATUS_RIGHT: [StatusItem<'static>; 1] = [StatusItem::new("UTF-8")];

/// Status bar fixture.
#[derive(Debug)]
pub struct StatusFixture {
    cfg: FixtureCfg,
}

impl StatusFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self { cfg }
    }
}

impl App for StatusFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        StatusBar::new(STATUS).update(cx).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let empty = self.cfg.state == ComponentState::Empty;
        let status = match self.cfg.state {
            ComponentState::Busy => Status::Busy,
            ComponentState::Error => Status::Error,
            _ => Status::Ready,
        };
        StatusBar::new(STATUS)
            .left(if empty { &[] } else { &STATUS_LEFT })
            .center(if empty { &[] } else { &STATUS_CENTER })
            .right(if empty { &[] } else { &STATUS_RIGHT })
            .status(status)
            .draw(ui, self.cfg.widget_rect());
    }
}

impl FixtureApp for StatusFixture {
    fn fixture_id(&self) -> Id {
        STATUS
    }
}

/// Hint bar fixture.
#[derive(Debug)]
pub struct HintFixture {
    cfg: FixtureCfg,
}

impl HintFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self { cfg }
    }
}

impl App for HintFixture {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let layer = HintLayer {
            hints: vec![
                Hint {
                    key: junie_tui::HintKey::Chord(Chord::key(KeyCode::Enter)),
                    label: "Open",
                    priority: 80,
                },
                Hint {
                    key: junie_tui::HintKey::Chord(Chord::key(KeyCode::Esc)),
                    label: "Close",
                    priority: 70,
                },
            ],
            badge: Some("F1"),
            status: Some(std::borrow::Cow::Borrowed("Ready")),
            centered: false,
        };
        HintBar::new(HINT, &layer)
            .status(Status::Ready)
            .draw(ui, self.cfg.widget_rect());
    }
}

impl FixtureApp for HintFixture {
    fn fixture_id(&self) -> Id {
        HINT
    }
}

/// Key hint fixture.
#[derive(Debug)]
pub struct KeyHintFixture {
    cfg: FixtureCfg,
}

impl KeyHintFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self { cfg }
    }
}

impl App for KeyHintFixture {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        KeyHint::new(KEYHINT, Chord::key(KeyCode::Enter), "Open").draw(ui, self.cfg.widget_rect());
    }
}

impl FixtureApp for KeyHintFixture {
    fn fixture_id(&self) -> Id {
        KEYHINT
    }
}
