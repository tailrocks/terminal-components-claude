//! Passive, same-layer reveal for logically cut [`Notice`] rows.
//!
//! The caller owns one controller for each live Notice and passes a new
//! `revision` whenever its content or options change. `update` observes the
//! runtime's passive hover state and injected clock. `draw` then resolves the
//! source again from the current Notice and validated row window before it
//! paints anything. The controller never retains the borrowed source text.

use core::time::Duration;

use ratatui_core::layout::{Position, Rect};
use termrock_feedback::{Notice, NoticeCut, NoticeSource};

use crate::scroll::ScrollState;
use crate::theme::Surface;
use crate::ui::{Cx, FrameRead, Ui};
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::response::Response;
use termrock_runtime::author::VerticalWindow;
use termrock_runtime::layer::LayerId;
use termrock_runtime::runtime::Moment;
use termrock_runtime::scroll_region::ScrollRegion;

const HOVER_REST: Duration = Duration::from_millis(300);
const MAX_PANE_ROWS: u16 = 8;
const CUT_PART: Part = Part::custom("notice-reveal-cut");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveCut {
    part: PartRef,
    revision: u64,
    since: Moment,
    ready: bool,
}

/// A passive, same-layer reveal for one caller-owned Notice.
///
/// `owner` must be stable and unique for the live Notice. Increment the
/// revision passed to [`Self::update`] and [`Self::draw`] whenever its text or
/// cut-related options change. The cut identity combines the semantic source
/// and Notice-relative row, so pointer movement within one row keeps the
/// original rest deadline while another row starts a new one.
///
/// This painter does not take focus, set a cursor, or handle pointer presses.
/// Its scroll area is passive: only vertical wheel input inside an overflowing
/// reveal is considered, and the runtime forwards input at its ends to another
/// scroll region on the same layer.
#[derive(Debug)]
pub struct NoticeReveal {
    owner: Id,
    scroll_id: Id,
    active: Option<ActiveCut>,
    scroll: ScrollState,
}

impl NoticeReveal {
    /// Create a reveal controller for one live Notice owner.
    pub fn new(owner: Id) -> Self {
        Self {
            owner,
            scroll_id: owner.sub("notice-reveal-scroll"),
            active: None,
            scroll: ScrollState::default(),
        }
    }

    /// Observe the current passive hover and advance the 300 ms rest delay.
    ///
    /// Call this from the owning app's update path, including tick and settle
    /// updates. It stores only a stable part identity, revision, and time. The
    /// current borrowed Notice is resolved by [`Self::draw`] each frame.
    pub fn update(&mut self, cx: &mut Cx<'_>, revision: u64) -> Response<()> {
        let now = cx.now();
        let hovered = if cx.top_layer() == LayerId::PAGE && cx.capture_owner().is_none() {
            cx.hovered_part(self.owner)
                .filter(|part| part.part == CUT_PART && part.item.is_some())
        } else {
            None
        };

        let mut response = Response::ignored();
        match (self.active, hovered) {
            (Some(active), Some(part)) if active.part == part && active.revision == revision => {}
            (_, Some(part)) => {
                self.active = Some(ActiveCut {
                    part,
                    revision,
                    since: now,
                    ready: false,
                });
                self.scroll = ScrollState::default();
                response |= Response::changed();
            }
            (Some(_), None) => {
                self.clear();
                response |= Response::changed();
            }
            (None, None) => {}
        }

        let Some(mut active) = self.active else {
            return response;
        };
        let deadline = active.since.saturating_add(HOVER_REST);
        if now < deadline {
            cx.request_repaint_at(deadline);
            return response;
        }
        if !active.ready {
            active.ready = true;
            self.active = Some(active);
            response |= Response::changed();
        }

        let content_rows = self.scroll.content_len();
        response |= ScrollRegion::new(self.scroll_id)
            .scrollbar_visible(false)
            .update(cx, &mut self.scroll, content_rows);
        response
    }

    /// Clear the active reveal when its owning page or Notice becomes inactive.
    ///
    /// A previously requested repaint deadline is harmless after this call:
    /// `update` starts a fresh rest interval only from a currently registered
    /// source row, and `draw` resolves that row against the current Notice.
    pub fn reset(&mut self) {
        self.clear();
    }

    /// Register current cut rows and paint the matching reveal after the
    /// Notice's normal draw.
    ///
    /// `logical_area` describes the un-clipped Notice body: its x, width, and
    /// height must match the row-only `window`'s horizontal geometry and
    /// logical row count. `window` supplies physical rows after scrolling;
    /// the current Ui clip supplies the final visible intersection. Calling
    /// this once after all live Notice content is drawn keeps the reveal above
    /// the page while it stays in the current layer.
    ///
    /// The returned rectangle is the actual painted pane, or `None` when the
    /// current source is absent, off-window, clipped out, suppressed, or not
    /// yet ready. The pane itself is paint-only apart from its opt-in passive
    /// scroll region.
    pub fn draw(
        &mut self,
        ui: &mut Ui<'_>,
        notice: &Notice<'_>,
        logical_area: Rect,
        window: VerticalWindow,
        revision: u64,
    ) -> Option<Rect> {
        if ui.is_inert() || ui.layer_id() != LayerId::PAGE {
            self.clear();
            return None;
        }

        let visible_body = window.visible_body();
        let Some(logical_rows) = u16::try_from(window.logical_rows()).ok() else {
            self.clear();
            return None;
        };
        if logical_area.x != visible_body.x
            || logical_area.width != visible_body.width
            || logical_area.height != logical_rows
            || visible_body.is_empty()
        {
            self.clear();
            return None;
        }

        if self
            .active
            .is_some_and(|active| active.revision != revision)
        {
            self.clear();
        }

        let clip = ui.full();
        let active_part = self.active.map(|active| active.part);
        let hovered_part = ui.hovered_part(self.owner);
        // `visit_cuts` borrows Ui immutably while invoking its visitor. Keep
        // only the clipped registrations that must be added after that borrow
        // ends; original source text is retained only for the active cut.
        let mut visible_targets = Vec::with_capacity(usize::from(visible_body.height));
        let mut resolved: Option<(NoticeCut<'_>, Rect)> = None;
        notice.visit_cuts(ui, logical_area.width, logical_rows, |cut| {
            let Some(row) = window.physical_row_for_source(usize::from(cut.logical_row())) else {
                return;
            };
            let Some(text_x) = logical_area.x.checked_add(cut.inset()) else {
                return;
            };
            let right = logical_area.right();
            if text_x >= right {
                return;
            }
            let target = Rect::new(text_x, row, right.saturating_sub(text_x), 1)
                .intersection(visible_body)
                .intersection(clip);
            if target.is_empty() {
                return;
            }

            let Some(part) = part_for_cut(cut) else {
                return;
            };
            visible_targets.push((part, target));
            if active_part == Some(part) && hovered_part == Some(part) {
                resolved = Some((cut, target));
            }
        });
        for (part, target) in visible_targets {
            ui.register_hover_part(self.owner, part, target);
        }

        let Some((cut, source_area)) = resolved else {
            if self.active.is_some() {
                self.clear();
            }
            return None;
        };
        if !self.active.is_some_and(|active| active.ready) {
            return None;
        }

        let Some(pane) = pane_for_anchor(clip, source_area) else {
            self.clear();
            return None;
        };
        let original_text = cut.original_text();
        let content_rows = wrapped_row_count(original_text, pane.width);
        self.scroll
            .apply_layout(usize::from(pane.height), content_rows);
        let offset = self.scroll.offset();
        let first_visible = offset;
        let end_visible = offset.saturating_add(usize::from(pane.height));

        ui.with_surface(Surface::Elevated, |ui| {
            let style = ui.surface_style();
            ui.fill(pane, style);
            visit_wrapped(original_text, pane.width, |row, column, piece| {
                if row < first_visible || row >= end_visible || piece.is_empty() {
                    return;
                }
                if column >= pane.width {
                    return;
                }
                let Some(y_offset) = u16::try_from(row.saturating_sub(first_visible)).ok() else {
                    return;
                };
                let x = pane.x.saturating_add(column);
                let Some(y) = pane.y.checked_add(y_offset) else {
                    return;
                };
                let width = pane.width.saturating_sub(column);
                ui.paint_str(Rect::new(x, y, width, 1), piece, style);
            });
        });

        if self.scroll.overflows() {
            ui.register_passive_reveal_scroll(self.scroll_id, pane, self.scroll.headroom_v());
        }
        Some(pane)
    }

    fn clear(&mut self) {
        self.active = None;
        self.scroll = ScrollState::default();
    }
}

fn part_for_cut(cut: NoticeCut<'_>) -> Option<PartRef> {
    let source = match cut.source() {
        NoticeSource::Message => 0_u64,
        NoticeSource::Detail => 1_u64,
        NoticeSource::Hint => 2_u64,
        _ => return None,
    };
    let key = (source << 16) | u64::from(cut.logical_row());
    Some(PartRef::item(CUT_PART, ItemKey::num(key)))
}

fn pane_for_anchor(clip: Rect, anchor: Rect) -> Option<Rect> {
    if clip.is_empty() || anchor.is_empty() {
        return None;
    }
    let height = clip.height.min(MAX_PANE_ROWS);
    if height == 0 {
        return None;
    }
    let bottom = clip.bottom();
    let below = anchor.y.checked_add(height);
    let y = if below.is_some_and(|end| end <= bottom) {
        anchor.y
    } else {
        bottom.checked_sub(height)?
    };
    let pane = Rect::new(anchor.x, y, anchor.width, height).intersection(clip);
    pane.contains(Position::new(anchor.x, anchor.y))
        .then_some(pane)
        .filter(|area| !area.is_empty())
}

fn wrapped_row_count(text: &str, width: u16) -> usize {
    visit_wrapped(text, width, |_, _, _| {})
}

/// Stream the same word and grapheme wrapping policy as `termrock_text::wrap`
/// without retaining all rows or building a full-source line vector.
fn visit_wrapped(text: &str, width: u16, mut visit: impl FnMut(usize, u16, &str)) -> usize {
    let width = width.max(1);
    let mut row = 0_usize;
    let mut column = 0_u16;

    for (paragraph_index, paragraph) in text.split('\n').enumerate() {
        if paragraph_index > 0 {
            row = row.saturating_add(1);
            column = 0;
        }
        for word in paragraph.split(' ') {
            let word_width = crate::text::width(word);
            if column == 0 {
                if word_width <= width {
                    emit_piece(row, &mut column, word, word_width, &mut visit);
                } else {
                    hard_wrap(word, width, &mut row, &mut column, &mut visit);
                }
            } else if column.saturating_add(1).saturating_add(word_width) <= width {
                emit_piece(row, &mut column, " ", 1, &mut visit);
                emit_piece(row, &mut column, word, word_width, &mut visit);
            } else {
                row = row.saturating_add(1);
                column = 0;
                if word_width <= width {
                    emit_piece(row, &mut column, word, word_width, &mut visit);
                } else {
                    hard_wrap(word, width, &mut row, &mut column, &mut visit);
                }
            }
        }
    }

    row.saturating_add(1)
}

fn emit_piece(
    row: usize,
    column: &mut u16,
    piece: &str,
    width: u16,
    visit: &mut impl FnMut(usize, u16, &str),
) {
    if !piece.is_empty() {
        visit(row, *column, piece);
    }
    *column = column.saturating_add(width);
}

fn hard_wrap(
    word: &str,
    width: u16,
    row: &mut usize,
    column: &mut u16,
    visit: &mut impl FnMut(usize, u16, &str),
) {
    for (_, grapheme) in crate::text::measure::graphemes(word) {
        let grapheme_width = crate::text::measure::grapheme_width(grapheme);
        let projected = if grapheme_width > width {
            "…"
        } else {
            grapheme
        };
        let projected_width = crate::text::measure::grapheme_width(projected);
        if column.saturating_add(projected_width) > width {
            *row = row.saturating_add(1);
            *column = 0;
        }
        emit_piece(*row, column, projected, projected_width, visit);
    }
}
