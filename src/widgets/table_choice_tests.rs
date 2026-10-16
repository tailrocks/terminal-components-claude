//! [`DataTable`] constrained column editors ([`ColumnEditor`]) driven by
//! keys, clicks and rendered cells.

use super::*;
use crate::core::{focus::FocusRing, hit::HitRegistry};
use crate::theme::Theme;
use crate::ui::ctx::Interaction;
use crate::ui::popup::backdrop_id;
use crate::widgets::option_editor::{OptionItem, Options};

fn key(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

fn envs() -> Options {
    Options::new([
        OptionItem::new("dev").label("Development"),
        OptionItem::new("stage").label("Staging"),
        OptionItem::new("prod")
            .label("Production")
            .tone(Tone::Warning),
        OptionItem::new("dr")
            .label("Recovery")
            .disabled("Read-only"),
    ])
}

/// Name · Env (Select) · Notify (Checkbox) · Labels (Tags) · Tier (Segmented)
fn table() -> DataTable {
    let columns = vec![
        Column::new("Name", Constraint::Length(8)).editable(),
        Column::new("Env", Constraint::Length(14)).editor(ColumnEditor::Select(envs())),
        Column::new("Notify", Constraint::Length(7))
            .editor(ColumnEditor::Checkbox(Options::values(&["off", "on"]))),
        Column::new("Labels", Constraint::Length(12))
            .editor(ColumnEditor::Tags(Options::values(&["api", "web", "db"]))),
        Column::new("Tier", Constraint::Length(6))
            .editor(ColumnEditor::Segmented(Options::values(&["S", "M", "L"]))),
    ];
    let rows = vec![
        vec![
            Cell::new("alpha"),
            Cell::new("stage"),
            Cell::new("on"),
            Cell::new("api"),
            Cell::new("M"),
        ],
        vec![
            Cell::new("beta"),
            Cell::new("qa"), // imported: outside the options
            Cell::new("off"),
            Cell::new(""),
            Cell::new("S"),
        ],
    ];
    let mut t = DataTable::new(WidgetId::of("tbl"), columns, rows).cell_nav(true);
    t.cursor_col = 1;
    t
}

fn draw(t: &mut DataTable, hover: Option<WidgetId>) -> (Buffer, HitRegistry) {
    let theme = Theme::junie();
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let inter = Interaction {
        focus: Some(t.id),
        hover,
        ..Default::default()
    };
    let mut buf = Buffer::empty(Rect::new(0, 0, 60, 14));
    {
        let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
        t.render(Rect::new(0, 0, 60, 8), &mut buf, &mut ctx, theme.canvas);
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
fn a_select_cell_opens_chooses_and_commits_by_keyboard() {
    let mut t = table();
    let (buf, _) = draw(&mut t, None);
    assert!(
        text(&buf).contains("Staging"),
        "the label, not the stored value"
    );
    assert_eq!(t.on_key(&key(KeyCode::Enter)), (Outcome::Changed, None));
    assert!(t.is_editing() && t.choice.is_some());
    let (buf, _) = draw(&mut t, None);
    assert!(
        text(&buf).contains("Production"),
        "the popup lists the options"
    );
    t.on_key(&key(KeyCode::Down));
    let (out, ev) = t.on_key(&key(KeyCode::Enter));
    assert_eq!(out, Outcome::Changed);
    assert_eq!(ev, Some(TableEvent::Committed { row: 0, col: 1 }));
    assert_eq!(t.rows[0][1].text, "prod");
    assert!(!t.is_editing());
    // Esc cancels and keeps the value
    t.on_key(&key(KeyCode::F(2)));
    t.on_key(&key(KeyCode::Up));
    assert_eq!(t.on_key(&key(KeyCode::Esc)).1, Some(TableEvent::Cancelled));
    assert_eq!(t.rows[0][1].text, "prod");
}

#[test]
fn space_opens_type_ahead_jumps_and_a_disabled_option_is_never_chosen() {
    let mut t = table();
    t.on_key(&key(KeyCode::Char(' ')));
    assert!(t.choice.is_some(), "Space opens a constrained cell");
    t.on_key(&key(KeyCode::Char('p')));
    assert_eq!(t.choice.as_ref().unwrap().editor.cursor, 2, "type-ahead");
    t.on_key(&key(KeyCode::Down));
    assert_eq!(
        t.choice.as_ref().unwrap().editor.cursor,
        2,
        "the disabled option is skipped"
    );
    t.on_key(&key(KeyCode::Char('r')));
    assert_eq!(
        t.choice.as_ref().unwrap().editor.cursor,
        2,
        "nor reached by name"
    );
    let id = t.choice.as_ref().unwrap().editor.option_id(3);
    assert_eq!(t.on_click_editor(id), Some((Outcome::Consumed, None)));
    assert_eq!(t.rows[0][1].text, "stage");
}

#[test]
fn a_select_cell_opens_chooses_and_cancels_by_mouse() {
    let mut t = table();
    let (_, hits) = draw(&mut t, None);
    // a click on the current cell opens it (as a second click edits text)
    let cell = hits.area_of(t.cell_id(0, 1)).unwrap();
    let (out, _) = t.on_click_cell(0, 1, Position::new(cell.x, cell.y));
    assert_eq!(out, Outcome::Changed);
    assert!(t.choice.is_some());
    let (buf, hits) = draw(&mut t, None);
    let opt = t.choice.as_ref().unwrap().editor.option_id(0);
    let at = hits.area_of(opt).expect("an option row is clickable");
    assert!(
        at.y > cell.y,
        "the popup sits under the cell:\n{}",
        text(&buf)
    );
    assert_eq!(
        t.on_click_editor(opt),
        Some((
            Outcome::Changed,
            Some(TableEvent::Committed { row: 0, col: 1 })
        ))
    );
    assert_eq!(t.rows[0][1].text, "dev");
    // the `▾` on a hovered cell opens it in one click; outside cancels
    let mark = t.open_id(1, 1);
    let hover = t.cell_id(1, 1);
    let (_, hits) = draw(&mut t, Some(hover));
    assert!(hits.area_of(mark).is_some(), "▾ on the hovered cell");
    assert_eq!(t.on_click_editor(mark).map(|r| r.0), Some(Outcome::Changed));
    assert_eq!(t.cursor_row, 1);
    assert!(t.choice.is_some());
    let backdrop = backdrop_id(t.choice_id());
    assert_eq!(
        t.on_click_editor(backdrop),
        Some((Outcome::Changed, Some(TableEvent::Cancelled)))
    );
    assert!(t.choice.is_none());
}

#[test]
fn an_out_of_set_value_is_shown_flagged_and_fixed_by_choosing() {
    let mut t = table();
    assert_eq!(t.out_of_set_cells(), vec![(1, 1)]);
    t.cursor_col = 0;
    let (buf, _) = draw(&mut t, None);
    let theme = Theme::junie();
    let y = (0..buf.area.height)
        .find(|&y| (0..60).any(|x| buf[(x, y)].symbol() == "q" && buf[(x + 1, y)].symbol() == "a"))
        .expect("the stored value is shown");
    let x = (0..59)
        .find(|&x| buf[(x, y)].symbol() == "q" && buf[(x + 1, y)].symbol() == "a")
        .unwrap();
    assert_eq!(buf[(x, y)].fg, theme.error, "flagged in the error tone");
    assert!((x..60).any(|xx| buf[(xx, y)].symbol() == "!"), "with a `!`");
    t.cursor_row = 1;
    t.cursor_col = 1;
    t.on_key(&key(KeyCode::Enter));
    assert_eq!(
        t.choice.as_ref().unwrap().editor.current,
        None,
        "no current option"
    );
    t.on_key(&key(KeyCode::Enter));
    assert_eq!(t.rows[1][1].text, "dev");
    assert!(t.out_of_set_cells().is_empty());
}

#[test]
fn a_toggle_flips_in_place_and_tags_commit_the_checked_set() {
    let mut t = table();
    t.cursor_col = 2;
    assert_eq!(
        t.on_key(&key(KeyCode::Enter)).1,
        Some(TableEvent::Committed { row: 0, col: 2 })
    );
    assert_eq!(t.rows[0][2].text, "off");
    assert!(t.choice.is_none(), "a toggle opens nothing");
    t.cursor_col = 3;
    t.on_key(&key(KeyCode::Enter));
    t.on_key(&key(KeyCode::Down));
    t.on_key(&key(KeyCode::Down));
    t.on_key(&key(KeyCode::Char(' ')));
    assert_eq!(
        t.on_key(&key(KeyCode::Enter)).1,
        Some(TableEvent::Committed { row: 0, col: 3 })
    );
    assert_eq!(t.rows[0][3].text, "api, db");
}

#[test]
fn tab_commits_and_opens_the_next_editable_cell() {
    let mut t = table();
    t.cursor_col = 3;
    t.on_key(&key(KeyCode::Enter));
    let (_, ev) = t.on_key(&key(KeyCode::Tab));
    assert_eq!(ev, Some(TableEvent::Committed { row: 0, col: 3 }));
    assert_eq!(t.cursor_col, 4);
    assert!(t.choice.is_some(), "the segmented cell opened");
    let (buf, _) = draw(&mut t, None);
    assert!(text(&buf).contains("S │ M │ L"), "{}", text(&buf));
    t.on_key(&key(KeyCode::Right));
    assert_eq!(
        t.on_key(&key(KeyCode::Tab)).1,
        Some(TableEvent::LeaveForward)
    );
    assert_eq!(t.rows[0][4].text, "L");
}

#[test]
fn the_validator_still_decides_and_blur_follows_the_config() {
    let mut t = table().validator(|col, v| (col == 1 && v == "dev").then(|| "No dev here".into()));
    t.on_key(&key(KeyCode::Enter));
    t.on_key(&key(KeyCode::Home));
    assert_eq!(t.on_key(&key(KeyCode::Enter)).1, None);
    assert!(t.choice.is_some(), "a rejected choice keeps the popup open");
    assert_eq!(t.edit_error(), Some("No dev here"));
    // default blur: cancel
    assert_eq!(t.on_blur(), Some(TableEvent::Cancelled));
    assert_eq!(t.rows[0][1].text, "stage");
    // configured: commit the highlighted option
    let mut t = table().editor_config(EditorConfig {
        blur: BlurAction::Commit,
        ..EditorConfig::default()
    });
    t.on_key(&key(KeyCode::Enter));
    t.on_key(&key(KeyCode::Down));
    assert_eq!(t.on_blur(), Some(TableEvent::Committed { row: 0, col: 1 }));
    assert_eq!(t.rows[0][1].text, "prod");
}

#[test]
fn alt_down_opens_and_a_checkbox_cell_shows_a_box() {
    let mut t = table();
    let down = |mods| Key {
        code: KeyCode::Down,
        mods,
    };
    t.on_key(&down(KeyModifiers::ALT));
    assert!(t.choice.is_some(), "Alt+↓ opens");
    t.on_key(&key(KeyCode::Esc));
    t.on_key(&down(KeyModifiers::NONE));
    assert!(t.choice.is_none() && t.cursor_row == 1, "plain ↓ moves");
    let (buf, _) = draw(&mut t, None);
    let theme = Theme::junie();
    let rows = text(&buf);
    assert!(rows.contains(theme.glyphs.check_on), "{rows}");
    assert!(rows.contains(theme.glyphs.check_off), "{rows}");
}

#[test]
fn the_affordance_follows_its_policy() {
    use crate::widgets::option_editor::Affordance;
    let marks = |t: &mut DataTable| {
        let (_, hits) = draw(t, None);
        (0..2)
            .filter(|&r| hits.area_of(t.open_id(r, 1)).is_some())
            .count()
    };
    let mut t = table();
    assert_eq!(marks(&mut t), 1, "focused: the cursor cell only");
    t.editor_config.affordance = Affordance::Always;
    assert_eq!(marks(&mut t), 2, "always: every editable option cell");
    t.editor_config.affordance = Affordance::Never;
    assert_eq!(marks(&mut t), 0);
}

#[test]
fn read_only_and_text_cells_show_no_affordance() {
    let mut t = table();
    t.cursor_col = 0;
    let hover = t.cell_id(0, 0);
    let (buf, hits) = draw(&mut t, Some(hover));
    assert!(
        hits.area_of(t.open_id(0, 0)).is_none(),
        "a text cell has no ▾"
    );
    assert!(hits.area_of(t.open_id(0, 2)).is_none(), "a toggle has no ▾");
    let glyph = Theme::junie().glyphs.disclosure_open;
    assert!(!text(&buf).contains(glyph), "{}", text(&buf));
}
