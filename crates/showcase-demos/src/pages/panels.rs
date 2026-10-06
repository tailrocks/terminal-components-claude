//! Card and framed panel composition, including caller-owned overrides.

use termrock::author::PaintStyle;
use termrock::{
    Cx, Family, FgStep, FrameRead, Id, ItemKey, List, ListState, Panel, PanelKind, Part, Rect,
    Response, Role, RowUi, ScrollState, SelectMode, StateFlags, StylePatch, TextViewport, Ui,
    Variant, ViewportLine, ViewportState, id, layout, wrap,
};

use showcase_data::{PROSE, log_lines};

use super::{Page, PageUpdate, frame};

const TITLED_CARD: Id = id!("panels.titled_card");
const UNTITLED_CARD: Id = id!("panels.untitled_card");
const NESTED_CARD: Id = id!("panels.nested_card");
const FRAMED_PANE: Id = id!("panels.framed_pane");
const LOG_CARD: Id = id!("panels.log_card");
const PROSE_VIEW: Id = id!("panels.prose");
const LOG_VIEW: Id = id!("panels.log");
const NESTED_LIST: Id = id!("panels.nested");
const PANEL_PARTS: &[(Part, StylePatch)] = &[
    (
        Part::TITLE,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .remove(termrock::Modifier::BOLD),
    ),
    (
        Part::DETAIL,
        StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
    ),
];
const VIEWPORT_PARTS: &[(Part, StylePatch)] = &[(
    Part::TEXT,
    StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)),
)];
const LIST_PARTS: &[(Part, StylePatch)] =
    &[(Part::GUTTER, StylePatch::new().set_fg(Role::CurrentSurface))];
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

/// The one constructor per card and pane on this page (§13): both phase paths
/// build the same props, so no per-phase tweak can go unseen.
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

fn framed_pane(meta: &str) -> Panel<'_> {
    Panel::new(FRAMED_PANE)
        .kind(PanelKind::Framed)
        .title("Framed · split pane")
        .meta(meta)
        .patch_part(PANEL_PARTS)
}

fn log_card(meta: &str) -> Panel<'_> {
    Panel::new(LOG_CARD)
        .title("Card · scrollable")
        .meta(meta)
        .patch_part(PANEL_PARTS)
}

fn prose_position_label(area: Rect, state: &ViewportState) -> String {
    let inner_height = area.height.saturating_sub(2);
    let text_w = area.width.saturating_sub(7);
    if text_w == 0 || inner_height == 0 {
        return String::new();
    }
    let total = wrap(PROSE, text_w).len();
    if total <= usize::from(inner_height) {
        return String::new();
    }
    let offset = state.scroll().offset();
    let start = offset.saturating_add(1);
    let end = offset.saturating_add(usize::from(inner_height)).min(total);
    format!("{start}–{end} of {total}")
}

fn log_position_label(card_area: Rect, state: &ViewportState, total: usize) -> String {
    let inner_height = card_area.height.saturating_sub(3);
    if inner_height == 0 || total <= usize::from(inner_height) {
        return String::new();
    }
    let offset = state.scroll().offset();
    let start = offset.saturating_add(1);
    let end = offset.saturating_add(usize::from(inner_height)).min(total);
    let pos = format!("{start}–{end} of {total}");
    if state.follow() {
        format!("{pos} · following")
    } else {
        pos
    }
}

fn paint_framed_meta(ui: &mut Ui<'_>, area: Rect, title: &str, meta: &str) {
    if area.width <= 4 {
        return;
    }
    let row = Rect {
        x: area.x.saturating_add(2),
        y: area.y,
        width: area.width.saturating_sub(4),
        height: 1,
    };
    let border_style = ui
        .style(
            Family::PANEL,
            Variant::DEFAULT,
            Part::BORDER,
            StateFlags::empty(),
        )
        .style;
    ui.fill(row, border_style);
    let title_style = ui
        .style(
            Family::PANEL,
            Variant::DEFAULT,
            Part::TITLE,
            StateFlags::empty(),
        )
        .style;
    let meta_style = panel_style(ui, FgStep::Faint);
    let pad = 2u16;
    let w = row.width;
    let title_min = termrock::width(title).min(4);
    let meta_opt = {
        let room = w.saturating_sub(pad + if title_min > 0 { title_min + 1 } else { 0 });
        if termrock::width(meta) > room {
            Some(termrock::truncate(meta, room))
        } else {
            Some(meta.to_string())
        }
    };
    let meta_w = meta_opt
        .as_ref()
        .map(|m| termrock::width(m) + pad)
        .unwrap_or(0);
    let mut cx = row.x;
    let room = if meta_w > 0 {
        w.saturating_sub(meta_w + 1 + pad)
    } else {
        w.saturating_sub(pad)
    };
    let t_trunc = termrock::truncate(title, room);
    let title_text = format!(" {t_trunc} ");
    let tw = termrock::width(&title_text);
    let _ = ui.paint_str(
        Rect {
            x: cx,
            y: row.y,
            width: tw,
            height: 1,
        },
        &title_text,
        title_style,
    );
    cx = cx.saturating_add(tw);
    let mut right = row.right();
    if let Some(m) = meta_opt {
        let meta_text = format!(" {m} ");
        let mw = termrock::width(&meta_text);
        if right >= cx + mw + u16::from(cx > row.x) {
            right = right.saturating_sub(mw);
            if right == cx.saturating_add(1) {
                let _ = ui.paint_str(
                    Rect {
                        x: cx,
                        y: row.y,
                        width: 1,
                        height: 1,
                    },
                    " ",
                    title_style,
                );
            }
            let _ = ui.paint_str(
                Rect {
                    x: right,
                    y: row.y,
                    width: mw,
                    height: 1,
                },
                &meta_text,
                meta_style,
            );
        }
    }
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

fn fixed_rows<const N: usize>(area: Rect, heights: &[u16; N]) -> [Rect; N] {
    let mut y = area.y;
    let mut rows = [Rect::default(); N];
    for (index, (row, height)) in rows.iter_mut().zip(heights.iter().copied()).enumerate() {
        let height = if index.saturating_add(1) == heights.len() {
            area.bottom().saturating_sub(y)
        } else {
            height.min(area.bottom().saturating_sub(y))
        };
        *row = Rect {
            x: area.x,
            y,
            width: area.width,
            height,
        };
        y = y.saturating_add(height);
    }
    rows
}

fn panel_style(ui: &Ui<'_>, step: FgStep) -> PaintStyle {
    ui.surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(step))))
}

fn wrapped_with_style(ui: &mut Ui<'_>, area: Rect, text: &str, style: PaintStyle) {
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
        width: area.width.saturating_sub(2),
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
    let track_style = ui
        .style(
            Family::VIEWPORT,
            Variant::DEFAULT,
            Part::TRACK,
            StateFlags::empty(),
        )
        .style;
    let thumb_style = ui
        .style(
            Family::VIEWPORT,
            Variant::DEFAULT,
            Part::THUMB,
            StateFlags::empty(),
        )
        .style;
    for row in 0..visible {
        let (glyph, style) = if row >= start && row < start.saturating_add(thumb) {
            ("┃", thumb_style)
        } else {
            ("│", track_style)
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
    let clear = legacy_clear_area(area);
    ui.fill(clear, ui.surface_style());
    let text = legacy_text_area(area);
    let style = panel_style(ui, FgStep::Secondary);
    if text.is_empty() {
        return;
    }
    for (offset, line) in wrap(PROSE, text.width).into_iter().enumerate() {
        let Ok(offset) = u16::try_from(offset) else {
            break;
        };
        if offset >= text.height {
            break;
        }
        let row = Rect {
            y: text.y.saturating_add(offset),
            height: 1,
            ..text
        };
        let pad = usize::from(text.width.saturating_sub(termrock::width(&line)));
        let fitted = format!("{line}{}", " ".repeat(pad));
        let _ = ui.paint_str(row, &fitted, style);
    }
    let total = wrap(PROSE, text.width).len();
    if total > usize::from(text.height) {
        let mut scroll = ScrollState::new(total);
        scroll.apply_layout(usize::from(text.height), total);
        scroll.scroll_to(state.scroll().offset());
        let fade_rect = Rect {
            x: area.x,
            y: area.y,
            width: area.width.saturating_sub(1),
            height: area.height,
        };
        ui.scroll_edges(fade_rect, &scroll);
    }
}

fn paint_legacy_log(ui: &mut Ui<'_>, area: Rect, state: &ViewportState, lines: &[String]) {
    let text = legacy_log_area(area);
    let clear = legacy_clear_area(area);
    ui.fill(clear, ui.surface_style());
    let base = panel_style(ui, FgStep::Secondary);
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
            base.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Danger)))
        } else if line.contains(" warn ") {
            base.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Warning)))
        } else {
            base
        };
        let row = Rect {
            y: text.y.saturating_add(offset),
            height: 1,
            ..text
        };
        let clipped = termrock::truncate(line, text.width);
        let pad = usize::from(text.width.saturating_sub(termrock::width(&clipped)));
        let fitted = format!("{clipped}{}", " ".repeat(pad));
        let _ = ui.paint_str(row, &fitted, style);
    }
    if lines.len() > usize::from(text.height) {
        let mut scroll = ScrollState::new(lines.len());
        scroll.apply_layout(usize::from(text.height), lines.len());
        scroll.scroll_to(start);
        let fade_rect = Rect {
            x: area.x,
            y: area.y,
            width: area.width.saturating_sub(1),
            height: area.height,
        };
        ui.scroll_edges(fade_rect, &scroll);
    }
}

/// Static panel surfaces still exercise the live theme, nested collection,
/// scroll ownership, and per-instance patch precedence.
#[derive(Debug)]
pub struct PanelsPage {
    prose: Vec<ViewportLine<'static>>,
    log: Vec<String>,
    prose_state: ViewportState,
    log_state: ViewportState,
    nested: ListState,
}

impl PanelsPage {
    pub fn new() -> Self {
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

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
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
        let _ = titled_card();
        let _ = untitled_card();
        let _ = nested_card();
        let _ = framed_pane("");
        let _ = log_card("");
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Cards group; a frame only where a pane needs an edge; nothing boxed twice",
            |ui, body| {
                let (left, right) = columns(body, (body.width / 2).saturating_sub(1), 2);
                let left_rows = fixed_rows(left, &[7, 1, 6, 1, 7, 0]);

                titled_card().draw(ui, left_rows[0], |ui, body| {
                    wrapped(
                        ui,
                        body,
                        "A card is a filled surface. Its title sits in the top-left and metadata on the right. It never has a border.",
                    );
                });

                untitled_card().draw(ui, left_rows[2], |ui, body| {
                    wrapped(
                        ui,
                        body,
                        "Untitled card. Same surface, content starts at the padding edge.",
                    );
                });

                nested_card().draw(ui, left_rows[4], |ui, body| self.draw_nested(ui, body));
                if left_rows[4].height >= 3 {
                    let card_surface = ui.theme().raise(ui.surface());
                    ui.with_surface(card_surface, |ui| {
                        let _ = ui.paint_str(
                            Rect {
                                x: left_rows[4].x.saturating_add(2),
                                y: left_rows[4].y.saturating_add(2),
                                width: left_rows[4].width.saturating_sub(4),
                                height: 1,
                            },
                            "Target",
                            panel_style(ui, FgStep::Muted),
                        );
                    });
                } else if left_rows[4].y <= body.bottom() {
                    let card_surface = ui.theme().raise(ui.surface());
                    ui.with_surface(card_surface, |ui| {
                        let _ = ui.paint_str(
                            Rect {
                                x: left_rows[4].x,
                                y: left_rows[4].y,
                                width: left_rows[4].width,
                                height: 1,
                            },
                            "Target",
                            panel_style(ui, FgStep::Muted),
                        );
                    });
                }

                let right_rows = fixed_rows(right, &[right.height / 2, 0]);
                let [prose_row, log_row] = right_rows;
                let prose_meta = prose_position_label(prose_row, &self.prose_state);
                let prose_inner = framed_pane(&prose_meta).draw(ui, prose_row, |ui, body| {
                    prose_view().draw(ui, body, &self.prose_state, &self.prose);
                    paint_legacy_prose(ui, body, &self.prose_state);
                    body
                });
                paint_framed_meta(ui, prose_row, "Framed · split pane", &prose_meta);
                paint_legacy_scrollbar(
                    ui,
                    legacy_text_area(prose_inner),
                    &self.prose_state,
                    wrap(PROSE, legacy_text_area(prose_inner).width).len(),
                    1,
                );

                let log = log_view_lines(&self.log);
                let log_area = Rect {
                    y: log_row.y.saturating_add(1),
                    height: log_row.height.saturating_sub(1),
                    ..log_row
                };
                let log_meta = log_position_label(log_area, &self.log_state, self.log.len());
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
                    1,
                );
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(NESTED_LIST).contains(StateFlags::FOCUSED) {
            &[("↑ ↓", "Move"), ("Enter", "Choose")]
        } else if ui.state(LOG_VIEW).contains(StateFlags::FOCUSED) {
            &[("↑ ↓", "Scroll"), ("f", "Follow tail"), ("g G", "Ends")]
        } else {
            &[("↑ ↓", "Scroll"), ("PgUp PgDn", "Page"), ("g G", "Ends")]
        }
    }
}

impl PanelsPage {
    fn draw_nested(&self, ui: &mut Ui<'_>, body: Rect) {
        let group = Rect {
            y: body.y.saturating_add(1),
            width: body.width.min(30),
            height: body.height.saturating_sub(1).min(3),
            ..body
        };
        nested_list().draw(ui, group, &self.nested, TARGETS);
        if ui.theme().capability.color == termrock::ColorLevel::Mono && group.height >= 3 {
            let row = Rect {
                x: group.x,
                y: group.y.saturating_add(2),
                width: group.width,
                height: 1,
            };
            let row_style = panel_style(ui, FgStep::Faint).add_modifier(termrock::Modifier::DIM);
            ui.fill(row, row_style);
            let gutter = Rect {
                x: row.x,
                y: row.y,
                width: 1,
                height: 1,
            };
            let gutter_style = row_style.with_fg_from_bg(row_style);
            let _ = ui.paint_str(gutter, " ", gutter_style);
            let label = Rect {
                x: row.x.saturating_add(3),
                y: row.y,
                width: 5.min(row.width.saturating_sub(3)),
                height: 1,
            };
            let _ = ui.paint_str(label, "Cloud", row_style);
        }
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
    }
}
