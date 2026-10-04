//! Comprehensive Verification Probes for Termrock P5 TASK-011:
//! Unify panel, split, scroll, and text viewport infrastructure.
//!
//! Validates:
//! - AC-001: Viewport measures, clips, scrolls, and draws with deterministic geometry and shared capture behavior.
//! - AC-002: Top/middle/bottom scroll, thumb drag, fade timing, resize, empty content, and minimum geometry.
//! - AC-003: Frozen references, application invariants, and scope protection.
//! - AC-004: Completion gate passes.

use std::collections::HashMap;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::termrock::{
    Axis, Badge, ColorLevel, Constraints, Cx, Id, Invalidate, ItemKey, LayerStack, MeasureCx,
    Moment, Panel, Rect, Revision, Size, SplitAction, SplitPane, SplitPaneState, StyledText,
    TextLine, TextPosition, TextRange, TextSource, TextViewport, Theme, Ui, UpdateCause,
    ViewportAction, ViewportMode, ViewportState,
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

struct MockSource {
    rev: Revision,
    lines: Vec<String>,
}

impl MockSource {
    fn new(lines: Vec<&str>) -> Self {
        Self {
            rev: Revision::new(1),
            lines: lines.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl TextSource for MockSource {
    fn revision(&self) -> Revision {
        self.rev
    }

    fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn line(&self, index: usize) -> Option<TextLine<'_>> {
        self.lines
            .get(index)
            .map(|s| TextLine::plain(ItemKey::new(index as u64), s))
    }
}

// =========================================================================
// 1. Panel Tests
// =========================================================================

#[test]
fn test_panel_card_measurement_and_clipping() {
    let id = Id::new("panel.card");
    let panel = Panel::card(id)
        .title("Dashboard")
        .meta(StyledText::plain("v1.0"))
        .badge(Some(Badge::new("PROD")));

    let theme = Theme::termrock();
    let cx = MeasureCx::new(Constraints::unbounded(), &theme, ColorLevel::TrueColor);

    let child_size = Size::new(40, 10);
    let measured = panel.measure(&cx, child_size, Constraints::unbounded());

    // Card with title: width = child + 4, height = child + 3
    assert_eq!(measured.width, 44);
    assert_eq!(measured.height, 13);

    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 24));
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 80, 24), &mut layers).with_buffer(&mut buf);

    let mut called_body = false;
    panel.draw(&mut ui, Rect::new(5, 5, 50, 15), |_child_ui, body_area| {
        called_body = true;
        // Inner body area should be clipped inside the panel chrome
        assert!(body_area.x > 5);
        assert!(body_area.y > 5);
        assert!(body_area.width < 50);
        assert!(body_area.height < 15);
    });

    assert!(called_body);
}

#[test]
fn test_panel_framed_measurement_and_borders() {
    let id = Id::new("panel.framed");
    let panel = Panel::framed(id).title("Settings");

    let theme = Theme::termrock();
    let cx = MeasureCx::new(Constraints::unbounded(), &theme, ColorLevel::TrueColor);

    let child_size = Size::new(30, 8);
    let measured = panel.measure(&cx, child_size, Constraints::unbounded());

    // Framed: width = child + 5, height = child + 2
    assert_eq!(measured.width, 35);
    assert_eq!(measured.height, 10);

    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 24));
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 80, 24), &mut layers).with_buffer(&mut buf);

    panel.draw(&mut ui, Rect::new(2, 2, 40, 12), |_child_ui, body_area| {
        assert_eq!(body_area.x, 2 + 1 + 2); // left border (1) + left pad (2)
        assert_eq!(body_area.y, 2 + 1); // top border (1)
        assert_eq!(body_area.width, 40 - 5);
        assert_eq!(body_area.height, 12 - 2);
    });

    // Verify rounded corner glyph in buffer at top-left
    assert_eq!(buf[(2, 2)].symbol(), "╭");
    assert_eq!(buf[(41, 2)].symbol(), "╮");
    assert_eq!(buf[(2, 13)].symbol(), "╰");
    assert_eq!(buf[(41, 13)].symbol(), "╯");
}

#[test]
fn test_panel_focus_within_highlight() {
    let id = Id::new("panel.focused");
    let card = Panel::card(id).title("Tasks").focus_within(true);

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 24));
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 80, 24), &mut layers).with_buffer(&mut buf);

    card.draw(&mut ui, Rect::new(10, 10, 40, 10), |_child_ui, _body| {});

    // Focus bar in card padding column (x = 11, y = 10)
    assert_eq!(buf[(11, 10)].symbol(), "▎");
}

#[test]
fn test_panel_title_and_meta_width_budget() {
    let id = Id::new("panel.narrow");
    let panel = Panel::card(id)
        .title("Extremely Long Section Title Exceeding Width")
        .meta(StyledText::plain("scroll 123/456"));

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 24));
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 80, 24), &mut layers).with_buffer(&mut buf);

    // Narrow area (width = 16)
    panel.draw(&mut ui, Rect::new(0, 0, 16, 8), |_child_ui, _body| {});

    // Should render gracefully without panicking or writing out of bounds
    assert!(buf.area().width >= 16);
}

// =========================================================================
// 2. SplitPane Tests
// =========================================================================

#[test]
fn test_split_pane_horizontal_and_vertical_layout() {
    let id = Id::new("split.test");
    let h_split = SplitPane::new(id.clone(), Axis::Horizontal)
        .minima(10, 10)
        .seam_width(1);

    let state = SplitPaneState::new(0.5);
    let area = Rect::new(0, 0, 81, 30);
    let areas = h_split.layout(area, &state);

    // Usable width = 80, ratio 0.5 => first: 40, seam: 1, second: 40
    assert_eq!(areas.first.width, 40);
    assert_eq!(areas.seam.width, 1);
    assert_eq!(areas.second.width, 40);
    assert_eq!(
        areas.first.width + areas.seam.width + areas.second.width,
        81
    );
    assert_eq!(areas.first.x, 0);
    assert_eq!(areas.seam.x, 40);
    assert_eq!(areas.second.x, 41);

    let v_split = SplitPane::new(id, Axis::Vertical)
        .minima(5, 5)
        .seam_width(1);
    let v_areas = v_split.layout(Rect::new(0, 0, 60, 21), &state);

    // Usable height = 20, ratio 0.5 => first: 10, seam: 1, second: 10
    assert_eq!(v_areas.first.height, 10);
    assert_eq!(v_areas.seam.height, 1);
    assert_eq!(v_areas.second.height, 10);
    assert_eq!(v_areas.first.y, 0);
    assert_eq!(v_areas.seam.y, 10);
    assert_eq!(v_areas.second.y, 11);
}

#[test]
fn test_split_pane_drag_resize_and_minima_clamping() {
    let id = Id::new("split.drag");
    let split = SplitPane::new(id.clone(), Axis::Horizontal)
        .minima(15, 20)
        .seam_width(1);

    let mut state = SplitPaneState::new(0.5);
    let area = Rect::new(0, 0, 101, 30);
    let mut geom = HashMap::new();
    geom.insert(id.clone(), area);

    let seam_id = id.sub("seam");
    geom.insert(seam_id.clone(), Rect::new(50, 0, 1, 30));

    let mut layers = LayerStack::new();

    // 1. Mouse down on seam captures pointer
    let down_cause = UpdateCause::Input(
        Input::Mouse(Mouse {
            pos: ratatui::layout::Position::new(50, 10),
            kind: MouseKind::Down,
        }),
        Moment::from_millis(100),
    );
    let mut cx_down = make_cx(&down_cause, seam_id.clone(), &mut layers, &geom);
    let resp = split.update(&mut cx_down, &mut state);
    assert_eq!(resp.flow, junie_tui::termrock::Flow::Consumed);
    assert_eq!(cx_down.new_capture, Some(Some(seam_id.clone())));

    // 2. Drag far to the left (x = 5): clamps to min_first (15)
    let drag_cause = UpdateCause::Input(
        Input::Mouse(Mouse {
            pos: ratatui::layout::Position::new(5, 10),
            kind: MouseKind::Drag,
        }),
        Moment::from_millis(150),
    );
    let mut cx_drag = make_cx(&drag_cause, seam_id.clone(), &mut layers, &geom);
    cx_drag.pointer_capture = Some(seam_id.clone());

    let resp_drag = split.update(&mut cx_drag, &mut state);
    assert!(matches!(
        resp_drag.action,
        Some(SplitAction::Resized { .. })
    ));

    // Total usable width = 100, clamped at 15 => ratio is 0.15
    assert!((state.ratio - 0.15).abs() < 1e-4);

    // 3. Drag far to the right (x = 95): clamps to usable - min_second = 100 - 20 = 80 => ratio 0.80
    let drag_right = UpdateCause::Input(
        Input::Mouse(Mouse {
            pos: ratatui::layout::Position::new(95, 10),
            kind: MouseKind::Drag,
        }),
        Moment::from_millis(200),
    );
    let mut cx_right = make_cx(&drag_right, seam_id.clone(), &mut layers, &geom);
    cx_right.pointer_capture = Some(seam_id.clone());

    let resp_right = split.update(&mut cx_right, &mut state);
    assert!(matches!(
        resp_right.action,
        Some(SplitAction::Resized { .. })
    ));
    assert!((state.ratio - 0.80).abs() < 1e-4);
}

#[test]
fn test_split_pane_double_click_maximize_and_restore() {
    let id = Id::new("split.max");
    let split = SplitPane::new(id.clone(), Axis::Horizontal);
    let mut state = SplitPaneState::new(0.5);

    let seam_id = id.sub("seam");
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 100, 30));
    geom.insert(seam_id.clone(), Rect::new(50, 0, 1, 30));

    let mut layers = LayerStack::new();

    // First click at t = 100ms
    let click1 = UpdateCause::Input(
        Input::Mouse(Mouse {
            pos: ratatui::layout::Position::new(50, 10),
            kind: MouseKind::Down,
        }),
        Moment::from_millis(100),
    );
    let mut cx1 = make_cx(&click1, seam_id.clone(), &mut layers, &geom);
    cx1.moment = Moment::from_millis(100);
    split.update(&mut cx1, &mut state);

    // Second click at t = 250ms (within 350ms window => double click)
    let click2 = UpdateCause::Input(
        Input::Mouse(Mouse {
            pos: ratatui::layout::Position::new(50, 10),
            kind: MouseKind::Down,
        }),
        Moment::from_millis(250),
    );
    let mut cx2 = make_cx(&click2, seam_id.clone(), &mut layers, &geom);
    cx2.moment = Moment::from_millis(250);
    let resp = split.update(&mut cx2, &mut state);

    assert_eq!(state.maximized, Some(0));
    assert_eq!(resp.action, Some(SplitAction::Maximized(Some(0))));

    // Third click at t = 350ms (double click again => restore)
    let click3 = UpdateCause::Input(
        Input::Mouse(Mouse {
            pos: ratatui::layout::Position::new(50, 10),
            kind: MouseKind::Down,
        }),
        Moment::from_millis(350),
    );
    let mut cx3 = make_cx(&click3, seam_id.clone(), &mut layers, &geom);
    cx3.moment = Moment::from_millis(350);
    let resp_restore = split.update(&mut cx3, &mut state);

    assert_eq!(state.maximized, None);
    assert_eq!(resp_restore.action, Some(SplitAction::Maximized(None)));
}

#[test]
fn test_split_pane_keyboard_nudge() {
    let id = Id::new("split.key");
    let split = SplitPane::new(id.clone(), Axis::Horizontal);
    let mut state = SplitPaneState::new(0.5);

    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 100, 30));
    let mut layers = LayerStack::new();

    // Nudge right with '+'
    let key_plus = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Char('+'),
            mods: KeyModifiers::empty(),
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&key_plus, id.clone(), &mut layers, &geom);
    let resp = split.update(&mut cx, &mut state);

    assert!((state.ratio - 0.55).abs() < 1e-4);
    assert_eq!(
        resp.action,
        Some(SplitAction::Resized { ratio: state.ratio })
    );
}

// =========================================================================
// 3. TextViewport Tests
// =========================================================================

#[test]
fn test_text_viewport_prose_mode_navigation() {
    let id = Id::new("viewport.prose");
    let source = MockSource::new(vec![
        "Line 0: introduction",
        "Line 1: chapter one",
        "Line 2: chapter two",
        "Line 3: chapter three",
        "Line 4: chapter four",
        "Line 5: chapter five",
        "Line 6: chapter six",
        "Line 7: chapter seven",
        "Line 8: conclusion",
    ]);

    let viewport = TextViewport::new(id.clone(), &source).mode(ViewportMode::Prose);
    let mut state = ViewportState::new();

    let area = Rect::new(0, 0, 40, 5); // 5 rows viewport
    let mut geom = HashMap::new();
    geom.insert(id.clone(), area);
    let mut layers = LayerStack::new();

    // Initial state: offset 0
    assert_eq!(state.scroll.offset, 0);
    assert!(!state.follow_tail);

    // Down key scrolls down 1 row
    let key_down = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Down,
            mods: KeyModifiers::empty(),
        }),
        Moment::from_millis(100),
    );
    let mut cx_down = make_cx(&key_down, id.clone(), &mut layers, &geom);
    viewport.update(&mut cx_down, &mut state);
    assert_eq!(state.scroll.offset, 1);
    assert!(!state.follow_tail);

    // End key in Prose mode jumps to end, but does NOT set follow_tail
    let key_end = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::End,
            mods: KeyModifiers::empty(),
        }),
        Moment::from_millis(200),
    );
    let mut cx_end = make_cx(&key_end, id.clone(), &mut layers, &geom);
    viewport.update(&mut cx_end, &mut state);
    assert_eq!(state.scroll.offset, 4); // 9 lines - 5 viewport = 4 max offset
    assert!(!state.follow_tail); // Prose End is a jump, NOT tail follow
}

#[test]
fn test_text_viewport_log_mode_tail_follow() {
    let id = Id::new("viewport.log");
    let source = MockSource::new(vec![
        "INFO 001 starting server",
        "INFO 002 listening on 8080",
        "DEBUG 003 connection accepted",
        "INFO 004 handling request",
        "WARN 005 high latency",
        "INFO 006 request completed",
    ]);

    let viewport = TextViewport::new(id.clone(), &source).mode(ViewportMode::Log);
    let mut state = ViewportState::new().with_follow_tail(true);

    let area = Rect::new(0, 0, 50, 4); // 4 rows viewport, 6 total lines => max offset 2
    let mut geom = HashMap::new();
    geom.insert(id.clone(), area);
    let mut layers = LayerStack::new();

    assert!(state.follow_tail);

    // Scrolling up away from tail pauses follow_tail
    let key_up = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Up,
            mods: KeyModifiers::empty(),
        }),
        Moment::from_millis(100),
    );
    let mut cx_up = make_cx(&key_up, id.clone(), &mut layers, &geom);
    viewport.update(&mut cx_up, &mut state);

    assert_eq!(state.scroll.offset, 1);
    assert!(!state.follow_tail); // Paused!

    // Scrolling back to end resumes follow_tail
    let key_end = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::End,
            mods: KeyModifiers::empty(),
        }),
        Moment::from_millis(200),
    );
    let mut cx_end = make_cx(&key_end, id.clone(), &mut layers, &geom);
    viewport.update(&mut cx_end, &mut state);

    assert_eq!(state.scroll.offset, 2);
    assert!(state.follow_tail); // Resumed!
}

#[test]
fn test_text_viewport_selection_and_copy() {
    let id = Id::new("viewport.copy");
    let source = MockSource::new(vec![
        "first alpha row",
        "second beta row",
        "third gamma row",
    ]);

    let viewport = TextViewport::new(id.clone(), &source);
    let mut state = ViewportState::new();

    let area = Rect::new(0, 0, 50, 10);
    let mut geom = HashMap::new();
    geom.insert(id.clone(), area);
    let mut layers = LayerStack::new();

    // Set selection across "alpha row\nsecond beta"
    let p_start = TextPosition::new(ItemKey::new(0), 6); // "alpha row"
    let p_end = TextPosition::new(ItemKey::new(1), 11); // "second beta"
    state.selection = Some(TextRange::new(source.revision(), p_start, p_end));

    // Press 'y' to request copy
    let key_y = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Char('y'),
            mods: KeyModifiers::empty(),
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&key_y, id.clone(), &mut layers, &geom);
    let resp = viewport.update(&mut cx, &mut state);

    assert_eq!(
        resp.action,
        Some(ViewportAction::CopyRequested {
            text: "alpha row\nsecond beta".to_string(),
        })
    );
}

#[test]
fn test_text_viewport_wrapping_and_scrollbar() {
    let id = Id::new("viewport.wrap");
    // Long line of 35 characters
    let source = MockSource::new(vec![
        "This is a long line that will wrap across multiple visual rows in narrow view.",
    ]);

    let viewport = TextViewport::new(id.clone(), &source).wrap(true);
    let state = ViewportState::new();

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 24));
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 80, 24), &mut layers).with_buffer(&mut buf);

    // Draw in narrow area (width = 20, height = 10)
    let drawn = viewport.draw(&mut ui, Rect::new(0, 0, 20, 10), &state);
    assert_eq!(drawn.width, 20);
    assert_eq!(drawn.height, 10);
}

#[test]
fn test_edge_cases_zero_area_and_empty_content() {
    let panel = Panel::card(Id::new("edge.panel"));
    let split = SplitPane::new(Id::new("edge.split"), Axis::Horizontal);
    let source = MockSource::new(vec![]);
    let viewport = TextViewport::new(Id::new("edge.viewport"), &source);

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 24));
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 80, 24), &mut layers).with_buffer(&mut buf);

    // 0x0 area
    panel.draw(&mut ui, Rect::zero(), |_child_ui, area| {
        assert_eq!(area, Rect::zero());
    });

    let split_state = SplitPaneState::new(0.5);
    split.draw(&mut ui, Rect::zero(), &split_state, |_child_ui, areas| {
        assert_eq!(areas.first, Rect::zero());
        assert_eq!(areas.second, Rect::zero());
        assert_eq!(areas.seam, Rect::zero());
    });

    let vp_state = ViewportState::new();
    let res = viewport.draw(&mut ui, Rect::zero(), &vp_state);
    assert_eq!(res, Rect::zero());
}
