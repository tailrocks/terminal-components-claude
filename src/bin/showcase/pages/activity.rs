//! Activity (Screens): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC51 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Activity",
        "SC51",
        "An activity screen: stream chips, a live log with find, attach and detach",
    ))
}
