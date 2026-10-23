use ratatui_core::layout::{Position, Rect};
use termrock::runtime::WheelRouteAttempt;
use termrock::{
    App, Axis, Cx, Id, LayerSpec, Part, PartRef, Response, ScrollRegion, ScrollState, Theme, Ui,
    UpdateCause,
};
use termrock_test_support::Harness;

const PARENT: Id = Id::root("scroll-route.parent");
const CHILD: Id = Id::root("scroll-route.child");
const MODAL: Id = Id::root("scroll-route.modal");
const PARENT_AREA: Rect = Rect::new(0, 0, 24, 12);
const CHILD_AREA: Rect = Rect::new(4, 3, 12, 5);
const CONTAINER: PartRef = PartRef::of(Part::CONTAINER);
const INITIAL_REVISION: u64 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UpdateOrder {
    ParentFirst,
    ChildFirst,
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
            source_revision: attempt.source_revision(),
        }
    }
}

struct RouteApp {
    parent: ScrollState,
    child: ScrollState,
    parent_len: usize,
    child_len: usize,
    parent_revision: u64,
    child_revision: u64,
    order: UpdateOrder,
    routed: bool,
    change_child_revision_on_event: bool,
    change_parent_revision_before_service: bool,
    open_modal_after_event: bool,
    event_updates: usize,
    tick_updates: usize,
    route_callbacks: usize,
    route_callbacks_empty: bool,
    update_attempts: Vec<AttemptRecord>,
    service_attempts: Vec<AttemptRecord>,
    update_responses: Vec<(Id, bool, bool)>,
    linked: std::cell::Cell<bool>,
}

impl RouteApp {
    fn new(
        parent_len: usize,
        child_len: usize,
        parent_offset: usize,
        child_offset: usize,
        order: UpdateOrder,
    ) -> Self {
        Self {
            parent: scroll_state(parent_len, PARENT_AREA.height.into(), parent_offset),
            child: scroll_state(child_len, CHILD_AREA.height.into(), child_offset),
            parent_len,
            child_len,
            parent_revision: INITIAL_REVISION,
            child_revision: INITIAL_REVISION,
            order,
            routed: true,
            change_child_revision_on_event: false,
            change_parent_revision_before_service: false,
            open_modal_after_event: false,
            event_updates: 0,
            tick_updates: 0,
            route_callbacks: 0,
            route_callbacks_empty: true,
            update_attempts: Vec::new(),
            service_attempts: Vec::new(),
            update_responses: Vec::new(),
            linked: std::cell::Cell::new(false),
        }
    }

    fn update_one(&mut self, cx: &mut Cx<'_>, owner: Id) -> Response<()> {
        let region = ScrollRegion::new(owner).scrollbar_visible(false);
        if let Some(attempt) = cx.wheel_route_attempt(owner, CONTAINER, Axis::V) {
            self.update_attempts.push(attempt.into());
        }
        let (state, content_len, source_revision) = if owner == PARENT {
            (&mut self.parent, self.parent_len, self.parent_revision)
        } else {
            (&mut self.child, self.child_len, self.child_revision)
        };
        let response = if self.routed {
            region.update_routed(cx, state, content_len, source_revision)
        } else {
            region.update(cx, state, content_len)
        };
        self.update_responses
            .push((owner, response.is_consumed(), response.is_changed()));
        response
    }

    fn update_regions(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        match self.order {
            UpdateOrder::ParentFirst => self.update_one(cx, PARENT) | self.update_one(cx, CHILD),
            UpdateOrder::ChildFirst => self.update_one(cx, CHILD) | self.update_one(cx, PARENT),
        }
    }
}

impl App for RouteApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let cause = cx.update_cause();
        match cause {
            UpdateCause::Event => {
                self.event_updates += 1;
                if self.change_child_revision_on_event {
                    self.child_revision = self.child_revision.saturating_add(1);
                    self.change_child_revision_on_event = false;
                }
            }
            UpdateCause::Tick => self.tick_updates += 1,
            _ => {}
        }
        let response = self.update_regions(cx);
        if cause == UpdateCause::Event && self.open_modal_after_event {
            self.open_modal_after_event = false;
            cx.open_layer(MODAL, LayerSpec::modal(MODAL));
        }
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let parent = ScrollRegion::new(PARENT).scrollbar_visible(false);
        let child = ScrollRegion::new(CHILD).scrollbar_visible(false);
        let parent_area = ui.full();
        let child_area = child_area(parent_area);
        if !self.routed {
            parent.draw(ui, parent_area, &self.parent, self.parent_len);
            child.draw(ui, child_area, &self.child, self.child_len);
            self.linked.set(false);
            return;
        }
        let (_, parent_key) = parent.draw_routed(
            ui,
            parent_area,
            &self.parent,
            self.parent_len,
            self.parent_revision,
        );
        let (_, child_key) = child.draw_routed(
            ui,
            child_area,
            &self.child,
            self.child_len,
            self.child_revision,
        );
        let linked = child_key
            .zip(parent_key)
            .is_some_and(|(child, parent)| ui.link_scroll_parent(child, parent));
        self.linked.set(linked);
    }

    fn route_wheel_attempt(&mut self, cx: &mut Cx<'_>, attempt: WheelRouteAttempt) -> Response<()> {
        self.route_callbacks += 1;
        self.route_callbacks_empty &= cx.update_cause() == UpdateCause::ScrollRoute
            && cx.intents(PARENT).next().is_none()
            && cx.intents(CHILD).next().is_none();
        self.service_attempts.push(attempt.into());
        if attempt.owner() == PARENT {
            if self.change_parent_revision_before_service {
                self.parent_revision = self.parent_revision.saturating_add(1);
                self.change_parent_revision_before_service = false;
            }
            ScrollRegion::new(PARENT).service_routed_wheel(
                cx,
                &mut self.parent,
                self.parent_len,
                self.parent_revision,
            )
        } else if attempt.owner() == CHILD {
            ScrollRegion::new(CHILD).service_routed_wheel(
                cx,
                &mut self.child,
                self.child_len,
                self.child_revision,
            )
        } else {
            Response::ignored()
        }
    }
}

fn scroll_state(content_len: usize, viewport_len: usize, offset: usize) -> ScrollState {
    let mut state = ScrollState::new(content_len);
    state.set_viewport(viewport_len);
    state.scroll_to(offset);
    state
}

fn child_area(parent: Rect) -> Rect {
    Rect::new(
        parent.x.saturating_add(4),
        parent.y.saturating_add(3),
        parent.width.saturating_sub(8).min(12),
        parent.height.saturating_sub(7).max(1),
    )
}

fn harness(app: RouteApp) -> Harness<RouteApp> {
    Harness::new(app, Theme::junie(), PARENT_AREA.width, PARENT_AREA.height)
}

fn wheel(harness: &mut Harness<RouteApp>, delta: i16) -> Response<()> {
    harness.wheel(Axis::V, delta, CHILD_AREA.x + 1, CHILD_AREA.y + 1)
}

fn latest_update_response(app: &RouteApp, owner: Id) -> (bool, bool) {
    let (response_owner, consumed, changed) = app
        .update_responses
        .iter()
        .rev()
        .find(|(response_owner, _, _)| *response_owner == owner)
        .copied()
        .expect("the region update records a response");
    assert_eq!(response_owner, owner);
    (consumed, changed)
}

#[test]
fn ignored_child_falls_back_once_in_both_app_update_orders() {
    for order in [UpdateOrder::ParentFirst, UpdateOrder::ChildFirst] {
        let app = RouteApp::new(60, usize::from(CHILD_AREA.height), 3, 0, order);
        let mut h = harness(app);
        assert!(
            h.app().linked.get(),
            "nested registrations link in {order:?}"
        );
        let parent_before = h.app().parent.offset();
        let child_before = h.app().child.offset();

        let response = wheel(&mut h, 1);

        assert!(response.is_changed(), "the containing region scrolls");
        assert_eq!(h.app().child.offset(), child_before);
        assert_eq!(
            h.app().parent.offset(),
            parent_before + h.app().service_attempts[0].delta as usize
        );
        assert_eq!(h.app().event_updates, 1, "one ordinary App event update");
        assert_eq!(h.app().tick_updates, 0, "wheel does not create a Tick");
        assert_eq!(h.app().route_callbacks, 1);
        assert!(h.app().route_callbacks_empty);
        assert_eq!(h.app().update_attempts.len(), 1);
        assert_eq!(h.app().update_attempts[0].owner, CHILD);
        assert_eq!(h.app().service_attempts.len(), 1);
        assert_eq!(h.app().service_attempts[0].owner, PARENT);
        assert_eq!(h.app().service_attempts[0].attempt_index, 1);
        assert_eq!(
            h.app().service_attempts[0].event_id,
            h.app().update_attempts[0].event_id
        );
        assert_eq!(
            h.app().service_attempts[0].delta,
            h.app().update_attempts[0].delta
        );
        assert_eq!(
            h.app().service_attempts[0].position,
            h.app().update_attempts[0].position
        );
        assert_eq!(h.app().update_attempts[0].axis, Axis::V);
        assert_eq!(h.app().update_attempts[0].part, CONTAINER);
        assert_eq!(
            h.app().service_attempts[0].source_revision,
            INITIAL_REVISION
        );
    }
}

#[test]
fn changed_child_stops_fallback_and_moves_by_the_attempt_delta() {
    let mut h = harness(RouteApp::new(60, 40, 3, 4, UpdateOrder::ChildFirst));
    let parent_before = h.app().parent.offset();
    let child_before = h.app().child.offset();

    let response = wheel(&mut h, 1);

    let delta = h.app().update_attempts[0].delta;
    assert!(response.is_changed());
    assert_eq!(h.app().child.offset(), child_before + delta as usize);
    assert_eq!(h.app().parent.offset(), parent_before);
    assert_eq!(h.app().route_callbacks, 0);
    assert_eq!(latest_update_response(h.app(), CHILD), (true, true));
}

#[test]
fn overflowing_child_consumes_both_saturated_edges() {
    for (offset, delta) in [(0, -1), (35, 1)] {
        let mut h = harness(RouteApp::new(60, 40, 3, offset, UpdateOrder::ParentFirst));
        let child_before = h.app().child;
        let parent_before = h.app().parent.offset();

        let response = wheel(&mut h, delta);

        assert!(response.is_consumed());
        assert!(!response.is_changed());
        assert_eq!(h.app().child, child_before);
        assert_eq!(h.app().parent.offset(), parent_before);
        assert_eq!(h.app().route_callbacks, 0);
        assert_eq!(latest_update_response(h.app(), CHILD), (true, false));
    }
}

#[test]
fn all_ignored_nested_targets_terminate_after_the_parent() {
    let mut h = harness(RouteApp::new(
        usize::from(PARENT_AREA.height),
        usize::from(CHILD_AREA.height),
        0,
        0,
        UpdateOrder::ChildFirst,
    ));

    let response = wheel(&mut h, 1);

    assert!(!response.is_changed());
    assert_eq!(h.app().route_callbacks, 1);
    assert_eq!(h.app().service_attempts.len(), 1);
    assert_eq!(h.app().service_attempts[0].owner, PARENT);
    assert_eq!(h.app().service_attempts[0].attempt_index, 1);
    assert!(h.app().route_callbacks_empty);
}

#[test]
fn stale_initial_source_revision_rejects_before_scroll_preparation() {
    let mut app = RouteApp::new(60, 40, 3, 4, UpdateOrder::ChildFirst);
    app.change_child_revision_on_event = true;
    let mut h = harness(app);
    let child_before = h.app().child;
    let parent_before = h.app().parent;

    let _ = wheel(&mut h, 1);

    assert_eq!(h.app().child, child_before);
    assert_eq!(h.app().parent, parent_before);
    assert_eq!(h.app().event_updates, 1);
    assert_eq!(
        h.app().route_callbacks,
        0,
        "stale target does not fall through"
    );
    assert_eq!(h.app().update_attempts.len(), 1);
    assert_eq!(h.app().update_attempts[0].source_revision, INITIAL_REVISION);
}

#[test]
fn parent_source_change_after_child_ignored_rejects_without_mutation() {
    let mut app = RouteApp::new(
        60,
        usize::from(CHILD_AREA.height),
        3,
        0,
        UpdateOrder::ParentFirst,
    );
    app.change_parent_revision_before_service = true;
    let mut h = harness(app);
    let parent_before = h.app().parent;

    let _ = wheel(&mut h, 1);

    assert_eq!(h.app().parent, parent_before);
    assert_eq!(h.app().event_updates, 1);
    assert_eq!(h.app().route_callbacks, 1);
    assert!(h.app().route_callbacks_empty);
    assert_eq!(h.app().service_attempts[0].owner, PARENT);
    assert_eq!(
        h.app().service_attempts[0].source_revision,
        INITIAL_REVISION
    );
}

#[test]
fn resize_rebuilds_nested_scroll_geometry_and_uses_the_new_viewport() {
    let mut h = harness(RouteApp::new(
        60,
        usize::from(CHILD_AREA.height),
        3,
        0,
        UpdateOrder::ChildFirst,
    ));
    assert!(h.app().linked.get());

    let _ = h.resize(PARENT_AREA.width, 10);
    let parent_before = h.app().parent.offset();
    let child_before = h.app().child.offset();

    let response = wheel(&mut h, 1);

    assert!(response.is_changed());
    assert!(
        h.app().linked.get(),
        "the resized frame rebuilds its route link"
    );
    assert_eq!(h.app().child.viewport_len(), 3);
    assert!(h.app().child.offset() > child_before);
    assert_eq!(h.app().parent.offset(), parent_before);
    assert_eq!(h.app().route_callbacks, 0);
}

#[test]
fn modal_opened_after_child_ignored_invalidates_parent_fallback() {
    let mut app = RouteApp::new(
        60,
        usize::from(CHILD_AREA.height),
        3,
        0,
        UpdateOrder::ChildFirst,
    );
    app.open_modal_after_event = true;
    let mut h = harness(app);
    let parent_before = h.app().parent;
    let child_before = h.app().child;

    let _ = wheel(&mut h, 1);

    assert!(h.is_open(MODAL));
    assert_eq!(h.app().parent, parent_before);
    assert_eq!(h.app().child, child_before);
    assert_eq!(h.app().route_callbacks, 0);
    assert_eq!(h.app().event_updates, 1);
}

#[test]
fn ordinary_scroll_region_methods_remain_single_target() {
    let mut app = RouteApp::new(60, 40, 3, 4, UpdateOrder::ChildFirst);
    app.routed = false;
    let mut h = harness(app);
    let parent_before = h.app().parent.offset();
    let child_before = h.app().child.offset();

    let response = wheel(&mut h, 1);

    let wheel_rows = usize::from(Theme::junie().design.motion.wheel_rows);
    assert!(response.is_changed());
    assert_eq!(h.app().child.offset(), child_before + wheel_rows);
    assert_eq!(h.app().parent.offset(), parent_before);
    assert_eq!(h.app().route_callbacks, 0);
    assert!(h.app().update_attempts.is_empty());
}
