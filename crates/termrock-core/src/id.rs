use core::fmt;

pub(crate) const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

const SEPARATOR: u8 = 0xFF;

const KIND_NAME: u8 = 1;
const KIND_PART: u8 = 2;
const KIND_ITEM: u8 = 3;
const KIND_INDEX: u8 = 4;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Revision(pub u64);

impl Revision {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

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

const fn mix(hash: u64, kind: u8, payload: &[u8]) -> u64 {
    let hash = fnv1a(hash, &[SEPARATOR, kind]);
    fnv1a(hash, payload)
}

pub const fn custom_hash16(name: &str) -> u16 {
    let h = fnv1a(FNV_OFFSET, name.as_bytes());
    0x8000 | ((h as u16) & 0x7FFF)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id {
    hash: u64,
    #[cfg(debug_assertions)]
    label: DebugLabel,
}

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

#[macro_export]
macro_rules! id {
    ($p:literal) => {
        $crate::Id::root(concat!(module_path!(), "::", $p))
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum ItemKey {
    Index(usize),
    Num(u64),
    Text(u64),
}

const ITEM_TAG_INDEX: u8 = 0;
const ITEM_TAG_NUM: u8 = 1;
const ITEM_TAG_TEXT: u8 = 2;

impl ItemKey {
    pub const fn index(i: usize) -> Self {
        ItemKey::Index(i)
    }

    pub const fn num(n: u64) -> Self {
        ItemKey::Num(n)
    }

    pub fn text(s: &str) -> Self {
        ItemKey::Text(mix(FNV_OFFSET, ITEM_TAG_TEXT, s.as_bytes()))
    }

    pub fn pair(a: u64, b: u64) -> Self {
        let h = mix(FNV_OFFSET, KIND_NAME, &a.to_le_bytes());
        ItemKey::Text(mix(h, KIND_NAME, &b.to_le_bytes()))
    }

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Part(u16);

macro_rules! parts {
    ($( $(#[$m:meta])* $name:ident = $v:expr ),* $(,)?) => {
        impl Part {
            $( $(#[$m])* pub const $name: Part = Part($v); )*

            pub const ALL: &'static [Part] = &[ $( Part::$name ),* ];

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
    CONTAINER = 0,
    BORDER = 1,
    BACKDROP = 2,
    GUTTER = 3,
    MARKER = 4,
    ICON = 5,
    LABEL = 6,
    META = 7,
    HELP = 8,
    TITLE = 9,
    BODY = 10,
    ACTIONS = 11,
    FIELD = 12,
    TEXT = 13,
    PLACEHOLDER = 14,
    ROW = 15,
    CELL = 16,
    HEADER = 17,
    TRACK = 18,
    THUMB = 19,
    RULE = 20,
    TAB = 21,
    CLOSE = 22,
    PREFIX = 23,
    BADGE = 24,
    OVERFLOW = 25,
    NEW = 26,
    EMPTY = 27,
    QUERY = 28,
    SEAM = 29,
    SUMMARY = 30,
    DETAIL = 31,
    KEY = 32,
    ACTION = 33,
    CHANGE = 34,
    ROW_NUMBER = 35,
}

impl Part {
    pub const fn custom(name: &'static str) -> Part {
        Part(custom_hash16(name))
    }

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PartRef {
    pub part: Part,
    pub item: Option<ItemKey>,
}

impl PartRef {
    pub const fn of(p: Part) -> Self {
        PartRef {
            part: p,
            item: None,
        }
    }

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
