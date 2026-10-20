//! End-to-end contract for the Selection page: real selection state, the
//! shared bulk-edit summary, and the delete-confirmation dialog opened in
//! the overlay layer through the real shell.

use showcase_ui::App;
use termrock::{KeyCode, Theme};
use termrock_test_support::Harness;

fn selection_harness() -> Harness<App> {
    let mut h = Harness::new(App::for_page_name("selectable"), Theme::junie(), 120, 40);
    // The scenario path: two tabs move focus from the nav into the table.
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Tab);
    h
}

fn full_text(h: &Harness<App>) -> String {
    let area = h.buffer().area();
    (area.y..area.bottom())
        .map(|y| {
            (0..area.width)
                .map(|x| h.cell(x, y).symbol().to_owned())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn footer_text(h: &Harness<App>) -> String {
    let height = h.buffer().area().height;
    (0..h.buffer().area().width)
        .map(|x| h.cell(x, height.saturating_sub(1)).symbol().to_owned())
        .collect()
}

#[test]
fn page_renders_the_member_table_and_details_card() {
    let h = selection_harness();
    let text = full_text(&h);
    assert!(text.contains("Dana Whitfield"), "table lists members");
    assert!(text.contains("Details"), "empty-selection editor card");
    assert!(text.contains("No member selected"), "editor hint");
}

#[test]
fn footer_hints_follow_the_selection() {
    let mut h = selection_harness();
    assert!(
        !footer_text(&h).contains("Clear selection"),
        "no selection hints before anything is picked"
    );
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    assert!(
        footer_text(&h).contains("Toggle row"),
        "selection hints once a member is picked"
    );
}

#[test]
fn selecting_a_member_renders_the_real_selection_and_summary() {
    let mut h = selection_harness();
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    let text = full_text(&h);
    assert!(text.contains("[✓]"), "checked row shows the on mark");
    assert!(text.contains("Edit 1 member"), "editor tracks the count");
    assert!(
        !text.contains("No member selected"),
        "details card is replaced by the bulk editor"
    );
    // Esc clears the selection and the editor returns to the empty state.
    let _ = h.key(KeyCode::Esc);
    let text = full_text(&h);
    assert!(!text.contains("[✓]"), "check marks clear");
    assert!(text.contains("No member selected"), "editor hint returns");
}

#[test]
fn delete_opens_the_shared_dialog_and_cancel_keeps_the_selection() {
    let mut h = selection_harness();
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    let _ = h.key(KeyCode::Char('d'));
    let text = full_text(&h);
    assert!(
        text.contains("Delete 1 member?"),
        "dialog title names the count"
    );
    assert!(text.contains("Cancel"), "dialog offers cancel");
    assert!(text.contains("Delete 1 member"), "dialog names the action");
    // Cancel (the focused action) resolves and keeps the selection.
    let _ = h.key(KeyCode::Enter);
    let text = full_text(&h);
    assert!(
        !text.contains("Delete 1 member?"),
        "dialog closes on cancel"
    );
    assert!(text.contains("[✓]"), "cancel keeps the selection");
}

#[test]
fn delete_dialog_esc_keeps_the_selection() {
    let mut h = selection_harness();
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    let _ = h.key(KeyCode::Char('d'));
    assert!(full_text(&h).contains("Delete 1 member?"));
    let _ = h.key(KeyCode::Esc);
    let text = full_text(&h);
    assert!(!text.contains("Delete 1 member?"), "esc dismisses");
    assert!(text.contains("[✓]"), "dismissal keeps the selection");
}

#[test]
fn confirming_the_delete_clears_the_selection() {
    let mut h = selection_harness();
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    let _ = h.key(KeyCode::Char('d'));
    // Move to the danger action and confirm.
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    let text = full_text(&h);
    assert!(!text.contains("Delete 1 member?"), "dialog closes");
    assert!(!text.contains("[✓]"), "confirm clears the selection");
    assert!(text.contains("No member selected"), "editor is empty again");
}

#[test]
fn bulk_selection_reports_mixed_fields_in_the_editor() {
    let mut h = selection_harness();
    // Select Ari, Cleo and Bo: roles differ, so the editor shows mixed.
    for _ in 0..3 {
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Char(' '));
        let _ = h.key(KeyCode::Down);
    }
    let text = full_text(&h);
    assert!(text.contains("Edit 3 members"), "editor names the count");
    assert!(
        text.contains("\u{2014} mixed \u{2014}"),
        "conflicting roles render the mixed marker"
    );
}

#[test]
fn dialog_layer_covers_the_footer_with_dialog_hints() {
    let mut h = selection_harness();
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    let _ = h.key(KeyCode::Char('d'));
    let footer = footer_text(&h);
    assert!(
        footer.contains("Esc") && footer.contains("Cancel"),
        "footer switches to the dialog hint set while open"
    );
}

#[test]
fn delete_on_an_empty_selection_does_not_open_the_dialog() {
    let mut h = selection_harness();
    let _ = h.key(KeyCode::Char('d'));
    assert!(
        !full_text(&h).contains("Delete 0 members?"),
        "no dialog without a selection"
    );
}

#[test]
fn narrow_terminal_keeps_the_page_rendering() {
    let mut h = Harness::new(App::for_page_name("selectable"), Theme::junie(), 80, 24);
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Tab);
    let text = full_text(&h);
    assert!(text.contains("Dana Whitfield"), "table renders at 80x24");
    assert!(text.contains("Members"), "members card renders at 80x24");
}
