//! Disk cleanup (Screens): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC49 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Disk cleanup",
        "SC49",
        "A disk-cleanup flow: usage tree, meters, checklist and gated review",
    ))
}
