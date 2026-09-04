//! `Props` — a two-column label / value list (`COMPONENT_ARCHITECTURE.md`
//! §12.4, §17.0 A7).

use core::fmt;

use ratatui_core::layout::Rect;

use super::Overrides;
use crate::id::Part;
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{Family, StylePatch, Variant};
use crate::ui::Ui;

/// Label / value rows: muted labels in a column sized to the widest, values
/// beside them.
///
/// ## Construction
/// `Props::new(rows)` over `&[(&str, &str)]`.
///
/// ## Ownership
/// Stateless; the rows are borrowed.
///
/// ## Configuration
/// `.patch_part`.
///
/// ## Variants
/// `Family::PROPS`, `DEFAULT` only.
///
/// ## States
/// None.
///
/// ## Actions
/// None.
///
/// ## Focus
/// Never a focus stop; registers nothing (it has no id).
///
/// ## Keyboard
/// None.
///
/// ## Mouse
/// None.
///
/// ## Layout
/// One row per pair; `measure` is `(widest label + 2 + widest value,
/// rows)`; `draw` returns the rows painted, clipped to `area`.
///
/// ## Parts
/// `META` (the label column), `LABEL` (the value column).
///
/// ## Overrides
/// `.patch_part` on both parts.
///
/// ## Identity
/// None.
///
/// ## Testing
/// `PropsCase` with no capabilities; `render::components::dialog::*` covers
/// it inside a body.
///
/// ## Invariants
/// Never writes outside `area`; never allocates.
pub struct Props<'a> {
    rows: &'a [(&'a str, &'a str)],
    ov: Overrides<'a>,
}

impl fmt::Debug for Props<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Props")
            .field("rows", &self.rows.len())
            .finish_non_exhaustive()
    }
}

impl<'a> Props<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[Part::META, Part::LABEL];

    /// Rows of `(label, value)`.
    pub const fn new(rows: &'a [(&'a str, &'a str)]) -> Self {
        Props {
            rows,
            ov: Overrides::new(),
        }
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    fn label_width(&self) -> u16 {
        self.rows.iter().map(|(k, _)| width(k)).max().unwrap_or(0)
    }

    /// The draw phase.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }
        let lw = self.label_width().min(area.width);
        let ov = self.ov;
        let owner = crate::id::Id::root("tui.props");
        let key_style = ov
            .style(
                ui,
                owner,
                Family::PROPS,
                Variant::DEFAULT,
                Part::META,
                StateFlags::empty(),
            )
            .style;
        let value_style = ov
            .style(
                ui,
                owner,
                Family::PROPS,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::empty(),
            )
            .style;
        let mut painted = 0u16;
        for (row, (k, v)) in area.rows().zip(self.rows.iter()) {
            ui.paint_str(row, k, key_style);
            let value = Rect {
                x: row.x.saturating_add(lw).saturating_add(2),
                width: row.width.saturating_sub(lw).saturating_sub(2),
                ..row
            };
            ui.paint_str(value, v, value_style);
            painted = painted.saturating_add(1);
        }
        Rect {
            height: painted,
            ..area
        }
    }

    /// The natural size.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let vw = self.rows.iter().map(|(_, v)| width(v)).max().unwrap_or(0);
        let w = self.label_width().saturating_add(2).saturating_add(vw);
        let h = self.rows.len().min(usize::from(u16::MAX)) as u16;
        Size::exact(w, h).fit(c)
    }
}
