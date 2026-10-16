//! Status & hints (App shell): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC4 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Status & hints",
        "SC4",
        "Hint-bar layers, alignment and styles, the EDIT badge and the reveal-key advert",
    ))
}
