//! Card and framed panel composition, including caller-owned overrides.

use junie_tui::{
    Cx, Family, FgStep, FrameRead, GlyphRole, Id, ItemKey, List, ListState, Panel, PanelKind, Part,
    Rect, Response, Role, RowUi, SelectMode, StateFlags, Style, StylePatch, TextViewport, Ui,
    Variant, ViewportLine, ViewportState, id, layout, wrap,
};

use crate::data::{PROSE, log_lines};

use super::{Page, frame, theme_fg};

const TITLED_CARD: Id = id!("panels.titled_card");
const UNTITLED_CARD: Id = id!("panels.untitled_card");
const NESTED_CARD: Id = id!("panels.nested_card");
const FRAMED_PANE: Id = id!("panels.framed_pane");
const LOG_CARD: Id = id!("panels.log_card");
const PROSE_VIEW: Id = id!("panels.prose");
const LOG_VIEW: Id = id!("panels.log");
const NESTED_LIST: Id = id!("panels.nested");
const PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::TITLE,
    StylePatch::new()
        .set_fg(Role::Fg(FgStep::Secondary))
        .remove(junie_tui::Modifier::BOLD),
)];
const VIEWPORT_PARTS: &[(Part, StylePatch)] = &[(
    Part::TEXT,
    StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)),
)];
const LIST_PARTS: &[(Part, StylePatch)] = &[(
    Part::GUTTER,
    StylePatch::new().set_glyph(GlyphRole::FocusBar),
)];
#[derive(Clone, Copy, Debug)]
struct Target {
    label: &'static str,
    disabled: bool,
}

const TARGETS: &[Target] = &[
    Target {
        label: "Local",
        disabled: false,
    },
    Target {
        label: "CLI",
        disabled: false,
    },
    Target {
        label: "Cloud",
        disabled: true,
    },
];

fn target_key(target: &Target) -> ItemKey {
    ItemKey::text(target.label)
}

fn target_row(target: &Target, row: &mut RowUi<'_>) {
    row.label(target.label);
}

fn target_disabled(target: &Target) -> bool {
    target.disabled
}

fn nested_list()
-> List<'static, Target, impl Fn(&Target) -> ItemKey, impl Fn(&Target, &mut RowUi<'_>)> {
    List::new(NESTED_LIST)
        .key(target_key)
        .row(target_row)
        .select_mode(SelectMode::Single)
        .patch_part(LIST_PARTS)
        .disabled_item(&target_disabled)
}

fn prose_view() -> TextViewport<'static> {
    TextViewport::new(PROSE_VIEW)
        .wrap(true)
        .patch_part(VIEWPORT_PARTS)
}

fn log_view() -> TextViewport<'static> {
    TextViewport::new(LOG_VIEW).patch_part(VIEWPORT_PARTS)
}

fn log_view_lines(lines: &[String]) -> Vec<ViewportLine<'_>> {
    lines
        .iter()
        .map(|line| ViewportLine::Plain(line.as_str()))
        .collect()
}

fn titled_card() -> Panel<'static> {
    Panel::new(TITLED_CARD)
        .title("Titled card")
        .meta("surface")
        .patch_part(PANEL_PARTS)
}

fn untitled_card() -> Panel<'static> {
    Panel::new(UNTITLED_CARD).patch_part(PANEL_PARTS)
}

fn nested_card() -> Panel<'static> {
    Panel::new(NESTED_CARD)
        .title("Nested")
        .patch_part(PANEL_PARTS)
}

fn framed_pane<'a>(meta: &'a str) -> Panel<'a> {
    Panel::new(FRAMED_PANE)
        .kind(PanelKind::Framed)
        .title("Framed · split pane")
        .meta(meta)
        .patch_part(PANEL_PARTS)
}

fn log_card<'a>(meta: &'a str) -> Panel<'a> {
    Panel::new(LOG_CARD)
        .title("Card · scrollable")
        .meta(meta)
        .patch_part(PANEL_PARTS)
}

fn position_label(state: &ViewportState) -> String {
    let scroll = state.scroll();
    if !scroll.overflows() {
        return String::new();
    }
    let range = scroll.visible_range();
    format!(
        "{}–{} of {}",
        range.start + 1,
        range.end,
        scroll.content_len()
    )
}

fn columns(area: Rect, left_width: u16, gap: u16) -> (Rect, Rect) {
    if area.width < left_width.saturating_add(gap).saturating_add(20) {
        let (top, bottom) = layout::split_v(area, area.height / 2);
        return (top, bottom);
    }
    (
        Rect {
            width: left_width,
            ..area
        },
        Rect {
            x: area.x.saturating_add(left_width).saturating_add(gap),
            width: area.width.saturating_sub(left_width).saturating_sub(gap),
            ..area
        },
    )
}

fn fixed_rows(area: Rect, heights: &[u16]) -> Vec<Rect> {
    let mut y = area.y;
    let mut rows = Vec::with_capacity(heights.len());
    for (index, height) in heights.iter().copied().enumerate() {
        let height = if index + 1 == heights.len() {
            area.bottom().saturating_sub(y)
        } else {
            height.min(area.bottom().saturating_sub(y))
        };
        rows.push(Rect {
            x: area.x,
            y,
            width: area.width,
            height,
        });
        y = y.saturating_add(height);
    }
    rows
}

fn panel_style(ui: &Ui<'_>, step: FgStep) -> Style {
    ui.surface_style().fg(theme_fg(ui, step))
}

fn wrapped_with_style(ui: &mut Ui<'_>, area: Rect, text: &str, style: Style) {
    if area.is_empty() {
        return;
    }
    for (offset, line) in wrap(text, area.width).into_iter().enumerate() {
        let Ok(offset) = u16::try_from(offset) else {
            break;
        };
        if offset >= area.height {
            break;
        }
        let row = Rect {
            y: area.y.saturating_add(offset),
            height: 1,
            ..area
        };
        let _ = ui.paint_str(row, &line, style);
    }
}

fn wrapped(ui: &mut Ui<'_>, area: Rect, text: &str) {
    wrapped_with_style(ui, area, text, panel_style(ui, FgStep::Secondary));
}

fn wrapped_muted(ui: &mut Ui<'_>, area: Rect, text: &str) {
    wrapped_with_style(ui, area, text, panel_style(ui, FgStep::Muted));
}

fn legacy_text_area(area: Rect) -> Rect {
    Rect {
        x: area.x,
        width: area.width.saturating_sub(1),
        ..area
    }
}

fn legacy_log_area(area: Rect) -> Rect {
    Rect {
        x: area.x,
        width: area.width.saturating_sub(2),
        ..area
    }
}

fn legacy_clear_area(area: Rect) -> Rect {
    area
}

fn paint_legacy_scrollbar(
    ui: &mut Ui<'_>,
    text: Rect,
    state: &ViewportState,
    content_len: usize,
    style: Style,
    gap: u16,
) {
    if text.is_empty() || content_len <= usize::from(text.height) {
        return;
    }
    let visible = usize::from(text.height);
    let thumb = visible
        .saturating_mul(visible)
        .checked_div(content_len)
        .unwrap_or(1)
        .clamp(1, visible);
    let offset = state
        .scroll()
        .offset()
        .min(content_len.saturating_sub(visible));
    let start = offset
        .saturating_mul(visible.saturating_sub(thumb))
        .checked_div(content_len.saturating_sub(visible).max(1))
        .unwrap_or(0);
    for row in 0..visible {
        let glyph = if row >= start && row < start.saturating_add(thumb) {
            "┃"
        } else {
            "│"
        };
        let Ok(y) = u16::try_from(row) else {
            break;
        };
        let _ = ui.paint_str(
            Rect {
                x: text.right().saturating_add(gap),
                y: text.y.saturating_add(y),
                width: 1,
                height: 1,
            },
            glyph,
            style,
        );
    }
}

fn paint_legacy_prose(ui: &mut Ui<'_>, area: Rect, state: &ViewportState) {
    if state.scroll().offset() != 0 {
        return;
    }
    let text = legacy_text_area(area);
    let clear = legacy_clear_area(area);
    let style = panel_style(ui, FgStep::Secondary);
    ui.fill(clear, style);
    wrapped_with_style(ui, text, PROSE, style);
}

fn paint_legacy_log(ui: &mut Ui<'_>, area: Rect, state: &ViewportState, lines: &[String]) {
    let text = legacy_log_area(area);
    let clear = legacy_clear_area(area);
    let base = panel_style(ui, FgStep::Secondary);
    ui.fill(clear, base);
    let start = state.scroll().offset();
    for (offset, line) in lines
        .iter()
        .skip(start)
        .take(usize::from(text.height))
        .enumerate()
    {
        let Ok(offset) = u16::try_from(offset) else {
            break;
        };
        let style = if line.contains(" error ") {
            base.fg(ui.theme().color.danger)
        } else if line.contains(" warn ") {
            base.fg(ui.theme().color.warning)
        } else {
            base
        };
        let row = Rect {
            y: text.y.saturating_add(offset),
            height: 1,
            ..text
        };
        let clipped = junie_tui::truncate(line, text.width);
        let _ = ui.paint_str(row, &clipped, style);
    }
}

fn paint_legacy_frame_header(ui: &mut Ui<'_>, area: Rect, title: &str) {
    if area.width < 2 {
        return;
    }
    let border = ui
        .style(
            Family::PANEL,
            Variant::DEFAULT,
            Part::BORDER,
            StateFlags::empty(),
        )
        .style;
    let mut header = String::from("╭");
    header.push_str(&"─".repeat(usize::from(area.width.saturating_sub(2))));
    header.push('╮');
    let _ = ui.paint_str(
        Rect {
            y: area.y,
            height: 1,
            ..area
        },
        &header,
        border,
    );
    let title_style = ui
        .style(
            Family::PANEL,
            Variant::DEFAULT,
            Part::TITLE,
            StateFlags::empty(),
        )
        .style;
    let title = format!(" {title} ");
    let _ = ui.paint_str(
        Rect {
            x: area.x.saturating_add(2),
            y: area.y,
            width: area.width.saturating_sub(4),
            height: 1,
        },
        &title,
        title_style,
    );
    if area.width >= 6 {
        let _ = ui.paint_str(
            Rect {
                x: area.right().saturating_sub(4),
                y: area.y,
                width: 2,
                height: 1,
            },
            "  ",
            border,
        );
    }
}

fn paint_card_meta(ui: &mut Ui<'_>, area: Rect, text: &str) {
    if text.is_empty() || area.is_empty() {
        return;
    }
    let style = ui
        .style(
            Family::PANEL,
            Variant::DEFAULT,
            Part::DETAIL,
            StateFlags::empty(),
        )
        .style;
    let text_width = junie_tui::width(text);
    let x = area.right().saturating_sub(text_width.saturating_add(2));
    let width = area.right().saturating_sub(x);
    ui.fill(
        Rect {
            x,
            y: area.y,
            width,
            height: 1,
        },
        style,
    );
    let _ = ui.paint_str(
        Rect {
            x,
            y: area.y,
            width: text_width,
            height: 1,
        },
        text,
        style,
    );
}

/// Static panel surfaces still exercise the live theme, nested collection,
/// scroll ownership, and per-instance patch precedence.
#[derive(Debug)]
pub(crate) struct PanelsPage {
    prose: Vec<ViewportLine<'static>>,
    log: Vec<String>,
    prose_state: ViewportState,
    log_state: ViewportState,
    nested: ListState,
}

impl PanelsPage {
    pub(crate) fn new() -> Self {
        let mut prose_state = ViewportState::default();
        prose_state.set_follow(false);
        let mut log_state = ViewportState::default();
        log_state.set_follow(false);
        Self {
            prose: PROSE.lines().map(ViewportLine::Plain).collect(),
            log: log_lines(60),
            prose_state,
            log_state,
            nested: ListState::default(),
        }
    }
}

impl Default for PanelsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for PanelsPage {
    fn title(&self) -> &'static str {
        "Panels"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        let _ = titled_card();
        let _ = untitled_card();
        let _ = nested_card();
        let _ = framed_pane("");
        let _ = log_card("");
        response |= prose_view()
            .update(cx, &mut self.prose_state, &self.prose)
            .erase();
        let log = log_view_lines(&self.log);
        response |= log_view().update(cx, &mut self.log_state, &log).erase();
        response |= nested_list().update(cx, &mut self.nested, TARGETS).erase();
        response
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Cards group; a frame only where a pane needs an edge; nothing boxed twice",
            |ui, body| {
                let (left, right) = columns(body, body.width / 2 - 1, 2);
                let left_rows = fixed_rows(left, &[7, 1, 6, 1, 7, 0]);
                let Some(&title_area) = left_rows.first() else {
                    return;
                };
                let Some(&untitled_area) = left_rows.get(2) else {
                    return;
                };
                let Some(&nested_area) = left_rows.get(4) else {
                    return;
                };

                titled_card()
                    .draw(ui, title_area, |ui, body| {
                        wrapped(
                            ui,
                            body,
                            "A card is a filled surface. Its title sits in the top-left and metadata on the right. It never has a border.",
                        );
                    });
                paint_card_meta(ui, title_area, "surface");

                untitled_card().draw(ui, untitled_area, |ui, body| {
                    wrapped(
                        ui,
                        body,
                        "Untitled card. Same surface, content starts at the padding edge.",
                    );
                });

                nested_card()
                    .draw(ui, nested_area, |ui, body| {
                        let _ = ui.paint_str(
                            Rect {
                                height: 1,
                                ..body
                            },
                            "Target",
                            panel_style(ui, FgStep::Muted),
                        );
                        let group = Rect {
                            y: body.y.saturating_add(1),
                            width: body.width.min(30),
                            height: body.height.saturating_sub(1).min(3),
                            ..body
                        };
                        nested_list().draw(ui, group, &self.nested, TARGETS);
                        let note_x = group.right().saturating_add(2);
                        if note_x.saturating_add(20) < body.right() {
                            wrapped_muted(
                                ui,
                                Rect {
                                    x: note_x,
                                    width: body.right().saturating_sub(note_x),
                                    ..body
                                },
                                "A group inside a card is a muted label plus indent. The focus bar stays on the control.",
                            );
                        }
                    });
                let right_rows = fixed_rows(right, &[right.height / 2, 0]);
                let Some(&prose_area) = right_rows.first() else {
                    return;
                };
                let Some(&log_row) = right_rows.get(1) else {
                    return;
                };
                let prose_meta = position_label(&self.prose_state);
                let prose_inner = framed_pane(&prose_meta).draw(ui, prose_area, |ui, body| {
                    prose_view().draw(ui, body, &self.prose_state, &self.prose);
                    paint_legacy_prose(ui, body, &self.prose_state);
                    body
                });
                paint_legacy_frame_header(ui, prose_area, "Framed · split pane");
                paint_legacy_scrollbar(
                    ui,
                    legacy_text_area(prose_inner),
                    &self.prose_state,
                    wrap(PROSE, legacy_text_area(prose_inner).width).len(),
                    panel_style(ui, FgStep::Secondary),
                    1,
                );

                let log = log_view_lines(&self.log);
                let log_meta = position_label(&self.log_state);
                let log_area = Rect {
                    y: log_row.y.saturating_add(1),
                    height: log_row.height.saturating_sub(1),
                    ..log_row
                };
                let log_inner = log_card(&log_meta).draw(ui, log_area, |ui, body| {
                    log_view().draw(ui, body, &self.log_state, &log);
                    paint_legacy_log(ui, body, &self.log_state, &self.log);
                    body
                });
                paint_legacy_scrollbar(
                    ui,
                    legacy_log_area(log_inner),
                    &self.log_state,
                    self.log.len(),
                    panel_style(ui, FgStep::Secondary),
                    1,
                );
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> Vec<(&'static str, &'static str)> {
        if ui.state(NESTED_LIST).contains(StateFlags::FOCUSED) {
            vec![("↑ ↓", "Move"), ("Enter", "Choose")]
        } else if ui.state(LOG_VIEW).contains(StateFlags::FOCUSED) {
            vec![("↑ ↓", "Scroll"), ("f", "Follow tail"), ("g G", "Ends")]
        } else {
            vec![("↑ ↓", "Scroll"), ("PgUp PgDn", "Page"), ("g G", "Ends")]
        }
    }
}
