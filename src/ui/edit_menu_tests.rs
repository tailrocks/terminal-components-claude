use super::*;
use crate::core::edit::clipboard;
use crate::core::focus::FocusRing;
use crate::core::hit::HitRegistry;
use crate::core::pointer::MouseTarget;
use crate::core::response::Response;
use crate::theme::Theme;
use crate::ui::ctx::Interaction;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

const SCREEN: Rect = Rect::new(0, 0, 60, 16);

fn frame(
    focus: Option<WidgetId>,
    draw: impl FnOnce(&mut Buffer, &mut RenderCtx),
) -> (Buffer, HitRegistry) {
    let theme = Theme::junie();
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut buf = Buffer::empty(SCREEN);
    {
        let mut ctx = RenderCtx::new(
            &theme,
            Interaction {
                focus,
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        draw(&mut buf, &mut ctx);
        ctx.flush_overlays(&mut buf);
    }
    (buf, hits)
}

fn at(hits: &HitRegistry, phase: Phase, pos: Position) -> PointerEvent {
    let id = hits.hit(pos).expect("a region under the pointer");
    PointerEvent::new(hits, phase, id, pos, 1, KeyModifiers::NONE)
}

fn row(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_owned())
        .collect()
}

fn find(buf: &Buffer, label: &str) -> Option<Position> {
    (0..buf.area.height).find_map(|y| {
        let r = row(buf, y);
        r.find(label).map(|i| {
            let x = r[..i].chars().count() as u16;
            Position::new(x, y)
        })
    })
}

fn menu_rows(buf: &Buffer) -> Vec<&'static str> {
    ["Cut", "Copy", "Paste", "Select all"]
        .into_iter()
        .filter(|l| find(buf, &format!(" {l} ")).is_some() || find(buf, &format!(" {l}")).is_some())
        .collect()
}

fn key(code: KeyCode, mods: KeyModifiers) -> Key {
    Key { code, mods }
}

fn right_click_then_choose<W: MouseTarget>(
    w: &mut W,
    draw: &mut dyn FnMut(&mut W, &mut Buffer, &mut RenderCtx),
    focus: WidgetId,
    pos: Position,
    label: &str,
) -> Buffer {
    let (_, hits) = frame(Some(focus), |b, c| draw(w, b, c));
    let r: Option<Response<W::Action>> = w.route(&at(&hits, Phase::Secondary, pos));
    assert!(
        r.is_some_and(|r| r.outcome() == Outcome::Changed),
        "a secondary click on the text opens the Edit menu"
    );
    let (buf, hits) = frame(Some(focus), |b, c| draw(w, b, c));
    let open = buf.clone();
    let p = find(&buf, label).unwrap_or_else(|| panic!("the menu shows {label}"));
    let r = w.route(&at(&hits, Phase::Click, p));
    assert!(r.is_some(), "the menu row is the widget's");
    open
}

#[test]
fn a_right_click_on_a_text_input_opens_cut_copy_paste_select_all() {
    use crate::widgets::input::TextInput;
    clipboard::copy("pasted");
    let id = WidgetId::of("em.input");
    let mut i = TextInput::new(id, "Name").value("hello world");
    let mut draw = |w: &mut TextInput, b: &mut Buffer, c: &mut RenderCtx| {
        w.render(Rect::new(0, 0, 40, 3), b, c, c.theme.canvas)
    };

    let open = right_click_then_choose(&mut i, &mut draw, id, Position::new(4, 1), "Select all");
    assert_eq!(menu_rows(&open), ["Cut", "Copy", "Paste", "Select all"]);
    assert!(
        !row(&open, 1).contains(open_focus_bar()),
        "the field under its menu shows no focus bar"
    );
    assert_eq!(i.field.buffer.selected_text(), Some("hello world"));
    assert!(!i.edit_menu.is_open(), "choosing closes the menu");

    right_click_then_choose(&mut i, &mut draw, id, Position::new(4, 1), "Copy");
    assert_eq!(clipboard::text().as_deref(), Some("hello world"));
    clipboard::copy("bye");
    right_click_then_choose(&mut i, &mut draw, id, Position::new(4, 1), "Paste");
    assert_eq!(i.text(), "bye");

    let (_, _) = frame(Some(id), |b, c| draw(&mut i, b, c));
    let (o, _) = i.on_key(&key(KeyCode::F(10), KeyModifiers::SHIFT));
    assert_eq!(o, Outcome::Changed);
    assert!(i.edit_menu.is_open(), "Shift+F10 opens the Edit menu");
    let (buf, _) = frame(Some(id), |b, c| draw(&mut i, b, c));
    assert_eq!(menu_rows(&buf), ["Cut", "Copy", "Paste", "Select all"]);
    let (o, _) = i.on_key(&key(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(o, Outcome::Changed);
    assert!(!i.edit_menu.is_open(), "Esc closes the menu");
    assert_eq!(i.text(), "bye", "Esc closed only the menu");

    let mut off = TextInput::new(id, "Name").value("x").context_menu(false);
    let (_, hits) = frame(Some(id), |b, c| draw(&mut off, b, c));
    assert!(
        off.route(&at(&hits, Phase::Secondary, Position::new(2, 1)))
            .is_none_or(|r| r.outcome() == Outcome::Ignored)
    );
    assert!(!off.edit_menu.is_open());
}

fn open_focus_bar() -> &'static str {
    Theme::junie().glyphs.focus_bar
}

#[test]
fn unavailable_commands_stay_with_their_reason() {
    use crate::core::edit::{FieldKeys, TextCmd};
    let field = EditField::single("abc").keys(FieldKeys::Desktop);
    let caps = EditCaps::of(&field);
    clipboard::clear_hook();
    let items = edit_items(caps, |c| field.shortcut(c));
    let get = |c: TextCmd| {
        items
            .iter()
            .find(|(x, _)| *x == c)
            .map(|(_, i)| i.clone())
            .unwrap()
    };
    assert!(get(TextCmd::Cut).item.disabled, "Cut needs a selection");
    assert!(get(TextCmd::Copy).item.disabled, "Copy needs a selection");
    assert!(!get(TextCmd::SelectAll).item.disabled);
    assert_eq!(
        get(TextCmd::SelectAll).shortcut.as_deref(),
        Some("Ctrl+A"),
        "Select all names the first key the terminal delivers"
    );

    let ro = edit_items(EditCaps::read_only(true, true), |_| None);
    let cmds: Vec<_> = ro.iter().map(|(c, _)| *c).collect();
    assert_eq!(cmds, [TextCmd::Copy, TextCmd::SelectAll]);

    let secret = edit_items(EditCaps::of(&field).secret(true), |_| None);
    assert!(
        secret
            .iter()
            .filter(|(c, _)| matches!(c, TextCmd::Cut | TextCmd::Copy))
            .all(|(_, i)| i.item.disabled)
    );
}

#[test]
fn a_right_click_on_a_text_area_places_the_caret_and_pastes_there() {
    use crate::widgets::textarea::TextArea;
    let id = WidgetId::of("em.area");
    let mut a = TextArea::new(id, "Notes", 4).value("one\ntwo");
    let mut draw = |w: &mut TextArea, b: &mut Buffer, c: &mut RenderCtx| {
        w.render(Rect::new(0, 0, 40, 6), b, c, c.theme.canvas)
    };
    let (buf, _) = frame(Some(id), |b, c| draw(&mut a, b, c));

    let two = find(&buf, "two").expect("the second line");
    let pos = Position::new(two.x + 1, two.y);
    clipboard::copy("X");
    let open = right_click_then_choose(&mut a, &mut draw, id, pos, "Paste");
    assert_eq!(menu_rows(&open), ["Cut", "Copy", "Paste", "Select all"]);
    assert_eq!(
        a.field.text(),
        "one\ntXwo",
        "Paste lands where the right click pointed"
    );
    right_click_then_choose(&mut a, &mut draw, id, pos, "Select all");
    assert_eq!(a.field.buffer.selected_text(), Some("one\ntXwo"));

    right_click_then_choose(&mut a, &mut draw, id, pos, "Cut");
    assert_eq!(
        (a.field.text(), clipboard::text().as_deref()),
        ("", Some("one\ntXwo"))
    );
}

#[test]
fn a_code_editor_offers_the_edit_menu_and_only_copy_and_select_all_read_only() {
    use crate::widgets::code::{CodeEditor, EditorAction};
    let id = WidgetId::of("em.code");
    let mut e = CodeEditor::new(id, "select 1;\nselect 2;");
    let mut draw = |w: &mut CodeEditor, b: &mut Buffer, c: &mut RenderCtx| {
        w.render(Rect::new(0, 0, 40, 6), b, c, c.theme.canvas)
    };
    let (buf, _) = frame(Some(id), |b, c| draw(&mut e, b, c));
    let pos = find(&buf, "select 1;").expect("the first line");
    let open = right_click_then_choose(&mut e, &mut draw, id, pos, "Select all");
    assert_eq!(menu_rows(&open), ["Cut", "Copy", "Paste", "Select all"]);
    assert_eq!(e.field.buffer.selected_text(), Some("select 1;\nselect 2;"));

    frame(Some(id), |b, c| draw(&mut e, b, c));
    clipboard::copy("x");
    e.on_key(&key(KeyCode::Menu, KeyModifiers::NONE));
    assert!(e.edit_menu.is_open(), "the Menu key opens it");

    e.on_key(&key(KeyCode::Up, KeyModifiers::NONE));
    e.on_key(&key(KeyCode::Up, KeyModifiers::NONE));
    let (o, ev) = e.on_key(&key(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!((o, ev), (Outcome::Changed, Some(EditorAction::Changed)));
    assert_eq!(e.text(), "x");

    let mut ro = CodeEditor::new(id, "select 1;").read_only(true);
    let (buf, _) = frame(Some(id), |b, c| draw(&mut ro, b, c));
    let pos = find(&buf, "select 1;").expect("the line");
    let open = right_click_then_choose(&mut ro, &mut draw, id, pos, "Select all");
    assert_eq!(menu_rows(&open), ["Copy", "Select all"]);
    right_click_then_choose(&mut ro, &mut draw, id, pos, "Copy");
    assert_eq!(clipboard::text().as_deref(), Some("select 1;"));
    assert_eq!(ro.text(), "select 1;");
}

#[test]
fn a_right_click_on_a_viewport_and_a_diff_offers_copy_and_select_all() {
    use crate::widgets::diff::{DiffFile, DiffHunk, DiffLine, DiffStatus, DiffView};
    use crate::widgets::viewport::{Span, TextViewport};
    let id = WidgetId::of("em.view");
    let mut v = TextViewport::new(id);
    v.set_lines(vec![
        vec![Span::plain("first line")],
        vec![Span::plain("second")],
    ]);
    let mut draw = |w: &mut TextViewport, b: &mut Buffer, c: &mut RenderCtx| {
        w.render(Rect::new(0, 0, 40, 5), b, c, c.theme.canvas)
    };
    let open = right_click_then_choose(&mut v, &mut draw, id, Position::new(2, 0), "Select all");
    assert_eq!(menu_rows(&open), ["Copy", "Select all"]);
    right_click_then_choose(&mut v, &mut draw, id, Position::new(2, 0), "Copy");
    assert_eq!(clipboard::text().as_deref(), Some("first line\nsecond"));

    v.on_key(&key(KeyCode::F(10), KeyModifiers::SHIFT));
    assert!(v.edit_menu.is_open());

    let did = WidgetId::of("em.diff");
    let mut d = DiffView::new(did);
    d.set_file(Some(DiffFile {
        path: "a.rs".into(),
        status: DiffStatus::Modified,
        hunks: vec![DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![DiffLine::context("keep"), DiffLine::add("added")],
        }],
    }));
    let mut draw = |w: &mut DiffView, b: &mut Buffer, c: &mut RenderCtx| {
        w.render(Rect::new(0, 0, 40, 6), b, c, c.theme.canvas)
    };
    let open = right_click_then_choose(&mut d, &mut draw, did, Position::new(2, 0), "Select all");
    assert_eq!(menu_rows(&open), ["Copy", "Select all"]);
    right_click_then_choose(&mut d, &mut draw, did, Position::new(2, 0), "Copy");
    assert!(
        clipboard::text().is_some_and(|t| t.contains("added")),
        "Copy puts the selected diff on the clipboard"
    );
}

#[test]
fn a_right_click_on_a_scroll_panel_offers_copy_and_select_all() {
    use crate::widgets::panel::ScrollPanel;
    let id = WidgetId::of("em.panel");
    let mut p = ScrollPanel::new(id, vec!["alpha".into(), "beta".into()]);
    let mut draw = |w: &mut ScrollPanel, b: &mut Buffer, c: &mut RenderCtx| {
        w.render(Rect::new(0, 0, 40, 5), b, c, c.theme.canvas, |t, _| {
            ratatui::style::Style::new().fg(t.text_primary)
        })
    };
    let (_, hits) = frame(Some(id), |b, c| draw(&mut p, b, c));
    let pos = (0..5)
        .flat_map(|y| (0..40).map(move |x| Position::new(x, y)))
        .find(|q| hits.hit(*q) == Some(id))
        .expect("the panel's text");
    let open = right_click_then_choose(&mut p, &mut draw, id, pos, "Select all");
    assert_eq!(menu_rows(&open), ["Copy", "Select all"]);
    right_click_then_choose(&mut p, &mut draw, id, pos, "Copy");
    assert_eq!(clipboard::text().as_deref(), Some("alpha\nbeta"));
}

#[test]
fn a_right_click_on_the_picker_query_edits_it_and_reports_the_change() {
    use crate::core::id::Part;
    use crate::widgets::option_list::Item;
    use crate::widgets::picker::{Picker, PickerAction};
    let id = WidgetId::of("em.picker");
    let mut p = Picker::new(id, "Open");
    p.set_items(vec![Item::new("alpha"), Item::new("beta")]);
    p.set_query("alp");
    let draw = |w: &mut Picker, b: &mut Buffer, c: &mut RenderCtx| w.draw(SCREEN, b, c);
    let (_, hits) = frame(Some(id), |b, c| draw(&mut p, b, c));
    let pos = (0..SCREEN.height)
        .flat_map(|y| (0..SCREEN.width).map(move |x| Position::new(x, y)))
        .find(|q| {
            hits.hit(*q).is_some() && at(&hits, Phase::Secondary, *q).part().part == Part::QUERY
        })
        .expect("the query line");
    clipboard::copy("ha");
    let r = p
        .route(&at(&hits, Phase::Secondary, pos))
        .expect("the picker's");
    assert_eq!(r.outcome(), Outcome::Changed);
    let (buf, hits) = frame(Some(id), |b, c| draw(&mut p, b, c));
    assert_eq!(menu_rows(&buf), ["Cut", "Copy", "Paste", "Select all"]);

    let corner = at(&hits, Phase::Click, Position::new(0, SCREEN.height - 1));
    assert!(p.edit_menu.owns(&corner));
    let paste = find(&buf, "Paste").unwrap();
    let mut r = p
        .route(&at(&hits, Phase::Click, paste))
        .expect("the menu's row");
    assert_eq!(r.take_action(), Some(PickerAction::QueryChanged));
    assert!(p.query().contains("ha"));
}
