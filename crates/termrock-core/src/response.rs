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






        const INACTIVE_LAYER = 1 << 16;






        const INVALID       = 1 << 17;
    }
}

impl StateFlags {
    pub const fn is_inactive_layer(self) -> bool {
        self.contains(Self::INACTIVE_LAYER)
    }

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
