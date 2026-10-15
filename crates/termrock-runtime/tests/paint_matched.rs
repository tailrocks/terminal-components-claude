//! Matched emphasis shares plain text clipping, continuations and semantic provenance.
use std::cell::Cell;
use std::rc::Rc;
use termrock_runtime::{App, Cx, Rect, Response, Surface, Ui};
use termrock_test_support::Harness;
use termrock_test_support::perf::{Counting, allocs};
use termrock_theme::{ColorLevel, Modifier, Role, StylePatch, Theme};
#[global_allocator]
static GLOBAL: Counting = Counting;
struct Page {
    text: &'static str,
    matched: &'static [usize],
    width: u16,
    clipped: bool,
    dim: bool,
    used: Rc<Cell<(u16, u16)>>,
}
impl App for Page {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.with_surface(Surface::FieldHover, |ui| {
            let style = ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Accent)
                    .set_bg(Role::CurrentSurface),
            );
            ui.fill(ui.full(), style);
            let draw = |ui: &mut Ui<'_>, y| {
                if y == 0 {
                    ui.paint_str(Rect::new(1, y, self.width, 1), self.text, style)
                } else {
                    ui.paint_matched(
                        Rect::new(1, y, self.width, 1),
                        self.text,
                        self.matched,
                        style,
                    )
                }
            };
            let (plain, matched) = if self.clipped {
                (
                    ui.with_area(Rect::new(4, 0, 5, 1), |ui| draw(ui, 0)),
                    ui.with_area(Rect::new(4, 1, 5, 1), |ui| draw(ui, 1)),
                )
            } else {
                (draw(ui, 0), draw(ui, 1))
            };
            self.used.set((plain, matched));
        });
        if self.dim {
            ui.dim_layer(Rect::new(0, 0, 24, 2), 2);
        }
    }
}
fn page(
    text: &'static str,
    matched: &'static [usize],
    width: u16,
    clipped: bool,
    dim: bool,
    level: ColorLevel,
) -> (Harness<Page>, Rc<Cell<(u16, u16)>>) {
    let used = Rc::new(Cell::new((0, 0)));
    let h = Harness::new(
        Page {
            text,
            matched,
            width,
            clipped,
            dim,
            used: Rc::clone(&used),
        },
        Theme::junie().for_level(level),
        24,
        2,
    );
    (h, used)
}
#[test]
fn empty_indices_are_cell_exact_plain_text_in_every_mode_and_clip() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for text in [
            "",
            "abc",
            "e\u{301}界x",
            "👩‍💻x",
            "a\n界\tb",
            "\u{301}zero",
            "ｶﾞx",
        ] {
            for width in 0..13 {
                for clipped in [false, true] {
                    for dim in [false, true] {
                        let (h, used) = page(text, &[], width, clipped, dim, level);
                        for x in 0..24 {
                            assert_eq!(
                                h.cell(x, 0),
                                h.cell(x, 1),
                                "{text:?} {width} {clipped} {dim} {level:?} x{x}"
                            );
                        }
                        let (plain, matched) = used.get();
                        assert_eq!(plain, matched);
                        assert!(h.diagnostics().is_empty());
                    }
                }
            }
        }
    }
}
#[test]
fn ordinals_include_controls_and_preserve_combining_and_wide_shadow_cells() {
    for (text, matched, expected) in [
        ("e\u{301}界x", &[0, 2][..], &[(0, "e\u{301}"), (3, "x")][..]),
        ("a\n界\tb", &[2, 4][..], &[(1, "界"), (3, "b")][..]),
        ("👩‍💻x", &[0, 0, 99][..], &[(0, "👩‍💻")][..]),
    ] {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            for dim in [false, true] {
                let (h, _) = page(text, matched, 12, false, dim, level);
                for x in 0..24 {
                    let plain = h.cell(x, 0);
                    let actual = h.cell(x, 1);
                    let mut expected_cell = plain.clone();
                    if expected.iter().any(|(offset, symbol)| {
                        x == 1u16.saturating_add(*offset) && plain.symbol() == *symbol
                    }) {
                        expected_cell.modifier.insert(Modifier::BOLD);
                    }
                    assert_eq!(*actual, expected_cell, "{text:?} {level:?} dim{dim} x{x}");
                }
            }
        }
    }
}
#[test]
fn matched_wide_grapheme_refuses_partial_write_and_later_text() {
    let (h, used) = page("界x", &[0, 1], 1, false, false, ColorLevel::TrueColor);
    assert_eq!(used.get(), (0, 0));
    assert_eq!(h.cell(1, 0), h.cell(1, 1));
}
#[test]
fn warmed_matched_paint_allocates_nothing() {
    let (mut h, _) = page(
        "e\u{301}界👩‍💻text",
        &[0, 2, 5],
        12,
        false,
        true,
        ColorLevel::TrueColor,
    );
    h.draw();
    let before = allocs();
    h.draw();
    assert_eq!(allocs().checked_sub(before), Some(0));
}
