//! `Button` (`COMPONENT_ARCHITECTURE.md` §17.0 A7, Appendix A 4A).

use core::fmt;

use ratatui_core::layout::Rect;

use super::{Overrides, SlotFn, cell_at, first_row, shift};
use crate::collection::Status;
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, Part};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Activated, Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, GlyphRole, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// The const-constructible command a button chord maps to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonCmd {
    /// Fire the button.
    Activate,
}

const BINDINGS: &[Binding<ButtonCmd>] = &[
    Binding {
        chord: Chord::key(KeyCode::Enter),
        cmd: ButtonCmd::Activate,
        label: "Activate",
        priority: 80,
        visible: true,
    },
    Binding {
        chord: Chord::key(KeyCode::Char(' ')),
        cmd: ButtonCmd::Activate,
        label: "Activate",
        priority: 80,
        visible: false,
    },
];

/// A one-row push button: ` label ` with a focus gutter, no box.
///
/// ## Construction
/// `Button::new(id, label)`. There is no alternate constructor; variants,
/// icons and toggles are builders.
///
/// ## Ownership
/// Stateless. The caller owns nothing; the runtime owns focus, hover, press
/// and the press flash. A toggle's on/off value is the caller's, passed
/// through `.checked(bool)` each frame.
///
/// ## Configuration
/// `.variant(Variant)` (default `Recipe.default_variant`), `.disabled(bool)`
/// (`false`), `.icon(GlyphRole)` (none), `.autofocus()` (off),
/// `.status(Status)` (`Ready`), `.checked(bool)` (none; a toggle marker),
/// `.patch`, `.patch_part`, `.slot`, `.state_override`.
///
/// ## Variants
/// `Family::BUTTON`: `PRIMARY`, `SECONDARY`, `SUBTLE`, `DANGER`, `TOGGLE`,
/// `QUIET`, `GHOST`; `DEFAULT` resolves to the recipe's `default_variant`
/// (`SECONDARY` look under Junie, `SECONDARY` under Paper).
///
/// ## States
/// Wears `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED`, `PRESSED` from the runtime;
/// derives `DISABLED` from `.disabled`, `BUSY`/`LOADING`/`ERROR` from
/// `.status`, `CHECKED` from `.checked(true)`. A disabled button drops
/// `HOVERED` and `PRESSED`; a busy button drops `PRESSED`.
///
/// ## Actions
/// `Response<Activated>` — `Activated` on Enter, Space or a click.
///
/// ## Focus
/// `Focusability::Focusable` (`Disabled` when disabled; `ClickOnly` never).
/// Does not swallow typing. `.autofocus()` requests focus on the first
/// `update` that runs before the button has ever been drawn.
///
/// ## Keyboard
/// Every state: `Enter` → `Activate`, `Space` → `Activate` (hidden hint).
///
/// ## Mouse
/// `PartRef::of(Part::CONTAINER)`: press/release/click; a click activates.
///
/// ## Layout
/// `measure` returns exactly `(gutter + [icon] + [marker] + label + pad, 1)`.
/// `draw` uses the first row of `area`, clipped to that width, and returns
/// the rect it painted; a `0×0` area registers nothing (R5).
///
/// ## Parts
/// `CONTAINER` (the whole button), `GUTTER` (the focus bar column),
/// `LABEL` (the text), `ICON` (the leading glyph), `MARKER` (the toggle
/// knob).
///
/// ## Overrides
/// `.patch`, `.patch_part` and `.slot` on any part; `CONTAINER` cannot be
/// replaced by a slot (its fill is the button).
///
/// ## Identity
/// One `Id` per instance; no items.
///
/// ## Testing
/// `ButtonCase` with `ACTIVATES | DISABLEABLE | FOCUSABLE`;
/// `render::components::button::{default, focused, hovered, pressed,
/// disabled, selected, editing, empty}`.
///
/// ## Invariants
/// Keyboard and mouse activation produce the identical `Activated`; a
/// disabled or busy button never activates; `state_override` renders a
/// reference state and registers nothing.
pub struct Button<'a> {
    id: Id,
    label: &'a str,
    variant: Variant,
    disabled: bool,
    icon: Option<GlyphRole>,
    autofocus: bool,
    status: Status,
    checked: Option<bool>,
    ov: Overrides<'a>,
}

impl fmt::Debug for Button<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Button")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("variant", &self.variant)
            .field("disabled", &self.disabled)
            .field("status", &self.status)
            .field("checked", &self.checked)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> Button<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::LABEL,
        Part::ICON,
        Part::MARKER,
    ];

    /// A button labelled `label`.
    pub const fn new(id: Id, label: &'a str) -> Self {
        Button {
            id,
            label,
            variant: Variant::DEFAULT,
            disabled: false,
            icon: None,
            autofocus: false,
            status: Status::Ready,
            checked: None,
            ov: Overrides::new(),
        }
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// Set the variant.
    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// Disable: registered in the ring but never reachable or activatable.
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    /// A leading glyph.
    #[must_use]
    pub const fn icon(mut self, g: GlyphRole) -> Self {
        self.icon = Some(g);
        self
    }

    /// Request focus on the first `update` before the button has been drawn.
    #[must_use]
    pub const fn autofocus(mut self) -> Self {
        self.autofocus = true;
        self
    }

    /// Data readiness (`Busy` disables activation and paints a spinner).
    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    /// A toggle marker: `true` paints the on knob, `false` a blank slot.
    #[must_use]
    pub const fn checked(mut self, on: bool) -> Self {
        self.checked = Some(on);
        self
    }

    /// An instance patch over every part (precedence 6).
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.patch(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    /// Replace one part's painting; layout, hit regions and focus stay.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Showcase / fixture use only (A11): render a forced state without
    /// owning it. Such a button registers no control and no ring entry.
    #[must_use]
    pub const fn state_override(mut self, s: StateFlags) -> Self {
        self.ov = self.ov.state_override(s);
        self
    }

    const fn busy(&self) -> bool {
        matches!(self.status, Status::Busy | Status::Loading)
    }

    const fn can_activate(&self) -> bool {
        !self.disabled && !self.busy()
    }

    /// The update phase.
    pub fn update(&self, cx: &mut Cx<'_>) -> Response<Activated> {
        if self.autofocus && cx.area(self.id).is_none() {
            cx.focus(self.id);
        }
        let mut r: Response<Activated> = Response::ignored();
        let can = self.can_activate();
        for it in cx.intents(self.id) {
            match it {
                Intent::Key(k) => {
                    if can && Binding::lookup(BINDINGS, &k).is_some() {
                        r = Response::action(Activated);
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click,
                    ..
                } if can => r = Response::action(Activated),
                Intent::Pointer { .. } if can && !r.activated() => r = Response::changed(),
                Intent::FocusIn { .. } | Intent::FocusOut { .. } if !r.activated() => {
                    r = Response::changed();
                }
                _ => {}
            }
        }
        r.for_id(self.id)
    }

    /// Columns the marker slot needs: the toggle knob or the busy spinner.
    fn marker_width(&self) -> u16 {
        if self.checked.is_some() || self.busy() {
            2
        } else {
            0
        }
    }

    /// The natural width: gutter, optional icon, optional marker, label, pad.
    fn natural_width(&self, ui: &Ui<'_>) -> u16 {
        let icon = self
            .icon
            .map_or(0, |g| width(ui.design().glyphs.get(g)).saturating_add(1));
        1u16.saturating_add(icon)
            .saturating_add(self.marker_width())
            .saturating_add(width(self.label))
            .saturating_add(1)
    }

    /// The draw phase.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass over gutter, icon, marker and label"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let w = self.natural_width(ui).min(area.width);
        let area = Rect {
            width: w,
            ..first_row(area)
        };
        if area.is_empty() {
            return area;
        }
        let forced = self.ov.is_forced();
        if !forced {
            let f = if self.disabled {
                Focusability::Disabled
            } else {
                Focusability::Focusable
            };
            ui.register_control(self.id, area, f);
        }
        let mut live = self.ov.flags(ui.state(self.id)) | self.status.flags();
        if self.disabled {
            live |= StateFlags::DISABLED;
            live = live.difference(StateFlags::HOVERED | StateFlags::PRESSED);
        }
        if self.busy() {
            live = live.difference(StateFlags::PRESSED);
        }
        if self.checked == Some(true) {
            live |= StateFlags::CHECKED;
        }
        let ov = self.ov;
        let style = |ui: &mut Ui<'_>, part: Part| {
            ov.style(ui, self.id, Family::BUTTON, self.variant, part, live)
        };
        let container = style(ui, Part::CONTAINER);
        ui.fill(area, container.style);

        // gutter: the focus bar when the recipe says so, else a blank cell
        let gutter_cell = cell_at(area, area.x);
        if let Some(f) = ov.slot_for(Part::GUTTER) {
            f(ui, gutter_cell);
        } else {
            let g = style(ui, Part::GUTTER);
            match g.glyph {
                Some(glyph) => {
                    ui.glyph(gutter_cell, glyph, g.style);
                }
                None => ui.fill(gutter_cell, g.style),
            }
        }

        // the text run: after the gutter, before the trailing pad
        let mut text = Rect {
            x: area.x.saturating_add(1),
            y: area.y,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        if let Some(g) = self.icon {
            let icon_cell = text;
            if let Some(f) = ov.slot_for(Part::ICON) {
                f(ui, icon_cell);
                text = shift(text, 2);
            } else {
                let is = style(ui, Part::ICON);
                let used = ui.glyph(icon_cell, g, is.style);
                text = shift(text, used.saturating_add(1));
            }
        }
        if self.busy() {
            let is = style(ui, Part::ICON);
            let frames = ui.design().motion.spinner_frames;
            let frame = frames.first().copied().unwrap_or("");
            let used = ui.paint_str(text, frame, is.style);
            text = shift(text, used.saturating_add(1));
        } else if let Some(on) = self.checked {
            let marker_cell = Rect {
                width: text.width.min(1),
                ..text
            };
            if let Some(f) = ov.slot_for(Part::MARKER) {
                f(ui, marker_cell);
            } else {
                let ms = style(ui, Part::MARKER);
                let glyph = ms.glyph.or(if on {
                    Some(GlyphRole::SwitchKnob)
                } else {
                    None
                });
                match glyph {
                    Some(g) => {
                        ui.glyph(marker_cell, g, ms.style);
                    }
                    None => ui.fill(marker_cell, ms.style),
                }
            }
            text = shift(text, 2);
        }
        if let Some(f) = ov.slot_for(Part::LABEL) {
            f(ui, text);
        } else {
            let ls = style(ui, Part::LABEL);
            if ls.glyph == Some(GlyphRole::PressLeft) {
                // the mono PRESSED rule: `[label]`
                let used = ui.glyph(text, GlyphRole::PressLeft, ls.style);
                let mut t = shift(text, used);
                let used = ui.paint_str(t, self.label, ls.style);
                t = shift(t, used);
                ui.glyph(t, GlyphRole::PressRight, ls.style);
            } else {
                ui.paint_str(text, self.label, ls.style);
            }
        }
        area
    }

    /// The natural size: one row, the label plus its chrome.
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        Size::exact(self.natural_width(ui), 1).fit(c)
    }
}

impl Bindings for Button<'_> {
    type Cmd = ButtonCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<ButtonCmd>] {
        BINDINGS
    }
}
