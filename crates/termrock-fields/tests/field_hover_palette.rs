#![allow(
    clippy::unwrap_used,
    reason = "focused consumer palette regression assertions"
)]

use ratatui_core::layout::Rect;
use ratatui_core::style::Color;
use termrock_fields::{TextInput, TextInputState};
use termrock_runtime::{App, Cx, Focusability, Id, MouseKind, Response, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::{
    CapabilityPalettes, ColorLevel, Family, Part, StateFlags, Surface, Theme, Variant,
    downgrade_color,
};

const INPUT: Id = Id::root("field-hover-palette.input");
const BACKGROUND: Id = Id::root("field-hover-palette.background");
const AREA: Rect = Rect::new(0, 0, 12, 1);

struct Page {
    state: TextInputState,
    value: String,
    editing: bool,
    disabled: bool,
    read_only: bool,
    focus_input: bool,
    width: u16,
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(if self.focus_input { INPUT } else { BACKGROUND });
            if self.editing {
                self.state.begin(&self.value);
            }
        }
        TextInput::new(INPUT)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .update(cx, &mut self.state, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(BACKGROUND, Rect::new(15, 1, 1, 1), Focusability::Focusable);
        TextInput::new(INPUT)
            .value(&self.value)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .draw(
                ui,
                Rect::new(AREA.x, AREA.y, self.width, AREA.height),
                &self.state,
            );
    }
}

fn hovered_field_bg(
    level: ColorLevel,
    value: &str,
    editing: bool,
    disabled: bool,
    read_only: bool,
    width: u16,
) -> Color {
    hovered_field_bg_with_theme(
        Theme::junie().for_level(level),
        value,
        editing,
        disabled,
        read_only,
        false,
        width,
    )
}

fn hovered_field_bg_with_theme(
    theme: Theme,
    value: &str,
    editing: bool,
    disabled: bool,
    read_only: bool,
    focus_input: bool,
    width: u16,
) -> Color {
    let mut harness = Harness::new(
        Page {
            state: TextInputState::default(),
            value: value.to_owned(),
            editing,
            disabled,
            read_only,
            focus_input,
            width,
        },
        theme,
        16,
        2,
    );
    let _ = harness.mouse(MouseKind::Move, width.saturating_sub(1).min(4), 0);
    assert_eq!(harness.hover(), Some(INPUT));
    assert!(harness.state_of(INPUT).contains(StateFlags::HOVERED));
    assert_eq!(harness.app().state.is_editing(), editing);
    assert_eq!(harness.focus() == Some(INPUT), focus_input);
    harness.cell(width.saturating_sub(1), 0).bg
}

#[test]
fn hovered_field_surface_uses_frozen_profile_values_in_real_text_input() {
    let cases = [
        (ColorLevel::TrueColor, Color::from_u32(0x00_2f_2f_34)),
        (ColorLevel::Ansi256, Color::Indexed(237)),
        (ColorLevel::Ansi16, Color::DarkGray),
        (ColorLevel::Mono, Color::Black),
    ];
    for (level, expected) in cases {
        assert_eq!(
            hovered_field_bg(level, "value", false, false, false, 12),
            expected,
            "real hovered TextInput at {level:?}"
        );
        assert_eq!(
            Theme::junie()
                .for_level(level)
                .resolve(
                    Family::INPUT,
                    Variant::DEFAULT,
                    Part::FIELD,
                    StateFlags::HOVERED,
                    Surface::Canvas,
                )
                .style
                .as_style()
                .bg,
            Some(expected),
            "semantic FieldHover role at {level:?}"
        );
    }
}

#[test]
fn authored_ansi16_profile_overlays_only_the_qualified_field_hover_token() {
    let source = Theme::junie();
    let generic = Theme::from_tokens(source.color)
        .for_level(ColorLevel::Ansi16)
        .color;
    let authored = source.for_level(ColorLevel::Ansi16).color;

    assert_eq!(authored.field_hover, Color::DarkGray);
    let mut compare = authored;
    compare.field_hover = generic.field_hover;
    compare.fg[3] = generic.fg[3];
    compare.meter.stale = generic.meter.stale;
    compare.meter.unknown = generic.meter.unknown;
    assert_eq!(compare, generic);
}

#[test]
fn editing_and_disabled_hover_keep_the_normal_field_surface() {
    let cases = [
        (
            ColorLevel::TrueColor,
            Color::from_u32(0x00_1e_1e_22),
            Color::from_u32(0x00_2f_2f_34),
        ),
        (
            ColorLevel::Ansi256,
            Color::Indexed(234),
            Color::Indexed(237),
        ),
        (ColorLevel::Ansi16, Color::Black, Color::DarkGray),
        (ColorLevel::Mono, Color::Black, Color::Black),
    ];
    for (level, field, hover) in cases {
        assert_eq!(
            hovered_field_bg(level, "value", true, false, false, 12),
            field,
            "editing+hover retains the field surface at {level:?}"
        );
        assert_eq!(
            hovered_field_bg(level, "value", false, true, false, 12),
            field,
            "disabled controls suppress hover at {level:?}"
        );
        assert_eq!(
            hovered_field_bg(level, "value", false, false, true, 12),
            hover,
            "read-only fields preserve the hover surface at {level:?}"
        );
        assert_eq!(
            hovered_field_bg(level, "", false, false, false, 2),
            hover,
            "empty narrow field uses the same hover surface at {level:?}"
        );
    }
}

#[test]
fn focused_read_only_hover_remains_visible_in_narrow_empty_input() {
    let cases = [
        (ColorLevel::TrueColor, Color::from_u32(0x00_2f_2f_34)),
        (ColorLevel::Ansi256, Color::Indexed(237)),
        (ColorLevel::Ansi16, Color::DarkGray),
        (ColorLevel::Mono, Color::Black),
    ];
    for (level, expected) in cases {
        assert_eq!(
            hovered_field_bg_with_theme(
                Theme::junie().for_level(level),
                "",
                false,
                false,
                true,
                true,
                2,
            ),
            expected,
            "focused read-only narrow empty input at {level:?}"
        );
    }
}

#[test]
fn explicit_custom_token_and_palette_values_override_the_builtin_profile() {
    let custom_hover = Color::from_u32(0x00_8a_43_c1);
    let custom_token = Theme::junie()
        .builder()
        .field(Color::from_u32(0x00_12_34_56), custom_hover)
        .build()
        .for_level(ColorLevel::Ansi256);
    let downgraded_custom = downgrade_color(custom_hover, ColorLevel::Ansi256);
    assert_eq!(custom_token.color.field_hover, downgraded_custom);
    assert_eq!(
        hovered_field_bg_with_theme(custom_token, "value", false, false, false, false, 12),
        downgraded_custom
    );

    let source = Theme::junie();
    let mut ansi256 = source
        .color
        .map_colors(&mut |color| downgrade_color(color, ColorLevel::Ansi256));
    ansi256.field_hover = Color::Indexed(99);
    let source_colors = source.color;
    let explicit_palette = source
        .builder()
        .capability_palettes(
            CapabilityPalettes::new(source_colors).with(ColorLevel::Ansi256, ansi256),
        )
        .build()
        .for_level(ColorLevel::Ansi256);
    assert_eq!(explicit_palette.color.field_hover, Color::Indexed(99));
    assert_eq!(
        hovered_field_bg_with_theme(explicit_palette, "value", false, false, false, false, 12,),
        Color::Indexed(99)
    );
}
