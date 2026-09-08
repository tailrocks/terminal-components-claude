//! Ranked action surface. Shared controls own text editing and row interaction.
use junie_tui::{
    BlurPolicy, Cx, Family, FgStep, FrameRead, Id, ItemKey, NavList, NavListAction, NavListState,
    Part, Rect, Response, Role, StateFlags, StylePatch, TextAction, TextInput, TextInputState,
    TypingPolicy, Ui, Variant,
};

use crate::domain::action::{Action, Availability, Scope};
use crate::sim::{catalogue, world::World};

pub(crate) const QUERY: Id = Id::root("home.query");
pub(crate) const ROWS: Id = Id::root("home.rows");

pub(crate) struct HomeState {
    value: String,
    editor: TextInputState,
    rows: NavListState,
    pub(crate) scope: Option<Scope>,
}

impl Default for HomeState {
    fn default() -> Self {
        let mut editor = TextInputState::default();
        editor.begin("");
        Self {
            value: String::new(),
            editor,
            rows: NavListState::default(),
            scope: None,
        }
    }
}

pub(crate) struct HomeRow {
    section: &'static str,
    action: Action,
}

impl std::fmt::Display for HomeRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.action.title)
    }
}

impl HomeState {
    pub(crate) fn query(&self) -> &str {
        self.editor.draft_text().unwrap_or(&self.value)
    }

    pub(crate) fn clear_query(&mut self) {
        self.value.clear();
        self.editor.cancel();
        self.editor.begin("");
    }

    pub(crate) fn rows(&self, world: &World) -> Vec<HomeRow> {
        catalogue::sections(world)
            .into_iter()
            .flat_map(|section| {
                catalogue::visible(
                    &section.actions,
                    self.scope,
                    self.query(),
                    &world.memory,
                    &world.cwd,
                )
                .into_iter()
                .map(move |action| HomeRow {
                    section: section.name,
                    action,
                })
            })
            .collect()
    }

    pub(crate) fn is_editing(&self) -> bool {
        self.editor.is_editing()
    }

    pub(crate) fn selected(&self, world: &World) -> Option<Action> {
        let rows = self.rows(world);
        self.rows
            .cursor()
            .and_then(|key| rows.iter().find(|row| row_key(row) == key))
            .or_else(|| rows.first())
            .map(|row| row.action.clone())
    }

    pub(crate) fn focus_first(&mut self, world: &World, cx: &mut Cx<'_>) {
        if let Some(row) = self.rows(world).first() {
            self.rows.set_cursor(0, row_key(row));
            cx.focus(ROWS);
        } else {
            cx.focus_next();
        }
    }

    pub(crate) fn subject(&self, world: &World, cx: &Cx<'_>) -> Option<Action> {
        if cx.state(QUERY).contains(StateFlags::FOCUSED) {
            self.rows(world).into_iter().next().map(|row| row.action)
        } else if cx.state(ROWS).contains(StateFlags::FOCUSED) {
            self.selected(world)
        } else {
            None
        }
    }

    pub(crate) fn update(
        &mut self,
        world: &World,
        cx: &mut Cx<'_>,
    ) -> (Response<()>, Option<Action>) {
        let mut text = query_input().update(cx, &mut self.editor, &mut self.value);
        let submit = matches!(text.take_action(), Some(TextAction::Committed));
        // The launcher query stays armed after submission or an empty cancel.
        // The shared editor remains the sole owner of its draft and cursor.
        if !self.editor.is_editing() {
            self.editor.begin(&self.value);
        }
        let rows = self.rows(world);
        let mut navigation = NavList::new(ROWS)
            .scrollable(true)
            .leave_at_boundary(true)
            .header_indent(2)
            .section(&section)
            .key(row_key)
            .update(cx, &mut self.rows, &rows);
        let chosen = match navigation.take_action() {
            Some(NavListAction::LeaveBackward) => {
                cx.focus_prev();
                None
            }
            Some(NavListAction::LeaveForward) => {
                cx.focus_next();
                None
            }
            Some(NavListAction::Chose(key) | NavListAction::EnterContent(key)) => rows
                .into_iter()
                .find(|row| row_key(row) == key)
                .map(|row| row.action),
            _ if submit => rows.into_iter().next().map(|row| row.action),
            _ => None,
        };
        (text.erase() | navigation.erase(), chosen)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "One pure Home projection keeps query, scope, rows and discovery geometry together"
    )]
    pub(crate) fn draw(&self, world: &World, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let scope_label = self.scope.map(|scope| format!("scope: {}", scope.label()));
        let scope_width = scope_label
            .as_ref()
            .map_or(0, |label| junie_tui::width(label).saturating_add(3));
        let field = ui
            .style(
                Family::INPUT,
                Variant::DEFAULT,
                Part::FIELD,
                StateFlags::empty(),
            )
            .style;
        ui.fill(Rect::new(area.x, area.y, area.width, 1), field);
        query_input().value(&self.value).draw(
            ui,
            Rect::new(area.x, area.y, area.width.saturating_sub(scope_width), 1),
            &self.editor,
        );
        if let Some(label) = scope_label {
            let style = ui
                .paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)))
                .with_bg_from(field);
            ui.paint_str(
                Rect::new(
                    area.right().saturating_sub(scope_width),
                    area.y,
                    scope_width,
                    1,
                ),
                &label,
                style,
            );
        }
        let rows = self.rows(world);
        let rows_area = Rect::new(
            area.x,
            area.y.saturating_add(2),
            area.width,
            area.height.saturating_sub(2),
        );
        if rows.is_empty() {
            let note = if self.query().trim().is_empty() {
                self.scope.map_or_else(
                    || "Nothing discovered here yet".to_owned(),
                    |scope| format!("Nothing in scope {} here", scope.label()),
                )
            } else {
                format!("Nothing matches “{}” here", self.query().trim())
            };
            let style = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
            ui.paint_str(
                Rect::new(
                    rows_area.x.saturating_add(2),
                    rows_area.y,
                    rows_area.width.saturating_sub(4),
                    1,
                ),
                &note,
                style,
            );
        } else {
            NavList::new(ROWS)
                .scrollable(true)
                .leave_at_boundary(true)
                .header_indent(2)
                .section(&section)
                .key(row_key)
                .render_row(&paint_action)
                .draw(ui, rows_area, &self.rows, &rows);
        }
        let groups = rows
            .iter()
            .map(|row| row.section)
            .fold((None, 0_usize), |(previous, count), name| {
                (
                    Some(name),
                    count.saturating_add(usize::from(previous != Some(name))),
                )
            })
            .1;
        let occupied = rows
            .len()
            .saturating_add(groups.saturating_mul(2).saturating_sub(1));
        if occupied.saturating_add(1) < usize::from(rows_area.height) {
            let pending: Vec<_> = world
                .discovery
                .iter()
                .filter(|item| !item.done && !item.failed)
                .map(|item| item.domain.label())
                .collect();
            let failed: Vec<_> = world
                .discovery
                .iter()
                .filter(|item| item.failed)
                .map(|item| item.domain.label())
                .collect();
            let note = if !pending.is_empty() {
                Some((
                    format!("Scanning this folder… ({})", pending.join(", ")),
                    Role::Fg(FgStep::Faint),
                ))
            } else if !failed.is_empty() {
                Some((
                    format!("▲ {}: discovery failed", failed.join(", ")),
                    Role::Warning,
                ))
            } else {
                None
            };
            if let Some((text, role)) = note {
                let style = ui.paint_patch(&StylePatch::new().set_fg(role));
                ui.paint_str(
                    Rect::new(
                        area.x.saturating_add(2),
                        area.bottom().saturating_sub(1),
                        area.width.saturating_sub(2),
                        1,
                    ),
                    &text,
                    style,
                );
            }
        }
    }
}

fn query_input() -> TextInput<'static> {
    TextInput::new(QUERY)
        .placeholder("Type to filter · actions, files, hosts")
        .blur(BlurPolicy::Keep)
        .typing_policy(TypingPolicy::Fallback { cursor: true })
}
fn section(row: &HomeRow) -> &str {
    row.section
}
fn row_key(row: &HomeRow) -> ItemKey {
    ItemKey::text(&row.action.id)
}

fn paint_action(ui: &mut Ui<'_>, area: Rect, flags: StateFlags, _key: ItemKey, row: &HomeRow) {
    let style = ui
        .style(Family::LIST, Variant::DEFAULT, Part::ROW, flags)
        .style;
    ui.fill(area, style);
    let gutter = ui
        .style(Family::LIST, Variant::DEFAULT, Part::GUTTER, flags)
        .style;
    ui.paint_str(Rect::new(area.x, area.y, 1, 1), "▎", gutter);
    let action = &row.action;
    let mut x = area.x.saturating_add(2);
    let title_width = junie_tui::width(&action.title);
    if x.saturating_add(title_width) < area.right() {
        ui.paint_str(Rect::new(x, area.y, title_width, 1), &action.title, style);
        x = x.saturating_add(title_width);
    } else {
        let budget = area.right().saturating_sub(x).saturating_sub(1);
        ui.paint_str(
            Rect::new(x, area.y, budget, 1),
            &junie_tui::truncate(&action.title, budget),
            style,
        );
        return;
    }
    if !matches!(action.availability, Availability::Ready) && x.saturating_add(2) < area.right() {
        let warning = ui
            .paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Warning)
                    .add(junie_tui::Modifier::BOLD),
            )
            .with_bg_from(style);
        ui.paint_str(Rect::new(x.saturating_add(1), area.y, 1, 1), "▲", warning);
        x = x.saturating_add(2);
    }
    let reason_width = junie_tui::width(&action.reason);
    let reason_x = if area.width > reason_width.saturating_add(24)
        && area.right() > x.saturating_add(2).saturating_add(reason_width)
    {
        area.right().saturating_sub(reason_width.saturating_add(1))
    } else {
        area.right()
    };
    if action.scope != Scope::Here {
        let tag = if action.scope_label.is_empty() {
            format!("· {}", action.scope.label())
        } else {
            format!("· {} {}", action.scope.label(), action.scope_label)
        };
        let budget = reason_x.saturating_sub(x.saturating_add(3));
        if budget >= 10 {
            let faint = ui
                .paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint)))
                .with_bg_from(style);
            ui.paint_str(
                Rect::new(x.saturating_add(1), area.y, budget, 1),
                &junie_tui::truncate_middle(&tag, budget),
                faint,
            );
        }
    }
    if reason_x < area.right() {
        let muted = ui
            .paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)))
            .with_bg_from(style);
        ui.paint_str(
            Rect::new(reason_x, area.y, reason_width, 1),
            &action.reason,
            muted,
        );
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "Named fixtures provide known rows and failed fixture assumptions must fail tests"
)]
mod tests {
    use super::*;
    use crate::{domain::fixtures, scenario::Scenario};
    use junie_tui::{App, KeyCode, Theme};
    use junie_tui_testing::Harness;

    struct HomeFixture {
        home: HomeState,
        world: World,
        invoked: Option<Action>,
    }
    impl App for HomeFixture {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let (response, action) = self.home.update(&self.world, cx);
            if action.is_some() {
                self.invoked = action;
            }
            response
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            self.home.draw(&self.world, ui, ui.full());
        }
    }
    fn fixture(scenario: Scenario) -> HomeFixture {
        let mut world = fixtures::world_for(scenario);
        world.seek(2_000);
        HomeFixture {
            home: HomeState::default(),
            world,
            invoked: None,
        }
    }

    #[test]
    fn all_scenarios_render_without_mutating_domain_or_query() {
        for scenario in Scenario::ALL {
            let mut harness = Harness::new(fixture(scenario), Theme::junie(), 120, 40);
            let before = catalogue::catalogue(&harness.app().world);
            let time = harness.app().world.now_ms();
            let query = harness.app().home.query().to_owned();
            harness.draw();
            harness.draw();
            assert_eq!(catalogue::catalogue(&harness.app().world), before);
            assert_eq!(harness.app().world.now_ms(), time);
            assert_eq!(harness.app().home.query(), query);
            assert!(harness.text().contains("Suggested here"));
        }
    }

    #[test]
    fn shared_query_filters_and_enter_reports_real_catalogue_identity() {
        let mut harness = Harness::new(fixture(Scenario::RustDirty), Theme::junie(), 120, 40);
        assert!(harness.tab_to(QUERY));
        let _ = harness.type_str("git");
        assert_eq!(harness.app().home.query(), "git");
        let expected = harness
            .app()
            .home
            .selected(&harness.app().world)
            .map(|action| action.id);
        let _ = harness.key(KeyCode::Enter);
        assert_eq!(
            harness
                .app()
                .invoked
                .as_ref()
                .map(|action| action.id.clone()),
            expected
        );
    }

    #[test]
    fn mouse_and_keyboard_row_activation_share_identity() {
        let mut mouse = Harness::new(fixture(Scenario::DockerCleanup), Theme::junie(), 120, 40);
        let selected = mouse.app().home.selected(&mouse.app().world);
        let Some(selected) = selected else {
            panic!("fixture has actions")
        };
        let _ = mouse.click_part(
            ROWS,
            junie_tui::PartRef::item(Part::ROW, ItemKey::text(&selected.id)),
        );
        assert_eq!(
            mouse.app().invoked.as_ref().map(|action| &action.id),
            Some(&selected.id)
        );
        let mut keyboard = Harness::new(fixture(Scenario::DockerCleanup), Theme::junie(), 120, 40);
        assert!(keyboard.tab_to(ROWS));
        let _ = keyboard.key(KeyCode::Enter);
        assert_eq!(
            keyboard.app().invoked.as_ref().map(|action| &action.id),
            Some(&selected.id)
        );
    }
    #[test]
    fn query_submission_uses_top_match_after_row_navigation() {
        let mut harness = Harness::new(fixture(Scenario::RustDirty), Theme::junie(), 120, 40);
        let expected = harness.app().home.rows(&harness.app().world)[0]
            .action
            .id
            .clone();
        assert!(harness.tab_to(ROWS));
        let _ = harness.key(KeyCode::Down);
        assert_ne!(
            harness
                .app()
                .home
                .selected(&harness.app().world)
                .unwrap()
                .id,
            expected
        );
        assert!(harness.tab_to(QUERY));
        let _ = harness.key(KeyCode::Enter);
        assert_eq!(harness.app().invoked.as_ref().unwrap().id, expected);
    }
}
