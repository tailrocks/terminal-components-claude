



use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Accounts & usage",
        "SC53",
        "Accounts with an inspector, meters and a usage dialog",
    ))
}
