//! Keyed collection rows, single selection and multi-selection.

use tui_next::{Cx, Id, ItemKey, List, ListAction, ListState, Rect, Response, RowUi, SelectMode, Ui, id, layout};

use crate::data::LANGUAGES;

use super::{Page, frame, lines};

const SINGLE: Id = id!("lists.single");
const MULTI: Id = id!("lists.multi");

fn language_key(value: &&'static str) -> ItemKey {
    ItemKey::text(value)
}

fn language_row(value: &&'static str, row: &mut RowUi<'_>) {
    row.label(value);
}

fn unavailable(value: &&'static str) -> bool {
    *value == "C#"
}

fn single_list() -> List<
    'static,
    &'static str,
    impl Fn(&&'static str) -> ItemKey,
    impl Fn(&&'static str, &mut RowUi<'_>),
> {
    List::new(SINGLE).key(language_key).row(language_row)
}

fn multi_list() -> List<
    'static,
    &'static str,
    impl Fn(&&'static str) -> ItemKey,
    impl Fn(&&'static str, &mut RowUi<'_>),
> {
    List::new(MULTI)
        .key(language_key)
        .row(language_row)
        .select_mode(SelectMode::Multi)
        .disabled_item(&unavailable)
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
        Self {
            single: ListState::default(),
            multi: ListState::default(),
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
        let many = multi_list().update(cx, &mut self.multi, LANGUAGES);
        if many.action_ref().is_some() {
            self.last = "multi selection changed";
        }
        response |= many.erase();
        response
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "keyed rows · arrows · Enter · Space · wheel", |ui, body| {
            let (left, right) = layout::split_h(body, body.width / 2);
            single_list().draw(ui, left, &self.single, LANGUAGES);
            multi_list().draw(ui, right, &self.multi, LANGUAGES);
            let chosen = self
                .chosen
                .and_then(|key| LANGUAGES.iter().find(|value| language_key(value) == key).copied())
                .unwrap_or("none");
            let summary = format!(
                "Chosen: {chosen} · checked rows: {} · {}",
                self.multi.checked().len_in(LANGUAGES.len()),
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
                &["C# is disabled to demonstrate non-activatable collection rows."],
            );
        });
    }
}
