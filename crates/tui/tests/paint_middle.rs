//! Public borrowed middle painter: immutable helper equivalence and semantic clipping.
use junie_tui::{App, ColorLevel, Cx, Rect, Response, Role, StylePatch, Surface, Theme, Ui};
use junie_tui_testing::Harness;
use std::cell::Cell;

#[global_allocator]
static GLOBAL: junie_tui_testing::perf::Counting = junie_tui_testing::perf::Counting;
struct Page {
    text: &'static str,
    expected: String,
    width: u16,
    clipped: bool,
    used: Cell<u16>,
}
impl Page {
    fn new(text: &'static str, width: u16, clipped: bool) -> Self {
        let budget = if clipped { width.min(5) } else { width };
        Self {
            text,
            expected: junie_tui::truncate_middle(text, budget),
            width,
            clipped,
            used: Cell::new(0),
        }
    }
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
            let x = if self.clipped { 4 } else { 1 };
            let budget = if self.clipped {
                self.width.min(5)
            } else {
                self.width
            };
            ui.paint_str(Rect::new(x, 0, budget, 1), &self.expected, style);
            if self.clipped {
                ui.with_area(Rect::new(4, 1, 5, 1), |ui| {
                    self.used.set(ui.paint_middle(
                        Rect::new(1, 1, self.width.saturating_add(3), 1),
                        self.text,
                        style,
                    ));
                });
            } else {
                self.used
                    .set(ui.paint_middle(Rect::new(1, 1, self.width, 1), self.text, style));
            }
        });
        ui.dim_layer(Rect::new(0, 0, 40, 2), 2);
    }
}
#[test]
fn frozen_literal_examples_preserve_the_existing_budget_algorithm() {
    for (text, width, expected) in [
        ("abcdefghij", 0, ""),
        ("abcdefghij", 1, "…"),
        ("abcdefghij", 4, "abc…"),
        ("abcdefghij", 5, "abc…j"),
        ("abcdefghij", 8, "abcde…ij"),
        ("short", 8, "short"),
        ("界界界尾", 5, "界…"),
        ("e\u{301}abcdefghij", 5, "e\u{301}ab…j"),
    ] {
        assert_eq!(junie_tui::truncate_middle(text, width), expected);
    }
}
#[test]
fn public_paint_matches_owned_helper_across_widths_modes_and_graphemes() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for text in [
            "",
            "a",
            "abcdefghij",
            "界界界尾",
            "e\u{301}abcdefghij",
            "👩‍💻/tools/👨‍👩‍👧/tail",
            "abcdefgh\n\u{301}",
            "\u{301}abc\tdefghi",
        ] {
            for width in 0..24 {
                let h = Harness::new(
                    Page::new(text, width, false),
                    Theme::junie().for_level(level),
                    40,
                    2,
                );
                for x in 0..40 {
                    assert_eq!(
                        h.cell(x, 0),
                        h.cell(x, 1),
                        "{text:?}, width {width}, mode {level:?}, x{x}"
                    );
                }
                assert_eq!(h.app().used.get(), junie_tui::width(&h.app().expected));
            }
        }
    }
}
#[test]
fn ancestor_clip_is_the_budget_and_delayed_dim_preserves_semantic_channels() {
    for text in ["abcdefghij", "界界界尾", "e\u{301}abcdefghij"] {
        for width in 0..12 {
            let h = Harness::new(Page::new(text, width, true), Theme::junie(), 40, 2);
            for x in 0..40 {
                assert_eq!(h.cell(x, 0), h.cell(x, 1), "{text:?}, width {width}, x{x}");
            }
        }
    }
}
#[test]
fn warmed_borrowed_paint_allocates_nothing() {
    let mut h = Harness::new(
        Page::new("界/e\u{301}/a_very_long_identifier_name", 16, false),
        Theme::junie(),
        40,
        2,
    );
    h.draw();
    let before = junie_tui_testing::perf::allocs();
    h.draw();
    assert_eq!(junie_tui_testing::perf::allocs() - before, 0);
}
