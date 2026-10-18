use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Read-only report",
        "SC56",
        "A read-only report: live meta, meters, facts and refresh",
    ))
}
