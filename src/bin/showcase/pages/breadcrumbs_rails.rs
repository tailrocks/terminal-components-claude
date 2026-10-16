//! Breadcrumbs & rails (App shell): a planned page (plan §7.2), registered with its
//! `--page` slug so parallel steps do not collide. SC5 replaces this
//! stub with the page.

use crate::pages::Page;
use crate::pages::stub::StubPage;

pub fn page() -> Box<dyn Page> {
    Box::new(StubPage::new(
        "Breadcrumbs & rails",
        "SC5",
        "Breadcrumb trails (static, clickable, collapsing) and the collapsible SidebarRail",
    ))
}
