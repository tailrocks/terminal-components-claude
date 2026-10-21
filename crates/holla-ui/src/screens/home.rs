use termrock::RowUi;
use termrock::layout::{ColumnSpec, Track, solve_columns};
use termrock::theme::CursorStyle;
use termrock::{
    BlurPolicy, Chord, Color, Cx, Family, FgStep, FrameRead, Hint, HintKey, HintLayer, Id, ItemKey,
    KeyCode, KeyModifiers, NavList, NavListAction, NavListState, Part, Rect, Response, Role,
    StateFlags, StylePatch, TextAction, TextInput, TextInputState, TypingPolicy, Ui, Variant,
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
    /// `docker clean` was seeded for the tick-40 plate. A later seek clears it.
    tag_query: bool,
    /// History-on tag plates. Set from `HOLLA_NO_HISTORY=0`.
    pub(crate) history: bool,
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
            tag_query: false,
            history: false,
        }
    }
}

pub(crate) struct HomeRow {
    section: &'static str,
    action: Action,
    columns: ActionColumns,
}

#[derive(Clone, Copy, Default)]
struct ActionColumns {
    title: u16,
    kind: u16,
    scope: u16,
    risk: u16,
}

impl ActionColumns {
    fn apply(rows: &mut [HomeRow]) {
        let columns = Self {
            title: rows
                .iter()
                .map(|row| termrock::width(&row.action.title))
                .max()
                .unwrap_or(14),
            kind: rows
                .iter()
                .map(|row| termrock::width(action_kind_label(row.action.kind)))
                .max()
                .unwrap_or(4),
            scope: rows
                .iter()
                .map(|row| termrock::width(&scope_text(&row.action)))
                .max()
                .unwrap_or(4),
            risk: rows
                .iter()
                .map(|row| termrock::width(&risk_text(&row.action)))
                .max()
                .unwrap_or(0),
        };
        for row in rows {
            row.columns = columns;
        }
    }
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
        self.tag_query = false;
        self.value.clear();
        self.editor.cancel();
        self.editor.begin("");
    }

    pub(crate) fn seed_query(&mut self, text: &str) {
        self.value = text.to_owned();
        self.editor.cancel();
        self.editor.begin(text);
        self.tag_query = true;
    }

    /// Drop the seeded docker query once the clock leaves tick 40.
    pub(crate) fn release_tag_query(&mut self, world: &World) {
        if self.tag_query && self.query() != catalogue::DOCKER_CLEANUP_QUERY {
            self.tag_query = false;
        }
        if self.tag_query && !catalogue::docker_cleanup_tag_frame(world) {
            self.clear_query();
        }
    }

    pub(crate) fn rows(&self, world: &World) -> Vec<HomeRow> {
        let recent_first =
            self.query().trim().is_empty() && world.scenario.boot_title() == super::finder::RECENT;
        catalogue::sections(world)
            .into_iter()
            .flat_map(|section| {
                let name = if recent_first && section.name == "Suggested here" {
                    "Explore"
                } else {
                    section.name
                };
                catalogue::visible(
                    &section.actions,
                    self.scope,
                    self.query(),
                    &world.memory,
                    &world.cwd,
                )
                .into_iter()
                .map(move |action| HomeRow {
                    section: name,
                    action,
                    columns: ActionColumns::default(),
                })
            })
            .collect()
    }

    pub(crate) fn hints(world: &World, rows_focused: bool) -> HintLayer {
        let key = |code| HintKey::Chord(Chord::key(code));
        let ctrl = |character| HintKey::ChordWithCase {
            chord: Chord::with(KeyCode::Char(character), KeyModifiers::CONTROL),
            case: termrock::ChordCase::UppercaseAscii,
        };
        let entries: &[(HintKey, &str)] = if rows_focused {
            &[
                (key(KeyCode::Enter), "Run"),
                (ctrl('p'), "Preview"),
                (ctrl('o'), "Actions"),
                (key(KeyCode::Up), "Query"),
                (key(KeyCode::Esc), "Clear"),
            ]
        } else {
            &[
                (HintKey::Label("Type"), "Filter"),
                (key(KeyCode::Down), "Results"),
                (key(KeyCode::Enter), "Run top match"),
                (ctrl('s'), "Scope"),
                (key(KeyCode::F(10)), "Menu"),
                (key(KeyCode::Char('q')), "Quit"),
            ]
        };
        let mut hints = Vec::with_capacity(entries.len().saturating_add(1));
        hints.extend(entries.iter().map(|&(key, label)| Hint {
            key,
            label,
            priority: 50,
        }));
        if !world.activities.is_empty() {
            hints.push(Hint {
                key: ctrl('a'),
                label: "Activities",
                priority: 50,
            });
        }
        HintLayer {
            hints,
            ..HintLayer::empty()
        }
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

        if !self.editor.is_editing() {
            self.editor.begin(&self.value);
        }
        let rows = self.rows(world);
        let mut navigation = nav_list().update(cx, &mut self.rows, &rows);
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
        let scope_name = self.scope.map_or("here", Scope::label);
        let scope_label = format!(" scope ‹ {scope_name} › ");
        let scope_width = termrock::width(&scope_label);
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
        let scope_style = ui.paint_patch(&TAG_GREY).with_bg_from(field);
        ui.paint_str(
            Rect::new(
                area.right().saturating_sub(scope_width),
                area.y,
                scope_width.min(area.width),
                1,
            ),
            &scope_label,
            scope_style,
        );
        let mut rows = self.rows(world);
        ActionColumns::apply(&mut rows);
        let mut rows_area = Rect::new(
            area.x,
            area.y.saturating_add(2),
            area.width,
            area.height.saturating_sub(2),
        );
        if self.query().trim().is_empty() && world.scenario.boot_title() == super::finder::RECENT {
            let used = super::finder::paint_recent_lead(ui, rows_area);
            rows_area.y = rows_area.y.saturating_add(used);
            rows_area.height = rows_area.height.saturating_sub(used);
        }
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
            nav_list().draw(ui, rows_area, &self.rows, &rows);
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

/// Tag finder grey. Junie `FgStep::Muted` is `#808080`.
const TAG_GREY: StylePatch = StylePatch::new().set_fg(Role::Custom(Color::Rgb(0x8a, 0x8a, 0x8a)));
const QUERY_PARTS: &[(Part, StylePatch)] = &[(Part::PLACEHOLDER, TAG_GREY)];
/// Home body is already one column in. Tag section titles start at screen column 4.
const SECTION_INDENT: u16 = 3;
/// Header recipe is bold secondary. The pad must stay primary; glyphs are retinted below.
const HEADER_PAD: StylePatch = StylePatch::new()
    .set_fg(Role::Fg(FgStep::Primary))
    .remove(termrock::Modifier::BOLD);
const HEADER_TEXT: StylePatch = TAG_GREY.remove(termrock::Modifier::BOLD);
const HEADER_PARTS: &[(Part, StylePatch)] = &[(Part::HEADER, HEADER_PAD)];

fn query_input() -> TextInput<'static> {
    TextInput::new(QUERY)
        .placeholder("Search actions and resources…")
        .placeholder_while_editing(true)
        .blur(BlurPolicy::Keep)
        .typing_policy(TypingPolicy::Fallback { cursor: true })
        .patch_part(QUERY_PARTS)
}
fn section(row: &HomeRow) -> &str {
    row.section
}
fn nav_list()
-> NavList<'static, HomeRow, impl Fn(&HomeRow) -> ItemKey, impl Fn(&HomeRow, &mut RowUi<'_>)> {
    NavList::new(ROWS)
        .cursor_style(CursorStyle::Fill)
        .scrollable(true)
        .leave_at_boundary(true)
        .header_indent(SECTION_INDENT)
        .section(&section)
        .patch_part(HEADER_PARTS)
        .header_text_patch(&HEADER_TEXT)
        .key(row_key)
        .row(paint_action)
}
fn row_key(row: &HomeRow) -> ItemKey {
    ItemKey::text(&row.action.id)
}

fn paint_action(row: &HomeRow, cells: &mut RowUi<'_>) {
    let action = &row.action;
    let kind = action_kind_label(action.kind);
    let scope = scope_text(action);
    let risk = risk_text(action);

    // Use the row owner's full allocation to keep optional columns responsive
    // when NavList reserves a scrollbar or its leading slots.
    let content_width = cells.remaining_width().min(cells.container_width());
    let title_width = row
        .columns
        .title
        .min((content_width.saturating_mul(36) / 100).max(14));
    let kind_width = row.columns.kind;
    let scope_width = row.columns.scope;
    let risk_width = row.columns.risk;
    let specs = [
        ColumnSpec::new(
            title_width.max(1),
            title_width.max(1),
            title_width.max(1),
            0,
            None,
        ),
        ColumnSpec::new(0, kind_width, kind_width, 0, Some(2)),
        ColumnSpec::new(0, scope_width, scope_width, 0, Some(1)),
        ColumnSpec::new(12, 12, u16::MAX, 1, Some(2)),
        ColumnSpec::new(0, risk_width, risk_width, 0, Some(0)),
    ];
    let resolved = solve_columns(&specs, content_width, 2);
    let tracks = (0..resolved.len())
        .map(|index| Track::Fixed(resolved.width(index).unwrap_or_default()))
        .collect::<Vec<_>>();
    let mut columns = cells.columns_with_gap(&tracks, 2);
    columns
        .cell_part(0, Part::LABEL)
        .text_matched(&action.title, &[]);
    if !kind.is_empty() {
        columns
            .cell_part(1, Part::META)
            .tone(Role::Fg(FgStep::Muted))
            .text_matched(kind, &[]);
    }
    if !scope.is_empty() {
        let scope_tone = if action.scope == Scope::Here {
            Role::Fg(FgStep::Muted)
        } else {
            Role::Fg(FgStep::Secondary)
        };
        columns
            .cell_part(2, Part::META)
            .tone(scope_tone)
            .text_matched(&scope, &[]);
    }
    if !action.reason.is_empty() {
        columns
            .cell_part(3, Part::META)
            .patch(&TAG_GREY)
            .text_matched(&action.reason, &[]);
    }
    if !risk.is_empty() {
        let mut risk_cell = columns.cell_part(4, Part::META);
        if !matches!(action.availability, Availability::Ready) {
            risk_cell
                .patch(
                    &StylePatch::new()
                        .set_fg(Role::Warning)
                        .add(termrock::Modifier::BOLD),
                )
                .text_matched(&risk, &[]);
        } else {
            risk_cell
                .tone(Role::Fg(FgStep::Secondary))
                .text_matched(&risk, &[]);
        }
    }
}

fn action_kind_label(kind: crate::domain::action::ActionKind) -> &'static str {
    use crate::domain::action::ActionKind;

    match kind {
        ActionKind::Task => "task",
        ActionKind::Git => "git",
        ActionKind::Plan => "plan",
        ActionKind::Connect => "connect",
        ActionKind::Flow => "flow",
        ActionKind::Clone => "clone",
        ActionKind::System => "system",
    }
}

fn scope_text(action: &Action) -> String {
    if action.scope_label.is_empty() {
        action.scope.label().to_owned()
    } else {
        format!("{} {}", action.scope.label(), action.scope_label)
    }
}

fn risk_text(action: &Action) -> String {
    match action.availability {
        Availability::Ready => risk_label(action.risk).to_owned(),
        Availability::NeedsTrust(_) => {
            format!("▲ untrusted config · {}", risk_label(action.risk))
        }
        Availability::Blocked(_) => format!("▲ blocked · {}", risk_label(action.risk)),
    }
}

fn risk_label(risk: crate::domain::action::Risk) -> &'static str {
    use crate::domain::action::Risk;

    match risk {
        Risk::ReadOnly => "read only",
        Risk::Bounded => "bounded",
        Risk::Broad => "broad",
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
    use termrock::{App, KeyCode, Theme};
    use termrock_test_support::Harness;

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
            let heading = if scenario.boot_title() == crate::screens::finder::RECENT {
                crate::screens::finder::RECENT
            } else {
                "Suggested here"
            };
            assert!(
                harness.text().contains(heading),
                "{} missing {heading}",
                scenario.name()
            );
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
            termrock::PartRef::item(Part::ROW, ItemKey::text(&selected.id)),
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
    fn shared_action_row_keeps_live_semantic_fields_and_owner_focus() {
        let mut harness = Harness::new(fixture(Scenario::LaunchFailure), Theme::junie(), 120, 40);
        let action = harness
            .app()
            .home
            .rows(&harness.app().world)
            .into_iter()
            .find(|row| !row.action.reason.is_empty())
            .expect("fixture has a live action with a reason")
            .action;
        assert!(harness.find(&action.title).is_some());
        assert!(harness.find(action_kind_label(action.kind)).is_some());
        assert!(harness.find(action.scope.label()).is_some());
        assert!(harness.find(risk_label(action.risk)).is_some());
        assert!(harness.find(&action.reason).is_some());

        let rows_bounds = harness.area_of(ROWS).expect("Home NavList is registered");
        assert!(harness.tab_to(ROWS));
        assert_eq!(harness.focus(), Some(ROWS));
        assert_eq!(
            (rows_bounds.y..rows_bounds.bottom())
                .flat_map(|y| (rows_bounds.x..rows_bounds.right()).map(move |x| (x, y)))
                .filter(|(x, y)| harness.cell(*x, *y).symbol() == "▎")
                .count(),
            0,
            "shared Fill cursor must not add a second focus gutter"
        );
        let _ = harness.key(KeyCode::Down);
        assert!(harness.diagnostics().is_empty());
    }

    #[test]
    fn section_headers_use_nav_list_text_policy_across_widths_and_color_levels() {
        use termrock::ColorLevel::{Ansi16, Ansi256, Mono, TrueColor};

        for width in [79, 80, 81, 99, 100, 101, 119, 120, 121] {
            for level in [TrueColor, Ansi256, Ansi16, Mono] {
                let mut harness =
                    Harness::new(fixture(Scenario::LaunchFailure), Theme::junie(), width, 16)
                        .with_color(level);
                let (text_x, header_y) = harness
                    .find("Suggested here")
                    .expect("live Home rows include a section header");
                assert_eq!(text_x, SECTION_INDENT);
                let rows_bounds = harness.area_of(ROWS).expect("Home NavList is addressable");
                let selected = harness
                    .app()
                    .home
                    .selected(&harness.app().world)
                    .expect("live Home rows have an action");
                let selected_hit = harness
                    .area_of_part(
                        ROWS,
                        termrock::PartRef::item(Part::ROW, ItemKey::text(&selected.id)),
                    )
                    .expect("selected Home row has a real hit region");

                let padding = (0..SECTION_INDENT)
                    .map(|x| {
                        let cell = harness.cell(x, header_y);
                        (cell.fg, cell.bg, cell.modifier)
                    })
                    .collect::<Vec<_>>();
                assert!(padding.iter().all(|style| *style == padding[0]));

                let header = harness.cell(text_x, header_y);
                assert!(!header.modifier.contains(termrock::Modifier::BOLD));
                let header_style = (header.fg, header.bg, header.modifier);
                assert_ne!(header.fg, padding[0].0);

                assert!(harness.tab_to(ROWS), "rows focus at {width} columns");
                assert_eq!(harness.area_of(ROWS), Some(rows_bounds));
                assert_eq!(
                    harness.area_of_part(
                        ROWS,
                        termrock::PartRef::item(Part::ROW, ItemKey::text(&selected.id)),
                    ),
                    Some(selected_hit)
                );
                for x in text_x..text_x.saturating_add(13) {
                    let cell = harness.cell(x, header_y);
                    assert_eq!((cell.fg, cell.bg, cell.modifier), header_style);
                }
                for (x, style) in padding.iter().enumerate() {
                    let cell = harness.cell(u16::try_from(x).unwrap(), header_y);
                    assert_eq!((cell.fg, cell.bg, cell.modifier), *style);
                }

                let _ = harness.key(KeyCode::End);
                assert_eq!(harness.area_of(ROWS), Some(rows_bounds));

                if width == 121 && level == TrueColor {
                    let before = harness
                        .app()
                        .home
                        .rows(&harness.app().world)
                        .into_iter()
                        .map(|row| row.action.title)
                        .collect::<Vec<_>>();
                    let task = harness
                        .app_mut()
                        .world
                        .mise
                        .as_mut()
                        .expect("live fixture includes task data")
                        .tasks
                        .first_mut()
                        .expect("live fixture includes a task");
                    task.id = "task:termrock-header-live-probe".to_owned();
                    let after = harness
                        .app()
                        .home
                        .rows(&harness.app().world)
                        .into_iter()
                        .map(|row| row.action.title)
                        .collect::<Vec<_>>();
                    assert_ne!(before, after, "fixture data changes the live labels");
                    assert!(
                        after
                            .iter()
                            .any(|label| label.contains("termrock-header-live-probe"))
                    );
                    let _ = harness.key(KeyCode::Home);
                    assert!(harness.find("Suggested here").is_some());
                }
            }
        }
    }

    #[test]
    fn home_header_text_policy_survives_menu_open_close_and_focus_restoration() {
        use termrock::ColorLevel::{Ansi16, Ansi256, Mono, TrueColor};
        use termrock::{FeedbackClock, SimulationMoment};

        for width in [119, 120, 121] {
            for level in [TrueColor, Ansi256, Ansi16, Mono] {
                let app = crate::App::for_scenario_without_history(
                    Scenario::RustDirty,
                    crate::Motion::Paused,
                    2_000,
                );
                let initial = u64::try_from(app.fixture_time_ms())
                    .expect("simulation fixture time is nonnegative");
                let mut harness = Harness::new_with_feedback_clock(
                    app,
                    Theme::junie(),
                    width,
                    40,
                    FeedbackClock::Simulation {
                        initial: SimulationMoment::from_millis(initial),
                    },
                )
                .with_color(level);
                let (text_x, header_y) = harness
                    .find("Suggested here")
                    .expect("live Home rows include a section header");
                let rows_bounds = harness.area_of(ROWS).expect("Home NavList is addressable");
                assert_eq!(text_x, rows_bounds.x.saturating_add(SECTION_INDENT));
                let home_focus = Some(QUERY);
                assert_eq!(harness.focus(), home_focus);
                let style = |harness: &Harness<crate::App>, x, y| {
                    let cell = harness.cell(x, y);
                    (cell.fg, cell.bg, cell.modifier)
                };
                let before_padding = (0..SECTION_INDENT)
                    .map(|offset| style(&harness, rows_bounds.x.saturating_add(offset), header_y))
                    .collect::<Vec<_>>();
                let before_text = style(&harness, text_x, header_y);
                assert!(
                    before_padding
                        .iter()
                        .all(|entry| *entry == before_padding[0])
                );
                assert_ne!(before_padding[0].0, before_text.0);

                assert_eq!(
                    (rows_bounds.y..rows_bounds.bottom())
                        .flat_map(|y| (rows_bounds.x..rows_bounds.right()).map(move |x| (x, y)))
                        .filter(|(x, y)| harness.cell(*x, *y).symbol() == "▎")
                        .count(),
                    0,
                    "Home rows must not paint an application-owned focus gutter"
                );
                let _ = harness.key(KeyCode::F(10));
                assert_eq!(harness.focus(), Some(Id::root("holla.menu")));
                assert!(harness.text().contains("Quit"));
                let (_, overlay_header_y) = harness
                    .find("Suggested here")
                    .expect("Home header remains rendered beneath the open menu layer");
                let overlay_padding = (0..SECTION_INDENT)
                    .map(|offset| {
                        style(
                            &harness,
                            rows_bounds.x.saturating_add(offset),
                            overlay_header_y,
                        )
                    })
                    .collect::<Vec<_>>();
                let overlay_text = style(&harness, text_x, overlay_header_y);
                assert_eq!(overlay_padding, before_padding);
                assert_eq!(overlay_text, before_text);
                assert_eq!(
                    (rows_bounds.y..rows_bounds.bottom())
                        .flat_map(|y| (rows_bounds.x..rows_bounds.right()).map(move |x| (x, y)))
                        .filter(|(x, y)| harness.cell(*x, *y).symbol() == "▎")
                        .count(),
                    0,
                    "inactive Home rows must not retain focus under the menu"
                );

                let _ = harness.key(KeyCode::Esc);
                assert_eq!(harness.focus(), home_focus);
                assert!(
                    harness.diagnostics().is_empty(),
                    "{:?}",
                    harness.diagnostics()
                );
                let (_, closed_header_y) = harness
                    .find("Suggested here")
                    .expect("Home header remains rendered after closing the menu");
                assert_eq!(
                    (0..SECTION_INDENT)
                        .map(|offset| {
                            style(
                                &harness,
                                rows_bounds.x.saturating_add(offset),
                                closed_header_y,
                            )
                        })
                        .collect::<Vec<_>>(),
                    before_padding
                );
                assert_eq!(style(&harness, text_x, closed_header_y), before_text);
            }
        }
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
