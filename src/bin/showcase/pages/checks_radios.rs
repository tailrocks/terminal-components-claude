//! Checks & radios (Inputs & forms): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC9 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Checks & radios",
        "SC9",
        "Checkbox (checked, mixed, disabled, hit areas), RadioGroup and Toggle with their States grids",
    ))
}
