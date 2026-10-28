use termrock::author::PartStyle;
use termrock::author::VerticalWindow;
use termrock::shell::Crumb;
use termrock::{
    ActionKey, Binding, Button, Chord, Constraints, Cx, Family, FgStep, Focusability, FrameRead,
    GlyphRole, Id, Intent, ItemKey, KeyCode, NavList, NavListAction, NavListState, Panel,
    PanelKind, Part, PartRef, Phase, Rect, ReferenceState, ReferenceTarget, Response, Role, RowUi,
    ScopeId, ScrollRegion, ScrollState, Select, SelectState, Size, StateFlags, StylePatch, Tabs,
    TabsState, Ui, UpdateCause, Variant, width, wrap,
};

use super::{Page, PageStatus, PageUpdate, frame};

pub const TOGGLE_RAIL_ACTION: ActionKey =
    ActionKey::application("showcase.breadcrumbs.toggle-rail");

const PAGE_TABS: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.page_tabs");
const PLAY_SCROLL: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.play_scroll");
const LIST: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.explorer");
const MAIN_CRUMB: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.main_crumb");
const COLLAPSE: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.collapse");
const EXPAND: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.expand");
const EXPLORER_PANEL: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.explorer_panel");
const LOCATION_PANEL: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.location_panel");
const RAIL_PANEL: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.rail_panel");
const TRAILS_PANEL: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.trails_panel");
const PLAIN_TRAIL: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.plain_trail");
const DEEP_TRAIL: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.deep_trail");
const STATE_ROOT: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.states");

const LINKS_OPTION: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.option.links");
const WIDTH_OPTION: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.option.width");
const COLLAPSED_OPTION: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.option.collapsed");
const RAIL_OPTION: Id = Id::root("showcase_ui::pages::breadcrumbs_rails.option.collapsed_rail");
const OPTION_IDS: [Id; 4] = [LINKS_OPTION, WIDTH_OPTION, COLLAPSED_OPTION, RAIL_OPTION];
const OPTION_KEYS: [&str; 4] = [
    "breadcrumbsrails.clickable",
    "breadcrumbsrails.width",
    "breadcrumbsrails.collapsed",
    "breadcrumbsrails.collapsed_rail",
];
const OPTION_LABELS: [&str; 4] = [
    "Links",
    "Deep trail width",
    "Sidebar collapsed",
    "Collapsed rail",
];
const OPTION_DEFAULTS: [usize; 4] = [1, 0, 0, 1];
const OPTION_ITEMS: [&[&str]; 4] = [
    &["off", "on"],
    &["full", "48", "32", "20"],
    &["off", "on"],
    &["off", "on"],
];

const SECTIONS: &[&str] = &["Playground", "Options", "States"];
const BLURB: &str =
    "Breadcrumb trails (static, clickable, collapsing) and the collapsible SidebarRail";
const SECTION_HINTS: &[(&str, &str)] =
    &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
const LIST_HINTS: &[(&str, &str)] = &[
    ("↑ ↓", "Move"),
    ("Enter", "Open"),
    ("Backspace", "Up a level"),
    ("Ctrl+B", "Hide explorer"),
];
const COLLAPSED_HINTS: &[(&str, &str)] = &[("Ctrl+B", "Show explorer")];
const OPTION_HINTS: &[(&str, &str)] = &[("↑ ↓", "Choose"), ("Enter", "Open / choose")];
const BACKSPACE_ACTION: ActionKey = ActionKey::application("showcase.breadcrumbs.go-up");
const BACKSPACE_BINDINGS: &[Binding<()>] = &[Binding {
    action: BACKSPACE_ACTION,
    chord: Some(Chord::key(KeyCode::Backspace)),
    cmd: (),
    label: "Go up one level",
    priority: 60,
    visible: true,
}];

const EXPLORER_WIDTH: u16 = 34;
const ENTRY_ROWS: u16 = 18;
const EXPLORER_BODY_START: usize = 1;
const DEEP_SEGMENTS: [&str; 6] = [
    "Disk",
    "Users",
    "alex",
    "Projects",
    "payments-platform",
    "settlement-backoff-retry-policy-v2.md",
];

#[derive(Clone, Copy)]
struct Entry {
    name: &'static str,
    meta: &'static str,
    opens: bool,
    disabled: bool,
}

const DISK: &[Entry] = &[
    Entry {
        name: "Applications",
        meta: "folder",
        opens: true,
        disabled: false,
    },
    Entry {
        name: "Library",
        meta: "folder",
        opens: true,
        disabled: false,
    },
    Entry {
        name: "System",
        meta: "locked",
        opens: false,
        disabled: true,
    },
    Entry {
        name: "Users",
        meta: "folder",
        opens: true,
        disabled: false,
    },
];
const USERS: &[Entry] = &[
    Entry {
        name: "alex",
        meta: "folder",
        opens: true,
        disabled: false,
    },
    Entry {
        name: "Shared",
        meta: "folder",
        opens: true,
        disabled: false,
    },
];
const ALEX: &[Entry] = &[
    Entry {
        name: "Desktop",
        meta: "folder",
        opens: true,
        disabled: false,
    },
    Entry {
        name: "Documents",
        meta: "folder",
        opens: true,
        disabled: false,
    },
    Entry {
        name: "Downloads",
        meta: "folder",
        opens: true,
        disabled: false,
    },
    Entry {
        name: "Projects",
        meta: "folder",
        opens: true,
        disabled: false,
    },
];
const PROJECTS: &[Entry] = &[
    Entry {
        name: "design-system",
        meta: "folder",
        opens: true,
        disabled: false,
    },
    Entry {
        name: "payments-platform",
        meta: "folder",
        opens: true,
        disabled: false,
    },
    Entry {
        name: "notes.md",
        meta: "4 KB",
        opens: false,
        disabled: false,
    },
];
const PAYMENTS: &[Entry] = &[
    Entry {
        name: "src",
        meta: "folder",
        opens: true,
        disabled: false,
    },
    Entry {
        name: "Cargo.toml",
        meta: "1 KB",
        opens: false,
        disabled: false,
    },
    Entry {
        name: "settlement-backoff-retry-policy-v2.md",
        meta: "12 KB",
        opens: false,
        disabled: false,
    },
];
const APPLICATIONS: &[Entry] = &[
    Entry {
        name: "Terminal",
        meta: "app",
        opens: false,
        disabled: false,
    },
    Entry {
        name: "Xcode",
        meta: "app",
        opens: false,
        disabled: false,
    },
];
const LIBRARY: &[Entry] = &[
    Entry {
        name: "Caches",
        meta: "folder",
        opens: true,
        disabled: false,
    },
    Entry {
        name: "Fonts",
        meta: "folder",
        opens: true,
        disabled: false,
    },
];

fn entries(path: &[String]) -> &'static [Entry] {
    match path
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["Disk"] => DISK,
        ["Disk", "Users"] => USERS,
        ["Disk", "Users", "alex"] => ALEX,
        ["Disk", "Users", "alex", "Projects"] => PROJECTS,
        ["Disk", "Users", "alex", "Projects", "payments-platform"] => PAYMENTS,
        ["Disk", "Applications"] => APPLICATIONS,
        ["Disk", "Library"] => LIBRARY,
        _ => &[],
    }
}

fn entry_key(entry: &Entry) -> ItemKey {
    ItemKey::text(entry.name)
}

fn entry_row(entry: &Entry, row: &mut RowUi<'_>) {
    row.label(entry.name);
    if entry.disabled {
        row.meta_with_defaults(
            entry.meta,
            termrock::StyleDefaults::new(StylePatch::new().set_fg(Role::Warning)),
        );
    } else if !entry.opens {
        row.meta_with_defaults(
            entry.meta,
            termrock::StyleDefaults::new(StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))),
        );
    } else {
        row.meta(entry.meta);
    }
}

fn entry_disabled(entry: &Entry) -> bool {
    entry.disabled
}

fn explorer() -> NavList<'static, Entry, impl Fn(&Entry) -> ItemKey, impl Fn(&Entry, &mut RowUi<'_>)>
{
    NavList::new(LIST)
        .key(entry_key)
        .row(entry_row)
        .disabled_item(&entry_disabled)
        .scrollable(true)
}

#[derive(Clone)]
struct VisibleCrumb {
    index: usize,
    text: String,
}

struct TrailLayout {
    leading_ellipsis: bool,
    ellipsis: &'static str,
    separator: String,
    pieces: Vec<VisibleCrumb>,
}

fn trail_layout(ui: &Ui<'_>, area: Rect, crumb: &Crumb) -> TrailLayout {
    let separator = format!(" {} ", ui.glyph_str(GlyphRole::PathSep));
    let sep_width = width(&separator);
    let ellipsis = ui.glyph_str(GlyphRole::Ellipsis);
    let ellipsis_width = width(ellipsis);
    let segments = &crumb.segments;
    if segments.is_empty() || area.width == 0 {
        return TrailLayout {
            leading_ellipsis: false,
            ellipsis,
            separator,
            pieces: Vec::new(),
        };
    }

    let width_from = |start: usize| {
        let mut total = 0_u16;
        for segment in &segments[start..] {
            total = total.saturating_add(width(segment));
        }
        let count = segments.len().saturating_sub(start);
        total = total.saturating_add(sep_width.saturating_mul(count.saturating_sub(1) as u16));
        if start > 0 {
            total = total
                .saturating_add(ellipsis_width)
                .saturating_add(sep_width);
        }
        total
    };

    if let Some(start) = (0..segments.len()).find(|start| width_from(*start) <= area.width) {
        let pieces = segments
            .iter()
            .enumerate()
            .skip(start)
            .map(|(index, text)| VisibleCrumb {
                index,
                text: text.clone(),
            })
            .collect();
        return TrailLayout {
            leading_ellipsis: start > 0,
            ellipsis,
            separator,
            pieces,
        };
    }

    let index = segments.len().saturating_sub(1);
    let prefix_width = if index > 0 {
        ellipsis_width.saturating_add(sep_width)
    } else {
        0
    };
    let leading_ellipsis = prefix_width < area.width;
    let budget = area
        .width
        .saturating_sub(if leading_ellipsis { prefix_width } else { 0 });
    let text = termrock::text::truncate_middle(&segments[index], budget);
    TrailLayout {
        leading_ellipsis,
        ellipsis,
        separator,
        pieces: vec![VisibleCrumb { index, text }],
    }
}

fn button_part_style(ui: &mut Ui<'_>, id: Id, part: Part) -> termrock::author::PaintStyle {
    let flags = PartStyle::flags(ui.state(id), StateFlags::empty());
    PartStyle::new()
        .style(ui, id, Family::BUTTON, Variant::SECONDARY, part, flags)
        .style
}

fn draw_link(ui: &mut Ui<'_>, id: Id, area: Rect, label: &str) {
    if area.is_empty() {
        return;
    }
    ui.register_control(id, area, Focusability::ClickOnly);
    let container = button_part_style(ui, id, Part::CONTAINER);
    let text = button_part_style(ui, id, Part::LABEL);
    ui.fill(area, container);
    ui.paint_str(area, label, text);
}

fn draw_trail(ui: &mut Ui<'_>, area: Rect, crumb: &Crumb, clickable: bool) {
    if area.is_empty() {
        return;
    }
    ui.with_area(area, |ui| {
        let layout = trail_layout(ui, area, crumb);
        let plain = ui.surface_style();
        let detail = ui
            .style(
                Family::PANEL,
                Variant::DEFAULT,
                Part::DETAIL,
                StateFlags::empty(),
            )
            .style;
        let mut x = area.x;
        let y = area.y;
        let mut need_separator = false;
        if layout.leading_ellipsis {
            let ellipsis_width = width(layout.ellipsis).min(area.right().saturating_sub(x));
            ui.paint_str(Rect::new(x, y, ellipsis_width, 1), layout.ellipsis, detail);
            x = x.saturating_add(ellipsis_width);
            let separator_width = width(&layout.separator).min(area.right().saturating_sub(x));
            ui.paint_str(
                Rect::new(x, y, separator_width, 1),
                &layout.separator,
                detail,
            );
            x = x.saturating_add(separator_width);
        }
        for piece in layout.pieces {
            if need_separator {
                let separator_width = width(&layout.separator).min(area.right().saturating_sub(x));
                ui.paint_str(
                    Rect::new(x, y, separator_width, 1),
                    &layout.separator,
                    detail,
                );
                x = x.saturating_add(separator_width);
            }
            let available = area.right().saturating_sub(x);
            if available == 0 {
                break;
            }
            let text_width = width(&piece.text).min(available);
            let text_area = Rect::new(x, y, text_width, 1);
            let is_ancestor = piece.index + 1 < crumb.segments.len();
            if clickable && is_ancestor {
                draw_link(ui, crumb.segment_id(piece.index), text_area, &piece.text);
            } else {
                ui.paint_str(text_area, &piece.text, plain);
            }
            x = x.saturating_add(text_width);
            need_separator = true;
        }
    });
}

fn draw_control_glyph(ui: &mut Ui<'_>, id: Id, area: Rect, glyph: &str) {
    if area.is_empty() {
        return;
    }
    ui.register_control(id, area, Focusability::ClickOnly);
    let container = button_part_style(ui, id, Part::CONTAINER);
    let label = button_part_style(ui, id, Part::LABEL);
    ui.fill(area, container);
    let glyph_width = width(glyph).min(area.width);
    let glyph_area = Rect::new(
        area.x
            .saturating_add(area.width.saturating_sub(glyph_width) / 2),
        area.y,
        glyph_width,
        1,
    );
    ui.paint_str(glyph_area, glyph, label);
}

fn draw_rail(ui: &mut Ui<'_>, id: Id, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.with_area(area, |ui| {
        ui.register_control(id, area, Focusability::ClickOnly);
        let container = button_part_style(ui, id, Part::CONTAINER);
        let label_style = button_part_style(ui, id, Part::LABEL);
        ui.fill(area, container);
        let glyph = ui.glyph_str(GlyphRole::OverflowRight);
        ui.paint_str(Rect::new(area.x, area.y, area.width, 1), glyph, label_style);
        for (offset, character) in "Explorer".chars().enumerate() {
            let Ok(offset) = u16::try_from(offset) else {
                break;
            };
            let y = area.y.saturating_add(2).saturating_add(offset);
            if y >= area.bottom() {
                break;
            }
            let character = character.to_string();
            ui.paint_str(Rect::new(area.x, y, area.width, 1), &character, label_style);
        }
    });
}

fn draw_rail_window(ui: &mut Ui<'_>, id: Id, area: Rect, window: VerticalWindow) {
    let visible = window.visible_body();
    if area.is_empty() || visible.is_empty() {
        return;
    }
    ui.with_area(visible, |ui| {
        ui.register_control(id, visible, Focusability::ClickOnly);
        let container = button_part_style(ui, id, Part::CONTAINER);
        let label_style = button_part_style(ui, id, Part::LABEL);
        ui.fill(visible, container);
        let glyph = ui.glyph_str(GlyphRole::OverflowRight);
        if let Some(y) = window.physical_row_for_source(0) {
            ui.paint_str(
                Rect::new(visible.x, y, visible.width, 1),
                glyph,
                label_style,
            );
        }
        for (offset, character) in "Explorer".chars().enumerate() {
            let source = 2_usize.saturating_add(offset);
            let Some(y) = window.physical_row_for_source(source) else {
                continue;
            };
            let character = character.to_string();
            ui.paint_str(
                Rect::new(visible.x, y, visible.width, 1),
                &character,
                label_style,
            );
        }
    });
}

fn draw_wrapped(ui: &mut Ui<'_>, area: Rect, text: &str) -> u16 {
    if area.is_empty() {
        return 0;
    }
    let lines = wrap(text, area.width.max(1));
    let style = ui.surface_style();
    let mut used = 0_u16;
    for line in lines {
        if used >= area.height {
            break;
        }
        let row = Rect {
            y: area.y.saturating_add(used),
            height: 1,
            ..area
        };
        ui.paint_str(row, &line, style);
        used = used.saturating_add(1);
    }
    used
}

fn wrapped_rows(text: &str, width: u16) -> u16 {
    u16::try_from(wrap(text, width.max(1)).len()).unwrap_or(u16::MAX)
}

fn draw_wrapped_window(
    ui: &mut Ui<'_>,
    window: VerticalWindow,
    source_row: usize,
    text: &str,
) -> usize {
    let body = window.visible_body();
    let lines = wrap(text, body.width.max(1));
    for (offset, line) in lines.iter().enumerate() {
        let source = source_row.saturating_add(offset);
        let Some(y) = window.physical_row_for_source(source) else {
            continue;
        };
        ui.paint_str(
            Rect::new(body.x, y, body.width, 1),
            line,
            ui.surface_style(),
        );
    }
    lines.len()
}

fn scroll_window(view: Rect, content_len: usize, state: &ScrollState) -> Option<VerticalWindow> {
    if view.width == 0 || view.height == 0 || content_len == 0 {
        return None;
    }
    let offset = ScrollRegion::view(state, view, content_len)
        .offset()
        .min(content_len);
    let visible_rows = usize::from(view.height).min(content_len.saturating_sub(offset));
    let visible_height = u16::try_from(visible_rows).ok()?;
    if visible_height == 0 {
        return None;
    }
    let visible = Rect {
        height: visible_height,
        ..view
    };
    VerticalWindow::new(visible, content_len, offset).ok()
}

fn child_window(
    parent: VerticalWindow,
    source_start: usize,
    logical_rows: u16,
    x: u16,
    width: u16,
) -> Option<(Rect, VerticalWindow)> {
    if logical_rows == 0 || width == 0 {
        return None;
    }
    let parent_area = parent.visible_body();
    if x < parent_area.x || x >= parent_area.right() {
        return None;
    }
    let width = width.min(parent_area.right().saturating_sub(x));
    if width == 0 {
        return None;
    }
    let source_end = source_start.checked_add(usize::from(logical_rows))?;
    let visible_source = parent.visible_source_range();
    let first = source_start.max(visible_source.start);
    let end = source_end.min(visible_source.end);
    if first >= end {
        return None;
    }
    let physical_y = parent.physical_row_for_source(first)?;
    let visible_height = u16::try_from(end.saturating_sub(first)).ok()?;
    let visible_body = Rect::new(x, physical_y, width, visible_height);
    let window = VerticalWindow::new(
        visible_body,
        usize::from(logical_rows),
        first.saturating_sub(source_start),
    )
    .ok()?;
    Some((Rect::new(x, physical_y, width, logical_rows), window))
}

fn main_path() -> Vec<String> {
    vec!["Disk".to_owned(), "Users".to_owned(), "alex".to_owned()]
}

fn deep_path() -> Vec<String> {
    DEEP_SEGMENTS
        .iter()
        .map(|segment| (*segment).to_owned())
        .collect()
}

pub struct BreadcrumbsRailsPage {
    page_tabs: TabsState,
    play_scroll: ScrollState,
    path: Vec<String>,
    list_state: NavListState,
    collapsed: bool,
    options: [usize; 4],
    select_states: [SelectState; 4],
    last_crumb: String,
    last_rail: String,
    pending_focus_transition: bool,
    scroll_focus_transition: bool,
}

impl BreadcrumbsRailsPage {
    pub fn new() -> Self {
        let mut page = Self {
            page_tabs: TabsState::default(),
            play_scroll: ScrollState::default(),
            path: main_path(),
            list_state: NavListState::default(),
            collapsed: false,
            options: OPTION_DEFAULTS,
            select_states: std::array::from_fn(|_| SelectState::default()),
            last_crumb: "—".to_owned(),
            last_rail: "—".to_owned(),
            pending_focus_transition: false,
            scroll_focus_transition: false,
        };
        page.restore_cursor(None);
        page
    }

    fn section(&self) -> usize {
        match self.page_tabs.active().or(self.page_tabs.cursor()) {
            Some(ItemKey::Index(index)) => index.min(SECTIONS.len().saturating_sub(1)),
            _ => 0,
        }
    }

    fn crumb(&self) -> Crumb {
        Crumb::new(MAIN_CRUMB, self.path.clone())
    }

    fn commit_collapse(&mut self, collapsed: bool, cx: &mut Cx<'_>) -> bool {
        if self.collapsed == collapsed {
            return false;
        }
        self.collapsed = collapsed;
        self.options[2] = usize::from(collapsed);
        self.last_rail = if collapsed { "Collapse" } else { "Expand" }.to_owned();
        if self.section() == 0 {
            if collapsed && cx.focused_in_scope(ScopeId::ROOT) == Some(LIST) {
                cx.focus(PAGE_TABS);
            } else if !collapsed {
                if self.explorer_target_visible(cx, entries(&self.path)) {
                    self.scroll_focus_transition = false;
                    cx.focus(LIST);
                } else {
                    self.scroll_focus_transition = true;
                    cx.focus(PAGE_TABS);
                }
            }
        }
        true
    }

    fn apply_pending_focus_transition(&mut self, cx: &mut Cx<'_>) {
        if !self.pending_focus_transition {
            return;
        }
        self.pending_focus_transition = false;
        if self.section() != 0 {
            return;
        }
        let focused = cx.focused_in_scope(ScopeId::ROOT);
        if self.collapsed && focused == Some(LIST) {
            cx.focus(PAGE_TABS);
        } else if !self.collapsed && focused == Some(PAGE_TABS) {
            if self.explorer_target_visible(cx, entries(&self.path)) {
                self.scroll_focus_transition = false;
                cx.focus(LIST);
            } else {
                self.scroll_focus_transition = true;
            }
        }
    }

    fn explorer_target_source_row(&self, items: &[Entry]) -> Option<usize> {
        if items.is_empty() {
            return Some(EXPLORER_BODY_START);
        }

        // The Explorer NavList has one row per item and no section rows.
        let cursor = self.list_state.cursor()?;
        let item_row = items.iter().position(|item| entry_key(item) == cursor)?;
        let visible_row = item_row.checked_sub(self.list_state.scroll().offset())?;
        (visible_row < usize::from(ENTRY_ROWS))
            .then(|| EXPLORER_BODY_START.saturating_add(visible_row))
    }

    fn explorer_target_visible(&self, cx: &Cx<'_>, items: &[Entry]) -> bool {
        if self.collapsed {
            return false;
        }
        let Some(source_row) = self.explorer_target_source_row(items) else {
            return false;
        };
        let Some(layout) = cx.layout(PLAY_SCROLL) else {
            return false;
        };
        let area = cx.area(PLAY_SCROLL).unwrap_or_default();
        if area.height == 0 {
            return false;
        }
        ScrollRegion::view(&self.play_scroll, area, layout.content_len)
            .visible_range()
            .contains(&source_row)
    }

    fn reconcile_scroll_focus(&mut self, cx: &mut Cx<'_>, list_target_visible: bool) {
        let focused = cx.focused_in_scope(ScopeId::ROOT);
        if !list_target_visible && focused == Some(LIST) {
            cx.focus(PAGE_TABS);
            self.scroll_focus_transition = !self.collapsed;
        } else if list_target_visible && self.scroll_focus_transition {
            if focused == Some(PAGE_TABS) {
                cx.focus(LIST);
            }
            self.scroll_focus_transition = false;
        } else if self.scroll_focus_transition
            && focused.is_some_and(|owner| owner != LIST && owner != PAGE_TABS)
        {
            self.scroll_focus_transition = false;
        }
    }

    fn restore_cursor(&mut self, name: Option<&str>) {
        self.list_state = NavListState::default();
        let items = entries(&self.path);
        let index = name.and_then(|name| items.iter().position(|item| item.name == name));
        if let Some(index) = index.and_then(|index| items.get(index).map(|item| (index, item))) {
            self.list_state.set_cursor(index.0, entry_key(index.1));
        } else if let Some(first) = items.first() {
            self.list_state.set_cursor(0, entry_key(first));
        }
    }

    fn go_to_level(&mut self, level: usize) -> String {
        let left = self.path.get(level.saturating_add(1)).cloned();
        self.path.truncate(level.saturating_add(1));
        self.restore_cursor(left.as_deref());
        format!("Went back to {}", self.path.join(" › "))
    }

    fn go_up(&mut self) -> Option<String> {
        if self.path.len() <= 1 {
            return None;
        }
        let level = self.path.len().saturating_sub(2);
        self.last_crumb = format!("Backspace → level {level}");
        Some(self.go_to_level(level))
    }

    fn open_row(&mut self, key: ItemKey) -> Option<String> {
        let item = entries(&self.path)
            .iter()
            .find(|item| entry_key(item) == key)
            .copied()?;
        if item.disabled {
            return None;
        }
        if item.opens {
            self.path.push(item.name.to_owned());
            self.restore_cursor(None);
            None
        } else {
            Some(format!("Opened {}", item.name))
        }
    }

    fn update_links(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<PageStatus>) {
        if self.options[0] == 1 {
            let crumb = self.crumb();
            for (level, id) in crumb.links() {
                let response = Button::new(id, "")
                    .focusability(Focusability::ClickOnly)
                    .update(cx);
                if response.activated() {
                    let label = self.path.get(level).cloned().unwrap_or_default();
                    self.last_crumb = format!("{label} (level {level})");
                    let status = self.go_to_level(level);
                    return (response.erase(), Some(PageStatus(status)));
                }
            }
        }
        let deep = Crumb::new(DEEP_TRAIL, deep_path());
        for (level, id) in deep.links() {
            let response = Button::new(id, "")
                .focusability(Focusability::ClickOnly)
                .update(cx);
            if response.activated() {
                self.last_crumb = format!("{} (deep level {level})", DEEP_SEGMENTS[level]);
                return (
                    response.erase(),
                    Some(PageStatus(format!("Selected deep trail level {level}"))),
                );
            }
        }
        (Response::ignored(), None)
    }

    fn update_options(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        for index in 0..OPTION_IDS.len() {
            let option_id = OPTION_IDS[index];
            let is_active =
                self.section() == 1 || self.select_states[index].is_open() || cx.is_open(option_id);
            if !is_active {
                continue;
            }
            let old_collapsed = self.options[2];
            let disabled = index == 3 && self.options[2] == 0;
            let update = Select::new(option_id).disabled(disabled).update_in_form(
                cx,
                &mut self.select_states[index],
                &mut self.options[index],
                OPTION_ITEMS[index],
                false,
            );
            response |= update.erase();
            if index == 2 && self.options[2] != old_collapsed {
                let _ = self.commit_collapse(self.options[2] == 1, cx);
            }
        }
        response
    }

    fn update_playground(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<PageStatus>) {
        let mut response = Response::ignored();
        let mut status = None;

        let content_len = cx
            .layout(PLAY_SCROLL)
            .map_or(0, |layout| layout.content_len);
        response |= ScrollRegion::new(PLAY_SCROLL).update(cx, &mut self.play_scroll, content_len);

        // Breadcrumb activation can change `path` and restore a cursor in the
        // destination directory. Resolve it before projecting or updating the
        // native Explorer so both use the same dataset and keyed cursor.
        let links = self.update_links(cx);
        response |= links.0;
        status = links.1.or(status);

        let items = entries(&self.path);
        let list_target_visible = self.explorer_target_visible(cx, items);
        let has_list_pointer_or_wheel = cx
            .intents(LIST)
            .any(|intent| matches!(intent, Intent::Pointer { .. } | Intent::Wheel { .. }));
        let has_list_pointer_action = cx.intents(LIST).any(|intent| {
            matches!(
                intent,
                Intent::Pointer {
                    phase: Phase::Press | Phase::Click | Phase::DoubleClick,
                    part: PartRef {
                        part: Part::ROW,
                        item: Some(_),
                    },
                    ..
                }
            )
        });
        let has_list_binding = cx
            .intents(LIST)
            .any(|intent| matches!(intent, Intent::Binding(_)));
        self.reconcile_scroll_focus(cx, list_target_visible || has_list_pointer_action);

        let collapse = if self.collapsed {
            if self.options[3] == 1 {
                Some(
                    Button::new(EXPAND, "")
                        .focusability(Focusability::ClickOnly)
                        .update(cx),
                )
            } else {
                None
            }
        } else {
            Some(
                Button::new(COLLAPSE, "")
                    .focusability(Focusability::ClickOnly)
                    .update(cx),
            )
        };
        if let Some(collapse) = collapse {
            if collapse.activated() {
                let _ = self.commit_collapse(!self.collapsed, cx);
                status = Some(PageStatus(self.last_rail.clone()));
            }
            response |= collapse.erase();
        }

        let has_backspace = cx
            .intents(LIST)
            .any(|intent| matches!(intent, Intent::Binding(action) if action == BACKSPACE_ACTION));
        if has_backspace && list_target_visible {
            if let Some(message) = self.go_up() {
                response |= Response::changed().for_id(LIST);
                status = Some(PageStatus(message));
            } else {
                response |= Response::consumed().for_id(LIST);
            }
            return (response, status);
        }

        if list_target_visible || (has_list_pointer_or_wheel && !has_list_binding) {
            let list = explorer().update(cx, &mut self.list_state, items);
            if let Some(NavListAction::Chose(key) | NavListAction::EnterContent(key)) =
                list.action_ref()
                && let Some(message) = self.open_row(*key)
            {
                status = Some(PageStatus(message));
            }
            response |= list.erase();
        }

        (response, status)
    }

    fn location_panel(&self) -> Panel<'static> {
        let meta = if self.options[0] == 1 {
            "clickable"
        } else {
            "clickable(false)"
        };
        Panel::new(LOCATION_PANEL)
            .kind(PanelKind::Card)
            .title("Location")
            .meta(meta)
    }

    fn rail_panel(&self) -> Panel<'static> {
        Panel::new(RAIL_PANEL)
            .kind(PanelKind::Card)
            .title("SidebarRail")
    }

    fn trails_panel(&self) -> Panel<'static> {
        Panel::new(TRAILS_PANEL)
            .kind(PanelKind::Card)
            .title("Trails")
    }

    fn location_panel_height(&self, ui: &Ui<'_>, width: u16) -> u16 {
        let panel = self.location_panel();
        let inner_width = panel.inner(ui, Rect::new(0, 0, width, u16::MAX)).width;
        let facts = [
            format!("Depth: {}", self.path.len()),
            format!("Last crumb: {}", self.last_crumb),
            format!("Last RailAction: {}", self.last_rail),
            "Keys: Enter opens · Backspace goes up · Ctrl+B explorer".to_owned(),
        ];
        let body_rows = 2_u16.saturating_add(
            facts
                .iter()
                .map(|fact| wrapped_rows(fact, inner_width))
                .fold(0_u16, u16::saturating_add),
        );
        panel
            .measure_with_body(ui, Constraints::loose(width, u16::MAX), |constraints| {
                Size::exact(constraints.max.0, body_rows)
            })
            .preferred
            .1
    }

    fn rail_panel_height(&self, ui: &Ui<'_>, width: u16) -> u16 {
        let panel = self.rail_panel();
        let inner_width = panel.inner(ui, Rect::new(0, 0, width, u16::MAX)).width;
        let collapsed_rail = if self.options[3] == 1 {
            "on: a collapsed sidebar leaves its rail"
        } else {
            "off: only the keyboard brings it back"
        };
        let facts = [
            "RailAction::Collapse: ‹ in the frame, or Ctrl+B",
            "RailAction::Expand: a click on the › rail, or Ctrl+B",
            collapsed_rail,
        ];
        let body_rows = facts
            .iter()
            .map(|fact| wrapped_rows(fact, inner_width))
            .fold(0_u16, u16::saturating_add);
        panel
            .measure_with_body(ui, Constraints::loose(width, u16::MAX), |constraints| {
                Size::exact(constraints.max.0, body_rows.max(1))
            })
            .preferred
            .1
    }

    fn trails_panel_height(&self, ui: &Ui<'_>, width: u16) -> u16 {
        self.trails_panel()
            .measure_with_body(ui, Constraints::loose(width, u16::MAX), |constraints| {
                Size::exact(constraints.max.0, 4)
            })
            .preferred
            .1
    }

    fn playground_columns(&self, width: u16) -> (u16, u16, u16) {
        let collapsed_rail = self.collapsed && self.options[3] == 1;
        let explorer_width = if self.collapsed {
            u16::from(collapsed_rail)
        } else if width > 0 {
            EXPLORER_WIDTH.min((width / 2).max(1))
        } else {
            0
        };
        let gap = if self.collapsed {
            if collapsed_rail { 2 } else { 1 }
        } else {
            2
        };
        let right_width = width.saturating_sub(explorer_width).saturating_sub(gap);
        (explorer_width, gap, right_width)
    }

    fn explorer_panel_height(&self, ui: &Ui<'_>, width: u16) -> u16 {
        let items = entries(&self.path);
        let meta = format!("{} items", items.len());
        Panel::new(EXPLORER_PANEL)
            .kind(PanelKind::Framed)
            .title("Explorer")
            .meta(&meta)
            .measure_with_body(ui, Constraints::loose(width, u16::MAX), |constraints| {
                Size::exact(constraints.max.0, ENTRY_ROWS)
            })
            .preferred
            .1
    }

    fn top_block_height(&self, ui: &Ui<'_>, width: u16) -> u16 {
        let (explorer_width, _, right_width) = self.playground_columns(width);
        let explorer = self.explorer_panel_height(ui, explorer_width);
        let location = self.location_panel_height(ui, right_width);
        let rail = self.rail_panel_height(ui, right_width);
        let details = location.saturating_add(1).saturating_add(rail);
        explorer.max(details)
    }

    fn playground_content_height(&self, ui: &Ui<'_>, width: u16) -> usize {
        let top = usize::from(self.top_block_height(ui, width));
        let trails = usize::from(self.trails_panel_height(ui, width));
        top.saturating_add(1).saturating_add(trails)
    }

    fn playground_content_len(&self, ui: &Ui<'_>, area: Rect) -> usize {
        let content = self.playground_content_height(ui, area.width);
        let mut projected = self.play_scroll;
        projected.apply_layout(usize::from(area.height), content);
        if projected.overflows() {
            self.playground_content_height(ui, area.width.saturating_sub(1))
        } else {
            content
        }
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let content_len = self.playground_content_len(ui, area);
        let view = ScrollRegion::new(PLAY_SCROLL).draw(ui, area, &self.play_scroll, content_len);
        let Some(content_window) = scroll_window(view, content_len, &self.play_scroll) else {
            return;
        };
        ui.with_area(view, |ui| {
            let top_height = self.top_block_height(ui, view.width);
            let trails_height = self.trails_panel_height(ui, view.width);
            let (explorer_width, gap, right_width) = self.playground_columns(view.width);
            let right_x = view.x.saturating_add(explorer_width).saturating_add(gap);
            let right_start =
                usize::from(self.location_panel_height(ui, right_width)).saturating_add(1);
            let trails_start = usize::from(top_height).saturating_add(1);

            if !self.collapsed {
                if let Some((panel_area, window)) =
                    child_window(content_window, 0, top_height, view.x, explorer_width)
                {
                    self.draw_explorer_window(ui, panel_area, window);
                }
            } else if self.options[3] == 1
                && let Some((rail_area, window)) =
                    child_window(content_window, 0, top_height, view.x, explorer_width)
            {
                draw_rail_window(ui, EXPAND, rail_area, window);
            }

            if let Some((panel_area, window)) = child_window(
                content_window,
                0,
                self.location_panel_height(ui, right_width),
                right_x,
                right_width,
            ) {
                self.draw_location_window(ui, panel_area, window);
            }
            if let Some((panel_area, window)) = child_window(
                content_window,
                right_start,
                self.rail_panel_height(ui, right_width),
                right_x,
                right_width,
            ) {
                self.draw_rail_facts_window(ui, panel_area, window);
            }
            if let Some((panel_area, window)) = child_window(
                content_window,
                trails_start,
                trails_height,
                view.x,
                view.width,
            ) {
                self.draw_trails_window(ui, panel_area, window);
            }
        });
    }

    fn draw_explorer_window(&self, ui: &mut Ui<'_>, area: Rect, window: VerticalWindow) {
        let items = entries(&self.path);
        let meta = format!("{} items", items.len());
        let panel = Panel::new(EXPLORER_PANEL)
            .kind(PanelKind::Framed)
            .title("Explorer")
            .meta(&meta)
            .focused(ui.state(LIST).contains(StateFlags::FOCUSED));
        let _ = panel.draw_window(ui, area, window, |ui, body_window| {
            let Some(body_window) = body_window else {
                return;
            };
            let visible = body_window.visible_body();
            let first_row = u16::try_from(body_window.first_source_row()).unwrap_or(u16::MAX);
            let logical_rows = u16::try_from(body_window.logical_rows())
                .unwrap_or(u16::MAX)
                .min(ENTRY_ROWS);
            let inner = Rect::new(
                visible.x,
                visible.y.saturating_sub(first_row),
                visible.width,
                logical_rows,
            );
            if body_window.first_source_row() < usize::from(logical_rows) {
                let _ = explorer().draw(ui, inner, &self.list_state, items);
                let list_state = ui.state(LIST);
                ui.publish_additional_bindings(LIST, list_state, BACKSPACE_BINDINGS);
                if items.is_empty() {
                    draw_wrapped(ui, inner, "Empty folder");
                }
            }
        });
        if area.width >= 12 && area.height >= 2 {
            if let Some(y) = window.physical_row_for_source(0) {
                let control = Rect::new(area.right().saturating_sub(5), y, 3.min(area.width), 1);
                let glyph = ui.glyph_str(GlyphRole::OverflowLeft);
                draw_control_glyph(ui, COLLAPSE, control, glyph);
            }
        }
    }

    fn draw_location_window(&self, ui: &mut Ui<'_>, area: Rect, window: VerticalWindow) {
        let panel = self.location_panel();
        let _ = panel.draw_window(ui, area, window, |ui, body_window| {
            let Some(body_window) = body_window else {
                return;
            };
            let body = body_window.visible_body();
            if let Some(y) = body_window.physical_row_for_source(0) {
                draw_trail(
                    ui,
                    Rect::new(body.x, y, body.width, 1),
                    &self.crumb(),
                    self.options[0] == 1,
                );
            }
            let facts = [
                format!("Depth: {}", self.path.len()),
                format!("Last crumb: {}", self.last_crumb),
                format!("Last RailAction: {}", self.last_rail),
                "Keys: Enter opens · Backspace goes up · Ctrl+B explorer".to_owned(),
            ];
            let mut source_row: usize = 2;
            for fact in facts {
                source_row = source_row.saturating_add(draw_wrapped_window(
                    ui,
                    body_window,
                    source_row,
                    &fact,
                ));
            }
        });
    }

    fn draw_rail_facts_window(&self, ui: &mut Ui<'_>, area: Rect, window: VerticalWindow) {
        let collapsed_rail = if self.options[3] == 1 {
            "on: a collapsed sidebar leaves its rail"
        } else {
            "off: only the keyboard brings it back"
        };
        let panel = self.rail_panel();
        let _ = panel.draw_window(ui, area, window, |ui, body_window| {
            let Some(body_window) = body_window else {
                return;
            };
            let facts = [
                "RailAction::Collapse: ‹ in the frame, or Ctrl+B",
                "RailAction::Expand: a click on the › rail, or Ctrl+B",
                collapsed_rail,
            ];
            let mut source_row: usize = 0;
            for fact in facts {
                source_row = source_row.saturating_add(draw_wrapped_window(
                    ui,
                    body_window,
                    source_row,
                    fact,
                ));
            }
        });
    }

    fn draw_trails_window(&self, ui: &mut Ui<'_>, area: Rect, window: VerticalWindow) {
        let panel = self.trails_panel();
        let _ = panel.draw_window(ui, area, window, |ui, body_window| {
            let Some(body_window) = body_window else {
                return;
            };
            let inner = body_window.visible_body();
            let deep_width = match self.options[1] {
                0 => inner.width,
                index => OPTION_ITEMS[1]
                    .get(index)
                    .and_then(|value| value.parse::<u16>().ok())
                    .unwrap_or(inner.width)
                    .min(inner.width),
            };
            let width_label = if self.options[1] == 0 {
                "full width".to_owned()
            } else {
                format!("width {deep_width}")
            };

            if let Some(y) = body_window.physical_row_for_source(0) {
                ui.paint_str(
                    Rect::new(inner.x, y, inner.width, 1),
                    "clickable(false) · full width",
                    ui.surface_style(),
                );
            }
            if let Some(y) = body_window.physical_row_for_source(1) {
                draw_trail(
                    ui,
                    Rect::new(inner.x, y, inner.width, 1),
                    &Crumb::new(PLAIN_TRAIL, self.path.clone()),
                    false,
                );
            }
            if let Some(y) = body_window.physical_row_for_source(2) {
                let label = format!("deep, cut to fit · {width_label}");
                ui.paint_str(
                    Rect::new(inner.x, y, inner.width, 1),
                    &label,
                    ui.surface_style(),
                );
            }
            if let Some(y) = body_window.physical_row_for_source(3) {
                draw_trail(
                    ui,
                    Rect::new(inner.x, y, deep_width, 1),
                    &Crumb::new(DEEP_TRAIL, deep_path()),
                    true,
                );
            }
        });
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let row_height = area.height.saturating_div(OPTION_IDS.len() as u16).max(1);
        for index in 0..OPTION_IDS.len() {
            let y = area
                .y
                .saturating_add((index as u16).saturating_mul(row_height));
            if y >= area.bottom() {
                break;
            }
            let label_area = Rect::new(area.x, y, area.width, 1);
            ui.paint_str(label_area, OPTION_LABELS[index], ui.surface_style());
            let select_y = y.saturating_add(1);
            if select_y >= area.bottom() {
                continue;
            }
            let select_area = Rect::new(
                area.x,
                select_y,
                area.width,
                area.bottom()
                    .saturating_sub(select_y)
                    .min(row_height.saturating_sub(1)),
            );
            let disabled = index == 3 && self.options[2] == 0;
            let _ = Select::new(OPTION_IDS[index]).disabled(disabled).draw(
                ui,
                select_area,
                &self.select_states[index],
                OPTION_ITEMS[index],
            );
        }
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        const STATE_NAMES: [&str; 5] = ["Rest", "Hover", "Pressed", "Overflow", "Narrow"];
        let label_width = width("Overflow").saturating_add(2).min(area.width / 3);
        let remaining = area.width.saturating_sub(label_width);
        let gap = u16::from(remaining >= 5);
        let column_width = remaining
            .saturating_sub(gap.saturating_mul(2))
            .saturating_div(3)
            .max(1);
        let starts = [
            area.x.saturating_add(label_width),
            area.x
                .saturating_add(label_width)
                .saturating_add(column_width)
                .saturating_add(gap),
            area.x
                .saturating_add(label_width)
                .saturating_add(column_width.saturating_add(gap).saturating_mul(2)),
        ];
        for (column, title) in ["Clickable", "Plain text", "Rail"].iter().enumerate() {
            if area.y < area.bottom() {
                ui.paint_str(
                    Rect::new(
                        starts[column],
                        area.y,
                        column_width.min(area.right().saturating_sub(starts[column])),
                        1,
                    ),
                    title,
                    ui.surface_style(),
                );
            }
        }
        for (row_index, name) in STATE_NAMES.iter().enumerate() {
            let y = area.y.saturating_add(1).saturating_add(row_index as u16);
            if y >= area.bottom() {
                break;
            }
            ui.paint_str(
                Rect::new(area.x, y, label_width, 1),
                name,
                ui.surface_style(),
            );
            let path = if row_index >= 3 {
                deep_path()
            } else {
                ["Disk", "Users", "alex", "Projects"]
                    .iter()
                    .map(|segment| (*segment).to_owned())
                    .collect()
            };
            let clickable_id = STATE_ROOT.index(row_index).index(0);
            let clickable_crumb = Crumb::new(clickable_id, path.clone());
            let specimen_width = match row_index {
                3 => column_width.min(34),
                4 => column_width.min(12),
                _ => column_width,
            };
            let clickable_area = Rect::new(
                starts[0],
                y,
                specimen_width.min(area.right().saturating_sub(starts[0])),
                1,
            );
            let clickable_layout = trail_layout(ui, clickable_area, &clickable_crumb);
            let target_id = clickable_layout
                .pieces
                .iter()
                .find(|piece| piece.index + 1 < clickable_crumb.segments.len())
                .map(|piece| clickable_crumb.segment_id(piece.index));
            let state = match row_index {
                1 => Some(ReferenceState::HOVERED),
                2 => Some(ReferenceState::PRESSED),
                _ => None,
            };
            let target = target_id
                .zip(state)
                .map(|(id, state)| ReferenceTarget::new(id, state));
            ui.reference(target, |ui| {
                draw_trail(ui, clickable_area, &clickable_crumb, true);
            });

            let plain_id = STATE_ROOT.index(row_index).index(1);
            let plain_crumb = Crumb::new(plain_id, path);
            let plain_area = Rect::new(
                starts[1],
                y,
                specimen_width.min(area.right().saturating_sub(starts[1])),
                1,
            );
            ui.reference(None, |ui| draw_trail(ui, plain_area, &plain_crumb, false));

            let rail_id = STATE_ROOT.index(row_index).index(2);
            let rail_area = Rect::new(
                starts[2],
                y,
                1.min(area.right().saturating_sub(starts[2])),
                1,
            );
            let rail_target = state.map(|state| ReferenceTarget::new(rail_id, state));
            ui.reference(rail_target, |ui| draw_rail(ui, rail_id, rail_area));
        }
    }
}

impl Default for BreadcrumbsRailsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for BreadcrumbsRailsPage {
    fn title(&self) -> &'static str {
        "Breadcrumbs & rails"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Tabs::new(PAGE_TABS)
            .update(cx, &mut self.page_tabs, SECTIONS)
            .erase();
        self.apply_pending_focus_transition(cx);
        let section = self.section();
        if section != 0 {
            self.scroll_focus_transition = false;
        }
        let mut status = None;
        if section == 0 {
            let (playground, playground_status) = self.update_playground(cx);
            response |= playground;
            status = playground_status.or(status);
        }
        response |= self.update_options(cx);
        PageUpdate { response, status }
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        OPTION_KEYS
            .iter()
            .position(|candidate| *candidate == key)
            .map(|index| self.options[index])
    }

    fn set_option(&mut self, key: &str, value: usize) {
        let Some(index) = OPTION_KEYS.iter().position(|candidate| *candidate == key) else {
            return;
        };
        let next = value.min(OPTION_ITEMS[index].len().saturating_sub(1));
        let changed = self.options[index] != next;
        self.options[index] = next;
        if index == 2 && changed {
            self.collapsed = next == 1;
            self.last_rail = if self.collapsed { "Collapse" } else { "Expand" }.to_owned();
            self.pending_focus_transition = true;
        }
    }

    fn command(&mut self, cx: &mut Cx<'_>, action: ActionKey) -> Response<()> {
        let viewport = cx.viewport();
        if action != TOGGLE_RAIL_ACTION
            || self.section() != 0
            || cx.update_cause() != UpdateCause::Event
            || cx.top_layer() != termrock::LayerId::PAGE
            || viewport.width < 72
            || viewport.height < 20
        {
            return Response::ignored();
        }
        if self.commit_collapse(!self.collapsed, cx) {
            Response::changed()
        } else {
            Response::consumed()
        }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), BLURB, |ui, body| {
            let tabs_area = Rect {
                height: 1.min(body.height),
                ..body
            };
            let content = Rect {
                y: tabs_area.bottom(),
                height: body.bottom().saturating_sub(tabs_area.bottom()),
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
            return SECTION_HINTS;
        }
        if self.section() == 1
            && OPTION_IDS
                .iter()
                .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            return OPTION_HINTS;
        }
        if ui.state(LIST).contains(StateFlags::FOCUSED) {
            return LIST_HINTS;
        }
        if self.collapsed {
            return COLLAPSED_HINTS;
        }
        &[]
    }
}
