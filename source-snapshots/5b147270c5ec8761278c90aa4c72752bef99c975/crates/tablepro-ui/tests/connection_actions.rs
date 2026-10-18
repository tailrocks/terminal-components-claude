//! WI-TABLEPRO-CONNECTION-ACTIONS: the details-card action row is 4 stock `Button`s.
//!
//! The legacy `paint_action_button` overpainter in `tablepro-ui` is deleted;
//! Connect/Edit/Duplicate/Delete render through the reusable `termrock::Button`
//! (`Button::draw` paints `Part::GUTTER` + `Part::LABEL` per row) with zero
//! patches: Connect→`PRIMARY`, Edit→`DEFAULT`, Duplicate→`SUBTLE`,
//! Delete→`DANGER`. Hardcoded literals below are captured from base
//! `ea7a054af14c` control dumps (120x40/100x30/160x50 row 17 + 72x20/80x24
//! absence + behavior probes); no test loads a frame file.
//!
//! * T1 pins the 120x40 row-17 symbols + per-cell styles exactly.
//! * T2 pins the shifted wide rows (100x30 x37-77, 160x50 x45-85; control
//!   shows the 41-cell span pixel-identical at all three sizes).
//! * T3 pins the small sizes (row absent: details-only marker gone, no
//!   action labels on row 17).
//! * T4 pins exact-once activation per button (Enter/Space/click each fire
//!   exactly one effect).
//! * T5 pins untouched shortcuts + click routing (Edit-click opens the form
//!   without card double-fire, gap-click still connects, outside-click no-op).
//! * T6 pins stock ownership: real-ring reachability in draw order, a
//!   theme LABEL mutation repainting Connect only, and the focused `▎`.
//!   T6(c) styles are disclosed non-frozen (no base oracle: base has no
//!   focusable buttons; behavior follows the button contract).

use tablepro_ui::{Screen, TableProApp};
use termrock::{Color, Family, Id, KeyCode, Part, Role, StylePatch, Theme, Variant};
use termrock_test_support::Harness;

/// Literal id contract — must match product Site 0
/// (`tablepro.connections.action.<name>`). Literals (not product consts) so
/// T4–T6 compile on base and fail there authentically at runtime.
const CONNECT: Id = Id::root("tablepro.connections.action.connect");
const EDIT: Id = Id::root("tablepro.connections.action.edit");
const DUPLICATE: Id = Id::root("tablepro.connections.action.duplicate");
const DELETE: Id = Id::root("tablepro.connections.action.delete");

const ACCENT: Color = Color::Rgb(72, 224, 84);
const ON_ACCENT: Color = Color::Rgb(25, 25, 28);
const OVERLAY: Color = Color::Rgb(39, 39, 42);
const WHITE: Color = Color::Rgb(255, 255, 255);
const CARD: Color = Color::Rgb(17, 17, 17);
const SECOND: Color = Color::Rgb(179, 179, 179);
const DANGER_FG: Color = Color::Rgb(228, 69, 69);

/// The 41-cell action span (Connect w9, Edit w6, Duplicate w11, Delete w9,
/// 2-cell gaps between). Control bytes are identical at 120x40 (x44),
/// 100x30 (x37) and 160x50 (x45).
const ROW_SYMBOLS: &str = " Connect    Edit    Duplicate    Delete… ";

/// (fg, bg, mods) of the span cell at offset `i` (0-based into `ROW_SYMBOLS`).
fn action_style(i: usize) -> (Color, Color, u16) {
    match i {
        0 => (ACCENT, ACCENT, 0),
        1..=8 => (ON_ACCENT, ACCENT, 1),
        9..=10 => (WHITE, CARD, 0),
        11 => (OVERLAY, OVERLAY, 0),
        12..=16 => (WHITE, OVERLAY, 0),
        17..=18 => (WHITE, CARD, 0),
        19 => (CARD, CARD, 0),
        20..=29 => (SECOND, CARD, 0),
        30..=31 => (WHITE, CARD, 0),
        32 => (OVERLAY, OVERLAY, 0),
        33..=40 => (DANGER_FG, OVERLAY, 0),
        _ => unreachable!("offset {i} outside the 41-cell action span"),
    }
}

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

fn assert_action_span(t: &Harness<TableProApp>, x0: u16, tag: &str) {
    assert_eq!(
        row_span(t, 17, x0, x0 + 40),
        ROW_SYMBOLS,
        "{tag}: row-17 action span symbols"
    );
    let syms: Vec<char> = ROW_SYMBOLS.chars().collect();
    for (i, sym) in syms.iter().enumerate() {
        let (fg, bg, mods) = action_style(i);
        assert_cell(t, x0 + i as u16, 17, &sym.to_string(), fg, bg, mods);
    }
}

/// T1: 120x40 boot pins the full row-17 span (symbols + per-cell styles).
#[test]
fn connection_actions_row_120() {
    let t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    assert_action_span(&t, 44, "120x40");
}

/// T2: shifted sizes carry the identical span (100x30 x37-77, 160x50 x45-85).
#[test]
fn connection_actions_row_shifted() {
    for (w, h, x0) in [(100u16, 30u16, 37u16), (160u16, 50u16, 45u16)] {
        let t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
        assert_action_span(&t, x0, &format!("{w}x{h}"));
    }
}

/// T3: narrow sizes show no details card and no action row.
#[test]
fn connection_actions_absent_small() {
    for (w, h) in [(72u16, 20u16), (80u16, 24u16)] {
        let t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
        assert!(
            t.find("SSL / SSH").is_none(),
            "{w}x{h}: details-only marker must be absent"
        );
        assert!(
            t.find("Delete…").is_none(),
            "{w}x{h}: action-only label must be absent"
        );
        let row = row_span(&t, 17, 0, w - 1);
        for needle in ["Connect", "Duplicate", "Delete…"] {
            assert!(
                !row.contains(needle),
                "{w}x{h}: row 17 must not contain {needle:?} (got {row:?})"
            );
        }
    }
}

/// T4: Connect fires exactly one connect per gesture (clean boot has no
/// unsaved work, so each gesture lands directly on the workbench).
#[test]
fn connection_actions_connect_once() {
    for gesture in ["Enter", "Space", "click"] {
        let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        match gesture {
            "click" => {
                let _ = t.click(48, 17);
            }
            "Enter" => {
                assert!(t.tab_to(CONNECT), "ring must reach Connect");
                let _ = t.key(KeyCode::Enter);
            }
            _ => {
                assert!(t.tab_to(CONNECT), "ring must reach Connect");
                let _ = t.key(KeyCode::Char(' '));
            }
        }
        assert_eq!(
            t.app().screen(),
            Screen::Workbench,
            "Connect via {gesture} must land on the workbench"
        );
        assert_eq!(
            t.app().workbench().tabs().len(),
            1,
            "Connect via {gesture} must open exactly one tab"
        );
        assert!(
            t.app().status().starts_with("Connected to "),
            "Connect via {gesture} must report the connection (got {:?})",
            t.app().status()
        );
    }
}

/// T4: Edit fires exactly one form-open per gesture (screen stays put).
#[test]
fn connection_actions_edit_once() {
    for gesture in ["Enter", "Space", "click"] {
        let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        match gesture {
            "click" => {
                let _ = t.click(57, 17);
            }
            "Enter" => {
                assert!(t.tab_to(EDIT), "ring must reach Edit");
                let _ = t.key(KeyCode::Enter);
            }
            _ => {
                assert!(t.tab_to(EDIT), "ring must reach Edit");
                let _ = t.key(KeyCode::Char(' '));
            }
        }
        assert!(
            t.app().connection_form_open(),
            "Edit via {gesture} must open the connection form"
        );
        assert!(
            t.find("Name").is_some(),
            "Edit via {gesture} must show the Name field"
        );
        assert_eq!(
            t.app().screen(),
            Screen::Connections,
            "Edit via {gesture} must stay on Connections"
        );
    }
}

/// T4: Duplicate fires exactly one copy per gesture (a double-fire would
/// insert two `(Copy)` rows).
#[test]
fn connection_actions_duplicate_once() {
    for gesture in ["Enter", "Space", "click"] {
        let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        assert_eq!(t.count("(Copy)"), 0, "clean boot must have no copies");
        match gesture {
            "click" => {
                let _ = t.click(68, 17);
            }
            "Enter" => {
                assert!(t.tab_to(DUPLICATE), "ring must reach Duplicate");
                let _ = t.key(KeyCode::Enter);
            }
            _ => {
                assert!(t.tab_to(DUPLICATE), "ring must reach Duplicate");
                let _ = t.key(KeyCode::Char(' '));
            }
        }
        assert_eq!(
            t.count("(Copy)"),
            1,
            "Duplicate via {gesture} must insert exactly one copy"
        );
        assert_eq!(
            t.app().status(),
            "Duplicated",
            "Duplicate via {gesture} must report"
        );
    }
}

/// T4: Delete fires exactly one dialog per gesture.
#[test]
fn connection_actions_delete_once() {
    for gesture in ["Enter", "Space", "click"] {
        let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        match gesture {
            "click" => {
                let _ = t.click(80, 17);
            }
            "Enter" => {
                assert!(t.tab_to(DELETE), "ring must reach Delete");
                let _ = t.key(KeyCode::Enter);
            }
            _ => {
                assert!(t.tab_to(DELETE), "ring must reach Delete");
                let _ = t.key(KeyCode::Char(' '));
            }
        }
        assert!(
            t.find("Delete connection?").is_some(),
            "Delete via {gesture} must open the delete dialog"
        );
        assert_eq!(
            t.count("Delete connection?"),
            1,
            "Delete via {gesture} must open exactly one dialog"
        );
    }
}

/// T5: shortcuts untouched + click routing (Edit-click opens the form with
/// no card double-fire, gap-click still connects, outside-click is a no-op).
#[test]
fn connection_actions_shortcuts_and_clicks() {
    // Tree Enter still connects.
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = t.key(KeyCode::Enter);
    assert_eq!(
        t.app().screen(),
        Screen::Workbench,
        "tree Enter must still connect"
    );
    assert_eq!(
        t.app().workbench().tabs().len(),
        1,
        "tree Enter must open exactly one tab"
    );
    // `e` still edits.
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = t.key(KeyCode::Char('e'));
    assert!(
        t.app().connection_form_open(),
        "`e` must still open the edit form"
    );
    assert!(t.find("Name").is_some(), "`e` must show the Name field");
    // `d` still deletes.
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = t.key(KeyCode::Char('d'));
    assert!(
        t.find("Delete connection?").is_some(),
        "`d` must still open the delete dialog"
    );
    // Ctrl+D still duplicates.
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = t.ctrl('d');
    assert_eq!(t.count("(Copy)"), 1, "Ctrl+D must still duplicate once");
    assert_eq!(t.app().status(), "Duplicated", "Ctrl+D must report");
    // Gap-click still connects through the card.
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = t.click(53, 17);
    assert_eq!(
        t.app().screen(),
        Screen::Workbench,
        "gap-click must still connect through the card"
    );
    // Outside-click is a no-op (card spans x42-112 at 120x40).
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = t.click(115, 17);
    assert_eq!(
        t.app().screen(),
        Screen::Connections,
        "outside-click must not leave Connections"
    );
    assert!(
        !t.app().connection_form_open(),
        "outside-click must not open the form"
    );
    assert_eq!(t.count("(Copy)"), 0, "outside-click must not duplicate");
    assert!(
        t.find("Delete connection?").is_none(),
        "outside-click must not delete"
    );
    // Button click routes to the button only (no `details_clicked` double-fire).
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = t.click(57, 17);
    assert!(
        t.app().connection_form_open(),
        "Edit-click must open the connection form"
    );
    assert_eq!(
        t.app().screen(),
        Screen::Connections,
        "Edit-click must not also connect through the card"
    );
}

/// T6: ownership — (a) real-ring reachability in draw order with tree-first
/// boot focus; (b) a theme LABEL mutation repaints Connect only (product
/// keeps zero patches); (c) focused `▎` + bold (disclosed non-frozen).
#[test]
fn connection_actions_ownership() {
    // (a) Boot focus is still the tree; all 4 ids reachable in draw order.
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    assert_eq!(
        format!("{:?}", t.focus()),
        "Some(tablepro.connections.list)",
        "boot focus must stay on the tree"
    );
    for (id, name) in [
        (CONNECT, "Connect"),
        (EDIT, "Edit"),
        (DUPLICATE, "Duplicate"),
        (DELETE, "Delete"),
    ] {
        assert!(t.tab_to(id), "real Tab ring must reach {name}");
    }
    let ring: Vec<String> = t
        .ring()
        .entries()
        .iter()
        .map(|e| format!("{:?}", e.id))
        .collect();
    let pos = |want: &str| {
        ring.iter()
            .position(|e| e == want)
            .unwrap_or_else(|| panic!("ring {ring:?} must contain {want}"))
    };
    assert!(
        pos("tablepro.connections.action.connect") < pos("tablepro.connections.action.edit")
            && pos("tablepro.connections.action.edit")
                < pos("tablepro.connections.action.duplicate")
            && pos("tablepro.connections.action.duplicate")
                < pos("tablepro.connections.action.delete"),
        "ring must list the buttons in draw order (got {ring:?})"
    );
    // (b) LABEL mutation repaints the Connect row only.
    let mutated = Theme::junie().override_variant(Family::BUTTON, Variant::PRIMARY, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let tm = Harness::new(TableProApp::default(), mutated, 120, 40);
    assert_ne!(
        tm.cell(45, 17).fg,
        ON_ACCENT,
        "LABEL override must repaint the Connect row"
    );
    assert_eq!(
        tm.cell(56, 17).fg,
        WHITE,
        "LABEL override must not touch Edit"
    );
    assert_eq!(
        tm.cell(64, 17).fg,
        SECOND,
        "LABEL override must not touch Duplicate"
    );
    assert_eq!(
        tm.cell(77, 17).fg,
        DANGER_FG,
        "LABEL override must not touch Delete"
    );
    // (c) Focused focus-bar + bold (disclosed non-frozen: styles follow
    // the button contract — FocusBar fg is Primary for PRIMARY, Focus
    // green otherwise; labels keep/carry bold).
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    assert!(t.tab_to(CONNECT), "must reach Connect");
    assert_cell(&t, 44, 17, "▎", WHITE, ACCENT, 0);
    assert_cell(&t, 45, 17, "C", ON_ACCENT, ACCENT, 1);
    assert!(t.tab_to(EDIT), "must reach Edit");
    assert_cell(&t, 55, 17, "▎", ACCENT, OVERLAY, 0);
    assert_cell(&t, 56, 17, "E", WHITE, OVERLAY, 1);
    assert_eq!(
        t.cell(44, 17).symbol(),
        " ",
        "unfocused Connect must lose the focus bar"
    );
}
