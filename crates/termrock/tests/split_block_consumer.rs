use std::cell::Cell;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::layout::{Position, Rect};
use termrock::blocks::SplitBlock;
use termrock::composition::{self, Group};
use termrock::controls::{Button, SplitPane, SplitPaneState};
use termrock::layout::{Composition, Constraints, Insets, Size, SplitAxis};
use termrock::{
    App, Cx, Id, Input, KeyModifiers, Maximized, Mouse, MouseKind, Response, Runtime, Theme,
};

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

const LIVE_INITIAL_SPLIT: Id = Id::root("consumer.live-initial.split");
const LIVE_INITIAL_DETAIL: Id = Id::root("consumer.live-initial.detail");
const LIVE_INITIAL_ITEMS: [Id; 10] = [
    Id::root("consumer.live-initial.item.0"),
    Id::root("consumer.live-initial.item.1"),
    Id::root("consumer.live-initial.item.2"),
    Id::root("consumer.live-initial.item.3"),
    Id::root("consumer.live-initial.item.4"),
    Id::root("consumer.live-initial.item.5"),
    Id::root("consumer.live-initial.item.6"),
    Id::root("consumer.live-initial.item.7"),
    Id::root("consumer.live-initial.item.8"),
    Id::root("consumer.live-initial.item.9"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LiveInitialMeasurements {
    live: Size,
    initial: Size,
    first_group: Size,
    second_group: Size,
}

struct LiveInitialConsumer {
    live_state: SplitPaneState,
    initial_query: SplitPaneState,
    domain_actions: usize,
    measurements: Cell<Option<LiveInitialMeasurements>>,
}

impl LiveInitialConsumer {
    fn new() -> Self {
        Self {
            live_state: SplitPaneState::new(38),
            initial_query: SplitPaneState::new(38),
            domain_actions: 0,
            measurements: Cell::new(None),
        }
    }
}

impl App for LiveInitialConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        SplitBlock::new(SplitPane::new(LIVE_INITIAL_SPLIT, SplitAxis::Horizontal).resizable(true))
            .update(cx, &mut self.live_state)
            .erase()
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        let buttons = LIVE_INITIAL_ITEMS.map(|id| Button::new(id, "Long action item"));
        let first = Group::wrap(
            buttons
                .iter()
                .map(composition::button)
                .map(|button| button.min(10, 1)),
        );
        let detail_button = Button::new(LIVE_INITIAL_DETAIL, "Workspace details");
        let second = Group::column([composition::button(&detail_button).min(18, 1)])
            .spec(Composition::new().cross_align(termrock::layout::CrossAlign::Start));
        let split = SplitBlock::new(
            SplitPane::new(LIVE_INITIAL_SPLIT, SplitAxis::Horizontal).resizable(true),
        )
        .first(&first)
        .second(&second);
        let constraints = Constraints::loose(ui.full().width, ui.full().height);

        let live_before_first_group_measure = self.live_state;
        let initial_before_first_group_measure = self.initial_query;
        let first_group = first
            .measure(ui, constraints)
            .expect("the public wrapping group has a valid measurement");
        assert_eq!(self.live_state, live_before_first_group_measure);
        assert_eq!(self.initial_query, initial_before_first_group_measure);

        let live_before_second_group_measure = self.live_state;
        let initial_before_second_group_measure = self.initial_query;
        let second_group = second
            .measure(ui, constraints)
            .expect("the public details group has a valid measurement");
        assert_eq!(self.live_state, live_before_second_group_measure);
        assert_eq!(self.initial_query, initial_before_second_group_measure);

        let live_before_measure = self.live_state;
        let initial_before_live_measure = self.initial_query;
        let live = split
            .measure(ui, &self.live_state, constraints)
            .expect("the public wrapping and detail groups have valid measurements");
        assert_eq!(self.live_state, live_before_measure);
        assert_eq!(self.initial_query, initial_before_live_measure);

        let live_before_initial_measure = self.live_state;
        let initial_before_measure = self.initial_query;
        let initial = split
            .measure(ui, &self.initial_query, constraints)
            .expect("the separately constructed initial query has valid measurements");
        assert_eq!(self.live_state, live_before_initial_measure);
        assert_eq!(self.initial_query, initial_before_measure);

        self.measurements.set(Some(LiveInitialMeasurements {
            live,
            initial,
            first_group,
            second_group,
        }));
        let live_before_show = self.live_state;
        split
            .show(ui, &self.live_state)
            .expect("the public groups fit within the supported composition depth");
        assert_eq!(self.live_state, live_before_show);
        assert_eq!(
            self.domain_actions, 0,
            "layout and draw do not run domain actions"
        );
    }
}

fn live_initial_measurements(runtime: &Runtime<LiveInitialConsumer>) -> LiveInitialMeasurements {
    runtime
        .app()
        .measurements
        .get()
        .expect("draw records both live and separate initial measurements")
}

fn assert_measurement_valid(size: Size, constraints: Constraints) {
    assert!(
        size.min.0 <= size.preferred.0,
        "minimum width exceeds preferred width"
    );
    assert!(
        size.min.1 <= size.preferred.1,
        "minimum height exceeds preferred height"
    );
    assert!(
        size.preferred.0 <= constraints.max.0,
        "preferred width exceeds constraints"
    );
    assert!(
        size.preferred.1 <= constraints.max.1,
        "preferred height exceeds constraints"
    );
}

fn live_initial_pane_constraints(state: &SplitPaneState, area: Rect) -> (Constraints, Constraints) {
    SplitPane::new(LIVE_INITIAL_SPLIT, SplitAxis::Horizontal)
        .resizable(true)
        .pane_constraints(state, Constraints::loose(area.width, area.height))
}

#[test]
fn live_split_measurement_and_maximize_policy_preserve_a_separate_initial_query() {
    let area = Rect::new(0, 0, 300, 16);
    let initial_state = SplitPaneState::new(38);
    let mut runtime = Runtime::new(LiveInitialConsumer::new(), Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    render(&mut runtime, &mut terminal, area);

    assert_eq!(runtime.app().live_state, initial_state);
    assert_eq!(runtime.app().initial_query, initial_state);
    assert_eq!(runtime.app().domain_actions, 0);
    let constraints = Constraints::loose(area.width, area.height);
    let before = live_initial_measurements(&runtime);
    assert_eq!(
        before.live, before.initial,
        "matching states and inputs agree"
    );
    assert_measurement_valid(before.live, constraints);
    assert_measurement_valid(before.initial, constraints);

    let first_item_before = runtime
        .area_of(LIVE_INITIAL_ITEMS[0])
        .expect("the first wrapped item is visible");
    let last_item_before = runtime
        .area_of(LIVE_INITIAL_ITEMS[9])
        .expect("the last wrapped item is visible");
    assert_ne!(first_item_before.y, last_item_before.y);
    let initial_state_before_input = runtime.app().initial_query;

    let seam = runtime
        .area_of_part(
            LIVE_INITIAL_SPLIT,
            termrock::PartRef::of(termrock::Part::SEAM),
        )
        .expect("the shared SplitPane registers its actual seam geometry");
    let press = Position::new(seam.x + seam.width / 2, seam.y + seam.height / 2);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: press,
            mods: KeyModifiers::NONE,
        }))
        .expect("the seam press routes to SplitPane");
    assert_eq!(runtime.capture_owner(), Some(LIVE_INITIAL_SPLIT));
    render(&mut runtime, &mut terminal, area);

    let drag = Position::new(210, press.y);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Drag,
            pos: drag,
            mods: KeyModifiers::NONE,
        }))
        .expect("the captured seam drag routes to SplitPane");
    assert_eq!(runtime.capture_owner(), Some(LIVE_INITIAL_SPLIT));
    render(&mut runtime, &mut terminal, area);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: drag,
            mods: KeyModifiers::NONE,
        }))
        .expect("the seam release routes to SplitPane");
    assert_eq!(runtime.capture_owner(), None, "Up releases seam capture");
    render(&mut runtime, &mut terminal, area);

    let post_drag_state = runtime.app().live_state;
    let after_drag = live_initial_measurements(&runtime);
    assert_ne!(post_drag_state.basis_points(), initial_state.basis_points());
    assert_eq!(runtime.app().initial_query, initial_state_before_input);
    assert_eq!(after_drag.initial, before.initial);
    assert_ne!(after_drag.live.preferred.0, before.live.preferred.0);
    assert_ne!(after_drag.live.preferred.1, before.live.preferred.1);
    assert_measurement_valid(after_drag.live, constraints);
    assert_measurement_valid(after_drag.initial, constraints);
    assert_eq!(runtime.app().domain_actions, 0);

    let post_release_last_item = runtime
        .area_of(LIVE_INITIAL_ITEMS[9])
        .expect("the wrapped item remains registered after release");
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Drag,
            pos: Position::new(area.width.saturating_sub(1), press.y),
            mods: KeyModifiers::NONE,
        }))
        .expect("an unpressed drag is accepted without capture");
    render(&mut runtime, &mut terminal, area);
    assert_eq!(runtime.capture_owner(), None);
    assert_eq!(runtime.app().live_state, post_drag_state);
    assert_eq!(
        runtime.area_of(LIVE_INITIAL_ITEMS[9]),
        Some(post_release_last_item),
        "a drag after release does not resize the split"
    );
    assert_eq!(live_initial_measurements(&runtime), after_drag);

    let first_item_after = runtime
        .area_of(LIVE_INITIAL_ITEMS[0])
        .expect("the first wrapped item remains visible after resizing");
    let last_item_after = runtime
        .area_of(LIVE_INITIAL_ITEMS[9])
        .expect("the last wrapped item remains visible after resizing");
    assert_eq!(first_item_after.y, last_item_after.y);
    assert_ne!(
        last_item_before, last_item_after,
        "the live row allocation changed"
    );

    let mut expected_first = post_drag_state;
    expected_first.toggle_max(Maximized::First);
    runtime.app_mut().live_state.toggle_max(Maximized::First);
    render(&mut runtime, &mut terminal, area);
    assert_eq!(runtime.app().live_state, expected_first);
    assert_eq!(runtime.app().live_state.maximized(), Maximized::First);
    assert_eq!(runtime.app().initial_query, initial_state_before_input);
    let first_content = runtime
        .area_of(LIVE_INITIAL_ITEMS[0])
        .expect("the maximized first group still draws its content");
    assert!(first_content.width > 0);
    let (first_constraints, second_constraints) =
        live_initial_pane_constraints(&runtime.app().live_state, area);
    assert_eq!(first_constraints.max.0, area.width);
    assert_eq!(second_constraints.max.0, 0);
    assert!(runtime.area_of(LIVE_INITIAL_DETAIL).is_none());
    let max_first = live_initial_measurements(&runtime);
    assert_eq!(max_first.initial, before.initial);
    assert_eq!(max_first.live, max_first.first_group);
    assert_measurement_valid(max_first.live, constraints);
    assert_measurement_valid(max_first.initial, constraints);
    assert_eq!(runtime.app().domain_actions, 0);

    runtime.app_mut().live_state.toggle_max(Maximized::First);
    render(&mut runtime, &mut terminal, area);
    assert_eq!(runtime.app().live_state, post_drag_state);
    assert_eq!(runtime.app().live_state.maximized(), Maximized::None);
    assert_eq!(live_initial_measurements(&runtime), after_drag);
    assert_eq!(runtime.app().initial_query, initial_state_before_input);

    let mut expected_second = post_drag_state;
    expected_second.toggle_max(Maximized::Second);
    runtime.app_mut().live_state.toggle_max(Maximized::Second);
    render(&mut runtime, &mut terminal, area);
    assert_eq!(runtime.app().live_state, expected_second);
    assert_eq!(runtime.app().live_state.maximized(), Maximized::Second);
    assert_eq!(runtime.app().initial_query, initial_state_before_input);
    let second_content = runtime
        .area_of(LIVE_INITIAL_DETAIL)
        .expect("the maximized second group still draws its content");
    assert!(second_content.width > 0);
    let (first_constraints, second_constraints) =
        live_initial_pane_constraints(&runtime.app().live_state, area);
    assert_eq!(first_constraints.max.0, 0);
    assert_eq!(second_constraints.max.0, area.width);
    assert!(runtime.area_of(LIVE_INITIAL_ITEMS[0]).is_none());
    let max_second = live_initial_measurements(&runtime);
    assert_eq!(max_second.initial, before.initial);
    assert_eq!(max_second.live, max_second.second_group);
    assert_measurement_valid(max_second.live, constraints);
    assert_measurement_valid(max_second.initial, constraints);
    assert_eq!(runtime.app().domain_actions, 0);

    runtime.app_mut().live_state.toggle_max(Maximized::Second);
    render(&mut runtime, &mut terminal, area);
    assert_eq!(runtime.app().live_state, post_drag_state);
    assert_eq!(runtime.app().live_state.maximized(), Maximized::None);
    assert_eq!(live_initial_measurements(&runtime), after_drag);
    assert_eq!(runtime.app().initial_query, initial_state_before_input);
    assert_eq!(runtime.app().domain_actions, 0);
}
