//! A small downstream-authored component using only `tui_next::author`.
//!
//! This is intentionally kept in the application package. It proves that an
//! app-owned control can register focus and parts, receive pointer intents,
//! resolve per-instance styles, and render through the same public facade as
//! the built-in controls.

use tui_next::author::{
    Cx, Family, Focusability, FrameRead, Id, Intent, Part, PartRef, PartStyle, Phase, Rect,
    Response, Role, StateFlags, StylePatch, Ui, Variant,
};

const FAMILY: Family = Family::custom("showcase-author");
const AUTHOR_PATCH: StylePatch = StylePatch::new().set_fg(Role::Accent);
const AUTHOR_PARTS: &[(Part, StylePatch)] = &[(Part::LABEL, AUTHOR_PATCH)];

#[derive(Debug)]
pub(crate) struct AuthorBadge {
    id: Id,
    selected: bool,
}

impl AuthorBadge {
    pub(crate) const fn new(id: Id) -> Self {
        Self {
            id,
            selected: false,
        }
    }

    pub(crate) fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        for intent in cx.intents(self.id) {
            if let Intent::Pointer {
                phase: Phase::Click,
                ..
            } = intent
            {
                self.selected = !self.selected;
                response = Response::changed();
            }
        }
        response
    }

    pub(crate) fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        ui.register_control(self.id, area, Focusability::Focusable);
        ui.register_part(self.id, PartRef::of(Part::LABEL), area);
        let flags = ui.state(self.id);
        let styles = PartStyle::new().part(AUTHOR_PARTS);
        let container = styles.style(
            ui,
            self.id,
            FAMILY,
            Variant::DEFAULT,
            Part::CONTAINER,
            flags,
        );
        ui.fill(area, container.style);
        let mut label_flags = flags;
        if self.selected {
            label_flags |= StateFlags::SELECTED;
        }
        let label = styles.style(
            ui,
            self.id,
            FAMILY,
            Variant::DEFAULT,
            Part::LABEL,
            label_flags,
        );
        let text = if self.selected {
            "Author component · selected"
        } else {
            "Author component · click or Tab"
        };
        let _ = ui.paint_str(area, text, label.style);
    }
}
