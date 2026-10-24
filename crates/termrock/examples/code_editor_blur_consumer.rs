//! Default-feature facade consumer for editor focus-out behavior.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use termrock::composition::{self, Group};
use termrock::editors::{CodeEditor, CodeEditorState};
use termrock::layout::Composition;
use termrock::{App, Cx, Id, Response, Runtime, Theme, Ui};

const SQL_EDITOR: Id = Id::root("consumer.sql-editor");
const SCRATCH_EDITOR: Id = Id::root("consumer.scratch-editor");

struct Documents {
    sql_editor: CodeEditor<'static>,
    scratch_editor: CodeEditor<'static>,
    sql_state: CodeEditorState,
    scratch_state: CodeEditorState,
}

impl Documents {
    fn new() -> Self {
        Self {
            sql_editor: CodeEditor::new(SQL_EDITOR, 3).fill_height(true),
            scratch_editor: CodeEditor::new(SCRATCH_EDITOR, 3)
                .fill_height(true)
                .commit_on_blur(false),
            sql_state: CodeEditorState::new("SELECT city, population FROM census"),
            scratch_state: CodeEditorState::new("東部集計 -- draft e\u{301}"),
        }
    }
}

impl App for Documents {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == termrock::UpdateCause::Bootstrap {
            cx.focus(SQL_EDITOR);
        }
        self.sql_editor.update(cx, &mut self.sql_state).erase()
            | self
                .scratch_editor
                .update(cx, &mut self.scratch_state)
                .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Group::column([
            composition::code_editor(&self.sql_editor, &self.sql_state).flex(1),
            composition::code_editor(&self.scratch_editor, &self.scratch_state).flex(1),
        ])
        .spec(
            Composition::new()
                .gap(1)
                .padding(termrock::layout::Insets::all(1)),
        )
        .show(ui)
        .expect("two document editors fit the supported composition depth");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let area = termrock::Rect::new(0, 0, 72, 12);
    let mut runtime = Runtime::new(Documents::new(), Theme::junie());
    let prepared = runtime.prepare(area)?;
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height))?;
    terminal.draw(|frame| {
        runtime
            .draw_prepared(prepared, frame)
            .expect("prepared area matches the framework terminal");
    })?;
    runtime.acknowledge_presented(prepared)?;
    Ok(())
}
