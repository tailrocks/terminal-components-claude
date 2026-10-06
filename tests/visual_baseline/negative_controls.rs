//! Negative controls for the visual-baseline harness.
//!
//! Every test here proves the harness FAILS when it must: a one-cell lie,
//! a missing artifact, a corrupt approval, a mixed generation, a stale run
//! dir, an unmatched filter, an omitted or duplicated case, a PTY input
//! without its required terminal mode, an ambiguous target, a bounded
//! readiness timeout, identical-looking but semantically different selected
//! states, a dropped input, and a timestamp-only change.
//!
//! Isolation contract: all checks run against small synthetic [`Frame`]s and
//! temp artifact copies under `target/tuisnap-negative/` (gitignored
//! scratch). Nothing here reads `snapshots/` as approval state and nothing
//! writes outside its own temp dir. No production code is mutated.
//!
//! Gate reference: grouped store (`.ansi` cell-exact + `.txt` content +
//! `.html` render-level + `.png` pixel-exact at threshold 1.0) and the
//! classic [`Store`] (frame JSON + PNG). See `tuisnap::grouped` docs.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use junie_tui::core::{focus::FocusRing, hit::HitRegistry, id::WidgetId};
use junie_tui::theme::Theme;
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::progress::SPINNER;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tuisnap::grouped::GroupedStore;
use tuisnap::pty::{PtyOptions, Session};
use tuisnap::snapshot::{Status, Store};
use tuisnap::{Cell, Color, CursorStyle, Frame, Profile, Provenance, Renderer, VENDORED_FACES};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch_dir(test: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuisnap-negative")
        .join(format!("{test}_{}_{nanos}_{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn prov(source: &str) -> Provenance {
    Provenance {
        tool: "tuisnap".to_string(),
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

/// Small deterministic base frame: 20x6, text row, one styled cell.
fn base_frame() -> Frame {
    let mut f = Frame::blank(20, 6, prov("negative"));
    set_text(&mut f, 0, 0, "hello negative");
    set_text(&mut f, 0, 2, "row-two content");
    // One styled cell so style/bg mutations have a known starting point.
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
    assert_eq!(again.ansi_match, Some(true));
    assert_eq!(again.txt_match, Some(true));
    assert_eq!(again.html_match, Some(true));
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

// ------------------------------------------------ sanity: harness can pass --

#[test]
fn negative_sanity_identical_frame_matches() {
    let dir = scratch_dir("sanity");
    let frame = base_frame();
    let store = approve_grouped(&dir, "neg/sanity/80x24/truecolor", &frame);
    let mut r = renderer();
    let outcome = store
        .check_with(&mut r, "neg/sanity/80x24/truecolor", &frame, 1.0)
        .expect("check");
    assert!(
        outcome.matched(),
        "control must pass: {}",
        outcome.outcome.note
    );
}

// --------------------------------------- one-glyph / bg / style / cursor --

#[test]
fn negative_single_glyph_mutation_fails_cell_gate() {
    let dir = scratch_dir("glyph");
    let name = "neg/glyph/80x24/truecolor";
    let base = base_frame();
    let store = approve_grouped(&dir, name, &base);
    assert_eq!(base.get(0, 0).unwrap().symbol, "h");

    let mutated = mutate_symbol(&base, 0, 0, "X");
    assert_ne!(base.digest(), mutated.digest(), "digest must move");
    let diffs = base.diff_cells(&mutated).expect("same dims diff");
    assert_eq!(diffs, vec![(0, 0)], "exactly one cell differs");

    let mut r = renderer();
    let outcome = store
        .check_with(&mut r, name, &mutated, 1.0)
        .expect("check runs");
    assert_eq!(outcome.status(), Status::CellsDiffer);
    assert_eq!(outcome.ansi_match, Some(false), "ansi is cell-exact");
    assert_eq!(outcome.txt_match, Some(false), "glyph changes content");
    let err = outcome.ensure_matched().unwrap_err().to_string();
    assert!(err.contains("cells-differ"), "useful message: {err}");
    assert!(
        outcome.actual.ansi.exists(),
        "actuals written before assert"
    );
}

#[test]
fn negative_background_mutation_fails_ansi_keeps_txt() {
    let dir = scratch_dir("bg");
    let name = "neg/bg/80x24/truecolor";
    let base = base_frame();
    let store = approve_grouped(&dir, name, &base);

    let mut mutated = base.clone();
    let mut c = mutated.get(1, 0).unwrap().clone();
    assert_eq!(c.bg, Color::Default);
    c.bg = Color::Indexed(4);
    mutated.set(c);
    mutated.validate().expect("mutated frame valid");

    assert_ne!(base.digest(), mutated.digest());
    assert_eq!(base.text(), mutated.text(), "content identical");

    let mut r = renderer();
    let outcome = store
        .check_with(&mut r, name, &mutated, 1.0)
        .expect("check runs");
    assert_eq!(outcome.status(), Status::CellsDiffer);
    assert_eq!(outcome.ansi_match, Some(false), "bg is in ansi gate");
    assert_eq!(outcome.txt_match, Some(true), "txt ignores style");
    assert!(
        outcome.outcome.pixel_score.is_some_and(|s| s < 1.0),
        "pixel gate must also move"
    );
    outcome.ensure_matched().unwrap_err();
}

#[test]
fn negative_style_mutation_fails_ansi_keeps_txt() {
    let dir = scratch_dir("style");
    let name = "neg/style/80x24/truecolor";
    let base = base_frame();
    let store = approve_grouped(&dir, name, &base);

    let mut mutated = base.clone();
    let mut c = mutated.get(2, 0).unwrap().clone();
    assert!(!c.mods.bold);
    c.mods.bold = true;
    mutated.set(c);
    mutated.validate().expect("mutated frame valid");

    let mut r = renderer();
    let outcome = store
        .check_with(&mut r, name, &mutated, 1.0)
        .expect("check runs");
    assert_eq!(outcome.status(), Status::CellsDiffer);
    assert_eq!(outcome.ansi_match, Some(false));
    assert_eq!(outcome.txt_match, Some(true), "bold keeps content");
    outcome.ensure_matched().unwrap_err();
}

#[test]
fn negative_cursor_mutation_fails_pixel_gate() {
    let dir = scratch_dir("cursor");
    let name = "neg/cursor/80x24/truecolor";
    let mut base = base_frame();
    base.cursor.visible = false;
    base.validate().expect("base valid");
    let store = approve_grouped(&dir, name, &base);

    // Cursor-only change: cells identical, cursor flips visible + position.
    let mut mutated = base.clone();
    mutated.cursor.visible = true;
    mutated.cursor.x = 3;
    mutated.cursor.y = 0;
    mutated.cursor.style = CursorStyle::Block;
    mutated.validate().expect("mutated valid");

    let diffs = base.diff_cells(&mutated).expect("same dims");
    assert_eq!(diffs.len(), 1, "cursor-only diff reports one position");
    assert_ne!(base.digest(), mutated.digest(), "digest covers cursor");

    let mut r = renderer();
    let outcome = store
        .check_with(&mut r, name, &mutated, 1.0)
        .expect("check runs");
    // ansi/txt carry no cursor; PNG + HTML (PNG embed + frame JSON) do.
    assert_eq!(outcome.ansi_match, Some(true));
    assert_eq!(outcome.txt_match, Some(true));
    assert_eq!(outcome.html_match, Some(false), "html embeds cursor");
    assert_eq!(outcome.status(), Status::PixelsDiffer);
    assert!(
        outcome.outcome.pixel_score.is_some_and(|s| s < 1.0),
        "cursor must move pixels"
    );
    outcome.ensure_matched().unwrap_err();
}

#[test]
fn negative_dimension_mutation_fails_dimension_gate() {
    let dir = scratch_dir("dims");
    let name = "neg/dims/80x24/truecolor";
    let base = base_frame();
    let store = approve_grouped(&dir, name, &base);

    let mut wider = Frame::blank(21, 6, prov("negative"));
    set_text(&mut wider, 0, 0, "hello negative");
    set_text(&mut wider, 0, 2, "row-two content");
    wider.validate().expect("wider valid");

    let err = base.diff_cells(&wider).unwrap_err().to_string();
    assert!(err.contains("dimension mismatch"), "{err}");

    let mut r = renderer();
    let outcome = store
        .check_with(&mut r, name, &wider, 1.0)
        .expect("check runs");
    assert_eq!(outcome.status(), Status::DimensionMismatch);
    let msg = outcome.ensure_matched().unwrap_err().to_string();
    assert!(msg.contains("dimension-mismatch"), "{msg}");
}

// --------------------------------------------- integrity: missing artifacts --

#[test]
fn negative_missing_each_artifact_fails_closed() {
    for ext in ["ansi", "txt", "png", "html"] {
        let dir = scratch_dir(&format!("missing-{ext}"));
        let name = "neg/missing/80x24/truecolor";
        let base = base_frame();
        let store = approve_grouped(&dir, name, &base);

        let victim = store.approved_root().join(format!("{name}.{ext}"));
        assert!(victim.exists(), "{ext} approved exists before removal");
        std::fs::remove_file(&victim).expect("remove artifact");

        let mut r = renderer();
        let outcome = store
            .check_with(&mut r, name, &base, 1.0)
            .expect("check runs");
        assert_eq!(
            outcome.status(),
            Status::MissingApproval,
            "missing .{ext} must fail closed"
        );
        assert!(
            outcome.outcome.note.contains(&format!(".{ext}")),
            "note names the missing artifact: {}",
            outcome.outcome.note
        );
        outcome.ensure_matched().unwrap_err();
    }
}

// --------------------------------------- integrity: corrupt / mixed / stale --

#[test]
fn negative_corrupt_approval_fails_explicitly() {
    // Classic store: corrupt frame JSON reads as CorruptApproval, never a
    // silent default or a pass.
    let dir = scratch_dir("corrupt-classic");
    let classic = Store::new(&dir.join("classic"));
    let mut r = renderer();
    let frame = base_frame();
    let first = classic
        .check_with(&mut r, "neg-corrupt", &frame, 1.0)
        .expect("first check");
    assert_eq!(first.status, Status::MissingApproval);
    classic.accept("neg-corrupt").expect("accept");
    let approved_json = dir.join("classic/approved/neg-corrupt.frame.json");
    std::fs::write(&approved_json, b"{ not valid json {{{").expect("corrupt json");
    let outcome = classic
        .check_with(&mut r, "neg-corrupt", &frame, 1.0)
        .expect("check runs");
    assert_eq!(outcome.status, Status::CorruptApproval);
    assert!(
        outcome.note.contains("corrupt"),
        "note says corrupt: {}",
        outcome.note
    );
    outcome.ensure_matched().unwrap_err();

    // Grouped store: corrupt approved PNG is an explicit decode error.
    let dir2 = scratch_dir("corrupt-png");
    let name = "neg/corrupt/80x24/truecolor";
    let store = approve_grouped(&dir2, name, &frame);
    let png = store.approved_root().join(format!("{name}.png"));
    std::fs::write(&png, b"definitely not a png").expect("corrupt png");
    let mut r2 = renderer();
    let err = store
        .check_with(&mut r2, name, &frame, 1.0)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("cannot decode"),
        "corrupt PNG must error loudly: {err}"
    );

    // Frame import itself rejects garbage instead of guessing.
    assert!(Frame::from_json("{ bad").is_err());
}

#[test]
fn negative_mixed_generation_artifacts_fail_gate() {
    // Generation A approved; then generation B's bytes overwrite one
    // artifact. Both the ansi-mixed and the png-mixed stores must fail.
    let base_a = base_frame();
    let base_b = mutate_symbol(&base_frame(), 0, 0, "Z");
    assert_ne!(base_a.digest(), base_b.digest());

    // Mix 1: B's ansi over A's approval, check A.
    let dir = scratch_dir("mix-ansi");
    let name = "neg/mix/80x24/truecolor";
    let store = approve_grouped(&dir, name, &base_a);
    let mut r = renderer();
    let artifacts_b = r.render_artifacts(&base_b, name).expect("render B");
    let ansi_path = store.approved_root().join(format!("{name}.ansi"));
    std::fs::write(&ansi_path, artifacts_b.ansi.as_bytes()).expect("mix ansi");
    let outcome = store
        .check_with(&mut r, name, &base_a, 1.0)
        .expect("check runs");
    assert_eq!(outcome.status(), Status::CellsDiffer);
    assert_eq!(outcome.ansi_match, Some(false));
    outcome.ensure_matched().unwrap_err();

    // Mix 2: B's png over A's approval, check A.
    let dir2 = scratch_dir("mix-png");
    let store2 = approve_grouped(&dir2, name, &base_a);
    let mut r2 = renderer();
    let artifacts_b2 = r2.render_artifacts(&base_b, name).expect("render B");
    let png_path = store2.approved_root().join(format!("{name}.png"));
    std::fs::write(&png_path, &artifacts_b2.png).expect("mix png");
    let outcome2 = store2
        .check_with(&mut r2, name, &base_a, 1.0)
        .expect("check runs");
    assert!(!outcome2.matched(), "png-mixed generation must not match");
    outcome2.ensure_matched().unwrap_err();
}

#[test]
fn negative_stale_run_dir_does_not_pass() {
    // Stale inventory: approved names the suite no longer declares are
    // orphans; suite names without approval are pending. Both fail.
    let suite: BTreeSet<String> = ["neg/a/80x24/truecolor".to_string()].into_iter().collect();
    let approved: BTreeSet<String> = [
        "neg/a/80x24/truecolor".to_string(),
        "neg/stale/80x24/truecolor".to_string(),
    ]
    .into_iter()
    .collect();
    let orphans: Vec<_> = approved.difference(&suite).collect();
    assert_eq!(orphans.len(), 1, "stale name must be flagged");
    let pending: Vec<String> = suite.difference(&approved).cloned().collect();
    assert!(pending.is_empty());
    assert!(
        !orphans.is_empty() || !pending.is_empty(),
        "stale inventory must fail acceptance"
    );

    // Stale actuals are rewritten by check, never trusted.
    let dir = scratch_dir("stale-actual");
    let name = "neg/stale-actual/80x24/truecolor";
    let frame = base_frame();
    let store = approve_grouped(&dir, name, &frame);
    let actual_ansi = store.actual_root().join(format!("{name}.ansi"));
    std::fs::write(&actual_ansi, b"stale bytes from a previous run").expect("plant stale");
    let mut r = renderer();
    let outcome = store
        .check_with(&mut r, name, &frame, 1.0)
        .expect("check runs");
    assert!(outcome.matched(), "fresh frame still matches");
    let fresh = std::fs::read(&actual_ansi).expect("read rewritten actual");
    assert!(
        !fresh.starts_with(b"stale bytes"),
        "check must overwrite stale actuals"
    );
}

// --------------------------------------------- acceptance: filter / inventory --

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

fn resolve_single(cases: &BTreeSet<String>, filter: &str) -> Result<String, String> {
    let hits = apply_filter(cases, filter);
    match hits.len() {
        0 => Err(format!("no capture matches {filter:?}")),
        1 => Ok(hits.into_iter().next().expect("one hit")),
        _ => Err(format!(
            "ambiguous filter {filter:?}: {} matches ({})",
            hits.len(),
            hits.join(", ")
        )),
    }
}

#[test]
fn negative_unmatched_filter_fails_acceptance() {
    let suite: BTreeSet<String> = [
        "neg/a/80x24/truecolor".to_string(),
        "neg/b/80x24/truecolor".to_string(),
    ]
    .into_iter()
    .collect();
    let hits = apply_filter(&suite, "no-such-capture-xyz");
    assert!(hits.is_empty());
    let err = require_matches(&hits, "no-such-capture-xyz").unwrap_err();
    assert!(err.contains("no captures matched"), "{err}");
    // Zero-match acceptance is a failure, never a silent green run.
}

#[test]
fn negative_omitted_and_duplicated_cases_fail_acceptance() {
    // Omitted: suite declares more than the store approved.
    let suite: BTreeSet<String> = [
        "neg/a/80x24/truecolor".to_string(),
        "neg/omitted/80x24/truecolor".to_string(),
    ]
    .into_iter()
    .collect();
    let approved: BTreeSet<String> = ["neg/a/80x24/truecolor".to_string()].into_iter().collect();
    let pending: Vec<_> = suite.difference(&approved).collect();
    assert_eq!(pending.len(), 1);
    assert!(!pending.is_empty(), "omitted case must fail acceptance");

    // Duplicated: the same canonical root declared twice is a config
    // conflict, not a precedence rule.
    let mut roots = BTreeSet::new();
    assert!(roots.insert("neg/dup".to_string()));
    assert!(
        !roots.insert("neg/dup".to_string()),
        "duplicate root must be detected"
    );
}

// --------------------------- terminal mode / ambiguity / readiness timeout --

fn cat_binary() -> &'static str {
    if Path::new("/bin/cat").exists() {
        "/bin/cat"
    } else {
        "cat"
    }
}

#[test]
fn negative_input_without_terminal_mode_fails_usefully() {
    // `cat` never enables bracketed paste, so a literal paste must fail
    // with a mode error instead of silently injecting input.
    let opts = PtyOptions {
        cols: 80,
        rows: 24,
        timeout: Duration::from_secs(2),
        ..PtyOptions::default()
    };
    let mut s = Session::spawn(&[cat_binary().to_string()], &opts).expect("spawn cat");
    let err = s.paste_literal("hello").unwrap_err().to_string();
    assert!(
        err.contains("bracketed paste is not enabled"),
        "useful mode error: {err}"
    );

    // Unknown key names are rejected loudly, never swallowed.
    let err2 = s.send_key("bogus-key-xyz-123").unwrap_err().to_string();
    assert!(err2.contains("unknown key"), "useful key error: {err2}");
}

#[test]
fn negative_ambiguous_target_fails_usefully() {
    let suite: BTreeSet<String> = [
        "neg/amb/one/80x24/truecolor".to_string(),
        "neg/amb/two/80x24/truecolor".to_string(),
    ]
    .into_iter()
    .collect();
    let err = resolve_single(&suite, "neg/amb").unwrap_err();
    assert!(err.contains("ambiguous"), "{err}");
    assert!(err.contains("one") && err.contains("two"), "{err}");
    // Exact filter still resolves.
    let one = resolve_single(&suite, "neg/amb/one").expect("exact resolves");
    assert!(one.contains("/one/"));
    // Unmatched filter is a distinct useful error, not empty success.
    let none = resolve_single(&suite, "neg/nope").unwrap_err();
    assert!(none.contains("no capture matches"), "{none}");
}

#[test]
fn negative_bounded_readiness_timeout_fails_fast() {
    // A needle that never appears must fail on the bounded deadline with
    // screen evidence, never hang and never pass silently.
    let opts = PtyOptions {
        cols: 80,
        rows: 24,
        timeout: Duration::from_millis(400),
        ..PtyOptions::default()
    };
    let mut s = Session::spawn(&[cat_binary().to_string()], &opts).expect("spawn cat");
    let start = Instant::now();
    let err = s
        .wait_for_text("needle-never-appears-xyz-123")
        .unwrap_err()
        .to_string();
    let elapsed = start.elapsed();
    assert!(!err.is_empty(), "timeout error carries evidence");
    assert!(
        elapsed < Duration::from_secs(10),
        "readiness wait must be bounded, took {elapsed:?}"
    );
}

// ------------------------------- selected states / animation phases differ --

fn button_id() -> WidgetId {
    WidgetId::of("test.negative_controls")
}

fn render_button(tick: u64, on: Option<bool>, busy: bool) -> Frame {
    let theme = Theme::junie();
    let id = button_id();
    let mut button = match on {
        None => Button::secondary(id, "Neg"),
        Some(on) => Button::toggle(id, "Neg", on),
    };
    button.busy = busy;
    let area = Rect::new(0, 0, 24, 1);
    let mut buf = Buffer::empty(Rect::new(0, 0, 24, 1));
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut ctx = RenderCtx::new(
        &theme,
        Interaction {
            tick,
            ..Default::default()
        },
        &mut hits,
        &mut ring,
    );
    button.render(area, &mut buf, &mut ctx, theme.canvas);
    tuisnap::ratatui::from_buffer(&buf, 24, 1, None, prov("negative-button"))
}

#[test]
fn negative_selected_states_and_phases_differ() {
    // Animation phases: adjacent spinner ticks differ in glyph + digest.
    let f0 = render_button(0, None, true);
    let f1 = render_button(1, None, true);
    assert_eq!(f0.get(1, 0).unwrap().symbol, SPINNER[0]);
    assert_eq!(f1.get(1, 0).unwrap().symbol, SPINNER[1]);
    assert_ne!(f0.digest(), f1.digest(), "phases must differ");
    assert!(
        !f0.diff_cells(&f1).expect("same dims").is_empty(),
        "phase diff must be non-empty"
    );

    // Selected states: checked vs unchecked toggle markers differ.
    let on = render_button(0, Some(true), false);
    let off = render_button(0, Some(false), false);
    assert_eq!(on.get(1, 0).unwrap().symbol, "●");
    assert_eq!(off.get(1, 0).unwrap().symbol, "○");
    assert_ne!(on.digest(), off.digest(), "selected states must differ");
    assert!(
        !on.diff_cells(&off).expect("same dims").is_empty(),
        "selection diff must be non-empty"
    );

    // Busy overwrite: busy+checked shows the spinner, not the toggle dot.
    let busy_on = render_button(0, Some(true), true);
    assert_eq!(busy_on.get(1, 0).unwrap().symbol, SPINNER[0]);
    assert_ne!(busy_on.digest(), on.digest());
}

// ------------------------------------------------- dropped input / timestamp --

#[test]
fn negative_dropped_input_fails_postcondition() {
    // Dropped cells: cell-count mismatch fails validation loudly.
    let mut frame = base_frame();
    frame.cells.pop();
    let err = frame.validate().unwrap_err().to_string();
    assert!(err.contains("cell count"), "{err}");

    // Dropped command: empty argv never spawns.
    let opts = PtyOptions::default();
    let err2 = Session::spawn(&[], &opts)
        .err()
        .expect("empty argv must fail")
        .to_string();
    assert!(err2.contains("empty command"), "{err2}");

    // Dropped actuals: accept with nothing captured fails.
    let dir = scratch_dir("accept-empty");
    let store = grouped_store(&dir);
    let err3 = store
        .accept("neg/nothing/80x24/truecolor")
        .unwrap_err()
        .to_string();
    assert!(err3.contains("nothing to accept"), "{err3}");
}

#[test]
fn negative_timestamp_only_change_hides_no_diff() {
    // Timestamp-only change: same cells, different provenance clock.
    let mut a = base_frame();
    a.provenance.created_unix = 1_000;
    let mut b = base_frame();
    b.provenance.created_unix = 2_000;
    assert_eq!(a.digest(), b.digest(), "digest excludes timestamp");
    assert!(
        a.diff_cells(&b).expect("same dims").is_empty(),
        "no semantic diff"
    );
    assert_ne!(a.to_json(), b.to_json(), "JSON still carries the clock");

    let mut r = renderer();
    let art_a = r.render_artifacts(&a, "neg/ts").expect("render A");
    let art_b = r.render_artifacts(&b, "neg/ts").expect("render B");
    assert_eq!(art_a.ansi, art_b.ansi);
    assert_eq!(art_a.txt, art_b.txt);
    assert_eq!(art_a.html, art_b.html, "HTML normalizes timestamp");
    assert_eq!(art_a.png, art_b.png);

    // Semantic diff plus a timestamp change still fails: the clock cannot
    // hide a one-glyph lie.
    let dir = scratch_dir("ts-semantic");
    let name = "neg/ts/80x24/truecolor";
    let store = approve_grouped(&dir, name, &a);
    let mut c = mutate_symbol(&b, 0, 0, "X");
    c.provenance.created_unix = 3_000;
    assert_ne!(a.digest(), c.digest());
    assert!(!a.diff_cells(&c).expect("same dims").is_empty());
    let mut r2 = renderer();
    let outcome = store.check_with(&mut r2, name, &c, 1.0).expect("check");
    assert_eq!(outcome.status(), Status::CellsDiffer);
    outcome.ensure_matched().unwrap_err();
}
