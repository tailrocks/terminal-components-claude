//! Theme tokens (`COMPONENT_ARCHITECTURE.md` §11.2, §11.4).
//!
//! `ColorTokens` is deliberately not `#[non_exhaustive]`: a new token is an
//! intentional breaking change for downstream themes, and
//! [`ColorTokens::map_colors`]'s exhaustive destructure is the mechanism
//! that makes every downgrade cover every token.

use ratatui_core::style::Color;

use super::border::BorderSet;
use super::glyph::GlyphSet;
use super::role::{FG_STEPS, SURFACE_LEVELS};

/// Syntax colours.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SyntaxTokens {
    /// Keyword.
    pub keyword: Color,
    /// Identifier.
    pub ident: Color,
    /// String literal.
    pub string: Color,
    /// Number literal.
    pub number: Color,
    /// Operator.
    pub operator: Color,
    /// Punctuation.
    pub punct: Color,
    /// Comment.
    pub comment: Color,
    /// Plain text.
    pub plain: Color,
    /// Type name.
    pub type_name: Color,
    /// Function name.
    pub function: Color,
    /// Constant.
    pub constant: Color,
    /// Invalid token.
    pub invalid: Color,
    /// Deprecated token.
    pub deprecated: Color,
    /// Find-match background.
    pub match_bg: Color,
    /// Current find-match background.
    pub match_current_bg: Color,
    /// Matching bracket.
    pub bracket_match: Color,
    /// Error diagnostic.
    pub diagnostic_error: Color,
    /// Warning diagnostic.
    pub diagnostic_warning: Color,
    /// Info diagnostic.
    pub diagnostic_info: Color,
}

impl SyntaxTokens {
    /// Derive from three hues. Everything not derivable is `Color::Reset`
    /// ("inherit the part's colour"); `Theme::from_tokens` fills the
    /// diagnostic and match colours from the main tokens.
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

    /// Apply `f` to every colour (exhaustive destructure).
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

/// Meter colours.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MeterTokens {
    /// Healthy.
    pub low: Color,
    /// Needs attention.
    pub medium: Color,
    /// Critical.
    pub high: Color,
    /// The track.
    pub track: Color,
    /// The unfilled remainder.
    pub fill_rest: Color,
    /// Stale data.
    pub stale: Color,
    /// Unknown value.
    pub unknown: Color,
    /// Series colours.
    pub series: [Color; 6],
}

impl MeterTokens {
    /// Derive from three hues; the rest is `Color::Reset` until
    /// `Theme::from_tokens` fills it from the main tokens.
    pub const fn derive(low: Color, medium: Color, high: Color) -> MeterTokens {
        MeterTokens {
            low,
            medium,
            high,
            track: Color::Reset,
            fill_rest: Color::Reset,
            stale: Color::Reset,
            unknown: Color::Reset,
            series: [low, medium, high, low, medium, high],
        }
    }

    /// Apply `f` to every colour (exhaustive destructure).
    #[must_use]
    pub fn map_colors(&self, f: &mut impl FnMut(Color) -> Color) -> MeterTokens {
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
            low: f(low),
            medium: f(medium),
            high: f(high),
            track: f(track),
            fill_rest: f(fill_rest),
            stale: f(stale),
            unknown: f(unknown),
            series: series.map(&mut *f),
        }
    }
}

/// Every colour a theme supplies.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ColorTokens {
    /// The surface ladder: canvas, surface, elevated, overlay, popover.
    pub surfaces: [Color; SURFACE_LEVELS],
    /// Text field plane.
    pub field: Color,
    /// Hovered text field plane.
    pub field_hover: Color,
    /// The foreground ladder: primary, secondary, muted, faint, ghost.
    pub fg: [Color; FG_STEPS],
    /// Text on an accent fill.
    pub on_accent: Color,
    /// Text on a danger fill.
    pub on_danger: Color,
    /// Text on an inverted fill.
    pub on_surface_inverse: Color,
    /// Subtle border.
    pub border_subtle: Color,
    /// Strong border.
    pub border_strong: Color,
    /// Accent.
    pub accent: Color,
    /// Accent, hovered.
    pub accent_hover: Color,
    /// Accent, pressed.
    pub accent_pressed: Color,
    /// Accent tint.
    pub accent_tint: Color,
    /// Focus indicator.
    pub focus: Color,
    /// Focus ring.
    pub focus_ring: Color,
    /// Text selection background.
    pub selection_bg: Color,
    /// Text selection foreground.
    pub selection_fg: Color,
    /// Menu highlight background.
    pub highlight_bg: Color,
    /// Menu highlight foreground.
    pub highlight_fg: Color,
    /// Destructive menu highlight background.
    pub highlight_danger_bg: Color,
    /// Destructive menu highlight foreground.
    pub highlight_danger_fg: Color,
    /// Backdrop foreground.
    pub backdrop_fg: Color,
    /// Backdrop background.
    pub backdrop_bg: Color,
    /// Danger.
    pub danger: Color,
    /// Danger at rest on a neutral plane.
    pub danger_soft: Color,
    /// Danger tint.
    pub danger_tint: Color,
    /// Warning.
    pub warning: Color,
    /// Warning tint.
    pub warning_tint: Color,
    /// Success.
    pub success: Color,
    /// Info.
    pub info: Color,
    /// Disabled foreground.
    pub disabled_fg: Color,
    /// Disabled background.
    pub disabled_bg: Color,
    /// Read-only foreground.
    pub read_only_fg: Color,
    /// Syntax colours.
    pub syntax: SyntaxTokens,
    /// Meter colours.
    pub meter: MeterTokens,
}

impl ColorTokens {
    /// Exhaustive destructure: adding a field is a compile error here.
    #[must_use]
    pub fn map_colors(&self, f: &mut impl FnMut(Color) -> Color) -> ColorTokens {
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
            surfaces: surfaces.map(&mut *f),
            field: f(field),
            field_hover: f(field_hover),
            fg: fg.map(&mut *f),
            on_accent: f(on_accent),
            on_danger: f(on_danger),
            on_surface_inverse: f(on_surface_inverse),
            border_subtle: f(border_subtle),
            border_strong: f(border_strong),
            accent: f(accent),
            accent_hover: f(accent_hover),
            accent_pressed: f(accent_pressed),
            accent_tint: f(accent_tint),
            focus: f(focus),
            focus_ring: f(focus_ring),
            selection_bg: f(selection_bg),
            selection_fg: f(selection_fg),
            highlight_bg: f(highlight_bg),
            highlight_fg: f(highlight_fg),
            highlight_danger_bg: f(highlight_danger_bg),
            highlight_danger_fg: f(highlight_danger_fg),
            backdrop_fg: f(backdrop_fg),
            backdrop_bg: f(backdrop_bg),
            danger: f(danger),
            danger_soft: f(danger_soft),
            danger_tint: f(danger_tint),
            warning: f(warning),
            warning_tint: f(warning_tint),
            success: f(success),
            info: f(info),
            disabled_fg: f(disabled_fg),
            disabled_bg: f(disabled_bg),
            read_only_fg: f(read_only_fg),
            syntax: syntax.map_colors(f),
            meter: meter.map_colors(f),
        }
    }

    /// Every colour, in field order (for tests and pinning).
    pub fn colors(&self) -> Vec<Color> {
        let mut out = Vec::with_capacity(72);
        let _ = self.map_colors(&mut |c| {
            out.push(c);
            c
        });
        out
    }
}

/// Spacing tokens, in cells.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpaceTokens {
    /// Focus gutter width.
    pub gutter: u16,
    /// Inline gap between a glyph and a label.
    pub inline: u16,
    /// Gap between siblings.
    pub gap: u16,
    /// Gap between columns.
    pub column_gap: u16,
    /// Gap between form rows.
    pub form_gap: u16,
    /// Card inset.
    pub card_inset: u16,
    /// Frame inset.
    pub frame_inset: u16,
    /// Dialog inset.
    pub dialog_inset: u16,
    /// Tree indent per depth.
    pub tree_indent: u16,
}

/// Dimension tokens, in cells.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SizeTokens {
    /// Field height (label, control, help).
    pub field_height: u16,
    /// Tab strip height.
    pub tabs_height: u16,
    /// Dialog width.
    pub dialog_width: u16,
    /// Wide dialog width.
    pub dialog_width_wide: u16,
    /// Popup maximum rows.
    pub popup_max_rows: u16,
    /// Popup minimum width.
    pub popup_min_width: u16,
    /// Popup maximum width.
    pub popup_max_width: u16,
    /// Minimum terminal width.
    pub min_width: u16,
    /// Minimum terminal height.
    pub min_height: u16,
    /// Scrollbar width.
    pub scrollbar_width: u16,
    /// Meter track length.
    pub meter_track: u16,
    /// Code preview lines.
    pub code_preview_lines: u16,
}

/// Animation cadence tokens.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MotionTokens {
    /// Spinner frames.
    pub spinner_frames: &'static [&'static str],
    /// Tick period while animating, in milliseconds.
    pub tick_ms: u64,
    /// Tick period while idle, in milliseconds.
    pub idle_tick_ms: u64,
    /// Press flash duration, in milliseconds.
    pub press_flash_ms: u64,
    /// Status message duration, in milliseconds.
    pub status_ms: u64,
    /// Rows per wheel notch.
    pub wheel_rows: u16,
    /// Double-click window, in milliseconds.
    pub double_click_ms: u64,
}

/// Meter thresholds, in percent.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MeterThresholds {
    /// Up to this is healthy.
    pub low_max: u8,
    /// Up to this needs attention; above is critical.
    pub medium_max: u8,
}

/// Row density.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Density {
    /// The default.
    #[default]
    Comfortable,
    /// Tighter rows.
    Compact,
}

/// Every non-colour token.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DesignTokens {
    /// Spacing.
    pub space: SpaceTokens,
    /// Dimensions.
    pub size: SizeTokens,
    /// Glyphs.
    pub glyphs: GlyphSet,
    /// Border set.
    pub borders: BorderSet,
    /// Motion.
    pub motion: MotionTokens,
    /// Meter thresholds.
    pub meter: MeterThresholds,
    /// Density.
    pub density: Density,
}

/// The colour depth a theme is resolved for.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[non_exhaustive]
pub enum ColorLevel {
    /// 24-bit colour.
    TrueColor,
    /// The 256-colour palette.
    Ansi256,
    /// The 16 ANSI colours.
    Ansi16,
    /// No colour.
    Mono,
}

impl ColorLevel {
    /// The level this process's terminal can paint, from the environment.
    ///
    /// A thin wrapper over [`ColorLevel::from_env`], which holds the whole
    /// decision table and is where the behaviour is tested; this reads the
    /// inputs and nothing else.
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

    /// The colour-capability decision table. Pure: no environment, no I/O.
    ///
    /// Split out from [`ColorLevel::detect`] because it is the only testable
    /// shape. Edition 2024 makes `std::env::set_var` `unsafe` and this crate
    /// is `#![forbid(unsafe_code)]`, which no inner `allow` lifts, so no test
    /// here can set an environment variable; the inputs must arrive as
    /// arguments.
    ///
    /// Precedence, highest first:
    ///
    /// 1. `term` is exactly `"dumb"` — [`ColorLevel::Mono`]. The device cannot
    ///    render SGR at all, so this is a fact about the hardware and outranks
    ///    everything below it, `clicolor_force` included.
    /// 2. `clicolor_force` present and neither `""` nor `"0"` — colour is
    ///    forced on, overriding both `no_color` and `stdout_is_terminal`. This
    ///    is the only way to force colour up.
    /// 3. `no_color` present with any non-empty value, per no-color.org —
    ///    [`ColorLevel::Mono`]. It is an [`OsStr`](std::ffi::OsStr) because a
    ///    value that is not UTF-8 is still present; the value is never parsed.
    ///    An empty `no_color` does *not* disable colour.
    /// 4. `stdout_is_terminal` is `false` — [`ColorLevel::Mono`], so redirected
    ///    or piped output carries no escape sequences.
    /// 5. `colorterm` is `truecolor` or `24bit` — [`ColorLevel::TrueColor`].
    /// 6. `term` contains `256color`, `ghostty` or `kitty` —
    ///    [`ColorLevel::Ansi256`].
    /// 7. Otherwise [`ColorLevel::Ansi16`], including when `term` is absent or
    ///    empty: Windows sets no `TERM` and crossterm enables VT processing
    ///    there, so `Mono` in that arm would strip colour from Windows Terminal.
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

    /// How much colour this level can paint, as a rank: `Mono` is 0 and
    /// `TrueColor` is 3.
    ///
    /// Private, and deliberately *not* a `PartialOrd`/`Ord` derive: the enum
    /// is declared richest-first so that reading it top-to-bottom is reading
    /// the downgrade ladder, and a derived order would therefore call
    /// `TrueColor` the *smallest* level. Every comparison would read backwards
    /// at the call site.
    const fn depth(self) -> u8 {
        match self {
            ColorLevel::Mono => 0,
            ColorLevel::Ansi16 => 1,
            ColorLevel::Ansi256 => 2,
            ColorLevel::TrueColor => 3,
        }
    }

    /// The poorer of the two levels (§34.3).
    ///
    /// The operation [`Theme::for_level`](crate::theme::Theme::for_level) is
    /// built from: a theme's own `capability.color` is the depth its tokens
    /// are already at, so combining it with a detected level may only take
    /// colour away. Commutative, associative and idempotent; `TrueColor` is
    /// its identity and `Mono` absorbs.
    #[must_use]
    pub const fn narrow_to(self, other: ColorLevel) -> ColorLevel {
        if self.depth() <= other.depth() {
            self
        } else {
            other
        }
    }

    /// A short label.
    pub const fn label(self) -> &'static str {
        match self {
            ColorLevel::TrueColor => "truecolor",
            ColorLevel::Ansi256 => "256 colors",
            ColorLevel::Ansi16 => "16 colors",
            ColorLevel::Mono => "no color",
        }
    }
}

/// Terminal capability the theme is resolved for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Capability {
    /// Colour depth.
    pub color: ColorLevel,
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::ColorLevel;

    /// `TERM=dumb` outranks every other input: the device cannot render SGR.
    #[test]
    fn term_dumb_is_mono() {
        assert_eq!(
            ColorLevel::from_env(None, None, Some("dumb"), Some("truecolor"), true),
            ColorLevel::Mono
        );
    }

    /// no-color.org: *presence* with any non-empty value disables colour, and
    /// an empty value does not. Pinned so nobody rewrites the check as
    /// `is_some()` (which would fire on `NO_COLOR=`) or as `== "1"`.
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

    /// Redirected or piped output must not carry escape sequences.
    #[test]
    fn a_non_tty_stdout_is_mono() {
        assert_eq!(
            ColorLevel::from_env(None, None, Some("xterm-256color"), Some("truecolor"), false),
            ColorLevel::Mono
        );
    }

    /// `CLICOLOR_FORCE` is the single way to force colour up.
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

    /// Forcing colour cannot invent a capability the device lacks.
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

    /// `narrow_to` is a meet on the colour ladder: it never widens, `Mono`
    /// absorbs and `TrueColor` is the identity. Pinned as a table because the
    /// ranking is hand-written — a derived `Ord` would read backwards against
    /// the enum's richest-first declaration order.
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

    /// Windows sets no `TERM` and crossterm enables VT processing, so an
    /// absent `TERM` must stay coloured rather than fall back to `Mono`.
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
