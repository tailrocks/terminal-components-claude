use showcase_ui::{App, PageId};
use termrock::{Id, Theme};
use termrock_test_support::Harness;

const APP_NAV: Id = Id::root("showcase_ui::app::navigation");

#[test]
fn debug_nav_regions() {
    let harness = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 120, 40);
    eprintln!("area_of: {:?}", harness.area_of(APP_NAV));
    panic!("debug");
}
