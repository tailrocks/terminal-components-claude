//! Stable widget identifiers.
//!
//! Every interactive element owns a [`WidgetId`]. Ids are derived from a
//! human readable path (`"forms.name"`) so they are stable across frames and
//! easy to debug, and can be extended with numeric children for repeated rows.
//!
//! A [`PartRef`] names a sub-region of one widget (a row, a thumb, a close
//! box) by [`Part`] and an optional [`ItemKey`]; it is what a pointer
//! [`Intent`](super::intent::Intent) carries instead of a raw hit id.

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
    /// Build an id from a static path such as `"buttons.primary"`.
    pub const fn of(path: &str) -> Self {
        Self(fnv1a(FNV_OFFSET, path.as_bytes()))
    }

    /// Derive a child id, e.g. one per table row or list item.
    pub const fn child(self, index: usize) -> Self {
        Self(fnv1a(self.0, &index.to_le_bytes()))
    }

    /// Derive a named child id.
    pub const fn sub(self, name: &str) -> Self {
        Self(fnv1a(self.0, name.as_bytes()))
    }

    /// The id of one part of this widget: the single rule every widget
    /// and [`HitRegistry::register_part`](super::hit::HitRegistry::register_part)
    /// derive part ids with, so no widget hand-rolls its own.
    ///
    /// The widget's own surface (`Part::CONTAINER`, no item) is the widget
    /// id itself, and a collection's item (`Part::ROW` or `Part::TAB` with
    /// an [`ItemKey::Index`]) is [`Self::child`], so row ids stay what they
    /// always were. Every other part hashes the item, then the part number.
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

    /// The inverse of [`Self::part`] over a known set of positional items:
    /// which `i` in `items` makes `id` this widget's `part` `i`. For a
    /// collection whose items all exist whether drawn or not (tabs, menu
    /// rows, select options); a widget that only knows what it drew reads
    /// its [`PartMap`](super::hit::PartMap) instead.
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

/// FNV-1a over `name`, folded into the custom half of a 16-bit key space.
///
/// Every 16-bit custom key namespace ([`Part::custom`],
/// [`ActionKey::custom`](super::intent::ActionKey::custom)) uses this one
/// rule: `0..=0x7FFF` is reserved for numbered keys and `0x8000..=0xFFFF`
/// for hashed names, so a numbered key and a named key are never the same
/// key. Two names can still fold to the same 15 bits.
pub const fn custom_hash16(name: &str) -> u16 {
    let h = fnv1a(FNV_OFFSET, name.as_bytes());
    0x8000 | ((h ^ (h >> 16) ^ (h >> 32) ^ (h >> 48)) as u16 & 0x7FFF)
}

/// A stable key for one item of a collection (a row, a tab, a chip).
///
/// Collection parts carry an `ItemKey`, never a raw hit id. Same shape as
/// `main` (termrock-core `id.rs`, plan §1.9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum ItemKey {
    /// A positional key: UNSTABLE under insert, remove and reorder.
    Index(usize),
    /// A numeric domain key (a database id, a handle).
    Num(u64),
    /// A hashed textual or composite key.
    Text(u64),
}

impl ItemKey {
    /// A positional key. UNSTABLE under insert, remove and reorder.
    pub const fn index(i: usize) -> Self {
        ItemKey::Index(i)
    }

    /// A numeric key.
    pub const fn num(n: u64) -> Self {
        ItemKey::Num(n)
    }

    /// A textual key: FNV-1a over the bytes.
    pub const fn text(s: &str) -> Self {
        ItemKey::Text(fnv1a(FNV_OFFSET, s.as_bytes()))
    }

    /// A composite key (schema + table, provider + account); order-sensitive.
    pub const fn pair(a: u64, b: u64) -> Self {
        let h = fnv1a(FNV_OFFSET, &a.to_le_bytes());
        ItemKey::Text(fnv1a(h, &b.to_le_bytes()))
    }

    /// The position of an [`ItemKey::Index`] key.
    pub const fn as_index(self) -> Option<usize> {
        match self {
            ItemKey::Index(i) => Some(i),
            _ => None,
        }
    }
}

/// A typed logical part of a widget (`Part::ROW`, `Part::THUMB`, ...).
///
/// `0..=255` is the library range; [`Part::custom`] maps an app's name into
/// `0x8000..=0xFFFF`. Equality is structural, so `PartRef { part:
/// Part::ROW, .. }` works as a `match` pattern. The numbers match `main`
/// exactly (termrock-core `id.rs`), so a part means the same on both.
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
    /// The whole widget surface.
    CONTAINER = 0,
    /// A frame or border.
    BORDER = 1,
    /// A dimmed backdrop behind a layer.
    BACKDROP = 2,
    /// The focus gutter column.
    GUTTER = 3,
    /// A selection or check marker.
    MARKER = 4,
    /// A leading icon.
    ICON = 5,
    /// The main label.
    LABEL = 6,
    /// Secondary text.
    META = 7,
    /// Help or caption text.
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
    /// A header row or header cell.
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
    /// A row change-state glyph, separate from selection.
    CHANGE = 34,
    /// A row number.
    ROW_NUMBER = 35,
}

impl Part {
    /// A custom part named by an app or a downstream widget; lands in the
    /// high range. Two names can fold to the same part (15 bits).
    pub const fn custom(name: &str) -> Part {
        Part(custom_hash16(name))
    }

    /// The raw part number.
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

/// A sub-region of one widget: a part plus an optional item key. Never
/// interchangeable with a derived [`WidgetId`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PartRef {
    /// The part.
    pub part: Part,
    /// The item, for per-row parts.
    pub item: Option<ItemKey>,
}

impl PartRef {
    /// The overflow indicator before the shown window (a tab strip's `‹N`,
    /// a scrolled menu's ▲, a grid's hidden columns on the left). The
    /// `+N` button of a whole overflow is `PartRef::of(Part::OVERFLOW)`.
    pub const OVERFLOW_BEFORE: PartRef = PartRef::item(Part::OVERFLOW, ItemKey::Index(0));
    /// The overflow indicator after the shown window (`N›`, ▼).
    pub const OVERFLOW_AFTER: PartRef = PartRef::item(Part::OVERFLOW, ItemKey::Index(1));

    /// A part without an item.
    pub const fn of(part: Part) -> Self {
        Self { part, item: None }
    }

    /// A per-item part.
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
