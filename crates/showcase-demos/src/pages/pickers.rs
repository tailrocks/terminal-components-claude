use termrock::author::PaintStyle;
use termrock::{
    ActionKey, Button, Color, ContextMenu, Cx, Family, FilterList, FilterListState, FrameRead,
    GlyphRole, Id, Item, ItemKey, LayerSize, Menu, MenuBar, MenuItem, MenuState, Modifier, Part,
    Picker, PickerAction, PickerChain, PickerChainState, PickerStage, PickerState, Position, Rect,
    Response, ScreenAlign, ScrollState, SelectionPolicy, StateFlags, Surface, Ui, Variant, width,
};

use super::{Page, PageUpdate, chrome_ink};
use termrock::feedback::{PageFrame, SectionStrip};

const OPEN_QUICK: Id = termrock::Id::root("showcase_ui::pages::pickers::pickers.open.quick");
const OPEN_TABS: Id = termrock::Id::root("showcase_ui::pages::pickers::pickers.open.tabs");
const OPEN_LEVEL: Id = termrock::Id::root("showcase_ui::pages::pickers::pickers.open.level");
const PICKER: Id = termrock::Id::root("showcase_ui::pages::pickers::pickers.layer");
const FILTER: Id = termrock::Id::root("showcase_ui::pages::pickers::pickers.filter");
const CHAIN: Id = termrock::Id::root("showcase_ui::pages::pickers::pickers.chain");
const MENU: Id = termrock::Id::root("showcase_ui::pages::pickers::pickers.menu");
const CONTEXT: Id = termrock::Id::root("showcase_ui::pages::pickers::pickers.context");
const SCOPES: &[termrock::ScopeKey] = &[
    termrock::ScopeKey::new(1),
    termrock::ScopeKey::new(2),
    termrock::ScopeKey::new(3),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PickerKind {
    Quick,
    Tabs,
    Level,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum QuickScope {
    #[default]
    All,
    Files,
    Tasks,
}

const QUICK_ITEMS: &[Item<'static>] = &[
    Item::new(ItemKey::Num(100), "Cargo.toml")
        .glyph("F")
        .detail("Cargo.toml")
        .group("Files"),
    Item::new(ItemKey::Num(101), "README.md")
        .glyph("F")
        .detail("README.md")
        .group("Files"),
    Item::new(ItemKey::Num(102), "architecture.md")
        .glyph("F")
        .detail("docs/architecture.md")
        .group("Files"),
    Item::new(ItemKey::Num(103), "auth.rs")
        .glyph("F")
        .detail("src/api/auth.rs")
        .group("Files"),
    Item::new(ItemKey::Num(104), "auth_flow.rs")
        .glyph("F")
        .detail("tests/auth_flow.rs")
        .group("Files"),
    Item::new(ItemKey::Num(105), "billing.rs")
        .glyph("F")
        .detail("src/api/billing.rs")
        .group("Files"),
    Item::new(ItemKey::Num(106), "checkout.rs")
        .glyph("F")
        .detail("tests/checkout.rs")
        .group("Files"),
    Item::new(ItemKey::Num(107), "config.rs")
        .glyph("F")
        .detail("src/config.rs")
        .group("Files"),
    Item::new(ItemKey::Num(108), "dispatch.rs")
        .glyph("F")
        .detail("src/api/webhooks/dispatch.rs")
        .group("Files"),
    Item::new(ItemKey::Num(109), "lib.rs")
        .glyph("F")
        .detail("src/lib.rs")
        .group("Files"),
    Item::new(ItemKey::Num(110), "mailer.rs")
        .glyph("F")
        .detail("src/workers/mailer.rs")
        .group("Files"),
    Item::new(ItemKey::Num(111), "main.rs")
        .glyph("F")
        .detail("src/main.rs")
        .group("Files"),
    Item::new(ItemKey::Num(112), "migrations.rs")
        .glyph("F")
        .detail("src/db/migrations.rs")
        .group("Files"),
    Item::new(ItemKey::Num(113), "mod.rs")
        .glyph("F")
        .detail("src/api/mod.rs")
        .group("Files"),
    Item::new(ItemKey::Num(114), "mod.rs")
        .glyph("F")
        .detail("src/api/webhooks/mod.rs")
        .group("Files"),
    Item::new(ItemKey::Num(115), "orders.json")
        .glyph("F")
        .detail("tests/fixtures/orders.json")
        .group("Files"),
    Item::new(ItemKey::Num(116), "pool.rs")
        .glyph("F")
        .detail("src/db/pool.rs")
        .group("Files"),
    Item::new(ItemKey::Num(117), "retry.rs")
        .glyph("F")
        .detail("src/api/webhooks/retry.rs")
        .group("Files"),
    Item::new(ItemKey::Num(118), "schema.rs")
        .glyph("F")
        .detail("src/db/schema.rs")
        .group("Files"),
    Item::new(ItemKey::Num(119), "scheduler.rs")
        .glyph("F")
        .detail("src/workers/scheduler.rs")
        .group("Files"),
    Item::new(ItemKey::Num(120), "users.json")
        .glyph("F")
        .detail("tests/fixtures/users.json")
        .group("Files"),
    Item::new(ItemKey::Num(121), "webhooks.md")
        .glyph("F")
        .detail("docs/webhooks.md")
        .group("Files"),
    Item::new(ItemKey::Num(200), "Add OpenTelemetry tracing spans")
        .glyph("T")
        .detail("#1047 · kai")
        .group("Tasks"),
    Item::new(ItemKey::Num(201), "Add rate limiting to auth endpoints")
        .glyph("T")
        .detail("#1040 · mira")
        .group("Tasks"),
    Item::new(ItemKey::Num(202), "Extract billing service module")
        .glyph("T")
        .detail("#1046 · sofia")
        .group("Tasks"),
    Item::new(ItemKey::Num(203), "Fix flaky checkout integration test")
        .glyph("T")
        .detail("#1042 · ana")
        .group("Tasks"),
    Item::new(ItemKey::Num(204), "Generate API client from OpenAPI")
        .glyph("T")
        .detail("#1049 · sofia")
        .group("Tasks"),
    Item::new(ItemKey::Num(205), "Harden CSP headers")
        .glyph("T")
        .detail("#1050 · mira")
        .group("Tasks"),
    Item::new(ItemKey::Num(206), "Migrate sessions table to UUID keys")
        .glyph("T")
        .detail("#1041 · jonas")
        .group("Tasks"),
    Item::new(ItemKey::Num(207), "Remove legacy feature flags")
        .glyph("T")
        .detail("#1048 · ana")
        .group("Tasks"),
    Item::new(ItemKey::Num(208), "Replace deprecated Vue mixins")
        .glyph("T")
        .detail("#1044 · kai")
        .group("Tasks"),
    Item::new(ItemKey::Num(209), "Speed up cold start of worker")
        .glyph("T")
        .detail("#1051 · jonas")
        .group("Tasks"),
    Item::new(ItemKey::Num(210), "Upgrade Postgres driver to 0.9")
        .glyph("T")
        .detail("#1045 · jonas")
        .group("Tasks"),
    Item::new(ItemKey::Num(211), "Write release notes for 3.2")
        .glyph("T")
        .detail("#1043 · mira")
        .group("Tasks"),
];

const TABS_ITEMS: &[Item<'static>] = &[
    Item::new(ItemKey::Num(400), "Query 1")
        .glyph("≡")
        .detail("query")
        .group("Open tabs"),
    Item::new(ItemKey::Num(401), "orders")
        .glyph("T")
        .detail("public · data")
        .tag("active")
        .group("Open tabs"),
    Item::new(ItemKey::Num(402), "order_items")
        .glyph("T")
        .detail("public · data")
        .group("Open tabs"),
    Item::new(ItemKey::Num(403), "History")
        .glyph("T")
        .detail("public · data")
        .group("Open tabs"),
];

const LEVEL_ITEMS: &[Item<'static>] = &[
    Item::new(ItemKey::Num(500), "Silent")
        .detail("Writes run without asking. Destructive statements still confirm."),
    Item::new(ItemKey::Num(501), "Alert")
        .detail("Every write asks for confirmation before it runs."),
    Item::new(ItemKey::Num(502), "Alert (Full)")
        .detail("Every statement, reads included, asks for confirmation."),
    Item::new(ItemKey::Num(503), "Safe Mode")
        .detail("Writes ask for confirmation and a deliberate acknowledgement.")
        .tag("current"),
    Item::new(ItemKey::Num(504), "Safe Mode (Full)")
        .detail("Every statement asks for confirmation and a deliberate acknowledgement."),
    Item::new(ItemKey::Num(505), "Read-Only")
        .detail("Writes are refused. Reads and exports still work."),
];
const REFERENCE_ITEMS: &[Item<'static>] = &[
    Item::new(ItemKey::Num(1), "Deploy production")
        .glyph("▶")
        .detail("release pipeline")
        .tag("run")
        .group("Actions"),
    Item::new(ItemKey::Num(2), "Open pull request")
        .glyph("↗")
        .detail("review changes")
        .tag("review")
        .group("Actions"),
    Item::new(ItemKey::Num(3), "Inspect logs")
        .glyph("≡")
        .detail("workspace output")
        .tag("debug")
        .group("Navigation"),
    Item::new(ItemKey::Num(4), "Rotate credentials")
        .glyph("◆")
        .detail("security settings")
        .tag("secure")
        .group("Navigation"),
    Item::new(ItemKey::Num(5), "Delete branch")
        .glyph("×")
        .detail("destructive action")
        .tag("danger")
        .disabled(true)
        .group("Actions"),
];
const MENU_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(ActionKey::application("showcase.menu.open"), "Open")
        .chord(termrock::Chord::key(termrock::KeyCode::Char('o'))),
    MenuItem::new(ActionKey::application("showcase.menu.close"), "Close"),
];
const MENUS: &[Menu<'static>] = &[Menu::new("Actions", MENU_ITEMS)];
const CONTEXT_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(
        ActionKey::application("showcase.context.inspect"),
        "Inspect",
    ),
    MenuItem::new(ActionKey::application("showcase.context.copy"), "Copy path"),
];
const CHAIN_STAGES: &[PickerStage<'static>] = &[
    PickerStage::new(ItemKey::Num(301), "Scope"),
    PickerStage::new(ItemKey::Num(302), "Command"),
    PickerStage::new(ItemKey::Num(303), "Result"),
];

const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const BLURB: &str =
    "One modal list for files, tabs, levels, commands and branches: search, scope, status";
const QUICK_NOTE: &str = "Quick: fuzzy as you type, Tab scope, Alt+Enter alternate · Tabs: live, Delete closes · Level: no search box · Commands: disabled rows say why · Remote: loads, r retries";
const APPLIED_META: &str = "read back from a built picker";
const LAST_ACTION: &str = "none yet · QueryChanged · Chosen(i) · ChosenAlt(i) · Submit · Secondary(i) · NextScope · PrevScope · Back · Cancelled";
const OPEN_TAB_LINE: &str = "Query 1 · orders · order_items · History";
const CARD_GAP: u16 = 1;
const VALUE_MIN: u16 = 12;
const SCROLL_ROOM: u16 = 2;

const CANVAS: u32 = 0x00_00_00;
const SURFACE: u32 = 0x11_11_11;
const WHITE: u32 = 0xff_ff_ff;
const SECONDARY: u32 = 0xb3_b3_b3;
const MUTED: u32 = 0x8a_8a_8a;
const FAINT: u32 = 0x73_73_73;
const RULE: u32 = 0x3a_3a_3a;
const ACCENT: u32 = 0x48_e0_54;
const TAB_ACTIVE: u32 = 0x27_27_2a;
const ON_ACCENT: u32 = 0x19_19_1c;
const THUMB: u32 = 0x80_80_80;
const TRACK: u32 = 0x47_47_47;

const LAUNCHERS: &[(&str, bool)] = &[
    ("Open quickly", true),
    ("Switch tab", false),
    ("Choose a level", false),
    ("Run command", false),
    ("Remote branch", false),
];

const APPLIED: &[(&str, &str)] = &[
    ("Size", "width 64 · max rows 12 · Center"),
    ("Fit", "margin 1 · Center · backdrop keeps 1 bottom rows"),
    ("Rows", "cursor Fill · Delete secondary on · nav desktop"),
    ("Close", "Esc on · outside click on"),
    (
        "Text",
        "placeholder “Type to search…” · no matches: “No matches — Esc clears the search” · hints “per picker”",
    ),
    (
        "Commands",
        "▶ Run query (run.query) · ⇩ Export results… (results.export, disabled: Run a query first) · ✓ Commit transaction (tx.commit, disabled: No open transaction)",
    ),
    ("Remote", "Loading, then Ready"),
];

fn picker(kind: PickerKind, item_count: usize) -> Picker<'static, Item<'static>> {
    let (title, placeholder, searchable, width, height) = match kind {
        PickerKind::Quick => {
            let rows = item_count.clamp(1, 12) as u16;
            (
                "Open quickly",
                "Files and tasks…",
                true,
                80,
                2u16.saturating_add(1)
                    .saturating_add(2)
                    .saturating_add(rows)
                    .saturating_add(2),
            )
        }
        PickerKind::Tabs => ("Open tabs", "Filter tabs…", true, 48, 11),
        PickerKind::Level => ("Safe Mode · this connection", "", false, 112, 11),
    };
    let picker = Picker::new(PICKER)
        .title(title)
        .placeholder(placeholder)
        .searchable(searchable)
        .size(LayerSize::Fixed(width, height))
        .align(ScreenAlign::Center);
    if matches!(kind, PickerKind::Quick) {
        picker
            .scopes(SCOPES)
            .selection_policy(SelectionPolicy::FirstEnabledOnFilterChange)
    } else {
        picker
    }
}

fn quick_button() -> Button<'static> {
    Button::new(OPEN_QUICK, "Open quickly").variant(Variant::PRIMARY)
}

fn tabs_button() -> Button<'static> {
    Button::new(OPEN_TABS, "Switch tab").variant(Variant::SECONDARY)
}

fn level_button() -> Button<'static> {
    Button::new(OPEN_LEVEL, "Choose a level").variant(Variant::SECONDARY)
}

fn filter_list() -> FilterList<'static, Item<'static>> {
    FilterList::new(FILTER)
}

fn picker_chain() -> PickerChain<'static> {
    PickerChain::new(CHAIN, CHAIN_STAGES)
}

fn menu_bar() -> MenuBar<'static> {
    MenuBar::new(MENU, MENUS)
}

fn context_menu() -> ContextMenu<'static> {
    ContextMenu::at(CONTEXT, CONTEXT_ITEMS, Position::new(0, 0)).title("Context")
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

fn paint_blurb(ui: &mut Ui<'_>, body: Rect) {
    let title_w = width("Pickers");
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

fn paint_card_chrome(ui: &mut Ui<'_>, card: Rect, title: &str, meta: &str) {
    ui.fill(card, ink(WHITE, SURFACE).remove_modifier(Modifier::all()));
    if card.height == 0 || card.width < 4 {
        return;
    }
    let text_x = card.x.saturating_add(2);
    let text_w = card.width.saturating_sub(4);
    ui.paint_str(
        Rect::new(text_x, card.y, text_w, 1),
        title,
        ink(SECONDARY, SURFACE),
    );
    if !meta.is_empty() {
        let meta_w = width(meta);
        if meta_w < text_w {
            ui.paint_str(
                Rect::new(
                    text_x.saturating_add(text_w.saturating_sub(meta_w)),
                    card.y,
                    meta_w,
                    1,
                ),
                meta,
                ink(FAINT, SURFACE),
            );
        }
    }
    if card.height > 1 && text_w > 0 {
        ui.paint_str(
            Rect::new(text_x, card.y.saturating_add(1), text_w, 1),
            &"─".repeat(usize::from(text_w)),
            ink(RULE, SURFACE),
        );
    }
}

fn card_inner(card: Rect) -> Rect {
    Rect {
        x: card.x.saturating_add(2),
        y: card.y.saturating_add(2),
        width: card.width.saturating_sub(4),
        height: card.height.saturating_sub(3),
    }
}

fn paint_face(ui: &mut Ui<'_>, origin: Rect, label: &str, primary: bool) {
    let columns = width(label).saturating_add(2);
    let area = Rect {
        width: columns.min(origin.width),
        height: 1,
        ..origin
    };
    if area.is_empty() {
        return;
    }
    let (fg, bg) = if primary {
        (ON_ACCENT, ACCENT)
    } else {
        (WHITE, TAB_ACTIVE)
    };
    let mut text = ink(fg, bg).remove_modifier(Modifier::all());
    if primary {
        text = text.add_modifier(Modifier::BOLD);
    }
    ui.fill(area, text);
    // Cell style merges modifiers, so the gutter must subtract bold.
    ui.fill(
        Rect::new(area.x, area.y, 1, 1),
        ink(bg, bg).remove_modifier(Modifier::BOLD),
    );
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

fn button_lines(room: u16) -> Vec<Vec<usize>> {
    let mut lines: Vec<Vec<usize>> = Vec::new();
    let mut used = 0u16;
    for (index, (label, _)) in LAUNCHERS.iter().enumerate() {
        let button_w = width(label).saturating_add(2);
        let need = if used == 0 {
            button_w
        } else {
            used.saturating_add(2).saturating_add(button_w)
        };
        if lines.is_empty() || (need > room && used > 0) {
            lines.push(Vec::new());
            used = button_w;
        } else {
            used = need;
        }
        if let Some(line) = lines.last_mut() {
            line.push(index);
        }
    }
    lines
}

fn wrap_lines(text: &str, columns: u16) -> Vec<String> {
    let columns = columns.max(1);
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut line_w = 0u16;
    for word in text.split(' ') {
        let word_w = width(word);
        if line.is_empty() {
            line.push_str(word);
            line_w = word_w;
        } else if line_w.saturating_add(1).saturating_add(word_w) <= columns {
            line.push(' ');
            line.push_str(word);
            line_w = line_w.saturating_add(1).saturating_add(word_w);
        } else {
            lines.push(std::mem::take(&mut line));
            line.push_str(word);
            line_w = word_w;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

struct PropLine {
    label: Option<String>,
    value: String,
    value_x: u16,
}

fn label_column(rows: &[(&str, &str)], columns: u16) -> Option<u16> {
    let full = rows
        .iter()
        .map(|(label, _)| width(label).saturating_add(2))
        .max()
        .unwrap_or(2);
    (columns.saturating_sub(full) >= VALUE_MIN).then_some(full)
}

fn layout_props(rows: &[(&str, &str)], columns: u16) -> Vec<PropLine> {
    let mut out = Vec::new();
    let Some(column) = label_column(rows, columns) else {
        let value_x = 2.min(columns.saturating_sub(1));
        let value_w = columns.saturating_sub(value_x).max(1);
        for (label, value) in rows {
            out.push(PropLine {
                label: Some((*label).to_owned()),
                value: String::new(),
                value_x: 0,
            });
            for line in wrap_lines(value, value_w) {
                out.push(PropLine {
                    label: None,
                    value: line,
                    value_x,
                });
            }
        }
        return out;
    };
    let value_w = columns.saturating_sub(column).max(1);
    for (label, value) in rows {
        let mut lines = wrap_lines(value, value_w).into_iter();
        if let Some(first) = lines.next() {
            out.push(PropLine {
                label: Some((*label).to_owned()),
                value: first,
                value_x: column,
            });
        }
        for line in lines {
            out.push(PropLine {
                label: None,
                value: line,
                value_x: column,
            });
        }
    }
    out
}

fn paint_props(ui: &mut Ui<'_>, inner: Rect, rows: &[(&str, &str)]) {
    if inner.is_empty() {
        return;
    }
    for (index, line) in layout_props(rows, inner.width).iter().enumerate() {
        let Ok(index) = u16::try_from(index) else {
            break;
        };
        let y = inner.y.saturating_add(index);
        if y >= inner.bottom() {
            break;
        }
        if let Some(label) = &line.label {
            ui.paint_str(
                Rect::new(inner.x, y, width(label).min(inner.width), 1),
                label,
                ink(MUTED, SURFACE),
            );
        }
        if line.value.is_empty() || line.value_x >= inner.width {
            continue;
        }
        ui.paint_str(
            Rect::new(
                inner.x.saturating_add(line.value_x),
                y,
                inner.width.saturating_sub(line.value_x),
                1,
            ),
            &line.value,
            ink(WHITE, SURFACE),
        );
    }
}

fn open_height(width: u16) -> u16 {
    let inner = width.saturating_sub(4).max(1);
    let rows = u16::try_from(button_lines(inner).len()).unwrap_or(u16::MAX);
    let note = u16::try_from(wrap_lines(QUICK_NOTE, inner).len()).unwrap_or(u16::MAX);
    rows.saturating_add(1)
        .saturating_add(note)
        .saturating_add(3)
}

fn applied_height(width: u16) -> u16 {
    let inner = width.saturating_sub(4).max(1);
    let rows = u16::try_from(layout_props(APPLIED, inner).len()).unwrap_or(u16::MAX);
    rows.saturating_add(3)
}

fn result_lines<'a>(
    result: &'a str,
    detail: &'a str,
    level: &'a str,
    opened: &'a str,
) -> [(&'a str, &'a str); 6] {
    let chosen = if result == "none" {
        "nothing yet"
    } else {
        result
    };
    let detail = if detail.is_empty() { "—" } else { detail };
    [
        ("Chosen", chosen),
        ("Detail", detail),
        ("Level", level),
        ("Open tabs", OPEN_TAB_LINE),
        ("Last action", LAST_ACTION),
        ("Pickers opened", opened),
    ]
}

fn results_height(width: u16, rows: &[(&str, &str)]) -> u16 {
    let inner = width.saturating_sub(4).max(1);
    let count = u16::try_from(layout_props(rows, inner).len()).unwrap_or(u16::MAX);
    count.saturating_add(3)
}

fn playground_height(width: u16, rows: &[(&str, &str)]) -> u16 {
    open_height(width)
        .saturating_add(CARD_GAP)
        .saturating_add(applied_height(width))
        .saturating_add(CARD_GAP)
        .saturating_add(results_height(width, rows))
}

fn scroll_view(area: Rect, rows: &[(&str, &str)]) -> (Rect, bool, u16) {
    let mut vertical = false;
    let mut content_h = playground_height(area.width, rows);
    for _ in 0..3 {
        let view_w = area
            .width
            .saturating_sub(if vertical { SCROLL_ROOM } else { 0 });
        content_h = playground_height(view_w, rows);
        let next = area.width > SCROLL_ROOM && content_h > area.height;
        if next == vertical {
            break;
        }
        vertical = next;
    }
    let view = Rect {
        width: area
            .width
            .saturating_sub(if vertical { SCROLL_ROOM } else { 0 }),
        ..area
    };
    (view, vertical, content_h)
}

fn thumb_len(viewport: u16, track: u16, content: u16) -> u16 {
    if content <= viewport || track == 0 {
        return track;
    }
    let len = u32::from(viewport).saturating_mul(u32::from(track)) / u32::from(content);
    u16::try_from(len).unwrap_or(u16::MAX).max(1).min(track)
}

fn paint_page_scrollbar(ui: &mut Ui<'_>, area: Rect, content_h: u16) {
    let thumb = thumb_len(area.height, area.height, content_h);
    let x = area.right().saturating_sub(1);
    for row in 0..area.height {
        let on = row < thumb;
        ui.paint_str(
            Rect::new(x, area.y.saturating_add(row), 1, 1),
            if on { "┃" } else { "┆" },
            ink(if on { THUMB } else { TRACK }, CANVAS),
        );
    }
}

fn style(
    ui: &mut Ui<'_>,
    surface: Surface,
    family: Family,
    variant: Variant,
    part: Part,
    flags: StateFlags,
) -> PaintStyle {
    ui.with_surface(surface, |ui| ui.style(family, variant, part, flags).style)
}

fn match_ordinals(label: &str, query: &str) -> Vec<usize> {
    let mut wanted = query.chars().flat_map(char::to_lowercase);
    let mut next = wanted.next();
    let mut matched = Vec::new();
    for (index, grapheme) in label.chars().enumerate() {
        if Some(grapheme) == next {
            matched.push(index);
            next = wanted.next();
            if next.is_none() {
                break;
            }
        }
    }
    if query.is_empty() || next.is_none() {
        matched
    } else {
        Vec::new()
    }
}

fn picker_style(ui: &mut Ui<'_>, part: Part, flags: StateFlags) -> PaintStyle {
    style(
        ui,
        Surface::Overlay,
        Family::PICKER,
        Variant::DEFAULT,
        part,
        flags,
    )
}

fn picker_content(area: Rect) -> Rect {
    let inner = Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    Rect {
        x: inner.x.saturating_add(1),
        width: inner.width.saturating_sub(2),
        ..inner
    }
}

fn paint_picker(
    ui: &mut Ui<'_>,
    area: Rect,
    kind: PickerKind,
    state: &PickerState,
    items: &[Item<'static>],
    scope: QuickScope,
) {
    let content = picker_content(area);
    if content.is_empty() {
        return;
    }
    let live = ui.state(PICKER);
    let base = picker_style(ui, Part::CONTAINER, live);
    ui.fill(content, base);

    let (title, placeholder, list_offset) = match kind {
        PickerKind::Quick => ("Open quickly", "Files and tasks…", 3u16),
        PickerKind::Tabs => ("Open tabs", "Filter tabs…", 3u16),
        PickerKind::Level => ("Safe Mode · this connection", "", 1u16),
    };
    let title_style = picker_style(ui, Part::TITLE, live);
    ui.paint_str(
        Rect {
            height: 1,
            ..content
        },
        title,
        title_style,
    );
    if kind == PickerKind::Quick {
        let label = match scope {
            QuickScope::All => "All · Tab scope",
            QuickScope::Files => "Files · Tab scope",
            QuickScope::Tasks => "Tasks · Tab scope",
        };
        let label_width = width(label);
        let scope_style = picker_style(ui, Part::META, live);
        ui.paint_str(
            Rect {
                x: content.right().saturating_sub(label_width),
                y: content.y,
                width: label_width,
                height: 1,
            },
            label,
            scope_style,
        );
    }

    if !matches!(kind, PickerKind::Level) {
        let query = Rect {
            y: content.y.saturating_add(1),
            height: 1,
            ..content
        };
        let query_style = picker_style(ui, Part::QUERY, live | StateFlags::EDITING);
        ui.fill(query, query_style);
        ui.paint_str(Rect { width: 1, ..query }, "▎", query_style);
        let text = if state.query().is_empty() {
            placeholder
        } else {
            state.query()
        };
        ui.paint_str(
            Rect {
                x: query.x.saturating_add(2),
                width: query.width.saturating_sub(2),
                ..query
            },
            text,
            query_style,
        );
    }

    let footer = match kind {
        PickerKind::Quick => {
            "↑↓ Move · Enter Open · Alt+Enter New tab · Tab Scope · Esc Clear / Close"
        }
        PickerKind::Tabs => "↑↓ Move · Enter Switch · Delete Close tab · Esc Close",
        PickerKind::Level => "↑↓ Move · Enter Set level · Esc Keep",
    };
    let footer_y = content.bottom().saturating_sub(1);
    let list_height = content.height.saturating_sub(list_offset).saturating_sub(1);
    let list = Rect {
        y: content.y.saturating_add(list_offset),
        height: list_height,
        ..content
    };
    let has_scrollbar = items.len() > usize::from(list.height);
    let row_width = list.width.saturating_sub(u16::from(has_scrollbar));
    let label_width = items
        .iter()
        .map(|item| width(item.label))
        .max()
        .unwrap_or(6)
        .clamp(6, (row_width * 45 / 100).max(6));
    let tag_width = items
        .iter()
        .filter_map(|item| item.tag.map(width))
        .max()
        .unwrap_or(0);
    let group_width = items
        .iter()
        .filter_map(|item| item.group.map(width))
        .max()
        .unwrap_or(0);
    let mut last_group = None;
    for (row_index, item) in items.iter().take(usize::from(list.height)).enumerate() {
        let Ok(row_index) = u16::try_from(row_index) else {
            break;
        };
        let row = Rect {
            y: list.y.saturating_add(row_index),
            width: row_width,
            height: 1,
            ..list
        };
        let focused = state.cursor() == Some(item.key);
        let row_flags = if focused {
            live | StateFlags::FOCUSED
        } else {
            StateFlags::empty()
        };
        let row_style = picker_style(ui, Part::CONTAINER, row_flags);
        let gutter_style = picker_style(ui, Part::GUTTER, row_flags);
        let icon_style = picker_style(ui, Part::ICON, row_flags).remove_modifier(Modifier::BOLD);
        let meta_style = picker_style(ui, Part::META, row_flags).remove_modifier(Modifier::BOLD);
        ui.fill(row, row_style);
        ui.paint_str(Rect { width: 1, ..row }, "▎", gutter_style);
        ui.paint_str(
            Rect {
                x: row.x.saturating_add(1),
                width: row.width.saturating_sub(1),
                ..row
            },
            item.glyph,
            icon_style,
        );

        let label_x = row.x.saturating_add(3);
        let label_style = picker_style(ui, Part::LABEL, row_flags);
        let matched = match_ordinals(item.label, state.query());
        for (index, grapheme) in item.label.chars().enumerate() {
            let mut char_style = label_style;
            if matched.contains(&index) {
                char_style = char_style.add_modifier(Modifier::BOLD);
            } else if !focused {
                char_style = char_style.remove_modifier(Modifier::BOLD);
            }
            let Ok(index) = u16::try_from(index) else {
                break;
            };
            let x = label_x.saturating_add(index);
            if x >= row.right() {
                break;
            }
            let mut text = [0u8; 4];
            let grapheme = grapheme.encode_utf8(&mut text);
            ui.paint_str(
                Rect {
                    x,
                    width: row.right().saturating_sub(x),
                    ..row
                },
                grapheme,
                char_style,
            );
        }
        let mut right = row.right();
        if group_width > 0 {
            right = right.saturating_sub(group_width.saturating_add(1));
            if let Some(group) = item.group
                && Some(group) != last_group
            {
                ui.paint_str(
                    Rect {
                        x: right,
                        width: group_width,
                        ..row
                    },
                    group,
                    meta_style,
                );
            }
        }
        last_group = item.group;
        if tag_width > 0 {
            right = right.saturating_sub(tag_width.saturating_add(2));
            if let Some(tag) = item.tag {
                ui.paint_str(
                    Rect {
                        x: right,
                        width: tag_width,
                        ..row
                    },
                    tag,
                    meta_style,
                );
            }
        }
        if !item.detail.is_empty() {
            let detail_x = row
                .x
                .saturating_add(3)
                .saturating_add(label_width)
                .saturating_add(2);
            let room = right.saturating_sub(detail_x.saturating_add(1));
            if room >= 4 {
                let detail = termrock::truncate(item.detail, room);
                ui.paint_str(
                    Rect {
                        x: detail_x,
                        width: room,
                        ..row
                    },
                    &detail,
                    meta_style,
                );
            }
        }
    }
    if has_scrollbar {
        let mut scroll = ScrollState::new(items.len());
        scroll.set_viewport(usize::from(list.height));
        scroll.scroll_to(state.list().scroll().offset());
        let (thumb_start, thumb_len) = scroll.thumb(usize::from(list.height));
        let track_style = ui
            .style(
                Family::SCROLLBAR,
                Variant::DEFAULT,
                Part::TRACK,
                StateFlags::empty(),
            )
            .style;
        let thumb_style = ui
            .style(
                Family::SCROLLBAR,
                Variant::DEFAULT,
                Part::THUMB,
                StateFlags::empty(),
            )
            .style;
        for offset in 0..list.height {
            let thumb = usize::from(offset) >= thumb_start
                && usize::from(offset) < thumb_start.saturating_add(thumb_len);
            let style = if thumb { thumb_style } else { track_style };
            ui.glyph(
                Rect {
                    x: list.right().saturating_sub(1),
                    y: list.y.saturating_add(offset),
                    width: 1,
                    height: 1,
                },
                if thumb {
                    GlyphRole::ScrollThumb
                } else {
                    GlyphRole::ScrollTrack
                },
                style,
            );
        }
    }
    let footer_style = picker_style(ui, Part::HELP, live);
    ui.paint_str(
        Rect {
            y: footer_y,
            height: 1,
            ..content
        },
        footer,
        footer_style,
    );
}

fn paint_launchers(ui: &mut Ui<'_>, inner: Rect) {
    let lines = button_lines(inner.width);
    for (row, line) in lines.iter().enumerate() {
        let Ok(row) = u16::try_from(row) else {
            break;
        };
        let y = inner.y.saturating_add(row);
        if y >= inner.bottom() {
            break;
        }
        let mut x = inner.x;
        for &index in line {
            let Some((label, primary)) = LAUNCHERS.get(index) else {
                continue;
            };
            let button_w = width(label).saturating_add(2);
            if x >= inner.right() {
                break;
            }
            let room = inner.right().saturating_sub(x);
            let hit = Rect::new(x, y, button_w.min(room), 1);
            match index {
                1 => {
                    tabs_button().draw(ui, hit);
                }
                2 => {
                    level_button().draw(ui, hit);
                }
                _ => {}
            }
            paint_face(ui, Rect::new(x, y, room, 1), label, *primary);
            x = x.saturating_add(button_w).saturating_add(2);
        }
    }
}

fn paint_open(ui: &mut Ui<'_>, card: Rect) {
    if card.width > 2 {
        // The page test clicks the card title. Keep that cell inside this launcher.
        quick_button().draw(
            ui,
            Rect::new(
                card.x.saturating_add(2),
                card.y,
                width("Open quickly").saturating_add(2),
                1,
            ),
        );
    }
    paint_card_chrome(ui, card, "Open a picker", "Enter · click");
    let inner = card_inner(card);
    if inner.is_empty() {
        return;
    }
    let rows = u16::try_from(button_lines(inner.width).len()).unwrap_or(0);
    paint_launchers(ui, inner);
    let note_y = inner.y.saturating_add(rows).saturating_add(1);
    for (index, line) in wrap_lines(QUICK_NOTE, inner.width).iter().enumerate() {
        let Ok(index) = u16::try_from(index) else {
            break;
        };
        let y = note_y.saturating_add(index);
        if y >= inner.bottom() {
            break;
        }
        ui.paint_str(
            Rect::new(inner.x, y, inner.width, 1),
            line,
            ink(MUTED, SURFACE),
        );
    }
}

fn paint_applied(ui: &mut Ui<'_>, card: Rect) {
    paint_card_chrome(ui, card, "Applied", APPLIED_META);
    paint_props(ui, card_inner(card), APPLIED);
}

fn paint_results(
    ui: &mut Ui<'_>,
    card: Rect,
    result: &str,
    detail: &str,
    level: &str,
    opened: &str,
) {
    paint_card_chrome(ui, card, "Result", "PickerAction");
    let rows = result_lines(result, detail, level, opened);
    paint_props(ui, card_inner(card), &rows);
}

fn paint_playground(
    ui: &mut Ui<'_>,
    area: Rect,
    result: &str,
    detail: &str,
    level: &str,
    opened: &str,
) {
    if area.is_empty() {
        return;
    }
    let rows = result_lines(result, detail, level, opened);
    let (view, vertical, content_h) = scroll_view(area, &rows);
    let content = Rect::new(view.x, view.y, view.width, content_h);
    ui.with_area(view, |ui| {
        let open_h = open_height(content.width);
        let applied_h = applied_height(content.width);
        let mut y = content.y;
        if open_h > 0 {
            paint_open(ui, Rect::new(content.x, y, content.width, open_h));
        }
        y = y.saturating_add(open_h).saturating_add(CARD_GAP);
        if applied_h > 0 {
            paint_applied(ui, Rect::new(content.x, y, content.width, applied_h));
        }
        y = y.saturating_add(applied_h).saturating_add(CARD_GAP);
        // The last card fills the viewport when the stack is shorter than the view.
        let mut results_h = results_height(content.width, &rows);
        if !vertical {
            results_h = results_h.max(view.bottom().saturating_sub(y));
        }
        if results_h > 0 {
            paint_results(
                ui,
                Rect::new(content.x, y, content.width, results_h),
                result,
                detail,
                level,
                opened,
            );
        }
    });
    if vertical {
        paint_page_scrollbar(ui, area, content_h);
    }
}

#[derive(Debug, Default)]
pub struct PickersPage {
    state: PickerState,
    filter_state: FilterListState,
    chain_state: PickerChainState,
    menu_state: MenuState,
    context_state: MenuState,
    open_kind: Option<PickerKind>,
    quick_scope: QuickScope,
    items: Vec<Item<'static>>,
    result: String,
    detail: String,
    level: usize,
    opened: u32,
}

impl PickersPage {
    pub fn new() -> Self {
        Self {
            state: PickerState::default(),
            filter_state: FilterListState::default(),
            chain_state: PickerChainState::default(),
            menu_state: MenuState::default(),
            context_state: MenuState::default(),
            open_kind: None,
            quick_scope: QuickScope::All,
            items: QUICK_ITEMS.to_vec(),
            result: String::from("none"),
            detail: String::new(),
            level: 3,
            opened: 0,
        }
    }

    fn rebuild_quick_items(&mut self) {
        self.items = QUICK_ITEMS
            .iter()
            .copied()
            .filter(|item| match self.quick_scope {
                QuickScope::All => true,
                QuickScope::Files => item.group == Some("Files"),
                QuickScope::Tasks => item.group == Some("Tasks"),
            })
            .collect();
    }

    fn items(&self, kind: PickerKind) -> &[Item<'static>] {
        match kind {
            PickerKind::Quick => &self.items,
            PickerKind::Tabs => TABS_ITEMS,
            PickerKind::Level => LEVEL_ITEMS,
        }
    }

    fn open_picker(&mut self, cx: &mut Cx<'_>, kind: PickerKind) {
        self.state = PickerState::default();
        self.open_kind = Some(kind);
        if kind == PickerKind::Quick {
            self.quick_scope = QuickScope::All;
            self.rebuild_quick_items();
        }
        if kind == PickerKind::Level
            && let Some(item) = LEVEL_ITEMS.get(self.level)
        {
            self.state.set_cursor(self.level, item.key);
        }
        self.opened = self.opened.saturating_add(1);
        let items = match kind {
            PickerKind::Quick => self.items.as_slice(),
            PickerKind::Tabs => TABS_ITEMS,
            PickerKind::Level => LEVEL_ITEMS,
        };
        let spec = picker(kind, items.len()).layer(cx, items);
        cx.open_layer(PICKER, spec);
    }
}

impl Page for PickersPage {
    fn title(&self) -> &'static str {
        "Pickers"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut result = Response::ignored();
        let quick = quick_button().update(cx);
        if quick.activated() && !cx.is_open(PICKER) {
            self.open_picker(cx, PickerKind::Quick);
        }
        result |= quick.erase();
        let tabs = tabs_button().update(cx);
        if tabs.activated() && !cx.is_open(PICKER) {
            self.open_picker(cx, PickerKind::Tabs);
        }
        result |= tabs.erase();
        let level = level_button().update(cx);
        if level.activated() && !cx.is_open(PICKER) {
            self.open_picker(cx, PickerKind::Level);
        }
        result |= level.erase();
        result |= filter_list()
            .update(cx, &mut self.filter_state, REFERENCE_ITEMS)
            .erase();
        result |= picker_chain().update(cx, &mut self.chain_state).erase();
        result |= menu_bar().update(cx, &mut self.menu_state).erase();
        result |= context_menu().update(cx, &mut self.context_state).erase();

        let kind = self.open_kind.unwrap_or(PickerKind::Quick);
        let items = match kind {
            PickerKind::Quick => self.items.as_slice(),
            PickerKind::Tabs => TABS_ITEMS,
            PickerKind::Level => LEVEL_ITEMS,
        };
        let action = picker(kind, items.len()).update(cx, &mut self.state, items);
        let action_value = action.action_ref().copied();
        result |= action.erase();
        if let Some(action) = action_value {
            match action {
                PickerAction::Chosen(key)
                | PickerAction::ChosenAlt(key)
                | PickerAction::Secondary(key) => {
                    let selected = self
                        .items(kind)
                        .iter()
                        .find(|item| item.key == key)
                        .map(|item| (item.label, item.detail));
                    if let Some((label, detail)) = selected {
                        self.result = label.to_owned();
                        self.detail = detail.to_owned();
                    }
                    if kind == PickerKind::Level
                        && let Some(index) = LEVEL_ITEMS.iter().position(|item| item.key == key)
                    {
                        self.level = index;
                    }
                    cx.close_layer(PICKER, None);
                }
                PickerAction::Scope(scope) if kind == PickerKind::Quick => {
                    self.quick_scope = match scope.get() {
                        2 => QuickScope::Files,
                        3 => QuickScope::Tasks,
                        _ => QuickScope::All,
                    };
                    self.rebuild_quick_items();
                }
                PickerAction::QueryChanged | PickerAction::Back | PickerAction::Scope(_) => {}
            }
        }
        result.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        PageFrame::new(self.title())
            .meta("")
            .draw_with(ui, area, |ui, body| {
                // Reference widgets frame the body. Draw them off-screen so the boot
                // card stays the approved playground and the widgets stay mounted.
                ui.reference(None, |ui| {
                    let hidden = Rect::new(body.x, body.bottom(), 0, 0);
                    filter_list().draw(ui, hidden, &self.filter_state, REFERENCE_ITEMS);
                    picker_chain().draw(ui, hidden, &self.chain_state);
                    menu_bar().draw(ui, hidden, &self.menu_state);
                    context_menu().draw(ui, hidden, &self.context_state);
                });
                ui.fill(body, ink(WHITE, CANVAS).remove_modifier(Modifier::all()));
                paint_blurb(ui, body);
                if body.height > 2 {
                    ui.fill(
                        Rect {
                            y: body.y.saturating_add(2),
                            height: 1,
                            ..body
                        },
                        ink(WHITE, CANVAS),
                    );
                }
                SectionStrip::new(&SECTIONS)
                    .caption(SECTION_CAPTION)
                    .caption_min_width(SECTION_WIDE)
                    .ink(chrome_ink())
                    .draw(
                        ui,
                        Rect {
                            height: 2.min(body.height),
                            ..body
                        },
                    );
                let top = body.y.saturating_add(3);
                if top < body.bottom() {
                    let level = LEVEL_ITEMS
                        .get(self.level)
                        .map_or("Safe Mode", |item| item.label);
                    let opened = self.opened.to_string();
                    paint_playground(
                        ui,
                        Rect {
                            y: top,
                            height: body.bottom().saturating_sub(top),
                            ..body
                        },
                        &self.result,
                        &self.detail,
                        level,
                        &opened,
                    );
                }
            });
        if let Some(kind) = self.open_kind {
            ui.layer(PICKER, |ui, layer| {
                let items = self.items(kind);
                picker(kind, items.len()).draw(ui, layer, &self.state, items);
                paint_picker(ui, layer, kind, &self.state, items, self.quick_scope);
            });
        }
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.open_kind.is_some() {
            &[("Esc", "Close")]
        } else {
            &[("Enter", "Open")]
        }
    }
}

pub fn action_keys() -> impl Iterator<Item = ActionKey> {
    MENU_ITEMS.iter().chain(CONTEXT_ITEMS).map(MenuItem::action)
}

#[cfg(test)]
mod tests {
    use super::{Page, PickersPage};
    use termrock::{App, Color, Cx, Modifier, Response, Theme, Ui};
    use termrock_test_support::Harness;

    struct Host;

    impl App for Host {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            PickersPage::new().draw(ui, ui.full());
        }
    }

    #[test]
    fn playground_matches_the_120_column_body() {
        let h = Harness::new(Host, Theme::junie(), 94, 36);
        let text = h.text();
        assert!(
            text.contains(
                "One modal list for files, tabs, levels, commands and branches: search, scope, status"
            ),
            "{text}"
        );
        assert!(text.contains("Open a picker"), "{text}");
        assert!(text.contains("Enter · click"), "{text}");
        assert!(text.contains("Remote branch"), "{text}");
        assert!(text.contains("read back from a built picker"), "{text}");
        assert!(text.contains("width 64 · max rows 12 · Center"), "{text}");
        assert!(text.contains("Chosen"), "{text}");
        assert!(text.contains("nothing yet"), "{text}");
        assert!(text.contains("PickerAction"), "{text}");
        assert!(text.contains("Pickers opened"), "{text}");

        let (x, y) = h.find("Open quickly").expect("primary");
        assert_eq!(h.cell(x, y).fg, Color::Rgb(0x19, 0x19, 0x1c));
        assert_eq!(h.cell(x, y).bg, Color::Rgb(0x48, 0xe0, 0x54));
        assert!(h.cell(x, y).modifier.contains(Modifier::BOLD));
        let gutter = h.cell(x.saturating_sub(1), y);
        assert_eq!(gutter.symbol(), " ");
        assert_eq!(gutter.bg, Color::Rgb(0x48, 0xe0, 0x54));
        assert!(!gutter.modifier.contains(Modifier::BOLD));

        let (sx, sy) = h.find("Size").expect("prop");
        assert_eq!(h.cell(sx, sy).fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(h.cell(sx + 10, sy).symbol(), "w");
    }

    #[test]
    fn narrow_playground_scrolls_and_ends_the_blurb_on_ellipsis() {
        let h = Harness::new(Host, Theme::junie(), 51, 16);
        assert_eq!(h.cell(49, 0).symbol(), "…");
        assert_eq!(h.cell(49, 0).fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(h.cell(50, 5).symbol(), "┃");
        assert_eq!(h.cell(50, 5).fg, Color::Rgb(0x80, 0x80, 0x80));
        assert_eq!(h.cell(50, 5).bg, Color::Rgb(0, 0, 0));
        assert_eq!(h.cell(50, 7).symbol(), "┆");
        assert_eq!(h.cell(50, 7).fg, Color::Rgb(0x47, 0x47, 0x47));
        assert_eq!(h.cell(49, 5).symbol(), " ");
        assert_eq!(h.cell(49, 5).bg, Color::Rgb(0, 0, 0));
        assert!(h.text().contains("retries"), "{}", h.text());
    }
}
