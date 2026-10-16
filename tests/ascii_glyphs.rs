//! `Glyphs::ASCII` reaches every widget family: rendered with it, no cell
//! holds a character outside 7-bit ASCII (api #25, #26).

use junie_tui::core::selection::SelectionMode;
use junie_tui::core::{focus::FocusRing, hit::HitRegistry, id::WidgetId};
use junie_tui::theme::{ColorLevel, Glyphs, LevelOverrides, Theme};
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::button::{Button, ButtonRow, RowOverflow};
use junie_tui::widgets::chips::{Chip, ChipBar};
use junie_tui::widgets::choice::{Checkbox, RadioGroup, Toggle};
use junie_tui::widgets::diff::{DiffFile, DiffHunk, DiffLine, DiffStatus, DiffView};
use junie_tui::widgets::grid::{CellKind, CellValue, ColumnSpec, DataGrid, GridRows, RowTotal};
use junie_tui::widgets::keyhint::{self, hint};
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::menu::{ContextMenu, MenuItem};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::progress::{self, ProgressStatus};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::select::Select;
use junie_tui::widgets::steps::{Step, StepRail, StepState};
use junie_tui::widgets::table::{Cell, Column, DataTable, SortDir};
use junie_tui::widgets::tabs::{TabItem, Tabs};
use junie_tui::widgets::tree::{TreeNode, TreeView};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect};

fn ascii() -> Theme {
    Theme::junie().with_glyphs(Glyphs::ASCII)
}

/// Render with `theme`, focus on `focus`, then flush overlays.
fn draw(
    theme: &Theme,
    size: (u16, u16),
    focus: Option<WidgetId>,
    f: impl FnOnce(&mut Buffer, &mut RenderCtx),
) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, size.0, size.1));
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut ctx = RenderCtx::new(
        theme,
        Interaction {
            focus,
            ..Default::default()
        },
        &mut hits,
        &mut ring,
    );
    f(&mut buf, &mut ctx);
    ctx.flush_overlays(&mut buf);
    buf
}

/// Every non-ASCII cell, with its position, for the failure message.
fn non_ascii(buf: &Buffer) -> Vec<(u16, u16, String)> {
    let a = buf.area;
    let mut out = vec![];
    for y in a.top()..a.bottom() {
        for x in a.left()..a.right() {
            let s = buf[(x, y)].symbol();
            if !s.is_ascii() {
                out.push((x, y, s.to_owned()));
            }
        }
    }
    out
}

fn rows(buf: &Buffer) -> String {
    let a = buf.area;
    (a.top()..a.bottom())
        .map(|y| {
            (a.left()..a.right())
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The same scene drawn with the default glyphs must contain Unicode (so
/// the test exercises real glyphs) and with ASCII must contain none.
fn check(
    name: &str,
    size: (u16, u16),
    focus: Option<WidgetId>,
    f: impl Fn(&mut Buffer, &mut RenderCtx),
) {
    let unicode = draw(&Theme::junie(), size, focus, &f);
    assert!(
        !non_ascii(&unicode).is_empty(),
        "{name}: the scene draws no glyph at all\n{}",
        rows(&unicode)
    );
    let buf = draw(&ascii(), size, focus, &f);
    let bad = non_ascii(&buf);
    assert!(bad.is_empty(), "{name}: {bad:?}\n{}", rows(&buf));
}

#[test]
fn choice_controls_draw_ascii() {
    let id = WidgetId::of("c");
    for w in [3u16, 20] {
        check("checkbox", (w, 3), Some(id), |buf, ctx| {
            let t = *ctx.theme;
            Checkbox::new(id, "On", true).render(Rect::new(0, 0, w, 1), buf, ctx, t.canvas);
            Checkbox::new(WidgetId::of("c2"), "Off", false).render(
                Rect::new(0, 1, w, 1),
                buf,
                ctx,
                t.canvas,
            );
            Checkbox::new(WidgetId::of("c3"), "Mixed", false)
                .mixed(true)
                .render(Rect::new(0, 2, w, 1), buf, ctx, t.canvas);
        });
        check("radio", (w, 3), Some(id), |buf, ctx| {
            let t = *ctx.theme;
            RadioGroup::new(id, "R", &["a", "b", "c"], 1).render(
                Rect::new(0, 0, w, 3),
                buf,
                ctx,
                t.canvas,
            );
        });
        check("toggle", (w, 2), Some(id), |buf, ctx| {
            let t = *ctx.theme;
            Toggle::new(id, "On", true).render(Rect::new(0, 0, w, 1), buf, ctx, t.canvas);
            Toggle::new(WidgetId::of("t2"), "Off", false).render(
                Rect::new(0, 1, w, 1),
                buf,
                ctx,
                t.canvas,
            );
        });
    }
}

#[test]
fn buttons_chips_and_tabs_draw_ascii() {
    let id = WidgetId::of("b");
    check("buttons", (40, 3), Some(id), |buf, ctx| {
        let t = *ctx.theme;
        Button::toggle(id, "Wrap", true).render(Rect::new(0, 0, 12, 1), buf, ctx, t.canvas);
        let mut busy = Button::primary(WidgetId::of("busy"), "Saving");
        busy.busy = true;
        busy.render(Rect::new(14, 0, 12, 1), buf, ctx, t.canvas);
        let mut row: Vec<Button> = (0..6)
            .map(|i| Button::secondary(WidgetId::of(&format!("r{i}")), "Action"))
            .collect();
        ButtonRow::default().overflow(RowOverflow::More).render(
            WidgetId::of("row"),
            Rect::new(0, 2, 30, 1),
            &mut row,
            buf,
            ctx,
            t.canvas,
        );
    });
    check("chips", (40, 1), Some(id), |buf, ctx| {
        let t = *ctx.theme;
        let mut bar = ChipBar::new(id);
        bar.chips = vec![Chip::new("name = x"), Chip::new("qty > 2")];
        bar.render(Rect::new(0, 0, 40, 1), buf, ctx, t.canvas);
    });
    check("tabs", (30, 2), Some(id), |buf, ctx| {
        let t = *ctx.theme;
        let mut tabs = Tabs::new(id, &[]);
        tabs.items = (0..8)
            .map(|i| {
                let mut it = TabItem::new(&format!("query {i}")).closable();
                it.dirty = i == 1;
                it.busy = i == 2;
                it
            })
            .collect();
        tabs.active = 3;
        tabs.render(Rect::new(0, 0, 30, 2), buf, ctx, t.canvas);
    });
}

#[test]
fn scrollbars_dividers_and_progress_draw_ascii() {
    use junie_tui::core::scroll::ScrollState;
    use junie_tui::ui::layout::SplitDir;
    use junie_tui::widgets::splitter::Splitter;
    let id = WidgetId::of("s");
    check("scrollbars", (20, 10), Some(id), |buf, ctx| {
        let mut s = ScrollState::default();
        s.set_content(100);
        s.set_viewport(8);
        s.offset = 10;
        scrollbar::render_vertical(Rect::new(0, 0, 1, 8), buf, ctx, id, &s, true);
        scrollbar::render_divider(
            Rect::new(2, 0, 1, 8),
            buf,
            ctx,
            WidgetId::of("d"),
            &ScrollState::default(),
            false,
        );
        scrollbar::render_horizontal(Rect::new(0, 9, 20, 1), buf, ctx, id, &s, true);
        let t = *ctx.theme;
        let mut sp = Splitter::new(WidgetId::of("sp"), SplitDir::Horizontal);
        sp.render(Rect::new(4, 0, 1, 8), buf, ctx, t.canvas);
        let mut sp = Splitter::new(WidgetId::of("sv"), SplitDir::Vertical);
        sp.render(Rect::new(6, 8, 10, 1), buf, ctx, t.canvas);
    });
    check("progress", (40, 6), None, |buf, ctx| {
        let t = *ctx.theme;
        for (i, st) in [
            ProgressStatus::Active,
            ProgressStatus::Done,
            ProgressStatus::Paused,
        ]
        .into_iter()
        .enumerate()
        {
            progress::render_bar(
                Rect::new(0, i as u16, 40, 1),
                buf,
                ctx,
                "copy",
                0.5,
                st,
                t.canvas,
            );
        }
        progress::render_indeterminate(Rect::new(0, 3, 40, 1), buf, ctx, "scan", t.canvas);
        progress::render_spinner(Rect::new(0, 4, 40, 1), buf, ctx, "Loading", t.canvas);
        progress::Meter::new(Some(80))
            .tone(progress::MeterTone::Warning)
            .render(Rect::new(0, 5, 40, 1), buf, ctx, t.canvas);
    });
    check("steps", (30, 3), Some(id), |buf, ctx| {
        let t = *ctx.theme;
        let mut steps = vec![Step::new("one"), Step::new("two"), Step::new("three")];
        steps[0].state = StepState::Done;
        steps[1].state = StepState::Running;
        StepRail::new(id, steps).render(Rect::new(0, 0, 30, 3), buf, ctx, t.canvas);
    });
}

#[test]
fn lists_menus_and_selects_draw_ascii() {
    let id = WidgetId::of("l");
    for mode in [SelectMode::Single, SelectMode::Multi] {
        check("list", (20, 4), Some(id), |buf, ctx| {
            let t = *ctx.theme;
            let items = (0..4)
                .map(|i| ListItem::new(&format!("item {i}")))
                .collect();
            let mut l = ListBox::new(id, items, mode);
            l.chosen = Some(0);
            l.checked = vec![true; 4];
            l.render(Rect::new(0, 0, 20, 4), buf, ctx, t.canvas);
        });
    }
    check("tree", (20, 4), Some(id), |buf, ctx| {
        let t = *ctx.theme;
        let mut tree = TreeView::new(
            id,
            vec![
                TreeNode::dir("open", vec![TreeNode::leaf("a")]),
                TreeNode::dir("closed", vec![TreeNode::leaf("b")]),
            ],
        );
        tree.toggle(0);
        tree.render(Rect::new(0, 0, 20, 4), buf, ctx, t.canvas);
    });
    check("menu", (30, 12), None, |buf, ctx| {
        let items = (0..12)
            .map(|i| MenuItem::new(format!("Item {i}")).separator_if(i == 2))
            .collect();
        let mut m = ContextMenu::new(id, items)
            .at(Position::new(1, 1))
            .max_rows(5);
        m.render(Rect::new(0, 0, 30, 12), buf, ctx);
    });
    check("select", (24, 10), Some(id), |buf, ctx| {
        let t = *ctx.theme;
        let opts: Vec<String> = (0..20).map(|i| format!("option {i}")).collect();
        let opts: Vec<&str> = opts.iter().map(String::as_str).collect();
        let mut s = Select::new(id, "Pick", &opts, 3);
        s.open = true;
        s.max_rows = 4;
        s.render(Rect::new(0, 0, 24, 2), buf, ctx, t.canvas);
    });
}

#[test]
fn tables_and_grids_draw_ascii() {
    let id = WidgetId::of("t");
    check("table", (40, 6), Some(id), |buf, ctx| {
        let t = *ctx.theme;
        let cols = (0..6)
            .map(|i| Column::new(&format!("col {i}"), Constraint::Length(10)))
            .collect();
        let rows = (0..4)
            .map(|r| (0..6).map(|c| Cell::new(format!("{r}.{c}"))).collect())
            .collect();
        let mut tb = DataTable::new(id, cols, rows).selection_mode(SelectionMode::Multi);
        tb.sort = Some((0, SortDir::Asc));
        tb.toggle_row(0);
        tb.render(Rect::new(0, 0, 40, 6), buf, ctx, t.canvas);
    });
    check("grid", (40, 8), Some(id), |buf, ctx| {
        let t = *ctx.theme;
        let cols = vec![
            ColumnSpec::new("id", CellKind::Id).primary(),
            ColumnSpec::new("name", CellKind::Text),
            ColumnSpec::new("qty", CellKind::Number),
            ColumnSpec::new("note", CellKind::Text),
            ColumnSpec::new("extra", CellKind::Text),
        ];
        let mut g = DataGrid::new(id, cols);
        g.set_rows(GridRows {
            rows: (0..4)
                .map(|i| {
                    vec![
                        CellValue::Text(format!("id-{i}")),
                        CellValue::Text("line\nbreak\ttab".into()),
                        CellValue::Int(i),
                        CellValue::Json("{\"k\": [1, 2, 3]}".into()),
                        CellValue::Text("x".into()),
                    ]
                })
                .collect(),
            total: RowTotal::Exact(400),
            more: false,
        });
        g.sort = Some((0, SortDir::Desc));
        g.filtered_cols.insert(1);
        g.selected_rows.insert(0);
        g.render(Rect::new(0, 0, 40, 8), buf, ctx, t.canvas);
    });
}

#[test]
fn text_fields_draw_ascii() {
    use junie_tui::widgets::input::TextInput;
    let id = WidgetId::of("i");
    check("masked input", (30, 3), Some(id), |buf, ctx| {
        let t = *ctx.theme;
        TextInput::new(id, "Secret")
            .value("hunter2")
            .masked()
            .render(Rect::new(0, 0, 30, 3), buf, ctx, t.canvas);
    });
}

#[test]
fn panels_dialogs_and_diffs_draw_ascii() {
    let id = WidgetId::of("p");
    check("panel", (30, 5), None, |buf, ctx| {
        let t = *ctx.theme;
        let p = Panel::framed(Some("Title")).meta("rows 1-10 of 40");
        let inner = p.render(Rect::new(0, 0, 30, 5), buf, &t);
        p.mark_clipped(inner, buf, &t);
    });
    check("dialog", (60, 20), None, |buf, ctx| {
        let mut d = junie_tui::widgets::dialog::Dialog::confirm(id, "Delete", "Sure?", "Delete");
        d.render(Rect::new(0, 0, 60, 20), buf, ctx);
    });
    check("diff", (80, 12), None, |buf, ctx| {
        let t = *ctx.theme;
        let mut v = DiffView::new(id);
        v.set_file(Some(DiffFile {
            path: "b.rs".into(),
            status: DiffStatus::Renamed {
                from: "a.rs".into(),
            },
            hunks: vec![
                DiffHunk {
                    old_start: 1,
                    new_start: 1,
                    lines: vec![DiffLine::remove("old"), DiffLine::add("new")],
                },
                DiffHunk {
                    old_start: 9,
                    new_start: 9,
                    lines: vec![DiffLine::context("same")],
                },
            ],
        }));
        v.mode = junie_tui::widgets::diff::DiffMode::Review;
        v.render(Rect::new(0, 0, 80, 12), buf, ctx, t.canvas);
    });
    check("hints", (40, 1), None, |buf, ctx| {
        let t = *ctx.theme;
        keyhint::render_toned(
            Rect::new(0, 0, 40, 1),
            buf,
            &t,
            &[hint("↑↓", "Move"), hint("1-6", "Jump"), hint("Esc", "Back")],
            None,
            Some(("disk low", junie_tui::theme::Tone::Warning)),
        );
    });
}

#[test]
fn ascii_mono_override_switches_only_mono() {
    let o = LevelOverrides::ASCII_MONO;
    assert_eq!(
        Theme::for_level_with(ColorLevel::Mono, &o).glyphs,
        Glyphs::ASCII
    );
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
    ] {
        assert_eq!(Theme::for_level_with(level, &o).glyphs, Glyphs::UNICODE);
    }
    assert_eq!(Theme::junie().glyphs, Glyphs::UNICODE);
}

#[test]
fn ascii_glyphs_keep_the_unicode_widths() {
    use unicode_width::UnicodeWidthStr;
    let (u, a) = (Glyphs::UNICODE, Glyphs::ASCII);
    let pairs = [
        (u.focus_bar, a.focus_bar),
        (u.current, a.current),
        (u.ellipsis, a.ellipsis),
        (u.separator, a.separator),
        (u.check_on, a.check_on),
        (u.check_mixed, a.check_mixed),
        (u.radio_on, a.radio_on),
        (u.toggle_on, a.toggle_on),
        (u.toggle_off, a.toggle_off),
        (u.scroll_held, a.scroll_held),
        (u.key, a.key),
        (u.arrow, a.arrow),
        (u.mask, a.mask),
    ];
    for (x, y) in pairs {
        assert_eq!(x.width(), y.width(), "{x:?} vs {y:?}");
        assert!(y.is_ascii());
    }
    assert!(a.spinner.iter().all(|f| f.is_ascii() && f.width() == 1));
}
