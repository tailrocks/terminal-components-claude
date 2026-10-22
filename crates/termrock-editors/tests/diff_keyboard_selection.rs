use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;

use termrock_editors::{DiffLineKind, DiffRow, DiffSource, DiffView, DiffViewState};
use termrock_runtime::event::{Input, Key, KeyCode, KeyModifiers};
use termrock_runtime::keymap::KeyMap;
use termrock_runtime::runtime::stub::{self, key};
use termrock_runtime::runtime::{App, Runtime};
use termrock_runtime::{Cx, Id, Response, Ui};
use termrock_theme::Theme;
use termrock_viewport::viewport::{CellPos, ViewportAction};

const DIFF: Id = Id::root("diff.keyboard-selection.integration");
const AREA: Rect = Rect::new(0, 0, 40, 8);
const PAYLOAD: &str = "payload";

struct Source;

impl DiffSource for Source {
    fn revision(&self) -> u64 {
        1
    }

    fn path(&self) -> &str {
        "src/example.rs"
    }

    fn status_marker(&self) -> &str {
        "M"
    }

    fn status_label(&self) -> &str {
        "modified"
    }

    fn row_count(&self) -> usize {
        1
    }

    fn row(&self, index: usize) -> Option<DiffRow<'_>> {
        (index == 0).then_some(DiffRow::Line {
            kind: DiffLineKind::Context,
            text: PAYLOAD,
        })
    }
}

static SOURCE: Source = Source;

struct DiffApp {
    state: DiffViewState,
    keymap: KeyMap,
    actions: Vec<ViewportAction>,
}

impl App for DiffApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = DiffView::new(DIFF, Some(&SOURCE)).update(cx, &mut self.state);
        if let Some(action) = response.action_ref() {
            self.actions.push(action.clone());
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        DiffView::new(DIFF, Some(&SOURCE)).draw(ui, AREA, &self.state);
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

#[test]
fn diff_view_forwards_keyboard_selection_and_copy_from_projected_source() {
    let mut runtime = Runtime::new(
        DiffApp {
            state: DiffViewState::default(),
            keymap: KeyMap::new(),
            actions: Vec::new(),
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();

    // Unified rows contain three-cell old/new gutters, two spaces, a marker,
    // and one separating space before the source text.
    runtime
        .app_mut()
        .state
        .viewport_mut()
        .select(CellPos::new(1, 10), CellPos::new(1, 10));
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();

    let response = stub::deliver(
        &mut runtime,
        Input::Key(Key {
            code: KeyCode::Right,
            mods: KeyModifiers::SHIFT,
        }),
    );

    assert!(response.is_consumed() && response.is_changed());
    assert_eq!(
        runtime.app().state.viewport().selection(),
        Some((CellPos::new(1, 10), CellPos::new(1, 11)))
    );
    assert_eq!(
        runtime.app().actions.last(),
        Some(&ViewportAction::SelectionChanged)
    );

    let _ = stub::deliver(&mut runtime, key(KeyCode::Char('y')));
    assert!(
        runtime
            .app()
            .actions
            .contains(&ViewportAction::Copy("p".to_owned()))
    );
}
