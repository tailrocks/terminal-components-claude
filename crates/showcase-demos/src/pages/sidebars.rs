use termrock::{
    Button, Color, Cx, Id, ItemKey, Modifier, NavList, NavListAction, NavListState, NavMode,
    PaintStyle, Rect, RowUi, Ui, Variant, width, wrap,
};

use super::{Page, PageUpdate, frame};

const NAV: Id = termrock::Id::root("showcase_ui::pages::sidebars::sidebars.nav");
const COLLAPSE: Id = termrock::Id::root("showcase_ui::pages::sidebars::sidebars.collapse");
const CONTENT: Id = termrock::Id::root("showcase_ui::pages::sidebars::sidebars.content");

const BLURB: &str =
    "Sections, current item, focus cursor, hover, collapsed mode; text first, no icons";
const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const SIDE_W: u16 = 28;
const CARD_GAP: u16 = 2;
const CARD_H: u16 = 20;
const CARD_MIN_W: u16 = 24;
const CONTENT_MIN_W: u16 = SIDE_W + CARD_GAP + CARD_MIN_W;
const SCROLL_ROOM: u16 = 2;
const NAV_ROWS: u16 = 16;

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const ACCENT: u32 = 0x48e054;
const ACCENT_BG: u32 = 0x173f1c;
const TAB_ACTIVE: u32 = 0x27272a;
const DISABLED: u32 = 0x4d4d4d;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SidebarItem {
    key: u8,
    label: &'static str,
    icon: &'static str,
    section: &'static str,
    badge: Option<&'static str>,
    disabled: bool,
}

const ITEMS: &[SidebarItem] = &[
    SidebarItem {
        key: 1,
        label: "Tasks",
        icon: "T",
        section: "Workspace",
        badge: Some("3"),
        disabled: false,
    },
    SidebarItem {
        key: 2,
        label: "Runs",
        icon: "R",
        section: "Workspace",
        badge: None,
        disabled: false,
    },
    SidebarItem {
        key: 3,
        label: "Branches",
        icon: "B",
        section: "Workspace",
        badge: None,
        disabled: false,
    },
    SidebarItem {
        key: 4,
        label: "Members",
        icon: "M",
        section: "Project",
        badge: None,
        disabled: false,
    },
    SidebarItem {
        key: 5,
        label: "Environment",
        icon: "E",
        section: "Project",
        badge: None,
        disabled: false,
    },
    SidebarItem {
        key: 6,
        label: "Billing",
        icon: "$",
        section: "Project",
        badge: None,
        disabled: true,
    },
    SidebarItem {
        key: 7,
        label: "Keyboard",
        icon: "K",
        section: "Preferences",
        badge: None,
        disabled: false,
    },
    SidebarItem {
        key: 8,
        label: "Appearance",
        icon: "A",
        section: "Preferences",
        badge: None,
        disabled: false,
    },
];

#[derive(Clone, Copy)]
enum NavSlot {
    Gap,
    Head(&'static str),
    Row(usize),
}

const NAV_SLOTS: [NavSlot; 16] = [
    NavSlot::Head("Workspace"),
    NavSlot::Row(0),
    NavSlot::Row(1),
    NavSlot::Row(2),
    NavSlot::Gap,
    NavSlot::Head("Project"),
    NavSlot::Row(3),
    NavSlot::Row(4),
    NavSlot::Row(5),
    NavSlot::Gap,
    NavSlot::Head("Preferences"),
    NavSlot::Row(6),
    NavSlot::Row(7),
    NavSlot::Gap,
    NavSlot::Gap,
    NavSlot::Gap,
];

fn collapse_button(collapsed: bool) -> Button<'static> {
    Button::new(COLLAPSE, if collapsed { "\u{203a}" } else { "Collapse" })
        .variant(Variant::SECONDARY)
}

fn item_key(item: &SidebarItem) -> ItemKey {
    ItemKey::num(u64::from(item.key))
}

fn item_section(item: &SidebarItem) -> &str {
    item.section
}

fn item_icon(item: &SidebarItem) -> &str {
    item.icon
}

fn item_badge(item: &SidebarItem) -> Option<&str> {
    item.badge
}

fn item_disabled(item: &SidebarItem) -> bool {
    item.disabled
}

fn item_row(item: &SidebarItem, row: &mut RowUi<'_>) {
    row.label(item.label);
}

fn sidebar(
    collapsed: bool,
) -> NavList<
    'static,
    SidebarItem,
    impl Fn(&SidebarItem) -> ItemKey,
    impl Fn(&SidebarItem, &mut RowUi<'_>),
> {
    NavList::new(NAV)
        .key(item_key)
        .section(&item_section)
        .icon(&item_icon)
        .badge(&item_badge)
        .disabled_item(&item_disabled)
        .mode(if collapsed {
            NavMode::Collapsed
        } else {
            NavMode::Full
        })
        .row(item_row)
}

fn rgb(hex: u32) -> Color {
    Color::Rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

fn ink(fg: u32, bg: u32) -> PaintStyle {
    PaintStyle::new().fg(rgb(fg)).bg(rgb(bg))
}

fn tab_width(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn paint_blurb(ui: &mut Ui<'_>, body: Rect) {
    let title_w = width("Sidebars");
    if body.width <= title_w.saturating_add(4) {
        return;
    }
    let meta = Rect::new(
        body.x.saturating_add(title_w).saturating_add(2),
        body.y.saturating_sub(2),
        body.width.saturating_sub(title_w).saturating_sub(3),
        1,
    );
    let style = ink(MUTED, CANVAS);
    if width(BLURB) <= meta.width {
        ui.paint_str(meta, BLURB, style);
        return;
    }
    if meta.width == 0 {
        return;
    }
    let budget = Rect {
        width: meta.width.saturating_sub(1),
        ..meta
    };
    let used = ui.paint_str(budget, BLURB, style);
    ui.paint_str(
        Rect::new(meta.x.saturating_add(used), meta.y, 1, 1),
        "…",
        style,
    );
}

fn paint_section_strip(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let mut x = area.x;
    let mut tab_end = area.x;
    for (index, label) in SECTIONS.iter().enumerate() {
        let w = tab_width(label);
        if x.saturating_add(w) > area.right() {
            break;
        }
        let style = if index == 0 {
            ink(WHITE, TAB_ACTIVE).add_modifier(Modifier::BOLD)
        } else {
            ink(SECONDARY, CANVAS)
        };
        ui.fill(Rect::new(x, area.y, w, 1), style);
        ui.paint_str(
            Rect::new(x.saturating_add(1), area.y, w.saturating_sub(1), 1),
            label,
            style,
        );
        tab_end = x.saturating_add(w);
        x = tab_end.saturating_add(1);
    }
    if area.width >= SECTION_WIDE {
        let caption_width = width(SECTION_CAPTION);
        let caption_x = area.right().saturating_sub(caption_width.saturating_add(1));
        if caption_x > tab_end.saturating_add(2) {
            ui.paint_str(
                Rect::new(caption_x, area.y, caption_width, 1),
                SECTION_CAPTION,
                ink(FAINT, CANVAS),
            );
        }
    }
    if area.height < 2 {
        return;
    }
    let rule_y = area.y.saturating_add(1);
    ui.paint_str(
        Rect {
            y: rule_y,
            height: 1,
            ..area
        },
        &"─".repeat(usize::from(area.width)),
        ink(RULE, CANVAS),
    );
    let active = tab_width(SECTIONS[0]).min(area.width);
    ui.paint_str(
        Rect::new(area.x, rule_y, active, 1),
        &"━".repeat(usize::from(active)),
        ink(ACCENT, CANVAS),
    );
}

fn paint_side(ui: &mut Ui<'_>, side: Rect, selected: &str, state: &NavListState, collapsed: bool) {
    if side.is_empty() {
        return;
    }
    let inner = Rect {
        y: side.y.saturating_add(1),
        height: side.height.saturating_sub(2),
        ..side
    };
    sidebar(collapsed).draw(
        ui,
        Rect {
            height: inner.height.saturating_sub(2),
            ..inner
        },
        state,
        ITEMS,
    );
    ui.fill(side, ink(WHITE, SURFACE));
    if side.width >= 12 {
        let rail_x = side.right().saturating_sub(5);
        ui.paint_str(Rect::new(rail_x, side.y, 1, 1), " ", ink(RULE, SURFACE));
        ui.paint_str(
            Rect::new(rail_x.saturating_add(1), side.y, 1, 1),
            "\u{2039}",
            ink(SECONDARY, SURFACE),
        );
        ui.paint_str(
            Rect::new(rail_x.saturating_add(2), side.y, 1, 1),
            " ",
            ink(RULE, SURFACE),
        );
    }
    let row_w = side.width.saturating_sub(1);
    for (index, slot) in NAV_SLOTS.iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        let y = side.y.saturating_add(1).saturating_add(offset);
        if y.saturating_add(1) >= side.bottom() || offset >= NAV_ROWS {
            break;
        }
        let row = Rect::new(side.x, y, row_w, 1);
        match slot {
            NavSlot::Gap => {}
            NavSlot::Head(name) => {
                if row.width > 3 {
                    ui.paint_str(
                        Rect::new(
                            row.x.saturating_add(3),
                            row.y,
                            row.width.saturating_sub(3),
                            1,
                        ),
                        name,
                        ink(FAINT, SURFACE),
                    );
                }
            }
            NavSlot::Row(item_index) => {
                if let Some(item) = ITEMS.get(*item_index) {
                    paint_nav_item(ui, row, item, item.label == selected);
                }
            }
        }
        if side.width > 0 {
            ui.paint_str(
                Rect::new(side.right().saturating_sub(1), y, 1, 1),
                "│",
                ink(RULE, SURFACE),
            );
        }
    }
    paint_collapse(ui, side, collapsed);
}

fn paint_nav_item(ui: &mut Ui<'_>, row: Rect, item: &SidebarItem, selected: bool) {
    if row.is_empty() {
        return;
    }
    if selected && !item.disabled {
        let style = ink(WHITE, ACCENT_BG).add_modifier(Modifier::BOLD);
        ui.fill(row, style);
        ui.fill(
            Rect::new(row.x, row.y, 1, 1),
            ink(ACCENT_BG, ACCENT_BG).add_modifier(Modifier::BOLD),
        );
        if row.width > 1 {
            ui.paint_str(
                Rect::new(row.x.saturating_add(1), row.y, 1, 1),
                "\u{203a}",
                ink(ACCENT, ACCENT_BG).add_modifier(Modifier::BOLD),
            );
        }
        if row.width > 3 {
            ui.paint_str(
                Rect::new(row.x.saturating_add(3), row.y, 1, 1),
                item.icon,
                ink(MUTED, ACCENT_BG).add_modifier(Modifier::BOLD),
            );
        }
        if row.width > 5 {
            ui.paint_str(
                Rect::new(
                    row.x.saturating_add(5),
                    row.y,
                    row.width.saturating_sub(5),
                    1,
                ),
                item.label,
                style,
            );
        }
        if let Some(badge) = item.badge {
            let badge_w = width(badge);
            if row.width > badge_w.saturating_add(1) {
                ui.paint_str(
                    Rect::new(
                        row.right().saturating_sub(1).saturating_sub(badge_w),
                        row.y,
                        badge_w,
                        1,
                    ),
                    badge,
                    ink(ACCENT, ACCENT_BG).add_modifier(Modifier::BOLD),
                );
            }
        }
        return;
    }
    ui.fill(Rect::new(row.x, row.y, 1, 1), ink(SURFACE, SURFACE));
    let style = if item.disabled {
        let body = Rect {
            x: row.x.saturating_add(1),
            width: row.width.saturating_sub(1),
            ..row
        };
        ui.fill(body, ink(DISABLED, SURFACE));
        ink(DISABLED, SURFACE)
    } else {
        ink(SECONDARY, SURFACE)
    };
    if row.width > 3 {
        let icon = if item.disabled {
            style
        } else {
            ink(MUTED, SURFACE)
        };
        ui.paint_str(
            Rect::new(row.x.saturating_add(3), row.y, 1, 1),
            item.icon,
            icon,
        );
    }
    if row.width > 5 {
        ui.paint_str(
            Rect::new(
                row.x.saturating_add(5),
                row.y,
                row.width.saturating_sub(5),
                1,
            ),
            item.label,
            style,
        );
    }
}

fn paint_collapse(ui: &mut Ui<'_>, side: Rect, collapsed: bool) {
    let label = if collapsed { "\u{203a}" } else { "Collapse" };
    let columns = width(label).saturating_add(2);
    if side.height <= 18 || side.width <= 2 {
        return;
    }
    let area = Rect::new(
        side.x.saturating_add(1),
        side.y.saturating_add(18),
        columns.min(side.width.saturating_sub(1)),
        1,
    );
    collapse_button(collapsed).draw(ui, area);
    let text = ink(WHITE, TAB_ACTIVE);
    ui.fill(area, text);
    ui.fill(Rect::new(area.x, area.y, 1, 1), ink(TAB_ACTIVE, TAB_ACTIVE));
    if area.width > 1 {
        ui.paint_str(
            Rect::new(
                area.x.saturating_add(1),
                area.y,
                area.width.saturating_sub(1),
                1,
            ),
            label,
            text,
        );
    }
}

fn content_lines(last: &str) -> [String; 11] {
    [
        "One focus stop. ↑↓ (or j k) move the cursor, Enter opens.".to_owned(),
        String::new(),
        "›  current item · persists when focus leaves".to_owned(),
        "▎  keyboard cursor · only while focused".to_owned(),
        "░  hover · follows the pointer".to_owned(),
        String::new(),
        "Disabled items are skipped and ignore the pointer.".to_owned(),
        "Collapse keeps rows and markers, initials only.".to_owned(),
        "‹ on the frame or Ctrl+B hides the sidebar behind its rail.".to_owned(),
        String::new(),
        format!("Last: {last}"),
    ]
}

fn paint_content(ui: &mut Ui<'_>, card: Rect, selected: &str, last: &str) {
    if card.width < 4 || card.height == 0 {
        return;
    }
    ui.register_control(CONTENT, card, termrock::Focusability::Disabled);
    ui.fill(card, ink(WHITE, SURFACE));
    let text_x = card.x.saturating_add(2);
    let text_w = card.width.saturating_sub(4);
    ui.paint_str(
        Rect::new(text_x, card.y, text_w, 1),
        selected,
        ink(SECONDARY, SURFACE),
    );
    if card.height > 1 && text_w > 0 {
        ui.paint_str(
            Rect::new(text_x, card.y.saturating_add(1), text_w, 1),
            &"─".repeat(usize::from(text_w)),
            ink(RULE, SURFACE),
        );
    }
    let mut y = card.y.saturating_add(2);
    for (index, line) in content_lines(last).iter().enumerate() {
        let parts = if line.is_empty() || text_w == 0 {
            vec![String::new()]
        } else {
            wrap(line, text_w)
        };
        let accent = (2..=4).contains(&index);
        for (part_index, part) in parts.iter().enumerate() {
            if y >= card.bottom() {
                return;
            }
            if !part.is_empty() {
                if accent && part_index == 0 {
                    paint_accent_line(ui, text_x, y, text_w, part);
                } else {
                    ui.paint_str(
                        Rect::new(text_x, y, text_w, 1),
                        part,
                        ink(SECONDARY, SURFACE),
                    );
                }
            }
            y = y.saturating_add(1);
        }
    }
}

fn paint_accent_line(ui: &mut Ui<'_>, text_x: u16, y: u16, text_w: u16, line: &str) {
    let Some(first) = line.chars().next() else {
        return;
    };
    let split = first.len_utf8();
    let (glyph, rest) = line.split_at(split);
    let glyph_w = width(glyph);
    ui.paint_str(Rect::new(text_x, y, text_w, 1), glyph, ink(ACCENT, SURFACE));
    ui.paint_str(
        Rect::new(
            text_x.saturating_add(glyph_w),
            y,
            text_w.saturating_sub(glyph_w),
            1,
        ),
        rest,
        ink(SECONDARY, SURFACE),
    );
}

fn text_columns(content_w: u16) -> u16 {
    let side_w = SIDE_W.min(content_w);
    let content_x = side_w.saturating_add(CARD_GAP);
    if content_x >= content_w {
        return 0;
    }
    content_w.saturating_sub(content_x).saturating_sub(4)
}

fn wrapped_rows(text: &str, columns: u16) -> u16 {
    u16::try_from(wrap(text, columns.max(1)).len()).unwrap_or(u16::MAX)
}

fn content_height(content_w: u16, last: &str) -> u16 {
    let columns = text_columns(content_w);
    let body = if columns == 0 {
        0
    } else {
        content_lines(last)
            .iter()
            .map(|line| {
                if line.is_empty() {
                    1
                } else {
                    wrapped_rows(line, columns)
                }
            })
            .fold(0u16, u16::saturating_add)
    };
    let fitted = body.saturating_add(2);
    if fitted > CARD_H {
        // One resting row under the last wrapped line. The thumb uses it.
        fitted.saturating_add(1)
    } else {
        CARD_H
    }
}

fn playground_width(area: Rect, last: &str) -> u16 {
    let mut width = area.width.max(CONTENT_MIN_W);
    let height = content_height(width, last);
    if height > area.height && area.width > SCROLL_ROOM && width <= area.width {
        let narrow = area.width.saturating_sub(SCROLL_ROOM);
        let narrowed = narrow.max(CONTENT_MIN_W);
        if narrowed <= narrow {
            width = narrowed;
        }
    }
    width
}

struct ScrollSplit {
    view: Rect,
    vertical: bool,
    horizontal: bool,
}

fn scroll_split(area: Rect, content_w: u16, content_h: u16) -> ScrollSplit {
    let (mut vertical, mut horizontal) = (false, false);
    for _ in 0..3 {
        let view_w = area
            .width
            .saturating_sub(if vertical { SCROLL_ROOM } else { 0 });
        let view_h = area.height.saturating_sub(u16::from(horizontal));
        let next_v = content_h > view_h;
        let next_h = content_w > view_w;
        if (next_v, next_h) == (vertical, horizontal) {
            break;
        }
        vertical = next_v;
        horizontal = next_h;
    }
    let vertical = vertical && area.width > SCROLL_ROOM;
    let horizontal = horizontal && area.height > 1;
    let view = Rect {
        width: area
            .width
            .saturating_sub(if vertical { SCROLL_ROOM } else { 0 }),
        height: area.height.saturating_sub(u16::from(horizontal)),
        ..area
    };
    ScrollSplit {
        view,
        vertical,
        horizontal,
    }
}

fn track_thumb(viewport: u16, track: u16, content: u16) -> u16 {
    if content <= viewport || track == 0 {
        return track;
    }
    let len = u32::from(viewport).saturating_mul(u32::from(track)) / u32::from(content);
    u16::try_from(len).unwrap_or(u16::MAX).max(1).min(track)
}

fn paint_page_scrollbars(
    ui: &mut Ui<'_>,
    area: Rect,
    split: &ScrollSplit,
    content_w: u16,
    content_h: u16,
) {
    if split.vertical {
        let bar = Rect::new(area.right().saturating_sub(1), area.y, 1, split.view.height);
        let thumb = track_thumb(split.view.height, bar.height, content_h);
        for row in 0..bar.height {
            let on = row < thumb;
            ui.paint_str(
                Rect::new(bar.x, bar.y.saturating_add(row), 1, 1),
                if on { "┃" } else { "┆" },
                if on {
                    ink(THUMB, CANVAS)
                } else {
                    ink(TRACK, CANVAS)
                },
            );
        }
    }
    if split.horizontal {
        let width = area.width.saturating_sub(u16::from(split.vertical));
        let bar = Rect::new(area.x, area.bottom().saturating_sub(1), width, 1);
        let thumb = track_thumb(split.view.width, bar.width, content_w);
        for col in 0..bar.width {
            let on = col < thumb;
            ui.paint_str(
                Rect::new(bar.x.saturating_add(col), bar.y, 1, 1),
                if on { "━" } else { "┄" },
                if on {
                    ink(THUMB, CANVAS)
                } else {
                    ink(TRACK, CANVAS)
                },
            );
        }
    }
}

fn paint_playground_body(
    ui: &mut Ui<'_>,
    area: Rect,
    selected: &str,
    last: &str,
    state: &NavListState,
    collapsed: bool,
) {
    if area.is_empty() {
        return;
    }
    let side_w = SIDE_W.min(area.width);
    paint_side(
        ui,
        Rect::new(area.x, area.y, side_w, area.height),
        selected,
        state,
        collapsed,
    );
    let content_x = area.x.saturating_add(side_w).saturating_add(CARD_GAP);
    if content_x >= area.right() {
        return;
    }
    paint_content(
        ui,
        Rect::new(
            content_x,
            area.y,
            area.right().saturating_sub(content_x),
            area.height,
        ),
        selected,
        last,
    );
}

fn paint_playground(
    ui: &mut Ui<'_>,
    area: Rect,
    selected: &str,
    last: &str,
    state: &NavListState,
    collapsed: bool,
) {
    if area.is_empty() {
        return;
    }
    let content_w = playground_width(area, last);
    let content_h = content_height(content_w, last);
    let split = scroll_split(area, content_w, content_h);
    let content = Rect::new(area.x, area.y, content_w, content_h);
    ui.with_area(split.view, |ui| {
        paint_playground_body(ui, content, selected, last, state, collapsed);
    });
    paint_page_scrollbars(ui, area, &split, content_w, content_h);
}

#[derive(Debug, Default)]
pub struct SidebarsPage {
    state: NavListState,
    selected: &'static str,
    collapsed: bool,
    last: String,
}

impl SidebarsPage {
    pub fn new() -> Self {
        Self {
            state: NavListState::default(),
            selected: "Tasks",
            collapsed: false,
            last: "none yet".to_owned(),
        }
    }
}

impl Page for SidebarsPage {
    fn title(&self) -> &'static str {
        "Sidebars"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let result = sidebar(self.collapsed).update(cx, &mut self.state, ITEMS);
        if let Some(NavListAction::Chose(key) | NavListAction::EnterContent(key)) =
            result.action_ref()
            && let Some(item) = ITEMS.iter().find(|item| item_key(item) == *key)
        {
            self.selected = item.label;
            self.last = format!("NavAction::Activated({}) · {}", item.key, item.label);
        }
        let collapse = collapse_button(self.collapsed).update(cx);
        if collapse.activated() {
            self.collapsed = !self.collapsed;
        }
        let mut response = result.erase();
        response |= collapse.erase();
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "", |ui, body| {
            paint_blurb(ui, body);
            paint_section_strip(
                ui,
                Rect {
                    height: 2.min(body.height),
                    ..body
                },
            );
            let top = body.y.saturating_add(3);
            if top >= body.bottom() {
                return;
            }
            paint_playground(
                ui,
                Rect {
                    y: top,
                    height: body.bottom().saturating_sub(top),
                    ..body
                },
                self.selected,
                &self.last,
                &self.state,
                self.collapsed,
            );
        });
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[("↑↓", "Move"), ("Enter", "Open"), ("Ctrl+B", "Sidebar")]
    }
}

#[cfg(test)]
mod tests {
    use super::{Page, SidebarsPage};
    use termrock::{App, Color, Cx, Modifier, Response, Theme, Ui};

    struct Host;

    impl App for Host {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            SidebarsPage::new().draw(ui, ui.full());
        }
    }

    #[test]
    fn playground_matches_the_120_column_body() {
        let h = termrock_test_support::Harness::new(Host, Theme::junie(), 94, 36);
        let text = h.text();
        assert!(text.contains("Playground"), "{text}");
        assert!(text.contains("the live widget"), "{text}");
        assert!(text.contains("Last: none yet"), "{text}");
        assert!(
            text.contains("One focus stop. ↑↓ (or j k) move the cursor, Enter opens."),
            "{text}"
        );

        let (x, y) = h.find("Playground").expect("tab");
        let tab = h.cell(x, y);
        assert_eq!(tab.bg, Color::Rgb(0x27, 0x27, 0x2a));
        assert!(tab.modifier.contains(Modifier::BOLD));
        assert_eq!(h.cell(x.saturating_sub(1), y + 1).symbol(), "━");

        let (mx, my) = h.find("Sections").expect("blurb");
        assert_eq!(h.cell(mx, my).fg, Color::Rgb(0x8a, 0x8a, 0x8a));

        let (wx, wy) = h.find("Workspace").expect("section");
        assert_eq!(h.cell(wx, wy).fg, Color::Rgb(0x73, 0x73, 0x73));
        assert_eq!((wx, wy), (3, 6), "{text}");
        assert_eq!(h.cell(24, 5).symbol(), "\u{2039}");
        assert_eq!(h.cell(27, wy).symbol(), "│");
        assert_eq!(h.cell(1, 7).symbol(), "\u{203a}");
        assert_eq!(h.cell(5, 7).symbol(), "T");
        assert_eq!(h.cell(5, 7).bg, Color::Rgb(0x17, 0x3f, 0x1c));
        assert!(h.cell(5, 7).modifier.contains(Modifier::BOLD));
        assert_eq!(h.cell(25, 7).symbol(), "3");
        assert_eq!(h.cell(25, 7).fg, Color::Rgb(0x48, 0xe0, 0x54));
        assert_eq!(h.cell(32, 5).symbol(), "T");
        assert_eq!(h.cell(32, 5).fg, Color::Rgb(0xb3, 0xb3, 0xb3));

        let (bx, by) = h.find("Billing").expect("disabled");
        assert_eq!((bx, by), (5, 14), "{text}");
        assert_eq!(h.cell(bx, by).fg, Color::Rgb(0x4d, 0x4d, 0x4d));
        assert_eq!(h.cell(0, by).fg, Color::Rgb(0x11, 0x11, 0x11));

        assert_eq!(h.cell(2, 23).symbol(), "C");
        assert_eq!(h.cell(2, 23).bg, Color::Rgb(0x27, 0x27, 0x2a));
        assert_eq!(h.cell(1, 23).fg, Color::Rgb(0x27, 0x27, 0x2a));
        assert_eq!(h.cell(32, 9).symbol(), "\u{203a}");
        assert_eq!(h.cell(32, 9).fg, Color::Rgb(0x48, 0xe0, 0x54));
        assert_eq!(h.cell(32, 6).symbol(), "─");
    }

    #[test]
    fn narrow_playground_scrolls_and_ends_the_blurb_on_ellipsis() {
        let h = termrock_test_support::Harness::new(Host, Theme::junie(), 51, 16);
        assert_eq!(h.cell(48, 0).symbol(), "h");
        assert_eq!(h.cell(49, 0).symbol(), "…");
        assert_eq!(h.cell(49, 0).fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(h.cell(49, 0).bg, Color::Rgb(0, 0, 0));
        assert_eq!(h.cell(48, 7).symbol(), "↑");
        assert_eq!(h.cell(32, 8).symbol(), "(");
        assert_eq!(h.cell(50, 5).symbol(), "┃");
        assert_eq!(h.cell(50, 5).fg, Color::Rgb(0x80, 0x80, 0x80));
        assert_eq!(h.cell(50, 8).symbol(), "┆");
        assert_eq!(h.cell(50, 8).fg, Color::Rgb(0x47, 0x47, 0x47));
        assert_eq!(h.cell(0, 15).symbol(), "━");
        assert_eq!(h.cell(44, 15).symbol(), "━");
        assert_eq!(h.cell(45, 15).symbol(), "┄");
        assert_eq!(h.cell(49, 15).symbol(), "┄");
        assert_eq!(h.cell(50, 15).symbol(), " ");
    }
}
