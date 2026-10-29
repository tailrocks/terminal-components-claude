//! The Master-detail page through the real app (plan §7 SC38): both panes
//! wide with a seam that drags and moves with the resize chords, the
//! collapsed list with its summary, the drawer that Tab or Show details
//! opens and Esc closes, and the options that drive the `MasterDetail`.

use junie_tui::core::event::MouseKind;
use junie_tui::core::id::WidgetId;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::PageId;
use crate::app_tests_feedback::Harness;
use crate::pages::template::Section;

const LIST: WidgetId = WidgetId::of("masterdetail").sub("tables");
const DETAIL: WidgetId = WidgetId::of("masterdetail").sub("facts");
const TOGGLE: WidgetId = WidgetId::of("masterdetail").sub("toggle");

fn seam() -> WidgetId {
    LIST.sub("seam")
}

/// Width of the Tables pane: its frame from `╭` to `╮`.
fn tables_width(h: &Harness) -> u16 {
    let (x, y) = h.at("╭─ Tables");
    let row: Vec<String> = (x..h.term.backend().buffer().area.width)
        .map(|cx| h.cell(cx, y).symbol().to_owned())
        .collect();
    row.iter().position(|s| s == "╮").expect("pane frame") as u16 + 1
}

#[test]
fn master_detail_shows_both_panes_wide_and_no_stub() {
    let h = Harness::new(120, 40, PageId::MasterDetail);
    let text = h.text();
    assert!(!text.contains("arrives with"), "{text}");
    assert!(text.contains("Tables") && text.contains("orders"), "{text}");
    assert!(text.contains("Rows") && text.contains("12,408"), "{text}");
    assert!(!h.area(seam()).is_empty(), "the seam is drawn");
}

#[test]
fn master_detail_follows_the_list_cursor() {
    let mut h = Harness::new(120, 40, PageId::MasterDetail);
    h.focus(LIST);
    h.key(KeyCode::Down);
    assert!(h.find("╭─ order_items").is_some(), "{}", h.text());
    assert!(h.text().contains("48,221"));
}

#[test]
fn master_detail_seam_drags_and_the_chords_resize_it() {
    // Alt chords delivered (a Linux client, WezTerm): the hints this
    // test reads do not depend on the host terminal (A29)
    junie_tui::core::keyboard::set_alt_keys(junie_tui::core::keyboard::AltKeys::Delivered);
    let mut h = Harness::new(120, 40, PageId::MasterDetail);
    let before = tables_width(&h);
    let s = h.area(seam());
    h.mouse(MouseKind::Down, s.x, s.y + 2);
    h.mouse(MouseKind::Drag, s.x + 8, s.y + 2);
    h.mouse(MouseKind::Up, s.x + 8, s.y + 2);
    assert_eq!(tables_width(&h), before + 8);
    h.focus(LIST);
    h.key_mod(KeyCode::Left, KeyModifiers::ALT | KeyModifiers::SHIFT);
    assert_eq!(tables_width(&h), before + 6);
    assert!(h.text().contains("Alt+Shift+←→ Resize"), "{}", h.text());
    // the custom chords and step
    h.opt("masterdetail.keys=custom");
    h.opt("masterdetail.step=5");
    let w = tables_width(&h);
    h.focus(LIST);
    h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    assert_eq!(tables_width(&h), w + 5);
}

#[test]
fn master_detail_narrow_shows_the_list_with_a_summary_and_tab_opens_the_drawer() {
    let mut h = Harness::new(120, 40, PageId::MasterDetail);
    h.opt("masterdetail.width=72");
    let text = h.text();
    assert!(text.contains("→ orders · 12,408 rows"), "{text}");
    assert!(h.find("╭─ orders").is_none(), "no detail pane: {text}");
    h.focus(LIST);
    h.key(KeyCode::Tab);
    assert!(h.find("╭─ orders").is_some(), "the drawer: {}", h.text());
    assert!(h.find("╭─ Tables").is_none(), "the drawer covers the list");
    h.key(KeyCode::Esc);
    assert!(h.find("╭─ Tables").is_some(), "Esc returns: {}", h.text());
    assert!(h.area(LIST).width > 0);
}

#[test]
fn master_detail_show_details_toggles_the_drawer() {
    let mut h = Harness::new(120, 40, PageId::MasterDetail);
    h.opt("masterdetail.width=72");
    assert!(h.find("Show details").is_some(), "{}", h.text());
    h.click_id(TOGGLE);
    assert!(h.find("Hide details").is_some(), "{}", h.text());
    assert!(h.find("╭─ orders").is_some());
    h.click_id(TOGGLE);
    assert!(h.find("╭─ Tables").is_some(), "{}", h.text());
}

#[test]
fn master_detail_options_drive_the_layout() {
    let mut h = Harness::new(120, 40, PageId::MasterDetail);
    // a fixed share: the chords do not move it and are not hinted
    h.opt("masterdetail.size=share");
    let w = tables_width(&h);
    h.focus(LIST);
    h.key_mod(KeyCode::Right, KeyModifiers::ALT | KeyModifiers::SHIFT);
    assert_eq!(tables_width(&h), w);
    assert!(!h.text().contains("Resize"), "{}", h.text());
    // collapsing above the frame width
    h.opt("masterdetail.size=resizable");
    h.opt("masterdetail.collapse=140");
    assert!(h.find("╭─ orders").is_none(), "{}", h.text());
    // no summary
    h.opt("masterdetail.summary=0");
    assert!(!h.text().contains("→ orders"));
    // Esc does not leave the drawer when turned off
    h.opt("masterdetail.esc_returns=off");
    h.focus(DETAIL);
    h.key(KeyCode::Esc);
    assert!(h.find("╭─ orders").is_some(), "{}", h.text());
}

#[test]
fn master_detail_states_grid_lists_its_rows() {
    let mut h = Harness::new(120, 40, PageId::MasterDetail);
    h.section(Section::States);
    let text = h.text();
    for row in ["rest", "hover", "pressed", "narrow"] {
        assert!(text.contains(row), "{row}: {text}");
    }
    assert!(text.contains("Master-detail"));
}

#[test]
fn master_detail_reads_at_80x24() {
    // laid out at its natural width (Priority 0b), the demo keeps both
    // panes at 80 columns and the page scrolls sideways to the facts
    let h = Harness::new(80, 24, PageId::MasterDetail);
    let text = h.text();
    assert!(
        text.contains("Tables") && text.contains("╭─ orders"),
        "{text}"
    );
}
