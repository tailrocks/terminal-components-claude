//! Style resolution (`COMPONENT_ARCHITECTURE.md` §11.3, §20.9-1/-2/-4).
//!
//! Precedence, lowest → highest: family base, variant delta, state rules (by
//! specificity), theme-level global override, scope overlay stack (outermost
//! → innermost), per-instance patch. Then, and only then, roles bind to
//! colours against `(theme.color, surface, theme.capability)`. Steps 1–5 are
//! memoised in a statically sized direct-mapped cache keyed by a 64-bit mix.

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
    let variant = recipes.get(f).map_or(v, |r| {
        if v == Variant::DEFAULT {
            r.default_variant
        } else {
            v
        }
    });
    if let Some(r) = recipes.get(f) {
        // 1 + 3: family base part and its state rules
        if let Some(part) = r.parts.get(p) {
            acc = part.apply(acc, live);
        }
        // 2 (+ its own state rules): the variant delta
        if let Some(part) = r.variant(variant).and_then(|m| m.get(p)) {
            acc = part.apply(acc, live);
        }
    }
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
    Resolved {
        style,
        glyph: acc.glyph.get(),
        size: acc.size.get(),
        align: acc.align.get(),
    }
}

/// Number of direct-mapped cache slots (§11.1 A3, §20.9-2).
const CACHE_SLOTS: usize = 256;

/// Allocation-free memo of steps 1–5, keyed by a 64-bit mix of
/// `(Family, Variant, Part, StateFlags, overlay-stack hash)` and cleared by
/// a generation stamp rather than by zeroing.
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
    pub(crate) fn clear(&mut self) {
        self.generation = self.generation.wrapping_add(1).max(1);
    }

    #[cfg(test)]
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
        let idx = (key as usize) % CACHE_SLOTS;
        if let Some((k, g, patch)) = self.slots.get(idx)
            && *k == key
            && *g == self.generation
        {
            self.hits = self.hits.wrapping_add(1);
            return *patch;
        }
        let patch = accumulate(theme, f, v, p, live, overlays);
        if let Some(slot) = self.slots.get_mut(idx) {
            *slot = (key, self.generation, patch);
        }
        self.misses = self.misses.wrapping_add(1);
        patch
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

    fn theme() -> Theme {
        let mut t = Theme::junie();
        let r = t.recipes.get_mut(Family::custom("t"));
        r.default_variant = Variant::PRIMARY;
        // 1 family base + 3 state rule
        let base = r.parts.entry(Part::LABEL);
        base.base = StylePatch::new().set_fg(Role::Fg(FgStep::Primary));
        base.when(StateFlags::FOCUSED, StylePatch::new().set_fg(Role::Focus));
        // 2 variant delta
        r.variant_mut(Variant::PRIMARY).entry(Part::LABEL).base =
            StylePatch::new().set_fg(Role::Accent);
        t
    }

    #[test]
    fn precedence_family_then_variant_then_state_then_global_then_scope_then_instance() {
        let mut t = theme();
        let f = Family::custom("t");
        let q = |t: &Theme, live, ovs: &[Overlay], inst: Option<&StylePatch>| {
            resolve_uncached(
                t,
                f,
                Variant::DEFAULT,
                Part::LABEL,
                live,
                Surface::Canvas,
                ovs,
                inst,
            )
            .style
            .fg
        };
        // 2 over 1
        assert_eq!(q(&t, StateFlags::empty(), &[], None), Some(t.color.accent));
        // 3 over 2
        assert_eq!(q(&t, StateFlags::FOCUSED, &[], None), Some(t.color.focus));
        // 4 over 3
        let mut parts: PartMap<PartRecipe> = PartMap::new();
        parts.entry(Part::LABEL).base = StylePatch::new().set_fg(Role::Warning);
        t.recipes.push_override(GlobalOverride {
            family: f,
            variant: None,
            parts,
        });
        assert_eq!(q(&t, StateFlags::FOCUSED, &[], None), Some(t.color.warning));
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
}
