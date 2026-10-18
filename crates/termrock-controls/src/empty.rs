use core::fmt;

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn};
use crate::id::{Id, Part};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::{width, wrapped_rows};
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};
use termrock_runtime::EmptyState;

pub struct Empty<'a> {
    id: Id,
    state: EmptyState<'a>,
    variant: Variant,
    frame: usize,
    ov: PartStyle<'a>,
}

impl fmt::Debug for Empty<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Empty")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("frame", &self.frame)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> Empty<'a> {
    pub const PARTS: &'static [Part] = &[Part::EMPTY, Part::TITLE, Part::HELP, Part::ICON];

    pub const fn new(id: Id, state: EmptyState<'a>) -> Self {
        Empty {
            id,
            state,
            variant: Variant::DEFAULT,
            frame: 0,
            ov: PartStyle::new(),
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    pub const fn state(&self) -> EmptyState<'a> {
        self.state
    }

    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }

        let live = PartStyle::flags(StateFlags::empty(), self.state.status().flags());
        let ov = self.ov;
        if let Some(f) = ov.slot_for(Part::EMPTY) {
            f(ui, area);
            return area;
        }
        let slot = ov.style(ui, self.id, Family::EMPTY, self.variant, Part::EMPTY, live);
        ui.fill(area, slot.style);
        let rows = self.rows(area.width);
        let top = area.y.saturating_add(area.height.saturating_sub(rows) / 2);
        let block = Rect {
            y: top,
            height: rows.min(area.height),
            ..area
        };
        let used = self.paint_state(ui, block, live);
        Rect {
            height: used,
            ..block
        }
    }

    fn paint_state(&self, ui: &mut Ui<'_>, area: Rect, live: StateFlags) -> u16 {
        if area.is_empty() {
            return 0;
        }
        let title = self
            .ov
            .style(ui, self.id, Family::EMPTY, self.variant, Part::TITLE, live);
        let help = self
            .ov
            .style(ui, self.id, Family::EMPTY, self.variant, Part::HELP, live);
        let icon = self
            .ov
            .style(ui, self.id, Family::EMPTY, self.variant, Part::ICON, live);
        let glyph = match self.state {
            EmptyState::Loading { .. } | EmptyState::Partial { .. } => {
                let frames = ui.design().motion.spinner_frames;
                frames
                    .get(self.frame.checked_rem(frames.len()).unwrap_or(0))
                    .copied()
            }
            EmptyState::Error { .. } => match icon.glyph {
                Slot::Set(g) => Some(ui.glyph_str(g)),
                Slot::Inherit => Some(ui.glyph_str(GlyphRole::Error)),
                Slot::Clear => None,
            },
            EmptyState::Empty { .. } => None,
        };
        let prefix_w = glyph.map_or(0, |g| width(g).saturating_add(1));
        let head = self.state.title();
        let total_w = prefix_w.saturating_add(width(head)).min(area.width);
        let x = area
            .x
            .saturating_add(area.width.saturating_sub(total_w) / 2);
        let mut row = Rect {
            x,
            y: area.y,
            width: total_w,
            height: 1,
        };
        if let Some(g) = glyph {
            let used = ui.paint_str(row, g, icon.style);
            row.x = row.x.saturating_add(used).saturating_add(1);
            row.width = row.width.saturating_sub(used).saturating_sub(1);
        }
        ui.paint_str(row, head, title.style);
        let mut rows = 1u16;
        if let Some(detail) = self.state.detail()
            && area.height >= 3
        {
            let detail_w = width(detail).min(area.width);
            let detail_x = area
                .x
                .saturating_add(area.width.saturating_sub(detail_w) / 2);
            ui.paint_str(
                Rect {
                    x: detail_x,
                    y: area.y.saturating_add(2),
                    width: detail_w,
                    height: 1,
                },
                detail,
                help.style,
            );
            rows = 3;
        }
        rows
    }

    fn rows(&self, w: u16) -> u16 {
        match self.state.detail() {
            Some(d) => wrapped_rows(d, w.max(1)).saturating_add(2),
            None => 1,
        }
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let title = width(self.state.title());
        let detail = self.state.detail().map_or(0, width);
        let w = title.max(detail);
        Size {
            min: (title.min(c.max.0), 1),
            preferred: (w, self.rows(c.max.0.max(1))),
        }
        .fit(c)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::Modifier;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::{Role, Theme};

    const AREA: Rect = Rect::new(0, 0, 24, 5);
    const EMPTY: Id = Id::root("empty.tests");

    fn cell_with<'a>(buffer: &'a Buffer, symbol: &str) -> Option<&'a ratatui_core::buffer::Cell> {
        buffer.content().iter().find(|cell| cell.symbol() == symbol)
    }

    fn render(empty: &Empty<'_>, theme: Theme) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                empty.draw(ui, area);
            })
            .commit_presented();
        buffer
    }

    #[test]
    fn nested_parts_honor_instance_patches() {
        let all = StylePatch::new().add(Modifier::BOLD);
        let parts = [
            (Part::TITLE, StylePatch::new().add(Modifier::ITALIC)),
            (Part::HELP, StylePatch::new().add(Modifier::UNDERLINED)),
            (Part::ICON, StylePatch::new().add(Modifier::REVERSED)),
        ];
        let buffer = render(
            &Empty::new(
                EMPTY,
                EmptyState::Error {
                    message: "T",
                    detail: Some("H"),
                },
            )
            .patch(&all)
            .patch_part(&parts),
            Theme::junie(),
        );
        let icon = Theme::junie().design.glyphs.get(GlyphRole::Error);

        assert!(
            cell_with(&buffer, "T")
                .map(|cell| cell.modifier)
                .unwrap_or_default()
                .contains(Modifier::BOLD | Modifier::ITALIC)
        );
        assert!(
            cell_with(&buffer, "H")
                .map(|cell| cell.modifier)
                .unwrap_or_default()
                .contains(Modifier::BOLD | Modifier::UNDERLINED)
        );
        assert!(
            cell_with(&buffer, icon)
                .map(|cell| cell.modifier)
                .unwrap_or_default()
                .contains(Modifier::BOLD | Modifier::REVERSED)
        );
    }

    #[test]
    fn readiness_reaches_nested_parts() {
        let theme = Theme::junie().override_family(Family::EMPTY, |recipe| {
            recipe
                .part(Part::TITLE)
                .when(StateFlags::ERROR, StylePatch::new().set_fg(Role::Info));
        });
        let info = theme.color.info;
        let error = theme.design.glyphs.get(GlyphRole::Error);
        let buffer = render(
            &Empty::new(
                EMPTY,
                EmptyState::Error {
                    message: "T",
                    detail: None,
                },
            ),
            theme,
        );

        assert_eq!(cell_with(&buffer, "T").map(|cell| cell.fg), Some(info));
        assert!(cell_with(&buffer, error).is_some());
    }
}
