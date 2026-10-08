//! `Note` — a draw-only wrapped help note (`docs/components/note.md`).
//!
//! Multi-line muted text under a control, where [`Field`](super::field::Field)'s
//! single truncated help row cannot carry the copy.

use core::fmt;

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn};
use crate::id::{Id, Part, PartRef};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::{width, wrap, wrapped_rows};
use crate::theme::{Family, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};

/// Wrapped muted text describing the control above it.
///
/// ## Construction
/// `Note::new(id, text)`.
///
/// ## Ownership
/// Stateless and draw-only: the caller owns the text. The chrome registers
/// `Decorative` regions under `id` (the `Field` precedent), so a note may
/// share its control's id.
///
/// ## Configuration
/// `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::FIELD`, `DEFAULT` only.
///
/// ## States
/// Wears no state; the text resolves the `FIELD` `HELP` tone.
///
/// ## Actions
/// None; `Note` has no `update` phase.
///
/// ## Focus
/// Never a focus stop; registers no ring entry.
///
/// ## Keyboard
/// None.
///
/// ## Mouse
/// None; the chrome is `Decorative`.
///
/// ## Layout
/// `measure` is the unwrapped width by the wrapped row count. `draw` fills
/// `area` with `CONTAINER`, then paints the wrapped lines left-aligned at
/// `area.x`, at most one per row of `area`; a degenerate rect paints
/// nothing (R5).
///
/// ## Parts
/// `CONTAINER` (the fill), `HELP` (the text).
///
/// ## Overrides
/// `.patch` and `.patch_part` reach both parts.
/// `.slot(Part::CONTAINER, …)` replaces the whole surface.
///
/// ## Identity
/// One `Id` per instance, used to attribute style resolution and overrides;
/// no items.
///
/// ## Testing
/// `note::tests::note_wrap_and_clip`.
///
/// ## Invariants
/// Never writes outside `area`; never registers a focus stop or a control.
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
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::HELP];

    /// A note for `text`.
    pub const fn new(id: Id, text: &'a str) -> Self {
        Note {
            id,
            text,
            ov: PartStyle::new(),
        }
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// An instance patch over every part (precedence 6).
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    /// Per-part patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    /// Replace the whole surface.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// The draw phase; returns the rows the note occupies.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }
        // runtime: the owner's frame state; derived: none — a note wears no
        // state of its own
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

    /// The natural size: the unwrapped width by the wrapped row count.
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
        // wraps to the area width, left-aligned at area.x
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

        // takes at most the area height: the second line is dropped at h=1
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

        // degenerate areas paint nothing
        let (mut rt, mut buf) = scene();
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            let used = Note::new(NOTE, "Writes run without asking.").draw(ui, Rect::ZERO);
            assert_eq!(used, Rect::ZERO);
        })
        .commit_presented();

        // measure states the wrapped rows without drawing
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
