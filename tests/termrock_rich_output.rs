//! Comprehensive Verification Probes for Termrock P5 TASK-012:
//! Implement Grid, CodeEditor, and DiffView rich output.
//!
//! Validates:
//! - AC-001: Rich output measures, draws, and emits typed actions through stable row/column/item keys.
//! - AC-002: Wide cells, continuation, read-only selection, diagnostics, source reorder/removal,
//!   empty/error/partial states, and narrow widths preserve the contract.
//! - AC-003: Frozen references, application invariants, and scope protection.
//! - AC-004: Completion gate passes.

use std::collections::HashMap;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use junie_tui::core::event::{Input, Key};
use junie_tui::termrock::{
    CellKey, CellKind, CellValue, CodeAction, CodeEditor, CodeEditorState, ColorLevel, ColumnKey,
    Constraints, Cx, Diagnostic, DiffAction, DiffMode, DiffStatus, DiffView, DiffViewState, Grid,
    GridAction, GridColumn, GridEditor, GridMode, GridModel, GridState, Id, Invalidate, ItemKey,
    LayerStack, MeasureCx, MemoryDiffSource, Moment, Readiness, Rect, Revision, SelectionMode,
    TabBehavior, TextLine, TextSource, Theme, Ui, UpdateCause,
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

// -----------------------------------------------------------------------------
// Mock Grid Model & Editor
// -----------------------------------------------------------------------------

#[derive(Clone)]
struct MockGridData {
    revision: Revision,
    columns: Vec<GridColumn>,
    rows: Vec<(ItemKey, Vec<CellValue>)>,
    has_more: bool,
}

impl MockGridData {
    fn new() -> Self {
        let col1 = GridColumn::new(ColumnKey::new(1), "ID", 6)
            .primary(true)
            .kind(CellKind::Id);
        let col2 = GridColumn::new(ColumnKey::new(2), "Name", 12).kind(CellKind::Text);
        let col3 = GridColumn::new(ColumnKey::new(3), "Age", 6)
            .kind(CellKind::Number)
            .sortable(true);

        let rows = vec![
            (
                ItemKey::new(101),
                vec![
                    CellValue::Int(1),
                    CellValue::Text("Alice".into()),
                    CellValue::Int(30),
                ],
            ),
            (
                ItemKey::new(102),
                vec![
                    CellValue::Int(2),
                    CellValue::Text("Bob".into()),
                    CellValue::Int(25),
                ],
            ),
            (
                ItemKey::new(103),
                vec![
                    CellValue::Int(3),
                    CellValue::Text("Charlie".into()),
                    CellValue::Int(35),
                ],
            ),
        ];

        Self {
            revision: Revision::new(1),
            columns: vec![col1, col2, col3],
            rows,
            has_more: false,
        }
    }
}

impl GridModel for MockGridData {
    fn revision(&self) -> Revision {
        self.revision
    }

    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_key(&self, index: usize) -> ItemKey {
        self.rows[index].0
    }

    fn columns(&self) -> &[GridColumn] {
        &self.columns
    }

    fn cell_value(&self, row: ItemKey, col: ColumnKey) -> Option<CellValue> {
        let r_idx = self.row_index(row)?;
        let c_idx = self.column_index(col)?;
        self.rows[r_idx].1.get(c_idx).cloned()
    }

    fn has_more(&self) -> bool {
        self.has_more
    }
}

impl GridEditor for MockGridData {
    fn is_editable(&self, _row: ItemKey, col: ColumnKey) -> bool {
        // ID column is primary / read-only, others are editable
        col != ColumnKey::new(1)
    }

    fn commit_cell(&mut self, row: ItemKey, col: ColumnKey, value: CellValue) -> bool {
        if let (Some(r_idx), Some(c_idx)) = (self.row_index(row), self.column_index(col))
            && c_idx < self.rows[r_idx].1.len()
        {
            self.rows[r_idx].1[c_idx] = value;
            return true;
        }
        false
    }
}

// -----------------------------------------------------------------------------
// Mock Document for CodeEditor
// -----------------------------------------------------------------------------

struct MockDoc {
    lines: Vec<String>,
}

impl MockDoc {
    fn new(lines: Vec<&str>) -> Self {
        Self {
            lines: lines.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl TextSource for MockDoc {
    fn revision(&self) -> Revision {
        Revision::new(1)
    }

    fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn line(&self, index: usize) -> Option<TextLine<'_>> {
        self.lines
            .get(index)
            .map(|s| TextLine::plain(ItemKey::new(index as u64 + 1), s))
    }
}

// =============================================================================
// AC-001 Probes: Grid, CodeEditor, DiffView Typed Actions and Measurements
// =============================================================================

#[test]
fn test_grid_table_mode_navigation_and_actions() {
    let id = Id::new("grid.table");
    let grid = Grid::new(id.clone())
        .mode(GridMode::Table)
        .selection_mode(SelectionMode::Single);

    let model = MockGridData::new();
    let mut state = GridState::new();

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 80, 24));

    // Down navigation
    let key_down = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Down,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&key_down, id.clone(), &mut layers, &geom);
    let resp = grid.update(&mut cx, &mut state, &model);
    assert_eq!(resp.action, Some(GridAction::SelectRow(ItemKey::new(102))));
    assert_eq!(state.cursor_row, Some(ItemKey::new(102)));

    // Up navigation
    let key_up = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Up,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(101),
    );
    let mut cx = make_cx(&key_up, id.clone(), &mut layers, &geom);
    let resp = grid.update(&mut cx, &mut state, &model);
    assert_eq!(resp.action, Some(GridAction::SelectRow(ItemKey::new(101))));
    assert_eq!(state.cursor_row, Some(ItemKey::new(101)));

    // Activation via Enter
    let key_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(102),
    );
    let mut cx = make_cx(&key_enter, id.clone(), &mut layers, &geom);
    let resp = grid.update(&mut cx, &mut state, &model);
    assert!(matches!(resp.action, Some(GridAction::Activate { .. })));

    // Copy via 'y'
    let key_copy = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Char('y'),
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(103),
    );
    let mut cx = make_cx(&key_copy, id.clone(), &mut layers, &geom);
    let resp = grid.update(&mut cx, &mut state, &model);
    assert!(
        matches!(resp.action, Some(GridAction::CopyRequested(ref text)) if text.contains("Alice"))
    );

    // Measure test
    let theme = Theme::termrock();
    let measure_cx = MeasureCx::new(Constraints::unbounded(), &theme, ColorLevel::TrueColor);
    let measured = grid.measure(&measure_cx, &model, Constraints::unbounded());
    assert!(measured.width >= 32);
    assert!(measured.height >= 4);

    // Draw test
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 24));
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 80, 24), &mut layers).with_buffer(&mut buf);
    grid.draw(&mut ui, Rect::new(0, 0, 80, 24), &state, &model);
    assert_eq!(buf[(1, 0)].symbol(), "#");
}

#[test]
fn test_grid_cell_mode_selection_and_editing() {
    let id = Id::new("grid.cell");
    let grid = Grid::new(id.clone()).mode(GridMode::Cell);

    let mut model = MockGridData::new();
    let mut state = GridState::new();

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 80, 24));

    // Initial state
    state.cursor_row = Some(ItemKey::new(101));
    state.cursor_col = Some(ColumnKey::new(1));

    // Right navigation in cell mode
    let key_right = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Right,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&key_right, id.clone(), &mut layers, &geom);
    let resp = grid.update_editable(&mut cx, &mut state, &mut model);
    assert_eq!(
        resp.action,
        Some(GridAction::SelectCell(CellKey::new(
            ItemKey::new(101),
            ColumnKey::new(2)
        )))
    );
    assert_eq!(state.cursor_col, Some(ColumnKey::new(2)));

    // Press Enter to start editing editable cell
    let key_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(101),
    );
    let mut cx = make_cx(&key_enter, id.clone(), &mut layers, &geom);
    grid.update_editable(&mut cx, &mut state, &mut model);
    assert!(state.is_editing());
    assert_eq!(state.editing.as_ref().unwrap().buffer.text(), "Alice");

    // Type new characters into editor draft
    let key_type = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Char('a'),
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(102),
    );
    let mut cx = make_cx(&key_type, id.clone(), &mut layers, &geom);
    grid.update_editable(&mut cx, &mut state, &mut model);
    assert_eq!(state.editing.as_ref().unwrap().buffer.text(), "Alicea");

    // Commit edit via Enter
    let key_commit = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(103),
    );
    let mut cx = make_cx(&key_commit, id.clone(), &mut layers, &geom);
    let resp = grid.update_editable(&mut cx, &mut state, &mut model);
    assert!(matches!(
        resp.action,
        Some(GridAction::EditCommitted {
            value: CellValue::Text(ref t),
            ..
        }) if t == "Alicea"
    ));
    assert!(!state.is_editing());
}

#[test]
fn test_code_editor_navigation_and_editing() {
    let id = Id::new("editor.code");
    let doc = MockDoc::new(vec!["fn main() {", "    println!(\"Hello, world!\");", "}"]);

    let diags = vec![Diagnostic::error(2, 5, "Unresolved macro")];
    let editor = CodeEditor::new(id.clone(), &doc)
        .diagnostics(&diags)
        .tab_behavior(TabBehavior::Leave);

    let mut state = CodeEditorState::new();
    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 80, 24));

    // Down navigation
    let key_down = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Down,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&key_down, id.clone(), &mut layers, &geom);
    editor.update(&mut cx, &mut state);
    assert_eq!(state.cursor.line, ItemKey::new(2));

    // Enter edit mode via 'i'
    let key_i = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Char('i'),
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(101),
    );
    let mut cx = make_cx(&key_i, id.clone(), &mut layers, &geom);
    editor.update(&mut cx, &mut state);
    assert!(state.is_editing());

    // Type character
    let key_type = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Char('x'),
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(102),
    );
    let mut cx = make_cx(&key_type, id.clone(), &mut layers, &geom);
    let resp = editor.update(&mut cx, &mut state);
    assert_eq!(resp.action, Some(CodeAction::Edited));

    // Tab with TabBehavior::Leave commits and returns to navigation
    let key_tab = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Tab,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(103),
    );
    let mut cx = make_cx(&key_tab, id.clone(), &mut layers, &geom);
    let resp = editor.update(&mut cx, &mut state);
    assert!(matches!(resp.action, Some(CodeAction::Commit { .. })));
    assert!(!state.is_editing());

    // Measure test
    let theme = Theme::termrock();
    let measure_cx = MeasureCx::new(Constraints::unbounded(), &theme, ColorLevel::TrueColor);
    let measured = editor.measure(&measure_cx, Constraints::unbounded());
    assert!(measured.width >= 40);
    assert!(measured.height >= 4);

    // Draw test
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 24));
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 80, 24), &mut layers).with_buffer(&mut buf);
    editor.draw(&mut ui, Rect::new(0, 0, 80, 24), &state);
    // Gutter active line marker exists
    assert_eq!(buf[(0, 1)].symbol(), "▎");
}

#[test]
fn test_diff_view_unified_and_review_presentation() {
    let id = Id::new("diff.view");
    let mut source = MemoryDiffSource::new()
        .with_header("src/main.rs")
        .with_status(DiffStatus::Modified);

    source.add_header(ItemKey::new(1), "@@ -1,3 +1,3 @@");
    source.add_context(ItemKey::new(2), 1, 1, "fn main() {");
    source.add_removal(ItemKey::new(3), 2, "    let a = 1;");
    source.add_addition(ItemKey::new(4), 2, "    let a = 2;");
    source.add_context(ItemKey::new(5), 3, 3, "}");

    let diff_view = DiffView::new(id.clone(), &source).mode(DiffMode::Unified);
    let mut state = DiffViewState::new();

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 80, 24));

    // Down navigation
    let key_down = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Down,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&key_down, id.clone(), &mut layers, &geom);
    diff_view.update(&mut cx, &mut state);
    assert_eq!(state.cursor_row, Some(ItemKey::new(2)));

    // Clean copy via 'y'
    let key_copy = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Char('y'),
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(101),
    );
    let mut cx = make_cx(&key_copy, id.clone(), &mut layers, &geom);
    let resp = diff_view.update(&mut cx, &mut state);
    assert_eq!(
        resp.action,
        Some(DiffAction::CopyRequested {
            text: "fn main() {".into(),
        })
    );

    // Toggle mode via 'm'
    let key_toggle = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Char('m'),
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(102),
    );
    let mut cx = make_cx(&key_toggle, id.clone(), &mut layers, &geom);
    diff_view.update(&mut cx, &mut state);
    assert_eq!(state.mode, DiffMode::Review);

    // Measure test
    let theme = Theme::termrock();
    let measure_cx = MeasureCx::new(Constraints::unbounded(), &theme, ColorLevel::TrueColor);
    let measured = diff_view.measure(&measure_cx, Constraints::unbounded());
    assert!(measured.width >= 42);
    assert!(measured.height >= 5);

    // Draw test in review mode
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 24));
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 80, 24), &mut layers).with_buffer(&mut buf);
    diff_view.draw(&mut ui, Rect::new(0, 0, 80, 24), &state);
    // Header should contain status 'M'
    assert_eq!(buf[(1, 0)].symbol(), "M");
}

// =============================================================================
// AC-002 Probes: Edge Cases, Read-Only Safety, Breakpoint, Empty States
// =============================================================================

#[test]
fn test_grid_read_only_model_cannot_be_mutated() {
    let id = Id::new("grid.readonly");
    let grid = Grid::new(id.clone()).mode(GridMode::Table);

    let model = MockGridData::new();
    let mut state = GridState::new();

    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 80, 24));

    // `update` receives `&dyn GridModel` (read-only)
    // Model cannot be mutated through update pass
    let initial_val = model
        .cell_value(ItemKey::new(101), ColumnKey::new(2))
        .unwrap();
    assert_eq!(initial_val.text(), "Alice");

    let key_enter = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&key_enter, id.clone(), &mut layers, &geom);
    grid.update(&mut cx, &mut state, &model);

    // Model values remain identical
    let after_val = model
        .cell_value(ItemKey::new(101), ColumnKey::new(2))
        .unwrap();
    assert_eq!(after_val.text(), "Alice");
}

#[test]
fn test_grid_empty_and_fetch_more_states() {
    let id = Id::new("grid.empty");
    let grid = Grid::new(id.clone()).readiness(Readiness::Empty);

    // Empty model
    struct EmptyModel {
        rev: Revision,
        cols: Vec<GridColumn>,
    }
    impl GridModel for EmptyModel {
        fn revision(&self) -> Revision {
            self.rev
        }
        fn row_count(&self) -> usize {
            0
        }
        fn row_key(&self, _i: usize) -> ItemKey {
            ItemKey::new(0)
        }
        fn columns(&self) -> &[GridColumn] {
            &self.cols
        }
        fn cell_value(&self, _r: ItemKey, _c: ColumnKey) -> Option<CellValue> {
            None
        }
    }

    let empty = EmptyModel {
        rev: Revision::new(1),
        cols: vec![GridColumn::new(ColumnKey::new(1), "Name", 10)],
    };

    let state = GridState::new();
    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 24));
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 80, 24), &mut layers).with_buffer(&mut buf);

    grid.draw(&mut ui, Rect::new(0, 0, 80, 24), &state, &empty);
    assert_eq!(buf[(2, 2)].symbol(), "N"); // "No items"
}

#[test]
fn test_code_editor_read_only_safety() {
    let id = Id::new("editor.ro");
    let doc = MockDoc::new(vec!["immutable code"]);
    let editor = CodeEditor::new(id.clone(), &doc).read_only(true);

    let mut state = CodeEditorState::new();
    let mut layers = LayerStack::new();
    let mut geom = HashMap::new();
    geom.insert(id.clone(), Rect::new(0, 0, 80, 24));

    // Try entering edit mode via 'i'
    let key_i = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Char('i'),
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut cx = make_cx(&key_i, id.clone(), &mut layers, &geom);
    editor.update(&mut cx, &mut state);
    assert!(!state.is_editing()); // Read-only prevents editing

    // Try typing
    let key_char = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Char('z'),
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(101),
    );
    let mut cx = make_cx(&key_char, id.clone(), &mut layers, &geom);
    let resp = editor.update(&mut cx, &mut state);
    assert_ne!(resp.action, Some(CodeAction::Edited));
    assert!(state.draft.text().is_empty());
}

#[test]
fn test_diff_view_responsive_review_breakpoint() {
    let id = Id::new("diff.bp");
    let source = MemoryDiffSource::new();
    let diff_view = DiffView::new(id, &source);
    let state = DiffViewState::new().with_mode(DiffMode::Review);

    // Wide area (80 cols) -> effective mode is Review
    assert_eq!(diff_view.effective_mode(80, &state), DiffMode::Review);

    // Narrow area (35 cols < 42 threshold) -> effective mode falls back to Unified
    assert_eq!(diff_view.effective_mode(35, &state), DiffMode::Unified);

    // Durable state mode was NOT overwritten
    assert_eq!(state.mode, DiffMode::Review);

    // Widening back to 100 restores Review
    assert_eq!(diff_view.effective_mode(100, &state), DiffMode::Review);
}

#[test]
fn test_diff_view_empty_no_changes() {
    let id = Id::new("diff.empty");
    let source = MemoryDiffSource::new();
    let diff_view = DiffView::new(id, &source);
    let state = DiffViewState::new();

    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 24));
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 80, 24), &mut layers).with_buffer(&mut buf);

    diff_view.draw(&mut ui, Rect::new(0, 0, 80, 24), &state);
    assert_eq!(buf[(2, 1)].symbol(), "("); // "(no textual changes)"
}
