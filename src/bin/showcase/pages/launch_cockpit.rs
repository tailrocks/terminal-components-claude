



use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Launch cockpit",
        "SC55",
        "A launch cockpit: step rail stages, build log and handoff",
    ))
}
