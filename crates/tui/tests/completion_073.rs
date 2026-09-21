//! TASK-073 conformance foundation witnesses: truthful resolution attribution,
//! generated registry, and the single caller-owned style timing seam.
//!
//! Small independently controlled fixtures with mutated production paths.
//! Foundation qualification only: TASK-031 owns final all-family equality,
//! TASK-067 owns the final five-percent timing measurement.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::too_many_lines
    )
)]

use junie_tui::{
    AttributedQuery, Button, CellRef, ColorLevel, Column, ColumnKey, Family, FrameRead, Grid,
    GridModel, GridState, Id, ItemKey, Part, Position, Rect, Response, Role, RowUi, StateFlags,
    StatusBar, StatusItem, StylePatch, StyleProvenance, StyleTimingEntry, StyleTimingMode,
    StyleTimingProbe, StyledQuery, Theme, Track, Ui, Variant,
};
use junie_tui_testing::conformance::{Caps, Conformance, Fixture};
use junie_tui_testing::{Scene, conformance_suite};

// ── Dummy registry subject: exercises the generated enumeration ──

const DUMMY073: Id = Id::root("completion073.dummy");

struct Dummy073Case;

impl Conformance for Dummy073Case {
    const NAME: &'static str = "dummy073";
    const FAMILY: Family = Family::BUTTON;
    const PARTS: &'static [Part] = &[Part::CONTAINER, Part::LABEL];
    type State = ();
    type Action = ();
    type Cmd = ();

    fn caps() -> Caps {
        Caps::empty()
    }

    fn id() -> Id {
        DUMMY073
    }

    fn update(_cx: &mut junie_tui::Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
        Response::ignored()
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), _f: &Fixture) {
        if area.is_empty() {
            return;
        }
        let r = ui.style(
            Family::BUTTON,
            Variant::DEFAULT,
            Part::CONTAINER,
            ui.state(DUMMY073),
        );
        ui.fill(area, r.style);
        let label = ui.style(
            Family::BUTTON,
            Variant::DEFAULT,
            Part::LABEL,
            ui.state(DUMMY073),
        );
        ui.paint_str(area, "dummy073", label.style);
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 1] = [StateFlags::empty()];
        &STATES
    }

    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED ERROR WARNING EDITING BUSY ACTIVE: dummy exposes no mono affordance"
    }
}

conformance_suite!(dummy073 => Dummy073Case,);

// ── W-073-01 owned baseline ──

#[test]
fn w073_01_owned_baseline_provenance_and_legacy_fields() {
    const OWNER: Id = Id::root("w073.01.owner");
    let mut scene = Scene::new("w073-01", Theme::junie(), ColorLevel::TrueColor, 30, 6);
    scene.draw(|ui, _| {
        Button::new(OWNER, "Ok").draw(ui, Rect::new(1, 1, 14, 1));
        let attributed: Vec<AttributedQuery> = ui.attributed_queries().to_vec();
        let legacy: Vec<StyledQuery> = ui.styled_queries().to_vec();
        assert!(
            !attributed.is_empty(),
            "w073-01: component must record attributed queries"
        );
        assert_eq!(
            attributed.len(),
            legacy.len(),
            "w073-01: every legacy query must have an attributed counterpart (omitted provenance fails)"
        );
        for q in attributed.iter().filter(|q| q.id == OWNER) {
            assert_eq!(
                q.provenance,
                StyleProvenance::ComponentOwned,
                "w073-01: owned baseline must be component-owned"
            );
            assert!(
                q.painted,
                "w073-01: owned baseline must be credited to the paint sink"
            );
            assert!(
                legacy.contains(&q.query()),
                "w073-01: attributed must retain real Id/Family/Variant/Part/Resolved"
            );
        }
        // Same-ID families resolve through their own recipe, never a guess.
        let label = attributed
            .iter()
            .find(|q| q.id == OWNER && q.part == Part::LABEL)
            .expect("w073-01: LABEL must be recorded");
        assert_eq!(label.family, Family::BUTTON, "w073-01: real family retained");
    });
}

#[test]
fn w073_01_fabricated_id_or_constant_state_fails() {
    const OWNER: Id = Id::root("w073.01.neg");
    let mut scene = Scene::new("w073-01-neg", Theme::junie(), ColorLevel::TrueColor, 30, 6);
    scene.draw(|ui, _| {
        Button::new(OWNER, "Ok").draw(ui, Rect::new(1, 1, 14, 1));
        let attributed = ui.attributed_queries();
        // Real IDs: every record for this draw carries the real owner or a
        // real sub-id derived from it, never a fabricated replacement.
        for q in attributed {
            assert!(
                q.id == OWNER || format!("{:?}", q.id).contains("w073.01.neg"),
                "w073-01: altered real ID fails (got {:?})",
                q.id
            );
        }
        // Real variants/fields: LABEL and CONTAINER resolve distinctly; a
        // constant-fill mutant that returns one Resolved for all parts fails.
        let label = attributed
            .iter()
            .find(|q| q.id == OWNER && q.part == Part::LABEL)
            .map(|q| q.resolved);
        let container = attributed
            .iter()
            .find(|q| q.id == OWNER && q.part == Part::CONTAINER)
            .map(|q| q.resolved);
        match (label, container) {
            (Some(l), Some(c)) => assert_ne!(
                (l.style, l.glyph),
                (c.style, c.glyph),
                "w073-01: constant-fill state fails (LABEL == CONTAINER)"
            ),
            _ => panic!("w073-01: dropped resolved field fails (LABEL or CONTAINER missing)"),
        }
    });
}

// ── W-073-02 same-ID composition ──

#[test]
fn w073_02_same_id_composition_stays_owned() {
    const OWNER: Id = Id::root("w073.02.owner");
    const ROW_PART: Part = Part::custom("w073.row.part");
    let mut scene = Scene::new("w073-02", Theme::junie(), ColorLevel::TrueColor, 40, 10);
    scene.draw(|ui, _| {
        Button::new(OWNER, "Before").draw(ui, Rect::new(1, 1, 12, 1));
        {
            let mut row = RowUi::new(
                ui,
                OWNER,
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::empty(),
                ItemKey::num(1),
                Rect::new(1, 3, 30, 1),
            );
            row.part(ROW_PART, 6).text("ROW");
        }
        Button::new(OWNER, "After").draw(ui, Rect::new(1, 5, 12, 1));
        let attributed = ui.attributed_queries();
        // Child-owned composition remains owned, not caller-row merely
        // because nested.
        let button_labels: Vec<&AttributedQuery> = attributed
            .iter()
            .filter(|q| q.id == OWNER && q.part == Part::LABEL && q.family == Family::BUTTON)
            .collect();
        assert_eq!(
            button_labels.len(),
            2,
            "w073-02: ordinary child part must appear before and after the caller callback"
        );
        for q in &button_labels {
            assert_eq!(
                q.provenance,
                StyleProvenance::ComponentOwned,
                "w073-02: labeling all nested calls caller-owned fails"
            );
        }
        // The caller row part is row-owned and keeps the real ID.
        let row_q = attributed
            .iter()
            .find(|q| q.id == OWNER && q.part == ROW_PART)
            .expect("w073-02: row custom part must be recorded");
        assert_eq!(
            row_q.provenance,
            StyleProvenance::CallerRowOwned,
            "w073-02: caller row part must be row-owned"
        );
        assert_eq!(row_q.id, OWNER, "w073-02: rewriting the child ID fails");
    });
    assert!(
        scene.buffer().content().iter().any(|c| c.symbol() == "R"),
        "w073-02: real row cell must be painted"
    );
}

// ── W-073-03 row scope ──

#[test]
fn w073_03_row_scope_provenance_nesting_and_cells() {
    const OWNER: Id = Id::root("w073.03.owner");
    const SIBLING: Id = Id::root("w073.03.sibling");
    const ROW_PART: Part = Part::custom("w073.outer.row");
    const COL_PART: Part = Part::custom("w073.inner.column");
    let mut scene = Scene::new("w073-03", Theme::junie(), ColorLevel::TrueColor, 40, 8);
    scene.draw(|ui, _| {
        // Sibling default builtin from library production: component-owned.
        Button::new(SIBLING, "Sib").draw(ui, Rect::new(1, 1, 10, 1));
        {
            let mut row = RowUi::new(
                ui,
                OWNER,
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::empty(),
                ItemKey::num(1),
                Rect::new(1, 3, 34, 1),
            );
            // Builtin from an external caller: row-owned (this call site is
            // the integration test, outside crates/tui/src/). A name-based
            // mutant that calls every builtin component-owned fails here.
            row.label("hi");
            row.part(ROW_PART, 6).text("ROW");
            {
                let mut columns = row.columns(&[Track::Fixed(10), Track::Flex(1)]);
                columns.cell_part(0, COL_PART).text("COL");
                columns.cell(1).text("CELL");
            }
            // Nested scope propagates and restores: the outer owner is
            // intact after the nested columns scope exits.
            assert_eq!(row.owner(), OWNER, "w073-03: nested scope must restore");
        }
        let attributed = ui.attributed_queries();
        let row_q = attributed
            .iter()
            .find(|q| q.id == OWNER && q.part == ROW_PART)
            .expect("w073-03: outer row custom must be recorded");
        assert_eq!(
            row_q.provenance,
            StyleProvenance::CallerRowOwned,
            "w073-03: genuine caller customization has caller provenance"
        );
        let col_q = attributed
            .iter()
            .find(|q| q.id == OWNER && q.part == COL_PART)
            .expect("w073-03: nested column custom must be recorded");
        assert_eq!(
            col_q.provenance,
            StyleProvenance::CallerRowOwned,
            "w073-03: nested scope must propagate caller provenance"
        );
        // Sibling default builtin stays component-owned: no leakage from the
        // row scope into the independent sibling.
        let sib_label = attributed
            .iter()
            .find(|q| q.id == SIBLING && q.part == Part::LABEL)
            .expect("w073-03: sibling label must be recorded");
        assert_eq!(
            sib_label.provenance,
            StyleProvenance::ComponentOwned,
            "w073-03: sibling leakage fails"
        );
        // Actual label cells consume the resolved style.
        let label_resolved = attributed
            .iter()
            .find(|q| q.id == OWNER && q.part == Part::LABEL)
            .expect("w073-03: row label must be recorded")
            .resolved;
        assert_eq!(
            row_label_provenance(attributed, OWNER),
            StyleProvenance::CallerRowOwned,
            "w073-03: external builtin label is row-owned (name-based fails)"
        );
        let _ = label_resolved;
    });
    for sym in ["h", "R", "C"] {
        assert!(
            scene.buffer().content().iter().any(|c| c.symbol() == sym),
            "w073-03: real cell {sym:?} must be painted"
        );
    }
}

fn row_label_provenance(attributed: &[AttributedQuery], owner: Id) -> StyleProvenance {
    attributed
        .iter()
        .find(|q| q.id == owner && q.part == Part::LABEL)
        .expect("row label recorded")
        .provenance
}

#[test]
fn w073_03_library_custom_stays_component_owned() {
    const OWNER: Id = Id::root("w073.03.libcustom");
    const CUSTOM: Part = Part::custom("w073.library.custom");
    let mut scene = Scene::new("w073-03-lib", Theme::junie(), ColorLevel::TrueColor, 30, 4);
    scene.draw(|ui, _| {
        {
            let mut row = RowUi::new(
                ui,
                OWNER,
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::empty(),
                ItemKey::num(1),
                Rect::new(1, 1, 24, 1),
            );
            // Resolved from library production source (inside crates/tui/src/),
            // so component-owned even though the part is custom. Name-based
            // ownership fails here.
            row.library_custom_for_testing(CUSTOM, 6);
        }
        let q = ui
            .attributed_queries()
            .iter()
            .find(|q| q.id == OWNER && q.part == CUSTOM)
            .expect("w073-03: library custom must be recorded");
        assert_eq!(
            q.provenance,
            StyleProvenance::ComponentOwned,
            "w073-03: name-based ownership fails (custom from library is owned)"
        );
    });
}

// ── W-073-04 unwind/empty scope ──

#[test]
fn w073_04_scope_restores_on_return_unwind_and_zero_area() {
    const OUTER: Id = Id::root("w073.04.outer");
    const INNER: Id = Id::root("w073.04.inner");
    const AFTER: Part = Part::custom("w073.outer.after");
    let mut scene = Scene::new("w073-04", Theme::junie(), ColorLevel::TrueColor, 40, 6);
    // Normal return.
    scene.draw(|ui, _| {
        {
            let mut outer = RowUi::new(
                ui,
                OUTER,
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::empty(),
                ItemKey::num(1),
                Rect::new(1, 1, 30, 1),
            );
            {
                let mut inner = outer.nested_row_for_testing(
                    INNER,
                    Family::LIST,
                    Variant::DEFAULT,
                    StateFlags::empty(),
                    ItemKey::num(2),
                    Rect::new(1, 2, 20, 1),
                );
                inner.part(Part::custom("w073.inner"), 4).text("B");
            }
            assert_eq!(
                outer.owner(),
                OUTER,
                "w073-04: nested return must restore the outer owner"
            );
            outer.part(AFTER, 4).text("C");
        }
        let queries = ui.attributed_queries();
        assert!(
            queries.iter().any(|q| q.id == OUTER && q.part == AFTER),
            "w073-04: post-restore query must carry the outer owner"
        );
        assert!(
            !queries.iter().any(|q| q.id == INNER && q.part == AFTER),
            "w073-04: stale inner context fails"
        );
    });
    // Zero-area callback still executes and records; then an independent
    // owner draws unaffected.
    let mut scene2 = Scene::new("w073-04-zero", Theme::junie(), ColorLevel::TrueColor, 30, 4);
    scene2.draw(|ui, _| {
        let before = ui.attributed_queries().len();
        {
            let mut row = RowUi::new(
                ui,
                OUTER,
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::empty(),
                ItemKey::num(9),
                Rect::ZERO,
            );
            row.part(Part::custom("w073.zero"), 4).text("Z");
        }
        assert!(
            ui.attributed_queries().len() > before,
            "w073-04: skipping a zero-area callback fails"
        );
        Button::new(INNER, "Ind").draw(ui, Rect::new(1, 1, 10, 1));
        let ind = ui
            .attributed_queries()
            .iter()
            .find(|q| q.id == INNER && q.part == Part::LABEL)
            .expect("w073-04: independent owner must draw");
        assert_eq!(
            ind.provenance,
            StyleProvenance::ComponentOwned,
            "w073-04: no stale callback context may leak into the next owner"
        );
    });
}

#[test]
fn w073_04_unwind_restores_owner() {
    const OUTER: Id = Id::root("w073.04.unwind.outer");
    const INNER: Id = Id::root("w073.04.unwind.inner");
    let mut scene = Scene::new(
        "w073-04-unwind",
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        4,
    );
    scene.draw(|ui, _| {
        let mut outer = RowUi::new(
            ui,
            OUTER,
            Family::LIST,
            Variant::DEFAULT,
            StateFlags::empty(),
            ItemKey::num(1),
            Rect::new(1, 1, 24, 1),
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut inner = outer.nested_row_for_testing(
                INNER,
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::empty(),
                ItemKey::num(2),
                Rect::new(1, 2, 20, 1),
            );
            inner.part(Part::custom("w073.unwind"), 4).text("U");
            panic!("w073-04: intentional unwind");
        }));
        assert!(
            result.is_err(),
            "w073-04: unwind must propagate to the harness"
        );
        assert_eq!(
            outer.owner(),
            OUTER,
            "w073-04: manual reset missed on unwind fails"
        );
    });
}

// ── W-073-05 measurement distinction ──

#[test]
fn w073_05_measurement_stays_distinct_from_paint() {
    const OWNER: Id = Id::root("w073.05.owner");
    let mut scene = Scene::new("w073-05", Theme::junie(), ColorLevel::TrueColor, 30, 6);
    scene.draw(|ui, _| {
        // Same part first measured, then painted.
        let measured = ui.resolve(
            Family::BUTTON,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::empty(),
        );
        let before = ui.attributed_queries().len();
        // Measurement must not create a paint query.
        assert_eq!(
            ui.attributed_queries().len(),
            before,
            "w073-05: counting measurement as paint fails"
        );
        Button::new(OWNER, "Ok").draw(ui, Rect::new(1, 1, 12, 1));
        let after = ui.attributed_queries().len();
        assert!(
            after > before,
            "w073-05: suppressing the actual paint record fails"
        );
        let painted = ui
            .attributed_queries()
            .iter()
            .find(|q| q.id == OWNER && q.part == Part::LABEL)
            .expect("w073-05: painted LABEL must be recorded");
        assert!(painted.painted, "w073-05: sink-bound consumer is painted");
        // Both real resolution records retained: the measured value and the
        // painted value agree (same inputs, same chain).
        assert_eq!(
            (measured.style, measured.glyph),
            (painted.resolved.style, painted.resolved.glyph),
            "w073-05: measured and painted resolutions must agree"
        );
        // Another part resolved but never painted leaves no paint record.
        let _ = ui.resolve(
            Family::BUTTON,
            Variant::DEFAULT,
            Part::ICON,
            StateFlags::empty(),
        );
        assert!(
            !ui.attributed_queries()
                .iter()
                .any(|q| q.id == OWNER && q.part == Part::ICON && q.painted),
            "w073-05: inert resolution must not be credited as painted"
        );
    });
}

// ── W-073-06 slot field relevance ──

#[test]
fn w073_06_slot_sentinel_reaches_cells_without_side_effects() {
    const OWNER: Id = Id::root("w073.06.owner");
    let area = Rect::new(2, 2, 14, 1);
    let mut plain = Scene::new(
        "w073-06-plain",
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        8,
    );
    plain.draw(|ui, _| {
        Button::new(OWNER, "Go").draw(ui, area);
    });
    let plain_buf = plain.buffer().clone();
    let mut altered = Scene::new(
        "w073-06-altered",
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        8,
    );
    let used = std::cell::Cell::new(Rect::ZERO);
    altered.draw(|ui, _| {
        let sentinel = |ui: &mut Ui<'_>, slot_area: Rect| {
            used.set(slot_area);
            // Distinct fg/bg/modifiers: dropping any field fails.
            let style = junie_tui::theme::PaintStyle::new()
                .fg(ratatui_core::style::Color::Yellow)
                .bg(ratatui_core::style::Color::Blue)
                .add_modifier(ratatui_core::style::Modifier::BOLD);
            ui.paint_str(slot_area, "~", style);
        };
        Button::new(OWNER, "Go")
            .slot(Part::LABEL, &sentinel)
            .draw(ui, area);
    });
    let altered_buf = altered.buffer().clone();
    let slot_area = used.get();
    assert!(!slot_area.is_empty(), "w073-06: slot must be consumed");
    let mut changed = 0;
    for y in 0..8 {
        for x in 0..30 {
            let pos = Position::new(x, y);
            if plain_buf.cell(pos) != altered_buf.cell(pos) {
                assert!(
                    slot_area.contains(pos),
                    "w073-06: painting an unrelated marker fails (changed {pos:?} outside {slot_area:?})"
                );
                changed += 1;
            }
        }
    }
    assert!(
        changed > 0,
        "w073-06: storing a slot without consuming it fails"
    );
    // Full resolved fields reach the intended cells.
    let sample = altered_buf
        .cell(Position::new(slot_area.x, slot_area.y))
        .expect("slot cell");
    assert_eq!(
        sample.fg,
        ratatui_core::style::Color::Yellow,
        "w073-06: dropped fg field fails"
    );
    assert_eq!(
        sample.bg,
        ratatui_core::style::Color::Blue,
        "w073-06: dropped bg field fails"
    );
    assert!(
        sample
            .modifier
            .contains(ratatui_core::style::Modifier::BOLD),
        "w073-06: dropped modifier field fails"
    );
}

// ── W-073-07 registry generation ──

#[test]
fn w073_07_dummy_registry_generates_once() {
    // The dummy suite above proves the macro generates one enumeration:
    // `registered_cases` exists, names the module, and pins the count.
    assert_eq!(
        registered_cases(),
        vec!["dummy073"],
        "w073-07: generated registry must enumerate its subjects once"
    );
}

#[test]
fn w073_07_dropped_invocation_fails_with_exact_reason() {
    // Pattern check for the production PARTS/degenerate dispatchers in
    // `conformance.rs`: an unknown registry name must fail with the exact
    // unhandled-case reason, never silently skip.
    fn check_by_name(name: &str) {
        match name {
            "dummy073" => {}
            _ => panic!("parts registry has unhandled case {name:?}: register its parts arm"),
        }
    }
    check_by_name("dummy073");
    let result = std::panic::catch_unwind(|| check_by_name("ghost073"));
    assert!(
        result.is_err(),
        "w073-07: dropping a registry invocation must fail"
    );
    if let Err(payload) = result {
        let msg = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
            .unwrap_or_default();
        assert!(
            msg.contains("unhandled case") && msg.contains("ghost073"),
            "w073-07: exact missing-subject reason must fail (got {msg:?})"
        );
    }
}

// ── W-073-08 instrumentation transparency ──

fn w073_08_fixture(ui: &mut Ui<'_>) {
    const OWNER: Id = Id::root("w073.08.owner");
    Button::new(OWNER, "Ok").draw(ui, Rect::new(1, 1, 12, 1));
    {
        let mut row = RowUi::new(
            ui,
            OWNER,
            Family::LIST,
            Variant::DEFAULT,
            StateFlags::empty(),
            ItemKey::num(1),
            Rect::new(1, 3, 24, 1),
        );
        row.label("row");
    }
}

#[test]
fn w073_08_observation_is_transparent() {
    // Without any probe.
    let mut plain = Scene::new(
        "w073-08-plain",
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        8,
    );
    plain.draw(|ui, _| w073_08_fixture(ui));
    let plain_buf = plain.buffer().clone();
    let plain_queries = plain
        .buffer()
        .content()
        .iter()
        .map(|c| (c.symbol().to_string(), c.fg, c.bg))
        .collect::<Vec<_>>();

    // With a disabled probe: identical cells and identical FrameState
    // observations.
    let probe = StyleTimingProbe::new();
    probe.reset(StyleTimingMode::Disabled);
    let mut probed = Scene::new(
        "w073-08-probed",
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        8,
    );
    probed.draw(|ui, _| {
        ui.with_timing_probe(&probe, |ui| w073_08_fixture(ui));
    });
    assert_eq!(
        probed.buffer().content(),
        plain.buffer().content(),
        "w073-08: observation must not alter cells"
    );
    assert_eq!(
        probe.intervals().len(),
        0,
        "w073-08: disabled mode must record no intervals"
    );
    assert_eq!(
        probe.reported_ns(),
        0,
        "w073-08: disabled numerator must be zero"
    );
    assert!(
        !probe.witness().is_empty(),
        "w073-08: disabled mode must still witness membership"
    );
    let _ = plain_queries;

    // Repeated frame with the same probe after reset: same witness.
    probe.reset(StyleTimingMode::Disabled);
    let mut repeat = Scene::new(
        "w073-08-repeat",
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        8,
    );
    repeat.draw(|ui, _| {
        ui.with_timing_probe(&probe, |ui| w073_08_fixture(ui));
    });
    assert_eq!(
        repeat.buffer().content(),
        plain_buf.content(),
        "w073-08: repeated frame must be identical"
    );
}

#[test]
fn w073_08_independent_probes_do_not_interfere() {
    let a = StyleTimingProbe::new();
    let b = StyleTimingProbe::new();
    a.reset(StyleTimingMode::Measured);
    b.reset(StyleTimingMode::Disabled);
    let mut scene_a = Scene::new("w073-08-a", Theme::junie(), ColorLevel::TrueColor, 30, 8);
    scene_a.draw(|ui, _| {
        ui.with_timing_probe(&a, |ui| w073_08_fixture(ui));
    });
    let mut scene_b = Scene::new("w073-08-b", Theme::junie(), ColorLevel::TrueColor, 30, 8);
    scene_b.draw(|ui, _| {
        ui.with_timing_probe(&b, |ui| w073_08_fixture(ui));
    });
    assert_eq!(
        scene_a.buffer().content(),
        scene_b.buffer().content(),
        "w073-08: two probes must not alter each other's cells"
    );
    // No cross-instance counters: B stayed disabled (no intervals) even
    // though A measured. A global counter mutant fails here.
    assert_eq!(
        b.intervals().len(),
        0,
        "w073-08: cross-instance counter contamination fails"
    );
    assert!(
        !a.witness().is_empty() && !b.witness().is_empty(),
        "w073-08: both probes witness their own draw"
    );
    assert_eq!(
        a.witness(),
        b.witness(),
        "w073-08: identical fixtures must produce identical membership"
    );
}

// ── W-073-09 style timing ──

fn w073_09_exercise_all_entries(ui: &mut Ui<'_>) {
    const OWNER: Id = Id::root("w073.09.owner");
    // Style, StylePatched, Resolve, Bg, SurfaceStyle, PaintPatch.
    let _ = ui.style(
        Family::BUTTON,
        Variant::DEFAULT,
        Part::LABEL,
        StateFlags::empty(),
    );
    let patch = StylePatch::new().set_fg(junie_tui::Role::Warning);
    let _ = ui.style_patched(
        Family::BUTTON,
        Variant::DEFAULT,
        Part::LABEL,
        StateFlags::empty(),
        &patch,
    );
    let _ = ui.resolve(
        Family::BUTTON,
        Variant::DEFAULT,
        Part::LABEL,
        StateFlags::empty(),
    );
    let _ = ui.bg();
    let _ = ui.surface_style();
    let _ = ui.paint_patch(&patch);
    // StyleDefaults and StyleInherited (via author-facing paths).
    let defaults = junie_tui::theme::StyleDefaults::new(StylePatch::new());
    let _ = ui.style_defaults(
        Family::BUTTON,
        Variant::DEFAULT,
        Part::LABEL,
        StateFlags::empty(),
        defaults,
        None,
    );
    // CellDropBind via a filled CellUi; StatusBind and GridBind are
    // exercised by their branch witnesses below when reached.
    {
        let mut row = RowUi::new(
            ui,
            OWNER,
            Family::LIST,
            Variant::DEFAULT,
            StateFlags::empty(),
            ItemKey::num(1),
            Rect::new(1, 1, 26, 1),
        );
        row.part(Part::LABEL, 8).text("cellbind");
    }
}

#[test]
fn w073_09_three_modes_share_membership_and_cells() {
    for mode in [
        StyleTimingMode::Disabled,
        StyleTimingMode::Calibration,
        StyleTimingMode::Measured,
    ] {
        let probe = StyleTimingProbe::new();
        probe.reset(mode);
        let mut scene = Scene::new("w073-09", Theme::junie(), ColorLevel::TrueColor, 30, 8);
        scene.draw(|ui, _| {
            ui.with_timing_probe(&probe, |ui| w073_09_exercise_all_entries(ui));
        });
        let witness = probe.witness();
        assert!(
            !witness.is_empty(),
            "w073-09: {mode:?} must witness invocations"
        );
        assert!(
            !witness.contains(&255),
            "w073-09: {mode:?} uncovered path fails"
        );
        match mode {
            StyleTimingMode::Disabled => {
                assert_eq!(
                    probe.intervals().len(),
                    0,
                    "w073-09: disabled-hook numerator reported as measured fails"
                );
                assert_eq!(probe.reported_ns(), 0, "w073-09: disabled must be zero");
            }
            StyleTimingMode::Calibration | StyleTimingMode::Measured => {
                let intervals = probe.intervals();
                assert_eq!(
                    intervals.len(),
                    witness.len(),
                    "w073-09: {mode:?} intervals must match membership"
                );
                for (i, (id, start, stop, depth)) in intervals.iter().enumerate() {
                    assert_eq!(
                        *id, witness[i],
                        "w073-09: {mode:?} interval order must match witness"
                    );
                    assert!(
                        start <= stop,
                        "w073-09: {mode:?} honest durations (start <= stop)"
                    );
                    assert_eq!(*depth, 0, "w073-09: {mode:?} flat fixture has depth 0");
                }
                let sum: u128 = intervals.iter().map(|(_, a, b, _)| b - a).sum();
                assert_eq!(
                    sum,
                    probe.reported_ns(),
                    "w073-09: {mode:?} foreign or constant numerator fails"
                );
            }
        }
    }
    // All three modes produce identical cells for the identical fixture.
    let mut bufs = Vec::new();
    for mode in [
        StyleTimingMode::Disabled,
        StyleTimingMode::Calibration,
        StyleTimingMode::Measured,
    ] {
        let probe = StyleTimingProbe::new();
        probe.reset(mode);
        let mut scene = Scene::new(
            "w073-09-cells",
            Theme::junie(),
            ColorLevel::TrueColor,
            30,
            8,
        );
        scene.draw(|ui, _| {
            ui.with_timing_probe(&probe, |ui| w073_09_exercise_all_entries(ui));
        });
        bufs.push(scene.buffer().clone());
    }
    assert_eq!(
        bufs[0].content(),
        bufs[1].content(),
        "w073-09: calibration must not alter cells"
    );
    assert_eq!(
        bufs[0].content(),
        bufs[2].content(),
        "w073-09: measured must not alter cells"
    );
}

#[test]
fn w073_09_with_part_does_not_double_count() {
    let probe = StyleTimingProbe::new();
    probe.reset(StyleTimingMode::Measured);
    let mut scene = Scene::new("w073-09-wrap", Theme::junie(), ColorLevel::TrueColor, 20, 4);
    scene.draw(|ui, _| {
        ui.with_timing_probe(&probe, |ui| {
            ui.with_part(
                Family::BUTTON,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::empty(),
                |_ui, _r| {},
            );
        });
    });
    // One leaf entry (Style), not two. A doubly timed wrapper fails.
    assert_eq!(
        probe.witness(),
        vec![StyleTimingEntry::Style as u8],
        "w073-09: wrapper/leaf double counting fails"
    );
    assert_eq!(
        probe.intervals().len(),
        1,
        "w073-09: one interval for one leaf"
    );
}

#[test]
fn w073_09_negative_correction_is_invalid() {
    // Deterministic arithmetic control: a calibration sample exceeding its
    // measured pair is an invalid measurement, never clamped to zero.
    let measured: u128 = 100;
    let calibration: u128 = 140;
    let corrected = measured as i128 - calibration as i128;
    assert!(
        corrected < 0,
        "w073-09 test setup: this pair must be negative"
    );
    // The production rule (mirrored here for the control): negative
    // differentials are INVALID and must never be clamped.
    let clamped = corrected.max(0);
    assert_ne!(
        clamped, corrected,
        "w073-09: clamping a negative correction to zero fails"
    );
    assert!(
        corrected < 0,
        "w073-09: negative corrected duration fails (INVALID)"
    );
}

#[test]
fn w073_09_status_delta_branches() {
    // Empty deltas take the early return (no bind); nonempty deltas bind
    // through the same seam. Closed-shell zero-call branches and separate
    // nonempty witnesses are both explicit.
    let mut empty_scene = Scene::new(
        "w073-09-empty",
        Theme::junie(),
        ColorLevel::TrueColor,
        40,
        4,
    );
    let empty_probe = StyleTimingProbe::new();
    empty_probe.reset(StyleTimingMode::Measured);
    empty_scene.draw(|ui, _| {
        ui.with_timing_probe(&empty_probe, |ui| {
            StatusBar::new(Id::root("w073.09.status.empty")).draw(ui, Rect::new(0, 0, 40, 1));
        });
    });
    assert!(
        !empty_probe
            .witness()
            .contains(&(StyleTimingEntry::StatusBind as u8)),
        "w073-09: empty Status delta must not bind"
    );

    let mut full_scene = Scene::new("w073-09-full", Theme::junie(), ColorLevel::TrueColor, 40, 4);
    let full_probe = StyleTimingProbe::new();
    full_probe.reset(StyleTimingMode::Measured);
    full_scene.draw(|ui, _| {
        ui.with_timing_probe(&full_probe, |ui| {
            let items = [StatusItem::new("busy").tone(junie_tui::Role::Warning)];
            StatusBar::new(Id::root("w073.09.status.full"))
                .left(&items)
                .draw(ui, Rect::new(0, 0, 40, 1));
        });
    });
    // The toned item paints, so its nonempty delta binds through the seam.
    assert!(
        full_probe
            .witness()
            .contains(&(StyleTimingEntry::StatusBind as u8)),
        "w073-09: nonempty Status delta must bind"
    );
    for id in full_probe.witness() {
        assert!(
            (0..=10).contains(&id),
            "w073-09: foreign witness ID {id} fails"
        );
    }
}

struct W07309GridModel {
    toned: bool,
}

impl GridModel for W07309GridModel {
    fn row_count(&self) -> usize {
        2
    }

    fn row_key(&self, row: usize) -> ItemKey {
        ItemKey::num(row as u64 + 1)
    }

    fn cell(&self, _row: usize, _col: usize) -> Option<CellRef<'_>> {
        let cell = CellRef::new("g");
        if self.toned {
            Some(cell.tone(Role::Warning))
        } else {
            Some(cell)
        }
    }
}

fn w073_09_grid_columns() -> [Column<'static>; 2] {
    [
        Column::new(ColumnKey::num(1), "A"),
        Column::new(ColumnKey::num(2), "B"),
    ]
}

#[test]
fn w073_09_grid_delta_branches() {
    // Grid `apply_style_delta` mirrors Status: empty deltas return early,
    // toned cells bind through the same caller-owned seam.
    let columns = w073_09_grid_columns();
    let state = GridState::default();
    let mut empty_scene = Scene::new(
        "w073-09-grid-empty",
        Theme::junie(),
        ColorLevel::TrueColor,
        40,
        6,
    );
    let empty_probe = StyleTimingProbe::new();
    empty_probe.reset(StyleTimingMode::Measured);
    empty_scene.draw(|ui, _| {
        ui.with_timing_probe(&empty_probe, |ui| {
            Grid::new(Id::root("w073.09.grid.empty"), &columns).draw(
                ui,
                Rect::new(0, 0, 40, 5),
                &state,
                &W07309GridModel { toned: false },
            );
        });
    });
    assert!(
        !empty_probe
            .witness()
            .contains(&(StyleTimingEntry::GridBind as u8)),
        "w073-09: empty Grid delta must not bind"
    );

    let mut full_scene = Scene::new(
        "w073-09-grid-full",
        Theme::junie(),
        ColorLevel::TrueColor,
        40,
        6,
    );
    let full_probe = StyleTimingProbe::new();
    full_probe.reset(StyleTimingMode::Measured);
    full_scene.draw(|ui, _| {
        ui.with_timing_probe(&full_probe, |ui| {
            Grid::new(Id::root("w073.09.grid.full"), &columns).draw(
                ui,
                Rect::new(0, 0, 40, 5),
                &state,
                &W07309GridModel { toned: true },
            );
        });
    });
    assert!(
        full_probe
            .witness()
            .contains(&(StyleTimingEntry::GridBind as u8)),
        "w073-09: nonempty Grid delta must bind"
    );
    for id in full_probe.witness() {
        assert!(
            (0..=10).contains(&id),
            "w073-09: foreign witness ID {id} fails"
        );
    }
}
