//! Mouse-only navigation in holla (docs/audits/design-review-2026-10/mouse.md):
//! every page can be left, and every level of the page stack reached, by
//! pointer alone.

use crate::app::CRUMBS;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::{Go, Page};

fn nested() -> H {
    let mut h = H::new(Scenario::FirstUse, Motion::Paused, 40, 120, 40);
    h.app.go(Go::Push(Page::DiskOverview));
    h.draw();
    let path = h.app.world.location.cwd.clone();
    h.app.go(Go::Push(Page::Files { path, query: None }));
    h.draw();
    assert_eq!(h.app.tabs[0].stack.len(), 3, "{}", h.text());
    h
}

// mouse #5: a nested page shows its trail, and each earlier level is a link
#[test]
fn the_breadcrumb_goes_back_one_level_or_to_the_root_by_mouse() {
    let mut h = nested();
    let parts = h.app.crumbs.segments.clone();
    assert_eq!(parts.len(), 3, "{parts:?}");
    // one level back: the middle link
    let mid = h.app.crumbs.areas[1];
    assert_eq!(h.app.hits.hit(mid.as_position()), Some(CRUMBS.child(1)));
    h.click(mid.x, mid.y);
    assert_eq!(h.app.tabs[0].stack.len(), 2, "{}", h.text());
    // to the root: the first link
    let root = h.app.crumbs.areas[0];
    h.click(root.x, root.y);
    assert_eq!(h.app.tabs[0].stack.len(), 1, "{}", h.text());
    assert!(
        h.app.crumbs.segments.is_empty(),
        "the root page shows no trail"
    );
}

#[test]
fn the_current_page_in_the_trail_is_not_a_link() {
    let mut h = nested();
    let last = h.app.crumbs.areas[2];
    assert_eq!(h.app.hits.hit(last.as_position()), None);
    h.click(last.x, last.y);
    assert_eq!(h.app.tabs[0].stack.len(), 3);
}
