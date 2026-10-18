




use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::options;
use crate::pages::query_workbench::QueryWorkbenchPage;
use crate::pages::template::Section;
use junie_tui::core::event::Input;
use junie_tui::runtime::Application;

fn page(h: &Harness) -> &QueryWorkbenchPage {
    h.app.pages[PageId::QueryWorkbench.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<QueryWorkbenchPage>())
        .expect("the Query workbench page")
}

fn ticks(h: &mut Harness, n: usize) {
    for _ in 0..n {
        h.app.handle(Input::Tick);
    }
    h.draw();
}

fn type_text(h: &mut Harness, s: &str) {
    for c in s.chars() {
        h.key(KeyCode::Char(c));
    }
}



#[test]
fn screen_dump() {
    let Ok(spec) = std::env::var("DUMP_SCREEN") else {
        return;
    };
    let (slug, size) = spec.split_once('@').unwrap_or((&spec, "120x40"));
    let (w, hh) = size.split_once('x').unwrap();
    let mut h = Harness::on(
        w.parse().unwrap(),
        hh.parse().unwrap(),
        PageId::from_name(slug).unwrap(),
    );
    h.section(Section::Playground);
    println!("{}", h.text());
}

#[test]
fn the_workbench_shows_explorer_tabs_editor_and_results() {
    let h = Harness::on(120, 40, PageId::QueryWorkbench);
    let text = h.text();
    assert!(!text.contains("arrives with SC46"), "{text}");
    for want in [
        "Explorer",
        "acme_prod",
        "Query 1",
        "customers",
        "Run",
        "Pending orders",
        "Results",
        "25 rows · 15 ms",
        "Northwind Traders",
        "production",
        "safe mode",
    ] {
        assert!(text.contains(want), "{want:?} missing:\n{text}");
    }
}

#[test]
fn opening_a_table_in_the_explorer_opens_its_tab_and_ctrl_r_runs_it() {
    let mut h = Harness::on(120, 40, PageId::QueryWorkbench);
    let (x, y) = h.find("T orders").expect("the orders table row");
    h.click(x + 3, y);
    assert!(h.app.shell.focus.is(page(&h).explorer.id));
    h.key(KeyCode::Enter);
    assert_eq!(page(&h).tabs.items.len(), 3, "a tab per opened table");
    assert!(
        h.text().contains("SELECT * FROM orders LIMIT 100"),
        "{}",
        h.text()
    );
    assert!(h.text().contains("Not run yet"));
    h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
    assert!(page(&h).tabs.items[2].busy, "the running tab is busy");
    assert!(h.text().contains("Running"), "{}", h.text());
    ticks(&mut h, 8);
    assert!(!page(&h).tabs.items[2].busy);
    assert!(h.text().contains("40 rows · 15 ms"), "{}", h.text());

    h.focus(page(&h).explorer.id);
    h.key(KeyCode::Enter);
    assert_eq!(page(&h).tabs.items.len(), 3);
}

#[test]
fn the_filter_narrows_the_explorer() {
    let mut h = Harness::on(120, 40, PageId::QueryWorkbench);
    h.focus(page(&h).filter.id);
    type_text(&mut h, "pay");
    let text = h.text();
    assert!(text.contains("payments"), "{text}");
    assert!(!text.contains("T customers"), "{text}");
}

#[test]
fn an_unknown_table_shows_the_error_card_and_marks_the_editor() {
    let mut h = Harness::on(120, 40, PageId::QueryWorkbench);
    let tabs = page(&h).tabs.id;
    h.focus(tabs);
    h.key(KeyCode::Char('n'));
    assert_eq!(page(&h).tabs.items.len(), 3, "n opens a query tab");
    assert!(h.text().contains("Query 2"));
    let editor = page(&h).editor.id;
    h.focus(editor);
    if !page(&h).editor.editing {
        h.key(KeyCode::Enter);
    }
    type_text(&mut h, "select * from ordrs");
    h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
    ticks(&mut h, 8);
    let text = h.text();
    assert!(text.contains("relation \"ordrs\" does not exist"), "{text}");
    assert!(page(&h).tabs.items[2].error, "the tab carries the error");
    assert_eq!(page(&h).editor.diagnostics.len(), 1, "the name is marked");
}

#[test]
fn closing_every_tab_leaves_an_empty_state_with_a_way_back() {
    let mut h = Harness::on(120, 40, PageId::QueryWorkbench);
    let tabs = page(&h).tabs.id;
    h.focus(tabs);

    h.key(KeyCode::Char('x'));
    h.key(KeyCode::Char('x'));
    assert!(page(&h).tabs.items.is_empty());
    assert!(h.text().contains("No query open"), "{}", h.text());
    h.focus(tabs);
    h.key(KeyCode::Char('n'));
    assert_eq!(page(&h).tabs.items.len(), 1);
}

#[test]
fn every_option_value_changes_the_playground() {
    for opt in options::page_options(PageId::QueryWorkbench) {
        let frame = |v: &str| {
            let mut h = Harness::on(120, 40, PageId::QueryWorkbench);
            h.opt(&format!("{}={v}", opt.key));
            h.text()
        };
        let base = frame(opt.values[opt.default]);
        for v in opt.values.iter().filter(|v| **v != opt.values[opt.default]) {
            let other = frame(v);
            if opt.key == "queryworkbench.close_reveal" {

                assert_ne!(
                    base.matches('×').count(),
                    other.matches('×').count(),
                    "{}={v}",
                    opt.key
                );
            } else {
                assert_ne!(base, other, "{}={v} shows the same as the default", opt.key);
            }
        }
    }
}

#[test]
fn at_80_by_24_the_explorer_yields_to_the_editor_and_results() {
    let h = Harness::on(80, 24, PageId::QueryWorkbench);
    let text = h.text();
    assert!(
        !text.contains("acme_prod"),
        "the explorer collapses:\n{text}"
    );
    assert!(text.contains("Query 1"), "{text}");
    assert!(text.contains("Results"), "{text}");
}

#[test]
fn the_states_section_shows_the_tab_strip_and_explorer_rows() {
    let mut h = Harness::on(120, 40, PageId::QueryWorkbench);
    h.section(Section::States);
    let text = h.text();
    for want in ["Query tabs", "Explorer", "rest", "hover", "focused", "busy"] {
        assert!(text.contains(want), "{want:?}:\n{text}");
    }
}
