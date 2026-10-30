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
pub struct CatalogueDocument {
    pub id: Id,
    pub name: &'static str,
    pub summary: &'static str,
    pub contents: &'static str,
}

const DOCUMENTS: [CatalogueDocument; 2] = [
    CatalogueDocument {
        id: README,
        name: "README.md",
        summary: "Project overview · 18 lines",
        contents: "# Field Notes\n\nA small file catalogue example.\n",
    },
    CatalogueDocument {
        id: MANIFEST,
        name: "manifest.toml",
        summary: "Catalogue settings · 6 lines",
        contents: "[catalogue]\nname = \"field-notes\"\nfiles = 2\n",
    },
];

/// Public consumer example using borrowed document data and Termrock blocks.
pub struct FileCatalogue {
    selected: usize,
    documents: &'static [CatalogueDocument],
    layout: MasterDetailState,
    editor: CodeEditorState,
    policy: MasterDetailPolicy,
    last_layout_changed: bool,
}

impl Default for FileCatalogue {
    fn default() -> Self {
        Self {
            selected: 0,
            documents: &DOCUMENTS,
            layout: MasterDetailState::default(),
            editor: CodeEditorState::new(DOCUMENTS[0].contents),
            policy: MasterDetailPolicy::default()
                .detail_share(40, 12, 30)
                .gap(2)
                .collapse_below(60)
                .summary_rows(1),
            last_layout_changed: false,
        }
    }
}

impl FileCatalogue {
    pub fn with_policy(policy: MasterDetailPolicy) -> Self {
        Self {
            policy,
            last_layout_changed: false,
            ..Self::default()
        }
    }

    /// Build a catalogue with caller-owned borrowed documents and layout policy.
    pub fn with_documents(
        policy: MasterDetailPolicy,
        documents: &'static [CatalogueDocument],
    ) -> Self {
        Self {
            selected: 0,
            documents,
            layout: MasterDetailState::default(),
            editor: CodeEditorState::new(
                documents.first().map_or("", |document| document.contents),
            ),
            policy,
            last_layout_changed: false,
        }
    }

    pub fn selected_document(&self) -> &'static str {
        self.documents
            .get(self.selected)
            .map_or("", |document| document.name)
    }

    pub const fn drawer_open(&self) -> bool {
        self.layout.drawer_open()
    }

    pub fn split_percent(&self) -> Option<u8> {
        self.layout.split().map(|split| split.percent())
    }

    pub fn split_basis_points(&self) -> Option<u16> {
        self.layout.split().map(|split| split.basis_points())
    }

    pub const fn last_layout_changed(&self) -> bool {
        self.last_layout_changed
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
            &[Button<'static>],
            &CodeEditor<'static>,
            &Button<'static>,
        ) -> R,
    ) -> R {
        let buttons: Vec<_> = self
            .documents
            .iter()
            .map(|document| Button::new(document.id, document.name))
            .collect();
        let editor = CodeEditor::new(PREVIEW, 1).fill_height(true);
        let summary = Button::new(
            SUMMARY,
            self.documents
                .get(self.selected)
                .map_or("", |document| document.summary),
        );
        let master = Group::column(buttons.iter().map(composition::button));
        let detail = Group::column([composition::code_editor(&editor, &self.editor)]);
        let summary_group = Group::row([composition::button(&summary)]);
        let master_owner = self
            .documents
            .first()
            .map_or(README, |document| document.id);
        let block = MasterDetail::new(WORKSPACE, master_owner, PREVIEW)
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
        let master_owner = self
            .documents
            .first()
            .map_or(README, |document| document.id);
        let block = MasterDetail::new(WORKSPACE, master_owner, PREVIEW).policy(self.policy);
        let dismissed = block.update(cx, &mut self.layout);
        if dismissed.is_changed() {
            self.last_layout_changed = true;
        }
        if dismissed.is_consumed() {
            return dismissed.erase();
        }

        let buttons: Vec<_> = self
            .documents
            .iter()
            .map(|document| Button::new(document.id, document.name))
            .collect();
        let editor = CodeEditor::new(PREVIEW, 1).fill_height(true);
        let mut response = Response::ignored();
        for (index, button) in buttons.iter().enumerate() {
            let activated = button.update(cx);
            if activated.activated() {
                self.selected = index;
                self.editor.set_text(self.documents[index].contents);
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
    render_with_policy_once(width, height, MasterDetailPolicy::default());
}

/// Render the same live catalogue through an explicitly resizable master pane.
pub fn render_with_policy_once(width: u16, height: u16, policy: MasterDetailPolicy) {
    let area = Rect::new(0, 0, width, height);
    let mut runtime = Runtime::new(FileCatalogue::with_policy(policy), Theme::junie());
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
    let policy = MasterDetailPolicy::default()
        .resizable_master(45, 18, 22)
        .gap(2)
        .collapse_below(60)
        .summary_rows(1);
    render_with_policy_once(72, 18, policy);
}
