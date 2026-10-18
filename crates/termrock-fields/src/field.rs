#![doc = r" `Field<C>` — draw-time field chrome (`COMPONENT_ARCHITECTURE.md` §15,"]
#![doc = r" §21 item 7, Appendix A 4B)."]

use core::fmt;

use ratatui_core::layout::Rect;

use super::{PartStyle, cell_at, first_row};
use crate::field_control::FieldControl;
use crate::id::{Part, PartRef};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};

#[doc = r" Label, required / optional marker, help and error rows around a control."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `Field::new(label, control)` — no `Id`: the control owns identity, and"]
#[doc = r" the chrome registers `Decorative` regions under the control's id."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" Draw-time chrome only: `Field` has no state and no `update`. The caller"]
#[doc = r" keeps calling the control's `update` and owns the control's state."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.required(bool)` (`false`; paints `*`), `.optional_suffix(bool)`"]
#[doc = r" (`true`; paints `optional` when not required and the row is wide"]
#[doc = r" enough), `.help(&str)`, `.error(Option<&str>)` (wins over help),"]
#[doc = r" `.plain(bool)` (`false`; suppresses the optional suffix),"]
#[doc = r" `.patch_part`."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::FIELD`, `DEFAULT` only."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" Reads the control's runtime flags (`FOCUSED`, `DISABLED`) and adds"]
#[doc = r" `ERROR` when `.error` is `Some`; reference fixtures inject runtime state"]
#[doc = r" through [`Ui::reference`](termrock_runtime::Ui::reference)."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" None."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" Never a focus stop; the control registers its own."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" None."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" None; the chrome is `Decorative`."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" Three rows — label, control, help/error — indented by two columns like"]
#[doc = r" the legacy field; `measure` is the control's width by"]
#[doc = r" `design.size.field_height`. A one-row area paints the label only, a"]
#[doc = r" two-row area label and control; `draw` returns the rows used."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `CONTAINER`, `GUTTER` (reserved), `LABEL`, `MARKER` (the required"]
#[doc = r" asterisk), `FIELD` (the control's row), `HELP` (help or error)."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch_part` on the chrome parts; the control keeps its own overrides."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" The control's id."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `FieldCase` (over a `TextInput`) with `FOCUSABLE | EDITS | CURSOR |"]
#[doc = r" TYPES | DISABLEABLE`; `render::components::field::*`."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" One id per field (no `DuplicateId`); the chrome never commits, cancels"]
#[doc = r" or validates; the error row is the caller's message, never derived."]
pub struct Field<'a, C: FieldControl> {
    label: &'a str,
    required: bool,
    optional_suffix: bool,
    help: Option<&'a str>,
    error: Option<&'a str>,
    plain: bool,
    control: C,
    ov: PartStyle<'a>,
}

impl<C: FieldControl> fmt::Debug for Field<'_, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("label", &self.label)
            .field("required", &self.required)
            .field("help", &self.help)
            .field("error", &self.error)
            .field("plain", &self.plain)
            .field("control", &self.control.id())
            .finish_non_exhaustive()
    }
}

impl<'a, C: FieldControl> Field<'a, C> {
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::LABEL,
        Part::MARKER,
        Part::FIELD,
        Part::HELP,
    ];

    #[doc = r" Chrome around `control`."]
    pub const fn new(label: &'a str, control: C) -> Self {
        Field {
            label,
            required: false,
            optional_suffix: true,
            help: None,
            error: None,
            plain: false,
            control,
            ov: PartStyle::new(),
        }
    }

    #[doc = r" Mark required (`*` after the label)."]
    #[must_use]
    pub const fn required(mut self, yes: bool) -> Self {
        self.required = yes;
        self
    }

    #[doc = r" Whether a non-required field shows the `optional` suffix."]
    #[must_use]
    pub const fn optional_suffix(mut self, yes: bool) -> Self {
        self.optional_suffix = yes;
        self
    }

    #[doc = r" Help text under the control."]
    #[must_use]
    pub const fn help(mut self, s: &'a str) -> Self {
        self.help = Some(s);
        self
    }

    #[doc = r" An error message under the control (wins over help)."]
    #[must_use]
    pub const fn error(mut self, s: Option<&'a str>) -> Self {
        self.error = s;
        self
    }

    #[doc = r" Plain label: no optional suffix."]
    #[must_use]
    pub const fn plain(mut self, yes: bool) -> Self {
        self.plain = yes;
        self
    }

    #[doc = r" Per-part instance patches for the chrome."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" The control."]
    pub const fn control(&self) -> &C {
        &self.control
    }

    #[doc = r" The draw phase: label row, control, help/error row."]
    #[expect(clippy::too_many_lines, reason = "one pass over the three chrome rows")]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &C::State) -> Rect {
        if area.is_empty() {
            return area;
        }
        let id = self.control.id();

        let derived = if self.error.is_some() {
            StateFlags::ERROR
        } else {
            StateFlags::empty()
        };
        let live = PartStyle::flags(ui.state(id), derived);
        let ov = self.ov;
        let style = |ui: &mut Ui<'_>, part: Part| {
            ov.style(ui, id, Family::FIELD, Variant::DEFAULT, part, live)
        };
        let container = style(ui, Part::CONTAINER);
        ui.fill(area, container.style);
        ui.register_decor(id, PartRef::of(Part::CONTAINER), area);

        let label_row = first_row(area);
        let text = Rect {
            x: area.x.saturating_add(2),
            width: area.width.saturating_sub(2),
            ..label_row
        };
        let ls = style(ui, Part::LABEL);
        let used = ui.paint_str(text, self.label, ls.style);
        let name_w = width(self.label);
        let show_optional = !self.required
            && !self.label.is_empty()
            && !self.plain
            && self.optional_suffix
            && name_w.saturating_add(12) <= area.width;
        if self.required && !self.label.is_empty() {
            let cell = cell_at(text, text.x.saturating_add(used).saturating_add(1));
            let ms = style(ui, Part::MARKER);
            match ms.glyph {
                Slot::Set(g) => {
                    ui.glyph(cell, g, ms.style);
                }
                Slot::Inherit => {
                    ui.paint_str(cell, "*", ms.style);
                }
                Slot::Clear => ui.fill(cell, ms.style),
            }
        } else if show_optional {
            let rest = Rect {
                x: text.x.saturating_add(used).saturating_add(2),
                width: text.width.saturating_sub(used).saturating_sub(2),
                ..text
            };
            let hs = style(ui, Part::HELP);
            ui.paint_str(rest, "optional", hs.style);
        }
        ui.register_decor(id, PartRef::of(Part::LABEL), label_row);
        if area.height < 2 {
            return label_row;
        }

        let control_h = self
            .control
            .measure(
                ui,
                Constraints::loose(area.width, area.height.saturating_sub(1)),
            )
            .preferred
            .1
            .max(1)
            .min(area.height.saturating_sub(1));
        let control_area = Rect {
            x: area.x,
            y: area.y.saturating_add(1),
            width: area.width,
            height: control_h,
        };
        let painted = self.control.draw(ui, control_area, st);
        let used_h = 1u16.saturating_add(painted.height.max(1));
        let rest_y = area.y.saturating_add(used_h);
        if rest_y >= area.bottom() {
            return Rect {
                height: used_h.min(area.height),
                ..area
            };
        }

        let msg_row = Rect {
            x: area.x.saturating_add(2),
            y: rest_y,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        let msg = self.error.or(self.help);
        if let Some(m) = msg {
            let hs = style(ui, Part::HELP);
            if self.error.is_some() {
                if let Slot::Set(g) = hs.glyph {
                    let used = ui.glyph(msg_row, g, hs.style);
                    ui.paint_str(super::shift(msg_row, used.saturating_add(1)), m, hs.style);
                } else {
                    ui.paint_str(msg_row, m, hs.style);
                }
            } else {
                ui.paint_str(msg_row, m, hs.style);
            }
            let _ = GlyphRole::Error;
        }
        ui.register_decor(id, PartRef::of(Part::HELP), msg_row);
        Rect {
            height: used_h.saturating_add(1).min(area.height),
            ..area
        }
    }

    #[doc = r" The control's width by `design.size.field_height`."]
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let inner = self.control.measure(ui, c);
        let h = ui.design().size.field_height;
        Size {
            min: (inner.min.0, h),
            preferred: (inner.preferred.0, h),
        }
        .fit(c)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;

    use super::super::input::{TextInput, TextInputState};
    use super::*;
    use crate::id::Id;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::Theme;

    const INPUT: Id = Id::root("field.tests.input");

    #[doc = r" A reference `Field` is a picture, not a control. Scoping only the"]
    #[doc = r" chrome previously left the control it owns registering a live"]
    #[doc = r" editor and a focus stop — a `DuplicateId` and a stray tab stop the"]
    #[doc = r" first time a page renders the same control twice."]
    #[test]
    fn a_reference_field_registers_no_control() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let st = TextInputState::default();
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            ui.reference(None, |ui| {
                Field::new("Name", TextInput::new(INPUT).value("Ada Lovelace"))
                    .required(true)
                    .help("The person's display name.")
                    .draw(ui, a, &st);
            });
        })
        .commit_presented();
        assert!(
            !rt.registry().delivers_to(INPUT),
            "the control registered a live region under a reference field"
        );
        assert_eq!(
            rt.ring().reachable().count(),
            0,
            "a reference rendering left a focus stop"
        );
    }
}
