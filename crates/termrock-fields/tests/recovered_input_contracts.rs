#![allow(
    clippy::unwrap_used,
    reason = "focused public API regression assertions"
)]

use ratatui_core::{
    buffer::{Buffer, Cell},
    layout::{Position, Rect},
    style::{Color, Modifier},
};
use termrock_fields::{TextInput, TextInputState};
use termrock_runtime::{App, Cx, Id, KeyCode, Part, PartRef, Response, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const SCREEN: Rect = Rect::new(0, 0, 20, 8);
const INPUT: Id = Id::root("recovered.input-contracts");
const WIDE_VALUE: &str = "日本語a";

struct InputPage {
    state: TextInputState,
    value: String,
    area: Rect,
}

impl App for InputPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            self.state.begin(&self.value);
            if !self.area.is_empty() {
                cx.focus(INPUT);
            }
        }
        TextInput::new(INPUT)
            .update(cx, &mut self.state, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(INPUT)
            .value(&self.value)
            .draw(ui, self.area, &self.state);
    }
}

fn harness(area: Rect, value: &str) -> Harness<InputPage> {
    Harness::new(
        InputPage {
            state: TextInputState::default(),
            value: value.to_owned(),
            area,
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    )
}

#[test]
fn tiny_input_full_cell_sentinel_stays_outside_allocated_area() {
    let mut sentinel = Cell::new("§");
    sentinel.fg = Color::Rgb(193, 37, 211);
    sentinel.bg = Color::Rgb(17, 83, 149);
    sentinel.modifier = Modifier::BOLD | Modifier::UNDERLINED;

    for width in 0..13 {
        for height in 0..4 {
            let area = Rect::new(3, 2, width, height);
            let mut page = harness(area, "日本語👩‍💻abcdef");

            let mut buffer = Buffer::filled(SCREEN, sentinel.clone());
            page.runtime_mut()
                .draw_buffer(SCREEN, &mut buffer)
                .commit_presented();
            for y in 0..SCREEN.height {
                for x in 0..SCREEN.width {
                    if !area.contains(Position::new(x, y)) {
                        assert_eq!(
                            buffer[(x, y)],
                            sentinel,
                            "outside {area:?}: cell ({x}, {y}) changed"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn tiny_input_published_cursor_stays_inside_allocated_area() {
    for width in 0..13 {
        for height in 0..4 {
            let area = Rect::new(3, 2, width, height);
            let page = harness(area, "日本語👩‍💻abcdef");
            if area.is_empty() {
                assert!(page.cursor().is_none(), "empty {area:?} published a cursor");
            }
            if let Some(cursor) = page.cursor() {
                assert!(area.contains(cursor), "{area:?}: cursor {cursor:?}");
            }
        }
    }
}

#[test]
fn cursor_outside_screen_clip_is_not_relocated_into_the_field() {
    let area = Rect::new(19, 2, 8, 1);
    let mut page = harness(area, "abcdefghijkl");
    let _ = page.key(KeyCode::End);

    assert!(
        page.cursor().is_none(),
        "clipped field published {:?}",
        page.cursor()
    );
}

#[test]
fn clicking_the_published_caret_after_wide_text_scroll_keeps_the_byte_offset() {
    let area = Rect::new(2, 1, 8, 1);
    let mut page = harness(area, WIDE_VALUE);
    let _ = page.key(KeyCode::End);

    let cursor = page.cursor().expect("focused editor publishes a caret");
    let text_area = page
        .area_of_part(INPUT, PartRef::of(Part::TEXT))
        .expect("text part is registered");
    assert!(text_area.contains(cursor), "{text_area:?}: {cursor:?}");
    let row = page.row(area.y);
    assert!(row.contains('…'), "scroll marker missing: {row:?}");
    assert!(
        !row.contains('日'),
        "leading wide grapheme should be scrolled out: {row:?}"
    );
    assert_eq!(page.app().state.cursor_offset(), Some(WIDE_VALUE.len()));

    let _ = page.click(cursor.x, cursor.y);

    assert_eq!(page.app().state.draft_text(), Some(WIDE_VALUE));
    assert_eq!(page.app().state.cursor_offset(), Some(WIDE_VALUE.len()));
    assert_eq!(page.cursor(), Some(cursor));
    assert!(page.diagnostics().is_empty(), "{:?}", page.diagnostics());
}
