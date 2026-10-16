//! Tabs (Navigation): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC14 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Tabs",
        "SC14",
        "Tab items (icons, dirty, busy, error, closable), rules, overflow and nested strips",
    ))
}
