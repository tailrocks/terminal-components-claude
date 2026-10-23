use ratatui_core::layout::Rect;

use crate::id::Part;
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{Family, GlyphRole, PaintStyle, Slot, Variant};
use crate::ui::{FrameRead, Ui};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum Status {
    #[default]
    Ready,
    Busy,
    Loading,
    Error,
}

impl Status {
    pub const fn flags(self) -> StateFlags {
        match self {
            Status::Ready => StateFlags::empty(),
            Status::Busy => StateFlags::BUSY,
            Status::Loading => StateFlags::LOADING,
            Status::Error => StateFlags::ERROR,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RowTotal {
    Exact(usize),
    Estimated(usize),
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EmptyState<'a> {
    Empty {
        title: &'a str,
        hint: Option<&'a str>,
    },
    Loading {
        label: &'a str,
    },
    Partial {
        loaded: usize,
        total: RowTotal,
        hint: &'a str,
    },
    Error {
        message: &'a str,
        detail: Option<&'a str>,
    },
}

impl EmptyState<'_> {
    pub const fn status(&self) -> Status {
        match self {
            EmptyState::Empty { .. } => Status::Ready,
            EmptyState::Loading { .. } | EmptyState::Partial { .. } => Status::Loading,
            EmptyState::Error { .. } => Status::Error,
        }
    }

    pub const fn title(&self) -> &str {
        match self {
            EmptyState::Empty { title, .. } => title,
            EmptyState::Loading { label } => label,
            EmptyState::Partial { hint, .. } => hint,
            EmptyState::Error { message, .. } => message,
        }
    }

    pub const fn detail(&self) -> Option<&str> {
        match self {
            EmptyState::Empty { hint, .. } | EmptyState::Error { detail: hint, .. } => *hint,
            EmptyState::Loading { .. } | EmptyState::Partial { .. } => None,
        }
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, frame: usize) -> u16 {
        if area.is_empty() {
            return 0;
        }
        self.paint_inherited(ui, area, frame, None)
    }

    pub fn draw_inherited(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        frame: usize,
        inherited: PaintStyle,
    ) -> u16 {
        if area.is_empty() {
            return 0;
        }
        ui.fill(area, inherited);
        self.paint_inherited(ui, area, frame, Some(inherited))
    }

    fn paint_inherited(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        frame: usize,
        inherited: Option<PaintStyle>,
    ) -> u16 {
        let flags = self.status().flags();
        let title = match inherited {
            Some(inherited) => {
                ui.style_inherited(
                    Family::EMPTY,
                    Variant::DEFAULT,
                    Part::TITLE,
                    flags,
                    inherited,
                )
                .style
            }
            None => {
                ui.style(Family::EMPTY, Variant::DEFAULT, Part::TITLE, flags)
                    .style
            }
        };
        let inherited = inherited.unwrap_or_default();
        let help = ui
            .style(Family::EMPTY, Variant::DEFAULT, Part::HELP, flags)
            .over(inherited);
        let icon = ui.style(Family::EMPTY, Variant::DEFAULT, Part::ICON, flags);
        let glyph: Option<&str> = match self {
            EmptyState::Loading { .. } | EmptyState::Partial { .. } => {
                let frames = ui.design().motion.spinner_frames;
                frames
                    .get(frame.checked_rem(frames.len()).unwrap_or(0))
                    .copied()
            }
            EmptyState::Error { .. } => match icon.glyph {
                Slot::Set(g) => Some(ui.design().glyphs.get(g)),
                Slot::Inherit => Some(ui.design().glyphs.get(GlyphRole::Error)),
                Slot::Clear => None,
            },
            EmptyState::Empty { .. } => None,
        };
        let icon_width = glyph.map_or(0, width);

        let title_owned = if width(self.title()) > area.width {
            Some(crate::text::truncate(self.title(), area.width))
        } else {
            None
        };
        let title_text = title_owned.as_deref().unwrap_or_else(|| self.title());
        draw_centered(
            ui,
            area,
            CenteredText {
                title: title_text,
                detail: self.detail(),
                icon_width,
            },
            |ui, icon_area| {
                if let Some(glyph) = glyph {
                    ui.paint_str(icon_area, glyph, icon.over(inherited));
                }
            },
            |ui, title_area| {
                ui.paint_str(title_area, title_text, title);
            },
            |ui, detail_area| {
                if let Some(detail) = self.detail() {
                    ui.paint_str(detail_area, detail, help);
                }
            },
        )
    }
}

#[derive(Clone, Copy)]
pub struct CenteredText<'a> {
    pub title: &'a str,
    pub detail: Option<&'a str>,
    pub icon_width: u16,
}

pub fn draw_centered(
    ui: &mut Ui<'_>,
    area: Rect,
    text: CenteredText<'_>,
    paint_icon: impl FnOnce(&mut Ui<'_>, Rect),
    paint_title: impl FnOnce(&mut Ui<'_>, Rect),
    paint_detail: impl FnOnce(&mut Ui<'_>, Rect),
) -> u16 {
    if area.is_empty() {
        return 0;
    }
    let prefix_width = if text.icon_width == 0 {
        0
    } else {
        text.icon_width.saturating_add(1)
    };
    let total_width = prefix_width
        .saturating_add(width(text.title))
        .min(area.width);
    let x = area
        .x
        .saturating_add(area.width.saturating_sub(total_width) / 2);
    let icon_width = text.icon_width.min(total_width);
    if icon_width != 0 {
        paint_icon(
            ui,
            Rect {
                x,
                y: area.y,
                width: icon_width,
                height: 1,
            },
        );
    }
    let title_x = x.saturating_add(prefix_width.min(total_width));
    paint_title(
        ui,
        Rect {
            x: title_x,
            y: area.y,
            width: total_width.saturating_sub(title_x.saturating_sub(x)),
            height: 1,
        },
    );
    let Some(detail) = text.detail else {
        return 1;
    };
    if area.height < 3 {
        return 1;
    }
    let detail_width = width(detail).min(area.width);
    let detail_x = area
        .x
        .saturating_add(area.width.saturating_sub(detail_width) / 2);
    paint_detail(
        ui,
        Rect {
            x: detail_x,
            y: area.y.saturating_add(2),
            width: detail_width,
            height: 1,
        },
    );
    3
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::text::width;
    use crate::theme::Theme;
    use crate::ui::cx::LastFrame;
    use crate::ui::{FrameState, UiCore};

    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 5,
    };

    fn render_buffer(e: &EmptyState<'_>, frame: usize, theme: Theme) -> (Buffer, u16) {
        let mut fs = FrameState::default();
        fs.reset(1, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let rows = {
            let mut ui = Ui::new(&mut fs, &mut page, &mut core, &theme, &last);
            e.draw(&mut ui, SCREEN, frame)
        };
        assert!(fs.diagnostics.items().is_empty());
        (page, rows)
    }

    fn render(e: &EmptyState<'_>, frame: usize) -> (String, u16) {
        let (page, rows) = render_buffer(e, frame, Theme::junie());
        let mut text = String::new();
        for y in 0..SCREEN.height {
            for x in 0..SCREEN.width {
                if let Some(c) = page.cell((x, y)) {
                    text.push_str(c.symbol());
                }
            }
            text.push('\n');
        }
        (text, rows)
    }

    fn assert_loading_cells(buffer: &Buffer, glyph: &str, label: &str) {
        let glyph_width = width(glyph);
        let prefix_width = if glyph_width == 0 {
            0
        } else {
            glyph_width.saturating_add(1)
        };
        let total_width = prefix_width.saturating_add(width(label)).min(SCREEN.width);
        let x = SCREEN
            .x
            .saturating_add(SCREEN.width.saturating_sub(total_width) / 2);
        if glyph_width > 0 {
            assert_eq!(
                buffer.cell((x, SCREEN.y)).map(|cell| cell.symbol()),
                Some(glyph)
            );
        }
        let label_x = x.saturating_add(prefix_width.min(total_width));
        for (offset, byte) in label.bytes().enumerate() {
            let symbol = char::from(byte).to_string();
            assert_eq!(
                buffer
                    .cell((
                        label_x + u16::try_from(offset).expect("short test label"),
                        SCREEN.y
                    ))
                    .map(|cell| cell.symbol()),
                Some(symbol.as_str()),
                "expected {label:?} after spinner {glyph:?} at x={label_x}"
            );
        }
    }

    #[test]
    fn loading_renders_the_configured_junie_cycle_and_wraps_full_buffers() {
        let theme = Theme::junie();
        let frames = theme.design.motion.spinner_frames;
        assert!(!frames.is_empty());
        let loading = EmptyState::Loading { label: "Loading" };
        assert_eq!(loading.status(), Status::Loading);
        assert_eq!(loading.status().flags(), StateFlags::LOADING);

        let mut first = None;
        for (frame, glyph) in frames.iter().enumerate() {
            let (buffer, rows) = render_buffer(&loading, frame, theme.clone());
            assert_eq!(rows, 1);
            assert_loading_cells(&buffer, glyph, "Loading");
            if frame == 0 {
                first = Some(buffer);
            }
        }

        let first = first.expect("configured Junie spinner has frame zero");
        assert_eq!(
            render_buffer(&loading, frames.len(), theme).0,
            first,
            "the complete styled frame must wrap at the configured sequence length"
        );
    }

    #[test]
    fn loading_uses_a_custom_three_frame_sequence_without_a_ten_frame_assumption() {
        const FRAMES: &[&str] = &["A", "界", "*"];
        let mut theme = Theme::junie();
        theme.design.motion.spinner_frames = FRAMES;
        let loading = EmptyState::Loading { label: "Load" };
        let mut buffers = Vec::new();

        for frame in [0, 1, 2, 3, 4, 5, usize::MAX] {
            let (buffer, rows) = render_buffer(&loading, frame, theme.clone());
            assert_eq!(rows, 1);
            assert_loading_cells(&buffer, FRAMES[frame % FRAMES.len()], "Load");
            buffers.push((frame, buffer));
        }

        assert_eq!(buffers[0].1, buffers[3].1);
        assert_eq!(buffers[1].1, buffers[4].1);
        assert_eq!(buffers[2].1, buffers[5].1);
        assert_eq!(
            buffers[usize::MAX % FRAMES.len()].1,
            buffers[6].1,
            "extreme frame indexes use the configured sequence length"
        );
    }

    #[test]
    fn loading_empty_sequences_draw_the_same_full_buffer() {
        let loading = EmptyState::Loading { label: "Loading" };
        for frame in [0, usize::MAX] {
            let mut no_frames = Theme::junie();
            no_frames.design.motion.spinner_frames = &[];
            let mut blank_frame = Theme::junie();
            blank_frame.design.motion.spinner_frames = &[""];

            let (without_glyph, rows) = render_buffer(&loading, frame, no_frames);
            let (with_blank_glyph, blank_rows) = render_buffer(&loading, frame, blank_frame);
            assert_eq!(rows, 1);
            assert_eq!(blank_rows, 1);
            assert_loading_cells(&without_glyph, "", "Loading");
            assert_loading_cells(&with_blank_glyph, "", "Loading");
            assert_eq!(without_glyph, with_blank_glyph);
        }
    }

    #[test]
    fn empty_state_covers_empty_loading_partial_error() {
        let empty = EmptyState::Empty {
            title: "No rows",
            hint: Some("Adjust the filter"),
        };
        let (t, rows) = render(&empty, 0);
        assert!(t.contains("No rows"), "{t}");
        assert!(t.contains("Adjust the filter"), "{t}");
        assert_eq!(rows, 3);
        assert_eq!(empty.status(), Status::Ready);
        assert_eq!(empty.status().flags(), StateFlags::empty());

        let loading = EmptyState::Loading { label: "Loading" };
        let (t, rows) = render(&loading, 0);
        assert!(t.contains("Loading"), "{t}");
        assert_eq!(rows, 1);
        assert_eq!(loading.status().flags(), StateFlags::LOADING);

        let partial = EmptyState::Partial {
            loaded: 2,
            total: RowTotal::Estimated(9),
            hint: "2 of about 9 sources",
        };
        let (t, _) = render(&partial, 0);
        assert!(t.contains("2 of about 9 sources"), "{t}");
        assert_eq!(partial.status(), Status::Loading);

        let error = EmptyState::Error {
            message: "Failed",
            detail: Some("timed out"),
        };
        let (t, _) = render(&error, 0);
        assert!(t.contains("Failed") && t.contains("timed out"), "{t}");
        assert_eq!(error.status().flags(), StateFlags::ERROR);

        let a = render(&loading, 0).0;
        let b = render(&loading, 1).0;
        assert_ne!(a, b, "the spinner must advance with the frame");
        assert_eq!(a, render(&loading, 0).0);

        assert_eq!(render(&empty, 0).1, 3);
    }

    #[test]
    fn empty_state_truncates_a_long_title_to_the_area() {
        let title = "12345678901234567890123456789012345678901234567890";
        assert_eq!(title.len(), 50);
        let long = EmptyState::Empty { title, hint: None };
        let (t, _) = render(&long, 0);
        assert!(t.contains('…'), "{t}");
        assert!(!t.contains(title), "{t}");
        let first = t.lines().next().unwrap_or("");
        assert!(width(first) <= 40, "{first:?}");

        let short = EmptyState::Empty {
            title: "No rows",
            hint: Some("Adjust the filter"),
        };
        let (t, _) = render(&short, 0);
        assert!(t.contains("No rows"), "{t}");
        assert!(t.contains("Adjust the filter"), "{t}");
        assert!(!t.contains('…'), "{t}");
    }
}
