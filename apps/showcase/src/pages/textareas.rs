//! Multiline editing and viewport scrolling.

use tui_next::{Cx, Id, Rect, Response, TextAction, TextArea, TextAreaState, Ui, id};

use super::{Page, frame, lines, rows};

const BODY: Id = id!("textareas.body");

fn checklist() -> String {
    (1..=28)
        .map(|line| match line % 4 {
            0 => format!("{line:>2}. Run the integration suite and attach the report."),
            1 => format!("{line:>2}. Read src/api/billing.rs before touching invoices."),
            2 => format!("{line:>2}. Keep the public API stable; add, never rename."),
            _ => format!("{line:>2}. Open a PR against main with a clear summary."),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A controlled multi-line document with enough rows to exercise wheel and
/// keyboard scrolling at both baseline sizes.
#[derive(Debug)]
pub(crate) struct TextAreasPage {
    value: String,
    state: TextAreaState,
    last: &'static str,
}

impl TextAreasPage {
    pub(crate) fn new() -> Self {
        Self {
            value: checklist(),
            state: TextAreaState::default(),
            last: "ready",
        }
    }
}

impl Default for TextAreasPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for TextAreasPage {
    fn title(&self) -> &'static str {
        "Text areas"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let edit = TextArea::new(BODY, 8)
            .placeholder("Write a checklist")
            .update(cx, &mut self.state, &mut self.value);
        if let Some(action) = edit.action_ref() {
            self.last = match action {
                TextAction::Changed => "draft changed",
                TextAction::Committed => "document committed",
                TextAction::Cancelled => "draft cancelled",
                TextAction::MoveNext | TextAction::MovePrev => "focus moved",
            };
        }
        edit.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "multiline · wheel · arrows · Enter commit",
            |ui, body| {
                let regions = rows(body, 3);
                TextArea::new(BODY, 8).value(&self.value).draw(
                    ui,
                    regions.first().copied().unwrap_or(body),
                    &self.state,
                );
                let phase = if self.state.is_editing() {
                    "editing"
                } else {
                    "idle"
                };
                let info = format!(
                    "document: {} lines · scroll={} · {} ({})",
                    self.value.lines().count(),
                    self.state.scroll().offset(),
                    self.last,
                    phase
                );
                let facts = regions.get(1).copied().unwrap_or(body);
                let _ = ui.paint_str(facts, &info, ui.surface_style());
                lines(
                    ui,
                    regions.get(2).copied().unwrap_or(body),
                    &[
                        "Wheel moves the owned viewport while focus stays on the editor.",
                        "The 28-step checklist keeps clipping behavior observable in tests.",
                    ],
                );
            },
        );
    }
}
