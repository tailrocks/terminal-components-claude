//! WI-TABLEPRO-FILTER-ROW: both filter rows paint through stock `Field`
//! chrome around `TextInput` instead of `paint_legacy_filter` plus the two
//! adjacent row fills.
//!
//! Hardcoded control bytes below are the pre-implementation dump at base
//! `5a4705a2d` (`/tmp/frdump`; Ansi16 ▎ cross-checked against oracle
//! `baselines/tuiscotti-v1/tablepro/connections/filter/120x40/16.frame.json`,
//! truecolor rows against `truecolor.frame.json`); no test loads a frame file.
//!
//! * T1 pins the 120x40 connections placeholder 2-row window cell-for-cell.
//! * T2 pins the focused "stag" window + flags + Staging-only tree.
//! * T3 pins stock ownership: INPUT TEXT / FIELD LABEL recipe overrides
//!   repaint the row (legacy `paint_patch` bypasses recipes, so both FAIL
//!   on base).
//! * T4 pins typing/Backspace/typing-Esc parity (editing-Esc only).
//! * T5 pins Tab-out commit + Tab-in same-frame re-begin.
//! * T6 pins Down into the tree.
//! * T7 pins the inert explorer row + its disabled (registered, unreachable)
//!   control, including the pointer-intent drain.
//! * T8a-h pin the accepted stock deltas (all FAIL on base).
//! * T9 pins both sites at 72x20 + 80x24.
//! * T10 pins the Ansi16 focused row (Focus≡Accent).
//! * T11 pins empty diagnostics after the full flow.
//! * T12 pins duplicate-during-draft: the tree stays on the draft query
//!   (base-consistent tree behavior; RED on staged pre-fix, where the
//!   duplicate rebuilt from the stale committed filter).
//!
//! T3/T7/T8 are the red-on-base discriminators;
//! T1/T2/T4/T5/T6/T9/T10/T11 are green-on-base parity guards.

use tablepro_ui::{Screen, TableProApp};
use termrock::{Color, ColorLevel, Family, Id, KeyCode, Part, Role, StateFlags, StylePatch, Theme};
use termrock_test_support::{Harness, harness_typed_input};

const FILTER_ID: Id = Id::root("tablepro.connections.filter");
const EXPLORER_FILTER_ID: Id = Id::root("tablepro.workbench.explorer.filter");
const CONNECTIONS_ID: Id = Id::root("tablepro.connections.list");

const WHITE: Color = Color::Rgb(255, 255, 255);
const BLACK: Color = Color::Rgb(0, 0, 0);
const SECOND: Color = Color::Rgb(179, 179, 179);
const MUTED: Color = Color::Rgb(128, 128, 128);
const FIELD_BG: Color = Color::Rgb(30, 30, 34);
const FIELD_HOVER: Color = Color::Rgb(35, 35, 40);
const INFO: Color = Color::Rgb(135, 135, 255);

const UNDERLINED: u16 = 0x0008;

/// One style run: (start column, cell count, fg, bg, mods).
type StyleRun = (u16, u16, Color, Color, u16);

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

/// Pin a 2-row filter window cell-for-cell: symbols plus the fg/bg/mods
/// run table. `fy` is the filter (control) row; the label row is `fy - 1`.
/// The coverage assert keeps the pin honest: every cell must be asserted.
fn assert_window(
    t: &Harness<TableProApp>,
    x: u16,
    fy: u16,
    syms: &[&str; 2],
    runs: &[&[StyleRun]; 2],
) {
    for (r, y) in [fy - 1, fy].into_iter().enumerate() {
        let chars: Vec<char> = syms[r].chars().collect();
        let w = chars.len() as u16;
        assert_eq!(row_span(t, y, x, x + w - 1), syms[r], "row {y} symbols");
        let mut covered = 0u16;
        for (start, len, fg, bg, mods) in runs[r] {
            for dx in *start..(*start + *len) {
                let mut buf = [0u8; 4];
                let sym = chars[dx as usize].encode_utf8(&mut buf);
                assert_cell(t, x + dx, y, sym, *fg, *bg, *mods);
            }
            covered += len;
        }
        assert_eq!(covered, w, "row {y} runs must cover every cell");
    }
}

/// Connections filter geometry from the live registration.
fn connections_geom(t: &Harness<TableProApp>) -> (u16, u16, u16) {
    let area = t.area_of(FILTER_ID).expect("filter registered");
    (area.x, area.y, area.width)
}

/// Explorer filter geometry from the live registration.
fn explorer_geom(t: &Harness<TableProApp>) -> (u16, u16, u16) {
    let area = t
        .area_of(EXPLORER_FILTER_ID)
        .expect("explorer filter registered");
    (area.x, area.y, area.width)
}

fn to_workbench(t: &mut Harness<TableProApp>) {
    for i in 0..10 {
        if t.app_mut().connect(i) && t.app().screen == Screen::Workbench {
            break;
        }
    }
    assert_eq!(t.app().screen, Screen::Workbench);
    t.draw();
}

fn focus_filter(t: &mut Harness<TableProApp>) {
    let _ = t.key(KeyCode::Char('/'));
    let _ = t.type_str("stag");
}

/// C120_IDLE: legacy bytes at base (x=3, filter row 4, w=35).
const C120_IDLE_GEOM: (u16, u16, u16) = (3, 4, 35);
const C120_IDLE_SYMS: [&str; 2] = [
    "                                   ",
    "  Filter connections               ",
];
const C120_IDLE_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), 0x0000),
        (
            2,
            33,
            Color::Rgb(179, 179, 179),
            Color::Rgb(0, 0, 0),
            0x0000,
        ),
    ],
    &[
        (0, 1, Color::Rgb(30, 30, 34), Color::Rgb(30, 30, 34), 0x0000),
        (
            1,
            1,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            2,
            18,
            Color::Rgb(128, 128, 128),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            20,
            15,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
    ],
];

/// C120_FOCUSED: legacy bytes at base (x=3, filter row 4, w=35).
const C120_FOCUSED_GEOM: (u16, u16, u16) = (3, 4, 35);
const C120_FOCUSED_SYMS: [&str; 2] = [
    "                                   ",
    "▎ stag                             ",
];
const C120_FOCUSED_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), 0x0000),
        (
            2,
            33,
            Color::Rgb(255, 255, 255),
            Color::Rgb(0, 0, 0),
            0x0001,
        ),
    ],
    &[
        (
            0,
            1,
            Color::Rgb(72, 224, 84),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            1,
            1,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            2,
            4,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0008,
        ),
        (
            6,
            29,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
    ],
];

/// C120_DOWN: legacy bytes at base (x=3, filter row 4, w=35).
const C120_DOWN_GEOM: (u16, u16, u16) = (3, 4, 35);
const C120_DOWN_SYMS: [&str; 2] = [
    "                                   ",
    "  stag                             ",
];
const C120_DOWN_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), 0x0000),
        (
            2,
            33,
            Color::Rgb(179, 179, 179),
            Color::Rgb(0, 0, 0),
            0x0000,
        ),
    ],
    &[
        (0, 1, Color::Rgb(30, 30, 34), Color::Rgb(30, 30, 34), 0x0000),
        (
            1,
            1,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            2,
            4,
            Color::Rgb(128, 128, 128),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            6,
            29,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
    ],
];

/// Stock F4 bytes: `C120_DOWN` with the 4 idle-value text cells Primary
/// (legacy paints Muted; the component owns TEXT). Red on base by design.
const IDLE_VALUE_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), 0x0000),
        (
            2,
            33,
            Color::Rgb(179, 179, 179),
            Color::Rgb(0, 0, 0),
            0x0000,
        ),
    ],
    &[
        (0, 1, Color::Rgb(30, 30, 34), Color::Rgb(30, 30, 34), 0x0000),
        (
            1,
            1,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            2,
            4,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            6,
            29,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
    ],
];

/// E120: legacy bytes at base (x=3, filter row 6, w=25).
const E120_GEOM: (u16, u16, u16) = (3, 6, 25);
const E120_SYMS: [&str; 2] = ["                         ", "  Filter objects         "];
const E120_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), 0x0000),
        (
            2,
            23,
            Color::Rgb(179, 179, 179),
            Color::Rgb(0, 0, 0),
            0x0000,
        ),
    ],
    &[
        (0, 1, Color::Rgb(30, 30, 34), Color::Rgb(30, 30, 34), 0x0000),
        (
            1,
            1,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            2,
            14,
            Color::Rgb(128, 128, 128),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            16,
            9,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
    ],
];

/// C72_IDLE: legacy bytes at base (x=3, filter row 4, w=66).
const C72_IDLE_GEOM: (u16, u16, u16) = (3, 4, 66);
const C72_IDLE_SYMS: [&str; 2] = [
    "                                                                  ",
    "  Filter connections                                              ",
];
const C72_IDLE_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), 0x0000),
        (
            2,
            64,
            Color::Rgb(179, 179, 179),
            Color::Rgb(0, 0, 0),
            0x0000,
        ),
    ],
    &[
        (0, 1, Color::Rgb(30, 30, 34), Color::Rgb(30, 30, 34), 0x0000),
        (
            1,
            1,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            2,
            18,
            Color::Rgb(128, 128, 128),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            20,
            46,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
    ],
];

/// C72_FOCUSED: legacy bytes at base (x=3, filter row 4, w=66).
const C72_FOCUSED_GEOM: (u16, u16, u16) = (3, 4, 66);
const C72_FOCUSED_SYMS: [&str; 2] = [
    "                                                                  ",
    "▎ stag                                                            ",
];
const C72_FOCUSED_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), 0x0000),
        (
            2,
            64,
            Color::Rgb(255, 255, 255),
            Color::Rgb(0, 0, 0),
            0x0001,
        ),
    ],
    &[
        (
            0,
            1,
            Color::Rgb(72, 224, 84),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            1,
            1,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            2,
            4,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0008,
        ),
        (
            6,
            60,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
    ],
];

/// E72: legacy bytes at base (x=3, filter row 6, w=66).
const E72_GEOM: (u16, u16, u16) = (3, 6, 66);
const E72_SYMS: [&str; 2] = [
    "                                                                  ",
    "  Filter objects                                                  ",
];
const E72_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), 0x0000),
        (
            2,
            64,
            Color::Rgb(179, 179, 179),
            Color::Rgb(0, 0, 0),
            0x0000,
        ),
    ],
    &[
        (0, 1, Color::Rgb(30, 30, 34), Color::Rgb(30, 30, 34), 0x0000),
        (
            1,
            1,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            2,
            14,
            Color::Rgb(128, 128, 128),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            16,
            50,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
    ],
];

/// C80_IDLE: legacy bytes at base (x=3, filter row 4, w=74).
const C80_IDLE_GEOM: (u16, u16, u16) = (3, 4, 74);
const C80_IDLE_SYMS: [&str; 2] = [
    "                                                                          ",
    "  Filter connections                                                      ",
];
const C80_IDLE_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), 0x0000),
        (
            2,
            72,
            Color::Rgb(179, 179, 179),
            Color::Rgb(0, 0, 0),
            0x0000,
        ),
    ],
    &[
        (0, 1, Color::Rgb(30, 30, 34), Color::Rgb(30, 30, 34), 0x0000),
        (
            1,
            1,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            2,
            18,
            Color::Rgb(128, 128, 128),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            20,
            54,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
    ],
];

/// C80_FOCUSED: legacy bytes at base (x=3, filter row 4, w=74).
const C80_FOCUSED_GEOM: (u16, u16, u16) = (3, 4, 74);
const C80_FOCUSED_SYMS: [&str; 2] = [
    "                                                                          ",
    "▎ stag                                                                    ",
];
const C80_FOCUSED_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), 0x0000),
        (
            2,
            72,
            Color::Rgb(255, 255, 255),
            Color::Rgb(0, 0, 0),
            0x0001,
        ),
    ],
    &[
        (
            0,
            1,
            Color::Rgb(72, 224, 84),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            1,
            1,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            2,
            4,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0008,
        ),
        (
            6,
            68,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
    ],
];

/// E80: legacy bytes at base (x=3, filter row 6, w=74).
const E80_GEOM: (u16, u16, u16) = (3, 6, 74);
const E80_SYMS: [&str; 2] = [
    "                                                                          ",
    "  Filter objects                                                          ",
];
const E80_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), 0x0000),
        (
            2,
            72,
            Color::Rgb(179, 179, 179),
            Color::Rgb(0, 0, 0),
            0x0000,
        ),
    ],
    &[
        (0, 1, Color::Rgb(30, 30, 34), Color::Rgb(30, 30, 34), 0x0000),
        (
            1,
            1,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            2,
            14,
            Color::Rgb(128, 128, 128),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
        (
            16,
            58,
            Color::Rgb(255, 255, 255),
            Color::Rgb(30, 30, 34),
            0x0000,
        ),
    ],
];

/// C120_FOCUSED_16: legacy bytes at base (x=3, filter row 4, w=35).
const C120_FOCUSED_16_GEOM: (u16, u16, u16) = (3, 4, 35);
const C120_FOCUSED_16_SYMS: [&str; 2] = [
    "                                   ",
    "▎ stag                             ",
];
const C120_FOCUSED_16_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::White, Color::Black, 0x0000),
        (2, 33, Color::White, Color::Black, 0x0001),
    ],
    &[
        (0, 1, Color::LightGreen, Color::Black, 0x0000),
        (1, 1, Color::White, Color::Black, 0x0000),
        (2, 4, Color::White, Color::Black, 0x0008),
        (6, 29, Color::White, Color::Black, 0x0000),
    ],
];

/// T1: connections placeholder 2-row window (120x40, boot).
#[test]
fn filter_row_connections_placeholder_120() {
    let t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let (ax, fy, w) = connections_geom(&t);
    assert_eq!((ax, fy, w), C120_IDLE_GEOM, "filter geometry");
    assert_window(&t, ax, fy, &C120_IDLE_SYMS, &C120_IDLE_RUNS);
}

/// T2: focused filter (send `/`, type `stag`).
#[test]
fn filter_row_connections_focused_120() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    focus_filter(&mut t);
    let (ax, fy, w) = connections_geom(&t);
    assert_eq!((ax, fy, w), C120_FOCUSED_GEOM, "filter geometry");
    assert_window(&t, ax, fy, &C120_FOCUSED_SYMS, &C120_FOCUSED_RUNS);
    assert_eq!(
        t.state_of(FILTER_ID),
        StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::EDITING,
        "focused filter flags"
    );
    assert!(t.find("Staging").is_some(), "tree shows Staging");
    assert!(t.find("Production").is_none(), "tree shows Staging only");
}

/// T3: stock ownership — INPUT TEXT / FIELD LABEL recipe overrides repaint
/// the row. Both FAIL on base (legacy `paint_patch` bypasses recipes).
#[test]
fn filter_row_ownership() {
    // (a) TEXT override repaints the focused text.
    let text_mut = Theme::junie().override_family(Family::INPUT, |r| {
        r.part(Part::TEXT)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut t = Harness::new(TableProApp::default(), text_mut, 120, 40);
    focus_filter(&mut t);
    let (ax, fy, _) = connections_geom(&t);
    assert_eq!(
        t.cell(ax + 2, fy).fg,
        INFO,
        "TEXT override must repaint the focused filter text"
    );
    // (b) FIELD LABEL override repaints the label row.
    let label_mut = Theme::junie().override_family(Family::FIELD, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let t = Harness::new(TableProApp::default(), label_mut, 120, 40);
    let (ax, fy, _) = connections_geom(&t);
    assert_eq!(
        t.cell(ax + 2, fy - 1).fg,
        INFO,
        "LABEL override must repaint the filter label row"
    );
}

/// T4: typing/Backspace/Esc-while-editing parity.
#[test]
fn filter_row_editing_keys() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    focus_filter(&mut t);
    assert!(t.find("Staging").is_some(), "typed filter applies live");
    assert!(t.find("Production").is_none(), "tree filtered");
    let _ = t.key(KeyCode::Backspace);
    assert_eq!(
        t.app().connections_screen.filter_state.draft_text(),
        Some("sta"),
        "Backspace edits the draft"
    );
    assert!(t.find("Staging").is_some(), "tree still filtered");
    assert!(t.find("Production").is_none(), "tree still filtered");
    let _ = t.key(KeyCode::Esc);
    assert_eq!(t.app().connections_screen.filter, "", "Esc clears");
    assert!(
        !t.app().connections_screen.filter_state.is_editing(),
        "Esc ends the edit"
    );
    assert_eq!(t.focus(), Some(CONNECTIONS_ID), "Esc focuses the tree");
    assert!(t.find("Production").is_some(), "full tree restored");
}

/// T5: Tab-out commits, Tab-in re-begins the same frame.
#[test]
fn filter_row_blur_commit() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    focus_filter(&mut t);
    let _ = t.key(KeyCode::Tab);
    assert_eq!(
        t.app().connections_screen.filter,
        "stag",
        "Tab-out keeps the text"
    );
    assert!(
        !t.app().connections_screen.filter_state.is_editing(),
        "Tab-out ends the edit"
    );
    let (ax, fy, _) = connections_geom(&t);
    assert_cell(&t, ax, fy, " ", FIELD_BG, FIELD_BG, 0);
    assert_cell(&t, ax + 2, fy - 1, " ", SECOND, BLACK, 0);
    assert!(t.tab_to(FILTER_ID), "Tab-in reaches the filter");
    assert!(
        t.app().connections_screen.filter_state.is_editing(),
        "Tab-in resumes editing"
    );
    assert_cell(&t, ax + 2, fy, "s", WHITE, FIELD_BG, UNDERLINED);
}

/// T6: Down moves focus into the tree, keeping the text.
#[test]
fn filter_row_down_to_tree() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    focus_filter(&mut t);
    let _ = t.key(KeyCode::Down);
    assert_eq!(t.focus(), Some(CONNECTIONS_ID), "Down focuses the tree");
    assert_eq!(
        t.app().connections_screen.filter,
        "stag",
        "Down keeps the text"
    );
    assert!(
        !t.app().connections_screen.filter_state.is_editing(),
        "Down ends the edit"
    );
    let (ax, fy, _) = connections_geom(&t);
    assert_cell(&t, ax + 2, fy - 1, " ", SECOND, BLACK, 0);
}

/// T7: explorer row is inert — registered but unreachable — and hovering
/// it drains pointer intents without diagnostics.
#[test]
fn filter_row_explorer_inert() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    to_workbench(&mut t);
    assert!(
        t.area_of(EXPLORER_FILTER_ID).is_some(),
        "disabled control still registers"
    );
    let (ax, fy, w) = explorer_geom(&t);
    assert_eq!((ax, fy, w), E120_GEOM, "explorer filter geometry");
    assert_window(&t, ax, fy, &E120_SYMS, &E120_RUNS);
    assert!(
        !t.tab_to(EXPLORER_FILTER_ID),
        "disabled control is never reachable"
    );
    let _ = harness_typed_input::hover(&mut t, ax + 5, fy);
    assert!(
        t.diagnostics().is_empty(),
        "hover over the inert row drains cleanly"
    );
}

/// T8a: idle non-empty value paints Primary (legacy: Muted).
#[test]
fn filter_row_stock_idle_value() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    focus_filter(&mut t);
    let _ = t.key(KeyCode::Down);
    let (ax, fy, w) = connections_geom(&t);
    assert_eq!((ax, fy, w), C120_DOWN_GEOM, "filter geometry");
    assert_eq!(
        IDLE_VALUE_RUNS[0], C120_DOWN_RUNS[0],
        "label row identical to legacy"
    );
    assert_window(&t, ax, fy, &C120_DOWN_SYMS, &IDLE_VALUE_RUNS);
}

/// T8b: hovering the filter shifts the field to FieldHover.
#[test]
fn filter_row_stock_hover() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let (ax, fy, _) = connections_geom(&t);
    let _ = harness_typed_input::hover(&mut t, ax + 5, fy);
    assert_eq!(
        t.cell(ax + 5, fy).bg,
        FIELD_HOVER,
        "hover shifts FIELD to FieldHover"
    );
    assert!(t.diagnostics().is_empty(), "hover drains cleanly");
}

/// T8c: a 40-char filter ellipsizes; the readiness cell stays blank.
#[test]
fn filter_row_stock_ellipsis() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = t.key(KeyCode::Char('/'));
    let _ = t.type_str("abcdefghijklmnopqrstuvwxyz0123456789abcd");
    let (ax, fy, w) = connections_geom(&t);
    assert_cell(&t, ax + 2, fy, "…", MUTED, FIELD_BG, UNDERLINED);
    assert_cell(&t, ax + w - 1, fy, " ", WHITE, FIELD_BG, 0);
}

/// T8d: Enter commits (tree stays filtered); further typing needs re-begin.
#[test]
fn filter_row_stock_enter_commit() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    focus_filter(&mut t);
    let _ = t.key(KeyCode::Enter);
    assert!(
        !t.app().connections_screen.filter_state.is_editing(),
        "Enter ends the edit"
    );
    assert_eq!(t.app().connections_screen.filter, "stag");
    assert!(t.find("Production").is_none(), "tree stays filtered");
    let _ = t.type_str("xq");
    assert_eq!(
        t.app().connections_screen.filter,
        "stag",
        "idle typing is ignored"
    );
    assert_eq!(
        t.app().connections_screen.filter_state.draft_text(),
        None,
        "no draft without re-begin"
    );
}

/// T8e: focused-idle Esc is a total no-op (legacy clears).
#[test]
fn filter_row_stock_idle_esc_noop() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    focus_filter(&mut t);
    let _ = t.key(KeyCode::Enter);
    let _ = t.key(KeyCode::Esc);
    assert_eq!(
        t.app().connections_screen.filter,
        "stag",
        "idle Esc keeps the value"
    );
    assert_eq!(t.focus(), Some(FILTER_ID), "idle Esc keeps focus");
    assert!(t.diagnostics().is_empty(), "idle Esc is silent");
}

/// T8f: focused-idle Backspace is a total no-op (legacy pops).
#[test]
fn filter_row_stock_idle_backspace_noop() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    focus_filter(&mut t);
    let _ = t.key(KeyCode::Enter);
    let _ = t.key(KeyCode::Backspace);
    assert_eq!(
        t.app().connections_screen.filter,
        "stag",
        "idle Backspace keeps the value"
    );
    assert_eq!(t.focus(), Some(FILTER_ID), "idle Backspace keeps focus");
    assert!(t.diagnostics().is_empty(), "idle Backspace is silent");
}

/// T8g: paste while editing inserts and live-filters (legacy ignores).
#[test]
fn filter_row_stock_paste() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    focus_filter(&mut t);
    let _ = t.paste("XY");
    assert_eq!(
        t.app().connections_screen.filter_state.draft_text(),
        Some("stagXY"),
        "paste inserts into the draft"
    );
    assert!(
        t.find("Staging").is_none(),
        "pasted text live-filters the tree"
    );
}

/// T8h: click-mid-text then type inserts mid-string (legacy appends).
#[test]
fn filter_row_stock_click_caret() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    focus_filter(&mut t);
    let (ax, fy, _) = connections_geom(&t);
    let _ = t.click(ax + 3, fy);
    let _ = t.type_str("Q");
    assert_eq!(
        t.app().connections_screen.filter_state.draft_text(),
        Some("sQtag"),
        "click places the caret mid-string"
    );
}

/// T9: both sites at 72x20 + 80x24.
#[test]
fn filter_row_small_sizes() {
    for (w, h) in [(72u16, 20u16), (80u16, 24u16)] {
        let mut t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
        let (ax, fy, fw) = connections_geom(&t);
        if (w, h) == (72, 20) {
            assert_eq!((ax, fy, fw), C72_IDLE_GEOM, "72x20 geometry");
            assert_window(&t, ax, fy, &C72_IDLE_SYMS, &C72_IDLE_RUNS);
        } else {
            assert_eq!((ax, fy, fw), C80_IDLE_GEOM, "80x24 geometry");
            assert_window(&t, ax, fy, &C80_IDLE_SYMS, &C80_IDLE_RUNS);
        }
        focus_filter(&mut t);
        let (ax, fy, fw) = connections_geom(&t);
        if (w, h) == (72, 20) {
            assert_eq!((ax, fy, fw), C72_FOCUSED_GEOM, "72x20 geometry");
            assert_window(&t, ax, fy, &C72_FOCUSED_SYMS, &C72_FOCUSED_RUNS);
        } else {
            assert_eq!((ax, fy, fw), C80_FOCUSED_GEOM, "80x24 geometry");
            assert_window(&t, ax, fy, &C80_FOCUSED_SYMS, &C80_FOCUSED_RUNS);
        }
        to_workbench(&mut t);
        let (eax, ey, ew) = explorer_geom(&t);
        if (w, h) == (72, 20) {
            assert_eq!((eax, ey, ew), E72_GEOM, "72x20 explorer geometry");
            assert_window(&t, eax, ey, &E72_SYMS, &E72_RUNS);
        } else {
            assert_eq!((eax, ey, ew), E80_GEOM, "80x24 explorer geometry");
            assert_window(&t, eax, ey, &E80_SYMS, &E80_RUNS);
        }
    }
}

/// T10: Ansi16 focused row — ▎ LightGreen + White UL text (Focus≡Accent).
#[test]
fn filter_row_ansi16_focused() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40)
        .with_color(ColorLevel::Ansi16);
    focus_filter(&mut t);
    let (ax, fy, w) = connections_geom(&t);
    assert_eq!((ax, fy, w), C120_FOCUSED_16_GEOM, "filter geometry");
    assert_window(&t, ax, fy, &C120_FOCUSED_16_SYMS, &C120_FOCUSED_16_RUNS);
    assert_cell(&t, ax, fy, "▎", Color::LightGreen, Color::Black, 0);
}

/// T11: empty diagnostics after the full T4+T5+T6+T8 flow.
#[test]
fn filter_row_diagnostics_empty() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let (ax, fy, _) = connections_geom(&t);
    focus_filter(&mut t);
    assert!(t.diagnostics().is_empty(), "after /+stag");
    let _ = t.key(KeyCode::Backspace);
    assert!(t.diagnostics().is_empty(), "after Backspace");
    let _ = t.key(KeyCode::Esc);
    assert!(t.diagnostics().is_empty(), "after editing-Esc");
    focus_filter(&mut t);
    let _ = t.key(KeyCode::Tab);
    assert!(t.diagnostics().is_empty(), "after Tab-out");
    assert!(t.tab_to(FILTER_ID));
    assert!(t.diagnostics().is_empty(), "after Tab-in");
    let _ = t.key(KeyCode::Down);
    assert!(t.diagnostics().is_empty(), "after Down");
    let _ = t.key(KeyCode::Char('/'));
    let _ = t.key(KeyCode::Enter);
    assert!(t.diagnostics().is_empty(), "after Enter");
    let _ = t.key(KeyCode::Esc);
    assert!(t.diagnostics().is_empty(), "after idle-Esc");
    let _ = t.key(KeyCode::Backspace);
    assert!(t.diagnostics().is_empty(), "after idle-Backspace");
    let _ = t.key(KeyCode::Char('/'));
    let _ = t.paste("XY");
    assert!(t.diagnostics().is_empty(), "after paste");
    let _ = t.key(KeyCode::Esc);
    focus_filter(&mut t);
    let _ = t.click(ax + 3, fy);
    assert!(t.diagnostics().is_empty(), "after click");
    let _ = t.type_str("Q");
    assert!(t.diagnostics().is_empty(), "after click-type");
    let _ = harness_typed_input::hover(&mut t, ax + 5, fy);
    assert!(t.diagnostics().is_empty(), "after hover");
    to_workbench(&mut t);
    let (eax, ey, _) = explorer_geom(&t);
    let _ = harness_typed_input::hover(&mut t, eax + 5, ey);
    assert!(t.diagnostics().is_empty(), "after explorer hover");
}

/// T12: Ctrl+D with an uncommitted draft duplicates AND keeps the tree on
/// the draft query — the base-consistent behavior (base: single live value,
/// duplicate rebuilds from it; verified by `/tmp` base probe `b2probe`).
/// The draft-vs-committed asserts pin the divergent precondition: typing
/// alone must not commit, so the duplicate has to read the effective query.
#[test]
fn filter_row_duplicate_during_draft() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    focus_filter(&mut t);
    assert_eq!(
        t.app().connections_screen.filter_state.draft_text(),
        Some("stag"),
        "typing opens a draft"
    );
    assert_eq!(
        t.app().connections_screen.filter,
        "",
        "typing alone does not commit"
    );
    let n = t.app().connections_screen.connections.len();
    let _ = t.ctrl('d');
    assert_eq!(t.app().status(), "Duplicated", "Ctrl+D duplicates");
    assert_eq!(
        t.app().connections_screen.connections.len(),
        n + 1,
        "copy inserted"
    );
    assert_eq!(
        t.app().connections_screen.filter_state.draft_text(),
        Some("stag"),
        "duplicate keeps the draft in flight"
    );
    assert!(t.find("Staging").is_some(), "tree shows Staging");
    assert!(
        t.find("Production").is_none(),
        "tree stays on the draft query"
    );
}
