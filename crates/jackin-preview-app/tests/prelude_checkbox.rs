//! WI-JACKIN-PRELUDE-CHECKBOX: the prelude mount row is a stock `Checkbox`.
//!
//! The `Mount read-only` row on the new-workspace prelude (step 1) renders
//! through the reusable `termrock::Checkbox` (id `READ_ONLY`), fed by
//! `App.prelude_ui.read_only`. Frozen row literals below are hardcoded from
//! the `visual-baseline` tag (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`);
//! no test loads a frame file as expected output.
//!
//! * T1 pins the 120x40 row-26 symbols + per-cell styles exactly.
//! * T2 pins the mount row on all 5 sizes.
//! * T3 pins single-flip Space toggling through the focus-gated arm
//!   (the arm-flip killer: the capture path must flip, not advance).
//! * T4 pins gating: unfocused Space still `choose_source`s (step 1->2).
//! * T5 pins pointer toggling and the outside-area no-op.
//! * T6 pins the checked marker (disclosed non-frozen: no frozen frame shows
//!   the checked state; styles still follow the frozen semantic roles).
//! * T7 pins stock ownership: real-ring reachability, focus restyle through
//!   the live patches, controlled-mutation repaint, and the v1.json 25 cap.
use jackin_preview_app::{
    App, Motion, Scenario,
    screens::prelude::{FILE_LIST, READ_ONLY},
};
use termrock::{Color, KeyCode};
use termrock_test_support::Harness;

const ELEVATED: Color = Color::Rgb(24, 24, 27);
const MUTED: Color = Color::Rgb(128, 128, 128);
const PRIMARY: Color = Color::Rgb(255, 255, 255);

fn open_prelude(w: u16, h: u16) -> Harness<App> {
    let app = App::for_scenario_at(Scenario::FirstUse, Motion::Paused, 400);
    let mut t = Harness::new(app, termrock::Theme::junie(), w, h);
    let _ = t.key(KeyCode::Char('n'));
    for _ in 0..60 {
        let _ = t.tick();
        if t.text().contains("step 1 of 5") {
            break;
        }
    }
    assert!(
        t.text().contains("step 1 of 5"),
        "{w}x{h}: journey must reach the prelude needle"
    );
    t
}

/// Symbols of row `y` over `x0..=x1`, one `String` per cell joined.
fn row_span(t: &Harness<App>, y: u16, x0: u16, x1: u16) -> String {
    (x0..=x1).map(|x| t.cell(x, y).symbol()).collect()
}

/// A cell snapshot: symbol, fg, bg, modifier bits.
fn snap(t: &Harness<App>, x: u16, y: u16) -> (String, Color, Color, u16) {
    let c = t.cell(x, y);
    (c.symbol().to_string(), c.fg, c.bg, c.modifier.bits())
}

/// T1: 120x40 row-26 parity — symbols, fg, bg, mods, all hardcoded.
#[test]
fn prelude_checkbox_parity_120() {
    let t = open_prelude(120, 40);
    assert_eq!(
        row_span(&t, 26, 20, 42),
        " [ ] Mount read-only   ",
        "120x40 row-26 checkbox span must match frozen"
    );
    for x in 20..=42u16 {
        let cell = t.cell(x, 26);
        let want_fg = if x == 20 {
            ELEVATED
        } else if (21..=23).contains(&x) {
            MUTED
        } else {
            PRIMARY
        };
        assert_eq!(cell.fg, want_fg, "x={x} fg");
        assert_eq!(cell.bg, ELEVATED, "x={x} bg");
        assert_eq!(cell.modifier.bits(), 0, "x={x} mods");
    }
}

/// T2: the mount row renders on all 5 sizes at the frozen rows.
#[test]
fn prelude_checkbox_all_sizes() {
    for (w, h, y) in [
        (120u16, 40u16, 26u16),
        (100, 30, 21),
        (160, 50, 31),
        (80, 24, 17),
        (72, 20, 13),
    ] {
        let t = open_prelude(w, h);
        assert!(
            t.row(y).contains("[ ] Mount read-only"),
            "{w}x{h} row {y} must carry the mount row"
        );
    }
}

/// T3: Tab-to-focus then Space flips exactly once per press (arm-flip);
/// the step never advances and the label never moves.
#[test]
fn prelude_checkbox_tab_space_flip() {
    let mut t = open_prelude(120, 40);
    assert!(t.tab_to(READ_ONLY), "real Tab ring must reach READ_ONLY");
    assert!(
        !t.app().prelude_ui.read_only,
        "fresh prelude starts unchecked"
    );
    let label_before = row_span(&t, 26, 24, 42);
    let _ = t.key(KeyCode::Char(' '));
    assert!(t.app().prelude_ui.read_only, "first Space must check");
    assert_eq!(
        t.app().prelude.step(),
        1,
        "arm-flip must not advance the step"
    );
    assert_eq!(
        row_span(&t, 26, 21, 23),
        "[✓]",
        "marker must repaint checked"
    );
    assert_eq!(
        row_span(&t, 26, 24, 42),
        label_before,
        "label must be untouched by the flip"
    );
    let _ = t.key(KeyCode::Char(' '));
    assert!(!t.app().prelude_ui.read_only, "second Space must uncheck");
    assert_eq!(
        t.app().prelude.step(),
        1,
        "second flip must not advance the step"
    );
    assert_eq!(
        row_span(&t, 26, 21, 23),
        "[ ]",
        "marker must repaint unchecked"
    );
}

/// T4: Space with the checkbox unfocused keeps the old behavior —
/// `choose_source` advances step 1->2 and the flag stays false.
#[test]
fn prelude_checkbox_space_unfocused_advances() {
    let mut t = open_prelude(120, 40);
    assert_eq!(
        t.focus(),
        Some(FILE_LIST),
        "fresh prelude focuses the file list"
    );
    assert!(
        !t.app().prelude_ui.read_only,
        "fresh prelude starts unchecked"
    );
    let _ = t.key(KeyCode::Char(' '));
    assert_eq!(
        t.app().prelude.step(),
        2,
        "unfocused Space must choose_source (step 1->2)"
    );
    assert!(
        !t.app().prelude_ui.read_only,
        "unfocused Space must not flip the flag"
    );
}

/// T5: clicks on the marker and the label each flip once; a click past the
/// row's right edge is a no-op for the flag and the step.
#[test]
fn prelude_checkbox_pointer() {
    let mut t = open_prelude(120, 40);
    let _ = t.click(22, 26);
    assert!(t.app().prelude_ui.read_only, "marker click must check");
    assert_eq!(
        row_span(&t, 26, 21, 23),
        "[✓]",
        "marker must repaint checked"
    );
    let _ = t.click(30, 26);
    assert!(!t.app().prelude_ui.read_only, "label click must uncheck");
    let _ = t.click(100, 26);
    assert!(
        !t.app().prelude_ui.read_only,
        "click past the row must not flip"
    );
    assert_eq!(
        t.app().prelude.step(),
        1,
        "outside click must not advance the step"
    );
}

/// T6: checked marker render (disclosed non-frozen: no frozen frame shows the
/// checked state; styles must still follow the frozen semantic roles).
#[test]
fn prelude_checkbox_checked_render() {
    let mut t = open_prelude(120, 40);
    assert!(t.tab_to(READ_ONLY), "must reach the checkbox");
    let _ = t.key(KeyCode::Char(' '));
    assert!(
        t.app().prelude_ui.read_only,
        "must be checked after one flip"
    );
    assert_eq!(row_span(&t, 26, 21, 23), "[✓]", "checked marker glyphs");
    for x in 21..=23u16 {
        let cell = t.cell(x, 26);
        assert_eq!(cell.fg, MUTED, "x={x} checked marker fg");
        assert_eq!(cell.bg, ELEVATED, "x={x} checked marker bg");
    }
    assert_eq!(
        row_span(&t, 26, 24, 42),
        " Mount read-only   ",
        "label must be untouched by the flip"
    );
}

/// T7: ownership — the row is painted by the registered stock control: the id
/// is reachable through the real Tab ring, focusing it restyles the row
/// through the live patches (impossible on a stored painter), the controlled
/// flag repaints the marker, and the v1.json OWN-02 cap sits at 25.
#[test]
fn prelude_checkbox_ownership() {
    let mut t = open_prelude(120, 40);
    assert_ne!(
        t.focus(),
        Some(READ_ONLY),
        "fresh prelude must not focus the checkbox"
    );
    let before: Vec<_> = (20..=42u16).map(|x| snap(&t, x, 26)).collect();
    assert!(
        t.tab_to(READ_ONLY),
        "READ_ONLY must be reachable via the real Tab ring"
    );
    let focused: Vec<_> = (20..=42u16).map(|x| snap(&t, x, 26)).collect();
    assert_ne!(before, focused, "stock focus restyle must move row output");
    let _ = t.key(KeyCode::Char(' '));
    assert!(
        t.app().prelude_ui.read_only,
        "Space must flip the controlled flag"
    );
    assert_eq!(
        row_span(&t, 26, 21, 23),
        "[✓]",
        "controlled mutation must repaint the marker"
    );
    assert_eq!(
        prelude_own02_max(),
        25,
        "v1.json OWN-02 prelude.rs cap must be ratcheted to 25"
    );
}

/// The OWN-02 `max` for prelude.rs, read as plain text (no serde here).
fn prelude_own02_max() -> u64 {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/termrock-xtask/exceptions/v1.json");
    let text = std::fs::read_to_string(&root).expect("v1.json must be readable");
    for chunk in text.split('{') {
        if chunk.contains("crates/jackin-preview-host-ui/src/prelude.rs")
            && chunk.contains("\"OWN-02\"")
        {
            let key = "\"max\":";
            let at = chunk
                .find(key)
                .expect("OWN-02 prelude entry must carry max");
            let rest = chunk[at + key.len()..].trim_start();
            let end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            return rest[..end].parse().expect("max must parse");
        }
    }
    panic!("OWN-02 prelude.rs entry not found in v1.json");
}
