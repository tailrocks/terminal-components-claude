#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "Source-bound public consumer assertions and retained full-cell capture"
)]
use ratatui_core::layout::Rect;
use termrock_controls::{Props, PropsRow};
use termrock_core::id::ItemKey;
use termrock_layout::Constraints;
use termrock_runtime::{App, Cx, Id, KeyCode, Response, StateFlags, Ui, UpdateCause};
use termrock_test_support::{Harness, Scene};
use termrock_text::Secret;
use termrock_theme::{
    ColorLevel, Family, GlyphRole, Overlay, OverlayRule, Part, Role, StylePatch, Theme, Variant,
};

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
                let draw = |ui: &mut Ui<'_>| {
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
            termrock_controls::Button::new(BEFORE, "Before")
                .update(cx)
                .erase()
                | termrock_controls::Button::new(AFTER, "After")
                    .update(cx)
                    .erase()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            termrock_controls::Button::new(BEFORE, "Before").draw(ui, Rect::new(0, 0, 12, 1));
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
            termrock_controls::Button::new(AFTER, "After").draw(ui, Rect::new(0, 4, 12, 1));
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
        .map(ratatui_core::buffer::Cell::symbol)
        .collect();
    assert!(!text.contains("private"));
}

#[test]
fn explicit_clear_does_not_reapply_the_row_tone() {
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

#[test]
fn sheet_rows_keep_a_single_focus_gutter_and_copy_hint_tracks_live_focus() {
    use termrock_controls::{PropsAction, PropsList, PropsState};

    const SHEET: termrock_runtime::Id = termrock_runtime::Id::root("props.sheet-contract");
    const RUN: ItemKey = ItemKey::num(7001);
    const ROLE: ItemKey = ItemKey::num(7002);
    const ROWS: [PropsRow<'static>; 2] = [
        PropsRow::new(RUN, "Run id:", "run-0123456789abcdef-XYZ")
            .copyable()
            .wrap(),
        PropsRow::new(ROLE, "Role:", "builder"),
    ];

    #[derive(Default)]
    struct Model {
        state: PropsState,
        copied: Option<ItemKey>,
    }

    impl App for Model {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.focus(SHEET);
            }
            let response = PropsList::new(SHEET)
                .sheet()
                .update(cx, &mut self.state, &ROWS);
            if let Some(PropsAction::Copy(key)) = response.action_ref() {
                self.copied = Some(*key);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            PropsList::new(SHEET)
                .sheet()
                .draw(ui, Rect::new(0, 0, 40, 5), &self.state, &ROWS);
        }
    }

    let mut harness = Harness::new(Model::default(), Theme::junie(), 40, 5);
    assert_eq!(harness.focus(), Some(SHEET));
    fn line(harness: &Harness<Model>, y: u16) -> String {
        (0..40)
            .map(|x| harness.buffer().cell((x, y)).unwrap().symbol())
            .collect::<String>()
    }
    let focus_glyph = Theme::junie().design.glyphs.get(GlyphRole::FocusBar);
    assert_eq!(line(&harness, 0).chars().next(), focus_glyph.chars().next());
    assert_eq!(
        line(&harness, 0).chars().nth(2),
        Some('R'),
        "sheet labels start after the focus gutter and its spacer"
    );
    assert!(
        line(&harness, 0).contains("y copy"),
        "focused copy row advertises y"
    );
    assert!(
        !line(&harness, 1).contains("y copy"),
        "non-copy row has no hint"
    );
    assert!(
        line(&harness, 1).contains("XYZ"),
        "focused wrapped value reveals its tail"
    );
    assert_eq!(harness.cursor(), None, "read-only rows own no text cursor");

    let hit = harness
        .area_of_part(
            SHEET,
            termrock_runtime::PartRef::item(termrock_runtime::Part::ROW, RUN),
        )
        .expect("sheet row must publish a keyed pointer target");
    let _ = harness.click(hit.x.saturating_add(2), hit.y);
    assert_eq!(harness.app().copied, Some(RUN));
    harness.app_mut().copied = None;

    let _ = harness.key(KeyCode::Char('y'));
    assert_eq!(harness.app().copied, Some(RUN));
    let _ = harness.key(KeyCode::Down);
    assert!(
        !harness
            .text()
            .lines()
            .next()
            .unwrap_or_default()
            .contains("y copy")
    );
    assert!(
        line(&harness, 1).contains("Role:"),
        "collapsed prior row restores row geometry"
    );
}

#[test]
fn sheet_measurement_uses_live_rows_and_handles_unicode_and_zero_width() {
    use termrock_controls::{PropsList, PropsState};

    const ID: termrock_runtime::Id = termrock_runtime::Id::root("props.sheet-measure");
    let long = String::from("界界 e\u{301} 👩‍💻 and a long run identifier");
    let rows = [
        PropsRow::new(ItemKey::num(7101), "Agent:", "Claude").wrap(),
        PropsRow::new(ItemKey::num(7102), "Run id:", long.as_str())
            .copyable()
            .wrap(),
    ];
    let props = PropsList::new(ID).sheet();
    let state = PropsState::default();
    let mut scene = Scene::new(
        "props_sheet_measure",
        Theme::junie(),
        ColorLevel::TrueColor,
        24,
        8,
    );
    scene.draw(|ui, area| {
        let measured = props.measure_rows(
            ui,
            Constraints::loose(area.width, area.height),
            &state,
            &rows,
        );
        assert!(measured.preferred.0 > 0);
        assert!(measured.preferred.1 >= 2);
        assert_ne!(
            measured.preferred,
            PropsList::new(ID)
                .measure(ui, Constraints::loose(area.width, area.height))
                .preferred,
            "data-aware sheet measurement must use the actual row content"
        );
        props.draw(ui, area, &state, &rows);

        let zero_width = props.measure_rows(ui, Constraints::loose(0, 0), &state, &rows);
        assert_eq!(zero_width.preferred, (0, 0));
        props.draw(ui, Rect::new(0, 0, 0, 0), &state, &rows);
    });
    assert!(
        scene
            .buffer()
            .content
            .iter()
            .any(|cell| cell.symbol() == "界")
    );
    assert_eq!(rows[1].value.text(), Some(long.as_str()));

    for color in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let mut profile = Scene::new("props_sheet_profiles", Theme::junie(), color, 24, 8);
        profile.draw(|ui, area| {
            props.draw(ui, area, &state, &rows);
        });
        assert!(
            profile
                .buffer()
                .content
                .iter()
                .any(|cell| cell.symbol() == "界")
        );
    }
}

#[test]
fn sheet_scroll_and_reordered_live_rows_keep_stable_item_identity() {
    use termrock_controls::{PropsList, PropsState};

    const ID: termrock_runtime::Id = termrock_runtime::Id::root("props.sheet-scroll");
    const KEYS: [ItemKey; 5] = [
        ItemKey::num(7201),
        ItemKey::num(7202),
        ItemKey::num(7203),
        ItemKey::num(7204),
        ItemKey::num(7205),
    ];

    fn rows(reversed: bool) -> [PropsRow<'static>; 5] {
        let entries = ["Alpha", "Bravo", "Charlie", "Delta", "Echo"];
        std::array::from_fn(|index| {
            let source = if reversed { 4 - index } else { index };
            PropsRow::new(KEYS[source], entries[source], "value")
        })
    }

    #[derive(Default)]
    struct Model {
        state: PropsState,
        reversed: bool,
    }

    impl App for Model {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.focus(ID);
            }
            let rows = rows(self.reversed);
            PropsList::new(ID)
                .sheet()
                .update(cx, &mut self.state, &rows)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let rows = rows(self.reversed);
            PropsList::new(ID)
                .sheet()
                .draw(ui, Rect::new(0, 0, 32, 3), &self.state, &rows);
        }
    }

    let mut harness = Harness::new(Model::default(), Theme::paper(), 32, 3);
    let _ = harness.key(KeyCode::End);
    let _ = harness.key(KeyCode::Char('!'));
    assert_eq!(harness.app().state.cursor(), Some(KEYS[4]));
    assert!(harness.app().state.scroll().offset() > 0);
    let last = harness
        .area_of_part(
            ID,
            termrock_runtime::PartRef::item(termrock_runtime::Part::ROW, KEYS[4]),
        )
        .expect("scrolling keeps the focused stable key registered");
    assert!(last.y < 3);

    harness.app_mut().reversed = true;
    let _ = harness.key(KeyCode::Char('!'));
    let _ = harness.key(KeyCode::Char('!'));
    assert_eq!(harness.app().state.cursor(), Some(KEYS[4]));
    assert_eq!(harness.app().state.cursor_index(), 0);
    assert!(
        harness
            .text()
            .lines()
            .next()
            .unwrap_or_default()
            .contains("Echo"),
        "the selected value follows its key after the live rows reorder"
    );
    assert!(
        harness
            .area_of_part(
                ID,
                termrock_runtime::PartRef::item(termrock_runtime::Part::ROW, KEYS[4]),
            )
            .is_some(),
        "the same key still owns its actual rendered row"
    );
}
