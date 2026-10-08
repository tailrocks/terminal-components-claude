//! WI-TABLEPRO-FORM-RADIO: the connection-form environment/safe-mode rows
//! are stock `RadioGroup` in a `Field` wrapper (ids `ENVIRONMENT`/
//! `SAFE_MODE`), fed per-frame from `draft.environment`/`draft.safe_mode`,
//! with the safe-mode description through stock `Note`.
//!
//! Coord convention (all tests): rows are 1-based oracle lines, cols are
//! 0-based harness x; harness y = row-1. Geometry pinned against the approved
//! `form-new/{120x40,80x24,100x30,160x50,72x20}/truecolor.txt` frames;
//! no test loads a frame file.
//!
//! * T1/T2 pin the 120x40 env/safe geometry (caption, markers, labels).
//! * T3 pins the 120x40 two-line desc (take(2)+clip: `still confirm.`
//!   dropped).
//! * T4 pins the 100x30 desc wrap (a `Field::help` single row fails this).
//! * T5 pins the 80x24 below-fold desc guard + the beneath-fg zone.
//! * T6 pins the 72x20 clipping (caption overpainted, one safe row left)
//!   + the beneath-fg zone.
//! * T7 pins the accepted D2 delta: arrows move the cursor without a draft
//!   write, Space commits, the desc follows `draft.safe_mode`.
//! * T8 pins the unfocused style recipe incl. the card-bg rows (the B1
//!   tripwire), the `( )` off marker (the L1 tripwire) and the Muted
//!   off-marker tone (the L3 tripwire).
//! * T9 pins the accepted D1 delta: the focused radio shows BOLD on the
//!   cursor row + BOLD Secondary caption with an invisible gutter (legacy
//!   rendered no radio focus).
//! * T10 pins deletion of the legacy `draw_form_radio` painter.
//! * T11 pins submit-wins: Enter on a focused radio submits (cursor
//!   discarded, never auto-committed); clicks choose; the ring order is
//!   …→ENVIRONMENT→GROUP→SAFE_MODE→….
//! * T12 pins the 160x50 geometry.
//!
//! T1–T6/T8/T10/T12 are green-on-base guards (T10 red-on-base by deletion);
//! T7/T9/T11 are red-on-base discriminators.

use tablepro_domain::Environment;
use tablepro_ui::TableProApp;
use tablepro_ui::connections::field::{ENVIRONMENT, GROUP, NAME, SAFE_MODE};
use termrock::{Color, KeyCode, Theme};
use termrock_test_support::Harness;

const CARD: Color = Color::Rgb(17, 17, 17);
const MUTED: Color = Color::Rgb(128, 128, 128);
const SECONDARY: Color = Color::Rgb(179, 179, 179);
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

fn drafts(t: &Harness<TableProApp>) -> (usize, usize) {
    let draft = t.app().connection_draft().expect("form must be open");
    (draft.environment, draft.safe_mode)
}

/// Symbols of row `y` over `x0..=x1`, one `String` per cell joined.
fn row_span(t: &Harness<TableProApp>, y: u16, x0: u16, x1: u16) -> String {
    (x0..=x1).map(|x| t.cell(x, y).symbol()).collect()
}

/// T1: 120x40 env geometry — caption line 8 (y=7) x=90, `(●)` line 9
/// (y=8) x=89 with `local` x=93, `( )` lines 10–12 (y=9–11) x=89 with
/// lowercase labels.
#[test]
fn form_radio_env_geometry_120() {
    let t = open_new_form(120, 40);
    assert_eq!(row_span(&t, 7, 90, 100), "Environment", "caption");
    assert_eq!(row_span(&t, 8, 89, 91), "(●)", "on marker");
    assert_eq!(row_span(&t, 8, 93, 97), "local", "on label");
    assert_eq!(row_span(&t, 9, 89, 91), "( )", "off marker 1");
    assert_eq!(row_span(&t, 9, 93, 103), "development", "label 1");
    assert_eq!(row_span(&t, 10, 89, 91), "( )", "off marker 2");
    assert_eq!(row_span(&t, 10, 93, 99), "staging", "label 2");
    assert_eq!(row_span(&t, 11, 89, 91), "( )", "off marker 3");
    assert_eq!(row_span(&t, 11, 93, 102), "production", "label 3");
}

/// T2: 120x40 safe geometry — caption line 17 (y=16) x=90, `(●)` line 18
/// (y=17) x=89 with `Silent` x=93, `( )` lines 19–23 (y=18–22) x=89 with
/// safe labels.
#[test]
fn form_radio_safe_geometry_120() {
    let t = open_new_form(120, 40);
    assert_eq!(row_span(&t, 16, 90, 98), "Safe Mode", "caption");
    assert_eq!(row_span(&t, 17, 89, 91), "(●)", "on marker");
    assert_eq!(row_span(&t, 17, 93, 98), "Silent", "on label");
    for (y, label) in [
        (18, "Alert"),
        (19, "Alert (Full)"),
        (20, "Safe Mode"),
        (21, "Safe Mode (Full)"),
        (22, "Read-Only"),
    ] {
        assert_eq!(row_span(&t, y, 89, 91), "( )", "off marker y={y}");
        let end = 93 + label.len() as u16 - 1;
        assert_eq!(row_span(&t, y, 93, end), label, "label y={y}");
    }
}

/// T3: 120x40 desc — line 24 (y=23) `Writes run without asking.` x=90,
/// line 25 (y=24) `Destructive statements` x=90; `still confirm.` is
/// take(2)-dropped.
#[test]
fn form_radio_desc_120() {
    let t = open_new_form(120, 40);
    assert_eq!(
        row_span(&t, 23, 90, 115),
        "Writes run without asking.",
        "desc row 1"
    );
    assert_eq!(
        row_span(&t, 24, 90, 111),
        "Destructive statements",
        "desc row 2"
    );
    assert!(
        !row_span(&t, 24, 90, 117).contains("still"),
        "take(2) must drop `still confirm.`"
    );
}

/// T4: 100x30 desc wrap — line 24 (y=23) `Writes run without` x=75,
/// line 25 (y=24) `asking. Destructive` x=75. A `Field::help` single
/// truncated row fails this.
#[test]
fn form_radio_desc_wrap_100x30() {
    let t = open_new_form(100, 30);
    assert_eq!(row_span(&t, 23, 75, 92), "Writes run without", "desc row 1");
    assert_eq!(
        row_span(&t, 24, 75, 93),
        "asking. Destructive",
        "desc row 2"
    );
}

/// T5: 80x24 (fresh form carries the same index-0 radios as the filled
/// oracle: typing is a no-op while the form is not editing) — `(●)` x=48
/// lines 9,18 (y=8,17); line 24 is the action row, the desc is below
/// the fold. The caption row keeps the beneath-fg Muted zone the
/// connections tree paints (bg-only Field fill tripwire).
#[test]
fn form_radio_80x24_no_desc() {
    let t = open_new_form(80, 24);
    assert_eq!(row_span(&t, 8, 48, 50), "(●)", "env on marker");
    assert_eq!(row_span(&t, 17, 48, 50), "(●)", "safe on marker");
    assert!(
        t.find("Writes run").is_none(),
        "80x24: the desc must be below the fold"
    );
    assert!(
        row_span(&t, 23, 0, 79).contains("Save"),
        "80x24 line 24 must be the action row"
    );
    for x in 70..=75u16 {
        let cell = t.cell(x, 7);
        assert_eq!(cell.symbol(), " ", "beneath x={x} symbol");
        assert_eq!(cell.fg, MUTED, "beneath x={x} fg");
        assert_eq!(cell.bg, CARD, "beneath x={x} bg");
    }
}

/// T6: 72x20 — `(●)` x=43 lines 9,18 (y=8,17); no `Safe Mode` caption
/// anywhere (the action row overpaints line 17); `( ) Alert` line 19
/// only (below-fold guard). The caption row keeps the beneath-fg Muted
/// zone the connections tree paints (bg-only Field fill tripwire).
#[test]
fn form_radio_72x20_clipped() {
    let t = open_new_form(72, 20);
    assert_eq!(row_span(&t, 8, 43, 45), "(●)", "env on marker");
    assert_eq!(row_span(&t, 17, 43, 45), "(●)", "safe on marker");
    assert!(
        t.find("Safe Mode").is_none(),
        "72x20: no Safe Mode text may survive"
    );
    assert_eq!(row_span(&t, 18, 43, 45), "( )", "Alert marker");
    assert_eq!(row_span(&t, 18, 47, 51), "Alert", "Alert label");
    assert!(
        t.find("Alert (Full)").is_none(),
        "72x20: deeper safe rows must be below the fold"
    );
    for x in 62..=67u16 {
        let cell = t.cell(x, 7);
        assert_eq!(cell.symbol(), " ", "beneath x={x} symbol");
        assert_eq!(cell.fg, MUTED, "beneath x={x} fg");
        assert_eq!(cell.bg, CARD, "beneath x={x} bg");
    }
}

/// T7: accepted D2 delta — editing-mode-first, then `tab_to` ENVIRONMENT
/// (fails on base: no stop); arrows move the cursor WITHOUT a draft
/// write (BOLD cursor row + unchanged draft); Space commits and repaints;
/// same for SAFE_MODE with the desc following `draft.safe_mode`.
/// Red on base.
#[test]
fn form_radio_commit_changes_draft() {
    let mut t = open_new_form(120, 40);
    let _ = t.key(KeyCode::Enter);
    assert!(t.app().is_editing(), "Enter on NAME must start editing");
    assert!(t.tab_to(ENVIRONMENT), "ENVIRONMENT must be Tab-reachable");
    assert_eq!(drafts(&t), (0, 0), "fresh draft must start at index 0");
    let _ = t.key(KeyCode::Down);
    let _ = t.key(KeyCode::Down);
    assert_eq!(drafts(&t).0, 0, "arrows must not write the draft");
    assert_eq!(
        t.cell(89, 10).modifier.bits(),
        1,
        "the cursor row (staging) must be BOLD"
    );
    assert_eq!(
        t.cell(89, 8).modifier.bits(),
        0,
        "the value row (local) must not be BOLD"
    );
    let _ = t.key(KeyCode::Char(' '));
    assert_eq!(drafts(&t).0, 2, "Space must commit the cursor");
    assert_eq!(row_span(&t, 10, 89, 91), "(●)", "marker must follow");
    assert_eq!(row_span(&t, 10, 93, 99), "staging", "label intact");
    assert_eq!(row_span(&t, 8, 89, 91), "( )", "old row must clear");

    assert!(t.tab_to(SAFE_MODE), "SAFE_MODE must be Tab-reachable");
    let _ = t.key(KeyCode::Down);
    assert_eq!(drafts(&t).1, 0, "arrows must not write the draft");
    let _ = t.key(KeyCode::Char(' '));
    assert_eq!(drafts(&t).1, 1, "Space must commit the cursor");
    assert_eq!(row_span(&t, 18, 89, 91), "(●)", "marker must follow");
    assert_eq!(
        row_span(&t, 23, 90, 109),
        "Every write asks for",
        "the desc must follow draft.safe_mode"
    );
}

/// T8: unfocused recipe — on-marker Accent `(●)`, off-marker Muted `( )`
/// (L3 tripwire: pre-L3 Primary), caption Secondary, option text
/// Primary, all on card (B1 tripwire), gutter blank card. L1 tripwire:
/// off-marker cells read `(`, ` `, `)`, never `○`.
#[test]
fn form_radio_unfocused_recipe() {
    let t = open_new_form(120, 40);
    // Env caption: Secondary on card, no mods.
    for x in 90..=100u16 {
        let cell = t.cell(x, 7);
        assert_eq!(cell.fg, SECONDARY, "caption x={x} fg");
        assert_eq!(cell.bg, CARD, "caption x={x} bg");
        assert_eq!(cell.modifier.bits(), 0, "caption x={x} mods");
    }
    // On marker: Accent on card, no mods.
    for x in 89..=91u16 {
        let cell = t.cell(x, 8);
        assert_eq!(cell.fg, ACCENT, "on-marker x={x} fg");
        assert_eq!(cell.bg, CARD, "on-marker x={x} bg");
        assert_eq!(cell.modifier.bits(), 0, "on-marker x={x} mods");
    }
    // Off markers: Muted on card, `(`, ` `, `)` cells, no mods.
    for y in 9..=11u16 {
        assert_eq!(t.cell(89, y).symbol(), "(", "off-marker y={y} head");
        assert_eq!(t.cell(90, y).symbol(), " ", "off-marker y={y} blank");
        assert_eq!(t.cell(91, y).symbol(), ")", "off-marker y={y} tail");
        for x in 89..=91u16 {
            let cell = t.cell(x, y);
            assert_eq!(cell.fg, MUTED, "off-marker ({x},{y}) fg");
            assert_eq!(cell.bg, CARD, "off-marker ({x},{y}) bg");
            assert_eq!(cell.modifier.bits(), 0, "off-marker ({x},{y}) mods");
        }
    }
    // Option text: Primary on card.
    for x in 93..=97u16 {
        let cell = t.cell(x, 8);
        assert_eq!(cell.fg, PRIMARY, "label x={x} fg");
        assert_eq!(cell.bg, CARD, "label x={x} bg");
    }
    // Gutter blank card + row fill card (B1 tripwire).
    let gutter = t.cell(88, 8);
    assert_eq!(gutter.symbol(), " ", "gutter symbol");
    assert_eq!(gutter.fg, CARD, "gutter fg");
    assert_eq!(gutter.bg, CARD, "gutter bg");
    for x in [88u16, 92, 100, 110] {
        let cell = t.cell(x, 8);
        assert_eq!(cell.bg, CARD, "fill x={x} bg");
    }
    // Safe site, same recipe (spot check).
    assert_eq!(t.cell(90, 16).fg, SECONDARY, "safe caption fg");
    assert_eq!(t.cell(89, 17).fg, ACCENT, "safe on-marker fg");
    assert_eq!(t.cell(89, 18).fg, MUTED, "safe off-marker fg");
    assert_eq!(row_span(&t, 18, 89, 91), "( )", "safe off marker");
    // Desc: Muted on card.
    assert_eq!(t.cell(90, 23).fg, MUTED, "desc fg");
    assert_eq!(t.cell(90, 23).bg, CARD, "desc bg");
}

/// T9: accepted D1 delta — editing-mode-first, then Tab to ENVIRONMENT:
/// BOLD cursor row + BOLD Secondary caption + invisible gutter. Red on
/// base (legacy rendered no radio focus).
#[test]
fn form_radio_focused_delta() {
    let mut t = open_new_form(120, 40);
    let _ = t.key(KeyCode::Enter);
    assert!(t.app().is_editing(), "Enter on NAME must start editing");
    assert!(t.tab_to(ENVIRONMENT), "ENVIRONMENT must be Tab-reachable");
    assert_eq!(
        t.focus(),
        Some(ENVIRONMENT),
        "focus must sit on ENVIRONMENT"
    );
    // The cursor seeds to the value: row y=8 is cursor and value.
    for x in 89..=91u16 {
        let cell = t.cell(x, 8);
        assert_eq!(cell.fg, ACCENT, "focused marker x={x} fg");
        assert_eq!(cell.bg, CARD, "focused marker x={x} bg");
        assert_eq!(cell.modifier.bits(), 1, "focused marker x={x} bold");
    }
    for x in 93..=97u16 {
        let cell = t.cell(x, 8);
        assert_eq!(cell.fg, PRIMARY, "focused label x={x} fg");
        assert_eq!(cell.bg, CARD, "focused label x={x} bg");
        assert_eq!(cell.modifier.bits(), 1, "focused label x={x} bold");
    }
    for x in 90..=100u16 {
        let cell = t.cell(x, 7);
        assert_eq!(cell.fg, SECONDARY, "focused caption x={x} fg");
        assert_eq!(cell.bg, CARD, "focused caption x={x} bg");
        assert_eq!(cell.modifier.bits(), 1, "focused caption x={x} bold");
    }
    let gutter = t.cell(88, 8);
    assert_eq!(gutter.symbol(), "▎", "focus bar glyph");
    assert_eq!(gutter.fg, CARD, "focus bar fg invisible");
    assert_eq!(gutter.bg, CARD, "focus bar bg");
}

/// T10: no legacy painter — `draw_form_radio` is deleted, both call
/// sites gone. Red on base.
#[test]
fn form_radio_no_legacy_painter() {
    let src = include_str!("../src/app.rs");
    assert!(
        !src.contains("fn draw_form_radio"),
        "legacy draw_form_radio definition survived"
    );
    assert!(
        !src.contains("draw_form_radio("),
        "legacy draw_form_radio call survived"
    );
}

/// T11: submit wins — the corpus `e` path (name+database pre-filled,
/// so the validity gate passes; a fresh form cannot fill DATABASE, which
/// has no ring stop yet), editing-mode-first, ring order
/// …→ENVIRONMENT→GROUP→SAFE_MODE→…; click chooses (unclaimed path);
/// arrows move the cursor without a draft write; Space commits; Enter
/// submits with the last committed value (cursor discarded, never
/// auto-committed). Red on base.
#[test]
fn form_radio_enter_submits() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    for _ in 0..7 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Char('e'));
    assert!(t.find("Name").is_some(), "e path must open the edit form");
    assert!(t.tab_to(NAME), "NAME must be Tab-reachable");
    let _ = t.key(KeyCode::Enter);
    assert!(t.app().is_editing(), "Enter on NAME must start editing");
    let entries = t.ring().entries();
    let env = entries
        .iter()
        .position(|e| e.id == ENVIRONMENT)
        .expect("ENVIRONMENT must be registered");
    let group = entries
        .iter()
        .position(|e| e.id == GROUP)
        .expect("GROUP must be registered");
    let safe = entries
        .iter()
        .position(|e| e.id == SAFE_MODE)
        .expect("SAFE_MODE must be registered");
    assert!(!entries[env].disabled, "ENVIRONMENT stop must be enabled");
    assert!(!entries[safe].disabled, "SAFE_MODE stop must be enabled");
    assert!(
        env < group && group < safe,
        "stops must register in draw order ENVIRONMENT < GROUP < SAFE_MODE"
    );
    assert_eq!(drafts(&t), (3, 3), "the corpus pick starts Production/Safe");
    assert!(t.tab_to(ENVIRONMENT), "ENVIRONMENT must be Tab-reachable");
    // Click chooses through the unclaimed path.
    let _ = t.click(95, 10);
    assert_eq!(drafts(&t).0, 2, "click must choose staging");
    assert_eq!(row_span(&t, 10, 89, 91), "(●)", "marker must follow");
    // Arrows move the cursor without a draft write; Space commits.
    let _ = t.key(KeyCode::Up);
    assert_eq!(drafts(&t).0, 2, "Up must not write the draft");
    let _ = t.key(KeyCode::Char(' '));
    assert_eq!(drafts(&t).0, 1, "Space must commit development");
    let _ = t.key(KeyCode::Down);
    assert_eq!(drafts(&t).0, 1, "Down must not write the draft");
    // Enter submits with the last committed value; the cursor (staging)
    // is discarded, never auto-committed.
    let _ = t.key(KeyCode::Enter);
    assert!(
        !t.app().connection_form_open(),
        "Enter must submit and close the form"
    );
    let pushed = t
        .app()
        .connections_screen
        .connections
        .last()
        .expect("submit must push a connection");
    assert_eq!(pushed.name, "Production");
    assert_eq!(
        pushed.environment,
        Environment::Development,
        "the pushed connection keeps the last Space-committed value"
    );
}

/// T12: 160x50 — marker x=94 lines 9,18 (y=8,17); captions/desc x=95;
/// both desc lines full.
#[test]
fn form_radio_160x50_geometry() {
    let t = open_new_form(160, 50);
    assert_eq!(row_span(&t, 7, 95, 105), "Environment", "env caption");
    assert_eq!(row_span(&t, 8, 94, 96), "(●)", "env on marker");
    assert_eq!(row_span(&t, 16, 95, 103), "Safe Mode", "safe caption");
    assert_eq!(row_span(&t, 17, 94, 96), "(●)", "safe on marker");
    assert_eq!(
        row_span(&t, 23, 95, 120),
        "Writes run without asking.",
        "desc row 1"
    );
    assert_eq!(
        row_span(&t, 24, 95, 122),
        "Destructive statements still",
        "desc row 2"
    );
}
