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

use ratatui_core::style::Color;
pub use ratatui_core::style::Modifier;

pub use border::BorderSet;
pub use builder::ThemeBuilder;
pub use downgrade::{MONO_RULES_PER_FAMILY, MonoRule, downgrade_color};
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

/// Rendering policy for the active row in a cursor-based list.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum CursorStyle {
    /// Show the theme's focus gutter.
    #[default]
    Bar,
    /// Highlight the active row while reserving a blank gutter cell.
    Fill,
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
        Theme {
            color: builtin::junie::tokens(),
            design: builtin::junie::design(),
            recipes: builtin::junie::recipes(),
            capability: Capability {
                color: ColorLevel::TrueColor,
            },
            capability_palettes: Some(std::sync::Arc::new(palettes::junie())),
        }
    }

    pub fn paper() -> Theme {
        builtin::paper::theme()
    }

    pub fn from_tokens(c: ColorTokens) -> Theme {
        let mut color = c;
        builder::derive_unset(&mut color);
        Theme {
            color,
            design: builtin::junie::design(),
            recipes: builtin::default_recipes(),
            capability: Capability {
                color: ColorLevel::TrueColor,
            },
            capability_palettes: None,
        }
    }

    pub fn builder(self) -> ThemeBuilder {
        ThemeBuilder::from_theme(self)
    }

    #[must_use]
    pub fn override_family(mut self, f: Family, edit: impl FnOnce(&mut RecipeEdit)) -> Theme {
        let mut e = RecipeEdit::default();
        edit(&mut e);
        if let Some(v) = e.default_variant_set() {
            self.recipes.get_mut(f).default_variant = v;
        }
        self.recipes.push_override(GlobalOverride {
            family: f,
            variant: None,
            parts: e.into_parts(),
        });
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
        if let Some(dv) = e.default_variant_set() {
            self.recipes.get_mut(f).default_variant = dv;
        }
        self.recipes.push_override(GlobalOverride {
            family: f,
            variant: Some(v),
            parts: e.into_parts(),
        });
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
        let recipe = self.recipes.get_mut(f);
        if let Some(dv) = e.default_variant_set() {
            recipe.default_variant = dv;
        }
        let target = recipe.variant_mut(v);
        for (p, part) in e.into_parts().iter() {
            target.entry(p).merge_from(part);
        }
        self
    }

    #[must_use]
    pub fn define_family(mut self, f: Family, edit: impl FnOnce(&mut RecipeEdit)) -> Theme {
        let mut e = RecipeEdit::default();
        edit(&mut e);
        let recipe = self.recipes.get_mut(f);
        if let Some(dv) = e.default_variant_set() {
            recipe.default_variant = dv;
        }
        for (p, part) in e.into_parts().iter() {
            recipe.parts.entry(p).merge_from(part);
        }
        self
    }

    /// Baseline modal dim. Glyphs stay. Accent and secondary ink become faint.
    /// A rule that already fails contrast stays the ghost gray.
    pub fn backdrop_colors(&self, fg: Color, bg: Color) -> (Color, Color) {
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
        let new_bg = if bg == canvas || bg == surface || bg == elevated {
            bg
        } else if bg == tokens.field || bg == tokens.field_hover {
            elevated
        } else if bg == Color::Reset {
            canvas
        } else {
            overlay
        };
        let dark = bg == Color::Reset
            || bg == canvas
            || bg == surface
            || bg == elevated
            || bg == tokens.field
            || bg == tokens.field_hover;
        let plane = if bg == Color::Reset { canvas } else { bg };
        let reads = |ink: Color, plane: Color| builder::contrast(ink, plane) >= 3.0;
        let blank = fg == bg || ((fg == canvas || fg == surface) && dark && !reads(fg, plane));
        let mut mapped = if blank {
            new_bg
        } else if fg == canvas || fg == surface {
            muted
        } else if fg == tokens.accent || fg == tokens.focus {
            faint
        } else if fg == primary || fg == tokens.danger || fg == tokens.warning {
            muted
        } else if fg == secondary || fg == tokens.on_accent {
            faint
        } else {
            ghost
        };
        if !blank && reads(fg, plane) && !reads(mapped, new_bg) {
            mapped = [faint, muted, secondary]
                .into_iter()
                .find(|ink| reads(*ink, new_bg))
                .unwrap_or(primary);
        }
        if !blank && mapped == new_bg {
            mapped = if muted == new_bg { primary } else { muted };
        }
        (mapped, new_bg)
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
}
