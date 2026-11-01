use std::ops::{BitAnd, BitOr, BitOrAssign, Not, Sub};

use super::event::Outcome;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord, Hash)]
pub enum Flow {
    #[default]
    Ignored,

    Consumed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord, Hash)]
pub enum Invalidate {
    #[default]
    None,

    Paint,

    Layout,
}

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
    pub const fn ignored() -> Self {
        Self {
            flow: Flow::Ignored,
            invalidate: Invalidate::None,
            action: None,
        }
    }

    pub const fn consumed() -> Self {
        Self {
            flow: Flow::Consumed,
            invalidate: Invalidate::None,
            action: None,
        }
    }

    pub const fn changed() -> Self {
        Self {
            flow: Flow::Consumed,
            invalidate: Invalidate::Paint,
            action: None,
        }
    }

    pub const fn emit(action: A) -> Self {
        Self {
            flow: Flow::Consumed,
            invalidate: Invalidate::Paint,
            action: Some(action),
        }
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

    pub fn outcome(&self) -> Outcome {
        match (self.flow, self.is_changed()) {
            (_, true) => Outcome::Changed,
            (Flow::Consumed, false) => Outcome::Consumed,
            (Flow::Ignored, false) => Outcome::Ignored,
        }
    }

    pub const fn action(&self) -> Option<&A> {
        self.action.as_ref()
    }

    pub fn take_action(&mut self) -> Option<A> {
        self.action.take()
    }

    pub fn into_action(self) -> Option<A> {
        self.action
    }

    pub fn into_parts(self) -> (Outcome, Option<A>) {
        (self.outcome(), self.action)
    }

    pub fn with_action<B>(self, action: B) -> Response<B> {
        Response {
            flow: Flow::Consumed,
            invalidate: self.invalidate.max(Invalidate::Paint),
            action: Some(action),
        }
    }

    pub fn map_action<B>(self, f: impl FnOnce(A) -> B) -> Response<B> {
        Response {
            flow: self.flow,
            invalidate: self.invalidate,
            action: self.action.map(f),
        }
    }

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

    pub fn erase(self) -> Response<()> {
        Response {
            flow: self.flow,
            invalidate: self.invalidate,
            action: None,
        }
    }
}

impl Response<()> {
    pub fn lift<B>(self) -> Response<B> {
        Response {
            flow: self.flow,
            invalidate: self.invalidate,
            action: None,
        }
    }
}

impl<A> From<Outcome> for Response<A> {
    fn from(o: Outcome) -> Self {
        match o {
            Outcome::Ignored => Self::ignored(),
            Outcome::Consumed => Self::consumed(),
            Outcome::Changed => Self::changed(),
        }
    }
}

impl<A> From<(Outcome, Option<A>)> for Response<A> {
    fn from((o, a): (Outcome, Option<A>)) -> Self {
        let r = Self::from(o);
        match a {
            Some(a) => r.with_action(a),
            None => r,
        }
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
        self.flow = self.flow.max(rhs.flow);
        self.invalidate = self.invalidate.max(rhs.invalidate);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct StateFlags(u16);

impl StateFlags {
    pub const FOCUSED: Self = Self(1 << 0);

    pub const FOCUS_VISIBLE: Self = Self(1 << 1);

    pub const HOVERED: Self = Self(1 << 2);

    pub const PRESSED: Self = Self(1 << 3);

    pub const SELECTED: Self = Self(1 << 4);

    pub const ACTIVE: Self = Self(1 << 5);

    pub const CHECKED: Self = Self(1 << 6);

    pub const DISABLED: Self = Self(1 << 7);

    pub const READ_ONLY: Self = Self(1 << 8);

    pub const ERROR: Self = Self(1 << 9);

    pub const WARNING: Self = Self(1 << 10);

    pub const BUSY: Self = Self(1 << 11);

    pub const EDITING: Self = Self(1 << 12);

    pub const DIRTY: Self = Self(1 << 13);

    pub const EXPANDED: Self = Self(1 << 14);

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

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    pub fn set(&mut self, other: Self, on: bool) {
        if on {
            self.insert(other);
        } else {
            self.remove(other);
        }
    }

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
