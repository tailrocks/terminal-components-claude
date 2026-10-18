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

/// Focus state shared by the whole application.
#[derive(Debug, Default, Clone)]
pub struct Focus {
    current: Option<WidgetId>,
}

impl Focus {
    pub fn current(&self) -> Option<WidgetId> {
        self.current
    }

    pub fn is(&self, id: WidgetId) -> bool {
        self.current == Some(id)
    }

    pub fn set(&mut self, id: Option<WidgetId>) {
        self.current = id;
    }

    pub fn focus(&mut self, id: WidgetId) {
        self.current = Some(id);
    }

    pub fn next(&mut self, ring: &FocusRing) {
        self.current = ring.next(self.current);
    }

    pub fn prev(&mut self, ring: &FocusRing) {
        self.current = ring.prev(self.current);
    }

    /// Make sure focus points at something reachable in this ring.
    pub fn ensure_valid(&mut self, ring: &FocusRing) {
        if !self.current.is_some_and(|c| ring.contains(c)) {
            self.current = ring.first();
        }
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
    fn ensure_valid_falls_back_to_first() {
        let r = ring(&["a", "b"]);
        let mut f = Focus::default();
        f.focus(WidgetId::of("zzz"));
        f.ensure_valid(&r);
        assert_eq!(f.current(), Some(WidgetId::of("a")));
    }
}
