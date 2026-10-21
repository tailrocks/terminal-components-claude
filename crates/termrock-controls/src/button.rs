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

/// Controls whether pointer focus receives the button's visible focus treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonFocusVisibility {
    /// Show the focus treatment whenever this button is focused.
    #[default]
    Always,
    /// Show the focus treatment only when the runtime marks focus as keyboard-visible.
    KeyboardOnly,
}

/// Controls whether a checked marker remains visible beside a busy spinner.
///
/// `Keep` preserves the extended Termrock presentation. `Replace` matches the
/// frozen application contract, where the spinner temporarily owns that lane.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BusyMarkerPolicy {
    Keep,
    /// Frozen-contract default: the spinner lane takes over from the marker.
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

    /// Set whether the button participates in keyboard focus.
    ///
    /// `ClickOnly` keeps the button hit-testable and pointer-activatable while
    /// excluding it from keyboard traversal and keyboard activation. `Disabled`
    /// suppresses keyboard and pointer activation and renders the disabled state.
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
        } else if r.activated() {
            // An activation can synchronously move application data into Busy
            // state; request one tick so the next update observes that state.
            // The observation wake never lands before the press flash ends:
            // `next_deadline` must keep reporting the flash expiry, and the
            // spinner phase it schedules is aligned to `tick_ms` either way.
            cx.request_repaint_after(Duration::from_millis(
                tick_ms.max(cx.design().motion.press_flash_ms),
            ));
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
            let f = if disabled {
                Focusability::Disabled
            } else {
                self.focusability
            };
            ui.register_control(self.id, area, f);
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
        // Bindings consume the actual runtime focus state. The focus-visibility
        // policy below only adapts state for style resolution.
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
            let label: &str = if crate::text::width(self.label) <= text.width {
                // Steady-state redraw of a fitting label must not allocate.
                self.label
            } else {
                truncated = crate::text::truncate(self.label, text.width);
                &truncated
            };
            ui.fill(text, ls.style);
            let label_style = ls.over(container.style);
            if matches!(ls.glyph, Slot::Set(GlyphRole::PressLeft)) {
                ui.paint_str(text, &label, label_style);
                if area.width >= 2 {
                    paint_pressed_bracket(
                        ui,
                        gutter_cell,
                        cell_at(area, area.right().saturating_sub(1)),
                        label_style,
                    );
                }
            } else {
                ui.paint_str(text, &label, label_style);
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

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::{Color, Modifier};

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
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                let mut button = Button::new(BUTTON, "Go");
                if let Some(status) = status {
                    button = button.status(status);
                }
                button.draw(ui, area);
            })
            .commit_presented();
        buffer
    }

    fn draw_checked(checked: bool) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                Button::new(BUTTON, "Go").checked(checked).draw(ui, area);
            })
            .commit_presented();
        buffer
    }

    fn draw_toggle(checked: bool, status: Status, policy: BusyMarkerPolicy) -> (Buffer, u16) {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let width = Cell::new(0);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                let button = Button::new(BUTTON, "Go")
                    .checked(checked)
                    .status(status)
                    .busy_marker_policy(policy);
                width.set(
                    button
                        .measure(ui, Constraints::loose(AREA.width, 1))
                        .preferred
                        .0,
                );
                button.draw(ui, area);
            })
            .commit_presented();
        (buffer, width.get())
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
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                ui.reference(
                    Some(crate::ReferenceTarget::new(
                        BUTTON,
                        crate::ReferenceState::PRESSED | crate::ReferenceState::FOCUSED,
                    )),
                    |ui| Button::new(BUTTON, LABEL).draw(ui, area),
                );
            })
            .commit_presented();

        assert_eq!(row_text(&buffer, AREA.width), "[Full width]");
    }

    #[test]
    fn button_focus_gutter_uses_focus_visible_state_and_container_surface() {
        let theme = Theme::junie();
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                ui.reference(
                    Some(crate::ReferenceTarget::new(
                        BUTTON,
                        crate::ReferenceState::FOCUSED | crate::ReferenceState::FOCUS_VISIBLE,
                    )),
                    |ui| Button::new(BUTTON, "Go").draw(ui, area),
                );
            })
            .commit_presented();

        let gutter = buffer.cell(Position::new(0, 0)).unwrap();
        let label = buffer.cell(Position::new(1, 0)).unwrap();
        assert_eq!(
            gutter.symbol(),
            Theme::junie().design.glyphs.get(GlyphRole::FocusBar)
        );
        assert_eq!(gutter.bg, label.bg);
    }

    #[test]
    fn pressed_checked_marker_uses_its_variant_on_fill_foreground() {
        for variant in [Variant::PRIMARY, Variant::DANGER] {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(AREA);
            runtime
                .draw_scene(AREA, &mut buffer, |ui, area| {
                    ui.reference(
                        Some(crate::ReferenceTarget::new(
                            BUTTON,
                            crate::ReferenceState::PRESSED,
                        )),
                        |ui| {
                            Button::new(BUTTON, "Go")
                                .variant(variant)
                                .checked(true)
                                .draw(ui, area);
                        },
                    );
                })
                .commit_presented();

            let marker = buffer.cell(Position::new(1, 0)).unwrap();
            let label = buffer.cell(Position::new(3, 0)).unwrap();
            assert_eq!(marker.symbol(), "●");
            assert_eq!(
                marker.fg,
                label.fg,
                "pressed marker mismatch for {variant:?}: symbols {} / {}, row {}",
                marker.symbol(),
                label.symbol(),
                row_text(&buffer, AREA.width),
            );
        }
    }

    #[test]
    fn pressed_checked_marker_keeps_explicit_style_overrides() {
        let patch = [(
            Part::MARKER,
            StylePatch::new()
                .set_fg(crate::theme::Role::Custom(Color::Red))
                .add(Modifier::UNDERLINED),
        )];
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                ui.reference(
                    Some(crate::ReferenceTarget::new(
                        BUTTON,
                        crate::ReferenceState::PRESSED,
                    )),
                    |ui| {
                        Button::new(BUTTON, "Go")
                            .checked(true)
                            .patch_part(&patch)
                            .draw(ui, area);
                    },
                );
            })
            .commit_presented();

        let marker = buffer.cell(Position::new(1, 0)).unwrap();
        assert_eq!(marker.symbol(), "●");
        assert_eq!(marker.fg, Color::Red);
        assert!(marker.modifier.contains(Modifier::UNDERLINED));
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
            runtime
                .draw_scene(screen, &mut buffer, |ui, _| {
                    ui.reference(
                        Some(crate::ReferenceTarget::new(
                            BUTTON,
                            crate::ReferenceState::PRESSED | crate::ReferenceState::FOCUSED,
                        )),
                        |ui| Button::new(BUTTON, LABEL).draw(ui, area),
                    );
                })
                .commit_presented();

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
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                used.set(
                    Button::new(BUTTON, "Go")
                        .status(Status::Error)
                        .checked(true)
                        .patch_part(&patch)
                        .slot(Part::ICON, &replacement)
                        .draw(ui, area),
                );
            })
            .commit_presented();

        assert_eq!(seen.get(), Some(Rect::new(1, 0, 1, 1)));
        assert_eq!(used.get().width, 8);
        assert!(row_text(&buffer, AREA.width).contains("Go"));

        let mut patched = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut patched, |ui, area| {
                Button::new(BUTTON, "Go")
                    .status(Status::Busy)
                    .patch_part(&patch)
                    .draw(ui, area);
            })
            .commit_presented();
        assert!(
            patched
                .cell(Position::new(1, 0))
                .is_some_and(|cell| cell.modifier.contains(Modifier::UNDERLINED))
        );
    }

    #[test]
    fn busy_marker_policy_preserves_extended_default_and_supports_baseline_replacement() {
        let (extended, extended_width) = draw_toggle(true, Status::Busy, BusyMarkerPolicy::Keep);
        let (baseline, baseline_width) = draw_toggle(true, Status::Busy, BusyMarkerPolicy::Replace);
        let (ready, _) = draw_toggle(true, Status::Ready, BusyMarkerPolicy::Replace);
        let (off, _) = draw_toggle(false, Status::Ready, BusyMarkerPolicy::Replace);

        assert_eq!(extended.cell(Position::new(3, 0)).unwrap().symbol(), "●");
        assert_eq!(baseline.cell(Position::new(1, 0)).unwrap().symbol(), "⠋");
        assert_eq!(baseline.cell(Position::new(3, 0)).unwrap().symbol(), "G");
        assert_eq!(ready.cell(Position::new(1, 0)).unwrap().symbol(), "●");
        assert_eq!(off.cell(Position::new(1, 0)).unwrap().symbol(), "○");
        assert_eq!(
            ready.cell(Position::new(1, 0)).unwrap().fg,
            Theme::junie().color.accent
        );
        assert_eq!(
            off.cell(Position::new(1, 0)).unwrap().fg,
            Theme::junie().color.fg[2]
        );
        assert_eq!(
            ready.cell(Position::new(0, 0)).unwrap().bg,
            ready.cell(Position::new(1, 0)).unwrap().bg,
            "the gutter lane keeps the button surface"
        );
        assert_eq!(extended_width, 8);
        assert_eq!(baseline_width, 6);
    }

    struct BusySpinnerApp;

    impl App for BusySpinnerApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            Button::new(BUTTON, "Go")
                .status(Status::Busy)
                .update(cx)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            Button::new(BUTTON, "Go")
                .status(Status::Busy)
                .draw(ui, AREA);
        }
    }

    #[test]
    fn busy_spinner_advances_on_shared_controlled_tick_deadlines() {
        let theme = Theme::junie();
        let expected = theme.design.motion.spinner_frames;
        assert!(expected.len() > 1);
        let mut runtime = Runtime::new(BusySpinnerApp, theme);
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        assert_eq!(
            buffer.cell(Position::new(1, 0)).unwrap().symbol(),
            expected[0]
        );
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(80))
        );

        runtime
            .advance_to(crate::runtime::Moment::from_millis(80))
            .unwrap();
        while runtime.needs_settle() {
            let _ = runtime.settle();
        }
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        assert_eq!(
            buffer.cell(Position::new(1, 0)).unwrap().symbol(),
            expected[1]
        );
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(160))
        );
    }

    const OTHER: Id = Id::root("button.tests.other");
    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 20,
        height: 4,
    };

    #[derive(Default)]
    struct AutofocusApp {
        disabled: bool,
    }

    impl AutofocusApp {
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

    #[test]
    fn autofocus_fires_once_and_never_takes_focus_back() {
        let mut runtime = Runtime::new(AutofocusApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.focus(), Some(OTHER));

        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.focus(), Some(BUTTON), "autofocus must fire once");

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Tab));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.focus(), Some(OTHER));

        for _ in 0..3 {
            let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        }
        assert_eq!(
            runtime.focus(),
            Some(OTHER),
            "a spent autofocus must never request focus again"
        );
    }

    #[test]
    fn a_disabled_autofocus_button_never_takes_focus() {
        let mut runtime = Runtime::new(AutofocusApp { disabled: true }, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.focus(), Some(OTHER));
        assert!(runtime.ring().is_registered(BUTTON));
        assert!(!runtime.ring().contains(BUTTON));

        for _ in 0..3 {
            let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
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

    #[test]
    fn a_pointer_move_without_a_hover_transition_does_not_repaint() {
        let mut runtime = Runtime::new(HoverApp, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let entered = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Move, 1, 0));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let stayed = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Move, 2, 0));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.hover(), Some(BUTTON));
        let pressed = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 2, 0));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let left = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 2, 0));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

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
