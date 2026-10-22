use showcase_ui::{App, PageId};
use termrock::{Color, Id, KeyCode, Theme};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");

fn visible_nav_row(h: &Harness<App>, label: &str) -> Option<(u16, u16)> {
    let area = h.area_of(NAV)?;
    (area.y..area.bottom()).find_map(|y| {
        let row = h.row(y);
        let sidebar = row
            .chars()
            .take(usize::from(area.width))
            .collect::<String>();
        sidebar.find(label).map(|byte| {
            let x = sidebar[..byte].chars().count();
            (
                u16::try_from(x).expect("navigation label position fits u16"),
                y,
            )
        })
    })
}

fn require(found: Option<(u16, u16)>, what: &str) -> (u16, u16) {
    found.unwrap_or_else(|| panic!("{what}"))
}

#[test]
fn overview_shell_uses_baseline_sidebar_colors() {
    let h = Harness::new(App::new(), Theme::junie(), 120, 40);
    let text = h.text();
    assert!(text.contains("Junie Design system"));
    assert!(text.contains("/ Foundations / Overview"));
    assert!(text.contains("Text areas"));
    assert!(text.contains("App shell"), "catalog section is visible");
    let (sx, sy) = require(visible_nav_row(&h, "Text areas"), "idle catalog leaf");
    assert_eq!(h.cell(sx, sy).fg, Color::Rgb(0xb3, 0xb3, 0xb3));
    assert_eq!(h.cell(sx, sy).bg, Color::Rgb(0, 0, 0));
}

#[test]
fn next_page_from_overview_opens_chrome() {
    let mut h = Harness::new(App::new(), Theme::junie(), 120, 40);
    let _ = h.key(KeyCode::Char(']'));
    assert_eq!(h.app().page(), PageId::Chrome);
    assert!(h.diagnostics().is_empty());
}

#[test]
fn pointer_release_outside_the_row_does_not_leave_overview() {
    let mut h = Harness::new(App::new(), Theme::junie(), 120, 40);
    let (x, y) = require(h.find("Overview"), "overview row");
    let _ = h.mouse(termrock::MouseKind::Down, x, y);
    assert_eq!(h.app().page(), PageId::Overview);
    let _ = h.mouse(termrock::MouseKind::Up, 110, 1);
    assert_eq!(h.app().page(), PageId::Overview);
}

#[test]
fn entering_static_overview_keeps_the_only_navigation_stop() {
    let mut h = Harness::new(App::new(), Theme::junie(), 80, 24);
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().page(), PageId::Overview);
    assert_eq!(h.focus(), Some(NAV));
}
