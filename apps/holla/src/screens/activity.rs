//! Retained activity output composed with the shared text viewport.
use crate::domain::activity::{Activity, ActivityState};
use crate::sim::world::World;
use junie_tui::{
    Cx, FgStep, Id, ItemKey, Modifier, Props, Rect, Response, Role, Span, StylePatch, TextViewport,
    Ui, ViewportLine, ViewportState,
};
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct Activities {
    current: Option<u32>,
    outputs: BTreeMap<u32, ViewportState>,
}
impl Activities {
    pub(crate) fn show(&mut self, id: u32) {
        self.current = Some(id);
        self.outputs.entry(id).or_insert_with(|| {
            let mut state = ViewportState::default();
            state.set_follow(true);
            state
        });
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
    pub(crate) fn update(&mut self, world: &World, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        for (id, state) in &mut self.outputs {
            let Some(activity) = world.activities.iter().find(|activity| activity.id == *id) else {
                continue;
            };
            let services = service_labels(activity);
            let spans = output_spans(activity, &services);
            let lines: Vec<_> = spans.iter().map(|line| ViewportLine::Spans(line)).collect();
            response |= TextViewport::new(output_id(*id))
                .wrap(false)
                .update(cx, state, &lines)
                .erase();
        }
        response
    }

    pub(crate) fn draw(&self, world: &World, ui: &mut Ui<'_>, area: Rect) {
        let Some(activity) = self.current(world) else {
            return;
        };
        let Some(state) = self.outputs.get(&activity.id) else {
            return;
        };
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
            &junie_tui::truncate(&activity.name, area.width.saturating_sub(4)),
            title,
        );
        let state_label = format!(
            "{} · {}",
            activity.state.label(),
            elapsed_label(activity, world.now_ms())
        );
        let count_label = format!("{} retained lines", activity.lines.len());
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
        let output = Rect::new(
            area.x,
            area.y.saturating_add(6),
            area.width,
            area.height.saturating_sub(6),
        );
        let services = service_labels(activity);
        let spans = output_spans(activity, &services);
        let lines: Vec<_> = spans.iter().map(|line| ViewportLine::Spans(line)).collect();
        TextViewport::new(output_id(activity.id))
            .wrap(false)
            .draw(ui, output, state, &lines);
        let visible = usize::from(output.height);
        if activity.lines.len() > visible {
            let offset = state
                .scroll()
                .offset()
                .min(activity.lines.len().saturating_sub(visible));
            let label = format!(
                "{}–{} of {}",
                offset.saturating_add(1),
                offset.saturating_add(visible).min(activity.lines.len()),
                activity.lines.len()
            );
            let width = junie_tui::width(&label);
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
    // Keep the pinned byte-sum modulo four without an unbounded accumulator.
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
fn service_labels(activity: &Activity) -> Vec<String> {
    activity
        .lines
        .iter()
        .map(|line| {
            line.split_once(" | ")
                .map_or_else(String::new, |(service, _)| junie_tui::truncate(service, 12))
        })
        .collect()
}
fn output_spans<'a>(activity: &'a Activity, services: &'a [String]) -> Vec<Vec<Span<'a>>> {
    activity
        .lines
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::fixtures, scenario::Scenario};
    use junie_tui::{App, KeyCode, Theme};
    use junie_tui_testing::Harness;
    struct Fixture {
        screen: Activities,
        world: World,
    }
    impl App for Fixture {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.screen.update(&self.world, cx)
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            self.screen.draw(&self.world, ui, ui.full());
        }
    }
    #[test]
    fn retained_output_scroll_is_per_stable_activity_and_draw_is_pure() {
        let mut world = fixtures::world_for(Scenario::ActivitiesMulti);
        world.seek(2_000);
        world.activities = [1, 2]
            .into_iter()
            .map(|id| {
                let mut activity =
                    Activity::new(id, "fixture activity", "fixture", ActivityState::Running, 0);
                activity.lines = (0..100)
                    .map(|n| format!("activity {id} line {n}"))
                    .collect();
                activity
            })
            .collect();
        let mut screen = Activities::default();
        screen.show(1);
        let mut harness = Harness::new(Fixture { screen, world }, Theme::junie(), 80, 20);
        assert!(harness.tab_to(output_id(1)));
        let _ = harness.key(KeyCode::Home);
        let first = harness.text();
        assert!(first.contains("activity 1 line 0"));
        harness.app_mut().screen.show(2);
        harness.draw();
        assert!(harness.tab_to(output_id(2)));
        let _ = harness.key(KeyCode::End);
        harness.app_mut().screen.show(1);
        harness.draw();
        assert_eq!(harness.text(), first);
        let before = harness.app().world.activities.clone();
        harness.draw();
        assert_eq!(harness.app().world.activities, before);
    }
}
