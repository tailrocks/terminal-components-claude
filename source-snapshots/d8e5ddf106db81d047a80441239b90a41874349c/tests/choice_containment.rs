use junie_tui::core::event::Key;
use junie_tui::core::{focus::FocusRing, hit::HitRegistry, id::WidgetId};
use junie_tui::theme::{ColorLevel, Theme};
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::choice::{Checkbox, RadioGroup, Toggle};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
};

#[test]
fn form_choices_keep_cells_and_hit_regions_inside_their_area() {
    let id = WidgetId::of("test.choice");
    let outer = Rect::new(2, 3, 24, 8);
    for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
        let theme = Theme::for_level(level);
        for width in 0..=12 {
            for disabled in [false, true] {
                for kind in 0..3 {
                    let area = Rect::new(5, 5, width, if kind == 1 { 3 } else { 1 });
                    let mut buf = Buffer::empty(outer);
                    for cell in &mut buf.content {
                        cell.set_symbol("#");
                    }
                    let before = buf.clone();
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
                    match kind {
                        0 => {
                            let mut widget = Checkbox::new(id, "Long 界e\u{301} label", true);
                            widget.disabled = disabled;
                            widget.render(area, &mut buf, &mut ctx, theme.canvas);
                        }
                        1 => {
                            let mut widget = RadioGroup::new(
                                id,
                                "Long 界e\u{301} label",
                                &["Long 界e\u{301} option", "Second option"],
                                0,
                            );
                            widget.disabled = disabled;
                            widget.render(area, &mut buf, &mut ctx, theme.canvas);
                        }
                        _ => Toggle::new(id, "Long 界e\u{301} label", true)
                            .disabled(disabled)
                            .render(area, &mut buf, &mut ctx, theme.canvas),
                    }
                    for y in outer.y..outer.bottom() {
                        for x in outer.x..outer.right() {
                            let pos = Position::new(x, y);
                            if !area.contains(pos) {
                                assert_eq!(
                                    buf[pos], before[pos],
                                    "kind={kind}, width={width}, disabled={disabled}, position={pos:?}"
                                );
                                assert!(hits.hit(pos).is_none());
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn radio_group_clears_old_option_areas_when_hidden() {
    let theme = Theme::junie();
    let id = WidgetId::of("test.radio");
    let mut radio = RadioGroup::new(id, "Choice", &["First", "Second"], 0);
    let mut buf = Buffer::empty(Rect::new(0, 0, 20, 4));
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
    radio.render(*buf.area(), &mut buf, &mut ctx, theme.canvas);
    assert_eq!(radio.areas.len(), 2);
    radio.render(Rect::ZERO, &mut buf, &mut ctx, theme.canvas);
    assert!(radio.areas.is_empty());
}

#[test]
fn radio_height_saturates_terminal_coordinates() {
    let mut radio = RadioGroup::new(WidgetId::of("test.radio.height"), "Choice", &[], 0);
    radio.options.resize(u16::MAX as usize, String::new());
    assert_eq!(radio.height(), u16::MAX);
}

#[test]
fn toggle_handles_labels_larger_than_terminal_coordinates() {
    let theme = Theme::junie();
    let area = Rect::new(5, 2, 12, 1);
    let mut buf = Buffer::empty(area);
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
    Toggle::new(
        WidgetId::of("test.toggle"),
        &"x".repeat(u16::MAX as usize),
        true,
    )
    .render(area, &mut buf, &mut ctx, theme.canvas);
}

#[test]
fn compact_choice_marks_keep_checked_state_without_colour() {
    let theme = Theme::for_level(ColorLevel::Mono);
    let id = WidgetId::of("test.compact");
    for width in [2, 3, 4] {
        for checked in [false, true] {
            let area = Rect::new(5, 2, width, 2);
            let mut buf = Buffer::empty(area);
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            Checkbox::new(id, "Choice", checked).render(area, &mut buf, &mut ctx, theme.canvas);
            let marker = buf[(6, 2)].symbol();
            assert_eq!(
                marker,
                if width >= 4 {
                    "["
                } else if checked {
                    "✓"
                } else {
                    "□"
                }
            );
            Toggle::new(id, "Choice", checked).render(area, &mut buf, &mut ctx, theme.canvas);
            if width < 4 {
                assert_eq!(buf[(6, 2)].symbol(), if checked { "●" } else { "○" });
            }
            RadioGroup::new(id, "Choice", &["First", "Second"], usize::from(!checked)).render(
                area,
                &mut buf,
                &mut ctx,
                theme.canvas,
            );
            if width < 4 {
                assert_eq!(buf[(6, 3)].symbol(), if checked { "●" } else { "○" });
            }
        }
    }
}

#[test]
fn disabled_choices_reject_keyboard_and_mouse_changes() {
    let enter = Key {
        code: KeyCode::Enter,
        mods: KeyModifiers::NONE,
    };
    let id = WidgetId::of("test.disabled.choice");
    let mut checkbox = Checkbox::new(id, "Choice", false);
    checkbox.disabled = true;
    checkbox.on_key(&enter);
    checkbox.on_click();
    assert!(!checkbox.checked);

    let mut toggle = Toggle::new(id, "Choice", false).disabled(true);
    toggle.on_key(&enter);
    toggle.on_click();
    assert!(!toggle.on);

    let mut radio = RadioGroup::new(id, "Choice", &["First", "Second"], 0);
    radio.disabled = true;
    radio.on_key(&Key {
        code: KeyCode::Down,
        mods: KeyModifiers::NONE,
    });
    radio.on_click(1);
    assert_eq!((radio.cursor, radio.selected), (0, 0));
}
