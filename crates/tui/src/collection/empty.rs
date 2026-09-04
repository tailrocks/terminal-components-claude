//! State of the data, not of the widget (`COMPONENT_ARCHITECTURE.md` §12.2, §17.0 A8).

use ratatui_core::layout::Rect;

use crate::id::Part;
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{Family, GlyphRole, Slot, Variant};
use crate::ui::{FrameRead, Ui};

/// Data readiness of a component; the runtime maps it onto
/// `StateFlags::{BUSY, LOADING, ERROR}`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum Status {
    /// Ready.
    #[default]
    Ready,
    /// Busy with an operation.
    Busy,
    /// Loading data.
    Loading,
    /// In error.
    Error,
}

impl Status {
    /// The flags this status adds.
    pub const fn flags(self) -> StateFlags {
        match self {
            Status::Ready => StateFlags::empty(),
            Status::Busy => StateFlags::BUSY,
            Status::Loading => StateFlags::LOADING,
            Status::Error => StateFlags::ERROR,
        }
    }
}

/// How many rows a source has.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RowTotal {
    /// Exactly `n`.
    Exact(usize),
    /// About `n`.
    Estimated(usize),
    /// Unknown.
    Unknown,
}

/// The empty / loading / partial / error vocabulary shared by every collection.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EmptyState<'a> {
    /// No items.
    Empty {
        /// The title.
        title: &'a str,
        /// A hint.
        hint: Option<&'a str>,
    },
    /// Loading.
    Loading {
        /// The label.
        label: &'a str,
    },
    /// Some rows loaded.
    Partial {
        /// Rows loaded.
        loaded: usize,
        /// Rows in total.
        total: RowTotal,
        /// A hint.
        hint: &'a str,
    },
    /// An error.
    Error {
        /// The message.
        message: &'a str,
        /// Detail.
        detail: Option<&'a str>,
    },
}

impl EmptyState<'_> {
    /// The status this state implies.
    pub const fn status(&self) -> Status {
        match self {
            EmptyState::Empty { .. } => Status::Ready,
            EmptyState::Loading { .. } | EmptyState::Partial { .. } => Status::Loading,
            EmptyState::Error { .. } => Status::Error,
        }
    }

    /// The primary line.
    pub const fn title(&self) -> &str {
        match self {
            EmptyState::Empty { title, .. } => title,
            EmptyState::Loading { label } => label,
            EmptyState::Partial { hint, .. } => hint,
            EmptyState::Error { message, .. } => message,
        }
    }

    /// The secondary line, if any.
    pub const fn detail(&self) -> Option<&str> {
        match self {
            EmptyState::Empty { hint, .. } | EmptyState::Error { detail: hint, .. } => *hint,
            EmptyState::Loading { .. } | EmptyState::Partial { .. } => None,
        }
    }

    /// Paint centred: a muted title, a blank row, a faint wrapped hint;
    /// a spinner frame for loading, an error glyph for errors. Returns the
    /// rows used.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, frame: usize) -> u16 {
        if area.is_empty() {
            return 0;
        }
        let flags = self.status().flags();
        let title = ui
            .style(Family::EMPTY, Variant::DEFAULT, Part::TITLE, flags)
            .style;
        let help = ui
            .style(Family::EMPTY, Variant::DEFAULT, Part::HELP, flags)
            .style;
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
        draw_centered(
            ui,
            area,
            CenteredText {
                title: self.title(),
                detail: self.detail(),
                icon_width,
            },
            |ui, icon_area| {
                if let Some(glyph) = glyph {
                    ui.paint_str(icon_area, glyph, icon.style);
                }
            },
            |ui, title_area| {
                ui.paint_str(title_area, self.title(), title);
            },
            |ui, detail_area| {
                if let Some(detail) = self.detail() {
                    ui.paint_str(detail_area, detail, help);
                }
            },
        )
    }
}

/// Inputs for the shared centred readiness painter.
#[derive(Clone, Copy)]
pub(crate) struct CenteredText<'a> {
    pub(crate) title: &'a str,
    pub(crate) detail: Option<&'a str>,
    pub(crate) icon_width: u16,
}

/// Paint centred icon/title/blank/detail geometry while leaving icon style,
/// glyph and slot ownership with the caller. Zero icon width reserves no
/// column and never invokes `paint_icon`.
pub(crate) fn draw_centered(
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
    use crate::theme::Theme;
    use crate::ui::cx::LastFrame;
    use crate::ui::{FrameState, UiCore};

    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 5,
    };

    fn render(e: &EmptyState<'_>, frame: usize) -> (String, u16) {
        let theme = Theme::junie();
        let mut fs = FrameState::default();
        fs.reset(1, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let rows = {
            let mut ui = Ui::new(&mut fs, &mut page, &mut core, &theme, &last);
            e.draw(&mut ui, SCREEN, frame)
        };
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

    /// §12.2: the four data-readiness shapes each render their own primary
    /// and secondary line, map onto the right `StateFlags`, and the spinner
    /// is a pure function of the frame counter.
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

        // the spinner advances with the frame and is otherwise deterministic
        let a = render(&loading, 0).0;
        let b = render(&loading, 1).0;
        assert_ne!(a, b, "the spinner must advance with the frame");
        assert_eq!(a, render(&loading, 0).0);

        // a rect too small to draw into writes nothing and reports 0 rows
        assert_eq!(render(&empty, 0).1, 3);
    }
}
