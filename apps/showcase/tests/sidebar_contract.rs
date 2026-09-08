//! Coordinates derived from pinned Holla app.rs `draw_sidebar`, not candidate regions.
use junie_tui::{Color, Id, ItemKey, KeyCode, Modifier, MouseKind, Part, PartRef, Rect, Theme};
use junie_tui_testing::Harness;
use showcase_app::{App, NAV_ENTRIES, PageId};

const NAV: Id = Id::root("showcase_app::app::navigation");

fn reference_row(index: usize, height: u16) -> Option<u16> {
    let index = u16::try_from(index).ok()?;
    let y = if height < 31 {
        2u16.saturating_add(index)
    } else if index == 0 {
        3
    } else if index < 20 {
        5u16.saturating_add(index)
    } else {
        7u16.saturating_add(index)
    };
    (y < height.saturating_sub(2)).then_some(y)
}

#[test]
fn every_reference_sidebar_row_activates_its_visible_page() {
    for (width, height) in [(80, 24), (80, 30), (80, 31), (120, 40), (160, 50)] {
        for (index, entry) in NAV_ENTRIES.iter().enumerate() {
            let Some(y) = reference_row(index, height) else {
                continue;
            };
            let mut h = Harness::new(App::new(), Theme::junie(), width, height);
            let _ = h.click(4, y);
            assert_eq!(
                h.app().page(),
                entry.id,
                "{width}x{height}, row {index}, y{y}"
            );
        }
    }
}

#[test]
fn reference_headers_gaps_and_clipped_rows_do_not_activate() {
    for height in [24, 30, 31, 40] {
        let mut h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 80, height);
        for y in 2..height.saturating_sub(2) {
            if NAV_ENTRIES
                .iter()
                .enumerate()
                .any(|(i, _)| reference_row(i, height) == Some(y))
            {
                continue;
            }
            let _ = h.click(4, y);
            assert_eq!(
                h.app().page(),
                PageId::Buttons,
                "non-row y{y} height{height}"
            );
        }
    }
}

#[test]
fn resize_switches_reference_policy_both_directions() {
    let mut h = Harness::new(App::new(), Theme::junie(), 80, 24);
    for height in [30, 31, 40, 31, 30, 24] {
        let _ = h.resize(80, height);
        let y = if height < 31 { 3 } else { 6 };
        let _ = h.click(4, y);
        assert_eq!(h.app().page(), PageId::Buttons, "height{height}");
        let _ = h.click(4, if height < 31 { 2 } else { 3 });
        assert_eq!(h.app().page(), PageId::Overview);
    }
}

#[test]
fn reference_hover_targets_only_the_visible_row() {
    for height in [24, 30, 31, 40] {
        let mut h = Harness::new(App::new(), Theme::junie(), 80, height);
        for (i, _) in NAV_ENTRIES.iter().enumerate() {
            let Some(y) = reference_row(i, height) else {
                continue;
            };
            let _ = h.mouse(MouseKind::Move, 4, y);
            assert_eq!(h.cell(4, y).fg, Color::Rgb(255, 255, 255));
            assert_eq!(h.cell(4, y).bg, Color::Rgb(24, 24, 27));
            for other_y in 2..height.saturating_sub(2) {
                if other_y != y {
                    assert_eq!(
                        h.cell(4, other_y).bg,
                        Color::Rgb(0, 0, 0),
                        "hover y{y} leaked to y{other_y} at height{height}"
                    );
                }
            }
        }
    }
}

#[test]
fn right_and_l_enter_page_but_enter_stays_navigation() {
    for key in [KeyCode::Right, KeyCode::Char('l'), KeyCode::Enter] {
        let mut h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 80, 24);
        let _ = h.key(key);
        if key == KeyCode::Enter {
            assert_eq!(h.focus(), Some(NAV));
        } else {
            // Pinned Buttons registers Run task first; index6/7 are disabled.
            assert_eq!(
                h.focus(),
                Some(Id::root("showcase_app::pages::buttons::buttons").index(0))
            );
        }
    }
}

#[test]
fn reference_row_regions_and_focus_match_at_every_visible_index() {
    for (width, height) in [(80, 24), (80, 30), (80, 31), (120, 40)] {
        let mut h = Harness::new(App::new(), Theme::junie(), width, height);
        for (index, entry) in NAV_ENTRIES.iter().enumerate() {
            let part = PartRef::item(Part::ROW, ItemKey::text(entry.id.slug()));
            let expected = reference_row(index, height)
                .map(|y| Rect::new(0, y, if width < 110 { 19 } else { 24 }, 1));
            assert_eq!(h.area_of_part(NAV, part), expected);
            if let Some(row) = expected {
                assert_eq!(h.cell(0, row.y).symbol(), "▎");
                assert_eq!(h.cell(0, row.y).fg, Color::Rgb(72, 224, 84));
                assert!(h.cell(3, row.y).modifier.contains(Modifier::BOLD));
                assert_eq!(h.cell(3, row.y).bg, Color::Rgb(0, 0, 0));
            }
            let _ = h.key(KeyCode::Down);
        }
    }
}

#[test]
fn pressing_moves_cursor_and_release_elsewhere_does_not_navigate() {
    for height in [24, 30, 31, 40] {
        for (index, _) in NAV_ENTRIES.iter().enumerate() {
            let Some(y) = reference_row(index, height) else {
                continue;
            };
            let mut h = Harness::new(App::new(), Theme::junie(), 80, height);
            let _ = h.mouse(MouseKind::Down, 4, y);
            assert_eq!(h.focus(), Some(NAV));
            assert_eq!(h.app().page(), PageId::Overview);
            assert_eq!(h.cell(4, y).bg, Color::Rgb(255, 255, 255));
            let _ = h.mouse(MouseKind::Up, 79, 1);
            assert_eq!(h.app().page(), PageId::Overview);
            assert!(h.cell(3, y).modifier.contains(Modifier::BOLD));
        }
    }
}

#[test]
fn row_hover_does_not_lift_headers_gaps_or_other_rows() {
    for height in [24, 30, 31, 40] {
        let mut h = Harness::new(App::new(), Theme::junie(), 80, height);
        let hovered_y = if height < 31 { 3 } else { 6 };
        let _ = h.mouse(MouseKind::Move, 4, hovered_y);
        for y in 2..height.saturating_sub(2) {
            if y != hovered_y {
                assert_eq!(
                    h.cell(4, y).bg,
                    Color::Rgb(0, 0, 0),
                    "hover leakage at height{height} y{y}"
                );
            }
        }
    }
}

#[test]
fn entering_new_route_focuses_run_task_and_skips_disabled_buttons() {
    let mut h = Harness::new(App::new(), Theme::junie(), 80, 24);
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Right);
    let buttons = Id::root("showcase_app::pages::buttons::buttons");
    assert_eq!(h.app().page(), PageId::Buttons);
    assert_eq!(h.focus(), Some(buttons.index(0)));
    for index in 1..6 {
        let _ = h.key(KeyCode::Tab);
        assert_eq!(h.focus(), Some(buttons.index(index)));
    }
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(buttons.index(8)));
}

#[test]
fn entering_static_overview_keeps_the_only_navigation_stop() {
    let mut h = Harness::new(App::new(), Theme::junie(), 80, 24);
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().page(), PageId::Overview);
    assert_eq!(h.focus(), Some(NAV));
}
