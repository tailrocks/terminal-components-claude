//! Terminal-style output with a scrollable viewport and a seven-step rail.

use core::fmt;
use std::time::Duration;

use termrock::{
    Button, ByIndex, Cx, FgStep, FrameRead, Id, Modifier, Moment, Panel, PanelKind, ProjectedText,
    Rect, Response, Role, RowUi, SplitAxis, SplitPane, SplitPaneState, StateFlags, StepState,
    Steps, StepsState, TextViewport, Ui, Variant, ViewportAction, ViewportState, id, layout,
};

use super::{Page, PageStatus, PageUpdate, frame};

const OUTPUT: Id = id!("terminal.output");
const OUTPUT_PANEL: Id = id!("terminal.output.panel");
const RAIL: Id = id!("terminal.rail");
const RAIL_PANEL: Id = id!("terminal.rail.panel");
const SEAM: Id = id!("terminal.seam");
const RUN: Id = id!("terminal.run");
const FAIL: Id = id!("terminal.fail");

const STAGES: &[&str] = &[
    "Resolve workspace",
    "Pull base image",
    "Build container",
    "Mount sources",
    "Resolve credentials",
    "Start agent",
    "Ready",
];

const DURATIONS: [u32; 7] = [10, 26, 40, 8, 18, 14, 1];

const TOTAL_TICKS: usize = 117;

const TICK_INTERVAL: Duration = Duration::from_millis(80);

#[derive(Clone, Debug)]
struct TerminalStep {
    label: &'static str,
    state: StepState,
    meta: Option<String>,
}

impl fmt::Display for TerminalStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

fn step_state(step: &TerminalStep) -> StepState {
    step.state
}

fn step_meta(step: &TerminalStep) -> Option<&str> {
    step.meta.as_deref()
}

fn step_row(step: &TerminalStep, row: &mut RowUi<'_>) {
    row.label(step.label);
}

fn step_rail() -> Steps<'static, TerminalStep, ByIndex, impl Fn(&TerminalStep, &mut RowUi<'_>)> {
    Steps::navigable(RAIL)
        .step(&step_state)
        .meta(&step_meta)
        .numbered(true)
        .row(step_row)
}

fn output_panel(meta: &str) -> Panel<'_> {
    Panel::new(OUTPUT_PANEL)
        .kind(PanelKind::Framed)
        .title("Viewport")
        .meta(meta)
}

fn rail_panel(meta: &str) -> Panel<'_> {
    Panel::new(RAIL_PANEL).title("Step rail").meta(meta)
}

fn output() -> TextViewport<'static> {
    TextViewport::new(OUTPUT)
}

fn run_button() -> Button<'static> {
    Button::new(RUN, "Run").variant(Variant::PRIMARY)
}

fn failure_button() -> Button<'static> {
    Button::new(FAIL, "Run with a failure").variant(Variant::SECONDARY)
}

fn split() -> SplitPane<'static> {
    SplitPane::new(SEAM, SplitAxis::Horizontal)
        .gap(2)
        .seam_end(1)
        .min_first(24)
        .min_second(16)
}

type Run = (String, Option<Role>, Modifier);

fn muted(text: String) -> Run {
    (text, Some(Role::Fg(FgStep::Muted)), Modifier::empty())
}

fn stage_lines(stage: usize, tick: u32) -> Vec<Vec<Run>> {
    let mut out = Vec::new();
    if tick == 0 {
        out.push(vec![(
            format!("▶ {}", STAGES[stage]),
            Some(Role::Fg(FgStep::Secondary)),
            Modifier::BOLD,
        )]);
    }
    match stage {
        1 if tick.is_multiple_of(4) => out.push(vec![muted(format!(
            "  layer {:02}/12  {:>3}%",
            (tick / 4 + 1).min(12),
            ((tick + 1) * 100 / DURATIONS[1]).min(100)
        ))]),
        2 if tick.is_multiple_of(3) => out.push(vec![
            muted("  #".to_owned()),
            (format!("{} ", tick / 3 + 1), None, Modifier::empty()),
            muted(
                match tick / 3 % 4 {
                    0 => "RUN apt-get install -y build-essential",
                    1 => "COPY rust-toolchain.toml ./",
                    2 => "RUN cargo fetch --locked",
                    _ => "RUN cargo build --release",
                }
                .to_owned(),
            ),
        ]),
        3 if tick == 2 => out.push(vec![muted(
            "  ~/src/payments-platform → /workspace/payments-platform  rw".to_owned(),
        )]),
        3 if tick == 5 => out.push(vec![muted(
            "  ~/src/shared-libs → /workspace/libs  ro".to_owned(),
        )]),
        4 if tick == 6 => out.push(vec![muted(
            "  1Password  Engineering › Anthropic · Work › credential  ok".to_owned(),
        )]),
        5 if tick == 4 => out.push(vec![muted("  claude --resume  pid 4188".to_owned())]),
        _ => {}
    }
    out
}

/// The page keeps terminal text and lifecycle data in app state; public
/// controls retain ownership of scrolling, focus, and activation semantics.
#[derive(Debug)]
pub struct TerminalPage {
    output: ProjectedText,
    line_count: usize,
    output_state: ViewportState,
    steps: Vec<TerminalStep>,
    steps_state: StepsState,
    tick: u32,
    stage: usize,
    stage_tick: u32,
    running: bool,
    fail_at: Option<usize>,
    motion_paused: bool,
    next_tick: Option<Moment>,
    split_state: SplitPaneState,
}

impl TerminalPage {
    pub fn new() -> Self {
        let mut page = Self {
            output: ProjectedText::default(),
            line_count: 0,
            output_state: ViewportState::default(),
            split_state: SplitPaneState::new(62),
            steps: Vec::new(),
            steps_state: StepsState::default(),
            tick: 0,
            stage: 0,
            stage_tick: 0,
            running: false,
            fail_at: None,
            motion_paused: false,
            next_tick: None,
        };
        page.reset(None);
        page
    }

    fn emit(&mut self, runs: Vec<Run>) {
        self.output.push_line(runs);
        self.line_count += 1;
    }

    fn reset(&mut self, fail_at: Option<usize>) {
        self.tick = 0;
        self.stage = 0;
        self.stage_tick = 0;
        self.running = true;
        self.fail_at = fail_at;
        self.next_tick = None;
        self.steps = STAGES
            .iter()
            .enumerate()
            .map(|(index, label)| TerminalStep {
                label,
                state: if index == 0 {
                    StepState::Running
                } else {
                    StepState::Queued
                },
                meta: None,
            })
            .collect();
        self.steps_state = StepsState::default();
        self.output.clear();
        self.line_count = 0;
        self.emit(vec![
            ("payments-platform ❯ ".to_owned(), None, Modifier::BOLD),
            ("jackin launch".to_owned(), None, Modifier::empty()),
        ]);
        self.output_state.set_follow(true);
        self.output_state.clear_selection();
    }

    fn advance(&mut self) {
        if !self.running {
            return;
        }
        self.tick += 1;
        let stage = self.stage;
        let stage_tick = self.stage_tick;
        for line in stage_lines(stage, stage_tick) {
            self.emit(line);
        }
        self.stage_tick += 1;
        if let Some(step) = self.steps.get_mut(stage) {
            step.meta = Some(format!("{:.1} s", self.stage_tick as f32 * 0.08));
        }
        if self.fail_at == Some(stage) && self.stage_tick >= DURATIONS[stage] / 2 {
            if let Some(step) = self.steps.get_mut(stage) {
                step.state = StepState::Failed;
                step.meta = Some("exit 1".to_owned());
            }
            for step in self.steps.iter_mut().skip(stage + 1) {
                step.state = StepState::Blocked;
            }
            self.emit(vec![
                ("✗ ".to_owned(), Some(Role::Danger), Modifier::empty()),
                (
                    format!("{} failed: network unreachable (curl: 6)", STAGES[stage]),
                    Some(Role::Danger),
                    Modifier::empty(),
                ),
            ]);
            self.running = false;
            return;
        }
        if self.stage_tick >= DURATIONS[stage] {
            if let Some(step) = self.steps.get_mut(stage) {
                step.state = StepState::Done;
            }
            self.emit(vec![
                (
                    "✓ ".to_owned(),
                    Some(Role::Fg(FgStep::Secondary)),
                    Modifier::empty(),
                ),
                muted(STAGES[stage].to_owned()),
            ]);
            self.stage += 1;
            self.stage_tick = 0;
            if self.stage >= STAGES.len() {
                self.running = false;
                self.next_tick = None;
                self.emit(vec![(
                    "payments-platform ❯ ".to_owned(),
                    None,
                    Modifier::BOLD,
                )]);
            } else {
                if self.stage == 3 {
                    // one stage is skipped when nothing needs mounting twice
                    if let Some(step) = self.steps.get_mut(self.stage) {
                        step.state = StepState::Skipped;
                        step.meta = Some("cached".to_owned());
                    }
                    self.stage += 1;
                }
                if let Some(step) = self.steps.get_mut(self.stage) {
                    step.state = StepState::Running;
                }
            }
        }
    }

    fn rail_meta(&self) -> String {
        let mut done = 0;
        let mut failed = 0;
        for step in &self.steps {
            match step.state {
                StepState::Done | StepState::Skipped => done += 1,
                StepState::Failed => failed += 1,
                _ => {}
            }
        }
        let total = self.steps.len();
        if failed > 0 {
            format!("{done} of {total} · failed")
        } else if self.running {
            format!("{done} of {total} · running")
        } else {
            format!("{done} of {total}")
        }
    }

    fn output_meta(&self) -> String {
        let scroll = self.output_state.scroll();
        if self.output_state.selection().is_some() {
            "selection · y copies".to_owned()
        } else if scroll.offset() >= scroll.max_offset() {
            format!("{} lines · following", self.line_count)
        } else {
            format!(
                "scrollback ↑{}",
                scroll.max_offset().saturating_sub(scroll.offset())
            )
        }
    }
}

impl Default for TerminalPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for TerminalPage {
    fn seek_paused(&mut self, frame: usize) {
        self.reset(None);
        for _ in 0..frame.min(TOTAL_TICKS) {
            self.advance();
        }
        self.motion_paused = true;
        self.next_tick = None;
    }

    fn title(&self) -> &'static str {
        "Terminal"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let mut status = None;
        let output = output().update_projected(cx, &mut self.output_state, &self.output);
        if let Some(action) = output.action_ref() {
            match action {
                ViewportAction::Copy(text) => {
                    let lines = text.lines().count().max(1);
                    status = Some(PageStatus(format!(
                        "Copied {lines} line{}",
                        if lines == 1 { "" } else { "s" }
                    )));
                }
                ViewportAction::FollowChanged(on) => {
                    status = Some(PageStatus(
                        if *on {
                            "Following the tail"
                        } else {
                            "Paused at the scrollback position"
                        }
                        .to_owned(),
                    ));
                }
                ViewportAction::SelectionChanged => {}
            }
        }
        response |= output.erase();

        if self.running && !self.motion_paused {
            let now = cx.now();
            let deadline = *self
                .next_tick
                .get_or_insert_with(|| now.saturating_add(TICK_INTERVAL));
            if cx.update_cause() == termrock::UpdateCause::Tick && now >= deadline {
                self.advance();
                if self.running {
                    self.next_tick = Some(now.saturating_add(TICK_INTERVAL));
                }
                response = response.repaint();
            }
            if cx.top_layer() == termrock::LayerId::PAGE
                && let Some(deadline) = self.next_tick
            {
                cx.request_repaint_at(deadline);
            }
        }

        response |= step_rail()
            .update(cx, &mut self.steps_state, &self.steps)
            .erase();

        let run = run_button().update(cx);
        if run.activated() {
            self.reset(None);
        }
        response |= run.erase();
        let fail = failure_button().update(cx);
        if fail.activated() {
            self.reset(Some(1));
        }
        response |= fail.erase();
        response |= split().update(cx, &mut self.split_state).erase();
        // Both phases build the same panels (§13); the update pass
        // only needs the construction to stay the single source of the props.
        let output_meta = self.output_meta();
        let rail_meta = self.rail_meta();
        let _ = split();
        let _ = output_panel(&output_meta);
        let _ = rail_panel(&rail_meta);
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Viewport with scrollback, selection and copy · drag the seam · step rail reports a job",
            |ui, body| {
                split().draw(ui, body, &self.split_state, |ui, left, right| {
                    // A collapsed split keeps the rail: the viewport drops
                    // and the rail takes the whole body.
                    let (output_rect, rail_rect) = if right.is_empty() {
                        (None, left)
                    } else {
                        (Some(left), right)
                    };
                    if let Some(left) = output_rect {
                        let output_meta = self.output_meta();
                        output_panel(&output_meta).draw(ui, left, |ui, inner| {
                            output().draw_projected(ui, inner, &self.output_state, &self.output);
                        });
                    }

                    let rail_meta = self.rail_meta();
                    rail_panel(&rail_meta).draw(ui, rail_rect, |ui, inner| {
                        let rail_area = Rect {
                            height: inner.height.saturating_sub(3),
                            ..inner
                        };
                        step_rail().draw(ui, rail_area, &self.steps_state, &self.steps);

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
                });
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(OUTPUT).contains(StateFlags::FOCUSED) {
            &[
                ("↑ ↓", "Scroll"),
                ("Home End", "Oldest / live"),
                ("f", "Follow"),
                ("drag", "Select"),
                ("y", "Copy"),
                ("Esc", "Clear"),
            ]
        } else if ui.state(RAIL).contains(StateFlags::FOCUSED) {
            &[("↑ ↓", "Move"), ("wheel", "Scroll")]
        } else {
            &[("Enter", "Activate"), ("drag ┃", "Resize")]
        }
    }
}

#[cfg(test)]
mod narrow_rail_tests {
    use super::*;
    use termrock::{App, Theme};
    use termrock_test_support::Harness;

    struct PageApp(TerminalPage);

    impl App for PageApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.0.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let full = ui.full();
            self.0.draw(ui, full);
        }
    }

    /// At every standard size the rail inner is narrower than 60 cells, where
    /// the retired historical painter overwrote the rail with clipped static
    /// strings and a duplicate button row. The composed rail must come from
    /// Steps/Spinner/Button state instead: lifecycle words visible where they
    /// fit, one button row, and the staged run advancing through the same
    /// components.
    #[test]
    fn narrow_rail_composes_owned_steps_spinner_and_buttons() {
        let mut h = Harness::new(PageApp(TerminalPage::new()), Theme::junie(), 80, 24);
        h.draw();
        let boot = h.text();
        assert!(
            boot.contains("01 Reso"),
            "running step label renders (ellipsis-truncated by the rail)"
        );
        assert!(
            boot.contains("queued"),
            "queued lifecycle word fits the narrow rail"
        );
        assert!(boot.contains("⠋"), "owned spinner animates the running row");
        assert!(
            boot.contains("0 of 7"),
            "rail panel counts no finished stages at boot"
        );
        assert_eq!(
            boot.matches("Run with a fail").count(),
            1,
            "exactly one failure button row (no historical duplicate)"
        );

        // One tick pins the first elapsed meta on the running row.
        let _ = h.advance(TICK_INTERVAL);
        h.draw();
        assert!(
            h.text().contains("0.1 s"),
            "running step shows elapsed row meta after a tick"
        );

        // Riding out the live run reaches the deterministic end state.
        for _ in 0..TOTAL_TICKS {
            let _ = h.advance(TICK_INTERVAL);
        }
        h.draw();
        let done = h.text();
        assert!(
            done.contains("7 of 7"),
            "rail panel counts every finished stage"
        );
        assert!(
            !done.contains("⠋") && !done.contains("⠏"),
            "spinner rests when nothing runs"
        );
        assert!(
            done.contains("✓ Ready"),
            "the last stage completes through the rail"
        );
    }

    #[test]
    fn paused_seek_reaches_mid_and_end_states() {
        let mut page = TerminalPage::new();
        page.seek_paused(60);
        assert_eq!(page.stage, 2);
        assert_eq!(page.stage_tick, 24);
        assert_eq!(page.line_count, 21);
        assert_eq!(page.rail_meta(), "2 of 7 · running");
        page.seek_paused(usize::MAX);
        assert!(!page.running);
        assert_eq!(page.rail_meta(), "7 of 7");
        assert_eq!(page.line_count, 37);
    }

    #[test]
    fn failure_action_fails_stage_two_and_blocks_the_rest() {
        let mut page = TerminalPage::new();
        page.reset(Some(1));
        for _ in 0..30 {
            page.advance();
        }
        assert!(!page.running);
        assert_eq!(page.steps[0].state, StepState::Done);
        assert_eq!(page.steps[1].state, StepState::Failed);
        assert_eq!(page.steps[1].meta.as_deref(), Some("exit 1"));
        assert!(
            page.steps[2..]
                .iter()
                .all(|step| step.state == StepState::Blocked)
        );
        assert_eq!(page.rail_meta(), "1 of 7 · failed");
        page.reset(None);
        assert!(page.running);
        assert_eq!(page.steps[0].state, StepState::Running);
        assert_eq!(page.rail_meta(), "0 of 7 · running");
    }
}
