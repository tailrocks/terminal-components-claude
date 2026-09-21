//! Overlay fixtures: picker, chain, completion, dialog, menu, help, wizard.

use junie_tui::{
    Action, ActionKey, Anchor, App, Chord, Completion, CompletionAction, CompletionController,
    CompletionState, ContextMenu, Cx, Dialog, DialogState, FrameRead, HelpOverlay,
    HelpOverlayState, HelpSection, Hint, HintLayer, Id, Item, ItemKey, KeyCode, Menu, MenuBar,
    MenuItem, MenuState, Part, Picker, PickerChain, PickerChainState, PickerStage, PickerState,
    Rect, Response, ScreenAlign, Status, TextInput, TextInputState, Ui, Wizard, WizardState,
    WizardStep,
};

use crate::fixtures::{FixtureApp, FixtureCfg};
use crate::states::ComponentState;

const PICKER: Id = Id::root("oracle.components.picker");
const CHAIN: Id = Id::root("oracle.components.chain");
const COMPLETION: Id = Id::root("oracle.components.completion");
const COMPLETION_EDITOR: Id = Id::root("oracle.components.completion.editor");
const DIALOG: Id = Id::root("oracle.components.dialog");
const MENU: Id = Id::root("oracle.components.menu");
const HELP: Id = Id::root("oracle.components.help");
const WIZARD: Id = Id::root("oracle.components.wizard");

const PICKER_ITEMS: [Item<'static>; 4] = [
    Item::new(ItemKey::num(1), "Ada Lovelace").detail("analyst"),
    Item::new(ItemKey::num(2), "Grace Hopper").detail("rear admiral"),
    Item::new(ItemKey::num(3), "Alan Turing").detail("logician"),
    Item::new(ItemKey::num(4), "Hedy Lamarr").detail("inventor"),
];

const DIALOG_ACTIONS: [Action<'static>; 2] = [
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ActionKey::CONFIRM, "OK"),
];

const MENU_ITEMS: [MenuItem<'static>; 2] = [
    MenuItem::new(ActionKey::SAVE, "Save").chord(Chord::key(KeyCode::Char('s'))),
    MenuItem::new(ActionKey::CLOSE, "Close"),
];
const MENUS: [Menu<'static>; 1] = [Menu::new("File", &MENU_ITEMS)];
const DISABLED_MENU_ITEMS: [MenuItem<'static>; 2] = [
    MenuItem::new(ActionKey::SAVE, "Save")
        .chord(Chord::key(KeyCode::Char('s')))
        .disabled(true),
    MenuItem::new(ActionKey::CLOSE, "Close").disabled(true),
];
const DISABLED_MENUS: [Menu<'static>; 1] = [Menu::new("File", &DISABLED_MENU_ITEMS)];

/// Command-palette picker fixture.
#[derive(Debug)]
pub struct PickerFixture {
    cfg: FixtureCfg,
    state: PickerState,
    items: Vec<Item<'static>>,
}

impl PickerFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        const LABELS: [(&str, &str); 4] = [
            ("Ada Lovelace", "analyst"),
            ("Grace Hopper", "rear admiral"),
            ("Alan Turing", "logician"),
            ("Hedy Lamarr", "inventor"),
        ];
        let count = match cfg.facet {
            crate::expansion::Facet::FadeViewport { position, .. } => match position {
                crate::expansion::FadePosition::Fits => 2,
                _ => 40,
            },
            _ => PICKER_ITEMS.len(),
        };
        let disabled = cfg.state == ComponentState::Disabled;
        let items: Vec<Item<'static>> = LABELS
            .iter()
            .cycle()
            .take(count)
            .enumerate()
            .map(|(index, base)| {
                Item::new(ItemKey::num(index.saturating_add(1) as u64), base.0)
                    .detail(base.1)
                    .disabled(disabled)
            })
            .collect();
        Self {
            cfg,
            state: PickerState::default(),
            items,
        }
    }
}

impl App for PickerFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Picker::new(PICKER)
            .update(cx, &mut self.state, &self.items)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Picker::new(PICKER).draw(ui, self.cfg.widget_rect(), &self.state, &self.items);
    }
}

impl FixtureApp for PickerFixture {
    fn fixture_id(&self) -> Id {
        PICKER
    }

    fn check(&self, state: ComponentState) -> bool {
        match state {
            ComponentState::Editing => !self.state.query().is_empty(),
            ComponentState::Current => self.state.cursor().is_some(),
            _ => true,
        }
    }
}

const CHAIN_STAGES: [PickerStage<'static>; 2] = [
    PickerStage::new(ItemKey::num(1), "Account"),
    PickerStage::new(ItemKey::num(2), "Vault").status(Status::Loading),
];

/// Staged picker-chain fixture.
#[derive(Debug)]
pub struct ChainFixture {
    cfg: FixtureCfg,
    state: PickerChainState,
}

impl ChainFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        let mut state = PickerChainState::default();
        state.enter(ItemKey::num(1));
        if cfg.state == ComponentState::Current {
            state.enter(ItemKey::num(2));
        }
        Self { cfg, state }
    }
}

impl App for ChainFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        PickerChain::new(CHAIN, &CHAIN_STAGES)
            .update(cx, &mut self.state)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        PickerChain::new(CHAIN, &CHAIN_STAGES).draw(ui, self.cfg.widget_rect(), &self.state);
    }
}

impl FixtureApp for ChainFixture {
    fn fixture_id(&self) -> Id {
        CHAIN
    }

    fn check(&self, state: ComponentState) -> bool {
        if state == ComponentState::Current {
            return self.state.current() == Some(ItemKey::num(2));
        }
        true
    }
}

/// Inline completion fixture: editor plus controller-driven popover.
#[derive(Debug)]
pub struct CompletionFixture {
    cfg: FixtureCfg,
    editor: TextInputState,
    value: String,
    state: CompletionState,
    accepted: Option<ItemKey>,
}

impl CompletionFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            editor: TextInputState::default(),
            value: "Ada".to_owned(),
            state: CompletionState::default(),
            accepted: None,
        }
    }
}

impl App for CompletionFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let controller = CompletionController::new(COMPLETION_EDITOR, COMPLETION);
        let editor_response =
            TextInput::new(COMPLETION_EDITOR).update(cx, &mut self.editor, &mut self.value);
        // Base rests with the popover closed; Current/Selected open it
        // through the real controller request. An always-open popover would
        // modal-trap focus and make the resting frame unreachable.
        if self.cfg.state != ComponentState::Base {
            let anchor = cx.area(COMPLETION_EDITOR).unwrap_or(Rect::new(0, 0, 20, 1));
            controller.request(cx, &mut self.state, anchor, 3, &PICKER_ITEMS);
        }
        let popup = Completion::new(COMPLETION);
        let response = popup.update_for(COMPLETION_EDITOR, cx, &mut self.state, &PICKER_ITEMS);
        if let Some(CompletionAction::Accepted(key)) = response.action_ref() {
            self.accepted = Some(*key);
        }
        editor_response.erase() | response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = self.cfg.widget_rect();
        TextInput::new(COMPLETION_EDITOR)
            .value(&self.value)
            .draw(ui, area, &self.editor);
        Completion::new(COMPLETION).draw(ui, area, &self.state, &PICKER_ITEMS);
    }
}

impl FixtureApp for CompletionFixture {
    fn fixture_id(&self) -> Id {
        COMPLETION_EDITOR
    }

    fn check(&self, state: ComponentState) -> bool {
        match state {
            ComponentState::Current => self.state.cursor().is_some(),
            ComponentState::Selected => self.accepted.is_some(),
            _ => true,
        }
    }
}

/// Modal dialog fixture over a dimmed host line.
#[derive(Debug)]
pub struct DialogFixture {
    cfg: FixtureCfg,
    state: DialogState,
}

impl DialogFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: DialogState::default(),
        }
    }
}

impl App for DialogFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Dialog::new(DIALOG).update(cx, &mut self.state).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Dialog::new(DIALOG)
            .title("Delete table")
            .description("This cannot be undone. Every row and every index goes with it.")
            .body_rows(0)
            .actions(&DIALOG_ACTIONS)
            .cancel(ActionKey::CANCEL)
            .draw(ui, self.cfg.widget_rect(), &self.state, |_, _| {});
    }
}

impl FixtureApp for DialogFixture {
    fn fixture_id(&self) -> Id {
        // Modal focus lands on the first action, never the dialog shell.
        DIALOG.part(Part::ACTIONS).index(0)
    }
}

/// Menu bar plus context-menu fixture.
#[derive(Debug)]
pub struct MenuFixture {
    cfg: FixtureCfg,
    state: MenuState,
}

impl MenuFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: MenuState::default(),
        }
    }
}

impl MenuFixture {
    fn menus(&self) -> &'static [Menu<'static>] {
        if self.cfg.state == ComponentState::Disabled {
            &DISABLED_MENUS
        } else {
            &MENUS
        }
    }
}

impl App for MenuFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        MenuBar::new(MENU, self.menus())
            .update(cx, &mut self.state)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = self.cfg.widget_rect();
        let bar_h = 1.min(area.height);
        let bar = Rect::new(area.x, area.y, area.width, bar_h);
        MenuBar::new(MENU, self.menus()).draw(ui, bar, &self.state);
        // Disabled is per-item and only observable with the menu open, so
        // the Disabled frame opens the menu over disabled items.
        if self.cfg.state == ComponentState::Pressed {
            ContextMenu::new(MENU, &MENU_ITEMS, Anchor::Screen(ScreenAlign::Center)).draw(
                ui,
                area,
                &self.state,
            );
        } else if self.cfg.state == ComponentState::Disabled {
            ContextMenu::new(
                MENU,
                &DISABLED_MENU_ITEMS,
                Anchor::Screen(ScreenAlign::Center),
            )
            .draw(ui, area, &self.state);
        }
    }
}

impl FixtureApp for MenuFixture {
    fn fixture_id(&self) -> Id {
        MENU
    }
}

/// Help overlay fixture.
#[derive(Debug)]
pub struct HelpFixture {
    cfg: FixtureCfg,
    state: HelpOverlayState,
}

impl HelpFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: HelpOverlayState::default(),
        }
    }
}

impl App for HelpFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let layer = HintLayer {
            hints: vec![Hint {
                key: junie_tui::HintKey::Chord(Chord::key(KeyCode::Enter)),
                label: "Choose",
                priority: 80,
            }],
            ..HintLayer::default()
        };
        let sections = [HelpSection::new("General", &layer)];
        HelpOverlay::new(HELP, "Application", &sections)
            .update(cx, &mut self.state)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let layer = HintLayer {
            hints: vec![Hint {
                key: junie_tui::HintKey::Chord(Chord::key(KeyCode::Enter)),
                label: "Choose",
                priority: 80,
            }],
            ..HintLayer::default()
        };
        let sections = [HelpSection::new("General", &layer)];
        HelpOverlay::new(HELP, "Application", &sections).draw(
            ui,
            self.cfg.widget_rect(),
            &self.state,
        );
    }
}

impl FixtureApp for HelpFixture {
    fn fixture_id(&self) -> Id {
        HELP
    }
}

const WIZARD_STEPS: [WizardStep<'static>; 3] = [
    WizardStep::new(ItemKey::num(1), "Account"),
    WizardStep::new(ItemKey::num(2), "Details"),
    WizardStep::new(ItemKey::num(3), "Review"),
];

/// Multi-step wizard fixture.
#[derive(Debug)]
pub struct WizardFixture {
    cfg: FixtureCfg,
    state: WizardState<()>,
}

impl WizardFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: WizardState::default(),
        }
    }
}

impl App for WizardFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Wizard::new(WIZARD, &WIZARD_STEPS)
            .update(cx, &mut self.state)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Wizard::new(WIZARD, &WIZARD_STEPS).draw(ui, self.cfg.widget_rect(), &self.state);
    }
}

impl FixtureApp for WizardFixture {
    fn fixture_id(&self) -> Id {
        WIZARD
    }

    fn check(&self, state: ComponentState) -> bool {
        if state == ComponentState::Current {
            return self.state.current() == Some(ItemKey::num(2));
        }
        true
    }
}
