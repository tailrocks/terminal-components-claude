use crate::domain::activity::{Activity, ActivityState};
use crate::sim::{
    activity_supervisor::{
        ActivityExecution, ActivityExecutionId, ActivityExecutionPhase, ActivityStopMilestone,
        ActivityStopPhase,
    },
    task_run::{TaskRun, TaskRunState},
    world::World,
};
use std::collections::BTreeMap;
use termrock::{
    Button, Cx, FgStep, FrameRead, Id, ItemKey, Modifier, Props, Rect, Response, Role, Span,
    StylePatch, TextViewport, Ui, ViewportLine, ViewportState,
};

#[derive(Clone, Debug)]
pub(crate) struct ActivityProjection {
    pub(crate) execution_id: Option<ActivityExecutionId>,
    pub(crate) state_label: String,

    pub(crate) stop_grace_ms: Option<u64>,
    pub(crate) simulated_stop_history: Option<String>,
    pub(crate) stop_enabled: bool,
    pub(crate) lines: Vec<String>,
}

pub(crate) struct StopEvent {
    pub(crate) view_token: u64,
    pub(crate) execution_id: ActivityExecutionId,
    pub(crate) snapshot: Activity,
}

struct ActiveStopOwner {
    id: Id,
    view_token: u64,
    execution_id: ActivityExecutionId,
}

struct RetiredStopOwner {
    id: Id,
}

pub(crate) struct Activities {
    current: Option<u32>,
    outputs: BTreeMap<u32, ViewportState>,
    next_view_token: Option<u64>,
    current_view_token: Option<u64>,
    active_stop_owner: Option<ActiveStopOwner>,
    retired_stop_owners: Vec<RetiredStopOwner>,
}

impl Default for Activities {
    fn default() -> Self {
        Self {
            current: None,
            outputs: BTreeMap::new(),
            next_view_token: Some(1),
            current_view_token: None,
            active_stop_owner: None,
            retired_stop_owners: Vec::new(),
        }
    }
}

impl Activities {
    pub(crate) fn show(&mut self, id: u32) {
        let _ = self.show_with_token(id);
    }

    pub(crate) fn show_with_token(&mut self, id: u32) -> Option<u64> {
        if let Some(owner) = self.active_stop_owner.take() {
            self.retired_stop_owners
                .push(RetiredStopOwner { id: owner.id });
        }
        self.current = Some(id);
        self.outputs.entry(id).or_insert_with(|| {
            let mut state = ViewportState::default();
            state.set_follow(true);
            state
        });
        let token = self.next_view_token;
        self.next_view_token = token.and_then(|value| value.checked_add(1));
        self.current_view_token = token;
        token
    }

    pub(crate) const fn current_view_token(&self) -> Option<u64> {
        self.current_view_token
    }

    pub(crate) fn current_id(&self) -> Option<u32> {
        self.current
    }

    pub(crate) fn current<'a>(&self, world: &'a World) -> Option<&'a Activity> {
        world
            .activities
            .iter()
            .find(|activity| Some(activity.id) == self.current)
    }

    pub(crate) fn update(
        &mut self,
        world: &World,
        projections: &BTreeMap<u32, ActivityProjection>,
        allow_stop: bool,
        cx: &mut Cx<'_>,
    ) -> (Response<()>, Option<StopEvent>) {
        let mut response = Response::ignored();
        for (id, state) in &mut self.outputs {
            let Some(activity) = world.activities.iter().find(|activity| activity.id == *id) else {
                continue;
            };
            let lines = projections.get(id).map_or_else(
                || activity.lines.clone(),
                |projection| projection.lines.clone(),
            );
            let services = service_labels(&lines);
            let spans = output_spans(&lines, &services);
            let viewport_lines: Vec<_> =
                spans.iter().map(|line| ViewportLine::Spans(line)).collect();
            response |= TextViewport::new(output_id(*id))
                .wrap(false)
                .update(cx, state, &viewport_lines)
                .erase();
        }

        let current = self.current(world).cloned();
        let desired_owner = current.as_ref().and_then(|activity| {
            let token = self.current_view_token?;
            let execution_id = projections.get(&activity.id)?.execution_id?;
            Some(ActiveStopOwner {
                id: stop_id(activity.id, token, execution_id),
                view_token: token,
                execution_id,
            })
        });
        if !same_stop_owner(self.active_stop_owner.as_ref(), desired_owner.as_ref()) {
            if let Some(owner) = self.active_stop_owner.take() {
                self.retired_stop_owners
                    .push(RetiredStopOwner { id: owner.id });
            }
            self.active_stop_owner = desired_owner;
        }

        let mut event = None;
        if let (Some(activity), Some(owner)) = (current, self.active_stop_owner.as_ref())
            && self.current_view_token == Some(owner.view_token)
            && let Some(projection) = projections.get(&activity.id)
            && projection.execution_id == Some(owner.execution_id)
        {
            let label = stop_button_label(projection);
            let stop_response = Button::new(owner.id, label)
                .disabled(!allow_stop || !projection.stop_enabled)
                .update(cx);
            let activated = stop_response.activated();
            response |= stop_response.erase();
            if activated && allow_stop && projection.stop_enabled {
                event = Some(StopEvent {
                    view_token: owner.view_token,
                    execution_id: owner.execution_id,
                    snapshot: activity,
                });
            }
        }

        self.retired_stop_owners.retain(|owner| {
            let had_routed_intents = cx.intents(owner.id).next().is_some();
            let mut update = Button::new(owner.id, "Stop").disabled(true).update(cx);
            let _ = update.take_action();
            response |= update.erase();
            let ready = !had_routed_intents
                && cx.area(owner.id).is_none()
                && !cx.state(owner.id).contains(termrock::StateFlags::FOCUSED)
                && cx.capture_owner() != Some(owner.id)
                && cx.top_layer() == termrock::LayerId::PAGE;
            if !ready {
                cx.request_repaint();
            }
            !ready
        });

        (response, event)
    }

    pub(crate) fn draw(
        &self,
        world: &World,
        projections: &BTreeMap<u32, ActivityProjection>,
        ui: &mut Ui<'_>,
        area: Rect,
    ) {
        let Some(activity) = self.current(world) else {
            return;
        };
        let Some(state) = self.outputs.get(&activity.id) else {
            return;
        };
        let projection = projections
            .get(&activity.id)
            .cloned()
            .unwrap_or_else(|| project(activity, None, None, world.now_ms()));
        let tone = ui.paint_patch(&StylePatch::new().set_fg(state_role(activity.state)));
        ui.paint_str(
            Rect::new(area.x, area.y, 1, 1),
            state_glyph(activity.state),
            tone,
        );
        let title = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        );
        ui.paint_str(
            Rect::new(
                area.x.saturating_add(2),
                area.y,
                area.width.saturating_sub(4),
                1,
            ),
            &termrock::truncate(&activity.name, area.width.saturating_sub(4)),
            title,
        );
        if let Some(owner) = self.active_stop_owner.as_ref()
            && self.current_view_token == Some(owner.view_token)
            && projection.execution_id == Some(owner.execution_id)
        {
            Button::new(owner.id, stop_button_label(&projection))
                .disabled(!projection.stop_enabled)
                .draw(
                    ui,
                    Rect::new(area.x, area.y.saturating_add(1), area.width.min(22), 1),
                );
        }
        let state_label = projection.stop_grace_ms.map_or_else(
            || {
                format!(
                    "{} · {}",
                    projection.state_label,
                    elapsed_label(activity, world.now_ms())
                )
            },
            |grace_ms| {
                format!(
                    "{} · {} · SIMULATED stop grace {grace_ms} ms",
                    projection.state_label,
                    elapsed_label(activity, world.now_ms())
                )
            },
        );
        let count_label = format!("{} retained lines", projection.lines.len());
        Props::new(&[
            ("Scope", &activity.scope),
            ("State", &state_label),
            ("Output", &count_label),
        ])
        .draw(
            ui,
            Rect::new(
                area.x,
                area.y.saturating_add(2),
                area.width,
                area.height.saturating_sub(2).min(3),
            ),
        );
        if let Some(history) = projection.simulated_stop_history.as_deref() {
            let history_style = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint)));
            ui.paint_str(
                Rect::new(area.x, area.y.saturating_add(5), area.width, 1),
                &termrock::truncate(history, area.width),
                history_style,
            );
        }
        let output = Rect::new(
            area.x,
            area.y.saturating_add(6),
            area.width,
            area.height.saturating_sub(6),
        );
        let services = service_labels(&projection.lines);
        let spans = output_spans(&projection.lines, &services);
        let lines: Vec<_> = spans.iter().map(|line| ViewportLine::Spans(line)).collect();
        TextViewport::new(output_id(activity.id))
            .wrap(false)
            .draw(ui, output, state, &lines);
        let visible = usize::from(output.height);
        if projection.lines.len() > visible {
            let offset = state
                .scroll()
                .offset()
                .min(projection.lines.len().saturating_sub(visible));
            let label = format!(
                "{}–{} of {}",
                offset.saturating_add(1),
                offset.saturating_add(visible).min(projection.lines.len()),
                projection.lines.len()
            );
            let width = termrock::width(&label);
            if width.saturating_add(2) < area.width {
                let faint = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint)));
                ui.paint_str(
                    Rect::new(area.right().saturating_sub(width), area.y, width, 1),
                    &label,
                    faint,
                );
            }
        }
    }
}

fn same_stop_owner(left: Option<&ActiveStopOwner>, right: Option<&ActiveStopOwner>) -> bool {
    left.zip(right).is_some_and(|(left, right)| {
        left.id == right.id
            && left.view_token == right.view_token
            && left.execution_id == right.execution_id
    }) || left.is_none() && right.is_none()
}

fn stop_id(activity_id: u32, view_token: u64, execution_id: ActivityExecutionId) -> Id {
    Id::root("holla.activity.stop")
        .item(ItemKey::pair(view_token, execution_id.generation()))
        .item(ItemKey::num(u64::from(activity_id)))
}

fn stop_button_label(projection: &ActivityProjection) -> &'static str {
    if projection.state_label.starts_with("stopping") {
        "Stopping…"
    } else if projection.state_label == "stopped"
        || projection.state_label == "completed"
        || projection.state_label == "cancelled"
    {
        "Stopped"
    } else {
        "Stop"
    }
}

fn format_simulated_stop_history(execution: &ActivityExecution) -> Option<String> {
    let requested_at_ms = execution.stop_requested_ms()?;
    let milestones = execution.milestones();
    if milestones.is_empty() {
        return None;
    }
    let history = milestones
        .iter()
        .map(|milestone| {
            let elapsed_ms = milestone.at_ms().saturating_sub(requested_at_ms).max(0);
            match milestone {
                ActivityStopMilestone::TermSent { .. } => format!("TERM +{elapsed_ms} ms"),
                ActivityStopMilestone::KillSent { .. } => format!("KILL +{elapsed_ms} ms"),
                ActivityStopMilestone::Reaped { .. } => format!("reap +{elapsed_ms} ms"),
                ActivityStopMilestone::CancelledBeforeStart { .. } => {
                    format!("queued cancellation before start +{elapsed_ms} ms")
                }
                _ => "simulated lifecycle event".to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join(" · ");
    Some(format!("SIMULATED stop · {history}"))
}

pub(crate) fn project(
    activity: &Activity,
    execution: Option<&ActivityExecution>,
    run: Option<&TaskRun>,
    now_ms: i64,
) -> ActivityProjection {
    let mut lines = activity.lines.clone();
    if let Some(run) = run {
        lines.extend(run.output().iter().map(|output| output.text().to_owned()));
    }
    let execution_id = execution.map(ActivityExecution::id);
    let phase = execution.map(ActivityExecution::phase);
    let task_state = run.map(TaskRun::state);
    let state_label = if phase == Some(ActivityExecutionPhase::Cancelling) {
        "stopping"
    } else if execution.is_some_and(|value| {
        value.phase() == ActivityExecutionPhase::Stopped
            && value.stop_phase() == ActivityStopPhase::Reaped
    }) {
        "stopped"
    } else if let Some(task_state) = task_state {
        match task_state {
            TaskRunState::Completed => "completed",
            TaskRunState::Cancelled => "cancelled",
            TaskRunState::Waiting => "waiting",
            TaskRunState::TimeExhausted => "time exhausted",
            TaskRunState::Running => "running",
            _ => activity.state.label(),
        }
    } else if let Some(execution) = execution {
        match (execution.phase(), execution.stop_phase(), activity.state) {
            (
                ActivityExecutionPhase::Stopped,
                ActivityStopPhase::NotRequested,
                ActivityState::Failed,
            ) => "failed",
            (
                ActivityExecutionPhase::Stopped,
                ActivityStopPhase::NotRequested,
                ActivityState::Succeeded,
            ) => "succeeded",
            (ActivityExecutionPhase::Stopped, _, _) => "stopped",
            (ActivityExecutionPhase::Queued, _, _) => "queued",
            (ActivityExecutionPhase::Running, _, _) => "running",
            (ActivityExecutionPhase::Waiting, _, _) => "waiting",
            (ActivityExecutionPhase::Detached, _, _) => "detached",
            (ActivityExecutionPhase::Cancelling, _, _) => "stopping",
            _ => activity.state.label(),
        }
    } else {
        activity.state.label()
    }
    .to_owned();
    let stop_grace_ms = execution.and_then(|value| match value.phase() {
        ActivityExecutionPhase::Cancelling => value
            .stop_requested_ms()
            .map(|started_ms| now_ms.saturating_sub(started_ms).max(0))
            .and_then(|elapsed| u64::try_from(elapsed).ok()),
        ActivityExecutionPhase::Stopped if value.stop_phase() == ActivityStopPhase::Reaped => {
            let term_at_ms = value
                .milestones()
                .iter()
                .find_map(|milestone| match milestone {
                    ActivityStopMilestone::TermSent { at_ms } => Some(*at_ms),
                    _ => None,
                })?;
            let reaped_at_ms = value
                .milestones()
                .iter()
                .find_map(|milestone| match milestone {
                    ActivityStopMilestone::Reaped { at_ms } => Some(*at_ms),
                    _ => None,
                })?;
            u64::try_from(reaped_at_ms.saturating_sub(term_at_ms).max(0)).ok()
        }
        _ => None,
    });
    let simulated_stop_history = execution.and_then(format_simulated_stop_history);
    let run_terminal = matches!(
        task_state,
        Some(TaskRunState::Completed | TaskRunState::Cancelled)
    );
    let stop_enabled = matches!(
        phase,
        Some(
            ActivityExecutionPhase::Queued
                | ActivityExecutionPhase::Running
                | ActivityExecutionPhase::Waiting
                | ActivityExecutionPhase::Detached
        )
    ) && !run_terminal;
    ActivityProjection {
        execution_id,
        state_label,
        stop_grace_ms,
        simulated_stop_history,
        stop_enabled,
        lines,
    }
}

pub(crate) fn strip_status(projection: &ActivityProjection) -> Option<String> {
    projection.execution_id?;
    if projection.state_label == "stopping" {
        projection
            .stop_grace_ms
            .map(|elapsed| format!("Stopping · {elapsed} ms stop grace"))
    } else if matches!(
        projection.state_label.as_str(),
        "stopped" | "completed" | "cancelled" | "waiting" | "detached" | "time exhausted"
    ) {
        Some(projection.state_label.clone())
    } else {
        None
    }
}

pub(crate) fn output_id(id: u32) -> Id {
    Id::root("activity.output").item(ItemKey::num(u64::from(id)))
}

pub(crate) fn ordered(world: &World) -> Vec<&Activity> {
    let mut activities: Vec<_> = world.activities.iter().collect();
    activities.sort_by_key(|activity| activity.id);
    activities
}

pub(crate) fn state_role(state: ActivityState) -> Role {
    match state {
        ActivityState::Running => Role::Success,
        ActivityState::Waiting => Role::Warning,
        ActivityState::Succeeded => Role::Fg(FgStep::Muted),
        ActivityState::Failed => Role::Danger,
        ActivityState::Detached => Role::Fg(FgStep::Faint),
    }
}

pub(crate) fn state_glyph(state: ActivityState) -> &'static str {
    match state {
        ActivityState::Running => "●",
        ActivityState::Waiting => "…",
        ActivityState::Succeeded => "✓",
        ActivityState::Failed => "✗",
        ActivityState::Detached => "−",
    }
}

fn elapsed_label(activity: &Activity, now_ms: i64) -> String {
    let mins = now_ms.saturating_sub(activity.started_ms) / 60_000;
    if mins < 1 {
        "just now".into()
    } else if mins < 60 {
        format!("{mins}m ago")
    } else if mins < 1_440 {
        format!("{}h ago", mins / 60)
    } else {
        format!("{}d ago", mins / 1_440)
    }
}

fn service_role(name: &str) -> Role {
    match name
        .bytes()
        .fold(0_u8, |sum, byte| sum.wrapping_add(byte) % 4)
    {
        0 => Role::Success,
        1 => Role::Warning,
        2 => Role::Fg(FgStep::Primary),
        _ => Role::Fg(FgStep::Secondary),
    }
}

fn service_labels(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .map(|line| {
            line.split_once(" | ")
                .map_or_else(String::new, |(service, _)| termrock::truncate(service, 12))
        })
        .collect()
}

fn output_spans<'a>(lines: &'a [String], services: &'a [String]) -> Vec<Vec<Span<'a>>> {
    lines
        .iter()
        .zip(services)
        .map(|(line, label)| {
            if let Some((service, rest)) = line.split_once(" | ") {
                vec![
                    Span::new(label).role(service_role(service.trim())),
                    Span::new(" | ").role(Role::Fg(FgStep::Faint)),
                    Span::new(rest).role(Role::Fg(FgStep::Secondary)),
                ]
            } else {
                vec![Span::new(line).role(Role::Fg(FgStep::Secondary))]
            }
        })
        .collect()
}
