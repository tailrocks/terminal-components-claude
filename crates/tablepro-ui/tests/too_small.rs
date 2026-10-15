//! WI-TABLEPRO-TOO-SMALL: stock `TooSmall` owns the too-small notice.
//!
//! The hand-painted `draw_too_small` in `tablepro-ui` is deleted; the
//! sub-minimum notice renders through the reusable `termrock::TooSmall`
//! (`TooSmall::new(TOO_SMALL, "TablePro").minimum(72, 20)`). Expected cells
//! below pin the oracle painter bytes (`visual-baseline:src/bin/tablepro/app.rs`)
//! at 60x15 under the junie theme: product TITLE in Primary+BOLD, detail in
//! Secondary, dimensions in Muted, quit hint in Faint, CONTAINER fill.
//!
//! * T1 pins the full notice block at 60x15 (rows, tiers, fill, no leaks).
//! * T2 pins the boundary: at exactly 72x20 the notice is not shown.
//! * T3 pins the resize ladder: entering and leaving the notice.
//! * T4 pins the deletion: no `draw_too_small` remains in `src/app.rs`.

use tablepro_ui::TableProApp;
use termrock::{Color, Theme};
use termrock_test_support::Harness;

const BLACK: Color = Color::Rgb(0, 0, 0);
const WHITE: Color = Color::Rgb(255, 255, 255);
const SECONDARY: Color = Color::Rgb(179, 179, 179);
const MUTED: Color = Color::Rgb(128, 128, 128);
const FAINT: Color = Color::Rgb(77, 77, 77);

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

fn assert_blank_row(t: &Harness<TableProApp>, y: u16, w: u16) {
    assert_eq!(t.row(y), " ".repeat(w as usize), "row {y} blank");
}

/// T1: notice block matches the oracle at 60x15.
#[test]
fn too_small_notice_matches_oracle_at_60x15() {
    let t = Harness::new(TableProApp::default(), Theme::junie(), 60, 15);
    assert_eq!(
        t.row(5),
        format!("{}TablePro{}", " ".repeat(26), " ".repeat(26)),
        "row 5 product"
    );
    assert_eq!(
        t.row(6),
        format!("{}Terminal too small{}", " ".repeat(21), " ".repeat(21)),
        "row 6 detail"
    );
    assert_eq!(
        t.row(7),
        format!(
            "{}Need 72\u{00d7}20, have 60\u{00d7}15{}",
            " ".repeat(19),
            " ".repeat(19)
        ),
        "row 7 dimensions"
    );
    assert_blank_row(&t, 8, 60);
    assert_eq!(
        t.row(9),
        format!("{}q Quit{}", " ".repeat(27), " ".repeat(27)),
        "row 9 quit"
    );
    assert_cell(&t, 26, 5, "T", WHITE, BLACK, 1);
    assert_cell(&t, 21, 6, "T", SECONDARY, BLACK, 0);
    assert_cell(&t, 19, 7, "N", MUTED, BLACK, 0);
    assert_cell(&t, 27, 9, "q", FAINT, BLACK, 0);
    assert_cell(&t, 0, 0, " ", WHITE, BLACK, 0);
    assert_cell(&t, 30, 8, " ", WHITE, BLACK, 0);
    assert_cell(&t, 59, 14, " ", WHITE, BLACK, 0);
    for y in 0..5 {
        assert_blank_row(&t, y, 60);
    }
    for y in 10..15 {
        assert_blank_row(&t, y, 60);
    }
}

/// T2: at exactly the minimum size the notice is not shown.
#[test]
fn at_exact_minimum_the_notice_is_not_shown() {
    let t = Harness::new(TableProApp::default(), Theme::junie(), 72, 20);
    assert!(
        !t.text().contains("Terminal too small"),
        "notice must not trigger at exactly 72x20"
    );
}

/// T3: resize ladder enters and leaves the notice.
#[test]
fn resize_ladder_enters_and_leaves_the_notice() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    assert!(
        !t.text().contains("Terminal too small"),
        "no notice at 120x40"
    );
    let _ = t.resize(60, 15);
    assert!(
        t.text().contains("Terminal too small"),
        "notice appears at 60x15"
    );
    let _ = t.resize(120, 40);
    assert!(
        !t.text().contains("Terminal too small"),
        "notice gone after resize back to 120x40"
    );
}

/// T4: the hand-painted notice is gone from production code.
#[test]
fn hand_painted_notice_is_gone() {
    let src = std::fs::read_to_string(format!("{}/src/app.rs", env!("CARGO_MANIFEST_DIR")))
        .expect("read tablepro-ui src/app.rs");
    assert!(
        !src.contains("fn draw_too_small"),
        "hand-painted draw_too_small must be deleted"
    );
    assert!(
        !src.contains("draw_too_small("),
        "draw_too_small call site must be deleted"
    );
}
