//! WI-TABLEPRO-SAFETY-TOKEN: the safety-dialog token row paints through
//! stock `Field` chrome around `TextInput` (`SAFETY_INPUT`) instead of the
//! manual label + field painter.
//!
//! Base-compatible by construction: every assertion uses only APIs present
//! on base `c7b6e2805` (`input_text`, `is_editing`, `focus`, `armed`,
//! `token`, pixels, flags, `area_of`, diagnostics) — never `input_state` —
//! so this file compiles and runs unmodified on both trees for the
//! red/green matrix. Discriminators fail on base for runtime reasons.
//! Dumped from staged; never loads `baselines/`.
//!
//! * T1 pins the 120x40 gate window cell-for-cell + focused-idle flags.
//! * T2 pins the post-commit armed window (committed echo, focus Cancel).
//! * T3 pins stock ownership: INPUT TEXT / FIELD LABEL recipe overrides
//!   repaint the row (legacy `paint_patch` bypasses recipes, so both FAIL
//!   on base). Danger leg via direct construction.
//! * T4 pins typing/Backspace live-`armed()` + footer-hints flip; the
//!   Esc-while-editing leg FAILS on base (drop vs keep-text).
//! * T5 pins idle Tab-out traversal + BackTab re-entry + char pre-begin.
//! * T6 pins nav parity: Right/Down idle, Left/BackTab return, idle Esc,
//!   Confirm gating (plus Down-while-editing no-op, unlisted but base-true).
//! * T7 pins the no-token variant via the real Silent/DELETE flow.
//! * T8a-h pin the accepted stock deltas (all FAIL on base).
//! * T9 pins token + no-token legs at 72x20 + 80x24.
//! * T10 pins the Ansi16 gate row.
//! * T11 pins empty diagnostics after the full gate→armed→executed flow.
//! * T12 pins the Save-dialog token flow end to end.
//!
//! T3/T4-Esc/T8 are the red-on-base discriminators;
//! T1/T2/T5/T6/T7/T9/T10/T11/T12 are green-on-base parity guards.

use tablepro_ui::{
    SAFETY_CONFIRM, SAFETY_DIALOG, SAFETY_INPUT, SafetyDialog, SafetyFocus, SafetyIntent, Screen,
    TableProApp,
};
use termrock::{
    App, Color, ColorLevel, Cx, Diagnostic, Family, Id, KeyCode, KeyModifiers, Part, Response,
    Role, StateFlags, StylePatch, Theme, Ui,
};
use termrock_test_support::{Harness, harness_typed_input};

const UPDATE_SQL: &str = "UPDATE orders SET status = 'paid' WHERE id = 'x'";
const DELETE_SQL: &str = "DELETE FROM orders";
const TOKEN: &str = "orders";
const EXPLORER: Id = Id::root("tablepro.workbench.explorer.tree");

const WHITE: Color = Color::Rgb(255, 255, 255);
const SECOND: Color = Color::Rgb(179, 179, 179);
const MUTED: Color = Color::Rgb(128, 128, 128);
const ELEVATED: Color = Color::Rgb(24, 24, 27);
const FIELD_BG: Color = Color::Rgb(30, 30, 34);
const FIELD_HOVER: Color = Color::Rgb(35, 35, 40);
const ACCENT: Color = Color::Rgb(72, 224, 84);
const INFO: Color = Color::Rgb(135, 135, 255);
const BTN_BG: Color = Color::Rgb(39, 39, 42);
const BTN_DARK: Color = Color::Rgb(25, 25, 28);
const BORDER: Color = Color::Rgb(77, 77, 77);
const WARN: Color = Color::Rgb(245, 158, 9);
const ERR: Color = Color::Rgb(228, 69, 69);

const BOLD: u16 = 0x0001;
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

/// Pin a 2-row token window cell-for-cell: symbols plus the fg/bg/mods
/// run table. `fy` is the field (control) row; the label row is `fy - 1`.
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

/// Pin one row cell-for-cell with the same coverage discipline.
fn assert_row(t: &Harness<TableProApp>, x: u16, y: u16, syms: &str, runs: &[StyleRun]) {
    let chars: Vec<char> = syms.chars().collect();
    let w = chars.len() as u16;
    assert_eq!(row_span(t, y, x, x + w - 1), syms, "row {y} symbols");
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

/// Token-field geometry from the live registration: (x, field row, width).
fn safety_geom(t: &Harness<TableProApp>) -> (u16, u16, u16) {
    let area = t.area_of(SAFETY_INPUT).expect("safety input registered");
    (area.x, area.y, area.width)
}

fn dialog(t: &Harness<TableProApp>) -> &SafetyDialog {
    t.app().safety_dialog.as_ref().expect("safety dialog open")
}

/// Focus the active query box via real Tab presses. (PTY sends a single
/// Tab because the binary starts with query-adjacent focus; the Harness
/// starts on the tab strip, so Tab-walk to the same control.)
fn focus_query(t: &mut Harness<TableProApp>) {
    let key = t
        .app()
        .workbench
        .active_key()
        .expect("active tab after connect");
    assert!(t.tab_to(key.control("query")), "reach the query box");
}

/// Production/Safe (index 4) + the real query-gate flow.
fn open_gate(t: &mut Harness<TableProApp>) {
    assert!(t.app_mut().connect(4), "connect Production");
    assert_eq!(t.app().screen, Screen::Workbench);
    t.draw();
    focus_query(t);
    let _ = t.key(KeyCode::Char('i'));
    let _ = t.type_str(UPDATE_SQL);
    let _ = t.key(KeyCode::Esc);
    let _ = t.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
    assert!(
        t.find("Type orders to confirm").is_some(),
        "token gate visible"
    );
    assert_eq!(dialog(t).token.as_deref(), Some(TOKEN));
}

/// Production/Safe (index 4) + DELETE: the dangerous deliberate gate
/// (Risk fact, token required).
fn open_delete_gate(t: &mut Harness<TableProApp>) {
    assert!(t.app_mut().connect(4), "connect Production");
    assert_eq!(t.app().screen, Screen::Workbench);
    t.draw();
    focus_query(t);
    let _ = t.key(KeyCode::Char('i'));
    let _ = t.type_str(DELETE_SQL);
    let _ = t.key(KeyCode::Esc);
    let _ = t.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
    assert!(
        t.find("Type orders to confirm").is_some(),
        "token gate visible"
    );
    assert_eq!(dialog(t).token.as_deref(), Some(TOKEN));
}

/// The real commit-dialog flow: explorer to orders, one edit, Ctrl+S.
fn open_save_dialog(t: &mut Harness<TableProApp>) {
    assert!(t.app_mut().connect(4), "connect Production");
    t.draw();
    assert!(t.tab_to(EXPLORER), "reach the explorer");
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    assert!(t.find("public › orders").is_some(), "orders open");
    let _ = t.key(KeyCode::Home);
    for _ in 0..4 {
        let _ = t.key(KeyCode::Right);
    }
    let _ = t.key(KeyCode::Enter);
    let _ = t.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let _ = t.type_str("paid");
    let _ = t.key(KeyCode::Enter);
    let _ = t.key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(t.find("Save changes?").is_some(), "save review open");
}

/// Local/Silent (index 0) + DELETE: Confirm{deliberate:false}, token None.
fn open_confirm(t: &mut Harness<TableProApp>) {
    assert!(t.app_mut().connect(0), "connect Local");
    assert_eq!(t.app().screen, Screen::Workbench);
    t.draw();
    focus_query(t);
    let _ = t.key(KeyCode::Char('i'));
    let _ = t.type_str(DELETE_SQL);
    let _ = t.key(KeyCode::Esc);
    let _ = t.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
    assert!(t.app().safety_dialog.is_some(), "confirm dialog open");
    assert_eq!(dialog(t).token, None, "no token without deliberate");
}

/// Enter-to-arm, type the token, commit: focus lands on Cancel.
fn arm(t: &mut Harness<TableProApp>) {
    let _ = t.key(KeyCode::Enter);
    let _ = t.type_str(TOKEN);
    let _ = t.key(KeyCode::Enter);
    assert_eq!(dialog(t).focus, SafetyFocus::Cancel, "commit moves on");
}

/// Minimal rig for directly-constructed dialogs (production-unreachable
/// legs: `danger=true`). Opens the modal layer once through the stock
/// dialog, then hosts it like the application does.
struct DialogRig {
    dialog: SafetyDialog,
    opened: bool,
}

impl App for DialogRig {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !self.opened {
            self.dialog.open_layer(cx);
            self.opened = true;
        }
        self.dialog.update(cx).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(SAFETY_DIALOG, |ui, area| {
            self.dialog.draw(ui, area);
        });
    }
}

fn danger_dialog() -> SafetyDialog {
    SafetyDialog::new(
        "Delete orders?",
        vec![tablepro_ui::Prop::new("Action", "Delete")],
        vec!["DELETE FROM orders".to_owned()],
        Some(TOKEN.to_owned()),
        "Delete",
        true,
        74,
        SafetyIntent::Query,
    )
}

/// T1: 120x40 gate window, focused idle.
const GATE_GEOM: (u16, u16, u16) = (25, 24, 69);
const GATE_SYMS: [&str; 2] = [
    "  Type orders to confirm                                             ",
    "▎                                                                    ",
];
const GATE_RUNS: [&[StyleRun]; 2] = [
    &[(0, 2, MUTED, ELEVATED, 0), (2, 67, WHITE, ELEVATED, BOLD)],
    &[(0, 1, ACCENT, FIELD_BG, 0), (1, 68, WHITE, FIELD_BG, 0)],
];

/// T1: gate window + focused-idle flags (NOT EDITING — no pre-begin).
#[test]
fn safety_token_gate_120() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    assert_eq!(safety_geom(&t), GATE_GEOM, "token field geometry");
    let (ax, fy, _) = GATE_GEOM;
    assert_window(&t, ax, fy, &GATE_SYMS, &GATE_RUNS);
    assert_eq!(
        t.state_of(SAFETY_INPUT),
        StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE,
        "focused idle, no pre-begin"
    );
    assert!(!dialog(&t).armed(), "empty gate is not armed");
}

/// T2: post-commit armed window — committed echo, TEXT idle (no
/// underline), focus Cancel, Execute enabled via `armed()`.
const ARMED_SYMS: [&str; 2] = [
    "  Type orders to confirm                                             ",
    "  orders                                                             ",
];
const ARMED_RUNS: [&[StyleRun]; 2] = [
    &[(0, 2, MUTED, ELEVATED, 0), (2, 67, SECOND, ELEVATED, 0)],
    &[(0, 1, FIELD_BG, FIELD_BG, 0), (1, 68, WHITE, FIELD_BG, 0)],
];

#[test]
fn safety_token_armed_120() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    arm(&mut t);
    assert_eq!(safety_geom(&t), GATE_GEOM, "geometry stable");
    let (ax, fy, _) = GATE_GEOM;
    assert_window(&t, ax, fy, &ARMED_SYMS, &ARMED_RUNS);
    assert!(dialog(&t).armed(), "committed token arms");
    assert!(!dialog(&t).is_editing(), "commit ends the edit");
    // Enabled Execute: Right reaches Confirm only when armed.
    let _ = t.key(KeyCode::Right);
    assert_eq!(dialog(&t).focus, SafetyFocus::Confirm, "armed Right");
}

/// T3: stock ownership — INPUT TEXT / FIELD LABEL recipe overrides
/// repaint the row. Both FAIL on base (legacy `paint_patch` bypasses
/// recipes). Danger leg via direct construction.
#[test]
fn safety_token_ownership() {
    // (a) TEXT override repaints the editing text.
    let text_mut = Theme::junie().override_family(Family::INPUT, |r| {
        r.part(Part::TEXT)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut t = Harness::new(TableProApp::default(), text_mut, 120, 40);
    open_gate(&mut t);
    let _ = t.key(KeyCode::Enter);
    let _ = t.type_str("ord");
    let (ax, fy, _) = safety_geom(&t);
    assert_eq!(
        t.cell(ax + 2, fy).fg,
        INFO,
        "TEXT override must repaint the token text"
    );
    // (b) FIELD LABEL override repaints the label row.
    let label_mut = Theme::junie().override_family(Family::FIELD, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut t = Harness::new(TableProApp::default(), label_mut, 120, 40);
    open_gate(&mut t);
    let (ax, fy, _) = safety_geom(&t);
    assert_eq!(
        t.cell(ax + 2, fy - 1).fg,
        INFO,
        "LABEL override must repaint the token label row"
    );
    // (c) Danger leg: direct construction (`danger=true` is
    // production-unreachable), armed through the real key path, TEXT
    // override.
    let danger_mut = Theme::junie().override_family(Family::INPUT, |r| {
        r.part(Part::TEXT)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut rig = Harness::new(
        DialogRig {
            dialog: danger_dialog(),
            opened: false,
        },
        danger_mut,
        120,
        40,
    );
    let _ = rig.key(KeyCode::Enter);
    let _ = rig.type_str(TOKEN);
    let _ = rig.key(KeyCode::Enter);
    assert!(
        rig.app().dialog.armed(),
        "key-driven arming commits the token"
    );
    let area = rig.area_of(SAFETY_INPUT).expect("danger input registered");
    assert_eq!(
        rig.cell(area.x + 2, area.y).fg,
        INFO,
        "TEXT override must repaint the danger echo"
    );
    assert_eq!((area.x, area.y, area.width), (25, 22, 69));
}

/// T4: typing/Backspace live-`armed()` + footer-hints flip (parity);
/// Esc-while-editing drops the draft (FAILS on base: keep-text).
#[test]
fn safety_token_editing_keys() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    assert!(t.row(39).contains("Choose"), "idle footer offers Choose");
    assert!(!t.row(39).contains("Next"), "idle footer has no Next");
    let _ = t.key(KeyCode::Enter);
    assert!(t.row(39).contains("Next"), "editing footer offers Next");
    assert!(
        !t.row(39).contains("Choose"),
        "editing footer has no Choose"
    );
    let _ = t.type_str("ord");
    assert!(!dialog(&t).armed(), "partial token is not armed");
    let _ = t.type_str("ers");
    assert!(dialog(&t).armed(), "full token arms live");
    let _ = t.type_str("x");
    assert!(!dialog(&t).armed(), "overshoot disarms live");
    let _ = t.key(KeyCode::Backspace);
    assert!(dialog(&t).armed(), "Backspace re-arms live");
    // Esc-while-editing: draft dropped, focus stays Input (base keeps
    // the text, so the empty assertions below FAIL on base).
    let _ = t.key(KeyCode::Esc);
    assert!(!dialog(&t).is_editing(), "Esc ends the edit");
    assert_eq!(dialog(&t).focus, SafetyFocus::Input, "Esc holds focus");
    assert!(!dialog(&t).armed(), "dropped draft disarms");
    let (ax, fy, _) = safety_geom(&t);
    assert_eq!(t.cell(ax + 2, fy).symbol(), " ", "field is empty");
    assert!(t.row(39).contains("Choose"), "footer flips back to idle");
}

/// T5: idle Tab-out traverses (focus follows via FocusIn, no draft);
/// BackTab re-enters idle; the next char pre-begins and lands.
#[test]
fn safety_token_tab_commit() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    let _ = t.key(KeyCode::Tab);
    assert_eq!(dialog(&t).focus, SafetyFocus::Cancel, "Tab-out traverses");
    assert!(!dialog(&t).is_editing(), "Tab-out leaves no draft");
    let (ax, fy, _) = safety_geom(&t);
    assert_eq!(t.cell(ax + 2, fy).symbol(), " ", "Tab-out commits nothing");
    let _ = t.key_mod(KeyCode::Tab, KeyModifiers::SHIFT);
    assert_eq!(dialog(&t).focus, SafetyFocus::Input, "BackTab returns");
    assert!(!dialog(&t).is_editing(), "re-entry is idle");
    let _ = t.type_str("o");
    assert!(dialog(&t).is_editing(), "char pre-begins");
    let (ax, fy, _) = safety_geom(&t);
    assert_eq!(t.cell(ax + 2, fy).symbol(), "o", "char lands");
}

/// T6: nav parity — Right/Down idle → Cancel; Left/BackTab from Cancel →
/// Input; Esc idle → Cancel action; Confirm Enter gated by `armed()`.
/// (Down-while-editing no-op is an extra pin: base-true, unlisted in §6.)
#[test]
fn safety_token_nav_parity() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    let _ = t.key(KeyCode::Right);
    assert_eq!(dialog(&t).focus, SafetyFocus::Cancel, "idle Right");
    let _ = t.key(KeyCode::Right);
    assert_eq!(
        dialog(&t).focus,
        SafetyFocus::Cancel,
        "unarmed Right holds Cancel"
    );
    let _ = t.key(KeyCode::Left);
    assert_eq!(dialog(&t).focus, SafetyFocus::Input, "Left returns");
    let _ = t.key(KeyCode::Down);
    assert_eq!(dialog(&t).focus, SafetyFocus::Cancel, "idle Down");
    let _ = t.key_mod(KeyCode::Tab, KeyModifiers::SHIFT);
    assert_eq!(dialog(&t).focus, SafetyFocus::Input, "BackTab returns");
    // Down-while-editing is a no-op (base: no Down arm while editing).
    let _ = t.key(KeyCode::Enter);
    let _ = t.type_str("or");
    let _ = t.key(KeyCode::Down);
    assert!(dialog(&t).is_editing(), "Down keeps the edit");
    assert_eq!(dialog(&t).focus, SafetyFocus::Input, "Down holds focus");
    // Confirm gating end to end (fresh gate: base Esc keeps the "or"
    // above, so the gating leg must not inherit this field).
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    arm(&mut t);
    let _ = t.key(KeyCode::Right);
    assert_eq!(dialog(&t).focus, SafetyFocus::Confirm, "armed Right");
    let _ = t.key(KeyCode::Enter);
    assert!(t.app().safety_dialog.is_none(), "Confirm executes");
    // Idle Esc closes (fresh gate).
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    let _ = t.key(KeyCode::Esc);
    assert!(t.app().safety_dialog.is_none(), "idle Esc cancels");
}

/// T7: no-token variant via the real Silent/DELETE flow — no
/// `SAFETY_INPUT` registration, focus starts Confirm, actions row pinned.
/// Danger-start leg (Cancel start) via direct construction.
const NOTOKEN_ACTIONS_X: u16 = 77;
const NOTOKEN_ACTIONS_Y: u16 = 25;
const NOTOKEN_ACTIONS_SYMS: &str = "Cancel  ▎Execute   │";
const NOTOKEN_ACTIONS_RUNS: &[StyleRun] = &[
    (0, 7, WHITE, BTN_BG, 0),
    (7, 1, MUTED, ELEVATED, 0),
    (8, 1, WHITE, ACCENT, 0),
    (9, 8, BTN_DARK, ACCENT, BOLD),
    (17, 2, MUTED, ELEVATED, 0),
    (19, 1, BORDER, ELEVATED, 0),
];

#[test]
fn safety_token_no_token() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_confirm(&mut t);
    assert!(
        t.area_of(SAFETY_INPUT).is_none(),
        "no input without a token"
    );
    assert_eq!(dialog(&t).focus, SafetyFocus::Confirm, "Confirm start");
    assert!(dialog(&t).armed(), "tokenless gate is armed");
    assert_row(
        &t,
        NOTOKEN_ACTIONS_X,
        NOTOKEN_ACTIONS_Y,
        NOTOKEN_ACTIONS_SYMS,
        NOTOKEN_ACTIONS_RUNS,
    );
    // Danger-start leg: direct construction (production-unreachable).
    let danger = SafetyDialog::new(
        "Delete?",
        vec![],
        vec![],
        None,
        "Delete",
        true,
        74,
        SafetyIntent::Query,
    );
    assert_eq!(danger.focus, SafetyFocus::Cancel, "danger Cancel start");
}

/// T8a: Esc-drop disarms (base: Esc keeps the text, stays armed).
#[test]
fn safety_token_stock_esc_drop() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    let _ = t.key(KeyCode::Enter);
    let _ = t.type_str(TOKEN);
    assert!(dialog(&t).armed(), "draft arms live");
    let _ = t.key(KeyCode::Esc);
    assert!(!dialog(&t).armed(), "dropped draft disarms");
    let (ax, fy, _) = safety_geom(&t);
    assert_eq!(t.cell(ax + 2, fy).symbol(), " ", "nothing committed");
}

/// T8b: idle Enter begins without committing (base begins too, but never
/// declares EDITING — the flags pin is the discriminator).
#[test]
fn safety_token_stock_idle_enter() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    let _ = t.key(KeyCode::Enter);
    assert!(dialog(&t).is_editing(), "idle Enter begins");
    let (ax, fy, _) = safety_geom(&t);
    assert_eq!(t.cell(ax + 2, fy).symbol(), " ", "nothing committed");
    assert_eq!(dialog(&t).focus, SafetyFocus::Input, "focus holds");
    assert_eq!(
        t.state_of(SAFETY_INPUT),
        StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::EDITING,
        "stock edit phase"
    );
}

/// T8c: the stock edit phase runs under the global cursor suppression —
/// EDITING is declared (base never declares it) while pixels stay clean.
#[test]
fn safety_token_stock_cursor_suppressed() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    let _ = t.key(KeyCode::Enter);
    let _ = t.type_str("ord");
    assert!(
        t.state_of(SAFETY_INPUT).contains(StateFlags::EDITING),
        "edit phase declares EDITING"
    );
    assert_eq!(t.cursor(), None, "global suppression holds");
}

/// T8d: editing text gains UNDERLINED (moved from T2).
#[test]
fn safety_token_stock_edit_underline() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    let _ = t.key(KeyCode::Enter);
    let _ = t.type_str("ord");
    let (ax, fy, _) = safety_geom(&t);
    for (i, sym) in ["o", "r", "d"].into_iter().enumerate() {
        assert_cell(&t, ax + 2 + i as u16, fy, sym, WHITE, FIELD_BG, UNDERLINED);
    }
    assert_cell(&t, ax + 5, fy, " ", WHITE, FIELD_BG, 0);
}

/// T8e: Tab-while-editing traverses + blur-commits (base: traversal with
/// `input_editing` stuck). Split from T5.
#[test]
fn safety_token_stock_tab_blur_commit() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    let _ = t.key(KeyCode::Enter);
    let _ = t.type_str("ord");
    let _ = t.key(KeyCode::Tab);
    assert!(!dialog(&t).is_editing(), "blur ends the edit");
    let (ax, fy, _) = safety_geom(&t);
    for (i, sym) in ["o", "r", "d"].into_iter().enumerate() {
        assert_eq!(t.cell(ax + 2 + i as u16, fy).symbol(), sym, "blur commits");
    }
    assert_eq!(dialog(&t).focus, SafetyFocus::Cancel, "traversal lands");
}

/// T8f: stock editing keys — cursor motion + mid-insert replace
/// append-only input (base commits "orsd").
#[test]
fn safety_token_stock_mid_insert() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    let _ = t.key(KeyCode::Enter);
    let _ = t.type_str("ors");
    let _ = t.key(KeyCode::Left);
    let _ = t.type_str("d");
    let _ = t.key(KeyCode::Enter);
    let (ax, fy, _) = safety_geom(&t);
    for (i, sym) in ["o", "r", "d", "s"].into_iter().enumerate() {
        assert_eq!(
            t.cell(ax + 2 + i as u16, fy).symbol(),
            sym,
            "caret insert lands mid"
        );
    }
}

/// T8g: click begins editing (base: focus-only; PTY-safe — no click path
/// in the ack sends).
#[test]
fn safety_token_stock_click_begins() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    let (ax, fy, _) = safety_geom(&t);
    let _ = t.click(ax + 5, fy);
    assert!(dialog(&t).is_editing(), "click begins the edit");
    assert_eq!(dialog(&t).focus, SafetyFocus::Input, "click holds focus");
}

/// T8h: hover notes FieldHover bg (HOVERED|EDITING reverts to Field).
#[test]
fn safety_token_stock_hover() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    let (ax, fy, _) = safety_geom(&t);
    let _ = harness_typed_input::hover(&mut t, ax + 5, fy);
    assert_eq!(
        t.cell(ax + 5, fy).bg,
        FIELD_HOVER,
        "hover shifts FIELD to FieldHover"
    );
    assert!(t.diagnostics().is_empty(), "hover drains cleanly");
}

/// T9: token + no-token legs at 72x20 + 80x24.
const GATE72_GEOM: (u16, u16, u16) = (4, 13, 63);
const GATE72_SYMS: [&str; 2] = [
    "  Type orders to confirm                                       ",
    "▎                                                              ",
];
const GATE72_RUNS: [&[StyleRun]; 2] = [
    &[(0, 2, MUTED, ELEVATED, 0), (2, 61, WHITE, ELEVATED, BOLD)],
    &[(0, 1, ACCENT, FIELD_BG, 0), (1, 62, WHITE, FIELD_BG, 0)],
];
const GATE80_GEOM: (u16, u16, u16) = (5, 16, 69);

#[test]
fn safety_token_small_sizes() {
    for (w, h) in [(72u16, 20u16), (80u16, 24u16)] {
        let mut t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
        open_gate(&mut t);
        if (w, h) == (72, 20) {
            assert_eq!(safety_geom(&t), GATE72_GEOM, "72x20 geometry");
            let (ax, fy, _) = GATE72_GEOM;
            assert_window(&t, ax, fy, &GATE72_SYMS, &GATE72_RUNS);
        } else {
            assert_eq!(safety_geom(&t), GATE80_GEOM, "80x24 geometry");
            let (ax, fy, _) = GATE80_GEOM;
            assert_window(&t, ax, fy, &GATE_SYMS, &GATE_RUNS);
        }
        let mut c = Harness::new(TableProApp::default(), Theme::junie(), w, h);
        open_confirm(&mut c);
        assert!(
            c.area_of(SAFETY_INPUT).is_none(),
            "{w}x{h}: no input without a token"
        );
        assert_eq!(
            dialog(&c).focus,
            SafetyFocus::Confirm,
            "{w}x{h}: Confirm start"
        );
    }
}

/// T10: Ansi16 gate row (Focus≡Accent).
const GATE16_RUNS: [&[StyleRun]; 2] = [
    &[
        (0, 2, Color::Gray, Color::Black, 0),
        (2, 67, Color::White, Color::Black, BOLD),
    ],
    &[
        (0, 1, Color::LightGreen, Color::Black, 0),
        (1, 68, Color::White, Color::Black, 0),
    ],
];

#[test]
fn safety_token_ansi16_gate() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40)
        .with_color(ColorLevel::Ansi16);
    open_gate(&mut t);
    assert_eq!(safety_geom(&t), GATE_GEOM, "geometry");
    let (ax, fy, _) = GATE_GEOM;
    assert_window(&t, ax, fy, &GATE_SYMS, &GATE16_RUNS);
    assert_cell(&t, ax, fy, "▎", Color::LightGreen, Color::Black, 0);
}

/// T11: empty diagnostics through the gate→armed dialog lifetime. The
/// execute frame itself carries the two pre-existing `UndeliveredIntent`
/// diagnostics (dialog + confirm owners, now under the stock component
/// ids) — pinned here so no NEW diagnostic can hide behind them.
#[test]
fn safety_token_diagnostics_empty() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    assert!(t.diagnostics().is_empty(), "after open");
    let _ = t.key(KeyCode::Enter);
    assert!(t.diagnostics().is_empty(), "after idle Enter");
    let _ = t.type_str(TOKEN);
    assert!(t.diagnostics().is_empty(), "after typing");
    let _ = t.key(KeyCode::Enter);
    assert!(t.diagnostics().is_empty(), "after commit");
    let _ = t.key(KeyCode::Right);
    assert!(t.diagnostics().is_empty(), "after Right");
    let _ = t.key(KeyCode::Enter);
    assert!(t.app().safety_dialog.is_none(), "executed");
    assert!(t.find("rows affected").is_some(), "executed status");
    let owners: Vec<Id> = t
        .diagnostics()
        .iter()
        .map(|d| match d {
            Diagnostic::UndeliveredIntent { owner } => *owner,
            other => panic!("unexpected diagnostic after execute: {other:?}"),
        })
        .collect();
    assert_eq!(
        owners,
        vec![SAFETY_DIALOG, SAFETY_CONFIRM],
        "only the two pre-existing undelivered intents"
    );
}

/// T12: Save-dialog token flow end to end via the real commit-dialog
/// sends (explorer path needs Tab-walk: the Harness starts on the tab
/// strip, the binary on the explorer). Save enabled only when armed;
/// danger leg via direct construction.
#[test]
fn safety_token_commit_intent() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_save_dialog(&mut t);
    assert_eq!(dialog(&t).token.as_deref(), Some(TOKEN));
    assert_eq!(dialog(&t).confirm_label, "Save");
    // Save disabled until armed: Confirm skipped in traversal, Right held.
    assert!(
        !t.tab_to(SAFETY_CONFIRM),
        "Confirm unreachable while unarmed"
    );
    assert_eq!(dialog(&t).focus, SafetyFocus::Input, "wrap holds");
    let _ = t.key(KeyCode::Right);
    assert_eq!(dialog(&t).focus, SafetyFocus::Cancel, "Right to Cancel");
    let _ = t.key(KeyCode::Right);
    assert_eq!(dialog(&t).focus, SafetyFocus::Cancel, "unarmed Right holds");
    let _ = t.key(KeyCode::Left);
    arm(&mut t);
    assert!(dialog(&t).armed(), "armed");
    assert!(t.tab_to(SAFETY_CONFIRM), "Confirm reachable armed");
    let _ = t.key(KeyCode::Enter);
    assert!(t.app().safety_dialog.is_none(), "saved");
    assert!(t.find("Saving").is_some(), "saving status");
    // Danger leg: direct construction (production-unreachable).
    let danger = danger_dialog();
    assert_eq!(danger.focus, SafetyFocus::Input, "token Input start");
    assert!(!danger.armed(), "danger starts disarmed");
    let mut rig = Harness::new(
        DialogRig {
            dialog: danger,
            opened: false,
        },
        Theme::junie(),
        120,
        40,
    );
    let _ = rig.key(KeyCode::Enter);
    let _ = rig.type_str(TOKEN);
    let _ = rig.key(KeyCode::Enter);
    assert!(rig.app().dialog.armed(), "committed token arms danger");
    assert!(rig.find("Type orders to confirm").is_some(), "label row");
    assert!(rig.find("Delete").is_some(), "danger confirm visible");
}

/// F1: the 120x40 gate frame through stock Dialog chrome + Props facts:
/// corners, borders, title, facts, SQL and actions cell-for-cell.
/// (Blank cells keep backdrop fg in both renders, so only painted spans
/// are pinned.)
#[test]
fn safety_frame_gate_120() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    // Corners + full horizontal borders (every cell painted).
    for (x, y, sym) in [
        (23u16, 11u16, "╭"),
        (96, 11, "╮"),
        (23, 29, "╰"),
        (96, 29, "╯"),
    ] {
        assert_cell(&t, x, y, sym, BORDER, ELEVATED, 0);
    }
    let top = format!("╭{}╮", "─".repeat(72));
    assert_row(&t, 23, 11, &top, &[(0, 74, BORDER, ELEVATED, 0)]);
    let bottom = format!("╰{}╯", "─".repeat(72));
    assert_row(&t, 23, 29, &bottom, &[(0, 74, BORDER, ELEVATED, 0)]);
    for y in 12..29 {
        assert_cell(&t, 23, y, "│", BORDER, ELEVATED, 0);
        assert_cell(&t, 96, y, "│", BORDER, ELEVATED, 0);
    }
    // Title.
    assert_row(
        &t,
        26,
        13,
        "Execute write query?",
        &[(0, 20, WHITE, ELEVATED, BOLD)],
    );
    // Facts: label + value spans.
    assert_row(&t, 26, 15, "Action", &[(0, 6, MUTED, ELEVATED, 0)]);
    assert_row(&t, 38, 15, "UPDATE", &[(0, 6, WHITE, ELEVATED, 0)]);
    assert_row(&t, 26, 16, "Target", &[(0, 6, MUTED, ELEVATED, 0)]);
    assert_row(
        &t,
        38,
        16,
        "Production · production · acme_prod · orders",
        &[(0, 44, WHITE, ELEVATED, 0)],
    );
    assert_row(&t, 26, 17, "Scope", &[(0, 5, MUTED, ELEVATED, 0)]);
    assert_row(
        &t,
        38,
        17,
        "matching rows in orders",
        &[(0, 23, SECOND, ELEVATED, 0)],
    );
    assert_row(&t, 26, 18, "Reversible", &[(0, 10, MUTED, ELEVATED, 0)]);
    assert_row(
        &t,
        38,
        18,
        "Reversible by a compensating UPDATE",
        &[(0, 35, SECOND, ELEVATED, 0)],
    );
    assert_row(&t, 26, 19, "Safe Mode", &[(0, 9, MUTED, ELEVATED, 0)]);
    assert_row(
        &t,
        38,
        19,
        "Safe Mode · deliberate confirmation required",
        &[(0, 44, MUTED, ELEVATED, 0)],
    );
    // SQL.
    assert_row(
        &t,
        26,
        21,
        "UPDATE orders SET status = 'paid' WHERE id = 'x'",
        &[(0, 48, SECOND, ELEVATED, 0)],
    );
    // The field's third row: blank symbols on the elevated surface.
    // (fg stays backdrop-defined, as on the manual render — unpinned).
    for x in 25..94 {
        let cell = t.cell(x, 25);
        assert_eq!(cell.symbol(), " ", "({x},25) symbol");
        assert_eq!(cell.bg, ELEVATED, "({x},25) bg");
    }
    // Actions: unfocused Cancel, disabled Execute.
    assert_row(
        &t,
        76,
        27,
        " Cancel ",
        &[(0, 1, BTN_BG, BTN_BG, 0), (1, 7, WHITE, BTN_BG, 0)],
    );
    assert_row(
        &t,
        85,
        27,
        " Execute ",
        &[(0, 1, BTN_BG, BTN_BG, 0), (1, 8, BORDER, BTN_BG, 0)],
    );
}

/// F2: delete-gate wrap rows — the Risk fact wraps at the value column
/// and the dangerous Action value carries the error tone.
#[test]
fn safety_frame_delete_gate_wrap() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_delete_gate(&mut t);
    assert_cell(&t, 23, 10, "╭", BORDER, ELEVATED, 0);
    assert_cell(&t, 96, 30, "╯", BORDER, ELEVATED, 0);
    assert_row(
        &t,
        26,
        12,
        "This query may permanently modify or delete data",
        &[(0, 48, WHITE, ELEVATED, BOLD)],
    );
    assert_row(
        &t,
        38,
        14,
        "DELETE without WHERE",
        &[(0, 20, ERR, ELEVATED, 0)],
    );
    assert_row(&t, 26, 17, "Risk", &[(0, 4, MUTED, ELEVATED, 0)]);
    assert_row(
        &t,
        38,
        17,
        "Removes all rows; dependent rows may go with ON DELETE",
        &[(0, 54, WARN, ELEVATED, 0)],
    );
    assert_row(&t, 38, 18, "CASCADE.", &[(0, 8, WARN, ELEVATED, 0)]);
}

/// F3: commit-dialog wrap + truncation — Transaction wraps, the long SQL
/// truncates with an ellipsis, Save stays right-edged.
#[test]
fn safety_frame_commit_wrap_truncate() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_save_dialog(&mut t);
    assert_cell(&t, 21, 10, "╭", BORDER, ELEVATED, 0);
    assert_cell(&t, 98, 29, "╯", BORDER, ELEVATED, 0);
    assert_row(
        &t,
        24,
        12,
        "Save changes?",
        &[(0, 13, WHITE, ELEVATED, BOLD)],
    );
    assert_row(
        &t,
        37,
        17,
        "All statements run in one transaction; a failure rolls",
        &[(0, 54, MUTED, ELEVATED, 0)],
    );
    assert_row(
        &t,
        37,
        18,
        "everything back.",
        &[(0, 16, MUTED, ELEVATED, 0)],
    );
    assert_row(
        &t,
        24,
        21,
        "UPDATE public.orders SET status = 'paid' WHERE id = '1d9f28fc-3e09-4488…",
        &[(0, 72, SECOND, ELEVATED, 0)],
    );
    assert_row(
        &t,
        81,
        27,
        " Cancel ",
        &[(0, 1, BTN_BG, BTN_BG, 0), (1, 7, WHITE, BTN_BG, 0)],
    );
    assert_row(
        &t,
        90,
        27,
        " Save ",
        &[(0, 1, BTN_BG, BTN_BG, 0), (1, 5, BORDER, BTN_BG, 0)],
    );
}

/// F4: the armed actions row — focused Cancel, enabled primary Execute.
#[test]
fn safety_frame_armed_actions() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    open_gate(&mut t);
    arm(&mut t);
    assert_row(
        &t,
        76,
        27,
        "▎Cancel ",
        &[(0, 1, ACCENT, BTN_BG, 0), (1, 7, WHITE, BTN_BG, BOLD)],
    );
    assert_row(
        &t,
        85,
        27,
        " Execute ",
        &[(0, 1, ACCENT, ACCENT, 0), (1, 8, BTN_DARK, ACCENT, BOLD)],
    );
}

/// F5: 72x20 frame — corners, title, clipped facts (Safe Mode gone),
/// surviving SQL, token field and actions.
#[test]
fn safety_frame_small() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 72, 20);
    open_gate(&mut t);
    for (x, y, sym) in [(2u16, 1u16, "╭"), (69, 1, "╮"), (2, 18, "╰"), (69, 18, "╯")] {
        assert_cell(&t, x, y, sym, BORDER, ELEVATED, 0);
    }
    assert_row(
        &t,
        5,
        3,
        "Execute write query?",
        &[(0, 20, WHITE, ELEVATED, BOLD)],
    );
    assert_row(&t, 5, 5, "Action", &[(0, 6, MUTED, ELEVATED, 0)]);
    assert_row(&t, 5, 8, "Reversible", &[(0, 10, MUTED, ELEVATED, 0)]);
    let frame: String = (1..=18).map(|y| row_span(&t, y, 2, 69)).collect();
    assert!(
        !frame.contains("Safe Mode"),
        "short screens clip the facts region"
    );
    assert_row(
        &t,
        5,
        10,
        "UPDATE orders SET status = 'paid' WHERE id = 'x'",
        &[(0, 48, SECOND, ELEVATED, 0)],
    );
    assert_row(
        &t,
        49,
        16,
        " Cancel ",
        &[(0, 1, BTN_BG, BTN_BG, 0), (1, 7, WHITE, BTN_BG, 0)],
    );
    assert_row(
        &t,
        58,
        16,
        " Execute ",
        &[(0, 1, BTN_BG, BTN_BG, 0), (1, 8, BORDER, BTN_BG, 0)],
    );
}

/// F6: Ansi16 frame — corners, title and actions through the 16-color lane.
#[test]
fn safety_frame_ansi16() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40)
        .with_color(ColorLevel::Ansi16);
    open_gate(&mut t);
    for (x, y, sym) in [
        (23u16, 11u16, "╭"),
        (96, 11, "╮"),
        (23, 29, "╰"),
        (96, 29, "╯"),
    ] {
        assert_cell(&t, x, y, sym, Color::DarkGray, Color::Black, 0);
    }
    assert_row(
        &t,
        26,
        13,
        "Execute write query?",
        &[(0, 20, Color::White, Color::Black, BOLD)],
    );
    assert_row(
        &t,
        76,
        27,
        " Cancel ",
        &[
            (0, 1, Color::DarkGray, Color::DarkGray, 0),
            (1, 7, Color::White, Color::DarkGray, 0),
        ],
    );
    assert_row(
        &t,
        85,
        27,
        " Execute ",
        &[
            (0, 1, Color::Black, Color::Black, 0),
            (1, 8, Color::DarkGray, Color::Black, 0),
        ],
    );
}

/// F7: stock ownership — DIALOG BORDER/TITLE + PROPS META/LABEL recipe
/// overrides repaint the frame (manual paint would bypass recipes).
#[test]
fn safety_frame_ownership() {
    let border_mut = Theme::junie().override_family(Family::DIALOG, |r| {
        r.part(Part::BORDER)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut t = Harness::new(TableProApp::default(), border_mut, 120, 40);
    open_gate(&mut t);
    assert_eq!(
        t.cell(23, 11).fg,
        INFO,
        "BORDER override must repaint the frame"
    );
    let title_mut = Theme::junie().override_family(Family::DIALOG, |r| {
        r.part(Part::TITLE)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut t = Harness::new(TableProApp::default(), title_mut, 120, 40);
    open_gate(&mut t);
    assert_eq!(
        t.cell(26, 13).fg,
        INFO,
        "TITLE override must repaint the title"
    );
    let meta_mut = Theme::junie().override_family(Family::PROPS, |r| {
        r.part(Part::META)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut t = Harness::new(TableProApp::default(), meta_mut, 120, 40);
    open_gate(&mut t);
    assert_eq!(
        t.cell(26, 15).fg,
        INFO,
        "META override must repaint the fact labels"
    );
    let label_mut = Theme::junie().override_family(Family::PROPS, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut t = Harness::new(TableProApp::default(), label_mut, 120, 40);
    open_gate(&mut t);
    assert_eq!(
        t.cell(38, 15).fg,
        INFO,
        "LABEL override must repaint the fact values"
    );
}

/// F8: no-paint tripwire — the dialog model paints nothing itself.
#[test]
fn safety_frame_no_manual_paint() {
    let src = include_str!("../src/safety_dialog.rs");
    for needle in [
        "paint_str",
        "ui.fill",
        "paint_patch",
        "with_surface",
        "Ui::frame",
    ] {
        assert!(!src.contains(needle), "manual painter survived: {needle}");
    }
}
