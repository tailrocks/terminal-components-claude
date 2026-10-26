#![allow(
    clippy::unwrap_used,
    reason = "focused public wheel-route preservation assertions"
)]

use ratatui_core::layout::{Position, Rect};
use termrock::runtime::{WheelRouteAttempt, WheelRouteResult};
use termrock::{
    App, Axes, Axis, Cx, Id, Input, Intent, KeyModifiers, Mouse, MouseKind, Part, PartRef,
    Response, Theme, Ui,
};
use termrock_test_support::Harness;

const SCREEN: Rect = Rect::new(0, 0, 12, 5);
const UNDER: Id = Id::root("wheel-unscaled.under");
const TOP: Id = Id::root("wheel-unscaled.top");
const BODY: PartRef = PartRef::of(Part::CONTAINER);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Record {
    event: u64,
    index: u32,
    owner: Id,
    scaled: i16,
    unscaled: i16,
    native_intent_delta: Option<i16>,
}

impl From<WheelRouteAttempt> for Record {
    fn from(attempt: WheelRouteAttempt) -> Self {
        Self {
            event: attempt.event_id(),
            index: attempt.attempt_index(),
            owner: attempt.owner(),
            scaled: attempt.delta(),
            unscaled: attempt.unscaled_delta(),
            native_intent_delta: None,
        }
    }
}

struct Probe {
    initial: Vec<Record>,
    fallback: Vec<Record>,
}

impl App for Probe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for intent in cx.intents(TOP) {
            if let Intent::Wheel {
                axis, delta, part, ..
            } = intent
                && let Some(attempt) = cx.wheel_route_attempt(TOP, part, axis)
            {
                self.initial.push(attempt.into());
                if let Some(record) = self.initial.last_mut() {
                    record.native_intent_delta = Some(delta);
                }
                assert!(cx.acknowledge_wheel_route(&attempt, WheelRouteResult::Ignored));
            }
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_hit_stack_wheel(UNDER, BODY, SCREEN, Axes::H, 1);
        ui.register_hit_stack_wheel(TOP, BODY, SCREEN, Axes::H, 2);
    }

    fn route_wheel_attempt(&mut self, cx: &mut Cx<'_>, attempt: WheelRouteAttempt) -> Response<()> {
        self.fallback.push(attempt.into());
        assert_eq!(cx.update_cause(), termrock::UpdateCause::ScrollRoute);
        assert!(cx.acknowledge_wheel_route(&attempt, WheelRouteResult::Consumed));
        Response::ignored()
    }
}

#[test]
fn decoded_delta_survives_theme_scaling_and_hit_stack_fallback() {
    let mut h = Harness::new(
        Probe {
            initial: Vec::new(),
            fallback: Vec::new(),
        },
        Theme::junie(),
        SCREEN.width,
        SCREEN.height,
    );

    let inputs = [
        (1, 3),
        (2, 6),
        (-1, -3),
        (20_000, i16::MAX),
        (-20_000, i16::MIN),
        (i16::MIN, i16::MIN),
        (0, 0),
    ];
    for (delta, scaled) in inputs {
        let _ = h.handle(Input::Mouse(Mouse {
            kind: MouseKind::Wheel(Axis::H, delta),
            pos: Position::new(5, 2),
            mods: KeyModifiers::NONE,
        }));
        let top = *h.app().initial.last().unwrap();
        let parent = *h.app().fallback.last().unwrap();
        assert_eq!(top.owner, TOP);
        assert_eq!(parent.owner, UNDER);
        assert_eq!(top.scaled, scaled);
        assert_eq!(top.native_intent_delta, Some(scaled));
        assert_eq!(parent.scaled, scaled);
        assert_eq!(parent.native_intent_delta, None);
        assert_eq!(top.unscaled, delta);
        assert_eq!(parent.unscaled, delta);
        assert_eq!(top.index, 0);
        assert_eq!(parent.index, 1);
        assert_eq!(top.event, parent.event);
    }

    assert_eq!(h.app().initial.len(), inputs.len());
    assert_eq!(h.app().fallback.len(), inputs.len());
    assert!(
        h.app()
            .initial
            .windows(2)
            .all(|pair| pair[0].event != pair[1].event)
    );
}
