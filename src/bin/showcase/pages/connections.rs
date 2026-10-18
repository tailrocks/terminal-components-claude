



use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Connections",
        "SC47",
        "A connection manager composed from library components only",
    ))
}
