//! Part-tagged hits (shared-components plan S1): every region a widget
//! registers resolves to that widget and a typed part
//! ([`HitRegistry::resolve`]), never to a derived id the caller has to
//! invert by hand.

use junie_tui::core::hit::{Hit, HitRegistry};
use junie_tui::core::id::{Part, PartRef, WidgetId};
use junie_tui::core::{focus::FocusRing, id::ItemKey};
use junie_tui::theme::{ColorLevel, Theme};
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::breadcrumb::Breadcrumb;
use junie_tui::widgets::chips::{Chip, ChipBar};
use junie_tui::widgets::choice::RadioGroup;
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::menu::{MenuBar, MenuItem};
use junie_tui::widgets::props::{Prop, PropsList};
use junie_tui::widgets::steps::{Step, StepRail};
use junie_tui::widgets::table::{Cell, Column, DataTable};
use junie_tui::widgets::tabs::{TabItem, Tabs};
use junie_tui::widgets::tree::{TreeNode, TreeView};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};

/// Render one widget alone and return the frame's hit registry.
fn hits_of(area: Rect, draw: impl FnOnce(&mut Buffer, &mut RenderCtx)) -> HitRegistry {
    let theme = Theme::for_level(ColorLevel::TrueColor);
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut buf = Buffer::empty(area);
    let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
    draw(&mut buf, &mut ctx);
    ctx.flush_overlays(&mut buf);
    drop(ctx);
    hits
}

/// Every clickable region resolves to `owner`, and `parts` all occur.
fn assert_parts(name: &str, hits: &HitRegistry, owner: WidgetId, parts: &[PartRef]) {
    let mut seen = vec![];
    for r in hits
        .regions()
        .iter()
        .filter(|r| !r.scroll_only && !r.hover_only)
    {
        assert_eq!(r.owner, owner, "{name}: {r:?} belongs to another owner");
        // a 2-D part (a cell, its `▾`) keeps one id per column
        if !matches!(r.part.part, Part::CELL | Part::ACTIONS) {
            assert_eq!(
                r.id,
                owner.part(r.part),
                "{name}: {r:?} is not its part's id"
            );
        }
        let at = Position::new(r.area.x, r.area.y);
        let hit = hits.resolve(at).expect("a registered region resolves");
        assert_eq!(hit.owner, owner, "{name}: the hit at {at:?}");
        seen.push(r.part);
    }
    for p in parts {
        assert!(seen.contains(p), "{name}: no {p:?} among {seen:?}");
    }
}

fn row(i: usize) -> PartRef {
    PartRef::item(Part::ROW, ItemKey::index(i))
}

fn container() -> PartRef {
    PartRef::of(Part::CONTAINER)
}

#[test]
fn list_tree_props_and_steps_rows_resolve_to_their_widget() {
    let area = Rect::new(0, 0, 40, 8);
    let id = WidgetId::of("parts.list");
    let mut list = ListBox::new(
        id,
        vec![ListItem::new("one"), ListItem::new("two")],
        SelectMode::Single,
    );
    let hits = hits_of(area, |buf, ctx| {
        let bg = ctx.theme.canvas;
        list.render(area, buf, ctx, bg);
    });
    assert_parts("list", &hits, id, &[container(), row(0), row(1)]);
    assert_eq!(
        hits.resolve(Position::new(5, 1)),
        Some(Hit {
            owner: id,
            part: row(1)
        })
    );
    assert_eq!(list.locate(list.row_id(1)), Some(1));
    assert!(list.owns(list.row_id(0)));

    let id = WidgetId::of("parts.tree");
    let mut tree = TreeView::new(id, vec![TreeNode::dir("dir", vec![TreeNode::leaf("leaf")])]);
    tree.expand_all();
    let hits = hits_of(area, |buf, ctx| {
        let bg = ctx.theme.canvas;
        tree.render(area, buf, ctx, bg);
    });
    let icon = PartRef::item(Part::ICON, ItemKey::index(0));
    assert_parts("tree", &hits, id, &[row(0), row(1), icon]);
    assert_eq!(tree.locate(tree.toggle_id(0)), Some((0, true)));
    assert_eq!(tree.locate(tree.row_id(1)), Some((1, false)));

    let id = WidgetId::of("parts.props");
    let mut props = PropsList::new(id, vec![Prop::new("a", "1"), Prop::new("b", "2")]);
    let hits = hits_of(area, |buf, ctx| {
        let bg = ctx.theme.canvas;
        props.render(area, buf, ctx, bg);
    });
    assert_parts("props", &hits, id, &[row(0), row(1)]);

    let id = WidgetId::of("parts.steps");
    let mut steps = StepRail::new(id, vec![Step::new("one"), Step::new("two")]).selectable(true);
    let hits = hits_of(area, |buf, ctx| {
        let bg = ctx.theme.canvas;
        steps.render(area, buf, ctx, bg);
    });
    assert_parts("steps", &hits, id, &[row(0), row(1)]);
}

#[test]
fn strip_parts_resolve_to_their_widget() {
    let area = Rect::new(0, 0, 60, 3);
    let id = WidgetId::of("parts.tabs");
    let mut tabs = Tabs::with_items(
        id,
        vec![TabItem::new("one").closable(), TabItem::new("two")],
    )
    .allow_new(true);
    let hits = hits_of(area, |buf, ctx| {
        let bg = ctx.theme.canvas;
        tabs.render(area, buf, ctx, bg);
    });
    let tab = |i| PartRef::item(Part::TAB, ItemKey::index(i));
    let close = PartRef::item(Part::CLOSE, ItemKey::index(0));
    assert_parts(
        "tabs",
        &hits,
        id,
        &[tab(0), tab(1), close, PartRef::of(Part::NEW)],
    );
    assert_eq!(tabs.locate(tabs.tab_id(1)), Some(1));

    let id = WidgetId::of("parts.chips");
    let mut chips = ChipBar::new(id);
    chips.chips = vec![Chip::new("a"), Chip::new("b")];
    let hits = hits_of(area, |buf, ctx| {
        let bg = ctx.theme.canvas;
        chips.render(area, buf, ctx, bg);
    });
    assert_parts("chips", &hits, id, &[row(0), row(1)]);

    let id = WidgetId::of("parts.crumbs");
    let mut crumbs = Breadcrumb::new(id, vec!["a".into(), "b".into(), "c".into()]);
    let hits = hits_of(area, |buf, ctx| {
        let bg = ctx.theme.canvas;
        crumbs.render(area, buf, ctx, bg);
    });
    assert_parts("breadcrumb", &hits, id, &[row(0), row(1)]);
    assert_eq!(crumbs.on_click(crumbs.segment_id(1)), Some(1));

    let id = WidgetId::of("parts.radio");
    let mut radio = RadioGroup::new(id, "pick", &["a", "b"], 0);
    let hits = hits_of(Rect::new(0, 0, 30, 4), |buf, ctx| {
        let bg = ctx.theme.canvas;
        radio.render(Rect::new(0, 0, 30, 4), buf, ctx, bg);
    });
    assert_parts("radio", &hits, id, &[row(0), row(1)]);

    let id = WidgetId::of("parts.menubar");
    let mut bar = MenuBar::new(
        id,
        vec![
            ("File", vec![MenuItem::new("Open")]),
            ("Edit", vec![MenuItem::new("Copy")]),
        ],
    );
    let hits = hits_of(area, |buf, ctx| {
        let bg = ctx.theme.canvas;
        bar.render(Rect::new(0, 0, 60, 1), buf, ctx, bg);
    });
    assert_parts("menubar", &hits, id, &[tab(0), tab(1)]);
}

#[test]
fn table_rows_cells_and_headers_resolve_to_the_table() {
    let area = Rect::new(0, 0, 40, 6);
    let id = WidgetId::of("parts.table");
    let mut table = DataTable::new(
        id,
        vec![
            Column::new("name", Constraint::Length(10)),
            Column::new("size", Constraint::Length(10)),
        ],
        vec![
            vec![Cell::new("a"), Cell::new("1")],
            vec![Cell::new("b"), Cell::new("2")],
        ],
    );
    let hits = hits_of(area, |buf, ctx| {
        let bg = ctx.theme.canvas;
        table.render(area, buf, ctx, bg);
    });
    let header = PartRef::item(Part::HEADER, ItemKey::index(0));
    let cell = PartRef::item(Part::CELL, ItemKey::index(1));
    assert_parts("table", &hits, id, &[header, row(0), cell]);
    // a cell keeps its own id (hover per cell) and names its column
    for r in hits.regions().iter().filter(|r| r.part == cell) {
        assert_eq!(r.owner, id);
    }
    let cid = table.cell_id(1, 1);
    assert_eq!(table.locate(cid), Some((1, Some(1))));
    assert_eq!(table.locate(table.row_id(0)), Some((0, None)));
    assert_eq!(table.locate_header(table.header_id(1)), Some(1));
}
