#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    reason = "runtime presentation assertions use explicit fixture events"
)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use ratatui_core::buffer::Buffer;
use termrock_core::{
    ActionKey, Chord, Id, Input, Intent, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Phase,
    Response,
};
use termrock_runtime::{
    Anchor, App, Cx, Dismiss, Focusability, FrameRead, LayerId, LayerSize, LayerSpec, Position,
    Rect, Runtime, StateFlags, Ui,
};
use termrock_theme::Theme;

const PAGE_A: Id = Id::root("ui.focus.presentation.page-a");
const PAGE_B: Id = Id::root("ui.focus.presentation.page-b");
const POPOVER: Id = Id::root("ui.focus.presentation.popover");
const MODAL: Id = Id::root("ui.focus.presentation.modal");
const TOOLTIP: Id = Id::root("ui.focus.presentation.tooltip");
const REFERENCE_ONLY: Id = Id::root("ui.focus.presentation.reference-only");
const REFERENCE_ACTION: ActionKey = ActionKey::custom("ui.focus.presentation.reference-only");
const AREA: Rect = Rect::new(0, 0, 40, 12);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Reading {
    context: &'static str,
    layer: LayerId,
    covered: bool,
    inert: bool,
    page_a_focused: bool,
    page_b_focused: bool,
    page_b_state_focused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EventMark {
    Key(Id, char),
    Binding(Id, ActionKey),
    Pointer(Id, Phase),
    FocusIn(Id),
}

#[derive(Clone, Debug)]
struct Probe {
    popover_open: bool,
    modal_open: bool,
    tooltip_open: bool,
    updates: usize,
    readings: Rc<RefCell<Vec<Reading>>>,
    events: Rc<RefCell<Vec<EventMark>>>,
    reference_layer_drawn: Rc<Cell<bool>>,
}

impl Default for Probe {
    fn default() -> Self {
        Self {
            popover_open: false,
            modal_open: false,
            tooltip_open: false,
            updates: 0,
            readings: Rc::new(RefCell::new(Vec::new())),
            events: Rc::new(RefCell::new(Vec::new())),
            reference_layer_drawn: Rc::new(Cell::new(false)),
        }
    }
}

impl App for Probe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates = self.updates.saturating_add(1);
        for owner in [PAGE_A, PAGE_B, POPOVER, MODAL, TOOLTIP] {
            for intent in cx.intents(owner) {
                match intent {
                    Intent::Key(key) => {
                        if let KeyCode::Char(character) = key.code {
                            self.events
                                .borrow_mut()
                                .push(EventMark::Key(owner, character));
                            match character {
                                'p' if owner == PAGE_A || owner == PAGE_B => {
                                    self.popover_open = true;
                                    cx.open_layer(
                                        POPOVER,
                                        LayerSpec::popover(
                                            POPOVER,
                                            Anchor::Point(Position::new(20, 4)),
                                        )
                                        .size(LayerSize::Fixed(8, 3))
                                        .dismiss(Dismiss::NONE),
                                    );
                                }
                                'm' if owner == PAGE_A || owner == PAGE_B => {
                                    self.modal_open = true;
                                    cx.open_layer(
                                        MODAL,
                                        LayerSpec::modal(MODAL).size(LayerSize::Fixed(8, 3)),
                                    );
                                }
                                't' if owner == PAGE_A || owner == PAGE_B => {
                                    self.tooltip_open = true;
                                    cx.open_layer(
                                        TOOLTIP,
                                        LayerSpec::tooltip(TOOLTIP, Position::new(20, 8))
                                            .size(LayerSize::Fixed(8, 2))
                                            .dismiss(Dismiss::NONE),
                                    );
                                }
                                'c' => {
                                    if self.tooltip_open {
                                        self.tooltip_open = false;
                                        cx.close_layer(TOOLTIP, None);
                                    }
                                    if self.modal_open {
                                        self.modal_open = false;
                                        cx.close_layer(MODAL, None);
                                    }
                                    if self.popover_open {
                                        self.popover_open = false;
                                        cx.close_layer(POPOVER, None);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Intent::Binding(action) => {
                        self.events
                            .borrow_mut()
                            .push(EventMark::Binding(owner, action));
                    }
                    Intent::Pointer { phase, .. } => {
                        self.events
                            .borrow_mut()
                            .push(EventMark::Pointer(owner, phase));
                    }
                    Intent::FocusIn { .. } => {
                        self.events.borrow_mut().push(EventMark::FocusIn(owner));
                    }
                    _ => {}
                }
            }
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.readings.borrow_mut().clear();
        self.reference_layer_drawn.set(false);

        ui.register_control(PAGE_A, Rect::new(1, 1, 8, 2), Focusability::Focusable);
        ui.register_control(PAGE_B, Rect::new(1, 4, 8, 2), Focusability::Focusable);
        // Declared component state must not be mistaken for committed focus.
        ui.declare_state(PAGE_B, StateFlags::FOCUSED);
        read(&self.readings, "page", ui);

        let paper = Theme::paper();
        ui.with_theme(&paper, |nested| read(&self.readings, "page-theme", nested));
        {
            let reborrowed = ui.reborrow();
            read(&self.readings, "page-reborrow", &reborrowed);
        }

        ui.reference(
            Some(termrock_runtime::ReferenceTarget::new(
                PAGE_A,
                termrock_runtime::ReferenceState::FOCUSED,
            )),
            |reference| {
                read(&self.readings, "reference-a", reference);
                {
                    let reborrowed = reference.reborrow();
                    read(&self.readings, "reference-a-reborrow", &reborrowed);
                }
                reference.with_theme(&paper, |nested| {
                    read(&self.readings, "reference-a-theme", nested);
                });

                reference.register_control(
                    REFERENCE_ONLY,
                    Rect::new(30, 1, 6, 1),
                    Focusability::Focusable,
                );
                reference.publish_dynamic_bindings(
                    REFERENCE_ONLY,
                    StateFlags::FOCUSED,
                    core::iter::once((REFERENCE_ACTION, Some(Chord::key(KeyCode::Char('z'))))),
                );
                reference.set_cursor(REFERENCE_ONLY, Position::new(30, 1));
                let _ = reference.layer(POPOVER, |_, _| {
                    self.reference_layer_drawn.set(true);
                });

                reference.reference(
                    Some(termrock_runtime::ReferenceTarget::new(
                        PAGE_B,
                        termrock_runtime::ReferenceState::FOCUSED,
                    )),
                    |nested| read(&self.readings, "reference-nested-b", nested),
                );
                reference.reference(None, |nested| {
                    read(&self.readings, "reference-targetless", nested);
                });
            },
        );
        ui.reference(
            Some(termrock_runtime::ReferenceTarget::new(
                PAGE_B,
                termrock_runtime::ReferenceState::FOCUSED,
            )),
            |reference| read(&self.readings, "reference-b", reference),
        );
        read(&self.readings, "page-after-reference", ui);

        let _ = ui.layer(POPOVER, |overlay, area| {
            overlay.register_control(POPOVER, area, Focusability::Focusable);
            read(&self.readings, "popover", overlay);
        });
        let _ = ui.layer(MODAL, |overlay, area| {
            overlay.register_control(MODAL, area, Focusability::Focusable);
            read(&self.readings, "modal", overlay);
        });
        let _ = ui.layer(TOOLTIP, |overlay, area| {
            overlay.register_control(TOOLTIP, area, Focusability::Focusable);
            read(&self.readings, "tooltip", overlay);
        });
    }
}

fn read(readings: &RefCell<Vec<Reading>>, context: &'static str, ui: &Ui<'_>) {
    readings.borrow_mut().push(Reading {
        context,
        layer: ui.layer_id(),
        covered: ui.is_covered_by_native_layer(),
        inert: ui.is_inert(),
        page_a_focused: ui.owner_has_focus_presentation(PAGE_A),
        page_b_focused: ui.owner_has_focus_presentation(PAGE_B),
        page_b_state_focused: ui.state(PAGE_B).contains(StateFlags::FOCUSED),
    });
}

fn reading(probe: &Probe, context: &str) -> Reading {
    probe
        .readings
        .borrow()
        .iter()
        .copied()
        .find(|reading| reading.context == context)
        .unwrap_or_else(|| panic!("missing context reading {context}"))
}

fn present(runtime: &mut Runtime<Probe>, buffer: &mut Buffer) {
    for _ in 0..16 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        if runtime.needs_present() || runtime.needs_settle() {
            runtime.draw_buffer(AREA, buffer).commit_presented();
        }
        if !runtime.needs_settle() && !runtime.needs_present() {
            return;
        }
    }
    panic!(
        "runtime presentation did not settle: {:?}",
        runtime.diagnostics()
    );
}

fn runtime() -> (Runtime<Probe>, Buffer) {
    let mut runtime = Runtime::new(Probe::default(), Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(AREA);
    present(&mut runtime, &mut buffer);
    runtime.set_focus(Some(PAGE_A));
    present(&mut runtime, &mut buffer);
    assert_eq!(runtime.focus(), Some(PAGE_A));
    (runtime, buffer)
}

fn deliver(runtime: &mut Runtime<Probe>, buffer: &mut Buffer, input: Input) {
    let _ = termrock_test_support::deliver_buffer(runtime, buffer, input);
}

fn key(character: char) -> Input {
    Input::Key(Key {
        code: KeyCode::Char(character),
        mods: KeyModifiers::NONE,
    })
}

fn mouse(kind: MouseKind, pos: Position) -> Input {
    Input::Mouse(Mouse {
        kind,
        pos,
        mods: KeyModifiers::NONE,
    })
}

#[test]
fn live_focus_uses_committed_snapshot_and_counts_pointer_focus_without_focus_visible() {
    let (mut runtime, mut buffer) = runtime();
    let initial = reading(runtime.app(), "page");
    assert!(!initial.covered);
    assert!(!initial.inert);
    assert!(initial.page_a_focused);
    assert!(!initial.page_b_focused);
    assert!(
        initial.page_b_state_focused,
        "B deliberately declares FOCUSED"
    );
    let detached = reading(runtime.app(), "reference-a");
    assert!(!detached.covered && detached.inert);
    assert!(detached.page_a_focused);
    assert!(!detached.page_b_focused);

    let updates = runtime.app().updates;
    let focus = runtime.focus();
    let regions = runtime.region_count();
    let cursor = runtime.cursor_position();
    let deadline = runtime.next_deadline();
    let activation = runtime.activation_feedback();
    let diagnostics = runtime.diagnostics().to_vec();
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert_eq!(
        runtime.app().updates,
        updates,
        "queries do not update the app"
    );
    assert_eq!(runtime.focus(), focus);
    assert_eq!(runtime.region_count(), regions);
    assert_eq!(runtime.cursor_position(), cursor);
    assert_eq!(runtime.next_deadline(), deadline);
    assert_eq!(runtime.activation_feedback(), activation);
    assert_eq!(runtime.diagnostics(), diagnostics);

    deliver(
        &mut runtime,
        &mut buffer,
        mouse(MouseKind::Down, Position::new(2, 4)),
    );
    present(&mut runtime, &mut buffer);
    assert_eq!(runtime.focus(), Some(PAGE_B));
    assert!(
        !runtime.focus_visible(),
        "pointer focus is not keyboard-visible"
    );
    let pointer_focused = reading(runtime.app(), "page");
    assert!(pointer_focused.page_b_focused);
    assert!(pointer_focused.page_b_state_focused);
    assert!(!pointer_focused.page_a_focused);
    assert!(
        runtime
            .app()
            .events
            .borrow()
            .contains(&EventMark::FocusIn(PAGE_B))
    );
}

#[test]
fn first_popover_frame_reports_native_coverage_and_separates_key_from_pointer_delivery() {
    let (mut runtime, mut buffer) = runtime();
    deliver(&mut runtime, &mut buffer, key('p'));
    assert!(runtime.is_open(POPOVER));
    let updates_before_draw = runtime.app().updates;
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert_eq!(
        runtime.app().updates,
        updates_before_draw,
        "the first frame observes the changed stack without another update or tick"
    );
    assert_ne!(runtime.top_layer(), LayerId::PAGE);

    let page = reading(runtime.app(), "page");
    let popover = reading(runtime.app(), "popover");
    assert!(page.covered);
    assert!(!page.inert, "the native Popover is nonmodal");
    assert!(!popover.covered);
    assert_eq!(runtime.focus(), Some(PAGE_A));

    deliver(&mut runtime, &mut buffer, key('x'));
    assert!(
        runtime
            .app()
            .events
            .borrow()
            .contains(&EventMark::Key(PAGE_A, 'x'))
    );

    deliver(
        &mut runtime,
        &mut buffer,
        mouse(MouseKind::Down, Position::new(2, 1)),
    );
    deliver(
        &mut runtime,
        &mut buffer,
        mouse(MouseKind::Up, Position::new(2, 1)),
    );
    assert!(
        !runtime
            .app()
            .events
            .borrow()
            .iter()
            .any(|event| { matches!(event, EventMark::Pointer(PAGE_A, _)) })
    );

    deliver(&mut runtime, &mut buffer, key('c'));
    let updates_before_close_draw = runtime.app().updates;
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert_eq!(runtime.app().updates, updates_before_close_draw);
    assert!(!reading(runtime.app(), "page").covered);
}

#[test]
fn modal_coverage_is_separate_from_the_retained_underlying_focus_fact() {
    let (mut runtime, mut buffer) = runtime();
    deliver(&mut runtime, &mut buffer, key('p'));
    present(&mut runtime, &mut buffer);
    deliver(&mut runtime, &mut buffer, key('m'));
    assert!(runtime.is_open(MODAL));
    let updates_before_modal_draw = runtime.app().updates;
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert_eq!(runtime.app().updates, updates_before_modal_draw);
    assert!(reading(runtime.app(), "page").covered);
    assert!(reading(runtime.app(), "page").inert);
    assert!(reading(runtime.app(), "popover").covered);
    assert!(!reading(runtime.app(), "modal").covered);
    assert!(reading(runtime.app(), "page").page_a_focused);
    assert!(reading(runtime.app(), "modal").page_a_focused);

    deliver(
        &mut runtime,
        &mut buffer,
        mouse(MouseKind::Down, Position::new(2, 1)),
    );
    deliver(
        &mut runtime,
        &mut buffer,
        mouse(MouseKind::Up, Position::new(2, 1)),
    );
    assert!(
        !runtime
            .app()
            .events
            .borrow()
            .iter()
            .any(|event| { matches!(event, EventMark::Pointer(PAGE_A, _)) })
    );

    deliver(&mut runtime, &mut buffer, key('c'));
    let updates_before_close_draw = runtime.app().updates;
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert_eq!(runtime.app().updates, updates_before_close_draw);
    assert_eq!(runtime.top_layer(), LayerId::PAGE);
    assert!(!reading(runtime.app(), "page").covered);
    present(&mut runtime, &mut buffer);
    assert_eq!(
        runtime.focus(),
        Some(PAGE_A),
        "modal close restores page focus"
    );
    assert!(reading(runtime.app(), "page").page_a_focused);
}

#[test]
fn tooltip_reports_native_top_without_claiming_host_overlay_equivalence() {
    let (mut runtime, mut buffer) = runtime();
    deliver(&mut runtime, &mut buffer, key('t'));
    assert!(runtime.is_open(TOOLTIP));
    present(&mut runtime, &mut buffer);
    assert!(reading(runtime.app(), "page").covered);
    assert!(!reading(runtime.app(), "page").inert);
    assert!(!reading(runtime.app(), "tooltip").covered);
}

#[test]
fn references_are_detached_synthetic_presentation_and_suppress_runtime_registration() {
    let (mut runtime, mut buffer) = runtime();
    deliver(&mut runtime, &mut buffer, key('p'));
    present(&mut runtime, &mut buffer);
    let before_focus = runtime.focus();
    let before_regions = runtime.region_count();
    let before_deadline = runtime.next_deadline();
    let before_feedback = runtime.activation_feedback();
    let before_diagnostics = runtime.diagnostics().to_vec();

    let reference_a = reading(runtime.app(), "reference-a");
    assert!(!reference_a.covered, "a reference is detached presentation");
    assert!(reference_a.inert, "detached presentation remains inert");
    assert!(reference_a.page_a_focused);
    assert!(!reference_a.page_b_focused);
    for context in ["reference-a-reborrow", "reference-a-theme"] {
        let nested = reading(runtime.app(), context);
        assert!(!nested.covered && nested.inert);
        assert!(nested.page_a_focused);
        assert!(!nested.page_b_focused);
    }

    let nested_b = reading(runtime.app(), "reference-nested-b");
    assert!(!nested_b.covered && nested_b.inert);
    assert!(!nested_b.page_a_focused);
    assert!(nested_b.page_b_focused);
    let targetless = reading(runtime.app(), "reference-targetless");
    assert!(!targetless.covered && targetless.inert);
    assert!(!targetless.page_a_focused && !targetless.page_b_focused);
    let other_target = reading(runtime.app(), "reference-b");
    assert!(!other_target.page_a_focused && other_target.page_b_focused);

    let after_reference = reading(runtime.app(), "page-after-reference");
    assert!(after_reference.covered);
    assert!(!after_reference.inert);
    assert_eq!(after_reference.page_a_focused, before_focus == Some(PAGE_A));
    assert!(runtime.area_of(REFERENCE_ONLY).is_none());
    assert!(runtime.ring().entry(REFERENCE_ONLY).is_none());
    assert_eq!(runtime.cursor_position(), None);
    assert!(!runtime.app().reference_layer_drawn.get());
    assert!(runtime.is_open(POPOVER));
    assert_eq!(runtime.region_count(), before_regions);
    assert_eq!(runtime.focus(), before_focus);
    assert_eq!(runtime.next_deadline(), before_deadline);
    assert_eq!(runtime.activation_feedback(), before_feedback);
    assert_eq!(runtime.diagnostics(), before_diagnostics);

    deliver(&mut runtime, &mut buffer, key('z'));
    assert!(
        !runtime
            .app()
            .events
            .borrow()
            .contains(&EventMark::Binding(REFERENCE_ONLY, REFERENCE_ACTION))
    );
    assert!(
        runtime
            .app()
            .events
            .borrow()
            .contains(&EventMark::Key(PAGE_A, 'z'))
    );
}

#[test]
fn snapshots_keep_their_own_top_and_focus_for_later_projection() {
    let (mut runtime, mut buffer) = runtime();
    let page_snapshot = runtime.render_snapshot().expect("settled page snapshot");

    deliver(&mut runtime, &mut buffer, key('p'));
    present(&mut runtime, &mut buffer);
    runtime.set_focus(Some(PAGE_B));
    present(&mut runtime, &mut buffer);
    let popover_snapshot = runtime.render_snapshot().expect("settled popover snapshot");
    assert_eq!(runtime.focus(), Some(PAGE_B));

    deliver(&mut runtime, &mut buffer, key('c'));
    present(&mut runtime, &mut buffer);
    runtime.set_focus(Some(PAGE_A));
    present(&mut runtime, &mut buffer);
    assert_eq!(runtime.top_layer(), LayerId::PAGE);

    let updates = runtime.app().updates;
    let live_probe = runtime.app().clone();
    runtime
        .draw_projection(AREA, &mut buffer, &page_snapshot, |ui, _| {
            live_probe.draw(ui)
        })
        .commit_inspected();
    let old_page = reading(runtime.app(), "page");
    assert!(!old_page.covered);
    assert!(old_page.page_a_focused);
    assert!(!old_page.page_b_focused);

    let live_probe = runtime.app().clone();
    runtime
        .draw_projection(AREA, &mut buffer, &popover_snapshot, |ui, _| {
            live_probe.draw(ui);
        })
        .commit_inspected();
    let old_popover_page = reading(runtime.app(), "page");
    assert!(old_popover_page.covered);
    assert!(!old_popover_page.page_a_focused);
    assert!(old_popover_page.page_b_focused);
    assert!(!reading(runtime.app(), "popover").covered);
    assert_eq!(runtime.app().updates, updates, "projection is read-only");
}
