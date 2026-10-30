use termrock::{Color, Cx, Modifier, PaintStyle, Rect, Response, Ui, width, wrap};

use showcase_data::{PROSE, log_lines};

use super::{Page, PageUpdate, frame};

const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const COL_MIN: u16 = 36;
const CARD_PAD: u16 = 2;
const NOTE_X: u16 = 32;
const STACK_GAP: u16 = 1;
const PANE_GAP: u16 = 2;

const TITLED: &str = "A card is a filled surface. Its title sits in the top-left and metadata on the right. It never has a border.";
const UNTITLED: &str = "Untitled card. Same surface, content starts at the padding edge.";
const NESTED_NOTE: &str =
    "A group inside a card is a muted label plus indent. The focus bar stays on the control.";

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const ACCENT: u32 = 0x48e054;
const TAB_ACTIVE: u32 = 0x27272a;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;
const DISABLED: u32 = 0x4d4d4d;
const WARNING: u32 = 0xf59e09;
const ERROR: u32 = 0xf05d5d;

const FADE_OUTER: f32 = 0.55;
const FADE_INNER: f32 = 0.8;
const FADE_DEEP: u16 = 12;
const FADE_MIN: u16 = 4;

const TARGETS: &[(&str, bool)] = &[("Local", false), ("CLI", false), ("Cloud", true)];

fn rgb(hex: u32) -> Color {
    Color::Rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

fn ink(fg: u32, bg: u32) -> PaintStyle {
    PaintStyle::new().fg(rgb(fg)).bg(rgb(bg))
}

fn mix_channel(fg: u8, bg: u8, keep: f32) -> u8 {
    (f32::from(bg) + (f32::from(fg) - f32::from(bg)) * keep)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn mix(fg: u32, bg: u32, keep: f32) -> u32 {
    let channel = |shift: u32| {
        u32::from(mix_channel(
            ((fg >> shift) & 0xff) as u8,
            ((bg >> shift) & 0xff) as u8,
            keep,
        ))
    };
    (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

fn tab_width(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn paint_blurb(ui: &mut Ui<'_>, body: Rect) {
    let blurb = "Cards group; a frame only where a pane needs an edge; nothing boxed twice";
    let title_w = width("Panels");
    if body.width <= title_w.saturating_add(4) {
        return;
    }
    let meta = Rect::new(
        body.x.saturating_add(title_w).saturating_add(2),
        body.y.saturating_sub(2),
        body.width.saturating_sub(title_w).saturating_sub(3),
        1,
    );
    let style = ink(MUTED, CANVAS);
    if width(blurb) <= meta.width {
        let _ = ui.paint_str(meta, blurb, style);
        return;
    }
    let budget = Rect {
        width: meta.width.saturating_sub(1),
        ..meta
    };
    let used = ui.paint_str(budget, blurb, style);
    let _ = ui.paint_str(
        Rect::new(meta.x.saturating_add(used), meta.y, 1, 1),
        "…",
        style,
    );
}

fn paint_section_strip(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let mut x = area.x;
    let mut tab_end = area.x;
    for (index, label) in SECTIONS.iter().enumerate() {
        let w = tab_width(label);
        if x.saturating_add(w) > area.right() {
            break;
        }
        let style = if index == 0 {
            ink(WHITE, TAB_ACTIVE).add_modifier(Modifier::BOLD)
        } else {
            ink(SECONDARY, CANVAS)
        };
        ui.fill(Rect::new(x, area.y, w, 1), style);
        let _ = ui.paint_str(
            Rect::new(x.saturating_add(1), area.y, w.saturating_sub(1), 1),
            label,
            style,
        );
        tab_end = x.saturating_add(w);
        x = tab_end.saturating_add(1);
    }
    if area.width >= SECTION_WIDE {
        let caption_width = width(SECTION_CAPTION);
        let caption_x = area.right().saturating_sub(caption_width.saturating_add(1));
        if caption_x > tab_end.saturating_add(2) {
            let _ = ui.paint_str(
                Rect::new(caption_x, area.y, caption_width, 1),
                SECTION_CAPTION,
                ink(FAINT, CANVAS),
            );
        }
    }
    if area.height < 2 {
        return;
    }
    let rule_y = area.y.saturating_add(1);
    let _ = ui.paint_str(
        Rect {
            y: rule_y,
            height: 1,
            ..area
        },
        &"─".repeat(usize::from(area.width)),
        ink(RULE, CANVAS),
    );
    let active = tab_width(SECTIONS[0]).min(area.width);
    let _ = ui.paint_str(
        Rect::new(area.x, rule_y, active, 1),
        &"━".repeat(usize::from(active)),
        ink(ACCENT, CANVAS),
    );
}

fn split_columns(area: Rect) -> Option<(Rect, Rect)> {
    let half = (area.width / 2).saturating_sub(1);
    if half < COL_MIN || area.width < half.saturating_add(PANE_GAP).saturating_add(20) {
        return None;
    }
    let left = Rect::new(area.x, area.y, half, area.height);
    let right = Rect::new(
        area.x.saturating_add(half).saturating_add(PANE_GAP),
        area.y,
        area.width.saturating_sub(half).saturating_sub(PANE_GAP),
        area.height,
    );
    Some((left, right))
}

fn stack_slots(area: Rect, heights: &[u16]) -> Vec<Rect> {
    let mut y = area.y;
    let mut slots = Vec::with_capacity(heights.len());
    for (index, height) in heights.iter().copied().enumerate() {
        let height = height.min(area.bottom().saturating_sub(y));
        slots.push(Rect::new(area.x, y, area.width, height));
        y = y.saturating_add(height);
        if index.saturating_add(1) != heights.len() {
            y = y.saturating_add(STACK_GAP).min(area.bottom());
        }
    }
    slots
}

fn card_inner_width(outer: u16) -> u16 {
    outer.saturating_sub(CARD_PAD.saturating_mul(2))
}

fn card_heights(inner_w: u16) -> [u16; 3] {
    let lines = |text: &str| u16::try_from(wrap(text, inner_w).len()).unwrap_or(u16::MAX);
    let note_lines = lines(NESTED_NOTE);
    let nested_body = if inner_w > NOTE_X.saturating_add(20) {
        note_lines.max(4)
    } else {
        note_lines.saturating_add(5)
    };
    [
        lines(TITLED).saturating_add(3).max(7),
        lines(UNTITLED).saturating_add(2).max(6),
        nested_body.saturating_add(3).max(7),
    ]
}

fn paint_rule(ui: &mut Ui<'_>, area: Rect) {
    if area.width <= CARD_PAD.saturating_mul(2) || area.height < 2 {
        return;
    }
    let span = card_inner_width(area.width);
    let _ = ui.paint_str(
        Rect::new(
            area.x.saturating_add(CARD_PAD),
            area.y.saturating_add(1),
            span,
            1,
        ),
        &"─".repeat(usize::from(span)),
        ink(RULE, SURFACE),
    );
}

fn paint_card_title(ui: &mut Ui<'_>, area: Rect, title: &str, meta: &str) {
    let x = area.x.saturating_add(CARD_PAD);
    let span = card_inner_width(area.width);
    let _ = ui.paint_str(
        Rect::new(x, area.y, span, 1),
        title,
        ink(SECONDARY, SURFACE),
    );
    paint_rule(ui, area);
    if meta.is_empty() || span <= width(meta) {
        return;
    }
    let meta_w = width(meta);
    let _ = ui.paint_str(
        Rect::new(
            x.saturating_add(span).saturating_sub(meta_w),
            area.y,
            meta_w,
            1,
        ),
        meta,
        ink(FAINT, SURFACE),
    );
}

fn paint_wrapped(ui: &mut Ui<'_>, area: Rect, text: &str, fg: u32, bg: u32) {
    if area.is_empty() {
        return;
    }
    for (index, line) in wrap(text, area.width).into_iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        if offset >= area.height {
            break;
        }
        let _ = ui.paint_str(
            Rect::new(area.x, area.y.saturating_add(offset), area.width, 1),
            &line,
            ink(fg, bg),
        );
    }
}

fn card_body(area: Rect, titled: bool) -> Rect {
    let inner = Rect::new(
        area.x.saturating_add(CARD_PAD),
        area.y.saturating_add(1),
        card_inner_width(area.width),
        area.height.saturating_sub(2),
    );
    if titled {
        Rect::new(
            inner.x,
            inner.y.saturating_add(1),
            inner.width,
            inner.height.saturating_sub(1),
        )
    } else {
        inner
    }
}

fn paint_titled(ui: &mut Ui<'_>, area: Rect) {
    ui.fill(area, ink(WHITE, SURFACE));
    paint_card_title(ui, area, "Titled card", "surface");
    paint_wrapped(ui, card_body(area, true), TITLED, SECONDARY, SURFACE);
}

fn paint_untitled(ui: &mut Ui<'_>, area: Rect) {
    ui.fill(area, ink(WHITE, SURFACE));
    paint_wrapped(ui, card_body(area, false), UNTITLED, SECONDARY, SURFACE);
}

fn paint_targets(ui: &mut Ui<'_>, area: Rect) {
    let width = area.width.min(NOTE_X.saturating_sub(2));
    let height = area
        .height
        .saturating_sub(1)
        .min(u16::try_from(TARGETS.len()).unwrap_or(0));
    let group = Rect::new(area.x, area.y.saturating_add(1), width, height);
    for (index, (label, disabled)) in TARGETS.iter().copied().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        if offset >= group.height {
            break;
        }
        let row = Rect::new(group.x, group.y.saturating_add(offset), group.width, 1);
        let fg = if disabled { DISABLED } else { WHITE };
        ui.fill(row, ink(fg, SURFACE));
        let _ = ui.paint_str(Rect::new(row.x, row.y, 1, 1), " ", ink(SURFACE, SURFACE));
        let _ = ui.paint_str(
            Rect::new(
                row.x.saturating_add(3),
                row.y,
                row.width.saturating_sub(3),
                1,
            ),
            label,
            ink(fg, SURFACE),
        );
    }
}

fn paint_nested(ui: &mut Ui<'_>, area: Rect) {
    ui.fill(area, ink(WHITE, SURFACE));
    paint_card_title(ui, area, "Nested", "");
    let body = card_body(area, true);
    let _ = ui.paint_str(
        Rect::new(body.x, body.y, body.width, 1),
        "Target",
        ink(MUTED, SURFACE),
    );
    paint_targets(ui, body);
    let (note_x, note_y) = if body.width > NOTE_X.saturating_add(20) {
        (body.x.saturating_add(NOTE_X), body.y)
    } else {
        (body.x, body.y.saturating_add(5))
    };
    if note_y < body.bottom() && note_x < body.right() {
        paint_wrapped(
            ui,
            Rect::new(
                note_x,
                note_y,
                body.right().saturating_sub(note_x),
                body.bottom().saturating_sub(note_y),
            ),
            NESTED_NOTE,
            MUTED,
            SURFACE,
        );
    }
}

fn fit_line(text: &str, columns: u16) -> String {
    let clipped = termrock::truncate(text, columns);
    let pad = usize::from(columns.saturating_sub(width(&clipped)));
    format!("{clipped}{}", " ".repeat(pad))
}

fn prose_rows(columns: u16) -> Vec<String> {
    PROSE
        .split('\n')
        .flat_map(|line| wrap(line, columns))
        .collect()
}

fn log_fg(line: &str) -> u32 {
    if line.contains(" error ") {
        ERROR
    } else if line.contains(" warn ") {
        WARNING
    } else {
        SECONDARY
    }
}

fn position_label(viewport: usize, content: usize) -> String {
    if viewport == 0 || content <= viewport {
        return String::new();
    }
    format!("1–{viewport} of {content}")
}

fn thumb_len(viewport: usize, content: usize, track: usize) -> usize {
    if content <= viewport || track == 0 {
        return track;
    }
    (viewport.saturating_mul(track) / content).clamp(1, track)
}

fn fade_fg(fg: u32, bg: u32, row: u16, rows: u16, overflows: bool) -> u32 {
    if !overflows || rows < FADE_MIN {
        return fg;
    }
    let depth = u16::from(rows >= FADE_DEEP).saturating_add(1);
    let from_end = rows.saturating_sub(row).saturating_sub(1);
    if from_end == 0 {
        mix(fg, bg, FADE_OUTER)
    } else if depth == 2 && from_end == 1 {
        mix(fg, bg, FADE_INNER)
    } else {
        fg
    }
}

fn paint_vbar(ui: &mut Ui<'_>, area: Rect, viewport: usize, content: usize, bg: u32) {
    let len = thumb_len(viewport, content, usize::from(area.height));
    for row in 0..area.height {
        let glyph = if usize::from(row) < len { "┃" } else { "┆" };
        let fg = if usize::from(row) < len { THUMB } else { TRACK };
        let _ = ui.paint_str(
            Rect::new(area.x, area.y.saturating_add(row), 1, 1),
            glyph,
            ink(fg, bg),
        );
    }
}

fn paint_hbar(ui: &mut Ui<'_>, area: Rect, viewport: usize, content: usize, bg: u32) {
    let len = thumb_len(viewport, content, usize::from(area.width));
    for col in 0..area.width {
        let glyph = if usize::from(col) < len { "━" } else { "┄" };
        let fg = if usize::from(col) < len { THUMB } else { TRACK };
        let _ = ui.paint_str(
            Rect::new(area.x.saturating_add(col), area.y, 1, 1),
            glyph,
            ink(fg, bg),
        );
    }
}

fn paint_scroll(
    ui: &mut Ui<'_>,
    area: Rect,
    rows: &[String],
    color_of: impl Fn(&str) -> u32,
    bg: u32,
    horizontal: bool,
) {
    if area.width < 2 || area.height == 0 {
        return;
    }
    let text_w = area.width.saturating_sub(2);
    let widest = rows
        .iter()
        .map(|row| usize::from(width(row)))
        .max()
        .unwrap_or(0);
    let h_bar = horizontal && area.height > 1 && widest > usize::from(text_w);
    let rows_h = area.height.saturating_sub(u16::from(h_bar));
    let viewport = usize::from(rows_h);
    let content = rows.len();
    let overflows = content > viewport;
    for (index, line) in rows.iter().take(viewport).enumerate() {
        let Ok(row) = u16::try_from(index) else {
            break;
        };
        let fg = fade_fg(color_of(line), bg, row, rows_h, overflows);
        let _ = ui.paint_str(
            Rect::new(area.x, area.y.saturating_add(row), text_w, 1),
            &fit_line(line, text_w),
            ink(fg, bg),
        );
        let gap = fade_fg(WHITE, bg, row, rows_h, overflows);
        let _ = ui.paint_str(
            Rect::new(
                area.x.saturating_add(text_w),
                area.y.saturating_add(row),
                1,
                1,
            ),
            " ",
            ink(gap, bg),
        );
    }
    if overflows {
        paint_vbar(
            ui,
            Rect::new(area.right().saturating_sub(1), area.y, 1, rows_h),
            viewport,
            content,
            bg,
        );
    }
    if h_bar {
        let track = area.width.saturating_sub(u16::from(overflows));
        paint_hbar(
            ui,
            Rect::new(area.x, area.y.saturating_add(rows_h), track, 1),
            usize::from(text_w),
            widest,
            bg,
        );
    }
}

fn framed_body(area: Rect) -> Rect {
    Rect::new(
        area.x.saturating_add(3),
        area.y.saturating_add(1),
        area.width.saturating_sub(5),
        area.height.saturating_sub(2),
    )
}

fn paint_frame_border(ui: &mut Ui<'_>, area: Rect) {
    if area.width < 2 || area.height < 2 {
        return;
    }
    let style = ink(RULE, CANVAS);
    let inner = usize::from(area.width.saturating_sub(2));
    let _ = ui.paint_str(
        Rect::new(area.x, area.y, area.width, 1),
        &format!("╭{}╮", "─".repeat(inner)),
        style,
    );
    for row in 1..area.height.saturating_sub(1) {
        let y = area.y.saturating_add(row);
        let _ = ui.paint_str(Rect::new(area.x, y, 1, 1), "│", style);
        let _ = ui.paint_str(
            Rect::new(area.right().saturating_sub(1), y, 1, 1),
            "│",
            style,
        );
    }
    let _ = ui.paint_str(
        Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1),
        &format!("╰{}╯", "─".repeat(inner)),
        style,
    );
}

fn paint_frame_title(ui: &mut Ui<'_>, area: Rect, title: &str, meta: &str) {
    if area.width <= 4 {
        return;
    }
    let x = area.x.saturating_add(2);
    let span = area.width.saturating_sub(4);
    ui.fill(Rect::new(x, area.y, span, 1), ink(RULE, CANVAS));
    let title = format!(" {title} ");
    let _ = ui.paint_str(
        Rect::new(x, area.y, span, 1),
        &title,
        ink(SECONDARY, CANVAS),
    );
    if meta.is_empty() {
        return;
    }
    let meta = format!(" {meta} ");
    let meta_w = width(&meta);
    if meta_w >= span {
        return;
    }
    let _ = ui.paint_str(
        Rect::new(
            x.saturating_add(span).saturating_sub(meta_w),
            area.y,
            meta_w,
            1,
        ),
        &meta,
        ink(FAINT, CANVAS),
    );
}

fn paint_framed_prose(ui: &mut Ui<'_>, area: Rect) {
    ui.fill(area, ink(WHITE, CANVAS));
    let body = framed_body(area);
    let text_w = body.width.saturating_sub(2);
    let rows = prose_rows(text_w);
    let shown = usize::from(body.height);
    paint_frame_border(ui, area);
    paint_frame_title(
        ui,
        area,
        "Framed · split pane",
        &position_label(shown, rows.len()),
    );
    paint_scroll(ui, body, &rows, |_| SECONDARY, CANVAS, false);
}

fn paint_log_card(ui: &mut Ui<'_>, area: Rect, lines: &[String]) {
    ui.fill(area, ink(WHITE, SURFACE));
    let body = card_body(area, true);
    let text_w = body.width.saturating_sub(2);
    let h_bar = body.height > 1 && lines.iter().any(|line| width(line) > text_w);
    let shown = usize::from(body.height.saturating_sub(u16::from(h_bar)));
    paint_card_title(
        ui,
        area,
        "Card · scrollable",
        &position_label(shown, lines.len()),
    );
    paint_scroll(ui, body, lines, log_fg, SURFACE, true);
}

fn paint_panes(ui: &mut Ui<'_>, area: Rect, lines: &[String]) {
    let top_h = area.height / 2;
    let frame = Rect::new(area.x, area.y, area.width, top_h.min(area.height));
    let rest_y = frame.bottom();
    let rest_h = area.bottom().saturating_sub(rest_y);
    let card = Rect::new(
        area.x,
        rest_y.saturating_add(1),
        area.width,
        rest_h.saturating_sub(1),
    );
    paint_framed_prose(ui, frame);
    if !card.is_empty() {
        paint_log_card(ui, card, lines);
    }
}

fn stacked_height(heights: &[u16]) -> u16 {
    let gaps = u16::try_from(heights.len().saturating_sub(1)).unwrap_or(u16::MAX);
    heights
        .iter()
        .copied()
        .fold(0, u16::saturating_add)
        .saturating_add(gaps.saturating_mul(STACK_GAP))
}

fn paint_card_column(ui: &mut Ui<'_>, area: Rect) {
    let slots = stack_slots(area, &card_heights(card_inner_width(area.width)));
    if let Some(slot) = slots.first().copied() {
        paint_titled(ui, slot);
    }
    if let Some(slot) = slots.get(1).copied() {
        paint_untitled(ui, slot);
    }
    if let Some(slot) = slots.get(2).copied() {
        paint_nested(ui, slot);
    }
}

fn paint_stacked(ui: &mut Ui<'_>, area: Rect, lines: &[String]) {
    if area.width < 2 || area.height == 0 {
        return;
    }
    let column_w = area.width.saturating_sub(2);
    let cards_h = stacked_height(&card_heights(card_inner_width(column_w)));
    let content_h = cards_h.saturating_mul(2).saturating_add(STACK_GAP);
    ui.with_area(area, |ui| {
        paint_card_column(ui, Rect::new(area.x, area.y, column_w, cards_h));
        paint_panes(
            ui,
            Rect::new(
                area.x,
                area.y.saturating_add(cards_h).saturating_add(STACK_GAP),
                column_w,
                cards_h,
            ),
            lines,
        );
    });
    if content_h > area.height {
        paint_vbar(
            ui,
            Rect::new(area.right().saturating_sub(1), area.y, 1, area.height),
            usize::from(area.height),
            usize::from(content_h),
            CANVAS,
        );
    }
}

fn paint_playground(ui: &mut Ui<'_>, area: Rect, lines: &[String]) {
    ui.fill(area, ink(WHITE, CANVAS));
    let Some((left, right)) = split_columns(area) else {
        paint_stacked(ui, area, lines);
        return;
    };
    paint_card_column(ui, left);
    paint_panes(ui, right, lines);
}

#[derive(Debug, Default)]
pub struct PanelsPage;

impl PanelsPage {
    pub fn new() -> Self {
        Self
    }
}

impl Page for PanelsPage {
    fn title(&self) -> &'static str {
        "Panels"
    }

    fn update(&mut self, _cx: &mut Cx<'_>) -> PageUpdate {
        Response::ignored().into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let lines = log_lines(60);
        frame(ui, area, self.title(), "", |ui, body| {
            paint_blurb(ui, body);
            paint_section_strip(
                ui,
                Rect {
                    height: 2.min(body.height),
                    ..body
                },
            );
            let top = body.y.saturating_add(3);
            if top >= body.bottom() {
                return;
            }
            paint_playground(
                ui,
                Rect {
                    y: top,
                    height: body.bottom().saturating_sub(top),
                    ..body
                },
                &lines,
            );
        });
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[("↑ ↓", "Scroll"), ("PgUp PgDn", "Page"), ("g G", "Ends")]
    }
}

#[cfg(test)]
mod tests {
    use super::{Page, PanelsPage, card_body, framed_body, prose_rows};
    use termrock::{App, Color, Cx, Modifier, Rect, Response, Theme, Ui, width};
    use termrock_test_support::Harness;

    struct Host;

    impl App for Host {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            PanelsPage::new().draw(ui, ui.full());
        }
    }

    #[test]
    fn playground_at_120_matches_the_card_copy() {
        let h = Harness::new(Host, Theme::junie(), 94, 36);
        let text = h.text();
        assert!(text.contains("Playground"), "{text}");
        assert!(text.contains("the live widget"), "{text}");
        assert!(text.contains("Titled card"), "{text}");
        assert!(text.contains("surface"), "{text}");
        assert!(text.contains("Framed · split pane"), "{text}");
        assert!(text.contains("1–13 of 24"), "{text}");
        assert!(
            text.contains("Untitled card. Same surface, content"),
            "{text}"
        );
        assert!(text.contains("Nested"), "{text}");
        assert!(text.contains("Target"), "{text}");
        assert!(text.contains("Local"), "{text}");
        assert!(text.contains("Cloud"), "{text}");
        assert!(
            text.contains("A group inside a card is a muted label"),
            "{text}"
        );
        assert!(text.contains("Card · scrollable"), "{text}");
        assert!(text.contains("1–11 of 60"), "{text}");
        assert!(text.contains("Resolving workspace me…"), "{text}");
        assert!(text.contains("0.00s"), "{text}");
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

        let title = h.cell(2, 5);
        assert_eq!(title.symbol(), "T");
        assert_eq!(title.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        assert!(!title.modifier.contains(Modifier::BOLD));

        let body = framed_body(Rect::new(48, 5, 46, 15));
        assert_eq!(body, Rect::new(51, 6, 41, 13));
        assert_eq!(prose_rows(body.width.saturating_sub(2)).len(), 24);
        let card = card_body(Rect::new(48, 21, 46, 15), true);
        assert_eq!(card.width.saturating_sub(2), 40);
        assert!(width("   0.00s  info   Resolving workspace members") > 40);
    }

    #[test]
    fn narrow_playground_stacks_the_cards_and_ends_the_blurb_on_ellipsis() {
        let h = Harness::new(Host, Theme::junie(), 51, 16);
        assert_eq!(h.cell(49, 0).symbol(), "…");
        assert_eq!(h.cell(49, 0).fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(h.cell(2, 5).symbol(), "T");
        assert_eq!(h.cell(2, 5).bg, Color::Rgb(0x11, 0x11, 0x11));
        assert_eq!(h.cell(50, 5).symbol(), "┃");
        assert_eq!(h.cell(50, 5).fg, Color::Rgb(0x80, 0x80, 0x80));
        assert!(h.text().contains("Untitled card"), "{}", h.text());
        assert!(!h.text().contains("Framed"), "{}", h.text());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
