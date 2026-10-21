use ratatui_core::style::Color;
use ratatui_core::symbols::{line, scrollbar};

use crate::theme::border;
use crate::theme::glyph::GlyphSet;
use crate::theme::tokens::{
    ColorTokens, Density, DesignTokens, MeterThresholds, MeterTokens, MotionTokens, SizeTokens,
    SpaceTokens, SyntaxTokens,
};

const BLACK: Color = Color::from_u32(0x00_00_00);
const CHROME: Color = Color::from_u32(0x11_11_11);
const CARD: Color = Color::from_u32(0x18_18_1b);
const INPUT: Color = Color::from_u32(0x1e_1e_22);
const INPUT_HOVER: Color = Color::from_u32(0x2f_2f_34);
const OVERLAY: Color = Color::from_u32(0x27_27_2a);
const POPOVER: Color = Color::from_u32(0x3f_3f_46);
const HIGHLIGHT: Color = Color::from_u32(0x2f_5a_a8);
const WHITE: Color = Color::from_u32(0xff_ff_ff);
const WHITE_70: Color = Color::from_u32(0xb3_b3_b3);
const WHITE_50: Color = Color::from_u32(0x80_80_80);
const TEXT_MUTED: Color = Color::from_u32(0x8a_8a_8a);
const TEXT_FAINT: Color = Color::from_u32(0x73_73_73);
const WHITE_30: Color = Color::from_u32(0x4d_4d_4d);
const WHITE_15: Color = Color::from_u32(0x26_26_26);
const GREEN: Color = Color::from_u32(0x48_e0_54);
const GREEN_80: Color = Color::from_u32(0x3a_b3_43);
const GREEN_60: Color = Color::from_u32(0x002b_8632);
const GREEN_20: Color = Color::from_u32(0x0f_2e_13);
const ON_GREEN: Color = Color::from_u32(0x19_19_1c);
const RED: Color = Color::from_u32(0xe4_45_45);
const RED_20: Color = Color::from_u32(0x2e_0f_0f);
const RED_SOFT: Color = Color::from_u32(0xd9_8a_8a);
const RED_45: Color = Color::from_u32(0x7a_2a_2a);
const AMBER: Color = Color::from_u32(0xf5_9e_09);
const AMBER_TINT: Color = Color::from_u32(0x2c_22_10);
const PURPLE: Color = Color::from_u32(0x87_87_ff);
const BLUE_INFO: Color = Color::from_u32(0x79_b8_ff);

pub(crate) const fn tokens() -> ColorTokens {
    ColorTokens {
        surfaces: [BLACK, CHROME, CARD, OVERLAY, POPOVER],
        field: INPUT,
        field_hover: INPUT_HOVER,
        fg: [WHITE, WHITE_70, TEXT_MUTED, TEXT_FAINT, WHITE_15],
        on_accent: ON_GREEN,
        on_danger: WHITE,
        on_surface_inverse: BLACK,
        border_subtle: WHITE_15,
        border_strong: WHITE_30,
        accent: GREEN,
        accent_hover: GREEN_80,
        accent_pressed: GREEN_60,
        accent_tint: GREEN_20,
        focus: GREEN,
        focus_ring: GREEN_60,
        selection_bg: POPOVER,
        selection_fg: WHITE,
        highlight_bg: HIGHLIGHT,
        highlight_fg: WHITE,
        highlight_danger_bg: RED_45,
        highlight_danger_fg: WHITE,
        backdrop_fg: WHITE_15,
        backdrop_bg: BLACK,
        danger: RED,
        danger_soft: RED_SOFT,
        danger_tint: RED_20,
        warning: AMBER,
        warning_tint: AMBER_TINT,
        success: GREEN,
        info: PURPLE,
        disabled_fg: WHITE_30,
        disabled_bg: OVERLAY,
        read_only_fg: WHITE_70,

        syntax: SyntaxTokens {
            keyword: BLUE_INFO,
            ident: WHITE,
            string: WHITE_70,
            number: WHITE_70,
            operator: TEXT_MUTED,
            punct: TEXT_MUTED,
            comment: WHITE_30,
            plain: WHITE,
            type_name: WHITE,
            function: WHITE,
            constant: WHITE_70,
            invalid: RED,
            deprecated: WHITE_50,
            match_bg: GREEN_20,
            match_current_bg: HIGHLIGHT,
            bracket_match: GREEN,
            diagnostic_error: RED,
            diagnostic_warning: AMBER,
            diagnostic_info: PURPLE,
        },
        meter: MeterTokens {
            low: WHITE_70,
            medium: AMBER,
            high: RED,
            track: WHITE_15,
            fill_rest: crate::theme::MeterFillRest::ReferenceLift,
            stale: TEXT_FAINT,
            unknown: TEXT_FAINT,
            series: [GREEN, AMBER, RED, PURPLE, WHITE_70, WHITE_50],
        },
    }
}

pub(crate) const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub(crate) const GLYPHS: GlyphSet = GlyphSet::new(
    [
        "▎", "›", "✓", "[✓]", "[ ]", "(●)", "(○)", "●", "•", "+", "−", "!", "▲", "▸", "▾", "▴",
        "▾", "∇", "▪", "▪", "→", "↓", "‹", "›", "…", "×", "›", "◆", "◇", "", "", "", "", "✓", "‖",
        "+", "[", "]", "•", "▾", "▴",
    ],
    scrollbar::Set {
        track: "┆",
        thumb: "┃",
        begin: "┆",
        end: "┆",
    },
    line::NORMAL,
    line::THICK,
);

pub(crate) const fn design() -> DesignTokens {
    DesignTokens {
        space: SpaceTokens {
            gutter: 1,
            inline: 1,
            gap: 2,
            column_gap: 2,
            form_gap: 4,
            card_inset: 2,
            frame_inset: 3,
            dialog_inset: 3,
            tree_indent: 2,
        },
        size: SizeTokens {
            field_height: 3,
            tabs_height: 2,
            dialog_width: 54,
            dialog_width_wide: 66,
            popup_max_rows: 10,
            popup_min_width: 12,
            popup_max_width: 48,
            min_width: 72,
            min_height: 20,
            scrollbar_width: 1,
            meter_track: 10,
            code_preview_lines: 6,
        },
        glyphs: GLYPHS,
        borders: border::ROUNDED,
        motion: MotionTokens {
            spinner_frames: &SPINNER,
            tick_ms: 80,
            idle_tick_ms: 400,
            press_flash_ms: 140,
            status_ms: 4000,
            wheel_rows: 3,
            double_click_ms: 500,
        },
        meter: MeterThresholds {
            low_max: 59,
            medium_max: 84,
        },
        density: Density::Comfortable,
    }
}

pub(crate) fn recipes() -> crate::theme::Recipes {
    use crate::theme::{Family, FgStep, Modifier, Role, Slot, StylePatch};
    use crate::{Part, StateFlags};

    let mut recipes = super::default_recipes();
    let bar_title = recipes
        .get_mut(Family::MENU)
        .variant_mut(crate::Variant::custom("termrock.menu-bar-title"));
    bar_title.entry(Part::TITLE).when(
        StateFlags::HOVERED,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Custom(Color::from_u32(0x1c_1c_1f))),
    );
    let row = &mut recipes.get_mut(Family::LIST).parts;
    let container = row.entry(Part::CONTAINER);
    container.states.clear();
    let p = StylePatch::new;
    let hover = p().set_bg(Role::HoverSurface);
    let pressed = p()
        .set_fg(Role::Surface(crate::theme::Surface::Canvas))
        .set_bg(Role::Fg(FgStep::Primary))
        .add(Modifier::BOLD);
    let disabled = p()
        .set_fg(Role::DisabledFg)
        .set_bg(Role::CurrentSurface)
        .remove(Modifier::BOLD);

    container
        .when(StateFlags::HOVERED, hover)
        .when(StateFlags::ERROR, p().set_fg(Role::Danger))
        .when(StateFlags::BUSY, p().set_fg(Role::Fg(FgStep::Secondary)))
        .when(StateFlags::FOCUSED, p().add(Modifier::BOLD))
        .when(
            StateFlags::SELECTED | StateFlags::FOCUSED,
            p().set_bg(Role::AccentTint),
        )
        .when(
            StateFlags::SELECTED | StateFlags::FOCUSED | StateFlags::HOVERED,
            hover,
        )
        .when(StateFlags::PRESSED, pressed)
        .when(
            StateFlags::SELECTED | StateFlags::FOCUSED | StateFlags::PRESSED,
            pressed,
        );

    let states = container.states.clone();
    container.when(StateFlags::DISABLED, disabled);
    for rule in states.into_iter().filter(|rule| rule.specificity() > 1) {
        container.when(rule.when | StateFlags::DISABLED, disabled);
    }
    row.entry(Part::GUTTER).when(
        StateFlags::DISABLED,
        StylePatch {
            glyph: Slot::Clear,
            ..p()
        },
    );

    recipes.set_mono_rules(
        Family::LIST,
        vec![(Part::CONTAINER, StateFlags::DISABLED, disabled)],
    );
    recipes
}
