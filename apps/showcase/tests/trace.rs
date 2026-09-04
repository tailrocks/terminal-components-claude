#![allow(missing_docs, unused_must_use)]

use showcase_app::{App, PageId};
use tui_next::{KeyCode, Theme};
use tui_next_testing::Harness;

fn dump(label: &str, h: &Harness<App>) {
    println!("{label}: focus={:?}", h.focus());
    for line in h.text().lines().filter(|line| {
        let line = line.trim_end();
        ["Short", "Fix", "Required", "Creating", "summary", "details"]
            .iter().any(|needle| line.contains(needle))
    }) {
        println!("{}", line.trim_end());
    }
}

#[test]
fn trace_forms() {
    let mut h = Harness::new(App::with_page(PageId::Forms), Theme::junie(), 120, 40);
    dump("initial", &h);
    let _ = h.ctrl('s'); dump("submit", &h);
    h.draw(); dump("settle", &h);
    for y in 0..40 { let row = h.row(y); if !row.trim().is_empty() { println!("row {y}: {row}"); } }
    let (x, y) = (40, 7);
    let _ = h.click(x, y); dump("click", &h);
    let _ = h.type_str("Fix login bug"); dump("typed", &h);
    let _ = h.key(KeyCode::Enter); dump("commit", &h);
    let _ = h.ctrl('s'); dump("done", &h);
}
