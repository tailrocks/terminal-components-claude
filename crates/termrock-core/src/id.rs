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
