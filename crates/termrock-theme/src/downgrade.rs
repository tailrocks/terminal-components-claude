use ratatui_core::style::{Color, Modifier};

use super::Theme;
use super::glyph::GlyphRole;
use super::patch::{Slot, StylePatch};
use super::recipe::{Family, Recipes};
use super::role::{FgStep, Role, Surface};
use super::tokens::{ColorLevel, ColorTokens};
use crate::id::Part;
use crate::response::StateFlags;

const XTERM16: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (205, 0, 0),
    (0, 205, 0),
    (205, 205, 0),
    (0, 0, 238),
    (205, 0, 205),
    (0, 205, 205),
    (229, 229, 229),
    (127, 127, 127),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (92, 92, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
];

const fn named_index(c: Color) -> Option<usize> {
    match c {
        Color::Black => Some(0),
        Color::Red => Some(1),
        Color::Green => Some(2),
        Color::Yellow => Some(3),
        Color::Blue => Some(4),
        Color::Magenta => Some(5),
        Color::Cyan => Some(6),
        Color::Gray => Some(7),
        Color::DarkGray => Some(8),
        Color::LightRed => Some(9),
        Color::LightGreen => Some(10),
        Color::LightYellow => Some(11),
        Color::LightBlue => Some(12),
        Color::LightMagenta => Some(13),
        Color::LightCyan => Some(14),
        Color::White => Some(15),
        Color::Reset | Color::Rgb(..) | Color::Indexed(_) => None,
    }
}

const fn cube_value(step: u8) -> u8 {
    if step == 0 {
        0
    } else {
        55u8.saturating_add(step.saturating_mul(40))
    }
}

fn rgb_of_index(i: u8) -> (u8, u8, u8) {
    if let Some(&c) = XTERM16.get(usize::from(i)) {
        return c;
    }
    if i >= 232 {
        let v = 8u8.saturating_add(i.saturating_sub(232).saturating_mul(10));
        return (v, v, v);
    }
    let n = i.saturating_sub(16);
    (
        cube_value(n / 36),
        cube_value((n / 6) % 6),
        cube_value(n % 6),
    )
}

pub(crate) fn rgb_of(c: Color) -> Option<(u8, u8, u8)> {
    match c {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        Color::Indexed(i) => Some(rgb_of_index(i)),
        Color::Reset => None,
        named => named_index(named).and_then(|i| XTERM16.get(i).copied()),
    }
}

fn baseline_ansi256(rgb: (u8, u8, u8)) -> Option<u8> {
    match rgb {
        (0xff, 0x6b, 0x6b) | (0xf0, 0x5d, 0x5d) => Some(203),

        (0x3f, 0xb9, 0x50) => Some(71),

        (0x19, 0x19, 0x1c) => Some(232),

        (0x3a, 0x3a, 0x3a) => Some(238),

        (0x47, 0x47, 0x47) | (0x4d, 0x4d, 0x4d) => Some(239),

        (0x80, 0x80, 0x80) => Some(245),
        _ => None,
    }
}

fn nearest_256(rgb: (u8, u8, u8)) -> u8 {
    let (r, g, b) = rgb;
    let step = |value: u8| ((u32::from(value) * 5 + 127) / 255) as u8;
    let (red, green, blue) = (step(r), step(g), step(b));
    let cube = 16 + 36 * red + 6 * green + blue;
    let cube_value = |index: u8| -> i32 {
        if index == 0 {
            0
        } else {
            55 + i32::from(index) * 40
        }
    };
    let distance = |candidate: (i32, i32, i32)| {
        (candidate.0 - i32::from(r)).pow(2)
            + (candidate.1 - i32::from(g)).pow(2)
            + (candidate.2 - i32::from(b)).pow(2)
    };
    let cube_error = distance((cube_value(red), cube_value(green), cube_value(blue)));

    let average = (i32::from(r) + i32::from(g) + i32::from(b)) / 3;
    let gray_step = ((average - 8).max(0) / 10).min(23);
    let gray_value = 8 + gray_step * 10;
    let gray_error = distance((gray_value, gray_value, gray_value));
    let chroma = i32::from(r.max(g).max(b)) - i32::from(r.min(g).min(b));

    if gray_error < cube_error && chroma <= 30 {
        232 + gray_step as u8
    } else {
        cube
    }
}

fn to_lin(c: u8) -> f64 {
    let v = f64::from(c) / 255.0;
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

pub(crate) fn lab_of(rgb: (u8, u8, u8)) -> (f64, f64, f64) {
    let (red, green, blue) = (to_lin(rgb.0), to_lin(rgb.1), to_lin(rgb.2));
    let x = (red * 0.412_456_4 + green * 0.357_576_1 + blue * 0.180_437_5) / 0.950_47;
    let y = red * 0.212_672_9 + green * 0.715_152_2 + blue * 0.072_175_0;
    let z = (red * 0.019_333_9 + green * 0.119_192_0 + blue * 0.950_304_1) / 1.088_83;
    let f = |t: f64| {
        if t > 0.008_856 {
            t.cbrt()
        } else {
            7.787 * t + 16.0 / 116.0
        }
    };
    let (fx, fy, fz) = (f(x), f(y), f(z));
    (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
}

fn nearest_16(rgb: (u8, u8, u8)) -> Color {
    let (r, g, b) = rgb;
    let lum = (u32::from(r)
        .saturating_mul(299)
        .saturating_add(u32::from(g).saturating_mul(587))
        .saturating_add(u32::from(b).saturating_mul(114)))
        / 1000;
    let max = u32::from(r.max(g).max(b));
    let min = u32::from(r.min(g).min(b));
    if max.saturating_sub(min) < 40 {
        return match lum {
            0..=30 => Color::Black,
            31..=110 => Color::DarkGray,
            111..=200 => Color::Gray,
            _ => Color::White,
        };
    }
    let bright = max > 180;
    match (r >= g && r >= b, g >= r && g >= b) {
        (true, _) if g > 120 && b < 80 => Color::Yellow,
        (true, _) => {
            if bright {
                Color::LightRed
            } else {
                Color::Red
            }
        }
        (_, true) => {
            if bright {
                Color::LightGreen
            } else {
                Color::Green
            }
        }
        _ => {
            if bright {
                Color::LightBlue
            } else {
                Color::Blue
            }
        }
    }
}

pub(crate) fn luminance(rgb: (u8, u8, u8)) -> f64 {
    0.2126 * to_lin(rgb.0) + 0.7152 * to_lin(rgb.1) + 0.0722 * to_lin(rgb.2)
}

fn mono(rgb: (u8, u8, u8)) -> Color {
    let y = luminance(rgb);
    if y < 0.35 {
        Color::Black
    } else if y > 0.75 {
        Color::White
    } else {
        Color::Reset
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegacyMono {
    pub fg: Color,
    pub bg: Color,
    pub underline: bool,
    pub reverse: bool,
}

pub fn legacy_mono(fg: Color, bg: Color, menu_row: bool) -> Option<LegacyMono> {
    let fg_rgb = rgb_of(fg)?;
    let bg_rgb = rgb_of(bg)?;
    let (out_fg, out_bg, underline, reverse) = match (fg_rgb, bg_rgb) {
        ((0x19, 0x19, 0x1c), (0x48, 0xe0, 0x54)) => (Color::Black, Color::White, false, menu_row),
        ((0x79, 0xb8, 0xff), (0, 0, 0)) => (Color::Gray, Color::Black, true, false),
        ((0xff, 0xff, 0xff), (0x27, 0x27, 0x2a)) => (Color::Black, Color::DarkGray, false, false),
        ((0x27, 0x27, 0x2a), (0x27, 0x27, 0x2a)) => {
            (Color::DarkGray, Color::DarkGray, false, false)
        }
        ((0x26, 0x26, 0x26), (0, 0, 0)) => (Color::DarkGray, Color::Black, false, false),
        ((0x80, 0x80, 0x80), (0, 0, 0)) => (Color::Gray, Color::Black, false, false),
        ((0x47, 0x47, 0x47), (0, 0, 0)) => (Color::DarkGray, Color::Black, false, false),
        ((0x8a, 0x8a, 0x8a) | (0xb3, 0xb3, 0xb3) | (0x73, 0x73, 0x73), (0, 0, 0)) => {
            (Color::Gray, Color::Black, false, false)
        }
        ((0x48, 0xe0, 0x54) | (0x3f, 0xb9, 0x50), (0, 0, 0)) => {
            (Color::White, Color::Black, false, false)
        }
        ((0xff, 0xff, 0xff), (0, 0, 0)) => (Color::White, Color::Black, false, false),
        ((0, 0, 0), (0, 0, 0)) => (Color::Black, Color::Black, false, false),
        _ => return None,
    };
    Some(LegacyMono {
        fg: out_fg,
        bg: out_bg,
        underline,
        reverse,
    })
}

pub fn downgrade_pair(fg: Color, bg: Color, level: ColorLevel) -> (Color, Color) {
    let mut fg = downgrade_color(fg, level);
    let bg = downgrade_color(bg, level);
    if level == ColorLevel::Ansi16 && fg == Color::White && bg == Color::DarkGray {
        fg = Color::Black;
    }
    (fg, bg)
}

pub fn downgrade_color(c: Color, level: ColorLevel) -> Color {
    let Some(rgb) = rgb_of(c) else {
        return c;
    };
    match level {
        ColorLevel::TrueColor => c,
        ColorLevel::Ansi256 => match c {
            Color::Rgb(..) => {
                Color::Indexed(baseline_ansi256(rgb).unwrap_or_else(|| nearest_256(rgb)))
            }
            other => other,
        },
        ColorLevel::Ansi16 => match c {
            Color::Rgb(..) | Color::Indexed(_) => nearest_16(rgb),
            other => other,
        },
        ColorLevel::Mono => mono(rgb),
    }
}

fn repair_ansi16_light_foreground(source: &ColorTokens, mapped: &mut ColorTokens) {
    let Some(canvas) = rgb_of(source.surfaces[0]) else {
        return;
    };
    if luminance(canvas) < 0.5 {
        return;
    }
    for foreground in &mut mapped.fg {
        if matches!(*foreground, Color::Gray | Color::White) {
            *foreground = Color::DarkGray;
        }
    }
}

struct PaletteProjection<'a> {
    source: &'a [super::MeterFillRest],
    target: &'a [super::MeterFillRest],
    generic: &'a [super::MeterFillRest],
    eligible: &'a mut [bool],
    index: usize,
    level: ColorLevel,
}
impl PaletteProjection<'_> {
    fn project(&mut self, current: super::MeterFillRest) -> super::MeterFillRest {
        let i = self.index;
        self.index = self.index.saturating_add(1);
        if let Some((((expected, authored), generic), eligible)) = self
            .source
            .get(i)
            .zip(self.target.get(i))
            .zip(self.generic.get(i))
            .zip(self.eligible.get_mut(i))
        {
            if current != *expected {
                *eligible = false;
            }
            if *eligible { *authored } else { *generic }
        } else {
            match current {
                super::MeterFillRest::Color(color) => {
                    super::MeterFillRest::Color(downgrade_color(color, self.level))
                }
                super::MeterFillRest::ReferenceLift => current,
            }
        }
    }
}
impl super::tokens::TokenMapper for PaletteProjection<'_> {
    fn color(&mut self, current: Color) -> Color {
        match self.project(super::MeterFillRest::Color(current)) {
            super::MeterFillRest::Color(color) => color,
            super::MeterFillRest::ReferenceLift => downgrade_color(current, self.level),
        }
    }
    fn meter_rest(&mut self, current: super::MeterFillRest) -> super::MeterFillRest {
        self.project(current)
    }
}

impl Theme {
    #[must_use]
    pub fn downgrade(&self, level: ColorLevel) -> Theme {
        if self.capability.color == level {
            return self.clone();
        }
        let mut out = self.clone();
        out.capability.color = level;
        out.color = self.color.map_colors(&mut |c| downgrade_color(c, level));
        if level == ColorLevel::Ansi16 {
            repair_ansi16_light_foreground(&self.color, &mut out.color);
        }
        if let Some(palettes) = &self.capability_palettes {
            let expected = palettes
                .projected
                .filter(|(level, _)| *level == self.capability.color)
                .map(|(_, tokens)| tokens)
                .or_else(|| palettes.get(self.capability.color))
                .unwrap_or_else(|| {
                    let mut expected = palettes
                        .source
                        .map_colors(&mut |c| downgrade_color(c, self.capability.color));
                    if self.capability.color == ColorLevel::Ansi16 {
                        repair_ansi16_light_foreground(&palettes.source, &mut expected);
                    }
                    expected
                });
            let source = expected.semantic_colors();
            let target = palettes.get(level).unwrap_or(out.color).semantic_colors();
            let generic = out.color.semantic_colors();
            let mut eligibility = palettes.eligible.clone();
            out.color = self.color.map_with(&mut PaletteProjection {
                source: &source,
                target: &target,
                generic: &generic,
                eligible: &mut eligibility,
                index: 0,
                level,
            });
            if let Some(palettes) = &mut out.capability_palettes {
                let palettes = std::sync::Arc::make_mut(palettes);
                palettes.eligible = eligibility;
                palettes.projected = Some((level, out.color));
            }
        }
        out
    }

    #[must_use]
    pub fn for_level(&self, detected: ColorLevel) -> Theme {
        self.downgrade(self.capability.color.narrow_to(detected))
    }

    #[must_use]
    pub fn for_terminal(&self) -> Theme {
        self.for_level(ColorLevel::detect())
    }
}

pub type MonoRule = (Part, StateFlags, StylePatch);

const BUTTON_GENERIC_MARKER_GLYPH_RULES: [(Part, StateFlags); 2] = [
    (Part::MARKER, StateFlags::SELECTED),
    (Part::MARKER, StateFlags::CHECKED),
];

fn mono_rules() -> [MonoRule; 15] {
    let p = StylePatch::new;
    [
        (
            Part::GUTTER,
            StateFlags::FOCUSED,
            p().set_glyph(GlyphRole::FocusBar),
        ),
        (Part::LABEL, StateFlags::FOCUSED, p().add(Modifier::BOLD)),
        (
            Part::MARKER,
            StateFlags::SELECTED,
            p().set_glyph(GlyphRole::Chosen),
        ),
        (
            Part::MARKER,
            StateFlags::CHECKED,
            p().set_glyph(GlyphRole::Checked),
        ),
        (
            Part::CONTAINER,
            StateFlags::PRESSED,
            p().set_bg(Role::Fg(FgStep::Primary))
                .set_fg(Role::Surface(Surface::Canvas))
                .add(Modifier::BOLD),
        ),
        (
            Part::LABEL,
            StateFlags::PRESSED,
            p().set_glyph(GlyphRole::PressLeft).add(Modifier::BOLD),
        ),
        (
            Part::GUTTER,
            StateFlags::DISABLED,
            StylePatch {
                glyph: Slot::Clear,
                ..p()
            },
        ),
        (
            Part::MARKER,
            StateFlags::DISABLED,
            StylePatch {
                glyph: Slot::Clear,
                ..p()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .remove(Modifier::all())
            },
        ),
        (
            Part::LABEL,
            StateFlags::DISABLED,
            p().set_fg(Role::DisabledFg)
                .remove(Modifier::all())
                .add(Modifier::DIM),
        ),
        (
            Part::FIELD,
            StateFlags::DISABLED,
            p().set_fg(Role::Fg(FgStep::Primary))
                .remove(Modifier::all())
                .add(Modifier::DIM),
        ),
        (
            Part::TEXT,
            StateFlags::DISABLED,
            p().set_fg(Role::Fg(FgStep::Primary))
                .remove(Modifier::all())
                .add(Modifier::DIM),
        ),
        (
            Part::MARKER,
            StateFlags::ERROR,
            p().set_glyph(GlyphRole::Error),
        ),
        (
            Part::FIELD,
            StateFlags::ERROR,
            p().add(Modifier::UNDERLINED),
        ),
        (
            Part::MARKER,
            StateFlags::WARNING,
            p().set_glyph(GlyphRole::Dirty),
        ),
        (
            Part::TEXT,
            StateFlags::EDITING,
            p().add(Modifier::UNDERLINED),
        ),
    ]
}

fn mono_rules_extra() -> [MonoRule; 5] {
    let p = StylePatch::new;
    [
        (
            Part::MARKER,
            StateFlags::DIRTY,
            p().set_glyph(GlyphRole::Dirty),
        ),
        (
            Part::RULE,
            StateFlags::ACTIVE,
            p().set_glyph(GlyphRole::RuleActive),
        ),
        (Part::LABEL, StateFlags::ACTIVE, p().add(Modifier::BOLD)),
        (
            Part::ICON,
            StateFlags::LOADING,
            p().add(Modifier::UNDERLINED),
        ),
        (Part::THUMB, StateFlags::PRESSED, p().add(Modifier::BOLD)),
    ]
}

pub const MONO_RULES_PER_FAMILY: usize = 20;

pub(crate) static VIEWPORT_MONO_RULES: [MonoRule; 1] = [(
    Part::TEXT,
    StateFlags::SELECTED,
    StylePatch::new()
        .set_fg(Role::Surface(Surface::Canvas))
        .set_bg(Role::Fg(FgStep::Primary))
        .add(Modifier::UNDERLINED),
)];

pub(crate) static GRID_MONO_RULES: [MonoRule; 2] = [
    (
        Part::CELL,
        StateFlags::ERROR,
        StylePatch::new().add(Modifier::UNDERLINED),
    ),
    (
        Part::ROW,
        StateFlags::PRESSED,
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary))
            .add(Modifier::BOLD),
    ),
];

pub(crate) static MENU_MONO_RULES: [MonoRule; 2] = [
    (
        Part::ROW,
        StateFlags::PRESSED,
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary))
            .add(Modifier::BOLD),
    ),
    (
        Part::TITLE,
        StateFlags::PRESSED,
        StylePatch::new()
            .set_glyph(GlyphRole::PressLeft)
            .add(Modifier::BOLD),
    ),
];

pub(crate) static HELP_MONO_RULES: [MonoRule; 2] = [
    (
        Part::BORDER,
        StateFlags::FOCUSED,
        StylePatch::new().add(Modifier::BOLD),
    ),
    (
        Part::TITLE,
        StateFlags::FOCUSED,
        StylePatch::new().add(Modifier::UNDERLINED),
    ),
];

pub(crate) static PICKER_MONO_RULES: [MonoRule; 3] = [
    (
        Part::GUTTER,
        StateFlags::ERROR,
        StylePatch::new().set_glyph(GlyphRole::Error),
    ),
    (
        Part::GUTTER,
        StateFlags::BUSY,
        StylePatch::new().set_glyph(GlyphRole::MoreRows),
    ),
    (
        Part::LABEL,
        StateFlags::PRESSED,
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary))
            .add(Modifier::BOLD.union(Modifier::UNDERLINED)),
    ),
];

pub(crate) static SELECT_MONO_RULES: [MonoRule; 3] = [
    (
        Part::FIELD,
        StateFlags::PRESSED,
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary))
            .add(Modifier::BOLD),
    ),
    (
        Part::GUTTER,
        StateFlags::PRESSED,
        StylePatch::new()
            .set_glyph(GlyphRole::PressLeft)
            .add(Modifier::BOLD),
    ),
    (
        Part::MARKER,
        StateFlags::PRESSED,
        StylePatch::new()
            .set_glyph(GlyphRole::PressRight)
            .add(Modifier::BOLD),
    ),
];

pub(crate) static INPUT_MONO_RULES: [MonoRule; 1] = [(
    Part::ROW,
    StateFlags::SELECTED,
    StylePatch::new().add(Modifier::REVERSED),
)];

pub(crate) static BUTTON_MONO_RULES: [MonoRule; 1] = [(
    Part::GUTTER,
    StateFlags::FOCUS_VISIBLE,
    StylePatch::new().set_glyph(GlyphRole::FocusBar),
)];

pub(crate) fn builtin_family_mono_rules(family: Family) -> &'static [MonoRule] {
    match family {
        Family::BUTTON => &BUTTON_MONO_RULES,
        Family::INPUT => &INPUT_MONO_RULES,
        Family::VIEWPORT => &VIEWPORT_MONO_RULES,
        Family::GRID => &GRID_MONO_RULES,
        Family::MENU => &MENU_MONO_RULES,
        Family::HELP => &HELP_MONO_RULES,
        Family::PICKER => &PICKER_MONO_RULES,
        Family::SELECT => &SELECT_MONO_RULES,
        _ => &[],
    }
}

pub(crate) fn apply_mono_fallback(
    acc: StylePatch,
    recipes: &Recipes,
    family: Family,
    part: Part,
    live: StateFlags,
) -> StylePatch {
    apply_mono_with_defaults(acc, recipes, family, part, live, None)
}

pub(crate) fn apply_mono_with_defaults(
    mut acc: StylePatch,
    recipes: &Recipes,
    family: Family,
    part: Part,
    live: StateFlags,
    authored: Option<&[MonoRule]>,
) -> StylePatch {
    let rules = mono_rules();
    let extra = mono_rules_extra();
    let builtin: &'static [MonoRule] = builtin_family_mono_rules(family);

    let targeted: &[MonoRule] = recipes
        .mono_rules(family)
        .or(authored.filter(|rules| !rules.is_empty()))
        .unwrap_or(builtin);
    for (rule_part, when, patch) in rules.iter().chain(extra.iter()) {
        if family == Family::BUTTON && *rule_part == Part::GUTTER && *when == StateFlags::FOCUSED {
            continue;
        }

        if family == Family::BUTTON
            && BUTTON_GENERIC_MARKER_GLYPH_RULES.contains(&(*rule_part, *when))
        {
            continue;
        }
        let applies = family != Family::SCROLLBAR
            || *rule_part != Part::CONTAINER
            || *when != StateFlags::PRESSED;
        if applies && *rule_part == part && live.contains(*when) {
            acc = acc.merge(*patch);
        }
    }
    for (rule_part, when, patch) in targeted {
        let applies = family != Family::SCROLLBAR
            || *rule_part != Part::CONTAINER
            || *when != StateFlags::PRESSED;
        if applies && *rule_part == part && live.contains(*when) {
            acc = acc.merge(*patch);
        }
    }
    acc
}
