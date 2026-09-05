//! The theme model (`COMPONENT_ARCHITECTURE.md` §11): concrete token data,
//! typed recipes, role-level patches, a scoped overlay stack and one
//! six-level precedence chain. No theme trait, no generic theme parameter.

pub mod border;
pub(crate) mod builder;
pub(crate) mod builtin;
pub(crate) mod downgrade;
pub(crate) mod glyph;
pub(crate) mod patch;
pub(crate) mod recipe;
pub(crate) mod resolve;
pub(crate) mod role;
pub(crate) mod tokens;

#[cfg(feature = "testing")]
use core::hash::{Hash, Hasher};

use ratatui_core::style::Color;
pub use ratatui_core::style::Modifier;

pub use border::BorderSet;
pub use builder::ThemeBuilder;
pub use downgrade::{MONO_RULES_PER_FAMILY, MonoRule, downgrade_color};
pub use glyph::{GlyphRole, GlyphSet};
pub use patch::{Slot, StateRule, StylePatch};
pub use recipe::{
    Family, Overlay, OverlayRule, PartEdit, PartMap, PartRecipe, Recipe, RecipeEdit, Recipes,
    Variant,
};
pub use resolve::{PartMetrics, Resolved};
pub use role::{Align, FG_STEPS, FgStep, MeterRole, Role, SURFACE_LEVELS, Surface, SyntaxRole};
pub use tokens::{
    Capability, ColorLevel, ColorTokens, Density, DesignTokens, MeterThresholds, MeterTokens,
    MotionTokens, SizeTokens, SpaceTokens, SyntaxTokens,
};

use self::recipe::GlobalOverride;

/// A complete theme: colours, design tokens, recipes and the capability
/// it is resolved for.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Theme {
    /// Colour tokens.
    pub color: ColorTokens,
    /// Design tokens.
    pub design: DesignTokens,
    /// Recipes.
    pub recipes: Recipes,
    /// Capability.
    pub capability: Capability,
}

impl Theme {
    /// The approved default; token values unchanged from the baseline.
    pub fn junie() -> Theme {
        Theme {
            color: builtin::junie::tokens(),
            design: builtin::junie::design(),
            recipes: builtin::default_recipes(),
            capability: Capability {
                color: ColorLevel::TrueColor,
            },
        }
    }

    /// The distinct non-Junie theme (§11.7): light, indigo, square, compact.
    pub fn paper() -> Theme {
        builtin::paper::theme()
    }

    /// A theme from colour tokens: design tokens and recipe defaults are
    /// filled in, and every `Color::Reset` token is derived (§11.2).
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
        }
    }

    /// Partial override with safe derivation.
    pub fn builder(self) -> ThemeBuilder {
        ThemeBuilder::from_theme(self)
    }

    /// A theme-level override applied to every variant of `f` (precedence 4).
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

    /// A theme-level override applied to one variant of `f` (precedence 4).
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

    /// Define (or extend) a variant delta of `f` (precedence 2).
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

    /// Define (or extend) a family's base recipe (precedence 1).
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

    /// The background colour of a surface.
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

    /// One plane up: `Field → FieldHover`; on the ladder `min(level + 1, last)`.
    /// Index arithmetic, never colour equality (§10).
    pub fn raise(&self, s: Surface) -> Surface {
        match s {
            Surface::Field | Surface::FieldHover => Surface::FieldHover,
            ladder => Surface::from_level(ladder.level().unwrap_or(0).saturating_add(1)),
        }
    }

    /// Resolve a style without a cache or an overlay stack.
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

    /// Sizes, glyphs and alignment for a part, with no colour binding and no
    /// overlay stack — an `update` has neither a surface nor a draw-time
    /// scope. This is the sizing path for `Cx`-phase arithmetic: `Form`'s
    /// field height (§15.1 F4) and `Dialog::layer` (Adjudication N1).
    ///
    /// It runs the same §11.3 `accumulate` as [`Theme::resolve`] and reads the
    /// same slots, so the two cannot disagree about a size.
    pub fn metrics(
        &self,
        f: Family,
        v: Variant,
        p: crate::id::Part,
        s: crate::response::StateFlags,
    ) -> PartMetrics {
        resolve::metrics_of(&resolve::accumulate(self, f, v, p, s, &[]))
    }

    /// A stable fingerprint of the whole theme (tests: byte-identical after
    /// a scoped render).
    ///
    /// It formats the whole theme and hashes the text, which allocates; that
    /// is acceptable for a test assertion and is why it is **behind the
    /// `testing` feature** rather than on the release surface (MI-12). A
    /// structural hash would need `Hash` on `StylePatch`, `PartRecipe` and
    /// `Recipes`, which are user-constructed data records.
    #[cfg(feature = "testing")]
    pub fn fingerprint(&self) -> u64 {
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

    /// `Theme::metrics` is the sizing path a component uses in `update`,
    /// where there is no `Surface`: the glyph and size it computes there are
    /// exactly the ones `draw` resolves and paints with (Adjudication N2).
    #[test]
    fn metrics_is_the_sizing_path_for_update() {
        let mut t = Theme::junie();
        // a themed size and glyph, the shape `Form`'s field height and
        // `Dialog::measured_height` read
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
            // the draw phase resolves the same numbers on every surface
            for s in [Surface::Canvas, Surface::Field, Surface::Overlay] {
                let r = t.resolve(Family::FIELD, Variant::DEFAULT, Part::FIELD, st, s);
                assert_eq!((r.size, r.glyph, r.align), (m.size, m.glyph, m.align));
            }
        }
    }

    #[test]
    fn raise_is_ladder_index_arithmetic_not_colour_equality() {
        let mut t = Theme::junie();
        // duplicate plane colours must not confuse raising
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
        // derived, pinned
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
