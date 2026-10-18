use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Master-detail",
        "SC38",
        "MasterDetail: wide split, narrow summary and drawer",
    ))
}
