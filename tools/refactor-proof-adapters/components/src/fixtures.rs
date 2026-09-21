//! Production-path fixture capture.
//!
//! Each Direct family owns a tiny production [`App`] that constructs a real
//! widget, routes real inputs through the widget's `update`, and paints
//! through the widget's `draw`. Runtime-owned states (focus/hover/pressed,
//! cursor, selection, edit mode) are reached through delivered `Input`, and
//! [`FixtureApp::check`] verifies the semantic arrival of every state that
//! has a readable signal. Nothing is injected through reference-state
//! overrides, and no oracle snapshot is read.

use junie_tui::{
    App, Cx, Focusability, Id, Input, Key, KeyCode, KeyModifiers, Moment, Mouse, MouseKind,
    Position, Rect, Response, Runtime, Theme, Ui,
};
use ratatui_core::buffer::Buffer;

use crate::color::{ColorSpec, Origin, TerminalSize};
use crate::disposition::{Lane, disposition};
use crate::error::AdapterError;
use crate::expansion::{ExpandedCase, Facet};
use crate::families::Family;
use crate::observe::{FrameObservation, HitObs};
use crate::states::ComponentState;

pub mod chrome;
pub mod fields;
pub mod lists;
pub mod overlays;
pub mod views;

const SETTLE_BUDGET: usize = 16;
const TAB_BUDGET: usize = 65;

impl<A: FixtureApp> core::fmt::Debug for Driver<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Driver")
            .field("cfg", &self.cfg)
            .finish_non_exhaustive()
    }
}

/// Configuration shared by every fixture.
#[derive(Clone, Copy, Debug)]
pub struct FixtureCfg {
    /// Family under capture.
    pub family: Family,
    /// Requested state.
    pub state: ComponentState,
    /// Frame size.
    pub size: TerminalSize,
    /// Widget origin inside the frame.
    pub origin: Origin,
    /// Colour capability.
    pub color: ColorSpec,
    /// Proof facet.
    pub facet: Facet,
}

impl FixtureCfg {
    /// Widget rectangle at the configured origin. Fade facets clamp the
    /// viewport to the exact proof height (3/4/11/12).
    #[must_use]
    pub fn widget_rect(self) -> Rect {
        let remaining = self.size.rows.saturating_sub(self.origin.y);
        let height = match self.facet {
            Facet::FadeViewport { height, .. } => height.min(remaining),
            _ => remaining,
        };
        Rect::new(
            self.origin.x,
            self.origin.y,
            self.size.cols.saturating_sub(self.origin.x),
            height,
        )
    }
}

/// A family fixture: a production `App` plus identity and state verification.
pub trait FixtureApp: App + core::fmt::Debug {
    /// Primary widget identity (programs resolve areas from this).
    fn fixture_id(&self) -> Id;
    /// True when the app observably reached `state`. Defaults to true for
    /// construction-guaranteed prop states; runtime states override this.
    fn check(&self, state: ComponentState) -> bool {
        let _ = state;
        true
    }
}

/// One program step, resolved against published `Id` areas at run time.
/// No text search, no frozen coordinates.
#[derive(Clone, Copy, Debug)]
pub enum Step {
    /// Tab until the fixture id is focused.
    TabTo,
    /// Unmodified key code.
    Key(KeyCode),
    /// Key code with modifiers.
    KeyMod(KeyCode, KeyModifiers),
    /// Type one literal character.
    TypeChar(char),
    /// Move the pointer to the fixture area centre.
    MoveTo,
    /// Move the pointer to a registered part centre (thumb, seam).
    MoveToPart(junie_tui::Part),
    /// Move the pointer to a registered keyed item part.
    MoveToItem(junie_tui::Part, junie_tui::ItemKey),
    /// Click a registered keyed item part.
    ClickItem(junie_tui::Part, junie_tui::ItemKey),
    /// Pointer Down on a registered keyed item part, held (no Up).
    DownHoldItem(junie_tui::Part, junie_tui::ItemKey),
    /// Click (Down then Up) the fixture area centre.
    Click,
    /// Pointer Down on the fixture area centre, held (no Up).
    DownHold,
    /// Pointer Down on a registered part centre, held (no Up).
    DownHoldPart(junie_tui::Part),
    /// Release the pointer on the fixture area centre.
    Up,
    /// Drag from the fixture centre `dx` cells right.
    DragRight(u16),
    /// Vertical wheel at the fixture centre (positive scrolls down).
    Wheel(i16),
    /// Tab until focus leaves the fixture id (resting states).
    TabAway,
    /// Production ticks.
    Ticks(usize),
}

/// Facet program: real inputs that reach a fade position. `Top` and `Fits`
/// hold at construction (offset zero / short content); `Middle` and `Bottom`
/// scroll there. Content is 40 rows at heights 3-12, so `Down` x12 lands
/// strictly inside by construction and `End`/large wheel clamp at the end.
#[must_use]
pub fn facet_program(family: Family, facet: Facet) -> Vec<Step> {
    use crate::expansion::FadePosition as P;
    let Facet::FadeViewport { position, .. } = facet else {
        return Vec::new();
    };
    // The text area opens with its caret at the document end, so its
    // positions are caret-relative (Up clamps at line 0, Down at line 39).
    if family == Family::TextArea {
        let ups = core::iter::repeat_n(Step::Key(KeyCode::Up), 40);
        let downs = |count: usize| core::iter::repeat_n(Step::Key(KeyCode::Down), count);
        return match position {
            P::Fits => Vec::new(),
            P::Top => core::iter::once(Step::TabTo).chain(ups).collect(),
            P::Middle => core::iter::once(Step::TabTo)
                .chain(ups)
                .chain(downs(20))
                .collect(),
            P::Bottom => core::iter::once(Step::TabTo).chain(downs(40)).collect(),
        };
    }
    match position {
        P::Fits | P::Top => Vec::new(),
        P::Middle => {
            if family == Family::ScrollStateRegion {
                vec![Step::MoveTo, Step::Wheel(2)]
            } else {
                let mut steps = vec![Step::TabTo];
                steps.extend(core::iter::repeat_n(Step::Key(KeyCode::Down), 12));
                steps
            }
        }
        P::Bottom => {
            if family == Family::ScrollStateRegion {
                vec![Step::MoveTo, Step::Wheel(100)]
            } else {
                vec![Step::TabTo, Step::Key(KeyCode::End)]
            }
        }
    }
}

/// State program: real inputs that reach `state` for `family`.
#[must_use]
pub fn program_for(family: Family, state: ComponentState) -> Vec<Step> {
    use ComponentState as S;
    match state {
        // Resting frames: Tab focus away from the widget so the rest
        // paint is genuinely unfocused (every fixture also hosts the
        // phantom stop below, exactly like a multi-stop production app).
        S::Base
        | S::Disabled
        | S::Active
        | S::Inactive
        | S::Busy
        | S::Loading
        | S::Error
        | S::Empty => vec![Step::TabAway],
        // Read-only code refuses the edit attempt: the observed frame is
        // the focused editor with no edit session, and `check` proves the
        // refusal. Other read-only widgets rest unfocused.
        S::ReadOnly => {
            if family == Family::CodeEditor {
                vec![Step::TabTo, Step::Key(KeyCode::Enter)]
            } else {
                vec![Step::TabAway]
            }
        }
        S::Focus => vec![Step::TabTo],
        S::Hover => hover_program(family),
        S::Pressed => pressed_program(family),
        S::Current => current_program(family),
        S::Selected => selected_program(family),
        S::Editing => editing_program(family),
    }
}

fn hover_program(family: Family) -> Vec<Step> {
    use junie_tui::{ItemKey, Part};
    match family {
        Family::ScrollStateRegion => vec![Step::MoveToPart(Part::THUMB)],
        Family::SplitPane => vec![Step::MoveToPart(Part::SEAM)],
        Family::Chips => vec![Step::MoveToItem(Part::LABEL, ItemKey::index(0))],
        Family::Tabs => vec![Step::MoveToItem(Part::TAB, ItemKey::text("General"))],
        Family::MenuContextMenubar => vec![Step::MoveToItem(Part::TITLE, ItemKey::index(0))],
        _ => vec![Step::MoveTo],
    }
}

fn pressed_program(family: Family) -> Vec<Step> {
    use junie_tui::{ItemKey, Part};
    match family {
        Family::ScrollStateRegion => vec![
            Step::MoveToPart(Part::THUMB),
            Step::DownHoldPart(Part::THUMB),
        ],
        Family::SplitPane => vec![Step::MoveToPart(Part::SEAM), Step::DownHoldPart(Part::SEAM)],
        Family::Chips => vec![
            Step::MoveToItem(Part::LABEL, ItemKey::index(0)),
            Step::DownHoldItem(Part::LABEL, ItemKey::index(0)),
        ],
        Family::Tabs => vec![
            Step::MoveToItem(Part::TAB, ItemKey::text("General")),
            Step::DownHoldItem(Part::TAB, ItemKey::text("General")),
        ],
        _ => vec![Step::MoveTo, Step::DownHold],
    }
}

fn current_program(family: Family) -> Vec<Step> {
    match family {
        Family::Tabs | Family::Wizard => vec![Step::TabTo, Step::Key(KeyCode::Right)],
        Family::Completion => vec![Step::Key(KeyCode::Down)],
        Family::PickerChain => Vec::new(),
        _ => vec![Step::TabTo, Step::Key(KeyCode::Down)],
    }
}

fn selected_program(family: Family) -> Vec<Step> {
    match family {
        Family::TextInput
        | Family::TextArea
        | Family::CodeEditor
        | Family::DiffView
        | Family::TextViewport => {
            vec![Step::TabTo, Step::Click, Step::DragRight(10)]
        }
        Family::Tabs => vec![Step::TabTo, Step::Key(KeyCode::Right)],
        Family::CheckboxToggle => vec![
            Step::TabTo,
            Step::Key(KeyCode::Char(' ')),
            Step::Key(KeyCode::Tab),
            Step::Key(KeyCode::Char(' ')),
        ],
        // ChipBar keyboard bindings do not fire headless (production
        // behavior); the pointer path toggles through the real update.
        Family::Chips => vec![Step::ClickItem(
            junie_tui::Part::LABEL,
            junie_tui::ItemKey::index(0),
        )],
        Family::RadioGroup => {
            vec![
                Step::TabTo,
                Step::Key(KeyCode::Down),
                Step::Key(KeyCode::Enter),
            ]
        }
        // Single-mode lists and grids choose on Space (Enter activates).
        Family::List | Family::NavList | Family::Grid | Family::Table => {
            vec![
                Step::TabTo,
                Step::Key(KeyCode::Down),
                Step::Key(KeyCode::Char(' ')),
            ]
        }
        // Tree Space on a parent toggles expansion; the second Down reaches
        // the first leaf, which Space chooses.
        Family::Tree => vec![
            Step::TabTo,
            Step::Key(KeyCode::Down),
            Step::Key(KeyCode::Down),
            Step::Key(KeyCode::Char(' ')),
        ],
        Family::Completion => vec![Step::Key(KeyCode::Down), Step::Key(KeyCode::Enter)],
        _ => vec![Step::TabTo, Step::Key(KeyCode::Enter)],
    }
}

fn editing_program(family: Family) -> Vec<Step> {
    match family {
        Family::Grid => vec![
            Step::TabTo,
            Step::Key(KeyCode::Down),
            Step::Key(KeyCode::Enter),
        ],
        Family::CodeEditor => vec![Step::TabTo, Step::Key(KeyCode::Enter)],
        // Form commits on focus-out: typing then Tab reaches dirty.
        Family::Form => vec![Step::TabTo, Step::TypeChar('x'), Step::Key(KeyCode::Tab)],
        _ => vec![Step::TabTo, Step::TypeChar('x')],
    }
}

/// Headless production session driving one fixture.
pub struct Driver<A: FixtureApp> {
    runtime: Runtime<A>,
    buffer: Buffer,
    cfg: FixtureCfg,
}

impl<A: FixtureApp> Driver<A> {
    /// Construct, bootstrap, and present a fresh production fixture.
    pub fn open(app: A, cfg: FixtureCfg) -> Result<Self, AdapterError> {
        let theme = Theme::junie().downgrade(cfg.color.level());
        let mut runtime = Runtime::new(app, theme);
        let _ = runtime.initialize();
        let mut driver = Self {
            runtime,
            buffer: Buffer::empty(cfg.size.rect()),
            cfg,
        };
        driver.present()?;
        Ok(driver)
    }

    /// Fixture identity.
    #[must_use]
    pub fn fixture_id(&self) -> Id {
        self.runtime.app().fixture_id()
    }

    /// Present frames until the runtime settles.
    pub fn present(&mut self) -> Result<(), AdapterError> {
        for _ in 0..SETTLE_BUDGET {
            if self.runtime.needs_settle() {
                drop(self.runtime.settle());
            }
            self.buffer = Buffer::empty(self.cfg.size.rect());
            self.runtime
                .draw_buffer(self.cfg.size.rect(), &mut self.buffer)
                .commit_presented();
            if !self.runtime.needs_settle() && !self.runtime.needs_present() {
                return Ok(());
            }
        }
        Err(AdapterError::StateUnreachable {
            message: "fixture present/settle did not finish".to_owned(),
        })
    }

    /// Handle one production input, then present.
    pub fn handle(&mut self, input: Input) -> Result<(), AdapterError> {
        if self.runtime.needs_present() || self.runtime.needs_settle() {
            self.present()?;
        }
        drop(
            self.runtime
                .handle(input)
                .map_err(|_| AdapterError::StateUnreachable {
                    message: "runtime input pending present/settle".to_owned(),
                })?,
        );
        self.present()
    }

    /// Tab until the fixture id is focused.
    pub fn tab_to(&mut self) -> Result<(), AdapterError> {
        self.present()?;
        let id = self.fixture_id();
        for _ in 0..TAB_BUDGET {
            if self.runtime.focus() == Some(id) {
                return Ok(());
            }
            self.key_code(KeyCode::Tab)?;
            if self.runtime.focus() == Some(id) {
                return Ok(());
            }
        }
        Err(AdapterError::StateUnreachable {
            message: format!("Tab never focused {id:?}"),
        })
    }

    /// Tab until focus leaves the fixture id. Widgets that trap Tab for
    /// their own keymap (filter query, completion) are left the way real
    /// users leave them: a click on the phantom corner stop.
    pub fn tab_away(&mut self) -> Result<(), AdapterError> {
        self.present()?;
        let id = self.fixture_id();
        for _ in 0..4 {
            if self.runtime.focus() != Some(id) {
                return Ok(());
            }
            self.key_code(KeyCode::Tab)?;
            if self.runtime.focus() != Some(id) {
                return Ok(());
            }
        }
        if self.runtime.focus() == Some(id) {
            let corner = Position::new(
                self.cfg.size.cols.saturating_sub(1),
                self.cfg.size.rows.saturating_sub(1),
            );
            self.handle(Input::Mouse(Mouse {
                kind: MouseKind::Down,
                pos: corner,
                mods: KeyModifiers::NONE,
            }))?;
            self.handle(Input::Mouse(Mouse {
                kind: MouseKind::Up,
                pos: corner,
                mods: KeyModifiers::NONE,
            }))?;
        }
        if self.runtime.focus() != Some(id) {
            return Ok(());
        }
        Err(AdapterError::StateUnreachable {
            message: format!("Tab never left {id:?}"),
        })
    }

    /// Unmodified key code.
    pub fn key_code(&mut self, code: KeyCode) -> Result<(), AdapterError> {
        self.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }))
    }

    /// Centre of the fixture's published area.
    fn centre(&mut self) -> Result<(u16, u16), AdapterError> {
        self.present()?;
        let id = self.fixture_id();
        let Some(area) = self.runtime.area_of(id) else {
            return Err(AdapterError::UnaddressableId);
        };
        Ok(centre(area))
    }

    /// Text anchor: two cells into the area on its second row, so
    /// selection drags start on text rather than whitespace or rows past
    /// the content end.
    fn text_anchor(&mut self) -> Result<(u16, u16), AdapterError> {
        self.present()?;
        let id = self.fixture_id();
        let Some(area) = self.runtime.area_of(id) else {
            return Err(AdapterError::UnaddressableId);
        };
        let last = area.bottom().saturating_sub(1);
        Ok((area.x.saturating_add(2), area.y.saturating_add(1).min(last)))
    }

    /// Centre of a registered part area.
    fn part_centre(&mut self, part: junie_tui::Part) -> Result<(u16, u16), AdapterError> {
        self.present()?;
        let id = self.fixture_id();
        let part_ref = junie_tui::PartRef::of(part);
        let Some(area) = self.runtime.area_of_part(id, part_ref) else {
            return Err(AdapterError::UnaddressableId);
        };
        Ok(centre(area))
    }

    /// Pointer event at the fixture centre.
    pub fn pointer(&mut self, kind: MouseKind) -> Result<(), AdapterError> {
        let (x, y) = self.centre()?;
        self.handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        }))
    }

    /// Pointer event at a registered part centre.
    pub fn part_pointer(
        &mut self,
        part: junie_tui::Part,
        kind: MouseKind,
    ) -> Result<(), AdapterError> {
        let (x, y) = self.part_centre(part)?;
        self.handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        }))
    }

    /// Centre of a registered keyed item part.
    fn item_centre(
        &mut self,
        part: junie_tui::Part,
        key: junie_tui::ItemKey,
    ) -> Result<(u16, u16), AdapterError> {
        self.present()?;
        let id = self.fixture_id();
        let part_ref = junie_tui::PartRef::item(part, key);
        let Some(area) = self.runtime.area_of_part(id, part_ref) else {
            return Err(AdapterError::UnaddressableId);
        };
        Ok(centre(area))
    }

    /// Pointer event at a registered keyed item part.
    pub fn item_pointer(
        &mut self,
        part: junie_tui::Part,
        key: junie_tui::ItemKey,
        kind: MouseKind,
    ) -> Result<(), AdapterError> {
        let (x, y) = self.item_centre(part, key)?;
        self.handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        }))
    }

    /// Execute one program step.
    pub fn step(&mut self, step: Step) -> Result<(), AdapterError> {
        match step {
            Step::TabTo => self.tab_to(),
            Step::Key(code) => self.key_code(code),
            Step::KeyMod(code, mods) => self.handle(Input::Key(Key { code, mods })),
            Step::TypeChar(value) => self.key_code(KeyCode::Char(value)),
            Step::MoveTo => self.pointer(MouseKind::Move),
            Step::MoveToPart(part) => self.part_pointer(part, MouseKind::Move),
            Step::MoveToItem(part, key) => self.item_pointer(part, key, MouseKind::Move),
            Step::ClickItem(part, key) => {
                self.item_pointer(part, key, MouseKind::Down)?;
                self.item_pointer(part, key, MouseKind::Up)
            }
            Step::DownHoldItem(part, key) => self.item_pointer(part, key, MouseKind::Down),
            Step::Click => {
                self.pointer(MouseKind::Down)?;
                self.pointer(MouseKind::Up)
            }
            Step::DownHold => self.pointer(MouseKind::Down),
            Step::DownHoldPart(part) => self.part_pointer(part, MouseKind::Down),
            Step::Up => self.pointer(MouseKind::Up),
            Step::DragRight(dx) => {
                let (x, y) = self.text_anchor()?;
                self.handle(Input::Mouse(Mouse {
                    kind: MouseKind::Down,
                    pos: Position::new(x, y),
                    mods: KeyModifiers::NONE,
                }))?;
                self.handle(Input::Mouse(Mouse {
                    kind: MouseKind::Drag,
                    pos: Position::new(x.saturating_add(dx), y),
                    mods: KeyModifiers::NONE,
                }))?;
                self.handle(Input::Mouse(Mouse {
                    kind: MouseKind::Up,
                    pos: Position::new(x.saturating_add(dx), y),
                    mods: KeyModifiers::NONE,
                }))
            }
            Step::Wheel(delta) => {
                let (x, y) = self.centre()?;
                self.handle(Input::Mouse(Mouse {
                    kind: MouseKind::Wheel(junie_tui::Axis::V, delta),
                    pos: Position::new(x, y),
                    mods: KeyModifiers::NONE,
                }))
            }
            Step::TabAway => self.tab_away(),
            Step::Ticks(count) => {
                for _ in 0..count {
                    self.handle(Input::Tick)?;
                }
                Ok(())
            }
        }
    }

    /// Complete observation of the last presented frame.
    pub fn observe(&self) -> Result<FrameObservation, AdapterError> {
        let hits = self
            .runtime
            .registry()
            .regions()
            .iter()
            .map(|region| HitObs {
                owner: region.owner,
                x: region.area.x,
                y: region.area.y,
                width: region.area.width,
                height: region.area.height,
                layer: region.layer,
            })
            .collect();
        FrameObservation::capture(
            self.cfg.family,
            self.cfg.state,
            self.cfg.size,
            self.cfg.origin,
            self.cfg.color,
            self.cfg.facet,
            &self.buffer,
            self.runtime.cursor(),
            self.runtime.focus(),
            self.runtime.hover(),
            hits,
        )
    }

    /// Runtime moment (monotonic; observed, never asserted across runs).
    #[must_use]
    pub fn now(&self) -> Moment {
        self.runtime.now()
    }
}

fn centre(area: Rect) -> (u16, u16) {
    (
        area.x
            .saturating_add(area.width / 2)
            .min(area.right().saturating_sub(1)),
        area.y
            .saturating_add(area.height / 2)
            .min(area.bottom().saturating_sub(1)),
    )
}

/// Capture one expanded case through its production fixture.
pub fn capture(case: &ExpandedCase) -> Result<FrameObservation, AdapterError> {
    if disposition(case.family).lane != Lane::Direct {
        return Err(AdapterError::ArchitectureHasNoFrame {
            family: case.family,
        });
    }
    if !crate::states::applicable_states(case.family).contains(&case.state) {
        return Err(AdapterError::StateNotApplicable {
            family: case.family,
            state: case.state.token(),
        });
    }
    let cfg = FixtureCfg {
        family: case.family,
        state: case.state,
        size: case.size,
        origin: case.origin,
        color: case.color,
        facet: case.facet,
    };
    match case.family {
        Family::Button => drive(chrome::ButtonFixture::new(cfg), cfg),
        Family::Brand => drive(chrome::BrandFixture::new(cfg), cfg),
        Family::CheckboxToggle => drive(chrome::CheckboxFixture::new(cfg), cfg),
        Family::RadioGroup => drive(chrome::RadioFixture::new(cfg), cfg),
        Family::Chips => drive(chrome::ChipsFixture::new(cfg), cfg),
        Family::EmptyReadiness => drive(chrome::EmptyFixture::new(cfg), cfg),
        Family::ProgressBar => drive(chrome::ProgressFixture::new(cfg), cfg),
        Family::Spinner => drive(chrome::SpinnerFixture::new(cfg), cfg),
        Family::Meter => drive(chrome::MeterFixture::new(cfg), cfg),
        Family::StatusbarSegments => drive(chrome::StatusFixture::new(cfg), cfg),
        Family::Hintbar => drive(chrome::HintFixture::new(cfg), cfg),
        Family::Keyhint => drive(chrome::KeyHintFixture::new(cfg), cfg),
        Family::FieldChrome => drive(fields::FieldFixture::new(cfg), cfg),
        Family::TextInput => drive(fields::InputFixture::new(cfg), cfg),
        Family::TextArea => drive(fields::AreaFixture::new(cfg), cfg),
        Family::SecretValidation => drive(fields::SecretFixture::new(cfg), cfg),
        Family::Select => drive(fields::SelectFixture::new(cfg), cfg),
        Family::Form => drive(fields::FormFixture::new(cfg), cfg),
        Family::List => drive(lists::ListFixture::new(cfg), cfg),
        Family::FilterList => drive(lists::FilterFixture::new(cfg), cfg),
        Family::NavList => drive(lists::NavFixture::new(cfg), cfg),
        Family::Tree => drive(lists::TreeFixture::new(cfg), cfg),
        Family::Steps => drive(lists::StepsFixture::new(cfg), cfg),
        Family::Tabs => drive(lists::TabsFixture::new(cfg), cfg),
        Family::Props => drive(lists::PropsFixture::new(cfg), cfg),
        Family::PickerCommandPalette => drive(overlays::PickerFixture::new(cfg), cfg),
        Family::PickerChain => drive(overlays::ChainFixture::new(cfg), cfg),
        Family::Completion => drive(overlays::CompletionFixture::new(cfg), cfg),
        Family::Dialog => drive(overlays::DialogFixture::new(cfg), cfg),
        Family::MenuContextMenubar => drive(overlays::MenuFixture::new(cfg), cfg),
        Family::HelpOverlay => drive(overlays::HelpFixture::new(cfg), cfg),
        Family::Wizard => drive(overlays::WizardFixture::new(cfg), cfg),
        Family::Grid => drive(views::GridFixture::new(cfg), cfg),
        Family::Table => drive(views::TableFixture::new(cfg), cfg),
        Family::CodeEditor => drive(views::CodeFixture::new(cfg), cfg),
        Family::DiffView => drive(views::DiffFixture::new(cfg), cfg),
        Family::TextViewport => drive(views::ViewportFixture::new(cfg), cfg),
        Family::ScrollPanel => drive(views::ScrollPanelFixture::new(cfg), cfg),
        Family::Panel => drive(views::PanelFixture::new(cfg), cfg),
        Family::SplitPane => drive(views::SplitFixture::new(cfg), cfg),
        Family::ScrollStateRegion => drive(views::ScrollRegionFixture::new(cfg), cfg),
        Family::TooSmall => drive(views::TooSmallFixture::new(cfg), cfg),
        _ => Err(AdapterError::ArchitectureHasNoFrame {
            family: case.family,
        }),
    }
}

fn drive<A: FixtureApp>(app: A, cfg: FixtureCfg) -> Result<FrameObservation, AdapterError> {
    let mut driver = Driver::open(WithOther::new(app, cfg.size), cfg)?;
    for step in program_for(cfg.family, cfg.state) {
        driver.step(step)?;
    }
    for step in facet_program(cfg.family, cfg.facet) {
        driver.step(step)?;
    }
    verify_runtime_state(&driver, cfg)?;
    driver.observe()
}

/// Phantom second focus stop shared by every fixture.
///
/// A lone widget always holds autofocus, so the unfocused rest paint would
/// be unreachable. The wrapper registers one extra 1x1 `Focusable` control
/// in the far corner through the production `register_control` API. It
/// paints nothing and owns no widget; it exists only so real Tab input can
/// move focus away from the fixture, exactly like any multi-stop app.
#[derive(Debug)]
pub struct WithOther<A: FixtureApp> {
    inner: A,
    size: TerminalSize,
}

impl<A: FixtureApp> WithOther<A> {
    /// Wrap a fixture.
    pub const fn new(inner: A, size: TerminalSize) -> Self {
        Self { inner, size }
    }

    /// Phantom stop identity.
    pub const fn other_id() -> Id {
        Id::root("oracle.components.other")
    }
}

impl<A: FixtureApp> App for WithOther<A> {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.inner.update(cx)
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.inner.draw(ui);
        ui.register_control(
            Self::other_id(),
            Rect::new(
                self.size.cols.saturating_sub(1),
                self.size.rows.saturating_sub(1),
                1,
                1,
            ),
            Focusability::Focusable,
        );
    }
}

impl<A: FixtureApp> FixtureApp for WithOther<A> {
    fn fixture_id(&self) -> Id {
        self.inner.fixture_id()
    }

    fn check(&self, state: ComponentState) -> bool {
        self.inner.check(state)
    }
}

fn verify_runtime_state<A: FixtureApp>(
    driver: &Driver<A>,
    cfg: FixtureCfg,
) -> Result<(), AdapterError> {
    match cfg.state {
        ComponentState::Focus if driver.runtime.focus().is_none() => {
            return Err(AdapterError::StateUnreachable {
                message: format!("{} never gained focus", cfg.family.slug()),
            });
        }
        ComponentState::Hover if driver.runtime.hover().is_none() => {
            return Err(AdapterError::StateUnreachable {
                message: format!("{} never gained hover", cfg.family.slug()),
            });
        }
        _ => {}
    }
    if !driver.runtime.app().check(cfg.state) {
        return Err(AdapterError::StateUnreachable {
            message: format!("{} did not reach {}", cfg.family.slug(), cfg.state.token()),
        });
    }
    Ok(())
}

/// Two independent captures of the same case for repeat-equality proof.
pub fn capture_repeat_pair(
    case: &ExpandedCase,
) -> Result<(FrameObservation, FrameObservation), AdapterError> {
    Ok((capture(case)?, capture(case)?))
}
