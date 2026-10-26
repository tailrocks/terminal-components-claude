use std::cell::Cell;
use std::time::Duration;

use termrock::controls::ButtonFocusVisibility;
use termrock::{
    BusyMarkerPolicy, Button, Constraints, Cx, Focusability, FrameRead, Id, Intent, ItemKey,
    KeyCode, Moment, Panel, PanelKind, Part, PartRef, Rect, ReferenceState, ReferenceTarget,
    Response, RowAlign, ScopeId, ScrollRegion, ScrollState, Select, SelectState, StateFlags,
    Status, Tabs, TabsState, Ui, Variant, author::VerticalWindow, layout,
};

use super::{Page, PageStatus, PageUpdate, frame};

const BUTTONS: Id = termrock::Id::root("showcase_ui::pages::buttons::buttons");
const PLAYGROUND_PANEL: Id = termrock::Id::root("showcase_ui::pages::buttons::buttons.playground");
const MATRIX: Id = termrock::Id::root("showcase_ui::pages::buttons::buttons.matrix");
const MATRIX_PANEL: Id = termrock::Id::root("showcase_ui::pages::buttons::buttons.matrix.panel");
const PAGE_TABS: Id = termrock::Id::root("showcase_ui::pages::buttons.page_tabs");
const OPTIONS_SCROLL: Id = termrock::Id::root("showcase_ui::pages::buttons.options_scroll");
const STATES_SCROLL: Id = termrock::Id::root("showcase_ui::pages::buttons.states_scroll");
const FOCUS_VISIBILITY: Id = termrock::Id::root("showcase_ui::pages::buttons.focus_visibility");
const ACTION_FOCUS: Id = termrock::Id::root("showcase_ui::pages::buttons.action_focus");
const ROW_ALIGNMENT: Id = termrock::Id::root("showcase_ui::pages::buttons.row_alignment");
const ROW_GAP: Id = termrock::Id::root("showcase_ui::pages::buttons.row_gap");
const STATE_BUTTONS: Id = termrock::Id::root("showcase_ui::pages::buttons.states.button");

const PAGE_SECTIONS: &[&str] = &["Playground", "Options", "States"];
const FOCUS_VISIBILITY_ITEMS: &[&str] = &["Always", "Keyboard only"];
const ACTION_FOCUS_ITEMS: &[&str] = &["In Tab order", "Pointer only"];
const ROW_ALIGNMENT_ITEMS: &[&str] = &["Start", "End"];
const ROW_GAP_ITEMS: &[&str] = &["Theme", "1", "4"];
const OPTION_IDS: [Id; 4] = [FOCUS_VISIBILITY, ACTION_FOCUS, ROW_ALIGNMENT, ROW_GAP];
const OPTION_LABELS: [&str; 4] = [
    "Focus ring",
    "Action buttons in Tab order",
    "Row alignment",
    "Row gap",
];
const OPTION_ROWS: usize = OPTION_IDS.len();
const STATE_ROWS: usize = 3;

const TABS_HINTS: &[(&str, &str)] = &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
const SELECT_HINTS: &[(&str, &str)] = &[("↑ ↓", "Choose"), ("Enter", "Choose")];
const SCROLL_HINTS: &[(&str, &str)] = &[("Wheel", "Scroll"), ("PgUp/Dn", "Scroll")];
const HIDDEN_OPTION_HINTS: &[(&str, &str)] = &[("Tab / Shift+Tab", "Focus"), ("Wheel", "Scroll")];
const BUTTON_HINTS: &[(&str, &str)] = &[("Enter / Space", "Activate")];

const CARD_TITLE: &[(Part, termrock::StylePatch)] = &[(
    Part::TITLE,
    termrock::StylePatch::new()
        .set_fg(termrock::Role::Fg(termrock::FgStep::Secondary))
        .remove(termrock::Modifier::BOLD),
)];

const DANGER_INK: &[(Part, termrock::StylePatch)] = &[(
    Part::CONTAINER,
    termrock::StylePatch::new().set_fg(termrock::Role::Danger),
)];

const DISABLED_PLATE: &[(Part, termrock::StylePatch)] = &[(
    Part::CONTAINER,
    termrock::StylePatch::new().set_bg(termrock::Role::Custom(termrock::Color::from_u32(
        0x0011_1111,
    ))),
)];

const PRESSED_BOLD: &[(Part, termrock::StylePatch)] = &[
    (
        Part::CONTAINER,
        termrock::StylePatch::new().add(termrock::Modifier::BOLD),
    ),
    (
        Part::LABEL,
        termrock::StylePatch::new().add(termrock::Modifier::BOLD),
    ),
];

fn playground_panel() -> Panel<'static> {
    Panel::new(PLAYGROUND_PANEL)
        .kind(PanelKind::Card)
        .title("Playground")
        .meta("hover · click · Tab · Enter / Space ")
        .patch_part(CARD_TITLE)
        .slot(Part::DETAIL, &paint_playground_meta)
}

fn matrix_panel() -> Panel<'static> {
    Panel::new(MATRIX_PANEL)
        .kind(PanelKind::Card)
        .title("State matrix")
        .meta("reference rendering ")
        .patch_part(CARD_TITLE)
        .slot(Part::DETAIL, &paint_matrix_meta)
}

fn paint_playground_meta(ui: &mut Ui<'_>, rect: Rect) {
    paint_faint(ui, rect, "hover · click · Tab · Enter / Space");
}

fn paint_matrix_meta(ui: &mut Ui<'_>, rect: Rect) {
    paint_faint(ui, rect, "reference rendering");
}

fn paint_faint(ui: &mut Ui<'_>, rect: Rect, text: &str) {
    let columns = termrock::width(text);
    let _ = ui.paint_str(
        Rect {
            width: columns.min(rect.width),
            ..rect
        },
        text,
        faint_ink(ui),
    );
}

fn paint_toggle_off(ui: &mut Ui<'_>, rect: Rect) {
    ui.glyph(rect, termrock::GlyphRole::DotOff, toggle_ink(ui, false));
}

fn paint_toggle_on(ui: &mut Ui<'_>, rect: Rect) {
    ui.glyph(rect, termrock::GlyphRole::DotOn, toggle_ink(ui, true));
}

fn toggle_ink(ui: &Ui<'_>, on: bool) -> termrock::author::PaintStyle {
    let level = ui.theme().capability.color;
    let fg = if on {
        ui.theme().color.accent
    } else {
        termrock::theme::baseline_muted(level)
    };
    ui.surface_style()
        .fg(fg)
        .bg(ui.theme().bg(termrock::Surface::Overlay))
}

fn faint_ink(ui: &Ui<'_>) -> termrock::author::PaintStyle {
    ui.surface_style()
        .fg(termrock::theme::baseline_faint(ui.theme().capability.color))
}

const SPECS: [(&str, Variant, bool, Option<bool>); 9] = [
    ("Run task", Variant::PRIMARY, false, None),
    ("Preview", Variant::SECONDARY, false, None),
    ("Cancel", Variant::SUBTLE, false, None),
    ("Delete branch", Variant::DANGER, false, None),
    ("Auto-approve", Variant::TOGGLE, false, Some(false)),
    ("Verbose", Variant::TOGGLE, false, Some(true)),
    ("Disabled primary", Variant::PRIMARY, true, None),
    ("Disabled", Variant::SECONDARY, true, None),
    ("Start long job", Variant::SECONDARY, false, None),
];

const GROUPS: [(&str, &[usize]); 4] = [
    ("Actions", &[0, 1, 2, 3]),
    ("Toggles", &[4, 5]),
    ("Disabled", &[6, 7]),
    ("Busy", &[8]),
];

const LONG_JOB: usize = 8;

const MATRIX_STATES: [(&str, StateFlags); 6] = [
    ("default", StateFlags::empty()),
    ("hover", StateFlags::HOVERED),
    (
        "focus",
        StateFlags::FOCUSED.union(StateFlags::FOCUS_VISIBLE),
    ),
    (
        "focus + hover",
        StateFlags::FOCUSED
            .union(StateFlags::FOCUS_VISIBLE)
            .union(StateFlags::HOVERED),
    ),
    (
        "pressed",
        StateFlags::PRESSED
            .union(StateFlags::FOCUSED)
            .union(StateFlags::FOCUS_VISIBLE),
    ),
    ("disabled", StateFlags::DISABLED),
];

const MATRIX_VARIANTS: [(Variant, &str); 4] = [
    (Variant::PRIMARY, "Primary"),
    (Variant::SECONDARY, "Secondary"),
    (Variant::SUBTLE, "Subtle"),
    (Variant::DANGER, "Danger"),
];

fn matrix_reference(flags: StateFlags) -> Option<ReferenceState> {
    let mut state = ReferenceState::default();
    let mut present = false;
    for (flag, reference) in [
        (StateFlags::FOCUSED, ReferenceState::FOCUSED),
        (StateFlags::FOCUS_VISIBLE, ReferenceState::FOCUS_VISIBLE),
        (StateFlags::HOVERED, ReferenceState::HOVERED),
        (StateFlags::PRESSED, ReferenceState::PRESSED),
    ] {
        if flags.contains(flag) {
            state |= reference;
            present = true;
        }
    }
    present.then_some(state)
}

fn section_tabs_area(area: Rect) -> Rect {
    Rect {
        y: area.y.saturating_add(1),
        height: area.height.saturating_sub(1).min(1),
        ..area
    }
}

fn option_row(id: Id) -> Option<usize> {
    OPTION_IDS.iter().position(|candidate| *candidate == id)
}

fn option_window(scroll: &ScrollState, view: Rect) -> Option<VerticalWindow> {
    let projected = ScrollRegion::view(scroll, view, OPTION_ROWS);
    let first_row = projected.offset().min(OPTION_ROWS);
    let visible = Rect {
        height: view
            .height
            .min(u16::try_from(OPTION_ROWS).unwrap_or(u16::MAX)),
        ..view
    };
    VerticalWindow::new(visible, OPTION_ROWS, first_row).ok()
}

fn option_row_areas(view: Rect, y: u16) -> (Rect, Rect) {
    let label_width = view.width.saturating_sub(14).min(28);
    let control_x = view.x.saturating_add(label_width).saturating_add(1);
    let control_width = view.right().saturating_sub(control_x);
    (
        Rect::new(view.x, y, label_width, 1),
        Rect::new(control_x, y, control_width, 1),
    )
}

fn option_field_is_visible(id: Id, scroll: &ScrollState, view: Rect) -> bool {
    let Some(row) = option_row(id) else {
        return false;
    };
    let Some(window) = option_window(scroll, view) else {
        return false;
    };
    if !window.visible_source_range().contains(&row) {
        return false;
    }
    let Some(y) = window.physical_row_for_source(row) else {
        return false;
    };
    let (_, control_area) = option_row_areas(view, y);
    !control_area.is_empty()
}

fn initial_select_state(index: usize) -> SelectState {
    let mut state = SelectState::default();
    state.set_value(Some(ItemKey::index(index)));
    state
}

fn wheel_delta(cx: &Cx<'_>, id: Id) -> i32 {
    cx.intents(id)
        .filter_map(|intent| match intent {
            Intent::Wheel { delta, .. } => Some(i32::from(delta)),
            _ => None,
        })
        .fold(0i32, i32::saturating_add)
}

fn page_scroll_keys(cx: &mut Cx<'_>, id: Id, scroll: &mut ScrollState) -> Response<()> {
    let mut response = Response::ignored();
    for intent in cx.intents(id) {
        let Intent::Key(key) = intent else {
            continue;
        };
        let before = scroll.offset();
        match key.code {
            KeyCode::PageUp => scroll.page_up(),
            KeyCode::PageDown => scroll.page_down(),
            _ => continue,
        }
        response |= if scroll.offset() == before {
            Response::consumed()
        } else {
            Response::changed()
        };
    }
    response
}

fn consume_hidden_option_intents(cx: &mut Cx<'_>, id: Id, locally_open: bool) -> Response<()> {
    if !cx.state(id).contains(StateFlags::FOCUSED)
        || cx.area(id).is_some()
        || locally_open
        || cx.is_open(id)
    {
        return Response::ignored();
    }
    if cx.intents(id).count() > 0 {
        Response::consumed()
    } else {
        Response::ignored()
    }
}

#[derive(Clone, Copy, Debug)]
struct ButtonsDrawFacts {
    section: usize,
    options_view: Option<Rect>,
}

#[derive(Debug)]
pub struct ButtonsPage {
    checked: [Option<bool>; 9],
    clicks: u32,
    last: Option<String>,
    busy_until: Option<Moment>,
    page_tabs: TabsState,
    focus_visibility_index: usize,
    focus_visibility_state: SelectState,
    action_focus_index: usize,
    action_focus_state: SelectState,
    row_alignment_index: usize,
    row_alignment_state: SelectState,
    row_gap_index: usize,
    row_gap_state: SelectState,
    options_scroll: ScrollState,
    options_focus: Option<Id>,
    states_scroll: ScrollState,
    drawn_layout: Cell<Option<ButtonsDrawFacts>>,
}

impl ButtonsPage {
    pub fn new() -> Self {
        let mut page = Self {
            checked: [None; 9],
            clicks: 0,
            last: None,
            busy_until: None,
            page_tabs: TabsState::default(),
            focus_visibility_index: 0,
            focus_visibility_state: initial_select_state(0),
            action_focus_index: 0,
            action_focus_state: initial_select_state(0),
            row_alignment_index: 0,
            row_alignment_state: initial_select_state(0),
            row_gap_index: 0,
            row_gap_state: initial_select_state(0),
            options_scroll: ScrollState::default(),
            options_focus: None,
            states_scroll: ScrollState::default(),
            drawn_layout: Cell::new(None),
        };
        for (slot, (_, _, _, checked)) in page.checked.iter_mut().zip(SPECS) {
            *slot = checked;
        }
        page
    }

    fn button_id(index: usize) -> Id {
        BUTTONS.index(index)
    }

    fn selected_section(&self) -> usize {
        match self.page_tabs.active().or(self.page_tabs.cursor()) {
            Some(ItemKey::Index(index)) => index.min(PAGE_SECTIONS.len().saturating_sub(1)),
            _ => 0,
        }
    }

    fn effective_section(&self, tabs_area: Rect) -> usize {
        if tabs_area.is_empty() {
            0
        } else {
            self.selected_section()
        }
    }

    fn focus_visibility(&self) -> ButtonFocusVisibility {
        match self.focus_visibility_index {
            1 => ButtonFocusVisibility::KeyboardOnly,
            _ => ButtonFocusVisibility::Always,
        }
    }

    fn action_focusability(&self) -> Focusability {
        match self.action_focus_index {
            1 => Focusability::ClickOnly,
            _ => Focusability::Focusable,
        }
    }

    fn row_alignment(&self) -> RowAlign {
        match self.row_alignment_index {
            1 => RowAlign::End,
            _ => RowAlign::Start,
        }
    }

    fn row_gap(&self, theme_gap: u16) -> u16 {
        match self.row_gap_index {
            1 => 1,
            2 => 4,
            _ => theme_gap,
        }
    }

    fn button(&self, index: usize) -> Option<Button<'static>> {
        let (label, variant, disabled, _) = SPECS.get(index).copied()?;
        let mut button = Button::new(Self::button_id(index), label)
            .variant(variant)
            .disabled(disabled)
            .focus_visible(self.focus_visibility());
        if index < GROUPS[0].1.len() {
            button = button.focusability(self.action_focusability());
        }
        button = button.busy_marker_policy(BusyMarkerPolicy::Replace);
        if let Some(checked) = self.checked.get(index).copied().flatten() {
            button = button.checked(checked);
        }
        if index == LONG_JOB && self.busy_until.is_some() {
            button = button.status(Status::Busy);
        }
        if variant == Variant::DANGER {
            button = button.patch_part(DANGER_INK);
        }
        if variant == Variant::TOGGLE {
            button = if self.checked.get(index).copied().flatten() == Some(true) {
                button.slot(Part::MARKER, &paint_toggle_on)
            } else {
                button.slot(Part::MARKER, &paint_toggle_off)
            };
        }
        Some(button)
    }

    fn activated(&mut self, index: usize, now: Moment) -> Option<PageStatus> {
        self.clicks = self.clicks.saturating_add(1);
        let (label, _, _, _) = SPECS.get(index).copied()?;
        if let Some(value) = self.checked.get(index).copied().flatten() {
            if let Some(slot) = self.checked.get_mut(index) {
                *slot = Some(!value);
            }
            self.last = Some(format!("{label} {}", if value { "off" } else { "on" }));
        } else {
            self.last = Some(format!("{label} ✓"));
        }
        if index == LONG_JOB {
            self.busy_until = Some(now.saturating_add(Duration::from_millis(2_200)));
            Some(PageStatus("Working…".to_owned()))
        } else {
            self.last.clone().map(PageStatus)
        }
    }
}

impl Default for ButtonsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for ButtonsPage {
    fn title(&self) -> &'static str {
        "Buttons"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let tabs_area = cx.area(PAGE_TABS).unwrap_or_default();
        let previous_section = self.effective_section(tabs_area);
        let mut response = Tabs::new(PAGE_TABS)
            .update(cx, &mut self.page_tabs, PAGE_SECTIONS)
            .erase();
        let section = self.effective_section(tabs_area);
        if section != previous_section {
            match section {
                1 => self.options_scroll.scroll_to(0),
                2 => self.states_scroll.scroll_to(0),
                _ => (),
            }
            self.options_focus = None;
        }

        let mut status = None;
        if cx.update_cause() == termrock::UpdateCause::Tick
            && self.busy_until.is_some_and(|deadline| cx.now() >= deadline)
        {
            self.busy_until = None;
            status = Some(PageStatus("Long job finished ✓".to_owned()));
            response = Response::changed();
        }

        if (section == 1 && cx.area(FOCUS_VISIBILITY).is_some())
            || self.focus_visibility_state.is_open()
            || cx.is_open(FOCUS_VISIBILITY)
        {
            response |= Select::new(FOCUS_VISIBILITY)
                .popup_rows(2)
                .update_in_form(
                    cx,
                    &mut self.focus_visibility_state,
                    &mut self.focus_visibility_index,
                    FOCUS_VISIBILITY_ITEMS,
                    false,
                )
                .erase();
        } else if section == 1 {
            response |= consume_hidden_option_intents(
                cx,
                FOCUS_VISIBILITY,
                self.focus_visibility_state.is_open(),
            );
        }
        if (section == 1 && cx.area(ACTION_FOCUS).is_some())
            || self.action_focus_state.is_open()
            || cx.is_open(ACTION_FOCUS)
        {
            response |= Select::new(ACTION_FOCUS)
                .popup_rows(2)
                .update_in_form(
                    cx,
                    &mut self.action_focus_state,
                    &mut self.action_focus_index,
                    ACTION_FOCUS_ITEMS,
                    false,
                )
                .erase();
        } else if section == 1 {
            response |=
                consume_hidden_option_intents(cx, ACTION_FOCUS, self.action_focus_state.is_open());
        }
        if (section == 1 && cx.area(ROW_ALIGNMENT).is_some())
            || self.row_alignment_state.is_open()
            || cx.is_open(ROW_ALIGNMENT)
        {
            response |= Select::new(ROW_ALIGNMENT)
                .popup_rows(2)
                .update_in_form(
                    cx,
                    &mut self.row_alignment_state,
                    &mut self.row_alignment_index,
                    ROW_ALIGNMENT_ITEMS,
                    false,
                )
                .erase();
        } else if section == 1 {
            response |= consume_hidden_option_intents(
                cx,
                ROW_ALIGNMENT,
                self.row_alignment_state.is_open(),
            );
        }
        if (section == 1 && cx.area(ROW_GAP).is_some())
            || self.row_gap_state.is_open()
            || cx.is_open(ROW_GAP)
        {
            response |= Select::new(ROW_GAP)
                .popup_rows(3)
                .update_in_form(
                    cx,
                    &mut self.row_gap_state,
                    &mut self.row_gap_index,
                    ROW_GAP_ITEMS,
                    false,
                )
                .erase();
        } else if section == 1 {
            response |= consume_hidden_option_intents(cx, ROW_GAP, self.row_gap_state.is_open());
        }

        if section == 1 {
            response |=
                ScrollRegion::new(OPTIONS_SCROLL).update(cx, &mut self.options_scroll, OPTION_ROWS);
            response |= page_scroll_keys(cx, OPTIONS_SCROLL, &mut self.options_scroll);

            let focused_option = cx
                .focused_in_scope(ScopeId::ROOT)
                .filter(|id| option_row(*id).is_some());
            let focus_changed = focused_option != self.options_focus;
            self.options_focus = focused_option;
            let wheel_delta = wheel_delta(cx, OPTIONS_SCROLL);
            if wheel_delta == 0
                && focus_changed
                && let Some(id) = focused_option
                && let Some(row) = option_row(id)
            {
                let _ = ScrollRegion::new(OPTIONS_SCROLL).prepare(
                    cx,
                    &mut self.options_scroll,
                    OPTION_ROWS,
                );
                let _ = self.options_scroll.reveal_span(row, 1);
            }
        } else {
            self.options_focus = None;
        }

        if section == 2 {
            response |=
                ScrollRegion::new(STATES_SCROLL).update(cx, &mut self.states_scroll, STATE_ROWS);
            response |= page_scroll_keys(cx, STATES_SCROLL, &mut self.states_scroll);
        }

        if section == 0 {
            for index in 0..SPECS.len() {
                if let Some(button) = self.button(index)
                    && cx.area(button.id()).is_some()
                    && button.update(cx).activated()
                {
                    status = self.activated(index, cx.now());
                    response |= Response::changed();
                }
            }
        }
        if cx.top_layer() == termrock::LayerId::PAGE
            && let Some(deadline) = self.busy_until
        {
            cx.request_repaint_at(deadline);
        }

        let _ = playground_panel();
        let _ = matrix_panel();
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        self.drawn_layout.set(None);
        let tabs_area = section_tabs_area(area);
        let tabs = Tabs::new(PAGE_TABS);
        let previous =
            ui.layout(PAGE_TABS)
                .map(|facts| termrock::navigation::tabs::TabWindowReceipt {
                    viewport_len: facts.viewport_len,
                    content_len: facts.content_len,
                });
        let tabs_layout =
            tabs.layout_default(tabs_area, area, &self.page_tabs, PAGE_SECTIONS, previous);
        let section = if tabs_layout.visible_area.is_empty() {
            0
        } else {
            self.selected_section()
        };
        let mut options_view = None;
        frame(
            ui,
            area,
            self.title(),
            "Primary, secondary, subtle, danger, toggle, disabled, busy",
            |ui, body| match section {
                1 => options_view = self.draw_options(ui, body),
                2 => self.draw_states(ui, body),
                _ => self.draw_playground_section(ui, body, area),
            },
        );

        if !tabs_layout.visible_area.is_empty() {
            let _ = tabs.draw_layout(ui, &tabs_layout);
        }
        self.drawn_layout.set(Some(ButtonsDrawFacts {
            section,
            options_view,
        }));
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        let drawn = self.drawn_layout.get().unwrap_or(ButtonsDrawFacts {
            section: 0,
            options_view: None,
        });
        if ui.state(PAGE_TABS).contains(StateFlags::FOCUSED) {
            return TABS_HINTS;
        }
        if let Some(id) = OPTION_IDS
            .into_iter()
            .find(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            let open = match id {
                FOCUS_VISIBILITY => self.focus_visibility_state.is_open(),
                ACTION_FOCUS => self.action_focus_state.is_open(),
                ROW_ALIGNMENT => self.row_alignment_state.is_open(),
                ROW_GAP => self.row_gap_state.is_open(),
                _ => false,
            };
            let visible = drawn.section == 1
                && drawn
                    .options_view
                    .is_some_and(|view| option_field_is_visible(id, &self.options_scroll, view));
            return if open || visible {
                SELECT_HINTS
            } else {
                HIDDEN_OPTION_HINTS
            };
        }
        if ui.state(OPTIONS_SCROLL).contains(StateFlags::FOCUSED)
            || ui.state(STATES_SCROLL).contains(StateFlags::FOCUSED)
        {
            return SCROLL_HINTS;
        }
        if (0..SPECS.len()).any(|index| {
            ui.state(Self::button_id(index))
                .contains(StateFlags::FOCUSED)
        }) {
            return BUTTON_HINTS;
        }
        match drawn.section {
            1 => SELECT_HINTS,
            2 => SCROLL_HINTS,
            _ => BUTTON_HINTS,
        }
    }
}

impl ButtonsPage {
    fn draw_playground_section(&self, ui: &mut Ui<'_>, body: Rect, page_area: Rect) {
        let regions = layout::rows(
            body,
            &[
                termrock::Track::Fixed(15),
                termrock::Track::Fixed(1),
                termrock::Track::Fixed(11),
                termrock::Track::Flex(1),
            ],
        );
        playground_panel().draw(ui, regions.first().copied().unwrap_or(body), |ui, inner| {
            self.draw_playground(ui, inner)
        });
        let matrix_area = regions.get(2).copied().unwrap_or(body);
        if page_area.width < 70 && matrix_area.is_empty() {
            let row = Rect {
                y: page_area.bottom(),
                height: 1,
                ..page_area
            };
            let ink = ui
                .surface_style()
                .fg(termrock::theme::baseline_muted(ui.theme().capability.color))
                .bg(termrock::Color::from_u32(0x0011_1111));
            for (index, title) in ["Primary", "Secondary"].into_iter().enumerate() {
                let x = page_area
                    .x
                    .saturating_add(15)
                    .saturating_add(15 * u16::try_from(index).unwrap_or(0));
                if x.saturating_add(15) > page_area.right() {
                    break;
                }
                ui.paint_str(Rect::new(x, row.y, 15, 1), title, ink);
            }
        } else if matrix_area.width < 70 && !matrix_area.is_empty() {
            matrix_panel().draw(ui, matrix_area, |_, _| ());
            Self::draw_matrix(
                ui,
                Rect {
                    x: matrix_area.x.saturating_add(2),
                    y: matrix_area.y.saturating_add(2),
                    width: matrix_area.width.saturating_sub(4),
                    height: 1,
                },
            );
        } else {
            matrix_panel().draw(ui, matrix_area, Self::draw_matrix);
        }
        if let Some(status) = regions.get(3).copied() {
            if let Some(last) = &self.last {
                let text = format!("last: {last} · {} activations", self.clicks);
                let _ = ui.paint_str(status, &text, ui.surface_style());
            } else if !status.is_empty() {
                ui.fill(
                    status,
                    ui.surface_style()
                        .bg(termrock::Color::from_u32(0x0011_1111)),
                );
            }
        }
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        let gap = self.row_gap(ui.design().space.gap);
        let mut y = area.y;
        for (caption, indices) in GROUPS {
            if y.saturating_add(1) >= area.bottom() {
                break;
            }
            let _ = ui.paint_str(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                caption,
                ui.surface_style()
                    .fg(termrock::theme::baseline_muted(ui.theme().capability.color)),
            );
            let widths: Vec<u16> = indices
                .iter()
                .map(|&index| {
                    self.button(index).map_or(0, |button| {
                        button
                            .measure(ui, Constraints::loose(area.width, 1))
                            .preferred
                            .0
                    })
                })
                .collect();
            let line = Rect {
                y: y.saturating_add(1),
                height: 1,
                ..area
            };
            for (&index, button_area) in
                indices
                    .iter()
                    .zip(layout::action_row(line, &widths, gap, self.row_alignment()))
            {
                if let Some(button) = self.button(index) {
                    button.draw(ui, button_area);
                }
            }
            y = y.saturating_add(3);
        }
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) -> Option<Rect> {
        if area.is_empty() {
            return Some(area);
        }
        let view =
            ScrollRegion::new(OPTIONS_SCROLL).draw(ui, area, &self.options_scroll, OPTION_ROWS);
        if view.is_empty() {
            return Some(view);
        }
        ui.register_focus_only(OPTIONS_SCROLL, Focusability::Focusable);
        let Some(window) = option_window(&self.options_scroll, view) else {
            return Some(view);
        };
        let source_rows = window.visible_source_range();
        let rows = [
            (FOCUS_VISIBILITY, FOCUS_VISIBILITY_ITEMS),
            (ACTION_FOCUS, ACTION_FOCUS_ITEMS),
            (ROW_ALIGNMENT, ROW_ALIGNMENT_ITEMS),
            (ROW_GAP, ROW_GAP_ITEMS),
        ];
        for (source_y, ((id, label), (select_id, items))) in OPTION_IDS
            .into_iter()
            .zip(OPTION_LABELS)
            .zip(rows)
            .enumerate()
        {
            debug_assert_eq!(id, select_id);
            if source_y < source_rows.start || source_y >= source_rows.end {
                Select::new(select_id).register_focus_only_in_form(ui, false);
                continue;
            }
            let Some(y) = window.physical_row_for_source(source_y) else {
                Select::new(select_id).register_focus_only_in_form(ui, false);
                continue;
            };
            let (label_area, control_area) = option_row_areas(view, y);
            if !label_area.is_empty() {
                let _ = ui.paint_str(label_area, label, ui.surface_style());
            }
            if control_area.is_empty() {
                Select::new(select_id).register_focus_only_in_form(ui, false);
                continue;
            }
            match source_y {
                0 => {
                    let _ = Select::new(FOCUS_VISIBILITY).popup_rows(2).draw(
                        ui,
                        control_area,
                        &self.focus_visibility_state,
                        items,
                    );
                }
                1 => {
                    let _ = Select::new(ACTION_FOCUS).popup_rows(2).draw(
                        ui,
                        control_area,
                        &self.action_focus_state,
                        items,
                    );
                }
                2 => {
                    let _ = Select::new(ROW_ALIGNMENT).popup_rows(2).draw(
                        ui,
                        control_area,
                        &self.row_alignment_state,
                        items,
                    );
                }
                3 => {
                    let _ = Select::new(ROW_GAP).popup_rows(3).draw(
                        ui,
                        control_area,
                        &self.row_gap_state,
                        items,
                    );
                }
                _ => unreachable!("the option list and native Select rows stay aligned"),
            }
        }
        Some(view)
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let view = ScrollRegion::new(STATES_SCROLL).draw(ui, area, &self.states_scroll, STATE_ROWS);
        if view.is_empty() {
            return;
        }
        ui.register_focus_only(STATES_SCROLL, Focusability::Focusable);
        let projected = ScrollRegion::view(&self.states_scroll, view, STATE_ROWS);
        let first_row = projected.offset().min(STATE_ROWS);
        let visible = Rect {
            height: view
                .height
                .min(u16::try_from(STATE_ROWS).unwrap_or(u16::MAX)),
            ..view
        };
        let Ok(window) = VerticalWindow::new(visible, STATE_ROWS, first_row) else {
            return;
        };
        for source_y in window.visible_source_range() {
            let Some(y) = window.physical_row_for_source(source_y) else {
                continue;
            };
            let label_width = 12u16.min(view.width);
            let button_x = view.x.saturating_add(label_width).saturating_add(1);
            let available_button_width = view.right().saturating_sub(button_x);
            let button_width = if source_y == 2 {
                18.min(available_button_width)
            } else {
                28.min(available_button_width)
            };
            let button_area = Rect::new(button_x, y, button_width, 1);
            let (label, button) = match source_y {
                0 => (
                    "Selected",
                    Button::new(STATE_BUTTONS.index(0), "Selected")
                        .variant(Variant::TOGGLE)
                        .checked(true),
                ),
                1 => (
                    "Busy",
                    Button::new(STATE_BUTTONS.index(1), "Running")
                        .variant(Variant::PRIMARY)
                        .status(Status::Busy),
                ),
                _ => (
                    "Overflow",
                    Button::new(
                        STATE_BUTTONS.index(2),
                        "Long action label clips to this row",
                    )
                    .variant(Variant::SECONDARY),
                ),
            };
            if label_width > 0 {
                let _ = ui.paint_str(
                    Rect::new(view.x, y, label_width, 1),
                    label,
                    ui.surface_style(),
                );
            }
            if !button_area.is_empty() {
                ui.reference(None, |ui| button.draw(ui, button_area));
            }
        }
    }

    fn draw_matrix(ui: &mut Ui<'_>, area: Rect) {
        let label_width = 15u16;
        let column_width = 15u16;
        let column_x = |index: usize| {
            area.x.saturating_add(
                label_width.saturating_add(column_width.saturating_mul(index as u16)),
            )
        };
        for (index, (_, title)) in MATRIX_VARIANTS.iter().enumerate() {
            let x = column_x(index);
            if x.saturating_add(column_width) > area.right() {
                break;
            }
            let _ = ui.paint_str(
                Rect {
                    x,
                    y: area.y,
                    width: column_width,
                    height: 1,
                },
                title,
                ui.surface_style()
                    .fg(termrock::theme::baseline_muted(ui.theme().capability.color))
                    .bg(termrock::Color::from_u32(0x0011_1111)),
            );
        }
        let row_ink = ui
            .surface_style()
            .fg(ui.theme().color.fg[termrock::FgStep::Secondary.index()]);
        for (state_index, (name, flags)) in MATRIX_STATES.iter().enumerate() {
            let y = area.y.saturating_add(1).saturating_add(state_index as u16);
            if y >= area.bottom() {
                break;
            }
            let _ = ui.paint_str(
                Rect {
                    x: area.x,
                    y,
                    width: label_width,
                    height: 1,
                },
                name,
                row_ink,
            );
            for (variant_index, (variant, _)) in MATRIX_VARIANTS.iter().enumerate() {
                let x = column_x(variant_index);
                if x.saturating_add(column_width) > area.right() {
                    break;
                }
                let id = MATRIX.index(state_index).index(variant_index);
                let target = matrix_reference(*flags).map(|state| {
                    ReferenceTarget::new(id, state).part(PartRef::of(Part::CONTAINER))
                });
                ui.reference(target, |ui| {
                    let mut button = Button::new(id, " Label")
                        .variant(*variant)
                        .disabled(flags.contains(StateFlags::DISABLED))
                        .busy_marker_policy(BusyMarkerPolicy::Replace);
                    if *variant == Variant::SUBTLE && flags.contains(StateFlags::DISABLED) {
                        button = button.patch_part(DISABLED_PLATE);
                    } else if *variant == Variant::PRIMARY && flags.contains(StateFlags::PRESSED) {
                        button = button.patch_part(PRESSED_BOLD);
                    } else if *variant == Variant::DANGER
                        && !flags.contains(StateFlags::PRESSED)
                        && !flags.contains(StateFlags::DISABLED)
                    {
                        button = button.patch_part(DANGER_INK);
                    }
                    button.draw(
                        ui,
                        Rect {
                            x,
                            y,
                            width: column_width,
                            height: 1,
                        },
                    );
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::{
        ACTION_FOCUS, BUTTONS, ButtonsPage, FOCUS_VISIBILITY, HIDDEN_OPTION_HINTS, MATRIX,
        OPTIONS_SCROLL, PAGE_TABS, Page, ROW_ALIGNMENT, ROW_GAP, SELECT_HINTS, STATE_BUTTONS,
        STATES_SCROLL, option_field_is_visible,
    };
    use termrock::{
        App, Axis, Cx, GlyphRole, Id, ItemKey, KeyCode, Part, PartRef, Rect, Response, ScrollState,
        Theme, Ui,
    };
    use termrock_test_support::Harness;

    struct ShowcaseButtons {
        page: ButtonsPage,
        area: Option<Rect>,
        focused_hints: Cell<&'static [(&'static str, &'static str)]>,
    }

    impl ShowcaseButtons {
        fn new(area: Option<Rect>) -> Self {
            Self {
                page: ButtonsPage::new(),
                area,
                focused_hints: Cell::new(&[]),
            }
        }
    }

    impl App for ShowcaseButtons {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.page.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.page.draw(ui, self.area.unwrap_or_else(|| ui.full()));
            self.focused_hints.set(self.page.hints(ui));
        }
    }

    fn host(width: u16, height: u16) -> Harness<ShowcaseButtons> {
        Harness::new(
            ShowcaseButtons::new(Some(Rect::new(0, 0, width, height))),
            Theme::junie(),
            width,
            height,
        )
    }

    fn offset_host(width: u16, height: u16, area: Rect) -> Harness<ShowcaseButtons> {
        Harness::new(
            ShowcaseButtons::new(Some(area)),
            Theme::junie(),
            width,
            height,
        )
    }

    fn choose_option(h: &mut Harness<ShowcaseButtons>, id: Id, index: usize) {
        let _ = h.click_part(id, PartRef::of(Part::FIELD));
        assert!(h.is_open(id), "the native Select popup opens");
        let _ = h.click_part(id, PartRef::item(Part::ROW, ItemKey::index(index)));
        assert!(!h.is_open(id), "choosing closes the native Select popup");
    }

    #[test]
    fn default_heading_tabs_keep_playground_geometry_and_focus_before_navigation() {
        let page = ShowcaseButtons::new(None);
        let theme = Theme::junie()
            .builder()
            .glyph(GlyphRole::DotOff, "o")
            .glyph(GlyphRole::DotOn, "x")
            .glyph(GlyphRole::FocusBar, "f")
            .build();
        let mut harness = Harness::new(page, theme, 120, 40);
        let tabs = harness
            .area_of(PAGE_TABS)
            .expect("native tabs occupy the heading strip");
        assert_eq!(tabs, Rect::new(0, 1, 120, 1));
        let first = BUTTONS.index(0);
        let first_area = harness
            .area_of(first)
            .expect("the first Playground button is drawn");
        assert_eq!(first_area.y, 5, "the heading strip does not shift the body");
        assert_eq!(harness.ring().entries()[0].id, first);
        let first_position = harness
            .ring()
            .entries()
            .iter()
            .position(|entry| entry.id == first)
            .expect("first button is registered");
        let tabs_position = harness
            .ring()
            .entries()
            .iter()
            .position(|entry| entry.id == PAGE_TABS)
            .expect("Tabs is a native focus stop");
        assert!(first_position < tabs_position);
        assert!(harness.tab_to(first));
        assert_eq!(harness.focus(), Some(first));
        let _ = harness.key(KeyCode::Tab);
        assert_eq!(harness.focus(), Some(BUTTONS.index(1)));

        let toggle = BUTTONS.index(4);
        let area = harness.area_of(toggle).expect("toggle is registered");

        assert_eq!(harness.app().page.checked[4], Some(false));
        assert_eq!(harness.cell(area.x.saturating_add(1), area.y).symbol(), "o");
        assert!(harness.tab_to(toggle));
        assert_eq!(harness.cell(area.x, area.y).symbol(), "f");
        assert!(harness.click_id(toggle).is_changed());
        assert_eq!(harness.app().page.checked[4], Some(true));
        assert_eq!(harness.cell(area.x.saturating_add(1), area.y).symbol(), "x");

        let reference_button = MATRIX.index(0).index(0);
        assert!(harness.area_of(reference_button).is_none());
        assert!(harness.text().contains("focus + hover"));
        assert!(area.width > 0);
    }

    #[test]
    fn native_options_change_focus_visibility_and_action_row_layout() {
        let mut baseline = Harness::new(
            ShowcaseButtons::new(None),
            Theme::junie()
                .builder()
                .glyph(GlyphRole::FocusBar, "f")
                .build(),
            120,
            40,
        );
        let run = BUTTONS.index(0);
        let default_run = baseline.area_of(run).expect("default action geometry");
        let _ = baseline.click_id(run);
        assert_eq!(baseline.cell(default_run.x, default_run.y).symbol(), "f");

        let mut h = Harness::new(
            ShowcaseButtons::new(None),
            Theme::junie()
                .builder()
                .glyph(GlyphRole::FocusBar, "f")
                .build(),
            120,
            40,
        );
        let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(1)));
        assert_eq!(h.app().page.selected_section(), 1);
        for value in ["Always", "In Tab order", "Start", "Theme"] {
            assert!(
                h.text().contains(value),
                "the initial native Select shows {value}"
            );
        }
        choose_option(&mut h, FOCUS_VISIBILITY, 1);
        choose_option(&mut h, ACTION_FOCUS, 1);
        choose_option(&mut h, ROW_ALIGNMENT, 1);
        choose_option(&mut h, ROW_GAP, 2);
        assert_eq!(h.app().page.focus_visibility_index, 1);
        assert_eq!(h.app().page.action_focus_index, 1);
        assert_eq!(h.app().page.row_alignment_index, 1);
        assert_eq!(h.app().page.row_gap_index, 2);

        let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(0)));
        assert_eq!(h.app().page.selected_section(), 0);
        let run_area = h
            .area_of(run)
            .expect("selected options still render real Buttons");
        let preview_area = h
            .area_of(BUTTONS.index(1))
            .expect("second action button is present");
        assert!(
            run_area.x > default_run.x,
            "End alignment changes native hit geometry"
        );
        assert_eq!(preview_area.x.saturating_sub(run_area.right()), 4);
        assert!(
            !h.ring().contains(run),
            "Pointer only removes Actions from Tab traversal"
        );
        assert!(
            h.ring().contains(BUTTONS.index(4)),
            "other button groups remain focusable"
        );
        let toggle = BUTTONS.index(4);
        let toggle_area = h.area_of(toggle).expect("toggle is still rendered");
        let _ = h.click_id(toggle);
        assert_ne!(
            h.cell(toggle_area.x, toggle_area.y).symbol(),
            "f",
            "Keyboard only suppresses the pointer focus marker on a focusable button"
        );
        let _ = h.click_id(run);
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    fn opening_select_consumes_the_first_outside_tab_click() {
        let mut h = host(120, 40);
        let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(1)));
        choose_option(&mut h, FOCUS_VISIBILITY, 0);
        let _ = h.click_part(FOCUS_VISIBILITY, PartRef::of(Part::FIELD));
        assert!(h.app().page.focus_visibility_state.is_open());
        assert!(h.is_open(FOCUS_VISIBILITY));
        assert_eq!(h.app().focused_hints.get(), SELECT_HINTS);

        let tab_states = PartRef::item(Part::TAB, ItemKey::index(2));
        let _ = h.click_part(PAGE_TABS, tab_states);
        assert_eq!(h.app().page.selected_section(), 1);
        assert!(!h.app().page.focus_visibility_state.is_open());
        assert!(!h.is_open(FOCUS_VISIBILITY));
        assert_eq!(
            h.app().focused_hints.get(),
            SELECT_HINTS,
            "a dismissed popup keeps selection hints while its field is visible",
        );

        let _ = h.click_part(PAGE_TABS, tab_states);
        assert_eq!(h.app().page.selected_section(), 2);
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    fn compact_options_scroll_reveals_focus_but_wheel_does_not_snap_back() {
        let page_area = Rect::new(7, 5, 104, 5);
        let mut h = offset_host(120, 40, page_area);
        let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(1)));
        assert_eq!(h.app().page.selected_section(), 1);
        let viewport = h
            .area_of(OPTIONS_SCROLL)
            .expect("the compact options page registers its wheel target");
        assert!(viewport.x >= page_area.x && viewport.y >= page_area.y);
        assert!(viewport.right() <= page_area.right() && viewport.bottom() <= page_area.bottom());
        assert!(h.ring().contains(ROW_GAP));
        assert!(
            h.area_of(ROW_GAP).is_none(),
            "the last Select is focus-only while clipped"
        );

        assert!(h.tab_to(OPTIONS_SCROLL));
        let _ = h.key(KeyCode::PageDown);
        assert_eq!(h.app().page.options_scroll.offset(), 1);
        let _ = h.key(KeyCode::PageUp);
        assert_eq!(h.app().page.options_scroll.offset(), 0);

        assert!(h.tab_to(ROW_GAP));
        assert_eq!(h.app().page.options_scroll.offset(), 1);
        assert!(
            h.area_of(ROW_GAP).is_some(),
            "focus transition reveals the native Select"
        );
        let _ = h.click_part(ROW_GAP, PartRef::of(Part::FIELD));
        assert!(h.is_open(ROW_GAP));
        assert_eq!(h.app().focused_hints.get(), SELECT_HINTS);
        let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(1)));
        assert!(!h.is_open(ROW_GAP));
        assert_eq!(h.app().page.selected_section(), 1);
        assert_eq!(
            h.app().focused_hints.get(),
            SELECT_HINTS,
            "closing the popup does not hide its still-visible field",
        );
        let viewport = h
            .area_of(OPTIONS_SCROLL)
            .expect("viewport remains registered");
        let _ = h.wheel(Axis::V, -1, viewport.x, viewport.y);
        assert_eq!(h.app().page.options_scroll.offset(), 0);
        assert_eq!(h.focus(), Some(ROW_GAP));
        assert!(h.area_of(ROW_GAP).is_none());
        assert!(h.ring().contains(ROW_GAP));
        assert!(h.cursor().is_none());
        assert_eq!(
            h.app().focused_hints.get(),
            &[("Tab / Shift+Tab", "Focus"), ("Wheel", "Scroll")],
            "hidden focus-only options do not advertise PgUp/Dn",
        );
        let old_value = h.app().page.row_gap_index;
        for key in [KeyCode::Enter, KeyCode::Char(' ')] {
            let _ = h.key(key);
            assert_eq!(h.app().page.row_gap_index, old_value);
            assert!(!h.app().page.row_gap_state.is_open());
            assert!(!h.is_open(ROW_GAP));
        }

        let _ = h.key(KeyCode::BackTab);
        assert_eq!(h.focus(), Some(ROW_ALIGNMENT));
        let _ = h.key(KeyCode::Tab);
        assert_eq!(h.focus(), Some(ROW_GAP));
        assert_eq!(h.app().page.options_scroll.offset(), 1);
        assert!(h.area_of(ROW_GAP).is_some());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn focused_option_hints_follow_the_current_draw_after_resize() {
        let full_area = Rect::new(7, 5, 104, 5);
        let short_area = Rect::new(7, 5, 104, 4);
        let mut h = offset_host(120, 40, full_area);
        let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(1)));
        assert_eq!(h.app().page.selected_section(), 1);
        assert_eq!(
            h.area_of(OPTIONS_SCROLL).map(|area| area.height),
            Some(3),
            "the starting view includes option row two",
        );
        assert!(h.tab_to(ROW_ALIGNMENT));
        assert_eq!(h.app().page.options_scroll.offset(), 0);
        assert!(h.area_of(ROW_ALIGNMENT).is_some());
        assert_eq!(h.app().focused_hints.get(), SELECT_HINTS);

        h.app_mut().area = Some(short_area);
        let _ = h.resize(120, 40);
        assert_eq!(h.focus(), Some(ROW_ALIGNMENT));
        assert_eq!(h.app().page.options_scroll.offset(), 0);
        assert_eq!(
            h.area_of(OPTIONS_SCROLL).map(|area| area.height),
            Some(2),
            "resizing reduces the current content view by one row",
        );
        assert!(h.area_of(ROW_ALIGNMENT).is_none());
        assert_eq!(h.app().focused_hints.get(), HIDDEN_OPTION_HINTS);

        h.app_mut().area = Some(full_area);
        let _ = h.resize(120, 40);
        assert_eq!(h.focus(), Some(ROW_ALIGNMENT));
        assert_eq!(h.app().page.options_scroll.offset(), 0);
        assert!(h.area_of(ROW_ALIGNMENT).is_some());
        assert_eq!(h.app().focused_hints.get(), SELECT_HINTS);
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    fn narrow_options_hints_use_the_scroll_gutter_adjusted_content_view() {
        let raw_view = Rect::new(0, 0, 2, 3);
        let content_view = Rect::new(0, 0, 1, 3);
        let scroll = ScrollState::default();
        assert!(option_field_is_visible(ROW_ALIGNMENT, &scroll, raw_view));
        assert!(!option_field_is_visible(
            ROW_ALIGNMENT,
            &scroll,
            content_view
        ));

        let mut h = offset_host(40, 20, Rect::new(0, 0, 24, 5));
        assert!(h.tab_to(PAGE_TABS));
        let _ = h.key(KeyCode::Right);
        assert_eq!(h.app().page.selected_section(), 1);
        let registered_view = h
            .area_of(OPTIONS_SCROLL)
            .expect("narrow native Options page registers its scroll region");
        let drawn = h
            .app()
            .page
            .drawn_layout
            .get()
            .expect("page publishes facts from this draw");
        let drawn_view = drawn
            .options_view
            .expect("the current Options frame has a content view");
        assert_eq!(drawn.section, 1);
        assert_eq!(drawn_view.height, registered_view.height);
        assert_eq!(drawn_view.x, registered_view.x);
        assert_eq!(drawn_view.y, registered_view.y);
        assert_eq!(drawn_view.width.saturating_add(1), registered_view.width);
        assert!(h.tab_to(ROW_ALIGNMENT));
        assert!(h.area_of(ROW_ALIGNMENT).is_some());
        assert_eq!(h.app().focused_hints.get(), SELECT_HINTS);
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    fn state_specimens_use_native_selected_busy_and_overflow_rendering_without_targets() {
        let mut h = host(120, 40);
        let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(2)));
        assert_eq!(h.app().page.selected_section(), 2);
        assert!(h.area_of(STATES_SCROLL).is_some());
        assert!(h.text().contains("● Selected"));
        assert!(h.text().contains("Running"));
        assert!(h.text().contains("Overflow"));
        assert!(h.text().contains("Long"));
        assert!(!h.text().contains("Long action label clips to this row"));
        for index in 0..3 {
            assert!(h.area_of(STATE_BUTTONS.index(index)).is_none());
        }
        assert!(h.diagnostics().is_empty());
    }
}
