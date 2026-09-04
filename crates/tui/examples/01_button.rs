//! `COMPONENT_ARCHITECTURE.md` §17 example 1, verbatim (crate name is temporary: `junie_tui` → `junie_tui` at Slice 5).
#![expect(
    clippy::arithmetic_side_effects,
    reason = "verbatim from §17 example 1"
)]

use junie_tui::{App, Button, Cx, Id, Insets, Response, Theme, Ui, id, layout, run};

const SAVE: Id = id!("save");

#[derive(Default)]
struct Demo {
    saves: u32,
}

impl App for Demo {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Button::new(SAVE, "Save")
            .update(cx)
            .on_activated(|| self.saves += 1)
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let area = layout::inset(
            ui.full(),
            Insets {
                l: 2,
                t: 1,
                r: 2,
                b: 1,
            },
        );
        Button::new(SAVE, "Save").draw(ui, area);
    }
}

fn main() -> std::io::Result<()> {
    run(Demo::default(), Theme::junie())
}
