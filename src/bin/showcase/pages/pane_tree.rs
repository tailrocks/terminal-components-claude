



use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Pane tree",
        "SC45",
        "PaneTree: nested splits, borders, zoom and resize chords",
    ))
}
