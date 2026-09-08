//! Compound review: caller-owned inclusion, shared scrolling, no draw effects.
use crate::domain::plan::{Plan, PlanStep, StepState};
use crate::sim::plans::ReviewedPlan;
use junie_tui::{
    Cx, Family, FgStep, Id, ItemKey, List, ListAction, ListState, Modifier, Part, Props, Rect,
    Response, Role, SelectMode, StateFlags, StylePatch, Ui, Variant,
};

pub(crate) const STEPS: Id = Id::root("plan.steps");
pub(crate) struct PlanState {
    pub(crate) review: ReviewedPlan,
    selection: ListState,
}
pub(crate) enum Event {
    Gate,
    Notice(String),
}
struct StepRow<'a> {
    step: &'a PlanStep,
    blocked: Option<String>,
    ran: bool,
}
impl std::fmt::Display for StepRow<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.step.title)
    }
}
fn key(row: &StepRow<'_>) -> ItemKey {
    ItemKey::text(&row.step.id)
}
fn rows(plan: &Plan) -> Vec<StepRow<'_>> {
    plan.steps()
        .iter()
        .enumerate()
        .map(|(index, step)| StepRow {
            step,
            blocked: plan.blocked_by_exclusion(index),
            ran: plan.ran(),
        })
        .collect()
}
impl PlanState {
    pub(crate) fn new(review: ReviewedPlan) -> Self {
        Self {
            review,
            selection: ListState::default(),
        }
    }
    pub(crate) fn plan(&self) -> &Plan {
        self.review.plan()
    }
    pub(crate) fn selected(&self) -> Option<&PlanStep> {
        self.selection.cursor().and_then(|key| {
            self.plan()
                .steps()
                .iter()
                .find(|step| ItemKey::text(&step.id) == key)
        })
    }
    pub(crate) fn toggle_selected(&mut self) -> String {
        let Some(index) = self.selection.cursor().and_then(|key| {
            self.plan()
                .steps()
                .iter()
                .position(|step| ItemKey::text(&step.id) == key)
        }) else {
            return "No step selected".into();
        };
        self.review
            .toggle(index)
            .unwrap_or_else(|error| error.to_string())
    }
    pub(crate) fn continuation(&self) -> Event {
        if self.plan().ran() {
            Event::Notice("Plan already ran · Esc returns home".into())
        } else if self.plan().included_count() == 0 {
            Event::Notice("Nothing included · every step is excluded".into())
        } else {
            Event::Gate
        }
    }
    pub(crate) fn update(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<Event>) {
        let rows = rows(self.review.plan());
        let mut response = List::new(STEPS)
            .key(key)
            .select_mode(SelectMode::None)
            .update(cx, &mut self.selection, &rows);
        let event = if matches!(response.take_action(), Some(ListAction::Activated(_)))
            && cx.activation_key() == Some(junie_tui::ActivationKey::Enter)
        {
            Some(self.continuation())
        } else {
            None
        };
        (response.erase(), event)
    }
    pub(crate) fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.height < 8 {
            return;
        }
        let plan = self.plan();
        let primary = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        );
        let faint = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint)));
        let title = junie_tui::truncate(plan.title(), area.width);
        ui.paint_str(Rect::new(area.x, area.y, area.width, 1), &title, primary);
        let posture = if plan.ran() {
            format!("ran · {}", plan.summary())
        } else {
            "review · Space excludes · dependents recalculate · nothing runs yet".into()
        };
        let posture_width = junie_tui::width(&posture);
        if junie_tui::width(&title)
            .saturating_add(2)
            .saturating_add(posture_width)
            < area.width
        {
            ui.paint_str(
                Rect::new(
                    area.right().saturating_sub(posture_width),
                    area.y,
                    posture_width,
                    1,
                ),
                &posture,
                faint,
            );
        }
        let facts = plan.facts();
        let facts: Vec<_> = facts
            .iter()
            .map(|fact| (fact.label.as_str(), fact.value.as_str()))
            .collect();
        let facts_height = facts.len().min(usize::from(u16::MAX)) as u16;
        Props::new(&facts).draw(
            ui,
            Rect::new(area.x, area.y.saturating_add(2), area.width, facts_height),
        );
        let selected = self.selected();
        let output_height = selected
            .filter(|step| !step.lines.is_empty())
            .map_or(0, |step| step.lines.len().saturating_add(2).min(8) as u16);
        let header_height = facts_height.saturating_add(4);
        let steps_area = Rect::new(
            area.x,
            area.y.saturating_add(header_height),
            area.width,
            area.height.saturating_sub(
                header_height
                    .saturating_add(output_height)
                    .saturating_add(u16::from(output_height > 0)),
            ),
        );
        ui.paint_str(
            Rect::new(area.x, steps_area.y.saturating_sub(1), area.width, 1),
            if plan.ran() {
                "Steps — outcome per step"
            } else {
                "Steps — branches run in parallel where no dependency holds"
            },
            faint,
        );
        let rows = rows(plan);
        List::new(STEPS)
            .key(key)
            .select_mode(SelectMode::None)
            .render_row(&paint_step)
            .draw(ui, steps_area, &self.selection, &rows);
        if output_height > 0
            && let Some(step) = selected
        {
            let top = area.bottom().saturating_sub(output_height);
            let label = format!("Output — {} · ${}", step.title, step.command);
            ui.paint_str(
                Rect::new(area.x, top, area.width, 1),
                &junie_tui::truncate(&label, area.width),
                faint,
            );
            let secondary = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)));
            for (line, text) in
                (top.saturating_add(1)..area.bottom().saturating_sub(1)).zip(&step.lines)
            {
                ui.paint_str(
                    Rect::new(area.x, line, area.width, 1),
                    &junie_tui::truncate(text, area.width),
                    secondary,
                );
            }
        }
    }
}
fn paint_step(ui: &mut Ui<'_>, area: Rect, flags: StateFlags, _key: ItemKey, row: &StepRow<'_>) {
    let style = ui
        .style(Family::LIST, Variant::DEFAULT, Part::ROW, flags)
        .style;
    ui.fill(area, style);
    let gutter = ui
        .style(Family::LIST, Variant::DEFAULT, Part::GUTTER, flags)
        .style;
    ui.paint_str(Rect::new(area.x, area.y, 1, 1), "▎", gutter);
    let step = row.step;
    let (glyph, role) = if row.blocked.is_some() {
        ("▲", Role::Warning)
    } else {
        match &step.state {
            StepState::Pending => ("·", Role::Fg(FgStep::Muted)),
            StepState::Excluded | StepState::PolicySkipped(_) | StepState::Skipped(_) => {
                ("−", Role::Fg(FgStep::Muted))
            }
            StepState::Succeeded => ("✓", Role::Success),
            StepState::Failed(_) => ("✗", Role::Danger),
        }
    };
    let glyph_style = ui
        .paint_patch(&StylePatch::new().set_fg(role))
        .with_bg_from(style);
    ui.paint_str(
        Rect::new(area.x.saturating_add(2), area.y, 1, 1),
        glyph,
        glyph_style,
    );
    let title = if !step.optional && !row.ran {
        format!("{} · required", step.title)
    } else {
        step.title.clone()
    };
    let x = area.x.saturating_add(4);
    let width = junie_tui::width(&title).min(46);
    ui.paint_str(
        Rect::new(x, area.y, area.right().saturating_sub(x).min(46), 1),
        &junie_tui::truncate(&title, 46),
        style,
    );
    let mut x = x.saturating_add(width);
    let branch = format!("· {}", step.branch);
    if x.saturating_add(junie_tui::width(&branch))
        .saturating_add(26)
        < area.right()
    {
        let faint = ui
            .paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint)))
            .with_bg_from(style);
        ui.paint_str(
            Rect::new(x.saturating_add(1), area.y, junie_tui::width(&branch), 1),
            &branch,
            faint,
        );
        x = x
            .saturating_add(1)
            .saturating_add(junie_tui::width(&branch));
    }
    let (note, role) = if let Some(dependency) = &row.blocked {
        (format!("needs {dependency}"), Role::Warning)
    } else {
        match &step.state {
            StepState::Pending => (step.command.clone(), Role::Fg(FgStep::Muted)),
            StepState::Excluded => ("excluded · Space restores".into(), Role::Fg(FgStep::Muted)),
            StepState::PolicySkipped(why) | StepState::Skipped(why) => {
                (why.clone(), Role::Fg(FgStep::Muted))
            }
            StepState::Succeeded => ("done".into(), Role::Success),
            StepState::Failed(why) => (why.clone(), Role::Danger),
        }
    };
    let width = junie_tui::width(&note);
    if area.width > width.saturating_add(30)
        && area.right().saturating_sub(width.saturating_add(1)) > x.saturating_add(2)
    {
        let note_style = ui
            .paint_patch(&StylePatch::new().set_fg(role))
            .with_bg_from(style);
        ui.paint_str(
            Rect::new(
                area.right().saturating_sub(width.saturating_add(1)),
                area.y,
                width,
                1,
            ),
            &note,
            note_style,
        );
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "Named deterministic compound plans must exist"
)]
mod tests {
    use super::*;
    use crate::{domain::fixtures, scenario::Scenario, sim::plans};
    use junie_tui::{App, KeyCode, Theme};
    use junie_tui_testing::Harness;
    struct Fixture {
        plan: PlanState,
        gate_requested: bool,
    }
    impl App for Fixture {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let (response, event) = self.plan.update(cx);
            if matches!(event, Some(Event::Gate)) {
                self.gate_requested = true;
            }
            response
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            self.plan.draw(ui, ui.full());
        }
    }
    #[test]
    fn last_step_remains_reachable_in_small_viewport_without_paint_mutation() {
        let mut world = fixtures::world_for(Scenario::MonorepoRoot);
        world.seek(4_000);
        let review = plans::plan_for(&world, "git.sync").unwrap();
        let last = review.plan().steps().last().unwrap().id.clone();
        let mut harness = Harness::new(
            Fixture {
                plan: PlanState::new(review),
                gate_requested: false,
            },
            Theme::junie(),
            80,
            16,
        );
        assert!(harness.tab_to(STEPS));
        let _ = harness.key(KeyCode::End);
        assert_eq!(
            harness.app().plan.selected().map(|step| &step.id),
            Some(&last)
        );
        assert!(
            harness
                .area_of_part(
                    STEPS,
                    junie_tui::PartRef::item(Part::ROW, ItemKey::text(&last))
                )
                .is_some()
        );
        let before = harness
            .app()
            .plan
            .plan()
            .steps()
            .iter()
            .map(|step| step.state.clone())
            .collect::<Vec<_>>();
        harness.draw();
        harness.draw();
        assert_eq!(
            harness
                .app()
                .plan
                .plan()
                .steps()
                .iter()
                .map(|step| step.state.clone())
                .collect::<Vec<_>>(),
            before
        );
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.app().gate_requested);
        assert!(!harness.app().plan.plan().ran());
    }
}
