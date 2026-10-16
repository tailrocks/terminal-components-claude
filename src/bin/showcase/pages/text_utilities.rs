//! Text utilities (Text & code): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC26 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Text utilities",
        "SC26",
        "Text helpers (truncation, wrapping, widths), property sheets and PropsList",
    ))
}
