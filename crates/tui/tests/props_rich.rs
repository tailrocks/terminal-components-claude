//! Decorative borrowed property rows preserve geometry and customization.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "Source-bound public consumer assertions and retained full-cell capture"
)]
use junie_tui::{ColorLevel, Constraints, Props, Rect, Theme};
use junie_tui_testing::Scene;

#[test]
fn legacy_constructor_cells_and_measurement() {
    let rows = [
        ("Key", "value"),
        ("Wide界", "two words"),
        ("Third", "new\nline"),
    ];
    for (theme_index, theme) in [Theme::junie(), Theme::paper()].into_iter().enumerate() {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            for area in [
                Rect::new(0, 0, 20, 8),
                Rect::new(2, 1, 14, 5),
                Rect::new(1, 2, 3, 3),
                Rect::new(0, 0, 0, 4),
            ] {
                let mut scene = Scene::new("props_default", theme.clone(), level, 20, 8);
                let mut used = Rect::default();
                let mut measured = None;
                scene.draw(|ui, _| {
                    let props = Props::new(&rows);
                    measured = Some(props.measure(ui, Constraints::loose(area.width, area.height)));
                    used = props.draw(ui, area);
                });
                assert_eq!(
                    used.height,
                    if area.is_empty() {
                        area.height
                    } else {
                        area.height.min(3)
                    }
                );
                println!(
                    "LEGACY|{theme_index}|{level:?}|{area:?}|{used:?}|{measured:?}|{:?}",
                    scene.buffer().content
                );
            }
        }
    }
}

#[test]
fn wrapped_rows_share_measurement_and_following_row_positions() {
    use junie_tui::{ItemKey, PropsRow};
    let rows = [
        PropsRow::new(ItemKey::num(1), "A", "one two three").wrap(),
        PropsRow::new(ItemKey::num(2), "B", "tail"),
    ];
    let mut scene = Scene::new("props_wrap", Theme::junie(), ColorLevel::TrueColor, 8, 5);
    scene.draw(|ui, area| {
        let props = Props::rich(&rows);
        assert_eq!(
            props.measure(ui, Constraints::loose(8, 5)).preferred,
            (8, 4)
        );
        assert_eq!(props.draw(ui, area).height, 4);
    });
    for (y, text) in ["A  one  ", "   two  ", "   three", "B  tail "]
        .into_iter()
        .enumerate()
    {
        let actual: String = (0..8)
            .map(|x| scene.buffer().cell((x, y as u16)).unwrap().symbol())
            .collect();
        assert_eq!(actual, text);
    }
}

#[test]
fn pinned_manager_mounts_keep_newlines_and_accounts_wrap() {
    use junie_tui::{ItemKey, PropsRow};
    // 794b095 screens/manager.rs1082–1170 uses newline mounts and wrapped accounts.
    let rows = [
        PropsRow::new(
            ItemKey::num(1),
            "Mounts",
            "~/src/payments-platform · rw worktree\n~/src/shared-libs · ro shared",
        )
        .wrap(),
        PropsRow::new(
            ItemKey::num(2),
            "Accounts",
            "Claude · Personal ★ · Claude · Work · Codex · Primary ★",
        )
        .wrap(),
        PropsRow::new(ItemKey::num(3), "Policies", "done"),
    ];
    let mut scene = Scene::new("props_source", Theme::junie(), ColorLevel::TrueColor, 50, 8);
    scene.draw(|ui, area| {
        let props = Props::rich(&rows);
        assert_eq!(props.measure(ui, Constraints::loose(50, 8)).preferred.1, 5);
        assert_eq!(props.draw(ui, area).height, 5);
    });
    let line = |y| {
        (0..50)
            .map(|x| scene.buffer().cell((x, y)).unwrap().symbol())
            .collect::<String>()
    };
    assert_eq!(
        line(0).trim_end(),
        "Mounts    ~/src/payments-platform · rw worktree"
    );
    assert_eq!(
        line(1).trim_end(),
        "          ~/src/shared-libs · ro shared"
    );
    assert!(line(2).starts_with("Accounts  Claude · Personal"));
    assert!(line(3).contains("Primary ★"));
    assert_eq!(line(4).trim_end(), "Policies  done");
}

#[test]
fn wide_graphemes_and_empty_paragraphs_follow_the_shared_engine() {
    use junie_tui::{ItemKey, PropsRow};
    let rows = [
        PropsRow::new(ItemKey::num(1), "W", "界界 e\u{301}界").wrap(),
        PropsRow::new(ItemKey::num(2), "N", "x\n\ny").wrap(),
    ];
    let mut scene = Scene::new("props_wide", Theme::junie(), ColorLevel::TrueColor, 6, 8);
    scene.draw(|ui, area| {
        let props = Props::rich(&rows);
        assert_eq!(
            props.measure(ui, Constraints::loose(6, 8)).preferred,
            (6, 6)
        );
        assert_eq!(props.draw(ui, area).height, 6);
    });
    for y in [0, 1] {
        assert_eq!(scene.buffer().cell((3, y)).unwrap().symbol(), "界");
    }
    assert_eq!(scene.buffer().cell((3, 2)).unwrap().symbol(), "e\u{301}");
    assert_eq!(scene.buffer().cell((4, 2)).unwrap().symbol(), "界");
    assert_eq!(scene.buffer().cell((3, 3)).unwrap().symbol(), "x");
    assert_eq!(scene.buffer().cell((3, 4)).unwrap().symbol(), " ");
    assert_eq!(scene.buffer().cell((3, 5)).unwrap().symbol(), "y");
}

#[test]
fn clipping_preserves_logical_wrap_alignment_and_outside_sentinels() {
    use junie_tui::{ItemKey, PropsRow};
    use ratatui_core::style::{Color, Style};
    let rows = [PropsRow::new(ItemKey::num(1), "A", "one two three").wrap()];
    let clip = Rect::new(4, 1, 5, 2);
    let mut scene = Scene::new("props_clip", Theme::junie(), ColorLevel::TrueColor, 12, 5);
    scene.draw_over(
        |buf| {
            for cell in &mut buf.content {
                cell.set_symbol(".")
                    .set_style(Style::new().fg(Color::Magenta).bg(Color::Cyan));
            }
        },
        |ui, _| {
            ui.with_area(clip, |ui| {
                assert_eq!(Props::rich(&rows).draw(ui, Rect::new(1, 0, 8, 5)).height, 3);
            });
        },
    );
    for y in 0..5 {
        for x in 0..12 {
            let cell = scene.buffer().cell((x, y)).unwrap();
            if !clip.contains((x, y).into()) {
                assert_eq!(
                    (cell.symbol(), cell.fg, cell.bg),
                    (".", Color::Magenta, Color::Cyan)
                );
            }
        }
    }
    let row = |y| {
        (4..9)
            .map(|x| scene.buffer().cell((x, y)).unwrap().symbol())
            .collect::<String>()
    };
    assert_eq!(row(1), "two..");
    assert_eq!(row(2), "three");
}

#[test]
fn semantic_tone_stays_below_theme_scope_and_part_overrides() {
    use junie_tui::{
        Family, ItemKey, Overlay, OverlayRule, Part, PropsRow, Role, StateFlags, StylePatch,
        Variant,
    };
    use ratatui_core::style::Color;
    const RULES: [OverlayRule; 1] = [(
        Family::PROPS,
        Variant::DEFAULT,
        Part::LABEL,
        StateFlags::empty(),
        StylePatch::new()
            .set_fg(Role::Custom(Color::Green))
            .set_bg(Role::Custom(Color::Blue)),
    )];
    let patch = [(
        Part::LABEL,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Yellow))
            .set_bg(Role::Custom(Color::Magenta)),
    )];
    for base in [Theme::junie(), Theme::paper()] {
        for (theme, scoped, local, expected) in [
            (base.clone(), false, false, (Color::Red, Color::Reset)),
            (
                base.clone().override_family(Family::PROPS, |r| {
                    r.part(Part::LABEL).base(
                        StylePatch::new()
                            .set_fg(Role::Custom(Color::Cyan))
                            .set_bg(Role::Custom(Color::Black)),
                    );
                }),
                false,
                false,
                (Color::Cyan, Color::Black),
            ),
            (base.clone(), true, false, (Color::Green, Color::Blue)),
            (base, true, true, (Color::Yellow, Color::Magenta)),
        ] {
            let rows = [PropsRow::new(ItemKey::num(1), "A", "one two")
                .tone(Role::Custom(Color::Red))
                .wrap()];
            let mut scene = Scene::new("props_tone", theme, ColorLevel::TrueColor, 6, 3);
            scene.draw(|ui, area| {
                let props = if local {
                    Props::rich(&rows).patch_part(&patch)
                } else {
                    Props::rich(&rows)
                };
                let draw = |ui: &mut junie_tui::Ui<'_>| {
                    props.draw(ui, area);
                };
                if scoped {
                    ui.with_overlay(&Overlay::new(&RULES), draw);
                } else {
                    draw(ui);
                }
            });
            for y in [0, 1] {
                let cell = scene.buffer().cell((3, y)).unwrap();
                assert_eq!((cell.fg, cell.bg), expected);
            }
        }
    }
}

#[test]
fn empty_and_narrow_rectangles_never_escape_their_area() {
    use junie_tui::{ItemKey, PropsRow};
    let rows = [PropsRow::new(ItemKey::num(1), "Long", "界 x\nvalue").wrap()];
    for w in 0..9 {
        for h in 0..5 {
            let area = Rect::new(2, 1, w, h);
            let mut scene =
                Scene::new("props_narrow", Theme::junie(), ColorLevel::TrueColor, 12, 7);
            scene.draw_over(
                |buf| {
                    for cell in &mut buf.content {
                        cell.set_symbol(".");
                    }
                },
                |ui, _| {
                    let used = Props::rich(&rows).draw(ui, area);
                    let measured = Props::rich(&rows).measure(ui, Constraints::loose(w, h));
                    assert!(used.height <= h);
                    assert_eq!(used.height, measured.preferred.1);
                },
            );
            for y in 0..7 {
                for x in 0..12 {
                    if !area.contains((x, y).into()) {
                        assert_eq!(scene.buffer().cell((x, y)).unwrap().symbol(), ".");
                    }
                }
            }
        }
    }
}

#[test]
fn rich_secret_rows_are_masked_and_never_become_controls() {
    use junie_tui::{
        App, Button, Cx, Id, ItemKey, KeyCode, Part, PropsRow, Response, Role, Secret, StylePatch,
        Ui, UpdateCause,
    };
    use junie_tui_testing::Harness;
    use ratatui_core::style::Color;
    const BEFORE: Id = Id::root("props.before");
    const AFTER: Id = Id::root("props.after");
    struct Model {
        secret: Secret,
    }
    impl App for Model {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.focus(BEFORE);
            }
            Button::new(BEFORE, "Before").update(cx).erase()
                | Button::new(AFTER, "After").update(cx).erase()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            Button::new(BEFORE, "Before").draw(ui, Rect::new(0, 0, 12, 1));
            let rows = [
                PropsRow::secret(ItemKey::num(42), "Token", &self.secret)
                    .tone(Role::Danger)
                    .copyable(),
                PropsRow::new(ItemKey::num(43), "Plain", "public").copyable(),
            ];
            let patch = [(
                Part::LABEL,
                StylePatch::new().set_fg(Role::Custom(Color::Cyan)),
            )];
            let props = Props::rich(&rows).patch_part(&patch);
            assert_eq!(
                props.measure(ui, Constraints::loose(32, 3)).preferred,
                (18, 2)
            );
            props.draw(ui, Rect::new(0, 1, 32, 3));
            Button::new(AFTER, "After").draw(ui, Rect::new(0, 4, 12, 1));
        }
    }
    let mut h = Harness::new(
        Model {
            secret: Secret::new("sensitive".into()),
        },
        Theme::junie(),
        32,
        6,
    );
    assert_eq!(h.focus(), Some(BEFORE));
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(AFTER));
    assert!(h.area_of(Id::root("tui.props")).is_none());
    assert!(!h.text().contains("sensitive"));
    for x in 7..16 {
        let cell = h.buffer().cell((x, 1)).unwrap();
        assert_eq!(cell.symbol(), "•");
        assert_eq!(cell.fg, Color::Cyan);
    }
    let before = h.text();
    let _ = h.click(9, 2);
    let _ = h.key(KeyCode::Char('y'));
    assert_eq!(h.focus(), Some(AFTER));
    assert_eq!(h.text(), before);
}

#[test]
fn semantic_roles_bind_to_each_theme_capability_palette() {
    use junie_tui::{ItemKey, PropsRow, Role};
    for theme in [Theme::junie(), Theme::paper()] {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let expected = theme.clone().downgrade(level).color.danger;
            let rows = [PropsRow::new(ItemKey::num(1), "A", "one two")
                .tone(Role::Danger)
                .wrap()];
            let mut scene = Scene::new("props_levels", theme.clone(), level, 6, 3);
            scene.draw(|ui, area| {
                Props::rich(&rows).draw(ui, area);
            });
            for y in [0, 1] {
                assert_eq!(
                    scene.buffer().cell((3, y)).unwrap().fg,
                    expected,
                    "{level:?}"
                );
            }
        }
    }
}

#[test]
fn secret_measurement_uses_the_authored_mask_glyph_width() {
    use junie_tui::{GlyphRole, ItemKey, PropsRow, Secret};
    let secret = Secret::new("private".into());
    let rows = [PropsRow::secret(ItemKey::num(1), "T", &secret)];
    let mut scene = Scene::new(
        "props_secret_glyph",
        Theme::junie()
            .builder()
            .glyph(GlyphRole::SecretMask, "界")
            .build(),
        ColorLevel::TrueColor,
        20,
        2,
    );
    scene.draw(|ui, area| {
        let props = Props::rich(&rows);
        assert_eq!(
            props.measure(ui, Constraints::loose(20, 2)).preferred,
            (19, 1)
        );
        props.draw(ui, area);
    });
    for x in (3..17).step_by(2) {
        assert_eq!(scene.buffer().cell((x, 0)).unwrap().symbol(), "界");
    }
    let text: String = scene
        .buffer()
        .content
        .iter()
        .map(junie_tui::Cell::symbol)
        .collect();
    assert!(!text.contains("private"));
}

#[test]
fn explicit_clear_does_not_reapply_the_row_tone() {
    use junie_tui::{ItemKey, Part, PropsRow, Role, StylePatch};
    use ratatui_core::style::{Color, Style};
    let rows = [PropsRow::new(ItemKey::num(1), "A", "one two")
        .tone(Role::Danger)
        .wrap()];
    let patches = [(
        Part::LABEL,
        StylePatch::new()
            .clear_fg()
            .set_bg(Role::Custom(Color::Blue)),
    )];
    let mut scene = Scene::new("props_clear", Theme::junie(), ColorLevel::TrueColor, 6, 3);
    scene.draw_over(
        |buf| {
            for cell in &mut buf.content {
                cell.set_style(Style::new().fg(Color::Magenta).bg(Color::Cyan));
            }
        },
        |ui, area| {
            Props::rich(&rows).patch_part(&patches).draw(ui, area);
        },
    );
    for y in [0, 1] {
        let cell = scene.buffer().cell((3, y)).unwrap();
        assert_eq!((cell.fg, cell.bg), (Color::Magenta, Color::Blue));
    }
}
