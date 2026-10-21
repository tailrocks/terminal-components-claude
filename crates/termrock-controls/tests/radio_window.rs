#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]

use ratatui_core::layout::Rect;
use termrock_controls::choice::{RadioGroup, RadioGroupAction, RadioGroupState};
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_runtime::{App, Cx, Response, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const RADIO: Id = Id::root("controls.radio.window");
const ITEMS: [&str; 10] = [
    "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
];

#[derive(Default)]
struct WindowApp {
    state: RadioGroupState,
    selected: usize,
}

impl App for WindowApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = RadioGroup::new(RADIO)
            .value(ItemKey::index(self.selected))
            .update(cx, &mut self.state, &ITEMS);
        if let Some(RadioGroupAction::Chose(ItemKey::Index(index))) = response.action_ref() {
            self.selected = *index;
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        RadioGroup::new(RADIO).draw_window(
            ui,
            Rect {
                x: 0,
                y: 0,
                width: 30,
                height: 2,
            },
            &self.state,
            &ITEMS,
            8,
        );
    }
}

fn harness() -> Harness<WindowApp> {
    Harness::new(WindowApp::default(), Theme::junie(), 30, 2)
}

#[test]
fn window_paints_and_registers_original_rows_only() {
    let h = harness();
    assert!(h.text().contains("iota"));
    assert!(h.text().contains("kappa"));
    assert!(!h.text().contains("alpha"));
    assert_eq!(
        h.area_of_part(RADIO, PartRef::item(Part::ROW, ItemKey::index(8))),
        Some(Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 1
        })
    );
    assert_eq!(
        h.area_of_part(RADIO, PartRef::item(Part::ROW, ItemKey::index(9))),
        Some(Rect {
            x: 0,
            y: 1,
            width: 30,
            height: 1
        })
    );
    assert_eq!(
        h.area_of_part(RADIO, PartRef::item(Part::ROW, ItemKey::index(0))),
        None
    );
}

#[test]
fn pointer_activation_targets_the_original_option_index() {
    let mut h = harness();
    let _ = h.click_part(RADIO, PartRef::item(Part::ROW, ItemKey::index(8)));
    assert_eq!(h.app().selected, 8);
    assert_eq!(h.app().state.cursor_index(), 8);
}

#[test]
fn empty_or_past_end_window_does_not_register_a_control() {
    struct EmptyWindow;
    impl App for EmptyWindow {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            RadioGroup::new(RADIO).draw_window(
                ui,
                Rect {
                    x: 0,
                    y: 0,
                    width: 30,
                    height: 2,
                },
                &RadioGroupState::default(),
                &ITEMS,
                ITEMS.len(),
            );
        }
    }
    let h = Harness::new(EmptyWindow, Theme::junie(), 30, 2);
    assert_eq!(h.area_of(RADIO), None);
}
