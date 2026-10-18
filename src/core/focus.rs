use super::id::WidgetId;

#[derive(Debug, Default, Clone)]
pub struct FocusRing {
    order: Vec<WidgetId>,
    barrier: Option<usize>,

    selections: Vec<WidgetId>,
}

impl FocusRing {
    pub fn register(&mut self, id: WidgetId) {
        self.order.push(id);
    }

    pub fn mark_selection(&mut self, id: WidgetId) {
        if !self.selections.contains(&id) {
            self.selections.push(id);
        }
    }

    pub fn has_selection(&self, id: WidgetId) -> bool {
        self.selections.contains(&id)
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn sort_from<K: Ord>(&mut self, start: usize, key: impl FnMut(&WidgetId) -> K) {
        if let Some(group) = self.order.get_mut(start..) {
            group.sort_by_key(key);
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusFallback {
    #[default]
    Nearest,

    First,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlurAction {
    #[default]
    Commit,

    Cancel,

    Keep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusChange {
    pub from: WidgetId,

    pub to: Option<WidgetId>,
}

pub trait BlurTarget {
    fn blur_id(&self) -> WidgetId;

    fn blur(&mut self) -> super::event::Outcome;
}

pub fn forward_blur(
    change: FocusChange,
    targets: &mut [&mut dyn BlurTarget],
) -> super::event::Outcome {
    targets
        .iter_mut()
        .find(|t| t.blur_id() == change.from)
        .map_or(super::event::Outcome::Ignored, |t| t.blur())
}

thread_local! {


    static REQUEST: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

pub fn request_focus_next(backward: bool) {
    REQUEST.with(|r| r.set(Some(backward)));
}

pub fn cancel_focus_request() {
    REQUEST.with(|r| r.set(None));
}

#[derive(Debug, Default, Clone)]
pub struct Focus {
    current: Option<WidgetId>,

    last_index: Option<usize>,

    last_valid: Option<WidgetId>,

    left: Vec<WidgetId>,

    end_edit: Option<WidgetId>,

    moves: u64,
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

    pub fn moves(&self) -> u64 {
        self.moves
    }

    fn move_to(&mut self, id: Option<WidgetId>) {
        self.moves = self.moves.wrapping_add(1);
        if self.current != id {
            cancel_focus_request();
        }
        if let Some(from) = self.current
            && self.current != id
            && !self.left.contains(&from)
        {
            self.left.push(from);
        }
        self.current = id;
    }

    pub fn apply_request(&mut self, ring: &FocusRing) -> bool {
        let Some(backward) = REQUEST.with(|r| r.take()) else {
            return false;
        };
        let before = self.current;
        if backward {
            self.prev(ring);
        } else {
            self.next(ring);
        }
        self.current != before
    }

    pub fn settle(&mut self, ring: &FocusRing) -> Vec<FocusChange> {
        self.apply_request(ring);
        self.take_changes()
    }

    pub fn end_edit(&mut self) {
        self.end_edit = self.current;
    }

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

    pub fn ensure_valid(&mut self, ring: &FocusRing) {
        self.ensure_valid_with(ring, FocusFallback::First);
    }

    pub fn ensure_valid_with(&mut self, ring: &FocusRing, fallback: FocusFallback) {
        let reach = ring.reachable();
        if let Some(i) = self
            .current
            .and_then(|c| reach.iter().position(|&id| id == c))
        {
            self.last_index = Some(i);
            self.last_valid = self.current;
            return;
        }
        let stayed = self.last_valid.filter(|id| reach.contains(id));
        let next = match (fallback, self.last_index) {
            (FocusFallback::Nearest, _) if stayed.is_some() => stayed,
            (FocusFallback::Nearest, Some(i)) if !reach.is_empty() => {
                Some(reach[i.saturating_sub(1).min(reach.len() - 1)])
            }
            _ => ring.first(),
        };
        self.move_to(next);
        self.last_index = self
            .current
            .and_then(|c| reach.iter().position(|&id| id == c));
        self.last_valid = self.last_index.and(self.current);
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
    fn sort_from_reorders_only_the_group() {
        let (a, b, c, d) = (
            WidgetId::of("a"),
            WidgetId::of("b"),
            WidgetId::of("c"),
            WidgetId::of("d"),
        );
        let mut r = FocusRing::default();
        r.register(d);
        let start = r.len();
        r.register(c);
        r.register(a);
        r.register(b);
        let rank = |id: &WidgetId| [a, b, c].iter().position(|x| x == id);
        r.sort_from(start, rank);
        assert_eq!(r.reachable(), &[d, a, b, c]);
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

        f.ensure_valid_with(&ring(&["a", "b", "d"]), FocusFallback::Nearest);
        assert_eq!(f.current(), Some(WidgetId::of("b")));
        let mut first = Focus::default();
        first.focus(WidgetId::of("c"));
        first.ensure_valid_with(&ring(&["a", "b", "c"]), FocusFallback::First);
        first.ensure_valid_with(&ring(&["a", "b"]), FocusFallback::First);
        assert_eq!(first.current(), Some(WidgetId::of("a")));
    }

    #[test]
    fn focus_moved_onto_an_unfocusable_control_stays_where_it_was() {
        let r = ring(&["a", "b", "c", "d"]);
        let mut f = Focus::default();
        f.focus(WidgetId::of("c"));
        f.ensure_valid_with(&r, FocusFallback::Nearest);
        f.focus(WidgetId::of("disabled"));
        f.ensure_valid_with(&r, FocusFallback::Nearest);
        assert_eq!(f.current(), Some(WidgetId::of("c")));

        let mut first = Focus::default();
        first.focus(WidgetId::of("c"));
        first.ensure_valid_with(&r, FocusFallback::First);
        first.focus(WidgetId::of("disabled"));
        first.ensure_valid_with(&r, FocusFallback::First);
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

        f.focus(c);
        f.focus(b);
        assert_eq!(
            f.take_changes(),
            vec![FocusChange {
                from: c,
                to: Some(b)
            }]
        );

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
    fn a_request_moves_focus_at_settle_unless_the_owner_moved_it() {
        let r = ring(&["a", "b", "c"]);
        let mut f = Focus::default();
        f.focus(WidgetId::of("a"));
        f.take_changes();
        request_focus_next(false);
        let changes = f.settle(&r);
        assert!(f.is(WidgetId::of("b")));
        assert_eq!(changes.len(), 1);

        request_focus_next(false);
        f.prev(&r);
        f.settle(&r);
        assert!(f.is(WidgetId::of("a")), "one move, not two");
        request_focus_next(true);
        f.settle(&r);
        assert!(f.is(WidgetId::of("c")), "backward wraps");
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
