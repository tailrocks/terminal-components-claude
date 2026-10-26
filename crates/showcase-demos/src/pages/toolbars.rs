use termrock::core::keymap::NavKeys;
use termrock::{
    Cx, FrameRead, Id, ItemKey, Panel, PanelKind, Rect, Response, Select, SelectState, StateFlags,
    Tabs, TabsState, ToolItem, Toolbar, ToolbarAction, ToolbarActionEvent, ToolbarReferenceState,
    ToolbarState, Track, Ui, layout, wrap,
};

use super::{Page, PageStatus, PageUpdate, frame};

const PAGE_TABS: Id = Id::root("showcase_ui::pages::toolbars.page_tabs");
const QUERY: Id = Id::root("showcase_ui::pages::toolbars.query");
const NARROW: Id = Id::root("showcase_ui::pages::toolbars.narrow");
const FORMAT: Id = Id::root("showcase_ui::pages::toolbars.format");
const QUERY_PANEL: Id = Id::root("showcase_ui::pages::toolbars.query_panel");
const NARROW_PANEL: Id = Id::root("showcase_ui::pages::toolbars.narrow_panel");
const FORMAT_PANEL: Id = Id::root("showcase_ui::pages::toolbars.format_panel");
const LAST_PANEL: Id = Id::root("showcase_ui::pages::toolbars.last_panel");
const NAV_OPTION: Id = Id::root("showcase_ui::pages::toolbars.option.nav");
const WRAP_OPTION: Id = Id::root("showcase_ui::pages::toolbars.option.wrap");
const SKIP_OPTION: Id = Id::root("showcase_ui::pages::toolbars.option.skip_disabled");
const GAP_OPTION: Id = Id::root("showcase_ui::pages::toolbars.option.gap");
const OVERFLOW_OPTION: Id = Id::root("showcase_ui::pages::toolbars.option.overflow");
const STATE_REFERENCE: Id = Id::root("showcase_ui::pages::toolbars.states");

const BLURB: &str =
    "A row of commands that is one focus stop: shortcuts, tips, toggles, separators";
const SECTIONS: &[&str] = &["Playground", "Options", "States"];
const QUERY_META: &str = "one Tab stop · ←→ · Enter";
const NARROW_META: &str = "» holds what does not fit";
const NARROW_META_ELLIPSIS: &str = "… holds what does not fit";
const NARROW_META_OFF: &str = "overflow off: no More button";
const FORMAT_META: &str = "toggles";
const NARROW_WIDTH: u16 = 32;
const OPTION_DEFAULTS: [usize; 5] = [0, 0, 1, 1, 0];
const NAV_ITEMS: &[&str] = &["Desktop", "Vim"];
const WRAP_ITEMS: &[&str] = &["off", "on"];
const SKIP_ITEMS: &[&str] = &["off", "on"];
const GAP_ITEMS: &[&str] = &["0", "1", "2"];
const OVERFLOW_ITEMS: &[&str] = &["»", "…", "off"];
fn nav_keys() -> [ItemKey; 7] {
    [
        ItemKey::text("query.run"),
        ItemKey::text("query.run_all"),
        ItemKey::text("query.stop"),
        ItemKey::text("query.format"),
        ItemKey::text("query.explain"),
        ItemKey::text("query.wrap"),
        ItemKey::text("query.line_numbers"),
    ]
}

fn format_keys() -> [ItemKey; 6] {
    [
        ItemKey::text("format.bold"),
        ItemKey::text("format.italic"),
        ItemKey::text("format.code"),
        ItemKey::text("format.left"),
        ItemKey::text("format.center"),
        ItemKey::text("format.right"),
    ]
}
const OPTION_IDS: [Id; 5] = [
    NAV_OPTION,
    WRAP_OPTION,
    SKIP_OPTION,
    GAP_OPTION,
    OVERFLOW_OPTION,
];
const OPTION_LABELS: [&str; 5] = [
    "Navigation keys",
    "Wrap at row ends",
    "Skip disabled items",
    "Gap between items",
    "Overflow button",
];
const STATE_NAMES: [&str; 9] = [
    "Rest",
    "Hover",
    "Pressed",
    "Focused",
    "FocusedHover",
    "Selected",
    "Disabled",
    "Overflow",
    "Narrow",
];
const STATE_VALUES: [ToolbarReferenceState; 9] = ToolbarReferenceState::ALL;
const TAB_HINTS: &[(&str, &str)] = &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
const TOOLBAR_HINTS: &[(&str, &str)] = &[
    ("← →", "Move"),
    ("Home / End", "First / last"),
    ("Enter / Space", "Run"),
];
const FORMAT_HINTS: &[(&str, &str)] = &[
    ("← →", "Move"),
    ("Home / End", "First / last"),
    ("Enter / Space", "Toggle"),
];
const MORE_HINTS: &[(&str, &str)] = &[("← →", "Move"), ("Enter / ↓", "Show hidden")];
const MENU_HINTS: &[(&str, &str)] = &[("↑ ↓", "Move"), ("Enter", "Run"), ("Esc", "Close")];
const SELECT_HINTS: &[(&str, &str)] = &[("↑ ↓", "Choose"), ("Enter", "Open / choose")];

fn query_items(running: bool, wrap: bool, line_numbers: bool) -> Vec<ToolItem> {
    let mut run = ToolItem::new("Run")
        .shortcut("F5")
        .tip("Run the statement under the cursor");
    let mut run_all = ToolItem::new("Run all")
        .shortcut("Shift+F5")
        .tip("Run every statement");
    let stop = ToolItem::new("Stop")
        .tip("Stop the running query")
        .separator();
    let stop = if running {
        stop
    } else {
        stop.reason("No query is running")
    };
    if running {
        run = run.reason("A query is running");
        run_all = run_all.reason("A query is running");
    }
    vec![
        run,
        run_all,
        stop,
        ToolItem::new("Format").tip("Format the script"),
        ToolItem::new("Explain")
            .tip("Show the query plan")
            .separator(),
        ToolItem::new("Wrap").toggle(wrap).tip("Wrap long lines"),
        ToolItem::new("Line numbers").toggle(line_numbers),
    ]
}

fn format_items(values: [bool; 6]) -> Vec<ToolItem> {
    vec![
        ToolItem::new("Bold").toggle(values[0]),
        ToolItem::new("Italic").toggle(values[1]),
        ToolItem::new("Code").toggle(values[2]).separator(),
        ToolItem::new("Left").toggle(values[3]),
        ToolItem::new("Center").toggle(values[4]),
        ToolItem::new("Right").toggle(values[5]),
    ]
}

fn build_toolbar(id: Id, items: Vec<ToolItem>, keys: &[ItemKey], options: [usize; 5]) -> Toolbar {
    let mut toolbar = Toolbar::new(id, items)
        .nav(if options[0] == 1 {
            NavKeys::VIM
        } else {
            NavKeys::DESKTOP
        })
        .wrap(options[1] == 1)
        .skip_disabled(options[2] == 1)
        .gap(options[3].min(2) as u16)
        .overflow(options[4] != 2);
    if options[4] == 1 {
        toolbar = toolbar.more_glyph("…");
    }
    toolbar
        .try_item_keys(keys.to_vec())
        .expect("the page's static Toolbar keys are unique and match its rows")
}

fn option_data(index: usize) -> &'static [&'static str] {
    match index {
        0 => NAV_ITEMS,
        1 => WRAP_ITEMS,
        2 => SKIP_ITEMS,
        3 => GAP_ITEMS,
        _ => OVERFLOW_ITEMS,
    }
}

fn selected_section(state: &TabsState) -> usize {
    match state.active().or(state.cursor()) {
        Some(ItemKey::Index(index)) => index.min(SECTIONS.len().saturating_sub(1)),
        _ => 0,
    }
}

pub struct ToolbarsPage {
    page_tabs: TabsState,
    query_state: ToolbarState,
    narrow_state: ToolbarState,
    format_state: ToolbarState,
    running: bool,
    query_wrap: bool,
    query_line_numbers: bool,
    narrow_wrap: bool,
    narrow_line_numbers: bool,
    format_values: [bool; 6],
    options: [usize; 5],
    select_states: [SelectState; 5],
    last: String,
}

impl ToolbarsPage {
    pub fn new() -> Self {
        Self {
            page_tabs: TabsState::default(),
            query_state: ToolbarState::default(),
            narrow_state: ToolbarState::default(),
            format_state: ToolbarState::default(),
            running: false,
            query_wrap: false,
            query_line_numbers: true,
            narrow_wrap: false,
            narrow_line_numbers: true,
            format_values: [false, false, false, true, false, false],
            options: OPTION_DEFAULTS,
            select_states: std::array::from_fn(|_| SelectState::default()),
            last: "none yet · Activated(i) or Toggled(i, on)".to_owned(),
        }
    }

    fn toolbar(&self, id: Id) -> Toolbar {
        if id == QUERY {
            let keys = nav_keys();
            build_toolbar(
                QUERY,
                query_items(self.running, self.query_wrap, self.query_line_numbers),
                &keys,
                self.options,
            )
        } else if id == NARROW {
            let keys = nav_keys();
            build_toolbar(
                NARROW,
                query_items(self.running, self.narrow_wrap, self.narrow_line_numbers),
                &keys,
                self.options,
            )
        } else {
            let keys = format_keys();
            build_toolbar(
                FORMAT,
                format_items(self.format_values),
                &keys,
                self.options,
            )
        }
    }

    fn state_mut(&mut self, id: Id) -> &mut ToolbarState {
        if id == QUERY {
            &mut self.query_state
        } else if id == NARROW {
            &mut self.narrow_state
        } else {
            &mut self.format_state
        }
    }

    fn toolbar_state(&self, id: Id) -> &ToolbarState {
        if id == QUERY {
            &self.query_state
        } else if id == NARROW {
            &self.narrow_state
        } else {
            &self.format_state
        }
    }

    fn commit(&mut self, id: Id, event: ToolbarActionEvent, label: &str) {
        self.last = format!("{:?} · {label}", event.action);
        match event.action {
            ToolbarAction::Activated(_) if id != FORMAT => {
                let keys = nav_keys();
                if event.key == keys[0] || event.key == keys[1] {
                    self.running = true;
                } else if event.key == keys[2] {
                    self.running = false;
                }
            }
            ToolbarAction::Toggled(_, on) if id == QUERY => {
                let keys = nav_keys();
                if event.key == keys[5] {
                    self.query_wrap = on;
                } else if event.key == keys[6] {
                    self.query_line_numbers = on;
                }
            }
            ToolbarAction::Toggled(_, on) if id == NARROW => {
                let keys = nav_keys();
                if event.key == keys[5] {
                    self.narrow_wrap = on;
                } else if event.key == keys[6] {
                    self.narrow_line_numbers = on;
                }
            }
            ToolbarAction::Toggled(_, on) if id == FORMAT => {
                if let Some(index) = format_keys().iter().position(|key| *key == event.key) {
                    self.format_values[index] = on;
                }
            }
            _ => {}
        }
    }

    fn update_toolbar(&mut self, cx: &mut Cx<'_>, id: Id) -> (Response<()>, Option<PageStatus>) {
        let toolbar = self.toolbar(id);
        let update = toolbar.update(cx, self.state_mut(id));
        let events = update
            .action_ref()
            .map(|actions| actions.as_slice().to_vec())
            .unwrap_or_default();
        let source = toolbar.items().to_vec();
        let mut status = None;
        for event in &events {
            let label = source
                .get(match event.action {
                    ToolbarAction::Activated(index) | ToolbarAction::Toggled(index, _) => index,
                })
                .map(|item| item.label.as_str())
                .unwrap_or("Toolbar item")
                .to_owned();
            self.commit(id, *event, &label);
            status = Some(PageStatus(self.last.clone()));
        }
        (update.erase(), status)
    }

    fn update_option(&mut self, cx: &mut Cx<'_>, index: usize) -> Response<()> {
        let id = OPTION_IDS[index];
        let items = option_data(index);
        let response = Select::new(id)
            .popup_rows(items.len() as u16)
            .update_in_form(
                cx,
                &mut self.select_states[index],
                &mut self.options[index],
                items,
                false,
            );
        response.erase()
    }

    fn section(&self) -> usize {
        selected_section(&self.page_tabs)
    }

    fn state_text(&self) -> &'static str {
        if self.running {
            "running · Stop is enabled, Run waits"
        } else {
            "idle · Stop is disabled until a run starts"
        }
    }

    fn narrow_meta(&self) -> &'static str {
        match self.options[4] {
            1 => NARROW_META_ELLIPSIS,
            2 => NARROW_META_OFF,
            _ => NARROW_META,
        }
    }

    fn narrow_readout(&self, toolbar: &Toolbar) -> String {
        let keys = self.narrow_state.hidden_keys();
        if keys.is_empty() {
            return if self.options[4] == 2 {
                "overflow off: no More button".to_owned()
            } else {
                "everything fits: no More button".to_owned()
            };
        }
        let hidden = keys
            .iter()
            .filter_map(|key| {
                let index = toolbar
                    .item_keys()
                    .iter()
                    .position(|candidate| candidate == key)?;
                let item = toolbar.items().get(index)?;
                Some(match (item.disabled, item.reason.as_deref()) {
                    (true, Some(reason)) => format!("{} ({reason})", item.label),
                    _ => item.label.clone(),
                })
            })
            .collect::<Vec<_>>();
        if hidden.is_empty() {
            "hidden actions are published by the native Toolbar".to_owned()
        } else {
            format!("behind More: {}", hidden.join(" · "))
        }
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        let rows = layout::rows(
            area,
            &[
                Track::Fixed(7),
                Track::Fixed(1),
                Track::Fixed(9),
                Track::Fixed(1),
                Track::Fixed(5),
                Track::Fixed(1),
                Track::Fixed(8),
            ],
        );
        let query_area = rows.first().copied().unwrap_or_default();
        let narrow_area = rows.get(2).copied().unwrap_or_default();
        let format_area = rows.get(4).copied().unwrap_or_default();
        let last_area = rows.get(6).copied().unwrap_or_default();

        Panel::new(QUERY_PANEL)
            .kind(PanelKind::Card)
            .title("Query toolbar")
            .meta(QUERY_META)
            .draw(ui, query_area, |ui, inner| {
                self.toolbar(QUERY)
                    .draw(ui, Rect { height: 1, ..inner }, &self.query_state);
                let state_area = Rect {
                    y: inner.y.saturating_add(2),
                    height: 1,
                    ..inner
                };
                let _ = ui.paint_str(state_area, self.state_text(), ui.surface_style());
            });

        let narrow_toolbar = self.toolbar(NARROW);
        Panel::new(NARROW_PANEL)
            .kind(PanelKind::Card)
            .title("Narrow toolbar")
            .meta(self.narrow_meta())
            .draw(ui, narrow_area, |ui, inner| {
                let toolbar_area = Rect {
                    width: NARROW_WIDTH.min(inner.width),
                    height: 1,
                    ..inner
                };
                narrow_toolbar.draw(ui, toolbar_area, &self.narrow_state);
                let readout = self.narrow_readout(&narrow_toolbar);
                let lines = wrap(&readout, inner.width.max(1));
                for (index, line) in lines.into_iter().enumerate() {
                    let y = inner.y.saturating_add(2).saturating_add(index as u16);
                    if y >= inner.bottom() {
                        break;
                    }
                    let _ = ui.paint_str(
                        Rect {
                            y,
                            height: 1,
                            ..inner
                        },
                        &line,
                        ui.surface_style(),
                    );
                }
            });

        Panel::new(FORMAT_PANEL)
            .kind(PanelKind::Card)
            .title("Formatting toolbar")
            .meta(FORMAT_META)
            .draw(ui, format_area, |ui, inner| {
                self.toolbar(FORMAT)
                    .draw(ui, Rect { height: 1, ..inner }, &self.format_state);
            });

        Panel::new(LAST_PANEL)
            .kind(PanelKind::Card)
            .title("Last")
            .draw(ui, last_area, |ui, inner| {
                let facts = [
                    format!("Action: {}", self.last),
                    String::from("Keys: Tab enters once · ←→ move · Home End · Enter Space run"),
                    format!("State: {}", self.state_text()),
                ];
                for (index, fact) in facts.iter().enumerate() {
                    let y = inner.y.saturating_add(index as u16);
                    if y >= inner.bottom() {
                        break;
                    }
                    let _ = ui.paint_str(
                        Rect {
                            y,
                            height: 1,
                            ..inner
                        },
                        fact,
                        ui.surface_style(),
                    );
                }
            });
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        for index in 0..OPTION_IDS.len() {
            let y = area.y.saturating_add((index as u16).saturating_mul(4));
            if y.saturating_add(4) > area.bottom() {
                break;
            }
            let _ = ui.paint_str(
                Rect::new(area.x, y, area.width, 1),
                OPTION_LABELS[index],
                ui.surface_style(),
            );
            let select_area = Rect::new(area.x, y.saturating_add(1), area.width, 3);
            let _ = Select::new(OPTION_IDS[index]).draw(
                ui,
                select_area,
                &self.select_states[index],
                option_data(index),
            );
        }
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        let label_width = 14u16;
        let content_width = 29u16;
        let narrow_width = 16u16;
        let col_x = [
            area.x.saturating_add(label_width),
            area.x.saturating_add(label_width).saturating_add(30),
        ];
        let header = ui.surface_style();
        for (x, title) in [(col_x[0], "Commands"), (col_x[1], "Toggles")] {
            let _ = ui.paint_str(Rect::new(x, area.y, 30, 1), title, header);
        }
        let command = build_toolbar(
            STATE_REFERENCE.index(0),
            vec![
                ToolItem::new("Run").shortcut("F5"),
                ToolItem::new("Stop").separator(),
                ToolItem::new("Wrap").toggle(false),
            ],
            &[
                ItemKey::text("states.commands.run"),
                ItemKey::text("states.commands.stop"),
                ItemKey::text("states.commands.wrap"),
            ],
            OPTION_DEFAULTS,
        );
        let toggles = build_toolbar(
            STATE_REFERENCE.index(1),
            vec![
                ToolItem::new("Bold").toggle(false),
                ToolItem::new("Italic").toggle(false),
                ToolItem::new("Code").toggle(false),
            ],
            &[
                ItemKey::text("states.toggles.bold"),
                ItemKey::text("states.toggles.italic"),
                ItemKey::text("states.toggles.code"),
            ],
            OPTION_DEFAULTS,
        );
        for (index, (name, state)) in STATE_NAMES.iter().zip(STATE_VALUES).enumerate() {
            let y = area.y.saturating_add(1).saturating_add(index as u16);
            if y >= area.bottom() {
                break;
            }
            let _ = ui.paint_str(
                Rect::new(area.x, y, label_width, 1),
                name,
                ui.surface_style(),
            );
            let narrow = state == ToolbarReferenceState::Narrow;
            let width = if narrow { narrow_width } else { content_width };
            command.draw_reference(ui, Rect::new(col_x[0], y, width, 1), state);
            toggles.draw_reference(ui, Rect::new(col_x[1], y, width, 1), state);
        }
    }
}

impl Default for ToolbarsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for ToolbarsPage {
    fn title(&self) -> &'static str {
        "Toolbars"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Tabs::new(PAGE_TABS)
            .update(cx, &mut self.page_tabs, SECTIONS)
            .erase();
        let section = self.section();
        let mut status = None;

        if section == 0 || self.query_state.menu_open() {
            let (update, action_status) = self.update_toolbar(cx, QUERY);
            response |= update;
            status = action_status.or(status);
        }
        if section == 0 || self.narrow_state.menu_open() {
            let (update, action_status) = self.update_toolbar(cx, NARROW);
            response |= update;
            status = action_status.or(status);
        }
        if section == 0 || self.format_state.menu_open() {
            let (update, action_status) = self.update_toolbar(cx, FORMAT);
            response |= update;
            status = action_status.or(status);
        }
        for (index, id) in OPTION_IDS.iter().enumerate() {
            if section == 1 || self.select_states[index].is_open() || cx.is_open(*id) {
                response |= self.update_option(cx, index);
            }
        }

        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), BLURB, |ui, body| {
            let tabs_area = Rect { height: 1, ..body };
            let content = Rect {
                y: body.y.saturating_add(1),
                height: body.height.saturating_sub(1),
                ..body
            };
            Tabs::new(PAGE_TABS).draw(ui, tabs_area, &self.page_tabs, SECTIONS);
            match self.section() {
                1 => self.draw_options(ui, content),
                2 => self.draw_states(ui, content),
                _ => self.draw_playground(ui, content),
            }
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(PAGE_TABS).contains(StateFlags::FOCUSED) {
            return TAB_HINTS;
        }
        for id in OPTION_IDS {
            if ui.state(id).contains(StateFlags::FOCUSED) {
                return SELECT_HINTS;
            }
        }
        if [QUERY, NARROW, FORMAT]
            .into_iter()
            .any(|id| self.toolbar_state(id).menu_open())
        {
            return MENU_HINTS;
        }
        for id in [QUERY, NARROW, FORMAT] {
            if !ui.state(id).contains(StateFlags::FOCUSED) {
                continue;
            }
            let state = self.toolbar_state(id);
            if state.menu_open() {
                return MENU_HINTS;
            }
            if state.on_more() {
                return MORE_HINTS;
            }
            return if id == FORMAT {
                FORMAT_HINTS
            } else {
                TOOLBAR_HINTS
            };
        }
        &[]
    }
}
