//! The footer every app draws on its last row (plan S8b, `shell::Footer`):
//! one composition of hints, the EDIT badge and the status line, so no app
//! writes a `draw_footer` of its own.
//!
//! Composition, in order:
//!
//! 1. **Hints**: the top layer's ([`crate::ui::layer::Layer::hints`],
//!    library modals derive theirs, CO4), else an open menu's, else the
//!    page's ([`super::Page::hints`]), else [`Footer::fallback`]. The first
//!    present layer wins ([`Footer::layer`]).
//! 2. The reveal key while the focused row is cut
//!    ([`crate::ui::reveal::advertise`]), and `Tab Next` when the page
//!    did not name Tab ([`Footer::tab_next`]).
//! 3. Hints are filtered to what the terminal delivers: a Cmd chord only
//!    once the kitty protocol confirmed it (SI7, SIg4), never Shift+click
//!    or Shift+drag (terminals keep them for their own selection).
//! 4. The status line at the right edge. An **error** takes precedence
//!    over every hint, the pinned ones included: it is never cut to leave
//!    the way out room (SV11, CO5). Other statuses keep the pinned hints'
//!    room.
//! 5. The drawn hints become click targets through
//!    [`Shell::footer_drawn`] (CO5), unless the frame is inert.
//!
//! Every rule stays overridable: alignment ([`Footer::align`], default the
//! theme's, left since SVg10), the fallback, `Tab Next`, and whether the
//! commit key stays pinned while the page is dirty ([`Footer::pin_dirty`],
//! owner decision D2, off by default).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::theme::{BadgeKind, HintAlign, StatusKind, Tone};
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::keyhint::{self, Hint, HintPriority, hint};

use super::Shell;

/// The status a footer shows at its right edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FooterStatus {
    /// A message in a tone (`!` leads an error, `▲` a warning), with
    /// optional parts in their own tones.
    Toned {
        text: String,
        tone: Tone,
        parts: Vec<(String, Tone)>,
    },
    /// A message of a [`StatusKind`] (`✓ Saved`, `✗ …`): its glyph and
    /// colour from the theme; `None` is a neutral message.
    Kind {
        text: String,
        kind: Option<StatusKind>,
    },
}

impl FooterStatus {
    /// Whether the status is an error, which wins over every hint.
    pub fn is_error(&self) -> bool {
        match self {
            Self::Toned { tone, .. } => *tone == Tone::Error,
            Self::Kind { kind, .. } => *kind == Some(StatusKind::Fail),
        }
    }
}

/// One frame's footer: build it from what is on top, then
/// [`Footer::render`].
#[derive(Debug, Clone, Default)]
pub struct Footer {
    /// The hints of the topmost context that has any (see [`Self::layer`]).
    pub hints: Option<Vec<Hint>>,
    /// Shown when no layer brought hints. Default none; apps set their
    /// way to help and back (`F1 Help · Esc Back`).
    pub fallback: Vec<Hint>,
    /// The EDIT badge leads the row while a field edits.
    pub editing: bool,
    /// Append `Tab Next` when no hint names Tab. Default off; a page with
    /// a Tab ring turns it on while nothing captures Tab.
    pub tab_next: bool,
    /// The status at the right edge.
    pub status: Option<FooterStatus>,
    /// Alignment for this footer. `None` follows the theme
    /// ([`crate::theme::HintBarStyle::align`]).
    pub align: Option<HintAlign>,
    /// Owner decision D2: keep the hint whose key is this (the commit
    /// action, `Ctrl+S`) pinned while [`Self::dirty`]. `None` (the
    /// default): only the theme's pinned keys (Esc, `?`, F1) are pinned.
    pub pin_dirty: Option<&'static str>,
    /// The page reports unsaved changes (see [`Self::pin_dirty`]).
    pub dirty: bool,
    /// Leave the status slot empty instead of showing the peek line (the
    /// focused cut value in full, [`RenderCtx::peek`]). Default `false`:
    /// the peek line shows (SI8). See [`Self::peek`].
    pub hide_peek: bool,
}

impl Footer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Offer one context's hints; the first context offered with hints
    /// wins, so call it from the topmost context down (a modal, an open
    /// menu, the page). `None` or empty hints let the next one speak.
    pub fn layer(mut self, hints: Option<Vec<Hint>>) -> Self {
        if self.hints.is_none() {
            self.hints = hints.filter(|h| !h.is_empty());
        }
        self
    }

    /// See [`Self::fallback`].
    pub fn fallback(mut self, hints: Vec<Hint>) -> Self {
        self.fallback = hints;
        self
    }

    /// See [`Self::editing`].
    pub fn editing(mut self, on: bool) -> Self {
        self.editing = on;
        self
    }

    /// See [`Self::tab_next`].
    pub fn tab_next(mut self, on: bool) -> Self {
        self.tab_next = on;
        self
    }

    /// See [`Self::align`].
    pub fn align(mut self, align: HintAlign) -> Self {
        self.align = Some(align);
        self
    }

    /// Show the peek line while the status slot is free (default `true`;
    /// [`Self::hide_peek`]).
    pub fn peek(mut self, on: bool) -> Self {
        self.hide_peek = !on;
        self
    }

    /// See [`Self::pin_dirty`].
    pub fn pin_dirty(mut self, key: Option<&'static str>) -> Self {
        self.pin_dirty = key;
        self
    }

    /// See [`Self::dirty`].
    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    /// A toned status (`None`: no status).
    pub fn status(mut self, status: Option<(&str, Tone)>) -> Self {
        self.status = status.map(|(text, tone)| FooterStatus::Toned {
            text: text.to_owned(),
            tone,
            parts: vec![],
        });
        self
    }

    /// A toned status made of parts in their own tones.
    pub fn status_parts(
        mut self,
        status: Option<(&str, Tone)>,
        parts: Vec<(String, Tone)>,
    ) -> Self {
        self.status = status.map(|(text, tone)| FooterStatus::Toned {
            text: text.to_owned(),
            tone,
            parts,
        });
        self
    }

    /// A status of a [`StatusKind`].
    pub fn status_kind(mut self, status: Option<(&str, Option<StatusKind>)>) -> Self {
        self.status = status.map(|(text, kind)| FooterStatus::Kind {
            text: text.to_owned(),
            kind,
        });
        self
    }

    /// The hints this footer draws, before they compete for room: the
    /// winning layer's (else the fallback), the reveal key, `Tab Next`,
    /// filtered to deliverable keys, with the D2 pin and the error rule
    /// applied.
    pub fn compose(&self, reveal: Option<(&'static str, &'static str)>) -> Vec<Hint> {
        let mut hints = self.hints.clone().unwrap_or_else(|| self.fallback.clone());
        crate::ui::reveal::advertise(&mut hints, reveal, |h| h.key, hint);
        if self.tab_next && !self.editing && !hints.iter().any(|h| h.key == "Tab") {
            hints.push(hint("Tab", "Next"));
        }
        hints.retain(|h| deliverable(h.key));
        if self.dirty
            && let Some(key) = self.pin_dirty
        {
            for h in hints.iter_mut().filter(|h| h.key == key) {
                h.priority = HintPriority::Pinned;
            }
        }
        if self.status.as_ref().is_some_and(FooterStatus::is_error) {
            // an error wins over every hint: none keeps room it would cut
            for h in &mut hints {
                h.priority = HintPriority::Normal;
            }
        }
        hints
    }

    /// Draw the footer on `area` (one row) and register its hints as click
    /// targets with `shell` (after the overlays, [`Shell::footer_drawn`]).
    pub fn render<P>(
        mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &RenderCtx,
        shell: &mut Shell<P>,
    ) {
        let t = ctx.theme;
        fill(buf, area, t.base());
        let hints = self.compose(ctx.reveal_hint);
        let badge = self.editing.then_some(("EDIT", BadgeKind::Edit));
        let style = crate::theme::HintBarStyle {
            align: self.align.unwrap_or(t.hintbar.align),
            ..t.hintbar
        };
        // the peek line: the focused cut value in full, while nothing else
        // is said in the status slot (SI8). It is the least important part
        // of the row: every hint keeps its room (keys before a preview),
        // the peek takes only the width the hints leave, cut with `…`, and
        // is left out when less than `PEEK_MIN` cells are left
        if !self.hide_peek
            && self.status.is_none()
            && let Some(text) = &ctx.peek
        {
            let need = keyhint::width(&hints, badge.map(|(b, _)| b), None);
            // the status gap and right margin
            let room = area.width.saturating_sub(need + 3) as usize;
            if room >= PEEK_MIN {
                self.status = Some(FooterStatus::Toned {
                    text: crate::ui::text::truncate(text, room),
                    tone: Tone::Secondary,
                    parts: vec![],
                });
            }
        }
        let spots = match &self.status {
            Some(FooterStatus::Kind { text, kind }) => {
                keyhint::render_status_styled_at(
                    area,
                    buf,
                    t,
                    &hints,
                    badge,
                    Some((text.as_str(), *kind)),
                    &style,
                )
                .1
            }
            Some(FooterStatus::Toned { text, tone, parts }) => {
                keyhint::render_parts_at(
                    area,
                    buf,
                    t,
                    &hints,
                    badge,
                    Some((text.as_str(), *tone)),
                    parts,
                    &style,
                )
                .1
            }
            None => keyhint::render_parts_at(area, buf, t, &hints, badge, None, &[], &style).1,
        };
        // an inert frame (a handoff) takes no clicks
        if !ctx.inert {
            shell.footer_drawn(hints, spots);
        }
    }
}

/// The narrowest peek line worth drawing; below it the line is left out.
const PEEK_MIN: usize = 8;

/// Whether a hint's key is one the terminal delivers
/// ([`crate::core::keyboard::label_deliverable`]): the footer, the help
/// reference and menu shortcuts all ask it.
pub fn deliverable(key: &str) -> bool {
    crate::core::keyboard::label_deliverable(key)
}

#[cfg(test)]
#[path = "footer_tests.rs"]
mod tests;
