//! WI-TABLEPRO-FORM-TOGGLE: the connection-form SSL/SSH/local-only rows are
//! stock `Toggle` (ids `SSL`/`SSH`/`LOCAL_ONLY`), fed per-frame from
//! `draft.ssl`/`draft.ssh` (`LOCAL_ONLY` hardcoded off, draw-only).
//!
//! Coord convention (all tests): rows are 1-based oracle lines, cols are
//! 0-based harness x; harness y = row-1. Geometry pinned against the approved
//! `form-advanced/{120x40,80x24,100x30,160x50,72x20}/truecolor.txt` frames;
//! no test loads a frame file.
//!
//! * T1/T2/T3 pin the 120x40 SSL/SSH/local-only geometry (switch, label, word).
//! * T4 pins the 80x24 on-states via the corpus `e` path.
//! * T5 pins the 73x21 word suppression (the L2 tripwire: without the guard
//!   stock paints `off`).
//! * T6 pins the unfocused style recipe incl. the card-bg rows (the B1
//!   tripwire: without the bg patches the rows render #000) and the `○`
//!   off knob (the L1 tripwire).
//! * T7 pins the accepted D1 delta: the focused toggle row is BOLD with an
//!   invisible gutter (legacy rendered no toggle focus).
//! * T8 pins deletion of the legacy `draw_form_toggle` painter.
//! * T9 pins the accepted D2/D3 deltas: SSL/SSH/LOCAL_ONLY registered as
//!   ring stops in draw order; Space flips `draft.ssl/ssh` and repaints the
//!   marker, while LOCAL_ONLY flips nothing.
//! * T10 pins the 72x20 on-states and the below-fold local-only row.
//!
//! T1–T6/T10 are green-on-base guards; T7/T8/T9 are red-on-base discriminators.

use tablepro_ui::TableProApp;
use tablepro_ui::connections::field::{LOCAL_ONLY, SSH, SSL, TABS};
use termrock::{Color, KeyCode, Theme};
use termrock_test_support::Harness;

const CARD: Color = Color::Rgb(17, 17, 17);
const MUTED: Color = Color::Rgb(128, 128, 128);
const PRIMARY: Color = Color::Rgb(255, 255, 255);
const ACCENT: Color = Color::Rgb(72, 224, 84);

fn open_new_form(w: u16, h: u16) -> Harness<TableProApp> {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
    let _ = t.ctrl('n');
    assert!(
        t.find("New connection").is_some(),
        "{w}x{h}: ctrl-n must open the new-connection form"
    );
    t
}

/// Fresh form on the Advanced tab via ctrl-n + BackTab + Right.
fn open_advanced_new(w: u16, h: u16) -> Harness<TableProApp> {
    let mut t = open_new_form(w, h);
    let _ = t.key(KeyCode::BackTab);
    let _ = t.key(KeyCode::Right);
    assert!(
        t.find("Startup").is_some(),
        "{w}x{h}: BackTab + Right must reach the Advanced tab"
    );
    t
}

/// Edit form on the Advanced tab via the corpus `e` path (T7 keys).
fn open_advanced_edit(w: u16, h: u16) -> Harness<TableProApp> {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
    for _ in 0..7 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Char('e'));
    assert!(t.find("Name").is_some(), "e path must open the edit form");
    let _ = t.key(KeyCode::BackTab);
    let _ = t.key(KeyCode::Right);
    assert!(
        t.find("Startup").is_some(),
        "{w}x{h}: BackTab + Right must reach the Advanced tab"
    );
    t
}

fn flags(t: &Harness<TableProApp>) -> (bool, bool) {
    let draft = t.app().connection_draft().expect("form must be open");
    (draft.ssl, draft.ssh)
}

/// Symbols of row `y` over `x0..=x1`, one `String` per cell joined.
fn row_span(t: &Harness<TableProApp>, y: u16, x0: u16, x1: u16) -> String {
    (x0..=x1).map(|x| t.cell(x, y).symbol()).collect()
}

/// T1: 120x40 SSL geometry — `○──` line 8 (y=7) x=45-47, label x=49,
/// `off` x=63.
#[test]
fn form_toggle_ssl_geometry_120() {
    let t = open_advanced_new(120, 40);
    assert_eq!(row_span(&t, 7, 45, 47), "○──", "ssl switch");
    assert_eq!(row_span(&t, 7, 49, 61), "Use SSL / TLS", "ssl label");
    assert_eq!(row_span(&t, 7, 63, 65), "off", "ssl word");
}

/// T2: 120x40 SSH geometry — `○──` line 10 (y=9) x=45, label x=49,
/// `off` x=60.
#[test]
fn form_toggle_ssh_geometry_120() {
    let t = open_advanced_new(120, 40);
    assert_eq!(row_span(&t, 9, 45, 47), "○──", "ssh switch");
    assert_eq!(row_span(&t, 9, 49, 58), "SSH tunnel", "ssh label");
    assert_eq!(row_span(&t, 9, 60, 62), "off", "ssh word");
}

/// T3: 120x40 local-only geometry — `○──` line 17 (y=16) x=45, label x=49,
/// `off` x=77; always off.
#[test]
fn form_toggle_local_only_120() {
    let t = open_advanced_new(120, 40);
    assert_eq!(row_span(&t, 16, 45, 47), "○──", "local-only switch");
    assert_eq!(
        row_span(&t, 16, 49, 75),
        "Local only (no iCloud sync)",
        "local-only label"
    );
    assert_eq!(row_span(&t, 16, 77, 79), "off", "local-only word");
}

/// T4: 80x24 on-states via the corpus `e` path — `──●` line 8 (y=7) x=4
/// with `on` x=22, `──●` line 10 (y=9) x=4 with `on` x=19; on-switch
/// cells Accent on card.
#[test]
fn form_toggle_on_states_80x24() {
    let t = open_advanced_edit(80, 24);
    assert_eq!(flags(&t), (true, true), "e-path draft must be on/on");
    assert_eq!(row_span(&t, 7, 4, 6), "──●", "ssl switch on");
    assert_eq!(row_span(&t, 7, 22, 23), "on", "ssl word on");
    assert_eq!(row_span(&t, 9, 4, 6), "──●", "ssh switch on");
    assert_eq!(row_span(&t, 9, 19, 20), "on", "ssh word on");
    for (x, y) in [(4, 7), (5, 7), (6, 7), (4, 9), (5, 9), (6, 9)] {
        let cell = t.cell(x, y);
        assert_eq!(cell.fg, ACCENT, "on-switch ({x},{y}) fg");
        assert_eq!(cell.bg, CARD, "on-switch ({x},{y}) bg");
    }
}

/// T5: 73x21 word suppression (lc.w=36) — line 17 (y=16) shows the full
/// 27-char label and blank card cells where the word would sit. L2
/// tripwire: without the guard stock paints `off`.
#[test]
fn form_toggle_word_suppressed_73x21() {
    let t = open_advanced_new(73, 21);
    let label = "Local only (no iCloud sync)";
    let (lx, ly) = t
        .find(label)
        .expect("73x21 line 17 must show the full label");
    assert_eq!(ly, 16, "local-only row must be oracle line 17");
    assert_eq!(row_span(&t, ly, lx - 4, lx - 2), "○──", "switch");
    for x in lx + 27..=lx + 29 {
        let cell = t.cell(x, ly);
        assert_eq!(cell.symbol(), " ", "word-zone x={x} must be blank");
        assert_eq!(cell.bg, CARD, "word-zone x={x} bg");
    }
    assert!(
        !row_span(&t, ly, lx, lx + 29).contains("off"),
        "73x21 local-only row must carry no word"
    );
}

/// T6: unfocused recipe — off-switch Muted on card, label Primary on card,
/// word Muted on card, gutter blank card, row fill card. B1 tripwire (drop
/// any bg patch and the row renders #000) + L1 tripwire (`○` at x=45).
#[test]
fn form_toggle_unfocused_recipe() {
    let t = open_advanced_new(120, 40);
    assert_eq!(t.cell(45, 7).symbol(), "○", "L1: off knob must be ○");
    // SSL switch: Muted on card, no mods.
    for x in 45..=47u16 {
        let cell = t.cell(x, 7);
        assert_eq!(cell.fg, MUTED, "ssl switch x={x} fg");
        assert_eq!(cell.bg, CARD, "ssl switch x={x} bg");
        assert_eq!(cell.modifier.bits(), 0, "ssl switch x={x} mods");
    }
    // SSL label: Primary on card, no mods.
    for x in 49..=61u16 {
        let cell = t.cell(x, 7);
        assert_eq!(cell.fg, PRIMARY, "ssl label x={x} fg");
        assert_eq!(cell.bg, CARD, "ssl label x={x} bg");
        assert_eq!(cell.modifier.bits(), 0, "ssl label x={x} mods");
    }
    // SSL word: Muted on card, no mods.
    for x in 63..=65u16 {
        let cell = t.cell(x, 7);
        assert_eq!(cell.fg, MUTED, "ssl word x={x} fg");
        assert_eq!(cell.bg, CARD, "ssl word x={x} bg");
        assert_eq!(cell.modifier.bits(), 0, "ssl word x={x} mods");
    }
    // Gutter blank + row fill (B1 tripwire).
    let gutter = t.cell(44, 7);
    assert_eq!(gutter.symbol(), " ", "gutter symbol");
    assert_eq!(gutter.fg, CARD, "gutter fg");
    assert_eq!(gutter.bg, CARD, "gutter bg");
    for x in [48u16, 62, 70, 80] {
        let cell = t.cell(x, 7);
        assert_eq!(cell.symbol(), " ", "fill x={x} symbol");
        assert_eq!(cell.bg, CARD, "fill x={x} bg");
    }
    // SSH site, same recipe.
    assert_eq!(row_span(&t, 9, 45, 47), "○──", "ssh switch");
    for x in 45..=47u16 {
        assert_eq!(t.cell(x, 9).fg, MUTED, "ssh switch x={x} fg");
        assert_eq!(t.cell(x, 9).bg, CARD, "ssh switch x={x} bg");
    }
    for x in 49..=58u16 {
        assert_eq!(t.cell(x, 9).fg, PRIMARY, "ssh label x={x} fg");
        assert_eq!(t.cell(x, 9).bg, CARD, "ssh label x={x} bg");
    }
    for x in 60..=62u16 {
        assert_eq!(t.cell(x, 9).fg, MUTED, "ssh word x={x} fg");
        assert_eq!(t.cell(x, 9).bg, CARD, "ssh word x={x} bg");
    }
}

/// T7: accepted D1 delta — the focused SSL row is BOLD with an invisible
/// gutter (FocusBar glyph in Surface on Surface). Red on base (legacy
/// rendered no toggle focus).
#[test]
fn form_toggle_focused_delta() {
    let mut t = open_advanced_new(120, 40);
    assert!(t.tab_to(SSL), "SSL must be Tab-reachable");
    assert_eq!(t.focus(), Some(SSL), "focus must sit on SSL");
    for x in 45..=47u16 {
        let cell = t.cell(x, 7);
        assert_eq!(cell.fg, MUTED, "focused switch x={x} fg");
        assert_eq!(cell.bg, CARD, "focused switch x={x} bg");
        assert_eq!(cell.modifier.bits(), 1, "focused switch x={x} bold");
    }
    for x in 49..=61u16 {
        let cell = t.cell(x, 7);
        assert_eq!(cell.fg, PRIMARY, "focused label x={x} fg");
        assert_eq!(cell.bg, CARD, "focused label x={x} bg");
        assert_eq!(cell.modifier.bits(), 1, "focused label x={x} bold");
    }
    for x in 63..=65u16 {
        let cell = t.cell(x, 7);
        assert_eq!(cell.fg, MUTED, "focused word x={x} fg");
        assert_eq!(cell.bg, CARD, "focused word x={x} bg");
        assert_eq!(cell.modifier.bits(), 1, "focused word x={x} bold");
    }
    let gutter = t.cell(44, 7);
    assert_eq!(gutter.symbol(), "▎", "focus bar glyph");
    assert_eq!(gutter.fg, CARD, "focus bar fg invisible");
    assert_eq!(gutter.bg, CARD, "focus bar bg");
}

/// T8: no legacy painter — `draw_form_toggle` is deleted, all three call
/// sites gone. Red on base.
#[test]
fn form_toggle_no_legacy_painter() {
    let src = include_str!("../src/app.rs");
    assert!(
        !src.contains("fn draw_form_toggle"),
        "legacy draw_form_toggle definition survived"
    );
    assert!(
        !src.contains("draw_form_toggle("),
        "legacy draw_form_toggle call survived"
    );
}

/// T9: accepted D2/D3 deltas — SSL/SSH/LOCAL_ONLY registered as enabled
/// ring stops in draw order; Space on SSL/SSH flips `draft.ssl/ssh` and
/// repaints the marker, while Space/click on LOCAL_ONLY flips nothing.
/// Red on base (legacy registered nothing). Editing mode first: the
/// existing form update only runs while editing (`app.rs`, untouched).
#[test]
fn form_toggle_ring_stops() {
    let mut t = open_new_form(120, 40);
    let _ = t.key(KeyCode::Enter);
    assert!(t.app().is_editing(), "Enter on NAME must start editing");
    assert!(t.tab_to(TABS), "TABS must be Tab-reachable");
    let _ = t.key(KeyCode::Right);
    assert!(
        t.find("Startup").is_some(),
        "Right on TABS must reach the Advanced tab"
    );
    assert!(t.app().is_editing(), "tab switch must not leave editing");
    let entries = t.ring().entries();
    let ssl = entries
        .iter()
        .position(|e| e.id == SSL)
        .expect("SSL must be registered");
    let ssh = entries
        .iter()
        .position(|e| e.id == SSH)
        .expect("SSH must be registered");
    let local = entries
        .iter()
        .position(|e| e.id == LOCAL_ONLY)
        .expect("LOCAL_ONLY must be registered");
    assert!(!entries[ssl].disabled, "SSL stop must be enabled");
    assert!(!entries[ssh].disabled, "SSH stop must be enabled");
    assert!(!entries[local].disabled, "LOCAL_ONLY stop must be enabled");
    assert!(
        ssl < ssh && ssh < local,
        "stops must register in draw order SSL < SSH < LOCAL_ONLY"
    );

    assert!(t.tab_to(SSL), "SSL must be Tab-reachable");
    let before_label = row_span(&t, 7, 49, 61);
    assert_eq!(row_span(&t, 7, 45, 47), "○──", "marker starts off");
    assert!(!flags(&t).0, "fresh draft must start with ssl off");
    let _ = t.key(KeyCode::Char(' '));
    assert!(flags(&t).0, "Space must flip draft.ssl");
    assert_eq!(
        row_span(&t, 7, 45, 47),
        "──●",
        "controlled mutation must repaint the marker"
    );
    assert_eq!(t.cell(45, 7).fg, ACCENT, "flipped marker fg");
    assert_eq!(
        row_span(&t, 7, 49, 61),
        before_label,
        "label must be untouched by the flip"
    );

    assert!(t.tab_to(SSH), "SSH must be Tab-reachable");
    assert!(!flags(&t).1, "fresh draft must start with ssh off");
    let _ = t.key(KeyCode::Char(' '));
    assert!(flags(&t).1, "Space must flip draft.ssh");
    assert_eq!(
        row_span(&t, 9, 45, 47),
        "──●",
        "controlled mutation must repaint the marker"
    );

    // D3: the dead stop — reachable, but Space/click flip nothing.
    assert!(t.tab_to(LOCAL_ONLY), "LOCAL_ONLY must be Tab-reachable");
    assert_eq!(t.focus(), Some(LOCAL_ONLY), "focus must sit on LOCAL_ONLY");
    let before = flags(&t);
    let _ = t.key(KeyCode::Char(' '));
    assert_eq!(flags(&t), before, "Space on LOCAL_ONLY must flip nothing");
    let _ = t.key(KeyCode::Enter);
    assert_eq!(flags(&t), before, "Enter on LOCAL_ONLY must flip nothing");
    let _ = t.click(46, 16);
    assert_eq!(flags(&t), before, "click on LOCAL_ONLY must flip nothing");
    assert_eq!(
        row_span(&t, 16, 45, 47),
        "○──",
        "LOCAL_ONLY marker must stay off"
    );
}

/// T10: 72x20 via the corpus `e` path — `──●` line 8 (y=7) x=4 and line
/// 10 (y=9) x=4; no `Local only` text anywhere (below-fold guard).
#[test]
fn form_toggle_72x20_no_local() {
    let t = open_advanced_edit(72, 20);
    assert_eq!(row_span(&t, 7, 4, 6), "──●", "ssl switch on");
    assert_eq!(row_span(&t, 9, 4, 6), "──●", "ssh switch on");
    assert!(
        t.find("Local only").is_none(),
        "72x20: local-only row must be below the fold"
    );
}
