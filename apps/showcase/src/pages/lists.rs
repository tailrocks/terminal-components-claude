//! Keyed collection rows, single selection and multi-selection.

use tui_next::{
    Cx, Id, ItemKey, List, ListAction, ListState, Rect, Response, RowUi, SelectMode, Ui, id, layout,
};

use crate::data::LANGUAGES;

use super::{Page, frame, lines};

const SINGLE: Id = id!("lists.single");
const MULTI: Id = id!("lists.multi");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileRow {
    key: u8,
    label: &'static str,
    meta: &'static str,
    disabled: bool,
}

const FILES: &[FileRow] = &[
    FileRow {
        key: 1,
        label: "src/api/auth.rs",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 2,
        label: "src/api/billing.rs",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 3,
        label: "src/db/schema.rs",
        meta: "generated",
        disabled: true,
    },
    FileRow {
        key: 4,
        label: "tests/checkout.rs",
        meta: "new",
        disabled: false,
    },
    FileRow {
        key: 5,
        label: "Cargo.lock",
        meta: "locked",
        disabled: true,
    },
    FileRow {
        key: 6,
        label: "docs/webhooks.md",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 7,
        label: "src/workers/mailer.rs",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 8,
        label: "src/config.rs",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 9,
        label: "README.md",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 10,
        label: "src/main.rs",
        meta: "modified",
        disabled: false,
    },
    FileRow {
        key: 11,
        label: "tests/auth_flow.rs",
        meta: "new",
        disabled: false,
    },
    FileRow {
        key: 12,
        label: "src/db/pool.rs",
        meta: "modified",
        disabled: false,
    },
];

fn language_key(value: &&'static str) -> ItemKey {
    ItemKey::text(value)
}

fn language_row(value: &&'static str, row: &mut RowUi<'_>) {
    row.label(value);
}

fn file_key(value: &FileRow) -> ItemKey {
    ItemKey::num(u64::from(value.key))
}
fn file_row(value: &FileRow, row: &mut RowUi<'_>) {
    row.label(value.label);
    row.meta(value.meta);
}
fn file_disabled(value: &FileRow) -> bool {
    value.disabled
}

fn single_list() -> List<
    'static,
    &'static str,
    impl Fn(&&'static str) -> ItemKey,
    impl Fn(&&'static str, &mut RowUi<'_>),
> {
    List::new(SINGLE).key(language_key).row(language_row)
}

fn multi_list()
-> List<'static, FileRow, impl Fn(&FileRow) -> ItemKey, impl Fn(&FileRow, &mut RowUi<'_>)> {
    List::new(MULTI)
        .key(file_key)
        .row(file_row)
        .select_mode(SelectMode::Multi)
        .disabled_item(&file_disabled)
}

/// Two independent keyed list states; selecting a row never relies on its
/// position after reconciliation.
#[derive(Debug, Default)]
pub(crate) struct ListsPage {
    single: ListState,
    multi: ListState,
    chosen: Option<ItemKey>,
    last: &'static str,
}

impl ListsPage {
    pub(crate) fn new() -> Self {
        let mut multi = ListState::default();
        if let Some(file) = FILES.first() {
            multi.checked_mut().insert(file_key(file));
        }
        if let Some(file) = FILES.get(1) {
            multi.checked_mut().insert(file_key(file));
        }
        Self {
            single: ListState::default(),
            multi,
            chosen: None,
            last: "choose a language",
        }
    }
}

impl Page for ListsPage {
    fn title(&self) -> &'static str {
        "Lists"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        let one = single_list().update(cx, &mut self.single, LANGUAGES);
        if let Some(ListAction::Chose(key) | ListAction::Activated(key)) = one.action_ref() {
            self.chosen = Some(*key);
            self.last = "single selection committed";
        }
        response |= one.erase();
        let many = multi_list().update(cx, &mut self.multi, FILES);
        if matches!(many.action_ref(), Some(ListAction::ToggledAll)) {
            for file in FILES.iter().filter(|file| file.disabled) {
                self.multi.checked_mut().remove(file_key(file));
            }
        }
        if many.action_ref().is_some() {
            self.last = "multi selection changed";
        }
        response |= many.erase();
        response
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "keyed rows · arrows · Enter · Space · wheel",
            |ui, body| {
                let (left, right) = layout::split_h(body, body.width / 2);
                single_list().draw(ui, left, &self.single, LANGUAGES);
                multi_list().draw(ui, right, &self.multi, FILES);
                let chosen = self
                    .chosen
                    .and_then(|key| {
                        LANGUAGES
                            .iter()
                            .find(|value| language_key(value) == key)
                            .copied()
                    })
                    .unwrap_or("none");
                let summary = format!(
                    "Chosen: {chosen} · checked rows: {} · {}",
                    self.multi.checked().len_in(FILES.len()),
                    self.last
                );
                let footer = Rect {
                    y: body.bottom().saturating_sub(1),
                    height: 1,
                    ..body
                };
                let _ = ui.paint_str(footer, &summary, ui.surface_style());
                lines(
                    ui,
                    Rect {
                        y: footer.y.saturating_sub(2),
                        height: 1,
                        ..body
                    },
                    &["Two file rows are disabled to demonstrate non-activatable collection rows."],
                );
            },
        );
    }
}
