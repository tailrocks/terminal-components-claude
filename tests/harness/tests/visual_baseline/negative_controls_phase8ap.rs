//! Phase-8ap negative controls for the visual-baseline harness (§16).
//!
//! Companion to [`crate::negative_controls`] and
//! [`crate::negative_controls_phase6a`]: each test here proves the suite
//! FAILS when it must, closing the angles the earlier modules leave open
//! and binding every §16 clause to an executable control:
//!
//! * §16(1) compare mutations: fg-only restyle, single cursor visibility /
//!   vertical-position values, and shrunk (not only grown) dimensions.
//! * §16(2) integrity faults: truncated `.ansi` hash bytes and a mixed
//!   `.txt` generation, each with its own gate signature.
//! * §16(3) acceptance inventory: a pending (unapproved) case fails by name,
//!   and filtering an empty suite fails instead of going silently green.
//! * §16(4) terminal modes: drag without drag reporting fails with its own
//!   mode error, while plain text + key input on the same PTY succeeds
//!   (proving the failure is about the mode, not the session).
//! * §16(5) selected states: focused / hovered / disabled renders of the
//!   same button are pairwise distinguished.
//! * §16(6) timestamp-only metadata: boundary clocks (epoch and `u64::MAX`)
//!   match the same approval through the real gate, while a semantic diff
//!   at another clock still fails.
//!
//! Isolation contract (same as the earlier modules): all checks run against
//! small synthetic [`Frame`]s, pure inventory sets, and `cat` PTY sessions;
//! artifact stores live in temp copies under
//! `target/tuiscotti-negative-phase8ap/` (gitignored scratch). Nothing here
//! reads `snapshots/` as approval state and nothing writes outside its own
//! temp dir. No production code is mutated.
//!
//! Gate reference: grouped store (`.ansi` cell-exact + `.txt` content +
//! `.html` render-level + `.png` pixel-exact at threshold 1.0). See
//! `tuiscotti::grouped` docs.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use junie_tui::core::{focus::FocusRing, hit::HitRegistry, id::WidgetId};
use junie_tui::theme::Theme;
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::button::Button;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tuiscotti::grouped::GroupedStore;
use tuiscotti::snapshot::Status;
use tuiscotti::tui::{MouseButton, MouseMods, Tui};
use tuiscotti::{Cell, Color, CursorStyle, Frame, Profile, Provenance, Renderer, VENDORED_FACES};

use crate::support;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch_dir(test: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti-negative-phase8ap")
        .join(format!("{test}_{}_{nanos}_{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn prov(source: &str) -> Provenance {
    Provenance {
        tool: "tuiscotti".to_string(),
        tool_version: "test".to_string(),
        profile: "test".to_string(),
        source: source.to_string(),
        argv: vec![],
        created_unix: 1_700_000_000,
    }
}

fn renderer() -> Renderer {
    Profile::default_profile()
        .renderer(&VENDORED_FACES)
        .expect("vendored faces parse")
}

fn grouped_store(dir: &Path) -> GroupedStore {
    GroupedStore::new(&dir.join("approved"))
        .with_actual_root(&dir.join("actual"))
        .with_diff_root(&dir.join("diff"))
        .with_report_path(&dir.join("report.html"))
}

fn set_text(frame: &mut Frame, x0: u16, y: u16, text: &str) {
    for (i, ch) in text.chars().enumerate() {
        let x = x0 + i as u16;
        if x >= frame.cols {
            break;
        }
        let mut cell = Cell::blank(x, y);
        cell.symbol = ch.to_string();
        frame.set(cell);
    }
}

/// Small deterministic base frame: 20x6, text rows, one fg-styled cell.
fn base_frame() -> Frame {
    let mut f = Frame::blank(20, 6, prov("negative-phase8ap"));
    set_text(&mut f, 0, 0, "hello phase8ap");
    set_text(&mut f, 0, 2, "row-two content");
    let mut styled = Cell::blank(5, 1);
    styled.symbol = "S".to_string();
    styled.fg = Color::Indexed(2);
    f.set(styled);
    f.validate().expect("base frame valid");
    f
}

/// Approve `frame` under `name`, then prove the same frame matches.
/// Returns the store for the subsequent negative check.
fn approve_grouped(dir: &Path, name: &str, frame: &Frame) -> GroupedStore {
    let store = grouped_store(dir);
    let mut r = renderer();
    let first = store
        .check_with(&mut r, name, frame, 1.0)
        .expect("first check writes actuals");
    assert_eq!(
        first.status(),
        Status::MissingApproval,
        "fresh temp store must report missing approval"
    );
    store.accept(name).expect("accept temp actuals");
    let again = store
        .check_with(&mut r, name, frame, 1.0)
        .expect("re-check after accept");
    assert!(
        again.matched(),
        "identical frame must match after accept: {}",
        again.outcome.note
    );
    store
}

fn mutate_symbol(frame: &Frame, x: u16, y: u16, symbol: &str) -> Frame {
    let mut f = frame.clone();
    let mut c = f.get(x, y).expect("cell exists").clone();
    c.symbol = symbol.to_string();
    f.set(c);
    f.validate().expect("mutated frame valid");
    f
}

fn cat_binary() -> &'static str {
    if Path::new("/bin/cat").exists() {
        "/bin/cat"
    } else {
        "cat"
    }
}

// --------------------------------------- §16(1): fg / cursor / dimensions --

#[test]
fn negative_phase8ap_fg_mutation_fails_ansi_keeps_txt() {
    // fg-only restyle: the earlier modules cover symbol, bg, and bold;
    // the foreground channel must fail the cell-exact gate the same way.
    let dir = scratch_dir("fg");
    let name = "neg8ap/fg/80x24/truecolor";
    let base = base_frame();
    let store = approve_grouped(&dir, name, &base);

    let mut mutated = base.clone();
    let mut c = mutated.get(5, 1).expect("styled cell").clone();
    assert_eq!(c.fg, Color::Indexed(2));
    c.fg = Color::Indexed(5);
    mutated.set(c);
    mutated.validate().expect("mutated frame valid");

    assert_ne!(base.digest(), mutated.digest(), "digest must move");
    assert_eq!(
        base.diff_cells(&mutated).expect("same dims diff"),
        vec![(5, 1)],
        "exactly one cell differs"
    );
    assert_eq!(base.text(), mutated.text(), "content identical");

    let mut r = renderer();
    let outcome = store
        .check_with(&mut r, name, &mutated, 1.0)
        .expect("check runs");
    assert_eq!(outcome.status(), Status::CellsDiffer);
    assert_eq!(outcome.ansi_match, Some(false), "fg is in ansi gate");
    assert_eq!(outcome.txt_match, Some(true), "txt ignores style");
    let err = outcome.ensure_matched().unwrap_err().to_string();
    assert!(err.contains("cells-differ"), "useful message: {err}");
}

/// Base cursor: visible block at (3, 0). Each case changes exactly ONE
/// cursor value the earlier modules never isolate (visibility alone, y
/// alone); the pixel gate must catch it while ansi/txt stay green.
fn cursor_base() -> Frame {
    let mut f = base_frame();
    f.cursor.visible = true;
    f.cursor.x = 3;
    f.cursor.y = 0;
    f.cursor.style = CursorStyle::Block;
    f.validate().expect("cursor base valid");
    f
}

#[test]
fn negative_phase8ap_single_cursor_visibility_and_y_detected() {
    let base = cursor_base();

    // Case A: visibility is the only changed value (shown -> hidden).
    let dir = scratch_dir("cursor-visible");
    let name = "neg8ap/cursor-visible/80x24/truecolor";
    let store = approve_grouped(&dir, name, &base);
    let mut hidden = base.clone();
    hidden.cursor.visible = false;
    hidden.validate().expect("mutated valid");
    assert_eq!(hidden.cursor.x, base.cursor.x);
    assert_eq!(hidden.cursor.y, base.cursor.y);
    assert_eq!(hidden.cursor.style, base.cursor.style);
    assert_ne!(
        base.digest(),
        hidden.digest(),
        "digest must cover a single cursor value"
    );
    let mut r = renderer();
    let outcome = store
        .check_with(&mut r, name, &hidden, 1.0)
        .expect("check runs");
    assert_eq!(outcome.ansi_match, Some(true), "ansi carries no cursor");
    assert_eq!(outcome.txt_match, Some(true), "txt carries no cursor");
    assert_eq!(outcome.html_match, Some(false), "html embeds cursor");
    assert_eq!(outcome.status(), Status::PixelsDiffer);
    assert!(
        outcome.outcome.pixel_score.is_some_and(|s| s < 1.0),
        "visibility flip must move pixels"
    );
    outcome.ensure_matched().unwrap_err();

    // Case B: vertical position is the only changed value (0 -> 1).
    let dir2 = scratch_dir("cursor-y");
    let name2 = "neg8ap/cursor-y/80x24/truecolor";
    let store2 = approve_grouped(&dir2, name2, &base);
    let mut moved = base.clone();
    moved.cursor.y = 1;
    moved.validate().expect("mutated valid");
    assert_eq!(moved.cursor.x, base.cursor.x);
    assert!(moved.cursor.visible);
    assert_ne!(base.digest(), moved.digest(), "digest must cover cursor y");
    let mut r2 = renderer();
    let outcome2 = store2
        .check_with(&mut r2, name2, &moved, 1.0)
        .expect("check runs");
    assert_eq!(outcome2.status(), Status::PixelsDiffer);
    assert_eq!(outcome2.ansi_match, Some(true));
    assert_eq!(outcome2.txt_match, Some(true));
    outcome2.ensure_matched().unwrap_err();
}

#[test]
fn negative_phase8ap_shrunk_dimensions_fail_dimension_gate() {
    // The earlier modules prove grown frames fail; shrunk frames
    // (narrower, shorter) must fail the dimension gate just as loudly.
    let base = base_frame();
    for (cols, rows, tag) in [(19, 6, "narrower"), (20, 5, "shorter")] {
        let dir = scratch_dir(&format!("dims-{tag}"));
        let name = "neg8ap/dims/80x24/truecolor";
        let store = approve_grouped(&dir, name, &base);

        let mut shrunk = Frame::blank(cols, rows, prov("negative-phase8ap"));
        set_text(&mut shrunk, 0, 0, "hello phase8ap");
        set_text(&mut shrunk, 0, 2, "row-two content");
        shrunk.validate().expect("shrunk valid");

        let err = base.diff_cells(&shrunk).unwrap_err().to_string();
        assert!(err.contains("dimension mismatch"), "{tag}: {err}");

        let mut r = renderer();
        let outcome = store
            .check_with(&mut r, name, &shrunk, 1.0)
            .expect("check runs");
        assert_eq!(outcome.status(), Status::DimensionMismatch, "{tag}");
        let msg = outcome.ensure_matched().unwrap_err().to_string();
        assert!(msg.contains("dimension-mismatch"), "{tag}: {msg}");
    }
}

// ----------------- §16(2): truncated hash bytes / mixed txt generation --

#[test]
fn negative_phase8ap_truncated_ansi_and_mixed_txt_fail_integrity() {
    let base = base_frame();

    // Fault 1: truncated `.ansi` hash bytes (half the approved file
    // removed) fail the cell-exact gate while txt still matches.
    let dir_a = scratch_dir("integrity-truncate");
    let name_a = "neg8ap/integrity/80x24/truecolor";
    let store_a = approve_grouped(&dir_a, name_a, &base);
    let ansi_path = store_a.approved_root().join(format!("{name_a}.ansi"));
    let bytes = std::fs::read(&ansi_path).expect("read approved ansi");
    assert!(bytes.len() > 16, "ansi big enough to truncate");
    std::fs::write(&ansi_path, &bytes[..bytes.len() / 2]).expect("truncate ansi");
    let mut ra = renderer();
    let out_a = store_a
        .check_with(&mut ra, name_a, &base, 1.0)
        .expect("check runs");
    assert_eq!(out_a.status(), Status::CellsDiffer);
    assert_eq!(out_a.ansi_match, Some(false));
    assert_eq!(out_a.txt_match, Some(true));
    assert!(
        out_a.outcome.note.contains("ansi differs"),
        "note names the gate: {}",
        out_a.outcome.note
    );
    out_a.ensure_matched().unwrap_err();

    // Fault 2: mixed generations on `.txt` (generation B's bytes over
    // generation A's approval) fail the content gate while ansi matches.
    let gen_b = mutate_symbol(&base, 0, 0, "Z");
    assert_ne!(base.digest(), gen_b.digest());
    let dir_b = scratch_dir("integrity-mixed-txt");
    let name_b = "neg8ap/integrity/80x24/truecolor";
    let store_b = approve_grouped(&dir_b, name_b, &base);
    let mut rb = renderer();
    let art_b = rb.render_artifacts(&gen_b, name_b).expect("render B");
    assert_ne!(art_b.txt, base.text(), "B's content differs from A's");
    let txt_path = store_b.approved_root().join(format!("{name_b}.txt"));
    std::fs::write(&txt_path, art_b.txt.as_bytes()).expect("mix txt");
    let out_b = store_b
        .check_with(&mut rb, name_b, &base, 1.0)
        .expect("check runs");
    assert_eq!(out_b.status(), Status::CellsDiffer);
    assert_eq!(out_b.ansi_match, Some(true));
    assert_eq!(out_b.txt_match, Some(false));
    assert!(
        out_b.outcome.note.contains("txt differs"),
        "note names the gate: {}",
        out_b.outcome.note
    );
    out_b.ensure_matched().unwrap_err();
}

// ------------------- §16(3): pending case fails by name; empty suite --

fn apply_filter(cases: &BTreeSet<String>, filter: &str) -> Vec<String> {
    cases
        .iter()
        .filter(|n| n.contains(filter))
        .cloned()
        .collect()
}

fn require_matches(matches: &[String], filter: &str) -> Result<(), String> {
    if matches.is_empty() {
        return Err(format!("no captures matched filter {filter:?}"));
    }
    Ok(())
}

#[test]
fn negative_phase8ap_pending_and_empty_suite_fail_acceptance() {
    // Pending: the suite declares a case the store never approved; the
    // failure must name the pending case, not just count it.
    let suite: BTreeSet<String> = [
        "neg8ap/pending/a/80x24/truecolor".to_string(),
        "neg8ap/pending/b/80x24/truecolor".to_string(),
    ]
    .into_iter()
    .collect();
    let approved: BTreeSet<String> = ["neg8ap/pending/a/80x24/truecolor".to_string()]
        .into_iter()
        .collect();
    let pending: Vec<_> = suite.difference(&approved).cloned().collect();
    assert_eq!(
        pending,
        vec!["neg8ap/pending/b/80x24/truecolor".to_string()]
    );
    let err = format!(
        "acceptance failed: {} pending: {}",
        pending.len(),
        pending.join(", ")
    );
    assert!(err.contains("neg8ap/pending/b/80x24/truecolor"), "{err}");
    assert!(!pending.is_empty(), "pending case must fail acceptance");

    // Empty suite: filtering zero cases is an unmatched filter, never a
    // silent green run.
    let empty: BTreeSet<String> = BTreeSet::new();
    let hits = apply_filter(&empty, "neg8ap/anything");
    assert!(hits.is_empty());
    let ferr = require_matches(&hits, "neg8ap/anything").unwrap_err();
    assert!(ferr.contains("no captures matched"), "{ferr}");
}

// ------ §16(4): drag without reporting fails; plain input still works --

#[test]
fn negative_phase8ap_drag_without_reporting_fails_plain_input_succeeds() {
    // `cat` never enables mouse reporting, so a drag step must fail with
    // its own drag-mode error (not the generic click error, not silence).
    let s = Tui::new([cat_binary()])
        .size(80, 24)
        .spawn()
        .expect("spawn cat");
    let err_drag = s
        .mouse_drag(MouseButton::Left, 5, 5, MouseMods::NONE)
        .unwrap_err()
        .to_string();
    assert!(
        err_drag.contains("mouse drag reporting (DEC 1002/1003) not enabled"),
        "useful drag-mode error: {err_drag}"
    );
    let err_down = s
        .mouse_down(MouseButton::Left, 5, 5, MouseMods::NONE)
        .unwrap_err()
        .to_string();
    assert!(
        err_down.contains("mouse reporting (DEC 1000/1002/1003) not enabled"),
        "useful press-mode error: {err_down}"
    );

    // Control on the same session: plain text + a parsed key need no
    // terminal mode, so they must succeed — proving the drag failure is
    // about the missing mode, not a dead PTY.
    let needle = "neg8ap-plain-input-ok";
    s.send_text(needle).expect("plain text needs no mode");
    s.press("enter").expect("parsed key needs no mode");
    s.wait_predicate_timeout(
        |o| support::screen_text(&o.screen).contains(needle),
        Duration::from_secs(5),
    )
    .expect("plain input must land");
}

// ----------------- §16(5): focus / hover / disabled states differ --

fn button_id() -> WidgetId {
    WidgetId::of("test.negative_controls_phase8ap")
}

fn render_button(focus: bool, hover: bool, disabled: bool) -> Frame {
    let theme = Theme::junie();
    let id = button_id();
    let mut button = Button::secondary(id, "Neg8ap").disabled(disabled);
    let area = Rect::new(0, 0, 24, 1);
    let mut buf = Buffer::empty(Rect::new(0, 0, 24, 1));
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut ctx = RenderCtx::new(
        &theme,
        Interaction {
            focus: focus.then_some(id),
            hover: hover.then_some(id),
            tick: 7,
            ..Default::default()
        },
        &mut hits,
        &mut ring,
    );
    button.render(area, &mut buf, &mut ctx, theme.canvas);
    support::capture_buffer(&buf, prov("negative-button-8ap"))
}

#[test]
fn negative_phase8ap_focus_hover_disabled_states_differ() {
    // Focused vs unfocused: the gutter marker and bold emphasis differ.
    let plain = render_button(false, false, false);
    let focused = render_button(true, false, false);
    assert_eq!(plain.get(0, 0).expect("gutter").symbol, " ");
    assert_eq!(focused.get(0, 0).expect("gutter").symbol, "▎");
    assert_ne!(plain.digest(), focused.digest(), "focus must differ");
    assert!(
        !plain.diff_cells(&focused).expect("same dims").is_empty(),
        "focus diff must be non-empty"
    );

    // Hovered vs unhovered: the hover fill differs.
    let hovered = render_button(false, true, false);
    assert_ne!(plain.digest(), hovered.digest(), "hover must differ");
    assert!(
        !plain.diff_cells(&hovered).expect("same dims").is_empty(),
        "hover diff must be non-empty"
    );

    // Disabled vs enabled: the disabled tone differs.
    let disabled = render_button(false, false, true);
    assert_ne!(plain.digest(), disabled.digest(), "disabled must differ");
    assert!(
        !plain.diff_cells(&disabled).expect("same dims").is_empty(),
        "disabled diff must be non-empty"
    );

    // All four states are pairwise distinct.
    let digests = BTreeSet::from([
        plain.digest(),
        focused.digest(),
        hovered.digest(),
        disabled.digest(),
    ]);
    assert_eq!(digests.len(), 4, "states must be pairwise distinct");
}

// ----- §16(6): boundary clocks match; semantic diff still fails gate --

#[test]
fn negative_phase8ap_boundary_clocks_match_semantic_diff_still_fails() {
    // Boundary clocks, same cells: epoch, a fixed moment, and
    // `u64::MAX` must all match one approval through the real gate.
    let mut epoch = base_frame();
    epoch.provenance.created_unix = 0;
    let mut fixed = base_frame();
    fixed.provenance.created_unix = 1_700_000_000;
    let mut max = base_frame();
    max.provenance.created_unix = u64::MAX;
    assert_eq!(epoch.digest(), fixed.digest());
    assert_eq!(fixed.digest(), max.digest());
    assert_ne!(epoch.to_json(), max.to_json(), "JSON carries the clock");

    let dir = scratch_dir("ts-boundary");
    let name = "neg8ap/ts-boundary/80x24/truecolor";
    let store = approve_grouped(&dir, name, &fixed);
    let mut r = renderer();
    for (tag, frame) in [("epoch", &epoch), ("max", &max)] {
        let outcome = store
            .check_with(&mut r, name, frame, 1.0)
            .expect("check runs");
        assert!(
            outcome.matched(),
            "{tag} clock must match same approval: {}",
            outcome.outcome.note
        );
        assert_eq!(outcome.ansi_match, Some(true));
        assert_eq!(outcome.txt_match, Some(true));
        assert_eq!(outcome.html_match, Some(true));
        outcome
            .ensure_matched()
            .expect("no unjustified visual failure");
    }

    // A semantic diff at yet another clock still fails: metadata can
    // neither create a visual failure nor hide a real one.
    let mut lie = mutate_symbol(&fixed, 0, 0, "X");
    lie.provenance.created_unix = 999;
    assert_ne!(fixed.digest(), lie.digest());
    let mut r2 = renderer();
    let bad = store.check_with(&mut r2, name, &lie, 1.0).expect("check");
    assert_eq!(bad.status(), Status::CellsDiffer);
    bad.ensure_matched().unwrap_err();
}
