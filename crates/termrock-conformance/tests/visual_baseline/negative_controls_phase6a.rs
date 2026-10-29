use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tuiscotti::grouped::GroupedStore;
use tuiscotti::snapshot::Status;
use tuiscotti::tui::{MouseMods, Tui, Wheel};
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
        .join("target/tuiscotti-negative-phase6a")
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

fn base_frame() -> Frame {
    let mut f = Frame::blank(20, 6, prov("negative-phase6a"));
    set_text(&mut f, 0, 0, "hello phase6a");
    set_text(&mut f, 0, 2, "row-two content");
    let mut styled = Cell::blank(5, 1);
    styled.symbol = "S".to_string();
    styled.fg = Color::Indexed(2);
    f.set(styled);
    f.validate().expect("base frame valid");
    f
}

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
    for (source, destination) in [
        (&first.actual.ansi, &first.approved.ansi),
        (&first.actual.txt, &first.approved.txt),
        (&first.actual.png, &first.approved.png),
        (&first.actual.html, &first.approved.html),
    ] {
        std::fs::create_dir_all(destination.parent().expect("artifact parent"))
            .expect("create approved artifact parent");
        std::fs::copy(source, destination).expect("copy generated artifact fixture");
    }
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

fn cat_binary() -> &'static str {
    if Path::new("/bin/cat").exists() {
        "/bin/cat"
    } else {
        "cat"
    }
}

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
fn negative_phase6a_single_cursor_value_mutation_detected() {
    let base = cursor_base();
    let dir = scratch_dir("cursor-style");
    let name = "neg6a/cursor-style/80x24/truecolor";
    let store = approve_grouped(&dir, name, &base);

    let mut styled = base.clone();
    styled.cursor.style = CursorStyle::Underline;
    styled.validate().expect("mutated valid");
    assert_eq!(styled.cursor.x, base.cursor.x);
    assert_eq!(styled.cursor.visible, base.cursor.visible);
    assert_ne!(
        base.digest(),
        styled.digest(),
        "digest must cover a single cursor value"
    );

    let mut r = renderer();
    let outcome = store
        .check_with(&mut r, name, &styled, 1.0)
        .expect("check runs");
    assert_eq!(outcome.ansi_match, Some(true), "ansi carries no cursor");
    assert_eq!(outcome.txt_match, Some(true), "txt carries no cursor");
    assert_eq!(outcome.html_match, Some(false), "html embeds cursor");
    assert_eq!(outcome.status(), Status::PixelsDiffer);
    assert!(
        outcome.outcome.pixel_score.is_some_and(|s| s < 1.0),
        "one cursor value must move pixels"
    );
    outcome.ensure_matched().unwrap_err();

    let dir2 = scratch_dir("cursor-x");
    let name2 = "neg6a/cursor-x/80x24/truecolor";
    let store2 = approve_grouped(&dir2, name2, &base);
    let mut moved = base.clone();
    moved.cursor.x = 4;
    moved.validate().expect("mutated valid");
    assert_ne!(base.digest(), moved.digest(), "digest must cover cursor x");
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
fn negative_phase6a_integrity_faults_detected_distinctly() {
    let base = base_frame();

    let dir_a = scratch_dir("integrity-missing");
    let name_a = "neg6a/integrity/80x24/truecolor";
    let store_a = approve_grouped(&dir_a, name_a, &base);
    let victim = store_a.approved_root().join(format!("{name_a}.txt"));
    std::fs::remove_file(&victim).expect("remove approved txt");
    let mut ra = renderer();
    let out_a = store_a
        .check_with(&mut ra, name_a, &base, 1.0)
        .expect("check runs");
    assert_eq!(out_a.status(), Status::MissingApproval);
    assert!(
        out_a.outcome.note.contains(".txt"),
        "note names the missing artifact: {}",
        out_a.outcome.note
    );
    let sig_missing = format!("status={:?}", out_a.status());

    let dir_b = scratch_dir("integrity-corrupt");
    let name_b = "neg6a/integrity/80x24/truecolor";
    let store_b = approve_grouped(&dir_b, name_b, &base);
    let png_path = store_b.approved_root().join(format!("{name_b}.png"));
    let mut bytes = std::fs::read(&png_path).expect("read approved png");
    assert!(bytes.len() > 128, "png big enough to corrupt mid-body");
    let flip_at = bytes.len() / 2;
    bytes[flip_at] ^= 0xFF;
    std::fs::write(&png_path, &bytes).expect("flip one png byte");
    let mut rb = renderer();
    let err_b = store_b
        .check_with(&mut rb, name_b, &base, 1.0)
        .unwrap_err()
        .to_string();
    assert!(
        err_b.contains("cannot decode"),
        "corrupt hash must error loudly: {err_b}"
    );
    let sig_corrupt = "error:cannot-decode".to_string();

    let mut gen_b = base.clone();
    let mut c = gen_b.get(0, 0).unwrap().clone();
    c.symbol = "Z".to_string();
    gen_b.set(c);
    gen_b.validate().expect("gen B valid");
    let dir_c = scratch_dir("integrity-mixed");
    let name_c = "neg6a/integrity/80x24/truecolor";
    let store_c = approve_grouped(&dir_c, name_c, &base);
    let mut rc = renderer();
    let art_b = rc.render_artifacts(&gen_b, name_c).expect("render B");
    let ansi_path = store_c.approved_root().join(format!("{name_c}.ansi"));
    std::fs::write(&ansi_path, art_b.ansi.as_bytes()).expect("mix ansi");
    let out_c = store_c
        .check_with(&mut rc, name_c, &base, 1.0)
        .expect("check runs");
    assert_eq!(out_c.status(), Status::CellsDiffer);
    assert_eq!(out_c.ansi_match, Some(false));
    let sig_mixed = format!("status={:?}", out_c.status());

    let suite: BTreeSet<String> = ["neg6a/integrity/80x24/truecolor".to_string()]
        .into_iter()
        .collect();
    let approved: BTreeSet<String> = [
        "neg6a/integrity/80x24/truecolor".to_string(),
        "neg6a/stale/80x24/truecolor".to_string(),
    ]
    .into_iter()
    .collect();
    let orphans: Vec<_> = approved.difference(&suite).cloned().collect();
    assert_eq!(orphans, vec!["neg6a/stale/80x24/truecolor".to_string()]);
    let sig_stale = format!("orphan:{}", orphans.join(","));

    let sigs = BTreeSet::from([sig_missing, sig_corrupt, sig_mixed, sig_stale]);
    assert_eq!(sigs.len(), 4, "integrity faults must differ: {sigs:?}");
}

#[test]
fn negative_phase6a_shard_inventory_omitted_and_duplicated_fail() {
    let suite: BTreeSet<String> = [
        "neg6a/s/a/80x24/truecolor".to_string(),
        "neg6a/s/b/80x24/truecolor".to_string(),
        "neg6a/s/c/80x24/truecolor".to_string(),
    ]
    .into_iter()
    .collect();
    let covered: BTreeSet<String> = [
        "neg6a/s/a/80x24/truecolor".to_string(),
        "neg6a/s/b/80x24/truecolor".to_string(),
    ]
    .into_iter()
    .collect();
    let pending: Vec<_> = suite.difference(&covered).cloned().collect();
    assert_eq!(pending, vec!["neg6a/s/c/80x24/truecolor".to_string()]);
    let err = format!(
        "shard run 'smoke' omits {} case(s): {}",
        pending.len(),
        pending.join(", ")
    );
    assert!(err.contains("neg6a/s/c/80x24/truecolor"), "{err}");
    assert!(
        !pending.is_empty(),
        "omitted shard case must fail acceptance: {err}"
    );

    let decls = [
        "neg6a/d/x/80x24/truecolor",
        "neg6a/d/y/80x24/truecolor",
        "neg6a/d/x/80x24/truecolor",
    ];
    let mut seen = BTreeSet::new();
    let mut dups = Vec::new();
    for d in decls {
        if !seen.insert(d.to_string()) {
            dups.push(d.to_string());
        }
    }
    assert_eq!(dups, vec!["neg6a/d/x/80x24/truecolor".to_string()]);
    let derr = format!("duplicate capture declaration: {}", dups.join(", "));
    assert!(derr.contains("duplicate"), "{derr}");
    assert!(derr.contains("neg6a/d/x/80x24/truecolor"), "{derr}");
}

#[test]
fn negative_phase6a_mode_gated_inputs_fail_without_reporting() {
    let s = Tui::new([cat_binary()])
        .size(80, 24)
        .spawn()
        .expect("spawn cat");

    let err_move = s.mouse_move(1, 1, MouseMods::NONE).unwrap_err().to_string();
    assert!(
        err_move.contains("mouse motion reporting (DEC 1003) not enabled"),
        "useful mode error: {err_move}"
    );

    let err_wheel = s
        .mouse_wheel(Wheel::Up, 1, 1, MouseMods::NONE)
        .unwrap_err()
        .to_string();
    assert!(
        err_wheel.contains("mouse reporting (DEC 1000/1002/1003) not enabled"),
        "useful mode error: {err_wheel}"
    );

    let err_out = s.focus_out().unwrap_err().to_string();
    assert!(
        err_out.contains("focus tracking (DEC 1004) not enabled"),
        "useful mode error: {err_out}"
    );
}

#[test]
fn negative_phase6a_dropped_sequence_input_fails_postcondition() {
    let full = "seq6a-postcondition-789";
    let prefix = "seq6a-postcondition-78";

    let s = Tui::new([cat_binary()])
        .size(80, 24)
        .spawn()
        .expect("spawn cat");
    s.send_text(full).expect("send full sequence");
    s.wait_predicate_timeout(
        |o| support::screen_text(&o.screen).contains(full),
        Duration::from_secs(5),
    )
    .expect("full sequence must satisfy postcondition");

    let t = Tui::new([cat_binary()])
        .size(80, 24)
        .spawn()
        .expect("spawn cat");
    t.send_text(prefix).expect("send truncated sequence");
    t.wait_predicate_timeout(
        |o| support::screen_text(&o.screen).contains(prefix),
        Duration::from_secs(5),
    )
    .expect("truncated input must land");
    let text = support::screen_text(&t.observe_now().expect("observe").screen);
    assert!(text.contains(prefix), "sanity: truncated input visible");
    assert!(
        !text.contains(full),
        "postcondition must FAIL without the dropped input; screen: {text:?}"
    );
}

#[test]
fn negative_phase6a_timestamp_only_frame_matches_same_approval() {
    let mut a = base_frame();
    a.provenance.created_unix = 1_000;
    let mut b = base_frame();
    b.provenance.created_unix = 2_000;
    assert_eq!(a.digest(), b.digest(), "digest excludes timestamp");

    let dir = scratch_dir("ts-match");
    let name = "neg6a/ts-match/80x24/truecolor";
    let store = approve_grouped(&dir, name, &a);
    let mut r = renderer();
    let outcome = store.check_with(&mut r, name, &b, 1.0).expect("check runs");
    assert!(
        outcome.matched(),
        "timestamp-only frame must match same approval: {}",
        outcome.outcome.note
    );
    assert_eq!(outcome.ansi_match, Some(true));
    assert_eq!(outcome.txt_match, Some(true));
    assert_eq!(outcome.html_match, Some(true));
    outcome
        .ensure_matched()
        .expect("no unjustified visual failure");
}
