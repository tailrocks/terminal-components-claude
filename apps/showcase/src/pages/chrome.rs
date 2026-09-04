//! Application chrome: brand lockup, status strip and inline meters.

use tui_next::{Brand, Cx, Id, Response, Role, Status, StatusBar, StatusItem, Ui, id, layout};

use super::{Page, frame, lines};

const BRAND: Id = id!("chrome.brand");
const BAR: Id = id!("chrome.status");
const LEFT: [StatusItem<'static>; 2] = [
    StatusItem::new("SHOWCASE").strong(),
    StatusItem::new("workspace").tone(Role::Fg(tui_next::FgStep::Secondary)),
];
const CENTER: [StatusItem<'static>; 1] = [StatusItem::new("public API").chip()];
const RIGHT: [StatusItem<'static>; 2] = [
    StatusItem::new("120×40").priority(8),
    StatusItem::new("ready").tone(Role::Success),
];

/// Chrome keeps a clickable brand and a deterministic status strip in state.
#[derive(Debug, Default)]
pub(crate) struct ChromePage {
    brand_clicks: u32,
    frame: usize,
}

impl ChromePage {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl Page for ChromePage {
    fn title(&self) -> &'static str {
        "Chrome"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let brand = Brand::new(BRAND, "Junie")
            .tagline("application chrome")
            .clickable(true)
            .update(cx);
        if brand.activated() {
            self.brand_clicks = self.brand_clicks.saturating_add(1);
        }
        brand.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: tui_next::Rect) {
        frame(ui, area, self.title(), "brand · status · meter", |ui, body| {
            let (brand_area, rest) = layout::split_v(body, 4);
            Brand::new(BRAND, "Junie")
                .tagline("deliberate terminal interfaces")
                .clickable(true)
                .draw(ui, brand_area);
            let (status_area, notes) = layout::split_v(rest, 1);
            let center = [StatusItem::new("tests 24/24").meter(0.96), CENTER[0]];
            StatusBar::new(BAR)
                .left(&LEFT)
                .center(&center)
                .right(&RIGHT)
                .status(Status::Ready)
                .frame(self.frame)
                .draw(ui, status_area);
            let clicks = format!("brand activations: {} · status strip is keyed and width-aware", self.brand_clicks);
            let _ = ui.paint_str(notes, &clicks, ui.surface_style());
            lines(
                ui,
                tui_next::Rect {
                    y: notes.y.saturating_add(2),
                    height: notes.height.saturating_sub(2),
                    ..notes
                },
                &[
                    "Chrome owns no business data: it composes public primitives.",
                    "Narrow terminals drop lower-priority status items first.",
                ],
            );
        });
    }
}
