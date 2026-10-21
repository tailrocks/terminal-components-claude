use ratatui_core::style::{Color, Modifier, Style};

use super::Theme;
use super::downgrade::downgrade_color;
use super::glyph::GlyphRole;
use super::patch::{Slot, StylePatch};
use super::recipe::{Family, Overlay, Variant};
use super::role::{Align, FgStep, MeterRole, Role, Surface, SyntaxRole};
use crate::id::{Part, fnv1a};
use crate::response::StateFlags;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PaintStyle {
    style: Style,
    pub fg_role: Option<(Role, Surface)>,
    pub bg_role: Option<(Role, Surface)>,
    underline_role: Option<(Role, Surface)>,
    underline_cleared: bool,
}

impl PaintStyle {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            style: Style::new(),
            fg_role: None,
            bg_role: None,
            underline_role: None,
            underline_cleared: false,
        }
    }

    pub fn bound(style: Style, fg: Option<Role>, bg: Option<Role>, surface: Surface) -> Self {
        Self {
            style,
            fg_role: fg.filter(|_| style.fg.is_some()).map(|r| (r, surface)),
            bg_role: bg.filter(|_| style.bg.is_some()).map(|r| (r, surface)),
            underline_role: None,
            underline_cleared: false,
        }
    }

    #[must_use]
    pub const fn underline_role(&self) -> Option<(Role, Surface)> {
        self.underline_role
    }

    /// Whether the final canonical underline channel explicitly clears its
    /// inherited value.
    #[must_use]
    pub const fn underline_cleared(&self) -> bool {
        self.underline_cleared
    }

    #[must_use]
    pub(crate) const fn with_underline_role(mut self, role: Option<(Role, Surface)>) -> Self {
        self.underline_role = role;
        self
    }

    #[must_use]
    const fn with_underline_cleared(mut self, cleared: bool) -> Self {
        self.underline_cleared = cleared;
        self
    }

    #[must_use]
    pub const fn as_style(&self) -> &Style {
        &self.style
    }

    #[must_use]
    pub const fn into_style(self) -> Style {
        self.style
    }

    #[must_use]
    pub fn fg(mut self, color: Color) -> Self {
        self.style = self.style.fg(color);
        self.fg_role = None;
        self
    }

    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.style = self.style.bg(color);
        self.bg_role = None;
        self
    }

    #[must_use]
    pub fn with_fg_from(mut self, other: Self) -> Self {
        self.style.fg = other.style.fg;
        self.fg_role = other.fg_role;
        self
    }

    #[must_use]
    pub fn with_bg_from(mut self, other: Self) -> Self {
        self.style.bg = other.style.bg;
        self.bg_role = other.bg_role;
        self
    }

    #[must_use]
    pub fn with_fg_from_bg(mut self, other: Self) -> Self {
        self.style.fg = other.style.bg;
        self.fg_role = other.bg_role;
        self
    }

    #[must_use]
    pub fn with_bg_from_fg(mut self, other: Self) -> Self {
        self.style.bg = other.style.fg;
        self.bg_role = other.fg_role;
        self
    }

    #[must_use]
    pub fn underline_color(mut self, color: Color) -> Self {
        self.style = self.style.underline_color(color);
        self.underline_role = None;
        self.underline_cleared = false;
        self
    }

    #[must_use]
    pub fn add_modifier(mut self, modifier: ratatui_core::style::Modifier) -> Self {
        self.style = self.style.add_modifier(modifier);
        self
    }

    #[must_use]
    pub fn remove_modifier(mut self, modifier: ratatui_core::style::Modifier) -> Self {
        self.style = self.style.remove_modifier(modifier);
        self
    }

    #[must_use]
    pub fn patch(mut self, other: impl Into<Self>) -> Self {
        let other = other.into();
        if other.style.fg.is_some() {
            self.fg_role = other.fg_role;
        }
        if other.style.bg.is_some() {
            self.bg_role = other.bg_role;
        }
        if other.style.underline_color.is_some() || other.underline_cleared {
            self.underline_role = other.underline_role;
            self.underline_cleared = other.underline_cleared;
        }
        self.style = self.style.patch(other.style);
        self
    }
}

impl From<Style> for PaintStyle {
    fn from(style: Style) -> Self {
        Self {
            style,
            fg_role: None,
            bg_role: None,
            underline_role: None,
            underline_cleared: false,
        }
    }
}

impl std::ops::Deref for PaintStyle {
    type Target = Style;
    fn deref(&self) -> &Style {
        &self.style
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StyleDefaults<'a> {
    pub base: StylePatch,
    pub mono: &'a [super::MonoRule],
}

impl<'a> StyleDefaults<'a> {
    #[must_use]
    pub const fn new(base: StylePatch) -> Self {
        Self { base, mono: &[] }
    }

    #[must_use]
    pub const fn mono(mut self, rules: &'a [super::MonoRule]) -> Self {
        self.mono = rules;
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Resolved {
    pub style: PaintStyle,
    pub glyph: Slot<GlyphRole>,
    pub size: Option<u16>,
    pub align: Option<Align>,
}

/// A resolved style together with the explicit theme, overlay, and instance
/// channels that overrode its defaults.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ResolvedStyle {
    pub resolved: Resolved,
    pub explicit: StylePatch,
}

impl Resolved {
    #[must_use]
    pub fn over(self, inherited: impl Into<PaintStyle>) -> PaintStyle {
        inherited.into().patch(self.style)
    }

    #[must_use]
    pub const fn metrics(self) -> PartMetrics {
        PartMetrics {
            glyph: self.glyph,
            size: self.size,
            align: self.align,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PartMetrics {
    pub glyph: Slot<GlyphRole>,
    pub size: Option<u16>,
    pub align: Option<Align>,
}

impl From<Resolved> for PartMetrics {
    fn from(r: Resolved) -> Self {
        r.metrics()
    }
}

pub(crate) fn metrics_of(acc: &StylePatch) -> PartMetrics {
    PartMetrics {
        glyph: acc.glyph,
        size: acc.size.get(),
        align: acc.align.get(),
    }
}

pub(crate) fn accumulate(
    theme: &Theme,
    f: Family,
    v: Variant,
    p: Part,
    live: StateFlags,
    overlays: &[Overlay],
) -> StylePatch {
    accumulate_profiled(theme, f, v, p, live, overlays, Surface::Canvas).0
}

fn accumulate_profiled(
    theme: &Theme,
    family: Family,
    requested_variant: Variant,
    part: Part,
    live: StateFlags,
    overlays: &[Overlay],
    surface: Surface,
) -> (StylePatch, ProfiledStyle) {
    let (defaults, variant, mut profiled) =
        accumulate_defaults(theme, family, requested_variant, part, live, surface);
    let explicit = apply_explicit(
        theme,
        (family, variant, part),
        live,
        overlays,
        StylePatch::new(),
    );
    profiled.suppress(explicit);
    (defaults.merge(explicit), profiled)
}

fn accumulate_defaults(
    theme: &Theme,
    f: Family,
    v: Variant,
    p: Part,
    live: StateFlags,
    surface: Surface,
) -> (StylePatch, Variant, ProfiledStyle) {
    let recipes = &theme.recipes;

    let (mut acc, variant) = apply_recipe(recipes.get_or_neutral(f), v, p, live, StylePatch::new());
    let (profiled_acc, exact_profile) = apply_level_profiles(
        theme,
        ProfileContext {
            family: f,
            requested_variant: v,
            variant,
            part: p,
            live,
            surface,
        },
        acc,
    );
    acc = profiled_acc;
    let mut profiled = ProfiledStyle::from_patch(exact_profile);

    if theme.capability.color == super::ColorLevel::Mono {
        let before_mono = acc;
        acc = super::downgrade::apply_mono_fallback(acc, recipes, f, p, live);
        let targeted = authored_mono_override(theme, f, p, live, None);
        profiled.restore_profiled_channels(before_mono, &mut acc, targeted);
        profiled.retain_unchanged(before_mono, acc);
        profiled.suppress_targeted_exact_channels(targeted);
    }
    (acc, variant, profiled)
}

#[derive(Clone, Copy, Debug, Default)]
struct ProfiledStyle {
    fg: bool,
    bg: bool,
    underline: bool,
    preserve_fg: bool,
    preserve_bg: bool,
    preserve_underline: bool,
    preserve_modifiers: bool,
    preserve_glyph: bool,
    preserve_size: bool,
    preserve_align: bool,
}

impl ProfiledStyle {
    fn from_patch(patch: StylePatch) -> Self {
        Self {
            fg: matches!(patch.fg, Slot::Set(Role::Custom(_))),
            bg: matches!(patch.bg, Slot::Set(Role::Custom(_))),
            underline: matches!(patch.underline, Slot::Set(Role::Custom(_))),
            preserve_fg: patch.fg.speaks(),
            preserve_bg: patch.bg.speaks(),
            preserve_underline: patch.underline.speaks(),
            preserve_modifiers: !patch.add.is_empty() || !patch.remove.is_empty(),
            preserve_glyph: patch.glyph.speaks(),
            preserve_size: patch.size.speaks(),
            preserve_align: patch.align.speaks(),
        }
    }

    fn restore_profiled_channels(
        &self,
        before: StylePatch,
        after: &mut StylePatch,
        targeted: StylePatch,
    ) {
        if self.preserve_fg && !targeted.fg.speaks() {
            after.fg = before.fg;
        }
        if self.preserve_bg && !targeted.bg.speaks() {
            after.bg = before.bg;
        }
        if self.preserve_underline && !targeted.underline.speaks() {
            after.underline = before.underline;
        }
        if self.preserve_modifiers && targeted.add.is_empty() && targeted.remove.is_empty() {
            after.add = before.add;
            after.remove = before.remove;
        }
        if self.preserve_glyph && !targeted.glyph.speaks() {
            after.glyph = before.glyph;
        }
        if self.preserve_size && !targeted.size.speaks() {
            after.size = before.size;
        }
        if self.preserve_align && !targeted.align.speaks() {
            after.align = before.align;
        }
    }

    fn retain_unchanged(&mut self, before: StylePatch, after: StylePatch) {
        if before.fg != after.fg {
            self.fg = false;
        }
        if before.bg != after.bg {
            self.bg = false;
        }
        if before.underline != after.underline {
            self.underline = false;
        }
    }

    fn suppress_targeted_exact_channels(&mut self, targeted: StylePatch) {
        // Ownership matters even when an authored Mono rule writes the same
        // role as the active exact profile. In that case retain_unchanged
        // cannot distinguish the authored value from the profile value, but
        // the authored channel still belongs to ordinary Mono binding.
        if targeted.fg.speaks() {
            self.fg = false;
        }
        if targeted.bg.speaks() {
            self.bg = false;
        }
        if targeted.underline.speaks() {
            self.underline = false;
        }
    }

    fn suppress(&mut self, patch: StylePatch) {
        if !matches!(patch.fg, Slot::Inherit) {
            self.fg = false;
        }
        if !matches!(patch.bg, Slot::Inherit) {
            self.bg = false;
        }
        if !matches!(patch.underline, Slot::Inherit) {
            self.underline = false;
        }
    }
}

#[derive(Clone, Copy)]
struct ProfileContext {
    family: Family,
    requested_variant: Variant,
    variant: Variant,
    part: Part,
    live: StateFlags,
    surface: Surface,
}

fn apply_level_profiles(
    theme: &Theme,
    context: ProfileContext,
    mut acc: StylePatch,
) -> (StylePatch, StylePatch) {
    let mut combined = StylePatch::new();
    for (profile_variant, profile_surface) in [
        (None, None),
        (Some(context.variant), None),
        (Some(context.variant), Some(context.surface)),
    ] {
        let Some(profile) = theme.recipes.level_family(
            theme.capability.color,
            context.family,
            profile_variant,
            profile_surface,
        ) else {
            continue;
        };
        let (next, applied) = apply_level_profile(theme, profile, context, acc);
        acc = next;
        combined = combined.merge(applied);
    }
    (acc, combined)
}

fn apply_level_profile(
    theme: &Theme,
    profile: &super::recipe::LevelFamilyRecipe,
    context: ProfileContext,
    mut acc: StylePatch,
) -> (StylePatch, StylePatch) {
    let Some(level_part) = profile.parts.get(context.part) else {
        return (acc, StylePatch::new());
    };
    let current_recipe = theme.recipes.get_or_neutral(profile.family);
    if let Some(profile_variant) = profile.variant {
        if profile_variant != context.variant {
            return (acc, StylePatch::new());
        }
    } else {
        let (_, seed_variant) = apply_recipe(
            &profile.seed,
            context.requested_variant,
            context.part,
            StateFlags::empty(),
            StylePatch::new(),
        );
        if seed_variant != context.variant {
            return (acc, StylePatch::new());
        }
    }

    let seed_family = profile.seed.parts.get(context.part);
    let current_family = current_recipe.parts.get(context.part);
    let seed_delta = profile
        .seed
        .variant(context.variant)
        .and_then(|map| map.get(context.part));
    let current_delta = current_recipe
        .variant(context.variant)
        .and_then(|map| map.get(context.part));
    let (seed_effective, _) = apply_recipe(
        &profile.seed,
        profile.variant.unwrap_or(context.requested_variant),
        context.part,
        context.live,
        StylePatch::new(),
    );
    let eligible = |channel| {
        part_channel_unchanged(current_family, seed_family, context.live, channel)
            && part_channel_unchanged(current_delta, seed_delta, context.live, channel)
            && profile_covers_active_states(
                current_family,
                seed_family,
                level_part,
                context.live,
                channel,
            )
            && profile_covers_active_states(
                current_delta,
                seed_delta,
                level_part,
                context.live,
                channel,
            )
            && profile_token_dependencies_unchanged(
                theme,
                profile.seed_source_tokens,
                profile.seed_level_tokens,
                seed_effective,
                context.surface,
                channel,
            )
    };

    let mut profile_patch = level_part.apply(StylePatch::new(), context.live);
    if !eligible(PatchChannel::Fg) {
        profile_patch.fg = Slot::Inherit;
    }
    if !eligible(PatchChannel::Bg) {
        profile_patch.bg = Slot::Inherit;
    }
    if !eligible(PatchChannel::Underline) {
        profile_patch.underline = Slot::Inherit;
    }
    if !eligible(PatchChannel::Modifiers) {
        profile_patch.add = Modifier::empty();
        profile_patch.remove = Modifier::empty();
    }
    if !eligible(PatchChannel::Glyph) {
        profile_patch.glyph = Slot::Inherit;
    }
    if !eligible(PatchChannel::Size) {
        profile_patch.size = Slot::Inherit;
    }
    if !eligible(PatchChannel::Align) {
        profile_patch.align = Slot::Inherit;
    }
    acc = acc.merge(profile_patch);
    (acc, profile_patch)
}

fn authored_mono_override(
    theme: &Theme,
    family: Family,
    part: Part,
    live: StateFlags,
    authored: Option<&[super::MonoRule]>,
) -> StylePatch {
    theme
        .recipes
        .mono_rules(family)
        .or(authored)
        .or_else(|| Some(super::downgrade::builtin_family_mono_rules(family)))
        .unwrap_or_default()
        .iter()
        .filter(|(rule_part, when, _)| *rule_part == part && live.contains(*when))
        .fold(StylePatch::new(), |patch, (_, _, rule)| patch.merge(*rule))
}

fn profile_token_dependencies_unchanged(
    theme: &Theme,
    source: super::ColorTokens,
    projected: super::ColorTokens,
    seed: StylePatch,
    surface: Surface,
    channel: PatchChannel,
) -> bool {
    let role = match channel {
        PatchChannel::Fg => seed.fg,
        PatchChannel::Bg => seed.bg,
        PatchChannel::Underline => seed.underline,
        PatchChannel::Modifiers
        | PatchChannel::Glyph
        | PatchChannel::Size
        | PatchChannel::Align => {
            return true;
        }
    };
    let Slot::Set(role) = role else {
        return true;
    };
    let Some(source_color) = profile_role_color(source, role, surface) else {
        return true;
    };
    let Some(current) = profile_role_color(theme.color, role, surface) else {
        return true;
    };
    current == source_color || profile_role_color(projected, role, surface) == Some(current)
}

fn profile_role_color(tokens: super::ColorTokens, role: Role, surface: Surface) -> Option<Color> {
    let bg = |surface| match surface {
        Surface::Field => tokens.field,
        Surface::FieldHover => tokens.field_hover,
        ladder => tokens
            .surfaces
            .get(ladder.level().unwrap_or(0))
            .copied()
            .unwrap_or(Color::Reset),
    };
    let raised = |surface| match surface {
        Surface::Field | Surface::FieldHover => Surface::FieldHover,
        Surface::Canvas => Surface::Surface,
        Surface::Surface => Surface::Elevated,
        Surface::Elevated => Surface::Overlay,
        Surface::Overlay | Surface::Popover => Surface::Popover,
    };
    let hover = match surface {
        Surface::Canvas => Surface::Elevated,
        Surface::Surface | Surface::Elevated => Surface::Overlay,
        Surface::Field => Surface::FieldHover,
        Surface::Overlay | Surface::Popover | Surface::FieldHover => Surface::Popover,
    };
    let color = match role {
        Role::CurrentSurface => bg(surface),
        Role::RaisedSurface => bg(raised(surface)),
        Role::HoverSurface => bg(hover),
        Role::Surface(surface) => bg(surface),
        Role::Fg(step) => tokens.fg.get(step.index()).copied().unwrap_or(Color::Reset),
        Role::OnAccent => tokens.on_accent,
        Role::OnDanger => tokens.on_danger,
        Role::OnSurfaceInverse => tokens.on_surface_inverse,
        Role::BorderSubtle => tokens.border_subtle,
        Role::BorderStrong => tokens.border_strong,
        Role::Accent => tokens.accent,
        Role::AccentHover => tokens.accent_hover,
        Role::AccentPressed => tokens.accent_pressed,
        Role::AccentTint => tokens.accent_tint,
        Role::Focus => tokens.focus,
        Role::FocusRing => tokens.focus_ring,
        Role::SelectionBg => tokens.selection_bg,
        Role::SelectionFg => tokens.selection_fg,
        Role::HighlightBg => tokens.highlight_bg,
        Role::HighlightFg => tokens.highlight_fg,
        Role::HighlightDangerBg => tokens.highlight_danger_bg,
        Role::HighlightDangerFg => tokens.highlight_danger_fg,
        Role::BackdropFg => tokens.backdrop_fg,
        Role::BackdropBg => tokens.backdrop_bg,
        Role::Danger => tokens.danger,
        Role::DangerSoft => tokens.danger_soft,
        Role::DangerTint => tokens.danger_tint,
        Role::Warning => tokens.warning,
        Role::WarningTint => tokens.warning_tint,
        Role::Success => tokens.success,
        Role::Info => tokens.info,
        Role::DisabledFg => tokens.disabled_fg,
        Role::DisabledBg => tokens.disabled_bg,
        Role::ReadOnlyFg => tokens.read_only_fg,
        Role::Syntax(role) => match role {
            SyntaxRole::Keyword => tokens.syntax.keyword,
            SyntaxRole::Ident => tokens.syntax.ident,
            SyntaxRole::Str => tokens.syntax.string,
            SyntaxRole::Number => tokens.syntax.number,
            SyntaxRole::Operator => tokens.syntax.operator,
            SyntaxRole::Punct => tokens.syntax.punct,
            SyntaxRole::Comment => tokens.syntax.comment,
            SyntaxRole::Plain => tokens.syntax.plain,
            SyntaxRole::TypeName => tokens.syntax.type_name,
            SyntaxRole::Function => tokens.syntax.function,
            SyntaxRole::Constant => tokens.syntax.constant,
            SyntaxRole::Invalid => tokens.syntax.invalid,
            SyntaxRole::Deprecated => tokens.syntax.deprecated,
            SyntaxRole::MatchBg => tokens.syntax.match_bg,
            SyntaxRole::MatchCurrentBg => tokens.syntax.match_current_bg,
            SyntaxRole::BracketMatch => tokens.syntax.bracket_match,
            SyntaxRole::DiagError => tokens.syntax.diagnostic_error,
            SyntaxRole::DiagWarning => tokens.syntax.diagnostic_warning,
            SyntaxRole::DiagInfo => tokens.syntax.diagnostic_info,
        },
        Role::Meter(role) => match role {
            MeterRole::Low => tokens.meter.low,
            MeterRole::Medium => tokens.meter.medium,
            MeterRole::High => tokens.meter.high,
            MeterRole::Track => tokens.meter.track,
            MeterRole::FillRest => match tokens.meter.fill_rest {
                super::MeterFillRest::Color(color) => color,
                super::MeterFillRest::ReferenceLift => {
                    let plane = bg(surface);
                    if plane == bg(Surface::Canvas) {
                        bg(Surface::Elevated)
                    } else if plane == bg(Surface::Surface) || plane == bg(Surface::Elevated) {
                        bg(Surface::Overlay)
                    } else if plane == bg(Surface::Field) {
                        bg(Surface::FieldHover)
                    } else {
                        bg(Surface::Popover)
                    }
                }
            },
            MeterRole::Stale => tokens.meter.stale,
            MeterRole::Unknown => tokens.meter.unknown,
            MeterRole::Series(index) => tokens
                .meter
                .series
                .get(usize::from(index) % 6)
                .copied()
                .unwrap_or(Color::Reset),
        },
        Role::Custom(_) => return None,
    };
    Some(color)
}

#[derive(Clone, Copy)]
enum PatchChannel {
    Fg,
    Bg,
    Underline,
    Modifiers,
    Glyph,
    Size,
    Align,
}

fn same_patch_channel(a: StylePatch, b: StylePatch, channel: PatchChannel) -> bool {
    match channel {
        PatchChannel::Fg => a.fg == b.fg,
        PatchChannel::Bg => a.bg == b.bg,
        PatchChannel::Underline => a.underline == b.underline,
        PatchChannel::Modifiers => a.add == b.add && a.remove == b.remove,
        PatchChannel::Glyph => a.glyph == b.glyph,
        PatchChannel::Size => a.size == b.size,
        PatchChannel::Align => a.align == b.align,
    }
}

fn patch_channel_speaks(patch: StylePatch, channel: PatchChannel) -> bool {
    match channel {
        PatchChannel::Fg => patch.fg.speaks(),
        PatchChannel::Bg => patch.bg.speaks(),
        PatchChannel::Underline => patch.underline.speaks(),
        PatchChannel::Modifiers => !patch.add.is_empty() || !patch.remove.is_empty(),
        PatchChannel::Glyph => patch.glyph.speaks(),
        PatchChannel::Size => patch.size.speaks(),
        PatchChannel::Align => patch.align.speaks(),
    }
}

fn profile_covers_active_states(
    current: Option<&super::PartRecipe>,
    seed: Option<&super::PartRecipe>,
    profile: &super::PartRecipe,
    live: StateFlags,
    channel: PatchChannel,
) -> bool {
    let Some(current) = current else {
        return true;
    };
    current
        .states
        .iter()
        .filter(|rule| rule.matches(live) && patch_channel_speaks(rule.patch, channel))
        .all(|rule| {
            let baseline = seed.and_then(|seed| {
                seed.states
                    .iter()
                    .find(|candidate| candidate.when == rule.when)
            });
            baseline.is_some_and(|baseline| same_patch_channel(rule.patch, baseline.patch, channel))
                && profile.states.iter().any(|candidate| {
                    candidate.when == rule.when && patch_channel_speaks(candidate.patch, channel)
                })
        })
}

fn part_channel_unchanged(
    current: Option<&super::PartRecipe>,
    seed: Option<&super::PartRecipe>,
    live: StateFlags,
    channel: PatchChannel,
) -> bool {
    let current_base = current.map_or(StylePatch::new(), |recipe| recipe.base);
    let seed_base = seed.map_or(StylePatch::new(), |recipe| recipe.base);
    if !same_patch_channel(current_base, seed_base, channel) {
        return false;
    }
    let current_applied = current.map_or(StylePatch::new(), |recipe| {
        recipe.apply(StylePatch::new(), live)
    });
    let seed_applied = seed.map_or(StylePatch::new(), |recipe| {
        recipe.apply(StylePatch::new(), live)
    });
    same_patch_channel(current_applied, seed_applied, channel)
}

fn apply_recipe(
    recipe: &super::Recipe,
    requested: Variant,
    part: Part,
    live: StateFlags,
    mut acc: StylePatch,
) -> (StylePatch, Variant) {
    let variant = if requested == Variant::DEFAULT {
        recipe.default_variant
    } else {
        requested
    };
    let family = recipe.parts.get(part);
    let delta = recipe.variant(variant).and_then(|m| m.get(part));
    if let Some(part) = family {
        acc = part.apply_base(acc);
    }
    if let Some(part) = delta {
        acc = part.apply_base(acc);
    }
    acc = super::recipe::merge_states(
        acc,
        family.map_or(&[][..], |p| &p.states),
        delta.map_or(&[][..], |p| &p.states),
        live,
    );
    (acc, variant)
}

pub fn bind_defaults(
    theme: &Theme,
    (family, variant, part): (Family, Variant, Part),
    flags: StateFlags,
    overlays: &[Overlay],
    surface: Surface,
    defaults: StyleDefaults<'_>,
    local: Option<&StylePatch>,
) -> Resolved {
    let requested_variant = variant;
    let (mut acc, variant) = match theme.recipes.get(family) {
        Some(recipe) => apply_recipe(recipe, requested_variant, part, flags, defaults.base),
        None => (defaults.base, requested_variant),
    };
    let (profiled, exact_profile_patch) = apply_level_profiles(
        theme,
        ProfileContext {
            family,
            requested_variant,
            variant,
            part,
            live: flags,
            surface,
        },
        acc,
    );
    acc = profiled;
    let mut exact_profile = ProfiledStyle::from_patch(exact_profile_patch);
    if theme.capability.color == super::ColorLevel::Mono {
        let before_mono = acc;
        acc = super::downgrade::apply_mono_with_defaults(
            acc,
            &theme.recipes,
            family,
            part,
            flags,
            Some(defaults.mono),
        );
        let targeted = authored_mono_override(theme, family, part, flags, Some(defaults.mono));
        exact_profile.restore_profiled_channels(before_mono, &mut acc, targeted);
        exact_profile.retain_unchanged(before_mono, acc);
        exact_profile.suppress_targeted_exact_channels(targeted);
    }
    let explicit = apply_explicit(
        theme,
        (family, variant, part),
        flags,
        overlays,
        StylePatch::new(),
    );
    exact_profile.suppress(explicit);
    acc = acc.merge(explicit);
    bind_with_profile(theme, acc, local, surface, exact_profile)
}

pub fn bind_defaults_with_explicit(
    theme: &Theme,
    (family, variant, part): (Family, Variant, Part),
    flags: StateFlags,
    overlays: &[Overlay],
    surface: Surface,
    defaults: StyleDefaults<'_>,
    local: Option<&StylePatch>,
) -> ResolvedStyle {
    let requested_variant = variant;
    let (mut acc, variant) = match theme.recipes.get(family) {
        Some(recipe) => apply_recipe(recipe, requested_variant, part, flags, defaults.base),
        None => (defaults.base, requested_variant),
    };
    let (profiled, exact_profile_patch) = apply_level_profiles(
        theme,
        ProfileContext {
            family,
            requested_variant,
            variant,
            part,
            live: flags,
            surface,
        },
        acc,
    );
    acc = profiled;
    let mut exact_profile = ProfiledStyle::from_patch(exact_profile_patch);
    if theme.capability.color == super::ColorLevel::Mono {
        let before_mono = acc;
        acc = super::downgrade::apply_mono_with_defaults(
            acc,
            &theme.recipes,
            family,
            part,
            flags,
            Some(defaults.mono),
        );
        let targeted = authored_mono_override(theme, family, part, flags, Some(defaults.mono));
        exact_profile.restore_profiled_channels(before_mono, &mut acc, targeted);
        exact_profile.retain_unchanged(before_mono, acc);
        exact_profile.suppress_targeted_exact_channels(targeted);
    }
    let explicit = apply_explicit(
        theme,
        (family, variant, part),
        flags,
        overlays,
        StylePatch::new(),
    );
    exact_profile.suppress(explicit);
    acc = acc.merge(explicit);
    if let Some(local) = local {
        exact_profile.suppress(*local);
    }
    let resolved = bind_with_profile(theme, acc, local, surface, exact_profile);
    ResolvedStyle {
        resolved,
        explicit: local.map_or(explicit, |local| explicit.merge(*local)),
    }
}

fn apply_explicit(
    theme: &Theme,
    (f, variant, p): (Family, Variant, Part),
    live: StateFlags,
    overlays: &[Overlay],
    mut acc: StylePatch,
) -> StylePatch {
    let recipes = &theme.recipes;

    for o in recipes.overrides() {
        if o.family != f {
            continue;
        }
        let applies = match o.variant {
            None => true,
            Some(ov) => ov == variant,
        };
        if applies && let Some(part) = o.parts.get(p) {
            acc = part.apply(acc, live);
        }
    }

    if !overlays.is_empty() {
        for ov in overlays {
            acc = ov.apply(acc, f, variant, p, live);
        }
    }
    acc
}

pub fn bind_inherited(
    theme: &Theme,
    (family, variant, part): (Family, Variant, Part),
    flags: StateFlags,
    overlays: &[Overlay],
    surface: Surface,
    inherited: PaintStyle,
) -> Resolved {
    let (defaults, variant, exact_profile) =
        accumulate_defaults(theme, family, variant, part, flags, surface);
    let explicit = apply_explicit(
        theme,
        (family, variant, part),
        flags,
        overlays,
        StylePatch::new(),
    );
    let base = bind_with_profile(theme, defaults, None, surface, exact_profile).style;
    let top = bind(theme, explicit, None, surface).style;
    let mut combined_profile = exact_profile;
    combined_profile.suppress(explicit);
    let mut result = bind_with_profile(
        theme,
        defaults.merge(explicit),
        None,
        surface,
        combined_profile,
    );
    let mut style = base.patch(inherited).patch(top);

    if !matches!(explicit.fg, Slot::Inherit) {
        style = style.with_fg_from(if top.fg.is_some() { top } else { inherited });
    }
    if !matches!(explicit.bg, Slot::Inherit) {
        style = style.with_bg_from(if top.bg.is_some() { top } else { inherited });
    }
    if !matches!(explicit.underline, Slot::Inherit) {
        style.style.underline_color = top.underline_color.or(inherited.underline_color);
    }
    result.style = style;
    result
}

fn reference_lift(theme: &Theme, surface: Surface) -> Color {
    let bg = theme.bg(surface);
    if bg == theme.bg(Surface::Canvas) {
        theme.bg(Surface::Elevated)
    } else if bg == theme.bg(Surface::Surface) || bg == theme.bg(Surface::Elevated) {
        theme.bg(Surface::Overlay)
    } else if bg == theme.bg(Surface::Field) {
        theme.bg(Surface::FieldHover)
    } else {
        theme.bg(Surface::Popover)
    }
}

pub fn bind_role(theme: &Theme, role: Role, surface: Surface) -> Option<Color> {
    let c = &theme.color;
    let fg = |i: usize| c.fg.get(i).copied().unwrap_or(Color::Reset);
    let color = match role {
        Role::CurrentSurface => theme.bg(surface),
        Role::RaisedSurface => theme.bg(theme.raise(surface)),
        Role::HoverSurface => theme.bg(match surface {
            Surface::Canvas => Surface::Elevated,
            Surface::Surface | Surface::Elevated => Surface::Overlay,
            Surface::Field => Surface::FieldHover,
            Surface::Overlay | Surface::Popover | Surface::FieldHover => Surface::Popover,
        }),
        Role::Surface(s) => theme.bg(s),
        Role::Fg(step) => fg(step.index()),
        Role::OnAccent => c.on_accent,
        Role::OnDanger => c.on_danger,
        Role::OnSurfaceInverse => c.on_surface_inverse,
        Role::BorderSubtle => c.border_subtle,
        Role::BorderStrong => c.border_strong,
        Role::Accent => c.accent,
        Role::AccentHover => c.accent_hover,
        Role::AccentPressed => c.accent_pressed,
        Role::AccentTint => c.accent_tint,
        Role::Focus => c.focus,
        Role::FocusRing => c.focus_ring,
        Role::SelectionBg => c.selection_bg,
        Role::SelectionFg => c.selection_fg,
        Role::HighlightBg => c.highlight_bg,
        Role::HighlightFg => c.highlight_fg,
        Role::HighlightDangerBg => c.highlight_danger_bg,
        Role::HighlightDangerFg => c.highlight_danger_fg,
        Role::BackdropFg => c.backdrop_fg,
        Role::BackdropBg => c.backdrop_bg,
        Role::Danger => c.danger,
        Role::DangerSoft => c.danger_soft,
        Role::DangerTint => c.danger_tint,
        Role::Warning => c.warning,
        Role::WarningTint => c.warning_tint,
        Role::Success => c.success,
        Role::Info => c.info,
        Role::DisabledFg => c.disabled_fg,
        Role::DisabledBg => c.disabled_bg,
        Role::ReadOnlyFg => c.read_only_fg,
        Role::Syntax(s) => {
            let t = &c.syntax;
            match s {
                SyntaxRole::Keyword => t.keyword,
                SyntaxRole::Ident => t.ident,
                SyntaxRole::Str => t.string,
                SyntaxRole::Number => t.number,
                SyntaxRole::Operator => t.operator,
                SyntaxRole::Punct => t.punct,
                SyntaxRole::Comment => t.comment,
                SyntaxRole::Plain => t.plain,
                SyntaxRole::TypeName => t.type_name,
                SyntaxRole::Function => t.function,
                SyntaxRole::Constant => t.constant,
                SyntaxRole::Invalid => t.invalid,
                SyntaxRole::Deprecated => t.deprecated,
                SyntaxRole::MatchBg => t.match_bg,
                SyntaxRole::MatchCurrentBg => t.match_current_bg,
                SyntaxRole::BracketMatch => t.bracket_match,
                SyntaxRole::DiagError => t.diagnostic_error,
                SyntaxRole::DiagWarning => t.diagnostic_warning,
                SyntaxRole::DiagInfo => t.diagnostic_info,
            }
        }
        Role::Meter(m) => {
            let t = &c.meter;
            match m {
                MeterRole::Low => t.low,
                MeterRole::Medium => t.medium,
                MeterRole::High => t.high,
                MeterRole::Track => t.track,
                MeterRole::FillRest => match t.fill_rest {
                    super::MeterFillRest::Color(color) => color,
                    super::MeterFillRest::ReferenceLift => reference_lift(theme, surface),
                },
                MeterRole::Stale => t.stale,
                MeterRole::Unknown => t.unknown,
                MeterRole::Series(n) => t
                    .series
                    .get(usize::from(n) % 6)
                    .copied()
                    .unwrap_or(Color::Reset),
            }
        }
        Role::Custom(raw) => downgrade_color(raw, theme.capability.color),
    };
    if color == Color::Reset {
        None
    } else {
        Some(color)
    }
}

fn bind_slot(theme: &Theme, slot: Slot<Role>, surface: Surface) -> Option<Color> {
    match slot {
        Slot::Set(r) => bind_role(theme, r, surface),
        Slot::Inherit | Slot::Clear => None,
    }
}

pub fn bind(
    theme: &Theme,
    acc: StylePatch,
    local: Option<&StylePatch>,
    surface: Surface,
) -> Resolved {
    bind_with_profile(theme, acc, local, surface, ProfiledStyle::default())
}

fn bind_with_profile(
    theme: &Theme,
    acc: StylePatch,
    local: Option<&StylePatch>,
    surface: Surface,
    mut exact_profile: ProfiledStyle,
) -> Resolved {
    if let Some(local) = local {
        exact_profile.suppress(*local);
    }
    let acc = match local {
        Some(p) => acc.merge(*p),
        None => acc,
    };
    let mut style = Style::new();
    style.fg = bind_profile_slot(theme, acc.fg, exact_profile.fg, surface);
    style.bg = bind_profile_slot(theme, acc.bg, exact_profile.bg, surface);
    style.underline_color =
        bind_profile_slot(theme, acc.underline, exact_profile.underline, surface);
    style.add_modifier = acc.add;
    style.sub_modifier = acc.remove;
    let disabled_surface_ink = matches!(acc.fg, Slot::Set(Role::DisabledFg))
        && matches!(acc.bg, Slot::Set(Role::CurrentSurface));
    let disabled_collision = disabled_surface_ink && style.fg.is_some() && style.fg == style.bg;
    let fg_role = if disabled_collision {
        style.fg = bind_slot(theme, Slot::Set(Role::Fg(FgStep::Muted)), surface);
        if !acc.remove.contains(Modifier::DIM) {
            style.add_modifier |= Modifier::DIM;
        }
        Some(Role::Fg(FgStep::Muted))
    } else {
        acc.fg.get().map(|role| canonical_role(theme, role))
    };
    let m = metrics_of(&acc);
    Resolved {
        style: PaintStyle::bound(
            style,
            fg_role,
            acc.bg.get().map(|role| canonical_role(theme, role)),
            surface,
        )
        .with_underline_role(
            acc.underline
                .get()
                .map(|role| canonical_role(theme, role))
                .filter(|_| style.underline_color.is_some())
                .map(|role| (role, surface)),
        )
        .with_underline_cleared(matches!(acc.underline, Slot::Clear)),
        glyph: acc.glyph,
        size: m.size,
        align: m.align,
    }
}

fn bind_profile_slot(
    theme: &Theme,
    slot: Slot<Role>,
    preserve_authored_custom: bool,
    surface: Surface,
) -> Option<Color> {
    if preserve_authored_custom && let Slot::Set(Role::Custom(color)) = slot {
        return Some(color);
    }
    bind_slot(theme, slot, surface)
}

fn canonical_role(theme: &Theme, role: Role) -> Role {
    if role == Role::Meter(MeterRole::FillRest)
        && theme.color.meter.fill_rest == super::MeterFillRest::ReferenceLift
    {
        Role::HoverSurface
    } else {
        role
    }
}

const CACHE_SLOTS: usize = 256;

const WAYS: usize = 2;
const CACHE_SETS: usize = CACHE_SLOTS / WAYS;

fn color_fingerprint(mut hash: u64, color: Color) -> u64 {
    let (tag, payload) = match color {
        Color::Reset => (0, [0, 0, 0]),
        Color::Black => (1, [0, 0, 0]),
        Color::Red => (2, [0, 0, 0]),
        Color::Green => (3, [0, 0, 0]),
        Color::Yellow => (4, [0, 0, 0]),
        Color::Blue => (5, [0, 0, 0]),
        Color::Magenta => (6, [0, 0, 0]),
        Color::Cyan => (7, [0, 0, 0]),
        Color::Gray => (8, [0, 0, 0]),
        Color::DarkGray => (9, [0, 0, 0]),
        Color::LightRed => (10, [0, 0, 0]),
        Color::LightGreen => (11, [0, 0, 0]),
        Color::LightYellow => (12, [0, 0, 0]),
        Color::LightBlue => (13, [0, 0, 0]),
        Color::LightMagenta => (14, [0, 0, 0]),
        Color::LightCyan => (15, [0, 0, 0]),
        Color::White => (16, [0, 0, 0]),
        Color::Indexed(index) => (17, [index, 0, 0]),
        Color::Rgb(r, g, b) => (18, [r, g, b]),
    };
    hash = fnv1a(hash, &[tag]);
    fnv1a(hash, &payload)
}

fn profile_token_fingerprint(theme: &Theme, context: ProfileContext, mut hash: u64) -> u64 {
    for (profile_variant, profile_surface) in [
        (None, None),
        (Some(context.variant), None),
        (Some(context.variant), Some(context.surface)),
    ] {
        let Some(profile) = theme.recipes.level_family(
            theme.capability.color,
            context.family,
            profile_variant,
            profile_surface,
        ) else {
            continue;
        };
        let profile_requested_variant = profile.variant.unwrap_or(context.requested_variant);
        let (seed, _) = apply_recipe(
            &profile.seed,
            profile_requested_variant,
            context.part,
            context.live,
            StylePatch::new(),
        );
        for (channel, slot) in [(0, seed.fg), (1, seed.bg), (2, seed.underline)] {
            let Slot::Set(role) = slot else {
                continue;
            };
            let Some(color) = profile_role_color(theme.color, role, context.surface) else {
                continue;
            };
            hash = fnv1a(hash, &[channel]);
            hash = color_fingerprint(hash, color);
        }
    }
    hash
}

#[derive(Clone, Debug)]
pub struct StyleCache {
    slots: Box<[(u64, u32, StylePatch, ProfiledStyle); CACHE_SLOTS]>,
    generation: u32,
    hits: u64,
    misses: u64,
}

impl Default for StyleCache {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleCache {
    pub(crate) fn new() -> Self {
        StyleCache {
            slots: Box::new([(0, 0, StylePatch::new(), ProfiledStyle::default()); CACHE_SLOTS]),
            generation: 1,
            hits: 0,
            misses: 0,
        }
    }

    pub fn clear(&mut self) {
        if self.generation == u32::MAX {
            self.slots
                .fill((0, 0, StylePatch::new(), ProfiledStyle::default()));
            self.generation = 1;
        } else {
            self.generation = self.generation.saturating_add(1);
        }
    }

    #[cfg(any(test, feature = "testing"))]
    pub const fn stats(&self) -> (u64, u64) {
        (self.hits, self.misses)
    }

    fn key(
        theme: &Theme,
        f: Family,
        v: Variant,
        p: Part,
        live: StateFlags,
        stack_hash: u64,
        surface: Surface,
    ) -> u64 {
        let mut h = fnv1a(0xcbf2_9ce4_8422_2325, &f.raw().to_le_bytes());
        h = fnv1a(h, &v.raw().to_le_bytes());
        h = fnv1a(h, &p.raw().to_le_bytes());
        h = fnv1a(h, &live.bits().to_le_bytes());
        h = fnv1a(h, &stack_hash.to_le_bytes());
        h = fnv1a(
            h,
            &[match surface {
                Surface::Canvas => 0,
                Surface::Surface => 1,
                Surface::Elevated => 2,
                Surface::Overlay => 3,
                Surface::Popover => 4,
                Surface::Field => 5,
                Surface::FieldHover => 6,
            }],
        );
        let resolved_variant = theme.recipes.get(f).map_or(v, |recipe| {
            apply_recipe(recipe, v, p, live, StylePatch::new()).1
        });
        h = profile_token_fingerprint(
            theme,
            ProfileContext {
                family: f,
                requested_variant: v,
                variant: resolved_variant,
                part: p,
                live,
                surface,
            },
            h,
        );
        h = fnv1a(
            h,
            &[match theme.capability.color {
                super::ColorLevel::TrueColor => 0,
                super::ColorLevel::Ansi256 => 1,
                super::ColorLevel::Ansi16 => 2,
                super::ColorLevel::Mono => 3,
            }],
        );
        fnv1a(h, &theme.recipes.revision().to_le_bytes())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the §11.1 A3 memo key plus the theme and the stack"
    )]
    pub fn accumulate(
        &mut self,
        theme: &Theme,
        f: Family,
        v: Variant,
        p: Part,
        live: StateFlags,
        overlays: &[Overlay],
        stack_hash: u64,
    ) -> StylePatch {
        self.accumulate_profiled(theme, f, v, p, live, overlays, stack_hash, Surface::Canvas)
            .0
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the cache key includes all resolved-style dimensions plus its binding surface"
    )]
    pub fn resolve(
        &mut self,
        theme: &Theme,
        f: Family,
        v: Variant,
        p: Part,
        live: StateFlags,
        overlays: &[Overlay],
        stack_hash: u64,
        surface: Surface,
    ) -> Resolved {
        let (patch, profile) =
            self.accumulate_profiled(theme, f, v, p, live, overlays, stack_hash, surface);
        bind_with_profile(theme, patch, None, surface, profile)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the cache key includes all resolved-style dimensions plus its binding surface"
    )]
    pub fn resolve_patched(
        &mut self,
        theme: &Theme,
        f: Family,
        v: Variant,
        p: Part,
        live: StateFlags,
        overlays: &[Overlay],
        stack_hash: u64,
        surface: Surface,
        local: &StylePatch,
    ) -> Resolved {
        let (patch, mut profile) =
            self.accumulate_profiled(theme, f, v, p, live, overlays, stack_hash, surface);
        profile.suppress(*local);
        bind_with_profile(theme, patch, Some(local), surface, profile)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the §11.1 A3 memo key plus the theme and the stack"
    )]
    fn accumulate_profiled(
        &mut self,
        theme: &Theme,
        f: Family,
        v: Variant,
        p: Part,
        live: StateFlags,
        overlays: &[Overlay],
        stack_hash: u64,
        surface: Surface,
    ) -> (StylePatch, ProfiledStyle) {
        let key = Self::key(theme, f, v, p, live, stack_hash, surface) | 1;

        let set = ((key >> 1) as usize) % CACHE_SETS;
        let base = set.saturating_mul(WAYS);
        for w in 0..WAYS {
            if let Some((k, g, patch, profile)) = self.slots.get(base.saturating_add(w))
                && *k == key
                && *g == self.generation
            {
                let cached = (*patch, *profile);
                self.hits = self.hits.wrapping_add(1);
                if w != 0 {
                    self.promote(base, w);
                }
                return cached;
            }
        }
        let (patch, profile) = accumulate_profiled(theme, f, v, p, live, overlays, surface);

        for w in (1..WAYS).rev() {
            let prev = self
                .slots
                .get(base.saturating_add(w).saturating_sub(1))
                .copied();
            if let (Some(p), Some(slot)) = (prev, self.slots.get_mut(base.saturating_add(w))) {
                *slot = p;
            }
        }
        if let Some(slot) = self.slots.get_mut(base) {
            *slot = (key, self.generation, patch, profile);
        }
        self.misses = self.misses.wrapping_add(1);
        (patch, profile)
    }

    fn promote(&mut self, base: usize, w: usize) {
        let Some(hit) = self.slots.get(base.saturating_add(w)).copied() else {
            return;
        };
        for i in (1..=w).rev() {
            let prev = self
                .slots
                .get(base.saturating_add(i).saturating_sub(1))
                .copied();
            if let (Some(p), Some(slot)) = (prev, self.slots.get_mut(base.saturating_add(i))) {
                *slot = p;
            }
        }
        if let Some(slot) = self.slots.get_mut(base) {
            *slot = hit;
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the six precedence inputs plus the theme and the surface"
)]
pub fn resolve_uncached(
    theme: &Theme,
    f: Family,
    v: Variant,
    p: Part,
    live: StateFlags,
    surface: Surface,
    overlays: &[Overlay],
    local: Option<&StylePatch>,
) -> Resolved {
    let (acc, mut exact_profile) = accumulate_profiled(theme, f, v, p, live, overlays, surface);
    if let Some(local) = local {
        exact_profile.suppress(*local);
    }
    bind_with_profile(theme, acc, local, surface, exact_profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ColorLevel;
    use crate::theme::patch::StateRule;
    use crate::theme::recipe::{GlobalOverride, OverlayRule, PartMap, PartRecipe};
    use crate::theme::role::FgStep;
    use ratatui_core::style::Modifier;

    fn theme_over(base: Theme) -> Theme {
        let mut t = base;
        let r = t.recipes.get_mut(Family::custom("t"));
        r.default_variant = Variant::PRIMARY;

        let part = r.parts.entry(Part::LABEL);
        part.base = StylePatch::new().set_fg(Role::Fg(FgStep::Primary));
        part.when(StateFlags::FOCUSED, StylePatch::new().set_fg(Role::Warning));

        r.variant_mut(Variant::PRIMARY).entry(Part::LABEL).base =
            StylePatch::new().set_fg(Role::Accent);
        t
    }

    fn theme() -> Theme {
        theme_over(Theme::junie())
    }

    #[test]
    fn explicit_style_channels_report_theme_overlay_and_instance_precedence() {
        static RULES: [OverlayRule; 1] = [(
            Family::LIST,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
            StylePatch::new()
                .clear_fg()
                .set_underline(Role::Warning)
                .add(Modifier::UNDERLINED),
        )];
        let theme = Theme::junie().override_family(Family::LIST, |recipe| {
            recipe.part(Part::CONTAINER).base(
                StylePatch::new()
                    .set_fg(Role::Danger)
                    .set_bg(Role::Danger)
                    .set_underline(Role::Accent)
                    .add(Modifier::BOLD),
            );
        });
        let overlay = Overlay::new(&RULES);
        let mut local = StylePatch::new()
            .set_fg(Role::Info)
            .clear_bg()
            .remove(Modifier::UNDERLINED);
        local.underline = Slot::Clear;
        let channels = bind_defaults_with_explicit(
            &theme,
            (Family::LIST, Variant::DEFAULT, Part::CONTAINER),
            StateFlags::empty(),
            &[overlay],
            Surface::Canvas,
            StyleDefaults::default(),
            Some(&local),
        );
        assert_eq!(channels.explicit.fg, Slot::Set(Role::Info));
        assert_eq!(channels.explicit.bg, Slot::Clear);
        assert_eq!(channels.explicit.underline, Slot::Clear);
        assert!(channels.explicit.add.contains(Modifier::BOLD));
        assert!(!channels.explicit.add.contains(Modifier::UNDERLINED));
        assert!(channels.explicit.remove.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn bound_underline_role_survives_inheritance_and_raw_color_replaces_it() {
        let theme = Theme::junie().for_level(ColorLevel::Mono);
        let role = Role::Fg(FgStep::Faint);
        let bound = bind(
            &theme,
            StylePatch::new().set_underline(role),
            None,
            Surface::Canvas,
        )
        .style;

        assert_eq!(bound.underline_role(), Some((role, Surface::Canvas)));

        let raw = bound.underline_color(Color::Yellow);
        assert_eq!(raw.underline_role(), None);
        assert_eq!(raw.as_style().underline_color, Some(Color::Yellow));

        let patch = bound.patch(Style::new().underline_color(Color::Blue));
        assert_eq!(patch.underline_role(), None);
        assert_eq!(patch.as_style().underline_color, Some(Color::Blue));

        let inherited = bound.patch(Style::new().add_modifier(Modifier::ITALIC));
        assert_eq!(inherited.underline_role(), Some((role, Surface::Canvas)));
    }

    #[test]
    fn underline_clear_provenance_tracks_the_final_canonical_channel() {
        for level in [
            super::super::ColorLevel::TrueColor,
            super::super::ColorLevel::Ansi256,
            super::super::ColorLevel::Ansi16,
            super::super::ColorLevel::Mono,
        ] {
            let theme = Theme::junie().for_level(level);
            let clear = bind(
                &theme,
                StylePatch {
                    underline: Slot::Clear,
                    ..StylePatch::new()
                },
                None,
                Surface::Canvas,
            )
            .style;
            assert!(clear.underline_cleared(), "at {level:?}");
            assert_eq!(clear.underline_role(), None, "at {level:?}");

            let inherited = bind(&theme, StylePatch::new(), None, Surface::Canvas).style;
            assert!(!inherited.underline_cleared(), "at {level:?}");

            let set = bind(
                &theme,
                StylePatch::new().set_underline(Role::Warning),
                None,
                Surface::Canvas,
            )
            .style;
            assert!(!set.underline_cleared(), "at {level:?}");
            assert!(set.underline_role().is_some(), "at {level:?}");
            let local_set = bind(
                &theme,
                StylePatch {
                    underline: Slot::Clear,
                    ..StylePatch::new()
                },
                Some(&StylePatch::new().set_underline(Role::Warning)),
                Surface::Canvas,
            )
            .style;
            assert!(
                !local_set.underline_cleared(),
                "local Set wins at {level:?}"
            );
            assert_eq!(
                local_set.underline_role().map(|(role, _)| role),
                Some(Role::Warning),
                "local Set retains its semantic binding at {level:?}"
            );
            let local_clear = bind(
                &theme,
                StylePatch::new().set_underline(Role::Warning),
                Some(&StylePatch {
                    underline: Slot::Clear,
                    ..StylePatch::new()
                }),
                Surface::Canvas,
            )
            .style;
            assert!(
                local_clear.underline_cleared(),
                "local Clear wins at {level:?}"
            );

            assert!(
                clear
                    .underline_color(Color::Yellow)
                    .underline_role()
                    .is_none()
                    && !clear.underline_color(Color::Yellow).underline_cleared(),
                "raw replacement drops bound Clear provenance at {level:?}"
            );
            assert!(
                clear
                    .patch(Style::new().underline_color(Color::Blue))
                    .underline_role()
                    .is_none()
                    && !clear
                        .patch(Style::new().underline_color(Color::Blue))
                        .underline_cleared(),
                "raw patch replacement drops bound Clear provenance at {level:?}"
            );
            assert!(
                clear
                    .patch(Style::new().add_modifier(Modifier::ITALIC))
                    .underline_cleared(),
                "unrelated patch preserves bound Clear provenance at {level:?}"
            );
            assert!(
                clear.patch(clear).underline_cleared(),
                "a resolved Clear patch replaces the channel with Clear at {level:?}"
            );
            assert!(
                clear.patch(set).underline_role().is_some()
                    && !clear.patch(set).underline_cleared(),
                "a bound Set patch replaces Clear at {level:?}"
            );
        }
    }

    fn label_fg(
        t: &Theme,
        live: StateFlags,
        ovs: &[Overlay],
        inst: Option<&StylePatch>,
    ) -> Option<Color> {
        resolve_uncached(
            t,
            Family::custom("t"),
            Variant::DEFAULT,
            Part::LABEL,
            live,
            Surface::Canvas,
            ovs,
            inst,
        )
        .style
        .fg
    }

    #[test]
    fn precedence_family_then_variant_then_state_then_global_then_scope_then_instance() {
        for base in [Theme::junie(), Theme::paper()] {
            let mut t = theme_over(base);
            let f = Family::custom("t");

            assert_ne!(t.color.warning, t.color.accent);
            assert_ne!(t.color.success, t.color.warning);
            assert_ne!(t.color.info, t.color.success);
            assert_ne!(t.color.danger, t.color.info);
            let q = label_fg;

            assert_eq!(q(&t, StateFlags::empty(), &[], None), Some(t.color.accent));

            assert_eq!(q(&t, StateFlags::FOCUSED, &[], None), Some(t.color.warning));

            let mut parts: PartMap<PartRecipe> = PartMap::new();
            parts.entry(Part::LABEL).base = StylePatch::new().set_fg(Role::Success);
            t.recipes.push_override(GlobalOverride {
                family: f,
                variant: None,
                parts,
            });
            assert_eq!(q(&t, StateFlags::FOCUSED, &[], None), Some(t.color.success));

            static OV: [OverlayRule; 1] = [(
                Family::custom("t"),
                Variant::PRIMARY,
                Part::LABEL,
                StateFlags::empty(),
                StylePatch::new().set_fg(Role::Info),
            )];
            let ov = Overlay::new(&OV);
            assert_eq!(q(&t, StateFlags::FOCUSED, &[ov], None), Some(t.color.info));

            let inst = StylePatch::new().set_fg(Role::Danger);
            assert_eq!(
                q(&t, StateFlags::FOCUSED, &[ov], Some(&inst)),
                Some(t.color.danger)
            );

            static OV2: [OverlayRule; 1] = [(
                Family::custom("t"),
                Variant::PRIMARY,
                Part::LABEL,
                StateFlags::empty(),
                StylePatch::new().set_fg(Role::Success),
            )];
            assert_eq!(
                q(&t, StateFlags::empty(), &[ov, Overlay::new(&OV2)], None),
                Some(t.color.success)
            );
        }
    }

    #[test]
    fn state_rules_beat_a_variant_base() {
        for base in [Theme::junie(), Theme::paper()] {
            let t = theme_over(base);
            assert_ne!(t.color.warning, t.color.accent);

            assert_eq!(
                label_fg(&t, StateFlags::empty(), &[], None),
                Some(t.color.accent)
            );

            assert_eq!(
                label_fg(&t, StateFlags::FOCUSED, &[], None),
                Some(t.color.warning)
            );
        }
    }

    #[test]
    fn family_and_variant_state_rules_interleave_by_specificity() {
        let mut t = Theme::junie();
        let f = Family::custom("i");
        {
            let r = t.recipes.get_mut(f);
            r.default_variant = Variant::PRIMARY;
            let part = r.parts.entry(Part::LABEL);
            part.when(
                StateFlags::FOCUSED,
                StylePatch::new()
                    .set_fg(Role::Warning)
                    .set_bg(Role::DangerTint),
            );
            part.when(
                StateFlags::FOCUSED | StateFlags::HOVERED,
                StylePatch::new().set_fg(Role::Info),
            );
            let vp = r.variant_mut(Variant::PRIMARY).entry(Part::LABEL);
            vp.when(
                StateFlags::FOCUSED,
                StylePatch::new()
                    .set_fg(Role::Success)
                    .set_bg(Role::AccentTint),
            );
        }
        let q = |live| {
            resolve_uncached(
                &t,
                f,
                Variant::DEFAULT,
                Part::LABEL,
                live,
                Surface::Canvas,
                &[],
                None,
            )
            .style
        };

        assert_eq!(q(StateFlags::FOCUSED).fg, Some(t.color.success));
        assert_eq!(q(StateFlags::FOCUSED).bg, Some(t.color.accent_tint));

        let both = StateFlags::FOCUSED | StateFlags::HOVERED;
        assert_eq!(q(both).fg, Some(t.color.info));
        assert_eq!(q(both).bg, Some(t.color.accent_tint));
    }

    #[test]
    fn a_custom_family_resolves_through_the_neutral_recipe() {
        let t = Theme::junie();
        let f = Family::custom("segmented");
        assert!(t.recipes.get(f).is_none());
        let container = t.resolve(
            f,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
            Surface::Canvas,
        );

        assert_eq!(container.style.fg, Some(t.color.fg[0]));
        assert_eq!(container.style.bg, Some(t.bg(Surface::Canvas)));

        let focused = t.resolve(
            f,
            Variant::DEFAULT,
            Part::GUTTER,
            StateFlags::FOCUSED,
            Surface::Canvas,
        );
        assert_eq!(focused.glyph, Slot::Set(GlyphRole::FocusBar));
        assert_eq!(focused.style.fg, Some(t.color.focus));
    }

    #[test]
    fn roles_bind_after_the_whole_chain() {
        let t = theme();
        let inst = StylePatch::new()
            .set_fg(Role::CurrentSurface)
            .set_bg(Role::RaisedSurface);
        let r = resolve_uncached(
            &t,
            Family::custom("t"),
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::empty(),
            Surface::Surface,
            &[],
            Some(&inst),
        );

        assert_eq!(r.style.fg, Some(t.bg(Surface::Surface)));
        assert_eq!(r.style.bg, Some(t.bg(Surface::Elevated)));
        let clear = StylePatch::new().clear_fg();
        let r = resolve_uncached(
            &t,
            Family::custom("t"),
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::empty(),
            Surface::Canvas,
            &[],
            Some(&clear),
        );
        assert_eq!(r.style.fg, None);
    }

    #[test]
    fn patch_merge_matches_ratatui_style_patch_for_modifiers() {
        let t = theme();
        let inst = StylePatch::new()
            .add(Modifier::ITALIC)
            .remove(Modifier::BOLD);
        let r = resolve_uncached(
            &t,
            Family::custom("t"),
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::empty(),
            Surface::Canvas,
            &[],
            Some(&inst),
        );
        let inherited = Style::new().add_modifier(Modifier::BOLD | Modifier::DIM);
        let out = r.over(inherited);
        assert_eq!(out.add_modifier, Modifier::ITALIC | Modifier::DIM);
        assert_eq!(out.sub_modifier, Modifier::BOLD);

        let merged = StylePatch::new()
            .add(Modifier::BOLD | Modifier::DIM)
            .merge(inst);
        assert_eq!(merged.add, out.add_modifier);
        assert_eq!(merged.remove, out.sub_modifier);
    }

    #[test]
    fn cache_hits_after_the_first_query_and_clears_by_generation() {
        let t = theme();
        let mut c = StyleCache::new();
        let f = Family::custom("t");
        let a = c.accumulate(
            &t,
            f,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::FOCUSED,
            &[],
            0,
        );
        let b = c.accumulate(
            &t,
            f,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::FOCUSED,
            &[],
            0,
        );
        assert_eq!(a, b);
        assert_eq!(c.stats(), (1, 1));
        c.clear();
        let _ = c.accumulate(
            &t,
            f,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::FOCUSED,
            &[],
            0,
        );
        assert_eq!(c.stats(), (1, 2));
        let rule = StateRule {
            when: StateFlags::FOCUSED,
            patch: StylePatch::new(),
        };
        assert!(rule.matches(StateFlags::FOCUSED));
        let r = PartRecipe::default();
        assert!(r.apply(StylePatch::new(), StateFlags::empty()).is_empty());
    }

    #[test]
    fn cache_generation_wrap_does_not_serve_a_stale_entry() {
        let t = theme();
        let mut c = StyleCache::new();
        let f = Family::custom("t");
        let query = |c: &mut StyleCache| {
            c.accumulate(
                &t,
                f,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::FOCUSED,
                &[],
                0,
            )
        };
        assert_eq!(c.generation, 1, "a fresh cache stamps at generation 1");
        let seeded = query(&mut c);
        assert_eq!(c.stats(), (0, 1), "the first query is a miss");

        c.generation = u32::MAX;
        c.clear();
        assert_eq!(c.generation, 1, "the stamp restarts at 1 after u32::MAX");
        let after = query(&mut c);
        assert_eq!(
            c.stats(),
            (0, 2),
            "the seeded key must miss after the stamp wraps, not serve a stale patch"
        );
        assert_eq!(seeded, after, "and the recomputed patch is the same value");
    }
}
