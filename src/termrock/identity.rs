//! Control identity, source item keys, column/field keys, and monotonic revisions.
//!
//! Provides length-delimited composition for [`Id`] to guarantee zero namespace
//! collisions, stable identifiers for collections, and overflow-checked revisions.

use std::fmt;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

pub(crate) const fn fnv1a(mut hash: u64, bytes: &[u8]) -> u64 {
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}

/// Parse segment starting offsets in an encoded [`Id`] path.
fn parse_segment_offsets(s: &str) -> Vec<usize> {
    let mut offsets = Vec::new();
    let bytes = s.as_bytes();
    let mut idx = 0;

    while idx < bytes.len() {
        let seg_start = idx;
        offsets.push(seg_start);

        if seg_start > 0 && bytes[idx] == b'/' {
            idx += 1;
        }
        if idx + 2 <= bytes.len() && bytes[idx + 1] == b':' {
            idx += 2;
        }
        let len_start = idx;
        while idx < bytes.len() && bytes[idx] != b':' {
            idx += 1;
        }
        if idx >= bytes.len() {
            break;
        }
        let len_str = &s[len_start..idx];
        let Ok(data_len) = len_str.parse::<usize>() else {
            break;
        };
        idx += 1; // skip ':'
        idx = (idx + data_len).min(bytes.len());
    }

    offsets
}

/// Stable control identity with length-delimited composition to guarantee zero
/// namespace collisions.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id {
    path: String,
}

impl Id {
    /// Construct a new root identity from a static namespace.
    pub fn new(namespace: &'static str) -> Self {
        let mut path = String::with_capacity(namespace.len() + 16);
        path.push_str("r:");
        path.push_str(&namespace.len().to_string());
        path.push(':');
        path.push_str(namespace);
        Self { path }
    }

    /// Derive a named sub-control identity.
    pub fn sub(&self, name: &str) -> Self {
        let mut path = String::with_capacity(self.path.len() + name.len() + 16);
        path.push_str(&self.path);
        path.push_str("/s:");
        path.push_str(&name.len().to_string());
        path.push(':');
        path.push_str(name);
        Self { path }
    }

    /// Derive an indexed child identity for a dynamic source item.
    pub fn child(&self, key: ItemKey) -> Self {
        let mut path = String::with_capacity(self.path.len() + 32);
        path.push_str(&self.path);
        path.push_str("/c:");
        let k = key.as_u64();
        let s = k.to_string();
        path.push_str(&s.len().to_string());
        path.push(':');
        path.push_str(&s);
        Self { path }
    }

    /// Return the parent identity, if any.
    pub fn parent(&self) -> Option<Id> {
        let offsets = parse_segment_offsets(&self.path);
        if offsets.len() <= 1 {
            None
        } else {
            let last_offset = offsets[offsets.len() - 1];
            Some(Id {
                path: self.path[..last_offset].to_string(),
            })
        }
    }

    /// Return string representation of the encoded identity path.
    pub fn as_str(&self) -> &str {
        &self.path
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({:?})", self.path)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

/// Stable identity for dynamic items or rows.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ItemKey(pub u64);

impl ItemKey {
    /// Construct a new ItemKey from a raw integer.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Hash a string slice to produce an ItemKey.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        Self(fnv1a(FNV_OFFSET, s.as_bytes()))
    }

    /// Return the raw integer value.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Debug for ItemKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ItemKey({})", self.0)
    }
}

impl fmt::Display for ItemKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for ItemKey {
    fn from(val: u64) -> Self {
        Self::new(val)
    }
}

impl From<&str> for ItemKey {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

/// Stable identity for grid columns.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ColumnKey(pub u64);

impl ColumnKey {
    /// Construct a new ColumnKey from a raw integer.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Hash a static column name to produce a ColumnKey.
    pub fn from_name(name: &'static str) -> Self {
        Self(fnv1a(FNV_OFFSET, name.as_bytes()))
    }

    /// Return the raw integer value.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Debug for ColumnKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ColumnKey({})", self.0)
    }
}

impl fmt::Display for ColumnKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Stable identity for a cell identified by row item key and column key.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct CellKey {
    pub row: ItemKey,
    pub col: ColumnKey,
}

impl CellKey {
    pub const fn new(row: ItemKey, col: ColumnKey) -> Self {
        Self { row, col }
    }
}

impl fmt::Debug for CellKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CellKey({}, {})", self.row.0, self.col.0)
    }
}

impl fmt::Display for CellKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.row.0, self.col.0)
    }
}

/// Stable identity for form fields.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct FieldKey(pub u64);

impl FieldKey {
    /// Construct a new FieldKey from a raw integer.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Hash a static field name to produce a FieldKey.
    pub fn from_name(name: &'static str) -> Self {
        Self(fnv1a(FNV_OFFSET, name.as_bytes()))
    }

    /// Return the raw integer value.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Debug for FieldKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FieldKey({})", self.0)
    }
}

impl fmt::Display for FieldKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Stable semantic command identity.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActionKey(&'static str);

impl ActionKey {
    /// Construct a new ActionKey from a static command name.
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// Return the static command name.
    pub const fn as_str(&self) -> &'static str {
        self.0
    }

    /// Return the static command name.
    pub const fn name(&self) -> &'static str {
        self.0
    }
}

impl fmt::Debug for ActionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ActionKey({:?})", self.0)
    }
}

impl fmt::Display for ActionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Revision overflow error when counter exceeds `u64::MAX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionError {
    Overflow,
}

impl fmt::Display for RevisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => write!(f, "revision counter overflow"),
        }
    }
}

impl std::error::Error for RevisionError {}

/// Monotonic accepted source generation.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Revision(pub u64);

impl Revision {
    /// Construct a new revision with a raw value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Starting revision generation zero.
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Advance to the next monotonic generation, failing closed on overflow.
    pub fn next(&self) -> Result<Self, RevisionError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(RevisionError::Overflow)
    }

    /// Check if this revision is older than `current`.
    pub fn is_stale(&self, current: Revision) -> bool {
        self.0 < current.0
    }

    /// Return the raw revision counter.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Debug for Revision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Revision({})", self.0)
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Standard or custom component part name.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Part(&'static str);

impl Part {
    pub const ROOT: Part = Part("root");
    pub const BODY: Part = Part("body");
    pub const HEADER: Part = Part("header");
    pub const LABEL: Part = Part("label");
    pub const ICON: Part = Part("icon");
    pub const BADGE: Part = Part("badge");
    pub const SCROLLBAR: Part = Part("scrollbar");
    pub const TRACK: Part = Part("track");
    pub const THUMB: Part = Part("thumb");
    pub const SURFACE: Part = Part("surface");
    pub const BACKDROP: Part = Part("backdrop");
    pub const BORDER: Part = Part("border");
    pub const TITLE: Part = Part("title");
    pub const META: Part = Part("meta");
    pub const SEAM: Part = Part("seam");
    pub const CONTAINER: Part = Part("container");
    pub const ROW: Part = Part("row");
    pub const CELL: Part = Part("cell");
    pub const GUTTER: Part = Part("gutter");
    pub const CURSOR: Part = Part("cursor");
    pub const SELECTION: Part = Part("selection");
    pub const EDITOR: Part = Part("editor");
    pub const SORT_MARKER: Part = Part("sort-marker");
    pub const FOOTER: Part = Part("footer");
    pub const FADE: Part = Part("fade");
    pub const EMPTY: Part = Part("empty");
    pub const LINE_NUMBER: Part = Part("line-number");
    pub const CURRENT_LINE: Part = Part("current-line");
    pub const TEXT: Part = Part("text");
    pub const SYNTAX: Part = Part("syntax");
    pub const DIAGNOSTIC: Part = Part("diagnostic");
    pub const FIND: Part = Part("find");
    pub const COMPLETION: Part = Part("completion");
    pub const OLD_GUTTER: Part = Part("old-gutter");
    pub const NEW_GUTTER: Part = Part("new-gutter");
    pub const CONTEXT: Part = Part("context");
    pub const ADDITION: Part = Part("addition");
    pub const DELETION: Part = Part("deletion");
    pub const EMPHASIS: Part = Part("emphasis");

    /// Construct a new part name.
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// Return the static part name.
    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

impl fmt::Debug for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Part({:?})", self.0)
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Component part reference with optional item identity for collections.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PartRef {
    pub owner: Id,
    pub part: Part,
    pub item: Option<ItemKey>,
}

impl PartRef {
    /// Construct a new part reference.
    pub fn new(owner: Id, part: Part, item: Option<ItemKey>) -> Self {
        Self { owner, part, item }
    }
}

/// Trait for items with a stable source key.
pub trait Keyed {
    fn key(&self) -> ItemKey;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_namespace_collision() {
        let id1 = Id::new("a").sub("b");
        let id2 = Id::new("ab").sub("");
        assert_ne!(
            id1, id2,
            "length delimited composition must prevent collision"
        );

        let id3 = Id::new("a").child(ItemKey::new(1));
        let id4 = Id::new("a").sub("1");
        assert_ne!(id3, id4, "child item vs sub name must not collide");

        let id5 = Id::new("a/s:1:b");
        assert_ne!(
            id1, id5,
            "embedded tag must not collide with structured path"
        );
    }

    #[test]
    fn parent_resolution() {
        let root = Id::new("table");
        assert_eq!(root.parent(), None);

        let sub = root.sub("header");
        assert_eq!(sub.parent(), Some(root.clone()));

        let child = sub.child(ItemKey::new(42));
        assert_eq!(child.parent(), Some(sub.clone()));

        let sub_child = child.sub("title");
        assert_eq!(sub_child.parent(), Some(child.clone()));
    }

    #[test]
    fn keys_and_actions() {
        let ik = ItemKey::from_str("row-1");
        assert_eq!(ik, ItemKey::from_str("row-1"));
        assert_ne!(ik, ItemKey::from_str("row-2"));
        assert_eq!(ik.as_u64(), ItemKey::new(ik.as_u64()).as_u64());

        let ck = ColumnKey::from_name("age");
        assert_eq!(ck, ColumnKey::from_name("age"));
        assert_ne!(ck, ColumnKey::from_name("name"));

        let fk = FieldKey::from_name("email");
        assert_eq!(fk, FieldKey::from_name("email"));

        let ak = ActionKey::new("commit");
        assert_eq!(ak.as_str(), "commit");
    }

    #[test]
    fn revision_monotonicity_and_overflow() {
        let r0 = Revision::zero();
        assert_eq!(r0.as_u64(), 0);

        let r1 = r0.next().unwrap();
        assert_eq!(r1.as_u64(), 1);
        assert!(r0.is_stale(r1));
        assert!(!r1.is_stale(r0));
        assert!(!r1.is_stale(r1));

        let r_max = Revision::new(u64::MAX);
        assert_eq!(r_max.next(), Err(RevisionError::Overflow));
    }

    #[test]
    fn part_and_part_ref() {
        let id = Id::new("form");
        let pref = PartRef::new(id.clone(), Part::LABEL, Some(ItemKey::new(10)));
        assert_eq!(pref.owner, id);
        assert_eq!(pref.part, Part::LABEL);
        assert_eq!(pref.item, Some(ItemKey::new(10)));
    }
}
