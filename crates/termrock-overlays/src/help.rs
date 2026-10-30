use core::fmt;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::intent::Intent;
use crate::keymap::{Binding, BindingState, Bindings, HintLayer};
use crate::layer::{Anchor, DismissReason, LayerEvent, LayerSize, LayerSpec, ScreenAlign};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::ScrollState;
use crate::text::width;
use crate::theme::{Family, GlyphRole, Modifier, Role, StylePatch, Surface, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_core::geometry::{Axes, Headroom};
use termrock_feedback::keyhint::HintText;
use termrock_runtime::scroll_region::ScrollRegion;

#[derive(Clone, Copy, Debug)]
pub struct HelpSection<'a> {
    title: &'a str,
    layer: Option<&'a HintLayer>,
    rows: &'a [(&'a str, &'a str)],
}

impl<'a> HelpSection<'a> {
    pub const fn new(title: &'a str, layer: &'a HintLayer) -> Self {
        HelpSection {
            title,
            layer: Some(layer),
            rows: &[],
        }
    }

    /// Fixed key rows. Same action text the baseline help section shows.
    pub const fn plain(title: &'a str, rows: &'a [(&'a str, &'a str)]) -> Self {
        HelpSection {
            title,
            layer: None,
            rows,
        }
    }

    pub const fn title(&self) -> &'a str {
        self.title
    }

    pub const fn layer(&self) -> &HintLayer {
        match self.layer {
            Some(layer) => layer,
            None => panic!("help section has explicit rows, not a hint layer"),
        }
    }

    fn entries(&self) -> Vec<(String, String)> {
        if !self.rows.is_empty() {
            return self
                .rows
                .iter()
                .map(|(key, action)| ((*key).to_owned(), (*action).to_owned()))
                .collect();
        }
        let Some(layer) = self.layer else {
            return Vec::new();
        };
        layer
            .hints
            .iter()
            .map(|hint| {
                (
                    HintText::of(hint.key).as_str().to_owned(),
                    hint.label.to_owned(),
                )
            })
            .collect()
    }

    const fn rows(&self) -> usize {
        let n = if !self.rows.is_empty() {
            self.rows.len()
        } else if let Some(layer) = self.layer {
            layer.hints.len()
        } else {
            0
        };
        n.saturating_add(2)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct HelpOverlayState {
    scroll: ScrollState,
}

impl HelpOverlayState {
    pub const fn scroll(&self) -> &ScrollState {
        &self.scroll
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HelpAction {
    Closed(DismissReason),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HelpCmd {
    Prev,
    Next,
    PageUp,
    PageDown,
    Close,
}

const fn binding(
    action: crate::ActionKey,
    chord: Chord,
    cmd: HelpCmd,
    label: &'static str,
    visible: bool,
) -> Binding<HelpCmd> {
    Binding {
        action,
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 70 } else { 10 },
        visible,
    }
}

const BINDINGS: &[Binding<HelpCmd>] = &[
    binding(
        crate::ActionKey::custom("help.prev.up"),
        Chord::key(KeyCode::Up),
        HelpCmd::Prev,
        "Scroll",
        true,
    ),
    binding(
        crate::ActionKey::custom("help.prev.k"),
        Chord::key(KeyCode::Char('k')),
        HelpCmd::Prev,
        "Scroll",
        false,
    ),
    binding(
        crate::ActionKey::custom("help.next.down"),
        Chord::key(KeyCode::Down),
        HelpCmd::Next,
        "Scroll",
        false,
    ),
    binding(
        crate::ActionKey::custom("help.next.j"),
        Chord::key(KeyCode::Char('j')),
        HelpCmd::Next,
        "Scroll",
        false,
    ),
    binding(
        crate::ActionKey::custom("help.page-up"),
        Chord::key(KeyCode::PageUp),
        HelpCmd::PageUp,
        "Page",
        false,
    ),
    binding(
        crate::ActionKey::custom("help.page-down"),
        Chord::key(KeyCode::PageDown),
        HelpCmd::PageDown,
        "Page",
        false,
    ),
    binding(
        crate::ActionKey::custom("help.close.question"),
        Chord::key(KeyCode::Char('?')),
        HelpCmd::Close,
        "Close",
        true,
    ),
    binding(
        crate::ActionKey::custom("help.close.q"),
        Chord::key(KeyCode::Char('q')),
        HelpCmd::Close,
        "Close",
        false,
    ),
    binding(
        crate::ActionKey::custom("help.close.enter"),
        Chord::key(KeyCode::Enter),
        HelpCmd::Close,
        "Close",
        false,
    ),
];

pub struct HelpOverlay<'a> {
    id: Id,
    scope: &'a str,
    sections: &'a [HelpSection<'a>],
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
}

impl fmt::Debug for HelpOverlay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HelpOverlay")
            .field("id", &self.id)
            .field("scope", &self.scope)
            .field("sections", &self.sections.len())
            .finish_non_exhaustive()
    }
}

impl<'a> HelpOverlay<'a> {
    pub const COLUMN_WIDTH: u16 = 36;
    pub const MAX_COLUMNS: usize = 3;

    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::BORDER,
        Part::TITLE,
        Part::DETAIL,
        Part::HEADER,
        Part::KEY,
        Part::ACTION,
        Part::TRACK,
        Part::THUMB,
    ];

    pub const fn new(id: Id, scope: &'a str, sections: &'a [HelpSection<'a>]) -> Self {
        HelpOverlay {
            id,
            scope,
            sections,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
        }
    }

    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self.ov = self.ov.global(patch);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.parts = patches;
        self.ov = self.ov.part(patches);
        self
    }

    #[must_use]
    pub const fn slot(mut self, part: Part, slot: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, slot);
        self
    }

    pub fn measured_size(&self, cx: &Cx<'_>) -> LayerSize {
        // Baseline HelpOverlay::max_percent is 80, with a 4-col and 3-row inset.
        let view = cx.viewport();
        let w = view
            .width
            .saturating_sub(4)
            .min(percent80(view.width))
            .max(16);
        let h = view
            .height
            .saturating_sub(3)
            .min(percent80(view.height))
            .max(8);
        LayerSize::Fixed(w, h)
    }

    pub fn layer(&self, cx: &Cx<'_>) -> LayerSpec {
        LayerSpec::modal(self.id)
            .anchor(Anchor::Screen(ScreenAlign::GeometricCenter))
            .initial_focus(self.done_id())
            .size(self.measured_size(cx))
    }

    fn done_id(&self) -> Id {
        self.id.part(Part::ACTIONS)
    }

    fn column_rows(&self, cols: usize) -> usize {
        (0..cols)
            .map(|column| {
                self.sections
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| index.checked_rem(cols) == Some(column))
                    .map(|(_, section)| section.rows())
                    .sum()
            })
            .max()
            .unwrap_or(0)
    }

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut HelpOverlayState) -> Response<HelpAction> {
        if cx.is_open(self.id) {
            cx.resize_layer(self.id, self.measured_size(cx));
        }
        let columns = cx
            .area(self.id)
            .map_or(1, |area| columns_for(area.width.saturating_sub(2)));
        let content_len = self.column_rows(columns);
        let scroll = self.scrollbar().update(cx, &mut st.scroll, content_len);
        let mut acc = Acc::new();
        acc.fold(&scroll);
        for intent in cx.intents(self.id) {
            let response = match intent {
                Intent::Layer(LayerEvent::Dismissed(reason)) => {
                    Response::action(HelpAction::Closed(reason))
                }
                Intent::Layer(_) => Response::ignored().repaint(),
                Intent::Cancel => {
                    cx.close_layer(self.id, None);
                    Response::action(HelpAction::Closed(DismissReason::Esc))
                }
                Intent::Binding(action) => match Binding::command(BINDINGS, action) {
                    Some(HelpCmd::Prev) => scroll_by(&mut st.scroll, -1),
                    Some(HelpCmd::Next) => scroll_by(&mut st.scroll, 1),
                    Some(HelpCmd::PageUp) => page(&mut st.scroll, false),
                    Some(HelpCmd::PageDown) => page(&mut st.scroll, true),
                    Some(HelpCmd::Close) => {
                        cx.close_layer(self.id, None);
                        Response::action(HelpAction::Closed(DismissReason::Programmatic))
                    }
                    None => Response::ignored(),
                },
                _ => Response::ignored(),
            };
            if let Some(action) = response.action_ref().copied() {
                acc.action(action);
            } else {
                acc.fold(&response.erase());
            }
        }
        acc.finish(self.id)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &HelpOverlayState) -> Rect {
        if area.is_empty() {
            return area;
        }
        let live = ui.state(self.id);
        ui.with_surface(Surface::Elevated, |ui| {
            self.paint_card(ui, area, live);
            let border_style = self
                .ov
                .style(
                    ui,
                    self.id,
                    Family::HELP,
                    Variant::DEFAULT,
                    Part::BORDER,
                    live,
                )
                .style;
            let _frame_inner = ui.frame(area, border_style);
            ui.register_control(self.id, area, Focusability::Focusable);
            ui.publish_bindings(self.id, live, BINDINGS);
            ui.register_decor(self.id, PartRef::of(Part::BORDER), area);
            if area.width < 16 || area.height < 8 {
                return;
            }
            self.paint_title(ui, area, live);
            let inner = Rect {
                x: area.x.saturating_add(3),
                y: area.y.saturating_add(1),
                width: area.width.saturating_sub(6),
                height: area.height.saturating_sub(2),
            };
            if inner.is_empty() {
                return;
            }
            let body_h = inner.height.saturating_sub(1);
            let columns = self.columns(inner.width);
            let total = columns.iter().map(Vec::len).max().unwrap_or(0);
            let offset = st.scroll.offset();
            self.paint_columns(ui, inner, &columns, offset, usize::from(body_h));
            self.paint_scroll_label(ui, area, offset, usize::from(body_h), total);
            self.paint_close(ui, area);
            let mut faded = ScrollState::new(total);
            faded.set_viewport(usize::from(body_h));
            let fade_area = Rect {
                x: area.x,
                y: inner.y,
                width: area.right().saturating_sub(2).saturating_sub(area.x),
                height: body_h,
            };
            ui.scroll_edges(fade_area, &faded);
            self.paint_scrollbar(ui, area, inner, usize::from(body_h), total);
            self.paint_done(ui, inner);
            let _ = self.scope;
        });
        area
    }

    fn scrollbar(&self) -> ScrollRegion<'a> {
        let mut scrollbar = ScrollRegion::new(self.id)
            .inherit_family(Family::HELP)
            .patch_part(self.parts);
        if let Some(patch) = self.patch {
            scrollbar = scrollbar.patch(patch);
        }
        if let Some(slot) = self.ov.slot_for(Part::TRACK) {
            scrollbar = scrollbar.slot(Part::TRACK, slot);
        } else if let Some(slot) = self.ov.slot_for(Part::THUMB) {
            scrollbar = scrollbar.slot(Part::THUMB, slot);
        }
        scrollbar
    }

    fn paint_card(&self, ui: &mut Ui<'_>, area: Rect, live: StateFlags) {
        let container = self.ov.style(
            ui,
            self.id,
            Family::HELP,
            Variant::DEFAULT,
            Part::CONTAINER,
            live,
        );
        ui.fill_card_from_page(
            area,
            container
                .style
                .bg
                .unwrap_or(ratatui_core::style::Color::Rgb(0x18, 0x18, 0x1b)),
        );
    }

    fn paint_title(&self, ui: &mut Ui<'_>, area: Rect, live: StateFlags) {
        let style = self
            .ov
            .style(
                ui,
                self.id,
                Family::HELP,
                Variant::DEFAULT,
                Part::TITLE,
                live,
            )
            .style;
        let title = format!(" {} ", "Keyboard shortcuts");
        ui.paint_str(
            Rect {
                x: area.x.saturating_add(2),
                y: area.y,
                width: area.width.saturating_sub(2),
                height: 1,
            },
            &title,
            style,
        );
    }

    fn paint_close(&self, ui: &mut Ui<'_>, area: Rect) {
        let x = area.right().saturating_sub(5);
        let border = ink(ui, 0x004d_4d4d, 0x0018_181b, false);
        let mark = ink(ui, 0x00b3_b3b3, 0x0018_181b, false);
        let mut row = |dx, text, style| {
            ui.paint_str(
                Rect {
                    x: x.saturating_add(dx),
                    y: area.y,
                    width: 1,
                    height: 1,
                },
                text,
                style,
            );
        };
        row(0, " ", border);
        row(1, "×", mark);
        row(2, " ", border);
    }

    fn paint_scroll_label(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        offset: usize,
        viewport: usize,
        total: usize,
    ) {
        if total <= viewport || viewport == 0 {
            return;
        }
        let end = offset.saturating_add(viewport).min(total);
        let label = format!(" {}–{} of {} ", offset.saturating_add(1), end, total);
        let pw = width(&label);
        // Close cluster plus the corner rule sit in the last five cells.
        let x = area.right().saturating_sub(pw.saturating_add(6));
        ui.paint_str(
            Rect {
                x,
                y: area.y,
                width: pw,
                height: 1,
            },
            &label,
            ink(ui, 0x0073_7373, 0x0018_181b, false),
        );
    }

    fn columns(&self, inner_width: u16) -> Vec<Vec<Line>> {
        let cols = (inner_width / HelpOverlay::COLUMN_WIDTH).clamp(1, 3) as usize;
        let col_w = inner_width / cols as u16;
        let widest = self
            .sections
            .iter()
            .flat_map(|section| section.entries().into_iter().map(|(key, _)| width(&key)))
            .max()
            .unwrap_or(0);
        let key_w = widest.clamp(4, col_w.saturating_mul(2) / 5);
        let action_w = col_w.saturating_sub(key_w.saturating_add(3)).max(8);
        let mut blocks = Vec::new();
        for section in self.sections {
            let mut block = vec![Line {
                key: section.title.to_owned(),
                action: String::new(),
                heading: true,
            }];
            for (key, action) in section.entries() {
                for (i, line) in wrap_words(&action, action_w).into_iter().enumerate() {
                    block.push(Line {
                        key: if i == 0 {
                            key.to_owned()
                        } else {
                            String::new()
                        },
                        action: line,
                        heading: false,
                    });
                }
            }
            block.push(Line {
                key: String::new(),
                action: String::new(),
                heading: false,
            });
            blocks.push(block);
        }
        let mut columns: Vec<Vec<Line>> = Vec::with_capacity(cols);
        for _ in 0..cols {
            columns.push(Vec::new());
        }
        for (i, block) in blocks.into_iter().enumerate() {
            columns[i % cols].extend(block);
        }
        let _ = key_w;
        columns
    }

    fn paint_columns(
        &self,
        ui: &mut Ui<'_>,
        inner: Rect,
        columns: &[Vec<Line>],
        offset: usize,
        viewport: usize,
    ) {
        let cols = columns.len().max(1) as u16;
        let col_w = inner.width / cols;
        let widest = columns
            .iter()
            .flat_map(|column| {
                column
                    .iter()
                    .filter(|line| !line.heading && !line.key.is_empty())
                    .map(|line| width(&line.key))
            })
            .max()
            .unwrap_or(0);
        let key_w = widest.clamp(4, col_w.saturating_mul(2) / 5);
        let head = ink(ui, 0x00b3_b3b3, 0x0018_181b, true);
        let key_style = ink(ui, 0x00ff_ffff, 0x0018_181b, true);
        let action_style = ink(ui, 0x00b3_b3b3, 0x0018_181b, false);
        for (ci, column) in columns.iter().enumerate() {
            let x = inner.x.saturating_add(ci as u16 * col_w);
            for (k, line) in column.iter().skip(offset).take(viewport).enumerate() {
                let y = inner.y.saturating_add(k as u16);
                if line.heading {
                    ui.paint_str(cell_row(x, y, col_w), &line.key, head);
                } else if !line.key.is_empty() || !line.action.is_empty() {
                    let shown = truncate_key(&line.key, key_w);
                    ui.paint_str(cell_row(x, y, key_w), &shown, key_style);
                    ui.paint_str(
                        cell_row(
                            x.saturating_add(key_w).saturating_add(1),
                            y,
                            col_w.saturating_sub(key_w).saturating_sub(1),
                        ),
                        &line.action,
                        action_style,
                    );
                }
            }
        }
    }

    fn paint_scrollbar(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        inner: Rect,
        viewport: usize,
        total: usize,
    ) {
        if total <= viewport || viewport == 0 {
            return;
        }
        let track_h = u16::try_from(viewport)
            .unwrap_or(u16::MAX)
            .min(inner.height);
        let x = area.right().saturating_sub(2);
        let thumb = (viewport.saturating_mul(usize::from(track_h)) / total).max(1) as u16;
        let thumb = thumb.min(track_h);
        let thumb_style = ink(ui, 0x0080_8080, 0x0018_181b, false);
        let track_style = ink(ui, 0x0047_4747, 0x0018_181b, false);
        for row in 0..track_h {
            let style = if row < thumb {
                thumb_style
            } else {
                track_style
            };
            let glyph = if row < thumb { "┃" } else { "┆" };
            ui.paint_str(cell_row(x, inner.y.saturating_add(row), 1), glyph, style);
        }
        let track_rect = Rect {
            x,
            y: inner.y,
            width: 1,
            height: track_h,
        };
        ui.register_part(self.id, PartRef::of(Part::TRACK), track_rect);
        ui.register_scroll(self.id, area, Axes::V, Headroom::default());
    }

    fn paint_done(&self, ui: &mut Ui<'_>, inner: Rect) {
        let label = "Done ";
        let bw = width(label) as u16;
        let x = inner.right().saturating_sub(bw);
        let y = inner.bottom().saturating_sub(1);
        let gutter = ink(ui, 0x0048_e054, 0x0027_272a, false);
        let face = ink(ui, 0x00ff_ffff, 0x0027_272a, true);
        ui.glyph(
            cell_row(x.saturating_sub(1), y, 1),
            GlyphRole::FocusBar,
            gutter,
        );
        ui.paint_str(cell_row(x, y, bw), label, face);
        ui.register_control(self.done_id(), cell_row(x, y, bw), Focusability::Focusable);
    }

    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let width = ui.design().size.dialog_width_wide;
        let columns = columns_for(width.saturating_sub(2));
        let height = self.column_rows(columns).min(usize::from(u16::MAX)) as u16;
        Size::exact(width, height.saturating_add(4)).fit(c)
    }
}

impl Bindings for HelpOverlay<'_> {
    type Cmd = HelpCmd;

    fn bindings(&self, _state: BindingState) -> &'static [Binding<HelpCmd>] {
        BINDINGS
    }
}

struct Line {
    key: String,
    action: String,
    heading: bool,
}

fn percent80(n: u16) -> u16 {
    u16::try_from(u32::from(n) * 80 / 100).unwrap_or(u16::MAX)
}

fn rgb(hex: u32) -> ratatui_core::style::Color {
    ratatui_core::style::Color::Rgb(
        u8::try_from((hex >> 16) & 0xff).unwrap_or(0),
        u8::try_from((hex >> 8) & 0xff).unwrap_or(0),
        u8::try_from(hex & 0xff).unwrap_or(0),
    )
}

fn ink(ui: &Ui<'_>, fg: u32, bg: u32, bold: bool) -> crate::theme::PaintStyle {
    let mut patch = StylePatch::new()
        .set_fg(Role::Custom(rgb(fg)))
        .set_bg(Role::Custom(rgb(bg)));
    if bold {
        patch = patch.add(Modifier::BOLD);
    }
    ui.paint_patch(&patch)
}

fn cell_row(x: u16, y: u16, width: u16) -> Rect {
    Rect {
        x,
        y,
        width,
        height: 1,
    }
}

fn truncate_key(text: &str, limit: u16) -> String {
    if width(text) <= limit {
        return text.to_owned();
    }
    text.chars().take(usize::from(limit)).collect()
}

fn wrap_words(text: &str, limit: u16) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0u16;
    for word in text.split(' ') {
        let word_width = width(word);
        if used == 0 {
            line = word.to_owned();
            used = word_width;
        } else if used.saturating_add(1).saturating_add(word_width) <= limit {
            line.push(' ');
            line.push_str(word);
            used = used.saturating_add(1).saturating_add(word_width);
        } else {
            lines.push(std::mem::take(&mut line));
            line = word.to_owned();
            used = word_width;
        }
    }
    lines.push(line);
    lines
}

fn columns_for(width: u16) -> usize {
    usize::from((width / HelpOverlay::COLUMN_WIDTH).clamp(1, HelpOverlay::MAX_COLUMNS as u16))
}

fn scroll_by(st: &mut ScrollState, delta: isize) -> Response<HelpAction> {
    let before = st.offset();
    st.scroll_by(delta);
    moved(st.offset() != before)
}

fn page(st: &mut ScrollState, down: bool) -> Response<HelpAction> {
    let before = st.offset();
    if down {
        st.page_down();
    } else {
        st.page_up();
    }
    moved(st.offset() != before)
}

fn moved(changed: bool) -> Response<HelpAction> {
    if changed {
        Response::changed()
    } else {
        Response::consumed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::KeyCode;

    #[derive(Clone, Copy)]
    enum TestCmd {
        Run,
    }

    const TEST_BINDINGS: &[Binding<TestCmd>] = &[
        Binding {
            action: crate::ActionKey::custom("help.test.run"),
            chord: Some(Chord::key(KeyCode::Char('r'))),
            cmd: TestCmd::Run,
            label: "Run",
            priority: 50,
            visible: true,
        },
        Binding {
            action: crate::ActionKey::custom("help.test.hidden"),
            chord: Some(Chord::key(KeyCode::Char('x'))),
            cmd: TestCmd::Run,
            label: "Hidden",
            priority: 10,
            visible: false,
        },
    ];

    #[test]
    fn sections_consume_the_same_visible_binding_metadata_as_hintbar() {
        let layer = HintLayer::from_bindings(TEST_BINDINGS);
        let section = HelpSection::new("Actions", &layer);
        assert_eq!(section.layer().hints.len(), 1);
        assert_eq!(section.layer().hints[0].label, "Run");
    }

    #[test]
    fn sections_are_distributed_round_robin_without_splitting() {
        let layer = HintLayer::from_bindings(TEST_BINDINGS);
        let sections = [
            HelpSection::new("A", &layer),
            HelpSection::new("B", &layer),
            HelpSection::new("C", &layer),
            HelpSection::new("D", &layer),
        ];
        let overlay = HelpOverlay::new(Id::root("help.tests"), "scope", &sections);
        assert_eq!(overlay.column_rows(2), 6);
        assert_eq!(overlay.column_rows(3), 6);
    }

    #[test]
    fn column_count_is_bounded_from_one_to_three() {
        assert_eq!(columns_for(0), 1);
        assert_eq!(columns_for(72), 2);
        assert_eq!(columns_for(u16::MAX), 3);
    }
}
