//! Borrowed `NavList` scrolling uses the same rows for reveal, paint and hits.
use termrock_core::{Id, ItemKey, KeyCode, Response};
use termrock_navigation::{NavList, NavListState};
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
const ID: Id = Id::root("scroll.nav");
struct Page {
    state: NavListState,
    items: Vec<String>,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        NavList::new(ID)
            .scrollable(true)
            .disabled_item(&|s: &String| s.starts_with("Disabled"))
            .key(|s: &String| ItemKey::text(s))
            .section(&|s: &String| s.split_once(':').map_or("Actions", |(group, _)| group))
            .update(cx, &mut self.state, &self.items)
            .erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        NavList::new(ID)
            .scrollable(true)
            .disabled_item(&|s: &String| s.starts_with("Disabled"))
            .key(|s: &String| ItemKey::text(s))
            .section(&|s: &String| s.split_once(':').map_or("Actions", |(group, _)| group))
            .draw(ui, ui.full(), &self.state, &self.items);
    }
}
#[test]
fn end_reveals_the_actual_last_stable_row() {
    let mut h = Harness::new(
        Page {
            state: NavListState::new(),
            items: (0..12).map(|i| format!("Action{i}")).collect(),
        },
        termrock_theme::Theme::junie(),
        24,
        4,
    );
    let _ = h.key(KeyCode::End);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("Action11")));
    assert!(
        (0..4).any(|y| h.row(y).contains("Action11")),
        "End cursor must be painted inside the NavList viewport"
    );
}

#[test]
fn wheel_keeps_cursor_and_reorder_resize_and_removal_reconcile() {
    let mut h = Harness::new(
        Page {
            state: NavListState::new(),
            items: (0..12).map(|i| format!("Action{i}")).collect(),
        },
        termrock_theme::Theme::junie(),
        24,
        4,
    );
    let _ = h.key(KeyCode::End);
    let _ = h.wheel(termrock_core::Axis::V, -3, 2, 2);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("Action11")));
    assert!(
        !(0..4).any(|y| h.row(y).contains("Action11")),
        "wheel must not snap back to cursor"
    );
    let _ = h.key(KeyCode::Up);
    assert!((0..4).any(|y| h.row(y).contains("Action10")));
    h.app_mut().items.reverse();
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("Action10")));
    assert!((0..4).any(|y| h.row(y).contains("Action10")));
    let _ = h.resize(24, 2);
    assert!((0..2).any(|y| h.row(y).contains("Action10")));
    h.app_mut().items.retain(|s| s != "Action10");
    let _ = h.key(KeyCode::Enter);
    assert_ne!(h.app().state.cursor(), Some(ItemKey::text("Action10")));
    h.app_mut().items.clear();
    let _ = h.key(KeyCode::End);
    assert_eq!(h.app().state.cursor(), None);
    assert_eq!(h.app().state.scroll().offset(), 0);
}

#[global_allocator]
static GLOBAL: termrock_test_support::perf::Counting = termrock_test_support::perf::Counting;

#[test]
fn sections_disabled_rows_and_warmed_redraw_use_authoritative_geometry() {
    let mut h = Harness::new(
        Page {
            state: NavListState::new(),
            items: ["A:First", "Disabled", "B:Last"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
        },
        termrock_theme::Theme::junie(),
        24,
        4,
    );
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("B:Last")));
    assert!((0..4).any(|y| h.row(y).contains("B:Last")));
    let rect = h.area_of_part(
        ID,
        termrock_runtime::PartRef::item(termrock_runtime::Part::ROW, ItemKey::text("B:Last")),
    );
    assert!(rect.is_some_and(|r| r.bottom() <= 4));
    assert!(
        h.area_of_part(
            ID,
            termrock_runtime::PartRef::item(termrock_runtime::Part::ROW, ItemKey::text("Disabled"))
        )
        .is_none()
    );
    h.draw();
    let before = termrock_test_support::perf::allocs();
    h.draw();
    assert_eq!(termrock_test_support::perf::allocs() - before, 0);
    let _ = h.resize(1, 1);
    let _ = h.key(KeyCode::Home);
    let _ = h.resize(0, 0);
    assert!(h.area_of(ID).is_none());
}

#[test]
fn opt_in_false_preserves_default_cells_and_no_scrollbar_column() {
    for theme in [
        termrock_theme::Theme::junie(),
        termrock_theme::Theme::paper(),
    ] {
        for level in [
            termrock_theme::ColorLevel::TrueColor,
            termrock_theme::ColorLevel::Ansi256,
            termrock_theme::ColorLevel::Ansi16,
            termrock_theme::ColorLevel::Mono,
        ] {
            let items = ["first", "second", "third", "fourth"];
            let mut default =
                termrock_test_support::Scene::new("nav_default", theme.clone(), level, 24, 3);
            let mut explicit =
                termrock_test_support::Scene::new("nav_false", theme.clone(), level, 24, 3);
            default.draw(|ui, area| {
                NavList::new(ID).draw(ui, area, &NavListState::new(), &items);
            });
            explicit.draw(|ui, area| {
                NavList::new(ID)
                    .scrollable(false)
                    .draw(ui, area, &NavListState::new(), &items);
            });
            assert_eq!(default.buffer(), explicit.buffer());
            assert!(
                default
                    .buffer()
                    .cell((23, 0))
                    .is_some_and(|c| c.symbol() == " ")
            );
        }
    }
}

#[test]
fn owned_scrollbar_parts_reach_actual_cells() {
    let patch = [(
        termrock_runtime::Part::TRACK,
        termrock_theme::StylePatch::new().set_bg(termrock_theme::Role::Custom(
            ratatui_core::style::Color::Red,
        )),
    )];
    let items = ["one", "two", "three", "four", "five", "six"];
    let mut scene = termrock_test_support::Scene::new(
        "nav_track",
        termrock_theme::Theme::junie(),
        termrock_theme::ColorLevel::TrueColor,
        24,
        4,
    );
    scene.draw(|ui, area| {
        NavList::new(ID).scrollable(true).patch_part(&patch).draw(
            ui,
            area,
            &NavListState::new(),
            &items,
        );
    });
    assert_eq!(
        scene.buffer().cell((23, 3)).map(|c| c.bg),
        Some(ratatui_core::style::Color::Red)
    );
}
