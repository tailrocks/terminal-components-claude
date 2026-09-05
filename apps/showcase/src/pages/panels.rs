//! Card and framed panel composition, including caller-owned overrides.

use junie_tui::{
    Modifier, Panel, PanelKind, Part, Rect, Response, Role, SplitAxis, SplitPane, SplitPaneState,
    StylePatch, Ui, id,
};

use super::{Page, frame, lines};

const OUTER: junie_tui::Id = id!("panels.outer");
const INNER: junie_tui::Id = id!("panels.inner");
const SPLIT: junie_tui::Id = id!("panels.split");
const PATCH: StylePatch = StylePatch::new().set_fg(Role::Accent).add(Modifier::BOLD);
const PARTS: &[(Part, StylePatch)] = &[(Part::TITLE, PATCH)];

fn outer_panel() -> Panel<'static> {
    Panel::new(OUTER)
        .kind(PanelKind::Card)
        .title("Raised card")
        .meta("surface")
}

fn inner_panel() -> Panel<'static> {
    Panel::new(INNER)
        .kind(PanelKind::Framed)
        .title("Patched title")
        .meta("accent + bold")
        .patch_part(PARTS)
}

fn split_pane() -> SplitPane<'static> {
    SplitPane::new(SPLIT, SplitAxis::Vertical)
        .min_first(4)
        .min_second(4)
        .resizable(true)
}

/// Static panel surfaces are still useful to test the live theme and patch
/// precedence without introducing application-owned rendering code.
#[derive(Debug, Default)]
pub(crate) struct PanelsPage {
    split: SplitPaneState,
}

impl PanelsPage {
    pub(crate) fn new() -> Self {
        Self {
            split: SplitPaneState::default(),
        }
    }
}

impl Page for PanelsPage {
    fn title(&self) -> &'static str {
        "Panels"
    }

    fn update(&mut self, cx: &mut junie_tui::Cx<'_>) -> Response<()> {
        let _ = outer_panel();
        let _ = inner_panel();
        let _ = split_pane().update(cx, &mut self.split);
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "card · framed · per-instance patch",
            |ui, body| {
                split_pane().draw(ui, body, &self.split, |ui, upper, lower| {
                    outer_panel().draw(ui, upper, |ui, body| {
                        lines(
                            ui,
                            body,
                            &[
                                "Cards lift content one surface plane.",
                                "Nested frames keep their own chrome.",
                            ],
                        );
                    });
                    inner_panel().draw(ui, lower, |ui, body| {
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
