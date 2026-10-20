use ratatui::layout::{Position, Rect};

use super::id::{ItemKey, Part, PartRef, WidgetId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitRegion {
    pub id: WidgetId,

    pub owner: WidgetId,

    pub part: PartRef,
    pub area: Rect,

    pub scroll_only: bool,

    pub pass_through: bool,

    pub hover_only: bool,

    pub backdrop: bool,

    pub surface: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bar {
    pub id: WidgetId,
    pub owner: WidgetId,
    pub axis: Axis,
    pub track: Rect,
}

#[derive(Debug, Default, Clone)]
pub struct HitRegistry {
    regions: Vec<HitRegion>,

    barriers: Vec<usize>,

    bars: Vec<Bar>,

    shapes: Vec<(WidgetId, super::shape::PointerShape)>,

    autoscroll: Vec<(usize, Rect)>,

    releases: Vec<(WidgetId, ReleaseFrom)>,

    shifts: Vec<(WidgetId, (u16, u16))>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HitMark {
    regions: usize,
    bars: usize,
    autoscroll: usize,
}

impl HitMark {
    pub(crate) fn regions_len(&self) -> usize {
        self.regions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseFrom {
    Press(WidgetId),

    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    pub owner: WidgetId,
    pub part: PartRef,
}

impl HitRegion {
    fn plain(id: WidgetId, area: Rect) -> Self {
        Self {
            id,
            owner: id,
            part: PartRef::of(Part::CONTAINER),
            area,
            scroll_only: false,
            pass_through: false,
            hover_only: false,
            backdrop: false,
            surface: false,
        }
    }

    pub fn hit(&self) -> Hit {
        Hit {
            owner: self.owner,
            part: self.part,
        }
    }

    fn clickable(&self) -> bool {
        !self.scroll_only && !self.hover_only
    }
}

impl HitRegistry {
    pub fn register(&mut self, id: WidgetId, area: Rect) {
        if area.is_empty() {
            return;
        }
        self.regions.push(HitRegion::plain(id, area));
    }

    pub fn register_part(&mut self, owner: WidgetId, part: PartRef, area: Rect) -> WidgetId {
        let id = owner.part(part);
        self.register_part_as(id, owner, part, area);
        id
    }

    pub fn register_part_as(&mut self, id: WidgetId, owner: WidgetId, part: PartRef, area: Rect) {
        if area.is_empty() {
            return;
        }
        self.regions.push(HitRegion {
            owner,
            part,
            ..HitRegion::plain(id, area)
        });
    }

    pub fn resolve(&self, pos: Position) -> Option<Hit> {
        self.reachable()
            .iter()
            .rev()
            .find(|r| r.clickable() && r.area.contains(pos))
            .map(HitRegion::hit)
    }

    pub fn part_of(&self, id: WidgetId) -> Option<Hit> {
        self.regions
            .iter()
            .rev()
            .find(|r| r.id == id)
            .map(HitRegion::hit)
    }

    pub fn raise(&mut self, id: WidgetId) {
        if let Some(bar) = self.bar(id) {
            let parts: Vec<HitRegion> = self
                .regions
                .iter()
                .filter(|r| r.id == id && bar.track.contains(r.area.as_position()))
                .copied()
                .collect();
            let from = parts.iter().rposition(|r| r.area == bar.track).unwrap_or(0);
            self.regions.extend_from_slice(&parts[from..]);
            return;
        }
        if let Some(r) = self.regions.iter().rev().find(|r| r.id == id).copied() {
            self.regions.push(r);
        }
    }

    pub fn regions(&self) -> &[HitRegion] {
        &self.regions
    }

    pub fn register_under(&mut self, id: WidgetId, area: Rect) {
        if area.is_empty() {
            return;
        }
        let at = self.barriers.last().copied().unwrap_or(0);
        self.regions.insert(at, HitRegion::plain(id, area));
    }

    pub fn register_scroll_under(&mut self, id: WidgetId, area: Rect) {
        if area.is_empty() {
            return;
        }
        let at = self.barriers.last().copied().unwrap_or(0);
        self.regions.insert(
            at,
            HitRegion {
                scroll_only: true,
                ..HitRegion::plain(id, area)
            },
        );
    }

    pub fn register_surface(&mut self, id: WidgetId, area: Rect) {
        if area.is_empty() {
            return;
        }
        self.regions.push(HitRegion {
            surface: true,
            ..HitRegion::plain(id, area)
        });
    }

    pub fn is_surface(&self, id: WidgetId) -> bool {
        self.regions.iter().rev().any(|r| r.id == id && r.surface)
    }

    pub fn set_shape(&mut self, id: WidgetId, shape: super::shape::PointerShape) {
        self.shapes.retain(|(i, _)| *i != id);
        self.shapes.push((id, shape));
    }

    pub fn shape_of(&self, id: WidgetId) -> Option<super::shape::PointerShape> {
        self.shapes.iter().find(|(i, _)| *i == id).map(|(_, s)| *s)
    }

    pub fn register_bar(&mut self, bar: Bar) {
        if bar.track.is_empty() {
            return;
        }
        self.register_part_as(bar.id, bar.owner, PartRef::of(Part::TRACK), bar.track);
        self.bars.push(bar);
    }

    pub fn register_thumb(&mut self, bar_id: WidgetId, owner: WidgetId, thumb: Rect) {
        if self.bar(bar_id).is_none() {
            return;
        }
        self.register_part_as(bar_id, owner, PartRef::of(Part::THUMB), thumb);
    }

    pub fn accept_release(&mut self, id: WidgetId, from: ReleaseFrom) {
        self.releases.push((id, from));
    }

    pub fn accepts_release(&self, id: WidgetId, from: ReleaseFrom) -> bool {
        self.releases.contains(&(id, from))
    }

    pub fn bars(&self) -> &[Bar] {
        &self.bars
    }

    pub fn scrollers(&self) -> Vec<(WidgetId, Rect)> {
        self.regions
            .iter()
            .filter(|r| r.scroll_only)
            .map(|r| (r.id, r.area))
            .collect()
    }

    pub fn bar(&self, id: WidgetId) -> Option<Bar> {
        self.bars.iter().rev().find(|b| b.id == id).copied()
    }

    pub fn register_backdrop(&mut self, id: WidgetId, area: Rect, pass_through: bool) {
        self.register_backdrop_of(id, id, area, pass_through);
    }

    pub fn register_backdrop_of(
        &mut self,
        id: WidgetId,
        owner: WidgetId,
        area: Rect,
        pass_through: bool,
    ) {
        if area.is_empty() {
            return;
        }
        self.regions.push(HitRegion {
            pass_through,
            owner,
            part: PartRef::of(Part::BACKDROP),
            backdrop: true,
            ..HitRegion::plain(id, area)
        });
    }

    pub fn register_scroll(&mut self, id: WidgetId, area: Rect) {
        if area.is_empty() {
            return;
        }
        self.regions.push(HitRegion {
            scroll_only: true,
            ..HitRegion::plain(id, area)
        });
    }

    pub fn register_hover(&mut self, id: WidgetId, area: Rect) {
        if area.is_empty() {
            return;
        }
        self.regions.push(HitRegion {
            hover_only: true,
            ..HitRegion::plain(id, area)
        });
    }

    pub fn register_autoscroll(&mut self, area: Rect) {
        if !area.is_empty() {
            self.autoscroll.push((self.barriers.len(), area));
        }
    }

    pub fn autoscroll_at(&self, pos: Position) -> Option<Rect> {
        let layer = self.barriers.len();
        self.autoscroll
            .iter()
            .rev()
            .find(|(at, area)| *at == layer && area.contains(pos))
            .map(|(_, area)| *area)
    }

    pub fn push_barrier(&mut self) {
        self.barriers.push(self.regions.len());
    }

    fn reachable(&self) -> &[HitRegion] {
        match self.barriers.last() {
            Some(&start) => &self.regions[start..],
            None => &self.regions,
        }
    }

    fn beneath(&self) -> &[HitRegion] {
        match self.barriers.as_slice() {
            [] => &[],
            [top] => &self.regions[..*top],
            [.., below, top] => &self.regions[*below..*top],
        }
    }

    pub fn pass_through_target(&self, id: WidgetId, pos: Position) -> Option<WidgetId> {
        let top = self
            .reachable()
            .iter()
            .rev()
            .find(|r| r.clickable() && r.area.contains(pos))?;
        if top.id != id || !top.pass_through {
            return None;
        }
        self.beneath()
            .iter()
            .rev()
            .find(|r| r.clickable() && r.area.contains(pos))
            .map(|r| r.id)
    }

    pub fn backdrop_at(&self, pos: Position) -> Option<(WidgetId, Option<WidgetId>)> {
        let top = self
            .reachable()
            .iter()
            .rev()
            .find(|r| r.clickable() && r.area.contains(pos))?;
        if !top.backdrop {
            return None;
        }
        let mut end = *self.barriers.last()?;
        for &start in self.barriers.iter().rev().skip(1) {
            let layer = &self.regions[start..end];
            if !layer.is_empty() {
                return Some((top.id, Self::top_clickable(layer, pos)));
            }
            end = start;
        }
        Some((top.id, Self::top_clickable(&self.regions[..end], pos)))
    }

    fn top_clickable(layer: &[HitRegion], pos: Position) -> Option<WidgetId> {
        layer
            .iter()
            .rev()
            .find(|r| r.clickable() && !r.backdrop && r.area.contains(pos))
            .map(|r| r.id)
    }

    pub fn hit(&self, pos: Position) -> Option<WidgetId> {
        self.reachable()
            .iter()
            .rev()
            .find(|r| r.clickable() && r.area.contains(pos))
            .map(|r| r.id)
    }

    pub fn hover_at(&self, pos: Position) -> Option<WidgetId> {
        self.reachable()
            .iter()
            .rev()
            .find(|r| !r.scroll_only && r.area.contains(pos))
            .map(|r| r.id)
    }

    pub fn hit_scroll(&self, pos: Position) -> Option<WidgetId> {
        self.reachable()
            .iter()
            .rev()
            .find(|r| !r.hover_only && r.area.contains(pos))
            .map(|r| r.id)
    }

    pub fn scroll_stack(&self, pos: Position) -> Vec<WidgetId> {
        let mut out: Vec<WidgetId> = Vec::new();
        for r in self.reachable().iter().rev() {
            if !r.hover_only && r.area.contains(pos) && !out.contains(&r.id) {
                out.push(r.id);
            }
        }
        out
    }

    pub fn mark(&self) -> HitMark {
        HitMark {
            regions: self.regions.len(),
            bars: self.bars.len(),
            autoscroll: self.autoscroll.len(),
        }
    }

    pub fn scroll_since(&mut self, mark: HitMark, content: Rect, window: Rect, to: Position) {
        let dx = window.x.saturating_sub(to.x);
        let dy = window.y.saturating_sub(to.y);
        let place = |r: Rect| -> Rect {
            let r = r.intersection(window);
            if r.is_empty() {
                return Rect::ZERO;
            }
            Rect::new(r.x - dx, r.y - dy, r.width, r.height)
        };
        let from = mark.regions.min(self.regions.len());
        let mut moved: Vec<WidgetId> = Vec::new();
        let mut kept = Vec::with_capacity(self.regions.len() - from);
        for mut r in self.regions.drain(from..) {
            if r.area.intersects(content) {
                if !moved.contains(&r.id) {
                    moved.push(r.id);
                }
                r.area = place(r.area);
                if r.area.is_empty() {
                    continue;
                }
            }
            kept.push(r);
        }
        self.regions.extend(kept);
        let from = mark.bars.min(self.bars.len());
        let bars: Vec<Bar> = self.bars.drain(from..).collect();
        for mut b in bars {
            if b.track.intersects(content) {
                if !moved.contains(&b.id) {
                    moved.push(b.id);
                }
                b.track = place(b.track);
                if b.track.is_empty() {
                    continue;
                }
            }
            self.bars.push(b);
        }
        let from = mark.autoscroll.min(self.autoscroll.len());
        let areas: Vec<(usize, Rect)> = self.autoscroll.drain(from..).collect();
        for (layer, area) in areas {
            let area = if area.intersects(content) {
                place(area)
            } else {
                area
            };
            if !area.is_empty() {
                self.autoscroll.push((layer, area));
            }
        }
        if dx == 0 && dy == 0 {
            return;
        }
        for id in moved {
            match self.shifts.iter_mut().find(|(i, _)| *i == id) {
                Some((_, (x, y))) => {
                    *x = x.saturating_add(dx);
                    *y = y.saturating_add(dy);
                }
                None => self.shifts.push((id, (dx, dy))),
            }
        }
    }

    pub fn shift_of(&self, id: WidgetId) -> (u16, u16) {
        self.shifts
            .iter()
            .find(|(i, _)| *i == id)
            .map_or((0, 0), |&(_, s)| s)
    }

    pub fn local(&self, id: WidgetId, pos: Position) -> Position {
        let (dx, dy) = self.shift_of(id);
        Position::new(pos.x.saturating_add(dx), pos.y.saturating_add(dy))
    }

    pub fn len(&self) -> usize {
        self.regions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    pub fn clip_since(&mut self, from: usize, area: Rect) {
        let from = from.min(self.regions.len());
        let mut kept = Vec::with_capacity(self.regions.len() - from);
        for mut r in self.regions.drain(from..) {
            r.area = r.area.intersection(area);
            if !r.area.is_empty() {
                kept.push(r);
            }
        }
        self.regions.extend(kept);
    }

    pub fn area_of(&self, id: WidgetId) -> Option<Rect> {
        if let Some(bar) = self.bar(id) {
            return Some(bar.track);
        }
        self.regions
            .iter()
            .rev()
            .find(|r| r.id == id)
            .map(|r| r.area)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PartEntry {
    id: WidgetId,
    part: PartRef,

    col: Option<usize>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct PartMap {
    entries: Vec<PartEntry>,
}

impl PartMap {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn record(&mut self, id: WidgetId, part: PartRef) {
        self.entries.push(PartEntry {
            id,
            part,
            col: None,
        });
    }

    pub fn record_cell(&mut self, id: WidgetId, part: PartRef, col: usize) {
        self.entries.push(PartEntry {
            id,
            part,
            col: Some(col),
        });
    }

    fn entry(&self, id: WidgetId) -> Option<&PartEntry> {
        self.entries.iter().rev().find(|e| e.id == id)
    }

    pub fn index(&self, id: WidgetId, part: Part) -> Option<usize> {
        self.entry(id)
            .filter(|e| e.part.part == part)
            .and_then(|e| e.part.item)
            .and_then(ItemKey::as_index)
    }

    pub fn cell(&self, id: WidgetId, part: Part) -> Option<(usize, usize)> {
        let e = self.entry(id).filter(|e| e.part.part == part)?;
        Some((e.part.item?.as_index()?, e.col?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_resolve_to_their_owner_and_part() {
        let mut h = HitRegistry::default();
        let list = WidgetId::of("list");
        h.register(list, Rect::new(0, 0, 20, 5));
        let row = PartRef::item(Part::ROW, ItemKey::index(2));
        let id = h.register_part(list, row, Rect::new(0, 2, 20, 1));
        assert_eq!(id, list.part(row));
        let on_row = Hit {
            owner: list,
            part: row,
        };
        assert_eq!(h.resolve(Position::new(3, 2)), Some(on_row));
        assert_eq!(h.part_of(id), Some(on_row));
        assert_eq!(
            h.resolve(Position::new(3, 0)),
            Some(Hit {
                owner: list,
                part: PartRef::of(Part::CONTAINER)
            }),
            "a widget's own region is its container"
        );
        h.register_bar(Bar {
            id: WidgetId::of("list.bar"),
            owner: list,
            axis: Axis::Vertical,
            track: Rect::new(19, 0, 1, 5),
        });
        assert_eq!(
            h.resolve(Position::new(19, 2)),
            Some(Hit {
                owner: list,
                part: PartRef::of(Part::TRACK)
            }),
            "a scrollbar resolves to its owner's track"
        );
        assert_eq!(
            h.bars().iter().map(|b| b.owner).collect::<Vec<_>>(),
            vec![list],
            "the frame's scrollbars are listed"
        );
        h.register_thumb(WidgetId::of("list.bar"), list, Rect::new(19, 1, 1, 2));
        assert_eq!(
            h.resolve(Position::new(19, 2)),
            Some(Hit {
                owner: list,
                part: PartRef::of(Part::THUMB)
            }),
            "the thumb is its own part"
        );
        assert_eq!(
            h.hit(Position::new(19, 2)),
            Some(WidgetId::of("list.bar")),
            "one control: the thumb keeps the bar's id"
        );
        assert_eq!(
            h.resolve(Position::new(19, 4)).map(|h| h.part),
            Some(PartRef::of(Part::TRACK))
        );
        assert_eq!(h.resolve(Position::new(30, 2)), None);
    }

    #[test]
    fn a_resolved_part_respects_barriers() {
        let mut h = HitRegistry::default();
        let list = WidgetId::of("list");
        h.register_part(
            list,
            PartRef::item(Part::ROW, ItemKey::index(0)),
            Rect::new(0, 0, 9, 1),
        );
        h.push_barrier();
        assert_eq!(h.resolve(Position::new(1, 0)), None);
    }

    #[test]
    fn a_part_map_answers_by_id() {
        let w = WidgetId::of("w");
        let mut m = PartMap::default();
        let row = PartRef::item(Part::ROW, ItemKey::index(4));
        m.record(w.part(row), row);
        let cell = PartRef::item(Part::CELL, ItemKey::index(4));
        m.record_cell(w.child(4).child(2), cell, 2);
        m.record(w.part(PartRef::of(Part::NEW)), PartRef::of(Part::NEW));
        assert_eq!(m.index(w.child(4), Part::ROW), Some(4));
        assert_eq!(m.index(w.child(4), Part::TAB), None);
        assert_eq!(m.cell(w.child(4).child(2), Part::CELL), Some((4, 2)));
        assert_eq!(m.index(w.part(PartRef::of(Part::NEW)), Part::NEW), None);
        assert_eq!(m.index(w, Part::ROW), None);
        m.clear();
        assert_eq!(m.index(w.child(4), Part::ROW), None);
    }

    #[test]
    fn scroll_stack_lists_the_row_then_its_list_once_each() {
        let mut h = HitRegistry::default();
        let list = WidgetId::of("list");
        let row = list.child(1);
        h.register(list, Rect::new(0, 0, 20, 5));
        h.register_scroll(list, Rect::new(0, 0, 20, 5));
        h.register(row, Rect::new(0, 1, 20, 1));
        assert_eq!(h.scroll_stack(Position::new(3, 1)), vec![row, list]);
        assert_eq!(h.scroll_stack(Position::new(3, 0)), vec![list]);
    }

    #[test]
    fn hover_only_regions_take_hover_and_let_clicks_and_the_wheel_through() {
        let mut h = HitRegistry::default();
        let card = WidgetId::of("card");
        let title = WidgetId::of("title");
        h.register(card, Rect::new(0, 0, 20, 5));
        h.register_scroll(card, Rect::new(0, 0, 20, 5));
        h.register_hover(title, Rect::new(2, 0, 8, 1));
        let on_title = Position::new(3, 0);
        assert_eq!(h.hover_at(on_title), Some(title));
        assert_eq!(h.hit(on_title), Some(card), "the click is the card's");
        assert_eq!(h.hit_scroll(on_title), Some(card));
        assert_eq!(h.hover_at(Position::new(3, 2)), Some(card));
    }

    #[test]
    fn a_backdrop_reports_what_is_beneath_it_across_empty_layers() {
        let mut h = HitRegistry::default();
        let row = WidgetId::of("row");
        let pop = WidgetId::of("pop");
        let back = WidgetId::of("pop.backdrop");
        h.register(row, Rect::new(0, 5, 20, 1));

        h.push_barrier();
        h.push_barrier();
        h.register_backdrop(back, Rect::new(0, 0, 40, 20), false);
        h.register(pop, Rect::new(10, 0, 10, 4));
        assert_eq!(h.backdrop_at(Position::new(2, 5)), Some((back, Some(row))));
        assert_eq!(h.backdrop_at(Position::new(2, 9)), Some((back, None)));
        assert_eq!(h.backdrop_at(Position::new(12, 1)), None, "on the popup");
    }

    #[test]
    fn a_backdrop_over_a_dialog_never_reaches_the_page_under_the_dialog() {
        let mut h = HitRegistry::default();
        let row = WidgetId::of("row");
        let dialog = WidgetId::of("dialog");
        let back = WidgetId::of("select.backdrop");
        h.register(row, Rect::new(0, 5, 20, 1));
        h.push_barrier();
        h.register(dialog, Rect::new(10, 2, 20, 10));
        h.push_barrier();
        h.register_backdrop(back, Rect::new(0, 0, 40, 20), false);
        assert_eq!(h.backdrop_at(Position::new(2, 5)), Some((back, None)));
        assert_eq!(
            h.backdrop_at(Position::new(12, 5)),
            Some((back, Some(dialog)))
        );
    }

    #[test]
    fn topmost_wins() {
        let mut h = HitRegistry::default();
        let a = WidgetId::of("a");
        let b = WidgetId::of("b");
        h.register(a, Rect::new(0, 0, 10, 10));
        h.register(b, Rect::new(2, 2, 3, 3));
        assert_eq!(h.hit(Position::new(1, 1)), Some(a));
        assert_eq!(h.hit(Position::new(3, 3)), Some(b));
        assert_eq!(h.hit(Position::new(20, 20)), None);
    }

    #[test]
    fn barrier_shadows_lower_regions() {
        let mut h = HitRegistry::default();
        let a = WidgetId::of("a");
        let b = WidgetId::of("b");
        h.register(a, Rect::new(0, 0, 10, 10));
        h.push_barrier();
        h.register(b, Rect::new(2, 2, 3, 3));
        assert_eq!(h.hit(Position::new(1, 1)), None);
        assert_eq!(h.hit(Position::new(3, 3)), Some(b));
    }

    #[test]
    fn pass_through_backdrops_reveal_the_layer_beneath() {
        let mut h = HitRegistry::default();
        let below = WidgetId::of("below");
        let back = WidgetId::of("back");
        let solid = WidgetId::of("solid");
        h.register(below, Rect::new(0, 0, 10, 10));
        h.push_barrier();
        h.register_backdrop(back, Rect::new(0, 0, 10, 10), true);
        let p = Position::new(2, 2);
        assert_eq!(h.hit(p), Some(back));
        assert_eq!(h.pass_through_target(back, p), Some(below));
        h.push_barrier();
        h.register_backdrop(solid, Rect::new(0, 0, 10, 10), false);
        assert_eq!(h.pass_through_target(solid, p), None);
    }

    #[test]
    fn scroll_only_regions_ignore_hover() {
        let mut h = HitRegistry::default();
        let a = WidgetId::of("a");
        h.register_scroll(a, Rect::new(0, 0, 10, 10));
        assert_eq!(h.hit(Position::new(1, 1)), None);
        assert_eq!(h.hit_scroll(Position::new(1, 1)), Some(a));
    }
}
