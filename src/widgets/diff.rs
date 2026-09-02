//! Diff viewer: a unified-diff data model and a viewer that renders one file
//! in two presentations — the classic unified `git diff` listing, and a
//! review layout with old and new columns side by side. Scrolling, wheel,
//! scrollbar, drag selection and copy come from the [`TextViewport`] it
//! wraps; the viewer only turns hunks into styled lines.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::theme::Tone;
use crate::ui::ctx::RenderCtx;
use crate::ui::text::{fit, truncate, width};
use crate::widgets::viewport::{Line, Span, TextViewport, ViewportEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Add,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub text: String,
}

impl DiffLine {
    pub fn context(text: impl Into<String>) -> Self {
        Self {
            kind: DiffLineKind::Context,
            text: text.into(),
        }
    }
    pub fn add(text: impl Into<String>) -> Self {
        Self {
            kind: DiffLineKind::Add,
            text: text.into(),
        }
    }
    pub fn remove(text: impl Into<String>) -> Self {
        Self {
            kind: DiffLineKind::Remove,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    pub old_start: usize,
    pub new_start: usize,
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    /// Number of lines on the old side (context + removes).
    pub fn old_len(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| l.kind != DiffLineKind::Add)
            .count()
    }
    /// Number of lines on the new side (context + adds).
    pub fn new_len(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| l.kind != DiffLineKind::Remove)
            .count()
    }
    pub fn header(&self) -> String {
        format!(
            "@@ -{},{} +{},{} @@",
            self.old_start,
            self.old_len(),
            self.new_start,
            self.new_len()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffStatus {
    Added,
    Modified,
    Deleted,
    Renamed { from: String },
}

impl DiffStatus {
    /// One-letter marker: `A M D R`.
    pub fn marker(&self) -> &'static str {
        match self {
            DiffStatus::Added => "A",
            DiffStatus::Modified => "M",
            DiffStatus::Deleted => "D",
            DiffStatus::Renamed { .. } => "R",
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            DiffStatus::Added => "added",
            DiffStatus::Modified => "modified",
            DiffStatus::Deleted => "deleted",
            DiffStatus::Renamed { .. } => "renamed",
        }
    }
    /// Tone of the marker: additions succeed, deletions err, the rest is
    /// ordinary text.
    pub fn tone(&self) -> Tone {
        match self {
            DiffStatus::Added => Tone::Success,
            DiffStatus::Deleted => Tone::Error,
            DiffStatus::Modified => Tone::Warning,
            DiffStatus::Renamed { .. } => Tone::Secondary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffFile {
    pub path: String,
    pub status: DiffStatus,
    pub hunks: Vec<DiffHunk>,
}

impl DiffFile {
    pub fn additions(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|h| h.lines.iter())
            .filter(|l| l.kind == DiffLineKind::Add)
            .count()
    }
    pub fn deletions(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|h| h.lines.iter())
            .filter(|l| l.kind == DiffLineKind::Remove)
            .count()
    }
    /// `+12 −4 · 3 hunks`
    pub fn summary(&self) -> String {
        let n = self.hunks.len();
        format!(
            "+{} −{} · {} hunk{}",
            self.additions(),
            self.deletions(),
            n,
            if n == 1 { "" } else { "s" }
        )
    }
    /// The header line of the diff: `M path  +12 −4 · 3 hunks`.
    pub fn header(&self) -> String {
        match &self.status {
            DiffStatus::Renamed { from } => {
                format!("{} {from} → {}", self.status.marker(), self.path)
            }
            _ => format!("{} {}", self.status.marker(), self.path),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffMode {
    /// Classic unified listing: gutter with old/new numbers, `+`/`-` lines.
    #[default]
    Unified,
    /// Review layout: old and new columns side by side, changed runs bold.
    Review,
}

impl DiffMode {
    pub fn label(self) -> &'static str {
        match self {
            DiffMode::Unified => "unified",
            DiffMode::Review => "review",
        }
    }
    pub fn toggled(self) -> Self {
        match self {
            DiffMode::Unified => DiffMode::Review,
            DiffMode::Review => DiffMode::Unified,
        }
    }
}

/// Renders one [`DiffFile`] into a [`TextViewport`].
pub struct DiffView {
    pub term: TextViewport,
    pub mode: DiffMode,
    file: Option<DiffFile>,
    laid_width: u16,
    dirty: bool,
}

impl DiffView {
    pub fn new(id: WidgetId) -> Self {
        Self {
            term: TextViewport::new(id),
            mode: DiffMode::Unified,
            file: None,
            laid_width: 0,
            dirty: true,
        }
    }

    pub fn id(&self) -> WidgetId {
        self.term.id
    }

    pub fn file(&self) -> Option<&DiffFile> {
        self.file.as_ref()
    }

    pub fn set_file(&mut self, file: Option<DiffFile>) {
        if self.file != file {
            self.file = file;
            self.dirty = true;
            self.term.scroll.jump_start();
            self.term.set_follow(false);
        }
    }

    pub fn set_mode(&mut self, mode: DiffMode) {
        if self.mode != mode {
            self.mode = mode;
            self.dirty = true;
        }
    }

    pub fn toggle_mode(&mut self) -> DiffMode {
        self.set_mode(self.mode.toggled());
        self.mode
    }

    /// Rebuild the viewport lines for `width` cells when something changed.
    pub fn layout(&mut self, width: u16) {
        if !self.dirty && self.laid_width == width {
            return;
        }
        let lines = match &self.file {
            Some(f) => match self.mode {
                DiffMode::Unified => unified_lines(f),
                DiffMode::Review => review_lines(f, width),
            },
            None => vec![vec![Span::muted("No file selected")]],
        };
        let offset = self.term.scroll.offset;
        self.term.set_lines(lines);
        self.term.set_follow(false);
        self.term.scroll.scroll_to(offset);
        self.laid_width = width;
        self.dirty = false;
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        self.term.owns(id)
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<ViewportEvent>) {
        self.term.on_key(key)
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.term.on_wheel(delta)
    }

    pub fn on_click(&mut self, pos: Position) -> Outcome {
        self.term.on_click(pos)
    }

    pub fn on_drag(&mut self, pos: Position) -> Outcome {
        self.term.on_drag(pos)
    }

    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        self.term.on_scrollbar(pos)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.layout(area.width);
        self.term.render(area, buf, ctx, bg);
    }
}

fn gutter(n: Option<usize>, w: usize) -> Span {
    let text = match n {
        Some(n) => format!("{n:>w$}"),
        None => " ".repeat(w),
    };
    Span::new(text, Tone::Faint)
}

fn num_width(f: &DiffFile) -> usize {
    let max = f
        .hunks
        .iter()
        .map(|h| (h.old_start + h.old_len()).max(h.new_start + h.new_len()))
        .max()
        .unwrap_or(1);
    max.to_string().len().max(3)
}

/// Unified listing: header, then per hunk a muted `@@` line and one row per
/// diff line with old/new numbers in the gutter.
pub fn unified_lines(f: &DiffFile) -> Vec<Line> {
    let nw = num_width(f);
    let mut out: Vec<Line> = vec![];
    out.push(vec![
        Span::new(f.status.marker(), f.status.tone()).bold(),
        Span::plain(" "),
        Span::plain(f.header()[2..].to_owned()).bold(),
        Span::muted(format!("  {}", f.summary())),
    ]);
    for h in &f.hunks {
        out.push(vec![Span::muted(h.header())]);
        let mut old = h.old_start;
        let mut new = h.new_start;
        for l in &h.lines {
            let (o, n, marker, tone) = match l.kind {
                DiffLineKind::Context => {
                    let r = (Some(old), Some(new), " ", Tone::Secondary);
                    old += 1;
                    new += 1;
                    r
                }
                DiffLineKind::Add => {
                    let r = (None, Some(new), "+", Tone::Success);
                    new += 1;
                    r
                }
                DiffLineKind::Remove => {
                    let r = (Some(old), None, "-", Tone::Error);
                    old += 1;
                    r
                }
            };
            out.push(vec![
                gutter(o, nw),
                Span::plain(" "),
                gutter(n, nw),
                Span::plain(" "),
                Span::new(marker, tone).bold(),
                Span::new(format!(" {}", l.text), tone),
            ]);
        }
    }
    if f.hunks.is_empty() {
        out.push(vec![Span::muted("(no textual changes)")]);
    }
    out
}

/// Longest common prefix and suffix (in chars) of two strings, so the
/// differing middle can be emphasised.
fn changed_range(a: &str, b: &str) -> (usize, usize) {
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let mut p = 0;
    while p < ac.len() && p < bc.len() && ac[p] == bc[p] {
        p += 1;
    }
    let mut s = 0;
    while s < ac.len() - p && s < bc.len() - p && ac[ac.len() - 1 - s] == bc[bc.len() - 1 - s] {
        s += 1;
    }
    (p, s)
}

/// Text with the middle `[p .. len-s]` in bold.
fn emphasised(text: &str, p: usize, s: usize, tone: Tone, col_w: usize) -> Vec<Span> {
    let chars: Vec<char> = text.chars().collect();
    let end = chars.len().saturating_sub(s).max(p);
    let head: String = chars[..p.min(chars.len())].iter().collect();
    let mid: String = chars[p.min(chars.len())..end].iter().collect();
    let tail: String = chars[end..].iter().collect();
    // clip to the column, keeping the emphasis where it falls
    let mut spans = vec![];
    let mut used = 0usize;
    for (text, bold) in [(head, false), (mid, true), (tail, false)] {
        if used >= col_w {
            break;
        }
        let t = truncate(&text, col_w - used);
        used += width(&t);
        if !t.is_empty() {
            let sp = Span::new(t, tone);
            spans.push(if bold { sp.bold() } else { sp });
        }
    }
    if used < col_w {
        spans.push(Span::plain(" ".repeat(col_w - used)));
    }
    spans
}

/// Review layout: `old │ new` columns with line numbers; paired changes get
/// their differing run in bold; unpaired sides stay blank.
pub fn review_lines(f: &DiffFile, width: u16) -> Vec<Line> {
    let nw = num_width(f);
    let total = width as usize;
    // each column: number + space + text; separator " │ "
    let col = total.saturating_sub(3) / 2;
    let text_w = col.saturating_sub(nw + 1).max(4);
    let mut out: Vec<Line> = vec![];
    out.push(vec![
        Span::new(f.status.marker(), f.status.tone()).bold(),
        Span::plain(" "),
        Span::plain(f.header()[2..].to_owned()).bold(),
        Span::muted(format!("  {} · review", f.summary())),
    ]);
    let sep = || Span::new(" │ ", Tone::Faint);
    let blank_col =
        |nw: usize, text_w: usize| -> Vec<Span> { vec![Span::plain(" ".repeat(nw + 1 + text_w))] };
    let side = |n: usize, text: &str, tone: Tone, bold: Option<(usize, usize)>| -> Vec<Span> {
        let mut v = vec![Span::new(format!("{n:>nw$} "), Tone::Faint)];
        match bold {
            Some((p, s)) => v.extend(emphasised(text, p, s, tone, text_w)),
            None => v.push(Span::new(fit(&truncate(text, text_w), text_w), tone)),
        }
        v
    };
    for (hi, h) in f.hunks.iter().enumerate() {
        if hi > 0 {
            out.push(vec![Span::new("─".repeat(total.min(200)), Tone::Faint)]);
        }
        out.push(vec![Span::muted(h.header())]);
        let mut old = h.old_start;
        let mut new = h.new_start;
        let mut i = 0;
        while i < h.lines.len() {
            match h.lines[i].kind {
                DiffLineKind::Context => {
                    let mut row = side(old, &h.lines[i].text, Tone::Secondary, None);
                    row.push(sep());
                    row.extend(side(new, &h.lines[i].text, Tone::Secondary, None));
                    out.push(row);
                    old += 1;
                    new += 1;
                    i += 1;
                }
                _ => {
                    // a run of removes followed by a run of adds pairs up
                    let mut removes = vec![];
                    while i < h.lines.len() && h.lines[i].kind == DiffLineKind::Remove {
                        removes.push(h.lines[i].text.clone());
                        i += 1;
                    }
                    let mut adds = vec![];
                    while i < h.lines.len() && h.lines[i].kind == DiffLineKind::Add {
                        adds.push(h.lines[i].text.clone());
                        i += 1;
                    }
                    let n = removes.len().max(adds.len());
                    for k in 0..n {
                        let mut row: Vec<Span> = vec![];
                        match (removes.get(k), adds.get(k)) {
                            (Some(r), Some(a)) => {
                                let (p, s) = changed_range(r, a);
                                row.extend(side(old, r, Tone::Error, Some((p, s))));
                                row.push(sep());
                                row.extend(side(new, a, Tone::Success, Some((p, s))));
                                old += 1;
                                new += 1;
                            }
                            (Some(r), None) => {
                                row.extend(side(old, r, Tone::Error, None));
                                row.push(sep());
                                row.extend(blank_col(nw, text_w));
                                old += 1;
                            }
                            (None, Some(a)) => {
                                row.extend(blank_col(nw, text_w));
                                row.push(sep());
                                row.extend(side(new, a, Tone::Success, None));
                                new += 1;
                            }
                            (None, None) => {}
                        }
                        out.push(row);
                    }
                }
            }
        }
    }
    if f.hunks.is_empty() {
        out.push(vec![Span::muted("(no textual changes)")]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use crate::widgets::viewport::line_text;

    fn sample() -> DiffFile {
        DiffFile {
            path: "src/settlement/retry.rs".into(),
            status: DiffStatus::Modified,
            hunks: vec![
                DiffHunk {
                    old_start: 10,
                    new_start: 10,
                    lines: vec![
                        DiffLine::context("fn retry() {"),
                        DiffLine::remove("    let attempts = 3;"),
                        DiffLine::add("    let attempts = 5;"),
                        DiffLine::add("    let backoff = Backoff::exponential();"),
                        DiffLine::context("}"),
                    ],
                },
                DiffHunk {
                    old_start: 40,
                    new_start: 41,
                    lines: vec![DiffLine::remove("// old"), DiffLine::context("done")],
                },
            ],
        }
    }

    #[test]
    fn counts_and_headers() {
        let f = sample();
        assert_eq!(f.additions(), 2);
        assert_eq!(f.deletions(), 2);
        assert_eq!(f.summary(), "+2 −2 · 2 hunks");
        assert_eq!(f.hunks[0].header(), "@@ -10,3 +10,4 @@");
        assert_eq!(f.header(), "M src/settlement/retry.rs");
    }

    #[test]
    fn unified_lists_every_line_with_markers() {
        let lines = unified_lines(&sample());
        let texts: Vec<String> = lines.iter().map(|l| line_text(l)).collect();
        assert!(texts[0].starts_with("M src/settlement/retry.rs"));
        assert!(texts[1].starts_with("@@ -10,3 +10,4 @@"));
        assert!(texts[3].contains("- ") && texts[3].contains("attempts = 3"));
        assert!(texts[4].contains("+ ") && texts[4].contains("attempts = 5"));
        assert!(texts[2].starts_with(" 10  10"));
        assert_eq!(lines[3].iter().filter(|s| s.tone == Tone::Error).count(), 2);
    }

    #[test]
    fn review_pairs_columns_and_emphasises_the_change() {
        let lines = review_lines(&sample(), 80);
        let texts: Vec<String> = lines.iter().map(|l| line_text(l)).collect();
        let paired = &texts[3];
        assert!(paired.contains("│"));
        assert!(paired.contains("attempts = 3") && paired.contains("attempts = 5"));
        let bold: Vec<&Span> = lines[3].iter().filter(|s| s.bold).collect();
        assert!(bold.iter().any(|s| s.text.contains('3')));
        assert!(bold.iter().any(|s| s.text.contains('5')));
        // the unpaired add leaves the old column blank
        assert!(texts[4].trim_start().starts_with("│") || texts[4].starts_with("     "));
        assert!(texts[4].contains("Backoff::exponential"));
        // hunk separator between hunks
        assert!(texts.iter().any(|t| t.starts_with("────")));
    }

    #[test]
    fn view_renders_and_scrolls() {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut v = DiffView::new(WidgetId::of("t.diff"));
        v.set_file(Some(sample()));
        let area = Rect::new(0, 0, 60, 4);
        let mut buf = Buffer::empty(area);
        v.render(area, &mut buf, &mut ctx, theme.canvas);
        let row0: String = (0..60).map(|x| buf[(x, 0)].symbol().to_owned()).collect();
        assert!(row0.starts_with("M src/settlement/retry.rs"));
        assert_eq!(v.on_wheel(2), Outcome::Changed);
        assert_eq!(v.term.scroll.offset, 2);
        v.render(area, &mut buf, &mut ctx, theme.canvas);
        assert_eq!(v.term.scroll.offset, 2, "render must not undo the wheel");
        v.toggle_mode();
        assert_eq!(v.mode, DiffMode::Review);
        v.render(area, &mut buf, &mut ctx, theme.canvas);
        let row0: String = (0..60).map(|x| buf[(x, 0)].symbol().to_owned()).collect();
        assert!(row0.contains("review"));
    }
}
