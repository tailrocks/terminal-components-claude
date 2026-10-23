use ratatui_core::layout::Rect;

use crate::measure::{Constraints, Size};
use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Role, StylePatch};
use crate::ui::Ui;

/// Plain title, subtitle, and optional body text with no surrounding chrome.
///
/// Each line is painted independently. Missing lines and cells outside the
/// painted text remain untouched.
#[derive(Clone, Copy, Debug)]
pub struct TextSection<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
    body: Option<&'a str>,
    body_row: u16,
    inset: u16,
    title_patch: StylePatch,
    text_patch: StylePatch,
    title_style: Option<PaintStyle>,
}

impl<'a> TextSection<'a> {
    #[must_use]
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            subtitle: None,
            body: None,
            body_row: 3,
            inset: 1,
            title_patch: StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(crate::theme::Modifier::BOLD),
            text_patch: StylePatch::new().set_fg(Role::Fg(FgStep::Muted)),
            title_style: None,
        }
    }

    #[must_use]
    pub const fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    #[must_use]
    pub const fn body(mut self, body: &'a str) -> Self {
        self.body = Some(body);
        self
    }

    #[must_use]
    pub const fn body_row(mut self, row: u16) -> Self {
        self.body_row = row;
        self
    }

    #[must_use]
    pub const fn inset(mut self, cells: u16) -> Self {
        self.inset = cells;
        self
    }

    #[must_use]
    pub const fn title_patch(mut self, patch: StylePatch) -> Self {
        self.title_patch = patch;
        self
    }

    #[must_use]
    pub const fn text_patch(mut self, patch: StylePatch) -> Self {
        self.text_patch = patch;
        self
    }

    #[must_use]
    pub const fn title_style(mut self, style: PaintStyle) -> Self {
        self.title_style = Some(style);
        self
    }

    /// Measure the section's intrinsic text extent, clipped to its constraints.
    ///
    /// Each configured line remains a single painted row; long lines are
    /// clipped by the available width rather than wrapped.
    pub fn measure(&self, _ui: &Ui<'_>, constraints: Constraints) -> Size {
        let text_width = width(self.title)
            .max(self.subtitle.map_or(0, width))
            .max(self.body.map_or(0, width));
        let width = text_width.saturating_add(self.inset.saturating_mul(2));
        let height = self
            .subtitle
            .map_or(1, |_| 2)
            .max(self.body.map_or(0, |_| self.body_row.saturating_add(1)));
        Size::exact(width, height).fit(constraints)
    }

    pub fn render(self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let line = |row| {
            Rect::new(
                area.x.saturating_add(self.inset),
                area.y.saturating_add(row),
                area.width.saturating_sub(self.inset.saturating_mul(2)),
                1,
            )
        };
        let title = self
            .title_style
            .unwrap_or_else(|| ui.paint_patch(&self.title_patch));
        ui.paint_str(line(0), self.title, title);
        let text = ui.paint_patch(&self.text_patch);
        if area.height > 1
            && let Some(subtitle) = self.subtitle
        {
            ui.paint_str(line(1), subtitle, text);
        }
        if area.height > self.body_row
            && let Some(body) = self.body
        {
            ui.paint_str(line(self.body_row), body, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ColorLevel, Theme};
    use ratatui_core::buffer::{Buffer, Cell, CellDiffOption};
    use ratatui_core::style::{Color, Modifier, Style};
    use termrock_test_support::Scene;

    const SENTINEL_FG: Color = Color::Rgb(17, 29, 43);
    const SENTINEL_BG: Color = Color::Rgb(53, 67, 79);

    fn prefill_sentinels(buffer: &mut Buffer) {
        let style = Style::default()
            .fg(SENTINEL_FG)
            .bg(SENTINEL_BG)
            .add_modifier(Modifier::ITALIC | Modifier::UNDERLINED);
        for cell in &mut buffer.content {
            cell.set_symbol("·")
                .set_style(style)
                .set_diff_option(CellDiffOption::AlwaysUpdate);
        }
    }

    fn assert_untouched(scene: &Scene, before: &[Cell], painted: impl Fn(u16, u16) -> bool) {
        let area = scene.area();
        for y in 0..area.height {
            for x in 0..area.width {
                if painted(x, y) {
                    continue;
                }
                let index = usize::from(y) * usize::from(area.width) + usize::from(x);
                assert_eq!(
                    scene.buffer().content[index],
                    before[index],
                    "untouched cell at ({x}, {y})"
                );
            }
        }
    }

    fn symbol(scene: &Scene, x: u16, y: u16) -> Option<&str> {
        scene.buffer().cell((x, y)).map(|cell| cell.symbol())
    }

    fn old_section(
        ui: &mut Ui<'_>,
        area: Rect,
        title: &str,
        subtitle: Option<&str>,
        body: Option<&str>,
        body_row: u16,
    ) {
        if area.is_empty() {
            return;
        }
        let line = |row| {
            Rect::new(
                area.x.saturating_add(1),
                area.y.saturating_add(row),
                area.width.saturating_sub(2),
                1,
            )
        };
        let title_style = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(crate::theme::Modifier::BOLD),
        );
        let text_style = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
        ui.paint_str(line(0), title, title_style);
        if area.height > 1
            && let Some(subtitle) = subtitle
        {
            ui.paint_str(line(1), subtitle, text_style);
        }
        if area.height > body_row
            && let Some(body) = body
        {
            ui.paint_str(line(body_row), body, text_style);
        }
    }

    #[test]
    fn output_matches_original_section_rendering_for_offsets_sizes_and_color_levels() {
        let cases = [
            (
                "Files · 界/🦀",
                Some("1 entry · directories first 界"),
                Some("/tmp/🦀"),
                3,
            ),
            ("Review · Save", Some("save changes"), None, 3),
            ("Snapshot", Some("Frame 42 · /tmp"), None, 3),
        ];
        let levels = [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ];
        for level in levels {
            for (title, subtitle, body, body_row) in cases {
                for width in 0..=9 {
                    for height in 0..=5 {
                        let area = Rect::new(2, 1, width, height);
                        let mut old = Scene::new("old-section", Theme::junie(), level, 12, 8);
                        old.draw(|ui, _| old_section(ui, area, title, subtitle, body, body_row));
                        let mut new = Scene::new("new-section", Theme::junie(), level, 12, 8);
                        new.draw(|ui, _| {
                            let mut section = TextSection::new(title).body_row(body_row);
                            if let Some(subtitle) = subtitle {
                                section = section.subtitle(subtitle);
                            }
                            if let Some(body) = body {
                                section = section.body(body);
                            }
                            section.render(ui, area);
                        });
                        assert_eq!(
                            old.buffer().content,
                            new.buffer().content,
                            "{title:?}, {width}x{height}, {level:?}"
                        );
                        assert_eq!(old.cursor_position(), new.cursor_position());
                    }
                }
            }
        }
    }

    #[test]
    fn paints_only_configured_lines_at_inset_and_clips_short_areas() {
        let mut scene = Scene::new("text-section", Theme::junie(), ColorLevel::TrueColor, 10, 4);
        scene.draw(|ui, area| {
            ui.fill(area, ui.surface_style());
            TextSection::new("Heading")
                .subtitle("detail")
                .body("body")
                .render(ui, Rect::new(0, 0, 8, 3));
        });
        assert_eq!(symbol(&scene, 1, 0), Some("H"));
        assert_eq!(symbol(&scene, 6, 0), Some("n"));
        assert_eq!(symbol(&scene, 1, 1), Some("d"));
        assert_eq!(symbol(&scene, 7, 1), Some(" "));
        assert_eq!(symbol(&scene, 1, 2), Some(" "));

        scene.draw(|ui, _area| {
            TextSection::new("Heading").render(ui, Rect::new(2, 1, 6, 1));
        });
        assert_eq!(symbol(&scene, 3, 1), Some("H"));
        assert_eq!(symbol(&scene, 6, 1), Some("d"));
    }

    #[test]
    fn showcase_title_style_preserves_prefilled_cells_and_all_paint_attributes() {
        use ratatui_core::style::{Color, Modifier};

        let title = "Components 界🦀";
        let levels = [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ];
        for level in levels {
            for width in 0..=16 {
                for height in 0..=3 {
                    let area = Rect::new(2, 1, width, height);
                    let mut old = Scene::new("old-showcase-title", Theme::junie(), level, 20, 6);
                    old.draw(|ui, frame| {
                        let plane = ui.surface_style().bg(Color::Rgb(21, 24, 31));
                        ui.fill(frame, plane);
                        let title_style = plane
                            .fg(Color::Rgb(72, 224, 84))
                            .add_modifier(Modifier::BOLD | Modifier::ITALIC);
                        if !area.is_empty() {
                            ui.paint_str(
                                Rect::new(area.x, area.y, area.width, 1),
                                title,
                                title_style,
                            );
                        }
                    });
                    let mut new = Scene::new("new-showcase-title", Theme::junie(), level, 20, 6);
                    new.draw(|ui, frame| {
                        let plane = ui.surface_style().bg(Color::Rgb(21, 24, 31));
                        ui.fill(frame, plane);
                        let title_style = plane
                            .fg(Color::Rgb(72, 224, 84))
                            .add_modifier(Modifier::BOLD | Modifier::ITALIC);
                        TextSection::new(title)
                            .inset(0)
                            .title_style(title_style)
                            .render(ui, area);
                    });
                    assert_eq!(
                        old.buffer().content,
                        new.buffer().content,
                        "{width}x{height}, {level:?}"
                    );
                    assert_eq!(old.cursor_position(), new.cursor_position());
                }
            }
        }
    }

    #[test]
    fn style_patch_and_body_row_overrides_match_direct_painting() {
        let title = "Wide 界🦀 title";
        let subtitle = "Muted detail";
        let body = "Body 界";
        let title_patch = StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .add(crate::theme::Modifier::ITALIC);
        let text_patch = StylePatch::new().set_fg(Role::Fg(FgStep::Primary));
        for level in [ColorLevel::TrueColor, ColorLevel::Ansi16, ColorLevel::Mono] {
            for width in 0..=12 {
                for height in 0..=5 {
                    let area = Rect::new(2, 1, width, height);
                    let mut old = Scene::new("old-custom-section", Theme::junie(), level, 16, 7);
                    old.draw(|ui, _| {
                        if area.is_empty() {
                            return;
                        }
                        let line = |row| {
                            Rect::new(
                                area.x.saturating_add(1),
                                area.y.saturating_add(row),
                                area.width.saturating_sub(2),
                                1,
                            )
                        };
                        let title_style = ui.paint_patch(&title_patch);
                        let text_style = ui.paint_patch(&text_patch);
                        ui.paint_str(line(0), title, title_style);
                        if area.height > 1 {
                            ui.paint_str(line(1), subtitle, text_style);
                        }
                        if area.height > 2 {
                            ui.paint_str(line(2), body, text_style);
                        }
                    });
                    let mut new = Scene::new("new-custom-section", Theme::junie(), level, 16, 7);
                    new.draw(|ui, _| {
                        TextSection::new(title)
                            .subtitle(subtitle)
                            .body(body)
                            .body_row(2)
                            .title_patch(title_patch)
                            .text_patch(text_patch)
                            .render(ui, area);
                    });
                    assert_eq!(
                        old.buffer().content,
                        new.buffer().content,
                        "{width}x{height}, {level:?}"
                    );
                    assert_eq!(old.cursor_position(), new.cursor_position());
                }
            }
        }
    }

    #[test]
    fn measure_uses_display_width_and_configured_rows_then_fits_constraints() {
        let section = TextSection::new("界")
            .subtitle("detail")
            .body("🦀")
            .inset(2);
        let mut scene = Scene::new(
            "text-section-measure",
            Theme::junie(),
            ColorLevel::TrueColor,
            10,
            4,
        );
        let mut natural = None;
        scene.draw(|ui, _| {
            natural = Some(section.measure(ui, Constraints::loose(20, 10)));
        });
        assert_eq!(natural, Some(Size::exact(10, 4)));

        let mut narrow = None;
        scene.draw(|ui, _| {
            narrow = Some(section.measure(ui, Constraints::loose(4, 2)));
        });
        assert_eq!(narrow, Some(Size::exact(4, 2)));

        let mut tight = None;
        scene.draw(|ui, _| {
            tight = Some(section.measure(ui, Constraints::tight(7, 6)));
        });
        assert_eq!(tight, Some(Size::exact(7, 6)));
    }

    #[test]
    fn measured_offset_extent_maps_unicode_lines_and_preserves_padding_cells() {
        let section = TextSection::new("界A")
            .subtitle("e\u{301}B")
            .body("🦀Z")
            .inset(2);
        let mut scene = Scene::new(
            "text-section-measured-unicode",
            Theme::junie(),
            ColorLevel::TrueColor,
            11,
            7,
        );
        let cursor_before = scene.cursor_position();
        let mut measured = None;
        let mut before = Vec::new();
        scene.draw_over(
            |buffer| {
                prefill_sentinels(buffer);
                before = buffer.content.clone();
            },
            |ui, _| {
                let size = section.measure(ui, Constraints::loose(20, 10));
                measured = Some(size);
                section.render(ui, Rect::new(2, 1, size.preferred.0, size.preferred.1));
            },
        );

        assert_eq!(measured, Some(Size::exact(7, 4)));
        assert_eq!(symbol(&scene, 4, 1), Some("界"));
        assert_eq!(scene.buffer().cell((5, 1)), Some(&Cell::EMPTY));
        assert_eq!(symbol(&scene, 6, 1), Some("A"));
        assert_eq!(symbol(&scene, 4, 2), Some("e\u{301}"));
        assert_eq!(symbol(&scene, 5, 2), Some("B"));
        assert_eq!(symbol(&scene, 4, 4), Some("🦀"));
        assert_eq!(scene.buffer().cell((5, 4)), Some(&Cell::EMPTY));
        assert_eq!(symbol(&scene, 6, 4), Some("Z"));

        let title = scene.buffer().cell((4, 1)).unwrap();
        let subtitle = scene.buffer().cell((4, 2)).unwrap();
        let body = scene.buffer().cell((4, 4)).unwrap();
        assert!(title.modifier.contains(Modifier::BOLD));
        assert!(!subtitle.modifier.contains(Modifier::BOLD));
        assert_ne!(title.fg, subtitle.fg);
        assert_eq!(subtitle.fg, body.fg);
        for cell in [title, subtitle, body] {
            assert_ne!(cell.fg, SENTINEL_FG);
            assert_eq!(cell.bg, SENTINEL_BG);
        }

        assert_untouched(&scene, &before, |x, y| {
            matches!((x, y), (4..=6, 1) | (4..=5, 2) | (4..=6, 4))
        });
        assert_eq!(scene.cursor_position(), cursor_before);
    }

    #[test]
    fn measured_nondefault_body_row_contains_body_at_its_configured_row() {
        let section = TextSection::new("T")
            .subtitle("S")
            .body("B")
            .body_row(2)
            .inset(1);
        let mut scene = Scene::new(
            "text-section-measured-body-row",
            Theme::junie(),
            ColorLevel::TrueColor,
            8,
            6,
        );
        let mut measured = None;
        let mut before = Vec::new();
        scene.draw_over(
            |buffer| {
                prefill_sentinels(buffer);
                before = buffer.content.clone();
            },
            |ui, _| {
                let size = section.measure(ui, Constraints::loose(8, 8));
                measured = Some(size);
                section.render(ui, Rect::new(2, 1, size.preferred.0, size.preferred.1));
            },
        );

        assert_eq!(measured, Some(Size::exact(3, 3)));
        assert_eq!(symbol(&scene, 3, 1), Some("T"));
        assert_eq!(symbol(&scene, 3, 2), Some("S"));
        assert_eq!(symbol(&scene, 3, 3), Some("B"));
        assert_untouched(&scene, &before, |x, y| matches!((x, y), (3, 1..=3)));
    }

    #[test]
    fn constrained_measured_extent_clips_without_wrapping_or_revealing_body() {
        let section = TextSection::new("界A")
            .subtitle("Detail")
            .body("Body")
            .inset(2);
        let mut scene = Scene::new(
            "text-section-measured-constrained",
            Theme::junie(),
            ColorLevel::TrueColor,
            12,
            7,
        );
        let mut measured = None;
        let mut before = Vec::new();
        scene.draw_over(
            |buffer| {
                prefill_sentinels(buffer);
                before = buffer.content.clone();
            },
            |ui, _| {
                let size = section.measure(ui, Constraints::loose(6, 2));
                measured = Some(size);
                section.render(ui, Rect::new(3, 2, size.preferred.0, size.preferred.1));
            },
        );

        assert_eq!(measured, Some(Size::exact(6, 2)));
        assert_eq!(symbol(&scene, 5, 2), Some("界"));
        assert_eq!(scene.buffer().cell((6, 2)), Some(&Cell::EMPTY));
        assert_eq!(symbol(&scene, 5, 3), Some("D"));
        assert_eq!(symbol(&scene, 6, 3), Some("e"));
        assert_eq!(symbol(&scene, 7, 2), Some("·"));
        assert_eq!(symbol(&scene, 5, 4), Some("·"));
        assert_untouched(&scene, &before, |x, y| {
            matches!((x, y), (5..=6, 2) | (5..=6, 3))
        });
    }

    #[test]
    fn inset_dominated_and_zero_measured_extents_leave_full_cells_untouched() {
        let inset_section = TextSection::new("A").subtitle("B").body("C").inset(2);
        let mut inset_scene = Scene::new(
            "text-section-measured-inset-boundary",
            Theme::junie(),
            ColorLevel::TrueColor,
            8,
            6,
        );
        let mut inset_size = None;
        let mut inset_before = Vec::new();
        inset_scene.draw_over(
            |buffer| {
                prefill_sentinels(buffer);
                inset_before = buffer.content.clone();
            },
            |ui, _| {
                let size = inset_section.measure(ui, Constraints::loose(3, 4));
                inset_size = Some(size);
                inset_section.render(ui, Rect::new(2, 1, size.preferred.0, size.preferred.1));
            },
        );
        assert_eq!(inset_size, Some(Size::exact(3, 4)));
        assert_eq!(inset_scene.buffer().content, inset_before);

        let mut zero_scene = Scene::new(
            "text-section-measured-zero-boundary",
            Theme::junie(),
            ColorLevel::TrueColor,
            8,
            6,
        );
        let mut zero_size = None;
        let mut zero_before = Vec::new();
        zero_scene.draw_over(
            |buffer| {
                prefill_sentinels(buffer);
                zero_before = buffer.content.clone();
            },
            |ui, _| {
                let size = inset_section.measure(ui, Constraints::loose(0, 0));
                zero_size = Some(size);
                inset_section.render(ui, Rect::new(5, 3, size.preferred.0, size.preferred.1));
            },
        );
        assert_eq!(zero_size, Some(Size::exact(0, 0)));
        assert_eq!(zero_scene.buffer().content, zero_before);
    }
}
