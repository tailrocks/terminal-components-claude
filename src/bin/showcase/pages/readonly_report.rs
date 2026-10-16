//! Read-only report (Screens): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC56 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Read-only report",
        "SC56",
        "A read-only report: live meta, meters, facts and refresh",
    ))
}
