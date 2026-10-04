//! Comprehensive Verification Probes for Termrock P5 TASK-013:
//! Implement progress, status, hints, and help chrome.
//!
//! Validates:
//! - AC-001: Chrome components render semantic states, animation phases, and metadata.
//! - AC-002: Determinate/indeterminate progress, spinner frame progression modulo 10, meter thresholds and tones, status bar responsive priority drops, hint bar overflow truncation with ellipsis, key hint pills, too-small centered display.
//! - AC-003: Frozen references, application invariants, zero clippy warnings.
//! - AC-004: All test gates pass.

use std::collections::HashMap;

use ratatui::buffer::Buffer;

use junie_tui::core::event::{Input, Mouse, MouseKind};
use junie_tui::termrock::hint_bar::{HintBar, HintItem};
use junie_tui::termrock::key_hint::KeyHint;
use junie_tui::termrock::{
    AnimationSample, ColorLevel, Constraints, Cx, Id, Invalidate, ItemKey, LayerStack, MeasureCx,
    Meter, MeterLevel, MeterTone, MeterVisual, Moment, ProgressBar, ProgressStatus, Rect, Size,
    Spinner, SpinnerStatus, StatusBar, StatusBarAction, StatusBarState, StatusItem, Theme, Tone,
    TooSmall, Ui, UpdateCause,
};

fn make_cx<'a>(
    cause: &'a UpdateCause,
    id: Id,
    layers: &'a mut LayerStack,
    geom: &'a HashMap<Id, Rect>,
) -> Cx<'a> {
    Cx {
        cause,
        moment: Moment::from_millis(100),
        intended_owner: Some(id.clone()),
        focus: Some(id),
        pointer_capture: None,
        invalidate: Invalidate::None,
        layer_stack: layers,
        cursor_request: None,
        new_focus: None,
        new_capture: None,
        feedback_requests: Vec::new(),
        published_geometry: Some(geom),
    }
}

fn make_ui<'a>(
    theme: &'a Theme,
    viewport: Rect,
    layers: &'a mut LayerStack,
    buf: &'a mut Buffer,
) -> Ui<'a> {
    Ui::new(theme, viewport, layers).with_buffer(buf)
}

fn buffer_row(buf: &Buffer, y: u16, width: u16) -> String {
    (0..width)
        .map(|x| buf[(x, y)].symbol().to_owned())
        .collect()
}

// =========================================================================
// 1. ProgressBar Verification
// =========================================================================

#[test]
fn test_progress_bar_determinate_fraction_clamping_and_nan() {
    let id = Id::new("pb.test");

    // Finite values clamped
    let pb_zero = ProgressBar::new(id.clone(), -0.5);
    assert_eq!(pb_zero.fraction, 0.0);

    let pb_full = ProgressBar::new(id.clone(), 1.5);
    assert_eq!(pb_full.fraction, 1.0);

    // Non-finite values clamped to 0.0
    let pb_nan = ProgressBar::new(id.clone(), f32::NAN);
    assert_eq!(pb_nan.fraction, 0.0);

    let pb_inf = ProgressBar::new(id, f32::INFINITY);
    assert_eq!(pb_inf.fraction, 0.0);
}

#[test]
fn test_progress_bar_measure_and_label_threshold() {
    let id = Id::new("pb.measure");
    let pb_no_label = ProgressBar::new(id.clone(), 0.5);
    let theme = Theme::termrock();
    let cx = MeasureCx::new(Constraints::unbounded(), &theme, ColorLevel::TrueColor);

    // min width without label: 6 (track) + 5 (pct) + 2 (suffix) = 13
    let size = pb_no_label.measure(&cx, Constraints::unbounded());
    assert_eq!(size.width, 13);
    assert_eq!(size.height, 1);

    // with label "Build" (5 chars): 5 + 2 + 6 + 5 + 2 = 20
    let pb_label = ProgressBar::new(id, 0.5).label("Build");
    let size_label = pb_label.measure(&cx, Constraints::unbounded());
    assert_eq!(size_label.width, 20);
    assert_eq!(size_label.height, 1);
}

#[test]
fn test_progress_bar_draw_determinate_status_suffixes() {
    let id = Id::new("pb.suffix");
    let theme = Theme::termrock();

    for (status, expected_suffix) in [
        (ProgressStatus::Active, "  "),
        (ProgressStatus::Done, " ✓"),
        (ProgressStatus::Error, " !"),
        (ProgressStatus::Paused, " ‖"),
    ] {
        let pb = ProgressBar::new(id.clone(), 0.5).status(status);
        let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 30, 1));
        let mut layers_buf = LayerStack::new();
        let mut ui = make_ui(&theme, Rect::new(0, 0, 100, 30), &mut layers_buf, &mut buf);
        pb.draw(&mut ui, Rect::new(0, 0, 30, 1));

        let row = buffer_row(ui.buffer.as_ref().unwrap(), 0, 30);
        assert!(row.contains("50%"), "Row should contain percentage: {row}");
        assert!(
            row.contains(expected_suffix),
            "Row should contain suffix {expected_suffix:?}: {row}"
        );
    }
}

#[test]
fn test_progress_bar_label_omitted_when_narrow() {
    let id = Id::new("pb.narrow");
    let theme = Theme::termrock();
    // Label "Download" is 8 cells. Label + 8 = 16.
    let pb = ProgressBar::new(id, 0.75).label("Download");

    // Width 16 is <= 16: label omitted
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 16, 1));
    let mut layers_buf = LayerStack::new();
    let mut ui = make_ui(&theme, Rect::new(0, 0, 100, 30), &mut layers_buf, &mut buf);
    pb.draw(&mut ui, Rect::new(0, 0, 16, 1));
    let row = buffer_row(ui.buffer.as_ref().unwrap(), 0, 16);
    assert!(
        !row.contains("Download"),
        "Label should be omitted when area width <= label_w + 8: {row}"
    );

    // Width 20 is > 16: label included
    let mut buf2 = Buffer::empty(ratatui::layout::Rect::new(0, 0, 20, 1));
    let mut layers_buf2 = LayerStack::new();
    let mut ui2 = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf2,
        &mut buf2,
    );
    pb.draw(&mut ui2, Rect::new(0, 0, 20, 1));
    let row2 = buffer_row(ui2.buffer.as_ref().unwrap(), 0, 20);
    assert!(
        row2.contains("Download"),
        "Label should be drawn when area width > label_w + 8: {row2}"
    );
}

#[test]
fn test_progress_bar_too_narrow_renders_percentage_only() {
    let id = Id::new("pb.pct_only");
    let theme = Theme::termrock();
    // Width 9 with percentage 5 and suffix 2 leaves track < 6: percentage only
    let pb = ProgressBar::new(id, 0.42);
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 9, 1));
    let mut layers_buf = LayerStack::new();
    let mut ui = make_ui(&theme, Rect::new(0, 0, 100, 30), &mut layers_buf, &mut buf);
    pb.draw(&mut ui, Rect::new(0, 0, 9, 1));

    let row = buffer_row(ui.buffer.as_ref().unwrap(), 0, 9);
    assert!(row.contains("42%"));
    assert!(!row.contains('━'), "No track drawn when track_w < 6: {row}");
}

#[test]
fn test_progress_bar_indeterminate_renders_moving_segment() {
    let id = Id::new("pb.indet");
    let theme = Theme::termrock();

    let pb_t10 = ProgressBar::new(id.clone(), 0.0)
        .indeterminate(true)
        .animation(AnimationSample::phase(10));
    let mut buf10 = Buffer::empty(ratatui::layout::Rect::new(0, 0, 30, 1));
    let mut layers_buf10 = LayerStack::new();
    let mut ui10 = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf10,
        &mut buf10,
    );
    pb_t10.draw(&mut ui10, Rect::new(0, 0, 30, 1));
    let row10 = buffer_row(ui10.buffer.as_ref().unwrap(), 0, 30);
    assert!(
        row10.contains('━'),
        "Row at phase 10 must contain filled segment: {row10}"
    );

    let pb_t20 = ProgressBar::new(id, 0.0)
        .indeterminate(true)
        .animation(AnimationSample::phase(20));
    let mut buf20 = Buffer::empty(ratatui::layout::Rect::new(0, 0, 30, 1));
    let mut layers_buf20 = LayerStack::new();
    let mut ui20 = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf20,
        &mut buf20,
    );
    pb_t20.draw(&mut ui20, Rect::new(0, 0, 30, 1));
    let row20 = buffer_row(ui20.buffer.as_ref().unwrap(), 0, 30);
    assert!(
        row20.contains('━'),
        "Row at phase 20 must contain filled segment: {row20}"
    );

    // Segment should move between phases
    assert_ne!(row10, row20);
}

// =========================================================================
// 2. Spinner Verification
// =========================================================================

#[test]
fn test_spinner_frames_modulo_ten_and_wraparound() {
    let id = Id::new("sp.frames");
    let expected_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let theme = Theme::termrock();

    for (tick, &expected) in expected_frames.iter().enumerate() {
        let sp = Spinner::new(id.clone(), tick as u64);
        let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 5, 1));
        let mut layers_buf = LayerStack::new();
        let mut ui = make_ui(&theme, Rect::new(0, 0, 100, 30), &mut layers_buf, &mut buf);
        sp.draw(&mut ui, Rect::new(0, 0, 5, 1));
        let row = buffer_row(ui.buffer.as_ref().unwrap(), 0, 5);
        assert!(
            row.starts_with(expected),
            "Tick {tick} expected {expected}, got {row}"
        );
    }

    // Wraparound at tick 10 -> same as tick 0
    let sp_wrap = Spinner::new(id, 10);
    let mut buf_wrap = Buffer::empty(ratatui::layout::Rect::new(0, 0, 5, 1));
    let mut layers_buf_wrap = LayerStack::new();
    let mut ui_wrap = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_wrap,
        &mut buf_wrap,
    );
    sp_wrap.draw(&mut ui_wrap, Rect::new(0, 0, 5, 1));
    let row_wrap = buffer_row(ui_wrap.buffer.as_ref().unwrap(), 0, 5);
    assert!(row_wrap.starts_with("⠋"));
}

#[test]
fn test_spinner_stopped_and_paused_status() {
    let id = Id::new("sp.status");
    let theme = Theme::termrock();

    // Stopped renders blank space
    let sp_stopped = Spinner::new(id.clone(), 3).status(SpinnerStatus::Stopped);
    let mut buf_s = Buffer::empty(ratatui::layout::Rect::new(0, 0, 5, 1));
    let mut layers_buf_s = LayerStack::new();
    let mut ui_s = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_s,
        &mut buf_s,
    );
    sp_stopped.draw(&mut ui_s, Rect::new(0, 0, 5, 1));
    let row_s = buffer_row(ui_s.buffer.as_ref().unwrap(), 0, 5);
    assert!(row_s.starts_with(' '));

    // Paused stays on frame 0 regardless of tick
    let sp_paused = Spinner::new(id, 7).status(SpinnerStatus::Paused);
    let mut buf_p = Buffer::empty(ratatui::layout::Rect::new(0, 0, 5, 1));
    let mut layers_buf_p = LayerStack::new();
    let mut ui_p = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_p,
        &mut buf_p,
    );
    sp_paused.draw(&mut ui_p, Rect::new(0, 0, 5, 1));
    let row_p = buffer_row(ui_p.buffer.as_ref().unwrap(), 0, 5);
    assert!(row_p.starts_with("⠋"));
}

#[test]
fn test_spinner_label_and_measurement() {
    let id = Id::new("sp.label");
    let theme = Theme::termrock();
    let cx = MeasureCx::new(Constraints::unbounded(), &theme, ColorLevel::TrueColor);

    let sp_no_label = Spinner::new(id.clone(), 0);
    assert_eq!(sp_no_label.measure(&cx, Constraints::unbounded()).width, 1);

    let sp_with_label = Spinner::new(id, 0).label("Syncing");
    assert_eq!(
        sp_with_label.measure(&cx, Constraints::unbounded()).width,
        9
    );

    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 15, 1));
    let mut layers_buf = LayerStack::new();
    let mut ui = make_ui(&theme, Rect::new(0, 0, 100, 30), &mut layers_buf, &mut buf);
    sp_with_label.draw(&mut ui, Rect::new(0, 0, 15, 1));
    let row = buffer_row(ui.buffer.as_ref().unwrap(), 0, 15);
    assert!(row.contains("Syncing"));
}

// =========================================================================
// 3. Meter Verification
// =========================================================================

#[test]
fn test_meter_threshold_levels() {
    assert_eq!(MeterLevel::of(0), MeterLevel::Low);
    assert_eq!(MeterLevel::of(59), MeterLevel::Low);
    assert_eq!(MeterLevel::of(60), MeterLevel::Medium);
    assert_eq!(MeterLevel::of(84), MeterLevel::Medium);
    assert_eq!(MeterLevel::of(85), MeterLevel::High);
    assert_eq!(MeterLevel::of(100), MeterLevel::High);
}

#[test]
fn test_meter_line_and_block_visual_modes() {
    let id = Id::new("meter.visual");
    let theme = Theme::termrock();

    // Line mode: renders track run with ━ and ─
    let meter_line = Meter::new(id.clone(), 38.0, 0.0, 100.0).visual(MeterVisual::Line);
    let mut buf_l = Buffer::empty(ratatui::layout::Rect::new(0, 0, 30, 1));
    let mut layers_buf_l = LayerStack::new();
    let mut ui_l = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_l,
        &mut buf_l,
    );
    meter_line.draw(&mut ui_l, Rect::new(0, 0, 30, 1));
    let row_l = buffer_row(ui_l.buffer.as_ref().unwrap(), 0, 30);
    assert!(row_l.contains('━'));
    assert!(row_l.contains('─'));
    assert!(row_l.contains("38%"));

    // Block mode: embeds percentage inside block
    let meter_block = Meter::new(id, 38.0, 0.0, 100.0).visual(MeterVisual::Block);
    let mut buf_b = Buffer::empty(ratatui::layout::Rect::new(0, 0, 30, 1));
    let mut layers_buf_b = LayerStack::new();
    let mut ui_b = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_b,
        &mut buf_b,
    );
    meter_block.draw(&mut ui_b, Rect::new(0, 0, 30, 1));
    let row_b = buffer_row(ui_b.buffer.as_ref().unwrap(), 0, 30);
    assert!(row_b.contains("38%"));
}

#[test]
fn test_meter_semantic_tones_and_refreshing() {
    let id = Id::new("meter.tones");
    let theme = Theme::termrock();

    // Warning tone carries ▲ marker
    let meter_warn = Meter::new(id.clone(), 70.0, 0.0, 100.0).tone(MeterTone::Warning);
    let mut buf_w = Buffer::empty(ratatui::layout::Rect::new(0, 0, 30, 1));
    let mut layers_buf_w = LayerStack::new();
    let mut ui_w = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_w,
        &mut buf_w,
    );
    meter_warn.draw(&mut ui_w, Rect::new(0, 0, 30, 1));
    let row_w = buffer_row(ui_w.buffer.as_ref().unwrap(), 0, 30);
    assert!(row_w.contains('▲'));

    // Exhausted tone carries ! marker
    let meter_exh = Meter::new(id.clone(), 95.0, 0.0, 100.0).tone(MeterTone::Exhausted);
    let mut buf_e = Buffer::empty(ratatui::layout::Rect::new(0, 0, 30, 1));
    let mut layers_buf_e = LayerStack::new();
    let mut ui_e = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_e,
        &mut buf_e,
    );
    meter_exh.draw(&mut ui_e, Rect::new(0, 0, 30, 1));
    let row_e = buffer_row(ui_e.buffer.as_ref().unwrap(), 0, 30);
    assert!(row_e.contains('!'));

    // Unknown has no track and renders em-dash —
    let meter_unk = Meter::new(id.clone(), 0.0, 0.0, 100.0).tone(MeterTone::Unknown);
    let mut buf_u = Buffer::empty(ratatui::layout::Rect::new(0, 0, 30, 1));
    let mut layers_buf_u = LayerStack::new();
    let mut ui_u = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_u,
        &mut buf_u,
    );
    meter_unk.draw(&mut ui_u, Rect::new(0, 0, 30, 1));
    let row_u = buffer_row(ui_u.buffer.as_ref().unwrap(), 0, 30);
    assert!(row_u.contains('—'));
    assert!(!row_u.contains('━'), "Unknown tone must not draw track");

    // Refreshing renders spinner + "refreshing"
    let meter_ref = Meter::new(id, 50.0, 0.0, 100.0)
        .tone(MeterTone::Refreshing)
        .animation(AnimationSample::phase(2));
    let mut buf_r = Buffer::empty(ratatui::layout::Rect::new(0, 0, 30, 1));
    let mut layers_buf_r = LayerStack::new();
    let mut ui_r = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_r,
        &mut buf_r,
    );
    meter_ref.draw(&mut ui_r, Rect::new(0, 0, 30, 1));
    let row_r = buffer_row(ui_r.buffer.as_ref().unwrap(), 0, 30);
    assert!(row_r.contains("refreshing"));
    assert!(row_r.contains('⠹'));
}

// =========================================================================
// 4. StatusBar Verification
// =========================================================================

#[test]
fn test_status_bar_layout_and_priority_degradation() {
    let id = Id::new("sb.layout");
    let left = [
        StatusItem::new(1u64, "SurfaceA").priority(9),
        StatusItem::new(2u64, "ModeB").priority(4),
    ];
    let center = [StatusItem::new(3u64, "ActivityC").priority(2)];
    let right = [
        StatusItem::new(4u64, "FactD").priority(6),
        StatusItem::new(5u64, "FactE").priority(3),
    ];

    let sb = StatusBar::new(id).left(&left).center(&center).right(&right);

    // Wide area: all 5 items fit
    let placed_wide = sb.layout(Rect::new(0, 0, 80, 1));
    assert_eq!(placed_wide.len(), 5);

    // Constrained width 40: lowest priority item (ActivityC: priority 2) dropped first
    let placed_mid = sb.layout(Rect::new(0, 0, 40, 1));
    let keys_mid: Vec<ItemKey> = placed_mid.iter().map(|p| p.item.key).collect();
    assert!(
        !keys_mid.contains(&ItemKey::new(3)),
        "Lowest priority ActivityC should be dropped first"
    );

    // Very narrow width 10: strongest left item (SurfaceA) survives truncated
    let placed_tiny = sb.layout(Rect::new(0, 0, 10, 1));
    assert_eq!(placed_tiny.len(), 1);
    assert_eq!(placed_tiny[0].item.key, ItemKey::new(1));
    assert!(placed_tiny[0].truncated_text.len() <= 10);
}

#[test]
fn test_status_bar_interactive_click_action() {
    let id = Id::new("sb.click");
    let item_key = ItemKey::new(42);
    let left = [StatusItem::new(item_key, "ClickMe").interactive(Id::new("click_target"))];
    let sb = StatusBar::new(id.clone()).left(&left);

    let mut state = StatusBarState::new();
    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(sb.row_id(item_key), Rect::new(1, 0, 10, 1));

    // Click inside item triggers action
    let cause_click = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: ratatui::layout::Position { x: 3, y: 0 },
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&cause_click, id.clone(), &mut layers, &geom);
    let resp = sb.update(&mut cx, &mut state);
    assert_eq!(resp.action, Some(StatusBarAction::ItemClicked(item_key)));
    assert_eq!(resp.invalidate, Invalidate::Paint);

    // Click outside item emits no action
    let cause_outside = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: ratatui::layout::Position { x: 25, y: 0 },
        }),
        Moment::from_millis(100),
    );
    let mut cx_out = make_cx(&cause_outside, id, &mut layers, &geom);
    let resp_out = sb.update(&mut cx_out, &mut state);
    assert_eq!(resp_out.action, None);
}

#[test]
fn test_status_bar_draw_chips_and_busy_spinner() {
    let id = Id::new("sb.draw");
    let theme = Theme::termrock();
    let left = [
        StatusItem::new(1u64, "PROJECT").strong(),
        StatusItem::new(2u64, "BRANCH").chip(),
        StatusItem::new(3u64, "Building").busy(),
    ];
    let sb = StatusBar::new(id)
        .left(&left)
        .animation(AnimationSample::phase(0));

    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 60, 1));
    let mut layers_buf = LayerStack::new();
    let mut ui = make_ui(&theme, Rect::new(0, 0, 100, 30), &mut layers_buf, &mut buf);
    sb.draw(&mut ui, Rect::new(0, 0, 60, 1));

    let row = buffer_row(ui.buffer.as_ref().unwrap(), 0, 60);
    assert!(row.contains("PROJECT"));
    assert!(row.contains("BRANCH"));
    assert!(row.contains("Building"));
    assert!(
        row.contains('⠋'),
        "Busy status item should render spinner frame"
    );
}

// =========================================================================
// 5. HintBar Verification
// =========================================================================

#[test]
fn test_hint_bar_draw_and_ellipsis_truncation() {
    let id = Id::new("hb.test");
    let theme = Theme::termrock();
    let hints = [
        HintItem::new("Enter", "Open"),
        HintItem::new("Space", "Toggle"),
        HintItem::new("Tab", "Next"),
        HintItem::new("Esc", "Close"),
    ];

    let hb = HintBar::new(id, &hints);

    // Wide area: all hints fit
    let mut buf_wide = Buffer::empty(ratatui::layout::Rect::new(0, 0, 60, 1));
    let mut layers_buf_wide = LayerStack::new();
    let mut ui_wide = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_wide,
        &mut buf_wide,
    );
    hb.draw(&mut ui_wide, Rect::new(0, 0, 60, 1));
    let row_wide = buffer_row(ui_wide.buffer.as_ref().unwrap(), 0, 60);
    assert!(row_wide.contains("Enter Open"));
    assert!(row_wide.contains("Space Toggle"));
    assert!(row_wide.contains("Tab Next"));
    assert!(row_wide.contains("Esc Close"));
    assert!(!row_wide.contains('…'));

    // Narrow area 20: hints that do not fit are dropped from right, leaving ellipsis …
    let mut buf_narrow = Buffer::empty(ratatui::layout::Rect::new(0, 0, 20, 1));
    let mut layers_buf_narrow = LayerStack::new();
    let mut ui_narrow = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_narrow,
        &mut buf_narrow,
    );
    hb.draw(&mut ui_narrow, Rect::new(0, 0, 20, 1));
    let row_narrow = buffer_row(ui_narrow.buffer.as_ref().unwrap(), 0, 20);
    assert!(row_narrow.contains("Enter Open"));
    assert!(
        row_narrow.contains('…'),
        "Narrow hint bar must mark dropped hints with ellipsis: {row_narrow}"
    );
}

#[test]
fn test_hint_bar_badge_and_status() {
    let id = Id::new("hb.badge");
    let theme = Theme::termrock();
    let hints = [HintItem::new("q", "Quit")];

    let hb = HintBar::new(id, &hints)
        .badge("NORMAL")
        .status("All systems nominal", Tone::Success);

    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 60, 1));
    let mut layers_buf = LayerStack::new();
    let mut ui = make_ui(&theme, Rect::new(0, 0, 100, 30), &mut layers_buf, &mut buf);
    hb.draw(&mut ui, Rect::new(0, 0, 60, 1));

    let row = buffer_row(ui.buffer.as_ref().unwrap(), 0, 60);
    assert!(row.contains("NORMAL"));
    assert!(row.contains("q Quit"));
    assert!(row.contains("All systems nominal"));
}

// =========================================================================
// 6. KeyHint Verification
// =========================================================================

#[test]
fn test_key_hint_measure_and_draw() {
    let kh = KeyHint::new("Ctrl+C", "Quit");
    let theme = Theme::termrock();
    let cx = MeasureCx::new(Constraints::unbounded(), &theme, ColorLevel::TrueColor);

    // width: "Ctrl+C" (6) + space (1) + "Quit" (4) = 11
    let size = kh.measure(&cx, Constraints::unbounded());
    assert_eq!(size.width, 11);
    assert_eq!(size.height, 1);

    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 15, 1));
    let mut layers_buf = LayerStack::new();
    let mut ui = make_ui(&theme, Rect::new(0, 0, 100, 30), &mut layers_buf, &mut buf);
    kh.draw(&mut ui, Rect::new(0, 0, 15, 1));

    let row = buffer_row(ui.buffer.as_ref().unwrap(), 0, 15);
    assert!(row.contains("Ctrl+C Quit"));
}

// =========================================================================
// 7. TooSmall Verification
// =========================================================================

#[test]
fn test_too_small_safe_bounded_and_centered() {
    let id = Id::new("too_small.test");
    let theme = Theme::termrock();
    let min_size = Size::new(80, 24);

    let ts = TooSmall::new(id.clone(), min_size);

    // Zero area must not panic or underflow
    let mut buf_zero = Buffer::empty(ratatui::layout::Rect::new(0, 0, 0, 0));
    let mut layers_buf_zero = LayerStack::new();
    let mut ui_zero = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_zero,
        &mut buf_zero,
    );
    ts.draw(&mut ui_zero, Rect::new(0, 0, 0, 0));

    // 1x1 area must not panic
    let mut buf_one = Buffer::empty(ratatui::layout::Rect::new(0, 0, 1, 1));
    let mut layers_buf_one = LayerStack::new();
    let mut ui_one = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_one,
        &mut buf_one,
    );
    ts.draw(&mut ui_one, Rect::new(0, 0, 1, 1));

    // Normal area: renders centered notice with dimensions
    let mut buf_normal = Buffer::empty(ratatui::layout::Rect::new(0, 0, 50, 10));
    let mut layers_buf_normal = LayerStack::new();
    let mut ui_normal = make_ui(
        &theme,
        Rect::new(0, 0, 100, 30),
        &mut layers_buf_normal,
        &mut buf_normal,
    );
    ts.draw(&mut ui_normal, Rect::new(0, 0, 50, 10));

    let mut found_title = false;
    let mut found_dimensions = false;
    let mut found_quit = false;

    for y in 0..10 {
        let row = buffer_row(ui_normal.buffer.as_ref().unwrap(), y, 50);
        if row.contains("Terminal too small") {
            found_title = true;
        }
        if row.contains("Need 80×24, have 50×10") {
            found_dimensions = true;
        }
        if row.contains("q Quit") {
            found_quit = true;
        }
    }

    assert!(found_title, "Notice must include title");
    assert!(
        found_dimensions,
        "Notice must include exact need/have dimensions"
    );
    assert!(found_quit, "Notice must include quit hint");
}
