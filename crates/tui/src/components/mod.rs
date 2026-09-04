//! The component families (`COMPONENT_ARCHITECTURE.md` §3–§15, Appendix A
//! Slice 4): props structs with consuming builders, caller-owned `XState`s,
//! `update`/`draw` phase methods and per-component binding tables.
//!
//! Every component here follows §13: `X::new(id, …)`, consuming builders,
//! `update(&self, cx, &mut st[, data]) -> Response<XAction>`,
//! `draw(&self, ui, area, &st[, data]) -> Rect`, `measure`, `PARTS`.

pub(crate) mod brand;
pub(crate) mod button;
pub(crate) mod chip;
pub(crate) mod choice;
pub(crate) mod dialog;
pub(crate) mod empty;
pub(crate) mod field;
pub(crate) mod hintbar;
pub(crate) mod input;
pub(crate) mod keyhint;
pub(crate) mod list;
pub(crate) mod meter;
pub(crate) mod progress;
pub(crate) mod props;
pub(crate) mod scroll_region;
pub(crate) mod select;
pub(crate) mod status;
pub(crate) mod tabs;
pub(crate) mod textarea;

pub use brand::Brand;
pub use button::{Button, ButtonCmd};
pub use chip::{ChipBar, ChipBarAction, ChipBarCmd, ChipBarState, LabelChips};
pub use choice::{
    Checkbox, ChoiceCmd, LabelRadio, RadioGroup, RadioGroupAction, RadioGroupState, Toggle,
};
pub use dialog::{Dialog, DialogAction, DialogCmd, DialogState};
pub use empty::Empty;
pub use field::Field;
pub use hintbar::HintBar;
pub use input::{BlurPolicy, EditPhase, TextAction, TextCmd, TextInput, TextInputState};
pub use keyhint::KeyHint;
pub use list::{List, ListAction, ListCmd, ListState};
pub use meter::{Meter, MeterTone, MeterVisual};
pub use progress::{ProgressBar, Spinner};
pub use props::Props;
pub use scroll_region::ScrollRegion;
pub use select::{LabelSelect, Select, SelectAction, SelectCmd, SelectState};
pub use status::{Emphasis, Group, MAX_ITEMS, StatusAction, StatusBar, StatusItem};
pub use tabs::{Tabs, TabsAction, TabsCmd, TabsState};
pub use textarea::{TextArea, TextAreaState};

use core::fmt;

use ratatui_core::layout::Rect;
use ratatui_core::style::Style;

use crate::id::{Id, Part};
use crate::response::StateFlags;
use crate::theme::{Family, GlyphRole, Resolved, StylePatch, Variant};
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

    /// The forced state, if any.
    pub(crate) const fn forced_state(&self) -> Option<StateFlags> {
        self.state
    }

    /// Adopt an owning container's forced state. This is the **composition**
    /// half of A11: when a forced container draws a child component it owns
    /// (a `Dialog`'s action buttons), the child must render that state and
    /// register nothing too, or the reference rendering would put live,
    /// clickable controls on the page. It is distinct from the public
    /// `.state_override` builder, which is the showcase / fixture entry
    /// point and the only way a *caller* can force a state.
    pub(crate) const fn inherit_forced(mut self, s: Option<StateFlags>) -> Self {
        if s.is_some() {
            self.state = s;
        }
        self
    }

    /// The flags a part resolves under (§39.2, Invariant Q).
    ///
    /// The two halves have **opposite ownership**. `runtime` is what the
    /// frame supplies — `Ui::state(id)`, the focus, hover and press the
    /// snapshot carries. `derived` is what the caller's own props imply —
    /// `Status::flags`, a `DISABLED` from a `.disabled` builder, a `CHECKED`
    /// from a `.checked` one. **A forced state substitutes for the runtime
    /// and never for the props**, so `flags(r, d) ⊇ d` holds unconditionally:
    /// a reference rendering may show a state the runtime never produced, and
    /// may never hide a state the props declare.
    pub(crate) fn flags(&self, runtime: StateFlags, derived: StateFlags) -> StateFlags {
        self.state
            .map_or(runtime | derived, |forced| forced | derived)
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
        ui.note_styled(owner, family, variant, part, r);
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

/// A one-cell-wide column of `area` at `x`, or an empty rect when `x` is
/// outside `area`.
///
/// The bound is enforced on **both** sides. Callers anchor a cell to the right
/// edge with `area.right().saturating_sub(n)`, which lands left of `area.x`
/// whenever `area.width < n` — at `area.width == 1`, `right() - 2` is
/// `area.x - 1`. Checking only the right edge made that a paintable cell one
/// column outside the component, so the helper enforces the whole extent its
/// name promises rather than leaving each caller to clamp.
pub(crate) const fn cell_at(area: Rect, x: u16) -> Rect {
    Rect {
        x,
        y: area.y,
        width: if x >= area.x && x < area.x.saturating_add(area.width) {
            1
        } else {
            0
        },
        height: area.height,
    }
}

/// Paint the mono pressed bracket into two cells reserved by the component.
pub(crate) fn paint_pressed_bracket(ui: &mut Ui<'_>, left: Rect, right: Rect, style: Style) {
    ui.glyph(left, GlyphRole::PressLeft, style);
    ui.glyph(right, GlyphRole::PressRight, style);
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

    /// Request a repaint **without** consuming: a notification the component
    /// drains and reacts to, but which must not swallow the input that is
    /// still being dispatched.
    pub(crate) fn repaint(&mut self) {
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
            None if self.invalidate == Invalidate::None => return Response::ignored(),
            None => Response::ignored(),
        };
        let r = match self.invalidate {
            Invalidate::None => r,
            Invalidate::Paint => r.repaint(),
            Invalidate::Layout => r.relayout(),
        };
        r.for_id(id)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;

    use super::{Overrides, cell_at};
    use crate::collection::Status;
    use crate::components::{HintBar, Meter, ProgressBar};
    use crate::event::{Chord, KeyCode};
    use crate::id::Id;
    use crate::keymap::{Hint, HintLayer};
    use crate::response::StateFlags;
    use crate::theme::{GlyphRole, Theme};
    use crate::ui::cx::LastFrame;
    use crate::ui::{FrameState, Ui, UiCore};

    const AREA: Rect = Rect {
        x: 4,
        y: 5,
        width: 3,
        height: 2,
    };

    /// `cell_at` names a cell **of `area`**, so it must yield a zero-width
    /// rect for every `x` outside `area`'s horizontal extent — to the left of
    /// `area.x` exactly as much as at or past `area.x + area.width`. Only the
    /// right edge was enforced, so a caller computing a right-anchored column
    /// with `saturating_sub` (`Select`'s trailing indicator,
    /// `cell_at(area, area.right() - 2)` at `area.width == 1`) got a paintable
    /// cell one column outside the component.
    #[test]
    fn cell_at_is_empty_for_every_x_outside_the_area_on_either_side() {
        // left of `area.x`: empty, both one short and far short
        assert_eq!(cell_at(AREA, AREA.x - 1).width, 0);
        assert_eq!(cell_at(AREA, 0).width, 0);
        // the `Select` case verbatim: a 1-wide area's `right() - 2` is `x - 1`
        let narrow = Rect { width: 1, ..AREA };
        assert_eq!(cell_at(narrow, narrow.right().saturating_sub(2)).width, 0);
        // a zero-width area has no cells at all, including at its own `x`
        let empty = Rect { width: 0, ..AREA };
        assert_eq!(cell_at(empty, empty.x).width, 0);

        // unchanged: every in-range `x` is a one-cell column of `area`
        for x in AREA.x..AREA.right() {
            assert_eq!(
                cell_at(AREA, x),
                Rect {
                    x,
                    y: AREA.y,
                    width: 1,
                    height: AREA.height,
                }
            );
        }

        // unchanged: at or past the right edge is empty, and keeps `x`, `y`
        // and `height` as they were
        assert_eq!(
            cell_at(AREA, AREA.right()),
            Rect {
                x: AREA.right(),
                y: AREA.y,
                width: 0,
                height: AREA.height,
            }
        );
        assert_eq!(cell_at(AREA, AREA.right() + 1).width, 0);
    }

    /// §39.2, Invariant Q — the operator law. A forced state substitutes for
    /// the **runtime** half the frame supplies and never for the **derived**
    /// half the caller's own props imply, so the flags a part resolves under
    /// are `forced.map_or(runtime | derived, |f| f | derived)` and
    /// `flags(r, d) ⊇ d` holds unconditionally.
    ///
    /// Before this, `Overrides::flags` took one argument and answered
    /// `self.state.unwrap_or(live)`, so a forced state *replaced* the
    /// props-derived half: `.status(Error).state_override(DISABLED)`
    /// resolved to `DISABLED` alone and meant "disabled and **not** in
    /// error", a rendering no caller can ask for by any other route.
    #[test]
    fn forcing_adds_to_the_derived_state_and_never_erases_it() {
        let unforced = Overrides::new();
        let runtime = StateFlags::HOVERED | StateFlags::FOCUSED;
        let derived = StateFlags::ERROR;

        // unforced: the plain union of the two halves
        assert_eq!(unforced.flags(runtime, derived), runtime | derived);
        assert_eq!(
            unforced.flags(StateFlags::empty(), StateFlags::empty()),
            StateFlags::empty()
        );

        // forcing substitutes for the runtime half, so a `HOVERED` the
        // snapshot still carries from a previous frame is gone — the job A11
        // exists for
        let forced = Overrides::new().state_override(StateFlags::DISABLED);
        assert_eq!(
            forced.flags(runtime, StateFlags::empty()),
            StateFlags::DISABLED
        );

        // and never for the derived half: this is §39.1's defect, verbatim
        assert_eq!(
            forced.flags(runtime, derived),
            StateFlags::DISABLED | StateFlags::ERROR
        );

        // forcing *nothing* adds nothing, which is why §39.2 keeps the
        // `Option` an `Option` instead of promoting it to a `Slot`
        let forced_empty = Overrides::new().state_override(StateFlags::empty());
        assert_eq!(
            forced_empty.flags(StateFlags::empty(), derived),
            unforced.flags(StateFlags::empty(), derived)
        );
        assert_eq!(forced_empty.flags(runtime, derived), derived);

        // `flags(r, d) ⊇ d`, swept over every pairing of a representative set
        let set = [
            StateFlags::empty(),
            StateFlags::DISABLED,
            StateFlags::ERROR,
            StateFlags::BUSY | StateFlags::LOADING,
            StateFlags::HOVERED | StateFlags::PRESSED,
            StateFlags::CHECKED | StateFlags::SELECTED,
            StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::EDITING,
        ];
        for r in set {
            for d in set {
                assert!(
                    unforced.flags(r, d).contains(d),
                    "unforced dropped the derived half: r={r:?} d={d:?}"
                );
                assert_eq!(unforced.flags(r, d), r | d, "r={r:?} d={d:?}");
                for f in set {
                    let ov = Overrides::new().state_override(f);
                    let got = ov.flags(r, d);
                    assert!(
                        got.contains(d),
                        "forcing {f:?} erased the derived half: r={r:?} d={d:?} -> {got:?}"
                    );
                    assert_eq!(got, f | d, "r={r:?} d={d:?} f={f:?}");
                }
            }
        }
    }

    const ROW: Rect = Rect {
        x: 0,
        y: 0,
        width: 60,
        height: 1,
    };

    /// Paints `f` onto a one-row screen and returns the row's text together
    /// with the theme's `GlyphRole::Error` string.
    fn painted(f: impl FnOnce(&mut Ui<'_>, Rect)) -> (String, &'static str) {
        let theme = Theme::junie();
        let mut fs = FrameState::default();
        fs.reset(1, ROW);
        let mut page = Buffer::empty(ROW);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let glyph = {
            let mut ui = Ui::new(&mut fs, &mut page, &mut core, &theme, &last);
            let g = ui.glyph_str(GlyphRole::Error);
            f(&mut ui, ROW);
            g
        };
        let mut text = String::new();
        for x in 0..ROW.width {
            if let Some(c) = page.cell((x, 0)) {
                text.push_str(c.symbol());
            }
        }
        (text, glyph)
    }

    /// §39.1's visible consequence. The render matrix forces `DISABLED` onto
    /// a component whose props say `Status::Error`, and under the replacing
    /// operator the forced state won outright: the bar **was** in error and
    /// painted no error glyph, contradicting `Caps::REPORTS_STATUS`'s own
    /// obligation and §11.4. Under Invariant Q the props-derived `ERROR`
    /// survives the forcing and the recipe's `.when(ERROR)` rule fires.
    ///
    /// `Status::Ready` is the control on every component here: the same
    /// forced state with no error in the props must *not* paint the glyph, so
    /// neither half of the assertion passes vacuously.
    #[test]
    fn a_forced_component_resolves_its_props_derived_state() {
        let id = Id::root("q");

        // ProgressBar: the trailing `Part::ICON` takes the recipe's
        // `ERROR -> GlyphRole::Error` rule
        let (bar, glyph) = painted(|ui, area| {
            ProgressBar::new(id)
                .ratio(0.65)
                .status(Status::Error)
                .state_override(StateFlags::DISABLED)
                .draw(ui, area);
        });
        assert!(
            bar.contains(glyph),
            "a forced, erroring progress bar painted no error glyph: {bar:?}"
        );
        let (bar_ready, _) = painted(|ui, area| {
            ProgressBar::new(id)
                .ratio(0.65)
                .status(Status::Ready)
                .state_override(StateFlags::DISABLED)
                .draw(ui, area);
        });
        assert!(
            !bar_ready.contains(glyph),
            "a forced, ready progress bar painted an error glyph: {bar_ready:?}"
        );

        // Meter: the same recipe, and its `icon` fallback reads the resolved
        // flags rather than `self.status`, so the glyph has one source
        let (meter, _) = painted(|ui, area| {
            Meter::new(id)
                .ratio(0.65)
                .value("65%")
                .status(Status::Error)
                .state_override(StateFlags::DISABLED)
                .draw(ui, area);
        });
        assert!(
            meter.contains(glyph),
            "a forced, erroring meter painted no error glyph: {meter:?}"
        );
        let (meter_ready, _) = painted(|ui, area| {
            Meter::new(id)
                .ratio(0.65)
                .value("65%")
                .status(Status::Ready)
                .state_override(StateFlags::DISABLED)
                .draw(ui, area);
        });
        assert!(
            !meter_ready.contains(glyph),
            "a forced, ready meter painted an error glyph: {meter_ready:?}"
        );

        // HintBar: `status_glyph` answered `None` for `DISABLED` alone, so
        // the two columns the message reserves for it stayed blank
        let layer = HintLayer {
            hints: vec![Hint {
                chord: Chord::key(KeyCode::Enter),
                label: "Open",
                priority: 0,
            }],
            badge: None,
            status: Some("Unable to load".into()),
            centered: false,
        };
        let (hints, _) = painted(|ui, area| {
            HintBar::new(id, &layer)
                .status(Status::Error)
                .state_override(StateFlags::DISABLED)
                .draw(ui, area);
        });
        assert!(
            hints.contains(glyph),
            "a forced, erroring hint bar painted no status glyph: {hints:?}"
        );
        let (hints_ready, _) = painted(|ui, area| {
            HintBar::new(id, &layer)
                .status(Status::Ready)
                .state_override(StateFlags::DISABLED)
                .draw(ui, area);
        });
        assert!(
            !hints_ready.contains(glyph),
            "a forced, ready hint bar painted a status glyph: {hints_ready:?}"
        );
    }
}
