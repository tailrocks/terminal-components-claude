use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::{focus::FocusRing, hit::HitRegistry, id::WidgetId};
use junie_tui::theme::{ButtonKind, ColorLevel, Theme};
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::menu::{ContextMenu, MenuItem};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Modifier;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
};

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

#[test]
fn button_focus_remains_unambiguous_without_colour() {
    let id = WidgetId::of("test.button");
    for level in LEVELS {
        let theme = Theme::for_level(level);
        for kind in [
            ButtonKind::Primary,
            ButtonKind::Secondary,
            ButtonKind::Subtle,
            ButtonKind::Danger,
            ButtonKind::Toggle,
        ] {
            // Render onto the same buffer: losing focus must erase its old bar.
            let area = Rect::new(0, 0, 12, 1);
            let mut buf = Buffer::empty(area);
            for (focused, disabled, expected) in
                [(true, false, "▎"), (false, false, " "), (true, true, " ")]
            {
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let mut ctx = RenderCtx::new(
                    &theme,
                    Interaction {
                        focus: focused.then_some(id),
                        hover: Some(id),
                        ..Default::default()
                    },
                    &mut hits,
                    &mut ring,
                );
                Button::new(id, "Action", kind).disabled(disabled).render(
                    area,
                    &mut buf,
                    &mut ctx,
                    theme.canvas,
                );
                // NO_COLOR removes colours in the backend; symbols must carry
                // focus without relying on foreground/background equality.
                assert_eq!(
                    buf[(0, 0)].symbol(),
                    expected,
                    "{level:?} {kind:?}: focused={focused}, disabled={disabled}"
                );
                assert_eq!(
                    buf[(1, 0)].modifier.contains(Modifier::DIM),
                    disabled && level == ColorLevel::Mono
                );
            }
        }
    }
}

#[test]
fn list_focus_and_selection_have_distinct_symbols_without_colour() {
    let id = WidgetId::of("test.list");
    for level in LEVELS {
        let theme = Theme::for_level(level);
        let mut list = ListBox::new(
            id,
            vec![ListItem::new("First"), ListItem::new("Second")],
            SelectMode::Multi,
        );
        list.checked[1] = true;
        let area = Rect::new(0, 0, 16, 2);
        let mut buf = Buffer::empty(area);
        for focus in [Some(id), None] {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(
                &theme,
                Interaction {
                    focus,
                    ..Default::default()
                },
                &mut hits,
                &mut ring,
            );
            list.render(area, &mut buf, &mut ctx, theme.canvas);
            assert_eq!(
                buf[(0, 0)].symbol(),
                if focus.is_some() { "▎" } else { " " }
            );
            assert_eq!(buf[(0, 1)].symbol(), " ");
            assert_eq!(buf[(1, 1)].symbol(), "✓");
        }
    }
}

#[test]
fn text_selection_uses_reverse_video_only_in_monochrome() {
    let id = WidgetId::of("test.selection");
    for level in LEVELS {
        let theme = Theme::for_level(level);
        let mut input = TextInput::new(id, "").value("select");
        input.editing = true;
        input.buffer.select_all();
        let area = Rect::new(0, 0, 16, 3);
        let mut buf = Buffer::empty(area);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &theme,
            Interaction {
                focus: Some(id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        input.render(area, &mut buf, &mut ctx, theme.canvas);
        for x in 2..8 {
            assert_eq!(
                buf[(x, 1)].modifier.contains(Modifier::REVERSED),
                level == ColorLevel::Mono
            );
        }
        assert!(!buf[(8, 1)].modifier.contains(Modifier::REVERSED));
    }
}

#[test]
fn disabled_fields_and_menu_commands_stay_distinct_and_inert() {
    let id = WidgetId::of("test.disabled");
    let enter = Key {
        code: KeyCode::Enter,
        mods: KeyModifiers::NONE,
    };
    for level in LEVELS {
        let theme = Theme::for_level(level);
        let area = Rect::new(0, 0, 24, 6);
        let mut buf = Buffer::empty(area);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut input = TextInput::new(id, "Name").value("locked").disabled(true);
        input.render(area, &mut buf, &mut ctx, theme.canvas);
        assert_eq!(
            buf[(2, 1)].modifier.contains(Modifier::DIM),
            level == ColorLevel::Mono
        );
        assert_eq!(input.on_key(&enter).0, Outcome::Ignored);
        input.on_click(Position::new(2, 1), true);
        input.on_paste("replacement");
        assert_eq!(input.text(), "locked");
        assert!(!input.editing);

        let mut button = Button::toggle(id, "Locked", false).disabled(true);
        assert!(!button.on_key(&enter).1);
        assert!(!button.on_click());
        assert_eq!(button.on, Some(false));

        let mut menu = ContextMenu::new(id, vec![MenuItem::new("Locked").disabled(true)]);
        menu.render(area, &mut buf, &mut ctx);
        assert_eq!(
            buf[(menu.area.x + 3, menu.area.y + 1)]
                .modifier
                .contains(Modifier::DIM),
            level == ColorLevel::Mono
        );
        assert!(menu.on_key(&enter).1.is_none());
        assert!(menu.on_click(menu.row_id(0)).is_none());
    }
}
