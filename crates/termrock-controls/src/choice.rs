//! `Checkbox`, `Toggle` and `RadioGroup` (`COMPONENT_ARCHITECTURE.md` §15,
//! §17.0 A7/A10, §18.2, §20.10 item 3, Appendix A 4B).
//!
//! The three controls share one keymap (`Space` / `Enter` commit) and one
//! marker vocabulary: the glyph is a [`GlyphRole`] the component names and
//! the theme binds, because a two-state affordance's *off* half cannot be
//! expressed by a `StateRule` — a rule binds one glyph to a state, and "not
//! checked" is the absence of a state, not a state of its own.

use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::{Position, Rect};

use super::form::InheritedFormState;
use super::{Acc, PartStyle, SlotFn, cell_at, first_row};
use crate::action::ActionKey;
use crate::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, Reconcile, Reconciliation, RowFn, RowUi, StepDir,
    index_of, key_at,
};
use crate::event::{Axis, Chord, KeyCode};
use crate::field_control::FieldControl;
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Activated, Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, Ui};

/// What a radio group reports; the cursor moving is **not** an action —
/// cursor and value are separate (§15, §20.10 item 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RadioGroupAction {
    /// The cursor option was committed as the value.
    Chose(ItemKey),
}

/// The const-constructible commands of the choice keymap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChoiceCmd {
    /// Commit the cursor option / flip the flag.
    Choose,
    /// Cursor to the previous option.
    Prev,
    /// Cursor to the next option.
    Next,
    /// Cursor to the first option.
    First,
    /// Cursor to the last option.
    Last,
}

const fn b(chord: Chord, cmd: ChoiceCmd, label: &'static str, visible: bool) -> Binding<ChoiceCmd> {
    Binding {
        action: ActionKey::custom(label),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 70 } else { 10 },
        visible,
    }
}

/// `Checkbox` / `Toggle`: one commit chord, two spellings.
const FLAG: &[Binding<ChoiceCmd>] = &[
    b(
        Chord::key(KeyCode::Char(' ')),
        ChoiceCmd::Choose,
        "Toggle",
        true,
    ),
    b(
        Chord::key(KeyCode::Enter),
        ChoiceCmd::Choose,
        "Toggle (Enter)",
        false,
    ),
];

/// `RadioGroup`: arrows move the cursor, `Space` / `Enter` commit it.
const RADIO: &[Binding<ChoiceCmd>] = &[
    b(
        Chord::key(KeyCode::Char(' ')),
        ChoiceCmd::Choose,
        "Choose",
        true,
    ),
    b(
        Chord::key(KeyCode::Enter),
        ChoiceCmd::Choose,
        "Choose (Enter)",
        false,
    ),
    b(Chord::key(KeyCode::Up), ChoiceCmd::Prev, "Up", true),
    b(Chord::key(KeyCode::Down), ChoiceCmd::Next, "Down", true),
    b(
        Chord::key(KeyCode::Char('k')),
        ChoiceCmd::Prev,
        "Up (K)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('j')),
        ChoiceCmd::Next,
        "Down (J)",
        false,
    ),
    b(Chord::key(KeyCode::Home), ChoiceCmd::First, "First", false),
    b(Chord::key(KeyCode::End), ChoiceCmd::Last, "Last", false),
];

/// `RadioGroup` laid out horizontally: Left/Right move the cursor,
/// `Space` / `Enter` commit it (W05-04).
const RADIO_H: &[Binding<ChoiceCmd>] = &[
    b(
        Chord::key(KeyCode::Char(' ')),
        ChoiceCmd::Choose,
        "Choose",
        true,
    ),
    b(
        Chord::key(KeyCode::Enter),
        ChoiceCmd::Choose,
        "Choose (Enter)",
        false,
    ),
    b(Chord::key(KeyCode::Left), ChoiceCmd::Prev, "Left", true),
    b(Chord::key(KeyCode::Right), ChoiceCmd::Next, "Right", true),
    b(
        Chord::key(KeyCode::Char('h')),
        ChoiceCmd::Prev,
        "Left (H)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('l')),
        ChoiceCmd::Next,
        "Right (L)",
        false,
    ),
    b(Chord::key(KeyCode::Home), ChoiceCmd::First, "First", false),
    b(Chord::key(KeyCode::End), ChoiceCmd::Last, "Last", false),
];

/// One flag row's chrome — gutter, marker glyph, label, trailing word.
///
/// Shared by [`Checkbox`] and [`Toggle`]: the only difference is the marker,
/// which each control supplies as a closure over its own glyph roles.
struct FlagRow<'r> {
    id: Id,
    ov: PartStyle<'r>,
    label: &'r str,
    marker_w: u16,
    trailing: Option<&'r str>,
}

impl FlagRow<'_> {
    /// Paint the row and return it.
    fn draw(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        live: StateFlags,
        marker: &dyn Fn(&mut Ui<'_>, Rect, crate::theme::PaintStyle),
    ) -> Rect {
        let (id, ov, label, marker_w, trailing) =
            (self.id, self.ov, self.label, self.marker_w, self.trailing);
        let style = |ui: &mut Ui<'_>, part: Part| {
            ov.style(ui, id, Family::CHOICE, Variant::DEFAULT, part, live)
        };
        let container = style(ui, Part::CONTAINER);
        ui.fill(area, container.style);
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
        let marker_cell = Rect {
            x: area.x.saturating_add(1),
            y: area.y,
            width: marker_w.min(area.width.saturating_sub(1)),
            height: 1,
        };
        if let Some(f) = ov.slot_for(Part::MARKER) {
            f(ui, marker_cell);
        } else {
            let ms = style(ui, Part::MARKER);
            marker(ui, marker_cell, ms.style);
        }
        let text = Rect {
            x: area
                .x
                .saturating_add(1)
                .saturating_add(marker_w)
                .saturating_add(1),
            y: area.y,
            width: area.width.saturating_sub(2).saturating_sub(marker_w),
            height: 1,
        };
        if let Some(f) = ov.slot_for(Part::LABEL) {
            f(ui, text);
        } else {
            let ls = style(ui, Part::LABEL);
            let used = if matches!(ls.glyph, Slot::Set(GlyphRole::PressLeft)) {
                // §11.4's mono `PRESSED` affordance: `[label]`
                let l = ui.glyph(text, GlyphRole::PressLeft, ls.style);
                let mut t = super::shift(text, l);
                let w = ui.paint_str(t, label, ls.style);
                t = super::shift(t, w);
                let r = ui.glyph(t, GlyphRole::PressRight, ls.style);
                l.saturating_add(w).saturating_add(r)
            } else {
                ui.paint_str(text, label, ls.style)
            };
            if let Some(s) = trailing {
                let rest = super::shift(text, used.saturating_add(1));
                let hs = style(ui, Part::META);
                ui.paint_str(rest, s, hs.style);
            }
        }
        area
    }
}

/// A one-row checkbox: `[✓]` / `[ ]`, a label, and the caller's `bool`.
///
/// ## Construction
/// `Checkbox::new(id, label)`. The controlled flag is passed per phase:
/// `&mut bool` to `update`, `.checked(bool)` for `draw`.
///
/// ## Ownership
/// Stateless (`State = ()`): the flag is the caller's, the runtime owns
/// focus, hover and press.
///
/// ## Configuration
/// `.checked(bool)` (draw; `false`), `.disabled(bool)`, `.read_only(bool)`,
/// `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::CHOICE`, `DEFAULT` only.
///
/// ## States
/// `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED`, `PRESSED` from the runtime;
/// `CHECKED | SELECTED` from the flag; `READ_ONLY`, `DISABLED` from the props.
///
/// ## Actions
/// [`Activated`] — the flag was flipped through the `&mut bool` (§6.1: the
/// activation action of a button-like control).
///
/// ## Focus
/// One `Focusable` stop (`FocusableReadOnly` / `Disabled`); does not
/// swallow typing.
///
/// ## Keyboard
/// `Space` (visible) and `Enter` toggle.
///
/// ## Mouse
/// `PartRef::of(Part::CONTAINER)`: a click toggles.
///
/// ## Layout
/// One row: gutter, a three-column marker, one space, the label. `measure`
/// is the natural width by one row; `draw` returns the row; `0×0` registers
/// nothing (R5).
///
/// ## Parts
/// `CONTAINER` (the row fill), `GUTTER` (the focus bar), `MARKER` (the box),
/// `LABEL`.
///
/// ## Overrides
/// `.patch`, `.patch_part`, `.slot` on `GUTTER`, `MARKER` and `LABEL`.
///
/// ## Identity
/// One `Id`; no items.
///
/// ## Testing
/// `CheckboxCase` with `ACTIVATES | FOCUSABLE | DISABLEABLE`;
/// `render::components::checkbox::*`.
///
/// ## Invariants
/// `draw` never writes the flag (it takes `&self` and a `bool` prop); the
/// marker is [`GlyphRole::CheckboxOn`] / [`GlyphRole::CheckboxOff`], so the
/// off state is a glyph and survives `ColorLevel::Mono`.
pub struct Checkbox<'a> {
    id: Id,
    label: &'a str,
    checked: bool,
    read_only: bool,
    disabled: bool,
    ov: PartStyle<'a>,
}

impl fmt::Debug for Checkbox<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Checkbox")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("checked", &self.checked)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .finish_non_exhaustive()
    }
}

impl<'a> Checkbox<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::MARKER, Part::LABEL];

    /// Columns the marker occupies.
    const MARKER_W: u16 = 3;

    /// A checkbox.
    pub const fn new(id: Id, label: &'a str) -> Self {
        Checkbox {
            id,
            label,
            checked: false,
            read_only: false,
            disabled: false,
            ov: PartStyle::new(),
        }
    }

    /// The controlled flag, for `draw`.
    #[must_use]
    pub const fn checked(mut self, yes: bool) -> Self {
        self.checked = yes;
        self
    }

    /// Read-only: stays in the ring, never toggles.
    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    /// Disabled: registered, never reachable.
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    /// An instance patch over every part.
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

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Showcase / fixture use only (A11).
    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    const fn with_inherited_disabled(&self, inherited: bool) -> Self {
        Checkbox {
            id: self.id,
            label: self.label,
            checked: self.checked,
            read_only: self.read_only,
            disabled: self.disabled || inherited,
            ov: self.ov,
        }
    }

    /// The update phase: `Space` / `Enter` / a click flip `value`.
    pub fn update(&self, cx: &mut Cx<'_>, value: &mut bool) -> Response<Activated> {
        let mut acc = Acc::<Activated>::new();
        let can = self.editable();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) if can => {
                    if Binding::command(FLAG, action).is_some() {
                        *value = !*value;
                        acc.action(Activated);
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    ..
                } if can => {
                    *value = !*value;
                    acc.action(Activated);
                }
                Intent::Pointer { .. } => acc.consumed(),
                _ => {}
            }
        }
        acc.finish(self.id)
    }

    pub fn update_in_form(
        &self,
        cx: &mut Cx<'_>,
        value: &mut bool,
        inherited_disabled: bool,
    ) -> Response<Activated> {
        self.with_inherited_disabled(inherited_disabled)
            .update(cx, value)
    }

    /// The natural width: gutter, marker, space, label.
    fn natural_width(&self) -> u16 {
        Self::MARKER_W
            .saturating_add(2)
            .saturating_add(width(self.label))
    }

    /// The draw phase.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }
        // runtime: the frame's own focus/hover/press; derived: the box's
        // `.checked`, `.read_only` and `.disabled` props
        let mut derived = StateFlags::empty();
        if self.checked {
            derived |= StateFlags::CHECKED | StateFlags::SELECTED;
        }
        if self.read_only {
            derived |= StateFlags::READ_ONLY;
        }
        if self.disabled {
            derived |= StateFlags::DISABLED;
        }
        let mut live = PartStyle::flags(crate::ui::FrameRead::state(ui, self.id), derived);
        if !self.checked {
            live = live.difference(StateFlags::SELECTED);
        }
        if self.disabled {
            live = live.difference(StateFlags::HOVERED | StateFlags::PRESSED);
        }
        if !ui.is_inert() {
            let f = if self.disabled {
                Focusability::Disabled
            } else if self.read_only {
                Focusability::FocusableReadOnly
            } else {
                Focusability::Focusable
            };
            ui.register_control(self.id, area, f);
            ui.publish_bindings(self.id, live, FLAG);
        }
        let on = live.contains(StateFlags::CHECKED);
        // W03-01: under four columns the box collapses to its single-cell
        // state mark (reference `Checkbox::render`: `area.width < 4`).
        let compact = area.width < 4;
        FlagRow {
            id: self.id,
            ov: self.ov,
            label: self.label,
            marker_w: Self::MARKER_W,
            trailing: None,
        }
        .draw(
            ui,
            area,
            live,
            &move |ui: &mut Ui<'_>, cell: Rect, style| {
                let g = match (compact, on) {
                    (true, true) => GlyphRole::Checked,
                    (true, false) => GlyphRole::CheckboxEmpty,
                    (false, true) => GlyphRole::CheckboxOn,
                    (false, false) => GlyphRole::CheckboxOff,
                };
                ui.glyph(cell, g, style);
            },
        )
    }

    pub fn draw_in_form(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        value: bool,
        inherited_disabled: bool,
    ) -> Rect {
        self.with_inherited_disabled(inherited_disabled)
            .checked(value)
            .draw(ui, area)
    }

    /// One row, the marker plus the label.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size::exact(self.natural_width(), 1).fit(c)
    }
}

impl Bindings for Checkbox<'_> {
    type Cmd = ChoiceCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<ChoiceCmd>] {
        FLAG
    }
}

impl FieldControl for Checkbox<'_> {
    type State = ();

    fn id(&self) -> Id {
        self.id
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect, _st: &()) -> Rect {
        Checkbox::draw(self, ui, area)
    }

    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        Checkbox::measure(self, ui, c)
    }
}

/// A one-row switch: a knob on a two-cell track, a label and an `on` / `off`
/// word.
///
/// ## Construction
/// `Toggle::new(id, label)`. The controlled flag is passed per phase:
/// `&mut bool` to `update`, `.on(bool)` for `draw`.
///
/// ## Ownership
/// Stateless (`State = ()`): the flag is the caller's.
///
/// ## Configuration
/// `.on(bool)` (draw; `false`), `.disabled(bool)`, `.read_only(bool)`,
/// `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::CHOICE`, `DEFAULT` only.
///
/// ## States
/// `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED`, `PRESSED` from the runtime;
/// `CHECKED | SELECTED` from the flag; `READ_ONLY`, `DISABLED` from the props.
///
/// ## Actions
/// [`Activated`] — the flag was flipped through the `&mut bool`.
///
/// ## Focus
/// One `Focusable` stop (`FocusableReadOnly` / `Disabled`); does not
/// swallow typing.
///
/// ## Keyboard
/// `Space` (visible) and `Enter` toggle.
///
/// ## Mouse
/// `PartRef::of(Part::CONTAINER)`: a click toggles.
///
/// ## Layout
/// One row: gutter, a three-column switch, one space, the label, then the
/// `on` / `off` word when the row is wide enough. `measure` is the natural
/// width by one row; `0×0` registers nothing (R5).
///
/// ## Parts
/// `CONTAINER`, `GUTTER`, `MARKER` (the switch), `LABEL`, `META` (the
/// `on` / `off` word).
///
/// ## Overrides
/// `.patch`, `.patch_part`, `.slot` on `GUTTER`, `MARKER` and `LABEL`.
///
/// ## Identity
/// One `Id`; no items.
///
/// ## Testing
/// `ToggleCase` with `ACTIVATES | FOCUSABLE | DISABLEABLE`;
/// `render::components::toggle::*`.
///
/// ## Invariants
/// `draw` never writes the flag; the switch is [`GlyphRole::SwitchKnob`] on
/// a [`GlyphRole::RuleQuiet`] track and the state word is text, so both
/// halves survive `ColorLevel::Mono`.
pub struct Toggle<'a> {
    id: Id,
    label: &'a str,
    on: bool,
    read_only: bool,
    disabled: bool,
    ov: PartStyle<'a>,
}

impl fmt::Debug for Toggle<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Toggle")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("on", &self.on)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .finish_non_exhaustive()
    }
}

impl<'a> Toggle<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::MARKER,
        Part::LABEL,
        Part::META,
    ];

    /// Columns the switch occupies.
    const MARKER_W: u16 = 3;

    /// A toggle.
    pub const fn new(id: Id, label: &'a str) -> Self {
        Toggle {
            id,
            label,
            on: false,
            read_only: false,
            disabled: false,
            ov: PartStyle::new(),
        }
    }

    /// The controlled flag, for `draw`.
    #[must_use]
    pub const fn on(mut self, yes: bool) -> Self {
        self.on = yes;
        self
    }

    /// Read-only: stays in the ring, never toggles.
    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    /// Disabled: registered, never reachable.
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    /// An instance patch over every part.
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

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Showcase / fixture use only (A11).
    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    const fn with_inherited_disabled(&self, inherited: bool) -> Self {
        Toggle {
            id: self.id,
            label: self.label,
            on: self.on,
            read_only: self.read_only,
            disabled: self.disabled || inherited,
            ov: self.ov,
        }
    }

    /// The update phase: `Space` / `Enter` / a click flip `value`.
    pub fn update(&self, cx: &mut Cx<'_>, value: &mut bool) -> Response<Activated> {
        let mut acc = Acc::<Activated>::new();
        let can = self.editable();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) if can => {
                    if Binding::command(FLAG, action).is_some() {
                        *value = !*value;
                        acc.action(Activated);
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    ..
                } if can => {
                    *value = !*value;
                    acc.action(Activated);
                }
                Intent::Pointer { .. } => acc.consumed(),
                _ => {}
            }
        }
        acc.finish(self.id)
    }

    pub fn update_in_form(
        &self,
        cx: &mut Cx<'_>,
        value: &mut bool,
        inherited_disabled: bool,
    ) -> Response<Activated> {
        self.with_inherited_disabled(inherited_disabled)
            .update(cx, value)
    }

    fn natural_width(&self) -> u16 {
        Self::MARKER_W
            .saturating_add(2)
            .saturating_add(width(self.label))
            .saturating_add(4)
    }

    /// The draw phase.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }
        // runtime: the frame's own focus/hover/press; derived: the switch's
        // `.on`, `.read_only` and `.disabled` props
        let mut derived = StateFlags::empty();
        if self.on {
            derived |= StateFlags::CHECKED | StateFlags::SELECTED;
        }
        if self.read_only {
            derived |= StateFlags::READ_ONLY;
        }
        if self.disabled {
            derived |= StateFlags::DISABLED;
        }
        let mut live = PartStyle::flags(crate::ui::FrameRead::state(ui, self.id), derived);
        if !self.on {
            live = live.difference(StateFlags::SELECTED);
        }
        if self.disabled {
            live = live.difference(StateFlags::HOVERED | StateFlags::PRESSED);
        }
        if !ui.is_inert() {
            let f = if self.disabled {
                Focusability::Disabled
            } else if self.read_only {
                Focusability::FocusableReadOnly
            } else {
                Focusability::Focusable
            };
            ui.register_control(self.id, area, f);
            ui.publish_bindings(self.id, live, FLAG);
        }
        let on = live.contains(StateFlags::CHECKED);
        // W04-01: under four columns the switch collapses to its
        // single-cell state dot (reference `Toggle::render`).
        let compact = area.width < 4;
        // Baseline guard (`visual-baseline:src/widgets/choice.rs:341-343`):
        // the trailing state word paints only when the full row fits, strict,
        // for both words (the `+3`-for-`on` quirk is the frozen authority).
        let show_word = 6u16.saturating_add(width(self.label)).saturating_add(3) < area.width;
        FlagRow {
            id: self.id,
            ov: self.ov,
            label: self.label,
            marker_w: Self::MARKER_W,
            trailing: show_word.then_some(if on { "on" } else { "off" }),
        }
        .draw(
            ui,
            area,
            live,
            &move |ui: &mut Ui<'_>, cell: Rect, style| {
                if compact {
                    let g = if on {
                        GlyphRole::SwitchKnob
                    } else {
                        GlyphRole::SwitchKnobOff
                    };
                    ui.glyph(cell, g, style);
                    return;
                }
                // the knob sits at the end of the track when the switch is on
                let (knob, track) = if on {
                    (cell.right().saturating_sub(1), cell.x)
                } else {
                    (cell.x, cell.x.saturating_add(1))
                };
                let rail = Rect {
                    x: track,
                    y: cell.y,
                    width: cell.width.saturating_sub(1),
                    height: 1,
                };
                for col in rail.columns() {
                    ui.glyph(col, GlyphRole::RuleQuiet, style);
                }
                let knob_glyph = if on {
                    GlyphRole::SwitchKnob
                } else {
                    GlyphRole::SwitchKnobOff
                };
                ui.glyph(cell_at(cell, knob), knob_glyph, style);
            },
        )
    }

    pub fn draw_in_form(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        value: bool,
        inherited_disabled: bool,
    ) -> Rect {
        self.with_inherited_disabled(inherited_disabled)
            .on(value)
            .draw(ui, area)
    }

    /// One row, the switch plus the label and the state word.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size::exact(self.natural_width(), 1).fit(c)
    }
}

impl Bindings for Toggle<'_> {
    type Cmd = ChoiceCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<ChoiceCmd>] {
        FLAG
    }
}

impl FieldControl for Toggle<'_> {
    type State = ();

    fn id(&self) -> Id {
        self.id
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect, _st: &()) -> Rect {
        Toggle::draw(self, ui, area)
    }

    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        Toggle::measure(self, ui, c)
    }
}

/// The default instantiation a form field holds (§15.1, §24 M3): options
/// are `&str` labels, keyed positionally, painted through `Display`.
pub type LabelRadio<'a> = RadioGroup<'a, &'a str, ByIndex, DefaultRow>;

/// Durable state of a [`RadioGroup`]: the **cursor** and the reconcile
/// stamp. The value is the caller's, supplied to both phases through
/// `.value(ItemKey)` and written by the caller when
/// [`RadioGroupAction::Chose`] arrives (§15, §20.10 item 3).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RadioGroupState {
    core: CollectionCore,
}

impl RadioGroupState {
    /// The cursor key.
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    /// The cursor's index as of the last reconcile.
    pub const fn cursor_index(&self) -> usize {
        self.core.cursor_index()
    }

    /// Point the cursor at `(index, key)`.
    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.core.set_cursor(index, key);
    }
}

impl Reconcile for RadioGroupState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        self.core.reconcile(len, key)
    }

    fn invalidate(&mut self) {
        self.core.invalidate();
    }
}

/// A radio group over borrowed options, with the **cursor
/// separated from the value**.
///
/// ## Construction
/// `RadioGroup::new(id)`; options are passed to each phase, never held
/// (§21 item 1). The value is a controlled prop (`.value(ItemKey)`), written by
/// the caller when [`RadioGroupAction::Chose`] arrives.
///
/// ## Ownership
/// The caller owns the options (`&[T]` per phase), the value and a
/// [`RadioGroupState`] (the cursor). The runtime owns focus, hover and
/// press.
///
/// ## Configuration
/// `.key(Fn(&T) -> ItemKey)` (`ByIndex`, unstable under reorder),
/// `.row(Fn(&T, &mut RowUi))` (`DefaultRow`: `Display`), `.value(ItemKey)`
/// (update and draw), `.read_only(bool)`, `.disabled(bool)`,
/// `.disabled_item(Fn(ItemKey) -> bool)`, `.orientation(Axis)` (`V`), `.patch`,
/// `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::CHOICE`, `DEFAULT` only.
///
/// ## States
/// The group wears `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED`, `PRESSED` from
/// the runtime and passes them to the **cursor** row only; the value row
/// wears `SELECTED`; `READ_ONLY` and `DISABLED` reach every row.
///
/// ## Actions
/// [`RadioGroupAction::Chose(k)`](RadioGroupAction::Chose) — `Space`,
/// `Enter` or a click committed option `k`. **Moving the cursor emits no
/// action**: this is the intentional change from the legacy fused
/// cursor-is-value behaviour (§20.10 item 3), so arrowing through a group
/// no longer fires a change per row.
///
/// ## Focus
/// One `Focusable` stop for the whole group (`FocusableReadOnly` /
/// `Disabled`, the latter also when every option is disabled); does not
/// swallow typing. Option rows are click targets, not focus stops.
///
/// ## Keyboard
/// `↑`/`k`, `↓`/`j` move the cursor (`←`/`h`, `→`/`l` when
/// `.orientation(Axis::H)`); `Home`/`End` jump; `Space` (visible)
/// and `Enter` commit the cursor option. The cursor skips disabled options.
///
/// ## Mouse
/// `PartRef::item(Part::ROW, k)`: a press moves the cursor, a click commits
/// option `k`. Disabled options register no hit target and never choose.
///
/// ## Layout
/// One row per option: gutter, a three-column marker, one space, the row
/// renderer's content. `.orientation(Axis::H)` lays the options out as one
/// horizontal strip of gutter + marker + label segments instead, each
/// measured from its painted label (ChipBar precedent); segments that do
/// not fit whole are dropped, never clipped. `measure` is `(16…, options)`
/// vertically and `(8…, 1)` horizontally; `draw` returns the rows it used;
/// `0×0` registers nothing (R5).
///
/// ## Parts
/// `CONTAINER` (the row fill), `GUTTER` (the focus bar), `MARKER` (the
/// radio), `LABEL` (through [`RowUi`]).
///
/// ## Overrides
/// `.patch`, `.patch_part`, `.slot` on `GUTTER` and `MARKER`.
///
/// ## Identity
/// `.key` supplies stable keys; `ByIndex` is unstable under
/// insert/remove/reorder. The action carries an `ItemKey`, never an index.
///
/// ## Testing
/// `RadioGroupCase` with `ACTIVATES | FOCUSABLE | COLLECTION |
/// DISABLEABLE | SELECTS`; `render::components::radio_group::*`;
/// `choice::radio_group_separates_cursor_from_value`.
///
/// ## Invariants
/// `reconcile` runs before any action is emitted; the marker is
/// [`GlyphRole::RadioOn`] / [`GlyphRole::RadioOff`], so the unselected half
/// is a glyph rather than the absence of colour; only visible rows invoke
/// the renderer.
pub struct RadioGroup<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    value: Option<ItemKey>,
    read_only: bool,
    disabled: bool,
    disabled_item: Option<&'a dyn Fn(ItemKey) -> bool>,
    orientation: Axis,
    ov: PartStyle<'a>,
    _t: PhantomData<fn() -> T>,
}

impl<T, K, R> fmt::Debug for RadioGroup<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RadioGroup")
            .field("id", &self.id)
            .field("value", &self.value)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .field("orientation", &self.orientation)
            .finish_non_exhaustive()
    }
}

impl<T> RadioGroup<'_, T, ByIndex, DefaultRow> {
    /// A radio group keyed by index and painted through `Display`.
    pub const fn new(id: Id) -> Self {
        RadioGroup {
            id,
            key: ByIndex,
            row: DefaultRow,
            value: None,
            read_only: false,
            disabled: false,
            disabled_item: None,
            orientation: Axis::V,
            ov: PartStyle::new(),
            _t: PhantomData,
        }
    }
}

impl<'a> RadioGroup<'a, &'a str, ByIndex, DefaultRow> {
    fn with_form_value(&self, value: usize, inherited_disabled: bool) -> Self {
        RadioGroup {
            id: self.id,
            key: ByIndex,
            row: DefaultRow,
            value: Some(ItemKey::index(value)),
            read_only: self.read_only,
            disabled: self.disabled || inherited_disabled,
            disabled_item: self.disabled_item,
            orientation: self.orientation,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    pub fn update_in_form(
        &self,
        cx: &mut Cx<'_>,
        st: &mut RadioGroupState,
        value: &mut usize,
        items: &[&'a str],
        inherited_disabled: bool,
    ) -> Response<RadioGroupAction> {
        let response = self
            .with_form_value(*value, inherited_disabled)
            .update(cx, st, items);
        if let Some(RadioGroupAction::Chose(ItemKey::Index(index))) = response.action_ref() {
            *value = *index;
        }
        response
    }

    pub fn draw_in_form(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &RadioGroupState,
        value: usize,
        items: &[&'a str],
        inherited: InheritedFormState,
    ) -> Rect {
        self.with_form_value(value, inherited.disabled)
            .draw(ui, area, st, items)
    }
}

impl<'a, T, K, R> RadioGroup<'a, T, K, R> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::MARKER, Part::LABEL];

    /// Columns the marker occupies.
    const MARKER_W: u16 = 3;

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// A stable key accessor.
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> RadioGroup<'a, T, K2, R> {
        RadioGroup {
            id: self.id,
            key: k,
            row: self.row,
            value: self.value,
            read_only: self.read_only,
            disabled: self.disabled,
            disabled_item: self.disabled_item,
            orientation: self.orientation,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    /// A row painter.
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> RadioGroup<'a, T, K, R2> {
        RadioGroup {
            id: self.id,
            key: self.key,
            row: r,
            value: self.value,
            read_only: self.read_only,
            disabled: self.disabled,
            disabled_item: self.disabled_item,
            orientation: self.orientation,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    /// The controlled value, for `update` and `draw`.
    ///
    /// The value is **not** state: `update` never writes it, it reports
    /// [`RadioGroupAction::Chose`] and the caller writes its own field —
    /// the controlled-value convention of §13, with the write happening in
    /// the action handler rather than through a `&mut` parameter, because a
    /// group's value is an `ItemKey` chosen from the items the phase call
    /// already carries.
    #[must_use]
    pub const fn value(mut self, k: ItemKey) -> Self {
        self.value = Some(k);
        self
    }

    /// Read-only: stays in the ring, never commits.
    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    /// Disabled: registered, never reachable.
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    /// Which options are disabled, by stable key. Disabled options stay
    /// visible, are skipped by the cursor, register no hit target and
    /// never choose (W05-02). Keyed rather than item-based (cf.
    /// `List::disabled_item`): a `&dyn Fn(&T)` predicate would pin the
    /// group invariant in `T` and break the form instantiations that
    /// shrink lifetimes across phases.
    #[must_use]
    pub const fn disabled_item(mut self, f: &'a dyn Fn(ItemKey) -> bool) -> Self {
        self.disabled_item = Some(f);
        self
    }

    /// The layout axis: vertical rows (`Axis::V`, default) or one
    /// horizontal strip of option segments (W05-04).
    #[must_use]
    pub const fn orientation(mut self, axis: Axis) -> Self {
        self.orientation = axis;
        self
    }

    /// An instance patch over every part.
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

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Showcase / fixture use only (A11).
    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    /// The key table for the layout axis: Up/Down vertically,
    /// Left/Right horizontally.
    fn table(&self) -> &'static [Binding<ChoiceCmd>] {
        match self.orientation {
            Axis::V => RADIO,
            Axis::H => RADIO_H,
        }
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> RadioGroup<'_, T, K, R> {
    fn is_disabled(&self, key: ItemKey) -> bool {
        self.disabled_item.is_some_and(|f| f(key))
    }

    fn enabled_at(&self, items: &[T], i: usize) -> bool {
        i < items.len() && !self.is_disabled(key_at(&self.key, items, i))
    }

    /// Shared clamped target resolution over the enabled options (W05-02):
    /// the cursor skips disabled options. Clamps, never wraps.
    fn move_cursor(
        &self,
        st: &mut RadioGroupState,
        items: &[T],
        from: usize,
        forward: bool,
        acc: &mut Acc<RadioGroupAction>,
    ) {
        if items.is_empty() {
            acc.consumed();
            return;
        }
        let from = from.min(items.len().saturating_sub(1));
        let dir = if forward {
            StepDir::Next
        } else {
            StepDir::Prev
        };
        let Some((to, key)) = CollectionCore::seek(
            items.len(),
            from,
            dir,
            |i| key_at(&self.key, items, i),
            |i| self.enabled_at(items, i),
        ) else {
            acc.consumed();
            return;
        };
        st.core.set_cursor(to, key);
        // the cursor is not the value: moving it repaints and reports
        // nothing (§20.10 item 3)
        acc.changed();
    }

    fn choose(
        &self,
        st: &mut RadioGroupState,
        items: &[T],
        i: usize,
        acc: &mut Acc<RadioGroupAction>,
    ) {
        if items.is_empty() {
            acc.consumed();
            return;
        }
        let i = i.min(items.len().saturating_sub(1));
        if !self.enabled_at(items, i) {
            acc.consumed();
            return;
        }
        let key = key_at(&self.key, items, i);
        st.core.set_cursor(i, key);
        acc.action(RadioGroupAction::Chose(key));
    }

    /// The update phase: reconcile when enabled, then move the cursor or
    /// commit it.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut RadioGroupState,
        items: &[T],
    ) -> Response<RadioGroupAction> {
        let can = self.editable();
        let len = items.len();
        if !self.disabled {
            let _ = st.core.reconcile(len, |i| key_at(&self.key, items, i));
            if st.core.cursor().is_none() && len > 0 {
                // the cursor starts on the value when there is one and it
                // is enabled, else on the first enabled option; an
                // all-disabled group keeps no cursor (W05-02/W05-04)
                let seed = self
                    .value
                    .and_then(|v| index_of(&self.key, items, v, None))
                    .filter(|&i| self.enabled_at(items, i))
                    .map(|i| (i, key_at(&self.key, items, i)))
                    .or_else(|| {
                        CollectionCore::seek(
                            items.len(),
                            0,
                            StepDir::Next,
                            |i| key_at(&self.key, items, i),
                            |i| self.enabled_at(items, i),
                        )
                    });
                if let Some((i, key)) = seed {
                    st.core.set_cursor(i, key);
                }
            }
        }
        let mut acc = Acc::<RadioGroupAction>::new();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) if can => {
                    let cur = st.core.cursor_index();
                    match Binding::command(self.table(), action) {
                        Some(ChoiceCmd::Prev) => {
                            self.move_cursor(st, items, cur.saturating_sub(1), false, &mut acc);
                        }
                        Some(ChoiceCmd::Next) => {
                            self.move_cursor(st, items, cur.saturating_add(1), true, &mut acc);
                        }
                        Some(ChoiceCmd::First) => {
                            self.move_cursor(st, items, 0, true, &mut acc);
                        }
                        Some(ChoiceCmd::Last) => {
                            self.move_cursor(st, items, usize::MAX, false, &mut acc);
                        }
                        Some(ChoiceCmd::Choose) => self.choose(st, items, cur, &mut acc),
                        None => {}
                    }
                }
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::ROW,
                            item: Some(k),
                        },
                    ..
                } if can => {
                    let Some(i) = index_of(&self.key, items, k, Some(st.core.cursor_index()))
                    else {
                        acc.consumed();
                        continue;
                    };
                    if !self.enabled_at(items, i) {
                        acc.consumed();
                        continue;
                    }
                    match phase {
                        Phase::Press => self.move_cursor(st, items, i, true, &mut acc),
                        Phase::Click | Phase::DoubleClick => self.choose(st, items, i, &mut acc),
                        _ => acc.consumed(),
                    }
                }
                Intent::Pointer { .. } => acc.consumed(),
                _ => {}
            }
        }
        acc.finish(self.id)
    }

    /// The rect the group paints into: one row per option that fits `area`.
    fn used_rect(area: Rect, len: usize) -> Rect {
        let rows = usize::from(area.height).min(len);
        Rect {
            height: rows.min(usize::from(u16::MAX)) as u16,
            ..area
        }
    }

    /// Registers the group as one control over `used`.
    ///
    /// A reference rendering registers nothing, so it cannot take focus from
    /// the live frame. A group with no enabled option registers `Disabled`
    /// (W05-04): like the empty group it paints but takes no focus.
    fn register(&self, ui: &mut Ui<'_>, used: Rect, has_enabled: bool) {
        if ui.is_inert() {
            return;
        }
        let f = if self.disabled || !has_enabled {
            Focusability::Disabled
        } else if self.read_only {
            Focusability::FocusableReadOnly
        } else {
            Focusability::Focusable
        };
        ui.register_control(self.id, used, f);
    }

    /// Whether any option can take the cursor.
    fn has_enabled(&self, items: &[T]) -> bool {
        (0..items.len()).any(|i| self.enabled_at(items, i))
    }

    /// The state flags row `i` paints with, and whether that row carries the
    /// value.
    ///
    /// Runtime state may style the actual cursor and value rows, but never
    /// supplies either semantic identity.
    fn row_flags(
        &self,
        live: StateFlags,
        cursor: Option<ItemKey>,
        key: ItemKey,
        hovered: bool,
        pressed: bool,
        item_disabled: bool,
    ) -> (StateFlags, bool) {
        let is_cursor = cursor == Some(key);
        let on = self.value == Some(key);
        let mut flags = StateFlags::empty();
        if is_cursor {
            flags |= live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
        }
        if hovered {
            flags |= StateFlags::HOVERED;
        }
        if pressed {
            flags |= StateFlags::PRESSED;
        }
        if on {
            flags |= StateFlags::SELECTED;
        }
        if self.read_only {
            flags |= StateFlags::READ_ONLY;
        }
        if item_disabled || self.disabled || live.contains(StateFlags::DISABLED) {
            flags |= StateFlags::DISABLED;
            flags = flags.difference(StateFlags::PRESSED | StateFlags::HOVERED);
        }
        (flags, on)
    }

    /// Paints the gutter column of one option row.
    fn paint_gutter(&self, ui: &mut Ui<'_>, cell: Rect, flags: StateFlags) {
        if let Some(f) = self.ov.slot_for(Part::GUTTER) {
            f(ui, cell);
            return;
        }
        let g = self.ov.style(
            ui,
            self.id,
            Family::CHOICE,
            Variant::DEFAULT,
            Part::GUTTER,
            flags,
        );
        match g.glyph {
            Slot::Set(glyph) => {
                ui.glyph(cell, glyph, g.style);
            }
            Slot::Inherit | Slot::Clear => ui.fill(cell, g.style),
        }
    }

    /// Paints the radio marker of one option row.
    fn paint_marker(&self, ui: &mut Ui<'_>, cell: Rect, flags: StateFlags, on: bool) {
        let g = if on {
            GlyphRole::RadioOn
        } else {
            GlyphRole::RadioOff
        };
        self.paint_marker_role(ui, cell, flags, g);
    }

    /// Paints the single-cell state dot of a narrow option row: the marker
    /// roles have no compact half, so the dot reuses the switch-knob pair
    /// (the `Toggle` compact precedent).
    fn paint_narrow_marker(&self, ui: &mut Ui<'_>, cell: Rect, flags: StateFlags, on: bool) {
        let g = if on {
            GlyphRole::SwitchKnob
        } else {
            GlyphRole::SwitchKnobOff
        };
        self.paint_marker_role(ui, cell, flags, g);
    }

    /// Paints one marker glyph through the `MARKER` part style.
    fn paint_marker_role(&self, ui: &mut Ui<'_>, cell: Rect, flags: StateFlags, g: GlyphRole) {
        if let Some(f) = self.ov.slot_for(Part::MARKER) {
            f(ui, cell);
            return;
        }
        let ms = self.ov.style(
            ui,
            self.id,
            Family::CHOICE,
            Variant::DEFAULT,
            Part::MARKER,
            flags,
        );
        ui.glyph(cell, g, ms.style);
    }

    /// Paints one option row: the container surface, the gutter, the marker
    /// and the caller's row body.
    fn paint_row(
        &self,
        ui: &mut Ui<'_>,
        row: Rect,
        item: &T,
        key: ItemKey,
        flags: StateFlags,
        on: bool,
    ) {
        let container = self.ov.style(
            ui,
            self.id,
            Family::CHOICE,
            Variant::DEFAULT,
            Part::CONTAINER,
            flags,
        );
        ui.fill(row, container.style);
        self.paint_gutter(ui, cell_at(row, row.x), flags);
        let marker_cell = Rect {
            x: row.x.saturating_add(1),
            y: row.y,
            width: Self::MARKER_W.min(row.width.saturating_sub(1)),
            height: 1,
        };
        // Narrow vertical allocations (reference `RadioGroup::render`): under
        // four columns the marker collapses to its single-cell state dot. A
        // one-column row skips the marker entirely — the legacy x+1 write
        // lands outside the area (W05-04 containment).
        if (2..4).contains(&row.width) {
            self.paint_narrow_marker(
                ui,
                Rect {
                    width: 1,
                    ..marker_cell
                },
                flags,
                on,
            );
        } else if row.width >= 4 {
            self.paint_marker(ui, marker_cell, flags, on);
        }
        let rest = Rect {
            x: row
                .x
                .saturating_add(1)
                .saturating_add(Self::MARKER_W)
                .saturating_add(1),
            y: row.y,
            width: row.width.saturating_sub(2).saturating_sub(Self::MARKER_W),
            height: 1,
        };
        if !rest.is_empty() {
            // Forward the instance patches (the `List` precedent): a bare
            // `RowUi::new` would never see them, so a LABEL patch would not
            // reach the row painter.
            let mut r = RowUi::new_with_patches(
                ui,
                self.id,
                Family::CHOICE,
                Variant::DEFAULT,
                flags,
                key,
                rest,
                self.ov.part_patch(Part::CONTAINER),
                self.ov.part_patch(Part::LABEL),
            );
            self.row.row(item, &mut r);
        }
    }

    /// The draw phase: one row per option, or one horizontal strip.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &RadioGroupState, items: &[T]) -> Rect {
        if area.is_empty() {
            return area;
        }
        match self.orientation {
            Axis::V => self.draw_vertical(ui, area, st, items),
            Axis::H => self.draw_horizontal(ui, area, st, items),
        }
    }

    /// The draw phase: one row per option.
    fn draw_vertical(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &RadioGroupState,
        items: &[T],
    ) -> Rect {
        let used = Self::used_rect(area, items.len());
        if used.is_empty() {
            return used;
        }
        self.register(ui, used, self.has_enabled(items));
        // runtime: the group's own frame state; derived: none — the group's
        // `.disabled` and `.read_only` enter per row, in `row_flags`
        let live = PartStyle::flags(
            crate::ui::FrameRead::state(ui, self.id),
            StateFlags::empty(),
        );
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, self.table());
        }
        let cursor = st.core.cursor();
        let rows = usize::from(used.height);
        for (i, item) in items.iter().enumerate().take(rows) {
            let key = self.key.key(item, i);
            let row = Rect {
                x: used.x,
                y: used.y.saturating_add(i.min(usize::from(u16::MAX)) as u16),
                width: used.width,
                height: 1,
            };
            let part = PartRef::item(Part::ROW, key);
            let item_disabled = self.is_disabled(key);
            let (flags, on) = self.row_flags(
                live,
                cursor,
                key,
                ui.hovered_part(self.id) == Some(part),
                ui.pressed_part(self.id) == Some(part),
                item_disabled,
            );
            self.paint_row(ui, row, item, key, flags, on);
            if !ui.is_inert() && !item_disabled {
                ui.register_part(self.id, PartRef::item(Part::ROW, key), row);
            }
        }
        used
    }

    /// Columns a horizontal segment reserves before its label: gutter,
    /// three-column marker, one space.
    const SEGMENT_CHROME: u16 = 5;

    /// The draw phase: one horizontal strip of option segments (W05-04).
    ///
    /// Each segment is measured from its painted label (the ChipBar
    /// paint-then-measure precedent): the row painter runs into the rest
    /// of the strip, the label width is scanned back, and a segment that
    /// does not fit whole is erased and dropped, never clipped.
    fn draw_horizontal(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &RadioGroupState,
        items: &[T],
    ) -> Rect {
        if items.is_empty() {
            return Rect {
                width: 0,
                ..first_row(area)
            };
        }
        let used = first_row(area);
        self.register(ui, used, self.has_enabled(items));
        let live = PartStyle::flags(
            crate::ui::FrameRead::state(ui, self.id),
            StateFlags::empty(),
        );
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, self.table());
        }
        let base = self.ov.style(
            ui,
            self.id,
            Family::CHOICE,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        );
        let cursor = st.core.cursor();
        let mut x = used.x;
        for (i, item) in items.iter().enumerate() {
            let key = self.key.key(item, i);
            let avail = used.right().saturating_sub(x);
            if avail < Self::SEGMENT_CHROME {
                break;
            }
            let part = PartRef::item(Part::ROW, key);
            let item_disabled = self.is_disabled(key);
            let (flags, on) = self.row_flags(
                live,
                cursor,
                key,
                ui.hovered_part(self.id) == Some(part),
                ui.pressed_part(self.id) == Some(part),
                item_disabled,
            );
            let content = Rect {
                x: x.saturating_add(Self::SEGMENT_CHROME),
                y: used.y,
                width: avail.saturating_sub(Self::SEGMENT_CHROME),
                height: 1,
            };
            if !content.is_empty() {
                let mut r = RowUi::new_with_patches(
                    ui,
                    self.id,
                    Family::CHOICE,
                    Variant::DEFAULT,
                    flags,
                    key,
                    content,
                    self.ov.part_patch(Part::CONTAINER),
                    self.ov.part_patch(Part::LABEL),
                );
                self.row.row(item, &mut r);
            }
            let label_w = painted_width(ui, content);
            let seg_w = Self::SEGMENT_CHROME.saturating_add(label_w);
            if seg_w > avail {
                // the segment does not fit whole: erase what the row
                // painter put down and stop, rather than leave half a
                // segment
                ui.fill(content, base.style);
                break;
            }
            let seg = Rect {
                x,
                y: used.y,
                width: seg_w,
                height: 1,
            };
            let container = self.ov.style(
                ui,
                self.id,
                Family::CHOICE,
                Variant::DEFAULT,
                Part::CONTAINER,
                flags,
            );
            ui.fill(
                Rect {
                    x: seg.x,
                    y: seg.y,
                    width: Self::SEGMENT_CHROME,
                    height: 1,
                },
                container.style,
            );
            self.paint_gutter(ui, cell_at(seg, seg.x), flags);
            let marker_cell = Rect {
                x: seg.x.saturating_add(1),
                y: seg.y,
                width: Self::MARKER_W.min(seg_w.saturating_sub(1)),
                height: 1,
            };
            self.paint_marker(ui, marker_cell, flags, on);
            if !ui.is_inert() && !item_disabled {
                ui.register_part(self.id, PartRef::item(Part::ROW, key), seg);
            }
            x = x.saturating_add(seg_w);
        }
        // the tail — the last segment's overshoot plus the unclaimed rest —
        // reads neutral
        ui.fill(
            Rect {
                x,
                y: used.y,
                width: used.right().saturating_sub(x),
                height: 1,
            },
            base.style,
        );
        used
    }

    /// The natural size: sixteen columns by one row per option vertically,
    /// one strip row horizontally.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        match self.orientation {
            Axis::V => Size {
                min: (16, 1),
                preferred: (24, c.max.1.max(1)),
            }
            .fit(c),
            Axis::H => Size {
                min: (8, 1),
                preferred: (24, 1),
            }
            .fit(c),
        }
    }
}

impl<T, K, R> Bindings for RadioGroup<'_, T, K, R> {
    type Cmd = ChoiceCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<ChoiceCmd>] {
        self.table()
    }
}

/// The painted label width of a horizontal segment's content rect: the
/// last non-blank column plus one, ignoring a `RowUi::meta` suffix after
/// a two-cell gap (the ChipBar `painted_width` precedent).
fn painted_width(ui: &mut Ui<'_>, row: Rect) -> u16 {
    ui.with_area(row, |ui| {
        let (buf, clip) = ui.raw();
        let mut last = 0u16;
        let mut blank_run = 0u16;
        let mut gap_start = None;
        for x in clip.columns().map(|c| c.x) {
            let non_blank = buf
                .cell(Position::new(x, clip.y))
                .is_some_and(|c| c.symbol() != " ");
            if non_blank {
                if blank_run >= 2 {
                    gap_start = Some(x.saturating_sub(blank_run).saturating_sub(clip.x));
                }
                blank_run = 0;
                last = x.saturating_sub(clip.x).saturating_add(1);
            } else {
                blank_run = blank_run.saturating_add(1);
            }
        }
        if last == clip.width {
            gap_start.unwrap_or(last)
        } else {
            last
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Input;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::runtime::{App, Runtime};
    use crate::theme::Theme;
    use ratatui_core::buffer::Buffer;

    const RG: Id = Id::root("choice.tests.radio");

    #[derive(Default)]
    struct DisabledRadioApp {
        state: RadioGroupState,
    }

    impl App for DisabledRadioApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let items = ["alpha", "beta"];
            RadioGroup::new(RG)
                .disabled(true)
                .update(cx, &mut self.state, &items)
                .erase()
        }

        fn draw(&self, _ui: &mut Ui<'_>) {}
    }

    struct ControlledRadioApp {
        state: RadioGroupState,
        value: ItemKey,
    }

    impl App for ControlledRadioApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let items = ["a", "b", "c"];
            RadioGroup::new(RG)
                .value(self.value)
                .update(cx, &mut self.state, &items)
                .erase()
        }

        fn draw(&self, _ui: &mut Ui<'_>) {}
    }

    fn draw_checkbox(checked: bool) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, area| {
                Checkbox::new(RG, "Choice").checked(checked).draw(ui, area);
            })
            .commit_presented();
        buffer
    }

    fn draw_toggle(on: bool) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, area| {
                Toggle::new(RG, "Choice").on(on).draw(ui, area);
            })
            .commit_presented();
        buffer
    }

    #[test]
    fn checked_painting_comes_only_from_the_checkbox_prop() {
        assert_ne!(draw_checkbox(true), draw_checkbox(false));
    }

    #[test]
    fn checked_painting_comes_only_from_the_toggle_prop() {
        assert_ne!(draw_toggle(true), draw_toggle(false));
    }

    fn draw_toggle_sized(label: &str, on: bool, w: u16) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        let area = Rect {
            x: 0,
            y: 0,
            width: w,
            height: 1,
        };
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, _| {
                Toggle::new(RG, label).on(on).draw(ui, area);
            })
            .commit_presented();
        buffer
    }

    /// Symbols of buffer row `y` over `x0..=x1`.
    fn buf_row(buf: &Buffer, y: u16, x0: u16, x1: u16) -> String {
        (x0..=x1)
            .map(|x| {
                buf.cell(Position::new(x, y))
                    .map(|c| c.symbol().to_string())
                    .unwrap_or_default()
            })
            .collect()
    }

    /// L1: the off knob is the `○──` half of the pair, not `●──`.
    #[test]
    fn toggle_off_switch_pair() {
        let row = buf_row(&draw_toggle(false), 0, 0, 39);
        assert!(row.contains("○──"), "off row must show ○──, got {row:?}");
        assert!(row.contains("off"), "off row must show off, got {row:?}");
        let row = buf_row(&draw_toggle(true), 0, 0, 39);
        assert!(row.contains("──●"), "on row must show ──●, got {row:?}");
        assert!(row.contains("on"), "on row must show on, got {row:?}");
    }

    /// L2: the clipped state word is suppressed unless `6+labelw+3 < width`.
    /// A 27-char label in 34/35/36-wide rects paints no word char (pre-fix
    /// fragments `o`/`of`/`off`); 40-wide paints `off`.
    #[test]
    fn toggle_suppresses_clipped_state_word() {
        let label = "Local only (no iCloud sync)";
        assert_eq!(width(label), 27, "test label must be 27 wide");
        for w in [34u16, 35, 36] {
            let buf = draw_toggle_sized(label, false, w);
            assert_eq!(
                buf_row(&buf, 0, 5, 31),
                label,
                "{w}-wide: label must be intact"
            );
            assert!(
                buf_row(&buf, 0, 33, w.saturating_sub(1))
                    .chars()
                    .all(|c| c == ' '),
                "{w}-wide: word zone must be blank"
            );
        }
        let buf = draw_toggle_sized(label, false, 40);
        assert_eq!(
            buf_row(&buf, 0, 33, 35),
            "off",
            "40-wide: the word must paint"
        );
    }

    /// A disabled collection remains drawable from its current item slice,
    /// but its update phase must not initialize or reconcile persistent state.
    #[test]
    fn disabled_update_does_not_initialize_collection_state() {
        let mut runtime = Runtime::new(DisabledRadioApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        assert_eq!(runtime.app().state, RadioGroupState::default());
    }

    /// §16.1 / §20.10 item 3: arrows move the cursor and commit nothing; the
    /// value changes only on `Space` / `Enter` / a click, and it is the
    /// caller's field, never state.
    #[test]
    fn radio_group_separates_cursor_from_value() {
        let items = ["a", "b", "c"];
        let mut st = RadioGroupState::default();
        let g: RadioGroup<'_, &str> = RadioGroup::new(RG);
        let mut acc = Acc::<RadioGroupAction>::new();
        let _ = st.core.reconcile(3, |i| key_at(&g.key, &items, i));
        st.set_cursor(0, ItemKey::index(0));
        g.move_cursor(&mut st, &items, 1, true, &mut acc);
        g.move_cursor(&mut st, &items, 2, true, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::index(2)));
        let moved = acc.finish(RG);
        assert!(moved.is_changed(), "the cursor repaints");
        assert_eq!(
            moved.action_ref(),
            None,
            "moving the cursor must not report a choice"
        );
        let mut acc = Acc::<RadioGroupAction>::new();
        let at = st.cursor_index();
        g.choose(&mut st, &items, at, &mut acc);
        assert_eq!(
            acc.finish(RG).action_ref(),
            Some(&RadioGroupAction::Chose(ItemKey::index(2)))
        );
        // the cursor also lands on the value when the group first draws
        let mut fresh = RadioGroupState::default();
        let valued: RadioGroup<'_, &str> = RadioGroup::new(RG).value(ItemKey::index(1));
        let _ = fresh.core.reconcile(3, |i| key_at(&valued.key, &items, i));
        assert!(fresh.cursor().is_none());
        assert_eq!(
            index_of(&valued.key, &items, ItemKey::index(1), None),
            Some(1)
        );
    }

    #[test]
    fn radio_group_value_is_controlled_by_the_caller() {
        let items = ["a", "b", "c"];
        let selected = ItemKey::index(1);
        let group: RadioGroup<'_, &str> = RadioGroup::new(RG).value(selected);
        let mut runtime = Runtime::new(
            ControlledRadioApp {
                state: RadioGroupState::default(),
                value: selected,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        assert_eq!(runtime.app().state.cursor(), Some(selected));

        let mut state = runtime.app().state.clone();
        let mut action = Acc::<RadioGroupAction>::new();
        group.choose(&mut state, &items, 2, &mut action);
        assert_eq!(
            action.finish(RG).action_ref(),
            Some(&RadioGroupAction::Chose(ItemKey::index(2)))
        );
        assert!(
            group
                .row_flags(
                    StateFlags::empty(),
                    state.cursor(),
                    selected,
                    false,
                    false,
                    false
                )
                .1
        );
        assert!(
            !group
                .row_flags(
                    StateFlags::empty(),
                    state.cursor(),
                    ItemKey::index(2),
                    false,
                    false,
                    false,
                )
                .1,
            "choosing emits an action; it does not mutate the caller's value"
        );
    }

    #[test]
    fn radio_group_missing_or_vanished_value_marks_no_option() {
        let items = ["a", "b"];
        let missing = ItemKey::text("missing");
        let group: RadioGroup<'_, &str> = RadioGroup::new(RG).value(missing);

        for (i, item) in items.iter().enumerate() {
            let key = group.key.key(item, i);
            assert!(
                !group
                    .row_flags(StateFlags::empty(), None, key, false, false, false)
                    .1,
                "a missing controlled value must not select a fallback row"
            );
        }

        let vanished_items = ["a"];
        let vanished_group: RadioGroup<'_, &str> = RadioGroup::new(RG).value(ItemKey::index(1));
        let only_key = key_at(&vanished_group.key, &vanished_items, 0);
        assert!(
            !vanished_group
                .row_flags(
                    StateFlags::empty(),
                    Some(only_key),
                    only_key,
                    false,
                    false,
                    false
                )
                .1,
            "the cursor is not stored chosen state after the value vanishes"
        );
    }

    #[test]
    fn selected_painting_comes_only_from_the_controlled_value() {
        let selected = ItemKey::text("beta");
        let radio = RadioGroup::new(RG).key(|item: &&str| ItemKey::text(item));
        assert_eq!(
            radio.row_flags(
                StateFlags::empty(),
                None,
                ItemKey::text("alpha"),
                false,
                false,
                false
            ),
            (StateFlags::empty(), false)
        );

        let controlled = radio.value(selected);
        let (flags, on) =
            controlled.row_flags(StateFlags::empty(), None, selected, false, false, false);
        assert!(on);
        assert_eq!(flags, StateFlags::SELECTED);
    }

    #[test]
    fn focus_styles_only_the_runtime_cursor_without_selecting_it() {
        let radio: RadioGroup<'_, &str> = RadioGroup::new(RG);

        assert_eq!(
            radio.row_flags(
                StateFlags::FOCUSED,
                Some(ItemKey::index(0)),
                ItemKey::index(0),
                false,
                false,
                false
            ),
            (StateFlags::FOCUSED, false)
        );
        assert_eq!(
            radio.row_flags(
                StateFlags::FOCUSED,
                Some(ItemKey::index(0)),
                ItemKey::index(1),
                false,
                false,
                false
            ),
            (StateFlags::empty(), false)
        );
    }

    #[test]
    fn radio_choose_action_uses_the_items_stable_key() {
        let items = ["alpha", "beta"];
        let group = RadioGroup::new(RG).key(|item: &&str| ItemKey::text(item));
        let mut state = RadioGroupState::default();
        let mut acc = Acc::<RadioGroupAction>::new();

        group.choose(&mut state, &items, 1, &mut acc);

        assert_eq!(
            acc.finish(RG).action_ref(),
            Some(&RadioGroupAction::Chose(ItemKey::text("beta")))
        );
        assert_eq!(state.cursor(), Some(ItemKey::text("beta")));
    }

    /// The marker is a glyph pair, so an unselected option is visible
    /// without colour (§11.4).
    #[test]
    fn the_radio_marker_is_a_glyph_pair() {
        let items = ["alpha", "beta"];
        let st = RadioGroupState::default();
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            let g: RadioGroup<'_, &str> = RadioGroup::new(RG).value(ItemKey::index(0));
            g.draw(ui, a, &st, &items);
        })
        .commit_presented();
        let mut text = String::new();
        for y in 0..2u16 {
            for x in 0..SCREEN.width {
                if let Some(c) = buf.cell(ratatui_core::layout::Position::new(x, y)) {
                    text.push_str(c.symbol());
                }
            }
        }
        let glyphs = &Theme::junie().design.glyphs;
        assert!(text.contains(glyphs.get(GlyphRole::RadioOn)), "{text}");
        assert!(text.contains(glyphs.get(GlyphRole::RadioOff)), "{text}");
        assert!(text.contains("alpha") && text.contains("beta"));
    }

    fn draw_radio(items: &[&str], value: usize, area: Rect) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, _| {
                let g: RadioGroup<'_, &str> = RadioGroup::new(RG).value(ItemKey::index(value));
                g.draw(ui, area, &RadioGroupState::default(), items);
            })
            .commit_presented();
        buffer
    }

    /// L1: the off marker is the frozen `( )`, not the theme's old `(○)`.
    #[test]
    fn radio_off_marker_is_blank() {
        let buf = draw_radio(
            &["alpha", "beta"],
            0,
            Rect {
                x: 0,
                y: 0,
                width: 30,
                height: 2,
            },
        );
        assert_eq!(buf_row(&buf, 0, 1, 3), "(●)", "on row marker");
        assert_eq!(buf_row(&buf, 1, 1, 3), "( )", "off row marker");
        assert!(
            !buf_row(&buf, 1, 0, 29).contains('○'),
            "no ○ may survive in the off row"
        );
    }

    /// L2: narrow rows collapse the marker to its single-cell state dot;
    /// a one-column row skips the marker — the legacy x+1 write lands
    /// outside the area (W05-04 containment).
    #[test]
    fn radio_narrow_markers() {
        use ratatui_core::style::Color;
        let items = ["aa", "bb"];
        // w=1: gutter only; the x+1 cell is untouched.
        let buf = draw_radio(&items, 0, Rect::new(2, 1, 1, 2));
        for y in 1..3 {
            assert_eq!(buf_row(&buf, y, 2, 2), " ", "w=1 gutter");
            let outside = buf.cell(Position::new(3, y)).expect("the x+1 cell exists");
            assert_eq!(outside.symbol(), " ", "w=1 x+1 symbol");
            assert_eq!(outside.fg, Color::Reset, "w=1 x+1 untouched");
        }
        // w=2..3: one dot at x+1, no label chars.
        let buf = draw_radio(&items, 0, Rect::new(2, 1, 2, 2));
        assert_eq!(buf_row(&buf, 1, 2, 3), " ●", "w=2 on row");
        assert_eq!(buf_row(&buf, 2, 2, 3), " ○", "w=2 off row");
        let buf = draw_radio(&items, 0, Rect::new(2, 1, 3, 2));
        assert_eq!(buf_row(&buf, 1, 2, 4), " ● ", "w=3 on row");
        assert_eq!(buf_row(&buf, 2, 2, 4), " ○ ", "w=3 off row");
        // w=4: full markers, still no room for labels.
        let buf = draw_radio(&items, 0, Rect::new(2, 1, 4, 2));
        assert_eq!(buf_row(&buf, 1, 2, 5), " (●)", "w=4 on row");
        assert_eq!(buf_row(&buf, 2, 2, 5), " ( )", "w=4 off row");
    }

    /// L3: the unselected marker resolves Muted, the selected one Accent;
    /// the shared recipe keeps checkbox/toggle on Accent and off Muted.
    #[test]
    fn radio_off_marker_muted() {
        use ratatui_core::style::Color;
        const MUTED: Color = Color::Rgb(128, 128, 128);
        const ACCENT: Color = Color::Rgb(72, 224, 84);
        let fg = |buf: &Buffer, x: u16, y: u16| {
            buf.cell(Position::new(x, y))
                .expect("marker cell exists")
                .fg
        };
        let buf = draw_radio(
            &["alpha", "beta"],
            0,
            Rect {
                x: 0,
                y: 0,
                width: 30,
                height: 2,
            },
        );
        for x in 1..4 {
            assert_eq!(fg(&buf, x, 0), ACCENT, "on marker x={x}");
            assert_eq!(fg(&buf, x, 1), MUTED, "off marker x={x}");
        }
        for x in 1..4 {
            assert_eq!(fg(&draw_checkbox(true), x, 0), ACCENT, "checked box x={x}");
            assert_eq!(
                fg(&draw_checkbox(false), x, 0),
                MUTED,
                "unchecked box x={x}"
            );
        }
        assert_eq!(fg(&draw_toggle(true), 3, 0), ACCENT, "on knob");
        assert_eq!(fg(&draw_toggle(false), 1, 0), MUTED, "off knob");
    }
}
