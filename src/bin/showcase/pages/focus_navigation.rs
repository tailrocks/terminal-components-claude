//! Focus & navigation (App shell): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC28 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Focus & navigation",
        "SC28",
        "Navigation policies, focus fallback, keyboard protocol and key-map overrides",
    ))
}
