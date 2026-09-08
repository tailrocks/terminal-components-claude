//! Authored capability output for semantic tokens, separate from RGB conversion.
use super::{ColorLevel, ColorTokens, MeterTokens, SyntaxTokens};
use ratatui_core::style::Color;

/// Semantic token palettes authored for particular terminal capabilities.
///
/// `source` is the `TrueColor` token set these tables describe. Each target
/// field is guarded against the same field's actual prior projected value (or
/// its declared initial capability value before the first projection):
/// direct token mutations and builder-derived replacements therefore retain
/// generic RGB conversion. Equal colors in different roles never identify or
/// borrow each other's palette entries.
/// Conversions through a level with no authored table retain their exact
/// projected values, so later narrowing cannot mistake quantization for a
/// caller's token mutation.
///
/// ```
/// use junie_tui::{CapabilityPalettes, ColorLevel, Theme};
/// use ratatui_core::style::Color;
/// let base = Theme::from_tokens(Theme::junie().color);
/// let mut mono = base.downgrade(ColorLevel::Mono).color;
/// mono.accent = Color::Gray;
/// let theme = base.clone().builder()
///     .capability_palettes(CapabilityPalettes::new(base.color).with(ColorLevel::Mono, mono))
///     .build();
/// assert_eq!(theme.downgrade(ColorLevel::Mono).color.accent, Color::Gray);
/// // Direct construction explicitly chooses generic conversion.
/// let generic = Theme { color: base.color, design: base.design,
///     recipes: base.recipes, capability: base.capability, capability_palettes: None };
/// assert!(generic.capability_palettes.is_none());
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CapabilityPalettes {
    /// `TrueColor` source of the authored token family.
    pub source: ColorTokens,
    /// Authored xterm-256 semantic output.
    pub ansi256: Option<ColorTokens>,
    /// Authored ANSI16 semantic output.
    pub ansi16: Option<ColorTokens>,
    /// Authored monochrome semantic output.
    pub mono: Option<ColorTokens>,
    /// Once detached, a token cannot accidentally reattach after quantization.
    pub(crate) eligible: Vec<bool>,
    /// Exact prior projection, including conversions through an unauthored level.
    pub(crate) projected: Option<(ColorLevel, ColorTokens)>,
}

impl CapabilityPalettes {
    /// A token family with generic conversion at every limited capability.
    pub fn new(source: ColorTokens) -> Self {
        let mut count = 0usize;
        let _ = source.map_colors(&mut |color| {
            count = count.saturating_add(1);
            color
        });
        Self {
            source,
            ansi256: None,
            ansi16: None,
            mono: None,
            eligible: vec![true; count],
            projected: None,
        }
    }

    /// Declare exact semantic output at `level`.
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
        .with(ColorLevel::Mono, junie_mono())
}

// Authored from immutable Holla 794b095 src/theme.rs capability output.
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
        field_hover: Color::Indexed(234),
        fg: [
            Color::Indexed(231),
            Color::Indexed(249),
            Color::Indexed(244),
            Color::Indexed(238),
            Color::Indexed(235),
        ],
        on_accent: Color::Indexed(233),
        on_danger: Color::Indexed(231),
        on_surface_inverse: Color::Indexed(16),
        border_subtle: Color::Indexed(235),
        border_strong: Color::Indexed(238),
        accent: Color::Indexed(78),
        accent_hover: Color::Indexed(77),
        accent_pressed: Color::Indexed(238),
        accent_tint: Color::Indexed(233),
        focus: Color::Indexed(78),
        focus_ring: Color::Indexed(238),
        selection_bg: Color::Indexed(237),
        selection_fg: Color::Indexed(231),
        highlight_bg: Color::Indexed(67),
        highlight_fg: Color::Indexed(231),
        highlight_danger_bg: Color::Indexed(238),
        highlight_danger_fg: Color::Indexed(231),
        backdrop_fg: Color::Indexed(235),
        backdrop_bg: Color::Indexed(16),
        danger: Color::Indexed(167),
        danger_soft: Color::Indexed(181),
        danger_tint: Color::Indexed(233),
        warning: Color::Indexed(214),
        warning_tint: Color::Indexed(234),
        success: Color::Indexed(78),
        info: Color::Indexed(147),
        disabled_fg: Color::Indexed(238),
        disabled_bg: Color::Indexed(235),
        read_only_fg: Color::Indexed(249),
        // restrained syntax palette: structure through weight and the text
        // ladder, not hue
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
            match_current_bg: Color::Indexed(67),
            bracket_match: Color::Indexed(78),
            diagnostic_error: Color::Indexed(167),
            diagnostic_warning: Color::Indexed(214),
            diagnostic_info: Color::Indexed(147),
        },
        meter: MeterTokens {
            low: Color::Indexed(78),
            medium: Color::Indexed(214),
            high: Color::Indexed(167),
            track: Color::Indexed(235),
            fill_rest: Color::Indexed(238),
            stale: Color::Indexed(244),
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
            Color::DarkGray,
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
        // restrained syntax palette: structure through weight and the text
        // ladder, not hue
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
            fill_rest: Color::DarkGray,
            stale: Color::Gray,
            unknown: Color::DarkGray,
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
