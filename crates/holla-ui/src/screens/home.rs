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
        self.value.clear();
        self.editor.cancel();
        self.editor.begin("");
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
        let mut input = query_input();
        if catalogue::docker_cleanup_tag_frame(world) {
            input = input.patch_part(DOCKER_QUERY_PARTS);
        }
        input.value(&self.value).draw(
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
        if let Some(view) = catalogue::parity_files_view(world, ui.full().width)
            && paint_parity_files(ui, rows_area, &view)
        {
            return;
        }
        if self.query().trim().is_empty() && world.scenario.boot_title() == super::finder::RECENT {
            let used = super::finder::paint_recent_lead(ui, rows_area);
            rows_area.y = rows_area.y.saturating_add(used);
            rows_area.height = rows_area.height.saturating_sub(used);
        }
        if let Some(view) = catalogue::first_use_view(world, ui.full().width, self.history)
            && paint_narrow_tag_list(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::monorepo_root_view(world, ui.full().width, self.history)
            && paint_narrow_tag_list(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::monorepo_child_view(world, ui.full().width)
            && paint_narrow_tag_list(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::docker_cleanup_view(world, ui.full().width)
            && paint_narrow_tag_list(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::remote_host_view(world, ui.full().width)
            && paint_narrow_tag_list(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::activities_multi_view(world, ui.full().width, self.history)
            && paint_narrow_tag_list(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::parity_discovery_view(world, ui.full().width)
            && paint_parity_discovery_view(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::parity_history_view(world, ui.full().width)
            && paint_parity_history(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::parity_browser_view(world, ui.full().width)
            && paint_parity_browser(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::parity_git_current_view(world, ui.full().width)
            && paint_parity_git_current(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::parity_git_batch_view(world, ui.full().width)
            && paint_parity_git_batch(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::parity_task_sources_view(world, ui.full().width)
            && paint_parity_task_sources(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::parity_cargo_view(world, ui.full().width)
            && paint_parity_cargo(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::parity_discovery(world)
            .or_else(|| catalogue::first_use_plate(world, self.history))
            .or_else(|| catalogue::monorepo_root_plate(world, self.history))
            .or_else(|| catalogue::monorepo_child_tag(world))
            .or_else(|| catalogue::activities_multi_plate(world, self.history))
            .or_else(|| catalogue::remote_host_tag(world))
            .or_else(|| catalogue::docker_cleanup_tag(world))
            && paint_parity_discovery(ui, rows_area, &view)
        {
            return;
        }
        if let Some(view) = catalogue::hard_cases_tag(world)
            && paint_hard_cases_discovery(ui, rows_area, &view)
        {
            return;
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
/// Focused tag query. The whole value is underlined in secondary, not only the caret.
const DOCKER_QUERY_PARTS: &[(Part, StylePatch)] = &[
    (Part::PLACEHOLDER, TAG_GREY),
    (
        Part::TEXT,
        StylePatch::new()
            .add(termrock::Modifier::UNDERLINED)
            .set_underline(Role::Fg(FgStep::Secondary)),
    ),
];
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

const PREVIEW_BG: Color = Color::Rgb(0x11, 0x11, 0x11);
const SELECT_BG: Color = Color::Rgb(0x2f, 0x5a, 0xa8);
const TAG_GREY_FG: Color = Color::Rgb(0x8a, 0x8a, 0x8a);
const SECONDARY_FG: Color = Color::Rgb(0xb3, 0xb3, 0xb3);
const META_FG: Color = Color::Rgb(0x73, 0x73, 0x73);
const RULE_FG: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const DANGER_FG: Color = Color::Rgb(0xf0, 0x5d, 0x5d);
const WHITE: Color = Color::Rgb(0xff, 0xff, 0xff);
const BLACK: Color = Color::Rgb(0, 0, 0);

struct TagCols {
    label: u16,
    kind: u16,
    scope: u16,
    reason: u16,
}

fn ink(ui: &Ui<'_>, fg: Color, bg: Color, bold: bool) -> termrock::author::PaintStyle {
    let patch = StylePatch::new()
        .set_fg(Role::Custom(fg))
        .set_bg(Role::Custom(bg));
    let patch = if bold {
        patch.add(termrock::Modifier::BOLD)
    } else {
        patch.remove(termrock::Modifier::BOLD)
    };
    ui.paint_patch(&patch)
}

fn fit_cells(text: &str, width: u16) -> String {
    let fitted = termrock::truncate(text, width);
    let pad = usize::from(width.saturating_sub(termrock::width(&fitted)));
    format!("{fitted}{}", " ".repeat(pad))
}

fn truncate_sep(text: &str, max: u16) -> String {
    if termrock::width(text) <= max {
        return text.to_owned();
    }
    if max < 4 {
        return termrock::truncate(text, max);
    }
    let mut best: Option<usize> = None;
    let mut pos = 0usize;
    while let Some(found) = text[pos..].find(" · ") {
        let at = pos.saturating_add(found);
        let width = termrock::width(text.get(..at).unwrap_or(""));
        if width < max && width.saturating_mul(3) >= max {
            best = Some(at);
        }
        if width.saturating_add(1) > max {
            break;
        }
        pos = at.saturating_add(3);
    }
    match best.and_then(|at| text.get(..at)) {
        Some(head) => format!("{head}…"),
        None => termrock::truncate(text, max),
    }
}

fn wrap_words(text: &str, width: u16) -> Vec<String> {
    let width = usize::from(width.max(1));
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0usize;
    for word in text.split(' ') {
        let word_w = usize::from(termrock::width(word));
        if line.is_empty() {
            if word_w <= width {
                line = word.to_owned();
                used = word_w;
            } else {
                lines.push(termrock::truncate(
                    word,
                    u16::try_from(width).unwrap_or(u16::MAX),
                ));
            }
        } else if used.saturating_add(1).saturating_add(word_w) <= width {
            line.push(' ');
            line.push_str(word);
            used = used.saturating_add(1).saturating_add(word_w);
        } else {
            lines.push(std::mem::take(&mut line));
            used = 0;
            if word_w <= width {
                line = word.to_owned();
                used = word_w;
            } else {
                lines.push(termrock::truncate(
                    word,
                    u16::try_from(width).unwrap_or(u16::MAX),
                ));
            }
        }
    }
    lines.push(line);
    lines
}

fn tag_columns(rows: &[&catalogue::TagRow], content_w: u16) -> TagCols {
    let max_label = rows
        .iter()
        .map(|row| termrock::width(&row.title))
        .max()
        .unwrap_or(14);
    let kind = rows
        .iter()
        .map(|row| termrock::width(row.kind))
        .max()
        .unwrap_or(4);
    let scope = rows
        .iter()
        .map(|row| termrock::width(row.scope_word))
        .max()
        .unwrap_or(4);
    let risk = rows
        .iter()
        .filter_map(|row| row.risk)
        .map(termrock::width)
        .max()
        .unwrap_or(0);
    let label_cap = max_label.min(content_w.saturating_mul(36) / 100).max(14);
    let gaps = 2u16.saturating_mul(3u16.saturating_add(u16::from(risk > 0)));
    let mut label = label_cap;
    let fixed = label
        .saturating_add(kind)
        .saturating_add(scope)
        .saturating_add(risk)
        .saturating_add(gaps);
    let need = fixed.saturating_add(12);
    if need > content_w {
        label = label.saturating_sub(need.saturating_sub(content_w));
    }
    let reason = content_w.saturating_sub(
        label
            .saturating_add(kind)
            .saturating_add(scope)
            .saturating_add(risk)
            .saturating_add(gaps),
    );
    TagCols {
        label,
        kind,
        scope,
        reason,
    }
}

fn split_detail(area: Rect) -> Option<(Rect, Rect)> {
    if area.width < 80 || area.height == 0 {
        return None;
    }
    let detail_w = u16::try_from(u32::from(area.width).saturating_mul(34) / 100).unwrap_or(0);
    let detail_w = detail_w.clamp(34, 48);
    let gap = 2u16;
    let master_w = area.width.saturating_sub(detail_w.saturating_add(gap));
    if master_w < 20 {
        return None;
    }
    Some((
        Rect::new(area.x, area.y, master_w, area.height),
        Rect::new(
            area.x.saturating_add(master_w).saturating_add(gap),
            area.y,
            detail_w,
            area.height,
        ),
    ))
}

/// Single-column band body for the concept screens. The 80–119 band replaces the
/// frozen 80×24 and 100×30 plate frames; overflow gets a black thumb and the preview fade.
fn paint_narrow_tag_list(ui: &mut Ui<'_>, area: Rect, view: &catalogue::ParityDiscovery) -> bool {
    let full = ui.full();
    (80..120).contains(&full.width) && paint_tag_list(ui, area, view, None)
}

/// Frozen columns. Same grid as 100×30. Only the 80×24 frame reaches this painter.
/// `parity-history` body. Single-column tag list below 120 with the frozen
/// columns, the wide card in the 120 band, and the plain action list from 160 up.
fn paint_parity_history(ui: &mut Ui<'_>, area: Rect, view: &catalogue::ParityDiscovery) -> bool {
    let full = ui.full();
    let cols = TagCols {
        label: 12,
        kind: 7,
        scope: 4,
        reason: 65,
    };
    if (80..120).contains(&full.width) {
        return paint_tag_list(ui, area, view, Some(cols));
    }
    if (120..160).contains(&full.width) {
        return paint_parity_card(ui, area, view, cols);
    }
    false
}

/// Frozen columns. The explore chips keep the accent fill. The system row does not.
/// Single-column in the 80–119 band, wide card from 120.
fn paint_parity_files(ui: &mut Ui<'_>, area: Rect, view: &catalogue::ParityDiscovery) -> bool {
    let full = ui.full();
    if (80..120).contains(&full.width) {
        return paint_parity_files_list(ui, area, view);
    }
    if (120..160).contains(&full.width) {
        return paint_parity_files_card(ui, area, view);
    }
    false
}

/// Frozen columns. The explore chips keep the accent fill. The system row does not.
fn paint_parity_files_list(ui: &mut Ui<'_>, area: Rect, view: &catalogue::ParityDiscovery) -> bool {
    let full = ui.full();
    if !(80..120).contains(&full.width) || area.is_empty() {
        return false;
    }
    let Some(text) = narrow_action_line(view) else {
        return false;
    };
    let cols = TagCols {
        label: 12,
        kind: 7,
        scope: 4,
        reason: 65,
    };
    let footer_y = area.bottom().saturating_sub(1);
    let gap_y = footer_y.saturating_sub(1);
    let mut y = area.y;
    for line in &view.lines {
        if y >= gap_y {
            break;
        }
        match line {
            catalogue::FinderLine::Heading(text) => paint_tag_heading(ui, area, y, text, None),
            catalogue::FinderLine::Blank => {}
            catalogue::FinderLine::Chips(labels) => paint_tag_chips_selected(ui, area, y, labels),
            catalogue::FinderLine::Row(row) => paint_tag_row(ui, area, y, row, &cols, None),
        }
        y = y.saturating_add(1);
    }
    ui.paint_str(
        Rect::new(
            area.x.saturating_add(3),
            footer_y,
            area.width.saturating_sub(3),
            1,
        ),
        &text,
        ink(ui, TAG_GREY_FG, BLACK, false),
    );
    true
}

/// Wide card. Explore chips keep the accent fill. The system row does not.
fn paint_parity_files_card(ui: &mut Ui<'_>, area: Rect, view: &catalogue::ParityDiscovery) -> bool {
    let full = ui.full();
    if !(120..160).contains(&full.width) || area.is_empty() {
        return false;
    }
    let Some((master, detail)) = split_detail(area) else {
        return false;
    };
    let rows: Vec<&catalogue::TagRow> = view
        .lines
        .iter()
        .filter_map(|line| match line {
            catalogue::FinderLine::Row(row) => Some(row),
            _ => None,
        })
        .collect();
    let cols = TagCols {
        label: 12,
        kind: 7,
        scope: 4,
        reason: 65,
    };
    let mut y = master.y;
    for line in &view.lines {
        if y >= master.bottom() {
            break;
        }
        match line {
            catalogue::FinderLine::Heading(text) => paint_tag_heading(ui, master, y, text, None),
            catalogue::FinderLine::Blank => {}
            catalogue::FinderLine::Chips(labels) => paint_tag_chips_selected(ui, master, y, labels),
            catalogue::FinderLine::Row(row) => paint_tag_row(ui, master, y, row, &cols, None),
        }
        y = y.saturating_add(1);
    }
    let title = view
        .lines
        .iter()
        .find_map(|line| match line {
            catalogue::FinderLine::Chips(labels) => labels.first().copied(),
            _ => None,
        })
        .unwrap_or("");
    paint_tag_preview(ui, detail, view, &rows, Some(title));
    true
}

/// Selected explore chips. The gutter is accent-on-accent. `›` sits in the gap before the first chip.
fn paint_tag_chips_selected(ui: &mut Ui<'_>, master: Rect, y: u16, labels: &[&str]) {
    let fill = ink(ui, WHITE, SELECT_BG, true);
    ui.fill(Rect::new(master.x, y, master.width, 1), fill);
    ui.paint_str(
        Rect::new(master.x, y, 1, 1),
        " ",
        ink(ui, SELECT_BG, SELECT_BG, true),
    );
    ui.paint_str(Rect::new(master.x.saturating_add(3), y, 1, 1), "›", fill);
    let mut x = master.x.saturating_add(3);
    for label in labels {
        let label_w = termrock::width(label);
        if x.saturating_add(label_w) >= master.right() {
            break;
        }
        ui.paint_str(Rect::new(x.saturating_add(1), y, label_w, 1), label, fill);
        x = x.saturating_add(label_w).saturating_add(3);
    }
}

/// `parity-browser` body. The single-column bands keep the frozen columns; the
/// 120 band keeps the wide card; from 160 up the reference shows the plain
/// action list, so the view painter steps aside.
fn paint_parity_browser(ui: &mut Ui<'_>, area: Rect, view: &catalogue::ParityDiscovery) -> bool {
    let full = ui.full();
    let cols = TagCols {
        label: 12,
        kind: 7,
        scope: 4,
        reason: 65,
    };
    if (80..120).contains(&full.width) {
        return paint_tag_list(ui, area, view, Some(cols));
    }
    if (120..160).contains(&full.width) {
        return paint_parity_card(ui, area, view, cols);
    }
    false
}

/// Wide card band for the replay families. From 120 up the split card keeps
/// the auto-sized tag columns.
fn paint_parity_wide_card(ui: &mut Ui<'_>, area: Rect, view: &catalogue::ParityDiscovery) -> bool {
    let full = ui.full();
    if !(120..160).contains(&full.width) || area.is_empty() {
        return false;
    }
    let Some((master, _)) = split_detail(area) else {
        return false;
    };
    let rows: Vec<&catalogue::TagRow> = view
        .lines
        .iter()
        .filter_map(|line| match line {
            catalogue::FinderLine::Row(row) => Some(row),
            _ => None,
        })
        .collect();
    paint_parity_card(
        ui,
        area,
        view,
        tag_columns(&rows, master.width.saturating_sub(4)),
    )
}

/// Default tag columns in the 80–119 band.
/// The action row middle-truncates `Runs` below 100 so the line fits. Do not write cursor bytes.
fn paint_parity_git_batch(ui: &mut Ui<'_>, area: Rect, view: &catalogue::ParityDiscovery) -> bool {
    let full = ui.full();
    if (80..120).contains(&full.width) {
        if area.is_empty() {
            return false;
        }
        let mut fitted = view.clone();
        fit_narrow_runs(&mut fitted, area.width.saturating_sub(3));
        return paint_tag_list(ui, area, &fitted, None);
    }
    paint_parity_wide_card(ui, area, view)
}

/// Shrink `Runs` so `→ {runs} · {scope} · {risk}` fits the action row.
fn fit_narrow_runs(view: &mut catalogue::ParityDiscovery, room: u16) {
    let scope = view
        .preview
        .iter()
        .find(|line| line.label == "Scope")
        .map(|line| line.value.clone());
    let risk = view
        .preview
        .iter()
        .find(|line| line.label == "Risk")
        .map(|line| line.value.clone());
    let (Some(scope), Some(risk)) = (scope, risk) else {
        return;
    };
    let suffix_w = termrock::width("→ ")
        .saturating_add(termrock::width(" · "))
        .saturating_add(termrock::width(&scope))
        .saturating_add(termrock::width(" · "))
        .saturating_add(termrock::width(&risk));
    let budget = room.saturating_sub(suffix_w);
    if let Some(runs) = view.preview.iter_mut().find(|line| line.label == "Runs") {
        runs.value = termrock::truncate_middle(&runs.value, budget);
    }
}

/// Frozen columns in the 80–119 band.
/// Git-batch, cargo, git-current, browser, files, history, discovery, and every other size stay off.
/// Do not write cursor bytes.
fn paint_parity_task_sources(
    ui: &mut Ui<'_>,
    area: Rect,
    view: &catalogue::ParityDiscovery,
) -> bool {
    let full = ui.full();
    if (80..120).contains(&full.width) {
        return paint_tag_list(
            ui,
            area,
            view,
            Some(TagCols {
                label: 12,
                kind: 7,
                scope: 4,
                reason: 65,
            }),
        );
    }
    false
}

/// Default tag columns in the 80–119 band, wide card from 120.
fn paint_parity_cargo(ui: &mut Ui<'_>, area: Rect, view: &catalogue::ParityDiscovery) -> bool {
    let full = ui.full();
    if (80..120).contains(&full.width) {
        return paint_tag_list(ui, area, view, None);
    }
    paint_parity_wide_card(ui, area, view)
}

/// Default tag columns in the 80–119 band, wide card from 120.
fn paint_parity_git_current(
    ui: &mut Ui<'_>,
    area: Rect,
    view: &catalogue::ParityDiscovery,
) -> bool {
    let full = ui.full();
    if (80..120).contains(&full.width) {
        return paint_tag_list(ui, area, view, None);
    }
    paint_parity_wide_card(ui, area, view)
}

/// Size-free single-column body. Callers keep their own layout policy.
fn paint_tag_list(
    ui: &mut Ui<'_>,
    area: Rect,
    view: &catalogue::ParityDiscovery,
    columns: Option<TagCols>,
) -> bool {
    if area.is_empty() {
        return false;
    }
    let Some(text) = narrow_action_line(view) else {
        return false;
    };
    let rows: Vec<&catalogue::TagRow> = view
        .lines
        .iter()
        .filter_map(|line| match line {
            catalogue::FinderLine::Row(row) => Some(row),
            _ => None,
        })
        .collect();
    let footer_y = area.bottom().saturating_sub(1);
    // Short lists never reach this row. A longer list clips here instead of covering it.
    let gap_y = footer_y.saturating_sub(1);
    let slots = usize::from(gap_y.saturating_sub(area.y));
    let overflow = slots > 0 && view.lines.len() > slots;
    // The thumb owns the last column. Leave the cell before it blank, so a truncated reason stops one earlier.
    let content_w = area.width.saturating_sub(if overflow { 5 } else { 4 });
    let cols = columns.unwrap_or_else(|| tag_columns(&rows, content_w));
    let shown = if overflow { slots } else { view.lines.len() };
    let mut y = area.y;
    for (index, line) in view.lines.iter().enumerate().take(shown) {
        if y >= gap_y {
            break;
        }
        let fade = if overflow {
            preview_fade(index, shown)
        } else {
            None
        };
        if let Some((num, den)) = fade {
            ui.fill(
                Rect::new(area.x, y, area.width.saturating_sub(1), 1),
                ink(ui, mix_on(WHITE, BLACK, num, den), BLACK, false),
            );
        }
        match line {
            catalogue::FinderLine::Heading(text) => paint_tag_heading(ui, area, y, text, fade),
            catalogue::FinderLine::Blank => {}
            catalogue::FinderLine::Chips(labels) => paint_tag_chips(ui, area, y, labels, fade),
            catalogue::FinderLine::Row(row) => paint_tag_row(ui, area, y, row, &cols, fade),
        }
        y = y.saturating_add(1);
    }
    if overflow {
        paint_narrow_scrollbar(
            ui,
            Rect::new(
                area.right().saturating_sub(1),
                area.y,
                1,
                u16::try_from(shown).unwrap_or(u16::MAX),
            ),
            view.lines.len(),
        );
    }
    ui.paint_str(
        Rect::new(
            area.x.saturating_add(3),
            footer_y,
            area.width.saturating_sub(3),
            1,
        ),
        &text,
        ink(ui, TAG_GREY_FG, BLACK, false),
    );
    true
}

fn narrow_action_line(view: &catalogue::ParityDiscovery) -> Option<String> {
    let value = |label: &str| {
        view.preview
            .iter()
            .find(|line| line.label == label)
            .map(|line| line.value.as_str())
    };
    Some(format!(
        "→ {} · {} · {}",
        value("Runs")?,
        value("Scope")?,
        value("Risk")?
    ))
}

/// `hard-cases` body. Below the 120-wide breakpoint the view keeps its single-column
/// tag list; from 120 up it uses the split master/detail card.
fn paint_hard_cases_discovery(
    ui: &mut Ui<'_>,
    area: Rect,
    view: &catalogue::ParityDiscovery,
) -> bool {
    let full = ui.full();
    if full.width >= 80 && full.width < 120 && paint_tag_list(ui, area, view, None) {
        return true;
    }
    paint_parity_discovery(ui, area, view)
}

/// `parity-discovery` body. Below the 120-wide breakpoint the view keeps its
/// single-column tag list; from 120 up it uses the split master/detail card.
fn paint_parity_discovery_view(
    ui: &mut Ui<'_>,
    area: Rect,
    view: &catalogue::ParityDiscovery,
) -> bool {
    if (80..120).contains(&ui.full().width) && paint_tag_list(ui, area, view, None) {
        return true;
    }
    paint_parity_discovery(ui, area, view)
}

fn paint_parity_discovery(ui: &mut Ui<'_>, area: Rect, view: &catalogue::ParityDiscovery) -> bool {
    let Some((master, _)) = split_detail(area) else {
        return false;
    };
    let rows: Vec<&catalogue::TagRow> = view
        .lines
        .iter()
        .filter_map(|line| match line {
            catalogue::FinderLine::Row(row) => Some(row),
            _ => None,
        })
        .collect();
    paint_parity_card(
        ui,
        area,
        view,
        tag_columns(&rows, master.width.saturating_sub(4)),
    )
}

fn paint_parity_card(
    ui: &mut Ui<'_>,
    area: Rect,
    view: &catalogue::ParityDiscovery,
    cols: TagCols,
) -> bool {
    let Some((master, detail)) = split_detail(area) else {
        return false;
    };
    let rows: Vec<&catalogue::TagRow> = view
        .lines
        .iter()
        .filter_map(|line| match line {
            catalogue::FinderLine::Row(row) => Some(row),
            _ => None,
        })
        .collect();
    let mut y = master.y;
    for line in &view.lines {
        if y >= master.bottom() {
            break;
        }
        match line {
            catalogue::FinderLine::Heading(text) => paint_tag_heading(ui, master, y, text, None),
            catalogue::FinderLine::Blank => {}
            catalogue::FinderLine::Chips(labels) => paint_tag_chips(ui, master, y, labels, None),
            catalogue::FinderLine::Row(row) => paint_tag_row(ui, master, y, row, &cols, None),
        }
        y = y.saturating_add(1);
    }
    paint_tag_preview(ui, detail, view, &rows, None);
    true
}

fn paint_tag_heading(ui: &mut Ui<'_>, master: Rect, y: u16, text: &str, fade: Option<(u32, u32)>) {
    let style = ink(ui, faded(TAG_GREY_FG, fade), BLACK, false);
    ui.paint_str(
        Rect::new(master.x.saturating_add(3), y, termrock::width(text), 1),
        text,
        style,
    );
}

fn paint_tag_chips(
    ui: &mut Ui<'_>,
    master: Rect,
    y: u16,
    labels: &[&str],
    fade: Option<(u32, u32)>,
) {
    // Unfocused explore gutter is ink-on-ink, so the cell reads as an empty black column.
    ui.paint_str(
        Rect::new(master.x, y, 1, 1),
        " ",
        ink(ui, BLACK, BLACK, false),
    );
    let style = ink(ui, faded(WHITE, fade), BLACK, false);
    let mut x = master.x.saturating_add(3);
    for label in labels {
        let label_w = termrock::width(label);
        if x.saturating_add(label_w) >= master.right() {
            break;
        }
        ui.paint_str(Rect::new(x.saturating_add(1), y, label_w, 1), label, style);
        x = x.saturating_add(label_w).saturating_add(3);
    }
}

fn paint_tag_row(
    ui: &mut Ui<'_>,
    master: Rect,
    y: u16,
    row: &catalogue::TagRow,
    cols: &TagCols,
    fade: Option<(u32, u32)>,
) {
    // A selected row keeps the solid accent. Fade is only the unselected tail.
    let fade = if row.selected { None } else { fade };
    let bg = if row.selected { SELECT_BG } else { BLACK };
    let title_style = ink(ui, faded(WHITE, fade), bg, row.selected);
    if row.selected {
        ui.fill(Rect::new(master.x, y, master.width, 1), title_style);
        ui.paint_str(
            Rect::new(master.x.saturating_add(1), y, 1, 1),
            "›",
            title_style,
        );
    }
    let mut x = master.x.saturating_add(3);
    let title = termrock::truncate(&row.title, cols.label);
    ui.paint_str(
        Rect::new(x, y, termrock::width(&title).max(1), 1),
        &title,
        title_style,
    );
    x = x.saturating_add(cols.label).saturating_add(2);
    // Tag row chrome bolds only the label. Kind, scope, reason, and risk stay plain.
    let plain = ink(ui, faded(WHITE, fade), bg, false);
    let kind_style = if row.selected {
        plain
    } else {
        ink(ui, faded(TAG_GREY_FG, fade), BLACK, false)
    };
    ui.paint_str(
        Rect::new(x, y, cols.kind, 1),
        &fit_cells(row.kind, cols.kind),
        kind_style,
    );
    x = x.saturating_add(cols.kind).saturating_add(2);
    let scope_fg = if row.selected {
        WHITE
    } else if row.here {
        TAG_GREY_FG
    } else {
        SECONDARY_FG
    };
    let scope_style = if row.selected {
        plain
    } else {
        ink(ui, faded(scope_fg, fade), BLACK, false)
    };
    ui.paint_str(
        Rect::new(x, y, cols.scope, 1),
        &fit_cells(row.scope_word, cols.scope),
        scope_style,
    );
    x = x.saturating_add(cols.scope).saturating_add(2);
    if cols.reason > 0 {
        let reason = truncate_sep(&row.reason, cols.reason);
        let reason_style = if row.selected {
            plain
        } else {
            ink(ui, faded(TAG_GREY_FG, fade), BLACK, false)
        };
        ui.paint_str(
            Rect::new(x, y, termrock::width(&reason).max(1), 1),
            &reason,
            reason_style,
        );
    }
    if let Some(risk) = row.risk {
        let risk_w = termrock::width(risk);
        let risk_x = master.right().saturating_sub(risk_w.saturating_add(1));
        let risk_style = if row.selected {
            plain
        } else {
            ink(ui, faded(SECONDARY_FG, fade), BLACK, false)
        };
        ui.paint_str(Rect::new(risk_x, y, risk_w, 1), risk, risk_style);
    }
}

fn paint_tag_preview(
    ui: &mut Ui<'_>,
    detail: Rect,
    view: &catalogue::ParityDiscovery,
    rows: &[&catalogue::TagRow],
    title_override: Option<&str>,
) {
    if detail.width < 8 || detail.height == 0 {
        return;
    }
    let label_w = view
        .preview
        .iter()
        .map(|line| termrock::width(line.label))
        .max()
        .unwrap_or(4)
        .saturating_add(2);
    let inner_w = detail.width.saturating_sub(4);
    let value_w = inner_w.saturating_sub(label_w.saturating_add(1));
    if value_w == 0 {
        return;
    }
    let flat = flatten_preview(&view.preview, value_w);
    let card_h = u16::try_from(flat.len())
        .unwrap_or(u16::MAX)
        .saturating_add(3)
        .min(detail.height);
    let card = Rect::new(detail.x, detail.y, detail.width, card_h);
    let canvas = ink(ui, WHITE, PREVIEW_BG, false);
    ui.fill(card, canvas);
    let inner_x = detail.x.saturating_add(2);
    let title = title_override.unwrap_or_else(|| {
        rows.iter()
            .find(|row| row.selected)
            .map(|row| row.title.as_str())
            .unwrap_or("")
    });
    let meta_w = termrock::width(&view.preview_meta);
    let title_w = inner_w.saturating_sub(meta_w).saturating_sub(1);
    let title = termrock::truncate(title, title_w);
    ui.paint_str(
        Rect::new(inner_x, detail.y, termrock::width(&title).max(1), 1),
        &title,
        ink(ui, SECONDARY_FG, PREVIEW_BG, false),
    );
    if meta_w > 0 {
        ui.paint_str(
            Rect::new(
                inner_x.saturating_add(inner_w).saturating_sub(meta_w),
                detail.y,
                meta_w,
                1,
            ),
            &view.preview_meta,
            ink(ui, META_FG, PREVIEW_BG, false),
        );
    }
    let rule_y = detail.y.saturating_add(1);
    if rule_y < card.bottom() {
        ui.paint_str(
            Rect::new(inner_x, rule_y, inner_w, 1),
            &"─".repeat(usize::from(inner_w)),
            ink(ui, RULE_FG, PREVIEW_BG, false),
        );
    }
    let slots = usize::from(card_h.saturating_sub(3));
    let shown = flat.len().min(slots);
    let overflow = flat.len() > slots && slots > 0;
    let value_x = inner_x.saturating_add(label_w);
    let content_w = inner_w.saturating_sub(u16::from(overflow));
    for (index, (label, value, tone)) in flat.iter().enumerate().take(shown) {
        let y = detail
            .y
            .saturating_add(2)
            .saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
        if y >= card.bottom() {
            break;
        }
        let fade = overflow.then(|| preview_fade(index, shown)).flatten();
        if let Some((num, den)) = fade {
            ui.fill(
                Rect::new(inner_x, y, content_w, 1),
                ink(ui, mix_color(WHITE, num, den), PREVIEW_BG, false),
            );
        }
        if !label.is_empty() {
            ui.paint_str(
                Rect::new(inner_x, y, termrock::width(label), 1),
                label,
                ink(
                    ui,
                    mix_color(TAG_GREY_FG, fade_num(fade), fade_den(fade)),
                    PREVIEW_BG,
                    false,
                ),
            );
        }
        if value.is_empty() {
            continue;
        }
        let fg = mix_color(tone_color(*tone), fade_num(fade), fade_den(fade));
        ui.paint_str(
            Rect::new(value_x, y, termrock::width(value).max(1), 1),
            value,
            ink(ui, fg, PREVIEW_BG, false),
        );
    }
    if overflow {
        paint_preview_scrollbar(
            ui,
            Rect::new(
                detail.x.saturating_add(detail.width).saturating_sub(3),
                detail.y.saturating_add(2),
                1,
                u16::try_from(shown).unwrap_or(u16::MAX),
            ),
            flat.len(),
        );
    }
}

fn fade_num(fade: Option<(u32, u32)>) -> u32 {
    fade.map_or(1, |(num, _)| num)
}

fn fade_den(fade: Option<(u32, u32)>) -> u32 {
    fade.map_or(1, |(_, den)| den)
}

fn tone_color(tone: catalogue::PreviewTone) -> Color {
    match tone {
        catalogue::PreviewTone::Text => WHITE,
        catalogue::PreviewTone::Muted => TAG_GREY_FG,
        catalogue::PreviewTone::Secondary => SECONDARY_FG,
        catalogue::PreviewTone::Meta => META_FG,
        catalogue::PreviewTone::Danger => DANGER_FG,
    }
}

/// Bottom edge of an overflowing preview. `4/5` is the inner keep, `11/20` the outer.
fn preview_fade(index: usize, shown: usize) -> Option<(u32, u32)> {
    if shown < 4 {
        return None;
    }
    let from_end = shown.saturating_sub(index);
    if from_end == 1 {
        Some((11, 20))
    } else if shown >= 12 && from_end == 2 {
        Some((4, 5))
    } else {
        None
    }
}

fn mix_color(fg: Color, num: u32, den: u32) -> Color {
    mix_on(fg, PREVIEW_BG, num, den)
}

fn mix_on(fg: Color, back: Color, num: u32, den: u32) -> Color {
    if num == den {
        return fg;
    }
    let Color::Rgb(red, green, blue) = fg else {
        return fg;
    };
    let Color::Rgb(back_r, back_g, back_b) = back else {
        return fg;
    };
    Color::Rgb(
        mix_channel(red, back_r, num, den),
        mix_channel(green, back_g, num, den),
        mix_channel(blue, back_b, num, den),
    )
}

fn faded(color: Color, fade: Option<(u32, u32)>) -> Color {
    match fade {
        Some((num, den)) => mix_on(color, BLACK, num, den),
        None => color,
    }
}

fn mix_channel(fg: u8, bg: u8, num: u32, den: u32) -> u8 {
    let den = den.max(1);
    let mixed = u32::from(bg)
        .saturating_mul(den.saturating_sub(num))
        .saturating_add(u32::from(fg).saturating_mul(num));
    let rounded = mixed.saturating_add(den / 2) / den;
    u8::try_from(rounded.min(u32::from(u8::MAX))).unwrap_or(u8::MAX)
}

const SCROLL_THUMB: Color = Color::Rgb(0x80, 0x80, 0x80);
const SCROLL_TRACK: Color = Color::Rgb(0x47, 0x47, 0x47);

fn paint_preview_scrollbar(ui: &mut Ui<'_>, area: Rect, content_len: usize) {
    if area.is_empty() {
        return;
    }
    let mut scroll = termrock::ScrollState::new(content_len);
    scroll.set_viewport(usize::from(area.height));
    let (start, len) = scroll.thumb(usize::from(area.height));
    let thumb_end = start.saturating_add(len);
    for row in 0..area.height {
        let on_thumb = (start..thumb_end).contains(&usize::from(row));
        let (glyph, color) = if on_thumb {
            ("┃", SCROLL_THUMB)
        } else {
            ("┆", SCROLL_TRACK)
        };
        ui.paint_str(
            Rect::new(area.x, area.y.saturating_add(row), 1, 1),
            glyph,
            ink(ui, color, PREVIEW_BG, false),
        );
    }
}

/// List thumb on the narrow plate. Same geometry as the preview thumb, on black not `#111111`.
fn paint_narrow_scrollbar(ui: &mut Ui<'_>, area: Rect, content_len: usize) {
    if area.is_empty() {
        return;
    }
    let mut scroll = termrock::ScrollState::new(content_len);
    scroll.set_viewport(usize::from(area.height));
    let (start, len) = scroll.thumb(usize::from(area.height));
    let thumb_end = start.saturating_add(len);
    for row in 0..area.height {
        let on_thumb = (start..thumb_end).contains(&usize::from(row));
        let (glyph, color) = if on_thumb {
            ("┃", SCROLL_THUMB)
        } else {
            ("┆", SCROLL_TRACK)
        };
        ui.paint_str(
            Rect::new(area.x, area.y.saturating_add(row), 1, 1),
            glyph,
            ink(ui, color, BLACK, false),
        );
    }
}

fn flatten_preview(
    lines: &[catalogue::PreviewLine],
    value_w: u16,
) -> Vec<(String, String, catalogue::PreviewTone)> {
    let mut flat = Vec::new();
    for line in lines {
        if line.wraps {
            let parts = wrap_words(&line.value, value_w);
            for (index, part) in parts.into_iter().enumerate() {
                flat.push((
                    if index == 0 {
                        line.label.to_owned()
                    } else {
                        String::new()
                    },
                    part,
                    line.tone,
                ));
            }
        } else {
            flat.push((
                line.label.to_owned(),
                termrock::truncate(&line.value, value_w),
                line.tone,
            ));
        }
    }
    flat
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
