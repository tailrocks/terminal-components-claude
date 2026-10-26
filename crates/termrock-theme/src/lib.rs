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
pub use palettes::{CapabilityPalettes, baseline_faint, baseline_muted};
pub use patch::{Slot, StateRule, StylePatch};
pub use recipe::{
    Family, ManagerPresentationProfile, Overlay, OverlayRule, PartEdit, PartMap, PartRecipe,
    Recipe, RecipeEdit, Recipes, Variant,
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum CursorStyle {
    #[default]
    Bar,

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

use self::recipe::{AuthoredDeclaration, DeclarationKind, GlobalOverride};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Theme {
    pub color: ColorTokens,
    pub design: DesignTokens,
    pub recipes: Recipes,
    pub capability: Capability,
    pub capability_palettes: Option<std::sync::Arc<CapabilityPalettes>>,
}

impl Theme {
    #[must_use]
    pub fn manager_presentation_profile(&self) -> ManagerPresentationProfile {
        self.recipes.manager_presentation_profile()
    }

    pub fn junie() -> Theme {
        let mut theme = Theme {
            color: builtin::junie::tokens(),
            design: builtin::junie::design(),
            recipes: builtin::junie::recipes(),
            capability: Capability {
                color: ColorLevel::TrueColor,
            },
            capability_palettes: Some(std::sync::Arc::new(palettes::junie())),
        };
        theme
            .recipes
            .set_manager_presentation_profile(ManagerPresentationProfile::Junie);
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
            theme.define_builtin_family_for_level(level, Family::SPLIT, |recipe| {
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
                    theme.define_builtin_variant_for_level(
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
            theme.define_builtin_variant_for_level(level, Family::MENU, Variant::DANGER, |recipe| {
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

        [
            (TrueColor, Color::from_u32(0x002b_8632)),
            (Ansi256, Color::Indexed(65)),
            (Ansi16, Color::LightGreen),
            (Mono, Color::White),
        ]
        .into_iter()
        .fold(profiled_theme, |theme, (level, pressed)| {
            theme.define_builtin_variant_for_level(
                level,
                Family::BUTTON,
                Variant::PRIMARY,
                |recipe| {
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
                },
            )
        })
    }

    pub fn paper() -> Theme {
        let mut theme = builtin::paper::theme();
        theme
            .recipes
            .set_manager_presentation_profile(ManagerPresentationProfile::Paper);
        theme
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
        if let Some(declaration) =
            AuthoredDeclaration::from_edit(f, None, DeclarationKind::FamilyOverride, None, None, &e)
        {
            if let Some(v) = declaration.default_variant {
                self.recipes.get_mut(f).default_variant = v;
            }
            self.recipes.push_override(GlobalOverride {
                family: f,
                variant: None,
                parts: declaration.parts.clone(),
            });
            self.recipes.push_declaration(declaration);
        }
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
        if let Some(declaration) = AuthoredDeclaration::from_edit(
            f,
            Some(v),
            DeclarationKind::VariantOverride,
            None,
            None,
            &e,
        ) {
            if let Some(dv) = declaration.default_variant {
                self.recipes.get_mut(f).default_variant = dv;
            }
            self.recipes.push_override(GlobalOverride {
                family: f,
                variant: Some(v),
                parts: declaration.parts.clone(),
            });
            self.recipes.push_declaration(declaration);
        }
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
        if let Some(declaration) =
            AuthoredDeclaration::from_edit(f, Some(v), DeclarationKind::Variant, None, None, &e)
        {
            let recipe = self.recipes.get_mut(f);
            if let Some(dv) = declaration.default_variant {
                recipe.default_variant = dv;
            }
            let target = recipe.variant_mut(v);
            for (p, part) in declaration.parts.iter() {
                target.entry(p).merge_from(part);
            }
            self.recipes.push_declaration(declaration);
        }
        self
    }

    #[must_use]
    pub fn define_family(mut self, f: Family, edit: impl FnOnce(&mut RecipeEdit)) -> Theme {
        let mut e = RecipeEdit::default();
        edit(&mut e);
        if let Some(declaration) =
            AuthoredDeclaration::from_edit(f, None, DeclarationKind::Family, None, None, &e)
        {
            let recipe = self.recipes.get_mut(f);
            if let Some(dv) = declaration.default_variant {
                recipe.default_variant = dv;
            }
            for (p, part) in declaration.parts.iter() {
                recipe.parts.entry(p).merge_from(part);
            }
            self.recipes.push_declaration(declaration);
        }
        self
    }

    #[must_use]
    pub fn define_family_for_level(
        self,
        level: ColorLevel,
        family: Family,
        edit: impl FnOnce(&mut RecipeEdit),
    ) -> Theme {
        self.define_profile_for_level(level, family, None, None, edit, true)
    }

    #[must_use]
    pub fn define_variant_for_level(
        self,
        level: ColorLevel,
        family: Family,
        variant: Variant,
        edit: impl FnOnce(&mut RecipeEdit),
    ) -> Theme {
        self.define_profile_for_level(level, family, Some(variant), None, edit, true)
    }

    #[must_use]
    pub fn define_variant_for_level_on_surface(
        self,
        level: ColorLevel,
        family: Family,
        variant: Variant,
        surface: Surface,
        edit: impl FnOnce(&mut RecipeEdit),
    ) -> Theme {
        self.define_profile_for_level(level, family, Some(variant), Some(surface), edit, true)
    }

    fn define_builtin_family_for_level(
        self,
        level: ColorLevel,
        family: Family,
        edit: impl FnOnce(&mut RecipeEdit),
    ) -> Theme {
        self.define_profile_for_level(level, family, None, None, edit, false)
    }

    fn define_builtin_variant_for_level(
        self,
        level: ColorLevel,
        family: Family,
        variant: Variant,
        edit: impl FnOnce(&mut RecipeEdit),
    ) -> Theme {
        self.define_profile_for_level(level, family, Some(variant), None, edit, false)
    }

    fn define_profile_for_level(
        mut self,
        level: ColorLevel,
        family: Family,
        variant: Option<Variant>,
        surface: Option<Surface>,
        edit: impl FnOnce(&mut RecipeEdit),
        consumer_authored: bool,
    ) -> Theme {
        let mut e = RecipeEdit::default();
        edit(&mut e);
        let declaration = AuthoredDeclaration::from_edit(
            family,
            variant,
            DeclarationKind::LevelProfile,
            Some(level),
            surface,
            &e,
        );
        let parts = e.normalized_parts();
        if parts.is_empty() {
            if consumer_authored && let Some(declaration) = declaration {
                self.recipes.push_declaration(declaration);
            }
            return self;
        }
        let seed_source_tokens = self.color;
        let seed_level_tokens = self.downgrade(level).color;
        self.recipes.define_level_family(
            recipe::LevelProfileKey {
                level,
                family,
                variant,
                surface,
            },
            parts,
            seed_source_tokens,
            seed_level_tokens,
        );
        if consumer_authored && let Some(declaration) = declaration {
            self.recipes.push_declaration(declaration);
        }
        self
    }

    pub fn backdrop_colors(&self, fg: Color, bg: Color) -> (Color, Color) {
        let (fg, bg, _) = self.backdrop_cell(fg, bg);
        (fg, bg)
    }

    pub fn backdrop_dim_level(&self) -> bool {
        matches!(self.capability.color, ColorLevel::Ansi16 | ColorLevel::Mono)
    }

    pub fn backdrop_cell(&self, fg: Color, bg: Color) -> (Color, Color, bool) {
        let (mut mapped_fg, mapped_bg, blank) = self.map_backdrop(Some(fg), Some(bg), true);
        if (!blank && mapped_fg == mapped_bg) || self.backdrop_dim_level() {
            mapped_fg = self.backdrop_collision_ink(mapped_bg);
        }
        (mapped_fg, mapped_bg, blank)
    }

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
}
