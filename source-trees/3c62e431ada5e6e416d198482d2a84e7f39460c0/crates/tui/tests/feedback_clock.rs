//! Exact coalesced simulation feedback, independent of elapsed input scheduling.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    reason = "deterministic clock and publication assertions"
)]
use core::time::Duration;
use junie_tui::*;
use junie_tui_testing::{
    Scene,
    perf::{Counting, bench, lock},
};
#[global_allocator]
static ALLOCATOR: Counting = Counting;
const BUTTON: Id = Id::root("feedback.button");
const FUTURE: Id = Id::root("feedback.future");
const MODAL: Id = Id::root("feedback.modal");
const NESTED: Id = Id::root("feedback.nested");
const NESTED_BUTTON: Id = Id::root("feedback.nested.button");
const MODAL_BUTTON: Id = Id::root("feedback.modal.button");
const AREA: Rect = Rect::new(0, 0, 30, 6);
#[derive(Clone, Copy)]
enum Motion {
    Full,
    Reduced,
    Paused,
}
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent model policy switches"
)]
struct Model {
    world: u64,
    motion: Motion,
    last_wake: Moment,
    keyboard_feedback: bool,
    sync: Option<SimulationMoment>,
    sync_result: Option<Result<(), FeedbackClockError>>,
    request: Option<(Id, PartRef)>,
    show_future: bool,
    modal: bool,
    open_modal: bool,
    open_nested: bool,
    activations: usize,
    updates: usize,
}
impl Model {
    fn new(world: u64, motion: Motion) -> Self {
        Self {
            world,
            motion,
            last_wake: Moment::ZERO,
            keyboard_feedback: false,
            sync: None,
            sync_result: None,
            request: None,
            show_future: false,
            modal: false,
            open_modal: false,
            open_nested: false,
            activations: 0,
            updates: 0,
        }
    }
    fn cadence(cx: &Cx<'_>) -> Duration {
        Duration::from_millis(if cx.activation_feedback().is_some() {
            80
        } else {
            200
        })
    }
}
impl App for Model {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates += 1;
        if cx.update_cause() == UpdateCause::Bootstrap {
            self.last_wake = cx.now();
        }
        if let Some(now) = self.sync.take() {
            self.sync_result = Some(cx.sync_feedback_time(now));
        }
        if !matches!(self.motion, Motion::Paused) {
            let cadence = Self::cadence(cx);
            let due = self.last_wake.saturating_add(cadence);
            if cx.update_cause() == UpdateCause::Tick && cx.now() >= due {
                self.world = self
                    .world
                    .saturating_add(u64::try_from(cadence.as_millis()).unwrap());
                cx.sync_feedback_time(SimulationMoment::from_millis(self.world))
                    .unwrap();
                self.last_wake = cx.now();
            }
        }
        let response = Button::new(BUTTON, "activate").update(cx);
        if response.activated() {
            self.activations += 1;
            if self.keyboard_feedback {
                cx.flash_activation(BUTTON);
            }
        }
        for _ in cx.intents(FUTURE) {}
        for _ in cx.intents(MODAL) {}
        let _ = Button::new(MODAL_BUTTON, "modal").update(cx);
        let _ = Button::new(NESTED_BUTTON, "nested").update(cx);
        for _ in cx.intents(NESTED) {}
        if self.open_nested {
            self.open_nested = false;
            cx.open_layer(NESTED, LayerSpec::modal(NESTED));
        }
        if self.open_modal {
            self.open_modal = false;
            self.modal = true;
            cx.open_layer(MODAL, LayerSpec::modal(MODAL));
        }
        if let Some((owner, part)) = self.request.take() {
            cx.flash_activation_part(owner, part);
        }
        if !matches!(self.motion, Motion::Paused) {
            cx.request_repaint_at(self.last_wake.saturating_add(Self::cadence(cx)));
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(BUTTON, "activate").draw(ui, Rect::new(0, 0, 12, 1));
        if self.show_future {
            ui.register_control(FUTURE, Rect::new(0, 2, 20, 2), Focusability::Focusable);
            for (index, key) in [ItemKey::text("first"), ItemKey::text("second")]
                .into_iter()
                .enumerate()
            {
                let part = PartRef::item(Part::ROW, key);
                let row = Rect::new(0, 2 + u16::try_from(index).unwrap(), 20, 1);
                ui.register_part(FUTURE, part, row);
                let text = if ui.pressed_part(FUTURE) == Some(part) {
                    "FLASH"
                } else {
                    "quiet"
                };
                ui.paint_str(row, text, ui.surface_style());
            }
        }
        ui.layer(NESTED, |ui, _| {
            Button::new(NESTED_BUTTON, "nested").draw(ui, Rect::new(0, 5, 12, 1));
        });
        ui.layer(MODAL, |ui, _| {
            Button::new(MODAL_BUTTON, "modal").draw(ui, Rect::new(0, 4, 12, 1));
        });
    }
}
fn present(rt: &mut Runtime<Model>, buf: &mut Buffer) {
    for _ in 0..16 {
        if rt.needs_settle() {
            let _ = rt.settle();
        }
        buf.reset();
        rt.draw_buffer(AREA, buf).commit_presented();
        if !rt.needs_settle() && !rt.needs_present() {
            return;
        }
    }
    panic!("feedback fixture did not settle");
}
fn runtime(world: u64, motion: Motion) -> (Runtime<Model>, Buffer) {
    let mut rt = Runtime::new_with_feedback_clock(
        Model::new(world, motion),
        Theme::junie(),
        FeedbackClock::Simulation {
            initial: SimulationMoment::from_millis(world),
        },
    );
    let _ = rt.initialize();
    let mut buf = Buffer::empty(AREA);
    present(&mut rt, &mut buf);
    (rt, buf)
}
fn send(rt: &mut Runtime<Model>, buf: &mut Buffer, input: Input) {
    present(rt, buf);
    let _ = rt.handle(input).unwrap();
    present(rt, buf);
}
fn click(rt: &mut Runtime<Model>, buf: &mut Buffer) {
    for kind in [MouseKind::Down, MouseKind::Up] {
        send(
            rt,
            buf,
            Input::Mouse(Mouse {
                kind,
                pos: Position::new(2, 0),
                mods: KeyModifiers::NONE,
            }),
        );
    }
}
fn advance(rt: &mut Runtime<Model>, buf: &mut Buffer, ms: u64) {
    rt.advance_to(Moment::from_millis(ms)).unwrap();
    present(rt, buf);
}
#[test]
fn full_and_reduced_feedback_match_source_ordinary_and_delayed_coalesced_steps() {
    for motion in [Motion::Full, Motion::Reduced] {
        for wakes in [[80, 160, 360], [1800, 1880, 2080]] {
            let (mut rt, mut buf) = runtime(4000, motion);
            click(&mut rt, &mut buf);
            assert_eq!(rt.next_deadline(), Some(Moment::from_millis(80)));
            assert_eq!(
                rt.activation_feedback().unwrap().remaining,
                Duration::from_millis(140)
            );
            advance(&mut rt, &mut buf, wakes[0]);
            assert_eq!(rt.app().world, 4080);
            assert_eq!(
                rt.activation_feedback().unwrap().remaining,
                Duration::from_millis(60)
            );
            assert_eq!(rt.next_deadline(), Some(Moment::from_millis(wakes[0] + 80)));
            advance(&mut rt, &mut buf, wakes[1]);
            assert_eq!(rt.app().world, 4160);
            assert_eq!(rt.activation_feedback(), None);
            assert_eq!(
                rt.next_deadline(),
                Some(Moment::from_millis(wakes[1] + 200))
            );
            advance(&mut rt, &mut buf, wakes[2]);
            assert_eq!(rt.app().world, 4360);
        }
    }
}
#[test]
fn paused_feedback_has_no_wall_deadline_or_idle_spin_and_keeps_nonzero_epoch() {
    for initial in [0, 4000, i64::MAX as u64] {
        let (mut rt, mut buf) = runtime(initial, Motion::Paused);
        click(&mut rt, &mut buf);
        for ms in [140, 500, 1800, 90000] {
            advance(&mut rt, &mut buf, ms);
            assert_eq!(rt.app().world, initial);
            assert_eq!(
                rt.activation_feedback().unwrap().remaining,
                Duration::from_millis(140)
            );
            assert_eq!(rt.next_deadline(), None);
            assert!(!rt.wants_tick());
        }
    }
}

#[test]
fn arbitrary_tick_key_paste_resize_and_draw_do_not_advance_simulation_or_feedback() {
    let (mut rt, mut buf) = runtime(4000, Motion::Full);
    click(&mut rt, &mut buf);
    for _ in 0..100 {
        for input in [
            Input::Tick,
            Input::Key(Key {
                code: KeyCode::Char('x'),
                mods: KeyModifiers::NONE,
            }),
            Input::Paste("private".into()),
            Input::Resize(30, 6),
        ] {
            send(&mut rt, &mut buf, input);
        }
    }
    advance(&mut rt, &mut buf, 79);
    for _ in 0..100 {
        send(&mut rt, &mut buf, Input::Tick);
    }
    assert_eq!(rt.app().world, 4000);
    assert!(
        rt.state_of(BUTTON).contains(StateFlags::PRESSED),
        "resize must not erase an active feedback record's visible state"
    );
    assert_eq!(
        rt.activation_feedback().unwrap().remaining,
        Duration::from_millis(140)
    );
    advance(&mut rt, &mut buf, 1800);
    assert_eq!(rt.app().world, 4080);
}
#[test]
fn synchronization_is_equal_time_idempotent_and_backwards_or_wrong_policy_atomic() {
    let (mut rt, mut buf) = runtime(4000, Motion::Paused);
    click(&mut rt, &mut buf);
    let before = rt.activation_feedback();
    for requested in [4000, 3999] {
        rt.app_mut().sync = Some(SimulationMoment::from_millis(requested));
        send(&mut rt, &mut buf, Input::Tick);
        assert_eq!(rt.activation_feedback(), before);
        assert_eq!(
            rt.app().sync_result,
            Some(if requested == 4000 {
                Ok(())
            } else {
                Err(FeedbackClockError::Backwards {
                    current: SimulationMoment::from_millis(4000),
                    requested: SimulationMoment::from_millis(3999),
                })
            })
        );
    }
    let mut elapsed = Runtime::new(Model::new(0, Motion::Paused), Theme::junie());
    let _ = elapsed.initialize();
    present(&mut elapsed, &mut buf);
    click(&mut elapsed, &mut buf);
    let before = elapsed.activation_feedback();
    elapsed.app_mut().sync = Some(SimulationMoment::from_millis(9000));
    send(&mut elapsed, &mut buf, Input::Tick);
    assert_eq!(
        elapsed.app().sync_result,
        Some(Err(FeedbackClockError::WrongPolicy))
    );
    assert_eq!(elapsed.activation_feedback(), before);
    assert_eq!(elapsed.next_deadline(), Some(Moment::from_millis(140)));
    advance(&mut elapsed, &mut buf, 139);
    assert!(elapsed.activation_feedback().is_some());
    advance(&mut elapsed, &mut buf, 140);
    assert_eq!(elapsed.activation_feedback(), None);
}
#[test]
fn semantic_keyboard_feedback_is_explicit_and_reactivation_replaces_the_one_record() {
    let (mut rt, mut buf) = runtime(0, Motion::Paused);
    send(
        &mut rt,
        &mut buf,
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
    );
    assert_eq!(rt.app().activations, 1);
    assert_eq!(rt.activation_feedback(), None);
    rt.app_mut().keyboard_feedback = true;
    send(
        &mut rt,
        &mut buf,
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
    );
    assert!(rt.activation_feedback().is_some());
    rt.app_mut().sync = Some(SimulationMoment::from_millis(80));
    send(&mut rt, &mut buf, Input::Tick);
    assert_eq!(
        rt.activation_feedback().unwrap().remaining,
        Duration::from_millis(60)
    );
    rt.app_mut().request = Some((FUTURE, PartRef::item(Part::ROW, ItemKey::text("second"))));
    send(&mut rt, &mut buf, Input::Tick);
    assert_eq!(rt.activation_feedback().unwrap().owner, FUTURE);
    assert_eq!(
        rt.activation_feedback().unwrap().remaining,
        Duration::from_millis(140)
    );
    rt.app_mut().sync = Some(SimulationMoment::from_millis(160));
    send(&mut rt, &mut buf, Input::Tick);
    assert!(rt.activation_feedback().is_some());
    rt.app_mut().sync = Some(SimulationMoment::from_millis(220));
    send(&mut rt, &mut buf, Input::Tick);
    assert_eq!(rt.activation_feedback(), None);
}

#[test]
fn future_owner_and_stable_row_part_keep_cadence_without_granting_focus_or_input() {
    let (mut rt, mut buf) = runtime(0, Motion::Full);
    let part = PartRef::item(Part::ROW, ItemKey::text("second"));
    rt.app_mut().request = Some((FUTURE, part));
    send(&mut rt, &mut buf, Input::Tick);
    assert_eq!(rt.focus(), Some(BUTTON));
    assert_eq!(rt.area_of(FUTURE), None);
    assert_eq!(rt.next_deadline(), Some(Moment::from_millis(80)));
    send(
        &mut rt,
        &mut buf,
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
    );
    assert_eq!(rt.app().activations, 1);
    assert_eq!(rt.activation_feedback().unwrap().owner, FUTURE);
    rt.app_mut().show_future = true;
    present(&mut rt, &mut buf);
    assert_eq!(buf[(0, 2)].symbol(), "q");
    assert_eq!(buf[(0, 3)].symbol(), "F");
    assert_eq!(rt.focus(), Some(BUTTON));
    rt.app_mut().open_modal = true;
    send(&mut rt, &mut buf, Input::Tick);
    assert_eq!(rt.focus(), Some(MODAL_BUTTON));
    assert!(!rt.state_of(MODAL_BUTTON).contains(StateFlags::PRESSED));
    rt.app_mut().open_nested = true;
    send(&mut rt, &mut buf, Input::Tick);
    assert_eq!(rt.focus(), Some(NESTED_BUTTON));
    assert!(!rt.state_of(NESTED_BUTTON).contains(StateFlags::PRESSED));
    send(
        &mut rt,
        &mut buf,
        Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }),
    );
    assert_eq!(rt.app().activations, 1);
    assert_eq!(rt.activation_feedback().unwrap().owner, FUTURE);
    rt.app_mut().show_future = false;
    present(&mut rt, &mut buf);
    assert!(rt.activation_feedback().is_some());
    advance(&mut rt, &mut buf, 80);
    assert!(rt.activation_feedback().is_some());
    advance(&mut rt, &mut buf, 160);
    assert_eq!(rt.activation_feedback(), None);
}
#[test]
fn simulation_expiry_preserves_an_independent_held_pointer_press() {
    let (mut rt, mut buf) = runtime(0, Motion::Paused);
    click(&mut rt, &mut buf);
    rt.app_mut().show_future = true;
    present(&mut rt, &mut buf);
    send(
        &mut rt,
        &mut buf,
        Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: Position::new(2, 2),
            mods: KeyModifiers::NONE,
        }),
    );
    rt.app_mut().sync = Some(SimulationMoment::from_millis(140));
    send(&mut rt, &mut buf, Input::Tick);
    assert_eq!(rt.activation_feedback(), None);
    assert!(rt.state_of(FUTURE).contains(StateFlags::PRESSED));
    send(
        &mut rt,
        &mut buf,
        Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: Position::new(29, 5),
            mods: KeyModifiers::NONE,
        }),
    );
    assert!(!rt.state_of(FUTURE).contains(StateFlags::PRESSED));
}
#[test]
fn bound_scene_freezes_feedback_without_clock_or_update_effects_and_allocates_nothing() {
    let _guard = lock();
    let (mut rt, mut buf) = runtime(4000, Motion::Paused);
    click(&mut rt, &mut buf);
    let feedback = rt.activation_feedback();
    let updates = rt.app().updates;
    let mut scene = Scene::new(
        "simulation-feedback",
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        6,
    );
    scene.set_snapshot(rt.render_snapshot().unwrap());
    let mut bound = scene.bind_app(rt.app());
    bound.draw();
    let first = bound.scene().buffer().clone();
    let cursor = bound.scene().cursor_position();
    bound.draw();
    assert_eq!(bound.scene().buffer(), &first);
    assert_eq!(bound.scene().cursor_position(), cursor);
    let stats = bench(4, 100, &mut || bound.draw());
    assert_eq!((stats.allocs, stats.bytes), (0, 0));
    assert_eq!(rt.activation_feedback(), feedback);
    assert_eq!(rt.app().updates, updates);
    assert_eq!(rt.app().world, 4000);
}
#[test]
fn harness_constructor_selects_simulation_epoch_before_bootstrap() {
    let mut h = junie_tui_testing::Harness::new_with_feedback_clock(
        Model::new(4000, Motion::Paused),
        Theme::junie(),
        30,
        6,
        FeedbackClock::Simulation {
            initial: SimulationMoment::from_millis(4000),
        },
    );
    let _ = h.handle(Input::Mouse(Mouse {
        kind: MouseKind::Down,
        pos: Position::new(2, 0),
        mods: KeyModifiers::NONE,
    }));
    let _ = h.handle(Input::Mouse(Mouse {
        kind: MouseKind::Up,
        pos: Position::new(2, 0),
        mods: KeyModifiers::NONE,
    }));
    let _ = h.advance(Duration::from_millis(1800));
    assert_eq!(
        h.activation_feedback().unwrap().remaining,
        Duration::from_millis(140)
    );
}

/// Run explicitly under a PTY; the parent waits for READY before observing idle.
#[test]
#[ignore = "real PTY fixture; external readiness-driven driver supplies q"]
#[cfg(feature = "crossterm")]
#[expect(
    clippy::print_stdout,
    reason = "readiness and completion markers for the explicit real PTY fixture"
)]
fn simulation_feedback_driver_paused_fixture() {
    struct Paused;
    impl App for Paused {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.flash_activation(BUTTON);
                println!("SIMULATION_FEEDBACK_READY");
            }
            let quit = cx.intents(BUTTON).any(|intent| {
                matches!(
                    intent,
                    Intent::Key(Key {
                        code: KeyCode::Char('q'),
                        ..
                    })
                )
            });
            if quit {
                assert_eq!(
                    cx.activation_feedback().unwrap().remaining,
                    Duration::from_millis(140)
                );
                println!("SIMULATION_FEEDBACK_PAUSED_OK");
                cx.quit();
            }
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_control(BUTTON, Rect::new(0, 0, 10, 1), Focusability::Focusable);
        }
    }
    run_with_feedback_clock(
        Paused,
        Theme::junie(),
        FeedbackClock::Simulation {
            initial: SimulationMoment::from_millis(4000),
        },
    )
    .unwrap();
}

#[test]
fn warm_equal_time_sync_and_live_publication_allocate_nothing() {
    let _guard = lock();
    let (mut rt, mut buf) = runtime(4000, Motion::Paused);
    click(&mut rt, &mut buf);
    let stats = bench(4, 100, &mut || {
        rt.app_mut().sync = Some(SimulationMoment::from_millis(4000));
        present(&mut rt, &mut buf);
        let _ = rt.handle(Input::Tick).expect("published feedback");
    });
    assert_eq!((stats.allocs, stats.bytes), (0, 0));
    assert_eq!(
        rt.activation_feedback().unwrap().remaining,
        Duration::from_millis(140)
    );
}

#[test]
fn activation_reanchors_to_previous_admitted_wake_without_postponing_overdue_work() {
    for activation_at in [50, 150] {
        let (mut rt, mut buf) = runtime(0, Motion::Full);
        advance(&mut rt, &mut buf, activation_at);
        click(&mut rt, &mut buf);
        assert_eq!(rt.next_deadline(), Some(Moment::from_millis(80)));
        advance(&mut rt, &mut buf, activation_at.max(80));
        assert_eq!(rt.app().world, 80);
        assert_eq!(
            rt.next_deadline(),
            Some(Moment::from_millis(activation_at.max(80) + 80))
        );
    }
}
