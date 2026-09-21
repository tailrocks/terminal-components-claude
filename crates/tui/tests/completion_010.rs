//! TASK-010: runtime focus capture and publication behavior.
//!
//! Direct production trajectories for published eligibility, aborted
//! publication, the disabled barrier, gesture phases, focus publication,
//! reference inertness, typing ownership, hover/boundary behavior, the
//! physical pass boundary (BF14) and effective-chord identity (BF13).
#![allow(clippy::unwrap_used, reason = "test lifecycle assertions")]

use junie_tui::*;
use junie_tui_testing::Harness;
use ratatui_core::buffer::Buffer;

const AREA: Rect = Rect::new(0, 0, 30, 8);

fn present<A: App>(rt: &mut Runtime<A>) {
    for _ in 0..8 {
        rt.draw_buffer(AREA, &mut Buffer::empty(AREA))
            .commit_presented();
        if !rt.needs_settle() && !rt.needs_present() {
            return;
        }
        let _ = rt.settle();
    }
    assert!(!rt.needs_settle() && !rt.needs_present(), "nonconvergent");
}

fn mouse_at<A: App>(rt: &mut Runtime<A>, kind: MouseKind, x: u16, y: u16) {
    let _ = rt
        .handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(rt);
}

fn tick<A: App>(rt: &mut Runtime<A>) {
    present(rt);
    let _ = rt.handle(Input::Tick).unwrap();
    present(rt);
}

// ---------------------------------------------------------------------------
// W-010-01 / W-010-02: published and aborted eligibility.
// ---------------------------------------------------------------------------

const E_A: Id = Id::root("c010.e.a");
const E_B: Id = Id::root("c010.e.b");
const E_POP: Id = Id::root("c010.e.pop");
const THUMB: PartRef = PartRef::of(Part::THUMB);

#[derive(Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent regression fixture axes"
)]
struct Eligible {
    moved: bool,
    disabled: bool,
    capture: bool,
    remove_part: bool,
    decor_part: bool,
    empty_part: bool,
    claim: Option<(Id, PartRef)>,
    phases: Vec<(Id, Phase)>,
}

impl App for Eligible {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut press = false;
        for intent in cx.intents(E_A) {
            if let Intent::Pointer { phase, .. } = intent {
                self.phases.push((E_A, phase));
                press |= phase == Phase::Press;
            }
        }
        for id in [E_B, E_POP] {
            for intent in cx.intents(id) {
                if let Intent::Pointer { phase, .. } = intent {
                    self.phases.push((id, phase));
                }
            }
        }
        if press && self.capture {
            assert!(cx.capture(E_A, THUMB));
        }
        if let Some((owner, part)) = self.claim.take() {
            let _ = cx.capture(owner, part);
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let a = Rect::new(if self.moved { 10 } else { 0 }, 0, 8, 2);
        let b = Rect::new(if self.moved { 0 } else { 10 }, 0, 8, 2);
        ui.register_control(
            E_A,
            a,
            if self.disabled {
                Focusability::Disabled
            } else {
                Focusability::Focusable
            },
        );
        if !self.remove_part {
            let area = if self.empty_part {
                Rect::ZERO
            } else {
                Rect::new(a.x, 0, 4, 2)
            };
            if self.decor_part {
                ui.register_decor(E_A, THUMB, area);
            } else {
                ui.register_part(E_A, THUMB, area);
            }
        }
        ui.register_control(E_B, b, Focusability::Focusable);
        ui.layer(E_POP, |ui, area| {
            ui.register_control(E_POP, area, Focusability::Focusable);
        });
    }
}

fn eligible(capture: bool) -> Runtime<Eligible> {
    let mut rt = Runtime::new(
        Eligible {
            capture,
            ..Eligible::default()
        },
        Theme::junie(),
    );
    let _ = rt.initialize();
    present(&mut rt);
    rt
}

/// W-010-01: re-enable never resurrects the old gesture, and a new
/// independent Press works afterwards.
#[test]
fn c010_fresh_press_after_reenable_activates() {
    for capture in [false, true] {
        let mut rt = eligible(capture);
        mouse_at(&mut rt, MouseKind::Down, 1, 1);
        rt.app_mut().disabled = true;
        present(&mut rt);
        assert_eq!(rt.capture_owner(), None);
        rt.app_mut().disabled = false;
        present(&mut rt);
        let n = rt.app().phases.len();
        // No resurrected Release/Click from the cancelled gesture.
        mouse_at(&mut rt, MouseKind::Up, 1, 1);
        assert!(
            !rt.app()
                .phases
                .get(n..)
                .unwrap()
                .iter()
                .any(|(_, p)| matches!(p, Phase::Release | Phase::Click | Phase::DragEnd)),
            "capture={capture}"
        );
        // A new independent gesture works. A captured release also
        // carries the terminal DragEnd phase.
        let m = rt.app().phases.len();
        mouse_at(&mut rt, MouseKind::Down, 1, 1);
        mouse_at(&mut rt, MouseKind::Up, 1, 1);
        let tail: Vec<Phase> = rt
            .app()
            .phases
            .get(m..)
            .unwrap()
            .iter()
            .map(|(_, p)| *p)
            .collect();
        let expected = if capture {
            vec![Phase::Press, Phase::Release, Phase::DragEnd, Phase::Click]
        } else {
            vec![Phase::Press, Phase::Release, Phase::Click]
        };
        assert_eq!(tail, expected, "capture={capture}");
    }
}

/// W-010-02: an aborted (dropped) disabled frame changes neither routing
/// nor capture; only the later successful publication cancels.
#[test]
fn c010_aborted_disable_preserves_routing_until_successful_publish() {
    let mut rt = eligible(true);
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    rt.app_mut().disabled = true;
    drop(rt.draw_buffer(AREA, &mut Buffer::empty(AREA)));
    // Aborted publication: the held capture survives the dropped frame.
    assert_eq!(rt.capture_owner(), Some(E_A));
    rt.app_mut().disabled = false;
    present(&mut rt);
    assert_eq!(rt.capture_owner(), Some(E_A));
    let n = rt.app().phases.len();
    mouse_at(&mut rt, MouseKind::Drag, 2, 1);
    assert!(
        rt.app()
            .phases
            .get(n..)
            .unwrap()
            .contains(&(E_A, Phase::Drag)),
        "aborted frame must not cancel the held gesture"
    );
    // Release outside the captured area: Release without Click.
    let m = rt.app().phases.len();
    mouse_at(&mut rt, MouseKind::Up, 25, 6);
    let tail: Vec<Phase> = rt
        .app()
        .phases
        .get(m..)
        .unwrap()
        .iter()
        .map(|(_, p)| *p)
        .collect();
    assert_eq!(tail, vec![Phase::Release, Phase::DragEnd]);
    // A fresh press still routes while the disable was never published...
    // until the successful publication cancels eligibility.
    rt.app_mut().disabled = true;
    present(&mut rt);
    assert_eq!(rt.capture_owner(), None);
    let k = rt.app().phases.len();
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    mouse_at(&mut rt, MouseKind::Up, 1, 1);
    assert!(
        rt.app().phases.get(k..).unwrap().is_empty(),
        "published disabled target must absorb the press"
    );
    // ...and recovery works after re-enable (captured release carries
    // the terminal DragEnd).
    rt.app_mut().disabled = false;
    present(&mut rt);
    let j = rt.app().phases.len();
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    mouse_at(&mut rt, MouseKind::Up, 1, 1);
    let tail: Vec<Phase> = rt
        .app()
        .phases
        .get(j..)
        .unwrap()
        .iter()
        .map(|(_, p)| *p)
        .collect();
    assert_eq!(
        tail,
        vec![Phase::Press, Phase::Release, Phase::DragEnd, Phase::Click]
    );
}

/// W-010-01: publishing an absent target cancels the held press, and a
/// restored target accepts a fresh gesture.
#[test]
fn c010_absent_target_publication_cancels_and_fresh_press_recovers() {
    let mut rt = eligible(false);
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    rt.app_mut().remove_part = true;
    present(&mut rt);
    let n = rt.app().phases.len();
    mouse_at(&mut rt, MouseKind::Drag, 2, 1);
    mouse_at(&mut rt, MouseKind::Up, 1, 1);
    assert!(
        rt.app().phases.get(n..).unwrap().is_empty(),
        "absent publication must cancel the held press"
    );
    rt.app_mut().remove_part = false;
    present(&mut rt);
    let m = rt.app().phases.len();
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    mouse_at(&mut rt, MouseKind::Up, 1, 1);
    let tail: Vec<Phase> = rt
        .app()
        .phases
        .get(m..)
        .unwrap()
        .iter()
        .map(|(_, p)| *p)
        .collect();
    assert_eq!(tail, vec![Phase::Press, Phase::Release, Phase::Click]);
}

// ---------------------------------------------------------------------------
// W-010-03: the disabled barrier over a truly overlapping enabled target.
// ---------------------------------------------------------------------------

const O_A: Id = Id::root("c010.o.a");
const O_B: Id = Id::root("c010.o.b");
const O_POP: Id = Id::root("c010.o.pop");
const OVERLAP: Rect = Rect::new(0, 0, 8, 2);

#[derive(Default)]
struct Barrier {
    cross: bool,
    b_disabled: bool,
    open: bool,
    phases: Vec<(Id, Phase)>,
    wheels: Vec<Id>,
}

impl App for Barrier {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for id in [O_A, O_B, O_POP] {
            for intent in cx.intents(id) {
                match intent {
                    Intent::Pointer { phase, .. } => self.phases.push((id, phase)),
                    Intent::Wheel { .. } => self.wheels.push(id),
                    _ => {}
                }
            }
        }
        if self.open {
            self.open = false;
            cx.open_layer(
                O_POP,
                LayerSpec::popover(O_POP, Anchor::Screen(ScreenAlign::Center)),
            );
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(O_A, OVERLAP, Focusability::Focusable);
        let flag = if self.b_disabled {
            Focusability::Disabled
        } else {
            Focusability::Focusable
        };
        if !self.cross {
            ui.register_control(O_B, OVERLAP, flag);
            ui.register_scroll(O_B, OVERLAP, Axes::V, Headroom::default());
        }
        ui.layer(O_POP, |ui, _| {
            if self.cross {
                ui.register_control(O_B, OVERLAP, flag);
                ui.register_scroll(O_B, OVERLAP, Axes::V, Headroom::default());
            } else {
                ui.register_control(O_POP, Rect::new(20, 5, 4, 1), Focusability::Focusable);
            }
        });
    }
}

fn barrier(cross: bool, b_disabled: bool) -> Runtime<Barrier> {
    let mut rt = Runtime::new(
        Barrier {
            cross,
            b_disabled,
            ..Barrier::default()
        },
        Theme::junie(),
    );
    let _ = rt.initialize();
    present(&mut rt);
    rt
}

/// W-010-03: a later disabled top over an enabled lower on the same layer
/// absorbs primary/secondary activation; neither owner leaks. The enabled
/// control proves the overlap is real by winning the click.
#[test]
fn c010_same_layer_disabled_overlap_absorbs_all_activation() {
    let mut rt = barrier(false, true);
    for kind in [MouseKind::Down, MouseKind::Up, MouseKind::Secondary] {
        mouse_at(&mut rt, kind, 1, 1);
    }
    assert!(
        rt.app().phases.is_empty(),
        "disabled barrier must absorb without lower fallthrough: {:?}",
        rt.app().phases
    );
    // Positive control: the enabled top wins the same click, proving a true
    // overlap rather than a misplaced fixture.
    rt.app_mut().b_disabled = false;
    present(&mut rt);
    let n = rt.app().phases.len();
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    mouse_at(&mut rt, MouseKind::Up, 1, 1);
    assert_eq!(
        rt.app().phases.get(n..).unwrap(),
        &[
            (O_B, Phase::Press),
            (O_B, Phase::Release),
            (O_B, Phase::Click)
        ]
    );
}

/// W-010-03: the same barrier across layers — a disabled popover control
/// over an enabled page control. Neither owner leaks activation.
#[test]
fn c010_cross_layer_disabled_overlap_absorbs_all_activation() {
    let mut rt = barrier(true, true);
    rt.app_mut().open = true;
    tick(&mut rt);
    for kind in [MouseKind::Down, MouseKind::Up, MouseKind::Secondary] {
        mouse_at(&mut rt, kind, 1, 1);
    }
    assert!(
        rt.app().phases.is_empty(),
        "cross-layer disabled barrier must absorb: {:?}",
        rt.app().phases
    );
    rt.app_mut().b_disabled = false;
    present(&mut rt);
    let n = rt.app().phases.len();
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    mouse_at(&mut rt, MouseKind::Up, 1, 1);
    assert_eq!(
        rt.app().phases.get(n..).unwrap(),
        &[
            (O_B, Phase::Press),
            (O_B, Phase::Release),
            (O_B, Phase::Click)
        ]
    );
}

/// W-010-03: wheel stays routable to a disabled scroll owner while the
/// press barrier holds.
#[test]
fn c010_wheel_over_disabled_scroll_region_still_routes() {
    let mut rt = barrier(false, true);
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    assert!(rt.app().phases.is_empty());
    mouse_at(&mut rt, MouseKind::Wheel(Axis::V, 1), 1, 1);
    assert_eq!(rt.app().wheels.as_slice(), &[O_B]);
}

// ---------------------------------------------------------------------------
// W-010-04: gesture phases.
// ---------------------------------------------------------------------------

const G_ID: Id = Id::root("c010.g");

#[derive(Default)]
struct Gestures {
    disabled: bool,
    phases: Vec<Phase>,
}

impl App for Gestures {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for intent in cx.intents(G_ID) {
            if let Intent::Pointer { phase, .. } = intent {
                self.phases.push(phase);
            }
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(
            G_ID,
            Rect::new(0, 0, 8, 2),
            if self.disabled {
                Focusability::Disabled
            } else {
                Focusability::Focusable
            },
        );
    }
}

fn gestures() -> Runtime<Gestures> {
    let mut rt = Runtime::new(Gestures::default(), Theme::junie());
    let _ = rt.initialize();
    present(&mut rt);
    rt
}

/// W-010-04: `Click`, `DoubleClick` and `Drag` arrive as exact ordered phase
/// sequences; a Press never becomes a completed activation by itself.
#[test]
fn c010_click_doubleclick_drag_sequences_are_exact() {
    let mut rt = gestures();
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    mouse_at(&mut rt, MouseKind::Up, 1, 1);
    assert_eq!(
        rt.app().phases.as_slice(),
        &[Phase::Press, Phase::Release, Phase::Click]
    );
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    mouse_at(&mut rt, MouseKind::Up, 1, 1);
    assert_eq!(
        rt.app().phases.as_slice(),
        &[
            Phase::Press,
            Phase::Release,
            Phase::Click,
            Phase::Press,
            Phase::Release,
            Phase::DoubleClick
        ]
    );
    let n = rt.app().phases.len();
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    mouse_at(&mut rt, MouseKind::Drag, 3, 1);
    mouse_at(&mut rt, MouseKind::Drag, 4, 1);
    mouse_at(&mut rt, MouseKind::Up, 4, 1);
    assert_eq!(
        rt.app().phases.get(n..).unwrap(),
        &[
            Phase::Press,
            Phase::DragStart,
            Phase::Drag,
            Phase::Drag,
            Phase::Release,
            Phase::DragEnd
        ]
    );
}

/// W-010-04: `Secondary` never produces `Click`/`Release`; `SecondaryUp` is silent.
#[test]
fn c010_secondary_never_produces_click_or_release() {
    let mut rt = gestures();
    mouse_at(&mut rt, MouseKind::Secondary, 1, 1);
    mouse_at(&mut rt, MouseKind::SecondaryUp, 1, 1);
    assert_eq!(rt.app().phases.as_slice(), &[Phase::Secondary]);
}

/// W-010-04: disabling mid-drag cancels the gesture; no tail leaks and a
/// fresh gesture works after re-enable.
#[test]
fn c010_disable_mid_drag_cancels_without_tail() {
    let mut rt = gestures();
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    mouse_at(&mut rt, MouseKind::Drag, 3, 1);
    rt.app_mut().disabled = true;
    present(&mut rt);
    let n = rt.app().phases.len();
    mouse_at(&mut rt, MouseKind::Drag, 4, 1);
    mouse_at(&mut rt, MouseKind::Up, 4, 1);
    assert!(
        rt.app().phases.get(n..).unwrap().is_empty(),
        "disabled mid-drag must cancel without tail: {:?}",
        rt.app().phases.get(n..).unwrap()
    );
    rt.app_mut().disabled = false;
    present(&mut rt);
    let m = rt.app().phases.len();
    mouse_at(&mut rt, MouseKind::Down, 1, 1);
    mouse_at(&mut rt, MouseKind::Up, 1, 1);
    assert_eq!(
        rt.app().phases.get(m..).unwrap(),
        &[Phase::Press, Phase::Release, Phase::Click]
    );
}

// ---------------------------------------------------------------------------
// W-010-05: focus publication and restoration.
// ---------------------------------------------------------------------------

const F_A: Id = Id::root("c010.f.a");
const F_B: Id = Id::root("c010.f.b");
const F_MODAL: Id = Id::root("c010.f.modal");
const F_INNER: Id = Id::root("c010.f.inner");

#[derive(Default)]
struct FocusPub {
    show_a: bool,
    open_modal: bool,
    close_modal: bool,
    /// (owner, arrived).
    focus_events: Vec<(Id, bool)>,
}

impl FocusPub {
    fn with_a() -> Self {
        FocusPub {
            show_a: true,
            ..FocusPub::default()
        }
    }
}

impl App for FocusPub {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for id in [F_A, F_B, F_INNER, F_MODAL] {
            for intent in cx.intents(id) {
                match intent {
                    Intent::FocusIn { .. } => self.focus_events.push((id, true)),
                    Intent::FocusOut { .. } => self.focus_events.push((id, false)),
                    _ => {}
                }
            }
        }
        if self.open_modal {
            self.open_modal = false;
            cx.open_layer(F_MODAL, LayerSpec::modal(F_MODAL).initial_focus(F_INNER));
        }
        if self.close_modal {
            self.close_modal = false;
            cx.close_layer(F_MODAL, None);
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.show_a {
            ui.register_control(F_A, Rect::new(0, 0, 8, 1), Focusability::Focusable);
        }
        ui.register_control(F_B, Rect::new(0, 2, 8, 1), Focusability::Focusable);
        ui.layer(F_MODAL, |ui, area| {
            ui.register_control(F_INNER, area, Focusability::Focusable);
        });
    }
}

/// W-010-05: closing a nested modal restores the live opener exactly once —
/// one `FocusOut` from the modal control, one `FocusIn` to the retained opener.
#[test]
fn c010_nested_modal_close_restores_live_opener_exactly_once() {
    let mut h = Harness::new(FocusPub::with_a(), Theme::junie(), 30, 8);
    let _ = h.click_id(F_A);
    assert_eq!(h.focus(), Some(F_A));
    h.app_mut().open_modal = true;
    let _ = h.tick();
    assert_eq!(h.focus(), Some(F_INNER));
    let n = h.app().focus_events.len();
    h.app_mut().close_modal = true;
    let _ = h.tick();
    assert_eq!(h.focus(), Some(F_A));
    let tail = h.app().focus_events.get(n..).unwrap();
    assert_eq!(
        tail.iter().filter(|(id, inn)| *id == F_A && *inn).count(),
        1,
        "retained opener FocusIn must arrive once: {tail:?}"
    );
    assert_eq!(
        tail.iter()
            .filter(|(id, inn)| *id == F_INNER && !*inn)
            .count(),
        1,
        "modal FocusOut must arrive once: {tail:?}"
    );
    assert!(
        !tail.iter().any(|(id, inn)| *id == F_A && !*inn),
        "no phantom FocusOut from the restored opener: {tail:?}"
    );
}

/// W-010-05: an opener removed before publication is never restored —
/// no phantom `FocusIn` — while the survivor keeps working.
#[test]
fn c010_removed_opener_restores_survivor_without_phantom_focus() {
    let mut h = Harness::new(FocusPub::with_a(), Theme::junie(), 30, 8);
    let _ = h.click_id(F_A);
    h.app_mut().open_modal = true;
    let _ = h.tick();
    assert_eq!(h.focus(), Some(F_INNER));
    let n = h.app().focus_events.len();
    h.app_mut().show_a = false;
    h.app_mut().close_modal = true;
    let _ = h.tick();
    assert_ne!(h.focus(), Some(F_A));
    let tail = h.app().focus_events.get(n..).unwrap();
    assert!(
        !tail.iter().any(|(id, inn)| *id == F_A && *inn),
        "removed opener must never receive FocusIn: {tail:?}"
    );
    // The survivor is reachable and focusable after the failed restore.
    assert!(h.tab_to(F_B));
    assert_eq!(h.focus(), Some(F_B));
}

#[derive(Default)]
struct Stops {
    stops: u8,
}

impl App for Stops {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for _ in cx.intents(F_A) {}
        for _ in cx.intents(F_B) {}
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.stops >= 1 {
            ui.register_control(F_A, Rect::new(0, 0, 8, 1), Focusability::Focusable);
        }
        if self.stops >= 2 {
            ui.register_control(F_B, Rect::new(0, 2, 8, 1), Focusability::Focusable);
        }
    }
}

/// W-010-05: Tab/BackTab with zero stops never moves; with one stop the
/// focus rests on it and never leaves.
#[test]
fn c010_tab_with_zero_or_single_stop_never_moves() {
    let mut h = Harness::new(Stops { stops: 0 }, Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::BackTab);
    assert_eq!(h.focus(), None);
    let mut h = Harness::new(Stops { stops: 1 }, Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(F_A));
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(F_A));
    let _ = h.key(KeyCode::BackTab);
    assert_eq!(h.focus(), Some(F_A));
}

// ---------------------------------------------------------------------------
// W-010-06: reference inertness with exact-target presentation.
// ---------------------------------------------------------------------------

const R_A: Id = Id::root("c010.r.a");
const R_B: Id = Id::root("c010.r.b");
const R_C: Id = Id::root("c010.r.c");

fn row_styles<A: App>(h: &Harness<A>, y: u16) -> Vec<(Color, Color)> {
    (0..14)
        .map(|x| {
            let cell = h.cell(x, y);
            (cell.bg, cell.fg)
        })
        .collect()
}

#[derive(Default)]
struct RefApp {
    target_b_hovered: bool,
    updates: usize,
    phases: Vec<(Id, Phase)>,
}

impl App for RefApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates = self.updates.saturating_add(1);
        for id in [R_A, R_B, R_C] {
            for intent in cx.intents(id) {
                if let Intent::Pointer { phase, .. } = intent {
                    self.phases.push((id, phase));
                }
            }
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(R_A, "Live action").draw(ui, Rect::new(0, 0, 14, 1));
        let target = self
            .target_b_hovered
            .then(|| ReferenceTarget::new(R_B, ReferenceState::HOVERED));
        ui.reference(target, |ui| {
            // Each callback attempts every registration type through real
            // component draws; all must be suppressed.
            Button::new(R_B, "Same label").draw(ui, Rect::new(0, 2, 14, 1));
            Button::new(R_C, "Same label").draw(ui, Rect::new(0, 4, 14, 1));
        });
    }
}

/// W-010-06: reference draws register no hit/focus/cursor/scroll/binding,
/// force presentation only on the exact target, and run no updates.
#[test]
fn c010_reference_registers_nothing_and_forces_only_exact_target() {
    let mut h = Harness::new(
        RefApp {
            target_b_hovered: true,
            ..RefApp::default()
        },
        Theme::junie(),
        30,
        8,
    );
    // Live control is registered and clickable; reference controls are not.
    assert!(h.area_of(R_A).is_some());
    assert_eq!(h.area_of(R_B), None);
    assert_eq!(h.area_of(R_C), None);
    assert!(h.ring().contains(R_A));
    assert!(!h.ring().contains(R_B));
    assert!(!h.ring().contains(R_C));
    // Clicks on reference-painted regions reach nobody and move no focus
    // from the live startup control.
    assert_eq!(h.focus(), Some(R_A));
    let _ = h.click(7, 2);
    let _ = h.click(7, 4);
    assert!(
        h.app().phases.is_empty(),
        "reference regions must be inert: {:?}",
        h.app().phases
    );
    assert_eq!(h.focus(), Some(R_A));
    let _ = h.click_id(R_A);
    assert_eq!(
        h.app().phases.as_slice(),
        &[
            (R_A, Phase::Press),
            (R_A, Phase::Release),
            (R_A, Phase::Click)
        ]
    );
    // Hover is forced only on the exact target: B's painted styles differ
    // from its identically labelled sibling C.
    assert_ne!(
        row_styles(&h, 2),
        row_styles(&h, 4),
        "forced hover must repaint the target"
    );
    // Drawing runs no updates and changes no durable app state.
    let updates = h.app().updates;
    let phases = h.app().phases.len();
    h.draw();
    assert_eq!(h.app().updates, updates);
    assert_eq!(h.app().phases.len(), phases);
}

/// W-010-06: a targetless reference paints siblings identically — no
/// synthetic runtime state leaks to any control.
#[test]
fn c010_targetless_reference_paints_siblings_identically() {
    let mut targeted = Harness::new(
        RefApp {
            target_b_hovered: true,
            ..RefApp::default()
        },
        Theme::junie(),
        30,
        8,
    );
    let plain = Harness::new(RefApp::default(), Theme::junie(), 30, 8);
    assert_eq!(plain.row(2), plain.row(4));
    assert_eq!(row_styles(&plain, 2), row_styles(&plain, 4));
    assert_eq!(
        row_styles(&plain, 4),
        row_styles(&targeted, 4),
        "the non-target sibling must paint identically with or without a target"
    );
    assert_eq!(plain.area_of(R_B), None);
    assert_eq!(targeted.area_of(R_B), None);
    let _ = targeted.click(7, 2);
    assert!(targeted.app().phases.is_empty());
}

// ---------------------------------------------------------------------------
// W-010-07: typing ownership.
// ---------------------------------------------------------------------------

const T_E1: Id = Id::root("c010.t.e1");
const T_E2: Id = Id::root("c010.t.e2");
const T_E3: Id = Id::root("c010.t.e3");
const T_MODAL: Id = Id::root("c010.t.modal");

#[derive(Default)]
struct Typing {
    open_modal: bool,
    close_modal: bool,
    keys: Vec<(Id, Key)>,
}

impl App for Typing {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for id in [T_E1, T_E2, T_E3, T_MODAL] {
            for intent in cx.intents(id) {
                if let Intent::Key(k) = intent {
                    self.keys.push((id, k));
                }
            }
        }
        if self.open_modal {
            self.open_modal = false;
            cx.open_layer(T_MODAL, LayerSpec::modal(T_MODAL).initial_focus(T_E3));
        }
        if self.close_modal {
            self.close_modal = false;
            cx.close_layer(T_MODAL, None);
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_editor(
            T_E1,
            Rect::new(0, 0, 10, 1),
            Focusability::Focusable,
            StateFlags::EDITING,
        );
        ui.register_editor(
            T_E2,
            Rect::new(0, 2, 10, 1),
            Focusability::Focusable,
            StateFlags::EDITING,
        );
        ui.layer(T_MODAL, |ui, area| {
            ui.register_editor(
                T_E3,
                Rect::new(area.x, area.y, 10, 1),
                Focusability::Focusable,
                StateFlags::EDITING,
            );
        });
    }
}

/// W-010-07: two editors keep exactly one typing owner across route input
/// and resize; every key lands once on the focused editor.
#[test]
fn c010_two_editors_keep_single_typing_owner_across_resize() {
    let mut h = Harness::new(Typing::default(), Theme::junie(), 30, 8);
    let _ = h.click_id(T_E1);
    assert_eq!(h.focus(), Some(T_E1));
    assert_eq!(h.runtime().typing_owner(), Some(T_E1));
    let _ = h.key(KeyCode::Char('x'));
    assert_eq!(h.runtime().typing_owner(), Some(T_E1));
    let _ = h.resize(28, 7);
    assert_eq!(h.focus(), Some(T_E1));
    assert_eq!(h.runtime().typing_owner(), Some(T_E1));
    let _ = h.key(KeyCode::Char('y'));
    assert_eq!(h.runtime().typing_owner(), Some(T_E1));
    assert_eq!(h.app().keys.len(), 2);
    assert!(h.app().keys.iter().all(|(id, _)| *id == T_E1));
    assert_eq!(
        h.app().keys.iter().map(|(_, k)| k.code).collect::<Vec<_>>(),
        vec![KeyCode::Char('x'), KeyCode::Char('y')]
    );
}

/// W-010-07: a modal barrier moves the typing owner to its editor and back
/// on close; nested `FocusOut` settles without duplicating any key.
#[test]
fn c010_modal_barrier_moves_typing_owner_without_duplicate_keys() {
    let mut h = Harness::new(Typing::default(), Theme::junie(), 30, 8);
    let _ = h.click_id(T_E1);
    let _ = h.key(KeyCode::Char('a'));
    h.app_mut().open_modal = true;
    let _ = h.tick();
    assert_eq!(h.focus(), Some(T_E3));
    assert_eq!(h.runtime().typing_owner(), Some(T_E3));
    let _ = h.key(KeyCode::Char('b'));
    h.app_mut().close_modal = true;
    let _ = h.tick();
    assert_eq!(h.focus(), Some(T_E1));
    assert_eq!(h.runtime().typing_owner(), Some(T_E1));
    let _ = h.key(KeyCode::Char('c'));
    assert_eq!(
        h.app()
            .keys
            .iter()
            .map(|(id, k)| (*id, k.code))
            .collect::<Vec<_>>(),
        vec![
            (T_E1, KeyCode::Char('a')),
            (T_E3, KeyCode::Char('b')),
            (T_E1, KeyCode::Char('c')),
        ]
    );
}

// ---------------------------------------------------------------------------
// W-010-08: hover, boundary wheel, ClickOnly and outside dismissal.
// ---------------------------------------------------------------------------

const H_A: Id = Id::root("c010.h.a");
const H_B: Id = Id::root("c010.h.b");
const H_C: Id = Id::root("c010.h.c");

#[derive(Default)]
struct HoverApp {
    phases: Vec<(Id, Phase)>,
}

impl App for HoverApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for id in [H_A, H_B, H_C] {
            for intent in cx.intents(id) {
                if let Intent::Pointer { phase, .. } = intent {
                    self.phases.push((id, phase));
                }
            }
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(H_A, Rect::new(0, 0, 8, 2), Focusability::Focusable);
        ui.register_control(H_B, Rect::new(10, 0, 8, 2), Focusability::Focusable);
        ui.register_control(H_C, Rect::new(20, 0, 8, 2), Focusability::ClickOnly);
    }
}

/// W-010-08: hover never steals focus, keyboard input suppresses hover
/// presentation until the pointer moves again, and a boundary wheel
/// consumes nothing, repaints nothing and moves no focus.
#[test]
fn c010_hover_keyboard_suppression_and_boundary_wheel() {
    let mut h = Harness::new(HoverApp::default(), Theme::junie(), 30, 8);
    // Startup focus rests on the first entry; hovering elsewhere never
    // steals it.
    assert_eq!(h.focus(), Some(H_A));
    let _ = h.mouse(MouseKind::Move, 11, 1);
    assert!(h.state_of(H_B).contains(StateFlags::HOVERED));
    assert_eq!(h.focus(), Some(H_A));
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(H_B));
    // Keyboard presentation rules apply: hover paint is suppressed.
    assert!(!h.state_of(H_B).contains(StateFlags::HOVERED));
    // The next pointer move restores truthful hover without stealing focus.
    let _ = h.mouse(MouseKind::Move, 11, 1);
    assert!(h.state_of(H_B).contains(StateFlags::HOVERED));
    assert_eq!(h.focus(), Some(H_B));
    // Boundary wheel: no scroll region, no repaint, no focus shift.
    let r = h.mouse(MouseKind::Wheel(Axis::V, 1), 11, 1);
    assert!(!r.is_consumed());
    assert_eq!(r.invalidate(), Invalidate::None);
    assert_eq!(h.focus(), Some(H_B));
}

/// W-010-08: a `ClickOnly` target is a hit target, never in the ring; an
/// outside release emits Release without Click and moves no focus.
#[test]
fn c010_click_only_and_outside_release_emit_no_click() {
    let mut h = Harness::new(HoverApp::default(), Theme::junie(), 30, 8);
    assert!(h.area_of(H_C).is_some());
    assert!(!h.ring().contains(H_C));
    assert_eq!(h.focus(), Some(H_A));
    let _ = h.mouse(MouseKind::Down, 21, 1);
    assert_eq!(h.app().phases.as_slice(), &[(H_C, Phase::Press)]);
    assert_eq!(h.focus(), Some(H_A));
    let _ = h.mouse(MouseKind::Up, 29, 7);
    assert_eq!(
        h.app().phases.as_slice(),
        &[(H_C, Phase::Press), (H_C, Phase::Release)]
    );
    assert_eq!(h.focus(), Some(H_A));
    // Ordinary press released outside: Release without Click; the press
    // focus stands.
    let _ = h.mouse(MouseKind::Down, 11, 1);
    assert_eq!(h.focus(), Some(H_B));
    let n = h.app().phases.len();
    let _ = h.mouse(MouseKind::Up, 29, 7);
    assert_eq!(h.app().phases.get(n..).unwrap(), &[(H_B, Phase::Release)]);
    assert_eq!(h.focus(), Some(H_B));
}

// ---------------------------------------------------------------------------
// W-010-09: the physical pass boundary (BF14).
// ---------------------------------------------------------------------------

const P_F: Id = Id::root("c010.p.f");
const P_INNER: Id = Id::root("c010.p.inner");
const P_MODAL: Id = Id::root("c010.p.modal");
const P_CMD: ActionKey = ActionKey::application("c010.pass.command");

/// An app that drains its focused component on EVERY update before also
/// handling `cx.command` — the shape that exposes a frozen-queue replay.
#[derive(Default)]
struct PassBoundary {
    map: KeyMap,
    consume_raw: bool,
    open_modal: bool,
    causes: Vec<(UpdateCause, Option<ActionKey>)>,
    keys: Vec<(Id, KeyCode)>,
    cancels: Vec<Id>,
    focus_events: Vec<(Id, bool)>,
}

impl App for PassBoundary {
    fn keymap(&self) -> &KeyMap {
        &self.map
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let before = self.keys.len();
        for id in [P_F, P_INNER, P_MODAL] {
            for intent in cx.intents(id) {
                match intent {
                    Intent::Key(k) => self.keys.push((id, k.code)),
                    Intent::Cancel => self.cancels.push(id),
                    Intent::FocusIn { .. } => self.focus_events.push((id, true)),
                    Intent::FocusOut { .. } => self.focus_events.push((id, false)),
                    _ => {}
                }
            }
        }
        self.causes.push((cx.update_cause(), cx.command()));
        // A raw-consuming control: consume only a pass that actually carried
        // the physical key, so Tab traversal still works.
        if self.consume_raw
            && cx.command().is_none()
            && cx.update_cause() == UpdateCause::Event
            && self.keys.len() > before
        {
            return Response::consumed();
        }
        if self.open_modal {
            self.open_modal = false;
            cx.open_layer(P_MODAL, LayerSpec::modal(P_MODAL).initial_focus(P_INNER));
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(P_F, Rect::new(0, 0, 8, 1), Focusability::Focusable);
        ui.layer(P_MODAL, |ui, area| {
            ui.register_control(P_INNER, area, Focusability::Focusable);
        });
    }
}

fn pass_app(map: KeyMap, consume_raw: bool) -> Harness<PassBoundary> {
    Harness::new(
        PassBoundary {
            map,
            consume_raw,
            ..PassBoundary::default()
        },
        Theme::junie(),
        30,
        8,
    )
}

/// W-010-09: a raw key ignored by the focused component reaches the Bubble
/// command exactly once — one physical Key across the raw and follow-up
/// passes, the command at its own boundary.
#[test]
fn c010_bubble_command_sees_physical_key_exactly_once() {
    let map = KeyMap::new().bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('x')), P_CMD);
    let mut h = pass_app(map, false);
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(P_F));
    let (causes, keys) = (h.app().causes.len(), h.app().keys.len());
    let _ = h.key(KeyCode::Char('x'));
    let tail = h.app().causes.get(causes..).unwrap();
    assert_eq!(
        tail,
        &[
            (UpdateCause::Event, None),
            (UpdateCause::Event, Some(P_CMD))
        ],
        "raw and bubble passes must both run"
    );
    assert_eq!(
        h.app().keys.get(keys..).unwrap(),
        &[(P_F, KeyCode::Char('x'))],
        "the physical Key must appear exactly once across both passes"
    );
}

/// W-010-09: a raw-consuming control blocks the follow-up pass — the key
/// arrives once and the Bubble command never fires.
#[test]
fn c010_consumed_raw_key_blocks_bubble_followup() {
    let map = KeyMap::new().bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('x')), P_CMD);
    let mut h = pass_app(map, true);
    let _ = h.key(KeyCode::Tab);
    let (causes, keys) = (h.app().causes.len(), h.app().keys.len());
    let _ = h.key(KeyCode::Char('x'));
    let tail = h.app().causes.get(causes..).unwrap();
    assert_eq!(tail, &[(UpdateCause::Event, None)]);
    assert_eq!(
        h.app().keys.get(keys..).unwrap(),
        &[(P_F, KeyCode::Char('x'))]
    );
}

/// W-010-09: raw Esc followed by dismissal delivers one physical Key, one
/// Cancel and one genuine focus transition — never a replayed Key.
#[test]
fn c010_esc_dismissal_delivers_key_and_cancel_exactly_once() {
    let mut h = pass_app(KeyMap::new(), false);
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(P_F));
    h.app_mut().open_modal = true;
    let _ = h.tick();
    assert_eq!(h.focus(), Some(P_INNER));
    assert!(h.is_open(P_MODAL));
    let (keys, cancels, focus) = (
        h.app().keys.len(),
        h.app().cancels.len(),
        h.app().focus_events.len(),
    );
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(P_MODAL));
    assert_eq!(h.focus(), Some(P_F));
    assert_eq!(
        h.app().keys.get(keys..).unwrap(),
        &[(P_INNER, KeyCode::Esc)],
        "raw Esc must be delivered exactly once"
    );
    assert_eq!(
        h.app().cancels.get(cancels..).unwrap(),
        &[P_MODAL],
        "Cancel must reach the dismissed layer owner exactly once"
    );
    let tail = h.app().focus_events.get(focus..).unwrap();
    assert_eq!(
        tail.iter()
            .filter(|(id, inn)| *id == P_INNER && !*inn)
            .count(),
        1,
        "genuine FocusOut must arrive once: {tail:?}"
    );
    assert_eq!(
        tail.iter().filter(|(id, inn)| *id == P_F && *inn).count(),
        1,
        "genuine restore FocusIn must arrive once: {tail:?}"
    );
}

// ---------------------------------------------------------------------------
// W-010-10: effective-chord identity on the public surface.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum ChordCmd {
    A,
    B,
}

fn chord_binding(action: ActionKey, chord: Chord, cmd: ChordCmd) -> Binding<ChordCmd> {
    Binding {
        action,
        chord: Some(chord),
        cmd,
        label: "test",
        priority: 1,
        visible: true,
    }
}

const CHORD_FIRST: ActionKey = ActionKey::custom("c010.chord.first");
const CHORD_SECOND: ActionKey = ActionKey::custom("c010.chord.second");

/// W-010-10: `NONE` and `SHIFT` forms of the same uppercase char conflict
/// in every public scope — table, ordinary and typing.
#[test]
fn c010_shifted_uppercase_conflicts_in_every_public_scope() {
    let owner = Id::root("c010.chord");
    let table = [
        chord_binding(CHORD_FIRST, Chord::key(KeyCode::Char('A')), ChordCmd::A),
        chord_binding(
            CHORD_SECOND,
            Chord::with(KeyCode::Char('A'), KeyModifiers::SHIFT),
            ChordCmd::B,
        ),
    ];
    assert_eq!(binding_conflicts(owner, KeyPhase::Bubble, &table).len(), 1);
    let map = KeyMap::new()
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('A')),
            ActionKey::CLOSE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('A'), KeyModifiers::SHIFT),
            ActionKey::SAVE,
        );
    assert_eq!(map.conflicts().len(), 1);
    let typed = KeyMap::new()
        .bind_before_typing(owner, Chord::key(KeyCode::Char('A')), ActionKey::CLOSE)
        .bind_before_typing(
            owner,
            Chord::with(KeyCode::Char('A'), KeyModifiers::SHIFT),
            ActionKey::SAVE,
        );
    assert_eq!(typed.conflicts().len(), 1);
}

/// W-010-10: case and non-character modifiers stay distinct in every
/// public scope, while structurally identical chords still conflict.
#[test]
fn c010_case_and_noncharacter_modifiers_stay_distinct() {
    let owner = Id::root("c010.chord.distinct");
    for (first, second, conflicts) in [
        (
            Chord::key(KeyCode::Char('a')),
            Chord::key(KeyCode::Char('A')),
            0,
        ),
        (
            Chord::key(KeyCode::Up),
            Chord::with(KeyCode::Up, KeyModifiers::SHIFT),
            0,
        ),
        (
            Chord::key(KeyCode::Char('A')),
            Chord::key(KeyCode::Char('A')),
            1,
        ),
    ] {
        let table = [
            chord_binding(CHORD_FIRST, first, ChordCmd::A),
            chord_binding(CHORD_SECOND, second, ChordCmd::B),
        ];
        assert_eq!(
            binding_conflicts(owner, KeyPhase::Bubble, &table).len(),
            conflicts,
            "table {first:?} vs {second:?}"
        );
        let map = KeyMap::new()
            .bind(KeyPhase::Bubble, first, ActionKey::CLOSE)
            .bind(KeyPhase::Bubble, second, ActionKey::SAVE);
        assert_eq!(
            map.conflicts().len(),
            conflicts,
            "ordinary {first:?} vs {second:?}"
        );
        let typed = KeyMap::new()
            .bind_before_typing(owner, first, ActionKey::CLOSE)
            .bind_before_typing(owner, second, ActionKey::SAVE);
        assert_eq!(
            typed.conflicts().len(),
            conflicts,
            "typing {first:?} vs {second:?}"
        );
    }
    // Matching folds SHIFT only for the same exact char.
    let upper = Chord::key(KeyCode::Char('A'));
    assert!(upper.matches(&Key {
        code: KeyCode::Char('A'),
        mods: KeyModifiers::SHIFT
    }));
    assert!(!upper.matches(&Key {
        code: KeyCode::Char('a'),
        mods: KeyModifiers::NONE
    }));
    assert!(!Chord::key(KeyCode::Up).matches(&Key {
        code: KeyCode::Up,
        mods: KeyModifiers::SHIFT
    }));
    // Structural identity still distinguishes the folded forms.
    assert_ne!(
        Chord::key(KeyCode::Char('A')),
        Chord::with(KeyCode::Char('A'), KeyModifiers::SHIFT)
    );
}

/// W-010-10: routing and conflicts agree end to end — a `SHIFT`ed char key
/// reaches the `NONE` Bubble binding exactly once.
#[test]
fn c010_shifted_char_routing_matches_effective_conflicts() {
    let map = KeyMap::new().bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('A')), P_CMD);
    let mut h = pass_app(map, false);
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(P_F));
    let (causes, keys) = (h.app().causes.len(), h.app().keys.len());
    let _ = h.key_mod(KeyCode::Char('A'), KeyModifiers::SHIFT);
    assert_eq!(
        h.app().causes.get(causes..).unwrap(),
        &[
            (UpdateCause::Event, None),
            (UpdateCause::Event, Some(P_CMD))
        ]
    );
    assert_eq!(
        h.app().keys.get(keys..).unwrap(),
        &[(P_F, KeyCode::Char('A'))]
    );
}
