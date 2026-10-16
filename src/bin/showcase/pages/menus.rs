//! Menus (Navigation): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC16 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Menus",
        "SC16",
        "Context menus: placement, anchors, reasons, danger items and press-drag-release",
    ))
}
