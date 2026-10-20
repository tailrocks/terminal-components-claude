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
use ratatui_core::style::Color;
use termrock_controls::{Button, ButtonFocusVisibility, ReferenceState, ReferenceTarget};
use termrock_core::id::{Id, Part};
use termrock_core::response::Response;
use termrock_runtime::{App, Cx, Status, Ui};
use termrock_test_support::Harness;
use termrock_theme::{Family, GlyphRole, Role, StateFlags, StylePatch, Theme};

const BUTTON: Id = Id::root("button.contract.main");
const AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 20,
    height: 1,
};

#[derive(Default)]
struct PropsPage {
    checked: bool,
    busy: bool,
    disabled: bool,
    activations: usize,
}

impl PropsPage {
    fn button(&self) -> Button<'static> {
        let button = Button::new(BUTTON, "Go")
            .checked(self.checked)
            .disabled(self.disabled);
        if self.busy {
            button.status(Status::Busy)
        } else {
            button
        }
    }
}

impl App for PropsPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.button().update(cx).activated() {
            self.activations += 1;
            Response::changed()
        } else {
            Response::ignored()
        }
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.button().draw(ui, AREA);
    }
}

#[test]
fn checked_fallback_keeps_single_cell_button_marker_and_explicit_patch_wins() {
    let on = Harness::new(
        PropsPage {
            checked: true,
            ..PropsPage::default()
        },
        Theme::junie(),
        20,
        1,
    );
    let off = Harness::new(PropsPage::default(), Theme::junie(), 20, 1);
    assert_eq!(
        on.cell(1, 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::SwitchKnob)
    );
    assert_eq!(off.cell(1, 0).symbol(), " ");

    struct PatchedMarker;
    impl App for PatchedMarker {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let patch = [(
                Part::MARKER,
                StylePatch::new().set_glyph(GlyphRole::WarningMark),
            )];
            Button::new(BUTTON, "Go")
                .checked(true)
                .patch_part(&patch)
                .draw(ui, AREA);
        }
    }
    let patched = Harness::new(PatchedMarker, Theme::junie(), 20, 1);
    assert_eq!(
        patched.cell(1, 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::WarningMark)
    );

    struct ClearedMarker;
    impl App for ClearedMarker {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let patch = [(
                Part::MARKER,
                StylePatch {
                    glyph: termrock_theme::Slot::Clear,
                    ..StylePatch::new()
                },
            )];
            Button::new(BUTTON, "Go")
                .checked(true)
                .patch_part(&patch)
                .draw(ui, AREA);
        }
    }
    let cleared = Harness::new(ClearedMarker, Theme::junie(), 20, 1);
    assert_eq!(cleared.cell(1, 0).symbol(), " ");

    fn marker_slot(ui: &mut Ui<'_>, area: Rect) {
        ui.glyph(area, GlyphRole::WarningMark, ui.surface_style());
    }
    struct SlottedMarker;
    impl App for SlottedMarker {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            Button::new(BUTTON, "Go")
                .checked(true)
                .slot(Part::MARKER, &marker_slot)
                .draw(ui, AREA);
        }
    }
    let slotted = Harness::new(SlottedMarker, Theme::junie(), 20, 1);
    assert_eq!(
        slotted.cell(1, 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::WarningMark)
    );
}

#[test]
fn busy_prefix_replaces_checked_marker_and_matches_measured_width() {
    let page = PropsPage {
        checked: true,
        busy: true,
        ..PropsPage::default()
    };
    let h = Harness::new(page, Theme::junie(), 20, 1);
    let area = h.area_of(BUTTON).expect("button area is registered");
    assert_eq!(
        area.width, 6,
        "gutter + spinner lane + label + trailing cell"
    );
    assert_eq!(h.cell(3, 0).symbol(), "G");
    assert_eq!(h.cell(4, 0).symbol(), "o");
    assert_eq!(h.app().activations, 0);
    let mut disabled = Harness::new(
        PropsPage {
            disabled: true,
            ..PropsPage::default()
        },
        Theme::junie(),
        20,
        1,
    );
    let _ = disabled.click_id(BUTTON);
    assert_eq!(disabled.app().activations, 0);
    let mut busy = Harness::new(
        PropsPage {
            busy: true,
            ..PropsPage::default()
        },
        Theme::junie(),
        20,
        1,
    );
    let _ = busy.click_id(BUTTON);
    assert_eq!(busy.app().activations, 0);
}

#[derive(Default)]
struct InertButton {
    activations: usize,
}

impl App for InertButton {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let target = ReferenceTarget::new(
            BUTTON,
            ReferenceState::FOCUSED
                .union(ReferenceState::FOCUS_VISIBLE)
                .union(ReferenceState::HOVERED)
                .union(ReferenceState::PRESSED),
        );
        let parts = [
            (Part::CONTAINER, Color::from_u32(0x0011_2233)),
            (Part::GUTTER, Color::from_u32(0x0022_3344)),
            (Part::ICON, Color::from_u32(0x0033_4455)),
            (Part::MARKER, Color::from_u32(0x0044_5566)),
            (Part::LABEL, Color::from_u32(0x0055_6677)),
        ];
        let patches =
            parts.map(|(part, color)| (part, StylePatch::new().set_fg(Role::Custom(color))));
        ui.reference(Some(target), |ui| {
            Button::new(BUTTON, "Go")
                .status(Status::Error)
                .checked(true)
                .patch_part(&patches)
                .draw(ui, AREA);
        });
    }
}

#[test]
fn owner_wide_reference_styles_every_part_without_live_registration_or_cursor() {
    let h = Harness::new(InertButton::default(), Theme::junie(), 20, 1);
    let expected = [
        (2, Color::from_u32(0x0011_2233)),
        (0, Color::from_u32(0x0022_3344)),
        (1, Color::from_u32(0x0033_4455)),
        (3, Color::from_u32(0x0044_5566)),
        (5, Color::from_u32(0x0055_6677)),
    ];
    for (x, color) in expected {
        assert_eq!(h.cell(x, 0).fg, color, "part style at x={x}");
    }
    assert!(!h.ring().is_registered(BUTTON));
    assert!(h.area_of(BUTTON).is_none());
    assert_eq!(h.cursor(), None);
    assert_eq!(h.app().activations, 0);
}

const OTHER: Id = Id::root("button.contract.other");

struct FocusPage {
    policy: ButtonFocusVisibility,
    activations: usize,
}

impl FocusPage {
    fn button(&self) -> Button<'static> {
        Button::new(BUTTON, "Go").focus_visible(self.policy)
    }
}

impl App for FocusPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let _ = Button::new(OTHER, "Other").update(cx);
        if self.button().update(cx).activated() {
            self.activations += 1;
            Response::changed()
        } else {
            Response::ignored()
        }
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(OTHER, "Other").draw(ui, Rect { width: 8, ..AREA });
        self.button().draw(ui, Rect { x: 10, ..AREA });
    }
}

fn focus_theme() -> Theme {
    Theme::junie().override_family(Family::BUTTON, |edit| {
        let gutter = edit.part(Part::GUTTER);
        gutter.base(StylePatch::new().set_glyph(GlyphRole::RuleQuiet));
        gutter.when(
            StateFlags::FOCUS_VISIBLE,
            StylePatch::new().set_glyph(GlyphRole::FocusBar),
        );
    })
}

#[test]
fn pointer_focus_visibility_obeys_always_and_keyboard_only_policies() {
    let mut always = Harness::new(
        FocusPage {
            policy: ButtonFocusVisibility::Always,
            activations: 0,
        },
        focus_theme(),
        30,
        1,
    );
    assert_eq!(always.focus(), Some(OTHER));
    let _ = always.mouse(termrock_core::event::MouseKind::Down, 10, 0);
    assert_eq!(always.focus(), Some(BUTTON));
    assert!(!always.focus_visible());
    assert_eq!(
        always.resolved(BUTTON, Part::GUTTER).glyph,
        termrock_theme::Slot::Set(GlyphRole::FocusBar)
    );
    assert_eq!(always.app().activations, 0, "press does not activate");
    let _ = always.mouse(termrock_core::event::MouseKind::Up, 10, 0);
    assert_eq!(always.app().activations, 1, "release activates once");

    let mut keyboard_only = Harness::new(
        FocusPage {
            policy: ButtonFocusVisibility::KeyboardOnly,
            activations: 0,
        },
        focus_theme(),
        30,
        1,
    );
    let _ = keyboard_only.mouse(termrock_core::event::MouseKind::Down, 10, 0);
    assert_eq!(keyboard_only.focus(), Some(BUTTON));
    assert!(!keyboard_only.focus_visible());
    assert_eq!(
        keyboard_only.resolved(BUTTON, Part::GUTTER).glyph,
        termrock_theme::Slot::Set(GlyphRole::RuleQuiet)
    );
    let _ = keyboard_only.mouse(termrock_core::event::MouseKind::Up, 10, 0);
    assert_eq!(keyboard_only.app().activations, 1);
    let _ = keyboard_only.key(termrock_core::keys::KeyCode::Tab);
    assert!(keyboard_only.focus_visible());
    assert_eq!(
        keyboard_only.resolved(OTHER, Part::GUTTER).glyph,
        termrock_theme::Slot::Set(GlyphRole::FocusBar)
    );
}

struct InertFocusedButton;

impl App for InertFocusedButton {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.reference(
            Some(ReferenceTarget::new(BUTTON, ReferenceState::FOCUSED)),
            |ui| {
                Button::new(BUTTON, "Go")
                    .focus_visible(ButtonFocusVisibility::Always)
                    .draw(ui, AREA);
            },
        );
    }
}

#[test]
fn always_policy_does_not_promote_focus_in_an_inert_reference_scope() {
    let h = Harness::new(InertFocusedButton, focus_theme(), 20, 1);
    assert_eq!(
        h.resolved(BUTTON, Part::GUTTER).glyph,
        termrock_theme::Slot::Set(GlyphRole::RuleQuiet)
    );
    assert!(!h.ring().is_registered(BUTTON));
}

#[test]
fn keyboard_enter_and_space_share_button_activation() {
    let mut h = Harness::new(
        FocusPage {
            policy: ButtonFocusVisibility::KeyboardOnly,
            activations: 0,
        },
        focus_theme(),
        30,
        1,
    );
    let _ = h.key(termrock_core::keys::KeyCode::Tab);
    assert_eq!(h.focus(), Some(BUTTON));
    let _ = h.key(termrock_core::keys::KeyCode::Enter);
    assert_eq!(h.app().activations, 1);
    let _ = h.key(termrock_core::keys::KeyCode::Char(' '));
    assert_eq!(h.app().activations, 2);
}

struct UnicodeButton;

impl App for UnicodeButton {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(BUTTON, "👩‍💻e\u{301}界").draw(ui, ui.full());
    }
}

#[test]
fn wide_and_combining_labels_clip_at_narrow_widths() {
    for width in [1, 2, 3] {
        let h = Harness::new(UnicodeButton, Theme::junie(), width, 1);
        let area = h.area_of(BUTTON).expect("button area is registered");
        assert_eq!(area.width, width);
        for x in 0..width {
            let _ = h.cell(x, 0).symbol();
        }
    }
}
