use std::cell::Cell;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use termrock_editors::code::{CodeEditor, CodeEditorState};
use termrock_layout::{Constraints, Size};
use termrock_runtime::event::{Input, KeyModifiers, Mouse, MouseKind};
use termrock_runtime::runtime::{App, Runtime};
use termrock_runtime::{Cx, Id, Response, Ui};
use termrock_theme::Theme;

const EDITOR: Id = Id::root("public.consumer.fill_height.editor");
const SCREEN: Rect = Rect::new(0, 0, 64, 12);

struct Consumer {
    rows: u16,
    fill_height: bool,
    state: CodeEditorState,
    drawn: Cell<Rect>,
    measured: Cell<Size>,
}

impl Consumer {
    fn editor(&self) -> CodeEditor<'static> {
        CodeEditor::new(EDITOR, self.rows).fill_height(self.fill_height)
    }
}

impl App for Consumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.editor().update(cx, &mut self.state).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        let editor = self.editor();
        self.measured
            .set(editor.measure(ui, Constraints::loose(area.width, area.height)));
        self.drawn.set(editor.draw(ui, area, &self.state));
    }
}

fn consumer(fill_height: bool) -> Consumer {
    let text = (0..24)
        .map(|line| format!("row {line:02} — 界 e\u{301}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut state = CodeEditorState::new(&text);
    state.begin_edit();
    Consumer {
        rows: 4,
        fill_height,
        state,
        drawn: Cell::new(Rect::ZERO),
        measured: Cell::new(Size::default()),
    }
}

fn draw<A: App>(runtime: &mut Runtime<A>, area: Rect) -> Buffer {
    let mut buffer = Buffer::empty(area);
    runtime.draw_buffer(area, &mut buffer).commit_presented();
    buffer
}

#[test]
fn public_fill_height_uses_allocation_for_measure_draw_focus_and_pointer_hits() {
    let mut runtime = Runtime::new(consumer(true), Theme::junie());
    let _ = runtime.initialize();
    let _buffer = draw(&mut runtime, SCREEN);
    runtime.set_focus(Some(EDITOR));
    let _focused_buffer = draw(&mut runtime, SCREEN);

    assert_eq!(runtime.app().drawn.get(), SCREEN);
    assert_eq!(runtime.app().measured.get().preferred, (64, 12));
    let focus = runtime
        .ring()
        .entries()
        .iter()
        .find(|entry| entry.id == EDITOR)
        .expect("editor owns its allocated focus bounds");
    assert_eq!(focus.area, Rect::new(0, 0, 64, 11));
    let text = runtime
        .area_of_part(
            EDITOR,
            termrock_core::id::PartRef::of(termrock_core::id::Part::TEXT),
        )
        .expect("editor text hit region");
    assert_eq!(text.height, 11);

    let click = Position::new(text.x, text.y.saturating_add(7));
    let _ = termrock_runtime::runtime::stub::deliver(
        &mut runtime,
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: click,
            mods: KeyModifiers::NONE,
        }),
    );
    let _after_click = draw(&mut runtime, SCREEN);
    let expected_line_start = runtime
        .app()
        .state
        .text()
        .split('\n')
        .take(7)
        .map(|line| line.len().saturating_add(1))
        .sum::<usize>();
    assert_eq!(runtime.app().state.cursor_offset(), expected_line_start);
    assert!(
        runtime
            .cursor()
            .is_some_and(|cursor| cursor.y < SCREEN.height)
    );

    // Measurement reads the same configuration and does not edit the consumer state.
    let state = runtime.app().state.clone();
    let measured = runtime.app().measured.get();
    assert_eq!(runtime.app().state, state);
    assert_eq!(measured.preferred.1, SCREEN.height);

    // Resize and narrow allocation remain bounded by the actual root region.
    let narrow = Rect::new(0, 0, 5, 3);
    let _narrow_buffer = draw(&mut runtime, narrow);
    assert_eq!(runtime.app().drawn.get(), narrow);
    assert_eq!(runtime.app().measured.get().preferred.1, 3);
    assert_eq!(
        runtime
            .ring()
            .entries()
            .iter()
            .find(|entry| entry.id == EDITOR)
            .unwrap()
            .area
            .height,
        2
    );
    assert!(
        runtime
            .cursor()
            .is_none_or(|cursor| cursor.x < narrow.width && cursor.y < narrow.height)
    );
}

#[test]
fn row_limit_remains_default_and_fill_can_be_turned_off() {
    let mut runtime = Runtime::new(consumer(false), Theme::junie());
    let _ = runtime.initialize();
    let _buffer = draw(&mut runtime, SCREEN);
    assert_eq!(runtime.app().drawn.get().height, 4);
    assert_eq!(runtime.app().measured.get().preferred.1, 4);

    runtime.app_mut().fill_height = true;
    let _buffer = draw(&mut runtime, SCREEN);
    assert_eq!(runtime.app().drawn.get().height, SCREEN.height);
    assert_eq!(runtime.app().measured.get().preferred.1, SCREEN.height);

    runtime.app_mut().fill_height = false;
    let _buffer = draw(&mut runtime, SCREEN);
    assert_eq!(runtime.app().drawn.get().height, 4);
    assert_eq!(runtime.app().measured.get().preferred.1, 4);
}

#[test]
fn fill_measurement_respects_zero_and_tight_constraints_without_mutating_state() {
    let app = consumer(true);
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();
    let before_zero_area_draw = runtime.app().state.clone();
    let _buffer = draw(&mut runtime, Rect::new(0, 0, 0, 0));
    assert_eq!(runtime.app().drawn.get(), Rect::ZERO);
    assert_eq!(runtime.app().measured.get(), Size::default());
    assert_eq!(runtime.app().state, before_zero_area_draw);
}

#[test]
fn empty_document_still_uses_fill_allocation_safely() {
    let mut app = consumer(true);
    app.state.set_text("");
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();
    let area = Rect::new(0, 0, 12, 5);
    let _buffer = draw(&mut runtime, area);
    assert_eq!(runtime.app().drawn.get(), area);
    assert_eq!(runtime.app().measured.get().preferred, (12, 5));
    assert!(
        runtime
            .cursor()
            .is_none_or(|cursor| cursor.x < area.width && cursor.y < area.height)
    );
}
