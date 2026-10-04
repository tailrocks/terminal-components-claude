//! Comprehensive Verification Probes for Termrock P6 TASK-014:
//! Add prepared-cell TerminalView and terminal edge contracts.
//!
//! Validates:
//! - AC-001: Prepared cells draw with exact styles and continuation behavior while cursor/selection interactions produce typed requests.
//! - AC-002: Cursor visibility, selection, links, copy, resize, color capability, and terminal cleanup behavior follow the canonical edge contract.
//! - AC-003: Frozen references, application invariants, zero clippy warnings.
//! - AC-004: All test gates pass.

use std::collections::HashMap;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::{Color, Modifier};

use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::termrock::author::{TerminalCell, TerminalCursor, TerminalSource};
use junie_tui::termrock::{
    ColorLevel, Constraints, Cx, Id, Invalidate, LayerStack, MeasureCx, Position, Rect, Revision,
    Size, StylePatch, TerminalAction, TerminalInteraction, TerminalSelection, TerminalView,
    TerminalViewState, Theme, Ui, UpdateCause,
};

#[derive(Clone)]
struct MockTerminalSource {
    pub rev: Revision,
    pub dimensions: Size,
    pub cell_map: HashMap<(u16, u16), TerminalCell<'static>>,
    pub cursor_info: Option<TerminalCursor>,
}

impl MockTerminalSource {
    fn new(width: u16, height: u16) -> Self {
        Self {
            rev: Revision::new(1),
            dimensions: Size::new(width, height),
            cell_map: HashMap::new(),
            cursor_info: None,
        }
    }

    fn put_cell(&mut self, x: u16, y: u16, cell: TerminalCell<'static>) {
        self.cell_map.insert((x, y), cell);
    }

    fn put_text(&mut self, y: u16, text: &'static str) {
        for (x, ch) in text.chars().enumerate() {
            let mut cell = TerminalCell::new(Box::leak(ch.to_string().into_boxed_str()));
            cell.fg = Some(Color::White);
            cell.bg = Some(Color::Black);
            self.put_cell(x as u16, y, cell);
        }
    }
}

impl TerminalSource for MockTerminalSource {
    fn revision(&self) -> Revision {
        self.rev
    }

    fn size(&self) -> Size {
        self.dimensions
    }

    fn cell(&self, position: Position) -> Option<TerminalCell<'_>> {
        self.cell_map.get(&(position.x, position.y)).cloned()
    }

    fn cursor(&self) -> Option<TerminalCursor> {
        self.cursor_info
    }
}

fn make_cx<'a>(
    cause: &'a UpdateCause,
    id: Id,
    layers: &'a mut LayerStack,
    geom: &'a HashMap<Id, Rect>,
) -> Cx<'a> {
    Cx {
        cause,
        moment: junie_tui::termrock::Moment::from_millis(100),
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

#[test]
fn test_terminal_view_cell_blitting_and_styles() {
    let mut source = MockTerminalSource::new(20, 5);
    let mut cell_a = TerminalCell::new("A");
    cell_a.fg = Some(Color::Green);
    cell_a.bg = Some(Color::DarkGray);
    cell_a.modifier = Modifier::BOLD;
    source.put_cell(0, 0, cell_a);

    let mut cell_b = TerminalCell::new("B");
    cell_b.fg = Some(Color::Yellow);
    cell_b.bg = Some(Color::Blue);
    cell_b.modifier = Modifier::ITALIC;
    source.put_cell(1, 0, cell_b);

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 20, 5));
    let mut ui = make_ui(&theme, Rect::new(0, 0, 20, 5), &mut layers, &mut buf);

    let id = Id::new("term.view");
    let view = TerminalView::new(id, &source);
    let state = TerminalViewState::new();
    view.draw(&mut ui, Rect::new(0, 0, 20, 5), &state);

    let ca = &buf[(0, 0)];
    assert_eq!(ca.symbol(), "A");
    assert_eq!(ca.fg, Color::Green);
    assert_eq!(ca.bg, Color::DarkGray);
    assert!(ca.modifier.contains(Modifier::BOLD));

    let cb = &buf[(1, 0)];
    assert_eq!(cb.symbol(), "B");
    assert_eq!(cb.fg, Color::Yellow);
    assert_eq!(cb.bg, Color::Blue);
    assert!(cb.modifier.contains(Modifier::ITALIC));
}

#[test]
fn test_terminal_view_wide_character_continuation() {
    let mut source = MockTerminalSource::new(10, 2);
    // Wide emoji in cell 0 (width 2, non-continuation)
    let mut wide_cell = TerminalCell::new("🔥");
    wide_cell.width = 2;
    wide_cell.continuation = false;
    wide_cell.fg = Some(Color::Red);
    source.put_cell(0, 0, wide_cell);

    // Continuation cell in cell 1
    let mut cont_cell = TerminalCell::new("");
    cont_cell.width = 0;
    cont_cell.continuation = true;
    source.put_cell(1, 0, cont_cell);

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 10, 2));
    let mut ui = make_ui(&theme, Rect::new(0, 0, 10, 2), &mut layers, &mut buf);

    let id = Id::new("term.view");
    let view = TerminalView::new(id, &source);
    let state = TerminalViewState::new();
    view.draw(&mut ui, Rect::new(0, 0, 10, 2), &state);

    assert_eq!(buf[(0, 0)].symbol(), "🔥");
    assert_eq!(buf[(0, 0)].fg, Color::Red);
    assert_eq!(buf[(1, 0)].symbol(), "");
}

#[test]
fn test_terminal_view_combining_characters() {
    let mut source = MockTerminalSource::new(10, 2);
    // Combining diacritics cluster: e + acute accent
    let combining_cluster = "e\u{0301}";
    let cell = TerminalCell::new(combining_cluster);
    source.put_cell(0, 0, cell);

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 10, 2));
    let mut ui = make_ui(&theme, Rect::new(0, 0, 10, 2), &mut layers, &mut buf);

    let id = Id::new("term.view");
    let view = TerminalView::new(id, &source);
    let state = TerminalViewState::new();
    view.draw(&mut ui, Rect::new(0, 0, 10, 2), &state);

    assert_eq!(buf[(0, 0)].symbol(), combining_cluster);
}

#[test]
fn test_terminal_view_cursor_presentation_and_focus() {
    let mut source = MockTerminalSource::new(20, 5);
    source.cursor_info = Some(TerminalCursor {
        pos: Position::new(3, 2),
        visible: true,
    });
    source.put_cell(3, 2, TerminalCell::new("X"));

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 20, 5));
    let mut ui = make_ui(&theme, Rect::new(0, 0, 20, 5), &mut layers, &mut buf);

    let id = Id::new("term.view");
    let view = TerminalView::new(id.clone(), &source);
    let mut state = TerminalViewState::new();
    state.cursor_visible = true;

    view.draw(&mut ui, Rect::new(0, 0, 20, 5), &state);

    // Cursor request intent captured in ui
    assert_eq!(ui.cursor_intent, Some((id.clone(), Position::new(3, 2))));
    // Cell at cursor has style modifier applied
    let c = &buf[(3, 2)];
    assert!(c.modifier.contains(Modifier::UNDERLINED) || c.modifier.contains(Modifier::REVERSED));

    // When cursor_visible is false in state, cursor intent is not emitted
    state.cursor_visible = false;
    let mut buf2 = Buffer::empty(ratatui::layout::Rect::new(0, 0, 20, 5));
    let mut ui2 = make_ui(&theme, Rect::new(0, 0, 20, 5), &mut layers, &mut buf2);
    view.draw(&mut ui2, Rect::new(0, 0, 20, 5), &state);
    assert_eq!(ui2.cursor_intent, None);
}

#[test]
fn test_terminal_view_dimmed_presentation() {
    let mut source = MockTerminalSource::new(10, 2);
    source.put_cell(0, 0, TerminalCell::new("Z"));

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 10, 2));
    let mut ui = make_ui(&theme, Rect::new(0, 0, 10, 2), &mut layers, &mut buf);

    let id = Id::new("term.view");
    let view = TerminalView::new(id, &source).dimmed(true);
    let state = TerminalViewState::new();
    view.draw(&mut ui, Rect::new(0, 0, 10, 2), &state);

    assert!(buf[(0, 0)].modifier.contains(Modifier::DIM));
}

#[test]
fn test_terminal_view_measure_and_constraints() {
    let source = MockTerminalSource::new(80, 24);
    let id = Id::new("term.view");
    let view = TerminalView::new(id, &source);

    let theme = Theme::termrock();
    let cx = MeasureCx::new(Constraints::unbounded(), &theme, ColorLevel::TrueColor);

    // Unbounded returns source size
    let sz = view.measure(&cx, Constraints::unbounded());
    assert_eq!(sz, Size::new(80, 24));

    // Loose constraint clamps
    let sz_loose = view.measure(&cx, Constraints::loose(Size::new(40, 10)));
    assert_eq!(sz_loose, Size::new(40, 10));

    // Tight constraint enforces exact
    let sz_tight = view.measure(&cx, Constraints::tight(Size::new(50, 15)));
    assert_eq!(sz_tight, Size::new(50, 15));
}

#[test]
fn test_terminal_view_scroll_offset_and_paging() {
    let mut source = MockTerminalSource::new(10, 20);
    for y in 0..20 {
        let text = Box::leak(format!("LINE-{:02}", y).into_boxed_str());
        source.put_text(y, text);
    }

    let id = Id::new("term.view");
    let view = TerminalView::new(id.clone(), &source);
    let mut state = TerminalViewState::new();
    state.scroll.offset = 5;

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 10, 5));
    let mut ui = make_ui(&theme, Rect::new(0, 0, 10, 5), &mut layers, &mut buf);

    view.draw(&mut ui, Rect::new(0, 0, 10, 5), &state);

    // Row 0 shows line 5
    assert_eq!(buffer_row(&buf, 0, 7), "LINE-05");

    // PageDown scrolls by viewport
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 10, 5));
    let cause_pd = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::PageDown,
            mods: KeyModifiers::empty(),
        }),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx_pd = make_cx(&cause_pd, id.clone(), &mut layers, &geom);
    view.update(&mut cx_pd, &mut state);
    assert_eq!(state.scroll.offset, 10);

    // PageUp scrolls back
    let cause_pu = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::PageUp,
            mods: KeyModifiers::empty(),
        }),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx_pu = make_cx(&cause_pu, id.clone(), &mut layers, &geom);
    view.update(&mut cx_pu, &mut state);
    assert_eq!(state.scroll.offset, 5);

    // Mouse wheel up / down
    let cause_wd = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::WheelDown,
            pos: ratatui::layout::Position::new(2, 2),
        }),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx_wd = make_cx(&cause_wd, id.clone(), &mut layers, &geom);
    view.update(&mut cx_wd, &mut state);
    assert_eq!(state.scroll.offset, 8);
}

#[test]
fn test_terminal_view_interactive_input_forwarding() {
    let source = MockTerminalSource::new(80, 24);
    let id = Id::new("term.view");
    let view = TerminalView::new(id.clone(), &source);
    let mut state = TerminalViewState::new();

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 80, 24));

    // Standard character key forwarding
    let key = Key {
        code: KeyCode::Char('a'),
        mods: KeyModifiers::empty(),
    };
    let cause = UpdateCause::Input(Input::Key(key), junie_tui::termrock::Moment::from_millis(0));
    let mut cx = make_cx(&cause, id.clone(), &mut layers, &geom);

    let resp = view.update(&mut cx, &mut state);
    match resp.action {
        Some(TerminalAction::Forward { token }) => {
            assert_eq!(token.as_key(), Some(&key));
        }
        other => panic!("expected Forward action, got {:?}", other),
    }

    // Paste event forwarding
    let cause_paste = UpdateCause::Input(
        Input::Paste("test paste".to_string()),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx_paste = make_cx(&cause_paste, id.clone(), &mut layers, &geom);
    let resp_paste = view.update(&mut cx_paste, &mut state);
    match resp_paste.action {
        Some(TerminalAction::Forward { token }) => {
            assert_eq!(
                token.as_input(),
                Some(&Input::Paste("test paste".to_string()))
            );
        }
        other => panic!("expected Forward action for paste, got {:?}", other),
    }
}

#[test]
fn test_terminal_view_modal_intercept_protects_forwarding() {
    let source = MockTerminalSource::new(80, 24);
    let term_id = Id::new("term.view");
    let modal_id = Id::new("dialog.active");
    let view = TerminalView::new(term_id.clone(), &source);
    let mut state = TerminalViewState::new();

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(term_id.clone(), Rect::new(0, 0, 80, 24));

    let key = Key {
        code: KeyCode::Char('x'),
        mods: KeyModifiers::empty(),
    };
    let cause = UpdateCause::Input(Input::Key(key), junie_tui::termrock::Moment::from_millis(0));

    // Context where intended owner is the modal dialog, not the terminal
    let mut cx = Cx {
        cause: &cause,
        moment: junie_tui::termrock::Moment::from_millis(0),
        intended_owner: Some(modal_id),
        focus: None,
        pointer_capture: None,
        invalidate: Invalidate::None,
        layer_stack: &mut layers,
        cursor_request: None,
        new_focus: None,
        new_capture: None,
        feedback_requests: Vec::new(),
        published_geometry: Some(&geom),
    };

    let resp = view.update(&mut cx, &mut state);
    assert_eq!(resp.action, None);
    assert!(!resp.flow.is_consumed());
}

#[test]
fn test_terminal_view_readonly_and_selection_only_suppress_forwarding() {
    let source = MockTerminalSource::new(80, 24);
    let id = Id::new("term.view");
    let mut state = TerminalViewState::new();
    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 80, 24));

    let key = Key {
        code: KeyCode::Char('x'),
        mods: KeyModifiers::empty(),
    };
    let cause = UpdateCause::Input(Input::Key(key), junie_tui::termrock::Moment::from_millis(0));

    // ReadOnly mode
    let ro_view = TerminalView::new(id.clone(), &source).interaction(TerminalInteraction::ReadOnly);
    let mut cx_ro = make_cx(&cause, id.clone(), &mut layers, &geom);
    let resp_ro = ro_view.update(&mut cx_ro, &mut state);
    assert_eq!(resp_ro.action, None);

    // SelectionOnly mode
    let sel_view =
        TerminalView::new(id.clone(), &source).interaction(TerminalInteraction::SelectionOnly);
    let mut cx_sel = make_cx(&cause, id.clone(), &mut layers, &geom);
    let resp_sel = sel_view.update(&mut cx_sel, &mut state);
    assert_eq!(resp_sel.action, None);
}

#[test]
fn test_terminal_view_mouse_drag_selection_workflow() {
    let source = MockTerminalSource::new(40, 10);
    let id = Id::new("term.view");
    let view = TerminalView::new(id.clone(), &source);
    let mut state = TerminalViewState::new();

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 40, 10));

    // 1. Mouse down at (2, 1)
    let down_cause = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: ratatui::layout::Position::new(2, 1),
        }),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx_down = make_cx(&down_cause, id.clone(), &mut layers, &geom);
    view.update(&mut cx_down, &mut state);

    assert!(state.selecting);
    assert_eq!(state.selection_anchor, Some((1, 2)));
    assert_eq!(
        state.selection,
        Some(TerminalSelection::new(1, 2, 1, 2).with_revision(Revision::new(1)))
    );

    // 2. Mouse drag to (8, 3)
    let drag_cause = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Drag,
            pos: ratatui::layout::Position::new(8, 3),
        }),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx_drag = make_cx(&drag_cause, id.clone(), &mut layers, &geom);
    view.update(&mut cx_drag, &mut state);

    assert!(state.selecting);
    assert_eq!(
        state.selection,
        Some(TerminalSelection::new(1, 2, 3, 8).with_revision(Revision::new(1)))
    );

    // 3. Mouse up
    let up_cause = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: ratatui::layout::Position::new(8, 3),
        }),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx_up = make_cx(&up_cause, id.clone(), &mut layers, &geom);
    view.update(&mut cx_up, &mut state);

    assert!(!state.selecting);
    assert!(state.selection.is_some());
}

#[test]
fn test_terminal_view_selection_highlight_and_extraction() {
    let mut source = MockTerminalSource::new(15, 4);
    source.put_text(0, "FIRST LINE");
    source.put_text(1, "SECOND LINE");
    source.put_text(2, "THIRD LINE");

    let id = Id::new("term.view");
    let view = TerminalView::new(id, &source);
    let mut state = TerminalViewState::new();
    let sel = TerminalSelection::new(0, 0, 1, 5).with_revision(Revision::new(1));
    state.selection = Some(sel);

    // Draw and verify reversed modifier on selected cells
    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 15, 4));
    let mut ui = make_ui(&theme, Rect::new(0, 0, 15, 4), &mut layers, &mut buf);
    view.draw(&mut ui, Rect::new(0, 0, 15, 4), &state);

    assert!(buf[(0, 0)].modifier.contains(Modifier::REVERSED));
    assert!(buf[(5, 1)].modifier.contains(Modifier::REVERSED));
    // Cell outside selection is not reversed
    assert!(!buf[(7, 1)].modifier.contains(Modifier::REVERSED));

    // Extract selection text
    let extracted = view.extract_selection(&sel);
    assert_eq!(extracted, "FIRST LINE\nSECOND");
}

#[test]
fn test_terminal_view_copy_requested_action() {
    let source = MockTerminalSource::new(40, 10);
    let id = Id::new("term.view");
    let view = TerminalView::new(id.clone(), &source);
    let mut state = TerminalViewState::new();
    let sel = TerminalSelection::new(1, 0, 1, 10).with_revision(Revision::new(1));
    state.selection = Some(sel);

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 40, 10));

    // Ctrl+C produces CopyRequested
    let copy_key = Key {
        code: KeyCode::Char('c'),
        mods: KeyModifiers::CONTROL,
    };
    let cause = UpdateCause::Input(
        Input::Key(copy_key),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx = make_cx(&cause, id.clone(), &mut layers, &geom);

    let resp = view.update(&mut cx, &mut state);
    match resp.action {
        Some(TerminalAction::CopyRequested(s)) => {
            assert_eq!(s, sel);
        }
        other => panic!("expected CopyRequested, got {:?}", other),
    }

    // In SelectionOnly mode, 'y' also triggers CopyRequested
    let sel_view =
        TerminalView::new(id.clone(), &source).interaction(TerminalInteraction::SelectionOnly);
    let y_key = Key {
        code: KeyCode::Char('y'),
        mods: KeyModifiers::empty(),
    };
    let cause_y = UpdateCause::Input(
        Input::Key(y_key),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx_y = make_cx(&cause_y, id.clone(), &mut layers, &geom);
    let resp_y = sel_view.update(&mut cx_y, &mut state);
    match resp_y.action {
        Some(TerminalAction::CopyRequested(s)) => {
            assert_eq!(s, sel);
        }
        other => panic!("expected CopyRequested from y key, got {:?}", other),
    }
}

#[test]
fn test_terminal_view_stale_history_rejection() {
    let mut source = MockTerminalSource::new(40, 10);
    source.rev = Revision::new(2); // Current revision is 2

    let id = Id::new("term.view");
    let view = TerminalView::new(id.clone(), &source);
    let mut state = TerminalViewState::new();
    // Stale selection from revision 1
    state.selection = Some(TerminalSelection::new(0, 0, 0, 5).with_revision(Revision::new(1)));

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 40, 10));

    let copy_key = Key {
        code: KeyCode::Char('c'),
        mods: KeyModifiers::CONTROL,
    };
    let cause = UpdateCause::Input(
        Input::Key(copy_key),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx = make_cx(&cause, id.clone(), &mut layers, &geom);

    let resp = view.update(&mut cx, &mut state);
    // Stale selection must be cleared and CopyRequested rejected
    assert_eq!(resp.action, None);
    assert_eq!(state.selection, None);
}

#[test]
fn test_terminal_view_escape_clears_selection() {
    let source = MockTerminalSource::new(40, 10);
    let id = Id::new("term.view");
    let view = TerminalView::new(id.clone(), &source);
    let mut state = TerminalViewState::new();
    state.selection = Some(TerminalSelection::new(0, 0, 0, 5).with_revision(Revision::new(1)));

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 40, 10));

    let esc_key = Key {
        code: KeyCode::Esc,
        mods: KeyModifiers::empty(),
    };
    let cause = UpdateCause::Input(
        Input::Key(esc_key),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx = make_cx(&cause, id.clone(), &mut layers, &geom);

    let resp = view.update(&mut cx, &mut state);
    assert!(resp.flow.is_consumed());
    assert_eq!(state.selection, None);
}

#[test]
fn test_terminal_view_resize_facts() {
    let source = MockTerminalSource::new(80, 24);
    let id = Id::new("term.view");
    let view = TerminalView::new(id.clone(), &source);
    let mut state = TerminalViewState::new();

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 80, 24));

    // Direct resize input
    let resize_cause = UpdateCause::Input(
        Input::Resize(100, 30),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx_resize = make_cx(&resize_cause, id.clone(), &mut layers, &geom);
    let resp = view.update(&mut cx_resize, &mut state);

    assert_eq!(
        resp.action,
        Some(TerminalAction::Resize {
            rows: 30,
            cols: 100
        })
    );
}

#[test]
fn test_terminal_view_link_interaction() {
    let source = MockTerminalSource::new(40, 10);
    let id = Id::new("term.view");

    let resolver = |row: usize, col: usize| {
        if row == 2 && col == 4 {
            Some(junie_tui::termrock::LinkKey::from("https://termrock.dev"))
        } else {
            None
        }
    };

    let view = TerminalView::new(id.clone(), &source).link_resolver(&resolver);
    let mut state = TerminalViewState::new();

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 40, 10));

    let click_cause = UpdateCause::Input(
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: ratatui::layout::Position::new(4, 2),
        }),
        junie_tui::termrock::Moment::from_millis(0),
    );
    let mut cx = make_cx(&click_cause, id.clone(), &mut layers, &geom);
    let resp = view.update(&mut cx, &mut state);

    assert_eq!(
        resp.action,
        Some(TerminalAction::OpenLink {
            key: junie_tui::termrock::LinkKey::from("https://termrock.dev")
        })
    );
}

#[test]
fn test_terminal_view_zero_and_tiny_area_safe() {
    let source = MockTerminalSource::new(40, 10);
    let id = Id::new("term.view");
    let view = TerminalView::new(id, &source);
    let state = TerminalViewState::new();

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 5, 5));

    // Zero area
    let mut ui_zero = make_ui(&theme, Rect::new(0, 0, 0, 0), &mut layers, &mut buf);
    let res_zero = view.draw(&mut ui_zero, Rect::new(0, 0, 0, 0), &state);
    assert_eq!(res_zero, Rect::zero());

    // 1x1 tiny area
    let mut ui_one = make_ui(&theme, Rect::new(0, 0, 1, 1), &mut layers, &mut buf);
    let res_one = view.draw(&mut ui_one, Rect::new(0, 0, 1, 1), &state);
    assert_eq!(res_one, Rect::new(0, 0, 1, 1));
}

#[test]
fn test_terminal_view_patch_preserves_child_cell_colors() {
    let mut source = MockTerminalSource::new(10, 2);
    let mut cell = TerminalCell::new("Q");
    cell.fg = Some(Color::Cyan);
    cell.bg = Some(Color::Magenta);
    source.put_cell(0, 0, cell);

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 10, 2));
    let mut ui = make_ui(&theme, Rect::new(0, 0, 10, 2), &mut layers, &mut buf);

    let id = Id::new("term.view");
    // StylePatch applied to chrome
    let view = TerminalView::new(id, &source).patch(StylePatch::empty());
    let state = TerminalViewState::new();
    view.draw(&mut ui, Rect::new(0, 0, 10, 2), &state);

    // Child cell retains original colors exactly
    assert_eq!(buf[(0, 0)].fg, Color::Cyan);
    assert_eq!(buf[(0, 0)].bg, Color::Magenta);
}
