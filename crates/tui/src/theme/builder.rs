//! Partial override with safe derivation (`COMPONENT_ARCHITECTURE.md` §11.2, §17.0 A5, §21 item 29).
//!
//! A token equal to `Color::Reset` is *unset*; `build` derives every unset
//! token from the seeds by the algorithm in §11.2. Setting a seed through
//! the builder resets the tokens that depend on it (unless they were set
//! explicitly in the same builder), so `Theme::junie().builder().accent(x)`
//! re-derives `accent_hover`, `accent_pressed`, `accent_tint`, `focus`,
//! `focus_ring` and `on_accent` and leaves every other Junie token intact.

use ratatui_core::style::Color;

use super::Theme;
use super::border::BorderSet;
use super::downgrade::{MonoRule, lab_of, luminance, rgb_of};
use super::glyph::{ASCII_RULE_ACTIVE, ASCII_RULE_QUIET, ASCII_SCROLLBAR, GlyphRole};
use super::recipe::Family;
use super::role::{FG_STEPS, SURFACE_LEVELS};
use super::tokens::{ColorTokens, Density, MotionTokens, SizeTokens, SpaceTokens};

/// Which tokens a builder set explicitly.
#[derive(Clone, Copy, Default, Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "one flag per explicitly set token group; a bitset would hide the names"
)]
struct Explicit {
    accent_hover: bool,
    accent_pressed: bool,
    accent_tint: bool,
    focus: bool,
    focus_ring: bool,
    on_accent: bool,
    on_danger: bool,
    danger_soft: bool,
    danger_tint: bool,
    warning_tint: bool,
    selection: bool,
    highlight: bool,
    field: bool,
    disabled: bool,
    borders: bool,
}

/// Builds a theme from a base plus partial overrides.
#[derive(Clone, Debug)]
pub struct ThemeBuilder {
    theme: Theme,
    explicit: Explicit,
}

impl ThemeBuilder {
    pub(crate) fn from_theme(theme: Theme) -> Self {
        ThemeBuilder {
            theme,
            explicit: Explicit::default(),
        }
    }

    /// Set the accent; re-derives its dependants unless set explicitly.
    #[must_use]
    pub fn accent(mut self, c: Color) -> Self {
        let t = &mut self.theme.color;
        t.accent = c;
        let e = self.explicit;
        if !e.accent_hover {
            t.accent_hover = Color::Reset;
        }
        if !e.accent_pressed {
            t.accent_pressed = Color::Reset;
        }
        if !e.accent_tint {
            t.accent_tint = Color::Reset;
        }
        if !e.focus {
            t.focus = Color::Reset;
        }
        if !e.focus_ring {
            t.focus_ring = Color::Reset;
        }
        if !e.on_accent {
            t.on_accent = Color::Reset;
        }
        self
    }

    /// Set the danger hue; re-derives `danger_soft`, `danger_tint`, `on_danger`.
    #[must_use]
    pub fn danger(mut self, c: Color) -> Self {
        let t = &mut self.theme.color;
        t.danger = c;
        if !self.explicit.danger_soft {
            t.danger_soft = Color::Reset;
        }
        if !self.explicit.danger_tint {
            t.danger_tint = Color::Reset;
        }
        if !self.explicit.on_danger {
            t.on_danger = Color::Reset;
        }
        self
    }

    /// Set the warning hue; re-derives `warning_tint`.
    #[must_use]
    pub fn warning(mut self, c: Color) -> Self {
        self.theme.color.warning = c;
        if !self.explicit.warning_tint {
            self.theme.color.warning_tint = Color::Reset;
        }
        self
    }

    /// Set the success hue.
    #[must_use]
    pub fn success(mut self, c: Color) -> Self {
        self.theme.color.success = c;
        self
    }

    /// Set the info hue.
    #[must_use]
    pub fn info(mut self, c: Color) -> Self {
        self.theme.color.info = c;
        self
    }

    /// Set the focus colour (and the focus ring, unless set explicitly).
    #[must_use]
    pub fn focus(mut self, c: Color) -> Self {
        self.theme.color.focus = c;
        self.explicit.focus = true;
        if !self.explicit.focus_ring {
            self.theme.color.focus_ring = Color::Reset;
        }
        self
    }

    /// Set the text selection colours.
    #[must_use]
    pub fn selection(mut self, background: Color, foreground: Color) -> Self {
        self.theme.color.selection_bg = background;
        self.theme.color.selection_fg = foreground;
        self.explicit.selection = true;
        self
    }

    /// Set the menu highlight colours.
    #[must_use]
    pub fn highlight(mut self, background: Color, foreground: Color) -> Self {
        self.theme.color.highlight_bg = background;
        self.theme.color.highlight_fg = foreground;
        self.explicit.highlight = true;
        self
    }

    /// Set the field planes.
    #[must_use]
    pub fn field(mut self, base: Color, hover: Color) -> Self {
        self.theme.color.field = base;
        self.theme.color.field_hover = hover;
        self.explicit.field = true;
        self
    }

    /// Set the disabled colours.
    #[must_use]
    pub fn disabled(mut self, foreground: Color, background: Color) -> Self {
        self.theme.color.disabled_fg = foreground;
        self.theme.color.disabled_bg = background;
        self.explicit.disabled = true;
        self
    }

    /// Set the whole surface ladder.
    #[must_use]
    pub fn surfaces(mut self, s: [Color; SURFACE_LEVELS]) -> Self {
        self.theme.color.surfaces = s;
        if !self.explicit.borders {
            self.theme.color.border_subtle = Color::Reset;
        }
        self
    }

    /// Set the whole foreground ladder.
    #[must_use]
    pub fn fg(mut self, f: [Color; FG_STEPS]) -> Self {
        self.theme.color.fg = f;
        if !self.explicit.borders {
            self.theme.color.border_strong = Color::Reset;
        }
        self
    }

    /// Set the border colours.
    #[must_use]
    pub fn borders(mut self, subtle: Color, strong: Color) -> Self {
        self.theme.color.border_subtle = subtle;
        self.theme.color.border_strong = strong;
        self.explicit.borders = true;
        self
    }

    /// Rebind every glyph whose Junie default falls in the box-drawing block
    /// (`U+2500..=U+257F`) to its ASCII equivalent: the quiet rule (`-`), the
    /// active rule (`=`) and the scrollbar track (`|`), thumb (`#`) and caps
    /// (`|`).
    ///
    /// The whole typed `line` and `scrollbar` sets are replaced, not the four
    /// glyphs a [`GlyphRole`] names, so the seam junctions of `line::Set` and
    /// `scrollbar::Set`'s `begin`/`end` — which no role reaches — are covered
    /// too (Adjudication O2).
    ///
    /// This is the box-drawing block **only**: the remaining ~31 roles (`›`,
    /// `✓`, `…`, `×`, the spinner frames) stay unicode, and a full `GlyphSet`
    /// ASCII table is a separate visual-design decision (§24 M2 risk 3).
    ///
    /// Idempotent, and "last write wins": call [`ThemeBuilder::glyph`]
    /// **after** this to override any of them.
    #[must_use]
    pub fn ascii_glyphs(mut self) -> Self {
        let g = &mut self.theme.design.glyphs;
        g.set_rule_quiet(ASCII_RULE_QUIET);
        g.set_rule_active(ASCII_RULE_ACTIVE);
        g.set_scrollbar(ASCII_SCROLLBAR);
        self
    }

    /// Set the border glyph set.
    ///
    /// Choosing [`border::ASCII`](crate::theme::border::ASCII) also applies
    /// [`ThemeBuilder::ascii_glyphs`]: the rules and the scrollbar come from
    /// typed `line`/`scrollbar` sets rather than from the border set, and an
    /// "ASCII theme" that still paints `─` in a divider is ASCII at the edges
    /// and unicode everywhere else — the outcome §24 M2 called worse than
    /// either consistent choice (`theme::ascii_theme_renders_without_box_drawing_glyphs`).
    ///
    /// The swap is sticky: `borders_set(ASCII).borders_set(PLAIN)` keeps the
    /// ASCII rules, because restoring the theme's own glyphs would clobber a
    /// deliberate [`ThemeBuilder::glyph`]. Override afterwards instead.
    #[must_use]
    pub fn borders_set(mut self, b: BorderSet) -> Self {
        self.theme.design.borders = b;
        if b == crate::theme::border::ASCII {
            self = self.ascii_glyphs();
        }
        self
    }

    /// Replace one glyph.
    #[must_use]
    pub fn glyph(mut self, r: GlyphRole, s: &'static str) -> Self {
        self.theme.design.glyphs.set(r, s);
        self
    }

    /// Set the spacing tokens.
    #[must_use]
    pub fn space(mut self, s: SpaceTokens) -> Self {
        self.theme.design.space = s;
        self
    }

    /// Set the size tokens.
    #[must_use]
    pub fn size(mut self, s: SizeTokens) -> Self {
        self.theme.design.size = s;
        self
    }

    /// Set the density.
    #[must_use]
    pub fn density(mut self, d: Density) -> Self {
        self.theme.design.density = d;
        self
    }

    /// Set the motion tokens.
    #[must_use]
    pub fn motion(mut self, m: MotionTokens) -> Self {
        self.theme.design.motion = m;
        self
    }

    /// Give `f` its own §11.4 mono fallback manifest: the non-colour signals
    /// the resolver adds for that family at [`ColorLevel::Mono`](super::ColorLevel::Mono), on top of
    /// the family-independent generic manifest.
    ///
    /// This is the seam a downstream family needs. `Family::custom("seg")`
    /// resolves through the neutral recipe and therefore receives the generic
    /// rules, but nothing else could express "at mono, a chosen segment gets
    /// the `Chosen` marker on `Part::MARKER`" — the six built-in targeted
    /// tables (`VIEWPORT`, `GRID`, `MENU`, `HELP`, `PICKER`, `SELECT`) are
    /// private and closed.
    ///
    /// **Whole-set, like [`ThemeBuilder::ascii_glyphs`].** `rules` becomes the
    /// family's entire targeted set, replacing the built-in one where there is
    /// one, so:
    ///
    /// * calling it twice for the same family keeps only the last call — the
    ///   set cannot silently accumulate duplicates;
    /// * `mono_rules(f, &[])` **removes** `f`'s built-in targeted rules;
    /// * the generic manifest ([`MONO_RULES_PER_FAMILY`](super::MONO_RULES_PER_FAMILY)
    ///   rules) is unaffected:
    ///   it is family-independent, and a theme that could drop it would be
    ///   able to make `DISABLED` invisible at mono (§29, §20.10 item 18).
    ///
    /// Precedence is unchanged (§11.4): these rules land after the family and
    /// variant state rules and **before** every author override, so
    /// `override_family` still wins over them, and they apply only when the
    /// theme is actually at [`ColorLevel::Mono`](super::ColorLevel::Mono).
    ///
    /// ```
    /// use junie_tui::{
    ///     ColorLevel, Family, GlyphRole, Part, Slot, StateFlags, StylePatch, Surface, Theme,
    ///     Variant,
    /// };
    ///
    /// let seg = Family::custom("seg");
    /// let theme = Theme::junie()
    ///     .builder()
    ///     .mono_rules(
    ///         seg,
    ///         &[(
    ///             Part::MARKER,
    ///             StateFlags::ACTIVE,
    ///             StylePatch::new().set_glyph(GlyphRole::Chosen),
    ///         )],
    ///     )
    ///     .build()
    ///     .downgrade(ColorLevel::Mono);
    ///
    /// let r = theme.resolve(
    ///     seg,
    ///     Variant::DEFAULT,
    ///     Part::MARKER,
    ///     StateFlags::ACTIVE,
    ///     Surface::Canvas,
    /// );
    /// assert_eq!(r.glyph, Slot::Set(GlyphRole::Chosen));
    /// ```
    #[must_use]
    pub fn mono_rules(mut self, f: Family, rules: &[MonoRule]) -> Self {
        self.theme.recipes.set_mono_rules(f, rules.to_vec());
        self
    }

    /// Fill every token the caller did not set by the derivation written
    /// in §11.2. Deterministic.
    pub fn build(mut self) -> Theme {
        derive_unset(&mut self.theme.color);
        self.theme
    }
}

// ───────────────────────────── colour math ────────────────────────────

fn from_lin(v: f64) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let s = if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

fn lab_to_rgb(lab: (f64, f64, f64)) -> (u8, u8, u8) {
    let fy = (lab.0 + 16.0) / 116.0;
    let fx = lab.1 / 500.0 + fy;
    let fz = fy - lab.2 / 200.0;
    let inv = |t: f64| {
        let t3 = t * t * t;
        if t3 > 0.008_856 {
            t3
        } else {
            (t - 16.0 / 116.0) / 7.787
        }
    };
    let xx = inv(fx) * 0.950_47;
    let yy = inv(fy);
    let zz = inv(fz) * 1.088_83;
    let red = xx * 3.240_454_2 + yy * -1.537_138_5 + zz * -0.498_531_4;
    let green = xx * -0.969_266_0 + yy * 1.876_010_8 + zz * 0.041_556_0;
    let blue = xx * 0.055_643_4 + yy * -0.204_025_9 + zz * 1.057_225_2;
    (from_lin(red), from_lin(green), from_lin(blue))
}

/// L* of a colour (0 for `Reset`).
pub(crate) fn lightness(c: Color) -> f64 {
    rgb_of(c).map_or(0.0, |rgb| lab_of(rgb).0)
}

/// Shift L* by `dl`, keeping a* and b*.
fn shift_l(color: Color, dl: f64) -> Color {
    let Some(rgb) = rgb_of(color) else {
        return color;
    };
    let (light, ca, cb) = lab_of(rgb);
    let (red, green, blue) = lab_to_rgb(((light + dl).clamp(0.0, 100.0), ca, cb));
    Color::Rgb(red, green, blue)
}

/// `top` at `alpha` over `bottom`, in sRGB.
fn blend(top: Color, bottom: Color, alpha: f64) -> Color {
    let (Some(t), Some(b)) = (rgb_of(top), rgb_of(bottom)) else {
        return bottom;
    };
    let mix = |x: u8, y: u8| {
        (f64::from(x) * alpha + f64::from(y) * (1.0 - alpha))
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color::Rgb(mix(t.0, b.0), mix(t.1, b.1), mix(t.2, b.2))
}

/// WCAG contrast ratio.
pub(crate) fn contrast(a: Color, b: Color) -> f64 {
    let (Some(a), Some(b)) = (rgb_of(a), rgb_of(b)) else {
        return 1.0;
    };
    let (la, lb) = (luminance(a) + 0.05, luminance(b) + 0.05);
    if la > lb { la / lb } else { lb / la }
}

/// The L* at which a neutral grey reaches `ratio` against `base`, searched
/// away from the base (up for a dark base, down for a light one).
fn anchor_l(base: Color, ratio: f64, light_base: bool) -> f64 {
    let mut lo = 0.0f64;
    let mut hi = 100.0f64;
    for _ in 0..24 {
        let mid = lo.midpoint(hi);
        let (r, g, b) = lab_to_rgb((mid, 0.0, 0.0));
        let c = contrast(Color::Rgb(r, g, b), base);
        let far_enough = c >= ratio;
        if light_base {
            // a light base needs darker text: move `hi` down while contrast holds
            if far_enough {
                lo = mid;
            } else {
                hi = mid;
            }
        } else if far_enough {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let mut l = if light_base { lo } else { hi };
    // rounding to 8-bit channels can land a hair under the ratio: nudge away from the base
    for _ in 0..16 {
        let (r, g, b) = lab_to_rgb((l, 0.0, 0.0));
        if contrast(Color::Rgb(r, g, b), base) >= ratio {
            break;
        }
        l = if light_base { l - 0.5 } else { l + 0.5 };
    }
    l.clamp(0.0, 100.0)
}

fn pick_readable(candidates: [Color; 2], over: Color) -> Color {
    let (a, b) = (candidates[0], candidates[1]);
    if contrast(a, over) >= contrast(b, over) {
        a
    } else {
        b
    }
}

fn is_unset(c: Color) -> bool {
    c == Color::Reset
}

fn fill(slot: &mut Color, value: impl FnOnce() -> Color) {
    if is_unset(*slot) {
        *slot = value();
    }
}

/// Derive every `Reset` token from the seeds (§11.2, exact).
pub(crate) fn derive_unset(c: &mut ColorTokens) {
    let junie = super::builtin::junie::tokens();
    // seeds that must exist
    fill(&mut c.surfaces[0], || junie.surfaces[0]);
    fill(&mut c.accent, || junie.accent);
    fill(&mut c.danger, || junie.danger);
    fill(&mut c.warning, || junie.warning);
    fill(&mut c.success, || junie.success);
    fill(&mut c.info, || junie.info);
    let base = c.surfaces[0];
    let light = lightness(base) > 50.0;
    let step = if light { -4.0 } else { 4.0 };
    // surfaces 1..4 step L* from the canvas
    for i in 1..SURFACE_LEVELS {
        if let Some(s) = c.surfaces.get(i).copied()
            && is_unset(s)
        {
            let prev = c.surfaces.get(i.saturating_sub(1)).copied().unwrap_or(base);
            if let Some(slot) = c.surfaces.get_mut(i) {
                *slot = shift_l(prev, step);
            }
        }
    }
    // fg ladder: contrast-7:1 anchor, then −18 L* per step towards the base
    let anchor = anchor_l(base, 7.0, light);
    let fg_step = if light { 18.0 } else { -18.0 };
    for i in 0..FG_STEPS {
        if let Some(f) = c.fg.get(i).copied()
            && is_unset(f)
        {
            let light = (anchor + fg_step * i as f64).clamp(0.0, 100.0);
            let (red, green, blue) = lab_to_rgb((light, 0.0, 0.0));
            if let Some(slot) = c.fg.get_mut(i) {
                *slot = Color::Rgb(red, green, blue);
            }
        }
    }
    let fg0 = c.fg[0];
    let s = c.surfaces;
    fill(&mut c.field, || s[1]);
    fill(&mut c.field_hover, || s[2]);
    fill(&mut c.accent_hover, || shift_l(c.accent, 8.0));
    fill(&mut c.accent_pressed, || shift_l(c.accent, -8.0));
    fill(&mut c.accent_tint, || blend(c.accent, s[1], 0.12));
    fill(&mut c.focus, || c.accent);
    fill(&mut c.focus_ring, || c.accent_pressed);
    fill(&mut c.border_subtle, || s[3]);
    fill(&mut c.border_strong, || c.fg[3]);
    fill(&mut c.danger_tint, || blend(c.danger, s[1], 0.12));
    fill(&mut c.warning_tint, || blend(c.warning, s[1], 0.12));
    fill(&mut c.danger_soft, || blend(c.danger, c.fg[1], 0.5));
    fill(&mut c.on_accent, || pick_readable([fg0, s[0]], c.accent));
    fill(&mut c.on_danger, || pick_readable([fg0, s[0]], c.danger));
    fill(&mut c.on_surface_inverse, || s[0]);
    fill(&mut c.selection_bg, || s[4]);
    fill(&mut c.selection_fg, || fg0);
    fill(&mut c.highlight_bg, || blend(c.accent, s[2], 0.25));
    fill(&mut c.highlight_fg, || fg0);
    fill(&mut c.highlight_danger_bg, || blend(c.danger, s[2], 0.35));
    fill(&mut c.highlight_danger_fg, || fg0);
    fill(&mut c.backdrop_fg, || c.fg[4]);
    fill(&mut c.backdrop_bg, || s[0]);
    fill(&mut c.disabled_fg, || c.fg[3]);
    fill(&mut c.disabled_bg, || s[2]);
    fill(&mut c.read_only_fg, || c.fg[1]);
    // syntax and meter slots left `Reset` by `derive`
    let sy = &mut c.syntax;
    fill(&mut sy.keyword, || fg0);
    fill(&mut sy.ident, || fg0);
    fill(&mut sy.plain, || fg0);
    fill(&mut sy.string, || c.fg[1]);
    fill(&mut sy.number, || c.fg[1]);
    fill(&mut sy.operator, || c.fg[2]);
    fill(&mut sy.punct, || c.fg[2]);
    fill(&mut sy.comment, || c.fg[3]);
    fill(&mut sy.type_name, || sy.keyword);
    fill(&mut sy.function, || sy.keyword);
    fill(&mut sy.constant, || sy.number);
    fill(&mut sy.bracket_match, || c.accent);
    fill(&mut sy.match_bg, || c.accent_tint);
    fill(&mut sy.match_current_bg, || c.highlight_bg);
    fill(&mut sy.diagnostic_error, || c.danger);
    fill(&mut sy.diagnostic_warning, || c.warning);
    fill(&mut sy.diagnostic_info, || c.info);
    fill(&mut sy.invalid, || c.danger);
    fill(&mut sy.deprecated, || c.fg[2]);
    let me = &mut c.meter;
    fill(&mut me.low, || c.success);
    fill(&mut me.medium, || c.warning);
    fill(&mut me.high, || c.danger);
    fill(&mut me.track, || c.border_subtle);
    fill(&mut me.fill_rest, || c.fg[3]);
    fill(&mut me.stale, || c.fg[2]);
    fill(&mut me.unknown, || c.fg[3]);
    let series_default = [me.low, me.medium, me.high, c.info, c.accent, c.fg[1]];
    for (slot, d) in me.series.iter_mut().zip(series_default) {
        fill(slot, || d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::tokens::{MeterTokens, SyntaxTokens};

    fn seeds() -> ColorTokens {
        let mut c = Theme::junie().color;
        let unset = c.map_colors(&mut |_| Color::Reset);
        c = unset;
        c.surfaces[0] = Color::Rgb(0x0A, 0x0C, 0x10);
        c.accent = Color::Rgb(0x7A, 0xA2, 0xF7);
        c.danger = Color::Rgb(0xF0, 0x6E, 0x78);
        c.warning = Color::Rgb(0xE0, 0xA8, 0x50);
        c.success = Color::Rgb(0x7E, 0xC8, 0x8C);
        c.info = Color::Rgb(0x78, 0xB4, 0xDC);
        c.syntax = SyntaxTokens::derive(c.accent, c.success, c.warning);
        c.meter = MeterTokens::derive(c.success, c.warning, c.danger);
        c
    }

    /// Adjudication O2: `ascii_glyphs` is a whole-set replacement, so applying
    /// it twice changes nothing, and it is "last write wins" against an
    /// explicit `.glyph(..)` that follows it.
    #[test]
    fn ascii_glyphs_is_idempotent_and_glyph_overrides_it() {
        let once = Theme::junie().builder().ascii_glyphs().build();
        let twice = Theme::junie()
            .builder()
            .ascii_glyphs()
            .ascii_glyphs()
            .build();
        assert_eq!(once.design.glyphs, twice.design.glyphs);
        // and it is exactly what `borders_set(ASCII)` applies
        let via_borders = Theme::junie()
            .builder()
            .borders_set(crate::theme::border::ASCII)
            .build();
        assert_eq!(once.design.glyphs, via_borders.design.glyphs);

        let overridden = Theme::junie()
            .builder()
            .borders_set(crate::theme::border::ASCII)
            .glyph(GlyphRole::RuleQuiet, "~")
            .build();
        assert_eq!(overridden.design.glyphs.get(GlyphRole::RuleQuiet), "~");
        // the rest of the ASCII swap survives the override
        assert_eq!(overridden.design.glyphs.get(GlyphRole::RuleActive), "=");
        assert_eq!(overridden.design.glyphs.scrollbar().begin, "|");
    }

    #[test]
    fn builder_derives_every_unset_token_deterministically() {
        let a = Theme::from_tokens(seeds());
        let b = Theme::from_tokens(seeds());
        assert_eq!(a.color, b.color);
        for c in a.color.colors() {
            assert_ne!(c, Color::Reset, "an unset token survived derivation");
        }
        // the ladder steps in the direction of the base
        let l: Vec<f64> = a.color.surfaces.iter().map(|c| lightness(*c)).collect();
        assert!(l.windows(2).all(|w| w[1] > w[0]), "{l:?}");
        let f: Vec<f64> = a.color.fg.iter().map(|c| lightness(*c)).collect();
        assert!(f.windows(2).all(|w| w[1] < w[0]), "{f:?}");
        assert_eq!(a.color.focus, a.color.accent);
        assert_eq!(a.color.focus_ring, a.color.accent_pressed);
        assert_eq!(a.color.border_subtle, a.color.surfaces[3]);
        assert_eq!(a.color.border_strong, a.color.fg[3]);
        // pinned sample of the table
        assert_eq!(a.color.surfaces[1], Color::Rgb(21, 22, 25));
        assert_eq!(a.color.accent_hover, Color::Rgb(145, 183, 255));
        assert_eq!(a.color.accent_pressed, Color::Rgb(99, 141, 224));
        // a partial override on Junie leaves untouched tokens intact
        let t = Theme::junie()
            .builder()
            .accent(Color::Rgb(0xC6, 0x7A, 0x2E))
            .build();
        assert_eq!(t.color.surfaces, Theme::junie().color.surfaces);
        assert_eq!(t.color.fg, Theme::junie().color.fg);
        assert_ne!(t.color.accent_hover, Theme::junie().color.accent_hover);
        assert_eq!(t.color.focus, t.color.accent);
        let t2 = Theme::junie()
            .builder()
            .focus(Color::Rgb(1, 2, 3))
            .accent(Color::Rgb(9, 9, 9))
            .build();
        assert_eq!(
            t2.color.focus,
            Color::Rgb(1, 2, 3),
            "explicit focus survives a later accent"
        );
    }

    #[test]
    fn derived_tokens_meet_design_contrast_ratios() {
        // derived themes meet the ratios the derivation promises; Junie's
        // hand-picked tokens are pinned, not derived, and only its text
        // ladder is checked here
        for t in [Theme::from_tokens(seeds()), Theme::paper()] {
            let c = &t.color;
            assert!(contrast(c.fg[0], c.surfaces[0]) >= 7.0, "{:?}", c.fg[0]);
            assert!(
                contrast(c.on_accent, c.accent) >= 4.5,
                "{:?} on {:?}",
                c.on_accent,
                c.accent
            );
            assert!(
                contrast(c.on_danger, c.danger) >= 4.5,
                "{:?} on {:?}",
                c.on_danger,
                c.danger
            );
        }
        let j = Theme::junie().color;
        assert!(contrast(j.fg[0], j.surfaces[0]) >= 7.0);
    }

    #[test]
    fn lab_round_trips_and_blends() {
        for rgb in [
            (0, 0, 0),
            (255, 255, 255),
            (0x48, 0xe0, 0x54),
            (0x11, 0x11, 0x11),
        ] {
            assert_eq!(lab_to_rgb(lab_of(rgb)), rgb);
        }
        assert_eq!(
            blend(Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), 0.5),
            Color::Rgb(128, 128, 128)
        );
        assert_eq!(
            blend(Color::Reset, Color::Rgb(1, 1, 1), 0.5),
            Color::Rgb(1, 1, 1)
        );
        assert!((contrast(Color::White, Color::Black) - 21.0).abs() < 0.01);
    }
}
