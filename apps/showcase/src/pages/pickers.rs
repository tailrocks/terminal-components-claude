//! Searchable semantic picker with query and scope state.

use junie_tui::{
    ActionKey, Button, ContextMenu, Cx, Family, FilterList, FilterListState, FrameRead, GlyphRole,
    Id, Item, ItemKey, LayerSize, Menu, MenuBar, MenuItem, MenuState, Modifier, Part, Picker,
    PickerAction, PickerChain, PickerChainState, PickerStage, PickerState, Position, Rect,
    Response, ScreenAlign, ScrollState, StateFlags, Style, Surface, Ui, Variant, id, width,
};

use super::{Page, frame};

const OPEN_QUICK: Id = id!("pickers.open.quick");
const OPEN_TABS: Id = id!("pickers.open.tabs");
const OPEN_LEVEL: Id = id!("pickers.open.level");
const PICKER: Id = id!("pickers.layer");
const FILTER: Id = id!("pickers.filter");
const CHAIN: Id = id!("pickers.chain");
const MENU: Id = id!("pickers.menu");
const CONTEXT: Id = id!("pickers.context");
const SCOPES: &[junie_tui::ScopeKey] = &[
    junie_tui::ScopeKey::new(1),
    junie_tui::ScopeKey::new(2),
    junie_tui::ScopeKey::new(3),
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
    MenuItem::new(ActionKey::custom("showcase.menu.open"), "Open")
        .chord(junie_tui::Chord::key(junie_tui::KeyCode::Char('o'))),
    MenuItem::new(ActionKey::custom("showcase.menu.close"), "Close"),
];
const MENUS: &[Menu<'static>] = &[Menu::new("Actions", MENU_ITEMS)];
const CONTEXT_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(ActionKey::custom("showcase.context.inspect"), "Inspect"),
    MenuItem::new(ActionKey::custom("showcase.context.copy"), "Copy path"),
];
const CHAIN_STAGES: &[PickerStage<'static>] = &[
    PickerStage::new(ItemKey::Num(301), "Scope"),
    PickerStage::new(ItemKey::Num(302), "Command"),
    PickerStage::new(ItemKey::Num(303), "Result"),
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
        picker.scopes(SCOPES)
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

fn paint_body(ui: &mut Ui<'_>, body: Rect, lines: &[&str]) {
    let mut canvas = ui.with_surface(Surface::Canvas, |ui| ui.surface_style());
    canvas.sub_modifier = Modifier::all();
    ui.fill(body, canvas);
    let mut surface = ui.with_surface(Surface::Surface, |ui| ui.surface_style());
    surface.sub_modifier = Modifier::all();
    let mut panel = ui.with_surface(Surface::Surface, |ui| {
        ui.style(
            Family::PANEL,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        )
        .style
    });
    panel.sub_modifier = Modifier::all();
    let top = Rect {
        height: body.height.min(7),
        ..body
    };
    ui.fill(top, surface);
    let result_y = body.y.saturating_add(8);
    let result = Rect {
        y: result_y,
        height: body.bottom().saturating_sub(result_y).min(8),
        ..body
    };
    ui.fill(result, surface);
    for (row, line) in lines.iter().enumerate() {
        let Ok(row) = u16::try_from(row) else {
            break;
        };
        if row > body.height {
            break;
        }
        let row_area = Rect {
            y: body.y.saturating_add(row),
            height: 1,
            ..body
        };
        if row >= 7 && !(8..16).contains(&row) {
            continue;
        }
        if let Some(rest) = line.strip_prefix("  ") {
            ui.paint_str(
                Rect {
                    width: 2,
                    ..row_area
                },
                "  ",
                panel,
            );
            ui.paint_str(
                Rect {
                    x: row_area.x.saturating_add(2),
                    width: row_area.width.saturating_sub(2),
                    ..row_area
                },
                rest,
                panel,
            );
        } else {
            ui.paint_str(row_area, line, panel);
        }
    }
}

fn style(
    ui: &mut Ui<'_>,
    surface: Surface,
    family: Family,
    variant: Variant,
    part: Part,
    flags: StateFlags,
) -> Style {
    ui.with_surface(surface, |ui| ui.style(family, variant, part, flags).style)
}

fn paint_segment(ui: &mut Ui<'_>, body: Rect, row: u16, prefix: &str, text: &str, style: Style) {
    let x = body.x.saturating_add(width(prefix));
    ui.paint_str(
        Rect {
            x,
            y: body.y.saturating_add(row),
            width: body.right().saturating_sub(x),
            height: 1,
        },
        text,
        style,
    );
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

fn picker_style(ui: &mut Ui<'_>, part: Part, flags: StateFlags) -> Style {
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
                char_style.add_modifier |= Modifier::BOLD;
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
                let detail = junie_tui::truncate(item.detail, room);
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

fn paint_historical(
    ui: &mut Ui<'_>,
    body: Rect,
    result: &str,
    detail_text: &str,
    level: &str,
    opened: u32,
) {
    let panel = style(
        ui,
        Surface::Surface,
        Family::PANEL,
        Variant::DEFAULT,
        Part::CONTAINER,
        StateFlags::empty(),
    );
    let title = style(
        ui,
        Surface::Surface,
        Family::PANEL,
        Variant::DEFAULT,
        Part::DETAIL,
        StateFlags::empty(),
    );
    let detail = style(
        ui,
        Surface::Surface,
        Family::PANEL,
        Variant::DEFAULT,
        Part::HELP,
        StateFlags::empty(),
    );
    let primary = style(
        ui,
        Surface::Surface,
        Family::BUTTON,
        Variant::PRIMARY,
        Part::CONTAINER,
        StateFlags::empty(),
    );
    let secondary = style(
        ui,
        Surface::Surface,
        Family::BUTTON,
        Variant::SECONDARY,
        Part::CONTAINER,
        StateFlags::empty(),
    );
    let primary_gutter = style(
        ui,
        Surface::Surface,
        Family::BUTTON,
        Variant::PRIMARY,
        Part::GUTTER,
        StateFlags::empty(),
    );
    let secondary_gutter = style(
        ui,
        Surface::Surface,
        Family::BUTTON,
        Variant::SECONDARY,
        Part::GUTTER,
        StateFlags::empty(),
    );
    let primary_gutter = primary_gutter.remove_modifier(Modifier::all());
    let primary_gutter = primary
        .bg
        .map_or(primary_gutter, |background| primary_gutter.fg(background));
    let secondary_gutter = secondary_gutter.remove_modifier(Modifier::all());
    let secondary_gutter = secondary.bg.map_or(secondary_gutter, |background| {
        secondary_gutter.fg(background)
    });

    paint_segment(ui, body, 0, "  ", "Open a picker", title);
    ui.fill(
        Rect {
            x: body.x.saturating_add(width("  ")),
            y: body.y.saturating_add(2),
            width: width("▎Open quickly "),
            height: 1,
        },
        primary,
    );
    ui.fill(
        Rect {
            x: body.x.saturating_add(width("  ▎Open quickly   ")),
            y: body.y.saturating_add(2),
            width: width("▎Switch tab "),
            height: 1,
        },
        secondary,
    );
    ui.fill(
        Rect {
            x: body
                .x
                .saturating_add(width("  ▎Open quickly   ▎Switch tab   ")),
            y: body.y.saturating_add(2),
            width: width("▎Choose a level "),
            height: 1,
        },
        secondary,
    );
    paint_segment(ui, body, 2, "  ", "▎", primary_gutter);
    paint_segment(ui, body, 2, "  ▎", "Open quickly", primary);
    paint_segment(ui, body, 2, "  ▎Open quickly   ", "▎", secondary_gutter);
    paint_segment(ui, body, 2, "  ▎Open quickly   ▎", "Switch tab", secondary);
    paint_segment(
        ui,
        body,
        2,
        "  ▎Open quickly   ▎Switch tab   ",
        "▎",
        secondary_gutter,
    );
    paint_segment(
        ui,
        body,
        2,
        "  ▎Open quickly   ▎Switch tab   ▎",
        "Choose a level",
        secondary,
    );
    paint_segment(
        ui,
        body,
        4,
        "  ",
        "Quick: fuzzy over files and tasks, Tab cycles the scope, Alt+Enter is the alternate actio…",
        detail,
    );
    paint_segment(ui, body, 8, "  ", "Result", title);
    paint_segment(ui, body, 10, "  ", "Chosen", detail);
    paint_segment(
        ui,
        body,
        10,
        "  Chosen          ",
        if result == "none" {
            "nothing yet"
        } else {
            result
        },
        panel,
    );
    paint_segment(ui, body, 11, "  ", "Detail", detail);
    paint_segment(
        ui,
        body,
        11,
        "  Detail          ",
        if detail_text.is_empty() {
            "—"
        } else {
            detail_text
        },
        panel,
    );
    paint_segment(ui, body, 12, "  ", "Level", detail);
    paint_segment(ui, body, 12, "  Level           ", level, panel);
    paint_segment(ui, body, 13, "  ", "Open tabs", detail);
    paint_segment(
        ui,
        body,
        13,
        "  Open tabs       ",
        "Query 1 · orders · order_items · History",
        panel,
    );
    paint_segment(ui, body, 14, "  ", "Pickers opened", detail);
    paint_segment(
        ui,
        body,
        14,
        "  Pickers opened  ",
        &opened.to_string(),
        panel,
    );
}

/// The picker owns query/cursor state while the app owns the selected result.
#[derive(Debug, Default)]
pub(crate) struct PickersPage {
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
    pub(crate) fn new() -> Self {
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
        if kind == PickerKind::Level {
            if let Some(item) = LEVEL_ITEMS.get(self.level) {
                self.state.set_cursor(self.level, item.key);
            }
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

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
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
        // Drain the layer owner on every pass. A dismissal event is addressed
        // to PICKER after the layer has left the active stack; gating update
        // on `is_open` would strand that event and trip the runtime diagnostic.
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
        result
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "One modal list for files, tabs and levels: search, scope, tag, alternate action",
            |ui, body| {
                quick_button().draw(
                    ui,
                    Rect {
                        x: body.x.saturating_add(width("  ")),
                        y: body.y,
                        width: body.width.saturating_sub(width("  ")),
                        height: 1,
                    },
                );
                tabs_button().draw(
                    ui,
                    Rect {
                        x: body.x.saturating_add(width("  ▎Open quickly   ")),
                        y: body.y.saturating_add(2),
                        width: body
                            .right()
                            .saturating_sub(body.x.saturating_add(width("  ▎Open quickly   "))),
                        height: 1,
                    },
                );
                level_button().draw(
                    ui,
                    Rect {
                        x: body
                            .x
                            .saturating_add(width("  ▎Open quickly   ▎Switch tab   ")),
                        y: body.y.saturating_add(2),
                        width: body.right().saturating_sub(
                            body.x
                                .saturating_add(width("  ▎Open quickly   ▎Switch tab   ")),
                        ),
                        height: 1,
                    },
                );
                ui.reference(None, |ui| {
                    filter_list().draw(ui, body, &self.filter_state, REFERENCE_ITEMS);
                    picker_chain().draw(ui, body, &self.chain_state);
                    menu_bar().draw(ui, body, &self.menu_state);
                    context_menu().draw(ui, body, &self.context_state);
                });
                paint_body(
                    ui,
                    body,
                    &[
                        "  Open a picker",
                        "",
                        "  ▎Open quickly   ▎Switch tab   ▎Choose a level",
                        "",
                        "  Quick: fuzzy over files and tasks, Tab cycles the scop…",
                        "",
                        "",
                        "",
                        "  Result",
                        "",
                        "  Chosen          nothing yet",
                        "  Detail          —",
                        "  Level           Safe Mode",
                        "  Open tabs       Query 1 · orders · order_items · History",
                        "  Pickers opened  0",
                        "",
                        "",
                        "",
                    ],
                );
                let level = LEVEL_ITEMS
                    .get(self.level)
                    .map_or("Safe Mode", |item| item.label);
                paint_historical(ui, body, &self.result, &self.detail, level, self.opened);
            },
        );
        if let Some(kind) = self.open_kind {
            ui.layer(PICKER, |ui, layer| {
                let items = self.items(kind);
                picker(kind, items.len()).draw(ui, layer, &self.state, items);
                paint_picker(ui, layer, kind, &self.state, items, self.quick_scope);
            });
        }
    }

    fn hints(&self, _ui: &Ui<'_>) -> Vec<(&'static str, &'static str)> {
        if self.open_kind.is_some() {
            vec![("Esc", "Close")]
        } else {
            vec![("Enter", "Open")]
        }
    }
}
