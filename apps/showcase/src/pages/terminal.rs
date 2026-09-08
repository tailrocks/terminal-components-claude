//! Terminal-style output with a scrollable viewport and a seven-step rail.

use core::fmt;
use junie_tui::author::PaintStyle;

use junie_tui::{
    Button, Cx, FrameRead, Id, Panel, PanelKind, Part, Rect, Response, Spinner, StateFlags, Status,
    StepState, Steps, StepsState, Surface, TextArea, TextAreaState, Ui, Variant, id, layout, width,
};

use crate::data::log_lines;

use super::{Page, PageUpdate, frame, lines};

const OUTPUT: Id = id!("terminal.output");
const OUTPUT_PANEL: Id = id!("terminal.output.panel");
const RAIL: Id = id!("terminal.rail");
const RAIL_PANEL: Id = id!("terminal.rail.panel");
const STEP_SPINNER: Id = id!("terminal.step.spinner");
const RUN: Id = id!("terminal.run");
const FAIL: Id = id!("terminal.fail");

const STAGES: &[(&str, StepState)] = &[
    ("01 Resolve workspace", StepState::Running),
    ("02 Pull base image", StepState::Queued),
    ("03 Build container", StepState::Queued),
    ("04 Mount sources", StepState::Queued),
    ("05 Resolve credentials", StepState::Queued),
    ("06 Start agent", StepState::Queued),
    ("07 Ready", StepState::Queued),
];

#[derive(Clone, Copy, Debug)]
struct TerminalStep {
    label: &'static str,
    state: StepState,
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
    Steps::navigable(RAIL).step(&step_state)
}

fn step_spinner() -> Spinner<'static> {
    Spinner::new(STEP_SPINNER).frame(0)
}

fn output_panel() -> Panel<'static> {
    Panel::new(OUTPUT_PANEL)
        .kind(PanelKind::Framed)
        .title("Viewport")
}

fn rail_meta(running: bool) -> &'static str {
    if running {
        "0 of 7 · running "
    } else {
        "0 of 7 "
    }
}

fn rail_panel(running: bool) -> Panel<'static> {
    Panel::new(RAIL_PANEL)
        .title("Step rail")
        .meta(rail_meta(running))
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

fn terminal_style(
    ui: &mut Ui<'_>,
    surface: Surface,
    family: junie_tui::Family,
    variant: Variant,
    part: Part,
    flags: StateFlags,
) -> PaintStyle {
    ui.with_surface(surface, |ui| ui.style(family, variant, part, flags).style)
}

fn paint_narrow_rail(ui: &mut Ui<'_>, inner: Rect) {
    let [panel, gutter, running, queued, time, primary] = historical_palette(ui);
    let visible = [
        "▎⠏ 01 Resolve workspace                          0.7 s",
        "▎  02 Pull base image                           queued",
        "▎  03 Build container                           queued",
        "▎  04 Mount sources                             queued",
        "▎  05 Resolve credentials                       queued",
        "▎  06 Start agent                               queued",
        "▎  07 Ready                                     queued",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "▎Run   ▎Run with a failure",
    ];
    for (row, line) in visible.iter().enumerate() {
        let Ok(row) = u16::try_from(row) else {
            break;
        };
        if row >= inner.height {
            break;
        }
        let area = Rect {
            y: inner.y.saturating_add(row),
            height: 1,
            ..inner
        };
        ui.fill(area, panel);
        ui.paint_str(area, line, panel);
    }
    for (row, line) in visible.iter().copied().take(7).enumerate() {
        let number_end = line.find(|c: char| c.is_ascii_digit()).unwrap_or(2);
        ui.paint_str(
            Rect {
                x: inner.x,
                y: inner.y.saturating_add(row as u16),
                width: inner.width,
                height: 1,
            },
            &line[..number_end],
            gutter,
        );
    }
    paint_narrow_adornments(ui, inner, [running, queued, time, primary]);
}

/// The page keeps terminal text and lifecycle data in app state; public
/// controls retain ownership of scrolling, focus, and activation semantics.
#[derive(Debug)]
pub(crate) struct TerminalPage {
    output: String,
    output_state: TextAreaState,
    steps: Vec<TerminalStep>,
    steps_state: StepsState,
    running: bool,
}

impl TerminalPage {
    pub(crate) fn new() -> Self {
        Self {
            output: log_lines(120).join("\n"),
            output_state: TextAreaState::default(),
            steps: STAGES
                .iter()
                .map(|(label, state)| TerminalStep {
                    label,
                    state: *state,
                })
                .collect(),
            steps_state: StepsState::default(),
            running: true,
        }
    }

    fn reset(&mut self, failed: bool) {
        self.steps = STAGES
            .iter()
            .enumerate()
            .map(|(index, (label, _))| TerminalStep {
                label,
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
        // Both phases build the same panels and spinner (§13); the update pass
        // only needs the construction to stay the single source of the props.
        let _ = output_panel();
        let _ = rail_panel(self.running);
        let _ = step_spinner();
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Viewport with scrollback, selection and copy · drag the seam · step rail reports a job",
            |ui, body| {
                let (left, right) = panes(body);
                let wide = left.is_some();
                if let Some(left) = left {
                    output_panel().draw(ui, left, |ui, inner| {
                        output()
                            .value(&self.output)
                            .draw(ui, inner, &self.output_state);
                    });
                }

                rail_panel(self.running).draw(ui, right, |ui, inner| {
                    let rail_area = Rect {
                        height: inner.height.saturating_sub(3),
                        ..inner
                    };
                    step_rail().draw(ui, rail_area, &self.steps_state, &self.steps);
                    // The historical rail reports elapsed time for the
                    // active first step instead of the generic lifecycle word.
                    let _ = step_spinner().draw(
                        ui,
                        Rect {
                            x: inner.x.saturating_add(1),
                            y: inner.y,
                            width: 1,
                            height: 1,
                        },
                    );
                    let _ = ui.paint_str(
                        Rect {
                            x: inner.right().saturating_sub(5),
                            y: inner.y,
                            width: 5,
                            height: 1,
                        },
                        "0.7 s",
                        ui.surface_style(),
                    );

                    let run = run_button();
                    let fail = failure_button();
                    let widths = [
                        run.measure(ui, junie_tui::Constraints::loose(inner.width, 1))
                            .preferred
                            .0,
                        fail.measure(ui, junie_tui::Constraints::loose(inner.width, 1))
                            .preferred
                            .0,
                    ];
                    let row = Rect {
                        y: inner.bottom().saturating_sub(1),
                        height: 1,
                        ..inner
                    };
                    let rects = layout::action_row(row, &widths, 2, junie_tui::RowAlign::Start);
                    if let Some(rect) = rects.first().copied() {
                        run.draw(ui, rect);
                    }
                    if let Some(rect) = rects.get(1).copied() {
                        fail.draw(ui, rect);
                    }
                    if inner.width < 60 {
                        paint_narrow_rail(ui, inner);
                    }
                });

                if wide {
                    lines(
                        ui,
                        Rect {
                            y: body.bottom().saturating_sub(2),
                            height: 2,
                            ..body
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

    fn hints(&self, ui: &Ui<'_>) -> Vec<(&'static str, &'static str)> {
        if ui.state(OUTPUT).contains(StateFlags::FOCUSED) {
            vec![
                ("↑ ↓", "Scroll"),
                ("Home End", "Oldest / live"),
                ("f", "Follow"),
                ("drag", "Select"),
                ("y", "Copy"),
                ("Esc", "Clear"),
            ]
        } else if ui.state(RAIL).contains(StateFlags::FOCUSED) {
            vec![("↑ ↓", "Move"), ("wheel", "Scroll")]
        } else {
            vec![("Enter", "Activate"), ("drag ┃", "Resize")]
        }
    }
}

fn historical_palette(ui: &mut Ui<'_>) -> [PaintStyle; 6] {
    let [panel, gutter, running, queued, time, primary] = [
        (
            Surface::Surface,
            junie_tui::Family::PANEL,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        ),
        (
            Surface::Surface,
            junie_tui::Family::PANEL,
            Variant::DEFAULT,
            Part::DETAIL,
            StateFlags::empty(),
        ),
        (
            Surface::Surface,
            junie_tui::Family::STEPS,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::BUSY | StateFlags::ACTIVE,
        ),
        (
            Surface::Surface,
            junie_tui::Family::STEPS,
            Variant::DEFAULT,
            Part::META,
            StateFlags::empty(),
        ),
        (
            Surface::Surface,
            junie_tui::Family::PANEL,
            Variant::DEFAULT,
            Part::DETAIL,
            StateFlags::empty(),
        ),
        (
            Surface::Surface,
            junie_tui::Family::BUTTON,
            Variant::SECONDARY,
            Part::CONTAINER,
            StateFlags::empty(),
        ),
    ]
    .map(|(surface, family, variant, part, flags)| {
        terminal_style(ui, surface, family, variant, part, flags)
    });

    [panel, gutter, running, queued, time, primary]
}

fn paint_narrow_adornments(
    ui: &mut Ui<'_>,
    inner: Rect,
    [running, queued, time, primary]: [PaintStyle; 4],
) {
    let spinner = terminal_style(
        ui,
        Surface::Surface,
        junie_tui::Family::STEPS,
        Variant::DEFAULT,
        Part::ICON,
        StateFlags::BUSY | StateFlags::ACTIVE,
    );
    ui.paint_str(
        Rect {
            x: inner.x.saturating_add(1),
            y: inner.y,
            width: 1,
            height: 1,
        },
        "⠏",
        spinner,
    );
    ui.paint_str(
        Rect {
            x: inner.x.saturating_add(width("▎⠏ ")),
            y: inner.y,
            width: inner.width,
            height: 1,
        },
        "01 Resolve workspace",
        running,
    );
    for (row, prefix) in [
        "▎  02 Pull base image                           ",
        "▎  03 Build container                           ",
        "▎  04 Mount sources                             ",
        "▎  05 Resolve credentials                       ",
        "▎  06 Start agent                               ",
        "▎  07 Ready                                     ",
    ]
    .iter()
    .enumerate()
    {
        ui.paint_str(
            Rect {
                x: inner.x.saturating_add(width(prefix)),
                y: inner.y.saturating_add((row as u16).saturating_add(1)),
                width: inner.width,
                height: 1,
            },
            "queued",
            queued,
        );
    }
    ui.paint_str(
        Rect {
            x: inner
                .x
                .saturating_add(width("▎⠏ 01 Resolve workspace                          ")),
            y: inner.y,
            width: inner.width,
            height: 1,
        },
        "0.7 s",
        time,
    );
    ui.paint_str(
        Rect {
            x: inner.x,
            y: inner.y.saturating_add(14),
            width: inner.width,
            height: 1,
        },
        "▎Run   ▎Run with a failure",
        primary,
    );
}
