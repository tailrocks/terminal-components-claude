




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
        "scope ‹ here ›",
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

    let r = page(&h).query.readout_area;
    assert_eq!(r.y, page(&h).query.area.y, "on the field row");
    assert!(h.text().contains("scope ‹ here ›"));
    h.click(r.x + 2, r.y);
    assert_eq!(page(&h).scope, 1);
    assert!(h.text().contains("scope ‹ project ›"));
    assert!(h.text().contains("Compose up"), "{}", h.text());

    h.focus(page(&h).query.id);
    h.key_mod(KeyCode::Up, KeyModifiers::ALT);
    assert_eq!(page(&h).scope, 2);
    h.key_mod(KeyCode::Down, KeyModifiers::ALT);
    h.key_mod(KeyCode::Down, KeyModifiers::ALT);
    assert_eq!(page(&h).scope, 0);
    assert!(!h.text().contains("Compose up"));
    assert_eq!(page(&h).query.text(), "co", "the keys never typed");

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
