use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::hit::{Axis, Bar, HitRegistry};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::{ScrollState, Scrollable};
use junie_tui::theme::Theme;
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::code::CodeEditor;
use junie_tui::widgets::completion::Completion;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::form_dialog::{FormDialog, FormField};
use junie_tui::widgets::grid::{CellKind, CellValue, ColumnSpec, DataGrid, GridRows, RowTotal};
use junie_tui::widgets::help::HelpOverlay;
use junie_tui::widgets::info_dialog::InfoDialog;
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::option_list::Item;
use junie_tui::widgets::panel::ScrollPanel;
use junie_tui::widgets::picker::Picker;
use junie_tui::widgets::props::{Prop, PropsList};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::steps::{Step, StepRail};
use junie_tui::widgets::table::{Cell, Column, DataTable};
use junie_tui::widgets::textarea::TextArea;
use junie_tui::widgets::tree::{TreeNode, TreeView};
use junie_tui::widgets::viewport::{Span, TextViewport};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Position, Rect};

const SCREEN: Rect = Rect::new(0, 0, 60, 16);

fn frame(draw: impl FnOnce(Rect, &mut Buffer, &mut RenderCtx)) -> HitRegistry {
    let theme = Theme::junie();
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut buf = Buffer::empty(SCREEN);
    {
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        draw(SCREEN, &mut buf, &mut ctx);
        ctx.flush_overlays(&mut buf);
    }
    hits
}

fn bar(hits: &HitRegistry, owner: WidgetId, axis: Axis) -> Bar {
    let id = match axis {
        Axis::Vertical => scrollbar::id_for(owner),
        Axis::Horizontal => scrollbar::h_id_for(owner),
    };
    let bar = hits
        .bar(id)
        .unwrap_or_else(|| panic!("{owner:?}: no {axis:?} bar"));
    assert_eq!(bar.axis, axis);
    bar
}

fn state(w: &mut dyn Scrollable, axis: Axis) -> ScrollState {
    *w.pane().axis(axis)
}

fn press_and_drag(name: &str, w: &mut dyn Scrollable, bar: Bar) {
    let axis = bar.axis;
    let t = bar.track;
    let (far, len) = match axis {
        Axis::Vertical => (Position::new(t.x, t.bottom() - 1), t.height as usize),
        Axis::Horizontal => (Position::new(t.right() - 1, t.y), t.width as usize),
    };
    let before = state(w, axis);
    assert!(before.overflows(), "{name}: the content must overflow");
    assert_eq!(before.offset, 0, "{name}");
    let r = w.bar_input(&bar, far, true);
    assert!(r.is_changed(), "{name}: a press on the bare track moves");
    let paged = state(w, axis);
    assert_eq!(
        paged.offset,
        before.viewport_len.min(before.max_offset()),
        "{name}: one page toward the pointer (TrackClick::Page)"
    );
    w.pane().release();

    w.pane().axis(axis).scroll_to(0);
    let (start, _) = state(w, axis).thumb(len);
    let thumb = match axis {
        Axis::Vertical => Position::new(t.x, t.y + start as u16),
        Axis::Horizontal => Position::new(t.x + start as u16, t.y),
    };
    let r = w.bar_input(&bar, thumb, true);
    assert!(
        !r.is_changed(),
        "{name}: a press on the thumb only grabs it"
    );
    let past = match axis {
        Axis::Vertical => Position::new(t.x, t.bottom() + 3),
        Axis::Horizontal => Position::new(t.right() + 3, t.y),
    };
    let r = w.bar_input(&bar, past, false);
    assert!(r.is_changed(), "{name}: the drag moves the view");
    assert!(state(w, axis).at_end(), "{name}: dragged to the end");
    w.pane().release();
}

fn labels(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("row {i}")).collect()
}

#[test]
fn row_widgets_page_and_drag_their_bars() {
    let id = WidgetId::of("list");
    let items = labels(50).iter().map(|l| ListItem::new(l)).collect();
    let mut w = ListBox::new(id, items, SelectMode::Single);
    let hits = frame(|a, b, c| w.render(a, b, c, Theme::junie().surface));
    press_and_drag("ListBox", &mut w, bar(&hits, id, Axis::Vertical));

    let id = WidgetId::of("tree");
    let nodes = labels(50).iter().map(|l| TreeNode::leaf(l)).collect();
    let mut w = TreeView::new(id, nodes);
    let hits = frame(|a, b, c| w.render(a, b, c, Theme::junie().surface));
    press_and_drag("TreeView", &mut w, bar(&hits, id, Axis::Vertical));

    let id = WidgetId::of("props");
    let props = labels(50)
        .iter()
        .map(|l| Prop::new("Key", l.clone()))
        .collect();
    let mut w = PropsList::new(id, props);
    let hits = frame(|a, b, c| w.render(a, b, c, Theme::junie().surface));
    press_and_drag("PropsList", &mut w, bar(&hits, id, Axis::Vertical));

    let id = WidgetId::of("steps");
    let steps = labels(50).iter().map(|l| Step::new(l)).collect();
    let mut w = StepRail::new(id, steps);
    let hits = frame(|a, b, c| w.render(a, b, c, Theme::junie().surface));
    press_and_drag("StepRail", &mut w, bar(&hits, id, Axis::Vertical));
}

fn wide_table() -> DataTable {
    let cols = (0..9)
        .map(|i| Column::new(&format!("col{i}"), Constraint::Length(10)))
        .collect();
    let rows = (0..50)
        .map(|r| (0..9).map(|c| Cell::new(format!("r{r}c{c}"))).collect())
        .collect();
    DataTable::new(WidgetId::of("table"), cols, rows)
}

fn wide_grid() -> DataGrid {
    let cols = (0..9)
        .map(|i| ColumnSpec::new(&format!("column {i}"), CellKind::Text))
        .collect();
    let mut g = DataGrid::new(WidgetId::of("grid"), cols);
    let rows = (0..50)
        .map(|r| {
            (0..9)
                .map(|c| CellValue::Text(format!("value r{r} c{c}")))
                .collect()
        })
        .collect();
    g.set_rows(GridRows {
        rows,
        total: RowTotal::Exact(50),
        more: false,
    });
    g
}

#[test]
fn tables_and_grids_page_and_drag_both_bars() {
    for axis in [Axis::Vertical, Axis::Horizontal] {
        let mut t = wide_table();
        let hits = frame(|a, b, c| t.render(a, b, c, Theme::junie().surface));
        let b = bar(&hits, t.id, axis);
        press_and_drag("DataTable", &mut t, b);

        let mut g = wide_grid();
        let hits = frame(|a, b, c| g.render(a, b, c, Theme::junie().surface));
        let b = bar(&hits, g.id, axis);
        press_and_drag("DataGrid", &mut g, b);
    }
}

#[test]
fn tables_and_grids_step_one_column_per_sideways_notch() {
    let mut t = wide_table();
    frame(|a, b, c| t.render(a, b, c, Theme::junie().surface));
    assert_eq!(t.wheel(Axis::Horizontal, 3).outcome(), Outcome::Changed);
    assert_eq!(t.scroll.h.offset, 1);
    assert_eq!(t.wheel(Axis::Horizontal, -3).outcome(), Outcome::Changed);
    assert_eq!(t.scroll.h.offset, 0);
    assert_eq!(
        t.wheel(Axis::Horizontal, -1).outcome(),
        Outcome::Consumed,
        "at the first column"
    );

    let mut g = wide_grid();
    frame(|a, b, c| g.render(a, b, c, Theme::junie().surface));
    assert_eq!(g.wheel(Axis::Horizontal, 3).outcome(), Outcome::Changed);
    assert_eq!(g.scroll.h.offset, 1);

    let cols = vec![Column::new("a", Constraint::Length(4))];
    let mut narrow = DataTable::new(WidgetId::of("n"), cols, vec![vec![Cell::new("x")]]);
    frame(|a, b, c| narrow.render(a, b, c, Theme::junie().surface));
    assert_eq!(
        narrow.wheel(Axis::Horizontal, 1).outcome(),
        Outcome::Ignored
    );
}

fn screen(draw: impl FnOnce(Rect, &mut Buffer, &mut RenderCtx)) -> Vec<String> {
    let theme = Theme::junie();
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut buf = Buffer::empty(SCREEN);
    {
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        draw(SCREEN, &mut buf, &mut ctx);
        ctx.flush_overlays(&mut buf);
    }
    (0..SCREEN.height)
        .map(|y| {
            (0..SCREEN.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect()
        })
        .collect()
}

const STEP_WIDTHS: [u16; 10] = [6, 6, 6, 6, 6, 6, 30, 30, 40, 70];

fn right() -> Key {
    Key {
        code: KeyCode::Right,
        mods: KeyModifiers::NONE,
    }
}

fn assert_cursor_column_whole(name: &str, offset: usize, fit: usize, cursor: usize, top: &str) {
    assert!(
        offset <= cursor && (cursor < offset + fit || cursor == offset),
        "{name}: cursor column {cursor} outside columns {offset}..{}\n{top}",
        offset + fit
    );
    let label = format!("h{cursor}x");
    assert!(top.contains(&label), "{name}: {label} not drawn\n{top}");
}

#[test]
fn moving_right_keeps_the_table_cursor_column_whole() {
    let cols = STEP_WIDTHS
        .iter()
        .enumerate()
        .map(|(i, &w)| Column::new(&format!("h{i}x"), Constraint::Length(w)))
        .collect();
    let rows = vec![
        (0..STEP_WIDTHS.len())
            .map(|c| Cell::new(format!("v{c}")))
            .collect(),
    ];
    let mut t = DataTable::new(WidgetId::of("table"), cols, rows).cell_nav(true);
    screen(|a, b, c| t.render(a, b, c, Theme::junie().surface));
    for _ in 1..STEP_WIDTHS.len() {
        t.on_key(&right());
        let lines = screen(|a, b, c| t.render(a, b, c, Theme::junie().surface));
        let h = t.scroll.h;
        assert_cursor_column_whole(
            "DataTable",
            h.offset,
            h.viewport_len,
            t.cursor_col,
            &lines[0],
        );
    }
    assert_eq!(
        t.scroll.h.offset,
        STEP_WIDTHS.len() - 1,
        "wider than the view: its start"
    );
}

#[test]
fn moving_right_keeps_the_grid_cursor_column_whole() {
    let cols = (0..STEP_WIDTHS.len())
        .map(|i| ColumnSpec::new(&format!("h{i}x"), CellKind::Text))
        .collect();
    let mut g = DataGrid::new(WidgetId::of("grid"), cols);
    let row = STEP_WIDTHS
        .iter()
        .map(|&w| CellValue::Text("w".repeat(w as usize)))
        .collect();
    g.set_rows(GridRows {
        rows: vec![row],
        total: RowTotal::Exact(1),
        more: false,
    });
    screen(|a, b, c| g.render(a, b, c, Theme::junie().surface));
    for _ in 1..STEP_WIDTHS.len() {
        g.on_key(&right());
        let lines = screen(|a, b, c| g.render(a, b, c, Theme::junie().surface));
        let h = g.scroll.h;
        assert_cursor_column_whole("DataGrid", h.offset, h.viewport_len, g.cursor.1, &lines[0]);
    }
}

#[test]
fn a_sideways_notch_after_a_move_is_not_undone() {
    let mut t = wide_table().cell_nav(true);
    screen(|a, b, c| t.render(a, b, c, Theme::junie().surface));
    t.on_key(&right());
    screen(|a, b, c| t.render(a, b, c, Theme::junie().surface));
    assert_eq!(t.wheel(Axis::Horizontal, 1).outcome(), Outcome::Changed);
    screen(|a, b, c| t.render(a, b, c, Theme::junie().surface));
    assert_eq!(t.scroll.h.offset, 1);
    assert_eq!(t.cursor_col, 1);
    assert_eq!(t.wheel(Axis::Horizontal, 1).outcome(), Outcome::Changed);
    screen(|a, b, c| t.render(a, b, c, Theme::junie().surface));
    assert_eq!(t.scroll.h.offset, 2, "the cursor column may leave the view");
}

fn long_lines(n: usize) -> Vec<String> {
    (0..n)
        .map(|i| format!("line {i} {}", "abcdefghij".repeat(12)))
        .collect()
}

#[test]
fn text_surfaces_page_and_drag_both_bars() {
    for axis in [Axis::Vertical, Axis::Horizontal] {
        let id = WidgetId::of("code");
        let mut w = CodeEditor::new(id, &long_lines(50).join("\n"));
        let hits = frame(|a, b, c| w.render(a, b, c, Theme::junie().surface));
        press_and_drag("CodeEditor", &mut w, bar(&hits, id, axis));
        assert_eq!(
            w.hscroll, w.scroll.h.offset,
            "the caret's sideways position follows the bar"
        );

        let id = WidgetId::of("viewport");
        let lines = long_lines(50)
            .into_iter()
            .map(|l| vec![Span::plain(l)])
            .collect();
        let mut w = TextViewport::with_lines(id, lines);
        w.set_follow(false);
        let hits = frame(|a, b, c| w.render(a, b, c, Theme::junie().surface));
        press_and_drag("TextViewport", &mut w, bar(&hits, id, axis));

        let id = WidgetId::of("panel");
        let mut w = ScrollPanel::new(id, long_lines(50));
        let hits =
            frame(|a, b, c| w.render(a, b, c, Theme::junie().surface, |_, _| Default::default()));
        press_and_drag("ScrollPanel", &mut w, bar(&hits, id, axis));
    }
}

#[test]
fn the_text_area_pages_and_drags_its_bar() {
    let id = WidgetId::of("notes");
    let mut w = TextArea::new(id, "Notes", 6).value(&labels(40).join("\n"));
    let hits = frame(|a, b, c| w.render(a, b, c, Theme::junie().surface));
    press_and_drag("TextArea", &mut w, bar(&hits, id, Axis::Vertical));
}

#[test]
fn popup_lists_page_and_drag_their_bars() {
    let id = WidgetId::of("picker");
    let mut w = Picker::new(id, "Pick");
    w.set_items(labels(50).iter().map(|l| Item::new(l.clone())).collect());
    let hits = frame(|a, b, c| w.draw(a, b, c));
    press_and_drag("Picker", &mut w.list, bar(&hits, id, Axis::Vertical));

    let id = WidgetId::of("completion");
    let mut w = Completion::new(id);
    let items = labels(30)
        .into_iter()
        .map(|l| Item::new(l).glyph("T"))
        .collect();
    w.open(items, Rect::new(2, 1, 1, 1), 0);
    let hits = frame(|a, b, c| w.render(a, b, c));
    press_and_drag("Completion", &mut w.list, bar(&hits, id, Axis::Vertical));
}

fn help() -> HelpOverlay {
    let rows: Vec<(String, String)> = (0..60)
        .map(|i| (format!("k{i}"), format!("action {i}")))
        .collect();
    let rows: Vec<(&str, &str)> = rows.iter().map(|(k, a)| (k.as_str(), a.as_str())).collect();
    HelpOverlay::new(WidgetId::of("help"), "Scope", vec![("Keys", rows)])
}

fn info() -> InfoDialog {
    InfoDialog::new(
        WidgetId::of("info"),
        "About",
        vec![Prop::new("Key", "value")],
    )
    .detail(labels(40))
}

fn form() -> FormDialog {
    let fields = (0..20)
        .map(|i| {
            let name = format!("field{i}");
            FormField::input(&name, TextInput::new(WidgetId::of("form").child(i), &name))
        })
        .collect();
    FormDialog::new(WidgetId::of("form"), "Settings", fields)
}

fn dialog() -> Dialog {
    Dialog::confirm(WidgetId::of("dialog"), "Read", &labels(60).join("\n"), "OK")
}

#[test]
fn modal_bodies_page_and_drag_their_bars() {
    let mut w = help();
    let hits = frame(|a, b, c| w.render(a, b, c));
    let b = bar(&hits, w.id, Axis::Vertical);
    press_and_drag("HelpOverlay", &mut w, b);

    let mut w = info();
    let hits = frame(|a, b, c| w.render(a, b, c));
    let b = bar(&hits, w.id, Axis::Vertical);
    press_and_drag("InfoDialog", &mut w, b);

    let mut w = form();
    let hits = frame(|a, b, c| w.render(a, b, c));
    let b = bar(&hits, w.id, Axis::Vertical);
    press_and_drag("FormDialog", &mut w, b);

    let mut w = dialog();
    let hits = frame(|a, b, c| w.render(a, b, c));
    let b = bar(&hits, w.id, Axis::Vertical);
    press_and_drag("Dialog", &mut w, b);
}

#[test]
fn a_click_on_a_modal_bar_pages_the_body_it_was_drawn_for() {
    let mut focus = Focus::default();
    let mut w = info();
    let hits = frame(|a, b, c| w.render(a, b, c));
    let b = bar(&hits, w.id, Axis::Vertical);
    let view = w.detail_scroll.viewport_len;
    w.on_click(
        b.id,
        Position::new(b.track.x, b.track.bottom() - 1),
        &mut focus,
    );
    assert_eq!(w.detail_scroll.offset, view, "one page, not a jump");

    let mut w = form();
    let hits = frame(|a, b, c| w.render(a, b, c));
    let b = bar(&hits, w.id, Axis::Vertical);
    let view = w.scroll.viewport_len;
    let o = w.on_click(
        b.id,
        Position::new(b.track.x, b.track.bottom() - 1),
        &mut focus,
    );
    assert_eq!(o, Outcome::Changed);
    assert_eq!(w.scroll.offset, view.min(w.scroll.max_offset()));
}

#[test]
fn a_boundary_notch_in_a_modal_body_is_consumed_without_a_repaint() {
    let mut w = help();
    frame(|a, b, c| w.render(a, b, c));
    assert_eq!(
        w.wheel(Axis::Vertical, -1).outcome(),
        Outcome::Consumed,
        "HelpOverlay at the top"
    );
    assert_eq!(w.wheel(Axis::Vertical, 1).outcome(), Outcome::Changed);

    let mut w = info();
    frame(|a, b, c| w.render(a, b, c));
    assert_eq!(
        w.wheel(Axis::Vertical, -1).outcome(),
        Outcome::Consumed,
        "InfoDialog at the top"
    );
    assert_eq!(w.wheel(Axis::Vertical, 1).outcome(), Outcome::Changed);

    let mut w = form();
    frame(|a, b, c| w.render(a, b, c));
    assert_eq!(
        w.wheel(Axis::Vertical, -1).outcome(),
        Outcome::Consumed,
        "FormDialog at the top"
    );
    assert_eq!(w.wheel(Axis::Vertical, 1).outcome(), Outcome::Changed);
}

fn last_visible(w: &mut dyn Scrollable) -> usize {
    let v = state(w, Axis::Vertical);
    assert!(v.overflows());
    v.offset + v.viewport_len - 1
}

#[test]
fn a_click_on_the_last_visible_row_never_scrolls_it_away() {
    let bg = Theme::junie().surface;

    let items = labels(50).iter().map(|l| ListItem::new(l)).collect();
    let mut w = ListBox::new(WidgetId::of("list"), items, SelectMode::Single);
    frame(|a, b, c| w.render(a, b, c, bg));
    let row = last_visible(&mut w);
    w.on_click(row);
    assert_eq!((w.cursor, w.scroll.offset), (row, 0), "ListBox");

    let nodes = labels(50).iter().map(|l| TreeNode::leaf(l)).collect();
    let mut w = TreeView::new(WidgetId::of("tree"), nodes);
    frame(|a, b, c| w.render(a, b, c, bg));
    let row = last_visible(&mut w);
    w.on_click_row(row);
    assert_eq!((w.cursor, w.scroll.offset), (row, 0), "TreeView");

    let props = labels(50)
        .iter()
        .map(|l| Prop::new("Key", l.clone()))
        .collect();
    let mut w = PropsList::new(WidgetId::of("props"), props);
    frame(|a, b, c| w.render(a, b, c, bg));
    let row = last_visible(&mut w);
    w.on_click(row);
    assert_eq!((w.cursor, w.scroll.offset), (row, 0), "PropsList");

    let steps = labels(50).iter().map(|l| Step::new(l)).collect();
    let mut w = StepRail::new(WidgetId::of("steps"), steps).selectable(true);
    frame(|a, b, c| w.render(a, b, c, bg));
    let row = last_visible(&mut w);
    w.on_click(row);
    assert_eq!((w.cursor, w.scroll.offset), (row, 0), "StepRail");

    let mut t = wide_table();
    frame(|a, b, c| t.render(a, b, c, bg));
    let row = last_visible(&mut t);
    t.on_click_cell(row, 0, Position::new(0, 0));
    assert_eq!((t.cursor_row, t.scroll.offset), (row, 0), "DataTable");

    let mut g = wide_grid();
    frame(|a, b, c| g.render(a, b, c, bg));
    let row = last_visible(&mut g);
    let id = g.cell_id(row, 0);
    g.on_click(id, Position::new(0, 0));
    assert_eq!((g.cursor.0, g.scroll.offset), (row, 0), "DataGrid");

    g.on_key(&junie_tui::core::event::Key {
        code: ratatui::crossterm::event::KeyCode::Down,
        mods: ratatui::crossterm::event::KeyModifiers::NONE,
    });
    let v = g.scroll.v;
    assert_eq!(v.offset + v.viewport_len - 1, row + 2, "DataGrid keyboard");
}
