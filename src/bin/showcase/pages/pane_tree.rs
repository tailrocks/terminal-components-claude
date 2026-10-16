//! Pane tree (Layout): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC45 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Pane tree",
        "SC45",
        "PaneTree: nested splits, borders, zoom and resize chords",
    ))
}
