use ratatui::{Terminal, backend::TestBackend};
use termrock::author::CompositionLeaf;
use termrock::composition::{self, Child, CompositionError, Group};
use termrock::controls::{Button, SplitPane, SplitPaneState};
use termrock::layout::{Constraints, Size, SplitAxis};
use termrock::runtime::Measure;
use termrock::{
    App, AppShell, Buffer, Cx, Dialog, DialogState, Id, Input, Key, KeyCode, KeyModifiers, Mouse,
    MouseKind, Position, Rect, Response, Runtime, TextSection, Theme, Ui,
};

const PANE: Id = Id::root("composition-adapter-behavior.pane");
const LEFT: Id = Id::root("composition-adapter-behavior.left");
const RIGHT: Id = Id::root("composition-adapter-behavior.right");
const DIALOG: Id = Id::root("composition-adapter-behavior.dialog");

fn publish<A: App>(runtime: &mut Runtime<A>, terminal: &mut Terminal<TestBackend>, area: Rect) {
    for _ in 0..16 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        if runtime.needs_present() {
            let prepared = runtime.prepare(area).expect("runtime prepares the frame");
            terminal
                .draw(|frame| {
                    runtime
                        .draw_prepared(prepared, frame)
                        .expect("prepared frame matches terminal bounds");
                })
                .expect("test backend draws");
            runtime
                .acknowledge_presented(prepared)
                .expect("prepared frame is acknowledged");
        }
        if !runtime.needs_settle() && !runtime.needs_present() {
            return;
        }
    }
    panic!("composition adapter frame did not settle");
}

fn symbol(buffer: &Buffer, x: u16, y: u16) -> Option<&str> {
    buffer.cell((x, y)).map(|cell| cell.symbol())
}

fn line(buffer: &Buffer, y: u16) -> String {
    (0..buffer.area().width)
        .filter_map(|x| buffer.cell((x, y)))
        .map(|cell| cell.symbol())
        .collect()
}

fn find(buffer: &Buffer, needle: &str) -> Option<(u16, u16)> {
    let area = *buffer.area();
    for y in 0..area.height {
        for x in 0..area.width {
            let mut found = String::new();
            for offset in 0..u16::try_from(needle.len()).ok()? {
                let Some(cell) = buffer.cell((x.saturating_add(offset), y)) else {
                    break;
                };
                found.push_str(cell.symbol());
                if found == needle {
                    return Some((x, y));
                }
                if !needle.starts_with(&found) {
                    break;
                }
            }
        }
    }
    None
}

struct GroupApp<'a> {
    group: Group<'a>,
    measured: std::cell::Cell<Option<Size>>,
}

impl App for GroupApp<'_> {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let size = self
            .group
            .measure(ui, Constraints::loose(ui.full().width, ui.full().height))
            .expect("test composition is shallow");
        self.measured.set(Some(size));
        self.group.show(ui).expect("test composition is shallow");
    }
}

#[test]
fn text_section_and_app_shell_adapters_measure_and_place_real_content() {
    let section = TextSection::new("界")
        .subtitle("detail")
        .body("body")
        .inset(1);
    let group = Group::column([composition::text_section(&section)]);
    let app = GroupApp {
        group,
        measured: std::cell::Cell::new(None),
    };
    let area = Rect::new(0, 0, 20, 8);
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    publish(&mut runtime, &mut terminal, area);
    let buffer = terminal.backend().buffer();

    assert_eq!(
        runtime.app().measured.get().map(|size| size.preferred),
        Some((20, 4))
    );
    assert_eq!(symbol(buffer, 1, 0), Some("界"));
    assert_eq!(symbol(buffer, 1, 1), Some("d"));
    assert_eq!(symbol(buffer, 1, 3), Some("b"));

    let header = TextSection::new("HEADER").inset(0);
    let body = TextSection::new("BODY").inset(0);
    let footer = TextSection::new("FOOTER").inset(0);
    let shell = AppShell::new().header(1).gap(0).footer(1).inset(0);
    let shell_group = Group::column([composition::app_shell(
        &shell,
        composition::text_section(&header),
        composition::text_section(&body),
        composition::text_section(&footer),
    )]);
    let app = GroupApp {
        group: shell_group,
        measured: std::cell::Cell::new(None),
    };
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    publish(&mut runtime, &mut terminal, area);
    let buffer = terminal.backend().buffer();
    assert_eq!(symbol(buffer, 0, 0), Some("H"));
    assert_eq!(symbol(buffer, 0, 1), Some("B"));
    assert_eq!(symbol(buffer, 0, 7), Some("F"));
}

#[test]
fn text_section_adapters_wrap_as_intrinsic_children_under_narrow_width() {
    let first = TextSection::new("One").body("A").body_row(1);
    let second = TextSection::new("Two").body("B").body_row(1);
    let group = Group::wrap([
        composition::text_section(&first),
        composition::text_section(&second),
    ]);
    let app = GroupApp {
        group,
        measured: std::cell::Cell::new(None),
    };
    let area = Rect::new(0, 0, 8, 6);
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    publish(&mut runtime, &mut terminal, area);
    let buffer = terminal.backend().buffer();

    assert_eq!(
        runtime.app().measured.get().map(|size| size.preferred),
        Some((5, 4))
    );
    assert_eq!(symbol(buffer, 1, 0), Some("O"));
    assert_eq!(symbol(buffer, 1, 1), Some("A"));
    assert_eq!(symbol(buffer, 1, 2), Some("T"));
    assert_eq!(symbol(buffer, 1, 3), Some("B"));
}

struct DialogApp {
    component: Dialog<'static>,
    state: DialogState,
}

impl App for DialogApp {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        let body = TextSection::new("Composed body")
            .inset(0)
            .title_style(ui.surface_style());
        composition::dialog(
            &self.component,
            &self.state,
            composition::text_section(&body),
        )
        .try_draw(ui, area)
        .expect("dialog and its body fit the supported depth");
    }
}

#[test]
fn dialog_adapter_draws_its_composed_body_in_the_component_resolved_area() {
    let app = DialogApp {
        component: Dialog::confirm(DIALOG, "Title", "Question").body_rows(1),
        state: DialogState::default(),
    };
    let area = Rect::new(0, 0, 48, 14);
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    publish(&mut runtime, &mut terminal, area);
    let buffer = terminal.backend().buffer();
    assert!(
        (0..area.height).any(|row| line(buffer, row).contains("Composed body")),
        "{}",
        (0..area.height)
            .map(|row| line(buffer, row))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn depth_group<'a>() -> Group<'a> {
    let mut group = Group::wrap(std::iter::empty());
    for _ in 1..termrock::runtime::MAX_COMPOSITION_DEPTH {
        group = Group::row([group.into_child()]);
    }
    group
}

fn assert_depth_is_fallible(ui: &mut Ui<'_>, child: Child<'_>, nested_depth: usize) {
    let expected = CompositionError::DepthExceeded {
        depth: nested_depth,
        maximum: termrock::runtime::MAX_COMPOSITION_DEPTH,
    };
    assert_eq!(
        child.try_measure(&*ui, Constraints::loose(20, 8)),
        Err(expected)
    );
    assert_eq!(child.try_draw(ui, Rect::new(0, 0, 20, 8)), Err(expected));

    // The infallible framework traits remain safe for direct callers; use
    // Child::try_* when the caller needs to observe a depth error.
    assert_eq!(
        Measure::measure(&child, &*ui, Constraints::loose(20, 8)),
        Size::default()
    );
    CompositionLeaf::draw(&child, ui, Rect::new(0, 0, 20, 8));

    let parent = Group::column([child]);
    let parent_error = CompositionError::DepthExceeded {
        depth: nested_depth + 1,
        maximum: termrock::runtime::MAX_COMPOSITION_DEPTH,
    };
    assert_eq!(
        parent.measure(&*ui, Constraints::loose(20, 8)),
        Err(parent_error)
    );
    assert_eq!(parent.show(ui), Err(parent_error));
}

struct DepthApp;

impl App for DepthApp {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let over = termrock::runtime::MAX_COMPOSITION_DEPTH + 1;

        let split = SplitPane::new(PANE, SplitAxis::Horizontal);
        let split_state = SplitPaneState::default();
        assert_depth_is_fallible(
            ui,
            composition::split_pane(
                &split,
                &split_state,
                depth_group().into_child(),
                Group::wrap(std::iter::empty()).into_child(),
            ),
            over,
        );

        let dialog = Dialog::confirm(DIALOG, "Title", "Question");
        let dialog_state = DialogState::default();
        assert_depth_is_fallible(
            ui,
            composition::dialog(&dialog, &dialog_state, depth_group().into_child()),
            over,
        );

        let shell = AppShell::new();
        assert_depth_is_fallible(
            ui,
            composition::app_shell(
                &shell,
                depth_group().into_child(),
                Group::wrap(std::iter::empty()).into_child(),
                Group::wrap(std::iter::empty()).into_child(),
            ),
            over,
        );

        let text = TextSection::new("leaf");
        let mut nested = composition::text_section(&text);
        for _ in 0..8_000 {
            nested = composition::app_shell(
                &shell,
                nested,
                Group::wrap(std::iter::empty()).into_child(),
                Group::wrap(std::iter::empty()).into_child(),
            );
        }
        let rejected = Group::column([nested]);
        assert_eq!(
            rejected.show(ui),
            Err(CompositionError::DepthExceeded {
                depth: 8_002,
                maximum: termrock::runtime::MAX_COMPOSITION_DEPTH,
            })
        );
    }
}

#[test]
fn split_dialog_and_app_shell_adapters_propagate_depth_without_panics() {
    let area = Rect::new(0, 0, 20, 8);
    let mut runtime = Runtime::new(DepthApp, Theme::junie());
    let _ = runtime.initialize();
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    publish(&mut runtime, &mut terminal, area);
}

struct SplitHost {
    state: SplitPaneState,
    activated: Option<Id>,
}

impl App for SplitHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let left = Button::new(LEFT, "Left");
        let right = Button::new(RIGHT, "Right");
        let split = SplitPane::new(PANE, SplitAxis::Horizontal);
        let left_response = left.update(cx);
        if left_response.activated() {
            self.activated = Some(LEFT);
        }
        let right_response = right.update(cx);
        if right_response.activated() {
            self.activated = Some(RIGHT);
        }
        left_response.erase() | right_response.erase() | split.update(cx, &mut self.state).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let split = SplitPane::new(PANE, SplitAxis::Horizontal);
        let left = Button::new(LEFT, "Left");
        let right = Button::new(RIGHT, "Right");
        Group::column([composition::split_pane(
            &split,
            &self.state,
            composition::button(&left),
            composition::button(&right),
        )])
        .show(ui)
        .expect("split pane composition fits the supported depth");
    }
}

#[test]
fn split_pane_composition_preserves_child_focus_and_activation() {
    let area = Rect::new(0, 0, 44, 10);
    let mut runtime = Runtime::new(
        SplitHost {
            state: SplitPaneState::default(),
            activated: None,
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    publish(&mut runtime, &mut terminal, area);
    let (x, y) = find(terminal.backend().buffer(), "Right").expect("right child is drawn");

    for kind in [MouseKind::Down, MouseKind::Up] {
        let _ = runtime
            .handle(Input::Mouse(Mouse {
                kind,
                pos: Position::new(x, y),
                mods: KeyModifiers::NONE,
            }))
            .expect("split pane child input is dispatched");
        publish(&mut runtime, &mut terminal, area);
    }
    assert_eq!(runtime.app().activated, Some(RIGHT));
    assert_eq!(runtime.focus(), Some(RIGHT));

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }))
        .expect("focused split pane child receives Enter");
    publish(&mut runtime, &mut terminal, area);
    assert_eq!(runtime.app().activated, Some(RIGHT));
}
