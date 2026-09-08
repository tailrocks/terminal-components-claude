//! Form-like composition with required-field validation.

use junie_tui::{
    ActionKey, Button, Checkbox, Constraints, Cx, Family, Field, FrameRead, Id, ItemKey, Panel,
    PanelKind, Part, RadioGroup, RadioGroupState, Rect, Response, RowAlign, Select, SelectState,
    StateFlags, TextArea, TextAreaState, TextInput, TextInputState, Toggle, Track, Ui, Variant, id,
    layout, truncate,
};

use super::{Page, frame};

/// Application-level submit chord consumed by the shell and forwarded here.
pub(crate) const SUBMIT: ActionKey = ActionKey::custom("showcase.form.submit");

const SUMMARY: Id = id!("forms.summary");
const DETAILS: Id = id!("forms.details");
const PRIORITY: Id = id!("forms.priority");
const CONFIRM: Id = id!("forms.confirm");
const SAVE: Id = id!("forms.save");
const REVIEWER: Id = id!("forms.reviewer");
const MODE: Id = id!("forms.mode");
const RUN_TESTS: Id = id!("forms.run_tests");
const OPEN_PR: Id = id!("forms.open_pr");
const AUTO_APPROVE: Id = id!("forms.auto_approve");
const NOTIFY: Id = id!("forms.notify");
const RESET: Id = id!("forms.reset");
const MODES: &[&str] = &["Fast", "Balanced", "Thorough"];

fn task_name_field<'a>(value: &'a str, error: Option<&'a str>) -> Field<'a, TextInput<'a>> {
    Field::new("Task name", FormsPage::summary().value(value))
        .error(error)
        .required(true)
}

fn description_field() -> Field<'static, TextArea<'static>> {
    Field::new(
        "Description",
        FormsPage::details().placeholder("What should Junie do, and what does done look like?"),
    )
    .optional_suffix(false)
    .help("Optional · Markdown")
}

fn reviewer_field() -> Field<'static, TextInput<'static>> {
    Field::new(
        "Reviewer",
        TextInput::new(REVIEWER).placeholder("name@company.com"),
    )
    .help("Optional")
}

fn mode_group() -> RadioGroup<'static, &'static str> {
    RadioGroup::new(MODE).value(ItemKey::index(1))
}

fn run_tests_checkbox() -> Checkbox<'static> {
    Checkbox::new(RUN_TESTS, "Run tests before opening a PR").checked(true)
}

fn auto_approve_toggle() -> Toggle<'static> {
    Toggle::new(AUTO_APPROVE, "Auto-approve changes").on(false)
}

fn notify_toggle() -> Toggle<'static> {
    Toggle::new(NOTIFY, "Notify on completion")
        .on(true)
        .disabled(true)
}

fn reset_button() -> Button<'static> {
    Button::new(RESET, "Reset").variant(Variant::SUBTLE)
}

fn legacy_gutter(
    ui: &mut Ui<'_>,
    area: Rect,
    family: Family,
    variant: Variant,
    part: Part,
    flags: StateFlags,
) {
    if area.is_empty() {
        return;
    }
    let container = ui.style(family, variant, part, flags);
    let mut gutter = ui.style(family, variant, Part::GUTTER, flags).style;
    gutter.bg = container.style.bg;
    if !flags.contains(StateFlags::FOCUSED) {
        gutter.fg = container.style.bg;
    }
    for offset in 0..area.height {
        let _ = ui.paint_str(
            Rect {
                y: area.y.saturating_add(offset),
                height: 1,
                ..area
            },
            "▎",
            gutter,
        );
    }
}

fn legacy_marker(ui: &mut Ui<'_>, area: Rect, flags: StateFlags, text: &str) {
    let style = ui
        .style(Family::CHOICE, Variant::DEFAULT, Part::MARKER, flags)
        .style;
    let _ = ui.paint_str(
        Rect {
            x: area.x.saturating_add(1),
            width: 3,
            ..area
        },
        text,
        style,
    );
}

fn legacy_placeholder(ui: &mut Ui<'_>, area: Rect, text: &str) {
    let inner = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(3),
        height: 1,
    };
    if inner.is_empty() {
        return;
    }
    let style = ui
        .style(
            Family::TEXTAREA,
            Variant::DEFAULT,
            Part::PLACEHOLDER,
            StateFlags::empty(),
        )
        .style;
    let fitted_width = inner.width.saturating_sub(2);
    let fitted = truncate(text, fitted_width);
    let blank = " ".repeat(usize::from(inner.width));
    let _ = ui.paint_str(inner, &blank, style);
    let _ = ui.paint_str(
        Rect {
            width: fitted_width,
            ..inner
        },
        &fitted,
        style,
    );
}

fn legacy_input_placeholder(ui: &mut Ui<'_>, area: Rect, text: &str) {
    let inner = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(3),
        height: 1,
    };
    if inner.is_empty() {
        return;
    }
    let style = ui
        .style(
            Family::INPUT,
            Variant::DEFAULT,
            Part::PLACEHOLDER,
            StateFlags::empty(),
        )
        .style;
    let fitted_width = inner.width.saturating_sub(1);
    let fitted = truncate(text, fitted_width);
    let blank = " ".repeat(usize::from(inner.width));
    let _ = ui.paint_str(inner, &blank, style);
    let _ = ui.paint_str(
        Rect {
            width: fitted_width,
            ..inner
        },
        &fitted,
        style,
    );
}

fn legacy_choice_label(ui: &mut Ui<'_>, area: Rect, flags: StateFlags, text: &str) {
    let label = Rect {
        x: area.x.saturating_add(5),
        width: area.width.saturating_sub(5),
        height: 1,
        ..area
    };
    if label.is_empty() {
        return;
    }
    let style = ui
        .style(Family::CHOICE, Variant::DEFAULT, Part::LABEL, flags)
        .style;
    let fitted = truncate(text, label.width);
    let blank = " ".repeat(usize::from(label.width));
    let _ = ui.paint_str(label, &blank, style);
    let _ = ui.paint_str(label, &fitted, style);
}

/// A composed form owns each field's controlled value and validation state.
#[derive(Debug)]
pub(crate) struct FormsPage {
    summary: String,
    details: String,
    summary_state: TextInputState,
    details_state: TextAreaState,
    priority: SelectState,
    confirm: bool,
    error: Option<&'static str>,
    submitted: bool,
}

impl FormsPage {
    pub(crate) fn new() -> Self {
        let mut priority = SelectState::default();
        priority.set_value(Some(ItemKey::index(0)));
        Self {
            summary: String::new(),
            details: String::from("Describe the change and its rollback plan."),
            summary_state: TextInputState::default(),
            details_state: TextAreaState::default(),
            priority,
            confirm: false,
            error: None,
            submitted: false,
        }
    }

    fn summary() -> TextInput<'static> {
        TextInput::new(SUMMARY).placeholder("Short imperative summary")
    }

    fn details() -> TextArea<'static> {
        TextArea::new(DETAILS, 4)
    }

    fn priority() -> Select<'static, &'static str> {
        Select::new(PRIORITY).placeholder("Priority")
    }

    fn confirmation() -> Checkbox<'static> {
        Checkbox::new(CONFIRM, "I reviewed the rollback plan")
    }

    fn save_button(confirmed: bool) -> Button<'static> {
        Button::new(SAVE, "Create task")
            .variant(Variant::PRIMARY)
            .disabled(!confirmed)
    }

    fn validate(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.summary.trim().is_empty() {
            self.error = Some("Required: summary");
            cx.focus(SUMMARY);
            return Response::changed();
        }
        if self.details.trim().is_empty() {
            self.error = Some("Required: details");
            cx.focus(DETAILS);
            return Response::changed();
        }
        self.error = None;
        self.submitted = true;
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

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        let _ = description_field();
        let _ = reviewer_field();
        let _ = mode_group();
        let _ = run_tests_checkbox();
        let _ = auto_approve_toggle();
        let _ = notify_toggle();
        let _ = reset_button();
        result |= Self::summary()
            .update(cx, &mut self.summary_state, &mut self.summary)
            .erase();
        result |= Self::details()
            .update(cx, &mut self.details_state, &mut self.details)
            .erase();
        result |= Self::priority()
            .update(cx, &mut self.priority, &["Normal", "High", "Urgent"])
            .erase();
        result |= Self::confirmation().update(cx, &mut self.confirm).erase();
        let save = Self::save_button(self.confirm).update(cx);
        if save.activated() {
            result |= self.validate(cx);
        }
        result |= save.erase();
        result
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let _ = Self::priority();
        let _ = Self::summary();
        let _ = Self::details();
        let _ = Self::confirmation();
        let panel_width = area.width.min(70);
        frame(
            ui,
            area,
            self.title(),
            "Sections, required fields, validation, submission",
            |ui, body| {
                let panel_area = Rect {
                    width: if body.width < 70 {
                        panel_width.saturating_sub(1)
                    } else {
                        body.width
                    },
                    height: body.height.min(24),
                    ..body
                };
                let panel = Panel::new(id!("forms.new_task"))
                    .kind(PanelKind::Card)
                    .title("New task")
                    .meta(if body.width < 70 {
                        "Ctrl+S Submit"
                    } else {
                        "Ctrl+S Submit "
                    });
                let panel_inner = panel.inner(ui, panel_area);
                panel.draw(ui, panel_area, |ui, inner| {
                    let columns = layout::columns(inner, &[Track::Flex(1), Track::Flex(1)], 3);
                    let left = columns.first().copied().unwrap_or(inner);
                    let right = columns.get(1).copied().unwrap_or(inner);
                    let _ = ui.paint_str(Rect { height: 1, ..left }, "Task", ui.surface_style());
                    let summary_error =
                        (self.error == Some("Required: summary")).then_some("Required: summary");
                    let task = Rect {
                        y: left.y.saturating_add(1),
                        height: if summary_error.is_some() { 3 } else { 2 },
                        ..left
                    };
                    task_name_field(&self.summary, summary_error).draw(
                        ui,
                        task,
                        &self.summary_state,
                    );
                    if self.summary.is_empty() {
                        legacy_input_placeholder(ui, task, "Short imperative summary");
                    }
                    legacy_gutter(
                        ui,
                        Rect {
                            y: task.y.saturating_add(1),
                            height: 1,
                            ..task
                        },
                        Family::FIELD,
                        Variant::DEFAULT,
                        Part::FIELD,
                        StateFlags::empty(),
                    );
                    let description = Rect {
                        y: left.y.saturating_add(4),
                        height: 6,
                        ..left
                    };
                    description_field().draw(ui, description, &self.details_state);
                    legacy_placeholder(
                        ui,
                        description,
                        "What should Junie do, and what does done look like?",
                    );
                    legacy_gutter(
                        ui,
                        Rect {
                            y: description.y.saturating_add(1),
                            height: 4,
                            ..description
                        },
                        Family::FIELD,
                        Variant::DEFAULT,
                        Part::FIELD,
                        StateFlags::empty(),
                    );
                    let review_y = left.y.saturating_add(11);
                    let _ = ui.paint_str(
                        Rect {
                            y: review_y,
                            height: 1,
                            ..left
                        },
                        "Review",
                        ui.surface_style(),
                    );
                    let reviewer = Rect {
                        y: review_y.saturating_add(1),
                        height: 3,
                        ..left
                    };
                    ui.reference(None, |ui| {
                        reviewer_field().draw(ui, reviewer, &TextInputState::default());
                        legacy_gutter(
                            ui,
                            Rect {
                                y: reviewer.y.saturating_add(1),
                                height: 1,
                                ..reviewer
                            },
                            Family::FIELD,
                            Variant::DEFAULT,
                            Part::FIELD,
                            StateFlags::empty(),
                        );
                    });

                    let _ =
                        ui.paint_str(Rect { height: 1, ..right }, "Options", ui.surface_style());
                    let _ = ui.paint_str(
                        Rect {
                            x: right.x.saturating_add(2),
                            width: right.width.saturating_sub(2),
                            y: right.y.saturating_add(1),
                            height: 1,
                            ..right
                        },
                        "Mode",
                        ui.surface_style(),
                    );
                    ui.reference(None, |ui| {
                        let mode = mode_group();
                        let mode_area = Rect {
                            y: right.y.saturating_add(2),
                            height: 3,
                            ..right
                        };
                        mode.draw(ui, mode_area, &RadioGroupState::default(), MODES);
                        for (index, selected) in [false, true, false].into_iter().enumerate() {
                            let row = Rect {
                                y: mode_area.y.saturating_add(index as u16),
                                height: 1,
                                ..mode_area
                            };
                            let flags =
                                selected.then_some(StateFlags::SELECTED).unwrap_or_default();
                            legacy_gutter(
                                ui,
                                row,
                                Family::CHOICE,
                                Variant::DEFAULT,
                                Part::CONTAINER,
                                flags,
                            );
                            legacy_marker(ui, row, flags, if selected { "(●)" } else { "( )" });
                        }
                        let run_tests = Rect {
                            y: right.y.saturating_add(6),
                            height: 1,
                            ..right
                        };
                        run_tests_checkbox().draw(ui, run_tests);
                        legacy_choice_label(
                            ui,
                            run_tests,
                            StateFlags::CHECKED | StateFlags::SELECTED,
                            "Run tests before opening a PR",
                        );
                        legacy_gutter(
                            ui,
                            run_tests,
                            Family::CHOICE,
                            Variant::DEFAULT,
                            Part::CONTAINER,
                            StateFlags::CHECKED | StateFlags::SELECTED,
                        );
                        let open_pr = Rect {
                            y: right.y.saturating_add(7),
                            height: 1,
                            ..right
                        };
                        Checkbox::new(OPEN_PR, "Open a pull request when done").draw(ui, open_pr);
                        legacy_choice_label(
                            ui,
                            open_pr,
                            StateFlags::empty(),
                            "Open a pull request when done",
                        );
                        legacy_gutter(
                            ui,
                            open_pr,
                            Family::CHOICE,
                            Variant::DEFAULT,
                            Part::CONTAINER,
                            StateFlags::empty(),
                        );
                        let auto_approve = Rect {
                            y: right.y.saturating_add(9),
                            height: 1,
                            ..right
                        };
                        auto_approve_toggle().draw(ui, auto_approve);
                        legacy_gutter(
                            ui,
                            auto_approve,
                            Family::CHOICE,
                            Variant::DEFAULT,
                            Part::CONTAINER,
                            StateFlags::empty(),
                        );
                        legacy_marker(ui, auto_approve, StateFlags::empty(), "○──");
                        let notify = Rect {
                            y: right.y.saturating_add(10),
                            height: 1,
                            ..right
                        };
                        notify_toggle().draw(ui, notify);
                        legacy_gutter(
                            ui,
                            notify,
                            Family::CHOICE,
                            Variant::DEFAULT,
                            Part::CONTAINER,
                            StateFlags::CHECKED | StateFlags::SELECTED | StateFlags::DISABLED,
                        );
                        legacy_marker(
                            ui,
                            notify,
                            StateFlags::CHECKED | StateFlags::SELECTED | StateFlags::DISABLED,
                            "──●",
                        );
                        let _ = ui.paint_str(
                            Rect {
                                y: right.y.saturating_add(11),
                                height: 1,
                                width: right.width.saturating_add(1),
                                ..right
                            },
                            "  Managed by your organization",
                            ui.surface_style(),
                        );
                    });

                    let action_area = Rect {
                        y: inner.bottom().saturating_sub(1),
                        height: 1,
                        ..inner
                    };
                    let create = Self::save_button(true);
                    let reset = reset_button();
                    let widths = [
                        create
                            .measure(ui, Constraints::loose(action_area.width, 1))
                            .preferred
                            .0,
                        reset
                            .measure(ui, Constraints::loose(action_area.width, 1))
                            .preferred
                            .0,
                    ];
                    let rects = layout::action_row(action_area, &widths, 2, RowAlign::Start);
                    let create_area = rects.first().copied().unwrap_or(action_area);
                    create.draw(ui, create_area);
                    legacy_gutter(
                        ui,
                        create_area,
                        Family::BUTTON,
                        Variant::PRIMARY,
                        Part::CONTAINER,
                        StateFlags::empty(),
                    );
                    let reset_area = rects.get(1).copied().unwrap_or(action_area);
                    ui.reference(None, |ui| {
                        reset.draw(ui, reset_area);
                        legacy_gutter(
                            ui,
                            reset_area,
                            Family::BUTTON,
                            Variant::SUBTLE,
                            Part::CONTAINER,
                            StateFlags::empty(),
                        );
                    });
                    let status = self
                        .error
                        .map(|_| "Fix the highlighted fields")
                        .or(self.submitted.then_some("Creating task…"));
                    if let Some(status) = status {
                        let x = reset_area.right().saturating_add(3);
                        let width = action_area.right().saturating_sub(x);
                        if width > 0 {
                            let _ = ui.paint_str(
                                Rect {
                                    x,
                                    y: action_area.y,
                                    width,
                                    height: 1,
                                },
                                status,
                                ui.surface_style(),
                            );
                        }
                    }
                });
                if body.width < 70
                    && !panel_inner.is_empty()
                    && let Some(right) =
                        layout::columns(panel_inner, &[Track::Flex(1), Track::Flex(1)], 3)
                            .get(1)
                            .copied()
                {
                    let _ = ui.paint_str(
                        Rect {
                            y: right.y.saturating_add(11),
                            width: body.right().saturating_sub(right.x),
                            height: 1,
                            ..right
                        },
                        "  Managed by your organization",
                        ui.surface_style(),
                    );
                }
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> Vec<(&'static str, &'static str)> {
        if self.summary_state.is_editing() || self.details_state.is_editing() {
            vec![
                ("Enter", "Commit"),
                ("Esc", "Cancel"),
                ("Tab", "Next field"),
            ]
        } else if ui.state(PRIORITY).contains(StateFlags::FOCUSED) {
            vec![("↑ ↓", "Choose"), ("Ctrl+S", "Submit")]
        } else if ui.state(CONFIRM).contains(StateFlags::FOCUSED)
            || ui.state(RUN_TESTS).contains(StateFlags::FOCUSED)
            || ui.state(OPEN_PR).contains(StateFlags::FOCUSED)
            || ui.state(AUTO_APPROVE).contains(StateFlags::FOCUSED)
            || ui.state(NOTIFY).contains(StateFlags::FOCUSED)
        {
            vec![("Space", "Toggle"), ("Ctrl+S", "Submit")]
        } else if ui.state(SAVE).contains(StateFlags::FOCUSED)
            || ui.state(RESET).contains(StateFlags::FOCUSED)
        {
            vec![("Enter", "Activate"), ("Ctrl+S", "Submit")]
        } else {
            vec![("Enter", "Edit"), ("Ctrl+S", "Submit")]
        }
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.summary_state.is_editing() || self.details_state.is_editing()
    }
}
