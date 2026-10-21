//! Scrolling inspector body beside an accounts list.
//!
//! The card header and the list stay with the split painter. This paints the
//! wrapped rows, the edge fade, and the scrollbar. Widths follow the panel
//! edges: a wide pass, then a narrower pass when the rows overflow.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Surface};
use crate::ui::Ui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InspectTone {
    Plain,
    Muted,
    Secondary,
    Warning,
    Danger,
    DangerBold,
    Info,
}

#[derive(Clone)]
pub struct InspectSpan {
    pub text: String,
    pub tone: InspectTone,
}

pub enum InspectLine {
    Prop {
        label: String,
        value: String,
        tone: InspectTone,
    },
    Spans {
        label: String,
        spans: Vec<InspectSpan>,
    },
    /// One paragraph. Words wrap on the body width, and a run of three spaces
    /// hangs the next line under the text that follows the gap.
    Text {
        text: String,
        tone: InspectTone,
    },
    /// Wrapped once on the panel width, then again on the body width.
    Prewrapped {
        text: String,
        tone: InspectTone,
    },
    Heading {
        label: String,
        meta: String,
    },
    Status {
        fail: bool,
        spans: Vec<InspectSpan>,
    },
    Blank,
}

const WARNING: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const DANGER: Color = Color::Rgb(0xf0, 0x5d, 0x5d);
const INFO: Color = Color::Rgb(0x79, 0xb8, 0xff);
const SEAM: Color = Color::Rgb(0x47, 0x47, 0x47);
const SEAM_HEAVY: Color = Color::Rgb(0x80, 0x80, 0x80);
const OUTER_KEEP: f32 = 0.55;
const INNER_KEEP: f32 = 0.8;
const LIST_W: u16 = 39;
const LABEL_MIN: u16 = 13;
const LABEL_MAX: u16 = 22;

struct Ink {
    chrome: Color,
    primary: Color,
    secondary: Color,
    muted: Color,
    faint: Color,
}

enum Row {
    Prop {
        label: String,
        value: String,
        tone: InspectTone,
    },
    Spans {
        label: String,
        spans: Vec<InspectSpan>,
    },
    Text {
        text: String,
        tone: InspectTone,
    },
    Heading {
        label: String,
        meta: String,
    },
    Status {
        fail: bool,
        spans: Vec<InspectSpan>,
    },
    Blank,
}

struct Body {
    x: u16,
    y: u16,
    full_w: u16,
    text_w: u16,
    height: u16,
    prewrap: u16,
}

struct PaintAt {
    x: u16,
    y: u16,
    text_w: u16,
    label_w: u16,
}

pub fn paint_inspector_body(ui: &mut Ui<'_>, area: Rect, lines: &[InspectLine]) {
    let Some(body) = body_geom(area) else {
        return;
    };
    let theme = ui.theme_ref();
    let ink = Ink {
        chrome: theme.bg(Surface::Surface),
        primary: step(theme, FgStep::Primary),
        secondary: step(theme, FgStep::Secondary),
        muted: step(theme, FgStep::Muted),
        faint: step(theme, FgStep::Faint),
    };
    let label_w = label_column(lines);
    let mut rows = layout(lines, body.full_w, body.prewrap, label_w);
    let mut text_w = body.full_w;
    if rows.len() > usize::from(body.height) {
        text_w = body.text_w;
        rows = layout(lines, text_w, body.prewrap, label_w);
    }
    let shown = body.height as usize;
    for (index, row) in rows.iter().take(shown).enumerate() {
        paint_row(
            ui,
            area,
            PaintAt {
                x: body.x,
                y: body.y.saturating_add(index as u16),
                text_w,
                label_w,
            },
            row,
            &ink,
        );
    }
    if rows.len() > shown {
        fade_edges(ui, body.x, body.y, body.full_w, body.height, &ink);
        paint_scrollbar(
            ui,
            area,
            body.x.saturating_add(body.full_w).saturating_sub(1),
            body.y,
            body.height,
            rows.len(),
            &ink,
        );
    }
}

fn body_geom(area: Rect) -> Option<Body> {
    if area.width < 80 || area.height < 24 {
        return None;
    }
    let list_right = area
        .x
        .saturating_add(1)
        .saturating_add(LIST_W)
        .saturating_sub(1);
    let panel_x = list_right.saturating_add(3);
    let panel_w = area.right().saturating_sub(panel_x).saturating_sub(1);
    let box_top = area.y.saturating_add(2);
    let box_bottom = area.bottom().saturating_sub(3);
    let panel_h = box_bottom.saturating_sub(box_top).saturating_add(1);
    let card_body = panel_h.saturating_sub(3);
    let height = card_body.saturating_sub(2);
    if panel_w <= 6 || height == 0 {
        return None;
    }
    Some(Body {
        x: panel_x.saturating_add(2),
        y: box_top.saturating_add(2),
        full_w: panel_w.saturating_sub(4),
        text_w: panel_w.saturating_sub(6),
        height,
        prewrap: panel_w.saturating_sub(6),
    })
}

fn label_column(lines: &[InspectLine]) -> u16 {
    let widest = lines
        .iter()
        .map(|line| match line {
            InspectLine::Prop { label, .. } | InspectLine::Spans { label, .. } => width(label),
            _ => 0,
        })
        .max()
        .unwrap_or(LABEL_MIN);
    widest.clamp(LABEL_MIN, LABEL_MAX).saturating_add(1)
}

fn layout(lines: &[InspectLine], text_w: u16, prewrap: u16, label_w: u16) -> Vec<Row> {
    let mut rows = Vec::new();
    for line in lines {
        match line {
            InspectLine::Prop { label, value, tone } => {
                let value_w = text_w.saturating_sub(label_w).max(8);
                for (index, part) in hang_wrap(value, value_w, false).into_iter().enumerate() {
                    rows.push(Row::Prop {
                        label: if index == 0 {
                            label.clone()
                        } else {
                            String::new()
                        },
                        value: part,
                        tone: *tone,
                    });
                }
            }
            InspectLine::Spans { label, spans } => {
                let value_w = text_w.saturating_sub(label_w).max(8);
                for (index, part) in wrap_spans(spans, value_w).into_iter().enumerate() {
                    rows.push(Row::Spans {
                        label: if index == 0 {
                            label.clone()
                        } else {
                            String::new()
                        },
                        spans: part,
                    });
                }
            }
            InspectLine::Text { text, tone } => {
                for part in hang_wrap(text, text_w, true) {
                    rows.push(Row::Text {
                        text: part,
                        tone: *tone,
                    });
                }
            }
            InspectLine::Prewrapped { text, tone } => {
                for piece in hang_wrap(text, prewrap, true) {
                    for part in hang_wrap(&piece, text_w, true) {
                        rows.push(Row::Text {
                            text: part,
                            tone: *tone,
                        });
                    }
                }
            }
            InspectLine::Heading { label, meta } => rows.push(Row::Heading {
                label: label.clone(),
                meta: meta.clone(),
            }),
            InspectLine::Status { fail, spans } => rows.push(Row::Status {
                fail: *fail,
                spans: spans.clone(),
            }),
            InspectLine::Blank => rows.push(Row::Blank),
        }
    }
    rows
}

fn paint_row(ui: &mut Ui<'_>, area: Rect, at: PaintAt, row: &Row, ink: &Ink) {
    let PaintAt {
        x,
        y,
        text_w,
        label_w,
    } = at;
    match row {
        Row::Prop { label, value, tone } => {
            put(ui, area, x, y, label, plain(ink.muted, ink.chrome));
            put(
                ui,
                area,
                x.saturating_add(label_w),
                y,
                value,
                plain(tone_color(*tone, ink), ink.chrome),
            );
        }
        Row::Spans { label, spans } => {
            put(ui, area, x, y, label, plain(ink.muted, ink.chrome));
            paint_spans(
                ui,
                area,
                x.saturating_add(label_w),
                y,
                text_w.saturating_sub(label_w),
                spans,
                ink,
            );
        }
        Row::Text { text, tone } => {
            put(
                ui,
                area,
                x,
                y,
                text,
                plain(tone_color(*tone, ink), ink.chrome),
            );
        }
        Row::Heading { label, meta } => {
            put(ui, area, x, y, label, bold(ink.secondary, ink.chrome));
            if meta.is_empty() {
                return;
            }
            let room = text_w.saturating_sub(width(label).saturating_add(3));
            if room < 8 {
                return;
            }
            let meta_w = width(meta);
            if meta_w == 0 || meta_w > room {
                return;
            }
            put(
                ui,
                area,
                x.saturating_add(text_w).saturating_sub(meta_w),
                y,
                meta,
                plain(ink.faint, ink.chrome),
            );
        }
        Row::Status { fail, spans } => {
            let (glyph, style) = if *fail {
                ("✗", bold(DANGER, ink.chrome))
            } else {
                ("▲", plain(WARNING, ink.chrome))
            };
            put(ui, area, x, y, glyph, style);
            paint_spans(
                ui,
                area,
                x.saturating_add(2),
                y,
                text_w.saturating_sub(2),
                spans,
                ink,
            );
        }
        Row::Blank => {}
    }
}

fn paint_spans(
    ui: &mut Ui<'_>,
    area: Rect,
    mut x: u16,
    y: u16,
    max: u16,
    spans: &[InspectSpan],
    ink: &Ink,
) {
    let end = x.saturating_add(max);
    for span in spans {
        if span.text.is_empty() || x >= end {
            continue;
        }
        let room = end.saturating_sub(x);
        let text = fit(&span.text, room);
        put(ui, area, x, y, &text, tone_style(span.tone, ink));
        x = x.saturating_add(width(&text));
    }
}

fn paint_scrollbar(
    ui: &mut Ui<'_>,
    area: Rect,
    x: u16,
    y: u16,
    height: u16,
    content: usize,
    ink: &Ink,
) {
    let track = usize::from(height);
    let len = thumb_len(track, content, track);
    for index in 0..track {
        let glyph = if index < len { "┃" } else { "┆" };
        let color = if index < len { SEAM_HEAVY } else { SEAM };
        put(
            ui,
            area,
            x,
            y.saturating_add(index as u16),
            glyph,
            plain(color, ink.chrome),
        );
    }
}

fn fade_edges(ui: &mut Ui<'_>, x: u16, y: u16, full_w: u16, height: u16, ink: &Ink) {
    if height < 4 {
        return;
    }
    let width = full_w.saturating_sub(1);
    let bottom = y.saturating_add(height).saturating_sub(1);
    fade_row(ui, x, bottom, width, OUTER_KEEP, ink.chrome);
    if height >= 12 {
        fade_row(
            ui,
            x,
            bottom.saturating_sub(1),
            width,
            INNER_KEEP,
            ink.chrome,
        );
    }
}

fn fade_row(ui: &mut Ui<'_>, x: u16, y: u16, width: u16, keep: f32, chrome: Color) {
    ui.recolor_fg(Rect::new(x, y, width, 1), |fg, bg| {
        if bg != chrome {
            return None;
        }
        let (Color::Rgb(red, green, blue), Color::Rgb(br, bgg, bb)) = (fg, bg) else {
            return None;
        };
        Some(Color::Rgb(
            mix(red, br, keep),
            mix(green, bgg, keep),
            mix(blue, bb, keep),
        ))
    });
}

fn mix(fg: u8, bg: u8, keep: f32) -> u8 {
    (f32::from(bg) + (f32::from(fg) - f32::from(bg)) * keep)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn thumb_len(viewport: usize, content: usize, track: usize) -> usize {
    if content <= viewport || track == 0 {
        return track;
    }
    ((viewport * track) / content).max(1).min(track)
}

fn hang_wrap(text: &str, limit: u16, hang: bool) -> Vec<String> {
    let limit = limit.max(1);
    let mut lines = Vec::new();
    for para in text.split('\n') {
        let body = para.trim_start_matches(' ');
        let lead = u16::try_from(para.len() - body.len()).unwrap_or(0);
        let mut first = if hang { lead } else { 0 };
        let mut cont = first;
        if hang && let Some(column) = hang_column(body, 3) {
            cont = first.saturating_add(column);
        }
        if first.saturating_mul(2) > limit {
            first = 0;
        }
        if cont.saturating_mul(2) > limit {
            cont = first;
        }
        wrap_para(body, limit, first, cont, &mut lines);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn hang_column(text: &str, gap: usize) -> Option<u16> {
    let at = text.find(&" ".repeat(gap))?;
    let rest = text[at..].trim_start_matches(' ');
    if rest.is_empty() {
        return None;
    }
    let prefix = text.len() - rest.len();
    Some(width(&text[..prefix]))
}

fn wrap_para(body: &str, limit: u16, first: u16, cont: u16, lines: &mut Vec<String>) {
    let mut line = " ".repeat(usize::from(first));
    let mut used = first;
    let mut base = first;
    for word in body.split(' ') {
        let word_w = width(word);
        if used == base {
            push_word(
                &mut line, &mut used, &mut base, word, word_w, limit, cont, lines,
            );
        } else if used.saturating_add(1).saturating_add(word_w) <= limit {
            line.push(' ');
            line.push_str(word);
            used = used.saturating_add(1).saturating_add(word_w);
        } else {
            break_line(&mut line, &mut used, &mut base, cont, lines);
            push_word(
                &mut line, &mut used, &mut base, word, word_w, limit, cont, lines,
            );
        }
    }
    lines.push(std::mem::take(&mut line));
}

fn break_line(
    line: &mut String,
    used: &mut u16,
    base: &mut u16,
    cont: u16,
    lines: &mut Vec<String>,
) {
    lines.push(std::mem::take(line));
    *line = " ".repeat(usize::from(cont));
    *used = cont;
    *base = cont;
}

#[allow(clippy::too_many_arguments)]
fn push_word(
    line: &mut String,
    used: &mut u16,
    base: &mut u16,
    word: &str,
    word_w: u16,
    limit: u16,
    cont: u16,
    lines: &mut Vec<String>,
) {
    if used.saturating_add(word_w) <= limit {
        line.push_str(word);
        *used = used.saturating_add(word_w);
        return;
    }
    for grapheme in word.chars() {
        let piece = grapheme.to_string();
        let piece_w = width(&piece).max(1);
        if used.saturating_add(piece_w) > limit && *used != *base {
            break_line(line, used, base, cont, lines);
        }
        line.push(grapheme);
        *used = used.saturating_add(piece_w.min(limit));
    }
}

fn wrap_spans(spans: &[InspectSpan], limit: u16) -> Vec<Vec<InspectSpan>> {
    let mut source = Vec::new();
    for (index, span) in spans.iter().enumerate() {
        for ch in span.text.chars() {
            source.push((ch, index));
        }
    }
    let text: String = source.iter().map(|(ch, _)| *ch).collect();
    let mut at = 0usize;
    let mut rows = Vec::new();
    for part in hang_wrap(&text, limit, false) {
        let mut pieces = Vec::new();
        for ch in part.chars() {
            // A break drops the space that did not fit. Skip those source
            // chars so the next span keeps its own ink.
            if let Some(found) = source[at..].iter().position(|(src, _)| *src == ch) {
                at = at.saturating_add(found);
            }
            let owner = source.get(at).map(|(_, index)| *index).unwrap_or(0);
            at = at.saturating_add(1);
            let tone = spans
                .get(owner)
                .map(|span| span.tone)
                .unwrap_or(InspectTone::Plain);
            match pieces.last_mut() {
                Some(InspectSpan { tone: prev, text }) if *prev == tone => text.push(ch),
                _ => pieces.push(InspectSpan {
                    text: ch.to_string(),
                    tone,
                }),
            }
        }
        rows.push(pieces);
    }
    if rows.is_empty() {
        rows.push(Vec::new());
    }
    rows
}

fn fit(text: &str, max: u16) -> String {
    if width(text) <= max {
        return text.to_owned();
    }
    let mut out = String::new();
    let mut used = 0u16;
    for ch in text.chars() {
        let piece = ch.to_string();
        let piece_w = width(&piece);
        if used.saturating_add(piece_w) > max {
            break;
        }
        out.push(ch);
        used = used.saturating_add(piece_w);
    }
    out
}

fn step(theme: &crate::theme::Theme, which: FgStep) -> Color {
    theme
        .color
        .fg
        .get(which.index())
        .copied()
        .unwrap_or(Color::Reset)
}

fn plain(fg: Color, bg: Color) -> PaintStyle {
    PaintStyle::new()
        .fg(fg)
        .bg(bg)
        .remove_modifier(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED | Modifier::DIM)
}

fn bold(fg: Color, bg: Color) -> PaintStyle {
    plain(fg, bg).add_modifier(Modifier::BOLD)
}

fn tone_color(tone: InspectTone, ink: &Ink) -> Color {
    match tone {
        InspectTone::Plain => ink.primary,
        InspectTone::Muted => ink.muted,
        InspectTone::Secondary => ink.secondary,
        InspectTone::Warning => WARNING,
        InspectTone::Danger | InspectTone::DangerBold => DANGER,
        InspectTone::Info => INFO,
    }
}

fn tone_style(tone: InspectTone, ink: &Ink) -> PaintStyle {
    let color = tone_color(tone, ink);
    if tone == InspectTone::DangerBold {
        bold(color, ink.chrome)
    } else {
        plain(color, ink.chrome)
    }
}

fn put(ui: &mut Ui<'_>, area: Rect, x: u16, y: u16, text: &str, style: PaintStyle) {
    let room = area.right().saturating_sub(x);
    if room == 0 || y >= area.bottom() || text.is_empty() {
        return;
    }
    ui.paint_str(Rect::new(x, y, room, 1), text, style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_hangs_under_the_value() {
        let lines = hang_wrap(
            "Anthropic · Session · 5-hour   2 accounts · 24\u{2013}62% remaining (1 last good)",
            71,
            true,
        );
        assert_eq!(
            lines,
            vec![
                "Anthropic · Session · 5-hour   2 accounts · 24\u{2013}62% remaining (1 last"
                    .to_owned(),
                "                               good)".to_owned(),
            ]
        );
    }

    #[test]
    fn health_value_wraps_before_the_scrollbar() {
        let text = "degraded · 4 warnings · 1 exhausted · 2 stale · 1 refreshing · 3 failed · 1 quota not visible · 2 identities unresolved";
        let lines = hang_wrap(text, 57, false);
        assert_eq!(
            lines,
            vec![
                "degraded · 4 warnings · 1 exhausted · 2 stale · 1".to_owned(),
                "refreshing · 3 failed · 1 quota not visible · 2".to_owned(),
                "identities unresolved".to_owned(),
            ]
        );
    }

    #[test]
    fn not_comparable_breaks_on_the_panel_width() {
        let text = "Amp (single account or different windows) · xAI (single account or different windows) · Z.AI (single account or different windows) · MiniMax (single account or different windows) · OpenCode (single account or different windows)";
        let lines = hang_wrap(text, 71, true);
        assert_eq!(lines.len(), 4);
        assert_eq!(
            lines[2],
            "MiniMax (single account or different windows) · OpenCode (single"
        );
        assert_eq!(lines[3], "account or different windows)");
    }
}
