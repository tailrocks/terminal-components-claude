use super::*;
use crate::core::event::Key;
use crate::ui::ctx::RenderCtx;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

struct Demo {
    shell: Shell,
    page: u64,
    buttons: Vec<&'static str>,
    primary: Option<&'static str>,
    blurs: Vec<FocusChange>,
}

impl Demo {
    fn new(buttons: &[&'static str]) -> Self {
        Self {
            shell: Shell::new(),
            page: 1,
            buttons: buttons.to_vec(),
            primary: None,
            blurs: vec![],
        }
    }
    fn focused(&self) -> Option<WidgetId> {
        self.shell.focus.current()
    }
}

impl App for Demo {
    type Status = ();
    fn shell(&self) -> &Shell {
        &self.shell
    }
    fn shell_mut(&mut self) -> &mut Shell {
        &mut self.shell
    }
    fn theme(&self) -> Theme {
        Theme::junie()
    }
    fn too_small(&self) -> TooSmall<'_> {
        TooSmall::new("demo").min(20, 3)
    }
    fn now_ms(&self) -> u64 {
        0
    }
    fn tick(&self) -> u64 {
        0
    }
    fn tick_interval(&self) -> Duration {
        Duration::from_millis(100)
    }
    fn draw(&mut self, area: Rect, _buf: &mut Buffer, ctx: &mut RenderCtx) {
        for (i, b) in self.buttons.iter().enumerate() {
            let r = Rect::new(area.x + i as u16 * 4, area.y, 3, 1);
            ctx.control(WidgetId::of(b), r, false);
        }
    }
    fn on_input(&mut self, input: Input) -> Outcome {
        match input {
            Input::Key(k) if k.code == KeyCode::Tab => {
                self.shell.focus_next();
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }
    fn on_blur(&mut self, change: FocusChange) -> Outcome {
        self.blurs.push(change);
        Outcome::Changed
    }
    fn page_key(&self) -> PageKey {
        PageKey(self.page)
    }
    fn primary_focus(&self) -> Option<WidgetId> {
        self.primary.map(WidgetId::of)
    }
}

fn draw_at(app: &mut Demo, w: u16, h: u16) {
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| Application::render(app, f)).unwrap();
}

fn draw(app: &mut Demo) {
    draw_at(app, 40, 5);
}

#[test]
fn a_new_page_focuses_its_primary_control() {
    let mut app = Demo::new(&["a", "b", "c"]);
    app.primary = Some("b");
    draw(&mut app);
    assert_eq!(app.focused(), Some(WidgetId::of("b")));
}

#[test]
fn without_a_primary_control_a_new_page_focuses_its_first() {
    let mut app = Demo::new(&["a", "b", "c"]);
    draw(&mut app);
    assert_eq!(app.focused(), Some(WidgetId::of("a")));
}

#[test]
fn a_control_that_vanishes_hands_focus_to_its_nearest_neighbour() {
    let mut app = Demo::new(&["a", "b", "c", "d"]);
    app.primary = Some("a");
    draw(&mut app);
    app.shell.focus.focus(WidgetId::of("c"));
    draw(&mut app);
    app.buttons.retain(|b| *b != "c");
    draw(&mut app);
    assert_eq!(app.focused(), Some(WidgetId::of("b")));
}

#[test]
fn a_page_change_with_lost_focus_goes_to_the_new_pages_primary() {
    let mut app = Demo::new(&["a", "b", "c"]);
    draw(&mut app);
    app.shell.focus.focus(WidgetId::of("c"));
    draw(&mut app);
    app.page = 2;
    app.buttons = vec!["x", "y", "z"];
    app.primary = Some("z");
    draw(&mut app);
    assert_eq!(app.focused(), Some(WidgetId::of("z")));
}

#[test]
fn every_focus_change_reaches_on_blur() {
    let mut app = Demo::new(&["a", "b"]);
    draw(&mut app);
    let out = Application::handle(
        &mut app,
        Input::Key(Key {
            code: KeyCode::Tab,
            mods: KeyModifiers::NONE,
        }),
    );
    assert_eq!(out, Outcome::Changed);
    assert_eq!(
        app.blurs,
        vec![FocusChange {
            from: WidgetId::of("a"),
            to: Some(WidgetId::of("b")),
        }]
    );
}

#[test]
fn a_resize_records_the_size() {
    let mut app = Demo::new(&["a"]);
    let _ = Application::handle(&mut app, Input::Resize(33, 7));
    assert_eq!(app.shell.size, (33, 7));
}

#[test]
fn the_too_small_screen_draws_no_page_and_keeps_focus() {
    let mut app = Demo::new(&["a", "b"]);
    draw(&mut app);
    app.shell.focus.focus(WidgetId::of("b"));
    draw(&mut app);
    draw_at(&mut app, 10, 2);
    assert!(app.shell.too_small);
    assert_eq!(app.focused(), Some(WidgetId::of("b")));
}

struct Clocked {
    shell: Shell,
    clock: crate::clock::SimClock,
    job_fired: bool,
    pastes: Vec<String>,
    steps: bool,
}

impl Clocked {
    fn new(motion: Motion, steps: bool) -> Self {
        let mut shell = Shell::new();
        shell.motion = motion;

        shell.clock_steps = true;
        let mut clock = crate::clock::SimClock::new(0, 0);
        clock.running = motion != Motion::Paused;
        Self {
            shell,
            clock,
            job_fired: false,
            pastes: vec![],
            steps,
        }
    }
    fn on_tick(&mut self) -> Outcome {
        self.clock.advance(500);
        if self.clock.now_ms >= 1_100 && !self.job_fired {
            self.job_fired = true;
            return Outcome::Changed;
        }
        Outcome::Ignored
    }
}

impl App for Clocked {
    type Status = ();
    fn shell(&self) -> &Shell {
        &self.shell
    }
    fn shell_mut(&mut self) -> &mut Shell {
        &mut self.shell
    }
    fn theme(&self) -> Theme {
        Theme::junie()
    }
    fn too_small(&self) -> TooSmall<'_> {
        TooSmall::new("demo").min(20, 3)
    }
    fn now_ms(&self) -> u64 {
        self.clock.now_ms as u64
    }
    fn tick(&self) -> u64 {
        0
    }
    fn tick_interval(&self) -> Duration {
        Duration::from_millis(500)
    }
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer, _ctx: &mut RenderCtx) {}
    fn on_input(&mut self, input: Input) -> Outcome {
        match input {
            Input::Tick => self.on_tick(),
            Input::Paste(text) => {
                self.pastes.push(text);
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }
    fn advance_clock(&mut self, ms: u64) -> Option<Outcome> {
        self.steps
            .then(|| crate::clock::run_step(self, |a| &mut a.clock, ms, Clocked::on_tick))
    }
}

#[test]
fn a_clock_step_runs_a_paused_apps_jobs_and_leaves_its_clock_stopped() {
    let mut app = Clocked::new(Motion::Paused, true);

    for _ in 0..5 {
        let _ = Application::handle(&mut app, Input::Tick);
    }
    assert!(!app.job_fired);
    let out = Application::handle(
        &mut app,
        Input::Paste(crate::clock::ClockStep::request(1_100)),
    );
    assert_eq!(out, Outcome::Changed);
    assert!(app.job_fired, "the step fired the job due at 1,100 ms");
    assert_eq!(app.clock.now_ms, 1_500);
    assert!(app.pastes.is_empty(), "the request is not pasted");
    let _ = Application::handle(&mut app, Input::Tick);
    assert_eq!(app.clock.now_ms, 1_500, "stopped again after the step");
}

#[test]
fn a_clock_step_request_is_an_ordinary_paste_unless_paused_and_supported() {
    let request = crate::clock::ClockStep::request(1_100);

    let mut full = Clocked::new(Motion::Full, true);
    let _ = Application::handle(&mut full, Input::Paste(request.clone()));
    assert_eq!(full.pastes, vec![request.clone()]);
    assert_eq!(full.clock.now_ms, 0);

    let mut plain = Clocked::new(Motion::Paused, false);
    let _ = Application::handle(&mut plain, Input::Paste(request.clone()));
    assert_eq!(plain.pastes, vec![request]);
    assert!(!plain.job_fired);
}

#[test]
fn without_the_test_opt_in_a_clock_step_request_is_delivered_unchanged() {
    let request = crate::clock::ClockStep::request(1_100);
    let mut app = Clocked::new(Motion::Paused, true);
    app.shell.clock_steps = false;
    let _ = Application::handle(&mut app, Input::Paste(request.clone()));
    assert_eq!(app.pastes, vec![request]);
    assert!(!app.job_fired);
    assert_eq!(app.clock.now_ms, 0);

    assert_eq!(
        Shell::<()>::new().clock_steps,
        std::env::var_os(crate::clock::ClockStep::ENV).is_some_and(|v| v == "1")
    );
}
