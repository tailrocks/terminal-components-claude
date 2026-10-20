use termrock::author::composition_leaf;
use termrock::layout::{ChildPolicy, Composition, Insets, OverflowPolicy};
use termrock::runtime::{CompositionSlots, ContentSlot};
use termrock::{App, Cx, Id, Response, Size, TextInput, TextInputState, Theme, Ui};
use termrock_test_support::Harness;

const INPUT: Id = Id::root("composition.cursor.input");

struct Page {
    value: String,
    state: TextInputState,
}

impl App for Page {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn min_size(&self) -> Size {
        Size::exact(0, 0)
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let fixed = composition_leaf(|_: &Ui<'_>, _| Size::exact(6, 1), |_: &mut Ui<'_>, _| {});
        let input = TextInput::new(INPUT).value(&self.value);
        let input_leaf = composition_leaf(
            |ui: &Ui<'_>, constraints| input.measure(ui, constraints),
            |ui: &mut Ui<'_>, area| {
                input.draw(ui, area, &self.state);
            },
        );
        let slots = CompositionSlots::row([
            ContentSlot::new(&fixed, ChildPolicy::default()),
            ContentSlot::new(
                &input_leaf,
                ChildPolicy::default().min(8, 1).max(Some(8), None),
            ),
        ])
        .spec(
            Composition::new()
                .padding(Insets {
                    l: 0,
                    t: 0,
                    r: 4,
                    b: 0,
                })
                .overflow(OverflowPolicy::PreserveMinimum),
        );
        ui.compose(&slots).expect("valid composition");
    }
}

#[test]
fn clipped_text_input_does_not_relocate_its_offscreen_caret() {
    let value = String::from("abcd");
    let mut state = TextInputState::default();
    state.begin(&value);
    state.place_cursor(4, 6);

    let harness = Harness::new(Page { value, state }, Theme::junie(), 14, 2);

    assert_eq!(harness.focus(), Some(INPUT));
    let visible = harness.area_of(INPUT).expect("visible input area");
    assert!(
        harness.cursor().is_none(),
        "offscreen caret must not be relocated into visible area {visible:?}"
    );
}
