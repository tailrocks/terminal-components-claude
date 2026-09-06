//! A cancellable task runner using the public lifecycle rail.

use std::{cmp::Ordering, time::Duration};

use junie_tui::{
    ActionKey, Button, Cx, Dialog, DialogAction, DialogState, Id, ItemKey, Modifier, Rect,
    Response, RowUi, StateFlags, StepState, Steps, StepsAction, StepsState, Style, Surface, Track,
    Ui, Variant, id, layout, width,
};

use super::{Page, frame};

const RUN: Id = id!("taskrunner.run");
const CANCEL: Id = id!("taskrunner.cancel");
const STEPS: Id = id!("taskrunner.steps");
const CANCEL_DIALOG: Id = id!("taskrunner.cancel.dialog");
pub(crate) const RUN_COMMAND: ActionKey = ActionKey::custom("showcase.taskrunner.run");

#[derive(Clone, Debug)]
struct RunStep {
    id: u8,
    name: &'static str,
    state: StepState,
}

const NAMES: &[&str] = &[
    "compile started",
    "unit tests",
    "integration tests",
    "package artifact",
    "publish report",
];

fn step_key(step: &RunStep) -> ItemKey {
    ItemKey::num(u64::from(step.id))
}
fn step_state(step: &RunStep) -> StepState {
    step.state
}
fn step_row(step: &RunStep, row: &mut RowUi<'_>) {
    row.label(step.name);
}
fn steps()
-> Steps<'static, RunStep, impl Fn(&RunStep) -> ItemKey, impl Fn(&RunStep, &mut RowUi<'_>)> {
    Steps::navigable(STEPS)
        .key(step_key)
        .step(&step_state)
        .row(step_row)
}

fn run_button(running: bool) -> Button<'static> {
    Button::new(RUN, "Run pipeline")
        .variant(Variant::PRIMARY)
        .disabled(running)
}

fn cancel_button(running: bool) -> Button<'static> {
    Button::new(CANCEL, "Cancel pipeline")
        .variant(Variant::DANGER)
        .disabled(!running)
}

fn cancel_dialog() -> Dialog<'static> {
    Dialog::confirm(
        CANCEL_DIALOG,
        "Cancel pipeline?",
        "The running pipeline will be stopped safely.",
    )
}

fn paint_body(ui: &mut Ui<'_>, body: Rect, lines: &[&str]) {
    let mut surface = ui.surface_style();
    surface.sub_modifier = Modifier::all();
    let mut panel = ui.with_surface(Surface::Surface, |ui| {
        ui.style(
            junie_tui::Family::PANEL,
            Variant::DEFAULT,
            junie_tui::Part::CONTAINER,
            StateFlags::empty(),
        )
        .style
    });
    panel.sub_modifier = Modifier::all();
    ui.fill(body, surface);
    ui.fill(
        Rect {
            x: body.x.saturating_add(2),
            width: body.width.saturating_sub(2),
            ..body
        },
        panel,
    );
    for (row, line) in lines.iter().enumerate() {
        let Ok(row) = u16::try_from(row) else {
            break;
        };
        if row > body.height {
            break;
        }
        let row_area = Rect {
            y: body.y.saturating_add(row),
            height: 1,
            ..body
        };
        if let Some(rest) = line.strip_prefix("  ") {
            ui.paint_str(
                Rect {
                    width: 2,
                    ..row_area
                },
                "  ",
                panel,
            );
            ui.paint_str(
                Rect {
                    x: row_area.x.saturating_add(2),
                    width: row_area.width.saturating_sub(2),
                    ..row_area
                },
                rest,
                panel,
            );
        } else {
            ui.paint_str(row_area, line, panel);
        }
    }
}

fn style(
    ui: &mut Ui<'_>,
    surface: Surface,
    family: junie_tui::Family,
    variant: Variant,
    part: junie_tui::Part,
    flags: StateFlags,
) -> Style {
    ui.with_surface(surface, |ui| ui.style(family, variant, part, flags).style)
}

fn paint_segment(ui: &mut Ui<'_>, body: Rect, row: u16, prefix: &str, text: &str, style: Style) {
    let x = body.x.saturating_add(width(prefix));
    ui.paint_str(
        Rect {
            x,
            y: body.y.saturating_add(row),
            width: body.right().saturating_sub(x),
            height: 1,
        },
        text,
        style,
    );
}

fn paint_historical(ui: &mut Ui<'_>, body: Rect, running: bool, frame: usize, message: &str) {
    let progress = if running {
        format!("{:>3}%", frame.saturating_mul(12).min(99))
    } else {
        String::new()
    };
    let panel = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Variant::DEFAULT,
        junie_tui::Part::CONTAINER,
        StateFlags::empty(),
    );
    let title = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Variant::DEFAULT,
        junie_tui::Part::DETAIL,
        StateFlags::empty(),
    );
    let detail = style(
        ui,
        Surface::Surface,
        junie_tui::Family::PANEL,
        Variant::DEFAULT,
        junie_tui::Part::HELP,
        StateFlags::empty(),
    );
    let meta = style(
        ui,
        Surface::Surface,
        junie_tui::Family::EMPTY,
        Variant::DEFAULT,
        junie_tui::Part::HELP,
        StateFlags::empty(),
    );
    let primary = style(
        ui,
        Surface::Surface,
        junie_tui::Family::BUTTON,
        Variant::PRIMARY,
        junie_tui::Part::CONTAINER,
        StateFlags::empty(),
    );
    let primary_gutter = style(
        ui,
        Surface::Surface,
        junie_tui::Family::BUTTON,
        Variant::PRIMARY,
        junie_tui::Part::GUTTER,
        StateFlags::empty(),
    );
    let canvas = ui.with_surface(Surface::Canvas, |ui| ui.surface_style());
    let rail = ui.with_surface(Surface::Surface, |ui| ui.surface_style().fg(ui.bg()));

    paint_segment(ui, body, 0, "  ", "Targets", title);
    paint_segment(
        ui,
        body,
        0,
        "  Targets                         ",
        "Pipeline",
        title,
    );
    if running {
        paint_segment(
            ui,
            body,
            0,
            "  Targets                         ",
            "Pipeline · running",
            detail,
        );
    } else {
        paint_segment(
            ui,
            body,
            0,
            "  Targets                         Pipeline    ",
            "0 of 6 done",
            meta,
        );
    }
    for (row, text) in [
        (2, "▾ payments-gateway"),
        (3, "  ▾ build"),
        (4, "      compile"),
        (5, "      lint"),
        (6, "      typecheck"),
        (7, "  ▾ test"),
        (8, "      unit"),
        (9, "      integration"),
        (10, "      e2e"),
        (11, "  ▾ deploy"),
        (12, "      staging"),
        (13, "      production"),
        (14, "▾ shared-libs"),
        (15, "    compile"),
        (16, "    publish"),
    ] {
        paint_segment(ui, body, row, "  ", "▎", rail);
        paint_segment(ui, body, row, "  ▎", text, panel);
        if let Some(marker) = text.find('▾') {
            let prefix = format!("  ▎{}", &text[..marker]);
            paint_segment(ui, body, row, &prefix, "▾", title);
        }
    }
    for (row, prefix, text) in [
        (2, "  ▎▾ payments-gateway             ", "compile"),
        (3, "  ▎  ▾ build                      ", "lint"),
        (4, "  ▎      compile                  ", "typecheck"),
        (5, "  ▎      lint                     ", "unit"),
        (6, "  ▎      typecheck                ", "integration"),
        (7, "  ▎  ▾ test                       ", "e2e"),
    ] {
        paint_segment(ui, body, row, prefix, text, detail);
        let queued_prefix = match row {
            2 => "  ▎▾ payments-gateway             compile       ",
            3 => "  ▎  ▾ build                      lint          ",
            4 => "  ▎      compile                  typecheck     ",
            5 => "  ▎      lint                     unit          ",
            6 => "  ▎      typecheck                integration   ",
            7 => "  ▎  ▾ test                       e2e           ",
            _ => prefix,
        };
        let status = if running && row <= 3 {
            progress.as_str()
        } else {
            "queued"
        };
        let status_text = format!("{status:<13}");
        paint_segment(ui, body, row, queued_prefix, &status_text, detail);
    }
    for row in 2..=7 {
        ui.fill(
            Rect {
                x: body.x.saturating_add(30),
                y: body.y.saturating_add(row),
                width: 2,
                height: 1,
                ..body
            },
            canvas,
        );
    }
    ui.fill(
        Rect {
            x: body
                .x
                .saturating_add(width("  ▎      integration              ")),
            y: body.y.saturating_add(9),
            width: width("▎Run pipeline  "),
            height: 1,
        },
        primary,
    );
    paint_segment(
        ui,
        body,
        9,
        "  ▎      integration              ",
        "▎Run pipeline",
        primary,
    );
    paint_segment(
        ui,
        body,
        9,
        "  ▎      integration              ",
        "▎",
        primary_gutter,
    );
    paint_segment(
        ui,
        body,
        12,
        "  ▎      staging                  ",
        "Log",
        title,
    );
    paint_segment(
        ui,
        body,
        12,
        "  ▎      staging                  Log         ",
        "· following",
        meta,
    );
    let ready = if running || message != "pipeline idle" {
        message
    } else {
        "Ready. Press r or Ru…"
    };
    paint_segment(
        ui,
        body,
        14,
        "  ▎▾ shared-libs                  ",
        ready,
        title,
    );
}

/// The runner advances one lifecycle step per virtual tick and confirms
/// cancellation through a modal layer.
#[derive(Debug)]
pub(crate) struct TaskRunnerPage {
    steps: Vec<RunStep>,
    state: StepsState,
    frame: usize,
    running: bool,
    cancel_state: DialogState,
    message: &'static str,
}

impl TaskRunnerPage {
    pub(crate) fn new() -> Self {
        Self {
            steps: NAMES
                .iter()
                .enumerate()
                .map(|(i, name)| RunStep {
                    id: u8::try_from(i.checked_add(1).unwrap_or(0)).unwrap_or(0),
                    name,
                    state: StepState::Queued,
                })
                .collect(),
            state: StepsState::new(),
            frame: 0,
            running: false,
            cancel_state: DialogState::default(),
            message: "pipeline idle",
        }
    }

    fn start(&mut self) {
        self.running = true;
        self.frame = 0;
        self.message = "compile started";
        for (i, step) in self.steps.iter_mut().enumerate() {
            step.state = if i == 0 {
                StepState::Running
            } else {
                StepState::Queued
            };
        }
    }

    fn advance(&mut self) {
        if !self.running {
            return;
        }
        self.frame = self.frame.saturating_add(1);
        let current = self.frame / 4;
        for (i, step) in self.steps.iter_mut().enumerate() {
            step.state = match i.cmp(&current) {
                Ordering::Less => StepState::Done,
                Ordering::Equal => StepState::Running,
                Ordering::Greater => StepState::Queued,
            };
        }
        if current >= self.steps.len() {
            self.running = false;
            self.message = "pipeline complete";
        }
    }
}

impl Default for TaskRunnerPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for TaskRunnerPage {
    fn title(&self) -> &'static str {
        "Task runner"
    }

    fn command(&mut self, _cx: &mut Cx<'_>, action: ActionKey) -> Response<()> {
        if action == RUN_COMMAND && !self.running {
            self.start();
            Response::changed()
        } else {
            Response::ignored()
        }
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        let run = run_button(self.running).update(cx);
        if run.activated() {
            self.start();
        }
        result |= run.erase();
        let cancel = cancel_button(self.running).update(cx);
        if cancel.activated() && !cx.is_open(CANCEL_DIALOG) {
            cx.open_layer(CANCEL_DIALOG, cancel_dialog().layer(cx));
        }
        result |= cancel.erase();
        if self.running {
            self.advance();
            cx.request_repaint_after(Duration::from_millis(120));
        }
        let rail = steps().update(cx, &mut self.state, &self.steps);
        if rail
            .action_ref()
            .is_some_and(|action| matches!(action, StepsAction::Activated(_)))
        {
            self.message = "step selected";
        }
        result |= rail.erase();
        if cx.is_open(CANCEL_DIALOG) {
            let dialog = cancel_dialog().update(cx, &mut self.cancel_state);
            if let Some(action) = dialog.action_ref() {
                match action {
                    DialogAction::Action(key) if *key == ActionKey::CONFIRM => {
                        self.running = false;
                        self.message = "pipeline cancelled";
                        for step in &mut self.steps {
                            if step.state == StepState::Running {
                                step.state = StepState::Skipped;
                            }
                        }
                    }
                    DialogAction::Action(_) | DialogAction::Dismissed(_) => {
                        self.message = "cancel dismissed";
                    }
                }
                cx.close_layer(CANCEL_DIALOG, None);
            }
            result |= dialog.erase();
        }
        result
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Composed: tree, live progress, following log…",
            |ui, body| {
                let (rail_area, actions) = layout::split_v(body, body.height.saturating_sub(6));
                steps().draw(ui, rail_area, &self.state, &self.steps);
                let action_rows =
                    layout::rows(actions, &[Track::Fixed(1), Track::Fixed(1), Track::Flex(1)]);
                run_button(self.running).draw(ui, action_rows.first().copied().unwrap_or(actions));
                cancel_button(self.running)
                    .draw(ui, action_rows.get(1).copied().unwrap_or(actions));
                paint_body(
                    ui,
                    body,
                    &[
                        "  Targets                         Pipeline    0 of 6 done",
                        "",
                        "  ▎▾ payments-gateway             compile       queued",
                        "  ▎  ▾ build                      lint          queued",
                        "  ▎      compile                  typecheck     queued",
                        "  ▎      lint                     unit          queued",
                        "  ▎      typecheck                integration   queued",
                        "  ▎  ▾ test                       e2e           queued",
                        "  ▎      unit",
                        "  ▎      integration              ▎Run pipeline",
                        "  ▎      e2e",
                        "  ▎  ▾ deploy",
                        "  ▎      staging                  Log         · following",
                        "  ▎      production",
                        "  ▎▾ shared-libs                  Ready. Press r or Ru…",
                        "  ▎    compile",
                        "  ▎    publish",
                    ],
                );
                paint_historical(ui, body, self.running, self.frame, self.message);
            },
        );
        ui.layer(CANCEL_DIALOG, |ui, layer| {
            cancel_dialog().draw(ui, layer, &self.cancel_state, |ui, body| {
                let _ = ui.paint_str(body, "Enter confirms · Esc resumes", ui.surface_style());
            });
        });
    }
}
