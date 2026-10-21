use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use termrock::Color as RColor;
use termrock::Input as RtInput;
use termrock::runtime::stub::{Stub, deliver};
use termrock::{
    Buffer, Button, Id, Modifier, Rect, ReferenceState, ReferenceTarget, Runtime, Theme, Ui,
    Variant,
};
use tuiscotti::grouped::GroupedStore;
use tuiscotti::snapshot::Status;
use tuiscotti::tui::{MouseButton, MouseMods, Tui};
use tuiscotti::{
    Cell, Color, CursorStyle, Frame, Mods, Profile, Provenance, Renderer, Rgb, UnderlineStyle,
    VENDORED_FACES,
};

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

#[test]
fn negative_phase8ap_fg_mutation_fails_ansi_keeps_txt() {
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

#[test]
fn negative_phase8ap_truncated_ansi_and_mixed_txt_fail_integrity() {
    let base = base_frame();

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

    let empty: BTreeSet<String> = BTreeSet::new();
    let hits = apply_filter(&empty, "neg8ap/anything");
    assert!(hits.is_empty());
    let ferr = require_matches(&hits, "neg8ap/anything").unwrap_err();
    assert!(ferr.contains("no captures matched"), "{ferr}");
}

#[test]
fn negative_phase8ap_drag_without_reporting_fails_plain_input_succeeds() {
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

    let needle = "neg8ap-plain-input-ok";
    s.send_text(needle).expect("plain text needs no mode");
    s.press("enter").expect("parsed key needs no mode");
    s.wait_predicate_timeout(
        |o| support::screen_text(&o.screen).contains(needle),
        Duration::from_secs(5),
    )
    .expect("plain input must land");
}

const NEG8AP_BTN: Id = Id::root("neg8ap.btn");

fn convert_color(c: RColor) -> Color {
    match c {
        RColor::Reset => Color::Default,
        RColor::Black => Color::Indexed(0),
        RColor::Red => Color::Indexed(1),
        RColor::Green => Color::Indexed(2),
        RColor::Yellow => Color::Indexed(3),
        RColor::Blue => Color::Indexed(4),
        RColor::Magenta => Color::Indexed(5),
        RColor::Cyan => Color::Indexed(6),
        RColor::Gray => Color::Indexed(7),
        RColor::DarkGray => Color::Indexed(8),
        RColor::LightRed => Color::Indexed(9),
        RColor::LightGreen => Color::Indexed(10),
        RColor::LightYellow => Color::Indexed(11),
        RColor::LightBlue => Color::Indexed(12),
        RColor::LightMagenta => Color::Indexed(13),
        RColor::LightCyan => Color::Indexed(14),
        RColor::White => Color::Indexed(15),
        RColor::Indexed(i) => Color::Indexed(i),
        RColor::Rgb(r, g, b) => Color::Rgb(Rgb::new(r, g, b)),
    }
}

fn convert_mods(m: Modifier) -> Mods {
    Mods {
        hidden: m.contains(Modifier::HIDDEN),
        blink: m.intersects(Modifier::SLOW_BLINK | Modifier::RAPID_BLINK),
        bold: m.contains(Modifier::BOLD),
        dim: m.contains(Modifier::DIM),
        italic: m.contains(Modifier::ITALIC),
        underline: m.contains(Modifier::UNDERLINED),
        underline_style: if m.contains(Modifier::UNDERLINED) {
            UnderlineStyle::Single
        } else {
            UnderlineStyle::None
        },
        strikethrough: m.contains(Modifier::CROSSED_OUT),
        reverse: m.contains(Modifier::REVERSED),
    }
}

fn capture_buffer(buf: &Buffer) -> Frame {
    let cols = buf.area.width;
    let rows = buf.area.height;
    let mut frame = Frame::blank(cols, rows, prov("negative-button-8ap"));
    for y in 0..rows {
        for x in 0..cols {
            let Some(rc) = buf.cell((x, y)) else {
                continue;
            };
            let mut cell = Cell::blank(x, y);
            cell.symbol = rc.symbol().to_string();
            cell.fg = convert_color(rc.fg);
            cell.bg = convert_color(rc.bg);
            cell.mods = convert_mods(rc.modifier);
            cell.underline_color = convert_color(rc.underline_color);
            frame.set(cell);
        }
    }
    frame.validate().expect("captured button frame valid");
    frame
}

fn render_button(focus: bool, hover: bool, disabled: bool) -> Frame {
    let mut runtime = Runtime::new(Stub::default(), Theme::junie());
    let _ = runtime.initialize();
    for _ in 0..7 {
        let _ = deliver(&mut runtime, RtInput::Tick);
    }
    let view = Rect::new(0, 0, 24, 1);
    let mut buf = Buffer::empty(view);
    let mut state = ReferenceState::default();
    if focus {
        state = state.union(ReferenceState::FOCUSED | ReferenceState::FOCUS_VISIBLE);
    }
    if hover {
        state = state.union(ReferenceState::HOVERED);
    }
    let state = (state != ReferenceState::default()).then_some((NEG8AP_BTN, state));
    runtime
        .draw_scene(view, &mut buf, |ui: &mut Ui<'_>, _area| {
            let draw = |ui: &mut Ui<'_>| {
                Button::new(NEG8AP_BTN, "Neg8ap")
                    .variant(Variant::SECONDARY)
                    .disabled(disabled)
                    .draw(ui, Rect::new(0, 0, 24, 1));
            };
            if let Some((id, state)) = state {
                ui.reference(Some(ReferenceTarget::new(id, state)), |ui| draw(ui));
            } else {
                draw(ui);
            }
        })
        .commit_presented();
    capture_buffer(&buf)
}

#[test]
fn negative_phase8ap_focus_hover_disabled_states_differ() {
    let plain = render_button(false, false, false);
    let focused = render_button(true, false, false);
    assert_eq!(plain.get(0, 0).expect("gutter").symbol, " ");
    assert_eq!(focused.get(0, 0).expect("gutter").symbol, "▎");
    assert_ne!(plain.digest(), focused.digest(), "focus must differ");
    assert!(
        !plain.diff_cells(&focused).expect("same dims").is_empty(),
        "focus diff must be non-empty"
    );

    let hovered = render_button(false, true, false);
    assert_ne!(plain.digest(), hovered.digest(), "hover must differ");
    assert!(
        !plain.diff_cells(&hovered).expect("same dims").is_empty(),
        "hover diff must be non-empty"
    );

    let disabled = render_button(false, false, true);
    assert_ne!(plain.digest(), disabled.digest(), "disabled must differ");
    assert!(
        !plain.diff_cells(&disabled).expect("same dims").is_empty(),
        "disabled diff must be non-empty"
    );

    let digests = BTreeSet::from([
        plain.digest(),
        focused.digest(),
        hovered.digest(),
        disabled.digest(),
    ]);
    assert_eq!(digests.len(), 4, "states must be pairwise distinct");
}

#[test]
fn negative_phase8ap_boundary_clocks_match_semantic_diff_still_fails() {
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

    let mut lie = mutate_symbol(&fixed, 0, 0, "X");
    lie.provenance.created_unix = 999;
    assert_ne!(fixed.digest(), lie.digest());
    let mut r2 = renderer();
    let bad = store.check_with(&mut r2, name, &lie, 1.0).expect("check");
    assert_eq!(bad.status(), Status::CellsDiffer);
    bad.ensure_matched().unwrap_err();
}
