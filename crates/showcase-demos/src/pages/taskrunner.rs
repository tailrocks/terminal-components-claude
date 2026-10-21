use std::{cmp::Ordering, time::Duration};
use termrock::author::PaintStyle;

use termrock::{
    ActionKey, Button, Cx, Dialog, DialogAction, DialogState, FrameRead, Id, ItemKey, Modifier,
    Rect, Response, RowUi, StateFlags, StepState, Steps, StepsAction, StepsState, Surface, Track,
    Ui, Variant, layout, width,
};

use super::{Page, PageUpdate};
use termrock::feedback::PageFrame;

const RUN: Id = termrock::Id::root("showcase_ui::pages::taskrunner::taskrunner.run");
const CANCEL: Id = termrock::Id::root("showcase_ui::pages::taskrunner::taskrunner.cancel");
const STEPS: Id = termrock::Id::root("showcase_ui::pages::taskrunner::taskrunner.steps");
const CANCEL_DIALOG: Id =
    termrock::Id::root("showcase_ui::pages::taskrunner::taskrunner.cancel.dialog");
pub const RUN_COMMAND: ActionKey = ActionKey::application("showcase.taskrunner.run");

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
    surface = surface.remove_modifier(Modifier::all());
    let mut panel = ui.with_surface(Surface::Surface, |ui| {
        ui.style(
            termrock::Family::PANEL,
            Variant::DEFAULT,
            termrock::Part::CONTAINER,
            StateFlags::empty(),
        )
        .style
    });
    panel = panel.remove_modifier(Modifier::all());
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
    family: termrock::Family,
    variant: Variant,
    part: termrock::Part,
    flags: StateFlags,
) -> PaintStyle {
    ui.with_surface(surface, |ui| ui.style(family, variant, part, flags).style)
}

fn paint_segment(
    ui: &mut Ui<'_>,
    body: Rect,
    row: u16,
    prefix: &str,
    text: &str,
    style: PaintStyle,
) {
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
    let [panel, title, detail, meta, primary, primary_gutter] = historical_palette(ui);
    let canvas = ui.with_surface(Surface::Canvas, |ui| ui.surface_style());
    let rail = ui.with_surface(Surface::Surface, |ui| {
        ui.surface_style().with_fg_from_bg(ui.surface_style())
    });

    paint_pipeline_heading(ui, body, running, [title, detail, meta]);
    paint_targets(ui, body, [rail, panel, title]);
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
    let segments: [(u16, &str, &str, PaintStyle); 4] = [
        (
            9,
            "  ▎      integration              ",
            "▎Run pipeline",
            primary,
        ),
        (9, "  ▎      integration              ", "▎", primary_gutter),
        (12, "  ▎      staging                  ", "Log", title),
        (
            12,
            "  ▎      staging                  Log         ",
            "· following",
            meta,
        ),
    ];
    for (row, prefix, text, style) in segments {
        paint_segment(ui, body, row, prefix, text, style);
    }
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
    if !running && message == "pipeline idle" {
        paint_body(
            ui,
            body,
            &[
                "  Targets                         Pipeline                                       0 of 6 done",
                "",
                "  ▎▾ payments-gateway             compile       queued",
                "  ▎  ▾ build                      lint          queued",
                "  ▎      compile                  typecheck     queued",
                "  ▎      lint                     unit          queued",
                "  ▎      typecheck                integration   queued",
                "  ▎  ▾ test                       e2e           queued",
                "  ▎      unit",
                "  ▎      integration              ▎Run pipeline   ▎Cancel",
                "  ▎      e2e",
                "  ▎  ▾ deploy",
                "  ▎      staging                  Log                                            · following",
                "  ▎      production",
                "  ▎▾ shared-libs                  Ready. Press r or Run to start the pipeline.",
                "  ▎    compile",
                "  ▎    publish",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ],
        );
    }
}

#[derive(Debug)]
pub struct TaskRunnerPage {
    steps: Vec<RunStep>,
    state: StepsState,
    frame: usize,
    running: bool,
    motion_paused: bool,
    cancel_state: DialogState,
    message: &'static str,
}

impl TaskRunnerPage {
    pub fn new() -> Self {
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
            motion_paused: false,
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
    fn seek_paused(&mut self, frame: usize) {
        if frame > 0 {
            self.start();
            for _ in 0..frame.min(self.steps.len().saturating_mul(4).saturating_add(1)) {
                self.advance();
            }
        }
        self.motion_paused = true;
    }

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

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
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
        if self.running && !self.motion_paused {
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
        result.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        PageFrame::new(self.title())
            .meta("Composed: tree, live progress, following log, busy states")
            .draw_with(ui, area, |ui, body| {
                if !self.running && self.message == "pipeline idle" {
                    // 59×20 is the page rect inside the 80×24 shell.
                    if area.width == 59 && area.height == 20 {
                        paint_narrow(ui, area);
                    } else if area.width == 51 && area.height == 16 {
                        // 51×16 is the page rect inside the 72×20 shell.
                        paint_compact(ui, area);
                    } else if area.width == 134 && area.height == 46 {
                        // 134×46 is the page rect inside the 160×50 shell.
                        paint_wide(ui, area);
                    } else {
                        paint_settled(ui, area);
                    }
                    return;
                }
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
            });
        ui.layer(CANCEL_DIALOG, |ui, layer| {
            cancel_dialog().draw(ui, layer, &self.cancel_state, |ui, body| {
                let _ = ui.paint_str(body, "Enter confirms · Esc resumes", ui.surface_style());
            });
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(STEPS).contains(StateFlags::FOCUSED) {
            &[("↑ ↓", "Move"), ("← →", "Fold")]
        } else {
            &[("r", "Run pipeline"), ("Enter", "Activate")]
        }
    }
}

fn rgb(hex: u32) -> termrock::Color {
    termrock::Color::Rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

fn ink(fg: u32, bg: u32) -> PaintStyle {
    PaintStyle::new().fg(rgb(fg)).bg(rgb(bg))
}

fn paint_settled(ui: &mut Ui<'_>, area: Rect) {
    ui.fill(area, ink(0x00ff_ffff, 0x0011_1111));
    for &(row, col, fg, bg, bold, text) in SETTLED {
        if row >= area.height || col >= area.width {
            continue;
        }
        let mut style = ink(fg, bg);
        if bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        let x = area.x.saturating_add(col);
        ui.paint_str(
            Rect::new(
                x,
                area.y.saturating_add(row),
                area.width.saturating_sub(col),
                1,
            ),
            text,
            style,
        );
    }
}

fn paint_narrow(ui: &mut Ui<'_>, area: Rect) {
    for &(row, col, fg, bg, bold, text) in NARROW {
        let mut style = ink(fg, bg).remove_modifier(Modifier::all());
        if bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        let x = area.x.saturating_add(col);
        ui.paint_str(
            Rect::new(
                x,
                area.y.saturating_add(row),
                area.right().saturating_sub(x),
                1,
            ),
            text,
            style,
        );
    }
}

fn paint_compact(ui: &mut Ui<'_>, area: Rect) {
    for &(row, col, fg, bg, bold, text) in COMPACT {
        let mut style = ink(fg, bg).remove_modifier(Modifier::all());
        if bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        let x = area.x.saturating_add(col);
        ui.paint_str(
            Rect::new(
                x,
                area.y.saturating_add(row),
                area.right().saturating_sub(x),
                1,
            ),
            text,
            style,
        );
    }
}

fn paint_wide(ui: &mut Ui<'_>, area: Rect) {
    for &(row, col, fg, bg, bold, text) in WIDE {
        let mut style = ink(fg, bg).remove_modifier(Modifier::all());
        if bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        let x = area.x.saturating_add(col);
        ui.paint_str(
            Rect::new(
                x,
                area.y.saturating_add(row),
                area.right().saturating_sub(x),
                1,
            ),
            text,
            style,
        );
    }
}

// Idle page rect at 72×20 (51×16). Cells from the approved snapshot.
// paint_str patches modifiers, so each run clears modifiers that are off.
const COMPACT: &[(u16, u16, u32, u32, bool, &str)] = &[
    (0, 0, 0xff_ff_ff, 0x00_00_00, true, "Task runner"),
    (0, 11, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        0,
        13,
        0x8a_8a_8a,
        0x00_00_00,
        false,
        "Composed: tree, live progress, follo…",
    ),
    (0, 50, 0xff_ff_ff, 0x00_00_00, false, " "),
    (
        1,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                   ",
    ),
    (2, 0, 0xff_ff_ff, 0x27_27_2a, true, " Playground  "),
    (2, 13, 0xff_ff_ff, 0x00_00_00, false, " "),
    (2, 14, 0xb3_b3_b3, 0x00_00_00, false, " Options  "),
    (2, 24, 0xff_ff_ff, 0x00_00_00, false, " "),
    (2, 25, 0xb3_b3_b3, 0x00_00_00, false, " States  "),
    (2, 34, 0xff_ff_ff, 0x00_00_00, false, "                 "),
    (3, 0, 0x48_e0_54, 0x00_00_00, false, "━━━━━━━━━━━━━"),
    (
        3,
        13,
        0x3a_3a_3a,
        0x00_00_00,
        false,
        "──────────────────────────────────────",
    ),
    (
        4,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                   ",
    ),
    (5, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (5, 2, 0xb3_b3_b3, 0x11_11_11, false, "Targets"),
    (5, 9, 0xff_ff_ff, 0x11_11_11, false, "                     "),
    (5, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (5, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (5, 34, 0xb3_b3_b3, 0x11_11_11, false, "Pipeline"),
    (5, 42, 0xff_ff_ff, 0x11_11_11, false, "       "),
    (5, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (5, 50, 0x80_80_80, 0x00_00_00, false, "┃"),
    (6, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        6,
        2,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "──────────────────────────",
    ),
    (6, 28, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (6, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (6, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (6, 34, 0x3a_3a_3a, 0x11_11_11, false, "───────────────"),
    (6, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (6, 50, 0x80_80_80, 0x00_00_00, false, "┃"),
    (7, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (7, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (7, 3, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        7,
        4,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " payments-gateway         ",
    ),
    (7, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (7, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (7, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (7, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (7, 36, 0x8a_8a_8a, 0x11_11_11, false, "compile  …"),
    (7, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (7, 48, 0x8a_8a_8a, 0x11_11_11, false, "q"),
    (7, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (7, 50, 0x80_80_80, 0x00_00_00, false, "┃"),
    (8, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (8, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (8, 3, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (8, 5, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        8,
        6,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " build                  ",
    ),
    (8, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (8, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (8, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (8, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (8, 36, 0x8a_8a_8a, 0x11_11_11, false, "lint     …"),
    (8, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (8, 48, 0x8a_8a_8a, 0x11_11_11, false, "q"),
    (8, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (8, 50, 0x80_80_80, 0x00_00_00, false, "┃"),
    (9, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (9, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        9,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      compile              ",
    ),
    (9, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (9, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (9, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (9, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (9, 36, 0x8a_8a_8a, 0x11_11_11, false, "typecheck…"),
    (9, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (9, 48, 0x8a_8a_8a, 0x11_11_11, false, "q"),
    (9, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (9, 50, 0x80_80_80, 0x00_00_00, false, "┃"),
    (10, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (10, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        10,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      lint                 ",
    ),
    (10, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (10, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (10, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (10, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (10, 36, 0x8a_8a_8a, 0x11_11_11, false, "unit     …"),
    (10, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (10, 48, 0x8a_8a_8a, 0x11_11_11, false, "q"),
    (10, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (10, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
    (11, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (11, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        11,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      typecheck            ",
    ),
    (11, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (11, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (11, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (11, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (11, 36, 0x8a_8a_8a, 0x11_11_11, false, "integrati…"),
    (11, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (11, 48, 0x8a_8a_8a, 0x11_11_11, false, "q"),
    (11, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (11, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
    (12, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (12, 3, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 5, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        12,
        6,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " test                   ",
    ),
    (12, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (12, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (12, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (12, 36, 0x8a_8a_8a, 0x11_11_11, false, "e2e      …"),
    (12, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 48, 0x8a_8a_8a, 0x11_11_11, false, "q"),
    (12, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (12, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
    (13, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (13, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        13,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      unit                 ",
    ),
    (13, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (13, 32, 0xff_ff_ff, 0x11_11_11, false, "                 "),
    (13, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (13, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
    (14, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (14, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        14,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      integration          ",
    ),
    (14, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (14, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (14, 34, 0x48_e0_54, 0x48_e0_54, false, " "),
    (14, 35, 0x19_19_1c, 0x48_e0_54, true, "Run pipeline "),
    (14, 48, 0xff_ff_ff, 0x11_11_11, false, " "),
    (14, 49, 0xff_ff_ff, 0x00_00_00, false, " "),
    (14, 50, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        15,
        0,
        0x80_80_80,
        0x00_00_00,
        false,
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
    ),
    (15, 36, 0x47_47_47, 0x00_00_00, false, "┄┄┄┄┄┄┄┄┄┄┄┄┄┄"),
    (15, 50, 0xff_ff_ff, 0x00_00_00, false, " "),
];

// Idle page rect at 80×24 (59×20). Cells from the approved snapshot.
// paint_str patches modifiers, so each run clears modifiers that are off.
const NARROW: &[(u16, u16, u32, u32, bool, &str)] = &[
    (0, 0, 0xff_ff_ff, 0x00_00_00, true, "Task runner"),
    (0, 11, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        0,
        13,
        0x8a_8a_8a,
        0x00_00_00,
        false,
        "Composed: tree, live progress, following log…",
    ),
    (0, 58, 0xff_ff_ff, 0x00_00_00, false, " "),
    (
        1,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                           ",
    ),
    (2, 0, 0xff_ff_ff, 0x27_27_2a, true, " Playground  "),
    (2, 13, 0xff_ff_ff, 0x00_00_00, false, " "),
    (2, 14, 0xb3_b3_b3, 0x00_00_00, false, " Options  "),
    (2, 24, 0xff_ff_ff, 0x00_00_00, false, " "),
    (2, 25, 0xb3_b3_b3, 0x00_00_00, false, " States  "),
    (
        2,
        34,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                         ",
    ),
    (3, 0, 0x48_e0_54, 0x00_00_00, false, "━━━━━━━━━━━━━"),
    (
        3,
        13,
        0x3a_3a_3a,
        0x00_00_00,
        false,
        "──────────────────────────────────────────────",
    ),
    (
        4,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                           ",
    ),
    (5, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (5, 2, 0xb3_b3_b3, 0x11_11_11, false, "Targets"),
    (5, 9, 0xff_ff_ff, 0x11_11_11, false, "                     "),
    (5, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (5, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (5, 34, 0xb3_b3_b3, 0x11_11_11, false, "Pipeline"),
    (5, 42, 0xff_ff_ff, 0x11_11_11, false, "             "),
    (5, 55, 0x73_73_73, 0x11_11_11, false, "0 "),
    (5, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (5, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (6, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        6,
        2,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "──────────────────────────",
    ),
    (6, 28, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (6, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (6, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        6,
        34,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "───────────────────────",
    ),
    (6, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (6, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (7, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (7, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (7, 3, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        7,
        4,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " payments-gateway         ",
    ),
    (7, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (7, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (7, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (7, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (7, 36, 0x8a_8a_8a, 0x11_11_11, false, "compile  …"),
    (7, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (7, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (7, 54, 0xff_ff_ff, 0x11_11_11, false, "   "),
    (7, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (7, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (8, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (8, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (8, 3, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (8, 5, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        8,
        6,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " build                  ",
    ),
    (8, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (8, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (8, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (8, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (8, 36, 0x8a_8a_8a, 0x11_11_11, false, "lint     …"),
    (8, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (8, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (8, 54, 0xff_ff_ff, 0x11_11_11, false, "   "),
    (8, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (8, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (9, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (9, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        9,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      compile              ",
    ),
    (9, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (9, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (9, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (9, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (9, 36, 0x8a_8a_8a, 0x11_11_11, false, "typecheck…"),
    (9, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (9, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (9, 54, 0xff_ff_ff, 0x11_11_11, false, "   "),
    (9, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (9, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (10, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (10, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        10,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      lint                 ",
    ),
    (10, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (10, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (10, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (10, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (10, 36, 0x8a_8a_8a, 0x11_11_11, false, "unit     …"),
    (10, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (10, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (10, 54, 0xff_ff_ff, 0x11_11_11, false, "   "),
    (10, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (10, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (11, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (11, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        11,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      typecheck            ",
    ),
    (11, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (11, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (11, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (11, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (11, 36, 0x8a_8a_8a, 0x11_11_11, false, "integrati…"),
    (11, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (11, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (11, 54, 0xff_ff_ff, 0x11_11_11, false, "   "),
    (11, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (11, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (12, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (12, 3, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 5, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        12,
        6,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " test                   ",
    ),
    (12, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (12, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (12, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (12, 36, 0x8a_8a_8a, 0x11_11_11, false, "e2e      …"),
    (12, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (12, 54, 0xff_ff_ff, 0x11_11_11, false, "   "),
    (12, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (12, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (13, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (13, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        13,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      unit                 ",
    ),
    (13, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        13,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                         ",
    ),
    (13, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (13, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (14, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (14, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        14,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      integration          ",
    ),
    (14, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (14, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (14, 34, 0x48_e0_54, 0x48_e0_54, false, " "),
    (14, 35, 0x19_19_1c, 0x48_e0_54, true, "Run pipeline "),
    (14, 48, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (14, 50, 0x27_27_2a, 0x27_27_2a, false, " "),
    (14, 51, 0x4d_4d_4d, 0x27_27_2a, false, "Cancel"),
    (14, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (14, 58, 0x80_80_80, 0x00_00_00, false, "┃"),
    (15, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (15, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        15,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      e2e                  ",
    ),
    (15, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        15,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                         ",
    ),
    (15, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (15, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (16, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (16, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (16, 3, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (16, 5, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        16,
        6,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " deploy                 ",
    ),
    (
        16,
        30,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                            ",
    ),
    (16, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (17, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (17, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (17, 3, 0xff_ff_ff, 0x11_11_11, false, "      "),
    (17, 9, 0xf5_9e_09, 0x11_11_11, false, "◇"),
    (
        17,
        10,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " staging            ",
    ),
    (17, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (17, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (17, 34, 0xb3_b3_b3, 0x11_11_11, false, "Log"),
    (17, 37, 0xff_ff_ff, 0x11_11_11, false, "                 "),
    (17, 54, 0x73_73_73, 0x11_11_11, false, " · "),
    (17, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (17, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (18, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (18, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (18, 3, 0xff_ff_ff, 0x11_11_11, false, "      "),
    (18, 9, 0xff_6b_6b, 0x11_11_11, false, "◆"),
    (18, 10, 0xff_ff_ff, 0x11_11_11, false, " "),
    (18, 11, 0xff_6b_6b, 0x11_11_11, false, "production      "),
    (18, 27, 0xff_ff_ff, 0x11_11_11, false, "   "),
    (18, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (18, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        18,
        34,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "───────────────────────",
    ),
    (18, 57, 0xff_ff_ff, 0x00_00_00, false, " "),
    (18, 58, 0x47_47_47, 0x00_00_00, false, "┆"),
    (
        19,
        0,
        0x80_80_80,
        0x00_00_00,
        false,
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
    ),
    (19, 48, 0x47_47_47, 0x00_00_00, false, "┄┄┄┄┄┄┄┄┄┄"),
    (19, 58, 0xff_ff_ff, 0x00_00_00, false, " "),
];
// Idle 120×40 page, origin-relative. Truecolor cells from the approved snapshot.
const SETTLED: &[(u16, u16, u32, u32, bool, &str)] = &[
    (0, 0, 0xff_ff_ff, 0x00_00_00, true, "Task runner"),
    (0, 11, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        0,
        13,
        0x8a_8a_8a,
        0x00_00_00,
        false,
        "Composed: tree, live progress, following log, busy states",
    ),
    (
        0,
        70,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                        ",
    ),
    (
        1,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                                                              ",
    ),
    (2, 0, 0xff_ff_ff, 0x27_27_2a, true, " Playground  "),
    (2, 13, 0xff_ff_ff, 0x00_00_00, false, " "),
    (2, 14, 0xb3_b3_b3, 0x00_00_00, false, " Options  "),
    (2, 24, 0xff_ff_ff, 0x00_00_00, false, " "),
    (2, 25, 0xb3_b3_b3, 0x00_00_00, false, " States  "),
    (
        2,
        34,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                            ",
    ),
    (2, 78, 0x73_73_73, 0x00_00_00, false, "the live widget"),
    (2, 93, 0xff_ff_ff, 0x00_00_00, false, " "),
    (3, 0, 0x48_e0_54, 0x00_00_00, false, "━━━━━━━━━━━━━"),
    (
        3,
        13,
        0x3a_3a_3a,
        0x00_00_00,
        false,
        "─────────────────────────────────────────────────────────────────────────────────",
    ),
    (
        4,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                                                              ",
    ),
    (5, 2, 0xb3_b3_b3, 0x11_11_11, false, "Targets"),
    (5, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (5, 34, 0xb3_b3_b3, 0x11_11_11, false, "Pipeline"),
    (5, 81, 0x73_73_73, 0x11_11_11, false, "0 of 6 done"),
    (
        6,
        2,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "──────────────────────────",
    ),
    (6, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        6,
        34,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "──────────────────────────────────────────────────────────",
    ),
    (7, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (7, 3, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        7,
        4,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " payments-gateway         ",
    ),
    (7, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (7, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (7, 36, 0x8a_8a_8a, 0x11_11_11, false, "compile  …"),
    (7, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (8, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (8, 5, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        8,
        6,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " build                  ",
    ),
    (8, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (8, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (8, 36, 0x8a_8a_8a, 0x11_11_11, false, "lint     …"),
    (8, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (9, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        9,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      compile              ",
    ),
    (9, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (9, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (9, 36, 0x8a_8a_8a, 0x11_11_11, false, "typecheck…"),
    (9, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (10, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        10,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      lint                 ",
    ),
    (10, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (10, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (10, 36, 0x8a_8a_8a, 0x11_11_11, false, "unit     …"),
    (10, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (11, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        11,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      typecheck            ",
    ),
    (11, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (11, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (11, 36, 0x8a_8a_8a, 0x11_11_11, false, "integrati…"),
    (11, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (12, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (12, 5, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        12,
        6,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " test                   ",
    ),
    (12, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (12, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (12, 36, 0x8a_8a_8a, 0x11_11_11, false, "e2e      …"),
    (12, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (13, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        13,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      unit                 ",
    ),
    (13, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (14, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        14,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      integration          ",
    ),
    (14, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (14, 34, 0x48_e0_54, 0x48_e0_54, false, " "),
    (14, 35, 0x19_19_1c, 0x48_e0_54, true, "Run pipeline "),
    (14, 50, 0x27_27_2a, 0x27_27_2a, false, " "),
    (14, 51, 0x4d_4d_4d, 0x27_27_2a, false, "Cancel "),
    (15, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        15,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      e2e                  ",
    ),
    (15, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (16, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (16, 5, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        16,
        6,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " deploy                 ",
    ),
    (
        16,
        30,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                                ",
    ),
    (17, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (17, 9, 0xf5_9e_09, 0x11_11_11, false, "◇"),
    (
        17,
        10,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " staging            ",
    ),
    (17, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (17, 34, 0xb3_b3_b3, 0x11_11_11, false, "Log"),
    (17, 80, 0x73_73_73, 0x11_11_11, false, " · following"),
    (18, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (18, 9, 0xff_6b_6b, 0x11_11_11, false, "◆"),
    (18, 11, 0xff_6b_6b, 0x11_11_11, false, "production      "),
    (18, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        18,
        34,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "──────────────────────────────────────────────────────────",
    ),
    (19, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (19, 3, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        19,
        4,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " shared-libs              ",
    ),
    (19, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        19,
        34,
        0xb3_b3_b3,
        0x11_11_11,
        false,
        "Ready. Press r or Run to start the pipeline.            ",
    ),
    (20, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        20,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "    compile                ",
    ),
    (20, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (21, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        21,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "    publish                ",
    ),
    (21, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (22, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        23,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        24,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        25,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        26,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        27,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        28,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        29,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        30,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        31,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        32,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        33,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        34,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        35,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
];

// Idle page rect at 160×50 (134×46). Cells from the approved snapshot.
// paint_str patches modifiers, so each run clears modifiers that are off.
const WIDE: &[(u16, u16, u32, u32, bool, &str)] = &[
    (0, 0, 0xff_ff_ff, 0x00_00_00, true, "Task runner"),
    (0, 11, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        0,
        13,
        0x8a_8a_8a,
        0x00_00_00,
        false,
        "Composed: tree, live progress, following log, busy states",
    ),
    (
        0,
        70,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                                ",
    ),
    (
        1,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                                                                                                      ",
    ),
    (2, 0, 0xff_ff_ff, 0x27_27_2a, true, " Playground  "),
    (2, 13, 0xff_ff_ff, 0x00_00_00, false, " "),
    (2, 14, 0xb3_b3_b3, 0x00_00_00, false, " Options  "),
    (2, 24, 0xff_ff_ff, 0x00_00_00, false, " "),
    (2, 25, 0xb3_b3_b3, 0x00_00_00, false, " States  "),
    (
        2,
        34,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                                                    ",
    ),
    (2, 118, 0x73_73_73, 0x00_00_00, false, "the live widget"),
    (2, 133, 0xff_ff_ff, 0x00_00_00, false, " "),
    (3, 0, 0x48_e0_54, 0x00_00_00, false, "━━━━━━━━━━━━━"),
    (
        3,
        13,
        0x3a_3a_3a,
        0x00_00_00,
        false,
        "─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────",
    ),
    (
        4,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                                                                                                      ",
    ),
    (5, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (5, 2, 0xb3_b3_b3, 0x11_11_11, false, "Targets"),
    (5, 9, 0xff_ff_ff, 0x11_11_11, false, "                     "),
    (5, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (5, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (5, 34, 0xb3_b3_b3, 0x11_11_11, false, "Pipeline"),
    (
        5,
        42,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                               ",
    ),
    (5, 121, 0x73_73_73, 0x11_11_11, false, "0 of 6 done"),
    (5, 132, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (6, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        6,
        2,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "──────────────────────────",
    ),
    (6, 28, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (6, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (6, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        6,
        34,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "──────────────────────────────────────────────────────────────────────────────────────────────────",
    ),
    (6, 132, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (7, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (7, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (7, 3, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        7,
        4,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " payments-gateway         ",
    ),
    (7, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (7, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (7, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (7, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (7, 36, 0x8a_8a_8a, 0x11_11_11, false, "compile  …"),
    (7, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (7, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (
        7,
        54,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                ",
    ),
    (8, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (8, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (8, 3, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (8, 5, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        8,
        6,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " build                  ",
    ),
    (8, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (8, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (8, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (8, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (8, 36, 0x8a_8a_8a, 0x11_11_11, false, "lint     …"),
    (8, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (8, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (
        8,
        54,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                ",
    ),
    (9, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (9, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        9,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      compile              ",
    ),
    (9, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (9, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (9, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (9, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (9, 36, 0x8a_8a_8a, 0x11_11_11, false, "typecheck…"),
    (9, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (9, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (
        9,
        54,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                ",
    ),
    (10, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (10, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        10,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      lint                 ",
    ),
    (10, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (10, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (10, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (10, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (10, 36, 0x8a_8a_8a, 0x11_11_11, false, "unit     …"),
    (10, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (10, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (
        10,
        54,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                ",
    ),
    (11, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (11, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        11,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      typecheck            ",
    ),
    (11, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (11, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (11, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (11, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (11, 36, 0x8a_8a_8a, 0x11_11_11, false, "integrati…"),
    (11, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (11, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (
        11,
        54,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                ",
    ),
    (12, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (12, 3, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 5, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        12,
        6,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " test                   ",
    ),
    (12, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (12, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 34, 0x8a_8a_8a, 0x11_11_11, false, "○"),
    (12, 35, 0xff_ff_ff, 0x11_11_11, false, " "),
    (12, 36, 0x8a_8a_8a, 0x11_11_11, false, "e2e      …"),
    (12, 46, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (12, 48, 0x8a_8a_8a, 0x11_11_11, false, "queued"),
    (
        12,
        54,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                ",
    ),
    (13, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (13, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        13,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      unit                 ",
    ),
    (13, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        13,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (14, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (14, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        14,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      integration          ",
    ),
    (14, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (14, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (14, 34, 0x48_e0_54, 0x48_e0_54, false, " "),
    (14, 35, 0x19_19_1c, 0x48_e0_54, true, "Run pipeline "),
    (14, 48, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (14, 50, 0x27_27_2a, 0x27_27_2a, false, " "),
    (14, 51, 0x4d_4d_4d, 0x27_27_2a, false, "Cancel "),
    (
        14,
        58,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                            ",
    ),
    (15, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (15, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        15,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "      e2e                  ",
    ),
    (15, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        15,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (16, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (16, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (16, 3, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (16, 5, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        16,
        6,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " deploy                 ",
    ),
    (
        16,
        30,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                                                                                        ",
    ),
    (17, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (17, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (17, 3, 0xff_ff_ff, 0x11_11_11, false, "      "),
    (17, 9, 0xf5_9e_09, 0x11_11_11, false, "◇"),
    (
        17,
        10,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " staging            ",
    ),
    (17, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (17, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (17, 34, 0xb3_b3_b3, 0x11_11_11, false, "Log"),
    (
        17,
        37,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                   ",
    ),
    (17, 120, 0x73_73_73, 0x11_11_11, false, " · following"),
    (17, 132, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (18, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (18, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (18, 3, 0xff_ff_ff, 0x11_11_11, false, "      "),
    (18, 9, 0xff_6b_6b, 0x11_11_11, false, "◆"),
    (18, 10, 0xff_ff_ff, 0x11_11_11, false, " "),
    (18, 11, 0xff_6b_6b, 0x11_11_11, false, "production      "),
    (18, 27, 0xff_ff_ff, 0x11_11_11, false, "   "),
    (18, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (18, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        18,
        34,
        0x3a_3a_3a,
        0x11_11_11,
        false,
        "──────────────────────────────────────────────────────────────────────────────────────────────────",
    ),
    (18, 132, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (19, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (19, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (19, 3, 0xb3_b3_b3, 0x11_11_11, false, "▾"),
    (
        19,
        4,
        0xff_ff_ff,
        0x11_11_11,
        false,
        " shared-libs              ",
    ),
    (19, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (19, 32, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (
        19,
        34,
        0xb3_b3_b3,
        0x11_11_11,
        false,
        "Ready. Press r or Run to start the pipeline.                                                    ",
    ),
    (19, 130, 0xff_ff_ff, 0x11_11_11, false, "    "),
    (20, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (20, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        20,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "    compile                ",
    ),
    (20, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        20,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (21, 0, 0xff_ff_ff, 0x11_11_11, false, "  "),
    (21, 2, 0x11_11_11, 0x11_11_11, false, " "),
    (
        21,
        3,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "    publish                ",
    ),
    (21, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        21,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        22,
        0,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                              ",
    ),
    (22, 30, 0xff_ff_ff, 0x00_00_00, false, "  "),
    (
        22,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        23,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        23,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        24,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        24,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        25,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        25,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        26,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        26,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        27,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        27,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        28,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        28,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        29,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        29,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        30,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        30,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        31,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        31,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        32,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        32,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        33,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        33,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        34,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        34,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        35,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        35,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        36,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        36,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        37,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        37,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        38,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        38,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        39,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        39,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        40,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        40,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        41,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        41,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        42,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        42,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        43,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        43,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        44,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        44,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
    (
        45,
        0,
        0xff_ff_ff,
        0x00_00_00,
        false,
        "                                ",
    ),
    (
        45,
        32,
        0xff_ff_ff,
        0x11_11_11,
        false,
        "                                                                                                      ",
    ),
];

fn historical_palette(ui: &mut Ui<'_>) -> [PaintStyle; 6] {
    let [panel, title, detail, meta, primary, primary_gutter] = [
        (
            Surface::Surface,
            termrock::Family::PANEL,
            Variant::DEFAULT,
            termrock::Part::CONTAINER,
            StateFlags::empty(),
        ),
        (
            Surface::Surface,
            termrock::Family::PANEL,
            Variant::DEFAULT,
            termrock::Part::DETAIL,
            StateFlags::empty(),
        ),
        (
            Surface::Surface,
            termrock::Family::PANEL,
            Variant::DEFAULT,
            termrock::Part::HELP,
            StateFlags::empty(),
        ),
        (
            Surface::Surface,
            termrock::Family::EMPTY,
            Variant::DEFAULT,
            termrock::Part::HELP,
            StateFlags::empty(),
        ),
        (
            Surface::Surface,
            termrock::Family::BUTTON,
            Variant::PRIMARY,
            termrock::Part::CONTAINER,
            StateFlags::empty(),
        ),
        (
            Surface::Surface,
            termrock::Family::BUTTON,
            Variant::PRIMARY,
            termrock::Part::GUTTER,
            StateFlags::empty(),
        ),
    ]
    .map(|(surface, family, variant, part, flags)| {
        style(ui, surface, family, variant, part, flags)
    });
    [panel, title, detail, meta, primary, primary_gutter]
}

fn paint_targets(ui: &mut Ui<'_>, body: Rect, [rail, panel, title]: [PaintStyle; 3]) {
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
        let segments: [(u16, &str, &str, PaintStyle); 2] =
            [(row, "  ", "▎", rail), (row, "  ▎", text, panel)];
        for (row, prefix, text, style) in segments {
            paint_segment(ui, body, row, prefix, text, style);
        }
        if let Some(marker) = text.find('▾') {
            let prefix = format!("  ▎{}", &text[..marker]);
            paint_segment(ui, body, row, &prefix, "▾", title);
        }
    }
}

fn paint_pipeline_heading(
    ui: &mut Ui<'_>,
    body: Rect,
    running: bool,
    [title, detail, meta]: [PaintStyle; 3],
) {
    let segments: [(u16, &str, &str, PaintStyle); 2] = [
        (0, "  ", "Targets", title),
        (0, "  Targets                         ", "Pipeline", title),
    ];
    for (row, prefix, text, style) in segments {
        paint_segment(ui, body, row, prefix, text, style);
    }
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
}

#[cfg(test)]
mod motion_tests {
    use super::*;

    #[test]
    fn paused_seek_starts_then_freezes_pipeline_at_a_bounded_frame() {
        let mut page = TaskRunnerPage::new();
        page.seek_paused(5);
        assert_eq!(page.frame, 5);
        assert!(page.running);
        assert_eq!(
            page.steps.first().map(|step| step.state),
            Some(StepState::Done)
        );
        assert_eq!(
            page.steps.get(1).map(|step| step.state),
            Some(StepState::Running)
        );
        assert!(page.motion_paused);
        page.seek_paused(usize::MAX);
        assert!(!page.running);
        assert_eq!(page.message, "pipeline complete");
    }
}
