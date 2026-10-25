use ratatui_core::style::Color;

use super::Theme;
use super::border::BorderSet;
use super::downgrade::{MonoRule, lab_of, luminance, rgb_of};
use super::glyph::{ASCII_RULE_ACTIVE, ASCII_RULE_QUIET, ASCII_SCROLLBAR, GlyphRole};
use super::recipe::{Family, ManagerPresentationProfile};
use super::role::{FG_STEPS, SURFACE_LEVELS};
use super::tokens::{ColorTokens, Density, MotionTokens, SizeTokens, SpaceTokens};

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

    #[must_use]
    pub fn manager_presentation_profile(mut self, profile: ManagerPresentationProfile) -> Self {
        self.theme.recipes.set_manager_presentation_profile(profile);
        self
    }

    #[must_use]
    pub fn capability_palettes(mut self, mut palettes: super::CapabilityPalettes) -> Self {
        palettes.eligible.fill(true);
        palettes.projected = None;
        self.theme.capability_palettes = Some(std::sync::Arc::new(palettes));
        self
    }

    #[must_use]
    pub fn clear_capability_palettes(mut self) -> Self {
        self.theme.capability_palettes = None;
        self
    }

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

    #[must_use]
    pub fn warning(mut self, c: Color) -> Self {
        self.theme.color.warning = c;
        if !self.explicit.warning_tint {
            self.theme.color.warning_tint = Color::Reset;
        }
        self
    }

    #[must_use]
    pub fn success(mut self, c: Color) -> Self {
        self.theme.color.success = c;
        self
    }

    #[must_use]
    pub fn info(mut self, c: Color) -> Self {
        self.theme.color.info = c;
        self
    }

    #[must_use]
    pub fn focus(mut self, c: Color) -> Self {
        self.theme.color.focus = c;
        self.explicit.focus = true;
        if !self.explicit.focus_ring {
            self.theme.color.focus_ring = Color::Reset;
        }
        self
    }

    #[must_use]
    pub fn selection(mut self, background: Color, foreground: Color) -> Self {
        self.theme.color.selection_bg = background;
        self.theme.color.selection_fg = foreground;
        self.explicit.selection = true;
        self
    }

    #[must_use]
    pub fn highlight(mut self, background: Color, foreground: Color) -> Self {
        self.theme.color.highlight_bg = background;
        self.theme.color.highlight_fg = foreground;
        self.explicit.highlight = true;
        self
    }

    #[must_use]
    pub fn field(mut self, base: Color, hover: Color) -> Self {
        self.theme.color.field = base;
        self.theme.color.field_hover = hover;
        self.explicit.field = true;
        self
    }

    #[must_use]
    pub fn disabled(mut self, foreground: Color, background: Color) -> Self {
        self.theme.color.disabled_fg = foreground;
        self.theme.color.disabled_bg = background;
        self.explicit.disabled = true;
        self
    }

    #[must_use]
    pub fn surfaces(mut self, s: [Color; SURFACE_LEVELS]) -> Self {
        self.theme.color.surfaces = s;
        if !self.explicit.borders {
            self.theme.color.border_subtle = Color::Reset;
        }
        self
    }

    #[must_use]
    pub fn fg(mut self, f: [Color; FG_STEPS]) -> Self {
        self.theme.color.fg = f;
        if !self.explicit.borders {
            self.theme.color.border_strong = Color::Reset;
        }
        self
    }

    #[must_use]
    pub fn borders(mut self, subtle: Color, strong: Color) -> Self {
        self.theme.color.border_subtle = subtle;
        self.theme.color.border_strong = strong;
        self.explicit.borders = true;
        self
    }

    #[must_use]
    pub fn ascii_glyphs(mut self) -> Self {
        let g = &mut self.theme.design.glyphs;
        g.set_rule_quiet(ASCII_RULE_QUIET);
        g.set_rule_active(ASCII_RULE_ACTIVE);
        g.set_scrollbar(ASCII_SCROLLBAR);
        g.set(GlyphRole::DotOn, "*");
        g.set(GlyphRole::DotOff, "o");
        self
    }

    #[must_use]
    pub fn borders_set(mut self, b: BorderSet) -> Self {
        self.theme.design.borders = b;
        if b == crate::theme::border::ASCII {
            self = self.ascii_glyphs();
        }
        self
    }

    #[must_use]
    pub fn glyph(mut self, r: GlyphRole, s: &'static str) -> Self {
        self.theme.design.glyphs.set(r, s);
        self
    }

    #[must_use]
    pub fn space(mut self, s: SpaceTokens) -> Self {
        self.theme.design.space = s;
        self
    }

    #[must_use]
    pub fn size(mut self, s: SizeTokens) -> Self {
        self.theme.design.size = s;
        self
    }

    #[must_use]
    pub fn density(mut self, d: Density) -> Self {
        self.theme.design.density = d;
        self
    }

    #[must_use]
    pub fn motion(mut self, m: MotionTokens) -> Self {
        self.theme.design.motion = m;
        self
    }

    #[must_use]
    pub fn mono_rules(mut self, f: Family, rules: &[MonoRule]) -> Self {
        self.theme
            .recipes
            .set_consumer_mono_rules(f, rules.to_vec());
        self
    }

    pub fn build(mut self) -> Theme {
        derive_unset(&mut self.theme.color);
        self.theme
    }
}

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

pub(crate) fn lightness(c: Color) -> f64 {
    rgb_of(c).map_or(0.0, |rgb| lab_of(rgb).0)
}

fn shift_l(color: Color, dl: f64) -> Color {
    let Some(rgb) = rgb_of(color) else {
        return color;
    };
    let (light, ca, cb) = lab_of(rgb);
    let (red, green, blue) = lab_to_rgb(((light + dl).clamp(0.0, 100.0), ca, cb));
    Color::Rgb(red, green, blue)
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FadeOutcome {
    Blended(Color),
    ApplyDim,
    Unchanged,
}

#[expect(clippy::float_cmp, reason = "edge rows use exact configured strengths")]
pub fn fade_mix(fg: Color, bg: Color, amount: f32) -> FadeOutcome {
    let (Color::Rgb(fr, fg, fb), Color::Rgb(br, bg, bb)) = (fg, bg) else {
        return if amount == 0.55 {
            FadeOutcome::ApplyDim
        } else {
            FadeOutcome::Unchanged
        };
    };
    let mix = |front: u8, back: u8| {
        (f32::from(front) * amount + f32::from(back) * (1.0 - amount))
            .round()
            .clamp(0.0, 255.0) as u8
    };
    FadeOutcome::Blended(Color::Rgb(mix(fr, br), mix(fg, bg), mix(fb, bb)))
}

pub(crate) fn contrast(a: Color, b: Color) -> f64 {
    let (Some(a), Some(b)) = (rgb_of(a), rgb_of(b)) else {
        return 1.0;
    };
    let (la, lb) = (luminance(a) + 0.05, luminance(b) + 0.05);
    if la > lb { la / lb } else { lb / la }
}

fn anchor_l(base: Color, ratio: f64, light_base: bool) -> f64 {
    let mut lo = 0.0f64;
    let mut hi = 100.0f64;
    for _ in 0..24 {
        let mid = lo.midpoint(hi);
        let (r, g, b) = lab_to_rgb((mid, 0.0, 0.0));
        let c = contrast(Color::Rgb(r, g, b), base);
        let far_enough = c >= ratio;
        if light_base {
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

pub(crate) fn derive_unset(c: &mut ColorTokens) {
    let junie = super::builtin::junie::tokens();

    fill(&mut c.surfaces[0], || junie.surfaces[0]);
    fill(&mut c.accent, || junie.accent);
    fill(&mut c.danger, || junie.danger);
    fill(&mut c.warning, || junie.warning);
    fill(&mut c.success, || junie.success);
    fill(&mut c.info, || junie.info);
    let base = c.surfaces[0];
    let light = lightness(base) > 50.0;
    let step = if light { -4.0 } else { 4.0 };

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

    fill(&mut c.status_bar_bg, || s[1]);
    fill(&mut c.status_bar_fg, || fg0);
    fill(&mut c.status_accent, || c.accent);
    fill(&mut c.chrome_rule, || c.border_subtle);
    fill(&mut c.scroll_track, || c.border_subtle);
    fill(&mut c.scroll_thumb, || c.fg[2]);
    fill(&mut c.tag_ink, || c.fg[2]);

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
    if let super::MeterFillRest::Color(color) = &mut me.fill_rest {
        fill(color, || c.fg[3]);
    }
    fill(&mut me.stale, || c.fg[2]);
    fill(&mut me.unknown, || c.fg[3]);
    let series_default = [me.low, me.medium, me.high, c.info, c.accent, c.fg[1]];
    for (slot, d) in me.series.iter_mut().zip(series_default) {
        fill(slot, || d);
    }
}
