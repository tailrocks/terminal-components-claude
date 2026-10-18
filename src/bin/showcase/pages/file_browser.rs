



use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "File browser",
        "SC41",
        "The FileBrowser modal",
    ))
}
