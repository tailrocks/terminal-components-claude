//! Form-like composition with required-field validation.

use std::time::Duration;

use termrock::{
    ActionKey, Button, Checkbox, Cx, Family, FgStep, Field, FrameRead, Id, ItemKey, Modifier,
    Moment, Panel, PanelKind, Part, RadioGroup, RadioGroupAction, RadioGroupState, Rect, Response,
    Role, RowAlign, StateFlags, Status, StylePatch, TextArea, TextAreaState, TextInput,
    TextInputState, Toggle, Ui, Variant, id, layout, truncate,
};

use super::{Page, PageStatus, PageUpdate, frame};

/// Application-level submit chord consumed by the shell and forwarded here.
pub const SUBMIT: ActionKey = ActionKey::application("showcase.form.submit");

const SUMMARY: Id = id!("forms.summary");
const DETAILS: Id = id!("forms.details");
const REVIEWER: Id = id!("forms.reviewer");
const MODE: Id = id!("forms.mode");
const RUN_TESTS: Id = id!("forms.run_tests");
const OPEN_PR: Id = id!("forms.open_pr");
const AUTO_APPROVE: Id = id!("forms.auto_approve");
const NOTIFY: Id = id!("forms.notify");
const SAVE: Id = id!("forms.save");
const RESET: Id = id!("forms.reset");

const MODES: &[&str] = &["Fast", "Balanced", "Thorough"];

/// Pre-success busy window: the tag holds `Submit::Busy` for 23 ticks at
/// the 80 ms live-tick interval before publishing `Task created ✓`.
const SUBMIT_BUSY: Duration = Duration::from_millis(23 * 80);

fn legacy_gutter(ui: &mut Ui<'_>, area: Rect, variant: Variant, flags: StateFlags) {
    if area.is_empty() {
        return;
    }
    let container = ui.style(Family::BUTTON, variant, Part::CONTAINER, flags);
    let mut gutter = ui.style(Family::BUTTON, variant, Part::GUTTER, flags).style;
    gutter = gutter
        .with_bg_from(container.style)
        .remove_modifier(Modifier::BOLD);
    if flags.contains(StateFlags::PRESSED) {
        match variant {
            Variant::PRIMARY => {
                gutter =
                    gutter.patch(ui.paint_patch(&StylePatch::new().set_bg(Role::AccentPressed)));
            }
            Variant::DEFAULT | Variant::SECONDARY | Variant::SUBTLE => {
                gutter = gutter
                    .patch(ui.paint_patch(&StylePatch::new().set_bg(Role::Fg(FgStep::Primary))));
            }
            _ => {}
        }
    }
    let _ = ui.paint_str(Rect { width: 1, ..area }, " ", gutter);
}

/// A composed form owns each field's controlled value and validation state.
#[derive(Debug)]
pub struct FormsPage {
    summary: String,
    details: String,
    reviewer: String,
    summary_state: TextInputState,
    details_state: TextAreaState,
    reviewer_state: TextInputState,
    mode_state: RadioGroupState,
    mode_value: ItemKey,
    run_tests: bool,
    open_pr: bool,
    auto_approve: bool,
    notify: bool,
    error: Option<&'static str>,
    submitted: bool,
    attempted: bool,
    pending_status: Option<String>,
    busy_until: Option<Moment>,
}

impl FormsPage {
    pub fn new() -> Self {
        let mut mode_state = RadioGroupState::default();
        mode_state.set_cursor(1, ItemKey::index(1));
        Self {
            summary: String::new(),
            details: String::new(),
            reviewer: String::new(),
            summary_state: TextInputState::default(),
            details_state: TextAreaState::default(),
            reviewer_state: TextInputState::default(),
            mode_state,
            mode_value: ItemKey::index(1),
            run_tests: true,
            open_pr: false,
            auto_approve: false,
            notify: true,
            error: None,
            submitted: false,
            attempted: false,
            pending_status: None,
            busy_until: None,
        }
    }

    fn busy(&self) -> bool {
        self.busy_until.is_some()
    }

    fn do_reset(&mut self, cx: &mut Cx<'_>) {
        self.summary.clear();
        self.details.clear();
        self.reviewer.clear();
        self.summary_state = TextInputState::default();
        self.details_state = TextAreaState::default();
        self.reviewer_state = TextInputState::default();
        let mut mode_state = RadioGroupState::default();
        mode_state.set_cursor(1, ItemKey::index(1));
        self.mode_state = mode_state;
        self.mode_value = ItemKey::index(1);
        self.run_tests = true;
        self.open_pr = false;
        self.auto_approve = false;
        self.notify = true;
        self.error = None;
        self.submitted = false;
        self.attempted = false;
        self.pending_status = Some("Form reset".to_string());
        self.busy_until = None;
        cx.focus(SUMMARY);
    }

    fn validate(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.busy() {
            return Response::ignored();
        }
        self.attempted = true;
        if self.summary.trim().is_empty() {
            self.error = Some("Required");
            self.pending_status = Some("Fix the highlighted fields".to_string());
            cx.focus(SUMMARY);
            return Response::changed();
        }
        self.error = None;
        self.busy_until = Some(cx.now().saturating_add(SUBMIT_BUSY));
        self.pending_status = Some("Creating task…".to_string());
        Response::changed()
    }
}

impl Default for FormsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for FormsPage {
    fn title(&self) -> &'static str {
        "Forms"
    }

    fn command(&mut self, cx: &mut Cx<'_>, action: ActionKey) -> Response<()> {
        if action == SUBMIT {
            self.validate(cx)
        } else {
            Response::ignored()
        }
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        if cx.update_cause() == termrock::UpdateCause::Tick
            && self.busy_until.is_some_and(|deadline| cx.now() >= deadline)
        {
            self.busy_until = None;
            self.submitted = true;
            self.pending_status = Some("Task created ✓".to_string());
            response = Response::changed();
        }
        response |= TextInput::new(SUMMARY)
            .placeholder("Short imperative summary")
            .update(cx, &mut self.summary_state, &mut self.summary)
            .erase();
        response |= TextArea::new(DETAILS, 4)
            .placeholder("What should Junie do, and what does done look like?")
            .update(cx, &mut self.details_state, &mut self.details)
            .erase();
        response |= TextInput::new(REVIEWER)
            .placeholder("name@company.com")
            .update(cx, &mut self.reviewer_state, &mut self.reviewer)
            .erase();
        response |= RadioGroup::new(MODE)
            .value(self.mode_value)
            .update(cx, &mut self.mode_state, MODES)
            .on_action(|action| {
                let RadioGroupAction::Chose(k) = action;
                self.mode_value = k;
            })
            .erase();
        response |= Checkbox::new(RUN_TESTS, "Run tests before opening a PR")
            .update(cx, &mut self.run_tests)
            .erase();
        response |= Checkbox::new(OPEN_PR, "Open a pull request when done")
            .update(cx, &mut self.open_pr)
            .erase();
        response |= Toggle::new(AUTO_APPROVE, "Auto-approve changes")
            .update(cx, &mut self.auto_approve)
            .erase();
        response |= Toggle::new(NOTIFY, "Notify on completion")
            .disabled(true)
            .update(cx, &mut self.notify)
            .erase();
        let mut save_button = Button::new(SAVE, "Create task").variant(Variant::PRIMARY);
        if self.busy() {
            save_button = save_button.status(Status::Busy);
        }
        let save = save_button.update(cx);
        if save.activated() {
            response |= self.validate(cx);
        }
        response |= save.erase();
        let reset = Button::new(RESET, "Reset")
            .variant(Variant::SUBTLE)
            .update(cx);
        if reset.activated() {
            self.do_reset(cx);
            response |= Response::changed();
        }
        response |= reset.erase();
        if cx.top_layer() == termrock::LayerId::PAGE
            && let Some(deadline) = self.busy_until
        {
            cx.request_repaint_at(deadline);
        }

        let status = self.pending_status.take().map(PageStatus);
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Sections, required fields, validation, submission",
            |ui, body| {
                self.draw_task_card(ui, body);
            },
        );
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.summary_state.is_editing()
            || self.details_state.is_editing()
            || self.reviewer_state.is_editing()
        {
            &[
                ("Enter", "Commit"),
                ("Esc", "Cancel"),
                ("Tab", "Next field"),
            ]
        } else {
            &[("Enter", "Edit"), ("Ctrl+S", "Submit")]
        }
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.summary_state.is_editing()
            || self.details_state.is_editing()
            || self.reviewer_state.is_editing()
    }
}

impl FormsPage {
    fn draw_task_card(&self, ui: &mut Ui<'_>, card_body: Rect) {
        let panel_area = Rect {
            height: card_body.height.min(24),
            ..card_body
        };
        let panel_patches = [(
            Part::DETAIL,
            StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
        )];
        let panel = Panel::new(id!("forms.new_task"))
            .kind(PanelKind::Card)
            .title("New task")
            .meta("Ctrl+S Submit")
            .patch_part(&panel_patches);
        panel.draw(ui, panel_area, |ui, inner| {
            if inner.height < 11 || inner.width < 40 {
                return;
            }
            let compact = inner.height < 17;
            let body = Rect::new(
                inner.x,
                inner.y,
                inner.width,
                inner.height.saturating_sub(2),
            );
            let left_w = body.width / 2 - 2;
            let gap = 4;
            let (left, right) = if body.width < left_w + gap + 20 {
                let h = body.height / 2;
                (
                    Rect::new(body.x, body.y, body.width, h),
                    Rect::new(body.x, body.y + h, body.width, body.height - h),
                )
            } else {
                (
                    Rect::new(body.x, body.y, left_w, body.height),
                    Rect::new(
                        body.x + left_w + gap,
                        body.y,
                        body.width - left_w - gap,
                        body.height,
                    ),
                )
            };

            self.draw_task_fields(ui, left, compact);
            self.draw_options_section(ui, right, compact);
            self.draw_actions(ui, inner);
        });
    }

    fn draw_task_fields(&self, ui: &mut Ui<'_>, left: Rect, compact: bool) {
        let mut y = left.y;
        let faint_style = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));

        let field_patch = [(Part::MARKER, StylePatch::new().set_fg(Role::Accent))];

        if !compact {
            let _ = ui.paint_str(
                Rect {
                    x: left.x,
                    y,
                    width: left.width,
                    height: 1,
                },
                "Task",
                faint_style,
            );
            y += 1;
        }

        let task = Rect {
            x: left.x,
            y,
            width: left.width,
            height: 3,
        };
        let err_placeholder = if self.error.is_some() {
            let error_w = task.width.saturating_sub(5);
            Some(truncate("Short imperative summary", error_w))
        } else {
            None
        };
        let mut input = TextInput::new(SUMMARY)
            .placeholder("Short imperative summary")
            .value(&self.summary);
        if let Some(ref p) = err_placeholder {
            input = input.placeholder(p);
        }
        Field::new("Task name", input)
            .patch_part(&field_patch)
            .error(self.error)
            .required(true)
            .draw(ui, task, &self.summary_state);
        if self.error.is_some() && task.width >= 2 {
            let field_style = ui
                .style(
                    Family::FIELD,
                    Variant::DEFAULT,
                    Part::FIELD,
                    StateFlags::empty(),
                )
                .style;
            let error_style = field_style
                .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Danger)))
                .add_modifier(Modifier::BOLD);
            let _ = ui.paint_str(
                Rect {
                    x: task.right().saturating_sub(2),
                    y: task.y.saturating_add(1),
                    width: 1,
                    height: 1,
                },
                "!",
                error_style,
            );
        }
        y += 3;

        let desc_h = if compact { 3 } else { 6 };
        let desc_rows = if compact { 1 } else { 4 };
        let description = Rect {
            x: left.x,
            y,
            width: left.width,
            height: desc_h,
        };
        Field::new(
            "Description",
            TextArea::new(DETAILS, desc_rows)
                .placeholder("What should Junie do, and what does done look like?")
                .value(&self.details),
        )
        .patch_part(&field_patch)
        .optional_suffix(false)
        .help("Optional · Markdown")
        .draw(ui, description, &self.details_state);
        y += desc_h;

        if !compact {
            y += 1;
            let _ = ui.paint_str(
                Rect {
                    x: left.x,
                    y,
                    width: left.width,
                    height: 1,
                },
                "Review",
                faint_style,
            );
            y += 1;
        }

        let reviewer = Rect {
            x: left.x,
            y,
            width: left.width,
            height: 3,
        };
        Field::new(
            "Reviewer",
            TextInput::new(REVIEWER)
                .placeholder("name@company.com")
                .value(&self.reviewer),
        )
        .patch_part(&field_patch)
        .help("Optional")
        .draw(ui, reviewer, &self.reviewer_state);
    }

    fn draw_options_section(&self, ui: &mut Ui<'_>, right: Rect, compact: bool) {
        let mut y = right.y;
        let bg = ui.surface_style();
        let blank_gutter = bg.with_fg_from_bg(bg);
        let faint_style =
            bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));
        let muted_style =
            bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
        let secondary_style =
            bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
        let primary_style =
            bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
        let accent_style = bg.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));

        if !compact {
            let _ = ui.paint_str(
                Rect {
                    x: right.x,
                    y,
                    width: right.width,
                    height: 1,
                },
                "Options",
                faint_style,
            );
            y += 1;
        }

        // Mode label
        let _ = ui.paint_str(
            Rect {
                x: right.x.saturating_add(2),
                y,
                width: right.width.saturating_sub(2),
                height: 1,
            },
            "Mode",
            secondary_style,
        );
        y += 1;

        // Mode options
        for (i, opt) in MODES.iter().enumerate() {
            let row = Rect {
                x: right.x,
                y,
                width: right.width,
                height: 1,
            };
            let _ = ui.paint_str(Rect::new(row.x, row.y, 1, 1), " ", blank_gutter);
            let on = i == 1; // "Balanced" is selected
            let mark = if on { "(●)" } else { "( )" };
            let mark_style = if on { accent_style } else { muted_style };
            let _ = ui.paint_str(
                Rect {
                    x: row.x.saturating_add(1),
                    y: row.y,
                    width: 3,
                    height: 1,
                },
                mark,
                mark_style,
            );
            let _ = ui.paint_str(
                Rect {
                    x: row.x.saturating_add(5),
                    y: row.y,
                    width: row.width.saturating_sub(5),
                    height: 1,
                },
                &truncate(opt, row.width.saturating_sub(5)),
                primary_style,
            );
            y += 1;
        }

        if !compact {
            y += 1;
        }

        // Run tests
        let run_tests_row = Rect {
            x: right.x,
            y,
            width: right.width,
            height: 1,
        };
        let _ = ui.paint_str(
            Rect::new(run_tests_row.x, run_tests_row.y, 1, 1),
            " ",
            blank_gutter,
        );
        let _ = ui.paint_str(
            Rect {
                x: run_tests_row.x.saturating_add(1),
                y: run_tests_row.y,
                width: 3,
                height: 1,
            },
            "[✓]",
            accent_style,
        );
        let _ = ui.paint_str(
            Rect {
                x: run_tests_row.x.saturating_add(5),
                y: run_tests_row.y,
                width: run_tests_row.width.saturating_sub(5),
                height: 1,
            },
            &truncate(
                "Run tests before opening a PR",
                run_tests_row.width.saturating_sub(6),
            ),
            primary_style,
        );
        y += 1;

        // Open PR
        let open_pr_row = Rect {
            x: right.x,
            y,
            width: right.width,
            height: 1,
        };
        let _ = ui.paint_str(
            Rect::new(open_pr_row.x, open_pr_row.y, 1, 1),
            " ",
            blank_gutter,
        );
        let _ = ui.paint_str(
            Rect {
                x: open_pr_row.x.saturating_add(1),
                y: open_pr_row.y,
                width: 3,
                height: 1,
            },
            "[ ]",
            muted_style,
        );
        let _ = ui.paint_str(
            Rect {
                x: open_pr_row.x.saturating_add(5),
                y: open_pr_row.y,
                width: open_pr_row.width.saturating_sub(5),
                height: 1,
            },
            &truncate(
                "Open a pull request when done",
                open_pr_row.width.saturating_sub(6),
            ),
            primary_style,
        );
        y += 1;

        if !compact {
            y += 1;
        }

        // Auto approve
        let auto_row = Rect {
            x: right.x,
            y,
            width: right.width,
            height: 1,
        };
        let _ = ui.paint_str(Rect::new(auto_row.x, auto_row.y, 1, 1), " ", blank_gutter);
        let _ = ui.paint_str(
            Rect {
                x: auto_row.x.saturating_add(1),
                y: auto_row.y,
                width: 3,
                height: 1,
            },
            "○──",
            muted_style,
        );
        let auto_label = "Auto-approve changes";
        let _ = ui.paint_str(
            Rect {
                x: auto_row.x.saturating_add(5),
                y: auto_row.y,
                width: auto_row.width.saturating_sub(5),
                height: 1,
            },
            &truncate(auto_label, auto_row.width.saturating_sub(5)),
            primary_style,
        );
        let state_offset = 6 + auto_label.len() as u16;
        if state_offset + 3 < auto_row.width {
            let _ = ui.paint_str(
                Rect {
                    x: auto_row.x.saturating_add(state_offset),
                    y: auto_row.y,
                    width: 3,
                    height: 1,
                },
                "off",
                muted_style,
            );
        }
        y += 1;

        // Notify (disabled, on)
        let notify_row = Rect {
            x: right.x,
            y,
            width: right.width,
            height: 1,
        };
        let mut disabled_style = faint_style;
        if ui.theme().capability.color == termrock::ColorLevel::Mono {
            disabled_style = disabled_style.add_modifier(Modifier::DIM);
        }
        ui.fill(notify_row, disabled_style);
        let disabled_gutter = disabled_style.with_fg_from_bg(disabled_style);
        let _ = ui.paint_str(
            Rect::new(notify_row.x, notify_row.y, 1, 1),
            " ",
            disabled_gutter,
        );
        let _ = ui.paint_str(
            Rect {
                x: notify_row.x.saturating_add(1),
                y: notify_row.y,
                width: 3,
                height: 1,
            },
            "──●",
            disabled_style,
        );
        let notify_label = "Notify on completion";
        let _ = ui.paint_str(
            Rect {
                x: notify_row.x.saturating_add(5),
                y: notify_row.y,
                width: notify_row.width.saturating_sub(5),
                height: 1,
            },
            &truncate(notify_label, notify_row.width.saturating_sub(5)),
            disabled_style,
        );
        let notify_state_offset = 6 + notify_label.len() as u16;
        if notify_state_offset + 3 < notify_row.width {
            let _ = ui.paint_str(
                Rect {
                    x: notify_row.x.saturating_add(notify_state_offset),
                    y: notify_row.y,
                    width: 2,
                    height: 1,
                },
                "on",
                disabled_style,
            );
        }
        y += 1;

        // Managed by your organization
        let _ = ui.paint_str(
            Rect {
                x: right.x.saturating_add(2),
                y,
                width: right.width.saturating_sub(2),
                height: 1,
            },
            &truncate(
                "Managed by your organization",
                right.width.saturating_sub(2),
            ),
            faint_style,
        );
    }

    fn draw_actions(&self, ui: &mut Ui<'_>, inner: Rect) {
        let action_area = Rect {
            y: inner.bottom().saturating_sub(1),
            height: 1,
            ..inner
        };
        let mut create = Button::new(SAVE, "Create task").variant(Variant::PRIMARY);
        if self.busy() {
            create = create.status(Status::Busy);
        }
        let reset = Button::new(RESET, "Reset").variant(Variant::SUBTLE);
        let widths = [13, 7];
        let rects = layout::action_row(action_area, &widths, 2, RowAlign::Start);
        let create_area = rects.first().copied().unwrap_or(action_area);
        create.draw(ui, create_area);
        legacy_gutter(ui, create_area, Variant::PRIMARY, ui.state(SAVE));
        let reset_area = rects.get(1).copied().unwrap_or(action_area);
        reset.draw(ui, reset_area);
        legacy_gutter(ui, reset_area, Variant::SUBTLE, ui.state(RESET));

        let status = if self.busy() {
            Some(("Creating task…", Role::Fg(FgStep::Secondary)))
        } else if self.submitted {
            Some(("Task created ✓", Role::Accent))
        } else if self.error.is_some() {
            Some(("Fix the highlighted fields", Role::Danger))
        } else {
            None
        };
        if let Some((status, role)) = status {
            let x = reset_area.right().saturating_add(3);
            let width = action_area.right().saturating_sub(x);
            if width >= termrock::width(status) {
                let style = ui
                    .surface_style()
                    .patch(ui.paint_patch(&StylePatch::new().set_fg(role)));
                let _ = ui.paint_str(
                    Rect {
                        x,
                        y: action_area.y,
                        width,
                        height: 1,
                    },
                    status,
                    style,
                );
            }
        }
    }
}
