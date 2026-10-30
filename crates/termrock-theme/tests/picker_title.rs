use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;
use termrock_core::id::Id;
use termrock_core::response::StateFlags;
use termrock_overlays::{Item, Picker, PickerState};
use termrock_runtime::runtime::{Runtime, stub::Stub};
use termrock_theme::{Family, Role, Surface, Theme, Variant, resolve};

#[test]
fn picker_widget_title_uses_primary_bold_style_for_every_title_cell() {
    const ID: Id = Id::root("theme.tests.picker-title-baseline");
    let area = Rect::new(0, 0, 80, 24);
    let theme = Theme::junie();
    let expected_fg = resolve::bind_role(
        &theme,
        Role::Fg(termrock_theme::FgStep::Primary),
        Surface::Overlay,
    )
    .expect("primary theme foreground");
    let expected_bg = theme.bg(Surface::Overlay);
    let mut runtime = Runtime::new(Stub::default(), theme);
    let mut buffer = Buffer::empty(area);
    runtime
        .draw_scene(area, &mut buffer, |ui, _area| {
            Picker::<Item<'_>>::new(ID).title("Launch Target").draw(
                ui,
                Rect::new(12, 8, 52, 9),
                &PickerState::default(),
                &[],
            );
        })
        .commit_presented();

    let title: Vec<_> = "Launch Target".chars().collect();
    let start = (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| Position::new(x, y)))
        .find(|position| {
            title.iter().enumerate().all(|(offset, ch)| {
                let x = position
                    .x
                    .saturating_add(u16::try_from(offset).unwrap_or_default());
                buffer
                    .cell(Position::new(x, position.y))
                    .is_some_and(|cell| cell.symbol().starts_with(*ch))
            })
        })
        .expect("picker title must render in the complete widget frame");
    for (offset, _) in title.iter().enumerate() {
        let x = start
            .x
            .saturating_add(u16::try_from(offset).unwrap_or_default());
        let cell = buffer
            .cell(Position::new(x, start.y))
            .expect("every title character must occupy a frame cell");
        assert_eq!(cell.fg, expected_fg);
        assert_eq!(cell.bg, expected_bg);
        assert!(cell.modifier.contains(Modifier::BOLD));
    }
    assert_eq!(
        runtime.cursor(),
        None,
        "rendering a picker title must not move the cursor"
    );
}

#[test]
fn picker_title_theme_override_overlay_and_local_patch_remain_overridable() {
    use termrock_theme::{Overlay, OverlayRule, Part, StylePatch};

    const VARIANT: Variant = Variant::custom("theme.tests.picker-title");
    const OVERLAY_RULES: [OverlayRule; 1] = [(
        Family::PICKER,
        VARIANT,
        Part::TITLE,
        StateFlags::HOVERED,
        StylePatch::new().set_fg(Role::Warning),
    )];
    const LOCAL: StylePatch = StylePatch::new()
        .set_bg(Role::Surface(Surface::Field))
        .remove(Modifier::BOLD);

    let theme = Theme::junie().define_variant(Family::PICKER, VARIANT, |recipe| {
        recipe.default_variant(VARIANT);
        recipe
            .part(Part::TITLE)
            .when(StateFlags::HOVERED, StylePatch::new().add(Modifier::ITALIC));
    });
    let resolved = resolve::resolve_uncached(
        &theme,
        Family::PICKER,
        Variant::DEFAULT,
        Part::TITLE,
        StateFlags::HOVERED,
        Surface::Canvas,
        &[Overlay::new(&OVERLAY_RULES)],
        Some(&LOCAL),
    );
    assert_eq!(resolved.style.fg, Some(theme.color.warning));
    assert_eq!(resolved.style.bg, Some(theme.bg(Surface::Field)));
    assert!(resolved.style.add_modifier.contains(Modifier::ITALIC));
    assert!(!resolved.style.add_modifier.contains(Modifier::BOLD));
}
