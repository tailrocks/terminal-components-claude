pub(crate) use crate as theme;
pub(crate) use termrock_core::id;
pub use termrock_core::id::Part;
pub(crate) use termrock_core::response;
pub use termrock_core::response::StateFlags;

pub mod border;
pub mod builder;
pub mod builtin;
pub mod downgrade;
pub mod glyph;
pub mod palettes;
pub mod patch;
pub mod recipe;
pub mod resolve;
pub mod role;
pub mod tokens;
pub mod visual;

pub use ratatui_core::style::Modifier;
use ratatui_core::style::{Color, Style};

pub use border::BorderSet;
pub use builder::ThemeBuilder;
pub use downgrade::{
    LegacyMono, MONO_RULES_PER_FAMILY, MonoRule, downgrade_color, downgrade_pair, legacy_mono,
};
pub use glyph::{GlyphRole, GlyphSet};
pub use palettes::CapabilityPalettes;
pub use patch::{Slot, StateRule, StylePatch};
pub use recipe::{
    Family, Overlay, OverlayRule, PartEdit, PartMap, PartRecipe, Recipe, RecipeEdit, Recipes,
    Variant,
};
pub use resolve::{PaintStyle, PartMetrics, Resolved, ResolvedStyle, StyleDefaults};
pub use role::{Align, FG_STEPS, FgStep, MeterRole, Role, SURFACE_LEVELS, Surface, SyntaxRole};
pub use tokens::{
    Capability, ColorLevel, ColorTokens, Density, DesignTokens, MeterFillRest, MeterThresholds,
    MeterTokens, MotionTokens, SizeTokens, SpaceTokens, SyntaxTokens,
};
pub use visual::{
    BACKDROP_SELECTIONS, BackdropSelection, COLOR_LEVELS, FOCUS_BAR_POLICIES, FocusBarPolicy,
    GUTTER_CELLS, GutterCell, InactiveSelection, contrast_ratio,
};

/// Rendering policy for the active row in a cursor-based list.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum CursorStyle {
    /// Show the theme's focus gutter.
    #[default]
    Bar,
    /// Highlight the active row while reserving a blank gutter cell.
    Fill,
}

fn menu_cursor_modifiers(level: ColorLevel) -> (Modifier, Modifier, Modifier, Modifier) {
    let active = Modifier::BOLD
        | if level == ColorLevel::Mono {
            Modifier::REVERSED
        } else {
            Modifier::empty()
        };
    let hovered = active
        | if matches!(level, ColorLevel::Ansi16 | ColorLevel::Mono) {
            Modifier::UNDERLINED
        } else {
            Modifier::empty()
        };
    let pressed = Modifier::BOLD
        | match level {
            ColorLevel::Ansi16 => Modifier::REVERSED,
            ColorLevel::Mono => Modifier::UNDERLINED,
            ColorLevel::TrueColor | ColorLevel::Ansi256 => Modifier::empty(),
        };
    let disabled = if matches!(level, ColorLevel::Ansi16 | ColorLevel::Mono) {
        Modifier::DIM
    } else {
        Modifier::empty()
    };
    (active, hovered, pressed, disabled)
}

fn add_menu_cursor_profile(recipe: &mut RecipeEdit, level: ColorLevel) {
    let (active, hovered, pressed, disabled) = menu_cursor_modifiers(level);
    let cursor_modifiers = active | hovered | pressed;
    let active_style = StylePatch::new().add(active);
    let hover_style = StylePatch::new().add(hovered);
    let pressed_style = StylePatch::new().remove(active | hovered).add(pressed);
    let disabled_style = StylePatch::new().remove(cursor_modifiers).add(disabled);

    for part in [Part::ROW, Part::LABEL] {
        let style = recipe.part(part);
        style
            .base(StylePatch::new().remove(cursor_modifiers))
            .when(StateFlags::ACTIVE, active_style)
            .when(StateFlags::HOVERED, hover_style)
            .when(StateFlags::ACTIVE | StateFlags::HOVERED, hover_style)
            .when(StateFlags::PRESSED, pressed_style)
            .when(StateFlags::ACTIVE | StateFlags::PRESSED, pressed_style)
            .when(StateFlags::HOVERED | StateFlags::PRESSED, pressed_style)
            .when(
                StateFlags::ACTIVE | StateFlags::HOVERED | StateFlags::PRESSED,
                pressed_style,
            )
            .when(StateFlags::DISABLED, disabled_style)
            .when(StateFlags::ACTIVE | StateFlags::DISABLED, disabled_style)
            .when(StateFlags::HOVERED | StateFlags::DISABLED, disabled_style)
            .when(
                StateFlags::ACTIVE | StateFlags::HOVERED | StateFlags::DISABLED,
                disabled_style,
            );
    }
}

use self::recipe::GlobalOverride;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Theme {
    pub color: ColorTokens,
    pub design: DesignTokens,
    pub recipes: Recipes,
    pub capability: Capability,
    pub capability_palettes: Option<std::sync::Arc<CapabilityPalettes>>,
}

impl Theme {
    pub fn junie() -> Theme {
        let theme = Theme {
            color: builtin::junie::tokens(),
            design: builtin::junie::design(),
            recipes: builtin::junie::recipes(),
            capability: Capability {
                color: ColorLevel::TrueColor,
            },
            capability_palettes: Some(std::sync::Arc::new(palettes::junie())),
        };
        use crate::id::Part;
        use crate::recipe::Family;
        use crate::recipe::Variant;
        use crate::response::StateFlags;
        use crate::role::Role;
        use ColorLevel::{Ansi16, Ansi256, Mono, TrueColor};
        use ratatui_core::style::Color;

        let profiled_theme = [
            (
                TrueColor,
                Color::from_u32(0x00_4d_4d_4d),
                Color::from_u32(0x00_80_80_80),
            ),
            (Ansi256, Color::Indexed(239), Color::Indexed(245)),
            (Ansi16, Color::DarkGray, Color::Gray),
            (Mono, Color::DarkGray, Color::Gray),
        ]
        .into_iter()
        .fold(theme, |theme, (level, rest, hover)| {
            theme.define_family_for_level(level, Family::SPLIT, |recipe| {
                recipe
                    .part(Part::SEAM)
                    .base(StylePatch::new().set_fg(Role::Custom(rest)))
                    .when(
                        StateFlags::HOVERED,
                        StylePatch::new().set_fg(Role::Custom(hover)),
                    );
            })
        });

        let profiled_theme =
            [TrueColor, Ansi256, Ansi16, Mono]
                .into_iter()
                .fold(profiled_theme, |theme, level| {
                    theme.define_variant_for_level(
                        level,
                        Family::MENU,
                        Variant::DEFAULT,
                        |recipe| {
                            add_menu_cursor_profile(recipe, level);
                        },
                    )
                });

        let profiled_theme = [
            (
                TrueColor,
                Color::Rgb(122, 42, 42),
                Color::Rgb(140, 48, 48),
                Color::Rgb(90, 31, 31),
            ),
            (
                Ansi256,
                Color::Indexed(88),
                Color::Indexed(124),
                Color::Indexed(52),
            ),
            (Ansi16, Color::Red, Color::Red, Color::Red),
            (Mono, Color::DarkGray, Color::DarkGray, Color::DarkGray),
        ]
        .into_iter()
        .fold(profiled_theme, |theme, (level, rest, hover, pressed)| {
            let (active_modifiers, hover_modifiers, pressed_modifiers, disabled_modifiers) =
                menu_cursor_modifiers(level);
            let disabled_ink = Role::DisabledFg;
            theme.define_variant_for_level(level, Family::MENU, Variant::DANGER, |recipe| {
                add_menu_cursor_profile(recipe, level);
                let disabled_style = StylePatch::new()
                    .set_fg(disabled_ink)
                    .set_bg(Role::CurrentSurface)
                    .add(disabled_modifiers)
                    .remove(
                        Modifier::BOLD | active_modifiers | hover_modifiers | pressed_modifiers,
                    );
                let danger_active = StylePatch::new()
                    .set_fg(Role::HighlightDangerFg)
                    .set_bg(Role::Custom(rest))
                    .add(active_modifiers);
                let danger_hover = StylePatch::new()
                    .set_fg(Role::HighlightDangerFg)
                    .set_bg(Role::Custom(hover))
                    .add(hover_modifiers);
                let danger_press = StylePatch::new()
                    .set_fg(Role::HighlightDangerFg)
                    .set_bg(Role::Custom(pressed))
                    .remove(hover_modifiers | active_modifiers)
                    .add(pressed_modifiers);

                recipe
                    .part(Part::ROW)
                    .when(StateFlags::ACTIVE, danger_active)
                    .when(StateFlags::HOVERED, danger_hover)
                    .when(StateFlags::PRESSED, danger_press)
                    .when(StateFlags::ACTIVE | StateFlags::HOVERED, danger_hover)
                    .when(StateFlags::ACTIVE | StateFlags::PRESSED, danger_press)
                    .when(StateFlags::DISABLED, disabled_style)
                    .when(StateFlags::ACTIVE | StateFlags::DISABLED, disabled_style);

                let label_active = StylePatch::new()
                    .set_fg(Role::HighlightDangerFg)
                    .clear_bg()
                    .add(active_modifiers);
                let label_hover = StylePatch::new()
                    .set_fg(Role::HighlightDangerFg)
                    .clear_bg()
                    .add(hover_modifiers);
                let label_press = StylePatch::new()
                    .set_fg(Role::HighlightDangerFg)
                    .clear_bg()
                    .remove(hover_modifiers | active_modifiers)
                    .add(pressed_modifiers);

                recipe
                    .part(Part::LABEL)
                    .when(StateFlags::ACTIVE, label_active)
                    .when(StateFlags::HOVERED, label_hover)
                    .when(StateFlags::PRESSED, label_press)
                    .when(StateFlags::ACTIVE | StateFlags::HOVERED, label_hover)
                    .when(StateFlags::ACTIVE | StateFlags::PRESSED, label_press)
                    .when(StateFlags::DISABLED, disabled_style)
                    .when(StateFlags::ACTIVE | StateFlags::DISABLED, disabled_style);
            })
        });

        let mut theme = [
            (TrueColor, Color::Rgb(46, 154, 56)),
            (Ansi256, Color::Indexed(65)),
            (Ansi16, Color::LightGreen),
            (Mono, Color::White),
        ]
        .into_iter()
        .fold(profiled_theme, |theme, (level, pressed)| {
            theme.define_variant_for_level(level, Family::BUTTON, Variant::PRIMARY, |recipe| {
                recipe
                    .part(Part::CONTAINER)
                    .when(
                        StateFlags::HOVERED,
                        StylePatch::new().set_bg(Role::AccentHover),
                    )
                    .when(
                        StateFlags::PRESSED,
                        StylePatch::new().set_bg(Role::Custom(pressed)),
                    );
            })
        });
        theme.recipes.mark_builtin();
        theme
    }

    pub fn paper() -> Theme {
        builtin::paper::theme()
    }

    pub fn from_tokens(c: ColorTokens) -> Theme {
        let mut color = c;
        builder::derive_unset(&mut color);
        let mut theme = Theme {
            color,
            design: builtin::junie::design(),
            recipes: builtin::default_recipes(),
            capability: Capability {
                color: ColorLevel::TrueColor,
            },
            capability_palettes: None,
        };
        theme.recipes.mark_builtin();
        theme
    }

    pub fn builder(self) -> ThemeBuilder {
        ThemeBuilder::from_theme(self)
    }

    #[must_use]
    pub fn override_family(mut self, f: Family, edit: impl FnOnce(&mut RecipeEdit)) -> Theme {
        let mut e = RecipeEdit::default();
        edit(&mut e);
        let default_variant = e.default_variant_set();
        if let Some(v) = default_variant {
            self.recipes.get_mut(f).default_variant = v;
            self.recipes.record_authored_default_variant(f, v);
        }
        let override_ = GlobalOverride {
            family: f,
            variant: None,
            parts: e.into_parts(),
        };
        self.recipes.push_override(override_.clone());
        self.recipes.record_authored_override(override_);
        self
    }

    #[must_use]
    pub fn override_variant(
        mut self,
        f: Family,
        v: Variant,
        edit: impl FnOnce(&mut RecipeEdit),
    ) -> Theme {
        let mut e = RecipeEdit::default();
        edit(&mut e);
        let default_variant = e.default_variant_set();
        if let Some(dv) = default_variant {
            self.recipes.get_mut(f).default_variant = dv;
            self.recipes.record_authored_default_variant(f, dv);
        }
        let override_ = GlobalOverride {
            family: f,
            variant: Some(v),
            parts: e.into_parts(),
        };
        self.recipes.push_override(override_.clone());
        self.recipes.record_authored_override(override_);
        self
    }

    #[must_use]
    pub fn define_variant(
        mut self,
        f: Family,
        v: Variant,
        edit: impl FnOnce(&mut RecipeEdit),
    ) -> Theme {
        let mut e = RecipeEdit::default();
        edit(&mut e);
        let default_variant = e.default_variant_set();
        let parts = e.into_parts();
        let recipe = self.recipes.get_mut(f);
        if let Some(dv) = default_variant {
            recipe.default_variant = dv;
        }
        let target = recipe.variant_mut(v);
        for (p, part) in parts.iter() {
            target.entry(p).merge_from(part);
        }
        self.recipes
            .record_authored_variant(f, v, default_variant, &parts);
        self
    }

    #[must_use]
    pub fn define_family(mut self, f: Family, edit: impl FnOnce(&mut RecipeEdit)) -> Theme {
        let mut e = RecipeEdit::default();
        edit(&mut e);
        let default_variant = e.default_variant_set();
        let parts = e.into_parts();
        let recipe = self.recipes.get_mut(f);
        if let Some(dv) = default_variant {
            recipe.default_variant = dv;
        }
        for (p, part) in parts.iter() {
            recipe.parts.entry(p).merge_from(part);
        }
        self.recipes
            .record_authored_family(f, default_variant, &parts);
        self
    }

    /// Define capability-specific family defaults that apply before ordinary user overrides.
    ///
    /// A level default only replaces channels that remain equal to the family recipe captured
    /// when the first profile for this family is defined. This lets a consumer customize one
    /// channel (including clearing it) without losing unrelated level-specific defaults.
    #[must_use]
    pub fn define_family_for_level(
        self,
        level: ColorLevel,
        family: Family,
        edit: impl FnOnce(&mut RecipeEdit),
    ) -> Theme {
        self.define_profile_for_level(level, family, None, None, edit)
    }

    /// Define capability-specific defaults for one resolved variant.
    ///
    /// These defaults apply after family defaults and before global, scope, and instance
    /// overrides. A profile only replaces channels whose source recipe and semantic token
    /// dependencies still match the theme captured when this profile was defined.
    #[must_use]
    pub fn define_variant_for_level(
        self,
        level: ColorLevel,
        family: Family,
        variant: Variant,
        edit: impl FnOnce(&mut RecipeEdit),
    ) -> Theme {
        self.define_profile_for_level(level, family, Some(variant), None, edit)
    }

    /// Define capability-specific defaults for one resolved variant on one surrounding surface.
    ///
    /// Use this for components whose semantic plane follows their container. The surface is a
    /// rendering context; it is independent of terminal dimensions and application identity.
    #[must_use]
    pub fn define_variant_for_level_on_surface(
        self,
        level: ColorLevel,
        family: Family,
        variant: Variant,
        surface: Surface,
        edit: impl FnOnce(&mut RecipeEdit),
    ) -> Theme {
        self.define_profile_for_level(level, family, Some(variant), Some(surface), edit)
    }

    fn define_profile_for_level(
        mut self,
        level: ColorLevel,
        family: Family,
        variant: Option<Variant>,
        surface: Option<Surface>,
        edit: impl FnOnce(&mut RecipeEdit),
    ) -> Theme {
        let mut e = RecipeEdit::default();
        edit(&mut e);
        let seed_source_tokens = self.color;
        let seed_level_tokens = self.downgrade(level).color;
        self.recipes.define_level_family(
            recipe::LevelProfileKey {
                level,
                family,
                variant,
                surface,
            },
            e.into_parts(),
            seed_source_tokens,
            seed_level_tokens,
        );
        self
    }

    /// Baseline modal dim. Glyphs stay. Accent and secondary ink become faint.
    /// A rule that already fails contrast stays the ghost gray.
    pub fn backdrop_colors(&self, fg: Color, bg: Color) -> (Color, Color) {
        let (mut mapped_fg, mapped_bg, blank) = self.map_backdrop(Some(fg), Some(bg), true);
        if !blank && mapped_fg == mapped_bg {
            mapped_fg = self.backdrop_collision_ink(mapped_bg);
        }
        (mapped_fg, mapped_bg)
    }

    /// Map a painted cell to the modal backdrop while replacing its modifiers.
    /// The glyph is preserved by the renderer; only nonblank dim cells receive DIM.
    ///
    /// The frozen baseline is split on the DIM modifier (see
    /// `docs/audits/2026-10-21-modal-backdrop-dim-vs-frozen-baseline.md`):
    /// the showcase dialog underlays carry zero SGR-2 runs while the frozen
    /// tablepro picker underlays are DIM-modded at 16/none. One shared
    /// mapping cannot serve both, so the mapping keeps the DIM behavior the
    /// currently-green tablepro frozen digests pin, and the dialog-side no-DIM
    /// divergence is an owner decision, not a silent change.
    #[must_use]
    pub fn backdrop_style(&self, style: Style) -> Style {
        let (mut fg, bg, blank) = self.map_backdrop(style.fg, style.bg, false);
        let dim = !blank
            && (fg == bg || matches!(self.capability.color, ColorLevel::Ansi16 | ColorLevel::Mono));
        if dim {
            fg = self.backdrop_collision_ink(bg);
        }
        let mut mapped = Style::new().fg(fg).bg(bg);
        if dim {
            mapped = mapped.add_modifier(Modifier::DIM);
        }
        mapped
    }

    fn backdrop_collision_ink(&self, bg: Color) -> Color {
        let muted = self.color.fg[2];
        if muted == bg { self.color.fg[0] } else { muted }
    }

    // The legacy concrete-color pair API treats Color::Reset as an absent
    // terminal default. The Style API must preserve Some(Reset) versus None.
    fn map_backdrop(
        &self,
        fg: Option<Color>,
        bg: Option<Color>,
        reset_background_is_canvas: bool,
    ) -> (Color, Color, bool) {
        let tokens = &self.color;
        let canvas = tokens.surfaces[0];
        let surface = tokens.surfaces[1];
        let elevated = tokens.surfaces[2];
        let overlay = tokens.surfaces[3];
        let primary = tokens.fg[0];
        let secondary = tokens.fg[1];
        let muted = tokens.fg[2];
        let faint = tokens.fg[3];
        let ghost = tokens.fg[4];
        let source_bg = bg.unwrap_or(Color::Reset);
        let new_bg = if bg == Some(canvas) || bg == Some(surface) || bg == Some(elevated) {
            source_bg
        } else if bg == Some(tokens.field) || bg == Some(tokens.field_hover) {
            elevated
        } else if bg.is_none() || (reset_background_is_canvas && source_bg == Color::Reset) {
            canvas
        } else {
            overlay
        };
        let dark = bg.is_none()
            || (reset_background_is_canvas && source_bg == Color::Reset)
            || source_bg == canvas
            || source_bg == surface
            || source_bg == elevated
            || source_bg == tokens.field
            || source_bg == tokens.field_hover;
        let plane = if bg.is_none() || (reset_background_is_canvas && source_bg == Color::Reset) {
            canvas
        } else {
            source_bg
        };
        let reads = |ink: Color, plane: Color| builder::contrast(ink, plane) >= 3.0;
        let blank = match fg {
            Some(color) if Some(color) == bg => true,
            Some(color) => (color == canvas || color == surface) && dark && !reads(color, plane),
            None => false,
        };
        let mut mapped = if blank {
            new_bg
        } else if matches!(fg, Some(color) if color == canvas || color == surface) {
            muted
        } else if matches!(fg, Some(color) if color == tokens.accent || color == tokens.focus) {
            faint
        } else if matches!(fg, Some(color) if color == primary || color == tokens.danger || color == Color::from_u32(0x00f0_5d5d) || color == tokens.warning)
        {
            muted
        } else if matches!(fg, Some(color) if color == secondary || color == tokens.on_accent) {
            faint
        } else {
            ghost
        };
        // The frozen capsule tab rule is seam `#3a3a3a`. It dims to ghost
        // and stays there; the contrast lift would turn it faint.
        let seam = Color::from_u32(0x003a_3a3a);
        if let Some(color) = fg
            && !blank
            && color != seam
            && reads(color, plane)
            && !reads(mapped, new_bg)
        {
            mapped = [faint, muted, secondary]
                .into_iter()
                .find(|ink| reads(*ink, new_bg))
                .unwrap_or(primary);
        }
        (mapped, new_bg, blank)
    }

    pub fn bg(&self, s: Surface) -> Color {
        match s {
            Surface::Field => self.color.field,
            Surface::FieldHover => self.color.field_hover,
            ladder => self
                .color
                .surfaces
                .get(ladder.level().unwrap_or(0))
                .copied()
                .unwrap_or(Color::Reset),
        }
    }

    pub fn raise(&self, s: Surface) -> Surface {
        match s {
            Surface::Field | Surface::FieldHover => Surface::FieldHover,
            ladder => Surface::from_level(ladder.level().unwrap_or(0).saturating_add(1)),
        }
    }

    pub fn resolve(
        &self,
        f: Family,
        v: Variant,
        p: crate::id::Part,
        s: crate::response::StateFlags,
        surface: Surface,
    ) -> Resolved {
        resolve::resolve_uncached(self, f, v, p, s, surface, &[], None)
    }

    pub fn metrics(
        &self,
        f: Family,
        v: Variant,
        p: crate::id::Part,
        s: crate::response::StateFlags,
    ) -> PartMetrics {
        resolve::metrics_of(&resolve::accumulate(self, f, v, p, s, &[]))
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn fingerprint(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        format!("{self:?}").hash(&mut h);
        h.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::Part;
    use crate::response::StateFlags;

    #[test]
    fn metrics_is_the_sizing_path_for_update() {
        let mut t = Theme::junie();

        {
            let r = t.recipes.get_mut(Family::FIELD);
            let p = r.parts.entry(Part::FIELD);
            p.size = Slot::Set(3);
            p.glyph = Slot::Set(GlyphRole::FocusBar);
        }
        for st in [StateFlags::empty(), StateFlags::FOCUSED, StateFlags::ERROR] {
            let m = t.metrics(Family::FIELD, Variant::DEFAULT, Part::FIELD, st);
            assert_eq!(m.size, Some(3));
            assert_eq!(m.glyph, Slot::Set(GlyphRole::FocusBar));

            for s in [Surface::Canvas, Surface::Field, Surface::Overlay] {
                let r = t.resolve(Family::FIELD, Variant::DEFAULT, Part::FIELD, st, s);
                assert_eq!((r.size, r.glyph, r.align), (m.size, m.glyph, m.align));
            }
        }
    }

    #[test]
    fn raise_is_ladder_index_arithmetic_not_colour_equality() {
        let mut t = Theme::junie();

        t.color.surfaces[1] = t.color.surfaces[0];
        assert_eq!(t.raise(Surface::Canvas), Surface::Surface);
        assert_eq!(t.raise(Surface::Surface), Surface::Elevated);
        assert_eq!(t.raise(Surface::Overlay), Surface::Popover);
        assert_eq!(t.bg(Surface::Canvas), t.bg(Surface::Surface));
        let light = Theme::paper();
        assert_eq!(light.raise(Surface::Canvas), Surface::Surface);
    }

    #[test]
    fn raise_saturates_at_the_last_level() {
        let t = Theme::junie();
        assert_eq!(t.raise(Surface::Popover), Surface::Popover);
        assert_eq!(Surface::from_level(99), Surface::Popover);
    }

    #[test]
    fn field_raises_to_field_hover() {
        let t = Theme::junie();
        assert_eq!(t.raise(Surface::Field), Surface::FieldHover);
        assert_eq!(t.raise(Surface::FieldHover), Surface::FieldHover);
        assert_eq!(t.bg(Surface::Field), t.color.field);
        assert_eq!(Surface::Field.level(), None);
    }

    #[test]
    fn paper_theme_inverts_the_plane_direction() {
        let p = Theme::paper();
        let l0 = builder::lightness(p.bg(Surface::Canvas));
        let l1 = builder::lightness(p.bg(Surface::Surface));
        assert!(l1 < l0, "paper hover darkens");
        let j = Theme::junie();
        assert!(
            builder::lightness(j.bg(Surface::Surface)) > builder::lightness(j.bg(Surface::Canvas))
        );
        assert_eq!(p.design.density, Density::Compact);
        assert_eq!(
            p.recipes.get(Family::BUTTON).map(|r| r.default_variant),
            Some(Variant::SECONDARY)
        );
    }

    #[test]
    fn paper_tokens_are_pinned() {
        let p = Theme::paper().color;
        assert_eq!(p.surfaces[0], Color::from_u32(0xfbfaf8));
        assert_eq!(p.surfaces[4], Color::from_u32(0xcfc8bb));
        assert_eq!(p.accent, Color::from_u32(0x3b5bdb));
        assert_eq!(p.danger, Color::from_u32(0xb02525));
        assert_eq!(p.border_strong, Color::from_u32(0x9c948a));

        assert_eq!(p.field, p.surfaces[1]);
        assert_eq!(p.focus, p.accent);
        assert_eq!(p.accent_hover, Color::Rgb(88, 110, 242));
        assert_eq!(p.accent_pressed, Color::Rgb(15, 72, 196));
        assert_eq!(p.on_accent, p.surfaces[0]);
        assert_eq!(p.on_danger, p.surfaces[0]);
        assert_eq!(p.accent_tint, Color::Rgb(220, 222, 234));
        assert!(!p.colors().contains(&Color::Reset));
    }

    #[test]
    fn theme_is_byte_identical_after_a_scoped_render() {
        let t = Theme::junie();
        let before = t.fingerprint();
        static OV: [OverlayRule; 1] = [(
            Family::BUTTON,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::empty(),
            StylePatch::new().set_fg(Role::Warning),
        )];
        let inst = StylePatch::new().set_fg(Role::Danger);
        let r = resolve::resolve_uncached(
            &t,
            Family::BUTTON,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::empty(),
            Surface::Canvas,
            &[Overlay::new(&OV)],
            Some(&inst),
        );
        assert_eq!(r.style.fg, Some(t.color.danger));
        assert_eq!(t.fingerprint(), before);
    }

    #[test]
    fn overrides_reach_resolution() {
        let t = Theme::junie().override_family(Family::BUTTON, |r| {
            r.default_variant(Variant::SECONDARY);
            r.part(Part::GUTTER).glyph(GlyphRole::FocusBar);
            r.part(Part::LABEL)
                .base(StylePatch::new().set_fg(Role::Fg(FgStep::Primary)))
                .when(StateFlags::FOCUSED, StylePatch::new().add(Modifier::BOLD))
                .when(
                    StateFlags::DISABLED,
                    StylePatch::new()
                        .set_fg(Role::DisabledFg)
                        .remove(Modifier::BOLD),
                );
            r.part(Part::CONTAINER)
                .when(
                    StateFlags::HOVERED,
                    StylePatch::new().set_bg(Role::AccentTint),
                )
                .when(
                    StateFlags::HOVERED | StateFlags::PRESSED,
                    StylePatch::new()
                        .set_bg(Role::AccentPressed)
                        .set_fg(Role::OnAccent),
                );
        });
        let r = t.resolve(
            Family::BUTTON,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::HOVERED | StateFlags::PRESSED,
            Surface::Canvas,
        );
        assert_eq!(r.style.bg, Some(t.color.accent_pressed));
        let g = t.resolve(
            Family::BUTTON,
            Variant::DEFAULT,
            Part::GUTTER,
            StateFlags::empty(),
            Surface::Canvas,
        );
        assert_eq!(g.glyph, Slot::Set(GlyphRole::FocusBar));
        let v = Theme::junie().override_variant(Family::BUTTON, Variant::DANGER, |r| {
            r.part(Part::LABEL)
                .base(StylePatch::new().set_fg(Role::Info));
        });
        assert_eq!(
            v.resolve(
                Family::BUTTON,
                Variant::DANGER,
                Part::LABEL,
                StateFlags::empty(),
                Surface::Canvas
            )
            .style
            .fg,
            Some(v.color.info)
        );
        assert_eq!(
            v.resolve(
                Family::BUTTON,
                Variant::PRIMARY,
                Part::LABEL,
                StateFlags::empty(),
                Surface::Canvas
            )
            .style
            .fg,
            None
        );
        let d = Theme::junie()
            .define_family(Family::custom("seg"), |r| {
                r.part(Part::LABEL)
                    .base(StylePatch::new().set_fg(Role::Accent))
                    .size(3);
            })
            .define_variant(Family::custom("seg"), Variant::custom("outline"), |r| {
                r.part(Part::LABEL)
                    .base(StylePatch::new().set_fg(Role::Warning));
            });
        let base = d.resolve(
            Family::custom("seg"),
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::empty(),
            Surface::Canvas,
        );
        assert_eq!((base.style.fg, base.size), (Some(d.color.accent), Some(3)));
        let out = d.resolve(
            Family::custom("seg"),
            Variant::custom("outline"),
            Part::LABEL,
            StateFlags::empty(),
            Surface::Canvas,
        );
        assert_eq!(out.style.fg, Some(d.color.warning));
    }

    fn seam_fg(theme: &Theme, state: StateFlags) -> Option<Color> {
        theme
            .resolve(
                Family::SPLIT,
                Variant::DEFAULT,
                Part::SEAM,
                state,
                Surface::Canvas,
            )
            .style
            .as_style()
            .fg
    }

    #[test]
    fn junie_split_seam_uses_capability_profile_and_pressed_accent_precedence() {
        let cases = [
            (
                ColorLevel::TrueColor,
                Color::Rgb(77, 77, 77),
                Color::Rgb(128, 128, 128),
            ),
            (
                ColorLevel::Ansi256,
                Color::Indexed(239),
                Color::Indexed(245),
            ),
            (ColorLevel::Ansi16, Color::DarkGray, Color::Gray),
            (ColorLevel::Mono, Color::DarkGray, Color::Gray),
        ];
        for (level, rest, hover) in cases {
            let mut theme = Theme::junie();
            theme.capability.color = level;
            assert_eq!(
                seam_fg(&theme, StateFlags::empty()),
                Some(rest),
                "{level:?}"
            );
            assert_eq!(
                seam_fg(&theme, StateFlags::HOVERED),
                Some(hover),
                "{level:?}"
            );
            let accent = resolve::bind_role(&theme, Role::Accent, Surface::Canvas);
            assert_eq!(
                seam_fg(&theme, StateFlags::HOVERED | StateFlags::PRESSED),
                accent,
                "pressed has precedence over hover at {level:?}"
            );
        }
    }

    #[test]
    fn seam_profile_does_not_alias_custom_disabled_tokens_and_user_recipe_channels_win() {
        let mut theme = Theme::junie();
        theme.capability.color = ColorLevel::Ansi256;
        theme.color.disabled_fg = Color::Indexed(17);
        assert_eq!(
            seam_fg(&theme, StateFlags::empty()),
            Some(Color::Indexed(239))
        );

        let custom = Theme::junie().define_family(Family::SPLIT, |recipe| {
            recipe
                .part(Part::SEAM)
                .base(StylePatch::new().set_fg(Role::Danger));
        });
        assert_eq!(
            seam_fg(&custom, StateFlags::empty()),
            Some(custom.color.danger)
        );

        let custom_hover = Theme::junie().define_family(Family::SPLIT, |recipe| {
            recipe
                .part(Part::SEAM)
                .when(StateFlags::HOVERED, StylePatch::new().set_fg(Role::Danger));
        });
        assert_eq!(
            seam_fg(&custom_hover, StateFlags::HOVERED),
            Some(custom_hover.color.danger)
        );

        let custom_background = Theme::junie().define_family(Family::SPLIT, |recipe| {
            recipe
                .part(Part::SEAM)
                .base(StylePatch::new().set_bg(Role::Danger));
        });
        let custom_background_style = custom_background
            .resolve(
                Family::SPLIT,
                Variant::DEFAULT,
                Part::SEAM,
                StateFlags::empty(),
                Surface::Canvas,
            )
            .style;
        assert_eq!(
            custom_background_style.as_style().fg,
            Some(Color::Rgb(77, 77, 77))
        );
        assert_eq!(
            custom_background_style.as_style().bg,
            Some(custom_background.color.danger)
        );

        let override_theme = Theme::junie().override_family(Family::SPLIT, |recipe| {
            recipe
                .part(Part::SEAM)
                .base(StylePatch::new().set_fg(Role::Danger));
        });
        assert_eq!(
            seam_fg(&override_theme, StateFlags::empty()),
            Some(override_theme.color.danger)
        );

        let mut cleared = Theme::junie();
        cleared.capability.color = ColorLevel::Ansi256;
        cleared
            .recipes
            .get_mut(Family::SPLIT)
            .parts
            .entry(Part::SEAM)
            .base
            .fg = Slot::Clear;
        assert_eq!(seam_fg(&cleared, StateFlags::empty()), None);
    }

    #[test]
    fn authored_mono_profile_survives_downgrade_without_changing_custom_colors() {
        let profile = Theme::junie()
            .define_family_for_level(ColorLevel::Mono, Family::SPLIT, |recipe| {
                recipe
                    .part(Part::SEAM)
                    .base(StylePatch::new().set_fg(Role::Custom(Color::DarkGray)))
                    .when(
                        StateFlags::HOVERED,
                        StylePatch::new().set_fg(Role::Custom(Color::Gray)),
                    )
                    .when(
                        StateFlags::PRESSED,
                        StylePatch::new().set_fg(Role::Custom(Color::White)),
                    );
            })
            .downgrade(ColorLevel::Mono);
        assert_eq!(
            seam_fg(&profile, StateFlags::empty()),
            Some(Color::DarkGray)
        );
        assert_eq!(seam_fg(&profile, StateFlags::HOVERED), Some(Color::Gray));
        assert_eq!(seam_fg(&profile, StateFlags::PRESSED), Some(Color::White));

        let custom = Theme::junie()
            .define_family(Family::SPLIT, |recipe| {
                recipe
                    .part(Part::SEAM)
                    .base(StylePatch::new().set_fg(Role::Custom(Color::Rgb(128, 128, 128))));
            })
            .downgrade(ColorLevel::Mono);
        assert_eq!(
            seam_fg(&custom, StateFlags::empty()),
            Some(crate::downgrade::downgrade_color(
                Color::Rgb(128, 128, 128),
                ColorLevel::Mono
            ))
        );
    }

    #[test]
    fn style_cache_key_tracks_capability_level_and_recipe_revision() {
        let mut theme = Theme::junie();
        let mut cache = resolve::StyleCache::new();
        let query = |cache: &mut resolve::StyleCache, theme: &Theme| {
            cache.accumulate(
                theme,
                Family::SPLIT,
                Variant::DEFAULT,
                Part::SEAM,
                StateFlags::empty(),
                &[],
                0,
            )
        };
        let truecolor = query(&mut cache, &theme);
        theme.capability.color = ColorLevel::Ansi256;
        let ansi = query(&mut cache, &theme);
        assert_ne!(truecolor.fg, ansi.fg);
        assert_eq!(cache.stats(), (0, 2));

        theme
            .recipes
            .get_mut(Family::SPLIT)
            .parts
            .entry(Part::SEAM)
            .base = StylePatch::new().set_fg(Role::Danger);
        let customized = query(&mut cache, &theme);
        assert_eq!(customized.fg, Slot::Set(Role::Danger));
        assert_eq!(cache.stats(), (0, 3));
    }

    #[test]
    fn style_cache_key_tracks_iter_mut_recipe_edits() {
        let mut theme = Theme::junie();
        theme.capability.color = ColorLevel::Ansi256;
        let mut cache = resolve::StyleCache::new();
        let query = |cache: &mut resolve::StyleCache, theme: &Theme| {
            cache.accumulate(
                theme,
                Family::SPLIT,
                Variant::DEFAULT,
                Part::SEAM,
                StateFlags::empty(),
                &[],
                0,
            )
        };

        let before = query(&mut cache, &theme);
        for (family, recipe) in theme.recipes.iter_mut() {
            if family == Family::SPLIT {
                recipe.parts.entry(Part::SEAM).base.fg = Slot::Set(Role::Danger);
            }
        }
        let after = query(&mut cache, &theme);

        assert_ne!(before.fg, after.fg);
        assert_eq!(after.fg, Slot::Set(Role::Danger));
        assert_eq!(cache.stats(), (0, 2));
    }

    #[test]
    fn style_cache_separates_divergent_cloned_theme_profiles() {
        let base = Theme::junie();
        let mut left = base.clone();
        left.capability.color = ColorLevel::Ansi256;
        let left = left.define_family_for_level(ColorLevel::Ansi256, Family::SPLIT, |recipe| {
            recipe
                .part(Part::SEAM)
                .base(StylePatch::new().set_fg(Role::Custom(Color::Indexed(38))));
        });
        let mut right = base;
        right.capability.color = ColorLevel::Ansi256;
        let right = right.define_family_for_level(ColorLevel::Ansi256, Family::SPLIT, |recipe| {
            recipe
                .part(Part::SEAM)
                .base(StylePatch::new().set_fg(Role::Custom(Color::Indexed(39))));
        });
        let mut cache = resolve::StyleCache::new();
        let query = |cache: &mut resolve::StyleCache, theme: &Theme| {
            cache.accumulate(
                theme,
                Family::SPLIT,
                Variant::DEFAULT,
                Part::SEAM,
                StateFlags::empty(),
                &[],
                0,
            )
        };

        let left_style = query(&mut cache, &left);
        let right_style = query(&mut cache, &right);

        assert_eq!(left_style.fg, Slot::Set(Role::Custom(Color::Indexed(38))));
        assert_eq!(right_style.fg, Slot::Set(Role::Custom(Color::Indexed(39))));
        assert_eq!(cache.stats(), (0, 2));
    }
}
