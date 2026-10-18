



use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Notices",
        "SC36",
        "Notices and inline error blocks",
    ))
}
