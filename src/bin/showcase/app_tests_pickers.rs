



use junie_tui::runtime::Application;
use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::pages::pickers::PickersPage;
use crate::pages::template::Section;
use junie_tui::core::event::Input;
use junie_tui::ui::specimen::SpecimenState;
use junie_tui::widgets::picker::{Picker, PickerStatus};

fn page(h: &Harness) -> &PickersPage {
    h.app.pages[PageId::Pickers.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<PickersPage>())
        .expect("the Pickers page")
}

fn picker(h: &Harness) -> &Picker {
    h.app.layers.top_as::<Picker>().expect("a picker is open")
}


fn is_open(h: &Harness) -> bool {
    h.app
        .layers
        .top()
        .is_some_and(|e| e.owner == Some(PageId::Pickers.index()))
}

fn harness() -> Harness {
    Harness::on(140, 50, PageId::Pickers)
}

fn open(h: &mut Harness, label: &str) {
    let (x, y) = h.find(label).unwrap_or_else(|| panic!("{label} button"));
    h.click(x + 1, y);
    assert!(is_open(h), "{label} opens a picker");
}

fn tick(h: &mut Harness) {
    h.app.handle(Input::Tick);
    h.draw();
}

#[test]
fn a_remote_fetch_loads_then_is_ready() {
    let mut h = harness();
    open(&mut h, "Remote branch");
    assert!(matches!(picker(&h).status, PickerStatus::Loading(_)));
    assert!(h.text().contains("Fetching branches from origin"));
    for _ in 0..3 {
        tick(&mut h);
    }
    assert_eq!(picker(&h).status, PickerStatus::Ready);
    assert!(h.text().contains("feat/rate-limit"), "{}", h.text());
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert!(!is_open(&h));
    assert_eq!(page(&h).last, "Chosen(1)");
}

#[test]
fn a_failed_fetch_shows_its_error_and_r_retries() {
    let mut h = harness();
    h.opt("pickers.remote=fails");
    open(&mut h, "Remote branch");
    for _ in 0..3 {
        tick(&mut h);
    }
    assert!(matches!(picker(&h).status, PickerStatus::Error { .. }));
    assert!(h.text().contains("Could not reach origin"));

    h.key(KeyCode::Enter);
    assert!(is_open(&h));
    h.key(KeyCode::Char('r'));
    assert!(
        matches!(picker(&h).status, PickerStatus::Loading(_)),
        "retrying"
    );
    assert_eq!(page(&h).last, "retry");
}

#[test]
fn the_command_palette_refuses_a_disabled_row_and_says_why() {
    let mut h = harness();
    open(&mut h, "Run command");
    assert!(h.text().contains("Run a query first"), "the reason shows");
    for c in "export".chars() {
        h.key(KeyCode::Char(c));
    }
    h.key(KeyCode::Enter);
    assert!(is_open(&h), "a disabled row is not chosen");

    h.key(KeyCode::Esc);
    assert!(is_open(&h));
    for c in "format".chars() {
        h.key(KeyCode::Char(c));
    }
    h.key(KeyCode::Enter);
    assert!(!is_open(&h));
    assert!(h.text().contains("Format SQL"), "the result card");
}

#[test]
fn the_tab_switcher_refreshes_live_and_keeps_its_cursor() {
    let mut h = harness();
    open(&mut h, "Switch tab");
    assert!(h.text().contains("2 live"), "{}", h.text());
    assert_eq!(picker(&h).items[0].detail, "0s", "a running tab's time");
    h.key(KeyCode::Down);
    let cursor = picker(&h).list.cursor.index;
    tick(&mut h);
    assert_eq!(picker(&h).items[0].detail, "1s", "it counts up on a tick");
    assert_eq!(
        picker(&h).list.cursor.index,
        cursor,
        "a refresh keeps the cursor row"
    );

    h.key(KeyCode::Delete);
    assert_eq!(page(&h).last, format!("Secondary({cursor})"));
}

#[test]
fn placement_below_anchors_the_picker_under_its_button() {
    let mut h = harness();
    h.opt("pickers.placement=below");
    let (_, by) = h.find("Open quickly").unwrap();
    open(&mut h, "Open quickly");
    assert_eq!(picker(&h).area.y, by + 1, "right under the button");
    h.key(KeyCode::Esc);
    h.opt("pickers.empty=state");
    open(&mut h, "Open quickly");
    for c in "zzzzqqq".chars() {
        h.key(KeyCode::Char(c));
    }
    assert!(h.text().contains("Nothing matches"), "{}", h.text());
}

#[test]
fn the_states_grid_draws_loading_and_error() {
    let mut h = Harness::on(220, 80, PageId::Pickers);
    h.section(Section::States);
    let rows = h.app.pages[PageId::Pickers.index()]
        .template()
        .unwrap()
        .state_rows()
        .to_vec();
    for s in [
        SpecimenState::Rest,
        SpecimenState::Hover,
        SpecimenState::Disabled,
        SpecimenState::Empty,
        SpecimenState::Loading,
        SpecimenState::Error,
        SpecimenState::Overflow,
    ] {
        assert!(rows.contains(&s), "{s:?} in {rows:?}");
    }
    assert!(h.text().contains("Fetching branches"));
    assert!(h.text().contains("Could not reach origin"));
    assert_eq!(h.bars(), 1);
}
