use std::time::Duration;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};
use termrock::core::keymap::NavKeys;
use termrock::theme::{ColorLevel, FgStep, Overlay, Role, StylePatch, Surface};
use termrock::{
    App, Button, Cx, Id, Input, ItemKey, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Part,
    PartRef, Response, Runtime, Theme, ToolItem, Toolbar, ToolbarAction, ToolbarActionEvent,
    ToolbarError, ToolbarMoreLook, ToolbarReferenceState, ToolbarState, ToolbarTip,
    ToolbarTooltipPolicy, Ui,
};

const TOOLBAR: Id = Id::root("termrock.tests.toolbar.root");
const OTHER_TOOLBAR: Id = Id::root("termrock.tests.toolbar.other-owner");
const OVERFLOW_MENU: Id = TOOLBAR.sub("more-menu");
const HOVER_TOOLTIP: Id = TOOLBAR.sub("hover-tooltip");
const THIRD_TOOLBAR: Id = Id::root("termrock.tests.toolbar.third-owner");
const NEXT: Id = Id::root("termrock.tests.toolbar.next");
const SCREEN: Rect = Rect::new(0, 0, 64, 8);
const REFERENCE_SCREEN: Rect = Rect::new(0, 0, 64, 24);
const DEFAULT_TOOLBAR_AREA: Rect = Rect::new(5, 1, 42, 1);
const DEFAULT_NEXT_AREA: Rect = Rect::new(5, 4, 12, 1);

#[derive(Clone, Copy)]
enum OverflowFixtureMode {
    /// Keep the original Stage1 row-focused controls in no-overflow mode.
    NoOverflowFixture,
    /// Exercise Toolbar's historical default, which enables overflow.
    Default,
}

struct ToolbarHost {
    toolbar_id: Id,
    items: Vec<ToolItem>,
    keys: Vec<ItemKey>,
    state: ToolbarState,
    events: Vec<ToolbarActionEvent>,
    toolbar_area: Rect,
    parent_clip: Option<Rect>,
    next_area: Rect,
    nav: NavKeys,
    wrap: bool,
    skip_disabled: bool,
    overflow_mode: OverflowFixtureMode,
    more_glyph: Option<String>,
    more_look: Option<ToolbarMoreLook>,
    more_danger_hover: Option<bool>,
    more_padding: u16,
    more_tip: ToolbarTip,
    tooltip_policy: ToolbarTooltipPolicy,
    more_surface: Option<Surface>,
    more_style_overlay: Option<&'static Overlay>,
    reference_probe: bool,
    reference_gallery: bool,
    draw_toolbar_snapshot: Option<Toolbar>,
    next_activations: usize,
    next: Button<'static>,
}

impl ToolbarHost {
    fn new(items: Vec<ToolItem>, keys: Vec<ItemKey>) -> Self {
        Self {
            toolbar_id: TOOLBAR,
            items,
            keys,
            state: ToolbarState::default(),
            events: Vec::new(),
            toolbar_area: DEFAULT_TOOLBAR_AREA,
            parent_clip: None,
            next_area: DEFAULT_NEXT_AREA,
            nav: NavKeys::DEFAULT,
            wrap: false,
            skip_disabled: true,
            overflow_mode: OverflowFixtureMode::NoOverflowFixture,
            more_glyph: None,
            more_look: None,
            more_danger_hover: None,
            more_padding: 1,
            more_tip: ToolbarTip::default(),
            tooltip_policy: ToolbarTooltipPolicy::DEFAULT,
            more_surface: None,
            more_style_overlay: None,
            reference_probe: false,
            reference_gallery: false,
            draw_toolbar_snapshot: None,
            next_activations: 0,
            next: Button::new(NEXT, "Outside"),
        }
    }

    fn toolbar(&self) -> Toolbar {
        let toolbar = Toolbar::new(self.toolbar_id, self.items.clone())
            .nav(self.nav)
            .wrap(self.wrap)
            .skip_disabled(self.skip_disabled);
        let toolbar = match self.overflow_mode {
            OverflowFixtureMode::NoOverflowFixture => toolbar.overflow(false),
            OverflowFixtureMode::Default => toolbar,
        };
        let toolbar = match self.more_look {
            Some(look) => toolbar.more_look(look),
            None => toolbar,
        };
        let toolbar = match self.more_danger_hover {
            Some(enabled) => toolbar.more_danger_hover(enabled),
            None => toolbar,
        };
        let toolbar = toolbar.more_padding(self.more_padding);
        let toolbar = match self.more_glyph.as_deref() {
            Some(glyph) => toolbar.more_glyph(glyph),
            None => toolbar,
        };
        toolbar
            .more_tip(self.more_tip.clone())
            .tooltip_policy(self.tooltip_policy)
            .try_item_keys(self.keys.clone())
            .expect("test keys match the configured items")
    }

    fn freeze_draw_model(&mut self) {
        self.draw_toolbar_snapshot = Some(self.toolbar());
    }

    fn release_draw_model(&mut self) {
        self.draw_toolbar_snapshot = None;
    }

    fn use_default_overflow(&mut self) {
        self.overflow_mode = OverflowFixtureMode::Default;
    }

    fn use_no_overflow_fixture(&mut self) {
        self.overflow_mode = OverflowFixtureMode::NoOverflowFixture;
    }

    fn tooltip_policy(&mut self, policy: ToolbarTooltipPolicy) {
        self.tooltip_policy = policy;
    }

    fn draw_toolbar_scope(&self, ui: &mut Ui<'_>, toolbar: &Toolbar) {
        if let Some(overlay) = self.more_style_overlay {
            ui.with_overlay(overlay, |ui| self.draw_toolbar_surface(ui, toolbar));
        } else {
            self.draw_toolbar_surface(ui, toolbar);
        }
    }

    fn draw_toolbar_surface(&self, ui: &mut Ui<'_>, toolbar: &Toolbar) {
        if let Some(surface) = self.more_surface {
            ui.with_surface(surface, |ui| self.draw_toolbar_clip(ui, toolbar));
        } else {
            self.draw_toolbar_clip(ui, toolbar);
        }
    }

    fn draw_toolbar_clip(&self, ui: &mut Ui<'_>, toolbar: &Toolbar) {
        if let Some(clip) = self.parent_clip {
            ui.with_area(clip, |nested| {
                toolbar.draw(nested, self.toolbar_area, &self.state);
            });
        } else {
            toolbar.draw(ui, self.toolbar_area, &self.state);
        }
    }

    fn more_tip(&mut self, tip: ToolbarTip) {
        self.more_tip = tip;
    }

    fn commit(&mut self, event: ToolbarActionEvent) {
        if let ToolbarAction::Toggled(_, on) = event.action
            && let Some(index) = self.keys.iter().position(|key| *key == event.key)
            && let Some(item) = self.items.get_mut(index)
        {
            item.on = Some(on);
        }
    }
}

impl App for ToolbarHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let toolbar = self.toolbar();
        let response = toolbar.update(cx, &mut self.state);
        let events = response
            .action_ref()
            .map(|actions| actions.as_slice().to_vec())
            .unwrap_or_default();
        for event in &events {
            self.commit(*event);
        }
        self.events.extend(events);

        let next = self.next.update(cx);
        if next.action_ref().is_some() {
            self.next_activations = self.next_activations.saturating_add(1);
        }
        response.erase() | next.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let toolbar = self
            .draw_toolbar_snapshot
            .clone()
            .unwrap_or_else(|| self.toolbar());
        self.draw_toolbar_scope(ui, &toolbar);
        if self.reference_probe {
            let reference = Toolbar::new(
                OTHER_TOOLBAR,
                vec![
                    ToolItem::new("Ref A"),
                    ToolItem::new("Reference-only B"),
                    ToolItem::new("Reference-only C"),
                ],
            )
            .overflow(true);
            ui.reference(None, |reference_ui| {
                reference.draw(reference_ui, self.toolbar_area, &self.state);
            });
        }
        if self.reference_gallery {
            for (index, reference_state) in ToolbarReferenceState::ALL.into_iter().enumerate() {
                let row = Rect::new(5, 7u16.saturating_add(index as u16), 29, 1);
                let _ = toolbar.draw_reference(ui, row, reference_state);
            }
        }
        self.next.draw(ui, self.next_area);
    }
}

struct MultiToolbarHost {
    toolbars: Vec<(Toolbar, ToolbarState, Rect)>,
}

impl App for MultiToolbarHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.toolbars
            .iter_mut()
            .fold(Response::ignored(), |response, (toolbar, state, _)| {
                response | toolbar.update(cx, state).erase()
            })
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        for (toolbar, state, area) in &self.toolbars {
            toolbar.draw(ui, *area, state);
        }
    }
}

fn keys(names: &[&str]) -> Vec<ItemKey> {
    names.iter().map(|name| ItemKey::text(name)).collect()
}

fn render<A: App>(runtime: &mut Runtime<A>, terminal: &mut Terminal<TestBackend>, screen: Rect) {
    let prepared = runtime.prepare(screen).expect("runtime prepares a frame");
    terminal
        .draw(|frame| {
            runtime
                .draw_prepared(prepared, frame)
                .expect("prepared frame matches terminal bounds");
        })
        .expect("test backend draws");
    runtime
        .acknowledge_presented(prepared)
        .expect("test frame is acknowledged");
}

fn ready_for_input<A: App>(
    runtime: &mut Runtime<A>,
    terminal: &mut Terminal<TestBackend>,
    screen: Rect,
) {
    if runtime.needs_present() || runtime.needs_settle() {
        render(runtime, terminal, screen);
    }
    assert!(
        !runtime.needs_present() && !runtime.needs_settle(),
        "runtime is ready for one native input"
    );
}

fn key<A: App>(
    runtime: &mut Runtime<A>,
    terminal: &mut Terminal<TestBackend>,
    screen: Rect,
    code: KeyCode,
) {
    ready_for_input(runtime, terminal, screen);
    let _ = runtime
        .handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }))
        .expect("key input is dispatched");
}

fn mouse<A: App>(
    runtime: &mut Runtime<A>,
    terminal: &mut Terminal<TestBackend>,
    screen: Rect,
    kind: MouseKind,
    pos: Position,
) {
    ready_for_input(runtime, terminal, screen);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind,
            pos,
            mods: KeyModifiers::NONE,
        }))
        .expect("pointer input is dispatched");
}

fn click<A: App>(
    runtime: &mut Runtime<A>,
    terminal: &mut Terminal<TestBackend>,
    screen: Rect,
    pos: Position,
) {
    mouse(runtime, terminal, screen, MouseKind::Down, pos);
    render(runtime, terminal, screen);
    mouse(runtime, terminal, screen, MouseKind::Up, pos);
    render(runtime, terminal, screen);
}

fn advance<A: App>(
    runtime: &mut Runtime<A>,
    terminal: &mut Terminal<TestBackend>,
    screen: Rect,
    millis: u64,
) {
    runtime
        .advance_to(runtime.now().saturating_add(Duration::from_millis(millis)))
        .expect("injected runtime clock advances monotonically");
    render(runtime, terminal, screen);
}

fn hover_part<A: App>(
    runtime: &mut Runtime<A>,
    terminal: &mut Terminal<TestBackend>,
    screen: Rect,
    owner: Id,
    part: PartRef,
) {
    let area = runtime
        .area_of_part(owner, part)
        .expect("hover target has committed native geometry");
    mouse(
        runtime,
        terminal,
        screen,
        MouseKind::Move,
        Position::new(area.x.saturating_add(area.width / 2), area.y),
    );
    render(runtime, terminal, screen);
}

fn screen_text(terminal: &Terminal<TestBackend>) -> String {
    let buffer = terminal.backend().buffer();
    let mut text = String::new();
    for y in 0..SCREEN.height {
        for x in 0..SCREEN.width {
            if let Some(cell) = buffer.cell(Position::new(x, y)) {
                text.push_str(cell.symbol());
            }
        }
        text.push('\n');
    }
    text
}

fn screen_row_text(terminal: &Terminal<TestBackend>, y: u16, width: u16) -> String {
    let buffer = terminal.backend().buffer();
    let mut text = String::new();
    for x in 0..width {
        if let Some(cell) = buffer.cell(Position::new(x, y)) {
            text.push_str(cell.symbol());
        }
    }
    text
}

fn runtime_for<A: App>(host: A) -> (Runtime<A>, Terminal<TestBackend>) {
    runtime_for_theme(host, Theme::junie())
}

fn runtime_for_theme<A: App>(host: A, theme: Theme) -> (Runtime<A>, Terminal<TestBackend>) {
    runtime_for_theme_on(host, theme, SCREEN)
}

fn runtime_for_theme_on<A: App>(
    host: A,
    theme: Theme,
    screen: Rect,
) -> (Runtime<A>, Terminal<TestBackend>) {
    (
        Runtime::new(host, theme),
        Terminal::new(TestBackend::new(screen.width, screen.height)).unwrap(),
    )
}

fn theme_with_overflow_glyph(role: termrock::theme::GlyphRole) -> Theme {
    let mut theme = Theme::junie();
    theme
        .recipes
        .get_mut(termrock::theme::Family::BUTTON)
        .variant_mut(termrock::theme::Variant::SUBTLE)
        .entry(Part::OVERFLOW)
        .glyph = termrock::theme::Slot::Set(role);
    theme
}

fn theme_with_hovered_overflow_glyph(role: termrock::theme::GlyphRole) -> Theme {
    let mut theme = Theme::junie();
    theme
        .design
        .glyphs
        .set(termrock::theme::GlyphRole::OverflowRight, "界");
    theme
        .recipes
        .get_mut(termrock::theme::Family::BUTTON)
        .variant_mut(termrock::theme::Variant::SUBTLE)
        .entry(Part::OVERFLOW)
        .when(
            termrock::core::response::StateFlags::HOVERED,
            termrock::theme::StylePatch::new().set_glyph(role),
        );
    theme
}

const CLOSE_STYLE_SELECTOR: (termrock::theme::Family, termrock::theme::Variant) =
    Toolbar::more_style_selector(Some(ToolbarMoreLook::Close));
const TITLE_STYLE_SELECTOR: (termrock::theme::Family, termrock::theme::Variant) =
    Toolbar::more_style_selector(Some(ToolbarMoreLook::Title));
const DANGER_STYLE_SELECTOR: (termrock::theme::Family, termrock::theme::Variant) =
    Toolbar::more_style_selector(None);

static CLOSE_STYLE_SCOPE_RULES: [termrock::theme::OverlayRule; 4] = [
    (
        CLOSE_STYLE_SELECTOR.0,
        CLOSE_STYLE_SELECTOR.1,
        Part::OVERFLOW,
        termrock::StateFlags::HOVERED,
        StylePatch::new().set_fg(Role::Warning),
    ),
    (
        CLOSE_STYLE_SELECTOR.0,
        CLOSE_STYLE_SELECTOR.1,
        Part::OVERFLOW,
        termrock::StateFlags::HOVERED,
        StylePatch::new().set_bg(Role::Accent),
    ),
    (
        CLOSE_STYLE_SELECTOR.0,
        CLOSE_STYLE_SELECTOR.1,
        Part::OVERFLOW,
        termrock::StateFlags::PRESSED,
        StylePatch::new().set_fg(Role::Warning),
    ),
    (
        CLOSE_STYLE_SELECTOR.0,
        CLOSE_STYLE_SELECTOR.1,
        Part::OVERFLOW,
        termrock::StateFlags::PRESSED,
        StylePatch::new().set_bg(Role::Accent),
    ),
];
static CLOSE_STYLE_SCOPE: Overlay = Overlay::new(&CLOSE_STYLE_SCOPE_RULES);
const LOOK_HOVER_CLEAR: StylePatch = StylePatch::new().clear_fg();
const LOOK_PRESS_CLEAR: StylePatch = StylePatch::new().clear_fg().clear_bg();
static CLOSE_CLEAR_STYLE_SCOPE_RULES: [termrock::theme::OverlayRule; 6] = [
    (
        CLOSE_STYLE_SELECTOR.0,
        CLOSE_STYLE_SELECTOR.1,
        Part::CONTAINER,
        termrock::StateFlags::HOVERED,
        LOOK_HOVER_CLEAR,
    ),
    (
        CLOSE_STYLE_SELECTOR.0,
        CLOSE_STYLE_SELECTOR.1,
        Part::GUTTER,
        termrock::StateFlags::HOVERED,
        LOOK_HOVER_CLEAR,
    ),
    (
        CLOSE_STYLE_SELECTOR.0,
        CLOSE_STYLE_SELECTOR.1,
        Part::OVERFLOW,
        termrock::StateFlags::HOVERED,
        LOOK_HOVER_CLEAR,
    ),
    (
        CLOSE_STYLE_SELECTOR.0,
        CLOSE_STYLE_SELECTOR.1,
        Part::CONTAINER,
        termrock::StateFlags::PRESSED,
        LOOK_PRESS_CLEAR,
    ),
    (
        CLOSE_STYLE_SELECTOR.0,
        CLOSE_STYLE_SELECTOR.1,
        Part::GUTTER,
        termrock::StateFlags::PRESSED,
        LOOK_PRESS_CLEAR,
    ),
    (
        CLOSE_STYLE_SELECTOR.0,
        CLOSE_STYLE_SELECTOR.1,
        Part::OVERFLOW,
        termrock::StateFlags::PRESSED,
        LOOK_PRESS_CLEAR,
    ),
];
static CLOSE_CLEAR_STYLE_SCOPE: Overlay = Overlay::new(&CLOSE_CLEAR_STYLE_SCOPE_RULES);
static TITLE_METRIC_SCOPE_RULES: [termrock::theme::OverlayRule; 1] = [(
    TITLE_STYLE_SELECTOR.0,
    TITLE_STYLE_SELECTOR.1,
    Part::OVERFLOW,
    termrock::StateFlags::empty(),
    StylePatch::new()
        .set_glyph(termrock::theme::GlyphRole::Close)
        .set_size(2),
)];
static TITLE_METRIC_SCOPE: Overlay = Overlay::new(&TITLE_METRIC_SCOPE_RULES);

const DANGER_HOVER_CLEAR: StylePatch = StylePatch::new()
    .clear_fg()
    .set_bg(Role::Accent)
    .add(Modifier::BOLD)
    .set_underline(Role::Warning)
    .set_glyph(termrock::theme::GlyphRole::Close)
    .set_size(2);
const DANGER_PRESS_CLEAR: StylePatch = StylePatch::new()
    .clear_fg()
    .clear_bg()
    .add(Modifier::ITALIC)
    .set_underline(Role::Info)
    .set_glyph(termrock::theme::GlyphRole::Close)
    .set_size(2);
static DANGER_STYLE_SCOPE_RULES: [termrock::theme::OverlayRule; 6] = [
    (
        DANGER_STYLE_SELECTOR.0,
        DANGER_STYLE_SELECTOR.1,
        Part::CONTAINER,
        termrock::StateFlags::HOVERED,
        DANGER_HOVER_CLEAR,
    ),
    (
        DANGER_STYLE_SELECTOR.0,
        DANGER_STYLE_SELECTOR.1,
        Part::GUTTER,
        termrock::StateFlags::HOVERED,
        DANGER_HOVER_CLEAR,
    ),
    (
        DANGER_STYLE_SELECTOR.0,
        DANGER_STYLE_SELECTOR.1,
        Part::OVERFLOW,
        termrock::StateFlags::HOVERED,
        DANGER_HOVER_CLEAR,
    ),
    (
        DANGER_STYLE_SELECTOR.0,
        DANGER_STYLE_SELECTOR.1,
        Part::CONTAINER,
        termrock::StateFlags::PRESSED,
        DANGER_PRESS_CLEAR,
    ),
    (
        DANGER_STYLE_SELECTOR.0,
        DANGER_STYLE_SELECTOR.1,
        Part::GUTTER,
        termrock::StateFlags::PRESSED,
        DANGER_PRESS_CLEAR,
    ),
    (
        DANGER_STYLE_SELECTOR.0,
        DANGER_STYLE_SELECTOR.1,
        Part::OVERFLOW,
        termrock::StateFlags::PRESSED,
        DANGER_PRESS_CLEAR,
    ),
];
static DANGER_STYLE_SCOPE: Overlay = Overlay::new(&DANGER_STYLE_SCOPE_RULES);
const DANGER_HOVER_CLEAR_ONLY: StylePatch = StylePatch::new().clear_fg();
const DANGER_PRESS_CLEAR_ONLY: StylePatch = StylePatch::new().clear_fg().clear_bg();
static DANGER_CLEAR_STYLE_SCOPE_RULES: [termrock::theme::OverlayRule; 6] = [
    (
        DANGER_STYLE_SELECTOR.0,
        DANGER_STYLE_SELECTOR.1,
        Part::CONTAINER,
        termrock::StateFlags::HOVERED,
        DANGER_HOVER_CLEAR_ONLY,
    ),
    (
        DANGER_STYLE_SELECTOR.0,
        DANGER_STYLE_SELECTOR.1,
        Part::GUTTER,
        termrock::StateFlags::HOVERED,
        DANGER_HOVER_CLEAR_ONLY,
    ),
    (
        DANGER_STYLE_SELECTOR.0,
        DANGER_STYLE_SELECTOR.1,
        Part::OVERFLOW,
        termrock::StateFlags::HOVERED,
        DANGER_HOVER_CLEAR_ONLY,
    ),
    (
        DANGER_STYLE_SELECTOR.0,
        DANGER_STYLE_SELECTOR.1,
        Part::CONTAINER,
        termrock::StateFlags::PRESSED,
        DANGER_PRESS_CLEAR_ONLY,
    ),
    (
        DANGER_STYLE_SELECTOR.0,
        DANGER_STYLE_SELECTOR.1,
        Part::GUTTER,
        termrock::StateFlags::PRESSED,
        DANGER_PRESS_CLEAR_ONLY,
    ),
    (
        DANGER_STYLE_SELECTOR.0,
        DANGER_STYLE_SELECTOR.1,
        Part::OVERFLOW,
        termrock::StateFlags::PRESSED,
        DANGER_PRESS_CLEAR_ONLY,
    ),
];
static DANGER_CLEAR_STYLE_SCOPE: Overlay = Overlay::new(&DANGER_CLEAR_STYLE_SCOPE_RULES);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MoreInteraction {
    Rest,
    Hover,
    Pressed,
    Focused,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MoreCell {
    symbol: String,
    fg: Color,
    bg: Color,
    underline: Color,
    modifier: Modifier,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MoreSnapshot {
    area: Rect,
    cells: Vec<MoreCell>,
    hidden: Vec<ItemKey>,
    on_more: bool,
    toolbar_focus_entries: usize,
}

fn more_snapshot(
    theme: Theme,
    look: Option<ToolbarMoreLook>,
    danger_hover: Option<bool>,
    surface: Option<Surface>,
    overlay: Option<&'static Overlay>,
    interaction: MoreInteraction,
) -> MoreSnapshot {
    let item_keys = keys(&["visible", "hidden-one", "hidden-two"]);
    let mut host = ToolbarHost::new(
        vec![
            ToolItem::new("Visible"),
            ToolItem::new("Hidden One"),
            ToolItem::new("Hidden Two"),
        ],
        item_keys,
    );
    host.toolbar_area = Rect::new(5, 1, 16, 1);
    host.use_default_overflow();
    host.more_look = look;
    host.more_danger_hover = danger_hover;
    host.more_surface = surface;
    host.more_style_overlay = overlay;
    let (mut runtime, mut terminal) = runtime_for_theme(host, theme);
    render(&mut runtime, &mut terminal, SCREEN);

    let area = runtime
        .area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW))
        .expect("overflow publishes More geometry");
    match interaction {
        MoreInteraction::Rest => {}
        MoreInteraction::Hover => {
            hover_part(
                &mut runtime,
                &mut terminal,
                SCREEN,
                TOOLBAR,
                PartRef::of(Part::OVERFLOW),
            );
        }
        MoreInteraction::Pressed => {
            mouse(
                &mut runtime,
                &mut terminal,
                SCREEN,
                MouseKind::Down,
                Position::new(area.x.saturating_add(1), area.y),
            );
            render(&mut runtime, &mut terminal, SCREEN);
        }
        MoreInteraction::Focused => {
            focus_toolbar(&mut runtime, &mut terminal, SCREEN);
            key(&mut runtime, &mut terminal, SCREEN, KeyCode::End);
            render(&mut runtime, &mut terminal, SCREEN);
        }
    }

    let buffer = terminal.backend().buffer();
    let cells = area
        .positions()
        .map(|position| {
            let cell = buffer.cell(position).expect("More cell is in the buffer");
            MoreCell {
                symbol: cell.symbol().to_owned(),
                fg: cell.fg,
                bg: cell.bg,
                underline: cell.underline_color,
                modifier: cell.modifier,
            }
        })
        .collect();
    MoreSnapshot {
        area,
        cells,
        hidden: runtime.app().state.hidden_keys(),
        on_more: runtime.app().state.on_more(),
        toolbar_focus_entries: runtime
            .ring()
            .entries()
            .iter()
            .filter(|entry| entry.id == TOOLBAR)
            .count(),
    }
}

fn focus_toolbar(
    runtime: &mut Runtime<ToolbarHost>,
    terminal: &mut Terminal<TestBackend>,
    screen: Rect,
) {
    if runtime.focus() != Some(TOOLBAR) {
        key(runtime, terminal, screen, KeyCode::Tab);
        render(runtime, terminal, screen);
    }
    assert_eq!(runtime.focus(), Some(TOOLBAR));
}

fn more_glyph(snapshot: &MoreSnapshot) -> &MoreCell {
    snapshot
        .cells
        .iter()
        .find(|cell| cell.symbol == "»")
        .expect("More keeps the native overflow glyph")
}

fn more_role(theme: &Theme, role: Role, surface: Surface) -> Color {
    termrock::theme::resolve::bind_role(theme, role, surface)
        .expect("the Toolbar semantic role resolves to a color")
}

#[test]
fn more_default_and_explicit_false_keep_the_existing_button_recipe() {
    for interaction in [
        MoreInteraction::Rest,
        MoreInteraction::Hover,
        MoreInteraction::Pressed,
        MoreInteraction::Focused,
    ] {
        let unchanged = more_snapshot(Theme::junie(), None, None, None, None, interaction);
        let explicitly_disabled =
            more_snapshot(Theme::junie(), None, Some(false), None, None, interaction);
        assert_eq!(explicitly_disabled, unchanged);
    }
}

#[test]
fn more_looks_resolve_their_state_and_plate_roles_on_each_surface() {
    for theme in [
        Theme::junie(),
        Theme::paper().for_level(ColorLevel::Ansi256),
    ] {
        for (look, surface) in [
            (ToolbarMoreLook::Close, Surface::Canvas),
            (ToolbarMoreLook::Close, Surface::Field),
            (ToolbarMoreLook::Title, Surface::Canvas),
            (ToolbarMoreLook::Title, Surface::Field),
        ] {
            let scoped_surface = Some(surface);
            let rest = more_snapshot(
                theme.clone(),
                Some(look),
                Some(false),
                scoped_surface,
                None,
                MoreInteraction::Rest,
            );
            let focused = more_snapshot(
                theme.clone(),
                Some(look),
                Some(false),
                scoped_surface,
                None,
                MoreInteraction::Focused,
            );
            assert_eq!(
                focused.cells, rest.cells,
                "explicit looks suppress focus-only styling"
            );

            for (interaction, pressed, hovered) in [
                (MoreInteraction::Rest, false, false),
                (MoreInteraction::Hover, false, true),
                (MoreInteraction::Pressed, true, false),
            ] {
                let snapshot = if interaction == MoreInteraction::Rest {
                    rest.clone()
                } else {
                    more_snapshot(
                        theme.clone(),
                        Some(look),
                        Some(false),
                        scoped_surface,
                        None,
                        interaction,
                    )
                };
                let (fg_role, bg_role, bold) = match (look, pressed, hovered) {
                    (ToolbarMoreLook::Close, true, _) => {
                        (Role::CurrentSurface, Role::Fg(FgStep::Primary), false)
                    }
                    (ToolbarMoreLook::Close, false, true) => {
                        (Role::Fg(FgStep::Primary), Role::RaisedSurface, false)
                    }
                    (ToolbarMoreLook::Close, false, false) => {
                        (Role::Fg(FgStep::Secondary), Role::CurrentSurface, false)
                    }
                    (ToolbarMoreLook::Title, true, _) => {
                        (Role::Fg(FgStep::Primary), Role::RaisedSurface, true)
                    }
                    (ToolbarMoreLook::Title, false, true) => {
                        (Role::Fg(FgStep::Primary), Role::HoverSurface, false)
                    }
                    (ToolbarMoreLook::Title, false, false) => {
                        (Role::Fg(FgStep::Secondary), Role::CurrentSurface, false)
                    }
                };
                let glyph = more_glyph(&snapshot);
                assert_eq!(glyph.fg, more_role(&theme, fg_role, surface));
                assert_eq!(glyph.bg, more_role(&theme, bg_role, surface));
                assert_eq!(glyph.modifier.contains(Modifier::BOLD), bold);
                assert!(
                    snapshot
                        .cells
                        .iter()
                        .all(|cell| { cell.bg == more_role(&theme, bg_role, surface) }),
                    "the full padded plate uses the state background role"
                );
                assert_eq!(snapshot.area.width, 3);
            }
        }
    }
}

#[test]
fn more_pointer_danger_changes_only_the_resolved_ink_and_plate_channels() {
    let theme = Theme::junie();
    for look in [
        None,
        Some(ToolbarMoreLook::Close),
        Some(ToolbarMoreLook::Title),
    ] {
        for interaction in [
            MoreInteraction::Rest,
            MoreInteraction::Focused,
            MoreInteraction::Hover,
            MoreInteraction::Pressed,
        ] {
            let plain = more_snapshot(theme.clone(), look, Some(false), None, None, interaction);
            let danger = more_snapshot(theme.clone(), look, Some(true), None, None, interaction);
            assert_eq!(plain.area, danger.area);
            assert_eq!(plain.hidden, danger.hidden);
            if matches!(
                interaction,
                MoreInteraction::Rest | MoreInteraction::Focused
            ) {
                assert_eq!(danger.cells, plain.cells);
                continue;
            }

            assert_eq!(danger.cells.len(), plain.cells.len());
            for (actual, original) in danger.cells.iter().zip(&plain.cells) {
                assert_eq!(actual.symbol, original.symbol);
                assert_eq!(actual.modifier, original.modifier);
                assert_eq!(actual.underline, original.underline);
                if interaction == MoreInteraction::Hover {
                    assert_eq!(actual.bg, original.bg, "hover changes foreground only");
                }
            }
            let glyph = more_glyph(&danger);
            if interaction == MoreInteraction::Hover {
                assert_eq!(glyph.fg, theme.color.danger_soft);
            } else {
                assert_eq!(glyph.fg, theme.color.on_danger);
                assert_eq!(glyph.bg, theme.color.danger);
                assert!(
                    danger
                        .cells
                        .iter()
                        .all(|cell| cell.bg == theme.color.danger)
                );
                if look == Some(ToolbarMoreLook::Title) {
                    assert!(glyph.modifier.contains(Modifier::BOLD));
                }
            }
        }
    }
}

#[test]
fn more_look_supports_authored_variant_and_scoped_overrides() {
    let (family, variant) = CLOSE_STYLE_SELECTOR;
    let theme = Theme::junie()
        .define_variant(family, variant, |recipe| {
            recipe
                .part(Part::OVERFLOW)
                .base(StylePatch::new().set_fg(Role::Warning).set_bg(Role::Accent));
        })
        .override_variant(family, variant, |edit| {
            edit.part(Part::OVERFLOW).when(
                termrock::StateFlags::HOVERED,
                StylePatch::new().set_fg(Role::Info),
            );
            edit.part(Part::OVERFLOW).when(
                termrock::StateFlags::PRESSED,
                StylePatch::new().set_bg(Role::Danger),
            );
        });

    let rest = more_snapshot(
        theme.clone(),
        Some(ToolbarMoreLook::Close),
        Some(false),
        None,
        None,
        MoreInteraction::Rest,
    );
    assert_eq!(more_glyph(&rest).fg, theme.color.warning);
    assert_eq!(more_glyph(&rest).bg, theme.color.accent);

    let authored_hover = more_snapshot(
        theme.clone(),
        Some(ToolbarMoreLook::Close),
        Some(true),
        None,
        None,
        MoreInteraction::Hover,
    );
    assert_eq!(more_glyph(&authored_hover).fg, theme.color.info);
    assert_eq!(more_glyph(&authored_hover).bg, theme.color.accent);

    let authored_press = more_snapshot(
        theme.clone(),
        Some(ToolbarMoreLook::Close),
        Some(true),
        None,
        None,
        MoreInteraction::Pressed,
    );
    assert_eq!(
        more_glyph(&authored_press).fg,
        theme.color.info,
        "the authored hover foreground remains active during press independently of the pressed background override"
    );
    assert_eq!(more_glyph(&authored_press).bg, theme.color.danger);

    let scoped_hover = more_snapshot(
        theme.clone(),
        Some(ToolbarMoreLook::Close),
        Some(true),
        None,
        Some(&CLOSE_STYLE_SCOPE),
        MoreInteraction::Hover,
    );
    assert_eq!(more_glyph(&scoped_hover).fg, Theme::junie().color.warning);
    assert_eq!(more_glyph(&scoped_hover).bg, Theme::junie().color.accent);

    let scoped_press = more_snapshot(
        theme,
        Some(ToolbarMoreLook::Close),
        Some(true),
        None,
        Some(&CLOSE_STYLE_SCOPE),
        MoreInteraction::Pressed,
    );
    assert_eq!(more_glyph(&scoped_press).fg, Theme::junie().color.warning);
    assert_eq!(more_glyph(&scoped_press).bg, Theme::junie().color.accent);
}

#[test]
fn explicit_look_authored_and_scoped_clears_override_danger_defaults() {
    let (family, variant) = CLOSE_STYLE_SELECTOR;
    let authored_clears = Theme::junie().override_variant(family, variant, |edit| {
        for part in [Part::CONTAINER, Part::GUTTER, Part::OVERFLOW] {
            edit.part(part)
                .when(termrock::StateFlags::HOVERED, LOOK_HOVER_CLEAR)
                .when(termrock::StateFlags::PRESSED, LOOK_PRESS_CLEAR);
        }
    });

    for interaction in [MoreInteraction::Hover, MoreInteraction::Pressed] {
        let cleared = more_snapshot(
            authored_clears.clone(),
            Some(ToolbarMoreLook::Close),
            Some(true),
            None,
            None,
            interaction,
        );
        let glyph = more_glyph(&cleared);
        assert_eq!(glyph.fg, Color::Reset);
        if interaction == MoreInteraction::Pressed {
            assert_eq!(glyph.bg, Color::Reset);
            assert!(cleared.cells.iter().all(|cell| cell.bg == Color::Reset));
        }
    }

    let scoped_set_channels = Theme::junie().define_variant(family, variant, |recipe| {
        for part in [Part::CONTAINER, Part::GUTTER, Part::OVERFLOW] {
            recipe
                .part(part)
                .base(StylePatch::new().set_fg(Role::Warning).set_bg(Role::Accent));
        }
    });
    for interaction in [MoreInteraction::Hover, MoreInteraction::Pressed] {
        let cleared = more_snapshot(
            scoped_set_channels.clone(),
            Some(ToolbarMoreLook::Close),
            Some(true),
            None,
            Some(&CLOSE_CLEAR_STYLE_SCOPE),
            interaction,
        );
        let glyph = more_glyph(&cleared);
        assert_eq!(glyph.fg, Color::Reset);
        if interaction == MoreInteraction::Hover {
            assert_eq!(glyph.bg, Theme::junie().color.accent);
            assert!(
                cleared
                    .cells
                    .iter()
                    .all(|cell| cell.bg == Theme::junie().color.accent)
            );
        } else {
            assert_eq!(glyph.bg, Color::Reset);
            assert!(cleared.cells.iter().all(|cell| cell.bg == Color::Reset));
        }
    }
}

#[test]
fn unset_danger_preserves_button_base_and_yields_to_specific_selector_rules() {
    let theme = Theme::junie()
        .override_variant(
            termrock::theme::Family::BUTTON,
            termrock::theme::Variant::SUBTLE,
            |edit| {
                edit.part(Part::OVERFLOW)
                    .base(StylePatch::new().set_fg(Role::Warning).set_bg(Role::Accent))
                    .when(
                        termrock::StateFlags::HOVERED,
                        StylePatch::new().set_bg(Role::Info),
                    );
            },
        )
        .override_variant(DANGER_STYLE_SELECTOR.0, DANGER_STYLE_SELECTOR.1, |edit| {
            edit.part(Part::OVERFLOW)
                .when(
                    termrock::StateFlags::HOVERED,
                    StylePatch::new().set_fg(Role::Success),
                )
                .when(termrock::StateFlags::PRESSED, StylePatch::new().clear_bg());
        });

    let rest = more_snapshot(
        theme.clone(),
        None,
        Some(false),
        None,
        None,
        MoreInteraction::Rest,
    );
    let focused = more_snapshot(
        theme.clone(),
        None,
        Some(false),
        None,
        None,
        MoreInteraction::Focused,
    );
    assert_eq!(more_glyph(&rest).fg, theme.color.warning);
    assert_eq!(more_glyph(&rest).bg, theme.color.accent);

    let danger_rest = more_snapshot(
        theme.clone(),
        None,
        Some(true),
        None,
        None,
        MoreInteraction::Rest,
    );
    let danger_focused = more_snapshot(
        theme.clone(),
        None,
        Some(true),
        None,
        None,
        MoreInteraction::Focused,
    );
    assert_eq!(danger_rest.cells, rest.cells);
    assert_eq!(danger_focused.cells, focused.cells);

    let plain_hover = more_snapshot(
        theme.clone(),
        None,
        Some(false),
        None,
        None,
        MoreInteraction::Hover,
    );
    let danger_hover = more_snapshot(
        theme.clone(),
        None,
        Some(true),
        None,
        None,
        MoreInteraction::Hover,
    );
    assert_eq!(more_glyph(&plain_hover).bg, theme.color.info);
    assert_eq!(more_glyph(&danger_hover).bg, theme.color.info);
    assert_eq!(more_glyph(&danger_hover).fg, theme.color.success);

    let plain_press = more_snapshot(
        theme.clone(),
        None,
        Some(false),
        None,
        None,
        MoreInteraction::Pressed,
    );
    let danger_press = more_snapshot(
        theme.clone(),
        None,
        Some(true),
        None,
        None,
        MoreInteraction::Pressed,
    );
    assert_eq!(
        more_glyph(&danger_press).fg,
        theme.color.success,
        "the authored hover foreground participates during press while the pressed danger background is independently cleared"
    );
    assert_eq!(more_glyph(&plain_press).bg, theme.color.info);
    assert_eq!(
        more_glyph(&danger_press).bg,
        theme.color.danger,
        "an overflow background clear inherits the pressed container plate"
    );
    assert!(
        danger_press
            .cells
            .iter()
            .all(|cell| cell.bg == theme.color.danger)
    );
}

#[test]
fn sparse_danger_clears_propagate_and_other_channels_stay_inert() {
    for interaction in [MoreInteraction::Hover, MoreInteraction::Pressed] {
        let full = more_snapshot(
            Theme::junie(),
            None,
            Some(true),
            None,
            Some(&DANGER_STYLE_SCOPE),
            interaction,
        );
        let clear_only = more_snapshot(
            Theme::junie(),
            None,
            Some(true),
            None,
            Some(&DANGER_CLEAR_STYLE_SCOPE),
            interaction,
        );
        assert_eq!(full, clear_only, "unsupported channels do not affect More");
        assert_eq!(full.area.width, 3, "size declarations do not change layout");
        assert_eq!(
            more_glyph(&full).symbol,
            "»",
            "glyph declarations are inert"
        );
        assert_eq!(more_glyph(&full).fg, Color::Reset);
        if interaction == MoreInteraction::Pressed {
            assert_eq!(more_glyph(&full).bg, Color::Reset);
        }
        assert!(full.cells.iter().all(|cell| cell.underline == Color::Reset));
        assert!(
            full.cells
                .iter()
                .all(|cell| !cell.modifier.contains(Modifier::ITALIC))
        );
    }
}

#[test]
fn danger_selector_keeps_profile_colors_across_terminal_levels() {
    for (level, custom) in [
        (ColorLevel::Ansi256, Color::Indexed(201)),
        (ColorLevel::Ansi16, Color::Indexed(9)),
        (ColorLevel::Mono, Color::White),
    ] {
        for (look, (family, variant)) in [
            (None, DANGER_STYLE_SELECTOR),
            (Some(ToolbarMoreLook::Close), CLOSE_STYLE_SELECTOR),
            (Some(ToolbarMoreLook::Title), TITLE_STYLE_SELECTOR),
        ] {
            let theme = Theme::junie()
                .define_variant_for_level(level, family, variant, |recipe| {
                    recipe
                        .part(Part::OVERFLOW)
                        .base(StylePatch::new().set_fg(Role::Custom(custom)));
                })
                .for_level(level);
            let expected = theme
                .resolve(
                    family,
                    variant,
                    Part::OVERFLOW,
                    termrock::StateFlags::HOVERED,
                    Surface::Canvas,
                )
                .style
                .as_style()
                .fg
                .expect("profile style sets its foreground");
            let snapshot =
                more_snapshot(theme, look, Some(true), None, None, MoreInteraction::Hover);
            assert_eq!(more_glyph(&snapshot).fg, expected, "{look:?}, {level:?}");
            assert_eq!(snapshot.area.width, 3);
            assert_eq!(more_glyph(&snapshot).symbol, "»");
        }
    }
}

#[test]
fn opt_in_more_reuses_native_overflow_geometry_menu_and_receipt_lifecycle() {
    let visible = ItemKey::text("opt-in-visible");
    let hidden = ItemKey::text("opt-in-hidden");
    let blocked = ItemKey::text("opt-in-blocked");
    let mut host = ToolbarHost::new(
        vec![
            ToolItem::new("Visible"),
            ToolItem::new("Hidden action"),
            ToolItem::new("Blocked").disabled(true),
        ],
        vec![visible, hidden, blocked],
    );
    host.toolbar_area = Rect::new(5, 1, 16, 1);
    host.use_default_overflow();
    host.more_look = Some(ToolbarMoreLook::Title);
    host.more_danger_hover = Some(true);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);

    assert_eq!(runtime.app().state.hidden_keys(), vec![hidden, blocked]);
    assert_eq!(
        runtime.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW)),
        Some(Rect::new(18, 1, 3, 1))
    );
    assert_eq!(
        runtime
            .ring()
            .entries()
            .iter()
            .filter(|entry| entry.id == TOOLBAR)
            .count(),
        1
    );

    focus_toolbar(&mut runtime, &mut terminal, SCREEN);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::End);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(runtime.app().state.on_more());
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(runtime.is_open(OVERFLOW_MENU));
    click_menu_row(&mut runtime, &mut terminal, SCREEN, 0);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.focus(), Some(TOOLBAR));
    assert!(!runtime.is_open(OVERFLOW_MENU));
    assert_eq!(runtime.app().events.len(), 1);
    assert_eq!(
        runtime.app().events.first().copied(),
        Some(ToolbarActionEvent {
            key: hidden,
            action: ToolbarAction::Activated(1),
        })
    );
}

fn row_part(key: ItemKey) -> PartRef {
    PartRef::item(Part::ROW, key)
}

fn menu_row_part(index: usize) -> PartRef {
    PartRef::item(Part::ROW, ItemKey::index(index))
}

fn click_menu_row(
    runtime: &mut Runtime<ToolbarHost>,
    terminal: &mut Terminal<TestBackend>,
    screen: Rect,
    index: usize,
) {
    let area = runtime
        .area_of_part(OVERFLOW_MENU, menu_row_part(index))
        .expect("overflow menu row has native geometry");
    click(
        runtime,
        terminal,
        screen,
        Position::new(area.x.saturating_add(area.width / 2), area.y),
    );
}

fn open_overflow_by_keyboard(
    runtime: &mut Runtime<ToolbarHost>,
    terminal: &mut Terminal<TestBackend>,
    screen: Rect,
) {
    focus_toolbar(runtime, terminal, screen);
    key(runtime, terminal, screen, KeyCode::End);
    render(runtime, terminal, screen);
    key(runtime, terminal, screen, KeyCode::Enter);
    render(runtime, terminal, screen);
    assert!(runtime.is_open(OVERFLOW_MENU));
}

#[test]
fn reference_gallery_uses_native_layout_states_and_is_inert() {
    let first = ItemKey::text("reference-bold");
    let second = ItemKey::text("reference-italic");
    let third = ItemKey::text("reference-code");
    let items = vec![
        ToolItem::new("Bold").toggle(false),
        ToolItem::new("Italic").toggle(false),
        ToolItem::new("Code").toggle(false),
    ];
    let item_keys = vec![first, second, third];

    let make_host = || {
        let mut host = ToolbarHost::new(items.clone(), item_keys.clone());
        host.toolbar_area = Rect::new(5, 1, 16, 1);
        host.use_default_overflow();
        host
    };
    let theme = Theme::junie();
    let (mut baseline, mut baseline_terminal) =
        runtime_for_theme_on(make_host(), theme.clone(), REFERENCE_SCREEN);
    render(&mut baseline, &mut baseline_terminal, REFERENCE_SCREEN);

    let mut probe_host = make_host();
    probe_host.reference_gallery = true;
    let (mut probe, mut probe_terminal) = runtime_for_theme_on(probe_host, theme, REFERENCE_SCREEN);
    render(&mut probe, &mut probe_terminal, REFERENCE_SCREEN);

    assert_eq!(probe.region_count(), baseline.region_count());
    assert_eq!(probe.ring().entries(), baseline.ring().entries());
    assert_eq!(probe.focus(), baseline.focus());
    assert_eq!(probe.hover(), baseline.hover());
    assert_eq!(probe.app().events, baseline.app().events);
    assert!(probe.app().events.is_empty());
    assert!(probe.app().items.iter().all(|item| item.on == Some(false)));
    assert!(probe.app().items.iter().all(|item| !item.disabled));
    assert_eq!(
        probe.app().state.cursor_key(),
        baseline.app().state.cursor_key()
    );
    assert_eq!(probe.app().state.on_more(), baseline.app().state.on_more());
    assert_eq!(
        probe.app().state.menu_open(),
        baseline.app().state.menu_open()
    );
    assert_eq!(probe.app().state.hidden_keys(), vec![second, third]);
    assert_eq!(
        probe.app().state.hidden_keys(),
        baseline.app().state.hidden_keys()
    );
    for key in item_keys {
        assert_eq!(
            probe.area_of_part(TOOLBAR, row_part(key)),
            baseline.area_of_part(TOOLBAR, row_part(key)),
            "reference rows do not add or replace live keyed areas"
        );
    }

    let row = |y| screen_row_text(&probe_terminal, y, REFERENCE_SCREEN.width);
    assert!(row(7).contains("Bold"));
    assert!(row(7).contains("Italic"));
    assert!(row(7).contains("Code"));
    let buffer = probe_terminal.backend().buffer();
    let label_style = |y| {
        buffer
            .cell(Position::new(17, y))
            .map(|cell| (cell.fg, cell.bg, cell.modifier))
            .expect("the second item's label is inside each reference row")
    };
    assert_ne!(
        label_style(8).1,
        label_style(7).1,
        "Hover applies the native hovered style to the second item"
    );
    assert_ne!(
        label_style(9).1,
        label_style(8).1,
        "Pressed applies the native pressed style to the second item"
    );
    assert_ne!(
        label_style(13).0,
        label_style(7).0,
        "Disabled applies the native disabled style to the second item"
    );
    assert_eq!(
        probe_terminal
            .backend()
            .buffer()
            .cell(Position::new(6, 7))
            .map(|cell| cell.symbol()),
        Some("○"),
        "Rest retains the configured false toggle value"
    );
    assert_eq!(
        probe_terminal
            .backend()
            .buffer()
            .cell(Position::new(14, 10))
            .map(|cell| cell.symbol()),
        Some("▏"),
        "Focused uses the inert Unicode focus cue on the second item"
    );
    assert_eq!(
        probe_terminal
            .backend()
            .buffer()
            .cell(Position::new(14, 11))
            .map(|cell| cell.symbol()),
        Some("▏"),
        "FocusedHover keeps the same source-backed target"
    );
    assert_eq!(
        probe_terminal
            .backend()
            .buffer()
            .cell(Position::new(6, 12))
            .map(|cell| cell.symbol()),
        Some("●"),
        "Selected turns on the first optional toggle only in its detached copy"
    );
    assert!(row(14).contains("»"), "Overflow uses native More geometry");
    assert!(row(14).contains("Italic"));
    assert!(!row(14).contains("Code"));
    assert!(
        row(15).contains("»"),
        "Narrow uses a sixteen-cell native row"
    );
    assert!(!row(15).contains("Italic"));
}

#[test]
fn reference_gallery_uses_ascii_focus_cue_and_handles_short_items() {
    let first = ItemKey::text("ascii-first");
    let second = ItemKey::text("ascii-second");
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("One"), ToolItem::new("Two")],
        vec![first, second],
    );
    host.reference_gallery = true;
    let theme = Theme::junie()
        .builder()
        .glyph(termrock::theme::GlyphRole::FocusBar, "f")
        .build();
    let (mut runtime, mut terminal) = runtime_for_theme_on(host, theme, REFERENCE_SCREEN);
    render(&mut runtime, &mut terminal, REFERENCE_SCREEN);
    assert_eq!(
        terminal
            .backend()
            .buffer()
            .cell(Position::new(11, 10))
            .map(|cell| cell.symbol()),
        Some(":"),
        "ASCII themes use the historical inert colon cue"
    );

    for items in [Vec::new(), vec![ToolItem::new("Solo")]] {
        let expected_live_stops = if items.is_empty() { 1 } else { 2 };
        let keys = if items.is_empty() {
            Vec::new()
        } else {
            vec![ItemKey::text("solo")]
        };
        let mut host = ToolbarHost::new(items, keys);
        host.reference_gallery = true;
        let (mut runtime, mut terminal) =
            runtime_for_theme_on(host, Theme::junie(), REFERENCE_SCREEN);
        render(&mut runtime, &mut terminal, REFERENCE_SCREEN);
        assert_eq!(runtime.ring().entries().len(), expected_live_stops);
        assert!(runtime.app().events.is_empty());
    }
}

#[test]
fn toolbar_is_one_focus_stop_and_controlled_toggles_commit_once() {
    let run_key = ItemKey::text("run");
    let disabled_key = ItemKey::text("disabled");
    let wrap_key = ItemKey::text("wrap");
    let host = ToolbarHost::new(
        vec![
            ToolItem::new("Run"),
            ToolItem::new("Blocked").reason("not ready"),
            ToolItem::new("Wrap").toggle(false).shortcut("W"),
        ],
        vec![run_key, disabled_key, wrap_key],
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);

    assert_eq!(
        runtime.area_of(TOOLBAR),
        Some(DEFAULT_TOOLBAR_AREA),
        "the component owns one native root region"
    );
    assert!(runtime.app().state.hidden_keys().is_empty());
    assert!(!runtime.app().state.on_more());
    assert!(!runtime.app().state.menu_open());
    assert!(runtime.area_of_part(TOOLBAR, row_part(run_key)).is_some());
    assert!(
        runtime
            .area_of_part(TOOLBAR, row_part(disabled_key))
            .is_some()
    );
    assert!(runtime.area_of_part(TOOLBAR, row_part(wrap_key)).is_some());
    assert_eq!(
        runtime
            .ring()
            .entries()
            .iter()
            .filter(|entry| entry.id == TOOLBAR)
            .count(),
        1,
        "items contribute one root focus entry, not one entry each"
    );

    focus_toolbar(&mut runtime, &mut terminal, SCREEN);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Right);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().state.cursor_key(), Some(wrap_key));

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    assert_eq!(runtime.app().events.len(), 1);
    assert_eq!(
        runtime.app().events.first().copied(),
        Some(ToolbarActionEvent {
            key: wrap_key,
            action: ToolbarAction::Toggled(2, true),
        })
    );
    assert_eq!(
        runtime.app().items.get(2).and_then(|item| item.on),
        Some(true)
    );

    render(&mut runtime, &mut terminal, SCREEN);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Char(' '));
    assert_eq!(runtime.app().events.len(), 2);
    assert_eq!(
        runtime.app().events.get(1).map(|event| event.action),
        Some(ToolbarAction::Toggled(2, false)),
        "the second proposal starts from the caller's committed value"
    );
    assert_eq!(
        runtime.app().items.get(2).and_then(|item| item.on),
        Some(false)
    );

    render(&mut runtime, &mut terminal, SCREEN);
    let focus_glyph = Theme::junie()
        .design
        .glyphs
        .get(termrock::theme::GlyphRole::FocusBar);
    let buffer = terminal.backend().buffer();
    let focus_glyphs = DEFAULT_TOOLBAR_AREA
        .positions()
        .filter(|pos| {
            buffer
                .cell(*pos)
                .is_some_and(|cell| cell.symbol() == focus_glyph)
        })
        .count();
    assert_eq!(focus_glyphs, 1, "the root focus produces one item cue");

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Tab);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.focus(), Some(NEXT));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::BackTab);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.focus(), Some(TOOLBAR));
}

#[test]
fn disabled_items_are_reachable_only_when_skip_disabled_is_off_and_stay_inert() {
    let first = ItemKey::text("first");
    let disabled = ItemKey::text("disabled");
    let last = ItemKey::text("last");
    let mut host = ToolbarHost::new(
        vec![
            ToolItem::new("First"),
            ToolItem::new("No").disabled(true),
            ToolItem::new("Last"),
        ],
        vec![first, disabled, last],
    );
    host.skip_disabled = false;
    host.wrap = true;
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Right);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().state.cursor_key(), Some(disabled));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    assert!(
        runtime.app().events.is_empty(),
        "disabled activation is inert"
    );

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Right);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().state.cursor_key(), Some(last));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Right);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().state.cursor_key(), Some(first));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::End);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().state.cursor_key(), Some(last));
}

#[test]
fn vim_profile_moves_horizontally_and_home_end_keep_current_key_identity() {
    let item_keys = keys(&["a", "b", "c"]);
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("A"), ToolItem::new("B"), ToolItem::new("C")],
        item_keys.clone(),
    );
    host.nav = NavKeys::VIM;
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Char('l'));
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().state.cursor_key(), item_keys.get(1).copied());
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Char('G'));
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().state.cursor_key(), item_keys.get(2).copied());
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Char('g'));
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().state.cursor_key(), item_keys.first().copied());
}

#[test]
fn cursor_tracks_stable_keys_across_same_stamp_reorder_and_enabled_changes() {
    let item_keys = keys(&["a", "b", "c", "d"]);
    let host = ToolbarHost::new(
        vec![
            ToolItem::new("A"),
            ToolItem::new("B"),
            ToolItem::new("C"),
            ToolItem::new("D"),
        ],
        item_keys.clone(),
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Right);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Right);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().state.cursor_key(), item_keys.get(2).copied());

    {
        let app = runtime.app_mut();
        app.freeze_draw_model();
        app.items = vec![
            ToolItem::new("A"),
            ToolItem::new("C"),
            ToolItem::new("B"),
            ToolItem::new("D"),
        ];
        app.keys = vec![
            *item_keys.first().expect("first key"),
            *item_keys.get(2).expect("third key"),
            *item_keys.get(1).expect("second key"),
            *item_keys.get(3).expect("fourth key"),
        ];
    }
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::F(12));
    assert_eq!(runtime.app().state.cursor_key(), item_keys.get(2).copied());
    assert_eq!(runtime.app().state.cursor_index(), Some(1));

    if let Some(item) = runtime.app_mut().items.get_mut(1) {
        item.disabled = true;
    }
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::F(12));
    assert_eq!(
        runtime.app().state.cursor_key(),
        item_keys.get(1).copied(),
        "same keys with a newly disabled cursor normalize forward"
    );
}

#[test]
fn pointer_uses_published_item_keys_and_removed_targets_cannot_activate_replacements() {
    let a = ItemKey::text("a");
    let b = ItemKey::text("b");
    let c = ItemKey::text("c");
    let host = ToolbarHost::new(
        vec![
            ToolItem::new("Alpha"),
            ToolItem::new("Bravo"),
            ToolItem::new("Charlie"),
        ],
        vec![a, b, c],
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);

    let removed_area = runtime
        .area_of_part(TOOLBAR, row_part(b))
        .expect("middle item has native geometry");
    let stale_position = Position::new(removed_area.x, removed_area.y);
    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Down,
        stale_position,
    );
    {
        let app = runtime.app_mut();
        app.freeze_draw_model();
        app.items.remove(1);
        app.keys.remove(1);
    }
    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Up,
        stale_position,
    );
    assert!(runtime.app().events.is_empty());
    assert_ne!(runtime.app().state.cursor_key(), Some(c));

    runtime.app_mut().release_draw_model();
    render(&mut runtime, &mut terminal, SCREEN);
    let live_area = runtime
        .area_of_part(TOOLBAR, row_part(c))
        .expect("remaining item retains its key");
    let live_position = Position::new(live_area.x, live_area.y);
    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Down,
        live_position,
    );
    render(&mut runtime, &mut terminal, SCREEN);
    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Up,
        live_position,
    );
    render(&mut runtime, &mut terminal, SCREEN);

    assert_eq!(runtime.focus(), Some(TOOLBAR));
    assert_eq!(runtime.app().state.cursor_key(), Some(c));
    assert_eq!(
        runtime.app().events.last().copied(),
        Some(ToolbarActionEvent {
            key: c,
            action: ToolbarAction::Activated(1),
        })
    );
}

#[test]
fn tiny_and_zero_width_rows_publish_no_item_targets() {
    let first = ItemKey::text("first");
    let second = ItemKey::text("second");
    let item_keys = vec![first, second];
    let host = ToolbarHost::new(
        vec![ToolItem::new("First"), ToolItem::new("Second")],
        item_keys,
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(runtime.area_of_part(TOOLBAR, row_part(first)).is_some());

    runtime.app_mut().toolbar_area = Rect::new(7, 2, 1, 1);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(first)), None);
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(second)), None);

    runtime.app_mut().toolbar_area = Rect::new(7, 2, 0, 1);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(first)), None);
    assert_eq!(runtime.area_of(TOOLBAR), None);
}

#[test]
fn right_clipping_publishes_only_the_visible_part_of_a_nonzero_origin_item() {
    let only = ItemKey::text("near-edge");
    let mut host = ToolbarHost::new(vec![ToolItem::new("Edge")], vec![only]);
    host.toolbar_area = Rect::new(60, 2, 12, 1);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);

    assert_eq!(
        runtime.area_of_part(TOOLBAR, row_part(only)),
        Some(Rect::new(60, 2, 4, 1)),
        "the native hit part is clipped at the screen edge"
    );
}

#[test]
fn clipped_enabled_suffix_does_not_make_disabled_prefix_reachable() {
    let disabled = ItemKey::text("visible-disabled");
    let hidden = ItemKey::text("clipped-enabled");
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("No").disabled(true), ToolItem::new("Go")],
        vec![disabled, hidden],
    );
    host.toolbar_area = Rect::new(60, 2, 20, 1);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);

    assert_eq!(
        runtime.area_of_part(TOOLBAR, row_part(disabled)),
        Some(Rect::new(60, 2, 4, 1))
    );
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(hidden)), None);
    assert!(
        runtime.ring().reachable().all(|entry| entry.id != TOOLBAR),
        "a wholly clipped enabled suffix cannot expose the visible disabled prefix"
    );

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    assert!(runtime.app().events.is_empty());
}

#[test]
fn skip_disabled_false_keeps_visible_disabled_prefix_reachable_but_inert() {
    let disabled = ItemKey::text("visible-disabled");
    let hidden = ItemKey::text("clipped-enabled");
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("No").disabled(true), ToolItem::new("Go")],
        vec![disabled, hidden],
    );
    host.toolbar_area = Rect::new(60, 2, 20, 1);
    host.skip_disabled = false;
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);

    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(hidden)), None);
    assert!(runtime.ring().reachable().any(|entry| entry.id == TOOLBAR));
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().state.cursor_key(), Some(disabled));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    assert!(
        runtime.app().events.is_empty(),
        "disabled activation is inert"
    );
}

#[test]
fn end_right_and_enter_cannot_reach_a_wholly_clipped_enabled_row() {
    let first = ItemKey::text("first");
    let clipped = ItemKey::text("clipped");
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("AB"), ToolItem::new("Go")],
        vec![first, clipped],
    );
    host.toolbar_area = Rect::new(60, 2, 20, 1);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(
        runtime.area_of_part(TOOLBAR, row_part(first)),
        Some(Rect::new(60, 2, 4, 1))
    );
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(clipped)), None);
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::End);
    assert_eq!(runtime.app().state.cursor_key(), Some(first));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Right);
    assert_eq!(runtime.app().state.cursor_key(), Some(first));
    render(&mut runtime, &mut terminal, SCREEN);
    let focus_glyph = Theme::junie()
        .design
        .glyphs
        .get(termrock::theme::GlyphRole::FocusBar);
    let buffer = terminal.backend().buffer();
    let first_area = runtime
        .area_of_part(TOOLBAR, row_part(first))
        .expect("visible first row retains its committed area");
    let focus_glyphs = first_area
        .positions()
        .filter(|pos| {
            buffer
                .cell(*pos)
                .is_some_and(|cell| cell.symbol() == focus_glyph)
        })
        .count();
    assert_eq!(focus_glyphs, 1, "focus remains visible on the first row");
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    assert_eq!(
        runtime.app().events,
        vec![ToolbarActionEvent {
            key: first,
            action: ToolbarAction::Activated(0),
        }]
    );
}

#[test]
fn parent_area_clip_controls_row_registration_and_root_reachability() {
    let disabled = ItemKey::text("visible-disabled");
    let hidden = ItemKey::text("parent-clipped-enabled");
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("No").disabled(true), ToolItem::new("Go")],
        vec![disabled, hidden],
    );
    host.parent_clip = Some(Rect::new(5, 1, 4, 1));
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);

    assert_eq!(
        runtime.area_of_part(TOOLBAR, row_part(disabled)),
        Some(Rect::new(5, 1, 4, 1))
    );
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(hidden)), None);
    assert!(
        runtime.ring().reachable().all(|entry| entry.id != TOOLBAR),
        "the parent Ui::with_area clip participates in the same visibility policy"
    );
}

#[test]
fn label_shrink_cannot_promote_a_row_before_its_part_is_published() {
    let first = ItemKey::text("first");
    let hidden = ItemKey::text("hidden");
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("LongLong"), ToolItem::new("B")],
        vec![first, hidden],
    );
    host.toolbar_area = Rect::new(5, 1, 12, 1);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(hidden)), None);
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);

    let app = runtime.app_mut();
    app.freeze_draw_model();
    app.items
        .get_mut(0)
        .expect("first item is configured")
        .label = "A".to_owned();
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Right);
    assert_eq!(runtime.app().state.cursor_key(), Some(first));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    assert_eq!(
        runtime.app().events,
        vec![ToolbarActionEvent {
            key: first,
            action: ToolbarAction::Activated(0),
        }]
    );
}

#[test]
fn shortcut_width_growth_does_not_hide_a_previously_published_key() {
    let first = ItemKey::text("first");
    let second = ItemKey::text("second");
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("A"), ToolItem::new("B")],
        vec![first, second],
    );
    host.toolbar_area = Rect::new(5, 1, 8, 1);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(runtime.area_of_part(TOOLBAR, row_part(second)).is_some());
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);

    let app = runtime.app_mut();
    app.freeze_draw_model();
    app.items
        .get_mut(0)
        .expect("first item is configured")
        .shortcut = Some("Long".to_owned());
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::End);
    assert_eq!(runtime.app().state.cursor_key(), Some(second));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    assert_eq!(
        runtime.app().events,
        vec![ToolbarActionEvent {
            key: second,
            action: ToolbarAction::Activated(1),
        }]
    );
}

#[test]
fn toggle_marker_width_growth_does_not_hide_a_previously_published_key() {
    let first = ItemKey::text("first");
    let second = ItemKey::text("second");
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("A"), ToolItem::new("B")],
        vec![first, second],
    );
    host.toolbar_area = Rect::new(5, 1, 8, 1);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(runtime.area_of_part(TOOLBAR, row_part(second)).is_some());
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);

    let app = runtime.app_mut();
    app.freeze_draw_model();
    app.items.get_mut(0).expect("first item is configured").on = Some(false);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Right);
    assert_eq!(runtime.app().state.cursor_key(), Some(second));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    assert_eq!(
        runtime.app().events,
        vec![ToolbarActionEvent {
            key: second,
            action: ToolbarAction::Activated(1),
        }]
    );
}

#[test]
fn navigation_uses_published_key_order_and_current_indexes_after_reorder() {
    let first = ItemKey::text("first");
    let second = ItemKey::text("second");
    let third = ItemKey::text("third");
    let host = ToolbarHost::new(
        vec![ToolItem::new("A"), ToolItem::new("B"), ToolItem::new("C")],
        vec![first, second, third],
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);

    {
        let app = runtime.app_mut();
        app.freeze_draw_model();
        app.items = vec![ToolItem::new("C"), ToolItem::new("B"), ToolItem::new("A")];
        app.keys = vec![third, second, first];
    }
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Right);
    assert_eq!(runtime.app().state.cursor_key(), Some(second));
    assert_eq!(runtime.app().state.cursor_index(), Some(1));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    assert_eq!(
        runtime.app().events,
        vec![ToolbarActionEvent {
            key: second,
            action: ToolbarAction::Activated(1),
        }]
    );
}

#[test]
fn all_disabled_and_empty_toolbars_do_not_create_a_reachable_root_stop() {
    let only = ItemKey::text("disabled");
    let host = ToolbarHost::new(vec![ToolItem::new("No").disabled(true)], vec![only]);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(
        runtime.ring().reachable().all(|entry| entry.id != TOOLBAR),
        "an all-disabled skip-enabled toolbar is not a keyboard stop"
    );

    let empty = ToolbarHost::new(Vec::new(), Vec::new());
    let (mut empty_runtime, mut empty_terminal) = runtime_for(empty);
    render(&mut empty_runtime, &mut empty_terminal, SCREEN);
    assert_eq!(empty_runtime.area_of(TOOLBAR), None);
    assert_eq!(empty_runtime.area_of_part(TOOLBAR, row_part(only)), None);
}

#[test]
fn stable_key_builder_reports_count_and_actual_key_collisions() {
    let items = vec![ToolItem::new("A"), ToolItem::new("B")];
    let mismatch = Toolbar::new(TOOLBAR, items.clone())
        .try_item_keys(vec![ItemKey::text("only")])
        .expect_err("a short key list is rejected");
    assert_eq!(
        mismatch,
        ToolbarError::ItemKeyCount {
            expected: 2,
            actual: 1,
        }
    );

    let duplicate = ItemKey::Text(0xA11CE);
    let collision = Toolbar::new(TOOLBAR, items)
        .try_item_keys(vec![duplicate, duplicate])
        .expect_err("an actual Text key collision is rejected");
    assert!(matches!(
        collision,
        ToolbarError::DuplicateItemKey {
            key,
            first_index: 0,
            duplicate_index: 1,
        } if key == duplicate
    ));

    let repeated = Toolbar::new(
        TOOLBAR,
        vec![ToolItem::new("A"), ToolItem::new("B"), ToolItem::new("C")],
    )
    .try_item_keys(vec![duplicate, ItemKey::text("other"), duplicate])
    .expect_err("the duplicate error keeps the first matching index");
    assert!(matches!(
        repeated,
        ToolbarError::DuplicateItemKey {
            key,
            first_index: 0,
            duplicate_index: 2,
        } if key == duplicate
    ));
}

#[test]
fn disabled_reachable_row_has_no_enabled_focus_cue() {
    let disabled = ItemKey::text("disabled");
    let mut host = ToolbarHost::new(vec![ToolItem::new("No").disabled(true)], vec![disabled]);
    host.skip_disabled = false;
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);

    let focus_glyph = Theme::junie()
        .design
        .glyphs
        .get(termrock::theme::GlyphRole::FocusBar);
    let buffer = terminal.backend().buffer();
    let region = runtime
        .area_of_part(TOOLBAR, row_part(disabled))
        .expect("disabled item remains a reachable part");
    let focus_glyphs = region
        .positions()
        .filter(|pos| {
            buffer
                .cell(*pos)
                .is_some_and(|cell| cell.symbol() == focus_glyph)
        })
        .count();
    assert_eq!(focus_glyphs, 0);
    assert_eq!(runtime.app().state.cursor_key(), Some(disabled));
}

#[test]
fn default_overflow_uses_native_menu_receipts_hints_reasons_and_focus_return() {
    let visible = ItemKey::text("visible");
    let alpha = ItemKey::text("alpha");
    let beta = ItemKey::text("beta");
    let gamma = ItemKey::text("gamma");
    let mut host = ToolbarHost::new(
        vec![
            ToolItem::new("Visible"),
            ToolItem::new("Alpha"),
            ToolItem::new("Beta")
                .toggle(false)
                .shortcut("Ctrl+B")
                .separator(),
            ToolItem::new("Gamma").reason("Unavailable in this mode"),
        ],
        vec![visible, alpha, beta, gamma],
    );
    host.toolbar_area = Rect::new(5, 1, 16, 1);
    host.use_default_overflow();
    host.reference_probe = true;
    let mut theme = Theme::junie();
    theme.design.size.popup_min_width = 30;
    let (mut runtime, mut terminal) = runtime_for_theme(host, theme);
    render(&mut runtime, &mut terminal, SCREEN);
    runtime.app_mut().reference_probe = false;

    assert_eq!(runtime.app().state.hidden_keys(), vec![alpha, beta, gamma]);
    assert!(!runtime.app().state.on_more());
    assert!(!runtime.app().state.menu_open());
    assert_eq!(
        runtime.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW)),
        Some(Rect::new(18, 1, 3, 1)),
        "the historical default publishes More when rows do not fit"
    );
    assert_eq!(
        runtime.area_of_part(TOOLBAR, row_part(visible)),
        Some(Rect::new(5, 1, 9, 1))
    );
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(alpha)), None);
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(beta)), None);
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(gamma)), None);
    assert_eq!(
        runtime
            .ring()
            .entries()
            .iter()
            .filter(|entry| entry.id == TOOLBAR)
            .count(),
        1,
        "More remains part of the Toolbar's single native focus stop"
    );
    assert_eq!(
        terminal
            .backend()
            .buffer()
            .cell(Position::new(19, 1))
            .map(|cell| cell.symbol()),
        Some("»"),
        "the default More glyph matches the historical Unicode title"
    );

    focus_toolbar(&mut runtime, &mut terminal, SCREEN);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::End);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(runtime.app().state.on_more());
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(runtime.is_open(OVERFLOW_MENU));
    let menu_area = runtime
        .area_of(OVERFLOW_MENU)
        .expect("More opens an anchored native menu");
    assert!(
        menu_area.y >= Rect::new(18, 1, 3, 1).bottom(),
        "the More menu opens below its committed anchor when space permits"
    );
    assert!(runtime.app().state.menu_open());
    let menu_text = screen_text(&terminal);
    assert!(menu_text.contains("Alpha"));
    assert!(menu_text.contains("○ Beta"));
    assert!(menu_text.contains("Ctrl+B"));
    assert!(menu_text.contains("Gamma"));
    assert!(
        runtime
            .area_of_part(OVERFLOW_MENU, PartRef::of(Part::RULE))
            .is_some(),
        "separator metadata is represented by the native menu"
    );
    assert!(
        !menu_text.contains("Reference-only"),
        "inert reference drawing cannot replace the live snapshot"
    );
    assert_eq!(
        runtime.app().state.hidden_keys(),
        vec![alpha, beta, gamma],
        "reference rows do not replace the live published hidden-key snapshot"
    );

    let gamma_area = runtime
        .area_of_part(OVERFLOW_MENU, menu_row_part(2))
        .expect("disabled menu row has native geometry");
    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Move,
        Position::new(gamma_area.x.saturating_add(2), gamma_area.y),
    );
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(screen_text(&terminal).contains("Unavailable in this mode"));

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Down);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    assert_eq!(
        runtime.app().events.last().copied(),
        Some(ToolbarActionEvent {
            key: beta,
            action: ToolbarAction::Toggled(2, true),
        }),
        "the native selected-row receipt resolves the toggle's stable key"
    );
    assert_eq!(
        runtime.app().items.get(2).and_then(|item| item.on),
        Some(true)
    );
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.focus(), Some(TOOLBAR));

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(runtime.is_open(OVERFLOW_MENU));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Char(' '));
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(
        runtime.app().events.last().copied(),
        Some(ToolbarActionEvent {
            key: alpha,
            action: ToolbarAction::Activated(1),
        }),
        "Space selects the native menu's current row through its receipt"
    );
    assert!(!runtime.is_open(OVERFLOW_MENU));
    assert!(!runtime.app().state.menu_open());
    assert_eq!(runtime.focus(), Some(TOOLBAR));

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(runtime.is_open(OVERFLOW_MENU));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Esc);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(!runtime.is_open(OVERFLOW_MENU));
    assert!(!runtime.app().state.menu_open());
    assert_eq!(runtime.focus(), Some(TOOLBAR));
}

#[test]
fn overflow_retirement_drains_only_its_terminal_receipt() {
    let visible = ItemKey::text("receipt-visible");
    let alpha = ItemKey::text("receipt-alpha");
    let beta = ItemKey::text("receipt-beta");
    let mut host = ToolbarHost::new(
        vec![
            ToolItem::new("Visible"),
            ToolItem::new("Alpha").toggle(false),
            ToolItem::new("Beta").toggle(false),
        ],
        vec![visible, alpha, beta],
    );
    host.toolbar_area = Rect::new(5, 1, 16, 1);
    host.use_default_overflow();
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);

    open_overflow_by_keyboard(&mut runtime, &mut terminal, SCREEN);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::End);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    render(&mut runtime, &mut terminal, SCREEN);

    assert_eq!(
        runtime.app().events,
        vec![ToolbarActionEvent {
            key: beta,
            action: ToolbarAction::Toggled(2, true),
        }]
    );
    assert_eq!(runtime.app().items[2].on, Some(true));
    assert!(!runtime.is_open(OVERFLOW_MENU));
    assert!(!runtime.app().state.menu_open());
    assert_eq!(runtime.focus(), Some(TOOLBAR));
    assert!(runtime.diagnostics().is_empty());

    ready_for_input(&mut runtime, &mut terminal, SCREEN);
    let _ = runtime
        .handle(Input::Tick)
        .expect("ordinary update after closing More is dispatched");
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().events.len(), 1);
    assert!(runtime.diagnostics().is_empty());

    open_overflow_by_keyboard(&mut runtime, &mut terminal, SCREEN);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    render(&mut runtime, &mut terminal, SCREEN);

    assert_eq!(
        runtime.app().events,
        vec![
            ToolbarActionEvent {
                key: beta,
                action: ToolbarAction::Toggled(2, true),
            },
            ToolbarActionEvent {
                key: alpha,
                action: ToolbarAction::Toggled(1, true),
            },
        ]
    );
    assert_eq!(runtime.app().items[1].on, Some(true));
    assert!(!runtime.is_open(OVERFLOW_MENU));
    assert!(!runtime.app().state.menu_open());
    assert_eq!(runtime.focus(), Some(TOOLBAR));
    assert!(runtime.diagnostics().is_empty());
}

#[test]
fn pointer_more_opens_native_menu_and_keyboard_receipt_selects_the_hidden_key() {
    let keys = keys(&["visible", "one", "two", "three"]);
    let mut host = ToolbarHost::new(
        vec![
            ToolItem::new("Visible"),
            ToolItem::new("One"),
            ToolItem::new("Two"),
            ToolItem::new("Three"),
        ],
        keys.clone(),
    );
    host.toolbar_area = Rect::new(5, 1, 16, 1);
    host.use_default_overflow();
    host.more_look = Some(ToolbarMoreLook::Title);
    host.more_danger_hover = Some(true);
    host.more_glyph = Some("界".to_owned());
    host.more_padding = 2;
    host.more_style_overlay = Some(&TITLE_METRIC_SCOPE);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);

    let more = runtime
        .area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW))
        .expect("opt-in overflow publishes More");
    assert_eq!(more, Rect::new(15, 1, 6, 1));
    assert_eq!(
        terminal
            .backend()
            .buffer()
            .cell(Position::new(17, 1))
            .map(|cell| cell.symbol()),
        Some("界"),
        "the explicit wide glyph and custom padding define the native layout"
    );
    click(
        &mut runtime,
        &mut terminal,
        SCREEN,
        Position::new(more.x.saturating_add(1), more.y),
    );
    assert!(runtime.is_open(OVERFLOW_MENU));
    let menu_area = runtime
        .area_of(OVERFLOW_MENU)
        .expect("the pointer-opened menu publishes native geometry");
    assert_eq!(
        menu_area.x, more.x,
        "the menu retains the More anchor column"
    );
    assert!(menu_area.y >= more.bottom(), "the menu opens below More");
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::End);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(
        runtime.app().events,
        vec![ToolbarActionEvent {
            key: *keys.get(3).expect("third hidden key exists"),
            action: ToolbarAction::Activated(3),
        }],
        "keyboard selection resolves the native terminal receipt to its hidden key"
    );
    assert!(!runtime.is_open(OVERFLOW_MENU));
    assert!(!runtime.app().state.menu_open());
    assert_eq!(runtime.focus(), Some(TOOLBAR));
    assert!(runtime.diagnostics().is_empty());

    ready_for_input(&mut runtime, &mut terminal, SCREEN);
    let _ = runtime
        .handle(Input::Tick)
        .expect("settled More action is retired");
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().events.len(), 1, "receipt is delivered once");
    assert_eq!(runtime.focus(), Some(TOOLBAR));
    assert!(runtime.diagnostics().is_empty());
}

#[test]
fn pre_open_model_changes_keep_published_hidden_order_and_reopen_replaces_it() {
    let visible = ItemKey::text("visible");
    let alpha = ItemKey::text("alpha");
    let beta = ItemKey::text("beta");
    let gamma = ItemKey::text("gamma");
    let delta = ItemKey::text("delta");
    let mut host = ToolbarHost::new(
        vec![
            ToolItem::new("Visible"),
            ToolItem::new("Alpha"),
            ToolItem::new("Beta"),
            ToolItem::new("Gamma"),
        ],
        vec![visible, alpha, beta, gamma],
    );
    host.toolbar_area = Rect::new(5, 1, 16, 1);
    host.use_default_overflow();
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::End);
    render(&mut runtime, &mut terminal, SCREEN);

    {
        let app = runtime.app_mut();
        app.freeze_draw_model();
        app.items = vec![
            ToolItem::new("Visible"),
            ToolItem::new("Gamma current"),
            ToolItem::new("Beta current"),
            ToolItem::new("Alpha current"),
            ToolItem::new("Delta new"),
        ];
        app.keys = vec![visible, gamma, beta, alpha, delta];
    }
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    runtime.app_mut().release_draw_model();
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(runtime.is_open(OVERFLOW_MENU));
    let text = screen_text(&terminal);
    assert!(text.contains("Alpha current"));
    assert!(text.contains("Beta current"));
    assert!(text.contains("Gamma current"));
    assert!(
        !text.contains("Delta new"),
        "new hidden keys wait for the next open"
    );
    assert!(
        text.find("Alpha current") < text.find("Beta current")
            && text.find("Beta current") < text.find("Gamma current"),
        "the open list keeps the last published hidden-key order"
    );

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Down);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Down);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    assert_eq!(
        runtime.app().events.last().copied(),
        Some(ToolbarActionEvent {
            key: gamma,
            action: ToolbarAction::Activated(1),
        }),
        "activation resolves the old snapshot key against the current reordered model"
    );
    render(&mut runtime, &mut terminal, SCREEN);

    open_overflow_by_keyboard(&mut runtime, &mut terminal, SCREEN);
    let reopened = screen_text(&terminal);
    assert!(reopened.contains("Delta new"));
    assert!(
        reopened.find("Gamma current") < reopened.find("Beta current")
            && reopened.find("Beta current") < reopened.find("Alpha current")
            && reopened.find("Alpha current") < reopened.find("Delta new"),
        "a later same-geometry draw replaces the next opening snapshot"
    );
}

#[test]
fn open_snapshot_keeps_removed_and_disabled_slots_until_reopen() {
    let visible = ItemKey::text("visible");
    let alpha = ItemKey::text("alpha");
    let beta = ItemKey::text("beta");
    let gamma = ItemKey::text("gamma");
    let delta = ItemKey::text("delta");
    let host = ToolbarHost::new(
        vec![
            ToolItem::new("Visible"),
            ToolItem::new("Alpha"),
            ToolItem::new("Beta"),
            ToolItem::new("Gamma"),
        ],
        vec![visible, alpha, beta, gamma],
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    runtime.app_mut().toolbar_area = Rect::new(5, 1, 16, 1);
    runtime.app_mut().use_default_overflow();
    render(&mut runtime, &mut terminal, SCREEN);
    open_overflow_by_keyboard(&mut runtime, &mut terminal, SCREEN);

    {
        let app = runtime.app_mut();
        app.items = vec![
            ToolItem::new("Visible"),
            ToolItem::new("Gamma current"),
            ToolItem::new("Beta current").reason("Beta is paused"),
            ToolItem::new("Delta new"),
        ];
        app.keys = vec![visible, gamma, beta, delta];
        app.toolbar_area = Rect::new(5, 1, 29, 1);
    }
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(
        runtime.is_open(OVERFLOW_MENU),
        "native More survives resize"
    );
    assert_eq!(
        runtime.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW)),
        Some(Rect::new(31, 1, 3, 1)),
        "the menu follows the new committed More geometry"
    );
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::F(12));
    render(&mut runtime, &mut terminal, SCREEN);
    let open_text = screen_text(&terminal);
    assert!(
        open_text.contains("Alpha"),
        "a removed key remains as its inert slot"
    );
    assert!(open_text.contains("Beta current"));
    assert!(open_text.contains("Gamma current"));
    assert!(
        !open_text.contains("Delta new"),
        "new keys do not enter an open session"
    );

    click_menu_row(&mut runtime, &mut terminal, SCREEN, 0);
    assert!(runtime.app().events.is_empty(), "a removed key is inert");
    click_menu_row(&mut runtime, &mut terminal, SCREEN, 1);
    assert!(
        runtime.app().events.is_empty(),
        "a newly disabled key is inert"
    );
    click_menu_row(&mut runtime, &mut terminal, SCREEN, 2);
    assert_eq!(
        runtime.app().events.last().copied(),
        Some(ToolbarActionEvent {
            key: gamma,
            action: ToolbarAction::Activated(1),
        })
    );
    render(&mut runtime, &mut terminal, SCREEN);

    open_overflow_by_keyboard(&mut runtime, &mut terminal, SCREEN);
    let reopened = screen_text(&terminal);
    assert!(!reopened.contains("Alpha"));
    assert!(reopened.contains("Delta new"));
}

#[test]
fn no_overflow_and_missing_native_anchor_clear_opening_permission() {
    let visible = ItemKey::text("visible");
    let alpha = ItemKey::text("alpha");
    let beta = ItemKey::text("beta");
    let mut host = ToolbarHost::new(
        vec![
            ToolItem::new("Visible"),
            ToolItem::new("Alpha"),
            ToolItem::new("Beta"),
        ],
        vec![visible, alpha, beta],
    );
    host.toolbar_area = Rect::new(5, 1, 16, 1);
    host.use_default_overflow();
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    open_overflow_by_keyboard(&mut runtime, &mut terminal, SCREEN);

    runtime.app_mut().parent_clip = Some(Rect::new(5, 1, 13, 1));
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(
        runtime.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW)),
        None
    );
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::F(12));
    assert!(
        !runtime.is_open(OVERFLOW_MENU),
        "anchor loss closes the native menu before Esc"
    );
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Esc);
    assert!(
        !runtime.is_open(OVERFLOW_MENU),
        "a missing committed anchor closes the native menu"
    );
    render(&mut runtime, &mut terminal, SCREEN);

    runtime.app_mut().parent_clip = None;
    runtime.app_mut().use_no_overflow_fixture();
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(
        runtime.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW)),
        None
    );
    assert!(runtime.area_of_part(TOOLBAR, row_part(visible)).is_some());
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(alpha)), None);
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(beta)), None);
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::End);
    render(&mut runtime, &mut terminal, SCREEN);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Enter);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(!runtime.is_open(OVERFLOW_MENU));

    runtime.app_mut().use_default_overflow();
    runtime.app_mut().toolbar_area = Rect::new(5, 1, 0, 1);
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(
        runtime.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW)),
        None
    );
    assert_eq!(runtime.area_of(TOOLBAR), None);
}

#[test]
fn unpresented_prepared_frame_cannot_authorize_opening_from_its_new_snapshot() {
    let mut host = ToolbarHost::new(
        vec![
            ToolItem::new("Visible"),
            ToolItem::new("Alpha"),
            ToolItem::new("Beta"),
        ],
        keys(&["visible", "alpha", "beta"]),
    );
    host.toolbar_area = Rect::new(5, 1, 16, 1);
    host.use_default_overflow();
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::End);
    render(&mut runtime, &mut terminal, SCREEN);

    {
        let app = runtime.app_mut();
        app.items = vec![
            ToolItem::new("Visible"),
            ToolItem::new("Beta current"),
            ToolItem::new("Alpha current"),
        ];
        app.keys = keys(&["visible", "beta", "alpha"]);
    }
    let prepared = runtime.prepare(SCREEN).expect("candidate frame prepares");
    terminal
        .draw(|frame| {
            runtime
                .draw_prepared(prepared, frame)
                .expect("prepared output matches the terminal");
        })
        .expect("test terminal draws the unacknowledged output");

    assert!(
        runtime
            .handle(Input::Key(Key {
                code: KeyCode::Enter,
                mods: KeyModifiers::NONE,
            }))
            .is_err(),
        "input waits until the prepared output has been presented"
    );
    assert!(!runtime.is_open(OVERFLOW_MENU));
}

#[test]
fn more_tip_and_hover_policy_defaults_and_overrides_are_owned_configuration() {
    let toolbar = Toolbar::new(TOOLBAR, vec![]).more_glyph("…");
    assert_eq!(
        toolbar.more_tip_value(),
        &ToolbarTip::new("Show hidden"),
        "changing only the glyph retains the default More-tip content"
    );
    assert_eq!(
        toolbar.tooltip_policy_value(),
        ToolbarTooltipPolicy {
            enabled: true,
            delay_ms: 600,
            max_width: 48,
        }
    );
    const {
        assert!(!ToolbarTooltipPolicy::OFF.enabled);
    }

    let custom_tip = ToolbarTip::new("Extra actions").shortcut("Ctrl+M");
    let custom_policy = ToolbarTooltipPolicy::OFF
        .enabled(true)
        .delay_ms(250)
        .max_width(32);
    let configured = Toolbar::new(TOOLBAR, vec![])
        .more_tip(custom_tip.clone())
        .tooltip_policy(custom_policy);
    assert_eq!(configured.more_tip_value(), &custom_tip);
    assert_eq!(configured.tooltip_policy_value(), custom_policy);
}

#[test]
fn ascii_theme_uses_its_named_more_title_glyph() {
    let host = ToolbarHost::new(
        vec![
            ToolItem::new("One"),
            ToolItem::new("Two"),
            ToolItem::new("Three"),
        ],
        keys(&["one", "two", "three"]),
    );
    let mut host = host;
    host.toolbar_area = Rect::new(5, 1, 12, 1);
    host.use_default_overflow();
    let theme = Theme::junie().builder().ascii_glyphs().build();
    let (mut runtime, mut terminal) = runtime_for_theme(host, theme);
    render(&mut runtime, &mut terminal, SCREEN);

    assert_eq!(
        runtime.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW)),
        Some(Rect::new(14, 1, 3, 1))
    );
    assert_eq!(
        terminal
            .backend()
            .buffer()
            .cell(Position::new(15, 1))
            .map(|cell| cell.symbol()),
        Some(">")
    );
}

#[test]
fn more_style_glyph_is_used_unless_the_toolbar_has_an_explicit_override() {
    let mut styled_host = ToolbarHost::new(
        vec![
            ToolItem::new("One"),
            ToolItem::new("Two"),
            ToolItem::new("Three"),
        ],
        keys(&["one-style", "two-style", "three-style"]),
    );
    styled_host.toolbar_area = Rect::new(5, 1, 12, 1);
    styled_host.use_default_overflow();
    let (mut runtime, mut terminal) = runtime_for_theme(
        styled_host,
        theme_with_overflow_glyph(termrock::theme::GlyphRole::OverflowRight),
    );
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(
        terminal
            .backend()
            .buffer()
            .cell(Position::new(15, 1))
            .map(|cell| cell.symbol()),
        Some("›"),
        "an explicit Part::OVERFLOW role precedes the named fallback"
    );

    let mut explicit_host = ToolbarHost::new(
        vec![
            ToolItem::new("One"),
            ToolItem::new("Two"),
            ToolItem::new("Three"),
        ],
        keys(&["one-explicit", "two-explicit", "three-explicit"]),
    );
    explicit_host.toolbar_area = Rect::new(5, 1, 12, 1);
    explicit_host.use_default_overflow();
    explicit_host.more_glyph = Some("…".to_owned());
    let (mut runtime, mut terminal) = runtime_for_theme(
        explicit_host,
        theme_with_overflow_glyph(termrock::theme::GlyphRole::OverflowRight),
    );
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(
        terminal
            .backend()
            .buffer()
            .cell(Position::new(15, 1))
            .map(|cell| cell.symbol()),
        Some("…"),
        "an explicit Toolbar glyph precedes the style role"
    );
}

#[test]
fn hovered_style_glyph_recomputes_more_width_and_its_row_boundary() {
    let mut host = ToolbarHost::new(
        vec![
            ToolItem::new("One"),
            ToolItem::new("Two"),
            ToolItem::new("Three"),
        ],
        keys(&["one-hover-style", "two-hover-style", "three-hover-style"]),
    );
    host.toolbar_area = Rect::new(5, 1, 12, 1);
    host.use_default_overflow();
    let (mut runtime, mut terminal) = runtime_for_theme(
        host,
        theme_with_hovered_overflow_glyph(termrock::theme::GlyphRole::OverflowRight),
    );
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(
        runtime.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW)),
        Some(Rect::new(14, 1, 3, 1)),
        "the unhovered named fallback is one cell plus default padding"
    );

    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Move,
        Position::new(15, 1),
    );
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(
        runtime.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW)),
        Some(Rect::new(13, 1, 4, 1)),
        "the hovered role is resolved before layout and uses its two-cell theme value"
    );
    assert_eq!(
        terminal
            .backend()
            .buffer()
            .cell(Position::new(14, 1))
            .map(|cell| cell.symbol()),
        Some("界")
    );
}

#[test]
fn two_cell_more_glyph_and_padding_drive_geometry_drawing_and_hidden_identity() {
    let visible = ItemKey::text("visible-wide-more");
    let hidden = ItemKey::text("hidden-wide-more");
    let hidden_after = ItemKey::text("hidden-wide-more-after");
    let mut host = ToolbarHost::new(
        vec![
            ToolItem::new("One"),
            ToolItem::new("Two"),
            ToolItem::new("Three"),
        ],
        vec![visible, hidden, hidden_after],
    );
    host.toolbar_area = Rect::new(5, 2, 12, 1);
    host.use_default_overflow();
    host.more_glyph = Some("界".to_owned());
    host.more_padding = 2;
    let (mut runtime, mut terminal) = runtime_for_theme(
        host,
        theme_with_overflow_glyph(termrock::theme::GlyphRole::OverflowRight),
    );
    render(&mut runtime, &mut terminal, SCREEN);

    let more = Rect::new(11, 2, 6, 1);
    assert_eq!(
        runtime.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW)),
        Some(more),
        "a two-cell glyph plus two cells of padding on each side is six cells"
    );
    assert_eq!(
        runtime.area_of_part(TOOLBAR, row_part(visible)),
        Some(Rect::new(5, 2, 5, 1)),
        "the visible row fits exactly before the More gap"
    );
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(hidden)), None);
    assert_eq!(runtime.area_of_part(TOOLBAR, row_part(hidden_after)), None);
    assert_eq!(
        terminal
            .backend()
            .buffer()
            .cell(Position::new(13, 2))
            .map(|cell| cell.symbol()),
        Some("界"),
        "the explicit two-cell string is painted at the configured inset"
    );
    assert_eq!(runtime.area_of(OVERFLOW_MENU), None);

    click(&mut runtime, &mut terminal, SCREEN, Position::new(13, 2));
    assert!(runtime.is_open(OVERFLOW_MENU));
    assert_eq!(
        runtime.area_of(OVERFLOW_MENU).map(|area| area.x),
        Some(more.x),
        "the native menu uses the custom-width More part as its anchor"
    );
    click_menu_row(&mut runtime, &mut terminal, SCREEN, 0);
    assert_eq!(
        runtime.app().events.last().copied(),
        Some(ToolbarActionEvent {
            key: hidden,
            action: ToolbarAction::Activated(1),
        }),
        "the selected native menu row resolves to the hidden stable key"
    );
}

#[test]
fn customized_named_more_title_survives_theme_clone_and_controls_geometry() {
    let mut theme = Theme::junie();
    theme.design.glyphs.set_more_titles("…");
    let cloned_theme = theme.clone();
    let mut host = ToolbarHost::new(
        vec![
            ToolItem::new("One"),
            ToolItem::new("Two"),
            ToolItem::new("Three"),
        ],
        keys(&["one-named", "two-named", "three-named"]),
    );
    host.toolbar_area = Rect::new(5, 1, 12, 1);
    host.use_default_overflow();
    let (mut runtime, mut terminal) = runtime_for_theme(host, cloned_theme);
    render(&mut runtime, &mut terminal, SCREEN);

    assert_eq!(
        runtime.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW)),
        Some(Rect::new(14, 1, 3, 1))
    );
    assert_eq!(
        terminal
            .backend()
            .buffer()
            .cell(Position::new(15, 1))
            .map(|cell| cell.symbol()),
        Some("…")
    );
}

#[test]
fn all_fitting_rows_do_not_reserve_custom_more_geometry() {
    let first = ItemKey::text("all-fit-first");
    let second = ItemKey::text("all-fit-second");
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("One"), ToolItem::new("Two")],
        vec![first, second],
    );
    host.toolbar_area = Rect::new(5, 1, 42, 1);
    host.use_default_overflow();
    host.more_glyph = Some("界".to_owned());
    host.more_padding = 2;
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);

    assert!(
        runtime
            .area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW))
            .is_none()
    );
    assert!(runtime.area_of_part(TOOLBAR, row_part(first)).is_some());
    assert!(runtime.area_of_part(TOOLBAR, row_part(second)).is_some());
}

#[test]
fn item_tip_uses_native_hover_and_the_default_deadline() {
    let key = ItemKey::text("tooltip-run");
    let host = ToolbarHost::new(
        vec![
            ToolItem::new("Run")
                .tip("Run the selected task")
                .shortcut("Ctrl+R"),
        ],
        vec![key],
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    let ring_before = runtime.ring().entries().len();
    let focus_before = runtime.focus();

    hover_part(&mut runtime, &mut terminal, SCREEN, TOOLBAR, row_part(key));
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    advance(&mut runtime, &mut terminal, SCREEN, 599);
    assert!(
        !runtime.is_open(HOVER_TOOLTIP),
        "599 ms is below the default delay"
    );
    advance(&mut runtime, &mut terminal, SCREEN, 1);

    assert!(runtime.is_open(HOVER_TOOLTIP));
    assert!(screen_text(&terminal).contains("Run the selected task"));
    assert!(screen_text(&terminal).contains("Ctrl+R"));
    assert_eq!(
        runtime.focus(),
        focus_before,
        "a tip adds no focus stop or request"
    );
    assert_eq!(runtime.ring().entries().len(), ring_before);
    assert_eq!(
        runtime.capture_owner(),
        None,
        "a tip does not capture the pointer"
    );
}

#[test]
fn focus_alone_does_not_start_a_hover_deadline() {
    let row_key = ItemKey::text("focus-no-tooltip");
    let host = ToolbarHost::new(
        vec![ToolItem::new("Focus target").tip("Hover only")],
        vec![row_key],
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    focus_toolbar(&mut runtime, &mut terminal, SCREEN);
    advance(&mut runtime, &mut terminal, SCREEN, 700);

    assert_eq!(runtime.focus(), Some(TOOLBAR));
    assert!(!runtime.is_open(HOVER_TOOLTIP));
}

#[test]
fn custom_zero_and_off_policies_are_scoped_to_each_toolbar() {
    let key = ItemKey::text("custom-delay");
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("Run").tip("A deliberately long explanation")],
        vec![key],
    );
    host.tooltip_policy = ToolbarTooltipPolicy::DEFAULT.delay_ms(25).max_width(12);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(&mut runtime, &mut terminal, SCREEN, TOOLBAR, row_part(key));
    advance(&mut runtime, &mut terminal, SCREEN, 24);
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    advance(&mut runtime, &mut terminal, SCREEN, 1);
    assert!(runtime.is_open(HOVER_TOOLTIP));
    assert!(
        runtime
            .layer_area(HOVER_TOOLTIP)
            .is_some_and(|area| area.width <= 14)
    );

    let off_key = ItemKey::text("off-policy");
    let mut off_host = ToolbarHost::new(
        vec![ToolItem::new("Never").tip("Disabled by policy")],
        vec![off_key],
    );
    off_host.tooltip_policy(ToolbarTooltipPolicy::OFF);
    let (mut off_runtime, mut off_terminal) = runtime_for(off_host);
    render(&mut off_runtime, &mut off_terminal, SCREEN);
    hover_part(
        &mut off_runtime,
        &mut off_terminal,
        SCREEN,
        TOOLBAR,
        row_part(off_key),
    );
    advance(&mut off_runtime, &mut off_terminal, SCREEN, 700);
    assert!(!off_runtime.is_open(HOVER_TOOLTIP));

    let zero_key = ItemKey::text("zero-delay");
    let mut zero_host =
        ToolbarHost::new(vec![ToolItem::new("Now").tip("Immediate")], vec![zero_key]);
    zero_host.tooltip_policy(ToolbarTooltipPolicy::DEFAULT.delay_ms(0));
    let (mut zero_runtime, mut zero_terminal) = runtime_for(zero_host);
    render(&mut zero_runtime, &mut zero_terminal, SCREEN);
    hover_part(
        &mut zero_runtime,
        &mut zero_terminal,
        SCREEN,
        TOOLBAR,
        row_part(zero_key),
    );
    render(&mut zero_runtime, &mut zero_terminal, SCREEN);
    assert!(zero_runtime.is_open(HOVER_TOOLTIP));

    let shortcut_key = ItemKey::text("long-shortcut-tip");
    let mut shortcut_host = ToolbarHost::new(
        vec![
            ToolItem::new("Go")
                .tip("Open")
                .shortcut("Ctrl+Shift+Alt+Right"),
        ],
        vec![shortcut_key],
    );
    shortcut_host.tooltip_policy = ToolbarTooltipPolicy::DEFAULT.delay_ms(0).max_width(8);
    let (mut shortcut_runtime, mut shortcut_terminal) = runtime_for(shortcut_host);
    render(&mut shortcut_runtime, &mut shortcut_terminal, SCREEN);
    hover_part(
        &mut shortcut_runtime,
        &mut shortcut_terminal,
        SCREEN,
        TOOLBAR,
        row_part(shortcut_key),
    );
    render(&mut shortcut_runtime, &mut shortcut_terminal, SCREEN);
    assert!(shortcut_runtime.is_open(HOVER_TOOLTIP));
    assert!(
        shortcut_runtime
            .layer_area(HOVER_TOOLTIP)
            .is_some_and(|area| area.width <= 10),
        "a wrapped display shortcut stays inside the configured width"
    );
}

#[test]
fn disabled_item_tip_is_independent_of_disabled_reason() {
    let described = ItemKey::text("disabled-described");
    let reason_only = ItemKey::text("disabled-reason-only");
    let host = ToolbarHost::new(
        vec![
            ToolItem::new("Unavailable")
                .reason("explanation is not a tooltip")
                .tip("Configured disabled tip"),
            ToolItem::new("Blocked").reason("reason-only private text"),
        ],
        vec![described, reason_only],
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(described),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(runtime.is_open(HOVER_TOOLTIP));
    assert!(screen_text(&terminal).contains("Configured disabled tip"));

    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(reason_only),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    assert!(!screen_text(&terminal).contains("reason-only private text"));
}

#[test]
fn more_tip_uses_default_or_structured_override_only_when_more_is_published() {
    let items = vec![
        ToolItem::new("One"),
        ToolItem::new("Two"),
        ToolItem::new("Three"),
    ];
    let mut default_host =
        ToolbarHost::new(items.clone(), keys(&["more-one", "more-two", "more-three"]));
    default_host.toolbar_area = Rect::new(5, 1, 12, 1);
    default_host.use_default_overflow();
    let (mut runtime, mut terminal) = runtime_for(default_host);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(
        runtime
            .area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW))
            .is_some()
    );
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        PartRef::of(Part::OVERFLOW),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(runtime.is_open(HOVER_TOOLTIP));
    assert!(screen_text(&terminal).contains("Show hidden"));

    let mut custom_host =
        ToolbarHost::new(items, keys(&["custom-one", "custom-two", "custom-three"]));
    custom_host.toolbar_area = Rect::new(5, 1, 12, 1);
    custom_host.use_default_overflow();
    custom_host.more_tip(ToolbarTip::new("More commands").shortcut("M"));
    let (mut custom_runtime, mut custom_terminal) = runtime_for(custom_host);
    render(&mut custom_runtime, &mut custom_terminal, SCREEN);
    hover_part(
        &mut custom_runtime,
        &mut custom_terminal,
        SCREEN,
        TOOLBAR,
        PartRef::of(Part::OVERFLOW),
    );
    advance(&mut custom_runtime, &mut custom_terminal, SCREEN, 600);
    assert!(custom_runtime.is_open(HOVER_TOOLTIP));
    assert!(screen_text(&custom_terminal).contains("More commands"));
    assert!(screen_text(&custom_terminal).contains("M"));
}

#[test]
fn escape_suppresses_the_same_hover_until_native_leave() {
    let row_key = ItemKey::text("escape-tooltip");
    let host = ToolbarHost::new(vec![ToolItem::new("Run").tip("Dismiss me")], vec![row_key]);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(runtime.is_open(HOVER_TOOLTIP));

    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Esc);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    advance(&mut runtime, &mut terminal, SCREEN, 700);
    assert!(
        !runtime.is_open(HOVER_TOOLTIP),
        "the old deadline cannot reopen a dismissed tip"
    );

    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Move,
        Position::new(2, 6),
    );
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    advance(&mut runtime, &mut terminal, SCREEN, 599);
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    advance(&mut runtime, &mut terminal, SCREEN, 1);
    assert!(runtime.is_open(HOVER_TOOLTIP));
}

#[test]
fn opened_receipt_is_nonterminal_and_immediate_escape_suppresses_until_leave() {
    let row_key = ItemKey::text("opened-receipt-tooltip");
    let mut host = ToolbarHost::new(vec![ToolItem::new("Run").tip("Stable tip")], vec![row_key]);
    host.tooltip_policy = ToolbarTooltipPolicy::DEFAULT.delay_ms(0);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);

    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    assert!(runtime.is_open(HOVER_TOOLTIP));
    advance(&mut runtime, &mut terminal, SCREEN, 1);
    assert!(
        runtime.is_open(HOVER_TOOLTIP),
        "claiming Opened on a later update keeps the live tip active"
    );

    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Move,
        Position::new(2, 6),
    );
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(!runtime.is_open(HOVER_TOOLTIP));

    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    assert!(runtime.is_open(HOVER_TOOLTIP));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Esc);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    advance(&mut runtime, &mut terminal, SCREEN, 700);
    assert!(
        !runtime.is_open(HOVER_TOOLTIP),
        "Esc queued with Opened keeps the same source dismissed"
    );
    let source = runtime
        .area_of_part(TOOLBAR, row_part(row_key))
        .expect("dismissed source retains committed geometry");
    let source_position = Position::new(source.x.saturating_add(source.width / 2), source.y);
    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Move,
        source_position,
    );
    render(&mut runtime, &mut terminal, SCREEN);
    advance(&mut runtime, &mut terminal, SCREEN, 700);
    assert!(
        !runtime.is_open(HOVER_TOOLTIP),
        "Esc dismissal survives an unsuppressed move over the same source"
    );

    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Move,
        Position::new(2, 6),
    );
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    assert!(runtime.is_open(HOVER_TOOLTIP));
}

#[test]
fn immediate_source_press_before_opened_receipt_suppresses_until_leave() {
    let row_key = ItemKey::text("immediate-press-tooltip");
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("Run").tip("Press to run")],
        vec![row_key],
    );
    host.tooltip_policy = ToolbarTooltipPolicy::DEFAULT.delay_ms(0);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);

    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    assert!(runtime.is_open(HOVER_TOOLTIP));
    let source = runtime
        .area_of_part(TOOLBAR, row_part(row_key))
        .expect("opened source has committed geometry");
    let source_position = Position::new(source.x.saturating_add(source.width / 2), source.y);

    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Down,
        source_position,
    );
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Up,
        source_position,
    );
    render(&mut runtime, &mut terminal, SCREEN);
    assert_eq!(runtime.app().events.len(), 1);
    assert_eq!(runtime.app().events[0].key, row_key);

    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Move,
        source_position,
    );
    render(&mut runtime, &mut terminal, SCREEN);
    advance(&mut runtime, &mut terminal, SCREEN, 700);
    assert!(
        !runtime.is_open(HOVER_TOOLTIP),
        "source press before the Opened receipt suppresses same-source hover"
    );

    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Move,
        Position::new(2, 6),
    );
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    assert!(runtime.is_open(HOVER_TOOLTIP));
}

#[test]
fn source_press_activates_once_and_suppresses_until_leave() {
    let row_key = ItemKey::text("press-tooltip");
    let host = ToolbarHost::new(
        vec![ToolItem::new("Run").tip("Press to run")],
        vec![row_key],
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(runtime.is_open(HOVER_TOOLTIP));

    let source = runtime
        .area_of_part(TOOLBAR, row_part(row_key))
        .expect("source row remains published");
    click(
        &mut runtime,
        &mut terminal,
        SCREEN,
        Position::new(source.x.saturating_add(source.width / 2), source.y),
    );
    assert_eq!(runtime.app().events.len(), 1);
    assert_eq!(runtime.app().events[0].key, row_key);
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    advance(&mut runtime, &mut terminal, SCREEN, 700);
    assert!(!runtime.is_open(HOVER_TOOLTIP));

    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Move,
        Position::new(2, 6),
    );
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(runtime.is_open(HOVER_TOOLTIP));
}

#[test]
fn cancelled_source_press_recovers_after_removal_release_and_real_reentry() {
    let row_key = ItemKey::text("cancelled-press-tooltip");
    let source_item = ToolItem::new("Run").tip("Press to run");
    let host = ToolbarHost::new(vec![source_item.clone()], vec![row_key]);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(runtime.is_open(HOVER_TOOLTIP));

    let source = runtime
        .area_of_part(TOOLBAR, row_part(row_key))
        .expect("the configured source has committed geometry");
    let source_position = Position::new(source.x.saturating_add(source.width / 2), source.y);
    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Down,
        source_position,
    );
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(!runtime.is_open(HOVER_TOOLTIP));

    let outside = Position::new(2, 6);
    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Move,
        outside,
    );
    render(&mut runtime, &mut terminal, SCREEN);
    advance(&mut runtime, &mut terminal, SCREEN, 700);
    assert!(
        !runtime.is_open(HOVER_TOOLTIP),
        "moving away while the native press is still held is not a leave"
    );

    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Move,
        source_position,
    );
    render(&mut runtime, &mut terminal, SCREEN);
    advance(&mut runtime, &mut terminal, SCREEN, 700);
    assert!(
        !runtime.is_open(HOVER_TOOLTIP),
        "returning to the same source while Down is still held does not restart timing"
    );

    {
        let app = runtime.app_mut();
        app.items.clear();
        app.keys.clear();
    }
    render(&mut runtime, &mut terminal, SCREEN);
    mouse(&mut runtime, &mut terminal, SCREEN, MouseKind::Up, outside);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(
        runtime.app().events.is_empty(),
        "removal cancels activation"
    );

    {
        let app = runtime.app_mut();
        app.items.push(source_item);
        app.keys.push(row_key);
    }
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 599);
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    advance(&mut runtime, &mut terminal, SCREEN, 1);
    assert!(runtime.is_open(HOVER_TOOLTIP));
}

#[test]
fn reused_toolbar_state_starts_a_fresh_interval_for_the_new_owner() {
    let row_key = ItemKey::text("owner-pending-tooltip");
    let host = ToolbarHost::new(
        vec![ToolItem::new("Run").tip("Same configured tip")],
        vec![row_key],
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 300);
    assert!(!runtime.is_open(HOVER_TOOLTIP));

    runtime.app_mut().toolbar_id = OTHER_TOOLBAR;
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        OTHER_TOOLBAR,
        row_part(row_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 599);
    assert!(!runtime.is_open(OTHER_TOOLBAR.sub("hover-tooltip")));
    advance(&mut runtime, &mut terminal, SCREEN, 1);
    assert!(runtime.is_open(OTHER_TOOLBAR.sub("hover-tooltip")));
    assert!(!runtime.is_open(HOVER_TOOLTIP));
}

#[test]
fn reused_toolbar_state_retires_the_old_owner_opening() {
    let row_key = ItemKey::text("owner-open-tooltip");
    let host = ToolbarHost::new(
        vec![ToolItem::new("Run").tip("Same configured tip")],
        vec![row_key],
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(runtime.is_open(HOVER_TOOLTIP));

    runtime.app_mut().toolbar_id = OTHER_TOOLBAR;
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        OTHER_TOOLBAR,
        row_part(row_key),
    );
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    assert!(!runtime.is_open(OTHER_TOOLBAR.sub("hover-tooltip")));
    advance(&mut runtime, &mut terminal, SCREEN, 599);
    assert!(!runtime.is_open(OTHER_TOOLBAR.sub("hover-tooltip")));
    advance(&mut runtime, &mut terminal, SCREEN, 1);
    assert!(runtime.is_open(OTHER_TOOLBAR.sub("hover-tooltip")));
}

#[test]
fn reused_toolbar_state_does_not_transfer_escape_suppression() {
    let row_key = ItemKey::text("owner-dismissed-tooltip");
    let host = ToolbarHost::new(
        vec![ToolItem::new("Run").tip("Same configured tip")],
        vec![row_key],
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(runtime.is_open(HOVER_TOOLTIP));
    key(&mut runtime, &mut terminal, SCREEN, KeyCode::Esc);
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(!runtime.is_open(HOVER_TOOLTIP));

    runtime.app_mut().toolbar_id = OTHER_TOOLBAR;
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        OTHER_TOOLBAR,
        row_part(row_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 599);
    assert!(!runtime.is_open(OTHER_TOOLBAR.sub("hover-tooltip")));
    advance(&mut runtime, &mut terminal, SCREEN, 1);
    assert!(runtime.is_open(OTHER_TOOLBAR.sub("hover-tooltip")));
}

#[test]
fn each_toolbar_keeps_its_own_hover_deadline_and_tip_identity() {
    let first_id = TOOLBAR;
    let second_id = OTHER_TOOLBAR;
    let third_id = THIRD_TOOLBAR;
    let make_toolbar = |id, label, tip| Toolbar::new(id, vec![ToolItem::new(label).tip(tip)]);
    let host = MultiToolbarHost {
        toolbars: vec![
            (
                make_toolbar(first_id, "First", "First owner's tip"),
                ToolbarState::default(),
                Rect::new(5, 1, 26, 1),
            ),
            (
                make_toolbar(second_id, "Second", "Second owner's tip"),
                ToolbarState::default(),
                Rect::new(5, 3, 26, 1),
            ),
            (
                make_toolbar(third_id, "Third", "Third owner's tip"),
                ToolbarState::default(),
                Rect::new(5, 5, 26, 1),
            ),
        ],
    };
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    let first_key = ItemKey::index(0);
    let second_key = ItemKey::index(0);
    let third_key = ItemKey::index(0);

    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        first_id,
        row_part(first_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(runtime.is_open(first_id.sub("hover-tooltip")));
    assert!(screen_text(&terminal).contains("First owner's tip"));

    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        second_id,
        row_part(second_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(!runtime.is_open(first_id.sub("hover-tooltip")));
    assert!(runtime.is_open(second_id.sub("hover-tooltip")));
    assert!(screen_text(&terminal).contains("Second owner's tip"));

    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        third_id,
        row_part(third_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(!runtime.is_open(second_id.sub("hover-tooltip")));
    assert!(runtime.is_open(third_id.sub("hover-tooltip")));
    assert!(screen_text(&terminal).contains("Third owner's tip"));
}

#[test]
fn changed_tip_content_restarts_the_current_hover_interval() {
    let row_key = ItemKey::text("mutable-tip");
    let host = ToolbarHost::new(
        vec![ToolItem::new("Run").tip("Original content")],
        vec![row_key],
    );
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(
        &mut runtime,
        &mut terminal,
        SCREEN,
        TOOLBAR,
        row_part(row_key),
    );
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    assert!(runtime.is_open(HOVER_TOOLTIP));

    runtime.app_mut().items[0].tip = Some("Replacement content".to_owned());
    mouse(
        &mut runtime,
        &mut terminal,
        SCREEN,
        MouseKind::Move,
        Position::new(7, 1),
    );
    render(&mut runtime, &mut terminal, SCREEN);
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    assert!(!screen_text(&terminal).contains("Original content"));
    advance(&mut runtime, &mut terminal, SCREEN, 599);
    assert!(!runtime.is_open(HOVER_TOOLTIP));
    advance(&mut runtime, &mut terminal, SCREEN, 1);
    assert!(runtime.is_open(HOVER_TOOLTIP));
    assert!(screen_text(&terminal).contains("Replacement content"));
}

#[test]
fn clicking_through_tip_activates_the_actual_underlay_once() {
    let key = ItemKey::text("click-through-source");
    let mut host = ToolbarHost::new(
        vec![ToolItem::new("Run").tip("Click-through explanation")],
        vec![key],
    );
    host.next_area = Rect::new(5, 2, 18, 1);
    let (mut runtime, mut terminal) = runtime_for(host);
    render(&mut runtime, &mut terminal, SCREEN);
    hover_part(&mut runtime, &mut terminal, SCREEN, TOOLBAR, row_part(key));
    advance(&mut runtime, &mut terminal, SCREEN, 600);
    let tip = runtime
        .layer_area(HOVER_TOOLTIP)
        .expect("managed tip has native layer geometry");
    assert_eq!(tip.y, 2);
    click(
        &mut runtime,
        &mut terminal,
        SCREEN,
        Position::new(tip.x.saturating_add(2), tip.y),
    );
    assert_eq!(runtime.app().next_activations, 1);
    assert_eq!(
        runtime.app().events.len(),
        0,
        "the tooltip click was not retargeted to its source"
    );
}
