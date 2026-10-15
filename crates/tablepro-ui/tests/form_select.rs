//! WI-TABLEPRO-FORM-SELECT: the connection-form Engine/Group rows are stock
//! `Field` + `SelectField` (ids `ENGINE`/`GROUP`), fed per-frame from
//! `draft.engine`/`draft.group` (`GROUP` clamped to `GROUPS.len()-1`).
//!
//! Coord convention (all tests): rows are 1-based oracle lines, cols are
//! 0-based harness x; harness y = row-1. Geometry pinned against the approved
//! `form-new/120x40/truecolor.txt`, `form-new-filled/72x20/truecolor.txt`, and
//! `form-new-filled/80x24/truecolor.txt` frames; no test loads a frame file.
//!
//! * T1/T2 pin the 120x40 Engine/Group geometry (label, value, `▾`).
//! * T3 pins engine variants via the tree `e` edit path (MySQL/SQLite rows).
//! * T4 pins Group=1 (`Acme`) via the tree `e` edit path.
//! * T5/T6 pin the 72x20 and 80x24 geometry.
//! * T7 pins the unfocused style recipe incl. the card-bg rows (the B1
//!   tripwire: without the CONTAINER patch they render #000).
//! * T8 pins the accepted D1 delta: focused label is BOLD Primary + a
//!   FocusBar gutter (legacy rendered no select focus).
//! * T9 pins deletion of the legacy `draw_form_select` painter.
//! * T10 pins the accepted D2 delta: ENGINE/GROUP registered as Focusable
//!   ring stops in draw order (legacy registered nothing).
//!
//! T1–T7 are green-on-base guards; T8/T9/T10 are red-on-base discriminators.

use tablepro_ui::TableProApp;
use tablepro_ui::connections::field::{ENGINE, GROUP};
use termrock::{Color, KeyCode, Theme};
use termrock_test_support::Harness;

const CARD: Color = Color::Rgb(17, 17, 17);
const INPUT: Color = Color::Rgb(30, 30, 34);
const PRIMARY: Color = Color::Rgb(255, 255, 255);
const SECONDARY: Color = Color::Rgb(179, 179, 179);
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

/// Open the edit form for the connection `downs` rows below the initial
/// cursor, asserting it is the expected fixture row.
fn open_edit_form(downs: u32, want_name: &str) -> Harness<TableProApp> {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    for _ in 0..downs {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Char('e'));
    let draft = t
        .app()
        .connection_draft()
        .expect("e path must open the edit form");
    assert_eq!(
        draft.name, want_name,
        "{downs} downs + e must open {want_name}"
    );
    t
}

/// Symbols of row `y` over `x0..=x1`, one `String` per cell joined.
fn row_span(t: &Harness<TableProApp>, y: u16, x0: u16, x1: u16) -> String {
    (x0..=x1).map(|x| t.cell(x, y).symbol()).collect()
}

/// T1: 120x40 Engine geometry — label line 11 (y=10) x=46, value line 12
/// (y=11) x=46, `▾` line 12 (y=11) x=82.
#[test]
fn form_select_engine_geometry_120() {
    let t = open_new_form(120, 40);
    assert_eq!(row_span(&t, 10, 46, 51), "Engine", "engine label");
    assert_eq!(row_span(&t, 11, 46, 55), "PostgreSQL", "engine value");
    assert_eq!(t.cell(82, 11).symbol(), "▾", "engine arrow");
}

/// T2: 120x40 Group geometry — label line 14 (y=13) x=90, value line 15
/// (y=14) x=90, `▾` line 15 (y=14) x=115.
#[test]
fn form_select_group_geometry_120() {
    let t = open_new_form(120, 40);
    assert_eq!(row_span(&t, 13, 90, 94), "Group", "group label");
    assert_eq!(row_span(&t, 14, 90, 97), "Personal", "group value");
    assert_eq!(t.cell(115, 14).symbol(), "▾", "group arrow");
}

/// T3: engine variants — value text swaps, arrow cell fixed at line 12
/// (y=11) x=82. Analytics is MySQL (5 downs), Scratch is SQLite (1 down).
#[test]
fn form_select_engine_variants() {
    let t = open_edit_form(5, "Analytics");
    assert_eq!(row_span(&t, 11, 46, 50), "MySQL", "engine=1 value");
    assert_eq!(t.cell(82, 11).symbol(), "▾", "engine=1 arrow fixed");

    let t = open_edit_form(1, "Scratch");
    assert_eq!(row_span(&t, 11, 46, 51), "SQLite", "engine=2 value");
    assert_eq!(t.cell(82, 11).symbol(), "▾", "engine=2 arrow fixed");
}

/// T4: Group=1 renders `Acme` at line 15 (y=14) x=90, arrow fixed at line 15
/// (y=14) x=115. Development sits in Acme (3 downs).
#[test]
fn form_select_group_acme() {
    let t = open_edit_form(3, "Development");
    assert_eq!(row_span(&t, 14, 90, 93), "Acme", "group=1 value");
    assert_eq!(t.cell(115, 14).symbol(), "▾", "group=1 arrow fixed");
}

/// T5: 72x20 geometry — `Engine` line 11 (y=10) x=5, `PostgreSQL` line 12
/// (y=11) x=5, `▾` line 12 (y=11) x=36, `Group` line 14 (y=13) x=44,
/// `Personal` line 15 (y=14) x=44, `▾` line 15 (y=14) x=67.
#[test]
fn form_select_tiny_72x20() {
    let t = open_new_form(72, 20);
    assert_eq!(row_span(&t, 10, 5, 10), "Engine", "engine label");
    assert_eq!(row_span(&t, 11, 5, 14), "PostgreSQL", "engine value");
    assert_eq!(t.cell(36, 11).symbol(), "▾", "engine arrow");
    assert_eq!(row_span(&t, 13, 44, 48), "Group", "group label");
    assert_eq!(row_span(&t, 14, 44, 51), "Personal", "group value");
    assert_eq!(t.cell(67, 14).symbol(), "▾", "group arrow");
}

/// T6: 80x24 geometry — arrows at x=41 (Engine, line 12/y=11) and x=75
/// (Group, line 15/y=14); labels/values at their approved columns.
#[test]
fn form_select_small_80x24() {
    let t = open_new_form(80, 24);
    assert_eq!(row_span(&t, 10, 5, 10), "Engine", "engine label");
    assert_eq!(row_span(&t, 11, 5, 14), "PostgreSQL", "engine value");
    assert_eq!(t.cell(41, 11).symbol(), "▾", "engine arrow");
    assert_eq!(row_span(&t, 13, 49, 53), "Group", "group label");
    assert_eq!(row_span(&t, 14, 49, 56), "Personal", "group value");
    assert_eq!(t.cell(75, 14).symbol(), "▾", "group arrow");
}

/// T7: unfocused recipe — value Primary on INPUT, arrow Secondary on INPUT,
/// label Secondary on card, label-row blanks + row 3 card bg. Doubles as the
/// B1 tripwire: without the CONTAINER patch the card rows render #000.
#[test]
fn form_select_unfocused_recipe() {
    let t = open_new_form(120, 40);
    // Engine label: Secondary on card, no mods.
    for x in 46..=51u16 {
        let cell = t.cell(x, 10);
        assert_eq!(cell.fg, SECONDARY, "engine label x={x} fg");
        assert_eq!(cell.bg, CARD, "engine label x={x} bg");
        assert_eq!(cell.modifier.bits(), 0, "engine label x={x} mods");
    }
    // Engine value: Primary on INPUT, no mods.
    for x in 46..=55u16 {
        let cell = t.cell(x, 11);
        assert_eq!(cell.fg, PRIMARY, "engine value x={x} fg");
        assert_eq!(cell.bg, INPUT, "engine value x={x} bg");
        assert_eq!(cell.modifier.bits(), 0, "engine value x={x} mods");
    }
    // Engine arrow: Secondary on INPUT.
    let arrow = t.cell(82, 11);
    assert_eq!(arrow.fg, SECONDARY, "engine arrow fg");
    assert_eq!(arrow.bg, INPUT, "engine arrow bg");
    // Card rows: label-row blanks + row 3 (B1 tripwire).
    for (x, y) in [(44, 10), (45, 10), (60, 10), (44, 12), (60, 12), (82, 12)] {
        assert_eq!(t.cell(x, y).bg, CARD, "card row ({x},{y}) bg");
    }
    // Group site, same recipe.
    for x in 90..=94u16 {
        let cell = t.cell(x, 13);
        assert_eq!(cell.fg, SECONDARY, "group label x={x} fg");
        assert_eq!(cell.bg, CARD, "group label x={x} bg");
    }
    for x in 90..=97u16 {
        let cell = t.cell(x, 14);
        assert_eq!(cell.fg, PRIMARY, "group value x={x} fg");
        assert_eq!(cell.bg, INPUT, "group value x={x} bg");
    }
    let arrow = t.cell(115, 14);
    assert_eq!(arrow.fg, SECONDARY, "group arrow fg");
    assert_eq!(arrow.bg, INPUT, "group arrow bg");
    assert_eq!(t.cell(88, 15).bg, CARD, "group row-3 bg");
}

/// T8: accepted D1 delta — focused ENGINE shows a BOLD Primary label plus a
/// FocusBar gutter; same for GROUP. Red on base (no stops, no focus paint).
#[test]
fn form_select_focused_delta() {
    let mut t = open_new_form(120, 40);
    assert!(t.tab_to(ENGINE), "ENGINE must be Tab-reachable");
    assert_eq!(t.focus(), Some(ENGINE), "focus must sit on ENGINE");
    for x in 46..=51u16 {
        let cell = t.cell(x, 10);
        assert_eq!(cell.fg, PRIMARY, "focused engine label x={x} fg");
        assert_eq!(cell.bg, CARD, "focused engine label x={x} bg");
        assert_eq!(cell.modifier.bits(), 1, "focused engine label x={x} bold");
    }
    let gutter = t.cell(44, 11);
    assert_eq!(gutter.symbol(), "▎", "engine focus bar glyph");
    assert_eq!(gutter.fg, ACCENT, "engine focus bar fg");
    assert_eq!(gutter.bg, INPUT, "engine focus bar bg");

    assert!(t.tab_to(GROUP), "GROUP must be Tab-reachable");
    assert_eq!(t.focus(), Some(GROUP), "focus must sit on GROUP");
    for x in 90..=94u16 {
        let cell = t.cell(x, 13);
        assert_eq!(cell.fg, PRIMARY, "focused group label x={x} fg");
        assert_eq!(cell.bg, CARD, "focused group label x={x} bg");
        assert_eq!(cell.modifier.bits(), 1, "focused group label x={x} bold");
    }
    let gutter = t.cell(88, 14);
    assert_eq!(gutter.symbol(), "▎", "group focus bar glyph");
    assert_eq!(gutter.fg, ACCENT, "group focus bar fg");
    assert_eq!(gutter.bg, INPUT, "group focus bar bg");
}

/// T9: no legacy painter — `draw_form_select` is deleted, both call sites
/// gone. Red on base.
#[test]
fn form_select_no_legacy_painter() {
    let src = include_str!("../src/app.rs");
    assert!(
        !src.contains("fn draw_form_select"),
        "legacy draw_form_select definition survived"
    );
    assert!(
        !src.contains("draw_form_select("),
        "legacy draw_form_select call survived"
    );
}

/// T10: accepted D2 delta — ENGINE/GROUP registered as enabled ring stops in
/// draw order. Red on base (legacy registered nothing).
#[test]
fn form_select_ring_stops() {
    let t = open_new_form(120, 40);
    let entries = t.ring().entries();
    let engine = entries
        .iter()
        .position(|e| e.id == ENGINE)
        .expect("ENGINE must be registered");
    let group = entries
        .iter()
        .position(|e| e.id == GROUP)
        .expect("GROUP must be registered");
    assert!(!entries[engine].disabled, "ENGINE stop must be enabled");
    assert!(!entries[group].disabled, "GROUP stop must be enabled");
    assert!(
        engine < group,
        "ENGINE (drawn first) must precede GROUP in the ring"
    );
}
