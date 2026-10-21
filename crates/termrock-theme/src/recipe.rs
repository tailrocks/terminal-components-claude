use core::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use super::downgrade::MonoRule;
use super::glyph::GlyphRole;
use super::patch::{Slot, StateRule, StylePatch};
use crate::id::{Part, fnv1a};
use crate::response::StateFlags;
use crate::role::Surface;
use crate::theme::ColorLevel;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

macro_rules! newtype_u16 {
    ($(#[$m:meta])* $name:ident, $prefix:literal, { $( $(#[$cm:meta])* $c:ident = $v:expr ),* $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(u16);

        impl $name {
            $( $(#[$cm])* pub const $c: $name = $name($v); )*

            pub const ALL: &'static [$name] = &[ $( $name::$c ),* ];

            pub const fn custom(name: &'static str) -> $name {
                let h = fnv1a(FNV_OFFSET, name.as_bytes());
                $name(0x8000 | ((h as u16) & 0x7FFF))
            }

            pub const fn name(self) -> Option<&'static str> {
                match self {
                    $( $name::$c => Some(stringify!($c)), )*
                    _ => None,
                }
            }

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
    Family, "Family", {
        BUTTON = 0,
        CHOICE = 1,
        CHIP = 2,
        FIELD = 3,
        INPUT = 4,
        TEXTAREA = 5,
        CODE = 6,
        SELECT = 7,
        LIST = 8,
        TREE = 9,
        GRID = 10,
        PROPS = 11,
        STEPS = 12,
        TABS = 13,
        PANEL = 14,
        SPLIT = 15,
        SCROLLBAR = 16,
        VIEWPORT = 17,
        DIFF = 18,
        DIALOG = 19,
        OVERLAY = 20,
        MENU = 21,
        PICKER = 22,
        COMPLETION = 23,
        FORM = 24,
        HELP = 25,
        WIZARD = 26,
        STATUSBAR = 27,
        HINTBAR = 28,
        PROGRESS = 29,
        METER = 30,
        EMPTY = 31,
        BRAND = 32,
        KEYHINT = 33,
        TOO_SMALL = 34,
    }
}

pub use termrock_core::variant::Variant;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PartMap<T> {
    entries: Vec<(Part, T)>,
}

impl<T> PartMap<T> {
    pub const fn new() -> Self {
        PartMap {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, p: Part) -> Option<&T> {
        self.entries
            .binary_search_by_key(&p, |(k, _)| *k)
            .ok()
            .and_then(|i| self.entries.get(i))
            .map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, p: Part) -> Option<&mut T> {
        match self.entries.binary_search_by_key(&p, |(k, _)| *k) {
            Ok(i) => self.entries.get_mut(i).map(|(_, v)| v),
            Err(_) => None,
        }
    }

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

    pub fn iter(&self) -> impl Iterator<Item = (Part, &T)> + '_ {
        self.entries.iter().map(|(k, v)| (*k, v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Part, &mut T)> + '_ {
        self.entries.iter_mut().map(|(k, v)| (*k, v))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PartRecipe {
    pub base: StylePatch,
    pub states: Vec<StateRule>,
    pub glyph: Slot<GlyphRole>,
    pub size: Slot<u16>,
}

impl PartRecipe {
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

    pub fn apply_base(&self, acc: StylePatch) -> StylePatch {
        let mut acc = acc.merge(self.base);
        acc.glyph = self.glyph.over(acc.glyph);
        acc.size = self.size.over(acc.size);
        acc
    }

    pub fn apply_states(&self, acc: StylePatch, live: StateFlags) -> StylePatch {
        let mut acc = acc;
        for r in &self.states {
            if r.matches(live) {
                acc = acc.merge(r.patch);
            }
        }
        acc
    }

    pub fn apply(&self, acc: StylePatch, live: StateFlags) -> StylePatch {
        self.apply_states(self.apply_base(acc), live)
    }

    pub fn merge_from(&mut self, other: &PartRecipe) {
        self.base = self.base.merge(other.base);
        self.glyph = other.glyph.over(self.glyph);
        self.size = other.size.over(self.size);
        for r in &other.states {
            self.when(r.when, r.patch);
        }
    }
}

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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Recipe {
    pub default_variant: Variant,
    pub parts: PartMap<PartRecipe>,
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
    pub fn variant(&self, v: Variant) -> Option<&PartMap<PartRecipe>> {
        self.variants.iter().find(|(k, _)| *k == v).map(|(_, m)| m)
    }

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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GlobalOverride {
    pub family: Family,
    pub variant: Option<Variant>,
    pub parts: PartMap<PartRecipe>,
}

#[derive(Clone)]
pub struct Recipes {
    by_family: Vec<(Family, Recipe)>,
    overrides: Vec<GlobalOverride>,
    neutral: Recipe,
    mono: Vec<(Family, Vec<MonoRule>)>,
    level_families: Vec<LevelFamilyRecipe>,
    revision: u64,
}

static NEXT_RECIPE_REVISION: AtomicU64 = AtomicU64::new(1);

fn next_recipe_revision() -> u64 {
    NEXT_RECIPE_REVISION.fetch_add(1, Ordering::Relaxed)
}

impl std::fmt::Debug for Recipes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `revision` is per-construction bookkeeping for cache invalidation.
        // Two themes with the same recipes must print (and fingerprint) the
        // same, so the revision is deliberately not part of the representation.
        f.debug_struct("Recipes")
            .field("by_family", &self.by_family)
            .field("overrides", &self.overrides)
            .field("neutral", &self.neutral)
            .field("mono", &self.mono)
            .field("level_families", &self.level_families)
            .finish_non_exhaustive()
    }
}

impl PartialEq for Recipes {
    fn eq(&self, other: &Self) -> bool {
        self.by_family == other.by_family
            && self.overrides == other.overrides
            && self.neutral == other.neutral
            && self.mono == other.mono
            && self.level_families == other.level_families
    }
}

impl Eq for Recipes {}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct LevelProfileKey {
    pub level: ColorLevel,
    pub family: Family,
    pub variant: Option<Variant>,
    pub surface: Option<Surface>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct LevelFamilyRecipe {
    pub level: ColorLevel,
    pub family: Family,
    pub variant: Option<Variant>,
    pub surface: Option<Surface>,
    pub seed: Recipe,
    pub seed_source_tokens: super::ColorTokens,
    pub seed_level_tokens: super::ColorTokens,
    pub parts: PartMap<PartRecipe>,
}

impl Default for Recipes {
    fn default() -> Self {
        Recipes {
            by_family: Vec::new(),
            overrides: Vec::new(),
            neutral: super::builtin::neutral_recipe(),
            mono: Vec::new(),
            level_families: Vec::new(),
            revision: 0,
        }
    }
}

impl Recipes {
    pub fn get(&self, f: Family) -> Option<&Recipe> {
        self.by_family
            .binary_search_by_key(&f, |(k, _)| *k)
            .ok()
            .and_then(|i| self.by_family.get(i))
            .map(|(_, r)| r)
    }

    pub fn neutral(&self) -> &Recipe {
        &self.neutral
    }

    pub fn get_or_neutral(&self, f: Family) -> &Recipe {
        self.get(f).unwrap_or(&self.neutral)
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "`i` is either the index binary_search found or the index Vec::insert(i, _) just \
                  filled, so it is in range by construction"
    )]
    pub fn get_mut(&mut self, f: Family) -> &mut Recipe {
        self.revision = next_recipe_revision();
        let i = match self.by_family.binary_search_by_key(&f, |(k, _)| *k) {
            Ok(i) => i,
            Err(i) => {
                self.by_family.insert(i, (f, Recipe::default()));
                i
            }
        };
        &mut self.by_family[i].1
    }

    pub fn iter(&self) -> impl Iterator<Item = (Family, &Recipe)> + '_ {
        self.by_family.iter().map(|(k, r)| (*k, r))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Family, &mut Recipe)> + '_ {
        self.revision = next_recipe_revision();
        self.by_family.iter_mut().map(|(k, r)| (*k, r))
    }

    pub fn len(&self) -> usize {
        self.by_family.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_family.is_empty()
    }

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

    #[expect(
        clippy::indexing_slicing,
        reason = "`i` is either the index binary_search found or the index Vec::insert(i, _) just \
                  filled, so it is in range by construction"
    )]
    pub(crate) fn set_mono_rules(&mut self, f: Family, rules: Vec<MonoRule>) {
        self.revision = next_recipe_revision();
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
        self.revision = next_recipe_revision();
        self.overrides.push(o);
    }

    pub(crate) fn level_family(
        &self,
        level: ColorLevel,
        family: Family,
        variant: Option<Variant>,
        surface: Option<Surface>,
    ) -> Option<&LevelFamilyRecipe> {
        self.level_families.iter().find(|entry| {
            entry.level == level
                && entry.family == family
                && entry.variant == variant
                && entry.surface == surface
        })
    }

    pub(crate) fn define_level_family(
        &mut self,
        key: LevelProfileKey,
        parts: PartMap<PartRecipe>,
        seed_source_tokens: super::ColorTokens,
        seed_level_tokens: super::ColorTokens,
    ) {
        self.revision = next_recipe_revision();
        if let Some(existing) = self.level_families.iter_mut().find(|entry| {
            entry.level == key.level
                && entry.family == key.family
                && entry.variant == key.variant
                && entry.surface == key.surface
        }) {
            for (part, recipe) in parts.iter() {
                existing.parts.entry(part).merge_from(recipe);
            }
            return;
        }
        let seed = self.get_or_neutral(key.family).clone();
        self.level_families.push(LevelFamilyRecipe {
            level: key.level,
            family: key.family,
            variant: key.variant,
            surface: key.surface,
            seed,
            seed_source_tokens,
            seed_level_tokens,
            parts,
        });
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PartEdit {
    recipe: PartRecipe,
}

impl PartEdit {
    pub fn base(&mut self, p: StylePatch) -> &mut Self {
        self.recipe.base = self.recipe.base.merge(p);
        self
    }

    pub fn when(&mut self, s: StateFlags, p: StylePatch) -> &mut Self {
        self.recipe.when(s, p);
        self
    }

    pub fn glyph(&mut self, g: GlyphRole) -> &mut Self {
        self.recipe.glyph = Slot::Set(g);
        self
    }

    pub fn size(&mut self, n: u16) -> &mut Self {
        self.recipe.size = Slot::Set(n);
        self
    }

    pub fn recipe(&self) -> &PartRecipe {
        &self.recipe
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RecipeEdit {
    default_variant: Option<Variant>,
    parts: PartMap<PartEdit>,
}

impl RecipeEdit {
    pub fn default_variant(&mut self, v: Variant) -> &mut Self {
        self.default_variant = Some(v);
        self
    }

    pub fn part(&mut self, p: Part) -> &mut PartEdit {
        self.parts.entry(p)
    }

    pub const fn default_variant_set(&self) -> Option<Variant> {
        self.default_variant
    }

    pub fn into_parts(self) -> PartMap<PartRecipe> {
        let mut out = PartMap::new();
        for (p, e) in self.parts.iter() {
            out.insert(p, e.recipe.clone());
        }
        out
    }
}

pub type OverlayRule = (Family, Variant, Part, StateFlags, StylePatch);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Overlay {
    rules: &'static [OverlayRule],
}

impl Overlay {
    pub const EMPTY: Overlay = Overlay { rules: &[] };

    pub const fn new(rules: &'static [OverlayRule]) -> Overlay {
        Overlay { rules }
    }

    pub const fn rules(&self) -> &'static [OverlayRule] {
        self.rules
    }

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
