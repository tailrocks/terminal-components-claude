use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::layout::{Position, Rect};
use termrock::composition::{self, Group};
use termrock::controls::{Button, Panel, PanelKind};
use termrock::editors::{CodeEditor, CodeEditorState};
use termrock::layout::{Composition, Constraints, Insets};
use termrock::{
    App, Cx, Id, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Response, Runtime, Theme, Ui,
};

const PANEL: Id = Id::root("external.diagnostics.panel");
const RUN: Id = Id::root("external.diagnostics.run");
const EDITOR: Id = Id::root("external.diagnostics.editor");

struct DiagnosticsConsumer {
    button: Button<'static>,
    editor: CodeEditor<'static>,
    editor_state: CodeEditorState,
    activations: usize,
}

impl DiagnosticsConsumer {
    fn new() -> Self {
        let mut editor_state = CodeEditorState::new("SELECT value\né");
        editor_state.begin_edit();
        Self {
            button: Button::new(RUN, "Run check"),
            editor: CodeEditor::new(EDITOR, 3).fill_height(true),
            editor_state,
            activations: 0,
        }
    }
}

impl App for DiagnosticsConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let run = self.button.update(cx);
        if run.activated() {
            self.activations = self.activations.saturating_add(1);
        }
        let edit = self.editor.update(cx, &mut self.editor_state);
        run.erase() | edit.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let panel = Panel::new(PANEL)
            .kind(PanelKind::Framed)
            .title("Diagnostics")
            .meta("live run")
            .badge("λ");
        let body = Group::column([
            composition::button(&self.button).min(8, 1),
            composition::code_editor(&self.editor, &self.editor_state)
                .min(10, 2)
                .flex(1),
        ])
        .spec(Composition::new().gap(1).padding(Insets::all(1)))
        .into_child();
        let composition = Group::column([composition::panel(&panel, body).flex(1)]);
        let before = self.editor_state.text().to_owned();
        let root = ui.full();
        let measured = composition
            .measure(ui, Constraints::loose(root.width, root.height))
            .expect("panel measurement is within the supported depth");
        assert!(measured.preferred.0 > 0 && measured.preferred.1 > 0);
        assert_eq!(self.editor_state.text(), before, "measurement is read-only");
        composition
            .show(ui)
            .expect("panel composition is within the supported depth");
    }
}

fn render<A: App>(runtime: &mut Runtime<A>, terminal: &mut Terminal<TestBackend>, area: Rect) {
    let prepared = runtime.prepare(area).expect("runtime prepares the frame");
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
fn public_panel_composes_live_button_and_editor_with_input_and_resize() {
    let mut runtime = Runtime::new(DiagnosticsConsumer::new(), Theme::junie());
    let initial = Rect::new(0, 0, 46, 12);
    let mut terminal = Terminal::new(TestBackend::new(initial.width, initial.height)).unwrap();
    render(&mut runtime, &mut terminal, initial);

    assert_eq!(
        runtime.app().activations,
        0,
        "measurement and drawing are pure"
    );
    let run_area = runtime.area_of(RUN).expect("button owns a real hit region");
    let editor_area = runtime
        .area_of(EDITOR)
        .expect("editor owns a real hit region");
    let panel_area = runtime
        .area_of(PANEL)
        .expect("panel registers its actual chrome");
    assert!(run_area.x > panel_area.x && run_area.y > panel_area.y);
    assert!(editor_area.x > panel_area.x && editor_area.y > panel_area.y);
    assert!(run_area.x > 0 && run_area.y > 0);
    assert!(editor_area.y > run_area.y);
    assert!(editor_area.width > 0 && editor_area.height > 0);

    let point = Position::new(run_area.x, run_area.y);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: point,
            mods: KeyModifiers::NONE,
        }))
        .expect("pointer press reaches the real button");
    render(&mut runtime, &mut terminal, initial);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: point,
            mods: KeyModifiers::NONE,
        }))
        .expect("pointer release reaches the real button");
    render(&mut runtime, &mut terminal, initial);
    assert_eq!(runtime.app().activations, 1);
    assert_eq!(runtime.focus(), Some(RUN));
    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }))
        .expect("keyboard activation reaches the same button action");
    render(&mut runtime, &mut terminal, initial);
    assert_eq!(runtime.app().activations, 2);

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Tab,
            mods: KeyModifiers::NONE,
        }))
        .expect("keyboard traversal reaches the real editor");
    render(&mut runtime, &mut terminal, initial);
    assert_eq!(runtime.focus(), Some(EDITOR));
    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Char('界'),
            mods: KeyModifiers::NONE,
        }))
        .expect("Unicode input reaches the real editor");
    assert!(runtime.app().editor_state.text().contains('界'));

    let resized = Rect::new(0, 0, 36, 9);
    let mut terminal = Terminal::new(TestBackend::new(resized.width, resized.height)).unwrap();
    render(&mut runtime, &mut terminal, resized);
    let resized_editor = runtime
        .area_of(EDITOR)
        .expect("resize refreshes child layout");
    assert!(resized_editor.width > 0 && resized_editor.height > 0);
    assert!(resized_editor.x.saturating_add(resized_editor.width) <= resized.width);
    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Char('λ'),
            mods: KeyModifiers::NONE,
        }))
        .expect("resized editor receives Unicode input");
    assert!(runtime.app().editor_state.text().contains('λ'));
    assert_eq!(runtime.app().activations, 2);
}

struct DepthConsumer {
    result: std::cell::Cell<Option<composition::CompositionError>>,
}

impl App for DepthConsumer {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let panel = Panel::new(PANEL).title("Too deep");
        let mut nested = Group::wrap(std::iter::empty());
        for _ in 0..8_000 {
            nested = Group::row([nested.into_child()]);
        }
        let button = Button::new(RUN, "Must not draw");
        let body = Group::column([composition::button(&button)]).into_child();
        let body = Group::column([body, nested.into_child()]).into_child();
        let root = Group::column([composition::panel(&panel, body)]);
        self.result.set(root.show(ui).err());
    }
}

#[test]
fn invalid_panel_descendant_tree_is_rejected_before_child_draw() {
    let mut runtime = Runtime::new(
        DepthConsumer {
            result: std::cell::Cell::new(None),
        },
        Theme::junie(),
    );
    let area = Rect::new(0, 0, 30, 8);
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    render(&mut runtime, &mut terminal, area);

    assert!(matches!(
        runtime.app().result.get(),
        Some(composition::CompositionError::DepthExceeded { .. })
    ));
    assert_eq!(runtime.area_of(RUN), None, "rejected tree paints no child");
}
