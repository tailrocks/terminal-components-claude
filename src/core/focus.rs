//! Keyboard focus ring.
//!
//! The ring is rebuilt every frame in render order, which makes Tab order
//! identical to reading order and keeps it deterministic. A modal pushes a
//! barrier so only its own controls are reachable.

use super::id::WidgetId;

#[derive(Debug, Default, Clone)]
pub struct FocusRing {
    order: Vec<WidgetId>,
    barrier: Option<usize>,
}

impl FocusRing {
    pub fn register(&mut self, id: WidgetId) {
        self.order.push(id);
    }

    pub fn push_barrier(&mut self) {
        self.barrier = Some(self.order.len());
    }

    pub fn reachable(&self) -> &[WidgetId] {
        match self.barrier {
            Some(start) => &self.order[start..],
            None => &self.order,
        }
    }

    pub fn contains(&self, id: WidgetId) -> bool {
        self.reachable().contains(&id)
    }

    pub fn first(&self) -> Option<WidgetId> {
        self.reachable().first().copied()
    }

    pub fn next(&self, current: Option<WidgetId>) -> Option<WidgetId> {
        let ring = self.reachable();
        if ring.is_empty() {
            return None;
        }
        match current.and_then(|c| ring.iter().position(|&id| id == c)) {
            Some(i) => Some(ring[(i + 1) % ring.len()]),
            None => ring.first().copied(),
        }
    }

    pub fn prev(&self, current: Option<WidgetId>) -> Option<WidgetId> {
        let ring = self.reachable();
        if ring.is_empty() {
            return None;
        }
        match current.and_then(|c| ring.iter().position(|&id| id == c)) {
            Some(0) | None => ring.last().copied(),
            Some(i) => Some(ring[i - 1]),
        }
    }
}

/// Where focus goes when the focused control disappears (a dialog
/// deleted the focused row, a button vanished with its panel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusFallback {
    /// The control just before where the lost one sat in the ring (or the
    /// first, when it was first): focus stays in the same neighbourhood.
    #[default]
    Nearest,
    /// The first control of the ring.
    First,
}

/// What a control does with an edit in progress when it loses focus
/// (focus moved elsewhere, or a click landed on dead space).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlurAction {
    /// Apply the edit and report it like Enter would (macOS: leaving a
    /// field ends editing and keeps the text).
    #[default]
    Commit,
    /// Revert to the value before editing, like Esc.
    Cancel,
    /// Leave the edit open; it continues when focus comes back.
    Keep,
}

/// One control lost focus (or was asked to end its edit in place).
///
/// Produced by [`Focus::take_changes`]. Applications forward every change
/// to the owner of `from`, which calls the control's `on_blur`, so the
/// control commits or cancels from the event path (never from render) and
/// the owner sees the resulting `Committed`/`Cancelled` event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusChange {
    /// The control that lost focus.
    pub from: WidgetId,
    /// Where focus is now. Equal to `Some(from)` when the control keeps
    /// focus but its edit ends ([`Focus::end_edit`]: a click on dead space).
    pub to: Option<WidgetId>,
}

/// A control that ends an edit in progress (or closes a popup) when focus
/// leaves it. Implemented by the editing widgets (`TextInput`, `TextArea`,
/// `Select`, `DataGrid`, `DataTable`) so an owner that has no use for the
/// resulting event forwards a change in one call ([`forward_blur`]); an
/// owner that runs save or validation hooks calls the widget's own
/// `on_blur` and reads its event instead.
pub trait BlurTarget {
    /// The focus id whose loss blurs this control.
    fn blur_id(&self) -> WidgetId;
    /// End the edit per the control's blur policy; `Changed` when
    /// anything changed.
    fn blur(&mut self) -> super::event::Outcome;
}

/// Hand `change` to whichever of `targets` lost focus.
pub fn forward_blur(
    change: FocusChange,
    targets: &mut [&mut dyn BlurTarget],
) -> super::event::Outcome {
    targets
        .iter_mut()
        .find(|t| t.blur_id() == change.from)
        .map_or(super::event::Outcome::Ignored, |t| t.blur())
}

/// Focus state shared by the whole application.
#[derive(Debug, Default, Clone)]
pub struct Focus {
    current: Option<WidgetId>,
    /// Ring position of `current` the last time it was validated.
    last_index: Option<usize>,
    /// Controls that lost focus since the last [`Self::take_changes`].
    left: Vec<WidgetId>,
    /// The current control was asked to end its edit in place.
    end_edit: Option<WidgetId>,
}

impl Focus {
    pub fn current(&self) -> Option<WidgetId> {
        self.current
    }

    pub fn is(&self, id: WidgetId) -> bool {
        self.current == Some(id)
    }

    pub fn set(&mut self, id: Option<WidgetId>) {
        self.move_to(id);
    }

    pub fn focus(&mut self, id: WidgetId) {
        self.move_to(Some(id));
    }

    pub fn next(&mut self, ring: &FocusRing) {
        self.move_to(ring.next(self.current));
    }

    pub fn prev(&mut self, ring: &FocusRing) {
        self.move_to(ring.prev(self.current));
    }

    /// Every focus move goes through here so the control that lost focus
    /// is remembered for [`Self::take_changes`].
    fn move_to(&mut self, id: Option<WidgetId>) {
        if let Some(from) = self.current
            && self.current != id
            && !self.left.contains(&from)
        {
            self.left.push(from);
        }
        self.current = id;
    }

    /// End the focused control's edit without moving focus: a click on
    /// dead space (no control under the pointer) ends editing the way
    /// macOS does when the window background is clicked. The next
    /// [`Self::take_changes`] reports it with `to == Some(from)`.
    pub fn end_edit(&mut self) {
        self.end_edit = self.current;
    }

    /// Drain the controls that lost focus (or were asked to end their
    /// edit) since the last call. A control that lost focus and got it
    /// back before the drain is not reported: it never really blurred.
    ///
    /// Applications call this after handling each input and forward every
    /// change to the owner of `from` (its `on_blur`), so blur behaviour
    /// runs on the event path, never inside `render`.
    pub fn take_changes(&mut self) -> Vec<FocusChange> {
        let to = self.current;
        let mut out: Vec<FocusChange> = std::mem::take(&mut self.left)
            .into_iter()
            .filter(|&from| Some(from) != to)
            .map(|from| FocusChange { from, to })
            .collect();
        if let Some(from) = self.end_edit.take()
            && Some(from) == to
        {
            out.push(FocusChange { from, to });
        }
        out
    }

    /// Make sure focus points at something reachable in this ring, falling
    /// back to the ring's first control.
    pub fn ensure_valid(&mut self, ring: &FocusRing) {
        self.ensure_valid_with(ring, FocusFallback::First);
    }

    /// Make sure focus points at something reachable in this ring, falling
    /// back per `fallback` when the focused control is gone.
    pub fn ensure_valid_with(&mut self, ring: &FocusRing, fallback: FocusFallback) {
        let reach = ring.reachable();
        if let Some(i) = self
            .current
            .and_then(|c| reach.iter().position(|&id| id == c))
        {
            self.last_index = Some(i);
            return;
        }
        let next = match (fallback, self.last_index) {
            (FocusFallback::Nearest, Some(i)) if !reach.is_empty() => {
                Some(reach[i.saturating_sub(1).min(reach.len() - 1)])
            }
            _ => ring.first(),
        };
        self.move_to(next);
        self.last_index = self
            .current
            .and_then(|c| reach.iter().position(|&id| id == c));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring(names: &[&str]) -> FocusRing {
        let mut r = FocusRing::default();
        for n in names {
            r.register(WidgetId::of(n));
        }
        r
    }

    #[test]
    fn tab_cycles_forward_and_backward() {
        let r = ring(&["a", "b", "c"]);
        let (a, b, c) = (WidgetId::of("a"), WidgetId::of("b"), WidgetId::of("c"));
        let mut f = Focus::default();
        f.next(&r);
        assert_eq!(f.current(), Some(a));
        f.next(&r);
        assert_eq!(f.current(), Some(b));
        f.next(&r);
        f.next(&r);
        assert_eq!(f.current(), Some(a));
        f.prev(&r);
        assert_eq!(f.current(), Some(c));
    }

    #[test]
    fn barrier_traps_focus_and_restores() {
        let mut r = ring(&["a", "b"]);
        r.push_barrier();
        r.register(WidgetId::of("ok"));
        r.register(WidgetId::of("cancel"));
        let mut f = Focus::default();
        f.focus(WidgetId::of("ok"));
        f.next(&r);
        assert_eq!(f.current(), Some(WidgetId::of("cancel")));
        f.next(&r);
        assert_eq!(f.current(), Some(WidgetId::of("ok")));
        assert!(!r.contains(WidgetId::of("a")));
    }

    #[test]
    fn a_vanished_focus_falls_back_to_its_nearest_neighbour() {
        let mut f = Focus::default();
        f.focus(WidgetId::of("c"));
        f.ensure_valid_with(&ring(&["a", "b", "c", "d"]), FocusFallback::Nearest);
        // "c" was deleted: focus lands on the control before it, not on "a"
        f.ensure_valid_with(&ring(&["a", "b", "d"]), FocusFallback::Nearest);
        assert_eq!(f.current(), Some(WidgetId::of("b")));
        let mut first = Focus::default();
        first.focus(WidgetId::of("c"));
        first.ensure_valid_with(&ring(&["a", "b", "c"]), FocusFallback::First);
        first.ensure_valid_with(&ring(&["a", "b"]), FocusFallback::First);
        assert_eq!(first.current(), Some(WidgetId::of("a")));
    }

    #[test]
    fn focus_moves_report_the_control_that_lost_focus() {
        let r = ring(&["a", "b", "c"]);
        let (a, b, c) = (WidgetId::of("a"), WidgetId::of("b"), WidgetId::of("c"));
        let mut f = Focus::default();
        f.focus(a);
        assert!(f.take_changes().is_empty(), "nothing had focus before");
        f.next(&r);
        assert_eq!(
            f.take_changes(),
            vec![FocusChange {
                from: a,
                to: Some(b)
            }]
        );
        assert!(f.take_changes().is_empty(), "drained");
        // leaving and coming back inside one input is not a blur for b;
        // c held focus in between and lost it
        f.focus(c);
        f.focus(b);
        assert_eq!(
            f.take_changes(),
            vec![FocusChange {
                from: c,
                to: Some(b)
            }]
        );
        // b -> c -> a: both b and c lost focus
        f.focus(c);
        f.focus(a);
        assert_eq!(
            f.take_changes(),
            vec![
                FocusChange {
                    from: b,
                    to: Some(a)
                },
                FocusChange {
                    from: c,
                    to: Some(a)
                }
            ]
        );
        f.set(None);
        assert_eq!(f.take_changes(), vec![FocusChange { from: a, to: None }]);
    }

    #[test]
    fn end_edit_reports_the_focused_control_in_place() {
        let a = WidgetId::of("a");
        let mut f = Focus::default();
        f.end_edit();
        assert!(f.take_changes().is_empty(), "no focus, nothing to end");
        f.focus(a);
        f.end_edit();
        assert_eq!(
            f.take_changes(),
            vec![FocusChange {
                from: a,
                to: Some(a)
            }]
        );
        assert_eq!(f.current(), Some(a), "focus stays");
    }

    #[test]
    fn ensure_valid_falls_back_to_first() {
        let r = ring(&["a", "b"]);
        let mut f = Focus::default();
        f.focus(WidgetId::of("zzz"));
        f.ensure_valid(&r);
        assert_eq!(f.current(), Some(WidgetId::of("a")));
    }
}
