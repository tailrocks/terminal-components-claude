//! WI-TABLEPRO-FILTER-CHIPS commit 2: the table filter chips row paints
//! through stock `ChipBar` (lead + chips + overflow + add) instead of
//! `draw_filter_chips`, wired live per oracle TP-042.
//!
//! Hardcoded control bytes: unfocused windows are the pre-commit-2 dump at
//! the 1b tip (`/tmp/base.dump`, byte-identical to the fresh staged dump
//! `/tmp/fresh.dump` over all 9 configs); focused/toggled/pressed/hover
//! windows are the staged dumps (`/tmp/staged-dumps.txt`,
//! `/tmp/staged-dumps2.txt`). No test loads a frame file. Windows pin the
//! registered bar area exactly (83/65/73 cells); the panel border cell past
//! it is panel-owned and excluded.
//!
//! * T1 pins the 120x40 unfocused window. T1b pins the focused window
//!   (truecolor + Mono ▎); it is split out because focus needs the live
//!   component, hence red-on-base, while T1 stays a green guard.
//! * T2 pins 256/16/Mono windows; the shared symbols const is the nocolor
//!   gate (no `ColorLevel` below Mono exists; symbols are level-independent).
//! * T3 pins stock ownership: CHIP CONTAINER-fg / LEAD-fg / NEW-fg recipe
//!   overrides repaint chips / lead / add (all FAIL on base — legacy
//!   `paint_patch` bypasses recipes). Deviation: the plan's CONTAINER-bg
//!   leg cannot repaint by design (the M2 instance bg-Overlay patch wins
//!   over recipe bg via `style_patched`, proven by probe leg H); fg is the
//!   recipe-owned CONTAINER channel, and a guard pins the bg no-op.
//! * T4a-i pin the six wired actions, each with its real `reload()` effect.
//! * T5 pins the focus inventory + registered geometry (one stop, lead/add
//!   are parts, flow positions incl the post-1b add x).
//! * T6a pins 3-filter overflow bytes + hidden add at 120x40 + 72x20.
//! * T6b pins X-clear: row gone, grid shifted up 2.
//! * T7 pins changed input: edit-apply relabel, middle-removal cursor
//!   determinism, edit-changes-key liveness.
//! * T8a-d pin the accepted stock deltas. T8d guards post-click bytes on
//!   the modal-free vectors (lead/gap click → S9b-shape, never Black).
//!   Chip-body click opens the editor, so it cannot settle on the row;
//!   that path additionally exhibits stuck Black+BOLD after Esc (runtime
//!   pressed-state finding, filed separately — never pinned here).
//! * T9 pins 72x20 + 80x24 windows. T10 pins Mono bytes + mods==0.
//! * T11 pins empty diagnostics across the key + coordinate-pointer +
//!   overflow flow. The modal-edit legs live in T7a/T7c instead: id-based
//!   addressing of the unopened editor is noisy-on-base by design
//!   (`UnaddressableId`), which would flip this guard red.
//!
//! T1b/T3/T4/T5/T6b/T7/T8 are the red-on-base discriminators (verified
//! against a pristine `git archive` base copy); T1/T2/T6a/T9/T10/T11 are
//! green-on-base parity guards. No test reads `chips_state`, so every test
//! compiles against the base tree.

use tablepro_ui::filter_editor::FILTER_VALUE;
use tablepro_ui::{Screen, TableProApp, filter_editor::FILTER_EDITOR};
use termrock::{
    Color, ColorLevel, Family, Id, ItemKey, KeyCode, KeyModifiers, Part, PartRef, Role, StylePatch,
    Theme,
};
use termrock_test_support::{Harness, harness_typed_input};

const WHITE: Color = Color::Rgb(255, 255, 255);
const BLACK: Color = Color::Rgb(0, 0, 0);
const MUTED: Color = Color::Rgb(128, 128, 128);
const SECOND: Color = Color::Rgb(179, 179, 179);
const OVERLAY: Color = Color::Rgb(39, 39, 42);
const FOCUS: Color = Color::Rgb(72, 224, 84);
const HOVER_BG: Color = Color::Rgb(24, 24, 27);
const FAINT: Color = Color::Rgb(77, 77, 77);
const INFO: Color = Color::Rgb(135, 135, 255);

const BOLD: u16 = 0x0001;

/// One style run: (start column, cell count, fg, bg, mods).
type StyleRun = (u16, u16, Color, Color, u16);

/// Symbols of row `y` over `x0..x0+w`, one `String` per cell joined.
fn row_span(t: &Harness<TableProApp>, y: u16, x0: u16, w: u16) -> String {
    (x0..x0 + w).map(|x| t.cell(x, y).symbol()).collect()
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

/// Pin one chips-row window cell-for-cell. The coverage assert keeps the
/// pin honest: every cell must be asserted.
fn assert_row(t: &Harness<TableProApp>, x: u16, y: u16, syms: &str, runs: &[StyleRun]) {
    let chars: Vec<char> = syms.chars().collect();
    let w = chars.len() as u16;
    assert_eq!(row_span(t, y, x, w), syms, "row {y} symbols");
    let mut covered = 0u16;
    for (start, len, fg, bg, mods) in runs {
        for dx in *start..(*start + *len) {
            let mut buf = [0u8; 4];
            let sym = chars[dx as usize].encode_utf8(&mut buf);
            assert_cell(t, x + dx, y, sym, *fg, *bg, *mods);
        }
        covered += len;
    }
    assert_eq!(covered, w, "row {y} runs must cover every cell");
}

/// Drive the app to the filtered (1) state. `connect(4)` runs BEFORE
/// `Harness::new` (post-wrap connect breaks the explorer drive).
fn drive_filtered(level: ColorLevel, w: u16, h: u16) -> Harness<TableProApp> {
    let mut app = TableProApp::default();
    assert!(app.connect(4), "connect(4) failed");
    assert_eq!(app.screen, Screen::Workbench);
    let mut t = Harness::new(app, Theme::junie(), w, h).with_color(level);
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    let _ = t.key(KeyCode::Home);
    for _ in 0..4 {
        let _ = t.key(KeyCode::Right);
    }
    let _ = t.key(KeyCode::Char('f'));
    let _ = t.key(KeyCode::BackTab);
    let _ = t.key(KeyCode::BackTab);
    let _ = t.key(KeyCode::Enter);
    let _ = t.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    for c in "pending".chars() {
        let _ = t.key(KeyCode::Char(c));
    }
    let _ = t.key(KeyCode::Enter);
    t.draw();
    assert!(
        t.text().contains("filtered (1)"),
        "not filtered: {}",
        t.text()
    );
    t
}

/// Drive to 3 filters (second and third via the `f` push path on the
/// first two grid columns).
fn drive_3(level: ColorLevel, w: u16, h: u16) -> Harness<TableProApp> {
    let mut t = drive_filtered(level, w, h);
    for (rights, word) in [(0usize, "shipped"), (1usize, "zero")] {
        let _ = t.key(KeyCode::Home);
        for _ in 0..rights {
            let _ = t.key(KeyCode::Right);
        }
        let _ = t.key(KeyCode::Char('f'));
        let _ = t.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
        for c in word.chars() {
            let _ = t.key(KeyCode::Char(c));
        }
        let _ = t.key(KeyCode::Enter);
        t.draw();
    }
    assert_eq!(t.app().workbench.active_table().unwrap().filters.len(), 3);
    t
}

/// Find the chips row by scanning CELLS for the lead (byte index != cell
/// index with multibyte symbols). Handles both lead labels.
fn find_chips_row(t: &Harness<TableProApp>, w: u16, h: u16) -> (u16, u16) {
    for y in 0..h {
        let cells: Vec<String> = (0..w).map(|x| t.cell(x, y).symbol().to_string()).collect();
        for x in 0..w as usize {
            let word: String = cells[x..(x + 9).min(w as usize)].concat();
            if word == "match all" || word == "match any" {
                return (y, (x as u16).saturating_sub(1));
            }
        }
    }
    panic!("no chips row:\n{}", t.text());
}

fn bar_id(t: &Harness<TableProApp>) -> Id {
    t.app().workbench.active_key().unwrap().control("filters")
}

fn chip_key(t: &Harness<TableProApp>, i: usize) -> ItemKey {
    let label = t.app().workbench.active_table().unwrap().filters[i].chip_label();
    ItemKey::text(&label)
}

fn filter_labels(t: &Harness<TableProApp>) -> Vec<String> {
    t.app()
        .workbench
        .active_table()
        .unwrap()
        .filters
        .iter()
        .map(|f| f.chip_label())
        .collect()
}

/// The grid header line (present in filtered and unfiltered states).
fn header_text(t: &Harness<TableProApp>) -> String {
    t.find_row("rows 1").map(|y| t.row(y)).unwrap_or_default()
}

/// Focus the bar per the S10b/S13 technique; the bool is asserted only by
/// T5 — every other test pins outcomes, so a base run fails on behavior.
fn focus_bar(t: &mut Harness<TableProApp>) {
    let bar = bar_id(t);
    let _ = t.tab_to(bar);
    t.draw();
}

/// Edit the first chip's value to `word` through the real modal path:
/// Enter on the chip, click into the value field (Tab-in does not begin
/// editing there), clear, type, Enter to apply.
fn apply_value_edit(t: &mut Harness<TableProApp>, word: &str) {
    focus_bar(t);
    let _ = t.key(KeyCode::Enter);
    assert!(t.is_open(FILTER_EDITOR), "Enter opens the editor");
    assert_eq!(t.app().filter_editor.as_ref().unwrap().index, Some(0));
    let _ = t.click_id(FILTER_VALUE);
    let _ = t.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    for c in word.chars() {
        let _ = t.key(KeyCode::Char(c));
    }
    let _ = t.key(KeyCode::Enter);
    assert!(!t.is_open(FILTER_EDITOR), "Enter applies and closes");
}

// 120x40 unfocused symbols (83 cells: lead 13 + gap + gutter + label 19 +
// × + pads 2 + gap + add gutter + add text 13 + tail 31).
const SYMS_120: &str =
    " match all ▾   status = 'pending' ×    + Add filter                                ";
// Focused symbols: the gutter cell wears ▎.
const SYMS_120_FOCUSED: &str =
    " match all ▾  ▎status = 'pending' ×    + Add filter                                ";
// Post-lead-click symbols: flipped lead + focused chip.
const SYMS_120_ANY_FOCUSED: &str =
    " match any ▾  ▎status = 'pending' ×    + Add filter                                ";
// 72x20 unfocused symbols (65 cells, tail 13).
const SYMS_72: &str = " match all ▾   status = 'pending' ×    + Add filter              ";
// 80x24 unfocused symbols (73 cells, tail 21).
const SYMS_80: &str = " match all ▾   status = 'pending' ×    + Add filter                      ";
// 120x40 3-filter symbols (83 cells: chip + … + tail 44, add hidden).
const SYMS_120_3F: &str =
    " match all ▾   status = 'pending' ×   …                                            ";
// 72x20 3-filter symbols (65 cells, tail 26, add hidden).
const SYMS_72_3F: &str = " match all ▾   status = 'pending' ×   …                          ";

/// T1 unfocused 120x40 truecolor (`/tmp/base.dump` leg 1, border excluded).
const RUNS_120_TC: &[StyleRun] = &[
    (0, 13, MUTED, BLACK, 0x0000),
    (13, 1, WHITE, BLACK, 0x0000),
    (14, 1, OVERLAY, OVERLAY, 0x0000),
    (15, 19, WHITE, OVERLAY, 0x0000),
    (34, 1, MUTED, OVERLAY, 0x0000),
    (35, 2, WHITE, OVERLAY, 0x0000),
    (37, 1, WHITE, BLACK, 0x0000),
    (38, 1, BLACK, BLACK, 0x0000),
    (39, 13, SECOND, BLACK, 0x0000),
    (52, 31, WHITE, BLACK, 0x0000),
];

/// T1 focused 120x40 truecolor (staged leg A): ▎ Focus-fg + BOLD chip.
const RUNS_120_FOCUSED_TC: &[StyleRun] = &[
    (0, 13, MUTED, BLACK, 0x0000),
    (13, 1, WHITE, BLACK, 0x0000),
    (14, 1, FOCUS, OVERLAY, BOLD),
    (15, 19, WHITE, OVERLAY, BOLD),
    (34, 1, MUTED, OVERLAY, BOLD),
    (35, 2, WHITE, OVERLAY, BOLD),
    (37, 1, WHITE, BLACK, 0x0000),
    (38, 1, BLACK, BLACK, 0x0000),
    (39, 13, SECOND, BLACK, 0x0000),
    (52, 31, WHITE, BLACK, 0x0000),
];

/// T1 focused 120x40 Mono (staged leg B): ▎ Gray + BOLD.
const RUNS_120_FOCUSED_MONO: &[StyleRun] = &[
    (0, 13, Color::Gray, Color::Black, 0x0000),
    (13, 1, Color::White, Color::Black, 0x0000),
    (14, 1, Color::Gray, Color::Black, BOLD),
    (15, 19, Color::White, Color::Black, BOLD),
    (34, 1, Color::Gray, Color::Black, BOLD),
    (35, 2, Color::White, Color::Black, BOLD),
    (37, 1, Color::White, Color::Black, 0x0000),
    (38, 1, Color::Black, Color::Black, 0x0000),
    (39, 13, Color::Gray, Color::Black, 0x0000),
    (52, 31, Color::White, Color::Black, 0x0000),
];

/// T2 120x40 256 (`/tmp/base.dump` leg 2).
const RUNS_120_256: &[StyleRun] = &[
    (0, 13, Color::Indexed(244), Color::Indexed(16), 0x0000),
    (13, 1, Color::Indexed(231), Color::Indexed(16), 0x0000),
    (14, 1, Color::Indexed(235), Color::Indexed(235), 0x0000),
    (15, 19, Color::Indexed(231), Color::Indexed(235), 0x0000),
    (34, 1, Color::Indexed(244), Color::Indexed(235), 0x0000),
    (35, 2, Color::Indexed(231), Color::Indexed(235), 0x0000),
    (37, 1, Color::Indexed(231), Color::Indexed(16), 0x0000),
    (38, 1, Color::Indexed(16), Color::Indexed(16), 0x0000),
    (39, 13, Color::Indexed(249), Color::Indexed(16), 0x0000),
    (52, 31, Color::Indexed(231), Color::Indexed(16), 0x0000),
];

/// T2 120x40 Ansi16 (`/tmp/base.dump` leg 3).
const RUNS_120_16: &[StyleRun] = &[
    (0, 13, Color::Gray, Color::Black, 0x0000),
    (13, 1, Color::White, Color::Black, 0x0000),
    (14, 1, Color::DarkGray, Color::DarkGray, 0x0000),
    (15, 19, Color::White, Color::DarkGray, 0x0000),
    (34, 1, Color::Gray, Color::DarkGray, 0x0000),
    (35, 2, Color::White, Color::DarkGray, 0x0000),
    (37, 1, Color::White, Color::Black, 0x0000),
    (38, 1, Color::Black, Color::Black, 0x0000),
    (39, 13, Color::Gray, Color::Black, 0x0000),
    (52, 31, Color::White, Color::Black, 0x0000),
];

/// T2/T10 120x40 Mono (`/tmp/base.dump` leg 4; the pads+gap run is split
/// for structural parallelism — identical bytes).
const RUNS_120_MONO: &[StyleRun] = &[
    (0, 13, Color::Gray, Color::Black, 0x0000),
    (13, 1, Color::White, Color::Black, 0x0000),
    (14, 1, Color::Black, Color::Black, 0x0000),
    (15, 19, Color::White, Color::Black, 0x0000),
    (34, 1, Color::Gray, Color::Black, 0x0000),
    (35, 2, Color::White, Color::Black, 0x0000),
    (37, 1, Color::White, Color::Black, 0x0000),
    (38, 1, Color::Black, Color::Black, 0x0000),
    (39, 13, Color::Gray, Color::Black, 0x0000),
    (52, 31, Color::White, Color::Black, 0x0000),
];

/// T9 72x20 truecolor (`/tmp/base.dump` leg 5).
const RUNS_72_TC: &[StyleRun] = &[
    (0, 13, MUTED, BLACK, 0x0000),
    (13, 1, WHITE, BLACK, 0x0000),
    (14, 1, OVERLAY, OVERLAY, 0x0000),
    (15, 19, WHITE, OVERLAY, 0x0000),
    (34, 1, MUTED, OVERLAY, 0x0000),
    (35, 2, WHITE, OVERLAY, 0x0000),
    (37, 1, WHITE, BLACK, 0x0000),
    (38, 1, BLACK, BLACK, 0x0000),
    (39, 13, SECOND, BLACK, 0x0000),
    (52, 13, WHITE, BLACK, 0x0000),
];

/// T9 72x20 Mono (`/tmp/base.dump` leg 6).
const RUNS_72_MONO: &[StyleRun] = &[
    (0, 13, Color::Gray, Color::Black, 0x0000),
    (13, 1, Color::White, Color::Black, 0x0000),
    (14, 1, Color::Black, Color::Black, 0x0000),
    (15, 19, Color::White, Color::Black, 0x0000),
    (34, 1, Color::Gray, Color::Black, 0x0000),
    (35, 2, Color::White, Color::Black, 0x0000),
    (37, 1, Color::White, Color::Black, 0x0000),
    (38, 1, Color::Black, Color::Black, 0x0000),
    (39, 13, Color::Gray, Color::Black, 0x0000),
    (52, 13, Color::White, Color::Black, 0x0000),
];

/// T9 80x24 truecolor (`/tmp/base.dump` leg 7).
const RUNS_80_TC: &[StyleRun] = &[
    (0, 13, MUTED, BLACK, 0x0000),
    (13, 1, WHITE, BLACK, 0x0000),
    (14, 1, OVERLAY, OVERLAY, 0x0000),
    (15, 19, WHITE, OVERLAY, 0x0000),
    (34, 1, MUTED, OVERLAY, 0x0000),
    (35, 2, WHITE, OVERLAY, 0x0000),
    (37, 1, WHITE, BLACK, 0x0000),
    (38, 1, BLACK, BLACK, 0x0000),
    (39, 13, SECOND, BLACK, 0x0000),
    (52, 21, WHITE, BLACK, 0x0000),
];

/// T9 80x24 Mono (`/tmp/base.dump` leg 8).
const RUNS_80_MONO: &[StyleRun] = &[
    (0, 13, Color::Gray, Color::Black, 0x0000),
    (13, 1, Color::White, Color::Black, 0x0000),
    (14, 1, Color::Black, Color::Black, 0x0000),
    (15, 19, Color::White, Color::Black, 0x0000),
    (34, 1, Color::Gray, Color::Black, 0x0000),
    (35, 2, Color::White, Color::Black, 0x0000),
    (37, 1, Color::White, Color::Black, 0x0000),
    (38, 1, Color::Black, Color::Black, 0x0000),
    (39, 13, Color::Gray, Color::Black, 0x0000),
    (52, 21, Color::White, Color::Black, 0x0000),
];

/// T4e toggled 120x40 truecolor (staged leg C): faint label + muted ×,
/// still focused (BOLD); the gutter wears ▎, never ✓ (checked stays empty).
const RUNS_120_TOGGLED: &[StyleRun] = &[
    (0, 13, MUTED, BLACK, 0x0000),
    (13, 1, WHITE, BLACK, 0x0000),
    (14, 1, FOCUS, OVERLAY, BOLD),
    (15, 19, FAINT, OVERLAY, BOLD),
    (34, 1, MUTED, OVERLAY, BOLD),
    (35, 2, WHITE, OVERLAY, BOLD),
    (37, 1, WHITE, BLACK, 0x0000),
    (38, 1, BLACK, BLACK, 0x0000),
    (39, 13, SECOND, BLACK, 0x0000),
    (52, 31, WHITE, BLACK, 0x0000),
];

/// T8a pressed 120x40 truecolor (staged leg D): label + pads flash
/// Black-on-Overlay+BOLD (accepted transient delta vs oracle
/// canvas-on-primary); × and ▎ hold.
const RUNS_120_PRESSED: &[StyleRun] = &[
    (0, 13, MUTED, BLACK, 0x0000),
    (13, 1, WHITE, BLACK, 0x0000),
    (14, 1, FOCUS, OVERLAY, BOLD),
    (15, 19, BLACK, OVERLAY, BOLD),
    (34, 1, MUTED, OVERLAY, BOLD),
    (35, 2, BLACK, OVERLAY, BOLD),
    (37, 1, WHITE, BLACK, 0x0000),
    (38, 1, BLACK, BLACK, 0x0000),
    (39, 13, SECOND, BLACK, 0x0000),
    (52, 31, WHITE, BLACK, 0x0000),
];

/// T8c hovered-lead 120x40 truecolor (staged leg E2): the lead lifts to
/// Primary-on-HoverSurface; the rest of the row holds.
const RUNS_120_HOVER_LEAD: &[StyleRun] = &[
    (0, 13, WHITE, HOVER_BG, 0x0000),
    (13, 1, WHITE, BLACK, 0x0000),
    (14, 1, OVERLAY, OVERLAY, 0x0000),
    (15, 19, WHITE, OVERLAY, 0x0000),
    (34, 1, MUTED, OVERLAY, 0x0000),
    (35, 2, WHITE, OVERLAY, 0x0000),
    (37, 1, WHITE, BLACK, 0x0000),
    (38, 1, BLACK, BLACK, 0x0000),
    (39, 13, SECOND, BLACK, 0x0000),
    (52, 31, WHITE, BLACK, 0x0000),
];

/// T8c hovered-add 120x40 truecolor (staged leg E2): gutter + text lift.
const RUNS_120_HOVER_ADD: &[StyleRun] = &[
    (0, 13, MUTED, BLACK, 0x0000),
    (13, 1, WHITE, BLACK, 0x0000),
    (14, 1, OVERLAY, OVERLAY, 0x0000),
    (15, 19, WHITE, OVERLAY, 0x0000),
    (34, 1, MUTED, OVERLAY, 0x0000),
    (35, 2, WHITE, OVERLAY, 0x0000),
    (37, 1, WHITE, BLACK, 0x0000),
    (38, 1, HOVER_BG, HOVER_BG, 0x0000),
    (39, 13, WHITE, HOVER_BG, 0x0000),
    (52, 31, WHITE, BLACK, 0x0000),
];

/// T6a 120x40 3-filter truecolor (`/tmp/base.dump` leg 9): one chip + …
/// Muted-on-Canvas, add hidden.
const RUNS_120_3F: &[StyleRun] = &[
    (0, 13, MUTED, BLACK, 0x0000),
    (13, 1, WHITE, BLACK, 0x0000),
    (14, 1, OVERLAY, OVERLAY, 0x0000),
    (15, 19, WHITE, OVERLAY, 0x0000),
    (34, 1, MUTED, OVERLAY, 0x0000),
    (35, 2, WHITE, OVERLAY, 0x0000),
    (37, 1, WHITE, BLACK, 0x0000),
    (38, 1, MUTED, BLACK, 0x0000),
    (39, 44, WHITE, BLACK, 0x0000),
];

/// T6a 72x20 3-filter truecolor (staged leg F).
const RUNS_72_3F: &[StyleRun] = &[
    (0, 13, MUTED, BLACK, 0x0000),
    (13, 1, WHITE, BLACK, 0x0000),
    (14, 1, OVERLAY, OVERLAY, 0x0000),
    (15, 19, WHITE, OVERLAY, 0x0000),
    (34, 1, MUTED, OVERLAY, 0x0000),
    (35, 2, WHITE, OVERLAY, 0x0000),
    (37, 1, WHITE, BLACK, 0x0000),
    (38, 1, MUTED, BLACK, 0x0000),
    (39, 26, WHITE, BLACK, 0x0000),
];

/// T1: filtered window 120x40 truecolor, cell-for-cell incl gaps/tail.
#[test]
fn chips_filtered_window_120() {
    let t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_eq!((y, x0), (8, 34), "chips row position");
    assert_row(&t, x0, y, SYMS_120, RUNS_120_TC);
}

/// T1b: focused window 120x40 (truecolor + Mono), pinning the ▎ MARKER
/// tone. Split from T1: focus requires the component, so this leg is
/// red-on-base while T1 stays a green guard.
#[test]
fn chips_focused_window_120() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_row(&t, x0, y, SYMS_120_FOCUSED, RUNS_120_FOCUSED_TC);
    assert_cell(&t, x0 + 14, y, "▎", FOCUS, OVERLAY, BOLD);

    let mut t = drive_filtered(ColorLevel::Mono, 120, 40);
    focus_bar(&mut t);
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_row(&t, x0, y, SYMS_120_FOCUSED, RUNS_120_FOCUSED_MONO);
}

/// T2: 256/16/Mono windows. The shared symbols const is the nocolor gate:
/// symbols are level-independent and identical at every level.
#[test]
fn chips_levels() {
    let t = drive_filtered(ColorLevel::Ansi256, 120, 40);
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_eq!((y, x0), (8, 34));
    assert_row(&t, x0, y, SYMS_120, RUNS_120_256);

    let t = drive_filtered(ColorLevel::Ansi16, 120, 40);
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_row(&t, x0, y, SYMS_120, RUNS_120_16);

    let t = drive_filtered(ColorLevel::Mono, 120, 40);
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_row(&t, x0, y, SYMS_120, RUNS_120_MONO);
}

/// Drive the filtered (1) state under an explicit theme (T3 overrides).
fn drive_themed(theme: Theme) -> Harness<TableProApp> {
    let mut app = TableProApp::default();
    assert!(app.connect(4));
    let mut t = Harness::new(app, theme, 120, 40);
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    let _ = t.key(KeyCode::Home);
    for _ in 0..4 {
        let _ = t.key(KeyCode::Right);
    }
    let _ = t.key(KeyCode::Char('f'));
    let _ = t.key(KeyCode::BackTab);
    let _ = t.key(KeyCode::BackTab);
    let _ = t.key(KeyCode::Enter);
    let _ = t.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    for c in "pending".chars() {
        let _ = t.key(KeyCode::Char(c));
    }
    let _ = t.key(KeyCode::Enter);
    t.draw();
    assert!(
        t.text().contains("filtered (1)"),
        "not filtered: {}",
        t.text()
    );
    t
}

/// T3: stock ownership — CHIP recipe overrides repaint the row. All three
/// legs FAIL on base (legacy `paint_patch` bypasses recipes).
#[test]
fn chips_ownership() {
    // (a) CONTAINER-fg override repaints the chip label + pads (the
    // recipe-owned CONTAINER channel; bg is instance-owned, see guard).
    let t = drive_themed(Theme::junie().override_family(Family::CHIP, |r| {
        r.part(Part::CONTAINER)
            .base(StylePatch::new().set_fg(Role::Info));
    }));
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_eq!(t.cell(x0 + 16, y).fg, INFO, "CONTAINER-fg repaints label");
    assert_eq!(t.cell(x0 + 35, y).fg, INFO, "CONTAINER-fg repaints pads");
    assert_eq!(
        t.cell(x0 + 34, y).fg,
        MUTED,
        "instance CLOSE-fg still wins on ×"
    );
    assert_eq!(t.cell(x0 + 14, y).fg, OVERLAY, "gutter keeps fg-from-bg");
    // Guard: CONTAINER-bg override is a no-op (the M2 instance bg-Overlay
    // patch wins over recipe bg by design) — documents why leg (a) uses fg.
    let t = drive_themed(Theme::junie().override_family(Family::CHIP, |r| {
        r.part(Part::CONTAINER)
            .base(StylePatch::new().set_bg(Role::Info));
    }));
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_eq!(t.cell(x0 + 16, y).fg, WHITE, "CONTAINER-bg: label holds");
    assert_eq!(t.cell(x0 + 35, y).bg, OVERLAY, "CONTAINER-bg: pads hold");
    // (b) LEAD-fg override repaints the lead only.
    let t = drive_themed(Theme::junie().override_family(Family::CHIP, |r| {
        r.part(Part::LEAD)
            .base(StylePatch::new().set_fg(Role::Info));
    }));
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_eq!(t.cell(x0 + 2, y).fg, INFO, "LEAD-fg repaints lead");
    assert_eq!(t.cell(x0 + 16, y).fg, WHITE, "LEAD-fg: chip holds");
    // (c) NEW-fg override repaints the add only.
    let t = drive_themed(Theme::junie().override_family(Family::CHIP, |r| {
        r.part(Part::NEW).base(StylePatch::new().set_fg(Role::Info));
    }));
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_eq!(t.cell(x0 + 40, y).fg, INFO, "NEW-fg repaints add");
    assert_eq!(t.cell(x0 + 2, y).fg, MUTED, "NEW-fg: lead holds");
}

/// T4a: Delete removes the cursor chip + reloads + status.
#[test]
fn chips_delete_removes() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Delete);
    assert!(filter_labels(&t).is_empty(), "chip removed");
    assert_eq!(t.app().status(), "Filter removed");
    assert!(t.find("match all").is_none(), "row gone");
    assert!(!t.is_open(FILTER_EDITOR), "no editor");
    let header = header_text(&t);
    assert!(header.contains("rows 1–27 of 500"), "reload: {header}");
    assert!(!header.contains("filtered"), "unfiltered: {header}");
}

/// T4b: click-× removes (Closed only — never Activated).
#[test]
fn chips_click_close_removes() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let bar = bar_id(&t);
    let k = chip_key(&t, 0);
    let _ = t.click_part(bar, PartRef::item(Part::CLOSE, k));
    assert!(filter_labels(&t).is_empty(), "chip removed");
    assert_eq!(t.app().status(), "Filter removed");
    assert!(!t.is_open(FILTER_EDITOR), "× never activates");
}

/// T4c: click-chip opens the editor on that index.
#[test]
fn chips_click_chip_edits() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let bar = bar_id(&t);
    let k = chip_key(&t, 0);
    let _ = t.click_part(bar, PartRef::item(Part::LABEL, k));
    assert!(t.is_open(FILTER_EDITOR), "editor opens");
    assert_eq!(t.app().filter_editor.as_ref().unwrap().index, Some(0));
}

/// T4d: Enter opens the editor on the cursor chip.
#[test]
fn chips_enter_edits() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Enter);
    assert!(t.is_open(FILTER_EDITOR), "editor opens");
    assert_eq!(t.app().filter_editor.as_ref().unwrap().index, Some(0));
}

/// T4e: Space toggles enabled + faint bytes + reload (row stays).
#[test]
fn chips_space_toggles() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Char(' '));
    let tab = t.app().workbench.active_table().unwrap();
    assert_eq!(tab.filters.len(), 1, "row stays");
    assert!(!tab.filters[0].enabled, "enabled flips");
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_row(&t, x0, y, SYMS_120_FOCUSED, RUNS_120_TOGGLED);
    let header = header_text(&t);
    assert!(header.contains("rows 1–25 of 500"), "reload: {header}");
    assert!(!header.contains("filtered"), "zero enabled: {header}");
}

/// T4f: lead-click flips match_all (label + reload only — the query is
/// inert, matching oracle `select()`).
#[test]
fn chips_lead_flips_match() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let bar = bar_id(&t);
    let before = header_text(&t);
    let _ = t.click_part(bar, PartRef::of(Part::LEAD));
    assert!(!t.app().workbench.active_table().unwrap().match_all);
    let (y, x0) = find_chips_row(&t, 120, 40);
    let lead: String = (x0..x0 + 13).map(|x| t.cell(x, y).symbol()).collect();
    assert_eq!(lead, " match any ▾ ");
    assert_eq!(header_text(&t), before, "query inert");
}

/// T4g: add-click opens a blank editor.
#[test]
fn chips_add_opens_blank() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let bar = bar_id(&t);
    let _ = t.click_part(bar, PartRef::of(Part::NEW));
    assert!(t.is_open(FILTER_EDITOR), "editor opens");
    assert_eq!(t.app().filter_editor.as_ref().unwrap().index, None);
}

/// T4h: X clears all filters (row vanishes, no status change).
#[test]
fn chips_x_clears() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Char('X'));
    assert!(filter_labels(&t).is_empty(), "filters cleared");
    assert_eq!(t.app().status(), "1 filter applied", "no clear status");
    assert!(t.find("match all").is_none(), "row gone");
    let header = header_text(&t);
    assert!(header.contains("rows 1–27 of 500"), "reload: {header}");
}

/// T4i: + opens a blank editor.
#[test]
fn chips_plus_opens_blank() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Char('+'));
    assert!(t.is_open(FILTER_EDITOR), "editor opens");
    assert_eq!(t.app().filter_editor.as_ref().unwrap().index, None);
}

/// T5: focus inventory — one reachable stop, lead/add are parts (live but
/// never Tab stops), registered geometry incl the post-1b flow positions.
#[test]
fn chips_focus_inventory() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let bar = bar_id(&t);
    assert!(t.tab_to(bar), "bar is reachable");
    t.draw();
    assert_eq!(t.focus(), Some(bar));
    let (chips_y, _) = find_chips_row(&t, 120, 40);
    // The ring holds the bar exactly once; no other stop sits on the row.
    let entries = t.ring().entries();
    assert_eq!(
        entries.iter().filter(|e| e.id == bar).count(),
        1,
        "bar registered once"
    );
    for e in entries {
        if e.id != bar {
            assert_ne!(e.area.y, chips_y, "no second stop on the chips row");
        }
    }
    // Registered geometry + flow part positions (add x=72 is post-1b;
    // pre-1b it sat one cell left).
    let area = t.area_of(bar).expect("bar registered");
    assert_eq!((area.x, area.y, area.width, area.height), (34, 8, 83, 1));
    let lead = t
        .area_of_part(bar, PartRef::of(Part::LEAD))
        .expect("LEAD part");
    assert_eq!((lead.x, lead.y, lead.width, lead.height), (34, 8, 13, 1));
    let new = t
        .area_of_part(bar, PartRef::of(Part::NEW))
        .expect("NEW part");
    assert_eq!((new.x, new.y, new.width, new.height), (72, 8, 14, 1));
    // Tab away leaves the bar; a full cycle returns with no repeats
    // (the ring also holds a disabled stop Tab skips, so the cycle is
    // shorter than the entry list — cycle until return instead).
    let _ = t.key(KeyCode::Tab);
    assert_ne!(t.focus(), Some(bar), "Tab leaves the bar");
    assert!(t.tab_to(bar), "bar reachable again");
    let len = t.ring().entries().len();
    let mut seen = vec![];
    for _ in 0..len + 1 {
        let _ = t.key(KeyCode::Tab);
        if t.focus() == Some(bar) {
            break;
        }
        assert!(!seen.contains(&t.focus()), "no stop repeats in-cycle");
        seen.push(t.focus());
    }
    assert_eq!(t.focus(), Some(bar), "cycle returns");
}

/// T6a: 3-filter overflow — … bytes + add hidden, 120x40 + 72x20.
#[test]
fn chips_overflow() {
    let t = drive_3(ColorLevel::TrueColor, 120, 40);
    assert_eq!(
        filter_labels(&t),
        vec![
            "status = 'pending'".to_string(),
            "id = '9157cff3-d585-4ed1-b900-a90377df59dc'".to_string(),
            "order_number = '10001'".to_string(),
        ]
    );
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_row(&t, x0, y, SYMS_120_3F, RUNS_120_3F);
    assert_cell(&t, x0 + 38, y, "…", MUTED, BLACK, 0);
    assert!(t.find("+ Add").is_none(), "add hidden on overflow");

    let t = drive_3(ColorLevel::TrueColor, 72, 20);
    let (y, x0) = find_chips_row(&t, 72, 20);
    assert_eq!((y, x0), (8, 4));
    assert_row(&t, x0, y, SYMS_72_3F, RUNS_72_3F);
    assert!(t.find("+ Add").is_none(), "add hidden on overflow");
}

/// T6b: X-clear removes the row and the grid shifts up 2.
#[test]
fn chips_clear_shifts_grid() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let data = t.app().workbench.active_key().unwrap().control("data");
    let y0 = t.area_of(data).expect("grid registered").y;
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Char('X'));
    assert!(filter_labels(&t).is_empty(), "filters cleared");
    assert!(t.find("match all").is_none(), "row gone");
    let y1 = t.area_of(data).expect("grid registered").y;
    assert_eq!(y0 - y1, 2, "grid shifts up 2");
}

/// T7a: edit-apply updates the label in place (same-width value swap keeps
/// the × anchor: geometry stability).
#[test]
fn chips_edit_apply_relabels() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    apply_value_edit(&mut t, "shipped");
    assert_eq!(filter_labels(&t), vec!["status = 'shipped'".to_string()]);
    assert_eq!(t.app().status(), "1 filter applied");
    let (y, x0) = find_chips_row(&t, 120, 40);
    let value: String = ((x0 + 25)..(x0 + 32))
        .map(|x| t.cell(x, y).symbol())
        .collect();
    assert_eq!(value, "shipped");
    assert_cell(&t, x0 + 34, y, "×", MUTED, OVERLAY, 0);
    assert!(t.diagnostics().is_empty(), "modal flow drains cleanly");
}

/// T7b: middle-removal cursor determinism — delete B of A/B/C, then the
/// next Delete removes C (cursor falls forward, index-preserved), and the
/// bar stays live throughout.
#[test]
fn chips_middle_removal_cursor() {
    let mut t = drive_3(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Right);
    let _ = t.key(KeyCode::Delete);
    assert_eq!(
        filter_labels(&t),
        vec![
            "status = 'pending'".to_string(),
            "order_number = '10001'".to_string(),
        ],
        "B removed"
    );
    let _ = t.key(KeyCode::Delete);
    assert_eq!(
        filter_labels(&t),
        vec!["status = 'pending'".to_string()],
        "cursor fell forward to C"
    );
    assert_eq!(t.app().status(), "Filter removed");
}

/// T7c: edit-changes-key — after the label (hence key) changes, the bar is
/// settled and live: Enter routes to the edited chip, Delete removes it.
#[test]
fn chips_edit_key_change_live() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    apply_value_edit(&mut t, "shipped");
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Enter);
    assert!(t.is_open(FILTER_EDITOR), "routes to edited chip");
    assert_eq!(t.app().filter_editor.as_ref().unwrap().index, Some(0));
    let _ = t.key(KeyCode::Esc);
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Delete);
    assert!(filter_labels(&t).is_empty(), "edited chip removed");
    assert!(t.diagnostics().is_empty(), "modal flow drains cleanly");
}

/// T8a: pressed-flash bytes while Down is held on the chip.
#[test]
fn chips_pressed_flash() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    let (y, x0) = find_chips_row(&t, 120, 40);
    let _ = t.mouse(termrock::MouseKind::Down, x0 + 20, y);
    assert_row(&t, x0, y, SYMS_120_FOCUSED, RUNS_120_PRESSED);
    let _ = t.mouse(termrock::MouseKind::Up, x0 + 20, y);
}

/// T8b: Home/End move the cursor — End reaches the add stop (Enter opens a
/// blank editor), Home returns to the first chip.
#[test]
fn chips_home_end() {
    let mut t = drive_3(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    let _ = t.key(KeyCode::End);
    let _ = t.key(KeyCode::Enter);
    assert!(t.is_open(FILTER_EDITOR), "End+Enter opens blank");
    assert_eq!(t.app().filter_editor.as_ref().unwrap().index, None);
    let _ = t.key(KeyCode::Esc);
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Home);
    let _ = t.key(KeyCode::Enter);
    assert!(t.is_open(FILTER_EDITOR), "Home+Enter opens first");
    assert_eq!(t.app().filter_editor.as_ref().unwrap().index, Some(0));
    let _ = t.key(KeyCode::Esc);
}

/// T8c: hover bytes — lead/add lift; chip/× hold (no LABEL hover rule;
/// the instance CLOSE-fg patch beats the recipe HOVERED rule).
#[test]
fn chips_hover() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let (y, x0) = find_chips_row(&t, 120, 40);
    let _ = harness_typed_input::hover(&mut t, x0 + 2, y);
    assert_row(&t, x0, y, SYMS_120, RUNS_120_HOVER_LEAD);

    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let (y, x0) = find_chips_row(&t, 120, 40);
    let _ = harness_typed_input::hover(&mut t, x0 + 16, y);
    assert_row(&t, x0, y, SYMS_120, RUNS_120_TC);

    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let (y, x0) = find_chips_row(&t, 120, 40);
    let _ = harness_typed_input::hover(&mut t, x0 + 34, y);
    assert_row(&t, x0, y, SYMS_120, RUNS_120_TC);

    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let (y, x0) = find_chips_row(&t, 120, 40);
    let _ = harness_typed_input::hover(&mut t, x0 + 41, y);
    assert_row(&t, x0, y, SYMS_120, RUNS_120_HOVER_ADD);
    assert!(t.diagnostics().is_empty(), "hover drains cleanly");
}

/// T8d: post-click guard — the modal-free clicks settle on the S9b-shape
/// (White+BOLD, never stuck Black).
#[test]
fn chips_post_click_settles() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let bar = bar_id(&t);
    let _ = t.click_part(bar, PartRef::of(Part::LEAD));
    assert_eq!(t.focus(), Some(bar), "lead-click focuses the group");
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_row(&t, x0, y, SYMS_120_ANY_FOCUSED, RUNS_120_FOCUSED_TC);

    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    let bar = bar_id(&t);
    let (y, x0) = find_chips_row(&t, 120, 40);
    let _ = t.click(x0 + 13, y);
    assert_eq!(t.focus(), Some(bar), "gap-click focuses the group");
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_row(&t, x0, y, SYMS_120_FOCUSED, RUNS_120_FOCUSED_TC);
}

/// T9: small sizes, truecolor + Mono.
#[test]
fn chips_small_sizes() {
    let t = drive_filtered(ColorLevel::TrueColor, 72, 20);
    let (y, x0) = find_chips_row(&t, 72, 20);
    assert_eq!((y, x0), (8, 4));
    assert_row(&t, x0, y, SYMS_72, RUNS_72_TC);

    let t = drive_filtered(ColorLevel::Mono, 72, 20);
    let (y, x0) = find_chips_row(&t, 72, 20);
    assert_row(&t, x0, y, SYMS_72, RUNS_72_MONO);

    let t = drive_filtered(ColorLevel::TrueColor, 80, 24);
    let (y, x0) = find_chips_row(&t, 80, 24);
    assert_eq!((y, x0), (8, 4));
    assert_row(&t, x0, y, SYMS_80, RUNS_80_TC);

    let t = drive_filtered(ColorLevel::Mono, 80, 24);
    let (y, x0) = find_chips_row(&t, 80, 24);
    assert_row(&t, x0, y, SYMS_80, RUNS_80_MONO);
}

/// T10: Mono bytes + mods==0 on every row cell (B3 regression pin: no
/// disabled state anywhere, so DIM can never reach the row).
#[test]
fn chips_mono_clean() {
    let t = drive_filtered(ColorLevel::Mono, 120, 40);
    let (y, x0) = find_chips_row(&t, 120, 40);
    assert_row(&t, x0, y, SYMS_120, RUNS_120_MONO);
    for dx in 0..83 {
        assert_eq!(t.cell(x0 + dx, y).modifier.bits(), 0, "cell {dx} mods");
    }
}

/// T11: empty diagnostics across the key + coordinate-pointer + overflow
/// flow (modal-edit diagnostics are pinned in T7a/T7c instead).
#[test]
fn chips_diagnostics_empty() {
    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    assert!(t.diagnostics().is_empty(), "after focus");
    let _ = t.key(KeyCode::Char(' '));
    assert!(t.diagnostics().is_empty(), "after Space");
    let _ = t.key(KeyCode::Char(' '));
    assert!(t.diagnostics().is_empty(), "after Space back");
    let _ = t.key(KeyCode::Home);
    assert!(t.diagnostics().is_empty(), "after Home");
    let _ = t.key(KeyCode::End);
    assert!(t.diagnostics().is_empty(), "after End");
    let _ = t.key(KeyCode::Enter);
    assert!(t.diagnostics().is_empty(), "after Enter");
    let _ = t.key(KeyCode::Esc);
    assert!(t.diagnostics().is_empty(), "after Esc");
    let (y, x0) = find_chips_row(&t, 120, 40);
    let _ = t.click(x0 + 16, y);
    assert!(t.diagnostics().is_empty(), "after chip click");
    let _ = t.key(KeyCode::Esc);
    assert!(t.diagnostics().is_empty(), "after Esc");
    let (y, x0) = find_chips_row(&t, 120, 40);
    let _ = t.click(x0 + 2, y);
    assert!(t.diagnostics().is_empty(), "after lead click");
    let _ = t.click(x0 + 41, y);
    assert!(t.diagnostics().is_empty(), "after add click");
    let _ = t.key(KeyCode::Esc);
    assert!(t.diagnostics().is_empty(), "after Esc");
    for (dx, what) in [(2u16, "lead"), (16, "chip"), (34, "x"), (41, "add")] {
        let _ = harness_typed_input::hover(&mut t, x0 + dx, y);
        assert!(t.diagnostics().is_empty(), "after hover {what}");
    }

    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Delete);
    assert!(t.diagnostics().is_empty(), "after Delete");

    let mut t = drive_filtered(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Char('X'));
    assert!(t.diagnostics().is_empty(), "after X");

    let mut t = drive_3(ColorLevel::TrueColor, 120, 40);
    focus_bar(&mut t);
    let _ = t.key(KeyCode::Right);
    assert!(t.diagnostics().is_empty(), "after Right");
    let _ = t.key(KeyCode::Delete);
    assert!(t.diagnostics().is_empty(), "after overflow Delete");
    let _ = t.key(KeyCode::Enter);
    assert!(t.diagnostics().is_empty(), "after overflow Enter");
    let _ = t.key(KeyCode::Esc);
    assert!(t.diagnostics().is_empty(), "after overflow Esc");
}
