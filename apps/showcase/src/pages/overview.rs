//! Overview screen: the public-facade contract and its stable sample data.

use junie_tui::{Brand, Id, Props, Rect, Response, Ui, id, layout};

use super::{Page, author::AuthorBadge, frame, lines};

const BRAND: Id = id!("overview.brand");
const AUTHOR: Id = id!("overview.author");
const PROPS: [(&str, &str); 6] = [
    ("Library", "junie-tui"),
    ("Ownership", "application state"),
    ("Input", "runtime intents"),
    ("Rendering", "public Ui facade"),
    ("Binary", "showcase"),
    ("Tokens", "surface · accent · focus"),
];

/// The landing page has no mutable controls; its content is deliberately
/// useful as a smoke test for themes, clipping and public component exports.
#[derive(Debug)]
pub(crate) struct OverviewPage {
    author: AuthorBadge,
}

impl OverviewPage {
    pub(crate) fn new() -> Self {
        Self {
            author: AuthorBadge::new(AUTHOR),
        }
    }
}

impl Default for OverviewPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for OverviewPage {
    fn title(&self) -> &'static str {
        "Overview"
    }

    fn update(&mut self, cx: &mut junie_tui::Cx<'_>) -> Response<()> {
        self.author.update(cx)
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "public junie-tui API",
            |ui, body| {
                let (intro, rest) = layout::split_v(body, 4);
                Brand::new(BRAND, "Junie")
                    .tagline("component showcase")
                    .draw(ui, intro);
                let (copy, props) = layout::split_h(rest, rest.width / 2);
                let (copy_text, author_area) = layout::split_v(copy, copy.height.saturating_sub(2));
                lines(
                    ui,
                    copy_text,
                    &[
                        "A complete app-owned migration of the legacy showcase.",
                        "Each page owns durable state and talks to junie-tui through",
                        "the same public facade available to downstream binaries.",
                        "Tab focuses controls · Enter activates · Esc returns home.",
                    ],
                );
                self.author.draw(ui, author_area);
                Props::new(&PROPS).draw(ui, props);
            },
        );
    }
}
