//! Ported from the PR #26 reference `tests/mouse_targets.rs`
//! (junie_tui at f4867e9b5f8a08a3bb61ce80d644198a29f51984), sliced to the
//! controls registry.
//!
//! Kept from the reference `check` helper: every clickable region a control
//! registers actually routes to its owner (a click at the region position
//! reaches the widget and changes its state), and a click outside the
//! widget changes nothing. The reference's cross-widget slice (collections,
//! strips, overlays) lives with the crates that own those registries; the
//! navigation slice is ported alongside this one.

use ratatui_core::layout::Rect;
use termrock_controls::Button;
use termrock_controls::choice::{Checkbox, RadioGroup, RadioGroupState, Toggle};
use termrock_core::id::{Id, ItemKey};
use termrock_core::response::Response;
use termrock_runtime::hit::Region;
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const BUTTON: Id = Id::root("mt.button");
const CHECK: Id = Id::root("mt.check");
const TOGGLE: Id = Id::root("mt.toggle");
const RADIO: Id = Id::root("mt.radio");

const RADIO_ITEMS: [&str; 3] = ["alpha", "beta", "gamma"];

struct ControlsPage {
    activations: usize,
    checked: bool,
    on: bool,
    selected: usize,
    state: RadioGroupState,
}

impl App for ControlsPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut acc = Response::ignored();
        if Button::new(BUTTON, "Run").update(cx).activated() {
            self.activations += 1;
            acc |= Response::changed();
        }
        if Checkbox::new(CHECK, "Check")
            .update(cx, &mut self.checked)
            .activated()
        {
            acc |= Response::changed();
        }
        if Toggle::new(TOGGLE, "Toggle")
            .update(cx, &mut self.on)
            .activated()
        {
            acc |= Response::changed();
        }
        let radio = RadioGroup::new(RADIO).update(cx, &mut self.state, &RADIO_ITEMS);
        if let Some(termrock_controls::choice::RadioGroupAction::Chose(ItemKey::Index(index))) =
            radio.action_ref()
        {
            self.selected = *index;
            acc |= Response::changed();
        }
        acc
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(BUTTON, "Run").draw(ui, Rect::new(1, 1, 12, 1));
        Checkbox::new(CHECK, "Check")
            .checked(self.checked)
            .draw(ui, Rect::new(1, 3, 12, 1));
        Toggle::new(TOGGLE, "Toggle")
            .on(self.on)
            .draw(ui, Rect::new(1, 5, 12, 1));
        RadioGroup::new(RADIO).draw(ui, Rect::new(1, 7, 12, 3), &self.state, &RADIO_ITEMS);
    }
}

fn owned_regions(h: &Harness<ControlsPage>, id: Id) -> Vec<Region> {
    h.runtime()
        .registry()
        .regions()
        .iter()
        .filter(|r| r.owner == id || r.id == id)
        .cloned()
        .collect()
}

#[test]
fn every_control_region_routes_to_its_owner() {
    let mut h = Harness::new(
        ControlsPage {
            activations: 0,
            checked: false,
            on: false,
            selected: usize::MAX,
            state: RadioGroupState::default(),
        },
        Theme::junie(),
        30,
        12,
    );
    let button_regions = owned_regions(&h, BUTTON);
    assert!(!button_regions.is_empty(), "button registers a region");
    for region in &button_regions {
        let before = h.app().activations;
        let _ = h.click(region.area.x, region.area.y);
        assert_eq!(
            h.app().activations,
            before + 1,
            "button region {:?} routes to its owner",
            region.part
        );
    }

    let check_regions = owned_regions(&h, CHECK);
    for region in &check_regions {
        let before = h.app().checked;
        let _ = h.click(region.area.x, region.area.y);
        assert_ne!(
            h.app().checked,
            before,
            "checkbox region {:?} toggles the value",
            region.part
        );
    }

    let toggle_regions = owned_regions(&h, TOGGLE);
    for region in &toggle_regions {
        let before = h.app().on;
        let _ = h.click(region.area.x, region.area.y);
        assert_ne!(
            h.app().on,
            before,
            "toggle region {:?} flips the value",
            region.part
        );
    }

    let radio_regions = owned_regions(&h, RADIO);
    assert!(
        radio_regions.len() >= RADIO_ITEMS.len(),
        "each option is hit tested"
    );
    for region in &radio_regions {
        let _ = h.click(region.area.x, region.area.y);
        let chosen = h.app().selected;
        assert_ne!(
            chosen,
            usize::MAX,
            "radio region {:?} selects an option",
            region.part
        );
    }

    // A click on nothing routes to no one.
    let before = (
        h.app().activations,
        h.app().checked,
        h.app().on,
        h.app().selected,
    );
    let _ = h.click(25, 10);
    assert_eq!(
        (
            h.app().activations,
            h.app().checked,
            h.app().on,
            h.app().selected
        ),
        before,
        "a click outside every control changes nothing"
    );
}
