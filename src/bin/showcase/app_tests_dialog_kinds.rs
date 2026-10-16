//! The Dialog kinds page (plan §7.5, SC18), driven through the real app:
//! the form's live preview, disclosure, chooser (a picker stacked over the
//! form), Test action and Save; the info dialog's copy; the choice; the
//! keyboard reference; the options; the States grid.

use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::pages::dialog_kinds::DialogKindsPage;
use crate::pages::template::Section;
use junie_tui::ui::specimen::SpecimenState;
use junie_tui::widgets::form_dialog::FormDialog;
use junie_tui::widgets::picker::Picker;

fn page(h: &Harness) -> &DialogKindsPage {
    h.app.pages[PageId::DialogKinds.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<DialogKindsPage>())
        .expect("the Dialog kinds page")
}

fn harness() -> Harness {
    Harness::on(140, 50, PageId::DialogKinds)
}

fn open(h: &mut Harness, label: &str) {
    let (x, y) = h.find(label).unwrap_or_else(|| panic!("{label} button"));
    h.click(x + 1, y);
    assert!(page(h).host.is_open(), "{label} opens a layer");
}

fn form(h: &Harness) -> &FormDialog {
    page(h)
        .host
        .layers
        .top_as::<FormDialog>()
        .expect("the form is on top")
}

fn last(h: &Harness) -> String {
    page(h).log.last().cloned().unwrap_or_default()
}

#[test]
fn the_form_previews_discloses_tests_and_saves() {
    let mut h = harness();
    open(&mut h, "Add connection…");
    assert_eq!(h.bars(), 1, "one ▎, inside the form");
    assert!(
        h.text()
            .contains("postgres://db.eu-west-1.internal/orders-prod")
    );
    // the masked password shows only its last four characters
    assert!(h.text().contains("7731"), "{}", h.text());
    assert!(!h.text().contains("hunter2"));
    // the name field opens with its text selected: typing replaces it,
    // and the live preview follows once the field commits (Tab)
    for c in "billing".chars() {
        h.key(KeyCode::Char(c));
    }
    h.key(KeyCode::Tab);
    assert!(
        h.text()
            .contains("postgres://db.eu-west-1.internal/billing"),
        "the preview follows the field\n{}",
        h.text()
    );
    // the tunnel check discloses the hidden key chooser
    assert!(!form(&h).field("key").unwrap().visible);
    let (x, y) = h.find("Connect through an SSH tunnel").unwrap();
    h.click(x, y);
    assert!(form(&h).field("key").unwrap().visible, "{}", last(&h));
    // the Test action reports in the form
    let (x, y) = h.find("Test connection").unwrap();
    h.click(x + 1, y);
    assert!(h.text().contains("Connected in 42 ms"), "{}", h.text());
    assert!(last(&h).contains("Action(\"test\")"));
    // Save closes with the values; focus goes back to the button
    let (x, y) = h.find("Save").unwrap();
    h.click(x + 1, y);
    assert!(!page(&h).host.is_open());
    assert!(last(&h).contains("Save · billing"), "{}", last(&h));
    assert_eq!(h.app.shell.focus.current(), Some(page(&h).buttons[0].id));
}

#[test]
fn the_chooser_stacks_a_picker_over_the_form_and_returns_to_it() {
    let mut h = harness();
    open(&mut h, "Add connection…");
    let (x, y) = h.find("Connect through an SSH tunnel").unwrap();
    h.click(x, y);
    let (x, y) = h.find("Choose…").expect("the chooser shows");
    h.click(x + 1, y);
    assert_eq!(page(&h).host.layers.len(), 2, "the picker is stacked");
    assert!(page(&h).host.layers.top_as::<Picker>().is_some());
    assert_eq!(
        h.bars(),
        0,
        "the form under the picker shows no ▎; the picker marks its row with a fill"
    );
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(page(&h).host.layers.len(), 1, "back on the form");
    assert!(h.text().contains("~/.ssh/prod_rsa"), "{}", h.text());
    // Esc on a stacked picker closes only the picker
    let (x, y) = h.find("Choose…").unwrap();
    h.click(x + 1, y);
    h.key(KeyCode::Esc);
    assert_eq!(page(&h).host.layers.len(), 1);
    assert!(page(&h).host.layers.top_as::<FormDialog>().is_some());
    // a click outside the stacked picker lands on its backdrop: the host
    // hands it to the picker, which cancels under its policy
    let (x, y) = h.find("Choose…").unwrap();
    h.click(x + 1, y);
    assert_eq!(page(&h).host.layers.len(), 2);
    h.click(2, 20);
    assert_eq!(page(&h).host.layers.len(), 1, "{}", last(&h));
    assert!(last(&h).contains("Cancelled"), "{}", last(&h));
}

#[test]
fn keep_open_on_save_leaves_closing_to_the_page() {
    let mut h = harness();
    h.opt("dialogkinds.keep_open=on");
    open(&mut h, "Add connection…");
    assert!(!form(&h).pop_on_save);
    let (x, y) = h.find("Save").unwrap();
    h.click(x + 1, y);
    assert!(!page(&h).host.is_open(), "{}", last(&h));
    assert!(last(&h).contains("closed by the page"), "{}", last(&h));
}

#[test]
fn the_info_dialog_copies_a_row_and_stays_open() {
    let mut h = harness();
    open(&mut h, "Server info");
    assert!(h.text().contains("read-only"), "the meta");
    assert!(h.text().contains("wal_level = logical"), "the detail block");
    h.key(KeyCode::Char('y'));
    assert!(page(&h).host.is_open(), "copy keeps it open");
    assert!(
        last(&h).contains("Copy(\"db.eu-west-1.internal\")"),
        "{}",
        last(&h)
    );
    h.key(KeyCode::Esc);
    assert!(!page(&h).host.is_open());
    assert!(last(&h).contains("Info · Closed"), "{}", last(&h));
    // the ctrl key map replaces y
    h.opt("dialogkinds.info_keymap=ctrl");
    open(&mut h, "Server info");
    h.key(KeyCode::Tab);
    h.key_mod(
        KeyCode::Char('y'),
        ratatui::crossterm::event::KeyModifiers::CONTROL,
    );
    assert!(last(&h).contains("Copy"), "{}", last(&h));
}

/// A light-dismiss info dialog and help overlay close on a click outside
/// them: their frame registers its backdrop and the layer stack turns the
/// click into an outside-click dismissal (mouse.md, SC18 finding); a form
/// whose policy refuses it stays.
#[test]
fn a_click_outside_a_light_info_or_help_closes_it_and_a_form_stays() {
    let mut h = harness();
    for (label, closed) in [
        ("Server info", "Info · Closed"),
        ("Keyboard help", "Help · closed"),
    ] {
        open(&mut h, label);
        h.click(1, 2);
        assert!(!page(&h).host.is_open(), "{label}\n{}", h.text());
        assert!(last(&h).contains(closed), "{label}: {}", last(&h));
    }
    open(&mut h, "Add connection…");
    h.click(1, 2);
    assert!(page(&h).host.is_open(), "the form refuses an outside click");
}

#[test]
fn the_choice_returns_the_option_and_the_help_closes() {
    let mut h = harness();
    open(&mut h, "Release channel…");
    assert!(h.text().contains("Step 2 of 3"), "the stepper");
    h.key(KeyCode::Down);
    let (x, y) = h.find("Cancel   Next").expect("the buttons");
    h.click(x + 10, y);
    assert!(!page(&h).host.is_open(), "{}", h.text());
    assert_eq!(last(&h), "Choice · Some(1)");
    open(&mut h, "Keyboard help");
    assert!(h.text().contains("Next field"));
    h.key(KeyCode::Esc);
    assert_eq!(last(&h), "Help · closed");
}

#[test]
fn the_danger_close_button_reaches_every_kind() {
    let mut h = harness();
    h.opt("dialogkinds.close_button=danger");
    for label in [
        "Add connection…",
        "Server info",
        "Release channel…",
        "Keyboard help",
    ] {
        open(&mut h, label);
        assert!(h.text().contains('✕'), "{label}: the custom ×");
        h.key(KeyCode::Esc);
        assert!(!page(&h).host.is_open(), "{label}: Esc closes");
    }
}

#[test]
fn the_states_grid_draws_every_row() {
    let mut h = Harness::on(220, 80, PageId::DialogKinds);
    h.section(Section::States);
    let tpl = h.app.pages[PageId::DialogKinds.index()].template().unwrap();
    for s in [
        SpecimenState::Rest,
        SpecimenState::Hover,
        SpecimenState::Focused,
        SpecimenState::Selected,
        SpecimenState::Disabled,
    ] {
        assert!(tpl.state_rows().contains(&s), "{s:?}");
    }
    assert_eq!(h.bars(), 1, "the samples depict focus with ▏");
    assert!(h.text().contains("Release channel"));
}
