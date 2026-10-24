//! Environment cell editor. Tab from the shell, Tab onto the field, Enter
//! opens the shared [`termrock::Select`] that lists Disaster recovery.

use termrock::{Button, Cx, Id, Rect, Response, Select, SelectState, Ui};

use super::{Page, PageUpdate, frame};

const ROW: Id = termrock::Id::root("showcase_ui::pages::celleditors::row");
const ENV: Id = termrock::Id::root("showcase_ui::pages::celleditors::env");

const ENVIRONMENTS: &[&str] = &[
    "Development",
    "Staging",
    "Production",
    "Sandbox",
    "Preview",
    "Performance",
    "Disaster recovery",
];

pub struct CellEditorsPage {
    env: SelectState,
}

impl CellEditorsPage {
    pub fn new() -> Self {
        Self {
            env: SelectState::default(),
        }
    }

    fn row() -> Button<'static> {
        Button::new(ROW, "checkout-api")
    }

    fn env_select() -> Select<'static, &'static str> {
        Select::new(ENV).popup_rows(8)
    }
}

impl Default for CellEditorsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for CellEditorsPage {
    fn title(&self) -> &'static str {
        "Cell editors"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        response |= Self::row().update(cx).erase();
        response |= Self::env_select()
            .update(cx, &mut self.env, ENVIRONMENTS)
            .erase();
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "Env select", |ui, body| {
            if body.height < 2 {
                return;
            }
            Self::row().draw(ui, Rect { height: 1, ..body });
            Self::env_select().draw(
                ui,
                Rect {
                    y: body.y.saturating_add(1),
                    height: 1,
                    ..body
                },
                &self.env,
                ENVIRONMENTS,
            );
        });
    }
}
