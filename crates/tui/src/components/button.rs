//! `Button` (`COMPONENT_ARCHITECTURE.md` §17.0 A7, Appendix A 4A).

use core::fmt;

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn, cell_at, first_row, paint_pressed_bracket, shift};
use crate::action::ActionKey;
use crate::collection::Status;
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, Part};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Activated, Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// The const-constructible command a button chord maps to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonCmd {
    /// Fire the button.
    Activate,
}

const BINDINGS: &[Binding<ButtonCmd>] = &[
    Binding {
        action: ActionKey::custom("button.activate.enter"),
        chord: Some(Chord::key(KeyCode::Enter)),
        cmd: ButtonCmd::Activate,
        label: "Activate",
        priority: 80,
        visible: true,
    },
    Binding {
        action: ActionKey::custom("button.activate.space"),
        chord: Some(Chord::key(KeyCode::Char(' '))),
        cmd: ButtonCmd::Activate,
        label: "Activate",
        priority: 80,
        visible: false,
    },
];

/// The `.autofocus()` one-shot, held in the runtime-owned derived cache.
///
/// Nothing semantic lives here (§5 R8): losing the entry costs one repeated
/// focus request when the instance reappears after a whole frame away, which
/// is the same thing a freshly built instance does.
#[derive(Default)]
struct Autofocus {
    spent: bool,
}

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
/// `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::BUTTON`: `PRIMARY`, `SECONDARY`, `SUBTLE`, `DANGER`, `TOGGLE`,
/// `QUIET`, `GHOST`; `DEFAULT` resolves to the recipe's `default_variant`
/// (`SECONDARY` look under Junie, `SECONDARY` under Paper).
///
/// ## States
/// Wears `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED`, `PRESSED` from the runtime;
/// derives `DISABLED` from `.disabled`, `BUSY`/`LOADING`/`ERROR` from
/// `.status`, `CHECKED | SELECTED` from `.checked(true)`. A disabled button drops
/// `HOVERED` and `PRESSED`; a busy button drops `PRESSED`.
///
/// ## Actions
/// `Response<Activated>` — `Activated` on Enter, Space or a click.
///
/// ## Focus
/// `Focusability::Focusable` (`Disabled` when disabled; `ClickOnly` never).
/// Does not swallow typing. `.autofocus()` requests focus on the **first**
/// `update` of the instance and never again: the one-shot is held in the
/// runtime-owned derived cache, so a button that stops drawing — scrolled
/// out of view, or made inert by a modal opening over it — cannot request
/// focus a second time and take it back from the layer above. A button that
/// is disabled on that first `update` spends the one-shot without
/// requesting anything: `autofocus` is evaluated when the instance appears,
/// not re-armed when `.disabled(false)` returns later. Should the instance
/// leave the tree for a whole frame, its cache entry is dropped and the
/// one-shot arms again with the new instance.
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
/// `LABEL` (the text), `ICON` (the leading/readiness glyph), `MARKER` (the toggle
/// knob).
///
/// ## Overrides
/// `.patch` and `.patch_part` on any part. `.slot` on exactly `GUTTER`,
/// `ICON`, `MARKER` and `LABEL` — the four parts the button paints into a
/// rect it reserves. `CONTAINER` is not slot-addressable: its fill *is* the
/// button. The `ICON` slot replaces the readiness symbol as well as `.icon(g)`,
/// because one `Part` may not have two answers in one component (§45.4).
///
/// ## Identity
/// One `Id` per instance; no items.
///
/// ## Testing
/// `ButtonCase` with `ACTIVATES | DISABLEABLE | FOCUSABLE | REPORTS_STATUS |
/// SELECTS`;
/// `render::components::button::{default, focused, hovered, pressed,
/// disabled, selected, editing, empty}`.
///
/// ## Invariants
/// Keyboard and mouse activation produce the identical `Activated`; a
/// disabled or busy button never activates.
pub struct Button<'a> {
    id: Id,
    label: &'a str,
    variant: Variant,
    disabled: bool,
    icon: Option<GlyphRole>,
    autofocus: bool,
    status: Status,
    checked: Option<bool>,
    ov: PartStyle<'a>,
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
            ov: PartStyle::new(),
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

    /// Request focus once, on this instance's first `update`.
    ///
    /// A disabled button spends the one-shot without requesting focus.
    #[must_use]
    pub const fn autofocus(mut self) -> Self {
        self.autofocus = true;
        self
    }

    /// Data readiness (`Busy` disables activation; non-ready states paint `ICON`).
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
        self.ov = self.ov.global(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    /// Replace one part's painting; layout, hit regions and focus stay.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    const fn busy(&self) -> bool {
        matches!(self.status, Status::Busy | Status::Loading)
    }

    const fn can_activate(&self) -> bool {
        !self.disabled && !self.busy()
    }

    const fn with_inherited_disabled(&self, inherited: bool) -> Self {
        Button {
            id: self.id,
            label: self.label,
            variant: self.variant,
            disabled: self.disabled || inherited,
            icon: self.icon,
            autofocus: self.autofocus,
            status: self.status,
            checked: self.checked,
            ov: self.ov,
        }
    }

    /// The update phase.
    pub fn update(&self, cx: &mut Cx<'_>) -> Response<Activated> {
        self.autofocus_once(cx);
        let mut r: Response<Activated> = Response::ignored();
        let can = self.can_activate();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) => {
                    if can && Binding::command(BINDINGS, action).is_some() {
                        r = Response::action(Activated);
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    ..
                } if can => r = Response::action(Activated),
                // A `Move` is hover bookkeeping the runtime already owns: it
                // repaints exactly on a hover transition (§3.3 step 3), so a
                // component that answered `changed()` here repainted on every
                // pointer motion inside its own area for no visual change.
                Intent::Pointer {
                    phase: Phase::Move, ..
                } => {}
                Intent::Pointer { .. } if can && !r.activated() => r = Response::changed(),
                _ => {}
            }
        }
        r.for_id(self.id)
    }

    /// The `.autofocus()` one-shot.
    ///
    /// The latch lives in the runtime-owned derived cache, which is keyed by
    /// this button's id and dropped only when the instance misses a whole
    /// frame. `cx.area(self.id).is_none()` was the previous test and is not
    /// a first-update test at all: it is true again on every frame the
    /// button does not draw, so a button that a modal made inert re-requested
    /// focus every frame and pulled it out of the layer above.
    fn autofocus_once(&self, cx: &mut Cx<'_>) {
        if !self.autofocus {
            return;
        }
        let latch = cx.cache::<Autofocus>(self.id);
        if latch.spent {
            return;
        }
        latch.spent = true;
        if !self.disabled {
            cx.focus(self.id);
        }
    }

    pub(crate) fn update_in_form(
        &self,
        cx: &mut Cx<'_>,
        inherited_disabled: bool,
    ) -> Response<Activated> {
        self.with_inherited_disabled(inherited_disabled).update(cx)
    }

    /// Columns the readiness lane needs: one symbol plus its gap.
    const fn readiness_width(&self) -> u16 {
        if self.icon.is_some() || !matches!(self.status, Status::Ready) {
            2
        } else {
            0
        }
    }

    /// Columns the independent toggle marker needs.
    fn marker_width(&self) -> u16 {
        if self.checked.is_some() { 2 } else { 0 }
    }

    /// The natural width: gutter, optional icon, optional marker, label, pad.
    fn natural_width(&self, _ui: &Ui<'_>) -> u16 {
        1u16.saturating_add(self.readiness_width())
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
        if !ui.is_inert() {
            let f = if self.disabled {
                Focusability::Disabled
            } else {
                Focusability::Focusable
            };
            ui.register_control(self.id, area, f);
        }
        // runtime: the frame's own focus/hover/press; derived: `.status`,
        // `.disabled` and `.checked`. The subtractions stay after the union —
        // they remove states the props forbid, whoever supplied them.
        let mut derived = self.status.flags();
        if self.disabled {
            derived |= StateFlags::DISABLED;
        }
        if self.checked == Some(true) {
            derived |= StateFlags::CHECKED | StateFlags::SELECTED;
        }
        let mut live = PartStyle::flags(ui.state(self.id), derived);
        if self.checked != Some(true) {
            live = live.difference(StateFlags::SELECTED);
        }
        if self.disabled {
            live = live.difference(StateFlags::HOVERED | StateFlags::PRESSED);
        }
        if self.busy() {
            live = live.difference(StateFlags::PRESSED);
        }
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, BINDINGS);
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
                Slot::Set(glyph) => {
                    ui.glyph(gutter_cell, glyph, g.style);
                }
                Slot::Inherit | Slot::Clear => ui.fill(gutter_cell, g.style),
            }
        }

        // the text run: after the gutter, before the trailing pad
        let mut text = Rect {
            x: area.x.saturating_add(1),
            y: area.y,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        if self.readiness_width() > 0 {
            let icon_cell = Rect {
                width: text.width.min(1),
                ..text
            };
            if let Some(f) = ov.slot_for(Part::ICON) {
                f(ui, icon_cell);
            } else {
                let is = style(ui, Part::ICON);
                match self.status {
                    Status::Busy | Status::Loading => {
                        let frames = ui.design().motion.spinner_frames;
                        let frame = frames.first().copied().unwrap_or("");
                        ui.paint_str(icon_cell, frame, is.style);
                    }
                    Status::Error => match is.glyph {
                        Slot::Set(glyph) => {
                            ui.glyph(icon_cell, glyph, is.style);
                        }
                        Slot::Inherit => {
                            ui.glyph(icon_cell, GlyphRole::Error, is.style);
                        }
                        Slot::Clear => ui.fill(icon_cell, is.style),
                    },
                    Status::Ready => {
                        if let Some(glyph) = self.icon {
                            ui.glyph(icon_cell, glyph, is.style);
                        }
                    }
                }
            }
            text = shift(text, 2);
        }
        if let Some(on) = self.checked {
            let marker_cell = Rect {
                width: text.width.min(1),
                ..text
            };
            if let Some(f) = ov.slot_for(Part::MARKER) {
                f(ui, marker_cell);
            } else {
                let ms = style(ui, Part::MARKER);
                let glyph = match ms.glyph {
                    Slot::Set(g) => Some(g),
                    Slot::Inherit if on => Some(GlyphRole::SwitchKnob),
                    Slot::Inherit | Slot::Clear => None,
                };
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
            if matches!(ls.glyph, Slot::Set(GlyphRole::PressLeft)) {
                ui.paint_str(text, self.label, ls.style);
                if area.width >= 2 {
                    paint_pressed_bracket(
                        ui,
                        gutter_cell,
                        cell_at(area, area.right().saturating_sub(1)),
                        ls.style,
                    );
                }
            } else {
                ui.paint_str(text, self.label, ls.style);
            }
        }
        area
    }

    pub(crate) fn draw_in_form(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        inherited_disabled: bool,
    ) -> Rect {
        self.with_inherited_disabled(inherited_disabled)
            .draw(ui, area)
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

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::Modifier;

    use super::*;
    use crate::event::{Input, MouseKind};
    use crate::response::Invalidate;
    use crate::runtime::stub::{Stub, key, mouse};
    use crate::runtime::{App, Runtime, UpdateCause};
    use crate::theme::{ColorLevel, Theme};

    const BUTTON: Id = Id::root("button.tests");
    const AREA: Rect = Rect {
        x: 0,
        y: 0,
        width: 12,
        height: 1,
    };

    fn row_text(buf: &Buffer, width: u16) -> String {
        let mut text = String::new();
        for x in 0..width {
            if let Some(cell) = buf.cell(Position::new(x, 0)) {
                text.push_str(cell.symbol());
            }
        }
        text
    }

    fn draw_status(status: Option<Status>) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            let mut button = Button::new(BUTTON, "Go");
            if let Some(status) = status {
                button = button.status(status);
            }
            button.draw(ui, area);
        });
        buffer
    }

    fn draw_checked(checked: bool) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            Button::new(BUTTON, "Go").checked(checked).draw(ui, area);
        });
        buffer
    }

    #[test]
    fn checked_painting_comes_only_from_the_controlled_prop() {
        assert_ne!(draw_checked(true), draw_checked(false));
    }

    #[test]
    fn mono_pressed_does_not_truncate_the_label() {
        const LABEL: &str = "Full width";
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(ColorLevel::Mono));
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            ui.reference(
                Some(crate::ReferenceTarget::new(
                    BUTTON,
                    crate::ReferenceState::PRESSED | crate::ReferenceState::FOCUSED,
                )),
                |ui| Button::new(BUTTON, LABEL).draw(ui, area),
            );
        });

        assert_eq!(row_text(&buffer, AREA.width), "[Full width]");
    }

    #[test]
    fn pressed_button_stays_inside_narrow_clip_bounds() {
        const LABEL: &str = "Full width";
        let screen = Rect::new(0, 0, 16, 1);
        let component_x = 4;

        for width in [0, 1, 2] {
            let area = Rect::new(component_x, 0, width, 1);
            let mut runtime =
                Runtime::new(Stub::default(), Theme::junie().downgrade(ColorLevel::Mono));
            let mut buffer = Buffer::empty(screen);
            let before = buffer.clone();
            runtime.draw_scene(screen, &mut buffer, |ui, _| {
                ui.reference(
                    Some(crate::ReferenceTarget::new(
                        BUTTON,
                        crate::ReferenceState::PRESSED | crate::ReferenceState::FOCUSED,
                    )),
                    |ui| Button::new(BUTTON, LABEL).draw(ui, area),
                );
            });

            for x in 0..screen.width {
                if x < area.x || x >= area.right() {
                    assert_eq!(
                        buffer.cell(Position::new(x, 0)),
                        before.cell(Position::new(x, 0)),
                        "pressed button wrote outside {area:?} at x={x}"
                    );
                }
            }
        }
    }

    #[test]
    fn readiness_owns_one_leading_lane_and_keeps_the_toggle_marker() {
        assert!(Button::PARTS.contains(&Part::ICON));
        assert_eq!(draw_status(None), draw_status(Some(Status::Ready)));
        let error = draw_status(Some(Status::Error));
        assert_eq!(
            error
                .cell(Position::new(1, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::Error))
        );
        let theme = Theme::junie();
        let seen = Cell::new(None);
        let used = Cell::new(Rect::default());
        let replacement = |_ui: &mut Ui<'_>, area: Rect| seen.set(Some(area));
        let patch = [(Part::ICON, StylePatch::new().add(Modifier::UNDERLINED))];
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            used.set(
                Button::new(BUTTON, "Go")
                    .status(Status::Error)
                    .checked(true)
                    .patch_part(&patch)
                    .slot(Part::ICON, &replacement)
                    .draw(ui, area),
            );
        });

        assert_eq!(seen.get(), Some(Rect::new(1, 0, 1, 1)));
        assert_eq!(used.get().width, 8);
        assert!(row_text(&buffer, AREA.width).contains("Go"));

        let mut patched = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut patched, |ui, area| {
            Button::new(BUTTON, "Go")
                .status(Status::Busy)
                .patch_part(&patch)
                .draw(ui, area);
        });
        assert!(
            patched
                .cell(Position::new(1, 0))
                .is_some_and(|cell| cell.modifier.contains(Modifier::UNDERLINED))
        );
    }

    const OTHER: Id = Id::root("button.tests.other");
    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 20,
        height: 4,
    };

    /// A page whose plain control is registered **first**, so the first
    /// draw's focus reconciliation lands on `OTHER` and every later move to
    /// `BUTTON` is the button's own doing.
    #[derive(Default)]
    struct AutofocusApp {
        disabled: bool,
    }

    impl AutofocusApp {
        /// The one constructor both phases use (§13.1 "props are built once").
        fn button(&self) -> Button<'static> {
            Button::new(BUTTON, "Go")
                .disabled(self.disabled)
                .autofocus()
        }
    }

    impl App for AutofocusApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                return Response::ignored();
            }
            self.button().update(cx).erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_control(OTHER, Rect::new(0, 2, 8, 1), Focusability::Focusable);
            self.button().draw(ui, AREA);
        }
    }

    /// S1: `.autofocus()` is a one-shot. The old test — "the button has no
    /// area yet" — is true again on **every** frame the button does not
    /// draw, so the request came back and took focus off whatever the user
    /// had moved to.
    #[test]
    fn autofocus_fires_once_and_never_takes_focus_back() {
        let mut runtime = Runtime::new(AutofocusApp::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.focus(), Some(OTHER));

        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.focus(), Some(BUTTON), "autofocus must fire once");

        let _ = runtime.handle(key(KeyCode::Tab));
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.focus(), Some(OTHER));

        for _ in 0..3 {
            let _ = runtime.handle(Input::Tick);
            runtime.draw_buffer(SCREEN, &mut buffer);
        }
        assert_eq!(
            runtime.focus(),
            Some(OTHER),
            "a spent autofocus must never request focus again"
        );
    }

    /// S1: a disabled button is in the ring and unreachable, so its
    /// `.autofocus()` spends its one-shot without asking for anything — and
    /// the runtime refuses the target anyway.
    #[test]
    fn a_disabled_autofocus_button_never_takes_focus() {
        let mut runtime = Runtime::new(AutofocusApp { disabled: true }, Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.focus(), Some(OTHER));
        assert!(runtime.ring().is_registered(BUTTON));
        assert!(!runtime.ring().contains(BUTTON));

        for _ in 0..3 {
            let _ = runtime.handle(Input::Tick);
            runtime.draw_buffer(SCREEN, &mut buffer);
        }

        assert_eq!(
            runtime.focus(),
            Some(OTHER),
            "a disabled button must not be focused by autofocus"
        );
    }

    struct HoverApp;

    impl App for HoverApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            Button::new(BUTTON, "Go").update(cx).erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            Button::new(BUTTON, "Go").draw(ui, AREA);
        }
    }

    /// S4: hover is runtime state and the runtime repaints exactly on a
    /// hover transition. A `Move` inside the same part must therefore cost
    /// no repaint, while entering, leaving and pressing still do.
    #[test]
    fn a_pointer_move_without_a_hover_transition_does_not_repaint() {
        let mut runtime = Runtime::new(HoverApp, Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer);

        let entered = runtime.handle(mouse(MouseKind::Move, 1, 0));
        runtime.draw_buffer(SCREEN, &mut buffer);
        let stayed = runtime.handle(mouse(MouseKind::Move, 2, 0));
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.hover(), Some(BUTTON));
        let pressed = runtime.handle(mouse(MouseKind::Down, 2, 0));
        runtime.draw_buffer(SCREEN, &mut buffer);
        let left = runtime.handle(mouse(MouseKind::Up, 2, 0));
        runtime.draw_buffer(SCREEN, &mut buffer);

        assert_eq!(entered.invalidate(), Invalidate::Paint);
        assert_eq!(
            stayed.invalidate(),
            Invalidate::None,
            "moving inside one part repainted with nothing to show"
        );
        assert_eq!(pressed.invalidate(), Invalidate::Paint);
        assert_eq!(left.invalidate(), Invalidate::Paint);
    }
}
