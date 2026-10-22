use core::ops::{BitOr, BitOrAssign};

use bitflags::bitflags;

use crate::id::Id;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord)]
pub enum Flow {
    #[default]
    Ignored,
    Consumed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord)]
pub enum Invalidate {
    #[default]
    None,
    Paint,
    Layout,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
    pub struct StateFlags: u32 {
        const FOCUSED       = 1 << 0;
        const FOCUS_VISIBLE = 1 << 1;
        const HOVERED       = 1 << 2;
        const PRESSED       = 1 << 3;
        const SELECTED      = 1 << 4;
        const ACTIVE        = 1 << 5;
        const CHECKED       = 1 << 6;
        const DISABLED      = 1 << 7;
        const READ_ONLY     = 1 << 8;
        const ERROR         = 1 << 9;
        const WARNING       = 1 << 10;
        const BUSY          = 1 << 11;
        const EDITING       = 1 << 12;
        const DIRTY         = 1 << 13;
        const EXPANDED      = 1 << 14;
        const LOADING       = 1 << 15;
        /// The owning layer is not the active layer.
        ///
        /// The flag is the state-vocabulary form of the runtime's inert
        /// context: a component drawn under a modal, a popup, or a reference
        /// frame reports it so a consumer can tell "not interactive right now"
        /// apart from "disabled by configuration".
        const INACTIVE_LAYER = 1 << 16;
        /// The current value does not satisfy its own contract.
        ///
        /// `ERROR` carries the severity of a failed operation; `INVALID`
        /// carries the fact that the value in hand cannot be used. A field
        /// that failed validation is both; a grid cell holding a malformed
        /// value is invalid without being an error.
        const INVALID       = 1 << 17;
    }
}

impl StateFlags {
    /// Whether the owning layer is not the active layer.
    pub const fn is_inactive_layer(self) -> bool {
        self.contains(Self::INACTIVE_LAYER)
    }

    /// Whether the value in hand cannot be used.
    pub const fn is_invalid(self) -> bool {
        self.contains(Self::INVALID)
    }
}

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
    pub const fn ignored() -> Self {
        Response {
            id: None,
            flow: Flow::Ignored,
            invalidate: Invalidate::None,
            state: StateFlags::empty(),
            action: None,
        }
    }

    pub const fn consumed() -> Self {
        Response {
            id: None,
            flow: Flow::Consumed,
            invalidate: Invalidate::None,
            state: StateFlags::empty(),
            action: None,
        }
    }

    pub const fn changed() -> Self {
        Response {
            id: None,
            flow: Flow::Consumed,
            invalidate: Invalidate::Paint,
            state: StateFlags::empty(),
            action: None,
        }
    }

    pub const fn action(a: A) -> Self {
        Response {
            id: None,
            flow: Flow::Consumed,
            invalidate: Invalidate::Paint,
            state: StateFlags::empty(),
            action: Some(a),
        }
    }

    pub const fn for_id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub const fn with_state(mut self, s: StateFlags) -> Self {
        self.state = s;
        self
    }

    pub fn repaint(mut self) -> Self {
        self.invalidate = self.invalidate.max(Invalidate::Paint);
        self
    }

    pub const fn relayout(mut self) -> Self {
        self.invalidate = Invalidate::Layout;
        self
    }

    pub const fn no_repaint(mut self) -> Self {
        self.invalidate = Invalidate::None;
        self
    }

    pub const fn id(&self) -> Option<Id> {
        self.id
    }

    pub const fn flow(&self) -> Flow {
        self.flow
    }

    pub fn is_consumed(&self) -> bool {
        self.flow == Flow::Consumed
    }

    pub const fn invalidate(&self) -> Invalidate {
        self.invalidate
    }

    pub fn is_changed(&self) -> bool {
        self.invalidate >= Invalidate::Paint
    }

    pub const fn state(&self) -> StateFlags {
        self.state
    }

    pub const fn focused(&self) -> bool {
        self.state.contains(StateFlags::FOCUSED)
    }

    pub const fn hovered(&self) -> bool {
        self.state.contains(StateFlags::HOVERED)
    }

    pub const fn pressed(&self) -> bool {
        self.state.contains(StateFlags::PRESSED)
    }

    pub const fn action_ref(&self) -> Option<&A> {
        self.action.as_ref()
    }

    pub const fn take_action(&mut self) -> Option<A> {
        self.action.take()
    }

    pub fn into_action(self) -> Option<A> {
        self.action
    }

    pub fn map_action<B>(self, f: impl FnOnce(A) -> B) -> Response<B> {
        Response {
            id: self.id,
            flow: self.flow,
            invalidate: self.invalidate,
            state: self.state,
            action: self.action.map(f),
        }
    }

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

    pub fn erase(self) -> Response<()> {
        self.on_action(|_| {})
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Activated;

impl Response<Activated> {
    pub const fn activated(&self) -> bool {
        self.action.is_some()
    }

    pub fn on_activated(self, f: impl FnOnce()) -> Response<()> {
        self.on_action(|Activated| f())
    }
}

impl BitOr for Response<()> {
    type Output = Response<()>;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}

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
        assert_eq!(StateFlags::all().bits(), 0x0003_ffff);
        assert_eq!(format!("{s:?}"), "StateFlags(FOCUSED | HOVERED | EDITING)");
        let r: Response<()> = Response::consumed().with_state(s);
        assert!(r.focused() && r.hovered() && !r.pressed());
    }

    #[test]
    fn inactive_layer_and_invalid_are_distinct_first_class_states() {
        assert!(StateFlags::INACTIVE_LAYER.contains(StateFlags::INACTIVE_LAYER));
        assert!(StateFlags::INVALID.contains(StateFlags::INVALID));
        assert!(!StateFlags::INACTIVE_LAYER.contains(StateFlags::INVALID));
        assert!(!StateFlags::INVALID.contains(StateFlags::INACTIVE_LAYER));
        assert!(!StateFlags::ERROR.contains(StateFlags::INVALID));
        assert!(!StateFlags::INVALID.contains(StateFlags::ERROR));
        assert!(!StateFlags::FOCUSED.contains(StateFlags::INACTIVE_LAYER));
    }
}
