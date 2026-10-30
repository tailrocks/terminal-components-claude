//! Ported from the PR #26 reference `tests/focus_gutter.rs`
//! (junie_tui at f4867e9b5f8a08a3bb61ce80d644198a29f51984), widget-level
//! sweep for the controls crate.
//!
//! Kept from the reference: the focus bar reads `▎` in the rendered buffer
//! exactly while a keyboard-focused, enabled control shows its indicator —
//! at every colour level, so focus stays unambiguous without colour — and
//! disabled controls stay visually distinct (DIM at the mono tier) and
//! inert. The reference's list, text-selection and menu assertions live
//! with the crates that own those widgets.

use ratatui_core::layout::Rect;
use termrock_controls::{Button, ButtonFocusVisibility, ReferenceState, ReferenceTarget};
use termrock_core::id::Id;
use termrock_core::response::Response;
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, Theme, Variant};

const BUTTON: Id = Id::root("focus.gutter.button");

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

const VARIANTS: [Variant; 8] = [
    Variant::DEFAULT,
    Variant::PRIMARY,
    Variant::SECONDARY,
    Variant::SUBTLE,
    Variant::DANGER,
    Variant::TOGGLE,
    Variant::QUIET,
    Variant::GHOST,
];

struct ButtonPage {
    variant: Variant,
    disabled: bool,
    policy: ButtonFocusVisibility,
    activations: usize,
}

impl App for ButtonPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if Button::new(BUTTON, "Action")
            .variant(self.variant)
            .disabled(self.disabled)
            .focus_visible(self.policy)
            .update(cx)
            .activated()
        {
            self.activations += 1;
            Response::changed()
        } else {
            Response::ignored()
        }
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(BUTTON, "Action")
            .variant(self.variant)
            .disabled(self.disabled)
            .focus_visible(self.policy)
            .draw(ui, Rect::new(0, 0, 12, 1));
    }
}

fn page(variant: Variant, disabled: bool) -> ButtonPage {
    ButtonPage {
        variant,
        disabled,
        policy: ButtonFocusVisibility::KeyboardOnly,
        activations: 0,
    }
}

#[test]
fn button_focus_remains_unambiguous_without_colour() {
    for level in LEVELS {
        let theme = Theme::junie().for_level(level);
        for variant in VARIANTS {
            // Keyboard focus: the bar glyph occupies the gutter cell. The
            // harness pre-focuses the first control without visibility, so
            // blur first and tab back in.
            let mut focused = Harness::new(page(variant, false), theme.clone(), 12, 1);
            focused.blur();
            let _ = focused.key(termrock_core::keys::KeyCode::Tab);
            assert!(focused.focus_visible(), "{level:?} {variant:?}");
            assert_eq!(
                focused.cell(0, 0).symbol(),
                "▎",
                "{level:?} {variant:?}: keyboard focus paints the bar"
            );

            // Unfocused: a blank, reserved gutter cell.
            let mut unfocused = Harness::new(page(variant, false), theme.clone(), 12, 1);
            unfocused.blur();
            assert_eq!(
                unfocused.cell(0, 0).symbol(),
                " ",
                "{level:?} {variant:?}: without focus the gutter is blank"
            );

            // Disabled: the bar is blank. (The recipe resolves a bar for a
            // focus-visible disabled state — see frozen_ink_pins — but the
            // widget never requests focus visibility while disabled, so the
            // rendered cell is blank in both cases.)
            struct DisabledFocused {
                visible: bool,
            }
            impl App for DisabledFocused {
                fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
                    Response::ignored()
                }
                fn draw(&self, ui: &mut Ui<'_>) {
                    let state = if self.visible {
                        ReferenceState::FOCUSED | ReferenceState::FOCUS_VISIBLE
                    } else {
                        ReferenceState::FOCUSED
                    };
                    ui.reference(Some(ReferenceTarget::new(BUTTON, state)), |ui| {
                        Button::new(BUTTON, "Action")
                            .disabled(true)
                            .draw(ui, Rect::new(0, 0, 12, 1))
                    });
                }
            }
            let disabled: Harness<DisabledFocused> =
                Harness::new(DisabledFocused { visible: false }, theme.clone(), 12, 1);
            assert_eq!(
                disabled.cell(0, 0).symbol(),
                " ",
                "{level:?} {variant:?}: a disabled control without focus visibility is blank"
            );
            let disabled_visible: Harness<DisabledFocused> =
                Harness::new(DisabledFocused { visible: true }, theme.clone(), 12, 1);
            assert_eq!(
                disabled_visible.cell(0, 0).symbol(),
                " ",
                "{level:?} {variant:?}: the widget blanks the bar while disabled even when visible"
            );

            // Disabled stays legible without colour: DIM only at mono.
            let dim = disabled
                .cell(1, 0)
                .modifier
                .contains(ratatui_core::style::Modifier::DIM);
            assert_eq!(
                dim,
                level == ColorLevel::Mono,
                "{level:?} {variant:?}: DIM carries the disabled state at mono only"
            );
        }
    }
}

#[test]
fn disabled_buttons_reject_keyboard_and_pointer_changes() {
    let mut h = Harness::new(page(Variant::DEFAULT, true), Theme::junie(), 12, 1);
    let _ = h.click(3, 0);
    let _ = h.key(termrock_core::keys::KeyCode::Enter);
    assert_eq!(h.app().activations, 0, "a disabled button never activates");

    let mut enabled = Harness::new(page(Variant::DEFAULT, false), Theme::junie(), 12, 1);
    let _ = enabled.click(3, 0);
    assert_eq!(
        enabled.app().activations,
        1,
        "the same click activates an enabled button"
    );
}
