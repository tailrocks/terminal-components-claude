//! Accounts & usage (Screens): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC53 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Accounts & usage",
        "SC53",
        "Accounts with an inspector, meters and a usage dialog",
    ))
}
