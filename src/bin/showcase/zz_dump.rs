//! scratch (not committed): dump a page's Playground screens
use crate::app::NAV_ENTRIES;
use crate::app_tests_feedback::Harness;
use crate::pages::template::Section;

#[test]
fn zz_dump() {
    let Ok(page) = std::env::var("DUMP_PAGE") else {
        return;
    };
    let sizes = std::env::var("DUMP_SIZES").unwrap_or("80x24,120x40,220x70".into());
    let id = NAV_ENTRIES
        .iter()
        .find(|e| format!("{:?}", e.id) == page)
        .unwrap()
        .id;
    let mut out = String::new();
    for s in sizes.split(',') {
        let (w, h) = s.split_once('x').unwrap();
        let mut hh = Harness::new(w.parse().unwrap(), h.parse().unwrap(), id);
        hh.section(Section::Playground);
        out.push_str(&format!("===== {s} first\n{}\n", hh.text()));
        if std::env::var("DUMP_ALL").is_ok() {
            out.push_str(&format!("===== {s} all\n{}\n", hh.page_text()));
        }
    }
    std::fs::write("/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/reach-r34-r44/dump.txt", out).unwrap();
}
