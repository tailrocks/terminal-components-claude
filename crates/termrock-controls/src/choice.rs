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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct InheritedFormState {
    pub disabled: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RadioGroupAction {
    Chose(ItemKey),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RadioNavigation {
    #[default]
    Immediate,

    Deferred,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum RadioKeyPolicy {
    #[default]
    Default,

    Desktop,

    Vim,

    NoHomeEnd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum CheckboxState {
    #[default]
    Unchecked,

    Checked,

    Mixed,
}

impl CheckboxState {
    const fn after_activation(self) -> Self {
        match self {
            Self::Checked => Self::Unchecked,
            Self::Unchecked | Self::Mixed => Self::Checked,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChoiceCmd {
    Choose,
    Prev,
    Next,
    First,
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

const fn action_without_default_chord(cmd: ChoiceCmd, label: &'static str) -> Binding<ChoiceCmd> {
    Binding {
        action: ActionKey::custom(label),
        chord: None,
        cmd,
        label,
        priority: 10,
        visible: false,
    }
}

const RADIO_DESKTOP: &[Binding<ChoiceCmd>] = &[
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
    b(Chord::key(KeyCode::Home), ChoiceCmd::First, "First", false),
    b(Chord::key(KeyCode::End), ChoiceCmd::Last, "Last", false),
];

const RADIO_VIM: &[Binding<ChoiceCmd>] = &[
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
        "Up (k)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('K')),
        ChoiceCmd::Prev,
        "Up (K)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('j')),
        ChoiceCmd::Next,
        "Down (j)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('J')),
        ChoiceCmd::Next,
        "Down (J)",
        false,
    ),
    b(
        Chord::key(KeyCode::Home),
        ChoiceCmd::First,
        "First (Home)",
        false,
    ),
    b(
        Chord::key(KeyCode::End),
        ChoiceCmd::Last,
        "Last (End)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('g')),
        ChoiceCmd::First,
        "First (g)",
        true,
    ),
    b(
        Chord::key(KeyCode::Char('G')),
        ChoiceCmd::Last,
        "Last (G)",
        true,
    ),
    action_without_default_chord(ChoiceCmd::First, "First"),
    action_without_default_chord(ChoiceCmd::Last, "Last"),
];

const RADIO_NO_HOME_END: &[Binding<ChoiceCmd>] = &[
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
    action_without_default_chord(ChoiceCmd::First, "First"),
    action_without_default_chord(ChoiceCmd::Last, "Last"),
];

impl RadioKeyPolicy {
    const fn bindings(self) -> &'static [Binding<ChoiceCmd>] {
        match self {
            Self::Default => RADIO,
            Self::Desktop => RADIO_DESKTOP,
            Self::Vim => RADIO_VIM,
            Self::NoHomeEnd => RADIO_NO_HOME_END,
        }
    }
}

struct FlagRow<'r> {
    id: Id,
    ov: PartStyle<'r>,
    label: &'r str,
    marker_w: u16,
    trailing: Option<&'r str>,
}

impl FlagRow<'_> {
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

pub struct Checkbox<'a> {
    id: Id,
    label: &'a str,
    checked: bool,
    mixed: bool,
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
            .field("mixed", &self.mixed)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .finish_non_exhaustive()
    }
}

impl<'a> Checkbox<'a> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::MARKER, Part::LABEL];

    const MARKER_W: u16 = 3;

    pub const fn new(id: Id, label: &'a str) -> Self {
        Checkbox {
            id,
            label,
            checked: false,
            mixed: false,
            read_only: false,
            disabled: false,
            ov: PartStyle::new(),
        }
    }

    #[must_use]
    pub const fn checked(mut self, yes: bool) -> Self {
        self.checked = yes;
        self.mixed = false;
        self
    }

    #[must_use]
    pub const fn state(mut self, state: CheckboxState) -> Self {
        match state {
            CheckboxState::Unchecked => {
                self.checked = false;
                self.mixed = false;
            }
            CheckboxState::Checked => {
                self.checked = true;
                self.mixed = false;
            }
            CheckboxState::Mixed => {
                self.checked = false;
                self.mixed = true;
            }
        }
        self
    }

    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    const fn form_focusability(&self, inherited_disabled: bool) -> Focusability {
        if self.disabled || inherited_disabled {
            Focusability::Disabled
        } else if self.read_only {
            Focusability::FocusableReadOnly
        } else {
            Focusability::Focusable
        }
    }

    pub fn register_focus_only_in_form(&self, ui: &mut Ui<'_>, inherited_disabled: bool) {
        if !ui.is_inert() {
            ui.register_focus_only(self.id, self.form_focusability(inherited_disabled));
        }
    }

    const fn with_inherited_disabled(&self, inherited: bool) -> Self {
        Checkbox {
            id: self.id,
            label: self.label,
            checked: self.checked,
            mixed: self.mixed,
            read_only: self.read_only,
            disabled: self.disabled || inherited,
            ov: self.ov,
        }
    }

    fn update_with(&self, cx: &mut Cx<'_>, mut activate: impl FnMut()) -> Response<Activated> {
        let mut acc = Acc::<Activated>::new();
        let can = self.editable();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) if can => {
                    if Binding::command(FLAG, action).is_some() {
                        activate();
                        acc.action(Activated);
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    ..
                } if can => {
                    activate();
                    acc.action(Activated);
                }
                Intent::Pointer { .. } => acc.consumed(),
                _ => {}
            }
        }
        acc.finish(self.id)
    }

    pub fn update(&self, cx: &mut Cx<'_>, value: &mut bool) -> Response<Activated> {
        self.update_with(cx, || *value = !*value)
    }

    pub fn update_state(&self, cx: &mut Cx<'_>, value: &mut CheckboxState) -> Response<Activated> {
        self.update_with(cx, || *value = (*value).after_activation())
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

    pub fn update_state_in_form(
        &self,
        cx: &mut Cx<'_>,
        value: &mut CheckboxState,
        inherited_disabled: bool,
    ) -> Response<Activated> {
        self.with_inherited_disabled(inherited_disabled)
            .update_state(cx, value)
    }

    fn natural_width(&self) -> u16 {
        Self::MARKER_W
            .saturating_add(2)
            .saturating_add(width(self.label))
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }

        let mut derived = StateFlags::empty();
        if self.checked || self.mixed {
            derived |= StateFlags::CHECKED | StateFlags::SELECTED;
        }
        if self.read_only {
            derived |= StateFlags::READ_ONLY;
        }
        if self.disabled {
            derived |= StateFlags::DISABLED;
        }
        let mut live = PartStyle::flags(crate::ui::FrameRead::state(ui, self.id), derived);
        if !(self.checked || self.mixed) {
            live = live.difference(StateFlags::SELECTED);
        }
        if self.disabled {
            live = live.difference(StateFlags::HOVERED | StateFlags::PRESSED);
        }
        if !ui.is_inert() {
            let f = self.form_focusability(false);
            ui.register_control(self.id, area, f);
            ui.publish_bindings(self.id, live, FLAG);
        }
        let on = live.contains(StateFlags::CHECKED);
        let mixed = self.mixed;
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
                if mixed {
                    let mark = if cell.width >= 3 { "[-]" } else { "-" };
                    ui.paint_str(cell, mark, style);
                } else {
                    let g = if on {
                        GlyphRole::CheckboxOn
                    } else {
                        GlyphRole::CheckboxOff
                    };
                    ui.glyph(cell, g, style);
                }
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

    pub fn draw_state_in_form(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        value: CheckboxState,
        inherited_disabled: bool,
    ) -> Rect {
        self.with_inherited_disabled(inherited_disabled)
            .state(value)
            .draw(ui, area)
    }

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
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::MARKER,
        Part::LABEL,
        Part::META,
    ];

    const MARKER_W: u16 = 3;

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

    #[must_use]
    pub const fn on(mut self, yes: bool) -> Self {
        self.on = yes;
        self
    }

    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    const fn form_focusability(&self, inherited_disabled: bool) -> Focusability {
        if self.disabled || inherited_disabled {
            Focusability::Disabled
        } else if self.read_only {
            Focusability::FocusableReadOnly
        } else {
            Focusability::Focusable
        }
    }

    pub fn register_focus_only_in_form(&self, ui: &mut Ui<'_>, inherited_disabled: bool) {
        if !ui.is_inert() {
            ui.register_focus_only(self.id, self.form_focusability(inherited_disabled));
        }
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
            let f = self.form_focusability(false);
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

pub type LabelRadio<'a> = RadioGroup<'a, &'a str, ByIndex, DefaultRow>;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RadioGroupState {
    core: CollectionCore,
}

impl RadioGroupState {
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    pub const fn cursor_index(&self) -> usize {
        self.core.cursor_index()
    }

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

pub struct RadioGroup<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    value: Option<ItemKey>,
    navigation: RadioNavigation,
    key_policy: RadioKeyPolicy,
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
            .field("navigation", &self.navigation)
            .field("key_policy", &self.key_policy)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .finish_non_exhaustive()
    }
}

impl<T> RadioGroup<'_, T, ByIndex, DefaultRow> {
    pub const fn new(id: Id) -> Self {
        RadioGroup {
            id,
            key: ByIndex,
            row: DefaultRow,
            value: None,
            navigation: RadioNavigation::Immediate,
            key_policy: RadioKeyPolicy::Default,
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
            navigation: self.navigation,
            key_policy: self.key_policy,
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

    #[allow(clippy::too_many_arguments)]
    pub fn draw_in_form_window(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &RadioGroupState,
        value: usize,
        items: &[&'a str],
        inherited: InheritedFormState,
        first_row: usize,
    ) -> Rect {
        self.with_form_value(value, inherited.disabled)
            .draw_window(ui, area, st, items, first_row)
    }

    pub fn register_focus_only_in_form(&self, ui: &mut Ui<'_>, inherited_disabled: bool) {
        if !ui.is_inert() {
            ui.register_focus_only(self.id, self.form_focusability(inherited_disabled));
        }
    }
}

impl<'a, T, K, R> RadioGroup<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::MARKER, Part::LABEL];

    const MARKER_W: u16 = 3;

    pub const fn id(&self) -> Id {
        self.id
    }

    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> RadioGroup<'a, T, K2, R> {
        RadioGroup {
            id: self.id,
            key: k,
            row: self.row,
            value: self.value,
            navigation: self.navigation,
            key_policy: self.key_policy,
            read_only: self.read_only,
            disabled: self.disabled,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> RadioGroup<'a, T, K, R2> {
        RadioGroup {
            id: self.id,
            key: self.key,
            row: r,
            value: self.value,
            navigation: self.navigation,
            key_policy: self.key_policy,
            read_only: self.read_only,
            disabled: self.disabled,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    #[must_use]
    pub const fn value(mut self, k: ItemKey) -> Self {
        self.value = Some(k);
        self
    }

    #[must_use]
    pub const fn navigation(mut self, policy: RadioNavigation) -> Self {
        self.navigation = policy;
        self
    }

    #[must_use]
    pub const fn key_policy(mut self, policy: RadioKeyPolicy) -> Self {
        self.key_policy = policy;
        self
    }

    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

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

    fn navigate(
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
        let previous = st.core.cursor();
        st.core.set_cursor(to, key);
        if self.navigation == RadioNavigation::Immediate && self.value != Some(key) {
            acc.action(RadioGroupAction::Chose(key));
        } else if previous == Some(key) {
            acc.consumed();
        } else {
            acc.changed();
        }
    }

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
        let bindings = self.key_policy.bindings();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) if can => {
                    let cur = st.core.cursor_index();
                    match Binding::command(bindings, action) {
                        Some(ChoiceCmd::Prev) => {
                            self.navigate(st, items, cur.saturating_sub(1), &mut acc);
                        }
                        Some(ChoiceCmd::Next) => {
                            self.navigate(st, items, cur.saturating_add(1), &mut acc);
                        }
                        Some(ChoiceCmd::First) => self.navigate(st, items, 0, &mut acc),
                        Some(ChoiceCmd::Last) => {
                            self.navigate(st, items, usize::MAX, &mut acc);
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

    fn used_rect(area: Rect, len: usize) -> Rect {
        let rows = usize::from(area.height).min(len);
        Rect {
            height: rows.min(usize::from(u16::MAX)) as u16,
            ..area
        }
    }

    const fn form_focusability(&self, inherited_disabled: bool) -> Focusability {
        if self.disabled || inherited_disabled {
            Focusability::Disabled
        } else if self.read_only {
            Focusability::FocusableReadOnly
        } else {
            Focusability::Focusable
        }
    }

    fn register(&self, ui: &mut Ui<'_>, used: Rect) {
        if ui.is_inert() {
            return;
        }
        ui.register_control(self.id, used, self.form_focusability(false));
    }

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

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &RadioGroupState, items: &[T]) -> Rect {
        self.draw_window(ui, area, st, items, 0)
    }

    pub fn draw_window(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &RadioGroupState,
        items: &[T],
        first_row: usize,
    ) -> Rect {
        if area.is_empty() {
            return area;
        }
        let first_row = first_row.min(items.len());
        let used = Self::used_rect(area, items.len().saturating_sub(first_row));
        if used.is_empty() {
            return used;
        }
        self.register(ui, used);

        let live = PartStyle::flags(
            crate::ui::FrameRead::state(ui, self.id),
            StateFlags::empty(),
        );
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, self.key_policy.bindings());
        }
        let cursor = st.core.cursor();
        for (i, item) in items
            .iter()
            .enumerate()
            .skip(first_row)
            .take(usize::from(used.height))
        {
            let key = self.key.key(item, i);
            let row = Rect {
                x: used.x,
                y: used
                    .y
                    .saturating_add(i.saturating_sub(first_row).min(usize::from(u16::MAX)) as u16),
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
        self.key_policy.bindings()
    }
}
