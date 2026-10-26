#![allow(
    clippy::unwrap_used,
    reason = "focused native routing contract assertions"
)]

use core::num::NonZeroU16;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use termrock_collections::ScrollState;
use termrock_core::EditEntry;
use termrock_fields::{
    BlurPolicy, EditPhase, TextAction, TextArea, TextAreaIdleHorizontalScroll, TextAreaState,
    TextEditCommand,
};
use termrock_runtime::author::VerticalWindow;
use termrock_runtime::event::{Axis, Input, KeyCode, KeyModifiers, Mouse, MouseKind};
use termrock_runtime::hit::Axes;
use termrock_runtime::runtime::WheelRouteAttempt;
use termrock_runtime::{
    App, Cx, FrameRead, Id, Part, PartRef, Response, Ui, UpdateCause, WheelRouteResult,
};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const FIELD: Id = Id::root("textarea.idle-horizontal");
const PARENT: Id = Id::root("textarea.idle-horizontal.parent");
const PARENT_PART: PartRef = PartRef::of(Part::CONTAINER);
const TEXT_PART: PartRef = PartRef::of(Part::TEXT);
const SCREEN: Rect = Rect::new(0, 0, 48, 12);
const WINDOW_AREA: Rect = Rect::new(3, 3, 30, 2);
const LOGICAL_ROWS: usize = 5;
const WINDOW_FIRST_ROW: usize = 1;
const PARENT_REVISION: u64 = 41;

const LONG_VALUE: &str = concat!(
    "zero-row\n",
    "one-row\n",
    "A0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ\n",
    "BabcdefghijKLMNOPQRSTUVWXYZ0123456789\n",
    "four-row\n",
    "five-row\n",
    "six-row\n",
    "seven-row",
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AttemptRecord {
    event_id: u64,
    attempt_index: u32,
    owner: Id,
    part: PartRef,
    axis: Axis,
    delta: i16,
    unscaled_delta: i16,
    modifiers: KeyModifiers,
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
            unscaled_delta: attempt.unscaled_delta(),
            modifiers: attempt.modifiers(),
            position: attempt.position(),
            source_revision: attempt.source_revision(),
        }
    }
}

struct IdleTextAreaApp {
    state: TextAreaState,
    value: String,
    policy: Option<TextAreaIdleHorizontalScroll>,
    edit_entry: EditEntry,
    esc_reverts: bool,
    blur_policy: BlurPolicy,
    disabled: bool,
    source_revision: u64,
    parent_columns: ScrollState,
    field_initial_attempts: Vec<AttemptRecord>,
    parent_initial_attempts: Vec<AttemptRecord>,
    route_attempts: Vec<AttemptRecord>,
    route_callbacks: usize,
    route_callbacks_empty: bool,
    actions: Vec<Option<TextAction>>,
    copy_next: bool,
    copy_result: Option<bool>,
    copied_text: Option<String>,
    source_replacement: Option<String>,
    observe_layout_once: bool,
    observed_layout_offsets: Vec<Option<usize>>,
}

impl IdleTextAreaApp {
    fn new(value: impl Into<String>, policy: Option<TextAreaIdleHorizontalScroll>) -> Self {
        let mut state = TextAreaState::default();
        state.scroll_mut().set_content(8);
        state.scroll_mut().set_viewport(LOGICAL_ROWS);
        state.scroll_mut().scroll_to(1);

        let mut parent_columns = ScrollState::new(1024);
        parent_columns.set_viewport(usize::from(SCREEN.width));
        parent_columns.scroll_to(12);

        Self {
            state,
            value: value.into(),
            policy,
            edit_entry: EditEntry::Explicit,
            esc_reverts: false,
            blur_policy: BlurPolicy::Commit,
            disabled: false,
            source_revision: 1,
            parent_columns,
            field_initial_attempts: Vec::new(),
            parent_initial_attempts: Vec::new(),
            route_attempts: Vec::new(),
            route_callbacks: 0,
            route_callbacks_empty: true,
            actions: Vec::new(),
            copy_next: false,
            copy_result: None,
            copied_text: None,
            source_replacement: None,
            observe_layout_once: false,
            observed_layout_offsets: Vec::new(),
        }
    }

    fn field(&self) -> TextArea<'static> {
        let field = TextArea::new(FIELD, LOGICAL_ROWS as u16)
            .edit_entry(self.edit_entry)
            .esc_reverts(self.esc_reverts)
            .blur(self.blur_policy)
            .disabled(self.disabled);
        match self.policy {
            Some(policy) => field.idle_horizontal_scroll(policy, self.source_revision),
            None => field,
        }
    }

    fn service_parent(&mut self, cx: &mut Cx<'_>, attempt: WheelRouteAttempt) -> Response<()> {
        if attempt.owner() != PARENT
            || attempt.part() != PARENT_PART
            || attempt.axis() != Axis::H
            || attempt.source_revision() != PARENT_REVISION
            || cx.wheel_route_attempt(PARENT, PARENT_PART, Axis::H) != Some(attempt)
        {
            return Response::ignored();
        }

        let mut next = self.parent_columns;
        next.scroll_by(isize::from(attempt.delta()));
        let result = if next.offset() != self.parent_columns.offset() {
            WheelRouteResult::Changed
        } else {
            WheelRouteResult::Consumed
        };
        if !cx.acknowledge_wheel_route(&attempt, result) {
            return Response::ignored();
        }
        self.parent_columns = next;
        match result {
            WheelRouteResult::Changed => Response::changed().for_id(PARENT),
            WheelRouteResult::Consumed => Response::consumed().for_id(PARENT),
            WheelRouteResult::Ignored => Response::ignored(),
            other => panic!("unexpected wheel route result: {other:?}"),
        }
    }
}

impl App for IdleTextAreaApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.observe_layout_once {
            self.observe_layout_once = false;
            self.observed_layout_offsets.push(
                cx.layout(FIELD)
                    .and_then(|layout| layout.horizontal_source_offset),
            );
        }
        if let Some(replacement) = self.source_replacement.take() {
            self.value = replacement;
            self.source_revision += 1;
        }
        if let Some(attempt) = cx.wheel_route_attempt(PARENT, PARENT_PART, Axis::H) {
            self.parent_initial_attempts.push(attempt.into());
            return self.service_parent(cx, attempt);
        }

        if let Some(attempt) = cx.wheel_route_attempt(FIELD, TEXT_PART, Axis::H) {
            self.field_initial_attempts.push(attempt.into());
        }
        let field = self.field();
        if self.copy_next {
            self.copy_next = false;
            self.copy_result = Some(field.execute_edit_command(
                cx,
                &mut self.state,
                &mut self.value,
                TextEditCommand::Copy,
            ));
            self.copied_text = cx.clipboard_text().map(str::to_owned);
        }
        let response = field.update(cx, &mut self.state, &mut self.value);
        if cx.update_cause() == UpdateCause::Event {
            self.actions.push(response.action_ref().copied());
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let _ =
            ui.register_hit_stack_wheel(PARENT, PARENT_PART, ui.full(), Axes::H, PARENT_REVISION);
        let window = VerticalWindow::new(WINDOW_AREA, LOGICAL_ROWS, WINDOW_FIRST_ROW)
            .expect("the test window is nonempty and inside the logical rows");
        self.field()
            .value(&self.value)
            .draw_window(ui, window, &self.state);
    }

    fn route_wheel_attempt(&mut self, cx: &mut Cx<'_>, attempt: WheelRouteAttempt) -> Response<()> {
        self.route_callbacks += 1;
        self.route_callbacks_empty &= cx.update_cause() == UpdateCause::ScrollRoute
            && cx.intents(FIELD).next().is_none()
            && cx.intents(PARENT).next().is_none();
        self.route_attempts.push(attempt.into());
        if attempt.owner() == FIELD {
            self.field()
                .service_idle_horizontal_wheel(cx, &mut self.state, &self.value, attempt)
        } else if attempt.owner() == PARENT {
            self.service_parent(cx, attempt)
        } else {
            Response::ignored()
        }
    }
}

fn harness(app: IdleTextAreaApp) -> Harness<IdleTextAreaApp> {
    Harness::new(app, Theme::junie(), SCREEN.width, SCREEN.height)
}

fn text_area(harness: &Harness<IdleTextAreaApp>) -> Rect {
    harness
        .area_of_part(FIELD, TEXT_PART)
        .expect("the clipped TextArea text part is registered")
}

fn pan(step: u16) -> TextAreaIdleHorizontalScroll {
    TextAreaIdleHorizontalScroll::pan(NonZeroU16::new(step).expect("test steps are positive"))
}

fn wheel(harness: &mut Harness<IdleTextAreaApp>, delta: i16) -> Response<()> {
    wheel_with_modifiers(harness, delta, KeyModifiers::NONE)
}

fn wheel_with_modifiers(
    harness: &mut Harness<IdleTextAreaApp>,
    delta: i16,
    mods: KeyModifiers,
) -> Response<()> {
    let text = text_area(harness);
    harness.handle(Input::Mouse(Mouse {
        kind: MouseKind::Wheel(Axis::H, delta),
        pos: Position::new(text.x + 2, text.y),
        mods,
    }))
}

fn assert_idle_invariants(harness: &Harness<IdleTextAreaApp>, original: &str, v_offset: usize) {
    assert_eq!(harness.app().value, original);
    assert_eq!(harness.app().state.phase(), EditPhase::Idle);
    assert_eq!(harness.app().state.scroll().offset(), v_offset);
    assert!(harness.app().actions.iter().all(Option::is_none));
    assert_eq!(harness.cursor(), None, "idle wheel never publishes a caret");
}

#[test]
fn opted_in_h_wheel_pans_by_decoded_units_and_preserves_the_clipped_rows() {
    let mut app = IdleTextAreaApp::new(LONG_VALUE, Some(pan(4)));
    app.source_revision = 9;
    let mut h = harness(app);
    let text = text_area(&h);
    assert!(text.x > 0 && text.y > 0, "the target has a nonzero origin");
    assert_eq!(h.cell(text.x, text.y - 1).symbol(), " ");
    assert_eq!(h.cell(text.x, text.y + text.height).symbol(), " ");
    assert!(
        h.app().state.scroll().offset() == 1,
        "the child V offset is seeded"
    );
    let original = h.app().value.clone();
    let v_offset = h.app().state.scroll().offset();

    let response = wheel(&mut h, 1);
    assert!(response.is_changed());
    assert_eq!(h.app().state.idle_horizontal_offset(), 4);
    let initial = h.app().field_initial_attempts[0];
    assert_eq!(initial.owner, FIELD);
    assert_eq!(initial.part, TEXT_PART);
    assert_eq!(initial.axis, Axis::H);
    assert_eq!(initial.attempt_index, 0);
    assert_eq!(initial.delta, 3, "Junie keeps its existing scaled delta");
    assert_eq!(initial.unscaled_delta, 1);
    assert_eq!(initial.source_revision, 9);
    assert_eq!(h.cell(text.x, text.y).symbol(), "3");
    assert_eq!(h.cell(text.x, text.y + 1).symbol(), "d");
    assert_idle_invariants(&h, &original, v_offset);

    let response = wheel(&mut h, 1);
    assert!(response.is_changed());
    let second = h.app().field_initial_attempts[1];
    assert_eq!(second.delta, 3);
    assert_eq!(second.unscaled_delta, 1);
    assert_ne!(second.event_id, initial.event_id);
    assert_eq!(h.app().state.idle_horizontal_offset(), 8);
    assert_eq!(h.cell(text.x, text.y).symbol(), "7");
    assert_eq!(h.cell(text.x, text.y + 1).symbol(), "h");
    assert_idle_invariants(&h, &original, v_offset);

    assert!(wheel(&mut h, -1).is_changed());
    assert_eq!(h.app().state.idle_horizontal_offset(), 4);
    assert_eq!(h.cell(text.x, text.y).symbol(), "3");
    assert!(wheel(&mut h, -1).is_changed());
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    assert_eq!(h.cell(text.x, text.y).symbol(), "A");
    assert_idle_invariants(&h, &original, v_offset);

    assert!(wheel(&mut h, 2).is_changed());
    let two_unit = h.app().field_initial_attempts[4];
    assert_eq!(two_unit.delta, 6);
    assert_eq!(two_unit.unscaled_delta, 2);
    assert_eq!(h.app().state.idle_horizontal_offset(), 8);
    assert_eq!(h.cell(text.x, text.y).symbol(), "7");
    assert_eq!(h.cell(text.x, text.y + 1).symbol(), "h");
    assert_idle_invariants(&h, &original, v_offset);
    assert_eq!(h.app().field_initial_attempts.len(), 5);
    assert!(
        h.app().route_attempts.is_empty(),
        "the child handles all five events"
    );
    assert_eq!(
        h.app()
            .field_initial_attempts
            .iter()
            .map(|attempt| (attempt.delta, attempt.unscaled_delta))
            .collect::<Vec<_>>(),
        vec![(3, 1), (3, 1), (-3, -1), (-3, -1), (6, 2)]
    );
    assert!(
        h.app()
            .field_initial_attempts
            .windows(2)
            .all(|events| events[0].event_id != events[1].event_id)
    );
}

#[test]
fn explicit_off_and_fitting_text_decline_to_a_moving_parent_model() {
    for (value, policy) in [
        (LONG_VALUE.to_owned(), TextAreaIdleHorizontalScroll::Off),
        ("short".to_owned(), pan(4)),
    ] {
        let mut h = harness(IdleTextAreaApp::new(value.clone(), Some(policy)));
        let parent_before = h.app().parent_columns.offset();
        let v_before = h.app().state.scroll().offset();
        assert!(wheel(&mut h, 1).is_changed());

        assert_eq!(h.app().field_initial_attempts.len(), 1);
        assert_eq!(h.app().field_initial_attempts[0].owner, FIELD);
        assert_eq!(h.app().field_initial_attempts[0].attempt_index, 0);
        assert_eq!(h.app().route_callbacks, 1);
        assert!(h.app().route_callbacks_empty);
        assert_eq!(h.app().route_attempts.len(), 1);
        let parent = h.app().route_attempts[0];
        assert_eq!(parent.owner, PARENT);
        assert_eq!(parent.part, PARENT_PART);
        assert_eq!(parent.axis, Axis::H);
        assert_eq!(parent.attempt_index, 1);
        assert_eq!(parent.event_id, h.app().field_initial_attempts[0].event_id);
        assert_eq!(parent.delta, h.app().field_initial_attempts[0].delta);
        assert_eq!(parent.unscaled_delta, 1);
        assert_eq!(parent.source_revision, PARENT_REVISION);
        assert_eq!(h.app().parent_columns.offset(), parent_before + 3);
        assert_eq!(h.app().state.idle_horizontal_offset(), 0);
        assert_idle_invariants(&h, &value, v_before);
    }
}

#[test]
fn zero_and_modified_h_wheels_preserve_parent_fallback_behavior() {
    for (delta, mods, expected_parent_movement) in
        [(0, KeyModifiers::NONE, 0), (1, KeyModifiers::SHIFT, 3)]
    {
        let mut h = harness(IdleTextAreaApp::new(LONG_VALUE, Some(pan(4))));
        let value = h.app().value.clone();
        let parent_before = h.app().parent_columns.offset();
        let v_before = h.app().state.scroll().offset();

        let response = wheel_with_modifiers(&mut h, delta, mods);

        if delta != 0 {
            assert!(response.is_changed());
        }
        assert_eq!(h.app().field_initial_attempts.len(), 1);
        let field = h.app().field_initial_attempts[0];
        assert_eq!(field.owner, FIELD);
        assert_eq!(field.attempt_index, 0);
        assert_eq!(field.delta, delta.saturating_mul(3));
        assert_eq!(field.unscaled_delta, delta);
        assert_eq!(field.modifiers, mods);
        assert_eq!(h.app().route_callbacks, 1);
        assert!(h.app().route_callbacks_empty);
        assert_eq!(h.app().route_attempts.len(), 1);
        let parent = h.app().route_attempts[0];
        assert_eq!(parent.owner, PARENT);
        assert_eq!(parent.attempt_index, 1);
        assert_eq!(parent.event_id, field.event_id);
        assert_eq!(parent.delta, field.delta);
        assert_eq!(parent.modifiers, mods);
        assert_eq!(
            h.app().parent_columns.offset(),
            parent_before + expected_parent_movement
        );
        assert_eq!(h.app().state.idle_horizontal_offset(), 0);
        assert_idle_invariants(&h, &value, v_before);
    }
}

#[test]
fn active_edit_with_keep_blur_declines_wheel_to_parent_and_commits_retained_session() {
    let mut app = IdleTextAreaApp::new(LONG_VALUE, Some(pan(4)));
    app.esc_reverts = true;
    app.blur_policy = BlurPolicy::Keep;
    let mut h = harness(app);
    let original = h.app().value.clone();

    assert!(wheel(&mut h, 1).is_changed());
    assert_eq!(h.app().state.idle_horizontal_offset(), 4);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().state.phase(), EditPhase::Editing);
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    let _ = h.key(KeyCode::Char('X'));
    assert_eq!(h.app().value, original);

    h.blur();
    assert_eq!(h.app().state.phase(), EditPhase::Editing);
    assert_eq!(h.app().value, original);
    let parent_before = h.app().parent_columns.offset();
    let _ = wheel(&mut h, 1);

    assert_eq!(h.app().state.phase(), EditPhase::Editing);
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    assert_eq!(h.app().parent_columns.offset(), parent_before + 3);
    assert_eq!(h.app().field_initial_attempts.len(), 2);
    assert_eq!(h.app().field_initial_attempts[1].source_revision, 1);
    assert_eq!(h.app().field_initial_attempts[1].attempt_index, 0);
    assert_eq!(
        h.app().route_attempts.last().map(|attempt| attempt.owner),
        Some(PARENT)
    );

    assert!(h.tab_to(FIELD));
    let _ = h.key(KeyCode::Char('Y'));
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().value, format!("{original}XY"));
    assert_eq!(h.app().state.phase(), EditPhase::Idle);
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    assert_eq!(h.app().actions.last(), Some(&Some(TextAction::Committed)));
}

#[test]
fn type_to_edit_cancel_after_idle_pan_restores_value_and_clears_offset() {
    let mut app = IdleTextAreaApp::new(LONG_VALUE, Some(pan(4)));
    app.edit_entry = EditEntry::TypeToEdit;
    app.esc_reverts = true;
    let mut h = harness(app);
    let original = h.app().value.clone();

    assert!(wheel(&mut h, 1).is_changed());
    assert_eq!(h.app().state.idle_horizontal_offset(), 4);
    let _ = h.key(KeyCode::Char('X'));
    assert_eq!(h.app().state.phase(), EditPhase::Editing);
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    assert_eq!(h.app().value, original);
    let _ = h.key(KeyCode::Esc);

    assert_eq!(h.app().state.phase(), EditPhase::Idle);
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    assert_eq!(h.app().value, original);
    assert_eq!(h.app().actions.last(), Some(&Some(TextAction::Cancelled)));
}

#[test]
fn stale_source_revision_cannot_acknowledge_or_move_either_owner() {
    let replacement = "zero-row\none-row\nnew-source\nthree-row";
    let mut h = harness(IdleTextAreaApp::new(LONG_VALUE, Some(pan(4))));
    let parent_before = h.app().parent_columns.offset();
    h.app_mut().source_replacement = Some(replacement.to_owned());

    let _ = wheel(&mut h, 1);
    assert_eq!(h.app().source_revision, 2);
    assert_eq!(h.app().value, replacement);
    assert_eq!(h.app().field_initial_attempts.len(), 1);
    let initial = h.app().field_initial_attempts[0];
    assert_eq!(initial.owner, FIELD);
    assert_eq!(initial.attempt_index, 0);
    assert_eq!(initial.source_revision, 1);
    assert!(h.app().route_attempts.is_empty());
    assert_eq!(h.app().route_callbacks, 0);
    assert_eq!(
        h.app().parent_columns.offset(),
        parent_before,
        "a stale child source invalidates the route before parent fallback"
    );
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    assert_eq!(h.app().state.phase(), EditPhase::Idle);
    assert!(h.app().actions.iter().all(Option::is_none));
}

#[test]
fn unconfigured_text_area_leaves_the_h_route_to_its_parent() {
    let mut h = harness(IdleTextAreaApp::new(LONG_VALUE, None));
    let parent_before = h.app().parent_columns.offset();
    let _ = wheel(&mut h, 1);

    assert!(h.app().field_initial_attempts.is_empty());
    assert_eq!(h.app().parent_initial_attempts.len(), 1);
    let parent = h.app().parent_initial_attempts[0];
    assert_eq!(parent.owner, PARENT);
    assert_eq!(parent.part, PARENT_PART);
    assert_eq!(parent.axis, Axis::H);
    assert_eq!(parent.attempt_index, 0);
    assert_eq!(h.app().route_callbacks, 0);
    assert_eq!(h.app().parent_columns.offset(), parent_before + 3);
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    assert_idle_invariants(&h, LONG_VALUE, 1);
}

#[test]
fn disabled_text_area_does_not_enroll_as_an_idle_h_target() {
    let mut app = IdleTextAreaApp::new(LONG_VALUE, Some(pan(4)));
    app.disabled = true;
    let mut h = harness(app);
    let parent_before = h.app().parent_columns.offset();
    let _ = wheel(&mut h, 1);

    assert!(h.app().field_initial_attempts.is_empty());
    assert_eq!(h.app().parent_initial_attempts.len(), 1);
    assert_eq!(h.app().parent_initial_attempts[0].owner, PARENT);
    assert_eq!(h.app().route_callbacks, 0);
    assert_eq!(h.app().parent_columns.offset(), parent_before + 3);
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    assert_eq!(h.app().state.phase(), EditPhase::Idle);
    assert_eq!(h.app().value, LONG_VALUE);
}

#[test]
fn enabled_pan_clamps_and_consumes_at_both_edges_without_parent_fallback() {
    let mut h = harness(IdleTextAreaApp::new(LONG_VALUE, Some(pan(4))));
    let text = text_area(&h);
    let max_offset = h
        .app()
        .value
        .lines()
        .map(str::len)
        .max()
        .unwrap()
        .saturating_sub(usize::from(text.width));

    assert!(wheel(&mut h, i16::MAX).is_changed());
    assert_eq!(h.app().state.idle_horizontal_offset(), max_offset);
    let parent_offset = h.app().parent_columns.offset();
    let edge = wheel(&mut h, 1);
    assert!(edge.is_consumed());
    assert!(!edge.is_changed());
    assert_eq!(h.app().state.idle_horizontal_offset(), max_offset);
    assert_eq!(h.app().route_callbacks, 0);
    assert_eq!(h.app().parent_columns.offset(), parent_offset);

    assert!(wheel(&mut h, i16::MIN).is_changed());
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    let parent_offset = h.app().parent_columns.offset();
    let edge = wheel(&mut h, -1);
    assert!(edge.is_consumed());
    assert!(!edge.is_changed());
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    assert_eq!(h.app().parent_columns.offset(), parent_offset);
    assert_eq!(h.app().route_callbacks, 0);
    assert_eq!(h.app().field_initial_attempts.len(), 4);
}

#[test]
fn a_wide_grapheme_crossed_by_the_left_window_keeps_following_source_cells() {
    let value = concat!(
        "zero-row\n",
        "one-row\n",
        "界ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\n",
        "BabcdefghijKLMNOPQRSTUVWXYZ0123456789\n",
        "four-row\n",
        "five-row\n",
        "six-row\n",
        "seven-row",
    );
    let mut h = harness(IdleTextAreaApp::new(value, Some(pan(1))));
    let text = text_area(&h);

    assert!(wheel(&mut h, 1).is_changed());
    assert_eq!(h.app().state.idle_horizontal_offset(), 1);
    assert_eq!(h.cell(text.x, text.y).symbol(), "…");
    assert_eq!(h.cell(text.x + 1, text.y).symbol(), "A");
    assert_eq!(h.cell(text.x + 2, text.y).symbol(), "B");
    assert_eq!(h.app().route_callbacks, 0);
}

#[test]
fn a_zwj_grapheme_and_combining_mark_keep_source_cell_boundaries() {
    let value = concat!(
        "zero-row\n",
        "one-row\n",
        "👩‍💻éABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\n",
        "BabcdefghijKLMNOPQRSTUVWXYZ0123456789\n",
        "four-row\n",
        "five-row\n",
        "six-row\n",
        "seven-row",
    );
    let mut h = harness(IdleTextAreaApp::new(value, Some(pan(1))));
    let text = text_area(&h);

    assert!(wheel(&mut h, 1).is_changed());
    assert_eq!(h.app().state.idle_horizontal_offset(), 1);
    assert_eq!(
        h.cell(text.x, text.y).symbol(),
        "…",
        "offset one clips the first source cell of the two-cell ZWJ grapheme"
    );
    assert_eq!(h.cell(text.x + 1, text.y).symbol(), "é");
    assert_eq!(h.cell(text.x + 2, text.y).symbol(), "A");
    assert_eq!(h.app().route_callbacks, 0);
}

#[test]
fn pointer_entry_after_idle_pan_targets_the_painted_source_column() {
    let mut h = harness(IdleTextAreaApp::new(LONG_VALUE, Some(pan(4))));
    let text = text_area(&h);
    let original = h.app().value.clone();
    assert!(wheel(&mut h, 1).is_changed());
    assert_eq!(h.cell(text.x, text.y).symbol(), "3");

    let _ = h.click(text.x, text.y);
    assert_eq!(h.app().state.phase(), EditPhase::Editing);
    assert_eq!(h.app().value, original);
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    let _ = h.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
    h.app_mut().copy_next = true;
    let _ = h.tick();

    assert_eq!(h.app().copy_result, Some(true));
    assert_eq!(h.app().copied_text.as_deref(), Some("3"));
    assert_eq!(h.app().value, original);
}

#[test]
fn pointer_entry_uses_the_presented_offset_if_source_revision_changes_in_update() {
    let mut h = harness(IdleTextAreaApp::new(LONG_VALUE, Some(pan(4))));
    let text = text_area(&h);
    assert!(wheel(&mut h, 1).is_changed());
    assert_eq!(h.cell(text.x, text.y).symbol(), "3");

    let replacement = concat!(
        "zero-row\n",
        "one-row\n",
        "ABCDx0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ\n",
        "BabcdefghijKLMNOPQRSTUVWXYZ0123456789\n",
        "four-row\n",
        "five-row\n",
        "six-row\n",
        "seven-row",
    );
    h.app_mut().source_replacement = Some(replacement.to_owned());
    let _ = h.click(text.x, text.y);

    assert_eq!(h.app().source_revision, 2);
    assert_eq!(h.app().state.phase(), EditPhase::Editing);
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    assert_eq!(h.app().value, replacement);
    let _ = h.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
    h.app_mut().copy_next = true;
    let _ = h.tick();

    assert_eq!(h.app().copy_result, Some(true));
    assert_eq!(h.app().copied_text.as_deref(), Some("x"));
    assert_eq!(h.app().value, replacement);
}

#[test]
fn pointer_entry_after_committed_revision_reset_uses_painted_column_zero() {
    let mut h = harness(IdleTextAreaApp::new(LONG_VALUE, Some(pan(4))));
    let text = text_area(&h);
    assert!(wheel(&mut h, 1).is_changed());
    assert_eq!(h.cell(text.x, text.y).symbol(), "3");

    let replacement = concat!(
        "zero-row\n",
        "one-row\n",
        "Q0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ\n",
        "BabcdefghijKLMNOPQRSTUVWXYZ0123456789\n",
        "four-row\n",
        "five-row\n",
        "six-row\n",
        "seven-row",
    );
    h.app_mut().source_revision += 1;
    h.app_mut().value = replacement.to_owned();
    h.draw();

    assert_eq!(
        h.app().state.idle_horizontal_offset(),
        4,
        "committing the reset presentation does not reconcile stored state"
    );
    assert_eq!(h.cell(text.x, text.y).symbol(), "Q");
    h.app_mut().observe_layout_once = true;
    let _ = h.click(text.x, text.y);

    assert_eq!(
        h.app().observed_layout_offsets,
        vec![Some(0)],
        "the click update reads the committed painted column before reconciliation"
    );
    assert_eq!(h.app().state.phase(), EditPhase::Editing);
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
    let _ = h.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
    h.app_mut().copy_next = true;
    let _ = h.tick();

    assert_eq!(h.app().copy_result, Some(true));
    assert_eq!(h.app().copied_text.as_deref(), Some("Q"));
    assert_eq!(h.app().value, replacement);
}

#[test]
fn pointer_entry_after_committed_resize_clamp_uses_painted_column() {
    let mut h = harness(IdleTextAreaApp::new(LONG_VALUE, Some(pan(4))));
    let _ = h.resize(14, SCREEN.height);
    let narrow_text = text_area(&h);
    assert_eq!(
        narrow_text.width, 9,
        "the parent clips the logical text area"
    );

    assert!(wheel(&mut h, i16::MAX).is_changed());
    assert_eq!(h.app().state.idle_horizontal_offset(), 28);

    let _ = h.resize(SCREEN.width, SCREEN.height);
    let text = text_area(&h);
    assert_eq!(text.width, 26);
    assert_eq!(h.app().state.idle_horizontal_offset(), 28);
    assert_eq!(h.cell(text.x, text.y).symbol(), "A");

    h.app_mut().observe_layout_once = true;
    let _ = h.click(text.x, text.y);
    assert_eq!(
        h.app().observed_layout_offsets,
        vec![Some(11)],
        "the click maps from the committed resize-clamped source column"
    );
    let _ = h.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
    h.app_mut().copy_next = true;
    let _ = h.tick();

    assert_eq!(h.app().copy_result, Some(true));
    assert_eq!(h.app().copied_text.as_deref(), Some("A"));
}

#[test]
fn aborted_draw_keeps_the_previous_committed_source_column_authoritative() {
    let mut h = harness(IdleTextAreaApp::new(LONG_VALUE, Some(pan(4))));
    let text = text_area(&h);
    assert!(wheel(&mut h, 1).is_changed());
    assert_eq!(h.cell(text.x, text.y).symbol(), "3");

    let replacement = concat!(
        "zero-row\n",
        "one-row\n",
        "Q0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ\n",
        "BabcdefghijKLMNOPQRSTUVWXYZ0123456789\n",
        "four-row\n",
        "five-row\n",
        "six-row\n",
        "seven-row",
    );
    h.app_mut().source_revision += 1;
    h.app_mut().value = replacement.to_owned();
    let mut buffer = Buffer::empty(SCREEN);
    let uncommitted = h.runtime_mut().draw_buffer(SCREEN, &mut buffer);
    assert_eq!(
        buffer.cell((text.x, text.y)).unwrap().symbol(),
        "Q",
        "the aborted candidate paints the new source from column zero"
    );
    drop(uncommitted);

    assert_eq!(
        h.cell(text.x, text.y).symbol(),
        "3",
        "the old output remains"
    );
    assert_eq!(h.app().state.idle_horizontal_offset(), 4);
    h.app_mut().observe_layout_once = true;
    let next_focus = (h.focus() != Some(FIELD)).then_some(FIELD);
    h.runtime_mut().set_focus(next_focus);
    let _ = h.runtime_mut().settle();
    assert_eq!(h.app().observed_layout_offsets, vec![Some(4)]);

    h.draw();
    assert_eq!(h.cell(text.x, text.y).symbol(), "Q");
    let _ = h.click(text.x, text.y);
    let _ = h.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
    h.app_mut().copy_next = true;
    let _ = h.tick();

    assert_eq!(h.app().copy_result, Some(true));
    assert_eq!(h.app().copied_text.as_deref(), Some("Q"));
}

#[test]
fn long_idle_source_columns_remain_usize_and_source_revision_resets_pan() {
    let long_line = "x".repeat(70_000);
    let value = format!(
        "zero\none\n{long_line}\nB{}\nfour\nfive\nsix\nseven",
        "y".repeat(70_000)
    );
    let mut h = harness(IdleTextAreaApp::new(value, Some(pan(40_000))));

    assert!(wheel(&mut h, 2).is_changed());
    assert!(h.app().state.idle_horizontal_offset() > usize::from(u16::MAX));
    assert_eq!(h.app().field_initial_attempts[0].delta, 6);
    assert_eq!(h.app().field_initial_attempts[0].unscaled_delta, 2);

    h.app_mut().source_revision += 1;
    h.app_mut().value = LONG_VALUE.to_owned();
    let _ = h.tick();
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);

    h.app_mut().source_revision += 1;
    let _ = h.tick();
    assert!(wheel(&mut h, 1).is_changed());
    assert_ne!(h.app().state.idle_horizontal_offset(), 0);
    h.app_mut().state.zeroize();
    assert_eq!(h.app().state.idle_horizontal_offset(), 0);
}
