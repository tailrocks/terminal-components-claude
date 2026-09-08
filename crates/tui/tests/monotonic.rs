//! Explicit monotonic time, deadline ordering and real terminal driver fixtures.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use core::time::Duration;
use junie_tui::{
    App, Buffer, Cx, Focusability, FrameRead, Id, Input, Intent, Key, KeyCode, KeyModifiers,
    Moment, Rect, Response, Runtime, StateFlags, Theme, Ui, UpdateCause,
};
use junie_tui_testing::Harness;

const EDITOR: Id = Id::root("monotonic.editor");
const AREA: Rect = Rect::new(0, 0, 20, 4);

struct Timed {
    delay: Option<Duration>,
    repeat: bool,
    ticks: usize,
    doubles: usize,
    updates: usize,
    causes: Vec<UpdateCause>,
    received: String,
    order: Vec<&'static str>,
}
impl Timed {
    fn delayed(ms: u64) -> Self {
        Self {
            delay: Some(Duration::from_millis(ms)),
            repeat: false,
            ticks: 0,
            doubles: 0,
            updates: 0,
            causes: Vec::new(),
            received: String::new(),
            order: Vec::new(),
        }
    }
}
impl App for Timed {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates += 1;
        self.causes.push(cx.update_cause());
        if cx.update_cause() == UpdateCause::Bootstrap {
            if let Some(delay) = self.delay {
                cx.request_repaint_after(delay);
            }
        } else if cx.update_cause() == UpdateCause::Tick {
            self.ticks += 1;
            self.order.push("due");
            if self.repeat
                && let Some(delay) = self.delay
            {
                cx.request_repaint_after(delay);
            }
        }
        for input in cx.intents(EDITOR) {
            match input {
                Intent::Paste(text) => {
                    self.received.push_str(text);
                    self.order.push("paste");
                }
                Intent::Key(Key {
                    code: KeyCode::Char(c),
                    ..
                }) => {
                    self.received.push(c);
                    self.order.push("key");
                }
                Intent::Pointer {
                    phase: junie_tui::Phase::DoubleClick,
                    ..
                } => self.doubles += 1,
                _ => {}
            }
        }
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let x = if self.ticks == 0 { 0 } else { 5 };
        let area = Rect::new(x, 0, 8, 1);
        ui.register_editor(EDITOR, area, Focusability::Focusable, StateFlags::EDITING);
        let label = if ui.state(EDITOR).contains(StateFlags::PRESSED) {
            "pressed"
        } else {
            "ready"
        };
        ui.paint_str(area, label, ui.surface_style());
    }
}
fn present(rt: &mut Runtime<Timed>, buffer: &mut Buffer) {
    for _ in 0..16 {
        if rt.needs_settle() {
            let _ = rt.settle();
        }
        buffer.reset();
        rt.draw_buffer(AREA, buffer).commit_presented();
        if !rt.needs_settle() && !rt.needs_present() {
            return;
        }
    }
    panic!("fixture did not settle")
}
fn runtime(app: Timed) -> (Runtime<Timed>, Buffer) {
    let mut rt = Runtime::new(app, Theme::junie());
    let _ = rt.initialize();
    let mut buffer = Buffer::empty(AREA);
    present(&mut rt, &mut buffer);
    (rt, buffer)
}

#[test]
fn absolute_2199_2200_deadline_requires_explicit_settle_and_draw_is_effect_free() {
    let (mut rt, mut buffer) = runtime(Timed::delayed(2200));
    let updates = rt.app().updates;
    rt.advance_to(Moment::from_millis(2199)).unwrap();
    assert!(!rt.needs_settle());
    assert_eq!(rt.app().ticks, 0);
    rt.advance_to(Moment::from_millis(2200)).unwrap();
    assert!(rt.needs_settle());
    assert!(rt.wants_tick());
    for _ in 0..100 {
        rt.draw_buffer(AREA, &mut buffer).commit_presented();
    }
    assert_eq!(rt.app().updates, updates);
    assert_eq!(rt.app().ticks, 0);
    let _ = rt.settle();
    assert_eq!(rt.app().ticks, 1);
    assert_eq!(rt.next_deadline(), None);
    let _ = rt.settle();
    assert_eq!(rt.app().ticks, 1);
}

#[test]
fn input_count_and_explicit_ticks_do_not_age_or_consume_future_deadlines() {
    let mut h = Harness::new(Timed::delayed(2200), Theme::junie(), 20, 4);
    for _ in 0..1000 {
        let _ = h.key(KeyCode::Char('x'));
    }
    h.ticks(3);
    assert_eq!(h.runtime().now(), Moment::ZERO);
    assert_eq!(h.runtime().next_deadline(), Some(Moment::from_millis(2200)));
    let _ = h.advance(Duration::from_millis(2199));
    assert_eq!(h.app().ticks, 3);
    let _ = h.advance(Duration::from_millis(1));
    assert_eq!(h.app().ticks, 4);
}

#[test]
fn immediate_rearm_waits_for_next_turn_and_retained_paste_and_key_progress() {
    let mut app = Timed::delayed(0);
    app.repeat = true;
    let (mut rt, mut buffer) = runtime(app);
    for (input, expected) in [
        (Input::Paste("first".into()), "first"),
        (
            Input::Key(Key {
                code: KeyCode::Char('!'),
                mods: KeyModifiers::NONE,
            }),
            "first!",
        ),
    ] {
        rt.advance_to(Moment::ZERO).unwrap();
        let pending = rt.handle(input).unwrap_err();
        present(&mut rt, &mut buffer);
        assert!(!rt.needs_settle(), "immediate rearm must not recurse");
        assert_eq!(rt.next_deadline(), Some(Moment::ZERO));
        let _ = rt.handle(pending.into_input()).unwrap();
        assert_eq!(rt.app().received, expected);
        present(&mut rt, &mut buffer);
    }
    assert_eq!(rt.app().order, ["due", "paste", "due", "key"]);
    assert_eq!(rt.app().ticks, 2);
}

#[test]
fn retained_resize_and_paste_use_geometry_after_due_route_update() {
    let (mut rt, mut buffer) = runtime(Timed::delayed(2200));
    rt.advance_to(Moment::from_millis(2200)).unwrap();
    let resize = rt.handle(Input::Resize(20, 4)).unwrap_err();
    present(&mut rt, &mut buffer);
    assert_eq!(rt.area_of(EDITOR), Some(Rect::new(5, 0, 8, 1)));
    let _ = rt.handle(resize.into_input()).unwrap();
    let paste = rt.handle(Input::Paste("retained\n二".into())).unwrap_err();
    present(&mut rt, &mut buffer);
    let _ = rt.handle(paste.into_input()).unwrap();
    assert_eq!(rt.app().received, "retained\n二");
}

#[test]
fn backward_time_rejects_without_changing_model_deadlines_or_publication() {
    let (mut rt, _) = runtime(Timed::delayed(2200));
    rt.advance_to(Moment::from_millis(100)).unwrap();
    let before = (
        rt.now(),
        rt.next_deadline(),
        rt.needs_present(),
        rt.needs_settle(),
        rt.app().updates,
    );
    let error = rt.advance_to(Moment::from_millis(99)).unwrap_err();
    assert_eq!(error.current, Moment::from_millis(100));
    assert_eq!(
        (
            rt.now(),
            rt.next_deadline(),
            rt.needs_present(),
            rt.needs_settle(),
            rt.app().updates
        ),
        before
    );
}

#[test]
fn nonzero_origin_and_same_duration_rearms_use_absolute_moments() {
    let mut app = Timed::delayed(100);
    app.repeat = true;
    let mut rt = Runtime::new(app, Theme::junie());
    rt.advance_to(Moment::from_millis(500)).unwrap();
    let _ = rt.initialize();
    assert_eq!(rt.next_deadline(), Some(Moment::from_millis(600)));
    let mut buffer = Buffer::empty(AREA);
    present(&mut rt, &mut buffer);
    rt.advance_to(Moment::from_millis(600)).unwrap();
    present(&mut rt, &mut buffer);
    assert_eq!(rt.next_deadline(), Some(Moment::from_millis(700)));
    let _ = rt
        .handle(Input::Key(Key {
            code: KeyCode::Char('x'),
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    rt.advance_to(Moment::from_millis(699)).unwrap();
    assert_eq!(rt.next_deadline(), Some(Moment::from_millis(700)));
    assert_eq!(rt.app().ticks, 1);
}

#[test]
fn runtime_flash_is_an_idle_deadline_and_expires_at_140ms_without_app_tick() {
    let mut app = Timed::delayed(0);
    app.delay = None;
    let mut h = Harness::new(app, Theme::junie(), 20, 4);
    let _ = h.click(1, 0);
    assert_eq!(h.runtime().next_deadline(), Some(Moment::from_millis(140)));
    let _ = h.advance(Duration::from_millis(139));
    assert_eq!(h.runtime().next_deadline(), Some(Moment::from_millis(140)));
    let _ = h.advance(Duration::from_millis(1));
    assert_eq!(h.runtime().next_deadline(), None);
    assert_eq!(h.app().ticks, 0);
    assert_eq!(h.buffer().cell((0, 0)).unwrap().symbol(), "r");
}

#[test]
fn double_click_window_uses_elapsed_time_without_tick_events() {
    let window = Theme::junie().design.motion.double_click_ms;
    for (elapsed, expected) in [(window, 1), (window + 1, 0)] {
        let mut h = Harness::new(Timed::delayed(2200), Theme::junie(), 20, 4);
        let _ = h.click(1, 0);
        let _ = h.advance(Duration::from_millis(elapsed));
        let _ = h.click(1, 0);
        assert_eq!(h.app().doubles, expected);
        assert_eq!(h.app().ticks, 0);
    }
}

#[test]
fn feedback_deadline_preserves_submillisecond_origin() {
    let mut app = Timed::delayed(0);
    app.delay = None;
    let mut h = Harness::new(app, Theme::junie(), 20, 4);
    let _ = h.advance(Duration::from_micros(500));
    let _ = h.click(1, 0);
    let due = Moment::from_duration(Duration::from_micros(140_500));
    assert_eq!(h.runtime().next_deadline(), Some(due));
    let _ = h.advance_to(Moment::from_millis(140)).unwrap();
    assert_eq!(h.runtime().next_deadline(), Some(due));
    let _ = h.advance_to(due).unwrap();
    assert_eq!(h.runtime().next_deadline(), None);
}

#[test]
fn committed_focus_settles_before_one_due_delivery_then_retained_input() {
    let (mut rt, mut buffer) = runtime(Timed::delayed(2200));
    rt.app_mut().causes.clear();
    rt.set_focus(None);
    rt.advance_to(Moment::from_millis(2200)).unwrap();
    let retained = rt.handle(Input::Paste("ordered".into())).unwrap_err();
    let _ = rt.settle();
    assert_eq!(rt.app().causes, [UpdateCause::Settle]);
    let _ = rt.settle();
    assert_eq!(rt.app().causes, [UpdateCause::Settle, UpdateCause::Tick]);
    present(&mut rt, &mut buffer);
    let _ = rt.handle(retained.into_input()).unwrap();
    assert_eq!(rt.app().received, "ordered");
    assert_eq!(rt.app().ticks, 1);
    assert_eq!(rt.app().causes.last(), Some(&UpdateCause::Event));
}

#[cfg(feature = "crossterm")]
struct DriverProbe {
    fair: bool,
    done: bool,
    due: Moment,
    received: String,
    ticks: usize,
}
#[cfg(feature = "crossterm")]
impl App for DriverProbe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            self.due =
                cx.now()
                    .saturating_add(Duration::from_millis(if self.fair { 0 } else { 2200 }));
            cx.request_repaint_at(self.due);
        } else if cx.update_cause() == UpdateCause::Tick {
            assert!(cx.now() >= self.due);
            self.ticks += 1;
            if self.fair {
                cx.request_repaint_at(cx.now());
            } else {
                self.done = true;
            }
        }
        for intent in cx.intents(EDITOR) {
            match intent {
                Intent::Paste(text) => self.received.push_str(text),
                Intent::Key(Key {
                    code: KeyCode::Char(c),
                    ..
                }) => self.received.push(c),
                _ => {}
            }
        }
        if self.fair && self.received == "paste!" {
            assert!(self.ticks > 0);
            self.done = true;
        }
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_editor(
            EDITOR,
            ui.full(),
            Focusability::Focusable,
            StateFlags::EDITING,
        );
        let text = match (self.fair, self.done) {
            (false, false) => "IDLE_READY",
            (false, true) => "IDLE_COMPLETE",
            (true, false) => "FAIR_READY",
            (true, true) => "FAIR_COMPLETE",
        };
        ui.paint_str(ui.full(), text, ui.surface_style());
    }
    fn should_quit(&self) -> bool {
        self.done
    }
}
#[cfg(feature = "crossterm")]
fn driver_probe(fair: bool) {
    let start = std::time::Instant::now();
    junie_tui::run(
        DriverProbe {
            fair,
            done: false,
            due: Moment::ZERO,
            received: String::new(),
            ticks: 0,
        },
        Theme::junie(),
    )
    .unwrap();
    if !fair {
        assert!(start.elapsed() >= Duration::from_millis(2200));
    }
}

#[cfg(feature = "crossterm")]
#[test]
#[ignore = "real PTY fixture; launched by monotonic terminal evidence runner"]
fn idle_terminal_driver_fixture() {
    driver_probe(false);
}

#[cfg(feature = "crossterm")]
#[test]
#[ignore = "real PTY fixture; launched by monotonic terminal evidence runner"]
fn immediate_rearm_terminal_input_fairness_fixture() {
    driver_probe(true);
}

#[test]
fn independent_moment_saturates_without_precision_loss() {
    let m = Moment::from_duration(Duration::new(u64::MAX, 999_999_999));
    assert_eq!(m.saturating_add(Duration::from_nanos(1)), m);
    assert_eq!(
        Moment::from_duration(Duration::from_nanos(7)).as_duration(),
        Duration::from_nanos(7)
    );
    assert_eq!(Moment::ZERO.saturating_duration_since(m), Duration::ZERO);
    let mut rt = Runtime::new(Timed::delayed(10), Theme::junie());
    rt.advance_to(m).unwrap();
    let _ = rt.initialize();
    assert_eq!(rt.next_deadline(), Some(m));
}
