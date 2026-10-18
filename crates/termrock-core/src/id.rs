#![doc = r" Component identity and stable keys (`COMPONENT_ARCHITECTURE.md` §7)."]
#![doc = r""]
#![doc = r" An [`Id`] is a 64-bit FNV-1a hash over kind-tagged, separator-delimited"]
#![doc = r#" segments, so `Id::root("a").sub("b") != Id::root("ab")` is an identity"#]
#![doc = r" rather than a probability. Debug builds carry a zero-cost-in-release"]
#![doc = r" `DebugLabel` so a diagnostic or a test failure names the component: the"]
#![doc = r" root path and the **last** derivation step, not the whole chain — see"]
#![doc = r" [`Id`]'s `Debug` for exactly what it prints and what it cannot show."]
#![doc = r""]
#![doc = r" Equality, hashing and ordering are derived structurally so a `const Id`"]
#![doc = r" can be matched as a pattern (`match id { NAME => … }`, §15.1). The label"]
#![doc = r" is a pure function of the segments that produced the hash, so two ids"]
#![doc = r" with equal hashes always carry equal labels and debug and release"]
#![doc = r" compare identically; only a genuine FNV collision would differ, and"]
#![doc = r" there the label is the more honest answer."]

use core::fmt;

pub(crate) const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

#[doc = r" Separator byte mixed before every segment."]
const SEPARATOR: u8 = 0xFF;

#[doc = r#" Segment kind discriminants (§7.1 "Hashing (exact)")."#]
const KIND_NAME: u8 = 1;
const KIND_PART: u8 = 2;
const KIND_ITEM: u8 = 3;
const KIND_INDEX: u8 = 4;

#[doc = r" Monotonic content revision for externally prepared surfaces."]
#[doc = r""]
#[doc = r" A `TerminalSource` bumps its revision when its prepared cells change;"]
#[doc = r" terminal interaction state keyed by an older revision is stale and is"]
#[doc = r" dropped instead of applied."]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Revision(pub u64);

impl Revision {
    #[doc = r" The unset revision. Selections and interactions stamped with `zero()`"]
    #[doc = r" never match a live source and are ignored."]
    pub const fn zero() -> Self {
        Self(0)
    }

    #[doc = r" The next revision after `self`."]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    #[doc = r" The raw counter value."]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[doc = r" FNV-1a over a byte slice, continuing from `hash`; shared by identity"]
#[doc = r" hashing and custom-variant naming."]
pub const fn fnv1a(mut hash: u64, bytes: &[u8]) -> u64 {
    let mut i = 0;
    while i < bytes.len() {
        #[expect(
            clippy::indexing_slicing,
            reason = "bounded by the loop condition; const fn cannot use iterators"
        )]
        let b = bytes[i];
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i = i.wrapping_add(1);
    }
    hash
}

#[doc = r" Mix one segment: separator, kind byte, payload bytes."]
const fn mix(hash: u64, kind: u8, payload: &[u8]) -> u64 {
    let hash = fnv1a(hash, &[SEPARATOR, kind]);
    fnv1a(hash, payload)
}

#[doc = r" FNV-1a over `name`, folded into the custom half of a 16-bit key space."]
#[doc = r""]
#[doc = r" Every 16-bit custom key namespace in the crate ([`Part::custom`], the"]
#[doc = r" grid's `ColumnKey::of`) states one truncation rule here rather than"]
#[doc = r" re-deriving its own hash: the low bit range is reserved for numbered keys"]
#[doc = r" and `0x8000..=0xFFFF` for hashed names, so a numbered key and a named key"]
#[doc = r" can never be the same key."]
pub const fn custom_hash16(name: &str) -> u16 {
    let h = fnv1a(FNV_OFFSET, name.as_bytes());
    0x8000 | ((h as u16) & 0x7FFF)
}

#[doc = r" A stable component identity."]
#[doc = r""]
#[doc = r" Built from a `const` path by [`id!`](mod@crate::id) or [`Id::root`] and derived with"]
#[doc = r" [`Id::sub`], [`Id::part`], [`Id::index`] and [`Id::item`]. Every"]
#[doc = r" derivation is a `const fn` except `item`, which takes a runtime key."]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id {
    hash: u64,
    #[cfg(debug_assertions)]
    label: DebugLabel,
}

#[doc = r" The diagnostic label carried by an [`Id`] in debug builds: the root path"]
#[doc = r" plus the last derivation step."]
#[doc = r""]
#[doc = r" Two `&'static`-sized fields, fixed size and copyable, so an `Id` stays a"]
#[doc = r" `Copy` value usable in a `const`. Storing the full chain would need an"]
#[doc = r" allocation or an arbitrary bound, which is why the label truncates rather"]
#[doc = r" than grows; the hash, not the label, is the identity."]
#[cfg(debug_assertions)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct DebugLabel {
    root: &'static str,
    tail: Tail,
}

#[cfg(debug_assertions)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Tail {
    Root,
    Sub(&'static str),
    Part(Part),
    Item(ItemKey),
    Index(usize),
}

impl Id {
    #[doc = r#" A root id from a static path such as `"orders.list"`."#]
    pub const fn root(path: &'static str) -> Id {
        Id {
            hash: mix(FNV_OFFSET, KIND_NAME, path.as_bytes()),
            #[cfg(debug_assertions)]
            label: DebugLabel {
                root: path,
                tail: Tail::Root,
            },
        }
    }

    #[doc = r" A named child id (`Kind::Name`)."]
    #[must_use]
    pub const fn sub(self, name: &'static str) -> Id {
        Id {
            hash: mix(self.hash, KIND_NAME, name.as_bytes()),
            #[cfg(debug_assertions)]
            label: DebugLabel {
                root: self.label.root,
                tail: Tail::Sub(name),
            },
        }
    }

    #[doc = r" A child *component* id addressed by a part (`Kind::Part`)."]
    #[doc = r""]
    #[doc = r" This mints a new component identity (a `Button` inside a `Dialog`),"]
    #[doc = r" never a sub-region of one component — sub-regions are [`PartRef`]s."]
    #[must_use]
    pub const fn part(self, p: Part) -> Id {
        Id {
            hash: mix(self.hash, KIND_PART, &p.0.to_le_bytes()),
            #[cfg(debug_assertions)]
            label: DebugLabel {
                root: self.label.root,
                tail: Tail::Part(p),
            },
        }
    }

    #[doc = r" A positional child id (`Kind::Index`) — UNSTABLE under reorder."]
    #[must_use]
    pub const fn index(self, i: usize) -> Id {
        Id {
            hash: mix(self.hash, KIND_INDEX, &i.to_le_bytes()),
            #[cfg(debug_assertions)]
            label: DebugLabel {
                root: self.label.root,
                tail: Tail::Index(i),
            },
        }
    }

    #[doc = r" A keyed child id (`Kind::Item`); stable under insert/remove/reorder"]
    #[doc = r" when the key is."]
    #[must_use]
    pub fn item(self, k: ItemKey) -> Id {
        Id {
            hash: mix(self.hash, KIND_ITEM, &k.payload()),
            #[cfg(debug_assertions)]
            label: DebugLabel {
                root: self.label.root,
                tail: Tail::Item(k),
            },
        }
    }

    #[doc = r" The raw 64-bit hash, for registries and tests."]
    pub const fn hash(self) -> u64 {
        self.hash
    }
}

#[doc = r" Debug builds print `root ▸ last-segment`; release builds print"]
#[doc = r" `Id(<16 hex digits>)`, because the label does not exist there."]
#[doc = r""]
#[doc = r" The label records only the root and the **most recent** derivation, so"]
#[doc = r#" `Id::root("a").sub("b").sub("c")` prints `a ▸ c` — the intermediate"#]
#[doc = r" `b` is not stored anywhere. Two different ids can therefore print the"]
#[doc = r#" same text (`a.sub("row").part(Part::LABEL)` and `a.part(Part::LABEL)`),"#]
#[doc = r" and a printed label alone does not reconstruct the path that built the id."]
#[doc = r" The label is diagnostic only: identity is the hash, and"]
#[doc = r" [`Id::hash`] is what a test should compare when it needs to be exact."]
impl fmt::Debug for Id {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root = self.label.root;
        match self.label.tail {
            Tail::Root => write!(f, "{root}"),
            Tail::Sub(name) => write!(f, "{root} ▸ {name}"),
            Tail::Part(p) => write!(f, "{root} ▸ {p:?}"),
            Tail::Item(k) => write!(f, "{root} ▸ {k:?}"),
            Tail::Index(i) => write!(f, "{root} ▸ #{i}"),
        }
    }

    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({:016x})", self.hash)
    }
}

#[doc = r" Mint a root [`Id`] from a string literal, qualified by the calling module"]
#[doc = r#" path so two screens may both declare `id!("save")`."#]
#[macro_export]
macro_rules! id {
    ($p:literal) => {
        $crate::Id::root(concat!(module_path!(), "::", $p))
    };
}

#[doc = r" A stable key for one item of a collection."]
#[doc = r""]
#[doc = r" Every collection action carries an `ItemKey`, never a display index."]
#[doc = r" [`ItemKey::index`] exists for genuinely positional cases and is documented"]
#[doc = r" as UNSTABLE under insert, remove and reorder."]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum ItemKey {
    #[doc = r" A positional key — UNSTABLE under insert/remove/reorder."]
    Index(usize),
    #[doc = r" A numeric domain key (a database id, a handle)."]
    Num(u64),
    #[doc = r" A hashed textual or composite key."]
    Text(u64),
}

const ITEM_TAG_INDEX: u8 = 0;
const ITEM_TAG_NUM: u8 = 1;
const ITEM_TAG_TEXT: u8 = 2;

impl ItemKey {
    #[doc = r" A positional key. Documented: UNSTABLE under insert/remove/reorder."]
    pub const fn index(i: usize) -> Self {
        ItemKey::Index(i)
    }

    #[doc = r" A numeric key."]
    pub const fn num(n: u64) -> Self {
        ItemKey::Num(n)
    }

    #[doc = r" A textual key: FNV-1a over the bytes, kind-tagged."]
    pub fn text(s: &str) -> Self {
        ItemKey::Text(mix(FNV_OFFSET, ITEM_TAG_TEXT, s.as_bytes()))
    }

    #[doc = r" A composite key (schema + table, provider + account); order-sensitive."]
    pub fn pair(a: u64, b: u64) -> Self {
        let h = mix(FNV_OFFSET, KIND_NAME, &a.to_le_bytes());
        ItemKey::Text(mix(h, KIND_NAME, &b.to_le_bytes()))
    }

    #[doc = r" Tag byte plus eight little-endian payload bytes."]
    fn payload(self) -> [u8; 9] {
        let (tag, n) = match self {
            ItemKey::Index(i) => (ITEM_TAG_INDEX, i as u64),
            ItemKey::Num(n) => (ITEM_TAG_NUM, n),
            ItemKey::Text(h) => (ITEM_TAG_TEXT, h),
        };
        let b = n.to_le_bytes();
        [tag, b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
    }
}

#[doc = r" A typed logical part of a component (`Part::LABEL`, `Part::THUMB`, …)."]
#[doc = r""]
#[doc = r" `0..=255` is reserved for the library; [`Part::custom`] maps into"]
#[doc = r" `0x8000..=0xFFFF` by the crate's one 16-bit custom-key fold"]
#[doc = r" (`custom_hash16`), shared with the grid's `ColumnKey::of`."]
#[doc = r""]
#[doc = r" Equality is derived and therefore *structural*, which is load-bearing:"]
#[doc = r" `PartRef { part: Part::LABEL, .. }` appears as a `match` pattern across"]
#[doc = r" the component set, and a pattern needs `#[derive(PartialEq)]`. A part is"]
#[doc = r" exactly its number, so debug and release compare identically — at the"]
#[doc = r" price that two custom names folding to the same 15 bits mint the same"]
#[doc = r" part. See [`Part::custom`]."]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Part(u16);

macro_rules! parts {
    ($( $(#[$m:meta])* $name:ident = $v:expr ),* $(,)?) => {
        impl Part {
            $( $(#[$m])* pub const $name: Part = Part($v); )*

            #[doc = r" Every library part constant, in declaration order."]
            pub const ALL: &'static [Part] = &[ $( Part::$name ),* ];

            #[doc = r" The library name of a part, or `None` for a custom part."]
            pub const fn name(self) -> Option<&'static str> {
                match self {
                    $( Part::$name => Some(stringify!($name)), )*
                    _ => None,
                }
            }
        }
    };
}

parts! {
    #[doc = r" The whole component surface."]
    CONTAINER = 0,
    #[doc = r" A frame or border."]
    BORDER = 1,
    #[doc = r" A dimmed backdrop behind a layer."]
    BACKDROP = 2,
    #[doc = r" The focus gutter column."]
    GUTTER = 3,
    #[doc = r" A selection / check marker."]
    MARKER = 4,
    #[doc = r" A leading icon."]
    ICON = 5,
    #[doc = r" The main label."]
    LABEL = 6,
    #[doc = r" Secondary text (dropped all-or-none)."]
    META = 7,
    #[doc = r" Help / caption text."]
    HELP = 8,
    #[doc = r" A title."]
    TITLE = 9,
    #[doc = r" The body slot."]
    BODY = 10,
    #[doc = r" The action row."]
    ACTIONS = 11,
    #[doc = r" A field surface."]
    FIELD = 12,
    #[doc = r" Editable text."]
    TEXT = 13,
    #[doc = r" Placeholder text."]
    PLACEHOLDER = 14,
    #[doc = r" A collection row."]
    ROW = 15,
    #[doc = r" A grid cell."]
    CELL = 16,
    #[doc = r" A header row."]
    HEADER = 17,
    #[doc = r" A scrollbar track."]
    TRACK = 18,
    #[doc = r" A scrollbar thumb."]
    THUMB = 19,
    #[doc = r" A horizontal rule."]
    RULE = 20,
    #[doc = r" One tab."]
    TAB = 21,
    #[doc = r" A close affordance."]
    CLOSE = 22,
    #[doc = r" A prefix glyph."]
    PREFIX = 23,
    #[doc = r" A badge."]
    BADGE = 24,
    #[doc = r" An overflow indicator."]
    OVERFLOW = 25,
    #[doc = r#" The "new" affordance."#]
    NEW = 26,
    #[doc = r" The empty-state area."]
    EMPTY = 27,
    #[doc = r" A query field."]
    QUERY = 28,
    #[doc = r" A split seam."]
    SEAM = 29,
    #[doc = r" A summary line."]
    SUMMARY = 30,
    #[doc = r" A detail line."]
    DETAIL = 31,
    #[doc = r" A key hint's key."]
    KEY = 32,
    #[doc = r" A key hint's action."]
    ACTION = 33,
    #[doc = r" A row change-state glyph, separate from selection."]
    CHANGE = 34,
    #[doc = r" A source row number."]
    ROW_NUMBER = 35,
}

impl Part {
    #[doc = r" A custom part named by a downstream component; lands in the high range."]
    #[doc = r""]
    #[doc = r" The name folds to 15 bits, so two names *can* mint the same part and"]
    #[doc = r" alias two components' sub-regions. `Part` cannot carry the name that"]
    #[doc = r" would name such a collision without giving up structural equality"]
    #[doc = r" (see the type documentation), so the fold is stated once in"]
    #[doc = r" `custom_hash16` and the aliasing is documented, not hidden."]
    pub const fn custom(name: &'static str) -> Part {
        Part(custom_hash16(name))
    }

    #[doc = r" The raw part number."]
    pub const fn raw(self) -> u16 {
        self.0
    }
}

impl fmt::Debug for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name() {
            Some(n) => write!(f, "Part::{n}"),
            None => write!(f, "Part::custom(#{:04x})", self.0),
        }
    }
}

#[doc = r" A sub-region of a single component: a part plus an optional item key."]
#[doc = r""]
#[doc = r" Stored directly in every registry region and delivered with every pointer"]
#[doc = r" intent. Never interchangeable with a derived [`Id`] (§21 item 16)."]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PartRef {
    #[doc = r" The part."]
    pub part: Part,
    #[doc = r" The item, for per-row parts."]
    pub item: Option<ItemKey>,
}

impl PartRef {
    #[doc = r" A part without an item."]
    pub const fn of(p: Part) -> Self {
        PartRef {
            part: p,
            item: None,
        }
    }

    #[doc = r" A per-item part."]
    pub const fn item(p: Part, k: ItemKey) -> Self {
        PartRef {
            part: p,
            item: Some(k),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cmp::Ordering;
    use core::hash::{Hash, Hasher};

    #[test]
    fn root_sub_part_index_item_are_all_distinct() {
        let r = Id::root("x");
        let all = [
            r,
            r.sub("y"),
            r.part(Part::LABEL),
            r.index(0),
            r.item(ItemKey::index(0)),
            r.item(ItemKey::num(0)),
            r.item(ItemKey::text("0")),
        ];
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                assert_eq!(a == b, i == j, "{a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn separator_prevents_concatenation_collision() {
        assert_ne!(Id::root("a").sub("b"), Id::root("ab"));
        assert_ne!(Id::root("ab").sub(""), Id::root("a").sub("b"));
    }

    #[test]
    fn kind_tag_separates_name_from_item_with_equal_bytes() {
        let r = Id::root("r");

        let name = "\u{61}\0\0\0\0\0\0\0";
        assert_ne!(r.sub(name), r.index(0x61));
        assert_ne!(r.item(ItemKey::index(0x61)), r.index(0x61));
        assert_ne!(r.item(ItemKey::num(7)), r.item(ItemKey::index(7)));
    }

    #[doc = r" §2.2: `Id` derives structural equality (a `const Id` in a `match`"]
    #[doc = r" pattern needs it). The invariant that makes debug and release compare"]
    #[doc = r" identically is that equality *is* hash equality — proved over a corpus"]
    #[doc = r" built by every derivation, not by comparing two ids with identical"]
    #[doc = r" labels."]
    #[test]
    fn id_equality_is_exactly_hash_equality() {
        let mut corpus: Vec<Id> = Vec::new();
        let roots = ["a", "ab", "b", "root", "root::a", ""];
        for r in roots {
            let base = Id::root(r);
            corpus.push(base);
            corpus.push(base.sub("b"));
            corpus.push(base.sub(""));
            corpus.push(base.sub("b").sub("c"));
            corpus.push(base.part(Part::LABEL));
            corpus.push(base.part(Part::GUTTER));
            corpus.push(base.index(0));
            corpus.push(base.index(1));
            corpus.push(base.item(ItemKey::num(0)));
            corpus.push(base.item(ItemKey::num(1)));
            corpus.push(base.item(ItemKey::text("b")));
            corpus.push(base.item(ItemKey::pair(1, 2)));
            corpus.push(base.item(ItemKey::pair(2, 1)));
            corpus.push(base.part(Part::LABEL).index(3));
            corpus.push(base.sub("b").part(Part::custom("z")));
        }
        assert!(corpus.len() >= 90);
        for a in &corpus {
            for b in &corpus {
                let (ha, hb) = (Id::hash(*a), Id::hash(*b));
                assert_eq!(
                    a == b,
                    ha == hb,
                    "equality must be exactly hash equality: {a:?} vs {b:?}"
                );
                assert_eq!(a.cmp(b), ha.cmp(&hb), "{a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn id_equality_ignores_debug_label() {
        let a = id!("same");
        let b = Id::root(concat!(module_path!(), "::same"));
        assert_eq!(a, b);
        assert_eq!(a.cmp(&b), Ordering::Equal);
        let mut h1 = std::collections::hash_map::DefaultHasher::new();
        let mut h2 = std::collections::hash_map::DefaultHasher::new();
        Hash::hash(&a, &mut h1);
        Hash::hash(&b, &mut h2);
        assert_eq!(h1.finish(), h2.finish());

        let x = Id::root("x");
        let y = Id::root("y");
        assert_eq!(x.cmp(&y), x.hash().cmp(&y.hash()));
    }

    #[test]
    fn id_is_const_constructible() {
        const A: Id = id!("a");
        const B: Id = A.sub("b").part(Part::LABEL).index(2);
        assert_ne!(A, B);
        assert_eq!(A, id!("a"));
    }

    #[test]
    fn item_key_text_is_stable_across_runs() {
        assert_eq!(ItemKey::text("orders"), ItemKey::text("orders"));
        assert_eq!(
            ItemKey::text("orders"),
            ItemKey::Text(13_814_024_800_606_603_357),
            "the FNV mix is part of the contract"
        );
        assert_ne!(ItemKey::text("orders"), ItemKey::text("order"));
    }

    #[test]
    fn item_key_pair_is_order_sensitive() {
        assert_ne!(ItemKey::pair(1, 2), ItemKey::pair(2, 1));
        assert_eq!(ItemKey::pair(1, 2), ItemKey::pair(1, 2));
    }

    #[test]
    fn part_custom_lands_in_the_high_range() {
        let p = Part::custom("segment");
        assert!(p.raw() >= 0x8000);
        assert_eq!(p.name(), None);
        assert_eq!(Part::custom("segment"), Part::custom("segment"));
        assert_ne!(Part::custom("segment"), Part::custom("segments"));
    }

    #[doc = r" The library's 16-bit custom key spaces must fold names the same way,"]
    #[doc = r#" or "custom" means something different per namespace."#]
    #[test]
    fn part_custom_uses_the_shared_custom_fold() {
        assert_eq!(Part::custom("segment").raw(), custom_hash16("segment"));
        assert_eq!(Part::custom("").raw(), custom_hash16(""));
        for name in ["segment", "", "a", "grid.preview", "\u{1f600}"] {
            let raw = custom_hash16(name);
            assert!(raw >= 0x8000, "{name:?} escaped the custom range");
            assert!(Part(raw).name().is_none(), "{name:?} hit a library part");
        }
    }

    #[doc = r" A named witness for the collision class the fold *cannot* remove:"]
    #[doc = r" 15 bits of FNV are not injective over names, so two custom parts can"]
    #[doc = r" be the same part. `Part` cannot carry the name that would name the"]
    #[doc = r" collision — `PartRef { part: Part::LABEL, .. }` is a `match` pattern"]
    #[doc = r" across the component set and a pattern needs derived (structural)"]
    #[doc = r" equality, which a debug-only name field would break. This test pins"]
    #[doc = r" the fold and the limitation so neither changes unnoticed."]
    #[test]
    fn part_custom_names_can_collide_in_fifteen_bits() {
        let a = Part::custom("chart.flag");
        let b = Part::custom("chart.ring");
        assert_eq!(a.raw(), 0xb9e1, "the fold is part of the contract");
        assert_eq!(a, b, "15 bits cannot separate every pair of names");
        assert_eq!(format!("{a:?}"), "Part::custom(#b9e1)");
        assert_eq!(format!("{a:?}"), format!("{b:?}"));
    }

    #[test]
    fn part_constants_are_unique() {
        let all = Part::ALL;
        for (i, a) in all.iter().enumerate() {
            assert!(a.raw() < 256);
            for (j, b) in all.iter().enumerate() {
                assert_eq!(a == b, i == j);
            }
        }
        assert_eq!(all.len(), 36);
        assert_eq!(Part::CHANGE.raw(), 34);
        assert_eq!(Part::ROW_NUMBER.raw(), 35);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn debug_prints_path_in_debug_builds() {
        let r = Id::root("orders.list");
        assert_eq!(format!("{r:?}"), "orders.list");
        assert_eq!(format!("{:?}", r.sub("row")), "orders.list ▸ row");
        assert_eq!(
            format!("{:?}", r.item(ItemKey::num(7))),
            "orders.list ▸ Num(7)"
        );
        assert_eq!(
            format!("{:?}", r.part(Part::LABEL)),
            "orders.list ▸ Part::LABEL"
        );
        assert_eq!(format!("{:?}", r.index(3)), "orders.list ▸ #3");
    }

    #[doc = r" Acceptance: the debug label is the root plus the **last** derivation"]
    #[doc = r" step, never the whole chain, and the documentation says so."]
    #[doc = r""]
    #[doc = r" A reader who believes `Debug` prints the path would use it to tell two"]
    #[doc = r" ids apart; it cannot, because intermediate segments are not stored."]
    #[doc = r" The hash is what separates them."]
    #[cfg(debug_assertions)]
    #[test]
    fn debug_label_keeps_only_the_root_and_the_last_segment() {
        let r = Id::root("orders.list");

        assert_eq!(
            format!("{:?}", r.sub("row").sub("cell")),
            "orders.list ▸ cell"
        );
        assert_eq!(
            format!("{:?}", r.sub("row").index(3).part(Part::LABEL)),
            "orders.list ▸ Part::LABEL"
        );

        let deep = r.sub("row").part(Part::LABEL);
        let shallow = r.part(Part::LABEL);
        assert_eq!(format!("{deep:?}"), format!("{shallow:?}"));
        assert_ne!(deep, shallow);
        assert_ne!(deep.hash(), shallow.hash());
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn debug_prints_hash_in_release_builds() {
        let r = Id::root("orders.list");
        assert_eq!(format!("{r:?}"), format!("Id({:016x})", r.hash()));
    }
}
