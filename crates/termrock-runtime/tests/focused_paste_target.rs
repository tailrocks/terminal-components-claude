#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "focused-paste routing fixture assertions"
)]

use std::process::Command;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock_core::{Id, Input, Intent, KeyCode};
use termrock_runtime::{
    App, Cx, Focusability, LayerSpec, Response, Runtime, ScopeMode, StateFlags, TEST_CLOCK_ENV,
    TEST_CLOCK_PASTE_PREFIX, Theme, Ui, UpdateCause,
};

const AREA: Rect = Rect::new(0, 0, 40, 8);
const TARGET: Id = Id::root("focused-paste.target");
const PEER: Id = Id::root("focused-paste.peer");
const FALLBACK: Id = Id::root("focused-paste.fallback");
const TRAP: Id = Id::root("focused-paste.trap");
const TRAPPED_PEER: Id = Id::root("focused-paste.trapped-peer");
const POPOVER: Id = Id::root("focused-paste.popover");
const POPOVER_PEER: Id = Id::root("focused-paste.popover-peer");
const PAUSED_CLOCK_CHILD_ENV: &str = "TERMROCK_FOCUSED_PASTE_CLOCK_CHILD";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scenario {
    FocusedOffer,
    NoOffer,
    UnfocusedOffer,
    NonEditorOffer,
    DisabledOffer,
    DisabledFlagOffer,
    ReadOnlyOffer,
    ClippedOffer,
    ReferenceOffer,
    InactiveLayerOffer,
    TrappedOutOffer,
    ActiveTypingOwner,
    FallbackTypingOwner,
    PausedClockOffer,
}

impl Scenario {
    const fn starts_focused(self) -> Id {
        match self {
            Self::UnfocusedOffer
            | Self::DisabledOffer
            | Self::ClippedOffer
            | Self::FallbackTypingOwner => PEER,
            _ => TARGET,
        }
    }

    const fn opens_popover(self) -> bool {
        matches!(self, Self::InactiveLayerOffer)
    }

    const fn has_fallback_typing_owner(self) -> bool {
        matches!(self, Self::FallbackTypingOwner)
    }

    const fn target_is_editor(self) -> bool {
        !matches!(self, Self::NonEditorOffer)
    }

    const fn target_is_disabled(self) -> bool {
        matches!(self, Self::DisabledOffer)
    }

    const fn target_declares_disabled(self) -> bool {
        matches!(self, Self::DisabledFlagOffer)
    }

    const fn target_is_read_only(self) -> bool {
        matches!(self, Self::ReadOnlyOffer)
    }

    const fn target_is_clipped(self) -> bool {
        matches!(self, Self::ClippedOffer)
    }

    const fn offers_paste(self) -> bool {
        !matches!(self, Self::NoOffer | Self::ActiveTypingOwner)
    }
}

struct Probe {
    scenario: Scenario,
    pastes: Vec<(Id, String)>,
    clock_steps: Vec<u64>,
    paused: bool,
}

impl Probe {
    fn new(scenario: Scenario) -> Self {
        Self {
            scenario,
            pastes: Vec::new(),
            clock_steps: Vec::new(),
            paused: scenario == Scenario::PausedClockOffer,
        }
    }
}

impl App for Probe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            if self.scenario.opens_popover() {
                cx.open_layer(
                    POPOVER,
                    LayerSpec::popover(
                        POPOVER,
                        termrock_runtime::Anchor::Screen(termrock_runtime::ScreenAlign::Center),
                    ),
                );
            }
            cx.focus(self.scenario.starts_focused());
        }

        for owner in [TARGET, PEER, FALLBACK, TRAPPED_PEER, POPOVER_PEER] {
            for intent in cx.intents(owner) {
                if let Intent::Paste(body) = intent {
                    self.pastes.push((owner, body.to_owned()));
                }
            }
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let scenario = self.scenario;
        let target_area = if scenario.target_is_clipped() {
            Rect::new(50, 0, 10, 1)
        } else {
            Rect::new(0, 0, 20, 1)
        };
        let focusability = if scenario.target_is_disabled() {
            Focusability::Disabled
        } else if scenario.target_is_read_only() {
            Focusability::FocusableReadOnly
        } else {
            Focusability::Focusable
        };
        let flags = if scenario.target_is_read_only() {
            StateFlags::READ_ONLY
        } else if scenario.target_declares_disabled() {
            StateFlags::DISABLED
        } else if scenario == Scenario::ActiveTypingOwner {
            StateFlags::EDITING
        } else {
            StateFlags::empty()
        };

        if scenario.target_is_editor() {
            ui.register_editor(TARGET, target_area, focusability, flags);
        } else {
            ui.register_control(TARGET, target_area, focusability);
        }

        if scenario == Scenario::ReferenceOffer {
            ui.reference(None, |ui| ui.publish_focused_paste_target(TARGET));
        } else if scenario.offers_paste() {
            ui.publish_focused_paste_target(TARGET);
        }

        ui.register_control(PEER, Rect::new(0, 2, 20, 1), Focusability::Focusable);

        if scenario.has_fallback_typing_owner() {
            ui.register_editor(
                FALLBACK,
                Rect::new(0, 4, 20, 1),
                Focusability::Focusable,
                StateFlags::EDITING,
            );
            ui.publish_typing_target::<KeyCode>(FALLBACK, &[], |_| false, false);
        }

        if scenario == Scenario::TrappedOutOffer {
            ui.focus_scope(TRAP, ScopeMode::Trap, |ui| {
                ui.register_control(
                    TRAPPED_PEER,
                    Rect::new(0, 6, 20, 1),
                    Focusability::Focusable,
                );
            });
        }

        if scenario.opens_popover() {
            ui.layer(POPOVER, |ui, _| {
                ui.register_control(
                    POPOVER_PEER,
                    Rect::new(20, 2, 10, 1),
                    Focusability::Focusable,
                );
            });
        }
    }

    fn accept_paused_clock_step(&mut self, step_ms: u64) -> bool {
        if self.paused {
            self.clock_steps.push(step_ms);
            true
        } else {
            false
        }
    }
}

fn boot(scenario: Scenario) -> Runtime<Probe> {
    let mut runtime = Runtime::new(Probe::new(scenario), Theme::junie());
    let mut buffer = Buffer::empty(AREA);
    let _ = runtime.initialize();
    for _ in 0..8 {
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        if !runtime.needs_settle() && !runtime.needs_present() {
            return runtime;
        }
        let _ = runtime.settle();
    }
    assert!(
        !runtime.needs_settle(),
        "the focused-paste fixture did not settle"
    );
    runtime
}

fn paste(runtime: &mut Runtime<Probe>, body: &str) {
    assert!(
        runtime.handle(Input::Paste(body.to_owned())).is_ok(),
        "the presented runtime rejected paste input"
    );
}

#[test]
fn focused_opt_in_editor_receives_one_whole_paste_without_typing_or_cursor_ownership() {
    let mut runtime = boot(Scenario::FocusedOffer);
    assert_eq!(runtime.focus(), Some(TARGET));
    assert_eq!(runtime.typing_owner(), None);
    assert_eq!(runtime.cursor_position(), None);

    let body = "é雪\nline two";
    paste(&mut runtime, body);

    assert_eq!(runtime.app().pastes, [(TARGET, body.to_owned())]);
    assert_eq!(runtime.typing_owner(), None);
}

#[test]
fn no_offer_or_unfocused_offer_does_not_route_to_a_fallback_recipient() {
    for scenario in [Scenario::NoOffer, Scenario::UnfocusedOffer] {
        let mut runtime = boot(scenario);
        assert_eq!(runtime.typing_owner(), None, "{scenario:?}");
        paste(&mut runtime, "unclaimed");
        assert!(runtime.app().pastes.is_empty(), "{scenario:?}");
    }
}

#[test]
fn a_focused_control_cannot_accept_a_paste_offer() {
    let mut runtime = boot(Scenario::NonEditorOffer);
    assert_eq!(runtime.focus(), Some(TARGET));
    assert_eq!(runtime.typing_owner(), None);
    paste(&mut runtime, "not an editor");
    assert!(runtime.app().pastes.is_empty());
}

#[test]
fn disabled_read_only_and_clipped_offers_are_rejected() {
    for scenario in [
        Scenario::DisabledOffer,
        Scenario::DisabledFlagOffer,
        Scenario::ReadOnlyOffer,
        Scenario::ClippedOffer,
    ] {
        let mut runtime = boot(scenario);
        assert_eq!(runtime.typing_owner(), None, "{scenario:?}");
        paste(&mut runtime, "rejected");
        assert!(runtime.app().pastes.is_empty(), "{scenario:?}");
    }
}

#[test]
fn reference_suppression_does_not_publish_a_focused_paste_offer() {
    let mut runtime = boot(Scenario::ReferenceOffer);
    assert_eq!(runtime.focus(), Some(TARGET));
    assert_eq!(runtime.typing_owner(), None);
    paste(&mut runtime, "reference only");
    assert!(runtime.app().pastes.is_empty());
}

#[test]
fn an_offer_from_a_page_below_the_active_layer_is_not_admissible() {
    let mut runtime = boot(Scenario::InactiveLayerOffer);
    assert_eq!(runtime.typing_owner(), None);
    paste(&mut runtime, "underlay");
    assert!(runtime.app().pastes.is_empty());
}

#[test]
fn an_offer_outside_the_active_focus_trap_is_not_admissible() {
    let mut runtime = boot(Scenario::TrappedOutOffer);
    assert_eq!(runtime.typing_owner(), None);
    paste(&mut runtime, "outside trap");
    assert!(runtime.app().pastes.is_empty());
}

#[test]
fn an_active_typing_owner_keeps_paste_precedence_and_receives_once() {
    let mut runtime = boot(Scenario::ActiveTypingOwner);
    assert_eq!(runtime.typing_owner(), Some(TARGET));
    paste(&mut runtime, "active edit");
    assert_eq!(
        runtime.app().pastes,
        [(TARGET, String::from("active edit"))]
    );
}

#[test]
fn a_fallback_typing_owner_keeps_precedence_over_an_unfocused_offer() {
    let mut runtime = boot(Scenario::FallbackTypingOwner);
    assert_eq!(runtime.focus(), Some(PEER));
    assert_eq!(runtime.typing_owner(), Some(FALLBACK));
    paste(&mut runtime, "fallback first");
    assert_eq!(
        runtime.app().pastes,
        [(FALLBACK, String::from("fallback first"))]
    );
}

#[test]
fn paused_clock_paste_is_consumed_before_the_focused_offer_route() {
    if std::env::var_os(PAUSED_CLOCK_CHILD_ENV).is_none() {
        let executable = std::env::current_exe().expect("test executable path");
        let output = Command::new(executable)
            .arg("--exact")
            .arg("paused_clock_paste_is_consumed_before_the_focused_offer_route")
            .arg("--nocapture")
            .env(TEST_CLOCK_ENV, "1")
            .env(PAUSED_CLOCK_CHILD_ENV, "1")
            .output()
            .expect("start isolated paused-clock test child");
        assert!(
            output.status.success(),
            "paused-clock child failed; stdout: {}; stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    assert_eq!(std::env::var(TEST_CLOCK_ENV).as_deref(), Ok("1"));
    let mut runtime = boot(Scenario::PausedClockOffer);
    let mut body = String::from(TEST_CLOCK_PASTE_PREFIX);
    body.push_str("1100");

    paste(&mut runtime, &body);

    assert_eq!(runtime.app().clock_steps, [1100]);
    assert!(runtime.app().pastes.is_empty());
}
