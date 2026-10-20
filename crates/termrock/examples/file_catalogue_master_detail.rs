use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::layout::Rect;
use termrock::blocks::{MasterDetail, MasterDetailPolicy, MasterDetailState};
use termrock::composition::{self, Group};
use termrock::controls::Button;
use termrock::editors::{CodeEditor, CodeEditorState};
use termrock::layout::Constraints;
use termrock::{App, Cx, Id, Response, Runtime, Theme, Ui};

pub const WORKSPACE: Id = Id::root("file-catalogue.workspace");
pub const README: Id = Id::root("file-catalogue.file.readme");
pub const MANIFEST: Id = Id::root("file-catalogue.file.manifest");
pub const PREVIEW: Id = Id::root("file-catalogue.preview");
pub const SUMMARY: Id = Id::root("file-catalogue.summary");

#[derive(Clone, Copy)]
struct Document {
    id: Id,
    name: &'static str,
    summary: &'static str,
    contents: &'static str,
}

const DOCUMENTS: [Document; 2] = [
    Document {
        id: README,
        name: "README.md",
        summary: "Project overview · 18 lines",
        contents: "# Field Notes\n\nA small file catalogue example.\n",
    },
    Document {
        id: MANIFEST,
        name: "manifest.toml",
        summary: "Catalogue settings · 6 lines",
        contents: "[catalogue]\nname = \"field-notes\"\nfiles = 2\n",
    },
];

/// Public consumer example using borrowed document data and Termrock blocks.
pub struct FileCatalogue {
    selected: usize,
    layout: MasterDetailState,
    editor: CodeEditorState,
    policy: MasterDetailPolicy,
}

impl Default for FileCatalogue {
    fn default() -> Self {
        Self {
            selected: 0,
            layout: MasterDetailState::default(),
            editor: CodeEditorState::new(DOCUMENTS[0].contents),
            policy: MasterDetailPolicy::default()
                .detail_share(40, 12, 30)
                .gap(2)
                .collapse_below(60)
                .summary_rows(1),
        }
    }
}

impl FileCatalogue {
    pub fn with_policy(policy: MasterDetailPolicy) -> Self {
        Self {
            policy,
            ..Self::default()
        }
    }

    pub fn selected_document(&self) -> &'static str {
        DOCUMENTS[self.selected].name
    }

    pub const fn drawer_open(&self) -> bool {
        self.layout.drawer_open()
    }

    pub fn preview_text(&self) -> &str {
        self.editor.text()
    }

    fn with_block<R>(
        &self,
        f: impl FnOnce(
            &MasterDetail<'_>,
            &Group<'_>,
            &Group<'_>,
            &Group<'_>,
            &[Button<'static>; 2],
            &CodeEditor<'static>,
            &Button<'static>,
        ) -> R,
    ) -> R {
        let buttons = DOCUMENTS.map(|document| Button::new(document.id, document.name));
        let editor = CodeEditor::new(PREVIEW, 1).fill_height(true);
        let summary = Button::new(SUMMARY, DOCUMENTS[self.selected].summary);
        let master = Group::column(buttons.iter().map(composition::button));
        let detail = Group::column([composition::code_editor(&editor, &self.editor)]);
        let summary_group = Group::row([composition::button(&summary)]);
        let block = MasterDetail::new(WORKSPACE, README, PREVIEW)
            .policy(self.policy)
            .master(&master)
            .detail(&detail)
            .summary(&summary_group);
        f(
            &block,
            &master,
            &detail,
            &summary_group,
            &buttons,
            &editor,
            &summary,
        )
    }
}

impl App for FileCatalogue {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        // The block gets first refusal on Escape. For every other input its
        // iterator is observational, so the focused editor still receives it.
        let block = MasterDetail::new(WORKSPACE, README, PREVIEW).policy(self.policy);
        let dismissed = block.update(cx, &mut self.layout);
        if dismissed.is_consumed() {
            return dismissed.erase();
        }

        let buttons = DOCUMENTS.map(|document| Button::new(document.id, document.name));
        let editor = CodeEditor::new(PREVIEW, 1).fill_height(true);
        let mut response = Response::ignored();
        for (index, button) in buttons.iter().enumerate() {
            let activated = button.update(cx);
            if activated.activated() {
                self.selected = index;
                self.editor.set_text(DOCUMENTS[index].contents);
                if self.policy.is_collapsed(cx.viewport().width) {
                    response |= block.open_detail(cx, &mut self.layout).erase();
                } else {
                    cx.focus(PREVIEW);
                }
            }
            response |= activated.erase();
        }
        response |= editor.update(cx, &mut self.editor).erase();
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.with_block(|block, _, _, _, _, _, _| {
            let constraints = Constraints::loose(ui.full().width, ui.full().height);
            let _ = block
                .measure(ui, &self.layout, constraints)
                .expect("catalogue composition has valid public group measurements");
            block
                .show(ui, &self.layout)
                .expect("catalogue composition fits the shared block depth");
        });
    }
}

/// Render one public example frame; root geometry enters only at the runtime boundary.
pub fn render_once(width: u16, height: u16) {
    let area = Rect::new(0, 0, width, height);
    let mut runtime = Runtime::new(FileCatalogue::default(), Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test terminal");
    let prepared = runtime.prepare(area).expect("prepare frame");
    terminal
        .draw(|frame| {
            runtime.draw_prepared(prepared, frame).expect("draw frame");
        })
        .expect("present frame");
    runtime
        .acknowledge_presented(prepared)
        .expect("acknowledge frame");
}

#[cfg(not(test))]
fn main() {
    render_once(72, 18);
}
