//! The Dialogs page (plan §7.5, SC17), driven through the real app: each
//! variant opens and returns its result, the two gates chain, the options
//! reach the dialog through its builders, and the States grid.

use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::pages::dialogs::DialogsPage;
use crate::pages::template::Section;
use junie_tui::ui::specimen::SpecimenState;
use junie_tui::widgets::dialog::DialogBody;

fn page(h: &Harness) -> &DialogsPage {
    h.app.pages[PageId::Dialogs.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<DialogsPage>())
        .expect("the Dialogs page")
}

fn harness() -> Harness {
    Harness::on(120, 40, PageId::Dialogs)
}

/// Click the variant button labelled `label`.
fn open(h: &mut Harness, label: &str) {
    let (x, y) = h.find(label).unwrap_or_else(|| panic!("{label} button"));
    h.click(x + 1, y);
    assert!(h.app.dialog().is_some(), "{label} opens a dialog");
}

#[test]
fn every_variant_opens_and_esc_resolves_to_its_cancel_action() {
    let mut h = harness();
    for label in [
        "Confirm run",
        "Rename task…",
        "Unsaved changes…",
        "Delete branch…",
        "View config",
        "Drop table…",
        "Push failed…",
        "Release notes",
    ] {
        open(&mut h, label);
        assert_eq!(h.bars(), 1, "{label}: one ▎, inside the dialog");
        h.key(KeyCode::Esc);
        assert!(h.app.dialog().is_none(), "{label}: Esc closes");
    }
    assert_eq!(page(&h).history.len(), 8, "every dialog returned a result");
    assert!(
        page(&h).history[0].contains("Action(0)"),
        "{:?}",
        page(&h).history
    );
}

#[test]
fn the_drop_gates_chain_and_the_token_arms_the_action() {
    let mut h = harness();
    open(&mut h, "Drop table…");
    assert!(h.text().contains("1 of 2"));
    assert!(
        h.text().contains("DROP TABLE public.orders"),
        "the code block"
    );
    // Cancel is focused first: → reaches Continue
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    let d = h.app.dialog().expect("the second gate opens");
    assert!(d.title.contains("2 of 2"));
    assert!(!d.armed(), "the action is gated until the token matches");
    for c in "orders".chars() {
        h.key(KeyCode::Char(c));
    }
    h.key(KeyCode::Enter);
    assert!(
        h.app.dialog().is_none(),
        "Enter submits once the token matches"
    );
    let last = page(&h).history.last().unwrap();
    assert!(last.contains("Table public.orders dropped"), "{last}");
}

#[test]
fn ack_enter_next_field_moves_to_the_buttons_instead_of_submitting() {
    let mut h = harness();
    h.opt("dialogs.ack_enter=next-field");
    open(&mut h, "Drop table…");
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    for c in "orders".chars() {
        h.key(KeyCode::Char(c));
    }
    h.key(KeyCode::Enter);
    let d = h.app.dialog().expect("still open: Enter moved on");
    assert!(d.armed());
}

#[test]
fn placement_below_opens_the_dialog_under_the_button_that_asked() {
    let mut h = harness();
    h.opt("dialogs.placement=below");
    open(&mut h, "Confirm run");
    let button = page(&h).buttons[0].area;
    let d = h.app.dialog().unwrap();
    assert_eq!(d.area.y, button.bottom(), "right under the button");
    // centred, it sits elsewhere
    h.key(KeyCode::Esc);
    h.opt("dialogs.placement=center");
    open(&mut h, "Confirm run");
    assert_ne!(h.app.dialog().unwrap().area.y, button.bottom());
}

#[test]
fn chrome_rows_keep_a_tall_dialog_clear_of_the_header_and_footer() {
    let mut h = Harness::on(120, 30, PageId::Dialogs);
    open(&mut h, "Release notes");
    let base = h.app.dialog().unwrap().area;
    h.key(KeyCode::Esc);
    h.opt("dialogs.chrome=3-2");
    open(&mut h, "Release notes");
    let d = h.app.dialog().unwrap().area;
    // the shell honours the dialog's own chrome: 3 rows above, 2 below,
    // then the outer margin
    assert_eq!(d.y, 3 + 1, "{d:?}");
    assert!(d.bottom() < 30 - 2, "{d:?}");
    assert!(d.height < base.height, "{d:?} vs {base:?}");
}

#[test]
fn options_reach_the_dialog_through_its_builders() {
    let mut h = harness();
    h.opt("dialogs.width=44");
    h.opt("dialogs.max_height=9");
    h.opt("dialogs.dismiss=never");
    h.opt("dialogs.cancel_label=keep");
    h.opt("dialogs.edit_entry=explicit");
    open(&mut h, "Rename task…");
    let d = h.app.dialog().unwrap();
    assert_eq!(d.width, 44);
    assert_eq!(d.max_height, Some(9));
    assert!(!d.dismiss.escape && !d.dismiss.outside_click);
    assert_eq!(d.actions[0].label, "Keep");
    match &d.body {
        DialogBody::Input(i) => assert!(!i.editing, "explicit: focus does not start the edit"),
        _ => panic!("a prompt"),
    }
    h.key(KeyCode::Esc);
    assert!(h.app.dialog().is_some(), "never: Esc does not close");
}

#[test]
fn the_ctrl_key_map_confirms_and_cancels() {
    let mut h = harness();
    h.opt("dialogs.keymap=ctrl");
    open(&mut h, "Confirm run");
    h.key_mod(
        KeyCode::Char('y'),
        ratatui::crossterm::event::KeyModifiers::CONTROL,
    );
    assert!(h.app.dialog().is_none());
    assert!(page(&h).history.last().unwrap().contains("Task started"));
}

#[test]
fn the_states_grid_draws_every_row_with_no_second_focus_bar() {
    let mut h = Harness::on(220, 80, PageId::Dialogs);
    h.section(Section::States);
    let tpl = h.app.pages[PageId::Dialogs.index()].template().unwrap();
    for s in [
        SpecimenState::Rest,
        SpecimenState::Hover,
        SpecimenState::Focused,
        SpecimenState::Disabled,
        SpecimenState::Editing,
        SpecimenState::Overflow,
    ] {
        assert!(tpl.state_rows().contains(&s), "{s:?}");
    }
    assert_eq!(h.bars(), 1, "the samples depict focus with ▏");
    assert!(h.text().contains("Run task now?"));
    assert!(h.text().contains("Drop table?"));
}
