//! The Finder screen (plan §7.5, SC48), driven through the real
//! showcase: typing ranks live, the scope widens the catalogue, Enter
//! runs, Alternatives… and Why? open over the page, and every option
//! changes the frame.

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::options;
use crate::pages::finder::FinderPage;
use crate::pages::template::Section;
use junie_tui::widgets::info_dialog::InfoDialog;

fn page(h: &Harness) -> &FinderPage {
    h.app.pages[PageId::Finder.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<FinderPage>())
        .expect("the Finder page")
}

#[test]
fn the_finder_ranks_the_query_and_previews_the_best_match() {
    let h = Harness::on(120, 40, PageId::Finder);
    let text = h.text();
    assert!(!text.contains("arrives with SC48"), "{text}");
    for want in [
        "Here",
        "Everywhere",
        "Results",
        "Pull",
        "Push",
        "git pull --ff-only",
        "Alternatives…",
        "Why?",
    ] {
        assert!(text.contains(want), "{want:?} missing:\n{text}");
    }
    assert!(!text.contains("Build"), "only matches are listed");
}

#[test]
fn typing_ranks_live_and_the_scope_widens_the_catalogue() {
    let mut h = Harness::on(120, 40, PageId::Finder);
    h.opt("finder.query=empty");
    h.focus(page(&h).query.id);
    for c in "co".chars() {
        h.key(KeyCode::Char(c));
    }
    assert!(!h.text().contains("Compose up"), "out of scope here");
    let (x, y) = h.find("Project").unwrap();
    h.click(x + 1, y);
    assert!(h.text().contains("Compose up"), "{}", h.text());
    // a query nothing matches says so
    h.opt("finder.query=nomatch");
    assert!(h.text().contains("No matches"), "{}", h.text());
}

#[test]
fn enter_on_a_result_runs_it() {
    let mut h = Harness::on(120, 40, PageId::Finder);
    h.focus(page(&h).results.id);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("✓ Ran Push"), "{}", h.text());
}

#[test]
fn alternatives_open_a_menu_and_why_explains_the_rank() {
    let mut h = Harness::on(120, 40, PageId::Finder);
    let (x, y) = h.find("Alternatives…").unwrap();
    h.click(x + 1, y);
    assert!(h.text().contains("Fetch only"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(h.text().contains("✓ Ran Fetch only"), "{}", h.text());
    // the context key on the results opens the same menu
    h.focus(page(&h).results.id);
    h.key_mod(KeyCode::F(10), KeyModifiers::SHIFT);
    assert!(h.text().contains("Pull with rebase"), "{}", h.text());
    h.key(KeyCode::Esc);
    let (x, y) = h.find("Why?").unwrap();
    h.click(x + 1, y);
    assert!(h.app.layers.top_as::<InfoDialog>().is_some());
    assert!(h.text().contains("Why Pull?"), "{}", h.text());
    assert!(h.text().contains("starts the name"), "{}", h.text());
}

#[test]
fn every_option_value_changes_the_playground() {
    for opt in options::page_options(PageId::Finder) {
        let frame = |v: &str| {
            let mut h = Harness::on(120, 40, PageId::Finder);
            h.opt(&format!("{}={v}", opt.key));
            h.text()
        };
        let base = frame(opt.values[opt.default]);
        for v in opt.values.iter().filter(|v| **v != opt.values[opt.default]) {
            assert_ne!(
                base,
                frame(v),
                "{}={v} shows the same as the default",
                opt.key
            );
        }
    }
}

#[test]
fn a_narrow_page_folds_the_preview_to_one_line() {
    let h = Harness::on(80, 24, PageId::Finder);
    let text = h.text();
    assert!(text.contains("→ git pull --ff-only"), "{text}");
    assert!(!text.contains("Alternatives…"), "{text}");
}

#[test]
fn the_states_section_shows_the_scope_strip_and_the_results() {
    let mut h = Harness::on(120, 40, PageId::Finder);
    h.section(Section::States);
    let text = h.text();
    for want in ["Scope", "Results", "empty", "No matches"] {
        assert!(text.contains(want), "{want:?}:\n{text}");
    }
}
