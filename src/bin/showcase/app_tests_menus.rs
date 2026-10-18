



use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::pages::menus::MenusPage;
use crate::pages::template::Section;
use junie_tui::core::event::MouseKind;
use junie_tui::ui::specimen::SpecimenState;

fn menus(h: &Harness) -> &MenusPage {
    h.app.pages[PageId::Menus.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<MenusPage>())
        .expect("the Menus page")
}

fn harness() -> Harness {
    Harness::on(120, 40, PageId::Menus)
}

#[test]
fn a_key_the_menu_has_no_use_for_closes_it_unless_swallowed() {
    let shift = ratatui::crossterm::event::KeyModifiers::SHIFT;
    let mut h = harness();
    let files = menus(&h).files.id;
    h.focus(files);
    h.key_mod(KeyCode::F(10), shift);
    h.key(KeyCode::Char('z'));
    assert!(!menus(&h).row_menu.is_open(), "closed, the key passed on");
    h.opt("menus.unknown_key=swallow");
    h.focus(files);
    h.key_mod(KeyCode::F(10), shift);
    h.key(KeyCode::Char('z'));
    assert!(menus(&h).row_menu.is_open(), "swallowed, still open");
}

#[test]
fn shift_f10_opens_the_cursor_rows_menu_and_enter_chooses() {
    let mut h = harness();
    let files = menus(&h).files.id;
    h.focus(files);
    h.key_mod(
        KeyCode::F(10),
        ratatui::crossterm::event::KeyModifiers::SHIFT,
    );
    assert!(menus(&h).row_menu.is_open());
    assert!(menus(&h).request.contains("report.pdf · under its row"));
    assert!(h.text().contains("Duplicate"), "the menu is drawn");
    assert_eq!(h.bars(), 0, "the open menu owns the keyboard: no ▎");

    for _ in 0..3 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    assert!(!menus(&h).row_menu.is_open());
    assert_eq!(menus(&h).last, "Chosen(3) · Copy path on report.pdf");

    h.key_mod(
        KeyCode::F(10),
        ratatui::crossterm::event::KeyModifiers::SHIFT,
    );
    h.key(KeyCode::Esc);
    assert_eq!(menus(&h).last, "Dismissed");
}

#[test]
fn a_right_click_on_a_read_only_file_shows_reasons_and_no_danger() {
    let mut h = harness();
    let (x, y) = h.find("README.md").expect("the row");
    h.mouse(MouseKind::Secondary, x, y);
    assert!(menus(&h).row_menu.is_open());
    assert!(h.text().contains("Read-only file"), "the reason is shown");
    assert!(menus(&h).request.contains("README.md · at the pointer"));
    assert_eq!(menus(&h).files.cursor, 5, "the click moved the cursor");

    let (tx, ty) = h.find("Move to Trash").unwrap();
    let ink = h.term.backend().buffer()[(tx, ty)].fg;
    assert_ne!(ink, junie_tui::theme::Theme::junie().error);
    h.key(KeyCode::Esc);

    h.opt("menus.moves_cursor=off");
    let (x, y) = h.find("photo.png").expect("the row");
    h.mouse(MouseKind::Secondary, x, y);
    assert!(menus(&h).request.contains("photo.png"));
    assert_eq!(menus(&h).files.cursor, 5, "the cursor stays on README.md");
}

#[test]
fn press_drag_release_chooses_the_row_under_the_release() {
    let mut h = harness();
    let (x, y) = h.find("notes.md").expect("the row");
    h.mouse(MouseKind::Secondary, x, y);
    let (dx, dy) = h.find("Duplicate").expect("the menu row");
    h.mouse(MouseKind::SecondaryDrag, dx + 1, dy);
    h.mouse(MouseKind::SecondaryUp, dx + 1, dy);
    assert_eq!(menus(&h).last, "Chosen(2) · Duplicate on notes.md");

    h.opt("menus.release_selects=off");
    h.mouse(MouseKind::Secondary, x, y);
    let (dx, dy) = h.find("Duplicate").unwrap();
    h.mouse(MouseKind::SecondaryDrag, dx + 1, dy);
    h.mouse(MouseKind::SecondaryUp, dx + 1, dy);
    assert!(menus(&h).row_menu.is_open(), "still open");
}

#[test]
fn the_context_keys_follow_the_options() {
    let mut h = harness();
    let files = menus(&h).files.id;
    h.focus(files);
    h.key(KeyCode::Char('m'));
    assert!(!menus(&h).row_menu.is_open(), "no extra key by default");
    h.opt("menus.extra_key=m");
    h.focus(files);
    h.key(KeyCode::Char('m'));
    assert!(menus(&h).row_menu.is_open(), "m opens it");
    h.key(KeyCode::Esc);
    h.opt("menus.shift_f10=off");
    h.focus(files);
    h.key_mod(
        KeyCode::F(10),
        ratatui::crossterm::event::KeyModifiers::SHIFT,
    );
    assert!(!menus(&h).row_menu.is_open(), "Shift+F10 turned off");
    assert!(h.text().contains("m (extra key)"));
    assert!(!h.text().contains("Shift+F10 ·"));
}

#[test]
fn the_actions_menu_opens_under_its_button_and_scrolls_under_max_rows() {
    let mut h = harness();
    let (bx, by) = h.find("Actions ▾").expect("the button");
    h.click(bx + 1, by);
    assert!(menus(&h).actions_menu.is_open());
    let (_, ny) = h.find("New file").expect("the first row");
    assert!(ny > by, "below the button");
    assert!(h.text().contains("Empty Trash"), "all rows fit");
    h.key(KeyCode::Esc);
    h.opt("menus.max_rows=4");
    h.opt("menus.placement=above");
    let (bx, by) = h.find("Actions ▾").unwrap();
    h.click(bx + 1, by);
    let (_, ny) = h.find("New file").expect("the first row");
    assert!(ny < by, "above the button");
    assert!(!h.text().contains("Empty Trash"), "the menu scrolls");
    assert!(h.text().matches('▾').count() >= 2, "with a marker below");

    h.click(2, 30);
    assert!(!menus(&h).actions_menu.is_open());
    h.opt("menus.dismiss=escape");
    let (bx, by) = h.find("Actions ▾").unwrap();
    h.click(bx + 1, by);
    let (x, y) = h.find("Request").unwrap();
    h.click(x, y);
    assert!(menus(&h).actions_menu.is_open(), "only Esc closes it");
    h.key(KeyCode::Esc);
    assert!(!menus(&h).actions_menu.is_open());
}

#[test]
fn the_states_grid_draws_every_menu_state() {
    let mut h = harness();
    h.section(Section::States);
    let page = &h.app.pages[PageId::Menus.index()];
    let rows = page.template().unwrap().state_rows().to_vec();
    use SpecimenState::*;
    for s in [Rest, Hover, Pressed, Disabled] {
        assert!(rows.contains(&s), "{s:?} row drawn");
    }
    assert!(h.text().contains("Share…    Off"), "a reason in its column");
}
