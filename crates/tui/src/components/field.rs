//! `Field<C>` — draw-time field chrome (`COMPONENT_ARCHITECTURE.md` §15,
//! §21 item 7, Appendix A 4B).

use core::fmt;

use ratatui_core::layout::Rect;

use super::{Overrides, cell_at, first_row};
use crate::field_control::FieldControl;
use crate::id::{Part, PartRef};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{Family, GlyphRole, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};

/// Label, required / optional marker, help and error rows around a control.
///
/// ## Construction
/// `Field::new(label, control)` — no `Id`: the control owns identity, and
/// the chrome registers `Decorative` regions under the control's id.
///
/// ## Ownership
/// Draw-time chrome only: `Field` has no state and no `update`. The caller
/// keeps calling the control's `update` and owns the control's state.
///
/// ## Configuration
/// `.required(bool)` (`false`; paints `*`), `.optional_suffix(bool)`
/// (`true`; paints `optional` when not required and the row is wide
/// enough), `.help(&str)`, `.error(Option<&str>)` (wins over help),
/// `.plain(bool)` (`false`; suppresses the optional suffix),
/// `.patch_part`, `.state_override`.
///
/// ## Variants
/// `Family::FIELD`, `DEFAULT` only.
///
/// ## States
/// Reads the control's runtime flags (`FOCUSED`, `DISABLED`) and adds
/// `ERROR` when `.error` is `Some`; `.state_override` replaces the runtime
/// half for a reference rendering (A11).
///
/// ## Actions
/// None.
///
/// ## Focus
/// Never a focus stop; the control registers its own.
///
/// ## Keyboard
/// None.
///
/// ## Mouse
/// None; the chrome is `Decorative`.
///
/// ## Layout
/// Three rows — label, control, help/error — indented by two columns like
/// the legacy field; `measure` is the control's width by
/// `design.size.field_height`. A one-row area paints the label only, a
/// two-row area label and control; `draw` returns the rows used.
///
/// ## Parts
/// `CONTAINER`, `GUTTER` (reserved), `LABEL`, `MARKER` (the required
/// asterisk), `FIELD` (the control's row), `HELP` (help or error).
///
/// ## Overrides
/// `.patch_part` on the chrome parts; the control keeps its own overrides.
///
/// ## Identity
/// The control's id.
///
/// ## Testing
/// `FieldCase` (over a `TextInput`) with `FOCUSABLE | EDITS | CURSOR |
/// TYPES | DISABLEABLE`; `render::components::field::*`.
///
/// ## Invariants
/// One id per field (no `DuplicateId`); the chrome never commits, cancels
/// or validates; the error row is the caller's message, never derived.
pub struct Field<'a, C: FieldControl> {
    label: &'a str,
    required: bool,
    optional_suffix: bool,
    help: Option<&'a str>,
    error: Option<&'a str>,
    plain: bool,
    control: C,
    ov: Overrides<'a>,
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
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::LABEL,
        Part::MARKER,
        Part::FIELD,
        Part::HELP,
    ];

    /// Chrome around `control`.
    pub const fn new(label: &'a str, control: C) -> Self {
        Field {
            label,
            required: false,
            optional_suffix: true,
            help: None,
            error: None,
            plain: false,
            control,
            ov: Overrides::new(),
        }
    }

    /// Mark required (`*` after the label).
    #[must_use]
    pub const fn required(mut self, yes: bool) -> Self {
        self.required = yes;
        self
    }

    /// Whether a non-required field shows the `optional` suffix.
    #[must_use]
    pub const fn optional_suffix(mut self, yes: bool) -> Self {
        self.optional_suffix = yes;
        self
    }

    /// Help text under the control.
    #[must_use]
    pub const fn help(mut self, s: &'a str) -> Self {
        self.help = Some(s);
        self
    }

    /// An error message under the control (wins over help).
    #[must_use]
    pub const fn error(mut self, s: Option<&'a str>) -> Self {
        self.error = s;
        self
    }

    /// Plain label: no optional suffix.
    #[must_use]
    pub const fn plain(mut self, yes: bool) -> Self {
        self.plain = yes;
        self
    }

    /// Per-part instance patches for the chrome.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    /// Showcase / fixture use only (A11): render the chrome in a forced
    /// state instead of the control's runtime state. A forced field
    /// registers no decorative region.
    #[must_use]
    pub const fn state_override(mut self, s: StateFlags) -> Self {
        self.ov = self.ov.state_override(s);
        self
    }

    /// The control.
    pub const fn control(&self) -> &C {
        &self.control
    }

    /// The draw phase: label row, control, help/error row.
    #[expect(clippy::too_many_lines, reason = "one pass over the three chrome rows")]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &C::State) -> Rect {
        if area.is_empty() {
            return area;
        }
        let id = self.control.id();
        let mut live = self.ov.flags(ui.state(id));
        if self.error.is_some() {
            live |= StateFlags::ERROR;
        }
        let ov = self.ov;
        let forced = ov.is_forced();
        let style = |ui: &mut Ui<'_>, part: Part| {
            ov.style(ui, id, Family::FIELD, Variant::DEFAULT, part, live)
        };
        let container = style(ui, Part::CONTAINER);
        ui.fill(area, container.style);
        if !forced {
            ui.register_decor(id, PartRef::of(Part::CONTAINER), area);
        }

        // label row
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
                Some(g) => {
                    ui.glyph(cell, g, ms.style);
                }
                None => {
                    ui.paint_str(cell, "*", ms.style);
                }
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
        if !forced {
            ui.register_decor(id, PartRef::of(Part::LABEL), label_row);
        }
        if area.height < 2 {
            return label_row;
        }

        // control row(s)
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

        // help / error row
        let msg_row = Rect {
            x: area.x.saturating_add(2),
            y: rest_y,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        let msg = self.error.or(self.help);
        if let Some(m) = msg {
            let hs = style(ui, Part::HELP);
            if self.error.is_some()
                && let Some(g) = hs.glyph
            {
                let used = ui.glyph(msg_row, g, hs.style);
                ui.paint_str(super::shift(msg_row, used.saturating_add(1)), m, hs.style);
            } else {
                ui.paint_str(msg_row, m, hs.style);
            }
            let _ = GlyphRole::Error;
        }
        if !forced {
            ui.register_decor(id, PartRef::of(Part::HELP), msg_row);
        }
        Rect {
            height: used_h.saturating_add(1).min(area.height),
            ..area
        }
    }

    /// The control's width by `design.size.field_height`.
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
