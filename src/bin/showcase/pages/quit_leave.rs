//! Quit & leave (App shell): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC6 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Quit & leave",
        "SC6",
        "QuitKeys, the leave guard with Clean, Unsaved and Running stakes, and its dialogs",
    ))
}
