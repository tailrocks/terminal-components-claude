use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Finder",
        "SC48",
        "A finder with a ranked, grouped result table, composed from library components",
    ))
}
