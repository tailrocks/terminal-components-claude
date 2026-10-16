//! Meters (Feedback): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC36 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Meters",
        "SC36",
        "MeterList with tones and levels",
    ))
}
