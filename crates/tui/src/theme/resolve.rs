//! Style resolution (`COMPONENT_ARCHITECTURE.md` §11.3, §20.9-1/-2/-4).
//!
//! Precedence, lowest → highest: family base, variant delta, state rules (by
//! specificity), theme-level global override, scope overlay stack (outermost
//! → innermost), per-instance patch. Then, and only then, roles bind to
//! colours against `(theme.color, surface, theme.capability)`. Steps 1–5 are
//! memoised in a statically sized two-way set-associative cache (256 entries
//! in 128 sets) keyed by a 64-bit mix.

use ratatui_core::style::{Color, Style};

use super::Theme;
use super::downgrade::downgrade_color;
use super::glyph::GlyphRole;
use super::patch::{Slot, StylePatch};
use super::recipe::{Family, Overlay, Variant};
use super::role::{Align, MeterRole, Role, Surface, SyntaxRole};
use crate::id::{Part, fnv1a};
use crate::response::StateFlags;

/// Bound paint attributes together with their semantic origin.
///
/// Raw colours deliberately carry no role. A copy retains its origin even
/// when painted after another query or outside its original surface scope.
/// Direct mutation cannot leave semantic metadata describing an old colour:
///
/// ```compile_fail
/// use junie_tui::theme::PaintStyle;
/// use ratatui_core::style::Color;
/// let mut style = PaintStyle::new();
/// style.bg = Some(Color::Red);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PaintStyle {
    style: Style,
    pub(crate) fg_role: Option<(Role, Surface)>,
    pub(crate) bg_role: Option<(Role, Surface)>,
}

impl PaintStyle {
    /// An inheriting style with no explicit attributes.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            style: Style::new(),
            fg_role: None,
            bg_role: None,
        }
    }

    pub(crate) fn bound(
        style: Style,
        fg: Option<Role>,
        bg: Option<Role>,
        surface: Surface,
    ) -> Self {
        Self {
            style,
            fg_role: fg.filter(|_| style.fg.is_some()).map(|r| (r, surface)),
            bg_role: bg.filter(|_| style.bg.is_some()).map(|r| (r, surface)),
        }
    }

    /// Borrow the raw attributes for a foreign renderer. This erases semantics
    /// at that boundary; Junie painters should receive the carrier itself.
    #[must_use]
    pub const fn as_style(&self) -> &Style {
        &self.style
    }

    /// Explicitly discard semantic provenance at a foreign renderer boundary.
    #[must_use]
    pub const fn into_style(self) -> Style {
        self.style
    }

    /// Set a raw foreground, discarding only its semantic provenance.
    #[must_use]
    pub fn fg(mut self, color: Color) -> Self {
        self.style = self.style.fg(color);
        self.fg_role = None;
        self
    }

    /// Set a raw background, discarding only its semantic provenance.
    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.style = self.style.bg(color);
        self.bg_role = None;
        self
    }

    /// Copy the fg channel and its semantic origin into the fg channel.
    #[must_use]
    pub fn with_fg_from(mut self, other: Self) -> Self {
        self.style.fg = other.style.fg;
        self.fg_role = other.fg_role;
        self
    }

    /// Copy the bg channel and its semantic origin into the bg channel.
    #[must_use]
    pub fn with_bg_from(mut self, other: Self) -> Self {
        self.style.bg = other.style.bg;
        self.bg_role = other.bg_role;
        self
    }

    /// Copy the bg channel and its semantic origin into the fg channel.
    #[must_use]
    pub fn with_fg_from_bg(mut self, other: Self) -> Self {
        self.style.fg = other.style.bg;
        self.fg_role = other.bg_role;
        self
    }

    /// Copy the fg channel and its semantic origin into the bg channel.
    #[must_use]
    pub fn with_bg_from_fg(mut self, other: Self) -> Self {
        self.style.bg = other.style.fg;
        self.bg_role = other.fg_role;
        self
    }

    /// Set the underline colour; foreground/background origins are unchanged.
    #[must_use]
    pub fn underline_color(mut self, color: Color) -> Self {
        self.style = self.style.underline_color(color);
        self
    }

    /// Add modifiers without changing colour origins.
    #[must_use]
    pub fn add_modifier(mut self, modifier: ratatui_core::style::Modifier) -> Self {
        self.style = self.style.add_modifier(modifier);
        self
    }

    /// Remove modifiers without changing colour origins.
    #[must_use]
    pub fn remove_modifier(mut self, modifier: ratatui_core::style::Modifier) -> Self {
        self.style = self.style.remove_modifier(modifier);
        self
    }

    /// Layer explicit channels from `other`, preserving inherited origins.
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

// Read-only attribute inspection cannot detach or mutate the carried origins.
impl std::ops::Deref for PaintStyle {
    type Target = Style;
    fn deref(&self) -> &Style {
        &self.style
    }
}

/// Role-level defaults supplied by a component author without installing a
/// theme recipe. Unlike ordinary unknown-family resolution, opting into
/// these defaults does not add the neutral recipe underneath them.
#[derive(Clone, Copy, Debug, Default)]
pub struct StyleDefaults<'a> {
    /// The authored role pair and modifiers before theme recipes and fallback.
    pub base: StylePatch,
    /// This author's whole targeted Mono manifest. Generic Mono rules still
    /// apply first; a theme's `mono_rules` replaces this slice, including `&[]`.
    pub mono: &'a [super::MonoRule],
}

impl<'a> StyleDefaults<'a> {
    /// Start with role-level defaults and no targeted Mono rules.
    #[must_use]
    pub const fn new(base: StylePatch) -> Self {
        Self { base, mono: &[] }
    }

    /// Supply the author's targeted Mono fallback manifest.
    #[must_use]
    pub const fn mono(mut self, rules: &'a [super::MonoRule]) -> Self {
        self.mono = rules;
        self
    }
}

/// The result of a style query.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Resolved {
    /// The style, with colours bound; apply over the inherited surface
    /// carrier with `resolved.over(inherited)` (§22 R‑9).
    pub style: PaintStyle,
    /// The glyph binding for the part. `Set` paints that glyph, `Inherit`
    /// leaves the caller's fallback in control, and `Clear` suppresses it
    /// (§5 R9).
    pub glyph: Slot<GlyphRole>,
    /// The part's size, if the recipe sets one.
    pub size: Option<u16>,
    /// The part's text alignment, if the recipe sets one.
    pub align: Option<Align>,
}

impl Resolved {
    /// This part's style layered over an inherited one — §11.3's final step,
    /// `Style::patch` semantics (modifier symmetry, §22 R‑9).
    ///
    /// Write `ui.fill(area, r.over(ui.surface_style()))`: the inherited style
    /// is the **left** operand, this part's style the right.
    #[must_use]
    pub fn over(self, inherited: impl Into<PaintStyle>) -> PaintStyle {
        inherited.into().patch(self.style)
    }

    /// The surface-independent half: glyph, size and alignment.
    #[must_use]
    pub const fn metrics(self) -> PartMetrics {
        PartMetrics {
            glyph: self.glyph,
            size: self.size,
            align: self.align,
        }
    }
}

/// The surface-independent half of resolution: everything §11.3 settles
/// before roles bind to colours. Available in `update`, where there is no
/// `Surface` (Adjudication N2).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PartMetrics {
    /// The glyph binding for the part. `Set` paints that glyph, `Inherit`
    /// leaves the caller's fallback in control, and `Clear` suppresses it
    /// (§5 R9).
    pub glyph: Slot<GlyphRole>,
    /// The part's size, if the recipe sets one.
    pub size: Option<u16>,
    /// The part's text alignment, if the recipe sets one.
    pub align: Option<Align>,
}

impl From<Resolved> for PartMetrics {
    fn from(r: Resolved) -> Self {
        r.metrics()
    }
}

/// The metrics carried by an accumulated patch — the one place `Theme::resolve`
/// and `Theme::metrics` read `glyph`/`size`/`align`, so they cannot drift.
pub(crate) fn metrics_of(acc: &StylePatch) -> PartMetrics {
    PartMetrics {
        glyph: acc.glyph,
        size: acc.size.get(),
        align: acc.align.get(),
    }
}

/// Steps 1–5: accumulate the role-level patch.
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
    // Ordinary queries retain the neutral unknown-family contract.
    let (mut acc, variant) = apply_recipe(recipes.get_or_neutral(f), v, p, live, StylePatch::new());
    // §11.4: mono fallback is a private static layer. It follows ordinary
    // family/variant states and precedes all author overrides.
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

/// Resolve author defaults before recipes, Mono policy, and all explicit
/// overrides. It deliberately does not cache caller-supplied defaults by the
/// ordinary family/part key, which cannot identify their content or lifetime.
pub(crate) fn bind_defaults(
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

fn apply_explicit(
    theme: &Theme,
    (f, variant, p): (Family, Variant, Part),
    live: StateFlags,
    overlays: &[Overlay],
    mut acc: StylePatch,
) -> StylePatch {
    let recipes = &theme.recipes;
    // 4: theme-level global overrides — family-wide, then variant-specific
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
    // 5: overlay stack, outermost → innermost (short-circuits when empty)
    if !overlays.is_empty() {
        for ov in overlays {
            acc = ov.apply(acc, f, variant, p, live);
        }
    }
    acc
}

/// Compose a logical owner's style above child defaults but below explicit
/// child theme/scope overrides. This bounded path does not affect the normal
/// cached resolver. It is used for shared collection empty-state titles.
pub(crate) fn bind_inherited(
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
    // Clear (and a Reset token) removes the child's channel, exposing the
    // owner, rather than resurrecting the built-in default underneath it.
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

/// Bind a role to a colour. `Color::Reset` tokens mean "no colour".
pub(crate) fn bind_role(theme: &Theme, role: Role, surface: Surface) -> Option<Color> {
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
                MeterRole::FillRest => t.fill_rest,
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

/// Step 6 + binding: apply the per-instance patch and bind roles.
pub(crate) fn bind(
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
        style: PaintStyle::bound(style, acc.fg.get(), acc.bg.get(), surface),
        glyph: acc.glyph,
        size: m.size,
        align: m.align,
    }
}

/// Cache entries (§11.1 A3, §20.9-2): 256, unchanged.
const CACHE_SLOTS: usize = 256;

/// Ways per set. The entries are the same 256; they are grouped into
/// `CACHE_SLOTS / WAYS` sets so that two hot keys landing on one set do not
/// evict each other on every access.
///
/// A **one-way** table of 256 entries cannot meet §16.6's ≥ 90 % hit rate
/// for a realistic frame: with `k` hot keys the expected number of
/// colliding pairs is `C(k,2)/256`, and a colliding pair in a hot loop misses
/// on *every* access. `style_resolve_10k_parts` touches 32 keys — 4 parts × 8
/// states — so ≈2 pairs collide by construction and the measured rate is
/// ≈87 %, whatever the hash. Two ways make a miss need three keys in one set
/// (`C(32,3)/128² ≈ 0.3`), which is what makes the memo's health assertable.
/// The array shape, the single construction-time allocation and the
/// generation stamp are unchanged.
const WAYS: usize = 2;
const CACHE_SETS: usize = CACHE_SLOTS / WAYS;

/// Allocation-free memo of steps 1–5, keyed by a 64-bit mix of
/// `(Family, Variant, Part, StateFlags, overlay-stack hash)` and cleared by
/// a generation stamp rather than by zeroing. Two-way set-associative with
/// insert-at-most-recent replacement.
#[derive(Clone, Debug)]
pub(crate) struct StyleCache {
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

    /// Invalidate every entry (theme change, new frame).
    ///
    /// The stamp must not wrap silently: `wrapping_add(1).max(1)` returns to 1
    /// after 2³² clears, at which point a slot still stamped with the original
    /// generation 1 becomes a false hit serving a stale `StylePatch`. At
    /// `u32::MAX` the slots are filled and the stamp restarts at 1 — one
    /// comparison per frame, and the 256-entry fill runs once per 2³² frames
    /// (§20.9-2, Adjudication O1).
    pub(crate) fn clear(&mut self) {
        if self.generation == u32::MAX {
            self.slots.fill((0, 0, StylePatch::new()));
            self.generation = 1;
        } else {
            self.generation = self.generation.saturating_add(1);
        }
    }

    /// `(hits, misses)` since construction. Promoted from `#[cfg(test)]` by
    /// adjudication 2.8: the memo's hit rate is the binding assertion that
    /// replaces the per-query ns ratio, so the harness must be able to read it.
    #[cfg(any(test, feature = "testing"))]
    pub(crate) const fn stats(&self) -> (u64, u64) {
        (self.hits, self.misses)
    }

    fn key(f: Family, v: Variant, p: Part, live: StateFlags, stack_hash: u64) -> u64 {
        let mut h = fnv1a(0xcbf2_9ce4_8422_2325, &f.raw().to_le_bytes());
        h = fnv1a(h, &v.raw().to_le_bytes());
        h = fnv1a(h, &p.raw().to_le_bytes());
        h = fnv1a(h, &live.bits().to_le_bytes());
        fnv1a(h, &stack_hash.to_le_bytes())
    }

    /// Steps 1–5 through the cache.
    #[expect(
        clippy::too_many_arguments,
        reason = "the §11.1 A3 memo key plus the theme and the stack"
    )]
    pub(crate) fn accumulate(
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
        // `| 1` keeps 0 free as the "empty entry" sentinel, so the set index
        // is taken from the bits above it.
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
        // insert at the most-recent way, pushing the previous ways down; the
        // last way is evicted
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

    /// Move way `w` of the set at `base` to way 0 (most recent).
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

/// The whole chain without a cache (tests, one-off queries).
#[expect(
    clippy::too_many_arguments,
    reason = "the six precedence inputs plus the theme and the surface"
)]
pub(crate) fn resolve_uncached(
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

    /// The precedence fixture, over an arbitrary base theme: a family base,
    /// a **family state rule** and a **variant base** whose bound colours are
    /// distinct under every built-in theme, so an ordering swap is visible.
    fn theme_over(base: Theme) -> Theme {
        let mut t = base;
        let r = t.recipes.get_mut(Family::custom("t"));
        r.default_variant = Variant::PRIMARY;
        // 1 family base + 3 family state rule
        let part = r.parts.entry(Part::LABEL);
        part.base = StylePatch::new().set_fg(Role::Fg(FgStep::Primary));
        part.when(StateFlags::FOCUSED, StylePatch::new().set_fg(Role::Warning));
        // 2 variant delta
        r.variant_mut(Variant::PRIMARY).entry(Part::LABEL).base =
            StylePatch::new().set_fg(Role::Accent);
        t
    }

    fn theme() -> Theme {
        theme_over(Theme::junie())
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
            // the fixture is only meaningful while these four differ
            assert_ne!(t.color.warning, t.color.accent);
            assert_ne!(t.color.success, t.color.warning);
            assert_ne!(t.color.info, t.color.success);
            assert_ne!(t.color.danger, t.color.info);
            let q = label_fg;
            // 2 over 1
            assert_eq!(q(&t, StateFlags::empty(), &[], None), Some(t.color.accent));
            // 3 over 2 — the family's state rule beats the variant's base
            assert_eq!(q(&t, StateFlags::FOCUSED, &[], None), Some(t.color.warning));
            // 4 over 3
            let mut parts: PartMap<PartRecipe> = PartMap::new();
            parts.entry(Part::LABEL).base = StylePatch::new().set_fg(Role::Success);
            t.recipes.push_override(GlobalOverride {
                family: f,
                variant: None,
                parts,
            });
            assert_eq!(q(&t, StateFlags::FOCUSED, &[], None), Some(t.color.success));
            // 5 over 4
            static OV: [OverlayRule; 1] = [(
                Family::custom("t"),
                Variant::PRIMARY,
                Part::LABEL,
                StateFlags::empty(),
                StylePatch::new().set_fg(Role::Info),
            )];
            let ov = Overlay::new(&OV);
            assert_eq!(q(&t, StateFlags::FOCUSED, &[ov], None), Some(t.color.info));
            // 6 over 5
            let inst = StylePatch::new().set_fg(Role::Danger);
            assert_eq!(
                q(&t, StateFlags::FOCUSED, &[ov], Some(&inst)),
                Some(t.color.danger)
            );
            // outer → inner: the inner overlay wins
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
            // no state: the variant base shows
            assert_eq!(
                label_fg(&t, StateFlags::empty(), &[], None),
                Some(t.color.accent)
            );
            // a *family* state rule is precedence 3 and outranks the variant
            // base at precedence 2, even though the variant is more specific
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
        // equal specificity: the family's rule is applied first, so the
        // variant's rule of the same specificity wins the slot
        assert_eq!(q(StateFlags::FOCUSED).fg, Some(t.color.success));
        assert_eq!(q(StateFlags::FOCUSED).bg, Some(t.color.accent_tint));
        // the family's 2-flag rule is applied *after* the variant's 1-flag
        // rule, which only a merged specificity order can produce
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
        // the neutral recipe is row-like: a real foreground and background
        assert_eq!(container.style.fg, Some(t.color.fg[0]));
        assert_eq!(container.style.bg, Some(t.bg(Surface::Canvas)));
        // and its state rules apply, so a custom family is distinguishable
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
        // the same role resolves against the surface passed at bind time
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
        // the role-level merge law and Style::patch agree on the modifier set
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

    /// §20.9-2 (Adjudication O1): the generation stamp must not wrap onto a
    /// live entry. A slot seeded at generation 1 must not be served again once
    /// the stamp has been round-tripped through `u32::MAX`.
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

        // one clear short of the wrap: the stamp restarts at 1, and the slot
        // seeded at generation 1 is still in the array
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
