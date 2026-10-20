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
            .measure(ui, Constraints::loose(ui.full().width, ui.full().height))
            .expect("consumer groups have valid intrinsic measurements");
        assert!(measured.min.0 > 0 && measured.min.1 > 0);
        split
            .show(ui, &self.split_state)
            .expect("consumer groups fit within the supported composition depth");
    }
}

fn render(
    runtime: &mut Runtime<WorkspaceConsumer>,
    terminal: &mut Terminal<TestBackend>,
    area: Rect,
) {
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
