//! The component families (`COMPONENT_ARCHITECTURE.md` §3–§15, Appendix A
//! Slice 4): props structs with consuming builders, caller-owned `XState`s,
//! `update`/`draw` phase methods and per-component binding tables.
//!
//! Every component here follows §13: `X::new(id, …)`, consuming builders,
//! `update(&self, cx, &mut st[, data]) -> Response<XAction>`,
//! `draw(&self, ui, area, &st[, data]) -> Rect`, `measure`, `PARTS`.

pub(crate) mod button;
pub(crate) mod dialog;
pub(crate) mod field;
pub(crate) mod input;
pub(crate) mod list;
pub(crate) mod props;
pub(crate) mod scroll_region;
pub(crate) mod tabs;

pub use button::{Button, ButtonCmd};
pub use dialog::{Dialog, DialogAction, DialogCmd, DialogState};
pub use field::Field;
pub use input::{BlurPolicy, EditPhase, TextAction, TextCmd, TextInput, TextInputState};
pub use list::{List, ListAction, ListCmd, ListState};
pub use props::Props;
pub use scroll_region::ScrollRegion;
pub use tabs::{Tabs, TabsAction, TabsCmd, TabsState};

use core::fmt;

use ratatui_core::layout::Rect;

use crate::id::{Id, Part};
use crate::response::StateFlags;
use crate::theme::{Family, Resolved, StylePatch, Variant};
use crate::ui::Ui;

/// A replaced part: the component keeps layout, hit registration, focus and
/// state; the closure paints the part's rect.
pub(crate) type SlotFn<'a> = &'a dyn Fn(&mut Ui<'_>, Rect);

/// The per-instance override set every component carries (§12.1, §13):
/// `.patch`, `.patch_part`, `.slot` and the showcase-only `.state_override`.
#[derive(Clone, Copy)]
pub(crate) struct Overrides<'a> {
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    slot: Option<(Part, SlotFn<'a>)>,
    state: Option<StateFlags>,
}

impl fmt::Debug for Overrides<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Overrides")
            .field("patch", &self.patch)
            .field("parts", &self.parts.len())
            .field("slot", &self.slot.map(|(p, _)| p))
            .field("state", &self.state)
            .finish()
    }
}

impl<'a> Overrides<'a> {
    pub(crate) const fn new() -> Self {
        Overrides {
            patch: None,
            parts: &[],
            slot: None,
            state: None,
        }
    }

    pub(crate) const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.patch = Some(p);
        self
    }

    pub(crate) const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self
    }

    pub(crate) const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.slot = Some((p, f));
        self
    }

    pub(crate) const fn state_override(mut self, s: StateFlags) -> Self {
        self.state = Some(s);
        self
    }

    /// Whether a forced state is set (a reference rendering, A11).
    pub(crate) const fn is_forced(&self) -> bool {
        self.state.is_some()
    }

    /// The live flags: the forced state when set, else `live`.
    pub(crate) fn flags(&self, live: StateFlags) -> StateFlags {
        self.state.unwrap_or(live)
    }

    /// The instance patch for `part`: `.patch` merged with every matching
    /// `.patch_part` entry, in declaration order.
    pub(crate) fn part_patch(&self, part: Part) -> Option<StylePatch> {
        let mut acc: Option<StylePatch> = self.patch.copied();
        for (p, patch) in self.parts {
            if *p == part {
                acc = Some(match acc {
                    Some(a) => a.merge(*patch),
                    None => *patch,
                });
            }
        }
        acc
    }

    /// The slot replacing `part`, if any.
    pub(crate) fn slot_for(&self, part: Part) -> Option<SlotFn<'a>> {
        match self.slot {
            Some((p, f)) if p == part => Some(f),
            _ => None,
        }
    }

    /// Resolve `part` through the whole chain including the instance patch.
    pub(crate) fn style(
        &self,
        ui: &mut Ui<'_>,
        owner: Id,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
    ) -> Resolved {
        let r = match self.part_patch(part) {
            Some(p) => ui.style_patched(family, variant, part, flags, &p),
            None => ui.style(family, variant, part, flags),
        };
        #[cfg(feature = "testing")]
        ui.note_styled(owner, part);
        #[cfg(not(feature = "testing"))]
        let _ = owner;
        r
    }
}

/// The first row of `area`, or an empty rect.
pub(crate) const fn first_row(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: if area.height == 0 { 0 } else { 1 },
    }
}

/// A one-cell-wide column of `area` at `x`.
pub(crate) const fn cell_at(area: Rect, x: u16) -> Rect {
    Rect {
        x,
        y: area.y,
        width: if x < area.x.saturating_add(area.width) {
            1
        } else {
            0
        },
        height: area.height,
    }
}

/// `area` shifted right by `by` columns, shrinking its width.
pub(crate) const fn shift(area: Rect, by: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(by),
        y: area.y,
        width: area.width.saturating_sub(by),
        height: area.height,
    }
}

/// Folds a component's per-intent outcomes into one `Response<A>`: an
/// action wins over a repaint, a repaint over a bare consume.
pub(crate) struct Acc<A> {
    consumed: bool,
    invalidate: crate::response::Invalidate,
    action: Option<A>,
}

impl<A> Acc<A> {
    pub(crate) const fn new() -> Self {
        Acc {
            consumed: false,
            invalidate: crate::response::Invalidate::None,
            action: None,
        }
    }

    pub(crate) fn consumed(&mut self) {
        self.consumed = true;
    }

    pub(crate) fn changed(&mut self) {
        self.consumed = true;
        self.invalidate = self.invalidate.max(crate::response::Invalidate::Paint);
    }

    pub(crate) fn action(&mut self, a: A) {
        self.changed();
        self.action = Some(a);
    }

    pub(crate) fn fold(&mut self, r: &crate::response::Response<()>) {
        self.consumed |= r.is_consumed();
        self.invalidate = self.invalidate.max(r.invalidate());
    }

    pub(crate) fn finish(self, id: Id) -> crate::response::Response<A> {
        use crate::response::{Invalidate, Response};
        let r = match self.action {
            Some(a) => Response::action(a),
            None if self.consumed => Response::consumed(),
            None => return Response::ignored(),
        };
        let r = match self.invalidate {
            Invalidate::None => r,
            Invalidate::Paint => r.repaint(),
            Invalidate::Layout => r.relayout(),
        };
        r.for_id(id)
    }
}
