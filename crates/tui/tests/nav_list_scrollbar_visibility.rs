//! Hidden scrollbars retain full-width shared row and wheel ownership.
use junie_tui::{
    App, Axis, Cx, Id, ItemKey, KeyCode, NavList, NavListState, Part, PartRef, Rect, Response,
    ScrollRegion, ScrollState, Theme, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("hidden.nav");
const BARE: Id = Id::root("hidden.bare");
struct Page {
    state: NavListState,
    visible: bool,
    captured: bool,
    items: Vec<String>,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = NavList::new(ID)
            .scrollable(true)
            .scrollbar_visible(self.visible)
            .key(|s: &String| ItemKey::text(s))
            .update(cx, &mut self.state, &self.items)
            .erase();
        self.captured = cx.capture_owner() == Some(ID);
        response
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        NavList::new(ID)
            .scrollable(true)
            .scrollbar_visible(self.visible)
            .key(|s: &String| ItemKey::text(s))
            .draw(ui, ui.full(), &self.state, &self.items);
    }
}
fn page(visible: bool) -> Harness<Page> {
    Harness::new(
        Page {
            state: NavListState::new(),
            visible,
            captured: false,
            items: (0..12).map(|i| format!("row{i}")).collect(),
        },
        Theme::junie(),
        20,
        4,
    )
}
#[test]
fn hidden_bar_preserves_full_row_hit_wheel_and_reveal() {
    let mut h = page(false);
    assert!(h.area_of_part(ID, PartRef::of(Part::TRACK)).is_none());
    assert!(h.area_of_part(ID, PartRef::of(Part::THUMB)).is_none());
    let row = h.area_of_part(ID, PartRef::item(Part::ROW, ItemKey::text("row1")));
    assert!(row.is_some_and(|r| r.width == 20));
    let _ = h.click(19, 1);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("row1")));
    let _ = h.wheel(Axis::V, 3, 19, 1);
    assert!(h.app().state.scroll().offset() > 0);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("row1")));
    let _ = h.key(KeyCode::End);
    assert!(h.text().contains("row11"));
    let _ = h.resize(1, 2);
    assert!(
        h.area_of_part(ID, PartRef::item(Part::ROW, ItemKey::text("row11")))
            .is_some_and(|r| r.width == 1)
    );
    assert!(h.diagnostics().is_empty());
}
#[test]
fn toggling_bar_removes_pointer_parts_without_losing_scroll() {
    let mut h = page(true);
    assert!(h.area_of_part(ID, PartRef::of(Part::TRACK)).is_some());
    let _ = h.key(KeyCode::End);
    let _ = h.key(KeyCode::Null);
    let offset = h.app().state.scroll().offset();
    h.app_mut().visible = false;
    h.draw();
    assert!(h.area_of_part(ID, PartRef::of(Part::TRACK)).is_none());
    assert_eq!(h.app().state.scroll().offset(), offset);
    let _ = h.click(19, 3);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("row11")));
    assert_eq!(h.app().state.scroll().offset(), offset);
    assert!(h.diagnostics().is_empty());
}
#[test]
fn explicit_visible_preserves_default_cells_in_both_themes_and_modes() {
    for theme in [Theme::junie(), Theme::paper()] {
        for level in [
            junie_tui::ColorLevel::TrueColor,
            junie_tui::ColorLevel::Ansi256,
            junie_tui::ColorLevel::Ansi16,
            junie_tui::ColorLevel::Mono,
        ] {
            let mut a = junie_tui_testing::Scene::new("default", theme.clone(), level, 20, 3);
            let mut b = junie_tui_testing::Scene::new("visible", theme.clone(), level, 20, 3);
            let items = ["one", "two", "three", "four"];
            a.draw(|ui, area| {
                NavList::new(ID)
                    .scrollable(true)
                    .draw(ui, area, &NavListState::new(), &items);
            });
            b.draw(|ui, area| {
                NavList::new(ID)
                    .scrollable(true)
                    .scrollbar_visible(true)
                    .draw(ui, area, &NavListState::new(), &items);
            });
            assert_eq!(a.buffer(), b.buffer());
        }
    }
}

#[test]
fn hiding_during_thumb_drag_releases_capture_without_scrolling() {
    let mut h = page(true);
    let _ = h.mouse(junie_tui::MouseKind::Down, 19, 0);
    assert!(h.app().captured);
    let _ = h.key(KeyCode::Null);
    let offset = h.app().state.scroll().offset();
    h.app_mut().visible = false;
    let _ = h.mouse(junie_tui::MouseKind::Drag, 19, 3);
    assert!(!h.app().captured);
    assert_eq!(h.app().state.scroll().offset(), offset);
    let _ = h.mouse(junie_tui::MouseKind::Up, 19, 3);
    assert!(h.diagnostics().is_empty());
}

struct BarePage {
    scroll: ScrollState,
    visible: bool,
    content_width: std::cell::Cell<u16>,
}
impl App for BarePage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        ScrollRegion::new(BARE)
            .scrollbar_visible(self.visible)
            .update(cx, &mut self.scroll, 100)
            .erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let area = Rect::new(2, 1, 6, 8);
        let content = ScrollRegion::new(BARE)
            .scrollbar_visible(self.visible)
            .draw(ui, area, &self.scroll, 100);
        self.content_width.set(content.width);
    }
}

#[test]
fn scrollregion_hidden_bar_keeps_full_content_width_and_measure_shrinks() {
    for (visible, expected_min, overflow_width) in [(true, 2_u16, 5_u16), (false, 1, 6)] {
        let mut scene = junie_tui_testing::Scene::new(
            "measure",
            Theme::junie(),
            junie_tui::ColorLevel::TrueColor,
            10,
            10,
        );
        scene.draw(|ui, _| {
            let bar = ScrollRegion::new(BARE)
                .scrollbar_visible(visible)
                .measure(ui, junie_tui::Constraints::loose(20, 10));
            assert_eq!(bar.min, (expected_min, 1), "visible={visible}");
        });
        let state = {
            let mut s = ScrollState::new(100);
            s.set_viewport(8);
            s
        };
        let mut scene = junie_tui_testing::Scene::new(
            "draw",
            Theme::junie(),
            junie_tui::ColorLevel::TrueColor,
            10,
            10,
        );
        let width = std::cell::Cell::new(0_u16);
        scene.draw(|ui, _| {
            let area = Rect::new(2, 1, 6, 8);
            let content = ScrollRegion::new(BARE)
                .scrollbar_visible(visible)
                .draw(ui, area, &state, 100);
            width.set(content.width);
        });
        assert_eq!(width.get(), overflow_width, "visible={visible}");
    }
}

#[test]
fn scrollregion_hidden_bar_still_routes_wheel_and_preserves_offset_history() {
    let mut h = junie_tui_testing::Harness::new(
        BarePage {
            scroll: ScrollState::new(100),
            visible: false,
            content_width: std::cell::Cell::new(0),
        },
        Theme::junie(),
        10,
        12,
    );
    let _ = h.wheel(Axis::V, 4, 3, 3);
    assert!(h.app().scroll.offset() > 0);
    h.draw();
    assert_eq!(h.app().content_width.get(), 6);
}
