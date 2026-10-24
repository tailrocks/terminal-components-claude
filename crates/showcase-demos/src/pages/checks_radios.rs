use termrock::{
    Checkbox, Cx, FrameRead, Id, ItemKey, RadioGroup, RadioGroupAction, RadioGroupState, Rect,
    Response, StateFlags, Tabs, TabsState, Toggle, Ui, controls::CheckboxState,
    controls::RadioKeyPolicy, width,
};

use super::{Page, PageUpdate, frame};

const PAGE_TABS: Id = Id::root("showcase_ui::pages::checks_radios.page_tabs");
const GROUP_TABS: Id = Id::root("showcase_ui::pages::checks_radios.group_tabs");
const CHECK_ALL: Id = Id::root("showcase_ui::pages::checks_radios.check_all");
const CHECK_NOTIFICATIONS: Id = Id::root("showcase_ui::pages::checks_radios.check_notifications");
const CHECK_ARCHIVED: Id = Id::root("showcase_ui::pages::checks_radios.check_archived");
const CHECK_MANAGED: Id = Id::root("showcase_ui::pages::checks_radios.check_managed");
const RADIO_ISOLATION: Id = Id::root("showcase_ui::pages::checks_radios.radio_isolation");
const RADIO_REGION: Id = Id::root("showcase_ui::pages::checks_radios.radio_region");
const TOGGLE_AUTOSAVE: Id = Id::root("showcase_ui::pages::checks_radios.toggle_autosave");
const TOGGLE_USAGE: Id = Id::root("showcase_ui::pages::checks_radios.toggle_usage");
const TOGGLE_BETA: Id = Id::root("showcase_ui::pages::checks_radios.toggle_beta");
const OPTION_HIT: Id = Id::root("showcase_ui::pages::checks_radios.option_hit");
const OPTION_MIXED: Id = Id::root("showcase_ui::pages::checks_radios.option_mixed");
const OPTION_RADIO_NAV: Id = Id::root("showcase_ui::pages::checks_radios.option_radio_nav");
const OPTION_DISABLED: Id = Id::root("showcase_ui::pages::checks_radios.option_disabled");
const GALLERY_CHECK: Id = Id::root("showcase_ui::pages::checks_radios.gallery.check");
const GALLERY_RADIO: Id = Id::root("showcase_ui::pages::checks_radios.gallery.radio");
const GALLERY_TOGGLE: Id = Id::root("showcase_ui::pages::checks_radios.gallery.toggle");

const PAGE_TABS_ITEMS: &[&str] = &["Playground", "Options", "States"];
const GROUP_TABS_ITEMS: &[&str] = &["Checkboxes", "Radios", "Toggles"];
const HIT_OPTIONS: &[&str] = &["Label only", "Full row"];
const RADIO_NAV_OPTIONS: &[&str] = &["Default", "Vim", "No Home/End"];
const ISOLATION_OPTIONS: &[&str] = &["Read committed", "Repeatable read", "Serializable"];
const REGION_OPTIONS: &[&str] = &["eu-west-1", "us-east-1"];
const GALLERY_COLUMNS: &[&str] = &["off", "on", "mix", "radio", "off", "on"];
const STATE_ROWS: &[(&str, ReferenceKind)] = &[
    ("Rest", ReferenceKind::Rest),
    ("Hover", ReferenceKind::Hover),
    ("Pressed", ReferenceKind::Pressed),
    ("Focused", ReferenceKind::Focused),
    ("Focused + hover", ReferenceKind::FocusedHover),
    ("Disabled", ReferenceKind::Disabled),
    ("Overflow", ReferenceKind::Overflow),
];
const CHECK_LABELS: [&str; 4] = [
    "Select all rows",
    "Send notifications",
    "Include archived projects",
    "Managed by policy",
];
const TOGGLE_LABELS: [&str; 3] = ["Auto-save drafts", "Share usage data", "Beta features"];
const CHECK_IDS: [Id; 4] = [
    CHECK_ALL,
    CHECK_NOTIFICATIONS,
    CHECK_ARCHIVED,
    CHECK_MANAGED,
];
const TOGGLE_IDS: [Id; 3] = [TOGGLE_AUTOSAVE, TOGGLE_USAGE, TOGGLE_BETA];
const TABS_HINTS: &[(&str, &str)] = &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
const RADIO_HINTS: &[(&str, &str)] = &[("↑ ↓", "Choose"), ("Space", "Choose")];
const TOGGLE_HINTS: &[(&str, &str)] = &[("Space", "Toggle")];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReferenceKind {
    Rest,
    Hover,
    Pressed,
    Focused,
    FocusedHover,
    Disabled,
    Overflow,
}

impl ReferenceKind {
    const fn flags(self) -> Option<termrock::ReferenceState> {
        use termrock::ReferenceState as Ref;
        match self {
            Self::Rest | Self::Disabled | Self::Overflow => None,
            Self::Hover => Some(Ref::HOVERED),
            Self::Pressed => Some(Ref::PRESSED),
            Self::Focused => Some(Ref::FOCUSED.union(Ref::FOCUS_VISIBLE)),
            Self::FocusedHover => Some(Ref::FOCUSED.union(Ref::FOCUS_VISIBLE).union(Ref::HOVERED)),
        }
    }

    const fn disabled(self) -> bool {
        matches!(self, Self::Disabled)
    }
}

#[derive(Debug)]
pub struct ChecksRadiosPage {
    page_tabs: TabsState,
    group_tabs: TabsState,
    hit: usize,
    mixed: bool,
    radio_nav: usize,
    globally_disabled: bool,
    hit_state: RadioGroupState,
    radio_nav_state: RadioGroupState,
    checks: [CheckboxState; 4],
    isolation: usize,
    region: usize,
    isolation_state: RadioGroupState,
    region_state: RadioGroupState,
    toggles: [bool; 3],
    last: Option<(&'static str, bool)>,
}

impl ChecksRadiosPage {
    pub fn new() -> Self {
        let mut isolation_state = RadioGroupState::default();
        isolation_state.set_cursor(0, ItemKey::index(0));
        let mut region_state = RadioGroupState::default();
        region_state.set_cursor(0, ItemKey::index(0));
        Self {
            page_tabs: TabsState::default(),
            group_tabs: TabsState::default(),
            hit: 0,
            mixed: true,
            radio_nav: 0,
            globally_disabled: false,
            hit_state: RadioGroupState::default(),
            radio_nav_state: RadioGroupState::default(),
            checks: [
                CheckboxState::Mixed,
                CheckboxState::Checked,
                CheckboxState::Unchecked,
                CheckboxState::Checked,
            ],
            isolation: 0,
            region: 0,
            isolation_state,
            region_state,
            toggles: [true, false, false],
            last: None,
        }
    }

    fn page_index(&self) -> usize {
        active_index(&self.page_tabs)
    }

    fn group_index(&self) -> usize {
        active_index(&self.group_tabs)
    }

    fn key_policy(&self) -> RadioKeyPolicy {
        match self.radio_nav {
            1 => RadioKeyPolicy::Vim,
            2 => RadioKeyPolicy::NoHomeEnd,
            _ => RadioKeyPolicy::Desktop,
        }
    }

    fn update_options(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let before = (self.hit, self.mixed, self.radio_nav, self.globally_disabled);
        let mut response = Response::ignored();

        let hit = RadioGroup::new(OPTION_HIT)
            .value(ItemKey::index(self.hit))
            .update(cx, &mut self.hit_state, HIT_OPTIONS);
        if let Some(RadioGroupAction::Chose(ItemKey::Index(index))) = hit.action_ref() {
            self.hit = *index;
        }
        response |= hit.erase();

        let mixed = Toggle::new(OPTION_MIXED, "Mixed checkbox")
            .on(self.mixed)
            .update(cx, &mut self.mixed);
        response |= mixed.erase();

        let radio_nav = RadioGroup::new(OPTION_RADIO_NAV)
            .value(ItemKey::index(self.radio_nav))
            .update(cx, &mut self.radio_nav_state, RADIO_NAV_OPTIONS);
        if let Some(RadioGroupAction::Chose(ItemKey::Index(index))) = radio_nav.action_ref() {
            self.radio_nav = *index;
        }
        response |= radio_nav.erase();

        let disabled = Toggle::new(OPTION_DISABLED, "Disable playground")
            .on(self.globally_disabled)
            .update(cx, &mut self.globally_disabled);
        response |= disabled.erase();

        let after = (self.hit, self.mixed, self.radio_nav, self.globally_disabled);
        if before != after {
            // The source page rebuilds this first field from the Mixed option on every option
            // change; the other values and radio cursor selections remain user state.
            self.checks[0] = if self.mixed {
                CheckboxState::Mixed
            } else {
                CheckboxState::Unchecked
            };
        }
        response
    }

    fn update_checkboxes(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        for index in 0..self.checks.len() {
            let disabled = self.globally_disabled || index == 3;
            let widget = Checkbox::new(CHECK_IDS[index], CHECK_LABELS[index])
                .state(self.checks[index])
                .disabled(disabled);
            if index == 0 {
                let result = widget.update_state(cx, &mut self.checks[index]);
                if result.activated() {
                    self.last = Some((
                        CHECK_LABELS[index],
                        self.checks[index] == CheckboxState::Checked,
                    ));
                }
                response |= result.erase();
            } else {
                let mut value = self.checks[index] == CheckboxState::Checked;
                let result = widget.update(cx, &mut value);
                if result.activated() {
                    self.checks[index] = if value {
                        CheckboxState::Checked
                    } else {
                        CheckboxState::Unchecked
                    };
                    self.last = Some((CHECK_LABELS[index], value));
                }
                response |= result.erase();
            }
        }
        response
    }

    fn update_radios(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        let isolation = RadioGroup::new(RADIO_ISOLATION)
            .value(ItemKey::index(self.isolation))
            .key_policy(self.key_policy())
            .disabled(self.globally_disabled)
            .update(cx, &mut self.isolation_state, ISOLATION_OPTIONS);
        if let Some(RadioGroupAction::Chose(ItemKey::Index(index))) = isolation.action_ref() {
            self.isolation = *index;
        }
        response |= isolation.erase();

        let region = RadioGroup::new(RADIO_REGION)
            .value(ItemKey::index(self.region))
            .key_policy(self.key_policy())
            .disabled(true)
            .update(cx, &mut self.region_state, REGION_OPTIONS);
        if let Some(RadioGroupAction::Chose(ItemKey::Index(index))) = region.action_ref() {
            self.region = *index;
        }
        response |= region.erase();
        response
    }

    fn update_toggles(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        for index in 0..self.toggles.len() {
            let widget = Toggle::new(TOGGLE_IDS[index], TOGGLE_LABELS[index])
                .on(self.toggles[index])
                .disabled(self.globally_disabled || index == 2);
            response |= widget.update(cx, &mut self.toggles[index]).erase();
        }
        response
    }

    fn draw_checkboxes(&self, ui: &mut Ui<'_>, area: Rect) {
        for index in 0..self.checks.len() {
            let y = area.y.saturating_add(index as u16);
            if y >= area.bottom() {
                break;
            }
            let row = Rect {
                y,
                height: 1,
                ..area
            };
            let disabled = self.globally_disabled || index == 3;
            let hit_width = checkbox_hit_width(CHECK_LABELS[index], row.width, self.hit == 1);
            Checkbox::new(CHECK_IDS[index], CHECK_LABELS[index])
                .state(self.checks[index])
                .disabled(disabled)
                .draw(
                    ui,
                    Rect {
                        width: hit_width,
                        ..row
                    },
                );
        }
    }

    fn draw_radios(&self, ui: &mut Ui<'_>, area: Rect) {
        paint_text(ui, Rect { height: 1, ..area }, "Isolation level");
        let first = Rect {
            y: area.y.saturating_add(1),
            height: area.height.saturating_sub(1).min(3),
            ..area
        };
        RadioGroup::new(RADIO_ISOLATION)
            .value(ItemKey::index(self.isolation))
            .key_policy(self.key_policy())
            .disabled(self.globally_disabled)
            .draw(ui, first, &self.isolation_state, ISOLATION_OPTIONS);

        let region_y = area.y.saturating_add(5);
        if region_y >= area.bottom() {
            return;
        }
        paint_text(
            ui,
            Rect::new(area.x, region_y, area.width, 1),
            "Region · fixed by policy",
        );
        RadioGroup::new(RADIO_REGION)
            .value(ItemKey::index(self.region))
            .key_policy(self.key_policy())
            .disabled(true)
            .draw(
                ui,
                Rect::new(
                    area.x,
                    region_y.saturating_add(1),
                    area.width,
                    area.bottom().saturating_sub(region_y.saturating_add(1)),
                ),
                &self.region_state,
                REGION_OPTIONS,
            );
    }

    fn draw_toggles(&self, ui: &mut Ui<'_>, area: Rect) {
        for index in 0..self.toggles.len() {
            let y = area.y.saturating_add(index as u16);
            if y >= area.bottom() {
                break;
            }
            Toggle::new(TOGGLE_IDS[index], TOGGLE_LABELS[index])
                .on(self.toggles[index])
                .disabled(self.globally_disabled || index == 2)
                .draw(
                    ui,
                    Rect {
                        y,
                        height: 1,
                        ..area
                    },
                );
        }
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let left_width = area.width.saturating_add(1) / 2;
        let right_width = area.width.saturating_sub(left_width).saturating_sub(1);
        let left = Rect {
            width: left_width,
            ..area
        };
        let right = Rect {
            x: area.x.saturating_add(left_width).saturating_add(1),
            width: right_width,
            ..area
        };
        paint_text(ui, Rect { height: 1, ..left }, "Checkbox hit area");
        RadioGroup::new(OPTION_HIT)
            .value(ItemKey::index(self.hit))
            .draw(
                ui,
                Rect {
                    y: left.y.saturating_add(1),
                    height: 2.min(left.height.saturating_sub(1)),
                    ..left
                },
                &self.hit_state,
                HIT_OPTIONS,
            );
        Toggle::new(OPTION_MIXED, "Mixed checkbox")
            .on(self.mixed)
            .draw(
                ui,
                Rect::new(left.x, left.y.saturating_add(4), left.width, 1),
            );

        paint_text(ui, Rect { height: 1, ..right }, "Radio keyboard");
        RadioGroup::new(OPTION_RADIO_NAV)
            .value(ItemKey::index(self.radio_nav))
            .draw(
                ui,
                Rect {
                    y: right.y.saturating_add(1),
                    height: 3.min(right.height.saturating_sub(1)),
                    ..right
                },
                &self.radio_nav_state,
                RADIO_NAV_OPTIONS,
            );
        Toggle::new(OPTION_DISABLED, "Disable playground")
            .on(self.globally_disabled)
            .draw(
                ui,
                Rect::new(right.x, right.y.saturating_add(5), right.width, 1),
            );
    }

    fn draw_gallery(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let labels_width = width("Focused + hover").min(area.width);
        let grid_x = area.x.saturating_add(labels_width);
        let grid_width = area.width.saturating_sub(labels_width);
        let col_width = grid_width / 6;
        let remainder = grid_width % 6;
        for (column, label) in GALLERY_COLUMNS.iter().enumerate() {
            let x_offset = (0..column).fold(0u16, |sum, current| {
                sum.saturating_add(col_width)
                    .saturating_add(u16::from((current as u16) < remainder))
            });
            let width = col_width.saturating_add(u16::from((column as u16) < remainder));
            paint_text(
                ui,
                Rect::new(grid_x.saturating_add(x_offset), area.y, width, 1),
                label,
            );
        }

        for (row_index, (label, kind)) in STATE_ROWS.iter().enumerate() {
            let y = area.y.saturating_add(1).saturating_add(row_index as u16);
            if y >= area.bottom() {
                break;
            }
            paint_text(ui, Rect::new(area.x, y, labels_width, 1), label);
            for column in 0..6 {
                let x_offset = (0..column).fold(0u16, |sum, current| {
                    sum.saturating_add(col_width)
                        .saturating_add(u16::from((current as u16) < remainder))
                });
                let width = col_width.saturating_add(u16::from((column as u16) < remainder));
                if width == 0 {
                    continue;
                }
                let id = match column {
                    0..=2 => GALLERY_CHECK.index(column),
                    3 => GALLERY_RADIO,
                    4..=5 => GALLERY_TOGGLE.index(column - 4),
                    _ => unreachable!(),
                };
                let target = kind
                    .flags()
                    .map(|flags| termrock::ReferenceTarget::new(id, flags));
                let specimen_width = if *kind == ReferenceKind::Overflow {
                    width.min(3)
                } else {
                    width
                };
                let rect = Rect::new(grid_x.saturating_add(x_offset), y, specimen_width, 1);
                ui.reference(target, |ui| match column {
                    0..=2 => {
                        let state = match column {
                            0 => CheckboxState::Unchecked,
                            1 => CheckboxState::Checked,
                            _ => CheckboxState::Mixed,
                        };
                        Checkbox::new(id, "X")
                            .state(state)
                            .disabled(kind.disabled())
                            .draw(ui, rect);
                    }
                    3 => {
                        let mut state = RadioGroupState::default();
                        state.set_cursor(0, ItemKey::index(0));
                        RadioGroup::new(id)
                            .value(ItemKey::index(0))
                            .disabled(kind.disabled())
                            .draw(ui, rect, &state, &["X"]);
                    }
                    4..=5 => {
                        Toggle::new(id, "X")
                            .on(column == 5)
                            .disabled(kind.disabled())
                            .draw(ui, rect);
                    }
                    _ => unreachable!(),
                });
            }
        }
    }
}

impl Default for ChecksRadiosPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for ChecksRadiosPage {
    fn title(&self) -> &'static str {
        "Checks & radios"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        response |= Tabs::new(PAGE_TABS)
            .update(cx, &mut self.page_tabs, PAGE_TABS_ITEMS)
            .erase();

        match self.page_index() {
            0 => {
                response |= Tabs::new(GROUP_TABS)
                    .update(cx, &mut self.group_tabs, GROUP_TABS_ITEMS)
                    .erase();
                match self.group_index() {
                    0 => response |= self.update_checkboxes(cx),
                    1 => response |= self.update_radios(cx),
                    _ => response |= self.update_toggles(cx),
                }
            }
            1 => response |= self.update_options(cx),
            _ => {}
        }
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.with_area(area, |ui| {
            frame(
                ui,
                area,
                self.title(),
                "real controls · persistent options",
                |ui, body| {
                    Tabs::new(PAGE_TABS).draw(
                        ui,
                        Rect { height: 2, ..body },
                        &self.page_tabs,
                        PAGE_TABS_ITEMS,
                    );
                    let content = Rect {
                        y: body.y.saturating_add(3),
                        height: body.height.saturating_sub(3),
                        ..body
                    };
                    match self.page_index() {
                        0 => {
                            Tabs::new(GROUP_TABS).draw(
                                ui,
                                Rect {
                                    height: 2,
                                    ..content
                                },
                                &self.group_tabs,
                                GROUP_TABS_ITEMS,
                            );
                            let status_y = content.y.saturating_add(2);
                            let status = match self.last {
                                None => "Last: nothing yet".to_owned(),
                                Some((label, checked)) => format!(
                                    "Last: {label} {}",
                                    if checked { "checked" } else { "unchecked" }
                                ),
                            };
                            paint_text(
                                ui,
                                Rect::new(content.x, status_y, content.width, 1),
                                &status,
                            );
                            let fields = Rect {
                                y: content.y.saturating_add(4),
                                height: content.height.saturating_sub(4),
                                ..content
                            };
                            match self.group_index() {
                                0 => self.draw_checkboxes(ui, fields),
                                1 => self.draw_radios(ui, fields),
                                _ => self.draw_toggles(ui, fields),
                            }
                        }
                        1 => self.draw_options(ui, content),
                        _ => self.draw_gallery(ui, content),
                    }
                },
            );
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if [PAGE_TABS, GROUP_TABS]
            .iter()
            .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            TABS_HINTS
        } else if [RADIO_ISOLATION, RADIO_REGION, OPTION_HIT, OPTION_RADIO_NAV]
            .iter()
            .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            RADIO_HINTS
        } else if [
            CHECK_ALL,
            CHECK_NOTIFICATIONS,
            CHECK_ARCHIVED,
            CHECK_MANAGED,
            TOGGLE_AUTOSAVE,
            TOGGLE_USAGE,
            TOGGLE_BETA,
            OPTION_MIXED,
            OPTION_DISABLED,
        ]
        .iter()
        .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            TOGGLE_HINTS
        } else {
            &[]
        }
    }

    fn seek_paused(&mut self, _frame: usize) {}
}

fn active_index(tabs: &TabsState) -> usize {
    match tabs.active().or(tabs.cursor()) {
        Some(ItemKey::Index(index)) => index,
        _ => 0,
    }
}

fn paint_text(ui: &mut Ui<'_>, area: Rect, text: &str) {
    if !area.is_empty() {
        ui.paint_str(area, text, ui.surface_style());
    }
}

fn checkbox_hit_width(label: &str, available: u16, full_row: bool) -> u16 {
    if full_row {
        available
    } else {
        width(label).saturating_add(5).min(available)
    }
}

#[cfg(test)]
mod tests {
    use super::checkbox_hit_width;

    #[test]
    fn label_hit_area_uses_terminal_columns_for_wide_and_combining_text() {
        let label = "e\u{301}界";
        assert_eq!(termrock::width(label), 3);
        assert_eq!(checkbox_hit_width(label, 20, false), 8);
        assert_eq!(checkbox_hit_width(label, 7, false), 7);
        assert_eq!(checkbox_hit_width(label, 20, true), 20);
    }
}
