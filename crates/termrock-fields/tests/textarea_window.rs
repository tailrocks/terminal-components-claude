#![allow(
    clippy::unwrap_used,
    reason = "focused public API regression assertions"
)]

use ratatui_core::buffer::Buffer;
use termrock_fields::{TextArea, TextAreaState};
use termrock_runtime::author::VerticalWindow;
use termrock_runtime::{
    App, Cx, Focusability, Id, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Position, Rect,
    Response, Runtime, Theme, Ui,
};
use termrock_test_support::Harness;

const ID: Id = Id::root("textarea.window.consumer");
const BACKGROUND: Id = Id::root("textarea.window.background");
const SCREEN: Rect = Rect::new(0, 0, 24, 12);
const VALUE: &str = "zero\none\ntwo\n三é\nfour\nfive\nsix\nseven";

struct Consumer {
    state: TextAreaState,
    value: String,
    window: VerticalWindow,
    focus_on_bootstrap: bool,
}

impl App for Consumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.focus_on_bootstrap && cx.update_cause() == termrock_runtime::UpdateCause::Bootstrap
        {
            cx.focus(ID);
        }
        TextArea::new(ID, 5)
            .blur(termrock_fields::BlurPolicy::Commit)
            .update(cx, &mut self.state, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(BACKGROUND, Rect::new(20, 0, 1, 1), Focusability::Focusable);
        TextArea::new(ID, 5)
            .value(&self.value)
            .placeholder("type here")
            .draw_window(ui, self.window, &self.state);
    }
}

fn runtime(window: VerticalWindow, offset: usize, focus_on_bootstrap: bool) -> Runtime<Consumer> {
    runtime_with_value(window, offset, focus_on_bootstrap, VALUE)
}

fn runtime_with_value(
    window: VerticalWindow,
    offset: usize,
    focus_on_bootstrap: bool,
    value: &str,
) -> Runtime<Consumer> {
    let mut state = TextAreaState::default();
    state.scroll_mut().set_content(8);
    state.scroll_mut().set_viewport(5);
    state.scroll_mut().scroll_to(offset);
    let mut runtime = Runtime::new(
        Consumer {
            state,
            value: value.to_owned(),
            window,
            focus_on_bootstrap,
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    runtime
}

struct LegacyConsumer {
    state: TextAreaState,
    value: String,
}

impl App for LegacyConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == termrock_runtime::UpdateCause::Bootstrap {
            cx.focus(ID);
        }
        TextArea::new(ID, 5)
            .update(cx, &mut self.state, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextArea::new(ID, 5)
            .value(&self.value)
            .draw(ui, Rect::new(2, 2, 14, 5), &self.state);
    }
}

fn draw<A: App>(runtime: &mut Runtime<A>) -> Buffer {
    let mut buffer = Buffer::empty(SCREEN);
    for _ in 0..16 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        if !runtime.needs_settle() {
            break;
        }
    }
    buffer
}

fn row(buffer: &Buffer, y: u16, x: u16, width: u16) -> String {
    (x..x.saturating_add(width))
        .map(|column| buffer[(column, y)].symbol())
        .collect::<String>()
        .trim_end()
        .to_owned()
}

#[test]
fn parent_window_keeps_original_document_rows_at_each_clip_edge() {
    let cases = [
        (
            VerticalWindow::new(Rect::new(2, 2, 14, 2), 5, 0).unwrap(),
            ["one", "two"],
        ),
        (
            VerticalWindow::new(Rect::new(2, 2, 14, 2), 5, 3).unwrap(),
            ["four", "five"],
        ),
    ];
    for (window, expected) in cases {
        let mut runtime = runtime(window, 1, false);
        let before = runtime.app().state.scroll().offset();
        let buffer = draw(&mut runtime);
        assert_eq!(row(&buffer, 2, 4, 8), expected[0]);
        assert_eq!(row(&buffer, 3, 4, 8), expected[1]);
        assert_eq!(runtime.app().state.scroll().offset(), before);
        assert_eq!(runtime.app().state.scroll().viewport_len(), 5);
    }

    let window = VerticalWindow::new(Rect::new(2, 2, 14, 3), 5, 1).unwrap();
    let mut runtime = runtime(window, 1, false);
    let buffer = draw(&mut runtime);
    assert_eq!(row(&buffer, 2, 4, 8), "two");
    assert_eq!(buffer[(4, 3)].symbol(), "三");
    assert_eq!(buffer[(5, 3)].symbol(), " ");
    assert_eq!(buffer[(6, 3)].symbol(), "é");
    assert_eq!(row(&buffer, 4, 4, 8), "four");
}

#[test]
fn pointer_uses_physical_window_and_original_document_coordinates() {
    let window = VerticalWindow::new(Rect::new(2, 2, 14, 2), 5, 2).unwrap();
    let mut runtime = runtime(window, 1, false);
    let _ = draw(&mut runtime);
    for input in [
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: Position::new(4, 2),
            mods: KeyModifiers::NONE,
        }),
        Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: Position::new(4, 2),
            mods: KeyModifiers::NONE,
        }),
        Input::Key(Key {
            code: KeyCode::Char('X'),
            mods: KeyModifiers::NONE,
        }),
    ] {
        let _ = termrock_runtime::stub::deliver(&mut runtime, input);
        let _ = draw(&mut runtime);
    }
    assert_eq!(
        TextArea::new(ID, 5).caret_source_row(&runtime.app().state),
        Some(2)
    );
    let input = Input::Key(Key {
        code: KeyCode::Esc,
        mods: KeyModifiers::NONE,
    });
    let _ = termrock_runtime::stub::deliver(&mut runtime, input);
    let _ = draw(&mut runtime);
    assert_eq!(
        runtime.app().value,
        "zero\none\ntwo\nX三é\nfour\nfive\nsix\nseven"
    );
}

#[test]
fn clipped_window_does_not_publish_a_cursor_outside_visible_text() {
    let window = VerticalWindow::new(Rect::new(2, 2, 14, 1), 5, 0).unwrap();
    let mut runtime = runtime(window, 1, true);
    let buffer = draw(&mut runtime);
    assert!(runtime.cursor_position().is_none());
    assert!(
        runtime
            .registry()
            .area_of_part(
                ID,
                termrock_runtime::PartRef::of(termrock_runtime::Part::TEXT)
            )
            .is_some()
    );
    assert_eq!(row(&buffer, 2, 4, 8), "三 é");
}

#[test]
fn page_commands_keep_the_full_logical_child_viewport() {
    let window = VerticalWindow::new(Rect::new(2, 2, 14, 2), 5, 2).unwrap();
    let mut runtime = runtime(window, 1, true);
    let _ = draw(&mut runtime);

    for input in [
        Input::Key(Key {
            code: KeyCode::Home,
            mods: KeyModifiers::CONTROL,
        }),
        Input::Key(Key {
            code: KeyCode::PageDown,
            mods: KeyModifiers::NONE,
        }),
    ] {
        let _ = termrock_runtime::stub::deliver(&mut runtime, input);
        let _ = draw(&mut runtime);
    }

    // PageDown moves by the TextArea's logical five-row viewport, not the
    // parent's two physically visible rows.
    assert_eq!(
        TextArea::new(ID, 5).caret_source_row(&runtime.app().state),
        Some(4)
    );
}

#[test]
fn clipped_empty_placeholder_stays_on_its_logical_source_row() {
    let visible = VerticalWindow::new(Rect::new(2, 2, 14, 2), 5, 0).unwrap();
    let mut visible_runtime = runtime(visible, 0, false);
    visible_runtime.app_mut().value.clear();
    let buffer = draw(&mut visible_runtime);
    assert_eq!(row(&buffer, 2, 4, 8), "type her");

    let hidden = VerticalWindow::new(Rect::new(2, 2, 14, 2), 5, 2).unwrap();
    let mut runtime = runtime(hidden, 0, false);
    runtime.app_mut().value.clear();
    let buffer = draw(&mut runtime);
    assert_eq!(row(&buffer, 2, 4, 8), "");
}

#[test]
fn ordinary_horizontal_scroll_keeps_legacy_origin_with_and_without_scrollbar() {
    for value in [
        "abcdefghijklmnopqrstuvwxyz",
        "one\ntwo\nthree\nfour\nfive\nabcdefghijklmnopqrstuvwxyz",
    ] {
        let mut harness = Harness::new(
            LegacyConsumer {
                state: TextAreaState::default(),
                value: value.to_owned(),
            },
            Theme::junie(),
            24,
            12,
        );
        let _ = harness.click(4, 2);
        let _ = harness.handle(Input::Key(Key {
            code: KeyCode::End,
            mods: KeyModifiers::CONTROL,
        }));
        let y = if value.contains('\n') { 6 } else { 2 };
        assert_eq!(
            harness.cell(4, y).symbol(),
            "q",
            "legacy horizontal origin for {value:?}: row={:?}, cursor={:?}",
            (4..15)
                .map(|x| harness.cell(x, y).symbol())
                .collect::<String>(),
            harness.cursor()
        );
        assert_eq!(harness.cursor(), Some(Position::new(14, y)));
    }
}

#[test]
fn captured_drag_outside_window_extends_in_original_unicode_row() {
    let window = VerticalWindow::new(Rect::new(2, 2, 14, 2), 5, 2).unwrap();
    let value = "zero\none\ntwo\nfour\nfive\n三é\nsix\nseven";
    let mut runtime = runtime_with_value(window, 3, true, value);
    let initial = draw(&mut runtime);
    assert_eq!(row(&initial, 2, 4, 8), "三 é");

    for input in [
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: Position::new(4, 2),
            mods: KeyModifiers::NONE,
        }),
        Input::Mouse(Mouse {
            kind: MouseKind::Drag,
            pos: Position::new(21, 2),
            mods: KeyModifiers::NONE,
        }),
        Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: Position::new(21, 2),
            mods: KeyModifiers::NONE,
        }),
        Input::Key(Key {
            code: KeyCode::Char('X'),
            mods: KeyModifiers::NONE,
        }),
        Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }),
    ] {
        let _ = termrock_runtime::stub::deliver(&mut runtime, input);
        let _ = draw(&mut runtime);
    }
    assert_eq!(
        runtime.app().value,
        "zero\none\ntwo\nfour\nfive\nX\nsix\nseven"
    );
}

#[test]
fn captured_vertical_drag_outside_parent_window_reaches_document_edge() {
    let window = VerticalWindow::new(Rect::new(2, 2, 14, 5), 5, 0).unwrap();
    let mut harness = Harness::new(
        Consumer {
            state: TextAreaState::default(),
            value: "a\nb\nc\nd\ne\nf\ng\nh\ni\nj".to_owned(),
            window,
            focus_on_bootstrap: true,
        },
        Theme::junie(),
        24,
        12,
    );
    let _ = harness.click(4, 2);
    let _ = harness.handle(Input::Key(Key {
        code: KeyCode::Home,
        mods: KeyModifiers::CONTROL,
    }));
    let _ = harness.drag((4, 2), (4, 11));
    let _ = harness.key(KeyCode::Char('X'));
    let _ = harness.key(KeyCode::Esc);
    assert_eq!(harness.app().value, "Xj");
}
