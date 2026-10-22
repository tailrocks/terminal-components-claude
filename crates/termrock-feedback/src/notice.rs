use core::mem;
use std::borrow::Cow;

use ratatui_core::layout::Rect;

use crate::measure::{Constraints, Size};
use crate::text::measure::{grapheme_width, graphemes};
use crate::text::width;
use crate::theme::{FgStep, GlyphRole, Modifier, PaintStyle, Role, StylePatch};
use crate::ui::Ui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NoticeKindValue {
    Error,
    Warning,
    Info,
    Success,
}

/// One of the four supported notice meanings.
///
/// The representation is private so callers cannot construct an unsupported
/// kind. Use the named constants in builder calls and matches.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct NoticeKind(NoticeKindValue);

impl PartialEq for NoticeKind {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for NoticeKind {}

impl NoticeKind {
    /// Error feedback.
    pub const ERROR: Self = Self(NoticeKindValue::Error);
    /// Warning feedback.
    pub const WARNING: Self = Self(NoticeKindValue::Warning);
    /// Informational feedback.
    pub const INFO: Self = Self(NoticeKindValue::Info);
    /// Successful feedback.
    pub const SUCCESS: Self = Self(NoticeKindValue::Success);
}

impl Default for NoticeKind {
    fn default() -> Self {
        Self::ERROR
    }
}

#[derive(Clone, Copy, Debug)]
enum RowKind {
    Message,
    Detail,
    Hint,
}

struct NoticeRow<'a> {
    text: Cow<'a, str>,
    cut: bool,
    kind: RowKind,
    inset: u16,
}

struct WrappedLine {
    text: String,
    cut: bool,
}

/// A borrowed feedback notice with independent wrapping and style controls.
///
/// This is an additive borrowed API. The older Showcase notice was an owned
/// public struct and is not source-compatible with this widget. Cut rows keep
/// private presentation metadata; this component does not provide hover reveal.
#[derive(Clone, Copy, Debug)]
pub struct Notice<'a> {
    kind: NoticeKind,
    message: &'a str,
    detail: Option<&'a str>,
    hint: Option<&'a str>,
    message_lines: usize,
    max_lines: usize,
    detail_tone: FgStep,
    mark_cut: bool,
    marker: Option<&'a str>,
    marker_patch: StylePatch,
    message_patch: StylePatch,
    detail_patch: StylePatch,
    hint_patch: StylePatch,
}

impl<'a> Notice<'a> {
    /// Create an Error notice, borrowing its message text.
    #[must_use]
    pub const fn new(message: &'a str) -> Self {
        Self {
            kind: NoticeKind::ERROR,
            message,
            detail: None,
            hint: None,
            message_lines: 2,
            max_lines: 2,
            detail_tone: FgStep::Secondary,
            mark_cut: true,
            marker: None,
            marker_patch: StylePatch::new(),
            message_patch: StylePatch::new(),
            detail_patch: StylePatch::new(),
            hint_patch: StylePatch::new(),
        }
    }

    /// Create an Error notice.
    #[must_use]
    pub const fn error(message: &'a str) -> Self {
        Self::new(message)
    }

    /// Create a Warning notice.
    #[must_use]
    pub const fn warning(message: &'a str) -> Self {
        Self::new(message).kind(NoticeKind::WARNING)
    }

    /// Create an Info notice.
    #[must_use]
    pub const fn info(message: &'a str) -> Self {
        Self::new(message).kind(NoticeKind::INFO)
    }

    /// Create a Success notice.
    #[must_use]
    pub const fn success(message: &'a str) -> Self {
        Self::new(message).kind(NoticeKind::SUCCESS)
    }

    /// Select the notice's semantic kind.
    #[must_use]
    pub const fn kind(mut self, kind: NoticeKind) -> Self {
        self.kind = kind;
        self
    }

    /// Add a secondary detail paragraph.
    #[must_use]
    pub const fn detail(mut self, detail: &'a str) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Add a one-row supporting hint.
    #[must_use]
    pub const fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    /// Set the message row cap. Zero is normalized to one row.
    #[must_use]
    pub const fn message_lines(mut self, lines: usize) -> Self {
        self.message_lines = if lines == 0 { 1 } else { lines };
        self
    }

    /// Set the detail row cap. Zero suppresses the detail.
    #[must_use]
    pub const fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = lines;
        self
    }

    /// Set the semantic foreground step used for detail text.
    #[must_use]
    pub const fn detail_tone(mut self, tone: FgStep) -> Self {
        self.detail_tone = tone;
        self
    }

    /// Show or hide the visible truncation mark without changing cut tracking.
    #[must_use]
    pub const fn mark_cut(mut self, mark: bool) -> Self {
        self.mark_cut = mark;
        self
    }

    /// Override the marker glyph with borrowed text.
    #[must_use]
    pub const fn marker(mut self, marker: &'a str) -> Self {
        self.marker = Some(marker);
        self
    }

    /// Apply an explicit local patch to the marker style.
    #[must_use]
    pub const fn marker_patch(mut self, patch: StylePatch) -> Self {
        self.marker_patch = patch;
        self
    }

    /// Apply an explicit local patch to the message style.
    #[must_use]
    pub const fn message_patch(mut self, patch: StylePatch) -> Self {
        self.message_patch = patch;
        self
    }

    /// Apply an explicit local patch to the detail style.
    #[must_use]
    pub const fn detail_patch(mut self, patch: StylePatch) -> Self {
        self.detail_patch = patch;
        self
    }

    /// Apply an explicit local patch to the hint style.
    #[must_use]
    pub const fn hint_patch(mut self, patch: StylePatch) -> Self {
        self.hint_patch = patch;
        self
    }

    /// Return the preferred height for a specific available width.
    ///
    /// Stacks should use this method after choosing their column width so their
    /// next child starts after the same rows that `draw` uses.
    pub fn height_for_width(&self, ui: &Ui<'_>, width: u16) -> u16 {
        self.walk_rows(width, self.marker_inset(ui), |_| {})
    }

    /// Measure the notice within the supplied constraints.
    pub fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        let natural_width = self.natural_width(ui).min(constraints.max.0);
        let measured_width = if constraints.tight_w {
            constraints.max.0
        } else {
            natural_width
        };
        let measured_height = self.height_for_width(ui, measured_width);
        Size::exact(natural_width, measured_height).fit(constraints)
    }

    /// Draw only the notice's text. Blank cells and the caller's surface remain untouched.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }

        let marker = self.marker_text(ui);
        let marker_inset = self.marker_inset(ui);
        let marker_patch = self.marker_default_patch().merge(self.marker_patch);
        let message_patch = self.message_default_patch().merge(self.message_patch);
        let detail_patch = StylePatch::new()
            .set_fg(Role::Fg(self.detail_tone))
            .merge(self.detail_patch);
        let hint_patch = StylePatch::new()
            .set_fg(Role::Fg(FgStep::Muted))
            .merge(self.hint_patch);
        let styles = DrawStyles {
            marker: ui.paint_patch(&marker_patch),
            message: ui.paint_patch(&message_patch),
            detail: ui.paint_patch(&detail_patch),
            hint: ui.paint_patch(&hint_patch),
        };

        let mut row_index = 0u16;
        let mut pending: Option<(u16, NoticeRow<'a>)> = None;
        self.walk_rows(area.width, marker_inset, |row| {
            let index = row_index;
            row_index = row_index.saturating_add(1);
            if index < area.height {
                if let Some((old_index, old_row)) = pending.replace((index, row)) {
                    self.paint_row(ui, area, old_index, old_row, marker, styles);
                }
            } else if let Some((old_index, mut old_row)) = pending.take() {
                old_row.cut = true;
                self.paint_row(ui, area, old_index, old_row, marker, styles);
            }
        });

        if let Some((index, row)) = pending {
            self.paint_row(ui, area, index, row, marker, styles);
        }
    }

    fn marker_text(&self, ui: &Ui<'_>) -> &'a str {
        if let Some(marker) = self.marker {
            return marker;
        }
        match self.kind.0 {
            NoticeKindValue::Error => "!",
            NoticeKindValue::Warning => ui.theme_ref().design.glyphs.get(GlyphRole::WarningMark),
            NoticeKindValue::Info => "i",
            NoticeKindValue::Success => "✓",
        }
    }

    fn marker_default_patch(&self) -> StylePatch {
        match self.kind.0 {
            NoticeKindValue::Error => StylePatch::new().set_fg(Role::Danger).add(Modifier::BOLD),
            NoticeKindValue::Warning => StylePatch::new().set_fg(Role::Warning),
            NoticeKindValue::Info => StylePatch::new().set_fg(Role::Info),
            NoticeKindValue::Success => StylePatch::new().set_fg(Role::Success),
        }
    }

    fn message_default_patch(&self) -> StylePatch {
        let role = match self.kind.0 {
            NoticeKindValue::Error => Role::Danger,
            NoticeKindValue::Warning => Role::Warning,
            NoticeKindValue::Info | NoticeKindValue::Success => Role::Fg(FgStep::Primary),
        };
        StylePatch::new().set_fg(role)
    }

    fn marker_inset(&self, ui: &Ui<'_>) -> u16 {
        width(self.marker_text(ui)).saturating_add(1).max(2)
    }

    fn natural_width(&self, ui: &Ui<'_>) -> u16 {
        let message_inset = self.marker_inset(ui);
        let message_width = max_physical_line_width(self.message).saturating_add(message_inset);
        let detail_width = if self.max_lines > 0 {
            self.detail
                .map_or(0, |text| max_physical_line_width(text).saturating_add(2))
        } else {
            0
        };
        let hint_width = self.hint.map_or(0, |text| {
            width(text.split('\n').next().unwrap_or("")).saturating_add(2)
        });
        message_width.max(detail_width).max(hint_width)
    }

    fn walk_rows(
        &self,
        available_width: u16,
        message_inset: u16,
        mut emit: impl FnMut(NoticeRow<'a>),
    ) -> u16 {
        let mut row_count = 0u16;
        let message_width = available_width.saturating_sub(message_inset);
        let message_cap = self.message_lines.max(1).min(usize::from(u16::MAX));
        walk_wrapped(self.message, message_width, message_cap, |line| {
            if row_count < u16::MAX {
                let mut cut = line.cut;
                if message_width == 0 && !line.text.is_empty() {
                    cut = true;
                }
                emit(NoticeRow {
                    text: Cow::Owned(line.text),
                    cut,
                    kind: RowKind::Message,
                    inset: message_inset,
                });
                row_count = row_count.saturating_add(1);
            }
        });

        if let Some(detail) = self.detail
            && self.max_lines > 0
            && row_count < u16::MAX
        {
            let detail_width = available_width.saturating_sub(2);
            let cap = self.max_lines.min(usize::from(u16::MAX - row_count));
            walk_wrapped(detail, detail_width, cap, |line| {
                if row_count < u16::MAX {
                    let mut cut = line.cut;
                    if detail_width == 0 && !line.text.is_empty() {
                        cut = true;
                    }
                    emit(NoticeRow {
                        text: Cow::Owned(line.text),
                        cut,
                        kind: RowKind::Detail,
                        inset: 2,
                    });
                    row_count = row_count.saturating_add(1);
                }
            });
        }

        if let Some(hint) = self.hint
            && row_count < u16::MAX
        {
            let hint_width = available_width.saturating_sub(2);
            let first_line = hint.split('\n').next().unwrap_or("");
            let cut = width(first_line) > hint_width
                || first_line.len() != hint.len()
                || (hint_width == 0 && !first_line.is_empty());
            emit(NoticeRow {
                text: Cow::Borrowed(first_line),
                cut,
                kind: RowKind::Hint,
                inset: 2,
            });
            row_count = row_count.saturating_add(1);
        }

        row_count
    }

    fn paint_row(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        row_index: u16,
        row: NoticeRow<'a>,
        marker: &str,
        styles: DrawStyles,
    ) {
        let y = area.y.saturating_add(row_index);
        if row_index == 0 {
            ui.paint_str(Rect::new(area.x, y, area.width, 1), marker, styles.marker);
        }

        let row_width = area.width.saturating_sub(row.inset);
        if row_width == 0 {
            return;
        }
        let x = area.x.saturating_add(row.inset);
        let rect = Rect::new(x, y, row_width, 1);
        let style = match row.kind {
            RowKind::Message => styles.message,
            RowKind::Detail => styles.detail,
            RowKind::Hint => styles.hint,
        };

        if row.cut && self.mark_cut {
            let text = row.text.as_ref();
            let head = prefix_at_width(text, row_width.saturating_sub(1));
            ui.paint_str(rect, head, style);
            let head_width = width(head);
            let marker_width = row_width.saturating_sub(head_width);
            if marker_width > 0 {
                let marker_x = x.saturating_add(head_width);
                ui.paint_str(Rect::new(marker_x, y, marker_width, 1), "…", style);
            }
        } else {
            ui.paint_str(rect, row.text.as_ref(), style);
        }
    }
}

#[derive(Clone, Copy)]
struct DrawStyles {
    marker: PaintStyle,
    message: PaintStyle,
    detail: PaintStyle,
    hint: PaintStyle,
}

fn max_physical_line_width(text: &str) -> u16 {
    text.split('\n').map(width).max().unwrap_or(0)
}

fn walk_wrapped(
    text: &str,
    available_width: u16,
    cap: usize,
    emit: impl FnMut(WrappedLine),
) -> usize {
    let limit = cap.max(1).min(usize::from(u16::MAX));
    if available_width == 0 {
        return walk_unwrapped_lines(text, limit, emit);
    }
    let wrap_width = available_width.max(1);
    let mut state = BoundedWrap::new(limit, emit);

    for (paragraph_index, paragraph) in text.split('\n').enumerate() {
        if paragraph_index > 0 {
            if state.current_active && !state.commit() {
                break;
            }
            state.begin_line();
        }
        for word in paragraph.split(' ') {
            if !state.push_word(word, wrap_width) {
                break;
            }
        }
        if state.stopped {
            break;
        }
    }

    state.finish()
}

fn walk_unwrapped_lines(text: &str, limit: usize, mut emit: impl FnMut(WrappedLine)) -> usize {
    let mut pending: Option<WrappedLine> = None;
    let mut emitted = 0usize;
    for (seen, line) in text.split('\n').enumerate() {
        if seen >= limit {
            if let Some(mut last) = pending.take() {
                last.cut = true;
                emit(last);
                emitted = emitted.saturating_add(1);
            }
            return emitted;
        }
        if let Some(previous) = pending.replace(WrappedLine {
            text: String::new(),
            cut: !line.is_empty(),
        }) {
            emit(previous);
            emitted = emitted.saturating_add(1);
        }
    }
    if let Some(last) = pending {
        emit(last);
        emitted = emitted.saturating_add(1);
    }
    emitted
}

fn prefix_at_width(text: &str, max_width: u16) -> &str {
    let mut end = 0usize;
    let mut used = 0u16;
    for (index, grapheme) in graphemes(text) {
        let next = used.saturating_add(grapheme_width(grapheme));
        if next > max_width {
            break;
        }
        end = index.saturating_add(grapheme.len());
        used = next;
    }
    text.get(..end).unwrap_or("")
}

struct BoundedWrap<F> {
    limit: usize,
    seen: usize,
    emitted: usize,
    current: String,
    current_width: u16,
    current_cut: bool,
    current_active: bool,
    pending: Option<WrappedLine>,
    stopped: bool,
    emit: F,
}

impl<F> BoundedWrap<F>
where
    F: FnMut(WrappedLine),
{
    fn new(limit: usize, emit: F) -> Self {
        Self {
            limit,
            seen: 0,
            emitted: 0,
            current: String::new(),
            current_width: 0,
            current_cut: false,
            current_active: true,
            pending: None,
            stopped: false,
            emit,
        }
    }

    fn commit(&mut self) -> bool {
        let current = WrappedLine {
            text: mem::take(&mut self.current),
            cut: mem::replace(&mut self.current_cut, false),
        };
        self.current_width = 0;
        self.current_active = false;

        if self.seen >= self.limit {
            self.stopped = true;
            if let Some(mut last) = self.pending.take() {
                last.cut = true;
                (self.emit)(last);
                self.emitted = self.emitted.saturating_add(1);
            }
            return false;
        }

        if let Some(previous) = self.pending.take() {
            (self.emit)(previous);
            self.emitted = self.emitted.saturating_add(1);
        }
        self.pending = Some(current);
        self.seen += 1;
        true
    }

    fn push_word(&mut self, word: &str, wrap_width: u16) -> bool {
        let word_width = width(word);
        if self.current_width == 0 {
            if word_width <= wrap_width {
                self.append(word, word_width);
                return true;
            }
            return self.push_hard_wrapped(word, wrap_width);
        }

        if self
            .current_width
            .saturating_add(1)
            .saturating_add(word_width)
            <= wrap_width
        {
            self.current.push(' ');
            self.current_width = self.current_width.saturating_add(1);
            if word_width <= wrap_width {
                self.append(word, word_width);
                return true;
            }
            return self.push_hard_wrapped(word, wrap_width);
        }

        if !self.commit() {
            return false;
        }
        if word_width <= wrap_width {
            self.append(word, word_width);
            true
        } else {
            self.push_hard_wrapped(word, wrap_width)
        }
    }

    fn push_hard_wrapped(&mut self, word: &str, wrap_width: u16) -> bool {
        for (_, grapheme) in graphemes(word) {
            let grapheme_width = grapheme_width(grapheme);
            if grapheme_width > wrap_width {
                if self.current_width > 0 && !self.commit() {
                    return false;
                }
                self.current_cut = true;
                self.current_width = wrap_width;
                self.current_active = true;
                if !self.commit() {
                    return false;
                }
                continue;
            }
            if self.current_width.saturating_add(grapheme_width) > wrap_width && !self.commit() {
                return false;
            }
            self.append(grapheme, grapheme_width);
        }
        true
    }

    fn append(&mut self, text: &str, text_width: u16) {
        self.current_active = true;
        for (_, grapheme) in graphemes(text) {
            if grapheme_width(grapheme) > 0 {
                self.current.push_str(grapheme);
            }
        }
        self.current_width = self.current_width.saturating_add(text_width);
    }

    fn begin_line(&mut self) {
        self.current_active = true;
    }

    fn finish(mut self) -> usize {
        if !self.stopped && self.current_active {
            self.commit();
        }
        if !self.stopped
            && let Some(last) = self.pending.take()
        {
            (self.emit)(last);
            self.emitted = self.emitted.saturating_add(1);
        }
        self.emitted
    }
}

#[cfg(test)]
mod tests {
    use super::{Notice, RowKind};

    #[test]
    fn long_hint_rows_retain_the_borrowed_text_and_cut_state() {
        let controls = "\0".repeat(65_536);
        let visible = format!("{controls}visible");
        for hint in [controls.as_str(), visible.as_str()] {
            for width in [0, 1, 2, 6, 10] {
                let notice = Notice::new("x").hint(hint);
                let mut retained = None;
                let rows = notice.walk_rows(width, 2, |row| {
                    if matches!(row.kind, RowKind::Hint) {
                        retained = Some((row.text.as_ptr(), row.text.len(), row.cut));
                    }
                });
                let (_, retained_len, cut) = retained.expect("the hint produces a row");
                assert_eq!(rows, 2);
                assert_eq!(retained_len, hint.len(), "width={width}");
                assert!(
                    retained.is_some_and(|(pointer, _, _)| pointer == hint.as_ptr()),
                    "the hint row copied input at width={width}"
                );
                let expected_cut = if hint == controls {
                    width <= 2
                } else {
                    width <= 6
                };
                assert_eq!(cut, expected_cut, "width={width}");
            }
        }
    }
}
