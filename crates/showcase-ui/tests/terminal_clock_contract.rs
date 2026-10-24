use showcase_ui::{App, PageId};
use termrock::{KeyCode, Theme};
use termrock_test_support::Harness;

#[test]
fn terminal_progress_waits_for_its_due_tick_after_a_route_return() {
    let mut harness = Harness::new(App::with_page(PageId::Terminal), Theme::junie(), 120, 40);
    assert!(harness.text().contains("0 of 7 · running"));

    for tick in 1_u32..=5 {
        let _ = harness
            .advance_to(termrock::Moment::from_millis(u64::from(tick) * 16))
            .expect("Terminal's next clock deadline is due");
    }
    assert_eq!(harness.now(), termrock::Moment::from_millis(80));
    assert!(harness.text().contains("0 of 7 · running"));

    let _ = harness.key(KeyCode::Char('['));
    assert!(
        harness.text().contains("Diff viewer"),
        "the previous catalog entry displays its live Diff page"
    );
    let _ = harness.key(KeyCode::Char(']'));

    assert_eq!(harness.app().page(), PageId::Terminal);
    assert_eq!(harness.now(), termrock::Moment::from_millis(80));
    assert!(
        harness.text().contains("0 of 7 · running"),
        "the route-return Event preserves the live page without spending its next tick"
    );

    let _ = harness
        .advance_to(termrock::Moment::from_millis(96))
        .expect("the retained Terminal deadline is now due");
    assert!(harness.text().contains("1 of 7 · running"));
}
