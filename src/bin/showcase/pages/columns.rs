//! Columns (Tables): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC11 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Columns",
        "SC11",
        "Column sizing (content, fixed, flex, droppable), resizable seams and the header menu",
    ))
}
