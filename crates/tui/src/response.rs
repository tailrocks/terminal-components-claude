//! The one reply type (`COMPONENT_ARCHITECTURE.md` §6.1, §21 item 4).
//!
//! `flow` (was the event consumed?) and `invalidate` (what must be redrawn?)
//! are orthogonal, which is what makes "a wheel at a boundary is consumed
//! without a repaint" expressible. Folding with `|` is defined for
//! `Response<()>` only: composing two action-carrying responses is a type
//! error, never silent loss.

use core::ops::{BitOr, BitOrAssign};

use bitflags::bitflags;

use crate::id::Id;

/// Whether an input was consumed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord)]
pub enum Flow {
    /// Not interested; keep offering the input to the next handler.
    #[default]
    Ignored,
    /// Handled; stop propagation.
    Consumed,
}

/// What must be redrawn. Ordered: `None < Paint < Layout`.
///
/// `Layout` ships from day one but currently behaves as `Paint`; it is
/// reserved for layout caching and only its ordering is asserted (§8.5).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord)]
pub enum Invalidate {
    /// Nothing visible changed.
    #[default]
    None,
    /// Repaint.
    Paint,
    /// Re-layout, then repaint.
    Layout,
}

bitflags! {
    /// The visual state a component wears, as resolved by the runtime and
    /// declared by the component.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
    pub struct StateFlags: u16 {
        /// Owns keyboard focus.
        const FOCUSED       = 1 << 0;
        /// Focus should be painted (the last input was a key).
        const FOCUS_VISIBLE = 1 << 1;
        /// The pointer is over it.
        const HOVERED       = 1 << 2;
        /// The primary button is down on it (or the press flash is live).
        const PRESSED       = 1 << 3;
        /// Selected within its collection.
        const SELECTED      = 1 << 4;
        /// The active item (tab, cursor row).
        const ACTIVE        = 1 << 5;
        /// Checked (checkbox, toggle, multi-select).
        const CHECKED       = 1 << 6;
        /// Disabled: registered but never reachable or activatable.
        const DISABLED      = 1 << 7;
        /// Read-only: reachable, never editable.
        const READ_ONLY     = 1 << 8;
        /// Carries a validation error.
        const ERROR         = 1 << 9;
        /// Carries a warning.
        const WARNING       = 1 << 10;
        /// Busy with an operation.
        const BUSY          = 1 << 11;
        /// An edit is in flight (the hardware cursor belongs to it).
        const EDITING       = 1 << 12;
        /// Has uncommitted changes.
        const DIRTY         = 1 << 13;
        /// Expanded (tree node, section).
        const EXPANDED      = 1 << 14;
        /// Loading data.
        const LOADING       = 1 << 15;
    }
}

/// The reply of every `update`.
///
/// `#[must_use]`: dropping a response silently loses the consumed / repaint
/// answer the runtime needs.
#[must_use = "a Response carries the consumed/repaint answer; fold it with `|` or read it"]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response<A = ()> {
    id: Option<Id>,
    flow: Flow,
    invalidate: Invalidate,
    state: StateFlags,
    action: Option<A>,
}

impl<A> Default for Response<A> {
    fn default() -> Self {
        Self::ignored()
    }
}

impl<A> Response<A> {
    /// Not consumed, nothing changed, no id.
    pub const fn ignored() -> Self {
        Response {
            id: None,
            flow: Flow::Ignored,
            invalidate: Invalidate::None,
            state: StateFlags::empty(),
            action: None,
        }
    }

    /// Consumed without a repaint (the boundary-wheel rule).
    pub const fn consumed() -> Self {
        Response {
            id: None,
            flow: Flow::Consumed,
            invalidate: Invalidate::None,
            state: StateFlags::empty(),
            action: None,
        }
    }

    /// Consumed and repaint.
    pub const fn changed() -> Self {
        Response {
            id: None,
            flow: Flow::Consumed,
            invalidate: Invalidate::Paint,
            state: StateFlags::empty(),
            action: None,
        }
    }

    /// Consumed, repaint, and carry an action.
    pub const fn action(a: A) -> Self {
        Response {
            id: None,
            flow: Flow::Consumed,
            invalidate: Invalidate::Paint,
            state: StateFlags::empty(),
            action: Some(a),
        }
    }

    /// Tag the response with the component that produced it.
    pub const fn for_id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Record the state the component ended in.
    pub const fn with_state(mut self, s: StateFlags) -> Self {
        self.state = s;
        self
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

    /// The producing component, `None` for [`Response::ignored`].
    pub const fn id(&self) -> Option<Id> {
        self.id
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

    /// The recorded state flags.
    pub const fn state(&self) -> StateFlags {
        self.state
    }

    /// Whether the recorded state carries `FOCUSED`.
    pub const fn focused(&self) -> bool {
        self.state.contains(StateFlags::FOCUSED)
    }

    /// Whether the recorded state carries `HOVERED`.
    pub const fn hovered(&self) -> bool {
        self.state.contains(StateFlags::HOVERED)
    }

    /// Whether the recorded state carries `PRESSED`.
    pub const fn pressed(&self) -> bool {
        self.state.contains(StateFlags::PRESSED)
    }

    /// Borrow the action, if any.
    pub const fn action_ref(&self) -> Option<&A> {
        self.action.as_ref()
    }

    /// Take the action out, leaving flow and invalidation.
    pub const fn take_action(&mut self) -> Option<A> {
        self.action.take()
    }

    /// Consume the response for its action.
    pub fn into_action(self) -> Option<A> {
        self.action
    }

    /// Translate the action at a composition boundary; flow and
    /// invalidation are preserved.
    pub fn map_action<B>(self, f: impl FnOnce(A) -> B) -> Response<B> {
        Response {
            id: self.id,
            flow: self.flow,
            invalidate: self.invalidate,
            state: self.state,
            action: self.action.map(f),
        }
    }

    /// Run `f` on the action, if any, and erase it.
    pub fn on_action(self, f: impl FnOnce(A)) -> Response<()> {
        let erased = Response {
            id: self.id,
            flow: self.flow,
            invalidate: self.invalidate,
            state: self.state,
            action: None,
        };
        if let Some(a) = self.action {
            f(a);
        }
        erased
    }

    /// Drop the action, keep flow and invalidation.
    pub fn erase(self) -> Response<()> {
        self.on_action(|_| {})
    }
}

/// The unit action of buttons, menu items and chips.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Activated;

impl Response<Activated> {
    /// Whether the control fired.
    pub const fn activated(&self) -> bool {
        self.action.is_some()
    }

    /// Run `f` iff the control fired, then erase the action.
    pub fn on_activated(self, f: impl FnOnce()) -> Response<()> {
        self.on_action(|Activated| f())
    }
}

/// Fold: `flow` — `Consumed` wins; `invalidate` — max; `id` and `state` —
/// the left-hand side. The fold is a control-flow summary; read `state` and
/// `id` from the individual responses.
impl BitOr for Response<()> {
    type Output = Response<()>;

    fn bitor(self, rhs: Self) -> Self::Output {
        Response {
            id: self.id.or(rhs.id),
            flow: self.flow.max(rhs.flow),
            invalidate: self.invalidate.max(rhs.invalidate),
            state: self.state,
            action: None,
        }
    }
}

impl BitOrAssign for Response<()> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.flow = self.flow.max(rhs.flow);
        self.invalidate = self.invalidate.max(rhs.invalidate);
        if self.id.is_none() {
            self.id = rhs.id;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignored_consumed_changed_action_constructors() {
        let i: Response<u8> = Response::ignored();
        assert!(!i.is_consumed() && !i.is_changed() && i.id().is_none());
        let c: Response<u8> = Response::consumed();
        assert!(c.is_consumed() && !c.is_changed());
        let ch: Response<u8> = Response::changed();
        assert!(ch.is_consumed() && ch.is_changed());
        let a = Response::action(7u8);
        assert!(a.is_consumed() && a.is_changed());
        assert_eq!(a.action_ref(), Some(&7));
        let r = a.for_id(Id::root("x"));
        assert_eq!(r.id(), Some(Id::root("x")));
    }

    #[test]
    fn bitor_takes_consumed_over_ignored() {
        let r = Response::<()>::ignored() | Response::consumed();
        assert!(r.is_consumed());
        let r = Response::<()>::consumed() | Response::ignored();
        assert!(r.is_consumed());
        let mut r = Response::<()>::ignored();
        r |= Response::consumed();
        assert!(r.is_consumed());
    }

    #[test]
    fn bitor_takes_max_invalidate() {
        let r = Response::<()>::consumed() | Response::changed();
        assert_eq!(r.invalidate(), Invalidate::Paint);
        let r = Response::<()>::changed().relayout() | Response::consumed();
        assert_eq!(r.invalidate(), Invalidate::Layout);
        let mut r = Response::<()>::ignored();
        r |= Response::changed();
        assert_eq!(r.invalidate(), Invalidate::Paint);
    }

    #[test]
    fn repaint_raises_relayout_raises_further() {
        let r: Response<()> = Response::consumed().repaint();
        assert_eq!(r.invalidate(), Invalidate::Paint);
        let r = r.relayout();
        assert_eq!(r.invalidate(), Invalidate::Layout);
        // repaint never lowers
        assert_eq!(r.repaint().invalidate(), Invalidate::Layout);
    }

    #[test]
    fn layout_is_strictly_greater_than_paint() {
        assert!(Invalidate::None < Invalidate::Paint);
        assert!(Invalidate::Paint < Invalidate::Layout);
    }

    #[test]
    fn no_repaint_lowers_to_none() {
        let r: Response<()> = Response::changed().no_repaint();
        assert!(r.is_consumed());
        assert_eq!(r.invalidate(), Invalidate::None);
        assert!(!r.is_changed());
    }

    #[test]
    fn map_action_preserves_flow_and_invalidate() {
        let r = Response::action(3u8).relayout().for_id(Id::root("m"));
        let m = r.map_action(|n| u32::from(n).saturating_mul(2));
        assert_eq!(m.action_ref(), Some(&6));
        assert_eq!(m.invalidate(), Invalidate::Layout);
        assert!(m.is_consumed());
        assert_eq!(m.id(), Some(Id::root("m")));
    }

    #[test]
    fn erase_drops_the_action_only() {
        let r = Response::action(1u8)
            .with_state(StateFlags::FOCUSED)
            .erase();
        assert!(r.is_consumed() && r.is_changed());
        assert!(r.focused());
        let mut a = Response::action(Activated);
        assert!(a.activated());
        assert_eq!(a.take_action(), Some(Activated));
        assert!(!a.activated());
        let mut fired = false;
        let r = Response::action(Activated).on_activated(|| fired = true);
        assert!(fired && r.is_changed());
    }

    #[test]
    fn state_flags_round_trip() {
        let s = StateFlags::FOCUSED | StateFlags::HOVERED | StateFlags::EDITING;
        assert_eq!(StateFlags::from_bits_truncate(s.bits()), s);
        assert_eq!(s.iter().count(), 3);
        assert_eq!(StateFlags::all().bits(), u16::MAX);
        assert_eq!(format!("{s:?}"), "StateFlags(FOCUSED | HOVERED | EDITING)");
        let r: Response<()> = Response::consumed().with_state(s);
        assert!(r.focused() && r.hovered() && !r.pressed());
    }
}
