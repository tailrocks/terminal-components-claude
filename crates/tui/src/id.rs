//! Component identity and stable keys (`COMPONENT_ARCHITECTURE.md` §7).
//!
//! An [`Id`] is a 64-bit FNV-1a hash over kind-tagged, separator-delimited
//! segments, so `Id::root("a").sub("b") != Id::root("ab")` is an identity
//! rather than a probability. Debug builds carry a zero-cost-in-release
//! `DebugLabel` so a diagnostic or a test failure prints the path.
//!
//! Equality, hashing and ordering are derived structurally so a `const Id`
//! can be matched as a pattern (`match id { NAME => … }`, §15.1). The label
//! is a pure function of the segments that produced the hash, so two ids
//! with equal hashes always carry equal labels and debug and release
//! compare identically; only a genuine FNV collision would differ, and
//! there the label is the more honest answer.

use core::fmt;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// Separator byte mixed before every segment.
const SEPARATOR: u8 = 0xFF;

/// Segment kind discriminants (§7.1 "Hashing (exact)").
const KIND_NAME: u8 = 1;
const KIND_PART: u8 = 2;
const KIND_ITEM: u8 = 3;
const KIND_INDEX: u8 = 4;

/// FNV-1a over a byte slice, continuing from `hash`.
pub(crate) const fn fnv1a(mut hash: u64, bytes: &[u8]) -> u64 {
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

/// Mix one segment: separator, kind byte, payload bytes.
const fn mix(hash: u64, kind: u8, payload: &[u8]) -> u64 {
    let hash = fnv1a(hash, &[SEPARATOR, kind]);
    fnv1a(hash, payload)
}

/// A stable component identity.
///
/// Built from a `const` path by [`id!`](crate::id) or [`Id::root`] and derived with
/// [`Id::sub`], [`Id::part`], [`Id::index`] and [`Id::item`]. Every
/// derivation is a `const fn` except `item`, which takes a runtime key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id {
    hash: u64,
    #[cfg(debug_assertions)]
    label: DebugLabel,
}

/// The human-readable path carried by an [`Id`] in debug builds.
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
    /// A root id from a static path such as `"orders.list"`.
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

    /// A named child id (`Kind::Name`).
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

    /// A child *component* id addressed by a part (`Kind::Part`).
    ///
    /// This mints a new component identity (a `Button` inside a `Dialog`),
    /// never a sub-region of one component — sub-regions are [`PartRef`]s.
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

    /// A positional child id (`Kind::Index`) — UNSTABLE under reorder.
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

    /// A keyed child id (`Kind::Item`); stable under insert/remove/reorder
    /// when the key is.
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

    /// The raw 64-bit hash, for registries and tests.
    pub const fn hash(self) -> u64 {
        self.hash
    }
}

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

/// Mint a root [`Id`] from a string literal, qualified by the calling module
/// path so two screens may both declare `id!("save")`.
#[macro_export]
macro_rules! id {
    ($p:literal) => {
        $crate::Id::root(concat!(module_path!(), "::", $p))
    };
}

/// A stable key for one item of a collection.
///
/// Every collection action carries an `ItemKey`, never a display index.
/// [`ItemKey::index`] exists for genuinely positional cases and is documented
/// as UNSTABLE under insert, remove and reorder.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum ItemKey {
    /// A positional key — UNSTABLE under insert/remove/reorder.
    Index(usize),
    /// A numeric domain key (a database id, a handle).
    Num(u64),
    /// A hashed textual or composite key.
    Text(u64),
}

const ITEM_TAG_INDEX: u8 = 0;
const ITEM_TAG_NUM: u8 = 1;
const ITEM_TAG_TEXT: u8 = 2;

impl ItemKey {
    /// A positional key. Documented: UNSTABLE under insert/remove/reorder.
    pub const fn index(i: usize) -> Self {
        ItemKey::Index(i)
    }

    /// A numeric key.
    pub const fn num(n: u64) -> Self {
        ItemKey::Num(n)
    }

    /// A textual key: FNV-1a over the bytes, kind-tagged.
    pub fn text(s: &str) -> Self {
        ItemKey::Text(mix(FNV_OFFSET, ITEM_TAG_TEXT, s.as_bytes()))
    }

    /// A composite key (schema + table, provider + account); order-sensitive.
    pub fn pair(a: u64, b: u64) -> Self {
        let h = mix(FNV_OFFSET, KIND_NAME, &a.to_le_bytes());
        ItemKey::Text(mix(h, KIND_NAME, &b.to_le_bytes()))
    }

    /// Tag byte plus eight little-endian payload bytes.
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

/// A typed logical part of a component (`Part::LABEL`, `Part::THUMB`, …).
///
/// `0..=255` is reserved for the library; [`Part::custom`] maps into
/// `0x8000..=0xFFFF` by FNV of the name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Part(u16);

macro_rules! parts {
    ($( $(#[$m:meta])* $name:ident = $v:expr ),* $(,)?) => {
        impl Part {
            $( $(#[$m])* pub const $name: Part = Part($v); )*

            /// Every library part constant, in declaration order.
            pub const ALL: &'static [Part] = &[ $( Part::$name ),* ];

            /// The library name of a part, or `None` for a custom part.
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
    /// The whole component surface.
    CONTAINER = 0,
    /// A frame or border.
    BORDER = 1,
    /// A dimmed backdrop behind a layer.
    BACKDROP = 2,
    /// The focus gutter column.
    GUTTER = 3,
    /// A selection / check marker.
    MARKER = 4,
    /// A leading icon.
    ICON = 5,
    /// The main label.
    LABEL = 6,
    /// Secondary text (dropped all-or-none).
    META = 7,
    /// Help / caption text.
    HELP = 8,
    /// A title.
    TITLE = 9,
    /// The body slot.
    BODY = 10,
    /// The action row.
    ACTIONS = 11,
    /// A field surface.
    FIELD = 12,
    /// Editable text.
    TEXT = 13,
    /// Placeholder text.
    PLACEHOLDER = 14,
    /// A collection row.
    ROW = 15,
    /// A grid cell.
    CELL = 16,
    /// A header row.
    HEADER = 17,
    /// A scrollbar track.
    TRACK = 18,
    /// A scrollbar thumb.
    THUMB = 19,
    /// A horizontal rule.
    RULE = 20,
    /// One tab.
    TAB = 21,
    /// A close affordance.
    CLOSE = 22,
    /// A prefix glyph.
    PREFIX = 23,
    /// A badge.
    BADGE = 24,
    /// An overflow indicator.
    OVERFLOW = 25,
    /// The "new" affordance.
    NEW = 26,
    /// The empty-state area.
    EMPTY = 27,
    /// A query field.
    QUERY = 28,
    /// A split seam.
    SEAM = 29,
    /// A summary line.
    SUMMARY = 30,
    /// A detail line.
    DETAIL = 31,
    /// A key hint's key.
    KEY = 32,
    /// A key hint's action.
    ACTION = 33,
}

impl Part {
    /// A custom part named by a downstream component; lands in the high range.
    pub const fn custom(name: &'static str) -> Part {
        let h = fnv1a(FNV_OFFSET, name.as_bytes());
        Part(0x8000 | ((h as u16) & 0x7FFF))
    }

    /// The raw part number.
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

/// A sub-region of a single component: a part plus an optional item key.
///
/// Stored directly in every registry region and delivered with every pointer
/// intent. Never interchangeable with a derived [`Id`] (§21 item 16).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PartRef {
    /// The part.
    pub part: Part,
    /// The item, for per-row parts.
    pub item: Option<ItemKey>,
}

impl PartRef {
    /// A part without an item.
    pub const fn of(p: Part) -> Self {
        PartRef {
            part: p,
            item: None,
        }
    }

    /// A per-item part.
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
        // an index whose little-endian bytes equal the name "a\0\0\0\0\0\0\0"
        let name = "\u{61}\0\0\0\0\0\0\0";
        assert_ne!(r.sub(name), r.index(0x61));
        assert_ne!(r.item(ItemKey::index(0x61)), r.index(0x61));
        assert_ne!(r.item(ItemKey::num(7)), r.item(ItemKey::index(7)));
    }

    /// §2.2: `Id` derives structural equality (a `const Id` in a `match`
    /// pattern needs it). The invariant that makes debug and release compare
    /// identically is that equality *is* hash equality — proved over a corpus
    /// built by every derivation, not by comparing two ids with identical
    /// labels.
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
        // ids built by different paths to the same segments compare equal,
        // hash equally and order equally in debug and release alike
        let a = id!("same");
        let b = Id::root(concat!(module_path!(), "::same"));
        assert_eq!(a, b);
        assert_eq!(a.cmp(&b), Ordering::Equal);
        let mut h1 = std::collections::hash_map::DefaultHasher::new();
        let mut h2 = std::collections::hash_map::DefaultHasher::new();
        Hash::hash(&a, &mut h1);
        Hash::hash(&b, &mut h2);
        assert_eq!(h1.finish(), h2.finish());
        // ordering follows the hash
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

    #[test]
    fn part_constants_are_unique() {
        let all = Part::ALL;
        for (i, a) in all.iter().enumerate() {
            assert!(a.raw() < 256);
            for (j, b) in all.iter().enumerate() {
                assert_eq!(a == b, i == j);
            }
        }
        assert_eq!(all.len(), 34);
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

    #[cfg(not(debug_assertions))]
    #[test]
    fn debug_prints_hash_in_release_builds() {
        let r = Id::root("orders.list");
        assert_eq!(format!("{r:?}"), format!("Id({:016x})", r.hash()));
    }
}
