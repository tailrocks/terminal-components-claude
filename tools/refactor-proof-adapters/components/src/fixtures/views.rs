//! View fixtures: grid, table, code, diff, viewport, panels, scroll, too-small.

use junie_tui::{
    App, CellRef, CodeEditor, CodeEditorState, Column, ColumnKey, Cx, DiffLineKind, DiffRow,
    DiffSource, DiffView, DiffViewState, EditIntent, Family as ThemeFamily, Grid, GridEditor,
    GridModel, GridState, Id, ItemKey, Panel, PanelKind, Part, Response, ScrollRegion, ScrollState,
    SelectMode, SplitAxis, SplitPane, SplitPaneState, StateFlags, TextViewport, TooSmall, Ui,
    Variant, ViewportLine, ViewportState,
};

use crate::expansion::{Facet, FadePosition, ViewportMutation};
use crate::fixtures::{FixtureApp, FixtureCfg};
use crate::states::ComponentState;

const GRID: Id = Id::root("oracle.components.grid");
const TABLE: Id = Id::root("oracle.components.table");
const CODE: Id = Id::root("oracle.components.code");
const DIFF: Id = Id::root("oracle.components.diff");
const VIEWPORT: Id = Id::root("oracle.components.viewport");
const SCROLL_PANEL: Id = Id::root("oracle.components.scrollpanel");
const SCROLL_PANEL_VIEW: Id = Id::root("oracle.components.scrollpanel.view");
const PANEL: Id = Id::root("oracle.components.panel");
const SPLIT: Id = Id::root("oracle.components.split");
const REGION: Id = Id::root("oracle.components.region");
const TOO_SMALL: Id = Id::root("oracle.components.toosmall");

const GRID_ROWS: [(&str, &str); 6] = [
    ("Ada Lovelace", "analyst"),
    ("Grace Hopper", "rear admiral"),
    ("Hedy Lamarr", "inventor"),
    ("Alan Turing", "logician"),
    ("Edsger Dijkstra", "programmer"),
    ("Barbara Liskov", "professor"),
];

const GRID_COLUMNS: [Column<'static>; 2] = [
    Column::new(ColumnKey::num(1), "Name"),
    Column::new(ColumnKey::num(2), "Role"),
];

/// Grid model over owned rows with unique text keys.
#[derive(Debug)]
struct PeopleModel {
    rows: Vec<(String, String)>,
}

impl PeopleModel {
    fn fixture_rows(count: usize) -> Vec<(String, String)> {
        GRID_ROWS
            .iter()
            .cycle()
            .take(count)
            .enumerate()
            .map(|(index, base)| {
                if count <= GRID_ROWS.len() {
                    (base.0.to_owned(), base.1.to_owned())
                } else {
                    (format!("{} {index:02}", base.0), base.1.to_owned())
                }
            })
            .collect()
    }
}

impl GridModel for PeopleModel {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        self.rows
            .get(row)
            .map_or(ItemKey::index(row), |item| ItemKey::text(&item.0))
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let row = self.rows.get(row)?;
        match col {
            0 => Some(CellRef::new(row.0.as_str())),
            1 => Some(CellRef::new(row.1.as_str())),
            _ => None,
        }
    }
}

impl GridEditor for PeopleModel {
    fn edit_intent(&self, _row: usize, _col: usize) -> EditIntent<'_> {
        EditIntent::Inline { initial: "" }
    }

    fn apply_cycle(&mut self, _row: usize, _col: usize) {}

    fn commit_cell(
        &mut self,
        _row: usize,
        _col: usize,
        _text: &str,
    ) -> Result<(), junie_tui::FieldError> {
        Ok(())
    }

    fn is_editable(&self, _row: usize, _col: usize) -> bool {
        true
    }
}

/// Fade-facet row count: short for `Fits`, tall otherwise.
const fn fade_rows(facet: Facet) -> usize {
    match facet {
        Facet::FadeViewport { position, .. } => match position {
            FadePosition::Fits => 2,
            FadePosition::Top | FadePosition::Middle | FadePosition::Bottom => 40,
        },
        _ => 6,
    }
}

/// Keyed grid fixture with an inline editor.
#[derive(Debug)]
pub struct GridFixture {
    cfg: FixtureCfg,
    state: GridState,
    model: PeopleModel,
    columns: [Column<'static>; 2],
}

impl GridFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        let mut first = Column::new(ColumnKey::num(1), "Name");
        first.editable = true;
        let mut second = Column::new(ColumnKey::num(2), "Role");
        second.editable = true;
        Self {
            cfg,
            state: GridState::default(),
            model: PeopleModel {
                rows: PeopleModel::fixture_rows(fade_rows(cfg.facet)),
            },
            columns: [first, second],
        }
    }

    fn widget(&self) -> Grid<'_> {
        Grid::new(GRID, &self.columns)
            .select_mode(SelectMode::Multi)
            .disabled(self.cfg.state == ComponentState::Disabled)
    }
}

impl App for GridFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let Self {
            cfg,
            state,
            model,
            columns,
        } = self;
        Grid::new(GRID, columns)
            .select_mode(SelectMode::Multi)
            .disabled(cfg.state == ComponentState::Disabled)
            .update_editable(cx, state, model)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget()
            .draw(ui, self.cfg.widget_rect(), &self.state, &self.model);
    }
}

impl FixtureApp for GridFixture {
    fn fixture_id(&self) -> Id {
        GRID
    }

    fn check(&self, state: ComponentState) -> bool {
        match state {
            ComponentState::Current => self.state.cursor().is_some(),
            ComponentState::Selected => !self.state.selected_rows().is_empty(),
            ComponentState::Editing => self.state.is_editing(),
            _ => true,
        }
    }
}

/// Read-only table fixture: grid in table policy, no inline editor.
#[derive(Debug)]
pub struct TableFixture {
    cfg: FixtureCfg,
    state: GridState,
    model: PeopleModel,
}

impl TableFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: GridState::default(),
            model: PeopleModel {
                rows: PeopleModel::fixture_rows(fade_rows(cfg.facet)),
            },
        }
    }

    fn widget(&self) -> Grid<'static> {
        Grid::new(TABLE, &GRID_COLUMNS)
            .select_mode(SelectMode::Single)
            .disabled(self.cfg.state == ComponentState::Disabled)
    }
}

impl App for TableFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        // Read-only table policy: `update`, never `update_editable`.
        self.widget()
            .update(cx, &mut self.state, &self.model)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.widget()
            .draw(ui, self.cfg.widget_rect(), &self.state, &self.model);
    }
}

impl FixtureApp for TableFixture {
    fn fixture_id(&self) -> Id {
        TABLE
    }

    fn check(&self, state: ComponentState) -> bool {
        match state {
            ComponentState::Current => self.state.cursor().is_some(),
            ComponentState::Selected => !self.state.selected_rows().is_empty(),
            _ => true,
        }
    }
}

const CODE_TEXT: &str =
    "fn retry() {\n  let attempts = 5; // 世界 🌍\n}\nfn backoff() {\n  sleep(2);\n}\n";

/// Code editor fixture.
#[derive(Debug)]
pub struct CodeFixture {
    cfg: FixtureCfg,
    state: CodeEditorState,
}

impl CodeFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        let text = if cfg.state == ComponentState::Empty {
            String::new()
        } else if let Facet::FadeViewport { position, .. } = cfg.facet {
            match position {
                FadePosition::Fits => "fn retry() {\n}\n".to_owned(),
                FadePosition::Top | FadePosition::Middle | FadePosition::Bottom => {
                    CODE_TEXT.repeat(6)
                }
            }
        } else {
            CODE_TEXT.to_owned()
        };
        Self {
            cfg,
            state: CodeEditorState::new(&text),
        }
    }
}

impl App for CodeFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        CodeEditor::new(CODE, self.cfg.widget_rect().height.max(2))
            .placeholder("No source")
            .disabled(self.cfg.state == ComponentState::Disabled)
            .read_only(self.cfg.state == ComponentState::ReadOnly)
            .update(cx, &mut self.state)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        CodeEditor::new(CODE, self.cfg.widget_rect().height.max(2))
            .placeholder("No source")
            .disabled(self.cfg.state == ComponentState::Disabled)
            .read_only(self.cfg.state == ComponentState::ReadOnly)
            .draw(ui, self.cfg.widget_rect(), &self.state);
    }
}

impl FixtureApp for CodeFixture {
    fn fixture_id(&self) -> Id {
        CODE
    }

    fn check(&self, state: ComponentState) -> bool {
        match state {
            ComponentState::Editing | ComponentState::Selected => self.state.is_editing(),
            ComponentState::ReadOnly => !self.state.is_editing(),
            _ => true,
        }
    }
}

const DIFF_ROWS: [DiffRow<'static>; 5] = [
    DiffRow::Hunk {
        old_start: 12,
        new_start: 12,
    },
    DiffRow::Line {
        kind: DiffLineKind::Context,
        text: "fn retry() {",
    },
    DiffRow::Line {
        kind: DiffLineKind::Remove,
        text: "    let attempts = 3;",
    },
    DiffRow::Line {
        kind: DiffLineKind::Add,
        text: "    let attempts = 5;",
    },
    DiffRow::Line {
        kind: DiffLineKind::Context,
        text: "}",
    },
];

#[derive(Debug)]
struct OracleDiff;

impl DiffSource for OracleDiff {
    fn revision(&self) -> u64 {
        1
    }

    fn path(&self) -> &'static str {
        "src/retry.rs"
    }

    fn status_marker(&self) -> &'static str {
        "M"
    }

    fn status_label(&self) -> &'static str {
        "modified"
    }

    fn row_count(&self) -> usize {
        DIFF_ROWS.len()
    }

    fn row(&self, index: usize) -> Option<DiffRow<'_>> {
        DIFF_ROWS.get(index).copied()
    }
}

/// Adaptive diff view fixture.
#[derive(Debug)]
pub struct DiffFixture {
    cfg: FixtureCfg,
    state: DiffViewState,
    source: OracleDiff,
}

impl DiffFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: DiffViewState::default(),
            source: OracleDiff,
        }
    }
}

impl App for DiffFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        DiffView::new(DIFF, Some(&self.source as &dyn DiffSource))
            .update(cx, &mut self.state)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        DiffView::new(DIFF, Some(&self.source as &dyn DiffSource)).draw(
            ui,
            self.cfg.widget_rect(),
            &self.state,
        );
    }
}

impl FixtureApp for DiffFixture {
    fn fixture_id(&self) -> Id {
        DIFF
    }
}

const VIEWPORT_BASE: [&str; 8] = [
    "2026-09-04 10:00 connected",
    "2026-09-04 10:01 loading workspace 世界",
    "2026-09-04 10:02 indexed 128 files 🌍",
    "2026-09-04 10:03 running checks",
    "2026-09-04 10:04 check 1 passed",
    "2026-09-04 10:05 check 2 passed",
    "2026-09-04 10:06 check 3 passed",
    "2026-09-04 10:07 ready",
];

/// Retained-output text viewport fixture.
#[derive(Debug)]
pub struct ViewportFixture {
    cfg: FixtureCfg,
    state: ViewportState,
    lines: Vec<ViewportLine<'static>>,
}

impl ViewportFixture {
    /// Build for `cfg`, applying retained-output mutations through the
    /// production caller-managed contract (`invalidate` after mutation).
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        let mut owned: Vec<String> = VIEWPORT_BASE.iter().map(ToString::to_string).collect();
        if let Facet::FadeViewport { position, .. } = cfg.facet {
            owned = match position {
                FadePosition::Fits => owned.into_iter().take(2).collect(),
                FadePosition::Top | FadePosition::Middle | FadePosition::Bottom => (0..40)
                    .map(|index| format!("log line {index:02} 世界"))
                    .collect(),
            };
        }
        if let Facet::ViewportMutation { kind } = cfg.facet {
            apply_mutation(&mut owned, kind);
        }
        if cfg.state == ComponentState::Empty {
            owned.clear();
        }
        // Borrowed projection: lines borrow fixture-owned storage through
        // one cloned static per line (short-lived capture process).
        let lines: Vec<ViewportLine<'static>> = owned
            .iter()
            .map(|line| ViewportLine::Plain(clone_static(line.as_str())))
            .collect();
        let mut state = ViewportState::default();
        // Fade positions need a fixed offset; tail-follow would pin every
        // position to the end. Follow itself is covered by ScrollPanel.
        state.set_follow(false);
        let mut fixture = Self { cfg, state, lines };
        if matches!(cfg.facet, Facet::ViewportMutation { .. }) {
            fixture.state.invalidate();
        }
        fixture
    }
}

impl App for ViewportFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        TextViewport::new(VIEWPORT)
            .wrap(true)
            .update(cx, &mut self.state, &self.lines)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextViewport::new(VIEWPORT).wrap(true).draw(
            ui,
            self.cfg.widget_rect(),
            &self.state,
            &self.lines,
        );
    }
}

impl FixtureApp for ViewportFixture {
    fn fixture_id(&self) -> Id {
        VIEWPORT
    }

    fn check(&self, state: ComponentState) -> bool {
        if state == ComponentState::Selected {
            return self.state.selection().is_some();
        }
        true
    }
}

/// Extend a line to `'static` for the borrowed projection.
fn clone_static(line: &str) -> &'static str {
    Box::leak(line.to_owned().into_boxed_str())
}

fn apply_mutation(owned: &mut Vec<String>, kind: ViewportMutation) {
    match kind {
        ViewportMutation::Append => {
            owned.push("2026-09-04 10:08 appended".to_owned());
        }
        ViewportMutation::ReplaceLast => {
            if let Some(last) = owned.last_mut() {
                "2026-09-04 10:07 replaced".clone_into(last);
            }
        }
        ViewportMutation::FrontEvict => {
            if !owned.is_empty() {
                owned.remove(0);
            }
        }
        ViewportMutation::WholesaleReplace => {
            owned.clear();
            owned.push("fresh output only".to_owned());
        }
        ViewportMutation::SameLength => {
            for line in owned.iter_mut() {
                let len = line.len();
                *line = "x".repeat(len);
            }
        }
    }
}

/// Legacy scroll-panel fixture: `Panel` plus `TextViewport` with an explicit
/// follow policy (Active tails, Inactive never follows).
#[derive(Debug)]
pub struct ScrollPanelFixture {
    cfg: FixtureCfg,
    state: ViewportState,
    lines: Vec<ViewportLine<'static>>,
}

impl ScrollPanelFixture {
    /// Build for `cfg`.
    ///
    /// The follow policy only manifests across a retained-output mutation:
    /// Active appends at the tail and follows it; Inactive pins its offset
    /// across a front insertion and shows the shifted head. Both mutations
    /// go through the production caller-managed [`ViewportState::invalidate`]
    /// contract.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        let mut state = ViewportState::default();
        state.set_follow(cfg.state == ComponentState::Active);
        let lines: Vec<ViewportLine<'static>> = match cfg.facet {
            Facet::FadeViewport { position, .. } => match position {
                FadePosition::Fits => vec![
                    ViewportLine::Plain("panel log line 00"),
                    ViewportLine::Plain("panel log line 01"),
                ],
                FadePosition::Top | FadePosition::Middle | FadePosition::Bottom => (0..40)
                    .map(|index| {
                        ViewportLine::Plain(clone_static(&format!("panel log line {index:02}")))
                    })
                    .collect(),
            },
            _ => match cfg.state {
                ComponentState::Active => VIEWPORT_BASE
                    .iter()
                    .copied()
                    .map(ViewportLine::Plain)
                    .chain([
                        ViewportLine::Plain("2026-09-04 10:08 live line"),
                        ViewportLine::Plain("2026-09-04 10:09 live line"),
                        ViewportLine::Plain("2026-09-04 10:10 live line"),
                    ])
                    .collect(),
                ComponentState::Inactive => [
                    ViewportLine::Plain("2026-09-04 09:58 caught up"),
                    ViewportLine::Plain("2026-09-04 09:59 caught up"),
                ]
                .into_iter()
                .chain(VIEWPORT_BASE.iter().copied().map(ViewportLine::Plain))
                .collect(),
                _ => VIEWPORT_BASE
                    .iter()
                    .copied()
                    .map(ViewportLine::Plain)
                    .collect(),
            },
        };
        let mut fixture = Self { cfg, state, lines };
        if matches!(cfg.state, ComponentState::Active | ComponentState::Inactive)
            && matches!(cfg.facet, Facet::None)
        {
            fixture.state.invalidate();
        }
        fixture
    }
}

impl App for ScrollPanelFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        TextViewport::new(SCROLL_PANEL_VIEW)
            .wrap(true)
            .update(cx, &mut self.state, &self.lines)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = self.cfg.widget_rect();
        Panel::new(SCROLL_PANEL)
            .kind(PanelKind::Framed)
            .title("Output")
            .draw(ui, area, |ui, body| {
                TextViewport::new(SCROLL_PANEL_VIEW).wrap(true).draw(
                    ui,
                    body,
                    &self.state,
                    &self.lines,
                );
            });
    }
}

impl FixtureApp for ScrollPanelFixture {
    fn fixture_id(&self) -> Id {
        SCROLL_PANEL_VIEW
    }
}

/// Container panel fixture.
#[derive(Debug)]
pub struct PanelFixture {
    cfg: FixtureCfg,
}

impl PanelFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self { cfg }
    }
}

impl App for PanelFixture {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Panel::new(PANEL)
            .kind(PanelKind::Framed)
            .title("Inspector")
            .meta("read only")
            .draw(ui, self.cfg.widget_rect(), |ui, body| {
                let style = ui
                    .style(
                        ThemeFamily::PANEL,
                        Variant::DEFAULT,
                        Part::TITLE,
                        StateFlags::empty(),
                    )
                    .style;
                ui.paint_str(body, "Selected object details", style);
            });
    }
}

impl FixtureApp for PanelFixture {
    fn fixture_id(&self) -> Id {
        PANEL
    }
}

/// Split pane fixture with a pointer-driven seam.
#[derive(Debug)]
pub struct SplitFixture {
    cfg: FixtureCfg,
    state: SplitPaneState,
}

impl SplitFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: SplitPaneState::default(),
        }
    }
}

impl App for SplitFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        SplitPane::new(SPLIT, SplitAxis::Horizontal)
            .gap(1)
            .min_first(8)
            .min_second(8)
            .resizable(true)
            .update(cx, &mut self.state)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        SplitPane::new(SPLIT, SplitAxis::Horizontal)
            .gap(1)
            .min_first(8)
            .min_second(8)
            .resizable(true)
            .draw(
                ui,
                self.cfg.widget_rect(),
                &self.state,
                |ui, first, second| {
                    let style = ui
                        .style(
                            ThemeFamily::SPLIT,
                            Variant::DEFAULT,
                            Part::SEAM,
                            StateFlags::empty(),
                        )
                        .style;
                    ui.paint_str(first, "Primary pane", style);
                    ui.paint_str(second, "Secondary pane", style);
                },
            );
    }
}

impl FixtureApp for SplitFixture {
    fn fixture_id(&self) -> Id {
        SPLIT
    }
}

const REGION_ROWS: usize = 40;

/// Scroll region fixture painting numbered content rows.
#[derive(Debug)]
pub struct ScrollRegionFixture {
    cfg: FixtureCfg,
    state: ScrollState,
}

impl ScrollRegionFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self {
            cfg,
            state: ScrollState::new(Self::content_len_for(cfg)),
        }
    }

    const fn content_len_for(cfg: FixtureCfg) -> usize {
        match cfg.facet {
            Facet::FadeViewport { position, .. } => match position {
                FadePosition::Fits => 2,
                FadePosition::Top | FadePosition::Middle | FadePosition::Bottom => REGION_ROWS,
            },
            // Hover/pressed target the thumb, which only registers when
            // content overflows the viewport.
            Facet::ViewportMutation { .. } => REGION_ROWS,
            Facet::None => match cfg.state {
                ComponentState::Hover | ComponentState::Pressed => REGION_ROWS + 20,
                _ => REGION_ROWS,
            },
        }
    }

    const fn content_len(&self) -> usize {
        Self::content_len_for(self.cfg)
    }
}

impl App for ScrollRegionFixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let content_len = self.content_len();
        ScrollRegion::new(REGION)
            .update(cx, &mut self.state, content_len)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let content_len = self.content_len();
        let area = self.cfg.widget_rect();
        let content = ScrollRegion::new(REGION).draw(ui, area, &self.state, content_len);
        let style = ui
            .style(
                ThemeFamily::LIST,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::empty(),
            )
            .style;
        let view = ScrollRegion::view(&self.state, content, content_len);
        for (row, index) in content.rows().zip(view.visible_range()) {
            ui.paint_str(row, &format!("row {index} 世界"), style);
        }
    }
}

impl FixtureApp for ScrollRegionFixture {
    fn fixture_id(&self) -> Id {
        REGION
    }
}

/// Narrow-allocation fallback fixture.
#[derive(Debug)]
pub struct TooSmallFixture {
    cfg: FixtureCfg,
}

impl TooSmallFixture {
    /// Build for `cfg`.
    #[must_use]
    pub fn new(cfg: FixtureCfg) -> Self {
        Self { cfg }
    }
}

impl App for TooSmallFixture {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TooSmall::new(TOO_SMALL, "Junie").draw(ui, self.cfg.widget_rect());
    }
}

impl FixtureApp for TooSmallFixture {
    fn fixture_id(&self) -> Id {
        TOO_SMALL
    }
}
