//! The conformance matrix (`COMPONENT_ARCHITECTURE.md` §16.2). Slice 3 has no
//! components yet; `ProbeCase` is a minimal button-like control written on
//! the `author` surface so the driver itself is exercised end to end. Slice 4
//! packages append their `Case`s to the `conformance_suite!` list below.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]

use tui_next::author::{
    Activated, Binding, BindingState, Chord, Cx, Family, Focusability, FrameRead, GlyphRole, Id,
    Intent, KeyCode, Part, PartRef, Phase, Position, Rect, Response, StateFlags, Ui, Variant,
};
use tui_next_testing::conformance::{Caps, Conformance, Fixture};
use tui_next_testing::conformance_suite;

const PROBE: Id = Id::root("conformance.probe");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeCmd {
    Activate,
}

const BINDINGS: &[Binding<ProbeCmd>] = &[
    Binding {
        chord: Chord::key(KeyCode::Enter),
        cmd: ProbeCmd::Activate,
        label: "Activate",
        priority: 80,
        visible: true,
    },
    Binding {
        chord: Chord::key(KeyCode::Char(' ')),
        cmd: ProbeCmd::Activate,
        label: "Activate",
        priority: 80,
        visible: false,
    },
];

#[derive(Clone, Debug, Default, PartialEq)]
struct ProbeState {
    fired: u32,
}

/// A button-like control: Enter / Space / click activate; writes the cursor
/// while focused; honours `disabled`, `state_override` and `patch`.
struct ProbeCase;

impl Conformance for ProbeCase {
    const NAME: &'static str = "probe";
    const FAMILY: Family = Family::BUTTON;
    const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::LABEL];
    type State = ProbeState;
    type Action = Activated;
    type Cmd = ProbeCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::DISABLEABLE | Caps::FOCUSABLE | Caps::CURSOR
    }

    fn id() -> Id {
        PROBE
    }

    fn update(cx: &mut Cx<'_>, st: &mut ProbeState, f: &Fixture) -> Response<Activated> {
        let mut r = Response::ignored();
        for it in cx.intents(PROBE) {
            match it {
                Intent::Key(k) if !f.disabled => {
                    if Binding::lookup(BINDINGS, &k).is_some() {
                        st.fired += 1;
                        r = Response::action(Activated);
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click,
                    ..
                } if !f.disabled => {
                    st.fired += 1;
                    r = Response::action(Activated);
                }
                Intent::Pointer { .. } if !f.disabled => r = Response::changed(),
                _ => {}
            }
        }
        r.for_id(PROBE)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &ProbeState, f: &Fixture) {
        if area.is_empty() {
            return;
        }
        let focus = if f.disabled {
            Focusability::Disabled
        } else {
            Focusability::Focusable
        };
        ui.register_control(PROBE, area, focus);
        // `state_override` replaces the runtime state (A11: render a forced state)
        let mut live = if f.state_override.is_empty() {
            ui.state(PROBE)
        } else {
            f.state_override
        };
        if f.disabled {
            live |= StateFlags::DISABLED;
        }
        let style_for = |ui: &mut Ui<'_>, part: Part| match f.patch {
            Some((p, patch)) if p == part => {
                ui.style_patched(Family::BUTTON, Variant::DEFAULT, part, live, &patch)
            }
            _ => ui.style(Family::BUTTON, Variant::DEFAULT, part, live),
        };
        let container = style_for(ui, Part::CONTAINER);
        ui.fill(area, container.style);
        let gutter = style_for(ui, Part::GUTTER);
        let gutter_cell = Rect {
            width: 1.min(area.width),
            ..area
        };
        if let Some(g) = gutter.glyph {
            ui.glyph(gutter_cell, g, gutter.style);
        }
        let label = style_for(ui, Part::LABEL);
        let mut text = Rect {
            x: area.x.saturating_add(1),
            width: area.width.saturating_sub(1),
            ..area
        };
        text.height = 1.min(area.height);
        if label.glyph == Some(GlyphRole::PressLeft) {
            let used = ui.glyph(text, GlyphRole::PressLeft, label.style);
            text.x = text.x.saturating_add(used);
            text.width = text.width.saturating_sub(used);
            let used = ui.paint_str(text, "Probe", label.style);
            text.x = text.x.saturating_add(used);
            text.width = text.width.saturating_sub(used);
            ui.glyph(text, GlyphRole::PressRight, label.style);
        } else {
            ui.paint_str(text, "Probe", label.style);
        }
        if live.contains(StateFlags::FOCUSED) {
            ui.set_cursor(PROBE, Position::new(area.x.saturating_add(1), area.y));
        }
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::of(Part::CONTAINER)
    }

    fn bindings(_s: BindingState) -> &'static [Binding<ProbeCmd>] {
        BINDINGS
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 4] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
        ];
        &STATES
    }
}

conformance_suite!(probe => ProbeCase);

mod registry {
    #[test]
    fn every_public_component_is_registered() {
        // Slice 3 ships no components; the registered list is the probe alone
        assert_eq!(super::registered_cases(), vec!["probe"]);
    }
}
