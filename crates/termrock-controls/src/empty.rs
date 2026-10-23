use core::fmt;

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn};
use crate::id::{Id, Part};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::{width, wrapped_rows};
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};
use termrock_runtime::EmptyState;

pub struct Empty<'a> {
    id: Id,
    state: EmptyState<'a>,
    variant: Variant,
    frame: usize,
    ov: PartStyle<'a>,
}

impl fmt::Debug for Empty<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Empty")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("frame", &self.frame)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> Empty<'a> {
    pub const PARTS: &'static [Part] = &[Part::EMPTY, Part::TITLE, Part::HELP, Part::ICON];

    pub const fn new(id: Id, state: EmptyState<'a>) -> Self {
        Empty {
            id,
            state,
            variant: Variant::DEFAULT,
            frame: 0,
            ov: PartStyle::new(),
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    pub const fn state(&self) -> EmptyState<'a> {
        self.state
    }

    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }

        let live = PartStyle::flags(StateFlags::empty(), self.state.status().flags());
        let ov = self.ov;
        if let Some(f) = ov.slot_for(Part::EMPTY) {
            f(ui, area);
            return area;
        }
        let slot = ov.style(ui, self.id, Family::EMPTY, self.variant, Part::EMPTY, live);
        ui.fill(area, slot.style);
        let rows = self.rows(area.width);
        let top = area.y.saturating_add(area.height.saturating_sub(rows) / 2);
        let block = Rect {
            y: top,
            height: rows.min(area.height),
            ..area
        };
        let used = self.paint_state(ui, block, live);
        Rect {
            height: used,
            ..block
        }
    }

    fn paint_state(&self, ui: &mut Ui<'_>, area: Rect, live: StateFlags) -> u16 {
        if area.is_empty() {
            return 0;
        }
        let title = self
            .ov
            .style(ui, self.id, Family::EMPTY, self.variant, Part::TITLE, live);
        let help = self
            .ov
            .style(ui, self.id, Family::EMPTY, self.variant, Part::HELP, live);
        let icon = self
            .ov
            .style(ui, self.id, Family::EMPTY, self.variant, Part::ICON, live);
        let glyph = match self.state {
            EmptyState::Loading { .. } | EmptyState::Partial { .. } => {
                let frames = ui.design().motion.spinner_frames;
                frames
                    .get(self.frame.checked_rem(frames.len()).unwrap_or(0))
                    .copied()
            }
            EmptyState::Error { .. } => match icon.glyph {
                Slot::Set(g) => Some(ui.glyph_str(g)),
                Slot::Inherit => Some(ui.glyph_str(GlyphRole::Error)),
                Slot::Clear => None,
            },
            EmptyState::Empty { .. } => None,
        };
        let glyph_w = glyph.map_or(0, width);
        let gap_w = u16::from(glyph_w > 0);
        let prefix_w = glyph_w.saturating_add(gap_w);
        let head = self.state.title();
        let total_w = prefix_w.saturating_add(width(head)).min(area.width);
        let x = area
            .x
            .saturating_add(area.width.saturating_sub(total_w) / 2);
        let mut row = Rect {
            x,
            y: area.y,
            width: total_w,
            height: 1,
        };
        if let Some(g) = glyph {
            let used = ui.paint_str(row, g, icon.style);
            row.x = row.x.saturating_add(used).saturating_add(gap_w);
            row.width = row.width.saturating_sub(used).saturating_sub(gap_w);
        }
        ui.paint_str(row, head, title.style);
        let mut rows = 1u16;
        if let Some(detail) = self.state.detail()
            && area.height >= 3
        {
            let detail_w = width(detail).min(area.width);
            let detail_x = area
                .x
                .saturating_add(area.width.saturating_sub(detail_w) / 2);
            ui.paint_str(
                Rect {
                    x: detail_x,
                    y: area.y.saturating_add(2),
                    width: detail_w,
                    height: 1,
                },
                detail,
                help.style,
            );
            rows = 3;
        }
        rows
    }

    fn rows(&self, w: u16) -> u16 {
        match self.state.detail() {
            Some(d) => wrapped_rows(d, w.max(1)).saturating_add(2),
            None => 1,
        }
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let title = width(self.state.title());
        let detail = self.state.detail().map_or(0, width);
        let w = title.max(detail);
        Size {
            min: (title.min(c.max.0), 1),
            preferred: (w, self.rows(c.max.0.max(1))),
        }
        .fit(c)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::{Color, Modifier, Style};

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::{Role, Theme};

    const AREA: Rect = Rect::new(0, 0, 24, 5);
    const EMPTY: Id = Id::root("empty.tests");

    fn cell_with<'a>(buffer: &'a Buffer, symbol: &str) -> Option<&'a ratatui_core::buffer::Cell> {
        buffer.content().iter().find(|cell| cell.symbol() == symbol)
    }

    fn render_in(empty: &Empty<'_>, theme: Theme, screen: Rect, area: Rect) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(screen);
        runtime
            .draw_scene(screen, &mut buffer, |ui, _| {
                empty.draw(ui, area);
            })
            .commit_presented();
        assert!(runtime.diagnostics().is_empty());
        buffer
    }

    fn render_with_outside_sentinels(
        empty: &Empty<'_>,
        theme: Theme,
        screen: Rect,
        area: Rect,
    ) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(screen);
        let sentinel_style = Style::new()
            .fg(Color::Yellow)
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD);
        for y in screen.y..screen.bottom() {
            for x in screen.x..screen.right() {
                if !area.contains(Position::new(x, y)) {
                    buffer
                        .cell_mut((x, y))
                        .expect("sentinel is within the screen")
                        .set_symbol("S")
                        .set_style(sentinel_style);
                }
            }
        }
        let before = buffer.clone();
        runtime
            .draw_scene(screen, &mut buffer, |ui, _| {
                empty.draw(ui, area);
            })
            .commit_presented();
        assert!(runtime.diagnostics().is_empty());
        for y in screen.y..screen.bottom() {
            for x in screen.x..screen.right() {
                if !area.contains(Position::new(x, y)) {
                    assert_eq!(buffer.cell((x, y)), before.cell((x, y)));
                }
            }
        }
        buffer
    }

    fn render(empty: &Empty<'_>, theme: Theme) -> Buffer {
        render_in(empty, theme, AREA, AREA)
    }

    fn theme_with_spinner_frames(frames: &'static [&'static str]) -> Theme {
        let mut theme = Theme::junie();
        theme.design.motion.spinner_frames = frames;
        theme
    }

    fn assert_ascii_text_at(buffer: &Buffer, x: u16, y: u16, text: &str) {
        for (offset, byte) in text.bytes().enumerate() {
            let expected = char::from(byte).to_string();
            assert_eq!(
                buffer
                    .cell((x + u16::try_from(offset).expect("short test label"), y))
                    .map(|cell| cell.symbol()),
                Some(expected.as_str()),
                "expected {text:?} at ({x}, {y}), offset {offset}"
            );
        }
    }

    #[test]
    fn empty_spinner_sequences_do_not_insert_a_gap_before_loading_label() {
        let area = Rect::new(0, 0, 30, 5);
        let loading = Empty::new(EMPTY, EmptyState::Loading { label: "Loading" }).frame(0);
        let no_frames = render_in(&loading, theme_with_spinner_frames(&[]), area, area);
        let blank_frame = render_in(&loading, theme_with_spinner_frames(&[""]), area, area);

        assert_ascii_text_at(&no_frames, 11, 2, "Loading");
        assert_ascii_text_at(&blank_frame, 11, 2, "Loading");
        assert_eq!(no_frames, blank_frame);
    }

    #[test]
    fn loading_custom_frames_render_and_wrap_the_complete_buffer() {
        const FRAMES: &[&str] = &["界", "*"];
        let screen = Rect::new(0, 0, 40, 10);
        let area = Rect::new(5, 3, 30, 5);
        let theme = theme_with_spinner_frames(FRAMES);
        let cases = [(0, "界", 16, 5), (1, "*", 17, 5), (2, "界", 16, 5)];
        let mut buffers = Vec::new();

        for (frame, glyph, glyph_x, y) in cases {
            let empty = Empty::new(EMPTY, EmptyState::Loading { label: "Load" }).frame(frame);
            let buffer = render_with_outside_sentinels(&empty, theme.clone(), screen, area);
            assert_eq!(
                buffer.cell((glyph_x, y)).map(|cell| cell.symbol()),
                Some(glyph)
            );
            assert_eq!(buffer.cell((18, y)).map(|cell| cell.symbol()), Some(" "));
            assert_ascii_text_at(&buffer, 19, y, "Load");
            buffers.push(buffer);
        }

        let extreme = Empty::new(EMPTY, EmptyState::Loading { label: "Load" }).frame(usize::MAX);
        let extreme_buffer = render_with_outside_sentinels(&extreme, theme, screen, area);
        assert_eq!(
            extreme_buffer.cell((17, 5)).map(|cell| cell.symbol()),
            Some("*")
        );
        assert_eq!(
            extreme_buffer.cell((18, 5)).map(|cell| cell.symbol()),
            Some(" ")
        );
        assert_ascii_text_at(&extreme_buffer, 19, 5, "Load");
        assert_eq!(buffers[0], buffers[2]);
        assert_eq!(buffers[1], extreme_buffer);
    }

    #[test]
    fn empty_spinner_sequences_match_at_frame_zero_and_extreme_index() {
        let area = Rect::new(0, 0, 30, 5);
        let theme_empty = theme_with_spinner_frames(&[]);
        let theme_blank = theme_with_spinner_frames(&[""]);
        for frame in [0, usize::MAX] {
            let no_frames =
                Empty::new(EMPTY, EmptyState::Loading { label: "Loading" }).frame(frame);
            let blank_frame =
                Empty::new(EMPTY, EmptyState::Loading { label: "Loading" }).frame(frame);
            let no_frames_buffer = render_in(&no_frames, theme_empty.clone(), area, area);
            let blank_frame_buffer = render_in(&blank_frame, theme_blank.clone(), area, area);

            assert_ascii_text_at(&no_frames_buffer, 11, 2, "Loading");
            assert_ascii_text_at(&blank_frame_buffer, 11, 2, "Loading");
            assert_eq!(no_frames_buffer, blank_frame_buffer);
        }
    }

    #[test]
    fn loading_part_patches_style_the_real_custom_glyph_and_label() {
        const FRAMES: &[&str] = &["*"];
        let area = Rect::new(4, 2, 30, 5);
        let patches = [
            (Part::ICON, StylePatch::new().add(Modifier::REVERSED)),
            (Part::TITLE, StylePatch::new().add(Modifier::ITALIC)),
        ];
        let theme = theme_with_spinner_frames(FRAMES);
        let plain = Empty::new(EMPTY, EmptyState::Loading { label: "Loading" }).frame(usize::MAX);
        let plain_buffer = render_in(&plain, theme.clone(), area, area);
        let empty = Empty::new(EMPTY, EmptyState::Loading { label: "Loading" })
            .frame(usize::MAX)
            .patch_part(&patches);
        let buffer = render_in(&empty, theme, area, area);

        assert_eq!(buffer.cell((14, 4)).map(|cell| cell.symbol()), Some("*"));
        assert!(
            buffer
                .cell((14, 4))
                .is_some_and(|cell| cell.modifier.contains(Modifier::REVERSED))
        );
        assert_ascii_text_at(&buffer, 16, 4, "Loading");
        assert!(
            buffer
                .cell((16, 4))
                .is_some_and(|cell| cell.modifier.contains(Modifier::ITALIC))
        );
        assert!(
            buffer
                .cell((14, 4))
                .is_some_and(|cell| !cell.modifier.contains(Modifier::ITALIC))
        );
        assert!(
            buffer
                .cell((16, 4))
                .is_some_and(|cell| !cell.modifier.contains(Modifier::REVERSED))
        );
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                let before = plain_buffer.cell((x, y)).expect("baseline cell is in area");
                let after = buffer.cell((x, y)).expect("styled cell is in area");
                assert_eq!(after.symbol(), before.symbol());
                assert_eq!(after.fg, before.fg);
                assert_eq!(after.bg, before.bg);
                let expected = if (x, y) == (14, 4) {
                    before.modifier | Modifier::REVERSED
                } else if y == 4 && (16..23).contains(&x) {
                    before.modifier | Modifier::ITALIC
                } else {
                    before.modifier
                };
                assert_eq!(after.modifier, expected, "unexpected patch at ({x}, {y})");
            }
        }
    }

    #[test]
    fn loading_wide_glyph_is_clipped_inside_offset_narrow_areas() {
        const FRAMES: &[&str] = &["界"];
        let screen = Rect::new(0, 0, 40, 9);
        let theme = theme_with_spinner_frames(FRAMES);

        for width in [0, 1, 2, 7] {
            let area = Rect::new(4, 2, width, 5);
            let empty = Empty::new(EMPTY, EmptyState::Loading { label: "Load" });
            let buffer = render_with_outside_sentinels(&empty, theme.clone(), screen, area);
            if width == 1 {
                assert_eq!(buffer.cell((4, 4)).map(|cell| cell.symbol()), Some(" "));
            } else if width == 2 {
                assert_eq!(buffer.cell((4, 4)).map(|cell| cell.symbol()), Some("界"));
            } else if width == 7 {
                assert_eq!(buffer.cell((4, 4)).map(|cell| cell.symbol()), Some("界"));
                assert_ascii_text_at(&buffer, 7, 4, "Load");
            }
        }
    }

    #[test]
    fn nested_parts_honor_instance_patches() {
        let all = StylePatch::new().add(Modifier::BOLD);
        let parts = [
            (Part::TITLE, StylePatch::new().add(Modifier::ITALIC)),
            (Part::HELP, StylePatch::new().add(Modifier::UNDERLINED)),
            (Part::ICON, StylePatch::new().add(Modifier::REVERSED)),
        ];
        let buffer = render(
            &Empty::new(
                EMPTY,
                EmptyState::Error {
                    message: "T",
                    detail: Some("H"),
                },
            )
            .patch(&all)
            .patch_part(&parts),
            Theme::junie(),
        );
        let icon = Theme::junie().design.glyphs.get(GlyphRole::Error);

        assert!(
            cell_with(&buffer, "T")
                .map(|cell| cell.modifier)
                .unwrap_or_default()
                .contains(Modifier::BOLD | Modifier::ITALIC)
        );
        assert!(
            cell_with(&buffer, "H")
                .map(|cell| cell.modifier)
                .unwrap_or_default()
                .contains(Modifier::BOLD | Modifier::UNDERLINED)
        );
        assert!(
            cell_with(&buffer, icon)
                .map(|cell| cell.modifier)
                .unwrap_or_default()
                .contains(Modifier::BOLD | Modifier::REVERSED)
        );
    }

    #[test]
    fn readiness_reaches_nested_parts() {
        let theme = Theme::junie().override_family(Family::EMPTY, |recipe| {
            recipe
                .part(Part::TITLE)
                .when(StateFlags::ERROR, StylePatch::new().set_fg(Role::Info));
        });
        let info = theme.color.info;
        let error = theme.design.glyphs.get(GlyphRole::Error);
        let buffer = render(
            &Empty::new(
                EMPTY,
                EmptyState::Error {
                    message: "T",
                    detail: None,
                },
            ),
            theme,
        );

        assert_eq!(cell_with(&buffer, "T").map(|cell| cell.fg), Some(info));
        assert!(cell_with(&buffer, error).is_some());
    }
}
