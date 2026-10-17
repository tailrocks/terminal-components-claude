//! What a list-like widget shows in place of its rows: loading, a load
//! error, or nothing to show.
//!
//! Every row widget built on [`ListCore`](crate::ui::list_core::ListCore)
//! (List, Tree, Props, Steps) holds a [`ContentState`]. While it is not
//! [`ContentState::Ready`], or while it is ready with no rows, the widget
//! draws the placeholder instead of rows ([`ListCore::draw_content`](crate::ui::list_core::ListCore::draw_content)),
//! takes no keys and, by default, is no focus stop (K37,
//! [`ListCore::skip_when_empty`](crate::ui::list_core::ListCore::skip_when_empty)).
//! An error's [`EmptyState::actions`] (Retry) are real buttons: they take
//! focus, hover, press and clicks like any [`Button`](crate::widgets::button::Button).

use std::borrow::Cow;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::theme::Theme;
use crate::ui::ctx::RenderCtx;
use crate::widgets::empty::{EmptyKind, EmptyState};

/// The content of a list-like widget.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ContentState {
    /// The rows show; with no rows the widget's empty placeholder
    /// ([`ListCore::empty`](crate::ui::list_core::ListCore::empty)).
    #[default]
    Ready,
    /// The rows are on their way: a spinner and this label (empty: the
    /// theme's `loading_rows` string).
    Loading(Cow<'static, str>),
    /// Loading failed: this error state (title, hint and actions such as
    /// Retry) replaces the rows.
    Error(EmptyState),
}

impl ContentState {
    /// Loading with `label` (`""` for the theme's string).
    pub fn loading(label: impl Into<Cow<'static, str>>) -> Self {
        Self::Loading(label.into())
    }

    /// A load error; `state` is drawn in the error tone whatever its kind.
    pub fn error(state: EmptyState) -> Self {
        Self::Error(EmptyState {
            kind: EmptyKind::Error,
            ..state
        })
    }

    /// The rows show.
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// The action buttons this state draws (an error's Retry), for an
    /// owner routing their clicks and keys.
    pub fn actions(&self) -> &[crate::widgets::empty::EmptyAction] {
        match self {
            Self::Error(e) => &e.actions,
            _ => &[],
        }
    }
}

/// One muted line centred in `area` (a list's plain empty text).
pub fn draw_line(area: Rect, buf: &mut Buffer, t: &Theme, text: &str, bg: Color) {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return;
    }
    let msg = crate::ui::text::truncate(text, area.width as usize);
    let y = area.y + area.height / 2;
    let x = area.x + area.width.saturating_sub(crate::ui::text::width(&msg) as u16) / 2;
    buf.set_string(x, y, &msg, t.muted().bg(bg));
}

/// A spinner and `label` centred in `area`.
pub fn draw_loading(area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, label: &str, bg: Color) {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return;
    }
    let label = if label.is_empty() {
        ctx.theme.strings.loading_rows
    } else {
        label
    };
    let label = crate::ui::text::truncate(label, area.width.saturating_sub(2) as usize);
    let w = 2 + crate::ui::text::width(&label) as u16;
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height / 2;
    crate::widgets::progress::render_spinner(Rect::new(x, y, w, 1), buf, ctx, &label, bg);
}

/// An empty state: a bare title (no hint, no actions) is one muted line
/// as a list draws it; anything richer goes through
/// [`crate::widgets::empty::render_ctx`] (its action buttons registered).
pub fn draw_empty(area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, e: &EmptyState, bg: Color) {
    if e.kind == EmptyKind::Empty && e.hint.is_none() && e.actions.is_empty() {
        draw_line(area, buf, ctx.theme, &e.title, bg);
    } else {
        crate::widgets::empty::render_ctx(area, buf, ctx, e, bg);
    }
}

/// The content a States grid shows for `state`: loading for
/// [`SpecimenState::Loading`](crate::ui::specimen::SpecimenState::Loading),
/// a load error for [`SpecimenState::Error`](crate::ui::specimen::SpecimenState::Error),
/// ready otherwise. Every list-like specimen draws these two rows the same
/// way.
pub fn specimen(state: crate::ui::specimen::SpecimenState) -> ContentState {
    use crate::ui::specimen::SpecimenState;
    match state {
        SpecimenState::Loading => ContentState::loading(""),
        SpecimenState::Error => ContentState::error(EmptyState::new("Could not load")),
        _ => ContentState::Ready,
    }
}
