#![doc = r" `Checkbox`, `Toggle` and `RadioGroup` (`COMPONENT_ARCHITECTURE.md` §15,"]
#![doc = r" §17.0 A7/A10, §18.2, §20.10 item 3, Appendix A 4B)."]
#![doc = r""]
#![doc = r" The three controls share one keymap (`Space` / `Enter` commit) and one"]
#![doc = r" marker vocabulary: the glyph is a [`GlyphRole`] the component names and"]
#![doc = r" the theme binds, because a two-state affordance's *off* half cannot be"]
#![doc = r#" expressed by a `StateRule` — a rule binds one glyph to a state, and "not"#]
#![doc = r#" checked" is the absence of a state, not a state of its own."#]

use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn, cell_at, first_row};
use crate::action::ActionKey;
use crate::event::{Chord, KeyCode};
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
use termrock_collections::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, Reconcile, Reconciliation,
};
use termrock_runtime::{RowFn, RowUi};

#[doc = r" What a radio group reports; the cursor moving is **not** an action —"]
#[doc = r" cursor and value are separate (§15, §20.10 item 3)."]
#[doc = r" Editing state a `Form` (in `termrock-forms`) hands to its field"]
#[doc = r" controls: shared flags every embedded control must respect."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct InheritedFormState {
    #[doc = r" The owning form field is disabled; the control paints and behaves"]
    #[doc = r" accordingly without owning the disable decision."]
    pub disabled: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RadioGroupAction {
    #[doc = r" The cursor option was committed as the value."]
    Chose(ItemKey),
}

#[doc = r" The const-constructible commands of the choice keymap."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChoiceCmd {
    #[doc = r" Commit the cursor option / flip the flag."]
    Choose,
    #[doc = r" Cursor to the previous option."]
    Prev,
    #[doc = r" Cursor to the next option."]
    Next,
    #[doc = r" Cursor to the first option."]
    First,
    #[doc = r" Cursor to the last option."]
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

#[doc = r" `Checkbox` / `Toggle`: one commit chord, two spellings."]
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

#[doc = r" `RadioGroup`: arrows move the cursor, `Space` / `Enter` commit it."]
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

#[doc = r" One flag row's chrome — gutter, marker glyph, label, trailing word."]
#[doc = r""]
#[doc = r" Shared by [`Checkbox`] and [`Toggle`]: the only difference is the marker,"]
#[doc = r" which each control supplies as a closure over its own glyph roles."]
struct FlagRow<'r> {
    id: Id,
    ov: PartStyle<'r>,
    label: &'r str,
    marker_w: u16,
    trailing: Option<&'r str>,
}

impl FlagRow<'_> {
    #[doc = r" Paint the row and return it."]
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

#[doc = r" A one-row checkbox: `[✓]` / `[ ]`, a label, and the caller's `bool`."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `Checkbox::new(id, label)`. The controlled flag is passed per phase:"]
#[doc = r" `&mut bool` to `update`, `.checked(bool)` for `draw`."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" Stateless (`State = ()`): the flag is the caller's, the runtime owns"]
#[doc = r" focus, hover and press."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.checked(bool)` (draw; `false`), `.disabled(bool)`, `.read_only(bool)`,"]
#[doc = r" `.patch`, `.patch_part`, `.slot`."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::CHOICE`, `DEFAULT` only."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED`, `PRESSED` from the runtime;"]
#[doc = r" `CHECKED | SELECTED` from the flag; `READ_ONLY`, `DISABLED` from the props."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" [`Activated`] — the flag was flipped through the `&mut bool` (§6.1: the"]
#[doc = r" activation action of a button-like control)."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" One `Focusable` stop (`FocusableReadOnly` / `Disabled`); does not"]
#[doc = r" swallow typing."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" `Space` (visible) and `Enter` toggle."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" `PartRef::of(Part::CONTAINER)`: a click toggles."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" One row: gutter, a three-column marker, one space, the label. `measure`"]
#[doc = r" is the natural width by one row; `draw` returns the row; `0×0` registers"]
#[doc = r" nothing (R5)."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `CONTAINER` (the row fill), `GUTTER` (the focus bar), `MARKER` (the box),"]
#[doc = r" `LABEL`."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch`, `.patch_part`, `.slot` on `GUTTER`, `MARKER` and `LABEL`."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" One `Id`; no items."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `CheckboxCase` with `ACTIVATES | FOCUSABLE | DISABLEABLE`;"]
#[doc = r" `render::components::checkbox::*`."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" `draw` never writes the flag (it takes `&self` and a `bool` prop); the"]
#[doc = r" marker is [`GlyphRole::CheckboxOn`] / [`GlyphRole::CheckboxOff`], so the"]
#[doc = r" off state is a glyph and survives `ColorLevel::Mono`."]
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
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::MARKER, Part::LABEL];

    #[doc = r" Columns the marker occupies."]
    const MARKER_W: u16 = 3;

    #[doc = r" A checkbox."]
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

    #[doc = r" The controlled flag, for `draw`."]
    #[must_use]
    pub const fn checked(mut self, yes: bool) -> Self {
        self.checked = yes;
        self
    }

    #[doc = r" Read-only: stays in the ring, never toggles."]
    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    #[doc = r" Disabled: registered, never reachable."]
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[doc = r" An instance patch over every part."]
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[doc = r" Per-part instance patches."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" Replace one part's painting."]
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    #[doc = r" Showcase / fixture use only (A11)."]
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

    #[doc = r" The update phase: `Space` / `Enter` / a click flip `value`."]
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

    #[doc = r" The natural width: gutter, marker, space, label."]
    fn natural_width(&self) -> u16 {
        Self::MARKER_W
            .saturating_add(2)
            .saturating_add(width(self.label))
    }

    #[doc = r" The draw phase."]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }

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
                let g = if on {
                    GlyphRole::CheckboxOn
                } else {
                    GlyphRole::CheckboxOff
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

    #[doc = r" One row, the marker plus the label."]
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

#[doc = r" A one-row switch: a knob on a two-cell track, a label and an `on` / `off`"]
#[doc = r" word."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `Toggle::new(id, label)`. The controlled flag is passed per phase:"]
#[doc = r" `&mut bool` to `update`, `.on(bool)` for `draw`."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" Stateless (`State = ()`): the flag is the caller's."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.on(bool)` (draw; `false`), `.disabled(bool)`, `.read_only(bool)`,"]
#[doc = r" `.patch`, `.patch_part`, `.slot`."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::CHOICE`, `DEFAULT` only."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED`, `PRESSED` from the runtime;"]
#[doc = r" `CHECKED | SELECTED` from the flag; `READ_ONLY`, `DISABLED` from the props."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" [`Activated`] — the flag was flipped through the `&mut bool`."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" One `Focusable` stop (`FocusableReadOnly` / `Disabled`); does not"]
#[doc = r" swallow typing."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" `Space` (visible) and `Enter` toggle."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" `PartRef::of(Part::CONTAINER)`: a click toggles."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" One row: gutter, a three-column switch, one space, the label, then the"]
#[doc = r" `on` / `off` word when the row is wide enough. `measure` is the natural"]
#[doc = r" width by one row; `0×0` registers nothing (R5)."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `CONTAINER`, `GUTTER`, `MARKER` (the switch), `LABEL`, `META` (the"]
#[doc = r" `on` / `off` word)."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch`, `.patch_part`, `.slot` on `GUTTER`, `MARKER` and `LABEL`."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" One `Id`; no items."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `ToggleCase` with `ACTIVATES | FOCUSABLE | DISABLEABLE`;"]
#[doc = r" `render::components::toggle::*`."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" `draw` never writes the flag; the switch is [`GlyphRole::SwitchKnob`] on"]
#[doc = r" a [`GlyphRole::RuleQuiet`] track and the state word is text, so both"]
#[doc = r" halves survive `ColorLevel::Mono`."]
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
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::MARKER,
        Part::LABEL,
        Part::META,
    ];

    #[doc = r" Columns the switch occupies."]
    const MARKER_W: u16 = 3;

    #[doc = r" A toggle."]
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

    #[doc = r" The controlled flag, for `draw`."]
    #[must_use]
    pub const fn on(mut self, yes: bool) -> Self {
        self.on = yes;
        self
    }

    #[doc = r" Read-only: stays in the ring, never toggles."]
    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    #[doc = r" Disabled: registered, never reachable."]
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[doc = r" An instance patch over every part."]
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[doc = r" Per-part instance patches."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" Replace one part's painting."]
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    #[doc = r" Showcase / fixture use only (A11)."]
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

    #[doc = r" The update phase: `Space` / `Enter` / a click flip `value`."]
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

    #[doc = r" The draw phase."]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }

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
        FlagRow {
            id: self.id,
            ov: self.ov,
            label: self.label,
            marker_w: Self::MARKER_W,
            trailing: Some(if on { "on" } else { "off" }),
        }
        .draw(
            ui,
            area,
            live,
            &move |ui: &mut Ui<'_>, cell: Rect, style| {
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
                ui.glyph(cell_at(cell, knob), GlyphRole::SwitchKnob, style);
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

    #[doc = r" One row, the switch plus the label and the state word."]
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

#[doc = r" The default instantiation a form field holds (§15.1, §24 M3): options"]
#[doc = r" are `&str` labels, keyed positionally, painted through `Display`."]
pub type LabelRadio<'a> = RadioGroup<'a, &'a str, ByIndex, DefaultRow>;

#[doc = r" Durable state of a [`RadioGroup`]: the **cursor** and the reconcile"]
#[doc = r" stamp. The value is the caller's, supplied to both phases through"]
#[doc = r" `.value(ItemKey)` and written by the caller when"]
#[doc = r" [`RadioGroupAction::Chose`] arrives (§15, §20.10 item 3)."]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RadioGroupState {
    core: CollectionCore,
}

impl RadioGroupState {
    #[doc = r" The cursor key."]
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    #[doc = r" The cursor's index as of the last reconcile."]
    pub const fn cursor_index(&self) -> usize {
        self.core.cursor_index()
    }

    #[doc = r" Point the cursor at `(index, key)`."]
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

#[doc = r" A vertical radio group over borrowed options, with the **cursor"]
#[doc = r" separated from the value**."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `RadioGroup::new(id)`; options are passed to each phase, never held"]
#[doc = r" (§21 item 1). The value is a controlled prop (`.value(ItemKey)`), written by"]
#[doc = r" the caller when [`RadioGroupAction::Chose`] arrives."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" The caller owns the options (`&[T]` per phase), the value and a"]
#[doc = r" [`RadioGroupState`] (the cursor). The runtime owns focus, hover and"]
#[doc = r" press."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.key(Fn(&T) -> ItemKey)` (`ByIndex`, unstable under reorder),"]
#[doc = r" `.row(Fn(&T, &mut RowUi))` (`DefaultRow`: `Display`), `.value(ItemKey)`"]
#[doc = r" (update and draw), `.read_only(bool)`, `.disabled(bool)`, `.patch`, `.patch_part`,"]
#[doc = r" `.slot`."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::CHOICE`, `DEFAULT` only."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" The group wears `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED`, `PRESSED` from"]
#[doc = r" the runtime and passes them to the **cursor** row only; the value row"]
#[doc = r" wears `SELECTED`; `READ_ONLY` and `DISABLED` reach every row."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" [`RadioGroupAction::Chose(k)`](RadioGroupAction::Chose) — `Space`,"]
#[doc = r" `Enter` or a click committed option `k`. **Moving the cursor emits no"]
#[doc = r" action**: this is the intentional change from the legacy fused"]
#[doc = r" cursor-is-value behaviour (§20.10 item 3), so arrowing through a group"]
#[doc = r" no longer fires a change per row."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" One `Focusable` stop for the whole group (`FocusableReadOnly` /"]
#[doc = r" `Disabled`); does not swallow typing. Option rows are click targets, not"]
#[doc = r" focus stops."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" `↑`/`k`, `↓`/`j` move the cursor; `Home`/`End` jump; `Space` (visible)"]
#[doc = r" and `Enter` commit the cursor option."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" `PartRef::item(Part::ROW, k)`: a press moves the cursor, a click commits"]
#[doc = r" option `k`."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" One row per option: gutter, a three-column marker, one space, the row"]
#[doc = r" renderer's content. `measure` is `(16…, options)`; `draw` returns the"]
#[doc = r" rows it used; `0×0` registers nothing (R5)."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `CONTAINER` (the row fill), `GUTTER` (the focus bar), `MARKER` (the"]
#[doc = r" radio), `LABEL` (through [`RowUi`])."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch`, `.patch_part`, `.slot` on `GUTTER` and `MARKER`."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" `.key` supplies stable keys; `ByIndex` is unstable under"]
#[doc = r" insert/remove/reorder. The action carries an `ItemKey`, never an index."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `RadioGroupCase` with `ACTIVATES | FOCUSABLE | COLLECTION |"]
#[doc = r" DISABLEABLE | SELECTS`; `render::components::radio_group::*`;"]
#[doc = r" `choice::radio_group_separates_cursor_from_value`."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" `reconcile` runs before any action is emitted; the marker is"]
#[doc = r" [`GlyphRole::RadioOn`] / [`GlyphRole::RadioOff`], so the unselected half"]
#[doc = r" is a glyph rather than the absence of colour; only visible rows invoke"]
#[doc = r" the renderer."]
pub struct RadioGroup<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    value: Option<ItemKey>,
    read_only: bool,
    disabled: bool,
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
            .finish_non_exhaustive()
    }
}

impl<T> RadioGroup<'_, T, ByIndex, DefaultRow> {
    #[doc = r" A radio group keyed by index and painted through `Display`."]
    pub const fn new(id: Id) -> Self {
        RadioGroup {
            id,
            key: ByIndex,
            row: DefaultRow,
            value: None,
            read_only: false,
            disabled: false,
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
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::MARKER, Part::LABEL];

    #[doc = r" Columns the marker occupies."]
    const MARKER_W: u16 = 3;

    #[doc = r" The id."]
    pub const fn id(&self) -> Id {
        self.id
    }

    #[doc = r" A stable key accessor."]
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> RadioGroup<'a, T, K2, R> {
        RadioGroup {
            id: self.id,
            key: k,
            row: self.row,
            value: self.value,
            read_only: self.read_only,
            disabled: self.disabled,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    #[doc = r" A row painter."]
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> RadioGroup<'a, T, K, R2> {
        RadioGroup {
            id: self.id,
            key: self.key,
            row: r,
            value: self.value,
            read_only: self.read_only,
            disabled: self.disabled,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    #[doc = r" The controlled value, for `update` and `draw`."]
    #[doc = r""]
    #[doc = r" The value is **not** state: `update` never writes it, it reports"]
    #[doc = r" [`RadioGroupAction::Chose`] and the caller writes its own field —"]
    #[doc = r" the controlled-value convention of §13, with the write happening in"]
    #[doc = r" the action handler rather than through a `&mut` parameter, because a"]
    #[doc = r" group's value is an `ItemKey` chosen from the items the phase call"]
    #[doc = r" already carries."]
    #[must_use]
    pub const fn value(mut self, k: ItemKey) -> Self {
        self.value = Some(k);
        self
    }

    #[doc = r" Read-only: stays in the ring, never commits."]
    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    #[doc = r" Disabled: registered, never reachable."]
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[doc = r" An instance patch over every part."]
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[doc = r" Per-part instance patches."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" Replace one part's painting."]
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    #[doc = r" Showcase / fixture use only (A11)."]
    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> RadioGroup<'_, T, K, R> {
    fn key_at(&self, items: &[T], i: usize) -> ItemKey {
        items
            .get(i)
            .map_or(ItemKey::index(i), |it| self.key.key(it, i))
    }

    fn index_of(&self, items: &[T], key: ItemKey, hint: Option<usize>) -> Option<usize> {
        if let Some(h) = hint
            && h < items.len()
            && self.key_at(items, h) == key
        {
            return Some(h);
        }
        (0..items.len()).find(|&i| self.key_at(items, i) == key)
    }

    fn move_cursor(
        &self,
        st: &mut RadioGroupState,
        items: &[T],
        to: usize,
        acc: &mut Acc<RadioGroupAction>,
    ) {
        if items.is_empty() {
            acc.consumed();
            return;
        }
        let to = to.min(items.len().saturating_sub(1));
        let key = self.key_at(items, to);
        st.core.set_cursor(to, key);

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
        let key = self.key_at(items, i);
        st.core.set_cursor(i, key);
        acc.action(RadioGroupAction::Chose(key));
    }

    #[doc = r" The update phase: reconcile when enabled, then move the cursor or"]
    #[doc = r" commit it."]
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut RadioGroupState,
        items: &[T],
    ) -> Response<RadioGroupAction> {
        let can = self.editable();
        let len = items.len();
        if !self.disabled {
            let _ = st.core.reconcile(len, |i| self.key_at(items, i));
            if st.core.cursor().is_none() && len > 0 {
                let i = self
                    .value
                    .and_then(|v| self.index_of(items, v, None))
                    .unwrap_or(0);
                let key = self.key_at(items, i);
                st.core.set_cursor(i, key);
            }
        }
        let mut acc = Acc::<RadioGroupAction>::new();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) if can => {
                    let cur = st.core.cursor_index();
                    match Binding::command(RADIO, action) {
                        Some(ChoiceCmd::Prev) => {
                            self.move_cursor(st, items, cur.saturating_sub(1), &mut acc);
                        }
                        Some(ChoiceCmd::Next) => {
                            self.move_cursor(st, items, cur.saturating_add(1), &mut acc);
                        }
                        Some(ChoiceCmd::First) => self.move_cursor(st, items, 0, &mut acc),
                        Some(ChoiceCmd::Last) => {
                            self.move_cursor(st, items, usize::MAX, &mut acc);
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
                    let Some(i) = self.index_of(items, k, Some(st.core.cursor_index())) else {
                        acc.consumed();
                        continue;
                    };
                    match phase {
                        Phase::Press => self.move_cursor(st, items, i, &mut acc),
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

    #[doc = r" The rect the group paints into: one row per option that fits `area`."]
    fn used_rect(area: Rect, len: usize) -> Rect {
        let rows = usize::from(area.height).min(len);
        Rect {
            height: rows.min(usize::from(u16::MAX)) as u16,
            ..area
        }
    }

    #[doc = r" Registers the group as one control over `used`."]
    #[doc = r""]
    #[doc = r" A reference rendering registers nothing, so it cannot take focus from"]
    #[doc = r" the live frame."]
    fn register(&self, ui: &mut Ui<'_>, used: Rect) {
        if ui.is_inert() {
            return;
        }
        let f = if self.disabled {
            Focusability::Disabled
        } else if self.read_only {
            Focusability::FocusableReadOnly
        } else {
            Focusability::Focusable
        };
        ui.register_control(self.id, used, f);
    }

    #[doc = r" The state flags row `i` paints with, and whether that row carries the"]
    #[doc = r" value."]
    #[doc = r""]
    #[doc = r" Runtime state may style the actual cursor and value rows, but never"]
    #[doc = r" supplies either semantic identity."]
    fn row_flags(
        &self,
        live: StateFlags,
        cursor: Option<ItemKey>,
        key: ItemKey,
        hovered: bool,
        pressed: bool,
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
        if self.disabled || live.contains(StateFlags::DISABLED) {
            flags |= StateFlags::DISABLED;
            flags = flags.difference(StateFlags::PRESSED | StateFlags::HOVERED);
        }
        (flags, on)
    }

    #[doc = r" Paints the gutter column of one option row."]
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

    #[doc = r" Paints the radio marker of one option row."]
    fn paint_marker(&self, ui: &mut Ui<'_>, cell: Rect, flags: StateFlags, on: bool) {
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
        let g = if on {
            GlyphRole::RadioOn
        } else {
            GlyphRole::RadioOff
        };
        ui.glyph(cell, g, ms.style);
    }

    #[doc = r" Paints one option row: the container surface, the gutter, the marker"]
    #[doc = r" and the caller's row body."]
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
        self.paint_marker(ui, marker_cell, flags, on);
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
            let mut r = RowUi::new(
                ui,
                self.id,
                Family::CHOICE,
                Variant::DEFAULT,
                flags,
                key,
                rest,
            );
            self.row.row(item, &mut r);
        }
    }

    #[doc = r" The draw phase: one row per option."]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &RadioGroupState, items: &[T]) -> Rect {
        if area.is_empty() {
            return area;
        }
        let used = Self::used_rect(area, items.len());
        if used.is_empty() {
            return used;
        }
        self.register(ui, used);

        let live = PartStyle::flags(
            crate::ui::FrameRead::state(ui, self.id),
            StateFlags::empty(),
        );
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, RADIO);
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
            let (flags, on) = self.row_flags(
                live,
                cursor,
                key,
                ui.hovered_part(self.id) == Some(part),
                ui.pressed_part(self.id) == Some(part),
            );
            self.paint_row(ui, row, item, key, flags, on);
            if !ui.is_inert() {
                ui.register_part(self.id, PartRef::item(Part::ROW, key), row);
            }
        }
        used
    }

    #[doc = r" The natural size: sixteen columns by one row per option."]
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (16, 1),
            preferred: (24, c.max.1.max(1)),
        }
        .fit(c)
    }
}

impl<T, K, R> Bindings for RadioGroup<'_, T, K, R> {
    type Cmd = ChoiceCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<ChoiceCmd>] {
        RADIO
    }
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

    #[doc = r" A disabled collection remains drawable from its current item slice,"]
    #[doc = r" but its update phase must not initialize or reconcile persistent state."]
    #[test]
    fn disabled_update_does_not_initialize_collection_state() {
        let mut runtime = Runtime::new(DisabledRadioApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        assert_eq!(runtime.app().state, RadioGroupState::default());
    }

    #[doc = r" §16.1 / §20.10 item 3: arrows move the cursor and commit nothing; the"]
    #[doc = r" value changes only on `Space` / `Enter` / a click, and it is the"]
    #[doc = r" caller's field, never state."]
    #[test]
    fn radio_group_separates_cursor_from_value() {
        let items = ["a", "b", "c"];
        let mut st = RadioGroupState::default();
        let g: RadioGroup<'_, &str> = RadioGroup::new(RG);
        let mut acc = Acc::<RadioGroupAction>::new();
        let _ = st.core.reconcile(3, |i| g.key_at(&items, i));
        st.set_cursor(0, ItemKey::index(0));
        g.move_cursor(&mut st, &items, 1, &mut acc);
        g.move_cursor(&mut st, &items, 2, &mut acc);
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

        let mut fresh = RadioGroupState::default();
        let valued: RadioGroup<'_, &str> = RadioGroup::new(RG).value(ItemKey::index(1));
        let _ = fresh.core.reconcile(3, |i| valued.key_at(&items, i));
        assert!(fresh.cursor().is_none());
        assert_eq!(valued.index_of(&items, ItemKey::index(1), None), Some(1));
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
                .row_flags(StateFlags::empty(), state.cursor(), selected, false, false)
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
                    .row_flags(StateFlags::empty(), None, key, false, false)
                    .1,
                "a missing controlled value must not select a fallback row"
            );
        }

        let vanished_items = ["a"];
        let vanished_group: RadioGroup<'_, &str> = RadioGroup::new(RG).value(ItemKey::index(1));
        let only_key = vanished_group.key_at(&vanished_items, 0);
        assert!(
            !vanished_group
                .row_flags(StateFlags::empty(), Some(only_key), only_key, false, false)
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
                false
            ),
            (StateFlags::empty(), false)
        );

        let controlled = radio.value(selected);
        let (flags, on) = controlled.row_flags(StateFlags::empty(), None, selected, false, false);
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

    #[doc = r" The marker is a glyph pair, so an unselected option is visible"]
    #[doc = r" without colour (§11.4)."]
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
}
