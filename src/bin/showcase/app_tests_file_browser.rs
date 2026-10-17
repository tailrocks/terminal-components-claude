//! The File browser page through the real app (plan §7 SC41): the page
//! shows the library `FileBrowser`'s demo, not the planned-page stub;
//! Browse… opens the modal on the shell's layer stack, its keys walk the
//! fixture, a choice or a dismissal comes back to the page, and the
//! options drive the browser's builders.

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::PageId;
use crate::app_tests_feedback::Harness;
use crate::pages::template::Section;
use junie_tui::core::id::WidgetId;

const BROWSE: WidgetId = WidgetId::of("filebrowser").sub("browse");

fn open(h: &mut Harness) {
    h.focus(BROWSE);
    h.key(KeyCode::Enter);
}

/// The results the page logged, oldest first.
fn log(h: &Harness) -> Vec<String> {
    h.app.pages[PageId::FileBrowser.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<crate::pages::file_browser::FileBrowserPage>())
        .expect("file browser page")
        .log
        .clone()
}

fn type_str(h: &mut Harness, s: &str) {
    for c in s.chars() {
        h.key(KeyCode::Char(c));
    }
}

#[test]
fn file_browser_page_shows_its_demo_not_the_stub() {
    for (w, hh) in [(120, 40), (80, 24)] {
        let h = Harness::new(w, hh, PageId::FileBrowser);
        let text = h.text();
        assert!(!text.contains("arrives with"), "{text}");
        for needle in ["New mount", "Browse…", "Host folder"] {
            assert!(text.contains(needle), "{w}: {needle}: {text}");
        }
    }
}

#[test]
fn browse_opens_the_modal_and_space_chooses_a_folder() {
    let mut h = Harness::new(120, 40, PageId::FileBrowser);
    open(&mut h);
    let text = h.text();
    for needle in [
        "Choose a folder",
        "~ › src",
        "payments-platform/",
        "git · main",
        "Mount read-only",
        "Show hidden files",
        "Filter",
    ] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
    // the footer names the browser's keys
    assert!(text.contains("Space Choose"), "{text}");
    // type-ahead to payments-platform, into it, back up
    h.key(KeyCode::Char('p'));
    h.key(KeyCode::Enter);
    let text = h.text();
    assert!(
        text.contains("crates/") && text.contains("Cargo.toml"),
        "{text}"
    );
    assert!(text.contains("~/src/payments-platform"), "{text}");
    h.key(KeyCode::Backspace);
    assert!(h.text().contains("customer-portal/"), "{}", h.text());
    h.key(KeyCode::Char('p'));
    h.key(KeyCode::Char(' '));
    let text = h.text();
    assert!(!text.contains("Choose a folder"), "closed: {text}");
    assert!(
        text.contains("Host folder   ~/src/payments-platform"),
        "{text}"
    );
    assert!(
        text.contains("Chosen { path: ~/src/payments-platform, checked: false }"),
        "{text}"
    );
}

#[test]
fn esc_cancels_and_the_policy_decides() {
    let mut h = Harness::new(120, 40, PageId::FileBrowser);
    open(&mut h);
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("Choose a folder"));
    assert_eq!(log(&h), ["Cancelled"]);
    h.opt("filebrowser.dismiss=never");
    open(&mut h);
    h.key(KeyCode::Esc);
    assert!(h.text().contains("Choose a folder"), "Esc refused");
    h.app
        .shell
        .focus
        .focus(WidgetId::of("filebrowser").sub("browser").sub("cancel"));
    h.key(KeyCode::Enter);
    assert!(!h.text().contains("Choose a folder"), "Cancel still closes");
}

#[test]
fn an_outside_click_dismisses_under_light() {
    let mut h = Harness::new(120, 40, PageId::FileBrowser);
    h.opt("filebrowser.dismiss=light");
    open(&mut h);
    assert!(h.text().contains("Choose a folder"));
    h.click(2, 38);
    assert!(!h.text().contains("Choose a folder"), "{}", h.text());
    assert!(h.text().contains("Cancelled"), "{}", h.text());
}

#[test]
fn hidden_files_toggle_with_the_dot_and_the_filter_narrows() {
    let mut h = Harness::new(120, 40, PageId::FileBrowser);
    open(&mut h);
    h.key(KeyCode::Backspace);
    assert!(!h.text().contains(".config/"));
    h.key(KeyCode::Char('.'));
    assert!(h.text().contains(".config/"), "{}", h.text());
    assert!(h.text().contains("[✓] Show hidden files"), "{}", h.text());
    h.key_mod(KeyCode::Char('f'), KeyModifiers::CONTROL);
    type_str(&mut h, "no");
    let text = h.text();
    assert!(
        text.contains("notes/") && !text.contains("design/"),
        "{text}"
    );
}

#[test]
fn a_typed_location_opens_it() {
    let mut h = Harness::new(120, 40, PageId::FileBrowser);
    open(&mut h);
    h.key(KeyCode::Char('/'));
    // the field edits `~/src` with the caret at the end
    for _ in 0..3 {
        h.key(KeyCode::Backspace);
    }
    type_str(&mut h, "design");
    h.key(KeyCode::Enter);
    let text = h.text();
    assert!(
        text.contains("tokens/") && text.contains("brand.fig"),
        "{text}"
    );
}

#[test]
fn options_drive_the_browser() {
    let mut h = Harness::new(120, 40, PageId::FileBrowser);
    h.opt("filebrowser.choose=files");
    h.opt("filebrowser.path_bar=off");
    h.opt("filebrowser.filter=off");
    h.opt("filebrowser.checkbox=off");
    h.opt("filebrowser.url_entry=on");
    h.opt("filebrowser.stepper=on");
    h.opt("filebrowser.keys=vim");
    open(&mut h);
    let text = h.text();
    assert!(text.contains("Open a file"), "{text}");
    assert!(!text.contains("~ › src"), "{text}");
    assert!(!text.contains("Mount read-only"), "{text}");
    assert!(text.contains("Git URL…"), "{text}");
    assert!(text.contains("✓ Source · ● Folder"), "{text}");
    assert!(text.contains("g Git URL"), "vim keys: {text}");
    // vim `g` switches to the URL entry; a known repository answers
    h.key(KeyCode::Char('g'));
    assert!(h.text().contains("Browse folders"), "{}", h.text());
    type_str(&mut h, "github.com/acme/customer-portal");
    h.key(KeyCode::Enter);
    let text = h.text();
    assert!(
        text.contains("Url(github.com/acme/customer-portal)"),
        "{text}"
    );
}

#[test]
fn states_show_the_listing_and_the_toggle() {
    let mut h = Harness::new(120, 40, PageId::FileBrowser);
    h.section(Section::States);
    let text = h.text();
    for needle in [
        "Listing",
        "Hidden toggle",
        "rest",
        "hover",
        "focused",
        "empty",
    ] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}
