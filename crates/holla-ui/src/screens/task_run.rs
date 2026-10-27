use crate::domain::activity::Activity;
use crate::screens::activity::ActivityProjection;
use crate::sim::{
    activity_supervisor::ActivityExecutionId,
    task_run::{PromptId, TaskRun, TaskRunId, TaskRunState},
};
use termrock::{
    BlurPolicy, Button, Cx, FgStep, FrameRead, Id, ItemKey, Modifier, Rect, Response, Role,
    StylePatch, TextAction, TextInput, TextInputState, TextSection, TypingPolicy, Ui,
};

pub(crate) enum Event {
    Answer {
        route_id: u64,
        run_id: TaskRunId,
        prompt_id: PromptId,
        answer: String,
    },
    Cancel {
        route_id: u64,
        run_id: TaskRunId,
        prompt_id: PromptId,
    },
    Stop {
        route_id: u64,
        run_id: TaskRunId,
        execution_id: ActivityExecutionId,
    },
}

struct RetiredInputOwner {
    id: Id,
    input: TextInputState,
    answer: String,
}

pub(crate) struct TaskRunScreen {
    route_id: u64,
    run_id: TaskRunId,
    activity_id: u32,
    execution_id: Option<ActivityExecutionId>,
    observed_prompt: Option<PromptId>,
    input: TextInputState,
    answer: String,
    focus_pending: bool,
    retired_inputs: Vec<RetiredInputOwner>,
}

impl TaskRunScreen {
    pub(crate) fn new(route_id: u64, run_id: TaskRunId, activity_id: u32) -> Self {
        Self {
            route_id,
            run_id,
            activity_id,
            execution_id: None,
            observed_prompt: None,
            input: TextInputState::default(),
            answer: String::new(),
            focus_pending: false,
            retired_inputs: Vec::new(),
        }
    }

    pub(crate) const fn route_id(&self) -> u64 {
        self.route_id
    }
    pub(crate) const fn run_id(&self) -> TaskRunId {
        self.run_id
    }
    pub(crate) const fn activity_id(&self) -> u32 {
        self.activity_id
    }

    pub(crate) const fn execution_id(&self) -> Option<ActivityExecutionId> {
        self.execution_id
    }

    pub(crate) fn bind_execution(&mut self, execution_id: ActivityExecutionId) {
        self.execution_id = Some(execution_id);
    }

    fn stop_owner_id(&self) -> Option<Id> {
        self.execution_id.map(|execution_id| {
            Id::root("holla.task-run.stop")
                .item(ItemKey::pair(self.route_id, self.run_id.get()))
                .item(ItemKey::pair(
                    u64::from(self.activity_id),
                    execution_id.generation(),
                ))
        })
    }

    pub(crate) const fn current_prompt_id(&self) -> Option<PromptId> {
        self.observed_prompt
    }

    pub(crate) fn current_input_id(&self) -> Option<Id> {
        self.observed_prompt
            .map(|prompt| Self::input_id_for(self.route_id, self.run_id.get(), prompt.get()))
    }

    pub(crate) fn input_id_for(route_id: u64, run_id: u64, prompt_id: u64) -> Id {
        Id::root("holla.task-run.answer")
            .item(ItemKey::pair(route_id, run_id))
            .item(ItemKey::num(prompt_id))
    }

    pub(crate) fn input_area(&self, area: Rect) -> Option<Rect> {
        Self::input_area_for(area, self.observed_prompt.is_some())
    }

    pub(crate) fn input_area_for(area: Rect, prompt_open: bool) -> Option<Rect> {
        if !prompt_open || area.width == 0 || area.height < 7 {
            return None;
        }
        Some(Rect::new(
            area.x,
            area.bottom().saturating_sub(2),
            area.width,
            1,
        ))
    }

    pub(crate) fn update(
        &mut self,
        cx: &mut Cx<'_>,
        run: &TaskRun,
        area: Rect,
        allow_input: bool,
        stop_enabled: bool,
        stop_label: &'static str,
    ) -> (Response<()>, Option<Event>) {
        let prompt = run.active_prompt();
        let prompt_id = prompt.map(|prompt| prompt.id());
        if prompt_id != self.observed_prompt {
            self.retain_current_input();
            self.observed_prompt = prompt_id;
            self.answer.clear();
            self.input = if prompt.is_some_and(|value| value.is_secret()) {
                TextInputState::sensitive()
            } else {
                TextInputState::default()
            };
            self.focus_pending = prompt.is_some();
            if allow_input
                && Self::input_area_for(area, true).is_some_and(|input_area| !input_area.is_empty())
                && let Some(id) = self.current_input_id()
            {
                cx.focus(id);
                self.focus_pending = false;
            }
            let response = self.poll_retired_inputs(cx);
            let (stop_response, stop_event) =
                self.update_stop(cx, allow_input, stop_enabled, stop_label);

            return (response | stop_response | Response::changed(), stop_event);
        }

        let mut response = self.poll_retired_inputs(cx);
        let Some(prompt) = prompt else {
            let (stop_response, stop_event) =
                self.update_stop(cx, allow_input, stop_enabled, stop_label);
            return (response | stop_response, stop_event);
        };
        let Some(id) = self.current_input_id() else {
            return (response, None);
        };
        let expected_area = Self::input_area_for(area, true);
        if self.focus_pending
            && allow_input
            && expected_area.is_some_and(|input_area| !input_area.is_empty())
        {
            cx.focus(id);
            self.focus_pending = false;
        }
        let presented =
            allow_input && expected_area.is_some_and(|expected| cx.area(id) == Some(expected));
        let lifecycle_focus_initialization = !presented
            && allow_input
            && expected_area.is_some_and(|input_area| !input_area.is_empty())
            && cx.update_cause() == termrock::UpdateCause::Settle
            && cx.state(id).contains(termrock::StateFlags::FOCUSED)
            && has_only_focus_initialization_intents(cx, id);
        let mut input_response = answer_input(id)
            .disabled(!(presented || lifecycle_focus_initialization))
            .update(cx, &mut self.input, &mut self.answer);
        let action = input_response.take_action();
        response |= input_response.erase();
        let event = if presented {
            match action {
                Some(TextAction::Committed) => Some(Event::Answer {
                    route_id: self.route_id,
                    run_id: self.run_id,
                    prompt_id: prompt.id(),
                    answer: std::mem::take(&mut self.answer),
                }),
                Some(TextAction::Cancelled) => Some(Event::Cancel {
                    route_id: self.route_id,
                    run_id: self.run_id,
                    prompt_id: prompt.id(),
                }),
                Some(TextAction::Changed | TextAction::MoveNext | TextAction::MovePrev) | None => {
                    None
                }
            }
        } else {
            None
        };
        let (stop_response, stop_event) =
            self.update_stop(cx, allow_input, stop_enabled, stop_label);
        let event = event.or(stop_event);
        (response | stop_response, event)
    }

    fn update_stop(
        &self,
        cx: &mut Cx<'_>,
        allow_input: bool,
        stop_enabled: bool,
        stop_label: &'static str,
    ) -> (Response<()>, Option<Event>) {
        let Some((id, execution_id)) = self.stop_owner_id().zip(self.execution_id) else {
            return (Response::ignored(), None);
        };
        let response = Button::new(id, stop_label)
            .disabled(!allow_input || !stop_enabled)
            .update(cx);
        let activated = response.activated();
        let event = activated.then_some(Event::Stop {
            route_id: self.route_id,
            run_id: self.run_id,
            execution_id,
        });
        (response.erase(), event)
    }

    pub(crate) fn poll_covered(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let (stop_response, _) = self.update_stop(cx, false, false, "Stopping…");
        let mut response = stop_response | self.poll_retired_inputs(cx);
        if let Some(id) = self.current_input_id() {
            let mut update =
                answer_input(id)
                    .disabled(true)
                    .update(cx, &mut self.input, &mut self.answer);
            let _ = update.take_action();
            response |= update.erase();
        }
        response
    }

    pub(crate) fn retire(&mut self, cx: &mut Cx<'_>) {
        self.retain_current_input();
        self.observed_prompt = None;
        self.answer.clear();
        self.input = TextInputState::default();
        self.focus_pending = false;
        let _ = self.poll_retired_inputs(cx);
    }

    pub(crate) fn poll_retired(&mut self, cx: &mut Cx<'_>) -> (Response<()>, bool) {
        let (stop_response, _) = self.update_stop(cx, false, false, "Stopping…");
        let response = stop_response | self.poll_retired_inputs(cx);
        let stop_ready = self.stop_owner_id().is_none_or(|id| {
            cx.area(id).is_none()
                && !cx.state(id).contains(termrock::StateFlags::FOCUSED)
                && cx.capture_owner() != Some(id)
                && cx.intents(id).next().is_none()
                && cx.top_layer() == termrock::LayerId::PAGE
        });
        (response, self.retired_inputs.is_empty() && stop_ready)
    }

    fn retain_current_input(&mut self) {
        let Some(id) = self.current_input_id() else {
            self.answer.clear();
            return;
        };
        self.retired_inputs.push(RetiredInputOwner {
            id,
            input: std::mem::take(&mut self.input),
            answer: std::mem::take(&mut self.answer),
        });
    }

    fn poll_retired_inputs(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        self.retired_inputs.retain_mut(|owner| {
            let had_routed_intents = cx.intents(owner.id).next().is_some();
            let mut update = answer_input(owner.id).disabled(true).update(
                cx,
                &mut owner.input,
                &mut owner.answer,
            );
            let _ = update.take_action();
            response |= update.erase();
            owner.input.cancel();
            owner.answer.clear();

            let ready = !had_routed_intents
                && !owner.input.is_editing()
                && cx.area(owner.id).is_none()
                && !cx.state(owner.id).contains(termrock::StateFlags::FOCUSED)
                && cx.capture_owner() != Some(owner.id)
                && cx.top_layer() == termrock::LayerId::PAGE;
            if !ready {
                cx.request_repaint();
            }
            !ready
        });
        response
    }

    pub(crate) fn draw(
        &self,
        activity: &Activity,
        projection: &ActivityProjection,
        run: &TaskRun,
        now_ms: i64,
        ui: &mut Ui<'_>,
        area: Rect,
    ) {
        if area.is_empty() {
            return;
        }
        let primary = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        );
        let muted = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
        let faint = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint)));
        render_line(
            ui,
            area,
            0,
            "Modeled task run · no process executed",
            primary,
        );
        let activity_label = format!(
            "Activity #{} · {} · {}",
            self.activity_id, activity.name, activity.scope
        );
        render_line(ui, area, 1, &activity_label, muted);
        let state = match run.state() {
            TaskRunState::Running => "running",
            TaskRunState::Waiting => "waiting",
            TaskRunState::Completed => "completed",
            TaskRunState::Cancelled => "cancelled",
            TaskRunState::TimeExhausted => "time exhausted",
            _ => "updated",
        };
        let history = format!(
            "Activity history: {} · age {} ms · TaskRun: {}",
            activity.state.label(),
            now_ms.saturating_sub(activity.started_ms).max(0),
            state,
        );
        render_line(ui, area, 2, &history, muted);
        let duration = format!(
            "TaskRun active duration: {} ms · prompt wait excluded",
            run.duration_ms()
        );
        render_line(ui, area, 3, &duration, muted);
        let execution = projection.stop_grace_ms.map_or_else(
            || format!("Execution: {}", projection.state_label),
            |elapsed| format!("Execution: stopping · stop grace {elapsed} ms"),
        );
        render_line(ui, area, 4, &execution, muted);
        if let Some(id) = self.stop_owner_id() {
            Button::new(
                id,
                if projection.stop_enabled {
                    "Stop"
                } else if projection.state_label == "stopping" {
                    "Stopping…"
                } else {
                    "Stopped"
                },
            )
            .disabled(!projection.stop_enabled)
            .draw(
                ui,
                Rect::new(
                    area.right().saturating_sub(22),
                    area.y.saturating_add(4),
                    area.width.min(22),
                    1,
                ),
            );
        }

        let input_area = Self::input_area_for(area, run.active_prompt().is_some());
        let reserved_rows = if input_area.is_some() { 4 } else { 0 };
        let output_end = area.height.saturating_sub(reserved_rows);
        let mut row = 5;
        for line in &projection.lines {
            if row >= output_end {
                break;
            }
            render_line(ui, area, row, line, muted);
            row = row.saturating_add(1);
        }
        if row < output_end {
            render_line(
                ui,
                area,
                row,
                "Esc returns · Ctrl-C cancels only while a prompt is open",
                faint,
            );
        }

        let Some(prompt) = run.active_prompt() else {
            return;
        };
        let Some(input_area) = input_area else {
            return;
        };
        render_line(
            ui,
            area,
            area.height.saturating_sub(3),
            prompt.text(),
            primary,
        );
        if self.observed_prompt == Some(prompt.id())
            && let Some(id) = self.current_input_id()
        {
            answer_input(id)
                .value(&self.answer)
                .draw(ui, input_area, &self.input);
        }
        if prompt.is_secret() {
            render_line(
                ui,
                area,
                area.height.saturating_sub(1),
                "Secret answer · not retained in Activity history",
                faint,
            );
        }
    }

    pub(crate) fn draw_stale(ui: &mut Ui<'_>, area: Rect) {
        let style = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
        render_line(
            ui,
            area,
            0,
            "Task run context changed · return to Home and reopen it",
            style,
        );
    }
}

fn render_line(ui: &mut Ui<'_>, area: Rect, row: u16, text: &str, style: termrock::PaintStyle) {
    if area.width == 0 || row >= area.height {
        return;
    }
    TextSection::new(text).inset(0).title_style(style).render(
        ui,
        Rect::new(area.x, area.y.saturating_add(row), area.width, 1),
    );
}

fn answer_input(id: Id) -> TextInput<'static> {
    TextInput::new(id)
        .placeholder("Type an answer, then Enter")
        .blur(BlurPolicy::Keep)
        .typing_policy(TypingPolicy::Fallback { cursor: true })
}

fn has_only_focus_initialization_intents(cx: &Cx<'_>, id: Id) -> bool {
    let mut saw_focus_in = false;
    let only_lifecycle = cx.intents(id).all(|intent| match intent {
        termrock::Intent::FocusIn { .. } => {
            saw_focus_in = true;
            true
        }
        termrock::Intent::FocusOut { .. } | termrock::Intent::Cancel => true,
        _ => false,
    });
    saw_focus_in && only_lifecycle
}
