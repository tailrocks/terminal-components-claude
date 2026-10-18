use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::pages::popups_tooltips::PopupsTooltipsPage;
use crate::pages::template::Section;
use junie_tui::core::event::MouseKind;
use junie_tui::ui::specimen::SpecimenState;

fn page(h: &Harness) -> &PopupsTooltipsPage {
    h.app.pages[PageId::PopupsTooltips.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<PopupsTooltipsPage>())
        .expect("the Popups & tooltips page")
}

fn harness() -> Harness {
    Harness::on(140, 50, PageId::PopupsTooltips)
}

fn open_popover(h: &mut Harness) {
    let (x, y) = h.find("Open popover").expect("the button");
    h.click(x + 1, y);
    assert!(page(h).popover.is_open());
}

fn popover_top(h: &Harness) -> u16 {
    h.find("report.pdf").expect("the popover").1
}

#[test]
fn the_popover_opens_where_its_placement_says_and_closes_by_its_policy() {
    let mut h = harness();
    let button = {
        let (_, y) = h.find("Open popover").unwrap();
        y
    };
    open_popover(&mut h);
    assert!(popover_top(&h) > button, "below the button");
    h.key(KeyCode::Esc);
    assert!(!page(&h).popover.is_open(), "Esc closes");
    assert_eq!(page(&h).last, "Dismissed");
    h.opt("popupstooltips.placement=above");
    open_popover(&mut h);
    assert!(popover_top(&h) < button, "above the button");
    h.key(KeyCode::Esc);

    h.opt("popupstooltips.escape=off");
    open_popover(&mut h);
    h.key(KeyCode::Esc);
    assert!(page(&h).popover.is_open(), "Esc refused by the policy");
    h.key(KeyCode::Enter);
    assert!(!page(&h).popover.is_open());
    assert_eq!(page(&h).last, "Chosen(0) · Open");
}

#[test]
fn the_fit_and_policy_options_change_what_the_page_reads_back() {
    let mut h = harness();
    assert!(h.text().contains("margin 1 · vertical margin as margin"));
    h.opt("popupstooltips.margin=2");
    h.opt("popupstooltips.vertical_margin=0");
    h.opt("popupstooltips.overflow=slide");
    assert!(
        h.text().contains("margin 2 · vertical margin 0"),
        "{}",
        h.text()
    );
    assert!(h.text().contains("Slide"));
    h.opt("popupstooltips.tip_delay=0");
    assert!(h.text().contains("after 0 ms"));
}

#[test]
fn a_resting_pointer_shows_a_tooltip_under_the_page_policy() {
    let mut h = harness();
    h.opt("popupstooltips.tip_delay=0");
    let (x, y) = h.find("Save draft").unwrap();
    h.mouse(MouseKind::Move, x + 1, y);
    h.draw();
    assert!(
        h.text().contains("Save the draft to this device"),
        "the tooltip shows\n{}",
        h.text()
    );

    h.opt("popupstooltips.tooltips=off");
    h.mouse(MouseKind::Move, x + 2, y);
    h.draw();
    let text = h.text();
    let shown = text
        .lines()
        .filter(|l| l.contains("Save the draft to this device") && !l.contains("Tips"))
        .count();
    assert_eq!(shown, 0, "{text}");
}

#[test]
fn the_reveal_key_shows_a_cut_row_whole() {
    let mut h = Harness::on(100, 40, PageId::PopupsTooltips);
    let list = page(&h).paths.id;
    h.focus(list);
    h.key(KeyCode::Down);
    h.key(KeyCode::Right);
    assert!(
        h.text().contains("of every popup"),
        "the reveal shows the cut row whole\n{}",
        h.text()
    );
    h.key(KeyCode::Left);
    h.opt("popupstooltips.reveal_key=none");
    h.focus(list);
    h.key(KeyCode::Right);
    assert!(!h.text().contains("of every popup"), "no reveal key");
}

#[test]
fn the_frame_states_follow_the_frame_options() {
    let mut h = Harness::on(220, 80, PageId::PopupsTooltips);
    h.section(Section::States);
    let rows = |h: &Harness| {
        h.app.pages[PageId::PopupsTooltips.index()]
            .template()
            .unwrap()
            .state_rows()
            .to_vec()
    };
    for s in [
        SpecimenState::Rest,
        SpecimenState::Hover,
        SpecimenState::Pressed,
    ] {
        assert!(rows(&h).contains(&s), "{s:?}");
    }
    assert!(h.text().contains("Frame"));
    let with_close = h.text().matches('×').count();
    assert!(with_close >= 3, "the close button in every row");

    h.opt("popupstooltips.frame_dismiss=never");
    h.section(Section::States);
    assert_eq!(h.text().matches('×').count(), with_close - 3);
}
