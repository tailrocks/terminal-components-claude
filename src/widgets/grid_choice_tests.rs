//! [`DataGrid`] constrained column editors ([`ColumnEditor`]): the popup
//! opens on the cell, commits into the pending-change queue, and follows
//! the grid's validator.

use super::*;
use crate::core::{focus::FocusRing, hit::HitRegistry};
use crate::theme::Theme;
use crate::ui::ctx::Interaction;
use crate::ui::popup::backdrop_id;
use crate::widgets::option_editor::Options;
use ratatui::crossterm::event::KeyModifiers;

fn key(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

/// id · plan (Select over 6 plans) · tier (Segmented) · beta (Checkbox)
fn grid() -> DataGrid {
    let plans = Options::values(&["free", "team", "pro", "enterprise", "edu", "nonprofit"]);
    let cols = vec![
        ColumnSpec::new("id", CellKind::Id).primary(),
        ColumnSpec::new("plan", CellKind::Enum).editor(ColumnEditor::for_options(plans)),
        ColumnSpec::new("tier", CellKind::Enum)
            .editor(ColumnEditor::for_options(Options::values(&["S", "M", "L"]))),
        ColumnSpec::new("beta", CellKind::Text)
            .editor(ColumnEditor::for_options(Options::values(&["no", "yes"]))),
    ];
    let mut g = DataGrid::new(WidgetId::of("g"), cols);
    let rows = vec![
        vec![
            CellValue::Text("1".into()),
            CellValue::Text("team".into()),
            CellValue::Text("M".into()),
            CellValue::Text("no".into()),
        ],
        vec![
            CellValue::Text("2".into()),
            CellValue::Text("legacy-gold".into()), // imported: outside the options
            CellValue::Text("S".into()),
            CellValue::Text("yes".into()),
        ],
    ];
    g.set_rows(GridRows {
        rows,
        total: RowTotal::Exact(2),
        more: false,
    });
    g.cursor = (0, 1);
    g
}

fn draw(g: &mut DataGrid) -> (Buffer, HitRegistry) {
    let theme = Theme::junie();
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let inter = Interaction {
        focus: Some(g.id),
        ..Default::default()
    };
    let mut buf = Buffer::empty(Rect::new(0, 0, 70, 16));
    {
        let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
        g.render(Rect::new(0, 0, 70, 8), &mut buf, &mut ctx, theme.canvas);
        ctx.flush_overlays(&mut buf);
    }
    (buf, hits)
}

fn text(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_select_cell_commits_into_the_pending_queue() {
    let mut g = grid();
    assert!(
        matches!(g.columns[1].editor, ColumnEditor::Select(_)),
        "6 values: a dropdown"
    );
    assert_eq!(g.on_key(&key(KeyCode::Enter)), (Outcome::Changed, None));
    assert!(
        g.is_editing() && g.edit.is_none(),
        "a popup, not a text editor"
    );
    let (buf, _) = draw(&mut g);
    assert!(text(&buf).contains("enterprise"), "{}", text(&buf));
    g.on_key(&key(KeyCode::Down));
    let (_, ev) = g.on_key(&key(KeyCode::Enter));
    assert_eq!(ev, Some(GridEvent::CellChanged { row: 0, col: 1 }));
    assert_eq!(g.value(0, 1), &CellValue::Text("pro".into()));
    assert!(
        g.pending.is_dirty(0, 1),
        "a pending change, as typed edits are"
    );
    // Esc cancels without a change
    g.on_key(&key(KeyCode::Enter));
    g.on_key(&key(KeyCode::Home));
    assert_eq!(g.on_key(&key(KeyCode::Esc)), (Outcome::Changed, None));
    assert_eq!(g.value(0, 1), &CellValue::Text("pro".into()));
}

#[test]
fn alt_down_and_the_mouse_open_and_choose() {
    let mut g = grid();
    let alt_down = Key {
        code: KeyCode::Down,
        mods: KeyModifiers::ALT,
    };
    g.on_key(&alt_down);
    assert!(g.choice.is_some());
    let (_, hits) = draw(&mut g);
    let opt = g.choice.as_ref().unwrap().editor.option_id(0);
    assert!(hits.area_of(opt).is_some(), "options are clickable");
    let (_, ev) = g.on_click(opt, Position::new(0, 0));
    assert_eq!(ev, Some(GridEvent::CellChanged { row: 0, col: 1 }));
    assert_eq!(g.value(0, 1), &CellValue::Text("free".into()));
    // the ▾ on the cursor cell opens in one click; outside cancels
    let (_, hits) = draw(&mut g);
    let mark = g.open_id(0, 1);
    assert!(hits.area_of(mark).is_some(), "▾ on the cursor cell");
    assert!(g.owns(mark));
    g.on_click(mark, Position::new(0, 0));
    assert!(g.choice.is_some());
    g.on_click(backdrop_id(g.choice_id()), Position::new(0, 0));
    assert!(g.choice.is_none());
}

#[test]
fn a_checkbox_flips_and_tab_opens_the_next_constrained_cell() {
    let mut g = grid();
    g.cursor = (0, 3);
    let (_, ev) = g.on_key(&key(KeyCode::Enter));
    assert_eq!(ev, Some(GridEvent::CellChanged { row: 0, col: 3 }));
    assert_eq!(g.value(0, 3), &CellValue::Text("yes".into()));
    g.cursor = (0, 1);
    g.on_key(&key(KeyCode::Enter));
    let (_, ev) = g.on_key(&key(KeyCode::Tab));
    assert_eq!(ev, Some(GridEvent::CellChanged { row: 0, col: 1 }));
    assert_eq!(g.cursor.1, 2);
    assert!(g.choice.is_some(), "the segmented tier opened");
    let (buf, _) = draw(&mut g);
    assert!(text(&buf).contains("S │ M │ L"), "{}", text(&buf));
}

#[test]
fn an_out_of_set_value_is_flagged_and_the_validator_still_decides() {
    let mut g = grid();
    assert!(g.out_of_set(1, 1));
    assert!(!g.out_of_set(0, 1));
    let (buf, _) = draw(&mut g);
    let rows = text(&buf);
    let y = rows
        .lines()
        .position(|l| l.contains("legacy"))
        .expect("shown as stored");
    let x = rows.lines().nth(y).unwrap().find("legacy").unwrap();
    // byte index equals the cell column on this ASCII row prefix
    assert_eq!(
        buf[(x as u16, y as u16)].fg,
        Theme::junie().error,
        "flagged"
    );
    // a validator that rejects a plan keeps the popup open
    let mut g = grid();
    g.validator = Validator::new(|_, v| {
        if v == "edu" {
            Err("Not for this account".into())
        } else {
            Ok(CellValue::Text(v.into()))
        }
    });
    g.on_key(&key(KeyCode::Enter));
    for _ in 0..3 {
        g.on_key(&key(KeyCode::Down));
    }
    assert_eq!(g.on_key(&key(KeyCode::Enter)).1, None);
    assert_eq!(g.edit_error(), Some("Not for this account"));
    // blur cancels by default
    assert_eq!(g.on_blur(), (Outcome::Changed, None));
    assert!(!g.is_editing());
}
