use core::fmt;
use std::time::Duration;

use termrock::{
    Button, Cx, FrameRead, Id, Moment, Panel, PanelKind, Rect, Response, Spinner, StateFlags,
    Status, StepState, Steps, StepsState, TextArea, TextAreaState, Ui, UpdateCause, Variant,
    layout,
};

use showcase_data::log_lines;

use super::{Page, PageUpdate, frame, lines};

const OUTPUT: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.output");
const OUTPUT_PANEL: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.output.panel");
const RAIL: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.rail");
const RAIL_PANEL: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.rail.panel");
const STEP_SPINNER: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.step.spinner");
const RUN: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.run");
const FAIL: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.fail");

/// The ticks a launch stage takes.
const STAGE_TICKS: u32 = 6;
const TICK_INTERVAL: Duration = Duration::from_millis(16);

fn future_deadline(now: Moment) -> Option<Moment> {
    let deadline = now.saturating_add(TICK_INTERVAL);
    (deadline > now).then_some(deadline)
}

/// The rows the page's footer keeps below the panes.
const FOOTER_ROWS: u16 = 2;

const STAGES: &[(&str, StepState, &str)] = &[
    ("01 Resolve workspace", StepState::Running, "0.8 s"),
    ("02 Pull base image", StepState::Queued, "2.1 s"),
    ("03 Build container", StepState::Queued, "3.2 s"),
    ("04 Mount sources", StepState::Queued, "cached"),
    ("05 Resolve credentials", StepState::Queued, "1.4 s"),
    ("06 Start agent", StepState::Queued, "1.1 s"),
    ("07 Ready", StepState::Queued, "0.1 s"),
];

#[derive(Clone, Copy, Debug)]
struct TerminalStep {
    label: &'static str,
    state: StepState,
    duration: &'static str,
}

impl fmt::Display for TerminalStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

fn step_state(step: &TerminalStep) -> StepState {
    step.state
}

fn step_rail() -> Steps<'static, TerminalStep> {
    Steps::navigable(RAIL)
        .step(&step_state)
        .metadata(&|step: &TerminalStep| Some(step.duration))
}

fn step_spinner() -> Spinner<'static> {
    Spinner::new(STEP_SPINNER).frame(0)
}

fn output_panel() -> Panel<'static> {
    Panel::new(OUTPUT_PANEL)
        .kind(PanelKind::Framed)
        .title("Viewport")
}

fn rail_panel(meta: &str) -> Panel<'_> {
    Panel::new(RAIL_PANEL).title("Step rail").meta(meta)
}

fn output() -> TextArea<'static> {
    TextArea::new(OUTPUT, 12)
        .read_only(true)
        .status(Status::Ready)
}

fn run_button() -> Button<'static> {
    Button::new(RUN, "Run").variant(Variant::SECONDARY)
}

fn failure_button() -> Button<'static> {
    Button::new(FAIL, "Run with a failure").variant(Variant::SECONDARY)
}

fn panes(area: Rect) -> (Option<Rect>, Rect) {
    let usable = area.width.saturating_sub(2);
    if usable < 58 {
        return (None, area);
    }
    let left_width = area.width.saturating_mul(62).checked_div(100).unwrap_or(0);
    let right_x = area.x.saturating_add(left_width).saturating_add(2);
    (
        Some(Rect {
            width: left_width,
            ..area
        }),
        Rect {
            x: right_x,
            width: area.right().saturating_sub(right_x),
            ..area
        },
    )
}

#[derive(Debug)]
pub struct TerminalPage {
    output: String,
    output_state: TextAreaState,
    steps: Vec<TerminalStep>,
    steps_state: StepsState,
    running: bool,
    tick: u32,
    next_deadline: Option<Moment>,
}

impl TerminalPage {
    pub fn new() -> Self {
        Self {
            output: log_lines(120).join("\n"),
            output_state: TextAreaState::default(),
            steps: STAGES
                .iter()
                .map(|(label, state, duration)| TerminalStep {
                    label,
                    state: *state,
                    duration,
                })
                .collect(),
            steps_state: StepsState::default(),
            running: true,
            tick: 0,
            next_deadline: None,
        }
    }

    fn reset(&mut self, failed: bool) {
        self.steps = STAGES
            .iter()
            .enumerate()
            .map(|(index, (label, _, duration))| TerminalStep {
                label,
                duration,
                state: if failed && index > 0 {
                    StepState::Blocked
                } else if failed && index == 0 {
                    StepState::Failed
                } else if index == 0 {
                    StepState::Running
                } else {
                    StepState::Queued
                },
            })
            .collect();
        self.steps_state = StepsState::default();
        self.running = !failed;
        self.tick = 0;
        self.next_deadline = None;
    }

    /// The launch advances one stage at a time. The page settles when the
    /// last stage is done.
    fn advance(&mut self) {
        if !self.running {
            return;
        }
        self.tick = self.tick.saturating_add(1);
        let current = (self.tick / STAGE_TICKS) as usize;
        for (index, step) in self.steps.iter_mut().enumerate() {
            step.state = match index.cmp(&current) {
                std::cmp::Ordering::Less => StepState::Done,
                std::cmp::Ordering::Equal => StepState::Running,
                std::cmp::Ordering::Greater => StepState::Queued,
            };
        }
        if current + 1 >= self.steps.len() {
            self.running = false;
            for step in &mut self.steps {
                step.state = StepState::Done;
            }
        }
    }

    /// The stages the launch finished.
    fn settled(&self) -> usize {
        self.steps
            .iter()
            .filter(|step| step.state == StepState::Done)
            .count()
    }
}

impl Default for TerminalPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for TerminalPage {
    fn title(&self) -> &'static str {
        "Terminal"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let output = output().update(cx, &mut self.output_state, &mut self.output);
        response |= output.erase();
        response |= step_rail()
            .update(cx, &mut self.steps_state, &self.steps)
            .erase();

        let run = run_button().update(cx);
        if run.activated() {
            self.reset(false);
        }
        response |= run.erase();
        let fail = failure_button().update(cx);
        if fail.activated() {
            self.reset(true);
        }
        response |= fail.erase();

        let _ = output_panel();
        let _ = step_spinner();
        // Keep the simulation on its own clock. Only one due Tick moves it;
        // ordinary page updates must not consume a stage interval.
        if self.running {
            let now = cx.now();
            if self.next_deadline.is_none() {
                self.next_deadline = future_deadline(now);
            }
            if self
                .next_deadline
                .is_some_and(|deadline| cx.update_cause() == UpdateCause::Tick && now >= deadline)
            {
                self.advance();
                self.next_deadline = if self.running {
                    future_deadline(now)
                } else {
                    None
                };
                response = response.repaint();
            }
            if let Some(deadline) = self.next_deadline {
                cx.request_repaint_at(deadline);
            }
        } else {
            self.next_deadline = None;
        }
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Viewport with scrollback, selection and copy · drag the seam · step rail reports a job",
            |ui, body| {
                super::section_strip(ui, body);
                let content = Rect {
                    y: body.y.saturating_add(3),
                    height: body.height.saturating_sub(3),
                    ..body
                };
                // The footer keeps its own rows below the panes, so it never
                // lands on a panel border.
                let panes_area = Rect {
                    height: content.height.saturating_sub(FOOTER_ROWS),
                    ..content
                };
                let (left, right) = panes(panes_area);
                let wide = left.is_some();
                if let Some(left) = left {
                    output_panel().draw(ui, left, |ui, inner| {
                        output()
                            .value(&self.output)
                            .draw(ui, inner, &self.output_state);
                    });
                }

                let count = if self.running {
                    format!("{} of 7 · running", self.settled())
                } else {
                    format!("{} of 7", self.settled())
                };
                rail_panel(&count).draw(ui, right, |ui, inner| {
                    let rail_area = Rect {
                        height: inner.height.saturating_sub(3),
                        ..inner
                    };
                    step_rail().draw(ui, rail_area, &self.steps_state, &self.steps);

                    let _ = step_spinner().draw(
                        ui,
                        Rect {
                            x: inner.x.saturating_add(1),
                            y: inner.y,
                            width: 1,
                            height: 1,
                        },
                    );

                    let run = run_button();
                    let fail = failure_button();
                    let widths = [
                        run.measure(ui, termrock::Constraints::loose(inner.width, 1))
                            .preferred
                            .0,
                        fail.measure(ui, termrock::Constraints::loose(inner.width, 1))
                            .preferred
                            .0,
                    ];
                    let row = Rect {
                        y: inner.bottom().saturating_sub(1),
                        height: 1,
                        ..inner
                    };
                    let rects = layout::action_row(row, &widths, 2, termrock::RowAlign::Start);
                    if let Some(rect) = rects.first().copied() {
                        run.draw(ui, rect);
                    }
                    if let Some(rect) = rects.get(1).copied() {
                        fail.draw(ui, rect);
                    }
                });

                if wide {
                    lines(
                        ui,
                        Rect {
                            y: panes_area.bottom(),
                            height: FOOTER_ROWS,
                            ..content
                        },
                        &[
                            "Output remains borrowed by the public control.",
                            "status: ready",
                        ],
                    );
                }
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(OUTPUT).contains(StateFlags::FOCUSED) {
            // The viewport is a read-only control: it scrolls and its caret
            // walks the log. It has no follow, copy or clear command.
            &[("↑ ↓", "Scroll"), ("Home End", "Oldest / live")]
        } else if ui.state(RAIL).contains(StateFlags::FOCUSED) {
            &[("↑ ↓", "Move")]
        } else {
            &[("Enter", "Activate")]
        }
    }
}

#[cfg(test)]
mod terminal_clock_contract {
    use super::*;
    use termrock::{App, Axis, Cx, ItemKey, KeyCode, Part, PartRef, Response, Theme, Ui};
    use termrock_test_support::Harness;

    const INITIAL_STATES: [StepState; 7] = [
        StepState::Running,
        StepState::Queued,
        StepState::Queued,
        StepState::Queued,
        StepState::Queued,
        StepState::Queued,
        StepState::Queued,
    ];

    struct TerminalHost {
        page: TerminalPage,
    }

    impl App for TerminalHost {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.page.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.page.draw(ui, ui.full());
        }
    }

    fn harness() -> Harness<TerminalHost> {
        Harness::new(
            TerminalHost {
                page: TerminalPage::new(),
            },
            Theme::junie(),
            120,
            40,
        )
    }

    fn states(harness: &Harness<TerminalHost>) -> Vec<StepState> {
        harness
            .app()
            .page
            .steps
            .iter()
            .map(|step| step.state)
            .collect()
    }

    fn assert_initial_progress(harness: &Harness<TerminalHost>) {
        assert_eq!(harness.app().page.tick, 0);
        assert_eq!(states(harness), INITIAL_STATES);
    }

    fn assert_component_bounds(harness: &Harness<TerminalHost>) {
        let frame = *harness.buffer().area();
        for id in [OUTPUT, RAIL, RUN, FAIL] {
            let area = harness
                .area_of(id)
                .unwrap_or_else(|| panic!("missing {id:?}"));
            assert!(area.width > 0 && area.height > 0, "empty area for {id:?}");
            assert!(
                area.x >= frame.x && area.y >= frame.y,
                "area starts outside frame: {id:?} {area:?}"
            );
            assert!(
                area.right() <= frame.right(),
                "area ends outside frame: {id:?} {area:?}"
            );
            assert!(
                area.bottom() <= frame.bottom(),
                "area ends outside frame: {id:?} {area:?}"
            );
        }
        assert!(
            harness
                .area_of_part(RAIL, PartRef::item(Part::ROW, ItemKey::index(0)))
                .is_some(),
            "the live rail exposes its first task row"
        );
    }

    #[test]
    fn bootstrap_and_draw_settle_do_not_advance_task_progress() {
        let harness = harness();

        assert_eq!(harness.now(), termrock::Moment::ZERO);
        assert_initial_progress(&harness);
        assert_eq!(
            harness.app().page.next_deadline,
            Some(Moment::from_millis(16))
        );
        assert!(harness.text().contains("0 of 7 · running"));
        assert_component_bounds(&harness);
    }

    #[test]
    fn ordinary_event_updates_at_one_moment_do_not_advance_task_progress() {
        let mut harness = harness();
        let before_tick = harness.app().page.tick;
        let before_states = states(&harness);
        let before_now = harness.now();

        let _ = harness.key(KeyCode::Char('x'));

        assert_eq!(harness.now(), before_now);
        assert_eq!(harness.app().page.tick, before_tick);
        assert_eq!(states(&harness), before_states);
    }

    #[test]
    fn only_due_ticks_advance_and_each_tick_moves_at_most_one_step() {
        let mut harness = harness();
        let output = harness.app().page.output.clone();

        let _ = harness
            .advance_to(Moment::from_millis(15))
            .expect("clock moves forward");
        let _ = harness.tick();
        assert_eq!(harness.now(), Moment::from_millis(15));
        assert_initial_progress(&harness);
        assert_eq!(
            harness.app().page.next_deadline,
            Some(Moment::from_millis(16))
        );

        let _ = harness
            .advance_to(Moment::from_millis(16))
            .expect("the first deadline is due");
        assert_eq!(harness.app().page.tick, 1);
        assert_eq!(states(&harness), INITIAL_STATES);
        assert_eq!(
            harness.app().page.next_deadline,
            Some(Moment::from_millis(32))
        );

        harness.draw();
        let _ = harness.tick();
        assert_eq!(
            harness.app().page.tick,
            1,
            "draw and an early Tick do not catch up"
        );

        for tick in 2..=6 {
            let _ = harness
                .advance_to(Moment::from_millis(u64::from(tick) * 16))
                .expect("the next stage deadline is due");
            assert_eq!(harness.app().page.tick, tick);
        }
        assert_eq!(
            states(&harness),
            [
                StepState::Done,
                StepState::Running,
                StepState::Queued,
                StepState::Queued,
                StepState::Queued,
                StepState::Queued,
                StepState::Queued,
            ]
        );
        assert!(harness.text().contains("1 of 7 · running"));
        assert_eq!(harness.app().page.output, output);
        assert_eq!(
            harness.app().page.next_deadline,
            Some(Moment::from_millis(112))
        );
    }

    #[test]
    fn overdue_deadline_advances_once_without_a_catch_up_burst() {
        let mut harness = harness();

        let _ = harness
            .advance_to(Moment::from_millis(200))
            .expect("clock moves forward past the first deadline");

        assert_eq!(harness.app().page.tick, 1);
        assert_eq!(
            harness.app().page.next_deadline,
            Some(Moment::from_millis(216))
        );
        assert_eq!(states(&harness), INITIAL_STATES);

        let _ = harness
            .advance_to(Moment::from_millis(216))
            .expect("the rearmed deadline is due");
        assert_eq!(harness.app().page.tick, 2);
        assert_eq!(
            harness.app().page.next_deadline,
            Some(Moment::from_millis(232))
        );
    }

    #[test]
    fn saturated_clock_advances_once_without_rearming_at_max() {
        let mut harness = harness();
        let max = Moment::from_duration(std::time::Duration::MAX);
        let output = harness.app().page.output.clone();

        let _ = harness.advance_to(max).expect("clock moves to MAX");

        assert_eq!(harness.now(), max);
        assert_eq!(harness.app().page.tick, 1);
        assert_eq!(states(&harness), INITIAL_STATES);
        assert_eq!(harness.app().page.next_deadline, None);

        let settled_states = states(&harness);
        let _ = harness.tick();
        let _ = harness
            .advance_to(max)
            .expect("same-Moment advance is valid");
        let _ = harness.tick();

        assert_eq!(harness.app().page.tick, 1);
        assert_eq!(states(&harness), settled_states);
        assert_eq!(harness.app().page.output, output);
        assert_eq!(harness.app().page.next_deadline, None);
    }

    #[test]
    fn run_at_max_stays_running_without_a_non_future_deadline() {
        let mut harness = harness();
        let max = Moment::from_duration(std::time::Duration::MAX);
        let _ = harness.advance_to(max).expect("clock moves to MAX");

        let _ = harness.click_id(FAIL);
        assert!(!harness.app().page.running);
        assert_eq!(harness.app().page.tick, 0);
        assert_eq!(harness.app().page.next_deadline, None);

        let _ = harness.click_id(RUN);
        assert!(harness.app().page.running);
        assert_eq!(harness.app().page.tick, 0);
        assert_eq!(states(&harness), INITIAL_STATES);
        assert_eq!(harness.app().page.next_deadline, None);

        let _ = harness.key(KeyCode::Char('x'));
        let _ = harness.tick();
        let _ = harness
            .advance_to(max)
            .expect("same-Moment advance is valid");

        assert!(harness.app().page.running);
        assert_eq!(harness.app().page.tick, 0);
        assert_eq!(states(&harness), INITIAL_STATES);
        assert_eq!(harness.app().page.next_deadline, None);
    }

    #[test]
    fn near_max_clock_keeps_one_strictly_future_deadline() {
        let mut harness = harness();
        let max = Moment::from_duration(std::time::Duration::MAX);
        let near_max =
            Moment::from_duration(std::time::Duration::MAX - std::time::Duration::from_millis(8));

        let _ = harness
            .advance_to(near_max)
            .expect("clock moves to a representable pre-MAX moment");
        assert_eq!(harness.now(), near_max);
        assert_eq!(harness.app().page.tick, 1);
        assert_eq!(harness.app().page.next_deadline, Some(max));

        let _ = harness
            .advance_to(max)
            .expect("the future MAX deadline is due");
        assert_eq!(harness.app().page.tick, 2);
        assert_eq!(harness.app().page.next_deadline, None);

        let _ = harness.tick();
        let _ = harness
            .advance_to(max)
            .expect("same-Moment advance is valid");
        assert_eq!(harness.app().page.tick, 2);
        assert_eq!(harness.app().page.next_deadline, None);
    }

    #[test]
    fn input_can_move_the_rail_and_scroll_output_without_moving_its_clock() {
        let mut harness = harness();
        let output = harness.app().page.output.clone();
        let deadline = harness.app().page.next_deadline;
        let output_area = harness
            .area_of(OUTPUT)
            .expect("output is visible at this size");
        let scroll_before = harness.app().page.output_state.scroll().offset();
        assert_component_bounds(&harness);

        for _ in 0..8 {
            let _ = harness.key(KeyCode::Char('x'));
        }
        let _ = harness.click_part(RAIL, PartRef::item(Part::ROW, ItemKey::index(3)));
        assert_eq!(
            harness.app().page.steps_state.cursor(),
            Some(ItemKey::index(3)),
            "pointer navigation remains independent from task progress"
        );
        let _ = harness.wheel(
            Axis::V,
            1,
            output_area.x.saturating_add(output_area.width / 2),
            output_area.y.saturating_add(output_area.height / 2),
        );
        assert!(harness.app().page.output_state.scroll().offset() > scroll_before);
        let _ = harness.resize(121, 40);
        assert_component_bounds(&harness);

        assert_eq!(harness.now(), Moment::ZERO);
        assert_eq!(harness.app().page.tick, 0);
        assert_eq!(states(&harness), INITIAL_STATES);
        assert_eq!(harness.app().page.next_deadline, deadline);
        assert_eq!(harness.app().page.output, output);
    }

    #[test]
    fn fail_stops_the_clock_and_run_rearms_without_same_event_progress() {
        let mut harness = harness();
        let _ = harness
            .advance_to(Moment::from_millis(16))
            .expect("first task tick is due");
        assert_eq!(harness.app().page.tick, 1);

        let _ = harness.click_id(FAIL);
        assert!(!harness.app().page.running);
        assert_eq!(harness.app().page.tick, 0);
        assert_eq!(
            states(&harness),
            [
                StepState::Failed,
                StepState::Blocked,
                StepState::Blocked,
                StepState::Blocked,
                StepState::Blocked,
                StepState::Blocked,
                StepState::Blocked,
            ]
        );
        assert_eq!(harness.app().page.next_deadline, None);

        let _ = harness.tick();
        let _ = harness
            .advance_to(Moment::from_millis(32))
            .expect("an already scheduled repaint may still be delivered");
        assert!(!harness.app().page.running);
        assert_eq!(harness.app().page.tick, 0);
        assert_eq!(harness.app().page.next_deadline, None);

        let _ = harness.click_id(RUN);
        assert!(harness.app().page.running);
        assert_initial_progress(&harness);
        assert_eq!(
            harness.app().page.next_deadline,
            Some(Moment::from_millis(48))
        );
        let _ = harness.tick();
        assert_initial_progress(&harness);
        assert_eq!(
            harness.app().page.next_deadline,
            Some(Moment::from_millis(48))
        );

        let _ = harness
            .advance_to(Moment::from_millis(48))
            .expect("Run rearmed the page clock");
        assert_eq!(harness.app().page.tick, 1);
    }

    #[test]
    fn completion_stops_the_page_deadline_and_repeated_ticks() {
        let mut harness = harness();
        for tick in 1_u32..=36 {
            let _ = harness
                .advance_to(Moment::from_millis(u64::from(tick) * 16))
                .expect("the task deadline is due");
        }

        assert_eq!(harness.app().page.tick, 36);
        assert!(!harness.app().page.running);
        assert_eq!(states(&harness), [StepState::Done; 7]);
        assert_eq!(harness.app().page.next_deadline, None);
        let _ = harness.tick();
        assert_eq!(harness.app().page.tick, 36);
    }
}

// Page rect inside the 160×50 shell: sidebar span 24, inspector closed.

// Finished 120×40 playground, column-relative to the page rect. Captured from
// the truecolor oracle once the launch reaches "✓ Ready".

// Finished 80×24 page, column-relative to the page rect. Truecolor cells from
// the oracle once the launch reaches "✓ Ready". paint_str merges modifiers, so
// every run clears the modifiers it does not set.

// Finished 72×20 page, column-relative to the page rect. Truecolor cells from
// the oracle once the launch reaches "✓ Ready". paint_str merges modifiers, so
// every run clears the modifiers it does not set.

// Finished 160×50 page, column-relative to the page rect. Truecolor cells from
// the oracle once the launch reaches "✓ Ready". paint_str merges modifiers, so
// every run clears the modifiers it does not set. Bits: 1 bold, 2 italic, 4 underline.
