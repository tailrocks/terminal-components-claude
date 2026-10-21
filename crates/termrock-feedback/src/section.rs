use ratatui_core::layout::Rect;

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
    use termrock_test_support::Scene;

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
}
