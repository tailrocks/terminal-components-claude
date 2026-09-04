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

/// The result of a style query.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Resolved {
    /// The style, with colours bound; apply over the inherited surface
    /// style with `inherited.patch(resolved.style)` (§22 R‑9).
    pub style: Style,
    /// The glyph the part must paint when `Some` (§5 R9).
    pub glyph: Option<GlyphRole>,
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
    pub fn over(self, inherited: Style) -> Style {
        inherited.patch(self.style)
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
    /// The glyph the part must paint when `Some` (§5 R9).
    pub glyph: Option<GlyphRole>,
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
        glyph: acc.glyph.get(),
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
    let mut acc = StylePatch::new();
    let recipes = &theme.recipes;
    // A family nobody declared resolves through the neutral recipe (§11.2).
    let r = recipes.get_or_neutral(f);
    let variant = if v == Variant::DEFAULT {
        r.default_variant
    } else {
        v
    };
    let fam = r.parts.get(p);
    let var = r.variant(variant).and_then(|m| m.get(p));
    // 1: the family base
    if let Some(part) = fam {
        acc = part.apply_base(acc);
    }
    // 2: the variant delta's base
    if let Some(part) = var {
        acc = part.apply_base(acc);
    }
    // 3: family and variant state rules are one level, merged in ascending
    //    specificity with the family's rule first on a tie
    acc = super::recipe::merge_states(
        acc,
        fam.map_or(&[][..], |x| &x.states),
        var.map_or(&[][..], |x| &x.states),
        live,
    );
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

/// Bind a role to a colour. `Color::Reset` tokens mean "no colour".
pub(crate) fn bind_role(theme: &Theme, role: Role, surface: Surface) -> Option<Color> {
    let c = &theme.color;
    let fg = |i: usize| c.fg.get(i).copied().unwrap_or(Color::Reset);
    let color = match role {
        Role::CurrentSurface => theme.bg(surface),
        Role::RaisedSurface => theme.bg(theme.raise(surface)),
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
        style,
        glyph: m.glyph,
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
        assert_eq!(focused.glyph, Some(GlyphRole::FocusBar));
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
        let out = inherited.patch(r.style);
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
