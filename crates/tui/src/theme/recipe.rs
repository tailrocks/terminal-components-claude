//! Families, variants, recipes, overlays and the recipe editor
//! (`COMPONENT_ARCHITECTURE.md` §11.2–§11.3, §17.0 A5, §20.9-1).
//!
//! State rules are stored **pre-sorted by specificity** (`when.count_ones()`
//! ascending, ties by declaration order) at recipe-build time, so resolution
//! is a single allocation-free scan.

use core::fmt;

use super::glyph::GlyphRole;
use super::patch::{Slot, StateRule, StylePatch};
use crate::id::{Part, fnv1a};
use crate::response::StateFlags;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

macro_rules! newtype_u16 {
    ($(#[$m:meta])* $name:ident, $prefix:literal, { $( $(#[$cm:meta])* $c:ident = $v:expr ),* $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(u16);

        impl $name {
            $( $(#[$cm])* pub const $c: $name = $name($v); )*

            /// Every library constant, in declaration order.
            pub const ALL: &'static [$name] = &[ $( $name::$c ),* ];

            /// A custom value named by a downstream author; lands in the high range.
            pub const fn custom(name: &'static str) -> $name {
                let h = fnv1a(FNV_OFFSET, name.as_bytes());
                $name(0x8000 | ((h as u16) & 0x7FFF))
            }

            /// The library name, or `None` for a custom value.
            pub const fn name(self) -> Option<&'static str> {
                match self {
                    $( $name::$c => Some(stringify!($c)), )*
                    _ => None,
                }
            }

            /// The raw number.
            pub const fn raw(self) -> u16 {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.name() {
                    Some(n) => write!(f, "{}::{n}", $prefix),
                    None => write!(f, "{}::custom(#{:04x})", $prefix, self.0),
                }
            }
        }
    };
}

newtype_u16! {
    /// A component family, the first key of a recipe.
    Family, "Family", {
        /// Buttons.
        BUTTON = 0,
        /// Checkbox, toggle, radio group.
        CHOICE = 1,
        /// Chips.
        CHIP = 2,
        /// Field chrome.
        FIELD = 3,
        /// Text input.
        INPUT = 4,
        /// Text area.
        TEXTAREA = 5,
        /// Code editor.
        CODE = 6,
        /// Select.
        SELECT = 7,
        /// List.
        LIST = 8,
        /// Tree.
        TREE = 9,
        /// Grid.
        GRID = 10,
        /// Props.
        PROPS = 11,
        /// Steps.
        STEPS = 12,
        /// Tabs.
        TABS = 13,
        /// Panel.
        PANEL = 14,
        /// Split pane.
        SPLIT = 15,
        /// Scrollbar.
        SCROLLBAR = 16,
        /// Text viewport.
        VIEWPORT = 17,
        /// Diff view.
        DIFF = 18,
        /// Dialog.
        DIALOG = 19,
        /// Overlay chrome.
        OVERLAY = 20,
        /// Menus.
        MENU = 21,
        /// Picker.
        PICKER = 22,
        /// Completion.
        COMPLETION = 23,
        /// Form.
        FORM = 24,
        /// Help overlay.
        HELP = 25,
        /// Wizard.
        WIZARD = 26,
        /// Status bar.
        STATUSBAR = 27,
        /// Hint bar.
        HINTBAR = 28,
        /// Progress.
        PROGRESS = 29,
        /// Meter.
        METER = 30,
        /// Empty state.
        EMPTY = 31,
        /// Brand lockup.
        BRAND = 32,
        /// Key hint.
        KEYHINT = 33,
        /// Below-minimum-size notice.
        TOO_SMALL = 34,
    }
}

newtype_u16! {
    /// A variant within a family, the second key of a recipe.
    Variant, "Variant", {
        /// The family's default look.
        DEFAULT = 0,
        /// Primary emphasis.
        PRIMARY = 1,
        /// Secondary emphasis.
        SECONDARY = 2,
        /// Subtle.
        SUBTLE = 3,
        /// Destructive.
        DANGER = 4,
        /// Toggle.
        TOGGLE = 5,
        /// Quiet.
        QUIET = 6,
        /// Ghost.
        GHOST = 7,
    }
}

/// A small sorted map keyed by [`Part`].
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PartMap<T> {
    entries: Vec<(Part, T)>,
}

impl<T> PartMap<T> {
    /// An empty map.
    pub const fn new() -> Self {
        PartMap {
            entries: Vec::new(),
        }
    }

    /// Look up a part.
    pub fn get(&self, p: Part) -> Option<&T> {
        self.entries
            .binary_search_by_key(&p, |(k, _)| *k)
            .ok()
            .and_then(|i| self.entries.get(i))
            .map(|(_, v)| v)
    }

    /// Look up a part mutably.
    pub fn get_mut(&mut self, p: Part) -> Option<&mut T> {
        match self.entries.binary_search_by_key(&p, |(k, _)| *k) {
            Ok(i) => self.entries.get_mut(i).map(|(_, v)| v),
            Err(_) => None,
        }
    }

    /// Insert or replace.
    pub fn insert(&mut self, p: Part, v: T) {
        match self.entries.binary_search_by_key(&p, |(k, _)| *k) {
            Ok(i) => {
                if let Some(slot) = self.entries.get_mut(i) {
                    slot.1 = v;
                }
            }
            Err(i) => self.entries.insert(i, (p, v)),
        }
    }

    /// The entry for `p`, created with `T::default()` if absent.
    #[expect(
        clippy::indexing_slicing,
        reason = "`i` is either the index binary_search found or the index Vec::insert(i, _) just \
                  filled, so it is in range by construction"
    )]
    pub fn entry(&mut self, p: Part) -> &mut T
    where
        T: Default,
    {
        let i = match self.entries.binary_search_by_key(&p, |(k, _)| *k) {
            Ok(i) => i,
            Err(i) => {
                self.entries.insert(i, (p, T::default()));
                i
            }
        };
        &mut self.entries[i].1
    }

    /// Iterate in part order.
    pub fn iter(&self) -> impl Iterator<Item = (Part, &T)> + '_ {
        self.entries.iter().map(|(k, v)| (*k, v))
    }

    /// Iterate mutably in part order.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Part, &mut T)> + '_ {
        self.entries.iter_mut().map(|(k, v)| (*k, v))
    }

    /// The number of parts.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// One part's recipe: a base patch plus state rules pre-sorted by specificity.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PartRecipe {
    /// The base patch.
    pub base: StylePatch,
    /// The state rules, sorted by `when.count_ones()`, ties by insertion.
    pub states: Vec<StateRule>,
    /// The part's glyph.
    pub glyph: Slot<GlyphRole>,
    /// The part's size.
    pub size: Slot<u16>,
}

impl PartRecipe {
    /// Insert a state rule at its specificity position (after rules of
    /// equal specificity, so declaration order breaks ties).
    pub fn when(&mut self, when: StateFlags, patch: StylePatch) -> &mut Self {
        let rule = StateRule { when, patch };
        let spec = rule.specificity();
        let pos = self
            .states
            .iter()
            .position(|r| r.specificity() > spec)
            .unwrap_or(self.states.len());
        self.states.insert(pos, rule);
        self
    }

    /// Accumulate the base patch and the `glyph`/`size` slots — §11.3
    /// precedence steps 1 (family base) and 2 (variant delta). State rules
    /// are step 3 and are applied afterwards, over *every* base, by
    /// [`PartRecipe::apply_states`].
    pub fn apply_base(&self, acc: StylePatch) -> StylePatch {
        let mut acc = acc.merge(self.base);
        acc.glyph = self.glyph.over(acc.glyph);
        acc.size = self.size.over(acc.size);
        acc
    }

    /// Accumulate every matching state rule — §11.3 precedence step 3.
    /// The rules are stored pre-sorted by specificity, so this is one scan.
    pub fn apply_states(&self, acc: StylePatch, live: StateFlags) -> StylePatch {
        let mut acc = acc;
        for r in &self.states {
            if r.matches(live) {
                acc = acc.merge(r.patch);
            }
        }
        acc
    }

    /// Base then state rules, for a recipe that is a whole precedence level
    /// on its own (a global override, §11.3 step 4). The family/variant pair
    /// must **not** use this: their bases both precede both their state rule
    /// sets, so they go through `apply_base` and the crate-internal
    /// specificity merge.
    pub fn apply(&self, acc: StylePatch, live: StateFlags) -> StylePatch {
        self.apply_states(self.apply_base(acc), live)
    }

    /// Merge another recipe over this one.
    pub fn merge_from(&mut self, other: &PartRecipe) {
        self.base = self.base.merge(other.base);
        self.glyph = other.glyph.over(self.glyph);
        self.size = other.size.over(self.size);
        for r in &other.states {
            self.when(r.when, r.patch);
        }
    }
}

/// Merge a family's and a variant's state rules over `acc` in one pass,
/// ascending by specificity, the family's rule first on a tie — §11.3
/// precedence step 3, which is a single level spanning both rule lists.
///
/// Both slices are stored pre-sorted (`PartRecipe::when`), so this is a
/// stable two-way merge: allocation-free and `O(n + m)`.
pub(crate) fn merge_states(
    acc: StylePatch,
    family: &[StateRule],
    variant: &[StateRule],
    live: StateFlags,
) -> StylePatch {
    let mut acc = acc;
    let mut fam = family.iter().peekable();
    let mut var = variant.iter().peekable();
    loop {
        let take_family = match (fam.peek(), var.peek()) {
            (None, None) => break,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (Some(a), Some(b)) => a.specificity() <= b.specificity(),
        };
        let rule = if take_family { fam.next() } else { var.next() };
        if let Some(r) = rule
            && r.matches(live)
        {
            acc = acc.merge(r.patch);
        }
    }
    acc
}

/// A family's recipe: its default variant, base parts and variant deltas.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Recipe {
    /// The variant used when an instance names none.
    pub default_variant: Variant,
    /// The base parts.
    pub parts: PartMap<PartRecipe>,
    /// Variant deltas, applied over the base.
    pub variants: Vec<(Variant, PartMap<PartRecipe>)>,
}

impl Default for Recipe {
    fn default() -> Self {
        Recipe {
            default_variant: Variant::DEFAULT,
            parts: PartMap::new(),
            variants: Vec::new(),
        }
    }
}

impl Recipe {
    /// The variant delta for `v`, if defined.
    pub fn variant(&self, v: Variant) -> Option<&PartMap<PartRecipe>> {
        self.variants.iter().find(|(k, _)| *k == v).map(|(_, m)| m)
    }

    /// The variant delta for `v`, created if absent.
    #[expect(
        clippy::indexing_slicing,
        reason = "`pos` is either the index `position` found or the last index after a push, so \
                  it is in range by construction"
    )]
    pub fn variant_mut(&mut self, v: Variant) -> &mut PartMap<PartRecipe> {
        let pos = if let Some(i) = self.variants.iter().position(|(k, _)| *k == v) {
            i
        } else {
            self.variants.push((v, PartMap::new()));
            self.variants.len().saturating_sub(1)
        };
        &mut self.variants[pos].1
    }
}

/// A theme-level global override: family, optional variant, parts.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct GlobalOverride {
    pub(crate) family: Family,
    pub(crate) variant: Option<Variant>,
    pub(crate) parts: PartMap<PartRecipe>,
}

/// Every family's recipe plus the global overrides; built once at theme
/// construction, only read at resolution.
///
/// # Invariant: the resolvable set is `by_family` ∪ `neutral`
///
/// Resolution never reads `by_family` alone. [`Recipes::get_or_neutral`] —
/// the single entry point `accumulate` uses — falls back to `neutral` for
/// every family `define_family` never declared, and `Family::custom` is a
/// `const fn` with no declaration event, so that fallback is a *reachable*
/// painting path, not a placeholder.
///
/// The static mono fallback layer keys on the requested family rather than
/// enumerating storage. Undeclared custom families therefore receive generic
/// non-colour state signals while their authored base remains neutral.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Recipes {
    by_family: Vec<(Family, Recipe)>,
    overrides: Vec<GlobalOverride>,
    neutral: Recipe,
}

impl Default for Recipes {
    fn default() -> Self {
        Recipes {
            by_family: Vec::new(),
            overrides: Vec::new(),
            neutral: super::builtin::neutral_recipe(),
        }
    }
}

impl Recipes {
    /// The recipe for a family.
    pub fn get(&self, f: Family) -> Option<&Recipe> {
        self.by_family
            .binary_search_by_key(&f, |(k, _)| *k)
            .ok()
            .and_then(|i| self.by_family.get(i))
            .map(|(_, r)| r)
    }

    /// The recipe a family that `define_family` never declared starts from
    /// (§11.2): the neutral row-like set `CONTAINER / GUTTER / MARKER /
    /// LABEL / META …`, so `Family::custom("x")` renders instead of
    /// resolving to an empty style. `define_family` replaces it.
    pub fn neutral(&self) -> &Recipe {
        &self.neutral
    }

    /// The recipe for a family, falling back to [`Recipes::neutral`].
    pub fn get_or_neutral(&self, f: Family) -> &Recipe {
        self.get(f).unwrap_or(&self.neutral)
    }

    /// The recipe for a family, created if absent.
    #[expect(
        clippy::indexing_slicing,
        reason = "`i` is either the index binary_search found or the index Vec::insert(i, _) just \
                  filled, so it is in range by construction"
    )]
    pub fn get_mut(&mut self, f: Family) -> &mut Recipe {
        let i = match self.by_family.binary_search_by_key(&f, |(k, _)| *k) {
            Ok(i) => i,
            Err(i) => {
                self.by_family.insert(i, (f, Recipe::default()));
                i
            }
        };
        &mut self.by_family[i].1
    }

    /// Iterate families in order.
    pub fn iter(&self) -> impl Iterator<Item = (Family, &Recipe)> + '_ {
        self.by_family.iter().map(|(k, r)| (*k, r))
    }

    /// Iterate declared families mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Family, &mut Recipe)> + '_ {
        self.by_family.iter_mut().map(|(k, r)| (*k, r))
    }

    /// The number of families.
    pub fn len(&self) -> usize {
        self.by_family.len()
    }

    /// Whether no family is defined.
    pub fn is_empty(&self) -> bool {
        self.by_family.is_empty()
    }

    pub(crate) fn overrides(&self) -> &[GlobalOverride] {
        &self.overrides
    }

    pub(crate) fn push_override(&mut self, o: GlobalOverride) {
        self.overrides.push(o);
    }
}

/// Editor for one part inside a [`RecipeEdit`].
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PartEdit {
    recipe: PartRecipe,
}

impl PartEdit {
    /// Set the base patch.
    pub fn base(&mut self, p: StylePatch) -> &mut Self {
        self.recipe.base = self.recipe.base.merge(p);
        self
    }

    /// Add a state rule (stored pre-sorted, §20.9-1).
    pub fn when(&mut self, s: StateFlags, p: StylePatch) -> &mut Self {
        self.recipe.when(s, p);
        self
    }

    /// Set the glyph.
    pub fn glyph(&mut self, g: GlyphRole) -> &mut Self {
        self.recipe.glyph = Slot::Set(g);
        self
    }

    /// Set the size.
    pub fn size(&mut self, n: u16) -> &mut Self {
        self.recipe.size = Slot::Set(n);
        self
    }

    /// The edited recipe.
    pub fn recipe(&self) -> &PartRecipe {
        &self.recipe
    }
}

/// Editor passed to `Theme::override_family` and friends.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RecipeEdit {
    default_variant: Option<Variant>,
    parts: PartMap<PartEdit>,
}

impl RecipeEdit {
    /// Set the default variant.
    pub fn default_variant(&mut self, v: Variant) -> &mut Self {
        self.default_variant = Some(v);
        self
    }

    /// Edit a part.
    pub fn part(&mut self, p: Part) -> &mut PartEdit {
        self.parts.entry(p)
    }

    /// The default variant, if set.
    pub const fn default_variant_set(&self) -> Option<Variant> {
        self.default_variant
    }

    /// The edited parts as recipes.
    pub fn into_parts(self) -> PartMap<PartRecipe> {
        let mut out = PartMap::new();
        for (p, e) in self.parts.iter() {
            out.insert(p, e.recipe.clone());
        }
        out
    }
}

/// One overlay rule.
pub type OverlayRule = (Family, Variant, Part, StateFlags, StylePatch);

/// A scoped override: borrowed, `const`-constructible, never mutates the theme.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Overlay {
    rules: &'static [OverlayRule],
}

impl Overlay {
    /// The empty overlay.
    pub const EMPTY: Overlay = Overlay { rules: &[] };

    /// An overlay over a static rule slice.
    pub const fn new(rules: &'static [OverlayRule]) -> Overlay {
        Overlay { rules }
    }

    /// The rules.
    pub const fn rules(&self) -> &'static [OverlayRule] {
        self.rules
    }

    /// Apply every matching rule over `acc`.
    pub fn apply(
        &self,
        acc: StylePatch,
        f: Family,
        v: Variant,
        p: Part,
        live: StateFlags,
    ) -> StylePatch {
        let mut acc = acc;
        for (rf, rv, rp, when, patch) in self.rules {
            if *rf == f && *rv == v && *rp == p && live.contains(*when) {
                acc = acc.merge(*patch);
            }
        }
        acc
    }

    /// A stable hash of the rule slice's identity (its address and length).
    pub fn hash(&self) -> u64 {
        let ptr = self.rules.as_ptr() as usize as u64;
        let len = self.rules.len() as u64;
        fnv1a(fnv1a(FNV_OFFSET, &ptr.to_le_bytes()), &len.to_le_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::role::Role;

    #[test]
    fn state_rules_are_stored_in_specificity_order() {
        let mut r = PartRecipe::default();
        r.when(
            StateFlags::HOVERED | StateFlags::PRESSED,
            StylePatch::new().set_bg(Role::Accent),
        );
        r.when(StateFlags::FOCUSED, StylePatch::new().set_fg(Role::Focus));
        r.when(StateFlags::empty(), StylePatch::new().set_fg(Role::Info));
        let specs: Vec<u32> = r.states.iter().map(StateRule::specificity).collect();
        assert_eq!(specs, vec![0, 1, 2]);
    }

    #[test]
    fn state_rules_tie_break_by_declaration_order() {
        let mut r = PartRecipe::default();
        r.when(
            StateFlags::SELECTED,
            StylePatch::new().set_glyph(GlyphRole::Chosen),
        );
        r.when(
            StateFlags::CHECKED,
            StylePatch::new().set_glyph(GlyphRole::Checked),
        );
        let live = StateFlags::SELECTED | StateFlags::CHECKED;
        let acc = r.apply(StylePatch::new(), live);
        assert_eq!(acc.glyph, Slot::Set(GlyphRole::Checked));
        let acc = r.apply(StylePatch::new(), StateFlags::SELECTED);
        assert_eq!(acc.glyph, Slot::Set(GlyphRole::Chosen));
    }

    #[test]
    fn custom_family_and_variant_round_trip() {
        let f = Family::custom("segmented");
        let v = Variant::custom("outline");
        assert!(f.raw() >= 0x8000 && v.raw() >= 0x8000);
        assert_eq!(f, Family::custom("segmented"));
        assert_eq!(format!("{:?}", Family::BUTTON), "Family::BUTTON");
        assert_eq!(
            format!("{v:?}"),
            format!("Variant::custom(#{:04x})", v.raw())
        );
        let mut rs = Recipes::default();
        rs.get_mut(f).default_variant = v;
        rs.get_mut(f).variant_mut(v).entry(Part::LABEL).base =
            StylePatch::new().set_fg(Role::Accent);
        assert_eq!(rs.get(f).map(|r| r.default_variant), Some(v));
        assert!(
            rs.get(f)
                .and_then(|r| r.variant(v))
                .and_then(|m| m.get(Part::LABEL))
                .is_some()
        );
        assert_eq!(rs.len(), 1);
    }

    #[test]
    fn part_map_stays_sorted() {
        let mut m: PartMap<u8> = PartMap::new();
        m.insert(Part::THUMB, 1);
        m.insert(Part::BORDER, 2);
        *m.entry(Part::LABEL) = 3;
        m.insert(Part::BORDER, 4);
        let keys: Vec<Part> = m.iter().map(|(k, _)| k).collect();
        assert_eq!(keys, vec![Part::BORDER, Part::LABEL, Part::THUMB]);
        assert_eq!(m.get(Part::BORDER), Some(&4));
        assert_eq!(m.get_mut(Part::THUMB).copied(), Some(1));
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn overlay_applies_only_matching_rules_and_hashes_by_identity() {
        static RULES: [OverlayRule; 1] = [(
            Family::BUTTON,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::FOCUSED,
            StylePatch::new().set_fg(Role::Warning),
        )];
        let ov = Overlay::new(&RULES);
        let hit = ov.apply(
            StylePatch::new(),
            Family::BUTTON,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::FOCUSED,
        );
        assert_eq!(hit.fg, Slot::Set(Role::Warning));
        let miss = ov.apply(
            StylePatch::new(),
            Family::BUTTON,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::empty(),
        );
        assert!(miss.is_empty());
        assert_ne!(ov.hash(), Overlay::EMPTY.hash());
        assert_eq!(ov.hash(), Overlay::new(&RULES).hash());
    }
}
