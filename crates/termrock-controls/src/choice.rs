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

/// Controls whether keyboard navigation also chooses the focused radio option.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RadioNavigation {
    /// Arrow, Home, and End navigation immediately choose the destination.
    #[default]
    Immediate,
    /// Navigation moves the cursor; Space or Enter chooses the destination.
    Deferred,
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
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::MARKER, Part::LABEL];

    const MARKER_W: u16 = 3;

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

    #[must_use]
    pub const fn checked(mut self, yes: bool) -> Self {
        self.checked = yes;
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
    }

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

    /// Draws a clipped window while keeping original option indices as keys.
    /// Update and input handling must continue to receive the full `items`.
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
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) if can => {
                    let cur = st.core.cursor_index();
                    match Binding::command(RADIO, action) {
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

    /// Draws a visible row window from the original option collection.
    /// Keys and state continue to use the original collection indexes.
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
            ui.publish_bindings(self.id, live, RADIO);
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
        RADIO
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Input, MouseKind};
    use crate::runtime::stub::{SCREEN, Stub, key, mouse};
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

    #[derive(Clone)]
    struct ControlledRadioApp {
        state: RadioGroupState,
        value: ItemKey,
        navigation: RadioNavigation,
    }

    impl App for ControlledRadioApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let items = ["a", "b", "c"];
            let response = RadioGroup::new(RG)
                .value(self.value)
                .navigation(self.navigation)
                .update(cx, &mut self.state, &items);
            if let Some(RadioGroupAction::Chose(key)) = response.action_ref() {
                self.value = *key;
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let items = ["a", "b", "c"];
            RadioGroup::new(RG)
                .value(self.value)
                .navigation(self.navigation)
                .draw(ui, SCREEN, &self.state, &items);
        }
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
    fn a_zero_offset_row_window_matches_the_legacy_radio_draw() {
        let items = ["alpha", "beta", "gamma"];
        let state = RadioGroupState::default();
        let area = Rect::new(0, 0, 30, 8);
        let draw = |window: bool| {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(area);
            runtime
                .draw_scene(area, &mut buffer, |ui, area| {
                    let radio = RadioGroup::new(RG);
                    if window {
                        radio.draw_window(ui, area, &state, &items, 0);
                    } else {
                        radio.draw(ui, area, &state, &items);
                    }
                })
                .commit_presented();
            (buffer, runtime.registry().regions().to_vec())
        };

        assert_eq!(draw(false), draw(true));
    }

    #[test]
    fn checked_painting_comes_only_from_the_checkbox_prop() {
        assert_ne!(draw_checkbox(true), draw_checkbox(false));
    }

    #[test]
    fn checked_painting_comes_only_from_the_toggle_prop() {
        assert_ne!(draw_toggle(true), draw_toggle(false));
    }

    #[test]
    fn disabled_update_does_not_initialize_collection_state() {
        let mut runtime = Runtime::new(DisabledRadioApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        assert_eq!(runtime.app().state, RadioGroupState::default());
    }

    #[test]
    fn radio_group_separates_cursor_from_value() {
        let items = ["a", "b", "c"];
        let mut st = RadioGroupState::default();
        let g: RadioGroup<'_, &str> = RadioGroup::new(RG).navigation(RadioNavigation::Deferred);
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
        let valued: RadioGroup<'_, &str> = RadioGroup::new(RG)
            .navigation(RadioNavigation::Deferred)
            .value(ItemKey::index(1));
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
                navigation: RadioNavigation::Immediate,
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
    fn keyboard_navigation_selects_immediately_or_can_be_deferred() {
        fn app_after_down(navigation: RadioNavigation) -> ControlledRadioApp {
            let mut runtime = Runtime::new(
                ControlledRadioApp {
                    state: RadioGroupState::default(),
                    value: ItemKey::index(0),
                    navigation,
                },
                Theme::junie(),
            );
            let _ = runtime.initialize();
            let mut buffer = Buffer::empty(SCREEN);
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
            let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Tab));
            assert_eq!(runtime.focus(), Some(RG));
            let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Down));
            runtime.app().clone()
        }

        let immediate = app_after_down(RadioNavigation::Immediate);
        assert_eq!(immediate.value, ItemKey::index(1));
        assert_eq!(immediate.state.cursor(), Some(ItemKey::index(1)));

        let deferred = app_after_down(RadioNavigation::Deferred);
        assert_eq!(deferred.value, ItemKey::index(0));
        assert_eq!(deferred.state.cursor(), Some(ItemKey::index(1)));
    }

    #[test]
    fn immediate_pointer_press_moves_cursor_without_choosing_until_click() {
        let mut runtime = Runtime::new(
            ControlledRadioApp {
                state: RadioGroupState::default(),
                value: ItemKey::index(0),
                navigation: RadioNavigation::Immediate,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 5, 1));
        assert_eq!(runtime.app().value, ItemKey::index(0));
        assert_eq!(runtime.app().state.cursor(), Some(ItemKey::index(1)));

        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 5, 5));
        assert_eq!(runtime.app().value, ItemKey::index(0));

        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 5, 1));
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 5, 1));
        assert_eq!(runtime.app().value, ItemKey::index(1));
    }

    #[test]
    fn immediate_keyboard_navigation_chooses_cursor_after_cancelled_pointer_press() {
        for (navigation, key_code, expected_value) in [
            (RadioNavigation::Immediate, KeyCode::End, ItemKey::index(2)),
            (RadioNavigation::Immediate, KeyCode::Down, ItemKey::index(2)),
            (RadioNavigation::Deferred, KeyCode::End, ItemKey::index(0)),
            (RadioNavigation::Deferred, KeyCode::Down, ItemKey::index(0)),
        ] {
            let mut runtime = Runtime::new(
                ControlledRadioApp {
                    state: RadioGroupState::default(),
                    value: ItemKey::index(0),
                    navigation,
                },
                Theme::junie(),
            );
            let _ = runtime.initialize();
            let mut buffer = Buffer::empty(SCREEN);
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 5, 2));
            assert_eq!(runtime.app().value, ItemKey::index(0));
            assert_eq!(runtime.app().state.cursor(), Some(ItemKey::index(2)));
            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 5, 5));
            assert_eq!(runtime.app().value, ItemKey::index(0));

            let _ = crate::runtime::stub::deliver(&mut runtime, key(key_code));
            assert_eq!(runtime.app().value, expected_value);
            assert_eq!(runtime.app().state.cursor(), Some(ItemKey::index(2)));
        }
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
