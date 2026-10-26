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
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::LABEL,
        Part::MARKER,
        Part::FIELD,
        Part::HELP,
    ];

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

    #[must_use]
    pub const fn required(mut self, yes: bool) -> Self {
        self.required = yes;
        self
    }

    #[must_use]
    pub const fn optional_suffix(mut self, yes: bool) -> Self {
        self.optional_suffix = yes;
        self
    }

    #[must_use]
    pub const fn help(mut self, s: &'a str) -> Self {
        self.help = Some(s);
        self
    }

    #[must_use]
    pub const fn error(mut self, s: Option<&'a str>) -> Self {
        self.error = s;
        self
    }

    #[must_use]
    pub const fn plain(mut self, yes: bool) -> Self {
        self.plain = yes;
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    pub const fn control(&self) -> &C {
        &self.control
    }

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
