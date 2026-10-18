



use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Meters",
        "SC36",
        "MeterList with tones and levels",
    ))
}
