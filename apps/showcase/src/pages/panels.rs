//! Card and framed panel composition, including caller-owned overrides.

use junie_tui::{Modifier, Panel, PanelKind, Part, Rect, Response, Role, StylePatch, Ui, id};

use super::{Page, frame, lines};

const OUTER: junie_tui::Id = id!("panels.outer");
const INNER: junie_tui::Id = id!("panels.inner");
const PATCH: StylePatch = StylePatch::new().set_fg(Role::Accent).add(Modifier::BOLD);
const PARTS: &[(Part, StylePatch)] = &[(Part::TITLE, PATCH)];

/// Static panel surfaces are still useful to test the live theme and patch
/// precedence without introducing application-owned rendering code.
#[derive(Debug, Default)]
pub(crate) struct PanelsPage;

impl PanelsPage {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Page for PanelsPage {
    fn title(&self) -> &'static str {
        "Panels"
    }

    fn update(&mut self, _cx: &mut junie_tui::Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "card · framed · per-instance patch",
            |ui, body| {
                Panel::new(OUTER)
                    .kind(PanelKind::Card)
                    .title("Raised card")
                    .meta("surface")
                    .draw(ui, body, |ui, inner| {
                        let upper = Rect {
                            height: inner.height / 2,
                            ..inner
                        };
                        lines(
                            ui,
                            upper,
                            &[
                                "Cards lift content one surface plane.",
                                "Nested frames keep their own chrome.",
                            ],
                        );
                        let lower = Rect {
                            y: upper.bottom(),
                            height: inner.height.saturating_sub(upper.height),
                            ..inner
                        };
                        Panel::new(INNER)
                            .kind(PanelKind::Framed)
                            .title("Patched title")
                            .meta("accent + bold")
                            .patch_part(PARTS)
                            .draw(ui, lower, |ui, body| {
                                lines(
                                    ui,
                                    body,
                                    &[
                                        "This title uses a borrowed per-instance override.",
                                        "No theme mutation is required.",
                                    ],
                                );
                            });
                    });
            },
        );
    }
}
