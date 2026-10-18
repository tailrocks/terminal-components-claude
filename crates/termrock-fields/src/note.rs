#![doc = r" `Note` — a draw-only wrapped help note (`docs/components/note.md`)."]
#![doc = r""]
#![doc = r" Multi-line muted text under a control, where [`Field`](crate::field::Field)'s"]
#![doc = r" single truncated help row cannot carry the copy."]

use core::fmt;

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn};
use crate::id::{Id, Part, PartRef};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::{width, wrap, wrapped_rows};
use crate::theme::{Family, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};

#[doc = r" Wrapped muted text describing the control above it."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `Note::new(id, text)`."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" Stateless and draw-only: the caller owns the text. The chrome registers"]
#[doc = r" `Decorative` regions under `id` (the `Field` precedent), so a note may"]
#[doc = r" share its control's id."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.patch`, `.patch_part`, `.slot`."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::FIELD`, `DEFAULT` only."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" Wears no state; the text resolves the `FIELD` `HELP` tone."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" None; `Note` has no `update` phase."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" Never a focus stop; registers no ring entry."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" None."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" None; the chrome is `Decorative`."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" `measure` is the unwrapped width by the wrapped row count. `draw` fills"]
#[doc = r" `area` with `CONTAINER`, then paints the wrapped lines left-aligned at"]
#[doc = r" `area.x`, at most one per row of `area`; a degenerate rect paints"]
#[doc = r" nothing (R5)."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `CONTAINER` (the fill), `HELP` (the text)."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch` and `.patch_part` reach both parts."]
#[doc = r" `.slot(Part::CONTAINER, …)` replaces the whole surface."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" One `Id` per instance, used to attribute style resolution and overrides;"]
#[doc = r" no items."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `note::tests::note_wrap_and_clip`."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" Never writes outside `area`; never registers a focus stop or a control."]
pub struct Note<'a> {
    id: Id,
    text: &'a str,
    ov: PartStyle<'a>,
}

impl fmt::Debug for Note<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Note")
            .field("id", &self.id)
            .field("text", &self.text)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> Note<'a> {
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::HELP];

    #[doc = r" A note for `text`."]
    pub const fn new(id: Id, text: &'a str) -> Self {
        Note {
            id,
            text,
            ov: PartStyle::new(),
        }
    }

    #[doc = r" The id."]
    pub const fn id(&self) -> Id {
        self.id
    }

    #[doc = r" An instance patch over every part (precedence 6)."]
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[doc = r" Per-part patches."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" Replace the whole surface."]
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    #[doc = r" The draw phase; returns the rows the note occupies."]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }

        let live = PartStyle::flags(ui.state(self.id), StateFlags::empty());
        let ov = self.ov;
        if let Some(f) = ov.slot_for(Part::CONTAINER) {
            f(ui, area);
            return area;
        }
        let container = ov.style(
            ui,
            self.id,
            Family::FIELD,
            Variant::DEFAULT,
            Part::CONTAINER,
            live,
        );
        ui.fill(area, container.style);
        ui.register_decor(self.id, PartRef::of(Part::CONTAINER), area);
        let hs = ov.style(
            ui,
            self.id,
            Family::FIELD,
            Variant::DEFAULT,
            Part::HELP,
            live,
        );
        let lines = wrap(self.text, area.width.max(1));
        let mut painted = 0u16;
        for line in lines.iter().take(usize::from(area.height)) {
            ui.paint_str(
                Rect {
                    x: area.x,
                    y: area.y.saturating_add(painted),
                    width: area.width,
                    height: 1,
                },
                line,
                hs.style,
            );
            painted = painted.saturating_add(1);
        }
        ui.register_decor(
            self.id,
            PartRef::of(Part::HELP),
            Rect {
                height: painted,
                ..area
            },
        );
        Rect {
            height: painted,
            ..area
        }
    }

    #[doc = r" The natural size: the unwrapped width by the wrapped row count."]
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let w = width(self.text);
        Size {
            min: (w.min(c.max.0), 1),
            preferred: (w, wrapped_rows(self.text, c.max.0.max(1))),
        }
        .fit(c)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Position;
    use ratatui_core::style::Color;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::Theme;

    const NOTE: Id = Id::root("note.tests");
    const MUTED: Color = Color::Rgb(128, 128, 128);

    fn scene() -> (Runtime<Stub>, Buffer) {
        (
            Runtime::new(Stub::default(), Theme::junie()),
            Buffer::empty(SCREEN),
        )
    }

    fn cell_symbol(buf: &Buffer, x: u16, y: u16) -> &str {
        buf.cell(Position::new(x, y))
            .map(|cell| cell.symbol())
            .unwrap_or("<none>")
    }

    fn row_text(buf: &Buffer, y: u16, x0: u16, len: u16) -> String {
        (x0..x0.saturating_add(len))
            .map(|x| cell_symbol(buf, x, y))
            .collect()
    }

    #[test]
    fn note_wrap_and_clip() {
        let (mut rt, mut buf) = scene();
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            let used = Note::new(
                NOTE,
                "Writes run without asking. Destructive statements still confirm.",
            )
            .draw(ui, Rect::new(4, 2, 26, 2));
            assert_eq!(used, Rect::new(4, 2, 26, 2));
        })
        .commit_presented();
        assert_eq!(
            row_text(&buf, 2, 4, 26),
            "Writes run without asking.",
            "first wrap row"
        );
        assert_eq!(
            row_text(&buf, 3, 4, 26),
            "Destructive statements    ",
            "second wrap row"
        );
        for (x, y) in [(4, 2), (10, 2), (4, 3), (20, 3)] {
            let cell = buf
                .cell(Position::new(x, y))
                .expect("wrapped text must paint");
            assert_eq!(cell.fg, MUTED, "muted tone at ({x},{y})");
        }

        let (mut rt, mut buf) = scene();
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            let used = Note::new(
                NOTE,
                "Writes run without asking. Destructive statements still confirm.",
            )
            .draw(ui, Rect::new(4, 5, 26, 1));
            assert_eq!(used, Rect::new(4, 5, 26, 1));
        })
        .commit_presented();
        assert_eq!(
            row_text(&buf, 5, 4, 26),
            "Writes run without asking.",
            "only the first row paints"
        );
        assert_eq!(row_text(&buf, 6, 4, 26).trim(), "", "no spill below");

        let (mut rt, mut buf) = scene();
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            let used = Note::new(NOTE, "Writes run without asking.").draw(ui, Rect::ZERO);
            assert_eq!(used, Rect::ZERO);
        })
        .commit_presented();

        let (mut rt, mut buf) = scene();
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            let note = Note::new(
                NOTE,
                "Writes run without asking. Destructive statements still confirm.",
            );
            let size = note.measure(ui, Constraints::loose(26, 5));
            assert_eq!(size.preferred.1, 3, "three wrap rows at w=26");
            assert_eq!(size.min.1, 1);
        })
        .commit_presented();
    }
}
