#![doc = r" Theme tokens (`COMPONENT_ARCHITECTURE.md` §11.2, §11.4)."]
#![doc = r""]
#![doc = r" `ColorTokens` is deliberately not `#[non_exhaustive]`: a new token is an"]
#![doc = r" intentional breaking change for downstream themes, and"]
#![doc = r" [`ColorTokens::map_colors`]'s exhaustive destructure is the mechanism"]
#![doc = r" that makes every downgrade cover every token."]

use ratatui_core::style::Color;

use super::border::BorderSet;
use super::glyph::GlyphSet;
use super::role::{FG_STEPS, SURFACE_LEVELS};

#[doc = r" Syntax colours."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SyntaxTokens {
    #[doc = r" Keyword."]
    pub keyword: Color,
    #[doc = r" Identifier."]
    pub ident: Color,
    #[doc = r" String literal."]
    pub string: Color,
    #[doc = r" Number literal."]
    pub number: Color,
    #[doc = r" Operator."]
    pub operator: Color,
    #[doc = r" Punctuation."]
    pub punct: Color,
    #[doc = r" Comment."]
    pub comment: Color,
    #[doc = r" Plain text."]
    pub plain: Color,
    #[doc = r" Type name."]
    pub type_name: Color,
    #[doc = r" Function name."]
    pub function: Color,
    #[doc = r" Constant."]
    pub constant: Color,
    #[doc = r" Invalid token."]
    pub invalid: Color,
    #[doc = r" Deprecated token."]
    pub deprecated: Color,
    #[doc = r" Find-match background."]
    pub match_bg: Color,
    #[doc = r" Current find-match background."]
    pub match_current_bg: Color,
    #[doc = r" Matching bracket."]
    pub bracket_match: Color,
    #[doc = r" Error diagnostic."]
    pub diagnostic_error: Color,
    #[doc = r" Warning diagnostic."]
    pub diagnostic_warning: Color,
    #[doc = r" Info diagnostic."]
    pub diagnostic_info: Color,
}

impl SyntaxTokens {
    #[doc = r" Derive from three hues. Everything not derivable is `Color::Reset`"]
    #[doc = r#" ("inherit the part's colour"); `Theme::from_tokens` fills the"#]
    #[doc = r" diagnostic and match colours from the main tokens."]
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

    #[doc = r" Apply `f` to every colour (exhaustive destructure)."]
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

#[doc = r" Background policy for the unfilled share of a block meter."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MeterFillRest {
    #[doc = r" An explicit theme color, narrowed with the other color tokens."]
    Color(Color),
    #[doc = r" Apply pinned ordered color comparisons to the inherited surface."]
    #[doc = r" Quantized aliases follow the same branch order as the reference theme."]
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

#[doc = r" Meter colours."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MeterTokens {
    #[doc = r" Healthy."]
    pub low: Color,
    #[doc = r" Needs attention."]
    pub medium: Color,
    #[doc = r" Critical."]
    pub high: Color,
    #[doc = r" The track."]
    pub track: Color,
    #[doc = r" The unfilled remainder."]
    pub fill_rest: MeterFillRest,
    #[doc = r" Stale data."]
    pub stale: Color,
    #[doc = r" Unknown value."]
    pub unknown: Color,
    #[doc = r" Series colours."]
    pub series: [Color; 6],
}

impl MeterTokens {
    #[doc = r" Derive from three hues; the rest is `Color::Reset` until"]
    #[doc = r" `Theme::from_tokens` fills it from the main tokens."]
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

    #[doc = r" Apply `f` to every colour (exhaustive destructure)."]
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

#[doc = r" Every colour a theme supplies."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ColorTokens {
    #[doc = r" The surface ladder: canvas, surface, elevated, overlay, popover."]
    pub surfaces: [Color; SURFACE_LEVELS],
    #[doc = r" Text field plane."]
    pub field: Color,
    #[doc = r" Hovered text field plane."]
    pub field_hover: Color,
    #[doc = r" The foreground ladder: primary, secondary, muted, faint, ghost."]
    pub fg: [Color; FG_STEPS],
    #[doc = r" Text on an accent fill."]
    pub on_accent: Color,
    #[doc = r" Text on a danger fill."]
    pub on_danger: Color,
    #[doc = r" Text on an inverted fill."]
    pub on_surface_inverse: Color,
    #[doc = r" Subtle border."]
    pub border_subtle: Color,
    #[doc = r" Strong border."]
    pub border_strong: Color,
    #[doc = r" Accent."]
    pub accent: Color,
    #[doc = r" Accent, hovered."]
    pub accent_hover: Color,
    #[doc = r" Accent, pressed."]
    pub accent_pressed: Color,
    #[doc = r" Accent tint."]
    pub accent_tint: Color,
    #[doc = r" Focus indicator."]
    pub focus: Color,
    #[doc = r" Focus ring."]
    pub focus_ring: Color,
    #[doc = r" Text selection background."]
    pub selection_bg: Color,
    #[doc = r" Text selection foreground."]
    pub selection_fg: Color,
    #[doc = r" Menu highlight background."]
    pub highlight_bg: Color,
    #[doc = r" Menu highlight foreground."]
    pub highlight_fg: Color,
    #[doc = r" Destructive menu highlight background."]
    pub highlight_danger_bg: Color,
    #[doc = r" Destructive menu highlight foreground."]
    pub highlight_danger_fg: Color,
    #[doc = r" Backdrop foreground."]
    pub backdrop_fg: Color,
    #[doc = r" Backdrop background."]
    pub backdrop_bg: Color,
    #[doc = r" Danger."]
    pub danger: Color,
    #[doc = r" Danger at rest on a neutral plane."]
    pub danger_soft: Color,
    #[doc = r" Danger tint."]
    pub danger_tint: Color,
    #[doc = r" Warning."]
    pub warning: Color,
    #[doc = r" Warning tint."]
    pub warning_tint: Color,
    #[doc = r" Success."]
    pub success: Color,
    #[doc = r" Info."]
    pub info: Color,
    #[doc = r" Disabled foreground."]
    pub disabled_fg: Color,
    #[doc = r" Disabled background."]
    pub disabled_bg: Color,
    #[doc = r" Read-only foreground."]
    pub read_only_fg: Color,
    #[doc = r" Syntax colours."]
    pub syntax: SyntaxTokens,
    #[doc = r" Meter colours."]
    pub meter: MeterTokens,
}

impl ColorTokens {
    #[doc = r" Exhaustive destructure: adding a field is a compile error here."]
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
            syntax: syntax.map_colors(&mut |color| mapper.color(color)),
            meter: meter.map_with(mapper),
        }
    }

    #[doc = r" Every semantic slot in field order, including the typed fill-rest policy."]
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
        let mut collect = Collect(Vec::with_capacity(73));
        let _ = self.map_with(&mut collect);
        collect.0
    }

    #[doc = r" Every concrete colour, in field order. Symbolic policies are omitted."]
    #[doc = r" Use `semantic_colors` when stable semantic-slot positions matter."]
    pub fn colors(&self) -> Vec<Color> {
        let mut out = Vec::with_capacity(72);
        let _ = self.map_colors(&mut |c| {
            out.push(c);
            c
        });
        out
    }
}

#[doc = r" Spacing tokens, in cells."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpaceTokens {
    #[doc = r" Focus gutter width."]
    pub gutter: u16,
    #[doc = r" Inline gap between a glyph and a label."]
    pub inline: u16,
    #[doc = r" Gap between siblings."]
    pub gap: u16,
    #[doc = r" Gap between columns."]
    pub column_gap: u16,
    #[doc = r" Gap between form rows."]
    pub form_gap: u16,
    #[doc = r" Card inset."]
    pub card_inset: u16,
    #[doc = r" Frame inset."]
    pub frame_inset: u16,
    #[doc = r" Dialog inset."]
    pub dialog_inset: u16,
    #[doc = r" Tree indent per depth."]
    pub tree_indent: u16,
}

#[doc = r" Dimension tokens, in cells."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SizeTokens {
    #[doc = r" Field height (label, control, help)."]
    pub field_height: u16,
    #[doc = r" Tab strip height."]
    pub tabs_height: u16,
    #[doc = r" Dialog width."]
    pub dialog_width: u16,
    #[doc = r" Wide dialog width."]
    pub dialog_width_wide: u16,
    #[doc = r" Popup maximum rows."]
    pub popup_max_rows: u16,
    #[doc = r" Popup minimum width."]
    pub popup_min_width: u16,
    #[doc = r" Popup maximum width."]
    pub popup_max_width: u16,
    #[doc = r" Minimum terminal width."]
    pub min_width: u16,
    #[doc = r" Minimum terminal height."]
    pub min_height: u16,
    #[doc = r" Scrollbar width."]
    pub scrollbar_width: u16,
    #[doc = r" Meter track length."]
    pub meter_track: u16,
    #[doc = r" Code preview lines."]
    pub code_preview_lines: u16,
}

#[doc = r" Animation cadence tokens."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MotionTokens {
    #[doc = r" Spinner frames."]
    pub spinner_frames: &'static [&'static str],
    #[doc = r" Tick period while animating, in milliseconds."]
    pub tick_ms: u64,
    #[doc = r" Tick period while idle, in milliseconds."]
    pub idle_tick_ms: u64,
    #[doc = r" Press flash duration, in milliseconds."]
    pub press_flash_ms: u64,
    #[doc = r" Status message duration, in milliseconds."]
    pub status_ms: u64,
    #[doc = r" Rows per wheel notch."]
    pub wheel_rows: u16,
    #[doc = r" Double-click window, in milliseconds."]
    pub double_click_ms: u64,
}

#[doc = r" Meter thresholds, in percent."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MeterThresholds {
    #[doc = r" Up to this is healthy."]
    pub low_max: u8,
    #[doc = r" Up to this needs attention; above is critical."]
    pub medium_max: u8,
}

#[doc = r" Row density."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Density {
    #[doc = r" The default."]
    #[default]
    Comfortable,
    #[doc = r" Tighter rows."]
    Compact,
}

#[doc = r" Every non-colour token."]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DesignTokens {
    #[doc = r" Spacing."]
    pub space: SpaceTokens,
    #[doc = r" Dimensions."]
    pub size: SizeTokens,
    #[doc = r" Glyphs."]
    pub glyphs: GlyphSet,
    #[doc = r" Border set."]
    pub borders: BorderSet,
    #[doc = r" Motion."]
    pub motion: MotionTokens,
    #[doc = r" Meter thresholds."]
    pub meter: MeterThresholds,
    #[doc = r" Density."]
    pub density: Density,
}

#[doc = r" The colour depth a theme is resolved for."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[non_exhaustive]
pub enum ColorLevel {
    #[doc = r" 24-bit colour."]
    TrueColor,
    #[doc = r" The 256-colour palette."]
    Ansi256,
    #[doc = r" The 16 ANSI colours."]
    Ansi16,
    #[doc = r" No colour."]
    Mono,
}

impl ColorLevel {
    #[doc = r" The level this process's terminal can paint, from the environment."]
    #[doc = r""]
    #[doc = r" A thin wrapper over [`ColorLevel::from_env`], which holds the whole"]
    #[doc = r" decision table and is where the behaviour is tested; this reads the"]
    #[doc = r" inputs and nothing else."]
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

    #[doc = r" The colour-capability decision table. Pure: no environment, no I/O."]
    #[doc = r""]
    #[doc = r" Split out from [`ColorLevel::detect`] because it is the only testable"]
    #[doc = r" shape. Edition 2024 makes `std::env::set_var` `unsafe` and this crate"]
    #[doc = r" is `#![forbid(unsafe_code)]`, which no inner `allow` lifts, so no test"]
    #[doc = r" here can set an environment variable; the inputs must arrive as"]
    #[doc = r" arguments."]
    #[doc = r""]
    #[doc = r" Precedence, highest first:"]
    #[doc = r""]
    #[doc = r#" 1. `term` is exactly `"dumb"` — [`ColorLevel::Mono`]. The device cannot"#]
    #[doc = r"    render SGR at all, so this is a fact about the hardware and outranks"]
    #[doc = r"    everything below it, `clicolor_force` included."]
    #[doc = r#" 2. `clicolor_force` present and neither `""` nor `"0"` — colour is"#]
    #[doc = r"    forced on, overriding both `no_color` and `stdout_is_terminal`. This"]
    #[doc = r"    is the only way to force colour up."]
    #[doc = r" 3. `no_color` present with any non-empty value, per no-color.org —"]
    #[doc = r"    [`ColorLevel::Mono`]. It is an [`OsStr`](std::ffi::OsStr) because a"]
    #[doc = r"    value that is not UTF-8 is still present; the value is never parsed."]
    #[doc = r"    An empty `no_color` does *not* disable colour."]
    #[doc = r" 4. `stdout_is_terminal` is `false` — [`ColorLevel::Mono`], so redirected"]
    #[doc = r"    or piped output carries no escape sequences."]
    #[doc = r" 5. `colorterm` is `truecolor` or `24bit` — [`ColorLevel::TrueColor`]."]
    #[doc = r" 6. `term` contains `256color`, `ghostty` or `kitty` —"]
    #[doc = r"    [`ColorLevel::Ansi256`]."]
    #[doc = r" 7. Otherwise [`ColorLevel::Ansi16`], including when `term` is absent or"]
    #[doc = r"    empty: Windows sets no `TERM` and crossterm enables VT processing"]
    #[doc = r"    there, so `Mono` in that arm would strip colour from Windows Terminal."]
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

    #[doc = r" How much colour this level can paint, as a rank: `Mono` is 0 and"]
    #[doc = r" `TrueColor` is 3."]
    #[doc = r""]
    #[doc = r" Private, and deliberately *not* a `PartialOrd`/`Ord` derive: the enum"]
    #[doc = r" is declared richest-first so that reading it top-to-bottom is reading"]
    #[doc = r" the downgrade ladder, and a derived order would therefore call"]
    #[doc = r" `TrueColor` the *smallest* level. Every comparison would read backwards"]
    #[doc = r" at the call site."]
    const fn depth(self) -> u8 {
        match self {
            ColorLevel::Mono => 0,
            ColorLevel::Ansi16 => 1,
            ColorLevel::Ansi256 => 2,
            ColorLevel::TrueColor => 3,
        }
    }

    #[doc = r" The poorer of the two levels (§34.3)."]
    #[doc = r""]
    #[doc = r" The operation [`Theme::for_level`](crate::theme::Theme::for_level) is"]
    #[doc = r" built from: a theme's own `capability.color` is the depth its tokens"]
    #[doc = r" are already at, so combining it with a detected level may only take"]
    #[doc = r" colour away. Commutative, associative and idempotent; `TrueColor` is"]
    #[doc = r" its identity and `Mono` absorbs."]
    #[must_use]
    pub const fn narrow_to(self, other: ColorLevel) -> ColorLevel {
        if self.depth() <= other.depth() {
            self
        } else {
            other
        }
    }

    #[doc = r" A short label."]
    pub const fn label(self) -> &'static str {
        match self {
            ColorLevel::TrueColor => "truecolor",
            ColorLevel::Ansi256 => "256 colors",
            ColorLevel::Ansi16 => "16 colors",
            ColorLevel::Mono => "no color",
        }
    }
}

#[doc = r" Terminal capability the theme is resolved for."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Capability {
    #[doc = r" Colour depth."]
    pub color: ColorLevel,
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::ColorLevel;

    #[doc = r" `TERM=dumb` outranks every other input: the device cannot render SGR."]
    #[test]
    fn term_dumb_is_mono() {
        assert_eq!(
            ColorLevel::from_env(None, None, Some("dumb"), Some("truecolor"), true),
            ColorLevel::Mono
        );
    }

    #[doc = r" no-color.org: *presence* with any non-empty value disables colour, and"]
    #[doc = r" an empty value does not. Pinned so nobody rewrites the check as"]
    #[doc = r#" `is_some()` (which would fire on `NO_COLOR=`) or as `== "1"`."#]
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

    #[doc = r" Redirected or piped output must not carry escape sequences."]
    #[test]
    fn a_non_tty_stdout_is_mono() {
        assert_eq!(
            ColorLevel::from_env(None, None, Some("xterm-256color"), Some("truecolor"), false),
            ColorLevel::Mono
        );
    }

    #[doc = r" `CLICOLOR_FORCE` is the single way to force colour up."]
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

    #[doc = r" Forcing colour cannot invent a capability the device lacks."]
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

    #[doc = r" `narrow_to` is a meet on the colour ladder: it never widens, `Mono`"]
    #[doc = r" absorbs and `TrueColor` is the identity. Pinned as a table because the"]
    #[doc = r" ranking is hand-written — a derived `Ord` would read backwards against"]
    #[doc = r" the enum's richest-first declaration order."]
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

    #[doc = r" Windows sets no `TERM` and crossterm enables VT processing, so an"]
    #[doc = r" absent `TERM` must stay coloured rather than fall back to `Mono`."]
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
