//! The showcase pages advertise only what they deliver.
//!
//! These tests drive the real app: the hint bar comes from the shell and
//! the page, and the step rail's title and its count share one row.

use showcase_ui::{App, PageId};
use termrock::{Id, KeyCode, Theme};
use termrock_test_support::Harness;

const CHROME_BRAND: Id = Id::root("showcase_ui::pages::chrome::chrome.brand");

fn harness(page: PageId, width: u16, height: u16) -> Harness<App> {
    Harness::new(App::with_page(page), Theme::junie(), width, height)
}

/// The hint bar is the last row of the frame.
fn hint_bar(harness: &Harness<App>, height: u16) -> String {
    harness.row(height - 1)
}

/// The page's own hints show once the focus leaves the navigation.
fn page_hints(page: PageId, width: u16, height: u16) -> String {
    let mut harness = harness(page, width, height);
    let _ = harness.key(KeyCode::Tab);
    hint_bar(&harness, height)
}

/// The page's own hints once the focus leaves the page's first widget.
fn deep_page_hints(page: PageId, width: u16, height: u16) -> String {
    let mut harness = harness(page, width, height);
    let _ = harness.key(KeyCode::Tab);
    let _ = harness.key(KeyCode::Tab);
    hint_bar(&harness, height)
}

/// The page hints never advertise a key the page does not handle.
#[test]
fn the_pages_advertise_only_the_keys_they_handle() {
    let cases: [(PageId, &[&str]); 3] = [
        (PageId::Terminal, &["Follow", "Copy", "Clear", "Resize"]),
        (PageId::TaskRunner, &["Fold"]),
        (PageId::Chrome, &["Context menu"]),
    ];
    for (page, forbidden) in cases {
        let bar = page_hints(page, 120, 40);
        for key in forbidden {
            assert!(
                !bar.contains(key),
                "{page:?} advertises {key:?} it does not implement: {bar:?}"
            );
        }
    }
}

/// The step rail keeps its title whole and its count in the card's meta.
#[test]
fn the_terminal_rail_keeps_its_title_and_its_count_apart() {
    for (width, height) in [(120u16, 40u16), (100, 30), (80, 24), (72, 20)] {
        let harness = harness(PageId::Terminal, width, height);
        let (title_x, title_y) = harness
            .find("Step rail")
            .unwrap_or_else(|| panic!("{width}x{height}: the step rail title is missing"));
        for (offset, ch) in "Step rail".chars().enumerate() {
            assert_eq!(
                harness.cell(title_x + offset as u16, title_y).symbol(),
                ch.to_string(),
                "{width}x{height}: the step rail title is whole"
            );
        }
        let (count_x, count_y) = harness
            .find("of 7")
            .unwrap_or_else(|| panic!("{width}x{height}: the stage count is missing"));
        assert_eq!(
            count_y, title_y,
            "{width}x{height}: the count rides the title row"
        );
        assert!(
            count_x > title_x + "Step rail".len() as u16,
            "{width}x{height}: the count never lands on the title"
        );
    }
}

/// The chrome page's scrollbar is the shared one: its thumb keeps the
/// library's rest ink.
#[test]
fn the_chrome_scrollbar_keeps_the_shared_scrollbar_inks() {
    let harness = harness(PageId::Chrome, 120, 40);
    let mut thumbs = 0;
    for y in 0..40u16 {
        for x in 0..120u16 {
            let cell = harness.cell(x, y);
            if cell.symbol() == "┃" {
                thumbs += 1;
                assert_eq!(
                    cell.fg,
                    termrock::Color::Rgb(128, 128, 128),
                    "the scrollbar thumb keeps the shared rest ink at ({x},{y})"
                );
            }
        }
    }
    assert!(thumbs > 0, "the chrome page paints no scrollbar thumb");
}

/// The shared scrollbar answers a pointer press with the pressed ink.
#[test]
fn the_chrome_scrollbar_answers_a_press_with_the_pressed_ink() {
    let mut harness = harness(PageId::Chrome, 120, 40);
    let (thumb_x, thumb_y) = harness
        .find("┃")
        .expect("the chrome page paints a scrollbar thumb");
    let _ = harness.mouse(termrock::MouseKind::Down, thumb_x, thumb_y);
    let cell = harness.cell(thumb_x, thumb_y);
    assert_eq!(
        cell.fg,
        termrock::Color::Rgb(72, 224, 84),
        "the pressed scrollbar thumb carries the accent ink"
    );
}

/// The chrome page's playground keeps one section strip.
#[test]
fn the_chrome_page_paints_one_section_strip() {
    let harness = harness(PageId::Chrome, 120, 40);
    assert_eq!(
        harness.count("Playground"),
        1,
        "the chrome page paints one section strip"
    );
}

/// The section strip's rule keeps the approved seam ink.
#[test]
fn the_section_strip_keeps_the_approved_seam_ink() {
    let harness = harness(PageId::TaskRunner, 120, 40);
    let (rule_x, rule_y) = harness.find("━").expect("the section strip's rule");
    let cell = harness.cell(rule_x, rule_y);
    assert_eq!(
        cell.fg,
        termrock::Color::Rgb(72, 224, 84),
        "the active rule carries the accent ink"
    );
    let quiet = harness.cell(rule_x + 20, rule_y);
    assert_eq!(
        quiet.symbol(),
        "─",
        "the quiet rule follows the active rule"
    );
    assert_eq!(
        quiet.fg,
        termrock::Color::Rgb(58, 58, 58),
        "the quiet rule keeps the approved seam ink"
    );
}

/// The task runner's run command reaches the hint bar.
#[test]
fn the_page_hints_reach_the_hint_bar() {
    let bar = deep_page_hints(PageId::TaskRunner, 120, 40);
    assert!(
        bar.contains("Run pipeline"),
        "the task runner advertises its run command: {bar:?}"
    );
}

/// The chrome page owns no context menu and says so.
#[test]
fn a_page_without_commands_advertises_no_key() {
    let bar = page_hints(PageId::Chrome, 120, 40);
    assert!(
        !bar.contains("Context menu"),
        "the chrome page owns no context menu: {bar:?}"
    );
}

/// The chrome page's brand answers a click.
#[test]
fn the_chrome_page_brand_answers_a_click() {
    let mut harness = harness(PageId::Chrome, 120, 40);
    let brand = harness
        .area_of(CHROME_BRAND)
        .expect("the chrome page registers its brand");
    let _ = harness.click(brand.x.saturating_add(1), brand.y);
    assert!(
        harness.text().contains("brand activations: 1"),
        "the brand click reaches the page status"
    );
}

/// The shell's hint bar keeps its own commands.
#[test]
fn the_shell_owns_the_hint_bar() {
    let harness = harness(PageId::Chrome, 120, 40);
    let bar = hint_bar(&harness, 40);
    assert!(
        bar.contains("Quit"),
        "the shell's hint bar keeps its own commands: {bar:?}"
    );
}
