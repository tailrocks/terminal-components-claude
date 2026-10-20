use ratatui_core::style::{Color, Style};

use super::Theme;
use super::downgrade::downgrade_color;
use super::glyph::GlyphRole;
use super::patch::{Slot, StylePatch};
use super::recipe::{Family, Overlay, Variant};
use super::role::{Align, MeterRole, Role, Surface, SyntaxRole};
use crate::id::{Part, fnv1a};
use crate::response::StateFlags;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PaintStyle {
    style: Style,
    pub fg_role: Option<(Role, Surface)>,
    pub bg_role: Option<(Role, Surface)>,
}

impl PaintStyle {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            style: Style::new(),
            fg_role: None,
            bg_role: None,
        }
    }

    pub fn bound(style: Style, fg: Option<Role>, bg: Option<Role>, surface: Surface) -> Self {
        Self {
            style,
            fg_role: fg.filter(|_| style.fg.is_some()).map(|r| (r, surface)),
            bg_role: bg.filter(|_| style.bg.is_some()).map(|r| (r, surface)),
        }
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
    let (defaults, variant) = accumulate_defaults(theme, f, v, p, live);
    apply_explicit(theme, (f, variant, p), live, overlays, defaults)
}

fn accumulate_defaults(
    theme: &Theme,
    f: Family,
    v: Variant,
    p: Part,
    live: StateFlags,
) -> (StylePatch, Variant) {
    let recipes = &theme.recipes;

    let (mut acc, variant) = apply_recipe(recipes.get_or_neutral(f), v, p, live, StylePatch::new());

    if theme.capability.color == super::ColorLevel::Mono {
        acc = super::downgrade::apply_mono_fallback(acc, recipes, f, p, live);
    }
    (acc, variant)
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
    let (mut acc, variant) = match theme.recipes.get(family) {
        Some(recipe) => apply_recipe(recipe, variant, part, flags, defaults.base),
        None => (defaults.base, variant),
    };
    if theme.capability.color == super::ColorLevel::Mono {
        acc = super::downgrade::apply_mono_with_defaults(
            acc,
            &theme.recipes,
            family,
            part,
            flags,
            Some(defaults.mono),
        );
    }
    acc = apply_explicit(theme, (family, variant, part), flags, overlays, acc);
    bind(theme, acc, local, surface)
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
    let (mut acc, variant) = match theme.recipes.get(family) {
        Some(recipe) => apply_recipe(recipe, variant, part, flags, defaults.base),
        None => (defaults.base, variant),
    };
    if theme.capability.color == super::ColorLevel::Mono {
        acc = super::downgrade::apply_mono_with_defaults(
            acc,
            &theme.recipes,
            family,
            part,
            flags,
            Some(defaults.mono),
        );
    }
    let explicit = apply_explicit(
        theme,
        (family, variant, part),
        flags,
        overlays,
        StylePatch::new(),
    );
    acc = acc.merge(explicit);
    let resolved = bind(theme, acc, local, surface);
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
    let (defaults, variant) = accumulate_defaults(theme, family, variant, part, flags);
    let explicit = apply_explicit(
        theme,
        (family, variant, part),
        flags,
        overlays,
        StylePatch::new(),
    );
    let base = bind(theme, defaults, None, surface).style;
    let top = bind(theme, explicit, None, surface).style;
    let mut result = bind(theme, defaults.merge(explicit), None, surface);
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
    let acc = match local {
        Some(p) => acc.merge(*p),
        None => acc,
    };
    let mut style = Style::new();
    style.fg = bind_slot(theme, acc.fg, surface);
    style.bg = bind_slot(theme, acc.bg, surface);
    style.underline_color = bind_slot(theme, acc.underline, surface);
    style.add_modifier = acc.add;
    style.sub_modifier = acc.remove;
    let m = metrics_of(&acc);
    Resolved {
        style: PaintStyle::bound(
            style,
            acc.fg.get().map(|role| canonical_role(theme, role)),
            acc.bg.get().map(|role| canonical_role(theme, role)),
            surface,
        ),
        glyph: acc.glyph,
        size: m.size,
        align: m.align,
    }
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

#[derive(Clone, Debug)]
pub struct StyleCache {
    slots: Box<[(u64, u32, StylePatch); CACHE_SLOTS]>,
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
            slots: Box::new([(0, 0, StylePatch::new()); CACHE_SLOTS]),
            generation: 1,
            hits: 0,
            misses: 0,
        }
    }

    pub fn clear(&mut self) {
        if self.generation == u32::MAX {
            self.slots.fill((0, 0, StylePatch::new()));
            self.generation = 1;
        } else {
            self.generation = self.generation.saturating_add(1);
        }
    }

    #[cfg(any(test, feature = "testing"))]
    pub const fn stats(&self) -> (u64, u64) {
        (self.hits, self.misses)
    }

    fn key(f: Family, v: Variant, p: Part, live: StateFlags, stack_hash: u64) -> u64 {
        let mut h = fnv1a(0xcbf2_9ce4_8422_2325, &f.raw().to_le_bytes());
        h = fnv1a(h, &v.raw().to_le_bytes());
        h = fnv1a(h, &p.raw().to_le_bytes());
        h = fnv1a(h, &live.bits().to_le_bytes());
        fnv1a(h, &stack_hash.to_le_bytes())
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
        let key = Self::key(f, v, p, live, stack_hash) | 1;

        let set = ((key >> 1) as usize) % CACHE_SETS;
        let base = set.saturating_mul(WAYS);
        for w in 0..WAYS {
            if let Some((k, g, patch)) = self.slots.get(base.saturating_add(w))
                && *k == key
                && *g == self.generation
            {
                let patch = *patch;
                self.hits = self.hits.wrapping_add(1);
                if w != 0 {
                    self.promote(base, w);
                }
                return patch;
            }
        }
        let patch = accumulate(theme, f, v, p, live, overlays);

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
            *slot = (key, self.generation, patch);
        }
        self.misses = self.misses.wrapping_add(1);
        patch
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
    let acc = accumulate(theme, f, v, p, live, overlays);
    bind(theme, acc, local, surface)
}

#[cfg(test)]
mod tests {
    use super::*;
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
