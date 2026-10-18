#![doc = r" Families, variants, recipes, overlays and the recipe editor"]
#![doc = r" (`COMPONENT_ARCHITECTURE.md` §11.2–§11.3, §17.0 A5, §20.9-1)."]
#![doc = r""]
#![doc = r" State rules are stored **pre-sorted by specificity** (`when.count_ones()`"]
#![doc = r" ascending, ties by declaration order) at recipe-build time, so resolution"]
#![doc = r" is a single allocation-free scan."]

use core::fmt;

use super::downgrade::MonoRule;
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

            #[doc = r" Every library constant, in declaration order."]
            pub const ALL: &'static [$name] = &[ $( $name::$c ),* ];

            #[doc = r" A custom value named by a downstream author; lands in the high range."]
            pub const fn custom(name: &'static str) -> $name {
                let h = fnv1a(FNV_OFFSET, name.as_bytes());
                $name(0x8000 | ((h as u16) & 0x7FFF))
            }

            #[doc = r" The library name, or `None` for a custom value."]
            pub const fn name(self) -> Option<&'static str> {
                match self {
                    $( $name::$c => Some(stringify!($c)), )*
                    _ => None,
                }
            }

            #[doc = r" The raw number."]
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
    #[doc = r" A component family, the first key of a recipe."]
    Family, "Family", {
        #[doc = r" Buttons."]
        BUTTON = 0,
        #[doc = r" Checkbox, toggle, radio group."]
        CHOICE = 1,
        #[doc = r" Chips."]
        CHIP = 2,
        #[doc = r" Field chrome."]
        FIELD = 3,
        #[doc = r" Text input."]
        INPUT = 4,
        #[doc = r" Text area."]
        TEXTAREA = 5,
        #[doc = r" Code editor."]
        CODE = 6,
        #[doc = r" Select."]
        SELECT = 7,
        #[doc = r" List."]
        LIST = 8,
        #[doc = r" Tree."]
        TREE = 9,
        #[doc = r" Grid."]
        GRID = 10,
        #[doc = r" Props."]
        PROPS = 11,
        #[doc = r" Steps."]
        STEPS = 12,
        #[doc = r" Tabs."]
        TABS = 13,
        #[doc = r" Panel."]
        PANEL = 14,
        #[doc = r" Split pane."]
        SPLIT = 15,
        #[doc = r" Scrollbar."]
        SCROLLBAR = 16,
        #[doc = r" Text viewport."]
        VIEWPORT = 17,
        #[doc = r" Diff view."]
        DIFF = 18,
        #[doc = r" Dialog."]
        DIALOG = 19,
        #[doc = r" Overlay chrome."]
        OVERLAY = 20,
        #[doc = r" Menus."]
        MENU = 21,
        #[doc = r" Picker."]
        PICKER = 22,
        #[doc = r" Completion."]
        COMPLETION = 23,
        #[doc = r" Form."]
        FORM = 24,
        #[doc = r" Help overlay."]
        HELP = 25,
        #[doc = r" Wizard."]
        WIZARD = 26,
        #[doc = r" Status bar."]
        STATUSBAR = 27,
        #[doc = r" Hint bar."]
        HINTBAR = 28,
        #[doc = r" Progress."]
        PROGRESS = 29,
        #[doc = r" Meter."]
        METER = 30,
        #[doc = r" Empty state."]
        EMPTY = 31,
        #[doc = r" Brand lockup."]
        BRAND = 32,
        #[doc = r" Key hint."]
        KEYHINT = 33,
        #[doc = r" Below-minimum-size notice."]
        TOO_SMALL = 34,
    }
}

pub use termrock_core::variant::Variant;

#[doc = r" A small sorted map keyed by [`Part`]."]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PartMap<T> {
    entries: Vec<(Part, T)>,
}

impl<T> PartMap<T> {
    #[doc = r" An empty map."]
    pub const fn new() -> Self {
        PartMap {
            entries: Vec::new(),
        }
    }

    #[doc = r" Look up a part."]
    pub fn get(&self, p: Part) -> Option<&T> {
        self.entries
            .binary_search_by_key(&p, |(k, _)| *k)
            .ok()
            .and_then(|i| self.entries.get(i))
            .map(|(_, v)| v)
    }

    #[doc = r" Look up a part mutably."]
    pub fn get_mut(&mut self, p: Part) -> Option<&mut T> {
        match self.entries.binary_search_by_key(&p, |(k, _)| *k) {
            Ok(i) => self.entries.get_mut(i).map(|(_, v)| v),
            Err(_) => None,
        }
    }

    #[doc = r" Insert or replace."]
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

    #[doc = r" The entry for `p`, created with `T::default()` if absent."]
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

    #[doc = r" Iterate in part order."]
    pub fn iter(&self) -> impl Iterator<Item = (Part, &T)> + '_ {
        self.entries.iter().map(|(k, v)| (*k, v))
    }

    #[doc = r" Iterate mutably in part order."]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Part, &mut T)> + '_ {
        self.entries.iter_mut().map(|(k, v)| (*k, v))
    }

    #[doc = r" The number of parts."]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[doc = r" Whether the map is empty."]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[doc = r" One part's recipe: a base patch plus state rules pre-sorted by specificity."]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PartRecipe {
    #[doc = r" The base patch."]
    pub base: StylePatch,
    #[doc = r" The state rules, sorted by `when.count_ones()`, ties by insertion."]
    pub states: Vec<StateRule>,
    #[doc = r" The part's glyph."]
    pub glyph: Slot<GlyphRole>,
    #[doc = r" The part's size."]
    pub size: Slot<u16>,
}

impl PartRecipe {
    #[doc = r" Insert a state rule at its specificity position (after rules of"]
    #[doc = r" equal specificity, so declaration order breaks ties)."]
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

    #[doc = r" Accumulate the base patch and the `glyph`/`size` slots — §11.3"]
    #[doc = r" precedence steps 1 (family base) and 2 (variant delta). State rules"]
    #[doc = r" are step 3 and are applied afterwards, over *every* base, by"]
    #[doc = r" [`PartRecipe::apply_states`]."]
    pub fn apply_base(&self, acc: StylePatch) -> StylePatch {
        let mut acc = acc.merge(self.base);
        acc.glyph = self.glyph.over(acc.glyph);
        acc.size = self.size.over(acc.size);
        acc
    }

    #[doc = r" Accumulate every matching state rule — §11.3 precedence step 3."]
    #[doc = r" The rules are stored pre-sorted by specificity, so this is one scan."]
    pub fn apply_states(&self, acc: StylePatch, live: StateFlags) -> StylePatch {
        let mut acc = acc;
        for r in &self.states {
            if r.matches(live) {
                acc = acc.merge(r.patch);
            }
        }
        acc
    }

    #[doc = r" Base then state rules, for a recipe that is a whole precedence level"]
    #[doc = r" on its own (a global override, §11.3 step 4). The family/variant pair"]
    #[doc = r" must **not** use this: their bases both precede both their state rule"]
    #[doc = r" sets, so they go through `apply_base` and the crate-internal"]
    #[doc = r" specificity merge."]
    pub fn apply(&self, acc: StylePatch, live: StateFlags) -> StylePatch {
        self.apply_states(self.apply_base(acc), live)
    }

    #[doc = r" Merge another recipe over this one."]
    pub fn merge_from(&mut self, other: &PartRecipe) {
        self.base = self.base.merge(other.base);
        self.glyph = other.glyph.over(self.glyph);
        self.size = other.size.over(self.size);
        for r in &other.states {
            self.when(r.when, r.patch);
        }
    }
}

#[doc = r" Merge a family's and a variant's state rules over `acc` in one pass,"]
#[doc = r" ascending by specificity, the family's rule first on a tie — §11.3"]
#[doc = r" precedence step 3, which is a single level spanning both rule lists."]
#[doc = r""]
#[doc = r" Both slices are stored pre-sorted (`PartRecipe::when`), so this is a"]
#[doc = r" stable two-way merge: allocation-free and `O(n + m)`."]
pub fn merge_states(
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

#[doc = r" A family's recipe: its default variant, base parts and variant deltas."]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Recipe {
    #[doc = r" The variant used when an instance names none."]
    pub default_variant: Variant,
    #[doc = r" The base parts."]
    pub parts: PartMap<PartRecipe>,
    #[doc = r" Variant deltas, applied over the base."]
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
    #[doc = r" The variant delta for `v`, if defined."]
    pub fn variant(&self, v: Variant) -> Option<&PartMap<PartRecipe>> {
        self.variants.iter().find(|(k, _)| *k == v).map(|(_, m)| m)
    }

    #[doc = r" The variant delta for `v`, created if absent."]
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

#[doc = r" A theme-level global override: family, optional variant, parts."]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GlobalOverride {
    pub family: Family,
    pub variant: Option<Variant>,
    pub parts: PartMap<PartRecipe>,
}

#[doc = r" Every family's recipe plus the global overrides; built once at theme"]
#[doc = r" construction, only read at resolution."]
#[doc = r""]
#[doc = r" # Invariant: the resolvable set is `by_family` ∪ `neutral`"]
#[doc = r""]
#[doc = r" Resolution never reads `by_family` alone. [`Recipes::get_or_neutral`] —"]
#[doc = r" the single entry point `accumulate` uses — falls back to `neutral` for"]
#[doc = r" every family `define_family` never declared, and `Family::custom` is a"]
#[doc = r" `const fn` with no declaration event, so that fallback is a *reachable*"]
#[doc = r" painting path, not a placeholder."]
#[doc = r""]
#[doc = r" The static mono fallback layer keys on the requested family rather than"]
#[doc = r" enumerating storage. Undeclared custom families therefore receive generic"]
#[doc = r" non-colour state signals while their authored base remains neutral."]
#[doc = r""]
#[doc = r" # Author-supplied mono rules"]
#[doc = r""]
#[doc = r" `mono` holds the §11.4 family-targeted mono manifests a theme author set"]
#[doc = r" through [`ThemeBuilder::mono_rules`](super::ThemeBuilder::mono_rules). An"]
#[doc = r" entry is the **whole** targeted set for that family: it replaces the"]
#[doc = r" built-in targeted set rather than extending it, and an entry with an empty"]
#[doc = r#" slice means "this family adds nothing to the generic manifest". A family"#]
#[doc = r" with no entry keeps whatever the built-in table declares for it. The"]
#[doc = r" generic manifest — [`MONO_RULES_PER_FAMILY`](super::MONO_RULES_PER_FAMILY)"]
#[doc = r" rules — is family-independent and is never replaced."]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Recipes {
    by_family: Vec<(Family, Recipe)>,
    overrides: Vec<GlobalOverride>,
    neutral: Recipe,
    mono: Vec<(Family, Vec<MonoRule>)>,
}

impl Default for Recipes {
    fn default() -> Self {
        Recipes {
            by_family: Vec::new(),
            overrides: Vec::new(),
            neutral: super::builtin::neutral_recipe(),
            mono: Vec::new(),
        }
    }
}

impl Recipes {
    #[doc = r" The recipe for a family."]
    pub fn get(&self, f: Family) -> Option<&Recipe> {
        self.by_family
            .binary_search_by_key(&f, |(k, _)| *k)
            .ok()
            .and_then(|i| self.by_family.get(i))
            .map(|(_, r)| r)
    }

    #[doc = r" The recipe a family that `define_family` never declared resolves"]
    #[doc = r" through (§11.2) the neutral row-like set `CONTAINER / GUTTER / MARKER /"]
    #[doc = r#" LABEL / META …`, so `Family::custom("x")` renders instead of"#]
    #[doc = r" resolving to an empty style."]
    pub fn neutral(&self) -> &Recipe {
        &self.neutral
    }

    #[doc = r" The recipe for a family, falling back to [`Recipes::neutral`]."]
    pub fn get_or_neutral(&self, f: Family) -> &Recipe {
        self.get(f).unwrap_or(&self.neutral)
    }

    #[doc = r" The recipe for a family, created if absent."]
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

    #[doc = r" Iterate families in order."]
    pub fn iter(&self) -> impl Iterator<Item = (Family, &Recipe)> + '_ {
        self.by_family.iter().map(|(k, r)| (*k, r))
    }

    #[doc = r" Iterate declared families mutably."]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Family, &mut Recipe)> + '_ {
        self.by_family.iter_mut().map(|(k, r)| (*k, r))
    }

    #[doc = r" The number of families."]
    pub fn len(&self) -> usize {
        self.by_family.len()
    }

    #[doc = r" Whether no family is defined."]
    pub fn is_empty(&self) -> bool {
        self.by_family.is_empty()
    }

    #[doc = r" The author-supplied §11.4 mono fallback rules for `f`, if the theme"]
    #[doc = r" set any: `Some(&[])` is a deliberately emptied set, `None` means the"]
    #[doc = r" built-in table still owns this family."]
    #[doc = r""]
    #[doc = r" Read at resolution only when `capability.color == ColorLevel::Mono`."]
    #[doc = r" The generic manifest applies either way."]
    pub fn mono_rules(&self, f: Family) -> Option<&[MonoRule]> {
        if self.mono.is_empty() {
            return None;
        }
        self.mono
            .binary_search_by_key(&f, |(k, _)| *k)
            .ok()
            .and_then(|i| self.mono.get(i))
            .map(|(_, r)| r.as_slice())
    }

    #[doc = r" Replace `f`'s whole targeted mono manifest (`ThemeBuilder::mono_rules`)."]
    #[doc = r""]
    #[doc = r" Kept sorted by family so two themes built by the same calls in the same"]
    #[doc = r" order compare equal and fingerprint identically."]
    #[expect(
        clippy::indexing_slicing,
        reason = "`i` is either the index binary_search found or the index Vec::insert(i, _) just \
                  filled, so it is in range by construction"
    )]
    pub(crate) fn set_mono_rules(&mut self, f: Family, rules: Vec<MonoRule>) {
        let i = match self.mono.binary_search_by_key(&f, |(k, _)| *k) {
            Ok(i) => i,
            Err(i) => {
                self.mono.insert(i, (f, Vec::new()));
                i
            }
        };
        self.mono[i].1 = rules;
    }

    pub fn overrides(&self) -> &[GlobalOverride] {
        &self.overrides
    }

    pub(crate) fn push_override(&mut self, o: GlobalOverride) {
        self.overrides.push(o);
    }
}

#[doc = r" Editor for one part inside a [`RecipeEdit`]."]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PartEdit {
    recipe: PartRecipe,
}

impl PartEdit {
    #[doc = r" Set the base patch."]
    pub fn base(&mut self, p: StylePatch) -> &mut Self {
        self.recipe.base = self.recipe.base.merge(p);
        self
    }

    #[doc = r" Add a state rule (stored pre-sorted, §20.9-1)."]
    pub fn when(&mut self, s: StateFlags, p: StylePatch) -> &mut Self {
        self.recipe.when(s, p);
        self
    }

    #[doc = r" Set the glyph."]
    pub fn glyph(&mut self, g: GlyphRole) -> &mut Self {
        self.recipe.glyph = Slot::Set(g);
        self
    }

    #[doc = r" Set the size."]
    pub fn size(&mut self, n: u16) -> &mut Self {
        self.recipe.size = Slot::Set(n);
        self
    }

    #[doc = r" The edited recipe."]
    pub fn recipe(&self) -> &PartRecipe {
        &self.recipe
    }
}

#[doc = r" Editor passed to `Theme::override_family` and friends."]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RecipeEdit {
    default_variant: Option<Variant>,
    parts: PartMap<PartEdit>,
}

impl RecipeEdit {
    #[doc = r" Set the default variant."]
    pub fn default_variant(&mut self, v: Variant) -> &mut Self {
        self.default_variant = Some(v);
        self
    }

    #[doc = r" Edit a part."]
    pub fn part(&mut self, p: Part) -> &mut PartEdit {
        self.parts.entry(p)
    }

    #[doc = r" The default variant, if set."]
    pub const fn default_variant_set(&self) -> Option<Variant> {
        self.default_variant
    }

    #[doc = r" The edited parts as recipes."]
    pub fn into_parts(self) -> PartMap<PartRecipe> {
        let mut out = PartMap::new();
        for (p, e) in self.parts.iter() {
            out.insert(p, e.recipe.clone());
        }
        out
    }
}

#[doc = r" One overlay rule."]
pub type OverlayRule = (Family, Variant, Part, StateFlags, StylePatch);

#[doc = r" A scoped override: borrowed, `const`-constructible, never mutates the theme."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Overlay {
    rules: &'static [OverlayRule],
}

impl Overlay {
    #[doc = r" The empty overlay."]
    pub const EMPTY: Overlay = Overlay { rules: &[] };

    #[doc = r" An overlay over a static rule slice."]
    pub const fn new(rules: &'static [OverlayRule]) -> Overlay {
        Overlay { rules }
    }

    #[doc = r" The rules."]
    pub const fn rules(&self) -> &'static [OverlayRule] {
        self.rules
    }

    #[doc = r" Apply every matching rule over `acc`."]
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

    #[doc = r" A stable hash of the rule slice's identity (its address and length)."]
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
