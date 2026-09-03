//! State of the data, not of the widget (`COMPONENT_ARCHITECTURE.md` §12.2, §17.0 A8).

use ratatui_core::layout::Rect;

use crate::id::Part;
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{Family, GlyphRole, Variant};
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
            EmptyState::Error { .. } => Some(
                ui.design()
                    .glyphs
                    .get(icon.glyph.unwrap_or(GlyphRole::Error)),
            ),
            EmptyState::Empty { .. } => None,
        };
        let prefix_w = glyph.map_or(0, |g| width(g).saturating_add(1));
        let head = self.title();
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
            row.x = row.x.saturating_add(used).saturating_add(1);
            row.width = row.width.saturating_sub(used).saturating_sub(1);
        }
        ui.paint_str(row, head, title);
        let mut rows = 1u16;
        if let Some(d) = self.detail()
            && area.height >= 3
        {
            let w = width(d).min(area.width);
            let dx = area.x.saturating_add(area.width.saturating_sub(w) / 2);
            ui.paint_str(
                Rect {
                    x: dx,
                    y: area.y.saturating_add(2),
                    width: w,
                    height: 1,
                },
                d,
                help,
            );
            rows = 3;
        }
        rows
    }
}
