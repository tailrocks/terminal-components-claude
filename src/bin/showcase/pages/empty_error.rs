//! Empty & error (Feedback): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC21 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Empty & error",
        "SC21",
        "EmptyState and its error variant, actions, and read-only explanations",
    ))
}
