use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::layout::{Position, Rect};
use termrock::blocks::SplitBlock;
use termrock::composition::{self, Group};
use termrock::controls::{Button, SplitPane, SplitPaneState};
use termrock::layout::{Composition, Constraints, Insets, SplitAxis};
use termrock::{App, Cx, Id, Input, KeyModifiers, Mouse, MouseKind, Response, Runtime, Theme};

const SPLIT: Id = Id::root("consumer.workspace");
const EXPLORER: Id = Id::root("consumer.explorer");
const DETAIL: Id = Id::root("consumer.detail");
const ROWS: [Id; 10] = [
    Id::root("consumer.minimum.row.0"),
    Id::root("consumer.minimum.row.1"),
    Id::root("consumer.minimum.row.2"),
    Id::root("consumer.minimum.row.3"),
    Id::root("consumer.minimum.row.4"),
    Id::root("consumer.minimum.row.5"),
    Id::root("consumer.minimum.row.6"),
    Id::root("consumer.minimum.row.7"),
    Id::root("consumer.minimum.row.8"),
    Id::root("consumer.minimum.row.9"),
];
const MINIMUM_SPLIT: Id = Id::root("consumer.minimum.split");
const MINIMUM_SECOND: Id = Id::root("consumer.minimum.second");
const WRAPPED_SPLIT: Id = Id::root("consumer.wrap.split");
const WRAPPED_SECOND: Id = Id::root("consumer.wrap.second");
const WRAPPED: [Id; 10] = [
    Id::root("consumer.wrap.0"),
    Id::root("consumer.wrap.1"),
    Id::root("consumer.wrap.2"),
    Id::root("consumer.wrap.3"),
    Id::root("consumer.wrap.4"),
    Id::root("consumer.wrap.5"),
    Id::root("consumer.wrap.6"),
    Id::root("consumer.wrap.7"),
    Id::root("consumer.wrap.8"),
    Id::root("consumer.wrap.9"),
];

struct WorkspaceConsumer {
    split_state: SplitPaneState,
    domain_actions: usize,
}

impl WorkspaceConsumer {
    fn new() -> Self {
        Self {
            split_state: SplitPaneState::new(38),
            domain_actions: 0,
        }
    }
}

impl App for WorkspaceConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        SplitBlock::new(SplitPane::new(SPLIT, SplitAxis::Horizontal).resizable(true))
            .update(cx, &mut self.split_state)
            .erase()
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        let explorer_button = Button::new(EXPLORER, "Files · 18");
        let detail_button = Button::new(DETAIL, "Workspace source");
        let explorer = Group::column([composition::button(&explorer_button)])
            .spec(Composition::new().padding(Insets::all(1)));
        let detail = Group::column([composition::button(&detail_button)])
            .spec(Composition::new().padding(Insets::all(1)));
        let split = SplitBlock::new(SplitPane::new(SPLIT, SplitAxis::Horizontal).resizable(true))
            .first(&explorer)
            .second(&detail);
        let measured = split
            .measure(
                ui,
                &self.split_state,
                Constraints::loose(ui.full().width, ui.full().height),
            )
            .expect("consumer groups have valid intrinsic measurements");
        assert!(measured.min.0 > 0 && measured.min.1 > 0);
        split
            .show(ui, &self.split_state)
            .expect("consumer groups fit within the supported composition depth");
    }
}

fn render<T: App>(runtime: &mut Runtime<T>, terminal: &mut Terminal<TestBackend>, area: Rect) {
    let prepared = runtime.prepare(area).expect("runtime prepares a frame");
    terminal
        .draw(|frame| {
            runtime
                .draw_prepared(prepared, frame)
                .expect("prepared frame matches terminal bounds");
        })
        .expect("test backend draws successfully");
    runtime
        .acknowledge_presented(prepared)
        .expect("test frame is acknowledged");
}

#[test]
fn split_block_is_a_public_data_driven_consumer_and_uses_live_split_geometry() {
    let area = Rect::new(0, 0, 60, 12);
    let mut runtime = Runtime::new(WorkspaceConsumer::new(), Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    render(&mut runtime, &mut terminal, area);

    let explorer = runtime
        .area_of(EXPLORER)
        .expect("explorer control is composed");
    let detail = runtime.area_of(DETAIL).expect("detail control is composed");
    assert!(explorer.width > 0 && detail.width > 0);
    assert!(explorer.x + explorer.width <= detail.x);
    assert_eq!(runtime.app().split_state.percent(), 38);
    assert_eq!(runtime.app().domain_actions, 0, "measure and draw are pure");

    let seam = runtime
        .area_of_part(SPLIT, termrock::PartRef::of(termrock::Part::SEAM))
        .expect("the shared SplitPane owns the seam hit region");
    let press = Position::new(seam.x, seam.y);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: press,
            mods: KeyModifiers::NONE,
        }))
        .expect("seam press routes to SplitPane");
    render(&mut runtime, &mut terminal, area);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Drag,
            pos: Position::new(34, seam.y),
            mods: KeyModifiers::NONE,
        }))
        .expect("seam drag routes to SplitPane");
    render(&mut runtime, &mut terminal, area);
    assert_ne!(runtime.app().split_state.percent(), 38);
    assert_eq!(runtime.app().domain_actions, 0);

    let retained_percent = runtime.app().split_state.percent();
    let resized = Rect::new(0, 0, 80, 12);
    let _ = runtime
        .handle(Input::Resize(resized.width, resized.height))
        .expect("resize invalidates the shared split layout");
    terminal = Terminal::new(TestBackend::new(resized.width, resized.height)).unwrap();
    render(&mut runtime, &mut terminal, resized);
    let explorer = runtime.area_of(EXPLORER).expect("explorer after resize");
    let detail = runtime.area_of(DETAIL).expect("detail after resize");
    assert_eq!(runtime.app().split_state.percent(), retained_percent);
    assert!(explorer.width > 0 && detail.width > 0);
    assert!(explorer.right() <= detail.x);

    let narrow = Rect::new(0, 0, 2, 1);
    let _ = runtime
        .handle(Input::Resize(narrow.width, narrow.height))
        .expect("narrow resize invalidates the shared split layout");
    terminal = Terminal::new(TestBackend::new(narrow.width, narrow.height)).unwrap();
    render(&mut runtime, &mut terminal, narrow);
    assert_eq!(runtime.app().split_state.percent(), retained_percent);
    assert_eq!(runtime.app().domain_actions, 0);
}

struct MinimumConsumer {
    state: SplitPaneState,
}

impl App for MinimumConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        SplitBlock::new(SplitPane::new(MINIMUM_SPLIT, SplitAxis::Vertical).resizable(true))
            .update(cx, &mut self.state)
            .erase()
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        let buttons = ROWS.map(|id| Button::new(id, "Row"));
        let first = Group::column(buttons.iter().map(composition::button));
        let second_button = Button::new(MINIMUM_SECOND, "Detail");
        let second = Group::column([composition::button(&second_button)]);
        let block =
            SplitBlock::new(SplitPane::new(MINIMUM_SPLIT, SplitAxis::Vertical).resizable(true))
                .first(&first)
                .second(&second);
        let size = block
            .measure(
                ui,
                &self.state,
                Constraints::loose(ui.full().width, ui.full().height),
            )
            .expect("the public groups have valid measurements");
        assert_eq!(
            size.min.1, 28,
            "ratio-aware minimum keeps every row visible"
        );
        block
            .show(ui, &self.state)
            .expect("both groups fit the supported depth");
    }
}

#[test]
fn measured_split_minimum_preserves_children_at_the_current_ratio() {
    let area = Rect::new(0, 0, 40, 28);
    let mut runtime = Runtime::new(
        MinimumConsumer {
            state: SplitPaneState::new(38),
        },
        Theme::junie(),
    );
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    render(&mut runtime, &mut terminal, area);

    let first = runtime
        .area_of(ROWS[9])
        .expect("the last row remains in the first pane at the advertised minimum");
    let second = runtime
        .area_of(MINIMUM_SECOND)
        .expect("the second pane remains visible at the advertised minimum");
    assert!(first.bottom() <= second.y);
}

struct WrapConsumer {
    state: SplitPaneState,
}

impl App for WrapConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        SplitBlock::new(SplitPane::new(WRAPPED_SPLIT, SplitAxis::Horizontal).resizable(true))
            .update(cx, &mut self.state)
            .erase()
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        let buttons = WRAPPED.map(|id| Button::new(id, "Action"));
        let first = Group::wrap(
            buttons
                .iter()
                .map(composition::button)
                .map(|button| button.min(10, 1)),
        );
        let second_button = Button::new(WRAPPED_SECOND, "Details");
        let second = Group::column([composition::button(&second_button)]);
        let block =
            SplitBlock::new(SplitPane::new(WRAPPED_SPLIT, SplitAxis::Horizontal).resizable(true))
                .first(&first)
                .second(&second);
        let measured = block
            .measure(
                ui,
                &self.state,
                Constraints::loose(ui.full().width, ui.full().height),
            )
            .expect("the public groups have valid measurements");
        assert_eq!(
            measured.preferred.1, 4,
            "first group wraps at its live pane width"
        );
        block
            .show(ui, &self.state)
            .expect("both groups fit the supported depth");
    }
}

#[test]
fn measured_split_uses_allocated_width_for_wrapping_groups() {
    let area = Rect::new(0, 0, 80, 4);
    let mut runtime = Runtime::new(
        WrapConsumer {
            state: SplitPaneState::new(38),
        },
        Theme::junie(),
    );
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    render(&mut runtime, &mut terminal, area);

    for id in WRAPPED {
        assert!(
            runtime.area_of(id).is_some(),
            "wrapped control {id:?} remains visible"
        );
    }
}

struct DeepMeasureConsumer;

impl App for DeepMeasureConsumer {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        let mut nested = Group::wrap(std::iter::empty());
        for _ in 0..8_000 {
            nested = Group::row([nested.into_child()]);
        }
        assert_eq!(
            nested.measure(ui, Constraints::loose(10, 4)),
            Err(composition::CompositionError::DepthExceeded {
                depth: 8_001,
                maximum: 128,
            })
        );
    }
}

#[test]
fn over_depth_group_measure_rejects_before_recursive_conversion() {
    let area = Rect::new(0, 0, 10, 4);
    let mut runtime = Runtime::new(DeepMeasureConsumer, Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    render(&mut runtime, &mut terminal, area);
}
