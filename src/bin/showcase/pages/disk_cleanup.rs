



use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Disk cleanup",
        "SC49",
        "A disk-cleanup flow: usage tree, meters, checklist and gated review",
    ))
}
