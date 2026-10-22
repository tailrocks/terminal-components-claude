use core::time::Duration;

use ratatui_core::layout::Rect;
use termrock_controls::Toggle;
use termrock_core::id::Id;
use termrock_core::intent::{Intent, Phase};
use termrock_core::response::Response;
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const TOGGLE_ID: Id = Id::root("controls.rapid_pointer_toggle");
const AREA: Rect = Rect::new(0, 0, 20, 1);

#[derive(Default)]
struct TogglePage {
    enabled: bool,
    activations: usize,
    pointer_events: Vec<(Phase, Option<u8>)>,
}

impl App for TogglePage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for intent in cx.intents(TOGGLE_ID) {
            if let Intent::Pointer { phase, part, .. } = intent
                && matches!(
                    phase,
                    Phase::Press | Phase::Release | Phase::Click | Phase::DoubleClick
                )
            {
                self.pointer_events
                    .push((phase, cx.pointer_click_count(TOGGLE_ID, part)));
            }
        }

        if Toggle::new(TOGGLE_ID, "Verbose")
            .update(cx, &mut self.enabled)
            .activated()
        {
            self.activations += 1;
            Response::changed()
        } else {
            Response::ignored()
        }
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let _ = Toggle::new(TOGGLE_ID, "Verbose")
            .on(self.enabled)
            .draw(ui, AREA);
    }
}

#[test]
fn rapid_pointer_pairs_activate_toggle_once_and_reset_after_double_click_window() {
    let theme = Theme::junie();
    let window = theme.design.motion.double_click_ms;
    let mut harness = Harness::new(TogglePage::default(), theme, 20, 1);

    let _ = harness.mouse(termrock_core::event::MouseKind::Down, 6, 0);
    assert_eq!(
        (harness.app().enabled, harness.app().activations),
        (false, 0)
    );
    assert_eq!(
        harness.app().pointer_events,
        [(Phase::Press, Some(1))],
        "the first down publishes press count one without activation"
    );

    let _ = harness.mouse(termrock_core::event::MouseKind::Up, 6, 0);
    assert_eq!(
        (harness.app().enabled, harness.app().activations),
        (true, 1)
    );
    assert_eq!(
        harness.app().pointer_events,
        [
            (Phase::Press, Some(1)),
            (Phase::Release, Some(1)),
            (Phase::Click, Some(1)),
        ]
    );

    let _ = harness.advance(Duration::from_millis(100));
    let _ = harness.mouse(termrock_core::event::MouseKind::Down, 6, 0);
    assert_eq!(
        (harness.app().enabled, harness.app().activations),
        (true, 1)
    );
    let _ = harness.mouse(termrock_core::event::MouseKind::Up, 6, 0);
    assert_eq!(
        (harness.app().enabled, harness.app().activations),
        (false, 2)
    );
    assert_eq!(
        harness.app().pointer_events,
        [
            (Phase::Press, Some(1)),
            (Phase::Release, Some(1)),
            (Phase::Click, Some(1)),
            (Phase::Press, Some(2)),
            (Phase::Release, Some(2)),
            (Phase::DoubleClick, Some(2)),
        ],
        "the second rapid pair reaches Toggle as one DoubleClick activation"
    );

    let _ = harness.advance(Duration::from_millis(window + 1));
    let _ = harness.mouse(termrock_core::event::MouseKind::Down, 6, 0);
    assert_eq!(
        (harness.app().enabled, harness.app().activations),
        (false, 2)
    );
    let _ = harness.mouse(termrock_core::event::MouseKind::Up, 6, 0);
    assert_eq!(
        (harness.app().enabled, harness.app().activations),
        (true, 3)
    );
    assert_eq!(
        harness.app().pointer_events,
        [
            (Phase::Press, Some(1)),
            (Phase::Release, Some(1)),
            (Phase::Click, Some(1)),
            (Phase::Press, Some(2)),
            (Phase::Release, Some(2)),
            (Phase::DoubleClick, Some(2)),
            (Phase::Press, Some(1)),
            (Phase::Release, Some(1)),
            (Phase::Click, Some(1)),
        ],
        "time beyond the click window restarts the count and returns to Click"
    );
}
