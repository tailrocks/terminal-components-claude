//! Control-state parity mirror: candidate-`termrock` rendering of the W01-W24
//! control-state slice of the vendored case registry.
//!
//! Single W vocabulary (Q03 reconcile). There is exactly one W namespace and
//! it lives in the vendored registry `tests/conformance/required_cases.json`
//! (read-only reference, sha256 `04ebb893bdf4f900…`). Every `Wxx-yy` id cited
//! anywhere in this file is a registry case id; no suite-side numbering
//! exists. The two counts seen in earlier docs describe one registry:
//!
//! * **222** = every registry W case, components W01-W45
//!   (`component_cases_count: 222`).
//! * **117** = the W01-W24 control-states slice mirrored by this suite:
//!   W01:4 W02:5 W03:4 W04:4 W05:4 W06:5 W07:4 W08:7 W09:6 W10:5 W11:5 W12:6
//!   W13:5 W14:5 W15:5 W16:5 W17:5 W18:5 W19:4 W20:5 W21:5 W22:5 W23:5 W24:4.
//!   The remaining 105 registry W cases (W25-W45) belong to other suites.
//!
//! Mirror status within the 117-case slice (see `parity_burndown.rs`, which
//! gates these counts executably):
//!
//! * 84 registry ids (W01-W17, every id cited at least once) have a
//!   behavioral mirror in this file.
//! * 33 registry ids (W18-W24) resolve through the ownership gate below but
//!   have no behavioral mirror yet.
//! * 24 mirror tests are `#[ignore]`d, deferring 26 registry-case references
//!   (22 single-case plus the dual-case `W05-02/W05-04` and `W06-02/W06-05`
//!   records). Each ignore maps to one `BD-xx` burndown item.
//!
//! No W coverage is claimed while any ignore remains: passing the
//! non-ignored subset proves nothing about the deferred cases, and the
//! W18-W24 slice is ownership-gated only. (Grep-count note: raw `PARITY`
//! matches exceed 26 because they also hit this doc template and two
//! `panic!` bodies inside already-ignored tests; those are not deferrals.)
//!
//! Reference contract (states, combinations, precedence):
//!
//! * same control states: normal, focus, hover, focus+hover, pressed and
//!   disabled, plus the component-owned axes (checked/on, busy/loading/error,
//!   editing, selected/current/cursor, open/closed, armed);
//! * same combinations: checked-but-unfocused, chosen-vs-cursor-vs-hovered,
//!   highlight-vs-committed, cursor-vs-selected-row-vs-hovered-row;
//! * same precedence: disabled suppresses focus/hover/press feedback and
//!   rejects every activation; read-only stays reachable but never edits;
//!   a modal/topmost layer owns paste, Escape and outside click.
//!
//! Adapted to the candidate API (`termrock` facade re-exports, `Runtime` +
//! `Stub` + `draw_scene` for inert paint states, `Harness` for interaction).
//! Assertions are the reference assertions, unweakened: where the candidate
//! cannot yet express a reference case, the test is `#[ignore]`d with a
//! `PARITY <case>: <reason>` record instead of weakening the expectation.
//!
//! Ownership map: `component-ownership.json` (all 45 components, `W01`-`W45`)
//! is loaded with `include_str!` and each component is proven to resolve
//! through its owner crate's facade module. Unknown component ids fail the
//! gate: the map must match the registry vocabulary exactly.

use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Duration;

use termrock::runtime::stub::{Stub, deliver, key, mouse};
use termrock::{
    Action, ActionKey, App, Axis, BlurPolicy, Brand, Buffer, Button, Checkbox, ChipBar,
    ChipBarAction, ChipBarState, CodeEditor, Color, ColorLevel, Constraints, Cx, DefaultRow,
    DiffView, Empty, EmptyState, Field, FieldError, FieldKind, FieldMut, FieldRef, FieldSpec,
    FilterList, FilterListAction, FilterListState, Focusability, Form, FormAction, FormData,
    FormState, FrameRead, Grid, HelpOverlay, HintBar, Id, Input, Item, ItemKey, ItemRowLayout,
    KeyCode, KeyHint, KeyModifiers, List, ListAction, ListState, MenuBar, Meter, Modifier,
    MouseKind, NavList, NavListAction, NavListState, NavMode, NodeKind, Panel, Part, PartRef,
    Picker, PickerAction, PickerState, Position, ProgressBar, Props, PropsList, RadioGroup,
    RadioGroupAction, RadioGroupState, Rect, ReferenceState, ReferenceTarget, Response, Role,
    RowTotal, RowUi, Runtime, ScrollRegion, ScrollState, SecretPolicy, Select, SelectAction,
    SelectMode, SelectState, Size, Span, Spinner, SplitPane, StateFlags, Status, StatusBar,
    StepState, Steps, StepsAction, StepsState, StylePatch, Tabs, TabsAction, TabsState,
    TerminalView, TextAction, TextArea, TextAreaState, TextInput, TextInputState, TextViewport,
    Theme, Toggle, TooSmall, Tree, TreeAction, TreeBranchActivation, TreeBranchClick, TreeNode,
    TreeState, Ui, Validate, Variant, Wizard, width,
};
use termrock_test_support::Harness;

const SCREEN: Rect = Rect {
    x: 0,
    y: 0,
    width: 40,
    height: 8,
};

fn row_text(buf: &Buffer, y: u16, width: u16) -> String {
    let mut text = String::new();
    for x in 0..width {
        if let Some(cell) = buf.cell(Position::new(x, y)) {
            text.push_str(cell.symbol());
        }
    }
    text
}

fn cell_symbol(buf: &Buffer, x: u16, y: u16) -> String {
    buf.cell(Position::new(x, y))
        .map(|cell| cell.symbol().to_string())
        .unwrap_or_default()
}

fn cell_dim(buf: &Buffer, x: u16, y: u16) -> bool {
    buf.cell(Position::new(x, y))
        .map(|cell| cell.modifier.contains(Modifier::DIM))
        .unwrap_or(false)
}

/// Inert single-component paint on a fresh runtime.
fn paint(area: Rect, theme: Theme, draw: impl FnOnce(&mut Ui<'_>, Rect)) -> Buffer {
    let mut runtime = Runtime::new(Stub::default(), theme);
    let mut buffer = Buffer::empty(area);
    runtime
        .draw_scene(area, &mut buffer, |ui, area| draw(ui, area))
        .commit_presented();
    buffer
}

/// Inert paint with injected runtime-owned visual state (focus/hover/press).
fn paint_state(
    area: Rect,
    id: Id,
    state: ReferenceState,
    draw: impl FnOnce(&mut Ui<'_>, Rect),
) -> Buffer {
    paint(area, Theme::junie(), |ui, area| {
        ui.reference(Some(ReferenceTarget::new(id, state)), |ui| draw(ui, area));
    })
}

fn paint_plain(area: Rect, draw: impl FnOnce(&mut Ui<'_>, Rect)) -> Buffer {
    paint(area, Theme::junie(), draw)
}

/// Measure a component at an unconstrained width.
fn measure_size(draw: impl Fn(&Ui<'_>, Constraints) -> Size) -> Size {
    let mut runtime = Runtime::new(Stub::default(), Theme::junie());
    let mut buffer = Buffer::empty(SCREEN);
    let size = Cell::new(Size {
        min: (0, 0),
        preferred: (0, 0),
    });
    runtime
        .draw_scene(SCREEN, &mut buffer, |ui, _area| {
            size.set(draw(ui, Constraints::loose(u16::MAX, u16::MAX)));
        })
        .commit_presented();
    size.get()
}

// ---------------------------------------------------------------------------
// Ownership map
// ---------------------------------------------------------------------------

const OWNERSHIP_JSON: &str = include_str!("../../../component-ownership.json");

/// The full `component-ownership.json` map: every one of W01-W45 resolves
/// through its owner crate's facade module, and any unknown component id
/// fails the gate.
#[test]
fn w01_w45_resolve_through_owner_crate_modules() {
    let doc: serde_json::Value =
        serde_json::from_str(OWNERSHIP_JSON).expect("ownership map must parse");
    verify_ownership_map(&doc).expect("ownership map must match W01-W45 exactly");

    // Each facade-root component type is identical to the type exported by
    // its owner crate's module: the ownership map is executable, not a comment.
    fn same<A: 'static, B: 'static>() {
        assert_eq!(
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            "facade/owner type identity"
        );
    }
    same::<Brand<'static>, termrock::controls::Brand<'static>>();
    same::<Button<'static>, termrock::controls::Button<'static>>();
    same::<Checkbox<'static>, termrock::controls::Checkbox<'static>>();
    same::<Toggle<'static>, termrock::controls::Toggle<'static>>();
    same::<
        termrock::RadioGroup<termrock::Item<'static>>,
        termrock::controls::RadioGroup<termrock::Item<'static>>,
    >();
    same::<
        termrock::ChipBar<termrock::Item<'static>>,
        termrock::navigation::ChipBar<termrock::Item<'static>>,
    >();
    same::<
        termrock::Field<termrock::TextInput<'static>>,
        termrock::fields::Field<termrock::TextInput<'static>>,
    >();
    same::<termrock::TextInput<'static>, termrock::fields::TextInput<'static>>();
    same::<termrock::TextArea<'static>, termrock::fields::TextArea<'static>>();
    same::<
        termrock::Select<termrock::Item<'static>>,
        termrock::overlays::Select<termrock::Item<'static>>,
    >();
    same::<termrock::Form<'static>, termrock::forms::Form<'static>>();
    same::<
        termrock::List<termrock::Item<'static>>,
        termrock::navigation::List<termrock::Item<'static>>,
    >();
    same::<
        termrock::FilterList<termrock::Item<'static>>,
        termrock::navigation::FilterList<termrock::Item<'static>>,
    >();
    same::<
        termrock::NavList<termrock::Item<'static>>,
        termrock::navigation::NavList<termrock::Item<'static>>,
    >();
    same::<
        termrock::Tree<termrock::Item<'static>>,
        termrock::navigation::Tree<termrock::Item<'static>>,
    >();
    same::<
        termrock::Steps<termrock::Item<'static>>,
        termrock::navigation::Steps<termrock::Item<'static>>,
    >();
    same::<
        termrock::Tabs<termrock::Item<'static>>,
        termrock::navigation::Tabs<termrock::Item<'static>>,
    >();
    same::<
        termrock::Picker<termrock::Item<'static>>,
        termrock::overlays::Picker<termrock::Item<'static>>,
    >();
    same::<
        termrock::CommandPalette<termrock::Item<'static>>,
        termrock::overlays::CommandPalette<termrock::Item<'static>>,
    >();
    same::<termrock::PickerChain<'static>, termrock::overlays::PickerChain<'static>>();
    same::<
        termrock::Completion<termrock::Item<'static>>,
        termrock::overlays::Completion<termrock::Item<'static>>,
    >();
    same::<termrock::Dialog<'static>, termrock::overlays::Dialog<'static>>();
    same::<termrock::Menu<'static>, termrock::overlays::Menu<'static>>();
    same::<termrock::ContextMenu<'static>, termrock::overlays::ContextMenu<'static>>();
    same::<MenuBar<'static>, termrock::overlays::MenuBar<'static>>();
    same::<HelpOverlay<'static>, termrock::overlays::HelpOverlay<'static>>();
    same::<Wizard<'static>, termrock::forms::Wizard<'static>>();
    same::<Grid<'static>, termrock::grid::Grid<'static>>();
    same::<CodeEditor<'static>, termrock::editors::CodeEditor<'static>>();
    same::<DiffView<'static>, termrock::editors::DiffView<'static>>();
    same::<TextViewport<'static>, termrock::viewport::TextViewport<'static>>();
    same::<Panel<'static>, termrock::controls::Panel<'static>>();
    same::<SplitPane<'static>, termrock::controls::SplitPane<'static>>();
    same::<Props<'static>, termrock::controls::Props<'static>>();
    same::<PropsList<'static>, termrock::navigation::PropsList<'static>>();
    same::<Empty<'static>, termrock::controls::Empty<'static>>();
    same::<ProgressBar<'static>, termrock::feedback::ProgressBar<'static>>();
    same::<Spinner<'static>, termrock::feedback::Spinner<'static>>();
    same::<Meter<'static>, termrock::feedback::Meter<'static>>();
    same::<StatusBar<'static>, termrock::feedback::StatusBar<'static>>();
    same::<HintBar<'static>, termrock::feedback::HintBar<'static>>();
    same::<KeyHint<'static>, termrock::feedback::KeyHint<'static>>();
    same::<TooSmall<'static>, termrock::controls::TooSmall<'static>>();
    same::<TerminalView<'static>, termrock::terminal::TerminalView<'static>>();
    same::<ScrollRegion<'static>, termrock::viewport::ScrollRegion<'static>>();
}

/// Exact-set verification of the ownership map: every id in the document
/// must be a known W01-W45 component with its exact owner, and every
/// expected component must be present. Unknown ids fail.
fn verify_ownership_map(doc: &serde_json::Value) -> Result<(), String> {
    const EXPECTED: &[(&str, &str)] = &[
        ("W01", "termrock-controls"),
        ("W02", "termrock-controls"),
        ("W03", "termrock-controls"),
        ("W04", "termrock-controls"),
        ("W05", "termrock-controls"),
        ("W06", "termrock-navigation"),
        ("W07", "termrock-fields"),
        ("W08", "termrock-fields"),
        ("W09", "termrock-fields"),
        ("W10", "termrock-overlays"),
        ("W11", "termrock-forms"),
        ("W12", "termrock-navigation"),
        ("W13", "termrock-navigation"),
        ("W14", "termrock-navigation"),
        ("W15", "termrock-navigation"),
        ("W16", "termrock-navigation"),
        ("W17", "termrock-navigation"),
        ("W18", "termrock-overlays"),
        ("W19", "termrock-overlays"),
        ("W20", "termrock-overlays"),
        ("W21", "termrock-overlays"),
        ("W22", "termrock-overlays"),
        ("W23", "termrock-overlays"),
        ("W24", "termrock-overlays"),
        ("W25", "termrock-overlays"),
        ("W26", "termrock-overlays"),
        ("W27", "termrock-forms"),
        ("W28", "termrock-grid"),
        ("W29", "termrock-editors"),
        ("W30", "termrock-editors"),
        ("W31", "termrock-viewport"),
        ("W32", "termrock-controls"),
        ("W33", "termrock-controls"),
        ("W34", "termrock-controls"),
        ("W35", "termrock-navigation"),
        ("W36", "termrock-controls"),
        ("W37", "termrock-feedback"),
        ("W38", "termrock-feedback"),
        ("W39", "termrock-feedback"),
        ("W40", "termrock-feedback"),
        ("W41", "termrock-feedback"),
        ("W42", "termrock-feedback"),
        ("W43", "termrock-controls"),
        ("W44", "termrock-terminal"),
        ("W45", "termrock-viewport"),
    ];
    let components = doc["components"]
        .as_array()
        .ok_or_else(|| "ownership map lacks a components array".to_string())?;
    let mut owners = std::collections::BTreeMap::new();
    for component in components {
        let id = component["id"]
            .as_str()
            .ok_or_else(|| "a component lacks a string id".to_string())?;
        let owner = component["owner"]
            .as_str()
            .ok_or_else(|| format!("component {id} lacks a string owner"))?;
        owners.insert(id.to_string(), owner.to_string());
    }
    for id in owners.keys() {
        if !EXPECTED.iter().any(|(known, _)| known == id) {
            return Err(format!("unknown component id {id}"));
        }
    }
    let missing: Vec<&&str> = EXPECTED
        .iter()
        .map(|(id, _)| id)
        .filter(|id| !owners.contains_key(**id))
        .collect();
    if !missing.is_empty() {
        return Err(format!("missing components: {missing:?}"));
    }
    for (id, owner) in EXPECTED {
        let actual = owners.get(*id).map(String::as_str);
        if actual != Some(*owner) {
            return Err(format!(
                "ownership of {id}: expected {owner}, got {actual:?}"
            ));
        }
    }
    Ok(())
}

/// Unknown component ids fail the ownership gate (negative probe).
#[test]
fn ownership_gate_rejects_unknown_component_ids() {
    let lone = serde_json::json!({
        "components": [{"id": "W99", "name": "Bogus", "owner": "termrock-controls"}]
    });
    let err = verify_ownership_map(&lone).expect_err("a lone W99 must fail");
    assert!(err.contains("W99"), "unexpected error: {err}");

    let mut full: serde_json::Value =
        serde_json::from_str(OWNERSHIP_JSON).expect("ownership map must parse");
    full["components"]
        .as_array_mut()
        .expect("components array")
        .push(serde_json::json!({"id": "W00", "name": "Bogus", "owner": "termrock-controls"}));
    let err = verify_ownership_map(&full).expect_err("an injected W00 must fail");
    assert!(err.contains("W00"), "unexpected error: {err}");
}

// ---------------------------------------------------------------------------
// W01 Brand (termrock-controls)
// ---------------------------------------------------------------------------

const BRAND: Id = Id::root("control.states.brand");
const BRAND_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 24,
    height: 1,
};

/// W01-01: static label plus metadata at exact width and one column too narrow.
#[test]
fn w01_brand_exact_width_and_one_column_narrow() {
    const LABEL: &str = "Jackin";
    const META: &str = "v2";
    let natural = measure_size(|ui, c| Brand::new(BRAND, LABEL).tagline(META).measure(ui, c));
    let exact = natural.preferred.0;
    assert!(exact > width(LABEL), "tagline must extend the natural width");

    let full_area = Rect::new(0, 0, exact, 1);
    let full = paint_plain(full_area, |ui, area| {
        Brand::new(BRAND, LABEL).tagline(META).draw(ui, area);
    });
    let row = row_text(&full, 0, exact);
    assert!(
        row.contains(LABEL) && row.contains(META),
        "W01-01: exact width must show label plus metadata, got {row:?}"
    );

    let narrow_area = Rect::new(0, 0, exact.saturating_sub(1), 1);
    let narrow = paint_plain(narrow_area, |ui, area| {
        Brand::new(BRAND, LABEL).tagline(META).draw(ui, area);
    });
    let narrow_row = row_text(&narrow, 0, exact.saturating_sub(1));
    assert_ne!(
        narrow_row, row,
        "W01-01: one column too narrow must clip, got {narrow_row:?} vs {row:?}"
    );
    assert!(
        narrow_row.contains(LABEL),
        "W01-01: the clipped lockup keeps its label, got {narrow_row:?}"
    );
}

/// W01-02: interactive lockup — hover, press, release, canceled release and
/// one typed activation (the click yields exactly one `Activated`; the
/// lockup stays click-only with no Tab stop, per the reference baseline).
#[test]
fn w01_brand_clickable_hover_press_release_and_typed_activation() {
    #[derive(Default)]
    struct BrandApp {
        fired: Rc<Cell<u32>>,
        clickable: bool,
    }

    impl App for BrandApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let fired = Rc::clone(&self.fired);
            Brand::new(BRAND, "Jackin")
                .clickable(self.clickable)
                .update(cx)
                .on_activated(|| fired.set(fired.get() + 1))
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            Brand::new(BRAND, "Jackin")
                .clickable(self.clickable)
                .draw(ui, BRAND_AREA);
        }
    }

    // Static lockup: hover paints no accent and clicks never fire.
    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        BrandApp {
            fired: Rc::clone(&fired),
            clickable: false,
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    let plain = app.buffer().clone();
    let _ = app.mouse(MouseKind::Move, 2, 0);
    assert_eq!(
        app.buffer(),
        &plain,
        "W01-02: a static lockup must not lift under the pointer"
    );
    let _ = app.click(2, 0);
    assert_eq!(fired.get(), 0, "W01-02: a static lockup never fires");

    // Clickable lockup: hover lifts, press/release fires once, a canceled
    // release (down inside, up outside) fires nothing, Enter fires once.
    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        BrandApp {
            fired: Rc::clone(&fired),
            clickable: true,
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    let plain = app.buffer().clone();
    let _ = app.mouse(MouseKind::Move, 2, 0);
    let hovered = app.buffer().clone();
    assert_ne!(
        hovered, plain,
        "W01-02: a clickable lockup must lift under the pointer"
    );
    let _ = app.mouse(MouseKind::Down, 2, 0);
    let pressed = app.buffer().clone();
    // Press-vs-hover distinctness is the separate
    // `w01_brand_press_paints_distinct_from_hover` parity record.
    let _ = app.mouse(MouseKind::Up, 2, 0);
    assert_eq!(fired.get(), 1, "W01-02: release inside must fire once");

    let _ = app.mouse(MouseKind::Down, 2, 0);
    assert_eq!(
        app.buffer(),
        &pressed,
        "W01-02: press must repaint pressed"
    );
    let _ = app.mouse(MouseKind::Up, 30, 6);
    assert_eq!(
        fired.get(),
        1,
        "W01-02: a release outside the lockup must cancel, not fire"
    );

    // Click-only mode (the reference baseline): no Tab stop, and Enter can
    // never produce the one typed `Activated` — only the completed click did.
    assert!(
        !app.tab_to(BRAND),
        "W01-02: a click-only lockup must take no Tab stop"
    );
    let _ = app.key(KeyCode::Enter);
    assert_eq!(
        fired.get(),
        1,
        "W01-02: exactly one typed activation comes from the click alone"
    );
}

/// W01-02 parity record: the reference oracle (`Lockup::render_clickable` in
/// `src/widgets/brand.rs` at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`)
/// paints a pressed lockup with the `accent_pressed` background, distinct
/// from the `accent_hover` lift. The candidate `BRAND`/`LABEL` recipe
/// (`termrock-theme/src/builtin/mod.rs`) has a `HOVERED` rule but no
/// `PRESSED` rule, so press repaints exactly the hover frame (Q65-S4/G6
/// deleted the generic mono bracket fallback, so at `Mono` press now
/// matches hover too — the recipe gap this record tracks is unchanged).
#[test]
#[ignore = "PARITY W01-02: pressed lockup repaints the hover frame, reference paints accent_pressed (BRAND recipe has no PRESSED rule)"]
fn w01_brand_press_paints_distinct_from_hover() {
    let area = Rect::new(0, 0, 12, 1);
    let hovered = paint_state(area, BRAND, ReferenceState::HOVERED, |ui, area| {
        Brand::new(BRAND, "Jackin").clickable(true).draw(ui, area);
    });
    let pressed = paint_state(
        area,
        BRAND,
        ReferenceState::HOVERED.union(ReferenceState::PRESSED),
        |ui, area| {
            Brand::new(BRAND, "Jackin").clickable(true).draw(ui, area);
        },
    );
    assert_ne!(
        hovered, pressed,
        "W01-02: press must paint below the hover lift (accent_pressed)"
    );
}

/// W01-03: two lockups with independent IDs; pointer targeting does not cross.
#[test]
fn w01_brand_two_lockups_keep_independent_targets() {
    const OTHER: Id = Id::root("control.states.brand.other");
    struct TwoBrands {
        fired_main: Rc<Cell<u32>>,
        fired_other: Rc<Cell<u32>>,
    }
    impl App for TwoBrands {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let main = Rc::clone(&self.fired_main);
            let other = Rc::clone(&self.fired_other);
            let mut r = Brand::new(BRAND, "One")
                .clickable(true)
                .update(cx)
                .on_activated(|| main.set(main.get() + 1));
            r |= Brand::new(OTHER, "Two")
                .clickable(true)
                .update(cx)
                .on_activated(|| other.set(other.get() + 1));
            r
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            Brand::new(BRAND, "One")
                .clickable(true)
                .draw(ui, Rect::new(0, 0, 10, 1));
            Brand::new(OTHER, "Two")
                .clickable(true)
                .draw(ui, Rect::new(0, 2, 10, 1));
        }
    }
    let main = Rc::new(Cell::new(0));
    let other = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        TwoBrands {
            fired_main: Rc::clone(&main),
            fired_other: Rc::clone(&other),
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    let _ = app.mouse(MouseKind::Move, 2, 0);
    assert_eq!(app.hover(), Some(BRAND));
    let _ = app.mouse(MouseKind::Move, 2, 2);
    assert_eq!(
        app.hover(),
        Some(OTHER),
        "W01-03: hover must target the lockup under the pointer"
    );
    let _ = app.click(2, 2);
    assert_eq!((main.get(), other.get()), (0, 1));
    let _ = app.click(2, 0);
    assert_eq!(
        (main.get(), other.get()),
        (1, 1),
        "W01-03: activation must not cross between lockups"
    );
}

/// W01-04: a static lockup is absent from Tab traversal.
#[test]
fn w01_brand_static_lockup_absent_from_tab_traversal() {
    struct StaticBrand;
    impl App for StaticBrand {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            Brand::new(BRAND, "Jackin").update(cx).erase()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_control(
                Id::root("control.states.brand.sentinel"),
                Rect::new(0, 2, 8, 1),
                Focusability::Focusable,
            );
            Brand::new(BRAND, "Jackin").draw(ui, BRAND_AREA);
        }
    }
    let mut app = Harness::new(StaticBrand, Theme::junie(), SCREEN.width, SCREEN.height);
    assert!(
        !app.tab_to(BRAND),
        "W01-04: a static lockup must never take Tab focus"
    );
}

// ---------------------------------------------------------------------------
// W02 Button (termrock-controls)
// ---------------------------------------------------------------------------

const BUTTON: Id = Id::root("control.states.button");
const BUTTON_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 14,
    height: 1,
};

const BUTTON_VARIANTS: [Variant; 5] = [
    Variant::PRIMARY,
    Variant::SECONDARY,
    Variant::SUBTLE,
    Variant::DANGER,
    Variant::TOGGLE,
];

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

/// W02-01: all variants at normal, focus, hover, focus+hover and disabled.
///
/// Mirrors the reference `button_focus_remains_unambiguous_without_colour`:
/// the focus gutter is `▎` when focused and enabled, a space otherwise, at
/// every color level. The disabled-suppresses-focus case is the separate
/// `w02_button_disabled_suppresses_focus_gutter` parity record.
#[test]
fn w02_button_variants_all_states_and_focus_gutter() {
    for level in LEVELS {
        let theme = Theme::junie().downgrade(level);
        for variant in BUTTON_VARIANTS {
            for (focused, disabled, expected) in [(true, false, "▎"), (false, false, " ")] {
                let state = if focused {
                    ReferenceState::FOCUSED
                } else {
                    ReferenceState::default()
                };
                let buf = paint(BUTTON_AREA, theme.clone(), |ui, area| {
                    ui.reference(Some(ReferenceTarget::new(BUTTON, state)), |ui| {
                        Button::new(BUTTON, "Action")
                            .variant(variant)
                            .disabled(disabled)
                            .draw(ui, area);
                    });
                });
                assert_eq!(
                    cell_symbol(&buf, 0, 0),
                    expected,
                    "W02-01 {level:?} {variant:?}: focused={focused}, disabled={disabled}"
                );
                assert_eq!(
                    cell_dim(&buf, 1, 0),
                    disabled && level == ColorLevel::Mono,
                    "W02-01 {level:?} {variant:?}: mono DIM marks disabled only"
                );
            }

            // Hover, focus+hover and pressed each paint distinctly while enabled.
            let normal = paint_plain(BUTTON_AREA, |ui, area| {
                Button::new(BUTTON, "Action").variant(variant).draw(ui, area);
            });
            let hover = paint_state(BUTTON_AREA, BUTTON, ReferenceState::HOVERED, |ui, area| {
                Button::new(BUTTON, "Action").variant(variant).draw(ui, area);
            });
            let focus_hover = paint_state(
                BUTTON_AREA,
                BUTTON,
                ReferenceState::FOCUSED.union(ReferenceState::HOVERED),
                |ui, area| {
                    Button::new(BUTTON, "Action").variant(variant).draw(ui, area);
                },
            );
            let pressed = paint_state(BUTTON_AREA, BUTTON, ReferenceState::PRESSED, |ui, area| {
                Button::new(BUTTON, "Action").variant(variant).draw(ui, area);
            });
            let disabled = paint_plain(BUTTON_AREA, |ui, area| {
                Button::new(BUTTON, "Action")
                    .variant(variant)
                    .disabled(true)
                    .draw(ui, area);
            });
            assert_ne!(normal, hover, "W02-01 {variant:?}: hover must differ");
            assert_ne!(
                normal, focus_hover,
                "W02-01 {variant:?}: focus+hover must differ"
            );
            assert_ne!(hover, focus_hover, "W02-01 {variant:?}: focus must add to hover");
            assert_ne!(normal, pressed, "W02-01 {variant:?}: pressed must differ");
            assert_ne!(
                normal, disabled,
                "W02-01 {variant:?}: disabled must differ"
            );
            // Precedence: disabled suppresses hover and press feedback.
            let disabled_hover = paint_state(
                BUTTON_AREA,
                BUTTON,
                ReferenceState::HOVERED,
                |ui, area| {
                    Button::new(BUTTON, "Action")
                        .variant(variant)
                        .disabled(true)
                        .draw(ui, area);
                },
            );
            assert_eq!(
                disabled, disabled_hover,
                "W02-01 {variant:?}: disabled must win over hover"
            );
        }
    }
}

/// W02-01 parity record: the reference oracle (`src/widgets/button.rs` at
/// `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, `Theme::gutter_symbol`) paints
/// the gutter as `▎` only when `focused && !disabled`, and the Stage A
/// `button_focus_remains_unambiguous_without_colour` pins `(true, true, " ")`.
/// The candidate theme recipe (`termrock-theme/src/builtin/mod.rs`) sets the
/// `GUTTER` glyph on `FOCUSED` alone, so a disabled-but-focused button keeps
/// the `▎`. The combo is unreachable in the live runtime (disabled controls
/// never take focus); it is expressible only through inert reference forcing,
/// exactly as the reference test constructs it.
#[test]
#[ignore = "PARITY W02-01: disabled+focused paints ▎, reference demands space (theme GUTTER recipe keys on FOCUSED alone)"]
fn w02_button_disabled_suppresses_focus_gutter() {
    for level in LEVELS {
        let theme = Theme::junie().downgrade(level);
        for variant in BUTTON_VARIANTS {
            let buf = paint(BUTTON_AREA, theme.clone(), |ui, area| {
                ui.reference(
                    Some(ReferenceTarget::new(BUTTON, ReferenceState::FOCUSED)),
                    |ui| {
                        Button::new(BUTTON, "Action")
                            .variant(variant)
                            .disabled(true)
                            .draw(ui, area);
                    },
                );
            });
            assert_eq!(
                cell_symbol(&buf, 0, 0),
                " ",
                "W02-01 {level:?} {variant:?}: disabled must suppress the focus gutter"
            );
        }
    }
}

/// Shared button rig: fire counter plus visibility/disabled switches.
struct ButtonApp {
    fired: Rc<Cell<u32>>,
    visible: Rc<Cell<bool>>,
    disabled: bool,
    status: Status,
}

impl ButtonApp {
    fn button(&self) -> Button<'static> {
        Button::new(BUTTON, "Action")
            .disabled(self.disabled)
            .status(self.status)
    }
}

impl App for ButtonApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !self.visible.get() {
            return Response::ignored();
        }
        let fired = Rc::clone(&self.fired);
        self.button()
            .update(cx)
            .on_activated(|| fired.set(fired.get() + 1))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.visible.get() {
            self.button().draw(ui, BUTTON_AREA);
        }
    }
}

impl ButtonApp {
    fn harness(
        fired: &Rc<Cell<u32>>,
        visible: &Rc<Cell<bool>>,
        disabled: bool,
        status: Status,
    ) -> Harness<ButtonApp> {
        Harness::new(
            ButtonApp {
                fired: Rc::clone(fired),
                visible: Rc::clone(visible),
                disabled,
                status,
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        )
    }
}

/// W02-02: pointer down inside; release outside; repeat with the target
/// removed or disabled.
#[test]
fn w02_button_release_outside_removed_or_disabled_never_fires() {
    // Positive control: down inside + up inside fires exactly once.
    let fired = Rc::new(Cell::new(0));
    let visible = Rc::new(Cell::new(true));
    let mut app = ButtonApp::harness(&fired, &visible, false, Status::Ready);
    let _ = app.mouse(MouseKind::Down, 2, 0);
    let _ = app.mouse(MouseKind::Up, 2, 0);
    assert_eq!(fired.get(), 1, "W02-02: the rig must fire on release inside");

    // Release outside cancels.
    let fired = Rc::new(Cell::new(0));
    let visible = Rc::new(Cell::new(true));
    let mut app = ButtonApp::harness(&fired, &visible, false, Status::Ready);
    let _ = app.mouse(MouseKind::Down, 2, 0);
    let _ = app.mouse(MouseKind::Up, 30, 6);
    assert_eq!(
        fired.get(),
        0,
        "W02-02: down inside + release outside must not fire"
    );

    // Target removed between press and release: no fire.
    let fired = Rc::new(Cell::new(0));
    let visible = Rc::new(Cell::new(true));
    let mut app = ButtonApp::harness(&fired, &visible, false, Status::Ready);
    let _ = app.mouse(MouseKind::Down, 2, 0);
    visible.set(false);
    let _ = app.mouse(MouseKind::Up, 2, 0);
    assert_eq!(
        fired.get(),
        0,
        "W02-02: a target removed before release must not fire"
    );

    // Disabled target: press, release and typed activation all ignored.
    let fired = Rc::new(Cell::new(0));
    let visible = Rc::new(Cell::new(true));
    let mut app = ButtonApp::harness(&fired, &visible, true, Status::Ready);
    let _ = app.mouse(MouseKind::Down, 2, 0);
    let _ = app.mouse(MouseKind::Up, 2, 0);
    let _ = app.click(2, 0);
    assert!(
        !app.tab_to(BUTTON),
        "W02-02: a disabled button must be unreachable by Tab"
    );
    let _ = app.key(KeyCode::Enter);
    assert_eq!(
        fired.get(),
        0,
        "W02-02: a disabled button must ignore every activation"
    );
}

/// W02-03: successful keyboard and pointer activation at 0, 139, 140 and
/// 141 ms after feedback start — feedback never swallows a new activation.
///
/// In the candidate the pointer path starts the 140 ms press flash
/// (`Runtime::pointer_up`); the reference property is that both origins keep
/// firing at every offset after that start.
#[test]
fn w02_button_reactivation_at_feedback_boundaries() {
    use core::time::Duration;

    let fired = Rc::new(Cell::new(0));
    let visible = Rc::new(Cell::new(true));
    let mut app = ButtonApp::harness(&fired, &visible, false, Status::Ready);
    assert!(app.tab_to(BUTTON));
    let _ = app.click(2, 0);
    assert_eq!(fired.get(), 1);
    assert_eq!(
        app.activation_feedback().map(|f| f.owner),
        Some(BUTTON),
        "W02-03: the opening activation must start feedback"
    );
    // Offsets are measured from that feedback start; Enter never restarts
    // the flash, so the keyboard half observes the original record aging.
    for (step_ms, offset) in [(0, 0), (139, 139), (1, 140), (1, 141)] {
        let _ = app.advance(Duration::from_millis(step_ms));
        let before = fired.get();
        let _ = app.key(KeyCode::Enter);
        assert_eq!(
            fired.get(),
            before + 1,
            "W02-03: keyboard activation at {offset} ms must fire"
        );
        let _ = app.click(2, 0);
        assert_eq!(
            fired.get(),
            before + 2,
            "W02-03: pointer activation at {offset} ms must fire"
        );
    }
}

/// W02-04: checked but unfocused; unchecked but focused; busy activation
/// rejection.
#[test]
fn w02_button_checked_focus_combos_and_busy_rejection() {
    // Checked-but-unfocused and unchecked-but-focused paint distinctly, and
    // focus still adds to a checked button.
    let checked_plain = paint_plain(BUTTON_AREA, |ui, area| {
        Button::new(BUTTON, "Sync")
            .variant(Variant::TOGGLE)
            .checked(true)
            .draw(ui, area);
    });
    let unchecked_focused = paint_state(
        BUTTON_AREA,
        BUTTON,
        ReferenceState::FOCUSED,
        |ui, area| {
            Button::new(BUTTON, "Sync")
                .variant(Variant::TOGGLE)
                .checked(false)
                .draw(ui, area);
        },
    );
    let checked_focused = paint_state(
        BUTTON_AREA,
        BUTTON,
        ReferenceState::FOCUSED,
        |ui, area| {
            Button::new(BUTTON, "Sync")
                .variant(Variant::TOGGLE)
                .checked(true)
                .draw(ui, area);
        },
    );
    assert_ne!(
        checked_plain, unchecked_focused,
        "W02-04: checked/unfocused must differ from unchecked/focused"
    );
    assert_ne!(
        checked_plain, checked_focused,
        "W02-04: focus must add to a checked button"
    );
    assert_ne!(
        unchecked_focused, checked_focused,
        "W02-04: checked must add to a focused button"
    );

    // Busy rejects keyboard and pointer activation alike.
    for status in [Status::Busy, Status::Loading] {
        let fired = Rc::new(Cell::new(0));
        let visible = Rc::new(Cell::new(true));
        let mut app = ButtonApp::harness(&fired, &visible, false, status);
        assert!(
            app.tab_to(BUTTON),
            "W02-04 {status:?}: a busy button stays reachable"
        );
        let _ = app.key(KeyCode::Enter);
        let _ = app.click(2, 0);
        assert_eq!(
            fired.get(),
            0,
            "W02-04 {status:?}: busy activation must be rejected"
        );
    }
}

/// W02-05: labels with combining characters, CJK and an empty label.
#[test]
fn w02_button_combining_cjk_and_empty_labels() {
    // Natural width is gutter + label + pad; combining marks add no columns.
    for (label, cells) in [("e\u{301}", 1), ("界", 2), ("", 0), ("Action", 6)] {
        assert_eq!(width(label), cells, "W02-05: width({label:?})");
        let used = Cell::new(Rect::default());
        let buf = paint_plain(BUTTON_AREA, |ui, area| {
            used.set(Button::new(BUTTON, label).draw(ui, area));
        });
        let used = used.get();
        assert_eq!(
            used.width,
            cells + 2,
            "W02-05: {label:?} must occupy gutter + label + pad"
        );
        assert!(
            used.width <= BUTTON_AREA.width && used.height == 1,
            "W02-05: {label:?} must stay inside its area"
        );
        if cells > 0 {
            assert!(
                row_text(&buf, 0, used.width).contains(label.trim()),
                "W02-05: {label:?} must paint its glyphs"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// W03 Checkbox (termrock-controls)
// ---------------------------------------------------------------------------

const CHECKBOX: Id = Id::root("control.states.checkbox");
const CHECKBOX_LABEL: &str = "Choice";
/// Gutter + 3-cell marker + gap + label.
const CHECKBOX_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 11,
    height: 1,
};

fn paint_checkbox(checked: bool, state: ReferenceState, disabled: bool) -> Buffer {
    paint_state(CHECKBOX_AREA, CHECKBOX, state, |ui, area| {
        Checkbox::new(CHECKBOX, CHECKBOX_LABEL)
            .checked(checked)
            .disabled(disabled)
            .draw(ui, area);
    })
}

/// W03-01: checked and unchecked with independent focus and hover.
///
/// Mirrors the reference `compact_choice_marks_keep_checked_state_without_colour`
/// for the marker axis: at widths 2-3 the marker cell keeps `✓`/`□`, at
/// width 4+ the box opens with `[`.
#[test]
fn w03_checkbox_checked_focus_hover_matrix() {
    let plain = ReferenceState::default();
    for checked in [false, true] {
        let normal = paint_checkbox(checked, plain, false);
        let focused = paint_checkbox(checked, ReferenceState::FOCUSED, false);
        let hovered = paint_checkbox(checked, ReferenceState::HOVERED, false);
        let both = paint_checkbox(
            checked,
            ReferenceState::FOCUSED.union(ReferenceState::HOVERED),
            false,
        );
        assert_ne!(normal, focused, "W03-01 checked={checked}: focus must add");
        assert_ne!(normal, hovered, "W03-01 checked={checked}: hover must add");
        if !checked {
            assert_ne!(
                focused, both,
                "W03-01 checked={checked}: hover must add to focus"
            );
        }
        // Checked focus+hover is the separate
        // `w03_checkbox_checked_focus_hover_independent` parity record.
        assert_ne!(
            hovered, both,
            "W03-01 checked={checked}: focus must add to hover"
        );
        assert_ne!(
            normal,
            paint_checkbox(!checked, plain, false),
            "W03-01: checked must differ from unchecked at rest"
        );
        assert_ne!(
            focused,
            paint_checkbox(!checked, ReferenceState::FOCUSED, false),
            "W03-01: checked must differ from unchecked under focus"
        );
        assert_ne!(
            both,
            paint_checkbox(
                !checked,
                ReferenceState::FOCUSED.union(ReferenceState::HOVERED),
                false
            ),
            "W03-01: checked must differ from unchecked under focus+hover"
        );
    }

    // Compact narrow markers are the separate `w03_checkbox_compact_markers`
    // parity record.
}

/// W03-01 parity record: the reference (`Checkbox::render` in
/// `src/widgets/choice.rs` at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`,
/// pinned by `compact_choice_marks_keep_checked_state_without_colour`) paints
/// compact `✓`/`□` state marks at widths 2-3. The candidate `Checkbox` has no
/// compact branch and clips the 3-cell `[✓]`/`[ ]` box instead, so a 2-wide
/// row shows `[` for both values (state unreadable). The candidate's own
/// checkbox doc promises compact `✓`/`□` on narrow rows.
#[test]
#[ignore = "PARITY W03-01: narrow checkbox clips [✓]/[ ] instead of compact ✓/□ marks (no compact branch)"]
fn w03_checkbox_compact_markers() {
    for width in [2, 3, 4] {
        for checked in [false, true] {
            let area = Rect::new(0, 0, width, 1);
            let buf = paint(
                area,
                Theme::junie().downgrade(ColorLevel::Mono),
                |ui, area| {
                    Checkbox::new(CHECKBOX, CHECKBOX_LABEL)
                        .checked(checked)
                        .draw(ui, area);
                },
            );
            let expected = if width >= 4 {
                "["
            } else if checked {
                "✓"
            } else {
                "□"
            };
            assert_eq!(
                cell_symbol(&buf, 1, 0),
                expected,
                "W03-01: width={width} checked={checked} marker"
            );
        }
    }
}

/// W03-01 parity record: the reference (`Theme::row` in `src/theme.rs` at
/// `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) lifts the row background on
/// hover after — and therefore over — the selection tint, so focus+hover
/// always differs from focus. The candidate `CONTAINER` recipe
/// (`termrock-theme/src/builtin/mod.rs`) resolves the more specific
/// `SELECTED|FOCUSED` rule (accent tint) over `HOVERED` (raised surface), so
/// a checked+focused checkbox repaints exactly the same frame with and
/// without hover. The candidate's own checkbox doc lists focus+hover as a
/// distinct axis state.
#[test]
#[ignore = "PARITY W03-01: checked+focus+hover repaints checked+focus (SELECTED|FOCUSED swallows the hover lift), reference lifts over the tint"]
fn w03_checkbox_checked_focus_hover_independent() {
    let focused = paint_checkbox(true, ReferenceState::FOCUSED, false);
    let both = paint_checkbox(
        true,
        ReferenceState::FOCUSED.union(ReferenceState::HOVERED),
        false,
    );
    assert_ne!(
        focused, both,
        "W03-01: hover must add to a checked+focused checkbox"
    );
}

/// Shared checkbox rig: controlled value plus fire counter.
struct CheckboxApp {
    value: Rc<Cell<bool>>,
    fired: Rc<Cell<u32>>,
    disabled: bool,
    /// When set, the caller rejects every change (W03-04).
    reject: bool,
}

impl App for CheckboxApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut v = self.value.get();
        let before = v;
        let fired = Rc::clone(&self.fired);
        let r = Checkbox::new(CHECKBOX, CHECKBOX_LABEL)
            .checked(v)
            .disabled(self.disabled)
            .update(cx, &mut v)
            .on_activated(|| fired.set(fired.get() + 1));
        self.value.set(if self.reject { before } else { v });
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Checkbox::new(CHECKBOX, CHECKBOX_LABEL)
            .checked(self.value.get())
            .disabled(self.disabled)
            .draw(ui, CHECKBOX_AREA);
    }
}

impl CheckboxApp {
    fn harness(value: &Rc<Cell<bool>>, fired: &Rc<Cell<u32>>, disabled: bool) -> Harness<CheckboxApp> {
        Self::harness_reject(value, fired, disabled, false)
    }

    fn harness_reject(
        value: &Rc<Cell<bool>>,
        fired: &Rc<Cell<u32>>,
        disabled: bool,
        reject: bool,
    ) -> Harness<CheckboxApp> {
        Harness::new(
            CheckboxApp {
                value: Rc::clone(value),
                fired: Rc::clone(fired),
                disabled,
                reject,
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        )
    }
}

/// W03-02: disabled checked and disabled unchecked ignore all activation.
///
/// Mirrors the reference `disabled_choices_reject_keyboard_and_mouse_changes`.
#[test]
fn w03_checkbox_disabled_ignores_all_activation() {
    for checked in [false, true] {
        let value = Rc::new(Cell::new(checked));
        let fired = Rc::new(Cell::new(0));
        let mut app = CheckboxApp::harness(&value, &fired, true);
        assert!(
            !app.tab_to(CHECKBOX),
            "W03-02: a disabled checkbox must be unreachable"
        );
        let _ = app.key(KeyCode::Enter);
        let _ = app.key(KeyCode::Char(' '));
        let _ = app.click(2, 0);
        let _ = app.click(6, 0);
        assert_eq!(fired.get(), 0, "W03-02 checked={checked}: never fires");
        assert_eq!(
            value.get(),
            checked,
            "W03-02 checked={checked}: value must not move"
        );
    }
}

/// W03-03: click marker, label, last valid cell and immediately outside.
#[test]
fn w03_checkbox_marker_label_edge_and_outside_clicks() {
    let value = Rc::new(Cell::new(false));
    let fired = Rc::new(Cell::new(0));
    let mut app = CheckboxApp::harness(&value, &fired, false);
    // Marker cell.
    let _ = app.click(2, 0);
    assert_eq!((value.get(), fired.get()), (true, 1), "W03-03: marker clicks");
    // Label cell.
    let _ = app.click(6, 0);
    assert_eq!((value.get(), fired.get()), (false, 2), "W03-03: label clicks");
    // Last valid cell of the 11-wide row.
    let _ = app.click(10, 0);
    assert_eq!(
        (value.get(), fired.get()),
        (true, 3),
        "W03-03: the last valid cell clicks"
    );
    // Immediately outside: nothing.
    let _ = app.click(11, 0);
    assert_eq!(
        (value.get(), fired.get()),
        (true, 3),
        "W03-03: immediately outside must not click"
    );
}

/// W03-04: the caller rejects the change; the checkbox must not drift into
/// an uncontrolled value.
#[test]
fn w03_checkbox_rejected_change_does_not_drift() {
    let value = Rc::new(Cell::new(false));
    let fired = Rc::new(Cell::new(0));
    let mut app = CheckboxApp::harness_reject(&value, &fired, false, true);
    assert!(app.tab_to(CHECKBOX));
    let _ = app.key(KeyCode::Enter);
    assert_eq!(
        fired.get(),
        1,
        "W03-04: the component still proposes the change"
    );
    assert!(
        !value.get(),
        "W03-04: a rejected change must not drift the value"
    );
    let _ = app.click(2, 0);
    assert_eq!(
        fired.get(),
        2,
        "W03-04: pointer changes are proposed too"
    );
    assert!(
        !value.get(),
        "W03-04: a rejected pointer change must not drift either"
    );
    // Symbol-level: the live runtime and the inert reference resolve
    // different absolute surfaces, but the controlled marker glyph is exact.
    let expected = paint_checkbox(false, ReferenceState::FOCUSED, false);
    assert_eq!(
        row_text(app.buffer(), 0, CHECKBOX_AREA.width),
        row_text(&expected, 0, CHECKBOX_AREA.width),
        "W03-04: paint must follow the controlled value, not the proposal"
    );
}

// ---------------------------------------------------------------------------
// W04 Toggle (termrock-controls)
// ---------------------------------------------------------------------------

const TOGGLE: Id = Id::root("control.states.toggle");
const TOGGLE_LABEL: &str = "Choice";
/// Gutter + 3-cell marker + gap + label + gap + on/off trailer.
const TOGGLE_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 15,
    height: 1,
};

fn paint_toggle(on: bool, state: ReferenceState, disabled: bool) -> Buffer {
    paint_state(TOGGLE_AREA, TOGGLE, state, |ui, area| {
        Toggle::new(TOGGLE, TOGGLE_LABEL)
            .on(on)
            .disabled(disabled)
            .draw(ui, area);
    })
}

/// W04-01: on/off crossed with normal, focused, hovered and disabled.
#[test]
fn w04_toggle_on_off_state_matrix() {
    let plain = ReferenceState::default();
    for on in [false, true] {
        let normal = paint_toggle(on, plain, false);
        let focused = paint_toggle(on, ReferenceState::FOCUSED, false);
        let hovered = paint_toggle(on, ReferenceState::HOVERED, false);
        let disabled = paint_toggle(on, plain, true);
        assert_ne!(normal, focused, "W04-01 on={on}: focus must add");
        assert_ne!(normal, hovered, "W04-01 on={on}: hover must add");
        assert_ne!(normal, disabled, "W04-01 on={on}: disabled must differ");
        assert_ne!(
            normal,
            paint_toggle(!on, plain, false),
            "W04-01: on must differ from off"
        );
        assert_ne!(
            focused,
            paint_toggle(!on, ReferenceState::FOCUSED, false),
            "W04-01: on must differ from off under focus"
        );
        assert_ne!(
            disabled,
            paint_toggle(!on, plain, true),
            "W04-01: on must differ from off while disabled"
        );
    }

    // Compact narrow markers are the separate
    // `w04_toggle_compact_markers` parity record.
}

/// W04-01 parity record: the reference (`Toggle::render` in
/// `src/widgets/choice.rs` at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`,
/// pinned by `compact_choice_marks_keep_checked_state_without_colour`) paints
/// compact `●`/`○` state dots at widths 2-3. The candidate `Toggle` paints a
/// track-plus-knob switch with no compact branch, so a 2-wide row shows `●`
/// for both on and off (state unreadable) and a 3-wide row shows knob-position
/// fragments (`●─`/`─●`) instead of the state dots.
#[test]
#[ignore = "PARITY W04-01: narrow toggle clips the switch instead of compact ●/○ dots (no compact branch)"]
fn w04_toggle_compact_markers() {
    for width in [2, 3] {
        for on in [false, true] {
            let area = Rect::new(0, 0, width, 1);
            let buf = paint(
                area,
                Theme::junie().downgrade(ColorLevel::Mono),
                |ui, area| {
                    Toggle::new(TOGGLE, TOGGLE_LABEL).on(on).draw(ui, area);
                },
            );
            assert_eq!(
                cell_symbol(&buf, 1, 0),
                if on { "●" } else { "○" },
                "W04-01: width={width} on={on} marker"
            );
        }
    }
}

/// Shared toggle rig: controlled value plus fire counter.
struct ToggleApp {
    value: Rc<Cell<bool>>,
    fired: Rc<Cell<u32>>,
}

impl App for ToggleApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut v = self.value.get();
        let fired = Rc::clone(&self.fired);
        let r = Toggle::new(TOGGLE, TOGGLE_LABEL)
            .on(v)
            .update(cx, &mut v)
            .on_activated(|| fired.set(fired.get() + 1));
        self.value.set(v);
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Toggle::new(TOGGLE, TOGGLE_LABEL)
            .on(self.value.get())
            .draw(ui, TOGGLE_AREA);
    }
}

/// W04-02: Enter/Space and mouse produce the same next controlled value.
#[test]
fn w04_toggle_origins_agree_on_next_value() {
    for origin in ["enter", "space", "mouse"] {
        let value = Rc::new(Cell::new(false));
        let fired = Rc::new(Cell::new(0));
        let mut app = Harness::new(
            ToggleApp {
                value: Rc::clone(&value),
                fired: Rc::clone(&fired),
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        match origin {
            "enter" => {
                assert!(app.tab_to(TOGGLE));
                let _ = app.key(KeyCode::Enter);
            }
            "space" => {
                assert!(app.tab_to(TOGGLE));
                let _ = app.key(KeyCode::Char(' '));
            }
            _ => {
                let _ = app.click(2, 0);
            }
        }
        assert_eq!(
            (value.get(), fired.get()),
            (true, 1),
            "W04-02: {origin} must flip false to true exactly once"
        );
    }
}

/// W04-03: adjacent toggles do not overlap hit regions.
#[test]
fn w04_toggle_adjacent_hit_regions_stay_separate() {
    const OTHER: Id = Id::root("control.states.toggle.other");
    struct TwoToggles {
        first: Rc<Cell<u32>>,
        second: Rc<Cell<u32>>,
    }
    impl App for TwoToggles {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let first = Rc::clone(&self.first);
            let second = Rc::clone(&self.second);
            let mut v = false;
            let mut r = Toggle::new(TOGGLE, "First")
                .on(false)
                .update(cx, &mut v)
                .on_activated(|| first.set(first.get() + 1));
            r |= Toggle::new(OTHER, "Second")
                .on(false)
                .update(cx, &mut v)
                .on_activated(|| second.set(second.get() + 1));
            r
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            Toggle::new(TOGGLE, "First")
                .on(false)
                .draw(ui, Rect::new(0, 0, 14, 1));
            Toggle::new(OTHER, "Second")
                .on(false)
                .draw(ui, Rect::new(0, 1, 15, 1));
        }
    }
    let first = Rc::new(Cell::new(0));
    let second = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        TwoToggles {
            first: Rc::clone(&first),
            second: Rc::clone(&second),
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    assert_eq!(app.area_of(TOGGLE), Some(Rect::new(0, 0, 14, 1)));
    assert_eq!(app.area_of(OTHER), Some(Rect::new(0, 1, 15, 1)));
    let _ = app.click(5, 0);
    assert_eq!((first.get(), second.get()), (1, 0));
    let _ = app.click(5, 1);
    assert_eq!(
        (first.get(), second.get()),
        (1, 1),
        "W04-03: each click must hit exactly one toggle"
    );
}

/// Paint into a sentinel-filled oversized buffer; the component draws at `at`.
fn paint_contained(at: Rect, draw: impl FnOnce(&mut Ui<'_>, Rect)) -> Buffer {
    let screen = Rect::new(0, 0, 20, 3);
    let mut runtime = Runtime::new(Stub::default(), Theme::junie());
    let mut buffer = Buffer::empty(screen);
    for y in 0..screen.height {
        for x in 0..screen.width {
            if let Some(cell) = buffer.cell_mut(Position::new(x, y)) {
                cell.set_symbol("#");
            }
        }
    }
    runtime
        .draw_scene(screen, &mut buffer, |ui, _area| draw(ui, at))
        .commit_presented();
    buffer
}

/// W04-04: minimum width and clipped marker/label behavior.
#[test]
fn w04_toggle_minimum_width_and_clipping() {
    let natural = measure_size(|ui, c| Toggle::new(TOGGLE, TOGGLE_LABEL).measure(ui, c));
    assert_eq!(
        natural.preferred,
        (15, 1),
        "W04-04: natural size is gutter + marker + gap + label + on/off trailer"
    );
    for width in 0..=12 {
        let at = Rect::new(2, 1, width, 1);
        let buf = paint_contained(at, |ui, area| {
            Toggle::new(TOGGLE, TOGGLE_LABEL).on(true).draw(ui, area);
        });
        for y in 0..3 {
            for x in 0..20 {
                let pos = Position::new(x, y);
                if !at.contains(pos) {
                    assert_eq!(
                        cell_symbol(&buf, x, y),
                        "#",
                        "W04-04: width={width} must not write outside {at:?}"
                    );
                }
            }
        }
    }
    // A clipped toggle keeps its marker state readable.
    let clipped = paint_state(
        Rect::new(0, 0, 4, 1),
        TOGGLE,
        ReferenceState::default(),
        |ui, area| {
            Toggle::new(TOGGLE, TOGGLE_LABEL).on(true).draw(ui, area);
        },
    );
    assert_eq!(
        row_text(&clipped, 0, 4).chars().count(),
        4,
        "W04-04: a 4-wide clip still fills its row"
    );
}

// ---------------------------------------------------------------------------
// W05 RadioGroup (termrock-controls)
// ---------------------------------------------------------------------------

const RADIO: Id = Id::root("control.states.radio");
const RADIO_ITEMS: [&str; 3] = ["First", "Second", "Third"];
const RADIO_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 16,
    height: 3,
};

/// W05-01: the chosen item differs from the cursor and the hovered item.
#[test]
fn w05_radio_chosen_cursor_and_hovered_differ() {
    let mut st = RadioGroupState::default();
    st.set_cursor(1, ItemKey::index(1));
    // Group-level focus (the cursor row wears it) plus hover on row 2 only.
    let buf = paint(RADIO_AREA, Theme::junie(), |ui, area| {
        ui.reference(
            Some(
                ReferenceTarget::new(
                    RADIO,
                    ReferenceState::FOCUSED.union(ReferenceState::HOVERED),
                )
                .part(PartRef::item(Part::ROW, ItemKey::index(2))),
            ),
            |ui| {
                RadioGroup::new(RADIO)
                    .value(ItemKey::index(0))
                    .draw(ui, area, &st, &RADIO_ITEMS);
            },
        );
    });
    // Chosen row carries the filled marker; cursor row carries the gutter.
    assert!(
        row_text(&buf, 0, RADIO_AREA.width).contains("●"),
        "W05-01: the chosen row must show ●"
    );
    assert!(
        !row_text(&buf, 1, RADIO_AREA.width).contains("●"),
        "W05-01: the cursor row is not chosen"
    );
    assert_eq!(
        cell_symbol(&buf, 0, 1),
        "▎",
        "W05-01: the cursor row must show the gutter"
    );
    assert_eq!(
        cell_symbol(&buf, 0, 0),
        " ",
        "W05-01: the chosen row is not the cursor"
    );
    // Hover lifts exactly the hovered row.
    let no_hover = paint_state(RADIO_AREA, RADIO, ReferenceState::FOCUSED, |ui, area| {
        RadioGroup::new(RADIO)
            .value(ItemKey::index(0))
            .draw(ui, area, &st, &RADIO_ITEMS);
    });
    assert_ne!(
        buf, no_hover,
        "W05-01: hovering a third row must change the paint"
    );
    for y in [0, 1] {
        for x in 0..RADIO_AREA.width {
            assert_eq!(
                buf.cell(Position::new(x, y)),
                no_hover.cell(Position::new(x, y)),
                "W05-01: hover must not leak outside row 2"
            );
        }
    }
}

/// Shared radio rig: controlled value, cursor state and choose log.
struct RadioApp {
    value: Rc<Cell<Option<ItemKey>>>,
    st: RadioGroupState,
    chose: Rc<RefCell<Vec<ItemKey>>>,
    disabled: bool,
}

impl RadioApp {
    fn items() -> [&'static str; 3] {
        RADIO_ITEMS
    }
}

impl App for RadioApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let chose = Rc::clone(&self.chose);
        let value = Rc::clone(&self.value);
        let mut group = RadioGroup::new(RADIO).disabled(self.disabled);
        if let Some(v) = value.get() {
            group = group.value(v);
        }
        group
            .update(cx, &mut self.st, &Self::items())
            .on_action(|action| {
                if let RadioGroupAction::Chose(key) = action {
                    value.set(Some(key));
                    chose.borrow_mut().push(key);
                }
            })
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let mut group = RadioGroup::new(RADIO).disabled(self.disabled);
        if let Some(v) = self.value.get() {
            group = group.value(v);
        }
        group.draw(ui, RADIO_AREA, &self.st, &Self::items());
    }
}

/// W05-02 green half: a disabled group ignores keyboard and pointer.
/// The per-option half is the `w05_radio_per_option_disabled` parity record.
#[test]
fn w05_radio_disabled_group_ignores_keyboard_and_pointer() {
    let value = Rc::new(Cell::new(Some(ItemKey::index(0))));
    let chose = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        RadioApp {
            value: Rc::clone(&value),
            st: RadioGroupState::default(),
            chose: Rc::clone(&chose),
            disabled: true,
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    assert!(
        !app.tab_to(RADIO),
        "W05-02: a disabled group must be unreachable"
    );
    let _ = app.key(KeyCode::Down);
    let _ = app.key(KeyCode::Up);
    let _ = app.key(KeyCode::Enter);
    let _ = app.key(KeyCode::Char(' '));
    let _ = app.click(4, 1);
    let _ = app.click(4, 2);
    assert!(
        chose.borrow().is_empty(),
        "W05-02: a disabled group must never choose"
    );
    assert_eq!(
        value.get(),
        Some(ItemKey::index(0)),
        "W05-02: the value must not move"
    );
}

/// W05-02/W05-04 parity record: the reference expects disabled first, middle
/// and last options (plus all-disabled groups) with keyboard and pointer.
/// The candidate `RadioGroup` has whole-group `.disabled(bool)` only — no
/// per-option disabled — so the case is inexpressible.
#[test]
#[ignore = "PARITY W05-02/W05-04: RadioGroup has no per-option disabled (whole-group .disabled only)"]
fn w05_radio_per_option_disabled() {
    panic!("PARITY W05-02: cannot disable option 1 of 3; RadioGroup has no per-option disabled");
}

/// W05-03: reorder or delete a chosen/cursor item between pointer press and
/// release — stable keys, no crash, no phantom choose.
#[test]
fn w05_radio_reorder_or_delete_between_press_and_release() {
    struct KeyedRadio {
        items: Rc<RefCell<Vec<String>>>,
        st: RadioGroupState,
        chose: Rc<RefCell<Vec<ItemKey>>>,
    }
    impl App for KeyedRadio {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let chose = Rc::clone(&self.chose);
            let items = self.items.borrow();
            RadioGroup::new(RADIO)
                .key(|s: &String| ItemKey::text(s))
                .update(cx, &mut self.st, &items)
                .on_action(|action| {
                    if let RadioGroupAction::Chose(key) = action {
                        chose.borrow_mut().push(key);
                    }
                })
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let items = self.items.borrow();
            RadioGroup::new(RADIO)
                .key(|s: &String| ItemKey::text(s))
                .draw(ui, RADIO_AREA, &self.st, &items);
        }
    }
    fn rig() -> (
        Harness<KeyedRadio>,
        Rc<RefCell<Vec<String>>>,
        Rc<RefCell<Vec<ItemKey>>>,
    ) {
        let items = Rc::new(RefCell::new(vec![
            "alpha".to_string(),
            "bravo".to_string(),
            "charlie".to_string(),
        ]));
        let chose = Rc::new(RefCell::new(Vec::new()));
        let app = Harness::new(
            KeyedRadio {
                items: Rc::clone(&items),
                st: RadioGroupState::default(),
                chose: Rc::clone(&chose),
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        (app, items, chose)
    }

    // Delete the pressed item before release: the captured key is gone, so
    // nothing is chosen (a removed target must not fire).
    let (mut app, items, chose) = rig();
    let _ = app.mouse(MouseKind::Down, 4, 1);
    items.borrow_mut().remove(1);
    let _ = app.mouse(MouseKind::Up, 4, 1);
    assert!(
        chose.borrow().is_empty(),
        "W05-03: deleting the pressed item must choose nothing, got {:?}",
        chose.borrow()
    );

    // Reorder under a held press: the click resolves by stable key — the
    // pressed item is chosen even though it moved rows.
    let (mut app, items, chose) = rig();
    let _ = app.mouse(MouseKind::Down, 4, 1);
    items.borrow_mut().swap(0, 1);
    let _ = app.mouse(MouseKind::Up, 4, 1);
    assert_eq!(
        chose.borrow().as_slice(),
        &[ItemKey::text("bravo")],
        "W05-03: a reorder under a held press must keep the stable key"
    );
}

/// W05-04 green half: empty groups draw and ignore input; narrow vertical
/// layouts stay contained. All-disabled and horizontal halves are parity
/// records (`w05_radio_per_option_disabled`, `w05_radio_no_horizontal`).
#[test]
fn w05_radio_empty_and_narrow_vertical() {
    // Empty: inert paint, inert input, cursor stays empty.
    let empty = paint_plain(RADIO_AREA, |ui, area| {
        RadioGroup::<&str>::new(RADIO).draw(ui, area, &RadioGroupState::default(), &[]);
    });
    assert_eq!(
        row_text(&empty, 0, RADIO_AREA.width).trim(),
        "",
        "W05-04: an empty group paints no rows"
    );
    struct EmptyRadio {
        st: RadioGroupState,
        chose: Rc<Cell<u32>>,
    }
    impl App for EmptyRadio {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let chose = Rc::clone(&self.chose);
            let empty: [&str; 0] = [];
            RadioGroup::new(RADIO)
                .update(cx, &mut self.st, &empty)
                .on_action(|_| chose.set(chose.get() + 1))
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let empty: [&str; 0] = [];
            RadioGroup::new(RADIO).draw(ui, RADIO_AREA, &self.st, &empty);
        }
    }
    let chose = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        EmptyRadio {
            st: RadioGroupState::default(),
            chose: Rc::clone(&chose),
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    let _ = app.key(KeyCode::Down);
    let _ = app.key(KeyCode::Enter);
    let _ = app.click(4, 1);
    assert_eq!(chose.get(), 0, "W05-04: an empty group never chooses");

    // Narrow vertical: widths 0-4 stay inside the allocation.
    for width in 0..=4 {
        let at = Rect::new(2, 1, width, 3);
        let buf = paint_contained(at, |ui, area| {
            RadioGroup::new(RADIO)
                .value(ItemKey::index(0))
                .draw(ui, area, &RadioGroupState::default(), &RADIO_ITEMS);
        });
        for y in 0..3 {
            for x in 0..20 {
                if !at.contains(Position::new(x, y)) {
                    assert_eq!(
                        cell_symbol(&buf, x, y),
                        "#",
                        "W05-04: width={width} must not write outside {at:?}"
                    );
                }
            }
        }
    }
}

/// W05-04 parity record: the reference expects a narrow horizontal layout.
/// The candidate `RadioGroup` has no orientation API (vertical only), so the
/// case is inexpressible.
#[test]
#[ignore = "PARITY W05-04: RadioGroup has no horizontal orientation (vertical only)"]
fn w05_radio_no_horizontal() {
    panic!("PARITY W05-04: cannot lay a radio group out horizontally; no orientation API");
}

// ---------------------------------------------------------------------------
// W06 ChipBar (termrock-navigation)
// ---------------------------------------------------------------------------

const CHIP: Id = Id::root("control.states.chip");
const CHIP_ITEMS: [&str; 3] = ["alpha", "beta", "gamma"];
const CHIP_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 32,
    height: 1,
};

fn chip_state(checked: &[usize], cursor: Option<usize>) -> ChipBarState {
    let mut st = ChipBarState::default();
    for i in checked {
        st.checked_mut().insert(ItemKey::index(*i));
    }
    if let Some(i) = cursor {
        st.set_cursor(i, ItemKey::index(i));
    }
    st
}

fn paint_chips(st: &ChipBarState, state: ReferenceState, closable: bool) -> Buffer {
    paint_state(CHIP_AREA, CHIP, state, |ui, area| {
        ChipBar::new(CHIP)
            .select_mode(SelectMode::Multi)
            .closable(closable)
            .draw(ui, area, st, &CHIP_ITEMS);
    })
}

/// W06-01: checked/unselected and checked/focused chips.
#[test]
fn w06_chipbar_checked_focus_matrix() {
    let plain = ReferenceState::default();
    let unchecked = chip_state(&[], None);
    let checked = chip_state(&[0], None);
    let checked_focused = chip_state(&[0], Some(0));

    let unchecked_plain = paint_chips(&unchecked, plain, false);
    let checked_plain = paint_chips(&checked, plain, false);
    assert_ne!(
        unchecked_plain, checked_plain,
        "W06-01: a checked chip must differ from unselected"
    );

    let checked_focus = paint_chips(&checked_focused, ReferenceState::FOCUSED, false);
    assert_ne!(
        checked_plain, checked_focus,
        "W06-01: focus must add to a checked chip"
    );

    let unchecked_focused = chip_state(&[], Some(1));
    let unchecked_focus = paint_chips(&unchecked_focused, ReferenceState::FOCUSED, false);
    assert_ne!(
        checked_focus, unchecked_focus,
        "W06-01: checked/focused must differ from unselected/focused"
    );

    // The checked marker survives focus: exactly one chip wears it in both.
    for buf in [&checked_plain, &checked_focus] {
        let row = row_text(buf, 0, CHIP_AREA.width);
        assert_eq!(
            row.matches('✓').count() + row.matches("●").count(),
            1,
            "W06-01: exactly one checked marker in {row:?}"
        );
    }
}

/// Shared chip rig: mutable items, cursor state and action log.
struct ChipApp {
    items: Rc<RefCell<Vec<String>>>,
    st: ChipBarState,
    actions: Rc<RefCell<Vec<ChipBarAction>>>,
    closable: bool,
    disabled: bool,
    add: Option<&'static str>,
}

impl App for ChipApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let actions = Rc::clone(&self.actions);
        let items = self.items.borrow();
        let mut bar = ChipBar::new(CHIP)
            .select_mode(SelectMode::Multi)
            .closable(self.closable)
            .disabled(self.disabled);
        if let Some(label) = self.add {
            bar = bar.add(label);
        }
        bar.update(cx, &mut self.st, &items)
            .on_action(|action| actions.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let items = self.items.borrow();
        let mut bar = ChipBar::new(CHIP)
            .select_mode(SelectMode::Multi)
            .closable(self.closable)
            .disabled(self.disabled);
        if let Some(label) = self.add {
            bar = bar.add(label);
        }
        bar.draw(ui, CHIP_AREA, &self.st, &items);
    }
}

impl ChipApp {
    fn rig(
        items: Vec<String>,
        closable: bool,
        disabled: bool,
        add: Option<&'static str>,
    ) -> (
        Harness<ChipApp>,
        Rc<RefCell<Vec<String>>>,
        Rc<RefCell<Vec<ChipBarAction>>>,
    ) {
        let items = Rc::new(RefCell::new(items));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let app = Harness::new(
            ChipApp {
                items: Rc::clone(&items),
                st: ChipBarState::default(),
                actions: Rc::clone(&actions),
                closable,
                disabled,
                add,
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        (app, items, actions)
    }
}

/// Keyed chip rig shared by the W06-02 close tests.
struct KeyedChips {
    inner: ChipApp,
}

impl App for KeyedChips {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let actions = Rc::clone(&self.inner.actions);
        let items = self.inner.items.borrow();
        ChipBar::new(CHIP)
            .key(|s: &String| ItemKey::text(s))
            .select_mode(SelectMode::Multi)
            .closable(true)
            .update(cx, &mut self.inner.st, &items)
            .on_action(|action| actions.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let items = self.inner.items.borrow();
        ChipBar::new(CHIP)
            .key(|s: &String| ItemKey::text(s))
            .select_mode(SelectMode::Multi)
            .closable(true)
            .draw(ui, CHIP_AREA, &self.inner.st, &items);
    }
}

fn keyed_chip_rig() -> (
    Harness<KeyedChips>,
    Rc<RefCell<Vec<String>>>,
    Rc<RefCell<Vec<ChipBarAction>>>,
) {
    let items = Rc::new(RefCell::new(vec![
        "alpha".to_string(),
        "beta".to_string(),
        "gamma".to_string(),
    ]));
    let actions = Rc::new(RefCell::new(Vec::new()));
    let app = Harness::new(
        KeyedChips {
            inner: ChipApp {
                items: Rc::clone(&items),
                st: ChipBarState::default(),
                actions: Rc::clone(&actions),
                closable: true,
                disabled: false,
                add: None,
            },
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    (app, items, actions)
}

/// W06-02/W06-05 parity record: every keyboard binding of a closable
/// `ChipBar` is dead — Space, Enter, arrows and Del all produce nothing. The
/// closable tables bind Delete, Backspace and `x` to the same
/// `ActionKey::custom("Remove")`, and `PublishedBindings::publish`
/// (`termrock-runtime/src/keymap.rs`) rejects the whole table when any action
/// repeats, so no binding of a closable bar is ever published. Only pointer
/// input reaches it.
#[test]
fn w06_chipbar_closable_keyboard_dead() {
    let (mut app, items, actions) = keyed_chip_rig();
    assert!(app.tab_to(CHIP));
    let _ = app.key(KeyCode::Char(' '));
    assert_eq!(
        actions.borrow().as_slice(),
        &[ChipBarAction::Toggled(ItemKey::text("alpha"))],
        "W06-02: Space must toggle the cursor chip"
    );
    let _ = app.key(KeyCode::Right);
    items.borrow_mut().swap(0, 1);
    let _ = app.key(KeyCode::Delete);
    assert_eq!(
        actions.borrow().as_slice(),
        &[
            ChipBarAction::Toggled(ItemKey::text("alpha")),
            ChipBarAction::Closed(ItemKey::text("beta")),
        ],
        "W06-02: Del after reorder must close the cursor chip by stable key"
    );
}

/// W06-02 parity record: the reference demands a working close-button press,
/// and the candidate's own `ChipBarAction::Closed` doc promises "a click on
/// `×`". The candidate registers the whole-chip `LABEL` part after — and
/// covering — the one-cell `CLOSE` part (`termrock-navigation/src/chip.rs`
/// `draw`), so a press on `×` hit-tests as the chip body and toggles the
/// chip instead of closing it.
#[test]
fn w06_chipbar_close_press_then_reorder_or_delete() {
    // Reorder under a held close press: the close keeps the pressed chip.
    let (mut app, items, actions) = keyed_chip_rig();
    let close = app
        .area_of_part(CHIP, PartRef::item(Part::CLOSE, ItemKey::text("beta")))
        .expect("W06-02: the beta close part must be registered");
    let _ = app.mouse(MouseKind::Down, close.x, close.y);
    items.borrow_mut().swap(0, 1);
    let _ = app.mouse(MouseKind::Up, close.x, close.y);
    assert_eq!(
        actions.borrow().as_slice(),
        &[ChipBarAction::Closed(ItemKey::text("beta"))],
        "W06-02: a reorder under a held close must keep the stable key"
    );

    // Delete the pressed chip before release: the close never fires.
    let (mut app, items, actions) = keyed_chip_rig();
    let close = app
        .area_of_part(CHIP, PartRef::item(Part::CLOSE, ItemKey::text("beta")))
        .expect("W06-02: the beta close part must be registered");
    let _ = app.mouse(MouseKind::Down, close.x, close.y);
    items.borrow_mut().remove(1);
    let _ = app.mouse(MouseKind::Up, close.x, close.y);
    assert!(
        actions.borrow().is_empty(),
        "W06-02: deleting the pressed chip must cancel the close, got {:?}",
        actions.borrow()
    );
}

/// W06-03: overflow at exact-fit, one-cell-short and long Unicode widths.
#[test]
fn w06_chipbar_overflow_exact_fit_and_unicode() {
    let items = ["alpha", "beta", "gamma"];
    fn shows_all(width: u16, items: &[&str]) -> bool {
        let area = Rect::new(0, 0, width, 1);
        let buf = paint_plain(area, |ui, area| {
            ChipBar::new(CHIP).draw(ui, area, &ChipBarState::default(), items);
        });
        let row = row_text(&buf, 0, width);
        items.iter().all(|item| row.contains(item))
    }
    // The exact fit is the smallest width showing every full label.
    let exact = (1..=CHIP_AREA.width)
        .find(|w| shows_all(*w, &items))
        .expect("W06-03: the strip must fit in 32 columns");
    // One cell short overflows: at least one full label is gone.
    let short_area = Rect::new(0, 0, exact - 1, 1);
    let short = paint_plain(short_area, |ui, area| {
        ChipBar::new(CHIP).draw(ui, area, &ChipBarState::default(), &items);
    });
    let short_row = row_text(&short, 0, exact - 1);
    assert!(
        !items.iter().all(|item| short_row.contains(item)),
        "W06-03: one cell short of exact fit ({exact}) must overflow, got {short_row:?}"
    );
    // Long Unicode labels measure by display width and stay contained.
    let wide = ["界 interface with a very long label", "e\u{301} combined"];
    let wide_area = Rect::new(2, 1, 16, 1);
    let buf = paint_contained(wide_area, |ui, area| {
        ChipBar::new(CHIP).draw(ui, area, &ChipBarState::default(), &wide);
    });
    for y in 0..3 {
        for x in 0..20 {
            if !wide_area.contains(Position::new(x, y)) {
                assert_eq!(
                    cell_symbol(&buf, x, y),
                    "#",
                    "W06-03: wide labels must not write outside {wide_area:?}"
                );
            }
        }
    }
}

/// W06-04: empty strip with and without Add.
#[test]
fn w06_chipbar_empty_with_and_without_add() {
    // Without Add: empty paint, inert input.
    let (mut app, _items, actions) = ChipApp::rig(vec![], false, false, None);
    assert_eq!(
        app.row(0).trim(),
        "",
        "W06-04: an empty strip without Add paints nothing"
    );
    assert!(app.tab_to(CHIP));
    let _ = app.key(KeyCode::Enter);
    let _ = app.click(2, 0);
    assert!(
        actions.borrow().is_empty(),
        "W06-04: an empty strip without Add stays inert"
    );

    // With Add: the affordance shows and fires AddRequested.
    let (mut app, _items, actions) =
        ChipApp::rig(vec![], false, false, Some("Add"));
    assert!(
        app.row(0).contains("Add"),
        "W06-04: an empty strip with Add must offer it"
    );
    assert!(app.tab_to(CHIP));
    let _ = app.key(KeyCode::Enter);
    assert_eq!(
        actions.borrow().as_slice(),
        &[ChipBarAction::AddRequested],
        "W06-04: Enter on the empty Add stop must request Add"
    );
}

/// W06-05: disabled chip body and close eligibility — nothing fires.
#[test]
fn w06_chipbar_disabled_body_and_close_inert() {
    let (mut app, _items, actions) = ChipApp::rig(
        vec!["alpha".to_string(), "beta".to_string()],
        true,
        true,
        None,
    );
    assert!(
        !app.tab_to(CHIP),
        "W06-05: a disabled strip must be unreachable"
    );
    let _ = app.key(KeyCode::Enter);
    let _ = app.key(KeyCode::Char(' '));
    let _ = app.key(KeyCode::Delete);
    let _ = app.click(3, 0);
    let _ = app.click(12, 0);
    assert!(
        actions.borrow().is_empty(),
        "W06-05: disabled body and close must never fire, got {:?}",
        actions.borrow()
    );
    // Disabled still paints every chip label.
    let row = app.row(0);
    assert!(
        row.contains("alpha") && row.contains("beta"),
        "W06-05: disabled chips stay legible, got {row:?}"
    );
}

// ---------------------------------------------------------------------------
// W07 Field (termrock-fields)
// ---------------------------------------------------------------------------

const FIELD_INPUT: Id = Id::root("control.states.field.input");
const FIELD_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 24,
    height: 3,
};

fn paint_field(
    area: Rect,
    help: Option<&'static str>,
    error: Option<&'static str>,
    required: bool,
    optional_suffix: bool,
) -> Buffer {
    paint_state(area, FIELD_INPUT, ReferenceState::FOCUSED, |ui, area| {
        let input = TextInput::new(FIELD_INPUT).value("text");
        let mut field = Field::new("Name", input)
            .required(required)
            .optional_suffix(optional_suffix);
        if let Some(h) = help {
            field = field.help(h);
        }
        field = field.error(error);
        field.draw(ui, area, &TextInputState::default());
    })
}

/// W07-01: label/help versus label/error at identical allocated height.
#[test]
fn w07_field_help_and_error_share_height() {
    let used = Cell::new(Rect::default());
    let with_help = paint_state(
        FIELD_AREA,
        FIELD_INPUT,
        ReferenceState::FOCUSED,
        |ui, area| {
            let input = TextInput::new(FIELD_INPUT).value("text");
            used.set(
                Field::new("Name", input)
                    .help("pick a name")
                    .draw(ui, area, &TextInputState::default()),
            );
        },
    );
    let help_h = used.get().height;
    let with_error = paint_state(
        FIELD_AREA,
        FIELD_INPUT,
        ReferenceState::FOCUSED,
        |ui, area| {
            let input = TextInput::new(FIELD_INPUT).value("text");
            used.set(
                Field::new("Name", input)
                    .error(Some("required"))
                    .draw(ui, area, &TextInputState::default()),
            );
        },
    );
    assert_eq!(
        help_h,
        used.get().height,
        "W07-01: help and error must occupy identical height"
    );
    assert!(
        row_text(&with_help, 2, FIELD_AREA.width).contains("pick a name"),
        "W07-01: the help row must show"
    );
    assert!(
        row_text(&with_error, 2, FIELD_AREA.width).contains("required"),
        "W07-01: the error row must show in the same row"
    );
    assert_ne!(
        with_help, with_error,
        "W07-01: error must style distinctly from help"
    );
}

/// W07-02: required and optional suffix at its width boundary.
#[test]
fn w07_field_suffix_width_boundaries() {
    // Optional shows while name + 12 fits: "Name" needs width 16.
    let shown = paint_field(Rect::new(0, 0, 16, 3), None, None, false, true);
    assert!(
        row_text(&shown, 0, 16).contains("optional"),
        "W07-02: width 16 must show the optional suffix"
    );
    let hidden = paint_field(Rect::new(0, 0, 15, 3), None, None, false, true);
    assert!(
        !row_text(&hidden, 0, 15).contains("optional"),
        "W07-02: width 15 must hide the optional suffix"
    );
    // Required marker "*" clips below width 8, shows at 8.
    let clipped = paint_field(Rect::new(0, 0, 7, 3), None, None, true, false);
    assert!(
        !row_text(&clipped, 0, 7).contains('*'),
        "W07-02: width 7 must clip the required marker"
    );
    let marked = paint_field(Rect::new(0, 0, 8, 3), None, None, true, false);
    assert!(
        row_text(&marked, 0, 8).contains('*'),
        "W07-02: width 8 must show the required marker"
    );
}

/// Shared field rig: labelled input with an optional error.
struct FieldApp {
    value: String,
    st: TextInputState,
    error: Option<&'static str>,
}

impl App for FieldApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        TextInput::new(FIELD_INPUT)
            .update(cx, &mut self.st, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let input = TextInput::new(FIELD_INPUT).value(&self.value);
        Field::new("Name", input)
            .error(self.error)
            .draw(ui, FIELD_AREA, &self.st);
        ui.register_control(
            Id::root("control.states.field.sentinel"),
            Rect::new(0, 5, 8, 1),
            Focusability::Focusable,
        );
    }
}

/// W07-03: child editing with error preserves its focus gutter and cursor.
#[test]
fn w07_field_editing_with_error_keeps_gutter_and_cursor() {
    let mut app = Harness::new(
        FieldApp {
            value: String::new(),
            st: TextInputState::default(),
            error: Some("required"),
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    assert!(app.tab_to(FIELD_INPUT));
    // Enter begins editing (the reference navigation/editing split); typing
    // alone never starts a draft.
    let _ = app.key(KeyCode::Enter);
    let _ = app.type_str("ab");
    assert!(
        app.state_of(FIELD_INPUT).contains(StateFlags::FOCUSED),
        "W07-03: the child keeps focus while editing"
    );
    assert!(
        app.state_of(FIELD_INPUT).contains(StateFlags::EDITING),
        "W07-03: typing must put the child into editing"
    );
    assert_eq!(
        cell_symbol(app.buffer(), 0, 1),
        "▎",
        "W07-03: the error must not steal the focus gutter"
    );
    assert!(
        app.cursor().is_some(),
        "W07-03: the error must not steal the hardware cursor"
    );
    assert!(
        app.row(2).contains("required"),
        "W07-03: the error row still shows while editing"
    );
    assert!(
        app.row(1).contains("ab"),
        "W07-03: the draft still shows while editing"
    );
}

/// W07-04: one Tab stop for a labelled input, not two.
#[test]
fn w07_field_has_one_tab_stop() {
    const SENTINEL: Id = Id::root("control.states.field.sentinel");
    let mut app = Harness::new(
        FieldApp {
            value: String::new(),
            st: TextInputState::default(),
            error: None,
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    assert!(app.tab_to(FIELD_INPUT));
    // One Tab leaves the field for the sentinel — no second field stop.
    let _ = app.key(KeyCode::Tab);
    assert!(
        app.state_of(SENTINEL).contains(StateFlags::FOCUSED),
        "W07-04: one Tab must reach the sentinel"
    );
    assert!(
        !app.state_of(FIELD_INPUT).contains(StateFlags::FOCUSED),
        "W07-04: the field must hold a single stop"
    );
    let _ = app.key(KeyCode::Tab);
    assert!(
        app.state_of(FIELD_INPUT).contains(StateFlags::FOCUSED),
        "W07-04: Tab must cycle back to the field"
    );
}

// ---------------------------------------------------------------------------
// W08 TextInput (termrock-fields)
// ---------------------------------------------------------------------------

const INPUT: Id = Id::root("control.states.input");
const INPUT_SENTINEL: Id = Id::root("control.states.input.sentinel");
const INPUT_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 24,
    height: 1,
};

fn required(s: &str) -> Result<(), FieldError> {
    if s.is_empty() {
        Err(FieldError::new("required"))
    } else {
        Ok(())
    }
}

fn min_len_3(s: &str) -> Result<(), FieldError> {
    if s.len() >= 3 {
        Ok(())
    } else {
        Err(FieldError::new("too short"))
    }
}

static REQUIRED: fn(&str) -> Result<(), FieldError> = required;
static MIN_LEN_3: fn(&str) -> Result<(), FieldError> = min_len_3;

/// Shared input rig: caller-owned value plus action/error/debug mirrors.
struct InputApp {
    value: Rc<RefCell<String>>,
    st: TextInputState,
    actions: Rc<RefCell<Vec<TextAction>>>,
    error_seen: Rc<Cell<bool>>,
    debug_text: Rc<RefCell<String>>,
    disabled: bool,
    secret: bool,
    validator: Option<&'static dyn Validate>,
}

impl InputApp {
    fn input(disabled: bool, secret: bool, validator: Option<&'static dyn Validate>) -> TextInput<'static> {
        let mut input = TextInput::new(INPUT).disabled(disabled);
        if secret {
            input = input.secret(SecretPolicy::default());
        }
        if let Some(v) = validator {
            input = input.validate(v);
        }
        input
    }
}

impl App for InputApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let actions = Rc::clone(&self.actions);
        let mut borrowed = self.value.borrow_mut();
        let r = Self::input(self.disabled, self.secret, self.validator)
            .update(cx, &mut self.st, &mut borrowed)
            .on_action(|action| actions.borrow_mut().push(action));
        self.error_seen.set(self.st.error().is_some());
        self.debug_text.replace(format!("{:?}", self.st));
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let borrowed = self.value.borrow();
        Self::input(self.disabled, self.secret, self.validator)
            .value(&borrowed)
            .draw(ui, INPUT_AREA, &self.st);
        ui.register_control(INPUT_SENTINEL, Rect::new(0, 3, 8, 1), Focusability::Focusable);
    }
}

struct InputRig {
    app: Harness<InputApp>,
    value: Rc<RefCell<String>>,
    actions: Rc<RefCell<Vec<TextAction>>>,
    error_seen: Rc<Cell<bool>>,
    debug_text: Rc<RefCell<String>>,
}

impl InputRig {
    fn new(initial: &str) -> Self {
        Self::with(initial, false, false, None)
    }

    fn with(
        initial: &str,
        disabled: bool,
        secret: bool,
        validator: Option<&'static dyn Validate>,
    ) -> Self {
        let value = Rc::new(RefCell::new(initial.to_string()));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let error_seen = Rc::new(Cell::new(false));
        let debug_text = Rc::new(RefCell::new(String::new()));
        let app = Harness::new(
            InputApp {
                value: Rc::clone(&value),
                st: TextInputState::default(),
                actions: Rc::clone(&actions),
                error_seen: Rc::clone(&error_seen),
                debug_text: Rc::clone(&debug_text),
                disabled,
                secret,
                validator,
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        Self {
            app,
            value,
            actions,
            error_seen,
            debug_text,
        }
    }

    fn committed(&self) -> usize {
        actions_count(&self.actions, TextAction::Committed)
    }
}

fn actions_count(actions: &Rc<RefCell<Vec<TextAction>>>, what: TextAction) -> usize {
    actions.borrow().iter().filter(|a| **a == what).count()
}

/// W08-01: focus then Enter begins; edit then Escape restores the original.
#[test]
fn w08_input_enter_begins_escape_restores() {
    let mut rig = InputRig::new("original");
    assert!(rig.app.tab_to(INPUT));
    assert!(
        !rig.app.state_of(INPUT).contains(StateFlags::EDITING),
        "W08-01: focus alone must not edit"
    );
    let _ = rig.app.key(KeyCode::Enter);
    assert!(
        rig.app.state_of(INPUT).contains(StateFlags::EDITING),
        "W08-01: Enter must begin editing"
    );
    let _ = rig.app.type_str("xy");
    assert!(rig.app.row(0).contains("originalxy"), "W08-01: draft shows");
    let _ = rig.app.key(KeyCode::Esc);
    assert_eq!(
        actions_count(&rig.actions, TextAction::Cancelled),
        1,
        "W08-01: Escape must cancel exactly once"
    );
    assert_eq!(
        rig.value.borrow().as_str(),
        "original",
        "W08-01: Escape must restore the original value"
    );
    assert!(
        !rig.app.state_of(INPUT).contains(StateFlags::EDITING),
        "W08-01: Escape must leave edit mode"
    );
    assert!(
        rig.app.row(0).contains("original") && !rig.app.row(0).contains("originalxy"),
        "W08-01: paint must show the restored value"
    );
}

/// W08-02 green half: a completed click positions the caret by displayed
/// grapheme width — wide, combining and masked. Down-only is the separate
/// `w08_input_down_only_does_not_edit` parity record.
#[test]
fn w08_input_click_positions_caret_by_grapheme() {
    // Text starts two cells in: a/界/e-combining/b at display cols 0/1/3/4.
    let mut rig = InputRig::new("a界e\u{301}b");
    let _ = rig.app.click(3, 0);
    assert_eq!(
        rig.app.cursor(),
        Some(Position::new(3, 0)),
        "W08-02: clicking the wide grapheme must land on its first cell"
    );
    let _ = rig.app.click(5, 0);
    assert_eq!(
        rig.app.cursor(),
        Some(Position::new(5, 0)),
        "W08-02: clicking the combining grapheme must land on its cell"
    );

    // Masked paint hides the plaintext (one mask cell per grapheme); the
    // click-geometry half is the `w08_input_masked_clicks_follow_display`
    // parity record.
    let rig = InputRig::with("日ab", false, true, None);
    assert!(
        !rig.app.row(0).contains("日"),
        "W08-02: masked paint must hide the plaintext"
    );
}

/// W08-02 parity record: masked paint shows one cell per grapheme (matching
/// the legacy `masked_clicks_follow_display_graphemes` oracle in
/// `src/widgets/input.rs` at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`),
/// but the candidate press arm maps the click column through the plaintext
/// widths (`set_cursor_line_col` on the unmasked draft in
/// `termrock-fields/src/input.rs`). Clicking the second mask cell of "日ab"
/// lands the caret on the first cell instead.
#[test]
#[ignore = "PARITY W08-02: masked clicks use plaintext widths (caret lands on cell 1 of 2), reference follows display graphemes"]
fn w08_input_masked_clicks_follow_display() {
    let mut rig = InputRig::with("日ab", false, true, None);
    let _ = rig.app.click(3, 0);
    assert_eq!(
        rig.app.cursor(),
        Some(Position::new(3, 0)),
        "W08-02: clicking the second mask cell must land there"
    );
    let _ = rig.app.click(4, 0);
    assert_eq!(
        rig.app.cursor(),
        Some(Position::new(4, 0)),
        "W08-02: clicking the third mask cell must land there"
    );
}

/// W08-02 parity record: the reference matrix ("Down-only does not edit"),
/// the candidate spec ("Enter/F2 or completed click enters edit") and the
/// legacy widget (no press handler — `on_click` only) agree that a press
/// without release edits nothing. The candidate `TextInput::update` begins
/// the draft and moves the caret on `Phase::Press`, so Down alone edits.
#[test]
#[ignore = "PARITY W08-02: mouse Down alone begins the draft (Press arm calls begin), reference demands completed click"]
fn w08_input_down_only_does_not_edit() {
    let mut rig = InputRig::new("original");
    let _ = rig.app.mouse(MouseKind::Down, 5, 0);
    assert!(
        !rig.app.state_of(INPUT).contains(StateFlags::EDITING),
        "W08-02: Down alone must not begin editing"
    );
    assert!(
        rig.actions.borrow().is_empty(),
        "W08-02: Down alone must emit no action"
    );
    assert_eq!(
        rig.value.borrow().as_str(),
        "original",
        "W08-02: Down alone must not touch the value"
    );
}

/// W08-03 green half: editing paste inserts; disabled paste is ignored. The
/// navigation-paste half is the `w08_input_navigation_paste_begins_editing`
/// parity record.
#[test]
fn w08_input_paste_inserts_and_disabled_ignores() {
    // Positive control: paste while editing inserts.
    let mut rig = InputRig::new("ab");
    assert!(rig.app.tab_to(INPUT));
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.paste("XY");
    assert_eq!(
        actions_count(&rig.actions, TextAction::Changed),
        1,
        "W08-03: paste while editing must change the draft"
    );
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(rig.value.borrow().as_str(), "abXY");

    // Disabled paste is ignored, editing or not.
    let mut rig = InputRig::with("locked", true, false, None);
    let _ = rig.app.paste("replacement");
    assert!(
        rig.actions.borrow().is_empty(),
        "W08-03: disabled paste must emit nothing"
    );
    assert_eq!(rig.value.borrow().as_str(), "locked");
    assert!(
        !rig.app.state_of(INPUT).contains(StateFlags::EDITING),
        "W08-03: disabled paste must not begin editing"
    );
}

/// W08-03 parity record: the reference (`TextInput::on_paste` in
/// `src/widgets/input.rs` at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`)
/// calls `begin_edit()` before inserting, so paste while navigating starts
/// editing. The candidate paste arm requires `st.is_editing()` and drops
/// navigation-mode paste entirely.
#[test]
#[ignore = "PARITY W08-03: paste while navigating is dropped (paste arm requires is_editing), reference begins editing"]
fn w08_input_navigation_paste_begins_editing() {
    let mut rig = InputRig::new("ab");
    assert!(rig.app.tab_to(INPUT));
    let _ = rig.app.paste("XY");
    assert!(
        rig.app.state_of(INPUT).contains(StateFlags::EDITING),
        "W08-03: navigation paste must begin editing"
    );
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(
        rig.value.borrow().as_str(),
        "abXY",
        "W08-03: navigation paste must land in the value"
    );
}

/// W08-04: edit then Tab/Shift+Tab/blur emits one commit and traverses.
#[test]
fn w08_input_tab_shift_tab_and_blur_commit_once() {
    // Tab commits once and moves forward.
    let mut rig = InputRig::new("");
    assert!(rig.app.tab_to(INPUT));
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.type_str("ab");
    let _ = rig.app.key(KeyCode::Tab);
    assert_eq!(rig.committed(), 1, "W08-04: Tab must commit exactly once");
    assert_eq!(rig.value.borrow().as_str(), "ab");
    assert!(
        rig.app.state_of(INPUT_SENTINEL).contains(StateFlags::FOCUSED),
        "W08-04: Tab must traverse forward after committing"
    );

    // Shift+Tab commits once and moves back.
    let mut rig = InputRig::new("");
    assert!(rig.app.tab_to(INPUT));
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.type_str("cd");
    let _ = rig.app.key(KeyCode::BackTab);
    assert_eq!(
        rig.committed(),
        1,
        "W08-04: Shift+Tab must commit exactly once"
    );
    assert_eq!(rig.value.borrow().as_str(), "cd");
    assert!(
        rig.app.state_of(INPUT_SENTINEL).contains(StateFlags::FOCUSED),
        "W08-04: Shift+Tab must traverse (two-stop ring wraps)"
    );

    // Blur commits once.
    let mut rig = InputRig::new("");
    assert!(rig.app.tab_to(INPUT));
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.type_str("ef");
    rig.app.blur();
    assert_eq!(rig.committed(), 1, "W08-04: blur must commit exactly once");
    assert_eq!(rig.value.borrow().as_str(), "ef");
    // The commit resolves the draft; focus itself is restored to the first
    // reachable entry by the documented MI-2 reconcile rule, so no unfocused
    // assertion belongs here.
    assert!(
        !rig.app.state_of(INPUT).contains(StateFlags::EDITING),
        "W08-04: blur must leave edit mode"
    );
}

/// W08-05: required empty and custom validation error; correction clears.
#[test]
fn w08_input_required_and_custom_errors_clear_on_correction() {
    // Required empty: commit fails, error latches, value untouched.
    let mut rig = InputRig::with("", false, false, Some(&REQUIRED));
    assert!(rig.app.tab_to(INPUT));
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.key(KeyCode::Enter);
    assert!(
        rig.error_seen.get(),
        "W08-05: committing empty required input must error"
    );
    assert_eq!(rig.value.borrow().as_str(), "");
    // Correction clears the prior error live and commits clean.
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.type_str("ok");
    assert!(
        !rig.error_seen.get(),
        "W08-05: correction must clear the required error"
    );
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(rig.value.borrow().as_str(), "ok");
    assert!(
        !rig.error_seen.get(),
        "W08-05: the clean commit must leave no error"
    );

    // Custom rule: short commits error, the fix clears.
    let mut rig = InputRig::with("", false, false, Some(&MIN_LEN_3));
    assert!(rig.app.tab_to(INPUT));
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.type_str("ab");
    let _ = rig.app.key(KeyCode::Enter);
    assert!(
        rig.error_seen.get(),
        "W08-05: the custom rule must reject 'ab'"
    );
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.type_str("c");
    assert!(
        !rig.error_seen.get(),
        "W08-05: 'abc' must satisfy the custom rule"
    );
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(rig.value.borrow().as_str(), "abc");
}

/// W08-06: secret mode keeps plaintext out of Debug, paint, cancel/close
/// rollback and zeroized state. The candidate library has no text-input
/// clipboard or log surface carrying input text (no `clipboard`/`copy`
/// path in core/runtime; `Diagnostic` carries ids and rects only), and
/// snapshots capture the masked paint asserted below — those reference
/// surfaces hold vacuously here.
#[test]
fn w08_input_secret_redacts_debug_paint_cancel_and_zeroize() {
    const SECRET: &str = "s3cr3t-pw";
    let mut rig = InputRig::with("", false, true, None);
    assert!(rig.app.tab_to(INPUT));
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.type_str(SECRET);
    assert!(
        !rig.app.row(0).contains(SECRET),
        "W08-06: secret paint must mask the draft"
    );
    assert!(
        rig.app.row(0).contains('•'),
        "W08-06: secret paint must show mask cells"
    );
    assert!(
        !rig.debug_text.borrow().contains(SECRET),
        "W08-06: secret state Debug must redact the draft, got {:?}",
        rig.debug_text.borrow()
    );
    // Cancel rolls back without leaking.
    let _ = rig.app.key(KeyCode::Esc);
    assert_eq!(rig.value.borrow().as_str(), "");
    assert!(
        !rig.debug_text.borrow().contains(SECRET),
        "W08-06: rollback after close must not leak the draft"
    );
    assert!(
        !rig.app.row(0).contains(SECRET),
        "W08-06: paint after rollback shows no plaintext"
    );

    // Committed secrets stay masked; zeroize wipes the draft.
    let mut rig = InputRig::with("", false, true, None);
    assert!(rig.app.tab_to(INPUT));
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.type_str(SECRET);
    let _ = rig.app.key(KeyCode::Enter);
    assert!(
        !rig.app.row(0).contains(SECRET),
        "W08-06: committed secret paint stays masked"
    );
    let mut st = TextInputState::sensitive();
    st.begin("wipe-me");
    st.zeroize();
    assert!(
        !format!("{st:?}").contains("wipe-me"),
        "W08-06: zeroize must wipe the draft"
    );
}

/// W08-07 parity record: the reference demands no silent overwrite when the
/// caller-owned value changes under an active draft. The candidate
/// `commit_target` writes the draft unconditionally
/// (`termrock-fields/src/input.rs`), so committing a stale draft silently
/// discards the external change.
#[test]
#[ignore = "PARITY W08-07: commit blindly overwrites external value changes (no revision/stamp check)"]
fn w08_input_external_change_is_not_silently_overwritten() {
    let mut rig = InputRig::new("orig");
    assert!(rig.app.tab_to(INPUT));
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.type_str("xy");
    // The world moves under the draft.
    rig.value.replace("external".to_string());
    let _ = rig.app.key(KeyCode::Enter);
    assert!(
        rig.value.borrow().as_str().contains("external"),
        "W08-07: commit must not silently discard the external change, got {:?}",
        rig.value.borrow()
    );
}

// ---------------------------------------------------------------------------
// W09 TextArea (termrock-fields)
// ---------------------------------------------------------------------------

const AREA: Id = Id::root("control.states.area");
const AREA_SENTINEL: Id = Id::root("control.states.area.sentinel");
const AREA_FIELD: Id = Id::root("control.states.area.field");
const AREA_TINY: Id = Id::root("control.states.area.tiny");
const AREA_ROWS: u16 = 3;
const AREA_RECT: Rect = Rect {
    x: 0,
    y: 0,
    width: 40,
    height: 6,
};
/// Eight single-grapheme-wide lines for scroll/page tests.
const AREA_DOC: &str = "L0\nL1\nL2\nL3\nL4\nL5\nL6\nL7";

/// Shared area rig: caller-owned value plus an action mirror.
struct AreaApp {
    value: Rc<RefCell<String>>,
    st: TextAreaState,
    actions: Rc<RefCell<Vec<TextAction>>>,
    rows: u16,
}

impl App for AreaApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let actions = Rc::clone(&self.actions);
        let mut borrowed = self.value.borrow_mut();
        TextArea::new(AREA, self.rows)
            .update(cx, &mut self.st, &mut borrowed)
            .on_action(|action| actions.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let borrowed = self.value.borrow();
        TextArea::new(AREA, self.rows)
            .value(&borrowed)
            .draw(ui, AREA_RECT, &self.st);
        ui.register_control(AREA_SENTINEL, Rect::new(0, 10, 8, 1), Focusability::Focusable);
    }
}

struct AreaRig {
    app: Harness<AreaApp>,
    value: Rc<RefCell<String>>,
    actions: Rc<RefCell<Vec<TextAction>>>,
}

impl AreaRig {
    fn new(initial: &str) -> Self {
        Self::with(initial, AREA_ROWS)
    }

    fn with(initial: &str, rows: u16) -> Self {
        let value = Rc::new(RefCell::new(initial.to_string()));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let app = Harness::new(
            AreaApp {
                value: Rc::clone(&value),
                st: TextAreaState::default(),
                actions: Rc::clone(&actions),
                rows,
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        Self {
            app,
            value,
            actions,
        }
    }

    fn committed(&self) -> usize {
        actions_count(&self.actions, TextAction::Committed)
    }
}

fn cell_style(app: &Harness<AreaApp>, x: u16, y: u16) -> (String, String, String) {
    let c = &app.buffer()[Position::new(x, y)];
    (format!("{:?}", c.fg), format!("{:?}", c.bg), format!("{:?}", c.modifier))
}

/// W09-01: edit Enter inserts a newline; Escape commits and leaves edit mode.
#[test]
fn w09_area_enter_newline_escape_commits() {
    let mut rig = AreaRig::new("ab");
    assert!(rig.app.tab_to(AREA));
    assert!(
        !rig.app.state_of(AREA).contains(StateFlags::EDITING),
        "W09-01: focus alone must not edit"
    );
    // A click begins without inserting (legacy `on_click`).
    let _ = rig.app.click(4, 0);
    assert!(
        rig.app.state_of(AREA).contains(StateFlags::EDITING),
        "W09-01: click must begin editing"
    );
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.key(KeyCode::Esc);
    assert_eq!(rig.committed(), 1, "W09-01: Escape must commit once");
    assert_eq!(
        actions_count(&rig.actions, TextAction::Cancelled),
        0,
        "W09-01: Escape must not cancel a document"
    );
    assert_eq!(
        rig.value.borrow().as_str(),
        "ab\n",
        "W09-01: the edit Enter newline must land in the value"
    );
    assert!(
        !rig.app.state_of(AREA).contains(StateFlags::EDITING),
        "W09-01: Escape must leave edit mode"
    );
}

/// W09-01 parity record: the legacy `TextArea::on_key` begins on nav Enter
/// without inserting — the newline belongs to *edit* Enter, which is why
/// the reference says "edit Enter inserts newline". The candidate binding
/// arm begins the draft and then runs `TextCmd::Newline` for the same key,
/// so nav Enter inserts a newline.
#[test]
#[ignore = "PARITY W09-01: nav Enter begins AND inserts a newline (begin then Newline run for one key), reference begins only"]
fn w09_area_nav_enter_begins_without_newline() {
    let mut rig = AreaRig::new("ab");
    assert!(rig.app.tab_to(AREA));
    let _ = rig.app.key(KeyCode::Enter);
    assert!(
        rig.app.state_of(AREA).contains(StateFlags::EDITING),
        "W09-01: nav Enter must still begin editing"
    );
    let _ = rig.app.key(KeyCode::Esc);
    assert_eq!(
        rig.value.borrow().as_str(),
        "ab",
        "W09-01: the begin-Enter must not insert a newline, got {:?}",
        rig.value.borrow()
    );
}

/// W09-02: navigation paste is ignored; edit-mode bracketed paste keeps newlines.
#[test]
fn w09_area_paste_ignored_navigating_kept_editing() {
    let mut rig = AreaRig::new("doc");
    assert!(rig.app.tab_to(AREA));
    let _ = rig.app.paste("ZZ");
    assert!(
        rig.actions.borrow().is_empty(),
        "W09-02: navigation paste must be ignored"
    );
    assert!(
        !rig.app.state_of(AREA).contains(StateFlags::EDITING),
        "W09-02: navigation paste must not begin editing"
    );
    assert_eq!(rig.value.borrow().as_str(), "doc");

    let _ = rig.app.click(2, 0);
    let _ = rig.app.paste("a\nb");
    assert_eq!(
        actions_count(&rig.actions, TextAction::Changed),
        1,
        "W09-02: edit paste must change the draft"
    );
    let _ = rig.app.key(KeyCode::Esc);
    assert_eq!(
        rig.value.borrow().as_str(),
        "a\nbdoc",
        "W09-02: bracketed paste must preserve the newline"
    );
    assert!(rig.app.row(0).contains('a'), "W09-02: first line paints");
    assert!(
        rig.app.row(1).contains("bdoc"),
        "W09-02: second line paints, got {:?}",
        rig.app.row(1)
    );
}

/// W09-03 green half: PageUp/PageDown and Home/End move the caret while
/// editing. The navigation half is the
/// `w09_area_nav_page_home_end_scroll_without_editing` parity record.
#[test]
fn w09_area_editing_page_home_end_move_caret() {
    let mut rig = AreaRig::new(AREA_DOC);
    assert!(rig.app.tab_to(AREA));
    let _ = rig.app.click(2, 0);
    assert_eq!(rig.app.cursor(), Some(Position::new(2, 0)));

    let _ = rig.app.key(KeyCode::PageDown);
    // Caret line 3 with the view scrolled one row (L1 on top).
    assert_eq!(
        rig.app.cursor(),
        Some(Position::new(2, 2)),
        "W09-03: editing PageDown must move the caret by the 3-row viewport"
    );
    assert!(
        rig.app.row(0).contains("L1"),
        "W09-03: the view must follow the paged caret, got {:?}",
        rig.app.row(0)
    );
    let _ = rig.app.key(KeyCode::End);
    assert_eq!(
        rig.app.cursor(),
        Some(Position::new(4, 2)),
        "W09-03: editing End must reach the line end"
    );
    let _ = rig.app.key(KeyCode::Home);
    assert_eq!(
        rig.app.cursor(),
        Some(Position::new(2, 2)),
        "W09-03: editing Home must reach the line start"
    );
    let _ = rig.app.key(KeyCode::PageUp);
    assert_eq!(
        rig.app.cursor(),
        Some(Position::new(2, 0)),
        "W09-03: editing PageUp must move the caret back by the viewport"
    );
    assert!(
        rig.app.state_of(AREA).contains(StateFlags::EDITING),
        "W09-03: caret motion must stay in edit mode"
    );
}

/// W09-03 parity record: the legacy `TextArea::on_key` scrolls the *view*
/// on nav PageUp/PageDown/Home/End (`page_up`, `jump_start`, ...) without
/// entering edit mode. The candidate binding arm begins the draft for every
/// command but `Commit`, so the same keys enter edit mode and move the
/// caret instead. (The same missing navigation mode also swallows nav
/// Up/Down and nav typing, which the legacy scrolls or ignores.)
#[test]
#[ignore = "PARITY W09-03: nav PageUp/Home/End begin editing and move the caret, reference scrolls the view"]
fn w09_area_nav_page_home_end_scroll_without_editing() {
    let mut rig = AreaRig::new(AREA_DOC);
    assert!(rig.app.tab_to(AREA));
    let _ = rig.app.key(KeyCode::PageDown);
    assert!(
        !rig.app.state_of(AREA).contains(StateFlags::EDITING),
        "W09-03: nav PageDown must not begin editing"
    );
    assert_eq!(
        rig.app.cursor(),
        None,
        "W09-03: no caret is requested outside edit mode"
    );
    assert!(
        rig.app.row(0).contains("L3"),
        "W09-03: nav PageDown must scroll the view by a page, got {:?}",
        rig.app.row(0)
    );
    let _ = rig.app.key(KeyCode::Home);
    assert!(
        rig.app.row(0).contains("L0"),
        "W09-03: nav Home must jump to the start, got {:?}",
        rig.app.row(0)
    );
    let _ = rig.app.key(KeyCode::End);
    assert!(
        rig.app.row(0).contains("L5"),
        "W09-03: nav End must jump to the end, got {:?}",
        rig.app.row(0)
    );
    assert!(
        !rig.app.state_of(AREA).contains(StateFlags::EDITING),
        "W09-03: nav Home/End must not begin editing"
    );
}

/// W09-04: the selection run spans a newline, covers a tab, and moves by
/// whole graphemes.
#[test]
fn w09_area_selection_spans_newline_tab_grapheme() {
    // Across the newline: both lines carry the selection style. The span
    // covers all of line 0, so the unselected baseline sits on line 1 past
    // the caret.
    let mut rig = AreaRig::new("aaXX\nbbYYYY");
    assert!(rig.app.tab_to(AREA));
    let _ = rig.app.click(2, 0);
    let _ = rig.app.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    let _ = rig.app.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
    let _ = rig.app.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
    assert_eq!(
        rig.app.cursor(),
        Some(Position::new(4, 1)),
        "W09-04: Shift+Down/Right must extend across the newline"
    );
    let line0_sel = cell_style(&rig.app, 2, 0);
    let line1_sel = cell_style(&rig.app, 2, 1);
    let unselected = cell_style(&rig.app, 6, 1);
    assert_eq!(
        line0_sel, line1_sel,
        "W09-04: both lines of the selection share one style"
    );
    assert_ne!(
        line0_sel, unselected,
        "W09-04: selected cells must differ from unselected text"
    );

    // Over a tab: the tab cell shares the selection style.
    let mut rig = AreaRig::new("a\tb");
    assert!(rig.app.tab_to(AREA));
    let _ = rig.app.click(2, 0);
    let _ = rig.app.key_mod(KeyCode::End, KeyModifiers::SHIFT);
    assert_eq!(
        cell_style(&rig.app, 2, 0),
        cell_style(&rig.app, 3, 0),
        "W09-04: the tab cell must share the selection style"
    );

    // Over a wide grapheme: the move is atomic and the grapheme's lead
    // cell carries the selection style. (Wide continuation cells render
    // Reset universally — selected or not, see the unselected nav paint —
    // so the matrix-level observable is the lead cell plus the caret.)
    let mut rig = AreaRig::new("a界b");
    assert!(rig.app.tab_to(AREA));
    let _ = rig.app.click(3, 0);
    let _ = rig.app.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
    assert_eq!(
        rig.app.cursor(),
        Some(Position::new(5, 0)),
        "W09-04: Shift+Right over 界 must move by the whole grapheme"
    );
    assert_ne!(
        cell_style(&rig.app, 3, 0),
        cell_style(&rig.app, 5, 0),
        "W09-04: the selected grapheme's lead cell must differ from unselected text"
    );
}

/// W09-05 green half: a wheel that keeps the caret in view holds, and caret
/// motion keeps following. The hold past the caret is the
/// `w09_area_wheel_holds_past_caret_until_it_moves` parity record.
#[test]
fn w09_area_wheel_holds_and_caret_motion_follows() {
    let mut rig = AreaRig::new(AREA_DOC);
    assert!(rig.app.tab_to(AREA));
    let _ = rig.app.click(2, 1);
    assert_eq!(rig.app.cursor(), Some(Position::new(2, 1)));
    let _ = rig.app.mouse(MouseKind::Wheel(Axis::V, 1), 5, 1);
    assert!(
        rig.app.row(0).contains("L1"),
        "W09-05: wheel +1 must hold while the caret stays visible, got {:?}",
        rig.app.row(0)
    );
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.key(KeyCode::Down);
    // Caret line 4 with the view followed one more row (L2 on top).
    assert_eq!(
        rig.app.cursor(),
        Some(Position::new(2, 2)),
        "W09-05: caret motion must track Down presses"
    );
    assert!(
        rig.app.row(0).contains("L2"),
        "W09-05: follow-caret must resume once the caret leaves, got {:?}",
        rig.app.row(0)
    );
    assert!(
        rig.app.state_of(AREA).contains(StateFlags::EDITING),
        "W09-05: the wheel must not resolve the draft"
    );
}

/// W09-05 parity record: the legacy wheel clears `follow_cursor`, so the
/// offset holds with the caret out of view until caret motion resumes the
/// follow. The candidate runs `ensure_visible` on every editing update, so
/// a wheel that leaves the caret behind snaps straight back.
#[test]
#[ignore = "PARITY W09-05: wheel-while-editing snaps back when the caret leaves view (ensure_visible every update), reference holds until caret moves"]
fn w09_area_wheel_holds_past_caret_until_it_moves() {
    let mut rig = AreaRig::new(AREA_DOC);
    assert!(rig.app.tab_to(AREA));
    let _ = rig.app.click(2, 0);
    let _ = rig.app.mouse(MouseKind::Wheel(Axis::V, 5), 5, 1);
    assert!(
        rig.app.state_of(AREA).contains(StateFlags::EDITING),
        "W09-05: the wheel must not resolve the draft"
    );
    assert!(
        rig.app.row(0).contains("L5"),
        "W09-05: the wheel offset must hold past the caret, got {:?}",
        rig.app.row(0)
    );
    // And then caret motion resumes the follow.
    let _ = rig.app.key(KeyCode::Down);
    assert!(
        rig.app.row(0).contains("L0") || rig.app.row(1).contains("L0"),
        "W09-05: caret motion must bring the caret line back, got {:?}/{:?}",
        rig.app.row(0),
        rig.app.row(1)
    );
}

/// W09-06: a Field around a 3-row area uses rows+2; height 0/1 stays
/// contained and panic-free.
#[test]
fn w09_area_field_layout_rows_plus_two_and_tiny_heights() {
    struct FieldApp {
        value: Rc<RefCell<String>>,
        st: TextAreaState,
        area_h: u16,
        used: Rc<Cell<Rect>>,
    }

    impl App for FieldApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let mut borrowed = self.value.borrow_mut();
            let _ = TextArea::new(AREA_FIELD, AREA_ROWS).update(cx, &mut self.st, &mut borrowed);
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let borrowed = self.value.borrow();
            let field = Field::new("Notes", TextArea::new(AREA_FIELD, AREA_ROWS).value(&borrowed))
                .help("help me");
            let area = Rect {
                x: 0,
                y: 0,
                width: 40,
                height: self.area_h,
            };
            self.used.set(field.draw(ui, area, &self.st));
        }
    }

    fn used_height(area_h: u16) -> Rect {
        let used = Rc::new(Cell::new(Rect::default()));
        let mut app = Harness::new(
            FieldApp {
                value: Rc::new(RefCell::new("x".to_string())),
                st: TextAreaState::default(),
                area_h,
                used: Rc::clone(&used),
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        app.draw();
        let _ = app;
        used.get()
    }

    // Label row + 3 body rows + help row.
    assert_eq!(
        used_height(8).height,
        5,
        "W09-06: Field rows+2 layout must use 5 rows"
    );
    // Tight frames degrade to the label row, then to nothing.
    assert_eq!(used_height(1).height, 1, "W09-06: height 1 keeps the label");
    assert_eq!(used_height(0).height, 0, "W09-06: height 0 uses nothing");

    // Standalone tiny draws stay contained with a contained cursor.
    struct TinyApp {
        value: Rc<RefCell<String>>,
        st: TextAreaState,
        used: Rc<Cell<(Rect, Rect)>>,
    }

    impl App for TinyApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let mut borrowed = self.value.borrow_mut();
            let _ = TextArea::new(AREA_TINY, AREA_ROWS).update(cx, &mut self.st, &mut borrowed);
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let borrowed = self.value.borrow();
            let control = TextArea::new(AREA_TINY, AREA_ROWS).value(&borrowed);
            let zero = control.draw(ui, Rect::new(0, 10, 40, 0), &self.st);
            let one = control.draw(ui, Rect::new(0, 12, 40, 1), &self.st);
            self.used.set((zero, one));
        }
    }

    let used = Rc::new(Cell::new((Rect::default(), Rect::default())));
    let mut app = Harness::new(
        TinyApp {
            value: Rc::new(RefCell::new("x".to_string())),
            st: TextAreaState::default(),
            used: Rc::clone(&used),
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    app.draw();
    let (zero, one) = used.get();
    assert_eq!(zero.height, 0, "W09-06: standalone height 0 uses nothing");
    assert_eq!(one.height, 1, "W09-06: standalone height 1 uses one row");
    assert!(
        app.cursor().is_none_or(|p| p.y == 12),
        "W09-06: tiny-frame cursor must stay contained, got {:?}",
        app.cursor()
    );
}

// ---------------------------------------------------------------------------
// W10 Select (termrock-overlays)
// ---------------------------------------------------------------------------

const SELECT: Id = Id::root("control.states.select");
const SELECT_SENTINEL: Id = Id::root("control.states.select.sentinel");
const SELECT_PLACEHOLDER: &str = "Pick one";
const SELECT_W: u16 = 20;

fn str_key(s: &String) -> ItemKey {
    ItemKey::text(s)
}

/// Shared select rig: caller-owned options plus value/cursor/open mirrors.
/// Options are keyed by label text (stable under reorder).
struct SelectApp {
    items: Rc<RefCell<Vec<String>>>,
    st: SelectState,
    actions: Rc<RefCell<Vec<SelectAction>>>,
    value: Rc<Cell<Option<ItemKey>>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    open: Rc<Cell<bool>>,
    field_y: u16,
    popup_rows: Option<u16>,
    disabled: bool,
}

impl SelectApp {
    fn select(
        disabled: bool,
        popup_rows: Option<u16>,
    ) -> Select<'static, String, fn(&String) -> ItemKey, DefaultRow> {
        let sel = Select::new(SELECT)
            .key(str_key as fn(&String) -> ItemKey)
            .placeholder(SELECT_PLACEHOLDER)
            .disabled(disabled);
        if let Some(n) = popup_rows {
            sel.popup_rows(n)
        } else {
            sel
        }
    }
}

impl App for SelectApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let actions = Rc::clone(&self.actions);
        let borrowed = self.items.borrow();
        let r = Self::select(self.disabled, self.popup_rows)
            .update(cx, &mut self.st, &borrowed)
            .on_action(|action| actions.borrow_mut().push(action));
        self.value.set(self.st.value());
        self.cursor.set(self.st.cursor());
        self.open.set(self.st.is_open());
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let borrowed = self.items.borrow();
        Self::select(self.disabled, self.popup_rows).draw(
            ui,
            Rect {
                x: 0,
                y: self.field_y,
                width: SELECT_W,
                height: 1,
            },
            &self.st,
            &borrowed,
        );
        ui.register_control(
            SELECT_SENTINEL,
            Rect::new(30, 0, 8, 1),
            Focusability::Focusable,
        );
    }
}

struct SelectRig {
    app: Harness<SelectApp>,
    items: Rc<RefCell<Vec<String>>>,
    actions: Rc<RefCell<Vec<SelectAction>>>,
    value: Rc<Cell<Option<ItemKey>>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    open: Rc<Cell<bool>>,
    field_y: u16,
}

impl SelectRig {
    fn with(
        items: &[&str],
        value: Option<&str>,
        field_y: u16,
        popup_rows: Option<u16>,
        disabled: bool,
    ) -> Self {
        let items_rc = Rc::new(RefCell::new(
            items.iter().map(|s| s.to_string()).collect(),
        ));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let value_rc = Rc::new(Cell::new(None));
        let cursor = Rc::new(Cell::new(None));
        let open = Rc::new(Cell::new(false));
        let mut st = SelectState::default();
        st.set_value(value.map(ItemKey::text));
        let app = Harness::new(
            SelectApp {
                items: Rc::clone(&items_rc),
                st,
                actions: Rc::clone(&actions),
                value: Rc::clone(&value_rc),
                cursor: Rc::clone(&cursor),
                open: Rc::clone(&open),
                field_y,
                popup_rows,
                disabled,
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        Self {
            app,
            items: items_rc,
            actions,
            value: value_rc,
            cursor,
            open,
            field_y,
        }
    }

    fn opened(&self) -> usize {
        self.actions
            .borrow()
            .iter()
            .filter(|a| **a == SelectAction::Opened)
            .count()
    }

    fn closed(&self) -> usize {
        self.actions
            .borrow()
            .iter()
            .filter(|a| **a == SelectAction::Closed)
            .count()
    }

    fn chose(&self) -> Vec<ItemKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                SelectAction::Chose(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    fn layer(&self) -> Rect {
        self.app
            .layer_area(SELECT)
            .expect("W10: the popup layer must have an area")
    }
}

fn sel_cell(app: &Harness<SelectApp>, x: u16, y: u16) -> (String, String, String) {
    let c = &app.buffer()[Position::new(x, y)];
    (
        format!("{:?}", c.fg),
        format!("{:?}", c.bg),
        format!("{:?}", c.modifier),
    )
}

/// W10-01: closed paint shows the value or the placeholder; focus and
/// disabled behave; the marker discloses open state.
#[test]
fn w10_select_closed_value_placeholder_focus_disabled() {
    // A value paints in the closed field with SELECTED.
    let mut rig = SelectRig::with(&["alpha", "bravo"], Some("bravo"), 2, None, false);
    assert!(rig.app.tab_to(SELECT));
    assert!(
        rig.app.row(2).contains("bravo"),
        "W10-01: closed field must show the value, got {:?}",
        rig.app.row(2)
    );
    assert!(
        rig.app.state_of(SELECT).contains(StateFlags::FOCUSED),
        "W10-01: tab must focus the field"
    );
    assert!(
        rig.app.row(2).contains("▾"),
        "W10-01: the closed marker must disclose, got {:?}",
        rig.app.row(2)
    );

    // No value paints the placeholder in its own muted voice.
    let mut rig = SelectRig::with(&["alpha", "bravo"], None, 2, None, false);
    assert!(rig.app.tab_to(SELECT));
    assert!(
        rig.app.row(2).contains(SELECT_PLACEHOLDER),
        "W10-01: an empty value must show the placeholder, got {:?}",
        rig.app.row(2)
    );
    // Valued and placeholder labels paint in different voices.
    let valued = SelectRig::with(&["alpha", "bravo"], Some("bravo"), 2, None, false);
    assert_ne!(
        sel_cell(&valued.app, 2, 2),
        sel_cell(&rig.app, 2, 2),
        "W10-01: valued and placeholder labels must paint differently"
    );

    // Disabled: keys and clicks are inert, the popup never opens.
    let mut rig = SelectRig::with(&["alpha", "bravo"], Some("alpha"), 2, None, true);
    assert!(
        rig.app.state_of(SELECT).contains(StateFlags::DISABLED),
        "W10-01: disabled must wear DISABLED"
    );
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.click(5, 2);
    assert!(
        rig.actions.borrow().is_empty(),
        "W10-01: disabled keys and clicks must emit nothing"
    );
    assert!(!rig.open.get(), "W10-01: disabled must not open");
    assert!(!rig.app.is_open(SELECT), "W10-01: no disabled layer");
}

/// W10-02: the popup flips above the field at the bottom edge, opens below
/// at the top edge, and re-resolves after a resize.
#[test]
fn w10_select_edge_flip_and_resize() {
    // Bottom edge: the 5-row popup cannot fit below row 7, so it flips up.
    let mut rig = SelectRig::with(&["a", "b", "c"], None, 7, None, false);
    assert!(rig.app.tab_to(SELECT));
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(rig.opened(), 1, "W10-02: Enter must open");
    let layer = rig.layer();
    assert!(
        layer.bottom() <= 7,
        "W10-02: bottom-edge popup must flip above, got {layer:?}"
    );
    assert!(
        rig.app.row(layer.y + 1).contains('a'),
        "W10-02: flipped popup must show options, got {:?}",
        rig.app.row(layer.y + 1)
    );

    // Top edge: the popup opens below.
    let mut rig = SelectRig::with(&["a", "b", "c"], None, 0, None, false);
    assert!(rig.app.tab_to(SELECT));
    let _ = rig.app.key(KeyCode::Enter);
    let layer = rig.layer();
    assert_eq!(layer.y, 1, "W10-02: top-edge popup must open below");
    assert!(
        rig.app.row(2).contains('a') && rig.app.row(3).contains('b'),
        "W10-02: below popup must show options, got {:?} / {:?}",
        rig.app.row(2),
        rig.app.row(3)
    );

    // Resize re-resolves the open popup inside the new screen.
    let mut rig = SelectRig::with(&["only"], None, 4, None, false);
    assert!(rig.app.tab_to(SELECT));
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(rig.layer().y, 5, "W10-02: one option opens below");
    let _ = rig.app.resize(40, 6);
    assert!(rig.open.get(), "W10-02: resize must keep the popup open");
    let layer = rig.layer();
    assert!(
        layer.bottom() <= 6,
        "W10-02: resized popup must stay on screen, got {layer:?}"
    );
    assert!(
        layer.bottom() <= 4 || layer.y >= 5,
        "W10-02: resized popup must stay attached to the field, got {layer:?}"
    );
    assert!(
        (0..6).any(|y| rig.app.row(y).contains("only")),
        "W10-02: resized popup must still show its option"
    );
}

/// W10-03: the highlight is not the value; Escape preserves the value and
/// restores the cursor.
#[test]
fn w10_select_highlight_is_not_value_escape_preserves() {
    let mut rig = SelectRig::with(&["alpha", "bravo", "charlie"], Some("alpha"), 0, None, false);
    assert!(rig.app.tab_to(SELECT));
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("alpha")),
        "W10-03: the cursor must seed on the value"
    );
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("bravo")),
        "W10-03: Down must move the highlight"
    );
    // The closed field still commits to alpha while bravo highlights.
    assert!(
        rig.app.row(0).contains("alpha"),
        "W10-03: moving the highlight must not move the value, got {:?}",
        rig.app.row(0)
    );
    assert_eq!(rig.value.get(), Some(ItemKey::text("alpha")));
    // Highlight and value rows paint differently; the value keeps its mark.
    let layer = rig.layer();
    let value_mark = rig.app.buffer()[Position::new(layer.x + 1, layer.y + 1)].symbol();
    let cursor_mark = rig.app.buffer()[Position::new(layer.x + 1, layer.y + 2)].symbol();
    assert_ne!(
        value_mark, cursor_mark,
        "W10-03: only the value row must carry the chosen mark"
    );
    assert_ne!(
        sel_cell(&rig.app, layer.x + 4, layer.y + 1),
        sel_cell(&rig.app, layer.x + 4, layer.y + 2),
        "W10-03: highlight and value rows must paint differently"
    );
    // Escape preserves the value and restores the cursor onto it.
    let _ = rig.app.key(KeyCode::Esc);
    assert_eq!(rig.closed(), 1, "W10-03: Escape must close once");
    assert!(rig.chose().is_empty(), "W10-03: Escape must choose nothing");
    assert_eq!(rig.value.get(), Some(ItemKey::text("alpha")));
    assert!(
        rig.app.row(0).contains("alpha"),
        "W10-03: Escape must preserve the chosen value"
    );
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(
        rig.chose(),
        vec![ItemKey::text("alpha")],
        "W10-03: reopen+Enter must commit the restored cursor"
    );
}

/// W10-04: reorder and delete while open; clicks resolve by stable key.
#[test]
fn w10_select_reorder_delete_click_by_stable_key() {
    let mut rig = SelectRig::with(&["alpha", "bravo", "charlie"], Some("alpha"), 0, None, false);
    assert!(rig.app.tab_to(SELECT));
    let _ = rig.app.key(KeyCode::Enter);
    // Reorder under the open popup; the bravo row moves but keeps its key.
    rig.items.replace(vec![
        "charlie".to_string(),
        "alpha".to_string(),
        "bravo".to_string(),
    ]);
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.key(KeyCode::Up);
    let _ = rig.app.click_part(SELECT, PartRef::item(Part::ROW, ItemKey::text("bravo")));
    assert_eq!(
        rig.chose(),
        vec![ItemKey::text("bravo")],
        "W10-04: the click must resolve by stable key, not position"
    );
    assert!(
        rig.app.row(0).contains("bravo"),
        "W10-04: the field must show the keyed choice, got {:?}",
        rig.app.row(0)
    );

    // Deleting the value option while open clears the value, cleanly.
    let mut rig = SelectRig::with(&["alpha", "bravo", "charlie"], Some("bravo"), 0, None, false);
    assert!(rig.app.tab_to(SELECT));
    let _ = rig.app.key(KeyCode::Enter);
    rig.items.replace(vec!["alpha".to_string(), "charlie".to_string()]);
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(
        rig.value.get(),
        None,
        "W10-04: deleting the value option must clear the value"
    );
    assert!(
        rig.app.row(0).contains(SELECT_PLACEHOLDER),
        "W10-04: the cleared field must fall back to the placeholder, got {:?}",
        rig.app.row(0)
    );
    let _ = rig.app.click_part(SELECT, PartRef::item(Part::ROW, ItemKey::text("charlie")));
    assert_eq!(
        rig.chose(),
        vec![ItemKey::text("charlie")],
        "W10-04: choosing must keep working after the delete"
    );

    // Deleting a bystander preserves the value.
    let mut rig = SelectRig::with(&["alpha", "bravo", "charlie"], Some("bravo"), 0, None, false);
    assert!(rig.app.tab_to(SELECT));
    let _ = rig.app.key(KeyCode::Enter);
    rig.items.replace(vec!["bravo".to_string(), "charlie".to_string()]);
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.key(KeyCode::Up);
    assert_eq!(
        rig.value.get(),
        Some(ItemKey::text("bravo")),
        "W10-04: deleting a bystander must preserve the value"
    );
    assert!(rig.open.get(), "W10-04: the popup must survive the delete");
}

/// W10-05 green half: empty options paint the empty state; a disabled
/// control never pops up; long lists scroll under a scrollbar. Edge fade is
/// the `w10_select_popup_edge_fade` parity record.
#[test]
fn w10_select_empty_disabled_and_scrollbar() {
    // Empty: the popup paints "No options" and commits nothing.
    let mut rig = SelectRig::with(&[], None, 0, None, false);
    assert!(rig.app.tab_to(SELECT));
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(rig.opened(), 1, "W10-05: an empty select still opens");
    let layer = rig.layer();
    assert!(
        (layer.y..layer.bottom()).any(|y| rig.app.row(y).contains("No options")),
        "W10-05: the empty popup must name the state"
    );
    let _ = rig.app.key(KeyCode::Enter);
    assert!(
        rig.chose().is_empty(),
        "W10-05: Enter on empty must choose nothing"
    );
    let _ = rig.app.key(KeyCode::Esc);
    assert_eq!(rig.closed(), 1, "W10-05: Escape must close the empty popup");

    // Disabled with options: nothing opens, ever.
    let mut rig = SelectRig::with(&["a", "b"], None, 0, None, true);
    let _ = rig.app.key(KeyCode::Enter);
    let _ = rig.app.click(5, 0);
    assert!(
        rig.actions.borrow().is_empty(),
        "W10-05: a disabled select must stay shut"
    );
    assert!(!rig.app.is_open(SELECT));

    // Twenty options in three popup rows: scrollbar, scroll, thumb travel.
    let options: Vec<String> = (0..20).map(|i| format!("opt{i:02}")).collect();
    let labels: Vec<&str> = options.iter().map(String::as_str).collect();
    let mut rig = SelectRig::with(&labels, None, 0, Some(3), false);
    assert!(rig.app.tab_to(SELECT));
    let _ = rig.app.key(KeyCode::Enter);
    let layer = rig.layer();
    assert_eq!(layer.height, 5, "W10-05: 3 option rows plus two pad rows");
    let bar_x = layer.right() - 1;
    let bar: String = (layer.y + 1..layer.y + 4)
        .map(|y| rig.app.buffer()[Position::new(bar_x, y)].symbol().to_string())
        .collect();
    assert!(
        bar.contains("┃"),
        "W10-05: the popup must paint a scrollbar thumb, got {bar:?}"
    );
    assert!(
        rig.app.row(layer.y + 1).contains("opt00"),
        "W10-05: the list must start at the top"
    );
    let thumb_top: Vec<u16> = (layer.y + 1..layer.y + 4)
        .filter(|&y| rig.app.buffer()[Position::new(bar_x, y)].symbol() == "┃")
        .collect();
    let _ = rig.app.key(KeyCode::End);
    assert!(
        rig.app.row(layer.y + 1).contains("opt17"),
        "W10-05: End must scroll to the tail, got {:?}",
        rig.app.row(layer.y + 1)
    );
    let thumb_bottom: Vec<u16> = (layer.y + 1..layer.y + 4)
        .filter(|&y| rig.app.buffer()[Position::new(bar_x, y)].symbol() == "┃")
        .collect();
    assert_ne!(
        thumb_top, thumb_bottom,
        "W10-05: the thumb must travel with the view"
    );
}

/// W10-05 parity record: the matrix requires popup edge fade, and the
/// candidate fades scrolled edges for text areas, grids and nav lists —
/// but `Select::draw_popup` never calls `ui.scroll_edges`, so a select
/// popup with hidden rows below paints its bottom option row exactly like
/// a settled one. (The legacy select has no oracle here: it cuts the popup
/// at ten rows instead of scrolling, so the matrix is the authority.)
#[test]
#[ignore = "PARITY W10-05: select popup never fades its scrolled edges (no scroll_edges call), reference requires edge fade"]
fn w10_select_popup_edge_fade() {
    let options: Vec<String> = (0..20).map(|i| format!("opt{i:02}")).collect();
    let labels: Vec<&str> = options.iter().map(String::as_str).collect();
    let mut rig = SelectRig::with(&labels, None, 0, Some(4), false);
    assert!(rig.app.tab_to(SELECT));
    let _ = rig.app.key(KeyCode::Enter);
    let layer = rig.layer();
    // Sixteen options hide below: the bottom option row must fade toward
    // them while a settled middle row stays crisp.
    let bottom = sel_cell(&rig.app, layer.x + 4, layer.y + 4);
    let middle = sel_cell(&rig.app, layer.x + 4, layer.y + 2);
    assert_ne!(
        bottom, middle,
        "W10-05: the bottom option row must fade toward hidden content"
    );
}

// ---------------------------------------------------------------------------
// W11 Form (termrock-forms)
// ---------------------------------------------------------------------------

const FORM: Id = Id::root("control.states.form");
const F1: Id = Id::root("control.states.form.f1");
const F2: Id = Id::root("control.states.form.f2");
const F3: Id = Id::root("control.states.form.f3");
const F_CHECK: Id = Id::root("control.states.form.check");
const F_NAME: Id = Id::root("control.states.form.name");
const F_PICK: Id = Id::root("control.states.form.pick");
const PICKER: Id = Id::root("control.states.form.picker");
const VALIDATE: ActionKey = ActionKey::application("validate");
const FORM_W: u16 = 80;
const FORM_H: u16 = 24;

struct FormModel {
    values: HashMap<Id, String>,
    flags: HashMap<Id, bool>,
    display: String,
    hidden: HashSet<Id>,
    disabled: HashSet<Id>,
    required: HashSet<Id>,
}

impl FormModel {
    fn new() -> Self {
        FormModel {
            values: HashMap::new(),
            flags: HashMap::new(),
            display: String::new(),
            hidden: HashSet::new(),
            disabled: HashSet::new(),
            required: HashSet::new(),
        }
    }
}

impl FormData for FormModel {
    fn value(&self, id: Id) -> FieldRef<'_> {
        if id == F_PICK {
            return FieldRef::Display {
                value: &self.display,
                detail: None,
            };
        }
        if id == F_CHECK {
            return FieldRef::Flag(self.flags.get(&id).copied().unwrap_or(false));
        }
        FieldRef::Text(self.values.get(&id).map_or("", String::as_str))
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        if id == F_PICK {
            return FieldMut::ReadOnly;
        }
        if id == F_CHECK {
            return FieldMut::Flag(self.flags.entry(id).or_default());
        }
        FieldMut::Text(self.values.entry(id).or_default())
    }

    fn visible(&self, id: Id) -> bool {
        !self.hidden.contains(&id)
    }

    fn disabled(&self, id: Id) -> bool {
        self.disabled.contains(&id)
    }

    fn validate(&self, id: Id, value: FieldRef<'_>) -> Result<(), FieldError> {
        if !self.required.contains(&id) {
            return Ok(());
        }
        match value {
            FieldRef::Text(s) if !s.is_empty() => Ok(()),
            _ => Err(FieldError::new("required")),
        }
    }
}

struct PickerRig {
    st: PickerState,
    items: Vec<Item<'static>>,
    open: bool,
}

impl PickerRig {
    fn new() -> Self {
        PickerRig {
            st: PickerState::default(),
            items: vec![
                Item::new(ItemKey::text("red"), "red"),
                Item::new(ItemKey::text("green"), "green"),
                Item::new(ItemKey::text("blue"), "blue"),
            ],
            open: false,
        }
    }
}

/// Shared form rig: owner model plus action/picker mirrors.
struct FormApp {
    data: Rc<RefCell<FormModel>>,
    specs: Vec<FieldSpec<'static>>,
    actions: Rc<RefCell<Vec<Action<'static>>>>,
    st: FormState,
    form_actions: Rc<RefCell<Vec<FormAction>>>,
    picker: Option<PickerRig>,
    chose_pick: Rc<Cell<bool>>,
}

impl App for FormApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let form_actions = Rc::clone(&self.form_actions);
        let chose_pick = Rc::clone(&self.chose_pick);
        let r = {
            let mut borrowed = self.data.borrow_mut();
            let actions = self.actions.borrow();
            Form::new(FORM, &self.specs)
                .actions(&actions)
                .update(cx, &mut self.st, &mut *borrowed)
                .on_action(|action| {
                    if action == FormAction::Chose(F_PICK) {
                        chose_pick.set(true);
                    }
                    form_actions.borrow_mut().push(action);
                })
        };
        if let Some(pk) = self.picker.as_mut() {
            if self.chose_pick.take() && !pk.open {
                pk.open = true;
                let picker = Picker::new(PICKER);
                picker.reconcile(&mut pk.st, &pk.items);
                cx.open_layer(PICKER, picker.layer(cx, &pk.items));
            }
            if pk.open {
                let mut borrowed = self.data.borrow_mut();
                let items = core::mem::take(&mut pk.items);
                let returned = Rc::new(Cell::new(false));
                let returned_in = Rc::clone(&returned);
                let pr = Picker::new(PICKER)
                    .update(cx, &mut pk.st, &items)
                    .on_action(|action| {
                        if let PickerAction::Chosen(k) = action
                            && let Some(item) = items.iter().find(|it| it.key == k)
                        {
                            borrowed.display = item.label.to_string();
                            returned_in.set(true);
                        }
                    });
                pk.items = items;
                let _ = pr;
                if returned.take() {
                    cx.close_layer(PICKER, None);
                    pk.open = false;
                } else if !cx.is_open(PICKER) {
                    pk.open = false;
                }
            }
        }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let borrowed = self.data.borrow();
        let actions = self.actions.borrow();
        Form::new(FORM, &self.specs).actions(&actions).draw(
            ui,
            Rect::new(0, 0, FORM_W, FORM_H),
            &self.st,
            &*borrowed,
        );
        if let Some(pk) = self.picker.as_ref().filter(|pk| pk.open) {
            let _ = ui.layer(PICKER, |ui, area| {
                Picker::new(PICKER).draw(ui, area, &pk.st, &pk.items)
            });
        }
    }
}

struct FormRig {
    app: Harness<FormApp>,
    data: Rc<RefCell<FormModel>>,
    actions: Rc<RefCell<Vec<FormAction>>>,
    app_actions: Rc<RefCell<Vec<Action<'static>>>>,
}

impl FormRig {
    fn new(specs: Vec<FieldSpec<'static>>, actions: Vec<Action<'static>>, with_picker: bool) -> Self {
        let data = Rc::new(RefCell::new(FormModel::new()));
        let form_actions = Rc::new(RefCell::new(Vec::new()));
        let app_actions = Rc::new(RefCell::new(actions));
        let app = Harness::new(
            FormApp {
                data: Rc::clone(&data),
                specs,
                actions: Rc::clone(&app_actions),
                st: FormState::default(),
                form_actions: Rc::clone(&form_actions),
                picker: with_picker.then(PickerRig::new),
                chose_pick: Rc::new(Cell::new(false)),
            },
            Theme::junie(),
            FORM_W,
            FORM_H,
        );
        Self {
            app,
            data,
            actions: form_actions,
            app_actions,
        }
    }

    fn submitted(&self) -> usize {
        self.actions
            .borrow()
            .iter()
            .filter(|a| **a == FormAction::Action(ActionKey::SAVE))
            .count()
    }

    fn invalids(&self) -> Vec<Id> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                FormAction::Invalid(id) => Some(*id),
                _ => None,
            })
            .collect()
    }

    fn auxes(&self) -> Vec<ActionKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                FormAction::Action(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    fn click_action(&mut self, index: usize) {
        let id = FORM.part(Part::ACTIONS).index(index);
        let area = self
            .app
            .area_of(id)
            .expect("W11: the action button must have an area");
        let _ = self
            .app
            .click(area.x + area.width / 2, area.y + area.height / 2);
    }

    fn any_row_contains(&self, needle: &str) -> bool {
        (0..FORM_H).any(|y| self.app.row(y).contains(needle))
    }
}

fn text_field(id: Id, label: &'static str) -> FieldSpec<'static> {
    FieldSpec::new(id, label, FieldKind::Text(TextInput::new(id)))
}

/// W11-01: an invalid submit focuses the first invalid *visible* field.
#[test]
fn w11_form_invalid_submit_focuses_first_visible() {
    let mut rig = FormRig::new(
        vec![
            text_field(F1, "Hidden"),
            text_field(F2, "Second").required(true),
            text_field(F3, "Third").required(true),
        ],
        vec![Action::new(ActionKey::SAVE, "Save")],
        false,
    );
    rig.data.borrow_mut().hidden.insert(F1);
    rig.data.borrow_mut().required.insert(F1);
    rig.data.borrow_mut().required.insert(F2);
    rig.data.borrow_mut().required.insert(F3);
    rig.click_action(0);
    assert_eq!(
        rig.invalids(),
        vec![F2],
        "W11-01: only the first invalid visible field must report"
    );
    assert_eq!(rig.submitted(), 0, "W11-01: invalid must not submit");
    assert!(
        rig.app.state_of(F2).contains(StateFlags::FOCUSED),
        "W11-01: focus must land on the first invalid visible field"
    );
    assert!(
        rig.any_row_contains("required"),
        "W11-01: the invalid field must paint its error"
    );
    assert!(
        rig.app.area_of(F1).is_none(),
        "W11-01: the hidden invalid field must own no geometry"
    );
}

/// W11-02: hiding then disabling after focus reconciles deterministically:
/// focus tracks the nearest survivor, geometry follows visibility, and the
/// hidden draft is preserved, not reset.
#[test]
fn w11_form_hide_disable_reconcile_deterministically() {
    let mut rig = FormRig::new(
        vec![
            text_field(F1, "First"),
            text_field(F2, "Second"),
            text_field(F3, "Third"),
        ],
        vec![Action::new(ActionKey::SAVE, "Save")],
        false,
    );
    assert!(rig.app.tab_to(F2));
    let _ = rig.app.type_str("zz");
    assert!(
        rig.app.state_of(F2).contains(StateFlags::EDITING),
        "W11-02: the focused field must hold its draft"
    );

    // Hiding the focused field moves focus forward and keeps the draft.
    rig.data.borrow_mut().hidden.insert(F2);
    rig.app.ticks(1);
    rig.app.draw();
    assert!(
        rig.app.state_of(F3).contains(StateFlags::FOCUSED),
        "W11-02: hiding focus must advance to the next survivor"
    );
    assert!(
        rig.app.area_of(F2).is_none(),
        "W11-02: the hidden field must own no geometry"
    );
    assert_eq!(
        rig.data.borrow().values.get(&F2).map(String::as_str),
        Some(""),
        "W11-02: the hidden draft must not commit behind our back"
    );

    // Showing it again restores the exact draft, focus stays put.
    rig.data.borrow_mut().hidden.remove(&F2);
    rig.app.ticks(1);
    rig.app.draw();
    assert!(
        rig.app.state_of(F3).contains(StateFlags::FOCUSED),
        "W11-02: showing a field must not steal focus back"
    );
    assert!(
        rig.app.row(4).contains("zz"),
        "W11-02: the restored field must repaint its draft, got {:?}",
        rig.app.row(4)
    );

    // Disabling the focused field moves focus to the next survivor — the
    // Save action, which follows the fields in the ring. The disabled field
    // keeps geometry but refuses focus and edits.
    rig.data.borrow_mut().disabled.insert(F3);
    rig.app.ticks(1);
    rig.app.draw();
    let save = FORM.part(Part::ACTIONS).index(0);
    assert!(
        rig.app.state_of(save).contains(StateFlags::FOCUSED),
        "W11-02: disabling focus must advance to the next survivor (Save)"
    );
    assert!(
        rig.app.area_of(F3).is_some(),
        "W11-02: a disabled field must keep its geometry"
    );
    let f3 = rig.app.area_of(F3).expect("W11-02: F3 area");
    let _ = rig.app.click(f3.x + f3.width / 2, f3.y);
    assert!(
        rig.app.state_of(save).contains(StateFlags::FOCUSED),
        "W11-02: clicking disabled must not move focus"
    );

    // Hiding a bystander never disturbs focus.
    rig.data.borrow_mut().hidden.insert(F3);
    rig.app.ticks(1);
    rig.app.draw();
    assert!(
        rig.app.state_of(save).contains(StateFlags::FOCUSED),
        "W11-02: hiding a bystander must not move focus"
    );
}

/// W11-03: a nested picker returns or cancels to the exact parent draft and
/// focus; the sibling draft survives either way.
#[test]
fn w11_form_nested_picker_preserves_sibling_draft() {
    let mut rig = FormRig::new(
        vec![
            FieldSpec::new(
                F_NAME,
                "Name",
                FieldKind::Text(TextInput::new(F_NAME).blur(BlurPolicy::Keep)),
            ),
            FieldSpec::new(
                F_PICK,
                "Colour",
                FieldKind::Chooser(Button::new(F_PICK, "Choose")),
            ),
        ],
        vec![Action::new(ActionKey::SAVE, "Save")],
        true,
    );
    assert!(rig.app.tab_to(F_NAME));
    let _ = rig.app.type_str("draft");
    assert!(rig.app.tab_to(F_PICK));
    assert!(
        rig.app.state_of(F_NAME).contains(StateFlags::EDITING),
        "W11-03: the sibling draft must survive the focus move"
    );
    assert_eq!(
        rig.data.borrow().values.get(&F_NAME).map(String::as_str),
        Some(""),
        "W11-03: the kept draft must stay uncommitted"
    );

    // Return path: choose red, land back on the chooser with the draft.
    let _ = rig.app.key(KeyCode::Char(' '));
    assert!(
        rig.app.is_open(PICKER),
        "W11-03: Space must open the nested picker"
    );
    assert!(
        rig.app.state_of(F_NAME).contains(StateFlags::EDITING),
        "W11-03: the draft must survive the picker opening"
    );
    let _ = rig.app.key(KeyCode::Enter);
    assert!(
        !rig.app.is_open(PICKER),
        "W11-03: Enter must return from the picker"
    );
    assert_eq!(
        rig.data.borrow().display.as_str(),
        "red",
        "W11-03: the return value must land in owner data"
    );
    assert!(
        rig.app.state_of(F_PICK).contains(StateFlags::FOCUSED),
        "W11-03: return must restore the parent focus"
    );
    assert!(
        rig.app.state_of(F_NAME).contains(StateFlags::EDITING),
        "W11-03: the draft must survive the picker return"
    );
    assert!(
        rig.app.row(1).contains("draft"),
        "W11-03: the draft must repaint after return, got {:?}",
        rig.app.row(1)
    );

    // Cancel path: Escape closes without touching the draft or the value.
    let _ = rig.app.key(KeyCode::Char(' '));
    assert!(rig.app.is_open(PICKER));
    let _ = rig.app.key(KeyCode::Esc);
    assert!(
        !rig.app.is_open(PICKER),
        "W11-03: Escape must cancel the picker"
    );
    assert_eq!(
        rig.data.borrow().display.as_str(),
        "red",
        "W11-03: cancel must keep the previous return value"
    );
    assert!(
        rig.app.state_of(F_PICK).contains(StateFlags::FOCUSED),
        "W11-03: cancel must restore the parent focus"
    );
    assert!(
        rig.app.state_of(F_NAME).contains(StateFlags::EDITING),
        "W11-03: the draft must survive the picker cancel"
    );
    assert!(
        rig.app.row(1).contains("draft"),
        "W11-03: the draft must repaint after cancel, got {:?}",
        rig.app.row(1)
    );
}

/// W11-04 green half: a disabled submit rejects pointer activation, and
/// Cancel fires exactly when eligible. The Enter-while-busy path is the
/// `w11_form_busy_enter_submit_blocked` parity record.
#[test]
fn w11_form_busy_submit_blocked_cancel_eligible() {
    let mut rig = FormRig::new(
        vec![
            text_field(F1, "First"),
            FieldSpec::new(F_CHECK, "Agree", FieldKind::Check(Checkbox::new(F_CHECK, "ok"))),
        ],
        vec![
            Action::new(ActionKey::SAVE, "Save").enabled(false),
            Action::new(ActionKey::CANCEL, "Cancel"),
        ],
        false,
    );
    // The busy submit rejects the pointer.
    rig.click_action(0);
    assert_eq!(rig.submitted(), 0, "W11-04: busy submit must not emit");
    assert!(
        rig.actions.borrow().is_empty(),
        "W11-04: the disabled submit must report nothing"
    );
    // Cancel fires while eligible, then goes quiet when it is not.
    rig.click_action(1);
    assert_eq!(
        rig.auxes(),
        vec![ActionKey::CANCEL],
        "W11-04: eligible Cancel must fire once"
    );
    rig.app_actions.borrow_mut()[1] = Action::new(ActionKey::CANCEL, "Cancel").enabled(false);
    rig.app.ticks(1);
    rig.app.draw();
    rig.click_action(1);
    assert_eq!(
        rig.auxes(),
        vec![ActionKey::CANCEL],
        "W11-04: ineligible Cancel must not fire again"
    );
}

/// W11-04 parity record: with the submit action disabled (the implemented
/// API's only busy expression — there is no `busy` builder), the pointer
/// and chord paths stay shut, but `EnterPolicy::SubmitsWhenIdle` claims
/// Enter on the focused idle field and `submit_form` never consults the
/// submit action's eligibility, so Enter submits while busy. The matrix
/// (`docs/components/form.md` negative tests: "Busy submit cannot emit")
/// blocks every submit path.
#[test]
#[ignore = "PARITY W11-04: Enter submits while the submit action is disabled (enter path ignores eligibility), reference blocks busy submit"]
fn w11_form_busy_enter_submit_blocked() {
    let mut rig = FormRig::new(
        vec![
            text_field(F1, "First"),
            FieldSpec::new(F_CHECK, "Agree", FieldKind::Check(Checkbox::new(F_CHECK, "ok"))),
        ],
        vec![
            Action::new(ActionKey::SAVE, "Save").enabled(false),
            Action::new(ActionKey::CANCEL, "Cancel"),
        ],
        false,
    );
    assert!(rig.app.tab_to(F_CHECK));
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(
        rig.submitted(),
        0,
        "W11-04: Enter while busy must not submit"
    );
    assert!(
        rig.actions.borrow().is_empty(),
        "W11-04: Enter while busy must report nothing"
    );
}

/// W11-05: the auxiliary Validate action reports its own key and neither
/// submits, validates-with-focus, nor saves.
#[test]
fn w11_form_aux_validate_does_not_submit_or_save() {
    static AUX_ACTIONS: &[Action<'static>] = &[
        Action::new(VALIDATE, "Validate"),
        Action::new(ActionKey::SAVE, "Save"),
        Action::new(ActionKey::CANCEL, "Cancel"),
    ];
    let mut rig = FormRig::new(
        vec![text_field(F1, "First").required(true)],
        AUX_ACTIONS.to_vec(),
        false,
    );
    rig.data.borrow_mut().required.insert(F1);
    assert!(rig.app.tab_to(F1));
    rig.click_action(0);
    assert_eq!(
        rig.auxes(),
        vec![VALIDATE],
        "W11-05: Validate must report its own key once"
    );
    assert_eq!(rig.submitted(), 0, "W11-05: Validate must not submit");
    assert!(
        rig.invalids().is_empty(),
        "W11-05: Validate must not run submit validation"
    );
    assert!(
        !rig.any_row_contains("required"),
        "W11-05: Validate must paint no submit error"
    );
    assert_eq!(
        rig.data.borrow().values.get(&F1).map(String::as_str),
        Some(""),
        "W11-05: Validate must save nothing"
    );
}

// ---------------------------------------------------------------------------
// W12 List (termrock-navigation)
// ---------------------------------------------------------------------------

const LIST: Id = Id::root("control.states.list");
const LIST_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 40,
    height: 8,
};

/// Shared list rig: caller-owned string options keyed by label text.
struct ListApp {
    items: Rc<RefCell<Vec<String>>>,
    st: ListState,
    actions: Rc<RefCell<Vec<ListAction>>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    chosen: Rc<Cell<Option<ItemKey>>>,
    status: Status,
    empty: Option<EmptyState<'static>>,
    mode: SelectMode,
    patches: Vec<(Part, StylePatch)>,
    area: Rect,
}

impl ListApp {
    fn list<'p>(
        status: Status,
        empty: Option<EmptyState<'static>>,
        mode: SelectMode,
        patches: &'p [(Part, StylePatch)],
    ) -> List<'p, String, fn(&String) -> ItemKey, DefaultRow> {
        let list = List::new(LIST)
            .key(str_key as fn(&String) -> ItemKey)
            .select_mode(mode)
            .status(status)
            .patch_part(patches);
        if let Some(e) = empty { list.empty(e) } else { list }
    }
}

impl App for ListApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let actions = Rc::clone(&self.actions);
        let borrowed = self.items.borrow();
        let r = Self::list(self.status, self.empty, self.mode, &self.patches)
            .update(cx, &mut self.st, &borrowed)
            .on_action(|action| actions.borrow_mut().push(action));
        self.cursor.set(self.st.cursor());
        self.chosen.set(self.st.chosen());
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let borrowed = self.items.borrow();
        Self::list(self.status, self.empty, self.mode, &self.patches).draw(
            ui,
            self.area,
            &self.st,
            &borrowed,
        );
    }
}

struct ListRig {
    app: Harness<ListApp>,
    items: Rc<RefCell<Vec<String>>>,
    actions: Rc<RefCell<Vec<ListAction>>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    chosen: Rc<Cell<Option<ItemKey>>>,
}

impl ListRig {
    fn with(
        items: &[&str],
        status: Status,
        empty: Option<EmptyState<'static>>,
        mode: SelectMode,
        patches: Vec<(Part, StylePatch)>,
        area: Rect,
    ) -> Self {
        let items_rc = Rc::new(RefCell::new(
            items.iter().map(|s| s.to_string()).collect(),
        ));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let cursor = Rc::new(Cell::new(None));
        let chosen = Rc::new(Cell::new(None));
        let app = Harness::new(
            ListApp {
                items: Rc::clone(&items_rc),
                st: ListState::default(),
                actions: Rc::clone(&actions),
                cursor: Rc::clone(&cursor),
                chosen: Rc::clone(&chosen),
                status,
                empty,
                mode,
                patches,
                area,
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        Self {
            app,
            items: items_rc,
            actions,
            cursor,
            chosen,
        }
    }

    fn plain(items: &[&str]) -> Self {
        Self::with(items, Status::Ready, None, SelectMode::Single, Vec::new(), LIST_AREA)
    }

    fn chose(&self) -> Vec<ItemKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                ListAction::Chose(k) => Some(*k),
                _ => None,
            })
            .collect()
    }
}

fn list_cell(app: &Harness<ListApp>, x: u16, y: u16) -> (String, String, String) {
    let c = &app.buffer()[Position::new(x, y)];
    (
        format!("{:?}", c.fg),
        format!("{:?}", c.bg),
        format!("{:?}", c.modifier),
    )
}

fn list_symbol(app: &Harness<ListApp>, x: u16, y: u16) -> String {
    app.buffer()[Position::new(x, y)].symbol().to_string()
}

/// W12-01: the cursor row, the chosen row and the hovered row each paint
/// their own chrome.
#[test]
fn w12_list_cursor_chosen_hovered_differ() {
    let mut rig = ListRig::plain(&["aaa", "bbb", "ccc", "ddd"]);
    assert!(rig.app.tab_to(LIST));
    // Choose aaa, move the cursor to ccc, hover bbb.
    let _ = rig.app.key(KeyCode::Char(' '));
    assert_eq!(rig.chose(), vec![ItemKey::text("aaa")]);
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("ccc")));
    let _ = rig.app.mouse(MouseKind::Move, 5, 1);
    assert_eq!(rig.app.hover(), Some(LIST), "W12-01: hover must land");

    // Cursor row: the focus bar in the gutter.
    assert_eq!(
        list_symbol(&rig.app, 0, 2),
        "▎",
        "W12-01: the cursor row must wear the focus bar"
    );
    assert_eq!(list_symbol(&rig.app, 0, 0), " ", "W12-01: chosen gutter");
    assert_eq!(list_symbol(&rig.app, 0, 1), " ", "W12-01: hovered gutter");
    // Chosen row: the chosen mark; cursor and hovered rows have none.
    assert_ne!(
        list_symbol(&rig.app, 1, 0),
        " ",
        "W12-01: the chosen row must carry its mark"
    );
    assert_eq!(list_symbol(&rig.app, 1, 1), " ");
    assert_eq!(list_symbol(&rig.app, 1, 2), " ");
    // Cursor padding carries BOLD; hovered padding its own background.
    let (_, _, cursor_mod) = list_cell(&rig.app, 30, 2);
    assert!(
        cursor_mod.contains("BOLD"),
        "W12-01: the cursor row must read bold, got {cursor_mod:?}"
    );
    let (_, hover_bg, _) = list_cell(&rig.app, 30, 1);
    let (_, chosen_bg, _) = list_cell(&rig.app, 30, 0);
    assert_ne!(
        hover_bg, chosen_bg,
        "W12-01: hovered and chosen padding must differ"
    );
}

/// W12-02: empty, loading, partial and error readiness each render
/// distinctly through the shared vocabulary.
#[test]
fn w12_list_readiness_renders_distinctly() {
    // Empty defaults to "Nothing here yet".
    let rig = ListRig::plain(&[]);
    assert!(
        (0..8).any(|y| rig.app.row(y).contains("Nothing here yet")),
        "W12-02: the empty list must name its state"
    );

    // Loading rows keep painting under the spinner rail.
    let mut rig = ListRig::with(
        &["alpha", "beta"],
        Status::Loading,
        None,
        SelectMode::Single,
        Vec::new(),
        LIST_AREA,
    );
    assert!(rig.app.tab_to(LIST));
    assert_eq!(
        list_symbol(&rig.app, 0, 0),
        "⠋",
        "W12-02: loading must raise the spinner, got {:?}",
        rig.app.row(0)
    );
    assert!(
        rig.app.row(0).contains("alpha") && rig.app.row(1).contains("beta"),
        "W12-02: loading rows must still paint"
    );

    // Partial names its hint beside the spinner.
    let rig = ListRig::with(
        &[],
        Status::Ready,
        Some(EmptyState::Partial {
            loaded: 2,
            total: RowTotal::Exact(9),
            hint: "2 of 9",
        }),
        SelectMode::Single,
        Vec::new(),
        LIST_AREA,
    );
    assert!(
        (0..8).any(|y| rig.app.row(y).contains("2 of 9")),
        "W12-02: partial must name its hint"
    );
    assert!(
        (0..8).any(|y| rig.app.row(y).contains("⠋")),
        "W12-02: partial must spin while more loads"
    );

    // Error raises the icon and re-voices the rows.
    let mut rig = ListRig::with(
        &["alpha"],
        Status::Error,
        None,
        SelectMode::Single,
        Vec::new(),
        LIST_AREA,
    );
    assert!(rig.app.tab_to(LIST));
    assert_ne!(
        list_symbol(&rig.app, 0, 0),
        " ",
        "W12-02: error must raise its icon"
    );
    let ready = ListRig::plain(&["alpha"]);
    assert_ne!(
        list_cell(&rig.app, 5, 0),
        list_cell(&ready.app, 5, 0),
        "W12-02: errored rows must re-voice against ready rows"
    );
}

/// W12-03: sort, insert and remove between Down and Up never retarget the
/// click: it resolves by the pressed key.
#[test]
fn w12_list_mutation_between_down_and_up_keeps_key() {
    // Remove above the pressed row: bravo slides 1 -> 0, the Up lands on
    // charlie's new row, but the pressed key wins.
    let mut rig = ListRig::plain(&["alpha", "bravo", "charlie"]);
    assert!(rig.app.tab_to(LIST));
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.mouse(MouseKind::Down, 5, 1);
    rig.items.borrow_mut().remove(0);
    let _ = rig.app.mouse(MouseKind::Up, 5, 1);
    assert_eq!(
        rig.chose(),
        vec![ItemKey::text("bravo")],
        "W12-03: remove-between-presses must keep the pressed key"
    );

    // Insert above the pressed row: bravo slides 1 -> 2.
    let mut rig = ListRig::plain(&["alpha", "bravo", "charlie"]);
    assert!(rig.app.tab_to(LIST));
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.mouse(MouseKind::Down, 5, 1);
    rig.items.borrow_mut().insert(0, "aardvark".to_string());
    let _ = rig.app.mouse(MouseKind::Up, 5, 1);
    assert_eq!(
        rig.chose(),
        vec![ItemKey::text("bravo")],
        "W12-03: insert-between-presses must keep the pressed key"
    );

    // Sort under the pressed row: bravo slides 2 -> 1.
    let mut rig = ListRig::plain(&["charlie", "alpha", "bravo"]);
    assert!(rig.app.tab_to(LIST));
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.mouse(MouseKind::Down, 5, 2);
    rig.items.borrow_mut().sort();
    let _ = rig.app.mouse(MouseKind::Up, 5, 2);
    assert_eq!(
        rig.chose(),
        vec![ItemKey::text("bravo")],
        "W12-03: sort-between-presses must keep the pressed key"
    );
}

/// W12-03 duplicate half: two rows share one label under distinct keys and
/// each resolves exactly.
#[test]
fn w12_list_duplicate_labels_resolve_by_key() {
    struct DupApp {
        items: Rc<RefCell<Vec<(String, u64)>>>,
        st: ListState,
        actions: Rc<RefCell<Vec<ListAction>>>,
    }

    fn dup_key(t: &(String, u64)) -> ItemKey {
        ItemKey::pair(7, t.1)
    }

    impl App for DupApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let actions = Rc::clone(&self.actions);
            let borrowed = self.items.borrow();
            List::new(LIST)
                .key(dup_key as fn(&(String, u64)) -> ItemKey)
                .row(|t: &(String, u64), r: &mut RowUi<'_>| {
                    r.label(&t.0);
                })
                .update(cx, &mut self.st, &borrowed)
                .on_action(|action| actions.borrow_mut().push(action))
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let borrowed = self.items.borrow();
            List::new(LIST)
                .key(dup_key as fn(&(String, u64)) -> ItemKey)
                .row(|t: &(String, u64), r: &mut RowUi<'_>| {
                    r.label(&t.0);
                })
                .draw(ui, LIST_AREA, &self.st, &borrowed);
        }
    }

    let items = Rc::new(RefCell::new(vec![
        ("dup".to_string(), 1),
        ("dup".to_string(), 2),
    ]));
    let actions = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        DupApp {
            items,
            st: ListState::default(),
            actions: Rc::clone(&actions),
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    assert!(app.tab_to(LIST));
    assert!(
        app.row(0).contains("dup") && app.row(1).contains("dup"),
        "W12-03: both duplicate rows must paint"
    );
    let _ = app.click_part(LIST, PartRef::item(Part::ROW, ItemKey::pair(7, 2)));
    assert_eq!(
        actions.borrow().as_slice(),
        &[ListAction::Chose(ItemKey::pair(7, 2))],
        "W12-03: the second duplicate must resolve to its own key"
    );
    let _ = app.click_part(LIST, PartRef::item(Part::ROW, ItemKey::pair(7, 1)));
    assert_eq!(
        actions.borrow().as_slice(),
        &[
            ListAction::Chose(ItemKey::pair(7, 2)),
            ListAction::Chose(ItemKey::pair(7, 1)),
        ],
        "W12-03: the first duplicate must resolve to its own key"
    );
}

/// W12-04: wheels at both scroll boundaries change nothing but consume;
/// focus and cursor stay exactly where they were.
#[test]
fn w12_list_boundary_wheels_hold_focus_and_cursor() {
    let items: Vec<String> = (0..10).map(|i| format!("item{i}")).collect();
    let labels: Vec<&str> = items.iter().map(String::as_str).collect();
    let area = Rect::new(0, 0, 40, 4);
    let mut rig = ListRig::with(
        &labels,
        Status::Ready,
        None,
        SelectMode::Single,
        Vec::new(),
        area,
    );
    assert!(rig.app.tab_to(LIST));
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("item0")));

    // Top boundary: wheel up consumes and holds everything.
    let r = rig.app.mouse(MouseKind::Wheel(Axis::V, -1), 5, 1);
    assert!(r.is_consumed(), "W12-04: top wheel must consume");
    assert!(
        rig.app.row(0).contains("item0"),
        "W12-04: top wheel must not scroll"
    );
    assert!(
        rig.app.state_of(LIST).contains(StateFlags::FOCUSED),
        "W12-04: top wheel must not move focus"
    );
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("item0")),
        "W12-04: top wheel must not move the cursor"
    );
    assert!(
        rig.actions.borrow().is_empty(),
        "W12-04: top wheel must report nothing"
    );

    // Bottom boundary: End then wheel down holds the tail.
    let _ = rig.app.key(KeyCode::End);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("item9")));
    assert!(
        rig.app.row(0).contains("item6"),
        "W12-04: End must scroll to the tail, got {:?}",
        rig.app.row(0)
    );
    let r = rig.app.mouse(MouseKind::Wheel(Axis::V, 5), 5, 1);
    assert!(r.is_consumed(), "W12-04: bottom wheel must consume");
    assert!(
        rig.app.row(0).contains("item6"),
        "W12-04: bottom wheel must not scroll"
    );
    assert!(
        rig.app.state_of(LIST).contains(StateFlags::FOCUSED),
        "W12-04: bottom wheel must not move focus"
    );
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("item9")),
        "W12-04: bottom wheel must not move the cursor"
    );
}

/// W12-05: 100k rows invoke the row painter for exactly the visible rows —
/// the documented overscan is zero ("only visible rows invoke the
/// renderer").
#[test]
fn w12_list_hundred_k_rows_paint_visible_only() {
    struct BigApp {
        items: Vec<usize>,
        st: ListState,
        seen: Rc<RefCell<HashSet<usize>>>,
    }

    impl App for BigApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let _ = List::new(LIST).update(cx, &mut self.st, &self.items);
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let seen = Rc::clone(&self.seen);
            List::new(LIST)
                .row(|i: &usize, r: &mut RowUi<'_>| {
                    seen.borrow_mut().insert(*i);
                    r.label(&format!("row{i}"));
                })
                .draw(ui, LIST_AREA, &self.st, &self.items);
        }
    }

    let seen = Rc::new(RefCell::new(HashSet::new()));
    let mut app = Harness::new(
        BigApp {
            items: (0..100_000).collect(),
            st: ListState::default(),
            seen: Rc::clone(&seen),
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    assert!(app.tab_to(LIST));
    assert_eq!(
        seen.borrow().clone(),
        HashSet::from([0, 1, 2, 3, 4, 5, 6, 7]),
        "W12-05: exactly the 8 visible rows may paint"
    );
    seen.borrow_mut().clear();
    let _ = app.key(KeyCode::End);
    assert_eq!(
        seen.borrow().clone(),
        HashSet::from([
            99_992, 99_993, 99_994, 99_995, 99_996, 99_997, 99_998, 99_999
        ]),
        "W12-05: the tail paints exactly its 8 visible rows"
    );
}

/// W12-06 green half: a MARKER part patch re-voices exactly the marker
/// cells; gutter and labels keep their own paint. The LABEL half is the
/// `w12_list_label_patch_reaches_labels` parity record.
#[test]
fn w12_list_marker_patch_scopes_to_markers() {
    let bold = StylePatch::new().add(Modifier::BOLD);
    let mut patched = ListRig::with(
        &["aaa", "bbb", "ccc"],
        Status::Ready,
        None,
        SelectMode::Single,
        vec![(Part::MARKER, bold)],
        LIST_AREA,
    );
    assert!(patched.app.tab_to(LIST));
    let plain = ListRig::plain(&["aaa", "bbb", "ccc"]);

    // Plain row 2: markers gain BOLD, labels and gutters do not.
    let (_, _, patched_mod) = list_cell(&patched.app, 1, 2);
    assert!(
        patched_mod.contains("BOLD"),
        "W12-06: the patched marker must read bold, got {patched_mod:?}"
    );
    let (_, _, plain_mod) = list_cell(&plain.app, 1, 2);
    assert!(
        !plain_mod.contains("BOLD"),
        "W12-06: the control marker must stay unbold, got {plain_mod:?}"
    );
    assert_eq!(
        list_cell(&patched.app, 0, 2),
        list_cell(&plain.app, 0, 2),
        "W12-06: the gutter must keep its own paint"
    );
    assert_eq!(
        list_cell(&patched.app, 3, 2),
        list_cell(&plain.app, 3, 2),
        "W12-06: the label must keep its own paint"
    );
}

/// W12-06 parity record: `.patch_part` nominally accepts any part and the
/// list contract (`docs/components/list.md`: "a part override must affect
/// the cells it claims") demands the effect — but `List::draw` builds its
/// rows with a bare `RowUi::new`, so LABEL/META patches never reach the
/// row painter. Siblings (`chip.rs`, `nav_list.rs`, `steps.rs`, `tree.rs`)
/// all forward `ov.part_patch(...)` through `RowUi::new_with_patches`.
#[test]
fn w12_list_label_patch_reaches_labels() {
    let bold = StylePatch::new().add(Modifier::BOLD);
    let mut patched = ListRig::with(
        &["aaa", "bbb", "ccc"],
        Status::Ready,
        None,
        SelectMode::Single,
        vec![(Part::LABEL, bold)],
        LIST_AREA,
    );
    assert!(patched.app.tab_to(LIST));
    let (_, _, patched_mod) = list_cell(&patched.app, 3, 2);
    assert!(
        patched_mod.contains("BOLD"),
        "W12-06: the patched label must read bold, got {patched_mod:?}"
    );
}

// ---------------------------------------------------------------------------
// W13 FilterList (termrock-navigation)
// ---------------------------------------------------------------------------

const FLIST: Id = Id::root("control.states.flist");
const FLIST_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 40,
    height: 8,
};

/// Shared filter rig: caller-owned semantic items.
struct FilterApp {
    items: Rc<RefCell<Vec<Item<'static>>>>,
    st: FilterListState,
    actions: Rc<RefCell<Vec<FilterListAction>>>,
    query: Rc<RefCell<String>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    matched: Rc<Cell<usize>>,
    columns: bool,
}

impl FilterApp {
    fn list(columns: bool) -> FilterList<'static, Item<'static>> {
        let list = FilterList::new(FLIST);
        if columns {
            list.item_layout(ItemRowLayout::Columns)
        } else {
            list
        }
    }
}

impl App for FilterApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let actions = Rc::clone(&self.actions);
        let borrowed = self.items.borrow();
        let r = Self::list(self.columns)
            .update(cx, &mut self.st, &borrowed)
            .on_action(|action| actions.borrow_mut().push(action));
        self.query.replace(self.st.query().to_string());
        self.cursor.set(self.st.cursor());
        self.matched.set(self.st.matched_len());
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let borrowed = self.items.borrow();
        Self::list(self.columns).draw(ui, FLIST_AREA, &self.st, &borrowed);
    }
}

struct FilterRig {
    app: Harness<FilterApp>,
    items: Rc<RefCell<Vec<Item<'static>>>>,
    actions: Rc<RefCell<Vec<FilterListAction>>>,
    query: Rc<RefCell<String>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    matched: Rc<Cell<usize>>,
}

impl FilterRig {
    fn with(items: Vec<Item<'static>>, columns: bool) -> Self {
        let items_rc = Rc::new(RefCell::new(items));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let query = Rc::new(RefCell::new(String::new()));
        let cursor = Rc::new(Cell::new(None));
        let matched = Rc::new(Cell::new(0));
        let app = Harness::new(
            FilterApp {
                items: Rc::clone(&items_rc),
                st: FilterListState::default(),
                actions: Rc::clone(&actions),
                query: Rc::clone(&query),
                cursor: Rc::clone(&cursor),
                matched: Rc::clone(&matched),
                columns,
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        Self {
            app,
            items: items_rc,
            actions,
            query,
            cursor,
            matched,
        }
    }

    fn plain(items: Vec<Item<'static>>) -> Self {
        Self::with(items, false)
    }

    fn chose(&self) -> Vec<ItemKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                FilterListAction::Chose(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    fn query_changed(&self) -> usize {
        self.actions
            .borrow()
            .iter()
            .filter(|a| matches!(a, FilterListAction::QueryChanged))
            .count()
    }

    /// Flush one update: matches reconcile at the top of `update`, so the
    /// frame that edits the query still shows the previous match set.
    fn settle(&mut self) {
        self.app.ticks(1);
        self.app.draw();
    }
}

fn flabel(key: &str, label: &'static str) -> Item<'static> {
    Item::new(ItemKey::text(key), label)
}

/// W13-01: an empty source and a nonempty source with zero matches both
/// render the empty state with no cursor and no activation.
#[test]
fn w13_filter_empty_source_and_zero_matches() {
    // Empty source: nothing to match, nothing to choose.
    let mut rig = FilterRig::plain(vec![]);
    assert!(rig.app.tab_to(FLIST));
    assert!(
        (0..8).any(|y| rig.app.row(y).contains("No matches")),
        "W13-01: the empty source must name its state"
    );
    assert_eq!(rig.cursor.get(), None, "W13-01: no cursor without rows");
    let _ = rig.app.key(KeyCode::Enter);
    assert!(
        rig.chose().is_empty(),
        "W13-01: Enter on empty must choose nothing"
    );

    // Nonempty source filtered to zero: same empty state, stale rows gone.
    let mut rig = FilterRig::plain(vec![
        flabel("alpha", "alpha"),
        flabel("bravo", "bravo"),
        flabel("charlie", "charlie"),
    ]);
    assert!(rig.app.tab_to(FLIST));
    assert!(
        rig.app.row(0).contains("alpha"),
        "W13-01: rows must paint before filtering"
    );
    let _ = rig.app.type_str("zzz");
    rig.settle();
    assert_eq!(rig.matched.get(), 0, "W13-01: nothing may match zzz");
    assert_eq!(rig.cursor.get(), None, "W13-01: no cursor without matches");
    assert!(
        (0..8).any(|y| rig.app.row(y).contains("No matches")),
        "W13-01: zero matches must name the state"
    );
    assert!(
        !(0..8).any(|y| rig.app.row(y).contains("alpha")),
        "W13-01: stale rows must clear on zero matches"
    );
    let _ = rig.app.key(KeyCode::Enter);
    assert!(
        rig.chose().is_empty(),
        "W13-01: Enter on zero matches must choose nothing"
    );
    // Clearing the query brings every row back.
    let _ = rig.app.key(KeyCode::Esc);
    rig.settle();
    assert_eq!(rig.matched.get(), 3, "W13-01: Esc must clear the query");
    assert!(
        rig.app.row(0).contains("alpha"),
        "W13-01: rows must return after clear"
    );
}

/// W13-02 green half: a Unicode paste lands as one query event and filters;
/// Backspace removes one plain char. Whole-cluster backspace is the
/// `w13_filter_backspace_removes_grapheme` parity record.
#[test]
fn w13_filter_unicode_paste_and_plain_backspace() {
    let mut rig = FilterRig::plain(vec![
        flabel("tokyo", "日本語"),
        flabel("cafe", "café"),
        flabel("plain", "plain"),
    ]);
    assert!(rig.app.tab_to(FLIST));
    let _ = rig.app.paste("日本");
    rig.settle();
    assert_eq!(
        rig.query_changed(),
        1,
        "W13-02: paste must be a single query event"
    );
    assert_eq!(rig.query.borrow().as_str(), "日本");
    assert_eq!(rig.matched.get(), 1, "W13-02: the paste must filter");
    assert!(
        rig.app.row(0).contains("日本語"),
        "W13-02: the CJK match must paint, got {:?}",
        rig.app.row(0)
    );
    // Plain backspace removes one char and re-filters.
    let _ = rig.app.key(KeyCode::Backspace);
    assert_eq!(rig.query.borrow().as_str(), "日");
    let _ = rig.app.key(KeyCode::Backspace);
    assert_eq!(
        rig.query.borrow().as_str(),
        "",
        "W13-02: plain backspace must clear one char at a time"
    );
    rig.settle();
    assert_eq!(rig.matched.get(), 3);
}

/// W13-02 parity record: the legacy picker pops a whole grapheme cluster
/// (`pop_grapheme` in `src/widgets/picker.rs`, pinned by
/// `query_edits_are_grapheme_safe_and_paste_is_one_event` at
/// `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`: "a👩‍💻" backspaces to "a").
/// The candidate pops one `char`, splitting the ZWJ cluster and leaving a
/// dangling "a👩‍".
#[test]
#[ignore = "PARITY W13-02: query backspace pops one char and splits grapheme clusters (String::pop), reference pops the cluster"]
fn w13_filter_backspace_removes_grapheme() {
    let mut rig = FilterRig::plain(vec![flabel("x", "x")]);
    assert!(rig.app.tab_to(FLIST));
    let _ = rig.app.paste("a👩‍💻");
    assert_eq!(rig.query.borrow().as_str(), "a👩‍💻");
    let _ = rig.app.key(KeyCode::Backspace);
    assert_eq!(
        rig.query.borrow().as_str(),
        "a",
        "W13-02: backspace must remove the whole cluster, got {:?}",
        rig.query.borrow()
    );
}

/// W13-03: the source may reorder and resupply under an active filter; the
/// selected key survives, and a removed cursor reseeds without stale keys.
#[test]
fn w13_filter_revision_keeps_selected_key() {
    let mut rig = FilterRig::plain(vec![
        flabel("alpha", "alpha"),
        flabel("apricot", "apricot"),
        flabel("beta", "beta"),
    ]);
    assert!(rig.app.tab_to(FLIST));
    let _ = rig.app.type_str("ap");
    rig.settle();
    assert_eq!(rig.matched.get(), 2);
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("apricot")),
        "W13-03: cursor must start on apricot"
    );
    // Reorder + resupply under the filter: the key survives the revision.
    rig.items.replace(vec![
        flabel("apex", "apex"),
        flabel("beta", "beta"),
        flabel("apricot", "apricot"),
        flabel("alpha", "alpha"),
    ]);
    rig.app.ticks(1);
    rig.app.draw();
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("apricot")),
        "W13-03: the selected key must survive the revision"
    );
    assert_eq!(rig.matched.get(), 3, "W13-03: apex must join the matches");
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(
        rig.chose(),
        vec![ItemKey::text("apricot")],
        "W13-03: activation must follow the surviving key"
    );

    // Removing the cursor row reseeds onto a live match, never stale.
    rig.items.replace(vec![flabel("apex", "apex"), flabel("beta", "beta")]);
    rig.app.ticks(1);
    rig.app.draw();
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("apex")),
        "W13-03: the cursor must reseed onto a live match, got {:?}",
        rig.cursor.get()
    );
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(
        rig.chose(),
        vec![ItemKey::text("apricot"), ItemKey::text("apex")],
        "W13-03: no activation may target the removed key"
    );
}

/// W13-04: when the query makes the current row ineligible the cursor leaves
/// it; Enter cannot activate the stale row.
#[test]
fn w13_filter_ineligible_cursor_cannot_activate_stale() {
    let mut rig = FilterRig::plain(vec![
        flabel("alpha", "alpha"),
        flabel("beta", "beta"),
    ]);
    assert!(rig.app.tab_to(FLIST));
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("beta")));
    let _ = rig.app.type_str("alp");
    rig.settle();
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("alpha")),
        "W13-04: the cursor must leave the ineligible row"
    );
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(
        rig.chose(),
        vec![ItemKey::text("alpha")],
        "W13-04: Enter must activate the eligible row, never stale beta"
    );

    // A query matching nothing leaves Enter with no target at all.
    let _ = rig.app.type_str("zzz");
    rig.settle();
    assert_eq!(rig.matched.get(), 0);
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(
        rig.chose(),
        vec![ItemKey::text("alpha")],
        "W13-04: Enter on zero matches must activate nothing new"
    );
}

/// W13-05: matched spans bold whole graphemes across CJK and combining
/// marks; unmatched graphemes stay plain. Each phase is a fresh settled
/// render (the reference is a boot capture), measured on the non-cursor
/// row so cursor-row bold cannot hide span bold. Cross-frame residue is
/// the `w13_filter_wide_trail_cells_clear` parity record.
#[test]
fn w13_filter_matched_spans_cover_graphemes() {
    fn bold_cells(app: &Harness<FilterApp>, y: u16) -> Vec<String> {
        (0..40)
            .filter(|&x| {
                format!("{:?}", app.buffer()[Position::new(x, y)].modifier).contains("BOLD")
            })
            .map(|x| app.buffer()[Position::new(x, y)].symbol().to_string())
            .collect()
    }

    // One CJK match per row: exactly the 界 lead cell reads bold.
    let mut rig = span_duo(0);
    assert!(rig.app.tab_to(FLIST));
    let _ = rig.app.type_str("界");
    rig.settle();
    assert_eq!(rig.matched.get(), 2);
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("wide1")),
        "W13-05: a fresh filter seeds the first match"
    );
    let bolds = bold_cells(&rig.app, 1);
    assert_eq!(
        bolds,
        vec!["界".to_string()],
        "W13-05: exactly the matched CJK lead must read bold, got {bolds:?}"
    );

    // One combining match per row: the composed e+acute cell reads bold.
    let mut rig = span_duo(1);
    assert!(rig.app.tab_to(FLIST));
    let _ = rig.app.type_str("e\u{301}");
    rig.settle();
    assert_eq!(rig.matched.get(), 2);
    let bolds = bold_cells(&rig.app, 1);
    assert_eq!(
        bolds,
        vec!["e\u{301}".to_string()],
        "W13-05: exactly the matched combining cell must read bold, got {bolds:?}"
    );

    // A two-grapheme CJK span: both leads read bold.
    let mut rig = span_duo(2);
    assert!(rig.app.tab_to(FLIST));
    let _ = rig.app.type_str("日本");
    rig.settle();
    assert_eq!(rig.matched.get(), 2);
    let bolds = bold_cells(&rig.app, 1);
    assert_eq!(
        bolds,
        vec!["日".to_string(), "本".to_string()],
        "W13-05: both matched CJK leads must read bold, got {bolds:?}"
    );
}

fn span_rig() -> FilterRig {
    static M1: &[usize] = &[1];
    static M01: &[usize] = &[0, 1];
    static M12: &[usize] = &[1, 2];
    FilterRig::with(
        vec![
            Item::new(ItemKey::text("wide1"), "a界b").matched(M1),
            Item::new(ItemKey::text("wide2"), "x界y").matched(M1),
            Item::new(ItemKey::text("comb1"), "xe\u{301}y").matched(M1),
            Item::new(ItemKey::text("comb2"), "ae\u{301}b").matched(M1),
            Item::new(ItemKey::text("cjk1"), "日本語").matched(M01),
            Item::new(ItemKey::text("cjk2"), "旧日本").matched(M12),
        ],
        true,
    )
}

/// One matched pair per rig: the initial unfiltered paint writes the same
/// rows the query keeps, so span bold is observed without cross-frame
/// residue (residue itself is the `w13_filter_wide_trail_cells_clear`
/// parity record).
fn span_duo(which: usize) -> FilterRig {
    static M1: &[usize] = &[1];
    static M01: &[usize] = &[0, 1];
    static M12: &[usize] = &[1, 2];
    let items = match which {
        0 => vec![
            Item::new(ItemKey::text("wide1"), "a界b").matched(M1),
            Item::new(ItemKey::text("wide2"), "x界y").matched(M1),
        ],
        1 => vec![
            Item::new(ItemKey::text("comb1"), "xe\u{301}y").matched(M1),
            Item::new(ItemKey::text("comb2"), "ae\u{301}b").matched(M1),
        ],
        _ => vec![
            Item::new(ItemKey::text("cjk1"), "日本語").matched(M01),
            Item::new(ItemKey::text("cjk2"), "旧日本").matched(M12),
        ],
    };
    FilterRig::with(items, true)
}

/// W13-05 parity record: repainting a CJK row over an earlier paint leaves
/// the previous glyphs in the wide-char trail cells (the painter advances
/// past them without clearing or setting `skip`), so "旧日本" carries
/// stale "界"/"y" cells with stale bold that ratatui backends will print.
/// (`Harness::row` skips trail cells, so this reads the raw buffer.)
#[test]
#[ignore = "PARITY W13-05: wide-char trail cells are never cleared, stale glyphs leak across repaints"]
fn w13_filter_wide_trail_cells_clear() {
    let mut rig = span_rig();
    assert!(rig.app.tab_to(FLIST));
    let _ = rig.app.type_str("界");
    rig.settle();
    let _ = rig.app.key(KeyCode::Esc);
    let _ = rig.app.type_str("日本");
    rig.settle();
    assert_eq!(rig.matched.get(), 2);
    let cells: Vec<String> = (0..10)
        .map(|x| rig.app.buffer()[Position::new(x, 1)].symbol().to_string())
        .collect();
    assert!(
        !cells.iter().any(|s| s == "界" || s == "y"),
        "W13-05: no earlier-paint glyph may leak into 旧日本, got {cells:?}"
    );
    assert!(
        cells.iter().any(|s| s == "旧")
            && cells.iter().any(|s| s == "日")
            && cells.iter().any(|s| s == "本"),
        "W13-05: the row must read 旧日本, got {cells:?}"
    );
}

/// W13-04 parity record: when the query makes the cursor row ineligible
/// under multiple remaining matches, the legacy picker selects the first
/// eligible row (`refresh_keeps_identity_and_query_reset_selects_first` at
/// `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`: "first eligible row, never
/// a neighbour pretending to be it"). The candidate keeps the clamped
/// cursor index and scans forward (`nearest` in
/// `termrock-collections/src/collection/reconcile.rs`), landing on a2.
#[test]
#[ignore = "PARITY W13-04: reseed picks nearest-from-index (a2), reference picks first eligible (a1)"]
fn w13_filter_reseed_selects_first_eligible() {
    let mut rig = FilterRig::plain(vec![
        flabel("a1", "a1"),
        flabel("a2", "a2"),
        flabel("b1", "b1"),
        flabel("b2", "b2"),
    ]);
    assert!(rig.app.tab_to(FLIST));
    for _ in 0..3 {
        let _ = rig.app.key(KeyCode::Down);
    }
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("b2")));
    let _ = rig.app.type_str("a");
    rig.settle();
    assert_eq!(rig.matched.get(), 2);
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("a1")),
        "W13-04: reseed must select the first eligible row, got {:?}",
        rig.cursor.get()
    );
}

// ---------------------------------------------------------------------------
// W14 NavList (termrock-navigation)
// ---------------------------------------------------------------------------

const NAV: Id = Id::root("control.states.nav");
const NAV_SENTINEL: Id = Id::root("control.states.nav.sentinel");

#[derive(Clone)]
struct NavEntry {
    key: &'static str,
    label: &'static str,
    section: &'static str,
    icon: &'static str,
    badge: Option<&'static str>,
    disabled: bool,
}

fn nav_entry(key: &'static str, label: &'static str) -> NavEntry {
    NavEntry {
        key,
        label,
        section: "",
        icon: ">",
        badge: None,
        disabled: false,
    }
}

fn nav_key(t: &NavEntry) -> ItemKey {
    ItemKey::text(t.key)
}

fn nav_row(t: &NavEntry, r: &mut RowUi<'_>) {
    r.label(t.label);
}

fn nav_section(t: &NavEntry) -> &str {
    t.section
}

fn nav_icon(t: &NavEntry) -> &str {
    t.icon
}

fn nav_badge(t: &NavEntry) -> Option<&str> {
    t.badge
}

fn nav_disabled(t: &NavEntry) -> bool {
    t.disabled
}

/// Shared nav rig: caller-owned entries, shared area cell for breakpoints.
struct NavApp {
    items: Rc<RefCell<Vec<NavEntry>>>,
    st: NavListState,
    actions: Rc<RefCell<Vec<NavListAction>>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    current: Rc<Cell<Option<ItemKey>>>,
    offset: Rc<Cell<usize>>,
    area: Rc<Cell<Rect>>,
    scrollable: bool,
    clipped: bool,
    collapsed: bool,
}

impl App for NavApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let actions = Rc::clone(&self.actions);
        let borrowed = self.items.borrow();
        let list = NavList::new(NAV)
            .key(nav_key)
            .row(nav_row)
            .section(&nav_section)
            .icon(&nav_icon)
            .badge(&nav_badge)
            .disabled_item(&nav_disabled)
            .scrollable(self.scrollable);
        let list = if self.clipped {
            list.compact_when_clipped()
        } else {
            list
        };
        let list = if self.collapsed {
            list.mode(NavMode::Collapsed)
        } else {
            list
        };
        let r = list
            .update(cx, &mut self.st, &borrowed)
            .on_action(|action| actions.borrow_mut().push(action));
        self.cursor.set(self.st.cursor());
        self.current.set(self.st.current());
        self.offset.set(self.st.scroll().offset());
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let borrowed = self.items.borrow();
        let list = NavList::new(NAV)
            .key(nav_key)
            .row(nav_row)
            .section(&nav_section)
            .icon(&nav_icon)
            .badge(&nav_badge)
            .disabled_item(&nav_disabled)
            .scrollable(self.scrollable);
        let list = if self.clipped {
            list.compact_when_clipped()
        } else {
            list
        };
        let list = if self.collapsed {
            list.mode(NavMode::Collapsed)
        } else {
            list
        };
        list.draw(ui, self.area.get(), &self.st, &borrowed);
        ui.register_control(NAV_SENTINEL, Rect::new(31, 7, 8, 1), Focusability::Focusable);
    }
}

struct NavRig {
    app: Harness<NavApp>,
    #[allow(dead_code)]
    items: Rc<RefCell<Vec<NavEntry>>>,
    actions: Rc<RefCell<Vec<NavListAction>>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    current: Rc<Cell<Option<ItemKey>>>,
    offset: Rc<Cell<usize>>,
    area: Rc<Cell<Rect>>,
}

impl NavRig {
    fn with(
        items: Vec<NavEntry>,
        area: Rect,
        scrollable: bool,
        clipped: bool,
        collapsed: bool,
    ) -> Self {
        let items_rc = Rc::new(RefCell::new(items));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let cursor = Rc::new(Cell::new(None));
        let current = Rc::new(Cell::new(None));
        let offset = Rc::new(Cell::new(0));
        let area_rc = Rc::new(Cell::new(area));
        let app = Harness::new(
            NavApp {
                items: Rc::clone(&items_rc),
                st: NavListState::new(),
                actions: Rc::clone(&actions),
                cursor: Rc::clone(&cursor),
                current: Rc::clone(&current),
                offset: Rc::clone(&offset),
                area: Rc::clone(&area_rc),
                scrollable,
                clipped,
                collapsed,
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        Self {
            app,
            items: items_rc,
            actions,
            cursor,
            current,
            offset,
            area: area_rc,
        }
    }

    fn plain(items: Vec<NavEntry>) -> Self {
        Self::with(
            items,
            Rect {
                x: 0,
                y: 0,
                width: 30,
                height: 8,
            },
            false,
            false,
            false,
        )
    }

    fn chose(&self) -> Vec<ItemKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                NavListAction::Chose(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    fn moved(&self) -> Vec<ItemKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                NavListAction::Moved(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    /// Flush one update: cursor reveals apply in the next prepare, so the
    /// frame that moves the cursor still shows the previous scroll offset.
    fn settle(&mut self) {
        self.app.ticks(1);
        self.app.draw();
    }

    /// Resize the nav area and settle so paint and scroll state agree.
    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
        self.app.draw();
        self.app.ticks(1);
        self.app.draw();
    }
}

/// W14-01: the keyboard cursor moves without navigating; only Enter chooses
/// and the active destination stays behind when the cursor leaves it.
#[test]
fn w14_nav_active_differs_from_cursor() {
    let mut rig = NavRig::plain(vec![
        nav_entry("home", "Home"),
        nav_entry("search", "Search"),
        nav_entry("settings", "Settings"),
    ]);
    assert!(rig.app.tab_to(NAV));
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("home")));
    assert_eq!(rig.current.get(), None, "W14-01: nothing is active yet");

    // Moving the cursor reports Moved and never navigates.
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("search")));
    assert_eq!(rig.current.get(), None, "W14-01: moving must not navigate");
    assert_eq!(rig.moved(), vec![ItemKey::text("search")]);
    assert!(rig.chose().is_empty(), "W14-01: no choice without Enter");

    // Enter chooses: cursor and current agree for a moment.
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(rig.chose(), vec![ItemKey::text("search")]);
    assert_eq!(rig.current.get(), Some(ItemKey::text("search")));

    // The cursor leaves; the active destination stays behind.
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("settings")));
    assert_eq!(
        rig.current.get(),
        Some(ItemKey::text("search")),
        "W14-01: the active destination must stay behind the cursor"
    );
    assert_eq!(rig.chose().len(), 1, "W14-01: no second choice");

    // The two rows resolve different flags: cursor-only versus active-only.
    let cursor_marker = cell_symbol(rig.app.buffer(), 1, 2);
    let active_marker = cell_symbol(rig.app.buffer(), 1, 1);
    assert_ne!(
        cursor_marker, active_marker,
        "W14-01: cursor and active rows must read differently"
    );
}

/// W14-02: section headings paint with separators; the cursor skips
/// disabled entries in both directions and Home/End land on enabled ends.
#[test]
fn w14_nav_sections_and_disabled_skip() {
    let mut search = nav_entry("search", "Search");
    search.section = "Main";
    search.disabled = true;
    let mut home = nav_entry("home", "Home");
    home.section = "Main";
    let mut files = nav_entry("files", "Files");
    files.section = "Main";
    let mut settings = nav_entry("settings", "Settings");
    settings.section = "Tools";
    let mut rig = NavRig::plain(vec![home, search, files, settings]);
    assert!(rig.app.tab_to(NAV));

    // Headings paint above their sections with a blank separator between.
    assert!(
        rig.app.row(0).contains("Main"),
        "W14-02: the first heading must paint, got {:?}",
        rig.app.row(0)
    );
    assert!(
        rig.app.row(5).contains("Tools"),
        "W14-02: the second heading must paint, got {:?}",
        rig.app.row(5)
    );
    assert!(
        rig.app.row(4).trim().is_empty(),
        "W14-02: sections must be separated, got {:?}",
        rig.app.row(4)
    );

    // Down skips the disabled entry; Up skips it backwards too.
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("home")));
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("files")),
        "W14-02: Down must skip the disabled entry"
    );
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("settings")));
    let _ = rig.app.key(KeyCode::Up);
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("files")),
        "W14-02: Up must skip the disabled entry backwards"
    );
    let _ = rig.app.key(KeyCode::Up);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("home")));

    // Home/End land on the enabled ends, never the disabled middle.
    let _ = rig.app.key(KeyCode::End);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("settings")));
    let _ = rig.app.key(KeyCode::Home);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("home")));

    // Pointer activation cannot target the disabled row or the heading.
    let _ = rig.app.click(6, 2);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("home")));
    assert!(rig.chose().is_empty(), "W14-02: disabled clicks choose nothing");
    let _ = rig.app.click(6, 0);
    assert!(rig.chose().is_empty(), "W14-02: heading clicks choose nothing");

    // The disabled row resolves the disabled foreground, not the row voice.
    let enabled_fg = rig.app.buffer()[Position::new(6, 1)].fg;
    let disabled_fg = rig.app.buffer()[Position::new(6, 2)].fg;
    assert_ne!(
        enabled_fg, disabled_fg,
        "W14-02: the disabled row must read differently"
    );
}

/// W14-03: shrinking through the compact breakpoint hides headings while
/// keeping labels; growing back restores them; active key, cursor and
/// scroll survive the roundtrip.
#[test]
fn w14_nav_breakpoint_roundtrip_preserves_key_and_scroll() {
    let sectioned = |key: &'static str, section: &'static str| NavEntry {
        section,
        ..nav_entry(key, key)
    };
    let mut rig = NavRig::with(
        vec![
            sectioned("a1", "A"),
            sectioned("a2", "A"),
            sectioned("b1", "B"),
            sectioned("b2", "B"),
            sectioned("b3", "B"),
        ],
        Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 8,
        },
        true,
        true,
        false,
    );
    assert!(rig.app.tab_to(NAV));
    for _ in 0..3 {
        let _ = rig.app.key(KeyCode::Down);
    }
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(rig.current.get(), Some(ItemKey::text("b2")));
    assert!(
        rig.app.row(0).contains('A'),
        "W14-03: headings must paint at full height"
    );

    // Shrink through the breakpoint: headings go, labels and keys stay.
    rig.set_area(Rect {
        x: 0,
        y: 0,
        width: 30,
        height: 4,
    });
    assert!(
        !(0..4).any(|y| rig.app.row(y).contains('A') || rig.app.row(y).contains('B')),
        "W14-03: compact must hide the headings"
    );
    assert!(
        rig.app.row(0).contains("a1"),
        "W14-03: compact must keep the labels"
    );
    assert_eq!(rig.current.get(), Some(ItemKey::text("b2")));
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("b2")));

    // Overflow at compact height scrolls; the active key is untouched.
    let _ = rig.app.key(KeyCode::Down);
    rig.settle();
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("b3")));
    assert_eq!(
        rig.offset.get(),
        1,
        "W14-03: the compact list must scroll to the cursor"
    );
    assert_eq!(
        rig.current.get(),
        Some(ItemKey::text("b2")),
        "W14-03: scrolling must not navigate"
    );

    // Grow back: headings return, keys and cursor survive.
    rig.set_area(Rect {
        x: 0,
        y: 0,
        width: 30,
        height: 8,
    });
    assert!(
        rig.app.row(0).contains('A'),
        "W14-03: growing back must restore the headings"
    );
    assert_eq!(rig.current.get(), Some(ItemKey::text("b2")));
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("b3")));
    assert!(
        rig.app.row(7).contains("b3"),
        "W14-03: the cursor row must stay visible, got {:?}",
        rig.app.row(7)
    );

    // Shrink again: the scroll offset roundtrips to the same value.
    rig.set_area(Rect {
        x: 0,
        y: 0,
        width: 30,
        height: 4,
    });
    assert_eq!(
        rig.offset.get(),
        1,
        "W14-03: the scroll offset must roundtrip"
    );
}

/// W14-04: an empty list paints blank and ignores keys; overflow scrolls;
/// narrow widths clip badges after labels; collapsed mode keeps icons only.
#[test]
fn w14_nav_empty_overflow_and_clipped_badges() {
    // Empty: blank paint, no cursor, keys choose nothing.
    let mut rig = NavRig::plain(vec![]);
    assert!(rig.app.tab_to(NAV));
    assert_eq!(rig.cursor.get(), None, "W14-04: no cursor without rows");
    assert!(
        rig.app.row(0).trim().is_empty(),
        "W14-04: the empty list must paint blank"
    );
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.key(KeyCode::Enter);
    assert!(rig.chose().is_empty(), "W14-04: empty keys choose nothing");
    assert!(rig.moved().is_empty(), "W14-04: empty keys move nothing");

    // Overflow: the wheel advances shared scroll and rows shift.
    let items: Vec<NavEntry> = (1..=12)
        .map(|n| {
            let label: &'static str = Box::leak(format!("c{n}").into_boxed_str());
            nav_entry(label, label)
        })
        .collect();
    let mut rig = NavRig::with(
        items,
        Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 8,
        },
        true,
        false,
        false,
    );
    assert!(rig.app.tab_to(NAV));
    assert!(rig.app.row(0).contains("c1"));
    // One wheel notch scrolls three rows (runtime convention), rows shift.
    let _ = rig.app.wheel(Axis::V, 1, 2, 2);
    assert_eq!(rig.offset.get(), 3, "W14-04: the wheel must scroll");
    assert!(
        rig.app.row(0).contains("c4"),
        "W14-04: rows must shift under scroll, got {:?}",
        rig.app.row(0)
    );

    // Clipped badges: the label yields first, then the badge clips.
    let mut badged = nav_entry("settings", "SettingsX");
    badged.badge = Some("99+");
    let mut rig = NavRig::with(
        vec![badged],
        Rect {
            x: 0,
            y: 0,
            width: 12,
            height: 8,
        },
        false,
        false,
        false,
    );
    assert!(rig.app.tab_to(NAV));
    assert!(
        rig.app.row(0).contains("99+"),
        "W14-04: the badge must survive clipping, got {:?}",
        rig.app.row(0)
    );
    assert!(
        rig.app.row(0).contains('…') && !rig.app.row(0).contains("SettingsX"),
        "W14-04: the label must clip first, got {:?}",
        rig.app.row(0)
    );
    rig.area.set(Rect {
        x: 0,
        y: 0,
        width: 7,
        height: 8,
    });
    rig.app.draw();
    assert!(
        rig.app.row(0).contains("99+") && !rig.app.row(0).contains("Set"),
        "W14-04: extreme widths keep the badge alone, got {:?}",
        rig.app.row(0)
    );

    // Collapsed: icons paint, labels are gone.
    let mut rig = NavRig::with(
        vec![nav_entry("home", "Home")],
        Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 8,
        },
        false,
        false,
        true,
    );
    assert!(rig.app.tab_to(NAV));
    assert!(
        rig.app.row(0).contains('>'),
        "W14-04: collapsed must keep the icon, got {:?}",
        rig.app.row(0)
    );
    assert!(
        !rig.app.row(0).contains("Home"),
        "W14-04: collapsed must drop the label, got {:?}",
        rig.app.row(0)
    );
}

/// W14-05: the sidebar shares scroll state and focus machinery instead of
/// copying them: one focus stop, wheel-driven shared scroll, and the same
/// `ScrollState` type the plain list exposes.
#[test]
fn w14_nav_shares_scroll_and_focus() {
    // The scroll state type unifies with the plain list's: one shared
    // implementation, not a sidebar copy.
    fn assert_shared_scroll(a: &ScrollState, b: &ScrollState) {
        assert_eq!(a.offset(), b.offset());
        assert_eq!(a.viewport_len(), b.viewport_len());
        assert_eq!(a.content_len(), b.content_len());
    }
    let nav = NavListState::new();
    let list = ListState::default();
    assert_shared_scroll(nav.scroll(), list.scroll());

    // One focus stop: Tab enters the sentinel, Tab cycles back.
    let mut rig = NavRig::plain(vec![nav_entry("home", "Home")]);
    assert!(rig.app.tab_to(NAV));
    let _ = rig.app.key(KeyCode::Tab);
    assert!(
        rig.app.state_of(NAV_SENTINEL).contains(StateFlags::FOCUSED),
        "W14-05: one Tab must reach the sentinel"
    );
    assert!(
        !rig.app.state_of(NAV).contains(StateFlags::FOCUSED),
        "W14-05: the list must hold a single stop"
    );
    let _ = rig.app.key(KeyCode::Tab);
    assert!(
        rig.app.state_of(NAV).contains(StateFlags::FOCUSED),
        "W14-05: Tab must cycle back to the list"
    );

    // The wheel drives the shared scroll region, not bespoke row code.
    let items: Vec<NavEntry> = (1..=12)
        .map(|n| {
            let label: &'static str = Box::leak(format!("w{n}").into_boxed_str());
            nav_entry(label, label)
        })
        .collect();
    let mut rig = NavRig::with(
        items,
        Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 8,
        },
        true,
        false,
        false,
    );
    assert!(rig.app.tab_to(NAV));
    assert_eq!(rig.offset.get(), 0);
    // Two notches at three rows each clamp to the four-row maximum.
    let _ = rig.app.wheel(Axis::V, 2, 2, 2);
    assert_eq!(
        rig.offset.get(),
        4,
        "W14-05: the wheel must drive shared scroll"
    );
}

// ---------------------------------------------------------------------------
// W15 Tree (termrock-navigation)
// ---------------------------------------------------------------------------

const TREE: Id = Id::root("control.states.tree");
const TREE_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 40,
    height: 8,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum TreeMark {
    Plain,
    Loading,
    Error,
}

#[derive(Clone)]
struct TreeEntry {
    key: String,
    label: String,
    depth: u16,
    kind: NodeKind,
    mark: TreeMark,
    disabled: bool,
}

fn tree_entry(key: &str, depth: u16, kind: NodeKind) -> TreeEntry {
    TreeEntry {
        key: key.to_string(),
        label: key.to_string(),
        depth,
        kind,
        mark: TreeMark::Plain,
        disabled: false,
    }
}

fn tree_key(t: &TreeEntry) -> ItemKey {
    ItemKey::text(&t.key)
}

fn tree_node(t: &TreeEntry) -> TreeNode {
    match t.kind {
        NodeKind::Leaf => TreeNode::leaf(t.depth),
        NodeKind::Parent => TreeNode::parent(t.depth),
        NodeKind::Lazy => TreeNode::lazy(t.depth),
    }
}

fn tree_row(t: &TreeEntry, r: &mut RowUi<'_>) {
    match t.mark {
        TreeMark::Plain => r.label(&t.label),
        TreeMark::Loading => r.label_spans(&[Span::new(&t.label).dim()]),
        TreeMark::Error => r.label_spans(&[Span::new(&t.label).role(Role::Danger)]),
    }
}

fn tree_disabled(t: &TreeEntry) -> bool {
    t.disabled
}

fn tree_match_rs(t: &TreeEntry) -> bool {
    t.label.ends_with(".rs")
}

fn tree_match_md(t: &TreeEntry) -> bool {
    t.label.ends_with(".md")
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TreeFilter {
    Off,
    Rs,
    Md,
}

struct TreeApp {
    items: Rc<RefCell<Vec<TreeEntry>>>,
    st: TreeState,
    actions: Rc<RefCell<Vec<TreeAction>>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    chosen: Rc<Cell<Option<ItemKey>>>,
    expanded: Rc<RefCell<Vec<ItemKey>>>,
    visible_len: Rc<Cell<usize>>,
    filter: Rc<Cell<TreeFilter>>,
    branch_activation: TreeBranchActivation,
    branch_click: TreeBranchClick,
}

impl App for TreeApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let actions = Rc::clone(&self.actions);
        let borrowed = self.items.borrow();
        let list = Tree::new(TREE)
            .key(tree_key)
            .row(tree_row)
            .node(&tree_node)
            .disabled_item(&tree_disabled)
            .branch_activation(self.branch_activation)
            .branch_click(self.branch_click);
        let list = match self.filter.get() {
            TreeFilter::Off => list,
            TreeFilter::Rs => list.query(1, &tree_match_rs),
            TreeFilter::Md => list.query(2, &tree_match_md),
        };
        let r = list
            .update(cx, &mut self.st, &borrowed)
            .on_action(|action| actions.borrow_mut().push(action));
        self.cursor.set(self.st.cursor());
        self.chosen.set(self.st.chosen());
        self.visible_len.set(self.st.scroll().content_len());
        self.expanded.replace(
            borrowed
                .iter()
                .map(tree_key)
                .filter(|k| self.st.is_expanded(*k))
                .collect(),
        );
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let borrowed = self.items.borrow();
        let list = Tree::new(TREE)
            .key(tree_key)
            .row(tree_row)
            .node(&tree_node)
            .disabled_item(&tree_disabled)
            .branch_activation(self.branch_activation)
            .branch_click(self.branch_click);
        let list = match self.filter.get() {
            TreeFilter::Off => list,
            TreeFilter::Rs => list.query(1, &tree_match_rs),
            TreeFilter::Md => list.query(2, &tree_match_md),
        };
        list.draw(ui, TREE_AREA, &self.st, &borrowed);
    }
}

struct TreeRig {
    app: Harness<TreeApp>,
    items: Rc<RefCell<Vec<TreeEntry>>>,
    actions: Rc<RefCell<Vec<TreeAction>>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    chosen: Rc<Cell<Option<ItemKey>>>,
    expanded: Rc<RefCell<Vec<ItemKey>>>,
    visible_len: Rc<Cell<usize>>,
    filter: Rc<Cell<TreeFilter>>,
}

impl TreeRig {
    fn with(
        items: Vec<TreeEntry>,
        branch_activation: TreeBranchActivation,
        branch_click: TreeBranchClick,
    ) -> Self {
        let items_rc = Rc::new(RefCell::new(items));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let cursor = Rc::new(Cell::new(None));
        let chosen = Rc::new(Cell::new(None));
        let expanded = Rc::new(RefCell::new(Vec::new()));
        let visible_len = Rc::new(Cell::new(0));
        let filter = Rc::new(Cell::new(TreeFilter::Off));
        let app = Harness::new(
            TreeApp {
                items: Rc::clone(&items_rc),
                st: TreeState::new(),
                actions: Rc::clone(&actions),
                cursor: Rc::clone(&cursor),
                chosen: Rc::clone(&chosen),
                expanded: Rc::clone(&expanded),
                visible_len: Rc::clone(&visible_len),
                filter: Rc::clone(&filter),
                branch_activation,
                branch_click,
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        Self {
            app,
            items: items_rc,
            actions,
            cursor,
            chosen,
            expanded,
            visible_len,
            filter,
        }
    }

    fn plain(items: Vec<TreeEntry>) -> Self {
        Self::with(
            items,
            TreeBranchActivation::Toggle,
            TreeBranchClick::Toggle,
        )
    }

    fn expanded_keys(&self) -> Vec<ItemKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                TreeAction::Expanded(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    fn collapsed_keys(&self) -> Vec<ItemKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                TreeAction::Collapsed(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    fn chose(&self) -> Vec<ItemKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                TreeAction::Chose(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    fn activated(&self) -> Vec<ItemKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                TreeAction::Activated(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    fn settle(&mut self) {
        self.app.ticks(1);
        self.app.draw();
    }
}

/// W15-01: collapsed branches hide children behind `▸`, expanded branches
/// show them behind `▾`, leaves keep a blank disclosure cell, and loading
/// and error placeholder rows read distinctly.
#[test]
fn w15_tree_states_and_status_rows() {
    let mut loading = tree_entry("loading…", 2, NodeKind::Leaf);
    loading.mark = TreeMark::Loading;
    let mut failed = tree_entry("failed.txt", 2, NodeKind::Leaf);
    failed.mark = TreeMark::Error;
    let mut rig = TreeRig::plain(vec![
        tree_entry("root", 0, NodeKind::Parent),
        tree_entry("src", 1, NodeKind::Parent),
        tree_entry("file.rs", 2, NodeKind::Leaf),
        loading,
        failed,
        tree_entry("docs", 1, NodeKind::Leaf),
    ]);
    assert!(rig.app.tab_to(TREE));

    // Collapsed: children hidden behind the closed disclosure.
    assert_eq!(
        cell_symbol(rig.app.buffer(), 1, 0),
        "▸",
        "W15-01: the collapsed branch must show ▸"
    );
    assert!(
        !rig.app.row(1).contains("src"),
        "W15-01: collapsed children must hide, got {:?}",
        rig.app.row(1)
    );

    // Expand the root, then src: children appear behind the open disclosure.
    let _ = rig.app.key(KeyCode::Right);
    assert_eq!(rig.expanded_keys(), vec![ItemKey::text("root")]);
    assert_eq!(
        cell_symbol(rig.app.buffer(), 1, 0),
        "▾",
        "W15-01: the expanded branch must show ▾"
    );
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.key(KeyCode::Right);
    assert_eq!(
        rig.expanded_keys(),
        vec![ItemKey::text("root"), ItemKey::text("src")]
    );
    assert!(rig.app.row(2).contains("file.rs"));
    assert!(rig.app.row(3).contains("loading…"));
    assert!(rig.app.row(4).contains("failed.txt"));

    // Leaves keep a blank disclosure cell.
    assert_eq!(
        cell_symbol(rig.app.buffer(), 5, 2),
        " ",
        "W15-01: leaves must keep a blank disclosure cell"
    );

    // The loading row reads dimmed, the error row danger-voiced.
    let loading_dim = rig.app.buffer()[Position::new(7, 3)]
        .modifier
        .contains(Modifier::DIM);
    assert!(loading_dim, "W15-01: the loading row must read dimmed");
    let plain_fg = rig.app.buffer()[Position::new(7, 2)].fg;
    let error_fg = rig.app.buffer()[Position::new(7, 4)].fg;
    assert_ne!(
        plain_fg, error_fg,
        "W15-01: the error row must read danger-voiced"
    );

    // Collapse src: its rows hide again; Left on a closed branch seeks up.
    let _ = rig.app.key(KeyCode::Left);
    assert_eq!(rig.collapsed_keys(), vec![ItemKey::text("src")]);
    assert!(
        !rig.app.row(2).contains("file.rs"),
        "W15-01: collapsed children must hide again"
    );
    let _ = rig.app.key(KeyCode::Left);
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("root")),
        "W15-01: Left on a closed branch must seek the parent"
    );
}

/// W15-01b: a lazy branch emits its fetch request on expand; the caller
/// appends the children and they paint without a second fetch.
#[test]
fn w15_tree_lazy_fetch_appends_children() {
    let mut rig = TreeRig::plain(vec![
        tree_entry("root", 0, NodeKind::Parent),
        tree_entry("remote", 1, NodeKind::Lazy),
    ]);
    assert!(rig.app.tab_to(TREE));
    let _ = rig.app.key(KeyCode::Right);
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("remote")));

    // Expanding the lazy node requests the fetch; nothing arrives yet.
    let _ = rig.app.key(KeyCode::Right);
    assert_eq!(
        rig.expanded_keys(),
        vec![ItemKey::text("root"), ItemKey::text("remote")]
    );
    assert!(
        rig.app.row(2).trim().is_empty(),
        "W15-01: children must arrive from the caller, got {:?}",
        rig.app.row(2)
    );

    // The caller appends the children; they paint under the open branch.
    rig.items.borrow_mut().push(tree_entry("r1", 2, NodeKind::Leaf));
    rig.items.borrow_mut().push(tree_entry("r2", 2, NodeKind::Leaf));
    rig.settle();
    assert!(rig.app.row(2).contains("r1"));
    assert!(rig.app.row(3).contains("r2"));

    // Collapse and re-expand: the fetched children persist, unfetched once.
    let _ = rig.app.key(KeyCode::Left);
    assert_eq!(rig.collapsed_keys(), vec![ItemKey::text("remote")]);
    let _ = rig.app.key(KeyCode::Right);
    assert_eq!(
        rig.expanded_keys(),
        vec![
            ItemKey::text("root"),
            ItemKey::text("remote"),
            ItemKey::text("remote")
        ]
    );
    assert!(
        rig.app.row(2).contains("r1"),
        "W15-01: fetched children must persist across collapse"
    );
}

/// W15-02: disclosure clicks always toggle; body clicks and Enter follow
/// the branch policies; leaves choose on Space/click and activate on Enter.
#[test]
fn w15_tree_disclosure_body_and_policies() {
    let items = || {
        vec![
            tree_entry("root", 0, NodeKind::Parent),
            tree_entry("leaf", 1, NodeKind::Leaf),
        ]
    };

    // Default policies: disclosure and body clicks both toggle the branch.
    let mut rig = TreeRig::plain(items());
    assert!(rig.app.tab_to(TREE));
    let _ = rig
        .app
        .click_part(TREE, PartRef::item(Part::ICON, ItemKey::text("root")));
    assert_eq!(rig.expanded_keys(), vec![ItemKey::text("root")]);
    assert!(rig.chose().is_empty(), "W15-02: disclosure must not choose");
    let _ = rig.app.click(10, 0);
    assert_eq!(
        rig.collapsed_keys(),
        vec![ItemKey::text("root")],
        "W15-02: a body click must toggle under TreeBranchClick::Toggle"
    );

    // Choose-on-click: the body chooses the branch, the disclosure toggles.
    let mut rig = TreeRig::with(items(), TreeBranchActivation::Toggle, TreeBranchClick::Choose);
    assert!(rig.app.tab_to(TREE));
    let _ = rig.app.click(10, 0);
    assert_eq!(rig.chose(), vec![ItemKey::text("root")]);
    assert!(
        rig.expanded_keys().is_empty(),
        "W15-02: a body click must not toggle under TreeBranchClick::Choose"
    );
    let _ = rig
        .app
        .click_part(TREE, PartRef::item(Part::ICON, ItemKey::text("root")));
    assert_eq!(rig.expanded_keys(), vec![ItemKey::text("root")]);

    // Activate-on-Enter: Enter activates the branch without toggling.
    let mut rig = TreeRig::with(
        items(),
        TreeBranchActivation::Activate,
        TreeBranchClick::Toggle,
    );
    assert!(rig.app.tab_to(TREE));
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(rig.activated(), vec![ItemKey::text("root")]);
    assert!(
        rig.expanded_keys().is_empty(),
        "W15-02: Enter must not toggle under TreeBranchActivation::Activate"
    );
    assert_eq!(rig.chosen.get(), Some(ItemKey::text("root")));

    // Toggle-on-Enter (default): Enter toggles the branch instead.
    let mut rig = TreeRig::plain(items());
    assert!(rig.app.tab_to(TREE));
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(rig.expanded_keys(), vec![ItemKey::text("root")]);
    assert!(rig.activated().is_empty());

    // Leaves: Space chooses, Enter activates, the blank fold cell chooses.
    let mut rig = TreeRig::plain(items());
    assert!(rig.app.tab_to(TREE));
    let _ = rig.app.key(KeyCode::Right);
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("leaf")));
    let _ = rig.app.key(KeyCode::Char(' '));
    assert_eq!(rig.chose(), vec![ItemKey::text("leaf")]);
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(rig.activated(), vec![ItemKey::text("leaf")]);
    // The leaf's blank fold cell is body, not disclosure: it chooses.
    let fold_x = 1 + 2;
    let _ = rig.app.click(fold_x, 1);
    assert_eq!(
        rig.chose(),
        vec![ItemKey::text("leaf"), ItemKey::text("leaf")],
        "W15-02: the leaf fold cell must behave as body"
    );
}

/// W15-03: removing the pressed row, the cursor subtree, or a lazily
/// loading branch leaves no stale activation and a valid cursor.
#[test]
fn w15_tree_removal_during_press_and_pending_load() {
    // Press on a leaf, remove it before release: no stale choice.
    let mut rig = TreeRig::plain(vec![
        tree_entry("root", 0, NodeKind::Parent),
        tree_entry("gone", 1, NodeKind::Leaf),
        tree_entry("stays", 1, NodeKind::Leaf),
    ]);
    assert!(rig.app.tab_to(TREE));
    let _ = rig.app.key(KeyCode::Right);
    let _ = rig.app.mouse(MouseKind::Down, 10, 1);
    rig.items.borrow_mut().remove(1);
    rig.settle();
    let _ = rig.app.mouse(MouseKind::Up, 10, 1);
    rig.settle();
    assert!(
        rig.chose().is_empty() && rig.activated().is_empty(),
        "W15-03: a removed press target must not activate, chose {:?} activated {:?}",
        rig.chose(),
        rig.activated()
    );
    assert_ne!(
        rig.cursor.get(),
        Some(ItemKey::text("gone")),
        "W15-03: the cursor must leave the removed row"
    );

    // Remove the cursor's parent subtree: the cursor reseeds onto a live row.
    let mut rig = TreeRig::plain(vec![
        tree_entry("root", 0, NodeKind::Parent),
        tree_entry("kid", 1, NodeKind::Leaf),
        tree_entry("other", 0, NodeKind::Leaf),
    ]);
    assert!(rig.app.tab_to(TREE));
    let _ = rig.app.key(KeyCode::Right);
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("kid")));
    rig.items.borrow_mut().drain(0..2);
    rig.settle();
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("other")),
        "W15-03: the cursor must reseed onto a live row, got {:?}",
        rig.cursor.get()
    );

    // Expand a lazy branch, then remove it before children arrive: quiet.
    let mut rig = TreeRig::plain(vec![
        tree_entry("root", 0, NodeKind::Parent),
        tree_entry("remote", 1, NodeKind::Lazy),
    ]);
    assert!(rig.app.tab_to(TREE));
    let _ = rig.app.key(KeyCode::Right);
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.key(KeyCode::Right);
    assert_eq!(
        rig.expanded_keys(),
        vec![ItemKey::text("root"), ItemKey::text("remote")]
    );
    rig.items.borrow_mut().remove(1);
    rig.settle();
    assert!(
        rig.chose().is_empty() && rig.activated().is_empty(),
        "W15-03: a removed pending branch must not activate"
    );
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("root")),
        "W15-03: the cursor must fall back to the live root"
    );
}

/// W15-04: expand-all and collapse-all on a large tree draw only the
/// viewport window and keep scroll consistent.
#[test]
fn w15_tree_fold_all_draws_bounded_window() {
    let mut items = Vec::new();
    for parent in 0..20 {
        items.push(tree_entry(&format!("p{parent}"), 0, NodeKind::Parent));
        for kid in 0..9 {
            items.push(tree_entry(&format!("p{parent}k{kid}"), 1, NodeKind::Leaf));
        }
    }
    assert_eq!(items.len(), 200);
    let mut rig = TreeRig::plain(items);
    assert!(rig.app.tab_to(TREE));
    assert_eq!(rig.visible_len.get(), 20, "W15-04: roots only at boot");

    // Expand all: every node visible, only the viewport window painted.
    let _ = rig.app.key(KeyCode::Char('*'));
    assert_eq!(rig.visible_len.get(), 200, "W15-04: expand-all must open all");
    assert!(rig.app.row(0).contains("p0"));
    assert!(
        rig.app.row(7).contains("p0k6"),
        "W15-04: the window must end at the viewport, got {:?}",
        rig.app.row(7)
    );
    let painted = (0..8).filter(|y| !rig.app.row(*y).trim().is_empty()).count();
    assert_eq!(painted, 8, "W15-04: exactly the viewport window paints");

    // End seeks the last row and scrolls there.
    let _ = rig.app.key(KeyCode::End);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("p19k8")));
    assert!(
        rig.app.row(7).contains("p19k8"),
        "W15-04: End must reveal the last row, got {:?}",
        rig.app.row(7)
    );

    // Collapse all: roots only again, cursor reseeds onto a live root.
    let _ = rig.app.key(KeyCode::Char('-'));
    assert_eq!(
        rig.visible_len.get(),
        20,
        "W15-04: collapse-all must close all"
    );
    rig.settle();
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("p19")),
        "W15-04: the cursor must reseed onto a live root, got {:?}",
        rig.cursor.get()
    );
    let _ = rig.app.key(KeyCode::Home);
    assert!(
        rig.app.row(0).contains("p0"),
        "W15-04: Home must reveal the first root, got {:?}",
        rig.app.row(0)
    );
}

/// W15-05: a query shows matches with their ancestors; the chosen key
/// survives filter changes even when it is not visible.
#[test]
fn w15_tree_filter_preserves_ancestors_and_selection() {
    let mut rig = TreeRig::plain(vec![
        tree_entry("root", 0, NodeKind::Parent),
        tree_entry("src", 1, NodeKind::Parent),
        tree_entry("main.rs", 2, NodeKind::Leaf),
        tree_entry("lib.rs", 2, NodeKind::Leaf),
        tree_entry("docs", 1, NodeKind::Parent),
        tree_entry("guide.md", 2, NodeKind::Leaf),
    ]);
    assert!(rig.app.tab_to(TREE));
    let _ = rig.app.key(KeyCode::Char('*'));
    let _ = rig.app.key(KeyCode::Down);
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("main.rs")));
    let _ = rig.app.key(KeyCode::Char(' '));
    assert_eq!(rig.chosen.get(), Some(ItemKey::text("main.rs")));

    // Filter to .rs: matches plus ancestors, docs hidden.
    rig.filter.set(TreeFilter::Rs);
    rig.settle();
    assert_eq!(rig.visible_len.get(), 4, "W15-05: root+src+2 matches");
    assert!(rig.app.row(0).contains("root"));
    assert!(rig.app.row(1).contains("src"));
    assert!(rig.app.row(2).contains("main.rs"));
    assert!(rig.app.row(3).contains("lib.rs"));
    assert_eq!(
        rig.chosen.get(),
        Some(ItemKey::text("main.rs")),
        "W15-05: the chosen key must survive filtering"
    );
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("main.rs")),
        "W15-05: the visible cursor must not move"
    );

    // Switch the query: the hidden choice survives, the cursor reseeds.
    rig.filter.set(TreeFilter::Md);
    rig.settle();
    assert_eq!(rig.visible_len.get(), 3, "W15-05: root+docs+guide");
    assert!(
        rig.app.row(2).contains("guide.md"),
        "W15-05: the new matches must paint, got {:?}",
        rig.app.row(2)
    );
    assert_eq!(
        rig.chosen.get(),
        Some(ItemKey::text("main.rs")),
        "W15-05: the hidden choice must survive the query switch"
    );
    assert_ne!(
        rig.cursor.get(),
        Some(ItemKey::text("main.rs")),
        "W15-05: the cursor must leave the hidden row"
    );

    // Clear the query: everything returns, the choice is intact.
    rig.filter.set(TreeFilter::Off);
    rig.settle();
    assert_eq!(rig.visible_len.get(), 6);
    assert_eq!(rig.chosen.get(), Some(ItemKey::text("main.rs")));
}

// ---------------------------------------------------------------------------
// W16 Steps (termrock-navigation)
// ---------------------------------------------------------------------------

const STEPS: Id = Id::root("control.states.steps");

#[derive(Clone)]
struct StepEntry {
    key: String,
    label: String,
    state: StepState,
}

fn step_entry(key: &str, state: StepState) -> StepEntry {
    StepEntry {
        key: key.to_string(),
        label: key.to_string(),
        state,
    }
}

fn steps_key(t: &StepEntry) -> ItemKey {
    ItemKey::text(&t.key)
}

fn steps_row(t: &StepEntry, r: &mut RowUi<'_>) {
    r.label(&t.label);
}

fn steps_state(t: &StepEntry) -> StepState {
    t.state
}

struct StepsApp {
    items: Rc<RefCell<Vec<StepEntry>>>,
    st: StepsState,
    actions: Rc<RefCell<Vec<StepsAction>>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    offset: Rc<Cell<usize>>,
    area: Rc<Cell<Rect>>,
    navigable: bool,
}

impl StepsApp {
    fn rail(&self) -> Steps<'static, StepEntry, fn(&StepEntry) -> ItemKey, fn(&StepEntry, &mut RowUi<'_>)> {
        let base = if self.navigable {
            Steps::navigable(STEPS)
        } else {
            Steps::new(STEPS)
        };
        base.key(steps_key as fn(&StepEntry) -> ItemKey)
            .row(steps_row as fn(&StepEntry, &mut RowUi<'_>))
            .step(&steps_state)
    }
}

impl App for StepsApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let actions = Rc::clone(&self.actions);
        let borrowed = self.items.borrow();
        let r = self
            .rail()
            .update(cx, &mut self.st, &borrowed)
            .on_action(|action| actions.borrow_mut().push(action));
        self.cursor.set(self.st.cursor());
        self.offset.set(self.st.scroll().offset());
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let borrowed = self.items.borrow();
        self.rail().draw(ui, self.area.get(), &self.st, &borrowed);
    }
}

struct StepsRig {
    app: Harness<StepsApp>,
    items: Rc<RefCell<Vec<StepEntry>>>,
    actions: Rc<RefCell<Vec<StepsAction>>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    offset: Rc<Cell<usize>>,
    area: Rc<Cell<Rect>>,
}

impl StepsRig {
    fn with(items: Vec<StepEntry>, area: Rect, navigable: bool) -> Self {
        let items_rc = Rc::new(RefCell::new(items));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let cursor = Rc::new(Cell::new(None));
        let offset = Rc::new(Cell::new(0));
        let area_rc = Rc::new(Cell::new(area));
        let app = Harness::new(
            StepsApp {
                items: Rc::clone(&items_rc),
                st: StepsState::new(),
                actions: Rc::clone(&actions),
                cursor: Rc::clone(&cursor),
                offset: Rc::clone(&offset),
                area: Rc::clone(&area_rc),
                navigable,
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        Self {
            app,
            items: items_rc,
            actions,
            cursor,
            offset,
            area: area_rc,
        }
    }

    fn plain(items: Vec<StepEntry>) -> Self {
        Self::with(
            items,
            Rect {
                x: 0,
                y: 0,
                width: 40,
                height: 8,
            },
            true,
        )
    }

    fn activated(&self) -> Vec<ItemKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                StepsAction::Activated(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    fn states(&self) -> Vec<StepState> {
        self.items.borrow().iter().map(|t| t.state).collect()
    }

    fn settle(&mut self) {
        self.app.ticks(1);
        self.app.draw();
    }
}

/// W16-01: every lifecycle state paints its icon, label and state word;
/// the frontier is the first unfinished step.
#[test]
fn w16_steps_all_lifecycle_states() {
    let mut rig = StepsRig::plain(vec![
        step_entry("done", StepState::Done),
        step_entry("skip", StepState::Skipped),
        step_entry("run", StepState::Running),
        step_entry("queue", StepState::Queued),
        step_entry("fail", StepState::Failed),
        step_entry("block", StepState::Blocked),
    ]);
    assert!(rig.app.tab_to(STEPS));

    // Each state paints its icon cell.
    let icons: Vec<String> = (0..6).map(|y| cell_symbol(rig.app.buffer(), 1, y)).collect();
    assert_eq!(
        icons,
        vec!["✓", " ", "▪", " ", "!", "▲"],
        "W16-01: every state must paint its icon, got {icons:?}"
    );

    // Each row carries its label and its state word.
    for (y, (label, word)) in [
        ("done", "done"),
        ("skip", "skipped"),
        ("run", "running"),
        ("queue", "queued"),
        ("fail", "failed"),
        ("block", "blocked"),
    ]
    .iter()
    .enumerate()
    {
        let row = rig.app.row(y as u16);
        assert!(
            row.contains(label) && row.contains(word),
            "W16-01: row {y} must carry {label}/{word}, got {row:?}"
        );
    }

    // The frontier is the first unfinished step: the running one.
    let borrowed = rig.items.borrow();
    let rail = Steps::navigable(STEPS)
        .key(steps_key as fn(&StepEntry) -> ItemKey)
        .step(&steps_state);
    assert_eq!(
        rail.frontier(&borrowed),
        Some(ItemKey::text("run")),
        "W16-01: the frontier must be the first unfinished step"
    );
}

/// W16-02 parity record: the legacy rail paints a tick-driven ten-phase
/// braille spinner for the running step (`spinner_frame` in
/// `src/widgets/progress.rs` at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`,
/// used by `src/widgets/steps.rs`). The candidate paints a static Bullet.
#[test]
#[ignore = "PARITY W16-02: running step shows a static Bullet, reference cycles a 10-phase spinner"]
fn w16_steps_running_spinner_cycles_phases() {
    let mut rig = StepsRig::plain(vec![
        step_entry("run", StepState::Running),
        step_entry("queue", StepState::Queued),
    ]);
    assert!(rig.app.tab_to(STEPS));
    let mut phases = HashSet::new();
    for _ in 0..12 {
        let _ = rig.app.advance(Duration::from_millis(100));
        phases.insert(cell_symbol(rig.app.buffer(), 1, 0));
    }
    assert_eq!(
        phases.len(),
        10,
        "W16-02: the running step must cycle 10 spinner phases, got {phases:?}"
    );
}

/// W16-03: navigating to a failed step and activating it reports the
/// activation without touching the caller-owned lifecycle.
#[test]
fn w16_steps_navigate_failed_without_lifecycle_change() {
    let mut rig = StepsRig::plain(vec![
        step_entry("one", StepState::Done),
        step_entry("two", StepState::Failed),
        step_entry("three", StepState::Queued),
    ]);
    assert!(rig.app.tab_to(STEPS));
    let before = rig.states();

    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("two")));
    let _ = rig.app.key(KeyCode::Enter);
    assert_eq!(rig.activated(), vec![ItemKey::text("two")]);
    assert_eq!(
        rig.states(),
        before,
        "W16-03: navigation and activation must not mutate the lifecycle"
    );

    // A pointer activation is equally side-effect free.
    let _ = rig.app.click(10, 2);
    assert_eq!(
        rig.activated(),
        vec![ItemKey::text("two"), ItemKey::text("three")]
    );
    assert_eq!(
        rig.states(),
        before,
        "W16-03: pointer activation must not mutate the lifecycle"
    );
}

/// W16-04: long labels clip with an ellipsis, narrow widths drop the state
/// word before starving the label, and overflow scrolls by viewport.
#[test]
fn w16_steps_overflow_and_narrow_clip() {
    // A long label clips with an ellipsis while the state word survives.
    let mut long = step_entry("run", StepState::Running);
    long.label = "RunTheEntireTestSuiteTwiceForLuck".to_string();
    let mut rig = StepsRig::plain(vec![long]);
    assert!(rig.app.tab_to(STEPS));
    assert!(
        rig.app.row(0).contains('…'),
        "W16-04: the long label must clip, got {:?}",
        rig.app.row(0)
    );
    assert!(
        rig.app.row(0).contains("running"),
        "W16-04: the state word must survive, got {:?}",
        rig.app.row(0)
    );

    // Narrow: the state word drops before the label starves.
    rig.area.set(Rect {
        x: 0,
        y: 0,
        width: 10,
        height: 8,
    });
    rig.app.draw();
    assert!(
        !rig.app.row(0).contains("running"),
        "W16-04: narrow widths must drop the state word, got {:?}",
        rig.app.row(0)
    );
    assert!(
        rig.app.row(0).contains("Run"),
        "W16-04: the label must survive narrowing, got {:?}",
        rig.app.row(0)
    );

    // Overflow: PageDown moves by viewport and scrolls to the cursor.
    let items: Vec<StepEntry> = (0..12)
        .map(|n| step_entry(&format!("s{n:02}"), StepState::Queued))
        .collect();
    let mut rig = StepsRig::plain(items);
    assert!(rig.app.tab_to(STEPS));
    let _ = rig.app.key(KeyCode::PageDown);
    rig.settle();
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("s08")));
    assert!(
        rig.offset.get() > 0,
        "W16-04: the overflow rail must scroll to the cursor"
    );
    assert!(
        rig.app.row(7).contains("s08"),
        "W16-04: the cursor row must be visible, got {:?}",
        rig.app.row(7)
    );
}

/// W16-05: replacing the live source preserves the cursor key and picks up
/// the new labels and states.
#[test]
fn w16_steps_source_replacement_preserves_key() {
    let mut rig = StepsRig::plain(vec![
        step_entry("k1", StepState::Done),
        step_entry("k2", StepState::Running),
        step_entry("k3", StepState::Queued),
    ]);
    assert!(rig.app.tab_to(STEPS));
    let _ = rig.app.key(KeyCode::Down);
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("k2")));

    // Reorder, relabel and advance states under the cursor key.
    rig.items.replace(vec![
        StepEntry {
            key: "k2".to_string(),
            label: "k2-renamed".to_string(),
            state: StepState::Done,
        },
        step_entry("k3", StepState::Running),
        step_entry("k1", StepState::Done),
    ]);
    rig.settle();
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("k2")),
        "W16-05: the cursor key must survive replacement, got {:?}",
        rig.cursor.get()
    );
    assert!(
        rig.app.row(0).contains("k2-renamed"),
        "W16-05: the new label must paint, got {:?}",
        rig.app.row(0)
    );
    assert_eq!(
        cell_symbol(rig.app.buffer(), 1, 0),
        "✓",
        "W16-05: the new state must paint"
    );
}

// ---------------------------------------------------------------------------
// W17 Tabs (termrock-navigation)
// ---------------------------------------------------------------------------

const TABS: Id = Id::root("control.states.tabs");
const TABS_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 40,
    height: 2,
};

#[derive(Clone)]
struct TabEntry {
    key: String,
    label: String,
}

fn tab_entry(key: &str) -> TabEntry {
    TabEntry {
        key: key.to_string(),
        label: key.to_string(),
    }
}

fn tabs_key(t: &TabEntry) -> ItemKey {
    ItemKey::text(&t.key)
}

fn tabs_row(t: &TabEntry, r: &mut RowUi<'_>) {
    r.label(&t.label);
}

struct TabsApp {
    items: Rc<RefCell<Vec<TabEntry>>>,
    st: TabsState,
    actions: Rc<RefCell<Vec<TabsAction>>>,
    active: Rc<Cell<Option<ItemKey>>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    first: Rc<Cell<Option<ItemKey>>>,
    closable: bool,
    allow_new: bool,
}

impl App for TabsApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let actions = Rc::clone(&self.actions);
        let borrowed = self.items.borrow();
        let r = Tabs::new(TABS)
            .key(tabs_key as fn(&TabEntry) -> ItemKey)
            .row(tabs_row as fn(&TabEntry, &mut RowUi<'_>))
            .closable(self.closable)
            .allow_new(self.allow_new)
            .update(cx, &mut self.st, &borrowed)
            .on_action(|action| actions.borrow_mut().push(action));
        self.active.set(self.st.active());
        self.cursor.set(self.st.cursor());
        self.first.set(self.st.first());
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let borrowed = self.items.borrow();
        Tabs::new(TABS)
            .key(tabs_key as fn(&TabEntry) -> ItemKey)
            .row(tabs_row as fn(&TabEntry, &mut RowUi<'_>))
            .closable(self.closable)
            .allow_new(self.allow_new)
            .draw(ui, TABS_AREA, &self.st, &borrowed);
    }
}

struct TabsRig {
    app: Harness<TabsApp>,
    items: Rc<RefCell<Vec<TabEntry>>>,
    actions: Rc<RefCell<Vec<TabsAction>>>,
    active: Rc<Cell<Option<ItemKey>>>,
    cursor: Rc<Cell<Option<ItemKey>>>,
    first: Rc<Cell<Option<ItemKey>>>,
}

impl TabsRig {
    fn with(items: Vec<TabEntry>, closable: bool, allow_new: bool) -> Self {
        let items_rc = Rc::new(RefCell::new(items));
        let actions = Rc::new(RefCell::new(Vec::new()));
        let active = Rc::new(Cell::new(None));
        let cursor = Rc::new(Cell::new(None));
        let first = Rc::new(Cell::new(None));
        let app = Harness::new(
            TabsApp {
                items: Rc::clone(&items_rc),
                st: TabsState::default(),
                actions: Rc::clone(&actions),
                active: Rc::clone(&active),
                cursor: Rc::clone(&cursor),
                first: Rc::clone(&first),
                closable,
                allow_new,
            },
            Theme::junie(),
            SCREEN.width,
            SCREEN.height,
        );
        Self {
            app,
            items: items_rc,
            actions,
            active,
            cursor,
            first,
        }
    }

    fn plain(items: Vec<TabEntry>) -> Self {
        Self::with(items, false, false)
    }

    fn activated(&self) -> Vec<ItemKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                TabsAction::Activated(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    fn closed(&self) -> Vec<ItemKey> {
        self.actions
            .borrow()
            .iter()
            .filter_map(|a| match a {
                TabsAction::Close(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    fn settle(&mut self) {
        self.app.ticks(1);
        self.app.draw();
    }
}

/// W17-01: the active tab reads apart from inactive tabs, and the strip
/// resolves hovered, focused, pressed and close-hovered variants.
#[test]
fn w17_tabs_state_variants() {
    let mut rig = TabsRig::with(
        vec![tab_entry("A"), tab_entry("B"), tab_entry("C")],
        true,
        false,
    );
    assert!(rig.app.tab_to(TABS));
    assert_eq!(rig.active.get(), Some(ItemKey::text("A")));

    fn tab_style(app: &Harness<TabsApp>, x: u16, y: u16) -> (Color, Color) {
        let cell = &app.buffer()[Position::new(x, y)];
        (cell.fg, cell.bg)
    }

    // Tab geometry, closable single-char labels: A x0-5, B x7-12, C x14-19.
    assert_ne!(
        tab_style(&rig.app, 1, 0),
        tab_style(&rig.app, 8, 0),
        "W17-01: the active tab must read apart from inactive tabs"
    );

    // The strip itself tracks focus; per-tab focused/pressed rendering is
    // the `w17_tabs_focused_pressed_bold` parity record.
    assert!(
        rig.app.state_of(TABS).contains(StateFlags::FOCUSED),
        "W17-01: the strip must hold focus"
    );

    // Hovered: moving over B restyles B alone.
    let before = tab_style(&rig.app, 8, 0);
    let _ = rig.app.mouse(MouseKind::Move, 8, 0);
    rig.app.draw();
    assert_ne!(
        before,
        tab_style(&rig.app, 8, 0),
        "W17-01: hovering a tab must restyle it"
    );

    // Pressed: holding B moves the cursor to B.
    let _ = rig.app.mouse(MouseKind::Down, 8, 0);
    assert_eq!(
        rig.cursor.get(),
        Some(ItemKey::text("B")),
        "W17-01: pressing a tab must move the cursor"
    );
    let _ = rig.app.mouse(MouseKind::Up, 8, 0);

    // Close-hovered: hovering B's × restyles the close cell alone.
    let close_before = tab_style(&rig.app, 10, 0);
    let _ = rig.app.mouse(MouseKind::Move, 10, 0);
    rig.app.draw();
    assert_ne!(
        close_before,
        tab_style(&rig.app, 10, 0),
        "W17-01: hovering a close cell must restyle it"
    );
}

/// W17-01 parity record: the legacy strip bolds the keyboard cursor tab
/// ("bold marks the keyboard cursor" in `src/widgets/tabs.rs` at
/// `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`). The candidate resolves the
/// FOCUSED/PRESSED flags but the TABS recipe (`builtin/mod.rs`) defines no
/// rule for them, so focused and pressed tabs render as plain inactive.
#[test]
#[ignore = "PARITY W17-01: focused/pressed tabs render as inactive, reference bolds the cursor tab"]
fn w17_tabs_focused_pressed_bold() {
    let mut rig = TabsRig::with(
        vec![tab_entry("A"), tab_entry("B"), tab_entry("C")],
        true,
        false,
    );
    assert!(rig.app.tab_to(TABS));

    fn bold(app: &Harness<TabsApp>, x: u16, y: u16) -> bool {
        app.buffer()[Position::new(x, y)]
            .modifier
            .contains(Modifier::BOLD)
    }

    // Press-hold B: the cursor (still inactive) tab reads bold while the
    // strip is focused. The candidate sets FOCUSED|PRESSED without ACTIVE
    // and the recipe renders that as plain inactive.
    let _ = rig.app.mouse(MouseKind::Down, 8, 0);
    rig.app.draw();
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("B")));
    assert_eq!(rig.active.get(), Some(ItemKey::text("A")));
    assert!(
        bold(&rig.app, 8, 0),
        "W17-01: the focused cursor tab must read bold"
    );
    let _ = rig.app.mouse(MouseKind::Up, 8, 0);
}

/// W17-02: an overflowing strip scrolls its window to the active tab, and
/// renaming the active tab to a longer label keeps its key visible.
#[test]
fn w17_tabs_overflow_scroll_and_rename() {
    let items: Vec<TabEntry> = (0..10).map(|n| tab_entry(&format!("t{n}"))).collect();
    let mut rig = TabsRig::plain(items);
    assert!(rig.app.tab_to(TABS));
    assert_eq!(rig.first.get(), Some(ItemKey::text("t0")));

    // Right overflow counter paints while tabs hide on the right.
    assert!(
        rig.app.row(0).contains('›'),
        "W17-02: the right overflow counter must paint, got {:?}",
        rig.app.row(0)
    );

    // Activate t8: the window follows, the left counter appears.
    for _ in 0..8 {
        let _ = rig.app.key(KeyCode::Right);
    }
    assert_eq!(rig.active.get(), Some(ItemKey::text("t8")));
    assert_ne!(
        rig.first.get(),
        Some(ItemKey::text("t0")),
        "W17-02: the window must follow the active tab"
    );
    assert!(
        rig.app.row(0).contains('‹'),
        "W17-02: the left overflow counter must paint, got {:?}",
        rig.app.row(0)
    );
    assert!(
        rig.app.row(0).contains("t8"),
        "W17-02: the active tab must stay visible, got {:?}",
        rig.app.row(0)
    );

    // Rename the active tab longer: its key stays active and in-window.
    // The overlong tab clips with an ellipsis; the counters stay coherent.
    rig.items.borrow_mut()[8].label = "t8-with-a-much-longer-label".to_string();
    rig.settle();
    assert_eq!(
        rig.active.get(),
        Some(ItemKey::text("t8")),
        "W17-02: renaming must keep the active key"
    );
    assert!(
        rig.app.row(0).contains("t8-wit"),
        "W17-02: the renamed tab must stay in-window, got {:?}",
        rig.app.row(0)
    );
    assert!(
        rig.app.row(0).contains('‹') && rig.app.row(0).contains('›'),
        "W17-02: counters must stay coherent, got {:?}",
        rig.app.row(0)
    );
}

/// W17-03: deleting the active tab activates its successor, or the
/// predecessor at the end; the choice is deterministic by key.
#[test]
fn w17_tabs_delete_active_restores_successor() {
    let mut rig = TabsRig::with(
        vec![tab_entry("a"), tab_entry("b"), tab_entry("c")],
        true,
        false,
    );
    assert!(rig.app.tab_to(TABS));
    let _ = rig.app.key(KeyCode::Right);
    assert_eq!(rig.active.get(), Some(ItemKey::text("b")));

    // Close the middle tab through its close cell: the successor wins.
    let _ = rig
        .app
        .click_part(TABS, PartRef::item(Part::CLOSE, ItemKey::text("b")));
    assert_eq!(rig.closed(), vec![ItemKey::text("b")]);
    rig.items.borrow_mut().remove(1);
    rig.settle();
    assert_eq!(
        rig.active.get(),
        Some(ItemKey::text("c")),
        "W17-03: the successor must activate, got {:?}",
        rig.active.get()
    );

    // Close the last tab: the predecessor wins.
    let _ = rig
        .app
        .click_part(TABS, PartRef::item(Part::CLOSE, ItemKey::text("c")));
    assert_eq!(
        rig.closed(),
        vec![ItemKey::text("b"), ItemKey::text("c")]
    );
    rig.items.borrow_mut().remove(1);
    rig.settle();
    assert_eq!(
        rig.active.get(),
        Some(ItemKey::text("a")),
        "W17-03: the predecessor must activate at the end, got {:?}",
        rig.active.get()
    );
}

/// W17-04: close targets the logical tab across a reorder, and secondary
/// clicks are consumed without acting.
#[test]
fn w17_tabs_close_targets_logical_tab_across_reorder() {
    let mut rig = TabsRig::with(
        vec![tab_entry("a"), tab_entry("b"), tab_entry("c")],
        true,
        false,
    );
    assert!(rig.app.tab_to(TABS));

    // Swap two tabs under a stable window: the × still names the logical tab.
    rig.items.borrow_mut().swap(1, 2);
    rig.settle();
    let _ = rig
        .app
        .click_part(TABS, PartRef::item(Part::CLOSE, ItemKey::text("b")));
    assert_eq!(
        rig.closed(),
        vec![ItemKey::text("b")],
        "W17-04: close must target the logical tab, got {:?}",
        rig.closed()
    );

    // The x key closes the cursor tab by key, not by position.
    assert_eq!(rig.cursor.get(), Some(ItemKey::text("a")));
    let _ = rig.app.key(KeyCode::Char('x'));
    assert_eq!(
        rig.closed(),
        vec![ItemKey::text("b"), ItemKey::text("a")],
        "W17-04: x must close the cursor key, got {:?}",
        rig.closed()
    );

    // Secondary clicks are context gestures: consumed, never actions.
    let before = rig.actions.borrow().len();
    let _ = rig.app.mouse(MouseKind::Secondary, 8, 0);
    let _ = rig.app.mouse(MouseKind::SecondaryUp, 8, 0);
    assert_eq!(
        rig.actions.borrow().len(),
        before,
        "W17-04: secondary clicks must not act"
    );
}

const TAB_CHILD_A1: Id = Id::root("control.states.tabs.a1");
const TAB_CHILD_A2: Id = Id::root("control.states.tabs.a2");
const TAB_CHILD_B1: Id = Id::root("control.states.tabs.b1");
const TAB_CHILD_B2: Id = Id::root("control.states.tabs.b2");

/// Owner-side per-tab focus memory: the strip reports activation, the owner
/// remembers each tab's child and restores it on the next switch.
struct TabsChildrenApp {
    items: Rc<RefCell<Vec<TabEntry>>>,
    st: TabsState,
    active: Rc<Cell<Option<ItemKey>>>,
    memory: Rc<RefCell<HashMap<ItemKey, Id>>>,
    pending: Rc<Cell<Option<ItemKey>>>,
}

fn tab_children(tab: ItemKey) -> [Id; 2] {
    if tab == ItemKey::text("A") {
        [TAB_CHILD_A1, TAB_CHILD_A2]
    } else {
        [TAB_CHILD_B1, TAB_CHILD_B2]
    }
}

impl App for TabsChildrenApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        // Restore the pending tab's remembered (or first) child: it was
        // registered by the previous draw, so the focus request lands.
        if let Some(tab) = self.pending.take() {
            let child = self
                .memory
                .borrow()
                .get(&tab)
                .copied()
                .unwrap_or(tab_children(tab)[0]);
            cx.focus(child);
        }
        let borrowed = self.items.borrow();
        let pending = Rc::clone(&self.pending);
        let r = Tabs::new(TABS)
            .key(tabs_key as fn(&TabEntry) -> ItemKey)
            .row(tabs_row as fn(&TabEntry, &mut RowUi<'_>))
            .update(cx, &mut self.st, &borrowed)
            .on_action(|action| {
                if let TabsAction::Activated(k) = action {
                    pending.set(Some(k));
                }
            });
        self.active.set(self.st.active());
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let borrowed = self.items.borrow();
        Tabs::new(TABS)
            .key(tabs_key as fn(&TabEntry) -> ItemKey)
            .row(tabs_row as fn(&TabEntry, &mut RowUi<'_>))
            .draw(ui, TABS_AREA, &self.st, &borrowed);
        // Only the active tab's children exist; record the focused one.
        if let Some(active) = self.active.get() {
            for (i, child) in tab_children(active).iter().enumerate() {
                let rect = Rect::new(0, 3 + i as u16, 10, 1);
                ui.register_control(*child, rect, Focusability::Focusable);
                if ui.state(*child).contains(StateFlags::FOCUSED) {
                    self.memory.borrow_mut().insert(active, *child);
                }
            }
        }
    }
}

/// W17-05: switching tabs restores each tab's remembered child focus.
#[test]
fn w17_tabs_restore_child_focus_per_tab() {
    let memory = Rc::new(RefCell::new(HashMap::new()));
    let pending = Rc::new(Cell::new(None));
    let active = Rc::new(Cell::new(None));
    let mut app = Harness::new(
        TabsChildrenApp {
            items: Rc::new(RefCell::new(vec![tab_entry("A"), tab_entry("B")])),
            st: TabsState::default(),
            active: Rc::clone(&active),
            memory: Rc::clone(&memory),
            pending: Rc::clone(&pending),
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );
    assert!(app.tab_to(TABS));
    assert_eq!(active.get(), Some(ItemKey::text("A")));

    // Focus A's second child, then switch to B: B starts on its first.
    let _ = app.key(KeyCode::Tab);
    let _ = app.key(KeyCode::Tab);
    assert!(
        app.state_of(TAB_CHILD_A2).contains(StateFlags::FOCUSED),
        "W17-05: A2 must focus"
    );
    assert!(app.tab_to(TABS));
    let _ = app.key(KeyCode::Right);
    app.ticks(1);
    app.draw();
    assert_eq!(active.get(), Some(ItemKey::text("B")));
    assert!(
        app.state_of(TAB_CHILD_B1).contains(StateFlags::FOCUSED),
        "W17-05: B must start on its first child"
    );

    // Focus B's second child, then switch back: A2 returns.
    let _ = app.key(KeyCode::Tab);
    assert!(
        app.state_of(TAB_CHILD_B2).contains(StateFlags::FOCUSED),
        "W17-05: B2 must focus"
    );
    assert!(app.tab_to(TABS));
    let _ = app.key(KeyCode::Left);
    app.ticks(1);
    app.draw();
    assert_eq!(active.get(), Some(ItemKey::text("A")));
    assert!(
        app.state_of(TAB_CHILD_A2).contains(StateFlags::FOCUSED),
        "W17-05: switching back must restore A2"
    );

    // And B2 returns symmetrically.
    assert!(app.tab_to(TABS));
    let _ = app.key(KeyCode::Right);
    app.ticks(1);
    app.draw();
    assert!(
        app.state_of(TAB_CHILD_B2).contains(StateFlags::FOCUSED),
        "W17-05: switching back must restore B2"
    );
}
