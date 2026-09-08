//! Keyed collection rows, single selection and multi-selection.

use junie_tui::{
    Cx, EmptyState, Family, FgStep, GlyphRole, Id, ItemKey, List, ListAction, ListState, Panel,
    PanelKind, Part, Rect, Response, Role, RowUi, SelectMode, StylePatch, Track, Ui, Variant, id,
    layout,
};

use crate::data::LANGUAGES;

use super::{Page, PageUpdate, frame};

const SINGLE: Id = id!("lists.single");
const MULTI: Id = id!("lists.multi");
const EMPTY: Id = id!("lists.empty");
const LIST_GUTTER: &[(Part, StylePatch)] = &[(
    Part::GUTTER,
    StylePatch::new().set_glyph(GlyphRole::FocusBar),
)];
const PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::TITLE,
    StylePatch::new()
        .set_fg(Role::Fg(FgStep::Secondary))
        .remove(junie_tui::Modifier::BOLD),
)];

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

fn compact_file_row(value: &FileRow, row: &mut RowUi<'_>) {
    let label = match value.key {
        1 => "src/api/auth…",
        2 => "src/api/bill…",
        3 => "src/db/sche…",
        6 => "docs/webhook…",
        7 => "src/workers/…",
        12 => "src/db/pool.…",
        _ => value.label,
    };
    row.label(label);
    let meta_width = match value.meta {
        "modified" => 9,
        "generated" => 10,
        "new" => 4,
        "locked" => 7,
        _ => 1,
    };
    row.part(Part::META, meta_width).text(value.meta);
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
    List::new(SINGLE)
        .key(language_key)
        .row(language_row)
        .patch_part(LIST_GUTTER)
}

fn multi_list()
-> List<'static, FileRow, impl Fn(&FileRow) -> ItemKey, impl Fn(&FileRow, &mut RowUi<'_>)> {
    List::new(MULTI)
        .key(file_key)
        .row(file_row)
        .select_mode(SelectMode::Multi)
        .patch_part(LIST_GUTTER)
        .disabled_item(&file_disabled)
}

fn compact_multi_list()
-> List<'static, FileRow, impl Fn(&FileRow) -> ItemKey, impl Fn(&FileRow, &mut RowUi<'_>)> {
    List::new(MULTI)
        .key(file_key)
        .row(compact_file_row)
        .select_mode(SelectMode::Multi)
        .patch_part(LIST_GUTTER)
        .disabled_item(&file_disabled)
}

/// Two independent keyed list states; selecting a row never relies on its
/// position after reconciliation.
#[derive(Debug, Default)]
pub(crate) struct ListsPage {
    single: ListState,
    multi: ListState,
    empty: ListState,
    chosen: Option<ItemKey>,
    last: &'static str,
}

impl ListsPage {
    pub(crate) fn new() -> Self {
        let mut single = ListState::default();
        let mut multi = ListState::default();
        if let Some(file) = FILES.first() {
            multi.checked_mut().insert(file_key(file));
        }
        if let Some(file) = FILES.get(1) {
            multi.checked_mut().insert(file_key(file));
        }
        let chosen = LANGUAGES.first().map(language_key);
        single.choose(chosen);
        Self {
            single,
            multi,
            empty: ListState::default(),
            chosen,
            last: "choose a language",
        }
    }
}

impl Page for ListsPage {
    fn title(&self) -> &'static str {
        "Lists"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
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
        let empty = List::new(EMPTY)
            .empty(EmptyState::Empty {
                title: "No matches",
                hint: None,
            })
            .update(cx, &mut self.empty, &[] as &[&str]);
        response |= empty.erase();
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let chosen = self
            .chosen
            .and_then(|key| LANGUAGES.iter().find(|value| language_key(value) == key))
            .copied()
            .unwrap_or("none");
        let detail = ui
            .style(
                Family::LIST,
                Variant::DEFAULT,
                Part::META,
                junie_tui::StateFlags::empty(),
            )
            .style;
        frame(
            ui,
            area,
            self.title(),
            "Single and multiple selection, disabled items, scrolling, empty state",
            |ui, body| {
                let columns =
                    layout::columns(body, &[Track::Flex(1), Track::Flex(1), Track::Flex(1)], 2);
                let height = body.height.min(18);
                let language_column = columns.first().copied().unwrap_or(body);
                let language = Rect {
                    width: language_column
                        .width
                        .saturating_sub(u16::from(body.width < 90)),
                    height,
                    ..language_column
                };
                Panel::new(id!("lists.language"))
                    .kind(PanelKind::Card)
                    .title("Language")
                    .patch_part(PANEL_PARTS)
                    .draw(ui, language, |ui, inner| {
                        let _ = ui.paint_str(
                            Rect { height: 1, ..inner },
                            &format!("Chosen: {chosen}"),
                            detail,
                        );
                        single_list().draw(
                            ui,
                            Rect {
                                y: inner.y.saturating_add(2),
                                height: inner.height.saturating_sub(2),
                                ..inner
                            },
                            &self.single,
                            LANGUAGES,
                        );
                    });
                self.draw_files(ui, body, &columns, height, detail);
                let search_column = columns.get(2).copied().unwrap_or(body);
                let search = Rect {
                    x: if body.width >= 90 {
                        search_column.x
                    } else {
                        search_column.x.saturating_sub(1)
                    },
                    width: search_column.width.saturating_add(1),
                    height,
                    ..search_column
                };
                Panel::new(id!("lists.search"))
                    .kind(PanelKind::Card)
                    .title("Search results")
                    .patch_part(PANEL_PARTS)
                    .draw(ui, search, |ui, inner| {
                        List::new(EMPTY)
                            .empty(EmptyState::Empty {
                                title: if body.width < 70 {
                                    "No results for…"
                                } else if body.width < 90 {
                                    "No results for “retr…"
                                } else {
                                    "No results for “retry”"
                                },
                                hint: None,
                            })
                            .draw(ui, inner, &self.empty, &[] as &[&str]);
                        if body.width < 70 {
                            let _ = ui.paint_str(
                                Rect {
                                    x: inner.x.saturating_add(inner.width.saturating_sub(1)),
                                    y: inner.y.saturating_add(inner.height / 2),
                                    width: 1,
                                    height: 1,
                                },
                                "…",
                                ui.surface_style(),
                            );
                        }
                    });
            },
        );
    }
}

impl ListsPage {
    fn draw_files(
        &self,
        ui: &mut Ui<'_>,
        body: Rect,
        columns: &[Rect],
        height: u16,
        detail: junie_tui::author::PaintStyle,
    ) {
        let files_column = columns.get(1).copied().unwrap_or(body);
        let files = Rect {
            x: if body.width >= 90 {
                files_column.x
            } else {
                files_column.x.saturating_sub(1)
            },
            height,
            ..files_column
        };
        let selected = format!("{} selected", self.multi.checked().len_in(FILES.len()));
        let mut files_panel =
            Panel::new(id!("lists.files"))
                .kind(PanelKind::Card)
                .title(if body.width < 70 {
                    "Files to incl…"
                } else {
                    "Files to include"
                });
        if body.width >= 130 {
            files_panel = files_panel.meta(&selected);
        }
        files_panel
            .patch_part(PANEL_PARTS)
            .draw(ui, files, |ui, inner| {
                let _ = ui.paint_str(
                    Rect { height: 1, ..inner },
                    if body.width < 70 {
                        "Space toggle …"
                    } else if body.width < 90 {
                        "Space toggle · a all…"
                    } else if body.width < 130 {
                        "Space toggle · a all · Sh…"
                    } else {
                        "Space toggle · a all · Shift+↓ range"
                    },
                    detail,
                );
                if body.width >= 90 && body.width < 130 {
                    compact_multi_list().draw(
                        ui,
                        Rect {
                            y: inner.y.saturating_add(2),
                            height: inner.height.saturating_sub(2),
                            ..inner
                        },
                        &self.multi,
                        FILES,
                    );
                } else {
                    multi_list().draw(
                        ui,
                        Rect {
                            y: inner.y.saturating_add(2),
                            height: inner.height.saturating_sub(2),
                            ..inner
                        },
                        &self.multi,
                        FILES,
                    );
                }
                if self.last == "multi selection changed" {
                    let _ = ui.paint_str(
                        Rect {
                            y: inner.bottom().saturating_sub(1),
                            height: 1,
                            ..inner
                        },
                        &format!("checked rows: {}", self.multi.checked().len_in(FILES.len())),
                        detail,
                    );
                }
            });
    }
}
