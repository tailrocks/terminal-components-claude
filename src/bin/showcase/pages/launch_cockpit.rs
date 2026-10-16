//! Launch cockpit (Screens): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC55 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Launch cockpit",
        "SC55",
        "A launch cockpit: step rail stages, build log and handoff",
    ))
}
