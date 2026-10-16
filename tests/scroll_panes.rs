//! Every scrolling widget reaches its bars through one path: the pane it
//! embeds (`ScrollPane`) behind `Scrollable`. For each widget, drawn with
//! more rows (or columns) than fit, a press on the bare end of each bar
//! pages toward the pointer, and a press on the thumb grabs it so a drag
//! to the far end of the track reaches the end of the content. The bar is
//! found the way the pointer layer finds it: from the hits of the frame.
//!
//! Before S3, Picker, Completion, TextArea, HelpOverlay, InfoDialog and
//! FormDialog drew bars that lit up on hover but had no handler (mouse
//! audit M17).

use junie_tui::core::event::Outcome;
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::hit::{Axis, Bar, HitRegistry};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::{ScrollState, Scrollable};
use junie_tui::theme::Theme;
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::code::CodeEditor;
use junie_tui::widgets::completion::{Completion, CompletionItem};
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::form_dialog::{FormDialog, FormField};
use junie_tui::widgets::grid::{CellKind, CellValue, ColumnSpec, DataGrid, GridRows, RowTotal};
use junie_tui::widgets::help::HelpOverlay;
use junie_tui::widgets::info_dialog::InfoDialog;
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::panel::ScrollPanel;
use junie_tui::widgets::picker::{Picker, PickerItem};
use junie_tui::widgets::props::{Prop, PropsList};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::steps::{Step, StepRail};
use junie_tui::widgets::table::{Cell, Column, DataTable};
use junie_tui::widgets::textarea::TextArea;
use junie_tui::widgets::tree::{TreeNode, TreeView};
use junie_tui::widgets::viewport::{Span, TextViewport};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};

const SCREEN: Rect = Rect::new(0, 0, 60, 16);

/// Draw one frame through `draw` (overlays flushed) and return its hits.
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

/// The bar the pointer layer would grab for `owner` on `axis`.
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

/// Press the far end of the track: one page toward the pointer. Then
/// grab the thumb and drag it past the far end: the end of the content.
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
    // back to the top, then grab the thumb and drag it to the far end
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

/// A notch of any size moves one column; a table or grid whose columns
/// all fit leaves the notch to its owner.
#[test]
fn tables_and_grids_step_one_column_per_sideways_notch() {
    let mut t = wide_table();
    frame(|a, b, c| t.render(a, b, c, Theme::junie().surface));
    assert_eq!(t.on_wheel_h(3), Outcome::Changed);
    assert_eq!(t.scroll.h.offset, 1);
    assert_eq!(t.on_wheel_h(-3), Outcome::Changed);
    assert_eq!(t.scroll.h.offset, 0);
    assert_eq!(t.on_wheel_h(-1), Outcome::Consumed, "at the first column");

    let mut g = wide_grid();
    frame(|a, b, c| g.render(a, b, c, Theme::junie().surface));
    assert_eq!(g.on_wheel_h(3), Outcome::Changed);
    assert_eq!(g.scroll.h.offset, 1);

    // every column fits: the notch is not the table's
    let cols = vec![Column::new("a", Constraint::Length(4))];
    let mut narrow = DataTable::new(WidgetId::of("n"), cols, vec![vec![Cell::new("x")]]);
    frame(|a, b, c| narrow.render(a, b, c, Theme::junie().surface));
    assert_eq!(narrow.on_wheel_h(1), Outcome::Ignored);
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
    w.set_items(
        labels(50)
            .iter()
            .map(|l| PickerItem::new(l.clone()))
            .collect(),
    );
    let hits = frame(|a, b, c| w.draw(a, b, c));
    press_and_drag("Picker", &mut w, bar(&hits, id, Axis::Vertical));

    let id = WidgetId::of("completion");
    let mut w = Completion::new(id);
    let items = labels(30)
        .into_iter()
        .map(|l| CompletionItem {
            insert: l.clone(),
            label: l,
            glyph: "T".into(),
            detail: String::new(),
            matched: vec![],
        })
        .collect();
    w.open(items, Rect::new(2, 1, 1, 1), 0);
    let hits = frame(|a, b, c| w.render(a, b, c));
    press_and_drag("Completion", &mut w, bar(&hits, id, Axis::Vertical));
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

/// A click routed to a modal's own bar id pages its body: the info dialog
/// mapped it through a track that was not the one drawn, and the form
/// dialog ignored it.
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

/// A notch that cannot move the view is consumed without a repaint, in
/// the help overlay, the info dialog and the form dialog alike (they
/// reported a change for every notch).
#[test]
fn a_boundary_notch_in_a_modal_body_is_consumed_without_a_repaint() {
    let mut w = help();
    frame(|a, b, c| w.render(a, b, c));
    assert_eq!(w.on_wheel(-1), Outcome::Consumed, "HelpOverlay at the top");
    assert_eq!(w.on_wheel(1), Outcome::Changed);

    let mut w = info();
    frame(|a, b, c| w.render(a, b, c));
    assert_eq!(w.on_wheel(-1), Outcome::Consumed, "InfoDialog at the top");
    assert_eq!(w.on_wheel(1), Outcome::Changed);

    let mut w = form();
    frame(|a, b, c| w.render(a, b, c));
    assert_eq!(w.on_wheel(-1), Outcome::Consumed, "FormDialog at the top");
    assert_eq!(w.on_wheel(1), Outcome::Changed);
}

/// The last row a pane shows, after drawing.
fn last_visible(w: &mut dyn Scrollable) -> usize {
    let v = state(w, Axis::Vertical);
    assert!(v.overflows());
    v.offset + v.viewport_len - 1
}

/// A click on the last visible row puts the cursor there and leaves the
/// view alone: the default row of context (`ScrollState::context`, SVg9)
/// is for keyboard moves. Pointer moves reveal exactly
/// (`ScrollState::pointer_move`), so the content never shifts under the
/// pointer between the two clicks of a double click.
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
    // a keyboard move to the next row keeps one row of context below it
    g.on_key(&junie_tui::core::event::Key {
        code: ratatui::crossterm::event::KeyCode::Down,
        mods: ratatui::crossterm::event::KeyModifiers::NONE,
    });
    let v = g.scroll.v;
    assert_eq!(v.offset + v.viewport_len - 1, row + 2, "DataGrid keyboard");
}
