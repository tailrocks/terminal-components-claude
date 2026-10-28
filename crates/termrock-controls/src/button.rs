use core::fmt;
use core::time::Duration;

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn, cell_at, first_row, paint_pressed_bracket, shift};
use crate::action::ActionKey;
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
use termrock_runtime::Status;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonCmd {
    Activate,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonFocusVisibility {
    #[default]
    Always,

    KeyboardOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BusyMarkerPolicy {
    Keep,

    #[default]
    Replace,
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

#[derive(Default)]
struct Autofocus {
    spent: bool,
}

#[derive(Default)]
struct SpinnerPhase {
    frame: usize,
}

pub struct Button<'a> {
    id: Id,
    label: &'a str,
    variant: Variant,
    focus_visibility: ButtonFocusVisibility,
    focusability: Focusability,
    disabled: bool,
    icon: Option<GlyphRole>,
    autofocus: bool,
    status: Status,
    checked: Option<bool>,
    busy_marker: BusyMarkerPolicy,
    ov: PartStyle<'a>,
}

impl fmt::Debug for Button<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Button")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("variant", &self.variant)
            .field("focus_visibility", &self.focus_visibility)
            .field("focusability", &self.focusability)
            .field("disabled", &self.disabled)
            .field("status", &self.status)
            .field("checked", &self.checked)
            .field("busy_marker", &self.busy_marker)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> Button<'a> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::LABEL,
        Part::ICON,
        Part::MARKER,
    ];

    pub const fn new(id: Id, label: &'a str) -> Self {
        Button {
            id,
            label,
            variant: Variant::DEFAULT,
            focus_visibility: ButtonFocusVisibility::Always,
            focusability: Focusability::Focusable,
            disabled: false,
            icon: None,
            autofocus: false,
            status: Status::Ready,
            checked: None,
            busy_marker: BusyMarkerPolicy::Replace,
            ov: PartStyle::new(),
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[must_use]
    pub const fn focus_visible(mut self, visibility: ButtonFocusVisibility) -> Self {
        self.focus_visibility = visibility;
        self
    }

    #[must_use]
    pub const fn focusability(mut self, focusability: Focusability) -> Self {
        self.focusability = focusability;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[must_use]
    pub const fn icon(mut self, g: GlyphRole) -> Self {
        self.icon = Some(g);
        self
    }

    #[must_use]
    pub const fn autofocus(mut self) -> Self {
        self.autofocus = true;
        self
    }

    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    #[must_use]
    pub const fn checked(mut self, on: bool) -> Self {
        self.checked = Some(on);
        self
    }

    #[must_use]
    pub const fn busy_marker_policy(mut self, policy: BusyMarkerPolicy) -> Self {
        self.busy_marker = policy;
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

    const fn busy(&self) -> bool {
        matches!(self.status, Status::Busy | Status::Loading)
    }

    fn effectively_disabled(&self) -> bool {
        self.disabled || self.focusability == Focusability::Disabled
    }

    fn form_focusability(&self, inherited_disabled: bool) -> Focusability {
        let control = self.with_inherited_disabled(inherited_disabled);
        if control.effectively_disabled() {
            Focusability::Disabled
        } else {
            control.focusability
        }
    }

    pub fn register_focus_only_in_form(&self, ui: &mut Ui<'_>, inherited_disabled: bool) {
        if !ui.is_inert() {
            ui.register_focus_only(self.id, self.form_focusability(inherited_disabled));
        }
    }

    fn can_activate(&self) -> bool {
        !self.effectively_disabled() && !self.busy()
    }

    const fn with_inherited_disabled(&self, inherited: bool) -> Self {
        Button {
            id: self.id,
            label: self.label,
            variant: self.variant,
            focus_visibility: self.focus_visibility,
            focusability: self.focusability,
            disabled: self.disabled || inherited,
            icon: self.icon,
            autofocus: self.autofocus,
            status: self.status,
            checked: self.checked,
            busy_marker: self.busy_marker,
            ov: self.ov,
        }
    }

    pub fn update(&self, cx: &mut Cx<'_>) -> Response<Activated> {
        self.autofocus_once(cx);
        let mut r: Response<Activated> = Response::ignored();
        let can = self.can_activate();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) if self.focusability != Focusability::ClickOnly => {
                    if can && Binding::command(BINDINGS, action).is_some() {
                        r = Response::action(Activated);
                    }
                }
                Intent::Binding(_) => {}
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    ..
                } if can => r = Response::action(Activated),

                Intent::Pointer {
                    phase: Phase::Move, ..
                } => {}
                Intent::Pointer { .. } if can && !r.activated() => r = Response::changed(),
                _ => {}
            }
        }

        let tick_ms = cx.design().motion.tick_ms.max(1);
        if self.busy() {
            let frames = cx.design().motion.spinner_frames;
            if !frames.is_empty() {
                let phase = cx.now().as_duration().as_millis() / u128::from(tick_ms);
                let frame = usize::try_from(phase % frames.len() as u128).unwrap_or(0);
                let spinner = cx.cache::<SpinnerPhase>(self.id);
                if spinner.frame != frame {
                    spinner.frame = frame;
                    if !r.activated() {
                        r = Response::changed();
                    }
                }
                let elapsed = cx.now().as_duration().as_millis() % u128::from(tick_ms);
                cx.request_repaint_after(Duration::from_millis(
                    tick_ms.saturating_sub(u64::try_from(elapsed).unwrap_or(0)),
                ));
            }
        }
        r.for_id(self.id)
    }

    fn autofocus_once(&self, cx: &mut Cx<'_>) {
        if !self.autofocus
            || matches!(
                self.focusability,
                Focusability::ClickOnly | Focusability::Disabled
            )
        {
            return;
        }
        let latch = cx.cache::<Autofocus>(self.id);
        if latch.spent {
            return;
        }
        latch.spent = true;
        if !self.effectively_disabled() {
            cx.focus(self.id);
        }
    }

    pub fn update_in_form(&self, cx: &mut Cx<'_>, inherited_disabled: bool) -> Response<Activated> {
        self.with_inherited_disabled(inherited_disabled).update(cx)
    }

    const fn readiness_width(&self) -> u16 {
        if self.icon.is_some() || !matches!(self.status, Status::Ready) {
            2
        } else {
            0
        }
    }

    fn marker_width(&self) -> u16 {
        if self.checked.is_some() && !(self.busy() && self.busy_marker == BusyMarkerPolicy::Replace)
        {
            2
        } else {
            0
        }
    }

    fn natural_width(&self, _ui: &Ui<'_>) -> u16 {
        1u16.saturating_add(self.readiness_width())
            .saturating_add(self.marker_width())
            .saturating_add(width(self.label))
            .saturating_add(1)
    }

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
        let disabled = self.effectively_disabled();
        if !ui.is_inert() {
            ui.register_control(self.id, area, self.form_focusability(false));
        }

        let mut derived = self.status.flags();
        if self.busy() {
            derived |= StateFlags::BUSY;
        }
        if disabled {
            derived |= StateFlags::DISABLED;
        }
        if self.checked == Some(true) {
            derived |= StateFlags::CHECKED | StateFlags::SELECTED;
        }
        let mut live = PartStyle::flags(ui.state(self.id), derived);
        if self.focusability == Focusability::ClickOnly {
            live = live.difference(StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
        }
        if self.checked != Some(true) {
            live = live.difference(StateFlags::SELECTED);
        }
        if disabled {
            live = live.difference(
                StateFlags::HOVERED
                    | StateFlags::PRESSED
                    | StateFlags::FOCUSED
                    | StateFlags::FOCUS_VISIBLE,
            );
        }
        if self.busy() {
            live = live.difference(StateFlags::PRESSED);
        }

        if !ui.is_inert() && !disabled && self.focusability != Focusability::ClickOnly {
            ui.publish_bindings(self.id, live, BINDINGS);
        }
        let mut style_flags = live;
        if self.focus_visibility == ButtonFocusVisibility::Always
            && !ui.is_inert()
            && !disabled
            && !self.busy()
            && style_flags.contains(StateFlags::FOCUSED)
        {
            style_flags |= StateFlags::FOCUS_VISIBLE;
        }
        let ov = self.ov;
        let style = |ui: &mut Ui<'_>, part: Part| {
            ov.style(ui, self.id, Family::BUTTON, self.variant, part, style_flags)
        };
        let container = style(ui, Part::CONTAINER);

        ui.fill(area, container.style);

        let pad = match container.style.as_style().bg {
            Some(bg) => container
                .style
                .fg(bg)
                .remove_modifier(ratatui_core::style::Modifier::BOLD),
            None => container.style,
        };

        let gutter_cell = cell_at(area, area.x);
        if let Some(f) = ov.slot_for(Part::GUTTER) {
            f(ui, gutter_cell);
        } else {
            let g = style(ui, Part::GUTTER);
            match g.glyph {
                Slot::Set(glyph) => {
                    ui.glyph(gutter_cell, glyph, g.style);
                }
                Slot::Inherit => ui.fill(gutter_cell, pad),
                Slot::Clear => match ov.part_patch(Part::GUTTER) {
                    Some(explicit) if explicit.glyph == Slot::Clear => {
                        ui.fill(gutter_cell, g.style);
                        termrock_runtime::author::patch_style(ui, gutter_cell, g.style, explicit);
                    }
                    _ => ui.fill(gutter_cell, pad),
                },
            }
        }

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
                        let frame_index = ui.cache::<SpinnerPhase>(self.id).frame;
                        let frame = frames
                            .get(frame_index % frames.len().max(1))
                            .copied()
                            .unwrap_or("");
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

                    _ => {}
                }
            }
            text = shift(text, 2);
        }
        if let Some(on) = self.checked
            && !(self.busy() && self.busy_marker == BusyMarkerPolicy::Replace)
        {
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
                    Slot::Inherit if on => Some(GlyphRole::DotOn),
                    Slot::Inherit => Some(GlyphRole::DotOff),
                    Slot::Clear => None,
                };
                let marker_style = ms.over(container.style);
                match glyph {
                    Some(g) => {
                        ui.glyph(marker_cell, g, marker_style);
                    }
                    None => ui.fill(marker_cell, marker_style),
                }
            }
            text = shift(text, 2);
        }
        if let Some(f) = ov.slot_for(Part::LABEL) {
            f(ui, text);
        } else {
            let ls = style(ui, Part::LABEL);
            let truncated;
            let label = if crate::text::width(self.label) <= text.width {
                self.label
            } else {
                truncated = crate::text::truncate(self.label, text.width);
                &truncated
            };
            ui.fill(text, ls.style);
            let label_style = ls.over(container.style);
            if matches!(ls.glyph, Slot::Set(GlyphRole::PressLeft)) {
                ui.paint_str(text, label, label_style);
                if area.width >= 2 {
                    paint_pressed_bracket(
                        ui,
                        gutter_cell,
                        cell_at(area, area.right().saturating_sub(1)),
                        label_style,
                    );
                }
            } else {
                ui.paint_str(text, label, label_style);
            }
        }
        area
    }

    pub fn draw_in_form(&self, ui: &mut Ui<'_>, area: Rect, inherited_disabled: bool) -> Rect {
        self.with_inherited_disabled(inherited_disabled)
            .draw(ui, area)
    }

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
