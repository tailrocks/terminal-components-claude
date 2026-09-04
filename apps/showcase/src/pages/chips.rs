//! Chip toggles and a keyed select field.

use junie_tui::{
    ChipBar, ChipBarAction, ChipBarState, Cx, Id, ItemKey, Rect, Response, RowUi, Select,
    SelectAction, SelectState, Ui, id, layout,
};

use crate::data::LANGUAGES;

use super::{Page, frame, lines};

const CHIPS: Id = id!("chips.filters");
const SELECT: Id = id!("chips.language");
const FILTERS: &[&str] = &[
    "Open",
    "Assigned",
    "Needs review",
    "Blocked",
    "Mine",
    "Recent",
];

fn chip_key(value: &&'static str) -> ItemKey {
    ItemKey::text(value)
}

fn chip_row(value: &&'static str, row: &mut RowUi<'_>) {
    row.label(value);
}

fn chips() -> ChipBar<
    'static,
    &'static str,
    impl Fn(&&'static str) -> ItemKey,
    impl Fn(&&'static str, &mut RowUi<'_>),
> {
    ChipBar::new(CHIPS)
        .key(chip_key)
        .row(chip_row)
        .select_mode(junie_tui::SelectMode::Multi)
        .closable(false)
}

fn select() -> Select<'static, &'static str> {
    Select::new(SELECT).placeholder("Choose language")
}

/// Filter chips and the language selector demonstrate two keyed collection
/// controls with independent cursor/value state.
#[derive(Debug, Default)]
pub(crate) struct ChipsPage {
    chip_state: ChipBarState,
    select_state: SelectState,
    last: &'static str,
}

impl ChipsPage {
    pub(crate) fn new() -> Self {
        Self {
            chip_state: ChipBarState::default(),
            select_state: SelectState::default(),
            last: "no filter selected",
        }
    }
}

impl Page for ChipsPage {
    fn title(&self) -> &'static str {
        "Chips & selects"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        let chips = chips().update(cx, &mut self.chip_state, FILTERS);
        if let Some(action) = chips.action_ref() {
            self.last = match action {
                ChipBarAction::Toggled(_) => "filter toggled",
                ChipBarAction::Activated(_) => "filter activated",
                ChipBarAction::Closed(_) => "filter closed",
                ChipBarAction::AddRequested => "filter add requested",
            };
        }
        result |= chips.erase();
        let select = select().update(cx, &mut self.select_state, LANGUAGES);
        if select
            .action_ref()
            .is_some_and(|action| matches!(action, SelectAction::Chose(_)))
        {
            self.last = "language selected";
        }
        result |= select.erase();
        result
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "stable keys · Space toggles chips · Enter opens select",
            |ui, body| {
                let (chip_area, rest) = layout::split_v(body, 4);
                chips().draw(ui, chip_area, &self.chip_state, FILTERS);
                let (select_area, note) = layout::split_v(rest, 3);
                select().draw(ui, select_area, &self.select_state, LANGUAGES);
                let language = self
                    .select_state
                    .value()
                    .and_then(|key| match key {
                        ItemKey::Index(index) => LANGUAGES.get(index).copied(),
                        _ => None,
                    })
                    .unwrap_or("none");
                let summary = format!(
                    "filters checked: {} · language: {language} · {}",
                    self.chip_state.checked().len_in(FILTERS.len()),
                    self.last
                );
                let _ = ui.paint_str(note, &summary, ui.surface_style());
                lines(
                    ui,
                    Rect {
                        y: note.y.saturating_add(1),
                        height: 1,
                        ..note
                    },
                    &["Chips retain checked identity when the source order changes."],
                );
            },
        );
    }
}
