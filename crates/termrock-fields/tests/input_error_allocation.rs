#![allow(
    clippy::unwrap_used,
    reason = "focused public API regression assertions"
)]

use ratatui_core::{
    buffer::{Buffer, Cell},
    layout::{Position, Rect},
    style::{Color, Modifier},
};
use termrock_core::validate::FieldError;
use termrock_fields::{TextInput, TextInputState};
use termrock_runtime::{App, Cx, Id, KeyCode, Response, StateFlags, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, Theme};

const SCREEN: Rect = Rect::new(0, 0, 20, 8);
const INPUT: Id = Id::root("input-error-allocation");
const INITIAL_AREA: Rect = Rect::new(3, 2, 12, 1);

fn validate_address(value: &str) -> Result<(), FieldError> {
    if value.contains('@') {
        Ok(())
    } else {
        Err(FieldError::new("Invalid address"))
    }
}

struct ErrorPage {
    state: TextInputState,
    value: String,
    area: Rect,
    disabled: bool,
}

impl App for ErrorPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            self.state.begin(&self.value);
            cx.focus(INPUT);
        }
        TextInput::new(INPUT)
            .validate(&validate_address)
            .disabled(self.disabled)
            .update(cx, &mut self.state, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(INPUT)
            .validate(&validate_address)
            .disabled(self.disabled)
            .value(&self.value)
            .draw(ui, self.area, &self.state);
    }
}

#[test]
fn disabled_invalid_input_stays_inside_tiny_allocations_at_truecolor_and_mono() {
    let mut sentinel = Cell::new("§");
    sentinel.fg = Color::Rgb(193, 37, 211);
    sentinel.bg = Color::Rgb(17, 83, 149);
    sentinel.underline_color = Color::Rgb(71, 191, 43);
    sentinel.modifier = Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED;

    for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
        for (value, is_invalid) in [
            ("日本語👩‍💻abcdef", true),
            ("日本語👩‍💻abcdef@example.test", false),
        ] {
            let mut page = Harness::new(
                ErrorPage {
                    state: TextInputState::default(),
                    value: value.to_owned(),
                    area: INITIAL_AREA,
                    disabled: false,
                },
                Theme::junie().downgrade(level),
                SCREEN.width,
                SCREEN.height,
            );

            let _ = page.key(KeyCode::Enter);
            assert_eq!(
                page.app().state.error().map(|error| error.message.as_ref()),
                is_invalid.then_some("Invalid address"),
                "validation result for {level:?} input {value:?}"
            );
            assert_eq!(page.app().value, value);
            assert!(!page.app().state.is_editing());

            page.app_mut().disabled = true;
            for width in 0..=12 {
                for height in 0..=3 {
                    let area = Rect::new(3, 2, width, height);
                    page.app_mut().area = area;
                    let _ = page.tick();

                    assert_eq!(
                        page.app().state.error().map(|error| error.message.as_ref()),
                        is_invalid.then_some("Invalid address"),
                        "error state changed for {level:?} {area:?}"
                    );
                    if !area.is_empty() {
                        let flags = page.state_of(INPUT);
                        assert!(
                            flags.contains(StateFlags::DISABLED),
                            "{level:?} {area:?}: {flags:?}"
                        );
                    }
                    assert!(
                        page.cursor().is_none_or(|cursor| area.contains(cursor)),
                        "{level:?} {area:?}: cursor {:?} escapes",
                        page.cursor()
                    );
                    if area.is_empty() {
                        assert!(
                            page.cursor().is_none(),
                            "{level:?} {area:?} published a cursor"
                        );
                    }

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
                                    "{level:?} {area:?}: outside cell ({x}, {y}) changed"
                                );
                            }
                        }
                    }
                    assert!(page.diagnostics().is_empty(), "{:?}", page.diagnostics());
                }
            }
        }
    }
}
