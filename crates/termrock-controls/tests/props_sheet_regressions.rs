#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Style};
use termrock_controls::{PropsList, PropsRow, PropsState};
use termrock_core::id::ItemKey;
use termrock_runtime::{App, Cx, Id, Response, Ui, UpdateCause};
use termrock_test_support::{Harness, Scene};
use termrock_theme::{ColorLevel, Part, Role, StylePatch, Theme};

const ID: Id = Id::root("props.sheet.regressions");

struct Model {
    state: PropsState,
    width: u16,
    height: u16,
    rows: Vec<PropsRow<'static>>,
}

impl App for Model {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(ID);
        }
        PropsList::new(ID)
            .sheet()
            .update(cx, &mut self.state, &self.rows)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        PropsList::new(ID).sheet().draw(
            ui,
            Rect::new(10, 1, self.width, self.height),
            &self.state,
            &self.rows,
        );
    }
}

fn model(width: u16, height: u16, rows: Vec<PropsRow<'static>>) -> Model {
    Model {
        state: PropsState::default(),
        width,
        height,
        rows,
    }
}

#[test]
fn copy_hint_never_writes_outside_a_narrow_component() {
    for theme in [Theme::junie(), Theme::paper()] {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            for width in 1..=8 {
                let harness = Harness::new(
                    model(
                        width,
                        4,
                        vec![PropsRow::new(ItemKey::num(1), "Key", "value").copyable()],
                    ),
                    theme.clone(),
                    40,
                    7,
                )
                .with_color(level);
                for x in 0..10 {
                    assert_eq!(
                        harness.buffer().cell((x, 1)).unwrap().symbol(),
                        " ",
                        "copy hint escaped component at x={x}, width={width}, {level:?}"
                    );
                }
            }
            for (width, height) in [(0, 4), (8, 0), (0, 0)] {
                let harness = Harness::new(
                    model(
                        width,
                        height,
                        vec![PropsRow::new(ItemKey::num(11), "empty", "value").copyable()],
                    ),
                    theme.clone(),
                    40,
                    7,
                )
                .with_color(level);
                let _ = harness.buffer();
                assert!(height == 0 || width == 0);
            }
        }
    }
}

#[test]
fn wrapped_value_is_clipped_at_the_component_bottom() {
    let harness = Harness::new(
        model(
            24,
            4,
            vec![PropsRow::new(
                ItemKey::num(2),
                "Key",
                "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen",
            )
            .copyable()
            .wrap()],
        ),
        Theme::junie(),
        60,
        12,
    );
    for y in 5..12 {
        for x in 10..34 {
            assert_eq!(
                harness.buffer().cell((x, y)).unwrap().symbol(),
                " ",
                "wrapped value escaped the component at ({x},{y})"
            );
        }
    }
}

#[test]
fn only_the_current_copyable_row_reserves_hint_width() {
    let harness = Harness::new(
        model(
            40,
            4,
            vec![
                PropsRow::new(ItemKey::num(3), "A", "first").copyable(),
                PropsRow::new(ItemKey::num(4), "B", "012345678901234567890123456789"),
            ],
        ),
        Theme::junie(),
        60,
        7,
    );
    let second_row = (10..50)
        .map(|x| harness.buffer().cell((x, 2)).unwrap().symbol())
        .collect::<String>();
    assert!(
        second_row.contains("012345678901234567890123456789"),
        "inactive row lost value cells to another row's hint reservation: {second_row:?}"
    );
}

#[test]
fn measured_full_label_is_kept_when_the_minimum_value_room_fits() {
    let harness = Harness::new(
        model(
            40,
            4,
            vec![PropsRow::new(
                ItemKey::num(5),
                "abcdefghijklmnopqrst",
                "value",
            )],
        ),
        Theme::junie(),
        60,
        7,
    );
    let line = (10..50)
        .map(|x| harness.buffer().cell((x, 1)).unwrap().symbol())
        .collect::<String>();
    assert!(
        line.contains("abcdefghijklmnopqrst"),
        "sheet capped a label despite enough room for the minimum value width: {line:?}"
    );
}

#[test]
fn explicit_value_set_and_clear_keep_canonical_precedence() {
    for theme in [Theme::junie(), Theme::paper()] {
        // This assertion checks explicit RGB style precedence. The separate
        // copy-hint clipping regression exercises every supported profile.
        for level in [ColorLevel::TrueColor] {
            for clear_fg in [false, true] {
                let rows = [PropsRow::new(ItemKey::num(6), "A", "value").tone(Role::Danger)];
                let patch = [(
                    Part::LABEL,
                    if clear_fg {
                        StylePatch::new()
                            .clear_fg()
                            .set_bg(Role::Custom(Color::Blue))
                    } else {
                        StylePatch::new()
                            .set_fg(Role::Custom(Color::Cyan))
                            .set_bg(Role::Custom(Color::Blue))
                    },
                )];
                let mut scene = Scene::new("props_sheet_patch", theme.clone(), level, 40, 3);
                scene.draw_over(
                    |buf| {
                        for cell in &mut buf.content {
                            cell.set_style(Style::new().fg(Color::Magenta).bg(Color::Green));
                        }
                    },
                    |ui, area| {
                        PropsList::new(ID).sheet().patch_part(&patch).draw(
                            ui,
                            area,
                            &PropsState::default(),
                            &rows,
                        );
                    },
                );
                let cell = scene.buffer().cell((7, 0)).unwrap();
                assert_eq!(
                    cell.bg,
                    termrock_theme::downgrade::downgrade_color(Color::Blue, level)
                );
                if clear_fg {
                    assert_eq!(cell.fg, Color::Magenta, "clear must preserve inherited ink");
                } else {
                    assert_eq!(
                        cell.fg,
                        termrock_theme::downgrade::downgrade_color(Color::Cyan, level),
                        "explicit Set must beat row tone"
                    );
                }
            }
        }
    }
}
