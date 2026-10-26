use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::layout::{Position, Rect};
use termrock::{
    App, Cx, Id, Input, ItemKey, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Part, PartRef,
    Response, Runtime, Theme, ToolItem, Toolbar, ToolbarAction, ToolbarActionEvent, ToolbarState,
    Ui, composition,
    layout::{Composition, Insets},
};

const TOOLBAR: Id = Id::root("termrock.tests.toolbar.composition");
const OVERFLOW_MENU: Id = TOOLBAR.sub("more-menu");
const RUN: ItemKey = ItemKey::Text(0xA11CF);
const WRAP: ItemKey = ItemKey::Text(0xA11CE);
const FORMAT: ItemKey = ItemKey::Text(0xA11CD);
const SEARCH: ItemKey = ItemKey::Text(0xA11CC);
const OPTIONS: ItemKey = ItemKey::Text(0xA11CB);
const SCREEN: Rect = Rect::new(0, 0, 24, 10);

struct CompositionConsumer {
    on: bool,
    state: ToolbarState,
    events: Vec<ToolbarActionEvent>,
}

impl CompositionConsumer {
    fn new() -> Self {
        Self {
            on: false,
            state: ToolbarState::default(),
            events: Vec::new(),
        }
    }

    fn toolbar(&self) -> Toolbar {
        Toolbar::new(
            TOOLBAR,
            vec![
                ToolItem::new("Run"),
                ToolItem::new("Wrap").toggle(self.on),
                ToolItem::new("Format"),
                ToolItem::new("Search"),
                ToolItem::new("Options"),
            ],
        )
        .try_item_keys(vec![RUN, WRAP, FORMAT, SEARCH, OPTIONS])
        .expect("the stable key sidecar has one key")
    }

    fn commit(&mut self, event: ToolbarActionEvent) {
        if event.key == WRAP
            && let ToolbarAction::Toggled(_, on) = event.action
        {
            self.on = on;
        }
    }
}

impl App for CompositionConsumer {
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
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let toolbar = self.toolbar();
        composition::Group::column([composition::toolbar(&toolbar, &self.state)])
            .spec(Composition::new().gap(1).padding(Insets::all(1)))
            .show(ui)
            .expect("the Toolbar placement stays within supported composition depth");
    }
}

fn render(runtime: &mut Runtime<CompositionConsumer>, terminal: &mut Terminal<TestBackend>) {
    let prepared = runtime.prepare(SCREEN).expect("runtime prepares a frame");
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

fn ready_for_input(
    runtime: &mut Runtime<CompositionConsumer>,
    terminal: &mut Terminal<TestBackend>,
) {
    if runtime.needs_present() || runtime.needs_settle() {
        render(runtime, terminal);
    }
    assert!(
        !runtime.needs_present() && !runtime.needs_settle(),
        "runtime is ready for one native input"
    );
}

fn mouse(
    runtime: &mut Runtime<CompositionConsumer>,
    terminal: &mut Terminal<TestBackend>,
    kind: MouseKind,
    pos: Position,
) {
    ready_for_input(runtime, terminal);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind,
            pos,
            mods: termrock::KeyModifiers::NONE,
        }))
        .expect("composition Toolbar receives native pointer input");
}

fn key(
    runtime: &mut Runtime<CompositionConsumer>,
    terminal: &mut Terminal<TestBackend>,
    code: KeyCode,
) {
    ready_for_input(runtime, terminal);
    let _ = runtime
        .handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }))
        .expect("composition Toolbar receives native keyboard input");
}

#[test]
fn public_toolbar_adapter_places_and_draws_the_same_native_hit_geometry() {
    let mut runtime = Runtime::new(CompositionConsumer::new(), Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(SCREEN.width, SCREEN.height)).unwrap();
    render(&mut runtime, &mut terminal);

    let root = runtime
        .area_of(TOOLBAR)
        .expect("the adapter delegates Toolbar registration");
    let item = runtime
        .area_of_part(TOOLBAR, termrock::PartRef::item(termrock::Part::ROW, WRAP))
        .expect("the adapter delegates item-part registration");
    assert!(
        root.x > 0 && root.y > 0,
        "composition padding sets a nonzero origin"
    );
    assert!(item.x >= root.x && item.y >= root.y);
    assert!(item.right() <= root.right() && item.bottom() <= root.bottom());
    assert!(item.width > 0 && item.height == 1);
    assert!(
        runtime
            .area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW))
            .is_some(),
        "composition placement keeps the Toolbar's native More part"
    );

    let point = Position::new(item.x, item.y);
    mouse(&mut runtime, &mut terminal, MouseKind::Down, point);
    render(&mut runtime, &mut terminal);
    mouse(&mut runtime, &mut terminal, MouseKind::Up, point);
    render(&mut runtime, &mut terminal);

    assert_eq!(runtime.focus(), Some(TOOLBAR));
    assert!(runtime.app().on);
    assert_eq!(
        runtime.app().events.as_slice(),
        &[ToolbarActionEvent {
            key: WRAP,
            action: ToolbarAction::Toggled(1, true),
        }]
    );

    let checked = Theme::junie()
        .design
        .glyphs
        .get(termrock::theme::GlyphRole::DotOn);
    assert_eq!(
        terminal
            .backend()
            .buffer()
            .cell(Position::new(item.x.saturating_add(1), item.y))
            .map(|cell| cell.symbol()),
        Some(checked),
        "the caller-committed checked value is rendered by the placed Toolbar"
    );
}

#[test]
fn public_toolbar_adapter_preserves_native_overflow_selection_identity() {
    let mut runtime = Runtime::new(CompositionConsumer::new(), Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(SCREEN.width, SCREEN.height)).unwrap();
    render(&mut runtime, &mut terminal);

    assert!(
        runtime
            .area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW))
            .is_some(),
        "the adapter delegates More registration at its placed origin"
    );
    for index in 0..3 {
        assert!(
            runtime
                .area_of_part(
                    OVERFLOW_MENU,
                    PartRef::item(Part::ROW, ItemKey::index(index))
                )
                .is_none(),
            "overflow rows are not registered before the native menu opens"
        );
    }

    let wrap_area = runtime
        .area_of_part(TOOLBAR, PartRef::item(Part::ROW, WRAP))
        .expect("the checked item remains visible beside More");
    let point = Position::new(wrap_area.x, wrap_area.y);
    mouse(&mut runtime, &mut terminal, MouseKind::Down, point);
    render(&mut runtime, &mut terminal);
    mouse(&mut runtime, &mut terminal, MouseKind::Up, point);
    render(&mut runtime, &mut terminal);
    assert!(runtime.app().on);

    key(&mut runtime, &mut terminal, KeyCode::End);
    render(&mut runtime, &mut terminal);
    key(&mut runtime, &mut terminal, KeyCode::Enter);
    render(&mut runtime, &mut terminal);
    assert!(runtime.is_open(OVERFLOW_MENU));
    assert!(
        (0..3).all(|index| {
            runtime
                .area_of_part(
                    OVERFLOW_MENU,
                    PartRef::item(Part::ROW, ItemKey::index(index)),
                )
                .is_some()
        }),
        "the native layer registers all frozen hidden row slots"
    );

    key(&mut runtime, &mut terminal, KeyCode::Down);
    key(&mut runtime, &mut terminal, KeyCode::Enter);
    assert_eq!(
        runtime.app().events.last().copied(),
        Some(ToolbarActionEvent {
            key: SEARCH,
            action: ToolbarAction::Activated(3),
        }),
        "composition preserves the native selected-row receipt through the adapter"
    );
}
