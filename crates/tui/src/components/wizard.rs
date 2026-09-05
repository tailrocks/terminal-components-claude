//! Stateful multi-step flow controller (`COMPONENT_ARCHITECTURE.md` §14.2 J7).

use ratatui_core::layout::Rect;
use std::collections::BTreeMap;

use super::{PartStyle, SlotFn};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::theme::{Family, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// One declared wizard step.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WizardStep<'a> {
    /// Stable step identity.
    pub key: ItemKey,
    /// Display label.
    pub label: &'a str,
    /// Whether forward navigation may enter this step.
    pub enabled: bool,
}

impl<'a> WizardStep<'a> {
    /// Declare an enabled step.
    pub const fn new(key: ItemKey, label: &'a str) -> Self {
        Self {
            key,
            label,
            enabled: true,
        }
    }

    /// Enable or disable the step.
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Semantic wizard output.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WizardAction {
    /// The active step changed.
    Moved(ItemKey),
    /// The current step requested completion.
    Finish(ItemKey),
}

/// Wizard key commands.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WizardCmd {
    /// Rewind one visited step.
    Back,
    /// Advance one enabled step.
    Next,
    /// Finish on the last step.
    Finish,
}

const BINDINGS: &[Binding<WizardCmd>] = &[
    Binding {
        action: crate::ActionKey::custom("wizard.back"),
        chord: Some(Chord::key(KeyCode::Left)),
        cmd: WizardCmd::Back,
        label: "Back",
        priority: 60,
        visible: true,
    },
    Binding {
        action: crate::ActionKey::custom("wizard.next"),
        chord: Some(Chord::key(KeyCode::Right)),
        cmd: WizardCmd::Next,
        label: "Next",
        priority: 60,
        visible: true,
    },
    Binding {
        action: crate::ActionKey::custom("wizard.finish"),
        chord: Some(Chord::key(KeyCode::Enter)),
        cmd: WizardCmd::Finish,
        label: "Continue",
        priority: 70,
        visible: true,
    },
];

/// Durable navigation plus caller-selected per-step state.
///
/// State slots are keyed, so rewinding and later returning never rebuilds a
/// step's state. The payload type defaults to `()` for stateless flows.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WizardState<S = ()> {
    current: Option<ItemKey>,
    history: Vec<ItemKey>,
    slots: BTreeMap<ItemKey, S>,
}

impl<S> Default for WizardState<S> {
    fn default() -> Self {
        Self {
            current: None,
            history: Vec::new(),
            slots: BTreeMap::new(),
        }
    }
}

impl<S> WizardState<S> {
    /// Active step key.
    pub const fn current(&self) -> Option<ItemKey> {
        self.current
    }

    /// The retained state for `key`, inserting it exactly once.
    pub fn state_mut(&mut self, key: ItemKey, init: impl FnOnce() -> S) -> &mut S {
        self.slots.entry(key).or_insert_with(init)
    }

    fn enter(&mut self, key: ItemKey) {
        if self.current != Some(key) {
            if let Some(old) = self.current {
                self.history.push(old);
            }
            self.current = Some(key);
        }
    }

    fn rewind(&mut self) -> Option<ItemKey> {
        let key = self.history.pop()?;
        self.current = Some(key);
        Some(key)
    }
}

/// A keyed wizard controller with an inline stepper rail.
///
/// ## Construction
/// `Wizard::new(id, steps)`; step payload remains in caller-owned
/// [`WizardState`].
///
/// ## Ownership
/// The caller owns state and step payloads. The component owns navigation.
///
/// ## Configuration
/// `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::WIZARD`, `Variant::DEFAULT`.
///
/// ## States
/// `FOCUSED`; active step uses `ACTIVE`, disabled steps use `DISABLED`.
///
/// ## Actions
/// [`WizardAction::Moved`] and [`WizardAction::Finish`], both keyed.
///
/// ## Focus
/// One focus stop for the complete flow.
///
/// ## Keyboard
/// Left rewinds; Right advances; Enter advances or finishes.
///
/// ## Mouse
/// Clicking an enabled step enters it.
///
/// ## Layout
/// One horizontal stepper row; labels share the available width.
///
/// ## Parts
/// `CONTAINER`, `MARKER`, `LABEL`.
///
/// ## Overrides
/// Patch all parts; slots replace `MARKER` or `LABEL`.
///
/// ## Identity
/// One component id; enabled steps publish root-owned keyed label parts.
///
/// ## Testing
/// Conformance plus retained-state rewind coverage.
#[derive(Debug)]
pub struct Wizard<'a> {
    id: Id,
    steps: &'a [WizardStep<'a>],
    ov: PartStyle<'a>,
}

impl<'a> Wizard<'a> {
    /// Styled parts.
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::MARKER, Part::LABEL];

    /// Build a wizard over borrowed declarations.
    pub const fn new(id: Id, steps: &'a [WizardStep<'a>]) -> Self {
        Self {
            id,
            steps,
            ov: PartStyle::new(),
        }
    }

    /// Patch every part.
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.ov = self.ov.global(patch);
        self
    }
    /// Patch selected parts.
    #[must_use]
    pub const fn patch_part(mut self, parts: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(parts);
        self
    }
    /// Replace a supported part painter.
    #[must_use]
    pub const fn slot(mut self, part: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, f);
        self
    }

    /// Update navigation. Declarations are borrowed in both phases.
    pub fn update<S>(&self, cx: &mut Cx<'_>, st: &mut WizardState<S>) -> Response<WizardAction> {
        if st.current.is_none() {
            st.current = self.steps.iter().find(|s| s.enabled).map(|s| s.key);
        }
        let mut action = None;
        let mut consumed = false;
        for intent in cx.intents(self.id) {
            match intent {
                Intent::Binding(binding_key) => {
                    let Some(cmd) = Binding::command(BINDINGS, binding_key) else {
                        continue;
                    };
                    consumed = true;
                    match cmd {
                        WizardCmd::Back => action = st.rewind().map(WizardAction::Moved),
                        WizardCmd::Next | WizardCmd::Finish => {
                            let current = st.current;
                            let pos = self
                                .steps
                                .iter()
                                .position(|s| Some(s.key) == current)
                                .unwrap_or(0);
                            if let Some(next) = self
                                .steps
                                .iter()
                                .skip(pos.saturating_add(1))
                                .find(|s| s.enabled)
                            {
                                st.enter(next.key);
                                action = Some(WizardAction::Moved(next.key));
                            } else if let Some(key) = current {
                                action = Some(WizardAction::Finish(key));
                            }
                        }
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    part:
                        PartRef {
                            part: Part::LABEL,
                            item: Some(key),
                        },
                    ..
                } if self
                    .steps
                    .iter()
                    .any(|step| step.key == key && step.enabled) =>
                {
                    st.enter(key);
                    action.get_or_insert(WizardAction::Moved(key));
                    consumed = true;
                }
                Intent::Pointer {
                    part:
                        PartRef {
                            part: Part::LABEL, ..
                        },
                    ..
                } => {
                    consumed = true;
                }
                _ => {}
            }
        }
        match action {
            Some(a) => Response::action(a),
            None if consumed => Response::consumed(),
            None => Response::ignored(),
        }
        .for_id(self.id)
    }

    /// Draw the inline stepper.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass owns keyed step geometry, state and painting"
    )]
    pub fn draw<S>(&self, ui: &mut Ui<'_>, area: Rect, st: &WizardState<S>) -> Rect {
        if area.is_empty() || self.steps.is_empty() {
            return Rect { height: 0, ..area };
        }
        ui.register_control(self.id, area, Focusability::Focusable);
        let live = PartStyle::flags(ui.state(self.id), StateFlags::empty());
        let base_live = live.difference(
            StateFlags::FOCUSED
                | StateFlags::FOCUS_VISIBLE
                | StateFlags::HOVERED
                | StateFlags::PRESSED,
        );
        ui.publish_bindings(self.id, live, BINDINGS);
        let base = self.ov.style(
            ui,
            self.id,
            Family::WIZARD,
            Variant::DEFAULT,
            Part::CONTAINER,
            base_live,
        );
        ui.fill(area, base.style);
        let row = Rect { height: 1, ..area };
        let count = u16::try_from(self.steps.len()).unwrap_or(u16::MAX).max(1);
        let width = row.width.checked_div(count).unwrap_or_default();
        for (i, step) in self.steps.iter().enumerate() {
            let x = row
                .x
                .saturating_add(u16::try_from(i).unwrap_or(u16::MAX).saturating_mul(width));
            let cell = Rect {
                x,
                y: row.y,
                width: if i.saturating_add(1) == self.steps.len() {
                    row.right().saturating_sub(x)
                } else {
                    width
                },
                height: 1,
            };
            let mut flags = base_live;
            if st.current == Some(step.key) {
                flags |=
                    StateFlags::ACTIVE | live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
            }
            if !step.enabled {
                flags |= StateFlags::DISABLED;
            }
            if step.enabled {
                ui.register_part(self.id, PartRef::item(Part::LABEL, step.key), cell);
            }
            let runtime_pressed =
                FrameRead::pressed_part(ui, self.id) == Some(PartRef::item(Part::LABEL, step.key));
            if FrameRead::hovered_part(ui, self.id) == Some(PartRef::item(Part::LABEL, step.key)) {
                flags |= StateFlags::HOVERED;
            }
            if runtime_pressed {
                flags |= StateFlags::PRESSED;
            }
            let marker = Rect {
                width: cell.width.min(2),
                ..cell
            };
            if let Some(slot) = self.ov.slot_for(Part::MARKER) {
                slot(ui, marker);
            } else {
                let s = self.ov.style(
                    ui,
                    self.id,
                    Family::WIZARD,
                    Variant::DEFAULT,
                    Part::MARKER,
                    flags,
                );
                if flags.contains(StateFlags::FOCUSED) {
                    // Focus must remain visible when colour is unavailable.
                    // Keep the active marker in the second cell so focus does
                    // not erase the step's current-state affordance.
                    ui.glyph(
                        Rect {
                            width: marker.width.min(1),
                            ..marker
                        },
                        crate::theme::GlyphRole::FocusBar,
                        s.style,
                    );
                    if st.current == Some(step.key) && marker.width > 1 {
                        ui.paint_str(
                            Rect {
                                x: marker.x.saturating_add(1),
                                width: marker.width.saturating_sub(1),
                                ..marker
                            },
                            "●",
                            s.style,
                        );
                    }
                } else {
                    ui.paint_str(
                        marker,
                        if st.current == Some(step.key) {
                            "●"
                        } else {
                            "○"
                        },
                        s.style,
                    );
                }
            }
            let label = Rect {
                x: marker.right(),
                width: cell.width.saturating_sub(marker.width),
                ..cell
            };
            if let Some(slot) = self.ov.slot_for(Part::LABEL) {
                slot(ui, label);
            } else {
                let s = self.ov.style(
                    ui,
                    self.id,
                    Family::WIZARD,
                    Variant::DEFAULT,
                    Part::LABEL,
                    flags,
                );
                ui.paint_str(label, step.label, s.style);
            }
        }
        row
    }

    /// Natural single-row size.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let w = self.steps.iter().fold(0u16, |n, s| {
            n.saturating_add(crate::text::width(s.label))
                .saturating_add(3)
        });
        Size {
            min: (1, 1),
            preferred: (w, 1),
        }
        .fit(c)
    }
}

impl Bindings for Wizard<'_> {
    type Cmd = WizardCmd;
    fn bindings(&self, _state: BindingState) -> &'static [Binding<Self::Cmd>] {
        BINDINGS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use crate::ui::cx::LastFrame;
    use crate::ui::{FrameState, Ui, UiCore};
    use ratatui_core::buffer::Buffer;

    const AREA: Rect = Rect::new(0, 0, 32, 2);

    fn draw(wizard: &Wizard<'_>, state: &WizardState) -> FrameState {
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, AREA);
        let mut page = Buffer::empty(AREA);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            wizard.draw(&mut ui, AREA, state);
        }
        frame
    }

    #[test]
    fn rewind_retains_per_step_state() {
        let a = ItemKey::text("a");
        let b = ItemKey::text("b");
        let mut state = WizardState::<String>::default();
        state.enter(a);
        state.state_mut(a, String::new).push_str("draft-a");
        state.enter(b);
        state.state_mut(b, String::new).push_str("draft-b");
        assert_eq!(state.rewind(), Some(a));
        assert_eq!(state.state_mut(a, String::new), "draft-a");
        assert_eq!(state.state_mut(b, String::new), "draft-b");
    }

    #[test]
    fn enabled_steps_register_root_owned_keyed_label_parts() {
        let a = WizardStep::new(ItemKey::num(1), "Account");
        let b = WizardStep::new(ItemKey::num(2), "Details");
        let steps = [a, b];
        let id = Id::root("wizard.parts");
        let wizard = Wizard::new(id, &steps);
        let state = WizardState {
            current: Some(a.key),
            ..WizardState::default()
        };
        let frame = draw(&wizard, &state);
        assert!(
            frame
                .registry
                .area_of_part(id, PartRef::item(Part::LABEL, b.key))
                .is_some()
        );
    }

    #[test]
    fn disabled_step_registers_no_activation_part() {
        let step = WizardStep::new(ItemKey::num(2), "Details").enabled(false);
        let steps = [step];
        let id = Id::root("wizard.disabled");
        let wizard = Wizard::new(id, &steps);
        let frame = draw(&wizard, &WizardState::default());
        assert!(
            frame
                .registry
                .area_of_part(id, PartRef::item(Part::LABEL, step.key))
                .is_none()
        );
    }
}
