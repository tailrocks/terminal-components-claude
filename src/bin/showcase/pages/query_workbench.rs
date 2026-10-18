



use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Query workbench",
        "SC46",
        "A query workbench composed from library components only",
    ))
}
