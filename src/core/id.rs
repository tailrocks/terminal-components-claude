use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WidgetId(u64);

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

const fn fnv1a(mut hash: u64, bytes: &[u8]) -> u64 {
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}

impl WidgetId {
    pub const fn of(path: &str) -> Self {
        Self(fnv1a(FNV_OFFSET, path.as_bytes()))
    }

    pub const fn child(self, index: usize) -> Self {
        Self(fnv1a(self.0, &index.to_le_bytes()))
    }

    pub const fn sub(self, name: &str) -> Self {
        Self(fnv1a(self.0, name.as_bytes()))
    }

    pub const fn part(self, p: PartRef) -> Self {
        let base = match p.item {
            None => self,
            Some(ItemKey::Index(i)) => self.child(i),
            Some(ItemKey::Num(n)) => Self(fnv1a(fnv1a(self.0, b"#num"), &n.to_le_bytes())),
            Some(ItemKey::Text(h)) => Self(fnv1a(fnv1a(self.0, b"#text"), &h.to_le_bytes())),
        };
        match (p.part, p.item) {
            (Part::CONTAINER, None) => self,
            (Part::ROW | Part::TAB, Some(_)) => base,
            _ => Self(fnv1a(fnv1a(base.0, b"#part"), &p.part.raw().to_le_bytes())),
        }
    }

    pub fn item_of(
        self,
        id: WidgetId,
        part: Part,
        items: impl IntoIterator<Item = usize>,
    ) -> Option<usize> {
        items
            .into_iter()
            .find(|&i| self.part(PartRef::item(part, ItemKey::Index(i))) == id)
    }
}

impl fmt::Debug for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WidgetId({:016x})", self.0)
    }
}

pub const fn custom_hash16(name: &str) -> u16 {
    let h = fnv1a(FNV_OFFSET, name.as_bytes());
    0x8000 | ((h ^ (h >> 16) ^ (h >> 32) ^ (h >> 48)) as u16 & 0x7FFF)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum ItemKey {
    Index(usize),

    Num(u64),

    Text(u64),
}

impl ItemKey {
    pub const fn index(i: usize) -> Self {
        ItemKey::Index(i)
    }

    pub const fn num(n: u64) -> Self {
        ItemKey::Num(n)
    }

    pub const fn text(s: &str) -> Self {
        ItemKey::Text(fnv1a(FNV_OFFSET, s.as_bytes()))
    }

    pub const fn pair(a: u64, b: u64) -> Self {
        let h = fnv1a(FNV_OFFSET, &a.to_le_bytes());
        ItemKey::Text(fnv1a(h, &b.to_le_bytes()))
    }

    pub const fn as_index(self) -> Option<usize> {
        match self {
            ItemKey::Index(i) => Some(i),
            _ => None,
        }
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
    pub const fn custom(name: &str) -> Part {
        Part(custom_hash16(name))
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

impl fmt::Debug for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name() {
            Some(name) => write!(f, "Part::{name}"),
            None => write!(f, "Part({:#06x})", self.0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PartRef {
    pub part: Part,

    pub item: Option<ItemKey>,
}

impl PartRef {
    pub const OVERFLOW_BEFORE: PartRef = PartRef::item(Part::OVERFLOW, ItemKey::Index(0));

    pub const OVERFLOW_AFTER: PartRef = PartRef::item(Part::OVERFLOW, ItemKey::Index(1));

    pub const fn of(part: Part) -> Self {
        Self { part, item: None }
    }

    pub const fn item(part: Part, key: ItemKey) -> Self {
        Self {
            part,
            item: Some(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable_and_distinct() {
        assert_eq!(WidgetId::of("a.b"), WidgetId::of("a.b"));
        assert_ne!(WidgetId::of("a.b"), WidgetId::of("a.c"));
        assert_ne!(WidgetId::of("a").child(0), WidgetId::of("a").child(1));
        assert_ne!(WidgetId::of("a").child(0), WidgetId::of("a"));
        assert_ne!(WidgetId::of("a").sub("x"), WidgetId::of("a").sub("y"));
    }

    #[test]
    fn part_constants_are_unique_and_named() {
        for (i, a) in Part::ALL.iter().enumerate() {
            assert!(a.name().is_some());
            assert!(a.raw() <= 255, "library parts stay in the low range");
            for b in &Part::ALL[i + 1..] {
                assert_ne!(a, b, "{a:?} and {b:?} share a number");
            }
        }
        assert_eq!(format!("{:?}", Part::ROW), "Part::ROW");
    }

    #[test]
    fn custom_parts_land_in_the_high_range() {
        let p = Part::custom("sparkline");
        assert!(p.raw() >= 0x8000);
        assert_eq!(p.name(), None);
        assert_eq!(p, Part::custom("sparkline"));
    }

    #[test]
    fn item_keys_are_stable_and_order_sensitive() {
        assert_eq!(ItemKey::text("orders"), ItemKey::text("orders"));
        assert_ne!(ItemKey::text("orders"), ItemKey::text("order"));
        assert_ne!(ItemKey::pair(1, 2), ItemKey::pair(2, 1));
        assert_eq!(ItemKey::index(4).as_index(), Some(4));
        assert_eq!(ItemKey::num(4).as_index(), None);
        let row = PartRef::item(Part::ROW, ItemKey::index(2));
        assert!(matches!(
            row,
            PartRef {
                part: Part::ROW,
                item: Some(ItemKey::Index(2))
            }
        ));
        assert_eq!(PartRef::of(Part::THUMB).item, None);
    }

    #[test]
    fn part_ids_follow_one_rule() {
        let w = WidgetId::of("w");
        assert_eq!(w.part(PartRef::of(Part::CONTAINER)), w);
        let row = PartRef::item(Part::ROW, ItemKey::index(3));
        assert_eq!(w.part(row), w.child(3), "row ids stay child ids");
        assert_eq!(
            w.part(PartRef::item(Part::TAB, ItemKey::index(3))),
            w.child(3)
        );
        let close = w.part(PartRef::item(Part::CLOSE, ItemKey::index(3)));
        assert_ne!(close, w.child(3));
        assert_ne!(close, w.part(PartRef::item(Part::CLOSE, ItemKey::index(4))));
        assert_ne!(
            w.part(PartRef::of(Part::NEW)),
            w.part(PartRef::of(Part::OVERFLOW))
        );
        let text = PartRef::item(Part::ROW, ItemKey::text("jk-1"));
        assert_eq!(w.part(text), w.part(text));
        assert_ne!(
            w.part(text),
            w.part(PartRef::item(Part::ROW, ItemKey::text("jk-2")))
        );
        assert_ne!(
            w.part(PartRef::item(Part::ROW, ItemKey::num(1))),
            w.part(PartRef::item(Part::ROW, ItemKey::index(1)))
        );
    }
}
