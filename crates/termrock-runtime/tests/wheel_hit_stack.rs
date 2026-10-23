use ratatui_core::layout::{Position, Rect};
use termrock::runtime::{ScrollRouteKey, WheelRouteAttempt, WheelRouteResult};
use termrock::{
    App, Axis, Cx, Focusability, Headroom, Id, Input, Intent, KeyModifiers, LayerSpec, Mouse,
    MouseKind, Part, PartRef, Phase, Response, Theme, Ui, UpdateCause,
};
use termrock_test_support::Harness;

const SCREEN: Rect = Rect::new(0, 0, 20, 8);
const UNDER: Id = Id::root("wheel-hit-stack.under");
const MIDDLE: Id = Id::root("wheel-hit-stack.middle");
const TOP: Id = Id::root("wheel-hit-stack.top");
const OWNER: Id = Id::root("wheel-hit-stack.owner");
const MODAL: Id = Id::root("wheel-hit-stack.modal");
const FOCUS_TARGET: Id = Id::root("wheel-hit-stack.focus");
const PAGE_SENTINEL: Id = Id::root("wheel-hit-stack.page-sentinel");
const MODAL_UNDER: Id = Id::root("wheel-hit-stack.modal-under");
const CONTAINER: PartRef = PartRef::of(Part::CONTAINER);
const LABEL: PartRef = PartRef::of(Part::LABEL);
const BODY: PartRef = PartRef::of(Part::BODY);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Registration {
    HitStack,
    Linked,
    PlainScroll,
    PassiveReveal,
    Control,
    FocusableControl,
    Part,
    ControlPartHitStack,
}

#[derive(Clone, Copy)]
struct Target {
    owner: Id,
    part: PartRef,
    area: Rect,
    axes: termrock::Axes,
    source_revision: u64,
    result: Option<WheelRouteResult>,
    registration: Registration,
    layer: Option<Id>,
}

impl Target {
    fn new(owner: Id, part: PartRef, area: Rect, registration: Registration) -> Self {
        Self {
            owner,
            part,
            area,
            axes: termrock::Axes::Both,
            source_revision: 1,
            result: Some(WheelRouteResult::Ignored),
            registration,
            layer: None,
        }
    }

    fn on_layer(mut self, layer: Id) -> Self {
        self.layer = Some(layer);
        self
    }

    fn axes(mut self, axes: termrock::Axes) -> Self {
        self.axes = axes;
        self
    }

    fn source(mut self, source_revision: u64) -> Self {
        self.source_revision = source_revision;
        self
    }

    fn result(mut self, result: Option<WheelRouteResult>) -> Self {
        self.result = result;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AttemptRecord {
    event_id: u64,
    attempt_index: u32,
    owner: Id,
    part: PartRef,
    axis: Axis,
    delta: i16,
    position: Position,
    modifiers: KeyModifiers,
    source_revision: u64,
}

impl From<WheelRouteAttempt> for AttemptRecord {
    fn from(attempt: WheelRouteAttempt) -> Self {
        Self {
            event_id: attempt.event_id(),
            attempt_index: attempt.attempt_index(),
            owner: attempt.owner(),
            part: attempt.part(),
            axis: attempt.axis(),
            delta: attempt.delta(),
            position: attempt.position(),
            modifiers: attempt.modifiers(),
            source_revision: attempt.source_revision(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WheelRecord {
    owner: Id,
    part: PartRef,
    axis: Axis,
    delta: i16,
    position: Position,
}

struct HitStackProbe {
    targets: Vec<Target>,
    links: Vec<(usize, usize)>,
    live_source_overrides: Vec<(Id, PartRef, u64)>,
    event_updates: usize,
    wheel_deliveries: Vec<WheelRecord>,
    initial_attempts: Vec<AttemptRecord>,
    service_attempts: Vec<AttemptRecord>,
    service_ack_results: Vec<bool>,
    service_intents_empty: bool,
    pointer_deliveries: Vec<(Id, PartRef, Phase)>,
    capture_on_press: bool,
    release_capture_on_wheel: bool,
    capture_results: Vec<bool>,
    open_modal_on_bootstrap: bool,
    open_modal_after_event: bool,
    close_modal_after_event: bool,
    focus_after_event: Option<Id>,
    relayout_after_event: bool,
}

impl HitStackProbe {
    fn new(targets: impl Into<Vec<Target>>) -> Self {
        Self {
            targets: targets.into(),
            links: Vec::new(),
            live_source_overrides: Vec::new(),
            event_updates: 0,
            wheel_deliveries: Vec::new(),
            initial_attempts: Vec::new(),
            service_attempts: Vec::new(),
            service_ack_results: Vec::new(),
            service_intents_empty: true,
            pointer_deliveries: Vec::new(),
            capture_on_press: false,
            release_capture_on_wheel: false,
            capture_results: Vec::new(),
            open_modal_on_bootstrap: false,
            open_modal_after_event: false,
            close_modal_after_event: false,
            focus_after_event: None,
            relayout_after_event: false,
        }
    }

    fn route_result(&self, owner: Id, part: PartRef) -> Option<WheelRouteResult> {
        self.targets
            .iter()
            .rev()
            .find(|target| {
                target.owner == owner
                    && target.part == part
                    && matches!(
                        target.registration,
                        Registration::HitStack
                            | Registration::Linked
                            | Registration::ControlPartHitStack
                    )
            })
            .and_then(|target| target.result)
    }

    fn live_source_revision(&self, owner: Id, part: PartRef) -> Option<u64> {
        self.live_source_overrides
            .iter()
            .rev()
            .find(|(candidate, candidate_part, _)| *candidate == owner && *candidate_part == part)
            .map(|(_, _, revision)| *revision)
            .or_else(|| {
                self.targets
                    .iter()
                    .rev()
                    .find(|target| {
                        target.owner == owner
                            && target.part == part
                            && matches!(
                                target.registration,
                                Registration::HitStack
                                    | Registration::Linked
                                    | Registration::ControlPartHitStack
                            )
                    })
                    .map(|target| target.source_revision)
            })
    }

    fn draw_target(ui: &mut Ui<'_>, target: Target) -> Option<ScrollRouteKey> {
        match target.registration {
            Registration::HitStack => ui.register_hit_stack_wheel(
                target.owner,
                target.part,
                target.area,
                target.axes,
                target.source_revision,
            ),
            Registration::Linked => ui.register_routed_scroll(
                target.owner,
                target.area,
                target.axes,
                Headroom::default(),
                target.source_revision,
            ),
            Registration::PlainScroll => {
                ui.register_scroll(target.owner, target.area, target.axes, Headroom::default());
                None
            }
            Registration::PassiveReveal => {
                ui.register_passive_reveal_scroll(
                    target.owner,
                    target.area,
                    Headroom {
                        down: 2,
                        ..Headroom::default()
                    },
                );
                None
            }
            Registration::Control => {
                ui.register_control(target.owner, target.area, Focusability::Disabled);
                None
            }
            Registration::FocusableControl => {
                ui.register_control(target.owner, target.area, Focusability::Focusable);
                None
            }
            Registration::Part => {
                ui.register_part(target.owner, target.part, target.area);
                None
            }
            Registration::ControlPartHitStack => {
                ui.register_control(target.owner, target.area, Focusability::Focusable);
                ui.register_part(target.owner, target.part, target.area);
                ui.register_hit_stack_wheel(
                    target.owner,
                    target.part,
                    target.area,
                    target.axes,
                    target.source_revision,
                )
            }
        }
    }

    fn owners(&self) -> impl Iterator<Item = Id> + '_ {
        self.targets
            .iter()
            .enumerate()
            .filter_map(|(index, target)| {
                (!self.targets[..index]
                    .iter()
                    .any(|previous| previous.owner == target.owner))
                .then_some(target.owner)
            })
    }
}

impl App for HitStackProbe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let cause = cx.update_cause();
        if cause == UpdateCause::Event {
            self.event_updates += 1;
        }
        if cause == UpdateCause::Bootstrap && self.open_modal_on_bootstrap {
            cx.open_layer(MODAL, LayerSpec::modal(MODAL));
        }

        let owners = self.owners().collect::<Vec<_>>();
        for owner in owners {
            for intent in cx.intents(owner) {
                match intent {
                    Intent::Wheel {
                        axis,
                        delta,
                        part,
                        pos,
                    } => {
                        self.wheel_deliveries.push(WheelRecord {
                            owner,
                            part,
                            axis,
                            delta,
                            position: pos,
                        });
                        if let Some(attempt) = cx.wheel_route_attempt(owner, part, axis) {
                            self.initial_attempts.push(attempt.into());
                            if self.release_capture_on_wheel {
                                cx.release_capture();
                            }
                            if self.live_source_revision(owner, part)
                                == Some(attempt.source_revision())
                                && let Some(result) = self.route_result(owner, part)
                            {
                                assert!(cx.acknowledge_wheel_route(&attempt, result));
                            }
                        }
                    }
                    Intent::Pointer { phase, part, .. } => {
                        self.pointer_deliveries.push((owner, part, phase));
                        if self.capture_on_press && phase == Phase::Press {
                            self.capture_results.push(cx.capture(owner, part));
                        }
                    }
                    _ => {}
                }
            }
        }

        if cause == UpdateCause::Event && self.open_modal_after_event {
            self.open_modal_after_event = false;
            cx.open_layer(MODAL, LayerSpec::modal(MODAL));
        }
        if cause == UpdateCause::Event && self.close_modal_after_event {
            self.close_modal_after_event = false;
            cx.close_layer(MODAL, None);
        }
        if cause == UpdateCause::Event
            && let Some(focus) = self.focus_after_event.take()
        {
            cx.focus(focus);
        }
        if cause == UpdateCause::Event && self.relayout_after_event {
            self.relayout_after_event = false;
            return Response::ignored().relayout();
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let mut keys = vec![None; self.targets.len()];
        let mut drawn_layers = Vec::new();
        for (index, target) in self.targets.iter().enumerate() {
            let Some(layer) = target.layer else {
                keys[index] = Self::draw_target(ui, *target);
                continue;
            };
            if drawn_layers.contains(&layer) {
                continue;
            }
            drawn_layers.push(layer);
            let _ = ui.layer(layer, |layer_ui, _| {
                for (layer_index, layer_target) in self.targets.iter().enumerate() {
                    if layer_target.layer == Some(layer) {
                        keys[layer_index] = Self::draw_target(layer_ui, *layer_target);
                    }
                }
            });
        }
        for (child, parent) in &self.links {
            if let (Some(Some(child)), Some(Some(parent))) = (keys.get(*child), keys.get(*parent)) {
                ui.link_scroll_parent(*child, *parent);
            }
        }
    }

    fn route_wheel_attempt(&mut self, cx: &mut Cx<'_>, attempt: WheelRouteAttempt) -> Response<()> {
        self.service_attempts.push(attempt.into());
        self.service_intents_empty &= cx.update_cause() == UpdateCause::ScrollRoute
            && self
                .owners()
                .all(|owner| cx.intents(owner).next().is_none());
        if self.live_source_revision(attempt.owner(), attempt.part())
            == Some(attempt.source_revision())
            && let Some(result) = self.route_result(attempt.owner(), attempt.part())
        {
            self.service_ack_results
                .push(cx.acknowledge_wheel_route(&attempt, result));
        }
        Response::ignored()
    }
}

fn target(owner: Id, area: Rect) -> Target {
    Target::new(owner, CONTAINER, area, Registration::HitStack)
}

fn harness(app: HitStackProbe) -> Harness<HitStackProbe> {
    Harness::new(app, Theme::junie(), SCREEN.width, SCREEN.height)
}

fn wheel(harness: &mut Harness<HitStackProbe>, axis: Axis, delta: i16, x: u16, y: u16) {
    wheel_with_modifiers(harness, axis, delta, x, y, KeyModifiers::NONE);
}

fn wheel_with_modifiers(
    harness: &mut Harness<HitStackProbe>,
    axis: Axis,
    delta: i16,
    x: u16,
    y: u16,
    mods: KeyModifiers,
) {
    let _ = harness.handle(Input::Mouse(Mouse {
        kind: MouseKind::Wheel(axis, delta),
        pos: Position::new(x, y),
        mods,
    }));
}

fn attempted_owners(app: &HitStackProbe) -> Vec<Id> {
    app.initial_attempts
        .iter()
        .chain(&app.service_attempts)
        .map(|attempt| attempt.owner)
        .collect()
}

#[test]
fn unlinked_overlapping_opt_in_targets_continue_on_the_same_wheel_event() {
    let mut h = harness(HitStackProbe::new([
        target(UNDER, SCREEN).source(11),
        target(TOP, SCREEN).source(22),
    ]));
    wheel_with_modifiers(
        &mut h,
        Axis::V,
        2,
        5,
        3,
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    );

    assert_eq!(
        h.app().event_updates,
        1,
        "the physical wheel updates App once"
    );
    assert_eq!(attempted_owners(h.app()), [TOP, UNDER]);
    assert_eq!(h.app().initial_attempts[0].attempt_index, 0);
    assert_eq!(h.app().service_attempts[0].attempt_index, 1);
    assert_eq!(
        h.app().initial_attempts[0].event_id,
        h.app().service_attempts[0].event_id
    );
    assert_eq!(h.app().initial_attempts[0].source_revision, 22);
    assert_eq!(h.app().service_attempts[0].source_revision, 11);
    assert_eq!(h.app().initial_attempts[0].axis, Axis::V);
    assert_eq!(h.app().service_attempts[0].axis, Axis::V);
    assert_eq!(h.app().initial_attempts[0].position, Position::new(5, 3));
    assert_eq!(h.app().service_attempts[0].position, Position::new(5, 3));
    assert_eq!(
        h.app().initial_attempts[0].delta,
        2 * Theme::junie().design.motion.wheel_rows as i16
    );
    assert_eq!(
        h.app().service_attempts[0].delta,
        h.app().initial_attempts[0].delta
    );
    assert_eq!(
        h.app().initial_attempts[0].modifiers,
        KeyModifiers::CONTROL | KeyModifiers::ALT
    );
    assert_eq!(
        h.app().service_attempts[0].modifiers,
        h.app().initial_attempts[0].modifiers
    );
    assert_eq!(h.app().service_ack_results, [true]);
    assert!(h.app().service_intents_empty);
}

#[test]
fn partial_overlap_uses_reverse_registration_order_without_containment() {
    let left = Rect::new(0, 0, 12, 8);
    let right = Rect::new(6, 0, 12, 8);
    let mut h = harness(HitStackProbe::new([
        target(UNDER, left),
        target(TOP, right),
    ]));

    wheel(&mut h, Axis::V, 1, 8, 2);

    assert_eq!(attempted_owners(h.app()), [TOP, UNDER]);
    assert_eq!(h.app().service_attempts[0].position, Position::new(8, 2));
}

#[test]
fn a_wide_target_uses_the_point_dependent_underlying_target() {
    let left = Rect::new(0, 0, 8, 8);
    let right = Rect::new(12, 0, 8, 8);
    let wide = SCREEN;
    for (x, expected) in [(2, UNDER), (16, MIDDLE)] {
        let mut h = harness(HitStackProbe::new([
            target(UNDER, left),
            target(MIDDLE, right),
            target(TOP, wide),
        ]));

        wheel(&mut h, Axis::V, -1, x, 2);

        assert_eq!(attempted_owners(h.app()), [TOP, expected]);
        assert_eq!(h.app().service_attempts[0].position, Position::new(x, 2));
    }
}

#[test]
fn each_owner_is_offered_once_and_exact_duplicate_targets_collapse() {
    let first_owner = Id::root("wheel-hit-stack.multi-part");
    let mut h = harness(HitStackProbe::new([
        target(UNDER, SCREEN),
        Target::new(first_owner, LABEL, SCREEN, Registration::HitStack),
        Target::new(first_owner, BODY, SCREEN, Registration::HitStack),
    ]));
    wheel(&mut h, Axis::V, 1, 4, 3);
    assert_eq!(h.app().initial_attempts[0].owner, first_owner);
    assert_eq!(h.app().initial_attempts[0].part, BODY);
    assert_eq!(h.app().service_attempts.len(), 1);
    assert_eq!(h.app().service_attempts[0].owner, UNDER);

    let duplicate_owner = Id::root("wheel-hit-stack.duplicate");
    let duplicate = Target::new(duplicate_owner, LABEL, SCREEN, Registration::HitStack);
    let mut h = harness(HitStackProbe::new([
        target(UNDER, SCREEN),
        duplicate,
        duplicate,
    ]));
    wheel(&mut h, Axis::V, 1, 4, 3);
    assert_eq!(h.app().initial_attempts[0].owner, duplicate_owner);
    assert_eq!(h.app().service_attempts.len(), 1);
    assert_eq!(h.app().service_attempts[0].owner, UNDER);
}

#[test]
fn conflicting_same_owner_part_metadata_fails_closed() {
    let conflicting = [
        target(UNDER, SCREEN),
        Target::new(OWNER, LABEL, SCREEN, Registration::HitStack).source(10),
        Target::new(OWNER, LABEL, SCREEN, Registration::HitStack).source(11),
    ];
    let mut h = harness(HitStackProbe::new(conflicting));

    wheel(&mut h, Axis::V, 1, 4, 3);

    assert_eq!(h.app().initial_attempts.len(), 1);
    assert_eq!(h.app().initial_attempts[0].owner, OWNER);
    assert!(h.app().service_attempts.is_empty());

    let conflicting_geometry = [
        target(UNDER, SCREEN),
        Target::new(OWNER, LABEL, Rect::new(0, 0, 12, 8), Registration::HitStack),
        Target::new(OWNER, LABEL, Rect::new(4, 0, 12, 8), Registration::HitStack),
    ];
    let mut h = harness(HitStackProbe::new(conflicting_geometry));
    wheel(&mut h, Axis::V, 1, 6, 3);
    assert!(h.app().service_attempts.is_empty());

    let mut h = harness(HitStackProbe::new([
        target(UNDER, SCREEN),
        Target::new(OWNER, LABEL, SCREEN, Registration::HitStack).axes(termrock::Axes::V),
        Target::new(OWNER, LABEL, SCREEN, Registration::HitStack).axes(termrock::Axes::Both),
    ]));
    wheel(&mut h, Axis::V, 1, 6, 3);
    assert!(h.app().service_attempts.is_empty());

    // Conflicting same-part registrations remain ambiguous even when the
    // second registration does not contain this pointer position.
    let mut h = harness(HitStackProbe::new([
        target(UNDER, SCREEN),
        Target::new(OWNER, LABEL, Rect::new(0, 0, 8, 8), Registration::HitStack),
        Target::new(OWNER, LABEL, Rect::new(12, 0, 8, 8), Registration::HitStack),
    ]));
    wheel(&mut h, Axis::V, 1, 4, 3);
    assert!(h.app().service_attempts.is_empty());

    // An axis-mismatched registration still conflicts on the same logical
    // owner/part; otherwise registration order would change ambiguity.
    let mut h = harness(HitStackProbe::new([
        target(UNDER, SCREEN),
        Target::new(OWNER, LABEL, SCREEN, Registration::HitStack).axes(termrock::Axes::H),
        Target::new(OWNER, LABEL, SCREEN, Registration::HitStack).axes(termrock::Axes::V),
    ]));
    wheel(&mut h, Axis::V, 1, 6, 3);
    assert!(h.app().service_attempts.is_empty());
}

#[test]
fn ordinary_scroll_parts_passive_and_linked_routes_block_hit_stack_fallback() {
    for registration in [
        Registration::PlainScroll,
        Registration::Control,
        Registration::Part,
        Registration::PassiveReveal,
        Registration::Linked,
    ] {
        let middle = Target::new(MIDDLE, LABEL, SCREEN, registration);
        let mut h = harness(HitStackProbe::new([
            target(UNDER, SCREEN),
            middle,
            target(TOP, SCREEN),
        ]));

        wheel(&mut h, Axis::V, 1, 4, 3);

        assert_eq!(h.app().initial_attempts[0].owner, TOP, "{registration:?}");
        assert!(h.app().service_attempts.is_empty(), "{registration:?}");
    }
}

#[test]
fn an_unqualified_part_above_the_initial_target_stops_fallback() {
    let mut h = harness(HitStackProbe::new([
        target(UNDER, SCREEN),
        target(TOP, SCREEN),
        Target::new(MIDDLE, LABEL, SCREEN, Registration::Part),
    ]));

    wheel(&mut h, Axis::V, 1, 4, 3);

    assert_eq!(h.app().initial_attempts.len(), 1);
    assert_eq!(h.app().initial_attempts[0].owner, TOP);
    assert!(h.app().service_attempts.is_empty());
}

#[test]
fn default_scroll_stays_single_target_and_linked_parent_policy_stays_explicit() {
    let mut h = harness(HitStackProbe::new([
        Target::new(UNDER, CONTAINER, SCREEN, Registration::PlainScroll),
        Target::new(TOP, CONTAINER, SCREEN, Registration::PlainScroll),
    ]));
    wheel(&mut h, Axis::V, 1, 4, 3);
    assert!(h.app().initial_attempts.is_empty());
    assert_eq!(h.app().wheel_deliveries.len(), 1);
    assert_eq!(h.app().wheel_deliveries[0].owner, TOP);
    assert!(h.app().service_attempts.is_empty());

    let mut linked = HitStackProbe::new([
        Target::new(UNDER, CONTAINER, SCREEN, Registration::Linked),
        Target::new(TOP, CONTAINER, Rect::new(2, 1, 12, 6), Registration::Linked),
    ]);
    linked.links.push((1, 0));
    let mut h = harness(linked);
    wheel(&mut h, Axis::V, 1, 4, 3);
    assert_eq!(attempted_owners(h.app()), [TOP, UNDER]);
    assert_eq!(h.app().service_attempts[0].owner, UNDER);
}

#[test]
fn changed_or_consumed_target_stops_fallback() {
    for result in [WheelRouteResult::Consumed, WheelRouteResult::Changed] {
        let mut h = harness(HitStackProbe::new([
            target(UNDER, SCREEN),
            target(TOP, SCREEN).result(Some(result)),
        ]));
        wheel(&mut h, Axis::V, 1, 4, 3);
        assert_eq!(h.app().initial_attempts.len(), 1);
        assert_eq!(h.app().initial_attempts[0].owner, TOP);
        assert!(h.app().service_attempts.is_empty(), "{result:?}");
    }

    let mut h = harness(HitStackProbe::new([
        target(UNDER, SCREEN),
        target(MIDDLE, SCREEN).result(Some(WheelRouteResult::Consumed)),
        target(TOP, SCREEN),
    ]));
    wheel(&mut h, Axis::V, 1, 4, 3);
    assert_eq!(attempted_owners(h.app()), [TOP, MIDDLE]);
    assert_eq!(h.app().service_ack_results, [true]);
}

#[test]
fn both_axes_signed_and_zero_delta_and_modifiers_are_preserved() {
    for (axis, delta) in [(Axis::V, 1), (Axis::H, -2), (Axis::V, 0)] {
        let mut h = harness(HitStackProbe::new([
            target(UNDER, SCREEN),
            target(TOP, SCREEN),
        ]));
        wheel_with_modifiers(
            &mut h,
            axis,
            delta,
            4,
            3,
            KeyModifiers::SHIFT | KeyModifiers::SUPER,
        );
        let initial = h.app().initial_attempts[0];
        let fallback = h.app().service_attempts[0];
        assert_eq!(initial.axis, axis);
        assert_eq!(fallback.axis, axis);
        assert_eq!(
            initial.delta,
            delta * Theme::junie().design.motion.wheel_rows as i16
        );
        assert_eq!(fallback.delta, initial.delta);
        assert_eq!(initial.modifiers, KeyModifiers::SHIFT | KeyModifiers::SUPER);
        assert_eq!(fallback.modifiers, initial.modifiers);
    }

    let mut h = harness(HitStackProbe::new([
        target(UNDER, SCREEN),
        target(TOP, SCREEN).axes(termrock::Axes::V),
    ]));
    wheel(&mut h, Axis::H, 1, 4, 3);
    assert_eq!(h.app().initial_attempts.len(), 1);
    assert_eq!(h.app().initial_attempts[0].owner, UNDER);
    assert!(h.app().service_attempts.is_empty());
}

#[test]
fn scroll_only_part_enrollment_preserves_click_focus_and_capture() {
    let mut app = HitStackProbe::new([
        target(UNDER, SCREEN),
        Target::new(OWNER, LABEL, SCREEN, Registration::ControlPartHitStack),
    ]);
    app.capture_on_press = true;
    let mut h = harness(app);

    wheel(&mut h, Axis::V, 1, 4, 3);
    assert_eq!(h.app().initial_attempts[0].owner, OWNER);
    assert_eq!(h.app().initial_attempts[0].part, LABEL);
    assert_eq!(h.app().service_attempts[0].owner, UNDER);

    let _ = h.click(4, 3);
    assert_eq!(h.focus(), Some(OWNER));
    assert!(
        h.app()
            .pointer_deliveries
            .iter()
            .any(|(owner, part, phase)| {
                *owner == OWNER && *part == LABEL && *phase == Phase::Click
            })
    );
    assert!(h.app().capture_results.iter().all(|captured| *captured));

    h.app_mut().pointer_deliveries.clear();
    let _ = h.drag((4, 3), (19, 7));
    assert!(
        h.app()
            .pointer_deliveries
            .iter()
            .any(|(owner, part, phase)| {
                *owner == OWNER && *part == LABEL && *phase == Phase::Drag
            })
    );
}

#[test]
fn fallback_is_confined_to_the_exact_top_layer() {
    let mut app = HitStackProbe::new([target(UNDER, SCREEN), target(TOP, SCREEN).on_layer(MODAL)]);
    app.open_modal_on_bootstrap = true;
    let mut h = harness(app);

    wheel(&mut h, Axis::V, 1, 4, 3);

    assert!(h.is_open(MODAL));
    assert_eq!(h.app().initial_attempts[0].owner, TOP);
    assert!(h.app().service_attempts.is_empty());
}

#[test]
fn opening_a_modal_during_the_event_invalidates_the_pending_stack() {
    let mut app = HitStackProbe::new([target(UNDER, SCREEN), target(TOP, SCREEN)]);
    app.open_modal_after_event = true;
    let mut h = harness(app);

    wheel(&mut h, Axis::V, 1, 4, 3);

    assert!(h.is_open(MODAL));
    assert_eq!(h.app().event_updates, 1);
    assert!(h.app().service_attempts.is_empty());
}

#[test]
fn closing_the_top_modal_during_the_event_invalidates_fallback() {
    let mut app = HitStackProbe::new([
        target(PAGE_SENTINEL, SCREEN),
        target(MODAL_UNDER, SCREEN).on_layer(MODAL),
        target(TOP, SCREEN).on_layer(MODAL),
    ]);
    app.open_modal_on_bootstrap = true;
    app.close_modal_after_event = true;
    let mut h = harness(app);

    wheel(&mut h, Axis::V, 1, 4, 3);

    assert!(!h.is_open(MODAL));
    assert_eq!(h.app().event_updates, 1);
    assert_eq!(h.app().initial_attempts[0].owner, TOP);
    assert!(
        !h.app()
            .initial_attempts
            .iter()
            .any(|attempt| attempt.owner == PAGE_SENTINEL)
    );
    assert!(h.app().service_attempts.is_empty());

    let mut app = HitStackProbe::new([
        target(PAGE_SENTINEL, SCREEN),
        target(MODAL_UNDER, SCREEN).on_layer(MODAL),
        target(TOP, SCREEN).on_layer(MODAL),
    ]);
    app.open_modal_on_bootstrap = true;
    let mut h = harness(app);
    wheel(&mut h, Axis::V, 1, 4, 3);

    assert!(h.is_open(MODAL));
    assert_eq!(h.app().event_updates, 1);
    assert_eq!(attempted_owners(h.app()), [TOP, MODAL_UNDER]);
    assert!(!attempted_owners(h.app()).contains(&PAGE_SENTINEL));
    assert_eq!(h.app().service_attempts.len(), 1);
}

#[test]
fn capture_change_during_the_event_invalidates_fallback() {
    let mut app = HitStackProbe::new([
        target(UNDER, SCREEN),
        Target::new(OWNER, LABEL, SCREEN, Registration::ControlPartHitStack),
    ]);
    app.capture_on_press = true;
    app.release_capture_on_wheel = true;
    let mut h = harness(app);

    let _ = h.mouse(MouseKind::Down, 4, 3);
    assert_eq!(h.app().capture_results, [true]);

    wheel(&mut h, Axis::V, 1, 4, 3);

    assert!(h.app().service_attempts.is_empty());
}

#[test]
fn stale_next_source_is_offered_but_cannot_acknowledge_or_continue() {
    let mut app = HitStackProbe::new([
        target(UNDER, SCREEN).source(31),
        target(MIDDLE, SCREEN).source(22),
        target(TOP, SCREEN).source(11),
    ]);
    app.live_source_overrides.push((UNDER, CONTAINER, 32));
    let mut h = harness(app);

    wheel(&mut h, Axis::V, 1, 4, 3);

    assert_eq!(attempted_owners(h.app()), [TOP, MIDDLE, UNDER]);
    assert_eq!(h.app().service_attempts.last().unwrap().source_revision, 31);
    assert_eq!(h.app().service_ack_results, [true]);
    assert!(h.app().service_intents_empty);
}

#[test]
fn layout_or_focus_change_invalidates_fallback_before_targeted_service() {
    let targets = [
        target(UNDER, SCREEN),
        target(TOP, SCREEN),
        Target::new(
            FOCUS_TARGET,
            CONTAINER,
            Rect::new(0, 6, 4, 2),
            Registration::FocusableControl,
        ),
    ];
    let mut h = harness(HitStackProbe::new(targets));
    wheel(&mut h, Axis::V, 1, 4, 3);
    assert_eq!(h.app().event_updates, 1);
    assert_eq!(attempted_owners(h.app()), [TOP, UNDER]);
    assert_eq!(h.app().service_attempts.len(), 1);

    let mut app = HitStackProbe::new(targets);
    app.relayout_after_event = true;
    let mut h = harness(app);
    wheel(&mut h, Axis::V, 1, 4, 3);
    assert_eq!(h.app().event_updates, 1);
    assert_eq!(h.app().initial_attempts[0].owner, TOP);
    assert!(h.app().service_attempts.is_empty());

    let mut app = HitStackProbe::new(targets);
    app.focus_after_event = Some(FOCUS_TARGET);
    let mut h = harness(app);
    wheel(&mut h, Axis::V, 1, 4, 3);
    assert_eq!(h.app().event_updates, 1);
    assert_eq!(h.app().initial_attempts[0].owner, TOP);
    assert_eq!(h.focus(), Some(FOCUS_TARGET));
    assert!(h.app().service_attempts.is_empty());
}
