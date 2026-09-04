//! Multiline editing and viewport scrolling.

use tui_next::{Cx, Id, Rect, Response, TextAction, TextArea, TextAreaState, Ui, id};

use super::{Page, frame, lines, rows};

const BODY: Id = id!("textareas.body");

const SAMPLE: &str = "# Release checklist\n\n1. Read the changelog\n2. Review migration notes\n3. Run the unit tests\n4. Run the integration suite\n5. Capture the visual frame\n6. Publish the release\n7. Verify the package\n8. Archive the report\n9. Notify the team\n10. Close the change\n11. Inspect logs\n12. Confirm rollback\n13. Tag the commit\n14. Push the branch\n15. Open the review\n16. Merge after approval\n17. Monitor deployment\n18. Check health\n19. Compare metrics\n20. Record evidence\n21. Remove temporary flags\n22. Update docs\n23. Send announcement\n24. Mark complete\n25. Retain audit trail\n26. Rotate credentials\n27. Run smoke checks\n28. Run final verification";

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
            value: String::from(SAMPLE),
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
