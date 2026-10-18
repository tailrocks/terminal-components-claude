#![doc = r" The one reply type (`COMPONENT_ARCHITECTURE.md` §6.1, §21 item 4)."]
#![doc = r""]
#![doc = r" `flow` (was the event consumed?) and `invalidate` (what must be redrawn?)"]
#![doc = r#" are orthogonal, which is what makes "a wheel at a boundary is consumed"#]
#![doc = r#" without a repaint" expressible. Folding with `|` is defined for"#]
#![doc = r" `Response<()>` only: composing two action-carrying responses is a type"]
#![doc = r" error, never silent loss."]

use core::ops::{BitOr, BitOrAssign};

use bitflags::bitflags;

use crate::id::Id;

#[doc = r" Whether an input was consumed."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord)]
pub enum Flow {
    #[doc = r" Not interested; keep offering the input to the next handler."]
    #[default]
    Ignored,
    #[doc = r" Handled; stop propagation."]
    Consumed,
}

#[doc = r" What must be redrawn. Ordered: `None < Paint < Layout`."]
#[doc = r""]
#[doc = r" `Layout` ships from day one but currently behaves as `Paint`; it is"]
#[doc = r" reserved for layout caching and only its ordering is asserted (§8.5)."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord)]
pub enum Invalidate {
    #[doc = r" Nothing visible changed."]
    #[default]
    None,
    #[doc = r" Repaint."]
    Paint,
    #[doc = r" Re-layout, then repaint."]
    Layout,
}

bitflags! {
    #[doc = r" The visual state a component wears, as resolved by the runtime and"]
    #[doc = r" declared by the component."]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
    pub struct StateFlags: u16 {
        #[doc = r" Owns keyboard focus."]
        const FOCUSED       = 1 << 0;
        #[doc = r" Focus should be painted (the last input was a key)."]
        const FOCUS_VISIBLE = 1 << 1;
        #[doc = r" The pointer is over it."]
        const HOVERED       = 1 << 2;
        #[doc = r" The primary button is down on it (or the press flash is live)."]
        const PRESSED       = 1 << 3;
        #[doc = r" Selected within its collection."]
        const SELECTED      = 1 << 4;
        #[doc = r" The active item (tab, cursor row)."]
        const ACTIVE        = 1 << 5;
        #[doc = r" Checked (checkbox, toggle, multi-select)."]
        const CHECKED       = 1 << 6;
        #[doc = r" Disabled: registered but never reachable or activatable."]
        const DISABLED      = 1 << 7;
        #[doc = r" Read-only: reachable, never editable."]
        const READ_ONLY     = 1 << 8;
        #[doc = r" Carries a validation error."]
        const ERROR         = 1 << 9;
        #[doc = r" Carries a warning."]
        const WARNING       = 1 << 10;
        #[doc = r" Busy with an operation."]
        const BUSY          = 1 << 11;
        #[doc = r" An edit is in flight (the hardware cursor belongs to it)."]
        const EDITING       = 1 << 12;
        #[doc = r" Has uncommitted changes."]
        const DIRTY         = 1 << 13;
        #[doc = r" Expanded (tree node, section)."]
        const EXPANDED      = 1 << 14;
        #[doc = r" Loading data."]
        const LOADING       = 1 << 15;
    }
}

#[doc = r" The reply of every `update`."]
#[doc = r""]
#[doc = r" `#[must_use]`: dropping a response silently loses the consumed / repaint"]
#[doc = r" answer the runtime needs."]
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
    #[doc = r" Not consumed, nothing changed, no id."]
    pub const fn ignored() -> Self {
        Response {
            id: None,
            flow: Flow::Ignored,
            invalidate: Invalidate::None,
            state: StateFlags::empty(),
            action: None,
        }
    }

    #[doc = r" Consumed without a repaint (the boundary-wheel rule)."]
    pub const fn consumed() -> Self {
        Response {
            id: None,
            flow: Flow::Consumed,
            invalidate: Invalidate::None,
            state: StateFlags::empty(),
            action: None,
        }
    }

    #[doc = r" Consumed and repaint."]
    pub const fn changed() -> Self {
        Response {
            id: None,
            flow: Flow::Consumed,
            invalidate: Invalidate::Paint,
            state: StateFlags::empty(),
            action: None,
        }
    }

    #[doc = r" Consumed, repaint, and carry an action."]
    pub const fn action(a: A) -> Self {
        Response {
            id: None,
            flow: Flow::Consumed,
            invalidate: Invalidate::Paint,
            state: StateFlags::empty(),
            action: Some(a),
        }
    }

    #[doc = r" Tag the response with the component that produced it."]
    pub const fn for_id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    #[doc = r" Record the state the component ended in."]
    pub const fn with_state(mut self, s: StateFlags) -> Self {
        self.state = s;
        self
    }

    #[doc = r" Raise invalidation to at least `Paint`."]
    pub fn repaint(mut self) -> Self {
        self.invalidate = self.invalidate.max(Invalidate::Paint);
        self
    }

    #[doc = r" Raise invalidation to `Layout`."]
    pub const fn relayout(mut self) -> Self {
        self.invalidate = Invalidate::Layout;
        self
    }

    #[doc = r" Lower invalidation to `None` (the boundary-wheel rule)."]
    pub const fn no_repaint(mut self) -> Self {
        self.invalidate = Invalidate::None;
        self
    }

    #[doc = r" The component that produced this response, or `None` when nothing"]
    #[doc = r" tagged it."]
    #[doc = r""]
    #[doc = r" Every constructor starts untagged — [`Response::action`] included —"]
    #[doc = r" and [`Response::for_id`] is the only writer, so `id()` answers `None`"]
    #[doc = r" for a consumed, repainting, action-carrying response that simply never"]
    #[doc = r#" named itself. It is not a "was this ignored?" test; read"#]
    #[doc = r" [`Response::flow`] for that."]
    pub const fn id(&self) -> Option<Id> {
        self.id
    }

    #[doc = r" The flow."]
    pub const fn flow(&self) -> Flow {
        self.flow
    }

    #[doc = r" Whether the input was consumed."]
    pub fn is_consumed(&self) -> bool {
        self.flow == Flow::Consumed
    }

    #[doc = r" The invalidation."]
    pub const fn invalidate(&self) -> Invalidate {
        self.invalidate
    }

    #[doc = r" Whether a repaint is needed (`invalidate >= Paint`)."]
    pub fn is_changed(&self) -> bool {
        self.invalidate >= Invalidate::Paint
    }

    #[doc = r" The recorded state flags."]
    pub const fn state(&self) -> StateFlags {
        self.state
    }

    #[doc = r" Whether the recorded state carries `FOCUSED`."]
    pub const fn focused(&self) -> bool {
        self.state.contains(StateFlags::FOCUSED)
    }

    #[doc = r" Whether the recorded state carries `HOVERED`."]
    pub const fn hovered(&self) -> bool {
        self.state.contains(StateFlags::HOVERED)
    }

    #[doc = r" Whether the recorded state carries `PRESSED`."]
    pub const fn pressed(&self) -> bool {
        self.state.contains(StateFlags::PRESSED)
    }

    #[doc = r" Borrow the action, if any."]
    pub const fn action_ref(&self) -> Option<&A> {
        self.action.as_ref()
    }

    #[doc = r" Take the action out, leaving flow, invalidation, state and id in place."]
    pub const fn take_action(&mut self) -> Option<A> {
        self.action.take()
    }

    #[doc = r" Consume the response for its action."]
    pub fn into_action(self) -> Option<A> {
        self.action
    }

    #[doc = r" Translate the action at a composition boundary."]
    #[doc = r""]
    #[doc = r" Every other field crosses unchanged: flow, invalidation, the recorded"]
    #[doc = r" state flags and the id. Only the action is rewritten, so a parent may"]
    #[doc = r" re-type a child's action without losing the child's identity."]
    pub fn map_action<B>(self, f: impl FnOnce(A) -> B) -> Response<B> {
        Response {
            id: self.id,
            flow: self.flow,
            invalidate: self.invalidate,
            state: self.state,
            action: self.action.map(f),
        }
    }

    #[doc = r" Run `f` on the action, if any, and erase it."]
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

    #[doc = r" Drop the action, keep flow and invalidation."]
    pub fn erase(self) -> Response<()> {
        self.on_action(|_| {})
    }
}

#[doc = r" The unit action of buttons, menu items and chips."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Activated;

impl Response<Activated> {
    #[doc = r" Whether the control fired."]
    pub const fn activated(&self) -> bool {
        self.action.is_some()
    }

    #[doc = r" Run `f` iff the control fired, then erase the action."]
    pub fn on_activated(self, f: impl FnOnce()) -> Response<()> {
        self.on_action(|Activated| f())
    }
}

#[doc = r" Fold: `flow` — `Consumed` wins; `invalidate` — max; `id` — the left-hand"]
#[doc = r" side if it carries one, otherwise the right-hand side; `state` — the"]
#[doc = r" left-hand side; `action` — cleared. The fold is a control-flow summary;"]
#[doc = r" read `state`, `id` and the action from the individual responses."]
#[doc = r""]
#[doc = r" [`BitOrAssign`] is defined to agree with this field by field, so"]
#[doc = r" `a = a | b` and `a |= b` are interchangeable."]
impl BitOr for Response<()> {
    type Output = Response<()>;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}

#[doc = r" The in-place form of [`BitOr`], field for field."]
impl BitOrAssign for Response<()> {
    fn bitor_assign(&mut self, rhs: Self) {
        if self.id.is_none() {
            self.id = rhs.id;
        }
        self.flow = self.flow.max(rhs.flow);
        self.invalidate = self.invalidate.max(rhs.invalidate);

        self.action = None;
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
        let r = Response::action(3u8)
            .relayout()
            .for_id(Id::root("m"))
            .with_state(StateFlags::FOCUSED | StateFlags::DIRTY);
        let m = r.map_action(|n| u32::from(n).saturating_mul(2));
        assert_eq!(m.action_ref(), Some(&6));
        assert_eq!(m.invalidate(), Invalidate::Layout);
        assert!(m.is_consumed());
        assert_eq!(m.id(), Some(Id::root("m")));

        assert_eq!(m.state(), StateFlags::FOCUSED | StateFlags::DIRTY);

        let mut taken = m;
        assert_eq!(taken.take_action(), Some(6));
        assert_eq!(taken.state(), StateFlags::FOCUSED | StateFlags::DIRTY);
        assert_eq!(taken.id(), Some(Id::root("m")));
        assert_eq!(taken.invalidate(), Invalidate::Layout);
        assert!(taken.is_consumed());
    }

    #[doc = r" Acceptance: `id()` is `None` until [`Response::for_id`] tags it, for"]
    #[doc = r#" every constructor — it is not a proxy for "this response was ignored"."#]
    #[test]
    fn id_is_none_until_for_id_tags_it() {
        assert!(Response::<u8>::ignored().id().is_none());
        assert!(Response::<u8>::consumed().id().is_none());
        assert!(Response::<u8>::changed().id().is_none());
        assert!(Response::action(1u8).id().is_none());
        let tagged = Response::<u8>::ignored().for_id(Id::root("t"));
        assert_eq!(tagged.id(), Some(Id::root("t")));
        assert!(!tagged.is_consumed());
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

    #[doc = r" Acceptance: `a |= b` must equal `a | b` on every field of `Response`."]
    #[doc = r""]
    #[doc = r" The destructuring pattern below is exhaustive on purpose: adding a"]
    #[doc = r" field to `Response` breaks this test at compile time, forcing both"]
    #[doc = r" fold implementations to be revisited together."]
    #[test]
    fn bitor_assign_agrees_with_bitor_on_every_field() {
        let ids = [None, Some(Id::root("l")), Some(Id::root("r"))];
        let flows = [Flow::Ignored, Flow::Consumed];
        let invalidates = [Invalidate::None, Invalidate::Paint, Invalidate::Layout];
        let states = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::HOVERED | StateFlags::PRESSED,
        ];
        let actions = [None, Some(())];

        let build = |i: Option<Id>, f: Flow, v: Invalidate, s: StateFlags, a: Option<()>| {
            let mut r: Response<()> = Response::ignored();
            let Response {
                id,
                flow,
                invalidate,
                state,
                action,
            } = &mut r;
            *id = i;
            *flow = f;
            *invalidate = v;
            *state = s;
            *action = a;
            r
        };

        let mut cases = Vec::new();
        for i in ids {
            for f in flows {
                for v in invalidates {
                    for s in states {
                        for a in actions {
                            cases.push(build(i, f, v, s, a));
                        }
                    }
                }
            }
        }

        for lhs in &cases {
            for rhs in &cases {
                let folded = lhs.clone() | rhs.clone();
                let mut assigned = lhs.clone();
                assigned |= rhs.clone();

                let Response {
                    id,
                    flow,
                    invalidate,
                    state,
                    action,
                } = &folded;
                let Response {
                    id: a_id,
                    flow: a_flow,
                    invalidate: a_invalidate,
                    state: a_state,
                    action: a_action,
                } = &assigned;

                assert_eq!(id, a_id, "id diverged for {lhs:?} | {rhs:?}");
                assert_eq!(flow, a_flow, "flow diverged for {lhs:?} | {rhs:?}");
                assert_eq!(
                    invalidate, a_invalidate,
                    "invalidate diverged for {lhs:?} | {rhs:?}"
                );
                assert_eq!(state, a_state, "state diverged for {lhs:?} | {rhs:?}");
                assert_eq!(action, a_action, "action diverged for {lhs:?} | {rhs:?}");

                assert_eq!(*id, lhs.id.or(rhs.id));
                assert_eq!(*flow, lhs.flow.max(rhs.flow));
                assert_eq!(*invalidate, lhs.invalidate.max(rhs.invalidate));
                assert_eq!(*state, lhs.state);
                assert_eq!(*action, None);
            }
        }
    }

    #[test]
    fn bitor_assign_clears_the_action_like_bitor() {
        let mut r = Response::action(());
        assert!(r.action_ref().is_some());
        r |= Response::ignored();
        assert!(r.action_ref().is_none());
        assert!(r.is_consumed() && r.is_changed());

        let folded = Response::action(()) | Response::<()>::ignored();
        assert!(folded.action_ref().is_none());
    }

    #[test]
    fn bitor_assign_takes_the_right_id_only_when_the_left_has_none() {
        let mut r: Response<()> = Response::ignored();
        r |= Response::consumed().for_id(Id::root("r"));
        assert_eq!(r.id(), Some(Id::root("r")));

        let mut r: Response<()> = Response::ignored().for_id(Id::root("l"));
        r |= Response::consumed().for_id(Id::root("r"));
        assert_eq!(r.id(), Some(Id::root("l")));
    }

    #[test]
    fn bitor_assign_keeps_the_left_state() {
        let mut r: Response<()> = Response::consumed().with_state(StateFlags::FOCUSED);
        r |= Response::changed().with_state(StateFlags::HOVERED);
        assert_eq!(r.state(), StateFlags::FOCUSED);
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
