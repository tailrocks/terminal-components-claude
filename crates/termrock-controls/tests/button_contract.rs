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
use termrock_runtime::{App, Cx, Focusability, Status, Ui};
use termrock_test_support::Harness;
use termrock_theme::{
    ColorLevel, Family, GlyphRole, Role, Slot, StateFlags, StylePatch, Theme, Variant,
};

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
        Theme::junie().design.glyphs.get(GlyphRole::DotOn)
    );
    assert_eq!(
        off.cell(1, 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::DotOff)
    );

    let ascii = Theme::junie().builder().ascii_glyphs().build();
    let ascii_on = Harness::new(
        PropsPage {
            checked: true,
            ..PropsPage::default()
        },
        ascii.clone(),
        20,
        1,
    );
    let ascii_off = Harness::new(PropsPage::default(), ascii, 20, 1);
    assert_eq!(ascii_on.cell(1, 0).symbol(), "*");
    assert_eq!(ascii_off.cell(1, 0).symbol(), "o");

    let custom_dots = Theme::junie()
        .builder()
        .glyph(GlyphRole::DotOn, "+")
        .glyph(GlyphRole::DotOff, "-")
        .build();
    let custom_on = Harness::new(
        PropsPage {
            checked: true,
            ..PropsPage::default()
        },
        custom_dots.clone(),
        20,
        1,
    );
    let custom_off = Harness::new(PropsPage::default(), custom_dots, 20, 1);
    assert_eq!(custom_on.cell(1, 0).symbol(), "+");
    assert_eq!(custom_off.cell(1, 0).symbol(), "-");

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
fn checked_button_marker_keeps_its_semantic_glyph_in_every_color_profile() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let theme = Theme::junie().for_level(level);
        let checked = Harness::new(
            PropsPage {
                checked: true,
                ..PropsPage::default()
            },
            theme.clone(),
            20,
            1,
        );
        assert_eq!(
            checked.cell(1, 0).symbol(),
            theme.design.glyphs.get(GlyphRole::DotOn),
            "checked button glyph changed at {level:?}"
        );

        let unchecked = Harness::new(PropsPage::default(), theme.clone(), 20, 1);
        assert_eq!(
            unchecked.cell(1, 0).symbol(),
            theme.design.glyphs.get(GlyphRole::DotOff),
            "unchecked button glyph changed at {level:?}"
        );
    }

    let custom_dots = Theme::junie()
        .builder()
        .glyph(GlyphRole::DotOn, "x")
        .glyph(GlyphRole::DotOff, "o")
        .build()
        .for_level(ColorLevel::Mono);
    let custom = Harness::new(
        PropsPage {
            checked: true,
            ..PropsPage::default()
        },
        custom_dots,
        20,
        1,
    );
    assert_eq!(custom.cell(1, 0).symbol(), "x");

    let authored_marker = Theme::junie()
        .override_family(Family::BUTTON, |edit| {
            edit.part(Part::MARKER).when(
                StateFlags::CHECKED,
                StylePatch::new().set_glyph(GlyphRole::WarningMark),
            );
        })
        .for_level(ColorLevel::Mono);
    let authored = Harness::new(
        PropsPage {
            checked: true,
            ..PropsPage::default()
        },
        authored_marker.clone(),
        20,
        1,
    );
    assert_eq!(
        authored.cell(1, 0).symbol(),
        authored_marker.design.glyphs.get(GlyphRole::WarningMark)
    );

    struct MarkerPatchPage {
        patch: StylePatch,
    }
    impl App for MarkerPatchPage {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            Button::new(BUTTON, "Go")
                .checked(true)
                .patch_part(&[(Part::MARKER, self.patch)])
                .draw(ui, AREA);
        }
    }

    let instance_set = Harness::new(
        MarkerPatchPage {
            patch: StylePatch::new().set_glyph(GlyphRole::WarningMark),
        },
        Theme::junie().for_level(ColorLevel::Mono),
        20,
        1,
    );
    assert_eq!(
        instance_set.cell(1, 0).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::WarningMark)
    );

    let instance_clear = Harness::new(
        MarkerPatchPage {
            patch: StylePatch {
                glyph: Slot::Clear,
                ..StylePatch::new()
            },
        },
        Theme::junie().for_level(ColorLevel::Mono),
        20,
        1,
    );
    assert_eq!(instance_clear.cell(1, 0).symbol(), " ");
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
    variant: Variant,
    activations: usize,
}

impl FocusPage {
    fn button(&self) -> Button<'static> {
        Button::new(BUTTON, "Go")
            .variant(self.variant)
            .focus_visible(self.policy)
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
        edit.part(Part::LABEL).when(
            StateFlags::FOCUSED,
            StylePatch::new().add(ratatui_core::style::Modifier::BOLD),
        );
    })
}

#[test]
fn primary_button_pressed_background_matches_frozen_palette_at_each_color_level() {
    let expected = [
        (ColorLevel::TrueColor, Color::from_u32(0x002b_8632)),
        (ColorLevel::Ansi256, Color::Indexed(65)),
        (ColorLevel::Ansi16, Color::LightGreen),
        (ColorLevel::Mono, Color::White),
    ];

    for (level, background) in expected {
        let mut h = Harness::new(
            FocusPage {
                policy: ButtonFocusVisibility::Always,
                variant: Variant::PRIMARY,
                activations: 0,
            },
            Theme::junie().for_level(level),
            30,
            1,
        );
        let _ = h.mouse(termrock_core::event::MouseKind::Down, 12, 0);

        assert!(
            h.state_of(BUTTON).contains(StateFlags::PRESSED),
            "the real button must be pressed at {level:?}"
        );
        assert!(
            h.state_of(BUTTON)
                .contains(StateFlags::FOCUSED | StateFlags::HOVERED),
            "pointer press must retain focus and hover state at {level:?}"
        );
        assert_eq!(
            h.cell(10, 0).bg,
            background,
            "the real primary button pressed background at {level:?}"
        );
    }
}

#[test]
fn primary_pressed_profile_keeps_global_and_instance_button_overrides() {
    let level = ColorLevel::TrueColor;
    let global_theme =
        Theme::junie().override_variant(Family::BUTTON, Variant::PRIMARY, |recipe| {
            recipe
                .part(Part::CONTAINER)
                .when(StateFlags::PRESSED, StylePatch::new().set_bg(Role::Warning));
        });
    let mut global = Harness::new(
        FocusPage {
            policy: ButtonFocusVisibility::Always,
            variant: Variant::PRIMARY,
            activations: 0,
        },
        global_theme.clone().for_level(level),
        30,
        1,
    );
    let _ = global.mouse(termrock_core::event::MouseKind::Down, 12, 0);
    assert_eq!(global.cell(10, 0).bg, global_theme.color.warning);

    struct InstancePatchPage;
    impl App for InstancePatchPage {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let patch = [(Part::CONTAINER, StylePatch::new().set_bg(Role::Info))];
            Button::new(BUTTON, "Go")
                .variant(Variant::PRIMARY)
                .patch_part(&patch)
                .draw(ui, ui.full());
        }
    }

    let mut instance = Harness::new(InstancePatchPage, Theme::junie().for_level(level), 30, 1);
    let _ = instance.mouse(termrock_core::event::MouseKind::Down, 2, 0);
    assert_eq!(instance.cell(0, 0).bg, Theme::junie().color.info);

    let custom_color = Color::Rgb(12, 34, 56);
    let mut custom_theme = Theme::junie();
    custom_theme.color.accent_pressed = custom_color;
    let mut token_override = Harness::new(
        FocusPage {
            policy: ButtonFocusVisibility::Always,
            variant: Variant::PRIMARY,
            activations: 0,
        },
        custom_theme,
        30,
        1,
    );
    let _ = token_override.mouse(termrock_core::event::MouseKind::Down, 12, 0);
    assert_eq!(token_override.cell(10, 0).bg, custom_color);

    let paper = Theme::paper();
    let expected_paper = paper
        .resolve(
            Family::BUTTON,
            Variant::PRIMARY,
            Part::CONTAINER,
            StateFlags::PRESSED | StateFlags::HOVERED | StateFlags::FOCUSED,
            termrock_theme::Surface::Canvas,
        )
        .style
        .bg;
    let mut paper_button = Harness::new(
        FocusPage {
            policy: ButtonFocusVisibility::Always,
            variant: Variant::PRIMARY,
            activations: 0,
        },
        paper,
        30,
        1,
    );
    let _ = paper_button.mouse(termrock_core::event::MouseKind::Down, 12, 0);
    assert_eq!(Some(paper_button.cell(10, 0).bg), expected_paper);
}

#[test]
fn pointer_focus_visibility_obeys_always_and_keyboard_only_policies() {
    let mut always = Harness::new(
        FocusPage {
            policy: ButtonFocusVisibility::Always,
            variant: Variant::DEFAULT,
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
            variant: Variant::DEFAULT,
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
    assert!(
        keyboard_only
            .resolved(BUTTON, Part::LABEL)
            .style
            .add_modifier
            .contains(ratatui_core::style::Modifier::BOLD)
    );
    let _ = keyboard_only.mouse(termrock_core::event::MouseKind::Up, 10, 0);
    assert_eq!(keyboard_only.app().activations, 1);
    // Keyboard visibility is presentation policy, not input focus ownership:
    // keyboard activation must work immediately without a Tab round trip.
    let _ = keyboard_only.key(termrock_core::keys::KeyCode::Enter);
    assert_eq!(keyboard_only.app().activations, 2);
    let _ = keyboard_only.key(termrock_core::keys::KeyCode::Char(' '));
    assert_eq!(keyboard_only.app().activations, 3);
    let _ = keyboard_only.key(termrock_core::keys::KeyCode::Tab);
    assert!(keyboard_only.focus_visible());
    assert_eq!(
        keyboard_only.resolved(OTHER, Part::GUTTER).glyph,
        termrock_theme::Slot::Set(GlyphRole::FocusBar)
    );
}

#[test]
fn focus_visibility_policies_hold_for_every_button_variant_and_color_level() {
    let variants = [
        Variant::DEFAULT,
        Variant::PRIMARY,
        Variant::SECONDARY,
        Variant::SUBTLE,
        Variant::DANGER,
        Variant::TOGGLE,
        Variant::QUIET,
        Variant::GHOST,
    ];
    let levels = [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ];
    for level in levels {
        for variant in variants {
            let mut always = Harness::new(
                FocusPage {
                    policy: ButtonFocusVisibility::Always,
                    variant,
                    activations: 0,
                },
                Theme::junie().downgrade(level),
                30,
                1,
            );
            let _ = always.mouse(termrock_core::event::MouseKind::Down, 10, 0);
            assert_eq!(always.focus(), Some(BUTTON), "{level:?}/{variant:?}");
            assert_eq!(
                always.resolved(BUTTON, Part::GUTTER).glyph,
                termrock_theme::Slot::Set(GlyphRole::FocusBar),
                "Always omitted pointer focus at {level:?}/{variant:?}"
            );

            let mut keyboard = Harness::new(
                FocusPage {
                    policy: ButtonFocusVisibility::KeyboardOnly,
                    variant,
                    activations: 0,
                },
                Theme::junie().downgrade(level),
                30,
                1,
            );
            let _ = keyboard.key(termrock_core::keys::KeyCode::Tab);
            assert_eq!(keyboard.focus(), Some(BUTTON), "{level:?}/{variant:?}");
            assert!(keyboard.focus_visible(), "{level:?}/{variant:?}");
            assert_eq!(
                keyboard.resolved(BUTTON, Part::GUTTER).glyph,
                termrock_theme::Slot::Set(GlyphRole::FocusBar),
                "keyboard focus omitted its indicator at {level:?}/{variant:?}"
            );

            let mut keyboard_only = Harness::new(
                FocusPage {
                    policy: ButtonFocusVisibility::KeyboardOnly,
                    variant,
                    activations: 0,
                },
                Theme::junie().downgrade(level),
                30,
                1,
            );
            let _ = keyboard_only.mouse(termrock_core::event::MouseKind::Down, 10, 0);
            let _ = keyboard_only.mouse(termrock_core::event::MouseKind::Up, 10, 0);
            assert_eq!(keyboard_only.focus(), Some(BUTTON), "{level:?}/{variant:?}");
            assert_ne!(
                keyboard_only.resolved(BUTTON, Part::GUTTER).glyph,
                termrock_theme::Slot::Set(GlyphRole::FocusBar),
                "KeyboardOnly showed pointer focus at {level:?}/{variant:?}"
            );
            let _ = keyboard_only.key(termrock_core::keys::KeyCode::Enter);
            assert_eq!(
                keyboard_only.app().activations,
                2,
                "actual focus/binding lost at {level:?}/{variant:?}"
            );
        }
    }
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
    assert!(
        h.resolved(BUTTON, Part::LABEL)
            .style
            .add_modifier
            .contains(ratatui_core::style::Modifier::BOLD)
    );
    assert!(!h.ring().is_registered(BUTTON));
}

#[test]
fn keyboard_enter_and_space_share_button_activation() {
    let mut h = Harness::new(
        FocusPage {
            policy: ButtonFocusVisibility::KeyboardOnly,
            variant: Variant::DEFAULT,
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

struct FittedButton {
    label: &'static str,
    checked: Option<bool>,
}

impl App for FittedButton {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let button = Button::new(BUTTON, self.label);
        match self.checked {
            Some(value) => button.checked(value).draw(ui, ui.full()),
            None => button.draw(ui, ui.full()),
        };
    }
}

#[test]
fn narrow_wide_label_uses_the_shared_cell_safe_ellipsis_in_all_profiles() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let theme = Theme::junie().downgrade(level);
        let h = Harness::new(
            FittedButton {
                label: "世界",
                checked: None,
            },
            theme.clone(),
            5,
            1,
        );

        assert_eq!(
            h.cell(0, 0).symbol(),
            theme.design.glyphs.get(GlyphRole::FocusBar),
            "leading focus gutter at {level:?}"
        );
        assert_eq!(h.cell(1, 0).symbol(), "世", "{level:?}");
        assert_eq!(
            h.cell(2, 0).symbol(),
            " ",
            "wide-grapheme continuation at {level:?}"
        );
        assert_eq!(h.cell(3, 0).symbol(), "…", "{level:?}");
        assert_eq!(h.cell(4, 0).symbol(), " ", "trailing padding at {level:?}");
        assert_eq!(h.area_of(BUTTON).expect("button is registered").width, 5);
    }
}

#[test]
fn narrow_labels_preserve_markers_empty_text_combining_graphemes_and_emoji() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let theme = Theme::junie().downgrade(level);

        let checked = Harness::new(
            FittedButton {
                label: "世界",
                checked: Some(true),
            },
            theme.clone(),
            6,
            1,
        );
        let marker = theme
            .resolve(
                Family::BUTTON,
                Variant::DEFAULT,
                Part::MARKER,
                StateFlags::CHECKED | StateFlags::SELECTED,
                termrock_theme::Surface::Canvas,
            )
            .glyph;
        let expected_marker = match marker {
            Slot::Set(role) => theme.design.glyphs.get(role),
            Slot::Inherit => theme.design.glyphs.get(GlyphRole::DotOn),
            Slot::Clear => " ",
        };
        assert_eq!(
            checked.cell(1, 0).symbol(),
            expected_marker,
            "checked marker at {level:?}"
        );
        assert_eq!(
            checked.cell(3, 0).symbol(),
            "…",
            "checked label at {level:?}"
        );

        let combining = Harness::new(
            FittedButton {
                label: "e\u{301}界",
                checked: None,
            },
            theme.clone(),
            4,
            1,
        );
        assert_eq!(combining.cell(1, 0).symbol(), "e\u{301}", "{level:?}");
        assert_eq!(combining.cell(2, 0).symbol(), "…", "{level:?}");

        let emoji = Harness::new(
            FittedButton {
                label: "👩‍💻x",
                checked: None,
            },
            theme.clone(),
            4,
            1,
        );
        assert_eq!(
            emoji.cell(1, 0).symbol(),
            "…",
            "emoji grapheme at {level:?}"
        );

        let empty = Harness::new(
            FittedButton {
                label: "",
                checked: None,
            },
            theme.clone(),
            2,
            1,
        );
        assert_eq!(
            empty
                .area_of(BUTTON)
                .expect("empty button is registered")
                .width,
            2
        );

        for width in [0, 1, 2, 3] {
            let narrow = Harness::new(UnicodeButton, theme.clone(), width, 1);
            let _ = narrow.snapshot();
            if width == 0 {
                assert_eq!(narrow.area_of(BUTTON), None, "zero width at {level:?}");
            } else {
                assert_eq!(
                    narrow.area_of(BUTTON).expect("visible narrow button").width,
                    width,
                    "{level:?} width {width}"
                );
            }
        }
    }
}

fn custom_label_slot(ui: &mut Ui<'_>, area: Rect) {
    ui.paint_str(area, "custom", ui.surface_style());
}

struct SlottedWideButton;

impl App for SlottedWideButton {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(BUTTON, "世界")
            .slot(Part::LABEL, &custom_label_slot)
            .draw(ui, ui.full());
    }
}

#[test]
fn narrow_custom_label_slot_keeps_ownership_of_its_content() {
    let h = Harness::new(SlottedWideButton, Theme::junie(), 5, 1);
    assert_eq!(h.cell(1, 0).symbol(), "c");
    assert_eq!(h.cell(2, 0).symbol(), "u");
    assert_eq!(h.cell(3, 0).symbol(), "s");
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

struct ClickOnlyPage {
    focusability: Focusability,
    activations: usize,
}

impl ClickOnlyPage {
    fn button(&self) -> Button<'static> {
        Button::new(CLICK_ONLY_BUTTON, "Close").focusability(self.focusability)
    }
}

impl App for ClickOnlyPage {
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

const CLICK_ONLY_BUTTON: Id = Id::root("button.contract.click-only");

#[test]
fn click_only_button_loses_keyboard_focus_but_keeps_pointer_activation() {
    let mut h = Harness::new(
        ClickOnlyPage {
            focusability: Focusability::Focusable,
            activations: 0,
        },
        focus_theme(),
        20,
        1,
    );
    assert!(h.ring().is_registered(CLICK_ONLY_BUTTON));
    let _ = h.key(termrock_core::keys::KeyCode::Tab);
    assert_eq!(h.focus(), Some(CLICK_ONLY_BUTTON));

    h.app_mut().focusability = Focusability::ClickOnly;
    let _ = h.tick();
    assert!(!h.ring().is_registered(CLICK_ONLY_BUTTON));
    assert_ne!(h.focus(), Some(CLICK_ONLY_BUTTON));
    assert_ne!(
        h.resolved(CLICK_ONLY_BUTTON, Part::GUTTER).glyph,
        termrock_theme::Slot::Set(GlyphRole::FocusBar),
        "a ClickOnly button never paints the keyboard focus marker"
    );

    let _ = h.key(termrock_core::keys::KeyCode::Enter);
    let _ = h.key(termrock_core::keys::KeyCode::Char(' '));
    assert_eq!(
        h.app().activations,
        0,
        "ClickOnly disables keyboard activation"
    );

    let _ = h.mouse(termrock_core::event::MouseKind::Down, 2, 0);
    assert_ne!(h.focus(), Some(CLICK_ONLY_BUTTON));
    assert_eq!(
        h.app().activations,
        0,
        "press does not activate the control"
    );
    let _ = h.mouse(termrock_core::event::MouseKind::Up, 2, 0);
    assert_eq!(
        h.app().activations,
        1,
        "the rendered control remains clickable"
    );
}

struct DisabledPolicyPage {
    target: Focusability,
    focusability: Focusability,
    autofocus: bool,
    activations: usize,
}

impl DisabledPolicyPage {
    fn button(&self) -> Button<'static> {
        let button = Button::new(DISABLED_POLICY_BUTTON, "Go").focusability(self.focusability);
        if self.autofocus {
            button.autofocus()
        } else {
            button
        }
    }
}

impl App for DisabledPolicyPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.intents(DISABLED_POLICY_BUTTON).any(|intent| {
            matches!(
                intent,
                termrock_core::Intent::Binding(_)
                    | termrock_core::Intent::Pointer {
                        phase: termrock_core::Phase::Click,
                        ..
                    }
            )
        }) {
            self.focusability = self.target;
        }
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

const DISABLED_POLICY_BUTTON: Id = Id::root("button.contract.disabled-policy");

#[test]
fn disabled_focusability_rejects_already_routed_keyboard_activation() {
    let mut h = Harness::new(
        DisabledPolicyPage {
            target: Focusability::Disabled,
            focusability: Focusability::Focusable,
            autofocus: false,
            activations: 0,
        },
        focus_theme(),
        20,
        1,
    );
    assert_eq!(h.focus(), Some(DISABLED_POLICY_BUTTON));

    let _ = h.key(termrock_core::keys::KeyCode::Enter);
    assert_eq!(
        h.app().activations,
        0,
        "Disabled focusability must reject an Enter already routed to this button"
    );
    assert!(!h.ring().contains(DISABLED_POLICY_BUTTON));
    assert!(
        h.ring()
            .entry(DISABLED_POLICY_BUTTON)
            .is_some_and(|entry| entry.disabled)
    );
    assert_ne!(
        h.resolved(DISABLED_POLICY_BUTTON, Part::GUTTER).glyph,
        termrock_theme::Slot::Set(GlyphRole::FocusBar),
        "Disabled focusability must suppress stale focus styling"
    );
}

#[test]
fn disabled_focusability_rejects_already_routed_pointer_activation() {
    let mut h = Harness::new(
        DisabledPolicyPage {
            target: Focusability::Disabled,
            focusability: Focusability::Focusable,
            autofocus: false,
            activations: 0,
        },
        focus_theme(),
        20,
        1,
    );
    let _ = h.mouse(termrock_core::event::MouseKind::Down, 2, 0);
    let _ = h.mouse(termrock_core::event::MouseKind::Up, 2, 0);
    assert_eq!(
        h.app().activations,
        0,
        "Disabled focusability must reject a click already routed to this button"
    );
    assert!(!h.ring().contains(DISABLED_POLICY_BUTTON));
}

#[test]
fn click_only_focusability_rejects_routed_key_but_accepts_routed_pointer_click() {
    let mut h = Harness::new(
        DisabledPolicyPage {
            target: Focusability::ClickOnly,
            focusability: Focusability::Focusable,
            autofocus: false,
            activations: 0,
        },
        focus_theme(),
        20,
        1,
    );
    assert_eq!(h.focus(), Some(DISABLED_POLICY_BUTTON));
    let _ = h.key(termrock_core::keys::KeyCode::Enter);
    assert_eq!(h.app().activations, 0);

    let _ = h.mouse(termrock_core::event::MouseKind::Down, 2, 0);
    let _ = h.mouse(termrock_core::event::MouseKind::Up, 2, 0);
    assert_eq!(h.app().activations, 1);
    assert_ne!(h.focus(), Some(DISABLED_POLICY_BUTTON));
}

#[test]
fn disabled_focusability_never_takes_autofocus() {
    let h = Harness::new(
        DisabledPolicyPage {
            target: Focusability::Disabled,
            focusability: Focusability::Disabled,
            autofocus: true,
            activations: 0,
        },
        focus_theme(),
        20,
        1,
    );
    assert_ne!(h.focus(), Some(DISABLED_POLICY_BUTTON));
    assert!(!h.ring().contains(DISABLED_POLICY_BUTTON));
}
