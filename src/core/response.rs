//! The one reply type of every widget and page, and the state flags a
//! widget wears.
//!
//! `flow` (was the input consumed?) and `invalidate` (what must be
//! redrawn?) are orthogonal, so "a wheel at a boundary is consumed without
//! a repaint" is expressible. The optional action replaces the six return
//! shapes in use today (`Outcome`, `(Outcome, bool)`, `bool`, `Option<E>`,
//! ...; docs/architecture/shared-components-plan.md §1.3). [`Outcome`]
//! stays as an adapter ([`Response::outcome`], `From<Outcome>`) until step
//! S15 removes it. Names and shapes follow termrock-core on `main`.

use std::ops::{BitAnd, BitOr, BitOrAssign, Not, Sub};

use super::event::Outcome;

/// Whether an input was consumed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord, Hash)]
pub enum Flow {
    /// Not interested; keep offering the input to the next handler.
    #[default]
    Ignored,
    /// Handled; stop propagation.
    Consumed,
}

/// What must be redrawn. Ordered: `None < Paint < Layout`.
///
/// `Layout` behaves as `Paint` today; it is reserved for layout caching.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord, Hash)]
pub enum Invalidate {
    /// Nothing visible changed.
    #[default]
    None,
    /// Repaint.
    Paint,
    /// Re-layout, then repaint.
    Layout,
}

/// The reply of every `update`: flow, invalidation and an optional action.
///
/// `#[must_use]`: dropping a response loses the consumed and repaint
/// answer the router needs.
#[must_use = "a Response carries the consumed/repaint answer; fold it with `|` or read it"]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response<A = ()> {
    flow: Flow,
    invalidate: Invalidate,
    action: Option<A>,
}

impl<A> Default for Response<A> {
    fn default() -> Self {
        Self::ignored()
    }
}

impl<A> Response<A> {
    /// Not consumed, nothing changed.
    pub const fn ignored() -> Self {
        Self {
            flow: Flow::Ignored,
            invalidate: Invalidate::None,
            action: None,
        }
    }

    /// Consumed without a repaint (the boundary-wheel rule).
    pub const fn consumed() -> Self {
        Self {
            flow: Flow::Consumed,
            invalidate: Invalidate::None,
            action: None,
        }
    }

    /// Consumed and repaint.
    pub const fn changed() -> Self {
        Self {
            flow: Flow::Consumed,
            invalidate: Invalidate::Paint,
            action: None,
        }
    }

    /// Consumed, repaint, and carry `action` (`main`: `Response::action`).
    pub const fn emit(action: A) -> Self {
        Self {
            flow: Flow::Consumed,
            invalidate: Invalidate::Paint,
            action: Some(action),
        }
    }

    /// Raise invalidation to at least `Paint`.
    pub fn repaint(mut self) -> Self {
        self.invalidate = self.invalidate.max(Invalidate::Paint);
        self
    }

    /// Raise invalidation to `Layout`.
    pub const fn relayout(mut self) -> Self {
        self.invalidate = Invalidate::Layout;
        self
    }

    /// Lower invalidation to `None` (the boundary-wheel rule).
    pub const fn no_repaint(mut self) -> Self {
        self.invalidate = Invalidate::None;
        self
    }

    /// The flow.
    pub const fn flow(&self) -> Flow {
        self.flow
    }

    /// Whether the input was consumed.
    pub fn is_consumed(&self) -> bool {
        self.flow == Flow::Consumed
    }

    /// The invalidation.
    pub const fn invalidate(&self) -> Invalidate {
        self.invalidate
    }

    /// Whether a repaint is needed (`invalidate >= Paint`).
    pub fn is_changed(&self) -> bool {
        self.invalidate >= Invalidate::Paint
    }

    /// The legacy [`Outcome`] for callers not yet migrated: `Changed` when
    /// a repaint is needed, else `Consumed` or `Ignored` by flow. Removed
    /// with `Outcome` in S15.
    pub fn outcome(&self) -> Outcome {
        match (self.flow, self.is_changed()) {
            (_, true) => Outcome::Changed,
            (Flow::Consumed, false) => Outcome::Consumed,
            (Flow::Ignored, false) => Outcome::Ignored,
        }
    }

    /// Borrow the action, if any.
    pub const fn action(&self) -> Option<&A> {
        self.action.as_ref()
    }

    /// Take the action out, leaving flow and invalidation in place.
    pub fn take_action(&mut self) -> Option<A> {
        self.action.take()
    }

    /// Consume the response for its action.
    pub fn into_action(self) -> Option<A> {
        self.action
    }

    /// Split into the legacy outcome and the action: the adapter for the
    /// `(Outcome, Option<E>)` call sites.
    pub fn into_parts(self) -> (Outcome, Option<A>) {
        (self.outcome(), self.action)
    }

    /// Attach `action` to this response; an `Ignored` response becomes
    /// consumed and repaints, as an action is always a visible result.
    pub fn with_action<B>(self, action: B) -> Response<B> {
        Response {
            flow: Flow::Consumed,
            invalidate: self.invalidate.max(Invalidate::Paint),
            action: Some(action),
        }
    }

    /// Translate the action at a composition boundary; flow and
    /// invalidation cross unchanged.
    pub fn map_action<B>(self, f: impl FnOnce(A) -> B) -> Response<B> {
        Response {
            flow: self.flow,
            invalidate: self.invalidate,
            action: self.action.map(f),
        }
    }

    /// Run `f` on the action, if any, and erase it.
    pub fn on_action(self, f: impl FnOnce(A)) -> Response<()> {
        if let Some(a) = self.action {
            f(a);
        }
        Response {
            flow: self.flow,
            invalidate: self.invalidate,
            action: None,
        }
    }

    /// Drop the action, keep flow and invalidation.
    pub fn erase(self) -> Response<()> {
        Response {
            flow: self.flow,
            invalidate: self.invalidate,
            action: None,
        }
    }
}

/// Adapter from the legacy [`Outcome`]: `Changed` repaints, `Consumed`
/// does not, `Ignored` keeps propagating. No action.
impl<A> From<Outcome> for Response<A> {
    fn from(o: Outcome) -> Self {
        match o {
            Outcome::Ignored => Self::ignored(),
            Outcome::Consumed => Self::consumed(),
            Outcome::Changed => Self::changed(),
        }
    }
}

/// Fold: `flow`, `Consumed` wins; `invalidate`, the maximum. Defined for
/// `Response<()>` only, so two actions are never silently merged.
impl BitOr for Response<()> {
    type Output = Response<()>;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}

impl BitOrAssign for Response<()> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.flow = self.flow.max(rhs.flow);
        self.invalidate = self.invalidate.max(rhs.invalidate);
    }
}

/// The visual state a widget or row wears: the single state input to
/// `Theme` and the row painter. Bit layout identical to `main`'s
/// `StateFlags` (termrock-core `response.rs`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct StateFlags(u16);

impl StateFlags {
    /// Owns keyboard focus.
    pub const FOCUSED: Self = Self(1 << 0);
    /// Focus should be painted (the last input was a key).
    pub const FOCUS_VISIBLE: Self = Self(1 << 1);
    /// The pointer is over it.
    pub const HOVERED: Self = Self(1 << 2);
    /// The primary button is down on it (or the press flash is live).
    pub const PRESSED: Self = Self(1 << 3);
    /// Selected within its collection.
    pub const SELECTED: Self = Self(1 << 4);
    /// The active item (the current tab, the current page, a chosen value).
    pub const ACTIVE: Self = Self(1 << 5);
    /// Checked (checkbox, toggle, multi-select).
    pub const CHECKED: Self = Self(1 << 6);
    /// Disabled: drawn but never reachable or activatable.
    pub const DISABLED: Self = Self(1 << 7);
    /// Read-only: reachable, never editable.
    pub const READ_ONLY: Self = Self(1 << 8);
    /// Carries a validation error.
    pub const ERROR: Self = Self(1 << 9);
    /// Carries a warning.
    pub const WARNING: Self = Self(1 << 10);
    /// Busy with an operation.
    pub const BUSY: Self = Self(1 << 11);
    /// An edit is in flight (the terminal cursor belongs to it).
    pub const EDITING: Self = Self(1 << 12);
    /// Has uncommitted changes.
    pub const DIRTY: Self = Self(1 << 13);
    /// Expanded (tree node, section).
    pub const EXPANDED: Self = Self(1 << 14);
    /// Loading data.
    pub const LOADING: Self = Self(1 << 15);

    const NAMES: [(Self, &'static str); 16] = [
        (Self::FOCUSED, "FOCUSED"),
        (Self::FOCUS_VISIBLE, "FOCUS_VISIBLE"),
        (Self::HOVERED, "HOVERED"),
        (Self::PRESSED, "PRESSED"),
        (Self::SELECTED, "SELECTED"),
        (Self::ACTIVE, "ACTIVE"),
        (Self::CHECKED, "CHECKED"),
        (Self::DISABLED, "DISABLED"),
        (Self::READ_ONLY, "READ_ONLY"),
        (Self::ERROR, "ERROR"),
        (Self::WARNING, "WARNING"),
        (Self::BUSY, "BUSY"),
        (Self::EDITING, "EDITING"),
        (Self::DIRTY, "DIRTY"),
        (Self::EXPANDED, "EXPANDED"),
        (Self::LOADING, "LOADING"),
    ];

    /// No flag set.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// The raw bits.
    pub const fn bits(self) -> u16 {
        self.0
    }

    /// Flags from raw bits (every bit is a defined flag).
    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    /// Whether no flag is set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Whether every flag in `other` is set.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether any flag in `other` is set.
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Both sets.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Set the flags in `other`.
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Clear the flags in `other`.
    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    /// Set or clear the flags in `other`.
    pub fn set(&mut self, other: Self, on: bool) {
        if on {
            self.insert(other);
        } else {
            self.remove(other);
        }
    }

    /// These flags with `other` set or cleared: the builder form of
    /// [`StateFlags::set`].
    pub const fn with(self, other: Self, on: bool) -> Self {
        if on {
            Self(self.0 | other.0)
        } else {
            Self(self.0 & !other.0)
        }
    }
}

impl BitOr for StateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for StateFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for StateFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl Sub for StateFlags {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0)
    }
}

impl Not for StateFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl std::fmt::Debug for StateFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return f.write_str("StateFlags(empty)");
        }
        f.write_str("StateFlags(")?;
        let mut first = true;
        for (flag, name) in Self::NAMES {
            if self.contains(flag) {
                if !first {
                    f.write_str(" | ")?;
                }
                f.write_str(name)?;
                first = false;
            }
        }
        f.write_str(")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_set_flow_and_invalidation() {
        let r: Response = Response::ignored();
        assert_eq!(
            (r.flow(), r.invalidate()),
            (Flow::Ignored, Invalidate::None)
        );
        let r: Response = Response::consumed();
        assert!(r.is_consumed() && !r.is_changed());
        let r: Response = Response::changed();
        assert!(r.is_consumed() && r.is_changed());
        let r = Response::emit(7);
        assert!(r.is_consumed() && r.is_changed());
        assert_eq!(r.action(), Some(&7));
        assert_eq!(Response::<u8>::default(), Response::ignored());
    }

    #[test]
    fn outcome_round_trips_through_the_adapter() {
        for o in [Outcome::Ignored, Outcome::Consumed, Outcome::Changed] {
            let r: Response<u8> = o.into();
            assert_eq!(r.outcome(), o);
            assert_eq!(r.action(), None);
        }
        assert_eq!(Response::emit(1).outcome(), Outcome::Changed);
        assert_eq!(
            Response::<()>::consumed().relayout().outcome(),
            Outcome::Changed
        );
        assert_eq!(
            Response::emit('x').into_parts(),
            (Outcome::Changed, Some('x'))
        );
    }

    #[test]
    fn fold_takes_consumed_and_the_larger_invalidation() {
        let a: Response = Response::consumed();
        let b: Response = Response::ignored().repaint();
        let folded = a.clone() | b.clone();
        assert_eq!(folded.flow(), Flow::Consumed);
        assert_eq!(folded.invalidate(), Invalidate::Paint);
        let mut c = a;
        c |= b;
        assert_eq!(c, folded, "`|=` agrees with `|`");
        assert!(Invalidate::None < Invalidate::Paint && Invalidate::Paint < Invalidate::Layout);
    }

    #[test]
    fn action_helpers_keep_flow_and_invalidation() {
        let r = Response::<u8>::consumed().with_action("go");
        assert!(r.is_changed(), "an action is a visible result");
        let mapped = Response::emit(2).map_action(|n: i32| n * 10);
        assert_eq!(mapped.action(), Some(&20));
        let mut seen = None;
        let erased = Response::emit(5).on_action(|n| seen = Some(n));
        assert_eq!(seen, Some(5));
        assert_eq!(erased, Response::<()>::changed());
        let mut r = Response::emit(9);
        assert_eq!(r.take_action(), Some(9));
        assert!(r.is_changed());
        assert_eq!(Response::emit(3).no_repaint().erase(), Response::consumed());
    }

    #[test]
    fn state_flags_combine_and_print() {
        let mut s = StateFlags::FOCUSED | StateFlags::HOVERED;
        assert!(s.contains(StateFlags::FOCUSED));
        assert!(!s.contains(StateFlags::FOCUSED | StateFlags::PRESSED));
        assert!(s.intersects(StateFlags::PRESSED | StateFlags::HOVERED));
        s.set(StateFlags::HOVERED, false);
        s.insert(StateFlags::DISABLED);
        assert_eq!(s, StateFlags::FOCUSED.with(StateFlags::DISABLED, true));
        assert_eq!(format!("{s:?}"), "StateFlags(FOCUSED | DISABLED)");
        assert_eq!(format!("{:?}", StateFlags::empty()), "StateFlags(empty)");
        assert_eq!((s - StateFlags::FOCUSED), StateFlags::DISABLED);
        assert_eq!(StateFlags::from_bits(s.bits()), s);
        assert_eq!(StateFlags::LOADING.bits(), 1 << 15, "main's bit layout");
        let all = StateFlags::NAMES
            .iter()
            .fold(StateFlags::empty(), |acc, (f, _)| acc | *f);
        assert_eq!(all.bits(), u16::MAX, "16 distinct flags");
    }
}
