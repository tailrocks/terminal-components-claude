//! WI-TABLEPRO-TREE-GUTTERS: stock `Tree` owns the tree gutters.
//!
//! The legacy `paint_legacy_tree_gutters` overpainter in `tablepro-ui` is
//! deleted; gutter + fold cells render through the reusable `termrock::Tree`
//! (`Tree::draw` paints `Part::GUTTER` per row). Hardcoded literals below are
//! captured from base `7995daebefed` control dumps (aggregate SHA over the
//! 32 dump+glyph files in the control set:
//! `0c79cbd6392fc228b79a733622e06d6dfc41cce663d5a1ad699e69d894b6f3de`);
//! no test loads a frame file.
//!
//! * T1 pins the connections gutter column at 120x40 (boot, cursor row 0).
//! * T1b pins the filter-active pair (input cursor kept, tree gutter blank).
//! * T2 pins the focused-leaf row span; the single fold cell carries stock
//!   BOLD post-delete (control mods 0, the legacy `ui.fill` clobber removed).
//! * T3 pins the explorer gutter (all blank, focus is in the grid).
//! * T4 pins narrow sizes (gutter present, `▎` on the cursor row).
//! * T5 pins live stock ownership: `▎` tracks Down/Up with no repaint call.

use tablepro_ui::TableProApp;
use termrock::{Color, KeyCode, Theme};
use termrock_test_support::Harness;

const BLACK: Color = Color::Rgb(0, 0, 0);
const WHITE: Color = Color::Rgb(255, 255, 255);
const FOCUS: Color = Color::Rgb(72, 224, 84);
const MUTED: Color = Color::Rgb(128, 128, 128);
const BORDER: Color = Color::Rgb(77, 77, 77);
const CARD_BG: Color = Color::Rgb(17, 17, 17);
const INPUT_BG: Color = Color::Rgb(30, 30, 34);

/// Gutter column for both trees at every captured size.
const GUTTER_X: u16 = 3;
/// Fold column for depth-1 connection leaves (`tree_area.x + 3`).
const LEAF_FOLD_X: u16 = 6;

/// Symbols of row `y` over `x0..=x1`, one `String` per cell joined.
fn row_span(t: &Harness<TableProApp>, y: u16, x0: u16, x1: u16) -> String {
    (x0..=x1).map(|x| t.cell(x, y).symbol()).collect()
}

fn assert_cell(
    t: &Harness<TableProApp>,
    x: u16,
    y: u16,
    sym: &str,
    fg: Color,
    bg: Color,
    mods: u16,
) {
    let cell = t.cell(x, y);
    assert_eq!(cell.symbol(), sym, "({x},{y}) symbol");
    assert_eq!(cell.fg, fg, "({x},{y}) fg");
    assert_eq!(cell.bg, bg, "({x},{y}) bg");
    assert_eq!(cell.modifier.bits(), mods, "({x},{y}) mods");
}

fn assert_blank_gutter(t: &Harness<TableProApp>, y: u16) {
    assert_cell(t, GUTTER_X, y, " ", BLACK, BLACK, 0);
}

fn assert_focus_bar(t: &Harness<TableProApp>, y: u16) {
    assert_cell(t, GUTTER_X, y, "▎", FOCUS, BLACK, 1);
}

/// T1: connections gutter parity (120x40, boot): `▎` on the cursor row,
/// blanks elsewhere; fold cells on unfocused rows pin control bytes.
#[test]
fn tree_gutters_connections_parity_120() {
    let t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    assert_focus_bar(&t, 5);
    for y in 6..=12u16 {
        assert_blank_gutter(&t, y);
    }
    // Fold column on unfocused rows: leaf blanks, plus the Acme label cell.
    for y in [6u16, 7, 9, 10, 11, 12] {
        assert_cell(&t, LEAF_FOLD_X, y, " ", WHITE, BLACK, 0);
    }
    assert_cell(&t, LEAF_FOLD_X, 8, "A", WHITE, BLACK, 0);
}

/// T1b: filter-active control pair (120x40): focus sits in the filter input
/// (input `▎` kept), the tree gutter is all blank in both frames.
#[test]
fn tree_gutters_filter_active_pair_120() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = t.key(KeyCode::Char('/'));
    let _ = t.type_str("stag");
    assert!(
        t.app().connections_screen.filter_active,
        "filter must be active after /+stag"
    );
    assert_eq!(t.app().connections_screen.filter, "stag", "filter text");
    assert_cell(&t, GUTTER_X, 4, "▎", FOCUS, INPUT_BG, 0);
    assert_blank_gutter(&t, 5);
    assert_blank_gutter(&t, 6);
}

/// Expected style of row 7 (focused Scratch leaf) at 120x40, per column.
/// The fold cell (x=6) pins the POST-DELETE stock bytes (BOLD restored).
fn leaf1_row7_style(x: u16) -> (Color, Color, u16) {
    match x {
        0 | 2 => (WHITE, BLACK, 0),
        1 | 39 => (BORDER, BLACK, 0),
        3 => (FOCUS, BLACK, 1),
        4 | 5 | 7 => (WHITE, BLACK, 1),
        // Candidate value: control mods 0 (legacy clobber); stock restores 1.
        6 => (WHITE, BLACK, 1),
        8 => (MUTED, BLACK, 0),
        9..=30 => (WHITE, BLACK, 1),
        31..=36 => (MUTED, BLACK, 1),
        37 => (WHITE, BLACK, 1),
        38 => (WHITE, BLACK, 0),
        40 | 41 => (WHITE, BLACK, 0),
        42..=43 => (WHITE, CARD_BG, 0),
        44..=47 => (MUTED, CARD_BG, 0),
        48..=111 => (WHITE, CARD_BG, 0),
        112..=119 => (WHITE, BLACK, 0),
        _ => unreachable!("x={x} outside the pinned 120-column span"),
    }
}

/// T2: focused-leaf fold-cell pin (120x40). One Down lands on the Scratch
/// leaf (empirical: the boot Down skips Local PostgreSQL); every cell of the
/// row pins control bytes EXCEPT the fold cell, which carries stock BOLD.
#[test]
fn tree_gutters_focused_leaf_fold_bold_120() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = t.key(KeyCode::Down);
    assert_eq!(
        row_span(&t, 7, 0, 57),
        " │ ▎    · Scratch              sqlite  │    User         —",
        "focused-leaf row symbols"
    );
    for x in 58..120u16 {
        assert_eq!(t.cell(x, 7).symbol(), " ", "({x},7) symbol");
    }
    for x in 0..120u16 {
        let (fg, bg, mods) = leaf1_row7_style(x);
        let cell = t.cell(x, 7);
        assert_eq!(cell.fg, fg, "({x},7) fg");
        assert_eq!(cell.bg, bg, "({x},7) bg");
        assert_eq!(cell.modifier.bits(), mods, "({x},7) mods");
    }
    for y in [5u16, 6, 8, 9, 10, 11, 12] {
        assert_blank_gutter(&t, y);
    }
}

/// T3: explorer gutter parity (120x40, `--connect Production` journey):
/// the orders table is open, focus is in the grid, the explorer gutter is
/// all blank with zero accepted delta (no `Leaf` kind in the explorer).
#[test]
fn tree_gutters_explorer_parity_120() {
    let mut app = TableProApp::default();
    assert!(app.connect(4), "connect(4) must open Production");
    let mut t = Harness::new(app, Theme::junie(), 120, 40);
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    assert!(
        t.find("public › orders").is_some(),
        "explorer journey must reach public › orders"
    );
    assert_cell(&t, 34, 9, "▎", FOCUS, BLACK, 1);
    for y in 7..=20u16 {
        assert_blank_gutter(&t, y);
    }
}

/// T4: narrow sizes keep the gutter column with `▎` on the cursor row.
#[test]
fn tree_gutters_narrow() {
    for (w, h) in [(72u16, 20u16), (80u16, 24u16)] {
        let t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
        assert_focus_bar(&t, 5);
        for y in 6..=12u16 {
            assert_blank_gutter(&t, y);
        }
        let mut t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
        let _ = t.key(KeyCode::Down);
        assert_focus_bar(&t, 7);
        for y in [5u16, 6, 8, 9, 10, 11, 12] {
            assert_blank_gutter(&t, y);
        }
    }
}

/// T5: ownership — `▎` tracks the stock cursor live: from the Development
/// row one Down moves it exactly one row, the old cell returns to blank,
/// and Up restores it. (Mid-tree start: the boot Down empirically jumps
/// y5→y7; mid-tree steps are single.)
#[test]
fn tree_gutters_focus_bar_moves_live() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    assert_focus_bar(&t, 5);
    for _ in 0..3 {
        let _ = t.key(KeyCode::Down);
    }
    assert_focus_bar(&t, 9);
    let _ = t.key(KeyCode::Down);
    assert_focus_bar(&t, 10);
    assert_blank_gutter(&t, 9);
    let _ = t.key(KeyCode::Up);
    assert_focus_bar(&t, 9);
    assert_blank_gutter(&t, 10);
}
