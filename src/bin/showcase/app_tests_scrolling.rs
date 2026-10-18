use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::pages::panels::PanelsPage;
use crate::pages::scrolling::ScrollingPage;
use crate::pages::template::Section;
use junie_tui::core::event::MouseKind;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::specimen::SpecimenState;
use junie_tui::widgets::scrollbar;

fn scrolling(h: &Harness) -> &ScrollingPage {
    h.app.pages[PageId::Scrolling.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<ScrollingPage>())
        .expect("the Scrolling page")
}

fn panels(h: &Harness) -> &PanelsPage {
    h.app.pages[PageId::Panels.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<PanelsPage>())
        .expect("the Panels page")
}

fn hint(h: &Harness, page: PageId, focus: WidgetId, label: &str) -> Option<&'static str> {
    h.app.pages[page.index()]
        .hints(Some(focus))
        .into_iter()
        .find(|(_, l)| *l == label)
        .map(|(k, _)| k)
}

fn key_of(token: &str) -> KeyCode {
    match token {
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PgUp" => KeyCode::PageUp,
        "PgDn" => KeyCode::PageDown,
        t => KeyCode::Char(t.chars().next().unwrap()),
    }
}

fn big() -> Harness {
    Harness::on(160, 60, PageId::Scrolling)
}

#[test]
fn ctrl_c_over_a_panel_selection_copies_instead_of_quitting() {
    use junie_tui::core::edit::clipboard;
    let mut h = big();
    let prose = WidgetId::of("scrolling").sub("prose");
    h.focus(prose);
    h.key_mod(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let selected = scrolling(&h).prose.selected_text();
    assert!(selected.is_some(), "Ctrl+A selects");
    clipboard::copy("");
    h.key_mod(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert!(!h.app.shell.quit, "Ctrl+C over a selection never quits");
    assert!(h.app.dialog().is_none(), "nor asks to quit");
    assert_eq!(clipboard::text(), selected, "the selection is copied");

    h.key(KeyCode::Esc);
    assert!(scrolling(&h).prose.selected_text().is_none());
    h.key_mod(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert!(h.app.shell.quit, "Ctrl+C with nothing selected quits");
}

#[test]
fn hinted_end_keys_reach_both_ends_of_the_list_under_either_profile() {
    for profile in ["desktop", "vim"] {
        let mut h = big();
        h.opt(&format!("scrolling.profile={profile}"));
        let list = scrolling(&h).list.id;
        h.focus(list);
        let ends = hint(&h, PageId::Scrolling, list, "Ends").expect("an Ends hint");
        assert_eq!(ends.contains("g G"), profile == "vim", "{profile}: {ends}");

        let tokens: Vec<&str> = ends.split(' ').collect();
        for pair in tokens.chunks(2) {
            h.key(key_of(pair[1]));
            assert_eq!(scrolling(&h).list.cursor, 118, "{profile}: {}", pair[1]);
            h.key(key_of(pair[0]));
            assert_eq!(scrolling(&h).list.cursor, 0, "{profile}: {}", pair[0]);
        }
        if profile == "desktop" {
            h.key(KeyCode::Char('G'));
            assert_eq!(scrolling(&h).list.cursor, 0, "G is not End on desktop");
        }
    }
}

#[test]
fn hinted_end_keys_reach_both_ends_of_a_panel_under_either_key_set() {
    for nav in ["desktop", "vim"] {
        let mut h = Harness::on(120, 40, PageId::Panels);
        h.opt(&format!("panels.nav={nav}"));
        let prose = panels(&h).prose.id;
        h.focus(prose);
        let ends = hint(&h, PageId::Panels, prose, "Ends").expect("an Ends hint");
        assert_eq!(ends.contains("g G"), nav == "vim", "{nav}: {ends}");
        let tokens: Vec<&str> = ends.split(' ').collect();
        for pair in tokens.chunks(2) {
            h.key(key_of(pair[1]));
            assert!(panels(&h).prose.scroll.at_end(), "{nav}: {}", pair[1]);
            h.key(key_of(pair[0]));
            assert_eq!(panels(&h).prose.scroll.offset, 0, "{nav}: {}", pair[0]);
        }
    }

    let mut h = Harness::on(120, 40, PageId::Panels);
    h.opt("panels.nav=no-paging");
    let prose = panels(&h).prose.id;
    h.focus(prose);
    assert_eq!(hint(&h, PageId::Panels, prose, "Page"), None);
    h.key(KeyCode::PageDown);
    assert_eq!(panels(&h).prose.scroll.offset, 0);
}

#[test]
fn context_rows_keep_rows_beyond_the_cursor_in_view() {
    for (context, offset) in [("0", 0), ("1", 1), ("2", 2)] {
        let mut h = big();
        h.opt(&format!("scrolling.context={context}"));

        h.opt("scrolling.skip_disabled=off");
        let list = scrolling(&h).list.id;
        h.focus(list);
        let rows = scrolling(&h).list.scroll.viewport_len;

        for _ in 0..rows - 1 {
            h.key(KeyCode::Down);
        }
        let l = &scrolling(&h).list;
        assert_eq!(l.cursor, rows - 1);
        assert_eq!(l.scroll.offset, offset, "context {context}");
    }
}

#[test]
fn skip_disabled_and_wrap_drive_the_list_cursor() {
    let mut h = big();
    let list = scrolling(&h).list.id;
    h.focus(list);
    for _ in 0..8 {
        h.key(KeyCode::Down);
    }
    assert_eq!(scrolling(&h).list.cursor, 8);
    h.key(KeyCode::Down);
    assert_eq!(scrolling(&h).list.cursor, 10, "the retired row is skipped");
    h.opt("scrolling.skip_disabled=off");
    h.key(KeyCode::Up);
    assert_eq!(scrolling(&h).list.cursor, 9, "now it is reachable");

    h.key(KeyCode::Home);
    h.key(KeyCode::Up);
    assert_eq!(scrolling(&h).list.cursor, 0);
    h.opt("scrolling.wrap=on");
    h.key(KeyCode::Up);
    assert_eq!(scrolling(&h).list.cursor, 119);
}

#[test]
fn the_flag_shortcut_wins_or_yields_by_precedence() {
    let mut h = big();
    let list = scrolling(&h).list.id;
    h.focus(list);
    h.key(KeyCode::Down);
    assert_eq!(hint(&h, PageId::Scrolling, list, "Flag"), Some("f"));
    h.key(KeyCode::Char('f'));
    assert!(
        scrolling(&h).flagged.contains(&1),
        "shortcuts first: f flags"
    );
    assert!(h.text().contains("flagged"));
    h.opt("scrolling.precedence=type-ahead");
    assert_eq!(
        hint(&h, PageId::Scrolling, list, "Flag"),
        None,
        "f types now: not advertised"
    );
    h.key(KeyCode::Char('f'));
    assert!(
        scrolling(&h).flagged.contains(&1),
        "type-ahead first: f is typed, the flag stays"
    );

    h.opt("scrolling.type_ahead=off");
    assert_eq!(hint(&h, PageId::Scrolling, list, "Flag"), Some("f"));
    h.key(KeyCode::Char('f'));
    assert!(!scrolling(&h).flagged.contains(&1));
}

#[test]
fn the_cursor_readout_names_the_last_step_and_move() {
    let mut h = big();
    let list = scrolling(&h).list.id;
    h.focus(list);
    assert!(h.text().contains("Last: no key yet"));
    h.key(KeyCode::Down);
    assert!(
        h.text().contains("Last: Nav(Down) → To(row 2)"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Up);
    h.key(KeyCode::Up);
    assert!(h.text().contains("Last: Nav(Up) → Stayed(row 1)"));
    h.key(KeyCode::Char('x'));
    assert!(h.text().contains("Last: Type('x') → Stayed(row 1)"));
}

#[test]
fn type_ahead_pause_and_precedence_reach_every_collection() {
    for (timeout, cursor) in [("2s", 1), ("0.5s", 0)] {
        let mut h = big();
        h.opt(&format!("scrolling.timeout={timeout}"));
        let tree = scrolling(&h).tree.id;
        h.focus(tree);
        h.key(KeyCode::Char('a'));
        std::thread::sleep(std::time::Duration::from_millis(700));
        h.key(KeyCode::Char('u'));
        assert_eq!(scrolling(&h).tree.cursor, cursor, "pause {timeout}");
    }
    let mut h = big();
    h.opt("scrolling.precedence=type-ahead");
    let p = scrolling(&h);
    use junie_tui::core::list::TypeAheadPrecedence::TypeAhead;
    assert_eq!(p.list.type_ahead_precedence, Some(TypeAhead));
    assert_eq!(p.tree.type_ahead_precedence, Some(TypeAhead));
    assert_eq!(p.region.type_ahead_precedence, Some(TypeAhead));
    assert_eq!(p.table.type_ahead_precedence, Some(TypeAhead));
    assert_eq!(p.grid.type_ahead_precedence, Some(TypeAhead));

    let (region, tree) = (p.region.id, p.tree.id);
    h.focus(region);
    h.key(KeyCode::Char('u'));
    assert_eq!(scrolling(&h).region.selected, 9, "us-east-1");
    h.focus(tree);
    h.key(KeyCode::Char('j'));
    assert_eq!(scrolling(&h).tree.cursor, 8, "jobs");
}

#[test]
fn the_hand_drawn_table_follows_the_list_and_scrolls_by_column() {
    let mut h = big();
    let list = scrolling(&h).list.id;
    h.focus(list);
    h.key(KeyCode::End);

    assert!(h.text().contains("jobs-12"), "the table followed the list");
    let hand = WidgetId::of("scrolling").sub("hand");
    let area = h
        .app
        .shell
        .hits
        .area_of(hand)
        .expect("the table takes the wheel");
    let before = scrolling(&h).hand.v.offset;
    h.mouse(MouseKind::WheelUp, area.x + 2, area.y + 1);
    assert!(
        scrolling(&h).hand.v.offset < before,
        "on-move: the wheel wins"
    );
    h.opt("scrolling.follow=always");
    h.mouse(MouseKind::WheelUp, area.x + 2, area.y + 1);
    assert!(h.text().contains("jobs-12"), "always: pinned to the cursor");

    h.mouse(MouseKind::WheelRight, area.x + 2, area.y + 1);
    assert_eq!(scrolling(&h).hand.h.offset, 1);
    h.opt("scrolling.columns=off");
    h.mouse(MouseKind::WheelRight, area.x + 2, area.y + 1);
    assert_eq!(scrolling(&h).hand.h.offset, 4);

    let hbar = scrollbar::h_id_for(hand);
    assert!(h.app.shell.hits.area_of(hbar).is_some());
    h.opt("scrolling.hbar=never");
    assert!(h.app.shell.hits.area_of(hbar).is_none());
}

#[test]
fn track_click_pages_or_jumps_the_list() {
    for (click, jumps) in [("page", false), ("jump", true)] {
        let mut h = big();
        h.opt(&format!("scrolling.track_click={click}"));
        let bar = scrollbar::id_for(scrolling(&h).list.id);
        let t = h.app.shell.hits.area_of(bar).unwrap();
        h.click(t.x, t.bottom() - 1);
        let l = &scrolling(&h).list;
        assert_eq!(l.scroll.at_end(), jumps, "{click}");
    }
}

#[test]
fn a_divider_is_solid_while_its_content_fits_and_a_scrollbar_once_it_overflows() {
    let h = big();
    let fits = WidgetId::of("scrolling").sub("fits");
    let over = WidgetId::of("scrolling").sub("over");
    assert!(h.app.shell.hits.area_of(scrollbar::id_for(fits)).is_none());
    let bar = h.app.shell.hits.area_of(scrollbar::id_for(over)).unwrap();
    let col = |x: u16| -> Vec<String> {
        let buf = h.term.backend().buffer();
        (bar.y..bar.bottom())
            .map(|y| buf[(x, y)].symbol().to_owned())
            .collect()
    };
    let g = junie_tui::theme::Theme::junie().glyphs;
    assert!(
        col(bar.x).iter().any(|s| s == g.scroll_track),
        "dotted track"
    );
    assert!(col(bar.x).iter().any(|s| s == g.scroll_thumb));

    let (fx, fy) = h.find("fits").unwrap();
    let row: Vec<char> = h.row(fy).chars().collect();
    let ox = (fx as usize..row.len())
        .find(|&x| row[x..].starts_with(&['o', 'v', 'e', 'r']))
        .unwrap() as u16;
    assert!(
        (fx..ox).any(|x| col(x).iter().all(|s| s == g.divider)),
        "a solid divider between the panes"
    );
}

#[test]
fn the_scrolling_states_grid_draws_every_panel_state() {
    let mut h = big();
    h.section(Section::States);
    let rows = h.app.pages[PageId::Scrolling.index()]
        .template()
        .unwrap()
        .state_rows()
        .to_vec();
    use SpecimenState::*;
    for s in [Rest, Hover, Pressed, Empty, Overflow, Narrow] {
        assert!(rows.contains(&s), "{s:?}");
    }
    assert!(h.bars() <= 1);
}

#[test]
fn the_card_focus_mark_puts_the_one_bar_on_the_title() {
    let mut h = Harness::on(120, 40, PageId::Panels);
    let log = panels(&h).log.id;
    h.focus(log);
    let (_, y) = h.find("Card · scrollable").unwrap();
    assert!(!h.row(y).contains('▎'), "title focus: no bar");
    h.opt("panels.focus_mark=bar");
    let (_, y) = h.find("Card · scrollable").unwrap();
    assert!(h.row(y).contains('▎'), "bar focus: the title carries it");
    assert_eq!(h.bars(), 1, "still one focus indicator");
}

#[test]
fn panel_options_rebuild_the_scrolling_panels() {
    let mut h = Harness::on(120, 40, PageId::Panels);
    let log = panels(&h).log.id;

    assert_eq!(hint(&h, PageId::Panels, log, "Follow tail"), None);
    h.opt("panels.tail=on");
    assert!(panels(&h).log.follow);
    assert_eq!(hint(&h, PageId::Panels, log, "Follow tail"), Some("f"));
    h.opt("panels.keymap=t");
    assert_eq!(hint(&h, PageId::Panels, log, "Follow tail"), Some("t"));
    h.focus(log);
    h.key(KeyCode::Char('t'));
    assert!(!panels(&h).log.follow, "t pauses following");

    let hbar = scrollbar::h_id_for(log);
    assert!(h.app.shell.hits.area_of(hbar).is_some());
    h.opt("panels.hscroll=off");
    assert!(h.app.shell.hits.area_of(hbar).is_none());
    let area = h.app.shell.hits.area_of(log).unwrap();
    h.mouse(MouseKind::WheelRight, area.x + 2, area.y + 1);
    assert_eq!(panels(&h).log.scroll.h.offset, 0);
    h.opt("panels.hscroll=step-1");
    h.mouse(MouseKind::WheelRight, area.x + 2, area.y + 1);
    assert_eq!(panels(&h).log.scroll.h.offset, 1, "one cell a notch");

    h.opt("panels.wrap=off");
    assert!(!panels(&h).prose.wrap);

    let (x2, _) = h.find("A card is").unwrap();
    h.opt("panels.pad_x=0");
    let (x0, _) = h.find("A card is").unwrap();
    assert_eq!(x2, x0 + 2);
}

#[test]
fn the_panels_states_grid_draws_every_panel_state() {
    let mut h = Harness::on(120, 40, PageId::Panels);
    h.section(Section::States);
    let rows = h.app.pages[PageId::Panels.index()]
        .template()
        .unwrap()
        .state_rows()
        .to_vec();
    use SpecimenState::*;
    for s in [Rest, Focused, Overflow, Narrow] {
        assert!(rows.contains(&s), "{s:?}");
    }

    assert!(h.text().contains("▏Log"), "{}", h.text());
    assert!(h.bars() <= 1);
    let _ = KeyModifiers::NONE;
}
