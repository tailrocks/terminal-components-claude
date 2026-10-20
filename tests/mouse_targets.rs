use junie_tui::core::focus::FocusRing;
use junie_tui::core::hit::{Axis, HitRegistry};
use junie_tui::core::id::WidgetId;
use junie_tui::core::intent::Phase;
use junie_tui::core::pointer::{MouseTarget, PointerEvent};
use junie_tui::theme::{ButtonKind, ColorLevel, Theme, Tone};
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::breadcrumb::Breadcrumb;
use junie_tui::widgets::button::Button;
use junie_tui::widgets::chips::{Chip, ChipBar};
use junie_tui::widgets::choice::{Checkbox, RadioGroup, Toggle};
use junie_tui::widgets::code::CodeEditor;
use junie_tui::widgets::completion::Completion;
use junie_tui::widgets::grid::{CellKind, CellValue, ColumnSpec, DataGrid, GridRows, RowTotal};
use junie_tui::widgets::help::HelpOverlay;
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::menu::{ContextMenu, MenuBar, MenuItem};
use junie_tui::widgets::option_list::Item;
use junie_tui::widgets::panel::ScrollPanel;
use junie_tui::widgets::picker::Picker;
use junie_tui::widgets::props::{Prop, PropsList};
use junie_tui::widgets::rail::SidebarRail;
use junie_tui::widgets::select::Select;
use junie_tui::widgets::steps::{Step, StepRail};
use junie_tui::widgets::table::{Cell, Column, DataTable};
use junie_tui::widgets::tabs::{TabItem, Tabs};
use junie_tui::widgets::textarea::TextArea;
use junie_tui::widgets::toolbar::{ToolItem, Toolbar};
use junie_tui::widgets::tree::{TreeNode, TreeView};
use junie_tui::widgets::viewport::{Span, TextViewport};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::{Constraint, Position, Rect};

const SCREEN: Rect = Rect::new(0, 0, 60, 20);
const NONE: KeyModifiers = KeyModifiers::NONE;

fn hits_of(draw: impl FnOnce(&mut Buffer, &mut RenderCtx)) -> HitRegistry {
    let theme = Theme::for_level(ColorLevel::TrueColor);
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut buf = Buffer::empty(SCREEN);
    let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
    draw(&mut buf, &mut ctx);
    ctx.flush_overlays(&mut buf);
    drop(ctx);
    hits
}

fn check<T: MouseTarget>(name: &str, w: &mut T, hits: &HitRegistry, wheel: Option<Position>) {
    let me = w.mouse_id();

    if let Some(pos) = wheel {
        let id = hits
            .scroll_stack(pos)
            .into_iter()
            .find(|id| hits.part_of(*id).is_some_and(|h| h.owner == me) || *id == me)
            .unwrap_or_else(|| panic!("{name}: nothing scrolls at {pos:?}"));
        let ev = PointerEvent::wheel_on(hits, id, Axis::Vertical, 1, pos, NONE);
        let r = w
            .route(&ev)
            .unwrap_or_else(|| panic!("{name}: wheel not routed"));
        assert!(r.is_consumed(), "{name}: the wheel is taken");
    }
    let parts: Vec<(WidgetId, Position)> = hits
        .regions()
        .iter()
        .filter(|r| !r.scroll_only && !r.hover_only && r.owner == me)
        .map(|r| (r.id, Position::new(r.area.x, r.area.y)))
        .collect();
    assert!(!parts.is_empty(), "{name}: no clickable part");
    for (id, pos) in parts {
        for phase in [Phase::Press, Phase::Click] {
            let ev = PointerEvent::new(hits, phase, id, pos, 1, NONE);
            assert_eq!(ev.owner(), me, "{name}: {id:?} resolves to its widget");
            assert!(w.route(&ev).is_some(), "{name}: {phase:?} on {id:?}");
        }
    }
    let other = PointerEvent::new(
        hits,
        Phase::Click,
        WidgetId::of("other"),
        Position::ORIGIN,
        1,
        NONE,
    );
    assert!(w.route(&other).is_none(), "{name}: another owner's event");
}

fn many(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("item {i:02}")).collect()
}

#[test]
fn collections_route_their_parts_and_take_the_wheel() {
    let area = Rect::new(0, 0, 40, 5);
    let wheel = Some(Position::new(2, 2));
    let bg = |ctx: &RenderCtx| ctx.theme.canvas;

    let items = many(30).iter().map(|s| ListItem::new(s)).collect();
    let mut list = ListBox::new(WidgetId::of("mt.list"), items, SelectMode::Single);
    let hits = hits_of(|buf, ctx| list.render(area, buf, ctx, bg(ctx)));
    check("list", &mut list, &hits, wheel);

    let leaves = many(30).iter().map(|s| TreeNode::leaf(s)).collect();
    let mut tree = TreeView::new(WidgetId::of("mt.tree"), vec![TreeNode::dir("dir", leaves)]);
    tree.expand_all();
    let hits = hits_of(|buf, ctx| tree.render(area, buf, ctx, bg(ctx)));
    check("tree", &mut tree, &hits, wheel);

    let props = many(30).iter().map(|s| Prop::new(s, "v")).collect();
    let mut props = PropsList::new(WidgetId::of("mt.props"), props);
    let hits = hits_of(|buf, ctx| props.render(area, buf, ctx, bg(ctx)));
    check("props", &mut props, &hits, wheel);

    let steps = many(30).iter().map(|s| Step::new(s)).collect();
    let mut steps = StepRail::new(WidgetId::of("mt.steps"), steps).selectable(true);
    let hits = hits_of(|buf, ctx| steps.render(area, buf, ctx, bg(ctx)));
    check("steps", &mut steps, &hits, wheel);

    let rows = many(30)
        .into_iter()
        .map(|s| vec![Cell::new(&s), Cell::new("1")])
        .collect();
    let cols = vec![
        Column::new("name", Constraint::Length(10)),
        Column::new("size", Constraint::Length(10)),
    ];
    let mut table = DataTable::new(WidgetId::of("mt.table"), cols, rows);
    let hits = hits_of(|buf, ctx| table.render(area, buf, ctx, bg(ctx)));
    check("table", &mut table, &hits, wheel);

    let cols = vec![
        ColumnSpec::new("name", CellKind::Text),
        ColumnSpec::new("n", CellKind::Number),
    ];
    let mut grid = DataGrid::new(WidgetId::of("mt.grid"), cols);
    grid.set_rows(GridRows {
        rows: (0..30)
            .map(|i| vec![CellValue::Text(format!("r{i}")), CellValue::Int(i)])
            .collect(),
        total: RowTotal::Exact(30),
        more: false,
    });
    let hits = hits_of(|buf, ctx| grid.render(Rect::new(0, 0, 40, 8), buf, ctx, bg(ctx)));
    check("grid", &mut grid, &hits, Some(Position::new(8, 4)));

    let mut panel = ScrollPanel::new(WidgetId::of("mt.panel"), many(30));
    let hits = hits_of(|buf, ctx| {
        panel.render(area, buf, ctx, bg(ctx), |t, _| t.muted());
    });
    check("panel", &mut panel, &hits, wheel);

    let mut view = TextViewport::new(WidgetId::of("mt.view"));
    view.set_lines(
        many(30)
            .into_iter()
            .map(|s| vec![Span::new(s, Tone::Normal)])
            .collect(),
    );
    let hits = hits_of(|buf, ctx| view.render(area, buf, ctx, bg(ctx)));
    check("viewport", &mut view, &hits, wheel);
}

#[test]
fn strips_and_controls_route_their_parts() {
    let row = Rect::new(0, 0, 50, 3);
    let bg = |ctx: &RenderCtx| ctx.theme.canvas;

    let items = vec![TabItem::new("one").closable(), TabItem::new("two")];
    let mut tabs = Tabs::with_items(WidgetId::of("mt.tabs"), items).allow_new(true);
    let hits = hits_of(|buf, ctx| tabs.render(row, buf, ctx, bg(ctx)));
    check("tabs", &mut tabs, &hits, None);

    let mut chips = ChipBar::new(WidgetId::of("mt.chips"));
    chips.chips = vec![Chip::new("a"), Chip::new("b")];
    let hits = hits_of(|buf, ctx| chips.render(row, buf, ctx, bg(ctx)));
    check("chips", &mut chips, &hits, None);

    let segs = vec!["a".into(), "b".into(), "c".into()];
    let mut crumbs = Breadcrumb::new(WidgetId::of("mt.crumbs"), segs);
    let hits = hits_of(|buf, ctx| crumbs.render(row, buf, ctx, bg(ctx)));
    check("breadcrumb", &mut crumbs, &hits, None);

    let tools = vec![ToolItem::new("Run"), ToolItem::new("Stop")];
    let mut toolbar = Toolbar::new(WidgetId::of("mt.toolbar"), tools);
    let hits = hits_of(|buf, ctx| toolbar.render(row, buf, ctx, bg(ctx)));
    check("toolbar", &mut toolbar, &hits, None);

    let menus = vec![
        ("File", vec![MenuItem::new("Open")]),
        ("Edit", vec![MenuItem::new("Copy")]),
    ];
    let mut bar = MenuBar::new(WidgetId::of("mt.menubar"), menus);
    let hits = hits_of(|buf, ctx| bar.render(Rect::new(0, 0, 50, 1), buf, ctx, bg(ctx)));
    check("menubar", &mut bar, &hits, None);

    let mut radio = RadioGroup::new(WidgetId::of("mt.radio"), "pick", &["a", "b"], 0);
    let hits = hits_of(|buf, ctx| radio.render(Rect::new(0, 0, 30, 4), buf, ctx, bg(ctx)));
    check("radio", &mut radio, &hits, None);

    let mut cb = Checkbox::new(WidgetId::of("mt.check"), "Check", false);
    let hits = hits_of(|buf, ctx| cb.render(row, buf, ctx, bg(ctx)));
    check("checkbox", &mut cb, &hits, None);

    let mut toggle = Toggle::new(WidgetId::of("mt.toggle"), "Toggle", false);
    let hits = hits_of(|buf, ctx| toggle.render(row, buf, ctx, bg(ctx)));
    check("toggle", &mut toggle, &hits, None);

    let mut rail = SidebarRail::new(WidgetId::of("mt.rail"), "Files");
    let hits = hits_of(|buf, ctx| rail.render_rail(Rect::new(0, 0, 20, 8), buf, ctx, bg(ctx)));
    check("rail", &mut rail, &hits, None);

    let mut button = Button::new(WidgetId::of("mt.button"), "Go", ButtonKind::Primary);
    let hits = hits_of(|buf, ctx| button.render(row, buf, ctx, bg(ctx)));
    check("button", &mut button, &hits, None);
}

#[test]
fn text_fields_route_their_gestures() {
    let bg = |ctx: &RenderCtx| ctx.theme.canvas;

    let mut input = TextInput::new(WidgetId::of("mt.input"), "Name");
    let hits = hits_of(|buf, ctx| input.render(Rect::new(0, 0, 30, 2), buf, ctx, bg(ctx)));
    check("input", &mut input, &hits, None);

    let mut area = TextArea::new(WidgetId::of("mt.area"), "Notes", 3).value(&many(30).join("\n"));
    let hits = hits_of(|buf, ctx| area.render(Rect::new(0, 0, 30, 5), buf, ctx, bg(ctx)));
    check("textarea", &mut area, &hits, Some(Position::new(3, 2)));

    let mut code = CodeEditor::new(WidgetId::of("mt.code"), &many(30).join("\n"));
    let hits = hits_of(|buf, ctx| code.render(Rect::new(0, 0, 40, 5), buf, ctx, bg(ctx)));
    check("code", &mut code, &hits, Some(Position::new(8, 2)));
}

#[test]
fn popups_route_their_rows_and_backdrops() {
    let options = many(30);
    let options: Vec<&str> = options.iter().map(String::as_str).collect();
    let mut select = Select::new(WidgetId::of("mt.select"), "Kind", &options, 0).max_rows(5);
    select.open = true;
    let hits = hits_of(|buf, ctx| {
        let bg = ctx.theme.canvas;
        select.render(Rect::new(0, 0, 30, 2), buf, ctx, bg);
    });
    let row = hits
        .area_of(select.option_id(1))
        .expect("the open list's rows");
    check(
        "select",
        &mut select,
        &hits,
        Some(Position::new(row.x + 1, row.y)),
    );

    let items = many(30).iter().map(|s| MenuItem::new(s.as_str())).collect();
    let mut menu = ContextMenu::new(WidgetId::of("mt.menu"), items).at(Position::new(2, 1));
    let hits = hits_of(|buf, ctx| menu.render(SCREEN, buf, ctx));
    let row = hits.area_of(menu.row_id(1)).expect("the menu's rows");
    check(
        "menu",
        &mut menu,
        &hits,
        Some(Position::new(row.x + 1, row.y)),
    );

    let mut picker = Picker::new(WidgetId::of("mt.picker"), "Go to");
    picker.list.max_rows = 5;
    picker.set_items(many(30).iter().map(|s| Item::new(s.as_str())).collect());
    let hits = hits_of(|buf, ctx| picker.draw(SCREEN, buf, ctx));
    let row = hits.area_of(picker.row_id(1)).expect("the picker's rows");
    check(
        "picker",
        &mut picker,
        &hits,
        Some(Position::new(row.x + 1, row.y)),
    );

    let mut completion = Completion::new(WidgetId::of("mt.completion"));
    let items = many(30)
        .into_iter()
        .map(|s| Item::new(s).glyph("T"))
        .collect();
    completion.open(items, Rect::new(2, 1, 1, 1), 0);
    let hits = hits_of(|buf, ctx| completion.render(SCREEN, buf, ctx));
    let row = hits
        .regions()
        .iter()
        .find(|r| r.owner == completion.id && !r.scroll_only)
        .map(|r| r.area)
        .expect("the completion's rows");
    check(
        "completion",
        &mut completion,
        &hits,
        Some(Position::new(row.x + 1, row.y)),
    );

    let keys: Vec<String> = many(40);
    let section = keys.iter().map(|k| (k.as_str(), "does a thing")).collect();
    let mut help = HelpOverlay::new(WidgetId::of("mt.help"), "Page", vec![("Keys", section)]);
    let hits = hits_of(|buf, ctx| help.render(SCREEN, buf, ctx));
    check("help", &mut help, &hits, Some(Position::new(30, 8)));
}
