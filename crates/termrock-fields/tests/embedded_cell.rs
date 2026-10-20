use ratatui_core::layout::Rect;
use termrock_fields::{TextInput, TextInputState};
use termrock_runtime::{App, Cx, Id, KeyCode, MouseKind, Response, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const INPUT: Id = Id::root("embedded-cell.input");
const AREA: Rect = Rect::new(4, 1, 8, 1);

struct Page {
    state: TextInputState,
    value: String,
    embedded: bool,
    width: u16,
}

impl Page {
    fn input(&self) -> TextInput<'_> {
        let input = TextInput::new(INPUT);
        if self.embedded {
            input.embedded_cell()
        } else {
            input
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(INPUT);
        }
        let input = if self.embedded {
            TextInput::new(INPUT).embedded_cell()
        } else {
            TextInput::new(INPUT)
        };
        input.update(cx, &mut self.state, &mut self.value).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.input().value(&self.value).draw(
            ui,
            Rect::new(AREA.x, AREA.y, self.width, AREA.height),
            &self.state,
        );
    }
}

fn page(embedded: bool, width: u16, value: &str) -> Harness<Page> {
    Harness::new(
        Page {
            state: TextInputState::default(),
            value: value.to_owned(),
            embedded,
            width,
        },
        Theme::junie(),
        20,
        3,
    )
}

#[test]
fn embedded_text_starts_at_cell_origin_and_keeps_one_trailing_cell() {
    let h = page(true, AREA.width, "ABCD");
    assert_eq!(h.cell(AREA.x, AREA.y).symbol(), "A");
    assert_eq!(h.cell(AREA.x + 3, AREA.y).symbol(), "D");
    assert_eq!(h.cell(AREA.right() - 1, AREA.y).symbol(), " ");
    assert_eq!(
        h.cursor(),
        Some(termrock_runtime::Position::new(AREA.x + 4, AREA.y))
    );

    let normal = page(false, AREA.width, "ABCD");
    assert_eq!(normal.cell(AREA.x, AREA.y).symbol(), "▎");
    assert_eq!(normal.cell(AREA.x + 2, AREA.y).symbol(), "A");
}

#[test]
fn embedded_pointer_coordinates_start_at_the_first_text_cell() {
    let mut h = page(true, AREA.width, "ABCD");
    let _ = h.click(AREA.x + 2, AREA.y);
    assert_eq!(h.app().state.cursor_offset(), Some(2));
    assert_eq!(
        h.cursor(),
        Some(termrock_runtime::Position::new(AREA.x + 2, AREA.y))
    );
}

#[test]
fn embedded_drag_selection_uses_the_same_text_origin_as_rendering() {
    let mut h = page(true, AREA.width, "ABCD");
    let _ = h.mouse(MouseKind::Down, AREA.x, AREA.y);
    let _ = h.mouse(MouseKind::Drag, AREA.x + 3, AREA.y);
    assert_eq!(h.app().state.selection(), Some(0..3));
}

#[test]
fn embedded_wide_text_scrolls_without_an_ellipsis_and_narrow_cells_hide_cursor() {
    let mut h = page(true, AREA.width, "界abcdefgh");
    let _ = h.key(KeyCode::End);
    assert_ne!(h.cell(AREA.x, AREA.y).symbol(), "…");
    assert!(h.cursor().is_some_and(|p| p.x < AREA.right() - 1));

    let narrow = page(true, 1, "界");
    assert_eq!(narrow.cursor(), None);
}
