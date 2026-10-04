//! The distinct non-Junie theme (`COMPONENT_ARCHITECTURE.md` §11.7): a
//! **light** theme that inverts the plane direction (hover darkens),
//! changes hue family (indigo accent), uses square borders and compact
//! density — the four axes that expose hidden Junie assumptions.
//!
//! Seeds: surfaces `#fbfaf8 #f2f0ec #e8e5df #ded9d0 #cfc8bb`, fg from
//! `#1b1a17` down to `#c6c0b6`, accent `#3b5bdb`, danger `#b02525`,
//! warning `#a86400`, success `#1f7a3d`, borders `#ded9d0` / `#9c948a`.
//! Everything else is derived by `ThemeBuilder::build`.

use ratatui_core::style::Color;

use crate::theme::Theme;
use crate::theme::border;
use crate::theme::recipe::{Family, Variant};
use crate::theme::tokens::{ColorTokens, Density, MeterTokens, SyntaxTokens};

const ACCENT: Color = Color::from_u32(0x3b_5b_db);
const DANGER: Color = Color::from_u32(0xb0_25_25);
const WARNING: Color = Color::from_u32(0xa8_64_00);
const SUCCESS: Color = Color::from_u32(0x1f_7a_3d);
const INFO: Color = Color::from_u32(0x2b_6c_b0);

/// The paper seeds; `Color::Reset` marks a token for derivation.
pub(crate) const fn seeds() -> ColorTokens {
    let r = Color::Reset;
    ColorTokens {
        surfaces: [
            Color::from_u32(0xfb_fa_f8),
            Color::from_u32(0xf2_f0_ec),
            Color::from_u32(0xe8_e5_df),
            Color::from_u32(0xde_d9_d0),
            Color::from_u32(0xcf_c8_bb),
        ],
        field: r,
        field_hover: r,
        fg: [
            Color::from_u32(0x1b_1a_17),
            Color::from_u32(0x4a_46_3f),
            Color::from_u32(0x77_71_6a),
            Color::from_u32(0x9c_94_8a),
            Color::from_u32(0xc6_c0_b6),
        ],
        on_accent: r,
        on_danger: r,
        on_surface_inverse: r,
        border_subtle: Color::from_u32(0xde_d9_d0),
        border_strong: Color::from_u32(0x9c_94_8a),
        accent: ACCENT,
        accent_hover: r,
        accent_pressed: r,
        accent_tint: r,
        focus: r,
        focus_ring: r,
        selection_bg: r,
        selection_fg: r,
        highlight_bg: r,
        highlight_fg: r,
        highlight_danger_bg: r,
        highlight_danger_fg: r,
        backdrop_fg: r,
        backdrop_bg: r,
        danger: DANGER,
        danger_soft: r,
        danger_tint: r,
        warning: WARNING,
        warning_tint: r,
        success: SUCCESS,
        info: INFO,
        disabled_fg: r,
        disabled_bg: r,
        read_only_fg: r,
        syntax: SyntaxTokens::derive(ACCENT, SUCCESS, WARNING),
        meter: MeterTokens::derive(SUCCESS, WARNING, DANGER),
    }
}

/// Build the paper theme.
pub(crate) fn theme() -> Theme {
    let mut t = Theme::from_tokens(seeds())
        .builder()
        .borders_set(border::PLAIN)
        .density(Density::Compact)
        .build();
    t.recipes.get_mut(Family::BUTTON).default_variant = Variant::SECONDARY;
    t
}
