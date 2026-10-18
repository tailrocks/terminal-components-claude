



use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Workspaces",
        "SC52",
        "A workspaces screen: tree, detail, roster and narrow drawer",
    ))
}
