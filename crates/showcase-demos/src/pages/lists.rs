//! Keyed collection rows, single selection and multi-selection.

use termrock::{
    Cx, EmptyState, Family, FgStep, FrameRead, Id, ItemKey, List, ListAction, ListState, Panel,
    PanelKind, Part, Rect, Response, Role, RowUi, SelectMode, StateFlags, StylePatch, Track, Ui,
    Variant, id, layout,
};

use crate::render_number::RenderNumber;
use showcase_data::LANGUAGES;

use super::{Page, PageUpdate, frame};

fn list_columns(area: Rect) -> [Rect; 3] {
    let mut widths = [0; 3];
    layout::distribute_into(area.width, &[Track::Flex(1); 3], 2, &mut widths);
    let mut x = area.x;
    widths.map(|width| {
        let start = x.min(area.right());
        let width = width.min(area.right().saturating_sub(start));
        x = start.saturating_add(width).saturating_add(2);
        Rect {
            x: start,
            width,
            ..area
        }
    })
}

fn paint_chosen(ui: &mut Ui<'_>, inner: Rect, chosen: &str, style: termrock::author::PaintStyle) {
    let prefix = ui.paint_str(Rect { height: 1, ..inner }, "Chosen: ", style);
    ui.paint_str(
        Rect::new(
            inner.x.saturating_add(prefix),
            inner.y,
            inner.width.saturating_sub(prefix),
            1,
        ),
        chosen,
        style,
    );
}
const SINGLE: Id = id!("lists.single");
const MULTI: Id = id!("lists.multi");
const EMPTY: Id = id!("lists.empty");
const PANEL_PARTS: &[(Part, StylePatch)] = &[
    (
        Part::TITLE,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .remove(termrock::Modifier::BOLD),
    ),
    // Q67-S13 (L-R4): panel metas are faint (`tag:panel.rs:199`).
    // Instance-scoped: the shared (PANEL, DETAIL) recipe stays
    // Secondary (Q67-S10 STOPped the T4 recharter — O1 owns it).
    (
        Part::DETAIL,
        StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
    ),
];

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
/// Minimum label cells the tag keeps before hiding file metadata
/// (tag `list.rs:313`: "hide metadata rather than starve the label").
const MIN_FILE_LABEL: u16 = 12;

fn file_row(value: &FileRow, row: &mut RowUi<'_>) {
    // Tag recipe (tag `list.rs:308-322`): meta paints first, the label takes
    // what is left; metadata that would starve the label below
    // `MIN_FILE_LABEL` stays hidden.
    let keep = row
        .remaining_width()
        .saturating_sub(termrock::width(value.meta).saturating_add(1));
    if keep >= MIN_FILE_LABEL {
        row.meta(value.meta);
    }
    row.label(value.label);
}
fn file_disabled(value: &FileRow) -> bool {
    value.disabled
}

/// Scroll-position meta for a list panel, mirroring the trees page
/// helper: empty unless the content overflows the viewport, otherwise
/// `1–13 of 20` (`tag:scrollbar.rs:75-81`). The item count is passed
/// explicitly — the list scroll extent carries layout padding, not the
/// item count — and `viewport_h` covers the first frame, before layout
/// publishes the viewport length.
fn position_label(st: &ListState, viewport_h: usize, len: usize) -> String {
    let scroll = st.scroll();
    let viewport = if scroll.viewport_len() > 0 {
        scroll.viewport_len()
    } else {
        viewport_h
    };
    if len <= viewport || viewport == 0 {
        return String::new();
    }
    let offset = scroll.offset().min(len.saturating_sub(1));
    let start = offset.saturating_add(1);
    let end = offset.saturating_add(viewport).min(len);
    format!("{start}–{end} of {len}")
}

fn single_list() -> List<
    'static,
    &'static str,
    impl Fn(&&'static str) -> ItemKey,
    impl Fn(&&'static str, &mut RowUi<'_>),
> {
    List::new(SINGLE).key(language_key).row(language_row)
}

/// The one multi-selection list constructor (§13): keyed, patched,
/// disabled-aware.
fn multi_list_with(
    row: impl Fn(&FileRow, &mut RowUi<'_>),
) -> List<'static, FileRow, impl Fn(&FileRow) -> ItemKey, impl Fn(&FileRow, &mut RowUi<'_>)> {
    List::new(MULTI)
        .key(file_key)
        .row(row)
        .select_mode(SelectMode::Multi)
        .disabled_item(&file_disabled)
}

fn multi_list()
-> List<'static, FileRow, impl Fn(&FileRow) -> ItemKey, impl Fn(&FileRow, &mut RowUi<'_>)> {
    multi_list_with(file_row)
}

/// The one empty-list constructor (§13): the update path drives it and the
/// draw path renders it, so the empty state cannot drift between phases.
fn empty_list(title: &'static str) -> List<'static, &'static str> {
    List::new(EMPTY).empty(EmptyState::Empty { title, hint: None })
}

/// Two independent keyed list states; selecting a row never relies on its
/// position after reconciliation.
#[derive(Debug, Default)]
pub struct ListsPage {
    single: ListState,
    multi: ListState,
    empty: ListState,
    chosen: Option<ItemKey>,
    last: &'static str,
}

impl ListsPage {
    pub fn new() -> Self {
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
        let empty = empty_list("No matches").update(cx, &mut self.empty, &[] as &[&str]);
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
                StateFlags::empty(),
            )
            .style;
        frame(
            ui,
            area,
            self.title(),
            "Single and multiple selection, disabled items, scrolling, empty state",
            |ui, body| {
                let columns = list_columns(body);
                let height = body.height.min(18);
                let language_column = columns.first().copied().unwrap_or(body);
                let language = Rect {
                    height,
                    ..language_column
                };
                let language_panel = Panel::new(id!("lists.language"))
                    .kind(PanelKind::Card)
                    .title("Language")
                    .patch_part(PANEL_PARTS);
                let language_inner = language_panel.inner(ui, language);
                // Q67-S13 (L-R4): the tag draws the scroll position as the
                // Language panel meta (`tag:lists.rs`); the list sits two
                // rows below the panel inner top (`Chosen:` + gap).
                let language_meta = position_label(
                    &self.single,
                    language_inner.height.saturating_sub(2) as usize,
                    LANGUAGES.len(),
                );
                language_panel
                    .meta(&language_meta)
                    .draw(ui, language, |ui, inner| {
                        paint_chosen(ui, inner, chosen, detail);
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
                    height,
                    ..search_column
                };
                Panel::new(id!("lists.search"))
                    .kind(PanelKind::Card)
                    .title("Search results")
                    .patch_part(PANEL_PARTS)
                    .draw(ui, search, |ui, inner| {
                        empty_list(if body.width < 70 {
                            "No results for…"
                        } else if body.width < 90 {
                            "No results for “retr…"
                        } else {
                            "No results for “retry”"
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

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(MULTI).contains(StateFlags::FOCUSED) {
            &[
                ("↑ ↓", "Move"),
                ("Space", "Toggle"),
                ("a", "All / none"),
                ("Shift+↓", "Range"),
            ]
        } else {
            &[("↑ ↓", "Move"), ("Enter", "Choose"), ("g G", "Ends")]
        }
    }
}

impl ListsPage {
    fn draw_files(
        &self,
        ui: &mut Ui<'_>,
        body: Rect,
        columns: &[Rect],
        height: u16,
        detail: termrock::author::PaintStyle,
    ) {
        let files_column = columns.get(1).copied().unwrap_or(body);
        let files = Rect {
            height,
            ..files_column
        };
        let selected = RenderNumber::selected(self.multi.checked().len_in(FILES.len()));
        let mut files_panel =
            Panel::new(id!("lists.files"))
                .kind(PanelKind::Card)
                .title(if body.width < 70 {
                    "Files to incl…"
                } else {
                    "Files to include"
                });
        // Q67-S13 (L-R4): the tag sets the count meta unconditionally
        // (`tag:lists.rs`); the Panel head already truncates title and
        // meta to fit, so no width gate is needed.
        files_panel = files_panel.meta(selected.as_str());
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
                if self.last == "multi selection changed" {
                    let row = Rect {
                        y: inner.bottom().saturating_sub(1),
                        height: 1,
                        ..inner
                    };
                    let count = RenderNumber::new(self.multi.checked().len_in(FILES.len()));
                    let prefix = ui.paint_str(row, "checked rows: ", detail);
                    ui.paint_str(
                        Rect::new(
                            row.x.saturating_add(prefix),
                            row.y,
                            row.width.saturating_sub(prefix),
                            1,
                        ),
                        count.as_str(),
                        detail,
                    );
                }
            });
    }
}
