//! First leaf: `SC-BASE-overview` at 80×24 truecolor through `App::update`/`App::draw`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "integration assertions"
)]

use oracle_showcase::{
    ColorSpec, OraclePageId, PRODUCT, ShowcaseDriver, TerminalSize, capture_case,
    sc_base_overview_80x24_truecolor,
};

#[test]
fn sc_base_overview_80x24_truecolor_uses_production_update_draw() {
    assert_eq!(PRODUCT, "oracle-showcase");
    let case = sc_base_overview_80x24_truecolor();
    assert_eq!(case.scenario_id, "SC-BASE-overview");
    assert_eq!(case.page, OraclePageId::Overview);
    assert_eq!(case.size, TerminalSize::new(80, 24));
    assert_eq!(case.color, ColorSpec::TrueColor);

    let mut driver = ShowcaseDriver::open(case.page, case.size, case.color).unwrap();
    driver.execute(&case.program).unwrap();
    let frame = driver.observe().unwrap();

    assert_eq!(frame.oracle_page, OraclePageId::Overview);
    assert_eq!(frame.production_page, Some(OraclePageId::Overview));
    assert!(!frame.production_route_absent);
    assert_eq!(frame.size, TerminalSize::new(80, 24));
    assert_eq!(frame.color, ColorSpec::TrueColor);
    assert!(frame.is_complete(), "80×24 frame must record every cell");
    assert_eq!(frame.cells.len(), 80 * 24);
    assert_eq!(driver.app().page(), showcase_app::PageId::Overview);

    let text = frame.text();
    assert!(
        text.contains("Overview"),
        "production overview shell must paint its route title\n{text}"
    );
    assert!(
        !text.is_empty(),
        "fresh draw through App::draw must publish a nonempty frame"
    );

    let via_capture = capture_case(case.page, case.size, case.color, &case.program).unwrap();
    assert_eq!(via_capture.cells.len(), frame.cells.len());
    assert_eq!(via_capture.production_page, frame.production_page);
}
