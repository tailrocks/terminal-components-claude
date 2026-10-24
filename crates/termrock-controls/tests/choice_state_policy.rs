use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Style};
use termrock_controls::{Checkbox, CheckboxState};
use termrock_core::id::{Id, Part};
use termrock_runtime::{App, Cx, KeyCode, Response, Ui};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, Theme};

const CHECKBOX: Id = Id::root("controls.checkbox.state-policy");
const AREA: Rect = Rect::new(2, 2, 18, 1);

#[derive(Clone, Copy)]
struct CheckPage {
    value: CheckboxState,
    disabled: bool,
    read_only: bool,
    inherited_disabled: bool,
    custom_marker: bool,
    width: u16,
}

impl Default for CheckPage {
    fn default() -> Self {
        Self {
            value: CheckboxState::Mixed,
            disabled: false,
            read_only: false,
            inherited_disabled: false,
            custom_marker: false,
            width: AREA.width,
        }
    }
}

fn paint_custom_marker(ui: &mut Ui<'_>, area: Rect) {
    ui.paint_str(area, "M", Style::default().fg(Color::Magenta));
}

impl App for CheckPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let checkbox = Checkbox::new(CHECKBOX, "Select all rows")
            .disabled(self.disabled)
            .read_only(self.read_only);
        if self.inherited_disabled {
            checkbox
                .update_state_in_form(cx, &mut self.value, true)
                .erase()
        } else {
            checkbox.update_state(cx, &mut self.value).erase()
        }
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = Rect {
            width: self.width,
            ..AREA
        };
        let checkbox = Checkbox::new(CHECKBOX, "Select all rows")
            .state(self.value)
            .disabled(self.disabled)
            .read_only(self.read_only);
        if self.custom_marker {
            checkbox
                .slot(Part::MARKER, &paint_custom_marker)
                .draw(ui, area);
        } else if self.inherited_disabled {
            checkbox.draw_state_in_form(ui, area, self.value, true);
        } else {
            checkbox.draw(ui, area);
        }
    }
}

fn harness(page: CheckPage, theme: Theme) -> Harness<CheckPage> {
    Harness::new(page, theme, 30, 8)
}

#[test]
fn mixed_checkbox_activates_checked_then_unchecked_through_pointer_and_keys() {
    let mut h = harness(CheckPage::default(), Theme::junie());
    assert_eq!(h.app().value, CheckboxState::Mixed);
    assert!(h.text().contains("[-]"), "{}", h.text());
    assert_eq!(h.area_of(CHECKBOX), Some(AREA));

    let _ = h.click(4, 2);
    assert_eq!(h.app().value, CheckboxState::Checked);

    let _ = h.key(KeyCode::Char(' '));
    assert_eq!(h.app().value, CheckboxState::Unchecked);

    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().value, CheckboxState::Checked);
}

#[test]
fn mixed_checkbox_marker_uses_choice_ink_at_every_color_level() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let theme = Theme::junie().for_level(level);
        let mixed = harness(CheckPage::default(), theme.clone());
        let checked = harness(
            CheckPage {
                value: CheckboxState::Checked,
                ..CheckPage::default()
            },
            theme,
        );

        assert!(mixed.text().contains("[-]"), "{level:?}: {}", mixed.text());
        let mixed_marker = mixed
            .buffer()
            .cell(Position::new(3, 2))
            .expect("mixed marker is inside the screen");
        let checked_marker = checked
            .buffer()
            .cell(Position::new(3, 2))
            .expect("checked marker is inside the screen");
        assert_eq!(
            (mixed_marker.fg, mixed_marker.bg, mixed_marker.modifier),
            (
                checked_marker.fg,
                checked_marker.bg,
                checked_marker.modifier
            ),
            "{level:?}: mixed mark keeps the checked choice ink"
        );

        let buffer = mixed.buffer();
        let blank = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 30, 8));
        for y in 0..8 {
            for x in 0..30 {
                if !AREA.contains(Position::new(x, y)) {
                    assert_eq!(
                        buffer.cell(Position::new(x, y)).expect("screen cell"),
                        blank.cell(Position::new(x, y)).expect("blank screen cell"),
                        "{level:?}: drawing stays inside {AREA:?} at ({x}, {y})"
                    );
                }
            }
        }
    }
}

#[test]
fn mixed_marker_clips_to_narrow_and_empty_hit_areas() {
    for width in 0..=5u16 {
        let h = harness(
            CheckPage {
                width,
                ..CheckPage::default()
            },
            Theme::junie(),
        );
        let area = Rect { width, ..AREA };
        assert_eq!(
            h.area_of(CHECKBOX),
            (!area.is_empty()).then_some(area),
            "width {width}: the hit area matches the visible control area"
        );
        assert!(
            h.diagnostics().is_empty(),
            "width {width}: {:?}",
            h.diagnostics()
        );

        for y in 0..8 {
            for x in 0..30 {
                if !area.contains(Position::new(x, y)) {
                    assert_eq!(
                        h.buffer()
                            .cell(Position::new(x, y))
                            .expect("screen cell")
                            .symbol(),
                        " ",
                        "width {width}: mixed mark stays within {area:?} at ({x}, {y})"
                    );
                }
            }
        }
    }
}

#[test]
fn mixed_marker_part_slot_and_checkbox_hit_area_remain_overridable() {
    let mut h = harness(
        CheckPage {
            custom_marker: true,
            ..CheckPage::default()
        },
        Theme::junie(),
    );
    assert_eq!(h.area_of(CHECKBOX), Some(AREA));
    assert_eq!(
        h.buffer()
            .cell(Position::new(3, 2))
            .expect("marker cell")
            .symbol(),
        "M"
    );
    assert_eq!(
        h.buffer()
            .cell(Position::new(3, 2))
            .expect("marker cell")
            .fg,
        Color::Magenta
    );

    let _ = h.click(0, 0);
    assert_eq!(h.app().value, CheckboxState::Mixed);
    let _ = h.click(3, 2);
    assert_eq!(h.app().value, CheckboxState::Checked);
}

#[test]
fn disabled_read_only_and_inherited_disabled_mixed_values_do_not_change() {
    for page in [
        CheckPage {
            disabled: true,
            ..CheckPage::default()
        },
        CheckPage {
            read_only: true,
            ..CheckPage::default()
        },
        CheckPage {
            inherited_disabled: true,
            ..CheckPage::default()
        },
    ] {
        let mut h = harness(page, Theme::junie());
        let _ = h.click(4, 2);
        let _ = h.key(KeyCode::Char(' '));
        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.app().value, CheckboxState::Mixed);
        if page.disabled || page.inherited_disabled {
            assert_ne!(h.focus(), Some(CHECKBOX));
        } else if page.read_only {
            assert_eq!(h.focus(), Some(CHECKBOX));
        }
    }
}

#[test]
fn bool_checkbox_form_apis_keep_their_binary_behavior() {
    struct BoolPage {
        value: bool,
    }

    impl App for BoolPage {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            Checkbox::new(CHECKBOX, "Boolean")
                .update_in_form(cx, &mut self.value, false)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            Checkbox::new(CHECKBOX, "Boolean").draw_in_form(ui, AREA, self.value, false);
        }
    }

    let mut h = Harness::new(BoolPage { value: false }, Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().value);
    let _ = h.key(KeyCode::Char(' '));
    assert!(!h.app().value);
}
