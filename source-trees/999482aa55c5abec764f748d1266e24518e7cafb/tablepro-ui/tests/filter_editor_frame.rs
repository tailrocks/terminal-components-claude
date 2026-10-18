//! WI-TABLEPRO-FILTER-EDITOR-FRAME commit 2: the filter editor dialog
//! frame, title, labels, help, no-value note and WHERE preview paint through
//! stock `Panel` + `Field` + `List` instead of the manual frame/label
//! painters. Interior `Select`x2/`TextInput`x2/`Button`x2 were already live.
//!
//! Base-compatible by construction: the three new ids appear ONLY as
//! `Id::root("…")` literals (importing them breaks base compile); the op
//! poke writes the public `FilterEditor::op` field present on base; every
//! red leg fails on base for runtime reasons. Pixels dumped from the pristine
//! base archive (`git archive 5400b5cbd`); never loads `baselines/`.
//!
//! Poke discipline (probed, see `zz` probes during development): an op flip
//! is poked only while runtime focus sits on a mode-stable stop — production
//! flips via the op dropdown with focus=Op (the popup dismisses on any focus
//! escape, so a stale Value/Value2 focus is unreachable there). Poking
//! IsNull while focused on Value strands runtime focus on an id the no-value
//! draw never registers; base reconciles forward to Cancel while staged
//! bounces the reconcile arrival back to the stale stop, so the suite clicks
//! OP (then Esc-dismisses the dropdown the click opens) before the poke.
//!
//! * T1 pins the 120x40 Add dialog cell-for-cell (frame, title, labels,
//!   fields, help, preview, buttons).
//! * T2 pins title parity for Add + Edit ("Edit filter" via the chips-bar
//!   Enter path); the top row carries no text spans.
//! * T3 pins stock ownership: FIELD LABEL / FIELD HELP / PANEL BORDER recipe
//!   overrides repaint labels / help / border incl. the gutter cell; LIST
//!   LABEL-bg repaints title+note+preview text bgs (fg leg impossible —
//!   `label_patched` wins fg, guarded); LIST CONTAINER-bg repaints row-blank
//!   bgs (fg leg impossible — the `clear_fg` instance patch wins, guarded).
//!   All five repaint legs FAIL on base (legacy `paint_patch` bypasses
//!   recipes); the fg-wins guards pass on both.
//! * T4 pins focus cycling incl. both TITLE-skipping wraps, the Between
//!   Value2 stop, and the no-value Op→Cancel / Cancel→Op static skips: every
//!   step pins label bolds + `state_of` of the six controls + empty statics.
//! * T5 pins the Between twins (labels + 28/29 geometry).
//! * T6 pins the no-value note (Muted text @x+4, marker@x+2 blank) + live
//!   `WHERE id IS NULL` preview.
//! * T7 pins preview draft-vs-committed (live draft, Esc revert, Enter apply).
//! * T8 pins click no-ops (title/note/preview rows + a preview-row border
//!   cell + a blank row): focus unchanged, Tab still cycles, diagnostics
//!   empty — plus a backdrop-dismiss guard. DEVIATION: the inside-click legs
//!   are RED→GREEN, not GREEN/GREEN as planned (base dead-area clicks fall
//!   through the bare paint and dismiss the modal — probed; staged captures
//!   them per the §5.2 bounce). The hover leg pins the HoverSurface repaint
//!   on static rows (PTY-blind accepted stock behavior; FAILS on base).
//! * T9 pins the 80x24 Add dialog + full-flow empty diagnostics (pins no
//!   `UnknownPart` from the CONTAINER patches).
//! * T10 pins the Edit leg (prefill title, Update label, Apply focus).
//!
//! T3-repaints/T8-clicks/T8-hover are the red-on-base discriminators;
//! T1/T2/T3-guards/T4/T5/T6/T7/T9/T10 are green-on-base guards. (The T8
//! backdrop leg rides inside the red click test, so it aborts early on base;
//! base backdrop-dismiss itself was probed green standalone.)

use tablepro_ui::filter_editor::{
    FILTER_APPLY, FILTER_CANCEL, FILTER_COL, FILTER_EDITOR, FILTER_OP, FILTER_VALUE, FILTER_VALUE2,
    FilterFocus, FilterOp,
};
use tablepro_ui::{Screen, TableProApp};
use termrock::{Color, Family, Id, KeyCode, KeyModifiers, Part, Role, StylePatch, Theme};
use termrock_test_support::{Harness, harness_typed_input};

const WHITE: Color = Color::Rgb(255, 255, 255);
const SECOND: Color = Color::Rgb(179, 179, 179);
const MUTED: Color = Color::Rgb(128, 128, 128);
const ELEVATED: Color = Color::Rgb(24, 24, 27);
const FIELD_BG: Color = Color::Rgb(30, 30, 34);
const ACCENT: Color = Color::Rgb(72, 224, 84);
const INFO: Color = Color::Rgb(135, 135, 255);
const BORDER: Color = Color::Rgb(77, 77, 77);
const BTN_DARK: Color = Color::Rgb(25, 25, 28);
const HOVER_BG: Color = Color::Rgb(39, 39, 42);
const C38: Color = Color::Rgb(38, 38, 38);
const BLACK: Color = Color::Rgb(0, 0, 0);

const BOLD: u16 = 0x0001;

/// New ids as literals: byte-identical to the staged consts, base-safe.
const TITLE_ID: Id = Id::root("tablepro.filter.title");
const NOTE_ID: Id = Id::root("tablepro.filter.note");
const PREVIEW_ID: Id = Id::root("tablepro.filter.preview");

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

/// Pin one dialog row cell-for-cell. The coverage assert keeps the pin
/// honest: every cell must be asserted.
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

/// Drive the app to an open table tab. `connect(4)` runs BEFORE
/// `Harness::new` (post-wrap connect breaks the explorer drive).
fn drive_table(w: u16, h: u16) -> Harness<TableProApp> {
    let mut app = TableProApp::default();
    assert!(app.connect(4), "connect(4) failed");
    assert_eq!(app.screen, Screen::Workbench);
    let mut t = Harness::new(app, Theme::junie(), w, h);
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    t.draw();
    t
}

/// Open the Add editor via ctrl-f; returns the dialog origin.
fn open_add(t: &mut Harness<TableProApp>) -> (u16, u16) {
    let _ = t.key_mod(KeyCode::Char('f'), KeyModifiers::CONTROL);
    t.draw();
    assert!(t.is_open(FILTER_EDITOR), "editor open:\n{}", t.text());
    dialog_origin(t)
}

/// Dialog origin from the live layer; asserts the fixed 64x15 geometry.
fn dialog_origin(t: &Harness<TableProApp>) -> (u16, u16) {
    let layer = t.layer_area(FILTER_EDITOR).expect("dialog layer area");
    assert_eq!((layer.width, layer.height), (64, 15), "dialog size");
    (layer.x, layer.y)
}

fn editor_focus(t: &Harness<TableProApp>) -> FilterFocus {
    t.app().filter_editor.as_ref().expect("editor").focus
}

/// Flip the op with the poke discipline: Between keeps Value registered.
fn poke_between(t: &mut Harness<TableProApp>) {
    t.app_mut().filter_editor.as_mut().unwrap().op = FilterOp::Between;
    t.draw();
}

/// Flip to a no-value op: click OP first so runtime focus sits on a
/// mode-stable stop, Esc-dismiss the dropdown the click opens, then poke.
fn poke_isnull(t: &mut Harness<TableProApp>) {
    let _ = t.click_id(FILTER_OP);
    t.draw();
    let _ = t.key(KeyCode::Esc);
    t.draw();
    t.app_mut().filter_editor.as_mut().unwrap().op = FilterOp::IsNull;
    t.draw();
    assert_eq!(editor_focus(t), FilterFocus::Op, "op stable stop");
}

// T1: 120x40 Add dialog, dumped from the pristine base archive. Blank-cell
// fg values are deterministic bleed-through from the fixed drive (the dialog
// fill is bg-only on both trees); bg is uniformly Elevated.
const T1_TOP_SYMS: &str = "╭──────────────────────────────────────────────────────────────╮";
const T1_TOP_RUNS: &[StyleRun] = &[(0, 64, BORDER, ELEVATED, 0)];
const T1_BOT_SYMS: &str = "╰──────────────────────────────────────────────────────────────╯";
const T1_BOT_RUNS: &[StyleRun] = &[(0, 64, BORDER, ELEVATED, 0)];
const T1_BLANK_SYMS: &str = "│                                                              │";
const T1_BLANK_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 1, C38, ELEVATED, 0),
    (4, 2, MUTED, ELEVATED, 0),
    (6, 1, BLACK, ELEVATED, 0),
    (7, 2, MUTED, ELEVATED, 0),
    (9, 3, C38, ELEVATED, 0),
    (12, 1, MUTED, ELEVATED, 0),
    (13, 36, BORDER, ELEVATED, 0),
    (49, 14, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T1_TITLE_SYMS: &str = "│  Add filter                                                  │";
const T1_TITLE_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 10, WHITE, ELEVATED, BOLD),
    (13, 36, BORDER, ELEVATED, 0),
    (49, 14, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T1_LABELS_SYMS: &str = "│   Column                        Operator                     │";
const T1_LABELS_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 1, C38, ELEVATED, 0),
    (4, 6, SECOND, ELEVATED, 0),
    (10, 2, C38, ELEVATED, 0),
    (12, 1, MUTED, ELEVATED, 0),
    (13, 21, BORDER, ELEVATED, 0),
    (34, 8, SECOND, ELEVATED, 0),
    (42, 7, BORDER, ELEVATED, 0),
    (49, 14, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T1_SELECTS_SYMS: &str = "│   id                      ▾     =                        ▾   │";
const T1_SELECTS_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, FIELD_BG, FIELD_BG, 0),
    (3, 25, WHITE, FIELD_BG, 0),
    (28, 1, SECOND, FIELD_BG, 0),
    (29, 1, WHITE, FIELD_BG, 0),
    (30, 2, BORDER, ELEVATED, 0),
    (32, 1, FIELD_BG, FIELD_BG, 0),
    (33, 26, WHITE, FIELD_BG, 0),
    (59, 1, SECOND, FIELD_BG, 0),
    (60, 1, WHITE, FIELD_BG, 0),
    (61, 2, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T1_HELP_SYMS: &str = "│                                 Operators that fit the col…  │";
const T1_HELP_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 1, C38, ELEVATED, 0),
    (4, 2, MUTED, ELEVATED, 0),
    (6, 1, BLACK, ELEVATED, 0),
    (7, 2, MUTED, ELEVATED, 0),
    (9, 3, C38, ELEVATED, 0),
    (12, 1, MUTED, ELEVATED, 0),
    (13, 21, BORDER, ELEVATED, 0),
    (34, 29, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T1_VALLABEL_SYMS: &str = "│   Value                                                      │";
const T1_VALLABEL_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 1, C38, ELEVATED, 0),
    (4, 57, WHITE, ELEVATED, BOLD),
    (61, 2, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T1_VAL_SYMS: &str = "│ ▎ value                                                      │";
const T1_VAL_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, ACCENT, FIELD_BG, 0),
    (3, 1, WHITE, FIELD_BG, 0),
    (4, 5, MUTED, FIELD_BG, 0),
    (9, 52, WHITE, FIELD_BG, 0),
    (61, 2, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T1_PREVIEW_SYMS: &str = "│  WHERE id = ''                                               │";
const T1_PREVIEW_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 13, SECOND, ELEVATED, 0),
    (16, 33, BORDER, ELEVATED, 0),
    (49, 14, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T1_BUTTONS_SYMS: &str = "│                                        Cancel   Add filter   │";
const T1_BUTTONS_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 1, C38, ELEVATED, 0),
    (4, 2, MUTED, ELEVATED, 0),
    (6, 1, BLACK, ELEVATED, 0),
    (7, 2, MUTED, ELEVATED, 0),
    (9, 3, C38, ELEVATED, 0),
    (12, 1, MUTED, ELEVATED, 0),
    (13, 27, BORDER, ELEVATED, 0),
    (40, 1, ELEVATED, ELEVATED, 0),
    (41, 7, SECOND, ELEVATED, 0),
    (48, 1, BORDER, ELEVATED, 0),
    (49, 1, ACCENT, ACCENT, 0),
    (50, 11, BTN_DARK, ACCENT, BOLD),
    (61, 2, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];

/// T1: full 120x40 Add dialog pin (GREEN/GREEN).
#[test]
fn filter_editor_frame_add_120() {
    let mut t = drive_table(120, 40);
    let (lx, ly) = open_add(&mut t);
    assert_eq!(editor_focus(&t), FilterFocus::Value, "open focus");
    assert_row(&t, lx, ly, T1_TOP_SYMS, T1_TOP_RUNS);
    assert_row(&t, lx, ly + 1, T1_TITLE_SYMS, T1_TITLE_RUNS);
    assert_row(&t, lx, ly + 2, T1_BLANK_SYMS, T1_BLANK_RUNS);
    assert_row(&t, lx, ly + 3, T1_LABELS_SYMS, T1_LABELS_RUNS);
    assert_row(&t, lx, ly + 4, T1_SELECTS_SYMS, T1_SELECTS_RUNS);
    assert_row(&t, lx, ly + 5, T1_HELP_SYMS, T1_HELP_RUNS);
    assert_row(&t, lx, ly + 6, T1_VALLABEL_SYMS, T1_VALLABEL_RUNS);
    assert_row(&t, lx, ly + 7, T1_VAL_SYMS, T1_VAL_RUNS);
    assert_row(&t, lx, ly + 8, T1_BLANK_SYMS, T1_BLANK_RUNS);
    assert_row(&t, lx, ly + 9, T1_BLANK_SYMS, T1_BLANK_RUNS);
    assert_row(&t, lx, ly + 10, T1_PREVIEW_SYMS, T1_PREVIEW_RUNS);
    assert_row(&t, lx, ly + 11, T1_BLANK_SYMS, T1_BLANK_RUNS);
    assert_row(&t, lx, ly + 12, T1_BLANK_SYMS, T1_BLANK_RUNS);
    assert_row(&t, lx, ly + 13, T1_BUTTONS_SYMS, T1_BUTTONS_RUNS);
    assert_row(&t, lx, ly + 14, T1_BOT_SYMS, T1_BOT_RUNS);
}

/// Drive the app to the filtered (1) state (filter_chips.rs idiom).
fn drive_filtered() -> Harness<TableProApp> {
    let mut app = TableProApp::default();
    assert!(app.connect(4), "connect(4) failed");
    assert_eq!(app.screen, Screen::Workbench);
    let mut t = Harness::new(app, Theme::junie(), 120, 40);
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

fn bar_id(t: &Harness<TableProApp>) -> Id {
    t.app().workbench.active_key().unwrap().control("filters")
}

/// T2: title parity for Add + Edit (GREEN/GREEN). Edit via the chips-bar
/// Enter path; the top row carries no text spans in either mode.
#[test]
fn filter_editor_frame_title() {
    let mut t = drive_table(120, 40);
    let (lx, ly) = open_add(&mut t);
    assert_eq!(row_span(&t, ly + 1, lx + 3, 10), "Add filter", "add title");
    assert_row(&t, lx, ly, T1_TOP_SYMS, T1_TOP_RUNS);

    let mut t = drive_filtered();
    let bar = bar_id(&t);
    let _ = t.tab_to(bar);
    t.draw();
    let _ = t.key(KeyCode::Enter);
    t.draw();
    assert!(t.is_open(FILTER_EDITOR), "edit editor open");
    assert_eq!(
        t.app().filter_editor.as_ref().unwrap().index,
        Some(0),
        "edit index"
    );
    let (lx, ly) = dialog_origin(&t);
    assert_eq!(
        row_span(&t, ly + 1, lx + 3, 11),
        "Edit filter",
        "edit title"
    );
    assert_cell(&t, lx + 3, ly + 1, "E", WHITE, ELEVATED, BOLD);
    assert_row(&t, lx, ly, T1_TOP_SYMS, T1_TOP_RUNS);
}

/// T3a: FIELD LABEL-fg override repaints the single-mode label texts
/// (RED→GREEN). Each label is asserted while unfocused; the focused arm adds
/// BOLD over the recipe base fg (pinned in T3b).
#[test]
fn filter_editor_frame_label_recipe() {
    let label_mut = Theme::junie().override_family(Family::FIELD, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut app = TableProApp::default();
    assert!(app.connect(4), "connect(4) failed");
    let mut t = Harness::new(app, label_mut, 120, 40);
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    t.draw();
    let (lx, ly) = open_add(&mut t);
    // Focus is Value: Column + Operator labels are unfocused.
    assert_eq!(t.cell(lx + 4, ly + 3).fg, INFO, "column label");
    assert_eq!(t.cell(lx + 34, ly + 3).fg, INFO, "op label");
    // Tab to Cancel: Value label goes unfocused.
    let _ = t.key(KeyCode::Tab);
    t.draw();
    assert_eq!(editor_focus(&t), FilterFocus::Cancel);
    assert_eq!(t.cell(lx + 4, ly + 6).fg, INFO, "value label");
    assert_eq!(t.cell(lx + 4, ly + 3).fg, INFO, "column still unfocused");
}

/// T3b: FIELD LABEL-fg override repaints the Between "and" label (RED→GREEN).
#[test]
fn filter_editor_frame_and_label_recipe() {
    let label_mut = Theme::junie().override_family(Family::FIELD, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut app = TableProApp::default();
    assert!(app.connect(4), "connect(4) failed");
    let mut t = Harness::new(app, label_mut, 120, 40);
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    t.draw();
    let (lx, ly) = open_add(&mut t);
    poke_between(&mut t);
    // Tab Value -> Value2: "and" focused, Value label unfocused.
    let _ = t.key(KeyCode::Tab);
    t.draw();
    assert_eq!(editor_focus(&t), FilterFocus::Value2);
    assert_eq!(t.cell(lx + 4, ly + 6).fg, INFO, "value label");
    // The focused arm adds BOLD over the recipe base fg (INFO here).
    assert_eq!(t.cell(lx + 34, ly + 6).fg, INFO, "and focused fg");
    assert_eq!(
        t.cell(lx + 34, ly + 6).modifier.bits(),
        BOLD,
        "and focused bold"
    );
    // BackTab Value2 -> Value: "and" goes unfocused.
    let _ = t.key(KeyCode::BackTab);
    t.draw();
    assert_eq!(editor_focus(&t), FilterFocus::Value);
    assert_eq!(t.cell(lx + 34, ly + 6).fg, INFO, "and label");
}

/// T3c: FIELD HELP-fg override repaints the op help (RED→GREEN).
#[test]
fn filter_editor_frame_help_recipe() {
    let help_mut = Theme::junie().override_family(Family::FIELD, |r| {
        r.part(Part::HELP)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut app = TableProApp::default();
    assert!(app.connect(4), "connect(4) failed");
    let mut t = Harness::new(app, help_mut, 120, 40);
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    t.draw();
    let (lx, ly) = open_add(&mut t);
    assert_eq!(
        row_span(&t, ly + 5, lx + 34, 27),
        "Operators that fit the col…",
        "help text"
    );
    assert_eq!(t.cell(lx + 34, ly + 5).fg, INFO, "help fg");
    assert_eq!(t.cell(lx + 60, ly + 5).fg, INFO, "help end fg");
}

/// T3d: PANEL BORDER-fg override repaints the border AND the gutter cell,
/// proving the GUTTER no-op slot preserves recipe paint (RED→GREEN).
#[test]
fn filter_editor_frame_border_recipe() {
    let border_mut = Theme::junie().override_family(Family::PANEL, |r| {
        r.part(Part::BORDER)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut app = TableProApp::default();
    assert!(app.connect(4), "connect(4) failed");
    let mut t = Harness::new(app, border_mut, 120, 40);
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    t.draw();
    let (lx, ly) = open_add(&mut t);
    assert_cell(&t, lx, ly, "╭", INFO, ELEVATED, 0);
    assert_cell(&t, lx + 1, ly, "─", INFO, ELEVATED, 0);
    assert_cell(&t, lx + 63, ly, "╮", INFO, ELEVATED, 0);
    assert_cell(&t, lx, ly + 5, "│", INFO, ELEVATED, 0);
    assert_cell(&t, lx, ly + 14, "╰", INFO, ELEVATED, 0);
}

/// T3e: LIST LABEL-bg override repaints title+note+preview text bgs; the fg
/// leg is impossible (`label_patched` wins fg) and pinned as a guard
/// (RED→GREEN repaint, GREEN/GREEN fg guard).
#[test]
fn filter_editor_frame_list_label_recipe() {
    let label_mut = Theme::junie().override_family(Family::LIST, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_bg(Role::Info));
    });
    let mut app = TableProApp::default();
    assert!(app.connect(4), "connect(4) failed");
    let mut t = Harness::new(app, label_mut, 120, 40);
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    t.draw();
    let (lx, ly) = open_add(&mut t);
    // Title text: bg repainted, explicit Primary+BOLD fg wins.
    assert_eq!(t.cell(lx + 3, ly + 1).bg, INFO, "title bg");
    assert_eq!(t.cell(lx + 3, ly + 1).fg, WHITE, "title fg wins");
    // Preview text: bg repainted, explicit Secondary fg wins.
    assert_eq!(t.cell(lx + 3, ly + 10).bg, INFO, "preview bg");
    assert_eq!(t.cell(lx + 3, ly + 10).fg, SECOND, "preview fg wins");
    // Note text (IsNull): bg repainted, explicit Muted fg wins.
    poke_isnull(&mut t);
    assert_eq!(t.cell(lx + 4, ly + 7).bg, INFO, "note bg");
    assert_eq!(t.cell(lx + 4, ly + 7).fg, MUTED, "note fg wins");
}

/// T3f: LIST CONTAINER-bg override repaints static row-blank bgs; the fg leg
/// is impossible (the `clear_fg` instance patch wins) and pinned as a guard
/// (RED→GREEN repaint, GREEN/GREEN fg guard).
#[test]
fn filter_editor_frame_list_container_recipe() {
    let bg_mut = Theme::junie().override_family(Family::LIST, |r| {
        r.part(Part::CONTAINER)
            .base(StylePatch::new().set_bg(Role::Info));
    });
    let mut app = TableProApp::default();
    assert!(app.connect(4), "connect(4) failed");
    let mut t = Harness::new(app, bg_mut, 120, 40);
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    t.draw();
    let (lx, ly) = open_add(&mut t);
    assert_eq!(t.cell(lx + 20, ly + 1).symbol(), " ", "title blank");
    assert_eq!(t.cell(lx + 20, ly + 1).bg, INFO, "title blank bg");
    assert_eq!(t.cell(lx + 30, ly + 10).bg, INFO, "preview blank bg");

    // fg guard: the instance `clear_fg` wins over a recipe fg override, so
    // the blank keeps its deterministic bleed fg on both trees.
    let fg_mut = Theme::junie().override_family(Family::LIST, |r| {
        r.part(Part::CONTAINER)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut app = TableProApp::default();
    assert!(app.connect(4), "connect(4) failed");
    let mut t = Harness::new(app, fg_mut, 120, 40);
    for _ in 0..5 {
        let _ = t.key(KeyCode::Down);
    }
    let _ = t.key(KeyCode::Enter);
    t.draw();
    let (lx, ly) = open_add(&mut t);
    assert_eq!(t.cell(lx + 20, ly + 1).fg, BORDER, "clear_fg wins");
}

/// Pin one cycle stop: editor focus, label bolds, the six control focus
/// flags, and empty static stops. `between` selects the Value2 label.
fn assert_stop(
    t: &Harness<TableProApp>,
    lx: u16,
    ly: u16,
    want: FilterFocus,
    between: bool,
    step: &str,
) {
    assert_eq!(editor_focus(t), want, "{step}: editor focus");
    let labels: [(FilterFocus, u16, u16); 4] = [
        (FilterFocus::Column, lx + 4, ly + 3),
        (FilterFocus::Op, lx + 34, ly + 3),
        (FilterFocus::Value, lx + 4, ly + 6),
        (FilterFocus::Value2, lx + 34, ly + 6),
    ];
    for (stop, x, y) in labels {
        if stop == FilterFocus::Value2 && !between {
            continue;
        }
        let cell = t.cell(x, y);
        if stop == want {
            assert_eq!(cell.fg, WHITE, "{step}: focused label fg");
            assert_eq!(cell.modifier.bits(), BOLD, "{step}: focused bold");
        } else {
            assert_eq!(cell.fg, SECOND, "{step}: unfocused label fg");
            assert_eq!(cell.modifier.bits(), 0, "{step}: unfocused plain");
        }
    }
    let controls = [
        (FilterFocus::Column, FILTER_COL),
        (FilterFocus::Op, FILTER_OP),
        (FilterFocus::Value, FILTER_VALUE),
        (FilterFocus::Value2, FILTER_VALUE2),
        (FilterFocus::Cancel, FILTER_CANCEL),
        (FilterFocus::Apply, FILTER_APPLY),
    ];
    for (stop, id) in controls {
        let flags = t.state_of(id);
        if stop == want {
            assert!(
                flags.contains(termrock::StateFlags::FOCUSED),
                "{step}: {stop:?} holds focus ({flags:?})"
            );
        } else {
            assert!(
                !flags.contains(termrock::StateFlags::FOCUSED),
                "{step}: {stop:?} unfocused ({flags:?})"
            );
        }
    }
    for (name, id) in [
        ("title", TITLE_ID),
        ("note", NOTE_ID),
        ("preview", PREVIEW_ID),
    ] {
        assert_eq!(
            t.state_of(id),
            termrock::StateFlags::empty(),
            "{step}: static {name} never holds focus"
        );
    }
}

/// T4a: single-value Tab/BackTab cycling incl. both TITLE-skipping wraps
/// (GREEN/GREEN guard).
#[test]
fn filter_editor_frame_cycle_single() {
    let mut t = drive_table(120, 40);
    let (lx, ly) = open_add(&mut t);
    assert_stop(&t, lx, ly, FilterFocus::Value, false, "open");
    for (i, want) in [
        FilterFocus::Cancel,
        FilterFocus::Apply,
        FilterFocus::Column,
        FilterFocus::Op,
        FilterFocus::Value,
        FilterFocus::Cancel,
    ]
    .into_iter()
    .enumerate()
    {
        let _ = t.key(KeyCode::Tab);
        t.draw();
        assert_stop(&t, lx, ly, want, false, &format!("tab{i}"));
    }
    for (i, want) in [
        FilterFocus::Value,
        FilterFocus::Op,
        FilterFocus::Column,
        FilterFocus::Apply,
        FilterFocus::Cancel,
        FilterFocus::Value,
    ]
    .into_iter()
    .enumerate()
    {
        let _ = t.key(KeyCode::BackTab);
        t.draw();
        assert_stop(&t, lx, ly, want, false, &format!("btab{i}"));
    }
}

/// T4b: Between cycling incl. the Value2 stop and PREVIEW skips (GREEN/GREEN).
#[test]
fn filter_editor_frame_cycle_between() {
    let mut t = drive_table(120, 40);
    let (lx, ly) = open_add(&mut t);
    poke_between(&mut t);
    assert_stop(&t, lx, ly, FilterFocus::Value, true, "open");
    let _ = t.key(KeyCode::Tab);
    t.draw();
    assert_stop(&t, lx, ly, FilterFocus::Value2, true, "tab-val2");
    let _ = t.key(KeyCode::Tab);
    t.draw();
    assert_stop(&t, lx, ly, FilterFocus::Cancel, true, "tab-cancel");
    let _ = t.key(KeyCode::BackTab);
    t.draw();
    assert_stop(&t, lx, ly, FilterFocus::Value2, true, "btab-val2");
    let _ = t.key(KeyCode::BackTab);
    t.draw();
    assert_stop(&t, lx, ly, FilterFocus::Value, true, "btab-val");
}

/// T4c: no-value cycling: Op→Cancel via NOTE, Cancel→Op via PREVIEW
/// (GREEN/GREEN guard).
#[test]
fn filter_editor_frame_cycle_novalue() {
    let mut t = drive_table(120, 40);
    let (lx, ly) = open_add(&mut t);
    poke_isnull(&mut t);
    // Column/Op labels only; Value labels are absent in no-value mode.
    assert_eq!(t.cell(lx + 4, ly + 3).fg, SECOND, "column unfocused");
    assert_eq!(t.cell(lx + 34, ly + 3).fg, WHITE, "op focused");
    assert_eq!(
        t.cell(lx + 34, ly + 3).modifier.bits(),
        BOLD,
        "op focused bold"
    );
    let _ = t.key(KeyCode::Tab);
    t.draw();
    assert_eq!(editor_focus(&t), FilterFocus::Cancel, "tab skips NOTE");
    assert_eq!(t.cell(lx + 34, ly + 3).fg, SECOND, "op unfocused");
    let _ = t.key(KeyCode::BackTab);
    t.draw();
    assert_eq!(editor_focus(&t), FilterFocus::Op, "btab skips PREVIEW");
    assert_eq!(t.cell(lx + 34, ly + 3).fg, WHITE, "op focused again");
    for (name, id) in [
        ("title", TITLE_ID),
        ("note", NOTE_ID),
        ("preview", PREVIEW_ID),
    ] {
        assert_eq!(
            t.state_of(id),
            termrock::StateFlags::empty(),
            "static {name} never holds focus"
        );
    }
}

// T5: Between twins, dumped from the pristine base archive.
const T5_LABELS_SYMS: &str = "│   Value                         and                          │";
const T5_LABELS_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 1, C38, ELEVATED, 0),
    (4, 26, WHITE, ELEVATED, BOLD),
    (30, 4, BORDER, ELEVATED, 0),
    (34, 27, SECOND, ELEVATED, 0),
    (61, 2, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T5_VALS_SYMS: &str = "│ ▎ value                         value                        │";
const T5_VALS_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, ACCENT, FIELD_BG, 0),
    (3, 1, WHITE, FIELD_BG, 0),
    (4, 5, MUTED, FIELD_BG, 0),
    (9, 21, WHITE, FIELD_BG, 0),
    (30, 2, BORDER, ELEVATED, 0),
    (32, 1, FIELD_BG, FIELD_BG, 0),
    (33, 1, WHITE, FIELD_BG, 0),
    (34, 5, MUTED, FIELD_BG, 0),
    (39, 22, WHITE, FIELD_BG, 0),
    (61, 2, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];

/// T5: Between twins — Value/and labels + 28/29 geometry (GREEN/GREEN).
#[test]
fn filter_editor_frame_between_twins() {
    let mut t = drive_table(120, 40);
    let (lx, ly) = open_add(&mut t);
    poke_between(&mut t);
    assert_row(&t, lx, ly + 6, T5_LABELS_SYMS, T5_LABELS_RUNS);
    assert_row(&t, lx, ly + 7, T5_VALS_SYMS, T5_VALS_RUNS);
    let val = t.area_of(FILTER_VALUE).expect("value area");
    let val2 = t.area_of(FILTER_VALUE2).expect("value2 area");
    assert_eq!((val.width, val2.width), (28, 29), "twin widths");
    assert_eq!(val.x, lx + 2, "value x");
    assert_eq!(val2.x, lx + 32, "value2 x");
}

// T6: no-value note row, dumped from the pristine base archive.
const T6_NOTE_SYMS: &str = "│   No value needed for this operator                          │";
const T6_NOTE_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 1, C38, ELEVATED, 0),
    (4, 33, MUTED, ELEVATED, 0),
    (37, 12, BORDER, ELEVATED, 0),
    (49, 14, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];

/// T6: no-value note + live preview (GREEN/GREEN). The dx2 marker cell is a
/// blank on Elevated (visible, not clipped — pins the B2 inset); its MUTED fg
/// is deterministic bleed shared by both trees (an unpatched `clear_fg`
/// would paint WHITE there instead).
#[test]
fn filter_editor_frame_novalue_note() {
    let mut t = drive_table(120, 40);
    let (lx, ly) = open_add(&mut t);
    poke_isnull(&mut t);
    assert_row(&t, lx, ly + 7, T6_NOTE_SYMS, T6_NOTE_RUNS);
    assert_cell(&t, lx + 2, ly + 7, " ", MUTED, ELEVATED, 0);
    assert_eq!(
        row_span(&t, ly + 10, lx + 3, 16),
        "WHERE id IS NULL",
        "live preview"
    );
}

/// T7: preview draft-vs-committed (GREEN/GREEN). The preview computation is
/// pure data over shared component state — untouched by the conversion.
#[test]
fn filter_editor_frame_preview_draft() {
    let mut t = drive_table(120, 40);
    let (lx, ly) = open_add(&mut t);
    let _ = t.click_id(FILTER_VALUE);
    t.draw();
    assert!(
        t.app()
            .filter_editor
            .as_ref()
            .unwrap()
            .value_state
            .is_editing(),
        "click begins the edit"
    );
    for c in "pe".chars() {
        let _ = t.key(KeyCode::Char(c));
    }
    t.draw();
    assert_eq!(
        row_span(&t, ly + 10, lx + 3, 15),
        "WHERE id = 'pe'",
        "draft preview live"
    );
    let _ = t.key(KeyCode::Esc);
    t.draw();
    assert!(t.is_open(FILTER_EDITOR), "esc keeps the editor");
    assert_eq!(
        row_span(&t, ly + 10, lx + 3, 13),
        "WHERE id = ''",
        "esc reverts to committed"
    );
    let _ = t.click_id(FILTER_VALUE);
    t.draw();
    for c in "pen".chars() {
        let _ = t.key(KeyCode::Char(c));
    }
    let _ = t.key(KeyCode::Enter);
    t.draw();
    assert!(!t.is_open(FILTER_EDITOR), "enter applies and closes");
    let filters = &t.app().workbench.active_table().unwrap().filters;
    assert_eq!(
        filters.last().unwrap().chip_label(),
        "id = 'pen'",
        "applied filter"
    );
}

/// T8a: clicks inside the dialog are no-ops; backdrop clicks still dismiss
/// (RED→GREEN + a GREEN/GREEN dismiss guard).
///
/// DEVIATION from plan §6 (T8 GREEN/GREEN): the plan's F2 premise — "clicks
/// today are no-ops (bare `paint_str`)" — is empirically false. Probed on the
/// pristine base: a click on any dead dialog area (title/note/preview rows,
/// border, blank rows) falls through the bare paint and DISMISSES the modal
/// (`open=false`, `Consumed`, no diagnostics). Staged captures inside clicks
/// via the registered `List` stops (§5.2 Pointer bounce) and Panel/Field
/// decor, so they no-op; true backdrop clicks still dismiss (pinned below —
/// the dismiss path itself is no regression). The §5.2 bounce behavior is
/// exactly as planned; only the matrix color changes.
#[test]
fn filter_editor_frame_click_noop() {
    let mut t = drive_table(120, 40);
    let (lx, ly) = open_add(&mut t);
    assert_eq!(t.focus(), Some(FILTER_VALUE), "runtime focus");
    for (x, y, what) in [
        (lx + 5, ly + 1, "title"),
        (lx + 10, ly + 10, "preview"),
        (lx, ly + 10, "preview-row border"),
        (lx + 30, ly + 8, "blank row"),
    ] {
        let _ = t.click(x, y);
        t.draw();
        assert_eq!(editor_focus(&t), FilterFocus::Value, "{what}: editor");
        assert_eq!(t.focus(), Some(FILTER_VALUE), "{what}: runtime");
    }
    poke_isnull(&mut t);
    let _ = t.click(lx + 10, ly + 7);
    t.draw();
    assert_eq!(editor_focus(&t), FilterFocus::Op, "note: editor");
    assert_eq!(t.focus(), Some(FILTER_OP), "note: runtime");
    // Tab still cycles after the clicks.
    let _ = t.key(KeyCode::BackTab);
    t.draw();
    assert_eq!(editor_focus(&t), FilterFocus::Column, "btab works");
    let _ = t.key(KeyCode::Tab);
    t.draw();
    assert_eq!(editor_focus(&t), FilterFocus::Op, "tab works");
    assert!(t.diagnostics().is_empty(), "no diagnostics");
    // Backdrop dismiss is preserved on both trees (GREEN/GREEN guard).
    let _ = t.click(0, 39);
    t.draw();
    assert!(!t.is_open(FILTER_EDITOR), "backdrop still dismisses");
    assert!(t.diagnostics().is_empty(), "no diagnostics");
}

/// T8b: hovering a static row repaints text + blanks via HoverSurface
/// (RED→GREEN; PTY-blind accepted stock behavior).
#[test]
fn filter_editor_frame_hover_static() {
    let mut t = drive_table(120, 40);
    let (lx, ly) = open_add(&mut t);
    let _ = harness_typed_input::hover(&mut t, lx + 5, ly + 1);
    t.draw();
    assert_eq!(t.cell(lx + 5, ly + 1).bg, HOVER_BG, "hover text bg");
    assert_eq!(t.cell(lx + 5, ly + 1).fg, WHITE, "hover text fg kept");
    assert_eq!(t.cell(lx + 20, ly + 1).bg, HOVER_BG, "hover blank bg");
    assert_eq!(editor_focus(&t), FilterFocus::Value, "hover keeps focus");
    let _ = harness_typed_input::hover(&mut t, 0, 0);
    t.draw();
    assert_eq!(t.cell(lx + 5, ly + 1).bg, ELEVATED, "unhover restores");
    assert!(t.diagnostics().is_empty(), "no diagnostics");
}

// T9: 80x24 Add dialog, dumped from the pristine base archive.
const T9_BLANK_A_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 3, MUTED, ELEVATED, 0),
    (4, 12, BORDER, ELEVATED, 0),
    (16, 47, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T9_BLANK_B_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 36, BORDER, ELEVATED, 0),
    (39, 24, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T9_TITLE_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 2, MUTED, ELEVATED, 0),
    (3, 10, WHITE, ELEVATED, BOLD),
    (13, 50, C38, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T9_LABELS_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 3, C38, ELEVATED, 0),
    (4, 6, SECOND, ELEVATED, 0),
    (10, 24, C38, ELEVATED, 0),
    (34, 8, SECOND, ELEVATED, 0),
    (42, 21, C38, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T9_SELECTS_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, MUTED, ELEVATED, 0),
    (2, 1, FIELD_BG, FIELD_BG, 0),
    (3, 25, WHITE, FIELD_BG, 0),
    (28, 1, SECOND, FIELD_BG, 0),
    (29, 1, WHITE, FIELD_BG, 0),
    (30, 2, MUTED, ELEVATED, 0),
    (32, 1, FIELD_BG, FIELD_BG, 0),
    (33, 26, WHITE, FIELD_BG, 0),
    (59, 1, SECOND, FIELD_BG, 0),
    (60, 1, WHITE, FIELD_BG, 0),
    (61, 2, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T9_HELP_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 2, MUTED, ELEVATED, 0),
    (3, 31, C38, ELEVATED, 0),
    (34, 27, MUTED, ELEVATED, 0),
    (61, 2, C38, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T9_VALLABEL_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 1, BORDER, ELEVATED, 0),
    (4, 57, WHITE, ELEVATED, BOLD),
    (61, 2, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T9_VAL_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, ACCENT, FIELD_BG, 0),
    (3, 1, WHITE, FIELD_BG, 0),
    (4, 5, MUTED, FIELD_BG, 0),
    (9, 52, WHITE, FIELD_BG, 0),
    (61, 2, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T9_PREVIEW_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 13, SECOND, ELEVATED, 0),
    (16, 23, BORDER, ELEVATED, 0),
    (39, 24, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];
const T9_BUTTONS_RUNS: &[StyleRun] = &[
    (0, 1, BORDER, ELEVATED, 0),
    (1, 1, C38, ELEVATED, 0),
    (2, 1, MUTED, ELEVATED, 0),
    (3, 36, BORDER, ELEVATED, 0),
    (39, 1, MUTED, ELEVATED, 0),
    (40, 1, ELEVATED, ELEVATED, 0),
    (41, 7, SECOND, ELEVATED, 0),
    (48, 1, MUTED, ELEVATED, 0),
    (49, 1, ACCENT, ACCENT, 0),
    (50, 11, BTN_DARK, ACCENT, BOLD),
    (61, 2, MUTED, ELEVATED, 0),
    (63, 1, BORDER, ELEVATED, 0),
];

/// T9: 80x24 Add pin + full-flow empty diagnostics (GREEN/GREEN). The empty
/// diagnostics pin no `UnknownPart` from the CONTAINER patches.
#[test]
fn filter_editor_frame_add_80() {
    let mut t = drive_table(80, 24);
    let (lx, ly) = open_add(&mut t);
    assert_row(&t, lx, ly, T1_TOP_SYMS, T1_TOP_RUNS);
    assert_row(&t, lx, ly + 1, T1_TITLE_SYMS, T9_TITLE_RUNS);
    assert_row(&t, lx, ly + 2, T1_BLANK_SYMS, T9_BLANK_A_RUNS);
    assert_row(&t, lx, ly + 3, T1_LABELS_SYMS, T9_LABELS_RUNS);
    assert_row(&t, lx, ly + 4, T1_SELECTS_SYMS, T9_SELECTS_RUNS);
    assert_row(&t, lx, ly + 5, T1_HELP_SYMS, T9_HELP_RUNS);
    assert_row(&t, lx, ly + 6, T1_VALLABEL_SYMS, T9_VALLABEL_RUNS);
    assert_row(&t, lx, ly + 7, T1_VAL_SYMS, T9_VAL_RUNS);
    assert_row(&t, lx, ly + 8, T1_BLANK_SYMS, T9_BLANK_B_RUNS);
    assert_row(&t, lx, ly + 9, T1_BLANK_SYMS, T9_BLANK_B_RUNS);
    assert_row(&t, lx, ly + 10, T1_PREVIEW_SYMS, T9_PREVIEW_RUNS);
    assert_row(&t, lx, ly + 11, T1_BLANK_SYMS, T9_BLANK_B_RUNS);
    assert_row(&t, lx, ly + 12, T1_BLANK_SYMS, T9_BLANK_B_RUNS);
    assert_row(&t, lx, ly + 13, T1_BUTTONS_SYMS, T9_BUTTONS_RUNS);
    assert_row(&t, lx, ly + 14, T1_BOT_SYMS, T1_BOT_RUNS);
    // Full flow: cycle, edit, cancel; diagnostics stay empty.
    for _ in 0..6 {
        let _ = t.key(KeyCode::Tab);
    }
    for _ in 0..6 {
        let _ = t.key(KeyCode::BackTab);
    }
    let _ = t.click_id(FILTER_VALUE);
    for c in "ab".chars() {
        let _ = t.key(KeyCode::Char(c));
    }
    let _ = t.key(KeyCode::Esc);
    t.draw();
    let _ = t.key(KeyCode::Esc);
    t.draw();
    assert!(!t.is_open(FILTER_EDITOR), "idle esc closes");
    assert!(t.diagnostics().is_empty(), "no diagnostics");
}

/// T10: Edit leg via the chips-bar Enter path (GREEN/GREEN).
#[test]
fn filter_editor_frame_edit() {
    let mut t = drive_filtered();
    let bar = bar_id(&t);
    let _ = t.tab_to(bar);
    t.draw();
    let _ = t.key(KeyCode::Enter);
    t.draw();
    assert!(t.is_open(FILTER_EDITOR), "edit editor open");
    let editor = t.app().filter_editor.as_ref().unwrap();
    assert_eq!(editor.index, Some(0), "edit index");
    assert_eq!(editor.focus, FilterFocus::Apply, "prefill focus");
    let (lx, ly) = dialog_origin(&t);
    assert_eq!(
        row_span(&t, ly + 1, lx + 3, 11),
        "Edit filter",
        "edit title"
    );
    assert_cell(&t, lx + 3, ly + 1, "E", WHITE, ELEVATED, BOLD);
    assert!(
        t.find("Update filter").is_some(),
        "confirm relabels for edit"
    );
    assert_eq!(t.focus(), Some(FILTER_APPLY), "apply focused");
}
