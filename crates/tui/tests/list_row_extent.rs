//! Multiline keyed List geometry, scrolling and publication contracts.
use junie_tui::{
    App, Axis, Cx, Family, Id, ItemKey, KeyCode, List, ListState, Part, PartRef, Rect, Response,
    StateFlags, Theme, Ui, Variant,
};
use junie_tui_testing::Harness;
use std::cell::RefCell;
const ID: Id = Id::root("multiline.list");
struct Page {
    state: ListState,
    items: Vec<u64>,
    height: u16,
    gap: u16,
    whole: bool,
    clip: Option<Rect>,
    row_ui: bool,
    seen: RefCell<Vec<(ItemKey, Rect)>>,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        List::new(ID)
            .key(|x: &u64| ItemKey::num(*x))
            .row_height(self.height)
            .row_gap(self.gap)
            .whole_rows(self.whole)
            .update(cx, &mut self.state, &self.items)
            .erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        self.seen.borrow_mut().clear();
        let paint = |ui: &mut Ui<'_>, area: Rect, flags: StateFlags, key, _: &u64| {
            self.seen.borrow_mut().push((key, area));
            let style = ui
                .style(Family::LIST, Variant::DEFAULT, Part::CONTAINER, flags)
                .style;
            for line in 0..area.height {
                ui.paint_str(
                    Rect::new(area.x, area.y.saturating_add(line), area.width, 1),
                    match line {
                        0 => "first",
                        1 => "second",
                        _ => "third",
                    },
                    style,
                );
            }
        };
        let area = ui.full();
        let draw = |ui: &mut Ui<'_>| {
            let list = List::new(ID)
                .key(|x: &u64| ItemKey::num(*x))
                .row_height(self.height)
                .row_gap(self.gap)
                .whole_rows(self.whole);
            if self.row_ui {
                list.row(|_: &u64, row: &mut junie_tui::RowUi<'_>| {
                    self.seen.borrow_mut().push((row.key(), row.area()));
                    row.label("label");
                })
                .draw(ui, area, &self.state, &self.items);
            } else {
                list.render_row(&paint)
                    .draw(ui, area, &self.state, &self.items);
            }
        };
        if let Some(clip) = self.clip {
            ui.with_area(clip, draw);
        } else {
            draw(ui);
        }
    }
}
fn make(height: u16, gap: u16, whole: bool, viewport: u16) -> Harness<Page> {
    Harness::new(
        Page {
            state: ListState::default(),
            items: vec![10, 20, 30, 40, 50],
            height,
            gap,
            whole,
            clip: None,
            row_ui: false,
            seen: RefCell::new(Vec::with_capacity(16)),
        },
        Theme::junie(),
        20,
        viewport,
    )
}
fn row(h: &Harness<Page>, key: u64) -> Option<Rect> {
    h.area_of_part(ID, PartRef::item(Part::ROW, ItemKey::num(key)))
}
#[test]
fn three_lines_share_key_and_gap_has_no_item_hit() {
    let mut h = make(3, 1, true, 8);
    assert_eq!(row(&h, 10), Some(Rect::new(0, 0, 19, 3)));
    assert_eq!(row(&h, 20), Some(Rect::new(0, 4, 19, 3)));
    assert_eq!(row(&h, 30), None);
    for y in 0..3 {
        let _ = h.click(2, y);
        assert_eq!(h.app().state.chosen(), Some(ItemKey::num(10)));
    }
    let _ = h.click(2, 3);
    assert_eq!(h.app().state.chosen(), Some(ItemKey::num(10)));
    let _ = h.click(2, 6);
    assert_eq!(h.app().state.chosen(), Some(ItemKey::num(20)));
    assert_eq!(h.buffer()[(0, 3)].symbol(), " ");
}
#[test]
fn reveal_wheel_and_resize_preserve_stride_and_reach_last_key() {
    let mut h = make(3, 1, true, 7);
    let _ = h.key(KeyCode::End);
    assert!(row(&h, 50).is_some());
    assert_eq!(h.app().state.scroll().offset() % 4, 0);
    let _ = h.key(KeyCode::Home);
    assert!(row(&h, 10).is_some());
    let _ = h.wheel(Axis::V, 1, 2, 1);
    assert_eq!(h.app().state.scroll().offset(), 12); // Junie admits three row units per wheel notch.
    assert_eq!(row(&h, 40).map(|r| r.y), Some(0));
    let _ = h.resize(20, 3);
    let _ = h.key(KeyCode::End);
    assert_eq!(row(&h, 50).map(|r| r.height), Some(3));
    let _ = h.resize(20, 2);
    assert!(h.app().seen.borrow().is_empty());
    h.app_mut().whole = false;
    h.draw();
    assert_eq!(row(&h, 50).map(|r| r.height), Some(2));
    assert_eq!(h.app().seen.borrow().last().map(|r| r.1.height), Some(3));
}
#[test]
fn partial_bottom_row_retains_logical_height_and_clipped_hit() {
    let h = make(3, 1, false, 6);
    assert_eq!(row(&h, 20), Some(Rect::new(0, 4, 19, 2)));
    assert_eq!(h.app().seen.borrow().get(1).map(|r| r.1.height), Some(3));
}
#[test]
fn zero_height_normalizes_and_default_rows_keep_one_line_spacing() {
    let a = make(1, 0, false, 4);
    let b = make(0, 0, false, 4);
    assert_eq!(a.buffer(), b.buffer());
    for (i, k) in [10, 20, 30, 40].into_iter().enumerate() {
        assert_eq!(row(&a, k), Some(Rect::new(0, i as u16, 19, 1)));
    }
}

#[global_allocator]
static ALLOCATOR: junie_tui_testing::perf::Counting = junie_tui_testing::perf::Counting;
#[test]
fn warmed_multiline_publication_has_no_allocations() {
    let mut h = make(3, 1, false, 7);
    for _ in 0..4 {
        h.draw();
    }
    let before = junie_tui_testing::perf::allocs();
    for _ in 0..100 {
        h.draw();
    }
    assert_eq!(junie_tui_testing::perf::allocs() - before, 0);
}
#[test]
fn scrollbar_drag_and_multi_step_wheel_share_row_aligned_extent() {
    let mut h = make(3, 1, false, 7);
    let _ = h.wheel(Axis::V, 2, 2, 1);
    assert_eq!(h.app().state.scroll().offset(), 12); // Two notches clamp at the final aligned viewport.
    let _ = h.key(KeyCode::Home);
    let thumb = h.area_of_part(ID, PartRef::of(Part::THUMB));
    assert!(thumb.is_some());
    let thumb = thumb.unwrap_or_default();
    let _ = h.mouse(junie_tui::MouseKind::Down, thumb.x, thumb.y);
    let _ = h.mouse(junie_tui::MouseKind::Drag, thumb.x, 6);
    let _ = h.mouse(junie_tui::MouseKind::Up, thumb.x, 6);
    assert_eq!(h.app().state.scroll().offset() % 4, 0);
    assert!(row(&h, 50).is_some());
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(10)));
}
#[test]
fn keyed_reorder_remove_and_geometry_change_keep_reveal_valid() {
    let mut h = make(3, 1, false, 7);
    let _ = h.key(KeyCode::End);
    h.app_mut().items.swap(0, 4);
    h.draw();
    let _ = h.key(KeyCode::Up);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(50)));
    assert!(row(&h, 50).is_some());
    h.app_mut().items = vec![70];
    h.draw();
    let _ = h.key(KeyCode::End);
    assert_eq!(row(&h, 70).map(|r| r.y), Some(0));
    h.app_mut().height = 2;
    h.app_mut().gap = 2;
    h.draw();
    assert_eq!(row(&h, 70).map(|r| r.height), Some(2));
    h.app_mut().items.clear();
    h.draw();
    assert!(h.app().seen.borrow().is_empty());
}

#[test]
fn ancestor_clip_preserves_three_line_logical_row_and_clips_hits() {
    let mut h = make(3, 1, false, 8);
    h.app_mut().clip = Some(Rect::new(2, 1, 10, 5));
    h.draw();
    assert_eq!(row(&h, 10), Some(Rect::new(2, 1, 10, 2)));
    assert_eq!(row(&h, 20), Some(Rect::new(2, 4, 10, 2)));
    assert_eq!(
        h.app().seen.borrow().first().map(|r| r.1),
        Some(Rect::new(0, 0, 19, 3))
    );
    assert_eq!(h.buffer()[(2, 1)].symbol(), "s"); // second line, not a shifted first line
    h.app_mut().clip = Some(Rect::new(2, 4, 10, 2));
    h.draw();
    assert_eq!(h.app().seen.borrow().len(), 1);
    assert_eq!(row(&h, 10), None);
}
#[test]
fn row_ui_receives_full_height_and_explicit_reconcile_preserves_visual_offset() {
    let mut h = make(3, 1, false, 7);
    h.app_mut().row_ui = true;
    h.draw();
    assert_eq!(h.app().seen.borrow().first().map(|r| r.1.height), Some(3));
    let _ = h.key(KeyCode::End);
    let offset = h.app().state.scroll().offset();
    {
        let app = h.app_mut();
        app.items.push(60);
        junie_tui::Reconcile::reconcile(&mut app.state, app.items.len(), |i| {
            ItemKey::num(app.items.get(i).copied().unwrap_or_default())
        });
    }
    assert_eq!(h.app().state.scroll().offset(), offset);
    h.draw();
    let _ = h.key(KeyCode::End);
    assert!(row(&h, 60).is_some());
}

#[test]
fn tiny_viewports_reach_every_key_and_maximum_extents_saturate_safely() {
    let mut h = make(3, 1, false, 1);
    for key in [10, 20, 30, 40, 50] {
        assert!(row(&h, key).is_some());
        let _ = h.key(KeyCode::Down);
    }
    h.app_mut().height = u16::MAX;
    h.app_mut().gap = u16::MAX;
    h.app_mut().row_ui = true;
    h.draw();
    let _ = h.key(KeyCode::End);
    assert_eq!(row(&h, 50).map(|r| r.height), Some(1));
    let _ = h.resize(0, 0);
    let _ = h.resize(20, 3);
    let _ = h.key(KeyCode::Home);
    assert!(row(&h, 10).is_some());
}
