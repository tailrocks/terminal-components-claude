//! WI-TABLEPRO-FOOTER: the bottom row is one stock `HintBar` (id `FOOTER`)
//! fed per-frame by `footer_layer` data plus `.status_text()`.
//!
//! Coord convention (all tests): rows are 1-based oracle lines, cols are
//! 0-based harness x; footer harness y = h-1. Geometry pinned against the
//! approved `connections/*`, `table/*`, and `workbench/*` frames; no test
//! loads a frame file.
//!
//! * T1/T2 pin the 120x40 full row and the 72x20 truncation incl the `…`.
//! * T3 pins the right status after `connect(4)` (Production is index 4).
//! * T4 pins the ` EDIT ` badge in the connections filter state.
//! * T5 pins the full-row Canvas fill with the form open (D2 tripwire).
//! * T6 pins the style recipe incl the zero-patch proof (B1 tripwire).
//! * T7 pins deletion of the legacy row painter (red on base).
//! * T8 pins ring and click silence (the §4 guard, not hidden absence).
//! * T11 pins the Connections destructive suppression (D3, load-bearing).
//! * T12 pins the cell-editing row: badge + hints + status at once.
//!
//! T9/T10/T13 live as src unit tests (privates) in `app.rs`.

use tablepro_ui::TableProApp;
use tablepro_ui::app::FOOTER;
use termrock::{Color, Id, KeyCode, Theme};
use termrock_test_support::Harness;

const BLACK: Color = Color::Rgb(0, 0, 0);
const MUTED: Color = Color::Rgb(128, 128, 128);
const PRIMARY: Color = Color::Rgb(255, 255, 255);
const SECONDARY: Color = Color::Rgb(179, 179, 179);
const FAINT: Color = Color::Rgb(77, 77, 77);
const ACCENT: Color = Color::Rgb(72, 224, 84);
const ON_ACCENT: Color = Color::Rgb(25, 25, 28);
const BOLD: u16 = 0x0001;

/// Symbols of row `y` over `x0..=x1`, one `String` per cell joined.
fn row_span(t: &Harness<TableProApp>, y: u16, x0: u16, x1: u16) -> String {
    (x0..=x1).map(|x| t.cell(x, y).symbol()).collect()
}

/// Whole row `y` as text.
fn row_text(t: &Harness<TableProApp>, y: u16, w: u16) -> String {
    row_span(t, y, 0, w - 1)
}

/// Fresh form on the Advanced tab via ctrl-n + BackTab + Right.
fn open_advanced_new(w: u16, h: u16) -> Harness<TableProApp> {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
    let _ = t.ctrl('n');
    assert!(
        t.find("New connection").is_some(),
        "{w}x{h}: ctrl-n must open the new-connection form"
    );
    let _ = t.key(KeyCode::BackTab);
    let _ = t.key(KeyCode::Right);
    assert!(
        t.find("Startup").is_some(),
        "{w}x{h}: BackTab + Right must reach the Advanced tab"
    );
    t
}

/// T1: 120x40 full row — all 8 connections hints on line 40 (y=39),
/// `↑ ↓ Move` x=1-8 through `Tab Next` x=84-91, no `…`, no status.
#[test]
fn footer_full_row_120() {
    let t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    assert_eq!(row_span(&t, 39, 1, 8), "↑ ↓ Move", "first hint");
    assert_eq!(row_span(&t, 39, 84, 91), "Tab Next", "last hint");
    let row = row_text(&t, 39, 120);
    assert!(!row.contains('…'), "full row must carry no cut marker");
    assert_eq!(
        row_span(&t, 39, 92, 119),
        " ".repeat(28),
        "no right status on a fresh app"
    );
}

/// T2: 72x20 truncation — 5 hints + `…` x=62. D5/peer-fix tripwire:
/// without the fit-loop fix stock admits `/ Filter`.
#[test]
fn footer_truncated_72x20() {
    let t = Harness::new(TableProApp::default(), Theme::junie(), 72, 20);
    let row = row_text(&t, 19, 72);
    assert_eq!(t.cell(62, 19).symbol(), "…", "cut marker x=62");
    assert!(row.contains("Duplicate"), "fifth hint must fit");
    assert!(!row.contains("Filter"), "sixth hint must drop");
    assert!(!row.contains("Next"), "last hint must drop");
}

/// T3: right status at 120x40 — `connect(4)` paints `Connected to
/// Production` x=96-118 in Secondary, hints intact left.
#[test]
fn footer_right_status_120() {
    let mut app = TableProApp::default();
    assert!(app.connect(4), "Production is fixture index 4");
    let t = Harness::new(app, Theme::junie(), 120, 40);
    assert_eq!(
        row_span(&t, 39, 96, 118),
        "Connected to Production",
        "status geometry"
    );
    for x in 96..=118u16 {
        let cell = t.cell(x, 39);
        assert_eq!(cell.fg, SECONDARY, "status x={x} fg");
        assert_eq!(cell.bg, BLACK, "status x={x} bg");
    }
    assert!(
        row_span(&t, 39, 0, 95).contains("Quick open"),
        "explorer hints must survive left of the status"
    );
}

/// T4: edit badge at 72x20 — `/`+`stag` paints ` EDIT ` x=1-6 in
/// OnAccent/Accent/BOLD with `Type Filter` x=9 (D4 pin).
#[test]
fn footer_edit_badge_72x20() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 72, 20);
    let _ = t.key(KeyCode::Char('/'));
    let _ = t.type_str("stag");
    assert!(t.app().is_editing(), "filter must be editing after /+stag");
    assert_eq!(row_span(&t, 19, 1, 6), " EDIT ", "badge cells");
    assert_eq!(row_span(&t, 19, 9, 19), "Type Filter", "first hint");
    for x in 1..=6u16 {
        let cell = t.cell(x, 19);
        assert_eq!(cell.fg, ON_ACCENT, "badge x={x} fg");
        assert_eq!(cell.bg, ACCENT, "badge x={x} bg");
        assert_eq!(cell.modifier.bits() & BOLD, BOLD, "badge x={x} bold");
    }
}

/// T5: form fill at 120x40 — ctrl-n + Advanced keeps the hints and fills
/// the full row with Canvas `#000` incl the gaps (D2 tripwire).
#[test]
fn footer_form_fill_120() {
    let t = open_advanced_new(120, 40);
    let row = row_text(&t, 39, 120);
    assert!(
        row.contains("Basic / Advanced"),
        "form arm must show, got {row:?}"
    );
    for x in 0..120u16 {
        assert_eq!(t.cell(x, 39).bg, BLACK, "footer x={x} bg");
    }
}

/// T6: style recipe — key Primary+BOLD, action Muted, `…` Faint, status
/// Secondary, badge Accent. Zero-patch proof: fails if any part needs a
/// bg patch (B1 tripwire).
#[test]
fn footer_recipe_styles() {
    let t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let key = t.cell(1, 39);
    assert_eq!(key.symbol(), "↑", "key symbol");
    assert_eq!(key.fg, PRIMARY, "key fg");
    assert_eq!(key.bg, BLACK, "key bg");
    assert_eq!(key.modifier.bits() & BOLD, BOLD, "key bold");
    for x in 5..=8u16 {
        let cell = t.cell(x, 39);
        assert_eq!(cell.fg, MUTED, "action x={x} fg");
        assert_eq!(cell.bg, BLACK, "action x={x} bg");
        assert_eq!(cell.modifier.bits(), 0, "action x={x} mods");
    }
    let gap = t.cell(4, 39);
    assert_eq!(gap.symbol(), " ", "gap symbol");
    assert_eq!(gap.bg, BLACK, "gap bg");

    let t = Harness::new(TableProApp::default(), Theme::junie(), 72, 20);
    let cut = t.cell(62, 19);
    assert_eq!(cut.symbol(), "…", "cut symbol");
    assert_eq!(cut.fg, FAINT, "cut fg");
    assert_eq!(cut.bg, BLACK, "cut bg");

    let mut app = TableProApp::default();
    assert!(app.connect(4));
    let t = Harness::new(app, Theme::junie(), 120, 40);
    let status = t.cell(100, 39);
    assert_eq!(status.fg, SECONDARY, "status fg");
    assert_eq!(status.bg, BLACK, "status bg");

    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 72, 20);
    let _ = t.key(KeyCode::Char('/'));
    let _ = t.type_str("stag");
    let badge = t.cell(2, 19);
    assert_eq!(badge.symbol(), "E", "badge symbol");
    assert_eq!(badge.fg, ON_ACCENT, "badge fg");
    assert_eq!(badge.bg, ACCENT, "badge bg");
}

/// T7: the legacy row painter is gone (red on base). Needles are joined
/// so the literal never appears in this source, not even in comments.
#[test]
fn footer_no_legacy_painter() {
    let src = include_str!("../src/app.rs");
    for needle in [
        concat!("fn draw_", "footer"),
        concat!("fn footer_", "hints"),
        concat!("draw_", "footer("),
    ] {
        assert!(!src.contains(needle), "legacy painter survived: {needle}");
    }
}

/// T8: ring and click silence — the footer registers nothing, consumes
/// no click, and tabbing past it changes no stops (§4 guard).
#[test]
fn footer_ring_and_click_silence() {
    let mut h = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let before: Vec<Id> = h.ring().entries().iter().map(|e| e.id).collect();
    assert!(!h.tab_to(FOOTER), "footer is not a ring stop");
    assert!(!h.ring().is_registered(FOOTER), "no entry even disabled");
    assert!(!h.ring().contains(FOOTER), "not reachable");
    assert!(h.ring().reachable().all(|e| e.id != FOOTER));
    let after: Vec<Id> = h.ring().entries().iter().map(|e| e.id).collect();
    assert_eq!(before, after, "tabbing past the footer changes no stops");
    let r = h.click(100, 39);
    assert!(!r.is_consumed(), "footer click unconsumed, got {r:?}");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

/// T11: Connections destructive suppression at 80x24 — ctrl+D arms the
/// `Duplicated` status, then 7×Down + `d` opens the destructive intent:
/// intent hints show, the right status is gone (D3, load-bearing).
#[test]
fn footer_delete_dialog_suppression() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 80, 24);
    let _ = t.ctrl('d');
    assert!(
        row_text(&t, 23, 80).contains("Duplicated"),
        "ctrl+D must arm the status"
    );
    for _ in 0..7 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Char('d'));
    let row = row_text(&t, 23, 80);
    assert!(
        row.contains("Quick answer"),
        "intent arm must show, got {row:?}"
    );
    assert!(
        !row.contains("Duplicated"),
        "status must suppress, got {row:?}"
    );
    assert_eq!(
        row_span(&t, 23, 70, 79),
        " ".repeat(10),
        "right edge must be blank"
    );
}

/// T12: cell editing at 80x24 — `connect(4)` + open orders + Home +
/// 4×Right + Enter: ` EDIT ` x=1, `Enter Commit` x=9, status x=56, no
/// `…` (all three row features at once).
#[test]
fn footer_badge_overflow_status_80x24() {
    let mut app = TableProApp::default();
    assert!(app.connect(4), "Production is fixture index 4");
    let mut t = Harness::new(app, Theme::junie(), 80, 24);
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    let _ = t.key(KeyCode::Home);
    for _ in 0..4 {
        let _ = t.key(KeyCode::Right);
    }
    let _ = t.key(KeyCode::Enter);
    assert!(t.app().is_editing(), "cell edit must be active");
    assert_eq!(row_span(&t, 23, 1, 6), " EDIT ", "badge cells");
    assert_eq!(row_span(&t, 23, 9, 20), "Enter Commit", "first hint");
    assert_eq!(
        row_span(&t, 23, 56, 78),
        "Connected to Production",
        "status geometry"
    );
    assert!(
        !row_text(&t, 23, 80).contains('…'),
        "drawn==len, so no cut marker"
    );
}
