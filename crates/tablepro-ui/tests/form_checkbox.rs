//! WI-TABLEPRO-FORM-CHECKBOX: the connection-form prompt row is a stock `Checkbox`.
//!
//! The `Ask password` row on the form's Basic tab renders through the reusable
//! `termrock::Checkbox` (id `ASK_PASSWORD`), fed by `draft.ask_password`.
//! Frozen row-25 literals below are hardcoded from the `visual-baseline` tag
//! (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`); no test loads a frame file.
//!
//! * T1 pins the 100x30 row-25 symbols + per-cell styles exactly.
//! * T2 pins the untruncated wide rows (no ellipsis).
//! * T3 pins the below-fold small sizes (row absent).
//! * T4/T5 pin single-flip toggling in non-editing and editing modes
//!   (T5 kills headless double-dispatch: two flippers = no net change).
//! * T6 pins pointer toggling and the outside-area no-op.
//! * T7 pins Advanced-tab isolation via the corpus `e` path.
//! * T8 pins stock ownership: real-ring reachability + a controlled mutation.
//! * T9 pins the checked marker (disclosed non-frozen: no frozen frame shows
//!   the checked state; this asserts component-contract behavior).

use tablepro_ui::TableProApp;
use tablepro_ui::connections::field::ASK_PASSWORD;
use termrock::{Color, KeyCode, Theme};
use termrock_test_support::Harness;

const BG: Color = Color::Rgb(17, 17, 17);
const MUTED: Color = Color::Rgb(128, 128, 128);
const PRIMARY: Color = Color::Rgb(255, 255, 255);

fn open_new_form(w: u16, h: u16) -> Harness<TableProApp> {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
    let _ = t.ctrl('n');
    assert!(
        t.find("New connection").is_some(),
        "{w}x{h}: ctrl-n must open the new-connection form"
    );
    assert!(
        !t.app().is_editing(),
        "{w}x{h}: fresh form must not be editing"
    );
    t
}

fn ask(t: &Harness<TableProApp>) -> bool {
    t.app()
        .connection_draft()
        .expect("form must be open")
        .ask_password
}

/// Symbols of row `y` over `x0..=x1`, one `String` per cell joined.
fn row_span(t: &Harness<TableProApp>, y: u16, x0: u16, x1: u16) -> String {
    (x0..=x1).map(|x| t.cell(x, y).symbol()).collect()
}

/// T1: 100x30 row-25 parity — symbols, fg, bg, mods, all hardcoded.
#[test]
fn form_checkbox_parity_100() {
    let t = open_new_form(100, 30);
    assert_eq!(
        row_span(&t, 25, 37, 67),
        " [ ] Prompt for password on co…",
        "100x30 row-25 checkbox span must match frozen"
    );
    for x in 37..=67u16 {
        let cell = t.cell(x, 25);
        let want_fg = if x == 37 {
            BG
        } else if (38..=40).contains(&x) {
            MUTED
        } else {
            PRIMARY
        };
        assert_eq!(cell.fg, want_fg, "x={x} fg");
        assert_eq!(cell.bg, BG, "x={x} bg");
        assert_eq!(cell.modifier.bits(), 0, "x={x} mods");
    }
}

/// T2: wide rows render the full label with no ellipsis.
#[test]
fn form_checkbox_no_ellipsis_wide() {
    let t = open_new_form(120, 40);
    assert_eq!(
        row_span(&t, 25, 44, 78),
        " [ ] Prompt for password on connect",
        "120x40 row-25 must show the full label"
    );
    assert!(
        !row_span(&t, 25, 44, 84).contains('…'),
        "120x40 row-25 must carry no ellipsis"
    );
    assert_eq!(t.cell(45, 25).fg, MUTED, "120x40 marker fg");
    assert_eq!(t.cell(49, 25).fg, PRIMARY, "120x40 label fg");

    let t = open_new_form(160, 50);
    assert_eq!(
        row_span(&t, 25, 45, 79),
        " [ ] Prompt for password on connect",
        "160x50 row-25 must show the full label"
    );
    assert!(
        !row_span(&t, 25, 45, 85).contains('…'),
        "160x50 row-25 must carry no ellipsis"
    );
}

/// T3: small sizes clip the checkbox row below the fold.
#[test]
fn form_checkbox_clipped_small() {
    for (w, h) in [(72u16, 20u16), (80u16, 24u16)] {
        let t = open_new_form(w, h);
        assert!(
            t.find("Prompt for password").is_none(),
            "{w}x{h}: checkbox row must be below the fold"
        );
    }
}

/// T4: non-editing Space/Enter each flip exactly once (headless is skipped
/// here, so this guards direct-update liveness).
#[test]
fn form_checkbox_toggle_nonediting() {
    let mut t = open_new_form(100, 30);
    for _ in 0..12 {
        if t.focus() == Some(ASK_PASSWORD) {
            break;
        }
        let _ = t.key(KeyCode::Tab);
    }
    assert_eq!(
        t.focus(),
        Some(ASK_PASSWORD),
        "real Tab ring must reach the checkbox"
    );
    assert!(!ask(&t), "fresh draft must start unchecked");
    let _ = t.key(KeyCode::Char(' '));
    assert!(ask(&t), "first Space must check");
    let _ = t.key(KeyCode::Char(' '));
    assert!(!ask(&t), "second Space must uncheck");
    let _ = t.key(KeyCode::Enter);
    assert!(ask(&t), "Enter must check too");
}

/// T5: editing-mode Space alternates (the double-dispatch killer — a live
/// headless Check spec would flip twice per press = no net change).
#[test]
fn form_checkbox_toggle_editing() {
    let mut t = open_new_form(100, 30);
    let _ = t.key(KeyCode::Enter);
    assert!(t.app().is_editing(), "Enter on NAME must start editing");
    assert!(
        t.tab_to(ASK_PASSWORD),
        "explicit tab_to(ASK_PASSWORD) must succeed in editing mode"
    );
    assert!(!ask(&t), "fresh draft must start unchecked");
    for (press, want) in [(1, true), (2, false), (3, true)] {
        let _ = t.key(KeyCode::Char(' '));
        assert_eq!(ask(&t), want, "Space #{press} in editing mode");
    }
    assert!(t.app().is_editing(), "toggling must not leave editing");
}

/// T6: clicks on the marker and the label each flip once; a click past the
/// row's right edge is a no-op for the flag.
#[test]
fn form_checkbox_pointer() {
    let mut t = open_new_form(100, 30);
    let _ = t.click(39, 25);
    assert!(ask(&t), "marker click must check");
    let _ = t.click(50, 25);
    assert!(!ask(&t), "label click must uncheck");
    let _ = t.click(70, 25);
    assert!(!ask(&t), "click past the row must not flip");
}

/// T7: Advanced tab via the corpus `e` path shows no prompt row.
#[test]
fn form_checkbox_isolation_advanced() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    for _ in 0..7 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Char('e'));
    assert!(t.find("Name").is_some(), "e path must open the edit form");
    let _ = t.key(KeyCode::BackTab);
    let _ = t.key(KeyCode::Right);
    assert!(
        t.find("Startup").is_some(),
        "Advanced tab must show Startup commands"
    );
    assert!(
        t.find("Prompt for password").is_none(),
        "Advanced tab must not show the prompt row"
    );
}

/// T8: ownership — the row is painted by the registered stock control: the id
/// is reachable through the real Tab ring, and flipping the controlled flag
/// through the real journey repaints the marker while the label stays put.
#[test]
fn form_checkbox_ownership() {
    let mut t = open_new_form(100, 30);
    assert!(
        t.tab_to(ASK_PASSWORD),
        "ASK_PASSWORD must be reachable via the real Tab ring"
    );
    let before_label = row_span(&t, 25, 42, 67);
    assert_eq!(row_span(&t, 25, 38, 40), "[ ]", "marker starts off");
    let _ = t.key(KeyCode::Char(' '));
    assert!(ask(&t), "Space must flip the controlled flag");
    assert_eq!(
        row_span(&t, 25, 38, 40),
        "[✓]",
        "controlled mutation must repaint the marker"
    );
    assert_eq!(
        row_span(&t, 25, 42, 67),
        before_label,
        "label must be untouched by the flip"
    );
}

/// T9: checked marker render (disclosed non-frozen: no frozen frame shows the
/// checked state; styles must still follow the frozen semantic roles).
#[test]
fn form_checkbox_checked_render() {
    let mut t = open_new_form(100, 30);
    assert!(t.tab_to(ASK_PASSWORD), "must reach the checkbox");
    // Focused stock rows are bold (`row_like`, FOCUSED rule); the bit comes
    // from focus, not from the flag, so it is already set while unchecked.
    assert_eq!(
        t.cell(38, 25).modifier.bits(),
        1,
        "focused unchecked marker carries the stock focus bold"
    );
    let _ = t.key(KeyCode::Char(' '));
    assert!(ask(&t), "must be checked after one flip");
    assert_eq!(row_span(&t, 25, 38, 40), "[✓]", "checked marker glyphs");
    for x in 38..=40u16 {
        let cell = t.cell(x, 25);
        assert_eq!(cell.fg, MUTED, "x={x} checked marker fg");
        assert_eq!(cell.bg, BG, "x={x} checked marker bg");
        assert_eq!(
            cell.modifier.bits(),
            1,
            "x={x} checked marker keeps focus bold"
        );
    }
    assert_eq!(
        row_span(&t, 25, 42, 67),
        "Prompt for password on co…",
        "label must be untouched by the flip"
    );
}
