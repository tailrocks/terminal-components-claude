use super::{ColorLevel, ColorTokens, MeterTokens, SyntaxTokens};
use ratatui_core::style::Color;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CapabilityPalettes {
    pub source: ColorTokens,
    pub ansi256: Option<ColorTokens>,
    pub ansi16: Option<ColorTokens>,
    pub mono: Option<ColorTokens>,
    pub(crate) eligible: Vec<bool>,
    pub(crate) projected: Option<(ColorLevel, ColorTokens)>,
}

impl CapabilityPalettes {
    pub fn new(source: ColorTokens) -> Self {
        let count = source.semantic_colors().len();
        Self {
            source,
            ansi256: None,
            ansi16: None,
            mono: None,
            eligible: vec![true; count],
            projected: None,
        }
    }

    #[must_use]
    pub fn with(mut self, level: ColorLevel, tokens: ColorTokens) -> Self {
        match level {
            ColorLevel::TrueColor => self.source = tokens,
            ColorLevel::Ansi256 => self.ansi256 = Some(tokens),
            ColorLevel::Ansi16 => self.ansi16 = Some(tokens),
            ColorLevel::Mono => self.mono = Some(tokens),
        }
        self
    }

    pub(crate) const fn get(&self, level: ColorLevel) -> Option<ColorTokens> {
        match level {
            ColorLevel::TrueColor => Some(self.source),
            ColorLevel::Ansi256 => self.ansi256,
            ColorLevel::Ansi16 => self.ansi16,
            ColorLevel::Mono => self.mono,
        }
    }
}

pub(crate) fn junie() -> CapabilityPalettes {
    CapabilityPalettes::new(super::builtin::junie::tokens())
        .with(ColorLevel::Ansi256, junie_256())
        .with(ColorLevel::Ansi16, junie_16())
        .with(ColorLevel::Mono, junie_mono())
}

const fn junie_256() -> ColorTokens {
    ColorTokens {
        surfaces: [
            Color::Indexed(16),
            Color::Indexed(232),
            Color::Indexed(233),
            Color::Indexed(235),
            Color::Indexed(237),
        ],
        field: Color::Indexed(234),
        field_hover: Color::Indexed(237),
        fg: [
            Color::Indexed(231),
            Color::Indexed(249),
            Color::Indexed(245),
            Color::Indexed(242),
            Color::Indexed(235),
        ],
        on_accent: Color::Indexed(232),
        on_danger: Color::Indexed(231),
        on_surface_inverse: Color::Indexed(16),
        border_subtle: Color::Indexed(235),
        border_strong: Color::Indexed(242),
        accent: Color::Indexed(78),
        accent_hover: Color::Indexed(77),
        accent_pressed: Color::Indexed(238),
        accent_tint: Color::Indexed(233),
        focus: Color::Indexed(78),
        focus_ring: Color::Indexed(238),
        selection_bg: Color::Indexed(237),
        selection_fg: Color::Indexed(231),
        // Frozen `apply_256` highlight plate (visual-baseline src/theme.rs:
        // `self.highlight = i(25)`).
        highlight_bg: Color::Indexed(25),
        highlight_fg: Color::Indexed(231),
        // Frozen `apply_256` danger highlight plate (`self.highlight_danger =
        // i(88)`).
        highlight_danger_bg: Color::Indexed(88),
        highlight_danger_fg: Color::Indexed(231),
        backdrop_fg: Color::Indexed(235),
        backdrop_bg: Color::Indexed(16),
        danger: Color::Indexed(167),
        danger_soft: Color::Indexed(181),
        danger_tint: Color::Indexed(52),
        warning: Color::Indexed(214),
        warning_tint: Color::Indexed(234),
        success: Color::Indexed(78),
        info: Color::Indexed(147),
        disabled_fg: Color::Indexed(239),
        disabled_bg: Color::Indexed(235),
        read_only_fg: Color::Indexed(249),

        syntax: SyntaxTokens {
            keyword: Color::Indexed(231),
            ident: Color::Indexed(231),
            string: Color::Indexed(249),
            number: Color::Indexed(249),
            operator: Color::Indexed(244),
            punct: Color::Indexed(244),
            comment: Color::Indexed(238),
            plain: Color::Indexed(231),
            type_name: Color::Indexed(231),
            function: Color::Indexed(231),
            constant: Color::Indexed(249),
            invalid: Color::Indexed(167),
            deprecated: Color::Indexed(244),
            match_bg: Color::Indexed(233),
            // Frozen `apply_256` current-match plate
            // (`self.highlight_on_selection = i(55)`).
            match_current_bg: Color::Indexed(55),
            bracket_match: Color::Indexed(78),
            diagnostic_error: Color::Indexed(167),
            diagnostic_warning: Color::Indexed(214),
            diagnostic_info: Color::Indexed(147),
        },
        meter: MeterTokens {
            low: Color::Indexed(249),
            medium: Color::Indexed(214),
            high: Color::Indexed(167),
            track: Color::Indexed(235),
            fill_rest: crate::theme::MeterFillRest::ReferenceLift,
            // Frozen ghost/chrome ink (text_ghost = i(238)).
            stale: Color::Indexed(238),
            unknown: Color::Indexed(238),
            series: [
                Color::Indexed(78),
                Color::Indexed(214),
                Color::Indexed(167),
                Color::Indexed(147),
                Color::Indexed(249),
                Color::Indexed(244),
            ],
        },
    }
}

fn junie_16() -> ColorTokens {
    // Start with the existing generic ANSI16 projection for every token, then
    // author only the source-qualified FieldHover surface. This is a profile
    // derivation, not a frozen-output table for unrelated semantic roles.
    let mut tokens = super::Theme::from_tokens(super::builtin::junie::tokens())
        .downgrade(ColorLevel::Ansi16)
        .color;
    tokens.field_hover = Color::DarkGray;
    // The frozen `apply_16` table renders the ghost/chrome ink as DarkGray
    // (visual-baseline src/theme.rs:1010 `self.text_ghost = DarkGray`), and
    // the stale/unknown meter ink shares that source.
    tokens.meter.stale = Color::DarkGray;
    tokens.meter.unknown = Color::DarkGray;
    tokens
}

const fn junie_mono() -> ColorTokens {
    ColorTokens {
        surfaces: [
            Color::Black,
            Color::Black,
            Color::Black,
            Color::Black,
            Color::DarkGray,
        ],
        field: Color::Black,
        field_hover: Color::Black,
        fg: [
            Color::White,
            Color::Gray,
            Color::Gray,
            Color::Gray,
            Color::Black,
        ],
        on_accent: Color::Black,
        on_danger: Color::White,
        on_surface_inverse: Color::Black,
        border_subtle: Color::Black,
        border_strong: Color::DarkGray,
        accent: Color::Gray,
        accent_hover: Color::DarkGray,
        accent_pressed: Color::DarkGray,
        accent_tint: Color::Black,
        focus: Color::Gray,
        focus_ring: Color::DarkGray,
        selection_bg: Color::DarkGray,
        selection_fg: Color::White,
        highlight_bg: Color::DarkGray,
        highlight_fg: Color::White,
        highlight_danger_bg: Color::DarkGray,
        highlight_danger_fg: Color::White,
        backdrop_fg: Color::Black,
        backdrop_bg: Color::Black,
        danger: Color::Gray,
        danger_soft: Color::Gray,
        danger_tint: Color::Black,
        warning: Color::Gray,
        warning_tint: Color::Black,
        success: Color::Gray,
        info: Color::Gray,
        disabled_fg: Color::DarkGray,
        disabled_bg: Color::Black,
        read_only_fg: Color::Gray,

        syntax: SyntaxTokens {
            keyword: Color::White,
            ident: Color::White,
            string: Color::Gray,
            number: Color::Gray,
            operator: Color::Gray,
            punct: Color::Gray,
            comment: Color::DarkGray,
            plain: Color::White,
            type_name: Color::White,
            function: Color::White,
            constant: Color::Gray,
            invalid: Color::Gray,
            deprecated: Color::Gray,
            match_bg: Color::Black,
            match_current_bg: Color::DarkGray,
            bracket_match: Color::Gray,
            diagnostic_error: Color::Gray,
            diagnostic_warning: Color::Gray,
            diagnostic_info: Color::Gray,
        },
        meter: MeterTokens {
            low: Color::Gray,
            medium: Color::Gray,
            high: Color::Gray,
            track: Color::Black,
            fill_rest: crate::theme::MeterFillRest::ReferenceLift,
            stale: Color::Gray,
            unknown: Color::Gray,
            series: [
                Color::Gray,
                Color::Gray,
                Color::Gray,
                Color::Gray,
                Color::Gray,
                Color::Gray,
            ],
        },
    }
}
