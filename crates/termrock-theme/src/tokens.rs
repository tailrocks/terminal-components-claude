use ratatui_core::style::Color;

use super::border::BorderSet;
use super::glyph::GlyphSet;
use super::role::{FG_STEPS, SURFACE_LEVELS};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SyntaxTokens {
    pub keyword: Color,
    pub ident: Color,
    pub string: Color,
    pub number: Color,
    pub operator: Color,
    pub punct: Color,
    pub comment: Color,
    pub plain: Color,
    pub type_name: Color,
    pub function: Color,
    pub constant: Color,
    pub invalid: Color,
    pub deprecated: Color,
    pub match_bg: Color,
    pub match_current_bg: Color,
    pub bracket_match: Color,
    pub diagnostic_error: Color,
    pub diagnostic_warning: Color,
    pub diagnostic_info: Color,
}

impl SyntaxTokens {
    pub const fn derive(keyword: Color, string: Color, number: Color) -> SyntaxTokens {
        SyntaxTokens {
            keyword,
            ident: Color::Reset,
            string,
            number,
            operator: Color::Reset,
            punct: Color::Reset,
            comment: Color::Reset,
            plain: Color::Reset,
            type_name: keyword,
            function: keyword,
            constant: number,
            invalid: Color::Reset,
            deprecated: Color::Reset,
            match_bg: Color::Reset,
            match_current_bg: Color::Reset,
            bracket_match: keyword,
            diagnostic_error: Color::Reset,
            diagnostic_warning: Color::Reset,
            diagnostic_info: Color::Reset,
        }
    }

    #[must_use]
    pub fn map_colors(&self, f: &mut impl FnMut(Color) -> Color) -> SyntaxTokens {
        let SyntaxTokens {
            keyword,
            ident,
            string,
            number,
            operator,
            punct,
            comment,
            plain,
            type_name,
            function,
            constant,
            invalid,
            deprecated,
            match_bg,
            match_current_bg,
            bracket_match,
            diagnostic_error,
            diagnostic_warning,
            diagnostic_info,
        } = *self;
        SyntaxTokens {
            keyword: f(keyword),
            ident: f(ident),
            string: f(string),
            number: f(number),
            operator: f(operator),
            punct: f(punct),
            comment: f(comment),
            plain: f(plain),
            type_name: f(type_name),
            function: f(function),
            constant: f(constant),
            invalid: f(invalid),
            deprecated: f(deprecated),
            match_bg: f(match_bg),
            match_current_bg: f(match_current_bg),
            bracket_match: f(bracket_match),
            diagnostic_error: f(diagnostic_error),
            diagnostic_warning: f(diagnostic_warning),
            diagnostic_info: f(diagnostic_info),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MeterFillRest {
    Color(Color),
    ReferenceLift,
}

pub(crate) trait TokenMapper {
    fn color(&mut self, color: Color) -> Color;
    fn meter_rest(&mut self, rest: MeterFillRest) -> MeterFillRest;
}
struct ColorMapper<'a, F>(&'a mut F);
impl<F: FnMut(Color) -> Color> TokenMapper for ColorMapper<'_, F> {
    fn color(&mut self, color: Color) -> Color {
        (self.0)(color)
    }
    fn meter_rest(&mut self, rest: MeterFillRest) -> MeterFillRest {
        match rest {
            MeterFillRest::Color(color) => MeterFillRest::Color(self.color(color)),
            MeterFillRest::ReferenceLift => rest,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MeterTokens {
    pub low: Color,
    pub medium: Color,
    pub high: Color,
    pub track: Color,
    pub fill_rest: MeterFillRest,
    pub stale: Color,
    pub unknown: Color,
    pub series: [Color; 6],
}

impl MeterTokens {
    pub const fn derive(low: Color, medium: Color, high: Color) -> MeterTokens {
        MeterTokens {
            low,
            medium,
            high,
            track: Color::Reset,
            fill_rest: MeterFillRest::Color(Color::Reset),
            stale: Color::Reset,
            unknown: Color::Reset,
            series: [low, medium, high, low, medium, high],
        }
    }

    #[must_use]
    pub fn map_colors(&self, f: &mut impl FnMut(Color) -> Color) -> MeterTokens {
        self.map_with(&mut ColorMapper(f))
    }

    pub(crate) fn map_with(&self, mapper: &mut impl TokenMapper) -> MeterTokens {
        let MeterTokens {
            low,
            medium,
            high,
            track,
            fill_rest,
            stale,
            unknown,
            series,
        } = *self;
        MeterTokens {
            low: mapper.color(low),
            medium: mapper.color(medium),
            high: mapper.color(high),
            track: mapper.color(track),
            fill_rest: mapper.meter_rest(fill_rest),
            stale: mapper.color(stale),
            unknown: mapper.color(unknown),
            series: series.map(|color| mapper.color(color)),
        }
    }
}

/// Complete semantic color channels for a theme.
///
/// Downstream struct literals must supply fields added by newer releases.
/// To override selected channels, start from a built-in theme's color tokens
/// and change the fields you need.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ColorTokens {
    pub surfaces: [Color; SURFACE_LEVELS],
    pub field: Color,
    pub field_hover: Color,
    pub fg: [Color; FG_STEPS],
    pub on_accent: Color,
    pub on_danger: Color,
    pub on_surface_inverse: Color,
    pub border_subtle: Color,
    pub border_strong: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub accent_tint: Color,
    pub focus: Color,
    pub focus_ring: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub highlight_danger_bg: Color,
    pub highlight_danger_fg: Color,
    pub backdrop_fg: Color,
    pub backdrop_bg: Color,
    pub danger: Color,
    pub danger_soft: Color,
    pub danger_tint: Color,
    pub warning: Color,
    pub warning_tint: Color,
    pub success: Color,
    pub info: Color,
    pub disabled_fg: Color,
    pub disabled_bg: Color,
    pub read_only_fg: Color,
    /// The background of an application status strip.
    pub status_bar_bg: Color,
    /// The primary ink of an application status strip.
    pub status_bar_fg: Color,
    /// The accent ink of an application status strip.
    pub status_accent: Color,
    /// The rule that separates application chrome from its content.
    pub chrome_rule: Color,
    /// The track of a scroll bar.
    pub scroll_track: Color,
    /// The thumb of a scroll bar.
    pub scroll_thumb: Color,
    /// The ink of a small tag or label chip.
    pub tag_ink: Color,
    pub syntax: SyntaxTokens,
    pub meter: MeterTokens,
}

impl ColorTokens {
    #[must_use]
    pub fn map_colors(&self, f: &mut impl FnMut(Color) -> Color) -> ColorTokens {
        self.map_with(&mut ColorMapper(f))
    }

    pub(crate) fn map_with(&self, mapper: &mut impl TokenMapper) -> ColorTokens {
        let ColorTokens {
            surfaces,
            field,
            field_hover,
            fg,
            on_accent,
            on_danger,
            on_surface_inverse,
            border_subtle,
            border_strong,
            accent,
            accent_hover,
            accent_pressed,
            accent_tint,
            focus,
            focus_ring,
            selection_bg,
            selection_fg,
            highlight_bg,
            highlight_fg,
            highlight_danger_bg,
            highlight_danger_fg,
            backdrop_fg,
            backdrop_bg,
            danger,
            danger_soft,
            danger_tint,
            warning,
            warning_tint,
            success,
            info,
            disabled_fg,
            disabled_bg,
            read_only_fg,
            status_bar_bg,
            status_bar_fg,
            status_accent,
            chrome_rule,
            scroll_track,
            scroll_thumb,
            tag_ink,
            syntax,
            meter,
        } = *self;
        ColorTokens {
            surfaces: surfaces.map(|color| mapper.color(color)),
            field: mapper.color(field),
            field_hover: mapper.color(field_hover),
            fg: fg.map(|color| mapper.color(color)),
            on_accent: mapper.color(on_accent),
            on_danger: mapper.color(on_danger),
            on_surface_inverse: mapper.color(on_surface_inverse),
            border_subtle: mapper.color(border_subtle),
            border_strong: mapper.color(border_strong),
            accent: mapper.color(accent),
            accent_hover: mapper.color(accent_hover),
            accent_pressed: mapper.color(accent_pressed),
            accent_tint: mapper.color(accent_tint),
            focus: mapper.color(focus),
            focus_ring: mapper.color(focus_ring),
            selection_bg: mapper.color(selection_bg),
            selection_fg: mapper.color(selection_fg),
            highlight_bg: mapper.color(highlight_bg),
            highlight_fg: mapper.color(highlight_fg),
            highlight_danger_bg: mapper.color(highlight_danger_bg),
            highlight_danger_fg: mapper.color(highlight_danger_fg),
            backdrop_fg: mapper.color(backdrop_fg),
            backdrop_bg: mapper.color(backdrop_bg),
            danger: mapper.color(danger),
            danger_soft: mapper.color(danger_soft),
            danger_tint: mapper.color(danger_tint),
            warning: mapper.color(warning),
            warning_tint: mapper.color(warning_tint),
            success: mapper.color(success),
            info: mapper.color(info),
            disabled_fg: mapper.color(disabled_fg),
            disabled_bg: mapper.color(disabled_bg),
            read_only_fg: mapper.color(read_only_fg),
            status_bar_bg: mapper.color(status_bar_bg),
            status_bar_fg: mapper.color(status_bar_fg),
            status_accent: mapper.color(status_accent),
            chrome_rule: mapper.color(chrome_rule),
            scroll_track: mapper.color(scroll_track),
            scroll_thumb: mapper.color(scroll_thumb),
            tag_ink: mapper.color(tag_ink),
            syntax: syntax.map_colors(&mut |color| mapper.color(color)),
            meter: meter.map_with(mapper),
        }
    }

    pub fn semantic_colors(&self) -> Vec<MeterFillRest> {
        struct Collect(Vec<MeterFillRest>);
        impl TokenMapper for Collect {
            fn color(&mut self, color: Color) -> Color {
                self.0.push(MeterFillRest::Color(color));
                color
            }
            fn meter_rest(&mut self, rest: MeterFillRest) -> MeterFillRest {
                self.0.push(rest);
                rest
            }
        }
        let mut collect = Collect(Vec::with_capacity(80));
        let _ = self.map_with(&mut collect);
        collect.0
    }

    pub fn colors(&self) -> Vec<Color> {
        let mut out = Vec::with_capacity(79);
        let _ = self.map_colors(&mut |c| {
            out.push(c);
            c
        });
        out
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpaceTokens {
    pub gutter: u16,
    pub inline: u16,
    pub gap: u16,
    pub column_gap: u16,
    pub form_gap: u16,
    pub card_inset: u16,
    pub frame_inset: u16,
    pub dialog_inset: u16,
    pub tree_indent: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SizeTokens {
    pub field_height: u16,
    pub tabs_height: u16,
    pub dialog_width: u16,
    pub dialog_width_wide: u16,
    pub popup_max_rows: u16,
    pub popup_min_width: u16,
    pub popup_max_width: u16,
    pub min_width: u16,
    pub min_height: u16,
    pub scrollbar_width: u16,
    pub meter_track: u16,
    pub code_preview_lines: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MotionTokens {
    pub spinner_frames: &'static [&'static str],
    pub tick_ms: u64,
    pub idle_tick_ms: u64,
    pub press_flash_ms: u64,
    pub status_ms: u64,
    pub wheel_rows: u16,
    pub double_click_ms: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MeterThresholds {
    pub low_max: u8,
    pub medium_max: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DesignTokens {
    pub space: SpaceTokens,
    pub size: SizeTokens,
    pub glyphs: GlyphSet,
    pub borders: BorderSet,
    pub motion: MotionTokens,
    pub meter: MeterThresholds,
    pub density: Density,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[non_exhaustive]
pub enum ColorLevel {
    TrueColor,
    Ansi256,
    Ansi16,
    Mono,
}

impl ColorLevel {
    #[must_use]
    pub fn detect() -> Self {
        use std::io::IsTerminal as _;

        Self::from_env(
            std::env::var_os("CLICOLOR_FORCE").as_deref(),
            std::env::var_os("NO_COLOR").as_deref(),
            std::env::var("TERM").ok().as_deref(),
            std::env::var("COLORTERM").ok().as_deref(),
            std::io::stdout().is_terminal(),
        )
    }

    #[must_use]
    pub fn from_env(
        clicolor_force: Option<&std::ffi::OsStr>,
        no_color: Option<&std::ffi::OsStr>,
        term: Option<&str>,
        colorterm: Option<&str>,
        stdout_is_terminal: bool,
    ) -> Self {
        let term = term.unwrap_or_default();
        if term == "dumb" {
            return ColorLevel::Mono;
        }

        let forced = clicolor_force.is_some_and(|v| !v.is_empty() && v != "0");
        if !forced {
            if no_color.is_some_and(|v| !v.is_empty()) {
                return ColorLevel::Mono;
            }
            if !stdout_is_terminal {
                return ColorLevel::Mono;
            }
        }

        let colorterm = colorterm.unwrap_or_default();
        if colorterm == "truecolor" || colorterm == "24bit" {
            return ColorLevel::TrueColor;
        }
        if term.contains("256color") || term.contains("ghostty") || term.contains("kitty") {
            return ColorLevel::Ansi256;
        }
        ColorLevel::Ansi16
    }

    const fn depth(self) -> u8 {
        match self {
            ColorLevel::Mono => 0,
            ColorLevel::Ansi16 => 1,
            ColorLevel::Ansi256 => 2,
            ColorLevel::TrueColor => 3,
        }
    }

    #[must_use]
    pub const fn narrow_to(self, other: ColorLevel) -> ColorLevel {
        if self.depth() <= other.depth() {
            self
        } else {
            other
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            ColorLevel::TrueColor => "truecolor",
            ColorLevel::Ansi256 => "256 colors",
            ColorLevel::Ansi16 => "16 colors",
            ColorLevel::Mono => "no color",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Capability {
    pub color: ColorLevel,
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::ColorLevel;

    #[test]
    fn term_dumb_is_mono() {
        assert_eq!(
            ColorLevel::from_env(None, None, Some("dumb"), Some("truecolor"), true),
            ColorLevel::Mono
        );
    }

    #[test]
    fn no_color_triggers_on_any_non_empty_value() {
        for v in ["1", "0", "false", " "] {
            assert_eq!(
                ColorLevel::from_env(
                    None,
                    Some(OsStr::new(v)),
                    Some("xterm-256color"),
                    Some("truecolor"),
                    true
                ),
                ColorLevel::Mono,
                "NO_COLOR={v:?} must disable colour"
            );
        }
        for v in [Some(OsStr::new("")), None] {
            assert_eq!(
                ColorLevel::from_env(None, v, Some("xterm-256color"), Some("truecolor"), true),
                ColorLevel::TrueColor,
                "NO_COLOR={v:?} must not disable colour"
            );
        }
    }

    #[test]
    fn a_non_tty_stdout_is_mono() {
        assert_eq!(
            ColorLevel::from_env(None, None, Some("xterm-256color"), Some("truecolor"), false),
            ColorLevel::Mono
        );
    }

    #[test]
    fn clicolor_force_overrides_no_color_and_the_tty_check() {
        assert_eq!(
            ColorLevel::from_env(
                Some(OsStr::new("1")),
                Some(OsStr::new("1")),
                Some("xterm-256color"),
                Some("truecolor"),
                false
            ),
            ColorLevel::TrueColor
        );
        for off in ["", "0"] {
            assert_eq!(
                ColorLevel::from_env(
                    Some(OsStr::new(off)),
                    Some(OsStr::new("1")),
                    Some("xterm-256color"),
                    Some("truecolor"),
                    false
                ),
                ColorLevel::Mono,
                "CLICOLOR_FORCE={off:?} must not force colour"
            );
        }
    }

    #[test]
    fn clicolor_force_does_not_override_term_dumb() {
        assert_eq!(
            ColorLevel::from_env(
                Some(OsStr::new("1")),
                None,
                Some("dumb"),
                Some("truecolor"),
                true
            ),
            ColorLevel::Mono
        );
    }

    #[test]
    fn narrow_to_returns_the_poorer_level() {
        use ColorLevel::{Ansi16, Ansi256, Mono, TrueColor};
        const LADDER: [ColorLevel; 4] = [Mono, Ansi16, Ansi256, TrueColor];
        for (i, a) in LADDER.into_iter().enumerate() {
            for (j, b) in LADDER.into_iter().enumerate() {
                let expected = if i <= j { a } else { b };
                assert_eq!(a.narrow_to(b), expected, "{a:?}.narrow_to({b:?})");
                assert_eq!(a.narrow_to(b), b.narrow_to(a), "{a:?}/{b:?} not symmetric");
            }
            assert_eq!(a.narrow_to(a), a, "{a:?} not idempotent");
            assert_eq!(a.narrow_to(TrueColor), a, "TrueColor is the identity");
            assert_eq!(a.narrow_to(Mono), Mono, "Mono absorbs");
        }
    }

    #[test]
    fn term_unset_or_empty_is_ansi16() {
        for term in [None, Some("")] {
            assert_eq!(
                ColorLevel::from_env(None, None, term, None, true),
                ColorLevel::Ansi16,
                "TERM={term:?}"
            );
        }
    }
}
